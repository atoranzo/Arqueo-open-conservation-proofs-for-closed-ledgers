//! **A3 del HVZK (§898): la condicion R3 de `doc/ocultacion-hvzk.md`, medida por geometria.**
//!
//! A2 dejo una condicion suficiente sin medir: que las dos mascaras que no dependen del testigo
//! -la columna aleatoria, que entra en DEEP por `cc_R·Q[R]`, y los aleatorizadores del cociente,
//! que entran por `cc'_k·Q[x^s·r_k − r_{k−1}]`- cubran todo lo que FRI abre del polinomio DEEP
//! fuera de lo que las aperturas ya determinan (apartados 3 y 6 del documento). Este modulo la
//! mide. Para una geometria (q, T, m, N, capas, resto), leida de una prueba real por
//! `instrumento_cota`, y unos desafios que elige el test -posiciones, z, coeficientes de DEEP y
//! los α de FRI, de una moneda Blake3 sembrada con un numero-, construye:
//!
//! - **O**, lo que FRI abre de un polinomio D de grado ≤ L − 2: en cada capa, la hoja entera del
//!   coset de plegado de cada posicion (ocho valores), con el pliegue de winter-fri
//!   (`apply_drp`, con su `fold_positions`), y el resto, los coeficientes de la ultima capa
//!   interpolada; exactamente lo que el probador de FRI pone en la prueba para ese D;
//! - **K**, los D que valen 0 en las q posiciones y en x₀ = (z + z·g)/2: lo que las aperturas no
//!   determinan;
//! - **M_R**, la imagen en D de lo que queda libre de la columna aleatoria: `cc_R·ℓ·V·(x − z̄)(x − z̄g)·u`
//!   con u ∈ F[x] de grado < L − q − 4, un espacio sobre F, no sobre E;
//! - **M_r**, la de lo que queda libre de cada aleatorizador: `(cc'_k·x^s − cc'_{k+1})·V·ℓ·v` con
//!   v ∈ E[x] de grado < m − q − 2;
//!
//! y compara rangos sobre F: R3 se cumple si rango(O∘[M_R | M_r]) = rango(O|_K). Los rangos de
//! espacios grandes se miden sobre combinaciones aleatorias (Schwartz-Zippel sobre un campo de
//! 2^64): con mas combinaciones que filas tiene O, el rango es el del espacio entero salvo una
//! probabilidad del orden de 2^{−50}.
//!
//! Lo que NO hace: no mide los desafios reales de ninguna prueba (aqui no se rehacen de sus bytes;
//! los falsadores de `instrumento_falsadores_hvzk` si lo hacen, con el AIR delante); no cuenta las
//! filas aleatorias de las columnas reales, que en una AIR cuyas restricciones las toquen con grado
//! >= 2 entran en los trozos del cociente de forma no lineal (objecion 1 del §897) y en una AIR que
//! las toque solo linealmente y con coeficientes publicos son una mascara lineal mas, independiente
//! del testigo, que este instrumento NO mide (la 134 de `BACKLOG.md`: M_lin, por AIR); y no cambia
//! la promesa de la casa. El veredicto, con precision: R3 con M = {columna aleatoria,
//! aleatorizadores} no se cumple; que lo que FRI abre no sea simulable no lo dice este modulo.

use std::collections::BTreeSet;

use winter_fri::folding::{apply_drp, fold_positions};
use winterfell::crypto::{hashers::Blake3_256, DefaultRandomCoin, RandomCoin};
use winterfell::math::fields::f64::BaseElement;
use winterfell::math::fields::QuadExtension;
use winterfell::math::{fft, FieldElement, StarkField};

use crate::instrumento_cota::Geometria;
use crate::proof_options;

/// El campo de produccion con su extension cuadratica, donde vive D.
type E = QuadExtension<BaseElement>;
/// El factor de plegado de FRI en produccion.
const PLEGADO: usize = 8;

