//! ⚠️ **ANDAMIO B13/B14 — GEMELO del mundo nuevo (paso 3, §144).**
//!
//! Copia declarada de `circuit_claim` donde vive su migración — hoja
//! envuelta (`native_leaf_salted`), frozen-32 local y las mutaciones
//! obligatorias — por el playbook de la réplica
//! (`doc/playbook-replica-gemelos.md`, destilado del piloto §140-§143):
//! gemelo-primero, SB0 interno, el legacy no se toca.
//!
//! **Cláusula de retirada**: en el flip (release única, spec de
//! migración D4) este módulo SUSTITUYE a `circuit_claim` y el legacy se
//! borra. Hasta entonces, nadie fuera de los tests de este crate lo
//! importa.
//!
//! ---
//!
//! **Reclamación**: segunda fase de una transferencia en dos pasos.
//!
//! El pagador creó un compromiso pendiente atado a la identidad del
//! receptor con `circuit_send`. Aquí el receptor **demuestra que es suyo**
//! y lo cobra.
//!
//! ## Qué demuestra
//!
//! 1. **El pendiente es suyo**: reconstruido con SU identidad, derivada de
//!    su clave de gasto, da el compromiso que está en el árbol.
//! 2. **Existe**: pertenencia al árbol de pendientes.
//! 3. **No está congelado.**
//! 4. **Su saldo sube exactamente el importe.**
//! 5. **El pendiente queda consumido**: sale del árbol.
//!
//! ## Las dos inversiones respecto al envío
//!
//! | | Envío | Reclamación |
//! |---|---|---|
//! | El saldo | **Baja** | **Sube** |
//! | El pendiente | Entra: carril A cero, B compromiso | **Sale**: carril A compromiso, B cero |
//!
//! ## Por qué el punto 5 va en el circuito
//!
//! Si lo marcara solo la capa, un segundo intento encontraría el pendiente
//! todavía ahí y **el mismo pago se cobraría indefinidamente**.
//!
//! ## ⚠️ Lo que NO resuelve
//!
//! - **Si el receptor nunca reclama, el dinero queda inmovilizado.**
//!   Haría falta devolución al pagador tras un plazo, y esta capa **no
//!   tiene noción de tiempo**.
//! - **El pagador reconoce cuándo se reclama**, porque eligió el
//!   aleatorio. Sabe cuándo cobras, no cuánto tienes.

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

use crate::circuit_settlement::SPEND_KEY_DOMAIN;
use crate::circuit_freeze::FROZEN_DEPTH;
use crate::merkle::{Digest, MerklePath, TREE_DEPTH};
use crate::rescue_hash::{apply_sbox, NUM_ROUNDS, STATE_WIDTH};

pub const CYCLE_LENGTH: usize = 8;
/// 1024 filas. La tubería completa acaba en `ROW_PENDING_ROOT` (fila
/// 743): quedan **280 filas de holgura** (35 ciclos) — misma geometría
/// que send legacy: `doc/mapa-geometria-circuit_send.md` §6-§7 aplica.
pub const TRACE_LENGTH: usize = 1024;
pub const SEGMENT_LENGTH: usize = 64;
/// Segmentos: saldo, importe, saldo nuevo, suministro nuevo.
pub const NUM_SEGMENTS: usize = 4;

// ===== Columnas =====
const LANE_B: usize = STATE_WIDTH;
const COL_BIT: usize = 24;
/// Clave de gasto del TITULAR.
/// Clave de gasto. ⚠️ **CUATRO elementos** desde §90 (entrada 15).
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
/// Bit de dirección del camino en el árbol de congelados.
const COL_FBIT: usize = 41;
/// Bit de dirección en el árbol de PENDIENTES.
const COL_PBIT: usize = 42;
/// **Identidad pública del receptor**, que funciona como dirección.
/// Privada: un tercero no debe saber a quién se paga.
const COL_R_ID: usize = 43; // 43..47
/// Aleatorio que ciega el compromiso. Lo elige el pagador.
const COL_SALT: usize = 47; // 47..51
pub const TRACE_WIDTH: usize = 51;

// ===== Filas =====
//
// Geometría derivada (playbook R2; el patrón de SB0, §140-§141): cada
// tramo arranca en un ciclo `CYC_*` y las filas-hito `ROW_*` se derivan
// de él — una sola fuente de verdad para el calendario.
//
// Convención: todo arranque de tramo es un `CYC_*`; ningún literal de
// ciclo vive fuera de este bloque — bucles de bits, periódicas y el
// `match` de `build_trace` lo derivan de aquí.
const CYC_NONCE: usize = 1;
const CYC_ACC: usize = CYC_NONCE + 1;
const CYC_PK: usize = CYC_ACC + TREE_DEPTH;
const CYC_FROZEN: usize = CYC_PK + 1;
const CYC_PEND_IN: usize = CYC_FROZEN + FROZEN_DEPTH;
const CYC_PEND_VAL: usize = CYC_PEND_IN + 1;
const CYC_PEND_CLIMB: usize = CYC_PEND_VAL + 1;
const CYC_FIN: usize = CYC_PEND_CLIMB + TREE_DEPTH;
const ROW_LEAF_LINK: usize = CYC_NONCE * CYCLE_LENGTH - 1;
const ROW_LEAF_DONE: usize = CYC_ACC * CYCLE_LENGTH - 1;
const ROW_ROOT: usize = CYC_PK * CYCLE_LENGTH - 1;
const ROW_PK_START: usize = CYC_PK * CYCLE_LENGTH;
const ROW_PK_DONE: usize = CYC_FROZEN * CYCLE_LENGTH - 1;
/// **Fase de no-pertenencia al árbol de CONGELADOS.**
///
/// Ocupa las filas 280..471, que estaban libres. Sin ella, una cuenta
/// congelada **podía destruir su dinero**: la liquidación comprobaba la
/// congelación y la destrucción no.
///
/// Congelar existe para que una cuenta bajo investigación no mueva fondos.
/// Destruirlos los mueve: los saca del sistema. Que sea público e
/// irreversible no los devuelve.
const ROW_FROZEN_ROOT: usize = CYC_PEND_IN * CYCLE_LENGTH - 1;
/// **Inserción del pendiente**: ciclos 60..91, filas 480..735.
///
/// Carril A: la posición vacía → raíz antigua de pendientes.
/// Carril B: con el compromiso → raíz nueva.
/// Compromiso interno del pendiente: `H(id_receptor, aleatorio)`.
const ROW_PEND_INNER: usize = CYC_PEND_VAL * CYCLE_LENGTH - 1;
/// El pendiente completo: `H(interno, importe)`.
const ROW_PENDING_ENTRY: usize = CYC_PEND_CLIMB * CYCLE_LENGTH - 1;
/// Raíz tras insertarlo. Ciclos 61..92, filas 488..743.
const ROW_PENDING_ROOT: usize = CYC_FIN * CYCLE_LENGTH - 1;

