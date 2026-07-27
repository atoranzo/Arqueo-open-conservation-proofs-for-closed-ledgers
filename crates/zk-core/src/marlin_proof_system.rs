//! Capa de pruebas con Marlin, como alternativa a Groth16 que elimina el
//! problema del "trusted setup por circuito".
//!
//! ## ⚠️ Nivel de riesgo de esta pieza
//!
//! Similar o mayor incertidumbre que `poseidon_hash.rs` en su momento:
//! `ark-marlin` y `ark-poly-commit` son piezas del ecosistema Arkworks que
//! he usado con menos frecuencia que `ark-groth16`. Es razonable esperar
//! varias rondas de corrección, incluyendo posibles problemas de versión
//! con el crate `blake2` (la función hash usada como oráculo aleatorio de
//! Fiat-Shamir) — Marlin en esta era del ecosistema Arkworks probablemente
//! espera el tipo `blake2::Blake2s` de una versión anterior del crate
//! `blake2` (0.9.x), no el `Blake2s256` de versiones más recientes (0.10+).
//! Si el error de compilación menciona esto, es la primera sospecha.
//!
//! ## Por qué esto existe (qué problema resuelve)
//!
//! A diferencia de Groth16 (`proof_system.rs`), que necesita un "trusted
//! setup" específico para CADA circuito (y hay que rehacerlo si el
//! circuito cambia aunque sea una línea), Marlin usa un esquema de
//! compromiso polinómico (aquí, una variante de KZG10) con un SETUP
//! UNIVERSAL: se genera una vez, hasta un tamaño máximo de circuito, y
//! sirve para CUALQUIER circuito de ese tamaño o menor — incluyendo
//! circuitos que ni siquiera existían cuando se generó el setup.
//!
//! Esto es lo que permite, en teoría, reutilizar una ceremonia MPC pública
//! ya existente (en vez de organizar una nueva cada vez que cambia
//! `ComplianceCircuit`), cerrando el hueco documentado en el README sobre
//! el trusted setup de un solo participante.
//!
//! El setup universal en sí SIGUE necesitando un origen honesto — esto no
//! elimina la necesidad de una ceremonia, la reduce a "una vez para
//! siempre" en vez de "una vez por versión del circuito".
//!
//! ## Alcance de esta primera versión
//!
//! Solo cubre `ComplianceCircuit` (el circuito simple, sin vinculación de
//! estado ni Poseidon). Extenderlo a `ComplianceCircuitWithState` es un
//! paso posterior, una vez que esto compile y funcione.

use ark_bls12_381::{Bls12_381, Fr};
use ark_marlin::Marlin;
use ark_poly::univariate::DensePolynomial;
use ark_poly_commit::marlin_pc::MarlinKZG10;
use blake2::Blake2s;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha20Rng;

use crate::circuit::ComplianceCircuit;

/// Esquema de compromiso polinómico usado por esta instancia de Marlin:
/// una variante de KZG10 sobre BLS12-381.
pub type MultiPC = MarlinKZG10<Bls12_381, DensePolynomial<Fr>>;

/// Instancia concreta de Marlin para este proyecto: cuerpo Fr (BLS12-381),
/// el esquema de compromiso de arriba, y Blake2s como función hash del
/// oráculo aleatorio (transformación de Fiat-Shamir).
pub type MarlinInst = Marlin<Fr, MultiPC, Blake2s>;

pub type MarlinUniversalSrs =
    ark_poly_commit::kzg10::UniversalParams<Bls12_381>;

#[derive(Debug)]
pub enum MarlinError {
    SetupFailed(String),
    IndexFailed(String),
    ProvingFailed(String),
    VerificationFailed(String),
}

impl std::fmt::Display for MarlinError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarlinError::SetupFailed(e) => write!(f, "fallo en el setup universal de Marlin: {e}"),
            MarlinError::IndexFailed(e) => write!(f, "fallo al indexar el circuito: {e}"),
            MarlinError::ProvingFailed(e) => write!(f, "fallo al generar la prueba Marlin: {e}"),
            MarlinError::VerificationFailed(e) => write!(f, "fallo al verificar la prueba Marlin: {e}"),
        }
    }
}
impl std::error::Error for MarlinError {}

/// Genera el SRS (Structured Reference String) UNIVERSAL. Esto se hace UNA
/// VEZ para todo el sistema, no una vez por circuito — esa es la ventaja
/// respecto a Groth16. `num_constraints`/`num_variables`/`num_non_zero`
/// deben ser cotas superiores holgadas sobre el tamaño de cualquier
/// circuito que se quiera soportar en el futuro con este mismo SRS.
pub fn universal_setup(
    num_constraints: usize,
    num_variables: usize,
    num_non_zero: usize,
    rng_seed: u64,
) -> Result<MarlinUniversalSrs, MarlinError> {
    let mut rng = ChaCha20Rng::seed_from_u64(rng_seed);
    MarlinInst::universal_setup(num_constraints, num_variables, num_non_zero, &mut rng)
        .map_err(|e| MarlinError::SetupFailed(format!("{e:?}")))
}

