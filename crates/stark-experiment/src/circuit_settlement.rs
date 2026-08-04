//! **Circuito de liquidación en STARK, Etapa 1**: partida doble con
//! autoridad de gasto.
//!
//! Port de `zk-core::circuit_settlement` al paradigma AIR, que es el
//! elegido para la capa por ser el único **sin ceremonia de confianza** y
//! con resistencia cuántica.
//!
//! ## El problema de seguridad que este port destapó
//!
//! En `zk-core` la identidad de cuenta es **un elemento de BLS12-381**:
//! 255 bits. Suficiente para que sea inviable encontrar otra clave con la
//! misma identidad.
//!
//! **En Goldilocks un elemento son 64 bits.** Si la identidad fuera un
//! solo elemento, encontrar `sk'` con la misma `pk` costaría unas 2^32
//! operaciones por la paradoja del cumpleaños — factible en un portátil.
//! **Un atacante podría fabricar una clave que controla una cuenta
//! ajena.**
//!
//! Es el mismo tipo de hallazgo que el techo de 63 bits de solidez: el
//! campo estrecho de Goldilocks tiene consecuencias que no existen en
//! curvas elípticas, y solo aparecen al portar.
//!
//! **Corrección**: la identidad es el **digest completo de 4 elementos**
//! (256 bits). Rescue ya los devuelve así, de modo que encaja con la
//! estructura de hash existente:
//!
//! ```text
//! pk        = Rescue(DOMAIN_PK, sk)            → digest de 4 elementos
//! leaf      = Rescue(Rescue(pk, saldo), nonce)
//! nullifier = Rescue(Rescue(DOMAIN_NULL, sk), nonce)
//! ```
//!
//! ## Estructura de la traza (48 columnas × 1024 filas)
//!
//! | Ciclos | Filas | Fase |
//! |---|---|---|
//! | 0-1 | 0..15 | Hojas del emisor (A = antigua, B = nueva) |
//! | 2-33 | 16..271 | Subida dual del emisor |
//! | 34-35 | 272..287 | Hojas del receptor |
//! | 36-67 | 288..543 | Subida dual del receptor |
//! | 68-69 | 544..559 | Nullifier, derivado de `sk` |
//! | 70 | 560..567 | Derivación de `pk` desde `sk` (AUTORIDAD) |
//!
//! 568 filas activas de 1024. Queda sitio para la Etapa 2 (árbol de
//! nullifiers), que necesitará 32 ciclos más.
//!
//! ## La autoridad de gasto, y dónde vive
//!
//! El último ciclo calcula `pk = Rescue(DOMAIN_PK, sk)` y una restricción
//! impone que coincida con la identidad usada en la hoja. Sin conocer
//! `sk` es imposible satisfacerla, por mucho que se conozcan el saldo, el
//! nonce y el camino de Merkle.
//!
//! Y el nullifier se deriva de `sk`, no de la identidad pública: solo el
//! titular puede calcularlo, así que el registro de gastados deja de ser
//! un oráculo de vigilancia.
//!
//! ## Etapa 2, pendiente
//!
//! Falta la no-pertenencia del nullifier (ya verificada de forma aislada
//! en `nullifier_tree`). Hasta integrarla, el doble gasto sigue
//! dependiendo de un registro externo.

use winterfell::crypto::hashers::{Blake3_256, Rp64_256};
use winterfell::crypto::{DefaultRandomCoin, MerkleTree};
use winterfell::math::{fields::f64::BaseElement, FieldElement, ToElements};
use winterfell::matrix::ColMatrix;
use winterfell::{
    Air, AirContext, Assertion, AuxRandElements, CompositionPoly, CompositionPolyTrace,
    ConstraintCompositionCoefficients, DefaultConstraintCommitment, DefaultConstraintEvaluator,
    DefaultTraceLde, EvaluationFrame, PartitionOptions, ProofOptions, Prover, StarkDomain,
    TraceInfo, TracePolyTable, TraceTable, TransitionConstraintDegree,
};

use crate::merkle::{native_merge, Digest, MerklePath, TREE_DEPTH};
use crate::circuit_freeze::FROZEN_DEPTH;
use crate::nullifier::NULLIFIER_DOMAIN;
use crate::rescue_hash::{apply_sbox, NUM_ROUNDS, STATE_WIDTH};

/// Dominio de derivación de la identidad desde la clave de gasto.
pub const SPEND_KEY_DOMAIN: u64 = 0x53504B59; // "SPKY"

pub const CYCLE_LENGTH: usize = 8;
pub const TRACE_LENGTH: usize = 1024;
pub const SEGMENT_LENGTH: usize = 64;
pub const NUM_SEGMENTS: usize = 7;

// ===== Columnas =====
const LANE_B: usize = STATE_WIDTH; // 12
const COL_BIT: usize = 24;
/// Clave de gasto del emisor. **El testigo más sensible del sistema.**
///
/// ⚠️ **CUATRO elementos desde el 31-07-2026** (entrada 15, §82, §90). Era
/// uno, y eso dejaba el espacio de secretos en 2^64 con `pk` publica:
/// agotarlo costaba 2^63, medido en 2,38 millones de años-nucleo.
///
/// Ensanchar **no invalida cuentas**: rellenar con ceros da la misma
/// identidad (§90.2). Lo que hace falta es **rotar la clave**, y hasta
/// entonces la seguridad es la que era.
const COL_S_KEY: usize = 25; // 25..29
/// Identidad del emisor: 4 elementos, no uno. Ver la nota de cabecera.
const COL_S_ID: usize = 29; // 29..33
const COL_S_BAL: usize = 33;
const COL_S_NONCE: usize = 34;
const COL_S_BAL_NEW: usize = 35;
/// Identidad del receptor: 4 elementos.
const COL_R_ID: usize = 36; // 36..40
const COL_R_BAL: usize = 40;
const COL_R_NONCE: usize = 41;
const COL_R_BAL_NEW: usize = 42;
const COL_AMT: usize = 43;
const COL_LIM: usize = 44;
const COL_ROOT_MID: usize = 45; // 45..49
const COL_SBIT: usize = 49;
const COL_SACC: usize = 50;
/// Bit de dirección del camino en el árbol de CONGELADOS.
///
/// Columna propia porque ese árbol tiene profundidad 24 y se recorre en
/// una fase distinta del de cuentas.
const COL_FBIT: usize = 51;
pub const TRACE_WIDTH: usize = 52;

// ===== Filas de eventos =====
const ROW_S_LEAF_LINK: usize = 7;
const ROW_S_LEAF_DONE: usize = 15;
const ROW_S_ROOT: usize = 271;
const ROW_R_LEAF_LINK: usize = 279;
const ROW_R_LEAF_DONE: usize = 287;
const ROW_R_ROOT: usize = 543;
/// La derivación de `pk` va ANTES del nullifier: así el nullifier queda
/// justo antes de su árbol y no hay que transportarlo con 4 columnas más.
const ROW_PK_START: usize = 544;
const ROW_PK_DONE: usize = 551;
const ROW_NULL_START: usize = 552;
const ROW_NULL_LINK: usize = 559;
/// Fila donde el nullifier está completo. Su enlace lo coloca en el nivel
/// 0 del árbol de nullifiers.
const ROW_NULLIFIER: usize = 567;
/// Última fila activa: raíz del árbol de nullifiers.
const ROW_NULL_ROOT: usize = 823;
/// **Fase de no-pertenencia al árbol de CONGELADOS.**
///
/// Ocupa las filas 824..1015, que estaban libres. Por eso el árbol tiene
/// profundidad 24 y no 32: así **no hay que agrandar la traza**, y el
/// coste de generación no se duplica.
const ROW_FROZEN_ROOT: usize = 1015;

// ===== Índices de restricción, derivados unos de otros =====
const C_HASH_A: usize = 0;
const C_HASH_B: usize = C_HASH_A + STATE_WIDTH;
const C_TREE_CAP_A: usize = C_HASH_B + STATE_WIDTH;
const C_TREE_CAP_B: usize = C_TREE_CAP_A + 4;
const C_PLACE_A: usize = C_TREE_CAP_B + 4;
const C_PLACE_B: usize = C_PLACE_A + 4;
const C_SIBLING: usize = C_PLACE_B + 4;
const C_BIT_BOOL: usize = C_SIBLING + 4;
const C_LEAF_CAP_A: usize = C_BIT_BOOL + 1;
const C_LEAF_CAP_B: usize = C_LEAF_CAP_A + 4;
const C_LEAF_DIG_A: usize = C_LEAF_CAP_B + 4;
const C_LEAF_DIG_B: usize = C_LEAF_DIG_A + 4;
/// Nonces: hoja del emisor (A, B), del receptor (A, B), nullifier (A, B).
const C_NONCE: usize = C_LEAF_DIG_B + 4; // 6
/// Entradas de la hoja del emisor: pk (4) + saldo, por carril.
const C_S_INPUT: usize = C_NONCE + 6; // 10
/// Entradas de la hoja del receptor: id (4) + saldo, por carril.
const C_R_INPUT: usize = C_S_INPUT + 10; // 10
const C_MID_CAPTURE: usize = C_R_INPUT + 10; // 4
const C_MID_CHECK: usize = C_MID_CAPTURE + 4; // 4
const C_CONSERVATION: usize = C_MID_CHECK + 4; // 2
/// Transporte escalar: s_key (**4**), s_bal, s_nonce, s_bal_new, r_bal,
/// r_nonce, r_bal_new, amt, lim.
///
/// ⚠️ Eran 9. La clave ocupa ahora cuatro columnas y las cuatro tienen que
/// ser constantes, o el compromiso no seria el declarado (§90).
const C_TRANSPORT: usize = C_CONSERVATION + 2; // 12
/// Transporte de identidades: 4 + 4.
const C_ID_CONST: usize = C_TRANSPORT + 12; // 8
const C_MID_CONST: usize = C_ID_CONST + 8; // 4
/// La CLAVE entra en el nullifier (ambos carriles, **4 elementos**).
const C_NULL_KEY: usize = C_MID_CONST + 4; // 8
/// La clave entra en la derivación de pk (ambos carriles, **4 elementos**).
const C_PK_INPUT: usize = C_NULL_KEY + 8; // 8
/// **AUTORIDAD**: la pk derivada coincide con la identidad usada.
const C_PK_CHECK: usize = C_PK_INPUT + 8; // 4
/// **NO-PERTENENCIA**: al entrar en el árbol de nullifiers, el carril A
/// coloca CERO — la posición estaba libre.
const C_NULL_EMPTY: usize = C_PK_CHECK + 4; // 4
const C_SBIT_BOOL: usize = C_NULL_EMPTY + 4; // 2
const C_FIRST_S: usize = C_SBIT_BOOL + 2; // 2
const C_HORNER: usize = C_FIRST_S + 2; // 1
const C_SEG_LINK: usize = C_HORNER + 1; // NUM_SEGMENTS
/// Capacidad a cero en la fase de congelados.
const C_FROZEN_CAP: usize = C_SEG_LINK + NUM_SEGMENTS; // 4
/// **LA NO-PERTENENCIA.** En la fila de entrada, la hoja colocada debe
/// ser CERO: si el emisor estuviera congelado, su hoja no lo sería.
const C_FROZEN_ENTRY: usize = C_FROZEN_CAP + 4; // 4
/// Colocación en cada nivel del árbol de congelados.
const C_FROZEN_PLACE: usize = C_FROZEN_ENTRY + 4; // 4
const C_FBIT_BOOL: usize = C_FROZEN_PLACE + 4; // 1
const NUM_CONSTRAINTS: usize = C_FBIT_BOOL + 1;

// ===== Columnas periódicas =====
const P_HASH_FLAG: usize = 0;
const P_ARK1: usize = 1;
const P_ARK2: usize = P_ARK1 + STATE_WIDTH;
const P_LINK_MERKLE: usize = P_ARK2 + STATE_WIDTH;
const P_LINK_LEAF: usize = P_LINK_MERKLE + 1;
const P_LINK_PLACE: usize = P_LINK_LEAF + 1;
const P_SEL_S_LEAF: usize = P_LINK_PLACE + 1;
const P_SEL_R_LEAF: usize = P_SEL_S_LEAF + 1;
const P_SEL_NULL_LEAF: usize = P_SEL_R_LEAF + 1;
const P_FIRST_ROW: usize = P_SEL_NULL_LEAF + 1;
const P_SEL_S_ROOT: usize = P_FIRST_ROW + 1;
const P_SEL_R_ROOT: usize = P_SEL_S_ROOT + 1;
const P_SEL_PK_START: usize = P_SEL_R_ROOT + 1;
const P_SEL_PK_DONE: usize = P_SEL_PK_START + 1;
/// Enlace que introduce el nullifier en su árbol.
const P_NULL_PLACE: usize = P_SEL_PK_DONE + 1;
const P_FIRST_S: usize = P_NULL_PLACE + 1;
const P_CONT_S: usize = P_FIRST_S + 1;
const P_SEG_LINK: usize = P_CONT_S + 1;
/// Fila que entra al árbol de congelados (la 823).
const P_FROZEN_ENTRY: usize = P_SEG_LINK + NUM_SEGMENTS;
/// Enlaces de la subida al árbol de congelados.
const P_FROZEN_LINK: usize = P_FROZEN_ENTRY + 1;

