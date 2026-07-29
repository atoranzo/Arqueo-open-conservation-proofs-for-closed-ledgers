//! **Módulo de privacidad y cumplimiento en STARK**: revelación
//! selectiva para auditoría regulatoria.
//!
//! ## El hueco que cierra
//!
//! La capa demuestra cumplimiento del límite regulatorio dentro del
//! circuito, pero **un supervisor no podía auditar nada**: no había forma
//! de verificar el saldo de una cuenta concreta sin que la entidad
//! enseñara todo su estado.
//!
//! Ese es el bloqueo real para adopción institucional. Un sistema
//! perfectamente privado en el que el regulador no puede comprobar nada
//! no es adoptable, por mucha matemática que tenga detrás.
//!
//! ## Un solo circuito para tres formas de auditar
//!
//! Demuestra `inferior <= saldo <= superior`, con ambos límites
//! públicos. Eso cubre los tres casos con una sola pieza:
//!
//! | Caso | Configuración | Qué revela |
//! |---|---|---|
//! | Revelación exacta | `inferior = superior = saldo` | El saldo |
//! | Solvencia mínima | `inferior = X`, `superior = MAX` | Que supera X |
//! | **Banda** | `inferior = X`, `superior = Y` | Que está entre X e Y |
//!
//! El tercero es el más útil para un supervisor y **no existe en el
//! backend Groth16**, que solo tiene revelación exacta y mínimo. *"Mi
//! posición está entre 10 y 50 millones"* cumple el requisito regulatorio
//! sin exponer la cifra.
//!
//! ## Revelación voluntaria, no custodia de claves
//!
//! Había dos caminos: que el supervisor tenga una clave de visualización,
//! o que el titular produzca la prueba dirigida a quien se la pida.
//!
//! **Se implementa el segundo**, por riesgo sistémico: una clave
//! custodiada es un punto único de fallo. Quien la robe —un atacante, un
//! empleado, un estado hostil— ve la actividad de todo el sistema,
//! retroactivamente y sin dejar rastro. Con revelación voluntaria **no
//! hay nada que robar**: cada revelación es un acto deliberado y puntual.
//!
//! **Contrapartida honesta**: el supervisor depende de la cooperación del
//! titular. Si una entidad se niega, la coerción viene de fuera
//! (requerimiento legal, sanción), igual que hoy con el secreto bancario.
//! Esto no sustituye a la autoridad legal; le da una herramienta para
//! verificar lo que se le entrega.
//!
//! ## Estructura de la traza (24 columnas × 512 filas)
//!
//! | Ciclos | Filas | Fase |
//! |---|---|---|
//! | 0-1 | 0..15 | Hoja de la cuenta |
//! | 2-33 | 16..271 | Subida del árbol hasta la raíz auditada |
//! | 34 | 272..279 | Derivación de `pk` desde `sk` (TITULARIDAD) |
//!
//! **Un solo carril**, no dos: aquí no hay transición de estado, solo se
//! comprueba una posición del árbol. Por eso es mucho más barato que una
//! liquidación.

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

use crate::circuit_settlement::{derive_public_id, SPEND_KEY_DOMAIN};
use crate::merkle::{Digest, MerklePath, TREE_DEPTH};
use crate::rescue_hash::{apply_sbox, NUM_ROUNDS, STATE_WIDTH};

pub const CYCLE_LENGTH: usize = 8;
pub const TRACE_LENGTH: usize = 512;
pub const SEGMENT_LENGTH: usize = 64;
/// Segmentos: saldo, saldo − inferior, superior − saldo.
pub const NUM_SEGMENTS: usize = 3;
/// Valor máximo representable en 63 bits: el techo del campo Goldilocks
/// para comprobaciones de rango. Ver la nota sobre el techo de 63 bits en
/// `FIVE_BACKENDS.md`.
pub const MAX_VALUE: u64 = (1u64 << 62) - 1;

// ===== Columnas =====
const COL_BIT: usize = 12;
const COL_KEY: usize = 13;
const COL_ID: usize = 14; // 14..18
const COL_BAL: usize = 18;
const COL_NONCE: usize = 19;
const COL_LOWER: usize = 20;
const COL_UPPER: usize = 21;
const COL_SBIT: usize = 22;
const COL_SACC: usize = 23;
pub const TRACE_WIDTH: usize = 24;

