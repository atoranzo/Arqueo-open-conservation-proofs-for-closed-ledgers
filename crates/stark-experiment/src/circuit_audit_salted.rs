//! ⚠️ **ANDAMIO B13/B14 — GEMELO del mundo nuevo (paso 3, §147).**
//!
//! Copia declarada de `circuit_audit` donde vive su migración — hoja
//! envuelta y las mutaciones obligatorias — por el playbook
//! (`doc/playbook-replica-gemelos.md`). Audit es de **UN carril** (no
//! muta estado: prueba `saldo ∈ [inferior, superior]` sin tocarlo):
//! las familias del salt son TRES (12 ranuras), el espejo es de una
//! hoja, y el salt entra como campo del testigo `AuditWitness` — cero
//! firmas tocadas. Sin fase frozen: R6 no aplica.
//!
//! **Cláusula de retirada**: en el flip (release única, D4) este módulo
//! SUSTITUYE a `circuit_audit` y el legacy se borra. Hasta entonces,
//! nadie fuera de los tests de este crate lo importa.
//!
//! ---
//!
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

use crate::circuit_settlement::{derive_public_id_wide, SPEND_KEY_DOMAIN};
use crate::merkle::{Digest, MerklePath, TREE_DEPTH};
use crate::rescue_hash::{apply_sbox, NUM_ROUNDS, STATE_WIDTH};

pub const CYCLE_LENGTH: usize = 8;
/// 512 filas. La tubería acaba en `ROW_PK_DONE` (fila 279): quedan
/// **232 filas de holgura** (29 ciclos). Sin fase frozen, el mundo
/// nuevo solo suma el ciclo del salt: 287, y 512 ALCANZA (spec §3).
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
/// Clave de gasto. ⚠️ **CUATRO elementos** desde §90 (entrada 15).
const COL_KEY: usize = 13; // 13..17
const COL_ID: usize = 17; // 17..21
const COL_BAL: usize = 21;
const COL_NONCE: usize = 22;
const COL_LOWER: usize = 23;
const COL_UPPER: usize = 24;
const COL_SBIT: usize = 25;
const COL_SACC: usize = 26;
/// **Salt de la hoja** (testigo, §117): envuelve la hoja como tercer
/// merge — UN carril, un salt (spec de la máquina de hoja §2). Sin
/// colisión en audit: no hay COL_SALT previo.
const COL_LEAF_SALT: usize = 27; // 27..31
pub const TRACE_WIDTH: usize = 31;

// ===== Filas de eventos =====
//
// Geometría derivada (playbook R2; el patrón de SB0, §140-§141). La
// cadena más corta de la campaña: hoja, subida de cuentas y
// titularidad — `CYC_FIN = CYC_PK + 1`, sin frozen ni pendientes.
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
const CYC_PK: usize = CYC_ACC + TREE_DEPTH;
const CYC_FIN: usize = CYC_PK + 1;
const ROW_LEAF_LINK: usize = CYC_NONCE * CYCLE_LENGTH - 1;
const ROW_SALT_LINK: usize = CYC_SALT * CYCLE_LENGTH - 1;
const ROW_LEAF_DONE: usize = CYC_ACC * CYCLE_LENGTH - 1;
const ROW_ROOT: usize = CYC_PK * CYCLE_LENGTH - 1;
const ROW_PK_START: usize = CYC_PK * CYCLE_LENGTH;
const ROW_PK_DONE: usize = CYC_FIN * CYCLE_LENGTH - 1;

// El presupuesto, en compilación: la tubería debe caber en la traza.
const _: () = assert!(ROW_PK_DONE < TRACE_LENGTH);

