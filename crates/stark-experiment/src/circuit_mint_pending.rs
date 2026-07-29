//! **Emisión a un pendiente**: crear dinero sin tocar ninguna cuenta.
//!
//! ## Por qué existe
//!
//! La emisión clásica acredita una cuenta ajena, así que **necesita su
//! saldo** para calcular la hoja nueva. Eso obliga a que el operador —o
//! los custodios— lo conozcan, y es justo lo que el modelo por compromisos
//! elimina.
//!
//! Aquí los custodios **no acreditan nada**: crean un compromiso pendiente
//! atado a la identidad del receptor, que este reclama con `circuit_claim`.
//!
//! ```text
//! Custodios                       Titular
//! · suben el suministro           · demuestra que el pendiente es suyo
//! · crean el pendiente            · acredita su cuenta
//!   P = H(H(id, aleatorio), imp)  · lo consume
//! ```
//!
//! ## Qué demuestra
//!
//! 1. **Dos custodios distintos** autorizan, con índices crecientes.
//! 2. El suministro sube **exactamente** el importe.
//! 3. **El tope de emisión se comprueba en el circuito.** Un segmento de
//!    64 filas descompone `tope − suministro_nuevo` en **63 bits**. Si el
//!    suministro se pasara, esa resta envuelve en el campo y da un valor de
//!    64 bits que **no cabe**: no hay descomposición posible.
//!
//!    ⚠️ El margen es de **un solo bit**. Ver `AUDITORIA.md` §13.

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

use crate::merkle::{native_merge, Digest, MerklePath, TREE_DEPTH};
use crate::circuit_threshold::CustodianPath;
use crate::rescue_hash::{apply_sbox, NUM_ROUNDS, STATE_WIDTH};

/// Dominio de derivación de identidades de custodio. **Distinto del de
/// cuentas y del de emisor**: una clave de gasto no puede hacerse pasar
/// por custodio.
pub const CUSTODIAN_DOMAIN: u64 = 0x43555354; // "CUST"

pub const CYCLE_LENGTH: usize = 8;
/// 512 filas. La autorización acaba en la 39; el compromiso y la
/// inserción del pendiente llegan a la 319.
pub const TRACE_LENGTH: usize = 512;
/// Profundidad del árbol de custodios: hasta 16.
pub const CUSTODIAN_DEPTH: usize = 4;
/// Bits de las comprobaciones de rango. Los índices son pequeños.
pub const SEGMENT_LENGTH: usize = 8;
/// Segmentos: índice A, índice B, y `B − A − 1`.
pub const NUM_SEGMENTS: usize = 3;

// ===== Columnas =====
const LANE_B: usize = STATE_WIDTH; // 12
/// Bits de dirección: **uno por carril**, porque los custodios están en
/// posiciones distintas del árbol y suben por caminos distintos.
const COL_BIT_A: usize = 24;
const COL_BIT_B: usize = 25;
const COL_KEY_A: usize = 26;
const COL_KEY_B: usize = 27;
const COL_IDX_A: usize = 28;
const COL_IDX_B: usize = 29;
/// Acumuladores que reconstruyen el índice desde los bits del camino.
const COL_ACC_A: usize = 30;
const COL_ACC_B: usize = 31;
const COL_SBIT: usize = 32;
const COL_SACC: usize = 33;
/// Suministro antes y después. Un emisión lo sube exactamente el importe.
const COL_SUPPLY_OLD: usize = 34;
const COL_SUPPLY_NEW: usize = 35;
/// Tope inmutable de emisión.
const COL_MAX_SUPPLY: usize = 36;
/// Importe emitido.
const COL_AMOUNT: usize = 37;
/// **Identidad pública del receptor.** Funciona como dirección.
const COL_R_ID: usize = 38; // 38..42
/// Aleatorio que ciega el compromiso.
const COL_SALT: usize = 42; // 42..46
/// Bit de dirección en el árbol de pendientes.
const COL_PBIT: usize = 46;
/// Bit de la descomposicion del margen `tope - suministro_nuevo`.
const COL_CBIT: usize = 47;
/// Acumulador de Horner de esa descomposicion.
const COL_CACC: usize = 48;
pub const TRACE_WIDTH: usize = 49;

/// Primera fila del segmento que comprueba el tope de emision.
///
/// Va DESPUES de `ROW_PENDING_ROOT` (311), en filas que estaban vacias.
const CAP_START: usize = 320;
/// Longitud del segmento: 64 filas dan **63 bits**, no 64.
///
/// Ese margen de un bit es lo que hace que el tope se imponga: si el
/// suministro se pasara, la resta envuelve y da un valor de 64 bits que
/// **no cabe** en 63. Ver `AUDITORIA.md` §13.
const CAP_LENGTH: usize = 64;

// ===== Filas =====
/// Última fila activa: raíz del conjunto de custodios.
const ROW_ROOT: usize = 39;
/// Compromiso interno del pendiente: `H(identidad_receptor, aleatorio)`.
const ROW_PEND_INNER: usize = 47;
/// El pendiente completo: `H(interno, importe)`.
const ROW_PENDING_ENTRY: usize = 55;
/// Raíz tras insertarlo. Ciclos 8..39, filas 56..319.
/// Raíz tras insertarlo: **entrada + 256**, como en los demás circuitos.
///
/// Una versión anterior ponía 319, un ciclo de más. El desfase hacía que
/// la restricción de entrada buscara el bit de dirección donde no estaba.
const ROW_PENDING_ROOT: usize = 311;

