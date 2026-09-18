//! **RFC-0008 E2: el PROBADOR del pago en curso portable.**
//!
//! El AIR vive en `zk-ssl-air` -el crate que el kit consume sin el probador- y aqui solo esta lo
//! que produce una prueba: la traza y el `Prover`. Lo que el AIR afirma, lo que NO prueba y lo
//! que es testigo estan en la cabecera de `zk_ssl_air::pago_en_curso`; aqui no se repiten.
//!
//! **La entrada es la apertura ENTERA que el pagador ya tiene** -`receptor`, `sal`, `importe`,
//! `refund_id` y `delta`, que `send_materials_v2` toma de el y `SendMaterials` guarda- mas los
//! dos caminos de la foto del latido y la meta de esa posicion (D-AE). Ninguna clave: el
//! enunciado es de estado, como en E1.
//!
//! **La diferencia con el cobro, en una linea**: alli el sobre `X` entraba por cuatro columnas de
//! testigo; aqui lo COMPONE el carril B en su ciclo 1 y el carril A lo lee de el en la fila 15
//! (D-AF). El pagador prueba asi que conoce la pareja `(refund_id, delta)` que abre el
//! compromiso, sin decir cual es.

use winterfell::crypto::hashers::{Blake3_256, Rp64_256};
use winterfell::crypto::{DefaultRandomCoin, MerkleTree};
use winterfell::math::{fields::f64::BaseElement, FieldElement};
use winterfell::matrix::ColMatrix;
use winterfell::{
    AuxRandElements, CompositionPoly, CompositionPolyTrace, ConstraintCompositionCoefficients,
    DefaultConstraintCommitment, DefaultConstraintEvaluator, DefaultTraceLde, PartitionOptions,
    ProofOptions, Prover, StarkDomain, TraceInfo, TracePolyTable, TraceTable,
};
use zk_ssl_hash::DOMINIO_META_PENDIENTE;

use crate::merkle::MerklePath;
use crate::rescue_hash::{NUM_ROUNDS, STATE_WIDTH};

pub use zk_ssl_air::pago_en_curso::{
    comprobar_enunciado, verificar, verificar_contra_cabeza, AfirmacionPago, CabezaPago,
    PagoEnCursoAir, PagoEnCursoPublicInputs, ANCHO, C_A, C_B, COL_BIT, COL_DELTA, COL_EMISOR,
    COL_IMPORTE, COL_NACIDO, COL_RECEPTOR, COL_REFUND, COL_SACC, COL_SAL, COL_SBIT, COL_T,
    CYC_ACC, CYC_META, CYC_RAIZ, CYC_X, LARGO_SEGMENTO, MAX_VALOR, NUM_ASERCIONES,
    NUM_RESTRICCIONES, ROW_ENLACE_IMPORTE, ROW_ENLACE_X, ROW_HOJA_LISTA, ROW_RAIZ, SEGMENTOS,
    TRAZA,
};
pub use zk_ssl_air::{opciones, Digest, CICLO, ESTADO, PROFUNDIDAD};

type Blake3 = Blake3_256<BaseElement>;

const _: () = assert!(NUM_ROUNDS == CICLO - 1);
const _: () = assert!(STATE_WIDTH == ESTADO);

fn bits_be(valor: u64) -> Vec<bool> {
    (0..LARGO_SEGMENTO)
        .map(|p| (valor >> (LARGO_SEGMENTO - 1 - p)) & 1 == 1)
        .collect()
}

/// **Lo que el pagador aporta.** El camino de pendientes lleva la posicion (sus bits); el de meta
/// solo aporta sus hermanos, porque la posicion es la MISMA. La pareja `(refund_id, delta)` es la
/// apertura del sobre, y no sale de aqui.
#[derive(Clone, Debug)]
pub struct PagoEnCursoWitness {
    pub receptor: Digest,
    pub sal: Digest,
    pub importe: u64,
    pub refund_id: Digest,
    pub delta: u64,
    pub emisor: u64,
    pub nacido: u64,
    pub camino_pendiente: MerklePath,
    pub hermanos_meta: Vec<Digest>,
}

