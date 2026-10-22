//! **Un probador de la casa con la ocultación apagada**, para que un falsador llegue al
//! verificador.
//!
//! Los probadores de los circuitos vivos ocultan (`ocultacion_encendida`), y el probador oculto
//! del fork comprueba la traza real contra el AIR ANTES de probar, también en release (D-AH): un
//! testigo malo es un `Err` del probador honesto y la prueba no llega a existir. Un falsador
//! escrito con ese probador mide al probador, no al verificador. Un probador malicioso se salta
//! esa comprobación.
//!
//! [`SinOcultar`] envuelve el probador de un circuito y lo delega todo salvo la ocultación, que
//! devuelve `None`: la prueba es la de winterfell, que en release no comprueba la traza. Así una
//! traza mala se prueba sin queja, y lo único que puede rechazarla es `verify`.
//!
//! ⚠️ En depuración winterfell valida la traza al probar (`Trace::validate`) y hace pánico antes
//! de llegar al verificador. Los falsadores que lo usan corren sólo en release.

use winterfell::math::FieldElement;
use winterfell::matrix::ColMatrix;
use winterfell::{
    Air, AuxRandElements, CompositionPoly, CompositionPolyTrace,
    ConstraintCompositionCoefficients, PartitionOptions, ProofOptions, Prover, StarkDomain,
    TraceInfo, TracePolyTable,
};

/// El probador `P`, con la ocultación apagada y todo lo demás igual: la misma AIR, el mismo
/// compromiso con sal y las mismas opciones.
pub(crate) struct SinOcultar<P>(pub P);

impl<P: Prover> Prover for SinOcultar<P> {
    type BaseField = P::BaseField;
    type Air = P::Air;
    type Trace = P::Trace;
    type HashFn = P::HashFn;
    type VC = P::VC;
    type RandomCoin = P::RandomCoin;
    type TraceLde<E: FieldElement<BaseField = Self::BaseField>> = P::TraceLde<E>;
    type ConstraintEvaluator<'a, E: FieldElement<BaseField = Self::BaseField>> =
        P::ConstraintEvaluator<'a, E>;
    type ConstraintCommitment<E: FieldElement<BaseField = Self::BaseField>> =
        P::ConstraintCommitment<E>;

    fn get_pub_inputs(&self, trace: &Self::Trace) -> <Self::Air as Air>::PublicInputs {
        self.0.get_pub_inputs(trace)
    }

    fn options(&self) -> &ProofOptions {
        self.0.options()
    }

    /// Lo único que cambia: la prueba de winterfell, sin la comprobación previa de la traza.
    fn ocultacion(&self) -> Option<winter_prover::Ocultacion> {
        None
    }

    fn new_trace_lde<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        trace_info: &TraceInfo,
        main_trace: &ColMatrix<Self::BaseField>,
        domain: &StarkDomain<Self::BaseField>,
        partition_option: PartitionOptions,
    ) -> (Self::TraceLde<E>, TracePolyTable<E>) {
        self.0.new_trace_lde(trace_info, main_trace, domain, partition_option)
    }

    fn new_evaluator<'a, E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        air: &'a Self::Air,
        aux_rand_elements: Option<AuxRandElements<E>>,
        composition_coefficients: ConstraintCompositionCoefficients<E>,
    ) -> Self::ConstraintEvaluator<'a, E> {
        self.0.new_evaluator(air, aux_rand_elements, composition_coefficients)
    }

    fn build_constraint_commitment<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        composition_poly_trace: CompositionPolyTrace<E>,
        num_constraint_composition_columns: usize,
        domain: &StarkDomain<Self::BaseField>,
        partition_options: PartitionOptions,
    ) -> (Self::ConstraintCommitment<E>, CompositionPoly<E>) {
        self.0.build_constraint_commitment(
            composition_poly_trace,
            num_constraint_composition_columns,
            domain,
            partition_options,
        )
    }

    fn build_aux_trace<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        main_trace: &Self::Trace,
        aux_rand_elements: &AuxRandElements<E>,
    ) -> ColMatrix<E> {
        self.0.build_aux_trace(main_trace, aux_rand_elements)
    }
}
