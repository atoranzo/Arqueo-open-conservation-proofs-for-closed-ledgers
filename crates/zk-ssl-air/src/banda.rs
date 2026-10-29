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
//!
//! ## La v2: el saldo a 62 bits (§723)
//!
//! El saldo es PRIVADO, y [`comprobar_enunciado`] solo acota los limites publicos. En la v1 los
//! tres segmentos de rango eran de 63 bits, y con un superior pequeno un saldo de la ventana
//! `[2^62, 2^63)` probaba la banda: medido, una hoja con saldo `2^63 - 2^32 + 2` probaba <<saldo en
//! `[0, 0]`>> y verificaba con [`verificar`]. Una transicion honesta no deja esa hoja, pero quien
//! firma la raiz a mano si. La v2 ([`BandaAir`]) fuerza a cero los DOS bits altos de cada segmento,
//! como el RFC-0017 hizo en las AIR vivas: el saldo y sus dos holguras caben en 62 bits, y una
//! resta que da la vuelta ya no cabe. La v1 ([`BandaAirV1`]) solo sigue aqui para juzgar las
//! pruebas ya emitidas, y [`verificar`] la acepta solo donde su ventana esta vacia
//! ([`crate::COTA_V1`]). La traza es la misma en las dos: con un saldo honesto, el bit 62 ya es
//! cero.

use winter_air::proof::Proof;
use winter_air::{
    Air, AirContext, Assertion, EvaluationFrame, ProofOptions, TraceInfo,
    TransitionConstraintDegree,
};
use winter_crypto::hashers::Rp64_256;
use winter_math::fields::f64::BaseElement;
use winter_math::{FieldElement, ToElements};

use crate::{ronda_rescue, Digest, CICLO, ESTADO, PROFUNDIDAD, RONDAS};

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

// Las celdas que ninguna restriccion mira. El censo de celdas (doc/fv/interprete_banda.py) exige
// cada una aqui: una celda sin dueno y sin declarar es un rojo del guardian. Todas son testigo por
// diseno SALVO la del §489, que es un fallo conocido y abierto, no un testigo.
// CELDAS_LIBRES: sal de hoja testigo, solo la lee el enlace de la fila 15 (clase *, cols 23..27)
// CELDAS_LIBRES: bit de camino: solo los enlaces de la subida lo miran (clase sin link_merkle ni link_place, col 12)
// CELDAS_LIBRES: descansos del acumulador de rango entre segmentos y tras ellos (clase sin cont_s, col 22)
// CELDAS_LIBRES: NO por diseno: limbos altos del rate en el enlace del nonce, la holgura conocida y abierta del §489, no explotable contra una raiz firmada; cerrarla cambia el AIR (clase cont_s+link_leaf, cols 9..12)
// CELDAS_LIBRES: carril muerto tras la raiz, capacidad (clase plana, cols 0..4)
// CELDAS_LIBRES: carril muerto tras la raiz, rate (clase plana, cols 8..12)

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

/// El AIR de la banda en su version `V`: la 1, con los segmentos de 63 bits, o la 2, con los de 62.
pub struct BandaAirDe<const V: u8> {
    context: AirContext<BaseElement>,
    pi: BandaPublicInputs,
}

/// **La banda que se prueba y se juzga hoy**: la v2, con el saldo y sus holguras a 62 bits.
pub type BandaAir = BandaAirDe<2>;

/// ⚠️ La v1, con los segmentos de 63 bits. Solo para juzgar las pruebas ya emitidas: con un
/// superior bajo [`crate::COTA_V1`] no acota el saldo privado. Nadie prueba con ella.
pub type BandaAirV1 = BandaAirDe<1>;

/// §913: la forma en claro de esta familia, para el juez unico (`crate::juez`); la v1 y la v2
/// tienen la misma traza.
impl<const V: u8> crate::juez::Juez for BandaAirDe<V> {
    const NOMBRE: &'static str = "BandaAir";
    fn forma(_: &BandaPublicInputs) -> crate::juez::Forma {
        crate::juez::Forma::fija(ANCHO, TRAZA)
    }
}

impl<const V: u8> Air for BandaAirDe<V> {
    type BaseField = BaseElement;
    type PublicInputs = BandaPublicInputs;

