//! **`PrendaAir`: la prenda sobre un pendiente, CON titularidad** (RFC-0008, E3).
//!
//! El enunciado, entero: **bajo la raiz de pendientes `pending_root` existe una hoja
//! `C2 = M(C1, X)` con `C1 = M(M(receptor, sal), [importe, 0, 0, 0])`; la `marca` declarada es
//! `commit_operation(DOMINIO_PRENDA, C2)`; y quien prueba conoce la clave de gasto cuya
//! identidad derivada es ese mismo `receptor`.** Es un enunciado de AUTORIZACION, y ahi se
//! separa del hermano de E1: el cobro pendiente dice lo que el ESTADO contiene y el pagador lo
//! produce igual que el receptor (D-G); esto dice ademas que **nadie mas HA PODIDO producirlo**
//! (D-AV).
//!
//! ## Lo que es testigo y lo que no (D-AW, D-I)
//!
//! Publico: `pending_root`, `receptor` y la `marca`. Testigo: la `sal`, el `importe`, el sobre
//! `X`, **la clave**, el camino y la posicion. `C2` NO se publica: publicarlo haria enlazables
//! todos los sobres de una misma hoja y rompe lo que las posiciones saladas prometen, asi que
//! el circuito prueba DENTRO que la marca es su hash y quien juzga compara la marca con lo que
//! el arbol de consumos lleva, sin calcular nada (D-AW).
//!
//! ## La geometria (D-AX, D-AY)
//!
//! Es el molde de [`crate::cobro_pendiente`] con dos piezas fuera y dos dentro, y **ni una fila
//! nueva**: la traza sigue midiendo [`TRAZA`] y la raiz sigue saliendo en [`ROW_RAIZ`].
//!
//! - fuera la BANDA (cuatro columnas, ocho restricciones y cinco periodicas): la prenda no dice
//!   el importe;
//! - fuera el carril de la META (su subida y sus dos columnas): el enunciado no necesita
//!   `nacido`, y con el se van `COL_EMISOR` y `COL_NACIDO` (D-AY);
//! - dentro el CICLO DE LA CLAVE, en las cuatro columnas que la banda deja ([`COL_KEY`]) y en el
//!   ciclo 0 del carril B, que en el molde no hashea;
//! - dentro el CICLO DE LA MARCA, en el ciclo 3 del carril B, sembrado en la fila
//!   [`ROW_HOJA_LISTA`], que es justo donde `C2` esta listo.
//!
//! Ancho 44 -> **42**; restricciones de transicion 110 -> **106**; periodicas 36 -> **31**;
//! aserciones 19 -> **20**. Los tres selectores que los dos ciclos nuevos necesitan
//! -[`P_FIRST_ROW`](self), la fila 7 y la fila 23- **ya estaban en el molde**: no hace falta
//! ninguna columna periodica nueva.
//!
//! ## La marca se ata al `C2` que ENTRA en el arbol
//!
//! El rate del ciclo de la marca se lee del digest del carril A **en la misma fila** en la que
//! ese digest entra a la subida ([`ROW_HOJA_LISTA`]). No hay dos `C2`: es el mismo, y no hace
//! falta una igualdad que haya que acordarse de escribir.
//!
//! ## La titularidad, derivada y no calcada
//!
//! El ciclo de la clave compone `M(as_digest(SPEND_KEY_DOMAIN), clave)`, que es lo que
//! `derive_public_id_wide` compone en nativo: capacidad a CERO, rate
//! `[SPEND_KEY_DOMAIN, 0, 0, 0 | clave]`, **atado entero**. No es el calco de
//! `circuit_claim_v2`, cuya siembra escribe solo dos tramos de su estado y no reinicia la
//! capacidad: aquella es otra maquina, y un operador copiado sin su invariante es un rojo
//! esperando. Lo que ata las dos formas es el probador, que produce la identidad en NATIVO y
//! cuyo primer testigo la falsa.
//!
//! ## Lo que NO prueba, y va escrito
//!
//! - Que la marca este en el arbol de consumos: eso lo cruza quien juzga, contra `consRoot`.
//! - Nada del importe, de la caducidad ni del `nacido`: la prenda no lleva la meta (D-AY).
//! - Nada sobre otra cabeza que la que firma `pending_root`.
//! - Que no haya otra prenda sobre la misma hoja: lo que D-AU sostiene es que no hay una segunda
//!   PROBADA.
//! - El enlace fuerte con el cobro. Lo tiene quien tiene el AVISO: con el recompone `C2`,
//!   calcula la marca y ve que las dos mitades hablan de la misma hoja (D-AY).
//!
//! La traza y el `Prover` viven en `stark-experiment` (`circuit_prenda`), como en el molde: el
//! AIR tiene UN productor y el probador y el kit lo comparten.