// El presupuesto, en compilación: la tubería debe caber en la traza.
// Con el salt y frozen-32 (B13/B14), esto es lo que avisará si
// `TRACE_LENGTH = 1024` se queda corto.
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
/// **TITULARIDAD**: la identidad derivada coincide con la de la cuenta.
const C_PK_CHECK: usize = C_KEY_INPUT + 8; // 4
/// **EL SALDO DISMINUYE EXACTAMENTE EN EL IMPORTE.**
const C_BALANCE: usize = C_PK_CHECK + 4; // 1
/// **EL SUMINISTRO DISMINUYE EXACTAMENTE EN EL IMPORTE.**
const C_SUPPLY: usize = C_BALANCE + 1; // 1
const C_TRANSPORT: usize = C_SUPPLY + 1; // 18 (10 + id receptor 4 + aleatorio 4)
// ⚠️ ENTRADA 36 / §50.7: era `C_TRANSPORT + 7`, mismo solapamiento que send.
// Aunque COL_R_ID esta muerto tras §39.1, el compromiso lee COL_SALT y su
// constancia estaba pisada -> fallo de solidez. Ahora C_TRANSPORT ocupa 15.
const C_ID_CONST: usize = C_TRANSPORT + 18; // 4
const C_SBIT_BOOL: usize = C_ID_CONST + 4; // 2
const C_FIRST_S: usize = C_SBIT_BOOL + 2; // 2
const C_HORNER: usize = C_FIRST_S + 2; // 1
const C_SEG_LINK: usize = C_HORNER + 1;
/// Capacidad a cero en la fase de congelados.
const C_FROZEN_CAP: usize = C_SEG_LINK + NUM_SEGMENTS; // 4
/// **LA NO-PERTENENCIA.** La hoja colocada debe ser CERO.
const C_FROZEN_ENTRY: usize = C_FROZEN_CAP + 4; // 4
/// Colocación en cada nivel.
const C_FROZEN_PLACE: usize = C_FROZEN_ENTRY + 4; // 4
const C_FBIT_BOOL: usize = C_FROZEN_PLACE + 4; // 1
/// El compromiso interno entra con la identidad del receptor y el
/// aleatorio.
const C_PEND_IN: usize = C_FBIT_BOOL + 1; // 12 (capacidad 4 + identidad 4 + aleatorio 4)
/// Y el compromiso completo, con el importe.
const C_PEND_VAL: usize = C_PEND_IN + 12; // 5 (digest 4 + importe 1)
/// Capacidad a cero en la subida al árbol de pendientes.
const C_PEND_CAP: usize = C_PEND_VAL + 5; // 8
/// **LA ENTRADA**: el carril A coloca CERO —la posición estaba libre— y
/// el B coloca el compromiso.
const C_PEND_ENTRY_A: usize = C_PEND_CAP + 8; // 4
const C_PEND_ENTRY_B: usize = C_PEND_ENTRY_A + 4; // 4
/// Colocación en cada nivel y hermano compartido.
const C_PEND_PLACE: usize = C_PEND_ENTRY_B + 4; // 8
const C_PEND_SIBLING: usize = C_PEND_PLACE + 8; // 4
const C_PBIT_BOOL: usize = C_PEND_SIBLING + 4; // 1
const NUM_CONSTRAINTS: usize = C_PBIT_BOOL + 1;

// ===== Periódicas =====
const P_HASH_FLAG: usize = 0;
const P_ARK1: usize = 1;
const P_ARK2: usize = P_ARK1 + STATE_WIDTH;
const P_LINK_MERKLE: usize = P_ARK2 + STATE_WIDTH;
const P_LINK_LEAF: usize = P_LINK_MERKLE + 1;
const P_LINK_PLACE: usize = P_LINK_LEAF + 1;
const P_FIRST_ROW: usize = P_LINK_PLACE + 1;
const P_SEL_ROOT: usize = P_FIRST_ROW + 1;
const P_SEL_PK_DONE: usize = P_SEL_ROOT + 1;
const P_FIRST_S: usize = P_SEL_PK_DONE + 1;
const P_CONT_S: usize = P_FIRST_S + 1;
const P_SEG_LINK: usize = P_CONT_S + 1;
/// Fila que entra al árbol de congelados.
const P_FROZEN_ENTRY: usize = P_SEG_LINK + NUM_SEGMENTS;
/// Enlaces de la subida.
const P_FROZEN_LINK: usize = P_FROZEN_ENTRY + 1;
/// Fila del compromiso interno del pendiente.
const P_PEND_IN: usize = P_FROZEN_LINK + 1;
/// Fila del compromiso completo.
const P_PEND_VAL: usize = P_PEND_IN + 1;
/// Fila que entra al árbol de pendientes.
const P_PEND_ENTRY: usize = P_PEND_VAL + 1;
/// Enlaces de la subida.
const P_PEND_LINK: usize = P_PEND_ENTRY + 1;

type Blake3 = Blake3_256<BaseElement>;

fn value_to_bits_be(value: u64) -> Vec<bool> {
    (0..SEGMENT_LENGTH)
        .map(|p| (value >> (SEGMENT_LENGTH - 1 - p)) & 1 == 1)
        .collect()
}

