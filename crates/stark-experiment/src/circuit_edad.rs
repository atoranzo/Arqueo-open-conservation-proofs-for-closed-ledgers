//! **RFC-0007 E4b-1 (§463): el PROBADOR de la prueba de edad.**
//!
//! El AIR vive en `zk-ssl-air` -el crate que el kit consume sin el probador- y aqui solo esta lo
//! que produce una prueba: las celdas de cada posicion, la traza principal, la auxiliar (los
//! inversos y la suma del multiconjunto) y el `Prover`. Lo que el AIR afirma, sus cotas y lo que
//! NO prueba estan en la cabecera de `zk_ssl_air`; aqui no se repite.
//!
//! La entrada es un libro: las hojas del arbol de pendientes en `0..n` (la cero es un hueco) y la
//! meta `(emisor, nacido)` de cada una. Un pendiente vivo sin meta NO se construye (decision
//! D-2): su edad no esta comprometida y la prueba falla cerrada. Los testigos negativos no pasan
//! por aqui: fabrican [`Celda`]s a mano y las llevan a [`trazar`], que no comprueba nada, para que
//! lo que tumbe la prueba sea el AIR y no esta funcion.

use winterfell::crypto::hashers::{Blake3_256, Rp64_256};
use winterfell::crypto::{DefaultRandomCoin, MerkleTree};
use winterfell::math::{batch_inversion, fields::f64::BaseElement, FieldElement};
use winterfell::matrix::ColMatrix;
use winterfell::{
    AuxRandElements, CompositionPoly, CompositionPolyTrace, ConstraintCompositionCoefficients,
    DefaultConstraintCommitment, DefaultConstraintEvaluator, DefaultTraceLde, EvaluationFrame,
    PartitionOptions, ProofOptions, Prover, StarkDomain, Trace, TraceInfo, TracePolyTable,
};
use zk_ssl_hash::{meta_pendiente_hoja, native_merge, DOMINIO_META_PENDIENTE};

pub use zk_ssl_air::{
    carril, codificar, m_canonico, opciones, potencias, raiz_desde_subraiz, subraiz, verificar,
    verificar_contra_cabeza, Afirmacion, CabezaEdad, Digest, EdadAir, EdadPublicInputs, ALEATORIOS,
    ANCHO, ANCHO_AUX, BITS, CICLO, ESTADO, PROFUNDIDAD,
};
use zk_ssl_air::{
    C_A, C_ACT, C_B, C_BITS, C_CICLO, C_CUENTA, C_EMISOR, C_FUERA, C_M, C_NACIDO, C_P, C_QA, C_QS,
    C_VIVO, C_WINV, C_Z,
};

type Blake3 = Blake3_256<BaseElement>;

/// Lo que se afirma: la altura de la cabeza, la edad `T`, y el emisor nombrado o todos.
#[derive(Clone, Debug)]
pub struct Enunciado {
    pub seq: u64,
    pub t: u64,
    pub emisor: u64,
    pub todos: bool,
}

/// **Una posicion, tal como la traza la lleva.** `qa` y `qs` son las elecciones del probador:
/// el AIR solo le deja ponerlas a 0 si puede demostrarlo (joven, otro emisor).
#[derive(Clone, Debug)]
pub struct Celda {
    pub hoja: Digest,
    pub vivo: bool,
    pub emisor: u64,
    pub nacido: u64,
    pub qa: bool,
    pub qs: bool,
}

const CERO: Digest = [BaseElement::ZERO; 4];

