//! Etapa 2: el circuito de cumplimiento COMPLETO en una sola traza AIR —
//! equivalente STARK de `ComplianceCircuitWithState` (Groth16) y
//! `ComplianceCircuit` (Halo2).
//!
//! ## Qué demuestra, con qué visibilidad
//!
//! Públicos: `state_root`, `regulatory_limit`, `nullifier`.
//! Privados: `account_id`, `balance`, `nonce`, `amount`, y el camino de
//! Merkle completo (hermanos y bits de dirección).
//!
//! 1. `amount <= balance` y `amount <= regulatory_limit` (solvencia).
//! 2. `leaf = Rescue(Rescue(account_id, balance), nonce)` pertenece al
//!    árbol cuya raíz es `state_root` (32 niveles).
//! 3. `nullifier = Rescue(Rescue(DOMAIN, account_id), nonce)`.
//!
//! ## Arquitectura: dos carriles en paralelo + transporte
//!
//! Traza de 512 filas × 20 columnas:
//! - **Carril de hash** (cols 0..12: estado Rescue + bit de dirección):
//!   36 ciclos de 8 filas = 288 filas. Ciclos 0-1: hoja. Ciclos 2-33:
//!   árbol. Ciclos 34-35: nullifier. Filas 288..511 inactivas.
//! - **Carril de solvencia** (cols 13..14: bit, acc de Horner): los 4
//!   segmentos de 64 filas validados en `solvency.rs`, filas 0..255.
//! - **Columnas de transporte** (cols 15..19: c_account, c_bal, c_amt,
//!   c_lim, c_nonce): constantes en toda la traza (`next - cur = 0`).
//!
//! ## El "cableado" que hace que esto sea UN circuito y no tres
//!
//! El problema de AIR (sin `constrain_equal` entre celdas lejanas) se
//! resuelve con las columnas de transporte: el MISMO `c_bal` que el link
//! de solvencia ata al acumulador (fila 63) es el que se fuerza como
//! entrada de la hoja (fila 0 del carril de hash). Lo mismo con
//! `c_account` (hoja + nullifier) y `c_nonce` (hoja + nullifier). Sin
//! esto, la solvencia y el árbol podrían hablar de saldos distintos y el
//! circuito sería tres pruebas independientes disfrazadas de una.
//!
//! ## Por qué todos los selectores son de longitud completa (512)
//!
//! La estructura ya no es uniformemente periódica: hay regiones inactivas
//! y eventos únicos (el enlace al nullifier ocurre UNA vez, en la fila
//! 271). La técnica de columnas periódicas de longitud igual a la traza
//! ya quedó validada con los links de `solvency.rs`.
//!
//! ## Diferencia documentada con los otros backends
//!
//! La hoja aquí se construye sobre digests de 4 elementos (escalares con
//! relleno de ceros), porque `Rp64_256` es una esponja de estado 12 — no
//! es bit-compatible con las hojas de Poseidon de Groth16/Halo2 (cuerpos
//! distintos, como ya documenta `settlement-prover`). Y el árbol tiene 32
//! niveles, no 20 (requisito de potencia de dos de la traza; elegido por
//! encima, no por debajo).

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

use crate::merkle::{native_merge, Digest, MerklePath};
use crate::nullifier::NULLIFIER_DOMAIN;
use crate::rescue_hash::{apply_sbox, NUM_ROUNDS, STATE_WIDTH};

pub const TREE_DEPTH: usize = 32;
pub const CYCLE_LENGTH: usize = 8;
/// Ciclos de hash: 2 (hoja) + 32 (árbol) + 2 (nullifier).
pub const HASH_CYCLES: usize = 2 + TREE_DEPTH + 2;
/// Filas activas del carril de hash: 288.
pub const HASH_ROWS: usize = HASH_CYCLES * CYCLE_LENGTH;
/// Longitud de la traza: la potencia de dos por encima de 288.
pub const TRACE_LENGTH: usize = 512;
/// Segmentos de solvencia (idénticos a `solvency.rs`).
pub const SEGMENT_LENGTH: usize = 64;
pub const MAX_VALUE: u64 = (1u64 << 63) - 1;