/// Construye la traza. **No comprueba nada**: los testigos negativos le pasan material a mano
/// para que lo que tumbe la prueba sea el AIR y no esta funcion.
pub fn trazar(w: &PagoEnCursoWitness, t: u64) -> TraceTable<BaseElement> {
    trazar_con(w, t, &w.camino_pendiente.is_right, BaseElement::ZERO)
}

/// La traza con dos palancas que solo usan los testigos: la direccion con la que sube el carril B
/// (la honesta es la del carril A, y la columna del bit guarda SIEMPRE la del A) y un limbo del
/// rate del ciclo del importe (el honesto es cero).
fn trazar_con(
    w: &PagoEnCursoWitness,
    t: u64,
    bits_meta: &[bool],
    relleno: BaseElement,
) -> TraceTable<BaseElement> {
    let cero = BaseElement::ZERO;
    let c_importe = BaseElement::new(w.importe);
    let c_t = BaseElement::new(t);
    let c_delta = BaseElement::new(w.delta);
    let c_emisor = BaseElement::new(w.emisor);
    let c_nacido = BaseElement::new(w.nacido);

    let mut filas: Vec<Vec<BaseElement>> = vec![vec![cero; ANCHO]; TRAZA];
    for fila in filas.iter_mut() {
        fila[COL_RECEPTOR..COL_RECEPTOR + 4].copy_from_slice(&w.receptor);
        fila[COL_IMPORTE] = c_importe;
        fila[COL_T] = c_t;
        fila[COL_SAL..COL_SAL + 4].copy_from_slice(&w.sal);
        fila[COL_REFUND..COL_REFUND + 4].copy_from_slice(&w.refund_id);
        fila[COL_DELTA] = c_delta;
        fila[COL_EMISOR] = c_emisor;
        fila[COL_NACIDO] = c_nacido;
    }

    // El unico segmento: delta - (T - nacido), la cota temporal (D-AD).
    let cota = (c_delta - (c_t - c_nacido)).as_int();
    let bits = bits_be(cota);
    let mut acc = cero;
    for p in 0..LARGO_SEGMENTO {
        let bit = if bits[p] { BaseElement::ONE } else { cero };
        acc = if p == 0 { bit } else { acc + acc + bit };
        filas[p][COL_SBIT] = bit;
        filas[p][COL_SACC] = acc;
    }

    let sitio = |estado: &mut [BaseElement; ESTADO], dig: &Digest, herm: &Digest, der: bool| {
        if der {
            estado[4..8].copy_from_slice(herm);
            estado[8..ESTADO].copy_from_slice(dig);
        } else {
            estado[4..8].copy_from_slice(dig);
            estado[8..ESTADO].copy_from_slice(herm);
        }
    };

    let mut a = [cero; ESTADO];
    a[4..8].copy_from_slice(&w.receptor);
    a[8..ESTADO].copy_from_slice(&w.sal);
    // El carril B esta OCIOSO en el ciclo 0: arranca el sobre en la fila 7 y lo entrega en la 15.
    let mut b = [cero; ESTADO];
    filas[0][C_A..C_A + ESTADO].copy_from_slice(&a);
    filas[0][C_B..C_B + ESTADO].copy_from_slice(&b);

    for r in 0..ROW_RAIZ {
        let pos = r % CICLO;
        if pos < NUM_ROUNDS {
            Rp64_256::apply_round(&mut a, pos);
            if r >= CYC_X * CICLO {
                Rp64_256::apply_round(&mut b, pos);
            }
        } else {
            let da: Digest = [a[4], a[5], a[6], a[7]];
            let db: Digest = [b[4], b[5], b[6], b[7]];
            a = [cero; ESTADO];
            b = [cero; ESTADO];
            match r {
                ROW_ENLACE_IMPORTE => {
                    a[4..8].copy_from_slice(&da);
                    a[8] = c_importe;
                    a[9] = relleno;
                    b[4..8].copy_from_slice(&w.refund_id);
                    b[8] = c_delta;
                }
                ROW_ENLACE_X => {
                    a[4..8].copy_from_slice(&da);
                    a[8..ESTADO].copy_from_slice(&db);
                    b[0] = BaseElement::new(DOMINIO_META_PENDIENTE);
                    b[4] = c_emisor;
                    b[5] = c_nacido;
                }
                _ => {
                    let sig = (r + 1) / CICLO;
                    if (CYC_ACC..CYC_RAIZ).contains(&sig) {
                        let n = sig - CYC_ACC;
                        let cp = &w.camino_pendiente;
                        sitio(&mut a, &da, &cp.siblings[n], cp.is_right[n]);
                        sitio(&mut b, &db, &w.hermanos_meta[n], bits_meta[n]);
                    }
                }
            }
        }
        filas[r + 1][C_A..C_A + ESTADO].copy_from_slice(&a);
        filas[r + 1][C_B..C_B + ESTADO].copy_from_slice(&b);
    }
    debug_assert_eq!(ROW_HOJA_LISTA, CYC_ACC * CICLO - 1);

    for nivel in 0..PROFUNDIDAD {
        let bit = if w.camino_pendiente.is_right[nivel] { BaseElement::ONE } else { cero };
        for p in 0..CICLO {
            filas[(CYC_ACC + nivel) * CICLO + p][COL_BIT] = bit;
        }
    }

    let mut traza = TraceTable::new(ANCHO, TRAZA);
    traza.fill(
        |s| s.copy_from_slice(&filas[0]),
        |paso, s| s.copy_from_slice(&filas[paso + 1]),
    );
    traza
}