use winter_air::proof::Proof;
use winter_air::{
    Air, AirContext, Assertion, EvaluationFrame, ProofOptions, TraceInfo,
    TransitionConstraintDegree,
};
use winter_crypto::hashers::Rp64_256;
use winter_crypto::{DefaultRandomCoin, MerkleTree};
use winter_math::fields::f64::BaseElement;
use winter_math::{FieldElement, ToElements};
use winter_verifier::{verify, AcceptableOptions};
use zk_ssl_hash::{DOMINIO_PRENDA, SPEND_KEY_DOMAIN};

use crate::{opciones, ronda_rescue, Blake3, Digest, CICLO, ESTADO, PROFUNDIDAD, RONDAS};

// ------------------------------------------------------------------ la geometria

/// Filas de la traza. La tuberia acaba en [`ROW_RAIZ`] y el resto es holgura.
pub const TRAZA: usize = 512;

/// Carril A: el arbol de pendientes (doce columnas del estado).
pub const C_A: usize = 0;
/// La direccion de la subida del carril A.
pub const COL_BIT: usize = 12;
pub const COL_RECEPTOR: usize = 13; // 13..17
pub const COL_IMPORTE: usize = 17;
/// **La clave de gasto, TESTIGO** (D-AV): las cuatro columnas que la banda deja (D-AX).
pub const COL_KEY: usize = 18; // 18..22
pub const COL_SAL: usize = 22; // 22..26
/// El sobre de reversion, TESTIGO (D-I).
pub const COL_X: usize = 26; // 26..30
/// Carril B: la clave en su ciclo 0 y la marca en su ciclo 3.
pub const C_B: usize = 30; // 30..42
/// Ancho de la traza principal.
pub const ANCHO: usize = 42;

// Ciclos y filas de evento.
pub const CYC_ACC: usize = 3;
/// El ciclo de la marca ES el primer ciclo de la subida: el digest que entra al arbol y el que
/// se hashea son el MISMO, en la misma fila de enlace.
pub const CYC_MARCA: usize = CYC_ACC;
pub const CYC_RAIZ: usize = 35;
pub const ROW_ENLACE_IMPORTE: usize = 7;
pub const ROW_ENLACE_X: usize = 15;
pub const ROW_HOJA_LISTA: usize = 23;
/// Fila en la que el carril B entrega la marca.
pub const ROW_MARCA: usize = 31;
/// Fila en la que la subida entrega la raiz de pendientes.
pub const ROW_RAIZ: usize = 279;

// El presupuesto, en compilacion.
const _: () = assert!(ROW_RAIZ < TRAZA);
const _: () = assert!(CYC_RAIZ == CYC_ACC + PROFUNDIDAD);
const _: () = assert!(ROW_RAIZ == CYC_RAIZ * CICLO - 1);
const _: () = assert!(ROW_ENLACE_IMPORTE == CICLO - 1);
const _: () = assert!(ROW_ENLACE_X == 2 * CICLO - 1);
const _: () = assert!(ROW_HOJA_LISTA == CYC_MARCA * CICLO - 1);
const _: () = assert!(ROW_MARCA == (CYC_MARCA + 1) * CICLO - 1);
const _: () = assert!(ROW_MARCA < ROW_RAIZ);
const _: () = assert!(C_B == COL_X + 4 && ANCHO == C_B + ESTADO);
const _: () = assert!(C_A + ESTADO == COL_BIT && COL_KEY + 4 == COL_SAL);

// Familias de restriccion.
const C_HASH_A: usize = 0;
const C_HASH_B: usize = 12;
const C_CAP_A: usize = 24;
const C_PLACE_A: usize = 28;
const C_BIT_BOOL: usize = 32;
const C_IMP_CAP: usize = 33;
const C_IMP_DIG: usize = 37;
const C_IMP_IN: usize = 41;
const C_IMP_PAD: usize = 42;
const C_X_CAP: usize = 45;
const C_X_DIG: usize = 49;
const C_X_IN: usize = 53;
const C_MARCA_CAP: usize = 57;
const C_MARCA_IN: usize = 61;
const C_INPUT: usize = 69;
const C_KEY_IN: usize = 77;
/// TITULARIDAD: la identidad derivada de la clave es el receptor declarado.
const C_PK_CHECK: usize = 85;
const C_TRANSPORT: usize = 89;
/// Restricciones de transicion declaradas.
pub const NUM_RESTRICCIONES: usize = 106;

