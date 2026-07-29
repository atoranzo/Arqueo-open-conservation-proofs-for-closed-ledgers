//! **Circuito de emisión con autoridad de umbral.**
//!
//! Cierra la última vía por la que se podía crear dinero, y elimina el
//! punto único de fallo del emisor.
//!
//! ## Qué cambió respecto a la versión de clave única
//!
//! Antes bastaba **una** clave para emitir: robarla permitía crear dinero
//! hasta el tope del sistema. Ahora la emisión exige **dos custodios
//! distintos** de un conjunto comprometido en una raíz pública.
//!
//! ## ⚠️ Qué garantía da esto exactamente, y cuál no
//!
//! **Da**: robar una clave ya no basta. Se necesitan dos.
//!
//! **No da**: que dos personas independientes hayan autorizado *esta*
//! emisión. En una arquitectura de nodo único, quien genera la prueba
//! necesita las dos claves a la vez, y si las tiene puede emitir lo que
//! quiera. No hay firma que ate a cada custodio a esta operación
//! concreta.
//!
//! La autorización verdaderamente separada —cada custodio firmando desde
//! su propio HSM, sin que las claves coincidan nunca en la misma
//! máquina— requiere delegación de la prueba, que a su vez requiere la
//! arquitectura descentralizada que este proyecto no tiene.
//!
//! **La garantía es "dos claves comprometidas en vez de una", no "dos
//! voluntades independientes".** Es una mejora real del principio de
//! eliminar puntos únicos de fallo, y conviene no leerla como más de lo
//! que es.
//!
//! ## El riesgo que hay que cerrar: el mismo custodio contando dos veces
//!
//! Un 2-de-N en el que un custodio pueda duplicarse es un **1-de-N
//! disfrazado**. Se cierra con índices estrictamente crecientes, atados
//! a los caminos de Merkle mediante un acumulador. Ver
//! `circuit_threshold`, donde la pieza está verificada de forma aislada.
//!
//! ## Estructura de la traza (45 columnas × 512 filas)
//!
//! | Ciclos | Filas | Fase |
//! |---|---|---|
//! | 0-1 | 0..15 | Hojas de la cuenta (A = antigua, B = nueva) |
//! | 2-33 | 16..271 | Subida dual del árbol de cuentas |
//! | 34 | 272..279 | Derivación de las dos identidades de custodio |
//! | 35-38 | 280..311 | Subida al conjunto de custodios |
//!
//! Los dos carriles se **reutilizan**: durante la fase de cuenta llevan
//! saldo antiguo y nuevo por el MISMO camino; durante la de custodios
//! llevan dos custodios por caminos DISTINTOS. De ahí que haya dos
//! columnas de bit de dirección.

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

use crate::circuit_threshold::{CustodianPath, CUSTODIAN_DEPTH, CUSTODIAN_DOMAIN};
use crate::merkle::{Digest, MerklePath, TREE_DEPTH};
use crate::rescue_hash::{apply_sbox, NUM_ROUNDS, STATE_WIDTH};

pub const CYCLE_LENGTH: usize = 8;
pub const TRACE_LENGTH: usize = 512;
pub const SEGMENT_LENGTH: usize = 64;
/// Segmentos: saldo, importe, saldo nuevo, suministro nuevo,
/// `tope − suministro nuevo`, índice A, índice B, y `B − A − 1`.
pub const NUM_SEGMENTS: usize = 8;

// ===== Columnas =====
const LANE_B: usize = STATE_WIDTH; // 12
/// Bit de dirección del carril A. Durante la fase de cuenta lo usan
/// ambos carriles (mismo camino); durante la de custodios, solo A.
const COL_BIT_A: usize = 24;
/// Bit del carril B, usado solo en la fase de custodios.
const COL_BIT_B: usize = 25;
const COL_KEY_A: usize = 26;
const COL_KEY_B: usize = 27;
const COL_IDX_A: usize = 28;
const COL_IDX_B: usize = 29;
const COL_ACC_A: usize = 30;
const COL_ACC_B: usize = 31;
const COL_ACC_ID: usize = 32; // 32..36
const COL_BAL: usize = 36;
const COL_BAL_NEW: usize = 37;
const COL_NONCE: usize = 38;
const COL_AMT: usize = 39;
const COL_SUPPLY_OLD: usize = 40;
const COL_SUPPLY_NEW: usize = 41;
const COL_MAX_SUPPLY: usize = 42;
const COL_SBIT: usize = 43;
const COL_SACC: usize = 44;
pub const TRACE_WIDTH: usize = 45;

