//! # Subida dual del árbol de CONGELADOS, sin autorización (§59)
//!
//! Es `dual_climb` a profundidad `FROZEN_DEPTH`: prueba que **una posición
//! del árbol de congelados cambió**, con los hermanos compartidos entre las
//! dos subidas para que un probador no pueda fabricar la raíz nueva con un
//! camino distinto del de la vieja.
//!
//! ## Por qué existe, y por qué es una copia
//!
//! `circuit_freeze` lleva esta subida **empotrada** junto a la de custodios
//! (§58.1): sus filas 0-191 son exactamente esto, y las 192-231 la
//! autorización. La entrada 33 amputa la segunda parte, y lo que queda es
//! este circuito.
//!
//! ⚠️ **Es una copia de `dual_climb` con la profundidad cambiada**, y eso es
//! deuda declarada. `dual_climb` opera a `FROZEN_DEPTH` = 32 y el árbol de
//! congelados tiene 24 (§58.2). Parametrizar la profundidad con genéricos
//! constantes evitaría duplicar, pero toca un circuito que hoy funciona y se
//! prefiere no hacerlo en el mismo paso que la amputación. **Si uno de los
//! dos se corrige, el otro necesita la misma corrección.**
//!
//! ## Qué NO prueba
//!
//! Ni quién autorizó el cambio ni que la hoja nueva sea la marca de
//! congelado: las hojas son **valores libres**, igual que en `circuit_freeze`
//! (§58.3). Lo que hace legítimo el cambio es que dos custodios firmen esta
//! transición de raíces, y eso lo comprueba la capa.

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

use crate::circuit_freeze::FROZEN_DEPTH;
use crate::merkle::{native_merge, Digest, MerklePath};
use crate::rescue_hash::{apply_sbox, NUM_ROUNDS, STATE_WIDTH};

pub const CYCLE_LENGTH: usize = 8;
/// 32 niveles × 8 filas = 256 (potencia de dos ✓).
pub const TRACE_LENGTH: usize = FROZEN_DEPTH * CYCLE_LENGTH;
/// 12 del carril A + 12 del carril B + 1 del bit compartido.
pub const TRACE_WIDTH: usize = 2 * STATE_WIDTH + 1;

/// Desplazamiento del carril B dentro de la fila.
const LANE_B: usize = STATE_WIDTH;
/// Columna del bit de dirección, compartido.
const COL_BIT: usize = 2 * STATE_WIDTH;

type Blake3 = Blake3_256<BaseElement>;

