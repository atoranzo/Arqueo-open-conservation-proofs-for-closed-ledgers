//! Medición de rendimiento del backend Groth16, con la MISMA metodología
//! que `halo2-experiment::compliance_real_proof` y
//! `stark-experiment::compliance_real_proof`, para que las tres cifras
//! sean comparables entre sí.
//!
//! ## Por qué existe este módulo
//!
//! Las cifras de Groth16 que circulaban en `PERFORMANCE.md` y
//! `THREE_BACKENDS.md` procedían de ejecuciones en modo **debug**,
//! mientras las de STARK se midieron en **release**. Comparar unas con
//! otras es un error metodológico que invalida la tabla: el mismo
//! circuito pasó de ~5,2 s (debug) a ~0,39 s (release), un factor de 13.
//!
//! Este módulo elimina esa asimetría midiendo lo mismo, de la misma forma,
//! en el mismo sitio.
//!
//! ## Qué se mide, y por qué así
//!
//! - `setup_ms`: generación de claves (la ceremonia real sería aparte).
//! - `prove_ms`: generación de la prueba.
//! - `proof_size_bytes`: prueba SERIALIZADA en forma comprimida — el
//!   número que viajaría por la red, comparable con los 36,7 KB de STARK
//!   y los ~4 KB de Halo2.
//! - `verify_ms`: verificación **partiendo de los bytes**
//!   (deserialización incluida), como haría un nodo que recibe la prueba,
//!   no verificando un objeto que ya tenía en memoria.
//!
//! ## Cómo ejecutarlo
//!
//! ```text
//! cargo test -p zk-core --release performance -- --nocapture
//! ```
//!
//! **En release siempre.** Los tiempos de debug no son citables, que es
//! precisamente el error que este módulo corrige.

use std::time::Instant;

use ark_bls12_381::Fr;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};

use crate::circuit_double_entry::{
    prove_double_entry, setup_double_entry, verify_double_entry, AccountWitness,
    DoubleEntryCircuit,
};
use crate::circuit_with_state::{
    compute_leaf, prove_with_state, setup_with_state, verify_with_state,
    ComplianceCircuitWithState,
};
use crate::merkle::{leaf_commitment, SimpleMerkleTree};
use crate::proof_system::ComplianceProof;
use crate::ZkCoreError;

#[derive(Debug)]
pub struct Groth16TimingReport {
    pub circuit_name: &'static str,
    pub setup_ms: u128,
    pub prove_ms: u128,
    pub verify_ms: u128,
    pub proof_size_bytes: usize,
}

/// Mide el circuito de cumplimiento con estado (solvencia + Merkle +
/// nullifier).
pub fn measure_compliance_with_state() -> Result<Groth16TimingReport, ZkCoreError> {
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
        account_id,
        balance,
        nonce,
        amount,
        path,
        root,
        regulatory_limit,
    );
    let nullifier = circuit.nullifier;

    let t0 = Instant::now();
    let (pk, vk) = setup_with_state(1)?;
    let setup_ms = t0.elapsed().as_millis();

    let t1 = Instant::now();
    let proof = prove_with_state(&pk, circuit, 2)?;
    let prove_ms = t1.elapsed().as_millis();

    let mut proof_bytes = Vec::new();
    proof
        .serialize_compressed(&mut proof_bytes)
        .map_err(|e| ZkCoreError::ProvingFailed(format!("serializacion de la prueba: {e}")))?;
    let proof_size_bytes = proof_bytes.len();

    let t2 = Instant::now();
    let received = ComplianceProof::deserialize_compressed(&proof_bytes[..])
        .map_err(|e| ZkCoreError::VerificationFailed(format!("deserializacion: {e}")))?;
    let ok = verify_with_state(&vk, &received, root, regulatory_limit, nullifier)?;
    let verify_ms = t2.elapsed().as_millis();

    assert!(ok, "la prueba medida deberia verificar correctamente");

    Ok(Groth16TimingReport {
        circuit_name: "ComplianceCircuitWithState (9.934 restricciones)",
        setup_ms,
        prove_ms,
        verify_ms,
        proof_size_bytes,
    })
}