type Blake3 = Blake3_256<BaseElement>;

fn as_digest(x: BaseElement) -> Digest {
    [x, BaseElement::ZERO, BaseElement::ZERO, BaseElement::ZERO]
}

/// Identidad de cuenta desde la clave de gasto. **Digest completo.**
pub fn derive_public_id(spend_key: BaseElement) -> Digest {
    native_merge(
        as_digest(BaseElement::new(SPEND_KEY_DOMAIN)),
        as_digest(spend_key),
    )
}

/// **Identidad desde una clave de CUATRO elementos** (entrada 15, §82).
///
/// La estrecha toma un solo elemento de Goldilocks: **2^64**, y `pk` es
/// publica, asi que agotar el espacio cuesta 2^63 —2,38 millones de
/// años-nucleo medidos en §82.3, cota floja—.
///
/// ⚠️ **Es una generalizacion, no un reemplazo**: rellenando con ceros
/// devuelve **exactamente lo mismo** que la estrecha, y hay test que lo fija
/// (`the_wide_derivation_generalises_the_narrow_one`). De ahi que migrar
/// **no invalide cuentas**.
///
/// ⚠️ **Pero conservar la identidad no conserva la seguridad.** Una clave
/// rellenada con ceros sigue teniendo 64 bits de entropia. Lo que la version
/// ancha permite es **generar claves de 256 bits**; las viejas hay que
/// rotarlas, y hasta entonces valen lo que valian.
pub fn derive_public_id_wide(spend_key: Digest) -> Digest {
    native_merge(as_digest(BaseElement::new(SPEND_KEY_DOMAIN)), spend_key)
}

/// Nullifier desde una clave de cuatro elementos.
///
/// Misma estructura que el estrecho —dominio, clave, nonce— con la clave
/// ocupando el digest entero en vez de su primer elemento.
pub fn native_nullifier_wide(spend_key: Digest, nonce: BaseElement) -> Digest {
    let inner = native_merge(as_digest(BaseElement::new(NULLIFIER_DOMAIN)), spend_key);
    native_merge(inner, as_digest(nonce))
}

/// Hoja de cuenta: `Rescue(Rescue(pk, saldo), nonce)`.
pub fn native_leaf(public_id: Digest, balance: BaseElement, nonce: BaseElement) -> Digest {
    let inner = native_merge(public_id, as_digest(balance));
    native_merge(inner, as_digest(nonce))
}

/// Nullifier desde la CLAVE, no desde la identidad pública.
pub fn native_nullifier(spend_key: BaseElement, nonce: BaseElement) -> Digest {
    let inner = native_merge(
        as_digest(BaseElement::new(NULLIFIER_DOMAIN)),
        as_digest(spend_key),
    );
    native_merge(inner, as_digest(nonce))
}

pub fn native_climb(leaf: Digest, path: &MerklePath) -> Digest {
    let mut current = leaf;
    for level in 0..TREE_DEPTH {
        current = if path.is_right[level] {
            native_merge(path.siblings[level], current)
        } else {
            native_merge(current, path.siblings[level])
        };
    }
    current
}

/// Testigos del emisor. Incluye la CLAVE DE GASTO.
#[derive(Clone, Debug)]
pub struct SenderWitness {
    /// ⚠️ **CUATRO elementos** desde §90. Ver `COL_S_KEY`.
    pub spend_key: Digest,
    pub balance: u64,
    pub nonce: BaseElement,
    pub path: MerklePath,
}

/// Testigos del receptor. **Sin clave**: recibir no requiere autorización.
#[derive(Clone, Debug)]
pub struct ReceiverWitness {
    pub public_id: Digest,
    pub balance: u64,
    pub nonce: BaseElement,
    pub path: MerklePath,
}

fn value_to_bits_be(value: u64) -> Vec<bool> {
    (0..SEGMENT_LENGTH)
        .map(|p| (value >> (SEGMENT_LENGTH - 1 - p)) & 1 == 1)
        .collect()
}

/// Construye la traza.
///
/// `credited` permite acreditar al receptor una cantidad distinta de la
/// debitada, para los tests que rompen la conservación.
pub fn build_trace(
    sender: &SenderWitness,
    receiver: &ReceiverWitness,
    amount: u64,
    credited: u64,
    limit: u64,
    null_path: &MerklePath,
    frozen_path: &MerklePath,
) -> TraceTable<BaseElement> {
    let s_id = derive_public_id_wide(sender.spend_key);
    build_trace_with_id(
        sender,
        receiver,
        amount,
        credited,
        limit,
        s_id,
        null_path,
        [BaseElement::ZERO; 4],
        frozen_path,
    )
}

/// Constructor con la identidad del emisor EXPLÍCITA y la hoja inicial
/// del árbol de nullifiers también explícita.
///
/// `null_leaf_a` es la hoja con la que arranca el carril A en el árbol de
/// nullifiers. En uso normal es **cero** (la posición estaba libre); los
/// tests la usan para construir el ataque de doble gasto: afirmar que la
/// posición ya estaba ocupada por el propio nullifier.
///
/// Existe para poder construir el ataque real en los tests: usar la
/// identidad de una víctima —para que la raíz antigua cuadre— con una
/// clave distinta. Sin esto, cambiar la clave cambiaría también la
/// identidad, la hoja y la raíz, y el test lo detectaría por la aserción
/// de la raíz en vez de por la restricción de autoridad. Sería confianza
/// falsa: no sabríamos si `C_PK_CHECK` hace su trabajo.
#[allow(clippy::too_many_arguments)]
pub fn build_trace_with_id(
    sender: &SenderWitness,
    receiver: &ReceiverWitness,
    amount: u64,
    credited: u64,
    limit: u64,
    s_id: Digest,
    null_path: &MerklePath,
    null_leaf_a: Digest,
    frozen_path: &MerklePath,
) -> TraceTable<BaseElement> {
    let zero = BaseElement::ZERO;
    let c_amt = BaseElement::new(amount);
    let c_lim = BaseElement::new(limit);
    let s_bal = BaseElement::new(sender.balance);
    let s_bal_new = s_bal - c_amt;
    let r_bal = BaseElement::new(receiver.balance);
    let r_bal_new = r_bal + BaseElement::new(credited);
    let s_nonce_new = sender.nonce + BaseElement::ONE;

    let mut rows: Vec<Vec<BaseElement>> = vec![vec![zero; TRACE_WIDTH]; TRACE_LENGTH];

    // --- Transporte constante ---
    for row in rows.iter_mut() {
        row[COL_S_KEY..COL_S_KEY + 4].copy_from_slice(&sender.spend_key);
        for i in 0..4 {
            row[COL_S_ID + i] = s_id[i];
            row[COL_R_ID + i] = receiver.public_id[i];
        }
        row[COL_S_BAL] = s_bal;
        row[COL_S_NONCE] = sender.nonce;
        row[COL_S_BAL_NEW] = s_bal_new;
        row[COL_R_BAL] = r_bal;
        row[COL_R_NONCE] = receiver.nonce;
        row[COL_R_BAL_NEW] = r_bal_new;
        row[COL_AMT] = c_amt;
        row[COL_LIM] = c_lim;
    }

    // --- Carril de solvencia ---
    let segment_values = [
        s_bal.as_int(),
        r_bal.as_int(),
        c_amt.as_int(),
        c_lim.as_int(),
        s_bal_new.as_int(),
        (c_lim - c_amt).as_int(),
        r_bal_new.as_int(),
    ];
    for (seg, value) in segment_values.iter().enumerate() {
        let bits = value_to_bits_be(*value);
        let mut acc = zero;
        for p in 0..SEGMENT_LENGTH {
            let r = seg * SEGMENT_LENGTH + p;
            let bit = if bits[p] { BaseElement::ONE } else { zero };
            acc = if p == 0 { bit } else { acc + acc + bit };
            rows[r][COL_SBIT] = bit;
            rows[r][COL_SACC] = acc;
        }
    }

    // --- Carriles de hash ---
    let mut state_a = [zero; STATE_WIDTH];
    let mut state_b = [zero; STATE_WIDTH];
    // Hoja del emisor: Rescue(pk, as_digest(saldo)).
    state_a[4..8].copy_from_slice(&s_id);
    state_a[8] = s_bal;
    state_b[4..8].copy_from_slice(&s_id);
    state_b[8] = s_bal_new;

    let place = |state: &mut [BaseElement; STATE_WIDTH],
                 digest: &Digest,
                 path: &MerklePath,
                 level: usize| {
        if path.is_right[level] {
            state[4..8].copy_from_slice(&path.siblings[level]);
            state[8..12].copy_from_slice(digest);
        } else {
            state[4..8].copy_from_slice(digest);
            state[8..12].copy_from_slice(&path.siblings[level]);
        }
    };

    rows[0][..STATE_WIDTH].copy_from_slice(&state_a);
    rows[0][LANE_B..LANE_B + STATE_WIDTH].copy_from_slice(&state_b);

    let mut root_mid: Digest = [zero; 4];

    for r in 0..ROW_FROZEN_ROOT {
        let pos = r % CYCLE_LENGTH;
        if pos < NUM_ROUNDS {
            Rp64_256::apply_round(&mut state_a, pos);
            Rp64_256::apply_round(&mut state_b, pos);
        } else {
            let digest_a: Digest = [state_a[4], state_a[5], state_a[6], state_a[7]];
            let digest_b: Digest = [state_b[4], state_b[5], state_b[6], state_b[7]];
            state_a = [zero; STATE_WIDTH];
            state_b = [zero; STATE_WIDTH];

            match r {
                ROW_S_LEAF_LINK => {
                    state_a[4..8].copy_from_slice(&digest_a);
                    state_a[8] = sender.nonce;
                    state_b[4..8].copy_from_slice(&digest_b);
                    state_b[8] = s_nonce_new;
                }
                ROW_S_LEAF_DONE => {
                    place(&mut state_a, &digest_a, &sender.path, 0);
                    place(&mut state_b, &digest_b, &sender.path, 0);
                }
                ROW_S_ROOT => {
                    root_mid = digest_b;
                    state_a[4..8].copy_from_slice(&receiver.public_id);
                    state_a[8] = r_bal;
                    state_b[4..8].copy_from_slice(&receiver.public_id);
                    state_b[8] = r_bal_new;
                }
                ROW_R_LEAF_LINK => {
                    state_a[4..8].copy_from_slice(&digest_a);
                    state_a[8] = receiver.nonce;
                    state_b[4..8].copy_from_slice(&digest_b);
                    state_b[8] = receiver.nonce;
                }
                ROW_R_LEAF_DONE => {
                    place(&mut state_a, &digest_a, &receiver.path, 0);
                    place(&mut state_b, &digest_b, &receiver.path, 0);
                }
                ROW_R_ROOT => {
                    // Arranque de la derivacion de pk: AUTORIDAD.
                    state_a[4] = BaseElement::new(SPEND_KEY_DOMAIN);
                    state_a[8..12].copy_from_slice(&sender.spend_key);
                    state_b[4] = BaseElement::new(SPEND_KEY_DOMAIN);
                    state_b[8..12].copy_from_slice(&sender.spend_key);
                }
                ROW_PK_DONE => {
                    // Arranque del nullifier: DESDE LA CLAVE.
                    state_a[4] = BaseElement::new(NULLIFIER_DOMAIN);
                    state_a[8..12].copy_from_slice(&sender.spend_key);
                    state_b[4] = BaseElement::new(NULLIFIER_DOMAIN);
                    state_b[8..12].copy_from_slice(&sender.spend_key);
                }
                ROW_NULL_LINK => {
                    state_a[4..8].copy_from_slice(&digest_a);
                    state_a[8] = sender.nonce;
                    state_b[4..8].copy_from_slice(&digest_b);
                    state_b[8] = sender.nonce;
                }
                ROW_NULL_ROOT => {
                    // ===== ENTRADA AL ARBOL DE CONGELADOS =====
                    //
                    // Se coloca una hoja CERO en la posicion del emisor.
                    // Si el emisor estuviera congelado, su hoja no seria
                    // cero y la subida no llegaria a la raiz declarada.
                    //
                    // Es la misma tecnica de no-pertenencia que usa el
                    // arbol de nullifiers para el doble gasto.
                    let libre: Digest = [zero; 4];
                    place(&mut state_a, &libre, frozen_path, 0);
                    place(&mut state_b, &libre, frozen_path, 0);
                }
                ROW_NULLIFIER => {
                    // ENTRADA AL ARBOL DE NULLIFIERS.
                    // Carril A: hoja CERO (la posicion estaba libre).
                    // Carril B: el nullifier recien calculado.
                    place(&mut state_a, &null_leaf_a, null_path, 0);
                    place(&mut state_b, &digest_b, null_path, 0);
                }
                _ => {
                    let next_cycle = (r + 1) / CYCLE_LENGTH;
                    if (2..34).contains(&next_cycle) {
                        place(&mut state_a, &digest_a, &sender.path, next_cycle - 2);
                        place(&mut state_b, &digest_b, &sender.path, next_cycle - 2);
                    } else if (36..68).contains(&next_cycle) {
                        place(&mut state_a, &digest_a, &receiver.path, next_cycle - 36);
                        place(&mut state_b, &digest_b, &receiver.path, next_cycle - 36);
                    } else if (104..127).contains(&next_cycle) {
                        // Arbol de CONGELADOS: ambos carriles suben por el
                        // mismo camino desde la misma hoja cero.
                        let level = next_cycle - 103;
                        place(&mut state_a, &digest_a, frozen_path, level);
                        place(&mut state_b, &digest_b, frozen_path, level);
                    } else if (72..103).contains(&next_cycle) {
                        // Arbol de nullifiers: ambos carriles suben por el
                        // MISMO camino, con el hermano compartido impuesto
                        // por restriccion.
                        let level = next_cycle - 71;
                        place(&mut state_a, &digest_a, null_path, level);
                        place(&mut state_b, &digest_b, null_path, level);
                    }
                }
            }
        }
        rows[r + 1][..STATE_WIDTH].copy_from_slice(&state_a);
        rows[r + 1][LANE_B..LANE_B + STATE_WIDTH].copy_from_slice(&state_b);
    }

    for row in rows.iter_mut() {
        for i in 0..4 {
            row[COL_ROOT_MID + i] = root_mid[i];
        }
    }

    // Bits del camino de CONGELADOS: ciclos 103..126.
    for level in 0..FROZEN_DEPTH {
        let bit = if frozen_path.is_right[level] {
            BaseElement::ONE
        } else {
            zero
        };
        for p in 0..CYCLE_LENGTH {
            rows[(103 + level) * CYCLE_LENGTH + p][COL_FBIT] = bit;
        }
    }

    for level in 0..TREE_DEPTH {
        let s_bit = if sender.path.is_right[level] {
            BaseElement::ONE
        } else {
            zero
        };
        let r_bit = if receiver.path.is_right[level] {
            BaseElement::ONE
        } else {
            zero
        };
        let n_bit = if null_path.is_right[level] {
            BaseElement::ONE
        } else {
            zero
        };
        for p in 0..CYCLE_LENGTH {
            rows[(2 + level) * CYCLE_LENGTH + p][COL_BIT] = s_bit;
            rows[(36 + level) * CYCLE_LENGTH + p][COL_BIT] = r_bit;
            rows[(71 + level) * CYCLE_LENGTH + p][COL_BIT] = n_bit;
        }
    }

    let mut trace = TraceTable::new(TRACE_WIDTH, TRACE_LENGTH);
    trace.fill(
        |state| state.copy_from_slice(&rows[0]),
        |step, state| state.copy_from_slice(&rows[step + 1]),
    );
    trace
}