const TRACE_WIDTH: usize = 20;
// Carril de hash.
const COL_MBIT: usize = 12; // bit de dirección del árbol
// Carril de solvencia.
const COL_SBIT: usize = 13;
const COL_SACC: usize = 14;
// Transporte.
const COL_ACCOUNT: usize = 15;
const COL_BAL: usize = 16;
const COL_AMT: usize = 17;
const COL_LIM: usize = 18;
const COL_NONCE: usize = 19;

// Filas de eventos únicos del carril de hash.
const ROW_LEAF_LINK_1: usize = 7; // inner_leaf -> (inner, nonce)
const ROW_ROOT: usize = (2 + TREE_DEPTH) * CYCLE_LENGTH - 1; // 271
const ROW_NULL_START: usize = ROW_ROOT + 1; // 272
const ROW_NULL_LINK: usize = ROW_NULL_START + CYCLE_LENGTH - 1; // 279
const ROW_NULLIFIER: usize = HASH_ROWS - 1; // 287

type Blake3 = Blake3_256<BaseElement>;

fn as_digest(x: BaseElement) -> Digest {
    [x, BaseElement::ZERO, BaseElement::ZERO, BaseElement::ZERO]
}

/// Hoja nativa: Rescue(Rescue(account, balance), nonce).
pub fn native_leaf(account_id: BaseElement, balance: BaseElement, nonce: BaseElement) -> Digest {
    let inner = native_merge(as_digest(account_id), as_digest(balance));
    native_merge(inner, as_digest(nonce))
}

/// Nullifier nativo (idéntico a `nullifier::native_nullifier`).
pub fn native_nullifier(account_id: BaseElement, nonce: BaseElement) -> Digest {
    let domain = as_digest(BaseElement::new(NULLIFIER_DOMAIN));
    let inner = native_merge(domain, as_digest(account_id));
    native_merge(inner, as_digest(nonce))
}

