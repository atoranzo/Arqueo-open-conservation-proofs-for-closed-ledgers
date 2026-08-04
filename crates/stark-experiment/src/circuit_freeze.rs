//! **Congelación de cuentas**: el supervisor puede bloquear una cuenta
//! bajo investigación.
//!
//! ## Por qué en circuito y no en la capa
//!
//! Congelar desde la capa sería que el operador se niegue a procesar. Y
//! **el operador ya puede censurar cualquier operación**: eso no añadiría
//! ninguna garantía, solo le pondría nombre.
//!
//! En circuito es distinto: la prueba de liquidación acredita que el
//! emisor **no estaba congelado** en esa raíz de estado. Cualquiera que
//! verifique la liquidación lo comprueba, **sin confiar en el operador**.
//!
//! ## El diseño: un árbol aparte, no un campo en la hoja
//!
//! Añadir un indicador a la hoja de cuenta obligaría a rehacer los seis
//! circuitos. En su lugar hay un **árbol de congelados** indexado por
//! número de cuenta, y la liquidación demuestra **no-pertenencia** —
//! exactamente la maquinaria que ya usa el doble gasto.
//!
//! Profundidad 24: hasta 16.777.216 cuentas. Se eligió porque su subida
//! **cabe en las 200 filas libres** del circuito de liquidación, sin
//! agrandar la traza y por tanto sin duplicar el coste de generación.
//!
//! ## Qué demuestra este circuito
//!
//! 1. **Dos custodios distintos** del conjunto autorizan.
//! 2. La cuenta pasa de **no congelada a congelada** (o al revés).
//! 3. El **contador público** incrementa en uno.
//!
//! ## ⚠️ Lo que NO resuelve
//!
//! - **No hay orden judicial ni motivo en el circuito.** Demuestra que
//!   dos custodios lo autorizaron, no que tuvieran razón.
//! - **No hay caducidad.** Una congelación dura hasta que alguien la
//!   levante.
//! - **No impide recibir.** Una cuenta congelada no puede gastar, pero sí
//!   puede seguir recibiendo. Impedir lo segundo exigiría comprobar
//!   también al receptor, y dejaría fondos en el limbo.

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

use crate::circuit_mint::ThresholdAuth;
use crate::circuit_threshold::{CustodianPath, CUSTODIAN_DEPTH, CUSTODIAN_DOMAIN};
use crate::merkle::{native_merge, Digest, MerklePath};
use crate::rescue_hash::{apply_sbox, NUM_ROUNDS, STATE_WIDTH};

/// Profundidad del árbol de congelados: hasta 16.777.216 cuentas.
///
/// Elegida porque su subida **cabe en las filas libres** del circuito de
/// liquidación (192 de 200), sin agrandar su traza.
pub const FROZEN_DEPTH: usize = 24;

/// Marca de cuenta congelada. Cualquier valor no nulo serviría; se usa
/// uno reconocible para que un volcado del árbol sea legible.
pub const FROZEN_MARK: u64 = 0x46524F5A; // "FROZ"

pub const CYCLE_LENGTH: usize = 8;
pub const TRACE_LENGTH: usize = 512;
pub const SEGMENT_LENGTH: usize = 8;
/// Segmentos: índice A, índice B, y `B − A − 1`.
pub const NUM_SEGMENTS: usize = 3;

// ===== Columnas =====
const LANE_B: usize = STATE_WIDTH; // 12
/// Bits de dirección de la subida al árbol de CONGELADOS. Ambos carriles
/// suben por el MISMO camino: la misma cuenta antes y después.
const COL_FBIT: usize = 24;
/// Bits de la subida al árbol de custodios: caminos DISTINTOS.
const COL_CBIT_A: usize = 25;
const COL_CBIT_B: usize = 26;
const COL_KEY_A: usize = 27;
const COL_KEY_B: usize = 28;
const COL_IDX_A: usize = 29;
const COL_IDX_B: usize = 30;
const COL_ACC_A: usize = 31;
const COL_ACC_B: usize = 32;
const COL_COUNT_OLD: usize = 33;
const COL_COUNT_NEW: usize = 34;
const COL_SBIT: usize = 35;
const COL_SACC: usize = 36;
pub const TRACE_WIDTH: usize = 37;