pub struct PagoEnCursoProver {
    options: ProofOptions,
}

impl PagoEnCursoProver {
    pub fn new(options: ProofOptions) -> Self {
        Self { options }
    }
}

/// **Prueba con las opciones de la casa**, las UNICAS que el juez acepta.
pub fn probar(
    traza: TraceTable<BaseElement>,
) -> Result<(Vec<u8>, PagoEnCursoPublicInputs), String> {
    probar_con(traza, opciones())
}

/// Prueba con OTRAS opciones: solo para el testigo de que el juez no las acepta.
pub fn probar_con(
    traza: TraceTable<BaseElement>,
    opts: ProofOptions,
) -> Result<(Vec<u8>, PagoEnCursoPublicInputs), String> {
    let prover = PagoEnCursoProver::new(opts);
    let pi = prover.get_pub_inputs(&traza);
    let prueba = prover.prove(traza).map_err(|e| format!("{e:?}"))?;
    Ok((prueba.to_bytes(), pi))
}

impl Prover for PagoEnCursoProver {
    type BaseField = BaseElement;
    type Air = PagoEnCursoAir;
    type Trace = TraceTable<BaseElement>;
    type HashFn = Blake3;
    type VC = MerkleTree<Blake3>;
    type RandomCoin = DefaultRandomCoin<Blake3>;
    type TraceLde<E: FieldElement<BaseField = Self::BaseField>> =
        DefaultTraceLde<E, Self::HashFn, Self::VC>;
    type ConstraintEvaluator<'a, E: FieldElement<BaseField = Self::BaseField>> =
        DefaultConstraintEvaluator<'a, Self::Air, E>;
    type ConstraintCommitment<E: FieldElement<BaseField = Self::BaseField>> =
        DefaultConstraintCommitment<E, Self::HashFn, Self::VC>;