/// Raíz nativa siguiendo el camino desde la hoja.
pub fn native_root(leaf: Digest, path: &MerklePath) -> Digest {
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

fn value_to_bits_be(value: u64) -> Vec<bool> {
    (0..SEGMENT_LENGTH)
        .map(|p| (value >> (SEGMENT_LENGTH - 1 - p)) & 1 == 1)
        .collect()
}

/// Construye la traza completa precomputando todas las filas. NO valida
/// las entradas: la solidez del circuito debe rechazar los casos
/// inválidos por sí misma (y los tests lo comprueban así).
pub fn build_trace(
    account_id: BaseElement,
    balance: u64,
    nonce: BaseElement,
    amount: u64,
    limit: u64,
    path: &MerklePath,
) -> TraceTable<BaseElement> {
    assert_eq!(path.siblings.len(), TREE_DEPTH);
    assert_eq!(path.is_right.len(), TREE_DEPTH);

    let c_bal = BaseElement::new(balance);
    let c_amt = BaseElement::new(amount);
    let c_lim = BaseElement::new(limit);

    let diff_bal = (c_bal - c_amt).as_int();
    let diff_lim = (c_lim - c_amt).as_int();
    let segment_values = [balance, amount, diff_bal, diff_lim];
    let segment_bits: Vec<Vec<bool>> =
        segment_values.iter().map(|v| value_to_bits_be(*v)).collect();

    let zero = BaseElement::ZERO;
    let mut rows: Vec<Vec<BaseElement>> = vec![vec![zero; TRACE_WIDTH]; TRACE_LENGTH];

    // --- Transporte: constante en todas las filas ---
    for row in rows.iter_mut() {
        row[COL_ACCOUNT] = account_id;
        row[COL_BAL] = c_bal;
        row[COL_AMT] = c_amt;
        row[COL_LIM] = c_lim;
        row[COL_NONCE] = nonce;
    }

    // --- Carril de solvencia: filas 0..255 ---
    for seg in 0..4 {
        let mut acc = zero;
        for p in 0..SEGMENT_LENGTH {
            let r = seg * SEGMENT_LENGTH + p;
            let bit = if segment_bits[seg][p] {
                BaseElement::ONE
            } else {
                zero
            };
            acc = if p == 0 { bit } else { acc + acc + bit };
            rows[r][COL_SBIT] = bit;
            rows[r][COL_SACC] = acc;
        }
    }

    // --- Carril de hash: filas 0..287 ---
    // Estado inicial (ciclo 0): hoja interna = merge(account, balance).
    let mut state = [zero; STATE_WIDTH];
    state[4] = account_id;
    state[8] = c_bal;
    rows[0][..STATE_WIDTH].copy_from_slice(&state);

    for r in 0..HASH_ROWS - 1 {
        let pos = r % CYCLE_LENGTH;
        if pos < NUM_ROUNDS {
            // Ronda de Rescue con la implementación real de la librería.
            Rp64_256::apply_round(&mut state, pos);
        } else {
            // Fila de enlace: preparar el siguiente ciclo.
            let digest: Digest = [state[4], state[5], state[6], state[7]];
            state = [zero; STATE_WIDTH];
            match r {
                ROW_LEAF_LINK_1 | ROW_NULL_LINK => {
                    // digest interno a la izquierda, nonce a la derecha.
                    state[4..8].copy_from_slice(&digest);
                    state[8] = nonce;
                }
                ROW_ROOT => {
                    // raíz calculada -> arranque del nullifier.
                    state[4] = BaseElement::new(NULLIFIER_DOMAIN);
                    state[8] = account_id;
                }
                _ => {
                    // Enlace de nivel del árbol: colocar el digest según
                    // el bit del nivel destino.
                    let next_cycle = (r + 1) / CYCLE_LENGTH;
                    let level = next_cycle - 2; // ciclos 2..33 -> niveles 0..31
                    if path.is_right[level] {
                        state[4..8].copy_from_slice(&path.siblings[level]);
                        state[8..12].copy_from_slice(&digest);
                    } else {
                        state[4..8].copy_from_slice(&digest);
                        state[8..12].copy_from_slice(&path.siblings[level]);
                    }
                }
            }
        }
        rows[r + 1][..STATE_WIDTH].copy_from_slice(&state);
    }

    // Bit de dirección: constante dentro de cada ciclo del árbol.
    for level in 0..TREE_DEPTH {
        let cycle = 2 + level;
        let bit = if path.is_right[level] {
            BaseElement::ONE
        } else {
            zero
        };
        for p in 0..CYCLE_LENGTH {
            rows[cycle * CYCLE_LENGTH + p][COL_MBIT] = bit;
        }
    }

    let mut trace = TraceTable::new(TRACE_WIDTH, TRACE_LENGTH);
    trace.fill(
        |state| state.copy_from_slice(&rows[0]),
        |step, state| state.copy_from_slice(&rows[step + 1]),
    );
    trace
}

/// Inputs públicos, mismo orden conceptual que en los otros backends.
#[derive(Clone, Debug)]
pub struct CompliancePublicInputs {
    pub state_root: Digest,
    pub regulatory_limit: BaseElement,
    pub nullifier: Digest,
}

impl ToElements<BaseElement> for CompliancePublicInputs {
    fn to_elements(&self) -> Vec<BaseElement> {
        let mut out = self.state_root.to_vec();
        out.push(self.regulatory_limit);
        out.extend_from_slice(&self.nullifier);
        out
    }
}

pub struct ComplianceAir {
    context: AirContext<BaseElement>,
    state_root: Digest,
    regulatory_limit: BaseElement,
    nullifier: Digest,
}

impl Air for ComplianceAir {
    type BaseField = BaseElement;
    type PublicInputs = CompliancePublicInputs;

    fn new(trace_info: TraceInfo, pub_inputs: Self::PublicInputs, options: ProofOptions) -> Self {
        assert_eq!(TRACE_WIDTH, trace_info.width());

        let mut degrees = Vec::new();
        // 0..11: hash (sbox grado 7, modulada por selector de traza completa)
        for _ in 0..STATE_WIDTH {
            degrees.push(TransitionConstraintDegree::with_cycles(7, vec![TRACE_LENGTH]));
        }
        // 12..23: link_left (cap, digest, nonce, relleno)
        for _ in 0..12 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, vec![TRACE_LENGTH]));
        }
        // 24..27: link_merkle cap = 0
        for _ in 0..4 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, vec![TRACE_LENGTH]));
        }
        // 28..31: link_merkle colocación por bit (grado 2)
        for _ in 0..4 {
            degrees.push(TransitionConstraintDegree::with_cycles(2, vec![TRACE_LENGTH]));
        }
        // 32: link_null; 33..34: first_hash
        for _ in 0..3 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, vec![TRACE_LENGTH]));
        }
        // 35..37: bits booleanos
        for _ in 0..3 {
            degrees.push(TransitionConstraintDegree::new(2));
        }
        // 38..40: solvencia (first_s, cont_s)
        for _ in 0..3 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, vec![TRACE_LENGTH]));
        }
        // 41..44: links de solvencia
        for _ in 0..4 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, vec![TRACE_LENGTH]));
        }
        // 45..49: constancia del transporte
        for _ in 0..5 {
            degrees.push(TransitionConstraintDegree::new(1));
        }

        ComplianceAir {
            // 30 aserciones: ver get_assertions.
            context: AirContext::new(trace_info, degrees, 30, options),
            state_root: pub_inputs.state_root,
            regulatory_limit: pub_inputs.regulatory_limit,
            nullifier: pub_inputs.nullifier,
        }
    }

    fn context(&self) -> &AirContext<Self::BaseField> {
        &self.context
    }

    /// 35 columnas periódicas, todas de longitud 512 (la traza completa).
    fn get_periodic_column_values(&self) -> Vec<Vec<Self::BaseField>> {
        let zero = BaseElement::ZERO;
        let one = BaseElement::ONE;
        let mut columns = Vec::with_capacity(35);

        // 0: hash_flag — 1 en las filas de ronda (pos 0..6) del carril activo.
        let mut hash_flag = vec![zero; TRACE_LENGTH];
        for r in 0..HASH_ROWS {
            if r % CYCLE_LENGTH < NUM_ROUNDS {
                hash_flag[r] = one;
            }
        }
        columns.push(hash_flag);

        // 1..13 y 13..25: ARK1 y ARK2, alineadas con las rondas.
        for i in 0..STATE_WIDTH {
            let mut col = vec![zero; TRACE_LENGTH];
            for r in 0..HASH_ROWS {
                let pos = r % CYCLE_LENGTH;
                if pos < NUM_ROUNDS {
                    col[r] = Rp64_256::ARK1[pos][i];
                }
            }
            columns.push(col);
        }
        for i in 0..STATE_WIDTH {
            let mut col = vec![zero; TRACE_LENGTH];
            for r in 0..HASH_ROWS {
                let pos = r % CYCLE_LENGTH;
                if pos < NUM_ROUNDS {
                    col[r] = Rp64_256::ARK2[pos][i];
                }
            }
            columns.push(col);
        }

        // 25: link_left — "digest a la izquierda, nonce a la derecha"
        // (filas 7 y 279: hoja interna y nullifier interno).
        let mut link_left = vec![zero; TRACE_LENGTH];
        link_left[ROW_LEAF_LINK_1] = one;
        link_left[ROW_NULL_LINK] = one;
        columns.push(link_left);

        // 26: link_merkle — enlaces de nivel del árbol (filas 15..263, paso 8).
        let mut link_merkle = vec![zero; TRACE_LENGTH];
        for level in 0..TREE_DEPTH {
            link_merkle[(2 + level) * CYCLE_LENGTH - 1] = one;
        }
        columns.push(link_merkle);

        // 27: link_null — arranque del nullifier (fila 271).
        let mut link_null = vec![zero; TRACE_LENGTH];
        link_null[ROW_ROOT] = one;
        columns.push(link_null);

        // 28: first_hash — fila 0 (entradas privadas de la hoja).
        let mut first_hash = vec![zero; TRACE_LENGTH];
        first_hash[0] = one;
        columns.push(first_hash);

        // 29: first_s — inicio de cada segmento de solvencia.
        let mut first_s = vec![zero; TRACE_LENGTH];
        for seg in 0..4 {
            first_s[seg * SEGMENT_LENGTH] = one;
        }
        columns.push(first_s);

        // 30: cont_s — continuación de Horner dentro de cada segmento.
        let mut cont_s = vec![zero; TRACE_LENGTH];
        for r in 0..4 * SEGMENT_LENGTH {
            if r % SEGMENT_LENGTH != SEGMENT_LENGTH - 1 {
                cont_s[r] = one;
            }
        }
        columns.push(cont_s);

        // 31..34: links de solvencia (posición 62 de cada segmento).
        for seg in 0..4 {
            let mut link = vec![zero; TRACE_LENGTH];
            link[(seg + 1) * SEGMENT_LENGTH - 2] = one;
            columns.push(link);
        }

        columns
    }

    fn evaluate_transition<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        frame: &EvaluationFrame<E>,
        periodic_values: &[E],
        result: &mut [E],
    ) {
        let current = frame.current();
        let next = frame.next();

        let hash_flag = periodic_values[0];
        let ark1 = &periodic_values[1..1 + STATE_WIDTH];
        let ark2 = &periodic_values[1 + STATE_WIDTH..1 + 2 * STATE_WIDTH];
        let link_left = periodic_values[25];
        let link_merkle = periodic_values[26];
        let link_null = periodic_values[27];
        let first_hash = periodic_values[28];
        let first_s = periodic_values[29];
        let cont_s = periodic_values[30];
        let link_bal = periodic_values[31];
        let link_amt = periodic_values[32];
        let link_db = periodic_values[33];
        let link_dl = periodic_values[34];

        // ===== 0..11: rondas de Rescue ("encontrarse en el medio") =====
        let mut a = [E::ZERO; STATE_WIDTH];
        for i in 0..STATE_WIDTH {
            let mut acc = E::ZERO;
            for j in 0..STATE_WIDTH {
                acc += E::from(Rp64_256::MDS[i][j]) * apply_sbox(current[j]);
            }
            a[i] = acc + ark1[i];
        }
        let mut b = [E::ZERO; STATE_WIDTH];
        for i in 0..STATE_WIDTH {
            let mut acc = E::ZERO;
            for j in 0..STATE_WIDTH {
                acc += E::from(Rp64_256::INV_MDS[i][j]) * (next[j] - ark2[j]);
            }
            b[i] = acc;
        }
        for i in 0..STATE_WIDTH {
            result[i] = hash_flag * (apply_sbox(b[i]) - a[i]);
        }

        // ===== 12..23: link_left (hoja interna y nullifier interno) =====
        for i in 0..4 {
            result[12 + i] = link_left * next[i]; // cap = 0
        }
        for i in 0..4 {
            result[16 + i] = link_left * (next[4 + i] - current[4 + i]); // digest
        }
        result[20] = link_left * (next[8] - current[COL_NONCE]); // nonce entra
        for i in 0..3 {
            result[21 + i] = link_left * next[9 + i]; // resto a cero
        }

        // ===== 24..31: link_merkle (enlaces de nivel del árbol) =====
        for i in 0..4 {
            result[24 + i] = link_merkle * next[i]; // cap = 0
        }
        // Colocación según el bit del nivel DESTINO (leído de `next` —
        // lección aprendida en `merkle.rs` con evidencia real).
        let mbit = next[COL_MBIT];
        for i in 0..4 {
            let digest_i = current[4 + i];
            let placed =
                (E::ONE - mbit) * (next[4 + i] - digest_i) + mbit * (next[8 + i] - digest_i);
            result[28 + i] = link_merkle * placed;
        }

        // ===== 32: link_null (el account_id entra al nullifier) =====
        result[32] = link_null * (next[8] - current[COL_ACCOUNT]);

        // ===== 33..34: first_hash (entradas privadas de la hoja) =====
        result[33] = first_hash * (current[4] - current[COL_ACCOUNT]);
        result[34] = first_hash * (current[8] - current[COL_BAL]);

        // ===== 35..37: bits booleanos =====
        result[35] = current[COL_MBIT] * (current[COL_MBIT] - E::ONE);
        let sbit_cur = current[COL_SBIT];
        let sbit_next = next[COL_SBIT];
        result[36] = sbit_cur * (sbit_cur - E::ONE);
        result[37] = sbit_next * (sbit_next - E::ONE);

        // ===== 38..40: solvencia (Horner) =====
        let sacc_cur = current[COL_SACC];
        let sacc_next = next[COL_SACC];
        result[38] = first_s * sbit_cur; // MSB = 0
        result[39] = first_s * sacc_cur; // Horner arranca en 0
        result[40] = cont_s * (sacc_next - (sacc_cur + sacc_cur + sbit_next));

        // ===== 41..44: links de solvencia =====
        result[41] = link_bal * (sacc_next - current[COL_BAL]);
        result[42] = link_amt * (sacc_next - current[COL_AMT]);
        result[43] = link_db * (sacc_next - (current[COL_BAL] - current[COL_AMT]));
        result[44] = link_dl * (sacc_next - (current[COL_LIM] - current[COL_AMT]));

        // ===== 45..49: constancia del transporte =====
        result[45] = next[COL_ACCOUNT] - current[COL_ACCOUNT];
        result[46] = next[COL_BAL] - current[COL_BAL];
        result[47] = next[COL_AMT] - current[COL_AMT];
        result[48] = next[COL_LIM] - current[COL_LIM];
        result[49] = next[COL_NONCE] - current[COL_NONCE];
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        let zero = BaseElement::ZERO;
        let mut assertions = Vec::with_capacity(30);

        // Fila 0: capacidad a cero + relleno de los digests de entrada.
        // (Las posiciones con valores PRIVADOS — account en col 4,
        // balance en col 8 — se atan con first_hash, no aquí.)
        for i in 0..4 {
            assertions.push(Assertion::single(i, 0, zero));
        }
        for i in 5..8 {
            assertions.push(Assertion::single(i, 0, zero));
        }
        for i in 9..12 {
            assertions.push(Assertion::single(i, 0, zero));
        }
        // Fila 271: la raíz reconstruida es la pública.
        for i in 0..4 {
            assertions.push(Assertion::single(4 + i, ROW_ROOT, self.state_root[i]));
        }
        // Fila 272: arranque del nullifier — capacidad a cero, constante
        // de dominio ANCLADA (no decorativa), y rellenos a cero. La
        // posición privada (col 8 = account) se ata con link_null.
        for i in 0..4 {
            assertions.push(Assertion::single(i, ROW_NULL_START, zero));
        }
        assertions.push(Assertion::single(
            4,
            ROW_NULL_START,
            BaseElement::new(NULLIFIER_DOMAIN),
        ));
        for i in 5..8 {
            assertions.push(Assertion::single(i, ROW_NULL_START, zero));
        }
        for i in 9..12 {
            assertions.push(Assertion::single(i, ROW_NULL_START, zero));
        }
        // Fila 287: el nullifier calculado es el público.
        for i in 0..4 {
            assertions.push(Assertion::single(4 + i, ROW_NULLIFIER, self.nullifier[i]));
        }
        // El límite regulatorio público, anclado en el transporte.
        assertions.push(Assertion::single(COL_LIM, 0, self.regulatory_limit));

        assertions
    }
}