/// Lo que R3 mide de una geometria: los rangos, y el veredicto.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MedidaR3 {
    /// Filas de O sobre E: 8 por coset abierto en cada capa, mas el resto.
    pub filas: usize,
    /// rango sobre E de O, sobre D entero; solo cuando se pide (no hace falta para el veredicto).
    pub rango_o: Option<usize>,
    /// rango sobre F de O restringida a K (dos veces su rango sobre E).
    pub rango_k: usize,
    /// rango sobre F de O∘M_R, la columna aleatoria sola.
    pub rango_columna: usize,
    /// rango sobre F de O∘M_r, los aleatorizadores solos.
    pub rango_trozos: usize,
    /// rango sobre F de O∘[M_R | M_r], las dos juntas.
    pub rango_mascaras: usize,
}

impl MedidaR3 {
    /// R3: las mascaras cubren todo lo que FRI abre y las aperturas no determinan.
    pub fn cumple(&self) -> bool {
        self.rango_mascaras == self.rango_k
    }
}

/// Los desafios que el test elige: las q posiciones unicas, z, el coeficiente DEEP de la columna
/// aleatoria, los de los N trozos y un α por capa de FRI. De una moneda sembrada con `semilla`.
pub(crate) struct Desafios {
    pub posiciones: Vec<usize>,
    pub z: E,
    pub cc_columna: E,
    pub cc_trozos: Vec<E>,
    pub alfas: Vec<E>,
}

type Moneda = DefaultRandomCoin<Blake3_256<BaseElement>>;

pub(crate) fn desafios(g: &Geometria, semilla: u64) -> Desafios {
    let lde = 2 * g.t * proof_options().blowup_factor();
    let mut moneda = Moneda::new(&[BaseElement::new(semilla)]);
    let mut vistas = BTreeSet::new();
    let mut posiciones = Vec::with_capacity(g.q);
    for p in moneda.draw_integers(g.q, lde, 0).expect("posiciones") {
        if vistas.insert(p) {
            posiciones.push(p);
        }
    }
    while posiciones.len() < g.q {
        let p = (saca::<BaseElement>(&mut moneda).as_int() as usize) & (lde - 1);
        if vistas.insert(p) {
            posiciones.push(p);
        }
    }
    posiciones.sort_unstable();
    let z: E = saca(&mut moneda);
    let cc_columna: E = saca(&mut moneda);
    let cc_trozos = (0..g.trozos).map(|_| saca(&mut moneda)).collect();
    let alfas = (0..g.capas_fri).map(|_| saca(&mut moneda)).collect();
    Desafios { posiciones, z, cc_columna, cc_trozos, alfas }
}

fn saca<F: FieldElement<BaseField = BaseElement>>(moneda: &mut Moneda) -> F {
    moneda.draw::<F>().expect("la moneda no dio un elemento")
}

// ---- polinomios sobre E, en orden ascendente ----

fn mul(a: &[E], b: &[E]) -> Vec<E> {
    let mut r = vec![E::ZERO; a.len() + b.len() - 1];
    for (i, &x) in a.iter().enumerate() {
        if x == E::ZERO {
            continue;
        }
        for (j, &y) in b.iter().enumerate() {
            r[i + j] += x * y;
        }
    }
    r
}

/// Π (x − raiz).
fn de_raices(raices: &[E]) -> Vec<E> {
    let mut p = vec![E::ONE];
    for &r in raices {
        p = mul(&p, &[-r, E::ONE]);
    }
    p
}

fn con_largo(mut p: Vec<E>, largo: usize) -> Vec<E> {
    assert!(p.len() <= largo, "un polinomio de grado {} no cabe en {largo}", p.len() - 1);
    p.resize(largo, E::ZERO);
    p
}

/// El conjugado de E sobre F: el Frobenius, e^p.
fn conjugado(e: E) -> E {
    e.exp(BaseElement::MODULUS)
}

/// Cada elemento de E, sus dos coordenadas sobre F.
fn sobre_f(v: &[E]) -> Vec<BaseElement> {
    v.iter().flat_map(|e| [e.base_element(0), e.base_element(1)]).collect()
}

