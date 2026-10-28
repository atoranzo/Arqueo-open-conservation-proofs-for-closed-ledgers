//! **A1 del HVZK (RFC-0009 D-I; §896): la cota de ocultacion, medida sobre las pruebas reales de
//! los 23 probadores con fila, y atada.**
//!
//! La D-I del RFC-0009 dice en prosa lo que un simulador necesita: «44 puntos abiertos por
//! columna -las 42 consultas, z y z·g- frente a T filas aleatorias», y nadie lo media por familia.
//! Este modulo lo mide. Para cada prueba que produce el testigo de lo que revela una prueba
//! (`instrumento_revela`: las 23 AIR con probador oculto, el cable y las cuatro experimentales de
//! la tabla de D-B) lee de sus bytes cuantos puntos distintos recibe el verificador de cada objeto
//! comprometido y los compara con los grados de libertad del aleatorizador que los enmascara. Es
//! SOLO de tests (`cfg(test)` en `lib.rs`): no crece la API de la capa ni la del probador. Se
//! llama `instrumento_` como el de D-B porque lo es, y porque el censo del cable deja fuera a los
//! instrumentos: el `verify` del juguete de abajo no es del cable.
//!
//! - **Lo que se abre, leido de la prueba.** De cada columna de la traza, principal y auxiliar, el
//!   verificador recibe su valor en las `q` posiciones UNICAS del dominio LDE
//!   (`num_unique_queries`, leido de la prueba) y en `z` y `z·g` (el marco OOD); de cada trozo del
//!   cociente, lo mismo: `QuotientOodFrame` lleva los trozos en `z` y en `z·g`. Las posiciones no
//!   viajan en la prueba -las deriva el verificador de la moneda-; su NUMERO si.
//! - **La direccion residual.** DEEP suma cada columna y cada trozo con su coeficiente, y FRI abre
//!   combinaciones de ese polinomio en sus capas y entrega su resto entero. La columna aleatoria
//!   entra en DEEP como `c·u(x)·(2x − z − z·g)`, con `u` uniforme de grado 2T − 3 (lo que queda de
//!   un polinomio uniforme de grado < 2T tras revelar `r(z)` y `r(z·g)`), asi que enmascara DEEP en
//!   todas las direcciones menos UNA, la de la raiz de ese factor lineal. Por eso cada cota cuenta
//!   una abertura mas de las que se ven, [`DIRECCION_RESIDUAL`]: es lo que A2 tiene que demostrar
//!   (su lema L3) y lo que aqui se cuenta sin demostrar.
//! - **La cota, por objeto.** Por columna: `q + 2 + 1 <= T`, las T filas aleatorias que el
//!   envoltorio `Oculta<A>` pone detras de las reales (RFC-0009 D-G). Por trozo del cociente:
//!   `q + 2 + 1 <= m`, el grado de los aleatorizadores de `segmentar`; m viaja en la marca (D-S).
//!   Los N trozos llevan N − 1 aleatorizadores que se cancelan en la suma, y la relacion que se
//!   cancela es publica en cada punto: la cota es por trozo y no depende de N.
//! - **El rango.** Que las aberturas de una columna sean uniformes no lo da solo el numero: la
//!   matriz de los polinomios de Lagrange de las filas aleatorias, evaluados en los puntos
//!   abiertos, tiene que ser de rango pleno -es una matriz de Cauchy, con los puntos fuera del
//!   dominio de la traza-, y la de los trozos, una de Vandermonde de grado < m. Se calculan en el
//!   campo de produccion, con su extension cuadratica, para la geometria (q, T, m) de cada prueba y
//!   con puntos que elige el test: la lema vale para cualesquiera puntos distintos fuera del
//!   dominio, y las posiciones reales no viajan.
//! - **Los mutantes.** Un juguete -`WorkAir`, el circuito canonico de winterfell, con las 42
//!   consultas y el blowup de produccion- con m = 43 deja los trozos sin cota y el test lo nombra;
//!   con T = 32, las columnas. Y la prueba con m = 43 sigue verificando: m decide la ocultacion,
//!   no la solidez (D-H).
//!
//! Lo que NO hace: no demuestra nada, eso es A2; no mide las posiciones reales ni `z`; no toca la
//! promesa de `SECURITY.md` §3.bis ni la D-A: hasta A4 la casa promete cero literales, no HVZK.

use std::cell::RefCell;
use std::collections::BTreeSet;
use std::path::Path;

