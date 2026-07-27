//! Pipeline de prueba real con IPA para `ComplianceCircuit` (el circuito
//! unificado completo). Reutiliza exactamente el mismo patrón ya
//! verificado en `real_proof.rs` (setup determinista, keygen, prove,
//! verify) — el riesgo de API aquí es bajo, ya que es la misma receta
//! aplicada a un circuito distinto. Lo que este módulo SÍ mide por
//! primera vez es el rendimiento real con un circuito de este tamaño.

use crate::compliance_circuit::{ComplianceCircuit, NULLIFIER_DOMAIN, TREE_DEPTH};
use halo2_gadgets::poseidon::primitives::{ConstantLength, P128Pow5T3};
use halo2_proofs::circuit::Value;
use halo2_proofs::pasta::{EqAffine, Fp};
use halo2_proofs::plonk::{create_proof, keygen_pk, keygen_vk, verify_proof, SingleVerifier};
use halo2_proofs::poly::commitment::Params;
use halo2_proofs::transcript::{Blake2bRead, Blake2bWrite, Challenge255};
use rand_core::OsRng;
use std::time::Instant;

fn native_hash(a: Fp, b: Fp) -> Fp {
    halo2_gadgets::poseidon::primitives::Hash::<Fp, P128Pow5T3, ConstantLength<2>, 3, 2>::init()
        .hash([a, b])
}

fn native_leaf(account_id: Fp, balance: Fp, nonce: Fp) -> Fp {
    native_hash(native_hash(account_id, balance), nonce)
}

fn native_nullifier(account_id: Fp, nonce: Fp) -> Fp {
    let domain = Fp::from(NULLIFIER_DOMAIN);
    native_hash(native_hash(domain, account_id), nonce)
}

struct NativeTree {
    levels: Vec<Vec<Fp>>,
}
impl NativeTree {
    fn build(mut leaves: Vec<Fp>) -> Self {
        let target = 1usize << TREE_DEPTH;
        leaves.resize(target, Fp::zero());
        let mut levels = vec![leaves];
        for _ in 0..TREE_DEPTH {
            let prev = levels.last().unwrap();
            let next: Vec<Fp> = prev.chunks(2).map(|p| native_hash(p[0], p[1])).collect();
            levels.push(next);
        }
        Self { levels }
    }
    fn root(&self) -> Fp {
        self.levels.last().unwrap()[0]
    }
    fn path_for(&self, index: usize) -> (Vec<Fp>, Vec<bool>) {
        let mut siblings = Vec::with_capacity(TREE_DEPTH);
        let mut is_right = Vec::with_capacity(TREE_DEPTH);
        let mut idx = index;
        for level in 0..TREE_DEPTH {
            siblings.push(self.levels[level][idx ^ 1]);
            is_right.push(idx % 2 == 1);
            idx /= 2;
        }
        (siblings, is_right)
    }
}

fn bool_to_fp(b: bool) -> Fp {
    if b {
        Fp::one()
    } else {
        Fp::zero()
    }
}

/// Construye un circuito y sus inputs públicos válidos, listos para
/// setup/keygen/prove/verify.
fn build_valid_circuit_and_public_input(
    account_id: u64,
    balance: u64,
    nonce: u64,
    amount: u64,
    regulatory_limit: u64,
) -> (ComplianceCircuit, Vec<Fp>) {
    let account_id = Fp::from(account_id);
    let balance = Fp::from(balance);
    let nonce = Fp::from(nonce);
    let amount = Fp::from(amount);
    let regulatory_limit = Fp::from(regulatory_limit);

    let leaf = native_leaf(account_id, balance, nonce);
    let mut leaves = vec![Fp::from(1), Fp::from(2), Fp::from(3), leaf];
    leaves.resize(8, Fp::zero());
    let tree = NativeTree::build(leaves);
    let root = tree.root();
    let (siblings, is_right) = tree.path_for(3);
    let nullifier = native_nullifier(account_id, nonce);

    let circuit = ComplianceCircuit {
        account_id: Value::known(account_id),
        balance: Value::known(balance),
        account_nonce: Value::known(nonce),
        amount: Value::known(amount),
        regulatory_limit: Value::known(regulatory_limit),
        siblings: siblings.into_iter().map(Value::known).collect(),
        path_bits: is_right.into_iter().map(|b| Value::known(bool_to_fp(b))).collect(),
    };

    (circuit, vec![root, regulatory_limit, nullifier])
}

