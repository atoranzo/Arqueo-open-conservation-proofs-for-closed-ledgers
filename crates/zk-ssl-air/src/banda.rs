//! **`BandaAir`: la banda del saldo bajo una raiz, SIN titularidad** (RFC-0007, E5, corte 4a).
//!
//! El enunciado, entero: **bajo la raiz `root` existe una hoja compuesta con la identidad
//! `public_id`, un nonce y un salt, cuyo saldo esta en `[lower, upper]`**. Ni una palabra de
//! titularidad, y es lo que hace util a este AIR: la causa `InsufficientBalance` la produce el
//! OPERADOR, que tiene la hoja, el nonce, el camino y el salt de la cuenta, y **no tiene la
//! clave**. Con el ciclo de titularidad dentro, el rechazo no seria producible.
//!
//! ## De donde sale y que se le quito (D-1, D-3)
//!
//! Es `crates/stark-experiment/src/circuit_audit.rs` -el AIR de la banda que la capa ya importa
//! y verifica- sin el ciclo de titularidad: fuera `COL_KEY` (4 columnas), el ciclo
//! `CYC_PK` con sus aserciones de `SPEND_KEY_DOMAIN`, y las familias `C_PK_INPUT` y
//! `C_PK_CHECK`. Aquel AIR **no se toca**: dos semanticas en una pieza es lo que la vara
//! prohibe, y su revelacion voluntaria es propiedad sellada.
//!
//! Ancho 31 -> **27** columnas; restricciones de transicion
//! 75 -> **63**. La tuberia acaba en la fila **279** y la
//! traza sigue midiendo **512** filas, porque 280 ya obligaban esa potencia
//! de dos: **el recorte ahorra ancho y restricciones, no filas**.
//!
//! ## Lo que ocupa el sitio del ciclo: CUATRO aserciones (D-2b)
//!
//! Quitar la titularidad sin poner nada dejaria un AIR que **no ata ninguna cuenta**. En el
//! original nada ata `COL_ID` a la entrada publica -sus aserciones no la nombran y el probador
//! la LEE de la propia traza-, y lo unico que hacia verdadera esa identidad era `C_PK_CHECK`;
//! su propio testigo lo dice: un tercero que conoce identidad, saldo, nonce, salt y camino de
//! una cuenta ajena construye una traza cuya raiz cuadra. Aqui la atan **cuatro aserciones de
//! frontera** contra el `public_id` que declara QUIEN VERIFICA, no el que el probador metio en
//! la traza. Cuestan cero columnas y cero restricciones de transicion, y con ellas las
//! aserciones siguen siendo **17**: salen cuatro de titularidad, entran cuatro de
//! identidad.
//!
//! ## Lo que NO prueba, y va escrito
//!
//! - Que quien produjo la prueba sea el titular. No puede y no debe: el enunciado es del
//!   ESTADO comprometido, no de la autorizacion.
//! - Nada sobre otra raiz que la declarada, ni sobre el saldo mas alla de la desigualdad.
//!
//! La traza y el `Prover` viven en `stark-experiment` (`circuit_banda`), como en el molde del
//! S463: el AIR tiene UN productor y el probador y el kit lo comparten.

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

use crate::{opciones, ronda_rescue, Blake3, Digest, CICLO, ESTADO, PROFUNDIDAD, RONDAS};

// ------------------------------------------------------------------ la geometria
//
// Publica, y no por comodidad: el probador vive en OTRO crate (`stark-experiment`,
// `circuit_banda`) y nombra estas columnas para rellenar la traza. En `circuit_audit`
// eran privadas porque la traza vivia en el mismo fichero; el hermano de este crate
// publica las suyas por la misma razon.
//
// DERIVADA del fichero real por el generador (`derivar_banda.py`): ninguna de estas cifras
// esta tecleada, y las seis del calendario reproducen las que la D-3 declaraba como deduccion.

/// Filas de la traza. La tuberia acaba en [`ROW_RAIZ`] y el resto es holgura.
pub const TRAZA: usize = 512;
/// Bits por segmento de rango.
pub const LARGO_SEGMENTO: usize = 64;
/// Segmentos: saldo, saldo - inferior, superior - saldo.
pub const SEGMENTOS: usize = 3;
/// Techo de valor representable sin que una resta de la vuelta en el campo.
pub const MAX_VALOR: u64 = 0x3fffffffffffffff;

