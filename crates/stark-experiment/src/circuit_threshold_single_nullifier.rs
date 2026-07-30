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
//! ## La autorización cubre LA OPERACIÓN (§55)
//!
//! El nulificador se deriva de la clave **y del compromiso de la operación**:
//!
//! ```text
//! nulificador = H(NULLIFIER_DOMAIN, clave, operación)
//! ```
//!
//! Esa sola atadura cierra dos cosas que parecían separadas:
//!
//! ✅ **No hay reproducción.** La prueba solo autoriza *esta* operación: sus
//! entradas públicas la nombran, y la capa comprueba que coincide con la que
//! está ejecutando. Antes, dos autorizaciones para emitir 1.000 a Alicia
//! servían para emitir 1.000.000 a Bob (§54.4).
//!
//! ✅ **No hay enlazabilidad.** El nulificador cambia con cada operación, así
//! que ya no se puede agrupar «el custodio desconocido X firmó estas cinco
//! emisiones» (§52.4).
//!
//! Y **conserva** lo que hacía falta: dentro de UNA operación, el mismo
//! custodio produce siempre el mismo nulificador, que es lo que permite
//! exigir que las dos autorizaciones sean de custodios distintos.
//!
//! ✅ **Conserva el anonimato dentro del conjunto**: se demuestra que un
//! custodio autorizado firmó, sin revelar cuál. La variante A lo pierde.
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
    verify, AcceptableOptions, Air, AirContext, Assertion, AuxRandElements, CompositionPoly,
    CompositionPolyTrace, Proof,
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
/// Compromiso de la operacion autorizada, constante en toda la traza.
const COL_OP: usize = STATE_WIDTH + 2; // 14..18
pub const TRACE_WIDTH: usize = STATE_WIDTH + 6; // 18

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
const C_TRANSPORT: usize = C_KEY_INPUT + 1; // 5 (clave + 4 de operacion)
const C_NULL_INIT: usize = C_TRANSPORT + 5; // STATE_WIDTH
/// Publica para que la tabla de §52/§55 la lea del codigo y no se quede
/// rancia: escribirla a mano ya fallo una vez (35 cuando eran 39).
pub const NUM_CONSTRAINTS: usize = C_NULL_INIT + STATE_WIDTH;

// ===== Columnas periódicas =====
const P_HASH_FLAG: usize = 0;
const P_ARK1: usize = 1;
const P_ARK2: usize = P_ARK1 + STATE_WIDTH;
const P_TREE_LINK: usize = P_ARK2 + STATE_WIDTH;
const P_FIRST_ROW: usize = P_TREE_LINK + 1;
const P_NULL_INIT: usize = P_FIRST_ROW + 1;

type Blake3 = Blake3_256<BaseElement>;

// ===== Compromiso de la operacion autorizada =====
//
// Los custodios firman un `Digest` que resume QUE se autoriza. Este es el
// puente entre la autorizacion y el circuito de la operacion: la capa calcula
// el compromiso desde las entradas publicas de la operacion y exige que sea
// el que los custodios firmaron.

/// Dominios de operacion. **Uno por tipo**, para que una autorizacion de
/// congelacion no pueda reutilizarse como autorizacion de emision.
pub const OP_MINT: u64 = 0x4D494E54; // "MINT"
pub const OP_MINT_PENDING: u64 = 0x4D504E44; // "MPND"
pub const OP_FREEZE: u64 = 0x46525A45; // "FRZE"
pub const OP_RECOVERY: u64 = 0x5245434F; // "RECO"
pub const OP_GOVERNANCE: u64 = 0x474F5652; // "GOVR"

