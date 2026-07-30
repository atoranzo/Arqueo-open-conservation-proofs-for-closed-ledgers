//! **Partida doble completa en STARK** — equivalente del
//! `zk-core::circuit_double_entry` y del
//! `halo2-experiment::circuit_double_entry`.
//!
//! Demuestra la transición de estado completa de una transferencia,
//! conservando el dinero, sin revelar saldos, identidades ni importes:
//!
//! ```text
//! saldo_emisor_nuevo   = saldo_emisor   - importe   (ADEUDO)
//! saldo_receptor_nuevo = saldo_receptor + importe   (ABONO)
//! ```
//!
//! Públicos: `root_old`, `root_new`, `regulatory_limit`, `nullifier`.
//! Privados: identidades, saldos, nonces, importe y ambos caminos.
//!
//! ## Los dos problemas de AIR que esta pieza resuelve
//!
//! **1. No hay restricciones de copia.** Las dos subidas del árbol (hoja
//! antigua y hoja nueva) van en **lockstep**: dos carriles avanzando
//! nivel a nivel a la vez, con una restricción que fuerza que el hermano
//! inyectado sea idéntico en ambos. Sin eso, un probador podría usar
//! hermanos distintos en cada subida y fabricar una raíz que no
//! corresponde a la misma posición del árbol. Verificado de forma
//! aislada en `dual_climb.rs`.
//!
//! **2. No se pueden comparar filas lejanas.** La raíz intermedia
//! (`root_mid`) aparece en la fila 271 y debe comprobarse en la 543.
//! Solución: cuatro columnas de transporte constantes que hacen de
//! puente entre ambas.
//!
//! ## Dónde vive la conservación
//!
//! En dos restricciones sobre transporte, activas en toda la traza:
//! `c_s_bal_new = c_s_bal - c_amt` y `c_r_bal_new = c_r_bal + c_amt`. Esos
//! valores alimentan las hojas hasheadas, cuyas raíces son públicas — así
//! que no es una tautología.
//!
//! ## Estructura de la traza (41 columnas × 1024 filas)
//!
//! | Filas | Contenido |
//! |---|---|
//! | 0..15 | Hojas del emisor: carril A = antigua, B = nueva |
//! | 16..271 | Subida dual del emisor (32 niveles) |
//! | 272..287 | Hojas del receptor |
//! | 288..543 | Subida dual del receptor (32 niveles) |
//! | 544..559 | Nullifier (ambos carriles calculan lo mismo) |
//! | 0..447 | Carril de solvencia, en paralelo: 7 segmentos de 64 filas |
//!
//! Que ambos carriles calculen el mismo nullifier desperdicia trabajo,
//! pero mantiene las restricciones de hash uniformes sobre los dos
//! carriles y evita selectores adicionales. Es una decisión de
//! simplicidad, no una necesidad.

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
use crate::nullifier::NULLIFIER_DOMAIN;
use crate::rescue_hash::{apply_sbox, NUM_ROUNDS, STATE_WIDTH};

pub const CYCLE_LENGTH: usize = 8;
pub const TRACE_LENGTH: usize = 1024;
/// Filas por segmento de range check.
pub const SEGMENT_LENGTH: usize = 64;
/// Segmentos: s_bal, r_bal, amount, limit, s_bal_new, limit-amount, r_bal_new.
pub const NUM_SEGMENTS: usize = 7;
/// Rango efectivo: 63 bits (techo del campo Goldilocks, ver `range_check.rs`).
pub const MAX_VALUE: u64 = (1u64 << 63) - 1;

// ===== Columnas =====
const LANE_B: usize = STATE_WIDTH; // 12
const COL_BIT: usize = 24;
const COL_S_ID: usize = 25;
const COL_S_BAL: usize = 26;
const COL_S_NONCE: usize = 27;
const COL_S_BAL_NEW: usize = 28;
const COL_R_ID: usize = 29;
const COL_R_BAL: usize = 30;
const COL_R_NONCE: usize = 31;
const COL_R_BAL_NEW: usize = 32;
const COL_AMT: usize = 33;
const COL_LIM: usize = 34;
const COL_ROOT_MID: usize = 35; // 35..39
const COL_SBIT: usize = 39;
const COL_SACC: usize = 40;
pub const TRACE_WIDTH: usize = 41;

