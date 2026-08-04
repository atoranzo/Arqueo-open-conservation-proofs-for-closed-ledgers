//! ⚠️ **ANDAMIO B13/B14 — GEMELO del mundo nuevo (paso 3, §148).**
//!
//! Copia declarada de `circuit_mint_climb` — la fase de cuentas de la
//! emisión, aislada: hoja + subida, dos carriles, sin titularidad ni
//! custodios ni frozen (R6 no aplica). Por el playbook
//! (`doc/playbook-replica-gemelos.md`); el ancho ya venía derivado
//! (`COL_SACC + 1`) y el testigo entra en el mismo estilo.
//!
//! **Cláusula de retirada**: en el flip (release única, D4) este módulo
//! SUSTITUYE a `circuit_mint_climb` y el legacy se borra. Hasta
//! entonces, nadie fuera de los tests de este crate lo importa.
//!
//! ---
//!
//! # Emision a una cuenta, SIN autorizacion (§66)
//!
//! La parte propia de `circuit_mint`: construir la hoja antigua y la nueva,
//! subirlas al arbol de cuentas con los mismos hermanos, y probar que el
//! saldo y el suministro **suben exactamente en el importe** sin pasar del
//! tope. La autorizacion de custodios se ha amputado (entrada 33).
//!
//! ## Lo que SI sigue probando, y es lo que hace que este circuito importe
//!
//! - `saldo_nuevo = saldo + importe` y `suministro_nuevo = suministro +
//!   importe`: **conservacion**. Emitir no puede regalar de mas a la cuenta
//!   ni contabilizar de menos en el suministro.
//! - **El margen del tope**: `tope - suministro_nuevo` descompuesto en 64
//!   bits. Es lo que impide emitir por encima del limite, y es la propiedad
//!   por la que este circuito no puede reducirse a una subida de Merkle.
//!
//! Sin esto, dos custodios podrian firmar una raiz cualquiera y un auditor
//! externo no sabria si se respeto el tope.
//!
//! ## Lo que NO prueba
//!
//! Quien autorizo. Eso lo comprueba la capa con dos pruebas de
//! `circuit_threshold_single_nullifier` atadas a esta transicion.

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
/// 512 filas. La tubería acaba en `ROW_ACCT_ROOT` (fila 271): quedan
/// **240 filas de holgura** (30 ciclos). Sin frozen ni custodios, el
/// mundo nuevo solo suma el ciclo del salt: 279, y 512 ALCANZA (§3).
pub const TRACE_LENGTH: usize = 512;
pub const SEGMENT_LENGTH: usize = 64;
/// Segmentos: saldo, importe, saldo nuevo, suministro nuevo,
/// `tope − suministro nuevo`, índice A, índice B, y `B − A − 1`.
/// Cinco: saldo, importe, saldo nuevo, suministro nuevo y **el margen del
/// tope**. Los tres que se van eran los indices de custodio y su orden.
///
/// ⚠️ Al bajar de 8, los segmentos **dejan de llenar la traza**
/// (5x64=320 de 512) y las columnas del rango pierden la periodicidad de 64
/// que tenian: sus grados vuelven a declararse con `TRACE_LENGTH`. Lo
/// advertia el comentario de la lista de grados del circuito original.
pub const NUM_SEGMENTS: usize = 5;

