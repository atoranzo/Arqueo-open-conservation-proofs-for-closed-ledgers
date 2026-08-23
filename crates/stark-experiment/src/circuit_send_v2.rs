//! **Envio v2** (RFC-0003, S352) - EN PARALELO a `circuit_send`, que no
//! se toca: la convivencia manda que el 0.2 siga IDENTICO byte a byte,
//! y el legado es inmune por dominio (RFC-0003), asi que ambos
//! circuitos viven mientras existan pendientes de su forma.
//!
//! Lo UNICO que cambia respecto del v1 es la forma del pendiente que el
//! pagador deposita: aqui compone
//!
//! ```text
//! C2 = M( C1(identidad, aleatorio, importe), X )
//! ```
//!
//! donde `X` es el SOBRE de devolucion (`refund_envelope`, E1a/S345):
//! el pagador lo compone FUERA del circuito con `f` y `delta`, y aqui
//! entra OPACO como testigo. Probar que `X` esta bien formado es
//! materia de `circuit_refund_v2`, no de este: el envio solo acredita
//! que el compromiso depositado ENVUELVE ese sobre. La cadena del
//! pendiente gana un CUARTO merge:
//!
//!   ciclo PEND_IN    : (identidad, aleatorio) -> interno
//!   ciclo PEND_VAL   : (interno, importe)     -> C1
//!   ciclo PEND_ENV   : (C1, X)                -> C2      [NUEVO]
//!   subida PEND_CLIMB: C2 -> raiz de pendientes
//!
//! El sobre entra por `COL_X` (56..60) y se LEE en su unico enlace -
//! el patron del salt de hoja (S117), que este mismo fichero ya usa en
//! `COL_LEAF_SALT`: un testigo de un solo enlace no se transporta; un
//! X' distinto desvia C2 y la raiz declarada lo rechaza.
//!
//! Las entradas publicas son las del v1 (`SendPublicInputs`): el sobre
//! JAMAS se publica. Todo lo demas - titularidad, no-pertenencia a
//! congelados, rangos con el limite regulatorio dentro, la envoltura
//! de hoja S117 - es el calco del v1, cuya doctrina y comentarios
//! largos viven alla. Fichero PROPIO porque el guardian de layout
//! barre por fichero: dos Air en uno mezclarian sus ranuras.

use winterfell::crypto::hashers::{Blake3_256, Rp64_256};
use winterfell::crypto::{DefaultRandomCoin, MerkleTree};
use winterfell::math::{fields::f64::BaseElement, FieldElement};
use winterfell::matrix::ColMatrix;
use winterfell::{
    Air, AirContext, Assertion, AuxRandElements, CompositionPoly, CompositionPolyTrace,
    ConstraintCompositionCoefficients, DefaultConstraintCommitment, DefaultConstraintEvaluator,
    DefaultTraceLde, EvaluationFrame, PartitionOptions, ProofOptions, Prover, StarkDomain,
    TraceInfo, TracePolyTable, TraceTable, TransitionConstraintDegree,
};

use crate::circuit_freeze::FROZEN_DEPTH;
use crate::circuit_send::SendPublicInputs;
use crate::merkle::{Digest, MerklePath, TREE_DEPTH};
use crate::native::SPEND_KEY_DOMAIN;
use crate::rescue_hash::{apply_sbox, NUM_ROUNDS, STATE_WIDTH};

pub const CYCLE_LENGTH: usize = 8;
/// 1024 filas. Con el cuarto merge la tuberia acaba en
/// `ROW_PENDING_ROOT` (fila 823): quedan 200 filas de holgura
/// (25 ciclos) sin `hash_flag` ni ARK.
pub const TRACE_LENGTH: usize = 1024;
pub const SEGMENT_LENGTH: usize = 64;
/// Cinco segmentos: saldo, importe, saldo nuevo, suministro nuevo, y
/// **`limite - importe`**, que impone el limite regulatorio dentro.
pub const NUM_SEGMENTS: usize = 5;

// ===== Columnas =====
const LANE_B: usize = STATE_WIDTH;
const COL_BIT: usize = 24;
/// Clave de gasto del titular. CUATRO elementos desde S90.
const COL_KEY: usize = 25; // 25..29
const COL_ACC_ID: usize = 29; // 29..33
const COL_BAL: usize = 33;
const COL_BAL_NEW: usize = 34;
const COL_NONCE: usize = 35;
const COL_AMT: usize = 36;
const COL_SUPPLY_OLD: usize = 37;
const COL_SUPPLY_NEW: usize = 38;
const COL_SBIT: usize = 39;
const COL_SACC: usize = 40;
/// Bit de direccion del camino en el arbol de congelados.
const COL_FBIT: usize = 41;
/// Bit de direccion en el arbol de PENDIENTES.
const COL_PBIT: usize = 42;
/// Identidad publica del receptor, que funciona como direccion.
const COL_R_ID: usize = 43; // 43..47
/// Aleatorio que ciega el compromiso. Lo elige el pagador.
const COL_SALT: usize = 47; // 47..51
/// Limite regulatorio, transportado y demostrado (v. doctrina del v1).
const COL_LIMIT: usize = 51;
/// Salt de la hoja (testigo, S117): tercer merge de la hoja.
const COL_LEAF_SALT: usize = 52; // 52..56
/// **El SOBRE `X` (testigo, RFC-0003).** Solo se LEE en el enlace
/// `pend_env` - patron del salt de hoja: no se transporta.
const COL_X: usize = 56; // 56..60
pub const TRACE_WIDTH: usize = 60;

// ===== Filas =====
//
// Geometria derivada (SB0, S140): cada tramo arranca en un ciclo
// `CYC_*` y las filas-hito `ROW_*` se derivan de el. Ningun literal de
// ciclo vive fuera de este bloque.
const CYC_NONCE: usize = 1;
const CYC_SALT: usize = CYC_NONCE + 1;
const CYC_ACC: usize = CYC_SALT + 1;
const CYC_PK: usize = CYC_ACC + TREE_DEPTH;
const CYC_FROZEN: usize = CYC_PK + 1;
const CYC_PEND_IN: usize = CYC_FROZEN + FROZEN_DEPTH;
const CYC_PEND_VAL: usize = CYC_PEND_IN + 1;
/// **El CUARTO merge (RFC-0003)**: `C2 = M(C1, X)`. Todo el calendario
/// posterior se corre +1 ciclo solo, por derivacion.
const CYC_PEND_ENV: usize = CYC_PEND_VAL + 1;
const CYC_PEND_CLIMB: usize = CYC_PEND_ENV + 1;
const CYC_FIN: usize = CYC_PEND_CLIMB + TREE_DEPTH;
const ROW_LEAF_LINK: usize = CYC_NONCE * CYCLE_LENGTH - 1;
const ROW_SALT_LINK: usize = CYC_SALT * CYCLE_LENGTH - 1;
const ROW_LEAF_DONE: usize = CYC_ACC * CYCLE_LENGTH - 1;
const ROW_ROOT: usize = CYC_PK * CYCLE_LENGTH - 1;
const ROW_PK_START: usize = CYC_PK * CYCLE_LENGTH;
const ROW_PK_DONE: usize = CYC_FROZEN * CYCLE_LENGTH - 1;
const ROW_FROZEN_ROOT: usize = CYC_PEND_IN * CYCLE_LENGTH - 1;
/// Fila de enlace que siembra `(interno, importe)`; el ciclo
/// `CYC_PEND_VAL` hashea C1.
const ROW_PEND_INNER: usize = CYC_PEND_VAL * CYCLE_LENGTH - 1;
/// **Fila de enlace del CUARTO merge**: C1 disponible; siembra
/// `(C1, X)` con el sobre leido de `COL_X`. El ciclo `CYC_PEND_ENV`
/// hashea C2.
const ROW_PEND_ENV: usize = CYC_PEND_ENV * CYCLE_LENGTH - 1;
/// C2 disponible; entrada al arbol de pendientes.
const ROW_PENDING_ENTRY: usize = CYC_PEND_CLIMB * CYCLE_LENGTH - 1;
/// Raiz tras insertarlo.
const ROW_PENDING_ROOT: usize = CYC_FIN * CYCLE_LENGTH - 1;