// ===== Índices de restricción =====
const C_HASH: usize = 0; // 12
const C_TREE_CAP: usize = C_HASH + STATE_WIDTH; // 4
const C_PLACE: usize = C_TREE_CAP + 4; // 4
const C_BIT_BOOL: usize = C_PLACE + 4; // 1
const C_LEAF_CAP: usize = C_BIT_BOOL + 1; // 4
const C_LEAF_DIG: usize = C_LEAF_CAP + 4; // 4
const C_NONCE: usize = C_LEAF_DIG + 4; // 1
const C_INPUT: usize = C_NONCE + 1; // 5: identidad (4) + saldo
const C_PK_INPUT: usize = C_INPUT + 5; // 4
/// **TITULARIDAD**: la pk derivada coincide con la identidad auditada.
const C_PK_CHECK: usize = C_PK_INPUT + 4; // 4
const C_TRANSPORT: usize = C_PK_CHECK + 4; // 8
const C_ID_CONST: usize = C_TRANSPORT + 8; // 4
const C_SBIT_BOOL: usize = C_ID_CONST + 4; // 2
const C_FIRST_S: usize = C_SBIT_BOOL + 2; // 2
const C_HORNER: usize = C_FIRST_S + 2; // 1
const C_SEG_LINK: usize = C_HORNER + 1; // NUM_SEGMENTS
/// **La envoltura de la hoja (§117, B13/B14) — UN carril.** Tres
/// familias cosidas por `link_salt` en `ROW_SALT_LINK`: capacidad a
/// cero, digest arrastrado, y los CUATRO limbos del rate atados al
/// salt testigo (§138 en los cuatro limbos).
const C_SALT_CAP: usize = C_SEG_LINK + NUM_SEGMENTS; // 4
const C_SALT_DIG: usize = C_SALT_CAP + 4; // 4
const C_SALT_IN: usize = C_SALT_DIG + 4; // 4
const NUM_CONSTRAINTS: usize = C_SALT_IN + 4;

