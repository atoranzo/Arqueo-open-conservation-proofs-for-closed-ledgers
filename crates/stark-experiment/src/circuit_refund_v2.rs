//! **Apertura del compromiso pendiente v2** (RFC-0003, S345/S346) - EN
//! PARALELO a `circuit_refund`, que no se toca: la convivencia manda que
//! el 0.2 siga IDENTICO byte a byte.
//!
//! Demuestra conocimiento de `(receptor, aleatorio, f, delta)` tales que
//!
//! ```text
//! C2 = M( C1(receptor, aleatorio, importe), M(f, [delta,0,0,0]) )
//! ```
//!
//! con `C2` y el `importe` como entradas publicas y TODO lo demas como
//! testigo privado: `f` y `delta` jamas salen del probador. Geometria:
//! CUATRO merges encadenados en una via (32 filas, potencia de 2):
//!
//!   merge 1: (f, d(delta)) -> X       [el sobre, primero]
//!   enlace de RESIEMBRA: capacidad a cero, digest NO arrastrado,
//!       X capturado a las columnas de transporte (COL_X, patron del
//!       salt de hoja del claim, S117)
//!   merge 2: (receptor, aleatorio) -> d1
//!   enlace clasico: digest arrastrado, absorbe el importe (publico)
//!   merge 3: (d1, importe) -> C1
//!   enlace clasico: digest arrastrado, absorbe X desde COL_X
//!   merge 4: (C1, X) -> C2
//!
//! El juez es `native_refund_commitment_v2` (E1a, S345). Fichero PROPIO
//! porque el guardian de layout barre por fichero: dos Air en uno
//! mezclarian sus ranuras.

use winterfell::crypto::hashers::{Blake3_256, Rp64_256};
use winterfell::crypto::{DefaultRandomCoin, MerkleTree};
use winterfell::math::{fields::f64::BaseElement, FieldElement};
use winterfell::matrix::ColMatrix;
use winterfell::{
    Air, AirContext, Assertion, AuxRandElements, CompositionPoly, CompositionPolyTrace,
    ConstraintCompositionCoefficients, DefaultConstraintCommitment, DefaultConstraintEvaluator,
    DefaultTraceLde, EvaluationFrame, PartitionOptions, ProofOptions, Prover, StarkDomain,
    TraceInfo, TracePolyTable, TraceTable, TransitionConstraintDegree,
};

use crate::circuit_refund::RefundPublicInputs;
use crate::merkle::{native_merge, Digest};
use crate::rescue_hash::{apply_sbox, NUM_ROUNDS, STATE_WIDTH};

type Blake3 = Blake3_256<BaseElement>;

pub const CYCLE_LENGTH: usize = 8;
/// CUATRO merges: X, el interior, C1 y C2.
pub const NUM_MERGES: usize = 4;
/// 32 filas: potencia de 2 (24 no lo es; el cuarto merge no es adorno).
pub const TRACE_LENGTH: usize = NUM_MERGES * CYCLE_LENGTH;
/// La via de estado mas las 4 columnas de transporte de X.
pub const TRACE_WIDTH: usize = STATE_WIDTH + 4;
/// Columnas donde X viaja constante tras su captura (12..16).
pub const COL_X: usize = STATE_WIDTH;
/// Fila donde el tercer merge absorbe: el importe queda aqui, publico.
pub const ROW_AMOUNT: usize = 2 * CYCLE_LENGTH;
/// Fila final: el compromiso `C2` queda en `estado[4..8]`.
pub const ROW_P: usize = TRACE_LENGTH - 1;
const _: () = assert!(ROW_P == NUM_MERGES * CYCLE_LENGTH - 1);

// -- Ranuras de restricciones (disposicion para el guardian) --
/// 12 rondas de Rescue sobre la via.
const C_HASH: usize = 0;
/// 4: la capacidad renace a cero tras TODO enlace.
const C_CAP: usize = STATE_WIDTH;
/// 4: el digest se porta en los enlaces del importe y de X (no en la
/// resiembra, donde el digest se sustituye por el testigo).
const C_CARRY: usize = C_CAP + 4;
/// 4: la CAPTURA de X - en la resiembra, COL_X := el digest del merge 1.
const C_XSET: usize = C_CARRY + 4;
/// 4: el TRANSPORTE de X - constante en toda fila que no sea la resiembra.
const C_XKEEP: usize = C_XSET + 4;
/// 4: la ABSORCION de X - el rate del merge 4 := COL_X.
const C_XIN: usize = C_XKEEP + 4;
const NUM_CONSTRAINTS: usize = C_XIN + 4;

