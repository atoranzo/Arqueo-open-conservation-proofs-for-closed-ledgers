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
//! - **La direccion residual, que no lo era (⚠️ §897).** DEEP suma cada columna y cada trozo con
//!   su coeficiente, y FRI abre combinaciones de ese polinomio en sus capas y entrega su resto
//!   entero. La columna aleatoria entra en DEEP a traves del factor lineal `2x − z − z·g`, que se
//!   anula en `x0 = (z + z·g)/2`; hasta el §897 este modulo contaba por eso una abertura mas por
//!   objeto, `DIRECCION_RESIDUAL`, «lo que A2 tiene que demostrar». A2 lo demostro al reves: el
//!   valor de DEEP en `x0` se calcula del marco OOD, es publico, y no es una abertura que
//!   enmascarar ni una fuga (`doc/ocultacion-hvzk.md`, L3). El modulo ya no la cuenta. Lo que
//!   FRI abre de DEEP mas alla de las posiciones -las otras siete evaluaciones de cada coset de
//!   plegado, las capas y el resto- es la condicion R3 de ese documento, que mide `instrumento_r3`
//!   desde el §898, por geometria y dentro del test de las 23 familias.
//! - **La cota, por objeto.** Por columna principal: las `q` posiciones viven en F y `z`, `z·g`
//!   en E, asi que lo abierto son `q + 4` dimensiones sobre F frente a las T filas aleatorias
//!   que el envoltorio `Oculta<A>` pone detras de las reales (RFC-0009 D-G); ⚠️ §897: hasta aqui
//!   decia `q + 2 + 1 <= T`, y `rango_pleno` ya exigia las `q + 4`. Por trozo del cociente:
//!   `q + 2` puntos de E frente a los m coeficientes de los aleatorizadores de `segmentar`, que
//!   viven en E; m viaja en la marca (D-S); ⚠️ §897: decia `q + 2 + 1 <= m`. Los N trozos llevan
//!   N − 1 aleatorizadores que se cancelan en la suma, y la relacion que se cancela es publica en
//!   cada punto: la cota es por trozo y no depende de N.
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
//!
//! ⚠️ **§897 (A2): la cuenta de arriba es la de los puntos que se VEN, y no basta.** Los trozos
//! del cociente abren en cada consulta `x_j` la suma `Σ_k x_j^{ks}·h'_k(x_j) = C(x_j)`, que
//! depende de la fila siguiente, `T(g·x_j)`; y el marco OOD de los trozos en `z·g` revela
//! `C(z·g)`, que toca `T(g²·z)`. Los puntos que las filas aleatorias tienen que enmascarar por
//! columna son `2q + 3`, y como las filas aleatorias de la traza principal viven en F y los
//! puntos fuera del dominio en E, son `2q + 6` dimensiones sobre F: 90 con q = 42. Desde el §897
//! el instrumento mide tambien esa cuenta -[`Aberturas::tocados`]- y el rango SOBRE F de la matriz
//! de Lagrange en el conjunto tocado, y una tabla declarada por AIR, [`INDEPENDIENTE_DE_LA_AIR`],
//! dice que familias la cumplen (T >= 256) y cuales no (las cinco con T = 64); un cambio en
//! cualquier sentido es rojo. El argumento, con sus lemas, lo que mide cada test de aqui y lo que
//! no cubre, es `doc/ocultacion-hvzk.md`. §900 (la 134): el mismo test sondea las 23 AIR —que columnas
//! entran linealmente con coeficiente publico— contra el censo declarado de `instrumento_mlin` y mide
//! R3 con M_lin en la solvencia, contra lo clavado.

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

