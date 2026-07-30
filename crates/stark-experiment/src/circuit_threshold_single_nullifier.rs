//! # Autorización de UN custodio, con nulificador (§47.5, §51)
//!
//! Variante B del experimento de la entrada 33. Misma finalidad que
//! `circuit_threshold_single`: que cada custodio pruebe en su máquina, sin
//! entregar su clave (§41). Difiere en **cómo la capa comprueba que las dos
//! autorizaciones vienen de custodios distintos**, papel que en el circuito
//! conjunto hacía el orden estricto `idx_b − idx_a − 1` (§51.2).
//!
//! ## Por qué el nulificador NO puede derivarse del índice
//!
//! La idea inmediata —publicar `H(dominio, índice)` en vez del índice— **no
//! da privacidad ninguna**: el conjunto tiene 16 posiciones, así que
//! cualquiera calcula los 16 hashes y lo invierte. Un nulificador sobre un
//! dominio diminuto es un identificador con pasos extra.
//!
//! Este circuito lo deriva de la **clave privada**:
//!
//! ```text
//! nulificador = H(NULLIFIER_DOMAIN, clave)
//! ```
//!
//! La clave tiene la entropía del campo, así que el nulificador no se
//! invierte. Y es **determinista**: el mismo custodio produce siempre el
//! mismo, que es justo lo que la capa necesita para exigir que dos
//! autorizaciones difieran.
//!
//! ## Lo que esta variante conserva y lo que cuesta
//!
//! ✅ **Conserva el anonimato dentro del conjunto**: se demuestra que un
//! custodio autorizado firmó, sin revelar cuál. La variante A lo pierde.
//!
//! ⚠️ **Cuesta enlazabilidad entre operaciones.** El nulificador es el mismo
//! en todas las autorizaciones de un custodio, así que un observador puede
//! agrupar: «el custodio desconocido X firmó estas cinco emisiones». No sabe
//! quién es X, pero sabe que es el mismo. Atarlo además al identificador de
//! la operación —`H(dominio, clave, operación)`— lo cerraría, y es
//! exactamente el puente con la otra mitad de la 33: que la autorización
//! cubra los parámetros (§41.4). **No está en este experimento.**
//!
//! ## Y sale más simple de lo que parece
//!
//! Aunque añade un hash, **elimina toda la maquinaria del índice**: sin
//! índice público que atar, sobran `COL_IDX`, `COL_ACC`, el acumulador de
//! bits y su comprobación final. La traza baja a 14 columnas frente a las 16
//! de la variante A.

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

use crate::circuit_threshold::{CustodianPath, CUSTODIAN_DEPTH, CUSTODIAN_DOMAIN, CYCLE_LENGTH};
use crate::merkle::{native_merge, Digest};
use crate::rescue_hash::{apply_sbox, NUM_ROUNDS, STATE_WIDTH};

/// Dominio del nulificador. **Distinto del de identidad de custodio**: si
/// coincidieran, el nulificador publicado sería la propia identidad y
/// revelaría al firmante.
pub const NULLIFIER_DOMAIN: u64 = 0x4E554C4C; // "NULL"

pub const TRACE_LENGTH: usize = 64;

const COL_BIT: usize = STATE_WIDTH; // 12
const COL_KEY: usize = STATE_WIDTH + 1; // 13
pub const TRACE_WIDTH: usize = STATE_WIDTH + 2; // 14

/// Fila donde el estado contiene la raíz del conjunto.
const ROW_ROOT: usize = 39;
/// Fila donde el estado contiene el nulificador. El hash arranca en la 40
/// —fila libre tras la raíz— y consume sus siete rondas hasta la 46.
const ROW_NULL: usize = 47;

// ===== Disposición de las restricciones =====
//
// ⚠️ Cada grupo declara EXACTAMENTE las ranuras que escribe (§38).
const C_HASH: usize = 0; // STATE_WIDTH
const C_CAP: usize = C_HASH + STATE_WIDTH; // 4
const C_PLACE: usize = C_CAP + 4; // 4
const C_BIT_BOOL: usize = C_PLACE + 4; // 1
const C_KEY_INPUT: usize = C_BIT_BOOL + 1; // 1
const C_TRANSPORT: usize = C_KEY_INPUT + 1; // 1
const C_NULL_INIT: usize = C_TRANSPORT + 1; // STATE_WIDTH
const NUM_CONSTRAINTS: usize = C_NULL_INIT + STATE_WIDTH;

// ===== Columnas periódicas =====
const P_HASH_FLAG: usize = 0;
const P_ARK1: usize = 1;
const P_ARK2: usize = P_ARK1 + STATE_WIDTH;
const P_TREE_LINK: usize = P_ARK2 + STATE_WIDTH;
const P_FIRST_ROW: usize = P_TREE_LINK + 1;
const P_NULL_INIT: usize = P_FIRST_ROW + 1;

