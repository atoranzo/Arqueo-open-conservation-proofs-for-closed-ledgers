//! **Autoridad de emisión con umbral**: la creación de dinero deja de
//! depender de una sola clave.
//!
//! ## El punto de confianza que esto elimina
//!
//! Hasta ahora, una única clave controlaba la emisión. Quien la robara
//! —un atacante, un empleado, un estado hostil— podría crear dinero hasta
//! el tope del sistema. Ningún banco central real opera así.
//!
//! Aquí la emisión exige **M firmas de N custodios**. El conjunto de
//! custodios está comprometido en una raíz pública, y el circuito
//! demuestra que M miembros **distintos** de ese conjunto han autorizado.
//!
//! ## El riesgo que hay que cerrar: el mismo custodio contando dos veces
//!
//! Lo difícil no es impedir que firme alguien de fuera —eso lo resuelve
//! la pertenencia al conjunto—. Lo difícil es impedir que **el mismo
//! custodio cuente como dos**. Un 2-de-3 en el que un custodio pueda
//! duplicarse es un **1-de-3 disfrazado**, y la diferencia es toda la
//! garantía.
//!
//! Se cierra en dos pasos:
//!
//! 1. **Índices estrictamente crecientes.** Se demuestra
//!    `indice_b - indice_a - 1 >= 0` con una comprobación de rango. Si
//!    fueran iguales, la resta daría la vuelta en el campo.
//! 2. **Los índices están atados a los caminos.** Un acumulador
//!    reconstruye el índice a partir de los bits de dirección del camino
//!    de Merkle, y se comprueba que coincide con el declarado. Sin esto,
//!    el índice sería un número inventado sin relación con la posición
//!    realmente demostrada, y el paso 1 no valdría nada.
//!
//! ## Alcance de esta pieza
//!
//! Implementa **2-de-N**, no M-de-N general. Cada firmante adicional
//! necesita su propio carril en la traza; con dos carriles, M=2.
//! Extender a M=3 sería añadir un tercero, mecánico pero no gratuito.
//!
//! 2-de-N ya es cualitativamente distinto de 1-de-1: elimina el punto
//! único de fallo, que es el principio en juego.
//!
//! ## Estructura de la traza (34 columnas × 64 filas)
//!
//! | Ciclos | Filas | Fase |
//! |---|---|---|
//! | 0 | 0..7 | Derivación de las dos identidades desde sus claves |
//! | 1-4 | 8..39 | Subida al conjunto de custodios (4 niveles) |
//!
//! Las comprobaciones de rango van en paralelo, en las filas 0..23.
//!
//! Es un circuito **pequeño** comparado con los demás: el conjunto de
//! custodios tiene 4 niveles, no 32.

use winterfell::crypto::hashers::{Blake3_256, Rp64_256};
use winterfell::crypto::{DefaultRandomCoin, MerkleTree};
use winterfell::math::{fields::f64::BaseElement, FieldElement, StarkField, ToElements};
use winterfell::matrix::ColMatrix;
use winterfell::{
    Air, AirContext, Assertion, AuxRandElements, CompositionPoly, CompositionPolyTrace,
    ConstraintCompositionCoefficients, DefaultConstraintCommitment, DefaultConstraintEvaluator,
    DefaultTraceLde, EvaluationFrame, PartitionOptions, ProofOptions, Prover, StarkDomain,
    TraceInfo, TracePolyTable, TraceTable, TransitionConstraintDegree,
};

use crate::merkle::{native_merge, Digest};
use crate::rescue_hash::{apply_sbox, NUM_ROUNDS, STATE_WIDTH};

/// Dominio de derivación de identidades de custodio. **Distinto del de
/// cuentas y del de emisor**: una clave de gasto no puede hacerse pasar
/// por custodio.
pub const CUSTODIAN_DOMAIN: u64 = 0x43555354; // "CUST"

pub const CYCLE_LENGTH: usize = 8;
pub const TRACE_LENGTH: usize = 64;
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
pub const TRACE_WIDTH: usize = 34;

