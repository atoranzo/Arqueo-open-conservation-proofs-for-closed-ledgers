//! # Emision a un pendiente, SIN autorizacion (AUDITORIA 68)
//!
//! La parte propia de `circuit_mint_pending`: subir el suministro, formar
//! el compromiso del pendiente e insertarlo en una posicion libre del
//! arbol, sin pasar del tope. La autorizacion de custodios se ha amputado
//! (entrada 33).
//!
//! ## Lo que SI sigue probando
//!
//! - `suministro_nuevo = suministro + importe`: **conservacion**.
//! - **El compromiso**: `P = H(H(identidad_receptor, aleatorio), importe)`,
//!   formado con la IDENTIDAD del receptor. Ninguna columna contiene su
//!   saldo, ni podria.
//! - **La posicion estaba libre**: el carril A sube la hoja CERO, el B el
//!   compromiso, con los mismos hermanos.
//! - **El tope**: un segmento de 64 filas descompone
//!   `tope - suministro_nuevo` en 63 bits. Si el suministro se pasara, esa
//!   resta envuelve en el campo y da un valor de 64 bits que no cabe.
//!
//! ## Por que el tope sobrevive a quitar los segmentos
//!
//! Aqui el tope tiene **mecanismo propio** (`COL_CBIT`/`COL_CACC`,
//! `C_CAP_LINK`) separado de los segmentos de rango. Los tres segmentos
//! que se van eran indices de custodio y su orden: **ninguno era un rango
//! de valor**, asi que `NUM_SEGMENTS` pasa de 3 a 0 sin llevarse por
//! delante la comprobacion del limite.
//!
//! Es la diferencia con `circuit_mint`, donde el margen del tope ERA el
//! quinto segmento y por eso alli quedaron cinco (66.1).
//!
//! ## Las filas 0-38 quedan MUERTAS
//!
//! Este circuito estaba montado al reves que los otros cuatro: el ascenso
//! de custodios iba **al principio** (filas 0-39), no al final. Al
//! quitarlo, las filas de delante quedan vacias.
//!
//! **El indicador de hash se apaga en ellas**, o las restricciones de
//! Rescue se activarian sobre ceros. No se ganan filas: `ROW_PENDING_ROOT`
//! = 311 obliga a 512 de todas formas.
//!
//! El compromiso arranca donde arrancaba porque `C_PEND_IN` lo ata a
//! `COL_R_ID` y `COL_SALT` con **su propio selector** (`P_PEND_IN`), no
//! con el de la raiz de custodios (68.2).
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
/// 512 filas. Las 0-38 estan muertas; el compromiso ocupa 39-55 y la
/// insercion llega a la 311.
pub const TRACE_LENGTH: usize = 512;

// ===== Columnas =====
//
// Fuera las diez de la autorizacion (68.3): los dos bits de direccion, las
// dos claves, los dos indices, los dos acumuladores y las dos del segmento
// de rango. De 49 a 39.
const LANE_B: usize = STATE_WIDTH; // 12
/// Suministro antes y despues. Una emision lo sube exactamente el importe.
const COL_SUPPLY_OLD: usize = 24;
const COL_SUPPLY_NEW: usize = 25;
/// Tope inmutable de emision.
const COL_MAX_SUPPLY: usize = 26;
/// Importe emitido.
const COL_AMOUNT: usize = 27;
/// **Identidad publica del receptor.** Funciona como direccion.
const COL_R_ID: usize = 28; // 28..32
/// Aleatorio que ciega el compromiso.
const COL_SALT: usize = 32; // 32..36
/// Bit de direccion en el arbol de pendientes.
const COL_PBIT: usize = 36;
/// Bit de la descomposicion del margen `tope - suministro_nuevo`.
const COL_CBIT: usize = 37;
/// Acumulador de Horner de esa descomposicion.
const COL_CACC: usize = 38;
pub const TRACE_WIDTH: usize = 39;