/// Las celdas HONESTAS de un libro, rellenas con huecos hasta la potencia de dos (al menos 2).
/// Falla cerrada ante un vivo sin meta o una meta en un hueco.
pub fn celdas_del_libro(
    hojas: &[Digest],
    meta: &[Option<(u64, u64)>],
    en: &Enunciado,
) -> Result<Vec<Celda>, String> {
    if hojas.len() != meta.len() {
        return Err(format!("{} hojas y {} metas", hojas.len(), meta.len()));
    }
    if en.t >= (1u64 << BITS) || en.seq >= (1u64 << BITS) {
        return Err(format!("T = {} o seq = {} no caben en {BITS} bits", en.t, en.seq));
    }
    let tam = hojas.len().next_power_of_two().max(2);
    let nombrado = BaseElement::new(en.emisor);
    let mut celdas = Vec::with_capacity(tam);
    for p in 0..tam {
        let hoja = if p < hojas.len() { hojas[p] } else { CERO };
        let vivo = hoja != CERO;
        let m = if p < meta.len() { meta[p] } else { None };
        let (emisor, nacido) = match (vivo, m) {
            (true, Some(x)) => x,
            (true, None) => {
                return Err(format!("posicion {p}: pendiente vivo sin meta (falla cerrada)"))
            }
            (false, Some(_)) => return Err(format!("posicion {p}: meta en un hueco")),
            (false, None) => (0, 0),
        };
        let (qa, qs) = if vivo {
            let resto = en.t as i128 - 1 - en.seq as i128 + nacido as i128;
            let joven = (0..(1i128 << BITS)).contains(&resto);
            (!joven, en.todos || BaseElement::new(emisor) == nombrado)
        } else {
            (true, true)
        };
        celdas.push(Celda { hoja, vivo, emisor, nacido, qa, qs });
    }
    Ok(celdas)
}

/// **La traza de unas celdas, sin comprobar nada.** `n` es la marca que se declara. La cuenta
/// `k` es la que la traza suma; las dos subraices, las de las hojas y de `vivo * hoja de meta`.
pub fn trazar(celdas: &[Celda], n: u64, en: &Enunciado) -> TrazaEdad {
    let hojas = celdas.len();
    assert!(hojas >= 2 && hojas.is_power_of_two(), "celdas: potencia de dos >= 2");
    let filas = hojas * CICLO;
    let m = hojas.trailing_zeros();

    let metas: Vec<Digest> = celdas
        .iter()
        .map(|c| if c.vivo { meta_pendiente_hoja(c.emisor, c.nacido) } else { CERO })
        .collect();
    let arbol = |hojas_de: &dyn Fn(usize) -> Digest| -> Vec<Digest> {
        let mut nodos = vec![CERO; 2 * hojas];
        for p in 0..hojas {
            nodos[hojas + p] = hojas_de(p);
        }
        for j in (1..hojas).rev() {
            nodos[j] = native_merge(nodos[2 * j], nodos[2 * j + 1]);
        }
        nodos
    };
    let nodos_a = arbol(&|p| celdas[p].hoja);
    let nodos_b = arbol(&|p| metas[p]);

    let mut col = vec![vec![BaseElement::ZERO; filas]; ANCHO];
    let nombrado = BaseElement::new(en.emisor);
    let mut cuenta = 0u64;
    for (c, celda) in celdas.iter().enumerate() {
        let f = c * CICLO;
        let mut inicial = [[BaseElement::ZERO; ESTADO]; 3];
        if c >= 1 {
            for (k, nodos) in [&nodos_a, &nodos_b].into_iter().enumerate() {
                inicial[k][4..8].copy_from_slice(&nodos[2 * c]);
                inicial[k][8..12].copy_from_slice(&nodos[2 * c + 1]);
            }
        }
        inicial[2][0] = BaseElement::new(DOMINIO_META_PENDIENTE);
        inicial[2][4] = BaseElement::new(celda.emisor);
        inicial[2][5] = BaseElement::new(celda.nacido);
        for (k, base) in [C_A, C_B, C_M].into_iter().enumerate() {
            let mut estado = inicial[k];
            for r in 0..CICLO {
                for i in 0..ESTADO {
                    col[base + i][f + r] = estado[i];
                }
                if r < CICLO - 1 {
                    Rp64_256::apply_round(&mut estado, r);
                }
            }
        }

        let z = celda.vivo && celda.qa && celda.qs;
        let resto = if celda.qa {
            0u64
        } else {
            let r = en.t as i128 - 1 - en.seq as i128 + celda.nacido as i128;
            r.rem_euclid(1i128 << BITS) as u64
        };
        let winv = if celda.qs {
            BaseElement::ZERO
        } else {
            (BaseElement::new(celda.emisor) - nombrado).inv()
        };
        let b = |x: bool| if x { BaseElement::ONE } else { BaseElement::ZERO };
        for r in 0..CICLO {
            let fila = f + r;
            for i in 0..4 {
                col[C_P + i][fila] = celda.hoja[i];
            }
            col[C_EMISOR][fila] = BaseElement::new(celda.emisor);
            col[C_NACIDO][fila] = BaseElement::new(celda.nacido);
            col[C_VIVO][fila] = b(celda.vivo);
            col[C_QA][fila] = b(celda.qa);
            col[C_QS][fila] = b(celda.qs);
            col[C_WINV][fila] = winv;
            col[C_Z][fila] = b(z);
            col[C_CUENTA][fila] = BaseElement::new(if r == 0 { cuenta } else { cuenta + z as u64 });
            col[C_CICLO][fila] = BaseElement::new(c as u64);
            col[C_ACT][fila] = b(c >= 1);
            col[C_FUERA][fila] = b(c as u64 >= n);
            for i in 0..BITS {
                col[C_BITS + i][fila] = BaseElement::new((resto >> i) & 1);
            }
        }
        cuenta += z as u64;
    }

    let pi = EdadPublicInputs {
        subraiz_pend: nodos_a[1],
        subraiz_meta: nodos_b[1],
        m,
        n,
        seq: en.seq,
        t: en.t,
        emisor: en.emisor,
        todos: en.todos,
        k: cuenta,
    };
    TrazaEdad {
        info: TraceInfo::new_multi_segment(ANCHO, ANCHO_AUX, ALEATORIOS, filas, vec![]),
        principal: ColMatrix::new(col),
        pi,
    }
}

