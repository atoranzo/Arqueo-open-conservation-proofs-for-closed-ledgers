//! **Circuito de destrucción de circulante (burn)** en STARK.
//!
//! El simétrico del de emisión: cierra el ciclo monetario permitiendo
//! retirar dinero, no solo crearlo.
//!
//! ## Quién puede destruir, y por qué
//!
//! **La clave del TITULAR, no la del emisor.** El razonamiento es de
//! seguridad, no de preferencia:
//!
//! > **Destruir no puede crear dinero.** Reduce un saldo y el suministro
//! > en la misma cantidad, así que la invariante global —la suma de
//! > saldos equivale al suministro— se preserva sin que el emisor
//! > autorice nada.
//!
//! Exigir además la firma del emisor sería **política monetaria**, no una
//! garantía criptográfica. Mezclarlas en el circuito confundiría lo
//! imprescindible con lo operativo. Y tendría un efecto difícil de
//! justificar: que el titular no pudiera deshacerse de su propio saldo
//! sin permiso.
//!
//! Si un despliegue concreto quisiera control sobre la retirada de
//! circulante, sería una capa de política por encima — no una
//! restricción del circuito.
//!
//! ## Qué demuestra
//!
//! 1. **Titularidad**: quien firma conoce la clave de la cuenta.
//! 2. **La cuenta existe** en el árbol con el saldo declarado.
//! 3. **El saldo disminuye exactamente en el importe**, y no queda
//!    negativo.
//! 4. **El suministro disminuye exactamente en el importe.**
//!
//! ## Estructura
//!
//! Idéntica al circuito de emisión: dos carriles (saldo antes y después),
//! una subida del árbol en lockstep, y un ciclo final para derivar la
//! identidad desde la clave. 512 filas.

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
pub const TRACE_LENGTH: usize = 512;
pub const SEGMENT_LENGTH: usize = 64;
/// Segmentos: saldo, importe, saldo nuevo, suministro nuevo.
pub const NUM_SEGMENTS: usize = 4;

// ===== Columnas =====
const LANE_B: usize = STATE_WIDTH;
const COL_BIT: usize = 24;
/// Clave de gasto del TITULAR.
const COL_KEY: usize = 25;
const COL_ACC_ID: usize = 26; // 26..30
const COL_BAL: usize = 30;
const COL_BAL_NEW: usize = 31;
const COL_NONCE: usize = 32;
const COL_AMT: usize = 33;
const COL_SUPPLY_OLD: usize = 34;
const COL_SUPPLY_NEW: usize = 35;
const COL_SBIT: usize = 36;
const COL_SACC: usize = 37;
/// Bit de dirección del camino en el árbol de congelados.
const COL_FBIT: usize = 38;
pub const TRACE_WIDTH: usize = 39;