/// Primera fila del segmento que comprueba el tope de emision.
///
/// Va DESPUES de `ROW_PENDING_ROOT` (311), en filas que estaban vacias.
const CAP_START: usize = 320;
/// Longitud del segmento: 64 filas dan **63 bits**, no 64.
///
/// Ese margen de un bit es lo que hace que el tope se imponga. Ver
/// `AUDITORIA.md` 13.
const CAP_LENGTH: usize = 64;

// ===== Filas =====
/// Arranque del compromiso interno `H(identidad_receptor, aleatorio)`.
///
/// Era `ROW_ROOT`: la fila donde acababa el ascenso de custodios y donde
/// **a la vez** arrancaba el compromiso (68.1). Amputado el ascenso, se
/// queda solo con la segunda funcion, y conserva el 39 porque su selector
/// siempre fue propio.
const ROW_PEND_START: usize = 39;
/// Fila donde el compromiso interno esta hecho y entra el importe.
const ROW_PEND_INNER: usize = 47;
/// El pendiente completo entra al arbol.
const ROW_PENDING_ENTRY: usize = 55;
/// Raiz tras insertarlo: **entrada + 256**.
const ROW_PENDING_ROOT: usize = 311;

// ===== Restricciones =====
//
// De 125 ranuras a 89. Fuera las 36 de la autorizacion: capacidad y
// colocacion del arbol de custodios (16), bits booleanos (2), entrada de
// claves (2), acumulador de indice (2) y su enlace final (2), transporte
// de claves e indices (4), y el rango de los indices (8).
const C_HASH_A: usize = 0;
const C_HASH_B: usize = C_HASH_A + STATE_WIDTH;
/// **El suministro sube exactamente el importe.**
const C_SUPPLY: usize = C_HASH_B + STATE_WIDTH; // 1
/// Compromiso interno: capacidad (4), identidad (4), aleatorio (4).
const C_PEND_IN: usize = C_SUPPLY + 1; // 12
/// Compromiso completo: digest (4) e importe (1).
const C_PEND_VAL: usize = C_PEND_IN + 12; // 5
/// Capacidad a cero en la subida.
const C_PEND_CAP: usize = C_PEND_VAL + 5; // 8
/// **LA POSICION ESTABA LIBRE**: carril A con cero, B con el compromiso.
const C_PEND_ENTRY_A: usize = C_PEND_CAP + 8; // 4
const C_PEND_ENTRY_B: usize = C_PEND_ENTRY_A + 4; // 4
const C_PEND_PLACE: usize = C_PEND_ENTRY_B + 4; // 8
const C_PEND_SIBLING: usize = C_PEND_PLACE + 8; // 4
const C_PBIT_BOOL: usize = C_PEND_SIBLING + 4; // 1
/// Transporte de las columnas constantes.
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
const NUM_CONSTRAINTS: usize = C_CAP_LINK + 1; // 89

// ===== Periodicas =====
//
// De 41 a 32. Fuera las ocho de la autorizacion (`P_TREE_LINK`, `P_POW2`,
// `P_SEL_ROOT`, `P_FIRST_S`, `P_CONT_S` y las tres de `P_SEG_LINK`) **y
// ademas `P_FIRST_ROW`**, que solo leia `C_KEY_INPUT`.
//
// 68 contaba ocho. Son nueve: dejar la novena seria una periodica que se
// construye y nadie lee, el peso muerto que 66.2 mando retirar en
// `mint_climb` y que la entrada 39 declara que nada comprueba.
const P_HASH_FLAG: usize = 0;
const P_ARK1: usize = 1;
const P_ARK2: usize = P_ARK1 + STATE_WIDTH;
/// Fila del compromiso interno.
const P_PEND_IN: usize = P_ARK2 + STATE_WIDTH;
/// Fila del compromiso completo.
const P_PEND_VAL: usize = P_PEND_IN + 1;
/// Fila que entra al arbol.
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

