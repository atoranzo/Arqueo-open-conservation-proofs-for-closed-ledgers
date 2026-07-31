//! Nullifier verificado en AIR: `Rescue(Rescue(DOMAIN, account_id), nonce)`,
//! con separación de dominio — equivalente STARK de
//! `zk-core::nullifier` y `halo2-experiment::nullifier`.
//!
//! ## Más simple que el árbol de Merkle
//!
//! Reutiliza la misma maquinaria de ciclos y columnas periódicas que
//! `merkle.rs`, pero sin bit de dirección: el digest intermedio siempre
//! va a la izquierda. Solo 2 ciclos (16 filas) en vez de 32 (256).
//!
//! ## Por qué la separación de dominio
//!
//! Si el nullifier se calculara igual que una hoja del árbol, un mismo
//! valor podría interpretarse como ambas cosas. Anteponer una constante
//! de dominio fija (`NULLIFIER_DOMAIN`) ata el hash a un propósito único.
//! La constante se ANCLA con una aserción de frontera, así que el
//! verificador comprueba que se usó realmente — no es un adorno.
//!
//! ## Diseño de la traza (12 columnas × 16 filas)
//!
//! - Ciclo 0 (filas 0..7): `inner = Rescue(DOMAIN_digest, account_id_digest)`
//! - Fila de enlace (transición 7→8): coloca `inner` a la izquierda y el
//!   nonce a la derecha.
//! - Ciclo 1 (filas 8..15): `nullifier = Rescue(inner, nonce_digest)`

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

use crate::merkle::{native_merge, Digest};
use crate::rescue_hash::{apply_sbox, NUM_ROUNDS, STATE_WIDTH};

/// Misma constante de dominio que en los otros dos backends.
pub const NULLIFIER_DOMAIN: u64 = 0x4E554C4C; // "NULL", mnemónico
/// Filas por ciclo de hash: 7 rondas + 1 de enlace.
pub const CYCLE_LENGTH: usize = 8;
/// Dos hashes encadenados → 16 filas (potencia de dos ✓).
pub const TRACE_LENGTH: usize = 2 * CYCLE_LENGTH;

type Blake3 = Blake3_256<BaseElement>;

/// Empaqueta un escalar como digest de 4 elementos (relleno con ceros).
fn as_digest(x: BaseElement) -> Digest {
    [x, BaseElement::ZERO, BaseElement::ZERO, BaseElement::ZERO]
}

/// Calcula el nullifier de forma nativa, con la librería real.
pub fn native_nullifier(account_id: BaseElement, nonce: BaseElement) -> Digest {
    let domain = as_digest(BaseElement::new(NULLIFIER_DOMAIN));
    let inner = native_merge(domain, as_digest(account_id));
    native_merge(inner, as_digest(nonce))
}

/// Construye la traza: dos hashes de Rescue encadenados.
pub fn build_trace(account_id: BaseElement, nonce: BaseElement) -> TraceTable<BaseElement> {
    let mut trace = TraceTable::new(STATE_WIDTH, TRACE_LENGTH);

    trace.fill(
        |state| {
            // Fila 0: capacidad a cero, DOMAIN a la izquierda,
            // account_id a la derecha.
            for s in state.iter_mut() {
                *s = BaseElement::ZERO;
            }
            state[4] = BaseElement::new(NULLIFIER_DOMAIN);
            state[8] = account_id;
        },
        |step, state| {
            let position_in_cycle = step % CYCLE_LENGTH;

            if position_in_cycle < NUM_ROUNDS {
                let mut arr: [BaseElement; STATE_WIDTH] = state[..].try_into().unwrap();
                Rp64_256::apply_round(&mut arr, position_in_cycle);
                state.copy_from_slice(&arr);
            } else {
                // Fila de enlace: el digest intermedio pasa a la
                // izquierda, el nonce entra por la derecha.
                let inner: Digest = [state[4], state[5], state[6], state[7]];
                for s in state.iter_mut() {
                    *s = BaseElement::ZERO;
                }
                state[4..8].copy_from_slice(&inner);
                state[8] = nonce;
            }
        },
    );

    trace
}

/// Inputs públicos: el nullifier. `account_id` y `nonce` son PRIVADOS.
#[derive(Clone, Debug)]
pub struct NullifierPublicInputs {
    pub nullifier: Digest,
}