/// Las columnas que viajan constantes por toda la traza.
const TRANSPORTE: [usize; 17] = [
    COL_RECEPTOR,
    COL_RECEPTOR + 1,
    COL_RECEPTOR + 2,
    COL_RECEPTOR + 3,
    COL_IMPORTE,
    COL_KEY,
    COL_KEY + 1,
    COL_KEY + 2,
    COL_KEY + 3,
    COL_SAL,
    COL_SAL + 1,
    COL_SAL + 2,
    COL_SAL + 3,
    COL_X,
    COL_X + 1,
    COL_X + 2,
    COL_X + 3,
];
const _: () = assert!(C_TRANSPORT + TRANSPORTE.len() == NUM_RESTRICCIONES);

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
/// Columnas periodicas declaradas.
pub const NUM_PERIODICAS: usize = 31;
const _: () = assert!(P_FIRST_ROW + 1 == NUM_PERIODICAS);

/// Aserciones de frontera: las dos capacidades, la raiz, el receptor y la marca.
pub const NUM_ASERCIONES: usize = 20;

// ------------------------------------------------------------------ las entradas publicas

/// **Lo que el juez declara.** Ni `C2`, ni la clave, ni el importe: son testigo (D-AW).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrendaPublicInputs {
    pub pending_root: Digest,
    pub receptor: Digest,
    pub marca: Digest,
}

impl ToElements<BaseElement> for PrendaPublicInputs {
    fn to_elements(&self) -> Vec<BaseElement> {
        let mut v = self.pending_root.to_vec();
        v.extend_from_slice(&self.receptor);
        v.extend_from_slice(&self.marca);
        v
    }
}

// ------------------------------------------------------------------ el AIR

pub struct PrendaAir {
    context: AirContext<BaseElement>,
    pi: PrendaPublicInputs,
}

impl Air for PrendaAir {
    type BaseField = BaseElement;
    type PublicInputs = PrendaPublicInputs;

