//! **Crédito de saldo con ascenso de Merkle** — el clímber del reembolso
//! (`doc/CADUCIDAD_PENDIENTE.md`, AUDITORIA §178, operación R-2b).
//!
//! Demuestra que el saldo de una cuenta sube EXACTAMENTE en el importe y
//! que la raíz de cuentas transita de `root_old` a `root_new` en
//! consecuencia, **sin tocar el suministro global** — devolver un pendiente
//! caducado no crea dinero: lo mueve del árbol de pendientes de vuelta a su
//! dueño.
//!
//! ## Origen y diferencias — LEER antes de editar
//!
//! Este circuito es un **clon de `circuit_mint_climb`** con TRES cortes, y
//! solo tres. Si tocas uno, cruza con el otro: comparten el 95% de su
//! maquinaria (doble carril Rescue, ascenso de Merkle, descomposición de
//! rango, el cerrojo del salt del titular §117) y un bug en la parte común
//! vive en ambos.
//!
//! 1. **Sin `C_SUPPLY`**: mint_climb ata `supply_new == supply_old + amount`
//!    en circuito (su rótulo «EL SUMINISTRO SUBE EXACTAMENTE EN EL IMPORTE»).
//!    El crédito NO menciona el suministro; esa restricción no existe aquí.
//! 2. **Rango de 3 segmentos, no 5**: mint_climb descompone en bits
//!    `[balance, amount, balance_new, supply_new, max−supply_new]`. Los dos
//!    últimos son del suministro y se van; quedan los tres del saldo, que
//!    siguen acotando que las sumas no desbordan el cuerpo.
//! 3. **`PublicInputs` sin suministro**: fuera `supply_old`, `supply_new`,
//!    `max_supply`. La capa NO declara ni verifica nada de suministro al
//!    aplicar un reembolso.
//!
//! Lo que se CONSERVA idéntico y es esencial: el ascenso de la cuenta
//! (`C_INPUT`/`C_BALANCE`/el árbol), y el salt del titular derivado de su
//! clave (§117) — que en el reembolso ES el cerrojo #2 de §178: solo el
//! emisor puede fabricar la subida de SU hoja.
//!
//! ## Grado en depuración (familia de la entrada 24, §46/§34)
//!
//! Como su origen, en release genera y verifica; la comprobación de grados
//! en depuración puede degenerar sobre valores de dominio (márgenes a cero).

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
/// CORTE 2: tres segmentos de rango (balance, amount, balance_new); los
/// dos de suministro de mint_climb no existen aquí.
pub const NUM_SEGMENTS: usize = 3;

const LANE_B: usize = STATE_WIDTH; // 12
const COL_BIT_A: usize = 24;
const COL_ACC_ID: usize = COL_BIT_A + 1; // 25..29
const COL_BAL: usize = COL_ACC_ID + 4; // 29
const COL_BAL_NEW: usize = COL_BAL + 1; // 30
const COL_NONCE: usize = COL_BAL_NEW + 1; // 31
const COL_AMT: usize = COL_NONCE + 1; // 32
const COL_SBIT: usize = COL_AMT + 1; // 33
const COL_SACC: usize = COL_SBIT + 1; // 34
const COL_LEAF_SALT: usize = COL_SACC + 1; // 35..39
pub const TRACE_WIDTH: usize = COL_LEAF_SALT + 4; // 39

const CYC_NONCE: usize = 1;
const CYC_SALT: usize = CYC_NONCE + 1;
const CYC_ACC: usize = CYC_SALT + 1;
const CYC_FIN: usize = CYC_ACC + TREE_DEPTH;
const ROW_LEAF_LINK: usize = CYC_NONCE * CYCLE_LENGTH - 1;
const ROW_SALT_LINK: usize = CYC_SALT * CYCLE_LENGTH - 1;
const ROW_LEAF_DONE: usize = CYC_ACC * CYCLE_LENGTH - 1;
const ROW_ACCT_ROOT: usize = CYC_FIN * CYCLE_LENGTH - 1;
const _: () = assert!(ROW_ACCT_ROOT < TRACE_LENGTH);

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
const C_NONCE: usize = C_LEAF_DIG_B + 4; // 2
const C_INPUT: usize = C_NONCE + 2; // 10
const C_BALANCE: usize = C_INPUT + 10; // 1
// CORTE 1: aquí iba C_SUPPLY (1). No existe. `C_TRANSPORT` la sucede.
const C_TRANSPORT: usize = C_BALANCE + 1; // 4  (transporte solo del saldo)
const C_ID_CONST: usize = C_TRANSPORT + 4; // 4
const C_SBIT_BOOL: usize = C_ID_CONST + 4; // 2
const C_FIRST_S: usize = C_SBIT_BOOL + 2; // 2
const C_HORNER: usize = C_FIRST_S + 2; // 1
const C_SEG_LINK: usize = C_HORNER + 1; // NUM_SEGMENTS (3)
const C_SALT_CAP_A: usize = C_SEG_LINK + NUM_SEGMENTS; // 4
const C_SALT_CAP_B: usize = C_SALT_CAP_A + 4; // 4
const C_SALT_DIG_A: usize = C_SALT_CAP_B + 4; // 4
const C_SALT_DIG_B: usize = C_SALT_DIG_A + 4; // 4
const C_SALT_IN_A: usize = C_SALT_DIG_B + 4; // 4
const C_SALT_IN_B: usize = C_SALT_IN_A + 4; // 4
pub const NUM_CONSTRAINTS: usize = C_SALT_IN_B + 4;

