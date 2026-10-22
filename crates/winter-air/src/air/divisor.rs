// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use alloc::vec::Vec;
use core::fmt::{Display, Formatter};

use math::{FieldElement, StarkField};

use crate::air::Assertion;

// CONSTRAINT DIVISOR
// ================================================================================================
/// The denominator portion of boundary and transition constraints.
///
/// A divisor is described by a combination of a sparse polynomial, which describes the numerator
/// of the divisor and a set of exemption points, which describe the denominator of the divisor.
/// The numerator polynomial is described as multiplication of tuples where each tuple encodes
/// an expression $(x^a - b)$. The exemption points encode expressions $(x - a)$.
///
/// For example divisor $(x^a - 1) \cdot (x^b - 2) / (x - 3)$ can be represented as:
/// numerator: `[(a, 1), (b, 2)]`, exemptions: `[3]`.
///
/// A divisor cannot be instantiated directly, and instead must be created either for an
/// [Assertion] or for a transition constraint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstraintDivisor<B: StarkField> {
    pub(super) numerator: Vec<(usize, B)>,
    pub(super) exemptions: Vec<B>,
    // ARQUEO (§717, el coste del juez de la edad): las exenciones de una transicion son
    // un bloque contiguo, y se guardan como bloque, sin lista. Ver [BloqueDeExenciones].
    pub(super) bloque: Option<BloqueDeExenciones<B>>,
}

/// ARQUEO (§717, el coste del juez de la edad): las `cuantas` ultimas filas del dominio de
/// la traza como exenciones, `primera * paso^i` para `i` en `0..cuantas`, sin materializarlas.
///
/// Upstream guarda la lista de puntos: `cuantas` exponenciaciones al construir el divisor y
/// `cuantas` elementos en memoria. Con un numero fijo de exenciones (una o dos, las de un AIR de
/// upstream) no importa. El envoltorio de ocultacion `Oculta` declara `1 + L/2` (la mitad de la
/// traza doblada), y entonces el verificador pagaba CPU y memoria lineales en la traza ANTES de la
/// primera comprobacion contra la prueba. Con el bloque, construir el divisor es una
/// exponenciacion, y evaluarlo, `cuantas` productos en flujo, sin memoria.
///
/// ⚠️ Evaluarlo sigue siendo Θ(cuantas): lo que acota el coste de un juez es el tope del
/// enunciado que lo llama, no este tipo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BloqueDeExenciones<B: StarkField> {
    /// `g^(L - cuantas)`, con `g` el generador del dominio de la traza y `L` su longitud.
    pub primera: B,
    /// `g`.
    pub paso: B,
    /// El numero de exenciones.
    pub cuantas: usize,
}

impl<B: StarkField> ConstraintDivisor<B> {
    // CONSTRUCTORS
    // --------------------------------------------------------------------------------------------

    /// Returns a new divisor instantiated from the provided parameters.
    fn new(numerator: Vec<(usize, B)>, exemptions: Vec<B>) -> Self {
        ConstraintDivisor { numerator, exemptions, bloque: None }
    }

    /// Builds a divisor for transition constraints.
    ///
    /// For transition constraints, the divisor polynomial $z(x)$ is always the same:
    ///
    /// $$ z(x) = \frac{x^n - 1}{ \prod_{i=1}^k (x - g^{n-i})} $$
    ///
    /// where, $n$ is the length of the execution trace, $g$ is the generator of the trace domain,
    /// and $k$ is the number of exemption points. The default value for $k$ is $1$.
    ///
    /// The above divisor specifies that transition constraints must hold on all steps of the
    /// constraint enforcement domain except for the last $k$ steps.
    ///
    /// ARQUEO (§717, el coste del juez de la edad): las exenciones van como
    /// [BloqueDeExenciones], no como lista: una exponenciacion en lugar de `k`. Los valores de
    /// [Self::evaluate_at] y de [Self::evaluate_exemptions_at] son los de upstream, elemento a
    /// elemento (lo atan los tests de este fichero).
    pub fn from_transition(
        constraint_enforcement_domain_size: usize,
        num_exemptions: usize,
    ) -> Self {
        let l = constraint_enforcement_domain_size;
        let bloque = if num_exemptions == 0 {
            None
        } else {
            Some(BloqueDeExenciones {
                primera: get_trace_domain_value_at::<B>(l, l - num_exemptions),
                paso: B::get_root_of_unity(l.ilog2()),
                cuantas: num_exemptions,
            })
        };
        ConstraintDivisor {
            numerator: vec![(l, B::ONE)],
            exemptions: Vec::new(),
            bloque,
        }
    }

