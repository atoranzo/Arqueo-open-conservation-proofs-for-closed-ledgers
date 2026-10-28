//! **La 134 del HVZK (§900): M_lin, lo que las filas aleatorias de las columnas reales enmascaran
//! de DEEP, sondeado y medido AIR a AIR.**
//!
//! El §898 midio la condicion R3 de `doc/ocultacion-hvzk.md` con las dos mascaras que no dependen
//! del testigo y no toca la AIR -la columna aleatoria y los aleatorizadores del cociente- y no se
//! cumple en ninguna geometria; el §899 la cumplio en laboratorio con dos columnas (la 133). Pero
//! el fork pone T filas aleatorias detras de CADA columna real, y en una AIR cuyas restricciones
//! toquen una columna solo linealmente y con coeficientes publicos -constantes, periodicos, o los
//! elementos aleatorios del tramo auxiliar, que son del verificador- esas filas son una mascara
//! lineal mas, independiente del testigo (la objecion del §897 que abrio la 134). Este modulo:
//!
//! 1. **Sondea cada columna real** de cada AIR oculta, sin saber su tipo (`Sonda`, que `Oculta<A>`
//!    implementa para toda `A`): con un marco al azar y un desplazamiento δ en E, la composicion
//!    C es afin en la entrada (segunda diferencia nula) y su pendiente no depende del resto del
//!    marco (la misma en dos marcos al azar). Tres veredictos por columna: lineal con coeficiente
//!    publico, no lineal (grado >= 2 en ella) o lineal con pendiente del testigo (la multiplica
//!    otra columna). Schwartz-Zippel sobre E: que una no lineal pase por lineal en tres marcos es
//!    un suceso de probabilidad del orden de 2^-120.
//! 2. **Construye M_lin** de las lineales: lo que queda libre de sus filas aleatorias una vez
//!    condicionadas a los 2q + 6 tocados de L1 -P_ρ = Z_real · V_toc · w, con Z_real nula en las T
//!    filas reales, V_toc en los tocados y sus conjugados, y w ∈ F[x] de grado < T − 2q − 6-, y su
//!    imagen en D por los dos caminos: el directo, cc_i · Q[P_ρ], y el del cociente, C(P_ρ) − C(0)
//!    -lineal en P_ρ, con la pendiente de cada columna precalculada en el dominio de restricciones-
//!    troceado como `segmentar` (paso s = L − m) y cada trozo con su coeficiente de DEEP; con el
//!    nucleo CONJUNTO con los aleatorizadores sobre lo que el verificador ve de cada trozo (las q
//!    posiciones, z y z·g): rango(O∘M) = rango([abiertos | O∘D]) − rango(abiertos), sobre
//!    combinaciones aleatorias de todo el espacio (mas que su rango).
//! 3. **Mide R3 con M = {columna aleatoria, aleatorizadores, M_lin}** contra O|_K, con la misma O,
//!    la misma K y los mismos desafios que `instrumento_r3`; sin M_lin reproduce sus cifras
//!    clavadas, que es como el instrumento se valida; y la inclusion de las mascaras en K se
//!    comprueba por rangos.
//!
//! Lo que NO hace: no cuenta las filas aleatorias del tramo auxiliar (solo `EdadAir` lo tiene, y
//! con T = 64); no cambia la cuenta de L1: condiciona cada columna lineal a sus 2q + 6 tocados,
//! mas de lo que el verificador ve de ella, asi que M_lin es una cota inferior y R3 con M_lin es
//! suficiente, no necesaria (la 135 de `BACKLOG.md`; ⚠️ §905: la cuenta fina esta medida:
//! `Base::Vistos` condiciona a lo visto, q + 4, `sobreyectiva` mide el rango de Λ -lo que C ve de
//! las lineales en S_ab y el verificador no- y `FINA_MEDIDO` clava lo que da en las familias que
//! no caben con la gruesa: ninguna se rescata hoy); con T = 64 no hay M_lin que medir con la
//! cuenta gruesa, T − 2q − 6 < 0, y esas cinco familias siguen en la 131; no mide los desafios de
//! ninguna prueba real; y no cambia la promesa de la casa.

use std::collections::BTreeSet;

use stark_experiment::circuit_audit::{AuditAir, AuditPublicInputs};
use stark_experiment::circuit_burn::{BurnAir, BurnPublicInputs};
use stark_experiment::circuit_claim::{ClaimAir, ClaimPublicInputs};
use stark_experiment::circuit_claim_v2::ClaimAirV2;
use stark_experiment::circuit_credit_climb::{CreditClimbAir, CreditClimbPublicInputs};
use stark_experiment::circuit_frozen_climb::{FrozenClimbAir, FrozenClimbPublicInputs};
use stark_experiment::circuit_mint_climb::{MintClimbAir, MintClimbPublicInputs};
use stark_experiment::circuit_mint_pending_climb::{MintPendingClimbAir, MintPendingClimbPublicInputs};
use stark_experiment::circuit_recovery_climb::{RecoveryClimbAir, RecoveryClimbPublicInputs};
use stark_experiment::circuit_refund::{RefundAir, RefundPublicInputs};
use stark_experiment::circuit_refund_v2::RefundAirV2;
use stark_experiment::circuit_send::{SendAir, SendPublicInputs};
use stark_experiment::circuit_send_v2::SendV2Air;
use stark_experiment::circuit_threshold::{ThresholdAir, ThresholdPublicInputs};
use stark_experiment::circuit_threshold_single_nullifier::{
    NullifierThresholdAir, NullifierThresholdPublicInputs,
};
use stark_experiment::double_entry::{DoubleEntryAir, DoubleEntryPublicInputs};
use stark_experiment::merkle::{MerkleAir, MerklePublicInputs};
use stark_experiment::solvency::{SolvencyAir, SolvencyPublicInputs};
use winter_air::{Air, AirContext, Assertion, Marca, Oculta, TransitionConstraintDegree};
use winter_prover::Ocultacion;
use winterfell::crypto::{hashers::Blake3_256, DefaultRandomCoin, MerkleTree, RandomCoin};
use winterfell::math::fields::f64::BaseElement;
use winterfell::math::{fft, polynom, ExtensionOf, FieldElement, StarkField};
use winterfell::matrix::ColMatrix;
use winterfell::{
    AuxRandElements, BatchingMethod, CompositionPoly, CompositionPolyTrace,
    ConstraintCompositionCoefficients, DefaultConstraintCommitment, DefaultConstraintEvaluator,
    DefaultTraceLde, EvaluationFrame, FieldExtension, PartitionOptions, Proof, ProofOptions,
    Prover, StarkDomain, TraceInfo, TracePolyTable, TraceTable,
};
use zk_ssl_air::banda::{BandaAir, BandaPublicInputs};
use zk_ssl_air::cobro_pendiente::{CobroPendienteAir, CobroPendientePublicInputs};
use zk_ssl_air::pago_en_curso::{PagoEnCursoAir, PagoEnCursoPublicInputs};
use zk_ssl_air::prenda::{PrendaAir, PrendaPublicInputs};
use zk_ssl_air::{EdadAir, EdadPublicInputs};

use crate::instrumento_cota::Geometria;
use crate::instrumento_r3::{
    con_largo, conjugado, de_raices, desafios, mul, rango_de, saca, sobre_f, Mirilla, Moneda, E,
};
use crate::proof_options;

/// Las cuatro palabras de un resumen de la casa.
type Digest4 = [BaseElement; 4];

// ---- la sonda: lo que el instrumento necesita de una AIR oculta, sin su tipo ----

/// Un marco de evaluacion: la fila actual y la siguiente de las columnas REALES de la traza
/// principal (las aleatorias no entran en ninguna restriccion) y las del tramo auxiliar, si lo hay.
#[derive(Clone)]
pub(crate) struct Marco {
    pub actual: Vec<E>,
    pub siguiente: Vec<E>,
    pub aux_actual: Vec<E>,
    pub aux_siguiente: Vec<E>,
}

/// Lo que la sonda y la medida piden a una AIR oculta: sus anchuras, su geometria de
/// restricciones, sus columnas periodicas y su composicion en un punto con un marco.
pub(crate) trait Sonda {
    /// L = 2T, la longitud de la traza oculta.
    fn largo(&self) -> usize;
    /// Las columnas reales de la traza principal, sin las aleatorias.
    fn reales(&self) -> usize;
    /// Las columnas del tramo auxiliar y sus elementos aleatorios.
    fn auxiliares(&self) -> usize;
    fn aleatorios_aux(&self) -> usize;
    /// El factor del dominio de restricciones y los trozos del cociente que dice el contexto.
    fn ce(&self) -> usize;
    fn trozos(&self) -> usize;
    /// Las restricciones de transicion (principales y auxiliares) y las aserciones.
    fn transiciones(&self) -> usize;
    fn aserciones(&self) -> usize;
    /// Las columnas periodicas en x, como las lee el verificador: p(x^{L/c}).
    fn periodicas(&self, x: E) -> Vec<E>;
    /// La composicion C en cada punto `xs[k]` con el marco `marcos[k]` y las periodicas `per[k]`:
    /// la combinacion de las transiciones dividida por su divisor mas cada grupo de aserciones
    /// dividido por el suyo, como la evalua el probador.
    fn composicion(
        &self,
        marcos: &[&Marco],
        xs: &[E],
        per: &[Vec<E>],
        coefs: &ConstraintCompositionCoefficients<E>,
        aux: &AuxRandElements<E>,
    ) -> Vec<E>;
}

