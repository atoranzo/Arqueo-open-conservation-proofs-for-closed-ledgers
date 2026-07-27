//! Registro de nullifiers persistente en disco, para el lado Halo2 —
//! mismo diseño que `zk-core::persistent_nullifier_registry`, adaptado
//! para serializar `halo2_proofs::pasta::Fp` (vía `ff::PrimeField::to_repr()`)
//! en vez de `ark_serialize::CanonicalSerialize`. La lógica de fondo
//! (insert atómico de `sled` como "comprobar y marcar" en una sola
//! operación) es idéntica y ya está verificada en la versión Groth16.

use ff::PrimeField;
use halo2_proofs::pasta::Fp;

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
                write!(f, "el nullifier ya fue usado: intento de doble gasto rechazado")
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
        let db = sled::open(path)
            .map_err(|e| NullifierError::StorageError(format!("no se pudo abrir la base de datos: {e}")))?;
        Ok(Self { db })
    }

    fn nullifier_key(nullifier: &Fp) -> Vec<u8> {
        nullifier.to_repr().as_ref().to_vec()
    }

    /// Comprueba y marca como gastado en una sola operación atómica
    /// (misma propiedad que la versión Groth16: `sled::Tree::insert`
    /// devuelve el valor anterior si la clave ya existía, dándonos
    /// "comprobar y marcar" sin condición de carrera).
    pub fn check_and_mark_spent(&self, nullifier: &Fp) -> Result<(), NullifierError> {
        let key = Self::nullifier_key(nullifier);

        let previous = self
            .db
            .insert(key, &[1u8])
            .map_err(|e| NullifierError::StorageError(format!("fallo al escribir: {e}")))?;

        if previous.is_some() {
            return Err(NullifierError::AlreadySpent);
        }

        self.db
            .flush()
            .map_err(|e| NullifierError::StorageError(format!("fallo al persistir en disco: {e}")))?;

        Ok(())
    }

    pub fn is_spent(&self, nullifier: &Fp) -> Result<bool, NullifierError> {
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
    use std::path::PathBuf;

    fn temp_db_path(test_name: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("halo2_nullifier_test_{}_{}", test_name, std::process::id()));
        p
    }

    /// EL TEST CLAVE: un nullifier marcado como gastado sigue estándolo
    /// tras cerrar y reabrir la base de datos — simulando un reinicio del
    /// proceso del validador. Misma propiedad que ya verificamos en la
    /// version Groth16.
    #[test]
    fn nullifier_remains_spent_after_reopening_database() {
        let path = temp_db_path("reopen");
        let _ = std::fs::remove_dir_all(&path);

        let nullifier = Fp::from(999_888_777u64);

        {
            let registry = PersistentNullifierRegistry::open(path.to_str().unwrap())
                .expect("deberia poder abrir la base de datos nueva");
            registry
                .check_and_mark_spent(&nullifier)
                .expect("el primer uso debe aceptarse");
        }

        {
            let registry = PersistentNullifierRegistry::open(path.to_str().unwrap())
                .expect("deberia poder reabrir la base de datos existente");

            assert!(registry.is_spent(&nullifier).unwrap());

            let second_attempt = registry.check_and_mark_spent(&nullifier);
            assert_eq!(
                second_attempt,
                Err(NullifierError::AlreadySpent),
                "CRITICO: tras reiniciar el proceso, el registro debio seguir rechazando el nullifier"
            );
        }

        let _ = std::fs::remove_dir_all(&path);
    }

    #[test]
    fn rejects_reuse_within_same_open_instance() {
        let path = temp_db_path("same_instance");
        let _ = std::fs::remove_dir_all(&path);

        let registry = PersistentNullifierRegistry::open(path.to_str().unwrap()).unwrap();
        let nullifier = Fp::from(111_222_333u64);

        assert!(registry.check_and_mark_spent(&nullifier).is_ok());
        assert_eq!(registry.spent_count(), 1);
        assert_eq!(
            registry.check_and_mark_spent(&nullifier),
            Err(NullifierError::AlreadySpent)
        );

        let _ = std::fs::remove_dir_all(&path);
    }

    #[test]
    fn different_nullifiers_do_not_collide() {
        let path = temp_db_path("no_collision");
        let _ = std::fs::remove_dir_all(&path);

        let registry = PersistentNullifierRegistry::open(path.to_str().unwrap()).unwrap();
        let n1 = Fp::from(1u64);
        let n2 = Fp::from(2u64);

        assert!(registry.check_and_mark_spent(&n1).is_ok());
        assert!(registry.check_and_mark_spent(&n2).is_ok());
        assert_eq!(registry.spent_count(), 2);

        let _ = std::fs::remove_dir_all(&path);
    }
}