// ===== Filas =====
/// Última fila activa: raíz del conjunto de custodios.
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
/// El acumulador reconstruye el índice desde los bits del camino.
const C_ACC: usize = C_KEY_INPUT + 2; // 2
/// El acumulador final coincide con el índice declarado.
const C_ACC_FINAL: usize = C_ACC + 2; // 2
const C_TRANSPORT: usize = C_ACC_FINAL + 2; // 4
const C_SBIT_BOOL: usize = C_TRANSPORT + 4; // 2
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

/// Identidad pública de un custodio desde su clave.
pub fn derive_custodian_id(key: BaseElement) -> Digest {
    native_merge(
        as_digest(BaseElement::new(CUSTODIAN_DOMAIN)),
        as_digest(key),
    )
}

/// Camino de autenticación dentro del conjunto de custodios.
#[derive(Clone, Debug)]
pub struct CustodianPath {
    pub siblings: Vec<Digest>,
    pub is_right: Vec<bool>,
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
) -> TraceTable<BaseElement> {
    let zero = BaseElement::ZERO;
    let mut rows: Vec<Vec<BaseElement>> = vec![vec![zero; TRACE_WIDTH]; TRACE_LENGTH];

    for row in rows.iter_mut() {
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
            let level = r / CYCLE_LENGTH; // 0..3
            if level < CUSTODIAN_DEPTH {
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
pub struct ThresholdPublicInputs {
    pub custodian_set_root: Digest,
}

impl ToElements<BaseElement> for ThresholdPublicInputs {
    fn to_elements(&self) -> Vec<BaseElement> {
        self.custodian_set_root.to_vec()
    }
}

pub struct ThresholdAir {
    context: AirContext<BaseElement>,
    pub_inputs: ThresholdPublicInputs,
}

impl Air for ThresholdAir {
    type BaseField = BaseElement;
    type PublicInputs = ThresholdPublicInputs;

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

        assert_eq!(degrees.len(), NUM_CONSTRAINTS, "cuenta de grados");

        ThresholdAir {
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
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        let zero = BaseElement::ZERO;
        let mut a = Vec::with_capacity(26);

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

        a
    }
}

pub struct ThresholdProver {
    options: ProofOptions,
}

impl ThresholdProver {
    pub fn new(options: ProofOptions) -> Self {
        Self { options }
    }
}

impl Prover for ThresholdProver {
    type BaseField = BaseElement;
    type Air = ThresholdAir;
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

    fn get_pub_inputs(&self, trace: &Self::Trace) -> ThresholdPublicInputs {
        ThresholdPublicInputs {
            custodian_set_root: [
                trace.get(4, ROW_ROOT),
                trace.get(5, ROW_ROOT),
                trace.get(6, ROW_ROOT),
                trace.get(7, ROW_ROOT),
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
        let trace = build_trace(key_a, idx_a, path_a, key_b, idx_b, path_b);
        let prover = ThresholdProver::new(default_options());

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
        verify::<ThresholdAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof,
            ThresholdPublicInputs {
                custodian_set_root: declared_root,
            },
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
        let trace = build_trace(keys[0], 0, &paths[0], keys[2], 2, &paths[2]);
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
        // ===== Y TODAS LAS ENTRADAS PÚBLICAS, NO SOLO LAS RAÍCES =====
        //
        // Comparar la estructura entera. En `circuit_send` la versión
        // parcial dejó pasar un campo heredado de otra operación y **costó
        // ocho rondas de diagnóstico**: el error de winterfell
        // —`InconsistentOodConstraintEvaluations`— apunta a las
        // restricciones, no a las entradas.
        let derivadas = ThresholdProver::new(default_options()).get_pub_inputs(&trace);
        assert_eq!(
            derivadas.to_elements(),
            ThresholdPublicInputs { custodian_set_root: root }.to_elements(),
            "las entradas DERIVADAS de la traza deben coincidir con las \
             DECLARADAS en todos sus campos"
        );

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
        let trace = build_trace(keys[1], 1, &paths[1], keys[3], 3, &paths[3]);
        let prover = ThresholdProver::new(default_options());
        let proof = prover.prove(trace).expect("la traza valida deberia probar");

        let min_opts = AcceptableOptions::OptionSet(vec![default_options()]);
        let v = verify::<ThresholdAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof,
            ThresholdPublicInputs {
                custodian_set_root: root,
            },
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
        let trace = build_trace(keys[1], 1, &paths[1], keys[3], 3, &paths[3]);

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
}