impl<A: Air<BaseField = BaseElement>> Sonda for Oculta<A> {
    fn largo(&self) -> usize {
        self.trace_length()
    }

    fn reales(&self) -> usize {
        self.trace_info().main_trace_width() - Marca::columnas_de(self.trace_info().meta())
    }

    fn auxiliares(&self) -> usize {
        self.trace_info().aux_segment_width()
    }

    fn aleatorios_aux(&self) -> usize {
        self.trace_info().get_num_aux_segment_rand_elements()
    }

    fn ce(&self) -> usize {
        self.ce_blowup_factor()
    }

    fn trozos(&self) -> usize {
        self.context().num_constraint_composition_columns()
    }

    fn transiciones(&self) -> usize {
        self.context().num_transition_constraints()
    }

    fn aserciones(&self) -> usize {
        self.context().num_assertions()
    }

    fn periodicas(&self, x: E) -> Vec<E> {
        let l = self.trace_length() as u64;
        self.get_periodic_column_polys()
            .iter()
            .map(|p| polynom::eval(p, x.exp(l / p.len() as u64)))
            .collect()
    }

    fn composicion(
        &self,
        marcos: &[&Marco],
        xs: &[E],
        per: &[Vec<E>],
        coefs: &ConstraintCompositionCoefficients<E>,
        aux: &AuxRandElements<E>,
    ) -> Vec<E> {
        assert!(marcos.len() == xs.len() && xs.len() == per.len());
        let tc = self.get_transition_constraints(&coefs.transition);
        let con_aux = self.auxiliares() > 0;
        let bc = self.get_boundary_constraints(con_aux.then_some(aux), &coefs.boundary);
        let mut principales = vec![E::ZERO; tc.num_main_constraints()];
        let mut auxiliares = vec![E::ZERO; tc.num_aux_constraints()];
        marcos
            .iter()
            .zip(xs)
            .zip(per)
            .map(|((m, &x), p)| {
                let marco = EvaluationFrame::from_rows(m.actual.clone(), m.siguiente.clone());
                principales.iter_mut().for_each(|e| *e = E::ZERO);
                self.evaluate_transition(&marco, p, &mut principales);
                if con_aux {
                    let marco_aux =
                        EvaluationFrame::from_rows(m.aux_actual.clone(), m.aux_siguiente.clone());
                    auxiliares.iter_mut().for_each(|e| *e = E::ZERO);
                    self.evaluate_aux_transition(&marco, &marco_aux, p, aux, &mut auxiliares);
                }
                let mut r = tc.combine_evaluations::<E>(&principales, &auxiliares, x);
                for g in bc.main_constraints() {
                    r += g.evaluate_at(&m.actual, x);
                }
                for g in bc.aux_constraints() {
                    r += g.evaluate_at(&m.aux_actual, x);
                }
                r
            })
            .collect()
    }
}

fn coeficientes(s: &dyn Sonda, moneda: &mut Moneda) -> ConstraintCompositionCoefficients<E> {
    ConstraintCompositionCoefficients {
        transition: (0..s.transiciones()).map(|_| saca(moneda)).collect(),
        boundary: (0..s.aserciones()).map(|_| saca(moneda)).collect(),
    }
}

fn aleatorios(s: &dyn Sonda, moneda: &mut Moneda) -> AuxRandElements<E> {
    AuxRandElements::new((0..s.aleatorios_aux()).map(|_| saca(moneda)).collect())
}

fn marco_al_azar(s: &dyn Sonda, moneda: &mut Moneda) -> Marco {
    let mut fila = |n: usize| (0..n).map(|_| saca::<E>(moneda)).collect::<Vec<E>>();
    let actual = fila(s.reales());
    let siguiente = fila(s.reales());
    let aux_actual = fila(s.auxiliares());
    let aux_siguiente = fila(s.auxiliares());
    Marco { actual, siguiente, aux_actual, aux_siguiente }
}

fn marco_cero(s: &dyn Sonda) -> Marco {
    Marco {
        actual: vec![E::ZERO; s.reales()],
        siguiente: vec![E::ZERO; s.reales()],
        aux_actual: vec![E::ZERO; s.auxiliares()],
        aux_siguiente: vec![E::ZERO; s.auxiliares()],
    }
}

// ---- el censo: como entra cada columna real en la composicion ----

/// Como entra una columna real en C.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Entrada {
    /// C es afin en la columna y la pendiente no depende del marco: coeficiente publico. Sus
    /// filas aleatorias son una mascara lineal independiente del testigo.
    Lineal,
    /// C tiene grado >= 2 en la columna.
    NoLineal,
    /// C es afin en la columna pero la pendiente cambia con el marco: la multiplica otra columna,
    /// y sus filas aleatorias entran en D a traves del testigo.
    Testigo,
}

/// El censo de una AIR: la entrada de cada columna real, en su orden.
#[derive(Debug, Clone)]
pub(crate) struct Censo {
    pub entradas: Vec<Entrada>,
}

impl Censo {
    /// Las columnas lineales con coeficiente publico, en su orden.
    pub fn lineales(&self) -> Vec<usize> {
        self.entradas
            .iter()
            .enumerate()
            .filter(|(_, e)| **e == Entrada::Lineal)
            .map(|(i, _)| i)
            .collect()
    }

    /// (lineales, con pendiente del testigo, no lineales).
    pub fn cuenta(&self) -> (usize, usize, usize) {
        let n = |e: Entrada| self.entradas.iter().filter(|x| **x == e).count();
        (n(Entrada::Lineal), n(Entrada::Testigo), n(Entrada::NoLineal))
    }
}

/// Sondea cada columna real: en tres puntos x de E al azar, con dos marcos al azar f y f' y un
/// desplazamiento δ, para la fila actual y para la siguiente: afin si
/// C(f + 2δe) − C(f) = 2·(C(f + δe) − C(f)), y publica si la pendiente en f' es la de f.
pub(crate) fn sondea(s: &dyn Sonda, semilla: u64) -> Censo {
    let mut moneda = Moneda::new(&[BaseElement::new(semilla ^ 0x50da)]);
    let coefs = coeficientes(s, &mut moneda);
    let aux = aleatorios(s, &mut moneda);
    let mut entradas = Vec::with_capacity(s.reales());
    for i in 0..s.reales() {
        let mut veredicto = Entrada::Lineal;
        for _ in 0..3 {
            let x: E = saca(&mut moneda);
            let per = s.periodicas(x);
            let f = marco_al_azar(s, &mut moneda);
            let f2 = marco_al_azar(s, &mut moneda);
            let delta: E = saca(&mut moneda);
            for lado in 0..2 {
                let mas = |m: &Marco, k: E| {
                    let mut m = m.clone();
                    if lado == 0 {
                        m.actual[i] += k;
                    } else {
                        m.siguiente[i] += k;
                    }
                    m
                };
                let marcos = [f.clone(), mas(&f, delta), mas(&f, delta + delta), f2.clone(), mas(&f2, delta)];
                let refs: Vec<&Marco> = marcos.iter().collect();
                let v = s.composicion(&refs, &[x; 5], &vec![per.clone(); 5], &coefs, &aux);
                let c1 = v[1] - v[0];
                if v[2] - v[0] != c1 + c1 {
                    veredicto = Entrada::NoLineal;
                } else if v[4] - v[3] != c1 && veredicto == Entrada::Lineal {
                    veredicto = Entrada::Testigo;
                }
            }
        }
        entradas.push(veredicto);
    }
    Censo { entradas }
}

// ---- la medida: R3 con M_lin ----

/// Lo que la medida dice de una AIR en una geometria.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MedidaLin {
    /// Las columnas lineales que entran en M_lin.
    pub lineales: usize,
    /// Las dimensiones libres sobre F por columna lineal, medidas: T menos el grado de V_toc.
    pub libres: usize,
    /// rango sobre F de O restringida a K: lo que FRI abre sin determinar.
    pub rango_k: usize,
    /// rango sobre F de O∘{columna aleatoria, aleatorizadores}: lo que midio el §898.
    pub rango_sin_lineales: usize,
    /// rango sobre F de O∘{M_lin, aleatorizadores}: lo que ponen las filas de las lineales.
    pub rango_lineales: usize,
    /// rango sobre F de O∘{columna, aleatorizadores, M_lin}, las tres juntas.
    pub rango_mascaras: usize,
}

impl MedidaLin {
    /// R3 con M_lin: las tres mascaras cubren todo lo que FRI abre y las aperturas no determinan.
    pub fn cumple(&self) -> bool {
        self.rango_mascaras == self.rango_k
    }
}

