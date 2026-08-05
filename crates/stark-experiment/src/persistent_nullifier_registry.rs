//! Registro de nullifiers persistente en disco para el lado STARK —
//! mismo diseño que `zk-core::persistent_nullifier_registry` y su
//! equivalente en Halo2, adaptado a la diferencia estructural real de
//! este backend.
//!
//! ## La diferencia con los otros dos registros
//!
//! En Groth16 y Halo2 el nullifier es UN escalar del cuerpo. Aquí es un
//! **digest de 4 elementos** de Goldilocks, porque `Rp64_256` es una
//! esponja cuyo digest ocupa las posiciones 4..8 del estado. La clave de
//! la base de datos son por tanto los 32 bytes de los cuatro elementos
//! concatenados en orden fijo (little-endian, el de `to_bytes()` del
//! campo), no 8.
//!
//! El orden importa y es parte del contrato de almacenamiento: cambiarlo
//! invalidaría todas las entradas previas de una base de datos existente,
//! y el registro dejaría de reconocer nullifiers ya gastados — un fallo
//! de doble gasto silencioso. Por eso está fijado explícitamente en
//! `nullifier_key` y no se deriva de ninguna representación automática.
//!
//! ## Por qué es seguro sin bloqueos explícitos
//!
//! `sled::Tree::insert` devuelve el valor anterior si la clave ya
//! existía. Eso convierte "comprobar si está gastado" y "marcarlo como
//! gastado" en UNA sola operación atómica, sin ventana de condición de
//! carrera entre ambas. Es la misma propiedad ya verificada en los otros
//! dos backends.
//!
//! ## Limitación honesta (idéntica en los tres backends)
//!
//! Es un registro de **un solo nodo**. Una federación real necesitaría
//! consenso distribuido sobre el conjunto de nullifiers gastados; esto
//! no lo proporciona y no debe presentarse como si lo hiciera.
//!
//! ## Y una limitación específica de este backend
//!
//! Los nullifiers de STARK viven en el campo Goldilocks y NO son
//! intercambiables con los de Groth16 (BLS12-381) ni con los de Halo2
//! (Pallas). Cada backend necesita su propio registro; compartir uno
//! sería un error de diseño, no una optimización. Ver la nota extensa en
//! `crates/settlement-prover/src/lib.rs`.

use crate::merkle::Digest;

pub struct PersistentNullifierRegistry {
    db: sled::Db,
}

#[derive(Debug, PartialEq, Eq)]
pub enum NullifierError {
    AlreadySpent,
    StorageError(String),
}

impl std::fmt::Display for NullifierError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NullifierError::AlreadySpent => {
                write!(
                    f,
                    "el nullifier ya fue usado: intento de doble gasto rechazado"
                )
            }
            NullifierError::StorageError(e) => {
                write!(f, "error de almacenamiento del registro de nullifiers: {e}")
            }
        }
    }
}
impl std::error::Error for NullifierError {}

impl PersistentNullifierRegistry {
    pub fn open(path: &str) -> Result<Self, NullifierError> {
        let db = sled::open(path).map_err(|e| {
            NullifierError::StorageError(format!("no se pudo abrir la base de datos: {e}"))
        })?;
        Ok(Self { db })
    }

    /// Serializa un digest de 4 elementos como 32 bytes.
    ///
    /// El orden (elemento 0 primero, y dentro de cada elemento la
    /// representación little-endian de `as_int()`) es parte del contrato
    /// de almacenamiento: cambiarlo invalidaría silenciosamente todas las
    /// entradas de una base de datos existente.
    fn nullifier_key(nullifier: &Digest) -> Vec<u8> {
        let mut key = Vec::with_capacity(32);
        for element in nullifier.iter() {
            key.extend_from_slice(&element.as_int().to_le_bytes());
        }
        key
    }

    /// Comprueba y marca como gastado en UNA sola operación atómica.
    pub fn check_and_mark_spent(&self, nullifier: &Digest) -> Result<(), NullifierError> {
        let key = Self::nullifier_key(nullifier);

        let previous = self
            .db
            .insert(key, &[1u8])
            .map_err(|e| NullifierError::StorageError(format!("fallo al escribir: {e}")))?;

        if previous.is_some() {
            return Err(NullifierError::AlreadySpent);
        }

        self.db.flush().map_err(|e| {
            NullifierError::StorageError(format!("fallo al persistir en disco: {e}"))
        })?;

        Ok(())
    }

    pub fn is_spent(&self, nullifier: &Digest) -> Result<bool, NullifierError> {
        let key = Self::nullifier_key(nullifier);
        self.db
            .contains_key(key)
            .map_err(|e| NullifierError::StorageError(format!("fallo al consultar: {e}")))
    }