use stark_experiment::{build_trace, WorkAir};
use winter_air::Marca;
use winter_prover::Ocultacion;
use winterfell::crypto::{hashers::Blake3_256, DefaultRandomCoin, MerkleTree};
use winterfell::math::fields::f128::BaseElement as Base128;
use winterfell::math::fields::f64::BaseElement;
use winterfell::math::fields::QuadExtension;
use winterfell::math::{FieldElement, StarkField};
use winterfell::matrix::ColMatrix;
use winterfell::{
    verify, AcceptableOptions, AuxRandElements, BatchingMethod, CompositionPoly,
    CompositionPolyTrace, ConstraintCompositionCoefficients, DefaultConstraintCommitment,
    DefaultConstraintEvaluator, DefaultTraceLde, FieldExtension, PartitionOptions, Proof,
    ProofOptions, Prover, StarkDomain, Trace, TraceInfo, TracePolyTable, TraceTable,
};

use crate::instrumento_revela::{self, FILAS, SOLO_DE_TESTS};
use crate::proof_options;

/// El campo de produccion con su extension cuadratica: donde viven `z`, `z·g` y los trozos.
type E = QuadExtension<BaseElement>;

/// Lo que DEEP y FRI dejan pasar de cada objeto ademas de lo que se ve: una direccion (la
/// cabecera, y A2, L3).
pub(crate) const DIRECCION_RESIDUAL: usize = 1;

// ---- las pruebas que pasan por `asierta`, anotadas en el hilo que las produce ----

thread_local! {
    static ANOTADAS: RefCell<Vec<(String, Vec<u8>)>> = const { RefCell::new(Vec::new()) };
}

/// `asierta` anota aqui cada prueba que cuenta, con su fila. Es por hilo: el test de la cota
/// produce las pruebas en el suyo y solo lee las suyas, aunque el arnes corra a la vez los tests
/// de `instrumento_revela` en otros hilos.
pub(crate) fn anota(fila: &str, pr: &[u8]) {
    ANOTADAS.with(|a| a.borrow_mut().push((fila.to_string(), pr.to_vec())));
}

fn recoge() -> Vec<(String, Vec<u8>)> {
    ANOTADAS.with(|a| a.borrow_mut().drain(..).collect())
}

// ---- la geometria de una prueba oculta, leida de sus bytes ----

/// Lo que una prueba oculta dice de si misma.
#[derive(Debug, Clone)]
pub(crate) struct Geometria {
    /// Las posiciones unicas que abre (`num_unique_queries`).
    pub q: usize,
    /// Las consultas que piden sus opciones: el tope de `q`.
    pub consultas: usize,
    /// Las filas reales: la traza oculta mide 2T.
    pub t: usize,
    /// El grado de los aleatorizadores del cociente, leido de la marca.
    pub m: usize,
    /// Las columnas de la traza principal, con la aleatoria.
    pub ancho: usize,
    /// Las del tramo auxiliar.
    pub ancho_aux: usize,
    /// Los trozos del cociente.
    pub trozos: usize,
    /// Las capas de FRI, y los elementos de su resto, que viajan enteros.
    pub capas_fri: usize,
    pub resto: usize,
    /// Los bytes de la prueba.
    pub bytes: usize,
}

/// Lee la geometria de unos bytes de prueba. `F` es el elemento con el que viajan `z`, los trozos
/// y el resto: el campo de produccion con su extension para las pruebas de la casa.
pub(crate) fn geometria<F: FieldElement>(nombre: &str, pr: &[u8]) -> Result<Geometria, String> {
    let p = Proof::from_bytes(pr).map_err(|e| format!("{nombre}: no se lee: {e:?}"))?;
    let info = p.trace_info();
    let m = match Marca::leer(info.meta()) {
        Ok(Some(marca)) => marca.m,
        Ok(None) => return Err(format!("{nombre}: sin marca: no es una prueba oculta")),
        Err(e) => return Err(format!("{nombre}: un meta que no es la marca: {e:?}")),
    };
    let largo = info.length();
    if largo % 2 != 0 || largo < 16 {
        return Err(format!("{nombre}: traza de {largo} filas: no se parte en dos"));
    }
    let (ancho, ancho_aux) = (info.main_trace_width(), info.aux_segment_width());
    // los trozos: el marco OOD lleva [2 filas][N trozos] y solo un N lo lee entero y sin resto
    let trozos = (1..=256usize)
        .find(|&n| p.ood_frame.clone().parse::<F>(ancho, ancho_aux, n).is_ok())
        .ok_or_else(|| format!("{nombre}: el marco OOD no se lee con ningun numero de trozos"))?;
    Ok(Geometria {
        q: p.num_unique_queries as usize,
        consultas: p.options().num_queries(),
        t: largo / 2,
        m,
        ancho,
        ancho_aux,
        trozos,
        capas_fri: p.fri_proof.num_layers(),
        resto: p.fri_proof.num_remainder_elements::<F>(),
        bytes: pr.len(),
    })
}