/// Que se mide ademas de las tres mascaras juntas; cada variante cuesta otra eliminacion.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Variantes {
    pub sin_lineales: bool,
    pub lineales: bool,
    /// La inclusion de las mascaras en K, por rangos: una eliminacion mas.
    pub inclusion: bool,
}

pub(crate) const TODAS: Variantes = Variantes { sin_lineales: true, lineales: true, inclusion: true };
/// Las del test largo: sin la variante del §898, que ya esta clavada por geometria.
pub(crate) const LARGO: Variantes = Variantes { sin_lineales: false, lineales: true, inclusion: true };

/// Las dimensiones libres por columna lineal que deja L1 en una geometria: T − (2q + 6), o cero
/// si no caben (T = 64).
pub(crate) fn libres_de(g: &Geometria) -> usize {
    g.t.saturating_sub(2 * g.q + 6)
}

/// §905 (la 135): a que se condiciona cada columna lineal antes de contar lo que le queda libre.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Base {
    /// La cuenta de L1: los 2q + 6 tocados (las posiciones, sus siguientes, z, z·g y g²·z, con
    /// sus conjugados). Lo del §900.
    Tocados,
    /// La cuenta fina de L1': lo VISTO de la columna, q + 4 (las posiciones, z y z·g, con sus
    /// conjugados); lo que C ve de ella en S_ab lo descuenta el nucleo conjunto, y que C(S_ab) sea
    /// uniforme lo mide `sobreyectiva`.
    Vistos,
}

/// Las dimensiones libres por columna lineal con la base fina: T − (q + 4), o cero.
pub(crate) fn libres_finos_de(g: &Geometria) -> usize {
    g.t.saturating_sub(g.q + 4)
}

/// (h(x) − h(a)) / (x − a), por division sintetica; el ultimo coeficiente queda a cero.
fn cociente_en(h: &[E], a: E) -> Vec<E> {
    let n = h.len();
    let mut q = vec![E::ZERO; n];
    let mut acc = E::ZERO;
    for i in (1..n).rev() {
        acc = acc * a + h[i];
        q[i - 1] = acc;
    }
    q
}

/// Q[h] = (h(x) − h(z))/(x − z) + (h(x) − h(z·g))/(x − z·g): lo que DEEP hace con cada polinomio.
fn q_deep(h: &[E], z: E, zg: E) -> Vec<E> {
    let a = cociente_en(h, z);
    let b = cociente_en(h, zg);
    a.iter().zip(&b).map(|(x, y)| *x + *y).collect()
}

fn clave(e: E) -> (u64, u64) {
    (e.base_element(0).as_int(), e.base_element(1).as_int())
}

/// Mide R3 con las tres mascaras para la AIR `s` en la geometria `g` con las columnas lineales
/// `lineales`, con los desafios de `instrumento_r3::desafios(g, semilla)` y los demas -los
/// coeficientes de DEEP de las columnas reales, los de las restricciones y los elementos
/// aleatorios del tramo auxiliar- de otra moneda sembrada con la misma semilla.
pub(crate) fn mide(
    s: &dyn Sonda,
    g: &Geometria,
    lineales: &[usize],
    semilla: u64,
    var: Variantes,
) -> MedidaLin {
    mide_con_base(s, g, lineales, semilla, var, Base::Tocados)
}

/// `mide` con la base de condicionamiento que se le diga (§905, la 135).
pub(crate) fn mide_con_base(
    s: &dyn Sonda,
    g: &Geometria,
    lineales: &[usize],
    semilla: u64,
    var: Variantes,
    base: Base,
) -> MedidaLin {
    assert_eq!(s.largo(), 2 * g.t, "la traza oculta no es la de la geometria");
    assert_eq!(s.trozos(), g.trozos, "los trozos de la prueba no son los del contexto");
    assert!(lineales.iter().all(|&i| i < s.reales()), "una columna lineal fuera de la traza");
    let d = desafios(g, semilla);
    let mirilla = Mirilla::nueva(g, &d);
    let largo = mirilla.largo;
    let lde = mirilla.lde;
    let omega = BaseElement::get_root_of_unity(lde.ilog2());
    let g_traza = BaseElement::get_root_of_unity(largo.ilog2());
    let x_pos: Vec<E> = d
        .posiciones
        .iter()
        .map(|&p| E::from(BaseElement::GENERATOR * omega.exp(p as u64)))
        .collect();
    let (z, zg) = (d.z, d.z * E::from(g_traza));
    let x0 = (z + zg) / E::from(BaseElement::new(2));
    let ell = vec![-(z + zg), E::from(BaseElement::new(2))];
    let v_pos = de_raices(&x_pos);
    let s_paso = largo - g.m;
    let n_trozos = g.trozos;
    let mut moneda = Moneda::new(&[BaseElement::new(semilla ^ 0x1340)]);
    let cc_reales: Vec<E> = (0..s.reales()).map(|_| saca(&mut moneda)).collect();
    let coefs = coeficientes(s, &mut moneda);
    let aux = aleatorios(s, &mut moneda);

    // K: los D nulos en las posiciones y en x0; su rango sobre F y sus filas, para la inclusion
    let n_k = mirilla.filas + 8;
    let base_k = mul(&v_pos, &[-x0, E::ONE]);
    let libres_k = largo - base_k.len();
    let filas_k: Vec<Vec<E>> = (0..n_k)
        .map(|_| {
            let u: Vec<E> = (0..libres_k).map(|_| saca(&mut moneda)).collect();
            mirilla.observa(&con_largo(mul(&base_k, &u), largo))
        })
        .collect();
    let rango_k = 2 * rango_de(filas_k.clone());

    // la columna aleatoria: Φ·u con Φ = cc_R·ℓ·V·(x − z̄)(x − z̄g), u ∈ F[x] de grado < L − q − 4
    let base_f = mul(&mul(&ell, &v_pos), &de_raices(&[conjugado(z), conjugado(zg)]));
    let phi: Vec<E> = base_f.iter().map(|x| *x * d.cc_columna).collect();
    let libres_r = largo - g.q - 4;

    // M_lin: la base de lo libre, Z_real·V_toc, un polinomio de F[x]
    let mut vistos = BTreeSet::new();
    let mut tocados = Vec::new();
    let g_e = E::from(g_traza);
    let fina = base == Base::Vistos;
    for &x in &x_pos {
        for y in if fina { vec![x] } else { vec![x, x * g_e] } {
            if vistos.insert(clave(y)) {
                tocados.push(y);
            }
        }
    }
    for y in if fina { vec![z, zg] } else { vec![z, zg, zg * g_e] } {
        for w in [y, conjugado(y)] {
            if vistos.insert(clave(w)) {
                tocados.push(w);
            }
        }
    }
    let v_toc = de_raices(&tocados);
    assert!(v_toc.iter().all(|c| c.base_element(1) == BaseElement::ZERO), "V_toc no vive en F[x]");
    let z_real = de_raices(&(0..g.t).map(|j| E::from(g_traza.exp(j as u64))).collect::<Vec<E>>());
    let base_lin = mul(&z_real, &v_toc);
    let libres = (largo + 1).saturating_sub(base_lin.len());
    let con_lin = libres > 0 && !lineales.is_empty();
    let ce = s.ce();
    let ce_tam = ce * largo;
    let twiddles_ce = fft::get_twiddles::<BaseElement>(ce_tam);
    let inv_twiddles_ce = fft::get_inv_twiddles::<BaseElement>(ce_tam);
    let omega_ce = BaseElement::get_root_of_unity(ce_tam.ilog2());
    let xs_ce: Vec<E> = (0..ce_tam)
        .map(|k| E::from(BaseElement::GENERATOR * omega_ce.exp(k as u64)))
        .collect();
    // la base en el dominio de restricciones, y la pendiente de C en cada columna lineal, en cada
    // punto: C(e) − C(0), para la fila actual y para la siguiente
    let (base_lin_ce, pendientes): (Vec<BaseElement>, Vec<(Vec<E>, Vec<E>)>) = if con_lin {
        let mut p: Vec<BaseElement> = base_lin.iter().map(|c| c.base_element(0)).collect();
        p.resize(ce_tam, BaseElement::ZERO);
        let base_ce = fft::evaluate_poly_with_offset(&p, &twiddles_ce, BaseElement::GENERATOR, 1);
        let per_ce: Vec<Vec<E>> = xs_ce.iter().map(|&x| s.periodicas(x)).collect();
        let cero = marco_cero(s);
        let c0 = s.composicion(&vec![&cero; ce_tam], &xs_ce, &per_ce, &coefs, &aux);
        let pend = lineales
            .iter()
            .map(|&i| {
                let mut ea = cero.clone();
                ea.actual[i] = E::ONE;
                let mut es = cero.clone();
                es.siguiente[i] = E::ONE;
                let ca = s.composicion(&vec![&ea; ce_tam], &xs_ce, &per_ce, &coefs, &aux);
                let cs = s.composicion(&vec![&es; ce_tam], &xs_ce, &per_ce, &coefs, &aux);
                (
                    ca.iter().zip(&c0).map(|(a, b)| *a - *b).collect::<Vec<E>>(),
                    cs.iter().zip(&c0).map(|(a, b)| *a - *b).collect::<Vec<E>>(),
                )
            })
            .collect();
        (base_ce, pend)
    } else {
        (Vec::new(), Vec::new())
    };

    // una combinacion aleatoria del espacio de parametros, observada: [abiertos | O∘D]
    let abiertos_y: Vec<E> = x_pos.iter().copied().chain([z, zg]).collect();
    let n_abiertos = 2 * n_trozos * abiertos_y.len();
    let muestra = |moneda: &mut Moneda, con_columna: bool, con_lineales: bool| -> Vec<BaseElement> {
        let mut c_lin = vec![E::ZERO; ce_tam];
        let mut directa = vec![E::ZERO; ce_tam];
        if con_lineales && con_lin {
            for (idx, &i) in lineales.iter().enumerate() {
                let mut w: Vec<BaseElement> = (0..libres).map(|_| saca(moneda)).collect();
                w.resize(ce_tam, BaseElement::ZERO);
                let ev = fft::evaluate_poly_with_offset(&w, &twiddles_ce, BaseElement::GENERATOR, 1);
                let (pa, ps) = &pendientes[idx];
                for k in 0..ce_tam {
                    let p_k = base_lin_ce[k] * ev[k];
                    let kg = (k + ce) % ce_tam;
                    let p_kg = base_lin_ce[kg] * ev[kg];
                    c_lin[k] += pa[k].mul_base(p_k) + ps[k].mul_base(p_kg);
                    directa[k] += cc_reales[i].mul_base(p_k);
                }
            }
            fft::interpolate_poly_with_offset(&mut c_lin, &inv_twiddles_ce, BaseElement::GENERATOR);
            fft::interpolate_poly_with_offset(&mut directa, &inv_twiddles_ce, BaseElement::GENERATOR);
            assert!(
                c_lin[n_trozos * s_paso..].iter().all(|c| *c == E::ZERO),
                "M_lin: C(P_ρ) no cabe en {n_trozos} trozos de paso {s_paso}"
            );
            assert!(directa[largo..].iter().all(|c| *c == E::ZERO), "M_lin: Σ cc_i·P_ρ,i no es de grado < L");
        }
        // los trozos de segmentar, h_k = C[k·s..(k+1)·s], y los aleatorizadores: +x^s·r_k en el
        // trozo k y −r_k en el k + 1
        let mut trozos: Vec<Vec<E>> = (0..n_trozos)
            .map(|k| {
                let mut t = vec![E::ZERO; largo];
                t[..s_paso].copy_from_slice(&c_lin[k * s_paso..(k + 1) * s_paso]);
                t
            })
            .collect();
        for k in 0..n_trozos - 1 {
            for j in 0..g.m {
                let v: E = saca(moneda);
                trozos[k][s_paso + j] += v;
                trozos[k + 1][j] -= v;
            }
        }
        let mut fila = Vec::with_capacity(n_abiertos + 2 * mirilla.filas);
        for t in &trozos {
            for &y in &abiertos_y {
                let v = polynom::eval(t, y);
                fila.push(v.base_element(0));
                fila.push(v.base_element(1));
            }
        }
        // D: Q de la suma de todo lo que lleva coeficiente de DEEP, mas la columna aleatoria
        let mut suma: Vec<E> = directa[..largo].to_vec();
        for (k, t) in trozos.iter().enumerate() {
            for (a, b) in suma.iter_mut().zip(t) {
                *a += d.cc_trozos[k] * *b;
            }
        }
        let mut dd = q_deep(&suma, z, zg);
        if con_columna {
            let u: Vec<E> = (0..libres_r).map(|_| E::from(saca::<BaseElement>(moneda))).collect();
            for (a, b) in dd.iter_mut().zip(mul(&phi, &u)) {
                *a += b;
            }
        }
        fila.extend(sobre_f(&mirilla.observa(&dd)));
        fila
    };
    let n_muestras = rango_k + n_abiertos + 8;
    let rangos = |filas: &[Vec<BaseElement>]| -> (usize, usize) {
        let abiertos: Vec<Vec<BaseElement>> = filas.iter().map(|f| f[..n_abiertos].to_vec()).collect();
        (rango_de(abiertos), rango_de(filas.to_vec()))
    };

    // las tres juntas
    let filas: Vec<Vec<BaseElement>> = (0..n_muestras).map(|_| muestra(&mut moneda, true, true)).collect();
    let (ra, rj) = rangos(&filas);
    let rango_mascaras = rj - ra;
    assert!(rango_mascaras <= rango_k, "las mascaras salen de K: {rango_mascaras} > {rango_k}");
    if var.inclusion {
        let mut todas = filas;
        todas.extend(filas_k.iter().map(|f| {
            let mut r = vec![BaseElement::ZERO; n_abiertos];
            r.extend(sobre_f(f));
            r
        }));
        let r = rango_de(todas) - ra;
        assert_eq!(r, rango_k, "las mascaras salen de K: con K dan {r}, y K es {rango_k}");
    }
    let rango_sin_lineales = if var.sin_lineales {
        let filas: Vec<Vec<BaseElement>> = (0..n_muestras).map(|_| muestra(&mut moneda, true, false)).collect();
        let (a, j) = rangos(&filas);
        j - a
    } else {
        0
    };
    let rango_lineales = if var.lineales {
        let filas: Vec<Vec<BaseElement>> = (0..n_muestras).map(|_| muestra(&mut moneda, false, true)).collect();
        let (a, j) = rangos(&filas);
        j - a
    } else {
        0
    };
    MedidaLin {
        lineales: if con_lin { lineales.len() } else { 0 },
        libres,
        rango_k,
        rango_sin_lineales,
        rango_lineales,
        rango_mascaras,
    }
}