// ===== Filas de eventos =====
const ROW_S_LEAF_LINK: usize = 7;
const ROW_S_LEAF_DONE: usize = 15;
const ROW_S_ROOT: usize = 271;
const ROW_R_LEAF_LINK: usize = 279;
const ROW_R_LEAF_DONE: usize = 287;
const ROW_R_ROOT: usize = 543;
const ROW_NULL_START: usize = 544;
const ROW_NULL_LINK: usize = 551;
const ROW_NULLIFIER: usize = 559;

// ===== Índices de restricción, con nombre =====
// Escribirlos a mano ha sido la fuente de error más frecuente de este
// proyecto; aquí se derivan unos de otros.
const C_HASH_A: usize = 0; // 0..11
const C_HASH_B: usize = C_HASH_A + STATE_WIDTH; // 12..23
const C_TREE_CAP_A: usize = C_HASH_B + STATE_WIDTH; // 24..27
const C_TREE_CAP_B: usize = C_TREE_CAP_A + 4; // 28..31
const C_PLACE_A: usize = C_TREE_CAP_B + 4; // 32..35
const C_PLACE_B: usize = C_PLACE_A + 4; // 36..39
const C_SIBLING: usize = C_PLACE_B + 4; // 40..43
const C_BIT_BOOL: usize = C_SIBLING + 4; // 44
const C_LEAF_CAP_A: usize = C_BIT_BOOL + 1; // 45..48
const C_LEAF_CAP_B: usize = C_LEAF_CAP_A + 4; // 49..52
const C_LEAF_DIG_A: usize = C_LEAF_CAP_B + 4; // 53..56
const C_LEAF_DIG_B: usize = C_LEAF_DIG_A + 4; // 57..60
const C_NONCE: usize = C_LEAF_DIG_B + 4; // 61..66 (6)
const C_S_INPUT: usize = C_NONCE + 6; // 67..70
const C_R_INPUT: usize = C_S_INPUT + 4; // 71..74
const C_MID_CAPTURE: usize = C_R_INPUT + 4; // 75..78
const C_MID_CHECK: usize = C_MID_CAPTURE + 4; // 79..82
const C_CONSERVATION: usize = C_MID_CHECK + 4; // 83..84
const C_TRANSPORT: usize = C_CONSERVATION + 2; // 85..94 (10)
const C_MID_CONST: usize = C_TRANSPORT + 10; // 95..98
const C_NULL_ID: usize = C_MID_CONST + 4; // 99..100
const C_SBIT_BOOL: usize = C_NULL_ID + 2; // 101..102
const C_FIRST_S: usize = C_SBIT_BOOL + 2; // 103..104
const C_HORNER: usize = C_FIRST_S + 2; // 105
const C_SEG_LINK: usize = C_HORNER + 1; // 106..112
const NUM_CONSTRAINTS: usize = C_SEG_LINK + NUM_SEGMENTS; // 113

// ===== Índices de columnas periódicas =====
const P_HASH_FLAG: usize = 0;
const P_ARK1: usize = 1; // 1..13
const P_ARK2: usize = P_ARK1 + STATE_WIDTH; // 13..25
const P_LINK_MERKLE: usize = P_ARK2 + STATE_WIDTH; // 25
const P_LINK_LEAF: usize = 26;
const P_LINK_PLACE: usize = 27;
const P_SEL_S_LEAF: usize = 28;
const P_SEL_R_LEAF: usize = 29;
const P_SEL_NULL_LEAF: usize = 30;
const P_FIRST_ROW: usize = 31;
const P_SEL_S_ROOT: usize = 32;
const P_SEL_R_ROOT: usize = 33;
const P_FIRST_S: usize = 34;
const P_CONT_S: usize = 35;
const P_SEG_LINK: usize = 36; // 36..43

type Blake3 = Blake3_256<BaseElement>;