/// §897: las familias que cumplen la cota INDEPENDIENTE de la AIR -las filas aleatorias cubren,
/// sobre F, los `2q + 6` puntos que la prueba toca por columna, [`Aberturas::tocados`]- y las que
/// no. Lo que mide el test tiene que ser esto, AIR a AIR; una T o una q que lo cambie es rojo.
/// Las cinco de T = 64 no lo cumplen: para ellas el argumento de A2 necesita su lema L6, que
/// depende de la AIR, o mas filas aleatorias (la 131 de `BACKLOG.md`).
pub(crate) const INDEPENDIENTE_DE_LA_AIR: &[(&str, bool)] = &[
    ("SendAir", true),
    ("SendV2Air", true),
    ("ClaimAir", true),
    ("ClaimAirV2", true),
    ("PrendaAir", true),
    ("MintPendingClimbAir", true),
    ("NullifierThresholdAir", false),
    ("ThresholdAir", false),
    ("AuditAir", true),
    ("BurnAir", true),
    ("SolvencyAir", true),
    ("DoubleEntryAir", true),
    ("BandaAir", true),
    ("EdadAir", false),
    ("CobroPendienteAir", true),
    ("PagoEnCursoAir", true),
    ("MerkleAir", true),
    ("RefundAir", false),
    ("RefundAirV2", false),
    ("CreditClimbAir", true),
    ("MintClimbAir", true),
    ("FrozenClimbAir", true),
    ("RecoveryClimbAir", true),
];

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

pub(crate) fn recoge() -> Vec<(String, Vec<u8>)> {
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

/// Las aberturas que la cota cuenta de cada objeto: los puntos que se ven y lo que miden en el
/// cuerpo de cada aleatorizador. ⚠️ §897: hasta aqui llevaba `a_enmascarar`, las vistas mas una
/// «direccion residual» que no era una abertura.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Aberturas {
    /// Los puntos abiertos por objeto: las `q` posiciones, `z` y `z·g`.
    pub vistas: usize,
    /// §897: lo abierto por columna principal, en dimensiones sobre F: `q` posiciones de F y dos
    /// puntos de E, `q + 4`; es lo que las T filas aleatorias, que viven en F, tienen que cubrir.
    pub dims_columna: usize,
    /// §897: lo abierto por trozo, en dimensiones sobre E: `q + 2`, frente a los m coeficientes
    /// de cada aleatorizador, que viven en E.
    pub dims_trozo: usize,
    /// §897: lo que la prueba TOCA por columna principal, en dimensiones sobre F: las `q`
    /// posiciones, sus `q` siguientes (`g·x_j`, por `Q(x_j)`) y los tres puntos fuera del dominio
    /// (`z`, `z·g` y `g²·z`, por `Q(z·g)`), a dos dimensiones cada uno: `2q + 6`.
    pub tocados: usize,
    /// §897: si las T filas aleatorias cubren los tocados: la cota independiente de la AIR.
    pub independiente: bool,
}