    fn new(trace_info: TraceInfo, pub_inputs: Self::PublicInputs, options: ProofOptions) -> Self {
        assert_eq!(ANCHO, trace_info.width());
        let entera = vec![TRAZA];
        let ciclo = |d: usize| TransitionConstraintDegree::with_cycles(d, entera.clone());

        let mut grados = Vec::with_capacity(NUM_RESTRICCIONES);
        // las rondas de los dos carriles
        for _ in 0..2 * ESTADO {
            grados.push(ciclo(7));
        }
        // la subida del carril A: capacidad y colocacion por el bit
        for _ in 0..4 {
            grados.push(ciclo(1));
        }
        for _ in 0..4 {
            grados.push(ciclo(2));
        }
        grados.push(TransitionConstraintDegree::new(2));
        // los enlaces del importe, de X, de la marca, y las entradas de la fila 0
        for _ in C_IMP_CAP..C_TRANSPORT {
            grados.push(ciclo(1));
        }
        for _ in C_TRANSPORT..NUM_RESTRICCIONES {
            grados.push(TransitionConstraintDegree::new(1));
        }
        assert_eq!(grados.len(), NUM_RESTRICCIONES, "cuenta de grados");

        PrendaAir {
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

        // el carril A hashea desde la fila 0 hasta la raiz
        let mut bandera_a = vec![cero; TRAZA];
        for r in 0..=ROW_RAIZ {
            if r % CICLO < RONDAS {
                bandera_a[r] = uno;
            }
        }
        columnas.push(bandera_a);

        // el carril B hashea SOLO en su ciclo de la clave y en el de la marca
        let mut bandera_b = vec![cero; TRAZA];
        for ciclo in [0, CYC_MARCA] {
            for pos in 0..RONDAS {
                bandera_b[ciclo * CICLO + pos] = uno;
            }
        }
        columnas.push(bandera_b);

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

        // La subida: solo la del carril A, porque la prenda no lleva la meta (D-AY).
        let bit = siguiente[COL_BIT];
        let enlace = enlace_arbol + enlace_sitio;
        for i in 0..4 {
            result[C_CAP_A + i] = enlace * siguiente[C_A + i];
            let d = actual[C_A + 4 + i];
            result[C_PLACE_A + i] = enlace
                * ((E::ONE - bit) * (siguiente[C_A + 4 + i] - d)
                    + bit * (siguiente[C_A + 8 + i] - d));
        }
        result[C_BIT_BOOL] = actual[COL_BIT] * (actual[COL_BIT] - E::ONE);

        // Ciclo 1 del carril A: C1 = M(M(receptor, sal), [importe, 0, 0, 0]). El rate ENTERO.
        for i in 0..4 {
            result[C_IMP_CAP + i] = enlace_importe * siguiente[C_A + i];
            result[C_IMP_DIG + i] =
                enlace_importe * (siguiente[C_A + 4 + i] - actual[C_A + 4 + i]);
        }
        result[C_IMP_IN] = enlace_importe * (siguiente[C_A + 8] - actual[COL_IMPORTE]);
        for i in 0..3 {
            result[C_IMP_PAD + i] = enlace_importe * siguiente[C_A + 9 + i];
        }

        // Ciclo 2 del carril A: C2 = M(C1, X), con X testigo (D-I).
        for i in 0..4 {
            result[C_X_CAP + i] = enlace_x * siguiente[C_A + i];
            result[C_X_DIG + i] = enlace_x * (siguiente[C_A + 4 + i] - actual[C_A + 4 + i]);
            result[C_X_IN + i] = enlace_x * (siguiente[C_A + 8 + i] - actual[COL_X + i]);
        }

        // Ciclo 3 del carril B: la MARCA, commit_operation(PREND_V1, C2). El dominio entra como
        // CAPACIDAD (D-AW) y el rate se lee del digest del carril A en la MISMA fila en la que
        // ese digest entra a la subida: no hay dos C2.
        let dominio = E::from(BaseElement::new(DOMINIO_PRENDA));
        result[C_MARCA_CAP] = enlace_sitio * (siguiente[C_B] - dominio);
        for i in 1..4 {
            result[C_MARCA_CAP + i] = enlace_sitio * siguiente[C_B + i];
        }
        for i in 0..4 {
            result[C_MARCA_IN + i] =
                enlace_sitio * (siguiente[C_B + 4 + i] - actual[C_A + 4 + i]);
        }
        for i in 4..8 {
            result[C_MARCA_IN + i] = enlace_sitio * siguiente[C_B + 4 + i];
        }

        // Fila 0 del carril A: el receptor y la sal.
        for i in 0..4 {
            result[C_INPUT + i] =
                primera_fila * (actual[C_A + 4 + i] - actual[COL_RECEPTOR + i]);
            result[C_INPUT + 4 + i] = primera_fila * (actual[C_A + 8 + i] - actual[COL_SAL + i]);
        }

        // Fila 0 del carril B: M(as_digest(SPEND_KEY_DOMAIN), clave). El rate ENTERO, y la
        // capacidad va de asercion, como la del carril A.
        let dominio_clave = E::from(BaseElement::new(SPEND_KEY_DOMAIN));
        result[C_KEY_IN] = primera_fila * (actual[C_B + 4] - dominio_clave);
        for i in 1..4 {
            result[C_KEY_IN + i] = primera_fila * actual[C_B + 4 + i];
        }
        for i in 0..4 {
            result[C_KEY_IN + 4 + i] = primera_fila * (actual[C_B + 8 + i] - actual[COL_KEY + i]);
        }

        // TITULARIDAD (D-AV): en la fila 7 el carril B entrega la identidad, y es el receptor.
        for i in 0..4 {
            result[C_PK_CHECK + i] =
                enlace_importe * (actual[C_B + 4 + i] - actual[COL_RECEPTOR + i]);
        }

        for (k, col) in TRANSPORTE.iter().enumerate() {
            result[C_TRANSPORT + k] = siguiente[*col] - actual[*col];
        }
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        let cero = BaseElement::ZERO;
        let mut a = Vec::with_capacity(NUM_ASERCIONES);

        // Las dos capacidades arrancan a cero: la del carril A y la del ciclo de la clave.
        for i in 0..4 {
            a.push(Assertion::single(C_A + i, 0, cero));
        }
        for i in 0..4 {
            a.push(Assertion::single(C_B + i, 0, cero));
        }
        // La subida entrega la raiz DECLARADA.
        for i in 0..4 {
            a.push(Assertion::single(C_A + 4 + i, ROW_RAIZ, self.pi.pending_root[i]));
        }
        // El receptor es el que declara QUIEN VERIFICA, y la marca, la que el sobre publica.
        for i in 0..4 {
            a.push(Assertion::single(COL_RECEPTOR + i, 0, self.pi.receptor[i]));
        }
        for i in 0..4 {
            a.push(Assertion::single(C_B + 4 + i, ROW_MARCA, self.pi.marca[i]));
        }

        debug_assert_eq!(a.len(), NUM_ASERCIONES);
        a
    }
}

// ------------------------------------------------------------------ el juez

/// **Verifica una prueba de prenda contra su enunciado.** Comprueba ANTES la forma de la traza
/// que la prueba declara -ancho, sin segmento auxiliar y longitud [`TRAZA`]- y acepta SOLO
/// [`opciones`]. Nunca entra en panico por lo que traiga la prueba. No hay enunciado que
/// comprobar aparte: la prenda no lleva cotas, asi que no hay resta que pueda dar la vuelta en
/// el campo.
pub fn verificar(prueba: &[u8], pi: &PrendaPublicInputs) -> Result<(), String> {
    let proof =
        Proof::from_bytes(prueba).map_err(|e| format!("la prueba no se deserializa: {e:?}"))?;
    let info = proof.trace_info();
    let forma = (
        info.main_trace_width(),
        info.aux_segment_width(),
        info.get_num_aux_segment_rand_elements(),
        info.length(),
    );
    if forma != (ANCHO, 0, 0, TRAZA) {
        return Err(format!(
            "forma de traza {forma:?}; el enunciado pide {:?}",
            (ANCHO, 0, 0, TRAZA)
        ));
    }
    let aceptadas = AcceptableOptions::OptionSet(vec![opciones()]);
    verify::<PrendaAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
        proof,
        pi.clone(),
        &aceptadas,
    )
    .map_err(|e| format!("{e:?}"))
}