#[derive(Clone, Debug)]
pub struct SettlementPublicInputs {
    pub root_old: Digest,
    pub root_new: Digest,
    pub regulatory_limit: BaseElement,
    pub nullifier: Digest,
    /// **Raíz del árbol de congelados.** La prueba acredita que el emisor
    /// NO está en él: cualquiera que verifique la liquidación lo
    /// comprueba, sin confiar en el operador.
    pub frozen_root: Digest,
    /// Raíz del árbol de nullifiers ANTES de esta operación.
    pub nullifier_root_old: Digest,
    /// Raíz DESPUÉS de insertar el nullifier.
    pub nullifier_root_new: Digest,
}

impl ToElements<BaseElement> for SettlementPublicInputs {
    fn to_elements(&self) -> Vec<BaseElement> {
        let mut out = self.root_old.to_vec();
        out.extend_from_slice(&self.root_new);
        out.push(self.regulatory_limit);
        out.extend_from_slice(&self.nullifier);
        out.extend_from_slice(&self.frozen_root);
        out.extend_from_slice(&self.nullifier_root_old);
        out.extend_from_slice(&self.nullifier_root_new);
        out
    }
}

pub struct SettlementAir {
    context: AirContext<BaseElement>,
    pub_inputs: SettlementPublicInputs,
}

impl Air for SettlementAir {
    type BaseField = BaseElement;
    type PublicInputs = SettlementPublicInputs;