pub struct ComplianceProver {
    options: ProofOptions,
}

impl ComplianceProver {
    pub fn new(options: ProofOptions) -> Self {
        Self { options }
    }
}

impl Prover for ComplianceProver {
    type BaseField = BaseElement;
    type Air = ComplianceAir;
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

    fn get_pub_inputs(&self, trace: &Self::Trace) -> CompliancePublicInputs {
        CompliancePublicInputs {
            state_root: [
                trace.get(4, ROW_ROOT),
                trace.get(5, ROW_ROOT),
                trace.get(6, ROW_ROOT),
                trace.get(7, ROW_ROOT),
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
    use winterfell::{verify, AcceptableOptions, BatchingMethod, FieldExtension, Proof};

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

    struct Scenario {
        account_id: BaseElement,
        balance: u64,
        nonce: BaseElement,
        amount: u64,
        limit: u64,
        path: MerklePath,
    }

    fn valid_scenario(balance: u64, amount: u64, limit: u64) -> Scenario {
        let account_id = BaseElement::new(12345);
        let nonce = BaseElement::new(1);
        let siblings: Vec<Digest> =
            (0..TREE_DEPTH).map(|i| digest_from(i as u64 * 10)).collect();
        let is_right: Vec<bool> = (0..TREE_DEPTH).map(|i| i % 3 == 0).collect();
        Scenario {
            account_id,
            balance,
            nonce,
            amount,
            limit,
            path: MerklePath { siblings, is_right },
        }
    }

    fn expected_public_inputs(s: &Scenario) -> CompliancePublicInputs {
        let leaf = native_leaf(s.account_id, BaseElement::new(s.balance), s.nonce);
        CompliancePublicInputs {
            state_root: native_root(leaf, &s.path),
            regulatory_limit: BaseElement::new(s.limit),
            nullifier: native_nullifier(s.account_id, s.nonce),
        }
    }

    fn run_proof(s: &Scenario, declared: CompliancePublicInputs) -> Result<(), String> {
        let trace = build_trace(s.account_id, s.balance, s.nonce, s.amount, s.limit, &s.path);
        let prover = ComplianceProver::new(default_options());

        let previous_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let prove_result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| prover.prove(trace)));
        std::panic::set_hook(previous_hook);

        let proof: Proof = match prove_result {
            Err(_) => return Err("prove hizo panic (traza invalida detectada en debug)".into()),
            Ok(Err(e)) => return Err(format!("prove devolvio Err: {e:?}")),
            Ok(Ok(p)) => p,
        };

        let min_opts = AcceptableOptions::OptionSet(vec![prover.options().clone()]);
        verify::<ComplianceAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof, declared, &min_opts,
        )
        .map_err(|e| format!("la verificacion fallo: {e:?}"))
    }