/// Las aberturas que la cota cuenta de cada objeto: las vistas y, con la residual, las que un
/// simulador tiene que enmascarar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Aberturas {
    pub vistas: usize,
    pub a_enmascarar: usize,
}

/// La cota, sobre la geometria: por columna frente a T, por trozo frente a m. Devuelve las
/// aberturas contadas, o cada desigualdad que no se cumple, con su nombre.
pub(crate) fn asierta_cota(nombre: &str, g: &Geometria) -> Result<Aberturas, String> {
    let vistas = g.q + 2;
    let a_enmascarar = vistas + DIRECCION_RESIDUAL;
    let mut mal = Vec::new();
    if g.q < 1 || g.q > g.consultas {
        mal.push(format!("q = {}, fuera de 1..={}", g.q, g.consultas));
    }
    if g.m >= 2 * g.t {
        mal.push(format!("m = {} no cabe en una traza de {} filas: el verificador la rechaza", g.m, 2 * g.t));
    }
    if a_enmascarar > g.t {
        mal.push(format!(
            "columnas: {vistas} aberturas vistas + {DIRECCION_RESIDUAL} residual = {a_enmascarar} \
             frente a T = {} filas aleatorias",
            g.t
        ));
    }
    if a_enmascarar > g.m {
        mal.push(format!(
            "trozos: {vistas} aberturas vistas + {DIRECCION_RESIDUAL} residual = {a_enmascarar} \
             frente a m = {}",
            g.m
        ));
    }
    if mal.is_empty() {
        Ok(Aberturas { vistas, a_enmascarar })
    } else {
        Err(format!("{nombre}: {}", mal.join("; ")))
    }
}

// ---- el rango: Lagrange de las filas aleatorias y Vandermonde de los trozos ----

/// El rango de una matriz sobre `E`, por eliminacion.
fn rango(mut a: Vec<Vec<E>>) -> usize {
    let filas = a.len();
    if filas == 0 {
        return 0;
    }
    let cols = a[0].len();
    let mut r = 0;
    for c in 0..cols {
        if r == filas {
            break;
        }
        let Some(piv) = (r..filas).find(|&i| a[i][c] != E::ZERO) else {
            continue;
        };
        a.swap(r, piv);
        let inv = a[r][c].inv();
        let fila_r: Vec<E> = a[r].iter().map(|&x| x * inv).collect();
        a[r] = fila_r.clone();
        for (i, fila) in a.iter_mut().enumerate() {
            if i != r && fila[c] != E::ZERO {
                let f = fila[c];
                for (x, y) in fila.iter_mut().zip(fila_r.iter()) {
                    *x -= f * *y;
                }
            }
        }
        r += 1;
    }
    r
}

/// Los puntos abiertos que el test elige para la geometria (q, T): `q` posiciones distintas del
/// dominio LDE de la traza oculta (el coset del `domain_offset` de las opciones, fuera del dominio
/// de la traza), `z` y
/// `z·g` en la extension, y la raiz del factor lineal de la direccion residual, (z + z·g)/2.
fn puntos(q: usize, t: usize) -> Vec<E> {
    let largo = 2 * t;
    let lde = largo * proof_options().blowup_factor();
    let omega = BaseElement::get_root_of_unity(lde.trailing_zeros());
    let g = BaseElement::get_root_of_unity(largo.trailing_zeros());
    let mut pts: Vec<E> = (0..q)
        .map(|j| {
            // un multiplicador impar es una biyeccion modulo una potencia de dos
            let pos = (j.wrapping_mul(0x9E37_79B1)) % lde;
            E::from(proof_options().domain_offset::<BaseElement>() * omega.exp(pos as u64))
        })
        .collect();
    let z = E::new(BaseElement::new(0x2545_F491_4F6C_DD1D), BaseElement::new(0x1234_5678_9ABC_DEF1));
    let zg = z * E::from(g);
    let x0 = (z + zg) * E::from(BaseElement::new(2)).inv();
    pts.extend([z, zg, x0]);
    let distintos: BTreeSet<[u8; 16]> = pts.iter().map(|e| elemento_a_bytes(*e)).collect();
    assert_eq!(distintos.len(), pts.len(), "dos puntos elegidos coinciden");
    pts
}