impl ToElements<BaseElement> for NullifierPublicInputs {
    fn to_elements(&self) -> Vec<BaseElement> {
        self.nullifier.to_vec()
    }
}

pub struct NullifierAir {
    context: AirContext<BaseElement>,
    nullifier: Digest,
}

impl Air for NullifierAir {
    type BaseField = BaseElement;
    type PublicInputs = NullifierPublicInputs;

    fn new(trace_info: TraceInfo, pub_inputs: Self::PublicInputs, options: ProofOptions) -> Self {
        assert_eq!(STATE_WIDTH, trace_info.width());

        let mut degrees = Vec::new();
        // 12 restricciones de hash (grado 7, moduladas por el selector).
        for _ in 0..STATE_WIDTH {
            degrees.push(TransitionConstraintDegree::with_cycles(7, vec![CYCLE_LENGTH]));
        }
        // 4 de reinicio de capacidad + 4 de traspaso del digest interno.
        for _ in 0..8 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, vec![CYCLE_LENGTH]));
        }

        NullifierAir {
            // 12 aserciones: ver get_assertions.
            context: AirContext::new(trace_info, degrees, 12, options),
            nullifier: pub_inputs.nullifier,
        }
    }

    fn context(&self) -> &AirContext<Self::BaseField> {
        &self.context
    }

    fn get_periodic_column_values(&self) -> Vec<Vec<Self::BaseField>> {
        let mut columns = Vec::with_capacity(1 + 2 * STATE_WIDTH);

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

        // --- Restricciones de hash (misma tecnica que rescue_hash.rs) ---
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

        // --- Restricciones de enlace ---
        let link_flag = E::ONE - hash_flag;

        // La capacidad se reinicia a cero.
        for i in 0..4 {
            result[STATE_WIDTH + i] = link_flag * next[i];
        }
        // El digest interno pasa intacto a la mitad izquierda del rate.
        // (Sin bit de direccion: aqui siempre va a la izquierda.)
        for i in 0..4 {
            result[STATE_WIDTH + 4 + i] = link_flag * (next[4 + i] - current[4 + i]);
        }
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        let last_step = TRACE_LENGTH - 1;
        let mut assertions = Vec::with_capacity(12);

        // Capacidad inicial a cero (seguridad de la esponja).
        for i in 0..4 {
            assertions.push(Assertion::single(i, 0, BaseElement::ZERO));
        }
        // La constante de dominio DEBE estar en su sitio: esto es lo que
        // hace que la separacion de dominio sea real y comprobable, no un
        // adorno que el probador pueda ignorar.
        assertions.push(Assertion::single(
            4,
            0,
            BaseElement::new(NULLIFIER_DOMAIN),
        ));
        // Y el resto del digest de dominio es relleno de ceros.
        for i in 5..8 {
            assertions.push(Assertion::single(i, 0, BaseElement::ZERO));
        }
        // El nullifier final debe coincidir con el publico.
        for i in 0..4 {
            assertions.push(Assertion::single(4 + i, last_step, self.nullifier[i]));
        }

        assertions
    }
}

pub struct NullifierProver {
    options: ProofOptions,
}

impl NullifierProver {
    pub fn new(options: ProofOptions) -> Self {
        Self { options }
    }
}

