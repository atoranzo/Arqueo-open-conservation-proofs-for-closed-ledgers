//! Árbol de Merkle de 32 niveles verificado en AIR, encadenando la
//! permutación Rescue ya validada en `rescue_hash.rs`.
//!
//! ## El salto de complejidad respecto a Halo2
//!
//! En Halo2 cada nivel del árbol era una región independiente donde se
//! invocaba el gadget de Poseidon. En AIR **no existe esa modularidad**:
//! todo vive en UNA sola traza y las restricciones se aplican
//! uniformemente entre cada par de filas consecutivas. Eso obliga a un
//! diseño con dos tipos de fila que se alternan, distinguidos por una
//! COLUMNA PERIÓDICA que actúa de selector:
//!
//! - **Filas de hash** (posiciones 0..6 de cada ciclo): aplican una ronda
//!   de Rescue, exactamente como en `rescue_hash.rs`.
//! - **Fila de enlace** (posición 7): toma el digest recién calculado, lo
//!   coloca a izquierda o derecha según el bit de dirección, y prepara el
//!   estado inicial del siguiente nivel.
//!
//! ## Diseño de la traza
//!
//! 13 columnas × 256 filas (32 niveles × 8 filas por nivel):
//! - Columnas 0..11: estado de Rescue (4 de capacidad + 8 de rate).
//! - Columna 12: bit de dirección del nivel actual (0 = el nodo va a la
//!   izquierda, 1 = a la derecha).
//!
//! ## Por qué profundidad 32 y no 20
//!
//! Winterfell exige que la traza tenga longitud potencia de dos. Con 8
//! filas por nivel, la profundidad 20 daría 160 filas, que no lo es. Las
//! opciones viables eran 16 (128 filas) o 32 (256 filas). Se eligió 32:
//! 2^32 ≈ 4.300 millones de cuentas, MÁS que la profundidad 20 de los
//! backends Groth16 y Halo2. Es una diferencia real entre backends, no
//! una equivalencia — documentada como tal.
//!
//! ## Qué demuestra
//!
//! Conocimiento de una hoja y un camino de autenticación tales que,
//! hasheando hacia arriba 32 niveles, se obtiene la raíz pública. La hoja
//! y el camino permanecen PRIVADOS.

use winterfell::crypto::hashers::{Blake3_256, Rp64_256};
use winterfell::crypto::{DefaultRandomCoin, MerkleTree};
use winterfell::math::{fields::f64::BaseElement, FieldElement, ToElements};
use winterfell::matrix::ColMatrix;
use winterfell::{
    Air, AirContext, Assertion, AuxRandElements, CompositionPoly, CompositionPolyTrace,
    ConstraintCompositionCoefficients, DefaultConstraintCommitment, DefaultConstraintEvaluator,
    DefaultTraceLde, EvaluationFrame, PartitionOptions, ProofOptions, Prover, StarkDomain, Trace,
    TraceInfo, TracePolyTable, TraceTable, TransitionConstraintDegree,
};

use crate::rescue_hash::{apply_sbox, NUM_ROUNDS, STATE_WIDTH};

/// Profundidad del árbol. Ver la nota de arriba sobre por qué 32.
pub const TREE_DEPTH: usize = 32;

/// **Profundidad del árbol de NULLIFICADORES, separada a propósito.**
///
/// Hasta ahora compartía valor con `TREE_DEPTH`, y eso ocultaba que los dos
/// árboles tienen exigencias distintas:
///
/// | Árbol | Posición | Consecuencia |
/// |---|---|---|
/// | Cuentas | **Se asigna** secuencialmente | 2³² cuentas, sin colisiones |
/// | Nullificadores | **Se deriva** del propio nullificador | Paradoja del cumpleaños |
///
/// Con 32 bits, dos nullificadores distintos caen en la misma posición con
/// un 39 % de probabilidad a los **65.536 pagos**, y el afectado **no puede
/// reintentar**: su nullificador es determinista. Ver `AUDITORIA.md` §13.
///
/// ⚠️ **Sigue valiendo 32.** Esta constante existe para poder subirla sin
/// tocar el árbol de cuentas, que no tiene el problema. Subirla exige
/// recalcular las constantes de fila de cuatro circuitos y llevar sus
/// trazas de 1024 a 4096 filas: **es el paso 2, y no está hecho.**
pub const NULLIFIER_DEPTH: usize = 32;
/// Filas por nivel: 7 rondas de hash + 1 fila de enlace.
pub const CYCLE_LENGTH: usize = 8;
/// Longitud total de la traza: 32 × 8 = 256 (potencia de dos ✓).
pub const TRACE_LENGTH: usize = TREE_DEPTH * CYCLE_LENGTH;
/// Ancho: 12 del estado de Rescue + 1 del bit de dirección.
pub const TRACE_WIDTH: usize = STATE_WIDTH + 1;
/// Índice de la columna del bit de dirección.
const BIT_COL: usize = STATE_WIDTH;