/// Resume los parametros de una operacion en un `Digest` que los custodios
/// firman.
///
/// Esponja sobre la permutacion Rescue: capacidad `state[0..4]` con el dominio
/// en `state[0]`, ritmo `state[4..12]` de ocho elementos, modo sobrescritura.
///
/// ⚠️ **Supone longitud FIJA por dominio.** No lleva relleno, asi que dos
/// mensajes del mismo dominio con longitudes distintas podrian colisionar
/// (`[a]` y `[a, 0]` dan lo mismo). Cada operacion tiene un numero fijo de
/// parametros, asi que la suposicion se cumple hoy —y los dominios impiden
/// colisiones ENTRE operaciones—. **Si alguna operacion pasa a tener
/// parametros de longitud variable, esto necesita una regla de relleno antes
/// de usarse.** Queda escrito porque es la clase de suposicion que se olvida.
pub fn commit_operation(domain: u64, elements: &[BaseElement]) -> Digest {
    let zero = BaseElement::ZERO;
    let mut state = [zero; STATE_WIDTH];
    state[0] = BaseElement::new(domain);
    for chunk in elements.chunks(8) {
        for i in 0..8 {
            state[4 + i] = if i < chunk.len() { chunk[i] } else { zero };
        }
        Rp64_256::apply_permutation(&mut state);
    }
    [state[4], state[5], state[6], state[7]]
}

/// El nulificador de un custodio **para una operacion concreta**.
///
/// Una sola permutacion absorbe los seis elementos: dominio y clave en la
/// mitad izquierda, el compromiso de la operacion en la derecha. No cuesta
/// filas adicionales respecto a la version sin atadura.
pub fn derive_nullifier(key: BaseElement, operation: Digest) -> Digest {
    let zero = BaseElement::ZERO;
    native_merge(
        [BaseElement::new(NULLIFIER_DOMAIN), key, zero, zero],
        operation,
    )
}