/// Mide el circuito de partida doble (transición de estado completa).
pub fn measure_double_entry() -> Result<Groth16TimingReport, ZkCoreError> {
    const SENDER_IDX: usize = 3;
    const RECEIVER_IDX: usize = 5;

    let sender_id = Fr::from(1001u64);
    let sender_nonce = Fr::from(7u64);
    let receiver_id = Fr::from(2002u64);
    let receiver_nonce = Fr::from(3u64);
    let (sender_balance, receiver_balance, amount, limit) =
        (1_000_000u64, 50_000u64, 250_000u64, 500_000u64);

    let mut leaves: Vec<Fr> = (0..8u64).map(Fr::from).collect();
    leaves[SENDER_IDX] = leaf_commitment(sender_id, Fr::from(sender_balance), sender_nonce);
    leaves[RECEIVER_IDX] =
        leaf_commitment(receiver_id, Fr::from(receiver_balance), receiver_nonce);

    let tree_old = SimpleMerkleTree::build(leaves.clone());
    let root_old = tree_old.root();
    let sender_path = tree_old.path_for(SENDER_IDX);

    let mut leaves_mid = leaves.clone();
    leaves_mid[SENDER_IDX] = leaf_commitment(
        sender_id,
        Fr::from(sender_balance - amount),
        sender_nonce + Fr::from(1u64),
    );
    let tree_mid = SimpleMerkleTree::build(leaves_mid.clone());
    let receiver_path = tree_mid.path_for(RECEIVER_IDX);

    let mut leaves_new = leaves_mid;
    leaves_new[RECEIVER_IDX] = leaf_commitment(
        receiver_id,
        Fr::from(receiver_balance + amount),
        receiver_nonce,
    );
    let root_new = SimpleMerkleTree::build(leaves_new).root();

    let circuit = DoubleEntryCircuit::new(
        AccountWitness {
            account_id: sender_id,
            balance: sender_balance,
            nonce: sender_nonce,
            merkle_path: sender_path,
        },
        AccountWitness {
            account_id: receiver_id,
            balance: receiver_balance,
            nonce: receiver_nonce,
            merkle_path: receiver_path,
        },
        amount,
        root_old,
        root_new,
        limit,
    );
    let nullifier = circuit.nullifier;

    let t0 = Instant::now();
    let (pk, vk) = setup_double_entry(1)?;
    let setup_ms = t0.elapsed().as_millis();

    let t1 = Instant::now();
    let proof = prove_double_entry(&pk, circuit, 2)?;
    let prove_ms = t1.elapsed().as_millis();

    let mut proof_bytes = Vec::new();
    proof
        .serialize_compressed(&mut proof_bytes)
        .map_err(|e| ZkCoreError::ProvingFailed(format!("serializacion de la prueba: {e}")))?;
    let proof_size_bytes = proof_bytes.len();

    let t2 = Instant::now();
    let received = ComplianceProof::deserialize_compressed(&proof_bytes[..])
        .map_err(|e| ZkCoreError::VerificationFailed(format!("deserializacion: {e}")))?;
    let ok = verify_double_entry(&vk, &received, root_old, root_new, limit, nullifier)?;
    let verify_ms = t2.elapsed().as_millis();

    assert!(ok, "la prueba medida deberia verificar correctamente");

    Ok(Groth16TimingReport {
        circuit_name: "DoubleEntryCircuit (27.562 restricciones)",
        setup_ms,
        prove_ms,
        verify_ms,
        proof_size_bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn print_report(report: &Groth16TimingReport) {
        println!("============ GROTH16 (BLS12-381) ============");
        println!("Circuito              : {}", report.circuit_name);
        println!("Setup de claves       : {} ms", report.setup_ms);
        println!("Generacion de prueba  : {} ms", report.prove_ms);
        println!("Verificacion (bytes)  : {} ms", report.verify_ms);
        println!(
            "Tamano de la prueba   : {} bytes",
            report.proof_size_bytes
        );
        println!("=============================================");
    }

    /// Mide los dos circuitos Groth16 en release.
    ///
    /// `cargo test -p zk-core --release performance -- --nocapture`
    #[test]
    fn performance_groth16_release_measurements() {
        let with_state =
            measure_compliance_with_state().expect("la medicion no deberia fallar");
        print_report(&with_state);

        let double_entry = measure_double_entry().expect("la medicion no deberia fallar");
        print_report(&double_entry);

        // Comprobacion de cordura: la prueba Groth16 es de tamano constante
        // e independiente del circuito. Si estos dos numeros difirieran,
        // algo estaria mal en la medicion.
        assert_eq!(
            with_state.proof_size_bytes, double_entry.proof_size_bytes,
            "el tamano de una prueba Groth16 no depende del circuito"
        );
    }
}