// ===== Columnas =====
const LANE_B: usize = STATE_WIDTH; // 12
/// Bit de dirección del carril A. Durante la fase de cuenta lo usan
/// ambos carriles (mismo camino); durante la de custodios, solo A.
const COL_BIT_A: usize = 24;
/// Bit del carril B, usado solo en la fase de custodios.
// No hay segundo bit de direccion: los dos carriles comparten camino en la
// subida al arbol de cuentas (misma posicion, distinto saldo). El otro
// existia solo para los custodios (§64.2).
const COL_ACC_ID: usize = COL_BIT_A + 1; // 25..29
const COL_BAL: usize = COL_ACC_ID + 4; // 29
const COL_BAL_NEW: usize = COL_BAL + 1; // 30
const COL_NONCE: usize = COL_BAL_NEW + 1; // 31
const COL_AMT: usize = COL_NONCE + 1; // 32
const COL_SUPPLY_OLD: usize = COL_AMT + 1; // 33
const COL_SUPPLY_NEW: usize = COL_SUPPLY_OLD + 1; // 34
const COL_MAX_SUPPLY: usize = COL_SUPPLY_NEW + 1; // 35
const COL_SBIT: usize = COL_MAX_SUPPLY + 1; // 36
const COL_SACC: usize = COL_SBIT + 1; // 37
/// **Salt de la hoja** (testigo, §117): envuelve la hoja como tercer
/// merge. UN solo salt compartido por ambos carriles (spec §2). Sin
/// colisión: no hay COL_SALT previo. En el estilo derivado de la casa.
const COL_LEAF_SALT: usize = COL_SACC + 1; // 38..42
pub const TRACE_WIDTH: usize = COL_LEAF_SALT + 4; // 42

// ===== Filas =====
//
// Geometría derivada (playbook R2; el patrón de SB0, §140-§141). Un
// solo tramo: hoja + subida de cuentas — `CYC_FIN = CYC_ACC +
// TREE_DEPTH`, sin titularidad, frozen ni pendientes.
//
// Convención: todo arranque de tramo es un `CYC_*`; ningún literal de
// ciclo vive fuera de este bloque — bucles de bits, periódicas y el
// `match` de `build_trace` lo derivan de aquí.
const CYC_NONCE: usize = 1;
// El TERCER merge (§117, B13/B14): la hoja se envuelve con el salt
// antes de entrar al camino. Todo el calendario posterior se corre +1
// ciclo solo, por derivación (playbook R2).
const CYC_SALT: usize = CYC_NONCE + 1;
const CYC_ACC: usize = CYC_SALT + 1;
const CYC_FIN: usize = CYC_ACC + TREE_DEPTH;
const ROW_LEAF_LINK: usize = CYC_NONCE * CYCLE_LENGTH - 1;
const ROW_SALT_LINK: usize = CYC_SALT * CYCLE_LENGTH - 1;
const ROW_LEAF_DONE: usize = CYC_ACC * CYCLE_LENGTH - 1;
/// Última fila activa: raíz del árbol de cuentas. (El doc heredado
/// hablaba de custodios — deriva del legacy de mint; aquí no hay.)
const ROW_ACCT_ROOT: usize = CYC_FIN * CYCLE_LENGTH - 1;

// El presupuesto, en compilación: la tubería debe caber en la traza.
const _: () = assert!(ROW_ACCT_ROOT < TRACE_LENGTH);
// No hace falta constante para el relleno: la 271 no es fila de enlace,
// asi que la transicion 271->272 no activa ninguna restriccion y las
// filas siguientes se quedan a cero sin que nada las mire.
/// Última fila activa: raíz del conjunto de custodios.

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
const C_BIT_BOOL: usize = C_SIBLING + 4; // 1
const C_LEAF_CAP_A: usize = C_BIT_BOOL + 1; // 4
const C_LEAF_CAP_B: usize = C_LEAF_CAP_A + 4;
const C_LEAF_DIG_A: usize = C_LEAF_CAP_B + 4; // 4
const C_LEAF_DIG_B: usize = C_LEAF_DIG_A + 4;
const C_NONCE: usize = C_LEAF_DIG_B + 4; // 2
const C_INPUT: usize = C_NONCE + 2; // 10
/// **EL SALDO Y EL SUMINISTRO SUBEN EXACTAMENTE EN EL IMPORTE.**
const C_BALANCE: usize = C_INPUT + 10; // 1
const C_SUPPLY: usize = C_BALANCE + 1; // 1
const C_TRANSPORT: usize = C_SUPPLY + 1; // 7
const C_ID_CONST: usize = C_TRANSPORT + 7; // 4
const C_SBIT_BOOL: usize = C_ID_CONST + 4; // 2
const C_FIRST_S: usize = C_SBIT_BOOL + 2; // 2
const C_HORNER: usize = C_FIRST_S + 2; // 1
const C_SEG_LINK: usize = C_HORNER + 1; // NUM_SEGMENTS
pub const NUM_CONSTRAINTS: usize = C_SEG_LINK + NUM_SEGMENTS;

