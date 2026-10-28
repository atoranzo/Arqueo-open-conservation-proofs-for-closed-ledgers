//! **A3 del HVZK (§898): los falsadores mas alla del censo, en un juguete con la geometria de
//! produccion.**
//!
//! El censo de E2 cuenta literales; estos tests hacen lo que un adversario haria con los bytes
//! de una prueba: rehacen la moneda publica como el verificador -de la prueba salen las
//! posiciones, z, los coeficientes de las restricciones y lo abierto- y atacan la columna
//! secreta con algebra lineal, con la lema L1 de `doc/ocultacion-hvzk.md` delante. El juguete es
//! `ConstanteAir`: una columna constante secreta `c`, la forma que tiene una clave en una traza de
//! la casa, con una transicion de grado 3 -(c' − c)·c·c, para que el cociente oculto tenga trozos
//! con T = 128, como `WorkAir`- y una columna publica constante con dos aserciones, porque un AIR
//! necesita alguna y la secreta no puede llevarla. Campo de produccion, extension cuadratica, 42
//! consultas, blowup 16, m = 64 (32 cuando T = 32, que no admite mas), sin molienda ni sal: las
//! dos no cambian lo que se abre, y sin ellas el juguete es determinista.
//!
//! 1. **La cota de la traza tiene dientes.** Con T = 32 filas aleatorias y q + 4 = 46 ecuaciones
//!    sobre F -las 42 posiciones, z y z·g- frente a T + 1 = 33 incognitas -la constante y los T
//!    coeficientes de la mascara, porque la columna es `c·I_real + Z_real·R`, con I_real el
//!    indicador de las filas reales y R de grado < T-, la constante sale por eliminacion. Con T = 64, las mismas 46 ecuaciones sobre 65 incognitas no
//!    la determinan: L1.
//! 2. **Con T = 64 la vista dice el testigo entre dos candidatos; con T = 128, no.** Lo que una
//!    prueba TOCA por columna son 2q + 3 puntos -las posiciones, sus filas siguientes (que salen
//!    de C(x_j), porque la composicion es afin en el valor siguiente de la columna secreta), z,
//!    z·g y g²·z (de C(z·g))-: 90 dimensiones sobre F. Con T = 64 las 64 filas aleatorias no las
//!    cubren, y para el candidato bueno los residuos caen en `Z_real·F[x]_{<T}` y para el malo no.
//!    Con T = 128 caen para los dos. Es la 131 de `BACKLOG.md`, medida, y como dijo la revision
//!    del §897 no es una recuperacion -87 ecuaciones sobre 128 incognitas- sino un distinguidor.
//! 3. **La semilla repetida recupera lo que cada prueba sola oculta.** Dos pruebas de dos
//!    constantes con la misma semilla de filas comparten la mascara R; sus 91 ecuaciones (46 y 45:
//!    una de las dos abre 41 posiciones unicas) sobre 2 + 64 incognitas devuelven las dos
//!    constantes con T = 64, donde cada prueba sola (46 sobre 65) no devuelve ninguna. Con semillas
//!    distintas, 91 sobre 130: ninguna. Es H9 del documento, y por que `sal::semilla()` es fresca en
//!    cada prueba (§652).
//!
//! Lo que valida la moneda rehecha no es el aserto de las posiciones unicas, que es debil, sino el
//! ataque 1: 45 ecuaciones sobre 33 incognitas solo son consistentes si z y las posiciones son las
//! de la prueba.
//!
//! Lo que NO hace: no ataca ninguna prueba de la casa ni ningun AIR del cable; no toca la
//! promesa; no mide R3 (`instrumento_r3`); no corre ningun distinguidor estadistico -el de aqui
//! es algebraico y decide con certeza-.

use std::collections::BTreeSet;