/// Sube una hoja por un camino, de forma nativa.
pub fn native_climb(leaf: Digest, path: &MerklePath) -> Digest {
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

/// Construye la traza: dos carriles subiendo el MISMO camino.
pub fn build_trace(leaf_a: Digest, leaf_b: Digest, path: &MerklePath) -> TraceTable<BaseElement> {
    // El camino llega como `MerklePath` del arbol de cuentas (32 niveles)
    // y solo se usan los `FROZEN_DEPTH` primeros, igual que en
    // `circuit_freeze`.
    assert!(path.siblings.len() >= FROZEN_DEPTH);
    assert!(path.is_right.len() >= FROZEN_DEPTH);

    let zero = BaseElement::ZERO;
    let mut rows: Vec<Vec<BaseElement>> = vec![vec![zero; TRACE_WIDTH]; TRACE_LENGTH];

    // Estado inicial de cada carril: nivel 0 del árbol.
    let mut state_a = [zero; STATE_WIDTH];
    let mut state_b = [zero; STATE_WIDTH];
    let place = |state: &mut [BaseElement; STATE_WIDTH], digest: &Digest, level: usize| {
        if path.is_right[level] {
            state[4..8].copy_from_slice(&path.siblings[level]);
            state[8..12].copy_from_slice(digest);
        } else {
            state[4..8].copy_from_slice(digest);
            state[8..12].copy_from_slice(&path.siblings[level]);
        }
    };
    place(&mut state_a, &leaf_a, 0);
    place(&mut state_b, &leaf_b, 0);

    rows[0][..STATE_WIDTH].copy_from_slice(&state_a);
    rows[0][LANE_B..LANE_B + STATE_WIDTH].copy_from_slice(&state_b);

    for r in 0..TRACE_LENGTH - 1 {
        let pos = r % CYCLE_LENGTH;
        if pos < NUM_ROUNDS {
            Rp64_256::apply_round(&mut state_a, pos);
            Rp64_256::apply_round(&mut state_b, pos);
        } else {
            // Fila de enlace: ambos carriles suben un nivel, con el MISMO
            // hermano y el MISMO bit.
            let digest_a: Digest = [state_a[4], state_a[5], state_a[6], state_a[7]];
            let digest_b: Digest = [state_b[4], state_b[5], state_b[6], state_b[7]];
            state_a = [zero; STATE_WIDTH];
            state_b = [zero; STATE_WIDTH];
            let next_level = (r + 1) / CYCLE_LENGTH;
            if next_level < FROZEN_DEPTH {
                place(&mut state_a, &digest_a, next_level);
                place(&mut state_b, &digest_b, next_level);
            }
        }
        rows[r + 1][..STATE_WIDTH].copy_from_slice(&state_a);
        rows[r + 1][LANE_B..LANE_B + STATE_WIDTH].copy_from_slice(&state_b);
    }

    // El bit de dirección es constante dentro de cada ciclo.
    for level in 0..FROZEN_DEPTH {
        let bit = if path.is_right[level] {
            BaseElement::ONE
        } else {
            zero
        };
        for p in 0..CYCLE_LENGTH {
            rows[level * CYCLE_LENGTH + p][COL_BIT] = bit;
        }
    }

    let mut trace = TraceTable::new(TRACE_WIDTH, TRACE_LENGTH);
    trace.fill(
        |state| state.copy_from_slice(&rows[0]),
        |step, state| state.copy_from_slice(&rows[step + 1]),
    );
    trace
}

/// Inputs públicos: las dos raíces. Las hojas y el camino son privados.
#[derive(Clone, Debug)]
pub struct FrozenClimbPublicInputs {
    pub root_a: Digest,
    pub root_b: Digest,
}

impl ToElements<BaseElement> for FrozenClimbPublicInputs {
    fn to_elements(&self) -> Vec<BaseElement> {
        let mut out = self.root_a.to_vec();
        out.extend_from_slice(&self.root_b);
        out
    }
}

pub struct FrozenClimbAir {
    context: AirContext<BaseElement>,
    root_a: Digest,
    root_b: Digest,
}

impl Air for FrozenClimbAir {
    type BaseField = BaseElement;
    type PublicInputs = FrozenClimbPublicInputs;

    fn new(trace_info: TraceInfo, pub_inputs: Self::PublicInputs, options: ProofOptions) -> Self {
        assert_eq!(TRACE_WIDTH, trace_info.width());

        let mut degrees = Vec::new();
        // 0..11 y 12..23: rondas de Rescue de cada carril (grado 7).
        for _ in 0..2 * STATE_WIDTH {
            degrees.push(TransitionConstraintDegree::with_cycles(7, vec![CYCLE_LENGTH]));
        }
        // 24..27 y 28..31: reinicio de capacidad de cada carril.
        for _ in 0..8 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, vec![CYCLE_LENGTH]));
        }
        // 32..35 y 36..39: colocación del digest en cada carril (grado 2).
        for _ in 0..8 {
            degrees.push(TransitionConstraintDegree::with_cycles(2, vec![CYCLE_LENGTH]));
        }
        // 40..43: LA RESTRICCIÓN CLAVE — hermano compartido (grado 2).
        for _ in 0..4 {
            degrees.push(TransitionConstraintDegree::with_cycles(2, vec![CYCLE_LENGTH]));
        }
        // 44: el bit es booleano.
        degrees.push(TransitionConstraintDegree::new(2));

        FrozenClimbAir {
            // 16 aserciones: ver get_assertions.
            context: AirContext::new(trace_info, degrees, 16, options),
            root_a: pub_inputs.root_a,
            root_b: pub_inputs.root_b,
        }
    }

    fn context(&self) -> &AirContext<Self::BaseField> {
        &self.context
    }

    /// Selector de ronda + constantes ARK, de ciclo 8.
    fn get_periodic_column_values(&self) -> Vec<Vec<Self::BaseField>> {
        let zero = BaseElement::ZERO;
        let one = BaseElement::ONE;
        let mut columns = Vec::with_capacity(1 + 2 * STATE_WIDTH);

        let mut hash_flag = vec![one; NUM_ROUNDS];
        hash_flag.push(zero);
        columns.push(hash_flag);

        for i in 0..STATE_WIDTH {
            let mut col: Vec<BaseElement> =
                (0..NUM_ROUNDS).map(|r| Rp64_256::ARK1[r][i]).collect();
            col.push(zero);
            columns.push(col);
        }
        for i in 0..STATE_WIDTH {
            let mut col: Vec<BaseElement> =
                (0..NUM_ROUNDS).map(|r| Rp64_256::ARK2[r][i]).collect();
            col.push(zero);
            columns.push(col);
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
        let link_flag = E::ONE - hash_flag;

        // ===== Rondas de Rescue en ambos carriles =====
        // Misma técnica de "encontrarse en el medio" que en rescue_hash.rs,
        // aplicada dos veces con el desplazamiento del carril.
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
                result[lane * STATE_WIDTH + i] = hash_flag * (apply_sbox(b[i]) - a[i]);
            }
        }

        // El bit del nivel DESTINO vive en la fila siguiente (lección
        // aprendida en `merkle.rs` con evidencia real).
        let bit = next[COL_BIT];

        // ===== Reinicio de capacidad en ambos carriles =====
        for i in 0..4 {
            result[24 + i] = link_flag * next[i];
            result[28 + i] = link_flag * next[LANE_B + i];
        }

        // ===== Colocación del digest según el bit, en cada carril =====
        for i in 0..4 {
            let digest_a = current[4 + i];
            let placed_a =
                (E::ONE - bit) * (next[4 + i] - digest_a) + bit * (next[8 + i] - digest_a);
            result[32 + i] = link_flag * placed_a;

            let digest_b = current[LANE_B + 4 + i];
            let placed_b = (E::ONE - bit) * (next[LANE_B + 4 + i] - digest_b)
                + bit * (next[LANE_B + 8 + i] - digest_b);
            result[36 + i] = link_flag * placed_b;
        }

        // ===== LA RESTRICCIÓN CLAVE: el hermano es el MISMO =====
        //
        // El hermano ocupa la mitad del estado que NO ocupa el digest:
        // - bit = 0 → digest a la izquierda (4..8), hermano a la derecha (8..12)
        // - bit = 1 → hermano a la izquierda (4..8), digest a la derecha (8..12)
        //
        // Sin esto, cada carril podría usar hermanos distintos y fabricar
        // una raíz que no corresponde a la misma actualización del árbol.
        for i in 0..4 {
            let sibling_a = (E::ONE - bit) * next[8 + i] + bit * next[4 + i];
            let sibling_b =
                (E::ONE - bit) * next[LANE_B + 8 + i] + bit * next[LANE_B + 4 + i];
            result[40 + i] = link_flag * (sibling_a - sibling_b);
        }

        // ===== El bit es booleano =====
        result[44] = current[COL_BIT] * (current[COL_BIT] - E::ONE);
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        let zero = BaseElement::ZERO;
        let last = TRACE_LENGTH - 1;
        let mut assertions = Vec::with_capacity(16);

        // Capacidad inicial a cero en ambos carriles (seguridad de la
        // construcción esponja).
        for i in 0..4 {
            assertions.push(Assertion::single(i, 0, zero));
            assertions.push(Assertion::single(LANE_B + i, 0, zero));
        }
        // Las dos raíces finales son públicas.
        for i in 0..4 {
            assertions.push(Assertion::single(4 + i, last, self.root_a[i]));
            assertions.push(Assertion::single(LANE_B + 4 + i, last, self.root_b[i]));
        }

        assertions
    }
}

pub struct FrozenClimbProver {
    options: ProofOptions,
}

impl FrozenClimbProver {
    pub fn new(options: ProofOptions) -> Self {
        Self { options }
    }
}

impl Prover for FrozenClimbProver {
    type BaseField = BaseElement;
    type Air = FrozenClimbAir;
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

    fn get_pub_inputs(&self, trace: &Self::Trace) -> FrozenClimbPublicInputs {
        use winterfell::Trace;
        let last = trace.length() - 1;
        FrozenClimbPublicInputs {
            root_a: [
                trace.get(4, last),
                trace.get(5, last),
                trace.get(6, last),
                trace.get(7, last),
            ],
            root_b: [
                trace.get(LANE_B + 4, last),
                trace.get(LANE_B + 5, last),
                trace.get(LANE_B + 6, last),
                trace.get(LANE_B + 7, last),
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