    fn new(trace_info: TraceInfo, pub_inputs: Self::PublicInputs, options: ProofOptions) -> Self {
        assert_eq!(TRACE_WIDTH, trace_info.width());
        let full = vec![TRACE_LENGTH];

        let mut degrees = Vec::with_capacity(NUM_CONSTRAINTS);
        for _ in 0..2 * STATE_WIDTH {
            degrees.push(TransitionConstraintDegree::with_cycles(7, full.clone()));
        }
        // Capacidad de arbol (8): grado 1.
        for _ in 0..8 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }
        // Colocacion (8) + hermano (4): grado 2.
        for _ in 0..12 {
            degrees.push(TransitionConstraintDegree::with_cycles(2, full.clone()));
        }
        degrees.push(TransitionConstraintDegree::new(2)); // bit booleano
        // Enlaces de hoja (16), nonces (6), entradas emisor (10),
        // receptor (10), root_mid (8): grado 1 con ciclo.
        for _ in 0..(16 + 6 + 10 + 10 + 8) {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }
        // Conservacion (2), transporte (9), ids (8), root_mid const (4):
        // grado 1 SIN ciclo.
        for _ in 0..(2 + 12 + 8 + 4) {
            degrees.push(TransitionConstraintDegree::new(1));
        }
        // Clave en nullifier (**8**), entrada de pk (**8**), pk (4).
        //
        // ⚠️ Eran 2 + 2 + 4. La clave entra DOS veces —para derivar `pk` y
        // para el nullifier— y cada una son 4 elementos por 2 carriles.
        for _ in 0..20 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }
        // No-pertenencia (4): GRADO 2, no 1. Su expresion selecciona la
        // mitad del estado con el bit —`(1-bit)*x + bit*y`— y eso es un
        // producto de dos columnas de traza. Winterfell exige exactitud en
        // los grados, no cotas superiores.
        for _ in 0..4 {
            degrees.push(TransitionConstraintDegree::with_cycles(2, full.clone()));
        }
        // Solvencia: bits booleanos (2) grado 2.
        for _ in 0..2 {
            degrees.push(TransitionConstraintDegree::new(2));
        }
        for _ in 0..(3 + NUM_SEGMENTS) {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }

        // --- Fase de congelados ---
        // Capacidad (4): grado 1 con ciclo.
        for _ in 0..4 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }
        // No-pertenencia (4) y colocacion (4): grado 2 con ciclo, porque
        // multiplican por el bit de direccion.
        for _ in 0..8 {
            degrees.push(TransitionConstraintDegree::with_cycles(2, full.clone()));
        }
        // Bit booleano (1): grado 2 sin ciclo.
        degrees.push(TransitionConstraintDegree::new(2));

        assert_eq!(degrees.len(), NUM_CONSTRAINTS, "cuenta de grados");

        SettlementAir {
            // 57 - 6: se retiraron las que fijaban a cero las ranuras
            // 9..12, que dejaron de ser relleno al ensanchar la clave (§90).
            context: AirContext::new(trace_info, degrees, 51, options),
            pub_inputs,
        }
    }

    fn context(&self) -> &AirContext<Self::BaseField> {
        &self.context
    }

    fn get_periodic_column_values(&self) -> Vec<Vec<Self::BaseField>> {
        let zero = BaseElement::ZERO;
        let one = BaseElement::ONE;
        let mut columns = Vec::new();

        let mut hash_flag = vec![zero; TRACE_LENGTH];
        for r in 0..=ROW_NULL_ROOT {
            if r % CYCLE_LENGTH < NUM_ROUNDS {
                hash_flag[r] = one;
            }
        }
        columns.push(hash_flag);

        for ark in [true, false] {
            for i in 0..STATE_WIDTH {
                let mut col = vec![zero; TRACE_LENGTH];
                for r in 0..=ROW_NULL_ROOT {
                    let pos = r % CYCLE_LENGTH;
                    if pos < NUM_ROUNDS {
                        col[r] = if ark {
                            Rp64_256::ARK1[pos][i]
                        } else {
                            Rp64_256::ARK2[pos][i]
                        };
                    }
                }
                columns.push(col);
            }
        }

        let mut link_merkle = vec![zero; TRACE_LENGTH];
        for level in 0..TREE_DEPTH - 1 {
            link_merkle[(2 + level) * CYCLE_LENGTH + 7] = one;
            link_merkle[(36 + level) * CYCLE_LENGTH + 7] = one;
            link_merkle[(71 + level) * CYCLE_LENGTH + 7] = one;
        }
        columns.push(link_merkle);

        let mut link_leaf = vec![zero; TRACE_LENGTH];
        link_leaf[ROW_S_LEAF_LINK] = one;
        link_leaf[ROW_R_LEAF_LINK] = one;
        link_leaf[ROW_NULL_LINK] = one;
        columns.push(link_leaf);

        let mut link_place = vec![zero; TRACE_LENGTH];
        link_place[ROW_S_LEAF_DONE] = one;
        link_place[ROW_R_LEAF_DONE] = one;
        columns.push(link_place);

        for row in [ROW_S_LEAF_LINK, ROW_R_LEAF_LINK, ROW_NULL_LINK] {
            let mut sel = vec![zero; TRACE_LENGTH];
            sel[row] = one;
            columns.push(sel);
        }

        // ROW_R_ROOT aparece DOS veces a proposito: en esa fila
        // conviven la comprobacion de root_mid (sobre `current`) y el
        // arranque de la derivacion de pk (sobre `next`).
        for row in [0, ROW_S_ROOT, ROW_R_ROOT, ROW_R_ROOT, ROW_PK_DONE, ROW_NULLIFIER] {
            let mut sel = vec![zero; TRACE_LENGTH];
            sel[row] = one;
            columns.push(sel);
        }

        let mut first_s = vec![zero; TRACE_LENGTH];
        let mut cont_s = vec![zero; TRACE_LENGTH];
        for seg in 0..NUM_SEGMENTS {
            first_s[seg * SEGMENT_LENGTH] = one;
            for p in 0..SEGMENT_LENGTH - 1 {
                cont_s[seg * SEGMENT_LENGTH + p] = one;
            }
        }
        columns.push(first_s);
        columns.push(cont_s);

        for seg in 0..NUM_SEGMENTS {
            let mut link = vec![zero; TRACE_LENGTH];
            link[(seg + 1) * SEGMENT_LENGTH - 2] = one;
            columns.push(link);
        }


        // Entrada al arbol de congelados: una sola fila.
        let mut frozen_entry = vec![zero; TRACE_LENGTH];
        frozen_entry[ROW_NULL_ROOT] = one;
        columns.push(frozen_entry);

        // Enlaces de la subida: 23, uno por nivel a partir del primero.
        let mut frozen_link = vec![zero; TRACE_LENGTH];
        for level in 0..FROZEN_DEPTH - 1 {
            frozen_link[(103 + level) * CYCLE_LENGTH + 7] = one;
        }
        columns.push(frozen_link);
        columns
    }

    fn evaluate_transition<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        frame: &EvaluationFrame<E>,
        periodic: &[E],
        result: &mut [E],
    ) {
        let current = frame.current();
        let next = frame.next();

        let hash_flag = periodic[P_HASH_FLAG];
        let ark1 = &periodic[P_ARK1..P_ARK1 + STATE_WIDTH];
        let ark2 = &periodic[P_ARK2..P_ARK2 + STATE_WIDTH];
        let link_merkle = periodic[P_LINK_MERKLE];
        let link_leaf = periodic[P_LINK_LEAF];
        let link_place = periodic[P_LINK_PLACE];
        let sel_s_leaf = periodic[P_SEL_S_LEAF];
        let sel_r_leaf = periodic[P_SEL_R_LEAF];
        let sel_null_leaf = periodic[P_SEL_NULL_LEAF];
        let first_row = periodic[P_FIRST_ROW];
        let sel_s_root = periodic[P_SEL_S_ROOT];
        let sel_r_root = periodic[P_SEL_R_ROOT];
        let sel_pk_start = periodic[P_SEL_PK_START];
        let sel_pk_done = periodic[P_SEL_PK_DONE];
        let null_place = periodic[P_NULL_PLACE];
        let first_s = periodic[P_FIRST_S];
        let cont_s = periodic[P_CONT_S];

        // ===== Rondas de Rescue =====
        for (lane, offset) in [(0usize, 0usize), (1usize, LANE_B)] {
            let mut a = [E::ZERO; STATE_WIDTH];
            for i in 0..STATE_WIDTH {
                let mut acc = E::ZERO;
                for j in 0..STATE_WIDTH {
                    acc += E::from(Rp64_256::MDS[i][j]) * apply_sbox(current[offset + j]);
                }
                a[i] = acc + ark1[i];
            }
            let mut b = [E::ZERO; STATE_WIDTH];
            for i in 0..STATE_WIDTH {
                let mut acc = E::ZERO;
                for j in 0..STATE_WIDTH {
                    acc += E::from(Rp64_256::INV_MDS[i][j]) * (next[offset + j] - ark2[j]);
                }
                b[i] = acc;
            }
            for i in 0..STATE_WIDTH {
                result[C_HASH_A + lane * STATE_WIDTH + i] = hash_flag * (apply_sbox(b[i]) - a[i]);
            }
        }

        let bit = next[COL_BIT];
        // Enlaces de arbol de CUENTAS: ambos carriles colocan su digest.
        let tree_link = link_merkle + link_place;
        // Enlaces donde se coloca ALGO en un arbol, incluida la entrada al
        // de nullifiers. Ahi el carril A NO coloca su digest sino CERO,
        // asi que su restriccion de colocacion usa `tree_link` a secas.
        let any_place = tree_link + null_place;

        for i in 0..4 {
            result[C_TREE_CAP_A + i] = any_place * next[i];
            result[C_TREE_CAP_B + i] = any_place * next[LANE_B + i];

            let da = current[4 + i];
            let placed_a = (E::ONE - bit) * (next[4 + i] - da) + bit * (next[8 + i] - da);
            result[C_PLACE_A + i] = tree_link * placed_a;

            let db = current[LANE_B + 4 + i];
            let placed_b =
                (E::ONE - bit) * (next[LANE_B + 4 + i] - db) + bit * (next[LANE_B + 8 + i] - db);
            result[C_PLACE_B + i] = any_place * placed_b;

            let sib_a = (E::ONE - bit) * next[8 + i] + bit * next[4 + i];
            let sib_b = (E::ONE - bit) * next[LANE_B + 8 + i] + bit * next[LANE_B + 4 + i];
            result[C_SIBLING + i] = any_place * (sib_a - sib_b);

            // NO-PERTENENCIA: al entrar en el arbol de nullifiers, el
            // carril A coloca CERO. Es lo que demuestra que la posicion
            // estaba libre.
            let leaf_a = (E::ONE - bit) * next[4 + i] + bit * next[8 + i];
            result[C_NULL_EMPTY + i] = null_place * leaf_a;
        }

        result[C_BIT_BOOL] = current[COL_BIT] * (current[COL_BIT] - E::ONE);

        for i in 0..4 {
            result[C_LEAF_CAP_A + i] = link_leaf * next[i];
            result[C_LEAF_CAP_B + i] = link_leaf * next[LANE_B + i];
            result[C_LEAF_DIG_A + i] = link_leaf * (next[4 + i] - current[4 + i]);
            result[C_LEAF_DIG_B + i] =
                link_leaf * (next[LANE_B + 4 + i] - current[LANE_B + 4 + i]);
        }

        // ===== Nonces =====
        result[C_NONCE] = sel_s_leaf * (next[8] - current[COL_S_NONCE]);
        result[C_NONCE + 1] = sel_s_leaf * (next[LANE_B + 8] - (current[COL_S_NONCE] + E::ONE));
        result[C_NONCE + 2] = sel_r_leaf * (next[8] - current[COL_R_NONCE]);
        result[C_NONCE + 3] = sel_r_leaf * (next[LANE_B + 8] - current[COL_R_NONCE]);
        result[C_NONCE + 4] = sel_null_leaf * (next[8] - current[COL_S_NONCE]);
        result[C_NONCE + 5] = sel_null_leaf * (next[LANE_B + 8] - current[COL_S_NONCE]);

        // ===== Entradas de la hoja del emisor: pk COMPLETA + saldo =====
        for i in 0..4 {
            result[C_S_INPUT + i] = first_row * (current[4 + i] - current[COL_S_ID + i]);
            result[C_S_INPUT + 5 + i] =
                first_row * (current[LANE_B + 4 + i] - current[COL_S_ID + i]);
        }
        result[C_S_INPUT + 4] = first_row * (current[8] - current[COL_S_BAL]);
        result[C_S_INPUT + 9] = first_row * (current[LANE_B + 8] - current[COL_S_BAL_NEW]);

        // ===== Entradas de la hoja del receptor =====
        for i in 0..4 {
            result[C_R_INPUT + i] = sel_s_root * (next[4 + i] - current[COL_R_ID + i]);
            result[C_R_INPUT + 5 + i] =
                sel_s_root * (next[LANE_B + 4 + i] - current[COL_R_ID + i]);
        }
        result[C_R_INPUT + 4] = sel_s_root * (next[8] - current[COL_R_BAL]);
        result[C_R_INPUT + 9] = sel_s_root * (next[LANE_B + 8] - current[COL_R_BAL_NEW]);

        // ===== Puente de root_mid =====
        for i in 0..4 {
            result[C_MID_CAPTURE + i] =
                sel_s_root * (current[COL_ROOT_MID + i] - current[LANE_B + 4 + i]);
            result[C_MID_CHECK + i] = sel_r_root * (current[COL_ROOT_MID + i] - current[4 + i]);
        }

        // ===== CONSERVACIÓN =====
        result[C_CONSERVATION] =
            current[COL_S_BAL_NEW] - (current[COL_S_BAL] - current[COL_AMT]);
        result[C_CONSERVATION + 1] =
            current[COL_R_BAL_NEW] - (current[COL_R_BAL] + current[COL_AMT]);

        // ===== Constancia del transporte =====
        let transport = [
            COL_S_KEY,
            COL_S_KEY + 1,
            COL_S_KEY + 2,
            COL_S_KEY + 3,
            COL_S_BAL,
            COL_S_NONCE,
            COL_S_BAL_NEW,
            COL_R_BAL,
            COL_R_NONCE,
            COL_R_BAL_NEW,
            COL_AMT,
            COL_LIM,
        ];
        for (k, col) in transport.iter().enumerate() {
            result[C_TRANSPORT + k] = next[*col] - current[*col];
        }
        for i in 0..4 {
            result[C_ID_CONST + i] = next[COL_S_ID + i] - current[COL_S_ID + i];
            result[C_ID_CONST + 4 + i] = next[COL_R_ID + i] - current[COL_R_ID + i];
            result[C_MID_CONST + i] = next[COL_ROOT_MID + i] - current[COL_ROOT_MID + i];
        }

        // ===== La CLAVE entra en el nullifier =====
        // El nullifier arranca tras la derivacion de pk.
        // ⚠️ **Los CUATRO elementos, en los dos carriles.** Atar solo el
        // primero dejaria los otros tres libres, y el compromiso dejaria de
        // estar determinado por la clave declarada — el mismo fallo que §72
        // en otro sitio.
        for i in 0..4 {
            result[C_NULL_KEY + i] = sel_pk_done * (next[8 + i] - current[COL_S_KEY + i]);
            result[C_NULL_KEY + 4 + i] =
                sel_pk_done * (next[LANE_B + 8 + i] - current[COL_S_KEY + i]);
        }

        // ===== La clave entra en la derivación de pk =====
        for i in 0..4 {
            result[C_PK_INPUT + i] = sel_pk_start * (next[8 + i] - current[COL_S_KEY + i]);
            result[C_PK_INPUT + 4 + i] =
                sel_pk_start * (next[LANE_B + 8 + i] - current[COL_S_KEY + i]);
        }

        // ===== AUTORIDAD DE GASTO =====
        // La pk derivada de la clave coincide con la identidad usada en
        // la hoja. Sin conocer `sk` es imposible satisfacerlo.
        for i in 0..4 {
            result[C_PK_CHECK + i] = sel_pk_done * (current[4 + i] - current[COL_S_ID + i]);
        }

        // ===== Solvencia =====
        let sbit_cur = current[COL_SBIT];
        let sbit_next = next[COL_SBIT];
        let sacc_cur = current[COL_SACC];
        let sacc_next = next[COL_SACC];

        result[C_SBIT_BOOL] = sbit_cur * (sbit_cur - E::ONE);
        result[C_SBIT_BOOL + 1] = sbit_next * (sbit_next - E::ONE);
        result[C_FIRST_S] = first_s * sbit_cur;
        result[C_FIRST_S + 1] = first_s * sacc_cur;
        result[C_HORNER] = cont_s * (sacc_next - (sacc_cur + sacc_cur + sbit_next));

        let expected = [
            current[COL_S_BAL],
            current[COL_R_BAL],
            current[COL_AMT],
            current[COL_LIM],
            current[COL_S_BAL_NEW],
            current[COL_LIM] - current[COL_AMT],
            current[COL_R_BAL_NEW],
        ];
        for seg in 0..NUM_SEGMENTS {
            result[C_SEG_LINK + seg] = periodic[P_SEG_LINK + seg] * (sacc_next - expected[seg]);
        }

        // ===== NO-PERTENENCIA AL ARBOL DE CONGELADOS =====
        //
        // Sin estas restricciones la raiz de congelados seria un valor
        // declarado sin atar a nada, y el circuito PARECERIA comprobar la
        // congelacion sin comprobarla.
        let frozen_entry = periodic[P_FROZEN_ENTRY];
        let frozen_link = periodic[P_FROZEN_LINK];
        let fbit = next[COL_FBIT];

        for i in 0..4 {
            // Capacidad a cero al entrar y en cada nivel.
            result[C_FROZEN_CAP + i] = (frozen_entry + frozen_link) * next[i];

            // **LA NO-PERTENENCIA**: la hoja colocada debe ser CERO.
            // Si el emisor estuviera congelado, su hoja llevaria la marca
            // y esta restriccion no se satisfaria.
            result[C_FROZEN_ENTRY + i] =
                frozen_entry * ((E::ONE - fbit) * next[4 + i] + fbit * next[8 + i]);

            // Colocacion normal en los niveles siguientes.
            let d = current[4 + i];
            result[C_FROZEN_PLACE + i] =
                frozen_link * ((E::ONE - fbit) * (next[4 + i] - d) + fbit * (next[8 + i] - d));
        }
        result[C_FBIT_BOOL] = current[COL_FBIT] * (current[COL_FBIT] - E::ONE);
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        let zero = BaseElement::ZERO;
        let mut a = Vec::with_capacity(57);

        // Fila 0: capacidad y relleno. Las posiciones 4..8 (pk) y 8
        // (saldo) se atan con C_S_INPUT.
        for i in 0..4 {
            a.push(Assertion::single(i, 0, zero));
            a.push(Assertion::single(LANE_B + i, 0, zero));
        }
        for i in 9..12 {
            a.push(Assertion::single(i, 0, zero));
            a.push(Assertion::single(LANE_B + i, 0, zero));
        }
        // Raices publicas.
        for i in 0..4 {
            a.push(Assertion::single(4 + i, ROW_S_ROOT, self.pub_inputs.root_old[i]));
            a.push(Assertion::single(
                LANE_B + 4 + i,
                ROW_R_ROOT,
                self.pub_inputs.root_new[i],
            ));
        }
        // Arranque del nullifier: dominio anclado y relleno.
        for i in 0..4 {
            a.push(Assertion::single(i, ROW_NULL_START, zero));
        }
        a.push(Assertion::single(
            4,
            ROW_NULL_START,
            BaseElement::new(NULLIFIER_DOMAIN),
        ));
        for i in 5..8 {
            a.push(Assertion::single(i, ROW_NULL_START, zero));
        }
        // ⚠️ Las ranuras 9..12 **ya no son relleno: son la clave** (§90).
        //
        // Las fijaba a cero cuando `sk` era un elemento. Ahora `state[8..12]`
        // lleva los cuatro, y quien las ata es `C_NULL_KEY` contra
        // `COL_S_KEY` —constante por `C_TRANSPORT`—. Pasan de estar fijadas
        // a CERO a estar fijadas a la CLAVE DECLARADA: mas fuerte, no mas
        // debil.
        // Nullifier publico.
        for i in 0..4 {
            a.push(Assertion::single(
                4 + i,
                ROW_NULLIFIER,
                self.pub_inputs.nullifier[i],
            ));
        }
        // Arranque de la derivacion de pk: dominio anclado.
        a.push(Assertion::single(
            4,
            ROW_PK_START,
            BaseElement::new(SPEND_KEY_DOMAIN),
        ));
        for i in 5..8 {
            a.push(Assertion::single(i, ROW_PK_START, zero));
        }
        // ⚠️ Igual que arriba: 9..12 es la clave, y la ata `C_PK_INPUT`.
        // Raices del arbol de nullifiers: la antigua (donde la posicion
        // estaba libre) y la nueva (con el nullifier insertado).
        for i in 0..4 {
            a.push(Assertion::single(
                4 + i,
                ROW_NULL_ROOT,
                self.pub_inputs.nullifier_root_old[i],
            ));
            a.push(Assertion::single(
                LANE_B + 4 + i,
                ROW_NULL_ROOT,
                self.pub_inputs.nullifier_root_new[i],
            ));
        }

        // Limite regulatorio publico.
        a.push(Assertion::single(
            COL_LIM,
            0,
            self.pub_inputs.regulatory_limit,
        ));

        // **La raíz de congelados**: el emisor no está en ese árbol.
        for i in 0..4 {
            a.push(Assertion::single(
                4 + i,
                ROW_FROZEN_ROOT,
                self.pub_inputs.frozen_root[i],
            ));
        }

        a
    }
}