/// Las celdas honestas del libro, trazadas con su marca `n = hojas.len()`.
pub fn construir(
    hojas: &[Digest],
    meta: &[Option<(u64, u64)>],
    en: &Enunciado,
) -> Result<TrazaEdad, String> {
    let celdas = celdas_del_libro(hojas, meta, en)?;
    Ok(trazar(&celdas, hojas.len() as u64, en))
}

/// **Prueba con las opciones de la casa** y devuelve los bytes y el enunciado que declaran.
pub fn probar(traza: TrazaEdad) -> Result<(Vec<u8>, EdadPublicInputs), String> {
    probar_con(traza, opciones())
}

/// Prueba con OTRAS opciones: solo para el testigo de que el juez no las acepta.
pub fn probar_con(
    traza: TrazaEdad,
    opts: ProofOptions,
) -> Result<(Vec<u8>, EdadPublicInputs), String> {
    let pi = traza.pi.clone();
    let prueba = EdadProver { options: opts }.prove(traza).map_err(|e| format!("{e:?}"))?;
    Ok((prueba.to_bytes(), pi))
}

// ------------------------------------------------------------------ la traza

/// La traza de dos segmentos: `TraceTable` solo declara el principal.
pub struct TrazaEdad {
    info: TraceInfo,
    principal: ColMatrix<BaseElement>,
    pi: EdadPublicInputs,
}

impl TrazaEdad {
    /// El enunciado que la traza declara.
    pub fn enunciado(&self) -> &EdadPublicInputs {
        &self.pi
    }

    /// Falsea una celda de la traza principal: solo para los testigos negativos.
    pub fn falsear(&mut self, columna: usize, fila: usize, valor: BaseElement) {
        self.principal.set(columna, fila, valor);
    }

    /// Declara otro enunciado sobre la misma traza: solo para los testigos negativos.
    pub fn declarar(&mut self, pi: EdadPublicInputs) {
        self.pi = pi;
    }
}

impl Trace for TrazaEdad {
    type BaseField = BaseElement;

    fn info(&self) -> &TraceInfo {
        &self.info
    }

    fn main_segment(&self) -> &ColMatrix<BaseElement> {
        &self.principal
    }

    fn read_main_frame(&self, row_idx: usize, frame: &mut EvaluationFrame<BaseElement>) {
        let siguiente = (row_idx + 1) % self.info.length();
        self.principal.read_row_into(row_idx, frame.current_mut());
        self.principal.read_row_into(siguiente, frame.next_mut());
    }
}