/// §905 (la 135), L1': el rango sobre F de Λ, la aplicacion de lo libre de las columnas lineales
/// -condicionadas a lo visto, la base fina- en lo que C ve de ellas y el verificador no:
/// Σ_i σ_i^next(x_j)·P_ρ,i(g·x_j) en las q posiciones y Σ_i σ_i^next(z·g)·P_ρ,i(g²·z); y la
/// dimension del blanco, 2(q + 1) sobre F. Λ sobreyectiva es lo que L1' pide para que C(S_ab) sea
/// uniforme e independiente del testigo. Con los mismos desafios que `mide`.
pub(crate) fn sobreyectiva(s: &dyn Sonda, g: &Geometria, lineales: &[usize], semilla: u64) -> (usize, usize) {
    let d = desafios(g, semilla);
    let largo = 2 * g.t;
    let lde = largo * proof_options().blowup_factor();
    let omega = BaseElement::get_root_of_unity(lde.ilog2());
    let g_traza = BaseElement::get_root_of_unity(largo.ilog2());
    let g_e = E::from(g_traza);
    let x_pos: Vec<E> = d
        .posiciones
        .iter()
        .map(|&p| E::from(BaseElement::GENERATOR * omega.exp(p as u64)))
        .collect();
    let (z, zg) = (d.z, d.z * g_e);
    let blanco = 2 * (g.q + 1);
    if lineales.is_empty() {
        return (0, blanco);
    }
    let mut moneda = Moneda::new(&[BaseElement::new(semilla ^ 0x1350)]);
    let coefs = coeficientes(s, &mut moneda);
    let aux = aleatorios(s, &mut moneda);
    // la base fina: Z_real·V_ab, con V_ab nula en S_ab y sus conjugados
    let mut vistos = BTreeSet::new();
    let mut puntos = Vec::new();
    for &x in &x_pos {
        if vistos.insert(clave(x)) {
            puntos.push(x);
        }
    }
    for y in [z, zg] {
        for w in [y, conjugado(y)] {
            if vistos.insert(clave(w)) {
                puntos.push(w);
            }
        }
    }
    let v_ab = de_raices(&puntos);
    let z_real = de_raices(&(0..g.t).map(|j| E::from(g_traza.exp(j as u64))).collect::<Vec<E>>());
    let base_lin = mul(&z_real, &v_ab);
    let libres = (largo + 1).saturating_sub(base_lin.len());
    if libres == 0 {
        return (0, blanco);
    }
    // los puntos donde C lee la fila siguiente: g·x_j y g²·z; las pendientes de cada lineal ahi
    let ys: Vec<E> = x_pos.iter().copied().chain([zg]).collect();
    let per: Vec<Vec<E>> = ys.iter().map(|&y| s.periodicas(y)).collect();
    let cero = marco_cero(s);
    let c0 = s.composicion(&vec![&cero; ys.len()], &ys, &per, &coefs, &aux);
    let sigmas: Vec<Vec<E>> = lineales
        .iter()
        .map(|&i| {
            let mut es = cero.clone();
            es.siguiente[i] = E::ONE;
            let cs = s.composicion(&vec![&es; ys.len()], &ys, &per, &coefs, &aux);
            cs.iter().zip(&c0).map(|(a, b)| *a - *b).collect()
        })
        .collect();
    let siguientes: Vec<E> = ys.iter().map(|&y| y * g_e).collect();
    let base_en: Vec<E> = siguientes.iter().map(|&y| polynom::eval(&base_lin, y)).collect();
    // combinaciones aleatorias de lo libre de todas las lineales, mas que el blanco
    let n = blanco + 8;
    let filas: Vec<Vec<BaseElement>> = (0..n)
        .map(|_| {
            let mut v = vec![E::ZERO; ys.len()];
            for sig in &sigmas {
                let w: Vec<E> = (0..libres).map(|_| E::from(saca::<BaseElement>(&mut moneda))).collect();
                for (k, &y) in siguientes.iter().enumerate() {
                    v[k] += sig[k] * base_en[k] * polynom::eval(&w, y);
                }
            }
            sobre_f(&v)
        })
        .collect();
    (rango_de(filas), blanco)
}

