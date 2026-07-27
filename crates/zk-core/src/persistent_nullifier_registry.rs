//! Versión persistente de `NullifierRegistry`.
//!
//! `nullifier::NullifierRegistry` (en memoria) modela correctamente la
//! LÓGICA de "rechazar si ya se usó", pero se documentó desde el principio
//! como una limitación real: si el proceso del validador se reinicia, se
//! "olvida" de qué nullifiers ya se gastaron — lo cual anula la protección
//! contra doble gasto en la práctica, no solo en teoría.
//!
//! Este módulo cierra ese hueco usando `sled` (base de datos embebida,
//! pura en Rust, sin dependencias de sistema) como almacenamiento
//! duradero. La propiedad que importa demostrar, y que el test principal
//! de este archivo comprueba explícitamente, es que un nullifier marcado
//! como gastado SIGUE marcado como gastado incluso después de cerrar y
//! volver a abrir la base de datos — es decir, después de un reinicio del
//! proceso, no solo dentro de la misma ejecución.
//!
//! ## Por qué `sled::insert` es la primitiva correcta aquí
//!
//! `Tree::insert(key, value)` en sled es una operación ATÓMICA que además
//! devuelve el valor anterior si la clave ya existía (`Ok(Some(old))`) o
//! `Ok(None)` si la clave era nueva. Esto nos da gratis exactamente la
//! semántica de "comprobar y marcar en una sola operación" que
//! necesitamos para evitar condiciones de carrera del tipo
//! "leer-luego-escribir" con dos operaciones separadas — el mismo
//! principio que ya se aplicó en la versión en memoria, pero aquí
//! garantizado por la propia base de datos, no por un `Mutex` en proceso.

use ark_serialize::CanonicalSerialize;

use crate::nullifier::NullifierError;

/// Registro de nullifiers gastados, respaldado en disco.
pub struct PersistentNullifierRegistry {
    db: sled::Db,
}

impl PersistentNullifierRegistry {
    /// Abre (o crea si no existe) la base de datos en `path`.
    pub fn open(path: &str) -> Result<Self, NullifierError> {
        let db = sled::open(path)
            .map_err(|e| NullifierError::StorageError(format!("no se pudo abrir la base de datos: {e}")))?;
        Ok(Self { db })
    }

    /// Serializa el nullifier a bytes canónicos, para usarlo como clave.
    /// Debe ser la MISMA representación siempre para el mismo valor de
    /// campo, o dos serializaciones distintas del mismo nullifier lógico
    /// no se reconocerían como iguales.
    fn nullifier_key<F: CanonicalSerialize>(nullifier: &F) -> Result<Vec<u8>, NullifierError> {
        let mut bytes = Vec::new();
        nullifier
            .serialize_compressed(&mut bytes)
            .map_err(|e| NullifierError::StorageError(format!("fallo al serializar el nullifier: {e}")))?;
        Ok(bytes)
    }

    /// Comprueba y marca como gastado en una sola operación atómica
    /// respaldada en disco. Si el nullifier ya estaba marcado (de esta
    /// ejecución o de una anterior, tras un reinicio), devuelve
    /// `NullifierError::AlreadySpent` y NO se debe aceptar la transacción.
    pub fn check_and_mark_spent<F: CanonicalSerialize>(&self, nullifier: &F) -> Result<(), NullifierError> {
        let key = Self::nullifier_key(nullifier)?;

        let previous = self
            .db
            .insert(key, &[1u8])
            .map_err(|e| NullifierError::StorageError(format!("fallo al escribir en la base de datos: {e}")))?;

        if previous.is_some() {
            return Err(NullifierError::AlreadySpent);
        }

        // Flush síncrono: la corrección de la protección contra doble
        // gasto depende de que "gastado" sea duradero de verdad, no solo
        // de que sled lo tenga en un buffer en memoria todavía sin volcar
        // a disco. Esto es más lento que dejar que sled decida cuándo
        // volcar, pero es la opción correcta aquí: preferir seguridad a
        // rendimiento en la escritura que decide si una transacción se
        // acepta o se rechaza.
        self.db
            .flush()
            .map_err(|e| NullifierError::StorageError(format!("fallo al persistir en disco: {e}")))?;

        Ok(())
    }