/// El rango de una matriz, por filas, sobre el cuerpo de sus elementos.
pub(crate) fn rango_de<F: FieldElement>(mut a: Vec<Vec<F>>) -> usize {
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
        let Some(piv) = (r..filas).find(|&i| a[i][c] != F::ZERO) else {
            continue;
        };
        a.swap(r, piv);
        let inv = a[r][c].inv();
        for x in a[r].iter_mut() {
            *x *= inv;
        }
        // solo hacia abajo: para el rango basta la forma escalonada
        let (hasta_r, despues) = a.split_at_mut(r + 1);
        let fila_r = &hasta_r[r];
        for fila in despues.iter_mut() {
            let f = fila[c];
            if f != F::ZERO {
                for (x, y) in fila[c..].iter_mut().zip(fila_r[c..].iter()) {
                    *x -= f * *y;
                }
            }
        }
        r += 1;
    }
    r
}

// ---- la mirilla: lo que FRI abre de un polinomio ----

/// La geometria de FRI de una prueba y los desafios, listos para observar polinomios.
struct Mirilla {
    largo: usize,
    lde: usize,
    capas: usize,
    resto: usize,
    posiciones: Vec<usize>,
    alfas: Vec<E>,
    twiddles: Vec<BaseElement>,
    inv_twiddles: Vec<BaseElement>,
    /// Las filas sobre E que devuelve `observa`.
    filas: usize,
}

impl Mirilla {
    fn nueva(g: &Geometria, d: &Desafios) -> Self {
        let largo = 2 * g.t;
        let blowup = proof_options().blowup_factor();
        let lde = largo * blowup;
        let tam_final = lde / PLEGADO.pow(g.capas_fri as u32);
        assert_eq!(tam_final / blowup, g.resto, "el resto no es el de la geometria");
        // cuantas filas: los cosets distintos de cada capa, por ocho, mas el resto
        let mut filas = g.resto;
        let mut posiciones = d.posiciones.clone();
        let mut tam = lde;
        for _ in 0..g.capas_fri {
            posiciones = fold_positions(&posiciones, tam, PLEGADO);
            filas += PLEGADO * posiciones.len();
            tam /= PLEGADO;
        }
        Mirilla {
            largo,
            lde,
            capas: g.capas_fri,
            resto: g.resto,
            posiciones: d.posiciones.clone(),
            alfas: d.alfas.clone(),
            twiddles: fft::get_twiddles::<BaseElement>(largo),
            inv_twiddles: fft::get_inv_twiddles::<BaseElement>(tam_final),
            filas,
        }
    }

    /// Lo que FRI abre de `d` (grado ≤ L − 2, `largo` coeficientes): capa a capa, la hoja entera
    /// del coset de plegado de cada posicion, y el resto. El mismo pliegue y el mismo offset que
    /// el probador de winter-fri.
    fn observa(&self, d: &[E]) -> Vec<E> {
        assert_eq!(d.len(), self.largo);
        let blowup = self.lde / self.largo;
        let mut evals = fft::evaluate_poly_with_offset(d, &self.twiddles, BaseElement::GENERATOR, blowup);
        let mut filas = Vec::with_capacity(self.filas);
        let mut posiciones = self.posiciones.clone();
        let mut tam = self.lde;
        for capa in 0..self.capas {
            let plegadas = fold_positions(&posiciones, tam, PLEGADO);
            let t = transpuesta::<PLEGADO>(&evals);
            for &p in &plegadas {
                filas.extend_from_slice(&t[p]);
            }
            evals = apply_drp::<BaseElement, E, PLEGADO>(&t, BaseElement::GENERATOR, self.alfas[capa]);
            tam /= PLEGADO;
            posiciones = plegadas;
        }
        fft::interpolate_poly_with_offset(&mut evals, &self.inv_twiddles, BaseElement::GENERATOR);
        filas.extend_from_slice(&evals[..self.resto]);
        assert_eq!(filas.len(), self.filas);
        filas
    }
}