// ===== Filas de eventos =====
const ROW_LEAF_LINK: usize = 7;
const ROW_LEAF_DONE: usize = 15;
const ROW_ROOT: usize = 271;
const ROW_PK_START: usize = 272;
const ROW_PK_DONE: usize = 279;

// ===== Índices de restricción =====
const C_HASH: usize = 0; // 12
const C_TREE_CAP: usize = C_HASH + STATE_WIDTH; // 4
const C_PLACE: usize = C_TREE_CAP + 4; // 4
const C_BIT_BOOL: usize = C_PLACE + 4; // 1
const C_LEAF_CAP: usize = C_BIT_BOOL + 1; // 4
const C_LEAF_DIG: usize = C_LEAF_CAP + 4; // 4
const C_NONCE: usize = C_LEAF_DIG + 4; // 1
const C_INPUT: usize = C_NONCE + 1; // 5: identidad (4) + saldo
const C_PK_INPUT: usize = C_INPUT + 5; // 1
/// **TITULARIDAD**: la pk derivada coincide con la identidad auditada.
const C_PK_CHECK: usize = C_PK_INPUT + 1; // 4
const C_TRANSPORT: usize = C_PK_CHECK + 4; // 5
const C_ID_CONST: usize = C_TRANSPORT + 5; // 4
const C_SBIT_BOOL: usize = C_ID_CONST + 4; // 2
const C_FIRST_S: usize = C_SBIT_BOOL + 2; // 2
const C_HORNER: usize = C_FIRST_S + 2; // 1
const C_SEG_LINK: usize = C_HORNER + 1; // NUM_SEGMENTS
const NUM_CONSTRAINTS: usize = C_SEG_LINK + NUM_SEGMENTS;

// ===== Columnas periódicas =====
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

type Blake3 = Blake3_256<BaseElement>;

fn value_to_bits_be(value: u64) -> Vec<bool> {
    (0..SEGMENT_LENGTH)
        .map(|p| (value >> (SEGMENT_LENGTH - 1 - p)) & 1 == 1)
        .collect()
}

/// Testigos de la auditoría. Todos privados salvo lo que se decida
/// revelar mediante los límites.
#[derive(Clone, Debug)]
pub struct AuditWitness {
    pub spend_key: BaseElement,
    pub balance: u64,
    pub nonce: BaseElement,
    pub path: MerklePath,
}

