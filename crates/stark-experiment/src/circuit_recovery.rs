//! **Recuperación de cuenta**: la única carencia que causaba pérdida
//! irreversible.
//!
//! ## El problema
//!
//! Si una clave de gasto se comprometía, **el dinero de esa cuenta se
//! perdía para siempre**. No había recuperación, revocación ni nada. Las
//! demás carencias del sistema degradan el servicio; esta destruía valor.
//!
//! ## Por qué la rotación voluntaria no lo resuelve
//!
//! Cambiar la clave usando la clave actual sirve para higiene, pero **no
//! para el compromiso**: si el atacante la tiene, también puede rotar, y
//! ganaría la carrera.
//!
//! Lo único que resuelve el compromiso es **recuperación asistida por los
//! custodios**, que el sistema ya tiene construidos.
//!
//! ## ⚠️ El precio, dicho sin adornos
//!
//! Esto significa que **dos custodios pueden reasignar cualquier cuenta**.
//! Se cambia *"pérdida irreversible si te roban la clave"* por *"los
//! custodios pueden apoderarse de una cuenta"*.
//!
//! Es un intercambio real. En un sistema bancario es el correcto —un
//! banco puede bloquear y reasignar bajo orden judicial— **pero exige que
//! sea visible**.
//!
//! ## El contador de recuperaciones
//!
//! Por eso hay un **contador público** que incrementa en cada
//! recuperación, atado en el circuito.
//!
//! Sin él, los custodios podrían reasignar cuentas en silencio: desde
//! fuera, una recuperación es indistinguible de cualquier otra transición
//! de estado. Con él, **cada intervención queda contada**, y una discrepancia
//! entre el contador y las recuperaciones justificadas es detectable.
//!
//! No impide el abuso —nada en un circuito puede— pero lo hace **contable**,
//! que es la condición para que exista rendición de cuentas.
//!
//! ## Qué demuestra
//!
//! 1. **Autoridad de umbral**: dos custodios distintos del conjunto.
//! 2. **La cuenta existe** en el árbol con su identidad antigua.
//! 3. **El saldo NO cambia.** Una recuperación reasigna el control, no
//!    mueve dinero.
//! 4. **La identidad cambia** a la nueva, en la misma posición del árbol.
//! 5. **El contador de recuperaciones incrementa** exactamente en uno.
//!
//! ## Lo que NO resuelve
//!
//! - **El circuito no verifica que el nuevo titular sea legítimo.** Eso
//!   lo comprueban los custodios fuera de línea; la criptografía no puede
//!   saber quién es el dueño real de una cuenta.
//! - **No hay revocación del conjunto de custodios.** Si un custodio se
//!   compromete, sigue siendo custodio. Esa es otra pieza.
//! - **Nada impide una carrera**: si el atacante gasta antes de que la
//!   recuperación se aplique, el dinero se va. La recuperación protege el
//!   saldo restante, no lo ya gastado.

use winterfell::crypto::hashers::{Blake3_256, Rp64_256};
use winterfell::crypto::{DefaultRandomCoin, MerkleTree};
use winterfell::math::{fields::f64::BaseElement, FieldElement, StarkField, ToElements};
use winterfell::matrix::ColMatrix;
use winterfell::{
    Air, AirContext, Assertion, AuxRandElements, CompositionPoly, CompositionPolyTrace,
    ConstraintCompositionCoefficients, DefaultConstraintCommitment, DefaultConstraintEvaluator,
    DefaultTraceLde, EvaluationFrame, PartitionOptions, ProofOptions, Prover, StarkDomain,
    TraceInfo, TracePolyTable, TraceTable, TransitionConstraintDegree,
};

use crate::circuit_mint::ThresholdAuth;
use crate::circuit_threshold::{CustodianPath, CUSTODIAN_DEPTH, CUSTODIAN_DOMAIN};
use crate::merkle::{Digest, MerklePath, TREE_DEPTH};
use crate::rescue_hash::{apply_sbox, NUM_ROUNDS, STATE_WIDTH};

pub const CYCLE_LENGTH: usize = 8;
pub const TRACE_LENGTH: usize = 512;
pub const SEGMENT_LENGTH: usize = 64;
/// Segmentos: saldo, índice A, índice B, y `B − A − 1`.
pub const NUM_SEGMENTS: usize = 4;