// Columnas. El hueco de la clave (4 ranuras) se cierra: todo lo que vivia por
// encima baja 4.
pub const COL_BIT: usize = 12;
pub const COL_ID: usize = 13; // 13..17
pub const COL_BAL: usize = 17;
pub const COL_NONCE: usize = 18;
pub const COL_LOWER: usize = 19;
pub const COL_UPPER: usize = 20;
pub const COL_SBIT: usize = 21;
pub const COL_SACC: usize = 22;
pub const COL_LEAF_SALT: usize = 23;
/// Ancho de la traza principal.
pub const ANCHO: usize = 27;

// Filas de evento. Sin el ciclo de titularidad la tuberia cierra en la raiz.
pub const CYC_ACC: usize = 3;
pub const CYC_RAIZ: usize = 35;
pub const ROW_ENLACE_HOJA: usize = 7;
pub const ROW_ENLACE_SALT: usize = 15;
pub const ROW_HOJA_LISTA: usize = 23;
/// Fila en la que la subida entrega la raiz.
pub const ROW_RAIZ: usize = 279;

// El presupuesto, en compilacion: la tuberia cabe en la traza.
const _: () = assert!(ROW_RAIZ < TRAZA);
const _: () = assert!(CYC_RAIZ == CYC_ACC + PROFUNDIDAD);

// Familias de restriccion.
const C_HASH: usize = 0;
const C_TREE_CAP: usize = 12;
const C_PLACE: usize = 16;
const C_BIT_BOOL: usize = 20;
const C_LEAF_CAP: usize = 21;
const C_LEAF_DIG: usize = 25;
const C_NONCE: usize = 29;
const C_INPUT: usize = 30;
const C_TRANSPORT: usize = 35;
const C_ID_CONST: usize = 39;
const C_SBIT_BOOL: usize = 43;
const C_FIRST_S: usize = 45;
const C_HORNER: usize = 47;
const C_SEG_LINK: usize = 48;
const C_SALT_CAP: usize = 51;
const C_SALT_DIG: usize = 55;
const C_SALT_IN: usize = 59;
/// Restricciones de transicion declaradas.
pub const NUM_RESTRICCIONES: usize = 63;

// Columnas periodicas.
const P_HASH_FLAG: usize = 0;
const P_ARK1: usize = 1;
const P_ARK2: usize = 13;
const P_LINK_MERKLE: usize = 25;
const P_LINK_LEAF: usize = 26;
const P_LINK_SALT: usize = 27;
const P_LINK_PLACE: usize = 28;
const P_FIRST_ROW: usize = 29;
const P_FIRST_S: usize = 30;
const P_CONT_S: usize = 31;
const P_SEG_LINK: usize = 32;
/// Columnas periodicas declaradas.
pub const NUM_PERIODICAS: usize = 35;

/// Aserciones de frontera. **No se mueven con el recorte**: salen las cuatro de titularidad y
/// entran las cuatro de identidad (D-2b).
pub const NUM_ASERCIONES: usize = 17;

// ------------------------------------------------------------------ las entradas publicas

/// **Lo que el juez declara.** Las mismas cuatro que el AIR de auditoria: la raiz del estado
/// comprometido, la identidad de la cuenta y los dos limites de la banda. El productor de un
/// rechazo fija `lower = 0` y `upper = pedido - 1`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BandaPublicInputs {
    pub root: Digest,
    pub public_id: Digest,
    pub lower: BaseElement,
    pub upper: BaseElement,
}

impl ToElements<BaseElement> for BandaPublicInputs {
    fn to_elements(&self) -> Vec<BaseElement> {
        let mut v = self.root.to_vec();
        v.extend_from_slice(&self.public_id);
        v.push(self.lower);
        v.push(self.upper);
        v
    }
}