    pub fn spent_count(&self) -> usize {
        self.db.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use winterfell::math::fields::f64::BaseElement;
    use crate::compliance_circuit::native_nullifier;
    use std::path::PathBuf;

    fn temp_db_path(test_name: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "stark_nullifier_test_{}_{}",
            test_name,
            std::process::id()
        ));
        p
    }

    fn cleanup(path: &PathBuf) {
        let _ = std::fs::remove_dir_all(path);
    }

    fn sample_nullifier(account: u64, nonce: u64) -> Digest {
        native_nullifier(BaseElement::new(account), BaseElement::new(nonce))
    }

    /// EL TEST CLAVE: el mismo nullifier no puede gastarse dos veces.
    #[test]
    fn double_spend_is_rejected() {
        let path = temp_db_path("double_spend");
        cleanup(&path);

        let registry = PersistentNullifierRegistry::open(path.to_str().unwrap())
            .expect("deberia poder abrirse");
        let nullifier = sample_nullifier(12345, 1);

        assert!(registry.check_and_mark_spent(&nullifier).is_ok());
        assert_eq!(
            registry.check_and_mark_spent(&nullifier),
            Err(NullifierError::AlreadySpent),
            "CRITICO: el segundo gasto del mismo nullifier debe rechazarse"
        );

        drop(registry);
        cleanup(&path);
    }

    /// Nullifiers distintos conviven sin interferirse.
    #[test]
    fn distinct_nullifiers_coexist() {
        let path = temp_db_path("distinct");
        cleanup(&path);

        let registry = PersistentNullifierRegistry::open(path.to_str().unwrap()).unwrap();

        // Misma cuenta con nonces distintos: el caso de uso real de
        // gastar varias veces desde la misma cuenta.
        assert!(registry.check_and_mark_spent(&sample_nullifier(12345, 1)).is_ok());
        assert!(registry.check_and_mark_spent(&sample_nullifier(12345, 2)).is_ok());
        // Y cuentas distintas.
        assert!(registry.check_and_mark_spent(&sample_nullifier(99999, 1)).is_ok());

        assert_eq!(registry.spent_count(), 3);

        drop(registry);
        cleanup(&path);
    }

    /// El rito de §165, en esta casa: tras soltar el registro, sled
    /// puede tardar en devolver el cerrojo (WouldBlock) — el primo del
    /// flake curado en zk-ssl, destapado por B-3b-i al mover el reloj.
    fn open_retry(path: &str) -> PersistentNullifierRegistry {
        for _ in 0..10 {
            match PersistentNullifierRegistry::open(path) {
                Ok(r) => return r,
                Err(_) => std::thread::sleep(std::time::Duration::from_millis(50)),
            }
        }
        PersistentNullifierRegistry::open(path).unwrap()
    }
    /// LA PROPIEDAD QUE JUSTIFICA QUE SEA PERSISTENTE: el registro
    /// sobrevive a un reinicio del proceso. Sin esto, reiniciar un nodo
    /// permitiría regastar todo lo anterior.
    #[test]
    fn spent_nullifiers_survive_restart() {
        let path = temp_db_path("restart");
        cleanup(&path);
        let path_str = path.to_str().unwrap().to_string();

        let nullifier = sample_nullifier(12345, 1);

        {
            let registry = open_retry(&path_str);
            registry.check_and_mark_spent(&nullifier).unwrap();
        } // se cierra la base de datos: simula el apagado del nodo

        {
            let registry = open_retry(&path_str);
            assert!(
                registry.is_spent(&nullifier).unwrap(),
                "CRITICO: tras reiniciar, el nullifier gastado debe seguir marcado"
            );
            assert_eq!(
                registry.check_and_mark_spent(&nullifier),
                Err(NullifierError::AlreadySpent),
                "CRITICO: reiniciar no debe permitir regastar"
            );
        }

        cleanup(&path);
    }

    /// Un nullifier nunca visto no está gastado.
    #[test]
    fn unseen_nullifier_is_not_spent() {
        let path = temp_db_path("unseen");
        cleanup(&path);

        let registry = PersistentNullifierRegistry::open(path.to_str().unwrap()).unwrap();
        assert!(!registry.is_spent(&sample_nullifier(777, 1)).unwrap());
        assert_eq!(registry.spent_count(), 0);

        drop(registry);
        cleanup(&path);
    }

    /// La clave debe distinguir digests que difieren en CUALQUIER
    /// elemento, incluido el último. Un error de serialización que solo
    /// mirara los primeros bytes provocaría colisiones y, con ellas,
    /// rechazos de gastos legítimos.
    #[test]
    fn key_distinguishes_all_four_elements() {
        let base: Digest = [
            BaseElement::new(1),
            BaseElement::new(2),
            BaseElement::new(3),
            BaseElement::new(4),
        ];
        let key_base = PersistentNullifierRegistry::nullifier_key(&base);
        assert_eq!(key_base.len(), 32, "la clave debe ocupar 32 bytes");

        for i in 0..4 {
            let mut variant = base;
            variant[i] = BaseElement::new(999);
            assert_ne!(
                PersistentNullifierRegistry::nullifier_key(&variant),
                key_base,
                "cambiar el elemento {i} debe producir una clave distinta"
            );
        }
    }
}