const P_HASH_FLAG: usize = 0;
const P_ARK1: usize = 1;
const P_ARK2: usize = P_ARK1 + STATE_WIDTH;
const P_ACCT_LINK: usize = P_ARK2 + STATE_WIDTH;
const P_LINK_LEAF: usize = P_ACCT_LINK + 1;
const P_LINK_SALT: usize = P_LINK_LEAF + 1;
const P_FIRST_ROW: usize = P_LINK_SALT + 1;
const P_FIRST_S: usize = P_FIRST_ROW + 1;
const P_CONT_S: usize = P_FIRST_S + 1;
const P_SEG_LINK: usize = P_CONT_S + 1;

// CELDAS_LIBRES: salt de hoja testigo, presente en todas las filas (clase *, cols 35..39) — §117
// CELDAS_LIBRES: bit de camino: solo los enlaces de cuenta lo miran (clase sin acct_link, col 24) — §191
// CELDAS_LIBRES: descansos del acumulador de saldo entre segmentos (clase sin cont_s, col 34) — §191
// CELDAS_LIBRES: limbos altos del primer merge, carril A: solo el limbo 8 lleva nonce (clase cont_s+link_leaf, cols 9..12) — §92.2
// CELDAS_LIBRES: limbos altos del primer merge, carril B (clase cont_s+link_leaf, cols 21..24) — §92.2
// CELDAS_LIBRES: carriles hash muertos tras la raíz, capacidad A (clase plana, cols 0..4) — §191
// CELDAS_LIBRES: carriles muertos tras la raíz, salvo digest asertados: rate A alto + capacidad B (clase plana, cols 8..16) — §191
// CELDAS_LIBRES: carriles muertos tras la raíz, rate B alto (clase plana, cols 20..24) — §191
type Blake3 = Blake3_256<BaseElement>;

fn value_to_bits_be(value: u64) -> Vec<bool> {
    (0..SEGMENT_LENGTH)
        .map(|p| (value >> (SEGMENT_LENGTH - 1 - p)) & 1 == 1)
        .collect()
}

/// Construye la traza del crédito: dos carriles (saldo viejo, saldo nuevo)
/// que ascienden a la misma posición del árbol. Sin `supply_delta` ni
/// `max_supply` — el suministro no participa (CORTE 1/2).
pub fn build_trace(
    account_id: Digest,
    balance: u64,
    nonce: BaseElement,
    leaf_salt: Digest,
    path: &MerklePath,
    amount: u64,
) -> TraceTable<BaseElement> {
    let zero = BaseElement::ZERO;
    let c_bal = BaseElement::new(balance);
    let c_amt = BaseElement::new(amount);
    let c_bal_new = c_bal + c_amt;
    let mut rows: Vec<Vec<BaseElement>> = vec![vec![zero; TRACE_WIDTH]; TRACE_LENGTH];
    for row in rows.iter_mut() {
        for i in 0..4 {
            row[COL_ACC_ID + i] = account_id[i];
        }
        row[COL_BAL] = c_bal;
        row[COL_BAL_NEW] = c_bal_new;
        row[COL_NONCE] = nonce;
        row[COL_AMT] = c_amt;
        row[COL_LEAF_SALT..COL_LEAF_SALT + 4].copy_from_slice(&leaf_salt);
    }
    // CORTE 2: tres segmentos de saldo; sin supply_new ni margen.
    let segment_values = [c_bal.as_int(), c_amt.as_int(), c_bal_new.as_int()];
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
        debug_assert!(level < TREE_DEPTH);
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
                    state_b[8] = nonce;
                }
                ROW_SALT_LINK => {
                    state_a[4..8].copy_from_slice(&digest_a);
                    state_a[8..12].copy_from_slice(&leaf_salt);
                    state_b[4..8].copy_from_slice(&digest_b);
                    state_b[8..12].copy_from_slice(&leaf_salt);
                }
                ROW_LEAF_DONE => {
                    place_acct(&mut state_a, &digest_a, 0);
                    place_acct(&mut state_b, &digest_b, 0);
                }
                _ => {
                    let next_cycle = (r + 1) / CYCLE_LENGTH;
                    if (CYC_ACC..CYC_FIN).contains(&next_cycle) {
                        let level = next_cycle - CYC_ACC;
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
            rows[(CYC_ACC + level) * CYCLE_LENGTH + p][COL_BIT_A] = bit;
        }
    }
    let mut trace = TraceTable::new(TRACE_WIDTH, TRACE_LENGTH);
    trace.fill(
        |s| s.copy_from_slice(&rows[0]),
        |step, s| s.copy_from_slice(&rows[step + 1]),
    );
    trace
}