// ===== Filas =====
/// Subida al árbol de congelados: ciclos 0-23, filas 0..191.
const ROW_FROZEN_ROOT: usize = 191;
/// Derivación de identidades de custodio: ciclo 24.
const ROW_CUST_START: usize = 192;
/// Subida al conjunto de custodios: ciclos 25-28.
const ROW_CUST_ROOT: usize = 231;

// ===== Restricciones =====
const C_HASH_A: usize = 0;
const C_HASH_B: usize = C_HASH_A + STATE_WIDTH;
const C_CAP_A: usize = C_HASH_B + STATE_WIDTH; // 4
const C_CAP_B: usize = C_CAP_A + 4;
/// Colocación en el árbol de CONGELADOS: ambos carriles, mismo bit.
const C_FPLACE_A: usize = C_CAP_B + 4; // 4
const C_FPLACE_B: usize = C_FPLACE_A + 4;
/// Hermano compartido: es la misma posición del mismo árbol.
const C_FSIBLING: usize = C_FPLACE_B + 4; // 4
/// Colocación en el árbol de CUSTODIOS: cada carril con su bit.
const C_CPLACE_A: usize = C_FSIBLING + 4; // 4
const C_CPLACE_B: usize = C_CPLACE_A + 4;
const C_BIT_BOOL: usize = C_CPLACE_B + 4; // 3
const C_CUST_INPUT: usize = C_BIT_BOOL + 3; // 2
const C_ACC: usize = C_CUST_INPUT + 2; // 2
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
const P_FROZEN_LINK: usize = P_ARK2 + STATE_WIDTH;
const P_CUST_LINK: usize = P_FROZEN_LINK + 1;
const P_POW2: usize = P_CUST_LINK + 1;
/// Fila que arranca la derivacion de identidades de custodio. No esta
/// en `frozen_link` —es la transicion entre fases— asi que necesita
/// selector propio.
const P_SEL_CUST_START: usize = P_POW2 + 1;
const P_SEL_CUST_ROOT: usize = P_SEL_CUST_START + 1;
const P_FIRST_S: usize = P_SEL_CUST_ROOT + 1;
const P_CONT_S: usize = P_FIRST_S + 1;
const P_SEG_LINK: usize = P_CONT_S + 1;

type Blake3 = Blake3_256<BaseElement>;

/// Hoja del árbol de congelados para una cuenta.
///
/// Cero si no está congelada; la marca si lo está. Es lo que permite
/// demostrar **no-pertenencia** desde la liquidación sin revelar qué
/// cuentas hay congeladas.
pub fn frozen_leaf(frozen: bool) -> Digest {
    if frozen {
        [
            BaseElement::new(FROZEN_MARK),
            BaseElement::ZERO,
            BaseElement::ZERO,
            BaseElement::ZERO,
        ]
    } else {
        [BaseElement::ZERO; 4]
    }
}

/// Sube una hoja hasta la raíz del árbol de congelados, de forma nativa.
pub fn frozen_climb(leaf: Digest, path: &MerklePath) -> Digest {
    let mut current = leaf;
    for level in 0..FROZEN_DEPTH {
        current = if path.is_right[level] {
            native_merge(path.siblings[level], current)
        } else {
            native_merge(current, path.siblings[level])
        };
    }
    current
}

fn value_to_bits_be(value: u64) -> Vec<bool> {
    (0..SEGMENT_LENGTH)
        .map(|p| (value >> (SEGMENT_LENGTH - 1 - p)) & 1 == 1)
        .collect()
}

