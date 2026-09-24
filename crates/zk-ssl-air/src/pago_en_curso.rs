//! **`PagoEnCursoAir`: el pago en curso, portable y desde el lado del PAGADOR** (RFC-0008, E2).
//!
//! El enunciado, entero: **bajo la raiz de pendientes `pending_root` existe una hoja
//! `C2 = M(C1, X)` con `C1 = M(M(receptor, sal), [importe, 0, 0, 0])` e `importe` EXACTO, con
//! `X = M(refund_id, [delta, 0, 0, 0])`, y en la MISMA posicion del arbol de meta, bajo
//! `pmeta_root`, esta la hoja `commit_operation(PMETA_V1, [emisor, nacido])` con el `nacido`
//! declarado; y `delta >= T - nacido`.** Es el espejo de E1: mismo estado, otra afirmacion.
//!
//! ## Lo que es testigo y lo que no (D-AD, D-AG)
//!
//! Publico: las dos raices, `receptor`, `importe`, `T` y `nacido`. Testigo: la `sal`, el
//! `refund_id`, el `delta`, el `emisor`, los dos caminos y la posicion. El importe pasa a
//! publico y EXACTO -es lo que el pagador afirma haber pagado- y el plazo NO viaja: el circuito
//! prueba una cota inferior de `delta`, nunca `delta` (D-AD).
//!
//! ## La geometria (D-AF): la de E1 con el hueco lleno
//!
//! Dos carriles de doce columnas, un solo bit. El carril A compone `M(receptor, sal)` en el
//! ciclo 0, `C1` en el 1 y `C2` en el 2, y sube del 3 al 34. El carril B, que en E1 estaba
//! OCIOSO en los ciclos 0 y 1, **compone el sobre `X` en el ciclo 1** -su digest aparece en la
//! fila 15, justo cuando el carril A lo lee-, compone la hoja de meta en el 2 y sube del 3 al
//! 34. Los dos leen la direccion de la MISMA columna (`COL_BIT`). Las dos raices salen en la
//! fila 279 y la traza mide 512: ni una fila ni una columna mas que E1.
//!
//! ## La cota temporal, y hasta donde alcanza
//!
//! Un segmento de 64 filas NO prueba <<cabe en 64 bits>>: en este campo eso no diria nada,
//! porque `p < 2^64`. Prueba **`v < 2^63`**, porque la primera fila del segmento exige bit y
//! acumulador a CERO y el acumulador dobla en las 63 siguientes. Luego este AIR sostiene el
//! enunciado mientras **`delta - (T - nacido) < 2^63`**, y [`comprobar_enunciado`] acota
//! `importe`, `T` y `nacido` a [`MAX_VALOR`] y exige `nacido <= T`. El <<nunca>> de D-AG, que el
//! campo reduce a `2^32 - 2`, cae holgadamente dentro.
//!
//! ## Lo que NO prueba, y va escrito
//!
//! - Quien la produjo, ni que el pago vaya a cobrarse, ni que el receptor lo acepte.
//! - Nada sobre otra cabeza que la que firma las dos raices; esa regla es del juez que la enlaza.
//! - Que `delta` sea el plazo EXACTO: prueba `delta >= T - nacido`, que es lo que se afirma.
//!
//! La traza y el `Prover` viven en `stark-experiment` (`circuit_pago_en_curso`), como en el
//! cobro: el AIR tiene UN productor y el probador y el kit lo comparten.

use winter_air::proof::Proof;
use winter_air::{
    Air, AirContext, Assertion, EvaluationFrame, ProofOptions, TraceInfo,
    TransitionConstraintDegree,
};
use winter_crypto::hashers::Rp64_256;
use winter_crypto::DefaultRandomCoin;
use crate::sal::MerkleConSal;
use winter_math::fields::f64::BaseElement;
use winter_math::{FieldElement, ToElements};
use winter_verifier::{verify, AcceptableOptions};
use zk_ssl_hash::DOMINIO_META_PENDIENTE;

use crate::{opciones, ronda_rescue, Blake3, Digest, CICLO, ESTADO, PROFUNDIDAD, RONDAS};

// ------------------------------------------------------------------ la geometria

