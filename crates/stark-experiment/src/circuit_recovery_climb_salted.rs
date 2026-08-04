//! ⚠️ **ANDAMIO B13/B14 — GEMELO del mundo nuevo (paso 3, §150).**
//!
//! Copia declarada de `circuit_recovery_climb` — la fase de cuentas de
//! la recuperación, aislada: hoja + subida, dos carriles con el SALTO
//! de nonce (`+1`) y LA COPIA del salt (§93.4) heredados de recovery.
//! Sin custodios ni frozen (R6 no aplica). Por el playbook.
//!
//! **Cláusula de retirada**: en el flip (release única, D4) este módulo
//! SUSTITUYE a `circuit_recovery_climb` y el legacy se borra. Hasta
//! entonces, nadie fuera de los tests de este crate lo importa.
//!
//! ---
//!
//! # Recuperacion de una cuenta, SIN autorizacion (§64)
//!
//! La parte propia de `circuit_recovery`: construir la hoja antigua y la
//! nueva, y subir las dos al arbol de cuentas con los mismos hermanos.
//! **La autorizacion de custodios se ha amputado** (entrada 33): llega en dos
//! pruebas aparte y la capa las exige.
//!
//! ## Lo que SI sigue probando, y es lo que importa
//!
//! **El saldo no cambia.** Los dos carriles construyen su hoja con la MISMA
//! columna `COL_BAL`, y el segmento de rango la descompone en 64 bits. Una
//! recuperacion reasigna el control, no mueve dinero — y eso lo prueba el
//! circuito, no la confianza en los custodios.
//!
//! A diferencia de `circuit_frozen_climb`, aqui las hojas **no** son libres
//! (§58.3): si lo fueran, dos custodios podrian vaciar una cuenta bajo
//! apariencia de recuperacion.
//!
//! Tambien prueba que el nonce sube en uno y que el contador de
//! recuperaciones sube en uno.
//!
//! ## Lo que NO prueba
//!
//! Quien autorizo. Eso lo comprueba la capa con dos pruebas de
//! `circuit_threshold_single_nullifier` atadas a esta transicion de raices.

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

use crate::merkle::{Digest, MerklePath, TREE_DEPTH};
use crate::rescue_hash::{apply_sbox, NUM_ROUNDS, STATE_WIDTH};

pub const CYCLE_LENGTH: usize = 8;
pub const TRACE_LENGTH: usize = 512;
pub const SEGMENT_LENGTH: usize = 64;
/// Segmentos: saldo, índice A, índice B, y `B − A − 1`.
/// Un solo segmento: el rango del saldo. Los otros tres eran los indices de
/// custodio y el orden estricto, que se van con la autorizacion.
pub const NUM_SEGMENTS: usize = 1;

// ===== Columnas =====
const LANE_B: usize = STATE_WIDTH; // 12
const COL_BIT_A: usize = 2 * STATE_WIDTH; // 24
// No hay segundo bit de direccion. En la subida al arbol de cuentas los DOS
// carriles comparten camino -es la misma posicion, distinta identidad- asi
// que uno basta. El otro existia solo para la subida de custodios; al
// amputarla quedaba una columna siempre a cero y una restriccion booleana
// sobre ella que no restringia nada (64.2).
/// Identidad de la cuenta ANTES de la recuperación.
const COL_ID_OLD: usize = COL_BIT_A + 1; // 25..29
/// Identidad DESPUÉS. La posición en el árbol no cambia.
const COL_ID_NEW: usize = COL_ID_OLD + 4; // 30..34
const COL_BAL: usize = COL_ID_NEW + 4; // 34
const COL_NONCE: usize = COL_BAL + 1; // 35
/// Contador público de recuperaciones. Lo que hace **contables** las
/// intervenciones de los custodios.
const COL_COUNT_OLD: usize = COL_NONCE + 1; // 36
const COL_COUNT_NEW: usize = COL_COUNT_OLD + 1; // 37
const COL_SBIT: usize = COL_COUNT_NEW + 1; // 38
const COL_SACC: usize = COL_SBIT + 1; // 39
pub const TRACE_WIDTH: usize = COL_SACC + 1; // 40

// ===== Filas =====
const ROW_LEAF_LINK: usize = 7;
const ROW_LEAF_DONE: usize = 15;
/// Fila donde el estado contiene las dos raices. Lo que hay despues es
/// relleno hasta la potencia de dos: la 271 **no es fila de enlace**, asi que
/// la transicion 271->272 no activa ninguna restriccion y el relleno sale
/// gratis (a diferencia de `circuit_frozen_climb`, §60.2).
pub const ROW_ACCT_ROOT: usize = 271;