pub struct SettlementProver {
    options: ProofOptions,
}

impl SettlementProver {
    pub fn new(options: ProofOptions) -> Self {
        Self { options }
    }
}

impl Prover for SettlementProver {
    type BaseField = BaseElement;
    type Air = SettlementAir;
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

    fn get_pub_inputs(&self, trace: &Self::Trace) -> SettlementPublicInputs {
        SettlementPublicInputs {
            root_old: [
                trace.get(4, ROW_S_ROOT),
                trace.get(5, ROW_S_ROOT),
                trace.get(6, ROW_S_ROOT),
                trace.get(7, ROW_S_ROOT),
            ],
            root_new: [
                trace.get(LANE_B + 4, ROW_R_ROOT),
                trace.get(LANE_B + 5, ROW_R_ROOT),
                trace.get(LANE_B + 6, ROW_R_ROOT),
                trace.get(LANE_B + 7, ROW_R_ROOT),
            ],
            regulatory_limit: trace.get(COL_LIM, 0),
            nullifier: [
                trace.get(4, ROW_NULLIFIER),
                trace.get(5, ROW_NULLIFIER),
                trace.get(6, ROW_NULLIFIER),
                trace.get(7, ROW_NULLIFIER),
            ],
            frozen_root: [
                trace.get(4, ROW_FROZEN_ROOT),
                trace.get(5, ROW_FROZEN_ROOT),
                trace.get(6, ROW_FROZEN_ROOT),
                trace.get(7, ROW_FROZEN_ROOT),
            ],
            nullifier_root_old: [
                trace.get(4, ROW_NULL_ROOT),
                trace.get(5, ROW_NULL_ROOT),
                trace.get(6, ROW_NULL_ROOT),
                trace.get(7, ROW_NULL_ROOT),
            ],
            nullifier_root_new: [
                trace.get(LANE_B + 4, ROW_NULL_ROOT),
                trace.get(LANE_B + 5, ROW_NULL_ROOT),
                trace.get(LANE_B + 6, ROW_NULL_ROOT),
                trace.get(LANE_B + 7, ROW_NULL_ROOT),
            ],
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
    use winterfell::{verify, AcceptableOptions, BatchingMethod, FieldExtension};

    /// ⚠️ **La derivacion ancha GENERALIZA la estrecha.**
    ///
    /// Es la pregunta de la que depende el plan de la entrada 15: §85 y esa
    /// entrada dicen que ensanchar la clave «invalida cualquier cuenta
    /// existente».
    ///
    /// Si rellenar con ceros da la MISMA identidad, es falso: las cuentas
    /// sobreviven y lo que hay que rotar son las claves, gradualmente.
    ///
    /// ⚠️ **Conservar la identidad no conserva la seguridad.** Una clave
    /// rellenada sigue teniendo 64 bits y sigue cayendo en los 2^63 de §82.
    /// Cambia el coste de MIGRAR, no el de ATACAR.
    #[test]
    fn the_wide_derivation_generalises_the_narrow_one() {
        for k in [1u64, 0xDEADBEEF, 0xA11CE, u64::MAX - 7] {
            let sk = BaseElement::new(k);
            assert_eq!(
                derive_public_id_wide(as_digest(sk)),
                derive_public_id(sk),
                "una clave estrecha rellenada con ceros debe dar la MISMA \
                 identidad: si no, migrar invalidaria las cuentas (k = {k:#x})"
            );
            assert_eq!(
                native_nullifier_wide(as_digest(sk), BaseElement::new(3)),
                native_nullifier(sk, BaseElement::new(3)),
                "y el mismo nullifier (k = {k:#x})"
            );
        }
    }

    /// **Y una clave ancha de verdad da otra identidad.**
    ///
    /// Sin esto, el test de arriba pasaria igual si la version ancha
    /// **ignorara** los tres elementos nuevos —que es justo el fallo que
    /// tendria si se escribiera mal—. Es el par discriminante: uno fija que
    /// generaliza, el otro que **usa** lo que se le da.
    #[test]
    fn a_wide_key_is_not_its_first_element() {
        let base = BaseElement::new(0xA11CE);
        let estrecha = as_digest(base);
        let ancha = [
            base,
            BaseElement::new(1),
            BaseElement::ZERO,
            BaseElement::ZERO,
        ];
        assert_ne!(
            derive_public_id_wide(ancha),
            derive_public_id_wide(estrecha),
            "cambiar un elemento distinto del primero DEBE cambiar la \
             identidad, o los 192 bits nuevos no valdrian nada"
        );
        assert_ne!(
            native_nullifier_wide(ancha, BaseElement::new(3)),
            native_nullifier_wide(estrecha, BaseElement::new(3)),
            "y el nullifier igual"
        );
    }

    /// **Cuanto cuesta agotar el espacio de claves de gasto.**
    ///
    /// La cabecera de este modulo documenta el problema de la IDENTIDAD y su
    /// correccion: paso a ser el digest completo de 4 elementos. Eso impide
    /// encontrar **otra** clave con la misma identidad.
    ///
    /// ⚠️ **No impide encontrar LA clave.** `sk` sigue siendo **un solo
    /// elemento de Goldilocks**, asi que el espacio de secretos es 2^64 y
    /// `pk` es publica —el pagador la necesita para direccionar—. El ataque
    /// es busqueda exhaustiva fuera de linea: enumerar `sk`, comparar `pk`.
    ///
    /// Este test **no juzga**: mide `derive_public_id`, que es exactamente
    /// la operacion que el atacante repite, y extrapola a 2^63. El numero
    /// decide si eso es un limite declarable o un fallo que hay que
    /// corregir.
    ///
    /// Se salta siempre: es un instrumento, no una comprobacion. Correr con
    /// `--ignored --nocapture`, y **en release**, o se mide el compilador.
    #[test]
    #[ignore = "instrumento de medida, no comprobacion: correr a mano"]
    fn el_coste_de_agotar_el_espacio_de_claves() {
        use std::time::Instant;

        // Calentamiento, para no medir la primera carga de las tablas.
        let mut sumidero = BaseElement::ZERO;
        for k in 0..10_000u64 {
            sumidero += derive_public_id(BaseElement::new(k))[0];
        }

        const N: u64 = 2_000_000;
        let t0 = Instant::now();
        for k in 0..N {
            sumidero += derive_public_id(BaseElement::new(k))[0];
        }
        let dt = t0.elapsed().as_secs_f64();
        // El sumidero existe para que el optimizador no borre el bucle.
        assert_ne!(sumidero, BaseElement::new(u64::MAX), "sumidero");

        let por_seg = N as f64 / dt;
        // 2^63 es el coste ESPERADO: de media se encuentra a mitad del
        // espacio. 2^64 seria el peor caso.
        let esperado = 2f64.powi(63);
        let seg_1_nucleo = esperado / por_seg;
        let anios_1_nucleo = seg_1_nucleo / (365.25 * 24.0 * 3600.0);

        println!("\n=== Coste de agotar el espacio de claves de gasto ===\n");
        println!("  Este nucleo, este binario, sin optimizar el ataque:");
        println!("    derive_public_id/s   {por_seg:>18.0}");
        println!("    N medido             {N:>18}");
        println!("    tiempo               {dt:>18.3} s");
        println!();
        println!("  Extrapolacion a 2^63 = {esperado:.3e} evaluaciones:");
        println!("    anios-nucleo         {anios_1_nucleo:>18.1}");
        for nucleos in [1_000f64, 100_000.0, 10_000_000.0] {
            let anios = anios_1_nucleo / nucleos;
            if anios >= 1.0 {
                println!("    con {nucleos:>12.0} nucleos  {anios:>10.1} anios");
            } else {
                println!("    con {nucleos:>12.0} nucleos  {:>10.1} dias", anios * 365.25);
            }
        }
        println!();
        println!("  ⚠️ Es una COTA SUPERIOR floja del coste real del ataque:");
        println!("     un atacante usaria GPU o ASIC, evitaria la asignacion");
        println!("     de memoria por llamada y compararia solo un elemento");
        println!("     del digest antes de descartar. El numero de arriba es");
        println!("     lo que cuesta HOY con este codigo, no lo que costaria");
        println!("     a quien lo intente en serio.");
        println!();
        println!("  ⚠️ Y no depende de la anchura de la IDENTIDAD (256 bits):");
        println!("     depende de la del SECRETO, que es un elemento = 64 bits.");
    }

    fn default_options() -> ProofOptions {
        ProofOptions::new(
            32,
            8,
            0,
            FieldExtension::None,
            8,
            31,
            BatchingMethod::Linear,
            BatchingMethod::Linear,
        )
    }

    fn empty_subtrees() -> Vec<Digest> {
        let mut empty = vec![[BaseElement::ZERO; 4]];
        for k in 1..=TREE_DEPTH {
            let prev = empty[k - 1];
            empty.push(native_merge(prev, prev));
        }
        empty
    }

    struct Scenario {
        sender: SenderWitness,
        receiver: ReceiverWitness,
        amount: u64,
        credited: u64,
        limit: u64,
        null_path: MerklePath,
        frozen_path: MerklePath,
        public_inputs: SettlementPublicInputs,
    }

    /// Escenario con árbol disperso: emisor en el índice 0, receptor en
    /// el 1 (hermanos en el nivel 0). El camino del receptor incluye la
    /// hoja NUEVA del emisor, que es lo que exige el encadenamiento.
    fn scenario(sender_balance: u64, amount: u64, credited: u64, limit: u64) -> Scenario {
        let empty = empty_subtrees();
        // ⚠️ Clave ANCHA de verdad: si fuera `as_digest(x)` los tres
        // elementos nuevos irian a cero y el circuito pasaria sin
        // ejercitarlos (§90.3).
        let s_key = [
            BaseElement::new(0xDEADBEEF),
            BaseElement::new(0xFEEDFACE),
            BaseElement::new(0x00C0FFEE),
            BaseElement::new(0x05EA51DE),
        ];
        let s_nonce = BaseElement::new(7);
        let s_id = derive_public_id_wide(s_key);
        let r_id = derive_public_id(BaseElement::new(0xCAFE));
        let r_bal = 50_000u64;
        let r_nonce = BaseElement::new(3);

        let s_leaf_old = native_leaf(s_id, BaseElement::new(sender_balance), s_nonce);
        let s_leaf_new = native_leaf(
            s_id,
            BaseElement::new(sender_balance) - BaseElement::new(amount),
            s_nonce + BaseElement::ONE,
        );
        let r_leaf_old = native_leaf(r_id, BaseElement::new(r_bal), r_nonce);
        let r_leaf_new = native_leaf(
            r_id,
            BaseElement::new(r_bal) + BaseElement::new(credited),
            r_nonce,
        );

        let mut s_sib = vec![r_leaf_old];
        let mut s_right = vec![false];
        let mut r_sib = vec![s_leaf_new];
        let mut r_right = vec![true];
        for level in 1..TREE_DEPTH {
            s_sib.push(empty[level]);
            s_right.push(false);
            r_sib.push(empty[level]);
            r_right.push(false);
        }
        let sender_path = MerklePath {
            siblings: s_sib,
            is_right: s_right,
        };
        let receiver_path = MerklePath {
            siblings: r_sib,
            is_right: r_right,
        };

        // Arbol de nullifiers: parte vacio, se inserta el de esta
        // operacion.
        let nullifier = native_nullifier_wide(s_key, s_nonce);
        let null_path =
            crate::nullifier_tree::path_for_empty_tree(
                crate::nullifier_tree::nullifier_position(&nullifier),
            );

        // Camino del arbol de congelados. Direcciones MIXTAS: con todas
        // iguales la traza degenera y winterfell rechaza los grados.
        let mut f_empty = vec![[BaseElement::ZERO; 4]];
        for k in 1..=FROZEN_DEPTH {
            let prev = f_empty[k - 1];
            f_empty.push(native_merge(prev, prev));
        }
        let frozen_path = MerklePath {
            siblings: (0..FROZEN_DEPTH).map(|l| f_empty[l]).collect(),
            is_right: (0..FROZEN_DEPTH).map(|l| l % 3 == 0).collect(),
        };
        let frozen_root = crate::circuit_freeze::frozen_climb(
            [BaseElement::ZERO; 4],
            &frozen_path,
        );

        Scenario {
            public_inputs: SettlementPublicInputs {
                root_old: native_climb(s_leaf_old, &sender_path),
                root_new: native_climb(r_leaf_new, &receiver_path),
                regulatory_limit: BaseElement::new(limit),
                nullifier,
                frozen_root,
                nullifier_root_old: crate::nullifier_tree::empty_root(),
                nullifier_root_new: crate::nullifier_tree::climb(nullifier, &null_path),
            },
            null_path,
            frozen_path,
            sender: SenderWitness {
                spend_key: s_key,
                balance: sender_balance,
                nonce: s_nonce,
                path: sender_path,
            },
            receiver: ReceiverWitness {
                public_id: r_id,
                balance: r_bal,
                nonce: r_nonce,
                path: receiver_path,
            },
            amount,
            credited,
            limit,
        }
    }

    fn run(s: &Scenario, declared: SettlementPublicInputs) -> Result<(), String> {
        let trace = build_trace(
            &s.sender,
            &s.receiver,
            s.amount,
            s.credited,
            s.limit,
            &s.null_path,
            &s.frozen_path,
        );
        let prover = SettlementProver::new(default_options());

        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| prover.prove(trace)));

        let proof = match r {
            Err(_) => return Err("prove hizo panic (traza invalida en debug)".into()),
            Ok(Err(e)) => return Err(format!("prove devolvio Err: {e:?}")),
            Ok(Ok(p)) => p,
        };

        let min_opts = AcceptableOptions::OptionSet(vec![default_options()]);
        verify::<SettlementAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof, declared, &min_opts,
        )
        .map_err(|e| format!("verificacion fallo: {e:?}"))
    }

    /// La identidad es un DIGEST de 4 elementos, no uno: 256 bits en vez
    /// de 64. Con 64 bits, encontrar otra clave con la misma identidad
    /// costaría 2^32 operaciones.
    #[test]
    fn public_id_is_a_full_digest() {
        let id = derive_public_id(BaseElement::new(12345));
        assert_eq!(id.len(), 4);
        let nonzero = id.iter().filter(|e| **e != BaseElement::ZERO).count();
        assert!(nonzero >= 3, "el digest deberia usar sus 4 elementos");
    }

    /// Claves distintas dan identidades distintas.
    #[test]
    fn distinct_keys_give_distinct_ids() {
        assert_ne!(
            derive_public_id(BaseElement::new(1)),
            derive_public_id(BaseElement::new(2))
        );
    }

    /// El nullifier NO es derivable de la identidad pública.
    #[test]
    fn nullifier_is_not_derivable_from_public_id() {
        let sk = BaseElement::new(31337);
        let nonce = BaseElement::new(1);
        let id = derive_public_id(sk);
        let real = native_nullifier(sk, nonce);
        let guess = {
            let inner = native_merge(as_digest(BaseElement::new(NULLIFIER_DOMAIN)), id);
            native_merge(inner, as_digest(nonce))
        };
        assert_ne!(real, guess);
    }

    /// La traza contiene las raíces y el nullifier esperados.
    #[test]
    fn trace_landmarks_match_native() {
        let s = scenario(1_000_000, 250_000, 250_000, 500_000);
        let trace = build_trace(
            &s.sender,
            &s.receiver,
            s.amount,
            s.credited,
            s.limit,
            &s.null_path,
            &s.frozen_path,
        );
        for i in 0..4 {
            assert_eq!(
                trace.get(4 + i, ROW_S_ROOT),
                s.public_inputs.root_old[i],
                "root_old {i}"
            );
            assert_eq!(
                trace.get(LANE_B + 4 + i, ROW_R_ROOT),
                s.public_inputs.root_new[i],
                "root_new {i}"
            );
            assert_eq!(
                trace.get(4 + i, ROW_NULLIFIER),
                s.public_inputs.nullifier[i],
                "nullifier {i}"
            );
            // La pk derivada al final coincide con la identidad usada.
            assert_eq!(
                trace.get(4 + i, ROW_PK_DONE),
                derive_public_id_wide(s.sender.spend_key)[i],
                "pk derivada {i}"
            );
        }
        // ===== Y TODAS LAS ENTRADAS PÚBLICAS, NO SOLO LAS RAÍCES =====
        //
        // Comparar la estructura entera. En `circuit_send` la versión
        // parcial dejó pasar un campo heredado de otra operación y **costó
        // ocho rondas de diagnóstico**: probador y verificador usaban
        // transcripciones de Fiat-Shamir distintas, y el error de winterfell
        // —`InconsistentOodConstraintEvaluations`— apunta a las
        // restricciones, no a las entradas.
        let derivadas = SettlementProver::new(default_options()).get_pub_inputs(&trace);
        assert_eq!(
            derivadas.to_elements(),
            s.public_inputs.to_elements(),
            "las entradas DERIVADAS de la traza deben coincidir con las \
             DECLARADAS en todos sus campos"
        );

    }

    /// EL TEST CLAVE.
    ///
    /// A diferencia de los negativos, este NO silencia el pánico: si la
    /// traza válida no satisface alguna restricción, queremos ver cuál y
    /// en qué fila, no un mensaje genérico.
    /// **Que cuestan +3 columnas y +15 restricciones.**
    ///
    /// §85.2 conto que ensanchar `sk` a cuatro elementos cuesta eso por
    /// circuito. Lo que el conteo NO dice es cuanto vale en **tamaño de
    /// prueba y tiempo**, y de ahi depende si el cambio se hace entero.
    ///
    /// Se corre ANTES y DESPUES del ensanchamiento de relleno: el delta
    /// entre las dos ejecuciones del **mismo** test es el coste por
    /// circuito.
    ///
    /// **INSTRUMENTO, no comprobacion**: mide y no juzga. Se salta siempre.
    /// Correr en release, o se mide el compilador:
    ///
    /// ```text
    /// cargo test --release -p stark-experiment el_coste_de_tres_columnas \
    ///     -- --ignored --nocapture
    /// ```
    #[test]
    #[ignore = "instrumento de medida, no comprobacion: correr a mano"]
    fn el_coste_de_tres_columnas_y_quince_restricciones() {
        use std::time::Instant;

        let s = scenario(1_000_000, 250_000, 250_000, 500_000);
        let prover = SettlementProver::new(default_options());

        // Calentamiento: la primera prueba paga tablas y asignaciones que
        // no son parte de lo que se mide.
        let cal = build_trace(
            &s.sender, &s.receiver, s.amount, s.credited, s.limit,
            &s.null_path, &s.frozen_path,
        );
        let _ = prover.prove(cal).expect("calentamiento");

        const N: usize = 5;
        let mut bytes = 0usize;
        let t0 = Instant::now();
        for _ in 0..N {
            let trace = build_trace(
                &s.sender, &s.receiver, s.amount, s.credited, s.limit,
                &s.null_path, &s.frozen_path,
            );
            let proof = prover.prove(trace).expect("la traza valida debe probar");
            bytes = proof.to_bytes().len();
        }
        let dt = t0.elapsed().as_secs_f64() / N as f64;

        println!("\n=== Coste de la anchura, en `circuit_settlement` ===\n");
        println!("  TRACE_WIDTH        {TRACE_WIDTH:>8}");
        println!("  NUM_CONSTRAINTS    {NUM_CONSTRAINTS:>8}");
        println!("  prueba             {bytes:>8} B");
        println!("  generar            {:>8.1} ms   (media de {N})", dt * 1000.0);
        println!();
        println!("  §85.2 cuenta que ensanchar `sk` a cuatro elementos son");
        println!("  +3 columnas y +15 restricciones. Corre esto ANTES y");
        println!("  DESPUES del ensanchamiento de relleno: el delta entre");
        println!("  las dos ejecuciones es lo que costaria por circuito.");
        println!();
        println!("  ⚠️ Y multiplicalo por CINCO circuitos de gasto, mas los");
        println!("     de umbral y gobernanza.");
    }

    #[test]
    fn authorized_valid_transfer_verifies() {
        let s = scenario(1_000_000, 250_000, 250_000, 500_000);
        let trace = build_trace(
            &s.sender,
            &s.receiver,
            s.amount,
            s.credited,
            s.limit,
            &s.null_path,
            &s.frozen_path,
        );
        let prover = SettlementProver::new(default_options());
        let proof = prover.prove(trace).expect("la traza valida deberia probar");

        let min_opts = AcceptableOptions::OptionSet(vec![default_options()]);
        let v = verify::<SettlementAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof,
            s.public_inputs.clone(),
            &min_opts,
        );
        assert!(v.is_ok(), "{v:?}");
    }

    /// Comprueba que la traza contiene las raíces del ÁRBOL DE
    /// NULLIFIERS esperadas. Separa "la traza está mal construida" de
    /// "las restricciones están mal escritas".
    #[test]
    fn nullifier_tree_roots_match_native() {
        let s = scenario(1_000_000, 250_000, 250_000, 500_000);
        let trace = build_trace(
            &s.sender,
            &s.receiver,
            s.amount,
            s.credited,
            s.limit,
            &s.null_path,
            &s.frozen_path,
        );
        for i in 0..4 {
            assert_eq!(
                trace.get(4 + i, ROW_NULL_ROOT),
                s.public_inputs.nullifier_root_old[i],
                "raiz antigua del arbol de nullifiers, elem {i}"
            );
            assert_eq!(
                trace.get(LANE_B + 4 + i, ROW_NULL_ROOT),
                s.public_inputs.nullifier_root_new[i],
                "raiz nueva del arbol de nullifiers, elem {i}"
            );
        }
    }

    /// **EL TEST DE LA AUTORIDAD DE GASTO.**
    ///
    /// ## Por qué la primera versión no valía
    ///
    /// Cambiaba `spend_key` sin más. Pero la identidad se DERIVA de la
    /// clave, así que cambiaba también la hoja y `root_old` — y el test
    /// fallaba por la aserción de la raíz, no por la restricción de
    /// autoridad. El test pasaba aunque `C_PK_CHECK` no hiciera nada.
    /// Tercer caso del mismo patrón en este proyecto.
    ///
    /// ## El ataque real
    ///
    /// El atacante conoce la identidad de la víctima, su saldo, su nonce
    /// y su camino de Merkle. Construye la traza **con la identidad de la
    /// víctima** —de modo que `root_old` cuadra perfectamente— pero con
    /// **su propia clave de gasto**.
    ///
    /// Todo encaja salvo una cosa: la `pk` derivada de su clave no
    /// coincide con la identidad de la hoja. **Solo `C_PK_CHECK` puede
    /// detectarlo.**
    #[test]
    fn attacker_without_spend_key_cannot_transfer() {
        let s = scenario(1_000_000, 250_000, 250_000, 500_000);
        let victim_id = derive_public_id_wide(s.sender.spend_key);

        // El atacante usa la identidad de la victima con SU clave.
        let attacker = SenderWitness {
            // ⚠️ Ancha de verdad, no `as_digest(x)`: con relleno de ceros
            // el test seguiria siendo valido —§90: rellenar conserva la
            // identidad— pero no ejercitaria los tres elementos nuevos.
            spend_key: [
                BaseElement::new(0x1337),
                BaseElement::new(0xBADC0DE),
                BaseElement::new(0x0DDBA11),
                BaseElement::new(0x1CEB00DA),
            ],
            balance: s.sender.balance,
            nonce: s.sender.nonce,
            path: s.sender.path.clone(),
        };
        let trace = build_trace_with_id(
            &attacker,
            &s.receiver,
            s.amount,
            s.credited,
            s.limit,
            victim_id,
            &s.null_path,
            [BaseElement::ZERO; 4],
            &s.frozen_path,
        );

        let prover = SettlementProver::new(default_options());
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| prover.prove(trace)));

        match r {
            Err(_) => {}
            Ok(Err(_)) => {}
            Ok(Ok(proof)) => {
                let min_opts = AcceptableOptions::OptionSet(vec![default_options()]);
                let v = verify::<
                    SettlementAir,
                    Blake3,
                    DefaultRandomCoin<Blake3>,
                    MerkleTree<Blake3>,
                >(proof, s.public_inputs.clone(), &min_opts);
                assert!(
                    v.is_err(),
                    "CRITICO: conocer la identidad, el saldo, el nonce y el camino \
                     de una cuenta NO debe bastar para gastar. Sin la clave de \
                     gasto no debe haber prueba valida."
                );
            }
        }
    }

    /// **EL TEST DE LA NO-PERTENENCIA.**
    ///
    /// Se construye una traza donde el carril A arranca en el árbol de
    /// nullifiers **desde el propio nullifier** en vez de desde cero. Es
    /// decir: se afirma que la posición ya estaba ocupada por él — el
    /// testigo que necesitaría alguien para gastarlo por segunda vez.
    ///
    /// La traza es internamente coherente (los hashes del carril A son
    /// correctos respecto a esa hoja), así que las restricciones de hash
    /// no la ven. **Solo `C_NULL_EMPTY` puede detectarlo.**
    #[test]
    fn occupied_nullifier_position_is_rejected() {
        let s = scenario(1_000_000, 250_000, 250_000, 500_000);
        let nullifier = s.public_inputs.nullifier;

        let trace = build_trace_with_id(
            &s.sender,
            &s.receiver,
            s.amount,
            s.credited,
            s.limit,
            derive_public_id_wide(s.sender.spend_key),
            &s.null_path,
            nullifier, // la posicion "ya estaba ocupada"
            &s.frozen_path,
        );

        let prover = SettlementProver::new(default_options());
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| prover.prove(trace)));

        match r {
            Err(_) => {}
            Ok(Err(_)) => {}
            Ok(Ok(proof)) => {
                let min_opts = AcceptableOptions::OptionSet(vec![default_options()]);
                let mut declared = s.public_inputs.clone();
                declared.nullifier_root_old =
                    crate::nullifier_tree::climb(nullifier, &s.null_path);
                let v = verify::<
                    SettlementAir,
                    Blake3,
                    DefaultRandomCoin<Blake3>,
                    MerkleTree<Blake3>,
                >(proof, declared, &min_opts);
                assert!(
                    v.is_err(),
                    "CRITICO: afirmar no-pertenencia en una posicion OCUPADA debe \
                     rechazarse. Si verifica, el doble gasto es posible."
                );
            }
        }
    }

    #[test]
    fn money_creation_is_rejected() {
        let s = scenario(1_000_000, 250_000, 260_000, 500_000);
        assert!(run(&s, s.public_inputs.clone()).is_err());
    }

    #[test]
    fn money_destruction_is_rejected() {
        let s = scenario(1_000_000, 250_000, 240_000, 500_000);
        assert!(run(&s, s.public_inputs.clone()).is_err());
    }

    #[test]
    fn insufficient_balance_is_rejected() {
        let s = scenario(100_000, 250_000, 250_000, 500_000);
        assert!(run(&s, s.public_inputs.clone()).is_err());
    }

    #[test]
    fn over_regulatory_limit_is_rejected() {
        let s = scenario(1_000_000, 750_000, 750_000, 500_000);
        assert!(run(&s, s.public_inputs.clone()).is_err());
    }

    #[test]
    fn forged_nullifier_is_rejected() {
        let s = scenario(1_000_000, 250_000, 250_000, 500_000);
        let mut declared = s.public_inputs.clone();
        declared.nullifier = [BaseElement::new(31337); 4];
        assert!(run(&s, declared).is_err());
    }

    /// **EL TEST QUE JUSTIFICA TODA LA FASE DE CONGELADOS.**
    ///
    /// Una cuenta congelada NO puede gastar.
    ///
    /// La prueba se construye con una hoja CERO en la posición del emisor
    /// —es lo único que `build_trace` sabe hacer— pero se declara la raíz
    /// que corresponde a esa cuenta **congelada**. Las dos no coinciden, y
    /// la verificación falla.
    ///
    /// Sin este caso, los otros trece tests pasarían igual con la fase de
    /// congelados **completamente rota**: todos usan cuentas libres.
    #[test]
    fn a_frozen_account_cannot_spend() {
        let s = scenario(1_000_000, 250_000, 250_000, 500_000);
        let trace = build_trace(
            &s.sender,
            &s.receiver,
            s.amount,
            s.credited,
            s.limit,
            &s.null_path,
            &s.frozen_path,
        );
        let prover = SettlementProver::new(default_options());
        let proof = prover.prove(trace).expect("prove");

        // Se declara la raiz del arbol con ESTA CUENTA CONGELADA.
        let mut declared = s.public_inputs.clone();
        declared.frozen_root = crate::circuit_freeze::frozen_climb(
            crate::circuit_freeze::frozen_leaf(true),
            &s.frozen_path,
        );
        assert_ne!(
            declared.frozen_root, s.public_inputs.frozen_root,
            "congelar debe cambiar la raiz, o el test no comprueba nada"
        );

        let min_opts = AcceptableOptions::OptionSet(vec![default_options()]);
        let v = verify::<SettlementAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof, declared, &min_opts,
        );
        assert!(
            v.is_err(),
            "CRITICO: una cuenta congelada NO debe poder gastar"
        );
    }

    /// **Y el que valida al anterior**: con la cuenta LIBRE, la misma
    /// liquidación sí verifica.
    ///
    /// Si no lo hiciera, el test de arriba pasaría por cualquier motivo y
    /// no probaría nada sobre la congelación.
    #[test]
    fn a_free_account_can_spend() {
        let s = scenario(1_000_000, 250_000, 250_000, 500_000);
        let trace = build_trace(
            &s.sender,
            &s.receiver,
            s.amount,
            s.credited,
            s.limit,
            &s.null_path,
            &s.frozen_path,
        );
        let prover = SettlementProver::new(default_options());
        let proof = prover.prove(trace).expect("prove");
        let min_opts = AcceptableOptions::OptionSet(vec![default_options()]);
        let v = verify::<SettlementAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof,
            s.public_inputs.clone(),
            &min_opts,
        );
        assert!(v.is_ok(), "una cuenta libre debe poder gastar: {v:?}");
    }

    /// **PRUEBA POR MUTACIÓN: ninguna restricción está vacía.**
    ///
    /// Si ninguna perturbación de una celda hace que una restricción se
    /// vuelva no nula, esa restricción no impone nada — y ningún test
    /// normal lo detecta. Ver `AUDITORIA.md` §12.
    ///
    /// Se prueban **todas** las filas: con muestreo, una restricción activa
    /// en una sola fila aparece como vacía sin serlo.
    ///
    /// ⚠️ Un resultado limpio **no significa que el circuito sea correcto**:
    /// significa que no tiene este fallo concreto.
    #[test]
    fn no_constraint_is_vacuous() {
        use crate::mutation::{buscar_vacias, rows_of};

        let s = scenario(1_000_000, 250_000, 250_000, 500_000);
        let trace = build_trace(
            &s.sender,
            &s.receiver,
            s.amount,
            s.credited,
            s.limit,
            &s.null_path,
            &s.frozen_path,
        );
        let rows = rows_of(&trace, TRACE_WIDTH, TRACE_LENGTH);

        let air = SettlementAir::new(
            TraceInfo::new(TRACE_WIDTH, TRACE_LENGTH),
            s.public_inputs.clone(),
            default_options(),
        );
        let informe = buscar_vacias(&air, &rows, 1);

        assert!(
            informe.nunca_disparadas.is_empty(),
            "restricciones que NINGUNA perturbacion activa (de {} totales, \
             {} celdas probadas): {:?}",
            informe.total,
            informe.celdas,
            informe.nunca_disparadas
        );
    }
}


