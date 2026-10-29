//! §911 (la 137 de `BACKLOG.md`): la AIR degenerada del falsador 13 —una columna constante con la
//! transicion `next − cur`—, en su fichero porque el barrido de `tools/check_constraint_layout.py`
//! lee un circuito por fichero de `src/`. La usa `falsadores_oculta_periodica.rs`.

use winterfell::crypto::{DefaultRandomCoin, MerkleTree};
use winterfell::math::fields::f128::BaseElement as Base128;
use winterfell::math::FieldElement;
use winterfell::matrix::ColMatrix;
use winterfell::{
    Air, AirContext, Assertion, AuxRandElements, CompositionPoly, CompositionPolyTrace,
    ConstraintCompositionCoefficients, DefaultConstraintCommitment, DefaultConstraintEvaluator,
    DefaultTraceLde, EvaluationFrame, PartitionOptions, ProofOptions, Prover, StarkDomain,
    TraceInfo, TracePolyTable, TraceTable, TransitionConstraintDegree,
};

use crate::falsadores_oculta::{opciones, Blake3W};

/// §911 (la 137): la AIR degenerada, una columna constante con la transicion `next − cur`, sin
/// aserciones: su composicion es identicamente nula y su DEEP no llega al grado L − 2.
pub(crate) struct DegeneradaAir {
    context: AirContext<Base128>,
}

impl Air for DegeneradaAir {
    type BaseField = Base128;
    type PublicInputs = Base128;

    fn new(trace_info: TraceInfo, _pub_inputs: Base128, options: ProofOptions) -> Self {
        assert_eq!(1, trace_info.width());
        DegeneradaAir { context: AirContext::new(trace_info, vec![TransitionConstraintDegree::new(1)], 1, options) }
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
        result[0] = frame.next()[0] - frame.current()[0];
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        vec![Assertion::single(0, 0, Base128::new(7))]
    }
}

pub(crate) struct ProbadorDegenerada;

impl Prover for ProbadorDegenerada {
    type BaseField = Base128;
    type Air = DegeneradaAir;
    type Trace = TraceTable<Base128>;
    type HashFn = Blake3W;
    type VC = MerkleTree<Blake3W>;
    type RandomCoin = DefaultRandomCoin<Blake3W>;
    type TraceLde<E: FieldElement<BaseField = Self::BaseField>> =
        DefaultTraceLde<E, Self::HashFn, Self::VC>;
    type ConstraintEvaluator<'a, E: FieldElement<BaseField = Self::BaseField>> =
        DefaultConstraintEvaluator<'a, Self::Air, E>;
    type ConstraintCommitment<E: FieldElement<BaseField = Self::BaseField>> =
        DefaultConstraintCommitment<E, Self::HashFn, Self::VC>;

    fn get_pub_inputs(&self, _trace: &Self::Trace) -> Base128 {
        Base128::new(7)
    }

    fn options(&self) -> &ProofOptions {
        OPCIONES.get_or_init(opciones)
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
        DefaultConstraintCommitment::new(composition_poly_trace, num_constraint_composition_columns, domain, partition_options)
    }
}

static OPCIONES: std::sync::OnceLock<ProofOptions> = std::sync::OnceLock::new();