// ===== Columnas =====
const LANE_B: usize = STATE_WIDTH; // 12
const COL_BIT_A: usize = 24;
const COL_BIT_B: usize = 25;
const COL_KEY_A: usize = 26;
const COL_KEY_B: usize = 27;
const COL_IDX_A: usize = 28;
const COL_IDX_B: usize = 29;
const COL_ACC_A: usize = 30;
const COL_ACC_B: usize = 31;
/// Identidad de la cuenta ANTES de la recuperación.
const COL_ID_OLD: usize = 32; // 32..36
/// Identidad DESPUÉS. La posición en el árbol no cambia.
const COL_ID_NEW: usize = 36; // 36..40
const COL_BAL: usize = 40;
const COL_NONCE: usize = 41;
/// Contador público de recuperaciones. Lo que hace **contables** las
/// intervenciones de los custodios.
const COL_COUNT_OLD: usize = 42;
const COL_COUNT_NEW: usize = 43;
const COL_SBIT: usize = 44;
const COL_SACC: usize = 45;
pub const TRACE_WIDTH: usize = 46;

// ===== Filas =====
const ROW_LEAF_LINK: usize = 7;
const ROW_LEAF_DONE: usize = 15;
const ROW_ACCT_ROOT: usize = 271;
const ROW_CUST_START: usize = 272;
const ROW_CUST_ROOT: usize = 311;

// ===== Restricciones =====
const C_HASH_A: usize = 0;
const C_HASH_B: usize = C_HASH_A + STATE_WIDTH;
const C_CAP_A: usize = C_HASH_B + STATE_WIDTH; // 4
const C_CAP_B: usize = C_CAP_A + 4;
const C_PLACE_A: usize = C_CAP_B + 4; // 4
const C_PLACE_B: usize = C_PLACE_A + 4;
const C_SIBLING: usize = C_PLACE_B + 4; // 4
const C_CPLACE_A: usize = C_SIBLING + 4; // 4
const C_CPLACE_B: usize = C_CPLACE_A + 4;
const C_BIT_BOOL: usize = C_CPLACE_B + 4; // 2
const C_LEAF_CAP_A: usize = C_BIT_BOOL + 2; // 4
const C_LEAF_CAP_B: usize = C_LEAF_CAP_A + 4;
const C_LEAF_DIG_A: usize = C_LEAF_CAP_B + 4; // 4
const C_LEAF_DIG_B: usize = C_LEAF_DIG_A + 4;
/// El nonce incrementa: marca que la cuenta cambió de control.
const C_NONCE: usize = C_LEAF_DIG_B + 4; // 2
/// Entradas: identidad antigua + saldo (A), identidad nueva + **el mismo
/// saldo** (B).
const C_INPUT: usize = C_NONCE + 2; // 10
const C_CUST_INPUT: usize = C_INPUT + 10; // 2
const C_ACC: usize = C_CUST_INPUT + 2; // 2
const C_ACC_FINAL: usize = C_ACC + 2; // 2
/// **EL CONTADOR INCREMENTA EXACTAMENTE EN UNO.**
const C_COUNT: usize = C_ACC_FINAL + 2; // 1
const C_TRANSPORT: usize = C_COUNT + 1; // 8
const C_ID_CONST: usize = C_TRANSPORT + 8; // 8
const C_SBIT_BOOL: usize = C_ID_CONST + 8; // 2
const C_FIRST_S: usize = C_SBIT_BOOL + 2; // 2
const C_HORNER: usize = C_FIRST_S + 2; // 1
const C_SEG_LINK: usize = C_HORNER + 1; // NUM_SEGMENTS
const NUM_CONSTRAINTS: usize = C_SEG_LINK + NUM_SEGMENTS;

// ===== Periódicas =====
const P_HASH_FLAG: usize = 0;
const P_ARK1: usize = 1;
const P_ARK2: usize = P_ARK1 + STATE_WIDTH;
const P_ACCT_LINK: usize = P_ARK2 + STATE_WIDTH;
const P_LINK_LEAF: usize = P_ACCT_LINK + 1;
const P_CUST_LINK: usize = P_LINK_LEAF + 1;
const P_POW2: usize = P_CUST_LINK + 1;
const P_FIRST_ROW: usize = P_POW2 + 1;
const P_SEL_ACCT_ROOT: usize = P_FIRST_ROW + 1;
const P_SEL_CUST_ROOT: usize = P_SEL_ACCT_ROOT + 1;
const P_FIRST_S: usize = P_SEL_CUST_ROOT + 1;
const P_CONT_S: usize = P_FIRST_S + 1;
const P_SEG_LINK: usize = P_CONT_S + 1;

type Blake3 = Blake3_256<BaseElement>;

fn value_to_bits_be(value: u64) -> Vec<bool> {
    (0..SEGMENT_LENGTH)
        .map(|p| (value >> (SEGMENT_LENGTH - 1 - p)) & 1 == 1)
        .collect()
}

