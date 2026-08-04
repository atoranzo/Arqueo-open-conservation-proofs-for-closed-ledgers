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
/// ⚠️ **Winterfell exige longitud de traza potencia de dos**, y la subida a
/// congelados ocupa `24 × 8 = 192`, que no lo es. Se sube a 256 y los ocho
/// niveles sobrantes son **relleno**: se sigue subiendo con hermano cero para
/// que las restricciones de enlace se satisfagan hasta el final, y no
/// significan nada porque las raices se anclan en `ROW_ROOT`.
///
/// Es la misma razon por la que `circuit_freeze` usa 512 filas para una
/// subida de 24 niveles mas una de 4: la potencia de dos siguiente.
pub const TRACE_LENGTH: usize = 256;

/// Fila donde el estado contiene las raices de verdad. Lo que hay despues es
/// relleno.
pub const ROW_ROOT: usize = FROZEN_DEPTH * CYCLE_LENGTH - 1;
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
            } else {
                // Niveles de relleno hasta la potencia de dos. Se sigue
                // subiendo con hermano CERO —y bit cero, que es el valor por
                // defecto de la columna— para que las restricciones de enlace
                // se satisfagan. Lo que sale de aqui no se usa: las raices
                // estan ancladas en ROW_ROOT.
                state_a[4..8].copy_from_slice(&digest_a);
                state_b[4..8].copy_from_slice(&digest_b);
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
        // ⚠️ En ROW_ROOT, no en la ultima fila: lo que viene despues es
        // relleno para llegar a la potencia de dos.
        let last = ROW_ROOT;
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
        // ROW_ROOT, no la ultima fila: las 64 ultimas son relleno para llegar
        // a la potencia de dos. Leer de ahi daba una raiz que no es la del
        // arbol, y la prueba no verificaba contra la declarada.
        debug_assert!(trace.length() > ROW_ROOT);
        let last = ROW_ROOT;
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

#[cfg(test)]
mod tests {
    // ⚠️ Adaptados de `dual_climb` junto con el circuito (§59.1). Si alli se
    // corrigen o se anaden, aqui hace falta la misma correccion: son dos
    // copias de la misma pieza a profundidades distintas.

    use super::*;
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

    fn digest_from(n: u64) -> Digest {
        [
            BaseElement::new(n),
            BaseElement::new(n + 1),
            BaseElement::new(n + 2),
            BaseElement::new(n + 3),
        ]
    }

    fn sample_path() -> MerklePath {
        MerklePath {
            siblings: (0..FROZEN_DEPTH).map(|i| digest_from(i as u64 * 10)).collect(),
            is_right: (0..FROZEN_DEPTH).map(|i| i % 3 == 0).collect(),
        }
    }

    /// El test más informativo si algo falla: la traza debe terminar con
    /// las mismas raíces que calcula la versión nativa, en ambos carriles.
    #[test]
    fn trace_roots_match_native_computation() {
        let path = sample_path();
        let leaf_a = digest_from(1000);
        let leaf_b = digest_from(2000);

        let trace = build_trace(leaf_a, leaf_b, &path);
        let expected_a = native_climb(leaf_a, &path);
        let expected_b = native_climb(leaf_b, &path);

        // La raiz esta en ROW_ROOT; despues solo hay relleno.
        let last = ROW_ROOT;
        for i in 0..4 {
            assert_eq!(trace.get(4 + i, last), expected_a[i], "carril A, elem {i}");
            assert_eq!(
                trace.get(LANE_B + 4 + i, last),
                expected_b[i],
                "carril B, elem {i}"
            );
        }
    }

    /// EL TEST CLAVE: dos hojas subiendo el mismo camino producen una
    /// prueba verificable.
    #[test]
    fn valid_frozen_climb_produces_verifiable_proof() {
        let path = sample_path();
        let leaf_a = digest_from(1000);
        let leaf_b = digest_from(2000);

        let trace = build_trace(leaf_a, leaf_b, &path);
        let prover = FrozenClimbProver::new(default_options());
        let proof = prover.prove(trace).expect("la generacion no deberia fallar");

        let min_opts = AcceptableOptions::OptionSet(vec![prover.options().clone()]);
        let verification =
            verify::<FrozenClimbAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
                proof,
                FrozenClimbPublicInputs {
                    root_a: native_climb(leaf_a, &path),
                    root_b: native_climb(leaf_b, &path),
                },
                &min_opts,
            );

        assert!(
            verification.is_ok(),
            "una subida dual valida deberia verificar: {verification:?}"
        );
    }

    /// EL TEST QUE JUSTIFICA TODO EL DISEÑO EN LOCKSTEP.
    ///
    /// ## Por qué la primera versión de este test no valía
    ///
    /// La primera versión corrompía un hermano del carril B directamente
    /// en la traza. Eso lo detectaba la restricción del HASH (el estado
    /// corrompido alimenta la ronda siguiente), no necesariamente la del
    /// hermano compartido — así que el test pasaba aunque la restricción
    /// clave no hiciera nada. Confianza falsa.
    ///
    /// ## El ataque real que este test sí construye
    ///
    /// Se construye una traza donde el carril A sube por el camino
    /// `path_a` y el carril B por `path_b`, **con hermanos distintos en
    /// el nivel 5**, y cada carril tiene todos sus hashes internamente
    /// CORRECTOS respecto a su propio camino.
    ///
    /// Es exactamente lo que un diseño secuencial permitiría: dos
    /// subidas coherentes por separado, pero por caminos distintos, para
    /// fabricar una raíz nueva que no corresponde a la misma posición del
    /// árbol. **Solo la restricción cruzada de hermano compartido puede
    /// detectarlo.**
    #[test]
    fn divergent_sibling_between_lanes_is_detected() {
        let path_a = sample_path();
        // Mismo camino salvo el hermano del nivel 5.
        let mut path_b = sample_path();
        path_b.siblings[5] = digest_from(777_777);

        let leaf_a = digest_from(1000);
        let leaf_b = digest_from(2000);

        // Traza "maliciosa": cada carril internamente coherente, caminos
        // distintos. Se construye combinando dos trazas honestas.
        let trace_a = build_trace(leaf_a, leaf_a, &path_a);
        let trace_b = build_trace(leaf_b, leaf_b, &path_b);
        let mut trace = build_trace(leaf_a, leaf_b, &path_a);
        for row in 0..TRACE_LENGTH {
            for col in 0..STATE_WIDTH {
                trace.set(col, row, trace_a.get(col, row));
                trace.set(LANE_B + col, row, trace_b.get(col, row));
            }
        }

        let prover = FrozenClimbProver::new(default_options());

        let prove_result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| prover.prove(trace)));

        match prove_result {
            Err(_) => { /* panic: detectado en modo debug */ }
            Ok(Err(_)) => { /* Err: detectado */ }
            Ok(Ok(proof)) => {
                let min_opts = AcceptableOptions::OptionSet(vec![prover.options().clone()]);
                let verification = verify::<
                    FrozenClimbAir,
                    Blake3,
                    DefaultRandomCoin<Blake3>,
                    MerkleTree<Blake3>,
                >(
                    proof,
                    FrozenClimbPublicInputs {
                        root_a: native_climb(leaf_a, &path_a),
                        root_b: native_climb(leaf_b, &path_b),
                    },
                    &min_opts,
                );
                assert!(
                    verification.is_err(),
                    "CRITICO: dos carriles subiendo por CAMINOS DISTINTOS, cada uno \
                     internamente coherente, deben rechazarse. Si esto verifica, la \
                     restriccion de hermano compartido no comprueba nada y todo el \
                     diseno en lockstep es decorativo."
                );
            }
        }
    }

    /// Declarar una raíz incorrecta debe fallar.
    #[test]
    fn wrong_declared_root_fails_verification() {
        let path = sample_path();
        let leaf_a = digest_from(1000);
        let leaf_b = digest_from(2000);

        let trace = build_trace(leaf_a, leaf_b, &path);
        let prover = FrozenClimbProver::new(default_options());
        let proof = prover.prove(trace).expect("la generacion no deberia fallar");

        let min_opts = AcceptableOptions::OptionSet(vec![prover.options().clone()]);
        let verification =
            verify::<FrozenClimbAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
                proof,
                FrozenClimbPublicInputs {
                    root_a: native_climb(leaf_a, &path),
                    root_b: digest_from(999_999),
                },
                &min_opts,
            );
        assert!(verification.is_err());
    }

    /// Corromper una fila intermedia de hash debe detectarse.
    #[test]
    fn corrupted_intermediate_row_is_detected() {
        let path = sample_path();
        let leaf_a = digest_from(1000);
        let leaf_b = digest_from(2000);
        let mut trace = build_trace(leaf_a, leaf_b, &path);

        let original = trace.get(6, 5 * CYCLE_LENGTH + 3);
        trace.set(6, 5 * CYCLE_LENGTH + 3, original + BaseElement::ONE);

        let prover = FrozenClimbProver::new(default_options());

        let prove_result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| prover.prove(trace)));

        match prove_result {
            Err(_) => {}
            Ok(Err(_)) => {}
            Ok(Ok(proof)) => {
                let min_opts = AcceptableOptions::OptionSet(vec![prover.options().clone()]);
                let verification = verify::<
                    FrozenClimbAir,
                    Blake3,
                    DefaultRandomCoin<Blake3>,
                    MerkleTree<Blake3>,
                >(
                    proof,
                    FrozenClimbPublicInputs {
                        root_a: native_climb(leaf_a, &path),
                        root_b: native_climb(leaf_b, &path),
                    },
                    &min_opts,
                );
                assert!(verification.is_err());
            }
        }
    }

    /// Ninguna restriccion es vacua.
    ///
    /// La herramienta de mutacion perturba la traza celda a celda y comprueba
    /// que **cada** restriccion reacciona a algo. Una que no reaccione nunca
    /// esta declarada, tiene grado asignado y no impone nada.
    ///
    /// AVISO: no detecta el defecto de §38 -una ranura sobrescrita por otro
    /// grupo sigue reaccionando, solo que a la restriccion equivocada-. Para
    /// eso esta `tools/check_constraint_layout.py`. Son dos herramientas
    /// distintas para dos defectos distintos.
    #[test]
    fn no_constraint_is_vacuous() {
        use crate::mutation::{buscar_vacias, rows_of};

        let path = sample_path();
        let leaf_a = digest_from(1000);
        let leaf_b = digest_from(2000);
        let trace = build_trace(leaf_a, leaf_b, &path);
        let rows = rows_of(&trace, TRACE_WIDTH, TRACE_LENGTH);

        let air = FrozenClimbAir::new(
            TraceInfo::new(TRACE_WIDTH, TRACE_LENGTH),
            FrozenClimbPublicInputs {
                root_a: native_climb(leaf_a, &path),
                root_b: native_climb(leaf_b, &path),
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
    fn medicion_130_frozen_climb_legado() {
        use std::time::Instant;
        let t0 = Instant::now();
        let trace = build_trace(digest_from(1000), digest_from(2000), &sample_path());
        let proof = FrozenClimbProver::new(default_options())
            .prove(trace)
            .expect("el honesto debe probar");
        let ms = t0.elapsed().as_secs_f64() * 1000.0;
        println!(
            "[§130] frozen_climb legado: prove {ms:.1} ms, proof {} bytes",
            proof.to_bytes().len()
        );
    }
}