/// La transposicion de winter-fri: la fila i son los N valores i, i + n/N, …, el coset de plegado.
fn transpuesta<const N: usize>(v: &[E]) -> Vec<[E; N]> {
    let filas = v.len() / N;
    (0..filas)
        .map(|i| {
            let mut fila = [E::ZERO; N];
            for (j, x) in fila.iter_mut().enumerate() {
                *x = v[i + j * filas];
            }
            fila
        })
        .collect()
}

// ---- la medida ----

/// Las mascaras que se miden: cuantas columnas aleatorias sobre F -la construccion de hoy lleva
/// una- y si hay una columna aleatoria sobre E (la 133: una columna del tramo auxiliar, o un
/// polinomio aleatorio de E[x] del grado de D, que es lo mismo para R3).
#[derive(Debug, Clone, Copy)]
pub(crate) struct Mascaras {
    pub columnas_f: usize,
    pub columna_e: bool,
}

/// La construccion de hoy: una columna aleatoria sobre F.
pub(crate) const DE_HOY: Mascaras = Mascaras { columnas_f: 1, columna_e: false };

/// §899, la 133: la construccion de laboratorio, dos columnas aleatorias sobre F, cada una con su
/// coeficiente de DEEP en E: juntas cubren las dos coordenadas de cada valor que FRI abre.
pub(crate) const LABORATORIO: Mascaras = Mascaras { columnas_f: 2, columna_e: false };

/// Mide R3 para la geometria `g` con los desafios de `semilla` y la construccion de hoy; con
/// `con_o`, tambien el rango de O entera, que informa y no decide.
pub(crate) fn mide(g: &Geometria, semilla: u64, con_o: bool) -> MedidaR3 {
    mide_con(g, semilla, con_o, DE_HOY)
}

