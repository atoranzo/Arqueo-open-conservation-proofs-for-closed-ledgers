// SPIKE-B-P4. El envoltorio del lateral (sesion 166): presenta al nucleo un AIR de longitud 2T
// con una columna mas y las exenciones del AIR interno mas T; delega en el las transiciones, las
// aserciones y las columnas periodicas, y el interno sigue viendo T, asi que sus aserciones de
// ultima fila no se mueven. El probador pone las T filas y la columna aleatorias; el verificador
// lo reconstruye desde la prueba por la marca del meta de la traza (ARQUEO, RFC-0009 E3a; desde
// el asiento 534 la marca lleva m y el envoltorio sube el ce siempre que haga falta).

use alloc::vec::Vec;

use math::{ExtensionOf, FieldElement, StarkField};

use crate::{Air, AirContext, Assertion, AuxRandElements, EvaluationFrame, ProofOptions, TraceInfo};

/// SPIKE-B-P4: el AIR `A` con su traza doblada por filas aleatorias y una columna aleatoria.
pub struct Oculta<A: Air> {
    interno: A,
    ctx: AirContext<A::BaseField>,
    cabe: bool,
}

impl<A: Air> Oculta<A> {
    /// SPIKE-B-P4 r2: si la segunda cota de las exenciones cabe con el factor del dominio de
    /// restricciones elegido; el probador no prueba si no cabe.
    pub fn cabe(&self) -> bool {
        self.cabe
    }

    /// ARQUEO (RFC-0009 E3a-2): si la segunda cota de las exenciones (context.rs:315-327) cabe
    /// con el factor `ce`: para cada restriccion, n + eval(L) <= ce * L - 1 + L. Consulta, no
    /// interruptor: el falsador de WorkAir (grado 3 sin ciclos) pregunta con 2, su ce de serie,
    /// y con 4, el que necesita.
    pub fn cabe_con(&self, ce: usize) -> bool {
        cabe_con(&self.ctx, ce)
    }

    /// ARQUEO (RFC-0009 D-AH): el AIR interno, de longitud T, contra el que el probador
    /// comprueba la traza real antes de ocultarla.
    pub fn interno(&self) -> &A {
        &self.interno
    }
}

/// La segunda cota de las exenciones de un contexto con el factor `ce`.
fn cabe_con<B: StarkField>(ctx: &AirContext<B>, ce: usize) -> bool {
    let l = ctx.trace_len();
    let n = ctx.num_transition_exemptions();
    ctx.main_transition_constraint_degrees
        .iter()
        .chain(ctx.aux_transition_constraint_degrees.iter())
        .all(|d| n + d.get_evaluation_degree(l) <= ce * l - 1 + l)
}

impl<A: Air> Air for Oculta<A> {
    type BaseField = A::BaseField;
    type PublicInputs = A::PublicInputs;

    fn new(info: TraceInfo, pub_inputs: Self::PublicInputs, options: ProofOptions) -> Self {
        let l = info.length();
        let t = l / 2;
        let info_interna = TraceInfo::new_multi_segment(
            info.main_trace_width() - 1,
            info.aux_segment_width(),
            info.get_num_aux_segment_rand_elements(),
            t,
            Vec::new(),
        );
        let interno = A::new(info_interna, pub_inputs, options.clone());
        let base = interno.context();
        let n = base.num_transition_exemptions() + t;
        let mut ctx = AirContext::new_multi_segment(
            info,
            base.main_transition_constraint_degrees.clone(),
            base.aux_transition_constraint_degrees.clone(),
            base.num_main_assertions,
            base.num_aux_assertions,
            options,
        );
        // r2: la segunda cota de las exenciones (context.rs:315-327) pide, por restriccion,
        // n <= ce*L - 1 + L - eval(L). Se sube ce a la menor potencia de dos que deje sitio, con
        // tope en el blowup del LDE, siempre (el interruptor del spike murio en el asiento 534).
        // El ce solo lo usa el probador (el dominio donde evalua); por eso las exenciones se
        // escriben sin la asercion y el verificador no entra en panico.
        ctx.num_transition_exemptions = n;
        let mut ce = ctx.ce_blowup_factor;
        while !cabe_con(&ctx, ce) && ce < ctx.options.blowup_factor() {
            ce *= 2;
        }
        let cabe = cabe_con(&ctx, ce);
        ctx.ce_blowup_factor = ce;
        Oculta { interno, ctx, cabe }
    }

    fn context(&self) -> &AirContext<Self::BaseField> {
        &self.ctx
    }

    fn evaluate_transition<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        frame: &EvaluationFrame<E>,
        periodic_values: &[E],
        result: &mut [E],
    ) {
        self.interno.evaluate_transition(frame, periodic_values, result)
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        self.interno.get_assertions()
    }

    fn evaluate_aux_transition<F, E>(
        &self,
        main_frame: &EvaluationFrame<F>,
        aux_frame: &EvaluationFrame<E>,
        periodic_values: &[F],
        aux_rand_elements: &AuxRandElements<E>,
        result: &mut [E],
    ) where
        F: FieldElement<BaseField = Self::BaseField>,
        E: FieldElement<BaseField = Self::BaseField> + ExtensionOf<F>,
    {
        self.interno.evaluate_aux_transition(
            main_frame,
            aux_frame,
            periodic_values,
            aux_rand_elements,
            result,
        )
    }

    fn get_aux_assertions<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        aux_rand_elements: &AuxRandElements<E>,
    ) -> Vec<Assertion<E>> {
        self.interno.get_aux_assertions(aux_rand_elements)
    }

    fn get_periodic_column_values(&self) -> Vec<Vec<Self::BaseField>> {
        self.interno.get_periodic_column_values()
    }
}