// ===== Filas =====
const ROW_LEAF_LINK: usize = 7;
const ROW_LEAF_DONE: usize = 15;
/// Raíz del árbol de cuentas. Su enlace arranca la fase de custodios.
const ROW_ACCT_ROOT: usize = 271;
const ROW_CUST_START: usize = 272;
/// Última fila activa: raíz del conjunto de custodios.
const ROW_CUST_ROOT: usize = 311;

// ===== Restricciones =====
const C_HASH_A: usize = 0;
const C_HASH_B: usize = C_HASH_A + STATE_WIDTH;
const C_CAP_A: usize = C_HASH_B + STATE_WIDTH; // 4
const C_CAP_B: usize = C_CAP_A + 4;
/// Colocación en el árbol de CUENTAS: ambos carriles con el bit A.
const C_PLACE_A: usize = C_CAP_B + 4; // 4
const C_PLACE_B: usize = C_PLACE_A + 4;
/// Hermano compartido: solo en el árbol de cuentas.
const C_SIBLING: usize = C_PLACE_B + 4; // 4
/// Colocación en el árbol de CUSTODIOS: cada carril con su bit.
const C_CPLACE_A: usize = C_SIBLING + 4; // 4
const C_CPLACE_B: usize = C_CPLACE_A + 4;
const C_BIT_BOOL: usize = C_CPLACE_B + 4; // 2
const C_LEAF_CAP_A: usize = C_BIT_BOOL + 2; // 4
const C_LEAF_CAP_B: usize = C_LEAF_CAP_A + 4;
const C_LEAF_DIG_A: usize = C_LEAF_CAP_B + 4; // 4
const C_LEAF_DIG_B: usize = C_LEAF_DIG_A + 4;
const C_NONCE: usize = C_LEAF_DIG_B + 4; // 2
const C_INPUT: usize = C_NONCE + 2; // 10
/// Las claves de custodio entran en su derivación de identidad.
const C_CUST_INPUT: usize = C_INPUT + 10; // 2
/// El acumulador ata el índice al camino demostrado.
const C_ACC: usize = C_CUST_INPUT + 2; // 2
const C_ACC_FINAL: usize = C_ACC + 2; // 2
const C_BALANCE: usize = C_ACC_FINAL + 2; // 1
const C_SUPPLY: usize = C_BALANCE + 1; // 1
const C_TRANSPORT: usize = C_SUPPLY + 1; // 11
const C_ID_CONST: usize = C_TRANSPORT + 11; // 4
const C_SBIT_BOOL: usize = C_ID_CONST + 4; // 2
const C_FIRST_S: usize = C_SBIT_BOOL + 2; // 2
const C_HORNER: usize = C_FIRST_S + 2; // 1
const C_SEG_LINK: usize = C_HORNER + 1; // NUM_SEGMENTS
const NUM_CONSTRAINTS: usize = C_SEG_LINK + NUM_SEGMENTS;

// ===== Periódicas =====
const P_HASH_FLAG: usize = 0;
const P_ARK1: usize = 1;
const P_ARK2: usize = P_ARK1 + STATE_WIDTH;
/// Enlaces del árbol de cuentas (niveles + colocación de la hoja).
const P_ACCT_LINK: usize = P_ARK2 + STATE_WIDTH;
const P_LINK_LEAF: usize = P_ACCT_LINK + 1;
/// Enlaces del árbol de custodios (entrada + niveles).
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

/// Autorización de dos custodios.
#[derive(Clone, Debug)]
pub struct ThresholdAuth {
    pub key_a: BaseElement,
    pub index_a: u64,
    pub path_a: CustodianPath,
    pub key_b: BaseElement,
    pub index_b: u64,
    pub path_b: CustodianPath,
}