/// Entradas públicas del crédito: las dos raíces y el importe. CORTE 3:
/// sin `supply_old`/`supply_new`/`max_supply`.
#[derive(Clone, Debug)]
pub struct CreditClimbPublicInputs {
    pub root_old: Digest,
    pub root_new: Digest,
    pub amount: BaseElement,
}

impl ToElements<BaseElement> for CreditClimbPublicInputs {
    fn to_elements(&self) -> Vec<BaseElement> {
        let mut out = self.root_old.to_vec();
        out.extend_from_slice(&self.root_new);
        out.push(self.amount);
        out
    }
}

pub struct CreditClimbAir {
    context: AirContext<BaseElement>,
    pub_inputs: CreditClimbPublicInputs,
}

impl Air for CreditClimbAir {
    type BaseField = BaseElement;
    type PublicInputs = CreditClimbPublicInputs;

    fn new(trace_info: TraceInfo, pub_inputs: Self::PublicInputs, options: ProofOptions) -> Self {
        assert_eq!(TRACE_WIDTH, trace_info.width());
        let full = vec![TRACE_LENGTH];
        let mut degrees = Vec::with_capacity(NUM_CONSTRAINTS);
        // Rescue de ambos carriles (24), capacidades de enlace (8), colocación
        // y hermano y bit (12), bit-bool (1) — idénticos a mint_climb.
        for _ in 0..2 * STATE_WIDTH {
            degrees.push(TransitionConstraintDegree::with_cycles(7, full.clone()));
        }
        for _ in 0..8 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }
        for _ in 0..12 {
            degrees.push(TransitionConstraintDegree::with_cycles(2, full.clone()));
        }
        degrees.push(TransitionConstraintDegree::new(2)); // C_BIT_BOOL
        // C_LEAF_* + C_NONCE + C_INPUT (28), idénticos.
        for _ in 0..28 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }
        // CORTE 1: mint_climb tenía 13 grados-1 aquí (C_BALANCE + C_SUPPLY +
        // C_TRANSPORT[7] + C_ID_CONST[4]). Sin supply: 12
        // (C_BALANCE[1] + C_TRANSPORT[4] + C_ID_CONST[4] = 9 grado-1
        //  y C_SBIT_BOOL[2] grado-2 aparte). Se cuentan explícitos abajo.
        degrees.push(TransitionConstraintDegree::new(1)); // C_BALANCE
        for _ in 0..4 {
            degrees.push(TransitionConstraintDegree::new(1)); // C_TRANSPORT (saldo)
        }
        for _ in 0..4 {
            degrees.push(TransitionConstraintDegree::new(1)); // C_ID_CONST
        }
        for _ in 0..2 {
            degrees.push(TransitionConstraintDegree::new(2)); // C_SBIT_BOOL
        }
        for _ in 0..3 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone())); // C_FIRST_S[2]+C_HORNER[1]
        }
        for _ in 0..NUM_SEGMENTS {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone())); // C_SEG_LINK (3)
        }
        for _ in 0..24 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone())); // C_SALT_* (6×4)
        }
        assert_eq!(degrees.len(), NUM_CONSTRAINTS, "cuenta de grados");
        CreditClimbAir {
            context: AirContext::new(trace_info, degrees, 23, options),
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
            acct_link[(CYC_ACC + level) * CYCLE_LENGTH + 7] = one;
        }
        columns.push(acct_link);
        let mut link_leaf = vec![zero; TRACE_LENGTH];
        link_leaf[ROW_LEAF_LINK] = one;
        columns.push(link_leaf);
        let mut link_salt = vec![zero; TRACE_LENGTH];
        link_salt[ROW_SALT_LINK] = one;
        columns.push(link_salt);
        for row in [0] {
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
        let link_salt = periodic[P_LINK_SALT];
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
            let sib_b = (E::ONE - bit_a) * next[LANE_B + 8 + i] + bit_a * next[LANE_B + 4 + i];
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
        result[C_NONCE] = link_leaf * (next[8] - current[COL_NONCE]);
        result[C_NONCE + 1] = link_leaf * (next[LANE_B + 8] - current[COL_NONCE]);
        for i in 0..4 {
            result[C_SALT_CAP_A + i] = link_salt * next[i];
            result[C_SALT_CAP_B + i] = link_salt * next[LANE_B + i];
            result[C_SALT_DIG_A + i] = link_salt * (next[4 + i] - current[4 + i]);
            result[C_SALT_DIG_B + i] =
                link_salt * (next[LANE_B + 4 + i] - current[LANE_B + 4 + i]);
            result[C_SALT_IN_A + i] = link_salt * (next[8 + i] - current[COL_LEAF_SALT + i]);
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
        result[C_BALANCE] = current[COL_BAL_NEW] - (current[COL_BAL] + current[COL_AMT]);
        // CORTE 1: sin C_SUPPLY. El transporte solo cubre el saldo.
        let transport = [COL_BAL, COL_BAL_NEW, COL_NONCE, COL_AMT];
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
        // CORTE 2: tres valores esperados (saldo), sin los dos de suministro.
        let expected = [
            current[COL_BAL],
            current[COL_AMT],
            current[COL_BAL_NEW],
        ];
        for seg in 0..NUM_SEGMENTS {
            result[C_SEG_LINK + seg] = periodic[P_SEG_LINK + seg] * (sacc_next - expected[seg]);
        }
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        let zero = BaseElement::ZERO;
        let mut a = Vec::with_capacity(28);
        for i in 0..4 {
            a.push(Assertion::single(i, 0, zero));
            a.push(Assertion::single(LANE_B + i, 0, zero));
        }
        for i in 9..12 {
            a.push(Assertion::single(i, 0, zero));
            a.push(Assertion::single(LANE_B + i, 0, zero));
        }
        for i in 0..4 {
            a.push(Assertion::single(4 + i, ROW_ACCT_ROOT, self.pub_inputs.root_old[i]));
            a.push(Assertion::single(
                LANE_B + 4 + i,
                ROW_ACCT_ROOT,
                self.pub_inputs.root_new[i],
            ));
        }
        a.push(Assertion::single(COL_AMT, 0, self.pub_inputs.amount));
        // CORTE 3: sin las tres aserciones de suministro.
        a
    }
}

