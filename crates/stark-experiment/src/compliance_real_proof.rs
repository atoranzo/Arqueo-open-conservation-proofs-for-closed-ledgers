//! Pipeline de extremo a extremo con métricas REALES — el equivalente
//! STARK de `halo2-experiment::compliance_real_proof`.
//!
//! ## Qué mide, y por qué así
//!
//! - `trace_ms`: construcción de la traza (el "witness generation").
//! - `prove_ms`: generación de la prueba STARK.
//! - `proof_size_bytes`: tamaño de la prueba SERIALIZADA — el número que
//!   la literatura estima en "decenas o cientos de KB" y que aquí por fin
//!   medimos en vez de citar.
//! - `verify_ms`: verificación, partiendo de los BYTES (deserialización
//!   incluida), porque es lo que haría un nodo real que recibe la prueba
//!   por la red — no verificar un objeto que ya tenía en memoria.
//!
//! ## Nota importante sobre los parámetros
//!
//! Las métricas dependen de `ProofOptions` (32 queries, blowup 8, sin
//! extensión de campo, sin grinding — los mismos de todos los tests del
//! crate). Cambiar estos parámetros cambia el equilibrio
//! seguridad/tamaño/velocidad; los números medidos solo valen para esta
//! configuración concreta y así deben citarse.
//!
//! ## Cómo ejecutarlo
//!
//! ```text
//! cargo test -p stark-experiment real_proof --release -- --nocapture
//! ```
//!
//! En release: los tiempos en debug no son representativos (sin
//! optimizar) y no deben citarse como métricas.

use std::time::Instant;

use winterfell::crypto::{hashers::Blake3_256, DefaultRandomCoin, MerkleTree};
use winterfell::math::fields::f64::BaseElement;
use winterfell::{verify, AcceptableOptions, Proof, ProofOptions, Prover};

use crate::compliance_circuit::{
    build_trace, native_leaf, native_nullifier, native_root, ComplianceAir, ComplianceProver,
    CompliancePublicInputs, TREE_DEPTH,
};
use crate::merkle::{Digest, MerklePath};

type Blake3 = Blake3_256<BaseElement>;

/// Informe de métricas de una ejecución completa.
#[derive(Debug)]
pub struct StarkTimingReport {
    pub trace_ms: u128,
    pub prove_ms: u128,
    pub verify_ms: u128,
    pub proof_size_bytes: usize,
    /// Seguridad CONJETURADA en bits, según winterfell. Es la cifra que
    /// se suele citar en el ecosistema STARK.
    pub conjectured_security: u32,
    /// Seguridad DEMOSTRABLE en el régimen de decodificación única.
    pub proven_security_udr: u32,
    /// Seguridad DEMOSTRABLE en el régimen de decodificación por listas.
    pub proven_security_ldr: u32,
}

fn digest_from(n: u64) -> Digest {
    [
        BaseElement::new(n),
        BaseElement::new(n + 1),
        BaseElement::new(n + 2),
        BaseElement::new(n + 3),
    ]
}