impl Prover for NullifierProver {
    type BaseField = BaseElement;
    type Air = NullifierAir;
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
        let last_step = trace.length() - 1;
        NullifierPublicInputs {
            nullifier: [
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

    /// Primer test, el más informativo si falla: la traza debe terminar
    /// con el mismo nullifier que calcula la versión nativa.
    #[test]
    fn trace_final_nullifier_matches_native_computation() {
        let account_id = BaseElement::new(12345);
        let nonce = BaseElement::new(1);

        let trace = build_trace(account_id, nonce);
        let expected = native_nullifier(account_id, nonce);

        let last = TRACE_LENGTH - 1;
        for i in 0..4 {
            assert_eq!(
                trace.get(4 + i, last),
                expected[i],
                "el elemento {i} del nullifier no coincide con el calculo nativo"
            );
        }
    }

    /// EL TEST CLAVE: prueba STARK real de conocimiento de la cuenta y el
    /// nonce que generan el nullifier público.
    #[test]
    fn valid_nullifier_produces_verifiable_proof() {
        let account_id = BaseElement::new(12345);
        let nonce = BaseElement::new(1);

        let trace = build_trace(account_id, nonce);
        let expected = native_nullifier(account_id, nonce);

        let prover = NullifierProver::new(default_options());
        let proof: Proof = prover
            .prove(trace)
            .expect("la generacion de la prueba no deberia fallar");

        let min_opts = AcceptableOptions::OptionSet(vec![prover.options().clone()]);
        let verification =
            verify::<NullifierAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
                proof,
                NullifierPublicInputs {
                    nullifier: expected,
                },
                &min_opts,
            );

        assert!(
            verification.is_ok(),
            "un nullifier valido deberia verificar: {verification:?}"
        );
    }

    /// TEST DE SEGURIDAD: declarar un nullifier FALSIFICADO (que no
    /// corresponde a la cuenta y nonce reales) debe fallar. Sin esto, el
    /// nullifier sería decorativo: cualquiera podría inventarse uno para
    /// esquivar el registro de gastados.
    #[test]
    fn forged_nullifier_fails_verification() {
        let account_id = BaseElement::new(12345);
        let nonce = BaseElement::new(1);
        let trace = build_trace(account_id, nonce);

        let prover = NullifierProver::new(default_options());
        let proof = prover.prove(trace).expect("la generacion no deberia fallar");

        let forged = [
            BaseElement::new(999),
            BaseElement::new(888),
            BaseElement::new(777),
            BaseElement::new(666),
        ];
        let min_opts = AcceptableOptions::OptionSet(vec![prover.options().clone()]);
        let verification =
            verify::<NullifierAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
                proof,
                NullifierPublicInputs { nullifier: forged },
                &min_opts,
            );

        assert!(
            verification.is_err(),
            "CRITICO: un nullifier falsificado no deberia verificar"
        );
    }

    /// Avanzar el nonce debe cambiar el nullifier: es lo que permite que
    /// la misma cuenta gaste más de una vez sin reutilizar el nullifier.
    #[test]
    fn different_nonce_produces_different_nullifier() {
        let account_id = BaseElement::new(12345);
        let n1 = native_nullifier(account_id, BaseElement::new(1));
        let n2 = native_nullifier(account_id, BaseElement::new(2));
        assert_ne!(n1, n2);
    }

    /// Cuentas distintas deben producir nullifiers distintos.
    #[test]
    fn different_account_produces_different_nullifier() {
        let nonce = BaseElement::new(1);
        let n1 = native_nullifier(BaseElement::new(111), nonce);
        let n2 = native_nullifier(BaseElement::new(222), nonce);
        assert_ne!(n1, n2);
    }

    /// TEST DISCRIMINANTE: corromper una fila intermedia debe detectarse.
    /// Ver la nota sobre las tres formas de detección en `merkle.rs`.
    #[test]
    fn corrupted_intermediate_row_is_detected() {
        let account_id = BaseElement::new(12345);
        let nonce = BaseElement::new(1);
        let mut trace = build_trace(account_id, nonce);

        let original = trace.get(6, 3);
        trace.set(6, 3, original + BaseElement::ONE);

        let expected = native_nullifier(account_id, nonce);
        let prover = NullifierProver::new(default_options());

        let prove_result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| prover.prove(trace)));

        match prove_result {
            Err(_) => { /* panic: detectado en modo debug */ }
            Ok(Err(_)) => { /* Err: detectado */ }
            Ok(Ok(proof)) => {
                let min_opts = AcceptableOptions::OptionSet(vec![prover.options().clone()]);
                let verification = verify::<
                    NullifierAir,
                    Blake3,
                    DefaultRandomCoin<Blake3>,
                    MerkleTree<Blake3>,
                >(
                    proof,
                    NullifierPublicInputs {
                        nullifier: expected,
                    },
                    &min_opts,
                );
                assert!(
                    verification.is_err(),
                    "CRITICO: una traza con una fila intermedia corrompida no deberia verificar"
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

        let trace = build_trace(BaseElement::new(12345), BaseElement::new(1));
        let rows = rows_of(&trace, STATE_WIDTH, TRACE_LENGTH);
        let pub_inputs = NullifierProver::new(default_options()).get_pub_inputs(&trace);
        let air = NullifierAir::new(
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
