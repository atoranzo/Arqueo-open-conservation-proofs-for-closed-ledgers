// SPIKE-B-P4. El envoltorio del lateral (sesion 166): presenta al nucleo un AIR de longitud 2T
// con una columna mas y las exenciones del AIR interno mas T; delega en el las transiciones, las
// aserciones y las columnas periodicas, y el interno sigue viendo T, asi que sus aserciones de
// ultima fila no se mueven. El probador pone las T filas y la columna aleatorias; el verificador
// lo reconstruye desde la prueba por la marca del meta de la traza (ARQUEO, RFC-0009 E3a; desde
// el asiento 534 la marca lleva m y el envoltorio sube el ce siempre que haga falta).

use alloc::vec::Vec;

use crypto::{RandomCoin, RandomCoinError};
use math::{ExtensionOf, FieldElement, StarkField};

use crate::{
    Air, AirContext, Assertion, AuxRandElements, BatchingMethod, DeepCompositionCoefficients,
    EvaluationFrame, ProofOptions, TraceInfo,
};

/// SPIKE-B-P4: el AIR `A` con su traza doblada por filas aleatorias y una columna aleatoria.
pub struct Oculta<A: Air> {
    interno: A,
    ctx: AirContext<A::BaseField>,
    cabe: bool,
    /// §899: las columnas aleatorias que dice la marca.
    columnas: usize,
    /// §909 (RFC-0009 D-AL bis): si las columnas aleatorias, cuando son dos o mas, entran en DEEP
    /// pareadas como una sobre E. Siempre, salvo en el laboratorio que mide que lo estan.
    parear: bool,
}

/// ARQUEO (§909; RFC-0009 D-AL bis): ν, el elemento de E fuera de F con el que se parean las
/// columnas aleatorias: el que tiene la primera coordenada sobre F a cero y la segunda a uno.
/// `None` sin extension, donde no hay con que parear.
pub fn nu_de_pareado<E: FieldElement>() -> Option<E> {
    if E::EXTENSION_DEGREE < 2 {
        return None;
    }
    let mut base = alloc::vec![E::BaseField::ZERO; E::EXTENSION_DEGREE];
    base[1] = E::BaseField::ONE;
    Some(E::slice_from_base_elements(&base)[0])
}

/// ARQUEO (§909; RFC-0009 D-AL bis): parea los coeficientes de DEEP de las `columnas` aleatorias,
/// que son las ultimas de la traza principal tras las `reales`: la k-esima lleva cc·ν^k, con cc el
/// de la primera. Con dos columnas, R₀ + ν·R₁ es una columna sobre E con un solo coeficiente, y el
/// lema L9 de `doc/ocultacion-hvzk.md` hace de R3 un teorema. Con una columna, o sin extension,
/// no toca nada.
pub fn parear_deep<E: FieldElement>(cc: &mut [E], reales: usize, columnas: usize) {
    if columnas < 2 {
        return;
    }
    if let Some(nu) = nu_de_pareado::<E>() {
        let primero = cc[reales];
        let mut potencia = E::ONE;
        for k in 1..columnas {
            potencia *= nu;
            cc[reales + k] = primero * potencia;
        }
    }
}

impl<A: Air> Oculta<A> {
    /// §909: el envoltorio de LABORATORIO que no parea las columnas aleatorias en DEEP: con el,
    /// una prueba hecha por el probador -que parea siempre- no verifica, y eso es lo que mide que
    /// el pareado esta en vigor. Ningun juez de produccion lo usa.
    pub fn sin_parear(info: TraceInfo, pub_inputs: A::PublicInputs, options: ProofOptions) -> Self {
        let mut oculta = Self::new(info, pub_inputs, options);
        oculta.parear = false;
        oculta
    }

    /// §899: las columnas aleatorias que lleva la traza principal, por la marca.
    pub fn columnas(&self) -> usize {
        self.columnas
    }

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
        // §899 (la 133): las columnas aleatorias las dice la marca: una la de produccion, mas la
        // de laboratorio; sin marca no hay envoltorio, y quien llama ya lo comprobo
        let columnas = crate::Marca::columnas_de(info.meta());
        assert!(columnas >= 1, "Oculta: el meta de la traza no es una marca");
        let info_interna = TraceInfo::new_multi_segment(
            info.main_trace_width() - columnas,
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
        Oculta { interno, ctx, cabe, columnas, parear: true }
    }