// El presupuesto, en compilacion: la tuberia debe caber en la traza.
const _: () = assert!(ROW_PENDING_ROOT < TRACE_LENGTH);

// ===== Restricciones =====
const C_HASH_A: usize = 0;
const C_HASH_B: usize = C_HASH_A + STATE_WIDTH;
const C_CAP_A: usize = C_HASH_B + STATE_WIDTH;
const C_CAP_B: usize = C_CAP_A + 4;
const C_PLACE_A: usize = C_CAP_B + 4;
const C_PLACE_B: usize = C_PLACE_A + 4;
const C_SIBLING: usize = C_PLACE_B + 4;
const C_BIT_BOOL: usize = C_SIBLING + 4;
const C_LEAF_CAP_A: usize = C_BIT_BOOL + 1;
const C_LEAF_CAP_B: usize = C_LEAF_CAP_A + 4;
const C_LEAF_DIG_A: usize = C_LEAF_CAP_B + 4;
const C_LEAF_DIG_B: usize = C_LEAF_DIG_A + 4;
const C_NONCE: usize = C_LEAF_DIG_B + 4; // 2
const C_INPUT: usize = C_NONCE + 2; // 10
const C_KEY_INPUT: usize = C_INPUT + 10; // 8
/// TITULARIDAD: la identidad derivada coincide con la de la cuenta.
const C_PK_CHECK: usize = C_KEY_INPUT + 8; // 4
/// EL SALDO DISMINUYE EXACTAMENTE EN EL IMPORTE.
const C_BALANCE: usize = C_PK_CHECK + 4; // 1
/// UN ENVIO NO CAMBIA EL SUMINISTRO.
const C_SUPPLY: usize = C_BALANCE + 1; // 1
const C_TRANSPORT: usize = C_SUPPLY + 1; // 18 (10 + id receptor 4 + aleatorio 4)
const C_ID_CONST: usize = C_TRANSPORT + 18; // 4
const C_SBIT_BOOL: usize = C_ID_CONST + 4; // 2
const C_FIRST_S: usize = C_SBIT_BOOL + 2; // 2
const C_HORNER: usize = C_FIRST_S + 2; // 1
const C_SEG_LINK: usize = C_HORNER + 1;
const C_FROZEN_CAP: usize = C_SEG_LINK + NUM_SEGMENTS; // 4
/// LA NO-PERTENENCIA: la hoja colocada debe ser CERO.
const C_FROZEN_ENTRY: usize = C_FROZEN_CAP + 4; // 4
const C_FROZEN_PLACE: usize = C_FROZEN_ENTRY + 4; // 4
const C_FBIT_BOOL: usize = C_FROZEN_PLACE + 4; // 1
const C_PEND_IN: usize = C_FBIT_BOOL + 1; // 12 (capacidad 4 + identidad 4 + aleatorio 4)
const C_PEND_VAL: usize = C_PEND_IN + 12; // 5 (digest 4 + importe 1)
const C_PEND_CAP: usize = C_PEND_VAL + 5; // 8
const C_PEND_ENTRY_A: usize = C_PEND_CAP + 8; // 4
const C_PEND_ENTRY_B: usize = C_PEND_ENTRY_A + 4; // 4
const C_PEND_PLACE: usize = C_PEND_ENTRY_B + 4; // 8
const C_PEND_SIBLING: usize = C_PEND_PLACE + 8; // 4
const C_PBIT_BOOL: usize = C_PEND_SIBLING + 4; // 1
const C_SALT_CAP_A: usize = C_PBIT_BOOL + 1; // 4
const C_SALT_CAP_B: usize = C_SALT_CAP_A + 4; // 4
const C_SALT_DIG_A: usize = C_SALT_CAP_B + 4; // 4
const C_SALT_DIG_B: usize = C_SALT_DIG_A + 4; // 4
const C_SALT_IN_A: usize = C_SALT_DIG_B + 4; // 4
const C_SALT_IN_B: usize = C_SALT_IN_A + 4; // 4
/// El limite es constante entre filas (doctrina del v1: va al final).
const C_LIMIT_CONST: usize = C_SALT_IN_B + 4; // 1
/// **El CUARTO merge, carril A** (precedente `C_PEND_VAL`): digest
/// arrastrado (C1) y el rate := el sobre, leido de `COL_X`. La
/// capacidad no se ata: basura ahi desvia el hash y la raiz declarada
/// la rechaza (el porque documentado de las re-siembras del v1).
const C_ENV_DIG: usize = C_LIMIT_CONST + 1; // 4
const C_ENV_IN: usize = C_ENV_DIG + 4; // 4
const NUM_CONSTRAINTS: usize = C_ENV_IN + 4;

// ===== Periodicas =====
const P_HASH_FLAG: usize = 0;
const P_ARK1: usize = 1;
const P_ARK2: usize = P_ARK1 + STATE_WIDTH;
const P_LINK_MERKLE: usize = P_ARK2 + STATE_WIDTH;
const P_LINK_LEAF: usize = P_LINK_MERKLE + 1;
const P_LINK_SALT: usize = P_LINK_LEAF + 1;
const P_LINK_PLACE: usize = P_LINK_SALT + 1;
const P_FIRST_ROW: usize = P_LINK_PLACE + 1;
const P_SEL_ROOT: usize = P_FIRST_ROW + 1;
const P_SEL_PK_DONE: usize = P_SEL_ROOT + 1;
const P_FIRST_S: usize = P_SEL_PK_DONE + 1;
const P_CONT_S: usize = P_FIRST_S + 1;
const P_SEG_LINK: usize = P_CONT_S + 1;
const P_FROZEN_ENTRY: usize = P_SEG_LINK + NUM_SEGMENTS;
const P_FROZEN_LINK: usize = P_FROZEN_ENTRY + 1;
const P_PEND_IN: usize = P_FROZEN_LINK + 1;
const P_PEND_VAL: usize = P_PEND_IN + 1;
/// Fila del cuarto merge.
const P_PEND_ENV: usize = P_PEND_VAL + 1;
const P_PEND_ENTRY: usize = P_PEND_ENV + 1;
const P_PEND_LINK: usize = P_PEND_ENTRY + 1;

type Blake3 = Blake3_256<BaseElement>;

fn value_to_bits_be(value: u64) -> Vec<bool> {
    (0..SEGMENT_LENGTH)
        .map(|p| (value >> (SEGMENT_LENGTH - 1 - p)) & 1 == 1)
        .collect()
}