    /// Estructura de la traza: hoja, raíz, nullifier y acumuladores de
    /// solvencia en sus filas exactas.
    #[test]
    fn trace_landmarks_match_native_computation() {
        let s = valid_scenario(1_000_000, 250_000, 500_000);
        let trace = build_trace(s.account_id, s.balance, s.nonce, s.amount, s.limit, &s.path);

        let leaf = native_leaf(s.account_id, BaseElement::new(s.balance), s.nonce);
        let root = native_root(leaf, &s.path);
        let nullifier = native_nullifier(s.account_id, s.nonce);

        for i in 0..4 {
            assert_eq!(trace.get(4 + i, 15), leaf[i], "hoja en fila 15, elem {i}");
            assert_eq!(trace.get(4 + i, ROW_ROOT), root[i], "raiz en fila 271, elem {i}");
            assert_eq!(
                trace.get(4 + i, ROW_NULLIFIER),
                nullifier[i],
                "nullifier en fila 287, elem {i}"
            );
        }
        assert_eq!(trace.get(COL_SACC, 63), BaseElement::new(s.balance));
        assert_eq!(trace.get(COL_SACC, 127), BaseElement::new(s.amount));
    }

    /// EL TEST CLAVE de todo el port: la transacción completamente válida
    /// (solvente, dentro del límite, cuenta real en el árbol, nullifier
    /// correcto) produce una prueba STARK verificable.
    #[test]
    fn fully_valid_transaction_verifies() {
        let s = valid_scenario(1_000_000, 250_000, 500_000);
        let result = run_proof(&s, expected_public_inputs(&s));
        assert!(result.is_ok(), "una transaccion valida deberia verificar: {result:?}");
    }