    /// Builds a divisor for a boundary constraint described by the assertion.
    ///
    /// For boundary constraints, the divisor polynomial is defined as:
    ///
    /// $$
    /// z(x) = x^k - g^{a \cdot k}
    /// $$
    ///
    /// where $g$ is the generator of the trace domain, $k$ is the number of asserted steps, and
    /// $a$ is the step offset in the trace domain. Specifically:
    /// * For an assertion against a single step, the polynomial is $(x - g^a)$, where $a$ is the
    ///   step on which the assertion should hold.
    /// * For an assertion against a sequence of steps which fall on powers of two, it is
    ///   $(x^k - 1)$ where $k$ is the number of asserted steps.
    /// * For assertions against a sequence of steps which repeat with a period that is a power
    ///   of two but don't fall exactly on steps which are powers of two (e.g. 1, 9, 17, ... )
    ///   it is $(x^k - g^{a \cdot k})$, where $a$ is the number of steps by which the assertion steps
    ///   deviate from a power of two, and $k$ is the number of asserted steps. This is equivalent to
    ///   $(x - g^a) \cdot (x - g^{a + j}) \cdot (x - g^{a + 2 \cdot j}) ... (x - g^{a + (k  - 1) \cdot j})$,
    ///   where $j$ is the length of interval between asserted steps (e.g. 8).
    ///
    /// # Panics
    /// Panics of the specified `trace_length` is inconsistent with the specified `assertion`.
    pub fn from_assertion<E>(assertion: &Assertion<E>, trace_length: usize) -> Self
    where
        E: FieldElement<BaseField = B>,
    {
        let num_steps = assertion.get_num_steps(trace_length);
        if assertion.first_step == 0 {
            Self::new(vec![(num_steps, B::ONE)], vec![])
        } else {
            let trace_offset = num_steps * assertion.first_step;
            let offset = get_trace_domain_value_at::<B>(trace_length, trace_offset);
            Self::new(vec![(num_steps, offset)], vec![])
        }
    }

    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Returns the numerator portion of this constraint divisor.
    pub fn numerator(&self) -> &[(usize, B)] {
        &self.numerator
    }

    /// ARQUEO (§717, el coste del juez de la edad): el numero de exenciones (el grado del
    /// denominador). Sustituye a `exemptions()`, que devolvia la lista: la lista ya no existe para
    /// las transiciones, y materializarla es justo el coste que el bloque quita.
    pub fn num_exemptions(&self) -> usize {
        self.exemptions.len() + self.bloque.map_or(0, |b| b.cuantas)
    }

    /// ARQUEO (§717, el coste del juez de la edad): el bloque de exenciones, si el
    /// denominador es SOLO un bloque contiguo (el de una transicion). El probador lo usa para
    /// evaluar las exenciones en todo su dominio por recurrencia.
    pub fn bloque_de_exenciones(&self) -> Option<BloqueDeExenciones<B>> {
        if self.exemptions.is_empty() {
            self.bloque
        } else {
            None
        }
    }

    /// Returns the degree of the divisor polynomial
    pub fn degree(&self) -> usize {
        let numerator_degree = self.numerator.iter().fold(0, |degree, term| degree + term.0);
        let denominator_degree = self.num_exemptions();
        numerator_degree - denominator_degree
    }

    // EVALUATORS
    // --------------------------------------------------------------------------------------------
    /// Evaluates the divisor polynomial at the provided `x` coordinate.
    pub fn evaluate_at<E: FieldElement<BaseField = B>>(&self, x: E) -> E {
        // compute the numerator value
        let mut numerator = E::ONE;
        for (degree, constant) in self.numerator.iter() {
            let v = x.exp((*degree as u32).into());
            let v = v - E::from(*constant);
            numerator *= v;
        }

        // compute the denominator value
        let denominator = self.evaluate_exemptions_at(x);

        numerator / denominator
    }