    /// ARQUEO (§909; RFC-0009 D-AL bis): los coeficientes de DEEP como los saca winterfell -el
    /// mismo lote y el mismo orden en la moneda-, y despues las columnas aleatorias PAREADAS: con
    /// dos o mas, la k-esima lleva cc·ν^k, el coeficiente de la primera por las potencias de
    /// ν ∈ E \ F, y las columnas son una sola sobre E (L9). El probador y el verificador pasan
    /// por aqui, asi que la prueba no cambia de forma: solo de combinacion.
    fn get_deep_composition_coefficients<E, R>(
        &self,
        public_coin: &mut R,
    ) -> Result<DeepCompositionCoefficients<E>, RandomCoinError>
    where
        E: FieldElement<BaseField = Self::BaseField>,
        R: RandomCoin<BaseField = Self::BaseField>,
    {
        let anchura = self.trace_info().width();
        let trozos = self.context().num_constraint_composition_columns();
        let mut cc = match self.context().options.deep_poly_batching_method() {
            BatchingMethod::Linear => DeepCompositionCoefficients::draw_linear(public_coin, anchura, trozos),
            BatchingMethod::Algebraic => {
                DeepCompositionCoefficients::draw_algebraic(public_coin, anchura, trozos)
            },
            BatchingMethod::Horner => DeepCompositionCoefficients::draw_horner(public_coin, anchura, trozos),
        }?;
        if self.parear {
            let reales = self.trace_info().main_trace_width() - self.columnas;
            parear_deep(&mut cc.trace, reales, self.columnas);
        }
        Ok(cc)
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

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use math::fields::f64::BaseElement;
    use math::fields::QuadExtension;
    use math::FieldElement;

    use super::{nu_de_pareado, parear_deep};

    /// **§909 (RFC-0009 D-AL bis): ν es (0, 1) sobre la extension cuadratica y no existe sobre F,
    /// y el pareado pone cc·ν en la segunda columna aleatoria, cc·ν² en una tercera, y nada con
    /// una columna.**
    #[test]
    fn nu_y_el_pareado_de_las_columnas_aleatorias() {
        type E = QuadExtension<BaseElement>;
        let nu = nu_de_pareado::<E>().expect("hay ν sobre la extension cuadratica");
        assert_eq!(nu, E::new(BaseElement::ZERO, BaseElement::ONE));
        assert_ne!(nu, E::from(BaseElement::ONE), "ν no esta en F");
        assert!(nu_de_pareado::<BaseElement>().is_none(), "sin extension no hay ν");
        let cc0: Vec<E> = (1..=6u64).map(|i| E::new(BaseElement::new(i), BaseElement::new(10 + i))).collect();
        // tres columnas reales y dos aleatorias: la cuarta y la quinta
        let mut cc = cc0.clone();
        parear_deep(&mut cc, 3, 2);
        assert_eq!(&cc[..4], &cc0[..4], "las reales y la primera aleatoria no cambian");
        assert_eq!(cc[4], cc0[3] * nu);
        assert_eq!(cc[5], cc0[5], "lo que hay detras -los trozos- no cambia");
        // tres aleatorias: cc, cc·ν, cc·ν²
        let mut cc = cc0.clone();
        parear_deep(&mut cc, 2, 3);
        assert_eq!((cc[2], cc[3], cc[4]), (cc0[2], cc0[2] * nu, cc0[2] * nu * nu));
        // una columna: nada
        let mut cc = cc0.clone();
        parear_deep(&mut cc, 4, 1);
        assert_eq!(cc, cc0);
        // sobre F, dos columnas: nada que parear
        let mut f: Vec<BaseElement> = (1..=4u64).map(BaseElement::new).collect();
        let f0 = f.clone();
        parear_deep(&mut f, 2, 2);
        assert_eq!(f, f0);
    }
}