fn as_digest(x: BaseElement) -> Digest {
    [x, BaseElement::ZERO, BaseElement::ZERO, BaseElement::ZERO]
}

/// Hoja nativa: Rescue(Rescue(id, saldo), nonce).
pub fn native_leaf(id: BaseElement, balance: BaseElement, nonce: BaseElement) -> Digest {
    let inner = native_merge(as_digest(id), as_digest(balance));
    native_merge(inner, as_digest(nonce))
}

/// Nullifier nativo: Rescue(Rescue(DOMAIN, id), nonce).
pub fn native_nullifier(id: BaseElement, nonce: BaseElement) -> Digest {
    let inner = native_merge(as_digest(BaseElement::new(NULLIFIER_DOMAIN)), as_digest(id));
    native_merge(inner, as_digest(nonce))
}

/// Sube una hoja por un camino, de forma nativa.
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

/// Testigos de una de las dos partes.
#[derive(Clone, Debug)]
pub struct PartyWitness {
    pub account_id: BaseElement,
    pub balance: u64,
    pub nonce: BaseElement,
    pub path: MerklePath,
}

/// Bits en BIG-ENDIAN (MSB primero), como en `solvency.rs`.
fn value_to_bits_be(value: u64) -> Vec<bool> {
    (0..SEGMENT_LENGTH)
        .map(|p| (value >> (SEGMENT_LENGTH - 1 - p)) & 1 == 1)
        .collect()
}

