//! Hash Rescue Prime verificado dentro de una traza AIR — la pieza más
//! laboriosa del port a STARK, y la única forma disponible de tener un
//! hash algebraico en Winterfell (no existe gadget reutilizable; ver la
//! investigación documentada en el README).
//!
//! ## ✅ Cero constantes criptográficas inventadas
//!
//! Todas las constantes provienen de `Rp64_256` en `winter-crypto`, que
//! las reexporta públicamente: `MDS`, `INV_MDS`, `ARK1`, `ARK2`,
//! `NUM_ROUNDS`, `STATE_WIDTH`. Los únicos dos valores escritos como
//! literales son los exponentes de la S-box, que son privados en el
//! crate pero verificables en su código fuente:
//! `winter-crypto-0.13.1/src/hash/rescue/rp64_256/mod.rs`, líneas 52 y 54
//! (`ALPHA = 7`, `INV_ALPHA = 10540996611094048183`).
//!
//! Esto es exactamente lo contrario de lo que hicimos con `toy_hash` en
//! `zk-core`: allí NO había parámetros verificables disponibles y por eso
//! se usó un placeholder honesto; aquí sí los hay y se usan tal cual.
//!
//! ## La técnica: "encontrarse en el medio" (evita la S-box inversa)
//!
//! Una ronda de Rescue (algoritmo 3 del paper, eprint 2020/1143) es:
//! 1. `state = sbox(state)` — elevar a ALPHA=7
//! 2. `state = MDS * state`
//! 3. `state = state + ARK1[r]`
//! 4. `state = inv_sbox(state)` — elevar a INV_ALPHA (¡enorme!)
//! 5. `state = MDS * state`
//! 6. `state = state + ARK2[r]`
//!
//! Expresar el paso 4 directamente como restricción daría un polinomio de
//! grado ~10^19, absurdo. La técnica estándar es verificar la ronda desde
//! los DOS extremos y comprobar que coinciden en el medio:
//!
//! - Hacia delante: `A = MDS * sbox(current) + ARK1[r]`
//! - Hacia atrás:   `B = INV_MDS * (next - ARK2[r])`
//! - Como `B` debe ser `inv_sbox(A)`, se cumple `sbox(B) = A`.
//!
//! Restricción final: `sbox(B) - A = 0`, de grado **7** en vez de 10^19.
//! La S-box inversa nunca llega a expresarse.
//!
//! ## Diseño de la traza
//!
//! 12 columnas (el estado completo) × 8 filas: fila 0 = estado inicial,
//! filas 1..7 = estado tras cada una de las 7 rondas. Las constantes de
//! ronda (que cambian en cada fila) se aportan como COLUMNAS PERIÓDICAS,
//! el mecanismo de Winterfell para valores que dependen del número de
//! fila.
//!
//! ## Qué demuestra
//!
//! Conocimiento de dos digests `a`, `b` (privados) tales que
//! `Rp64_256::merge([a, b]) = digest_publico`. Es el bloque base del
//! árbol de Merkle: cada nivel es exactamente esta operación.
//!
//! **Nota**: este módulo usa el campo de 64 bits (Goldilocks), no el
//! `f128` de `range_check.rs`, porque `Rp64_256` está definido sobre él.
//! Unificar ambos módulos al mismo campo es trabajo pendiente para el
//! circuito de cumplimiento integrado.

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

/// Ancho del estado de Rescue: 12 elementos (4 de capacidad + 8 de rate).
pub const STATE_WIDTH: usize = Rp64_256::STATE_WIDTH;
/// 7 rondas (nivel de seguridad de 128 bits con 40% de margen).
pub const NUM_ROUNDS: usize = Rp64_256::NUM_ROUNDS;
/// Longitud de la traza: 8 filas (estado inicial + 7 rondas). Debe ser
/// potencia de dos, y 8 encaja exactamente.
pub const TRACE_LENGTH: usize = 8;

type Blake3 = Blake3_256<BaseElement>;

/// S-box de Rescue: elevar a ALPHA = 7.
/// (ALPHA es privado en winter-crypto; verificable en
/// `src/hash/rescue/rp64_256/mod.rs:52`.)
///
/// Pública para que `merkle.rs` la reutilice sin duplicar la definición.
pub fn apply_sbox<E: FieldElement>(x: E) -> E {
    let x2 = x * x;
    let x4 = x2 * x2;
    x4 * x2 * x // x^7
}