/// Construye la traza de una recuperación.
///
/// `balance_new` permite alterar el saldo, para el test que comprueba que
/// una recuperación **no puede mover dinero**.
#[allow(clippy::too_many_arguments)]
pub fn build_trace(
    auth: &ThresholdAuth,
    id_old: Digest,
    id_new: Digest,
    balance: u64,
    balance_new: u64,
    nonce: BaseElement,
    path: &MerklePath,
    count_old: u64,
    count_delta: u64,
) -> TraceTable<BaseElement> {
    let zero = BaseElement::ZERO;
    let c_bal = BaseElement::new(balance);
    let c_bal_new = BaseElement::new(balance_new);
    let nonce_new = nonce + BaseElement::ONE;
    let c_count_old = BaseElement::new(count_old);
    let c_count_new = c_count_old + BaseElement::new(count_delta);

    let mut rows: Vec<Vec<BaseElement>> = vec![vec![zero; TRACE_WIDTH]; TRACE_LENGTH];

    for row in rows.iter_mut() {
        row[COL_KEY_A] = auth.key_a;
        row[COL_KEY_B] = auth.key_b;
        row[COL_IDX_A] = BaseElement::new(auth.index_a);
        row[COL_IDX_B] = BaseElement::new(auth.index_b);
        for i in 0..4 {
            row[COL_ID_OLD + i] = id_old[i];
            row[COL_ID_NEW + i] = id_new[i];
        }
        row[COL_BAL] = c_bal;
        row[COL_NONCE] = nonce;
        row[COL_COUNT_OLD] = c_count_old;
        row[COL_COUNT_NEW] = c_count_new;
    }

    let diff = BaseElement::new(auth.index_b) - BaseElement::new(auth.index_a) - BaseElement::ONE;
    let segment_values = [
        c_bal.as_int(),
        auth.index_a,
        auth.index_b,
        diff.as_int(),
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

    let place_acct = |state: &mut [BaseElement; STATE_WIDTH], digest: &Digest, level: usize| {
        if path.is_right[level] {
            state[4..8].copy_from_slice(&path.siblings[level]);
            state[8..12].copy_from_slice(digest);
        } else {
            state[4..8].copy_from_slice(digest);
            state[8..12].copy_from_slice(&path.siblings[level]);
        }
    };
    let place_cust = |state: &mut [BaseElement; STATE_WIDTH],
                      digest: &Digest,
                      p: &CustodianPath,
                      level: usize| {
        if p.is_right[level] {
            state[4..8].copy_from_slice(&p.siblings[level]);
            state[8..12].copy_from_slice(digest);
        } else {
            state[4..8].copy_from_slice(digest);
            state[8..12].copy_from_slice(&p.siblings[level]);
        }
    };

    let mut state_a = [zero; STATE_WIDTH];
    let mut state_b = [zero; STATE_WIDTH];
    // Carril A: identidad antigua. Carril B: identidad nueva, MISMO saldo.
    state_a[4..8].copy_from_slice(&id_old);
    state_a[8] = c_bal;
    state_b[4..8].copy_from_slice(&id_new);
    state_b[8] = c_bal_new;

    rows[0][..STATE_WIDTH].copy_from_slice(&state_a);
    rows[0][LANE_B..LANE_B + STATE_WIDTH].copy_from_slice(&state_b);

    let mut acc_a = zero;
    let mut acc_b = zero;

    for r in 0..ROW_CUST_ROOT {
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
                    state_a[4..8].copy_from_slice(&digest_a);
                    state_a[8] = nonce;
                    state_b[4..8].copy_from_slice(&digest_b);
                    state_b[8] = nonce_new;
                }
                ROW_LEAF_DONE => {
                    place_acct(&mut state_a, &digest_a, 0);
                    place_acct(&mut state_b, &digest_b, 0);
                }
                ROW_ACCT_ROOT => {
                    state_a[4] = BaseElement::new(CUSTODIAN_DOMAIN);
                    state_a[8] = auth.key_a;
                    state_b[4] = BaseElement::new(CUSTODIAN_DOMAIN);
                    state_b[8] = auth.key_b;
                }
                _ => {
                    let next_cycle = (r + 1) / CYCLE_LENGTH;
                    if (2..34).contains(&next_cycle) {
                        let level = next_cycle - 2;
                        place_acct(&mut state_a, &digest_a, level);
                        place_acct(&mut state_b, &digest_b, level);
                    } else if (35..39).contains(&next_cycle) {
                        let level = next_cycle - 35;
                        place_cust(&mut state_a, &digest_a, &auth.path_a, level);
                        place_cust(&mut state_b, &digest_b, &auth.path_b, level);
                        let p = BaseElement::new(1u64 << level);
                        if auth.path_a.is_right[level] {
                            acc_a += p;
                        }
                        if auth.path_b.is_right[level] {
                            acc_b += p;
                        }
                    }
                }
            }
        }
        rows[r + 1][..STATE_WIDTH].copy_from_slice(&state_a);
        rows[r + 1][LANE_B..LANE_B + STATE_WIDTH].copy_from_slice(&state_b);
        rows[r + 1][COL_ACC_A] = acc_a;
        rows[r + 1][COL_ACC_B] = acc_b;
    }
    for r in ROW_CUST_ROOT..TRACE_LENGTH {
        rows[r][COL_ACC_A] = acc_a;
        rows[r][COL_ACC_B] = acc_b;
    }

    for level in 0..TREE_DEPTH {
        let bit = if path.is_right[level] {
            BaseElement::ONE
        } else {
            zero
        };
        for p in 0..CYCLE_LENGTH {
            rows[(2 + level) * CYCLE_LENGTH + p][COL_BIT_A] = bit;
        }
    }
    for level in 0..CUSTODIAN_DEPTH {
        let ba = if auth.path_a.is_right[level] {
            BaseElement::ONE
        } else {
            zero
        };
        let bb = if auth.path_b.is_right[level] {
            BaseElement::ONE
        } else {
            zero
        };
        for p in 0..CYCLE_LENGTH {
            rows[(35 + level) * CYCLE_LENGTH + p][COL_BIT_A] = ba;
            rows[(35 + level) * CYCLE_LENGTH + p][COL_BIT_B] = bb;
        }
    }

    let mut trace = TraceTable::new(TRACE_WIDTH, TRACE_LENGTH);
    trace.fill(
        |s| s.copy_from_slice(&rows[0]),
        |step, s| s.copy_from_slice(&rows[step + 1]),
    );
    trace
}