/// Construye la traza de una emision a pendiente, sin autorizacion.
///
/// `supply_delta` permite variar el suministro en una cantidad distinta de
/// la emitida, para los tests de conservacion.
pub fn build_trace(
    supply_old: u64,
    supply_delta: u64,
    max_supply: u64,
    amount: u64,
    // **Identidad publica del receptor.** Funciona como direccion.
    //
    // **No hace falta su saldo**, que es lo que distingue esta emision de
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

    // ===== FILAS 0..=38 MUERTAS =====
    //
    // Quedan a cero en los dos carriles. El indicador de hash esta apagado
    // en ellas, asi que ninguna restriccion de Rescue las mira.
    let mut state_a = [zero; STATE_WIDTH];
    let mut state_b = [zero; STATE_WIDTH];

    for r in ROW_PEND_START..ROW_PENDING_ROOT {
        let pos = r % CYCLE_LENGTH;
        if pos < NUM_ROUNDS {
            Rp64_256::apply_round(&mut state_a, pos);
            Rp64_256::apply_round(&mut state_b, pos);
        } else {
            let digest_a: Digest = [state_a[4], state_a[5], state_a[6], state_a[7]];
            let digest_b: Digest = [state_b[4], state_b[5], state_b[6], state_b[7]];
            state_a = [zero; STATE_WIDTH];
            state_b = [zero; STATE_WIDTH];

            if r == ROW_PEND_START {
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
                // ENTRADA: carril A con hoja CERO -la posicion estaba
                // libre- y carril B con el compromiso.
                let libre: Digest = [zero; 4];
                place_pending(&mut state_a, &libre, 0);
                place_pending(&mut state_b, &digest_b, 0);
            } else if (7..38).contains(&(r / CYCLE_LENGTH)) {
                // El nivel 0 lo coloca la ENTRADA, en el ciclo 6. Aqui van
                // los niveles 1..31, en los ciclos 7..37.
                let nivel = r / CYCLE_LENGTH - 6;
                place_pending(&mut state_a, &digest_a, nivel);
                place_pending(&mut state_b, &digest_b, nivel);
            }
        }
        rows[r + 1][..STATE_WIDTH].copy_from_slice(&state_a);
        rows[r + 1][LANE_B..LANE_B + STATE_WIDTH].copy_from_slice(&state_b);
    }

    // Bits de direccion del camino de pendientes: ciclos 7..38.
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
    // El margen de un bit es lo que impone el tope. Ver `AUDITORIA.md` 13.
    let supply_new = supply_old + supply_delta;
    let margen = max_supply.wrapping_sub(supply_new);
    for p in 1..CAP_LENGTH {
        // p=1 toma el bit 62; p=63, el bit 0.
        let bit = (margen >> (CAP_LENGTH - 1 - p)) & 1;
        rows[CAP_START + p][COL_CBIT] = BaseElement::new(bit);
        rows[CAP_START + p][COL_CACC] = rows[CAP_START + p - 1][COL_CACC]
            + rows[CAP_START + p - 1][COL_CACC]
            + BaseElement::new(bit);
    }

    let mut trace = TraceTable::new(TRACE_WIDTH, TRACE_LENGTH);
    trace.fill(
        |s| s.copy_from_slice(&rows[0]),
        |step, s| s.copy_from_slice(&rows[step + 1]),
    );
    trace
}

/// Entradas publicas.
///
/// Sin `custodian_set_root`: **este circuito ya no sabe de custodios**. La
/// autoridad la comprueba la capa, por separado.
#[derive(Clone, Debug)]
pub struct MintPendingClimbPublicInputs {
    /// Suministro antes y despues. La diferencia es el importe emitido.
    pub supply_old: BaseElement,
    pub supply_new: BaseElement,
    /// Tope inmutable.
    pub max_supply: BaseElement,
    pub amount: BaseElement,
    /// **Arbol de pendientes ANTES.** La posicion estaba libre.
    pub pending_root_old: Digest,
    /// **Y DESPUES**, con el compromiso dentro.
    pub pending_root_new: Digest,
}