// ------------------------------------------------------------------ el enlace (D-AV, D-AY)

/// **Lo que una cabeza v5 firma y la prenda necesita** (D-AV). Una sola raiz: la prenda no lleva
/// la meta. Se lee de una cabeza ya verificada; este crate no verifica firmas, las verifica el
/// kit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CabezaPrenda {
    pub pending_root: Digest,
}

/// **Lo que el sobre afirma** (D-AW): a nombre de `receptor`, sobre la hoja cuya marca es
/// `marca`. Ni `C2` ni el importe viajan.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AfirmacionPrenda {
    pub receptor: Digest,
    pub marca: Digest,
}

/// **La prueba de prenda contra una cabeza v5.** Compone el enunciado con la raiz que la cabeza
/// firma y con lo que el sobre afirma, y solo entonces llama al juez; devuelve el enunciado
/// verificado. Una prueba de otro arbol o de otra marca no se enlaza: la prueba es la de SU
/// enunciado.
///
/// ⚠️ **Lo que esta funcion NO hace, a proposito**: comprobar que `marca` este bajo el
/// `consRoot` de esa misma cabeza. Eso no lo puede saber quien solo tiene la prueba -- hace
/// falta el arbol de consumos-, y es la puerta de quien escribe en el, no la del juez. Una
/// prenda cuya marca no este publicada verifica y no vale nada: **el par es marca-bajo-la-raiz
/// MAS este sobre** (D-AS).
pub fn verificar_contra_cabeza(
    prueba: &[u8],
    af: &AfirmacionPrenda,
    cabeza: &CabezaPrenda,
) -> Result<PrendaPublicInputs, String> {
    let pi = PrendaPublicInputs {
        pending_root: cabeza.pending_root,
        receptor: af.receptor,
        marca: af.marca,
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

    fn enunciado() -> PrendaPublicInputs {
        PrendaPublicInputs { pending_root: d(1), receptor: d(3), marca: d(5) }
    }

    fn air() -> PrendaAir {
        PrendaAir::new(TraceInfo::new(ANCHO, TRAZA), enunciado(), opciones())
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

    /// Las aserciones atan el receptor, la marca y la raiz que declara quien verifica; y ninguna
    /// nombra la clave, la sal, el importe ni `X` (D-AW).
    #[test]
    fn las_aserciones_atan_lo_declarado_y_nada_del_testigo() {
        let a = air().get_assertions();
        assert_eq!(a.len(), NUM_ASERCIONES);
        let pi = enunciado();
        let mut esperadas = Vec::new();
        for i in 0..4 {
            esperadas.push(Assertion::single(COL_RECEPTOR + i, 0, pi.receptor[i]));
            esperadas.push(Assertion::single(C_A + 4 + i, ROW_RAIZ, pi.pending_root[i]));
            esperadas.push(Assertion::single(C_B + 4 + i, ROW_MARCA, pi.marca[i]));
        }
        for e in &esperadas {
            let e = format!("{e:?}");
            assert!(a.iter().any(|x| format!("{x:?}") == e), "sin la asercion {e}");
        }
        for x in &a {
            let col = x.column();
            assert!(
                !(COL_KEY..COL_KEY + 4).contains(&col)
                    && !(COL_SAL..COL_SAL + 4).contains(&col)
                    && !(COL_X..COL_X + 4).contains(&col)
                    && col != COL_IMPORTE,
                "una asercion sobre el testigo: columna {col}"
            );
        }
    }

    /// Las doce entradas publicas van al transcripto, cada una en su sitio; `C2` no esta, y la
    /// clave tampoco.
    #[test]
    fn las_entradas_publicas_son_doce_y_sin_testigo() {
        let pi = enunciado();
        let v = pi.to_elements();
        assert_eq!(v.len(), 12);
        assert_eq!(&v[0..4], &pi.pending_root);
        assert_eq!(&v[4..8], &pi.receptor);
        assert_eq!(&v[8..12], &pi.marca);
    }

    /// Una prueba que no es prueba se rechaza con `Err`, sin entrar en panico.
    #[test]
    fn lo_que_no_es_prueba_se_rechaza_sin_panico() {
        assert!(verificar(&[1, 2, 3], &enunciado()).is_err(), "tres bytes verificaron");
        assert!(verificar(&[], &enunciado()).is_err(), "cero bytes verificaron");
    }

    /// El enlace toma la raiz de la CABEZA, no la del sobre, y lo demas del sobre: dos cabezas
    /// distintas componen dos enunciados distintos aunque el sobre sea el mismo.
    #[test]
    fn el_enlace_compone_con_la_raiz_de_la_cabeza() {
        let af = AfirmacionPrenda { receptor: d(3), marca: d(5) };
        let uno = CabezaPrenda { pending_root: d(1) };
        let otro = CabezaPrenda { pending_root: d(7) };
        // La prueba es mentira en los dos casos, asi que los dos dan Err; lo que se mide es que
        // el enunciado que se compone es el de SU cabeza.
        assert!(verificar_contra_cabeza(&[1, 2, 3], &af, &uno).is_err());
        assert!(verificar_contra_cabeza(&[1, 2, 3], &af, &otro).is_err());
        let compuesto = |c: &CabezaPrenda| PrendaPublicInputs {
            pending_root: c.pending_root,
            receptor: af.receptor,
            marca: af.marca,
        };
        assert_ne!(compuesto(&uno), compuesto(&otro), "la cabeza no entra en el enunciado");
        assert_eq!(compuesto(&uno), enunciado());
    }

    /// La geometria de la D-AX y la D-AY, atada a su fuente y al molde del que sale.
    #[test]
    fn la_geometria_es_la_de_la_d_ax() {
        assert_eq!(ANCHO, 2 * ESTADO + 18);
        assert_eq!(TRAZA, crate::cobro_pendiente::TRAZA);
        assert_eq!(ROW_RAIZ, crate::cobro_pendiente::ROW_RAIZ);
        assert_eq!((ROW_RAIZ + 1).next_power_of_two(), TRAZA);
        assert_eq!(C_KEY_IN + 8, C_PK_CHECK);
        assert_eq!(C_PK_CHECK + 4, C_TRANSPORT);
        assert_eq!((C_HASH_B, C_CAP_A), (C_HASH_A + ESTADO, C_HASH_B + ESTADO));
    }

    /// El dominio sexto es el del REGISTRO, y no es el de la meta ni el de la clave.
    #[test]
    fn el_dominio_de_la_marca_es_el_suyo() {
        assert_eq!(DOMINIO_PRENDA, u64::from_be_bytes(*b"PREND_V1"));
        assert_ne!(DOMINIO_PRENDA, SPEND_KEY_DOMAIN);
        assert_ne!(DOMINIO_PRENDA, zk_ssl_hash::DOMINIO_META_PENDIENTE);
    }
}