// ===== Periódicas =====
const P_HASH_FLAG: usize = 0;
const P_ARK1: usize = 1;
const P_ARK2: usize = P_ARK1 + STATE_WIDTH;
/// Enlaces del árbol de cuentas (niveles + colocación de la hoja).
const P_ACCT_LINK: usize = P_ARK2 + STATE_WIDTH;
const P_LINK_LEAF: usize = P_ACCT_LINK + 1;
/// Enlaces del árbol de custodios (entrada + niveles).
// Se fueron `P_CUST_LINK`, `P_POW2` y `P_SEL_CUST_ROOT` con la amputacion.
// Dejarlos en la cadena desplazaba `P_SEG_LINK` tres posiciones y el indice
// se salia del array de periodicas.
const P_FIRST_ROW: usize = P_LINK_LEAF + 1;
// Se fue tambien `P_SEL_ACCT_ROOT`: solo lo leia `C_CUST_INPUT`. Una
// periodica que se construye y nadie lee es peso muerto, y ensucia una
// cadena que -entrada 39- no comprueba nada.
const P_FIRST_S: usize = P_FIRST_ROW + 1;
const P_CONT_S: usize = P_FIRST_S + 1;
const P_SEG_LINK: usize = P_CONT_S + 1;

type Blake3 = Blake3_256<BaseElement>;

fn value_to_bits_be(value: u64) -> Vec<bool> {
    (0..SEGMENT_LENGTH)
        .map(|p| (value >> (SEGMENT_LENGTH - 1 - p)) & 1 == 1)
        .collect()
}

/// Autorización de dos custodios.

/// Construye la traza de una emisión autorizada por umbral.
///
/// `supply_delta` permite variar el suministro en una cantidad distinta
/// de la emitida, para los tests de emisión encubierta.
#[allow(clippy::too_many_arguments)]
pub fn build_trace(
    account_id: Digest,
    balance: u64,
    nonce: BaseElement,
    // **Salt de la hoja (testigo).** Del TITULAR de la cuenta acreditada
    // (§117): la pertenencia se prueba sobre `H(native_leaf, salt)`.
    leaf_salt: Digest,
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
        row[COL_LEAF_SALT..COL_LEAF_SALT + 4].copy_from_slice(&leaf_salt);
    }

    // Rangos. El quinto demuestra que no se supera el tope; los tres
    // ultimos, que los custodios son distintos y estan en orden.
    let segment_values = [
        c_bal.as_int(),
        c_amt.as_int(),
        c_bal_new.as_int(),
        c_supply_new.as_int(),
        (c_max - c_supply_new).as_int(),
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
        debug_assert!(
            level < TREE_DEPTH,
            "place_acct: nivel {} sobre path de {}",
            level,
            TREE_DEPTH
        );
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
                    // El nonce NO cambia: emitir no consume el derecho de
                    // gasto del titular.
                    state_a[4..8].copy_from_slice(&digest_a);
                    state_a[8] = nonce;
                    state_b[4..8].copy_from_slice(&digest_b);
                    state_b[8] = nonce;
                }
                ROW_SALT_LINK => {
                    // EL TERCER MERGE (§117): la hoja se envuelve con el
                    // salt. Digest arrastrado; el rate recibe los CUATRO
                    // limbos del salt (spec §2 — atar solo [8] sería el
                    // bug de §92.2 en su forma nueva).
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
                    // Convención única (playbook R2): tramo genérico =
                    // `(CYC_arranque..CYC_fin_de_tramo)`, nivel =
                    // `next_cycle - CYC_arranque`; el arranque lo sombrea
                    // el brazo de `ROW_LEAF_DONE` (nivel 0 explícito).
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

    // Bits del arbol de cuentas: ambos carriles el mismo camino.
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
    // Bits del arbol de custodios: caminos DISTINTOS por carril.

    let mut trace = TraceTable::new(TRACE_WIDTH, TRACE_LENGTH);
    trace.fill(
        |s| s.copy_from_slice(&rows[0]),
        |step, s| s.copy_from_slice(&rows[step + 1]),
    );
    trace
}

