//! Registro de nullifiers persistente en disco para el backend
//! PLONK-KZG — mismo diseño que en los otros tres backends.
//!
//! ## La diferencia con STARK
//!
//! Aquí el nullifier es **un solo escalar** (`BlsScalar`), como en
//! Groth16 y Halo2. En STARK era un digest de 4 elementos porque
//! `Rp64_256` es una esponja de estado 12. La clave son por tanto los 32
//! bytes de la representación canónica del escalar.
//!
//! ## Por qué es seguro sin bloqueos explícitos
//!
//! `sled::Tree::insert` devuelve el valor anterior si la clave ya
//! existía, lo que convierte "comprobar" y "marcar" en UNA operación
//! atómica, sin ventana de condición de carrera.
//!
//! ## Limitaciones (idénticas en los cuatro backends)
//!
//! Es un registro de **un solo nodo**. Una federación real necesitaría
//! consenso distribuido sobre el conjunto de nullifiers gastados.
//!
//! Y los nullifiers de este backend NO son intercambiables con los de
//! los otros tres: aunque `zk-core` también use BLS12-381, los
//! parámetros de Poseidon son distintos (Hades de dusk frente al de
//! arkworks), así que los valores no coinciden. Cada backend necesita su
//! propio registro.

use dusk_bytes::Serializable;
use dusk_plonk::prelude::BlsScalar;

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
            NullifierError::AlreadySpent => write!(
                f,
                "el nullifier ya fue usado: intento de doble gasto rechazado"
            ),
            NullifierError::StorageError(e) => {
                write!(f, "error de almacenamiento del registro: {e}")
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

    /// Clave de 32 bytes: la representación canónica del escalar.
    ///
    /// Es parte del contrato de almacenamiento. Cambiar la codificación
    /// invalidaría silenciosamente todas las entradas de una base de
    /// datos existente, y el registro dejaría de reconocer nullifiers ya
    /// gastados — un fallo de doble gasto sin ningún error visible.
    fn key(nullifier: &BlsScalar) -> [u8; 32] {
        nullifier.to_bytes()
    }

    /// Comprueba y marca como gastado en UNA operación atómica.
    pub fn check_and_mark_spent(&self, nullifier: &BlsScalar) -> Result<(), NullifierError> {
        let previous = self
            .db
            .insert(Self::key(nullifier), &[1u8])
            .map_err(|e| NullifierError::StorageError(format!("fallo al escribir: {e}")))?;

        if previous.is_some() {
            return Err(NullifierError::AlreadySpent);
        }

        self.db.flush().map_err(|e| {
            NullifierError::StorageError(format!("fallo al persistir en disco: {e}"))
        })?;
        Ok(())
    }

    pub fn is_spent(&self, nullifier: &BlsScalar) -> Result<bool, NullifierError> {
        self.db
            .contains_key(Self::key(nullifier))
            .map_err(|e| NullifierError::StorageError(format!("fallo al consultar: {e}")))
    }

    pub fn spent_count(&self) -> usize {
        self.db.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::poseidon_hash::native_nullifier;
    use std::path::PathBuf;

    fn temp_db_path(name: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("plonk_nullifier_test_{}_{}", name, std::process::id()));
        p
    }

    fn cleanup(path: &PathBuf) {
        let _ = std::fs::remove_dir_all(path);
    }

    fn sample(account: u64, nonce: u64) -> BlsScalar {
        native_nullifier(BlsScalar::from(account), BlsScalar::from(nonce))
    }

    /// EL TEST CLAVE: el mismo nullifier no puede gastarse dos veces.
    #[test]
    fn double_spend_is_rejected() {
        let path = temp_db_path("double_spend");
        cleanup(&path);

        let registry = PersistentNullifierRegistry::open(path.to_str().unwrap()).unwrap();
        let n = sample(12345, 1);

        assert!(registry.check_and_mark_spent(&n).is_ok());
        assert_eq!(
            registry.check_and_mark_spent(&n),
            Err(NullifierError::AlreadySpent),
            "CRITICO: el segundo gasto del mismo nullifier debe rechazarse"
        );

        drop(registry);
        cleanup(&path);
    }

    /// Nullifiers distintos conviven.
    #[test]
    fn distinct_nullifiers_coexist() {
        let path = temp_db_path("distinct");
        cleanup(&path);

        let registry = PersistentNullifierRegistry::open(path.to_str().unwrap()).unwrap();
        assert!(registry.check_and_mark_spent(&sample(12345, 1)).is_ok());
        assert!(registry.check_and_mark_spent(&sample(12345, 2)).is_ok());
        assert!(registry.check_and_mark_spent(&sample(99999, 1)).is_ok());
        assert_eq!(registry.spent_count(), 3);

        drop(registry);
        cleanup(&path);
    }

    /// LA PROPIEDAD QUE JUSTIFICA LA PERSISTENCIA: sobrevive al reinicio.
    /// Sin esto, reiniciar un nodo permitiría regastar todo lo anterior.
    #[test]
    fn spent_nullifiers_survive_restart() {
        let path = temp_db_path("restart");
        cleanup(&path);
        let path_str = path.to_str().unwrap().to_string();
        let n = sample(12345, 1);

        {
            let registry = PersistentNullifierRegistry::open(&path_str).unwrap();
            registry.check_and_mark_spent(&n).unwrap();
        } // se cierra: simula el apagado del nodo

        {
            let registry = PersistentNullifierRegistry::open(&path_str).unwrap();
            assert!(
                registry.is_spent(&n).unwrap(),
                "CRITICO: tras reiniciar, el nullifier gastado debe seguir marcado"
            );
            assert_eq!(
                registry.check_and_mark_spent(&n),
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
        assert!(!registry.is_spent(&sample(777, 1)).unwrap());
        assert_eq!(registry.spent_count(), 0);

        drop(registry);
        cleanup(&path);
    }

    /// La clave ocupa 32 bytes y distingue escalares distintos.
    #[test]
    fn key_is_canonical_and_distinguishing() {
        let a = BlsScalar::from(1u64);
        let b = BlsScalar::from(2u64);
        assert_eq!(PersistentNullifierRegistry::key(&a).len(), 32);
        assert_ne!(
            PersistentNullifierRegistry::key(&a),
            PersistentNullifierRegistry::key(&b)
        );
    }
}
