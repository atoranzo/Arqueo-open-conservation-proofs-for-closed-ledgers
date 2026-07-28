//! **No-pertenencia demostrable del nullifier en AIR.**
//!
//! Port a STARK de `zk-core::nullifier_tree`, y la pieza que permite que
//! el doble gasto sea matemáticamente imposible en vez de "detectable por
//! una base de datos externa".
//!
//! ## Por qué esta pieza casi estaba hecha
//!
//! `dual_climb` demuestra que **dos hojas suben por el MISMO camino** —
//! mismos hermanos, mismas direcciones— hasta sus respectivas raíces, con
//! el hermano compartido impuesto por restricción (diseño en lockstep).
//!
//! La no-pertenencia más inserción es exactamente eso:
//!
//! ```text
//! carril A: hoja CERO      → root_old   (la posición estaba vacía)
//! carril B: hoja NULLIFIER → root_new   (queda insertado)
//! ```
//!
//! El mismo camino ata ambas raíces a la MISMA posición del árbol. Sin la
//! restricción de hermano compartido, un probador podría usar caminos
//! distintos y fabricar una raíz nueva que no corresponde a la posición
//! donde comprobó que no había nada — el agujero silencioso que
//! `dual_climb` cierra.
//!
//! ## Lo que se añade respecto a `dual_climb`
//!
//! Dos restricciones sobre la fila 0, que son las que convierten una
//! subida dual genérica en una prueba de no-pertenencia:
//!
//! 1. La hoja del carril A es **cero** (la posición estaba libre).
//! 2. La hoja del carril B es **el nullifier** declarado.
//!
//! La sutileza: qué mitad del estado ocupa la hoja depende del bit de
//! dirección, así que ambas restricciones seleccionan con el mismo bit —
//! igual que la colocación del digest.
//!
//! ## Limitación heredada: colisiones de posición
//!
//! La posición sale de los bits bajos del nullifier. Dos nullifiers
//! pueden colisionar y el segundo no podría gastarse: **denegación de
//! servicio, no doble gasto**. Ver la nota extensa en
//! `zk-core::nullifier_tree`.

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
use crate::rescue_hash::{apply_sbox, NUM_ROUNDS, STATE_WIDTH};

pub const CYCLE_LENGTH: usize = 8;
pub const TRACE_LENGTH: usize = TREE_DEPTH * CYCLE_LENGTH; // 256
/// 12 del carril A + 12 del carril B + bit + 4 de transporte del nullifier.
pub const TRACE_WIDTH: usize = 2 * STATE_WIDTH + 1 + 4;

const LANE_B: usize = STATE_WIDTH;
const COL_BIT: usize = 2 * STATE_WIDTH; // 24
const COL_NULL: usize = 25; // 25..29

type Blake3 = Blake3_256<BaseElement>;

/// Digest cero: la hoja de una posición libre.
pub fn empty_leaf() -> Digest {
    [BaseElement::ZERO; 4]
}

/// Hashes de subárboles vacíos por nivel.
pub fn empty_subtrees() -> Vec<Digest> {
    let mut empty = vec![empty_leaf()];
    for k in 1..=TREE_DEPTH {
        let prev = empty[k - 1];
        empty.push(native_merge(prev, prev));
    }
    empty
}

/// Raíz de un árbol de nullifiers completamente vacío.
pub fn empty_root() -> Digest {
    empty_subtrees()[TREE_DEPTH]
}

/// Posición de un nullifier: sus bits bajos.
pub fn nullifier_position(nullifier: &Digest) -> u64 {
    // Se usa el primer elemento del digest; con TREE_DEPTH bits basta
    // para indexar el árbol.
    let v = nullifier[0].as_int();
    v & ((1u64 << TREE_DEPTH) - 1)
}

/// Camino de una posición en un árbol vacío.
pub fn path_for_empty_tree(position: u64) -> MerklePath {
    let empty = empty_subtrees();
    let mut siblings = Vec::with_capacity(TREE_DEPTH);
    let mut is_right = Vec::with_capacity(TREE_DEPTH);
    let mut idx = position;
    for level in 0..TREE_DEPTH {
        siblings.push(empty[level]);
        is_right.push(idx % 2 == 1);
        idx /= 2;
    }
    MerklePath { siblings, is_right }
}