// ===== Restricciones =====
const C_HASH_A: usize = 0;
const C_HASH_B: usize = C_HASH_A + STATE_WIDTH;
const C_CAP_A: usize = C_HASH_B + STATE_WIDTH; // 4
const C_CAP_B: usize = C_CAP_A + 4;
const C_PLACE_A: usize = C_CAP_B + 4; // 4
const C_PLACE_B: usize = C_PLACE_A + 4;
const C_BIT_BOOL: usize = C_PLACE_B + 4; // 2
const C_KEY_INPUT: usize = C_BIT_BOOL + 2; // 2
/// El acumulador reconstruye el índice desde los bits del camino.
const C_ACC: usize = C_KEY_INPUT + 2; // 2
/// El acumulador final coincide con el índice declarado.
const C_ACC_FINAL: usize = C_ACC + 2; // 2
const C_TRANSPORT: usize = C_ACC_FINAL + 2; // 4
const C_SBIT_BOOL: usize = C_TRANSPORT + 4; // 2
const C_FIRST_S: usize = C_SBIT_BOOL + 2; // 2
const C_HORNER: usize = C_FIRST_S + 2; // 1
const C_SEG_LINK: usize = C_HORNER + 1; // NUM_SEGMENTS
/// **El suministro sube exactamente el importe.**
const C_SUPPLY: usize = C_SEG_LINK + NUM_SEGMENTS; // 1
/// Compromiso interno: capacidad (4), identidad (4), aleatorio (4).
const C_PEND_IN: usize = C_SUPPLY + 1; // 12
/// Compromiso completo: digest (4) e importe (1).
const C_PEND_VAL: usize = C_PEND_IN + 12; // 5
/// Capacidad a cero en la subida.
const C_PEND_CAP: usize = C_PEND_VAL + 5; // 8
/// **LA POSICIÓN ESTABA LIBRE**: carril A con cero, B con el compromiso.
const C_PEND_ENTRY_A: usize = C_PEND_CAP + 8; // 4
const C_PEND_ENTRY_B: usize = C_PEND_ENTRY_A + 4; // 4
const C_PEND_PLACE: usize = C_PEND_ENTRY_B + 4; // 8
const C_PEND_SIBLING: usize = C_PEND_PLACE + 8; // 4
const C_PBIT_BOOL: usize = C_PEND_SIBLING + 4; // 1
/// Transporte de las columnas constantes nuevas.
const C_TRANSPORT_NEW: usize = C_PBIT_BOOL + 1; // 12
/// Los bits del margen son booleanos.
const C_CBIT_BOOL: usize = C_TRANSPORT_NEW + 12; // 2
/// El acumulador arranca en cero.
const C_CAP_FIRST: usize = C_CBIT_BOOL + 2; // 2
/// Horner: cada fila duplica y suma el bit siguiente.
const C_CAP_HORNER: usize = C_CAP_FIRST + 2; // 1
/// **El acumulado final ES `tope - suministro_nuevo`.**
///
/// Esta es la restriccion que impone el tope.
const C_CAP_LINK: usize = C_CAP_HORNER + 1; // 1
const NUM_CONSTRAINTS: usize = C_CAP_LINK + 1;

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
/// Fila del compromiso interno.
const P_PEND_IN: usize = P_SEG_LINK + NUM_SEGMENTS;
/// Fila del compromiso completo.
const P_PEND_VAL: usize = P_PEND_IN + 1;
/// Fila que entra al árbol.
const P_PEND_ENTRY: usize = P_PEND_VAL + 1;
/// Enlaces de la subida.
const P_PEND_LINK: usize = P_PEND_ENTRY + 1;
/// Arranque del segmento del tope.
const P_CAP_FIRST: usize = P_PEND_LINK + 1;
/// Filas donde el acumulador del tope avanza.
const P_CAP_CONT: usize = P_CAP_FIRST + 1;
/// Fila donde se compara el acumulado con el margen declarado.
const P_CAP_LINK: usize = P_CAP_CONT + 1;

type Blake3 = Blake3_256<BaseElement>;

fn as_digest(x: BaseElement) -> Digest {
    [x, BaseElement::ZERO, BaseElement::ZERO, BaseElement::ZERO]
}

/// Identidad pública de un custodio desde su clave.
pub fn derive_custodian_id(key: BaseElement) -> Digest {
    native_merge(
        as_digest(BaseElement::new(CUSTODIAN_DOMAIN)),
        as_digest(key),
    )
}