/// Mide R3 con las mascaras que se le digan.
pub(crate) fn mide_con(g: &Geometria, semilla: u64, con_o: bool, mascaras: Mascaras) -> MedidaR3 {
    let d = desafios(g, semilla);
    let mirilla = Mirilla::nueva(g, &d);
    let largo = mirilla.largo;
    let lde = mirilla.lde;
    let omega = BaseElement::get_root_of_unity(lde.ilog2());
    let g_traza = E::from(BaseElement::get_root_of_unity(largo.ilog2()));
    let x: Vec<E> = d
        .posiciones
        .iter()
        .map(|&p| E::from(BaseElement::GENERATOR * omega.exp(p as u64)))
        .collect();
    let (z, zg) = (d.z, d.z * g_traza);
    let x0 = (z + zg) / E::from(BaseElement::new(2));
    let ell = vec![-(z + zg), E::from(BaseElement::new(2))];
    let v_pos = de_raices(&x);
    let s = largo - g.m;
    let mut moneda = Moneda::new(&[BaseElement::new(semilla ^ 0x5eed)]);

    // O entero y O sobre K: combinaciones aleatorias, mas que filas tiene O
    let n = mirilla.filas + 8;
    let rango_o = con_o.then(|| {
        let filas_o: Vec<Vec<E>> = (0..n)
            .map(|_| {
                let p: Vec<E> = (0..largo - 1).map(|_| saca(&mut moneda)).collect();
                mirilla.observa(&con_largo(p, largo))
            })
            .collect();
        rango_de(filas_o)
    });
    let base_k = mul(&v_pos, &[-x0, E::ONE]);
    let libres_k = largo - 1 - base_k.len() + 1; // grado de u < L − 2 − q
    let filas_k: Vec<Vec<E>> = (0..n)
        .map(|_| {
            let u: Vec<E> = (0..libres_k).map(|_| saca(&mut moneda)).collect();
            mirilla.observa(&con_largo(mul(&base_k, &u), largo))
        })
        .collect();
    let rango_k = 2 * rango_de(filas_k);

    // las columnas aleatorias sobre F: Σ_c Φ_c·u_c con u_c ∈ F[x] de grado < L − q − 4,
    // combinaciones aleatorias del espacio de todas juntas, mas que su rango puede ser; cada
    // columna con su coeficiente de DEEP, como en la prueba
    let conj = [conjugado(z), conjugado(zg)];
    let base_f = mul(&mul(&ell, &v_pos), &de_raices(&conj));
    let libres_r = largo - g.q - 4;
    let phis: Vec<Vec<E>> = (0..mascaras.columnas_f)
        .map(|c| {
            let cc = if c == 0 { d.cc_columna } else { saca::<E>(&mut moneda) };
            base_f.iter().map(|x| *x * cc).collect()
        })
        .collect();
    let n_r = (mascaras.columnas_f * libres_r).min(rango_k) + 8;
    let mut filas_columna: Vec<Vec<BaseElement>> = Vec::new();
    for _ in 0..n_r {
        let mut p = vec![E::ZERO; largo];
        for phi in &phis {
            let u: Vec<E> = (0..libres_r).map(|_| E::from(saca::<BaseElement>(&mut moneda))).collect();
            for (x, y) in p.iter_mut().zip(mul(phi, &u)) {
                *x += y;
            }
        }
        filas_columna.push(sobre_f(&mirilla.observa(&p)));
    }
    // la columna aleatoria sobre E, si la hay: cc·V·ℓ·x^i, i < L − q − 2, exactamente K
    if mascaras.columna_e {
        let cc: E = saca(&mut moneda);
        let base_e: Vec<E> = mul(&ell, &v_pos).iter().map(|x| *x * cc).collect();
        let libres_e = largo - g.q - 2;
        let n_e = libres_e.min(rango_k / 2 + 4);
        for _ in 0..n_e {
            let v: Vec<E> = (0..libres_e).map(|_| saca(&mut moneda)).collect();
            let fila = mirilla.observa(&con_largo(mul(&base_e, &v), largo));
            filas_columna.push(sobre_f(&fila));
            filas_columna.push(sobre_f(&fila.iter().map(|e| *e * E::new(BaseElement::ZERO, BaseElement::ONE)).collect::<Vec<E>>()));
        }
    }
    let rango_columna = rango_de(filas_columna.clone());

    // los aleatorizadores: (cc'_k·x^s − cc'_{k+1})·V·ℓ·x^i, i < m − q − 2, un espacio sobre E
    let libres_v = g.m.saturating_sub(g.q + 2);
    let mut filas_trozos_e: Vec<Vec<E>> = Vec::new();
    for k in 0..g.trozos.saturating_sub(1) {
        let mut w = vec![E::ZERO; s + 1];
        w[0] = -d.cc_trozos[k + 1];
        w[s] = d.cc_trozos[k];
        let w = mul(&mul(&w, &v_pos), &ell);
        for i in 0..libres_v {
            let mut p = vec![E::ZERO; i];
            p.extend_from_slice(&w);
            filas_trozos_e.push(mirilla.observa(&con_largo(p, largo)));
        }
    }
    let rango_trozos = 2 * rango_de(filas_trozos_e.clone());
    let u_e = E::new(BaseElement::ZERO, BaseElement::ONE);
    let mut juntas = filas_columna;
    for fila in &filas_trozos_e {
        juntas.push(sobre_f(fila));
        juntas.push(sobre_f(&fila.iter().map(|e| *e * u_e).collect::<Vec<E>>()));
    }
    let rango_mascaras = rango_de(juntas);
    // las inclusiones que la construccion promete: cada mascara dentro de las dos juntas, y las
    // dos juntas dentro de K; si una mascara saliera de K (una conjugacion mal puesta), aqui
    assert!(
        rango_columna <= rango_mascaras && rango_trozos <= rango_mascaras,
        "una mascara sola no cabe en las dos juntas: {rango_columna}, {rango_trozos} frente a {rango_mascaras}"
    );
    assert!(rango_mascaras <= rango_k, "las mascaras salen de K: {rango_mascaras} > {rango_k}");

    MedidaR3 { filas: mirilla.filas, rango_o, rango_k, rango_columna, rango_trozos, rango_mascaras }
}

