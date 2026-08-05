//! **Apertura del compromiso pendiente** — el circuito de la caducidad
//! (`doc/CADUCIDAD_PENDIENTE.md`, AUDITORIA §178).
//!
//! Demuestra conocimiento de `(identidad_receptor, aleatorio)` tales que
//!
//! ```text
//! P = H( H(identidad_receptor, aleatorio), [importe, 0, 0, 0] )
//! ```
//!
//! con `P` y el `importe` como **entradas públicas** y el receptor y el
//! aleatorio como **testigo privado**. Es la reconstrucción `C_PEND_IN`
//! de `circuit_claim` (§39.1), aislada: dos merges de Rescue y nada más.
//!
//! ## Lo que este circuito NO lleva, a propósito (§178)
//!
//! - **Sin PK-check**: el destino del reembolso lo fija la capa desde sus
//!   registros (`pending_meta[pos].sender_index`) — probar la apertura no
//!   elige a quién se acredita.
//! - **Sin nullifier**: la hoja vacía es su propio anti-replay; el segundo
//!   intento no encuentra `P` en el árbol.
//! - **Sin frozen-check**: devolver no es cobrar.
//! - **Sin camino de Merkle**: `P` ya es público (vive en la hoja); la capa
//!   comprueba `hoja[pos] == P` y vacía nativamente, como todo `apply`.
//!
//! El crédito al emisor viaja aparte, en un `mint_climb` sin tocar.
//!
//! ## Grado en depuración (familia de la entrada 24, §46/§34)
//!
//! El bloque `[importe, 0, 0, 0]` lleva tres ceros por construcción: en
//! modo depuración la comprobación de grados de winterfell puede degenerar
//! sobre esos valores del dominio, como en los casos ya declarados. En
//! release —donde la suite corre— genera y verifica correctamente.

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

use crate::merkle::{native_merge, Digest};
use crate::rescue_hash::{apply_sbox, NUM_ROUNDS, STATE_WIDTH};

type Blake3 = Blake3_256<BaseElement>;

/// Rondas de una permutación por ciclo (7 rondas + 1 fila de enlace).
pub const CYCLE_LENGTH: usize = 8;
/// Dos merges: `H(id_r, salt)` y `H(interior, importe)`.
pub const NUM_MERGES: usize = 2;
/// 16 filas: la traza más corta de la casa.
pub const TRACE_LENGTH: usize = NUM_MERGES * CYCLE_LENGTH;
/// Una sola vía de estado Rescue; sin columnas extra.
pub const TRACE_WIDTH: usize = STATE_WIDTH;
/// Fila donde el segundo merge absorbe: el importe queda aquí, público.
pub const ROW_AMOUNT: usize = CYCLE_LENGTH;
/// Fila final: el compromiso `P` queda en `estado[4..8]`.
pub const ROW_P: usize = TRACE_LENGTH - 1;
const _: () = assert!(ROW_P == NUM_MERGES * CYCLE_LENGTH - 1);

// ── Ranuras de restricciones (disposición para el guardián) ──
/// 12 rondas de Rescue sobre la vía.
const C_HASH: usize = 0;
/// 4: la capacidad renace a cero tras el enlace.
const C_CAP: usize = STATE_WIDTH;
/// 4: el digest del primer merge se porta al segundo.
const C_CARRY: usize = STATE_WIDTH + 4;
const TRANSITION_WIDTH: usize = STATE_WIDTH + 8;

/// Gemelo nativo del compromiso, para paridad en tests y en la capa.
/// Idéntico a `pending_commitment` de `zk-ssl` (pending.rs), que sigue
/// siendo el canónico del lado de la capa.
pub fn native_refund_commitment(
    receiver_id: Digest,
    salt: Digest,
    amount: u64,
) -> Digest {
    let inner = native_merge(receiver_id, salt);
    native_merge(
        inner,
        [
            BaseElement::new(amount),
            BaseElement::ZERO,
            BaseElement::ZERO,
            BaseElement::ZERO,
        ],
    )
}