/// Filas de la traza. La tuberia acaba en [`ROW_RAIZ`] y el resto es holgura.
pub const TRAZA: usize = 512;
/// Bits por segmento de rango.
pub const LARGO_SEGMENTO: usize = 64;
/// Segmentos: uno, `delta - (T - nacido)`. El importe es exacto y publico (D-AF).
pub const SEGMENTOS: usize = 1;
/// Techo de valor representable sin que una resta de la vuelta en el campo.
pub const MAX_VALOR: u64 = 0x3fffffffffffffff;

/// Carril A: el arbol de pendientes (doce columnas del estado).
pub const C_A: usize = 0;
/// La direccion de la subida, COMPARTIDA por los dos carriles.
pub const COL_BIT: usize = 12;
pub const COL_RECEPTOR: usize = 13; // 13..17
/// El importe, PUBLICO y exacto (D-AD).
pub const COL_IMPORTE: usize = 17;
/// El instante que el pago no puede revertir, PUBLICO.
pub const COL_T: usize = 18;
pub const COL_SBIT: usize = 19;
pub const COL_SACC: usize = 20;
pub const COL_SAL: usize = 21; // 21..25
/// El identificador del reembolso, TESTIGO: con `delta` compone el sobre.
pub const COL_REFUND: usize = 25; // 25..29
/// El plazo comprometido, TESTIGO (D-AD).
pub const COL_DELTA: usize = 29;
/// Carril B: el sobre `X` en el ciclo 1, y luego el arbol de meta.
pub const C_B: usize = 30; // 30..42
pub const COL_EMISOR: usize = 42;
pub const COL_NACIDO: usize = 43;
/// Ancho de la traza principal.
pub const ANCHO: usize = 44;

// Ciclos y filas de evento.
/// El ciclo en que el carril B compone el sobre: el 1, que en E1 estaba ocioso.
pub const CYC_X: usize = 1;
pub const CYC_META: usize = 2;
pub const CYC_ACC: usize = 3;
pub const CYC_RAIZ: usize = 35;
pub const ROW_ENLACE_IMPORTE: usize = 7;
pub const ROW_ENLACE_X: usize = 15;
pub const ROW_HOJA_LISTA: usize = 23;
/// Fila en la que las dos subidas entregan sus raices.
pub const ROW_RAIZ: usize = 279;

// El presupuesto, en compilacion.
const _: () = assert!(ROW_RAIZ < TRAZA);
const _: () = assert!(CYC_RAIZ == CYC_ACC + PROFUNDIDAD);
const _: () = assert!(ROW_RAIZ == CYC_RAIZ * CICLO - 1);
const _: () = assert!(ROW_ENLACE_IMPORTE == CICLO - 1);
const _: () = assert!(ROW_ENLACE_X == CYC_META * CICLO - 1);
const _: () = assert!(ROW_HOJA_LISTA == CYC_ACC * CICLO - 1);
const _: () = assert!(CYC_X * CICLO == ROW_ENLACE_IMPORTE + 1);
const _: () = assert!(C_B == COL_DELTA + 1 && ANCHO == COL_NACIDO + 1);
const _: () = assert!(C_A + ESTADO == COL_BIT && C_B + ESTADO == COL_EMISOR);
const _: () = assert!(SEGMENTOS * LARGO_SEGMENTO < ROW_RAIZ);

// Familias de restriccion.
const C_HASH_A: usize = 0;
const C_HASH_B: usize = 12;
const C_CAP_A: usize = 24;
const C_PLACE_A: usize = 28;
const C_CAP_B: usize = 32;
const C_PLACE_B: usize = 36;
const C_BIT_BOOL: usize = 40;
const C_XB_CAP: usize = 41;
const C_XB_IN: usize = 45;
const C_IMP_CAP: usize = 53;
const C_IMP_DIG: usize = 57;
const C_IMP_IN: usize = 61;
const C_IMP_PAD: usize = 62;
const C_X_CAP: usize = 65;
const C_X_DIG: usize = 69;
const C_X_IN: usize = 73;
const C_M_CAP: usize = 77;
const C_M_IN: usize = 81;
const C_INPUT: usize = 89;
const C_TRANSPORT: usize = 97;
const C_SBIT_BOOL: usize = 114;
const C_FIRST_S: usize = 116;
const C_HORNER: usize = 118;
const C_SEG_LINK: usize = 119;
/// Restricciones de transicion declaradas.
pub const NUM_RESTRICCIONES: usize = 120;
const _: () = assert!(C_SEG_LINK + SEGMENTOS == NUM_RESTRICCIONES);
const _: () = assert!(C_XB_IN + 8 == C_IMP_CAP);