/// Una linea por medida.
pub(crate) fn linea(nombre: &str, g: &Geometria, m: &MedidaR3) -> String {
    format!(
        "[R3] {nombre}: q {} · T {} · m {} · {} trozos · FRI {} capas, resto {} · O: {} filas sobre E, \
         rango {} · sin determinar (O sobre K): {} sobre F · mascaras: columna {} · trozos {} · juntas {} \
         · R3 {}",
        g.q,
        g.t,
        g.m,
        g.trozos,
        g.capas_fri,
        g.resto,
        m.filas,
        m.rango_o.map_or("-".to_string(), |r| r.to_string()),
        m.rango_k,
        m.rango_columna,
        m.rango_trozos,
        m.rango_mascaras,
        if m.cumple() { "SE CUMPLE" } else { "NO se cumple" }
    )
}

/// Lo medido, clavado: por geometria de produccion -T, trozos, capas de FRI y resto, con q = 42
/// y m = 64-, lo que FRI abre sin determinar (el rango de O sobre K, sobre F) y lo que cubren las
/// mascaras: la columna aleatoria, los aleatorizadores y las dos juntas. En ninguna se cumple R3,
/// y un cambio en cualquier cifra es rojo: la medida es la del §898, con la semilla 1.
pub(crate) const R3_MEDIDO: &[(usize, usize, usize, usize, usize, usize, usize, usize)] = &[
    // T, trozos, capas, resto, sin determinar, columna, trozos, juntas
    (64, 15, 1, 16, 168, 82, 80, 126),
    (256, 2, 2, 8, 632, 338, 40, 378),
    (256, 9, 2, 8, 632, 338, 80, 382),
    (512, 8, 2, 16, 760, 466, 80, 510),
    (1024, 8, 2, 32, 1016, 722, 80, 766),
];

/// La fila de `R3_MEDIDO` de una geometria, si la tiene.
pub(crate) fn clavado(g: &Geometria) -> Option<(usize, usize, usize, usize)> {
    fila_de(R3_MEDIDO, g)
}

/// §899 (la 133): lo medido con la construccion de LABORATORIO, dos columnas aleatorias sobre F,
/// clavado como lo de hoy: en las cinco geometrias las dos columnas y los aleatorizadores juntos
/// cubren todo lo que FRI abre sin determinar -R3 se cumple-, y con T >= 256 las dos columnas
/// solas; un cambio en cualquier cifra es rojo.
pub(crate) const R3_MEDIDO_LABORATORIO: &[(usize, usize, usize, usize, usize, usize, usize, usize)] = &[
    // T, trozos, capas, resto, sin determinar, columnas, trozos, juntas
    (64, 15, 1, 16, 168, 164, 80, 168),
    (256, 2, 2, 8, 632, 632, 40, 632),
    (256, 9, 2, 8, 632, 632, 80, 632),
    (512, 8, 2, 16, 760, 760, 80, 760),
    (1024, 8, 2, 32, 1016, 1016, 80, 1016),
];

/// La fila de `R3_MEDIDO_LABORATORIO` de una geometria, si la tiene.
pub(crate) fn clavado_laboratorio(g: &Geometria) -> Option<(usize, usize, usize, usize)> {
    fila_de(R3_MEDIDO_LABORATORIO, g)
}

fn fila_de(
    tabla: &[(usize, usize, usize, usize, usize, usize, usize, usize)],
    g: &Geometria,
) -> Option<(usize, usize, usize, usize)> {
    tabla
        .iter()
        .find(|f| f.0 == g.t && f.1 == g.trozos && f.2 == g.capas_fri && f.3 == g.resto)
        .map(|f| (f.4, f.5, f.6, f.7))
}