/// Construye la traza de la apertura: dos permutaciones encadenadas.
///
/// El receptor y el aleatorio entran SOLO aquí (testigo); el importe y el
/// compromiso resultante quedan atados por aserciones públicas.
pub fn build_trace(
    receiver_id: Digest,
    salt: Digest,
    amount: u64,
) -> TraceTable<BaseElement> {
    let zero = BaseElement::ZERO;
    let mut rows: Vec<Vec<BaseElement>> = vec![vec![zero; TRACE_WIDTH]; TRACE_LENGTH];

    let mut state = [zero; STATE_WIDTH];
    state[4..8].copy_from_slice(&receiver_id);
    state[8..12].copy_from_slice(&salt);
    rows[0][..STATE_WIDTH].copy_from_slice(&state);

    for r in 0..TRACE_LENGTH - 1 {
        let pos = r % CYCLE_LENGTH;
        if pos < NUM_ROUNDS {
            Rp64_256::apply_round(&mut state, pos);
        } else {
            // Fila de enlace: el digest sale de [4..8], la capacidad
            // renace, y el segundo merge absorbe (digest, importe).
            let digest: Digest = [state[4], state[5], state[6], state[7]];
            state = [zero; STATE_WIDTH];
            state[4..8].copy_from_slice(&digest);
            state[8] = BaseElement::new(amount);
            // state[9..12] quedan a cero: [importe, 0, 0, 0].
        }
        rows[r + 1][..STATE_WIDTH].copy_from_slice(&state);
    }

    let columns: Vec<Vec<BaseElement>> = (0..TRACE_WIDTH)
        .map(|c| rows.iter().map(|fila| fila[c]).collect())
        .collect();
    TraceTable::init(columns)
}

/// Entradas públicas: el compromiso que se abre y el importe que ata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefundPublicInputs {
    pub commitment: Digest,
    pub amount: BaseElement,
}

impl ToElements<BaseElement> for RefundPublicInputs {
    fn to_elements(&self) -> Vec<BaseElement> {
        let mut e = self.commitment.to_vec();
        e.push(self.amount);
        e
    }
}

pub struct RefundAir {
    context: AirContext<BaseElement>,
    commitment: Digest,
    amount: BaseElement,
}

impl Air for RefundAir {
    type BaseField = BaseElement;
    type PublicInputs = RefundPublicInputs;

    fn new(trace_info: TraceInfo, pub_inputs: Self::PublicInputs, options: ProofOptions) -> Self {
        assert_eq!(TRACE_WIDTH, trace_info.width());

        let mut degrees = Vec::new();
        for _ in 0..STATE_WIDTH {
            degrees.push(TransitionConstraintDegree::with_cycles(7, vec![CYCLE_LENGTH]));
        }
        for _ in 0..8 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, vec![CYCLE_LENGTH]));
        }
        debug_assert_eq!(degrees.len(), TRANSITION_WIDTH);

        RefundAir {
            context: AirContext::new(trace_info, degrees, 12, options),
            commitment: pub_inputs.commitment,
            amount: pub_inputs.amount,
        }
    }

    fn context(&self) -> &AirContext<Self::BaseField> {
        &self.context
    }

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

        // Ronda de Rescue sobre la única vía (forma de frozen_climb).
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

        // Enlace: capacidad a cero y porte del digest interior.
        for i in 0..4 {
            result[C_CAP + i] = link_flag * next[i];
        }
        for i in 0..4 {
            result[C_CARRY + i] = link_flag * (next[4 + i] - current[4 + i]);
        }
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        let zero = BaseElement::ZERO;
        let mut assertions = Vec::with_capacity(12);

        // Fila 0: capacidad limpia. El receptor y el aleatorio (4..12)
        // quedan LIBRES: son el testigo.
        for i in 0..4 {
            assertions.push(Assertion::single(i, 0, zero));
        }
        // Fila del segundo merge: el importe, público, y sus tres ceros.
        assertions.push(Assertion::single(8, ROW_AMOUNT, self.amount));
        for i in 1..4 {
            assertions.push(Assertion::single(8 + i, ROW_AMOUNT, zero));
        }
        // Fila final: el compromiso.
        for i in 0..4 {
            assertions.push(Assertion::single(4 + i, ROW_P, self.commitment[i]));
        }
        assertions
    }
}

pub struct RefundProver {
    options: ProofOptions,
}

impl RefundProver {
    pub fn new(options: ProofOptions) -> Self {
        RefundProver { options }
    }
}

