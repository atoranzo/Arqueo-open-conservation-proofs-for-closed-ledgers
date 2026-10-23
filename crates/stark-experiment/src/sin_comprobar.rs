//! **Un probador que oculta y NO comprueba la traza**, para que un falsador llegue al juez del
//! kit.
//!
//! El juez del kit (`zk-ssl-air`) solo acepta pruebas OCULTAS: la marca de la ocultacion es parte
//! de la forma (§651), y una prueba de [`crate::sin_ocultar::SinOcultar`] no la lleva. El probador
//! oculto del fork, en cambio, comprueba la traza real contra el AIR antes de probar, tambien en
//! release (D-AH), y un testigo malo no llega a ser prueba. Esa comprobacion es un metodo con
//! cuerpo del trait `Trace` (`Trace::comprobar`), asi que un probador malicioso la salta con su
//! propio tipo de traza: [`TrazaSinComprobar`] envuelve la traza de la casa, lo delega todo y
//! devuelve `Ok` sin mirar. [`OcultaSinComprobar`] envuelve el probador de un circuito: la misma
//! AIR, la misma ocultacion, el mismo compromiso con sal y las mismas opciones. Cambia dos cosas:
//! el tipo de la traza, y el cociente. Con una traza mala el cociente no es un polinomio de grado
//! bajo, y el cociente oculto del fork lo asierta (`segmentar`, en `winter-prover`). El malicioso
//! recorta lo que no cabe -los coeficientes desde `(T - m) * trozos`- y prueba con el resto: es
//! un cociente que no es el de sus restricciones. Lo unico que puede rechazar su prueba es el juez.
//!
//! ⚠️ En depuracion winterfell comprueba los grados de las restricciones al evaluarlas
//! (`evaluation_table.rs`: «transition constraint degrees didn't match», medido) y hace panico
//! antes de probar. Los falsadores que lo usan corren solo en release.

use winterfell::math::{fft, FieldElement};
use winterfell::matrix::ColMatrix;
use winterfell::{
    Air, AuxRandElements, CompositionPoly, CompositionPolyTrace,
    ConstraintCompositionCoefficients, EvaluationFrame, PartitionOptions, ProofOptions, Prover,
    StarkDomain, Trace, TraceInfo, TracePolyTable,
};

/// La traza `T` sin la comprobacion previa: todo igual salvo `comprobar`, que no mira nada.
pub(crate) struct TrazaSinComprobar<T>(pub T);

impl<T: Trace> Trace for TrazaSinComprobar<T> {
    type BaseField = T::BaseField;

    fn info(&self) -> &TraceInfo {
        self.0.info()
    }

    fn main_segment(&self) -> &ColMatrix<Self::BaseField> {
        self.0.main_segment()
    }

    fn read_main_frame(&self, row_idx: usize, frame: &mut EvaluationFrame<Self::BaseField>) {
        self.0.read_main_frame(row_idx, frame)
    }

    /// Lo unico que cambia: la traza no se comprueba contra el AIR antes de probarla.
    fn comprobar<A, E>(
        &self,
        _air: &A,
        _aux: Option<(&ColMatrix<E>, &AuxRandElements<E>)>,
    ) -> Result<(), winter_prover::ProverError>
    where
        A: Air<BaseField = Self::BaseField>,
        E: FieldElement<BaseField = Self::BaseField>,
    {
        Ok(())
    }
}

/// El probador `P`, con la ocultacion de `P` y sin la comprobacion de la traza.
pub(crate) struct OcultaSinComprobar<P>(pub P);

impl<P: Prover> Prover for OcultaSinComprobar<P> {
    type BaseField = P::BaseField;
    type Air = P::Air;
    type Trace = TrazaSinComprobar<P::Trace>;
    type HashFn = P::HashFn;
    type VC = P::VC;
    type RandomCoin = P::RandomCoin;
    type TraceLde<E: FieldElement<BaseField = Self::BaseField>> = P::TraceLde<E>;
    type ConstraintEvaluator<'a, E: FieldElement<BaseField = Self::BaseField>> =
        P::ConstraintEvaluator<'a, E>;
    type ConstraintCommitment<E: FieldElement<BaseField = Self::BaseField>> =
        P::ConstraintCommitment<E>;

    fn get_pub_inputs(&self, trace: &Self::Trace) -> <Self::Air as Air>::PublicInputs {
        self.0.get_pub_inputs(&trace.0)
    }

    fn options(&self) -> &ProofOptions {
        self.0.options()
    }

    fn ocultacion(&self) -> Option<winter_prover::Ocultacion> {
        self.0.ocultacion()
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
        // El cociente recortado a lo que cabe en los trozos del cociente oculto (sin ocultar,
        // `m = 0`, se queda entero).
        let mut c = composition_poly_trace.into_inner();
        let n = c.len();
        let offset = domain.offset();
        let inversos = fft::get_inv_twiddles::<Self::BaseField>(n);
        fft::interpolate_poly_with_offset(&mut c, &inversos, offset);
        let m = domain.cociente_m();
        if m > 0 {
            let cabe = (domain.trace_length() - m) * num_constraint_composition_columns;
            for x in c.iter_mut().skip(cabe) {
                *x = E::ZERO;
            }
        }
        let recortado =
            fft::evaluate_poly_with_offset(&c, &fft::get_twiddles::<Self::BaseField>(n), offset, 1);
        self.0.build_constraint_commitment(
            CompositionPolyTrace::new(recortado),
            num_constraint_composition_columns,
            domain,
            partition_options,
        )
    }
}