/// Una linea por medida.
pub(crate) fn linea(nombre: &str, g: &Geometria, m: &MedidaLin) -> String {
    format!(
        "[134] {nombre}: q {} · T {} · m {} · {} trozos · {} columnas lineales con {} libres sobre F cada \
         una · sin determinar (O sobre K): {} · mascaras: columna y aleatorizadores {} · lineales y \
         aleatorizadores {} · las tres {} · R3 con M_lin {}",
        g.q,
        g.t,
        g.m,
        g.trozos,
        m.lineales,
        m.libres,
        m.rango_k,
        m.rango_sin_lineales,
        m.rango_lineales,
        m.rango_mascaras,
        if m.cumple() { "SE CUMPLE" } else { "NO se cumple" }
    )
}

// ---- las 23 AIR con fila, sin su tipo ----

/// La AIR oculta de una familia, por su nombre, con la traza y las opciones de una de sus pruebas
/// y unas entradas publicas vacias: ninguna de las 22 las mira al construirse mas que para
/// guardarlas; `EdadAir` si (el numero de hojas y de aserciones), y va con las de una prueba real
/// del instrumento de edad. `None` si el nombre no es de las 23.
pub(crate) fn sonda_de(air: &str, info: &TraceInfo, opciones: &ProofOptions) -> Option<Box<dyn Sonda>> {
    let d: Digest4 = [BaseElement::ZERO; 4];
    let b = BaseElement::ZERO;
    let i = info.clone();
    let o = opciones.clone();
    let send = || SendPublicInputs {
        root_old: d,
        root_new: d,
        frozen_root: d,
        pending_root_old: d,
        pending_root_new: d,
        amount: b,
        regulatory_limit: b,
        supply_old: b,
        supply_new: b,
    };
    let claim = || ClaimPublicInputs {
        root_old: d,
        root_new: d,
        frozen_root: d,
        pending_root_old: d,
        pending_root_new: d,
        amount: b,
        supply_old: b,
        supply_new: b,
    };
    let refund = || RefundPublicInputs { commitment: d, amount: b };
    Some(match air {
        "SendAir" => Box::new(Oculta::<SendAir>::new(i, send(), o)),
        "SendV2Air" => Box::new(Oculta::<SendV2Air>::new(i, send(), o)),
        "ClaimAir" => Box::new(Oculta::<ClaimAir>::new(i, claim(), o)),
        "ClaimAirV2" => Box::new(Oculta::<ClaimAirV2>::new(i, claim(), o)),
        "MintPendingClimbAir" => Box::new(Oculta::<MintPendingClimbAir>::new(
            i,
            MintPendingClimbPublicInputs {
                supply_old: b,
                supply_new: b,
                max_supply: b,
                amount: b,
                pending_root_old: d,
                pending_root_new: d,
            },
            o,
        )),
        "NullifierThresholdAir" => Box::new(Oculta::<NullifierThresholdAir>::new(
            i,
            NullifierThresholdPublicInputs {
                identity_domain: b,
                custodian_set_root: d,
                nullifier: d,
                operation: d,
            },
            o,
        )),
        "ThresholdAir" => {
            Box::new(Oculta::<ThresholdAir>::new(i, ThresholdPublicInputs { custodian_set_root: d }, o))
        },
        "AuditAir" => Box::new(Oculta::<AuditAir>::new(
            i,
            AuditPublicInputs { root: d, public_id: d, lower: b, upper: b },
            o,
        )),
        "BurnAir" => Box::new(Oculta::<BurnAir>::new(
            i,
            BurnPublicInputs {
                root_old: d,
                root_new: d,
                frozen_root: d,
                amount: b,
                supply_old: b,
                supply_new: b,
            },
            o,
        )),
        "SolvencyAir" => {
            Box::new(Oculta::<SolvencyAir>::new(i, SolvencyPublicInputs { regulatory_limit: b }, o))
        },
        "DoubleEntryAir" => Box::new(Oculta::<DoubleEntryAir>::new(
            i,
            DoubleEntryPublicInputs { root_old: d, root_new: d, regulatory_limit: b, nullifier: d },
            o,
        )),
        "MerkleAir" => Box::new(Oculta::<MerkleAir>::new(i, MerklePublicInputs { root: d }, o)),
        "RefundAir" => Box::new(Oculta::<RefundAir>::new(i, refund(), o)),
        "RefundAirV2" => Box::new(Oculta::<RefundAirV2>::new(i, refund(), o)),
        "CreditClimbAir" => Box::new(Oculta::<CreditClimbAir>::new(
            i,
            CreditClimbPublicInputs { root_old: d, root_new: d, amount: b },
            o,
        )),
        "MintClimbAir" => Box::new(Oculta::<MintClimbAir>::new(
            i,
            MintClimbPublicInputs {
                root_old: d,
                root_new: d,
                amount: b,
                supply_old: b,
                supply_new: b,
                max_supply: b,
            },
            o,
        )),
        "FrozenClimbAir" => Box::new(Oculta::<FrozenClimbAir>::new(
            i,
            FrozenClimbPublicInputs { root_a: d, root_b: d },
            o,
        )),
        "RecoveryClimbAir" => Box::new(Oculta::<RecoveryClimbAir>::new(
            i,
            RecoveryClimbPublicInputs {
                root_old: d,
                root_new: d,
                recovery_count_old: b,
                recovery_count_new: b,
            },
            o,
        )),
        "PrendaAir" => Box::new(Oculta::<PrendaAir>::new(
            i,
            PrendaPublicInputs { pending_root: d, receptor: d, marca: d },
            o,
        )),
        "PagoEnCursoAir" => Box::new(Oculta::<PagoEnCursoAir>::new(
            i,
            PagoEnCursoPublicInputs {
                pending_root: d,
                pmeta_root: d,
                receptor: d,
                importe: b,
                t: b,
                nacido: b,
            },
            o,
        )),
        "CobroPendienteAir" => Box::new(Oculta::<CobroPendienteAir>::new(
            i,
            CobroPendientePublicInputs {
                pending_root: d,
                pmeta_root: d,
                receptor: d,
                nacido: b,
                inferior: b,
                superior: b,
            },
            o,
        )),
        "BandaAir" => Box::new(Oculta::<BandaAir>::new(
            i,
            BandaPublicInputs { root: d, public_id: d, lower: b, upper: b },
            o,
        )),
        "EdadAir" => {
            // la edad mira sus entradas al construirse: el numero de hojas, 2^m, y las aserciones,
            // que dependen de n (en las filas 8·(n − 1) y 8·n) y de k; las de la familia son las de
            // ocho hojas (T = 64, 128 filas ocultas) y el enunciado de todos los emisores, que es
            // el que prueban sus dos filas del testigo de D-B
            assert_eq!(info.length(), 128, "la edad de la familia ya no es la de ocho hojas: cambia m y n aqui");
            let pi = EdadPublicInputs {
                subraiz_pend: d,
                subraiz_meta: d,
                m: 3,
                n: 5,
                seq: 100,
                t: 70,
                emisor: 0,
                todos: true,
                k: 2,
            };
            Box::new(Oculta::<EdadAir>::new(i, pi, o))
        },
        _ => return None,
    })
}

// ---- lo declarado y lo clavado ----

/// Lo que la 134 dice de una familia.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Veredicto {
    /// T = 64: T − 2q − 6 < 0, no hay filas libres que medir; la familia sigue en la 131 y la 133
    /// no la decide sola.
    NoCabe,
    /// Ninguna columna real entra linealmente con coeficiente publico: M_lin = 0 y R3 es la del
    /// §898, que no se cumple. Necesita la 133.
    SinLineales,
    /// R3 se cumple con M_lin: no necesita la 133 para R3.
    Cumple,
    /// Hay M_lin y R3 no se cumple con ella: necesita la 133.
    NoCumple,
}