// ===== Restricciones =====
const C_HASH_A: usize = 0;
const C_HASH_B: usize = C_HASH_A + STATE_WIDTH;
const C_CAP_A: usize = C_HASH_B + STATE_WIDTH; // 4
const C_CAP_B: usize = C_CAP_A + 4;
const C_PLACE_A: usize = C_CAP_B + 4; // 4
const C_PLACE_B: usize = C_PLACE_A + 4;
const C_SIBLING: usize = C_PLACE_B + 4; // 4
const C_BIT_BOOL: usize = C_SIBLING + 4; // 1
const C_LEAF_CAP_A: usize = C_BIT_BOOL + 1; // 4
const C_LEAF_CAP_B: usize = C_LEAF_CAP_A + 4;
const C_LEAF_DIG_A: usize = C_LEAF_CAP_B + 4; // 4
const C_LEAF_DIG_B: usize = C_LEAF_DIG_A + 4;
/// El nonce incrementa: marca que la cuenta cambió de control.
const C_NONCE: usize = C_LEAF_DIG_B + 4; // 2
/// Entradas: identidad antigua + saldo (A), identidad nueva + **el mismo
/// saldo** (B).
const C_INPUT: usize = C_NONCE + 2; // 10
/// **EL CONTADOR INCREMENTA EXACTAMENTE EN UNO.**
const C_COUNT: usize = C_INPUT + 10; // 1
const C_TRANSPORT: usize = C_COUNT + 1; // 4
const C_ID_CONST: usize = C_TRANSPORT + 4; // 8
const C_SBIT_BOOL: usize = C_ID_CONST + 8; // 2
const C_FIRST_S: usize = C_SBIT_BOOL + 2; // 2
const C_HORNER: usize = C_FIRST_S + 2; // 1
const C_SEG_LINK: usize = C_HORNER + 1; // NUM_SEGMENTS
pub const NUM_CONSTRAINTS: usize = C_SEG_LINK + NUM_SEGMENTS;

// ===== Periódicas =====
const P_HASH_FLAG: usize = 0;
const P_ARK1: usize = 1;
const P_ARK2: usize = P_ARK1 + STATE_WIDTH;
const P_ACCT_LINK: usize = P_ARK2 + STATE_WIDTH;
const P_LINK_LEAF: usize = P_ACCT_LINK + 1;
const P_FIRST_ROW: usize = P_LINK_LEAF + 1;
const P_FIRST_S: usize = P_FIRST_ROW + 1;
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
        for i in 0..4 {
            row[COL_ID_OLD + i] = id_old[i];
            row[COL_ID_NEW + i] = id_new[i];
        }
        row[COL_BAL] = c_bal;
        row[COL_NONCE] = nonce;
        row[COL_COUNT_OLD] = c_count_old;
        row[COL_COUNT_NEW] = c_count_new;
    }

    // Un solo segmento: el rango del saldo.
    let segment_values = [c_bal.as_int()];
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
    let mut state_a = [zero; STATE_WIDTH];
    let mut state_b = [zero; STATE_WIDTH];
    // Carril A: identidad antigua. Carril B: identidad nueva, MISMO saldo.
    state_a[4..8].copy_from_slice(&id_old);
    state_a[8] = c_bal;
    state_b[4..8].copy_from_slice(&id_new);
    state_b[8] = c_bal_new;

    rows[0][..STATE_WIDTH].copy_from_slice(&state_a);
    rows[0][LANE_B..LANE_B + STATE_WIDTH].copy_from_slice(&state_b);

    for r in 0..ROW_ACCT_ROOT {
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
                _ => {
                    let next_cycle = (r + 1) / CYCLE_LENGTH;
                    if (2..34).contains(&next_cycle) {
                        let level = next_cycle - 2;
                        place_acct(&mut state_a, &digest_a, level);
                        place_acct(&mut state_b, &digest_b, level);
                    }
                }
            }
        }
        rows[r + 1][..STATE_WIDTH].copy_from_slice(&state_a);
        rows[r + 1][LANE_B..LANE_B + STATE_WIDTH].copy_from_slice(&state_b);
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
    let mut trace = TraceTable::new(TRACE_WIDTH, TRACE_LENGTH);
    trace.fill(
        |s| s.copy_from_slice(&rows[0]),
        |step, s| s.copy_from_slice(&rows[step + 1]),
    );
    trace
}

#[derive(Clone, Debug)]
pub struct RecoveryClimbPublicInputs {
    pub root_old: Digest,
    pub root_new: Digest,
    /// Contador de recuperaciones ANTES.
    pub recovery_count_old: BaseElement,
    /// Y DESPUÉS: siempre `old + 1`. Lo que hace contables las
    /// intervenciones de los custodios.
    pub recovery_count_new: BaseElement,
}