type Blake3 = Blake3_256<BaseElement>;

/// Digest de 4 elementos, la unidad de trabajo del árbol.
pub type Digest = [BaseElement; 4];

/// Hash 2-a-1 nativo, usando la implementación real de `winter-crypto`.
pub fn native_merge(left: Digest, right: Digest) -> Digest {
    let mut state = [BaseElement::ZERO; STATE_WIDTH];
    state[4..8].copy_from_slice(&left);
    state[8..12].copy_from_slice(&right);
    Rp64_256::apply_permutation(&mut state);
    [state[4], state[5], state[6], state[7]]
}

/// Camino de autenticación: un hermano y un bit de dirección por nivel.
#[derive(Clone, Debug)]
pub struct MerklePath {
    pub siblings: Vec<Digest>,
    /// `false` = el nodo actual va a la izquierda, `true` = a la derecha.
    pub is_right: Vec<bool>,
}

/// Calcula la raíz de forma nativa siguiendo el camino, para saber qué
/// esperar sin adivinarlo.
pub fn native_root(leaf: Digest, path: &MerklePath) -> Digest {
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

/// Construye la traza completa: 32 niveles encadenados.
pub fn build_trace(leaf: Digest, path: &MerklePath) -> TraceTable<BaseElement> {
    assert_eq!(path.siblings.len(), TREE_DEPTH);
    assert_eq!(path.is_right.len(), TREE_DEPTH);

    let siblings = path.siblings.clone();
    let is_right = path.is_right.clone();

    let mut trace = TraceTable::new(TRACE_WIDTH, TRACE_LENGTH);

    trace.fill(
        |state| {
            // Fila 0: estado inicial del nivel 0.
            for s in state.iter_mut() {
                *s = BaseElement::ZERO;
            }
            if is_right[0] {
                state[4..8].copy_from_slice(&siblings[0]);
                state[8..12].copy_from_slice(&leaf);
            } else {
                state[4..8].copy_from_slice(&leaf);
                state[8..12].copy_from_slice(&siblings[0]);
            }
            state[BIT_COL] = if is_right[0] {
                BaseElement::ONE
            } else {
                BaseElement::ZERO
            };
        },
        |step, state| {
            let position_in_cycle = step % CYCLE_LENGTH;

            if position_in_cycle < NUM_ROUNDS {
                // --- Fila de hash: aplicar una ronda de Rescue ---
                let mut arr: [BaseElement; STATE_WIDTH] =
                    state[..STATE_WIDTH].try_into().unwrap();
                Rp64_256::apply_round(&mut arr, position_in_cycle);
                state[..STATE_WIDTH].copy_from_slice(&arr);
                // El bit no cambia dentro del ciclo.
            } else {
                // --- Fila de enlace: preparar el siguiente nivel ---
                let digest: Digest = [state[4], state[5], state[6], state[7]];
                let next_level = (step + 1) / CYCLE_LENGTH;

                for s in state.iter_mut() {
                    *s = BaseElement::ZERO;
                }

                if next_level < TREE_DEPTH {
                    if is_right[next_level] {
                        state[4..8].copy_from_slice(&siblings[next_level]);
                        state[8..12].copy_from_slice(&digest);
                    } else {
                        state[4..8].copy_from_slice(&digest);
                        state[8..12].copy_from_slice(&siblings[next_level]);
                    }
                    state[BIT_COL] = if is_right[next_level] {
                        BaseElement::ONE
                    } else {
                        BaseElement::ZERO
                    };
                }
            }
        },
    );

    trace
}

/// Inputs públicos: solo la raíz. La hoja y el camino son privados.
#[derive(Clone, Debug)]
pub struct MerklePublicInputs {
    pub root: Digest,
}

impl ToElements<BaseElement> for MerklePublicInputs {
    fn to_elements(&self) -> Vec<BaseElement> {
        self.root.to_vec()
    }
}

pub struct MerkleAir {
    context: AirContext<BaseElement>,
    root: Digest,
}

impl Air for MerkleAir {
    type BaseField = BaseElement;
    type PublicInputs = MerklePublicInputs;

    fn new(trace_info: TraceInfo, pub_inputs: Self::PublicInputs, options: ProofOptions) -> Self {
        assert_eq!(TRACE_WIDTH, trace_info.width());

        let mut degrees = Vec::new();
        // 12 restricciones de hash (grado 7 por la S-box, moduladas por
        // el selector periódico de ciclo 8).
        for _ in 0..STATE_WIDTH {
            degrees.push(TransitionConstraintDegree::with_cycles(7, vec![CYCLE_LENGTH]));
        }
        // 4 restricciones de reinicio de capacidad en la fila de enlace.
        for _ in 0..4 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, vec![CYCLE_LENGTH]));
        }
        // 4 restricciones de colocación del digest según el bit.
        for _ in 0..4 {
            degrees.push(TransitionConstraintDegree::with_cycles(2, vec![CYCLE_LENGTH]));
        }
        // 1 restricción de que el bit es booleano.
        degrees.push(TransitionConstraintDegree::new(2));

        MerkleAir {
            // 8 aserciones: 4 de capacidad inicial + 4 de la raíz final.
            context: AirContext::new(trace_info, degrees, 8, options),
            root: pub_inputs.root,
        }
    }

    fn context(&self) -> &AirContext<Self::BaseField> {
        &self.context
    }

    /// Columnas periódicas: 1 selector de hash + 12 de ARK1 + 12 de ARK2.
    /// Todas de longitud 8 (la del ciclo).
    fn get_periodic_column_values(&self) -> Vec<Vec<Self::BaseField>> {
        let mut columns = Vec::with_capacity(1 + 2 * STATE_WIDTH);

        // Selector: 1 en las filas de hash (0..6), 0 en la de enlace (7).
        let mut hash_flag = vec![BaseElement::ONE; NUM_ROUNDS];
        hash_flag.push(BaseElement::ZERO);
        columns.push(hash_flag);

        for i in 0..STATE_WIDTH {
            let mut col: Vec<BaseElement> =
                (0..NUM_ROUNDS).map(|r| Rp64_256::ARK1[r][i]).collect();
            col.push(BaseElement::ZERO);
            columns.push(col);
        }
        for i in 0..STATE_WIDTH {
            let mut col: Vec<BaseElement> =
                (0..NUM_ROUNDS).map(|r| Rp64_256::ARK2[r][i]).collect();
            col.push(BaseElement::ZERO);
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

        // ============ Restricciones de hash (activas si hash_flag = 1) ============
        // Misma técnica de "encontrarse en el medio" que en rescue_hash.rs.
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
            result[i] = hash_flag * (apply_sbox(b[i]) - a[i]);
        }

        // ============ Restricciones de enlace (activas si hash_flag = 0) ============
        let link_flag = E::ONE - hash_flag;

        // La capacidad del siguiente nivel debe reiniciarse a cero.
        for i in 0..4 {
            result[STATE_WIDTH + i] = link_flag * next[i];
        }

        // El digest recién calculado (current[4..8]) debe colocarse en la
        // mitad correcta del siguiente estado según el bit de dirección:
        // - bit = 0 → va a las posiciones 4..8 (izquierda)
        // - bit = 1 → va a las posiciones 8..12 (derecha)
        //
        // ⚠️ EL BIT SE LEE DE `next`, NO DE `current`. Corregido tras un
        // fallo real ("main transition constraint 16 did not evaluate to
        // ZERO at step 7"): `is_right[level]` determina dónde va el digest
        // ACUMULADO en ese nivel, así que al enlazar del nivel L al L+1,
        // la posición la decide `is_right[L+1]` — el bit del nivel
        // DESTINO, que es el que vive en la fila siguiente. Leerlo de
        // `current` introducía un desfase de un nivel entre la traza y la
        // restricción que la verifica.
        let bit = next[BIT_COL];
        for i in 0..4 {
            let digest_i = current[4 + i];
            let placed = (E::ONE - bit) * (next[4 + i] - digest_i) + bit * (next[8 + i] - digest_i);
            result[STATE_WIDTH + 4 + i] = link_flag * placed;
        }

        // ============ El bit debe ser booleano, siempre ============
        result[STATE_WIDTH + 8] = current[BIT_COL] * (current[BIT_COL] - E::ONE);
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        let last_step = TRACE_LENGTH - 1;
        let mut assertions = Vec::with_capacity(8);

        // La capacidad inicial debe ser cero (seguridad de la esponja).
        for i in 0..4 {
            assertions.push(Assertion::single(i, 0, BaseElement::ZERO));
        }
        // La raíz reconstruida debe coincidir con la pública.
        for i in 0..4 {
            assertions.push(Assertion::single(4 + i, last_step, self.root[i]));
        }

        assertions
    }
}