impl ToElements<BaseElement> for MintPendingClimbPublicInputs {
    fn to_elements(&self) -> Vec<BaseElement> {
        let mut out = vec![
            self.supply_old,
            self.supply_new,
            self.max_supply,
            self.amount,
        ];
        out.extend_from_slice(&self.pending_root_old);
        out.extend_from_slice(&self.pending_root_new);
        out
    }
}

pub struct MintPendingClimbAir {
    context: AirContext<BaseElement>,
    pub_inputs: MintPendingClimbPublicInputs,
}

impl Air for MintPendingClimbAir {
    type BaseField = BaseElement;
    type PublicInputs = MintPendingClimbPublicInputs;

    fn new(trace_info: TraceInfo, pub_inputs: Self::PublicInputs, options: ProofOptions) -> Self {
        assert_eq!(TRACE_WIDTH, trace_info.width());
        let full = vec![TRACE_LENGTH];

        let mut degrees = Vec::with_capacity(NUM_CONSTRAINTS);
        // Rescue, los dos carriles (24): grado 7 con ciclo.
        for _ in 0..2 * STATE_WIDTH {
            degrees.push(TransitionConstraintDegree::with_cycles(7, full.clone()));
        }
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
        // El ciclo es `full`, NO `CAP_LENGTH`: el segmento es un bloque
        // unico en una traza de 512 filas y no la llena. Las filas a cero
        // fuera del bloque rompen la periodicidad.
        for _ in 0..4 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }

        assert_eq!(degrees.len(), NUM_CONSTRAINTS, "cuenta de grados");