/// Lo que el enunciado tiene que cumplir ANTES de construir el AIR: fuera de estos rangos una
/// resta daria la vuelta en el campo y la banda no diria lo que parece. Nunca entra en panico.
pub fn comprobar_enunciado(pi: &BandaPublicInputs) -> Result<(), String> {
    let (l, u) = (pi.lower.as_int(), pi.upper.as_int());
    if l > MAX_VALOR || u > MAX_VALOR {
        return Err(format!("los limites {l} y {u} pasan del techo {MAX_VALOR}"));
    }
    if l > u {
        return Err(format!("banda vacia: inferior {l} sobre superior {u}"));
    }
    Ok(())
}

// ------------------------------------------------------------------ el AIR

pub struct BandaAir {
    context: AirContext<BaseElement>,
    pi: BandaPublicInputs,
}

impl Air for BandaAir {
    type BaseField = BaseElement;
    type PublicInputs = BandaPublicInputs;

    fn new(trace_info: TraceInfo, pub_inputs: Self::PublicInputs, options: ProofOptions) -> Self {
        assert_eq!(ANCHO, trace_info.width());
        let entera = vec![TRAZA];
        let ciclo = |d: usize| TransitionConstraintDegree::with_cycles(d, entera.clone());

        let mut grados = Vec::with_capacity(NUM_RESTRICCIONES);
        for _ in 0..12 {
            grados.push(ciclo(7));
        }
        for _ in 0..4 {
            grados.push(ciclo(1));
        }
        for _ in 0..4 {
            grados.push(ciclo(2));
        }
        for _ in 0..1 {
            grados.push(TransitionConstraintDegree::new(2));
        }
        for _ in 0..14 {
            grados.push(ciclo(1));
        }
        for _ in 0..8 {
            grados.push(TransitionConstraintDegree::new(1));
        }
        for _ in 0..2 {
            grados.push(TransitionConstraintDegree::new(2));
        }
        for _ in 0..6 {
            grados.push(ciclo(1));
        }
        for _ in 0..12 {
            grados.push(ciclo(1));
        }
        assert_eq!(grados.len(), NUM_RESTRICCIONES, "cuenta de grados");

        BandaAir {
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

        let mut bandera = vec![cero; TRAZA];
        for r in 0..=ROW_RAIZ {
            if r % CICLO < RONDAS {
                bandera[r] = uno;
            }
        }
        columnas.push(bandera);

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

        for fila in [ROW_ENLACE_HOJA, ROW_ENLACE_SALT, ROW_HOJA_LISTA, 0] {
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

        let bandera = periodic[P_HASH_FLAG];
        let ark1 = &periodic[P_ARK1..P_ARK1 + ESTADO];
        let ark2 = &periodic[P_ARK2..P_ARK2 + ESTADO];
        let enlace_arbol = periodic[P_LINK_MERKLE];
        let enlace_hoja = periodic[P_LINK_LEAF];
        let enlace_salt = periodic[P_LINK_SALT];
        let enlace_sitio = periodic[P_LINK_PLACE];
        let primera_fila = periodic[P_FIRST_ROW];
        let primera_s = periodic[P_FIRST_S];
        let sigue_s = periodic[P_CONT_S];

        // Las rondas de Rescue, por el UNICO productor del crate: la deuda de las veintiocho
        // copias en linea (5.A-182) no se reincide aqui.
        ronda_rescue(
            actual,
            siguiente,
            ark1,
            ark2,
            bandera,
            &mut result[C_HASH..C_HASH + ESTADO],
        );

        let bit = siguiente[COL_BIT];
        let enlace = enlace_arbol + enlace_sitio;

        for i in 0..4 {
            result[C_TREE_CAP + i] = enlace * siguiente[i];
            let d = actual[4 + i];
            result[C_PLACE + i] = enlace
                * ((E::ONE - bit) * (siguiente[4 + i] - d) + bit * (siguiente[8 + i] - d));
        }

        result[C_BIT_BOOL] = actual[COL_BIT] * (actual[COL_BIT] - E::ONE);

        for i in 0..4 {
            result[C_LEAF_CAP + i] = enlace_hoja * siguiente[i];
            result[C_LEAF_DIG + i] = enlace_hoja * (siguiente[4 + i] - actual[4 + i]);
        }
        result[C_NONCE] = enlace_hoja * (siguiente[8] - actual[COL_NONCE]);

        // La envoltura de la hoja: capacidad a cero, digest arrastrado y los CUATRO limbos del
        // rate atados al salt testigo.
        for i in 0..4 {
            result[C_SALT_CAP + i] = enlace_salt * siguiente[i];
            result[C_SALT_DIG + i] = enlace_salt * (siguiente[4 + i] - actual[4 + i]);
            result[C_SALT_IN + i] = enlace_salt * (siguiente[8 + i] - actual[COL_LEAF_SALT + i]);
        }

        // Entradas de la hoja: identidad completa + saldo.
        for i in 0..4 {
            result[C_INPUT + i] = primera_fila * (actual[4 + i] - actual[COL_ID + i]);
        }
        result[C_INPUT + 4] = primera_fila * (actual[8] - actual[COL_BAL]);

        let transporte = [
            COL_BAL,
            COL_NONCE,
            COL_LOWER,
            COL_UPPER,
        ];
        for (k, col) in transporte.iter().enumerate() {
            result[C_TRANSPORT + k] = siguiente[*col] - actual[*col];
        }
        for i in 0..4 {
            result[C_ID_CONST + i] = siguiente[COL_ID + i] - actual[COL_ID + i];
        }

        // ===== LA BANDA: inferior <= saldo <= superior =====
        let sbit_act = actual[COL_SBIT];
        let sbit_sig = siguiente[COL_SBIT];
        let sacc_act = actual[COL_SACC];
        let sacc_sig = siguiente[COL_SACC];

        result[C_SBIT_BOOL] = sbit_act * (sbit_act - E::ONE);
        result[C_SBIT_BOOL + 1] = sbit_sig * (sbit_sig - E::ONE);
        result[C_FIRST_S] = primera_s * sbit_act;
        result[C_FIRST_S + 1] = primera_s * sacc_act;
        result[C_HORNER] = sigue_s * (sacc_sig - (sacc_act + sacc_act + sbit_sig));

        let esperado = [
            actual[COL_BAL],
            actual[COL_BAL] - actual[COL_LOWER],
            actual[COL_UPPER] - actual[COL_BAL],
        ];
        for seg in 0..SEGMENTOS {
            result[C_SEG_LINK + seg] = periodic[P_SEG_LINK + seg] * (sacc_sig - esperado[seg]);
        }
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        let cero = BaseElement::ZERO;
        let mut a = Vec::with_capacity(NUM_ASERCIONES);

        // La capacidad del estado arranca a cero, y el relleno del rate tambien.
        for i in 0..4 {
            a.push(Assertion::single(i, 0, cero));
        }
        for i in 9..ESTADO {
            a.push(Assertion::single(i, 0, cero));
        }
        // La subida entrega la raiz DECLARADA.
        for i in 0..4 {
            a.push(Assertion::single(4 + i, ROW_RAIZ, self.pi.root[i]));
        }
        // **D-2b**: la identidad de la traza es la que declara quien verifica. Es lo unico que
        // ata la cuenta desde que el ciclo de titularidad salio; sin estas cuatro, cualquiera
        // probaria la banda de cualquiera bajo la misma raiz.
        for i in 0..4 {
            a.push(Assertion::single(COL_ID + i, 0, self.pi.public_id[i]));
        }
        a.push(Assertion::single(COL_LOWER, 0, self.pi.lower));
        a.push(Assertion::single(COL_UPPER, 0, self.pi.upper));

        debug_assert_eq!(a.len(), NUM_ASERCIONES);
        a
    }
}

// ------------------------------------------------------------------ el juez

/// **Verifica una prueba de banda contra su enunciado.** Comprueba ANTES la forma de la traza
/// que la prueba declara -ancho, sin segmento auxiliar y longitud [`TRAZA`]- para que un
/// fichero ajeno no llegue a construir el AIR, y acepta SOLO [`opciones`]. Nunca entra en
/// panico por lo que traiga la prueba.
pub fn verificar(prueba: &[u8], pi: &BandaPublicInputs) -> Result<(), String> {
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
    if forma != (ANCHO, 0, 0, TRAZA) {
        return Err(format!(
            "forma de traza {forma:?}; el enunciado pide {:?}",
            (ANCHO, 0, 0, TRAZA)
        ));
    }
    let aceptadas = AcceptableOptions::OptionSet(vec![opciones()]);
    verify::<BandaAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
        proof,
        pi.clone(),
        &aceptadas,
    )
    .map_err(|e| format!("{e:?}"))
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

    fn enunciado() -> BandaPublicInputs {
        BandaPublicInputs {
            root: d(1),
            public_id: d(2),
            lower: BaseElement::ZERO,
            upper: BaseElement::new(4_999_999),
        }
    }

    fn air() -> BandaAir {
        BandaAir::new(TraceInfo::new(ANCHO, TRAZA), enunciado(), opciones())
    }

    /// Cada ranura declarada se escribe: con un centinela en `result`, ninguna sobrevive a una
    /// evaluacion sobre un marco cualquiera, y el numero de ranuras es el que el contexto
    /// cuenta.
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

    /// **D-2b, ensenada:** las aserciones son las mismas que las del AIR de auditoria en
    /// numero, y CUATRO de ellas atan `COL_ID` en la fila 0 al `public_id` que declara quien
    /// verifica. Si esas cuatro no estuvieran, el AIR no ataria ninguna cuenta.
    #[test]
    fn las_cuatro_aserciones_atan_la_identidad_declarada() {
        let a = air().get_assertions();
        assert_eq!(a.len(), NUM_ASERCIONES);
        let pi = enunciado();
        for i in 0..4 {
            let esperada = Assertion::single(COL_ID + i, 0, pi.public_id[i]);
            assert!(
                a.iter().any(|x| format!("{x:?}") == format!("{esperada:?}")),
                "sin la asercion de identidad {i}"
            );
        }
        let ajena = Assertion::single(COL_ID, 0, BaseElement::new(0xBADC0DE));
        assert!(
            !a.iter().any(|x| format!("{x:?}") == format!("{ajena:?}")),
            "una identidad que nadie declaro"
        );
    }

    /// Un enunciado fuera de rango y una prueba que no es prueba se rechazan con `Err`, sin
    /// construir el AIR ni entrar en panico.
    #[test]
    fn lo_que_no_tiene_forma_se_rechaza_sin_panico() {
        let bien = enunciado();
        let malos = [
            BandaPublicInputs { lower: BaseElement::new(10), upper: BaseElement::new(9), ..bien.clone() },
            BandaPublicInputs { upper: BaseElement::new(MAX_VALOR + 1), ..bien.clone() },
        ];
        for malo in malos {
            assert!(comprobar_enunciado(&malo).is_err(), "aceptado: {malo:?}");
        }
        assert!(comprobar_enunciado(&bien).is_ok());
        assert!(verificar(&[1, 2, 3], &bien).is_err(), "tres bytes verificaron");
    }

    /// Las diez entradas publicas van al transcripto, cada una en su sitio.
    #[test]
    fn las_entradas_publicas_van_enteras_al_transcripto() {
        let pi = enunciado();
        let v = pi.to_elements();
        assert_eq!(v.len(), 10);
        assert_eq!(&v[0..4], &pi.root);
        assert_eq!(&v[4..8], &pi.public_id);
        assert_eq!(v[8], pi.lower);
        assert_eq!(v[9], pi.upper);
    }

    /// La geometria del recorte, atada a su fuente: la tuberia cierra en la raiz y la traza no
    /// encoge, porque las filas que quedan siguen obligando la misma potencia de dos.
    #[test]
    fn la_geometria_del_recorte_cierra() {
        assert_eq!(CYC_RAIZ, CYC_ACC + PROFUNDIDAD);
        assert_eq!(ROW_RAIZ, CYC_RAIZ * CICLO - 1);
        assert_eq!((ROW_RAIZ + 1).next_power_of_two(), TRAZA);
        assert_eq!(ANCHO, COL_LEAF_SALT + 4);
        assert_eq!(NUM_RESTRICCIONES, C_SALT_IN + 4);
    }
}