/// Construye la traza de una destrucción.
///
/// `supply_delta` permite reducir el suministro en una cantidad distinta
/// de la destruida, para el test de destrucción encubierta.
#[allow(clippy::too_many_arguments)]
pub fn build_trace(
    // ⚠️ `spend_key` son **CUATRO elementos** desde §90 (entrada 15).
    spend_key: Digest,
    account_id: Digest,
    balance: u64,
    nonce: BaseElement,
    path: &MerklePath,
    frozen_path: &MerklePath,
    amount: u64,
    supply_old: u64,
    // Debe ser CERO: un envío no cambia el suministro. Se mantiene como
    // parámetro para que un test pueda intentar lo contrario.
    supply_delta: u64,
    // **Identidad pública del receptor.** Funciona como dirección: el
    // pagador la obtiene del propio receptor, **no del operador**.
    //
    // Con ella construye el pendiente
    // `H(H(id_receptor, aleatorio), importe)`. **No necesita su saldo ni
    // su nonce**, que es lo que cierra la fuga.
    receiver_id: Digest,
    // Aleatorio que ciega el compromiso. Lo elige el pagador.
    salt: Digest,
    // Camino de la posición libre donde se inserta el pendiente.
    pending_path: &MerklePath,
) -> TraceTable<BaseElement> {
    let zero = BaseElement::ZERO;
    let c_bal = BaseElement::new(balance);
    let c_amt = BaseElement::new(amount);
    // **EL SALDO SUBE.** Al reves que en el envio: el dinero vuelve del
    // pendiente a la cuenta del receptor.
    let c_bal_new = c_bal + BaseElement::new(amount);
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
        row[COL_R_ID..COL_R_ID + 4].copy_from_slice(&receiver_id);
        row[COL_SALT..COL_SALT + 4].copy_from_slice(&salt);
        row[COL_SUPPLY_OLD] = c_supply_old;
        row[COL_SUPPLY_NEW] = c_supply_new;
    }

    // Rangos. `c_bal_new` demuestra que no se destruye mas de lo que hay:
    // si el importe superara el saldo, la resta daria la vuelta en el
    // campo y no cabria en el rango.
    let segment_values = [
        c_bal.as_int(),
        c_amt.as_int(),
        c_bal_new.as_int(),
        c_supply_new.as_int(),
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

    let place_pending = |state: &mut [BaseElement; STATE_WIDTH], digest: &Digest, level: usize| {
        debug_assert!(
            level < TREE_DEPTH,
            "place_pending: nivel {} sobre path de {}",
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
            "place_frozen: nivel {} sobre path de {}",
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
            "place: nivel {} sobre path de {}",
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
                    // El nonce NO cambia: destruir no consume el derecho
                    // de gasto, igual que emitir no lo consumía.
                    state_a[4..8].copy_from_slice(&digest_a);
                    state_a[8] = nonce;
                    state_b[4..8].copy_from_slice(&digest_b);
                    state_b[8] = nonce;
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
                    // EL PENDIENTE: H(interno, importe).
                    state_a[4..8].copy_from_slice(&digest_a);
                    state_a[8] = c_amt;
                    state_a[9] = zero;
                    state_a[10] = zero;
                    state_a[11] = zero;
                    state_b.copy_from_slice(&state_a);
                }
                ROW_PENDING_ENTRY => {
                    // ENTRADA AL ARBOL DE PENDIENTES.
                    //
                    // Carril A: hoja CERO -> la posicion estaba libre.
                    // Carril B: el compromiso -> raiz nueva.
                    // SALIDA DEL ARBOL DE PENDIENTES.
                    //
                    // Carril A: el compromiso -> raiz ANTES, y con ella la
                    // prueba de que el pendiente existia.
                    // Carril B: cero -> raiz DESPUES, ya consumido.
                    let libre: Digest = [zero; 4];
                    place_pending(&mut state_a, &digest_a, 0);
                    place_pending(&mut state_b, &libre, 0);
                }
                ROW_PK_DONE => {
                    // ENTRADA AL ARBOL DE CONGELADOS.
                    //
                    // Hoja CERO en la posicion del titular: si estuviera
                    // congelado, su hoja no seria cero y la subida no
                    // llegaria a la raiz declarada.
                    let libre: Digest = [zero; 4];
                    place_frozen(&mut state_a, &libre, 0);
                    place_frozen(&mut state_b, &libre, 0);
                }
                _ => {
                    let next_cycle = (r + 1) / CYCLE_LENGTH;
                    // Convención única (playbook R2): cada tramo genérico
                    // es `(CYC_arranque..CYC_fin_de_tramo)` y el nivel es
                    // `next_cycle - CYC_arranque`. El arranque lo sombrea
                    // su brazo explícito (que coloca el nivel 0); el final
                    // queda FUERA del rango: la raíz no se coloca, la atan
                    // las aserciones.
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

/// Inputs públicos.
///
/// **La identidad de la cuenta NO es pública**: una destrucción no queda
/// vinculada a ninguna cuenta concreta desde fuera. Lo que se publica es
/// que *alguien con autoridad* destruyó ese importe y que el suministro
/// bajó en consecuencia.
#[derive(Clone, Debug)]
pub struct ClaimPublicInputs {
    pub root_old: Digest,
    pub root_new: Digest,
    /// **Raíz del árbol de congelados.** La prueba acredita que el titular
    /// NO está en él: una cuenta congelada no puede destruir su dinero.
    pub frozen_root: Digest,
    /// **Árbol de pendientes ANTES.** La posición donde se inserta estaba
    /// libre: ahí reside la prueba de que no se pisa otro pendiente.
    pub pending_root_old: Digest,
    /// **Y DESPUÉS**, con el compromiso dentro.
    pub pending_root_new: Digest,
    pub amount: BaseElement,
    pub supply_old: BaseElement,
    pub supply_new: BaseElement,
}

impl ToElements<BaseElement> for ClaimPublicInputs {
    fn to_elements(&self) -> Vec<BaseElement> {
        let mut out = self.root_old.to_vec();
        out.extend_from_slice(&self.root_new);
        out.extend_from_slice(&self.frozen_root);
        out.extend_from_slice(&self.pending_root_old);
        out.extend_from_slice(&self.pending_root_new);
        out.push(self.amount);
        out.push(self.supply_old);
        out.push(self.supply_new);
        out
    }
}

pub struct ClaimAir {
    context: AirContext<BaseElement>,
    pub_inputs: ClaimPublicInputs,
}

impl Air for ClaimAir {
    type BaseField = BaseElement;
    type PublicInputs = ClaimPublicInputs;

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
        // Enlaces de hoja (16), nonce (2), entradas (10), clave (2),
        // titularidad (4) = 34, grado 1 con ciclo.
        // ⚠️ Eran 34: la clave paso de 2 ranuras a 8 (§90).
        for _ in 0..40 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }
        // Saldo (1), suministro (1), transporte (15: 7 + identidad 4 +
        // aleatorio 4), identidad-constante (4) = 21, grado 1 sin ciclo.
        // ⚠️ ENTRADA 36 / §50.7: era 13; ver send §50.5.
        // ⚠️ Eran 21: `C_TRANSPORT` paso de 15 a 18 (§90).
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

        assert_eq!(degrees.len(), NUM_CONSTRAINTS, "cuenta de grados");

        ClaimAir {
            context: AirContext::new(trace_info, degrees, 41, options),
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

        // Compromiso completo.
        let mut pend_val = vec![zero; TRACE_LENGTH];
        pend_val[ROW_PEND_INNER] = one;
        columns.push(pend_val);

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

        for i in 0..4 {
            result[C_INPUT + i] = first_row * (current[4 + i] - current[COL_ACC_ID + i]);
            result[C_INPUT + 5 + i] =
                first_row * (current[LANE_B + 4 + i] - current[COL_ACC_ID + i]);
        }
        result[C_INPUT + 4] = first_row * (current[8] - current[COL_BAL]);
        result[C_INPUT + 9] = first_row * (current[LANE_B + 8] - current[COL_BAL_NEW]);

        // ⚠️ Los CUATRO elementos, en los dos carriles (§92.2).
        for i in 0..4 {
            result[C_KEY_INPUT + i] = sel_root * (next[8 + i] - current[COL_KEY + i]);
            result[C_KEY_INPUT + 4 + i] =
                sel_root * (next[LANE_B + 8 + i] - current[COL_KEY + i]);
        }

        // ===== TITULARIDAD =====
        // La identidad derivada de la clave es la de la cuenta. Sin la
        // clave del titular no se puede destruir su saldo.
        for i in 0..4 {
            result[C_PK_CHECK + i] = sel_pk_done * (current[4 + i] - current[COL_ACC_ID + i]);
        }
        // ===== EL SALDO SUBE EXACTAMENTE EL IMPORTE =====
        //
        // Al reves que en el envio. El dinero no se crea: sale de un
        // pendiente que este mismo circuito consume.
        result[C_BALANCE] = current[COL_BAL_NEW] - (current[COL_BAL] + current[COL_AMT]);
        // ===== UN ENVÍO NO CAMBIA EL SUMINISTRO =====
        //
        // Heredado del circuito de destrucción, donde el suministro bajaba.
        // Aquí **no debe moverse**: el dinero no se destruye, se traslada
        // a un pendiente que el receptor reclamará.
        //
        // No es maquinaria muerta: es la prueba de que una transferencia
        // **no crea ni destruye dinero**, verificable por cualquiera.
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
        for i in 0..4 {
            // ⚠️ **10 y 14, no 7 y 11**: el array `transport` paso de 7 a
            // 10 columnas. Es la TERCERA vez que estos desplazamientos dan
            // problemas —entrada 36, §50.7, y §92.12 en `send`, donde los
            // cazo `check_constraint_layout.py`—.
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
        //
        // Una cuenta congelada no puede destruir su dinero. Sin esto, la
        // liquidacion comprobaba la congelacion y la destruccion no: el
        // saldo investigado podia desaparecer.
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
        // Tres fases: el compromiso interno, el compromiso completo, y la
        // insercion en el arbol.
        //
        // Lo que cierra la fuga esta aqui: el compromiso se forma con la
        // IDENTIDAD del receptor, no con su saldo. El circuito no tiene
        // ninguna columna donde ese saldo pudiera entrar.
        let pend_in = periodic[P_PEND_IN];
        let pend_val = periodic[P_PEND_VAL];
        let pend_entry = periodic[P_PEND_ENTRY];
        let pend_link = periodic[P_PEND_LINK];
        let pbit = next[COL_PBIT];

        for i in 0..4 {
            // Compromiso interno: entra la identidad del receptor y el
            // aleatorio, con capacidad a cero.
            result[C_PEND_IN + i] = pend_in * next[i];
            // DOS restricciones separadas, no su suma.
            //
            // Una version anterior las sumaba: un probador podia poner la
            // identidad de mas y el aleatorio de menos, y la suma seguia
            // dando cero. **Sumar comprobaciones independientes las
            // anula.**
            // ⚠️ **LA IDENTIDAD ES LA DE LA CUENTA QUE COBRA**, no un
            // parametro aparte. Con `COL_R_ID` el circuito demostraba
            // «existe este pendiente» y «tengo esta clave» sin exigir que
            // fueran lo mismo: cualquiera con el aleatorio cobraba un
            // pendiente ajeno (`AUDITORIA.md` §39). `COL_ACC_ID` esta atado
            // a la clave por `C_PK_CHECK`, asi que aqui no cabe mentir.
            result[C_PEND_IN + 4 + i] = pend_in * (next[4 + i] - current[COL_ACC_ID + i]);
            result[C_PEND_IN + 8 + i] = pend_in * (next[8 + i] - current[COL_SALT + i]);

            // Compromiso completo: el digest interno, y el importe.
            result[C_PEND_VAL + i] = pend_val * (next[4 + i] - current[4 + i]);
        }
        result[C_PEND_VAL + 4] = pend_val * (next[8] - current[COL_AMT]);

        // Subida al arbol de pendientes.
        let pend_any = pend_entry + pend_link;
        for i in 0..4 {
            result[C_PEND_CAP + i] = pend_any * next[i];
            result[C_PEND_CAP + 4 + i] = pend_any * next[LANE_B + i];

            // **EL PENDIENTE EXISTIA**: el carril A entra con el
            // compromiso reconstruido con la identidad de quien reclama.
            // Si no fuera suyo, el compromiso seria otro y la subida no
            // llegaria a la raiz declarada.
            result[C_PEND_ENTRY_A + i] = pend_entry
                * ((E::ONE - pbit) * (next[4 + i] - current[4 + i])
                    + pbit * (next[8 + i] - current[4 + i]));
            // Y el B con CERO: queda consumido, y no puede cobrarse dos
            // veces.
            result[C_PEND_ENTRY_B + i] =
                pend_entry * ((E::ONE - pbit) * next[LANE_B + 4 + i] + pbit * next[LANE_B + 8 + i]);

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

        let expected = [
            current[COL_BAL],
            current[COL_AMT],
            current[COL_BAL_NEW],
            current[COL_SUPPLY_NEW],
        ];
        for seg in 0..NUM_SEGMENTS {
            result[C_SEG_LINK + seg] = periodic[P_SEG_LINK + seg] * (sacc_next - expected[seg]);
        }
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        let zero = BaseElement::ZERO;
        let mut a = Vec::with_capacity(29);

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
            COL_SUPPLY_OLD,
            0,
            self.pub_inputs.supply_old,
        ));
        a.push(Assertion::single(
            COL_SUPPLY_NEW,
            0,
            self.pub_inputs.supply_new,
        ));

        // **La raíz de congelados**: el titular no está en ese árbol.
        for i in 0..4 {
            a.push(Assertion::single(
                4 + i,
                ROW_FROZEN_ROOT,
                self.pub_inputs.frozen_root[i],
            ));
        }

        // **Las raíces del árbol de pendientes**: antes libre, después con
        // el compromiso.
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

pub struct ClaimProver {
    options: ProofOptions,
}

impl ClaimProver {
    pub fn new(options: ProofOptions) -> Self {
        Self { options }
    }
}

impl Prover for ClaimProver {
    type BaseField = BaseElement;
    type Air = ClaimAir;
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

    fn get_pub_inputs(&self, trace: &Self::Trace) -> ClaimPublicInputs {
        ClaimPublicInputs {
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
    use crate::circuit_settlement::{
        derive_public_id, derive_public_id_wide, native_climb, native_leaf,
    };
    use crate::merkle::native_merge;
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
        /// ⚠️ **CUATRO elementos** desde §90. Su tipo va primero (§92.7).
        key: Digest,
        account_id: Digest,
        balance: u64,
        nonce: BaseElement,
        path: MerklePath,
        frozen_path: MerklePath,
        pending_path: MerklePath,
        receiver_id: Digest,
        salt: Digest,
        amount: u64,
        supply_old: u64,
        public_inputs: ClaimPublicInputs,
    }

    /// Raíz del árbol de pendientes con la posición libre.
    fn climb_empty(path: &MerklePath) -> Digest {
        let mut cur = [BaseElement::ZERO; 4];
        for level in 0..TREE_DEPTH {
            cur = if path.is_right[level] {
                native_merge(path.siblings[level], cur)
            } else {
                native_merge(cur, path.siblings[level])
            };
        }
        cur
    }

    fn scenario(balance: u64, amount: u64, supply_old: u64) -> Scenario {
        escenario_para(balance, amount, supply_old, None)
    }

    /// `destinatario` permite construir el pendiente a nombre de OTRO, que
    /// es el ataque de §39. Con `None`, a nombre del titular: lo honesto.
    fn escenario_para(
        balance: u64,
        amount: u64,
        supply_old: u64,
        destinatario: Option<Digest>,
    ) -> Scenario {
        let mut empty = vec![[BaseElement::ZERO; 4]];
        for k in 1..=TREE_DEPTH {
            let prev = empty[k - 1];
            empty.push(native_merge(prev, prev));
        }
        // ⚠️ Ancha de verdad, no `as_digest(x)`: con relleno de ceros el
        // test pasaria sin ejercitar los tres elementos nuevos (§90.3).
        let key = [
            BaseElement::new(SK),
            BaseElement::new(0xC1A1),
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

        let leaf_old = native_leaf(account_id, BaseElement::new(balance), nonce);
        // **EL SALDO SUBE**: el receptor cobra el pendiente.
        let leaf_new = native_leaf(
            account_id,
            BaseElement::new(balance) + BaseElement::new(amount),
            nonce,
        );

        // Camino del arbol de congelados, con la cuenta LIBRE. Direcciones
        // mixtas: con todas iguales la traza degenera.
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

        // Camino del arbol de PENDIENTES, con la posicion libre.
        // Direcciones mixtas: con todas iguales la traza degenera.
        let pending_path = MerklePath {
            siblings: (0..TREE_DEPTH).map(|l| empty[l]).collect(),
            is_right: (0..TREE_DEPTH).map(|l| l % 4 == 0).collect(),
        };
        // ⚠️ Aqui decía `derive_public_id(0xB0B)` —una identidad DISTINTA
        // de la del titular— y los tests verificaban igual. Eso era §39: el
        // escenario documentaba el agujero en vez de detectarlo.
        let receiver_id = destinatario.unwrap_or(account_id);
        let salt: Digest = [
            BaseElement::new(0x5EED_0001),
            BaseElement::new(0x5EED_0002),
            BaseElement::new(0x5EED_0003),
            BaseElement::new(0x5EED_0004),
        ];
        // El compromiso, calculado de forma nativa para comparar.
        let pend_inner = native_merge(receiver_id, salt);
        let pending = native_merge(
            pend_inner,
            [BaseElement::new(amount), BaseElement::ZERO, BaseElement::ZERO, BaseElement::ZERO],
        );
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
            public_inputs: ClaimPublicInputs {
                root_old: native_climb(leaf_old, &path),
                root_new: native_climb(leaf_new, &path),
                amount: BaseElement::new(amount),
                supply_old: BaseElement::new(supply_old),
                // **UN ENVIO NO CAMBIA EL SUMINISTRO.**
                //
                // Heredado del circuito de destruccion, donde bajaba en el
                // importe. Declarar aqui otro valor que el de la traza hace
                // que probador y verificador usen entradas publicas
                // distintas, y con ellas transcripciones de Fiat-Shamir
                // distintas: la prueba se genera y **no verifica**, con el
                // error opaco `InconsistentOodConstraintEvaluations`.
                supply_new: BaseElement::new(supply_old),
                frozen_root,
                // **INVERTIDAS respecto al envio**: antes el pendiente
                // ESTA en el arbol, despues ya no.
                pending_root_old: climb_pending(pending),
                pending_root_new: climb_pending([BaseElement::ZERO; 4]),
            },
            key,
            account_id,
            balance,
            nonce,
            path,
            frozen_path,
            pending_path,
            receiver_id,
            salt,
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
            &s.path,
            &s.frozen_path,
            s.amount,
            s.supply_old,
            supply_delta,
            s.receiver_id,
            s.salt,
            &s.pending_path,
        );
        let prover = ClaimProver::new(default_options());

        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| prover.prove(trace)));

        let proof = match r {
            // ⚠️ **El mensaje del panico se conserva.**
            //
            // Winterfell comprueba en modo depuracion que las restricciones
            // se cumplan y que la cuenta de aserciones cuadre, y **da el
            // detalle**: *"expected 41 assertions, received 42"*, o el
            // indice y la fila de la restriccion que falla.
            //
            // Descartarlo con `Err(_) => "prove hizo panic"` tira justo el
            // dato que hace falta. En esta auditoria **costo tres rondas**
            // llegar a un fallo de una linea por eso. Ver `AUDITORIA.md`
            // §25.
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
        verify::<ClaimAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof,
            s.public_inputs.clone(),
            &min_opts,
        )
        .map_err(|e| format!("verificacion fallo: {e:?}"))
    }

    /// EL TEST CLAVE.
    // ENTRADA 36 / §50.7 CORREGIDA: la constancia de COL_SALT se impone;
    // este test estuvo en #[ignore] mientras el fallo vivia, ahora pasa verde.
    #[test]
    fn a_claim_with_inconsistent_salt_is_rejected() {
        // ENTRADA 36 / hermano de §50. La 36 se catalogo como "borrar 8
        // muertas". Pero claim tiene el MISMO solapamiento que send, y aunque
        // COL_R_ID esta muerto tras §39.1, el compromiso AUN lee COL_SALT
        // (C_PEND_IN + 8). Si la constancia de COL_SALT esta muerta por el
        // solapamiento, un probador mete un aleatorio en la fila del
        // compromiso y otro en el resto. Dejamos el salt real en
        // ROW_FROZEN_ROOT y metemos otro en las demas filas.
        let s = scenario(1_000_000, 250_000, 10_000_000);
        let mut trace = build_trace(
            s.key, s.account_id, s.balance, s.nonce, &s.path, &s.frozen_path,
            s.amount, s.supply_old, 0, s.receiver_id, s.salt, &s.pending_path,
        );

        let otro_salt = derive_public_id(BaseElement::new(0x5A17));
        assert_ne!(otro_salt, s.salt, "el testigo debe diferir");
        for row in 0..TRACE_LENGTH {
            if row == ROW_FROZEN_ROOT { continue; } // fila del compromiso, intacta
            for i in 0..4 {
                trace.set(COL_SALT + i, row, otro_salt[i]);
            }
        }

        let prover = ClaimProver::new(default_options());
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(
            || prover.prove(trace)));
        let verifica = match r {
            Err(_) => false,
            Ok(Err(_)) => false,
            Ok(Ok(proof)) => {
                let min_opts = AcceptableOptions::OptionSet(vec![default_options()]);
                verify::<ClaimAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
                    proof, s.public_inputs.clone(), &min_opts,
                ).is_ok()
            }
        };
        assert!(
            !verifica,
            "SOLIDEZ (entrada 36): claim acepta COL_SALT inconsistente entre la \
             fila del compromiso y el resto -> §50 sobre el aleatorio. La \
             constancia de COL_SALT esta muerta por el solapamiento."
        );
    }

    #[test]
    fn an_authorized_claim_verifies() {
        let s = scenario(1_000_000, 250_000, 10_000_000);
        let trace = build_trace(
            s.key,
            s.account_id,
            s.balance,
            s.nonce,
            &s.path,
            &s.frozen_path,
            s.amount,
            s.supply_old,
            // ⚠️ Heredado del circuito de destruccion, donde el suministro
            // bajaba en el importe. Un ENVIO no cambia el suministro: debe
            // ser CERO.
            0,
            s.receiver_id,
            s.salt,
            &s.pending_path,
        );
        let prover = ClaimProver::new(default_options());
        let proof = prover.prove(trace).expect("la destruccion valida deberia probar");
        let min_opts = AcceptableOptions::OptionSet(vec![default_options()]);
        let v = verify::<ClaimAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof,
            s.public_inputs.clone(),
            &min_opts,
        );
        assert!(v.is_ok(), "{v:?}");
    }

    /// **NADIE PUEDE DESTRUIR EL DINERO DE OTRO.**
    ///
    /// Un tercero conoce la identidad, el saldo, el nonce y el camino de
    /// una cuenta ajena, pero no la clave. La identidad derivada de su
    /// clave no coincide con la de la cuenta.
    #[test]
    fn a_third_party_cannot_claim_with_someone_elses_key() {
        let s = scenario(1_000_000, 250_000, 10_000_000);
        assert!(
            run(
                &s,
                [
                    BaseElement::new(0x1337),
                    BaseElement::new(0xBADC0DE),
                    BaseElement::new(0x0DDBA11),
                    BaseElement::new(0x1CEB00DA),
                ],
                s.amount
            )
            .is_err(),
            "CRITICO: sin la clave del titular no debe poder destruirse su saldo"
        );
    }

    /// **DESTRUCCIÓN ENCUBIERTA.**
    ///
    /// Se reduce un saldo sin reflejarlo en el suministro. El dinero
    /// desaparecería pero la cifra auditable no lo registraría, y la
    /// invariante global se rompería en silencio.
    #[test]
    fn changing_the_supply_is_rejected() {
        let s = scenario(1_000_000, 250_000, 10_000_000);
        assert!(
            run(&s, s.key, 250_000).is_err(),
            "CRITICO: destruir sin registrarlo en el suministro romperia la \
             invariante global"
        );
    }

    /// Reducir el suministro más de lo destruido tampoco cuela.
    #[test]
    fn inflating_the_supply_is_rejected() {
        let s = scenario(1_000_000, 250_000, 10_000_000);
        assert!(run(&s, s.key, 500_000).is_err());
    }

    /// **SIN LA CLAVE DE LA CUENTA NO SE COBRA.**
    ///
    /// ⚠️ Este test se llamaba `nobody_else_can_claim_a_pending_transfer` y
    /// decía sostener toda la segunda fase, afirmando que fallaba porque *el
    /// compromiso reconstruido con otra identidad es otro*. **No era
    /// cierto**: cambia la clave de gasto, así que falla por `C_PK_CHECK`.
    /// La propiedad que decía cubrir no la cubría nadie, y de ahí salió
    /// §39. Renombrado a lo que sí prueba.
    #[test]
    fn without_the_account_key_nothing_can_be_claimed() {
        let s = scenario(1_000_000, 250_000, 10_000_000);
        assert!(
            run(
                &s,
                [
                    BaseElement::new(0xBADCAFE),
                    BaseElement::new(0xBADC0DE),
                    BaseElement::new(0x0DDBA11),
                    BaseElement::new(0x1CEB00DA),
                ],
                0
            )
            .is_err(),
            "CRITICO: sin la clave de la cuenta no debe poder cobrarse"
        );
    }

    /// **NADIE MÁS PUEDE COBRAR UN PENDIENTE AJENO.** — el de verdad.
    ///
    /// Mallory tiene **su propia cuenta y su propia clave**, y conoce el
    /// aleatorio y el importe de un pendiente dirigido a otra identidad.
    /// Todo lo demás de la traza es coherente. Si el circuito no atara la
    /// identidad del compromiso a la de la cuenta que cobra, esto
    /// verificaría — y durante todo el desarrollo verificó (§39).
    #[test]
    fn nobody_else_can_claim_a_pending_transfer() {
        let ajeno = derive_public_id(BaseElement::new(0xB0B));
        let s = escenario_para(1_000_000, 250_000, 10_000_000, Some(ajeno));
        assert_ne!(ajeno, s.account_id);
        assert!(
            run(&s, s.key, 0).is_err(),
            "CRITICO: un pendiente dirigido a OTRA identidad no debe poder \
             cobrarse aunque se tenga una cuenta y una clave validas"
        );
    }

    /// **PASO 1 DE LA ENTRADA 27 DEL BACKLOG (`AUDITORIA.md` §38.2).**
    ///
    /// El test de arriba dice fallar porque *«el compromiso reconstruido
    /// con su identidad es otro»*, pero cambia **la clave**, no la
    /// identidad del receptor: falla por `C_PK_CHECK`. Este cambia la
    /// identidad del receptor y **nada más**.
    ///
    /// Si el compromiso está atado al árbol de pendientes, la subida no
    /// puede llegar a la raíz declarada y esto debe rechazarse. Si
    /// verifica, `COL_R_ID` no está atado a nada.
    #[test]
    fn a_claim_reconstructing_a_different_commitment_is_rejected() {
        let s = scenario(1_000_000, 250_000, 10_000_000);
        let otra_identidad = derive_public_id(BaseElement::new(0xDECAFB));
        assert_ne!(otra_identidad, s.receiver_id);

        let trace = build_trace(
            s.key,
            s.account_id,
            s.balance,
            s.nonce,
            &s.path,
            &s.frozen_path,
            s.amount,
            s.supply_old,
            0,
            otra_identidad,
            s.salt,
            &s.pending_path,
        );
        let prover = ClaimProver::new(default_options());

        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| prover.prove(trace)));

        let verifica = match r {
            Err(_) => false,
            Ok(Err(_)) => false,
            Ok(Ok(proof)) => {
                let min_opts = AcceptableOptions::OptionSet(vec![default_options()]);
                verify::<ClaimAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
                    proof,
                    s.public_inputs.clone(),
                    &min_opts,
                )
                .is_ok()
            }
        };
        assert!(
            !verifica,
            "CRITICO: reconstruir el compromiso con OTRA identidad no debe \
             producir una prueba valida contra la misma raiz de pendientes"
        );
    }

    /// **Y el que valida al anterior**: con la clave correcta, sí.
    ///
    /// Sin esto, el anterior pasaría aunque la reclamación fallara siempre
    /// por cualquier otra razón.
    #[test]
    fn the_rightful_recipient_can_claim() {
        let s = scenario(1_000_000, 250_000, 10_000_000);
        assert!(run(&s, s.key, 0).is_ok());
    }

    /// Declarar una raíz nueva que no corresponde al saldo reducido.
    #[test]
    fn wrong_new_root_is_rejected() {
        let s = scenario(1_000_000, 250_000, 10_000_000);
        let mut declared = s.public_inputs.clone();
        declared.root_new = [BaseElement::new(999_999); 4];

        let trace = build_trace(
            s.key,
            s.account_id,
            s.balance,
            s.nonce,
            &s.path,
            &s.frozen_path,
            s.amount,
            s.supply_old,
            // ⚠️ Heredado del circuito de destruccion, donde el suministro
            // bajaba en el importe. Un ENVIO no cambia el suministro: debe
            // ser CERO.
            0,
            s.receiver_id,
            s.salt,
            &s.pending_path,
        );
        let prover = ClaimProver::new(default_options());
        let proof = prover.prove(trace).expect("prove");
        let min_opts = AcceptableOptions::OptionSet(vec![default_options()]);
        let v = verify::<ClaimAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof, declared, &min_opts,
        );
        assert!(v.is_err());
    }

    // -----------------------------------------------------------------
    // No-pertenencia al árbol de congelados
    // -----------------------------------------------------------------

    /// **UNA CUENTA CONGELADA NO PUEDE DESTRUIR SU DINERO.**
    ///
    /// Es el test que justifica toda la fase nueva del circuito.
    ///
    /// Antes de esto, la liquidación comprobaba la congelación y la
    /// destrucción no: un titular bajo investigación podía **vaciar su
    /// cuenta a cero**. No se llevaba el dinero —se destruía— pero el
    /// saldo investigado desaparecía.
    ///
    /// Congelar existe para que una cuenta bajo investigación no mueva
    /// fondos. Destruirlos los mueve.
    #[test]
    fn a_frozen_account_cannot_claim() {
        let mut s = scenario(1_000_000, 250_000, 10_000_000);

        // La cuenta SI esta en el arbol de congelados.
        let hoja = crate::circuit_freeze::frozen_leaf(true);
        s.public_inputs.frozen_root =
            crate::circuit_freeze::frozen_climb(hoja, &s.frozen_path);

        assert!(
            run(&s, s.key, 0).is_err(),
            "CRITICO: una cuenta congelada no debe poder destruir su dinero"
        );
    }

    /// **Y el que valida al anterior**: una cuenta libre SÍ puede.
    ///
    /// Sin esto, el test anterior pasaría aunque la fase de congelados
    /// rechazara todo — o no impusiera nada y fallara por otra razón.
    #[test]
    fn a_free_account_can_claim() {
        let s = scenario(1_000_000, 250_000, 10_000_000);
        assert!(
            run(&s, s.key, 0).is_ok(),
            "una cuenta libre debe poder enviar"
        );
    }

    /// **DECLARAR UNA RAÍZ DE CONGELADOS FALSA SE RECHAZA.**
    ///
    /// Sin esto, un congelado se libraría declarando la raíz de un árbol
    /// vacío que él mismo construye.
    #[test]
    fn a_forged_frozen_root_is_rejected() {
        let mut s = scenario(1_000_000, 250_000, 10_000_000);
        s.public_inputs.frozen_root = [BaseElement::new(0xFA15E); 4];
        assert!(run(&s, s.key, 0).is_err());
    }

    /// **SEPARA "LA TRAZA ESTÁ MAL" DE "LAS RESTRICCIONES ESTÁN MAL".**
    ///
    /// Compara cada punto de referencia de la traza con su cálculo nativo.
    /// Si pasa, la traza es correcta y el fallo está en restricciones,
    /// grados o aserciones. Si falla, dice **exactamente cuál**.
    ///
    /// En `circuit_redeem` este test ahorró varias rondas de diagnóstico:
    /// al pasar, descartó de entrada la mitad del espacio de búsqueda.
    #[test]
    fn trace_landmarks_match_native() {
        let s = scenario(1_000_000, 250_000, 10_000_000);
        let trace = build_trace(
            s.key,
            s.account_id,
            s.balance,
            s.nonce,
            &s.path,
            &s.frozen_path,
            s.amount,
            s.supply_old,
            0,
            s.receiver_id,
            s.salt,
            &s.pending_path,
        );
        let esperados = &s.public_inputs;

        // ===== TODAS LAS ENTRADAS PUBLICAS, NO SOLO LAS RAICES =====
        //
        // Una version anterior comprobaba solo las raices. El escenario
        // declaraba `supply_new = supply_old - amount` —heredado del
        // circuito de destruccion— mientras la traza tenia
        // `supply_new = supply_old`.
        //
        // Probador y verificador usaban entradas publicas distintas, y con
        // ellas transcripciones de Fiat-Shamir distintas: la prueba se
        // generaba y **no verificaba**, con el error opaco
        // `InconsistentOodConstraintEvaluations`.
        //
        // **Costo ocho rondas de diagnostico.** Comparar la estructura
        // entera, y no los campos que parecen importantes, lo habria
        // cazado a la primera.
        let derivadas = ClaimProver::new(default_options()).get_pub_inputs(&trace);
        assert_eq!(
            derivadas.to_elements(),
            esperados.to_elements(),
            "las entradas publicas DERIVADAS de la traza deben coincidir con \
             las DECLARADAS, o probador y verificador usaran transcripciones \
             distintas"
        );

        for i in 0..4 {
            assert_eq!(
                trace.get(4 + i, ROW_ROOT),
                esperados.root_old[i],
                "raiz de cuentas ANTES, elemento {i}"
            );
            assert_eq!(
                trace.get(LANE_B + 4 + i, ROW_ROOT),
                esperados.root_new[i],
                "raiz de cuentas DESPUES, elemento {i}"
            );
            assert_eq!(
                trace.get(4 + i, ROW_FROZEN_ROOT),
                esperados.frozen_root[i],
                "raiz de congelados, elemento {i}"
            );
            assert_eq!(
                trace.get(4 + i, ROW_PENDING_ROOT),
                esperados.pending_root_old[i],
                "raiz de pendientes ANTES, elemento {i}"
            );
            assert_eq!(
                trace.get(LANE_B + 4 + i, ROW_PENDING_ROOT),
                esperados.pending_root_new[i],
                "raiz de pendientes DESPUES, elemento {i}"
            );
        }
    }

    /// **EL COMPROMISO SE FORMA CON LA IDENTIDAD, NO CON EL SALDO.**
    ///
    /// Es la propiedad que cierra la fuga, y va **en el tipo**: la firma
    /// de `build_trace` recibe `receiver_id` y `salt`. **No hay parámetro
    /// donde pudiera entrar un saldo.**
    ///
    /// Este test lo confirma calculando el compromiso de forma nativa a
    /// partir de la identidad y comprobando que es el que la traza
    /// inserta.
    #[test]
    fn the_commitment_is_built_from_the_identity_not_the_balance() {
        let s = scenario(1_000_000, 250_000, 10_000_000);
        let nativo = native_merge(
            native_merge(s.receiver_id, s.salt),
            [
                BaseElement::new(s.amount),
                BaseElement::ZERO,
                BaseElement::ZERO,
                BaseElement::ZERO,
            ],
        );
        // La raiz nueva de pendientes debe ser la de insertar ESE
        // compromiso, calculado sin conocer ningun saldo del receptor.
        let mut cur = nativo;
        for level in 0..TREE_DEPTH {
            cur = if s.pending_path.is_right[level] {
                native_merge(s.pending_path.siblings[level], cur)
            } else {
                native_merge(cur, s.pending_path.siblings[level])
            };
        }
        assert_eq!(
            cur, s.public_inputs.pending_root_old,
            "el compromiso se reconstruye con la identidad de quien reclama: \
             es lo que impide cobrar el pendiente de otro"
        );
        assert_eq!(
            s.public_inputs.pending_root_new,
            climb_empty(&s.pending_path),
            "y tras reclamarlo el arbol queda sin el: no puede cobrarse dos veces"
        );
    }

    /// **DECLARAR OTRA RAIZ DE PENDIENTES SE RECHAZA.**
    ///
    /// Sin esto, se podria afirmar haber insertado un compromiso distinto
    /// del que la traza construye.
    #[test]
    fn a_wrong_pending_root_is_rejected() {
        let mut s = scenario(1_000_000, 250_000, 10_000_000);
        s.public_inputs.pending_root_new = [BaseElement::new(0xFA15E); 4];
        assert!(run(&s, s.key, 0).is_err());
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

        let s = scenario(1_000_000, 250_000, 10_000_000);
        let trace = build_trace(
            s.key,
            s.account_id,
            s.balance,
            s.nonce,
            &s.path,
            &s.frozen_path,
            s.amount,
            s.supply_old,
            0,
            s.receiver_id,
            s.salt,
            &s.pending_path,
        );
        let rows = rows_of(&trace, TRACE_WIDTH, TRACE_LENGTH);

        let air = ClaimAir::new(
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

    /// **NO SE PUEDE COBRAR UN IMPORTE DISTINTO DEL COMPROMETIDO.**
    ///
    /// Si se pudiera, **se crearía dinero**: el pagador comprometió N y el
    /// receptor se abonaría M.
    ///
    /// El circuito lo impide por construcción —el compromiso se forma con
    /// `(identidad, aleatorio, importe)`, así que cambiar el importe da
    /// otro compromiso que no está en el árbol— pero **nada lo comprobaba
    /// en esta vía**.
    ///
    /// ⚠️ **Estaba verificado solo sobre el modelo.** `pending.rs` tiene
    /// `claiming_a_different_amount_is_rejected`, pero ese módulo es un
    /// prototipo que la capa **no usa**: la producción va por
    /// `two_phase.rs` y este circuito. Ver `AUDITORIA.md` §24.
    ///
    /// Una propiedad de seguridad demostrada sobre un modelo **no está
    /// demostrada sobre lo que se ejecuta**.
    #[test]
    fn claiming_a_different_amount_is_rejected() {
        let s = scenario(1_000_000, 250_000, 10_000_000);

        // El compromiso del árbol se formó con `s.amount`. Se intenta
        // cobrar el doble, dejando todo lo demás igual.
        let trace = build_trace(
            s.key,
            s.account_id,
            s.balance,
            s.nonce,
            &s.path,
            &s.frozen_path,
            s.amount * 2,
            s.supply_old,
            0,
            s.receiver_id,
            s.salt,
            &s.pending_path,
        );
        let prover = ClaimProver::new(default_options());
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| prover.prove(trace)));

        let ok = match r {
            Ok(Ok(proof)) => {
                let min_opts = AcceptableOptions::OptionSet(vec![default_options()]);
                verify::<ClaimAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
                    proof,
                    s.public_inputs.clone(),
                    &min_opts,
                )
                .is_ok()
            }
            _ => false,
        };
        assert!(
            !ok,
            "CRITICO: cobrar mas de lo comprometido crearia dinero"
        );
    }
}