/// Dominio del salt de hoja (entrada 50; cierra §108.4).
///
/// Valor autodescriptivo —"SALTLEAF" en ASCII— y **distinto de todo dominio
/// existente: `t2a_dominio` lo comprueba, no lo promete**. Si colisionara
/// con `NULLIFIER_DOMAIN`, el salt seria el estado interno del nullifier.
pub const LEAF_SALT_DOMAIN: u64 = 0x53414C54_4C454146;

/// **Salt de hoja, derivado de la clave de gasto ANCHA.**
///
/// Decision de la entrada 50: el salt no es un secreto nuevo — se deriva de
/// la clave, en cliente, con la familia de hash del proyecto. Quien tiene la
/// clave lo re-deriva; quien la pierde ya lo habia perdido todo (§93.4: el
/// cliente no custodia estado, y esto no se lo pide).
///
/// ⚠️ Declarado: (1) acopla el salt a la clave — rotar clave implicara
/// nueva hoja; (2) es **convencion del cliente de referencia**, el protocolo
/// no impone el origen; (3) protege de terceros que ven caminos y pruebas —
/// **del operador no, y no lo pretende** (el operador ve los saldos).
pub fn derive_leaf_salt_wide(spend_key: Digest) -> Digest {
    native_merge(as_digest(BaseElement::new(LEAF_SALT_DOMAIN)), spend_key)
}