impl Prover for RefundProver {
    type BaseField = BaseElement;
    type Air = RefundAir;
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

    fn get_pub_inputs(&self, trace: &Self::Trace) -> RefundPublicInputs {
        use winterfell::Trace;
        debug_assert!(trace.length() > ROW_P);
        RefundPublicInputs {
            commitment: [
                trace.get(4, ROW_P),
                trace.get(5, ROW_P),
                trace.get(6, ROW_P),
                trace.get(7, ROW_P),
            ],
            amount: trace.get(8, ROW_AMOUNT),
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

    fn digest_from(n: u64) -> Digest {
        [
            BaseElement::new(n),
            BaseElement::new(n + 1),
            BaseElement::new(n + 2),
            BaseElement::new(n + 3),
        ]
    }

    /// Paridad traza↔nativo: la última fila reproduce el compromiso y la
    /// fila del enlace lleva el importe donde la aserción lo mira.
    #[test]
    fn trace_commitment_matches_native_computation() {
        let receptor = digest_from(1000);
        let salt = digest_from(2000);
        let importe = 250_000u64;

        let trace = build_trace(receptor, salt, importe);
        let esperado = native_refund_commitment(receptor, salt, importe);

        for i in 0..4 {
            assert_eq!(trace.get(4 + i, ROW_P), esperado[i], "elem {i}");
        }
        assert_eq!(trace.get(8, ROW_AMOUNT), BaseElement::new(importe));
        for i in 1..4 {
            assert_eq!(trace.get(8 + i, ROW_AMOUNT), BaseElement::ZERO);
        }
    }

    /// La apertura legítima produce una prueba que verifica.
    #[test]
    fn valid_refund_opening_produces_verifiable_proof() {
        let receptor = digest_from(1000);
        let salt = digest_from(2000);
        let importe = 250_000u64;

        let trace = build_trace(receptor, salt, importe);
        let prover = RefundProver::new(default_options());
        let proof = prover.prove(trace).expect("la generacion no deberia fallar");

        let min_opts = AcceptableOptions::OptionSet(vec![prover.options().clone()]);
        let ok = verify::<RefundAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof,
            RefundPublicInputs {
                commitment: native_refund_commitment(receptor, salt, importe),
                amount: BaseElement::new(importe),
            },
            &min_opts,
        );
        assert!(ok.is_ok(), "una apertura correcta debe verificar");
    }

    /// **Discriminante**: la prueba ata el IMPORTE. Verificarla contra otro
    /// importe debe fallar — sin esto, el ladrón cobraría lo que quisiera.
    #[test]
    fn proof_does_not_verify_for_another_amount() {
        let receptor = digest_from(1000);
        let salt = digest_from(2000);
        let importe = 250_000u64;

        let trace = build_trace(receptor, salt, importe);
        let prover = RefundProver::new(default_options());
        let proof = prover.prove(trace).expect("prueba");

        let min_opts = AcceptableOptions::OptionSet(vec![prover.options().clone()]);
        let mal = verify::<RefundAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof,
            RefundPublicInputs {
                commitment: native_refund_commitment(receptor, salt, importe),
                amount: BaseElement::new(importe + 1),
            },
            &min_opts,
        );
        assert!(mal.is_err(), "otro importe NO debe verificar");
    }

    /// **Discriminante**: la prueba ata el COMPROMISO. Un `P` ajeno debe
    /// fallar — la apertura no es transferible a otra hoja.
    #[test]
    fn proof_does_not_verify_for_another_commitment() {
        let receptor = digest_from(1000);
        let salt = digest_from(2000);
        let importe = 250_000u64;

        let trace = build_trace(receptor, salt, importe);
        let prover = RefundProver::new(default_options());
        let proof = prover.prove(trace).expect("prueba");

        let mut ajeno = native_refund_commitment(receptor, salt, importe);
        ajeno[0] += BaseElement::ONE;

        let min_opts = AcceptableOptions::OptionSet(vec![prover.options().clone()]);
        let mal = verify::<RefundAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof,
            RefundPublicInputs {
                commitment: ajeno,
                amount: BaseElement::new(importe),
            },
            &min_opts,
        );
        assert!(mal.is_err(), "otro compromiso NO debe verificar");
    }
}