// -- Columnas periodicas --
const P_HASH_FLAG: usize = 0;
const P_ARK1: usize = 1;
const P_ARK2: usize = P_ARK1 + STATE_WIDTH;
/// Enlace de RESIEMBRA (fila 7): arranca la cadena de C1.
const P_RESEED: usize = P_ARK2 + STATE_WIDTH;
/// Enlace del IMPORTE (fila 15).
const P_AMT: usize = P_RESEED + 1;
/// Enlace de X (fila 23).
const P_X: usize = P_AMT + 1;

/// Construye la traza v2: el sobre primero, la apertura despues.
pub fn build_trace(
    receiver_id: Digest,
    salt: Digest,
    amount: u64,
    refund_id: Digest,
    delta: u64,
) -> TraceTable<BaseElement> {
    let zero = BaseElement::ZERO;
    let mut rows: Vec<Vec<BaseElement>> = vec![vec![zero; TRACE_WIDTH]; TRACE_LENGTH];

    // El X nativo, para sembrar las columnas de transporte: la captura
    // del circuito lo ata al digest real del merge 1.
    let x_nativo = native_merge(
        refund_id,
        [
            BaseElement::new(delta),
            BaseElement::ZERO,
            BaseElement::ZERO,
            BaseElement::ZERO,
        ],
    );

    let mut state = [zero; STATE_WIDTH];
    state[4..8].copy_from_slice(&refund_id);
    state[8] = BaseElement::new(delta);
    rows[0][..STATE_WIDTH].copy_from_slice(&state);

    for r in 0..TRACE_LENGTH - 1 {
        let pos = r % CYCLE_LENGTH;
        if pos < NUM_ROUNDS {
            Rp64_256::apply_round(&mut state, pos);
        } else if r == CYCLE_LENGTH - 1 {
            // RESIEMBRA: X queda capturado en COL_X; la via arranca la
            // cadena de C1 con el receptor y el aleatorio (testigo).
            state = [zero; STATE_WIDTH];
            state[4..8].copy_from_slice(&receiver_id);
            state[8..12].copy_from_slice(&salt);
        } else if r == 2 * CYCLE_LENGTH - 1 {
            // Enlace clasico: digest arrastrado, absorbe el importe.
            let digest: Digest = [state[4], state[5], state[6], state[7]];
            state = [zero; STATE_WIDTH];
            state[4..8].copy_from_slice(&digest);
            state[8] = BaseElement::new(amount);
        } else {
            // Enlace de X: digest (C1) arrastrado, absorbe el sobre.
            let digest: Digest = [state[4], state[5], state[6], state[7]];
            state = [zero; STATE_WIDTH];
            state[4..8].copy_from_slice(&digest);
            state[8..12].copy_from_slice(&x_nativo);
        }
        rows[r + 1][..STATE_WIDTH].copy_from_slice(&state);
    }

    // COL_X: cero hasta la resiembra, X constante despues.
    for (r, row) in rows.iter_mut().enumerate() {
        if r >= CYCLE_LENGTH {
            row[COL_X..COL_X + 4].copy_from_slice(&x_nativo);
        }
    }

    let columns: Vec<Vec<BaseElement>> = (0..TRACE_WIDTH)
        .map(|c| rows.iter().map(|fila| fila[c]).collect())
        .collect();
    TraceTable::init(columns)
}

pub struct RefundAirV2 {
    context: AirContext<BaseElement>,
    commitment: Digest,
    amount: BaseElement,
}

impl Air for RefundAirV2 {
    type BaseField = BaseElement;
    type PublicInputs = RefundPublicInputs;