/// **La traza auxiliar**: por ciclo, los inversos de los cinco terminos de la fila 0 (cuatro
/// lecturas y la hoja de pendientes) y de los tres de la fila 6 (los dos nodos y la hoja de meta,
/// con los valores de la fila 7), y la suma acumulada que el AIR comprueba fila a fila.
fn construir_auxiliar<E: FieldElement<BaseField = BaseElement>>(
    m: &ColMatrix<BaseElement>,
    r: &[E],
) -> ColMatrix<E> {
    let filas = m.num_rows();
    let hojas = filas / CICLO;
    let pot = potencias(r[0]);
    let beta = r[1];
    let v = |col: usize, fila: usize| E::from(m.get(col, fila));
    let dig = |col: usize, fila: usize| {
        [v(col, fila), v(col + 1, fila), v(col + 2, fila), v(col + 3, fila)]
    };
    let (ca, cb) = (carril::<E>(false), carril::<E>(true));
    let n_hojas = E::from(BaseElement::new(hojas as u64));
    let dos = E::from(2u32);

    let mut den = Vec::with_capacity(8 * hojas);
    for c in 0..hojas {
        let (f0, f7) = (c * CICLO, c * CICLO + 7);
        let ciclo = v(C_CICLO, f0);
        let vivo = v(C_VIVO, f0);
        den.push(codificar(&pot, beta, dos * ciclo, dig(C_A + 4, f0), ca));
        den.push(codificar(&pot, beta, dos * ciclo + E::ONE, dig(C_A + 8, f0), ca));
        den.push(codificar(&pot, beta, dos * ciclo, dig(C_B + 4, f0), cb));
        den.push(codificar(&pot, beta, dos * ciclo + E::ONE, dig(C_B + 8, f0), cb));
        den.push(codificar(&pot, beta, n_hojas + ciclo, dig(C_P, f0), ca));
        let meta = dig(C_M + 4, f7);
        let hoja_meta = [vivo * meta[0], vivo * meta[1], vivo * meta[2], vivo * meta[3]];
        den.push(codificar(&pot, beta, ciclo, dig(C_A + 4, f7), ca));
        den.push(codificar(&pot, beta, ciclo, dig(C_B + 4, f7), cb));
        den.push(codificar(&pot, beta, n_hojas + ciclo, hoja_meta, cb));
    }
    let inv = batch_inversion(&den);

    let mut h = vec![vec![E::ZERO; filas]; 5];
    for c in 0..hojas {
        for k in 0..5 {
            h[k][c * CICLO] = inv[8 * c + k];
        }
        for k in 0..3 {
            h[k][c * CICLO + 6] = inv[8 * c + 5 + k];
        }
    }
    let mut s = vec![E::ZERO; filas];
    for fila in 0..filas - 1 {
        let act = v(C_ACT, fila);
        let delta = match fila % CICLO {
            0 => h[4][fila] - act * (h[0][fila] + h[1][fila] + h[2][fila] + h[3][fila]),
            6 => act * (h[0][fila] + h[1][fila]) + h[2][fila],
            _ => E::ZERO,
        };
        s[fila + 1] = s[fila] + delta;
    }
    h.push(s);
    ColMatrix::new(h)
}

// ------------------------------------------------------------------ el probador

pub struct EdadProver {
    options: ProofOptions,
}

impl Prover for EdadProver {
    type BaseField = BaseElement;
    type Air = EdadAir;
    type Trace = TrazaEdad;
    type HashFn = Blake3;
    type VC = MerkleTree<Blake3>;
    type RandomCoin = DefaultRandomCoin<Blake3>;
    type TraceLde<E: FieldElement<BaseField = Self::BaseField>> =
        DefaultTraceLde<E, Self::HashFn, Self::VC>;
    type ConstraintEvaluator<'a, E: FieldElement<BaseField = Self::BaseField>> =
        DefaultConstraintEvaluator<'a, Self::Air, E>;
    type ConstraintCommitment<E: FieldElement<BaseField = Self::BaseField>> =
        DefaultConstraintCommitment<E, Self::HashFn, Self::VC>;

