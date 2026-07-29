//! **Gobernanza**: actualización del conjunto de custodios.
//!
//! El último componente que faltaba, y el que resuelve una circularidad
//! incómoda.
//!
//! ## El problema
//!
//! Los custodios pueden **emitir dinero** y, desde la pieza de
//! recuperación, **reasignar cualquier cuenta**. Si el conjunto es
//! inmutable, un custodio comprometido conserva ese poder para siempre y
//! la única salida es crear un ledger nuevo.
//!
//! Pero si los propios custodios autorizan cambiar el conjunto, **dos
//! comprometidos pueden expulsar a los honestos y perpetuarse**. Es
//! circular.
//!
//! ## Opciones descartadas
//!
//! **Umbral más alto (3-de-N)**: reduce el riesgo, no lo elimina, y cada
//! firmante necesita su propio carril en la traza — el circuito se
//! dispara.
//!
//! **Retardo con veto**: en un nodo único **no protege nada**, porque el
//! operador controla el orden de las operaciones y por tanto el reloj.
//!
//! ## La solución: separar autoridades por nivel
//!
//! | Conjunto | Puede | Mutabilidad |
//! |---|---|---|
//! | **Custodios operativos** | Emitir, recuperar cuentas | **Cambiable** por gobernanza |
//! | **Gobernanza** | Cambiar el conjunto de custodios | **Inmutable** |
//!
//! **La circularidad no desaparece: se traslada.** Pero se traslada a
//! claves que se usan casi nunca y pueden protegerse físicamente —caja
//! fuerte, HSM sin conexión— frente a claves operativas que se usan a
//! diario y están expuestas.
//!
//! Es lo que hacen las jerarquías de claves reales. Y es honesto decir
//! dónde para la cadena: **el conjunto de gobernanza es inmutable**, y
//! cambiarlo exige crear un ledger nuevo, lo cual deja un rastro
//! imposible de ocultar.
//!
//! ## Qué demuestra
//!
//! 1. **Dos miembros distintos** del conjunto de gobernanza autorizan.
//! 2. Se pasa de una raíz de custodios a otra, **ambas públicas**.
//! 3. El **contador de cambios de gobernanza** incrementa en uno.
//!
//! ## Un circuito barato
//!
//! No toca el árbol de cuentas: 64 filas frente a las 512 de emisión. Un
//! cambio de gobernanza es raro y no mueve dinero.
//!
//! ## Lo que NO resuelve
//!
//! - **Si el conjunto de gobernanza se compromete, no hay salida** salvo
//!   crear un ledger nuevo. Es el final consciente de la cadena de
//!   autoridad.
//! - **No hay retardo ni ventana de impugnación.** Un cambio es inmediato.
//! - **El circuito no verifica que los nuevos custodios sean legítimos**,
//!   igual que en la recuperación: eso es responsabilidad de quien
//!   gobierna.

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

use crate::circuit_threshold::{CustodianPath, CUSTODIAN_DEPTH};
use crate::merkle::{native_merge, Digest};
use crate::rescue_hash::{apply_sbox, NUM_ROUNDS, STATE_WIDTH};

/// Dominio de derivación de identidades de gobernanza. **Distinto del de
/// custodios operativos**: una clave de custodio no puede hacerse pasar
/// por gobernanza ni al revés.
pub const GOVERNANCE_DOMAIN: u64 = 0x474F5645; // "GOVE"

pub const CYCLE_LENGTH: usize = 8;
pub const TRACE_LENGTH: usize = 64;
pub const SEGMENT_LENGTH: usize = 8;
/// Segmentos: índice A, índice B, y `B − A − 1`.
pub const NUM_SEGMENTS: usize = 3;

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
/// Contador público de cambios de gobernanza.
const COL_COUNT_OLD: usize = 32;
const COL_COUNT_NEW: usize = 33;
const COL_SBIT: usize = 34;
const COL_SACC: usize = 35;
pub const TRACE_WIDTH: usize = 36;

// ===== Filas =====
const ROW_ROOT: usize = 39;