pub struct MerkleProver {
    options: ProofOptions,
}

impl MerkleProver {
    pub fn new(options: ProofOptions) -> Self {
        Self { options }
    }
}

impl Prover for MerkleProver {
    type BaseField = BaseElement;
    type Air = MerkleAir;
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

    fn get_pub_inputs(&self, trace: &Self::Trace) -> MerklePublicInputs {
        let last_step = trace.length() - 1;
        MerklePublicInputs {
            root: [
                trace.get(4, last_step),
                trace.get(5, last_step),
                trace.get(6, last_step),
                trace.get(7, last_step),
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
    use winterfell::{verify, AcceptableOptions, BatchingMethod, FieldExtension, Proof};

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

    fn sample_path() -> (Digest, MerklePath) {
        let leaf = digest_from(1000);
        let siblings: Vec<Digest> = (0..TREE_DEPTH).map(|i| digest_from(i as u64 * 10)).collect();
        // Patrón alterno de direcciones, para ejercitar ambos casos.
        let is_right: Vec<bool> = (0..TREE_DEPTH).map(|i| i % 3 == 0).collect();
        (leaf, MerklePath { siblings, is_right })
    }

    /// Primer test, el más informativo si falla: la traza construida debe
    /// terminar con la misma raíz que calcula la versión nativa.
    #[test]
    fn trace_final_root_matches_native_computation() {
        let (leaf, path) = sample_path();
        let trace = build_trace(leaf, &path);
        let expected = native_root(leaf, &path);

        let last = TRACE_LENGTH - 1;
        for i in 0..4 {
            assert_eq!(
                trace.get(4 + i, last),
                expected[i],
                "el elemento {i} de la raiz en la traza no coincide con el calculo nativo"
            );
        }
    }

    /// EL TEST CLAVE: pertenencia al árbol demostrada con una prueba STARK
    /// real, sin revelar la hoja ni el camino.
    #[test]
    fn valid_merkle_path_produces_verifiable_proof() {
        let (leaf, path) = sample_path();
        let trace = build_trace(leaf, &path);
        let expected_root = native_root(leaf, &path);

        let prover = MerkleProver::new(default_options());
        let proof: Proof = prover
            .prove(trace)
            .expect("la generacion de la prueba no deberia fallar");

        let min_opts = AcceptableOptions::OptionSet(vec![prover.options().clone()]);
        let verification = verify::<MerkleAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof,
            MerklePublicInputs { root: expected_root },
            &min_opts,
        );

        assert!(
            verification.is_ok(),
            "un camino de Merkle valido deberia verificar: {verification:?}"
        );
    }

    /// TEST DE SOLIDEZ: declarar una raíz que no corresponde debe fallar.
    #[test]
    fn wrong_declared_root_fails_verification() {
        let (leaf, path) = sample_path();
        let trace = build_trace(leaf, &path);

        let prover = MerkleProver::new(default_options());
        let proof = prover.prove(trace).expect("la generacion no deberia fallar");

        let min_opts = AcceptableOptions::OptionSet(vec![prover.options().clone()]);
        let verification = verify::<MerkleAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof,
            MerklePublicInputs {
                root: digest_from(999_999),
            },
            &min_opts,
        );

        assert!(
            verification.is_err(),
            "CRITICO: una raiz declarada incorrecta no deberia verificar"
        );
    }

    /// TEST DISCRIMINANTE: corromper una fila intermedia debe detectarse.
    /// Sin esto no sabríamos si las restricciones comprueban algo real.
    ///
    /// La detección puede manifestarse de TRES formas legítimas, y las
    /// tres son correctas:
    /// 1. `prove` hace panic (modo debug: winterfell tiene una assertion
    ///    interna que comprueba la traza antes de generar la prueba),
    /// 2. `prove` devuelve `Err`,
    /// 3. genera una prueba que luego no verifica.
    ///
    /// Lo ÚNICO inaceptable sería que la prueba verificara correctamente.
    #[test]
    fn corrupted_intermediate_row_is_detected() {
        let (leaf, path) = sample_path();
        let mut trace = build_trace(leaf, &path);

        // Corromper el estado en mitad del hash del nivel 5.
        let original = trace.get(6, 5 * CYCLE_LENGTH + 3);
        trace.set(6, 5 * CYCLE_LENGTH + 3, original + BaseElement::ONE);

        let expected_root = native_root(leaf, &path);
        let prover = MerkleProver::new(default_options());

        // Silenciar el mensaje de panic esperado, para no ensuciar la
        // salida de los tests.
        let prove_result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| prover.prove(trace)));

        match prove_result {
            Err(_) => { /* panic: la traza invalida se detecto (modo debug) */ }
            Ok(Err(_)) => { /* Err: detectado */ }
            Ok(Ok(proof)) => {
                let min_opts = AcceptableOptions::OptionSet(vec![prover.options().clone()]);
                let verification =
                    verify::<MerkleAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
                        proof,
                        MerklePublicInputs { root: expected_root },
                        &min_opts,
                    );
                assert!(
                    verification.is_err(),
                    "CRITICO: una traza con una fila intermedia corrompida no deberia verificar"
                );
            }
        }
    }
}