/// Anchura estrecha: rellena y hereda la garantia de §90 —
/// `[sk,0,0,0]` es la MISMA cuenta, luego el MISMO salt.
pub fn derive_leaf_salt(spend_key: BaseElement) -> Digest {
    derive_leaf_salt_wide(as_digest(spend_key))
}

/// **Hoja salteada** (entrada 50). Extiende `native_leaf` con el salt
/// de §117 SIN tocar la hoja vieja ni sus call-sites: el despliegue —
/// sustituir `native_leaf` por esta en los cinco circuitos y sus AIR —
/// es B13/B14. Aqui existe para que la propiedad de recuperacion sea
/// demostrable (T2b-nativo) antes de tocar traza alguna.
///
/// Estructura = la vieja con un merge mas de salt al final: el circuito
/// paga un bloque Rescue adicional por hoja (clase entrada 15, §82).
pub fn native_leaf_salted(
    public_id: Digest,
    balance: BaseElement,
    nonce: BaseElement,
    leaf_salt: Digest,
) -> Digest {
    native_merge(native_leaf(public_id, balance, nonce), leaf_salt)
}

/// Dominio de la CLAVE DE VISTA (entrada 49). Distinto de todo dominio
/// vivo — `t7_vista` lo comprueba: si coincidiera con SPEND_KEY la clave
/// de vista SERIA la identidad y no cegaria nada; si con LEAF_SALT,
/// presentar la vista revelaria el salt de hoja.
pub const VIEW_KEY_DOMAIN: u64 = 0x56494557_4B455900; // "VIEWKEY\0"

/// **Clave de vista**: credencial de LECTURA derivada de la clave de
/// gasto (entrada 49; patron de §117 aplicado a lectura). El titular la
/// presenta; la capa la compara contra el `view_id` guardado al abrir la
/// cuenta. Barata (un merge, verificable NATIVAMENTE — no el STARK de
/// ~600 ms que la 49 declara inaceptable), y NO viaja en cada operacion
/// —a diferencia del salt que §109 descarto por eso—.
///
/// ⚠️ Limitacion declarada: acoplada a la clave, solo rota rotando la
/// clave (como el salt de §117). Una credencial de lectura ROTABLE de
/// verdad exigiria un secreto nuevo custodiado (§93.4 lo prohibe) — se
/// elige el acoplamiento sobre el secreto nuevo, conscientemente.
pub fn derive_view_key(spend_key: BaseElement) -> Digest {
    native_merge(
        as_digest(BaseElement::new(VIEW_KEY_DOMAIN)),
        as_digest(spend_key),
    )
}