/// Construye la traza: subida al árbol y, a continuación, el nulificador.
pub fn build_trace(
    key: BaseElement,
    path: &CustodianPath,
    operation: Digest,
) -> TraceTable<BaseElement> {
    let zero = BaseElement::ZERO;
    let mut rows: Vec<Vec<BaseElement>> = vec![vec![zero; TRACE_WIDTH]; TRACE_LENGTH];

    for row in rows.iter_mut() {
        row[COL_KEY] = key;
        row[COL_OP..COL_OP + 4].copy_from_slice(&operation);
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
    null_state[5] = key;
    null_state[8..12].copy_from_slice(&operation);
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
    /// ⚠️ **Compromiso de la operacion autorizada.** Sin esto la prueba
    /// autorizaria «algo» y se podria reproducir en otra operacion (§54.4).
    pub operation: Digest,
}

impl ToElements<BaseElement> for NullifierThresholdPublicInputs {
    fn to_elements(&self) -> Vec<BaseElement> {
        let mut v = self.custodian_set_root.to_vec();
        v.extend_from_slice(&self.nullifier);
        v.extend_from_slice(&self.operation);
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
        // C_TRANSPORT (5): clave y las 4 de operacion, grado 1 sin ciclo.
        for _ in 0..5 {
            degrees.push(TransitionConstraintDegree::new(1));
        }
        // C_NULL_INIT (12): un selector periódico, grado 1.
        for _ in 0..STATE_WIDTH {
            degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        }

        assert_eq!(degrees.len(), NUM_CONSTRAINTS, "cuenta de grados");

        NullifierThresholdAir {
            context: AirContext::new(trace_info, degrees, 20, options),
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
        // Y la operacion tambien: si variara entre filas, el nulificador no
        // seria el de la operacion declarada.
        for i in 0..4 {
            result[C_TRANSPORT + 1 + i] = next[COL_OP + i] - current[COL_OP + i];
        }

        // ===== Reinicio del estado para el nulificador =====
        // En la transición 39 → 40 el estado pasa a ser
        // [0,0,0,0, NULLIFIER_DOMAIN, clave,0,0, operacion(4)].
        for i in 0..4 {
            result[C_NULL_INIT + i] = null_init * next[i];
        }
        result[C_NULL_INIT + 4] =
            null_init * (next[4] - E::from(BaseElement::new(NULLIFIER_DOMAIN)));
        result[C_NULL_INIT + 5] = null_init * (next[5] - current[COL_KEY]);
        for i in 6..8 {
            result[C_NULL_INIT + i] = null_init * next[i];
        }
        for i in 0..4 {
            result[C_NULL_INIT + 8 + i] = null_init * (next[8 + i] - current[COL_OP + i]);
        }
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        let zero = BaseElement::ZERO;
        let mut a = Vec::with_capacity(20);

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
        // ⚠️ La operacion de la traza es la publicada: sin esto se podria
        // hashear una operacion y declarar otra.
        for i in 0..4 {
            a.push(Assertion::single(
                COL_OP + i,
                0,
                self.pub_inputs.operation[i],
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
            operation: [
                trace.get(COL_OP, 0),
                trace.get(COL_OP + 1, 0),
                trace.get(COL_OP + 2, 0),
                trace.get(COL_OP + 3, 0),
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

/// Por que se rechaza un par de autorizaciones.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PairRejection {
    /// Una de las dos pruebas no verifica.
    InvalidProof,
    /// ⚠️ Las dos vienen del **mismo custodio**: mismo nulificador. Sin
    /// esta comprobacion el umbral 2-de-N seria 1-de-N.
    SameCustodian,
    /// ⚠️ Alguna prueba autoriza **otra operacion**. Es lo que impide
    /// reproducir un par valido en una operacion distinta (§54.4).
    WrongOperation,
    /// ⚠️ Alguna prueba demuestra pertenencia a **otro conjunto de
    /// custodios**. Un atacante puede construirse un conjunto con dieciseis
    /// claves suyas y firmar dos veces con dos de ellas: los nulificadores
    /// serian distintos y el par pasaria. La raiz la pone la **capa**, no
    /// la prueba.
    WrongCustodianSet,
}

/// Verifica un par de autorizaciones de custodio y **decide si constituyen
/// el umbral 2-de-N**.
///
/// Esta funcion es donde vive el umbral tras separar los carriles (§51.2).
/// En `circuit_threshold` lo imponia el orden estricto `idx_b - idx_a - 1`
/// dentro del circuito; con dos pruebas independientes no hay traza conjunta
/// donde imponerlo, asi que se impone aqui.
///
/// ⚠️ **`expected_root` la aporta la capa**, no se lee de las pruebas. Es
/// la defensa contra `WrongCustodianSet`, y es la comprobacion que un lector
/// desprevenido omitiria: sin ella, cada prueba es valida por separado y el
/// par tambien, pero los custodios son del atacante.
///
/// ⚠️ **`expected_operation` tambien la aporta la capa**, y por la misma
/// razon: es lo que ata la autorizacion a **esta** operacion y no a
/// cualquiera. Sin ella, dos custodios que autorizan emitir 1.000 a Alicia
/// estarian autorizando de hecho cualquier emision, porque sus pruebas no
/// dirian nada del importe ni del destinatario (§54.4, cerrado en §55).
///
/// # Lo que sigue faltando para produccion
///
/// Esta funcion es correcta en lo que comprueba, pero el camino completo
/// exige aun **sustituir `ThresholdAuth` en los cinco circuitos** que lo
/// consumen (`mint`, `mint_to_pending`, `freeze`, `recovery`,
/// `governance`), y eso es cirugia en la creacion de dinero: va con la
/// cautela de §50, con test discriminante antes de tocar nada.
pub fn verify_threshold_pair(
    proof_a: Proof,
    inputs_a: NullifierThresholdPublicInputs,
    proof_b: Proof,
    inputs_b: NullifierThresholdPublicInputs,
    expected_root: Digest,
    expected_operation: Digest,
    accepted: &AcceptableOptions,
) -> Result<(), PairRejection> {
    // 1. Las dos autorizan sobre el conjunto que la capa dice, no sobre uno
    //    que traiga la prueba.
    if inputs_a.custodian_set_root != expected_root
        || inputs_b.custodian_set_root != expected_root
    {
        return Err(PairRejection::WrongCustodianSet);
    }

    // 2. Y autorizan ESTA operacion, no otra. Sin esto un par valido se
    //    reproduce en cualquier operacion posterior.
    if inputs_a.operation != expected_operation || inputs_b.operation != expected_operation {
        return Err(PairRejection::WrongOperation);
    }

    // 3. Son custodios DISTINTOS. Aqui es donde el umbral es umbral.
    if inputs_a.nullifier == inputs_b.nullifier {
        return Err(PairRejection::SameCustodian);
    }

    // 4. Y las dos pruebas son validas.
    let ok = |p: Proof, i: NullifierThresholdPublicInputs| {
        verify::<NullifierThresholdAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            p, i, accepted,
        )
        .is_ok()
    };
    if !ok(proof_a, inputs_a) || !ok(proof_b, inputs_b) {
        return Err(PairRejection::InvalidProof);
    }

    Ok(())
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
        op: Digest,
        declared: NullifierThresholdPublicInputs,
    ) -> bool {
        let trace = build_trace(key, path, op);
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
        let op = operacion(1);
        let declared = NullifierThresholdPublicInputs {
            custodian_set_root: root,
            nullifier: derive_nullifier(keys[2], op),
            operation: op,
        };
        assert!(prove_and_verify(keys[2], &paths[2], op, declared));
    }

    /// Quien no está en el conjunto no autoriza.
    #[test]
    fn a_key_outside_the_set_cannot_authorize() {
        let keys = custodian_keys();
        let (root, paths) = build_custodian_set(&keys);
        let intruso = BaseElement::new(0xBADC0DE);
        let op = operacion(1);
        let declared = NullifierThresholdPublicInputs {
            custodian_set_root: root,
            nullifier: derive_nullifier(intruso, op),
            operation: op,
        };
        assert!(!prove_and_verify(intruso, &paths[2], op, declared));
    }

    /// ⚠️ **La propiedad que sostiene la vía B con nulificador.** El mismo
    /// custodio produce SIEMPRE el mismo nulificador. Si no fuera así, podría
    /// presentar dos autorizaciones con nulificadores distintos y el umbral
    /// 2-de-N se caería a 1-de-N.
    #[test]
    fn the_same_custodian_always_yields_the_same_nullifier() {
        let keys = custodian_keys();
        let op = operacion(1);
        // Dentro de UNA operacion es estable: es lo que permite exigir que
        // las dos autorizaciones sean de custodios distintos.
        assert_eq!(derive_nullifier(keys[1], op), derive_nullifier(keys[1], op));
        assert_ne!(derive_nullifier(keys[1], op), derive_nullifier(keys[2], op));
    }

    /// ⚠️ **Y entre operaciones cambia**, que es lo que cierra la
    /// enlazabilidad de §52.4: ya no se puede agrupar «el custodio desconocido
    /// X firmo estas cinco emisiones».
    #[test]
    fn the_nullifier_changes_across_operations() {
        let keys = custodian_keys();
        assert_ne!(
            derive_nullifier(keys[1], operacion(1)),
            derive_nullifier(keys[1], operacion(2)),
            "el nulificador debe variar con la operacion, o es un pseudonimo estable"
        );
    }

    /// Y no se puede publicar un nulificador que no sea el de la clave con
    /// la que se probó la pertenencia.
    #[test]
    fn a_custodian_cannot_publish_someone_elses_nullifier() {
        let keys = custodian_keys();
        let (root, paths) = build_custodian_set(&keys);
        let op = operacion(1);
        let declared = NullifierThresholdPublicInputs {
            custodian_set_root: root,
            nullifier: derive_nullifier(keys[3], op), // el de OTRO custodio
            operation: op,
        };
        assert!(
            !prove_and_verify(keys[2], &paths[2], op, declared),
            "SOLIDEZ: el nulificador debe salir de la clave que probo pertenencia \
             (entrada 33, variante B)"
        );
    }

    // ===== El umbral 2-de-N, que ahora vive en la capa =====

    /// Un compromiso de operacion de juguete. En produccion seria el hash de
    /// los parametros reales (destinatario, importe, contador).
    fn operacion(n: u64) -> Digest {
        [
            BaseElement::new(0xDEAD_0000 + n),
            BaseElement::ZERO,
            BaseElement::ZERO,
            BaseElement::ZERO,
        ]
    }

    fn autorizar(
        key: BaseElement,
        path: &CustodianPath,
        op: Digest,
    ) -> (Proof, NullifierThresholdPublicInputs) {
        let trace = build_trace(key, path, op);
        let prover = NullifierThresholdProver::new(default_options());
        let inputs = NullifierThresholdPublicInputs {
            custodian_set_root: [
                trace.get(4, ROW_ROOT),
                trace.get(5, ROW_ROOT),
                trace.get(6, ROW_ROOT),
                trace.get(7, ROW_ROOT),
            ],
            nullifier: derive_nullifier(key, op),
            operation: op,
        };
        (prover.prove(trace).expect("deberia probar"), inputs)
    }

    fn opciones() -> AcceptableOptions {
        AcceptableOptions::OptionSet(vec![default_options()])
    }

    /// Dos custodios distintos del conjunto: el umbral se cumple.
    #[test]
    fn two_distinct_custodians_meet_the_threshold() {
        let keys = custodian_keys();
        let (root, paths) = build_custodian_set(&keys);
        let op = operacion(7);
        let (pa, ia) = autorizar(keys[1], &paths[1], op);
        let (pb, ib) = autorizar(keys[2], &paths[2], op);
        assert_eq!(
            verify_threshold_pair(pa, ia, pb, ib, root, op, &opciones()),
            Ok(())
        );
    }

    /// ⚠️ **La razon de ser de esta funcion.** El mismo custodio firmando
    /// dos veces produce el mismo nulificador. Sin esta comprobacion el
    /// umbral 2-de-N seria 1-de-N: cualquiera con UNA clave de custodio
    /// podria emitir dinero.
    #[test]
    fn the_same_custodian_twice_does_not_meet_the_threshold() {
        let keys = custodian_keys();
        let (root, paths) = build_custodian_set(&keys);
        let op = operacion(7);
        let (pa, ia) = autorizar(keys[2], &paths[2], op);
        let (pb, ib) = autorizar(keys[2], &paths[2], op);
        assert_eq!(
            verify_threshold_pair(pa, ia, pb, ib, root, op, &opciones()),
            Err(PairRejection::SameCustodian),
            "SOLIDEZ: dos autorizaciones del mismo custodio no son un umbral"
        );
    }

    /// ⚠️ **El ataque que un lector desprevenido omitiria.** El atacante se
    /// construye SU conjunto de custodios con dieciseis claves suyas y firma
    /// dos veces con dos de ellas. Las dos pruebas son validas, los
    /// nulificadores son distintos, y el par pasaria... si la raiz saliera de
    /// las pruebas en vez de ponerla la capa.
    #[test]
    fn an_attacker_cannot_bring_their_own_custodian_set() {
        let keys = custodian_keys();
        let (root_real, _) = build_custodian_set(&keys);

        let mias: Vec<BaseElement> = (1..=4).map(|i| BaseElement::new(0xA77AC0 + i)).collect();
        let (_root_mia, paths_mias) = build_custodian_set(&mias);
        let op = operacion(7);
        let (pa, ia) = autorizar(mias[0], &paths_mias[0], op);
        let (pb, ib) = autorizar(mias[1], &paths_mias[1], op);

        assert_eq!(
            verify_threshold_pair(pa, ia, pb, ib, root_real, op, &opciones()),
            Err(PairRejection::WrongCustodianSet),
            "SOLIDEZ: la raiz del conjunto la pone la capa, no la prueba"
        );
    }

    /// Y un custodio real emparejado con uno del conjunto del atacante
    /// tampoco cuela: basta que UNA de las dos sea de otro conjunto.
    #[test]
    fn one_real_and_one_forged_custodian_do_not_meet_the_threshold() {
        let keys = custodian_keys();
        let (root_real, paths) = build_custodian_set(&keys);
        let mias: Vec<BaseElement> = (1..=4).map(|i| BaseElement::new(0xA77AC0 + i)).collect();
        let (_r, paths_mias) = build_custodian_set(&mias);

        let op = operacion(7);
        let (pa, ia) = autorizar(keys[1], &paths[1], op);
        let (pb, ib) = autorizar(mias[0], &paths_mias[0], op);
        assert_eq!(
            verify_threshold_pair(pa, ia, pb, ib, root_real, op, &opciones()),
            Err(PairRejection::WrongCustodianSet)
        );
    }

    /// ⚠️ **La constancia de `COL_OP`, que es lo que sostiene la atadura.**
    ///
    /// La cadena es: asercion en la fila 0 (la operacion declarada es la de
    /// la traza) -> constancia entre filas -> el hash del nulificador lee
    /// `COL_OP` en la fila 39. Si la constancia estuviera muerta —como lo
    /// estaba la de `COL_SALT` en §50.7— un custodio pondria la operacion
    /// declarada en la fila 0 y otra en la 39, obteniendo **nulificadores
    /// distintos para si mismo**: el umbral 2-de-N volveria a ser 1-de-N.
    ///
    /// El barrido de disposiciones (§53) dice que las ranuras estan bien
    /// asignadas, pero eso **no prueba que la restriccion sea correcta**
    /// (§53.5). Esto si.
    #[test]
    fn an_operation_inconsistent_across_the_trace_is_rejected() {
        let keys = custodian_keys();
        let (root, paths) = build_custodian_set(&keys);
        let declarada = operacion(7);
        let otra = operacion(8);

        let mut trace = build_trace(keys[2], &paths[2], declarada);
        // La fila 0 conserva la operacion declarada; el resto pasa a otra.
        for row in 1..TRACE_LENGTH {
            for i in 0..4 {
                trace.set(COL_OP + i, row, otra[i]);
            }
        }

        let prover = NullifierThresholdProver::new(default_options());
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| prover.prove(trace)));
        let verifica = match r {
            Err(_) | Ok(Err(_)) => false,
            Ok(Ok(proof)) => {
                let declared = NullifierThresholdPublicInputs {
                    custodian_set_root: root,
                    nullifier: derive_nullifier(keys[2], otra),
                    operation: declarada,
                };
                verify::<
                    NullifierThresholdAir,
                    Blake3,
                    DefaultRandomCoin<Blake3>,
                    MerkleTree<Blake3>,
                >(proof, declared, &opciones())
                .is_ok()
            }
        };
        assert!(
            !verifica,
            "SOLIDEZ: la operacion debe ser constante en toda la traza; si no, \
             un custodio genera nulificadores distintos para si mismo y el \
             umbral 2-de-N cae a 1-de-N (entrada 33 / §55)"
        );
    }

    /// ⚠️ **El ataque que §55 cierra.** Dos custodios autorizan la operacion
    /// 7. Alguien reenvia esas mismas dos pruebas para ejecutar la operacion
    /// 8. Antes de atar la operacion al nulificador, esto funcionaba.
    #[test]
    fn a_valid_pair_cannot_be_replayed_on_another_operation() {
        let keys = custodian_keys();
        let (root, paths) = build_custodian_set(&keys);
        let autorizada = operacion(7);
        let otra = operacion(8);

        let (pa, ia) = autorizar(keys[1], &paths[1], autorizada);
        let (pb, ib) = autorizar(keys[2], &paths[2], autorizada);

        assert_eq!(
            verify_threshold_pair(pa, ia, pb, ib, root, otra, &opciones()),
            Err(PairRejection::WrongOperation),
            "SOLIDEZ: un par valido para una operacion no autoriza otra"
        );
    }

    // ===== El compromiso de operacion: el puente con los cinco circuitos =====

    /// Los parametros de una emision, tal como los expondria `circuit_mint`.
    fn params_mint(amount: u64, supply_old: u64, supply_new: u64) -> Vec<BaseElement> {
        let root_old = [BaseElement::new(11); 4];
        let root_new = [BaseElement::new(22); 4];
        let mut v = root_old.to_vec();
        v.extend_from_slice(&root_new);
        v.push(BaseElement::new(amount));
        v.push(BaseElement::new(supply_old));
        v.push(BaseElement::new(supply_new));
        v.push(BaseElement::new(1_000_000_000));
        v
    }

    #[test]
    fn the_operation_commitment_is_deterministic() {
        let p = params_mint(1_000, 0, 1_000);
        assert_eq!(
            commit_operation(OP_MINT, &p),
            commit_operation(OP_MINT, &p)
        );
    }

    /// ⚠️ **Separacion por dominio.** Los mismos parametros bajo otro tipo de
    /// operacion dan otro compromiso: una autorizacion de congelacion no vale
    /// como autorizacion de emision.
    #[test]
    fn operation_domains_are_separated() {
        let p = params_mint(1_000, 0, 1_000);
        assert_ne!(
            commit_operation(OP_MINT, &p),
            commit_operation(OP_FREEZE, &p),
            "SOLIDEZ: sin separacion de dominio, una autorizacion sirve para \
             cualquier tipo de operacion"
        );
    }

    /// Cambiar cualquier parametro cambia el compromiso.
    #[test]
    fn every_parameter_is_covered_by_the_commitment() {
        let base = commit_operation(OP_MINT, &params_mint(1_000, 0, 1_000));
        assert_ne!(base, commit_operation(OP_MINT, &params_mint(1_001, 0, 1_000)));
        assert_ne!(base, commit_operation(OP_MINT, &params_mint(1_000, 1, 1_000)));
        assert_ne!(base, commit_operation(OP_MINT, &params_mint(1_000, 0, 1_001)));
    }

    /// ⚠️ **La prueba de que el puente funciona.** Dos custodios autorizan
    /// emitir 1.000. Alguien intenta usar esas autorizaciones para emitir
    /// 1.000.000. El compromiso no coincide y el par se rechaza.
    ///
    /// Es el mismo agujero de §54.4, ahora **entre circuitos**: sin esta
    /// atadura, la autorizacion y la operacion serian dos cosas sueltas que
    /// nadie obliga a corresponderse.
    #[test]
    fn an_authorization_for_one_mint_does_not_authorize_another() {
        let keys = custodian_keys();
        let (root, paths) = build_custodian_set(&keys);

        let autorizada = commit_operation(OP_MINT, &params_mint(1_000, 0, 1_000));
        let pretendida = commit_operation(OP_MINT, &params_mint(1_000_000, 0, 1_000_000));

        let (pa, ia) = autorizar(keys[1], &paths[1], autorizada);
        let (pb, ib) = autorizar(keys[2], &paths[2], autorizada);

        assert_eq!(
            verify_threshold_pair(pa, ia, pb, ib, root, pretendida, &opciones()),
            Err(PairRejection::WrongOperation),
            "SOLIDEZ: una autorizacion para emitir 1.000 no autoriza emitir \
             1.000.000 (entrada 33 / §56)"
        );
    }

    /// El dominio del nulificador está separado del de identidad: el
    /// nulificador publicado no es la identidad del custodio.
    #[test]
    fn the_nullifier_domain_is_separated_from_the_identity_domain() {
        use crate::circuit_threshold::derive_custodian_id;
        let k = BaseElement::new(0xC0FFEE);
        assert_ne!(derive_nullifier(k, operacion(1)), derive_custodian_id(k));
    }
}