    /// **Las entradas publicas DERIVADAS de la traza.** Lo que impide declarar otras son las
    /// aserciones del AIR, que el verificador comprueba contra las que EL declara.
    fn get_pub_inputs(&self, trace: &Self::Trace) -> PagoEnCursoPublicInputs {
        let dig = |base: usize, fila: usize| -> Digest {
            [
                trace.get(base, fila),
                trace.get(base + 1, fila),
                trace.get(base + 2, fila),
                trace.get(base + 3, fila),
            ]
        };
        PagoEnCursoPublicInputs {
            pending_root: dig(C_A + 4, ROW_RAIZ),
            pmeta_root: dig(C_B + 4, ROW_RAIZ),
            receptor: dig(COL_RECEPTOR, 0),
            importe: trace.get(COL_IMPORTE, 0),
            t: trace.get(COL_T, 0),
            nacido: trace.get(COL_NACIDO, 0),
        }
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::merkle::native_merge;
    use crate::native::native_climb;
    use winterfell::Air;
    use zk_ssl_hash::meta_pendiente_hoja;

    const P_IMPORTE: u64 = 250_000;
    const P_EMISOR: u64 = 3;
    const P_NACIDO: u64 = 1040;
    const P_DELTA: u64 = 96;
    /// El instante que el pagador afirma: cabe, porque `T - nacido = 60 <= delta`.
    const P_T: u64 = 1100;
    /// La frontera exacta: `T = nacido + delta`.
    const T_FRONTERA: u64 = P_NACIDO + P_DELTA;
    const Q_EMISOR: u64 = 4;
    const Q_NACIDO: u64 = 1077;

    fn dg(k: u64) -> Digest {
        [
            BaseElement::new(k),
            BaseElement::new(k.wrapping_mul(5) + 1),
            BaseElement::new(k.wrapping_mul(13) + 2),
            BaseElement::new(k.wrapping_mul(17) + 3),
        ]
    }

    fn embebe(v: u64) -> Digest {
        [BaseElement::new(v), BaseElement::ZERO, BaseElement::ZERO, BaseElement::ZERO]
    }

    /// El sobre, en nativo: `refund_envelope` de la capa.
    fn sobre(refund_id: Digest, delta: u64) -> Digest {
        native_merge(refund_id, embebe(delta))
    }

    /// La hoja v2 compuesta en nativo, como `pending_commitment_v2` de la capa.
    fn hoja_v2(receptor: Digest, sal: Digest, importe: u64, x: Digest) -> Digest {
        native_merge(native_merge(native_merge(receptor, sal), embebe(importe)), x)
    }

    /// El escenario: el pendiente vive en `p` (bit 0 a la izquierda) y la meta de `p` y la de su
    /// vecina `q` son hermanas en el nivel 0 del arbol de meta. Por encima, hermanos DISTINTOS en
    /// los dos arboles y direcciones MIXTAS (con todas iguales la traza degenera).
    struct Escenario {
        w: PagoEnCursoWitness,
        pending_root: Digest,
        pmeta_root: Digest,
        hoja_meta_q: Digest,
        hermanos_meta_q: Vec<Digest>,
        bits_q: Vec<bool>,
    }

    fn escenario() -> Escenario {
        let receptor = dg(0xB0B);
        let sal = dg(0x5A17);
        let refund_id = dg(0xA11CE);
        let x = sobre(refund_id, P_DELTA);
        let hoja_p = hoja_v2(receptor, sal, P_IMPORTE, x);
        let meta_p = meta_pendiente_hoja(P_EMISOR, P_NACIDO);
        let meta_q = meta_pendiente_hoja(Q_EMISOR, Q_NACIDO);

        let mut bits = vec![false];
        let mut hermanos_p = vec![[BaseElement::ZERO; 4]];
        let mut hermanos_m = vec![meta_q];
        for nivel in 1..PROFUNDIDAD {
            bits.push(nivel % 3 == 0);
            hermanos_p.push(dg(100 + nivel as u64));
            hermanos_m.push(dg(900 + nivel as u64));
        }
        let camino_pendiente = MerklePath { siblings: hermanos_p, is_right: bits.clone() };
        let pending_root = native_climb(hoja_p, &camino_pendiente);
        let camino_meta_p = MerklePath { siblings: hermanos_m.clone(), is_right: bits.clone() };
        let pmeta_root = native_climb(meta_p, &camino_meta_p);

        let mut bits_q = bits;
        bits_q[0] = true;
        let mut hermanos_meta_q = hermanos_m.clone();
        hermanos_meta_q[0] = meta_p;
        let camino_meta_q =
            MerklePath { siblings: hermanos_meta_q.clone(), is_right: bits_q.clone() };
        assert_eq!(native_climb(meta_q, &camino_meta_q), pmeta_root, "q sube con lo suyo");

        Escenario {
            w: PagoEnCursoWitness {
                receptor,
                sal,
                importe: P_IMPORTE,
                refund_id,
                delta: P_DELTA,
                emisor: P_EMISOR,
                nacido: P_NACIDO,
                camino_pendiente,
                hermanos_meta: hermanos_m,
            },
            pending_root,
            pmeta_root,
            hoja_meta_q: meta_q,
            hermanos_meta_q,
            bits_q,
        }
    }

    fn pi(
        e: &Escenario,
        receptor: Digest,
        importe: u64,
        t: u64,
        nacido: u64,
    ) -> PagoEnCursoPublicInputs {
        PagoEnCursoPublicInputs {
            pending_root: e.pending_root,
            pmeta_root: e.pmeta_root,
            receptor,
            importe: BaseElement::new(importe),
            t: BaseElement::new(t),
            nacido: BaseElement::new(nacido),
        }
    }

    /// Produce y JUZGA con el juez del kit: lo que decide es `verificar`, no un `verify` propio.
    fn juzga(traza: TraceTable<BaseElement>, declarado: &PagoEnCursoPublicInputs) -> bool {
        let prover = PagoEnCursoProver::new(opciones());
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| prover.prove(traza)));
        match r {
            Err(_) | Ok(Err(_)) => false,
            Ok(Ok(prueba)) => verificar(&prueba.to_bytes(), declarado).is_ok(),
        }
    }

    /// **El positivo**: el pagador de `p` prueba <<pague 250.000 a este receptor, nacido en 1040,
    /// y no es reversible antes de 1100>>.
    #[test]
    fn el_positivo_verifica() {
        let e = escenario();
        let declarado = pi(&e, e.w.receptor, P_IMPORTE, P_T, P_NACIDO);
        assert!(juzga(trazar(&e.w, P_T), &declarado));
    }

    /// **Puntos de referencia**: el sobre sale del carril B en la fila 15, la hoja de pendientes y
    /// la de meta son las nativas, y las entradas DERIVADAS son las DECLARADAS. No produce prueba.
    #[test]
    fn los_puntos_de_referencia_espejan_el_nativo() {
        let e = escenario();
        let traza = trazar(&e.w, P_T);
        let x = sobre(e.w.refund_id, P_DELTA);
        let hoja = hoja_v2(e.w.receptor, e.w.sal, P_IMPORTE, x);
        let meta = meta_pendiente_hoja(P_EMISOR, P_NACIDO);
        for i in 0..4 {
            assert_eq!(traza.get(C_B + 4 + i, ROW_ENLACE_X), x[i], "el sobre en el carril B");
            assert_eq!(traza.get(C_A + 4 + i, ROW_HOJA_LISTA), hoja[i], "hoja de pendientes");
            assert_eq!(traza.get(C_B + 4 + i, ROW_HOJA_LISTA), meta[i], "hoja de meta");
        }
        let derivadas = PagoEnCursoProver::new(opciones()).get_pub_inputs(&traza);
        assert_eq!(derivadas, pi(&e, e.w.receptor, P_IMPORTE, P_T, P_NACIDO));
    }

    /// **Negativo 1 (D-H heredado): una meta de otra posicion no verifica.** El carril B sube la
    /// meta de `q` con SUS hermanos y SUS bits mientras la columna del bit guarda los de `p`. Cae
    /// porque solo hay UNA columna de bit.
    #[test]
    fn una_meta_de_otra_posicion_no_verifica() {
        let e = escenario();
        let mut w = e.w.clone();
        w.emisor = Q_EMISOR;
        w.nacido = Q_NACIDO;
        w.hermanos_meta = e.hermanos_meta_q.clone();
        assert_eq!(meta_pendiente_hoja(w.emisor, w.nacido), e.hoja_meta_q);
        let traza = trazar_con(&w, P_T, &e.bits_q, BaseElement::ZERO);
        let declarado = pi(&e, e.w.receptor, P_IMPORTE, P_T, Q_NACIDO);
        assert!(!juzga(traza, &declarado), "CRITICO: la meta de otra posicion verifico");
    }

    /// **Negativo 2 (D-AF): el sobre lo compone el carril B, y el par ATA.** Otro `refund_id` u
    /// otro `delta` dan otra hoja, luego otra raiz, y la prueba no verifica contra la declarada.
    #[test]
    fn el_sobre_lo_compone_el_carril_b() {
        let e = escenario();
        let declarado = pi(&e, e.w.receptor, P_IMPORTE, P_T, P_NACIDO);
        let x = sobre(e.w.refund_id, P_DELTA);
        assert_ne!(sobre(dg(0xBAD), P_DELTA), x, "otro refund_id tenia que mover el sobre");
        assert_ne!(sobre(e.w.refund_id, P_DELTA + 1), x, "otro delta tenia que moverlo");

        let mut otro_id = e.w.clone();
        otro_id.refund_id = dg(0xBAD);
        assert!(!juzga(trazar(&otro_id, P_T), &declarado), "CRITICO: el refund_id no ata");

        let mut otro_delta = e.w.clone();
        otro_delta.delta = P_DELTA + 1;
        assert!(!juzga(trazar(&otro_delta, P_T), &declarado), "CRITICO: el delta no ata");
    }

    /// **Negativo 3, por los DOS lados: la frontera de T.** Con `T = nacido + delta` verifica; con
    /// un paso mas la resta da la vuelta, el valor pasa de `2^63` y la cota rechaza.
    #[test]
    fn la_frontera_de_t_se_prueba_por_los_dos_lados() {
        let e = escenario();
        let justo = pi(&e, e.w.receptor, P_IMPORTE, T_FRONTERA, P_NACIDO);
        assert!(juzga(trazar(&e.w, T_FRONTERA), &justo), "T = nacido + delta tenia que valer");
        let pasado = pi(&e, e.w.receptor, P_IMPORTE, T_FRONTERA + 1, P_NACIDO);
        assert!(
            !juzga(trazar(&e.w, T_FRONTERA + 1), &pasado),
            "CRITICO: un T por encima de nacido + delta probo la cota"
        );
    }

    /// **Negativo 4: otro receptor declarado no verifica** (las cuatro aserciones del receptor).
    #[test]
    fn otro_receptor_no_verifica() {
        let e = escenario();
        let declarado = pi(&e, dg(0x1337), P_IMPORTE, P_T, P_NACIDO);
        assert!(!juzga(trazar(&e.w, P_T), &declarado), "CRITICO: el receptor no ata");
    }

    /// **Negativo 5: otro importe declarado no verifica.** El importe es EXACTO y publico (D-AD):
    /// no hay banda que absorba.
    #[test]
    fn otro_importe_declarado_no_verifica() {
        let e = escenario();
        for otro in [P_IMPORTE - 1, P_IMPORTE + 1] {
            let declarado = pi(&e, e.w.receptor, otro, P_T, P_NACIDO);
            assert!(!juzga(trazar(&e.w, P_T), &declarado), "CRITICO: el importe {otro} paso");
        }
    }

    /// **Negativo 6: otro nacido declarado no verifica** (la asercion del nacido).
    #[test]
    fn otro_nacido_no_verifica() {
        let e = escenario();
        let declarado = pi(&e, e.w.receptor, P_IMPORTE, P_T, P_NACIDO + 1);
        assert!(!juzga(trazar(&e.w, P_T), &declarado), "CRITICO: el nacido no ata");
    }

    /// **Negativo 7: un relleno distinto de cero en el ciclo del importe no verifica**, aunque las
    /// raices declaradas sean las que esa traza alcanza. Lo que cae es el atado del rate entero.
    #[test]
    fn un_relleno_en_el_ciclo_del_importe_no_verifica() {
        let e = escenario();
        let bits = e.w.camino_pendiente.is_right.clone();
        let traza = trazar_con(&e.w, P_T, &bits, BaseElement::ONE);
        let declarado = PagoEnCursoProver::new(opciones()).get_pub_inputs(&traza);
        assert_ne!(declarado.pending_root, e.pending_root, "el relleno tenia que mover la hoja");
        assert_eq!(declarado.pmeta_root, e.pmeta_root);
        assert!(!juzga(traza, &declarado), "CRITICO: el rate del importe no esta atado");
    }

    /// **Negativo 8 (D-AD): el plazo no esta en el enunciado.** La prueba verifica y ni el `delta`
    /// ni un limbo del `refund_id` entran al transcripto.
    #[test]
    fn el_delta_no_esta_en_las_entradas_publicas() {
        let e = escenario();
        let declarado = pi(&e, e.w.receptor, P_IMPORTE, P_T, P_NACIDO);
        let v = winterfell::math::ToElements::to_elements(&declarado);
        assert_eq!(v.len(), 15);
        assert!(!v.contains(&BaseElement::new(P_DELTA)), "el delta en el transcripto");
        assert!(e.w.refund_id.iter().all(|xi| !v.contains(xi)), "un limbo del refund_id");
        let (bytes, derivado) = probar(trazar(&e.w, P_T)).expect("probar");
        assert_eq!(derivado, declarado);
        assert!(verificar(&bytes, &declarado).is_ok());
    }

    fn cabeza(e: &Escenario, seq: u64) -> CabezaPago {
        CabezaPago { seq, pending_root: e.pending_root, pmeta_root: e.pmeta_root }
    }

    fn afirma(t: u64) -> AfirmacionPago {
        AfirmacionPago {
            receptor: dg(0xB0B),
            importe: P_IMPORTE,
            t,
            nacido: P_NACIDO,
        }
    }

    /// **El enlace: la cabeza fija las dos raices, y el nacido va antes que ella.** Con las raices
    /// del escenario y un `seq` posterior al nacido la prueba se enlaza y el juez devuelve SU
    /// enunciado; con otra raiz, otra meta o un `seq` que no es posterior, no.
    #[test]
    fn la_cabeza_fija_las_raices_y_el_nacido_va_antes() {
        let e = escenario();
        let (bytes, pi) = probar(trazar(&e.w, P_T)).expect("probar");
        let buena = cabeza(&e, P_NACIDO + 1);
        assert_eq!(verificar_contra_cabeza(&bytes, &afirma(P_T), &buena), Ok(pi));
        let mut otra = e.pending_root;
        otra[0] += BaseElement::ONE;
        let mut otra_meta = e.pmeta_root;
        otra_meta[1] += BaseElement::ONE;
        for mala in [
            CabezaPago { pending_root: otra, ..buena },
            CabezaPago { pmeta_root: otra_meta, ..buena },
            cabeza(&e, P_NACIDO),
        ] {
            let r = verificar_contra_cabeza(&bytes, &afirma(P_T), &mala);
            assert!(r.is_err(), "enlazo con {mala:?}");
        }
    }

    /// **PRUEBA POR MUTACION: ninguna restriccion esta vacia**, en TODAS las filas.
    #[test]
    fn ninguna_restriccion_esta_vacia() {
        use crate::mutation::{buscar_vacias, rows_of};
        let e = escenario();
        let traza = trazar(&e.w, P_T);
        let filas = rows_of(&traza, ANCHO, TRAZA);
        let air = PagoEnCursoAir::new(
            TraceInfo::new(ANCHO, TRAZA),
            pi(&e, e.w.receptor, P_IMPORTE, P_T, P_NACIDO),
            opciones(),
        );
        let informe = buscar_vacias(&air, &filas, 1);
        assert!(
            informe.nunca_disparadas.is_empty(),
            "restricciones que NINGUNA perturbacion activa (de {} totales, {} celdas): {:?}",
            informe.total,
            informe.celdas,
            informe.nunca_disparadas
        );
    }

    /// El juez acepta UN conjunto de opciones y solo uno.
    #[test]
    fn otras_opciones_no_se_aceptan() {
        use winterfell::{BatchingMethod, FieldExtension};
        let e = escenario();
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
        let (bytes, pi) = probar_con(trazar(&e.w, P_T), otras).expect("probar");
        assert_eq!(pi.pending_root, e.pending_root, "el escenario no es el que se cree");
        assert!(verificar(&bytes, &pi).is_err(), "el juez acepto otras opciones");
    }

    /// La geometria que este probador escribe es la que el AIR declara.
    #[test]
    fn la_geometria_del_probador_es_la_del_air() {
        assert_eq!((ANCHO, TRAZA, SEGMENTOS), (44, 512, 1));
        assert_eq!((CYC_X, CYC_META, CYC_ACC, CYC_RAIZ), (1, 2, 3, 35));
        assert_eq!(NUM_RESTRICCIONES, 120);
        assert_eq!(NUM_ASERCIONES, 19);
        assert_eq!(MAX_VALOR, 0x3fffffffffffffff);
        assert_eq!(ROW_ENLACE_IMPORTE + 1, CYC_X * CICLO);
        assert!(SEGMENTOS * LARGO_SEGMENTO < ROW_RAIZ);
        assert_eq!(COL_SACC - COL_SBIT, 1);
    }
}