/// Construye la traza.
///
/// `claimed_id` permite declarar una identidad distinta de la derivada de
/// la clave, para el test que comprueba que un tercero no puede revelar
/// por otro.
pub fn build_trace_with_id(
    witness: &AuditWitness,
    lower: u64,
    upper: u64,
    claimed_id: Digest,
) -> TraceTable<BaseElement> {
    let zero = BaseElement::ZERO;
    let c_bal = BaseElement::new(witness.balance);
    let c_lower = BaseElement::new(lower);
    let c_upper = BaseElement::new(upper);

    let mut rows: Vec<Vec<BaseElement>> = vec![vec![zero; TRACE_WIDTH]; TRACE_LENGTH];

    for row in rows.iter_mut() {
        row[COL_KEY] = witness.spend_key;
        for i in 0..4 {
            row[COL_ID + i] = claimed_id[i];
        }
        row[COL_BAL] = c_bal;
        row[COL_NONCE] = witness.nonce;
        row[COL_LOWER] = c_lower;
        row[COL_UPPER] = c_upper;
    }

    // Rangos: saldo, saldo − inferior, superior − saldo.
    //
    // Si el saldo estuviera fuera de la banda, alguna resta daria la
    // vuelta en el campo y su bit mas significativo seria uno, lo que la
    // restriccion `first_s` rechaza.
    let segment_values = [
        c_bal.as_int(),
        (c_bal - c_lower).as_int(),
        (c_upper - c_bal).as_int(),
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

    let place = |state: &mut [BaseElement; STATE_WIDTH], digest: &Digest, level: usize| {
        if witness.path.is_right[level] {
            state[4..8].copy_from_slice(&witness.path.siblings[level]);
            state[8..12].copy_from_slice(digest);
        } else {
            state[4..8].copy_from_slice(digest);
            state[8..12].copy_from_slice(&witness.path.siblings[level]);
        }
    };

    let mut state = [zero; STATE_WIDTH];
    state[4..8].copy_from_slice(&claimed_id);
    state[8] = c_bal;
    rows[0][..STATE_WIDTH].copy_from_slice(&state);

    for r in 0..ROW_PK_DONE {
        let pos = r % CYCLE_LENGTH;
        if pos < NUM_ROUNDS {
            Rp64_256::apply_round(&mut state, pos);
        } else {
            let digest: Digest = [state[4], state[5], state[6], state[7]];
            state = [zero; STATE_WIDTH];
            match r {
                ROW_LEAF_LINK => {
                    state[4..8].copy_from_slice(&digest);
                    state[8] = witness.nonce;
                }
                ROW_LEAF_DONE => place(&mut state, &digest, 0),
                ROW_ROOT => {
                    // Derivacion de pk: TITULARIDAD.
                    state[4] = BaseElement::new(SPEND_KEY_DOMAIN);
                    state[8] = witness.spend_key;
                }
                _ => {
                    let next_cycle = (r + 1) / CYCLE_LENGTH;
                    if (2..34).contains(&next_cycle) {
                        place(&mut state, &digest, next_cycle - 2);
                    }
                }
            }
        }
        rows[r + 1][..STATE_WIDTH].copy_from_slice(&state);
    }

    for level in 0..TREE_DEPTH {
        let bit = if witness.path.is_right[level] {
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

pub fn build_trace(witness: &AuditWitness, lower: u64, upper: u64) -> TraceTable<BaseElement> {
    let id = derive_public_id(witness.spend_key);
    build_trace_with_id(witness, lower, upper, id)
}

#[derive(Clone, Debug)]
pub struct AuditPublicInputs {
    /// Raíz del estado auditado.
    pub root: Digest,
    /// Identidad de la cuenta. El supervisor sabe A QUIÉN audita.
    pub public_id: Digest,
    pub lower: BaseElement,
    pub upper: BaseElement,
}

impl ToElements<BaseElement> for AuditPublicInputs {
    fn to_elements(&self) -> Vec<BaseElement> {
        let mut out = self.root.to_vec();
        out.extend_from_slice(&self.public_id);
        out.push(self.lower);
        out.push(self.upper);
        out
    }
}

pub struct AuditAir {
    context: AirContext<BaseElement>,
    pub_inputs: AuditPublicInputs,
}

impl Air for AuditAir {
    type BaseField = BaseElement;
    type PublicInputs = AuditPublicInputs;

    fn new(trace_info: TraceInfo, pub_inputs: Self::PublicInputs, options: ProofOptions) -> Self {
        assert_eq!(TRACE_WIDTH, trace_info.width());
        let full = vec![TRACE_LENGTH];

        let mut degrees = Vec::with_capacity(NUM_CONSTRAINTS);
        for _ in 0..STATE_WIDTH {
            degrees.push(TransitionConstraintDegree::with_cycles(7, full.clone()));
        }
        for _ in 0..4 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }
        for _ in 0..4 {
            degrees.push(TransitionConstraintDegree::with_cycles(2, full.clone()));
        }
        degrees.push(TransitionConstraintDegree::new(2)); // bit booleano
        // leaf cap (4), leaf dig (4), nonce (1), input (5), pk input (1),
        // pk check (4) = 19, grado 1 con ciclo.
        for _ in 0..19 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }
        // Transporte (5) + identidad (4): grado 1 sin ciclo.
        for _ in 0..9 {
            degrees.push(TransitionConstraintDegree::new(1));
        }
        for _ in 0..2 {
            degrees.push(TransitionConstraintDegree::new(2));
        }
        for _ in 0..(3 + NUM_SEGMENTS) {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }

        assert_eq!(degrees.len(), NUM_CONSTRAINTS, "cuenta de grados");

        AuditAir {
            context: AirContext::new(trace_info, degrees, 20, options),
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
        for r in 0..=ROW_PK_DONE {
            if r % CYCLE_LENGTH < NUM_ROUNDS {
                hash_flag[r] = one;
            }
        }
        columns.push(hash_flag);

        for ark in [true, false] {
            for i in 0..STATE_WIDTH {
                let mut col = vec![zero; TRACE_LENGTH];
                for r in 0..=ROW_PK_DONE {
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

        // Rondas de Rescue.
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
            result[C_HASH + i] = hash_flag * (apply_sbox(b[i]) - a[i]);
        }

        let bit = next[COL_BIT];
        let tree_link = link_merkle + link_place;

        for i in 0..4 {
            result[C_TREE_CAP + i] = tree_link * next[i];
            let d = current[4 + i];
            result[C_PLACE + i] =
                tree_link * ((E::ONE - bit) * (next[4 + i] - d) + bit * (next[8 + i] - d));
        }

        result[C_BIT_BOOL] = current[COL_BIT] * (current[COL_BIT] - E::ONE);

        for i in 0..4 {
            result[C_LEAF_CAP + i] = link_leaf * next[i];
            result[C_LEAF_DIG + i] = link_leaf * (next[4 + i] - current[4 + i]);
        }
        result[C_NONCE] = link_leaf * (next[8] - current[COL_NONCE]);

        // Entradas de la hoja: identidad completa + saldo.
        for i in 0..4 {
            result[C_INPUT + i] = first_row * (current[4 + i] - current[COL_ID + i]);
        }
        result[C_INPUT + 4] = first_row * (current[8] - current[COL_BAL]);

        // La clave entra en la derivación de pk.
        result[C_PK_INPUT] = sel_root * (next[8] - current[COL_KEY]);

        // ===== TITULARIDAD =====
        // La pk derivada de la clave coincide con la identidad auditada.
        // Sin la clave del titular no se puede producir la revelación:
        // impide que un tercero fabrique revelaciones sobre cuentas
        // ajenas.
        for i in 0..4 {
            result[C_PK_CHECK + i] = sel_pk_done * (current[4 + i] - current[COL_ID + i]);
        }

        let transport = [COL_KEY, COL_BAL, COL_NONCE, COL_LOWER, COL_UPPER];
        for (k, col) in transport.iter().enumerate() {
            result[C_TRANSPORT + k] = next[*col] - current[*col];
        }
        for i in 0..4 {
            result[C_ID_CONST + i] = next[COL_ID + i] - current[COL_ID + i];
        }

        // ===== LA BANDA: inferior <= saldo <= superior =====
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
            current[COL_BAL] - current[COL_LOWER],
            current[COL_UPPER] - current[COL_BAL],
        ];
        for seg in 0..NUM_SEGMENTS {
            result[C_SEG_LINK + seg] = periodic[P_SEG_LINK + seg] * (sacc_next - expected[seg]);
        }
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        let zero = BaseElement::ZERO;
        let mut a = Vec::with_capacity(20);

        for i in 0..4 {
            a.push(Assertion::single(i, 0, zero));
        }
        for i in 9..12 {
            a.push(Assertion::single(i, 0, zero));
        }
        for i in 0..4 {
            a.push(Assertion::single(4 + i, ROW_ROOT, self.pub_inputs.root[i]));
        }
        a.push(Assertion::single(
            4,
            ROW_PK_START,
            BaseElement::new(SPEND_KEY_DOMAIN),
        ));
        for i in 5..8 {
            a.push(Assertion::single(i, ROW_PK_START, zero));
        }
        for i in 9..12 {
            a.push(Assertion::single(i, ROW_PK_START, zero));
        }
        a.push(Assertion::single(COL_LOWER, 0, self.pub_inputs.lower));
        a.push(Assertion::single(COL_UPPER, 0, self.pub_inputs.upper));

        a
    }
}

pub struct AuditProver {
    options: ProofOptions,
}

impl AuditProver {
    pub fn new(options: ProofOptions) -> Self {
        Self { options }
    }
}

impl Prover for AuditProver {
    type BaseField = BaseElement;
    type Air = AuditAir;
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

    fn get_pub_inputs(&self, trace: &Self::Trace) -> AuditPublicInputs {
        AuditPublicInputs {
            root: [
                trace.get(4, ROW_ROOT),
                trace.get(5, ROW_ROOT),
                trace.get(6, ROW_ROOT),
                trace.get(7, ROW_ROOT),
            ],
            public_id: [
                trace.get(COL_ID, 0),
                trace.get(COL_ID + 1, 0),
                trace.get(COL_ID + 2, 0),
                trace.get(COL_ID + 3, 0),
            ],
            lower: trace.get(COL_LOWER, 0),
            upper: trace.get(COL_UPPER, 0),
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
    use crate::circuit_settlement::{native_climb, native_leaf};
    use super::*;
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

    fn scenario(balance: u64) -> (AuditWitness, Digest, Digest) {
        let mut empty = vec![[BaseElement::ZERO; 4]];
        for k in 1..=TREE_DEPTH {
            let prev = empty[k - 1];
            empty.push(native_merge(prev, prev));
        }
        let key = BaseElement::new(SK);
        let id = derive_public_id(key);
        let nonce = BaseElement::ZERO;

        let mut siblings = Vec::with_capacity(TREE_DEPTH);
        let mut is_right = Vec::with_capacity(TREE_DEPTH);
        for level in 0..TREE_DEPTH {
            siblings.push(empty[level]);
            // Direcciones MIXTAS: con todas iguales la traza degenera y
            // las restricciones de grado 2 colapsan a grado 1.
            is_right.push(level % 3 == 0);
        }
        let path = MerklePath { siblings, is_right };
        let root = native_climb(native_leaf(id, BaseElement::new(balance), nonce), &path);

        (
            AuditWitness {
                spend_key: key,
                balance,
                nonce,
                path,
            },
            root,
            id,
        )
    }

    fn run(w: &AuditWitness, lower: u64, upper: u64, declared: AuditPublicInputs) -> bool {
        run_with_id(w, lower, upper, declared.public_id, declared)
    }

    fn run_with_id(
        w: &AuditWitness,
        lower: u64,
        upper: u64,
        claimed_id: Digest,
        declared: AuditPublicInputs,
    ) -> bool {
        let trace = build_trace_with_id(w, lower, upper, claimed_id);
        let prover = AuditProver::new(default_options());

        let hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| prover.prove(trace)));
        std::panic::set_hook(hook);

        match r {
            Err(_) | Ok(Err(_)) => false,
            Ok(Ok(proof)) => {
                let min_opts = AcceptableOptions::OptionSet(vec![default_options()]);
                verify::<AuditAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
                    proof, declared, &min_opts,
                )
                .is_ok()
            }
        }
    }

    fn pi(root: Digest, id: Digest, lower: u64, upper: u64) -> AuditPublicInputs {
        AuditPublicInputs {
            root,
            public_id: id,
            lower: BaseElement::new(lower),
            upper: BaseElement::new(upper),
        }
    }

    /// **REVELACIÓN EXACTA**: inferior = superior = saldo.
    #[test]
    fn exact_disclosure_verifies() {
        let (w, root, id) = scenario(1_000_000);
        let trace = build_trace(&w, 1_000_000, 1_000_000);
        let prover = AuditProver::new(default_options());
        let proof = prover.prove(trace).expect("prove");
        let min_opts = AcceptableOptions::OptionSet(vec![default_options()]);
        let v = verify::<AuditAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof,
            pi(root, id, 1_000_000, 1_000_000),
            &min_opts,
        );
        assert!(v.is_ok(), "{v:?}");
    }

    /// **SOLVENCIA MÍNIMA**: "supero X", sin decir cuánto.
    #[test]
    fn minimum_solvency_verifies() {
        let (w, root, id) = scenario(1_000_000);
        assert!(run(&w, 500_000, MAX_VALUE, pi(root, id, 500_000, MAX_VALUE)));
    }

    /// **BANDA**: "estoy entre X e Y", que es lo más útil para un
    /// supervisor y no existe en el backend Groth16.
    #[test]
    fn band_disclosure_verifies() {
        let (w, root, id) = scenario(1_000_000);
        assert!(run(&w, 900_000, 1_100_000, pi(root, id, 900_000, 1_100_000)));
    }

    /// **EL TEST QUE HACE ÚTIL LA AUDITORÍA.**
    ///
    /// Declarar un saldo distinto del real debe fallar. Sin esto, la
    /// entidad diría al supervisor lo que le conviniera.
    #[test]
    fn lying_about_the_balance_is_rejected() {
        let (w, root, id) = scenario(1_000_000);
        assert!(
            !run(&w, 9_999_999, 9_999_999, pi(root, id, 9_999_999, 9_999_999)),
            "CRITICO: declarar un saldo falso al supervisor debe rechazarse"
        );
    }

    /// **NO SE PUEDE FINGIR SOLVENCIA.**
    #[test]
    fn balance_below_the_floor_is_rejected() {
        let (w, root, id) = scenario(100_000);
        assert!(
            !run(&w, 500_000, MAX_VALUE, pi(root, id, 500_000, MAX_VALUE)),
            "CRITICO: no debe poder demostrarse un minimo de reservas que no se cumple"
        );
    }

    /// Tampoco se puede fingir estar por debajo de un techo.
    #[test]
    fn balance_above_the_ceiling_is_rejected() {
        let (w, root, id) = scenario(1_000_000);
        assert!(!run(&w, 0, 500_000, pi(root, id, 0, 500_000)));
    }

    /// **NADIE PUEDE REVELAR POR OTRO.**
    ///
    /// Un tercero conoce la identidad, el saldo, el nonce y el camino de
    /// una cuenta ajena, y construye la traza **con la identidad de la
    /// víctima** para que la raíz cuadre. Solo `C_PK_CHECK` puede
    /// detectar que su clave no corresponde.
    #[test]
    fn third_party_cannot_disclose_someone_elses_balance() {
        let (victim, root, victim_id) = scenario(1_000_000);
        let attacker = AuditWitness {
            spend_key: BaseElement::new(0x1337),
            balance: victim.balance,
            nonce: victim.nonce,
            path: victim.path.clone(),
        };
        assert!(
            !run_with_id(
                &attacker,
                1_000_000,
                1_000_000,
                victim_id,
                pi(root, victim_id, 1_000_000, 1_000_000)
            ),
            "CRITICO: solo el titular puede revelar su saldo"
        );
    }

    /// Revelar contra una raíz distinta a la del estado auditado debe
    /// fallar: impide revelar sobre un estado antiguo favorable.
    #[test]
    fn disclosure_against_wrong_root_is_rejected() {
        let (w, _, id) = scenario(1_000_000);
        let fake_root: Digest = [BaseElement::new(999); 4];
        assert!(!run(
            &w,
            1_000_000,
            1_000_000,
            pi(fake_root, id, 1_000_000, 1_000_000)
        ));
    }

    /// **SEPARA "LA TRAZA ESTÁ MAL" DE "LAS RESTRICCIONES ESTÁN MAL".**
    ///
    /// Este circuito **no tenía ningún test de puntos de referencia**, pese
    /// a estar en producción. Se detectó al inventariar cuáles comparaban
    /// sus entradas públicas con lo que la traza produce (ver
    /// `AUDITORIA.md` §11).
    ///
    /// Compara la **estructura entera**: en `circuit_send` la versión
    /// parcial dejó pasar un campo heredado y **costó ocho rondas de
    /// diagnóstico**.
    #[test]
    fn trace_landmarks_match_native() {
        let (w, root, id) = scenario(1_000_000);
        let trace = build_trace(&w, 700_000, 800_000);

        for i in 0..4 {
            assert_eq!(trace.get(4 + i, ROW_ROOT), root[i], "raiz, elemento {i}");
            assert_eq!(
                trace.get(4 + i, ROW_PK_DONE),
                id[i],
                "identidad publica, elemento {i}"
            );
        }

        let derivadas = AuditProver::new(default_options()).get_pub_inputs(&trace);
        assert_eq!(
            derivadas.to_elements(),
            pi(root, id, 700_000, 800_000).to_elements(),
            "las entradas DERIVADAS de la traza deben coincidir con las \
             DECLARADAS en todos sus campos"
        );
    }
}