/// **view_id a partir de una CLAVE DE VISTA ya derivada** (49-A paso 4).
/// El titular presenta `derive_view_key(sk)` —no su clave de gasto— y la
/// capa computa este merge para comparar contra el `view_id` guardado.
/// Es el segundo merge de `view_id_of`: `view_id_of(sk) ==
/// view_id_from_view_key(derive_view_key(sk))`. Existe para que la puerta
/// autenticada no reimplemente el hash ni reciba la clave de gasto.
pub fn view_id_from_view_key(view_key: Digest) -> Digest {
    native_merge(as_digest(BaseElement::new(VIEW_KEY_DOMAIN)), view_key)
}

/// El `view_id` que la cuenta guarda: hash de la clave de vista. Guardar
/// el hash y no la clave permite verificar por presentacion sin que el
/// operador quede con material que le deje LEER (solo COMPARAR).
/// Variante ANCHA de la clave de vista (49-A paso 2). Hereda §90:
/// `[sk,0,0,0]` y `sk` dan el MISMO view_id porque `derive_public_id`
/// ya lo garantiza y esta se define sobre la misma anchura.
pub fn derive_view_key_wide(spend_key: Digest) -> Digest {
    native_merge(as_digest(BaseElement::new(VIEW_KEY_DOMAIN)), spend_key)
}

/// Variante ANCHA del view_id almacenado (49-A paso 2).
pub fn view_id_of_wide(spend_key: Digest) -> Digest {
    native_merge(
        as_digest(BaseElement::new(VIEW_KEY_DOMAIN)),
        derive_view_key_wide(spend_key),
    )
}

pub fn view_id_of(spend_key: BaseElement) -> Digest {
    native_merge(
        as_digest(BaseElement::new(VIEW_KEY_DOMAIN)),
        derive_view_key(spend_key),
    )
}

#[cfg(test)]
mod t7_clave_de_vista {
    //! T7 (entrada 49): el NUCLEO de la credencial de lectura, demostrado
    //! realizable. NO es el cierre de la 49 —autenticar las cuatro puertas
    //! toca AccountRecord y ~100 call-sites: eso es el despliegue—. Aqui
    //! se decide el mecanismo y se prueba su propiedad.
    use super::*;

    const SK: u64 = 0xA11CE;
    const SK_OTRO: u64 = 0xBADCAFE;

    #[test]
    fn t7_solo_el_titular_presenta_la_vista_correcta() {
        let sk = BaseElement::new(SK);
        // Al abrir, la cuenta guarda view_id_of(sk).
        let guardado = view_id_of(sk);
        // El titular re-deriva su clave de vista y la capa comprueba.
        let presentada = derive_view_key(sk);
        assert_eq!(
            native_merge(as_digest(BaseElement::new(VIEW_KEY_DOMAIN)), presentada),
            guardado,
            "el titular no reproduce su propio view_id"
        );
        // Un tercero SIN la clave no puede presentar la vista correcta.
        let intruso = derive_view_key(BaseElement::new(SK_OTRO));
        assert_ne!(
            native_merge(as_digest(BaseElement::new(VIEW_KEY_DOMAIN)), intruso),
            guardado,
            "un tercero paso el control de vista"
        );
    }

    #[test]
    fn t7_la_vista_no_es_credencial_de_GASTO() {
        // Presentar la clave de vista no debe permitir DERIVAR la de gasto
        // ni la identidad: son dominios separados. Comprobamos que ni la
        // clave de vista ni el view_id coinciden con identidad o nullifier.
        let sk = BaseElement::new(SK);
        let vk = derive_view_key(sk);
        assert_ne!(vk, derive_public_id(sk), "vista == identidad");
        for n in [0u64, 1, 7] {
            assert_ne!(vk, native_nullifier_wide(as_digest(sk), BaseElement::new(n)));
        }
        // Y la vista es DERIVABLE de la clave, no al reves: dado solo el
        // view_id, no hay atajo a la clave (lo garantiza Rescue; aqui
        // fijamos que view_id != clave y != clave de vista en claro).
        assert_ne!(view_id_of(sk), vk, "view_id == clave de vista en claro");
        assert_ne!(view_id_of(sk), as_digest(sk), "view_id == clave");
    }

    #[test]
    fn t7_dominio() {
        assert_ne!(VIEW_KEY_DOMAIN, SPEND_KEY_DOMAIN);
        assert_ne!(VIEW_KEY_DOMAIN, NULLIFIER_DOMAIN);
        assert_ne!(VIEW_KEY_DOMAIN, LEAF_SALT_DOMAIN);
        assert_ne!(VIEW_KEY_DOMAIN, crate::circuit_mint_pending::CUSTODIAN_DOMAIN);
        assert_ne!(VIEW_KEY_DOMAIN, crate::circuit_governance::GOVERNANCE_DOMAIN);
    }
}

mod t2b_recuperacion_nativa {
    //! T2b-nativo (entrada 50): la propiedad de RECUPERACION de §117,
    //! demostrada SIN el circuito. Decide la clausula de caida de §116:
    //! si esto compila y pasa, la propiedad es realizable y §117 se
    //! sostiene. T2b-circuito, condicionado entonces a B13/B14, quedó
    //! ESCRITO en el paso 4 (§154) sobre el gemelo del piloto:
    //! `circuit_send_salted::tests::t2b_circuito_*` — la clave sola
    //! produce prueba que VERIFICA; el diccionario sin salt NO. La
    //! versión por `apply` de la capa llega con el flip (D4).
    use super::*;

    const SK: u64 = 0xA11CE;

    fn d(x: u64) -> Digest {
        [BaseElement::new(x), BaseElement::ZERO, BaseElement::ZERO, BaseElement::ZERO]
    }

    /// El titular pierde TODO menos la clave. Con solo la clave rederiva
    /// identidad y salt (§117) y, con balance/nonce que su `ClientState`
    /// ya porta, reconstruye su hoja identica. Es T2b sin `apply`.
    #[test]
    fn t2b_solo_la_clave_reconstruye_la_hoja() {
        let sk = BaseElement::new(SK);
        let (balance, nonce) = (BaseElement::new(250_000), BaseElement::new(7));

        // Apertura: la hoja que quedo en el arbol.
        let id = derive_public_id(sk);
        let salt = derive_leaf_salt(sk);
        let hoja_en_arbol = native_leaf_salted(id, balance, nonce, salt);

        // Recuperacion: SOLO la clave (+ balance/nonce de ClientState).
        let id_r = derive_public_id(sk);
        let salt_r = derive_leaf_salt(sk);
        let hoja_r = native_leaf_salted(id_r, balance, nonce, salt_r);

        assert_eq!(hoja_r, hoja_en_arbol, "la clave sola no reconstruye la hoja");
    }

    /// La hoja salteada NO revela el balance por diccionario sobre el
    /// hermano de camino: cada balance candidato produce una hoja
    /// distinta SOLO si el atacante conoce el salt — que no conoce. Con
    /// el salt derivado de una clave que no tiene, el barrido de 0..N
    /// balances no acierta ninguno.
    #[test]
    fn t2b_diccionario_sin_salt_no_acierta() {
        let sk = BaseElement::new(SK);
        let (balance_real, nonce) = (BaseElement::new(3_500), BaseElement::new(1));
        let salt = derive_leaf_salt(sk);
        let id = derive_public_id(sk);
        let objetivo = native_leaf_salted(id, balance_real, nonce, salt);

        // Atacante: ve la hoja objetivo y el id (publico), NO el salt.
        // Barre el rango realista de §50 (aqui comprimido) con salt=0.
        let salt_falso = [BaseElement::ZERO; 4];
        let mut acierta = false;
        for b in 0..10_000u64 {
            if native_leaf_salted(id, BaseElement::new(b), nonce, salt_falso) == objetivo {
                acierta = true;
                break;
            }
        }
        assert!(!acierta, "el diccionario acerto SIN el salt: cegado roto");

        // Control: CON el salt correcto, el balance real si reproduce la
        // hoja — la hoja no es opaca al legitimo, solo al tercero.
        assert_eq!(native_leaf_salted(id, balance_real, nonce, salt), objetivo);
    }

    /// La hoja vieja (sin salt) SIGUE siendo vulnerable — esto NO es
    /// retroactivo, y el test lo fija: `native_leaf` sin salt reproduce
    /// el balance por barrido. Documenta que la 50 se cierra hacia
    /// delante, no sobre hojas ya escritas (coherente con §117).
    #[test]
    fn t2b_hoja_vieja_sigue_expuesta() {
        let id = d(0xB0B);
        let (balance_real, nonce) = (BaseElement::new(42), BaseElement::new(0));
        let objetivo = native_leaf(id, balance_real, nonce);
        let mut encontrado = None;
        for b in 0..1_000u64 {
            if native_leaf(id, BaseElement::new(b), nonce) == objetivo {
                encontrado = Some(b);
                break;
            }
        }
        assert_eq!(encontrado, Some(42), "la hoja vieja debe seguir siendo barrible");
    }
}

mod t2a_salt_hoja {
    //! T2a - la decision del salt, sometida a uso (disciplina de §108.5).
    use super::*;

    fn claves() -> [Digest; 3] {
        [
            as_digest(BaseElement::new(1)),
            as_digest(BaseElement::new(0xDEAD_BEEF)),
            [
                BaseElement::new(7),
                BaseElement::new(11),
                BaseElement::new(13),
                BaseElement::new(17),
            ],
        ]
    }

    #[test]
    fn t2a_dominio() {
        // La apuesta real: colision con NULLIFIER_DOMAIN haria del salt una
        // llave maestra de trazabilidad de la cuenta.
        assert_ne!(LEAF_SALT_DOMAIN, SPEND_KEY_DOMAIN);
        assert_ne!(LEAF_SALT_DOMAIN, NULLIFIER_DOMAIN);
        // Censo repo-wide (§116): los OTROS dos dominios vivos del crate.
        assert_ne!(LEAF_SALT_DOMAIN, crate::circuit_mint_pending::CUSTODIAN_DOMAIN);
        assert_ne!(LEAF_SALT_DOMAIN, crate::circuit_governance::GOVERNANCE_DOMAIN);
        // Los duplicados-por-copia YA NO EXISTEN (entrada 60, §125): cada
        // dominio tiene una definicion y el resto son reexports. Los
        // assert_eq que vivian aqui se volvieron tautologicos, y un test
        // que no discrimina es una garantia falsa (§9 del metodo): se
        // retiran con esta nota en su lugar.
        for k in claves() {
            let s = derive_leaf_salt_wide(k);
            assert_ne!(s, derive_public_id_wide(k), "salt == identidad");
            assert_ne!(s, k, "salt == clave");
            for n in [0u64, 1, 7] {
                assert_ne!(
                    s,
                    native_nullifier_wide(k, BaseElement::new(n)),
                    "salt == nullifier(nonce={n})"
                );
            }
        }
    }

    #[test]
    fn t2a_anchura_coherente() {
        // Hereda §90: dos vias de apertura, una cuenta, UN salt.
        for sk in [1u64, 0xDEAD_BEEF, 0xFFFF_FFFF_0000_0000] {
            let e = BaseElement::new(sk);
            assert_eq!(derive_leaf_salt(e), derive_leaf_salt_wide(as_digest(e)));
        }
    }

    #[test]
    fn t2a_determinista_no_trivial() {
        let ks = claves();
        assert_eq!(derive_leaf_salt_wide(ks[0]), derive_leaf_salt_wide(ks[0]));
        for i in 0..ks.len() {
            for j in i + 1..ks.len() {
                assert_ne!(
                    derive_leaf_salt_wide(ks[i]),
                    derive_leaf_salt_wide(ks[j]),
                    "claves distintas, mismo salt"
                );
            }
        }
        // La posicion importa: [sk,0,0,0] y [0,0,0,sk] no comparten salt.
        let sk = BaseElement::new(42);
        let inv = [BaseElement::ZERO, BaseElement::ZERO, BaseElement::ZERO, sk];
        assert_ne!(derive_leaf_salt(sk), derive_leaf_salt_wide(inv));
    }
}