// ===== Columnas periódicas =====
const P_HASH_FLAG: usize = 0;
const P_ARK1: usize = 1;
const P_ARK2: usize = P_ARK1 + STATE_WIDTH;
const P_LINK_MERKLE: usize = P_ARK2 + STATE_WIDTH;
const P_LINK_LEAF: usize = P_LINK_MERKLE + 1;
/// Fila del TERCER merge: la envoltura del salt (§117).
const P_LINK_SALT: usize = P_LINK_LEAF + 1;
const P_LINK_PLACE: usize = P_LINK_SALT + 1;
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
    /// ⚠️ **CUATRO elementos** desde §90 (entrada 15).
    pub spend_key: Digest,
    pub balance: u64,
    pub nonce: BaseElement,
    /// **Salt de la hoja (testigo, §117).** Deriva de la clave; la
    /// pertenencia se prueba sobre `H(native_leaf, salt)`.
    pub leaf_salt: Digest,
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
        row[COL_KEY..COL_KEY + 4].copy_from_slice(&witness.spend_key);
        for i in 0..4 {
            row[COL_ID + i] = claimed_id[i];
        }
        row[COL_BAL] = c_bal;
        row[COL_NONCE] = witness.nonce;
        row[COL_LOWER] = c_lower;
        row[COL_UPPER] = c_upper;
        row[COL_LEAF_SALT..COL_LEAF_SALT + 4].copy_from_slice(&witness.leaf_salt);
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
        debug_assert!(
            level < TREE_DEPTH,
            "place: nivel {} sobre path de {}",
            level,
            TREE_DEPTH
        );
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
                ROW_SALT_LINK => {
                    // EL TERCER MERGE (§117): la hoja se envuelve con el
                    // salt. Digest arrastrado; el rate recibe los CUATRO
                    // limbos del salt (spec §2 — atar solo [8] sería el
                    // bug de §92.2 en su forma nueva).
                    state[4..8].copy_from_slice(&digest);
                    state[8..12].copy_from_slice(&witness.leaf_salt);
                }
                ROW_LEAF_DONE => place(&mut state, &digest, 0),
                ROW_ROOT => {
                    // Derivacion de pk: TITULARIDAD.
                    state[4] = BaseElement::new(SPEND_KEY_DOMAIN);
                    state[8..12].copy_from_slice(&witness.spend_key);
                }
                _ => {
                    let next_cycle = (r + 1) / CYCLE_LENGTH;
                    // Convención única (playbook R2): tramo genérico =
                    // `(CYC_arranque..CYC_fin_de_tramo)`, nivel =
                    // `next_cycle - CYC_arranque`; el arranque lo sombrea
                    // el brazo de `ROW_LEAF_DONE` (nivel 0 explícito).
                    if (CYC_ACC..CYC_PK).contains(&next_cycle) {
                        let level = next_cycle - CYC_ACC;
                        place(&mut state, &digest, level);
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
            rows[(CYC_ACC + level) * CYCLE_LENGTH + p][COL_BIT] = bit;
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
    let id = derive_public_id_wide(witness.spend_key);
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
        // leaf cap (4), leaf dig (4), nonce (1), input (5), pk input (**4**),
        // pk check (4) = 22, grado 1 con ciclo.
        //
        // ⚠️ Eran 19: la clave paso de 1 ranura a 4 —cuatro elementos por UN
        // carril, que es lo que distingue a este circuito (§90)—.
        for _ in 0..22 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }
        // Transporte (**8**) + identidad (4): grado 1 sin ciclo.
        for _ in 0..12 {
            degrees.push(TransitionConstraintDegree::new(1));
        }
        for _ in 0..2 {
            degrees.push(TransitionConstraintDegree::new(2));
        }
        for _ in 0..(3 + NUM_SEGMENTS) {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }
        // La envoltura del salt (12): grado 1 con ciclo — el molde de
        // los enlaces de hoja, gate periódico × expresión lineal.
        for _ in 0..12 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }

        assert_eq!(degrees.len(), NUM_CONSTRAINTS, "cuenta de grados");

        AuditAir {
            // 20 - 3: se retiraron las que fijaban a cero `state[9..12]` en
            // `ROW_PK_START`, que dejaron de ser relleno al ensanchar la
            // clave (§92.2). Son tres y no seis porque hay UN carril.
            context: AirContext::new(trace_info, degrees, 17, options),
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
            link_merkle[(CYC_ACC + level) * CYCLE_LENGTH + 7] = one;
        }
        columns.push(link_merkle);

        let mut link_leaf = vec![zero; TRACE_LENGTH];
        link_leaf[ROW_LEAF_LINK] = one;
        columns.push(link_leaf);

        let mut link_salt = vec![zero; TRACE_LENGTH];
        link_salt[ROW_SALT_LINK] = one;
        columns.push(link_salt);

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
        let link_salt = periodic[P_LINK_SALT];
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

        // EL TERCER MERGE (§117): la envoltura, cosida por `link_salt`.
        // Digest arrastrado y los CUATRO limbos del rate := salt testigo
        // (§138 en los cuatro limbos) — UN carril.
        for i in 0..4 {
            result[C_SALT_CAP + i] = link_salt * next[i];
            result[C_SALT_DIG + i] = link_salt * (next[4 + i] - current[4 + i]);
            result[C_SALT_IN + i] =
                link_salt * (next[8 + i] - current[COL_LEAF_SALT + i]);
        }

        // Entradas de la hoja: identidad completa + saldo.
        for i in 0..4 {
            result[C_INPUT + i] = first_row * (current[4 + i] - current[COL_ID + i]);
        }
        result[C_INPUT + 4] = first_row * (current[8] - current[COL_BAL]);

        // La clave entra en la derivación de pk.
        // ⚠️ Los CUATRO elementos. Un solo carril: no hay `LANE_B` (§92.2).
        for i in 0..4 {
            result[C_PK_INPUT + i] = sel_root * (next[8 + i] - current[COL_KEY + i]);
        }

        // ===== TITULARIDAD =====
        // La pk derivada de la clave coincide con la identidad auditada.
        // Sin la clave del titular no se puede producir la revelación:
        // impide que un tercero fabrique revelaciones sobre cuentas
        // ajenas.
        for i in 0..4 {
            result[C_PK_CHECK + i] = sel_pk_done * (current[4 + i] - current[COL_ID + i]);
        }

        let transport = [
            COL_KEY,
            COL_KEY + 1,
            COL_KEY + 2,
            COL_KEY + 3,
            COL_BAL,
            COL_NONCE,
            COL_LOWER,
            COL_UPPER,
        ];
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
        // ⚠️ Las ranuras 9..12 **ya no son relleno: son la clave** (§92.2).
        //
        // Las ataba a cero cuando `sk` era un elemento. Ahora las fija
        // `C_PK_INPUT` contra `COL_KEY` —constante por `C_TRANSPORT`— y
        // `C_PK_CHECK` exige que la `pk` derivada iguale `COL_ID`. Pasan de
        // estar fijadas a CERO a estarlo a la CLAVE: mas fuerte.
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
    use crate::circuit_settlement::{
        derive_leaf_salt_wide, native_climb, native_leaf, native_leaf_salted,
    };
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
        // ⚠️ Ancha de verdad, no `as_digest(x)`: con relleno de ceros el
        // test pasaria sin ejercitar los tres elementos nuevos (§90.3).
        let key = [
            BaseElement::new(SK),
            BaseElement::new(0xA0D17),
            BaseElement::new(0x0DDBA11),
            BaseElement::new(0x5EA51DE),
        ];
        let id = derive_public_id_wide(key);
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
        // El salt REAL del titular (§117): derivado de la clave, no un
        // literal de juguete — el escenario vive en el mundo envuelto.
        let leaf_salt = derive_leaf_salt_wide(key);
        let root = native_climb(
            native_leaf_salted(id, BaseElement::new(balance), nonce, leaf_salt),
            &path,
        );

        (
            AuditWitness {
                spend_key: key,
                balance,
                nonce,
                leaf_salt,
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

        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| prover.prove(trace)));

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
    /// Un tercero conoce la identidad, el saldo, el nonce, el salt y el camino de
    /// una cuenta ajena, y construye la traza **con la identidad de la
    /// víctima** para que la raíz cuadre. Solo `C_PK_CHECK` puede
    /// detectar que su clave no corresponde.
    #[test]
    fn third_party_cannot_disclose_someone_elses_balance() {
        let (victim, root, victim_id) = scenario(1_000_000);
        let attacker = AuditWitness {
            spend_key: [
                BaseElement::new(0x1337),
                BaseElement::new(0xBADC0DE),
                BaseElement::new(0x0DDBA11),
                BaseElement::new(0x1CEB00DA),
            ],
            balance: victim.balance,
            nonce: victim.nonce,
            // El salt es OBSERVABLE (el secreto es la clave, §117): el
            // ataque sigue apuntando a titularidad, no a pertenencia.
            leaf_salt: victim.leaf_salt,
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

        // ⚠️ **LA BANDA DEBE CONTENER EL SALDO.**
        //
        // Este test usaba `700_000..800_000` con un saldo de 1.000.000: la
        // traza de referencia declaraba algo **falso**, y la restriccion que
        // detecta el saldo fuera de banda saltaba —haciendo su trabajo—.
        //
        // En `--release` el `debug_assert` de `buscar_vacias` no se ejecuta,
        // asi que la herramienta seguia adelante **con una referencia rota**:
        // marcaba como «disparadas» restricciones que ya lo estaban antes de
        // perturbar nada. Su informe para este circuito **no valia**.
        //
        // Lo delato ejecutar la suite sin `--release`. Ver `AUDITORIA.md` §20.
        let (w, root, id) = scenario(1_000_000);
        let trace = build_trace(&w, 900_000, 1_100_000);
        let rows = rows_of(&trace, TRACE_WIDTH, TRACE_LENGTH);

        let air = AuditAir::new(
            TraceInfo::new(TRACE_WIDTH, TRACE_LENGTH),
            pi(root, id, 900_000, 1_100_000),
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

    /// **NATIVO↔CIRCUITO de la envoltura (spec §4, playbook R5) — un
    /// carril, una hoja.**
    #[test]
    fn la_cadena_de_tres_merges_espeja_native_leaf_salted() {
        let (w, _root, id) = scenario(1_000_000);
        let trace = build_trace(&w, 900_000, 1_100_000);

        let sin_sal = native_leaf(id, BaseElement::new(w.balance), w.nonce);
        let con_sal =
            native_leaf_salted(id, BaseElement::new(w.balance), w.nonce, w.leaf_salt);
        for i in 0..4 {
            assert_eq!(
                trace.get(4 + i, ROW_SALT_LINK),
                sin_sal[i],
                "hoja sin envolver"
            );
            assert_eq!(
                trace.get(4 + i, ROW_LEAF_DONE),
                con_sal[i],
                "hoja envuelta"
            );
        }
    }

    /// **MUTACIÓN OBLIGATORIA (a) de la spec §4.** Veneno = honesto + 1.
    #[test]
    fn mutacion_a_un_limbo_del_salt_testigo_alterado_se_rechaza() {
        let (w, root, id) = scenario(1_000_000);
        let mut trace = build_trace(&w, 900_000, 1_100_000);

        let veneno = trace.get(COL_LEAF_SALT + 2, ROW_SALT_LINK) + BaseElement::ONE;
        trace.set(COL_LEAF_SALT + 2, ROW_SALT_LINK, veneno);

        let prover = AuditProver::new(default_options());
        let verifica = {
            let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                || prover.prove(trace)));
            match r {
                Err(_) => false,        // panic al generar -> no verifica
                Ok(Err(_)) => false,    // prove Err
                Ok(Ok(proof)) => {
                    let min_opts = AcceptableOptions::OptionSet(vec![default_options()]);
                    verify::<AuditAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
                        proof, pi(root, id, 900_000, 1_100_000), &min_opts,
                    ).is_ok()
                }
            }
        };
        assert!(
            !verifica,
            "un limbo del salt testigo alterado DEBE rechazar (C_SALT_IN); \
             si verifica, la envoltura es decorativa"
        );
    }

    /// **MUTACIÓN OBLIGATORIA (b) de la spec §4.** AMBAS mitades del
    /// estado siguiente (bit-agnóstico); `C_PLACE` dispara.
    #[test]
    fn mutacion_b_la_hoja_sin_envolver_no_entra_al_camino() {
        let (w, root, id) = scenario(1_000_000);
        let mut trace = build_trace(&w, 900_000, 1_100_000);

        let sin_sal = native_leaf(id, BaseElement::new(w.balance), w.nonce);
        for i in 0..4 {
            trace.set(4 + i, ROW_LEAF_DONE + 1, sin_sal[i]);
            trace.set(8 + i, ROW_LEAF_DONE + 1, sin_sal[i]);
        }

        let prover = AuditProver::new(default_options());
        let verifica = {
            let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                || prover.prove(trace)));
            match r {
                Err(_) => false,        // panic al generar -> no verifica
                Ok(Err(_)) => false,    // prove Err
                Ok(Ok(proof)) => {
                    let min_opts = AcceptableOptions::OptionSet(vec![default_options()]);
                    verify::<AuditAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
                        proof, pi(root, id, 900_000, 1_100_000), &min_opts,
                    ).is_ok()
                }
            }
        };
        assert!(
            !verifica,
            "la hoja sin envolver NO debe entrar al camino (C_PLACE en el \
             enlace corrido); si verifica, el corrimiento no protege nada"
        );
    }
}