// ===== Filas =====
const ROW_LEAF_LINK: usize = 7;
const ROW_LEAF_DONE: usize = 15;
const ROW_ROOT: usize = 271;
const ROW_PK_START: usize = 272;
const ROW_PK_DONE: usize = 279;
/// **Fase de no-pertenencia al árbol de CONGELADOS.**
///
/// Ocupa las filas 280..471, que estaban libres. Sin ella, una cuenta
/// congelada **podía destruir su dinero**: la liquidación comprobaba la
/// congelación y la destrucción no.
///
/// Congelar existe para que una cuenta bajo investigación no mueva fondos.
/// Destruirlos los mueve: los saca del sistema. Que sea público e
/// irreversible no los devuelve.
const ROW_FROZEN_ROOT: usize = 471;

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
const C_KEY_INPUT: usize = C_INPUT + 10; // 2
/// **TITULARIDAD**: la identidad derivada coincide con la de la cuenta.
const C_PK_CHECK: usize = C_KEY_INPUT + 2; // 4
/// **EL SALDO DISMINUYE EXACTAMENTE EN EL IMPORTE.**
const C_BALANCE: usize = C_PK_CHECK + 4; // 1
/// **EL SUMINISTRO DISMINUYE EXACTAMENTE EN EL IMPORTE.**
const C_SUPPLY: usize = C_BALANCE + 1; // 1
const C_TRANSPORT: usize = C_SUPPLY + 1; // 7
const C_ID_CONST: usize = C_TRANSPORT + 7; // 4
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
const NUM_CONSTRAINTS: usize = C_FBIT_BOOL + 1;

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
    spend_key: BaseElement,
    account_id: Digest,
    balance: u64,
    nonce: BaseElement,
    path: &MerklePath,
    frozen_path: &MerklePath,
    amount: u64,
    supply_old: u64,
    supply_delta: u64,
) -> TraceTable<BaseElement> {
    let zero = BaseElement::ZERO;
    let c_bal = BaseElement::new(balance);
    let c_amt = BaseElement::new(amount);
    let c_bal_new = c_bal - c_amt;
    let c_supply_old = BaseElement::new(supply_old);
    let c_supply_new = c_supply_old - BaseElement::new(supply_delta);

    let mut rows: Vec<Vec<BaseElement>> = vec![vec![zero; TRACE_WIDTH]; TRACE_LENGTH];

    for row in rows.iter_mut() {
        row[COL_KEY] = spend_key;
        for i in 0..4 {
            row[COL_ACC_ID + i] = account_id[i];
        }
        row[COL_BAL] = c_bal;
        row[COL_BAL_NEW] = c_bal_new;
        row[COL_NONCE] = nonce;
        row[COL_AMT] = c_amt;
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

    let place_frozen = |state: &mut [BaseElement; STATE_WIDTH], digest: &Digest, level: usize| {
        if frozen_path.is_right[level] {
            state[4..8].copy_from_slice(&frozen_path.siblings[level]);
            state[8..12].copy_from_slice(digest);
        } else {
            state[4..8].copy_from_slice(digest);
            state[8..12].copy_from_slice(&frozen_path.siblings[level]);
        }
    };

    let place = |state: &mut [BaseElement; STATE_WIDTH], digest: &Digest, level: usize| {
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
                    state_a[8] = spend_key;
                    state_b[4] = BaseElement::new(SPEND_KEY_DOMAIN);
                    state_b[8] = spend_key;
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
                    if (2..34).contains(&next_cycle) {
                        place(&mut state_a, &digest_a, next_cycle - 2);
                        place(&mut state_b, &digest_b, next_cycle - 2);
                    } else if (36..60).contains(&next_cycle) {
                        let level = next_cycle - 35;
                        place_frozen(&mut state_a, &digest_a, level);
                        place_frozen(&mut state_b, &digest_b, level);
                    }
                }
            }
        }
        rows[r + 1][..STATE_WIDTH].copy_from_slice(&state_a);
        rows[r + 1][LANE_B..LANE_B + STATE_WIDTH].copy_from_slice(&state_b);
    }

    for level in 0..FROZEN_DEPTH {
        let bit = if frozen_path.is_right[level] {
            BaseElement::ONE
        } else {
            zero
        };
        for p in 0..CYCLE_LENGTH {
            rows[(35 + level) * CYCLE_LENGTH + p][COL_FBIT] = bit;
        }
    }

    for level in 0..TREE_DEPTH {
        let bit = if path.is_right[level] {
            BaseElement::ONE
        } else {
            zero
        };
        for p in 0..CYCLE_LENGTH {
            rows[(2 + level) * CYCLE_LENGTH + p][COL_BIT] = bit;
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
pub struct BurnPublicInputs {
    pub root_old: Digest,
    pub root_new: Digest,
    /// **Raíz del árbol de congelados.** La prueba acredita que el titular
    /// NO está en él: una cuenta congelada no puede destruir su dinero.
    pub frozen_root: Digest,
    pub amount: BaseElement,
    pub supply_old: BaseElement,
    pub supply_new: BaseElement,
}

impl ToElements<BaseElement> for BurnPublicInputs {
    fn to_elements(&self) -> Vec<BaseElement> {
        let mut out = self.root_old.to_vec();
        out.extend_from_slice(&self.root_new);
        out.extend_from_slice(&self.frozen_root);
        out.push(self.amount);
        out.push(self.supply_old);
        out.push(self.supply_new);
        out
    }
}

pub struct BurnAir {
    context: AirContext<BaseElement>,
    pub_inputs: BurnPublicInputs,
}

impl Air for BurnAir {
    type BaseField = BaseElement;
    type PublicInputs = BurnPublicInputs;

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
        for _ in 0..34 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }
        // Saldo (1), suministro (1), transporte (7), identidad (4).
        for _ in 0..13 {
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

        assert_eq!(degrees.len(), NUM_CONSTRAINTS, "cuenta de grados");

        BurnAir {
            context: AirContext::new(trace_info, degrees, 33, options),
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
        for r in 0..=ROW_FROZEN_ROOT {
            if r % CYCLE_LENGTH < NUM_ROUNDS {
                hash_flag[r] = one;
            }
        }
        columns.push(hash_flag);

        for ark in [true, false] {
            for i in 0..STATE_WIDTH {
                let mut col = vec![zero; TRACE_LENGTH];
                for r in 0..=ROW_FROZEN_ROOT {
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
            frozen_link[(35 + level) * CYCLE_LENGTH + 7] = one;
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

        result[C_KEY_INPUT] = sel_root * (next[8] - current[COL_KEY]);
        result[C_KEY_INPUT + 1] = sel_root * (next[LANE_B + 8] - current[COL_KEY]);

        // ===== TITULARIDAD =====
        // La identidad derivada de la clave es la de la cuenta. Sin la
        // clave del titular no se puede destruir su saldo.
        for i in 0..4 {
            result[C_PK_CHECK + i] = sel_pk_done * (current[4 + i] - current[COL_ACC_ID + i]);
        }

        // ===== EL SALDO DISMINUYE EXACTAMENTE EN EL IMPORTE =====
        result[C_BALANCE] = current[COL_BAL_NEW] - (current[COL_BAL] - current[COL_AMT]);

        // ===== EL SUMINISTRO DISMINUYE EXACTAMENTE EN EL IMPORTE =====
        result[C_SUPPLY] =
            current[COL_SUPPLY_NEW] - (current[COL_SUPPLY_OLD] - current[COL_AMT]);

        let transport = [
            COL_KEY,
            COL_BAL,
            COL_BAL_NEW,
            COL_NONCE,
            COL_AMT,
            COL_SUPPLY_OLD,
            COL_SUPPLY_NEW,
        ];
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

        a
    }
}

pub struct BurnProver {
    options: ProofOptions,
}

impl BurnProver {
    pub fn new(options: ProofOptions) -> Self {
        Self { options }
    }
}

impl Prover for BurnProver {
    type BaseField = BaseElement;
    type Air = BurnAir;
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

    fn get_pub_inputs(&self, trace: &Self::Trace) -> BurnPublicInputs {
        BurnPublicInputs {
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
    use crate::circuit_settlement::{derive_public_id, native_climb, native_leaf};
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
        key: BaseElement,
        account_id: Digest,
        balance: u64,
        nonce: BaseElement,
        path: MerklePath,
        frozen_path: MerklePath,
        amount: u64,
        supply_old: u64,
        public_inputs: BurnPublicInputs,
    }

    fn scenario(balance: u64, amount: u64, supply_old: u64) -> Scenario {
        let mut empty = vec![[BaseElement::ZERO; 4]];
        for k in 1..=TREE_DEPTH {
            let prev = empty[k - 1];
            empty.push(native_merge(prev, prev));
        }
        let key = BaseElement::new(SK);
        let account_id = derive_public_id(key);
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
        let leaf_new = native_leaf(
            account_id,
            BaseElement::new(balance) - BaseElement::new(amount),
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

        Scenario {
            public_inputs: BurnPublicInputs {
                root_old: native_climb(leaf_old, &path),
                root_new: native_climb(leaf_new, &path),
                amount: BaseElement::new(amount),
                supply_old: BaseElement::new(supply_old),
                supply_new: BaseElement::new(supply_old - amount),
                frozen_root,
            },
            key,
            account_id,
            balance,
            nonce,
            path,
            frozen_path,
            amount,
            supply_old,
        }
    }

    fn run(s: &Scenario, key: BaseElement, supply_delta: u64) -> Result<(), String> {
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
        );
        let prover = BurnProver::new(default_options());

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
        verify::<BurnAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof,
            s.public_inputs.clone(),
            &min_opts,
        )
        .map_err(|e| format!("verificacion fallo: {e:?}"))
    }

    /// EL TEST CLAVE.
    #[test]
    fn authorized_burn_verifies() {
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
            s.amount,
        );
        let prover = BurnProver::new(default_options());
        let proof = prover.prove(trace).expect("la destruccion valida deberia probar");
        let min_opts = AcceptableOptions::OptionSet(vec![default_options()]);
        let v = verify::<BurnAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
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
    fn third_party_cannot_burn_someone_elses_money() {
        let s = scenario(1_000_000, 250_000, 10_000_000);
        assert!(
            run(&s, BaseElement::new(0x1337), s.amount).is_err(),
            "CRITICO: sin la clave del titular no debe poder destruirse su saldo"
        );
    }

    /// **DESTRUCCIÓN ENCUBIERTA.**
    ///
    /// Se reduce un saldo sin reflejarlo en el suministro. El dinero
    /// desaparecería pero la cifra auditable no lo registraría, y la
    /// invariante global se rompería en silencio.
    #[test]
    fn burning_without_updating_supply_is_rejected() {
        let s = scenario(1_000_000, 250_000, 10_000_000);
        assert!(
            run(&s, s.key, 0).is_err(),
            "CRITICO: destruir sin registrarlo en el suministro romperia la \
             invariante global"
        );
    }

    /// Reducir el suministro más de lo destruido tampoco cuela.
    #[test]
    fn deflating_supply_beyond_amount_is_rejected() {
        let s = scenario(1_000_000, 250_000, 10_000_000);
        assert!(run(&s, s.key, 500_000).is_err());
    }

    /// **NO SE PUEDE DESTRUIR MÁS DE LO QUE HAY.**
    ///
    /// El saldo resultante daría la vuelta en el campo y no cabría en el
    /// rango de 64 bits.
    #[test]
    fn burning_more_than_the_balance_is_rejected() {
        let s = scenario(100_000, 250_000, 10_000_000);
        assert!(
            run(&s, s.key, s.amount).is_err(),
            "CRITICO: destruir mas del saldo disponible debe rechazarse"
        );
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
            s.amount,
        );
        let prover = BurnProver::new(default_options());
        let proof = prover.prove(trace).expect("prove");
        let min_opts = AcceptableOptions::OptionSet(vec![default_options()]);
        let v = verify::<BurnAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
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
    fn a_frozen_account_cannot_burn() {
        let mut s = scenario(1_000_000, 250_000, 10_000_000);

        // La cuenta SI esta en el arbol de congelados.
        let hoja = crate::circuit_freeze::frozen_leaf(true);
        s.public_inputs.frozen_root =
            crate::circuit_freeze::frozen_climb(hoja, &s.frozen_path);

        assert!(
            run(&s, s.key, s.amount).is_err(),
            "CRITICO: una cuenta congelada no debe poder destruir su dinero"
        );
    }

    /// **Y el que valida al anterior**: una cuenta libre SÍ puede.
    ///
    /// Sin esto, el test anterior pasaría aunque la fase de congelados
    /// rechazara todo — o no impusiera nada y fallara por otra razón.
    #[test]
    fn a_free_account_can_burn() {
        let s = scenario(1_000_000, 250_000, 10_000_000);
        assert!(
            run(&s, s.key, s.amount).is_ok(),
            "una cuenta libre debe poder destruir su dinero"
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
        assert!(run(&s, s.key, s.amount).is_err());
    }

    /// **SEPARA "LA TRAZA ESTÁ MAL" DE "LAS RESTRICCIONES ESTÁN MAL".**
    ///
    /// Este circuito **no tenía ningún test de puntos de referencia**, pese
    /// a estar en producción. Se detectó al inventariar cuáles comparaban
    /// sus entradas públicas con lo que la traza produce.
    ///
    /// Compara la **estructura entera**, no los campos que parecen
    /// importantes: en `circuit_send` la versión parcial dejó pasar un
    /// campo heredado de otra operación y **costó ocho rondas de
    /// diagnóstico**, porque el error de winterfell
    /// —`InconsistentOodConstraintEvaluations`— apunta a las restricciones
    /// y no a las entradas.
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
            s.amount,
        );

        for i in 0..4 {
            assert_eq!(
                trace.get(4 + i, ROW_ROOT),
                s.public_inputs.root_old[i],
                "raiz de cuentas ANTES, elemento {i}"
            );
            assert_eq!(
                trace.get(LANE_B + 4 + i, ROW_ROOT),
                s.public_inputs.root_new[i],
                "raiz de cuentas DESPUES, elemento {i}"
            );
            assert_eq!(
                trace.get(4 + i, ROW_FROZEN_ROOT),
                s.public_inputs.frozen_root[i],
                "raiz de congelados, elemento {i}"
            );
        }

        // Y TODAS las entradas públicas, no solo las raíces.
        let derivadas = BurnProver::new(default_options()).get_pub_inputs(&trace);
        assert_eq!(
            derivadas.to_elements(),
            s.public_inputs.to_elements(),
            "las entradas DERIVADAS de la traza deben coincidir con las \
             DECLARADAS en todos sus campos"
        );
    }

    /// **PRUEBA POR MUTACIÓN: ninguna restricción está vacía.**
    ///
    /// Si **ninguna perturbación** de una celda hace que una restricción se
    /// vuelva no nula, esa restricción no impone nada — y ningún test
    /// normal lo detecta: el testigo honesto la satisface (vale cero, como
    /// debe) y los adversariales fallan por otras antes de llegar a ella.
    ///
    /// Este proyecto ha visto ese modo de fallo **tres veces**: una
    /// restricción idénticamente cero, siete columnas declaradas y nunca
    /// rellenadas, y un tope transportado pero sin comprobar. Ver
    /// `AUDITORIA.md`.
    ///
    /// ⚠️ **Un resultado limpio no significa que el circuito sea correcto**:
    /// significa que no tiene este fallo concreto. No detecta restricciones
    /// que se disparan pero imponen lo que no se cree.
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
            s.amount,
        );
        let rows = rows_of(&trace, TRACE_WIDTH, TRACE_LENGTH);

        let air = BurnAir::new(
            TraceInfo::new(TRACE_WIDTH, TRACE_LENGTH),
            s.public_inputs.clone(),
            default_options(),
        );

        // **TODAS las filas**, sin muestrear.
        //
        // Con muestreo, una restricción activa solo en filas no muestreadas
        // aparece como vacía sin serlo. Aquí el coste es asumible:
        // 39 columnas x 512 filas x 2 evaluaciones.
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
