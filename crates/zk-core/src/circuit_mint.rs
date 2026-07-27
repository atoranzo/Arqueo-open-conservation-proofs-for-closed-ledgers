//! **Circuito de emisión.** La pieza que cierra la última puerta por la
//! que se podía crear dinero.
//!
//! ## El agujero que esto cierra
//!
//! `circuit_settlement` demuestra que el dinero **se conserva** en cada
//! transferencia — 15.522 restricciones dedicadas a ello. Pero
//! `SettlementLayer::open_account` creaba cuentas **con saldo, sin
//! ninguna prueba**. El operador del nodo podía abrir una cuenta con mil
//! millones y el sistema lo aceptaba.
//!
//! Toda la conservación es irrelevante si existe una puerta trasera para
//! crear dinero. Este circuito la cierra.
//!
//! ## El modelo: apertura ≠ emisión
//!
//! ```text
//! open_account(sk)      → cuenta con saldo CERO.
//!                         No crea dinero, no necesita prueba.
//! mint(issuer_key, ...) → aumenta un saldo.
//!                         REQUIERE prueba y clave del emisor.
//! ```
//!
//! Es el modelo real: los bancos comerciales mueven dinero, solo el
//! emisor lo crea.
//!
//! ## El suministro total, público y auditable
//!
//! `supply_old` y `supply_new` son entradas públicas, y el circuito
//! impone `supply_new = supply_old + amount`. Así:
//!
//! - **Cada emisión queda registrada** en una cifra pública que solo
//!   crece con emisiones demostradas.
//! - **Cualquiera puede auditar** que la suma de todos los saldos
//!   equivale al suministro emitido, sin ver ningún saldo concreto.
//! - **La conservación pasa a ser global**, no solo por transferencia.
//!
//! ## Qué demuestra
//!
//! 1. **Autoridad de emisión**: quien firma conoce `issuer_key` tal que
//!    `issuer_id = H(DOMAIN_ISSUER, issuer_key)`. `issuer_id` es público
//!    y fijo: es la identidad del banco central.
//! 2. **La cuenta existe** en el árbol, con el saldo declarado.
//! 3. **El saldo aumenta exactamente en `amount`**, y la raíz nueva lo
//!    refleja.
//! 4. **El suministro aumenta exactamente en `amount`.**
//!
//! ## Lo que NO resuelve
//!
//! - **No hay destrucción de dinero (burn).** Un sistema real necesita
//!   retirar circulante; ese sería el circuito simétrico.
//! - **No hay límite de emisión ni política monetaria.** El emisor puede
//!   emitir sin tope; imponer reglas (techos, calendarios, votaciones)
//!   sería otra capa.
//! - **La clave del emisor es única.** Un banco central real usaría
//!   umbral (m-de-n), no una sola clave.

use ark_bls12_381::Fr;
use ark_crypto_primitives::sponge::Absorb;
use ark_ff::PrimeField;
use ark_r1cs_std::alloc::AllocVar;
use ark_r1cs_std::boolean::Boolean;
use ark_r1cs_std::eq::EqGadget;
use ark_r1cs_std::fields::fp::FpVar;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};

use crate::circuit::enforce_range;
use crate::merkle::{compute_merkle_root, MerklePath, TREE_DEPTH};
use crate::poseidon_hash::{secure_hash, secure_hash_gadget};
use crate::proof_system::{
    prove_generic, setup_generic, verify_generic, ComplianceProof, ComplianceProvingKey,
    ComplianceVerifyingKey,
};
use crate::ZkCoreError;

/// Dominio de derivación de la identidad del emisor. Distinto del de las
/// cuentas: una clave de gasto no puede hacerse pasar por emisor ni al
/// revés.
pub const ISSUER_DOMAIN: u64 = 0x49535355; // "ISSU"

/// Identidad pública del emisor a partir de su clave.
pub fn derive_issuer_id<F: PrimeField + Absorb>(issuer_key: F) -> F {
    secure_hash(F::from(ISSUER_DOMAIN), issuer_key)
}