/// La geometria canonica: la de la prueba con q en su tope, 42, que es con la que se clava.
pub(crate) fn canonica(g: &Geometria) -> Geometria {
    Geometria { q: g.consultas, ..g.clone() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn geo(q: usize, t: usize, m: usize, trozos: usize, capas: usize, resto: usize) -> Geometria {
        Geometria {
            q,
            consultas: 42,
            t,
            m,
            ancho: 0,
            ancho_aux: 0,
            trozos,
            capas_fri: capas,
            resto,
            bytes: 0,
        }
    }

    /// **El juguete de `instrumento_cota`, T = 128 con cuatro trozos: R3 no se cumple, con sus
    /// cifras.** Y la prueba de vida del instrumento: con pocas consultas -cuatro- los
    /// aleatorizadores cubren lo que la columna aleatoria deja, y R3 SE cumple: el instrumento
    /// sabe decir que si.
    #[test]
    fn el_juguete_no_cumple_r3_y_con_cuatro_consultas_si() {
        let g = geo(42, 128, 64, 4, 1, 32);
        let m = mide(&g, 1, true);
        println!("{}", linea("juguete (WorkAir)", &g, &m));
        assert_eq!((m.rango_k, m.rango_columna, m.rango_trozos, m.rango_mascaras), (424, 210, 80, 254));
        assert!(!m.cumple());
        let pocas = Geometria { q: 4, consultas: 4, ..g };
        let m = mide(&pocas, 1, true);
        println!("{}", linea("cuatro consultas", &pocas, &m));
        assert!(m.cumple(), "con cuatro consultas las mascaras cubren lo que FRI abre");
    }

    /// **§899 (la 133): el juguete con dos columnas aleatorias cumple R3, y con una columna sobre E
    /// tambien, exactamente K.** Las dos columnas solas cubren K menos cuatro dimensiones (420 de
    /// 424) y los aleatorizadores ponen el resto; la columna sobre E, que es K por construccion,
    /// lo cubre sola.
    #[test]
    fn el_juguete_con_dos_columnas_cumple_r3_y_con_una_sobre_e_tambien() {
        let g = geo(42, 128, 64, 4, 1, 32);
        let m = mide_con(&g, 1, false, LABORATORIO);
        println!("{} · laboratorio", linea("juguete (WorkAir)", &g, &m));
        assert_eq!((m.rango_k, m.rango_columna, m.rango_trozos, m.rango_mascaras), (424, 420, 80, 424));
        assert!(m.cumple());
        let e = mide_con(&g, 1, false, Mascaras { columnas_f: 1, columna_e: true });
        println!("{} · una columna sobre E", linea("juguete (WorkAir)", &g, &e));
        assert_eq!((e.rango_k, e.rango_columna, e.rango_mascaras), (424, 424, 424));
        assert!(e.cumple());
    }

    /// **La tabla clavada tiene una fila por geometria de produccion, y ninguna cumple.**
    #[test]
    fn la_tabla_clavada_es_de_las_cinco_geometrias_y_ninguna_cumple() {
        assert_eq!(R3_MEDIDO.len(), 5);
        for f in R3_MEDIDO {
            assert!(f.7 < f.4, "T = {}: {} de mascara frente a {} sin determinar", f.0, f.7, f.4);
            assert!(f.5 <= f.7 && f.6 <= f.7);
        }
        // §899: la de laboratorio, las mismas cinco geometrias, y todas cumplen
        assert_eq!(R3_MEDIDO_LABORATORIO.len(), 5);
        for (f, l) in R3_MEDIDO.iter().zip(R3_MEDIDO_LABORATORIO) {
            assert_eq!((f.0, f.1, f.2, f.3, f.4), (l.0, l.1, l.2, l.3, l.4), "la misma geometria y el mismo K");
            assert_eq!(l.7, l.4, "T = {}: el laboratorio cubre todo lo que FRI abre", l.0);
            assert!(l.5 <= l.7 && l.6 <= l.7);
        }
    }
}