/// Construye la traza de un envio v2.
///
/// Identica a la del v1 salvo el parametro `sobre` (el `X` opaco del
/// RFC-0003), que entra por `COL_X` y se envuelve en el cuarto merge.
#[allow(clippy::too_many_arguments)]
pub fn build_trace(
    // `spend_key` son CUATRO elementos desde S90 (entrada 15).
    spend_key: Digest,
    account_id: Digest,
    balance: u64,
    nonce: BaseElement,
    // Salt de la hoja (testigo, S117): el tercer merge envuelve la
    // hoja; la pertenencia se prueba sobre `H(native_leaf, salt)`.
    leaf_salt: Digest,
    path: &MerklePath,
    frozen_path: &MerklePath,
    amount: u64,
    // Limite regulatorio del sistema. Lo aporta la capa, no el titular.
    regulatory_limit: u64,
    supply_old: u64,
    // Debe ser CERO: un envio no cambia el suministro. Se mantiene como
    // parametro para que un test pueda intentar lo contrario.
    supply_delta: u64,
    // Identidad publica del receptor, que funciona como direccion.
    receiver_id: Digest,
    // Aleatorio que ciega el compromiso. Lo elige el pagador.
    salt: Digest,
    // El SOBRE `X` (RFC-0003): opaco, compuesto fuera del circuito.
    sobre: Digest,
    // Camino de la posicion libre donde se inserta el pendiente.
    pending_path: &MerklePath,
) -> TraceTable<BaseElement> {
    let zero = BaseElement::ZERO;
    let c_bal = BaseElement::new(balance);
    let c_amt = BaseElement::new(amount);
    let c_bal_new = c_bal - c_amt;
    let c_supply_old = BaseElement::new(supply_old);
    let c_supply_new = c_supply_old - BaseElement::new(supply_delta);

    let mut rows: Vec<Vec<BaseElement>> = vec![vec![zero; TRACE_WIDTH]; TRACE_LENGTH];

    for row in rows.iter_mut() {
        row[COL_KEY..COL_KEY + 4].copy_from_slice(&spend_key);
        for i in 0..4 {
            row[COL_ACC_ID + i] = account_id[i];
        }
        row[COL_BAL] = c_bal;
        row[COL_BAL_NEW] = c_bal_new;
        row[COL_NONCE] = nonce;
        row[COL_AMT] = c_amt;
        row[COL_LIMIT] = BaseElement::new(regulatory_limit);
        row[COL_R_ID..COL_R_ID + 4].copy_from_slice(&receiver_id);
        row[COL_SALT..COL_SALT + 4].copy_from_slice(&salt);
        row[COL_LEAF_SALT..COL_LEAF_SALT + 4].copy_from_slice(&leaf_salt);
        row[COL_X..COL_X + 4].copy_from_slice(&sobre);
        row[COL_SUPPLY_OLD] = c_supply_old;
        row[COL_SUPPLY_NEW] = c_supply_new;
    }

    // Rangos. El quinto segmento impone `importe <= limite`: si lo
    // superara, la resta envuelve y no cabe en los 63 bits.
    let segment_values = [
        c_bal.as_int(),
        c_amt.as_int(),
        c_bal_new.as_int(),
        c_supply_new.as_int(),
        (BaseElement::new(regulatory_limit) - c_amt).as_int(),
    ];
    // Esta cuenta tiene que coincidir con NUM_SEGMENTS (doctrina v1).
    debug_assert_eq!(
        segment_values.len(),
        NUM_SEGMENTS,
        "cada segmento declarado necesita su valor"
    );
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

    let place_pending = |state: &mut [BaseElement; STATE_WIDTH], digest: &Digest, level: usize| {
        debug_assert!(
            level < TREE_DEPTH,
            "place_pending: nivel {} con path de {}",
            level,
            TREE_DEPTH
        );
        if pending_path.is_right[level] {
            state[4..8].copy_from_slice(&pending_path.siblings[level]);
            state[8..12].copy_from_slice(digest);
        } else {
            state[4..8].copy_from_slice(digest);
            state[8..12].copy_from_slice(&pending_path.siblings[level]);
        }
    };

    let place_frozen = |state: &mut [BaseElement; STATE_WIDTH], digest: &Digest, level: usize| {
        debug_assert!(
            level < FROZEN_DEPTH,
            "place_frozen: nivel {} con path de {}",
            level,
            FROZEN_DEPTH
        );
        if frozen_path.is_right[level] {
            state[4..8].copy_from_slice(&frozen_path.siblings[level]);
            state[8..12].copy_from_slice(digest);
        } else {
            state[4..8].copy_from_slice(digest);
            state[8..12].copy_from_slice(&frozen_path.siblings[level]);
        }
    };

    let place = |state: &mut [BaseElement; STATE_WIDTH], digest: &Digest, level: usize| {
        debug_assert!(
            level < TREE_DEPTH,
            "place: nivel {} con path de {}",
            level,
            TREE_DEPTH
        );
        if path.is_right[level] {
            state[4..8].copy_from_slice(&path.siblings[level]);
            state[8..12].copy_from_slice(digest);
        } else {
            state[4..8].copy_from_slice(digest);
            state[8..12].copy_from_slice(&path.siblings[level]);
        }
    };

    let mut state_a = [zero; STATE_WIDTH];
    let mut state_b = [zero; STATE_WIDTH];
    state_a[4..8].copy_from_slice(&account_id);
    state_a[8] = c_bal;
    state_b[4..8].copy_from_slice(&account_id);
    state_b[8] = c_bal_new;

    rows[0][..STATE_WIDTH].copy_from_slice(&state_a);
    rows[0][LANE_B..LANE_B + STATE_WIDTH].copy_from_slice(&state_b);

    for r in 0..ROW_PENDING_ROOT {
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
                ROW_LEAF_LINK => {
                    // El nonce NO cambia: enviar no consume el derecho
                    // de gasto (doctrina del v1).
                    state_a[4..8].copy_from_slice(&digest_a);
                    state_a[8] = nonce;
                    state_b[4..8].copy_from_slice(&digest_b);
                    state_b[8] = nonce;
                }
                ROW_SALT_LINK => {
                    // EL TERCER MERGE (S117): la hoja se envuelve con el
                    // salt. Digest arrastrado; el rate recibe los CUATRO
                    // limbos del salt.
                    state_a[4..8].copy_from_slice(&digest_a);
                    state_a[8..12].copy_from_slice(&leaf_salt);
                    state_b[4..8].copy_from_slice(&digest_b);
                    state_b[8..12].copy_from_slice(&leaf_salt);
                }
                ROW_LEAF_DONE => {
                    place(&mut state_a, &digest_a, 0);
                    place(&mut state_b, &digest_b, 0);
                }
                ROW_ROOT => {
                    state_a[4] = BaseElement::new(SPEND_KEY_DOMAIN);
                    state_a[8..12].copy_from_slice(&spend_key);
                    state_b[4] = BaseElement::new(SPEND_KEY_DOMAIN);
                    state_b[8..12].copy_from_slice(&spend_key);
                }
                ROW_FROZEN_ROOT => {
                    // COMPROMISO INTERNO: H(identidad_receptor, aleatorio).
                    state_a[..4].copy_from_slice(&[zero; 4]);
                    state_a[4..8].copy_from_slice(&receiver_id);
                    state_a[8..12].copy_from_slice(&salt);
                    state_b.copy_from_slice(&state_a);
                }
                ROW_PEND_INNER => {
                    // C1: H(interno, importe).
                    state_a[4..8].copy_from_slice(&digest_a);
                    state_a[8] = c_amt;
                    state_a[9] = zero;
                    state_a[10] = zero;
                    state_a[11] = zero;
                    state_b.copy_from_slice(&state_a);
                }
                ROW_PEND_ENV => {
                    // EL CUARTO MERGE (RFC-0003): C2 = M(C1, X). Digest
                    // arrastrado (C1) y el rate := el sobre, leido de
                    // COL_X. Ambos carriles llevan el mismo compromiso.
                    state_a[4..8].copy_from_slice(&digest_a);
                    state_a[8..12].copy_from_slice(&sobre);
                    state_b.copy_from_slice(&state_a);
                }
                ROW_PENDING_ENTRY => {
                    // ENTRADA AL ARBOL DE PENDIENTES.
                    //
                    // Carril A: hoja CERO -> la posicion estaba libre.
                    // Carril B: el compromiso C2 -> raiz nueva.
                    let libre: Digest = [zero; 4];
                    place_pending(&mut state_a, &libre, 0);
                    place_pending(&mut state_b, &digest_b, 0);
                }
                ROW_PK_DONE => {
                    // ENTRADA AL ARBOL DE CONGELADOS: hoja CERO en la
                    // posicion del titular.
                    let libre: Digest = [zero; 4];
                    place_frozen(&mut state_a, &libre, 0);
                    place_frozen(&mut state_b, &libre, 0);
                }
                _ => {
                    let next_cycle = (r + 1) / CYCLE_LENGTH;
                    // Convencion unica (SB0, S140): tramo generico
                    // `(CYC_arranque..CYC_fin)`, nivel = next_cycle -
                    // CYC_arranque. El arranque lo sombrea su brazo
                    // explicito; el final queda FUERA del rango.
                    if (CYC_ACC..CYC_PK).contains(&next_cycle) {
                        let level = next_cycle - CYC_ACC;
                        place(&mut state_a, &digest_a, level);
                        place(&mut state_b, &digest_b, level);
                    } else if (CYC_FROZEN..CYC_PEND_IN).contains(&next_cycle) {
                        let level = next_cycle - CYC_FROZEN;
                        place_frozen(&mut state_a, &digest_a, level);
                        place_frozen(&mut state_b, &digest_b, level);
                    } else if (CYC_PEND_CLIMB..CYC_FIN).contains(&next_cycle) {
                        let level = next_cycle - CYC_PEND_CLIMB;
                        place_pending(&mut state_a, &digest_a, level);
                        place_pending(&mut state_b, &digest_b, level);
                    }
                }
            }
        }
        rows[r + 1][..STATE_WIDTH].copy_from_slice(&state_a);
        rows[r + 1][LANE_B..LANE_B + STATE_WIDTH].copy_from_slice(&state_b);
    }

    for level in 0..TREE_DEPTH {
        let bit = if pending_path.is_right[level] {
            BaseElement::ONE
        } else {
            zero
        };
        for p in 0..CYCLE_LENGTH {
            rows[(CYC_PEND_CLIMB + level) * CYCLE_LENGTH + p][COL_PBIT] = bit;
        }
    }

    for level in 0..FROZEN_DEPTH {
        let bit = if frozen_path.is_right[level] {
            BaseElement::ONE
        } else {
            zero
        };
        for p in 0..CYCLE_LENGTH {
            rows[(CYC_FROZEN + level) * CYCLE_LENGTH + p][COL_FBIT] = bit;
        }
    }

    for level in 0..TREE_DEPTH {
        let bit = if path.is_right[level] {
            BaseElement::ONE
        } else {
            zero
        };
        for p in 0..CYCLE_LENGTH {
            rows[(CYC_ACC + level) * CYCLE_LENGTH + p][COL_BIT] = bit;
        }
    }

    let mut trace = TraceTable::new(TRACE_WIDTH, TRACE_LENGTH);
    trace.fill(
        |s| s.copy_from_slice(&rows[0]),
        |step, s| s.copy_from_slice(&rows[step + 1]),
    );
    trace
}

