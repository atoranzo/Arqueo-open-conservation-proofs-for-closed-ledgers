//! Circuito mínimo de prueba STARK con Winterfell: demuestra conocimiento
//! de un valor inicial `start` tal que, tras `N` iteraciones de
//! `x = x³ + 42`, se alcanza un `resultado` público — sin trusted setup,
//! basado puramente en hashes (Blake3) y aritmética de campo finito.
//!
//! Es el ejemplo canónico de introducción de Winterfell ("do work"),
//! elegido deliberadamente por ser el más simple posible, antes de
//! acercarnos a nada de la lógica de cumplimiento real.
//!
//! ## ⚠️ Nivel de riesgo: el más alto de los tres experimentos hasta ahora
//!
//! Winterfell usa un paradigma AIR (Algebraic Intermediate
//! Representation) + tabla de traza de ejecución, con más maquinaria de
//! traits (`Air`, `Prover`, `Trace`) que Arkworks o Halo2 en su ejemplo
//! mínimo respectivo. Es razonable esperar MÁS rondas de corrección que
//! con `SquareCircuit` en Halo2, no menos.

//! ## Estado verificado hasta ahora
//! - `WorkAir`/`WorkProver` (circuito mínimo, x=x³+42): ✅ verificado.
//! - `range_check` (range check de 64 bits): ✅ verificado en release
//!   (ver nota sobre trazas degeneradas en ese módulo).
//! - `rescue_hash` (permutación Rescue Prime verificada en AIR): ✅
//!   verificado, incluido el test discriminante de fila corrompida.
//! - `nullifier` (Rescue con separación de dominio): ✅ verificado a la
//!   primera, 6/6 tests.
//! - `solvency` (Etapa 1 del circuito unificado: range checks +
//!   aritmética de solvencia con valores privados): ✅ verificado 7/7 en
//!   debug, tras un rediseño real (acumulación de Horner big-endian para
//!   eliminar la columna `power`, estructuralmente periódica, que rompía
//!   la contabilidad de grados de winterfell).
//! - `compliance_circuit` (Etapa 2: TODO unificado — solvencia + hoja +
//!   árbol + nullifier con columnas de transporte): ✅ verificado 8/8 en
//!   debug A LA PRIMERA. El circuito de cumplimiento completo existe en
//!   los tres paradigmas: R1CS, Plonkish y AIR.
//! - `compliance_real_proof` (pipeline de extremo a extremo con métricas
//!   reales): ✅ verificado. Halazgo clave: sin extensión de campo el
//!   techo son 63 bits (tamaño de Goldilocks); 128 bits DEMOSTRABLES
//!   cuestan 125,6 KB y 45 ms. Ver `THREE_BACKENDS.md`.
//! - `iso_bridge` (puente ISO 20022 pacs.008): ✅ verificado 4/4 a la
//!   primera. Nótese que sus funciones NO reciben claves: la ausencia de
//!   trusted setup se manifiesta en la propia firma de la API.
//! - `persistent_nullifier_registry` (registro sled contra doble gasto):
//!   ✅ verificado 5/5. Con las 8 piezas del plan completas, el circuito
//!   de cumplimiento existe y está verificado en los TRES paradigmas.
//! - `settlement_prover_impl`: implementación del trait
//!   `SettlementProver`, compartido con `zk-core` y `halo2-experiment`.
//!   Pendiente de verificar.
//! - `dual_climb`: subida dual del árbol en lockstep con hermanos
//!   compartidos. ✅ verificado, incluido el test que construye dos
//!   carriles internamente coherentes por CAMINOS DISTINTOS — el ataque
//!   que un diseño secuencial permitiría.
//! - `double_entry`: partida doble completa (transición de estado,
//!   conservación del dinero, solvencia y nullifier). ✅ verificado 8/8.
//! - `nullifier_tree`: no-pertenencia e inserción en lockstep. ✅ 5/5.
//! - `circuit_settlement`: **el circuito de liquidación completo**.
//!   ✅ 13/13. Partida doble, autoridad de gasto, identidades de 256
//!   bits, nullifier derivado de la clave, solvencia, límite regulatorio
//!   y no-pertenencia demostrable. 142 restricciones, 48 columnas, 824
//!   filas activas de 1024.
//!
//!   **Sin ceremonia de confianza y con resistencia cuántica** — las dos
//!   propiedades que llevaron a elegir este paradigma para la capa.

pub mod circuit_audit;
pub mod circuit_burn;
pub mod circuit_freeze;
pub mod circuit_governance;
pub mod circuit_mint;
pub mod circuit_mint_pending;
pub mod circuit_recovery;
pub mod circuit_claim;
pub mod circuit_send;
pub mod circuit_settlement;
pub mod circuit_threshold;
pub mod compliance_circuit;
pub mod compliance_real_proof;
pub mod double_entry;
pub mod dual_climb;
pub mod iso_bridge;
pub mod merkle;
pub mod nullifier;
pub mod nullifier_tree;
pub mod persistent_nullifier_registry;
pub mod range_check;
pub mod rescue_hash;
pub mod settlement_prover_impl;
pub mod solvency;

use winterfell::crypto::{hashers::Blake3_256, DefaultRandomCoin, MerkleTree};
use winterfell::math::{fields::f128::BaseElement, FieldElement};
use winterfell::matrix::ColMatrix;
use winterfell::{
    Air, AirContext, Assertion, AuxRandElements, CompositionPoly,
    CompositionPolyTrace, ConstraintCompositionCoefficients, DefaultConstraintCommitment,
    DefaultConstraintEvaluator, DefaultTraceLde, EvaluationFrame, PartitionOptions, ProofOptions,
    Prover, StarkDomain, Trace, TraceInfo, TracePolyTable, TraceTable,
    TransitionConstraintDegree,
};