/// Los dieciseis bytes de un elemento de la extension, para distinguir puntos.
fn elemento_a_bytes(e: E) -> [u8; 16] {
    let partes = e.base_element(0).as_int().to_le_bytes();
    let mut b = [0u8; 16];
    b[..8].copy_from_slice(&partes);
    b[8..].copy_from_slice(&e.base_element(1).as_int().to_le_bytes());
    b
}

/// `L_r(y)` para cada punto `y` y cada fila aleatoria `r` de `T..2T`: `Z_H(y)·h_r / (2T·(y − h_r))`,
/// con `Z_H(y) = y^{2T} − 1` y `h_r = g^r`. Una fila por punto, una columna por fila aleatoria.
fn matriz_de_lagrange(pts: &[E], t: usize) -> Vec<Vec<E>> {
    let largo = 2 * t;
    let g = BaseElement::get_root_of_unity(largo.trailing_zeros());
    let dos_t = E::from(BaseElement::new(largo as u64));
    let h: Vec<E> = (t..largo).map(|r| E::from(g.exp(r as u64))).collect();
    pts.iter()
        .map(|&y| {
            let zh = y.exp(largo as u64) - E::ONE;
            h.iter().map(|&hr| zh * hr * (dos_t * (y - hr)).inv()).collect()
        })
        .collect()
}

/// `y^k` para cada punto `y` y cada grado `k < m`: lo que un aleatorizador de grado < m vale en
/// los puntos abiertos.
fn matriz_de_vandermonde(pts: &[E], m: usize) -> Vec<Vec<E>> {
    pts.iter()
        .map(|&y| {
            let mut fila = Vec::with_capacity(m);
            let mut p = E::ONE;
            for _ in 0..m {
                fila.push(p);
                p *= y;
            }
            fila
        })
        .collect()
}

/// Las dos matrices de la geometria (q, T, m), de rango pleno: tantas filas independientes como
/// puntos abiertos, con la residual. Si no, dice cual y con que rango.
pub(crate) fn rango_pleno(nombre: &str, g: &Geometria) -> Result<(), String> {
    let pts = puntos(g.q, g.t);
    let n = pts.len();
    let mut mal = Vec::new();
    let rl = rango(matriz_de_lagrange(&pts, g.t));
    if rl != n {
        mal.push(format!(
            "columnas: la matriz de Lagrange de las T = {} filas aleatorias en los {n} puntos tiene \
             rango {rl}",
            g.t
        ));
    }
    let rv = rango(matriz_de_vandermonde(&pts, g.m));
    if rv != n {
        mal.push(format!(
            "trozos: la matriz de Vandermonde de grado < m = {} en los {n} puntos tiene rango {rv}",
            g.m
        ));
    }
    if mal.is_empty() {
        Ok(())
    } else {
        Err(format!("{nombre}: {}", mal.join("; ")))
    }
}

// ---- el juguete: WorkAir con la ocultacion que el test le da ----

type Blake3W = Blake3_256<Base128>;

/// Las opciones del juguete: las consultas, el blowup, el plegado y el resto de produccion; sin
/// molienda, que no cambia lo que se abre, y sin extension, porque `WorkAir` vive en f128.
fn opciones_juguete() -> ProofOptions {
    ProofOptions::new(
        proof_options().num_queries(),
        proof_options().blowup_factor(),
        0,
        FieldExtension::None,
        proof_options().to_fri_options().folding_factor(),
        proof_options().to_fri_options().remainder_max_degree(),
        BatchingMethod::Linear,
        BatchingMethod::Linear,
    )
}

/// Una semilla de 32 bytes reproducible: `n` en los ocho primeros y el resto fijo (§652).
const fn s32(n: u64) -> [u8; 32] {
    let b = n.to_le_bytes();
    let mut s = [0xa1; 32];
    let mut i = 0;
    while i < 8 {
        s[i] = b[i];
        i += 1;
    }
    s
}