/// El censo de las 23 familias, declarado y sellado: por AIR, T, las columnas reales, cuantas
/// entran linealmente con coeficiente publico, cuantas con pendiente del testigo, cuantas no
/// linealmente, y el veredicto. Lo que mide la sonda en cada canon tiene que ser esto; un cambio
/// en cualquier cifra es rojo.
pub(crate) const CENSO_134: &[(&str, usize, usize, usize, usize, usize, Veredicto)] = &[
    // AIR, T, reales, lineales, testigo, no lineales, veredicto
    ("SendAir", 1024, 58, 30, 0, 28, Veredicto::Cumple),
    ("SendV2Air", 1024, 62, 34, 0, 28, Veredicto::Cumple),
    ("ClaimAir", 1024, 57, 29, 0, 28, Veredicto::Cumple),
    ("ClaimAirV2", 1024, 61, 33, 0, 28, Veredicto::Cumple),
    ("PrendaAir", 512, 42, 17, 0, 25, Veredicto::Cumple),
    ("MintPendingClimbAir", 512, 39, 13, 0, 26, Veredicto::Cumple),
    ("NullifierThresholdAir", 64, 21, 8, 0, 13, Veredicto::NoCabe),
    ("ThresholdAir", 64, 34, 7, 0, 27, Veredicto::NoCabe),
    ("AuditAir", 512, 31, 17, 0, 14, Veredicto::Cumple),
    ("BurnAir", 1024, 48, 21, 0, 27, Veredicto::Cumple),
    ("SolvencyAir", 256, 5, 4, 0, 1, Veredicto::Cumple),
    ("DoubleEntryAir", 1024, 41, 15, 0, 26, Veredicto::Cumple),
    ("BandaAir", 512, 27, 13, 0, 14, Veredicto::Cumple),
    ("EdadAir", 64, 83, 4, 7, 72, Veredicto::NoCabe),
    ("CobroPendienteAir", 512, 44, 18, 0, 26, Veredicto::Cumple),
    ("PagoEnCursoAir", 512, 44, 18, 0, 26, Veredicto::Cumple),
    ("MerkleAir", 256, 13, 0, 0, 13, Veredicto::SinLineales),
    ("RefundAir", 64, 12, 0, 0, 12, Veredicto::NoCabe),
    ("RefundAirV2", 64, 16, 4, 0, 12, Veredicto::NoCabe),
    ("CreditClimbAir", 512, 39, 13, 0, 26, Veredicto::Cumple),
    ("MintClimbAir", 512, 42, 16, 0, 26, Veredicto::Cumple),
    ("FrozenClimbAir", 256, 25, 0, 0, 25, Veredicto::SinLineales),
    ("RecoveryClimbAir", 512, 43, 17, 0, 26, Veredicto::Cumple),
];

/// Lo medido con M_lin, clavado: por AIR, T, trozos, lo que FRI abre sin determinar, la mascara
/// del §898 (columna y aleatorizadores), la de las lineales con los aleatorizadores, y las tres
/// juntas; con la semilla 1.
pub(crate) const MLIN_MEDIDO: &[(&str, usize, usize, usize, usize, usize, usize)] = &[
    // AIR, T, trozos, sin determinar, sin lineales (§898), lineales y aleatorizadores, las tres
    ("SendAir", 1024, 8, 1016, 766, 1016, 1016),
    ("SendV2Air", 1024, 8, 1016, 766, 1016, 1016),
    ("ClaimAir", 1024, 8, 1016, 766, 1016, 1016),
    ("ClaimAirV2", 1024, 8, 1016, 766, 1016, 1016),
    ("PrendaAir", 512, 8, 760, 510, 760, 760),
    ("MintPendingClimbAir", 512, 8, 760, 510, 760, 760),
    ("AuditAir", 512, 8, 760, 510, 760, 760),
    ("BurnAir", 1024, 8, 1016, 766, 1016, 1016),
    ("SolvencyAir", 256, 2, 632, 378, 632, 632),
    ("DoubleEntryAir", 1024, 8, 1016, 766, 1016, 1016),
    ("BandaAir", 512, 8, 760, 510, 760, 760),
    ("CobroPendienteAir", 512, 8, 760, 510, 760, 760),
    ("PagoEnCursoAir", 512, 8, 760, 510, 760, 760),
    ("CreditClimbAir", 512, 8, 760, 510, 760, 760),
    ("MintClimbAir", 512, 8, 760, 510, 760, 760),
    ("RecoveryClimbAir", 512, 8, 760, 510, 760, 760),
];

/// La fila del censo de una AIR: (T, reales, lineales, testigo, no lineales, veredicto).
pub(crate) fn censo_declarado(air: &str) -> Option<(usize, usize, usize, usize, usize, Veredicto)> {
    CENSO_134.iter().find(|f| f.0 == air).map(|f| (f.1, f.2, f.3, f.4, f.5, f.6))
}

/// La fila de lo medido de una AIR.
pub(crate) fn medido(air: &str) -> Option<(usize, usize, usize, usize, usize, usize)> {
    MLIN_MEDIDO.iter().find(|f| f.0 == air).map(|f| (f.1, f.2, f.3, f.4, f.5, f.6))
}

/// §905 (la 135): lo medido con la cuenta fina en las familias que no caben con la gruesa
/// (T = 64), clavado: por AIR, T, trozos, el rango de Λ sobre F y su blanco (2q + 2), y, si Λ es
/// sobreyectiva, R3 con la base fina: lo que FRI abre sin determinar, las lineales con los
/// aleatorizadores y las tres juntas (cero si no se mide). Con la semilla 1.
pub(crate) const FINA_MEDIDO: &[(&str, usize, usize, usize, usize, usize, usize, usize)] = &[
    // AIR, T, trozos, rango de Λ, blanco, sin determinar, lineales y aleatorizadores, las tres
    ("NullifierThresholdAir", 64, 15, 36, 86, 0, 0, 0),
    ("ThresholdAir", 64, 15, 86, 86, 168, 120, 158),
    ("EdadAir", 64, 15, 18, 86, 0, 0, 0),
    ("RefundAirV2", 64, 15, 72, 86, 0, 0, 0),
];

/// La fila de lo medido con la cuenta fina de una AIR.
pub(crate) fn fino(air: &str) -> Option<(usize, usize, usize, usize, usize, usize, usize)> {
    FINA_MEDIDO.iter().find(|f| f.0 == air).map(|f| (f.1, f.2, f.3, f.4, f.5, f.6, f.7))
}

/// §905: la medida fina de una familia que no cabe con la gruesa: Λ y, si es sobreyectiva, R3
/// con la base fina. Devuelve la fila de `FINA_MEDIDO` medida.
pub(crate) fn mide_fina(s: &dyn Sonda, g: &Geometria, lineales: &[usize], semilla: u64) -> (usize, usize, usize, usize, usize, usize, usize) {
    let (r, b) = sobreyectiva(s, g, lineales, semilla);
    if r == b && !lineales.is_empty() {
        let m = mide_con_base(s, g, lineales, semilla, LARGO, Base::Vistos);
        (g.t, g.trozos, r, b, m.rango_k, m.rango_lineales, m.rango_mascaras)
    } else {
        (g.t, g.trozos, r, b, 0, 0, 0)
    }
}

// ---- el juguete de la sonda: cuatro columnas con las tres entradas ----

const CICLO_SONDA: usize = 8;

/// Cuatro columnas: la 0 entra con grado 3, la 1 es un transporte (lineal, publica), la 2 es
/// lineal con la pendiente de la 0 (testigo) y la 3 lineal con una pendiente periodica (publica).
struct SondaAir {
    context: AirContext<BaseElement>,
    publico: BaseElement,
}

impl Air for SondaAir {
    type BaseField = BaseElement;
    type PublicInputs = BaseElement;

    fn new(trace_info: TraceInfo, pub_inputs: BaseElement, options: ProofOptions) -> Self {
        let degrees = vec![
            TransitionConstraintDegree::new(3),
            TransitionConstraintDegree::new(1),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::with_cycles(1, vec![CICLO_SONDA]),
        ];
        SondaAir { context: AirContext::new(trace_info, degrees, 2, options), publico: pub_inputs }
    }

    fn context(&self) -> &AirContext<BaseElement> {
        &self.context
    }

    fn get_periodic_column_values(&self) -> Vec<Vec<BaseElement>> {
        let mut v = vec![BaseElement::ONE; CICLO_SONDA];
        v[CICLO_SONDA - 1] = BaseElement::ZERO;
        vec![v]
    }

    fn evaluate_transition<F: FieldElement<BaseField = BaseElement>>(
        &self,
        frame: &EvaluationFrame<F>,
        per: &[F],
        result: &mut [F],
    ) {
        let c = frame.current();
        let n = frame.next();
        result[0] = (n[0] - c[0]) * c[0] * c[0];
        result[1] = n[1] - c[1];
        result[2] = n[2] - c[0] * c[2];
        result[3] = per[0] * (n[3] - (c[3] + c[3]));
    }

    fn get_assertions(&self) -> Vec<Assertion<BaseElement>> {
        vec![Assertion::single(1, 0, self.publico), Assertion::single(3, 0, BaseElement::ONE)]
    }
}

type H = Blake3_256<BaseElement>;
type VC = MerkleTree<H>;

/// Las opciones de produccion sin molienda, como el juguete del §898.
fn opciones() -> ProofOptions {
    ProofOptions::new(
        proof_options().num_queries(),
        proof_options().blowup_factor(),
        0,
        FieldExtension::Quadratic,
        proof_options().to_fri_options().folding_factor(),
        proof_options().to_fri_options().remainder_max_degree(),
        BatchingMethod::Linear,
        BatchingMethod::Linear,
    )
}