    /// SOLIDEZ: gastar más del saldo debe fallar.
    #[test]
    fn insufficient_balance_fails() {
        let s = valid_scenario(100_000, 250_000, 500_000);
        let result = run_proof(&s, expected_public_inputs(&s));
        assert!(result.is_err(), "CRITICO: gastar mas del saldo no deberia verificar");
    }

    /// SOLIDEZ: superar el límite regulatorio debe fallar.
    #[test]
    fn amount_over_regulatory_limit_fails() {
        let s = valid_scenario(1_000_000, 750_000, 500_000);
        let result = run_proof(&s, expected_public_inputs(&s));
        assert!(result.is_err(), "CRITICO: superar el limite no deberia verificar");
    }

    /// SOLIDEZ: declarar una raíz que no corresponde debe fallar.
    #[test]
    fn wrong_declared_root_fails() {
        let s = valid_scenario(1_000_000, 250_000, 500_000);
        let mut declared = expected_public_inputs(&s);
        declared.state_root = digest_from(999_999);
        let result = run_proof(&s, declared);
        assert!(result.is_err(), "CRITICO: una raiz incorrecta no deberia verificar");
    }

    /// SOLIDEZ: declarar un nullifier falsificado debe fallar.
    #[test]
    fn forged_nullifier_fails() {
        let s = valid_scenario(1_000_000, 250_000, 500_000);
        let mut declared = expected_public_inputs(&s);
        declared.nullifier = digest_from(31_337);
        let result = run_proof(&s, declared);
        assert!(result.is_err(), "CRITICO: un nullifier falsificado no deberia verificar");
    }