/// Deriva las claves de proving/verifying ESPECÍFICAS de
/// `ComplianceCircuit` a partir del SRS universal. Esto es determinista y
/// no requiere ninguna ceremonia adicional — el SRS universal ya contiene
/// toda la confianza necesaria.
pub fn index_circuit(
    srs: &MarlinUniversalSrs,
    circuit: ComplianceCircuit<Fr>,
) -> Result<
    (
        ark_marlin::IndexProverKey<Fr, MultiPC>,
        ark_marlin::IndexVerifierKey<Fr, MultiPC>,
    ),
    MarlinError,
> {
    MarlinInst::index(srs, circuit).map_err(|e| MarlinError::IndexFailed(format!("{e:?}")))
}

/// Genera una prueba Marlin.
pub fn prove(
    index_pk: &ark_marlin::IndexProverKey<Fr, MultiPC>,
    circuit: ComplianceCircuit<Fr>,
    rng_seed: u64,
) -> Result<ark_marlin::Proof<Fr, MultiPC>, MarlinError> {
    let mut rng = ChaCha20Rng::seed_from_u64(rng_seed);
    MarlinInst::prove(index_pk, circuit, &mut rng)
        .map_err(|e| MarlinError::ProvingFailed(format!("{e:?}")))
}

/// Verifica una prueba Marlin. A diferencia de `Groth16::verify` (que no
/// necesita aleatoriedad), la verificación en Marlin puede necesitar un
/// generador de números aleatorios para la apertura del compromiso
/// polinómico — de ahí el parámetro `rng_seed` aquí, que no existía en la
/// versión de Groth16 de `proof_system.rs`.
pub fn verify(
    index_vk: &ark_marlin::IndexVerifierKey<Fr, MultiPC>,
    public_input: &[Fr],
    proof: &ark_marlin::Proof<Fr, MultiPC>,
    rng_seed: u64,
) -> Result<bool, MarlinError> {
    let mut rng = ChaCha20Rng::seed_from_u64(rng_seed);
    MarlinInst::verify(index_vk, public_input, proof, &mut rng)
        .map_err(|e| MarlinError::VerificationFailed(format!("{e:?}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Cotas de tamaño generosas para el circuito simple (ComplianceCircuit
    /// tiene un número de restricciones pequeño: unas pocas decenas por
    /// los range checks de 64 bits). Estos números son deliberadamente
    /// holgados; si el circuito real los supera, `index_circuit` fallará
    /// con un error claro, no silenciosamente.
    const MAX_CONSTRAINTS: usize = 4_000;
    const MAX_VARIABLES: usize = 4_000;
    const MAX_NON_ZERO: usize = 4_000;

    #[test]
    fn end_to_end_valid_transaction_with_universal_setup() {
        let srs = universal_setup(MAX_CONSTRAINTS, MAX_VARIABLES, MAX_NON_ZERO, 1)
            .expect("el setup universal no deberia fallar");

        let circuit = ComplianceCircuit::<Fr>::new(1_000_000, 250_000, 500_000);

        let (index_pk, index_vk) =
            index_circuit(&srs, circuit.clone()).expect("indexar el circuito no deberia fallar");

        let proof = prove(&index_pk, circuit, 7).expect("generar la prueba no deberia fallar");

        let public_input = vec![Fr::from(500_000u64)];
        let is_valid = verify(&index_vk, &public_input, &proof, 99)
            .expect("la verificacion no deberia devolver error");

        assert!(is_valid, "una transaccion valida debe verificar como verdadera con Marlin");
    }

    /// Confirma la propiedad central de este módulo: el MISMO SRS
    /// universal, generado una sola vez, sirve para indexar y probar un
    /// circuito con parámetros distintos (otra transacción), sin repetir
    /// el setup. Esto es justo lo que Groth16 no puede hacer.
    #[test]
    fn same_universal_srs_serves_multiple_circuit_instances() {
        let srs = universal_setup(MAX_CONSTRAINTS, MAX_VARIABLES, MAX_NON_ZERO, 2)
            .expect("el setup universal no deberia fallar");

        let circuit_a = ComplianceCircuit::<Fr>::new(1_000_000, 100_000, 500_000);
        let circuit_b = ComplianceCircuit::<Fr>::new(2_000_000, 300_000, 500_000);

        let (pk_a, vk_a) = index_circuit(&srs, circuit_a.clone()).expect("indexar A no deberia fallar");
        let (pk_b, vk_b) = index_circuit(&srs, circuit_b.clone()).expect("indexar B no deberia fallar");

        let proof_a = prove(&pk_a, circuit_a, 10).expect("prove A no deberia fallar");
        let proof_b = prove(&pk_b, circuit_b, 11).expect("prove B no deberia fallar");

        let public_input = vec![Fr::from(500_000u64)];
        assert!(verify(&vk_a, &public_input, &proof_a, 20).unwrap());
        assert!(verify(&vk_b, &public_input, &proof_b, 21).unwrap());
    }
}