const fn s32(n: u64) -> [u8; 32] {
    let b = n.to_le_bytes();
    let mut s = [0x13; 32];
    let mut i = 0;
    while i < 8 {
        s[i] = b[i];
        i += 1;
    }
    s
}

struct ProbadorSonda {
    opciones: ProofOptions,
}

impl Prover for ProbadorSonda {
    type BaseField = BaseElement;
    type Air = SondaAir;
    type Trace = TraceTable<BaseElement>;
    type HashFn = H;
    type VC = VC;
    type RandomCoin = DefaultRandomCoin<H>;
    type TraceLde<F: FieldElement<BaseField = BaseElement>> = DefaultTraceLde<F, H, VC>;
    type ConstraintEvaluator<'a, F: FieldElement<BaseField = BaseElement>> =
        DefaultConstraintEvaluator<'a, SondaAir, F>;
    type ConstraintCommitment<F: FieldElement<BaseField = BaseElement>> =
        DefaultConstraintCommitment<F, H, VC>;

    fn get_pub_inputs(&self, trace: &Self::Trace) -> BaseElement {
        trace.get(1, 0)
    }

    fn options(&self) -> &ProofOptions {
        &self.opciones
    }

    fn ocultacion(&self) -> Option<Ocultacion> {
        Some(Ocultacion {
            m: 64,
            semilla_filas: s32(0x134_0001),
            semilla_cociente: s32(0x134_0002),
            columnas: 1,
            marca_v2: None,
        })
    }

    fn new_trace_lde<F: FieldElement<BaseField = BaseElement>>(
        &self,
        trace_info: &TraceInfo,
        main_trace: &ColMatrix<BaseElement>,
        domain: &StarkDomain<BaseElement>,
        partition_option: PartitionOptions,
    ) -> (Self::TraceLde<F>, TracePolyTable<F>) {
        DefaultTraceLde::new(trace_info, main_trace, domain, partition_option)
    }

    fn new_evaluator<'a, F: FieldElement<BaseField = BaseElement>>(
        &self,
        air: &'a SondaAir,
        aux_rand_elements: Option<AuxRandElements<F>>,
        composition_coefficients: ConstraintCompositionCoefficients<F>,
    ) -> Self::ConstraintEvaluator<'a, F> {
        DefaultConstraintEvaluator::new(air, aux_rand_elements, composition_coefficients)
    }

    fn build_constraint_commitment<F: FieldElement<BaseField = BaseElement>>(
        &self,
        composition_poly_trace: CompositionPolyTrace<F>,
        num_constraint_composition_columns: usize,
        domain: &StarkDomain<BaseElement>,
        partition_options: PartitionOptions,
    ) -> (Self::ConstraintCommitment<F>, CompositionPoly<F>) {
        DefaultConstraintCommitment::new(
            composition_poly_trace,
            num_constraint_composition_columns,
            domain,
            partition_options,
        )
    }
}