/// Construye la traza completa.
///
/// `credited` permite acreditar al receptor una cantidad distinta de la
/// debitada, para construir los tests que rompen la conservación. En uso
/// normal debe ser igual a `amount`.
pub fn build_trace(
    sender: &PartyWitness,
    receiver: &PartyWitness,
    amount: u64,
    credited: u64,
    limit: u64,
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
        row[COL_S_ID] = sender.account_id;
        row[COL_S_BAL] = s_bal;
        row[COL_S_NONCE] = sender.nonce;
        row[COL_S_BAL_NEW] = s_bal_new;
        row[COL_R_ID] = receiver.account_id;
        row[COL_R_BAL] = r_bal;
        row[COL_R_NONCE] = receiver.nonce;
        row[COL_R_BAL_NEW] = r_bal_new;
        row[COL_AMT] = c_amt;
        row[COL_LIM] = c_lim;
    }

    // --- Carril de solvencia: 7 segmentos de Horner big-endian ---
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
    state_a[4] = sender.account_id;
    state_a[8] = s_bal;
    state_b[4] = sender.account_id;
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

    for r in 0..ROW_NULLIFIER {
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
                    state_a[4] = receiver.account_id;
                    state_a[8] = r_bal;
                    state_b[4] = receiver.account_id;
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
                    // Arranque del nullifier: AMBOS carriles calculan lo
                    // mismo, para mantener uniformes las restricciones de
                    // hash sin selectores adicionales.
                    state_a[4] = BaseElement::new(NULLIFIER_DOMAIN);
                    state_a[8] = sender.account_id;
                    state_b[4] = BaseElement::new(NULLIFIER_DOMAIN);
                    state_b[8] = sender.account_id;
                }
                ROW_NULL_LINK => {
                    state_a[4..8].copy_from_slice(&digest_a);
                    state_a[8] = sender.nonce;
                    state_b[4..8].copy_from_slice(&digest_b);
                    state_b[8] = sender.nonce;
                }
                _ => {
                    let next_cycle = (r + 1) / CYCLE_LENGTH;
                    if (2..34).contains(&next_cycle) {
                        let level = next_cycle - 2;
                        place(&mut state_a, &digest_a, &sender.path, level);
                        place(&mut state_b, &digest_b, &sender.path, level);
                    } else if (36..68).contains(&next_cycle) {
                        let level = next_cycle - 36;
                        place(&mut state_a, &digest_a, &receiver.path, level);
                        place(&mut state_b, &digest_b, &receiver.path, level);
                    }
                }
            }
        }
        rows[r + 1][..STATE_WIDTH].copy_from_slice(&state_a);
        rows[r + 1][LANE_B..LANE_B + STATE_WIDTH].copy_from_slice(&state_b);
    }

    // --- root_mid en el transporte ---
    for row in rows.iter_mut() {
        for i in 0..4 {
            row[COL_ROOT_MID + i] = root_mid[i];
        }
    }

    // --- Bit de dirección ---
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
        for p in 0..CYCLE_LENGTH {
            rows[(2 + level) * CYCLE_LENGTH + p][COL_BIT] = s_bit;
            rows[(36 + level) * CYCLE_LENGTH + p][COL_BIT] = r_bit;
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
pub struct DoubleEntryPublicInputs {
    pub root_old: Digest,
    pub root_new: Digest,
    pub regulatory_limit: BaseElement,
    pub nullifier: Digest,
}

impl ToElements<BaseElement> for DoubleEntryPublicInputs {
    fn to_elements(&self) -> Vec<BaseElement> {
        let mut out = self.root_old.to_vec();
        out.extend_from_slice(&self.root_new);
        out.push(self.regulatory_limit);
        out.extend_from_slice(&self.nullifier);
        out
    }
}

pub struct DoubleEntryAir {
    context: AirContext<BaseElement>,
    pub_inputs: DoubleEntryPublicInputs,
}

impl Air for DoubleEntryAir {
    type BaseField = BaseElement;
    type PublicInputs = DoubleEntryPublicInputs;

    fn new(trace_info: TraceInfo, pub_inputs: Self::PublicInputs, options: ProofOptions) -> Self {
        assert_eq!(TRACE_WIDTH, trace_info.width());
        let full = vec![TRACE_LENGTH];

        let mut degrees = Vec::with_capacity(NUM_CONSTRAINTS);
        // Hash de ambos carriles: grado 7.
        for _ in 0..2 * STATE_WIDTH {
            degrees.push(TransitionConstraintDegree::with_cycles(7, full.clone()));
        }
        // Capacidad de árbol (8) — grado 1.
        for _ in 0..8 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }
        // Colocación (8) y hermano compartido (4) — grado 2.
        for _ in 0..12 {
            degrees.push(TransitionConstraintDegree::with_cycles(2, full.clone()));
        }
        // Bit booleano — grado 2, sin ciclo.
        degrees.push(TransitionConstraintDegree::new(2));
        // Enlaces de hoja: capacidad (8) + digest (8) — grado 1.
        for _ in 0..16 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }
        // Nonces (6), entradas emisor (4), receptor (4) — grado 1.
        for _ in 0..14 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }
        // Captura y comprobación de root_mid (8) — grado 1.
        for _ in 0..8 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }
        // Conservación (2) y transporte (10) y root_mid constante (4) —
        // grado 1, SIN ciclo (activas en toda la traza).
        for _ in 0..16 {
            degrees.push(TransitionConstraintDegree::new(1));
        }
        // account_id en el nullifier (2) — grado 1 con ciclo.
        for _ in 0..2 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }
        // Solvencia: bits booleanos (2) — grado 2 sin ciclo.
        for _ in 0..2 {
            degrees.push(TransitionConstraintDegree::new(2));
        }
        // first_s (2), Horner (1), links de segmento (7) — grado 1 con ciclo.
        for _ in 0..(3 + NUM_SEGMENTS) {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }

        assert_eq!(degrees.len(), NUM_CONSTRAINTS, "cuenta de grados");

        DoubleEntryAir {
            // 44 aserciones: ver get_assertions.
            context: AirContext::new(trace_info, degrees, 44, options),
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

        // Selector de ronda, sobre TODO el tramo activo (incluido el
        // nullifier).
        let mut hash_flag = vec![zero; TRACE_LENGTH];
        for r in 0..=ROW_NULLIFIER {
            if r % CYCLE_LENGTH < NUM_ROUNDS {
                hash_flag[r] = one;
            }
        }
        columns.push(hash_flag);

        for ark in [true, false] {
            for i in 0..STATE_WIDTH {
                let mut col = vec![zero; TRACE_LENGTH];
                for r in 0..=ROW_NULLIFIER {
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
        }
        columns.push(link_merkle);

        // Enlaces "interior → exterior": las dos hojas y el nullifier.
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

        let mut first_row = vec![zero; TRACE_LENGTH];
        first_row[0] = one;
        columns.push(first_row);

        let mut sel_s_root = vec![zero; TRACE_LENGTH];
        sel_s_root[ROW_S_ROOT] = one;
        columns.push(sel_s_root);

        let mut sel_r_root = vec![zero; TRACE_LENGTH];
        sel_r_root[ROW_R_ROOT] = one;
        columns.push(sel_r_root);

        // Solvencia.
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
        let first_s = periodic[P_FIRST_S];
        let cont_s = periodic[P_CONT_S];

        // ===== Rondas de Rescue en ambos carriles =====
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

        // ===== Enlaces de árbol =====
        for i in 0..4 {
            result[C_TREE_CAP_A + i] = tree_link * next[i];
            result[C_TREE_CAP_B + i] = tree_link * next[LANE_B + i];

            let da = current[4 + i];
            let placed_a = (E::ONE - bit) * (next[4 + i] - da) + bit * (next[8 + i] - da);
            result[C_PLACE_A + i] = tree_link * placed_a;

            let db = current[LANE_B + 4 + i];
            let placed_b =
                (E::ONE - bit) * (next[LANE_B + 4 + i] - db) + bit * (next[LANE_B + 8 + i] - db);
            result[C_PLACE_B + i] = tree_link * placed_b;

            // EL HERMANO ES EL MISMO EN AMBOS CARRILES.
            let sib_a = (E::ONE - bit) * next[8 + i] + bit * next[4 + i];
            let sib_b = (E::ONE - bit) * next[LANE_B + 8 + i] + bit * next[LANE_B + 4 + i];
            result[C_SIBLING + i] = tree_link * (sib_a - sib_b);
        }

        result[C_BIT_BOOL] = current[COL_BIT] * (current[COL_BIT] - E::ONE);

        // ===== Enlaces interior → exterior (hojas y nullifier) =====
        for i in 0..4 {
            result[C_LEAF_CAP_A + i] = link_leaf * next[i];
            result[C_LEAF_CAP_B + i] = link_leaf * next[LANE_B + i];
            result[C_LEAF_DIG_A + i] = link_leaf * (next[4 + i] - current[4 + i]);
            result[C_LEAF_DIG_B + i] =
                link_leaf * (next[LANE_B + 4 + i] - current[LANE_B + 4 + i]);
        }

        // ===== El nonce correcto entra en cada hash =====
        result[C_NONCE] = sel_s_leaf * (next[8] - current[COL_S_NONCE]);
        result[C_NONCE + 1] = sel_s_leaf * (next[LANE_B + 8] - (current[COL_S_NONCE] + E::ONE));
        result[C_NONCE + 2] = sel_r_leaf * (next[8] - current[COL_R_NONCE]);
        result[C_NONCE + 3] = sel_r_leaf * (next[LANE_B + 8] - current[COL_R_NONCE]);
        result[C_NONCE + 4] = sel_null_leaf * (next[8] - current[COL_S_NONCE]);
        result[C_NONCE + 5] = sel_null_leaf * (next[LANE_B + 8] - current[COL_S_NONCE]);

        // ===== Entradas de las hojas: aquí entran adeudo y abono =====
        result[C_S_INPUT] = first_row * (current[4] - current[COL_S_ID]);
        result[C_S_INPUT + 1] = first_row * (current[8] - current[COL_S_BAL]);
        result[C_S_INPUT + 2] = first_row * (current[LANE_B + 4] - current[COL_S_ID]);
        result[C_S_INPUT + 3] = first_row * (current[LANE_B + 8] - current[COL_S_BAL_NEW]);

        result[C_R_INPUT] = sel_s_root * (next[4] - current[COL_R_ID]);
        result[C_R_INPUT + 1] = sel_s_root * (next[8] - current[COL_R_BAL]);
        result[C_R_INPUT + 2] = sel_s_root * (next[LANE_B + 4] - current[COL_R_ID]);
        result[C_R_INPUT + 3] = sel_s_root * (next[LANE_B + 8] - current[COL_R_BAL_NEW]);

        // ===== El puente de root_mid entre filas lejanas =====
        for i in 0..4 {
            result[C_MID_CAPTURE + i] =
                sel_s_root * (current[COL_ROOT_MID + i] - current[LANE_B + 4 + i]);
            result[C_MID_CHECK + i] = sel_r_root * (current[COL_ROOT_MID + i] - current[4 + i]);
        }

        // ===== CONSERVACIÓN DEL DINERO =====
        result[C_CONSERVATION] =
            current[COL_S_BAL_NEW] - (current[COL_S_BAL] - current[COL_AMT]);
        result[C_CONSERVATION + 1] =
            current[COL_R_BAL_NEW] - (current[COL_R_BAL] + current[COL_AMT]);

        // ===== Constancia del transporte =====
        let transport = [
            COL_S_ID,
            COL_S_BAL,
            COL_S_NONCE,
            COL_S_BAL_NEW,
            COL_R_ID,
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
            result[C_MID_CONST + i] = next[COL_ROOT_MID + i] - current[COL_ROOT_MID + i];
        }

        // ===== El account_id entra en el nullifier =====
        result[C_NULL_ID] = sel_r_root * (next[8] - current[COL_S_ID]);
        result[C_NULL_ID + 1] = sel_r_root * (next[LANE_B + 8] - current[COL_S_ID]);

        // ===== Solvencia (Horner big-endian, como en solvency.rs) =====
        let sbit_cur = current[COL_SBIT];
        let sbit_next = next[COL_SBIT];
        let sacc_cur = current[COL_SACC];
        let sacc_next = next[COL_SACC];

        result[C_SBIT_BOOL] = sbit_cur * (sbit_cur - E::ONE);
        result[C_SBIT_BOOL + 1] = sbit_next * (sbit_next - E::ONE);
        // El MSB de cada segmento es cero (rango de 63 bits sobre Goldilocks).
        result[C_FIRST_S] = first_s * sbit_cur;
        result[C_FIRST_S + 1] = first_s * sacc_cur;
        result[C_HORNER] = cont_s * (sacc_next - (sacc_cur + sacc_cur + sbit_next));

        // Cada acumulador completo se ata a su valor.
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
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        let zero = BaseElement::ZERO;
        let mut a = Vec::with_capacity(44);

        // Fila 0: capacidad y relleno de ambos carriles. Las posiciones
        // privadas (4 y 8) se atan con C_S_INPUT.
        for i in 0..4 {
            a.push(Assertion::single(i, 0, zero));
            a.push(Assertion::single(LANE_B + i, 0, zero));
        }
        for i in 5..8 {
            a.push(Assertion::single(i, 0, zero));
            a.push(Assertion::single(LANE_B + i, 0, zero));
        }
        for i in 9..12 {
            a.push(Assertion::single(i, 0, zero));
            a.push(Assertion::single(LANE_B + i, 0, zero));
        }
        // Raíces públicas.
        for i in 0..4 {
            a.push(Assertion::single(4 + i, ROW_S_ROOT, self.pub_inputs.root_old[i]));
            a.push(Assertion::single(
                LANE_B + 4 + i,
                ROW_R_ROOT,
                self.pub_inputs.root_new[i],
            ));
        }
        // Arranque del nullifier: la constante de dominio ANCLADA y el
        // relleno a cero. La posición privada (8 = account_id) se ata con
        // C_NULL_ID.
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
        for i in 9..12 {
            a.push(Assertion::single(i, ROW_NULL_START, zero));
        }
        // Nullifier público.
        for i in 0..4 {
            a.push(Assertion::single(
                4 + i,
                ROW_NULLIFIER,
                self.pub_inputs.nullifier[i],
            ));
        }
        // Límite regulatorio público, anclado en el transporte.
        a.push(Assertion::single(
            COL_LIM,
            0,
            self.pub_inputs.regulatory_limit,
        ));

        a
    }
}

pub struct DoubleEntryProver {
    options: ProofOptions,
}

impl DoubleEntryProver {
    pub fn new(options: ProofOptions) -> Self {
        Self { options }
    }
}

impl Prover for DoubleEntryProver {
    type BaseField = BaseElement;
    type Air = DoubleEntryAir;
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

    fn get_pub_inputs(&self, trace: &Self::Trace) -> DoubleEntryPublicInputs {
        DoubleEntryPublicInputs {
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

    fn digest_from(n: u64) -> Digest {
        [
            BaseElement::new(n),
            BaseElement::new(n + 1),
            BaseElement::new(n + 2),
            BaseElement::new(n + 3),
        ]
    }

    /// Hashes de subárboles VACÍOS por nivel. Con profundidad 32 no se
    /// puede materializar un árbol de 2^32 hojas; se usa uno DISPERSO.
    fn empty_subtrees() -> Vec<Digest> {
        let mut empty = vec![[BaseElement::ZERO; 4]];
        for k in 1..=TREE_DEPTH {
            let prev = empty[k - 1];
            empty.push(native_merge(prev, prev));
        }
        empty
    }

    struct Scenario {
        sender: PartyWitness,
        receiver: PartyWitness,
        amount: u64,
        credited: u64,
        limit: u64,
        public_inputs: DoubleEntryPublicInputs,
    }

    /// Escenario con un árbol disperso REAL: emisor en el índice 0 y
    /// receptor en el 1, hermanos en el nivel 0. El camino del receptor
    /// incluye la hoja NUEVA del emisor, que es lo que exige el
    /// encadenamiento `root_old → root_mid → root_new`.
    fn scenario(sender_balance: u64, amount: u64, credited: u64, limit: u64) -> Scenario {
        let empty = empty_subtrees();

        let s_id = BaseElement::new(1001);
        let s_nonce = BaseElement::new(7);
        let r_id = BaseElement::new(2002);
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

        let mut s_siblings = vec![r_leaf_old];
        let mut s_is_right = vec![false];
        let mut r_siblings = vec![s_leaf_new];
        let mut r_is_right = vec![true];
        for level in 1..TREE_DEPTH {
            s_siblings.push(empty[level]);
            s_is_right.push(false);
            r_siblings.push(empty[level]);
            r_is_right.push(false);
        }

        let sender_path = MerklePath {
            siblings: s_siblings,
            is_right: s_is_right,
        };
        let receiver_path = MerklePath {
            siblings: r_siblings,
            is_right: r_is_right,
        };

        Scenario {
            public_inputs: DoubleEntryPublicInputs {
                root_old: native_climb(s_leaf_old, &sender_path),
                root_new: native_climb(r_leaf_new, &receiver_path),
                regulatory_limit: BaseElement::new(limit),
                nullifier: native_nullifier(s_id, s_nonce),
            },
            sender: PartyWitness {
                account_id: s_id,
                balance: sender_balance,
                nonce: s_nonce,
                path: sender_path,
            },
            receiver: PartyWitness {
                account_id: r_id,
                balance: r_bal,
                nonce: r_nonce,
                path: receiver_path,
            },
            amount,
            credited,
            limit,
        }
    }

    fn run(s: &Scenario, declared: DoubleEntryPublicInputs) -> Result<(), String> {
        let trace = build_trace(&s.sender, &s.receiver, s.amount, s.credited, s.limit);
        let prover = DoubleEntryProver::new(default_options());

        let prove_result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| prover.prove(trace)));

        let proof = match prove_result {
            Err(_) => return Err("prove hizo panic (traza invalida en debug)".into()),
            Ok(Err(e)) => return Err(format!("prove devolvio Err: {e:?}")),
            Ok(Ok(p)) => p,
        };

        let min_opts = AcceptableOptions::OptionSet(vec![default_options()]);
        verify::<DoubleEntryAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof, declared, &min_opts,
        )
        .map_err(|e| format!("verificacion fallo: {e:?}"))
    }

    /// Estructura de la traza: raíces y nullifier en sus filas exactas.
    #[test]
    fn trace_landmarks_match_native_computation() {
        let s = scenario(1_000_000, 250_000, 250_000, 500_000);
        let trace = build_trace(&s.sender, &s.receiver, s.amount, s.credited, s.limit);

        for i in 0..4 {
            assert_eq!(
                trace.get(4 + i, ROW_S_ROOT),
                s.public_inputs.root_old[i],
                "root_old, elem {i}"
            );
            assert_eq!(
                trace.get(LANE_B + 4 + i, ROW_R_ROOT),
                s.public_inputs.root_new[i],
                "root_new, elem {i}"
            );
            assert_eq!(
                trace.get(4 + i, ROW_NULLIFIER),
                s.public_inputs.nullifier[i],
                "nullifier, elem {i}"
            );
        }
        // Los acumuladores de solvencia reconstruyen sus valores.
        assert_eq!(trace.get(COL_SACC, 63), BaseElement::new(1_000_000));
        assert_eq!(trace.get(COL_SACC, 191), BaseElement::new(250_000));
        // ===== Y TODAS LAS ENTRADAS PÚBLICAS, NO SOLO LAS RAÍCES =====
        //
        // Comparar la estructura entera. En `circuit_send` la versión
        // parcial dejó pasar un campo heredado de otra operación y **costó
        // ocho rondas de diagnóstico**: probador y verificador usaban
        // transcripciones de Fiat-Shamir distintas, y el error de winterfell
        // —`InconsistentOodConstraintEvaluations`— apunta a las
        // restricciones, no a las entradas.
        let derivadas = DoubleEntryProver::new(default_options()).get_pub_inputs(&trace);
        assert_eq!(
            derivadas.to_elements(),
            s.public_inputs.to_elements(),
            "las entradas DERIVADAS de la traza deben coincidir con las \
             DECLARADAS en todos sus campos"
        );

    }

    /// EL TEST CLAVE de todo el port.
    #[test]
    fn fully_valid_transfer_verifies() {
        let s = scenario(1_000_000, 250_000, 250_000, 500_000);
        let r = run(&s, s.public_inputs.clone());
        assert!(r.is_ok(), "una transferencia valida deberia verificar: {r:?}");
    }

    /// EL TEST QUE DA SENTIDO A LA PIEZA: creación de dinero.
    #[test]
    fn money_creation_is_rejected() {
        let s = scenario(1_000_000, 250_000, 260_000, 500_000);
        assert!(
            run(&s, s.public_inputs.clone()).is_err(),
            "CRITICO: acreditar mas de lo debitado debe rechazarse"
        );
    }

    /// Destrucción de dinero.
    #[test]
    fn money_destruction_is_rejected() {
        let s = scenario(1_000_000, 250_000, 240_000, 500_000);
        assert!(
            run(&s, s.public_inputs.clone()).is_err(),
            "CRITICO: acreditar menos de lo debitado debe rechazarse"
        );
    }

    /// Gastar más del saldo.
    #[test]
    fn insufficient_balance_is_rejected() {
        let s = scenario(100_000, 250_000, 250_000, 500_000);
        assert!(
            run(&s, s.public_inputs.clone()).is_err(),
            "CRITICO: gastar mas del saldo debe rechazarse"
        );
    }

    /// Superar el límite regulatorio.
    #[test]
    fn over_regulatory_limit_is_rejected() {
        let s = scenario(1_000_000, 750_000, 750_000, 500_000);
        assert!(
            run(&s, s.public_inputs.clone()).is_err(),
            "CRITICO: superar el limite debe rechazarse"
        );
    }

    /// Raíz final declarada incorrecta.
    #[test]
    fn wrong_declared_new_root_is_rejected() {
        let s = scenario(1_000_000, 250_000, 250_000, 500_000);
        let mut declared = s.public_inputs.clone();
        declared.root_new = digest_from(999_999);
        assert!(run(&s, declared).is_err());
    }

    /// Nullifier falsificado.
    #[test]
    fn forged_nullifier_is_rejected() {
        let s = scenario(1_000_000, 250_000, 250_000, 500_000);
        let mut declared = s.public_inputs.clone();
        declared.nullifier = digest_from(31_337);
        assert!(run(&s, declared).is_err());
    }
}