    /// Evaluates the denominator of this divisor (the exemption points) at the provided `x`
    /// coordinate.
    ///
    /// ARQUEO (§717, el coste del juez de la edad): el bloque se recorre en flujo, un
    /// producto por exencion y sin lista, en el orden de upstream (de `g^(L - k)` a `g^(L - 1)`).
    #[inline(always)]
    pub fn evaluate_exemptions_at<E: FieldElement<BaseField = B>>(&self, x: E) -> E {
        let mut r = self.exemptions.iter().fold(E::ONE, |r, &e| r * (x - E::from(e)));
        if let Some(b) = self.bloque {
            let mut p = b.primera;
            for _ in 0..b.cuantas {
                r *= x - E::from(p);
                p *= b.paso;
            }
        }
        r
    }
}

impl<B: StarkField> Display for ConstraintDivisor<B> {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        for (degree, offset) in self.numerator.iter() {
            write!(f, "(x^{degree} - {offset})")?;
        }
        if self.num_exemptions() > 0 {
            write!(f, " / ")?;
            for x in self.exemptions.iter() {
                write!(f, "(x - {x})")?;
            }
            // ARQUEO (§717, el coste del juez de la edad): el bloque, sin desplegarlo.
            if let Some(b) = self.bloque {
                write!(f, "prod_{{i < {}}}(x - {} * {}^i)", b.cuantas, b.primera, b.paso)?;
            }
        }
        Ok(())
    }
}

// HELPER FUNCTIONS
// ================================================================================================

/// Returns g^step, where g is the generator of trace domain.
fn get_trace_domain_value_at<B: StarkField>(trace_length: usize, step: usize) -> B {
    debug_assert!(step < trace_length, "step must be in the trace domain [0, {trace_length})");
    let g = B::get_root_of_unity(trace_length.ilog2());
    g.exp((step as u64).into())
}

// TESTS
// ================================================================================================

#[cfg(test)]
mod tests {
    use math::{fields::f128::BaseElement, polynom};

    use super::*;

    #[test]
    fn constraint_divisor_degree() {
        // single term numerator
        let div = ConstraintDivisor::new(vec![(4, BaseElement::ONE)], vec![]);
        assert_eq!(4, div.degree());

        // multi-term numerator
        let div = ConstraintDivisor::new(
            vec![(4, BaseElement::ONE), (2, BaseElement::new(2)), (3, BaseElement::new(3))],
            vec![],
        );
        assert_eq!(9, div.degree());

        // multi-term numerator with exemption points
        let div = ConstraintDivisor::new(
            vec![(4, BaseElement::ONE), (2, BaseElement::new(2)), (3, BaseElement::new(3))],
            vec![BaseElement::ONE, BaseElement::new(2)],
        );
        assert_eq!(7, div.degree());
    }

    #[test]
    fn constraint_divisor_evaluation() {
        // single term numerator: (x^4 - 1)
        let div = ConstraintDivisor::new(vec![(4, BaseElement::ONE)], vec![]);
        assert_eq!(BaseElement::new(15), div.evaluate_at(BaseElement::new(2)));

        // multi-term numerator: (x^4 - 1) * (x^2 - 2) * (x^3 - 3)
        let div = ConstraintDivisor::new(
            vec![(4, BaseElement::ONE), (2, BaseElement::new(2)), (3, BaseElement::new(3))],
            vec![],
        );
        let expected = BaseElement::new(15) * BaseElement::new(2) * BaseElement::new(5);
        assert_eq!(expected, div.evaluate_at(BaseElement::new(2)));

        // multi-term numerator with exemption points:
        // (x^4 - 1) * (x^2 - 2) * (x^3 - 3) / ((x - 1) * (x - 2))
        let div = ConstraintDivisor::new(
            vec![(4, BaseElement::ONE), (2, BaseElement::new(2)), (3, BaseElement::new(3))],
            vec![BaseElement::ONE, BaseElement::new(2)],
        );
        let expected = BaseElement::new(255) * BaseElement::new(14) * BaseElement::new(61)
            / BaseElement::new(6);
        assert_eq!(expected, div.evaluate_at(BaseElement::new(4)));
    }