    fn get_pub_inputs(&self, trace: &Self::Trace) -> EdadPublicInputs {
        trace.pi.clone()
    }

    fn options(&self) -> &ProofOptions {
        &self.options
    }

    fn new_trace_lde<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        trace_info: &TraceInfo,
        main_trace: &ColMatrix<Self::BaseField>,
        domain: &StarkDomain<Self::BaseField>,
        partition_option: PartitionOptions,
    ) -> (Self::TraceLde<E>, TracePolyTable<E>) {
        DefaultTraceLde::new(trace_info, main_trace, domain, partition_option)
    }

    fn new_evaluator<'a, E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        air: &'a Self::Air,
        aux_rand_elements: Option<AuxRandElements<E>>,
        composition_coefficients: ConstraintCompositionCoefficients<E>,
    ) -> Self::ConstraintEvaluator<'a, E> {
        DefaultConstraintEvaluator::new(air, aux_rand_elements, composition_coefficients)
    }

    fn build_constraint_commitment<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        composition_poly_trace: CompositionPolyTrace<E>,
        num_constraint_composition_columns: usize,
        domain: &StarkDomain<Self::BaseField>,
        partition_options: PartitionOptions,
    ) -> (Self::ConstraintCommitment<E>, CompositionPoly<E>) {
        DefaultConstraintCommitment::new(
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
        construir_auxiliar(main_trace.main_segment(), aux_rand_elements.rand_elements())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic::{catch_unwind, AssertUnwindSafe};
    use winterfell::{BatchingMethod, FieldExtension};

    /// Un digest distinto de cero por `k`: la hoja de un pendiente vivo.
    fn d(k: u64) -> Digest {
        [
            BaseElement::new(k + 1),
            BaseElement::new(3 * k + 2),
            BaseElement::new(7 * k + 3),
            BaseElement::new(11 * k + 4),
        ]
    }

    /// Trece posiciones: huecos en las multiplos de 4; emisor `p % 3`; nacido `10 p`. Con
    /// `seq = 130`, las vivas tienen 120, 110, 100, 80, 70, 60, 40, 30 y 20 de edad.
    fn libro() -> (Vec<Digest>, Vec<Option<(u64, u64)>>) {
        let (mut hojas, mut meta) = (Vec::new(), Vec::new());
        for p in 0..13u64 {
            if p % 4 == 0 {
                hojas.push(CERO);
                meta.push(None);
            } else {
                hojas.push(d(p));
                meta.push(Some((p % 3, 10 * p)));
            }
        }
        (hojas, meta)
    }

    fn en(t: u64, emisor: u64, todos: bool) -> Enunciado {
        Enunciado { seq: 130, t, emisor, todos }
    }

    /// La cuenta del enunciado, hecha a mano sobre el libro y sin la traza.
    fn a_mano(meta: &[Option<(u64, u64)>], e: &Enunciado) -> u64 {
        let cuenta = meta.iter().flatten().filter(|(emisor, nacido)| {
            e.seq - nacido >= e.t && (e.todos || *emisor == e.emisor)
        });
        cuenta.count() as u64
    }

    /// **Lo unico inaceptable es que verifique** (el molde de `merkle.rs`): el probador puede
    /// devolver `Err`, o producir una prueba que el juez rechace.
    fn no_verifica(traza: TrazaEdad, pi: &EdadPublicInputs) -> bool {
        match catch_unwind(AssertUnwindSafe(|| probar(traza))) {
            Err(_) | Ok(Err(_)) => true,
            Ok(Ok((bytes, _))) => verificar(&bytes, pi).is_err(),
        }
    }

    /// La cabeza que firmaria el libro con la marca `n`: sus dos raices, a 32 niveles, compuestas
    /// en nativo desde las hojas (no desde la traza) con la `m` minima de la marca.
    fn cabeza_del_libro(
        hojas: &[Digest],
        meta: &[Option<(u64, u64)>],
        n: u64,
        seq: u64,
    ) -> CabezaEdad {
        let m = m_canonico(n);
        let tam = 1usize << m;
        let mut pend = hojas.to_vec();
        pend.resize(tam, CERO);
        let metas: Vec<Digest> = (0..tam)
            .map(|p| match meta.get(p).copied().flatten() {
                Some((e, b)) => meta_pendiente_hoja(e, b),
                None => CERO,
            })
            .collect();
        CabezaEdad {
            seq,
            pending_root: raiz_desde_subraiz(subraiz(&pend), m, PROFUNDIDAD),
            pmeta_root: raiz_desde_subraiz(subraiz(&metas), m, PROFUNDIDAD),
            next_pending: n,
        }
    }

    fn afirmacion(pi: &EdadPublicInputs) -> Afirmacion {
        Afirmacion {
            t: pi.t,
            k: pi.k,
            emisor: None,
            subraiz_pend: pi.subraiz_pend,
            subraiz_meta: pi.subraiz_meta,
        }
    }

    fn honestas(e: &Enunciado) -> Vec<Celda> {
        let (hojas, meta) = libro();
        celdas_del_libro(&hojas, &meta, e).expect("celdas honestas")
    }

    /// Las subraices de la traza son las de las hojas: la de pendientes, y la de meta con la hoja
    /// cero en los huecos. Sin probar nada.
    #[test]
    fn la_subraiz_de_la_traza_es_la_de_las_hojas() {
        let (hojas, meta) = libro();
        let traza = construir(&hojas, &meta, &en(70, 0, true)).expect("construir");
        let pi = traza.enunciado().clone();
        let mut pend = hojas.clone();
        pend.resize(16, CERO);
        let metas: Vec<Digest> = (0..16)
            .map(|p| match meta.get(p).copied().flatten() {
                Some((e, n)) => meta_pendiente_hoja(e, n),
                None => CERO,
            })
            .collect();
        assert_eq!((pi.m, pi.n), (4, 13));
        assert_eq!(pi.subraiz_pend, subraiz(&pend));
        assert_eq!(pi.subraiz_meta, subraiz(&metas));
    }

    /// La caja vacia: con `T` por encima de toda edad, `K = 0`, y la prueba verifica.
    #[test]
    #[cfg_attr(debug_assertions, ignore = "winterfell valida grados en depuracion: juez release")]
    fn la_caja_vacia_prueba_y_verifica() {
        let (hojas, meta) = libro();
        let e = en(121, 0, true);
        let (bytes, pi) = probar(construir(&hojas, &meta, &e).expect("construir")).expect("probar");
        assert_eq!(pi.k, 0);
        assert!(verificar(&bytes, &pi).is_ok(), "la caja vacia no verifico");
    }

    /// Con `T = 70` hay cinco viejas, y la cota que la prueba declara es la cuenta a mano.
    #[test]
    #[cfg_attr(debug_assertions, ignore = "winterfell valida grados en depuracion: juez release")]
    fn la_cuenta_de_viejos_es_la_del_libro() {
        let (hojas, meta) = libro();
        let e = en(70, 0, true);
        let (bytes, pi) = probar(construir(&hojas, &meta, &e).expect("construir")).expect("probar");
        assert_eq!((pi.k, a_mano(&meta, &e)), (5, 5));
        assert!(verificar(&bytes, &pi).is_ok(), "la cuenta honesta no verifico");
    }

    /// La concentracion por cuenta: con el emisor 1 nombrado solo cuentan sus tres vivas viejas.
    #[test]
    #[cfg_attr(debug_assertions, ignore = "winterfell valida grados en depuracion: juez release")]
    fn el_filtro_por_emisor_cuenta_solo_los_suyos() {
        let (hojas, meta) = libro();
        let e = en(30, 1, false);
        let (bytes, pi) = probar(construir(&hojas, &meta, &e).expect("construir")).expect("probar");
        assert_eq!((pi.k, a_mano(&meta, &e)), (3, 3));
        assert!(verificar(&bytes, &pi).is_ok(), "el filtro honesto no verifico");
    }

    /// **D-2, fail-closed.** Un pendiente vivo sin meta no se construye; y la falsificacion que lo
    /// esconde -declararlo hueco con su hoja real, que es lo que haria un operador- no verifica
    /// contra las raices del libro. La tumba `(1 - vivo) * P = 0` y nada mas.
    #[test]
    #[cfg_attr(debug_assertions, ignore = "winterfell valida grados en depuracion: juez release")]
    fn un_pendiente_vivo_sin_meta_no_se_prueba() {
        let (hojas, mut meta) = libro();
        meta[5] = None;
        let e = en(70, 0, true);
        let err = celdas_del_libro(&hojas, &meta, &e).err().expect("tenia que fallar");
        assert!(err.contains("sin meta"), "{err}");
        let mut celdas = honestas(&e);
        let real = celdas[5].clone();
        celdas[5] = Celda { vivo: false, emisor: 0, nacido: 0, qa: true, qs: true, ..real };
        let traza = trazar(&celdas, 13, &e);
        let pi = traza.enunciado().clone();
        assert_eq!(pi.k, 4, "la falsificacion tenia que esconder una vieja");
        assert!(no_verifica(traza, &pi), "un vivo escondido verifico");
    }

    /// Una vieja declarada joven (`qa = 0` con 120 de edad y `T = 70`) no verifica: su resto no
    /// cabe en 32 bits. La tumba la restriccion de la edad y nada mas.
    #[test]
    #[cfg_attr(debug_assertions, ignore = "winterfell valida grados en depuracion: juez release")]
    fn un_viejo_declarado_joven_no_verifica() {
        let e = en(70, 0, true);
        let mut celdas = honestas(&e);
        assert!(celdas[1].vivo && celdas[1].qa, "la posicion 1 tiene que ser vieja");
        celdas[1].qa = false;
        let traza = trazar(&celdas, 13, &e);
        let pi = traza.enunciado().clone();
        assert_eq!(pi.k, 4);
        assert!(no_verifica(traza, &pi), "una vieja declarada joven verifico");
    }

    /// **El testigo negativo del RFC (Seguridad):** con la marca declarada en 11, la posicion 11
    /// esta viva por encima de ella, y la prueba no verifica. La tumban `fuera * P` y
    /// `fuera * vivo`.
    #[test]
    #[cfg_attr(debug_assertions, ignore = "winterfell valida grados en depuracion: juez release")]
    fn una_posicion_viva_por_encima_de_la_marca_no_verifica() {
        let e = en(70, 0, true);
        let celdas = honestas(&e);
        assert!(celdas[11].vivo, "la posicion 11 tiene que estar viva");
        let traza = trazar(&celdas, 11, &e);
        let pi = traza.enunciado().clone();
        assert!(no_verifica(traza, &pi), "una viva por encima de la marca verifico");
    }

    /// **El cableado:** la hoja que el ciclo 5 escribe deja de ser la que su padre lee (un
    /// elemento de la hoja de pendientes, en la fila 0 del ciclo). La suma del multiconjunto ya no
    /// cierra en la raiz y la prueba no verifica.
    #[test]
    #[cfg_attr(debug_assertions, ignore = "winterfell valida grados en depuracion: juez release")]
    fn una_hoja_que_no_es_la_leida_no_verifica() {
        let (hojas, meta) = libro();
        let mut traza = construir(&hojas, &meta, &en(70, 0, true)).expect("construir");
        let pi = traza.enunciado().clone();
        let fila = 5 * CICLO;
        traza.falsear(C_P, fila, hojas[5][0] + BaseElement::ONE);
        assert!(no_verifica(traza, &pi), "una hoja escrita distinta de la leida verifico");
    }

    /// **D-4:** una prueba honesta con OTRAS opciones (sin grinding) no la acepta el juez.
    #[test]
    #[cfg_attr(debug_assertions, ignore = "winterfell valida grados en depuracion: juez release")]
    fn otras_opciones_no_se_aceptan() {
        let (hojas, meta) = libro();
        let traza = construir(&hojas, &meta, &en(70, 0, true)).expect("construir");
        let otras = ProofOptions::new(
            42,
            16,
            0,
            FieldExtension::Quadratic,
            8,
            31,
            BatchingMethod::Linear,
            BatchingMethod::Linear,
        );
        let (bytes, pi) = probar_con(traza, otras).expect("probar");
        assert!(verificar(&bytes, &pi).is_err(), "el juez acepto otras opciones");
    }

    /// Una prueba honesta es la de SU enunciado: con otra cota, otra `T`, otra marca u otra raiz,
    /// el juez la rechaza.
    #[test]
    #[cfg_attr(debug_assertions, ignore = "winterfell valida grados en depuracion: juez release")]
    fn la_prueba_no_sirve_para_otro_enunciado() {
        let (hojas, meta) = libro();
        let (bytes, pi) =
            probar(construir(&hojas, &meta, &en(70, 0, true)).expect("construir")).expect("probar");
        assert!(verificar(&bytes, &pi).is_ok(), "la de control no verifico");
        let mut raiz = pi.subraiz_pend;
        raiz[0] += BaseElement::ONE;
        for otro in [
            EdadPublicInputs { k: 4, ..pi.clone() },
            EdadPublicInputs { t: 71, ..pi.clone() },
            EdadPublicInputs { n: 12, ..pi.clone() },
            EdadPublicInputs { subraiz_pend: raiz, ..pi.clone() },
        ] {
            assert!(verificar(&bytes, &otro).is_err(), "verifico con {otro:?}");
        }
    }

    /// **E4b-2 (S465): la cabeza fija la marca y el `seq`.** Con las raices del libro la prueba
    /// se enlaza y el juez devuelve SU enunciado; con otra `nextPending` o con otro `seq` en la
    /// cabeza, no: el juez compone el enunciado con lo que la cabeza firma.
    #[test]
    #[cfg_attr(debug_assertions, ignore = "winterfell valida grados en depuracion: juez release")]
    fn la_cabeza_fija_la_marca_y_el_seq() {
        let (hojas, meta) = libro();
        let (bytes, pi) =
            probar(construir(&hojas, &meta, &en(70, 0, true)).expect("construir")).expect("probar");
        let cabeza = cabeza_del_libro(&hojas, &meta, 13, 130);
        let af = afirmacion(&pi);
        assert_eq!(verificar_contra_cabeza(&bytes, &af, &cabeza), Ok(pi.clone()));
        for otra in [CabezaEdad { next_pending: 12, ..cabeza }, CabezaEdad { seq: 131, ..cabeza }] {
            assert!(verificar_contra_cabeza(&bytes, &af, &otra).is_err(), "enlazo con {otra:?}");
        }
    }

    /// **D-2 de E4b-2 (S465): `m` se deriva de la marca.** Una prueba honesta del mismo libro con
    /// un subarbol de mas (`m` = 5 para `n` = 13) verifica sola, y sus subraices suben a las MISMAS
    /// raices; pero no se enlaza: el juez sube con la `m` minima y la subida ya no llega.
    #[test]
    #[cfg_attr(debug_assertions, ignore = "winterfell valida grados en depuracion: juez release")]
    fn una_m_de_mas_no_se_enlaza() {
        let (mut hojas, mut meta) = libro();
        hojas.resize(32, CERO);
        meta.resize(32, None);
        let e = en(70, 0, true);
        let celdas = celdas_del_libro(&hojas, &meta, &e).expect("celdas");
        let (bytes, pi) = probar(trazar(&celdas, 13, &e)).expect("probar");
        assert_eq!(pi.m, 5);
        assert!(verificar(&bytes, &pi).is_ok(), "la de m = 5 tenia que verificar sola");
        let (h13, m13) = libro();
        let cabeza = cabeza_del_libro(&h13, &m13, 13, 130);
        assert_eq!(raiz_desde_subraiz(pi.subraiz_pend, 5, PROFUNDIDAD), cabeza.pending_root);
        let af = afirmacion(&pi);
        assert!(verificar_contra_cabeza(&bytes, &af, &cabeza).is_err(), "una m de mas se enlazo");
    }
}