#[derive(Clone, Debug)]
pub struct MintClimbPublicInputs {
    pub root_old: Digest,
    pub root_new: Digest,
    pub amount: BaseElement,
    pub supply_old: BaseElement,
    pub supply_new: BaseElement,
    pub max_supply: BaseElement,
}

impl ToElements<BaseElement> for MintClimbPublicInputs {
    fn to_elements(&self) -> Vec<BaseElement> {
        let mut out = self.root_old.to_vec();
        out.extend_from_slice(&self.root_new);
        out.push(self.amount);
        out.push(self.supply_old);
        out.push(self.supply_new);
        out.push(self.max_supply);
        out
    }
}

pub struct MintClimbAir {
    context: AirContext<BaseElement>,
    pub_inputs: MintClimbPublicInputs,
}

impl Air for MintClimbAir {
    type BaseField = BaseElement;
    type PublicInputs = MintClimbPublicInputs;

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
        // Colocacion de cuentas (8) + hermano (4): grado 2.
        for _ in 0..12 {
            degrees.push(TransitionConstraintDegree::with_cycles(2, full.clone()));
        }
        // Bit booleano (1): grado 2 sin ciclo.
        degrees.push(TransitionConstraintDegree::new(2));
        // Enlaces de hoja (16), nonce (2), entradas (10): grado 1 con ciclo.
        for _ in 0..28 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }
        // Saldo (1), suministro (1), transporte (7), identidad (4):
        // grado 1 sin ciclo.
        for _ in 0..13 {
            degrees.push(TransitionConstraintDegree::new(1));
        }
        // Bits de rango (2): grado 2 sin ciclo.
        for _ in 0..2 {
            degrees.push(TransitionConstraintDegree::new(2));
        }
        // ⚠️ Con OCHO segmentos el rango llenaba exactamente las 512 filas y
        // estas columnas eran periodicas de periodo 64. Con CINCO ya no
        // (5x64=320): quedan filas a cero que rompen la periodicidad, y el
        // ciclo correcto vuelve a ser `TRACE_LENGTH`. Lo advertia el
        // comentario del circuito original, y sin el esto habria petado con
        // un panico de grados.
        for _ in 0..3 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }
        // Enlaces de segmento: cada uno tiene un único uno en una
        // posición distinta, así que NO son periódicos.
        for _ in 0..NUM_SEGMENTS {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }

        assert_eq!(degrees.len(), NUM_CONSTRAINTS, "cuenta de grados");

        MintClimbAir {
            context: AirContext::new(trace_info, degrees, 26, options),
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

        // Enlaces del arbol de cuentas: colocacion de la hoja + niveles.
        let mut acct_link = vec![zero; TRACE_LENGTH];
        acct_link[ROW_LEAF_DONE] = one;
        for level in 0..TREE_DEPTH - 1 {
            acct_link[(CYC_ACC + level) * CYCLE_LENGTH + 7] = one;
        }
        columns.push(acct_link);

        let mut link_leaf = vec![zero; TRACE_LENGTH];
        link_leaf[ROW_LEAF_LINK] = one;
        columns.push(link_leaf);

        // Enlaces del arbol de custodios: entrada + niveles.

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
            result[C_INPUT + i] = first_row * (current[4 + i] - current[COL_ACC_ID + i]);
            result[C_INPUT + 5 + i] =
                first_row * (current[LANE_B + 4 + i] - current[COL_ACC_ID + i]);
        }
        result[C_INPUT + 4] = first_row * (current[8] - current[COL_BAL]);
        result[C_INPUT + 9] = first_row * (current[LANE_B + 8] - current[COL_BAL_NEW]);

        // Las claves de custodio entran en su derivación de identidad.

        // ===== EL SALDO Y EL SUMINISTRO SUBEN EXACTAMENTE EN EL IMPORTE =====
        result[C_BALANCE] = current[COL_BAL_NEW] - (current[COL_BAL] + current[COL_AMT]);
        result[C_SUPPLY] =
            current[COL_SUPPLY_NEW] - (current[COL_SUPPLY_OLD] + current[COL_AMT]);

        let transport = [
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
            // El margen del tope: si fuera negativo no cabria en 64 bits.
            current[COL_MAX_SUPPLY] - current[COL_SUPPLY_NEW],
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

pub struct MintClimbProver {
    options: ProofOptions,
}

impl MintClimbProver {
    pub fn new(options: ProofOptions) -> Self {
        Self { options }
    }
}

impl Prover for MintClimbProver {
    type BaseField = BaseElement;
    type Air = MintClimbAir;
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

    fn get_pub_inputs(&self, trace: &Self::Trace) -> MintClimbPublicInputs {
        MintClimbPublicInputs {
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
    use crate::circuit_settlement::{
        derive_leaf_salt_wide, derive_public_id_wide, native_climb,
        native_leaf_salted,
    };
    use crate::merkle::native_merge;
    use winterfell::{verify, AcceptableOptions, BatchingMethod, FieldExtension, Prover};

    const MAX_SUPPLY: u64 = 100_000_000;

    fn default_options() -> ProofOptions {
        ProofOptions::new(
            32, 8, 0, FieldExtension::None, 8, 31,
            BatchingMethod::Linear, BatchingMethod::Linear,
        )
    }

    struct Scenario {
        account_id: Digest,
        balance: u64,
        nonce: BaseElement,
        leaf_salt: Digest,
        path: MerklePath,
        amount: u64,
        supply_old: u64,
        public_inputs: MintClimbPublicInputs,
    }

    fn scenario(balance: u64, amount: u64, supply_old: u64) -> Scenario {
        let mut empty = vec![[BaseElement::ZERO; 4]];
        for k in 1..=TREE_DEPTH {
            let prev = empty[k - 1];
            empty.push(native_merge(prev, prev));
        }
        // ⚠️ Ancha de verdad, no `as_digest(x)`: con relleno de ceros el
        // test pasaria sin ejercitar los elementos altos (§90.3). El
        // TITULAR de la cuenta acreditada asciende al mundo ancho: su
        // clave manda sobre identidad Y salt (§117).
        let key = [
            BaseElement::new(0xA11CE_0001),
            BaseElement::new(0xA11CE_0002),
            BaseElement::new(0xA11CE_0003),
            BaseElement::new(0xA11CE_0004),
        ];
        let account_id = derive_public_id_wide(key);
        let nonce = BaseElement::ZERO;

        let mut siblings = Vec::with_capacity(TREE_DEPTH);
        let mut is_right = Vec::with_capacity(TREE_DEPTH);
        for level in 0..TREE_DEPTH {
            siblings.push(empty[level]);
            is_right.push(level % 3 == 0);
        }
        let path = MerklePath { siblings, is_right };

        // El salt REAL del titular (§117): derivado de la clave, no un
        // literal de juguete — el escenario vive en el mundo envuelto.
        let leaf_salt = derive_leaf_salt_wide(key);
        let leaf_old =
            native_leaf_salted(account_id, BaseElement::new(balance), nonce, leaf_salt);
        let leaf_new =
            native_leaf_salted(account_id, BaseElement::new(balance + amount), nonce, leaf_salt);

        Scenario {
            public_inputs: MintClimbPublicInputs {
                root_old: native_climb(leaf_old, &path),
                root_new: native_climb(leaf_new, &path),
                amount: BaseElement::new(amount),
                supply_old: BaseElement::new(supply_old),
                supply_new: BaseElement::new(supply_old + amount),
                max_supply: BaseElement::new(MAX_SUPPLY),
            },
            account_id, balance, nonce, leaf_salt, path, amount, supply_old,
        }
    }

    fn build(s: &Scenario, supply_delta: u64) -> TraceTable<BaseElement> {
        build_trace(s.account_id, s.balance, s.nonce, s.leaf_salt, &s.path,
                    s.amount, s.supply_old, supply_delta, MAX_SUPPLY)
    }

    fn run(s: &Scenario, supply_delta: u64) -> Result<(), String> {
        let trace = build(s, supply_delta);
        let prover = MintClimbProver::new(default_options());
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| prover.prove(trace)));
        let proof = match r {
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
        verify::<MintClimbAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof, s.public_inputs.clone(), &min_opts,
        ).map_err(|e| format!("verificacion fallo: {e:?}"))
    }

    #[test]
    fn trace_roots_match_native() {
        let s = scenario(1_000, 500, 10_000);
        let trace = build(&s, 500);
        for i in 0..4 {
            assert_eq!(trace.get(4 + i, ROW_ACCT_ROOT), s.public_inputs.root_old[i],
                       "carril A, elem {i}");
            assert_eq!(trace.get(LANE_B + 4 + i, ROW_ACCT_ROOT), s.public_inputs.root_new[i],
                       "carril B, elem {i}");
        }
    }

    #[test]
    fn a_valid_mint_climb_verifies() {
        let s = scenario(1_000, 500, 10_000);
        assert_eq!(run(&s, 500), Ok(()));
    }

    /// CONSERVACION. El suministro debe subir EXACTAMENTE en el importe: ni
    /// mas -se crearia dinero sin contabilizar- ni menos.
    #[test]
    fn the_supply_must_rise_by_exactly_the_amount() {
        let s = scenario(1_000, 500, 10_000);
        assert!(run(&s, 499).is_err(), "SOLIDEZ: contabilizar de menos");
        assert!(run(&s, 501).is_err(), "SOLIDEZ: contabilizar de mas");
        assert!(run(&s, 0).is_err(), "SOLIDEZ: no contabilizar nada");
    }

    /// EL TOPE. Es la propiedad por la que este circuito no puede reducirse a
    /// una subida de Merkle: el margen `tope - suministro_nuevo` se
    /// descompone en 64 bits, y si fuera negativo no cabria.
    #[test]
    fn minting_over_the_cap_is_rejected() {
        let s = scenario(1_000, 500, MAX_SUPPLY);
        assert!(run(&s, 500).is_err(),
            "SOLIDEZ: emitir por encima del tope debe rechazarse");
    }

    /// Y el limite exacto SI vale: emitir hasta el tope justo es legitimo.
    #[test]
    fn minting_exactly_up_to_the_cap_is_allowed() {
        let s = scenario(1_000, 500, MAX_SUPPLY - 500);
        assert_eq!(run(&s, 500), Ok(()));
    }

    #[test]
    fn wrong_declared_root_is_rejected() {
        let mut s = scenario(1_000, 500, 10_000);
        s.public_inputs.root_new = s.public_inputs.root_old;
        assert!(run(&s, 500).is_err());
    }

    #[test]
    fn no_constraint_is_vacuous() {
        use crate::mutation::{buscar_vacias, rows_of};
        let s = scenario(1_000, 500, 10_000);
        let trace = build(&s, 500);
        let rows = rows_of(&trace, TRACE_WIDTH, TRACE_LENGTH);
        let air = MintClimbAir::new(
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