    fn new(trace_info: TraceInfo, pub_inputs: Self::PublicInputs, options: ProofOptions) -> Self {
        assert_eq!(TRACE_WIDTH, trace_info.width());

        let mut degrees = Vec::new();
        for _ in 0..STATE_WIDTH {
            degrees.push(TransitionConstraintDegree::with_cycles(7, vec![CYCLE_LENGTH]));
        }
        for _ in STATE_WIDTH..NUM_CONSTRAINTS {
            degrees.push(TransitionConstraintDegree::with_cycles(1, vec![TRACE_LENGTH]));
        }
        debug_assert_eq!(degrees.len(), NUM_CONSTRAINTS);

        RefundAirV2 {
            context: AirContext::new(trace_info, degrees, 16, options),
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
        let mut columns = Vec::with_capacity(1 + 2 * STATE_WIDTH + 3);

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

        // Los tres enlaces, cada uno con su bandera de longitud completa.
        let mut reseed = vec![zero; TRACE_LENGTH];
        reseed[CYCLE_LENGTH - 1] = one;
        columns.push(reseed);
        let mut amt = vec![zero; TRACE_LENGTH];
        amt[2 * CYCLE_LENGTH - 1] = one;
        columns.push(amt);
        let mut lx = vec![zero; TRACE_LENGTH];
        lx[3 * CYCLE_LENGTH - 1] = one;
        columns.push(lx);

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

        let hash_flag = periodic_values[P_HASH_FLAG];
        let ark1 = &periodic_values[P_ARK1..P_ARK1 + STATE_WIDTH];
        let ark2 = &periodic_values[P_ARK2..P_ARK2 + STATE_WIDTH];
        let reseed = periodic_values[P_RESEED];
        let amt = periodic_values[P_AMT];
        let lx = periodic_values[P_X];
        let link_any = reseed + amt + lx;
        let carry = amt + lx;

        // Ronda de Rescue sobre la via (la forma de circuit_refund).
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

        for i in 0..4 {
            // Todo enlace renace con capacidad limpia.
            result[C_CAP + i] = link_any * next[i];
            // El digest se porta en los enlaces clasicos; en la resiembra
            // NO: ahi el digest siguiente es el testigo (receptor).
            result[C_CARRY + i] = carry * (next[4 + i] - current[4 + i]);
            // La captura: en la resiembra, COL_X := el digest del merge 1
            // (X), que esta en current[4..8].
            result[C_XSET + i] = reseed * (next[COL_X + i] - current[4 + i]);
            // El transporte: fuera de la resiembra, COL_X no se mueve.
            result[C_XKEEP + i] =
                (E::ONE - reseed) * (next[COL_X + i] - current[COL_X + i]);
            // La absorcion: el rate del merge 4 := X transportado.
            result[C_XIN + i] = lx * (next[8 + i] - current[COL_X + i]);
        }
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        let zero = BaseElement::ZERO;
        let mut assertions = Vec::with_capacity(16);

        // Fila 0: capacidad limpia y COL_X a cero. El sobre (f, delta) en
        // 4..12 queda LIBRE: es el testigo, y jamas se publica.
        for i in 0..4 {
            assertions.push(Assertion::single(i, 0, zero));
        }
        for i in 0..4 {
            assertions.push(Assertion::single(COL_X + i, 0, zero));
        }
        // Fila del tercer merge: el importe, publico, y sus tres ceros.
        assertions.push(Assertion::single(8, ROW_AMOUNT, self.amount));
        for i in 1..4 {
            assertions.push(Assertion::single(8 + i, ROW_AMOUNT, zero));
        }
        // Fila final: el compromiso C2.
        for i in 0..4 {
            assertions.push(Assertion::single(4 + i, ROW_P, self.commitment[i]));
        }
        assertions
    }
}

pub struct RefundV2Prover {
    options: ProofOptions,
}

impl RefundV2Prover {
    pub fn new(options: ProofOptions) -> Self {
        RefundV2Prover { options }
    }
}

impl Prover for RefundV2Prover {
    type BaseField = BaseElement;
    type Air = RefundAirV2;
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
    use crate::circuit_refund::{native_refund_commitment, native_refund_commitment_v2};
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