#[derive(Clone)]
pub struct MintCircuit<F: PrimeField> {
    // --- Testigos privados ---
    /// Clave de emisión. Solo el banco central la conoce.
    pub issuer_key: Option<F>,
    /// Identidad pública de la cuenta destinataria.
    pub account_public_id: Option<F>,
    pub account_balance: Option<u64>,
    pub account_nonce: Option<F>,
    pub merkle_path: Option<MerklePath<F>>,

    // --- Entradas públicas ---
    pub root_old: F,
    pub root_new: F,
    /// Identidad del emisor autorizado. Es un parámetro del sistema, fijo
    /// y conocido por todos.
    pub issuer_id: F,
    pub amount: F,
    pub supply_old: F,
    pub supply_new: F,
}

impl<F: PrimeField + Absorb> MintCircuit<F> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        issuer_key: F,
        account_public_id: F,
        account_balance: u64,
        account_nonce: F,
        merkle_path: MerklePath<F>,
        amount: u64,
        root_old: F,
        root_new: F,
        supply_old: u64,
    ) -> Self {
        Self {
            issuer_key: Some(issuer_key),
            account_public_id: Some(account_public_id),
            account_balance: Some(account_balance),
            account_nonce: Some(account_nonce),
            merkle_path: Some(merkle_path),
            root_old,
            root_new,
            issuer_id: derive_issuer_id(issuer_key),
            amount: F::from(amount),
            supply_old: F::from(supply_old),
            supply_new: F::from(supply_old + amount),
        }
    }

    pub fn empty_for_setup() -> Self {
        Self {
            issuer_key: None,
            account_public_id: None,
            account_balance: None,
            account_nonce: None,
            merkle_path: None,
            root_old: F::zero(),
            root_new: F::zero(),
            issuer_id: F::zero(),
            amount: F::zero(),
            supply_old: F::zero(),
            supply_new: F::zero(),
        }
    }
}

impl<F: PrimeField + Absorb> ConstraintSynthesizer<F> for MintCircuit<F> {
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        // --- Testigos ---
        let key_var =
            FpVar::new_witness(cs.clone(), || Ok(self.issuer_key.unwrap_or_else(F::zero)))?;
        let acc_id_var = FpVar::new_witness(cs.clone(), || {
            Ok(self.account_public_id.unwrap_or_else(F::zero))
        })?;
        let balance_var = FpVar::new_witness(cs.clone(), || {
            Ok(F::from(self.account_balance.unwrap_or(0)))
        })?;
        let nonce_var =
            FpVar::new_witness(cs.clone(), || Ok(self.account_nonce.unwrap_or_else(F::zero)))?;

        let (siblings, is_right) = match &self.merkle_path {
            Some(p) => (p.siblings.clone(), p.is_right.clone()),
            None => (vec![F::zero(); TREE_DEPTH], vec![false; TREE_DEPTH]),
        };
        let siblings_var = siblings
            .iter()
            .map(|s| FpVar::new_witness(cs.clone(), || Ok(*s)))
            .collect::<Result<Vec<_>, _>>()?;
        let bits_var = is_right
            .iter()
            .map(|b| Boolean::new_witness(cs.clone(), || Ok(*b)))
            .collect::<Result<Vec<_>, _>>()?;

        // --- Entradas públicas, EN ESTE ORDEN ---
        let root_old_var = FpVar::new_input(cs.clone(), || Ok(self.root_old))?;
        let root_new_var = FpVar::new_input(cs.clone(), || Ok(self.root_new))?;
        let issuer_id_var = FpVar::new_input(cs.clone(), || Ok(self.issuer_id))?;
        let amount_var = FpVar::new_input(cs.clone(), || Ok(self.amount))?;
        let supply_old_var = FpVar::new_input(cs.clone(), || Ok(self.supply_old))?;
        let supply_new_var = FpVar::new_input(cs.clone(), || Ok(self.supply_new))?;

        // ===== 1. AUTORIDAD DE EMISIÓN =====
        // Sin la clave del emisor es imposible satisfacer esto. Es lo que
        // impide que el operador del nodo cree dinero.
        let computed_issuer = secure_hash_gadget(
            cs.clone(),
            &FpVar::Constant(F::from(ISSUER_DOMAIN)),
            &key_var,
        )?;
        computed_issuer.enforce_equal(&issuer_id_var)?;