    #[test]
    fn constraint_divisor_equivalence() {
        let n = 8_usize;
        let g = BaseElement::get_root_of_unity(n.trailing_zeros());
        let k = 4_u32;
        let j = n as u32 / k;

        // ----- periodic assertion divisor, no offset --------------------------------------------

        // create a divisor for assertion which repeats every 2 steps starting at step 0
        let assertion = Assertion::periodic(0, 0, j as usize, BaseElement::ONE);
        let divisor = ConstraintDivisor::from_assertion(&assertion, n);

        // z(x) = x^4 - 1 = (x - 1) * (x - g^2) * (x - g^4) * (x - g^6)
        let poly = polynom::mul(
            &polynom::mul(
                &[-BaseElement::ONE, BaseElement::ONE],
                &[-g.exp(j.into()), BaseElement::ONE],
            ),
            &polynom::mul(
                &[-g.exp((2 * j).into()), BaseElement::ONE],
                &[-g.exp((3 * j).into()), BaseElement::ONE],
            ),
        );

        for i in 0..n {
            let expected = polynom::eval(&poly, g.exp((i as u32).into()));
            let actual = divisor.evaluate_at(g.exp((i as u32).into()));
            assert_eq!(expected, actual);
            if i.is_multiple_of(j as usize) {
                assert_eq!(BaseElement::ZERO, actual);
            }
        }

        // ----- periodic assertion divisor, with offset ------------------------------------------

        // create a divisor for assertion which repeats every 2 steps starting at step 1
        let offset = 1_u32;
        let assertion = Assertion::periodic(0, offset as usize, j as usize, BaseElement::ONE);
        let divisor = ConstraintDivisor::from_assertion(&assertion, n);
        assert_eq!(ConstraintDivisor::new(vec![(k as usize, g.exp(k.into()))], vec![]), divisor);

        // z(x) = x^4 - g^4 = (x - g) * (x - g^3) * (x - g^5) * (x - g^7)
        let poly = polynom::mul(
            &polynom::mul(
                &[-g.exp(offset.into()), BaseElement::ONE],
                &[-g.exp((offset + j).into()), BaseElement::ONE],
            ),
            &polynom::mul(
                &[-g.exp((offset + 2 * j).into()), BaseElement::ONE],
                &[-g.exp((offset + 3 * j).into()), BaseElement::ONE],
            ),
        );

        for i in 0..n {
            let expected = polynom::eval(&poly, g.exp((i as u32).into()));
            let actual = divisor.evaluate_at(g.exp((i as u32).into()));
            assert_eq!(expected, actual);
            if i % (j as usize) == offset as usize {
                assert_eq!(BaseElement::ZERO, actual);
            }
        }

        // create a divisor for assertion which repeats every 4 steps starting at step 3
        let offset = 3_u32;
        let k = 2_u32;
        let j = n as u32 / k;
        let assertion = Assertion::periodic(0, offset as usize, j as usize, BaseElement::ONE);
        let divisor = ConstraintDivisor::from_assertion(&assertion, n);
        assert_eq!(
            ConstraintDivisor::new(vec![(k as usize, g.exp((offset * k).into()))], vec![]),
            divisor
        );

        // z(x) = x^2 - g^6 = (x - g^3) * (x - g^7)
        let poly = polynom::mul(
            &[-g.exp(offset.into()), BaseElement::ONE],
            &[-g.exp((offset + j).into()), BaseElement::ONE],
        );

        for i in 0..n {
            let expected = polynom::eval(&poly, g.exp((i as u32).into()));
            let actual = divisor.evaluate_at(g.exp((i as u32).into()));
            assert_eq!(expected, actual);
            if i % (j as usize) == offset as usize {
                assert_eq!(BaseElement::ZERO, actual);
            }
        }
    }
}

// ARQUEO (§717, el coste del juez de la edad): los testigos del bloque de exenciones. Los
// tests de upstream, arriba, no cambian.
#[cfg(test)]
mod tests_bloque {
    use math::fields::{f64::BaseElement, QuadExtension};

    use super::*;

    type E = QuadExtension<BaseElement>;

    /// El divisor de upstream: la lista de las `n` ultimas filas, una exponenciacion por fila.
    fn de_upstream(l: usize, n: usize) -> ConstraintDivisor<BaseElement> {
        let lista =
            (l - n..l).map(|step| get_trace_domain_value_at::<BaseElement>(l, step)).collect();
        ConstraintDivisor::new(vec![(l, BaseElement::ONE)], lista)
    }

    /// Puntos de la extension que no salen de ninguna estructura: un generador lineal congruente.
    fn puntos(k: usize) -> Vec<E> {
        let mut s = 0x9E37_79B9_7F4A_7C15u64;
        let mut siguiente = move || {
            s = s.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
            BaseElement::new(s >> 1)
        };
        (0..k).map(|_| E::new(siguiente(), siguiente())).collect()
    }