/// Las columnas que viajan constantes por toda la traza.
const TRANSPORTE: [usize; 17] = [
    COL_RECEPTOR,
    COL_RECEPTOR + 1,
    COL_RECEPTOR + 2,
    COL_RECEPTOR + 3,
    COL_IMPORTE,
    COL_T,
    COL_SAL,
    COL_SAL + 1,
    COL_SAL + 2,
    COL_SAL + 3,
    COL_REFUND,
    COL_REFUND + 1,
    COL_REFUND + 2,
    COL_REFUND + 3,
    COL_DELTA,
    COL_EMISOR,
    COL_NACIDO,
];
const _: () = assert!(C_TRANSPORT + TRANSPORTE.len() == C_SBIT_BOOL);

// Columnas periodicas.
const P_HASH_A: usize = 0;
const P_HASH_B: usize = 1;
const P_ARK1: usize = 2;
const P_ARK2: usize = 14;
const P_LINK_MERKLE: usize = 26;
const P_LINK_IMPORTE: usize = 27;
const P_LINK_X: usize = 28;
const P_LINK_PLACE: usize = 29;
const P_FIRST_ROW: usize = 30;
const P_FIRST_S: usize = 31;
const P_CONT_S: usize = 32;
const P_SEG_LINK: usize = 33;
/// Columnas periodicas declaradas.
pub const NUM_PERIODICAS: usize = 34;
const _: () = assert!(P_SEG_LINK + SEGMENTOS == NUM_PERIODICAS);

/// Aserciones de frontera: la capacidad de A, las dos raices, el receptor, el importe, la T y el
/// nacido.
pub const NUM_ASERCIONES: usize = 19;

// ------------------------------------------------------------------ las entradas publicas

/// **Lo que el juez declara.** Ni `refund_id` ni `delta` ni el emisor: testigo, no oculto (S521).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PagoEnCursoPublicInputs {
    pub pending_root: Digest,
    pub pmeta_root: Digest,
    pub receptor: Digest,
    pub importe: BaseElement,
    /// El instante, en epocas del registro, que el pago no puede revertir.
    pub t: BaseElement,
    pub nacido: BaseElement,
}

impl ToElements<BaseElement> for PagoEnCursoPublicInputs {
    fn to_elements(&self) -> Vec<BaseElement> {
        let mut v = self.pending_root.to_vec();
        v.extend_from_slice(&self.pmeta_root);
        v.extend_from_slice(&self.receptor);
        v.push(self.importe);
        v.push(self.t);
        v.push(self.nacido);
        v
    }
}

/// Lo que el enunciado tiene que cumplir ANTES de construir el AIR: fuera de estos rangos una
/// resta daria la vuelta en el campo y la cota no diria lo que parece. Nunca entra en panico.
pub fn comprobar_enunciado(pi: &PagoEnCursoPublicInputs) -> Result<(), String> {
    let (i, t, n) = (pi.importe.as_int(), pi.t.as_int(), pi.nacido.as_int());
    if i > MAX_VALOR {
        return Err(format!("el importe {i} pasa del techo {MAX_VALOR}"));
    }
    if t > MAX_VALOR || n > MAX_VALOR {
        return Err(format!("T {t} o nacido {n} pasan del techo {MAX_VALOR}"));
    }
    if n > t {
        return Err(format!("nacido {n} posterior a T {t}: el plazo iria hacia atras"));
    }
    Ok(())
}

// ------------------------------------------------------------------ el AIR

pub struct PagoEnCursoAir {
    context: AirContext<BaseElement>,
    pi: PagoEnCursoPublicInputs,
}

impl Air for PagoEnCursoAir {
    type BaseField = BaseElement;
    type PublicInputs = PagoEnCursoPublicInputs;