use winter_air::proof::merge_ood_evaluations;
use winter_air::{Air, AirContext, Assertion, Marca, Oculta, TransitionConstraintDegree};
use winter_prover::Ocultacion;
use winterfell::crypto::{hashers::Blake3_256, DefaultRandomCoin, ElementHasher, MerkleTree, RandomCoin};
use winterfell::math::fields::f64::BaseElement;
use winterfell::math::fields::QuadExtension;
use winterfell::math::{FieldElement, StarkField, ToElements};
use winterfell::matrix::ColMatrix;
use winterfell::{
    verify, AcceptableOptions, AuxRandElements, BatchingMethod, CompositionPoly,
    CompositionPolyTrace, ConstraintCompositionCoefficients, DefaultConstraintCommitment,
    DefaultConstraintEvaluator, DefaultTraceLde, EvaluationFrame, FieldExtension,
    PartitionOptions, Proof, ProofOptions, Prover, StarkDomain, TraceInfo, TracePolyTable,
    TraceTable,
};

use crate::proof_options;

type E = QuadExtension<BaseElement>;
type H = Blake3_256<BaseElement>;
type VC = MerkleTree<H>;
type Moneda = DefaultRandomCoin<H>;

/// La constante publica de la segunda columna.
const PUBLICO: u64 = 7;

// ---- el juguete ----

/// Una columna constante secreta y una publica; ver la cabecera del modulo.
pub struct ConstanteAir {
    context: AirContext<BaseElement>,
    publico: BaseElement,
}

impl Air for ConstanteAir {
    type BaseField = BaseElement;
    type PublicInputs = BaseElement;

    fn new(trace_info: TraceInfo, pub_inputs: BaseElement, options: ProofOptions) -> Self {
        let degrees = vec![TransitionConstraintDegree::new(3)];
        ConstanteAir { context: AirContext::new(trace_info, degrees, 2, options), publico: pub_inputs }
    }

    fn context(&self) -> &AirContext<BaseElement> {
        &self.context
    }

    fn evaluate_transition<F: FieldElement<BaseField = BaseElement>>(
        &self,
        frame: &EvaluationFrame<F>,
        _periodic: &[F],
        result: &mut [F],
    ) {
        let c = frame.current()[0];
        result[0] = (frame.next()[0] - c) * c * c;
    }

    fn get_assertions(&self) -> Vec<Assertion<BaseElement>> {
        let ultima = self.trace_length() - 1;
        vec![Assertion::single(1, 0, self.publico), Assertion::single(1, ultima, self.publico)]
    }
}

/// Las opciones de produccion sin molienda: 42 consultas, blowup 16, extension cuadratica, el
/// plegado y el resto de FRI y el lote lineal.
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
    let mut s = [0x5a; 32];
    let mut i = 0;
    while i < 8 {
        s[i] = b[i];
        i += 1;
    }
    s
}

fn ocultacion(m: usize, filas: u64, cociente: u64) -> Ocultacion {
    Ocultacion { m, semilla_filas: s32(filas), semilla_cociente: s32(cociente), columnas: 1, marca_v2: None }
}

struct ProbadorConstante {
    opciones: ProofOptions,
    ocultacion: Ocultacion,
}

impl Prover for ProbadorConstante {
    type BaseField = BaseElement;
    type Air = ConstanteAir;
    type Trace = TraceTable<BaseElement>;
    type HashFn = H;
    type VC = VC;
    type RandomCoin = Moneda;
    type TraceLde<F: FieldElement<BaseField = BaseElement>> = DefaultTraceLde<F, H, VC>;
    type ConstraintEvaluator<'a, F: FieldElement<BaseField = BaseElement>> =
        DefaultConstraintEvaluator<'a, ConstanteAir, F>;
    type ConstraintCommitment<F: FieldElement<BaseField = BaseElement>> =
        DefaultConstraintCommitment<F, H, VC>;

    fn get_pub_inputs(&self, trace: &Self::Trace) -> BaseElement {
        trace.get(1, 0)
    }

    fn options(&self) -> &ProofOptions {
        &self.opciones
    }