/// Construye la traza: 12 columnas × 8 filas, aplicando las rondas de
/// Rescue con la implementación REAL de `winter-crypto` (no una
/// reimplementación nuestra).
pub fn build_trace(input_a: [BaseElement; 4], input_b: [BaseElement; 4]) -> TraceTable<BaseElement> {
    let mut trace = TraceTable::new(STATE_WIDTH, TRACE_LENGTH);

    trace.fill(
        |state| {
            // Fila 0: capacidad a cero, rate con los dos digests de
            // entrada — el mismo layout que usa `Rp64_256::merge`.
            for s in state.iter_mut() {
                *s = BaseElement::ZERO;
            }
            state[4..8].copy_from_slice(&input_a);
            state[8..12].copy_from_slice(&input_b);
        },
        |step, state| {
            // `step` es el índice de la fila anterior; aplicamos la ronda
            // correspondiente usando la implementación de la librería.
            let mut arr: [BaseElement; STATE_WIDTH] = state.try_into().unwrap();
            Rp64_256::apply_round(&mut arr, step);
            state.copy_from_slice(&arr);
        },
    );

    trace
}

/// Calcula el digest esperado de forma nativa, con la librería.
///
/// **Delegado a `merkle::native_merge`** (entrada 59, §125): eran dos
/// copias carácter a carácter del mismo wrapper sobre `Rp64_256`. Una
/// sola definición, por construcción — el miedo de §117 («dos Rescue que
/// deben coincidir») quedó desmentido por lectura y cerrado aquí.
pub fn native_merge(a: [BaseElement; 4], b: [BaseElement; 4]) -> [BaseElement; 4] {
    crate::merkle::native_merge(a, b)
}

/// Inputs públicos: solo el digest resultante. Los dos valores de entrada
/// son PRIVADOS (testigos) — que es justo lo que necesita el árbol de
/// Merkle, donde el camino no debe revelarse.
#[derive(Clone, Debug)]
pub struct RescuePublicInputs {
    pub digest: [BaseElement; 4],
}

impl ToElements<BaseElement> for RescuePublicInputs {
    fn to_elements(&self) -> Vec<BaseElement> {
        self.digest.to_vec()
    }
}

pub struct RescueAir {
    context: AirContext<BaseElement>,
    digest: [BaseElement; 4],
}

impl Air for RescueAir {
    type BaseField = BaseElement;
    type PublicInputs = RescuePublicInputs;

    fn new(trace_info: TraceInfo, pub_inputs: Self::PublicInputs, options: ProofOptions) -> Self {
        assert_eq!(STATE_WIDTH, trace_info.width());
        // 12 restricciones (una por elemento del estado), todas de grado 7
        // gracias a la técnica de "encontrarse en el medio".
        let degrees = vec![TransitionConstraintDegree::new(7); STATE_WIDTH];
        RescueAir {
            // 8 aserciones: 4 de capacidad inicial + 4 del digest final.
            context: AirContext::new(trace_info, degrees, 8, options),
            digest: pub_inputs.digest,
        }
    }

    fn context(&self) -> &AirContext<Self::BaseField> {
        &self.context
    }