        MintPendingClimbAir {
            context: AirContext::new(trace_info, degrees, 12, options),
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

        // ===== EL INDICADOR DE HASH SE APAGA EN LAS FILAS MUERTAS =====
        //
        // Arranca en `ROW_PEND_START`, no en 0. Sin esto las restricciones
        // de Rescue se activarian sobre las filas 0-38, que ahora estan a
        // cero (68.2).
        let mut hash_flag = vec![zero; TRACE_LENGTH];
        for r in ROW_PEND_START..=ROW_PENDING_ROOT {
            if r % CYCLE_LENGTH < NUM_ROUNDS {
                hash_flag[r] = one;
            }
        }
        columns.push(hash_flag);

        for ark in [true, false] {
            for i in 0..STATE_WIDTH {
                let mut col = vec![zero; TRACE_LENGTH];
                for r in ROW_PEND_START..=ROW_PENDING_ROOT {
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

        // Compromiso interno del pendiente: una sola fila. **Selector
        // propio**: es lo que hace que el compromiso no dependa del
        // ascenso de custodios que se ha amputado (68.2).
        let mut pend_in = vec![zero; TRACE_LENGTH];
        pend_in[ROW_PEND_START] = one;
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
        //
        // **Mecanismo propio, no un segmento**: por eso quitar los tres
        // segmentos de rango no se lleva por delante el tope (68.3).
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
        let mut a = Vec::with_capacity(12);

        // ===== LA FILA 0 YA NO ANCLA NINGUN DOMINIO =====
        //
        // Las 26 aserciones de custodio se van: capacidad de los dos
        // carriles, `CUSTODIAN_DOMAIN`, relleno, acumuladores y la raiz del
        // conjunto. Las filas 0-38 estan muertas y ninguna restriccion las
        // lee, asi que fijarles valores seria ruido.
        //
        // Quedan las que sujetan algo: las constantes que `C_TRANSPORT_NEW`
        // propaga a toda la traza, y las dos raices del arbol de
        // pendientes.
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
        a.push(Assertion::single(COL_AMOUNT, 0, self.pub_inputs.amount));

        // **Las raices del arbol de pendientes**: antes libre, despues con
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

pub struct MintPendingClimbProver {
    options: ProofOptions,
}

impl MintPendingClimbProver {
    pub fn new(options: ProofOptions) -> Self {
        Self { options }
    }
}

impl Prover for MintPendingClimbProver {
    type BaseField = BaseElement;
    type Air = MintPendingClimbAir;
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

    fn get_pub_inputs(&self, trace: &Self::Trace) -> MintPendingClimbPublicInputs {
        MintPendingClimbPublicInputs {
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
    use crate::merkle::native_merge;
    use winterfell::{verify, AcceptableOptions, BatchingMethod, FieldExtension};

    const SUPPLY_OLD: u64 = 10_000_000;
    const MAX_SUPPLY: u64 = 100_000_000;
    const AMOUNT: u64 = 250_000;

    /// Camino del arbol de pendientes con la posicion libre.
    ///
    /// Direcciones **mixtas**: con todas iguales la traza degenera y el
    /// test pasaria sin comprobar el caso general.
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

    /// Sube una hoja hasta la raiz del arbol de pendientes.
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

    /// El compromiso que se deposita, para un importe dado.
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

    fn commitment() -> Digest {
        commitment_de(AMOUNT)
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

    /// Traza y entradas de una emision de `delta` al pendiente.
    fn caso(delta: u64) -> (TraceTable<BaseElement>, MintPendingClimbPublicInputs) {
        let trace = build_trace(
            SUPPLY_OLD,
            delta,
            MAX_SUPPLY,
            delta,
            receiver_id(),
            salt(),
            &pending_path(),
        );
        let inputs = MintPendingClimbPublicInputs {
            supply_old: BaseElement::new(SUPPLY_OLD),
            supply_new: BaseElement::new(SUPPLY_OLD + delta),
            max_supply: BaseElement::new(MAX_SUPPLY),
            amount: BaseElement::new(delta),
            pending_root_old: climb_pending([BaseElement::ZERO; 4]),
            pending_root_new: climb_pending(commitment_de(delta)),
        };
        (trace, inputs)
    }

    fn traza_valida() -> TraceTable<BaseElement> {
        caso(AMOUNT).0
    }

    /// Intenta generar y verificar; devuelve el detalle del fallo.
    ///
    /// **El mensaje del panico se conserva.** Winterfell da el indice y la
    /// fila de la restriccion que falla, o *"expected N assertions,
    /// received M"*. Descartarlo tira justo el dato que hace falta: en esta
    /// auditoria costo tres rondas por eso (`AUDITORIA.md` 25).
    fn correr(
        trace: TraceTable<BaseElement>,
        inputs: MintPendingClimbPublicInputs,
    ) -> Result<(), String> {
        let prover = MintPendingClimbProver::new(default_options());
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| prover.prove(trace)));
        let proof = match r {
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
        verify::<MintPendingClimbAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof, inputs, &min_opts,
        )
        .map_err(|e| format!("verificacion fallo: {e:?}"))
    }

    fn intenta(
        trace: TraceTable<BaseElement>,
        inputs: MintPendingClimbPublicInputs,
    ) -> bool {
        correr(trace, inputs).is_ok()
    }

    // =================================================================
    // EL POSITIVO VA PRIMERO.
    //
    // Los negativos pueden pasar TODOS con el circuito roto, porque
    // rechazan por cualquier motivo. En `mint` costo dos rondas
    // descubrirlo (66.2).
    // =================================================================

    #[test]
    fn a_valid_mint_pending_climb_verifies() {
        let (trace, inputs) = caso(AMOUNT);
        assert_eq!(correr(trace, inputs), Ok(()));
    }

    /// La traza reconstruye las raices que calcula la version nativa.
    #[test]
    fn trace_roots_match_native() {
        let trace = traza_valida();
        let vacio = climb_pending([BaseElement::ZERO; 4]);
        let lleno = climb_pending(commitment());
        for i in 0..4 {
            assert_eq!(
                trace.get(4 + i, ROW_PENDING_ROOT),
                vacio[i],
                "carril A (posicion libre), elem {i}"
            );
            assert_eq!(
                trace.get(LANE_B + 4 + i, ROW_PENDING_ROOT),
                lleno[i],
                "carril B (con el compromiso), elem {i}"
            );
        }
    }

    /// **LAS FILAS 0-38 ESTAN MUERTAS Y VACIAS.**
    ///
    /// Es la condicion que 68.2 pone para que la amputacion sea limpia: si
    /// quedara estado ahi, el indicador de hash apagado lo dejaria sin
    /// comprobar.
    #[test]
    fn the_rows_before_the_commitment_are_dead_and_empty() {
        let trace = traza_valida();
        for row in 0..=ROW_PEND_START {
            for col in 0..2 * STATE_WIDTH {
                assert_eq!(
                    trace.get(col, row),
                    BaseElement::ZERO,
                    "fila muerta {row}, columna {col} deberia estar a cero"
                );
            }
        }
        // Y la 40, la primera viva, NO esta vacia.
        assert_ne!(trace.get(4, ROW_PEND_START + 1), BaseElement::ZERO);
    }

    /// **EL COMPROMISO SE INSERTA EN LA POSICION LIBRE.**
    #[test]
    fn the_pending_commitment_is_inserted() {
        let trace = traza_valida();
        let esperado = commitment();
        for i in 0..4 {
            assert_eq!(
                trace.get(LANE_B + 4 + i, ROW_PENDING_ENTRY),
                esperado[i],
                "el carril B debe llevar el compromiso al entrar, elem {i}"
            );
        }
    }

    /// **NINGUN SALDO INTERVIENE.**
    ///
    /// El compromiso se reconstruye solo con identidad, aleatorio e
    /// importe. Si hiciera falta un saldo, esto no cuadraria.
    #[test]
    fn no_account_balance_is_involved() {
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

    /// CONSERVACION. El suministro debe subir EXACTAMENTE el importe.
    #[test]
    fn the_supply_must_rise_by_exactly_the_amount() {
        for delta in [AMOUNT - 1, AMOUNT + 1] {
            let trace = build_trace(
                SUPPLY_OLD,
                delta,
                MAX_SUPPLY,
                AMOUNT,
                receiver_id(),
                salt(),
                &pending_path(),
            );
            let inputs = MintPendingClimbPublicInputs {
                supply_old: BaseElement::new(SUPPLY_OLD),
                supply_new: BaseElement::new(SUPPLY_OLD + delta),
                max_supply: BaseElement::new(MAX_SUPPLY),
                amount: BaseElement::new(AMOUNT),
                pending_root_old: climb_pending([BaseElement::ZERO; 4]),
                pending_root_new: climb_pending(commitment()),
            };
            assert!(
                !intenta(trace, inputs),
                "SOLIDEZ: el suministro subio {delta} y el importe era {AMOUNT}"
            );
        }
    }

    /// **REGRESION (entrada 35 / 50.6).** Identidad del receptor
    /// inconsistente entre la fila del compromiso y el resto.
    ///
    /// En `send` esa constancia estaba muerta por el solapamiento de 38 y
    /// una traza con dos identidades verificaba. Aqui la disposicion cuadra
    /// por lectura, pero eso mismo parecia en `send`: **lo decide el test**.
    #[test]
    fn an_inconsistent_receiver_identity_is_rejected() {
        let (mut trace, inputs) = caso(AMOUNT);
        let otra = derive_public_id(BaseElement::new(0xA77ACC));
        assert_ne!(otra, receiver_id(), "el testigo debe diferir");
        for row in 0..TRACE_LENGTH {
            if row == ROW_PEND_START {
                continue; // la fila del compromiso, intacta
            }
            for i in 0..4 {
                trace.set(COL_R_ID + i, row, otra[i]);
            }
        }
        assert!(
            !intenta(trace, inputs),
            "SOLIDEZ (entrada 35): COL_R_ID inconsistente entre la fila del \
             compromiso y el resto -> mismo fallo que 50 en send."
        );
    }

    /// Una raiz nueva declarada que no corresponde se rechaza.
    #[test]
    fn wrong_declared_pending_root_is_rejected() {
        let (trace, mut inputs) = caso(AMOUNT);
        inputs.pending_root_new = inputs.pending_root_old;
        assert!(!intenta(trace, inputs));
    }

    /// **EL TOPE SE IMPONE EN EL CIRCUITO, NO SOLO EN LA CAPA.**
    ///
    /// Y sobrevive a la amputacion porque tiene mecanismo propio, no un
    /// segmento de rango (68.3).
    #[test]
    fn minting_beyond_the_cap_is_rejected() {
        let (trace, inputs) = caso(MAX_SUPPLY - SUPPLY_OLD + 1);
        assert!(
            !intenta(trace, inputs),
            "CRITICO: emitir por encima del tope debe ser imposible EN EL \
             CIRCUITO, no solo en la capa"
        );
    }

    /// **Y llegar justo al tope si se permite.**
    ///
    /// Sin este test, el anterior pasaria igual si el circuito rechazara
    /// **cualquier** emision. El par distingue *impone el limite* de *no
    /// deja emitir nada*.
    #[test]
    // **NO SE EJECUTA EN MODO DEPURACION, y no por estar mal.**
    //
    // Alcanza el tope exactamente, asi que el margen vale **cero** y los 63
    // bits del segmento son todos cero. Las restricciones booleanas sobre
    // ellos tienen grado real 0 en esta traza concreta, y winterfell
    // comprueba en depuracion que el grado declarado se realice.
    //
    // Se salta en depuracion en vez de debilitar el test: probar con margen
    // 1 dejaria sin comprobar el limite exacto, que es para lo que existe.
    // Ver `AUDITORIA.md` 20 y entrada 24.
    #[cfg_attr(debug_assertions, ignore = "grado 0: el margen del tope es cero")]
    fn minting_exactly_up_to_the_cap_is_allowed() {
        let (trace, inputs) = caso(MAX_SUPPLY - SUPPLY_OLD);
        assert!(
            intenta(trace, inputs),
            "alcanzar el tope exactamente es legitimo: lo alcanza, no lo supera"
        );
    }

    /// **PRUEBA POR MUTACION: ninguna restriccion esta vacia.**
    ///
    /// Se prueban **todas** las filas: con muestreo, una restriccion activa
    /// en una sola fila aparece como vacia sin serlo.
    ///
    /// Un resultado limpio **no significa que el circuito sea correcto**:
    /// significa que no tiene este fallo concreto.
    #[test]
    fn no_constraint_is_vacuous() {
        use crate::mutation::{buscar_vacias, rows_of};

        let (trace, inputs) = caso(AMOUNT);
        let rows = rows_of(&trace, TRACE_WIDTH, TRACE_LENGTH);
        let air = MintPendingClimbAir::new(
            TraceInfo::new(TRACE_WIDTH, TRACE_LENGTH),
            inputs,
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

    /// Camino de la POSICION 0: todos los bits de direccion a cero.
    fn camino_posicion_cero() -> MerklePath {
        MerklePath {
            siblings: pending_path().siblings,
            is_right: vec![false; TREE_DEPTH],
        }
    }

    /// Sube una hoja por un camino todo-izquierda.
    fn climb_izquierda(leaf: Digest) -> Digest {
        let p = camino_posicion_cero();
        let mut cur = leaf;
        for level in 0..TREE_DEPTH {
            cur = native_merge(cur, p.siblings[level]);
        }
        cur
    }

    /// **DISCRIMINANTE: la degeneracion es de la POSICION 0, no del
    /// circuito.**
    ///
    /// El camino de la posicion 0 tiene **todos** los bits de direccion a
    /// cero, asi que `COL_PBIT` es la columna identicamente nula. Todo
    /// termino `pbit * X` se anula y las veinte restricciones de
    /// `C_PEND_ENTRY_A/B`, `C_PEND_PLACE` y `C_PEND_SIBLING` **caen de
    /// grado 2 a 1**; `C_PBIT_BOOL`, de 1 a 0.
    ///
    /// Es el hallazgo de 37.7 -donde intervenir en `allocate_pending` para
    /// usar la segunda posicion dejaba «ni una `C_PEND_*` ni `C_PBIT_BOOL`
    /// desviada»- reproducido en el circuito nuevo.
    ///
    /// ## Por que este test decide algo y el positivo normal no
    ///
    /// `a_valid_mint_pending_climb_verifies` usa un camino **mixto**
    /// (`l % 3 == 0`) y pasa en depuracion. Este usa el degenerado y falla
    /// **solo ahi**. Con los dos, la causa queda aislada a una variable: el
    /// camino. Sin este, «es la posicion 0» seria una hipotesis sobre
    /// indices, que es como se degrado tres veces un fallo real a
    /// «cosmetico» (36).
    ///
    /// ## Lo que fija, y lo que no
    ///
    /// Fija que el circuito **es correcto tambien para la posicion 0**: en
    /// release genera y verifica. Lo que no encaja es la comprobacion de
    /// depuracion de winterfell, que asume que todo grado declarado se
    /// realiza en todo testigo. Limite conocido de la herramienta, no fallo
    /// de solidez: entradas 6, 24, 25 y 34, decision «declarar, no migrar»
    /// en 46.
    #[test]
    #[cfg_attr(
        debug_assertions,
        ignore = "grado dependiente del testigo: la posicion 0 anula COL_PBIT (entrada 6)"
    )]
    fn the_all_left_path_of_position_zero_still_verifies() {
        let camino = camino_posicion_cero();
        let trace = build_trace(
            SUPPLY_OLD,
            AMOUNT,
            MAX_SUPPLY,
            AMOUNT,
            receiver_id(),
            salt(),
            &camino,
        );
        let inputs = MintPendingClimbPublicInputs {
            supply_old: BaseElement::new(SUPPLY_OLD),
            supply_new: BaseElement::new(SUPPLY_OLD + AMOUNT),
            max_supply: BaseElement::new(MAX_SUPPLY),
            amount: BaseElement::new(AMOUNT),
            pending_root_old: climb_izquierda([BaseElement::ZERO; 4]),
            pending_root_new: climb_izquierda(commitment()),
        };
        assert_eq!(
            correr(trace, inputs),
            Ok(()),
            "el circuito debe ser correcto TAMBIEN para la posicion 0: lo que \
             no encaja es la comprobacion de grados de depuracion, no la prueba"
        );
    }

    /// **Y la columna del bit es efectivamente nula.** Sin esto, el test de
    /// arriba probaria que algo falla en la posicion 0, pero no QUE.
    #[test]
    fn the_position_zero_path_leaves_the_direction_bit_identically_zero() {
        let cero = build_trace(
            SUPPLY_OLD,
            AMOUNT,
            MAX_SUPPLY,
            AMOUNT,
            receiver_id(),
            salt(),
            &camino_posicion_cero(),
        );
        for row in 0..TRACE_LENGTH {
            assert_eq!(
                cero.get(COL_PBIT, row),
                BaseElement::ZERO,
                "posicion 0: COL_PBIT deberia ser identicamente nula, fila {row}"
            );
        }
        // Y con un camino mixto NO lo es: es la unica variable que cambia.
        let mixto = traza_valida();
        assert!(
            (0..TRACE_LENGTH).any(|r| mixto.get(COL_PBIT, r) != BaseElement::ZERO),
            "camino mixto: COL_PBIT no deberia ser nula"
        );
    }
}