    fn ocultacion(&self) -> Option<Ocultacion> {
        Some(self.ocultacion)
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
        air: &'a ConstanteAir,
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

/// Los bytes de una prueba oculta de la constante `c` con `t` filas reales y la ocultacion dada.
fn prueba(c: u64, t: usize, oc: Ocultacion) -> Vec<u8> {
    let traza = TraceTable::init(vec![
        vec![BaseElement::new(c); t],
        vec![BaseElement::new(PUBLICO); t],
    ]);
    ProbadorConstante { opciones: opciones(), ocultacion: oc }
        .prove(traza)
        .expect("probar la constante")
        .to_bytes()
}

fn verifica(bytes: &[u8]) {
    let proof = Proof::from_bytes(bytes).expect("leer");
    verify::<ConstanteAir, H, Moneda, VC>(
        proof,
        BaseElement::new(PUBLICO),
        &AcceptableOptions::OptionSet(vec![opciones()]),
    )
    .expect("la prueba del juguete verifica");
}

// ---- la vista: la moneda publica, rehecha como la rehace el verificador ----

/// Lo que un adversario saca de los bytes de una prueba: los desafios, que derive de la moneda
/// como el verificador, y lo abierto.
struct Vista {
    t: usize,
    s: usize,
    g_traza: E,
    /// Las posiciones abiertas, como puntos del dominio LDE, en el orden de las tablas.
    x: Vec<E>,
    z: E,
    coefs: ConstraintCompositionCoefficients<E>,
    /// Las filas abiertas de la traza principal: [c, publica, aleatoria] en cada posicion.
    traza: Vec<Vec<BaseElement>>,
    /// Los trozos abiertos en cada posicion.
    trozos: Vec<Vec<E>>,
    /// El marco OOD de la traza, en z y en z·g, y el de los trozos.
    ood: (Vec<E>, Vec<E>),
    ood_trozos: (Vec<E>, Vec<E>),
    air: Oculta<ConstanteAir>,
}

fn vista(bytes: &[u8]) -> Vista {
    let publico = BaseElement::new(PUBLICO);
    let proof = Proof::from_bytes(bytes).expect("leer");
    let air = Oculta::<ConstanteAir>::new(proof.trace_info().clone(), publico, proof.options().clone());
    let lde = air.lde_domain_size();
    let largo = air.trace_length();
    let capas = air.options().to_fri_options().num_fri_layers(lde);
    let n = air.context().num_constraint_composition_columns();
    let ancho = air.trace_info().main_trace_width();
    let m = Marca::leer(proof.trace_info().meta()).expect("meta").expect("marca").m;

    // la moneda: el contexto y las entradas publicas, y despues lo que la prueba compromete
    let mut semilla: Vec<BaseElement> = proof.context.to_elements();
    semilla.append(&mut publico.to_elements());
    let mut moneda = Moneda::new(&semilla);
    let (raices_traza, raiz_cociente, raices_fri) =
        proof.commitments.clone().parse::<H>(1, capas).expect("compromisos");
    moneda.reseed(raices_traza[0]);
    let coefs = air
        .get_constraint_composition_coefficients::<E, Moneda>(&mut moneda)
        .expect("coeficientes de las restricciones");
    moneda.reseed(raiz_cociente);
    let z: E = moneda.draw().expect("z");
    let (ood_traza, ood_trozos) = proof.ood_frame.clone().parse::<E>(ancho, 0, n).expect("marco OOD");
    moneda.reseed(H::hash_elements(&merge_ood_evaluations(&ood_traza, &ood_trozos)));
    let _cc = air.get_deep_composition_coefficients::<E, Moneda>(&mut moneda).expect("cc");
    for raiz in raices_fri {
        moneda.reseed(raiz);
        let _alfa: E = moneda.draw().expect("alfa");
    }
    let mut posiciones = moneda
        .draw_integers(air.options().num_queries(), lde, proof.pow_nonce)
        .expect("posiciones");
    posiciones.sort_unstable();
    posiciones.dedup();
    assert_eq!(
        posiciones.len(),
        proof.num_unique_queries as usize,
        "la moneda rehecha no da las posiciones unicas de la prueba"
    );
    let q = posiciones.len();
    let (_, tabla_traza) = proof.trace_queries[0]
        .clone()
        .parse::<BaseElement, H, VC>(lde, q, ancho)
        .expect("consultas de la traza");
    let (_, tabla_trozos) =
        proof.constraint_queries.clone().parse::<E, H, VC>(lde, q, n).expect("consultas del cociente");
    let omega = BaseElement::get_root_of_unity(lde.ilog2());
    Vista {
        t: largo / 2,
        s: largo - m,
        g_traza: E::from(BaseElement::get_root_of_unity(largo.ilog2())),
        x: posiciones.iter().map(|&p| E::from(BaseElement::GENERATOR * omega.exp(p as u64))).collect(),
        z,
        coefs,
        traza: (0..q).map(|j| tabla_traza.get_row(j).to_vec()).collect(),
        trozos: (0..q).map(|j| tabla_trozos.get_row(j).to_vec()).collect(),
        ood: (ood_traza.current_row().to_vec(), ood_traza.next_row().to_vec()),
        ood_trozos: (ood_trozos.current_row().to_vec(), ood_trozos.next_row().to_vec()),
        air,
    }
}

/// C(y) sobre un marco, como lo calcula el verificador en z (`evaluate_constraints`, sin tramo
/// auxiliar ni columnas periodicas, que el juguete no tiene).
fn composicion(v: &Vista, actual: &[E], siguiente: &[E], y: E) -> E {
    let frame = EvaluationFrame::from_rows(actual.to_vec(), siguiente.to_vec());
    let tc = v.air.get_transition_constraints(&v.coefs.transition);
    let mut evaluaciones = vec![E::ZERO; tc.num_main_constraints()];
    v.air.evaluate_transition(&frame, &[], &mut evaluaciones);
    let ninguna: Vec<E> = Vec::new();
    let mut r = tc.combine_evaluations::<E>(&evaluaciones, &ninguna, y);
    let bc = v.air.get_boundary_constraints::<E>(None, &v.coefs.boundary);
    for grupo in bc.main_constraints().iter() {
        r += grupo.evaluate_at(frame.current(), y);
    }
    r
}

/// El valor de la columna secreta en g·y, despejado de C(y) = Σ_k y^{k·s}·h'_k(y): la composicion
/// es afin en ese valor, y la columna publica y la aleatoria no entran en la transicion.
fn siguiente_de(v: &Vista, actual: &[E], trozos: &[E], y: E) -> E {
    let observado = trozos
        .iter()
        .enumerate()
        .fold(E::ZERO, |acc, (k, h)| acc + y.exp(((k * v.s) as u64).into()) * *h);
    let mut siguiente = vec![E::ZERO; actual.len()];
    let en_cero = composicion(v, actual, &siguiente, y);
    siguiente[0] = E::ONE;
    let en_uno = composicion(v, actual, &siguiente, y);
    let pendiente = en_uno - en_cero;
    // la pendiente es α·c²/Z(y): con una constante secreta 0 no habria nada que despejar
    assert!(pendiente != E::ZERO, "la composicion no depende del valor siguiente en este punto");
    (observado - en_cero) / pendiente
}

/// I_real(y) = Σ_{r<T} L_r(y), la base de Lagrange de las filas reales sumada: vale 1 en ellas y
/// 0 en las aleatorias. Una columna constante `c` con filas aleatorias ρ es `c·I_real + S`, con
/// S = Z_real·R y R ∈ F[x] de grado < T, determinado por ρ y solo por ρ: por eso dos pruebas con
/// la misma semilla de filas comparten R aunque no compartan c.
fn indicador_real(v: &Vista, y: E) -> E {
    let largo = 2 * v.t;
    let zh = y.exp((largo as u64).into()) - E::ONE;
    let mut suma = E::ZERO;
    let mut h = E::ONE;
    for _ in 0..v.t {
        suma += h / (y - h);
        h *= v.g_traza;
    }
    zh * suma / E::from(BaseElement::new(largo as u64))
}

/// Z_real(y) = Π_{r<T} (y − g^r): lo que anula las filas reales.
fn z_real(v: &Vista, y: E) -> E {
    let mut z = E::ONE;
    let mut h = E::ONE;
    for _ in 0..v.t {
        z *= y - h;
        h *= v.g_traza;
    }
    z
}

/// Los puntos ABIERTOS de la columna secreta con su valor: las posiciones, z y z·g.
fn abiertos(v: &Vista) -> Vec<(E, E)> {
    let mut p: Vec<(E, E)> = v.x.iter().zip(&v.traza).map(|(&x, fila)| (x, E::from(fila[0]))).collect();
    p.push((v.z, v.ood.0[0]));
    p.push((v.z * v.g_traza, v.ood.1[0]));
    p
}

/// Los puntos TOCADOS: los abiertos y, de C, las filas siguientes y g²·z. Sin repetidos.
fn tocados(v: &Vista) -> Vec<(E, E)> {
    let mut p = abiertos(v);
    for ((&x, fila), trozos) in v.x.iter().zip(&v.traza).zip(&v.trozos) {
        let actual: Vec<E> = fila.iter().map(|&a| E::from(a)).collect();
        p.push((x * v.g_traza, siguiente_de(v, &actual, trozos, x)));
    }
    let zg = v.z * v.g_traza;
    p.push((zg * v.g_traza, siguiente_de(v, &v.ood.1, &v.ood_trozos.1, zg)));
    let mut vistos = BTreeSet::new();
    p.into_iter()
        .filter(|(y, _)| {
            let mut b = y.base_element(0).as_int().to_le_bytes().to_vec();
            b.extend_from_slice(&y.base_element(1).as_int().to_le_bytes());
            vistos.insert(b)
        })
        .collect()
}

// ---- algebra lineal sobre F ----

/// Un sistema lineal sobre F, con las incognitas en columnas y el termino independiente al final.
struct Sistema {
    n: usize,
    filas: Vec<Vec<BaseElement>>,
}

impl Sistema {
    fn nuevo(n: usize) -> Self {
        Sistema { n, filas: Vec::new() }
    }

    /// Una ecuacion sobre E, `Σ coef_i·u_i = rhs`, como sus dos ecuaciones sobre F; la segunda
    /// solo si no es la ecuacion vacia (un punto de F la tiene vacia).
    fn ecuacion(&mut self, coefs: &[(usize, E)], rhs: E) {
        for k in 0..2 {
            let mut fila = vec![BaseElement::ZERO; self.n + 1];
            for &(i, c) in coefs {
                fila[i] += c.base_element(k);
            }
            fila[self.n] = rhs.base_element(k);
            if fila.iter().any(|x| *x != BaseElement::ZERO) {
                self.filas.push(fila);
            }
        }
    }

    /// `c·I_real(y) + Z_real(y)·R(y) = valor`, con la constante en la columna `col_c` -o ya
    /// restada, si se conoce- y los T coeficientes de R desde la columna `col_r`.
    fn punto(&mut self, v: &Vista, y: E, valor: E, col_c: Option<usize>, col_r: usize, c: Option<E>) {
        let (z, indicador) = (z_real(v, y), indicador_real(v, y));
        let mut coefs = Vec::with_capacity(v.t + 1);
        if let Some(cc) = col_c {
            coefs.push((cc, indicador));
        }
        let mut pot = E::ONE;
        for i in 0..v.t {
            coefs.push((col_r + i, z * pot));
            pot *= y;
        }
        self.ecuacion(&coefs, valor - c.map_or(E::ZERO, |c| c * indicador));
    }

    /// La forma escalonada reducida; devuelve (consistente, los pivotes).
    fn reduce(&mut self) -> bool {
        let n = self.n;
        let filas = self.filas.len();
        let mut r = 0;
        for c in 0..n {
            if r == filas {
                break;
            }
            let Some(piv) = (r..filas).find(|&i| self.filas[i][c] != BaseElement::ZERO) else {
                continue;
            };
            self.filas.swap(r, piv);
            let inv = self.filas[r][c].inv();
            for x in self.filas[r].iter_mut() {
                *x *= inv;
            }
            let fila_r = self.filas[r].clone();
            for (i, fila) in self.filas.iter_mut().enumerate() {
                if i != r && fila[c] != BaseElement::ZERO {
                    let f = fila[c];
                    for (x, y) in fila.iter_mut().zip(fila_r.iter()) {
                        *x -= f * *y;
                    }
                }
            }
            r += 1;
        }
        self.filas.iter().all(|f| f[..n].iter().any(|x| *x != BaseElement::ZERO) || f[n] == BaseElement::ZERO)
    }

    /// Tras `reduce`: el valor de la incognita `j` si el sistema la determina.
    fn determinada(&self, j: usize) -> Option<BaseElement> {
        self.filas
            .iter()
            .find(|f| f[j] == BaseElement::ONE && f[..self.n].iter().enumerate().all(|(i, x)| i == j || *x == BaseElement::ZERO))
            .map(|f| f[self.n])
    }
}

/// Ataque 1: la constante por eliminacion sobre lo ABIERTO de una prueba. `Some(c)` si las
/// q + 4 ecuaciones la determinan.
fn recupera(v: &Vista) -> (usize, Option<BaseElement>) {
    let mut sis = Sistema::nuevo(1 + v.t);
    for (y, valor) in abiertos(v) {
        sis.punto(v, y, valor, Some(0), 1, None);
    }
    let ecuaciones = sis.filas.len();
    assert!(sis.reduce(), "lo abierto no es consistente con c + Z_real·R");
    (ecuaciones, sis.determinada(0))
}

/// Ataque 2: entre dos candidatos, cual es consistente con lo TOCADO. `Some(i)` si solo uno.
fn decide(v: &Vista, candidatos: [u64; 2]) -> (usize, Option<usize>) {
    let puntos = tocados(v);
    let dims = puntos.iter().map(|(y, _)| if y.base_element(1) == BaseElement::ZERO { 1 } else { 2 }).sum();
    let consistentes: Vec<bool> = candidatos
        .iter()
        .map(|&c| {
            let mut sis = Sistema::nuevo(v.t);
            for &(y, valor) in &puntos {
                sis.punto(v, y, valor, None, 0, Some(E::from(BaseElement::new(c))));
            }
            sis.reduce()
        })
        .collect();
    let cual = match (consistentes[0], consistentes[1]) {
        (true, false) => Some(0),
        (false, true) => Some(1),
        _ => None,
    };
    (dims, cual)
}

/// Ataque 3: dos pruebas de dos constantes. Con `comparten`, las dos llevan la misma mascara R;
/// si no, cada una la suya. Devuelve (ecuaciones, incognitas, las dos constantes si salen).
fn recupera_dos(a: &Vista, b: &Vista, comparten: bool) -> (usize, usize, Option<(BaseElement, BaseElement)>) {
    let n = if comparten { 2 + a.t } else { 2 + a.t + b.t };
    let mut sis = Sistema::nuevo(n);
    for (y, valor) in abiertos(a) {
        sis.punto(a, y, valor, Some(0), 2, None);
    }
    for (y, valor) in abiertos(b) {
        sis.punto(b, y, valor, Some(1), if comparten { 2 } else { 2 + a.t }, None);
    }
    let ecuaciones = sis.filas.len();
    assert!(sis.reduce(), "lo abierto de las dos no es consistente");
    (ecuaciones, n, sis.determinada(0).zip(sis.determinada(1)))
}

// ======================= LOS TESTS =======================

const SECRETO: u64 = 0x5ec2_e70c_0b5a_1d3f;
const OTRO: u64 = 0x0a1b_2c3d_4e5f_6071;

/// **La cota de la traza tiene dientes: con T = 32 la constante sale; con T = 64, no.**
#[test]
fn con_t_32_la_constante_sale_por_eliminacion_y_con_t_64_no() {
    let bytes = prueba(SECRETO, 32, ocultacion(32, 1, 2));
    verifica(&bytes);
    let v = vista(&bytes);
    let (ecuaciones, c) = recupera(&v);
    println!("[A3] T 32: {ecuaciones} ecuaciones sobre F, {} incognitas: c = {c:?}", 1 + v.t);
    assert!(ecuaciones >= 1 + v.t, "{ecuaciones} ecuaciones no bastan para {} incognitas", 1 + v.t);
    assert_eq!(c, Some(BaseElement::new(SECRETO)), "con T = 32 la constante sale");

    let bytes = prueba(SECRETO, 64, ocultacion(64, 1, 2));
    verifica(&bytes);
    let v = vista(&bytes);
    let (ecuaciones, c) = recupera(&v);
    println!("[A3] T 64: {ecuaciones} ecuaciones sobre F, {} incognitas: c = {c:?}", 1 + v.t);
    assert!(ecuaciones < 1 + v.t);
    assert_eq!(c, None, "con T = 64 lo abierto no determina la constante (L1)");
}

/// **Con T = 64 lo tocado dice el testigo entre dos candidatos; con T = 128, no (la 131).**
#[test]
fn con_t_64_lo_tocado_dice_el_testigo_y_con_t_128_no() {
    for (t, espera) in [(64usize, true), (128, false)] {
        for (i, c) in [SECRETO, OTRO].into_iter().enumerate() {
            let bytes = prueba(c, t, ocultacion(64, 3, 4));
            verifica(&bytes);
            let v = vista(&bytes);
            let (dims, cual) = decide(&v, [SECRETO, OTRO]);
            println!("[A3] T {t}, testigo {i}: {dims} dimensiones tocadas sobre F frente a T = {t}: decide {cual:?}");
            if espera {
                assert_eq!(cual, Some(i), "con T = {t} la vista dice el testigo");
            } else {
                assert_eq!(cual, None, "con T = {t} los dos candidatos son consistentes");
            }
        }
    }
}

/// **La semilla repetida recupera lo que cada prueba sola oculta (H9).**
#[test]
fn la_semilla_repetida_recupera_las_dos_constantes_y_la_fresca_no() {
    let a = prueba(SECRETO, 64, ocultacion(64, 9, 10));
    let b = prueba(OTRO, 64, ocultacion(64, 9, 11));
    verifica(&a);
    verifica(&b);
    let (va, vb) = (vista(&a), vista(&b));
    assert_eq!(recupera(&va).1, None, "cada prueba sola no da su constante");
    assert_eq!(recupera(&vb).1, None);
    let (ecuaciones, incognitas, dos) = recupera_dos(&va, &vb, true);
    println!("[A3] misma semilla de filas: {ecuaciones} ecuaciones sobre {incognitas} incognitas: {dos:?}");
    assert_eq!(dos, Some((BaseElement::new(SECRETO), BaseElement::new(OTRO))));

    let b = prueba(OTRO, 64, ocultacion(64, 12, 11));
    let vb = vista(&b);
    let (ecuaciones, incognitas, dos) = recupera_dos(&va, &vb, false);
    println!("[A3] semillas distintas: {ecuaciones} ecuaciones sobre {incognitas} incognitas: {dos:?}");
    assert_eq!(dos, None, "con semillas frescas las dos pruebas no dan ninguna constante");
}