/// Construye la traza de una congelación o descongelación.
///
/// `frozen_before` y `frozen_after` permiten pasar valores incoherentes
/// para los tests negativos.
pub fn build_trace(
    auth: &ThresholdAuth,
    frozen_before: bool,
    frozen_after: bool,
    path: &MerklePath,
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
                 p: &MerklePath,
                 level: usize| {
        if p.is_right[level] {
            state[4..8].copy_from_slice(&p.siblings[level]);
            state[8..12].copy_from_slice(digest);
        } else {
            state[4..8].copy_from_slice(digest);
            state[8..12].copy_from_slice(&p.siblings[level]);
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

    // Carril A: estado ANTES. Carril B: DESPUES. Misma posicion.
    let mut state_a = [zero; STATE_WIDTH];
    let mut state_b = [zero; STATE_WIDTH];
    place(&mut state_a, &frozen_leaf(frozen_before), path, 0);
    place(&mut state_b, &frozen_leaf(frozen_after), path, 0);

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

            let next_cycle = (r + 1) / CYCLE_LENGTH;
            if next_cycle < FROZEN_DEPTH {
                place(&mut state_a, &digest_a, path, next_cycle);
                place(&mut state_b, &digest_b, path, next_cycle);
            } else if next_cycle == FROZEN_DEPTH {
                // Arranca la derivacion de identidades de custodio.
                state_a[4] = BaseElement::new(CUSTODIAN_DOMAIN);
                state_a[8] = auth.key_a;
                state_b[4] = BaseElement::new(CUSTODIAN_DOMAIN);
                state_b[8] = auth.key_b;
            } else {
                let level = next_cycle - FROZEN_DEPTH - 1;
                if level < CUSTODIAN_DEPTH {
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
        rows[r + 1][..STATE_WIDTH].copy_from_slice(&state_a);
        rows[r + 1][LANE_B..LANE_B + STATE_WIDTH].copy_from_slice(&state_b);
        rows[r + 1][COL_ACC_A] = acc_a;
        rows[r + 1][COL_ACC_B] = acc_b;
    }
    for r in ROW_CUST_ROOT..TRACE_LENGTH {
        rows[r][COL_ACC_A] = acc_a;
        rows[r][COL_ACC_B] = acc_b;
    }

    // Bits del arbol de congelados: mismo camino para ambos carriles.
    for level in 0..FROZEN_DEPTH {
        let bit = if path.is_right[level] {
            BaseElement::ONE
        } else {
            zero
        };
        for p in 0..CYCLE_LENGTH {
            rows[level * CYCLE_LENGTH + p][COL_FBIT] = bit;
        }
    }
    // Bits del arbol de custodios: caminos distintos.
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
        let cycle = FROZEN_DEPTH + 1 + level;
        for p in 0..CYCLE_LENGTH {
            rows[cycle * CYCLE_LENGTH + p][COL_CBIT_A] = ba;
            rows[cycle * CYCLE_LENGTH + p][COL_CBIT_B] = bb;
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
pub struct FreezePublicInputs {
    pub custodian_set_root: Digest,
    /// Raíz del árbol de congelados ANTES.
    pub frozen_root_old: Digest,
    /// Y DESPUÉS. **La identidad de la cuenta no aparece**: se sabe que
    /// alguien fue congelado, no quién.
    pub frozen_root_new: Digest,
    pub freeze_count_old: BaseElement,
    pub freeze_count_new: BaseElement,
}

impl ToElements<BaseElement> for FreezePublicInputs {
    fn to_elements(&self) -> Vec<BaseElement> {
        let mut out = self.custodian_set_root.to_vec();
        out.extend_from_slice(&self.frozen_root_old);
        out.extend_from_slice(&self.frozen_root_new);
        out.push(self.freeze_count_old);
        out.push(self.freeze_count_new);
        out
    }
}

pub struct FreezeAir {
    context: AirContext<BaseElement>,
    pub_inputs: FreezePublicInputs,
}

impl Air for FreezeAir {
    type BaseField = BaseElement;
    type PublicInputs = FreezePublicInputs;

    fn new(trace_info: TraceInfo, pub_inputs: Self::PublicInputs, options: ProofOptions) -> Self {
        assert_eq!(TRACE_WIDTH, trace_info.width());
        let full = vec![TRACE_LENGTH];

        let mut degrees = Vec::with_capacity(NUM_CONSTRAINTS);
        for _ in 0..2 * STATE_WIDTH {
            degrees.push(TransitionConstraintDegree::with_cycles(7, full.clone()));
        }
        // Capacidad (8): grado 1.
        for _ in 0..8 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }
        // Colocacion congelados (8) + hermano (4) + custodios (8): grado 2.
        for _ in 0..20 {
            degrees.push(TransitionConstraintDegree::with_cycles(2, full.clone()));
        }
        // Bits booleanos (3): grado 2 sin ciclo.
        for _ in 0..3 {
            degrees.push(TransitionConstraintDegree::new(2));
        }
        // Claves de custodio (2).
        for _ in 0..2 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }
        // Acumulador (2): DOS periodicas.
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

        FreezeAir {
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

        // Enlaces de la subida al arbol de congelados.
        let mut frozen_link = vec![zero; TRACE_LENGTH];
        for level in 0..FROZEN_DEPTH - 1 {
            frozen_link[level * CYCLE_LENGTH + 7] = one;
        }
        columns.push(frozen_link);

        // Enlaces del arbol de custodios.
        let mut cust_link = vec![zero; TRACE_LENGTH];
        let mut pow2 = vec![zero; TRACE_LENGTH];
        for level in 0..CUSTODIAN_DEPTH {
            let row = (FROZEN_DEPTH + level) * CYCLE_LENGTH + 7;
            cust_link[row] = one;
            pow2[row] = BaseElement::new(1u64 << level);
        }
        columns.push(cust_link);
        columns.push(pow2);

        let mut sel_cust_start = vec![zero; TRACE_LENGTH];
        sel_cust_start[ROW_FROZEN_ROOT] = one;
        columns.push(sel_cust_start);

        let mut sel_cust_root = vec![zero; TRACE_LENGTH];
        sel_cust_root[ROW_CUST_ROOT] = one;
        columns.push(sel_cust_root);

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
        let frozen_link = periodic[P_FROZEN_LINK];
        let cust_link = periodic[P_CUST_LINK];
        let pow2 = periodic[P_POW2];
        let sel_cust_start = periodic[P_SEL_CUST_START];
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

        let fbit = next[COL_FBIT];
        let cbit_a = next[COL_CBIT_A];
        let cbit_b = next[COL_CBIT_B];
        let any_link = frozen_link + cust_link;

        for i in 0..4 {
            result[C_CAP_A + i] = any_link * next[i];
            result[C_CAP_B + i] = any_link * next[LANE_B + i];

            let da = current[4 + i];
            let db = current[LANE_B + 4 + i];

            // --- Arbol de CONGELADOS: misma posicion, hermano compartido ---
            result[C_FPLACE_A + i] =
                frozen_link * ((E::ONE - fbit) * (next[4 + i] - da) + fbit * (next[8 + i] - da));
            result[C_FPLACE_B + i] = frozen_link
                * ((E::ONE - fbit) * (next[LANE_B + 4 + i] - db)
                    + fbit * (next[LANE_B + 8 + i] - db));

            let sib_a = (E::ONE - fbit) * next[8 + i] + fbit * next[4 + i];
            let sib_b =
                (E::ONE - fbit) * next[LANE_B + 8 + i] + fbit * next[LANE_B + 4 + i];
            result[C_FSIBLING + i] = frozen_link * (sib_a - sib_b);

            // --- Arbol de CUSTODIOS: caminos distintos ---
            result[C_CPLACE_A + i] =
                cust_link * ((E::ONE - cbit_a) * (next[4 + i] - da) + cbit_a * (next[8 + i] - da));
            result[C_CPLACE_B + i] = cust_link
                * ((E::ONE - cbit_b) * (next[LANE_B + 4 + i] - db)
                    + cbit_b * (next[LANE_B + 8 + i] - db));
        }

        result[C_BIT_BOOL] = current[COL_FBIT] * (current[COL_FBIT] - E::ONE);
        result[C_BIT_BOOL + 1] = current[COL_CBIT_A] * (current[COL_CBIT_A] - E::ONE);
        result[C_BIT_BOOL + 2] = current[COL_CBIT_B] * (current[COL_CBIT_B] - E::ONE);

        // Las claves de custodio entran en su derivacion de identidad,
        // en la fila que separa las dos fases.
        result[C_CUST_INPUT] = sel_cust_start * (next[8] - current[COL_KEY_A]);
        result[C_CUST_INPUT + 1] =
            sel_cust_start * (next[LANE_B + 8] - current[COL_KEY_B]);

        result[C_ACC] = cust_link * (next[COL_ACC_A] - (current[COL_ACC_A] + cbit_a * pow2));
        result[C_ACC + 1] = cust_link * (next[COL_ACC_B] - (current[COL_ACC_B] + cbit_b * pow2));

        result[C_ACC_FINAL] = sel_cust_root * (current[COL_ACC_A] - current[COL_IDX_A]);
        result[C_ACC_FINAL + 1] = sel_cust_root * (current[COL_ACC_B] - current[COL_IDX_B]);

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
        let mut a = Vec::with_capacity(44);

        // Fila 0: capacidad a cero en ambos carriles.
        for i in 0..4 {
            a.push(Assertion::single(i, 0, zero));
            a.push(Assertion::single(LANE_B + i, 0, zero));
        }
        a.push(Assertion::single(COL_ACC_A, 0, zero));
        a.push(Assertion::single(COL_ACC_B, 0, zero));

        // Raices del arbol de congelados, antes y despues.
        for i in 0..4 {
            a.push(Assertion::single(
                4 + i,
                ROW_FROZEN_ROOT,
                self.pub_inputs.frozen_root_old[i],
            ));
            a.push(Assertion::single(
                LANE_B + 4 + i,
                ROW_FROZEN_ROOT,
                self.pub_inputs.frozen_root_new[i],
            ));
        }

        // Las claves de custodio entran en su derivacion.
        for i in 0..4 {
            a.push(Assertion::single(i, ROW_CUST_START, zero));
            a.push(Assertion::single(LANE_B + i, ROW_CUST_START, zero));
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

        // **AUTORIDAD**: los dos carriles llegan a la raiz de custodios.
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
            self.pub_inputs.freeze_count_old,
        ));
        a.push(Assertion::single(
            COL_COUNT_NEW,
            0,
            self.pub_inputs.freeze_count_new,
        ));

        a
    }
}

pub struct FreezeProver {
    options: ProofOptions,
}

impl FreezeProver {
    pub fn new(options: ProofOptions) -> Self {
        Self { options }
    }
}

impl Prover for FreezeProver {
    type BaseField = BaseElement;
    type Air = FreezeAir;
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

    fn get_pub_inputs(&self, trace: &Self::Trace) -> FreezePublicInputs {
        FreezePublicInputs {
            custodian_set_root: [
                trace.get(4, ROW_CUST_ROOT),
                trace.get(5, ROW_CUST_ROOT),
                trace.get(6, ROW_CUST_ROOT),
                trace.get(7, ROW_CUST_ROOT),
            ],
            frozen_root_old: [
                trace.get(4, ROW_FROZEN_ROOT),
                trace.get(5, ROW_FROZEN_ROOT),
                trace.get(6, ROW_FROZEN_ROOT),
                trace.get(7, ROW_FROZEN_ROOT),
            ],
            frozen_root_new: [
                trace.get(LANE_B + 4, ROW_FROZEN_ROOT),
                trace.get(LANE_B + 5, ROW_FROZEN_ROOT),
                trace.get(LANE_B + 6, ROW_FROZEN_ROOT),
                trace.get(LANE_B + 7, ROW_FROZEN_ROOT),
            ],
            freeze_count_old: trace.get(COL_COUNT_OLD, 0),
            freeze_count_new: trace.get(COL_COUNT_NEW, 0),
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
    use crate::circuit_threshold::build_custodian_set;
    use winterfell::{verify, AcceptableOptions, BatchingMethod, FieldExtension};

    const COUNT_OLD: u64 = 4;

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
        (0..5).map(|i| BaseElement::new(0xC0570D1A + i)).collect()
    }

    /// Camino con direcciones MIXTAS: con todas iguales la traza degenera
    /// y winterfell rechaza los grados. Es la tercera vez que ocurre en
    /// este proyecto.
    fn frozen_path() -> MerklePath {
        let mut empty = vec![[BaseElement::ZERO; 4]];
        for k in 1..=FROZEN_DEPTH {
            let prev = empty[k - 1];
            empty.push(native_merge(prev, prev));
        }
        let mut siblings = Vec::with_capacity(FROZEN_DEPTH);
        let mut is_right = Vec::with_capacity(FROZEN_DEPTH);
        for level in 0..FROZEN_DEPTH {
            siblings.push(empty[level]);
            is_right.push(level % 3 == 0);
        }
        MerklePath { siblings, is_right }
    }

    fn valid_auth() -> ThresholdAuth {
        let keys = custodian_keys();
        let (_, paths) = build_custodian_set(&keys);
        ThresholdAuth {
            key_a: keys[1],
            index_a: 1,
            path_a: paths[1].clone(),
            key_b: keys[3],
            index_b: 3,
            path_b: paths[3].clone(),
        }
    }

    fn inputs(before: bool, after: bool) -> FreezePublicInputs {
        let path = frozen_path();
        FreezePublicInputs {
            custodian_set_root: build_custodian_set(&custodian_keys()).0,
            frozen_root_old: frozen_climb(frozen_leaf(before), &path),
            frozen_root_new: frozen_climb(frozen_leaf(after), &path),
            freeze_count_old: BaseElement::new(COUNT_OLD),
            freeze_count_new: BaseElement::new(COUNT_OLD + 1),
        }
    }

    fn run(auth: &ThresholdAuth, before: bool, after: bool, delta: u64) -> Result<(), String> {
        let trace = build_trace(auth, before, after, &frozen_path(), COUNT_OLD, delta);
        let prover = FreezeProver::new(default_options());

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
        verify::<FreezeAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof,
            inputs(before, after),
            &min_opts,
        )
        .map_err(|e| format!("verificacion fallo: {e:?}"))
    }

    /// La traza reconstruye las raíces que calcula la versión nativa.
    #[test]
    fn trace_roots_match_native() {
        let path = frozen_path();
        let trace = build_trace(&valid_auth(), false, true, &path, COUNT_OLD, 1);
        let esperada_old = frozen_climb(frozen_leaf(false), &path);
        let esperada_new = frozen_climb(frozen_leaf(true), &path);
        for i in 0..4 {
            assert_eq!(trace.get(4 + i, ROW_FROZEN_ROOT), esperada_old[i], "old {i}");
            assert_eq!(
                trace.get(LANE_B + 4 + i, ROW_FROZEN_ROOT),
                esperada_new[i],
                "new {i}"
            );
        }
        // ===== Y TODAS LAS ENTRADAS PÚBLICAS, NO SOLO LAS RAÍCES =====
        //
        // Comparar la estructura entera. En `circuit_send` la versión
        // parcial dejó pasar un campo heredado de otra operación y **costó
        // ocho rondas de diagnóstico**: el error de winterfell
        // —`InconsistentOodConstraintEvaluations`— apunta a las
        // restricciones, no a las entradas.
        let derivadas = FreezeProver::new(default_options()).get_pub_inputs(&trace);
        assert_eq!(
            derivadas.to_elements(),
            FreezePublicInputs {
                custodian_set_root: build_custodian_set(&custodian_keys()).0,
                frozen_root_old: esperada_old,
                frozen_root_new: esperada_new,
                freeze_count_old: BaseElement::new(COUNT_OLD),
                freeze_count_new: BaseElement::new(COUNT_OLD + 1),
            }.to_elements(),
            "las entradas DERIVADAS de la traza deben coincidir con las \
             DECLARADAS en todos sus campos"
        );

    }

    /// EL TEST CLAVE. No silencia el pánico.
    #[test]
    fn two_custodians_can_freeze_an_account() {
        let trace = build_trace(&valid_auth(), false, true, &frozen_path(), COUNT_OLD, 1);
        let prover = FreezeProver::new(default_options());
        let proof = prover.prove(trace).expect("la congelacion valida deberia probar");
        let min_opts = AcceptableOptions::OptionSet(vec![default_options()]);
        let v = verify::<FreezeAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof,
            inputs(false, true),
            &min_opts,
        );
        assert!(v.is_ok(), "{v:?}");
    }

    /// **EL TEST QUE LAS RESTRICCIONES VACÍAS HABRÍAN DEJADO PASAR.**
    ///
    /// Una primera versión de este circuito tenía `C_CUST_INPUT` escrita
    /// como `frozen_link * E::ZERO` —un marcador que se satisface
    /// siempre—. Con eso, **cualquiera podría congelar cuentas**.
    ///
    /// Una restricción idénticamente cero no falla ningún test negativo:
    /// por eso este caso concreto tiene que estar.
    #[test]
    fn a_non_custodian_cannot_freeze() {
        let keys = custodian_keys();
        let (_, paths) = build_custodian_set(&keys);
        let intruso = ThresholdAuth {
            key_a: BaseElement::new(0x1337),
            index_a: 1,
            path_a: paths[1].clone(),
            key_b: keys[3],
            index_b: 3,
            path_b: paths[3].clone(),
        };
        assert!(
            run(&intruso, false, true, 1).is_err(),
            "CRITICO: quien no es custodio NO debe poder congelar cuentas"
        );
    }

    /// Un solo custodio no basta.
    #[test]
    fn the_same_custodian_cannot_count_twice() {
        let keys = custodian_keys();
        let (_, paths) = build_custodian_set(&keys);
        let solo = ThresholdAuth {
            key_a: keys[2],
            index_a: 2,
            path_a: paths[2].clone(),
            key_b: keys[2],
            index_b: 2,
            path_b: paths[2].clone(),
        };
        assert!(run(&solo, false, true, 1).is_err());
    }

    /// **DESCONGELAR TAMBIÉN EXIGE DOS CUSTODIOS.**
    ///
    /// Si levantar una congelación fuese más fácil que imponerla, la
    /// congelación no valdría de nada.
    #[test]
    fn unfreezing_also_requires_two_custodians() {
        assert!(run(&valid_auth(), true, false, 1).is_ok(), "descongelar valido");
        let keys = custodian_keys();
        let (_, paths) = build_custodian_set(&keys);
        let intruso = ThresholdAuth {
            key_a: BaseElement::new(0x1337),
            index_a: 1,
            path_a: paths[1].clone(),
            key_b: keys[3],
            index_b: 3,
            path_b: paths[3].clone(),
        };
        assert!(
            run(&intruso, true, false, 1).is_err(),
            "CRITICO: si descongelar fuese mas facil que congelar, la \
             congelacion no valdria de nada"
        );
    }

    /// Un cambio silencioso se rechaza: cada intervención queda contada.
    #[test]
    fn a_silent_freeze_is_rejected() {
        assert!(run(&valid_auth(), false, true, 0).is_err());
        assert!(run(&valid_auth(), false, true, 2).is_err());
    }

    /// Declarar una raíz de custodios distinta debe fallar.
    #[test]
    fn wrong_custodian_set_root_is_rejected() {
        let trace = build_trace(&valid_auth(), false, true, &frozen_path(), COUNT_OLD, 1);
        let prover = FreezeProver::new(default_options());
        let proof = prover.prove(trace).expect("prove");
        let mut declared = inputs(false, true);
        declared.custodian_set_root = [BaseElement::new(999); 4];
        let min_opts = AcceptableOptions::OptionSet(vec![default_options()]);
        let v = verify::<FreezeAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
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

        let path = frozen_path();
        let trace = build_trace(&valid_auth(), false, true, &path, COUNT_OLD, 1);
        let esperada_old = frozen_climb(frozen_leaf(false), &path);
        let esperada_new = frozen_climb(frozen_leaf(true), &path);
        let rows = rows_of(&trace, TRACE_WIDTH, TRACE_LENGTH);

        let air = FreezeAir::new(
            TraceInfo::new(TRACE_WIDTH, TRACE_LENGTH),
            FreezePublicInputs {
                custodian_set_root: build_custodian_set(&custodian_keys()).0,
                frozen_root_old: esperada_old,
                frozen_root_new: esperada_new,
                freeze_count_old: BaseElement::new(COUNT_OLD),
                freeze_count_new: BaseElement::new(COUNT_OLD + 1),
            },
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
    /// **[§130] Instrumento de la medición apareada (paso 5).** Prove
    /// del LEGADO en su escenario honesto — construcción + prove
    /// dentro del reloj (patrón `metrics_33`). Correr a mano, en release:
    /// `cargo test --release -p stark-experiment medicion_130 -- --ignored --nocapture`
    #[test]
    #[ignore = "instrumento de medida, no comprobacion: correr a mano"]
    fn medicion_130_freeze_legado() {
        use std::time::Instant;
        let t0 = Instant::now();
        let keys = custodian_keys();
        let (_set_root, cpaths) = build_custodian_set(&keys);
        let auth = ThresholdAuth {
            key_a: keys[1], index_a: 1, path_a: cpaths[1].clone(),
            key_b: keys[3], index_b: 3, path_b: cpaths[3].clone(),
        };
        let trace = build_trace(&auth, false, true, &frozen_path(), COUNT_OLD, 1);
        let proof = FreezeProver::new(default_options())
            .prove(trace)
            .expect("el honesto debe probar");
        let ms = t0.elapsed().as_secs_f64() * 1000.0;
        println!(
            "[§130] freeze legado: prove {ms:.1} ms, proof {} bytes",
            proof.to_bytes().len()
        );
    }
}