// ===== Restricciones =====
const C_HASH_A: usize = 0;
const C_HASH_B: usize = C_HASH_A + STATE_WIDTH;
const C_CAP_A: usize = C_HASH_B + STATE_WIDTH; // 4
const C_CAP_B: usize = C_CAP_A + 4;
const C_PLACE_A: usize = C_CAP_B + 4; // 4
const C_PLACE_B: usize = C_PLACE_A + 4;
const C_BIT_BOOL: usize = C_PLACE_B + 4; // 2
const C_KEY_INPUT: usize = C_BIT_BOOL + 2; // 2
const C_ACC: usize = C_KEY_INPUT + 2; // 2
const C_ACC_FINAL: usize = C_ACC + 2; // 2
/// **EL CONTADOR INCREMENTA EXACTAMENTE EN UNO.**
const C_COUNT: usize = C_ACC_FINAL + 2; // 1
const C_TRANSPORT: usize = C_COUNT + 1; // 6
const C_SBIT_BOOL: usize = C_TRANSPORT + 6; // 2
const C_FIRST_S: usize = C_SBIT_BOOL + 2; // 2
const C_HORNER: usize = C_FIRST_S + 2; // 1
const C_SEG_LINK: usize = C_HORNER + 1; // NUM_SEGMENTS
const NUM_CONSTRAINTS: usize = C_SEG_LINK + NUM_SEGMENTS;

// ===== Periódicas =====
const P_HASH_FLAG: usize = 0;
const P_ARK1: usize = 1;
const P_ARK2: usize = P_ARK1 + STATE_WIDTH;
const P_TREE_LINK: usize = P_ARK2 + STATE_WIDTH;
const P_POW2: usize = P_TREE_LINK + 1;
const P_FIRST_ROW: usize = P_POW2 + 1;
const P_SEL_ROOT: usize = P_FIRST_ROW + 1;
const P_FIRST_S: usize = P_SEL_ROOT + 1;
const P_CONT_S: usize = P_FIRST_S + 1;
const P_SEG_LINK: usize = P_CONT_S + 1;

type Blake3 = Blake3_256<BaseElement>;

fn as_digest(x: BaseElement) -> Digest {
    [x, BaseElement::ZERO, BaseElement::ZERO, BaseElement::ZERO]
}

/// Identidad pública de un miembro de la gobernanza.
pub fn derive_governor_id(key: BaseElement) -> Digest {
    native_merge(
        as_digest(BaseElement::new(GOVERNANCE_DOMAIN)),
        as_digest(key),
    )
}

/// Construye el conjunto de gobernanza. Misma estructura que el de
/// custodios, distinto dominio.
pub fn build_governance_set(keys: &[BaseElement]) -> (Digest, Vec<CustodianPath>) {
    let size = 1usize << CUSTODIAN_DEPTH;
    assert!(keys.len() <= size, "demasiados gobernadores");

    let empty: Digest = [BaseElement::ZERO; 4];
    let mut leaves: Vec<Digest> = keys.iter().map(|k| derive_governor_id(*k)).collect();
    leaves.resize(size, empty);

    let mut levels = vec![leaves];
    for _ in 0..CUSTODIAN_DEPTH {
        let prev = levels.last().unwrap();
        let next: Vec<Digest> = prev.chunks(2).map(|p| native_merge(p[0], p[1])).collect();
        levels.push(next);
    }
    let root = levels[CUSTODIAN_DEPTH][0];

    let paths = (0..keys.len())
        .map(|index| {
            let mut siblings = Vec::with_capacity(CUSTODIAN_DEPTH);
            let mut is_right = Vec::with_capacity(CUSTODIAN_DEPTH);
            let mut idx = index;
            for level in 0..CUSTODIAN_DEPTH {
                siblings.push(levels[level][idx ^ 1]);
                is_right.push(idx % 2 == 1);
                idx /= 2;
            }
            CustodianPath { siblings, is_right }
        })
        .collect();

    (root, paths)
}

/// Autorización de dos miembros de la gobernanza.
#[derive(Clone, Debug)]
pub struct GovernanceAuth {
    pub key_a: BaseElement,
    pub index_a: u64,
    pub path_a: CustodianPath,
    pub key_b: BaseElement,
    pub index_b: u64,
    pub path_b: CustodianPath,
}

fn value_to_bits_be(value: u64) -> Vec<bool> {
    (0..SEGMENT_LENGTH)
        .map(|p| (value >> (SEGMENT_LENGTH - 1 - p)) & 1 == 1)
        .collect()
}