/// Construye la traza de una emisión autorizada por umbral.
///
/// `supply_delta` permite variar el suministro en una cantidad distinta
/// de la emitida, para los tests de emisión encubierta.
#[allow(clippy::too_many_arguments)]
pub fn build_trace(
    auth: &ThresholdAuth,
    account_id: Digest,
    balance: u64,
    nonce: BaseElement,
    path: &MerklePath,
    amount: u64,
    supply_old: u64,
    supply_delta: u64,
    max_supply: u64,
) -> TraceTable<BaseElement> {
    let zero = BaseElement::ZERO;
    let c_bal = BaseElement::new(balance);
    let c_amt = BaseElement::new(amount);
    let c_bal_new = c_bal + c_amt;
    let c_supply_old = BaseElement::new(supply_old);
    let c_supply_new = c_supply_old + BaseElement::new(supply_delta);
    let c_max = BaseElement::new(max_supply);

    let mut rows: Vec<Vec<BaseElement>> = vec![vec![zero; TRACE_WIDTH]; TRACE_LENGTH];

    for row in rows.iter_mut() {
        row[COL_KEY_A] = auth.key_a;
        row[COL_KEY_B] = auth.key_b;
        row[COL_IDX_A] = BaseElement::new(auth.index_a);
        row[COL_IDX_B] = BaseElement::new(auth.index_b);
        for i in 0..4 {
            row[COL_ACC_ID + i] = account_id[i];
        }
        row[COL_BAL] = c_bal;
        row[COL_BAL_NEW] = c_bal_new;
        row[COL_NONCE] = nonce;
        row[COL_AMT] = c_amt;
        row[COL_SUPPLY_OLD] = c_supply_old;
        row[COL_SUPPLY_NEW] = c_supply_new;
        row[COL_MAX_SUPPLY] = c_max;
    }

    // Rangos. El quinto demuestra que no se supera el tope; los tres
    // ultimos, que los custodios son distintos y estan en orden.
    let diff = BaseElement::new(auth.index_b) - BaseElement::new(auth.index_a) - BaseElement::ONE;
    let segment_values = [
        c_bal.as_int(),
        c_amt.as_int(),
        c_bal_new.as_int(),
        c_supply_new.as_int(),
        (c_max - c_supply_new).as_int(),
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
    state_a[4..8].copy_from_slice(&account_id);
    state_a[8] = c_bal;
    state_b[4..8].copy_from_slice(&account_id);
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
                    // El nonce NO cambia: emitir no consume el derecho de
                    // gasto del titular.
                    state_a[4..8].copy_from_slice(&digest_a);
                    state_a[8] = nonce;
                    state_b[4..8].copy_from_slice(&digest_b);
                    state_b[8] = nonce;
                }
                ROW_LEAF_DONE => {
                    place_acct(&mut state_a, &digest_a, 0);
                    place_acct(&mut state_b, &digest_b, 0);
                }
                ROW_ACCT_ROOT => {
                    // Arranca la derivacion de identidades de custodio.
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

    // Bits del arbol de cuentas: ambos carriles el mismo camino.
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
    // Bits del arbol de custodios: caminos DISTINTOS por carril.
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
pub struct MintPublicInputs {
    pub root_old: Digest,
    pub root_new: Digest,
    /// **Raíz del conjunto de custodios autorizados.** Sustituye a la
    /// identidad única del emisor.
    pub custodian_set_root: Digest,
    pub amount: BaseElement,
    pub supply_old: BaseElement,
    pub supply_new: BaseElement,
    pub max_supply: BaseElement,
}

impl ToElements<BaseElement> for MintPublicInputs {
    fn to_elements(&self) -> Vec<BaseElement> {
        let mut out = self.root_old.to_vec();
        out.extend_from_slice(&self.root_new);
        out.extend_from_slice(&self.custodian_set_root);
        out.push(self.amount);
        out.push(self.supply_old);
        out.push(self.supply_new);
        out.push(self.max_supply);
        out
    }
}

pub struct MintAir {
    context: AirContext<BaseElement>,
    pub_inputs: MintPublicInputs,
}

impl Air for MintAir {
    type BaseField = BaseElement;
    type PublicInputs = MintPublicInputs;

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
        // Colocacion de cuentas (8) + hermano (4) + colocacion de
        // custodios (8): grado 2.
        for _ in 0..20 {
            degrees.push(TransitionConstraintDegree::with_cycles(2, full.clone()));
        }
        // Bits booleanos (2): grado 2 sin ciclo.
        for _ in 0..2 {
            degrees.push(TransitionConstraintDegree::new(2));
        }
        // Enlaces de hoja (16), nonce (2), entradas (10), claves de
        // custodio (2): grado 1 con ciclo.
        for _ in 0..30 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }
        // Acumulador (2): **DOS columnas periódicas** (enlace y potencia).
        for _ in 0..2 {
            degrees.push(TransitionConstraintDegree::with_cycles(
                1,
                vec![TRACE_LENGTH, TRACE_LENGTH],
            ));
        }
        // Acumulador final (2): un selector.
        for _ in 0..2 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }
        // Saldo (1), suministro (1), transporte (11), identidad (4):
        // grado 1 sin ciclo.
        for _ in 0..17 {
            degrees.push(TransitionConstraintDegree::new(1));
        }
        // Bits de rango (2): grado 2 sin ciclo.
        for _ in 0..2 {
            degrees.push(TransitionConstraintDegree::new(2));
        }
        // `first_s` y `cont_s` (3 restricciones): con 8 segmentos de 64
        // filas, las comprobaciones de rango llenan EXACTAMENTE las 512
        // filas de la traza. Eso las convierte en columnas genuinamente
        // periódicas de periodo 64, que interpolan a grado 63×8 = 504, no
        // 511.
        //
        // En los demás circuitos los segmentos no llenan la traza —quedan
        // filas a cero que rompen la periodicidad— y ahí TRACE_LENGTH sí
        // es el ciclo correcto.
        for _ in 0..3 {
            degrees.push(TransitionConstraintDegree::with_cycles(
                1,
                vec![SEGMENT_LENGTH],
            ));
        }
        // Enlaces de segmento: cada uno tiene un único uno en una
        // posición distinta, así que NO son periódicos.
        for _ in 0..NUM_SEGMENTS {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }

        assert_eq!(degrees.len(), NUM_CONSTRAINTS, "cuenta de grados");

        MintAir {
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

        // Enlaces del arbol de cuentas: colocacion de la hoja + niveles.
        let mut acct_link = vec![zero; TRACE_LENGTH];
        acct_link[ROW_LEAF_DONE] = one;
        for level in 0..TREE_DEPTH - 1 {
            acct_link[(2 + level) * CYCLE_LENGTH + 7] = one;
        }
        columns.push(acct_link);

        let mut link_leaf = vec![zero; TRACE_LENGTH];
        link_leaf[ROW_LEAF_LINK] = one;
        columns.push(link_leaf);

        // Enlaces del arbol de custodios: entrada + niveles.
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
            // Capacidad: en ambos arboles.
            result[C_CAP_A + i] = any_link * next[i];
            result[C_CAP_B + i] = any_link * next[LANE_B + i];

            let da = current[4 + i];
            let db = current[LANE_B + 4 + i];

            // --- Arbol de CUENTAS: mismo camino, hermano compartido ---
            result[C_PLACE_A + i] =
                acct_link * ((E::ONE - bit_a) * (next[4 + i] - da) + bit_a * (next[8 + i] - da));
            result[C_PLACE_B + i] = acct_link
                * ((E::ONE - bit_a) * (next[LANE_B + 4 + i] - db)
                    + bit_a * (next[LANE_B + 8 + i] - db));

            let sib_a = (E::ONE - bit_a) * next[8 + i] + bit_a * next[4 + i];
            let sib_b =
                (E::ONE - bit_a) * next[LANE_B + 8 + i] + bit_a * next[LANE_B + 4 + i];
            result[C_SIBLING + i] = acct_link * (sib_a - sib_b);

            // --- Arbol de CUSTODIOS: caminos distintos, cada carril con
            //     su bit. Aqui NO hay hermano compartido: son posiciones
            //     distintas del arbol.
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

        result[C_NONCE] = link_leaf * (next[8] - current[COL_NONCE]);
        result[C_NONCE + 1] = link_leaf * (next[LANE_B + 8] - current[COL_NONCE]);

        for i in 0..4 {
            result[C_INPUT + i] = first_row * (current[4 + i] - current[COL_ACC_ID + i]);
            result[C_INPUT + 5 + i] =
                first_row * (current[LANE_B + 4 + i] - current[COL_ACC_ID + i]);
        }
        result[C_INPUT + 4] = first_row * (current[8] - current[COL_BAL]);
        result[C_INPUT + 9] = first_row * (current[LANE_B + 8] - current[COL_BAL_NEW]);

        // Las claves de custodio entran en su derivación de identidad.
        result[C_CUST_INPUT] = sel_acct_root * (next[8] - current[COL_KEY_A]);
        result[C_CUST_INPUT + 1] = sel_acct_root * (next[LANE_B + 8] - current[COL_KEY_B]);

        // ===== EL ACUMULADOR ATA EL ÍNDICE AL CAMINO =====
        result[C_ACC] = cust_link * (next[COL_ACC_A] - (current[COL_ACC_A] + bit_a * pow2));
        result[C_ACC + 1] = cust_link * (next[COL_ACC_B] - (current[COL_ACC_B] + bit_b * pow2));

        result[C_ACC_FINAL] = sel_cust_root * (current[COL_ACC_A] - current[COL_IDX_A]);
        result[C_ACC_FINAL + 1] = sel_cust_root * (current[COL_ACC_B] - current[COL_IDX_B]);

        // ===== EL SALDO Y EL SUMINISTRO SUBEN EXACTAMENTE EN EL IMPORTE =====
        result[C_BALANCE] = current[COL_BAL_NEW] - (current[COL_BAL] + current[COL_AMT]);
        result[C_SUPPLY] =
            current[COL_SUPPLY_NEW] - (current[COL_SUPPLY_OLD] + current[COL_AMT]);

        let transport = [
            COL_KEY_A,
            COL_KEY_B,
            COL_IDX_A,
            COL_IDX_B,
            COL_BAL,
            COL_BAL_NEW,
            COL_NONCE,
            COL_AMT,
            COL_SUPPLY_OLD,
            COL_SUPPLY_NEW,
            COL_MAX_SUPPLY,
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

        let expected = [
            current[COL_BAL],
            current[COL_AMT],
            current[COL_BAL_NEW],
            current[COL_SUPPLY_NEW],
            current[COL_MAX_SUPPLY] - current[COL_SUPPLY_NEW],
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

        // Raices del arbol de cuentas.
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

        // Arranque de la derivacion de custodios: dominio anclado.
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

        // **AUTORIDAD**: los DOS carriles llegan a la misma raiz del
        // conjunto de custodios.
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
        a.push(Assertion::single(
            COL_MAX_SUPPLY,
            0,
            self.pub_inputs.max_supply,
        ));

        a
    }
}

pub struct MintProver {
    options: ProofOptions,
}

impl MintProver {
    pub fn new(options: ProofOptions) -> Self {
        Self { options }
    }
}

impl Prover for MintProver {
    type BaseField = BaseElement;
    type Air = MintAir;
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

    fn get_pub_inputs(&self, trace: &Self::Trace) -> MintPublicInputs {
        MintPublicInputs {
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
            amount: trace.get(COL_AMT, 0),
            supply_old: trace.get(COL_SUPPLY_OLD, 0),
            supply_new: trace.get(COL_SUPPLY_NEW, 0),
            max_supply: trace.get(COL_MAX_SUPPLY, 0),
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

    const MAX_SUPPLY: u64 = 100_000_000;

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
        account_id: Digest,
        balance: u64,
        nonce: BaseElement,
        path: MerklePath,
        amount: u64,
        supply_old: u64,
        public_inputs: MintPublicInputs,
    }

    fn scenario(balance: u64, amount: u64, supply_old: u64) -> Scenario {
        let mut empty = vec![[BaseElement::ZERO; 4]];
        for k in 1..=TREE_DEPTH {
            let prev = empty[k - 1];
            empty.push(native_merge(prev, prev));
        }
        let account_id = derive_public_id(BaseElement::new(0xA11CE));
        let nonce = BaseElement::ZERO;

        // Direcciones MIXTAS: con todas iguales la traza degenera.
        let mut siblings = Vec::with_capacity(TREE_DEPTH);
        let mut is_right = Vec::with_capacity(TREE_DEPTH);
        for level in 0..TREE_DEPTH {
            siblings.push(empty[level]);
            is_right.push(level % 3 == 0);
        }
        let path = MerklePath { siblings, is_right };

        let keys = custodian_keys();
        let (set_root, cpaths) = build_custodian_set(&keys);

        let leaf_old = native_leaf(account_id, BaseElement::new(balance), nonce);
        let leaf_new = native_leaf(account_id, BaseElement::new(balance + amount), nonce);

        Scenario {
            public_inputs: MintPublicInputs {
                root_old: native_climb(leaf_old, &path),
                root_new: native_climb(leaf_new, &path),
                custodian_set_root: set_root,
                amount: BaseElement::new(amount),
                supply_old: BaseElement::new(supply_old),
                supply_new: BaseElement::new(supply_old + amount),
                max_supply: BaseElement::new(MAX_SUPPLY),
            },
            auth: ThresholdAuth {
                key_a: keys[1],
                index_a: 1,
                path_a: cpaths[1].clone(),
                key_b: keys[3],
                index_b: 3,
                path_b: cpaths[3].clone(),
            },
            account_id,
            balance,
            nonce,
            path,
            amount,
            supply_old,
        }
    }

    fn build(s: &Scenario, auth: &ThresholdAuth, supply_delta: u64) -> TraceTable<BaseElement> {
        build_trace(
            auth,
            s.account_id,
            s.balance,
            s.nonce,
            &s.path,
            s.amount,
            s.supply_old,
            supply_delta,
            MAX_SUPPLY,
        )
    }

    fn run(s: &Scenario, auth: &ThresholdAuth, supply_delta: u64) -> Result<(), String> {
        let trace = build(s, auth, supply_delta);
        let prover = MintProver::new(default_options());

        let hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| prover.prove(trace)));
        std::panic::set_hook(hook);

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
        verify::<MintAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
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
        let s = scenario(1000, 500_000, 10_000_000);
        let trace = build(&s, &s.auth, s.amount);
        for i in 0..4 {
            assert_eq!(
                trace.get(4 + i, ROW_ACCT_ROOT),
                s.public_inputs.root_old[i],
                "root_old {i}"
            );
            assert_eq!(
                trace.get(LANE_B + 4 + i, ROW_ACCT_ROOT),
                s.public_inputs.root_new[i],
                "root_new {i}"
            );
            assert_eq!(
                trace.get(4 + i, ROW_CUST_ROOT),
                s.public_inputs.custodian_set_root[i],
                "raiz de custodios, carril A, elem {i}"
            );
            assert_eq!(
                trace.get(LANE_B + 4 + i, ROW_CUST_ROOT),
                s.public_inputs.custodian_set_root[i],
                "raiz de custodios, carril B, elem {i}"
            );
        }
        assert_eq!(trace.get(COL_ACC_A, ROW_CUST_ROOT), BaseElement::new(1));
        assert_eq!(trace.get(COL_ACC_B, ROW_CUST_ROOT), BaseElement::new(3));
        // ===== Y TODAS LAS ENTRADAS PÚBLICAS, NO SOLO LAS RAÍCES =====
        //
        // Comparar la estructura entera. En `circuit_send` la versión
        // parcial dejó pasar un campo heredado de otra operación y **costó
        // ocho rondas de diagnóstico**: probador y verificador usaban
        // transcripciones de Fiat-Shamir distintas, y el error de winterfell
        // —`InconsistentOodConstraintEvaluations`— apunta a las
        // restricciones, no a las entradas.
        let derivadas = MintProver::new(default_options()).get_pub_inputs(&trace);
        assert_eq!(
            derivadas.to_elements(),
            s.public_inputs.to_elements(),
            "las entradas DERIVADAS de la traza deben coincidir con las \
             DECLARADAS en todos sus campos"
        );

    }

    /// EL TEST CLAVE. No silencia el pánico: si una traza válida falla,
    /// queremos ver qué restricción y en qué fila.
    #[test]
    fn threshold_authorized_mint_verifies() {
        let s = scenario(1000, 500_000, 10_000_000);
        let trace = build(&s, &s.auth, s.amount);
        let prover = MintProver::new(default_options());
        let proof = prover.prove(trace).expect("la emision valida deberia probar");
        let min_opts = AcceptableOptions::OptionSet(vec![default_options()]);
        let v = verify::<MintAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof,
            s.public_inputs.clone(),
            &min_opts,
        );
        assert!(v.is_ok(), "{v:?}");
    }

    /// **EL TEST QUE IMPIDE EL 1-DE-N DISFRAZADO.**
    #[test]
    fn the_same_custodian_cannot_count_twice() {
        let s = scenario(1000, 500_000, 10_000_000);
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
        assert!(
            run(&s, &auth, s.amount).is_err(),
            "CRITICO: un 2-de-N en el que un custodio cuente dos veces es un \
             1-de-N disfrazado"
        );
    }

    /// **QUIEN NO ES CUSTODIO NO PUEDE EMITIR.**
    #[test]
    fn a_non_custodian_cannot_mint() {
        let s = scenario(1000, 500_000, 10_000_000);
        let keys = custodian_keys();
        let (_, cpaths) = build_custodian_set(&keys);
        let auth = ThresholdAuth {
            key_a: BaseElement::new(0x1337), // intruso
            index_a: 1,
            path_a: cpaths[1].clone(),
            key_b: keys[3],
            index_b: 3,
            path_b: cpaths[3].clone(),
        };
        assert!(
            run(&s, &auth, s.amount).is_err(),
            "CRITICO: sin pertenecer al conjunto de custodios no debe poder \
             crearse dinero"
        );
    }

    /// El índice está atado al camino: no se puede mentir para burlar el
    /// orden estricto.
    #[test]
    fn a_lied_custodian_index_is_rejected() {
        let s = scenario(1000, 500_000, 10_000_000);
        let keys = custodian_keys();
        let (_, cpaths) = build_custodian_set(&keys);
        let auth = ThresholdAuth {
            key_a: keys[2],
            index_a: 0, // miente para "adelantarse"
            path_a: cpaths[2].clone(),
            key_b: keys[1],
            index_b: 1,
            path_b: cpaths[1].clone(),
        };
        assert!(run(&s, &auth, s.amount).is_err());
    }

    /// Emisión encubierta: aumentar un saldo sin reflejarlo en el
    /// suministro público.
    #[test]
    fn minting_without_updating_supply_is_rejected() {
        let s = scenario(1000, 500_000, 10_000_000);
        assert!(run(&s, &s.auth, 0).is_err());
    }

    /// Inflar el suministro más de lo emitido.
    #[test]
    fn inflating_supply_beyond_amount_is_rejected() {
        let s = scenario(1000, 500_000, 10_000_000);
        assert!(run(&s, &s.auth, 1_000_000).is_err());
    }

    /// **NI DOS CUSTODIOS PUEDEN SUPERAR EL TOPE.**
    #[test]
    fn minting_beyond_the_cap_is_rejected() {
        let s = scenario(1000, 5_000_000, 99_000_000);
        assert!(
            run(&s, &s.auth, s.amount).is_err(),
            "CRITICO: el tope limita incluso a la autoridad completa"
        );
    }

    /// Declarar una raíz de cuentas nueva que no corresponde.
    #[test]
    fn wrong_new_root_is_rejected() {
        let s = scenario(1000, 500_000, 10_000_000);
        let trace = build(&s, &s.auth, s.amount);
        let prover = MintProver::new(default_options());
        let proof = prover.prove(trace).expect("prove");
        let mut declared = s.public_inputs.clone();
        declared.root_new = [BaseElement::new(999_999); 4];
        let min_opts = AcceptableOptions::OptionSet(vec![default_options()]);
        let v = verify::<MintAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof, declared, &min_opts,
        );
        assert!(v.is_err());
    }

    /// Declarar un conjunto de custodios distinto.
    #[test]
    fn wrong_custodian_set_root_is_rejected() {
        let s = scenario(1000, 500_000, 10_000_000);
        let trace = build(&s, &s.auth, s.amount);
        let prover = MintProver::new(default_options());
        let proof = prover.prove(trace).expect("prove");
        let mut declared = s.public_inputs.clone();
        declared.custodian_set_root = [BaseElement::new(999); 4];
        let min_opts = AcceptableOptions::OptionSet(vec![default_options()]);
        let v = verify::<MintAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
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

        let s = scenario(1000, 500_000, 10_000_000);
        let trace = build(&s, &s.auth, s.amount);
        let rows = rows_of(&trace, TRACE_WIDTH, TRACE_LENGTH);

        let air = MintAir::new(
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

    /// **EL RANGO ES DE 63 BITS, Y DE ESO DEPENDE QUE EL TOPE SE IMPONGA.**
    ///
    /// El tope se comprueba descomponiendo `tope − suministro_nuevo` en
    /// bits. Si el suministro se pasara del tope, esa resta **envuelve** en
    /// el campo y da un valor cercano a `p ≈ 2^64`.
    ///
    /// Que ese valor envuelto sea rechazado depende de que el segmento dé
    /// **63 bits y no 64**:
    ///
    /// | | |
    /// |---|---|
    /// | Máximo con 63 bits | 9.223.372.036.854.775.807 |
    /// | Valor envuelto | ~18.446.744.069.xxx.xxx.xxx |
    /// | ¿Cabe? | **No** — por eso se rechaza |
    ///
    /// **Con 64 bits sí cabría**, el tope dejaría de imponerse, y ningún
    /// test lo notaría: los testigos honestos pasan igual y los
    /// adversariales fallan antes por otras restricciones.
    ///
    /// El margen sale de que `cont_s` cubre `SEGMENT_LENGTH − 1 = 63`
    /// transiciones, no 64. Es un invariante de **un solo bit** que nada
    /// más comprueba.
    #[test]
    fn the_range_segment_is_63_bits_not_64() {
        let air = MintAir::new(
            TraceInfo::new(TRACE_WIDTH, TRACE_LENGTH),
            scenario(1000, 500_000, 10_000_000).public_inputs,
            default_options(),
        );
        let periodic = air.get_periodic_column_values();

        // Transiciones activas dentro del primer segmento.
        let cont = &periodic[P_CONT_S];
        let activas = (0..SEGMENT_LENGTH)
            .filter(|p| cont[*p] == BaseElement::ONE)
            .count();

        assert_eq!(
            activas,
            SEGMENT_LENGTH - 1,
            "el segmento debe dar SEGMENT_LENGTH-1 = {} bits. Con {} bits \
             el valor envuelto de una resta negativa cabria en el rango y \
             el tope dejaria de imponerse",
            SEGMENT_LENGTH - 1,
            SEGMENT_LENGTH
        );

        // Y que ese numero de bits deja fuera el valor envuelto.
        let bits = activas as u32;
        assert!(bits < 64, "con 64 bits todo elemento del campo cabe");

        let maximo_representable = (1u128 << bits) - 1;
        let p = (1u128 << 64) - (1u128 << 32) + 1;
        let envuelto = p - 50_000_000; // tope 100M, suministro 150M
        assert!(
            envuelto > maximo_representable,
            "el valor envuelto ({envuelto}) debe quedar FUERA del rango \
             representable ({maximo_representable}), o el tope no se impone"
        );
    }
}