/// Ejecuta el pipeline completo (traza → prueba → bytes → verificación)
/// con la configuración dada y devuelve las métricas medidas.
pub fn run_end_to_end_with_timing(options: ProofOptions) -> Result<StarkTimingReport, String> {
    // Escenario idéntico al de los tests del circuito unificado.
    let account_id = BaseElement::new(12345);
    let balance: u64 = 1_000_000;
    let nonce = BaseElement::new(1);
    let amount: u64 = 250_000;
    let limit: u64 = 500_000;
    let siblings: Vec<Digest> = (0..TREE_DEPTH).map(|i| digest_from(i as u64 * 10)).collect();
    let is_right: Vec<bool> = (0..TREE_DEPTH).map(|i| i % 3 == 0).collect();
    let path = MerklePath { siblings, is_right };

    let leaf = native_leaf(account_id, BaseElement::new(balance), nonce);
    let public_inputs = CompliancePublicInputs {
        state_root: native_root(leaf, &path),
        regulatory_limit: BaseElement::new(limit),
        nullifier: native_nullifier(account_id, nonce),
    };

    // 1. Construcción de la traza.
    let t0 = Instant::now();
    let trace = build_trace(account_id, balance, nonce, amount, limit, &path);
    let trace_ms = t0.elapsed().as_millis();

    // 2. Generación de la prueba.
    let prover = ComplianceProver::new(options.clone());
    let t1 = Instant::now();
    let proof = prover
        .prove(trace)
        .map_err(|e| format!("prove fallo: {e:?}"))?;
    let prove_ms = t1.elapsed().as_millis();

    // 3. Serialización: el tamaño real que viajaría por la red. Y las
    //    estimaciones de seguridad DE LA PROPIA LIBRERÍA, no nuestras.
    //    Se reportan las DOS: la conjeturada (la que suele citarse en el
    //    ecosistema STARK) y la demostrable (más conservadora — la propia
    //    documentación de winterfell señala que alcanzar el mismo nivel
    //    de forma demostrable exige de 2 a 3 veces más queries).
    let conjectured_security = proof.conjectured_security::<Blake3>().bits();
    let proven = proof.proven_security::<Blake3>();
    let proven_security_udr = proven.udr_bits();
    let proven_security_ldr = proven.ldr_bits();
    let proof_bytes = proof.to_bytes();
    let proof_size_bytes = proof_bytes.len();

    // 4. Verificación DESDE LOS BYTES (deserialización incluida), como
    //    haría un nodo que recibe la prueba por la red.
    let t2 = Instant::now();
    let received =
        Proof::from_bytes(&proof_bytes).map_err(|e| format!("deserializacion fallo: {e:?}"))?;
    let min_opts = AcceptableOptions::OptionSet(vec![options]);
    verify::<ComplianceAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
        received,
        public_inputs,
        &min_opts,
    )
    .map_err(|e| format!("la verificacion fallo: {e:?}"))?;
    let verify_ms = t2.elapsed().as_millis();

    Ok(StarkTimingReport {
        trace_ms,
        prove_ms,
        verify_ms,
        proof_size_bytes,
        conjectured_security,
        proven_security_udr,
        proven_security_ldr,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use winterfell::{BatchingMethod, FieldExtension};

    fn print_report(label: &str, report: &StarkTimingReport) {
        println!("============ METRICAS STARK REALES: {label} ============");
        println!("Construccion de traza : {} ms", report.trace_ms);
        println!("Generacion de prueba  : {} ms", report.prove_ms);
        println!("Verificacion (bytes)  : {} ms", report.verify_ms);
        println!(
            "Tamano de la prueba   : {} bytes ({:.1} KB)",
            report.proof_size_bytes,
            report.proof_size_bytes as f64 / 1024.0
        );
        println!(
            "Seguridad conjeturada : {} bits",
            report.conjectured_security
        );
        println!(
            "Seguridad demostrable : {} bits (udr) / {} bits (ldr)",
            report.proven_security_udr, report.proven_security_ldr
        );
        println!("(estimaciones de winterfell, no calculadas por nosotros)");
        println!("=======================================================");
    }

    /// EL TEST DE MÉTRICAS: mide CUATRO configuraciones y las imprime.
    ///
    /// Motivo de que sean cuatro, y no una: las dos primeras revelaron
    /// (con evidencia, no con estimaciones) que la configuración cómoda
    /// NO alcanza el nivel de seguridad de Groth16/Halo2:
    /// - base (sin extensión): 63 bits conjeturados — es el techo del
    ///   propio campo Goldilocks de 64 bits; ninguna cantidad de queries
    ///   lo supera sin extender el campo.
    /// - cuadrática: 95 bits conjeturados — mejor, aún insuficiente.
    ///
    /// Las dos siguientes buscan la comparación justa: `blowup 16` (cada
    /// query aporta log2(16) = 4 bits en vez de 3) para alcanzar ~128
    /// bits conjeturados, y una configuración cara que intenta subir la
    /// seguridad DEMOSTRABLE, mucho más baja en todos los casos.
    ///
    /// El número que se cite en la comparativa debe ser el de la
    /// configuración con seguridad equiparable, no el más favorable.
    ///
    /// `cargo test -p stark-experiment real_proof --release -- --nocapture`
    #[test]
    fn real_proof_end_to_end_with_metrics() {
        let base = ProofOptions::new(
            32,
            8,
            0,
            FieldExtension::None,
            8,
            31,
            BatchingMethod::Linear,
            BatchingMethod::Linear,
        );
        let quadratic = ProofOptions::new(
            32,
            8,
            0,
            FieldExtension::Quadratic,
            8,
            31,
            BatchingMethod::Linear,
            BatchingMethod::Linear,
        );
        // Objetivo ~128 bits conjeturados: blowup 16 -> 4 bits por query.
        let target_128 = ProofOptions::new(
            32,
            16,
            0,
            FieldExtension::Quadratic,
            8,
            31,
            BatchingMethod::Linear,
            BatchingMethod::Linear,
        );
        // Intento de subir la seguridad DEMOSTRABLE: muchas más queries
        // (la documentación de winterfell advierte que hacen falta de 2 a
        // 3 veces más) mas grinding, con extensión cúbica.
        let high_proven = ProofOptions::new(
            120,
            16,
            20,
            FieldExtension::Cubic,
            8,
            31,
            BatchingMethod::Linear,
            BatchingMethod::Linear,
        );

        for (label, opts) in [
            ("base (sin extension)", base),
            ("extension cuadratica", quadratic),
            ("objetivo 128 bits (blowup 16)", target_128),
            ("alta seguridad demostrable", high_proven),
        ] {
            match run_end_to_end_with_timing(opts) {
                Ok(report) => print_report(label, &report),
                Err(e) => println!("!! configuracion '{label}' fallo: {e}"),
            }
        }
    }
}
