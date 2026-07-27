//! Nullifier: un valor público, derivado determinísticamente de la cuenta
//! y su contador de transacciones (`account_nonce`), que el circuito
//! expone junto a la prueba. El ledger/verificador mantiene un registro de
//! nullifiers ya usados y rechaza cualquier prueba que reutilice uno.
//!
//! Esto cierra el hueco documentado desde el principio: "no hay nullifier
//! ni prevención de doble gasto". Sin esto, alguien podía generar UNA
//! prueba válida de "tengo fondos suficientes" y reenviarla indefinidamente
//! como si fuera una transacción nueva cada vez.
//!
//! Se deriva con `secure_hash` (Poseidon real, ver `poseidon_hash.rs`) —
//! ya no usa `toy_hash`.
//!
//! ## Por qué la separación de dominio importa
//!
//! El nullifier NO puede calcularse con la misma composición exacta que la
//! hoja del árbol (`leaf_commitment`), porque si ambas fueran
//! `hash(hash(account_id, balance), nonce)` con la misma constante,
//! cualquiera podría confundir un valor con otro, o un atacante podría
//! intentar reutilizar la hoja como si fuera un nullifier válido (o
//! viceversa) en algún punto del sistema. Por eso se antepone una
//! constante de dominio distinta (`NULLIFIER_DOMAIN`) que ata el hash a un
//! propósito único. Esto es una práctica estándar en diseño de circuitos
//! ZK ("domain separation"), no un detalle cosmético.

use ark_crypto_primitives::sponge::Absorb;
use ark_ff::PrimeField;
use ark_r1cs_std::fields::fp::FpVar;
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{ConstraintSystemRef, SynthesisError};
use std::collections::HashSet;

use crate::poseidon_hash::{secure_hash, secure_hash_gadget};

/// Constante de separación de dominio para el nullifier. El valor exacto
/// es arbitrario; lo único que importa es que sea distinto de cualquier
/// otra constante de dominio usada en el sistema (aquí no hay ninguna
/// otra todavía, pero conviene dejar el patrón establecido desde ya).
pub const NULLIFIER_DOMAIN: u64 = 0x4E554C4C; // ASCII "NULL", solo mnemónico

/// Calcula el nullifier de forma nativa (fuera de circuito). Debe
/// coincidir EXACTAMENTE con `enforce_nullifier_computation`, igual que
/// `compute_leaf` debe coincidir con la versión en-circuito del árbol.
pub fn compute_nullifier<F: PrimeField + Absorb>(account_id: F, account_nonce: F) -> F {
    let domain = F::from(NULLIFIER_DOMAIN);
    secure_hash(secure_hash(domain, account_id), account_nonce)
}

/// Verifica en-circuito que `claimed_nullifier_var` (un valor PÚBLICO que
/// el probador declara) es efectivamente el nullifier correcto derivado de
/// `account_id_var` y `nonce_var` (ambos PRIVADOS). Esto es lo que impide
/// que alguien declare un nullifier arbitrario para evadir el registro de
/// gastados, o reutilice el nullifier de otra cuenta.
pub fn enforce_nullifier_computation<F: PrimeField + Absorb>(
    cs: ConstraintSystemRef<F>,
    account_id_var: &FpVar<F>,
    nonce_var: &FpVar<F>,
    claimed_nullifier_var: &FpVar<F>,
) -> Result<(), SynthesisError> {
    let domain_var = FpVar::<F>::Constant(F::from(NULLIFIER_DOMAIN));
    let inner = secure_hash_gadget(cs.clone(), &domain_var, account_id_var)?;
    let computed = secure_hash_gadget(cs, &inner, nonce_var)?;
    computed.enforce_equal(claimed_nullifier_var)?;
    Ok(())
}

/// Registro de nullifiers gastados. Esto vive FUERA del circuito: es la
/// pieza del lado del ledger/validador que decide si acepta o rechaza una
/// prueba entrante, comprobando si su nullifier ya fue usado antes.
///
/// Implementación actual: en memoria, no persistente, no distribuida. Para
/// un nodo real esto debe respaldarse en el `StateLedger` (o equivalente)
/// con las mismas garantías de atomicidad que cualquier otra escritura de
/// estado — aquí se modela solo la lógica de decisión, no la persistencia.
#[derive(Debug, Default)]
pub struct NullifierRegistry<F: PrimeField + std::hash::Hash> {
    spent: HashSet<F>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum NullifierError {
    AlreadySpent,
    /// Error de la capa de almacenamiento persistente (ver
    /// `persistent_nullifier_registry.rs`). No aplica a la versión en
    /// memoria de este archivo, que no puede fallar por I/O.
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

impl<F: PrimeField + std::hash::Hash> NullifierRegistry<F> {
    pub fn new() -> Self {
        Self {
            spent: HashSet::new(),
        }
    }

    pub fn is_spent(&self, nullifier: &F) -> bool {
        self.spent.contains(nullifier)
    }

    /// Comprueba y marca como gastado en una sola operación (para evitar
    /// condiciones de carrera del tipo "comprobar y luego marcar" con dos
    /// llamadas separadas). Devuelve error si el nullifier ya estaba
    /// gastado; en ese caso NO se acepta la transacción.
    pub fn check_and_mark_spent(&mut self, nullifier: F) -> Result<(), NullifierError> {
        if self.spent.contains(&nullifier) {
            return Err(NullifierError::AlreadySpent);
        }
        self.spent.insert(nullifier);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bls12_381::Fr;

    #[test]
    fn nullifier_is_deterministic() {
        let account_id = Fr::from(42u64);
        let nonce = Fr::from(3u64);

        let n1 = compute_nullifier(account_id, nonce);
        let n2 = compute_nullifier(account_id, nonce);
        assert_eq!(n1, n2, "el mismo account_id y nonce deben producir siempre el mismo nullifier");
    }

    #[test]
    fn different_nonce_produces_different_nullifier() {
        let account_id = Fr::from(42u64);
        let n1 = compute_nullifier(account_id, Fr::from(1u64));
        let n2 = compute_nullifier(account_id, Fr::from(2u64));
        assert_ne!(
            n1, n2,
            "avanzar el nonce de la cuenta debe cambiar el nullifier, \
             para que cada transaccion tenga uno distinto"
        );
    }

    #[test]
    fn registry_rejects_reused_nullifier() {
        let mut registry = NullifierRegistry::<Fr>::new();
        let nullifier = compute_nullifier(Fr::from(7u64), Fr::from(1u64));

        assert!(registry.check_and_mark_spent(nullifier).is_ok(), "el primer uso debe aceptarse");
        assert!(registry.is_spent(&nullifier));

        let second_attempt = registry.check_and_mark_spent(nullifier);
        assert_eq!(
            second_attempt,
            Err(NullifierError::AlreadySpent),
            "reutilizar el mismo nullifier (doble gasto) debe rechazarse"
        );
    }
}