    /// Comprueba si un nullifier ya está marcado como gastado, sin
    /// modificarlo.
    pub fn is_spent<F: CanonicalSerialize>(&self, nullifier: &F) -> Result<bool, NullifierError> {
        let key = Self::nullifier_key(nullifier)?;
        let exists = self
            .db
            .contains_key(key)
            .map_err(|e| NullifierError::StorageError(format!("fallo al consultar la base de datos: {e}")))?;
        Ok(exists)
    }

    /// Número de nullifiers registrados como gastados. Principalmente para
    /// telemetría/depuración.
    pub fn spent_count(&self) -> usize {
        self.db.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nullifier::compute_nullifier;
    use ark_bls12_381::Fr;
    use std::path::PathBuf;

    /// Ruta temporal única por test, para no pisarse entre ejecuciones ni
    /// dejar basura persistente de una ejecución a otra.
    fn temp_db_path(test_name: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "zk_ssl_nullifier_test_{}_{}",
            test_name,
            std::process::id()
        ));
        p
    }

    /// EL TEST CLAVE de este módulo: un nullifier marcado como gastado
    /// debe seguir estando marcado después de CERRAR la base de datos y
    /// volver a ABRIRLA — simulando un reinicio del proceso del validador.
    /// Esto es exactamente lo que `NullifierRegistry` (en memoria) NO
    /// puede garantizar, y es la razón de ser de este módulo.
    #[test]
    fn nullifier_remains_spent_after_reopening_database() {
        let path = temp_db_path("reopen");
        let _ = std::fs::remove_dir_all(&path);

        let nullifier = compute_nullifier(Fr::from(111u64), Fr::from(1u64));

        {
            let registry = PersistentNullifierRegistry::open(path.to_str().unwrap())
                .expect("deberia poder abrir la base de datos nueva");
            registry
                .check_and_mark_spent(&nullifier)
                .expect("el primer uso del nullifier debe aceptarse");
        }
        // El registro anterior se cierra aqui (fin de scope / drop de sled::Db).

        {
            let registry = PersistentNullifierRegistry::open(path.to_str().unwrap())
                .expect("deberia poder reabrir la base de datos existente");

            assert!(
                registry.is_spent(&nullifier).expect("is_spent no deberia fallar"),
                "el nullifier deberia seguir marcado como gastado tras reabrir la base de datos"
            );

            let second_attempt = registry.check_and_mark_spent(&nullifier);
            assert!(
                matches!(second_attempt, Err(NullifierError::AlreadySpent)),
                "CRITICO: tras reiniciar el proceso, el registro persistente debio seguir \
                 rechazando la reutilizacion del nullifier. Si esto falla, la persistencia \
                 en disco no esta funcionando de verdad."
            );
        }

        let _ = std::fs::remove_dir_all(&path);
    }

    #[test]
    fn rejects_reuse_within_same_open_instance() {
        let path = temp_db_path("same_instance");
        let _ = std::fs::remove_dir_all(&path);

        let registry = PersistentNullifierRegistry::open(path.to_str().unwrap())
            .expect("deberia poder abrir la base de datos");
        let nullifier = compute_nullifier(Fr::from(222u64), Fr::from(1u64));

        assert!(registry.check_and_mark_spent(&nullifier).is_ok());
        assert_eq!(registry.spent_count(), 1);

        let second_attempt = registry.check_and_mark_spent(&nullifier);
        assert!(matches!(second_attempt, Err(NullifierError::AlreadySpent)));

        let _ = std::fs::remove_dir_all(&path);
    }

    #[test]
    fn different_nullifiers_do_not_collide() {
        let path = temp_db_path("no_collision");
        let _ = std::fs::remove_dir_all(&path);

        let registry = PersistentNullifierRegistry::open(path.to_str().unwrap())
            .expect("deberia poder abrir la base de datos");

        let n1 = compute_nullifier(Fr::from(1u64), Fr::from(1u64));
        let n2 = compute_nullifier(Fr::from(2u64), Fr::from(1u64));

        assert!(registry.check_and_mark_spent(&n1).is_ok());
        assert!(registry.check_and_mark_spent(&n2).is_ok(), "un nullifier distinto no deberia verse afectado por el otro");
        assert_eq!(registry.spent_count(), 2);

        let _ = std::fs::remove_dir_all(&path);
    }
}