    fn new(trace_info: TraceInfo, pub_inputs: Self::PublicInputs, options: ProofOptions) -> Self {
        assert_eq!(ANCHO, trace_info.width());
        let entera = vec![TRAZA];
        let ciclo = |d: usize| TransitionConstraintDegree::with_cycles(d, entera.clone());

        let mut grados = Vec::with_capacity(NUM_RESTRICCIONES);
        // las rondas de los dos carriles
        for _ in 0..2 * ESTADO {
            grados.push(ciclo(7));
        }
        // la subida de los dos carriles: capacidad y colocacion por el bit compartido
        for _ in 0..2 {
            for _ in 0..4 {
                grados.push(ciclo(1));
            }
            for _ in 0..4 {
                grados.push(ciclo(2));
            }
        }
        grados.push(TransitionConstraintDegree::new(2));
        // el arranque del sobre, los enlaces del importe, de X y de la meta, y la fila 0
        for _ in C_XB_CAP..C_TRANSPORT {
            grados.push(ciclo(1));
        }
        for _ in C_TRANSPORT..C_SBIT_BOOL {
            grados.push(TransitionConstraintDegree::new(1));
        }
        for _ in 0..2 {
            grados.push(TransitionConstraintDegree::new(2));
        }
        for _ in C_FIRST_S..NUM_RESTRICCIONES {
            grados.push(ciclo(1));
        }
        assert_eq!(grados.len(), NUM_RESTRICCIONES, "cuenta de grados");

        PagoEnCursoAir {
            context: AirContext::new(trace_info, grados, NUM_ASERCIONES, options),
            pi: pub_inputs,
        }
    }

    fn context(&self) -> &AirContext<Self::BaseField> {
        &self.context
    }

    fn get_periodic_column_values(&self) -> Vec<Vec<Self::BaseField>> {
        let cero = BaseElement::ZERO;
        let uno = BaseElement::ONE;
        let mut columnas = Vec::with_capacity(NUM_PERIODICAS);

        // el carril A hashea desde la fila 0; el B, desde su ciclo del sobre (la 8)
        for desde in [0, CYC_X * CICLO] {
            let mut bandera = vec![cero; TRAZA];
            for r in desde..=ROW_RAIZ {
                if r % CICLO < RONDAS {
                    bandera[r] = uno;
                }
            }
            columnas.push(bandera);
        }

        for ark1 in [true, false] {
            for i in 0..ESTADO {
                let mut col = vec![cero; TRAZA];
                for r in 0..=ROW_RAIZ {
                    let pos = r % CICLO;
                    if pos < RONDAS {
                        col[r] = if ark1 {
                            Rp64_256::ARK1[pos][i]
                        } else {
                            Rp64_256::ARK2[pos][i]
                        };
                    }
                }
                columnas.push(col);
            }
        }

        let mut enlace_arbol = vec![cero; TRAZA];
        for nivel in 0..PROFUNDIDAD - 1 {
            enlace_arbol[(CYC_ACC + nivel) * CICLO + CICLO - 1] = uno;
        }
        columnas.push(enlace_arbol);

        for fila in [ROW_ENLACE_IMPORTE, ROW_ENLACE_X, ROW_HOJA_LISTA, 0] {
            let mut sel = vec![cero; TRAZA];
            sel[fila] = uno;
            columnas.push(sel);
        }

        let mut primera_s = vec![cero; TRAZA];
        let mut sigue_s = vec![cero; TRAZA];
        for seg in 0..SEGMENTOS {
            primera_s[seg * LARGO_SEGMENTO] = uno;
            for p in 0..LARGO_SEGMENTO - 1 {
                sigue_s[seg * LARGO_SEGMENTO + p] = uno;
            }
        }
        columnas.push(primera_s);
        columnas.push(sigue_s);

        for seg in 0..SEGMENTOS {
            let mut enlace = vec![cero; TRAZA];
            enlace[(seg + 1) * LARGO_SEGMENTO - 2] = uno;
            columnas.push(enlace);
        }

        debug_assert_eq!(columnas.len(), NUM_PERIODICAS);
        columnas
    }