const TRACE_WIDTH: usize = 1;

/// Genera la traza de ejecución: `N` iteraciones de `x = x^3 + 42`,
/// partiendo de `start`.
pub fn build_trace(start: BaseElement, n: usize) -> TraceTable<BaseElement> {
    let mut trace = TraceTable::new(TRACE_WIDTH, n);
    trace.fill(
        |state| {
            state[0] = start;
        },
        |_step, state| {
            state[0] = state[0] * state[0] * state[0] + BaseElement::new(42);
        },
    );
    trace
}

pub struct WorkAir {
    context: AirContext<BaseElement>,
    result: BaseElement,
}

impl Air for WorkAir {
    type BaseField = BaseElement;
    type PublicInputs = BaseElement;

    fn new(trace_info: TraceInfo, pub_inputs: BaseElement, options: ProofOptions) -> Self {
        let degrees = vec![TransitionConstraintDegree::new(3)];
        assert_eq!(TRACE_WIDTH, trace_info.width());
        WorkAir {
            context: AirContext::new(trace_info, degrees, 1, options),
            result: pub_inputs,
        }
    }

    fn context(&self) -> &AirContext<Self::BaseField> {
        &self.context
    }

    fn evaluate_transition<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        frame: &EvaluationFrame<E>,
        _periodic_values: &[E],
        result: &mut [E],
    ) {
        let current = frame.current()[0];
        let next = frame.next()[0];
        result[0] = next - (current * current * current + E::from(42u32));
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        let last_step = self.trace_length() - 1;
        vec![Assertion::single(0, last_step, self.result)]
    }
}

type Blake3 = Blake3_256<BaseElement>;

pub struct WorkProver {
    options: ProofOptions,
}

impl WorkProver {
    pub fn new(options: ProofOptions) -> Self {
        Self { options }
    }
}

impl Prover for WorkProver {
    type BaseField = BaseElement;
    type Air = WorkAir;
    type Trace = TraceTable<BaseElement>;
    type HashFn = Blake3;
    type VC = MerkleTree<Blake3>;
    type RandomCoin = DefaultRandomCoin<Blake3>;
    type TraceLde<E: FieldElement<BaseField = Self::BaseField>> =
        DefaultTraceLde<E, Self::HashFn, Self::VC>;
    type ConstraintEvaluator<'a, E: FieldElement<BaseField = Self::BaseField>> =
        DefaultConstraintEvaluator<'a, Self::Air, E>;
    type ConstraintCommitment<E: FieldElement<BaseField = Self::BaseField>> =
        DefaultConstraintCommitment<E, Self::HashFn, Self::VC>;

    fn get_pub_inputs(&self, trace: &Self::Trace) -> BaseElement {
        let last_step = trace.length() - 1;
        trace.get(0, last_step)
    }

    fn options(&self) -> &ProofOptions {
        &self.options
    }

    fn new_trace_lde<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        trace_info: &TraceInfo,
        main_trace: &ColMatrix<Self::BaseField>,
        domain: &StarkDomain<Self::BaseField>,
        partition_option: PartitionOptions,
    ) -> (Self::TraceLde<E>, TracePolyTable<E>) {
        DefaultTraceLde::new(trace_info, main_trace, domain, partition_option)
    }

    fn new_evaluator<'a, E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        air: &'a Self::Air,
        aux_rand_elements: Option<AuxRandElements<E>>,
        composition_coefficients: ConstraintCompositionCoefficients<E>,
    ) -> Self::ConstraintEvaluator<'a, E> {
        DefaultConstraintEvaluator::new(air, aux_rand_elements, composition_coefficients)
    }

    fn build_constraint_commitment<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        composition_poly_trace: CompositionPolyTrace<E>,
        num_constraint_composition_columns: usize,
        domain: &StarkDomain<Self::BaseField>,
        partition_options: PartitionOptions,
    ) -> (Self::ConstraintCommitment<E>, CompositionPoly<E>) {
        DefaultConstraintCommitment::new(
            composition_poly_trace,
            num_constraint_composition_columns,
            domain,
            partition_options,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use winterfell::{verify, AcceptableOptions, BatchingMethod, FieldExtension, Proof};

    fn native_result(start: BaseElement, n: usize) -> BaseElement {
        let mut x = start;
        for _ in 0..(n - 1) {
            x = x * x * x + BaseElement::new(42);
        }
        x
    }

    /// EL TEST CLAVE: genera una prueba STARK real (setup, prove, verify)
    /// para un cómputo válido, sin ningún trusted setup.
    #[test]
    fn valid_computation_produces_verifiable_stark_proof() {
        let start = BaseElement::new(3);
        let n = 8; // numero de pasos, potencia de 2 por requisito de winterfell

        let trace = build_trace(start, n);
        let result = native_result(start, n);

        let options = ProofOptions::new(
            32,   // num_queries
            8,    // blowup_factor
            0,    // grinding_factor
            FieldExtension::None,
            8,    // fri_folding_factor
            31,   // fri_remainder_max_degree
            BatchingMethod::Linear,
            BatchingMethod::Linear,
        );

        let prover = WorkProver::new(options);
        let proof: Proof = prover.prove(trace).expect("la generacion de la prueba no deberia fallar");

        let min_opts = AcceptableOptions::OptionSet(vec![prover.options().clone()]);
        let verification = verify::<WorkAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof,
            result,
            &min_opts,
        );

        assert!(
            verification.is_ok(),
            "una prueba STARK real de un computo valido deberia verificar correctamente: {verification:?}"
        );
    }
}