    /// El bloque da lo que daba la lista, elemento a elemento: el grado, las exenciones y el
    /// divisor entero, en puntos de la extension y en todo el dominio de la traza hasta `2^8`
    /// (donde las exenciones se anulan justo en las filas exentas). Longitudes de `2^3` a `2^20`:
    /// la traza oculta de la prueba de edad mide `2^(m + 4)`, de `2^7`, con `m = 3`, a
    /// `2^(M_MAX + 4)` (el tope de `zk-ssl-air`), y `2^20` cubre un tope de hasta 16. Las
    /// exenciones, las de un AIR de upstream (1 y 2) y las del envoltorio de ocultacion
    /// (`L/2 + 1`, y `L/2` hasta `2^16`).
    /// Falsador: con el bloque desplazado una fila (`l - n + 1`), falla en el primer caso.
    #[test]
    fn el_bloque_da_lo_que_daba_la_lista() {
        let zs = puntos(8);
        for log_l in 3..=20u32 {
            let l = 1usize << log_l;
            let mitad: &[usize] = if log_l <= 16 { &[l / 2, l / 2 + 1] } else { &[l / 2 + 1] };
            for &n in [1, 2].iter().chain(mitad) {
                let viejo = de_upstream(l, n);
                let nuevo = ConstraintDivisor::<BaseElement>::from_transition(l, n);
                assert!(nuevo.exemptions.is_empty(), "L = {l}, n = {n}: el bloque trae lista");
                assert_eq!(nuevo.num_exemptions(), n, "L = {l}, n = {n}");
                assert_eq!(nuevo.degree(), viejo.degree(), "L = {l}, n = {n}");
                assert_eq!(nuevo.numerator(), viejo.numerator(), "L = {l}, n = {n}");
                for &z in &zs {
                    assert_eq!(
                        nuevo.evaluate_exemptions_at(z),
                        viejo.evaluate_exemptions_at(z),
                        "L = {l}, n = {n}, z = {z}"
                    );
                    assert_eq!(nuevo.evaluate_at(z), viejo.evaluate_at(z), "L = {l}, n = {n}");
                }
                if l <= 256 {
                    let g = BaseElement::get_root_of_unity(log_l);
                    for j in 0..l {
                        let x = g.exp((j as u64).into());
                        let e = nuevo.evaluate_exemptions_at(x);
                        assert_eq!(e, viejo.evaluate_exemptions_at(x), "L = {l}, n = {n}, j = {j}");
                        assert_eq!(e == BaseElement::ZERO, j >= l - n, "L = {l}, n = {n}, j = {j}");
                    }
                }
            }
        }
    }

    /// Construir el divisor de una transicion no depende del numero de exenciones: con
    /// `L = 2^31` y `2^30 + 1` exenciones, la lista de upstream serian 8 GiB y 2^30
    /// exponenciaciones antes de mirar nada; el bloque son tres numeros. Lo que se construye es el
    /// bloque de upstream: su primera exencion es `g^(L - n)`, y la siguiente a la ultima, `1`.
    #[test]
    fn construir_el_divisor_no_depende_de_las_exenciones() {
        let l = 1usize << 31;
        let n = (1usize << 30) + 1;
        let d = ConstraintDivisor::<BaseElement>::from_transition(l, n);
        let b = d.bloque_de_exenciones().expect("una transicion es un bloque");
        let g = BaseElement::get_root_of_unity(31);
        assert_eq!(b.paso, g);
        assert_eq!(b.primera, g.exp(((l - n) as u64).into()));
        assert_eq!(b.primera * g.exp((n as u64).into()), BaseElement::ONE);
        assert_eq!(b.cuantas, n);
        assert_eq!(d.degree(), l - n);
        // sin exenciones, sin bloque; y un divisor con lista no se presenta como bloque
        let sin = ConstraintDivisor::<BaseElement>::from_transition(8, 0);
        assert_eq!(sin.bloque_de_exenciones(), None);
        let lista = ConstraintDivisor::new(vec![(8, BaseElement::ONE)], vec![BaseElement::ONE]);
        assert_eq!(lista.bloque_de_exenciones(), None);
        assert_eq!(lista.num_exemptions(), 1);
    }
}
