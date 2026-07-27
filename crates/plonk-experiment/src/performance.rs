//! Medición de rendimiento del backend PLONK-KZG, con la MISMA
//! metodología que `zk-core::performance`,
//! `halo2-experiment::compliance_real_proof` y
//! `stark-experiment::compliance_real_proof`.
//!
//! ## Qué se mide, y por qué así
//!
//! - `srs_ms`: generación del SRS universal. **Se mide aparte a
//!   propósito**: es la parte reutilizable entre TODOS los circuitos, y
//!   en producción vendría de una ceremonia pública, no de aquí.
//! - `compile_ms`: compilación del circuito (Prover + Verifier). Es
//!   determinista y sin secretos, a diferencia de la fase 2 de Groth16.
//! - `prove_ms`, `verify_ms`, `proof_size_bytes`: como en los demás.
//!   La verificación parte de los BYTES, incluyendo deserialización.
//!
//! Separar SRS de compilación importa para la comparación: en Groth16 el
//! `setup` produce claves específicas del circuito y hay que repetir una
//! ceremonia por cada uno; aquí solo el SRS necesita ceremonia, y se
//! reutiliza.
//!
//! ## Cómo ejecutarlo
//!
//! ```text
//! cargo test -p plonk-experiment --release performance -- --nocapture
//! ```

use std::time::Instant;

use dusk_bytes::Serializable;
use dusk_plonk::prelude::*;

use crate::circuit_double_entry::{build_scenario, DoubleEntryCircuit};
use crate::compliance_circuit::ComplianceCircuit;
use crate::merkle::MerklePath;

#[derive(Debug)]
pub struct PlonkTimingReport {
    pub circuit_name: &'static str,
    pub gates: usize,
    pub srs_ms: u128,
    pub compile_ms: u128,
    pub prove_ms: u128,
    pub verify_ms: u128,
    pub proof_size_bytes: usize,
}

/// Mide un circuito cualquiera con la metodología común.
fn measure<C: Circuit>(
    name: &'static str,
    circuit: &C,
    capacity: usize,
    label: &'static [u8],
) -> Result<PlonkTimingReport, String> {
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    let mut rng = StdRng::seed_from_u64(0x8E7C);

    let t0 = Instant::now();
    let pp = PublicParameters::setup(capacity, &mut rng)
        .map_err(|e| format!("setup del SRS: {e:?}"))?;
    let srs_ms = t0.elapsed().as_millis();

    let t1 = Instant::now();
    let (prover, verifier) =
        Compiler::compile::<C>(&pp, label).map_err(|e| format!("compilacion: {e:?}"))?;
    let compile_ms = t1.elapsed().as_millis();

    let t2 = Instant::now();
    let (proof, public_inputs) = prover
        .prove(&mut rng, circuit)
        .map_err(|e| format!("prove: {e:?}"))?;
    let prove_ms = t2.elapsed().as_millis();

    let proof_bytes = proof.to_bytes();
    let proof_size_bytes = proof_bytes.len();

    // Verificacion DESDE LOS BYTES, como haria un nodo receptor.
    let t3 = Instant::now();
    let received =
        Proof::from_bytes(&proof_bytes).map_err(|e| format!("deserializacion: {e:?}"))?;
    verifier
        .verify(&received, &public_inputs)
        .map_err(|e| format!("verificacion: {e:?}"))?;
    let verify_ms = t3.elapsed().as_millis();

    Ok(PlonkTimingReport {
        circuit_name: name,
        gates: circuit.size(),
        srs_ms,
        compile_ms,
        prove_ms,
        verify_ms,
        proof_size_bytes,
    })
}

pub fn measure_compliance() -> Result<PlonkTimingReport, String> {
    use crate::merkle::TREE_DEPTH;
    use crate::poseidon_hash::native_hash_pair;

    // Camino disperso: hoja en el indice 0, subarboles vacios encima.
    let mut empty = vec![BlsScalar::zero()];
    for k in 1..=TREE_DEPTH {
        let prev = empty[k - 1];
        empty.push(native_hash_pair(prev, prev));
    }
    let mut siblings = vec![BlsScalar::from(999u64)];
    let mut is_right = vec![false];
    for level in 1..TREE_DEPTH {
        siblings.push(empty[level]);
        is_right.push(false);
    }
    let path = MerklePath { siblings, is_right };

    let circuit = ComplianceCircuit::new(42, 1_000_000, 1, 250_000, 500_000, path);
    measure(
        "ComplianceCircuit",
        &circuit,
        crate::compliance_circuit::CAPACITY,
        b"perf-compliance",
    )
}

pub fn measure_double_entry() -> Result<PlonkTimingReport, String> {
    use crate::merkle::TREE_DEPTH;
    use crate::poseidon_hash::native_hash_pair;

    let empty_of = || {
        let mut empty = vec![BlsScalar::zero()];
        for k in 1..=TREE_DEPTH {
            let prev = empty[k - 1];
            empty.push(native_hash_pair(prev, prev));
        }
        empty
    };
    let path_index_0 = |sibling0: BlsScalar| {
        let empty = empty_of();
        let mut siblings = vec![sibling0];
        let mut is_right = vec![false];
        for level in 1..TREE_DEPTH {
            siblings.push(empty[level]);
            is_right.push(false);
        }
        MerklePath { siblings, is_right }
    };
    let path_index_1 = |sibling0: BlsScalar| {
        let empty = empty_of();
        let mut siblings = vec![sibling0];
        let mut is_right = vec![true];
        for level in 1..TREE_DEPTH {
            siblings.push(empty[level]);
            is_right.push(false);
        }
        MerklePath { siblings, is_right }
    };

    let circuit: DoubleEntryCircuit = build_scenario(
        1001,
        1_000_000,
        7,
        2002,
        50_000,
        3,
        250_000,
        250_000,
        500_000,
        path_index_0,
        path_index_1,
    );
    measure(
        "DoubleEntryCircuit",
        &circuit,
        crate::circuit_double_entry::CAPACITY,
        b"perf-double-entry",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn print_report(r: &PlonkTimingReport) {
        println!("============ PLONK-KZG (BLS12-381) ============");
        println!("Circuito              : {}", r.circuit_name);
        println!("Puertas               : {}", r.gates);
        println!("SRS universal         : {} ms  (reutilizable entre circuitos)", r.srs_ms);
        println!("Compilacion           : {} ms  (determinista, sin secretos)", r.compile_ms);
        println!("Generacion de prueba  : {} ms", r.prove_ms);
        println!("Verificacion (bytes)  : {} ms", r.verify_ms);
        println!("Tamano de la prueba   : {} bytes", r.proof_size_bytes);
        println!("==============================================");
    }

    /// `cargo test -p plonk-experiment --release performance -- --nocapture`
    #[test]
    fn performance_plonk_release_measurements() {
        let compliance = measure_compliance().expect("medicion de cumplimiento");
        print_report(&compliance);

        let double_entry = measure_double_entry().expect("medicion de partida doble");
        print_report(&double_entry);

        // Comprobacion de cordura: como en Groth16, el tamano de la
        // prueba NO depende del circuito. Si difirieran, la medicion
        // estaria mal.
        assert_eq!(
            compliance.proof_size_bytes, double_entry.proof_size_bytes,
            "el tamano de una prueba PLONK-KZG no depende del circuito"
        );
    }
}