pub struct CreditClimbProver {
    options: ProofOptions,
}

impl CreditClimbProver {
    pub fn new(options: ProofOptions) -> Self {
        Self { options }
    }
}

impl Prover for CreditClimbProver {
    type BaseField = BaseElement;
    type Air = CreditClimbAir;
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

    fn get_pub_inputs(&self, trace: &Self::Trace) -> CreditClimbPublicInputs {
        CreditClimbPublicInputs {
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
            amount: trace.get(COL_AMT, 0),
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
    use crate::native::{
        derive_leaf_salt_wide, derive_public_id_wide, native_climb, native_leaf_salted,
    };
    use crate::merkle::native_merge;
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

    struct Scenario {
        account_id: Digest,
        balance: u64,
        nonce: BaseElement,
        leaf_salt: Digest,
        path: MerklePath,
        amount: u64,
        public_inputs: CreditClimbPublicInputs,
    }

    /// El salt del titular es ANCHO y derivado de su clave (§117): en el
    /// reembolso, es el cerrojo #2 — solo el emisor lo reproduce.
    fn scenario_con_clave(balance: u64, amount: u64, key: [BaseElement; 4]) -> Scenario {
        let mut empty = vec![[BaseElement::ZERO; 4]];
        for k in 1..=TREE_DEPTH {
            let prev = empty[k - 1];
            empty.push(native_merge(prev, prev));
        }
        let account_id = derive_public_id_wide(key);
        let nonce = BaseElement::ZERO;
        let mut siblings = Vec::with_capacity(TREE_DEPTH);
        let mut is_right = Vec::with_capacity(TREE_DEPTH);
        for level in 0..TREE_DEPTH {
            siblings.push(empty[level]);
            is_right.push(level % 3 == 0);
        }
        let path = MerklePath { siblings, is_right };
        let leaf_salt = derive_leaf_salt_wide(key);
        let leaf_old = native_leaf_salted(account_id, BaseElement::new(balance), nonce, leaf_salt);
        let leaf_new =
            native_leaf_salted(account_id, BaseElement::new(balance + amount), nonce, leaf_salt);
        Scenario {
            public_inputs: CreditClimbPublicInputs {
                root_old: native_climb(leaf_old, &path),
                root_new: native_climb(leaf_new, &path),
                amount: BaseElement::new(amount),
            },
            account_id,
            balance,
            nonce,
            leaf_salt,
            path,
            amount,
        }
    }

    fn key_alice() -> [BaseElement; 4] {
        [
            BaseElement::new(0xA11CE_0001),
            BaseElement::new(0xA11CE_0002),
            BaseElement::new(0xA11CE_0003),
            BaseElement::new(0xA11CE_0004),
        ]
    }

    fn scenario(balance: u64, amount: u64) -> Scenario {
        scenario_con_clave(balance, amount, key_alice())
    }

    fn build(s: &Scenario) -> TraceTable<BaseElement> {
        build_trace(s.account_id, s.balance, s.nonce, s.leaf_salt, &s.path, s.amount)
    }

    /// Paridad traza↔nativo: las dos raíces coinciden con el ascenso nativo.
    #[test]
    fn trace_roots_match_native() {
        let s = scenario(1_000, 500);
        let trace = build(&s);
        for i in 0..4 {
            assert_eq!(
                trace.get(4 + i, ROW_ACCT_ROOT),
                s.public_inputs.root_old[i],
                "carril A, elem {i}"
            );
            assert_eq!(
                trace.get(LANE_B + 4 + i, ROW_ACCT_ROOT),
                s.public_inputs.root_new[i],
                "carril B, elem {i}"
            );
        }
    }

    /// El crédito legítimo produce una prueba que verifica.
    #[test]
    fn a_valid_credit_climb_verifies() {
        let s = scenario(1_000, 500);
        let trace = build(&s);
        let prover = CreditClimbProver::new(default_options());
        let proof = prover.prove(trace).expect("la generacion no deberia fallar");
        let min_opts = AcceptableOptions::OptionSet(vec![default_options()]);
        let ok = verify::<CreditClimbAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof,
            s.public_inputs.clone(),
            &min_opts,
        );
        assert!(ok.is_ok(), "un credito correcto debe verificar");
    }

    /// **Discriminante**: la prueba ata el IMPORTE. Verificar contra otro
    /// importe falla — sin esto, el reembolso acreditaría lo que quisiera.
    #[test]
    fn proof_does_not_verify_for_another_amount() {
        let s = scenario(1_000, 500);
        let trace = build(&s);
        let prover = CreditClimbProver::new(default_options());
        let proof = prover.prove(trace).expect("prueba");
        let mut pi = s.public_inputs.clone();
        pi.amount = BaseElement::new(501);
        let min_opts = AcceptableOptions::OptionSet(vec![default_options()]);
        let mal = verify::<CreditClimbAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof, pi, &min_opts,
        );
        assert!(mal.is_err(), "otro importe NO debe verificar");
    }