    /// Paridad traza<->nativo: la fila final reproduce C2, el importe queda
    /// donde la asercion lo mira, y COL_X transporta exactamente X.
    #[test]
    fn v2_trace_matches_native_computation() {
        let receptor = digest_from(1000);
        let salt = digest_from(2000);
        let importe = 250_000u64;
        let f = digest_from(9000);
        let delta = 40u64;

        let trace = build_trace(receptor, salt, importe, f, delta);
        let esperado = native_refund_commitment_v2(receptor, salt, importe, f, delta);
        let x = native_merge(
            f,
            [
                BaseElement::new(delta),
                BaseElement::ZERO,
                BaseElement::ZERO,
                BaseElement::ZERO,
            ],
        );

        for i in 0..4 {
            assert_eq!(trace.get(4 + i, ROW_P), esperado[i], "elem {i}");
            assert_eq!(trace.get(COL_X + i, ROW_AMOUNT), x[i], "col X {i}");
        }
        assert_eq!(trace.get(8, ROW_AMOUNT), BaseElement::new(importe));
        for i in 1..4 {
            assert_eq!(trace.get(8 + i, ROW_AMOUNT), BaseElement::ZERO);
        }
    }

    /// La apertura legitima del v2 produce una prueba que verifica.
    #[test]
    fn valid_v2_opening_produces_verifiable_proof() {
        let (receptor, salt, importe) = (digest_from(1000), digest_from(2000), 250_000u64);
        let (f, delta) = (digest_from(9000), 40u64);

        let trace = build_trace(receptor, salt, importe, f, delta);
        let prover = RefundV2Prover::new(default_options());
        let proof = prover.prove(trace).expect("la generacion no deberia fallar");

        let min_opts = AcceptableOptions::OptionSet(vec![prover.options().clone()]);
        let ok = verify::<RefundAirV2, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof,
            RefundPublicInputs {
                commitment: native_refund_commitment_v2(receptor, salt, importe, f, delta),
                amount: BaseElement::new(importe),
            },
            &min_opts,
        );
        assert!(ok.is_ok(), "una apertura v2 correcta debe verificar");
    }

    /// Discriminante: la prueba ata el IMPORTE.
    #[test]
    fn v2_proof_does_not_verify_for_another_amount() {
        let (receptor, salt, importe) = (digest_from(1000), digest_from(2000), 250_000u64);
        let (f, delta) = (digest_from(9000), 40u64);

        let trace = build_trace(receptor, salt, importe, f, delta);
        let prover = RefundV2Prover::new(default_options());
        let proof = prover.prove(trace).expect("prueba");

        let min_opts = AcceptableOptions::OptionSet(vec![prover.options().clone()]);
        let mal = verify::<RefundAirV2, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof,
            RefundPublicInputs {
                commitment: native_refund_commitment_v2(receptor, salt, importe, f, delta),
                amount: BaseElement::new(importe + 1),
            },
            &min_opts,
        );
        assert!(mal.is_err(), "otro importe NO debe verificar");
    }

    /// Discriminante: la prueba ata el DELTA comprometido - la prueba de
    /// (f, delta) no abre el C2 de (f, delta+1). Es la punta nueva del v2.
    #[test]
    fn v2_proof_does_not_verify_for_another_delta() {
        let (receptor, salt, importe) = (digest_from(1000), digest_from(2000), 250_000u64);
        let (f, delta) = (digest_from(9000), 40u64);

        let trace = build_trace(receptor, salt, importe, f, delta);
        let prover = RefundV2Prover::new(default_options());
        let proof = prover.prove(trace).expect("prueba");

        let min_opts = AcceptableOptions::OptionSet(vec![prover.options().clone()]);
        let mal = verify::<RefundAirV2, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof,
            RefundPublicInputs {
                commitment: native_refund_commitment_v2(receptor, salt, importe, f, delta + 1),
                amount: BaseElement::new(importe),
            },
            &min_opts,
        );
        assert!(mal.is_err(), "otro delta NO debe verificar");
    }

    /// Dominio, no marca: la prueba v2 no abre el compromiso v1 (C1).
    #[test]
    fn v2_proof_does_not_open_v1_commitment() {
        let (receptor, salt, importe) = (digest_from(1000), digest_from(2000), 250_000u64);
        let (f, delta) = (digest_from(9000), 40u64);

        let trace = build_trace(receptor, salt, importe, f, delta);
        let prover = RefundV2Prover::new(default_options());
        let proof = prover.prove(trace).expect("prueba");

        let min_opts = AcceptableOptions::OptionSet(vec![prover.options().clone()]);
        let mal = verify::<RefundAirV2, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof,
            RefundPublicInputs {
                commitment: native_refund_commitment(receptor, salt, importe),
                amount: BaseElement::new(importe),
            },
            &min_opts,
        );
        assert!(mal.is_err(), "un C1 no es apertura valida de la forma C2");
    }
}