fn ocultacion_juguete(m: usize) -> Ocultacion {
    Ocultacion { m, semilla_filas: s32(0xc07a_0001), semilla_cociente: s32(0xc07a_0002) }
}

struct ProbadorJuguete {
    opciones: ProofOptions,
    ocultacion: Ocultacion,
}

impl Prover for ProbadorJuguete {
    type BaseField = Base128;
    type Air = WorkAir;
    type Trace = TraceTable<Base128>;
    type HashFn = Blake3W;
    type VC = MerkleTree<Blake3W>;
    type RandomCoin = DefaultRandomCoin<Blake3W>;
    type TraceLde<F: FieldElement<BaseField = Self::BaseField>> =
        DefaultTraceLde<F, Self::HashFn, Self::VC>;
    type ConstraintEvaluator<'a, F: FieldElement<BaseField = Self::BaseField>> =
        DefaultConstraintEvaluator<'a, Self::Air, F>;
    type ConstraintCommitment<F: FieldElement<BaseField = Self::BaseField>> =
        DefaultConstraintCommitment<F, Self::HashFn, Self::VC>;

    fn get_pub_inputs(&self, trace: &Self::Trace) -> Base128 {
        trace.get(0, trace.length() - 1)
    }

    fn options(&self) -> &ProofOptions {
        &self.opciones
    }

    fn ocultacion(&self) -> Option<Ocultacion> {
        Some(self.ocultacion)
    }

    fn new_trace_lde<F: FieldElement<BaseField = Self::BaseField>>(
        &self,
        trace_info: &TraceInfo,
        main_trace: &ColMatrix<Self::BaseField>,
        domain: &StarkDomain<Self::BaseField>,
        partition_option: PartitionOptions,
    ) -> (Self::TraceLde<F>, TracePolyTable<F>) {
        DefaultTraceLde::new(trace_info, main_trace, domain, partition_option)
    }