/// Sube una hoja por el camino, de forma nativa.
pub fn climb(leaf: Digest, path: &MerklePath) -> Digest {
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

/// Construye la traza: carril A desde cero, carril B desde el nullifier,
/// ambos por el MISMO camino.
pub fn build_trace(nullifier: Digest, path: &MerklePath) -> TraceTable<BaseElement> {
    build_trace_with_leaves(empty_leaf(), nullifier, nullifier, path)
}

/// Constructor con las hojas de ambos carriles explícitas.
///
/// Existe para poder construir trazas MALICIOSAS internamente coherentes
/// en los tests: un carril que sube correctamente desde una hoja que no
/// es la que las restricciones exigen. Sin esto, el test discriminante
/// no discrimina — la corrupción directa de la fila 0 la detectaría
/// también la restricción de hash, y no sabríamos si la de
/// no-pertenencia hace su trabajo.
pub fn build_trace_with_leaves(
    leaf_a: Digest,
    leaf_b: Digest,
    nullifier: Digest,
    path: &MerklePath,
) -> TraceTable<BaseElement> {
    assert_eq!(path.siblings.len(), TREE_DEPTH);

    let zero = BaseElement::ZERO;
    let mut rows: Vec<Vec<BaseElement>> = vec![vec![zero; TRACE_WIDTH]; TRACE_LENGTH];

    // Transporte del nullifier, constante en toda la traza.
    for row in rows.iter_mut() {
        for i in 0..4 {
            row[COL_NULL + i] = nullifier[i];
        }
    }

    let place = |state: &mut [BaseElement; STATE_WIDTH], digest: &Digest, level: usize| {
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
            let digest_a: Digest = [state_a[4], state_a[5], state_a[6], state_a[7]];
            let digest_b: Digest = [state_b[4], state_b[5], state_b[6], state_b[7]];
            state_a = [zero; STATE_WIDTH];
            state_b = [zero; STATE_WIDTH];
            let next_level = (r + 1) / CYCLE_LENGTH;
            if next_level < TREE_DEPTH {
                place(&mut state_a, &digest_a, next_level);
                place(&mut state_b, &digest_b, next_level);
            }
        }
        rows[r + 1][..STATE_WIDTH].copy_from_slice(&state_a);
        rows[r + 1][LANE_B..LANE_B + STATE_WIDTH].copy_from_slice(&state_b);
    }

    for level in 0..TREE_DEPTH {
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

/// Inputs públicos: las dos raíces y el nullifier.
///
/// El nullifier ES público, y debe serlo: quien aplica la operación lo
/// necesita para mantener su propio árbol. Al derivarse de la clave de
/// gasto es indistinguible, así que publicarlo no revela nada. Es el
/// mismo razonamiento (y el mismo error corregido) que en `zk-core`.
#[derive(Clone, Debug)]
pub struct NullifierPublicInputs {
    pub root_old: Digest,
    pub root_new: Digest,
    pub nullifier: Digest,
}

impl ToElements<BaseElement> for NullifierPublicInputs {
    fn to_elements(&self) -> Vec<BaseElement> {
        let mut out = self.root_old.to_vec();
        out.extend_from_slice(&self.root_new);
        out.extend_from_slice(&self.nullifier);
        out
    }
}

pub struct NullifierTreeAir {
    context: AirContext<BaseElement>,
    pub_inputs: NullifierPublicInputs,
}

impl Air for NullifierTreeAir {
    type BaseField = BaseElement;
    type PublicInputs = NullifierPublicInputs;

    fn new(trace_info: TraceInfo, pub_inputs: Self::PublicInputs, options: ProofOptions) -> Self {
        assert_eq!(TRACE_WIDTH, trace_info.width());
        let full = vec![TRACE_LENGTH];

        let mut degrees = Vec::new();
        // 0..23: rondas de Rescue de ambos carriles.
        for _ in 0..2 * STATE_WIDTH {
            degrees.push(TransitionConstraintDegree::with_cycles(7, vec![CYCLE_LENGTH]));
        }
        // 24..31: reinicio de capacidad.
        for _ in 0..8 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, vec![CYCLE_LENGTH]));
        }
        // 32..39: colocación del digest (grado 2).
        // 40..43: hermano compartido (grado 2).
        for _ in 0..12 {
            degrees.push(TransitionConstraintDegree::with_cycles(2, vec![CYCLE_LENGTH]));
        }
        // 44: bit booleano.
        degrees.push(TransitionConstraintDegree::new(2));
        // 45..48: la hoja del carril A es CERO (no-pertenencia).
        // 49..52: la hoja del carril B es el nullifier (inserción).
        for _ in 0..8 {
            degrees.push(TransitionConstraintDegree::with_cycles(2, full.clone()));
        }
        // 53..56: constancia del transporte del nullifier.
        for _ in 0..4 {
            degrees.push(TransitionConstraintDegree::new(1));
        }

        NullifierTreeAir {
            // 8 aserciones: las dos raices finales.
            context: AirContext::new(trace_info, degrees, 8, options),
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

        let mut hash_flag = vec![one; NUM_ROUNDS];
        hash_flag.push(zero);
        columns.push(hash_flag);

        for ark in [true, false] {
            for i in 0..STATE_WIDTH {
                let mut col: Vec<BaseElement> = (0..NUM_ROUNDS)
                    .map(|r| {
                        if ark {
                            Rp64_256::ARK1[r][i]
                        } else {
                            Rp64_256::ARK2[r][i]
                        }
                    })
                    .collect();
                col.push(zero);
                columns.push(col);
            }
        }

        // Selector de la primera fila, de longitud completa.
        let mut first_row = vec![zero; TRACE_LENGTH];
        first_row[0] = one;
        columns.push(first_row);

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

        let hash_flag = periodic[0];
        let ark1 = &periodic[1..1 + STATE_WIDTH];
        let ark2 = &periodic[1 + STATE_WIDTH..1 + 2 * STATE_WIDTH];
        let first_row = periodic[1 + 2 * STATE_WIDTH];
        let link_flag = E::ONE - hash_flag;

        // ===== Rondas de Rescue en ambos carriles =====
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

        let bit = next[COL_BIT];

        // ===== Reinicio de capacidad =====
        for i in 0..4 {
            result[24 + i] = link_flag * next[i];
            result[28 + i] = link_flag * next[LANE_B + i];
        }

        // ===== Colocación del digest =====
        for i in 0..4 {
            let da = current[4 + i];
            let placed_a = (E::ONE - bit) * (next[4 + i] - da) + bit * (next[8 + i] - da);
            result[32 + i] = link_flag * placed_a;

            let db = current[LANE_B + 4 + i];
            let placed_b =
                (E::ONE - bit) * (next[LANE_B + 4 + i] - db) + bit * (next[LANE_B + 8 + i] - db);
            result[36 + i] = link_flag * placed_b;
        }

        // ===== EL HERMANO ES EL MISMO EN AMBOS CARRILES =====
        for i in 0..4 {
            let sib_a = (E::ONE - bit) * next[8 + i] + bit * next[4 + i];
            let sib_b =
                (E::ONE - bit) * next[LANE_B + 8 + i] + bit * next[LANE_B + 4 + i];
            result[40 + i] = link_flag * (sib_a - sib_b);
        }

        result[44] = current[COL_BIT] * (current[COL_BIT] - E::ONE);

        // ===== LO QUE CONVIERTE ESTO EN NO-PERTENENCIA + INSERCIÓN =====
        //
        // En la fila 0, la hoja ocupa la mitad del estado que indique el
        // bit de ESA fila (no el de la siguiente: aquí no hay transición
        // previa que la coloque).
        let bit0 = current[COL_BIT];
        for i in 0..4 {
            // Carril A: la hoja es CERO. La posición estaba libre.
            let leaf_a = (E::ONE - bit0) * current[4 + i] + bit0 * current[8 + i];
            result[45 + i] = first_row * leaf_a;

            // Carril B: la hoja es el nullifier declarado.
            let leaf_b =
                (E::ONE - bit0) * current[LANE_B + 4 + i] + bit0 * current[LANE_B + 8 + i];
            result[49 + i] = first_row * (leaf_b - current[COL_NULL + i]);
        }

        // ===== Constancia del transporte =====
        for i in 0..4 {
            result[53 + i] = next[COL_NULL + i] - current[COL_NULL + i];
        }
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        let last = TRACE_LENGTH - 1;
        let mut a = Vec::with_capacity(8);
        for i in 0..4 {
            a.push(Assertion::single(4 + i, last, self.pub_inputs.root_old[i]));
            a.push(Assertion::single(
                LANE_B + 4 + i,
                last,
                self.pub_inputs.root_new[i],
            ));
        }
        a
    }
}