    /// Las constantes de ronda cambian en cada fila, así que se aportan
    /// como columnas periódicas: 24 columnas (12 de ARK1 + 12 de ARK2),
    /// cada una de longitud 8 (7 rondas + una fila de relleno).
    fn get_periodic_column_values(&self) -> Vec<Vec<Self::BaseField>> {
        let mut columns = Vec::with_capacity(2 * STATE_WIDTH);

        for i in 0..STATE_WIDTH {
            let mut col = Vec::with_capacity(TRACE_LENGTH);
            for r in 0..NUM_ROUNDS {
                col.push(Rp64_256::ARK1[r][i]);
            }
            col.push(BaseElement::ZERO); // relleno de la ultima fila
            columns.push(col);
        }
        for i in 0..STATE_WIDTH {
            let mut col = Vec::with_capacity(TRACE_LENGTH);
            for r in 0..NUM_ROUNDS {
                col.push(Rp64_256::ARK2[r][i]);
            }
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

        let ark1 = &periodic_values[0..STATE_WIDTH];
        let ark2 = &periodic_values[STATE_WIDTH..2 * STATE_WIDTH];

        // --- Mitad hacia delante: A = MDS * sbox(current) + ARK1 ---
        let mut sboxed = [E::ZERO; STATE_WIDTH];
        for i in 0..STATE_WIDTH {
            sboxed[i] = apply_sbox(current[i]);
        }
        let mut a = [E::ZERO; STATE_WIDTH];
        for i in 0..STATE_WIDTH {
            let mut acc = E::ZERO;
            for j in 0..STATE_WIDTH {
                acc += E::from(Rp64_256::MDS[i][j]) * sboxed[j];
            }
            a[i] = acc + ark1[i];
        }

        // --- Mitad hacia atrás: B = INV_MDS * (next - ARK2) ---
        let mut unark2 = [E::ZERO; STATE_WIDTH];
        for i in 0..STATE_WIDTH {
            unark2[i] = next[i] - ark2[i];
        }
        let mut b = [E::ZERO; STATE_WIDTH];
        for i in 0..STATE_WIDTH {
            let mut acc = E::ZERO;
            for j in 0..STATE_WIDTH {
                acc += E::from(Rp64_256::INV_MDS[i][j]) * unark2[j];
            }
            b[i] = acc;
        }

        // --- Se encuentran en el medio: sbox(B) debe igualar A ---
        for i in 0..STATE_WIDTH {
            result[i] = apply_sbox(b[i]) - a[i];
        }
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        let last_step = self.trace_length() - 1;
        let mut assertions = Vec::with_capacity(8);

        // La capacidad DEBE arrancar a cero. Sin esto, un probador podría
        // elegir una capacidad arbitraria y romper la seguridad de la
        // construcción esponja.
        for i in 0..4 {
            assertions.push(Assertion::single(i, 0, BaseElement::ZERO));
        }
        // El digest final debe coincidir con el público (elementos 4..8).
        for i in 0..4 {
            assertions.push(Assertion::single(4 + i, last_step, self.digest[i]));
        }

        assertions
    }
}

pub struct RescueProver {
    options: ProofOptions,
}

impl RescueProver {
    pub fn new(options: ProofOptions) -> Self {
        Self { options }
    }
}

impl Prover for RescueProver {
    type BaseField = BaseElement;
    type Air = RescueAir;
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

