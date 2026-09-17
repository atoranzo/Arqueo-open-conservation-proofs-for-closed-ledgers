//! **RFC-0008 E1: el PROBADOR del cobro pendiente portable.**
//!
//! El AIR vive en `zk-ssl-air` -el crate que el kit consume sin el probador- y aqui solo esta lo
//! que produce una prueba: la traza y el `Prover`. Lo que el AIR afirma, lo que NO prueba y lo
//! que es testigo estan en la cabecera de `zk_ssl_air::cobro_pendiente`; aqui no se repiten.
//!
//! **La entrada es lo que el aviso le da al cobrador** -`receptor`, `sal`, `importe`, el sobre
//! `X` opaco- mas los dos caminos de la foto del latido y la meta de esa posicion (D-F). Ninguna
//! clave: el enunciado es de estado (D-G). Los testigos de punta a punta viven AQUI: `zk-ssl-air`
//! no compila al probador. Es el reparto del S463.

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

pub use zk_ssl_air::cobro_pendiente::{
    comprobar_enunciado, verificar, CobroPendienteAir, CobroPendientePublicInputs, ANCHO, C_A,
    C_B, COL_BIT, COL_EMISOR, COL_IMPORTE, COL_INFERIOR, COL_NACIDO, COL_RECEPTOR, COL_SACC,
    COL_SAL, COL_SBIT, COL_SUPERIOR, COL_X, CYC_ACC, CYC_META, CYC_RAIZ, LARGO_SEGMENTO,
    MAX_VALOR, NUM_ASERCIONES, NUM_RESTRICCIONES, ROW_ENLACE_IMPORTE, ROW_ENLACE_X,
    ROW_HOJA_LISTA, ROW_RAIZ, SEGMENTOS, TRAZA,
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

/// **Lo que el cobrador aporta.** El camino de pendientes lleva la posicion (sus bits); el de
/// meta solo aporta sus hermanos, porque la posicion es la MISMA (D-H).
#[derive(Clone, Debug)]
pub struct CobroPendienteWitness {
    pub receptor: Digest,
    pub sal: Digest,
    pub importe: u64,
    pub x: Digest,
    pub emisor: u64,
    pub nacido: u64,
    pub camino_pendiente: MerklePath,
    pub hermanos_meta: Vec<Digest>,
}

/// Construye la traza. **No comprueba nada**: los testigos negativos le pasan material a mano
/// para que lo que tumbe la prueba sea el AIR y no esta funcion.
pub fn trazar(
    w: &CobroPendienteWitness,
    inferior: u64,
    superior: u64,
) -> TraceTable<BaseElement> {
    trazar_con(w, inferior, superior, &w.camino_pendiente.is_right, BaseElement::ZERO)
}

/// La traza con dos palancas que solo usan los testigos: la direccion con la que sube el carril
/// B (la honesta es la del carril A, y la columna del bit guarda SIEMPRE la del A) y un limbo del
/// rate del ciclo del importe (el honesto es cero).
fn trazar_con(
    w: &CobroPendienteWitness,
    inferior: u64,
    superior: u64,
    bits_meta: &[bool],
    relleno: BaseElement,
) -> TraceTable<BaseElement> {
    let cero = BaseElement::ZERO;
    let c_importe = BaseElement::new(w.importe);
    let c_inferior = BaseElement::new(inferior);
    let c_superior = BaseElement::new(superior);
    let c_emisor = BaseElement::new(w.emisor);
    let c_nacido = BaseElement::new(w.nacido);

    let mut filas: Vec<Vec<BaseElement>> = vec![vec![cero; ANCHO]; TRAZA];
    for fila in filas.iter_mut() {
        fila[COL_RECEPTOR..COL_RECEPTOR + 4].copy_from_slice(&w.receptor);
        fila[COL_IMPORTE] = c_importe;
        fila[COL_INFERIOR] = c_inferior;
        fila[COL_SUPERIOR] = c_superior;
        fila[COL_SAL..COL_SAL + 4].copy_from_slice(&w.sal);
        fila[COL_X..COL_X + 4].copy_from_slice(&w.x);
        fila[COL_EMISOR] = c_emisor;
        fila[COL_NACIDO] = c_nacido;
    }

    // Rangos: importe, importe - inferior, superior - importe (el molde de la banda).
    let segmentos = [
        c_importe.as_int(),
        (c_importe - c_inferior).as_int(),
        (c_superior - c_importe).as_int(),
    ];
    for (seg, valor) in segmentos.iter().enumerate() {
        let bits = bits_be(*valor);
        let mut acc = cero;
        for p in 0..LARGO_SEGMENTO {
            let r = seg * LARGO_SEGMENTO + p;
            let bit = if bits[p] { BaseElement::ONE } else { cero };
            acc = if p == 0 { bit } else { acc + acc + bit };
            filas[r][COL_SBIT] = bit;
            filas[r][COL_SACC] = acc;
        }
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
    // El carril B no hashea en los ciclos 0 y 1: se queda en cero hasta su enlace.
    let mut b = [cero; ESTADO];
    filas[0][C_A..C_A + ESTADO].copy_from_slice(&a);
    filas[0][C_B..C_B + ESTADO].copy_from_slice(&b);

    for r in 0..ROW_RAIZ {
        let pos = r % CICLO;
        if pos < NUM_ROUNDS {
            Rp64_256::apply_round(&mut a, pos);
            if r >= CYC_META * CICLO {
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
                }
                ROW_ENLACE_X => {
                    a[4..8].copy_from_slice(&da);
                    a[8..ESTADO].copy_from_slice(&w.x);
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

pub struct CobroPendienteProver {
    options: ProofOptions,
}

impl CobroPendienteProver {
    pub fn new(options: ProofOptions) -> Self {
        Self { options }
    }
}

/// **Prueba con las opciones de la casa**, las UNICAS que el juez acepta.
pub fn probar(
    traza: TraceTable<BaseElement>,
) -> Result<(Vec<u8>, CobroPendientePublicInputs), String> {
    probar_con(traza, opciones())
}

/// Prueba con OTRAS opciones: solo para el testigo de que el juez no las acepta.
pub fn probar_con(
    traza: TraceTable<BaseElement>,
    opts: ProofOptions,
) -> Result<(Vec<u8>, CobroPendientePublicInputs), String> {
    let prover = CobroPendienteProver::new(opts);
    let pi = prover.get_pub_inputs(&traza);
    let prueba = prover.prove(traza).map_err(|e| format!("{e:?}"))?;
    Ok((prueba.to_bytes(), pi))
}

impl Prover for CobroPendienteProver {
    type BaseField = BaseElement;
    type Air = CobroPendienteAir;
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
    fn get_pub_inputs(&self, trace: &Self::Trace) -> CobroPendientePublicInputs {
        let dig = |base: usize, fila: usize| -> Digest {
            [
                trace.get(base, fila),
                trace.get(base + 1, fila),
                trace.get(base + 2, fila),
                trace.get(base + 3, fila),
            ]
        };
        CobroPendientePublicInputs {
            pending_root: dig(C_A + 4, ROW_RAIZ),
            pmeta_root: dig(C_B + 4, ROW_RAIZ),
            receptor: dig(COL_RECEPTOR, 0),
            nacido: trace.get(COL_NACIDO, 0),
            inferior: trace.get(COL_INFERIOR, 0),
            superior: trace.get(COL_SUPERIOR, 0),
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

    /// La hoja v2 compuesta en nativo, como `pending_commitment_v2` de la capa.
    fn hoja_v2(receptor: Digest, sal: Digest, importe: u64, x: Digest) -> Digest {
        native_merge(native_merge(native_merge(receptor, sal), embebe(importe)), x)
    }

    /// El escenario: el pendiente vive en `p` (bit 0 a la izquierda) y la meta de `p` y la de su
    /// vecina `q` son hermanas en el nivel 0 del arbol de meta. Por encima, hermanos DISTINTOS en
    /// los dos arboles y direcciones MIXTAS (con todas iguales la traza degenera).
    /// `X` es la de los tres productores del arbol: el `public_id` de quien paga y `delta = 96`.
    struct Escenario {
        w: CobroPendienteWitness,
        pending_root: Digest,
        pmeta_root: Digest,
        hoja_meta_q: Digest,
        hermanos_meta_q: Vec<Digest>,
        bits_q: Vec<bool>,
    }

    fn escenario() -> Escenario {
        let receptor = dg(0xB0B);
        let sal = dg(0x5A17);
        let pagador = dg(0xA11CE);
        let x = native_merge(pagador, embebe(96));
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
            w: CobroPendienteWitness {
                receptor,
                sal,
                importe: P_IMPORTE,
                x,
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
        nacido: u64,
        inf: u64,
        sup: u64,
    ) -> CobroPendientePublicInputs {
        CobroPendientePublicInputs {
            pending_root: e.pending_root,
            pmeta_root: e.pmeta_root,
            receptor,
            nacido: BaseElement::new(nacido),
            inferior: BaseElement::new(inf),
            superior: BaseElement::new(sup),
        }
    }

    /// Produce y JUZGA con el juez del kit: lo que decide es `verificar`, no un `verify` propio.
    fn juzga(traza: TraceTable<BaseElement>, declarado: &CobroPendientePublicInputs) -> bool {
        let prover = CobroPendienteProver::new(opciones());
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| prover.prove(traza)));
        match r {
            Err(_) | Ok(Err(_)) => false,
            Ok(Ok(prueba)) => verificar(&prueba.to_bytes(), declarado).is_ok(),
        }
    }

    /// **El positivo**: el cobrador de `p` prueba <<a mi nombre, al menos 100, nacido en 1040>>.
    #[test]
    fn el_positivo_verifica() {
        let e = escenario();
        let declarado = pi(&e, e.w.receptor, P_NACIDO, 100, MAX_VALOR);
        assert!(juzga(trazar(&e.w, 100, MAX_VALOR), &declarado));
    }

    /// **Puntos de referencia**: la hoja de pendientes, la de meta y las dos raices de la traza
    /// son las nativas, y las entradas DERIVADAS son las DECLARADAS. No produce prueba.
    #[test]
    fn los_puntos_de_referencia_espejan_el_nativo() {
        let e = escenario();
        let traza = trazar(&e.w, 100, MAX_VALOR);
        let hoja = hoja_v2(e.w.receptor, e.w.sal, P_IMPORTE, e.w.x);
        let meta = meta_pendiente_hoja(P_EMISOR, P_NACIDO);
        for i in 0..4 {
            assert_eq!(traza.get(C_A + 4 + i, ROW_HOJA_LISTA), hoja[i], "hoja de pendientes");
            assert_eq!(traza.get(C_B + 4 + i, ROW_HOJA_LISTA), meta[i], "hoja de meta");
        }
        let derivadas = CobroPendienteProver::new(opciones()).get_pub_inputs(&traza);
        assert_eq!(derivadas, pi(&e, e.w.receptor, P_NACIDO, 100, MAX_VALOR));
    }

    /// **Negativo 1 (D-H, el del instrumento en el AIR): una meta de otra posicion no verifica.**
    /// El carril B sube la meta de `q` con SUS hermanos y SUS bits -que son los que llevan a la
    /// raiz de meta de verdad- mientras la columna del bit guarda los de `p`. Cae porque solo hay
    /// UNA columna de bit: es justo el atado que a `COL_FBIT` le falta (5.A-272).
    #[test]
    fn una_meta_de_otra_posicion_no_verifica() {
        let e = escenario();
        let mut w = e.w.clone();
        w.emisor = Q_EMISOR;
        w.nacido = Q_NACIDO;
        w.hermanos_meta = e.hermanos_meta_q.clone();
        assert_eq!(meta_pendiente_hoja(w.emisor, w.nacido), e.hoja_meta_q);
        let traza = trazar_con(&w, 100, MAX_VALOR, &e.bits_q, BaseElement::ZERO);
        let declarado = pi(&e, e.w.receptor, Q_NACIDO, 100, MAX_VALOR);
        assert!(!juzga(traza, &declarado), "CRITICO: la meta de otra posicion verifico");
    }

    /// **Negativo 2, por los DOS lados: el suelo.** Con `inferior == importe` verifica; con
    /// `inferior == importe + 1` la resta da la vuelta y la banda rechaza.
    #[test]
    fn el_suelo_se_prueba_por_los_dos_lados() {
        let e = escenario();
        let justo = pi(&e, e.w.receptor, P_NACIDO, P_IMPORTE, MAX_VALOR);
        assert!(juzga(trazar(&e.w, P_IMPORTE, MAX_VALOR), &justo), "el importe en el suelo");
        let pasado = pi(&e, e.w.receptor, P_NACIDO, P_IMPORTE + 1, MAX_VALOR);
        assert!(
            !juzga(trazar(&e.w, P_IMPORTE + 1, MAX_VALOR), &pasado),
            "CRITICO: un importe bajo el inferior probo la banda"
        );
    }

    /// **Negativo 3: otro receptor declarado no verifica.** La traza es la honesta; quien verifica
    /// declara otra identidad. Cae por las cuatro aserciones del receptor.
    #[test]
    fn otro_receptor_no_verifica() {
        let e = escenario();
        let declarado = pi(&e, dg(0x1337), P_NACIDO, 100, MAX_VALOR);
        let traza = trazar(&e.w, 100, MAX_VALOR);
        assert!(!juzga(traza, &declarado), "CRITICO: el receptor no ata");
    }

    /// **Negativo 4: otro nacido declarado no verifica** (la asercion del nacido).
    #[test]
    fn otro_nacido_no_verifica() {
        let e = escenario();
        let declarado = pi(&e, e.w.receptor, P_NACIDO + 1, 100, MAX_VALOR);
        let traza = trazar(&e.w, 100, MAX_VALOR);
        assert!(!juzga(traza, &declarado), "CRITICO: el nacido no ata");
    }

    /// **Negativo 5 (asiento 489): un relleno distinto de cero en el ciclo del importe no
    /// verifica**, aunque las raices declaradas sean las que esa traza alcanza. Lo unico que cae
    /// es el atado del rate entero, que la banda no tiene.
    #[test]
    fn un_relleno_en_el_ciclo_del_importe_no_verifica() {
        let e = escenario();
        let bits = e.w.camino_pendiente.is_right.clone();
        let traza = trazar_con(&e.w, 100, MAX_VALOR, &bits, BaseElement::ONE);
        let declarado = CobroPendienteProver::new(opciones()).get_pub_inputs(&traza);
        assert_ne!(declarado.pending_root, e.pending_root, "el relleno tenia que mover la hoja");
        assert_eq!(declarado.pmeta_root, e.pmeta_root);
        assert!(!juzga(traza, &declarado), "CRITICO: el rate del importe no esta atado");
    }

    /// **Negativo 6 (D-I): `X` no esta en el enunciado.** Con la `X` adivinable de los tres
    /// productores del arbol, la prueba verifica y ningun elemento de `X` entra al transcripto;
    /// otra `X` en otro arbol da una prueba que verifica con un enunciado de la misma forma.
    #[test]
    fn x_no_esta_en_las_entradas_publicas() {
        let e = escenario();
        let declarado = pi(&e, e.w.receptor, P_NACIDO, 100, MAX_VALOR);
        let v = winterfell::math::ToElements::to_elements(&declarado);
        assert_eq!(v.len(), 15);
        assert!(e.w.x.iter().all(|xi| !v.contains(xi)), "un elemento de X en el transcripto");
        let (bytes, derivado) = probar(trazar(&e.w, 100, MAX_VALOR)).expect("probar");
        assert_eq!(derivado, declarado);
        assert!(verificar(&bytes, &declarado).is_ok());
    }

    /// **PRUEBA POR MUTACION: ninguna restriccion esta vacia**, en TODAS las filas.
    #[test]
    fn ninguna_restriccion_esta_vacia() {
        use crate::mutation::{buscar_vacias, rows_of};
        let e = escenario();
        let traza = trazar(&e.w, 100, MAX_VALOR);
        let filas = rows_of(&traza, ANCHO, TRAZA);
        let air = CobroPendienteAir::new(
            TraceInfo::new(ANCHO, TRAZA),
            pi(&e, e.w.receptor, P_NACIDO, 100, MAX_VALOR),
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
        let (bytes, pi) = probar_con(trazar(&e.w, 100, MAX_VALOR), otras).expect("probar");
        assert_eq!(pi.pending_root, e.pending_root, "el escenario no es el que se cree");
        assert!(verificar(&bytes, &pi).is_err(), "el juez acepto otras opciones");
    }

    /// **INSTRUMENTO, no comprobacion** (RFC-0008 E1). En release, a mano:
    /// `cargo test --release -p stark-experiment instrumento_del_air_de_e1 -- --ignored
    /// --nocapture`. La geometria del AIR y la prueba del positivo con las opciones de la casa:
    /// bytes, probar y verificar, contra la cota de la D-H (65.313 B).
    #[test]
    #[ignore = "instrumento de medida, no comprobacion: correr a mano, en release"]
    fn instrumento_del_air_de_e1() {
        use std::time::Instant;
        let e = escenario();
        println!(
            "E1-AIR| geometria: {ANCHO} columnas x {TRAZA} filas . {NUM_RESTRICCIONES} \
             restricciones . {NUM_ASERCIONES} aserciones . raiz en la fila {ROW_RAIZ}"
        );
        let declarado = pi(&e, e.w.receptor, P_NACIDO, 100, MAX_VALOR);
        for corrida in 0..3 {
            let traza = trazar(&e.w, 100, MAX_VALOR);
            let t = Instant::now();
            let (bytes, _) = probar(traza).expect("probar");
            let probar_s = t.elapsed().as_secs_f64();
            let t = Instant::now();
            let ok = verificar(&bytes, &declarado).is_ok();
            let verificar_ms = t.elapsed().as_secs_f64() * 1000.0;
            println!(
                "E1-AIR| corrida {corrida} . prueba {} B . probar {probar_s:.2} s . verificar \
                 {verificar_ms:.1} ms . verifica {ok} . cota 65313 B",
                bytes.len()
            );
        }
    }
}