    fn new(trace_info: TraceInfo, pub_inputs: Self::PublicInputs, options: ProofOptions) -> Self {
        assert!(V == 1 || V == 2, "la banda no tiene version {V}");
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

        BandaAirDe {
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
            // ⚠️ La v2: tambien la segunda fila, y el bit 62 a cero. 62 bits, no 63 (la cabecera).
            if V >= 2 {
                primera_s[seg * LARGO_SEGMENTO + 1] = uno;
            }
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
///
/// Juzga con la v2 ([`BandaAir`]). Si no verifica, prueba la v1 ([`BandaAirV1`]), la de las pruebas
/// ya emitidas: con un superior en [`crate::COTA_V1`] o por encima la acepta, porque ahi dice lo
/// mismo que la v2; por debajo la rehusa con la causa [`crate::PRUEBA_V1_EN_LA_VENTANA`], porque un
/// saldo de la ventana `[2^62, 2^63)` tambien la pasa. La excepcion es [`PUBLICADAS_V1`]: una
/// prueba de la v1 ya publicada, reconocida por la huella de su contenido y su enunciado. Si
/// tampoco verifica con la v1, la causa es la de la version que llego mas lejos
/// ([`crate::la_causa_de_la_que_llego_mas_lejos`]).
pub fn verificar(prueba: &[u8], pi: &BandaPublicInputs) -> Result<(), String> {
    verificar_con(prueba, pi, PUBLICADAS_V1)
}

/// Una prueba de la v1 con el superior bajo [`crate::COTA_V1`] que ya se publico: la huella de su
/// contenido ([`crate::huella_del_contenido`]) y su enunciado entero. La raiz y la cuenta van como
/// los bytes de `digest_to_bytes`, en hex sin `0x`.
pub struct PublicadaV1 {
    /// Donde vive, para quien la busque.
    pub vector: &'static str,
    pub huella: &'static str,
    pub raiz: &'static str,
    pub cuenta: &'static str,
    pub inferior: u64,
    pub superior: u64,
}

/// **Las pruebas de la v1 bajo la cota que ya se publicaron, y SOLO esas** (§723, la
/// ventana del saldo privado). El juez no distingue una banda honesta de la v1 de una fabricada:
/// por eso rehusa la v1 bajo la cota. Pero esta prueba la capturo un banco de un nodo de la casa, y
/// su vector pide VERDE desde que se publico: su enunciado es verdadero. La lista cerrada la
/// reconoce por la huella de su CONTENIDO -sus compromisos: sin el testigo nadie abre otros que
/// verifiquen- y por su enunciado, y solo despues de que verifique con la v1. Un falsador no
/// reproduce la huella. ⚠️ No crece: una prueba nueva es de la v2. La segunda implementacion
/// lleva la misma lista (`PUBLICADAS_V1` de `tools/segunda/airs.py`; un test de aqui las compara).
pub const PUBLICADAS_V1: &[PublicadaV1] = &[PublicadaV1 {
    vector: "spec/vectors/0.4/rechazo/saldo-insuficiente.json", // §914: mudado a su version
    huella: "95d8fd37865d8d0c8322535b6b8c5edbd0a6eadee05c78dba1a27a19742c32de",
    raiz: "87b9805f534c39aecaa316fd0d8203f6feab81ba9a8ac7780f033ed465c69f1d",
    cuenta: "d38feef2725224670cef36f48f56313698363536f76355e95bea532a29660862",
    inferior: 0,
    superior: 1000,
}];

/// ¿Es esta prueba, con este enunciado, una de `publicadas`? Solo la pregunta quien ya la verifico
/// con la v1.
fn es_publicada_v1(prueba: &[u8], pi: &BandaPublicInputs, publicadas: &[PublicadaV1]) -> bool {
    let Some(huella) = crate::huella_del_contenido(prueba) else {
        return false;
    };
    let hex = |b: &[u8]| b.iter().map(|x| format!("{x:02x}")).collect::<String>();
    let (h, raiz, cuenta) = (
        hex(&huella),
        hex(&zk_ssl_hash::digest_to_bytes(&pi.root)),
        hex(&zk_ssl_hash::digest_to_bytes(&pi.public_id)),
    );
    publicadas.iter().any(|e| {
        e.huella == h
            && e.raiz == raiz
            && e.cuenta == cuenta
            && e.inferior == pi.lower.as_int()
            && e.superior == pi.upper.as_int()
    })
}

/// [`verificar`] con la lista de las publicadas como parametro: los tests la vacian o la tocan.
fn verificar_con(
    prueba: &[u8],
    pi: &BandaPublicInputs,
    publicadas: &[PublicadaV1],
) -> Result<(), String> {
    // §914 (RFC-0018 D-G): la marca v2 antes que nada, de la cabeza; despues el enunciado; despues
    // la prueba entera, sus opciones y su forma oculta, por el juez unico; los textos, los de siempre
    crate::marca_del_kit::<BandaAirDe<2>>(prueba)?;
    comprobar_enunciado(pi)?;
    let proof =
        Proof::from_bytes(prueba).map_err(|e| format!("la prueba no se deserializa: {e:?}"))?;
    crate::juez::comprobar_opciones(&proof).or_else(crate::rechazo_del_kit)?;
    crate::juez::comprobar_forma(proof.trace_info(), &<BandaAirDe<2> as crate::juez::Juez>::forma(pi))
        .or_else(crate::rechazo_del_kit)?;
    let v2 = crate::juez::juzgar::<BandaAirDe<2>>(proof.clone(), pi.clone()).or_else(crate::rechazo_del_kit);
    if v2.is_ok() {
        return v2;
    }
    let v1 = crate::juez::juzgar::<BandaAirDe<1>>(proof, pi.clone()).or_else(crate::rechazo_del_kit);
    if v1.is_ok() {
        let u = pi.upper.as_int();
        if crate::v1_sin_ventana(u) || es_publicada_v1(prueba, pi, publicadas) {
            return Ok(());
        }
        return Err(format!(
            "{}: con el superior {u}, bajo {}, no acota el saldo privado: un saldo de la ventana \
             [2^62, 2^63) tambien la pasa",
            crate::PRUEBA_V1_EN_LA_VENTANA,
            crate::COTA_V1
        ));
    }
    crate::la_causa_de_la_que_llego_mas_lejos(v2, v1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::opciones;

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

    /// **§723, la v2.** El selector del primer bit marca DOS filas por segmento en la v2 y
    /// una en la v1, y ninguna otra columna periodica cambia: las dos versiones son el mismo AIR
    /// salvo el bit 62 de cada segmento, que la v2 fuerza a cero. Falsador: sin el `if V >= 2`, la
    /// v2 es la v1 y la primera comparacion cae.
    #[test]
    fn la_v2_fuerza_a_cero_el_bit_62_de_cada_segmento() {
        let v2 = air().get_periodic_column_values();
        let v1 = BandaAirV1::new(TraceInfo::new(ANCHO, TRAZA), enunciado(), opciones())
            .get_periodic_column_values();
        let filas = |c: &[BaseElement]| -> Vec<usize> {
            c.iter().enumerate().filter(|(_, x)| **x == BaseElement::ONE).map(|(i, _)| i).collect()
        };
        let l = LARGO_SEGMENTO;
        assert_eq!(filas(&v2[P_FIRST_S]), vec![0, 1, l, l + 1, 2 * l, 2 * l + 1], "la v2");
        assert_eq!(filas(&v1[P_FIRST_S]), vec![0, l, 2 * l], "la v1");
        assert_eq!(v1.len(), v2.len());
        for (k, (a, b)) in v1.iter().zip(&v2).enumerate() {
            if k != P_FIRST_S {
                assert_eq!(a, b, "la columna periodica {k} cambia de la v1 a la v2");
            }
        }
    }

    /// El valor de texto de `clave` en un vector JSON: lo justo para leer cuatro campos sin
    /// `serde_json`, que este crate no lleva. Cada clave que se lee aparece una vez, o la primera es
    /// la buena (`requested` va en `campos` y en `banda`, con el mismo valor).
    fn campo<'a>(texto: &'a str, clave: &str) -> &'a str {
        let i = texto.find(&format!("\"{clave}\"")).unwrap_or_else(|| panic!("falta {clave}"));
        let resto = texto[i + clave.len() + 2..].trim_start().strip_prefix(':').expect(":");
        let resto = resto.trim_start().strip_prefix('"').expect("una cadena");
        &resto[..resto.find('"').expect("cierre")]
    }

    fn de_hex(t: &str) -> Vec<u8> {
        zk_ssl_hash::bytes_de_hex(t.strip_prefix("0x").unwrap_or(t)).expect("hex")
    }

    /// La prueba y el enunciado que el mando compone de un sobre de rechazo `InsufficientBalance`
    /// de `spec/vectors/rechazo/`: la raiz de la cabeza, la cuenta de la banda y `[0, pedido - 1]`.
    fn del_vector(fichero: &str) -> (Vec<u8>, BandaPublicInputs) {
        // §914: los vectores de zkssl/0.4 viven bajo su version (R6 b del RFC-0021)
        let ruta = format!("{}/../../spec/vectors/0.4/rechazo/{fichero}", env!("CARGO_MANIFEST_DIR"));
        let t = std::fs::read_to_string(&ruta).expect("el vector");
        let dg = |c: &str| zk_ssl_hash::digest_from_bytes(&de_hex(campo(&t, c))).expect("digest");
        let pedido = u64::from_str_radix(campo(&t, "requested").trim_start_matches("0x"), 16)
            .expect("requested");
        let pi = BandaPublicInputs {
            root: dg("accountsRoot"),
            public_id: dg("publicId"),
            lower: BaseElement::ZERO,
            upper: BaseElement::new(pedido - 1),
        };
        (de_hex(campo(&t, "prueba")), pi)
    }

    /// **§723, la v1 ya publicada; §914, bajo el kit 0.5.** La prueba de `saldo-insuficiente.json`
    /// es de la v1 con la marca v1, el superior 1000 bajo la cota: bajo el kit 0.4.x verificaba por
    /// la lista de [`PUBLICADAS_V1`] (su huella y su enunciado). Desde el corte zkssl/0.5 toda
    /// prueba de `zkssl/0.4` cae por su marca, antes del enunciado (RFC-0018 D-G, 1c; D-E, R6 e
    /// del RFC-0021): la juzga el kit 0.4.x, y la lista queda como lo que fue. Con la lista y sin
    /// ella, la misma causa; con otro superior, la misma causa.
    #[test]
    fn la_v1_publicada_cae_por_su_marca_bajo_el_kit_0_5() {
        let marca_v1 = |r: Result<(), String>| {
            r.as_ref().is_err_and(|e| e == "marca v1: prueba de zkssl/0.4; la juzga el kit 0.4.x")
        };
        let (prueba, pi) = del_vector("saldo-insuficiente.json");
        let e = &PUBLICADAS_V1[0];
        assert_eq!((pi.lower.as_int(), pi.upper.as_int()), (e.inferior, e.superior));
        assert!(!crate::v1_sin_ventana(e.superior), "la publicada esta bajo la cota");
        assert!(marca_v1(verificar(&prueba, &pi)), "la publicada, con la lista: por la marca");
        assert!(marca_v1(verificar_con(&prueba, &pi, &[])), "sin la lista, por la marca");
        let (falsa, pi_falsa) = del_vector("rechazo-rech-banda-ventana-v1.json");
        assert!(marca_v1(verificar(&falsa, &pi_falsa)), "la del operador, por la marca");
        let mut otro_superior = pi.clone();
        otro_superior.upper = BaseElement::new(999);
        assert!(marca_v1(verificar(&prueba, &otro_superior)), "con otro enunciado, por la marca");
    }

    /// La segunda implementacion lleva la MISMA lista: su `PUBLICADAS_V1` (`tools/segunda/airs.py`),
    /// sin comentarios ni blancos, es la de aqui. Falsador: con una tupla de mas, o un digito
    /// cambiado en `airs.py`, cae.
    #[test]
    fn la_lista_de_la_segunda_es_la_del_juez() {
        let ruta = format!("{}/../../tools/segunda/airs.py", env!("CARGO_MANIFEST_DIR"));
        let fuente = std::fs::read_to_string(ruta).expect("airs.py");
        let mut dentro = false;
        let mut leida = String::new();
        for l in fuente.lines() {
            if l.starts_with("PUBLICADAS_V1 = [") {
                dentro = true;
            }
            if dentro && !l.trim_start().starts_with('#') {
                leida.extend(l.chars().filter(|c| !c.is_whitespace()));
            }
            if dentro && l == "]" {
                break;
            }
        }
        let mut esperada = String::from("PUBLICADAS_V1=[");
        for e in PUBLICADAS_V1 {
            esperada.push_str(&format!(
                "(\"{}\",\"{}\",\"{}\",{},{}),",
                e.huella, e.raiz, e.cuenta, e.inferior, e.superior
            ));
        }
        esperada.push(']');
        assert_eq!(leida, esperada, "la lista de la segunda");
    }
}