impl ToElements<BaseElement> for RecoveryClimbPublicInputs {
    fn to_elements(&self) -> Vec<BaseElement> {
        let mut out = self.root_old.to_vec();
        out.extend_from_slice(&self.root_new);
        out.push(self.recovery_count_old);
        out.push(self.recovery_count_new);
        out
    }
}

pub struct RecoveryClimbAir {
    context: AirContext<BaseElement>,
    pub_inputs: RecoveryClimbPublicInputs,
}

impl Air for RecoveryClimbAir {
    type BaseField = BaseElement;
    type PublicInputs = RecoveryClimbPublicInputs;

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
        // Colocacion de cuentas (8) + hermano (4).
        for _ in 0..12 {
            degrees.push(TransitionConstraintDegree::with_cycles(2, full.clone()));
        }
        degrees.push(TransitionConstraintDegree::new(2));
        // Hojas (16), nonce (2), entradas (10).
        for _ in 0..28 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }
        // Contador (1), transporte (4), identidades (8): sin ciclo.
        for _ in 0..13 {
            degrees.push(TransitionConstraintDegree::new(1));
        }
        for _ in 0..2 {
            degrees.push(TransitionConstraintDegree::new(2));
        }
        for _ in 0..(3 + NUM_SEGMENTS) {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }

        assert_eq!(degrees.len(), NUM_CONSTRAINTS, "cuenta de grados");

        RecoveryClimbAir {
            context: AirContext::new(trace_info, degrees, 24, options),
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
        for r in 0..=ROW_ACCT_ROOT {
            if r % CYCLE_LENGTH < NUM_ROUNDS {
                hash_flag[r] = one;
            }
        }
        columns.push(hash_flag);

        for ark in [true, false] {
            for i in 0..STATE_WIDTH {
                let mut col = vec![zero; TRACE_LENGTH];
                for r in 0..=ROW_ACCT_ROOT {
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

        let mut first_row = vec![zero; TRACE_LENGTH];
        first_row[0] = one;
        columns.push(first_row);

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
        let first_row = periodic[P_FIRST_ROW];
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
        let any_link = acct_link;

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
        }

        result[C_BIT_BOOL] = current[COL_BIT_A] * (current[COL_BIT_A] - E::ONE);

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

        // ===== EL CONTADOR INCREMENTA EXACTAMENTE EN UNO =====
        // Sin esto, los custodios podrían reasignar cuentas en silencio:
        // desde fuera, una recuperación sería indistinguible de cualquier
        // otra transición de estado.
        result[C_COUNT] =
            current[COL_COUNT_NEW] - (current[COL_COUNT_OLD] + E::ONE);

        let transport = [
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

        let expected = [current[COL_BAL]];
        for seg in 0..NUM_SEGMENTS {
            result[C_SEG_LINK + seg] = periodic[P_SEG_LINK + seg] * (sacc_next - expected[seg]);
        }
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        let zero = BaseElement::ZERO;
        let mut a = Vec::with_capacity(24);

        for i in 0..4 {
            a.push(Assertion::single(i, 0, zero));
            a.push(Assertion::single(LANE_B + i, 0, zero));
        }
        for i in 9..12 {
            a.push(Assertion::single(i, 0, zero));
            a.push(Assertion::single(LANE_B + i, 0, zero));
        }
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

pub struct RecoveryClimbProver {
    options: ProofOptions,
}

impl RecoveryClimbProver {
    pub fn new(options: ProofOptions) -> Self {
        Self { options }
    }
}

impl Prover for RecoveryClimbProver {
    type BaseField = BaseElement;
    type Air = RecoveryClimbAir;
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

    fn get_pub_inputs(&self, trace: &Self::Trace) -> RecoveryClimbPublicInputs {
        RecoveryClimbPublicInputs {
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
    use crate::merkle::native_merge;
    use winterfell::{verify, AcceptableOptions, BatchingMethod, FieldExtension, Prover};

    const BALANCE: u64 = 1_000_000;
    const COUNT_OLD: u64 = 7;

    fn default_options() -> ProofOptions {
        ProofOptions::new(
            32, 8, 0, FieldExtension::None, 8, 31,
            BatchingMethod::Linear, BatchingMethod::Linear,
        )
    }

    struct Scenario {
        id_old: Digest,
        id_new: Digest,
        nonce: BaseElement,
        path: MerklePath,
        public_inputs: RecoveryClimbPublicInputs,
    }

    fn scenario() -> Scenario {
        let mut empty = vec![[BaseElement::ZERO; 4]];
        for k in 1..=TREE_DEPTH {
            let prev = empty[k - 1];
            empty.push(native_merge(prev, prev));
        }
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

        let leaf_old = native_leaf(id_old, BaseElement::new(BALANCE), nonce);
        let leaf_new = native_leaf(id_new, BaseElement::new(BALANCE), nonce + BaseElement::ONE);

        Scenario {
            public_inputs: RecoveryClimbPublicInputs {
                root_old: native_climb(leaf_old, &path),
                root_new: native_climb(leaf_new, &path),
                recovery_count_old: BaseElement::new(COUNT_OLD),
                recovery_count_new: BaseElement::new(COUNT_OLD + 1),
            },
            id_old, id_new, nonce, path,
        }
    }

    fn build(s: &Scenario, bal_new: u64, count_delta: u64) -> TraceTable<BaseElement> {
        build_trace(s.id_old, s.id_new, BALANCE, bal_new, s.nonce, &s.path,
                    COUNT_OLD, count_delta)
    }

    fn run(s: &Scenario, bal_new: u64, count_delta: u64) -> Result<(), String> {
        let trace = build(s, bal_new, count_delta);
        let prover = RecoveryClimbProver::new(default_options());
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| prover.prove(trace)));
        let proof = match r {
            // El mensaje del panico se conserva: winterfell da el detalle en
            // depuracion y descartarlo costo tres rondas en su dia (25).
            Err(e) => {
                let msg = e.downcast_ref::<String>().cloned()
                    .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
                    .unwrap_or_else(|| "panico sin mensaje".into());
                return Err(format!("prove hizo panic: {msg}"));
            }
            Ok(Err(e)) => return Err(format!("prove Err: {e:?}")),
            Ok(Ok(pr)) => pr,
        };
        let min_opts = AcceptableOptions::OptionSet(vec![default_options()]);
        verify::<RecoveryClimbAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof, s.public_inputs.clone(), &min_opts,
        ).map_err(|e| format!("verificacion fallo: {e:?}"))
    }

    /// El mas informativo si algo falla: las raices de la traza deben ser las
    /// que calcula la version nativa.
    #[test]
    fn trace_roots_match_native() {
        let s = scenario();
        let trace = build(&s, BALANCE, 1);
        for i in 0..4 {
            assert_eq!(trace.get(4 + i, ROW_ACCT_ROOT), s.public_inputs.root_old[i],
                       "carril A (antigua), elem {i}");
            assert_eq!(trace.get(LANE_B + 4 + i, ROW_ACCT_ROOT), s.public_inputs.root_new[i],
                       "carril B (nueva), elem {i}");
        }
    }

    #[test]
    fn a_valid_recovery_climb_verifies() {
        let s = scenario();
        assert_eq!(run(&s, BALANCE, 1), Ok(()));
    }

    /// LA RAZON DE SER DE ESTE CIRCUITO. Si las hojas fueran libres -como en
    /// `circuit_frozen_climb`, donde es correcto (58.3)- dos custodios
    /// podrian vaciar una cuenta bajo apariencia de recuperacion, y un
    /// auditor externo solo veria dos raices cambiando.
    #[test]
    fn recovery_cannot_change_the_balance() {
        let s = scenario();
        assert!(run(&s, BALANCE - 1, 1).is_err(),
            "SOLIDEZ: una recuperacion que altere el saldo permitiria a los \
             custodios vaciar cuentas bajo apariencia de recuperacion");
        assert!(run(&s, BALANCE + 1, 1).is_err(),
            "SOLIDEZ: tampoco puede CREAR dinero");
    }

    /// El contador hace contables las intervenciones: sin incrementarlo, una
    /// recuperacion seria indistinguible de cualquier otra transicion.
    #[test]
    fn a_silent_recovery_is_rejected() {
        let s = scenario();
        assert!(run(&s, BALANCE, 0).is_err(), "SOLIDEZ: el contador debe subir");
    }

    #[test]
    fn wrong_declared_root_is_rejected() {
        let mut s = scenario();
        s.public_inputs.root_new = s.public_inputs.root_old;
        assert!(run(&s, BALANCE, 1).is_err());
    }

    /// Ninguna restriccion es vacua (entrada 38, 62).
    #[test]
    fn no_constraint_is_vacuous() {
        use crate::mutation::{buscar_vacias, rows_of};
        let s = scenario();
        let trace = build(&s, BALANCE, 1);
        let rows = rows_of(&trace, TRACE_WIDTH, TRACE_LENGTH);
        let air = RecoveryClimbAir::new(
            TraceInfo::new(TRACE_WIDTH, TRACE_LENGTH),
            s.public_inputs.clone(),
            default_options(),
        );
        let informe = buscar_vacias(&air, &rows, 1);
        assert!(informe.nunca_disparadas.is_empty(),
            "restricciones que NINGUNA perturbacion activa (de {} totales, \
             {} celdas probadas): {:?}",
            informe.total, informe.celdas, informe.nunca_disparadas);
    }
}