    fn evaluate_transition<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        frame: &EvaluationFrame<E>,
        periodic: &[E],
        result: &mut [E],
    ) {
        let actual = frame.current();
        let siguiente = frame.next();

        let ark1 = &periodic[P_ARK1..P_ARK1 + ESTADO];
        let ark2 = &periodic[P_ARK2..P_ARK2 + ESTADO];
        let enlace_arbol = periodic[P_LINK_MERKLE];
        let enlace_importe = periodic[P_LINK_IMPORTE];
        let enlace_x = periodic[P_LINK_X];
        let enlace_sitio = periodic[P_LINK_PLACE];
        let primera_fila = periodic[P_FIRST_ROW];
        let primera_s = periodic[P_FIRST_S];
        let sigue_s = periodic[P_CONT_S];

        // Las rondas de Rescue de los dos carriles, por el UNICO productor del crate.
        ronda_rescue(
            &actual[C_A..C_A + ESTADO],
            &siguiente[C_A..C_A + ESTADO],
            ark1,
            ark2,
            periodic[P_HASH_A],
            &mut result[C_HASH_A..C_HASH_A + ESTADO],
        );
        ronda_rescue(
            &actual[C_B..C_B + ESTADO],
            &siguiente[C_B..C_B + ESTADO],
            ark1,
            ark2,
            periodic[P_HASH_B],
            &mut result[C_HASH_B..C_HASH_B + ESTADO],
        );

        // La subida: los DOS carriles se colocan con el MISMO bit (D-H).
        let bit = siguiente[COL_BIT];
        let enlace = enlace_arbol + enlace_sitio;
        for (c_cap, c_place, base) in [(C_CAP_A, C_PLACE_A, C_A), (C_CAP_B, C_PLACE_B, C_B)] {
            for i in 0..4 {
                result[c_cap + i] = enlace * siguiente[base + i];
                let d = actual[base + 4 + i];
                result[c_place + i] = enlace
                    * ((E::ONE - bit) * (siguiente[base + 4 + i] - d)
                        + bit * (siguiente[base + 8 + i] - d));
            }
        }
        result[C_BIT_BOOL] = actual[COL_BIT] * (actual[COL_BIT] - E::ONE);

        // Fila 7, carril B: arranca el sobre X = M(refund_id, [delta, 0, 0, 0]). Es el hueco que
        // E1 dejaba ocioso, y el rate entero queda atado: cuatro limbos de refund_id, el delta y
        // tres ceros.
        for i in 0..4 {
            result[C_XB_CAP + i] = enlace_importe * siguiente[C_B + i];
            result[C_XB_IN + i] =
                enlace_importe * (siguiente[C_B + 4 + i] - actual[COL_REFUND + i]);
        }
        result[C_XB_IN + 4] = enlace_importe * (siguiente[C_B + 8] - actual[COL_DELTA]);
        for i in 0..3 {
            result[C_XB_IN + 5 + i] = enlace_importe * siguiente[C_B + 9 + i];
        }

        // Fila 7, carril A: C1 = M(M(receptor, sal), [importe, 0, 0, 0]). El rate ENTERO.
        for i in 0..4 {
            result[C_IMP_CAP + i] = enlace_importe * siguiente[C_A + i];
            result[C_IMP_DIG + i] =
                enlace_importe * (siguiente[C_A + 4 + i] - actual[C_A + 4 + i]);
        }
        result[C_IMP_IN] = enlace_importe * (siguiente[C_A + 8] - actual[COL_IMPORTE]);
        for i in 0..3 {
            result[C_IMP_PAD + i] = enlace_importe * siguiente[C_A + 9 + i];
        }

        // Fila 15, carril A: C2 = M(C1, X), con X LEIDO DEL CARRIL B: el sobre lo ata la
        // ESTRUCTURA, no cuatro columnas de testigo (D-AF).
        for i in 0..4 {
            result[C_X_CAP + i] = enlace_x * siguiente[C_A + i];
            result[C_X_DIG + i] = enlace_x * (siguiente[C_A + 4 + i] - actual[C_A + 4 + i]);
            result[C_X_IN + i] = enlace_x * (siguiente[C_A + 8 + i] - actual[C_B + 4 + i]);
        }

        // Fila 15, carril B: la hoja de meta, commit_operation(PMETA_V1, [emisor, nacido]).
        let dominio = E::from(BaseElement::new(DOMINIO_META_PENDIENTE));
        result[C_M_CAP] = enlace_x * (siguiente[C_B] - dominio);
        for i in 1..4 {
            result[C_M_CAP + i] = enlace_x * siguiente[C_B + i];
        }
        result[C_M_IN] = enlace_x * (siguiente[C_B + 4] - actual[COL_EMISOR]);
        result[C_M_IN + 1] = enlace_x * (siguiente[C_B + 5] - actual[COL_NACIDO]);
        for i in 2..8 {
            result[C_M_IN + i] = enlace_x * siguiente[C_B + 4 + i];
        }

        // Fila 0 del carril A: el receptor y la sal. El carril B no se toca: en el ciclo 0 esta
        // ocioso, y lo que entra en el 1 lo ata el enlace de la fila 7.
        for i in 0..4 {
            result[C_INPUT + i] =
                primera_fila * (actual[C_A + 4 + i] - actual[COL_RECEPTOR + i]);
            result[C_INPUT + 4 + i] = primera_fila * (actual[C_A + 8 + i] - actual[COL_SAL + i]);
        }

        for (k, col) in TRANSPORTE.iter().enumerate() {
            result[C_TRANSPORT + k] = siguiente[*col] - actual[*col];
        }

        // ===== LA COTA TEMPORAL: delta - (T - nacido) en [0, 2^63) =====
        let sbit_act = actual[COL_SBIT];
        let sbit_sig = siguiente[COL_SBIT];
        let sacc_act = actual[COL_SACC];
        let sacc_sig = siguiente[COL_SACC];

        result[C_SBIT_BOOL] = sbit_act * (sbit_act - E::ONE);
        result[C_SBIT_BOOL + 1] = sbit_sig * (sbit_sig - E::ONE);
        result[C_FIRST_S] = primera_s * sbit_act;
        result[C_FIRST_S + 1] = primera_s * sacc_act;
        result[C_HORNER] = sigue_s * (sacc_sig - (sacc_act + sacc_act + sbit_sig));

        let esperado = actual[COL_DELTA] - (actual[COL_T] - actual[COL_NACIDO]);
        result[C_SEG_LINK] = periodic[P_SEG_LINK] * (sacc_sig - esperado);
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        let cero = BaseElement::ZERO;
        let mut a = Vec::with_capacity(NUM_ASERCIONES);

        // La capacidad del carril A arranca a cero; su rate entero es receptor y sal.
        for i in 0..4 {
            a.push(Assertion::single(C_A + i, 0, cero));
        }
        // Las dos subidas entregan las raices DECLARADAS.
        for i in 0..4 {
            a.push(Assertion::single(C_A + 4 + i, ROW_RAIZ, self.pi.pending_root[i]));
        }
        for i in 0..4 {
            a.push(Assertion::single(C_B + 4 + i, ROW_RAIZ, self.pi.pmeta_root[i]));
        }
        // El receptor, el importe, la T y el nacido son los que declara QUIEN VERIFICA.
        for i in 0..4 {
            a.push(Assertion::single(COL_RECEPTOR + i, 0, self.pi.receptor[i]));
        }
        a.push(Assertion::single(COL_IMPORTE, 0, self.pi.importe));
        a.push(Assertion::single(COL_T, 0, self.pi.t));
        a.push(Assertion::single(COL_NACIDO, 0, self.pi.nacido));

        debug_assert_eq!(a.len(), NUM_ASERCIONES);
        a
    }
}