    /// **Discriminante del cerrojo #2 (§117)**: una prueba construida con el
    /// salt de OTRA clave asciende a una raíz distinta. Verificarla contra
    /// las raíces del titular legítimo falla — solo el emisor, dueño de SU
    /// salt, fabrica la subida que casa con SU hoja.
    #[test]
    fn proof_with_another_holders_salt_does_not_verify() {
        let legitimo = scenario(1_000, 500);
        let key_mallory = [
            BaseElement::new(0x5A110_0001),
            BaseElement::new(0x5A110_0002),
            BaseElement::new(0x5A110_0003),
            BaseElement::new(0x5A110_0004),
        ];
        // Mismo saldo e importe, salt ajeno: raíces distintas.
        let ajeno = scenario_con_clave(1_000, 500, key_mallory);
        let trace = build(&ajeno);
        let prover = CreditClimbProver::new(default_options());
        let proof = prover.prove(trace).expect("prueba");
        let min_opts = AcceptableOptions::OptionSet(vec![default_options()]);
        // Se verifica contra las raíces del LEGÍTIMO: no casan.
        let mal = verify::<CreditClimbAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof,
            legitimo.public_inputs.clone(),
            &min_opts,
        );
        assert!(
            mal.is_err(),
            "una subida con salt ajeno NO debe verificar contra la raíz del titular"
        );
    }
}
