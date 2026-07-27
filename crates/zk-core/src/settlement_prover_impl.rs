//! Implementación de `SettlementProver` para el backend Groth16
//! (`ComplianceCircuitWithState`). Envuelve las funciones ya verificadas
//! `setup_with_state`/`prove_with_state`/`verify_with_state` — no añade
//! ninguna lógica criptográfica nueva, solo las adapta a la forma común
//! del trait.

use ark_bls12_381::Fr;
use settlement_prover::SettlementProver;

use crate::circuit_with_state::{
    prove_with_state, setup_with_state, verify_with_state, ComplianceCircuitWithState,
};
use crate::proof_system::{ComplianceProof, ComplianceProvingKey, ComplianceVerifyingKey};
use crate::ZkCoreError;

/// Marcador (sin estado propio) para seleccionar el backend Groth16 a
/// través del trait `SettlementProver`.
pub struct Groth16Backend;

/// Inputs públicos de una prueba Groth16 con estado: raíz, límite
/// (como `u64`, porque `verify_with_state` lo exige así — ver la nota de
/// asimetría en `settlement-prover::lib`) y nullifier.
pub struct Groth16PublicInput {
    pub state_root: Fr,
    pub regulatory_limit_minor_units: u64,
    pub nullifier: Fr,
}

impl SettlementProver for Groth16Backend {
    /// El circuito YA calcula/expone `state_root` y `nullifier`
    /// internamente (ver `circuit_with_state.rs`), pero `regulatory_limit`
    /// se almacena ahí como `Fr`, no como el `u64` original que
    /// `verify_with_state` necesita — así que el testigo incluye ambos.
    type Witness = (ComplianceCircuitWithState<Fr>, u64);
    type PublicInput = Groth16PublicInput;
    type Proof = ComplianceProof;
    type ProvingKey = ComplianceProvingKey;
    type VerifyingKey = ComplianceVerifyingKey;
    type Error = ZkCoreError;

    fn setup(rng_seed: u64) -> Result<(Self::ProvingKey, Self::VerifyingKey), Self::Error> {
        setup_with_state(rng_seed)
    }

    fn prove(
        pk: &Self::ProvingKey,
        witness: Self::Witness,
        rng_seed: u64,
    ) -> Result<(Self::Proof, Self::PublicInput), Self::Error> {
        let (circuit, regulatory_limit_minor_units) = witness;
        let state_root = circuit.state_root;
        let nullifier = circuit.nullifier;

        let proof = prove_with_state(pk, circuit, rng_seed)?;

        Ok((
            proof,
            Groth16PublicInput {
                state_root,
                regulatory_limit_minor_units,
                nullifier,
            },
        ))
    }

    fn verify(
        vk: &Self::VerifyingKey,
        public_input: &Self::PublicInput,
        proof: &Self::Proof,
    ) -> Result<bool, Self::Error> {
        verify_with_state(
            vk,
            proof,
            public_input.state_root,
            public_input.regulatory_limit_minor_units,
            public_input.nullifier,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::merkle::SimpleMerkleTree;
    use crate::circuit_with_state::compute_leaf;

    /// EL TEST CLAVE: el mismo flujo setup/prove/verify que ya
    /// verificamos directamente contra `circuit_with_state.rs`, ahora
    /// pasando exclusivamente por la interfaz genérica `SettlementProver`
    /// — confirma que la abstracción no cambia el comportamiento real.
    #[test]
    fn groth16_backend_valid_transaction_via_trait() {
        let account_id = Fr::from(42u64);
        let nonce = Fr::from(1u64);
        let balance: u64 = 1_000_000;
        let amount: u64 = 250_000;
        let regulatory_limit: u64 = 500_000;

        let leaf = compute_leaf(account_id, balance, nonce);
        let mut leaves = vec![Fr::from(1u64), Fr::from(2u64), Fr::from(3u64), leaf];
        leaves.resize(8, Fr::from(0u64));
        let tree = SimpleMerkleTree::build(leaves);
        let path = tree.path_for(3);
        let root = tree.root();

        let circuit = ComplianceCircuitWithState::new(
            account_id, balance, nonce, amount, path, root, regulatory_limit,
        );

        let (pk, vk) = Groth16Backend::setup(1).expect("setup no deberia fallar");
        let (proof, public_input) =
            Groth16Backend::prove(&pk, (circuit, regulatory_limit), 2).expect("prove no deberia fallar");
        let is_valid = Groth16Backend::verify(&vk, &public_input, &proof)
            .expect("verify no deberia devolver error");

        assert!(is_valid, "una transaccion valida debe verificar como verdadera via el trait");
    }
}