pub struct SendV2Air {
    context: AirContext<BaseElement>,
    pub_inputs: SendPublicInputs,
}

impl Air for SendV2Air {
    type BaseField = BaseElement;
    type PublicInputs = SendPublicInputs;

    fn new(trace_info: TraceInfo, pub_inputs: Self::PublicInputs, options: ProofOptions) -> Self {
        assert_eq!(TRACE_WIDTH, trace_info.width());
        let full = vec![TRACE_LENGTH];

        let mut degrees = Vec::with_capacity(NUM_CONSTRAINTS);
        for _ in 0..2 * STATE_WIDTH {
            degrees.push(TransitionConstraintDegree::with_cycles(7, full.clone()));
        }
        for _ in 0..8 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }
        // Colocacion (8) + hermano (4): grado 2.
        for _ in 0..12 {
            degrees.push(TransitionConstraintDegree::with_cycles(2, full.clone()));
        }
        degrees.push(TransitionConstraintDegree::new(2));
        // Enlaces de hoja (16), nonce (2), entradas (10), clave (8),
        // titularidad (4) = 40, grado 1 con ciclo.
        for _ in 0..40 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }
        // Saldo (1), suministro (1), transporte (18), identidad-constante
        // (4) = 24, grado 1 sin ciclo.
        for _ in 0..24 {
            degrees.push(TransitionConstraintDegree::new(1));
        }
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
        // No-pertenencia (4) y colocacion (4): grado 2, multiplican por el bit.
        for _ in 0..8 {
            degrees.push(TransitionConstraintDegree::with_cycles(2, full.clone()));
        }
        // Bit booleano (1): grado 2 sin ciclo.
        degrees.push(TransitionConstraintDegree::new(2));

        // --- El pendiente ---
        // Compromiso interno (12) y completo (5): grado 1 con ciclo.
        for _ in 0..17 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }
        // Capacidad de la subida (8): grado 1 con ciclo.
        for _ in 0..8 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }
        // Entradas (8), colocacion (8), hermano (4): grado 2, multiplican
        // por el bit de direccion.
        for _ in 0..20 {
            degrees.push(TransitionConstraintDegree::with_cycles(2, full.clone()));
        }
        // Bit booleano (1): grado 2 sin ciclo.
        degrees.push(TransitionConstraintDegree::new(2));
        // La envoltura del salt (24): grado 1 con ciclo.
        for _ in 0..24 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }
        // Transporte del limite (1): grado 1 sin ciclo.
        degrees.push(TransitionConstraintDegree::new(1));
        // El cuarto merge (8): grado 1 con ciclo - el molde del salt.
        for _ in 0..8 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }

        assert_eq!(degrees.len(), NUM_CONSTRAINTS, "cuenta de grados");

        SendV2Air {
            // 42, como el v1: el conjunto de aserciones no cambia (las
            // filas-hito se derivan; el sobre es testigo, no se asierta).
            context: AirContext::new(trace_info, degrees, 42, options),
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
        for r in 0..=ROW_PENDING_ROOT {
            if r % CYCLE_LENGTH < NUM_ROUNDS {
                hash_flag[r] = one;
            }
        }
        columns.push(hash_flag);

        for ark in [true, false] {
            for i in 0..STATE_WIDTH {
                let mut col = vec![zero; TRACE_LENGTH];
                for r in 0..=ROW_PENDING_ROOT {
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
            link_merkle[(CYC_ACC + level) * CYCLE_LENGTH + 7] = one;
        }
        columns.push(link_merkle);

        let mut link_leaf = vec![zero; TRACE_LENGTH];
        link_leaf[ROW_LEAF_LINK] = one;
        columns.push(link_leaf);

        let mut link_salt = vec![zero; TRACE_LENGTH];
        link_salt[ROW_SALT_LINK] = one;
        columns.push(link_salt);

        let mut link_place = vec![zero; TRACE_LENGTH];
        link_place[ROW_LEAF_DONE] = one;
        columns.push(link_place);

        for row in [0, ROW_ROOT, ROW_PK_DONE] {
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
        frozen_entry[ROW_PK_DONE] = one;
        columns.push(frozen_entry);

        // Enlaces de la subida: uno por nivel a partir del primero.
        let mut frozen_link = vec![zero; TRACE_LENGTH];
        for level in 0..FROZEN_DEPTH - 1 {
            frozen_link[(CYC_FROZEN + level) * CYCLE_LENGTH + 7] = one;
        }
        columns.push(frozen_link);

        // Compromiso interno del pendiente: una sola fila.
        let mut pend_in = vec![zero; TRACE_LENGTH];
        pend_in[ROW_FROZEN_ROOT] = one;
        columns.push(pend_in);

        // Compromiso completo (C1).
        let mut pend_val = vec![zero; TRACE_LENGTH];
        pend_val[ROW_PEND_INNER] = one;
        columns.push(pend_val);

        // El cuarto merge (C2): una sola fila.
        let mut pend_env = vec![zero; TRACE_LENGTH];
        pend_env[ROW_PEND_ENV] = one;
        columns.push(pend_env);

        // Entrada al arbol de pendientes.
        let mut pend_entry = vec![zero; TRACE_LENGTH];
        pend_entry[ROW_PENDING_ENTRY] = one;
        columns.push(pend_entry);

        // Enlaces de la subida: uno por nivel a partir del primero.
        let mut pend_link = vec![zero; TRACE_LENGTH];
        for level in 0..TREE_DEPTH - 1 {
            pend_link[(CYC_PEND_CLIMB + level) * CYCLE_LENGTH + 7] = one;
        }
        columns.push(pend_link);

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
        let link_salt = periodic[P_LINK_SALT];
        let link_place = periodic[P_LINK_PLACE];
        let first_row = periodic[P_FIRST_ROW];
        let sel_root = periodic[P_SEL_ROOT];
        let sel_pk_done = periodic[P_SEL_PK_DONE];
        let first_s = periodic[P_FIRST_S];
        let cont_s = periodic[P_CONT_S];

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
        let tree_link = link_merkle + link_place;

        for i in 0..4 {
            result[C_CAP_A + i] = tree_link * next[i];
            result[C_CAP_B + i] = tree_link * next[LANE_B + i];

            let da = current[4 + i];
            result[C_PLACE_A + i] =
                tree_link * ((E::ONE - bit) * (next[4 + i] - da) + bit * (next[8 + i] - da));

            let db = current[LANE_B + 4 + i];
            result[C_PLACE_B + i] = tree_link
                * ((E::ONE - bit) * (next[LANE_B + 4 + i] - db)
                    + bit * (next[LANE_B + 8 + i] - db));

            let sib_a = (E::ONE - bit) * next[8 + i] + bit * next[4 + i];
            let sib_b = (E::ONE - bit) * next[LANE_B + 8 + i] + bit * next[LANE_B + 4 + i];
            result[C_SIBLING + i] = tree_link * (sib_a - sib_b);
        }

        result[C_BIT_BOOL] = current[COL_BIT] * (current[COL_BIT] - E::ONE);

        for i in 0..4 {
            result[C_LEAF_CAP_A + i] = link_leaf * next[i];
            result[C_LEAF_CAP_B + i] = link_leaf * next[LANE_B + i];
            result[C_LEAF_DIG_A + i] = link_leaf * (next[4 + i] - current[4 + i]);
            result[C_LEAF_DIG_B + i] =
                link_leaf * (next[LANE_B + 4 + i] - current[LANE_B + 4 + i]);
        }

        result[C_NONCE] = link_leaf * (next[8] - current[COL_NONCE]);
        result[C_NONCE + 1] = link_leaf * (next[LANE_B + 8] - current[COL_NONCE]);

        // EL TERCER MERGE (S117): la envoltura, cosida por link_salt.
        for i in 0..4 {
            result[C_SALT_CAP_A + i] = link_salt * next[i];
            result[C_SALT_CAP_B + i] = link_salt * next[LANE_B + i];
            result[C_SALT_DIG_A + i] = link_salt * (next[4 + i] - current[4 + i]);
            result[C_SALT_DIG_B + i] =
                link_salt * (next[LANE_B + 4 + i] - current[LANE_B + 4 + i]);
            result[C_SALT_IN_A + i] =
                link_salt * (next[8 + i] - current[COL_LEAF_SALT + i]);
            result[C_SALT_IN_B + i] =
                link_salt * (next[LANE_B + 8 + i] - current[COL_LEAF_SALT + i]);
        }

        for i in 0..4 {
            result[C_INPUT + i] = first_row * (current[4 + i] - current[COL_ACC_ID + i]);
            result[C_INPUT + 5 + i] =
                first_row * (current[LANE_B + 4 + i] - current[COL_ACC_ID + i]);
        }
        result[C_INPUT + 4] = first_row * (current[8] - current[COL_BAL]);
        result[C_INPUT + 9] = first_row * (current[LANE_B + 8] - current[COL_BAL_NEW]);

        // Los CUATRO elementos, en los dos carriles (S92.2).
        for i in 0..4 {
            result[C_KEY_INPUT + i] = sel_root * (next[8 + i] - current[COL_KEY + i]);
            result[C_KEY_INPUT + 4 + i] =
                sel_root * (next[LANE_B + 8 + i] - current[COL_KEY + i]);
        }

        // ===== TITULARIDAD =====
        for i in 0..4 {
            result[C_PK_CHECK + i] = sel_pk_done * (current[4 + i] - current[COL_ACC_ID + i]);
        }

        // ===== EL SALDO DISMINUYE EXACTAMENTE EN EL IMPORTE =====
        result[C_BALANCE] = current[COL_BAL_NEW] - (current[COL_BAL] - current[COL_AMT]);
        // ===== UN ENVIO NO CAMBIA EL SUMINISTRO =====
        result[C_SUPPLY] = current[COL_SUPPLY_NEW] - current[COL_SUPPLY_OLD];

        let transport = [
            COL_KEY,
            COL_KEY + 1,
            COL_KEY + 2,
            COL_KEY + 3,
            COL_BAL,
            COL_BAL_NEW,
            COL_NONCE,
            COL_AMT,
            COL_SUPPLY_OLD,
            COL_SUPPLY_NEW,
        ];
        // La identidad del receptor y el aleatorio tambien son constantes:
        // si variaran entre filas, el compromiso no seria el declarado.
        // Desplazamientos 10 y 14 (doctrina del v1, S81).
        for i in 0..4 {
            result[C_TRANSPORT + 10 + i] =
                next[COL_R_ID + i] - current[COL_R_ID + i];
            result[C_TRANSPORT + 14 + i] =
                next[COL_SALT + i] - current[COL_SALT + i];
        }
        for (k, col) in transport.iter().enumerate() {
            result[C_TRANSPORT + k] = next[*col] - current[*col];
        }
        for i in 0..4 {
            result[C_ID_CONST + i] = next[COL_ACC_ID + i] - current[COL_ACC_ID + i];
        }

        let sbit_cur = current[COL_SBIT];
        let sbit_next = next[COL_SBIT];
        let sacc_cur = current[COL_SACC];
        let sacc_next = next[COL_SACC];

        result[C_SBIT_BOOL] = sbit_cur * (sbit_cur - E::ONE);
        result[C_SBIT_BOOL + 1] = sbit_next * (sbit_next - E::ONE);
        result[C_FIRST_S] = first_s * sbit_cur;
        result[C_FIRST_S + 1] = first_s * sacc_cur;
        result[C_HORNER] = cont_s * (sacc_next - (sacc_cur + sacc_cur + sbit_next));

        // ===== NO-PERTENENCIA AL ARBOL DE CONGELADOS =====
        let frozen_entry = periodic[P_FROZEN_ENTRY];
        let frozen_link = periodic[P_FROZEN_LINK];
        let fbit = next[COL_FBIT];

        for i in 0..4 {
            result[C_FROZEN_CAP + i] = (frozen_entry + frozen_link) * next[i];
            result[C_FROZEN_ENTRY + i] =
                frozen_entry * ((E::ONE - fbit) * next[4 + i] + fbit * next[8 + i]);
            let d = current[4 + i];
            result[C_FROZEN_PLACE + i] =
                frozen_link * ((E::ONE - fbit) * (next[4 + i] - d) + fbit * (next[8 + i] - d));
        }
        result[C_FBIT_BOOL] = current[COL_FBIT] * (current[COL_FBIT] - E::ONE);

        // ===== EL PENDIENTE =====
        //
        // CUATRO fases: el compromiso interno, C1, el cuarto merge con
        // el sobre (C2), y la insercion en el arbol.
        let pend_in = periodic[P_PEND_IN];
        let pend_val = periodic[P_PEND_VAL];
        let pend_env = periodic[P_PEND_ENV];
        let pend_entry = periodic[P_PEND_ENTRY];
        let pend_link = periodic[P_PEND_LINK];
        let pbit = next[COL_PBIT];

        for i in 0..4 {
            // Compromiso interno: DOS restricciones separadas, no su
            // suma (doctrina del v1: sumar comprobaciones las anula).
            result[C_PEND_IN + i] = pend_in * next[i];
            result[C_PEND_IN + 4 + i] = pend_in * (next[4 + i] - current[COL_R_ID + i]);
            result[C_PEND_IN + 8 + i] = pend_in * (next[8 + i] - current[COL_SALT + i]);

            // Compromiso completo: el digest interno, y el importe.
            result[C_PEND_VAL + i] = pend_val * (next[4 + i] - current[4 + i]);

            // EL CUARTO MERGE: digest arrastrado (C1) y el rate := el
            // sobre, leido de COL_X donde vive como testigo.
            result[C_ENV_DIG + i] = pend_env * (next[4 + i] - current[4 + i]);
            result[C_ENV_IN + i] = pend_env * (next[8 + i] - current[COL_X + i]);
        }
        result[C_PEND_VAL + 4] = pend_val * (next[8] - current[COL_AMT]);

        // Subida al arbol de pendientes.
        let pend_any = pend_entry + pend_link;
        for i in 0..4 {
            result[C_PEND_CAP + i] = pend_any * next[i];
            result[C_PEND_CAP + 4 + i] = pend_any * next[LANE_B + i];

            // LA POSICION ESTABA LIBRE: el carril A entra con cero.
            result[C_PEND_ENTRY_A + i] =
                pend_entry * ((E::ONE - pbit) * next[4 + i] + pbit * next[8 + i]);
            // Y el B con el compromiso C2, que acaba de calcular.
            result[C_PEND_ENTRY_B + i] = pend_entry
                * ((E::ONE - pbit) * (next[LANE_B + 4 + i] - current[LANE_B + 4 + i])
                    + pbit * (next[LANE_B + 8 + i] - current[LANE_B + 4 + i]));

            let da = current[4 + i];
            let db = current[LANE_B + 4 + i];
            result[C_PEND_PLACE + i] =
                pend_link * ((E::ONE - pbit) * (next[4 + i] - da) + pbit * (next[8 + i] - da));
            result[C_PEND_PLACE + 4 + i] = pend_link
                * ((E::ONE - pbit) * (next[LANE_B + 4 + i] - db)
                    + pbit * (next[LANE_B + 8 + i] - db));

            // Hermano compartido: es la misma posicion del mismo arbol.
            let sib_a = (E::ONE - pbit) * next[8 + i] + pbit * next[4 + i];
            let sib_b = (E::ONE - pbit) * next[LANE_B + 8 + i] + pbit * next[LANE_B + 4 + i];
            result[C_PEND_SIBLING + i] = pend_link * (sib_a - sib_b);
        }
        result[C_PBIT_BOOL] = current[COL_PBIT] * (current[COL_PBIT] - E::ONE);
        result[C_LIMIT_CONST] = next[COL_LIMIT] - current[COL_LIMIT];

        let expected = [
            current[COL_BAL],
            current[COL_AMT],
            current[COL_BAL_NEW],
            current[COL_SUPPLY_NEW],
            current[COL_LIMIT] - current[COL_AMT],
        ];
        for seg in 0..NUM_SEGMENTS {
            result[C_SEG_LINK + seg] = periodic[P_SEG_LINK + seg] * (sacc_next - expected[seg]);
        }
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        let zero = BaseElement::ZERO;
        let mut a = Vec::with_capacity(42);

        for i in 0..4 {
            a.push(Assertion::single(i, 0, zero));
            a.push(Assertion::single(LANE_B + i, 0, zero));
        }
        for i in 9..12 {
            a.push(Assertion::single(i, 0, zero));
            a.push(Assertion::single(LANE_B + i, 0, zero));
        }
        for i in 0..4 {
            a.push(Assertion::single(4 + i, ROW_ROOT, self.pub_inputs.root_old[i]));
            a.push(Assertion::single(
                LANE_B + 4 + i,
                ROW_ROOT,
                self.pub_inputs.root_new[i],
            ));
        }
        a.push(Assertion::single(
            4,
            ROW_PK_START,
            BaseElement::new(SPEND_KEY_DOMAIN),
        ));
        for i in 5..8 {
            a.push(Assertion::single(i, ROW_PK_START, zero));
        }
        a.push(Assertion::single(COL_AMT, 0, self.pub_inputs.amount));
        a.push(Assertion::single(
            COL_LIMIT,
            0,
            self.pub_inputs.regulatory_limit,
        ));
        a.push(Assertion::single(
            COL_SUPPLY_OLD,
            0,
            self.pub_inputs.supply_old,
        ));
        a.push(Assertion::single(
            COL_SUPPLY_NEW,
            0,
            self.pub_inputs.supply_new,
        ));

        // La raiz de congelados: el titular no esta en ese arbol.
        for i in 0..4 {
            a.push(Assertion::single(
                4 + i,
                ROW_FROZEN_ROOT,
                self.pub_inputs.frozen_root[i],
            ));
        }

        // Las raices del arbol de pendientes: antes libre, despues con
        // el compromiso C2.
        for i in 0..4 {
            a.push(Assertion::single(
                4 + i,
                ROW_PENDING_ROOT,
                self.pub_inputs.pending_root_old[i],
            ));
            a.push(Assertion::single(
                LANE_B + 4 + i,
                ROW_PENDING_ROOT,
                self.pub_inputs.pending_root_new[i],
            ));
        }

        a
    }
}

pub struct SendV2Prover {
    options: ProofOptions,
}

impl SendV2Prover {
    pub fn new(options: ProofOptions) -> Self {
        Self { options }
    }
}

impl Prover for SendV2Prover {
    type BaseField = BaseElement;
    type Air = SendV2Air;
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

    fn get_pub_inputs(&self, trace: &Self::Trace) -> SendPublicInputs {
        SendPublicInputs {
            root_old: [
                trace.get(4, ROW_ROOT),
                trace.get(5, ROW_ROOT),
                trace.get(6, ROW_ROOT),
                trace.get(7, ROW_ROOT),
            ],
            root_new: [
                trace.get(LANE_B + 4, ROW_ROOT),
                trace.get(LANE_B + 5, ROW_ROOT),
                trace.get(LANE_B + 6, ROW_ROOT),
                trace.get(LANE_B + 7, ROW_ROOT),
            ],
            frozen_root: [
                trace.get(4, ROW_FROZEN_ROOT),
                trace.get(5, ROW_FROZEN_ROOT),
                trace.get(6, ROW_FROZEN_ROOT),
                trace.get(7, ROW_FROZEN_ROOT),
            ],
            pending_root_old: [
                trace.get(4, ROW_PENDING_ROOT),
                trace.get(5, ROW_PENDING_ROOT),
                trace.get(6, ROW_PENDING_ROOT),
                trace.get(7, ROW_PENDING_ROOT),
            ],
            pending_root_new: [
                trace.get(LANE_B + 4, ROW_PENDING_ROOT),
                trace.get(LANE_B + 5, ROW_PENDING_ROOT),
                trace.get(LANE_B + 6, ROW_PENDING_ROOT),
                trace.get(LANE_B + 7, ROW_PENDING_ROOT),
            ],
            amount: trace.get(COL_AMT, 0),
            regulatory_limit: trace.get(COL_LIMIT, 0),
            supply_old: trace.get(COL_SUPPLY_OLD, 0),
            supply_new: trace.get(COL_SUPPLY_NEW, 0),
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
    use crate::native::{
        derive_leaf_salt_wide, derive_public_id, derive_public_id_wide, native_climb,
        native_leaf_salted,
    };
    use winterfell::{verify, AcceptableOptions, BatchingMethod, FieldExtension};

    const SK: u64 = 0xA11CE;

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

    struct Scenario {
        key: Digest,
        account_id: Digest,
        balance: u64,
        nonce: BaseElement,
        leaf_salt: Digest,
        path: MerklePath,
        frozen_path: MerklePath,
        pending_path: MerklePath,
        receiver_id: Digest,
        salt: Digest,
        sobre: Digest,
        amount: u64,
        supply_old: u64,
        public_inputs: SendPublicInputs,
    }

    /// Limite holgado a proposito: estos tests comprueban otras cosas.
    const TEST_LIMIT: u64 = 10_000_000;

    fn scenario(balance: u64, amount: u64, supply_old: u64) -> Scenario {
        let mut empty = vec![[BaseElement::ZERO; 4]];
        for k in 1..=TREE_DEPTH {
            let prev = empty[k - 1];
            empty.push(native_merge(prev, prev));
        }
        // Ancha de verdad, no as_digest(x) (S90.3).
        let key = [
            BaseElement::new(SK),
            BaseElement::new(0x5E4D),
            BaseElement::new(0x0DDBA11),
            BaseElement::new(0x5EA51DE),
        ];
        let account_id = derive_public_id_wide(key);
        let nonce = BaseElement::ZERO;

        // Direcciones MIXTAS: con todas iguales la traza degenera.
        let mut siblings = Vec::with_capacity(TREE_DEPTH);
        let mut is_right = Vec::with_capacity(TREE_DEPTH);
        for level in 0..TREE_DEPTH {
            siblings.push(empty[level]);
            is_right.push(level % 3 == 0);
        }
        let path = MerklePath { siblings, is_right };

        // El salt REAL del titular (S117): derivado de la clave.
        let leaf_salt = derive_leaf_salt_wide(key);
        let leaf_old =
            native_leaf_salted(account_id, BaseElement::new(balance), nonce, leaf_salt);
        let leaf_new = native_leaf_salted(
            account_id,
            BaseElement::new(balance) - BaseElement::new(amount),
            nonce,
            leaf_salt,
        );

        // Arbol de congelados, con la cuenta LIBRE.
        let mut f_empty = vec![[BaseElement::ZERO; 4]];
        for k in 1..=FROZEN_DEPTH {
            let prev = f_empty[k - 1];
            f_empty.push(native_merge(prev, prev));
        }
        let frozen_path = MerklePath {
            siblings: (0..FROZEN_DEPTH).map(|l| f_empty[l]).collect(),
            is_right: (0..FROZEN_DEPTH).map(|l| l % 3 == 0).collect(),
        };
        let frozen_root = crate::circuit_freeze::frozen_climb([BaseElement::ZERO; 4], &frozen_path);

        // Arbol de PENDIENTES, con la posicion libre.
        let pending_path = MerklePath {
            siblings: (0..TREE_DEPTH).map(|l| empty[l]).collect(),
            is_right: (0..TREE_DEPTH).map(|l| l % 4 == 0).collect(),
        };
        let receiver_id = derive_public_id(BaseElement::new(0xB0B));
        let salt: Digest = [
            BaseElement::new(0x5EED_0001),
            BaseElement::new(0x5EED_0002),
            BaseElement::new(0x5EED_0003),
            BaseElement::new(0x5EED_0004),
        ];
        // El sobre X, opaco: el pagador lo compone fuera del circuito.
        let sobre: Digest = [
            BaseElement::new(0xE3C1_0001),
            BaseElement::new(0xE3C1_0002),
            BaseElement::new(0xE3C1_0003),
            BaseElement::new(0xE3C1_0004),
        ];
        // El compromiso v2, calculado de forma nativa para comparar:
        // C2 = M( M( M(id, salt), importe ), X ).
        let c1 = native_merge(
            native_merge(receiver_id, salt),
            [BaseElement::new(amount), BaseElement::ZERO, BaseElement::ZERO, BaseElement::ZERO],
        );
        let pending = native_merge(c1, sobre);
        let climb_pending = |hoja: Digest| {
            let mut cur = hoja;
            for level in 0..TREE_DEPTH {
                cur = if pending_path.is_right[level] {
                    native_merge(pending_path.siblings[level], cur)
                } else {
                    native_merge(cur, pending_path.siblings[level])
                };
            }
            cur
        };

        Scenario {
            public_inputs: SendPublicInputs {
                regulatory_limit: BaseElement::new(TEST_LIMIT),
                root_old: native_climb(leaf_old, &path),
                root_new: native_climb(leaf_new, &path),
                amount: BaseElement::new(amount),
                supply_old: BaseElement::new(supply_old),
                // UN ENVIO NO CAMBIA EL SUMINISTRO (doctrina del v1).
                supply_new: BaseElement::new(supply_old),
                frozen_root,
                pending_root_old: climb_pending([BaseElement::ZERO; 4]),
                pending_root_new: climb_pending(pending),
            },
            key,
            account_id,
            balance,
            nonce,
            leaf_salt,
            path,
            frozen_path,
            pending_path,
            receiver_id,
            salt,
            sobre,
            amount,
            supply_old,
        }
    }

    fn run(s: &Scenario, key: Digest, supply_delta: u64) -> Result<(), String> {
        let trace = build_trace(
            key,
            s.account_id,
            s.balance,
            s.nonce,
            s.leaf_salt,
            &s.path,
            &s.frozen_path,
            s.amount,
            TEST_LIMIT,
            s.supply_old,
            supply_delta,
            s.receiver_id,
            s.salt,
            s.sobre,
            &s.pending_path,
        );
        let prover = SendV2Prover::new(default_options());

        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| prover.prove(trace)));

        let proof = match r {
            // El mensaje del panico se conserva (doctrina del v1).
            Err(e) => {
                let msg = e
                    .downcast_ref::<String>()
                    .cloned()
                    .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
                    .unwrap_or_else(|| "panico sin mensaje".into());
                return Err(format!("prove hizo panic: {msg}"));
            }
            Ok(Err(e)) => return Err(format!("prove Err: {e:?}")),
            Ok(Ok(p)) => p,
        };

        let min_opts = AcceptableOptions::OptionSet(vec![default_options()]);
        verify::<SendV2Air, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof,
            s.public_inputs.clone(),
            &min_opts,
        )
        .map_err(|e| format!("verificacion fallo: {e:?}"))
    }

    /// Un envio v2 honesto prueba y verifica.
    #[test]
    fn un_envio_v2_verifica() {
        let s = scenario(1_000_000, 250_000, 10_000_000);
        let r = run(&s, s.key, 0);
        assert!(r.is_ok(), "el envio v2 honesto debe verificar: {r:?}");
    }

    /// Los hitos de la traza espejan el calculo nativo: C1 en la fila
    /// del cuarto merge, el sobre en COL_X donde el enlace lo lee, C2 a
    /// la entrada del arbol, y las raices declaradas.
    #[test]
    fn los_hitos_de_la_traza_espejan_el_nativo() {
        let s = scenario(1_000_000, 250_000, 10_000_000);
        let trace = build_trace(
            s.key, s.account_id, s.balance, s.nonce, s.leaf_salt, &s.path, &s.frozen_path,
            s.amount, TEST_LIMIT, s.supply_old, 0, s.receiver_id, s.salt, s.sobre,
            &s.pending_path,
        );

        let c1 = native_merge(
            native_merge(s.receiver_id, s.salt),
            [BaseElement::new(s.amount), BaseElement::ZERO, BaseElement::ZERO, BaseElement::ZERO],
        );
        let c2 = native_merge(c1, s.sobre);
        for i in 0..4 {
            assert_eq!(
                trace.get(4 + i, ROW_PEND_ENV),
                c1[i],
                "C1 disponible en la fila del cuarto merge, elemento {i}"
            );
            assert_eq!(
                trace.get(COL_X + i, ROW_PEND_ENV),
                s.sobre[i],
                "el sobre en COL_X donde el enlace lo lee, elemento {i}"
            );
            assert_eq!(
                trace.get(4 + i, ROW_PENDING_ENTRY),
                c2[i],
                "C2 a la entrada del arbol, elemento {i}"
            );
            assert_eq!(
                trace.get(4 + i, ROW_PENDING_ROOT),
                s.public_inputs.pending_root_old[i],
                "raiz de pendientes ANTES, elemento {i}"
            );
            assert_eq!(
                trace.get(LANE_B + 4 + i, ROW_PENDING_ROOT),
                s.public_inputs.pending_root_new[i],
                "raiz de pendientes DESPUES (con C2), elemento {i}"
            );
        }
    }

    /// MUTACION (a): un limbo del sobre alterado en la fila donde el
    /// enlace lo LEE debe rechazarse (C_ENV_IN dispara). Veneno =
    /// honesto + 1: distinto por construccion.
    #[test]
    fn mutacion_a_un_limbo_del_sobre_alterado_se_rechaza() {
        let s = scenario(1_000_000, 250_000, 10_000_000);
        let mut trace = build_trace(
            s.key, s.account_id, s.balance, s.nonce, s.leaf_salt, &s.path, &s.frozen_path,
            s.amount, TEST_LIMIT, s.supply_old, 0, s.receiver_id, s.salt, s.sobre,
            &s.pending_path,
        );

        let veneno = trace.get(COL_X + 2, ROW_PEND_ENV) + BaseElement::ONE;
        trace.set(COL_X + 2, ROW_PEND_ENV, veneno);

        let prover = SendV2Prover::new(default_options());
        let verifica = {
            let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                || prover.prove(trace)));
            match r {
                Err(_) => false,        // panic al generar -> no verifica
                Ok(Err(_)) => false,    // prove Err
                Ok(Ok(proof)) => {
                    let min_opts = AcceptableOptions::OptionSet(vec![default_options()]);
                    verify::<SendV2Air, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
                        proof, s.public_inputs.clone(), &min_opts,
                    ).is_ok()
                }
            }
        };
        assert!(
            !verifica,
            "un limbo del sobre alterado DEBE rechazar (C_ENV_IN); si \
             verifica, el sobre no envuelve nada"
        );
    }

    /// MUTACION (b): C1 DESNUDO a la entrada del arbol - el cuarto
    /// merge omitido. Se escriben ambas mitades de la lane B del estado
    /// siguiente para alcanzar al gate sea cual sea el bit; la entrada
    /// dispara sobre la mitad colocada, o la raiz declarada rechaza.
    #[test]
    fn mutacion_b_el_c1_desnudo_no_entra_al_arbol() {
        let s = scenario(1_000_000, 250_000, 10_000_000);
        let mut trace = build_trace(
            s.key, s.account_id, s.balance, s.nonce, s.leaf_salt, &s.path, &s.frozen_path,
            s.amount, TEST_LIMIT, s.supply_old, 0, s.receiver_id, s.salt, s.sobre,
            &s.pending_path,
        );

        let c1 = native_merge(
            native_merge(s.receiver_id, s.salt),
            [BaseElement::new(s.amount), BaseElement::ZERO, BaseElement::ZERO, BaseElement::ZERO],
        );
        for i in 0..4 {
            trace.set(LANE_B + 4 + i, ROW_PENDING_ENTRY + 1, c1[i]);
            trace.set(LANE_B + 8 + i, ROW_PENDING_ENTRY + 1, c1[i]);
        }

        let prover = SendV2Prover::new(default_options());
        let verifica = {
            let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                || prover.prove(trace)));
            match r {
                Err(_) => false,        // panic al generar -> no verifica
                Ok(Err(_)) => false,    // prove Err
                Ok(Ok(proof)) => {
                    let min_opts = AcceptableOptions::OptionSet(vec![default_options()]);
                    verify::<SendV2Air, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
                        proof, s.public_inputs.clone(), &min_opts,
                    ).is_ok()
                }
            }
        };
        assert!(
            !verifica,
            "C1 sin envolver NO debe entrar al arbol de pendientes; si \
             verifica, el cuarto merge es decorativo"
        );
    }

    /// Un importe distinto del declarado no verifica: el compromiso y
    /// el debito cambian con el, y las entradas publicas declaradas ya
    /// no corresponden.
    #[test]
    fn un_importe_distinto_no_verifica() {
        let s = scenario(1_000_000, 250_000, 10_000_000);
        let trace = build_trace(
            s.key, s.account_id, s.balance, s.nonce, s.leaf_salt, &s.path, &s.frozen_path,
            s.amount * 2, TEST_LIMIT, s.supply_old, 0, s.receiver_id, s.salt, s.sobre,
            &s.pending_path,
        );
        let prover = SendV2Prover::new(default_options());
        let proof = prover.prove(trace).expect("la traza es coherente consigo misma");
        let min_opts = AcceptableOptions::OptionSet(vec![default_options()]);
        let v = verify::<SendV2Air, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof,
            s.public_inputs.clone(),
            &min_opts,
        );
        assert!(
            v.is_err(),
            "un importe distinto del declarado NO debe verificar"
        );
    }

    /// DOMINIO: un arbol cuya raiz nueva se compuso con C1 (la forma
    /// v1, sin sobre) no se prueba con el circuito v2 - este SIEMPRE
    /// envuelve, y el legado es inmune por dominio (RFC-0003).
    #[test]
    fn un_arbol_con_c1_no_se_prueba_con_el_v2() {
        let s = scenario(1_000_000, 250_000, 10_000_000);

        let c1 = native_merge(
            native_merge(s.receiver_id, s.salt),
            [BaseElement::new(s.amount), BaseElement::ZERO, BaseElement::ZERO, BaseElement::ZERO],
        );
        let climb = |hoja: Digest| {
            let mut cur = hoja;
            for level in 0..TREE_DEPTH {
                cur = if s.pending_path.is_right[level] {
                    native_merge(s.pending_path.siblings[level], cur)
                } else {
                    native_merge(cur, s.pending_path.siblings[level])
                };
            }
            cur
        };
        let mut declaradas = s.public_inputs.clone();
        declaradas.pending_root_new = climb(c1);

        let trace = build_trace(
            s.key, s.account_id, s.balance, s.nonce, s.leaf_salt, &s.path, &s.frozen_path,
            s.amount, TEST_LIMIT, s.supply_old, 0, s.receiver_id, s.salt, s.sobre,
            &s.pending_path,
        );
        let prover = SendV2Prover::new(default_options());
        let proof = prover.prove(trace).expect("la traza honesta prueba");
        let min_opts = AcceptableOptions::OptionSet(vec![default_options()]);
        let v = verify::<SendV2Air, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof, declaradas, &min_opts,
        );
        assert!(
            v.is_err(),
            "una raiz compuesta con C1 desnudo (forma v1) NO debe \
             verificar con el circuito v2"
        );
    }
}