/// La cota, sobre la geometria: por columna frente a T, por trozo frente a m. Devuelve las
/// aberturas contadas, o cada desigualdad que no se cumple, con su nombre.
pub(crate) fn asierta_cota(nombre: &str, g: &Geometria) -> Result<Aberturas, String> {
    let vistas = g.q + 2;
    // §897: q posiciones de F y dos puntos de E son q + 4 dimensiones sobre F por columna, y
    // q + 2 puntos de E por trozo
    let dims_columna = g.q + 2 * 2;
    let dims_trozo = g.q + 2;
    let mut mal = Vec::new();
    if g.q < 1 || g.q > g.consultas {
        mal.push(format!("q = {}, fuera de 1..={}", g.q, g.consultas));
    }
    if g.m >= 2 * g.t {
        mal.push(format!("m = {} no cabe en una traza de {} filas: el verificador la rechaza", g.m, 2 * g.t));
    }
    if dims_columna > g.t {
        mal.push(format!(
            "columnas: {vistas} puntos abiertos, {dims_columna} dimensiones sobre F, frente a T = {} \
             filas aleatorias",
            g.t
        ));
    }
    if dims_trozo > g.m {
        mal.push(format!(
            "trozos: {vistas} puntos abiertos, {dims_trozo} dimensiones sobre E, frente a m = {}",
            g.m
        ));
    }
    let tocados = 2 * g.q + 6;
    if mal.is_empty() {
        Ok(Aberturas { vistas, dims_columna, dims_trozo, tocados, independiente: tocados <= g.t })
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

/// El rango SOBRE F de una matriz con filas en E: cada fila de E son sus dos filas de
/// coordenadas sobre F. Es el rango que vale para las columnas principales, cuyas filas
/// aleatorias viven en F y se evaluan en puntos de E (§897).
fn rango_f(a: &[Vec<E>]) -> usize {
    let mut b: Vec<Vec<BaseElement>> = Vec::with_capacity(2 * a.len());
    for fila in a {
        b.push(fila.iter().map(|e| e.base_element(0)).collect());
        b.push(fila.iter().map(|e| e.base_element(1)).collect());
    }
    let filas = b.len();
    if filas == 0 {
        return 0;
    }
    let cols = b[0].len();
    let mut r = 0;
    for c in 0..cols {
        if r == filas {
            break;
        }
        let Some(piv) = (r..filas).find(|&i| b[i][c] != BaseElement::ZERO) else {
            continue;
        };
        b.swap(r, piv);
        let inv = b[r][c].inv();
        let fila_r: Vec<BaseElement> = b[r].iter().map(|&x| x * inv).collect();
        b[r] = fila_r.clone();
        for (i, fila) in b.iter_mut().enumerate() {
            if i != r && fila[c] != BaseElement::ZERO {
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

/// Las `q` posiciones del dominio LDE de la traza oculta que el test elige (el coset del
/// `domain_offset` de las opciones, fuera del dominio de la traza) y los dos puntos fuera del
/// dominio, `z` y `z·g`: lo que se ABRE de cada objeto.
fn puntos_abiertos(q: usize, t: usize) -> Vec<E> {
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
    pts.push(z);
    pts.push(z * E::from(g));
    distintos(pts)
}

/// §897: lo que la prueba TOCA por columna: los abiertos, la fila siguiente de cada posicion
/// (`g·x_j`, que `Q(x_j)` lee) y `g²·z` (que `Q(z·g)` lee). Sin repetidos: una siguiente que
/// coincide con otra posicion abierta cuenta una vez.
fn puntos_tocados(q: usize, t: usize) -> Vec<E> {
    let largo = 2 * t;
    let g = E::from(BaseElement::get_root_of_unity(largo.trailing_zeros()));
    let abiertos = puntos_abiertos(q, t);
    let mut pts = abiertos.clone();
    for (i, &y) in abiertos.iter().enumerate() {
        if i < q {
            pts.push(y * g);
        }
    }
    let zg = abiertos[q + 1];
    pts.push(zg * g);
    distintos(pts)
}

/// Los puntos, sin repetidos y en su orden.
fn distintos(pts: Vec<E>) -> Vec<E> {
    let mut vistos: BTreeSet<[u8; 16]> = BTreeSet::new();
    pts.into_iter().filter(|e| vistos.insert(elemento_a_bytes(*e))).collect()
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

/// Las matrices de la geometria (q, T, m), de rango pleno sobre los puntos ABIERTOS: la de
/// Lagrange de las T filas aleatorias, sobre F (§897: antes sobre E, que no es el cuerpo de las
/// filas aleatorias de la traza principal), y la de Vandermonde de grado < m, sobre E, que es el
/// cuerpo de los aleatorizadores. Si no, dice cual y con que rango.
pub(crate) fn rango_pleno(nombre: &str, g: &Geometria) -> Result<(), String> {
    let pts = puntos_abiertos(g.q, g.t);
    let n_e = pts.len();
    // sobre F, cada punto de E son dos dimensiones y cada punto de F una (q + 2 * 2)
    let n_f = g.q + 2 * 2;
    let mut mal = Vec::new();
    let rl = rango_f(&matriz_de_lagrange(&pts, g.t));
    if rl != n_f {
        mal.push(format!(
            "columnas: la matriz de Lagrange de las T = {} filas aleatorias en los {n_e} puntos \
             abiertos tiene rango {rl} sobre F, y los puntos son {n_f} dimensiones",
            g.t
        ));
    }
    let rv = rango(matriz_de_vandermonde(&pts, g.m));
    if rv != n_e {
        mal.push(format!(
            "trozos: la matriz de Vandermonde de grado < m = {} en los {n_e} puntos tiene rango {rv}",
            g.m
        ));
    }
    if mal.is_empty() {
        Ok(())
    } else {
        Err(format!("{nombre}: {}", mal.join("; ")))
    }
}

/// §897: el rango sobre F de la matriz de Lagrange de las T filas aleatorias en los puntos
/// TOCADOS, y las dimensiones que esos puntos piden. Pleno cuando `independiente`; si no, el
/// rango es a lo sumo T y lo dice: es la medida de la 131 de `BACKLOG.md`.
pub(crate) fn rango_de_los_tocados(g: &Geometria) -> (usize, usize) {
    let pts = puntos_tocados(g.q, g.t);
    let dims = pts.iter().filter(|e| e.base_element(1) == BaseElement::ZERO).count()
        + 2 * pts.iter().filter(|e| e.base_element(1) != BaseElement::ZERO).count();
    (rango_f(&matriz_de_lagrange(&pts, g.t)), dims)
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
    Ocultacion { m, semilla_filas: s32(0xc07a_0001), semilla_cociente: s32(0xc07a_0002), columnas: 1 }
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
pub(crate) fn air_de(fila: &str) -> String {
    fila.rsplit('(').next().unwrap_or("").trim_end_matches(')').to_string()
}

/// Una linea por prueba, con lo medido.
fn linea(fila: &str, g: &Geometria, ab: Option<Aberturas>) -> String {
    let a = ab.map(|a| {
        format!(
            "{} puntos abiertos: {} dimensiones sobre F por columna frente a T = {}, {} sobre E por \
             trozo frente a m = {}; tocados {} sobre F frente a T: {}",
            a.vistas,
            a.dims_columna,
            g.t,
            a.dims_trozo,
            g.m,
            a.tocados,
            if a.independiente { "cubre" } else { "NO cubre" }
        )
    });
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
/// opciones de produccion. Un rojo junta todas las pruebas que fallan y las nombra. §898 (A3):
/// y mide R3 (`instrumento_r3`) en cada geometria de produccion, contra sus cifras clavadas.
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
        match cota {
            Err(e) => mal.push(e),
            Ok(ab) => {
                // §897: lo medido tiene que ser lo declarado, AIR a AIR
                let air = air_de(fila);
                match INDEPENDIENTE_DE_LA_AIR.iter().find(|(a, _)| *a == air) {
                    None => mal.push(format!("{fila}: sin fila en INDEPENDIENTE_DE_LA_AIR")),
                    Some((_, declarado)) if *declarado != ab.independiente => mal.push(format!(
                        "{fila}: tocados {} frente a T = {}: {} y la tabla dice {}",
                        ab.tocados,
                        g.t,
                        if ab.independiente { "cubre" } else { "no cubre" },
                        if *declarado { "cubre" } else { "no cubre" }
                    )),
                    Some(_) => {}
                }
                let (rango, dims) = rango_de_los_tocados(&g);
                println!("[A1] {fila}: rango sobre F en los tocados {rango} de {dims} dimensiones");
                if ab.independiente && rango != dims {
                    mal.push(format!("{fila}: cubre y el rango en los tocados es {rango} de {dims}"));
                }
                if !ab.independiente && rango != g.t {
                    mal.push(format!("{fila}: no cubre y el rango en los tocados es {rango}, no T = {}", g.t));
                }
            }
        }
        if let Err(e) = rango_pleno(fila, &g) {
            mal.push(e);
        }
    }
    println!("[A1] {} pruebas de {} AIR, {} probadores que encienden", pruebas.len(), vistas.len(), encienden);
    assert!(mal.is_empty(), "la cota de ocultacion no se cumple:\n{}", mal.join("\n"));

    // §898 (A3): R3, por geometria canonica (q = 42), contra lo clavado; en ninguna se cumple
    let mut geometrias: Vec<Geometria> = Vec::new();
    for (fila, bytes) in &pruebas {
        let g = crate::instrumento_r3::canonica(&geometria::<E>(fila, bytes).expect("geometria"));
        if !geometrias.iter().any(|h| (h.t, h.trozos, h.capas_fri, h.resto) == (g.t, g.trozos, g.capas_fri, g.resto)) {
            geometrias.push(g);
        }
    }
    let mut mal = Vec::new();
    for g in &geometrias {
        let nombre = format!("T {} · {} trozos", g.t, g.trozos);
        let m = crate::instrumento_r3::mide(g, 1, false);
        println!("{}", crate::instrumento_r3::linea(&nombre, g, &m));
        match crate::instrumento_r3::clavado(g) {
            None => mal.push(format!("{nombre}: sin fila en R3_MEDIDO")),
            Some(clavado) => {
                let medido = (m.rango_k, m.rango_columna, m.rango_trozos, m.rango_mascaras);
                if medido != clavado {
                    mal.push(format!("{nombre}: medido {medido:?} y clavado {clavado:?}"));
                }
            }
        }
        if m.cumple() {
            mal.push(format!("{nombre}: R3 se cumple, y lo clavado dice que no"));
        }
        // §899 (la 133): la construccion de laboratorio, dos columnas aleatorias, cumple R3 en
        // esta geometria, con sus cifras clavadas
        let lab = crate::instrumento_r3::mide_con(g, 1, false, crate::instrumento_r3::LABORATORIO);
        println!("{} · laboratorio", crate::instrumento_r3::linea(&nombre, g, &lab));
        match crate::instrumento_r3::clavado_laboratorio(g) {
            None => mal.push(format!("{nombre}: sin fila en R3_MEDIDO_LABORATORIO")),
            Some(clavado) => {
                let medido = (lab.rango_k, lab.rango_columna, lab.rango_trozos, lab.rango_mascaras);
                if medido != clavado {
                    mal.push(format!("{nombre}, laboratorio: medido {medido:?} y clavado {clavado:?}"));
                }
            }
        }
        if !lab.cumple() {
            mal.push(format!("{nombre}: el laboratorio no cumple R3"));
        }
        if g.t == 64 {
            // la geometria barata, con las otras consultas que una prueba abre y otra semilla de
            // desafios: el veredicto no cambia (lo demas, en la exploracion del §898)
            for (q, semilla) in [(40usize, 1u64), (41, 1), (42, 2)] {
                let h = Geometria { q, ..g.clone() };
                let m = crate::instrumento_r3::mide(&h, semilla, false);
                println!("{} · semilla {semilla}", crate::instrumento_r3::linea(&nombre, &h, &m));
                if m.cumple() {
                    mal.push(format!("{nombre} con q = {q} y semilla {semilla}: R3 se cumple"));
                }
            }
        }
    }
    assert_eq!(geometrias.len(), crate::instrumento_r3::R3_MEDIDO.len(), "geometrias de produccion");
    assert!(mal.is_empty(), "R3 no es lo clavado:\n{}", mal.join("\n"));

    // §900 (la 134): el censo de la sonda, AIR a AIR, contra lo declarado en CENSO_134 -cuantas
    // columnas entran linealmente con coeficiente publico, cuantas con pendiente del testigo,
    // cuantas no linealmente- y el veredicto, coherente con el censo y con lo clavado; y M_lin
    // medida en la solvencia, la familia barata, contra lo clavado (las demas, en el test largo de
    // instrumento_mlin, clavadas tambien)
    use crate::instrumento_mlin::{self, Veredicto};
    let mut mal = Vec::new();
    let mut sondeadas: BTreeSet<String> = BTreeSet::new();
    for (fila, bytes) in &pruebas {
        let air = air_de(fila);
        let g = crate::instrumento_r3::canonica(&geometria::<E>(fila, bytes).expect("geometria"));
        if g.m != 64 || !sondeadas.insert(air.clone()) {
            continue;
        }
        let p = Proof::from_bytes(bytes).expect("la prueba se deserializa");
        let Some(sonda) = instrumento_mlin::sonda_de(&air, p.trace_info(), p.options()) else {
            mal.push(format!("{air}: sin sonda"));
            continue;
        };
        let censo = instrumento_mlin::sondea(&*sonda, 1);
        let (l, t, n) = censo.cuenta();
        let libres = instrumento_mlin::libres_de(&g);
        println!(
            "[134] {air}: T {} · {} reales · {l} lineales · {t} con pendiente del testigo · {n} no lineales · \
             {libres} libres por lineal",
            g.t,
            sonda.reales()
        );
        match instrumento_mlin::censo_declarado(&air) {
            None => mal.push(format!("{air}: sin fila en CENSO_134")),
            Some((dt_, r, dl, dt, dn, veredicto)) => {
                if (dt_, r, dl, dt, dn) != (g.t, sonda.reales(), l, t, n) {
                    mal.push(format!(
                        "{air}: censo medido (T {}, {}, {l}, {t}, {n}) y declarado (T {dt_}, {r}, {dl}, {dt}, {dn})",
                        g.t,
                        sonda.reales()
                    ));
                }
                let esperado = if libres == 0 {
                    Some(Veredicto::NoCabe)
                } else if l == 0 {
                    Some(Veredicto::SinLineales)
                } else {
                    match instrumento_mlin::medido(&air) {
                        Some(m) if m.0 == g.t && m.1 == g.trozos => {
                            Some(if m.5 == m.2 { Veredicto::Cumple } else { Veredicto::NoCumple })
                        },
                        Some(m) => {
                            mal.push(format!("{air}: la fila de MLIN_MEDIDO es de T {} y {} trozos", m.0, m.1));
                            None
                        },
                        None => {
                            mal.push(format!("{air}: con lineales y sin fila en MLIN_MEDIDO"));
                            None
                        },
                    }
                };
                if let Some(e) = esperado {
                    if veredicto != e {
                        mal.push(format!("{air}: el veredicto declarado es {veredicto:?} y lo medido dice {e:?}"));
                    }
                }
            },
        }
        if air == "SolvencyAir" {
            let m = instrumento_mlin::mide(&*sonda, &g, &censo.lineales(), 1, instrumento_mlin::TODAS);
            println!("{}", instrumento_mlin::linea(&air, &g, &m));
            let r3 = crate::instrumento_r3::clavado(&g).map(|c| c.3).expect("la solvencia tiene fila en R3_MEDIDO");
            if m.rango_sin_lineales != r3 {
                mal.push(format!("{air}: sin lineales da {} y R3_MEDIDO dice {r3}", m.rango_sin_lineales));
            }
            let medida = (g.t, g.trozos, m.rango_k, r3, m.rango_lineales, m.rango_mascaras);
            match instrumento_mlin::medido(&air) {
                None => mal.push(format!("{air}: sin fila en MLIN_MEDIDO: {medida:?}")),
                Some(c) if c != medida => mal.push(format!("{air}: medido {medida:?} y clavado {c:?}")),
                Some(_) => {},
            }
        }
    }
    assert_eq!(sondeadas.len(), FILAS.len(), "familias sondeadas");
    assert!(mal.is_empty(), "la 134 no es lo clavado:\n{}", mal.join("\n"));
}

/// **El juguete con la geometria de produccion cumple la cota, y verifica.** `WorkAir` con 128
/// filas reales (⚠️ §897: eran 64, que no cubre los 90 tocados) y m = 64, las consultas y el
/// blowup de produccion: 46 dimensiones sobre F frente a T = 128 y 44 sobre E frente a m = 64,
/// 90 tocados cubiertos, rango pleno, y el verificador la acepta.
#[test]
fn el_juguete_con_la_geometria_de_produccion_cumple_la_cota_y_verifica() {
    let bytes = prueba_juguete(128, 64);
    verifica_juguete(&bytes, 128).expect("la prueba del juguete verifica");
    let g = geometria::<Base128>("juguete", &bytes).expect("la geometria se lee");
    assert_eq!((g.t, g.m, g.ancho), (128, 64, 2), "la geometria del juguete");
    assert!(g.trozos >= 2, "el cociente oculto va en dos trozos al menos: {}", g.trozos);
    let ab = asierta_cota("juguete", &g).expect("la cota se cumple");
    println!("{}", linea("juguete (WorkAir)", &g, Some(ab)));
    assert_eq!((ab.vistas, ab.dims_columna, ab.dims_trozo), (g.q + 2, g.q + 4, g.q + 2));
    assert!(ab.independiente, "T = 128 cubre los {} tocados", ab.tocados);
    let (rango, dims) = rango_de_los_tocados(&g);
    assert_eq!(rango, dims, "rango pleno sobre F en los tocados");
    rango_pleno("juguete", &g).expect("rango pleno");
}

/// **Mutante 1: m = 43 deja los trozos sin cota, y el test lo nombra.** La misma prueba sigue
/// verificando: m decide la ocultacion, no la solidez (RFC-0009 D-H).
#[test]
fn el_mutante_m_43_deja_los_trozos_sin_cota_y_sigue_verificando() {
    let bytes = prueba_juguete(128, 43);
    verifica_juguete(&bytes, 128).expect("m = 43 no toca la solidez: verifica");
    let g = geometria::<Base128>("m 43", &bytes).expect("la geometria se lee");
    assert_eq!(g.m, 43, "la marca lleva m = 43");
    // §897: con la cuenta buena, q + 2 puntos de E frente a m, el mutante m = 43 tiene dientes
    // solo si la prueba abre las 42 posiciones unicas: con 41, 43 frente a 43 cumple. El juguete
    // es determinista (sin sal, semillas fijas) y abre 42; el mutante que cae con toda prueba de
    // produccion, que abre de 40 a 42, es m <= 41.
    assert_eq!(g.q, 42, "el juguete abre {} posiciones unicas y m = 43 no caeria", g.q);
    let e = asierta_cota("m 43", &g).expect_err("los trozos no tienen cota con m = 43");
    assert!(e.contains("trozos:") && e.contains("frente a m = 43"), "{e}");
    assert!(!e.contains("columnas:"), "las columnas siguen con cota (T = 128): {e}");
    let r = rango_pleno("m 43", &g).expect_err("la Vandermonde de grado < 43 no da 44 filas");
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
    let r = rango_pleno("T 32", &g).expect_err("la Lagrange de 32 filas no da 46 dimensiones");
    assert!(r.contains("columnas:") && r.contains("rango 32"), "{r}");
}

/// **El rango es pleno con puntos distintos y cae con uno repetido**: la prueba de vida de las
/// dos eliminaciones, sobre F y sobre E, con la geometria de produccion (q 42, m 64) en T = 128,
/// que cubre los tocados, y en T = 64, que no (§897).
#[test]
fn el_rango_es_pleno_con_puntos_distintos_y_cae_con_uno_repetido() {
    let abiertos = puntos_abiertos(42, 128);
    assert_eq!(abiertos.len(), 44);
    assert_eq!(rango_f(&matriz_de_lagrange(&abiertos, 128)), 46, "q + 2 puntos de E, sobre F");
    assert_eq!(rango(matriz_de_vandermonde(&abiertos, 64)), 44);
    let mut repetido = abiertos.clone();
    repetido[7] = repetido[3];
    assert_eq!(rango_f(&matriz_de_lagrange(&repetido, 128)), 45);
    assert_eq!(rango(matriz_de_vandermonde(&repetido, 64)), 43);
    // los tocados: 2q + 3 puntos, 2q + 6 dimensiones sobre F; T = 128 los cubre y T = 64 no
    let tocados = puntos_tocados(42, 128);
    assert_eq!(tocados.len(), 87, "sin repetidos entre las posiciones y sus siguientes");
    assert_eq!(rango_f(&matriz_de_lagrange(&tocados, 128)), 90);
    assert_eq!(rango_f(&matriz_de_lagrange(&puntos_tocados(42, 64), 64)), 64);
}