    fn get_pub_inputs(&self, trace: &Self::Trace) -> RescuePublicInputs {
        let last_step = trace.length() - 1;
        RescuePublicInputs {
            digest: [
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

    fn sample_inputs() -> ([BaseElement; 4], [BaseElement; 4]) {
        (
            [
                BaseElement::new(1),
                BaseElement::new(2),
                BaseElement::new(3),
                BaseElement::new(4),
            ],
            [
                BaseElement::new(5),
                BaseElement::new(6),
                BaseElement::new(7),
                BaseElement::new(8),
            ],
        )
    }

    /// Confirma que la traza construida a mano coincide con lo que
    /// calcula la implementación nativa de la librería — si esto falla,
    /// el resto no tiene sentido.
    #[test]
    fn trace_final_state_matches_native_permutation() {
        let (a, b) = sample_inputs();
        let trace = build_trace(a, b);
        let expected = native_merge(a, b);

        let last = TRACE_LENGTH - 1;
        for i in 0..4 {
            assert_eq!(
                trace.get(4 + i, last),
                expected[i],
                "el elemento {i} del digest en la traza no coincide con el calculo nativo"
            );
        }
    }

    /// EL TEST CLAVE: una prueba STARK real de que se conoce una preimagen
    /// cuyo hash Rescue es el digest público.
    #[test]
    fn valid_rescue_hash_produces_verifiable_proof() {
        let (a, b) = sample_inputs();
        let trace = build_trace(a, b);
        let expected = native_merge(a, b);

        let prover = RescueProver::new(default_options());
        let proof: Proof = prover
            .prove(trace)
            .expect("la generacion de la prueba no deberia fallar");

        let min_opts = AcceptableOptions::OptionSet(vec![prover.options().clone()]);
        let verification = verify::<RescueAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof,
            RescuePublicInputs { digest: expected },
            &min_opts,
        );

        assert!(
            verification.is_ok(),
            "una prueba de hash Rescue valida deberia verificar: {verification:?}"
        );
    }

    /// TEST DE SOLIDEZ: declarar un digest público que NO corresponde a la
    /// preimagen real debe hacer fallar la verificación.
    #[test]
    fn wrong_declared_digest_fails_verification() {
        let (a, b) = sample_inputs();
        let trace = build_trace(a, b);

        let prover = RescueProver::new(default_options());
        let proof = prover.prove(trace).expect("la generacion no deberia fallar");

        let wrong_digest = [
            BaseElement::new(111),
            BaseElement::new(222),
            BaseElement::new(333),
            BaseElement::new(444),
        ];
        let min_opts = AcceptableOptions::OptionSet(vec![prover.options().clone()]);
        let verification = verify::<RescueAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof,
            RescuePublicInputs {
                digest: wrong_digest,
            },
            &min_opts,
        );

        assert!(
            verification.is_err(),
            "CRITICO: un digest declarado incorrecto no deberia verificar"
        );
    }

    /// Confirma que entradas distintas producen digests distintos.
    #[test]
    fn different_inputs_produce_different_digests() {
        let (a, b) = sample_inputs();
        let d1 = native_merge(a, b);
        let d2 = native_merge(b, a); // orden invertido
        assert_ne!(d1, d2, "invertir el orden deberia cambiar el digest");
    }

    /// EL TEST QUE DE VERDAD VALIDA LAS RESTRICCIONES.
    ///
    /// Los otros tests tienen una laguna: si las restricciones de
    /// transición fueran VACUAS (siempre cero por un error), seguirían
    /// pasando igual — el caso válido pasaría trivialmente, y el del
    /// digest incorrecto fallaría por la ASERCIÓN, no por las
    /// restricciones. No distinguirían "restricciones correctas" de
    /// "restricciones que no comprueban nada".
    ///
    /// Este sí: corrompe una fila INTERMEDIA de la traza (dejando intactas
    /// la primera y la última, que son las que cubren las aserciones). Si
    /// las restricciones de transición son reales, esto debe detectarse.
    /// Si pasara, significaría que las restricciones son decorativas.
    #[test]
    fn corrupted_intermediate_row_is_detected() {
        let (a, b) = sample_inputs();
        let mut trace = build_trace(a, b);

        // Corromper un elemento de una fila intermedia (fila 3, columna 5).
        let original = trace.get(5, 3);
        trace.set(5, 3, original + BaseElement::new(1));

        let prover = RescueProver::new(default_options());

        // La deteccion puede manifestarse de tres formas legitimas: panic
        // (modo debug), Err, o prueba que no verifica. Ver la nota
        // completa en `merkle.rs`.
        let prove_result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| prover.prove(trace)));

        match prove_result {
            Err(_) => { /* panic: detectado en modo debug */ }
            Ok(Err(_)) => { /* Err: detectado */ }
            Ok(Ok(proof)) => {
                let expected = native_merge(a, b);
                let min_opts = AcceptableOptions::OptionSet(vec![prover.options().clone()]);
                let verification =
                    verify::<RescueAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
                        proof,
                        RescuePublicInputs { digest: expected },
                        &min_opts,
                    );
                assert!(
                    verification.is_err(),
                    "CRITICO: una traza con una fila intermedia corrompida NO deberia \
                     producir una prueba verificable. Si esto pasa, las restricciones \
                     de transicion no estan comprobando nada."
                );
            }
        }
    }

    /// Ninguna restriccion es vacua (entrada 38, §62).
    ///
    /// La prueba por mutacion perturba la traza celda a celda y comprueba que
    /// **cada** restriccion reacciona a algo. Una que no reaccione nunca esta
    /// declarada, tiene grado asignado y no impone nada.
    ///
    /// AVISO: no detecta el defecto de §38 -una ranura sobrescrita sigue
    /// reaccionando, solo que a la restriccion equivocada-. Para eso esta
    /// `tools/check_constraint_layout.py`. Dos herramientas, dos defectos.
    #[test]
    fn no_constraint_is_vacuous() {
        use crate::mutation::{buscar_vacias, rows_of};
        use winterfell::Prover;

        let (a, b) = sample_inputs();
        let trace = build_trace(a, b);
        let rows = rows_of(&trace, STATE_WIDTH, TRACE_LENGTH);
        let pub_inputs = RescueProver::new(default_options()).get_pub_inputs(&trace);
        let air = RescueAir::new(
            TraceInfo::new(STATE_WIDTH, TRACE_LENGTH),
            pub_inputs,
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
}