/// Los bytes de una prueba oculta del juguete con `t` filas reales: la columna 0 constante 3, la 1
/// constante 7, la 2 las potencias de 3 y la 3 las de 2.
fn prueba_juguete(t: usize) -> Vec<u8> {
    let tres = BaseElement::new(3);
    let dos = BaseElement::new(2);
    let mut c2 = Vec::with_capacity(t);
    let mut c3 = Vec::with_capacity(t);
    let (mut a, mut b) = (BaseElement::ONE, BaseElement::ONE);
    for _ in 0..t {
        c2.push(a);
        c3.push(b);
        a *= tres;
        b *= dos;
    }
    let traza = TraceTable::init(vec![vec![tres; t], vec![BaseElement::new(7); t], c2, c3]);
    ProbadorSonda { opciones: opciones() }.prove(traza).expect("probar el juguete").to_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instrumento_cota::geometria;
    use crate::instrumento_r3::canonica;

    /// El juguete, probado con T = 128 y la geometria del de `instrumento_cota`: su sonda, su
    /// geometria canonica y la prueba, que verifica.
    fn juguete() -> (Box<dyn Sonda>, Geometria) {
        let bytes = prueba_juguete(128);
        let p = Proof::from_bytes(&bytes).expect("leer");
        winterfell::verify::<SondaAir, H, DefaultRandomCoin<H>, VC>(
            p.clone(),
            BaseElement::new(7),
            &winterfell::AcceptableOptions::OptionSet(vec![opciones()]),
        )
        .expect("la prueba del juguete verifica");
        let g = canonica(&geometria::<E>("juguete", &bytes).expect("geometria"));
        let s: Box<dyn Sonda> = Box::new(Oculta::<SondaAir>::new(p.trace_info().clone(), BaseElement::new(7), opciones()));
        (s, g)
    }

    /// **La sonda distingue las tres entradas en el juguete**: grado 3, lineal publica, lineal con
    /// pendiente del testigo y lineal con pendiente periodica; y el juguete tiene la geometria del
    /// de `instrumento_cota` (T 128, 4 trozos, 1 capa, resto 32), con los trozos del contexto.
    #[test]
    fn la_sonda_distingue_las_tres_entradas_en_el_juguete() {
        let (s, g) = juguete();
        assert_eq!((g.t, g.m, g.trozos, g.capas_fri, g.resto), (128, 64, 4, 1, 32));
        assert_eq!((s.reales(), s.auxiliares(), s.trozos(), s.transiciones(), s.aserciones()), (4, 0, 4, 4, 2));
        for semilla in [1u64, 2] {
            let censo = sondea(&*s, semilla);
            assert_eq!(
                censo.entradas,
                [Entrada::NoLineal, Entrada::Lineal, Entrada::Testigo, Entrada::Lineal],
                "semilla {semilla}"
            );
            assert_eq!(censo.lineales(), vec![1, 3]);
            assert_eq!(censo.cuenta(), (2, 1, 1));
        }
    }

    /// **El juguete reproduce lo clavado del §898 sin M_lin, y M_lin suma lo libre dentro de K.**
    /// Sin columnas lineales la medida es la de `instrumento_r3` para la misma geometria -424 sin
    /// determinar, 254 de mascara: la misma O, la misma K, los mismos desafios, otro muestreo-;
    /// con las dos lineales, 38 libres cada una (T − 2q − 6 con T = 128), la mascara crece, no
    /// sale de K (la inclusion se comprueba por rangos dentro de `mide`) y no llega: el juguete no
    /// cumple R3 con M_lin, y lo dice.
    #[test]
    fn el_juguete_reproduce_r3_sin_lineales_y_m_lin_suma_lo_libre() {
        let (s, g) = juguete();
        let t0 = std::time::Instant::now();
        let sin = mide(&*s, &g, &[], 1, TODAS);
        println!("{} · {:?}", linea("juguete sin lineales", &g, &sin), t0.elapsed());
        assert_eq!((sin.rango_k, sin.rango_mascaras, sin.rango_sin_lineales), (424, 254, 254));
        assert_eq!(sin.rango_lineales, 80, "sin lineales quedan los aleatorizadores solos");
        let t0 = std::time::Instant::now();
        let con = mide(&*s, &g, &[1, 3], 1, TODAS);
        println!("{} · {:?}", linea("juguete con las dos lineales", &g, &con), t0.elapsed());
        assert_eq!((con.lineales, con.libres, con.rango_k, con.rango_sin_lineales), (2, 38, 424, 254));
        assert!(con.rango_mascaras > 254 && con.rango_mascaras <= 254 + 76, "{}", con.rango_mascaras);
        assert!(!con.cumple());
    }

    /// **La 134 en las 23 familias, medida una vez** (`#[ignore]`: se corre a mano, en release, y
    /// lo que mide queda clavado en `CENSO_134` y en `MLIN_MEDIDO`; el test de las 23 familias de
    /// `instrumento_cota` comprueba el censo entero en cada canon y mide la solvencia, la barata).
    /// Produce las pruebas del testigo de D-B, sondea cada AIR con la traza y las opciones de su
    /// prueba con m = 64, con dos semillas, y en las que caben (T >= 256) y tienen lineales mide
    /// R3 con M_lin.
    #[test]
    #[ignore = "instrumento de medida: decenas de minutos en release; lo medido queda clavado"]
    fn mide_m_lin_en_las_23_familias() {
        crate::instrumento_cota::recoge();
        crate::instrumento_revela::todas_las_filas();
        let pruebas = crate::instrumento_cota::recoge();
        // MLIN_SOLO=AirA,AirB mide solo esas (la sonda corre en todas): para repetir una medida
        let solo: Option<Vec<String>> =
            std::env::var("MLIN_SOLO").ok().map(|v| v.split(',').map(|s| s.trim().to_string()).collect());
        let mut vistas = BTreeSet::new();
        let mut mal = Vec::new();
        for (fila, bytes) in &pruebas {
            let air = crate::instrumento_cota::air_de(fila);
            let g = canonica(&geometria::<E>(fila, bytes).expect("geometria"));
            if g.m != 64 || !vistas.insert(air.clone()) {
                continue;
            }
            let p = Proof::from_bytes(bytes).expect("leer");
            let s = sonda_de(&air, p.trace_info(), p.options()).unwrap_or_else(|| panic!("{air}: sin sonda"));
            let censo = sondea(&*s, 1);
            if sondea(&*s, 2).entradas != censo.entradas {
                mal.push(format!("{air}: la sonda cambia con la semilla"));
            }
            let (l, t, n) = censo.cuenta();
            println!(
                "[134] {air}: T {} · {} trozos · {} reales + {} auxiliares · {l} lineales {:?} · {t} con pendiente del testigo · {n} no lineales · {} libres por lineal",
                g.t, g.trozos, s.reales(), s.auxiliares(), censo.lineales(), libres_de(&g)
            );
            match censo_declarado(&air) {
                None => mal.push(format!("{air}: sin fila en CENSO_134")),
                Some((dt_, r, dl, dt, dn, _)) if (dt_, r, dl, dt, dn) != (g.t, s.reales(), l, t, n) => mal.push(format!(
                    "{air}: censo medido (T {}, {}, {l}, {t}, {n}) y declarado (T {dt_}, {r}, {dl}, {dt}, {dn})",
                    g.t,
                    s.reales()
                )),
                Some(_) => {},
            }
            let se_mide = solo.as_ref().is_none_or(|v| v.iter().any(|a| *a == air));
            // §905 (la 135): las que no caben con la cuenta gruesa, con la fina
            if libres_de(&g) == 0 && l > 0 && se_mide {
                let t0 = std::time::Instant::now();
                let f = mide_fina(&*s, &g, &censo.lineales(), 1);
                println!("[135] {air}: T {} · {} trozos · Λ de rango {} sobre F, blanco {} · sin determinar {} · lineales y aleatorizadores {} · las tres {} · {:?}", f.0, f.1, f.2, f.3, f.4, f.5, f.6, t0.elapsed());
                match fino(&air) {
                    None => mal.push(format!("{air}: sin fila en FINA_MEDIDO: {f:?}")),
                    Some(c) if c != f => mal.push(format!("{air}: fino medido {f:?} y clavado {c:?}")),
                    Some(_) => {},
                }
            }
            if libres_de(&g) > 0 && l > 0 && se_mide {
                let t0 = std::time::Instant::now();
                let m = mide(&*s, &g, &censo.lineales(), 1, LARGO);
                println!("{} · {:?}", linea(&air, &g, &m), t0.elapsed());
                let medida = (g.t, g.trozos, m.rango_k, instrumento_r3_clavado(&g), m.rango_lineales, m.rango_mascaras);
                match medido(&air) {
                    None => mal.push(format!("{air}: sin fila en MLIN_MEDIDO: {medida:?}")),
                    Some(clavado) if clavado != medida => mal.push(format!("{air}: medido {medida:?} y clavado {clavado:?}")),
                    Some(_) => {},
                }
            }
        }
        assert_eq!(vistas.len(), 23, "familias vistas");
        assert!(mal.is_empty(), "la 134 no es lo clavado:\n{}", mal.join("\n"));
    }

    /// **§905 (la 135): la base fina en el juguete.** Condicionar cada lineal a lo visto (q + 4)
    /// en vez de a lo tocado (2q + 6) deja 82 libres por columna en vez de 38; Λ, lo que C ve de
    /// las lineales en S_ab y el verificador no, es sobreyectiva con las dos (86 de 86 sobre F) y
    /// nula sin ellas; y la mascara con la base fina no baja de la gruesa ni sale de K.
    #[test]
    fn el_juguete_con_la_base_fina_tiene_lambda_sobre_y_mas_mascara() {
        let (s, g) = juguete();
        let (r, b) = sobreyectiva(&*s, &g, &[1, 3], 1);
        println!("[135] juguete: Λ de rango {r} sobre F, blanco {b}");
        assert_eq!((r, b), (86, 86));
        assert_eq!(sobreyectiva(&*s, &g, &[], 1), (0, 86));
        let t0 = std::time::Instant::now();
        let fina = mide_con_base(&*s, &g, &[1, 3], 1, LARGO, Base::Vistos);
        println!("{} · base fina · {:?}", linea("juguete", &g, &fina), t0.elapsed());
        assert_eq!((fina.lineales, fina.libres, fina.rango_k), (2, libres_finos_de(&g), 424));
        assert_eq!(libres_finos_de(&g), 82);
        // la fina suma dos dimensiones a la gruesa (156 y 330): de las 2 · 82 libres, 86 se van en
        // hacer uniforme C(S_ab) -el nucleo conjunto fija cada trozo en S_ab- y quedan 78, frente
        // a las 2 · 38 = 76 de la gruesa
        assert_eq!((fina.rango_lineales, fina.rango_mascaras), (158, 332));
    }

    /// La mascara del §898 de una geometria, clavada en `instrumento_r3` (las dos juntas).
    fn instrumento_r3_clavado(g: &Geometria) -> usize {
        crate::instrumento_r3::clavado(g).map(|c| c.3).unwrap_or(0)
    }

    /// **La tabla del censo es de las 23 familias y dice quien necesita la 133.** Cada fila es
    /// coherente consigo misma y con lo clavado: con T = 64 no cabe; sin lineales, M_lin es cero y
    /// R3 es la del §898; con lineales, hay fila medida con la misma T, su mascara del §898 es la
    /// clavada en `instrumento_r3`, las lineales no bajan de lo que ponen los aleatorizadores
    /// solos ni las tres juntas de lo del §898, nada sale de K, y cumple si y solo si las tres
    /// juntas cubren todo lo que FRI abre sin determinar.
    #[test]
    fn la_tabla_del_censo_es_de_las_23_y_dice_quien_necesita_la_133() {
        assert_eq!(CENSO_134.len(), 23);
        let nombres: BTreeSet<&str> = CENSO_134.iter().map(|f| f.0).collect();
        assert_eq!(nombres.len(), 23, "una fila por AIR");
        let de_filas: BTreeSet<&str> = crate::instrumento_revela::FILAS.iter().map(|f| f.0).collect();
        assert_eq!(nombres, de_filas, "las AIR del censo son las de FILAS");
        let mut necesitan = 0;
        for &(air, t, reales, l, tst, n, veredicto) in CENSO_134 {
            assert_eq!(l + tst + n, reales, "{air}: las entradas suman las columnas reales");
            // las capas de FRI y el resto de cada T con las opciones de produccion (blowup 16,
            // plegado 8, resto de 32 elementos): lo que leen las pruebas reales en instrumento_cota
            let (capas_fri, resto) = match t {
                64 => (1, 16),
                256 => (2, 8),
                512 => (2, 16),
                1024 => (2, 32),
                _ => panic!("{air}: T = {t} sin geometria de FRI conocida"),
            };
            let geo = |trozos: usize| Geometria {
                q: 42,
                consultas: 42,
                t,
                m: 64,
                ancho: reales + 1,
                ancho_aux: 0,
                trozos,
                capas_fri,
                resto,
                bytes: 0,
            };
            match veredicto {
                Veredicto::NoCabe => {
                    assert_eq!(t, 64, "{air}: no cabe solo con T = 64");
                    assert_eq!(libres_de(&geo(0)), 0);
                    assert!(medido(air).is_none(), "{air}: no cabe y tiene medida");
                    necesitan += 1;
                },
                Veredicto::SinLineales => {
                    assert!(t >= 256 && l == 0, "{air}: sin lineales con T = {t} y {l} lineales");
                    assert!(medido(air).is_none(), "{air}: sin lineales y con medida");
                    necesitan += 1;
                },
                Veredicto::Cumple | Veredicto::NoCumple => {
                    assert!(t >= 256 && l >= 1, "{air}: medido con T = {t} y {l} lineales");
                    let (mt, trozos, k, sin, lin, tres) = medido(air).unwrap_or_else(|| panic!("{air}: sin medida"));
                    assert_eq!(mt, t, "{air}: la medida es de otra T");
                    let (ck, _, trozos_solos, c_sin) =
                        crate::instrumento_r3::clavado(&geo(trozos)).unwrap_or_else(|| panic!("{air}: geometria sin R3"));
                    assert_eq!((k, sin), (ck, c_sin), "{air}: K y la mascara del §898 son las clavadas en instrumento_r3");
                    assert!(lin >= trozos_solos, "{air}: las lineales no bajan de los aleatorizadores solos");
                    assert!(tres >= sin && tres >= lin && tres <= k, "{air}: las tres juntas, entre lo del §898 y K");
                    assert_eq!(veredicto == Veredicto::Cumple, tres == k, "{air}: el veredicto es el de la medida");
                    if veredicto == Veredicto::NoCumple {
                        necesitan += 1;
                    }
                },
            }
        }
        println!("[134] {necesitan} de 23 familias necesitan la 133 para R3 (o la 131 antes)");
        assert_eq!(MLIN_MEDIDO.len(), CENSO_134.iter().filter(|f| matches!(f.6, Veredicto::Cumple | Veredicto::NoCumple)).count());
    }
}