// ------------------------------------------------------------------ el juez

/// **Verifica una prueba de pago en curso contra su enunciado.** Comprueba ANTES la forma de la
/// traza que la prueba declara -ancho, sin segmento auxiliar y longitud [`TRAZA`]- y acepta SOLO
/// [`opciones`]. Nunca entra en panico por lo que traiga la prueba.
pub fn verificar(prueba: &[u8], pi: &PagoEnCursoPublicInputs) -> Result<(), String> {
    comprobar_enunciado(pi)?;
    let proof =
        Proof::from_bytes(prueba).map_err(|e| format!("la prueba no se deserializa: {e:?}"))?;
    let info = proof.trace_info();
    let forma = (
        info.main_trace_width(),
        info.aux_segment_width(),
        info.get_num_aux_segment_rand_elements(),
        info.length(),
    );
    if forma != (ANCHO + 1, 0, 0, 2 * TRAZA) {
        return Err(format!(
            "forma de traza {forma:?}; el enunciado pide {:?}",
            (ANCHO + 1, 0, 0, 2 * TRAZA)
        ));
    }
    let aceptadas = AcceptableOptions::OptionSet(vec![opciones()]);
    verify::<PagoEnCursoAir, Blake3, DefaultRandomCoin<Blake3>, MerkleConSal<Blake3>>(
        proof,
        pi.clone(),
        &aceptadas,
    )
    .map_err(|e| format!("{e:?}"))
}

// ------------------------------------------------------------------ el enlace (D-AD, D-AH)

/// **Lo que una cabeza v5 firma y el pago en curso necesita.** Se lee de una cabeza ya
/// verificada: este crate no verifica firmas, las verifica el kit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CabezaPago {
    pub seq: u64,
    pub pending_root: Digest,
    pub pmeta_root: Digest,
}