type Blake3 = Blake3_256<BaseElement>;

/// El nulificador de un custodio, calculado de forma nativa.
pub fn derive_nullifier(key: BaseElement) -> Digest {
    let zero = BaseElement::ZERO;
    native_merge(
        [BaseElement::new(NULLIFIER_DOMAIN), zero, zero, zero],
        [key, zero, zero, zero],
    )
}

/// Construye la traza: subida al árbol y, a continuación, el nulificador.
pub fn build_trace(key: BaseElement, path: &CustodianPath) -> TraceTable<BaseElement> {
    let zero = BaseElement::ZERO;
    let mut rows: Vec<Vec<BaseElement>> = vec![vec![zero; TRACE_WIDTH]; TRACE_LENGTH];

    for row in rows.iter_mut() {
        row[COL_KEY] = key;
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

    // Ciclo 0: derivación de la identidad desde la clave.
    let mut state = [zero; STATE_WIDTH];
    state[4] = BaseElement::new(CUSTODIAN_DOMAIN);
    state[8] = key;
    rows[0][..STATE_WIDTH].copy_from_slice(&state);

    for r in 0..ROW_ROOT {
        let pos = r % CYCLE_LENGTH;
        if pos < NUM_ROUNDS {
            Rp64_256::apply_round(&mut state, pos);
        } else {
            let digest: Digest = [state[4], state[5], state[6], state[7]];
            state = [zero; STATE_WIDTH];
            let level = r / CYCLE_LENGTH; // 0..3
            if level < CUSTODIAN_DEPTH {
                place(&mut state, &digest, level);
            }
        }
        rows[r + 1][..STATE_WIDTH].copy_from_slice(&state);
    }

    // ===== El nulificador, en las filas libres tras la raíz =====
    // Fila 40: estado reiniciado con el dominio del nulificador y la clave.
    let mut null_state = [zero; STATE_WIDTH];
    null_state[4] = BaseElement::new(NULLIFIER_DOMAIN);
    null_state[8] = key;
    rows[ROW_ROOT + 1][..STATE_WIDTH].copy_from_slice(&null_state);

    for r in (ROW_ROOT + 1)..ROW_NULL {
        let pos = r % CYCLE_LENGTH;
        if pos < NUM_ROUNDS {
            Rp64_256::apply_round(&mut null_state, pos);
        }
        rows[r + 1][..STATE_WIDTH].copy_from_slice(&null_state);
    }

    // Bits de dirección, constantes dentro de cada ciclo de subida.
    for level in 0..CUSTODIAN_DEPTH {
        let bit = if path.is_right[level] {
            BaseElement::ONE
        } else {
            zero
        };
        for p in 0..CYCLE_LENGTH {
            rows[(1 + level) * CYCLE_LENGTH + p][COL_BIT] = bit;
        }
    }

    let mut trace = TraceTable::new(TRACE_WIDTH, TRACE_LENGTH);
    trace.fill(
        |s| s.copy_from_slice(&rows[0]),
        |step, s| s.copy_from_slice(&rows[step + 1]),
    );
    trace
}

/// Entradas públicas: la raíz del conjunto **y el nulificador**.
///
/// El índice del custodio **no aparece**: es lo que distingue esta variante
/// de la A. La capa exige que los nulificadores de las dos autorizaciones
/// difieran, sin llegar a saber qué custodios son.
#[derive(Clone, Debug)]
pub struct NullifierThresholdPublicInputs {
    pub custodian_set_root: Digest,
    pub nullifier: Digest,
}

impl ToElements<BaseElement> for NullifierThresholdPublicInputs {
    fn to_elements(&self) -> Vec<BaseElement> {
        let mut v = self.custodian_set_root.to_vec();
        v.extend_from_slice(&self.nullifier);
        v
    }
}

pub struct NullifierThresholdAir {
    context: AirContext<BaseElement>,
    pub_inputs: NullifierThresholdPublicInputs,
}

impl Air for NullifierThresholdAir {
    type BaseField = BaseElement;
    type PublicInputs = NullifierThresholdPublicInputs;

    fn new(trace_info: TraceInfo, pub_inputs: Self::PublicInputs, options: ProofOptions) -> Self {
        assert_eq!(TRACE_WIDTH, trace_info.width());
        let full = vec![TRACE_LENGTH];

        let mut degrees = Vec::with_capacity(NUM_CONSTRAINTS);
        // C_HASH: la ronda de Rescue, grado 7 con ciclo.
        for _ in 0..STATE_WIDTH {
            degrees.push(TransitionConstraintDegree::with_cycles(7, full.clone()));
        }
        // C_CAP (4): grado 1.
        for _ in 0..4 {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }
        // C_PLACE (4): grado 2.
        for _ in 0..4 {
            degrees.push(TransitionConstraintDegree::with_cycles(2, full.clone()));
        }
        // C_BIT_BOOL (1): grado 2 sin ciclo.
        degrees.push(TransitionConstraintDegree::new(2));
        // C_KEY_INPUT (1): un selector periódico.
        degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        // C_TRANSPORT (1): grado 1 sin ciclo.
        degrees.push(TransitionConstraintDegree::new(1));
        // C_NULL_INIT (12): un selector periódico, grado 1.
        for _ in 0..STATE_WIDTH {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }

        assert_eq!(degrees.len(), NUM_CONSTRAINTS, "cuenta de grados");

        NullifierThresholdAir {
            context: AirContext::new(trace_info, degrees, 16, options),
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

        // ⚠️ El indicador de hash cubre AMBOS tramos: la subida al árbol
        // (0..=39) y el nulificador (40..=47). La fila 39 tiene posición 7
        // en su ciclo, así que queda fuera por construcción y es donde se
        // reinicia el estado.
        let mut hash_flag = vec![zero; TRACE_LENGTH];
        for r in 0..=ROW_NULL {
            if r % CYCLE_LENGTH < NUM_ROUNDS {
                hash_flag[r] = one;
            }
        }
        columns.push(hash_flag);

        for ark in [true, false] {
            for i in 0..STATE_WIDTH {
                let mut col = vec![zero; TRACE_LENGTH];
                for r in 0..=ROW_NULL {
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

        let mut tree_link = vec![zero; TRACE_LENGTH];
        for level in 0..CUSTODIAN_DEPTH {
            tree_link[level * CYCLE_LENGTH + 7] = one;
        }
        columns.push(tree_link);

        let mut first_row = vec![zero; TRACE_LENGTH];
        first_row[0] = one;
        columns.push(first_row);

        // Reinicio del estado para el nulificador, en la transición 39 → 40.
        let mut null_init = vec![zero; TRACE_LENGTH];
        null_init[ROW_ROOT] = one;
        columns.push(null_init);

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
        let first_row = periodic[P_FIRST_ROW];
        let null_init = periodic[P_NULL_INIT];

        // La ronda de Rescue. Cubre la subida al árbol y el nulificador:
        // son el mismo hash sobre tramos distintos de la traza.
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

        let bit = next[COL_BIT];

        for i in 0..4 {
            result[C_CAP + i] = tree_link * next[i];

            let d = current[4 + i];
            result[C_PLACE + i] =
                tree_link * ((E::ONE - bit) * (next[4 + i] - d) + bit * (next[8 + i] - d));
        }

        result[C_BIT_BOOL] = current[COL_BIT] * (current[COL_BIT] - E::ONE);

        // La clave entra en la derivación de identidad.
        result[C_KEY_INPUT] = first_row * (current[8] - current[COL_KEY]);

        // La clave es la misma en toda la traza. ⚠️ Esto es lo que ata el
        // nulificador a la identidad probada: sin ello se podría subir al
        // árbol con una clave y nulificar con otra, y dos autorizaciones del
        // mismo custodio pasarían por distintas.
        result[C_TRANSPORT] = next[COL_KEY] - current[COL_KEY];

        // ===== Reinicio del estado para el nulificador =====
        // En la transición 39 → 40 el estado pasa a ser
        // [0,0,0,0, NULLIFIER_DOMAIN,0,0,0, clave,0,0,0].
        for i in 0..4 {
            result[C_NULL_INIT + i] = null_init * next[i];
        }
        result[C_NULL_INIT + 4] =
            null_init * (next[4] - E::from(BaseElement::new(NULLIFIER_DOMAIN)));
        for i in 5..8 {
            result[C_NULL_INIT + i] = null_init * next[i];
        }
        result[C_NULL_INIT + 8] = null_init * (next[8] - current[COL_KEY]);
        for i in 9..STATE_WIDTH {
            result[C_NULL_INIT + i] = null_init * next[i];
        }
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        let zero = BaseElement::ZERO;
        let mut a = Vec::with_capacity(16);

        // Fila 0: capacidad, dominio anclado y relleno.
        for i in 0..4 {
            a.push(Assertion::single(i, 0, zero));
        }
        a.push(Assertion::single(4, 0, BaseElement::new(CUSTODIAN_DOMAIN)));
        for i in 5..8 {
            a.push(Assertion::single(i, 0, zero));
        }
        // El carril llega a la raíz del conjunto autorizado.
        for i in 0..4 {
            a.push(Assertion::single(
                4 + i,
                ROW_ROOT,
                self.pub_inputs.custodian_set_root[i],
            ));
        }
        // Y el nulificador publicado es el que sale del segundo hash.
        for i in 0..4 {
            a.push(Assertion::single(
                4 + i,
                ROW_NULL,
                self.pub_inputs.nullifier[i],
            ));
        }

        a
    }
}

pub struct NullifierThresholdProver {
    options: ProofOptions,
}

impl NullifierThresholdProver {
    pub fn new(options: ProofOptions) -> Self {
        Self { options }
    }
}

impl Prover for NullifierThresholdProver {
    type BaseField = BaseElement;
    type Air = NullifierThresholdAir;
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

    fn get_pub_inputs(&self, trace: &Self::Trace) -> NullifierThresholdPublicInputs {
        NullifierThresholdPublicInputs {
            custodian_set_root: [
                trace.get(4, ROW_ROOT),
                trace.get(5, ROW_ROOT),
                trace.get(6, ROW_ROOT),
                trace.get(7, ROW_ROOT),
            ],
            nullifier: [
                trace.get(4, ROW_NULL),
                trace.get(5, ROW_NULL),
                trace.get(6, ROW_NULL),
                trace.get(7, ROW_NULL),
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
    use crate::circuit_threshold::build_custodian_set;
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

    fn custodian_keys() -> Vec<BaseElement> {
        (1..=4).map(|i| BaseElement::new(0xC0000 + i)).collect()
    }

    fn prove_and_verify(
        key: BaseElement,
        path: &CustodianPath,
        declared: NullifierThresholdPublicInputs,
    ) -> bool {
        let trace = build_trace(key, path);
        let prover = NullifierThresholdProver::new(default_options());
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| prover.prove(trace)));
        match r {
            Err(_) => false,
            Ok(Err(_)) => false,
            Ok(Ok(proof)) => {
                let min_opts = AcceptableOptions::OptionSet(vec![default_options()]);
                verify::<
                    NullifierThresholdAir,
                    Blake3,
                    DefaultRandomCoin<Blake3>,
                    MerkleTree<Blake3>,
                >(proof, declared, &min_opts)
                .is_ok()
            }
        }
    }

    /// El caso honesto.
    #[test]
    fn a_single_custodian_authorization_with_nullifier_verifies() {
        let keys = custodian_keys();
        let (root, paths) = build_custodian_set(&keys);
        let declared = NullifierThresholdPublicInputs {
            custodian_set_root: root,
            nullifier: derive_nullifier(keys[2]),
        };
        assert!(prove_and_verify(keys[2], &paths[2], declared));
    }

    /// Quien no está en el conjunto no autoriza.
    #[test]
    fn a_key_outside_the_set_cannot_authorize() {
        let keys = custodian_keys();
        let (root, paths) = build_custodian_set(&keys);
        let intruso = BaseElement::new(0xBADC0DE);
        let declared = NullifierThresholdPublicInputs {
            custodian_set_root: root,
            nullifier: derive_nullifier(intruso),
        };
        assert!(!prove_and_verify(intruso, &paths[2], declared));
    }

    /// ⚠️ **La propiedad que sostiene la vía B con nulificador.** El mismo
    /// custodio produce SIEMPRE el mismo nulificador. Si no fuera así, podría
    /// presentar dos autorizaciones con nulificadores distintos y el umbral
    /// 2-de-N se caería a 1-de-N.
    #[test]
    fn the_same_custodian_always_yields_the_same_nullifier() {
        let keys = custodian_keys();
        assert_eq!(derive_nullifier(keys[1]), derive_nullifier(keys[1]));
        assert_ne!(derive_nullifier(keys[1]), derive_nullifier(keys[2]));
    }

    /// Y no se puede publicar un nulificador que no sea el de la clave con
    /// la que se probó la pertenencia.
    #[test]
    fn a_custodian_cannot_publish_someone_elses_nullifier() {
        let keys = custodian_keys();
        let (root, paths) = build_custodian_set(&keys);
        let declared = NullifierThresholdPublicInputs {
            custodian_set_root: root,
            nullifier: derive_nullifier(keys[3]), // el de OTRO custodio
        };
        assert!(
            !prove_and_verify(keys[2], &paths[2], declared),
            "SOLIDEZ: el nulificador debe salir de la clave que probo pertenencia \
             (entrada 33, variante B)"
        );
    }

    /// El dominio del nulificador está separado del de identidad: el
    /// nulificador publicado no es la identidad del custodio.
    #[test]
    fn the_nullifier_domain_is_separated_from_the_identity_domain() {
        use crate::circuit_threshold::derive_custodian_id;
        let k = BaseElement::new(0xC0FFEE);
        assert_ne!(derive_nullifier(k), derive_custodian_id(k));
    }
}