        // ===== 2. La cuenta existe con el saldo declarado =====
        let leaf_old = {
            let inner = secure_hash_gadget(cs.clone(), &acc_id_var, &balance_var)?;
            secure_hash_gadget(cs.clone(), &inner, &nonce_var)?
        };
        let computed_old = compute_merkle_root(cs.clone(), &leaf_old, &siblings_var, &bits_var)?;
        computed_old.enforce_equal(&root_old_var)?;

        // ===== 3. El saldo aumenta exactamente en `amount` =====
        let balance_new = &balance_var + &amount_var;
        let leaf_new = {
            let inner = secure_hash_gadget(cs.clone(), &acc_id_var, &balance_new)?;
            secure_hash_gadget(cs.clone(), &inner, &nonce_var)?
        };
        let computed_new = compute_merkle_root(cs.clone(), &leaf_new, &siblings_var, &bits_var)?;
        computed_new.enforce_equal(&root_new_var)?;

        // ===== 4. EL SUMINISTRO CRECE EXACTAMENTE EN `amount` =====
        // Aquí es donde la emisión queda registrada públicamente. La
        // conservación deja de ser local (por transferencia) y pasa a ser
        // global y auditable.
        let expected_supply = &supply_old_var + &amount_var;
        expected_supply.enforce_equal(&supply_new_var)?;

        // ===== 5. Rangos =====
        enforce_range(&balance_var)?;
        enforce_range(&amount_var)?;
        enforce_range(&balance_new)?; // el saldo no desborda
        enforce_range(&supply_old_var)?;
        enforce_range(&supply_new_var)?; // el suministro no desborda

        Ok(())
    }
}

// ---------------------------------------------------------------------
// Wrappers Groth16
// ---------------------------------------------------------------------

pub fn setup_mint(
    rng_seed: u64,
) -> Result<(ComplianceProvingKey, ComplianceVerifyingKey), ZkCoreError> {
    setup_generic(MintCircuit::<Fr>::empty_for_setup(), rng_seed)
}

pub fn prove_mint(
    pk: &ComplianceProvingKey,
    circuit: MintCircuit<Fr>,
    rng_seed: u64,
) -> Result<ComplianceProof, ZkCoreError> {
    prove_generic(pk, circuit, rng_seed)
}