/// Ejecuta el flujo completo real (no `MockProver`) y devuelve los
/// tiempos medidos de cada fase, para tener datos reales de rendimiento.
pub struct TimingReport {
    pub setup_ms: u128,
    pub keygen_vk_ms: u128,
    pub keygen_pk_ms: u128,
    pub prove_ms: u128,
    pub verify_ms: u128,
    pub proof_size_bytes: usize,
}

pub fn run_end_to_end_with_timing(k: u32) -> Result<TimingReport, String> {
    let t0 = Instant::now();
    let params: Params<EqAffine> = Params::new(k);
    let setup_ms = t0.elapsed().as_millis();

    let empty_circuit = ComplianceCircuit::default();

    let t1 = Instant::now();
    let vk = keygen_vk(&params, &empty_circuit).map_err(|e| format!("keygen_vk: {e:?}"))?;
    let keygen_vk_ms = t1.elapsed().as_millis();

    let t2 = Instant::now();
    let pk = keygen_pk(&params, vk.clone(), &empty_circuit).map_err(|e| format!("keygen_pk: {e:?}"))?;
    let keygen_pk_ms = t2.elapsed().as_millis();

    let (circuit, public_input) =
        build_valid_circuit_and_public_input(12345, 1_000_000, 1, 250_000, 500_000);

    let t3 = Instant::now();
    let mut transcript = Blake2bWrite::<_, EqAffine, Challenge255<_>>::init(vec![]);
    create_proof(
        &params,
        &pk,
        &[circuit],
        &[&[&public_input]],
        OsRng,
        &mut transcript,
    )
    .map_err(|e| format!("create_proof: {e:?}"))?;
    let proof = transcript.finalize();
    let prove_ms = t3.elapsed().as_millis();
    let proof_size_bytes = proof.len();

    let t4 = Instant::now();
    let strategy = SingleVerifier::new(&params);
    let mut transcript_reader = Blake2bRead::<_, EqAffine, Challenge255<_>>::init(&proof[..]);
    verify_proof(&params, &vk, strategy, &[&[&public_input]], &mut transcript_reader)
        .map_err(|e| format!("verify_proof: {e:?}"))?;
    let verify_ms = t4.elapsed().as_millis();

    Ok(TimingReport {
        setup_ms,
        keygen_vk_ms,
        keygen_pk_ms,
        prove_ms,
        verify_ms,
        proof_size_bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// EL TEST FINAL: prueba real con IPA, de extremo a extremo, sobre el
    /// circuito de cumplimiento completo. Si esto pasa, tenemos la
    /// confirmación definitiva de que Halo2 es una alternativa viable de
    /// verdad, no solo en teoria/satisfacibilidad.
    #[test]
    fn end_to_end_real_ipa_proof_full_compliance_circuit() {
        let k = 15;
        let report = run_end_to_end_with_timing(k).expect("el flujo completo no deberia fallar");

        println!("=== INFORME DE RENDIMIENTO REAL (k={k}) ===");
        println!("Setup (deterministico, sin trusted setup): {} ms", report.setup_ms);
        println!("keygen_vk: {} ms", report.keygen_vk_ms);
        println!("keygen_pk: {} ms", report.keygen_pk_ms);
        println!("create_proof: {} ms", report.prove_ms);
        println!("verify_proof: {} ms", report.verify_ms);
        println!("Tamaño de la prueba: {} bytes", report.proof_size_bytes);
    }
}