    /// DISCRIMINANTE: corromper una fila intermedia (en mitad de un hash
    /// del árbol) debe detectarse.
    #[test]
    fn corrupted_intermediate_row_is_detected() {
        let s = valid_scenario(1_000_000, 250_000, 500_000);
        let mut trace =
            build_trace(s.account_id, s.balance, s.nonce, s.amount, s.limit, &s.path);
        let original = trace.get(6, 100);
        trace.set(6, 100, original + BaseElement::ONE);

        let prover = ComplianceProver::new(default_options());

        let previous_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let prove_result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| prover.prove(trace)));
        std::panic::set_hook(previous_hook);

        match prove_result {
            Err(_) => { /* panic: detectado */ }
            Ok(Err(_)) => { /* Err: detectado */ }
            Ok(Ok(proof)) => {
                let min_opts = AcceptableOptions::OptionSet(vec![prover.options().clone()]);
                let verification = verify::<
                    ComplianceAir,
                    Blake3,
                    DefaultRandomCoin<Blake3>,
                    MerkleTree<Blake3>,
                >(proof, expected_public_inputs(&s), &min_opts);
                assert!(
                    verification.is_err(),
                    "CRITICO: una traza corrompida no deberia verificar"
                );
            }
        }
    }

    /// Caso frontera legítimo: amount == balance.
    #[test]
    fn boundary_amount_equals_balance_verifies() {
        let s = valid_scenario(250_000, 250_000, 500_000);
        let result = run_proof(&s, expected_public_inputs(&s));
        assert!(result.is_ok(), "amount == balance deberia verificar: {result:?}");
    }
}