#[allow(clippy::too_many_arguments)]
pub fn verify_mint(
    vk: &ComplianceVerifyingKey,
    proof: &ComplianceProof,
    root_old: Fr,
    root_new: Fr,
    issuer_id: Fr,
    amount: u64,
    supply_old: u64,
    supply_new: u64,
) -> Result<bool, ZkCoreError> {
    let inputs = vec![
        root_old,
        root_new,
        issuer_id,
        Fr::from(amount),
        Fr::from(supply_old),
        Fr::from(supply_new),
    ];
    verify_generic(vk, proof, &inputs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuit_settlement::{account_id_from_key, account_leaf};
    use crate::merkle::SimpleMerkleTree;
    use ark_ff::Zero;
    use ark_relations::r1cs::ConstraintSystem;

    const ACCOUNT_IDX: usize = 3;
    const ISSUER_KEY: u64 = 0xBA1CE47;

    fn build(balance: u64, amount: u64, supply_old: u64) -> MintCircuit<Fr> {
        let acc_id = account_id_from_key(Fr::from(0xA11CEu64));
        let nonce = Fr::zero();

        let mut leaves: Vec<Fr> = (0..8u64).map(Fr::from).collect();
        leaves[ACCOUNT_IDX] = account_leaf(acc_id, balance, nonce);
        let tree_old = SimpleMerkleTree::build(leaves.clone());
        let root_old = tree_old.root();
        let path = tree_old.path_for(ACCOUNT_IDX);

        let mut leaves_new = leaves;
        leaves_new[ACCOUNT_IDX] = account_leaf(acc_id, balance + amount, nonce);
        let root_new = SimpleMerkleTree::build(leaves_new).root();

        MintCircuit::new(
            Fr::from(ISSUER_KEY),
            acc_id,
            balance,
            nonce,
            path,
            amount,
            root_old,
            root_new,
            supply_old,
        )
    }

    fn satisfied(circuit: MintCircuit<Fr>) -> bool {
        let cs = ConstraintSystem::<Fr>::new_ref();
        match circuit.generate_constraints(cs.clone()) {
            Ok(()) => cs.is_satisfied().unwrap_or(false),
            Err(_) => false,
        }
    }

    /// Una emisión autorizada y coherente satisface el circuito.
    #[test]
    fn authorized_mint_satisfies() {
        let circuit = build(1000, 500_000, 10_000_000);
        let cs = ConstraintSystem::<Fr>::new_ref();
        circuit.clone().generate_constraints(cs.clone()).unwrap();
        println!("Restricciones del circuito de emision: {}", cs.num_constraints());
        assert!(cs.is_satisfied().unwrap());
    }

    /// **EL TEST QUE CIERRA LA PUERTA TRASERA.**
    ///
    /// El operador del nodo intenta emitir sin la clave del banco
    /// central. Con `open_account` esto FUNCIONABA: se creaba dinero sin
    /// ninguna prueba. Aquí debe fallar.
    #[test]
    fn operator_without_issuer_key_cannot_mint() {
        let mut circuit = build(1000, 500_000, 10_000_000);
        circuit.issuer_key = Some(Fr::from(0x1337u64)); // clave inventada
        assert!(
            !satisfied(circuit),
            "CRITICO: sin la clave del emisor no debe poder crearse dinero. \
             Si esto pasa, toda la conservacion demostrada es irrelevante."
        );
    }

    /// **EL TEST DEL SUMINISTRO.**
    ///
    /// El emisor autorizado intenta aumentar un saldo SIN reflejarlo en
    /// el suministro público. Es emisión encubierta: el dinero aparece
    /// pero la cifra auditable no lo registra.
    #[test]
    fn minting_without_updating_supply_is_rejected() {
        let mut circuit = build(1000, 500_000, 10_000_000);
        // El suministro nuevo no incluye la emision.
        circuit.supply_new = Fr::from(10_000_000u64);
        assert!(
            !satisfied(circuit),
            "CRITICO: emitir sin registrarlo en el suministro publico seria \
             emision encubierta y debe rechazarse"
        );
    }

    /// Inflar el suministro más de lo emitido tampoco cuela.
    #[test]
    fn inflating_supply_beyond_amount_is_rejected() {
        let mut circuit = build(1000, 500_000, 10_000_000);
        circuit.supply_new = Fr::from(20_000_000u64);
        assert!(!satisfied(circuit));
    }

    /// Declarar una raíz nueva que no corresponde al saldo aumentado.
    #[test]
    fn wrong_new_root_is_rejected() {
        let mut circuit = build(1000, 500_000, 10_000_000);
        circuit.root_new = Fr::from(999_999u64);
        assert!(!satisfied(circuit));
    }

    /// Una identidad de emisor distinta a la derivada de la clave.
    #[test]
    fn mismatched_issuer_id_is_rejected() {
        let mut circuit = build(1000, 500_000, 10_000_000);
        circuit.issuer_id = Fr::from(0xDEADu64);
        assert!(!satisfied(circuit));
    }

    /// La identidad del emisor está separada por dominio de las de
    /// cuenta: una clave de gasto no puede hacerse pasar por emisor.
    #[test]
    fn issuer_domain_is_separated_from_accounts() {
        let key = Fr::from(12345u64);
        assert_ne!(
            derive_issuer_id(key),
            account_id_from_key(key),
            "CRITICO: la misma clave no debe dar la misma identidad como \
             emisor y como cuenta"
        );
    }

    /// PRUEBA REAL de extremo a extremo.
    #[test]
    fn mint_end_to_end_proof() {
        let circuit = build(1000, 500_000, 10_000_000);
        let root_old = circuit.root_old;
        let root_new = circuit.root_new;
        let issuer_id = circuit.issuer_id;

        let (pk, vk) = setup_mint(1).expect("setup");
        let proof = prove_mint(&pk, circuit, 2).expect("prove");
        let ok = verify_mint(
            &vk,
            &proof,
            root_old,
            root_new,
            issuer_id,
            500_000,
            10_000_000,
            10_500_000,
        )
        .expect("verify");
        assert!(ok, "una emision autorizada deberia verificar");
    }
}