    fn new_evaluator<'a, F: FieldElement<BaseField = Self::BaseField>>(
        &self,
        air: &'a Self::Air,
        aux_rand_elements: Option<AuxRandElements<F>>,
        composition_coefficients: ConstraintCompositionCoefficients<F>,
    ) -> Self::ConstraintEvaluator<'a, F> {
        DefaultConstraintEvaluator::new(air, aux_rand_elements, composition_coefficients)
    }

    fn build_constraint_commitment<F: FieldElement<BaseField = Self::BaseField>>(
        &self,
        composition_poly_trace: CompositionPolyTrace<F>,
        num_constraint_composition_columns: usize,
        domain: &StarkDomain<Self::BaseField>,
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

/// El resultado publico de `WorkAir` con start 3 y n filas.
fn resultado(n: usize) -> Base128 {
    let mut x = Base128::new(3);
    for _ in 0..(n - 1) {
        x = x * x * x + Base128::new(42);
    }
    x
}

/// Los bytes de una prueba oculta de `WorkAir` con `n` filas reales (T = n) y el m que se le da.
fn prueba_juguete(n: usize, m: usize) -> Vec<u8> {
    let probador = ProbadorJuguete { opciones: opciones_juguete(), ocultacion: ocultacion_juguete(m) };
    probador.prove(build_trace(Base128::new(3), n)).expect("probar WorkAir").to_bytes()
}

/// Verifica unos bytes como prueba de `WorkAir` con `n` filas reales.
fn verifica_juguete(bytes: &[u8], n: usize) -> Result<(), String> {
    let proof = Proof::from_bytes(bytes).map_err(|e| format!("leer: {e:?}"))?;
    let acc = AcceptableOptions::OptionSet(vec![opciones_juguete()]);
    verify::<WorkAir, Blake3W, DefaultRandomCoin<Blake3W>, MerkleTree<Blake3W>>(
        proof,
        resultado(n),
        &acc,
    )
    .map_err(|e| format!("{e:?}"))
}

// ---- el censo de los probadores que ocultan ----

/// Cuantos `.rs` de `crates/stark-experiment/src` encienden la ocultacion de produccion, con la
/// regla del censo de las bandas (§842, §713): el fichero llama a `crate::ocultacion_encendida(`,
/// y no es uno de los modulos de solo tests de `SOLO_DE_TESTS`, que ocultan sin ser probadores de
/// nadie. Uno por probador con fila (RFC-0009 D-Z, D-AE).
fn probadores_que_encienden(src: &Path) -> usize {
    let mut n = 0;
    for e in std::fs::read_dir(src).expect("crates/stark-experiment/src") {
        let p = e.expect("entrada").path();
        let nombre = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if !nombre.ends_with(".rs") || SOLO_DE_TESTS.contains(&nombre.trim_end_matches(".rs")) {
            continue;
        }
        let texto = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
        if texto.contains("crate::ocultacion_encendida(") {
            n += 1;
        }
    }
    n
}

/// La AIR que nombra una fila de `asierta`: lo que va entre el ultimo parentesis.
fn air_de(fila: &str) -> String {
    fila.rsplit('(').next().unwrap_or("").trim_end_matches(')').to_string()
}

/// Una linea por prueba, con lo medido.
fn linea(fila: &str, g: &Geometria, ab: Option<Aberturas>) -> String {
    let a = ab.map(|a| format!("{} vistas, {} a enmascarar", a.vistas, a.a_enmascarar));
    format!(
        "[A1] {fila}: q {} de {} · T {} · m {} · {} + {} columnas · {} trozos · FRI {} capas y resto \
         de {} · {} B · aberturas por columna y por trozo: {}",
        g.q,
        g.consultas,
        g.t,
        g.m,
        g.ancho,
        g.ancho_aux,
        g.trozos,
        g.capas_fri,
        g.resto,
        g.bytes,
        a.unwrap_or_else(|| "sin cota".to_string())
    )
}

// ======================= LOS TESTS =======================

/// **La cota, sobre las 23 familias.** Produce en este hilo cada prueba del testigo de D-B -las
/// 21 filas de `instrumento_revela`, con sus variantes-, exige que las AIR que pasaron por
/// `asierta` sean exactamente las 23 de `FILAS` y que `FILAS` cuente lo que cuenta el fuente, un
/// probador por fichero que llama a `ocultacion_encendida()`; y en cada prueba exige la cota por
/// columna y por trozo y el rango pleno de sus dos matrices. Las del cable, ademas, llevan las
/// opciones de produccion. Un rojo junta todas las pruebas que fallan y las nombra.
#[test]
#[cfg_attr(debug_assertions, ignore = "winterfell valida grados en depuracion: juez release")]
fn la_cota_de_ocultacion_de_los_23_probadores_con_fila() {
    recoge();
    instrumento_revela::todas_las_filas();
    let pruebas = recoge();
    assert!(!pruebas.is_empty(), "ninguna prueba paso por asierta");

    // el censo: las AIR vistas son las de FILAS, y FILAS son los probadores que encienden
    let vistas: BTreeSet<String> = pruebas.iter().map(|(f, _)| air_de(f)).collect();
    let esperadas: BTreeSet<String> = FILAS.iter().map(|(a, _, _)| a.to_string()).collect();
    assert_eq!(vistas, esperadas, "las AIR que pasaron por asierta no son las de FILAS");
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("../stark-experiment/src");
    let encienden = probadores_que_encienden(&src);
    assert_eq!(
        encienden,
        FILAS.len(),
        "{encienden} ficheros de stark-experiment llaman a ocultacion_encendida() y FILAS tiene {}",
        FILAS.len()
    );

    let mut mal = Vec::new();
    for (fila, bytes) in &pruebas {
        let g = match geometria::<E>(fila, bytes) {
            Ok(g) => g,
            Err(e) => {
                mal.push(e);
                continue;
            }
        };
        let del_cable = FILAS.iter().any(|(a, _, cable)| *a == air_de(fila) && *cable);
        if del_cable {
            let p = Proof::from_bytes(bytes).expect("la prueba se deserializa");
            if p.options() != &proof_options() {
                mal.push(format!("{fila}: no lleva las opciones de produccion"));
            }
        }
        let cota = asierta_cota(fila, &g);
        println!("{}", linea(fila, &g, cota.as_ref().ok().copied()));
        if let Err(e) = cota {
            mal.push(e);
        }
        if let Err(e) = rango_pleno(fila, &g) {
            mal.push(e);
        }
    }
    println!("[A1] {} pruebas de {} AIR, {} probadores que encienden", pruebas.len(), vistas.len(), encienden);
    assert!(mal.is_empty(), "la cota de ocultacion no se cumple:\n{}", mal.join("\n"));
}

/// **El juguete con la geometria de produccion cumple la cota, y verifica.** `WorkAir` con 64
/// filas reales y m = 64, las consultas y el blowup de produccion: 45 a enmascarar frente a T = 64
/// y m = 64, rango pleno, y el verificador la acepta.
#[test]
fn el_juguete_con_la_geometria_de_produccion_cumple_la_cota_y_verifica() {
    let bytes = prueba_juguete(64, 64);
    verifica_juguete(&bytes, 64).expect("la prueba del juguete verifica");
    let g = geometria::<Base128>("juguete", &bytes).expect("la geometria se lee");
    assert_eq!((g.t, g.m, g.ancho), (64, 64, 2), "la geometria del juguete");
    assert!(g.trozos >= 2, "el cociente oculto va en dos trozos al menos: {}", g.trozos);
    let ab = asierta_cota("juguete", &g).expect("la cota se cumple");
    println!("{}", linea("juguete (WorkAir)", &g, Some(ab)));
    assert_eq!(ab.a_enmascarar, g.q + 2 + DIRECCION_RESIDUAL);
    rango_pleno("juguete", &g).expect("rango pleno");
}

/// **Mutante 1: m = 43 deja los trozos sin cota, y el test lo nombra.** La misma prueba sigue
/// verificando: m decide la ocultacion, no la solidez (RFC-0009 D-H).
#[test]
fn el_mutante_m_43_deja_los_trozos_sin_cota_y_sigue_verificando() {
    let bytes = prueba_juguete(64, 43);
    verifica_juguete(&bytes, 64).expect("m = 43 no toca la solidez: verifica");
    let g = geometria::<Base128>("m 43", &bytes).expect("la geometria se lee");
    assert_eq!(g.m, 43, "la marca lleva m = 43");
    // El mutante tiene dientes solo si la prueba abre 41 o 42 posiciones unicas: con 40, 43 a
    // enmascarar frente a m = 43 cumple. El juguete es determinista (sin sal, semillas fijas).
    assert!(g.q >= 41, "el juguete abre {} posiciones unicas y m = 43 no caeria", g.q);
    let e = asierta_cota("m 43", &g).expect_err("los trozos no tienen cota con m = 43");
    assert!(e.contains("trozos:") && e.contains("frente a m = 43"), "{e}");
    assert!(!e.contains("columnas:"), "las columnas siguen con cota (T = 64): {e}");
    let r = rango_pleno("m 43", &g).expect_err("la Vandermonde de grado < 43 no da 45 filas");
    assert!(r.contains("trozos:") && r.contains("rango 43"), "{r}");
}

/// **Mutante 2: T = 32 deja las columnas sin cota, y el test lo nombra.** Con m = 32, para que
/// quepa en la traza de 64 filas; los trozos caen tambien, por el mismo m.
#[test]
fn el_mutante_t_32_deja_las_columnas_sin_cota() {
    let bytes = prueba_juguete(32, 32);
    verifica_juguete(&bytes, 32).expect("verifica");
    let g = geometria::<Base128>("T 32", &bytes).expect("la geometria se lee");
    assert_eq!((g.t, g.m), (32, 32));
    let e = asierta_cota("T 32", &g).expect_err("las columnas no tienen cota con T = 32");
    assert!(e.contains("columnas:") && e.contains("frente a T = 32"), "{e}");
    let r = rango_pleno("T 32", &g).expect_err("la Lagrange de 32 filas no da 45");
    assert!(r.contains("columnas:") && r.contains("rango 32"), "{r}");
}

/// **El rango es pleno con puntos distintos y cae con uno repetido**: la prueba de vida de la
/// eliminacion, sobre la geometria de produccion mas corta (q 42, T 64, m 64).
#[test]
fn el_rango_es_pleno_con_puntos_distintos_y_cae_con_uno_repetido() {
    let pts = puntos(42, 64);
    assert_eq!(pts.len(), 45);
    assert_eq!(rango(matriz_de_lagrange(&pts, 64)), 45);
    assert_eq!(rango(matriz_de_vandermonde(&pts, 64)), 45);
    let mut repetido = pts.clone();
    repetido[7] = repetido[3];
    assert_eq!(rango(matriz_de_lagrange(&repetido, 64)), 44);
    assert_eq!(rango(matriz_de_vandermonde(&repetido, 64)), 44);
    // y con mas puntos que filas aleatorias, el rango es el de las filas
    assert_eq!(rango(matriz_de_lagrange(&pts, 32)), 32);
}