/// Construye el conjunto de custodios y devuelve su raíz junto con los
/// caminos de cada uno.
///
/// El conjunto es pequeño (16 posiciones), así que se materializa entero.
pub fn build_custodian_set(keys: &[BaseElement]) -> (Digest, Vec<CustodianPath>) {
    let size = 1usize << CUSTODIAN_DEPTH;
    assert!(keys.len() <= size, "demasiados custodios");

    let empty: Digest = [BaseElement::ZERO; 4];
    let mut leaves: Vec<Digest> = keys.iter().map(|k| derive_custodian_id(*k)).collect();
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

/// Sube una identidad hasta la raíz del conjunto, de forma nativa.
pub fn climb(leaf: Digest, path: &CustodianPath) -> Digest {
    let mut current = leaf;
    for level in 0..CUSTODIAN_DEPTH {
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

/// Construye la traza de una autorización 2-de-N.
///
/// `index_a` debe ser estrictamente menor que `index_b`. Los tests pasan
/// valores que violan esa condición para comprobar que se rechaza.
pub fn build_trace(
    key_a: BaseElement,
    index_a: u64,
    path_a: &CustodianPath,
    key_b: BaseElement,
    index_b: u64,
    path_b: &CustodianPath,
    // Suministro antes y despues: la restriccion exige que la diferencia
    // sea exactamente el importe.
    supply_old: u64,
    supply_delta: u64,
    max_supply: u64,
    amount: u64,
    // **Identidad publica del receptor.** Funciona como direccion: los
    // custodios la obtienen de el, no de la capa.
    //
    // **No necesitan su saldo**, que es lo que distingue esta emision de
    // la clasica.
    receiver_id: Digest,
    salt: Digest,
    pending_path: &MerklePath,
) -> TraceTable<BaseElement> {
    let zero = BaseElement::ZERO;
    let c_amount = BaseElement::new(amount);
    let mut rows: Vec<Vec<BaseElement>> = vec![vec![zero; TRACE_WIDTH]; TRACE_LENGTH];

    for row in rows.iter_mut() {
        row[COL_SUPPLY_OLD] = BaseElement::new(supply_old);
        row[COL_SUPPLY_NEW] = BaseElement::new(supply_old + supply_delta);
        row[COL_MAX_SUPPLY] = BaseElement::new(max_supply);
        row[COL_AMOUNT] = c_amount;
        row[COL_R_ID..COL_R_ID + 4].copy_from_slice(&receiver_id);
        row[COL_SALT..COL_SALT + 4].copy_from_slice(&salt);
        row[COL_KEY_A] = key_a;
        row[COL_KEY_B] = key_b;
        row[COL_IDX_A] = BaseElement::new(index_a);
        row[COL_IDX_B] = BaseElement::new(index_b);
    }

    // Rangos: los dos indices y la diferencia menos uno.
    //
    // Si los indices fueran iguales o estuvieran en orden inverso, la
    // resta daria la vuelta en el campo y no cabria en 8 bits.
    let diff = BaseElement::new(index_b) - BaseElement::new(index_a) - BaseElement::ONE;
    let segment_values = [index_a, index_b, diff.as_int()];
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

    let place_pending = |state: &mut [BaseElement; STATE_WIDTH], digest: &Digest, level: usize| {
        if pending_path.is_right[level] {
            state[4..8].copy_from_slice(&pending_path.siblings[level]);
            state[8..12].copy_from_slice(digest);
        } else {
            state[4..8].copy_from_slice(digest);
            state[8..12].copy_from_slice(&pending_path.siblings[level]);
        }
    };

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

    // Ciclo 0: derivacion de las identidades.
    let mut state_a = [zero; STATE_WIDTH];
    let mut state_b = [zero; STATE_WIDTH];
    state_a[4] = BaseElement::new(CUSTODIAN_DOMAIN);
    state_a[8] = key_a;
    state_b[4] = BaseElement::new(CUSTODIAN_DOMAIN);
    state_b[8] = key_b;

    rows[0][..STATE_WIDTH].copy_from_slice(&state_a);
    rows[0][LANE_B..LANE_B + STATE_WIDTH].copy_from_slice(&state_b);

    let mut acc_a = zero;
    let mut acc_b = zero;

    for r in 0..ROW_PENDING_ROOT {
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

            // ===== FASES DEL PENDIENTE =====
            if r == ROW_ROOT {
                // COMPROMISO INTERNO: H(identidad_receptor, aleatorio).
                state_a[4..8].copy_from_slice(&receiver_id);
                state_a[8..12].copy_from_slice(&salt);
                state_b.copy_from_slice(&state_a);
            } else if r == ROW_PEND_INNER {
                // EL PENDIENTE: H(interno, importe).
                state_a[4..8].copy_from_slice(&digest_a);
                state_a[8] = c_amount;
                state_b.copy_from_slice(&state_a);
            } else if r == ROW_PENDING_ENTRY {
                // ENTRADA: carril A con hoja CERO —la posicion estaba
                // libre— y carril B con el compromiso.
                let libre: Digest = [zero; 4];
                place_pending(&mut state_a, &libre, 0);
                place_pending(&mut state_b, &digest_b, 0);
            } else if (7..38).contains(&(r / CYCLE_LENGTH)) {
                // El nivel 0 lo coloca la ENTRADA, en el ciclo 7. Aqui van
                // los niveles 1..31, en los ciclos 7..37.
                let nivel = r / CYCLE_LENGTH - 6;
                place_pending(&mut state_a, &digest_a, nivel);
                place_pending(&mut state_b, &digest_b, nivel);
            } else if level < CUSTODIAN_DEPTH {
                place(&mut state_a, &digest_a, path_a, level);
                place(&mut state_b, &digest_b, path_b, level);
                // El acumulador incorpora el bit de ESTE nivel.
                let p = BaseElement::new(1u64 << level);
                if path_a.is_right[level] {
                    acc_a += p;
                }
                if path_b.is_right[level] {
                    acc_b += p;
                }
            }
        }
        rows[r + 1][..STATE_WIDTH].copy_from_slice(&state_a);
        rows[r + 1][LANE_B..LANE_B + STATE_WIDTH].copy_from_slice(&state_b);
        rows[r + 1][COL_ACC_A] = acc_a;
        rows[r + 1][COL_ACC_B] = acc_b;
    }

    // Los acumuladores permanecen tras la ultima fila activa.
    for r in ROW_ROOT..TRACE_LENGTH {
        rows[r][COL_ACC_A] = acc_a;
        rows[r][COL_ACC_B] = acc_b;
    }

    // Bits de direccion, constantes dentro de cada ciclo de subida.
    for level in 0..CUSTODIAN_DEPTH {
        let ba = if path_a.is_right[level] {
            BaseElement::ONE
        } else {
            zero
        };
        let bb = if path_b.is_right[level] {
            BaseElement::ONE
        } else {
            zero
        };
        for p in 0..CYCLE_LENGTH {
            rows[(1 + level) * CYCLE_LENGTH + p][COL_BIT_A] = ba;
            rows[(1 + level) * CYCLE_LENGTH + p][COL_BIT_B] = bb;
        }
    }

    // Bits de direccion del camino de pendientes: ciclos 8..39.
    for level in 0..TREE_DEPTH {
        let bit = if pending_path.is_right[level] {
            BaseElement::ONE
        } else {
            zero
        };
        for p in 0..CYCLE_LENGTH {
            rows[(7 + level) * CYCLE_LENGTH + p][COL_PBIT] = bit;
        }
    }

    // ===== SEGMENTO DEL TOPE DE EMISION =====
    //
    // Descompone `tope - suministro_nuevo` en 63 bits. Si el suministro se
    // pasara del tope, esa resta ENVUELVE en el campo y da un valor de 64
    // bits que no cabe en 63: la descomposicion seria imposible y la
    // prueba no se generaria.
    //
    // El margen de un bit es lo que impone el tope. Ver `AUDITORIA.md` §13.
    let supply_new = supply_old + supply_delta;
    let margen = max_supply.wrapping_sub(supply_new);
    for p in 1..CAP_LENGTH {
        // p=1 toma el bit 62; p=63, el bit 0.
        let bit = (margen >> (CAP_LENGTH - 1 - p)) & 1;
        rows[CAP_START + p][COL_CBIT] = BaseElement::new(bit);
        rows[CAP_START + p][COL_CACC] =
            rows[CAP_START + p - 1][COL_CACC] + rows[CAP_START + p - 1][COL_CACC]
                + BaseElement::new(bit);
    }

    let mut trace = TraceTable::new(TRACE_WIDTH, TRACE_LENGTH);
    trace.fill(
        |s| s.copy_from_slice(&rows[0]),
        |step, s| s.copy_from_slice(&rows[step + 1]),
    );
    trace
}

/// Input público: la raíz del conjunto de custodios autorizados.
///
/// **Los índices y las claves de quienes firman son privados**: se sabe
/// que dos custodios distintos del conjunto autorizaron, pero no cuáles.
#[derive(Clone, Debug)]
pub struct MintPendingPublicInputs {
    pub custodian_set_root: Digest,
    /// Suministro antes y después. La diferencia es el importe emitido.
    pub supply_old: BaseElement,
    pub supply_new: BaseElement,
    /// Tope inmutable.
    pub max_supply: BaseElement,
    pub amount: BaseElement,
    /// **Árbol de pendientes ANTES.** La posición estaba libre.
    pub pending_root_old: Digest,
    /// **Y DESPUÉS**, con el compromiso dentro.
    pub pending_root_new: Digest,
}

impl ToElements<BaseElement> for MintPendingPublicInputs {
    fn to_elements(&self) -> Vec<BaseElement> {
        let mut out = self.custodian_set_root.to_vec();
        out.push(self.supply_old);
        out.push(self.supply_new);
        out.push(self.max_supply);
        out.push(self.amount);
        out.extend_from_slice(&self.pending_root_old);
        out.extend_from_slice(&self.pending_root_new);
        out
    }
}

pub struct MintPendingAir {
    context: AirContext<BaseElement>,
    pub_inputs: MintPendingPublicInputs,
}

impl Air for MintPendingAir {
    type BaseField = BaseElement;
    type PublicInputs = MintPendingPublicInputs;

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
        // Colocacion (8): grado 2.
        for _ in 0..8 {
            degrees.push(TransitionConstraintDegree::with_cycles(2, full.clone()));
        }
        // Bits booleanos (2): grado 2 sin ciclo.
        for _ in 0..2 {
            degrees.push(TransitionConstraintDegree::new(2));
        }
        // Claves (2): un selector periódico.
        for _ in 0..2 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }
        // Acumulador (2): **DOS columnas periódicas**, el selector de
        // enlace y la potencia de dos. `with_cycles` recibe una longitud
        // por cada periódica que interviene; declarar una sola cuando hay
        // dos hace que winterfell calcule el doble de grado.
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
        // Transporte (4): grado 1 sin ciclo.
        for _ in 0..4 {
            degrees.push(TransitionConstraintDegree::new(1));
        }
        // Solvencia: bits booleanos (2) grado 2.
        for _ in 0..2 {
            degrees.push(TransitionConstraintDegree::new(2));
        }
        for _ in 0..(3 + NUM_SEGMENTS) {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }

        // --- Suministro y pendiente ---
        // Suministro (1): grado 1 sin ciclo.
        degrees.push(TransitionConstraintDegree::new(1));
        // Compromiso interno (12) y completo (5): grado 1 con ciclo.
        for _ in 0..17 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }
        // Capacidad de la subida (8): grado 1 con ciclo.
        for _ in 0..8 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }
        // Entradas (8), colocacion (8), hermano (4): grado 2 con ciclo.
        for _ in 0..20 {
            degrees.push(TransitionConstraintDegree::with_cycles(2, full.clone()));
        }
        // Bit booleano (1) y transporte (12): sin ciclo.
        degrees.push(TransitionConstraintDegree::new(2));
        for _ in 0..12 {
            degrees.push(TransitionConstraintDegree::new(1));
        }
        // Bits del margen (2): grado 2 sin ciclo.
        for _ in 0..2 {
            degrees.push(TransitionConstraintDegree::new(2));
        }
        // Arranque (2), Horner (1) y enlace (1) del segmento del tope.
        //
        // ⚠️ El ciclo es `full`, NO `CAP_LENGTH`: aqui el segmento es un
        // bloque unico en una traza de 512 filas, no la llena. Las filas a
        // cero fuera del bloque **rompen la periodicidad**, asi que
        // declararlo periodico de periodo 64 seria falso.
        //
        // En `circuit_mint` si es periodico porque sus 8 segmentos de 64
        // filas llenan la traza entera.
        for _ in 0..4 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }

        assert_eq!(degrees.len(), NUM_CONSTRAINTS, "cuenta de grados");

        MintPendingAir {
            context: AirContext::new(trace_info, degrees, 38, options),
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
        for r in 0..=ROW_PENDING_ROOT {
            if r % CYCLE_LENGTH < NUM_ROUNDS {
                hash_flag[r] = one;
            }
        }
        columns.push(hash_flag);

        for ark in [true, false] {
            for i in 0..STATE_WIDTH {
                let mut col = vec![zero; TRACE_LENGTH];
                for r in 0..=ROW_PENDING_ROOT {
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

        // Enlaces que colocan en el arbol: uno por nivel.
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

        // Compromiso interno del pendiente: una sola fila.
        let mut pend_in = vec![zero; TRACE_LENGTH];
        pend_in[ROW_ROOT] = one;
        columns.push(pend_in);

        // Compromiso completo.
        let mut pend_val = vec![zero; TRACE_LENGTH];
        pend_val[ROW_PEND_INNER] = one;
        columns.push(pend_val);

        // Entrada al arbol de pendientes.
        let mut pend_entry = vec![zero; TRACE_LENGTH];
        pend_entry[ROW_PENDING_ENTRY] = one;
        columns.push(pend_entry);

        // Enlaces de la subida: uno por nivel a partir del primero.
        let mut pend_link = vec![zero; TRACE_LENGTH];
        for level in 0..TREE_DEPTH - 1 {
            pend_link[(7 + level) * CYCLE_LENGTH + 7] = one;
        }
        columns.push(pend_link);

        // ===== SEGMENTO DEL TOPE =====
        //
        // `cont` marca CAP_LENGTH-1 = 63 transiciones, no 64. De ese unico
        // bit de margen depende que el tope se imponga.
        let mut cap_first = vec![zero; TRACE_LENGTH];
        let mut cap_cont = vec![zero; TRACE_LENGTH];
        let mut cap_link = vec![zero; TRACE_LENGTH];
        cap_first[CAP_START] = one;
        for p in 0..CAP_LENGTH - 1 {
            cap_cont[CAP_START + p] = one;
        }
        cap_link[CAP_START + CAP_LENGTH - 2] = one;
        columns.push(cap_first);
        columns.push(cap_cont);
        columns.push(cap_link);

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

        // Cada carril usa SU bit: los custodios están en posiciones
        // distintas y suben por caminos distintos.
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

        // Las claves entran en la derivación de identidad.
        result[C_KEY_INPUT] = first_row * (current[8] - current[COL_KEY_A]);
        result[C_KEY_INPUT + 1] = first_row * (current[LANE_B + 8] - current[COL_KEY_B]);

        // ===== EL ACUMULADOR ATA EL ÍNDICE AL CAMINO =====
        // Sin esto, el índice sería un número declarado sin relación con
        // la posición realmente demostrada, y la comprobación de orden
        // estricto no valdría nada.
        result[C_ACC] = tree_link * (next[COL_ACC_A] - (current[COL_ACC_A] + bit_a * pow2));
        result[C_ACC + 1] = tree_link * (next[COL_ACC_B] - (current[COL_ACC_B] + bit_b * pow2));

        // El acumulado final es el índice declarado.
        result[C_ACC_FINAL] = sel_root * (current[COL_ACC_A] - current[COL_IDX_A]);
        result[C_ACC_FINAL + 1] = sel_root * (current[COL_ACC_B] - current[COL_IDX_B]);

        let transport = [COL_KEY_A, COL_KEY_B, COL_IDX_A, COL_IDX_B];
        for (k, col) in transport.iter().enumerate() {
            result[C_TRANSPORT + k] = next[*col] - current[*col];
        }

        // ===== ORDEN ESTRICTO: indice_b > indice_a =====
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

        // ===== EL SUMINISTRO SUBE EXACTAMENTE EL IMPORTE =====
        result[C_SUPPLY] =
            current[COL_SUPPLY_NEW] - (current[COL_SUPPLY_OLD] + current[COL_AMOUNT]);

        // ===== EL PENDIENTE =====
        //
        // Lo que distingue esta emision de la clasica esta aqui: el
        // compromiso se forma con la IDENTIDAD del receptor. **Ninguna
        // columna contiene su saldo, ni podria.**
        let pend_in = periodic[P_PEND_IN];
        let pend_val = periodic[P_PEND_VAL];
        let pend_entry = periodic[P_PEND_ENTRY];
        let pend_link = periodic[P_PEND_LINK];
        let pbit = next[COL_PBIT];
        let pend_any = pend_entry + pend_link;

        for i in 0..4 {
            // Compromiso interno: capacidad a cero, y entran identidad y
            // aleatorio. **Dos restricciones separadas, no su suma**: si se
            // sumaran, un exceso en una compensaria un defecto en la otra.
            result[C_PEND_IN + i] = pend_in * next[i];
            result[C_PEND_IN + 4 + i] = pend_in * (next[4 + i] - current[COL_R_ID + i]);
            result[C_PEND_IN + 8 + i] = pend_in * (next[8 + i] - current[COL_SALT + i]);

            // Compromiso completo: el digest interno.
            result[C_PEND_VAL + i] = pend_val * (next[4 + i] - current[4 + i]);

            // Subida al arbol.
            result[C_PEND_CAP + i] = pend_any * next[i];
            result[C_PEND_CAP + 4 + i] = pend_any * next[LANE_B + i];

            result[C_PEND_ENTRY_A + i] =
                pend_entry * ((E::ONE - pbit) * next[4 + i] + pbit * next[8 + i]);
            result[C_PEND_ENTRY_B + i] = pend_entry
                * ((E::ONE - pbit) * (next[LANE_B + 4 + i] - current[LANE_B + 4 + i])
                    + pbit * (next[LANE_B + 8 + i] - current[LANE_B + 4 + i]));

            let da = current[4 + i];
            let db = current[LANE_B + 4 + i];
            result[C_PEND_PLACE + i] =
                pend_link * ((E::ONE - pbit) * (next[4 + i] - da) + pbit * (next[8 + i] - da));
            result[C_PEND_PLACE + 4 + i] = pend_link
                * ((E::ONE - pbit) * (next[LANE_B + 4 + i] - db)
                    + pbit * (next[LANE_B + 8 + i] - db));

            let sib_a = (E::ONE - pbit) * next[8 + i] + pbit * next[4 + i];
            let sib_b = (E::ONE - pbit) * next[LANE_B + 8 + i] + pbit * next[LANE_B + 4 + i];
            result[C_PEND_SIBLING + i] = pend_link * (sib_a - sib_b);
        }
        result[C_PEND_VAL + 4] = pend_val * (next[8] - current[COL_AMOUNT]);
        result[C_PBIT_BOOL] = current[COL_PBIT] * (current[COL_PBIT] - E::ONE);

        // ===== TRANSPORTE DE LAS COLUMNAS CONSTANTES =====
        //
        // Si la identidad del receptor o el aleatorio variaran entre filas,
        // el compromiso no seria el declarado.
        for (k, col) in [COL_SUPPLY_OLD, COL_SUPPLY_NEW, COL_MAX_SUPPLY, COL_AMOUNT]
            .iter()
            .enumerate()
        {
            result[C_TRANSPORT_NEW + k] = next[*col] - current[*col];
        }
        for i in 0..4 {
            result[C_TRANSPORT_NEW + 4 + i] = next[COL_R_ID + i] - current[COL_R_ID + i];
            result[C_TRANSPORT_NEW + 8 + i] = next[COL_SALT + i] - current[COL_SALT + i];
        }

        // ===== EL TOPE DE EMISION =====
        //
        // El acumulador reconstruye `tope - suministro_nuevo` desde 63
        // bits. Si el suministro se pasara, esa resta envuelve y da un
        // valor de 64 bits: no cabe, y no hay descomposicion que satisfaga
        // el enlace final.
        let cbit_cur = current[COL_CBIT];
        let cbit_next = next[COL_CBIT];
        let cacc_cur = current[COL_CACC];
        let cacc_next = next[COL_CACC];
        let cap_first = periodic[P_CAP_FIRST];
        let cap_cont = periodic[P_CAP_CONT];
        let cap_link = periodic[P_CAP_LINK];

        result[C_CBIT_BOOL] = cbit_cur * (cbit_cur - E::ONE);
        result[C_CBIT_BOOL + 1] = cbit_next * (cbit_next - E::ONE);
        result[C_CAP_FIRST] = cap_first * cbit_cur;
        result[C_CAP_FIRST + 1] = cap_first * cacc_cur;
        result[C_CAP_HORNER] = cap_cont * (cacc_next - (cacc_cur + cacc_cur + cbit_next));
        result[C_CAP_LINK] =
            cap_link * (cacc_next - (current[COL_MAX_SUPPLY] - current[COL_SUPPLY_NEW]));
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        let zero = BaseElement::ZERO;
        let mut a = Vec::with_capacity(38);

        // Fila 0: capacidad, dominio anclado y relleno.
        for i in 0..4 {
            a.push(Assertion::single(i, 0, zero));
            a.push(Assertion::single(LANE_B + i, 0, zero));
        }
        a.push(Assertion::single(
            4,
            0,
            BaseElement::new(CUSTODIAN_DOMAIN),
        ));
        a.push(Assertion::single(
            LANE_B + 4,
            0,
            BaseElement::new(CUSTODIAN_DOMAIN),
        ));
        for i in 5..8 {
            a.push(Assertion::single(i, 0, zero));
            a.push(Assertion::single(LANE_B + i, 0, zero));
        }
        // Acumuladores a cero.
        a.push(Assertion::single(COL_ACC_A, 0, zero));
        a.push(Assertion::single(COL_ACC_B, 0, zero));
        // **Los dos carriles llegan a la MISMA raíz**: ambos custodios
        // pertenecen al conjunto autorizado.
        for i in 0..4 {
            a.push(Assertion::single(
                4 + i,
                ROW_ROOT,
                self.pub_inputs.custodian_set_root[i],
            ));
            a.push(Assertion::single(
                LANE_B + 4 + i,
                ROW_ROOT,
                self.pub_inputs.custodian_set_root[i],
            ));
        }

        // Suministro, tope e importe: constantes declaradas.
        a.push(Assertion::single(COL_SUPPLY_OLD, 0, self.pub_inputs.supply_old));
        a.push(Assertion::single(COL_SUPPLY_NEW, 0, self.pub_inputs.supply_new));
        a.push(Assertion::single(COL_MAX_SUPPLY, 0, self.pub_inputs.max_supply));
        a.push(Assertion::single(COL_AMOUNT, 0, self.pub_inputs.amount));

        // **Las raíces del árbol de pendientes**: antes libre, después con
        // el compromiso dentro.
        for i in 0..4 {
            a.push(Assertion::single(
                4 + i,
                ROW_PENDING_ROOT,
                self.pub_inputs.pending_root_old[i],
            ));
            a.push(Assertion::single(
                LANE_B + 4 + i,
                ROW_PENDING_ROOT,
                self.pub_inputs.pending_root_new[i],
            ));
        }

        a
    }
}

pub struct MintPendingProver {
    options: ProofOptions,
}

impl MintPendingProver {
    pub fn new(options: ProofOptions) -> Self {
        Self { options }
    }
}

impl Prover for MintPendingProver {
    type BaseField = BaseElement;
    type Air = MintPendingAir;
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

    fn get_pub_inputs(&self, trace: &Self::Trace) -> MintPendingPublicInputs {
        MintPendingPublicInputs {
            custodian_set_root: [
                trace.get(4, ROW_ROOT),
                trace.get(5, ROW_ROOT),
                trace.get(6, ROW_ROOT),
                trace.get(7, ROW_ROOT),
            ],
            supply_old: trace.get(COL_SUPPLY_OLD, 0),
            supply_new: trace.get(COL_SUPPLY_NEW, 0),
            max_supply: trace.get(COL_MAX_SUPPLY, 0),
            amount: trace.get(COL_AMOUNT, 0),
            pending_root_old: [
                trace.get(4, ROW_PENDING_ROOT),
                trace.get(5, ROW_PENDING_ROOT),
                trace.get(6, ROW_PENDING_ROOT),
                trace.get(7, ROW_PENDING_ROOT),
            ],
            pending_root_new: [
                trace.get(LANE_B + 4, ROW_PENDING_ROOT),
                trace.get(LANE_B + 5, ROW_PENDING_ROOT),
                trace.get(LANE_B + 6, ROW_PENDING_ROOT),
                trace.get(LANE_B + 7, ROW_PENDING_ROOT),
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
    use crate::circuit_settlement::derive_public_id;
    use winterfell::{verify, AcceptableOptions, BatchingMethod, FieldExtension};

    const SUPPLY_OLD: u64 = 10_000_000;
    const MAX_SUPPLY: u64 = 100_000_000;
    const AMOUNT: u64 = 250_000;

    /// Camino del árbol de pendientes con la posición libre.
    ///
    /// Direcciones **mixtas**: con todas iguales la traza degenera y el
    /// test pasaría sin comprobar el caso general.
    fn pending_path() -> MerklePath {
        let mut empty = vec![[BaseElement::ZERO; 4]];
        for k in 1..=TREE_DEPTH {
            let prev = empty[k - 1];
            empty.push(native_merge(prev, prev));
        }
        MerklePath {
            siblings: (0..TREE_DEPTH).map(|l| empty[l]).collect(),
            is_right: (0..TREE_DEPTH).map(|l| l % 3 == 0).collect(),
        }
    }

    fn receiver_id() -> Digest {
        [
            BaseElement::new(0xB0B_0001),
            BaseElement::new(0xB0B_0002),
            BaseElement::new(0xB0B_0003),
            BaseElement::new(0xB0B_0004),
        ]
    }

    fn salt() -> Digest {
        [
            BaseElement::new(0x5EED_0001),
            BaseElement::new(0x5EED_0002),
            BaseElement::new(0x5EED_0003),
            BaseElement::new(0x5EED_0004),
        ]
    }

    /// Sube una hoja hasta la raíz del árbol de pendientes.
    fn climb_pending(leaf: Digest) -> Digest {
        let p = pending_path();
        let mut cur = leaf;
        for level in 0..TREE_DEPTH {
            cur = if p.is_right[level] {
                native_merge(p.siblings[level], cur)
            } else {
                native_merge(cur, p.siblings[level])
            };
        }
        cur
    }

    /// El compromiso que los custodios depositan.
    fn commitment() -> Digest {
        native_merge(
            native_merge(receiver_id(), salt()),
            [
                BaseElement::new(AMOUNT),
                BaseElement::ZERO,
                BaseElement::ZERO,
                BaseElement::ZERO,
            ],
        )
    }

    fn inputs_for(root: Digest) -> MintPendingPublicInputs {
        MintPendingPublicInputs {
            custodian_set_root: root,
            supply_old: BaseElement::new(SUPPLY_OLD),
            supply_new: BaseElement::new(SUPPLY_OLD + AMOUNT),
            max_supply: BaseElement::new(MAX_SUPPLY),
            amount: BaseElement::new(AMOUNT),
            pending_root_old: climb_pending([BaseElement::ZERO; 4]),
            pending_root_new: climb_pending(commitment()),
        }
    }

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

    /// Cinco custodios. Un 2-de-5.
    fn custodian_keys() -> Vec<BaseElement> {
        vec![
            BaseElement::new(0xC0570D1A),
            BaseElement::new(0xC0570D1B),
            BaseElement::new(0xC0570D1C),
            BaseElement::new(0xC0570D1D),
            BaseElement::new(0xC0570D1E),
        ]
    }

    fn run(
        key_a: BaseElement,
        idx_a: u64,
        path_a: &CustodianPath,
        key_b: BaseElement,
        idx_b: u64,
        path_b: &CustodianPath,
        declared_root: Digest,
    ) -> Result<(), String> {
        let trace = build_trace(
            key_a, idx_a, path_a, key_b, idx_b, path_b,
            SUPPLY_OLD,
            AMOUNT,
            MAX_SUPPLY,
            AMOUNT,
            receiver_id(),
            salt(),
            &pending_path(),
        );
        let prover = MintPendingProver::new(default_options());

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
        verify::<MintPendingAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof,
            inputs_for(declared_root),
            &min_opts,
        )
        .map_err(|e| format!("verificacion fallo: {e:?}"))
    }

    /// El dominio de custodio está separado del de cuentas: una clave de
    /// gasto no puede hacerse pasar por custodio.
    #[test]
    fn custodian_domain_is_separated() {
        let k = BaseElement::new(12345);
        assert_ne!(derive_custodian_id(k), derive_public_id(k));
    }

    /// La traza reconstruye la raíz que calcula la versión nativa.
    #[test]
    fn trace_roots_match_native() {
        let keys = custodian_keys();
        let (root, paths) = build_custodian_set(&keys);
        let trace = build_trace(
            keys[0], 0, &paths[0], keys[2], 2, &paths[2],
            SUPPLY_OLD,
            AMOUNT,
            MAX_SUPPLY,
            AMOUNT,
            receiver_id(),
            salt(),
            &pending_path(),
        );
        for i in 0..4 {
            assert_eq!(trace.get(4 + i, ROW_ROOT), root[i], "carril A elem {i}");
            assert_eq!(
                trace.get(LANE_B + 4 + i, ROW_ROOT),
                root[i],
                "carril B elem {i}"
            );
        }
        // Los acumuladores reconstruyen los indices.
        assert_eq!(trace.get(COL_ACC_A, ROW_ROOT), BaseElement::new(0));
        assert_eq!(trace.get(COL_ACC_B, ROW_ROOT), BaseElement::new(2));
    }

    /// EL TEST CLAVE: dos custodios distintos del conjunto autorizan.
    ///
    /// A diferencia de los negativos, NO silencia el pánico: si una traza
    /// válida no satisface alguna restricción, queremos ver cuál y en qué
    /// fila.
    #[test]
    fn two_distinct_custodians_authorize() {
        let keys = custodian_keys();
        let (root, paths) = build_custodian_set(&keys);
        let trace = build_trace(
            keys[1], 1, &paths[1], keys[3], 3, &paths[3],
            SUPPLY_OLD,
            AMOUNT,
            MAX_SUPPLY,
            AMOUNT,
            receiver_id(),
            salt(),
            &pending_path(),
        );
        let prover = MintPendingProver::new(default_options());
        let proof = prover.prove(trace).expect("la traza valida deberia probar");

        let min_opts = AcceptableOptions::OptionSet(vec![default_options()]);
        let v = verify::<MintPendingAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof,
            inputs_for(root),
            &min_opts,
        );
        assert!(v.is_ok(), "{v:?}");
    }

    /// Separa "la traza está mal construida" de "las restricciones están
    /// mal escritas": comprueba los acumuladores y los bits.
    #[test]
    fn trace_accumulators_and_bits_are_consistent() {
        let keys = custodian_keys();
        let (_, paths) = build_custodian_set(&keys);
        let trace = build_trace(
            keys[1], 1, &paths[1], keys[3], 3, &paths[3],
            SUPPLY_OLD,
            AMOUNT,
            MAX_SUPPLY,
            AMOUNT,
            receiver_id(),
            salt(),
            &pending_path(),
        );

        // El acumulador arranca a cero.
        assert_eq!(trace.get(COL_ACC_A, 0), BaseElement::ZERO);
        assert_eq!(trace.get(COL_ACC_B, 0), BaseElement::ZERO);

        // Y termina en el indice declarado.
        assert_eq!(trace.get(COL_ACC_A, ROW_ROOT), BaseElement::new(1));
        assert_eq!(trace.get(COL_ACC_B, ROW_ROOT), BaseElement::new(3));

        // Los bits de cada nivel corresponden al indice.
        for level in 0..CUSTODIAN_DEPTH {
            let row = (1 + level) * CYCLE_LENGTH;
            let expect_a = if (1u64 >> level) & 1 == 1 {
                BaseElement::ONE
            } else {
                BaseElement::ZERO
            };
            let expect_b = if (3u64 >> level) & 1 == 1 {
                BaseElement::ONE
            } else {
                BaseElement::ZERO
            };
            assert_eq!(trace.get(COL_BIT_A, row), expect_a, "bit A nivel {level}");
            assert_eq!(trace.get(COL_BIT_B, row), expect_b, "bit B nivel {level}");
        }
    }

    /// **EL TEST QUE IMPIDE EL 1-DE-N DISFRAZADO.**
    ///
    /// El mismo custodio intenta contar como dos firmantes. Si esto
    /// pasara, un esquema 2-de-5 sería en realidad un 1-de-5 y toda la
    /// garantía del umbral desaparecería.
    ///
    /// La resta `indice_b − indice_a − 1` daría la vuelta en el campo y
    /// no cabría en el rango.
    #[test]
    fn the_same_custodian_cannot_count_twice() {
        let keys = custodian_keys();
        let (root, paths) = build_custodian_set(&keys);
        assert!(
            run(keys[2], 2, &paths[2], keys[2], 2, &paths[2], root).is_err(),
            "CRITICO: el mismo custodio no debe poder contar como dos firmantes. \
             Si esto pasa, un 2-de-N es un 1-de-N disfrazado."
        );
    }

    /// Los índices en orden inverso también se rechazan: el orden
    /// estricto es lo que fuerza la distinción.
    #[test]
    fn reversed_order_is_rejected() {
        let keys = custodian_keys();
        let (root, paths) = build_custodian_set(&keys);
        assert!(run(keys[3], 3, &paths[3], keys[1], 1, &paths[1], root).is_err());
    }

    /// **NADIE DE FUERA PUEDE AUTORIZAR.**
    ///
    /// Una clave que no está en el conjunto no llega a la raíz.
    #[test]
    fn a_non_custodian_cannot_authorize() {
        let keys = custodian_keys();
        let (root, paths) = build_custodian_set(&keys);
        let intruder = BaseElement::new(0x1337);
        assert!(
            run(intruder, 0, &paths[0], keys[3], 3, &paths[3], root).is_err(),
            "CRITICO: quien no pertenece al conjunto no debe poder autorizar"
        );
    }

    /// **EL ÍNDICE ESTÁ ATADO AL CAMINO.**
    ///
    /// Se declara un índice que no corresponde a la posición demostrada.
    /// Sin la restricción del acumulador, un custodio podría declarar
    /// cualquier índice y burlar la comprobación de orden estricto — que
    /// es lo único que impide firmar dos veces.
    #[test]
    fn a_lied_index_is_rejected() {
        let keys = custodian_keys();
        let (root, paths) = build_custodian_set(&keys);
        // El custodio 2 declara ser el 0 para "adelantarse" al 1.
        assert!(
            run(keys[2], 0, &paths[2], keys[1], 1, &paths[1], root).is_err(),
            "CRITICO: el indice debe estar atado al camino demostrado, o la \
             comprobacion de orden estricto no vale nada"
        );
    }

    /// Declarar una raíz de conjunto distinta debe fallar.
    #[test]
    fn wrong_declared_set_root_is_rejected() {
        let keys = custodian_keys();
        let (_, paths) = build_custodian_set(&keys);
        let fake: Digest = [BaseElement::new(999); 4];
        assert!(run(keys[0], 0, &paths[0], keys[2], 2, &paths[2], fake).is_err());
    }

    // -----------------------------------------------------------------
    // La fase nueva: suministro y pendiente
    // -----------------------------------------------------------------

    /// Construye la traza válida del escenario.
    fn traza_valida() -> TraceTable<BaseElement> {
        let keys = custodian_keys();
        let (_, paths) = build_custodian_set(&keys);
        build_trace(
            keys[1], 1, &paths[1], keys[3], 3, &paths[3],
            SUPPLY_OLD,
            AMOUNT,
            MAX_SUPPLY,
            AMOUNT,
            receiver_id(),
            salt(),
            &pending_path(),
        )
    }

    /// **TODAS LAS ENTRADAS PÚBLICAS, NO SOLO LAS RAÍCES.**
    ///
    /// Comparar la estructura entera y no los campos que parecen
    /// importantes. En `circuit_send` la versión parcial dejó pasar un
    /// campo heredado y **costó ocho rondas de diagnóstico**.
    #[test]
    fn all_public_inputs_match_the_trace() {
        let trace = traza_valida();
        let root = build_custodian_set(&custodian_keys()).0;
        let derivadas = MintPendingProver::new(default_options()).get_pub_inputs(&trace);
        assert_eq!(
            derivadas.to_elements(),
            inputs_for(root).to_elements(),
            "las entradas DERIVADAS de la traza deben coincidir con las \
             DECLARADAS, o probador y verificador usaran transcripciones \
             distintas"
        );
    }

    /// **EL PENDIENTE QUEDA INSERTADO.**
    ///
    /// Y su compromiso se forma con la **identidad** del receptor: ninguna
    /// columna contiene su saldo, ni podría.
    #[test]
    fn the_pending_commitment_is_inserted() {
        let trace = traza_valida();
        let esperado = climb_pending(commitment());
        for i in 0..4 {
            assert_eq!(
                trace.get(LANE_B + 4 + i, ROW_PENDING_ROOT),
                esperado[i],
                "raiz nueva de pendientes, elemento {i}"
            );
            assert_eq!(
                trace.get(4 + i, ROW_PENDING_ROOT),
                climb_pending([BaseElement::ZERO; 4])[i],
                "raiz antigua: la posicion estaba libre, elemento {i}"
            );
        }
    }

    /// **EL SUMINISTRO SUBE EXACTAMENTE EL IMPORTE.**
    ///
    /// Declarar otro incremento se rechaza: sería crear dinero sin que el
    /// suministro lo refleje, o al revés.
    #[test]
    fn the_supply_must_rise_by_exactly_the_amount() {
        let keys = custodian_keys();
        let (root, paths) = build_custodian_set(&keys);
        // El suministro sube MAS que el importe.
        let trace = build_trace(
            keys[1], 1, &paths[1], keys[3], 3, &paths[3],
            SUPPLY_OLD,
            AMOUNT + 1000,
            MAX_SUPPLY,
            AMOUNT,
            receiver_id(),
            salt(),
            &pending_path(),
        );
        let prover = MintPendingProver::new(default_options());
        let hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| prover.prove(trace)));
        std::panic::set_hook(hook);
        let ok = match r {
            Ok(Ok(proof)) => {
                let min_opts = AcceptableOptions::OptionSet(vec![default_options()]);
                verify::<MintPendingAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
                    proof,
                    inputs_for(root),
                    &min_opts,
                )
                .is_ok()
            }
            _ => false,
        };
        assert!(
            !ok,
            "CRITICO: el suministro no puede subir mas que el importe emitido"
        );
    }

    /// **NINGUNA CUENTA INTERVIENE.**
    ///
    /// Es la propiedad que justifica este circuito: los custodios emiten
    /// **sin conocer el saldo de nadie**.
    ///
    /// Va en el tipo: la firma de `build_trace` recibe la identidad del
    /// receptor y un aleatorio. **No hay parámetro donde entrara un
    /// saldo**, ni columna donde alojarlo.
    #[test]
    fn no_account_balance_is_involved() {
        // El compromiso se reconstruye solo con identidad, aleatorio e
        // importe. Si hiciera falta un saldo, esto no cuadraria.
        let nativo = native_merge(
            native_merge(receiver_id(), salt()),
            [
                BaseElement::new(AMOUNT),
                BaseElement::ZERO,
                BaseElement::ZERO,
                BaseElement::ZERO,
            ],
        );
        assert_eq!(nativo, commitment());
        assert_eq!(
            traza_valida().get(LANE_B + 4, ROW_PENDING_ROOT),
            climb_pending(nativo)[0]
        );
    }

    /// **PRUEBA POR MUTACIÓN: ninguna restricción está vacía.**
    ///
    /// Si ninguna perturbación de una celda hace que una restricción se
    /// vuelva no nula, esa restricción no impone nada — y ningún test
    /// normal lo detecta. Ver `AUDITORIA.md` §12.
    ///
    /// **Este circuito es donde más importa.** Al construirlo se declararon
    /// **siete columnas** con sus restricciones escritas que la traza nunca
    /// rellenaba: valían cero y sus restricciones se cumplían trivialmente.
    /// Se corrigieron, pero nada comprobaba que no quedara ninguna.
    ///
    /// Se prueban **todas** las filas: con muestreo, una restricción activa
    /// en una sola fila aparece como vacía sin serlo.
    ///
    /// ⚠️ Un resultado limpio **no significa que el circuito sea correcto**:
    /// significa que no tiene este fallo concreto. El tope de emisión, por
    /// ejemplo, se transporta sin comprobarse — y eso esta prueba no lo ve.
    #[test]
    fn no_constraint_is_vacuous() {
        use crate::mutation::{buscar_vacias, rows_of};

        let trace = traza_valida();
        let rows = rows_of(&trace, TRACE_WIDTH, TRACE_LENGTH);
        let root = build_custodian_set(&custodian_keys()).0;

        let air = MintPendingAir::new(
            TraceInfo::new(TRACE_WIDTH, TRACE_LENGTH),
            inputs_for(root),
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

    /// Intenta generar y verificar una prueba; devuelve si lo consigue.
    fn intenta(trace: TraceTable<BaseElement>, inputs: MintPendingPublicInputs) -> bool {
        let prover = MintPendingProver::new(default_options());
        let hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| prover.prove(trace)));
        std::panic::set_hook(hook);
        match r {
            Ok(Ok(proof)) => {
                let min_opts = AcceptableOptions::OptionSet(vec![default_options()]);
                verify::<MintPendingAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
                    proof,
                    inputs,
                    &min_opts,
                )
                .is_ok()
            }
            _ => false,
        }
    }

    /// El compromiso pendiente **depende del importe**, asi que un test
    /// con importe variable necesita su propio compromiso y sus propias
    /// entradas publicas. Usar `inputs_for` fijo hace que la verificacion
    /// falle por entradas descuadradas, no por lo que el test comprueba.
    fn commitment_de(importe: u64) -> Digest {
        native_merge(
            native_merge(receiver_id(), salt()),
            [
                BaseElement::new(importe),
                BaseElement::ZERO,
                BaseElement::ZERO,
                BaseElement::ZERO,
            ],
        )
    }

    fn traza_con_suministro(
        delta: u64,
    ) -> (TraceTable<BaseElement>, MintPendingPublicInputs) {
        let keys = custodian_keys();
        let (root, paths) = build_custodian_set(&keys);
        let trace = build_trace(
            keys[1], 1, &paths[1], keys[3], 3, &paths[3],
            SUPPLY_OLD,
            delta,
            MAX_SUPPLY,
            delta,
            receiver_id(),
            salt(),
            &pending_path(),
        );
        let inputs = MintPendingPublicInputs {
            custodian_set_root: root,
            supply_old: BaseElement::new(SUPPLY_OLD),
            supply_new: BaseElement::new(SUPPLY_OLD + delta),
            max_supply: BaseElement::new(MAX_SUPPLY),
            amount: BaseElement::new(delta),
            pending_root_old: climb_pending([BaseElement::ZERO; 4]),
            pending_root_new: climb_pending(commitment_de(delta)),
        };
        (trace, inputs)
    }

    /// **EL TOPE SE IMPONE EN EL CIRCUITO, NO SOLO EN LA CAPA.**
    ///
    /// Hasta ahora el tope se transportaba como columna pero **nadie lo
    /// comprobaba**: la capa lo rechazaba, el circuito no. Estaba
    /// documentado como fallo abierto en `AUDITORIA.md`.
    ///
    /// Ahora un segmento de 64 filas descompone `tope − suministro_nuevo`
    /// en **63 bits**. Si el suministro se pasa, esa resta envuelve en el
    /// campo y da un valor de 64 bits: **no cabe**, y no hay descomposición
    /// que satisfaga el enlace final.
    #[test]
    fn minting_beyond_the_cap_is_rejected() {
        let (trace, inputs) = traza_con_suministro(MAX_SUPPLY - SUPPLY_OLD + 1);
        assert!(
            !intenta(trace, inputs),
            "CRITICO: emitir por encima del tope debe ser imposible EN EL \
             CIRCUITO, no solo en la capa"
        );
    }

    /// **Y llegar justo al tope sí se permite.**
    ///
    /// Sin este test, el anterior pasaría igual si el circuito rechazara
    /// **cualquier** emisión. El par distingue *impone el límite* de *no
    /// deja emitir nada*.
    #[test]
    fn minting_exactly_up_to_the_cap_is_allowed() {
        let (trace, inputs) = traza_con_suministro(MAX_SUPPLY - SUPPLY_OLD);
        assert!(
            intenta(trace, inputs),
            "alcanzar el tope exactamente es legitimo: lo alcanza, no lo supera"
        );
    }
}