pub struct NullifierTreeProver {
    options: ProofOptions,
}

impl NullifierTreeProver {
    pub fn new(options: ProofOptions) -> Self {
        Self { options }
    }
}

impl Prover for NullifierTreeProver {
    type BaseField = BaseElement;
    type Air = NullifierTreeAir;
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

    fn get_pub_inputs(&self, trace: &Self::Trace) -> NullifierPublicInputs {
        use winterfell::Trace;
        let last = trace.length() - 1;
        NullifierPublicInputs {
            root_old: [
                trace.get(4, last),
                trace.get(5, last),
                trace.get(6, last),
                trace.get(7, last),
            ],
            root_new: [
                trace.get(LANE_B + 4, last),
                trace.get(LANE_B + 5, last),
                trace.get(LANE_B + 6, last),
                trace.get(LANE_B + 7, last),
            ],
            nullifier: [
                trace.get(COL_NULL, 0),
                trace.get(COL_NULL + 1, 0),
                trace.get(COL_NULL + 2, 0),
                trace.get(COL_NULL + 3, 0),
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

    fn sample_nullifier(n: u64) -> Digest {
        [
            BaseElement::new(n),
            BaseElement::new(n + 1),
            BaseElement::new(n + 2),
            BaseElement::new(n + 3),
        ]
    }

    /// La raíz de un árbol vacío coincide con subir una hoja cero.
    #[test]
    fn empty_tree_is_consistent() {
        let path = path_for_empty_tree(12345);
        assert_eq!(climb(empty_leaf(), &path), empty_root());
    }

    /// La traza debe terminar con las raíces que calcula la versión
    /// nativa: el test más informativo si algo falla.
    #[test]
    fn trace_roots_match_native() {
        let nullifier = sample_nullifier(0xABCDEF);
        let path = path_for_empty_tree(nullifier_position(&nullifier));
        let trace = build_trace(nullifier, &path);

        let expected_old = climb(empty_leaf(), &path);
        let expected_new = climb(nullifier, &path);
        let last = TRACE_LENGTH - 1;
        for i in 0..4 {
            assert_eq!(trace.get(4 + i, last), expected_old[i], "root_old elem {i}");
            assert_eq!(
                trace.get(LANE_B + 4 + i, last),
                expected_new[i],
                "root_new elem {i}"
            );
        }
        // ===== Y TODAS LAS ENTRADAS PÚBLICAS, NO SOLO LAS RAÍCES =====
        //
        // Comparar la estructura entera. En `circuit_send` la versión
        // parcial dejó pasar un campo heredado de otra operación y **costó
        // ocho rondas de diagnóstico**: el error de winterfell
        // —`InconsistentOodConstraintEvaluations`— apunta a las
        // restricciones, no a las entradas.
        let derivadas = NullifierTreeProver::new(default_options()).get_pub_inputs(&trace);
        assert_eq!(
            derivadas.to_elements(),
            NullifierPublicInputs {
                root_old: expected_old,
                root_new: expected_new,
                nullifier,
            }.to_elements(),
            "las entradas DERIVADAS de la traza deben coincidir con las \
             DECLARADAS en todos sus campos"
        );

    }

    /// EL TEST CLAVE: insertar un nullifier no gastado verifica.
    #[test]
    fn inserting_unspent_nullifier_verifies() {
        let nullifier = sample_nullifier(0xABCDEF);
        let path = path_for_empty_tree(nullifier_position(&nullifier));
        let trace = build_trace(nullifier, &path);

        let prover = NullifierTreeProver::new(default_options());
        let proof = prover.prove(trace).expect("prove");

        let min_opts = AcceptableOptions::OptionSet(vec![prover.options().clone()]);
        let verification =
            verify::<NullifierTreeAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
                proof,
                NullifierPublicInputs {
                    root_old: climb(empty_leaf(), &path),
                    root_new: climb(nullifier, &path),
                    nullifier,
                },
                &min_opts,
            );
        assert!(verification.is_ok(), "{verification:?}");
    }

    /// **EL TEST QUE CIERRA EL DOBLE GASTO.**
    ///
    /// ## Por qué la primera versión de este test no valía
    ///
    /// Corrompía la fila 0 directamente. Eso rompe DOS cosas a la vez: la
    /// restricción de no-pertenencia y las de hash (el estado alterado
    /// alimenta la ronda siguiente). El test pasaba aunque la restricción
    /// clave no hiciera nada. Confianza falsa — el mismo error que ya se
    /// coló una vez en `dual_climb`.
    ///
    /// ## El ataque real
    ///
    /// Se construye una traza donde el carril A sube **coherentemente
    /// desde el nullifier** en vez de desde cero: todos sus hashes son
    /// correctos respecto a esa hoja. Es decir, se afirma que la posición
    /// ya estaba ocupada por el propio nullifier — el testigo que
    /// necesitaría alguien para gastarlo por segunda vez.
    ///
    /// **Solo la restricción de no-pertenencia puede detectarlo.**
    #[test]
    fn occupied_position_cannot_prove_non_membership() {
        let nullifier = sample_nullifier(0xABCDEF);
        let path = path_for_empty_tree(nullifier_position(&nullifier));

        // Traza internamente COHERENTE, pero con el carril A arrancando
        // desde el nullifier en vez de desde cero.
        let trace = build_trace_with_leaves(nullifier, nullifier, nullifier, &path);

        let prover = NullifierTreeProver::new(default_options());
        let previous_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| prover.prove(trace)));
        std::panic::set_hook(previous_hook);

        match r {
            Err(_) => { /* panic: detectado en debug */ }
            Ok(Err(_)) => { /* Err: detectado */ }
            Ok(Ok(proof)) => {
                let min_opts = AcceptableOptions::OptionSet(vec![prover.options().clone()]);
                let v = verify::<
                    NullifierTreeAir,
                    Blake3,
                    DefaultRandomCoin<Blake3>,
                    MerkleTree<Blake3>,
                >(
                    proof,
                    NullifierPublicInputs {
                        root_old: climb(nullifier, &path),
                        root_new: climb(nullifier, &path),
                        nullifier,
                    },
                    &min_opts,
                );
                assert!(
                    v.is_err(),
                    "CRITICO: una traza internamente coherente que afirma \
                     no-pertenencia en una posicion OCUPADA debe rechazarse. \
                     Si verifica, la restriccion de no-pertenencia no comprueba \
                     nada y el doble gasto es posible."
                );
            }
        }
    }

    /// Declarar un nullifier distinto al de la traza debe fallar.
    #[test]
    fn wrong_declared_nullifier_fails() {
        let nullifier = sample_nullifier(0xABCDEF);
        let path = path_for_empty_tree(nullifier_position(&nullifier));
        let trace = build_trace(nullifier, &path);

        let prover = NullifierTreeProver::new(default_options());
        let proof = prover.prove(trace).expect("prove");

        let min_opts = AcceptableOptions::OptionSet(vec![prover.options().clone()]);
        let v = verify::<NullifierTreeAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof,
            NullifierPublicInputs {
                root_old: climb(empty_leaf(), &path),
                root_new: climb(nullifier, &path),
                nullifier: sample_nullifier(999_999),
            },
            &min_opts,
        );
        assert!(v.is_err());
    }
}