/// **Lo que el sobre afirma**: pague `importe` a `receptor`, nacido en `nacido`, y no es
/// reversible antes de `t`. El plazo no viaja.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AfirmacionPago {
    pub receptor: Digest,
    pub importe: u64,
    pub t: u64,
    pub nacido: u64,
}

/// **La prueba de pago en curso contra una cabeza v5.** Exige, en nativo y ANTES de tocar la
/// prueba, que el pendiente haya nacido antes que la cabeza que lo firma (`nacido < seq`: una
/// meta nacida despues es una cabeza que miente); compone el enunciado con las dos raices que la
/// cabeza firma y lo que el sobre afirma; y solo entonces llama al juez. Devuelve el enunciado
/// verificado. Una prueba de otro arbol, de otra meta o con otro importe no se enlaza: la prueba
/// es la de SU enunciado.
pub fn verificar_contra_cabeza(
    prueba: &[u8],
    af: &AfirmacionPago,
    cabeza: &CabezaPago,
) -> Result<PagoEnCursoPublicInputs, String> {
    if af.nacido >= cabeza.seq {
        return Err(format!(
            "nacido {} no es anterior a la cabeza de seq {}: una meta nacida despues de la cabeza \
             que la firma",
            af.nacido, cabeza.seq
        ));
    }
    let pi = PagoEnCursoPublicInputs {
        pending_root: cabeza.pending_root,
        pmeta_root: cabeza.pmeta_root,
        receptor: af.receptor,
        importe: BaseElement::new(af.importe),
        t: BaseElement::new(af.t),
        nacido: BaseElement::new(af.nacido),
    };
    verificar(prueba, &pi)?;
    Ok(pi)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(k: u64) -> Digest {
        [
            BaseElement::new(k),
            BaseElement::new(3 * k + 1),
            BaseElement::new(7 * k + 2),
            BaseElement::new(11 * k + 3),
        ]
    }

    fn enunciado() -> PagoEnCursoPublicInputs {
        PagoEnCursoPublicInputs {
            pending_root: d(1),
            pmeta_root: d(2),
            receptor: d(3),
            importe: BaseElement::new(250_000),
            t: BaseElement::new(1100),
            nacido: BaseElement::new(1040),
        }
    }

    fn air() -> PagoEnCursoAir {
        PagoEnCursoAir::new(TraceInfo::new(ANCHO, TRAZA), enunciado(), opciones())
    }

    /// Cada ranura declarada se escribe, y el numero de ranuras es el que el contexto cuenta.
    #[test]
    fn las_restricciones_declaradas_son_las_escritas() {
        let air = air();
        assert_eq!(air.context().num_main_transition_constraints(), NUM_RESTRICCIONES);
        let centinela = BaseElement::new(0xDEAD_BEEF);
        let fila = |k: u64| -> Vec<BaseElement> {
            (0..ANCHO as u64).map(|i| BaseElement::new(k * 1000 + i + 5)).collect()
        };
        let marco = EvaluationFrame::from_rows(fila(1), fila(2));
        let periodicas: Vec<BaseElement> =
            (0..NUM_PERIODICAS as u64).map(|i| BaseElement::new(i + 2)).collect();
        let mut r = vec![centinela; NUM_RESTRICCIONES];
        air.evaluate_transition(&marco, &periodicas, &mut r);
        assert!(r.iter().all(|x| *x != centinela), "una ranura sin escribir");
        assert_eq!(air.get_periodic_column_values().len(), NUM_PERIODICAS);
    }

    /// Las aserciones atan el receptor, el importe, la T, el nacido y las DOS raices que declara
    /// quien verifica; y ninguna nombra el refund_id, el delta, la sal ni el emisor.
    #[test]
    fn las_aserciones_atan_lo_declarado_y_nada_del_testigo() {
        let a = air().get_assertions();
        assert_eq!(a.len(), NUM_ASERCIONES);
        let pi = enunciado();
        let mut esperadas = vec![
            Assertion::single(COL_IMPORTE, 0, pi.importe),
            Assertion::single(COL_T, 0, pi.t),
            Assertion::single(COL_NACIDO, 0, pi.nacido),
        ];
        for i in 0..4 {
            esperadas.push(Assertion::single(COL_RECEPTOR + i, 0, pi.receptor[i]));
            esperadas.push(Assertion::single(C_A + 4 + i, ROW_RAIZ, pi.pending_root[i]));
            esperadas.push(Assertion::single(C_B + 4 + i, ROW_RAIZ, pi.pmeta_root[i]));
        }
        for e in &esperadas {
            let e = format!("{e:?}");
            assert!(a.iter().any(|x| format!("{x:?}") == e), "sin la asercion {e}");
        }
        for x in &a {
            let col = x.column();
            assert!(
                !(COL_REFUND..COL_REFUND + 4).contains(&col)
                    && !(COL_SAL..COL_SAL + 4).contains(&col)
                    && col != COL_DELTA
                    && col != COL_EMISOR,
                "una asercion sobre el testigo: columna {col}"
            );
        }
    }

    /// Un enunciado fuera de rango y una prueba que no es prueba se rechazan con `Err`, sin
    /// construir el AIR ni entrar en panico.
    #[test]
    fn lo_que_no_tiene_forma_se_rechaza_sin_panico() {
        let bien = enunciado();
        let malos = [
            PagoEnCursoPublicInputs { t: BaseElement::new(MAX_VALOR + 1), ..bien.clone() },
            PagoEnCursoPublicInputs {
                nacido: BaseElement::new(1101),
                ..bien.clone()
            },
            PagoEnCursoPublicInputs {
                importe: BaseElement::new(MAX_VALOR + 1),
                ..bien.clone()
            },
        ];
        for malo in malos {
            assert!(comprobar_enunciado(&malo).is_err(), "aceptado: {malo:?}");
        }
        assert!(comprobar_enunciado(&bien).is_ok());
        assert!(verificar(&[1, 2, 3], &bien).is_err(), "tres bytes verificaron");
    }

    /// Las quince entradas publicas van al transcripto, cada una en su sitio; el plazo no esta.
    #[test]
    fn las_entradas_publicas_son_quince_y_sin_el_plazo() {
        let pi = enunciado();
        let v = pi.to_elements();
        assert_eq!(v.len(), 15);
        assert_eq!(&v[0..4], &pi.pending_root);
        assert_eq!(&v[4..8], &pi.pmeta_root);
        assert_eq!(&v[8..12], &pi.receptor);
        assert_eq!((v[12], v[13], v[14]), (pi.importe, pi.t, pi.nacido));
    }

    /// **D-K, el molde de E1: un nacido que no es anterior a la cabeza se rechaza ANTES de la
    /// prueba**: tres bytes que no son prueba no llegan al juez; con un `seq` posterior, si.
    #[test]
    fn un_nacido_que_no_es_anterior_a_la_cabeza_se_rechaza_antes_de_la_prueba() {
        let af = AfirmacionPago { receptor: d(3), importe: 250_000, t: 1100, nacido: 1040 };
        for seq in [1040, 1039, 0] {
            let cab = CabezaPago { seq, pending_root: d(1), pmeta_root: d(2) };
            let e = verificar_contra_cabeza(&[1, 2, 3], &af, &cab).unwrap_err();
            assert!(e.contains("no es anterior a la cabeza"), "seq {seq}: {e}");
        }
        let cab = CabezaPago { seq: 1041, pending_root: d(1), pmeta_root: d(2) };
        let e = verificar_contra_cabeza(&[1, 2, 3], &af, &cab).unwrap_err();
        assert!(!e.contains("no es anterior a la cabeza"), "con seq 1041 tenia que llegar: {e}");
    }

    /// La geometria de E1 con el hueco lleno, atada a su fuente: el ancho no se mueve, el sobre
    /// ocupa el ciclo 1 y las familias encajan una detras de otra.
    #[test]
    fn la_geometria_es_la_de_e1_con_el_hueco_lleno() {
        assert_eq!((ROW_RAIZ + 1).next_power_of_two(), TRAZA);
        assert_eq!(ANCHO, 2 * ESTADO + 20);
        assert_eq!(CYC_X, 1);
        assert_eq!(CYC_X * CICLO, ROW_ENLACE_IMPORTE + 1);
        assert_eq!(C_XB_IN + 8, C_IMP_CAP);
        assert_eq!(C_M_IN + 8, C_INPUT);
        assert_eq!(C_INPUT + 8, C_TRANSPORT);
        assert_eq!((C_HASH_B, C_CAP_A), (C_HASH_A + ESTADO, C_HASH_B + ESTADO));
        assert_eq!(SEGMENTOS, 1);
    }
}