/// Construye la traza de un cambio de gobernanza.
///
/// `count_delta` permite no incrementar el contador, para el test que
/// comprueba que un cambio silencioso se rechaza.
pub fn build_trace(
    auth: &GovernanceAuth,
    count_old: u64,
    count_delta: u64,
) -> TraceTable<BaseElement> {
    let zero = BaseElement::ZERO;
    let c_count_old = BaseElement::new(count_old);
    let c_count_new = c_count_old + BaseElement::new(count_delta);

    let mut rows: Vec<Vec<BaseElement>> = vec![vec![zero; TRACE_WIDTH]; TRACE_LENGTH];

    for row in rows.iter_mut() {
        row[COL_KEY_A] = auth.key_a;
        row[COL_KEY_B] = auth.key_b;
        row[COL_IDX_A] = BaseElement::new(auth.index_a);
        row[COL_IDX_B] = BaseElement::new(auth.index_b);
        row[COL_COUNT_OLD] = c_count_old;
        row[COL_COUNT_NEW] = c_count_new;
    }

    let diff = BaseElement::new(auth.index_b) - BaseElement::new(auth.index_a) - BaseElement::ONE;
    let segment_values = [auth.index_a, auth.index_b, diff.as_int()];
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

    let place = |state: &mut [BaseElement; STATE_WIDTH],
                 digest: &Digest,
                 path: &CustodianPath,
                 level: usize| {
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
    state_a[4] = BaseElement::new(GOVERNANCE_DOMAIN);
    state_a[8] = auth.key_a;
    state_b[4] = BaseElement::new(GOVERNANCE_DOMAIN);
    state_b[8] = auth.key_b;

    rows[0][..STATE_WIDTH].copy_from_slice(&state_a);
    rows[0][LANE_B..LANE_B + STATE_WIDTH].copy_from_slice(&state_b);

    let mut acc_a = zero;
    let mut acc_b = zero;

    for r in 0..ROW_ROOT {
        let pos = r % CYCLE_LENGTH;
        if pos < NUM_ROUNDS {
            Rp64_256::apply_round(&mut state_a, pos);
            Rp64_256::apply_round(&mut state_b, pos);
        } else {
            let digest_a: Digest = [state_a[4], state_a[5], state_a[6], state_a[7]];
            let digest_b: Digest = [state_b[4], state_b[5], state_b[6], state_b[7]];
            state_a = [zero; STATE_WIDTH];
            state_b = [zero; STATE_WIDTH];
            let level = r / CYCLE_LENGTH;
            if level < CUSTODIAN_DEPTH {
                place(&mut state_a, &digest_a, &auth.path_a, level);
                place(&mut state_b, &digest_b, &auth.path_b, level);
                let p = BaseElement::new(1u64 << level);
                if auth.path_a.is_right[level] {
                    acc_a += p;
                }
                if auth.path_b.is_right[level] {
                    acc_b += p;
                }
            }
        }
        rows[r + 1][..STATE_WIDTH].copy_from_slice(&state_a);
        rows[r + 1][LANE_B..LANE_B + STATE_WIDTH].copy_from_slice(&state_b);
        rows[r + 1][COL_ACC_A] = acc_a;
        rows[r + 1][COL_ACC_B] = acc_b;
    }
    for r in ROW_ROOT..TRACE_LENGTH {
        rows[r][COL_ACC_A] = acc_a;
        rows[r][COL_ACC_B] = acc_b;
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
            rows[(1 + level) * CYCLE_LENGTH + p][COL_BIT_A] = ba;
            rows[(1 + level) * CYCLE_LENGTH + p][COL_BIT_B] = bb;
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
/// **Las dos raíces de custodios son públicas**: cualquiera puede
/// comprobar de qué conjunto a cuál se pasó. Y el contador hace contables
/// los cambios.
#[derive(Clone, Debug)]
pub struct GovernancePublicInputs {
    /// Raíz del conjunto de GOBERNANZA. Inmutable.
    pub governance_set_root: Digest,
    /// Conjunto de custodios ANTES del cambio.
    pub custodian_root_old: Digest,
    /// Y DESPUÉS.
    pub custodian_root_new: Digest,
    pub change_count_old: BaseElement,
    pub change_count_new: BaseElement,
}

impl ToElements<BaseElement> for GovernancePublicInputs {
    fn to_elements(&self) -> Vec<BaseElement> {
        let mut out = self.governance_set_root.to_vec();
        out.extend_from_slice(&self.custodian_root_old);
        out.extend_from_slice(&self.custodian_root_new);
        out.push(self.change_count_old);
        out.push(self.change_count_new);
        out
    }
}

pub struct GovernanceAir {
    context: AirContext<BaseElement>,
    pub_inputs: GovernancePublicInputs,
}

impl Air for GovernanceAir {
    type BaseField = BaseElement;
    type PublicInputs = GovernancePublicInputs;

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
        for _ in 0..8 {
            degrees.push(TransitionConstraintDegree::with_cycles(2, full.clone()));
        }
        for _ in 0..2 {
            degrees.push(TransitionConstraintDegree::new(2));
        }
        // Claves (2).
        for _ in 0..2 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }
        // Acumulador (2): DOS columnas periódicas.
        for _ in 0..2 {
            degrees.push(TransitionConstraintDegree::with_cycles(
                1,
                vec![TRACE_LENGTH, TRACE_LENGTH],
            ));
        }
        // Acumulador final (2).
        for _ in 0..2 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }
        // Contador (1) + transporte (6): sin ciclo.
        for _ in 0..7 {
            degrees.push(TransitionConstraintDegree::new(1));
        }
        for _ in 0..2 {
            degrees.push(TransitionConstraintDegree::new(2));
        }
        for _ in 0..(3 + NUM_SEGMENTS) {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }

        assert_eq!(degrees.len(), NUM_CONSTRAINTS, "cuenta de grados");

        GovernanceAir {
            context: AirContext::new(trace_info, degrees, 28, options),
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
        for r in 0..=ROW_ROOT {
            if r % CYCLE_LENGTH < NUM_ROUNDS {
                hash_flag[r] = one;
            }
        }
        columns.push(hash_flag);

        for ark in [true, false] {
            for i in 0..STATE_WIDTH {
                let mut col = vec![zero; TRACE_LENGTH];
                for r in 0..=ROW_ROOT {
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

        let mut tree_link = vec![zero; TRACE_LENGTH];
        let mut pow2 = vec![zero; TRACE_LENGTH];
        for level in 0..CUSTODIAN_DEPTH {
            let row = level * CYCLE_LENGTH + 7;
            tree_link[row] = one;
            pow2[row] = BaseElement::new(1u64 << level);
        }
        columns.push(tree_link);
        columns.push(pow2);

        let mut first_row = vec![zero; TRACE_LENGTH];
        first_row[0] = one;
        columns.push(first_row);

        let mut sel_root = vec![zero; TRACE_LENGTH];
        sel_root[ROW_ROOT] = one;
        columns.push(sel_root);

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
        let tree_link = periodic[P_TREE_LINK];
        let pow2 = periodic[P_POW2];
        let first_row = periodic[P_FIRST_ROW];
        let sel_root = periodic[P_SEL_ROOT];
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

        for i in 0..4 {
            result[C_CAP_A + i] = tree_link * next[i];
            result[C_CAP_B + i] = tree_link * next[LANE_B + i];

            let da = current[4 + i];
            result[C_PLACE_A + i] =
                tree_link * ((E::ONE - bit_a) * (next[4 + i] - da) + bit_a * (next[8 + i] - da));

            let db = current[LANE_B + 4 + i];
            result[C_PLACE_B + i] = tree_link
                * ((E::ONE - bit_b) * (next[LANE_B + 4 + i] - db)
                    + bit_b * (next[LANE_B + 8 + i] - db));
        }

        result[C_BIT_BOOL] = current[COL_BIT_A] * (current[COL_BIT_A] - E::ONE);
        result[C_BIT_BOOL + 1] = current[COL_BIT_B] * (current[COL_BIT_B] - E::ONE);

        result[C_KEY_INPUT] = first_row * (current[8] - current[COL_KEY_A]);
        result[C_KEY_INPUT + 1] = first_row * (current[LANE_B + 8] - current[COL_KEY_B]);

        result[C_ACC] = tree_link * (next[COL_ACC_A] - (current[COL_ACC_A] + bit_a * pow2));
        result[C_ACC + 1] = tree_link * (next[COL_ACC_B] - (current[COL_ACC_B] + bit_b * pow2));

        result[C_ACC_FINAL] = sel_root * (current[COL_ACC_A] - current[COL_IDX_A]);
        result[C_ACC_FINAL + 1] = sel_root * (current[COL_ACC_B] - current[COL_IDX_B]);

        // ===== EL CONTADOR INCREMENTA EXACTAMENTE EN UNO =====
        result[C_COUNT] = current[COL_COUNT_NEW] - (current[COL_COUNT_OLD] + E::ONE);

        let transport = [
            COL_KEY_A,
            COL_KEY_B,
            COL_IDX_A,
            COL_IDX_B,
            COL_COUNT_OLD,
            COL_COUNT_NEW,
        ];
        for (k, col) in transport.iter().enumerate() {
            result[C_TRANSPORT + k] = next[*col] - current[*col];
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
        let mut a = Vec::with_capacity(28);

        for i in 0..4 {
            a.push(Assertion::single(i, 0, zero));
            a.push(Assertion::single(LANE_B + i, 0, zero));
        }
        a.push(Assertion::single(
            4,
            0,
            BaseElement::new(GOVERNANCE_DOMAIN),
        ));
        a.push(Assertion::single(
            LANE_B + 4,
            0,
            BaseElement::new(GOVERNANCE_DOMAIN),
        ));
        for i in 5..8 {
            a.push(Assertion::single(i, 0, zero));
            a.push(Assertion::single(LANE_B + i, 0, zero));
        }
        a.push(Assertion::single(COL_ACC_A, 0, zero));
        a.push(Assertion::single(COL_ACC_B, 0, zero));

        // **Los dos carriles llegan a la raíz del conjunto de
        // GOBERNANZA**: ambos autorizantes pertenecen a él.
        for i in 0..4 {
            a.push(Assertion::single(
                4 + i,
                ROW_ROOT,
                self.pub_inputs.governance_set_root[i],
            ));
            a.push(Assertion::single(
                LANE_B + 4 + i,
                ROW_ROOT,
                self.pub_inputs.governance_set_root[i],
            ));
        }

        a.push(Assertion::single(
            COL_COUNT_OLD,
            0,
            self.pub_inputs.change_count_old,
        ));
        a.push(Assertion::single(
            COL_COUNT_NEW,
            0,
            self.pub_inputs.change_count_new,
        ));

        a
    }
}

pub struct GovernanceProver {
    options: ProofOptions,
    /// Las raíces de custodios no se calculan en la traza: son datos que
    /// la gobernanza declara. El circuito demuestra **quién autoriza el
    /// cambio**, no de dónde salen las raíces.
    custodian_root_old: Digest,
    custodian_root_new: Digest,
}

impl GovernanceProver {
    pub fn new(
        options: ProofOptions,
        custodian_root_old: Digest,
        custodian_root_new: Digest,
    ) -> Self {
        Self {
            options,
            custodian_root_old,
            custodian_root_new,
        }
    }
}

impl Prover for GovernanceProver {
    type BaseField = BaseElement;
    type Air = GovernanceAir;
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

    fn get_pub_inputs(&self, trace: &Self::Trace) -> GovernancePublicInputs {
        GovernancePublicInputs {
            governance_set_root: [
                trace.get(4, ROW_ROOT),
                trace.get(5, ROW_ROOT),
                trace.get(6, ROW_ROOT),
                trace.get(7, ROW_ROOT),
            ],
            custodian_root_old: self.custodian_root_old,
            custodian_root_new: self.custodian_root_new,
            change_count_old: trace.get(COL_COUNT_OLD, 0),
            change_count_new: trace.get(COL_COUNT_NEW, 0),
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
    use crate::circuit_threshold::{build_custodian_set, derive_custodian_id};
    use winterfell::{verify, AcceptableOptions, BatchingMethod, FieldExtension};

    const COUNT_OLD: u64 = 2;

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

    fn governance_keys() -> Vec<BaseElement> {
        vec![
            BaseElement::new(0x60_5E_00),
            BaseElement::new(0x60_5E_01),
            BaseElement::new(0x60_5E_02),
            BaseElement::new(0x60_5E_03),
        ]
    }

    fn roots() -> (Digest, Digest) {
        let old = build_custodian_set(&[BaseElement::new(1), BaseElement::new(2)]).0;
        let new = build_custodian_set(&[
            BaseElement::new(1),
            BaseElement::new(2),
            BaseElement::new(3),
        ])
        .0;
        (old, new)
    }

    fn valid_auth() -> GovernanceAuth {
        let keys = governance_keys();
        let (_, paths) = build_governance_set(&keys);
        // Indices 1 y 3, NO 0 y 2.
        //
        // El indice 0 tiene todos los bits de camino a cero, lo que deja
        // `COL_BIT_A` identicamente nula y hace que toda expresion que la
        // use colapse de grado. Winterfell calcula el grado REAL de las
        // evaluaciones y lo rechaza, aunque la declaracion sea correcta
        // para cualquier traza no degenerada.
        //
        // Es el mismo fenomeno que obligo a usar caminos mixtos en
        // `circuit_mint` y en `circuit_audit`: **un caso de prueba
        // demasiado simple no ejercita las restricciones**.
        GovernanceAuth {
            key_a: keys[1],
            index_a: 1,
            path_a: paths[1].clone(),
            key_b: keys[3],
            index_b: 3,
            path_b: paths[3].clone(),
        }
    }

    fn run(auth: &GovernanceAuth, count_delta: u64) -> Result<(), String> {
        let (r_old, r_new) = roots();
        let trace = build_trace(auth, COUNT_OLD, count_delta);
        let prover = GovernanceProver::new(default_options(), r_old, r_new);
        let declared = GovernancePublicInputs {
            governance_set_root: build_governance_set(&governance_keys()).0,
            custodian_root_old: r_old,
            custodian_root_new: r_new,
            change_count_old: BaseElement::new(COUNT_OLD),
            change_count_new: BaseElement::new(COUNT_OLD + 1),
        };

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
        verify::<GovernanceAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof, declared, &min_opts,
        )
        .map_err(|e| format!("verificacion fallo: {e:?}"))
    }

    /// **LOS DOS DOMINIOS ESTÁN SEPARADOS.**
    ///
    /// Una clave de custodio operativo no puede hacerse pasar por
    /// gobernanza. Sin esta separación, la jerarquía de autoridades sería
    /// ficticia: quien controla el nivel operativo controlaría el de
    /// gobernanza.
    #[test]
    fn governance_and_custodian_domains_are_separated() {
        let k = BaseElement::new(12345);
        assert_ne!(
            derive_governor_id(k),
            derive_custodian_id(k),
            "CRITICO: la misma clave no debe valer en ambos niveles, o la \
             jerarquia de autoridades seria ficticia"
        );
    }

    /// EL TEST CLAVE. No silencia el pánico.
    #[test]
    fn two_governors_can_change_the_custodian_set() {
        let (r_old, r_new) = roots();
        assert_ne!(r_old, r_new, "el conjunto debe cambiar de verdad");

        let trace = build_trace(&valid_auth(), COUNT_OLD, 1);
        let prover = GovernanceProver::new(default_options(), r_old, r_new);
        let proof = prover.prove(trace).expect("el cambio valido deberia probar");

        let min_opts = AcceptableOptions::OptionSet(vec![default_options()]);
        let v = verify::<GovernanceAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof,
            GovernancePublicInputs {
                governance_set_root: build_governance_set(&governance_keys()).0,
                custodian_root_old: r_old,
                custodian_root_new: r_new,
                change_count_old: BaseElement::new(COUNT_OLD),
                change_count_new: BaseElement::new(COUNT_OLD + 1),
            },
            &min_opts,
        );
        assert!(v.is_ok(), "{v:?}");
    }

    /// **UN SOLO GOBERNADOR NO PUEDE CAMBIAR EL CONJUNTO.**
    #[test]
    fn the_same_governor_cannot_count_twice() {
        let keys = governance_keys();
        let (_, paths) = build_governance_set(&keys);
        let solo = GovernanceAuth {
            key_a: keys[1],
            index_a: 1,
            path_a: paths[1].clone(),
            key_b: keys[1],
            index_b: 1,
            path_b: paths[1].clone(),
        };
        assert!(
            run(&solo, 1).is_err(),
            "CRITICO: un gobernador contando dos veces convertiria el 2-de-N \
             en 1-de-N sobre la autoridad MAS alta del sistema"
        );
    }

    /// **UN CUSTODIO OPERATIVO NO PUEDE GOBERNAR.**
    ///
    /// Es la prueba de que la jerarquía funciona: quien puede emitir y
    /// recuperar cuentas NO puede cambiar quién tiene ese poder.
    #[test]
    fn a_custodian_cannot_change_the_custodian_set() {
        let keys = governance_keys();
        let (_, paths) = build_governance_set(&keys);
        // Una clave de custodio operativo, no de gobernanza.
        let intruder = GovernanceAuth {
            key_a: BaseElement::new(0xC0570D1A),
            index_a: 1,
            path_a: paths[1].clone(),
            key_b: keys[3],
            index_b: 3,
            path_b: paths[3].clone(),
        };
        assert!(
            run(&intruder, 1).is_err(),
            "CRITICO: quien puede emitir y recuperar NO debe poder cambiar \
             quien tiene ese poder"
        );
    }

    /// **UN CAMBIO SILENCIOSO SE RECHAZA.**
    #[test]
    fn a_silent_governance_change_is_rejected() {
        assert!(run(&valid_auth(), 0).is_err());
        assert!(run(&valid_auth(), 2).is_err());
    }

    /// Declarar una raíz de gobernanza distinta.
    #[test]
    fn wrong_governance_root_is_rejected() {
        let (r_old, r_new) = roots();
        let trace = build_trace(&valid_auth(), COUNT_OLD, 1);
        let prover = GovernanceProver::new(default_options(), r_old, r_new);
        let proof = prover.prove(trace).expect("prove");
        let min_opts = AcceptableOptions::OptionSet(vec![default_options()]);
        let v = verify::<GovernanceAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof,
            GovernancePublicInputs {
                governance_set_root: [BaseElement::new(999); 4],
                custodian_root_old: r_old,
                custodian_root_new: r_new,
                change_count_old: BaseElement::new(COUNT_OLD),
                change_count_new: BaseElement::new(COUNT_OLD + 1),
            },
            &min_opts,
        );
        assert!(v.is_err());
    }

    /// Las entradas públicas declaradas de un cambio válido.
    ///
    /// Existían **repetidas en cada test**, construidas a mano. Que un
    /// campo se copiara mal en una de las copias no lo detectaba nada.
    fn declared_inputs() -> GovernancePublicInputs {
        let (r_old, r_new) = roots();
        GovernancePublicInputs {
            governance_set_root: build_governance_set(&governance_keys()).0,
            custodian_root_old: r_old,
            custodian_root_new: r_new,
            change_count_old: BaseElement::new(COUNT_OLD),
            change_count_new: BaseElement::new(COUNT_OLD + 1),
        }
    }

    /// **SEPARA "LA TRAZA ESTÁ MAL" DE "LAS RESTRICCIONES ESTÁN MAL".**
    ///
    /// Este circuito **no tenía ningún test de puntos de referencia**, pese
    /// a estar en producción. Se detectó al inventariar cuáles comparaban
    /// sus entradas públicas con lo que la traza produce (`AUDITORIA.md`
    /// §11).
    ///
    /// Compara la **estructura entera**, sus cinco campos: en
    /// `circuit_send` la versión parcial dejó pasar un campo heredado y
    /// **costó ocho rondas de diagnóstico**.
    #[test]
    fn trace_landmarks_match_native() {
        let (r_old, r_new) = roots();
        assert_ne!(r_old, r_new, "el conjunto debe cambiar de verdad");

        let trace = build_trace(&valid_auth(), COUNT_OLD, 1);

        // La raíz del conjunto de GOBERNANZA, que la traza calcula subiendo
        // el árbol desde las identidades de los dos gobernadores.
        let esperada = build_governance_set(&governance_keys()).0;
        for i in 0..4 {
            assert_eq!(
                trace.get(4 + i, ROW_ROOT),
                esperada[i],
                "raiz de gobernanza, elemento {i}"
            );
        }

        let derivadas = GovernanceProver::new(default_options(), r_old, r_new)
            .get_pub_inputs(&trace);
        assert_eq!(
            derivadas.to_elements(),
            declared_inputs().to_elements(),
            "las entradas DERIVADAS de la traza deben coincidir con las \
             DECLARADAS en todos sus campos"
        );
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

        let trace = build_trace(&valid_auth(), COUNT_OLD, 1);
        let rows = rows_of(&trace, TRACE_WIDTH, TRACE_LENGTH);

        let air = GovernanceAir::new(
            TraceInfo::new(TRACE_WIDTH, TRACE_LENGTH),
            declared_inputs(),
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