#[derive(Clone, Debug)]
pub struct RecoveryPublicInputs {
    pub root_old: Digest,
    pub root_new: Digest,
    pub custodian_set_root: Digest,
    /// Contador de recuperaciones ANTES.
    pub recovery_count_old: BaseElement,
    /// Y DESPUÉS: siempre `old + 1`. Lo que hace contables las
    /// intervenciones de los custodios.
    pub recovery_count_new: BaseElement,
}

impl ToElements<BaseElement> for RecoveryPublicInputs {
    fn to_elements(&self) -> Vec<BaseElement> {
        let mut out = self.root_old.to_vec();
        out.extend_from_slice(&self.root_new);
        out.extend_from_slice(&self.custodian_set_root);
        out.push(self.recovery_count_old);
        out.push(self.recovery_count_new);
        out
    }
}

pub struct RecoveryAir {
    context: AirContext<BaseElement>,
    pub_inputs: RecoveryPublicInputs,
}

impl Air for RecoveryAir {
    type BaseField = BaseElement;
    type PublicInputs = RecoveryPublicInputs;

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
        // Colocacion de cuentas (8) + hermano (4) + custodios (8).
        for _ in 0..20 {
            degrees.push(TransitionConstraintDegree::with_cycles(2, full.clone()));
        }
        for _ in 0..2 {
            degrees.push(TransitionConstraintDegree::new(2));
        }
        // Hojas (16), nonce (2), entradas (10), claves (2).
        for _ in 0..30 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }
        // Acumulador: DOS periodicas.
        for _ in 0..2 {
            degrees.push(TransitionConstraintDegree::with_cycles(
                1,
                vec![TRACE_LENGTH, TRACE_LENGTH],
            ));
        }
        for _ in 0..2 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }
        // Contador (1), transporte (8), identidades (8): sin ciclo.
        for _ in 0..17 {
            degrees.push(TransitionConstraintDegree::new(1));
        }
        for _ in 0..2 {
            degrees.push(TransitionConstraintDegree::new(2));
        }
        for _ in 0..(3 + NUM_SEGMENTS) {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }

        assert_eq!(degrees.len(), NUM_CONSTRAINTS, "cuenta de grados");

        RecoveryAir {
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
        for r in 0..=ROW_CUST_ROOT {
            if r % CYCLE_LENGTH < NUM_ROUNDS {
                hash_flag[r] = one;
            }
        }
        columns.push(hash_flag);

        for ark in [true, false] {
            for i in 0..STATE_WIDTH {
                let mut col = vec![zero; TRACE_LENGTH];
                for r in 0..=ROW_CUST_ROOT {
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

        let mut acct_link = vec![zero; TRACE_LENGTH];
        acct_link[ROW_LEAF_DONE] = one;
        for level in 0..TREE_DEPTH - 1 {
            acct_link[(2 + level) * CYCLE_LENGTH + 7] = one;
        }
        columns.push(acct_link);

        let mut link_leaf = vec![zero; TRACE_LENGTH];
        link_leaf[ROW_LEAF_LINK] = one;
        columns.push(link_leaf);

        let mut cust_link = vec![zero; TRACE_LENGTH];
        let mut pow2 = vec![zero; TRACE_LENGTH];
        for level in 0..CUSTODIAN_DEPTH {
            let row = (34 + level) * CYCLE_LENGTH + 7;
            cust_link[row] = one;
            pow2[row] = BaseElement::new(1u64 << level);
        }
        columns.push(cust_link);
        columns.push(pow2);

        for row in [0, ROW_ACCT_ROOT, ROW_CUST_ROOT] {
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
        let acct_link = periodic[P_ACCT_LINK];
        let link_leaf = periodic[P_LINK_LEAF];
        let cust_link = periodic[P_CUST_LINK];
        let pow2 = periodic[P_POW2];
        let first_row = periodic[P_FIRST_ROW];
        let sel_acct_root = periodic[P_SEL_ACCT_ROOT];
        let sel_cust_root = periodic[P_SEL_CUST_ROOT];
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

        let bit_a = next[COL_BIT_A];
        let bit_b = next[COL_BIT_B];
        let any_link = acct_link + cust_link;

        for i in 0..4 {
            result[C_CAP_A + i] = any_link * next[i];
            result[C_CAP_B + i] = any_link * next[LANE_B + i];

            let da = current[4 + i];
            let db = current[LANE_B + 4 + i];

            result[C_PLACE_A + i] =
                acct_link * ((E::ONE - bit_a) * (next[4 + i] - da) + bit_a * (next[8 + i] - da));
            result[C_PLACE_B + i] = acct_link
                * ((E::ONE - bit_a) * (next[LANE_B + 4 + i] - db)
                    + bit_a * (next[LANE_B + 8 + i] - db));

            let sib_a = (E::ONE - bit_a) * next[8 + i] + bit_a * next[4 + i];
            let sib_b =
                (E::ONE - bit_a) * next[LANE_B + 8 + i] + bit_a * next[LANE_B + 4 + i];
            result[C_SIBLING + i] = acct_link * (sib_a - sib_b);

            result[C_CPLACE_A + i] =
                cust_link * ((E::ONE - bit_a) * (next[4 + i] - da) + bit_a * (next[8 + i] - da));
            result[C_CPLACE_B + i] = cust_link
                * ((E::ONE - bit_b) * (next[LANE_B + 4 + i] - db)
                    + bit_b * (next[LANE_B + 8 + i] - db));
        }

        result[C_BIT_BOOL] = current[COL_BIT_A] * (current[COL_BIT_A] - E::ONE);
        result[C_BIT_BOOL + 1] = current[COL_BIT_B] * (current[COL_BIT_B] - E::ONE);

        for i in 0..4 {
            result[C_LEAF_CAP_A + i] = link_leaf * next[i];
            result[C_LEAF_CAP_B + i] = link_leaf * next[LANE_B + i];
            result[C_LEAF_DIG_A + i] = link_leaf * (next[4 + i] - current[4 + i]);
            result[C_LEAF_DIG_B + i] =
                link_leaf * (next[LANE_B + 4 + i] - current[LANE_B + 4 + i]);
        }

        // El nonce incrementa: marca que la cuenta cambió de control.
        result[C_NONCE] = link_leaf * (next[8] - current[COL_NONCE]);
        result[C_NONCE + 1] = link_leaf * (next[LANE_B + 8] - (current[COL_NONCE] + E::ONE));

        // ===== EL SALDO NO CAMBIA =====
        // Ambos carriles usan la MISMA columna de saldo. Una recuperación
        // reasigna el control, no mueve dinero.
        for i in 0..4 {
            result[C_INPUT + i] = first_row * (current[4 + i] - current[COL_ID_OLD + i]);
            result[C_INPUT + 5 + i] =
                first_row * (current[LANE_B + 4 + i] - current[COL_ID_NEW + i]);
        }
        result[C_INPUT + 4] = first_row * (current[8] - current[COL_BAL]);
        result[C_INPUT + 9] = first_row * (current[LANE_B + 8] - current[COL_BAL]);

        result[C_CUST_INPUT] = sel_acct_root * (next[8] - current[COL_KEY_A]);
        result[C_CUST_INPUT + 1] = sel_acct_root * (next[LANE_B + 8] - current[COL_KEY_B]);

        result[C_ACC] = cust_link * (next[COL_ACC_A] - (current[COL_ACC_A] + bit_a * pow2));
        result[C_ACC + 1] = cust_link * (next[COL_ACC_B] - (current[COL_ACC_B] + bit_b * pow2));

        result[C_ACC_FINAL] = sel_cust_root * (current[COL_ACC_A] - current[COL_IDX_A]);
        result[C_ACC_FINAL + 1] = sel_cust_root * (current[COL_ACC_B] - current[COL_IDX_B]);

        // ===== EL CONTADOR INCREMENTA EXACTAMENTE EN UNO =====
        // Sin esto, los custodios podrían reasignar cuentas en silencio:
        // desde fuera, una recuperación sería indistinguible de cualquier
        // otra transición de estado.
        result[C_COUNT] =
            current[COL_COUNT_NEW] - (current[COL_COUNT_OLD] + E::ONE);

        let transport = [
            COL_KEY_A,
            COL_KEY_B,
            COL_IDX_A,
            COL_IDX_B,
            COL_BAL,
            COL_NONCE,
            COL_COUNT_OLD,
            COL_COUNT_NEW,
        ];
        for (k, col) in transport.iter().enumerate() {
            result[C_TRANSPORT + k] = next[*col] - current[*col];
        }
        for i in 0..4 {
            result[C_ID_CONST + i] = next[COL_ID_OLD + i] - current[COL_ID_OLD + i];
            result[C_ID_CONST + 4 + i] = next[COL_ID_NEW + i] - current[COL_ID_NEW + i];
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

        let expected = [
            current[COL_BAL],
            current[COL_IDX_A],
            current[COL_IDX_B],
            current[COL_IDX_B] - current[COL_IDX_A] - E::ONE,
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
        a.push(Assertion::single(COL_ACC_A, 0, zero));
        a.push(Assertion::single(COL_ACC_B, 0, zero));

        for i in 0..4 {
            a.push(Assertion::single(
                4 + i,
                ROW_ACCT_ROOT,
                self.pub_inputs.root_old[i],
            ));
            a.push(Assertion::single(
                LANE_B + 4 + i,
                ROW_ACCT_ROOT,
                self.pub_inputs.root_new[i],
            ));
        }

        a.push(Assertion::single(
            4,
            ROW_CUST_START,
            BaseElement::new(CUSTODIAN_DOMAIN),
        ));
        a.push(Assertion::single(
            LANE_B + 4,
            ROW_CUST_START,
            BaseElement::new(CUSTODIAN_DOMAIN),
        ));
        for i in 5..8 {
            a.push(Assertion::single(i, ROW_CUST_START, zero));
            a.push(Assertion::single(LANE_B + i, ROW_CUST_START, zero));
        }

        for i in 0..4 {
            a.push(Assertion::single(
                4 + i,
                ROW_CUST_ROOT,
                self.pub_inputs.custodian_set_root[i],
            ));
            a.push(Assertion::single(
                LANE_B + 4 + i,
                ROW_CUST_ROOT,
                self.pub_inputs.custodian_set_root[i],
            ));
        }

        a.push(Assertion::single(
            COL_COUNT_OLD,
            0,
            self.pub_inputs.recovery_count_old,
        ));
        a.push(Assertion::single(
            COL_COUNT_NEW,
            0,
            self.pub_inputs.recovery_count_new,
        ));

        a
    }
}

pub struct RecoveryProver {
    options: ProofOptions,
}

impl RecoveryProver {
    pub fn new(options: ProofOptions) -> Self {
        Self { options }
    }
}

impl Prover for RecoveryProver {
    type BaseField = BaseElement;
    type Air = RecoveryAir;
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

    fn get_pub_inputs(&self, trace: &Self::Trace) -> RecoveryPublicInputs {
        RecoveryPublicInputs {
            root_old: [
                trace.get(4, ROW_ACCT_ROOT),
                trace.get(5, ROW_ACCT_ROOT),
                trace.get(6, ROW_ACCT_ROOT),
                trace.get(7, ROW_ACCT_ROOT),
            ],
            root_new: [
                trace.get(LANE_B + 4, ROW_ACCT_ROOT),
                trace.get(LANE_B + 5, ROW_ACCT_ROOT),
                trace.get(LANE_B + 6, ROW_ACCT_ROOT),
                trace.get(LANE_B + 7, ROW_ACCT_ROOT),
            ],
            custodian_set_root: [
                trace.get(4, ROW_CUST_ROOT),
                trace.get(5, ROW_CUST_ROOT),
                trace.get(6, ROW_CUST_ROOT),
                trace.get(7, ROW_CUST_ROOT),
            ],
            recovery_count_old: trace.get(COL_COUNT_OLD, 0),
            recovery_count_new: trace.get(COL_COUNT_NEW, 0),
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
    use crate::circuit_threshold::build_custodian_set;
    use crate::merkle::native_merge;
    use winterfell::{verify, AcceptableOptions, BatchingMethod, FieldExtension};

    const BALANCE: u64 = 1_000_000;
    const COUNT_OLD: u64 = 7;

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

    fn custodian_keys() -> Vec<BaseElement> {
        vec![
            BaseElement::new(0xC0570D1A),
            BaseElement::new(0xC0570D1B),
            BaseElement::new(0xC0570D1C),
            BaseElement::new(0xC0570D1D),
            BaseElement::new(0xC0570D1E),
        ]
    }

    struct Scenario {
        auth: ThresholdAuth,
        id_old: Digest,
        id_new: Digest,
        nonce: BaseElement,
        path: MerklePath,
        public_inputs: RecoveryPublicInputs,
    }

    fn scenario() -> Scenario {
        let mut empty = vec![[BaseElement::ZERO; 4]];
        for k in 1..=TREE_DEPTH {
            let prev = empty[k - 1];
            empty.push(native_merge(prev, prev));
        }
        // La clave comprometida y la nueva del titular legitimo.
        let id_old = derive_public_id(BaseElement::new(0xA11CE));
        let id_new = derive_public_id(BaseElement::new(0xBEEF_CAFE));
        let nonce = BaseElement::new(3);

        let mut siblings = Vec::with_capacity(TREE_DEPTH);
        let mut is_right = Vec::with_capacity(TREE_DEPTH);
        for level in 0..TREE_DEPTH {
            siblings.push(empty[level]);
            is_right.push(level % 3 == 0);
        }
        let path = MerklePath { siblings, is_right };

        let keys = custodian_keys();
        let (set_root, cpaths) = build_custodian_set(&keys);

        let leaf_old = native_leaf(id_old, BaseElement::new(BALANCE), nonce);
        let leaf_new = native_leaf(
            id_new,
            BaseElement::new(BALANCE),
            nonce + BaseElement::ONE,
        );

        Scenario {
            public_inputs: RecoveryPublicInputs {
                root_old: native_climb(leaf_old, &path),
                root_new: native_climb(leaf_new, &path),
                custodian_set_root: set_root,
                recovery_count_old: BaseElement::new(COUNT_OLD),
                recovery_count_new: BaseElement::new(COUNT_OLD + 1),
            },
            auth: ThresholdAuth {
                key_a: keys[1],
                index_a: 1,
                path_a: cpaths[1].clone(),
                key_b: keys[3],
                index_b: 3,
                path_b: cpaths[3].clone(),
            },
            id_old,
            id_new,
            nonce,
            path,
        }
    }

    fn build(s: &Scenario, auth: &ThresholdAuth, bal_new: u64, count_delta: u64) -> TraceTable<BaseElement> {
        build_trace(
            auth,
            s.id_old,
            s.id_new,
            BALANCE,
            bal_new,
            s.nonce,
            &s.path,
            COUNT_OLD,
            count_delta,
        )
    }

    fn run(s: &Scenario, auth: &ThresholdAuth, bal_new: u64, count_delta: u64) -> Result<(), String> {
        let trace = build(s, auth, bal_new, count_delta);
        let prover = RecoveryProver::new(default_options());

        let hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| prover.prove(trace)));
        std::panic::set_hook(hook);

        let proof = match r {
            Err(_) => return Err("prove hizo panic".into()),
            Ok(Err(e)) => return Err(format!("prove Err: {e:?}")),
            Ok(Ok(p)) => p,
        };

        let min_opts = AcceptableOptions::OptionSet(vec![default_options()]);
        verify::<RecoveryAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof,
            s.public_inputs.clone(),
            &min_opts,
        )
        .map_err(|e| format!("verificacion fallo: {e:?}"))
    }

    /// Separa "la traza está mal construida" de "las restricciones están
    /// mal escritas".
    #[test]
    fn trace_landmarks_match_native() {
        let s = scenario();
        let trace = build(&s, &s.auth, BALANCE, 1);
        for i in 0..4 {
            assert_eq!(
                trace.get(4 + i, ROW_ACCT_ROOT),
                s.public_inputs.root_old[i]
            );
            assert_eq!(
                trace.get(LANE_B + 4 + i, ROW_ACCT_ROOT),
                s.public_inputs.root_new[i]
            );
            assert_eq!(
                trace.get(4 + i, ROW_CUST_ROOT),
                s.public_inputs.custodian_set_root[i]
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
        let derivadas = RecoveryProver::new(default_options()).get_pub_inputs(&trace);
        assert_eq!(
            derivadas.to_elements(),
            s.public_inputs.to_elements(),
            "las entradas DERIVADAS de la traza deben coincidir con las \
             DECLARADAS en todos sus campos"
        );

    }

    /// EL TEST CLAVE. No silencia el pánico.
    #[test]
    fn custodian_authorized_recovery_verifies() {
        let s = scenario();
        let trace = build(&s, &s.auth, BALANCE, 1);
        let prover = RecoveryProver::new(default_options());
        let proof = prover.prove(trace).expect("la recuperacion valida deberia probar");
        let min_opts = AcceptableOptions::OptionSet(vec![default_options()]);
        let v = verify::<RecoveryAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof,
            s.public_inputs.clone(),
            &min_opts,
        );
        assert!(v.is_ok(), "{v:?}");
    }

    /// **UNA RECUPERACIÓN NO PUEDE MOVER DINERO.**
    ///
    /// Reasigna el control de una cuenta, no su saldo. Sin esta
    /// restricción, dos custodios podrían vaciar cualquier cuenta bajo
    /// apariencia de recuperación.
    #[test]
    fn recovery_cannot_change_the_balance() {
        let s = scenario();
        assert!(
            run(&s, &s.auth, BALANCE - 1, 1).is_err(),
            "CRITICO: una recuperacion que altere el saldo permitiria a los \
             custodios vaciar cuentas bajo apariencia de recuperacion"
        );
        assert!(run(&s, &s.auth, BALANCE + 1, 1).is_err());
    }

    /// **EL CONTADOR HACE CONTABLES LAS INTERVENCIONES.**
    ///
    /// Sin incrementarlo, los custodios podrían reasignar cuentas en
    /// silencio: desde fuera, una recuperación sería indistinguible de
    /// cualquier otra transición de estado.
    #[test]
    fn a_silent_recovery_is_rejected() {
        let s = scenario();
        assert!(
            run(&s, &s.auth, BALANCE, 0).is_err(),
            "CRITICO: una recuperacion que no incremente el contador seria \
             invisible desde fuera"
        );
        // Tampoco vale saltar el contador.
        assert!(run(&s, &s.auth, BALANCE, 2).is_err());
    }

    /// **UN SOLO CUSTODIO NO PUEDE RECUPERAR.**
    #[test]
    fn the_same_custodian_cannot_count_twice() {
        let s = scenario();
        let keys = custodian_keys();
        let (_, cpaths) = build_custodian_set(&keys);
        let auth = ThresholdAuth {
            key_a: keys[2],
            index_a: 2,
            path_a: cpaths[2].clone(),
            key_b: keys[2],
            index_b: 2,
            path_b: cpaths[2].clone(),
        };
        assert!(run(&s, &auth, BALANCE, 1).is_err());
    }

    /// Quien no es custodio no puede recuperar.
    #[test]
    fn a_non_custodian_cannot_recover() {
        let s = scenario();
        let keys = custodian_keys();
        let (_, cpaths) = build_custodian_set(&keys);
        let auth = ThresholdAuth {
            key_a: BaseElement::new(0x1337),
            index_a: 1,
            path_a: cpaths[1].clone(),
            key_b: keys[3],
            index_b: 3,
            path_b: cpaths[3].clone(),
        };
        assert!(run(&s, &auth, BALANCE, 1).is_err());
    }

    /// Declarar una raíz nueva que no corresponde.
    #[test]
    fn wrong_new_root_is_rejected() {
        let s = scenario();
        let trace = build(&s, &s.auth, BALANCE, 1);
        let prover = RecoveryProver::new(default_options());
        let proof = prover.prove(trace).expect("prove");
        let mut declared = s.public_inputs.clone();
        declared.root_new = [BaseElement::new(999_999); 4];
        let min_opts = AcceptableOptions::OptionSet(vec![default_options()]);
        let v = verify::<RecoveryAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof, declared, &min_opts,
        );
        assert!(v.is_err());
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

        let s = scenario();
        let trace = build(&s, &s.auth, BALANCE, 1);
        let rows = rows_of(&trace, TRACE_WIDTH, TRACE_LENGTH);

        let air = RecoveryAir::new(
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
