//! # Autorización de UN custodio, en prueba independiente (§47.5, §51)
//!
//! Experimento de la entrada 33. `circuit_threshold` demuestra en una
//! sola traza que **dos** custodios autorizaron, y para construirla la
//! capa necesita **las dos claves en crudo** (§41): ese es el fallo de
//! modelo de confianza de la 32.
//!
//! Este circuito prueba lo mismo para **un solo custodio**, de modo que
//! cada uno pueda generarlo en su máquina sin entregar su clave. La capa
//! recoge dos pruebas y **exige ambas** (vía B, §47.3): no se componen,
//! se verifican por separado.
//!
//! # ⚠️ VARIANTE NO ELEGIDA — NO CABLEAR EN PRODUCCION
//!
//! El proyecto eligio la **variante B** (`circuit_threshold_single_nullifier`)
//! el 30-07-2026 (§52.7): esta revela **que** custodios firman, y eso rompe
//! el anonimato dentro del conjunto que `circuit_threshold` ya declaraba.
//!
//! Este fichero **se conserva a proposito**: es la comparacion medida que
//! sostiene la tabla de §52.1, y el proyecto marca lo descartado en vez de
//! borrarlo. Sus tests siguen corriendo como regresion de esa medicion.
//! **Pero no debe conectarse a ninguna operacion privilegiada.**
//!
//! ## VARIANTE A: el índice del custodio es PÚBLICO
//!
//! Al separar los carriles desaparece la restricción de orden estricto
//! `idx_b − idx_a − 1`, que era lo único que garantizaba **dos custodios
//! distintos** (§51.2). Con pruebas independientes hay que reimponerlo
//! fuera: la capa compara los índices de las dos pruebas y exige que
//! difieran.
//!
//! ⚠️ **Coste de privacidad, declarado.** `circuit_threshold` mantiene en
//! secreto *qué* custodios firman —solo se sabe que son dos del conjunto—.
//! Esta variante **revela cuáles**. Es un cambio del modelo de confianza,
//! no un detalle de implementación: quien observe la cadena sabe qué dos
//! custodios autorizaron cada emisión. Ver la variante B
//! (`circuit_threshold_single_nullifier`) para la alternativa que lo evita.
//!
//! ## Qué demuestra
//!
//! 1. Conocimiento de la preimagen: `identidad = H(CUSTODIAN_DOMAIN, clave)`.
//! 2. Pertenencia: esa identidad sube por su camino hasta la raíz del
//!    conjunto de custodios, que es entrada pública.
//! 3. El índice declarado **es** la posición realmente demostrada: el
//!    acumulador de bits de camino se compara con él en la fila de la raíz.
//!    Sin esto, el índice sería un número suelto y la comparación que hace
//!    la capa no valdría nada.
//!
//! ## Qué NO demuestra, y va fuera
//!
//! Que este custodio sea **distinto** del otro firmante: eso lo comprueba
//! la capa comparando los dos índices públicos. Y que la autorización cubra
//! los parámetros de la operación: es la otra mitad de la 33 (§41.4), que
//! ata un mensaje a la firma y **no está en este experimento**.

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
use crate::merkle::Digest;
use crate::rescue_hash::{apply_sbox, NUM_ROUNDS, STATE_WIDTH};

/// 40 filas bastan: 4 niveles de subida + la fila de la raíz, redondeado a
/// potencia de dos. El circuito de dos carriles usa 64 porque necesita 24
/// filas más para el segmento de rango del orden estricto, que aquí no
/// existe.
pub const TRACE_LENGTH: usize = 64;

/// Columna del bit de dirección del nivel en curso.
const COL_BIT: usize = STATE_WIDTH; // 12
/// Clave privada del custodio, constante en toda la traza.
const COL_KEY: usize = STATE_WIDTH + 1; // 13
/// Índice declarado del custodio dentro del conjunto.
const COL_IDX: usize = STATE_WIDTH + 2; // 14
/// Acumulador que reconstruye el índice desde los bits de camino.
const COL_ACC: usize = STATE_WIDTH + 3; // 15

pub const TRACE_WIDTH: usize = STATE_WIDTH + 4; // 16

/// Fila donde el estado contiene la raíz del conjunto.
const ROW_ROOT: usize = 39;

// ===== Disposición de las restricciones =====
//
// ⚠️ Cada grupo declara EXACTAMENTE las ranuras que escribe, y el grupo
// siguiente arranca donde termina el anterior. Contar mal aquí es el
// defecto de §38, que produjo tres fallos de solidez (§39, §50, §50.7).
const C_HASH: usize = 0; // STATE_WIDTH
const C_CAP: usize = C_HASH + STATE_WIDTH; // 4
const C_PLACE: usize = C_CAP + 4; // 4
const C_BIT_BOOL: usize = C_PLACE + 4; // 1
const C_KEY_INPUT: usize = C_BIT_BOOL + 1; // 1
const C_ACC: usize = C_KEY_INPUT + 1; // 1
const C_ACC_FINAL: usize = C_ACC + 1; // 1
const C_TRANSPORT: usize = C_ACC_FINAL + 1; // 2
const NUM_CONSTRAINTS: usize = C_TRANSPORT + 2;

// ===== Columnas periódicas =====
const P_HASH_FLAG: usize = 0;
const P_ARK1: usize = 1;
const P_ARK2: usize = P_ARK1 + STATE_WIDTH;
const P_TREE_LINK: usize = P_ARK2 + STATE_WIDTH;
const P_POW2: usize = P_TREE_LINK + 1;
const P_FIRST_ROW: usize = P_POW2 + 1;
const P_SEL_ROOT: usize = P_FIRST_ROW + 1;

type Blake3 = Blake3_256<BaseElement>;

/// Construye la traza de la autorización de **un** custodio.
pub fn build_trace(
    key: BaseElement,
    index: u64,
    path: &CustodianPath,
) -> TraceTable<BaseElement> {
    let zero = BaseElement::ZERO;
    let mut rows: Vec<Vec<BaseElement>> = vec![vec![zero; TRACE_WIDTH]; TRACE_LENGTH];

    for row in rows.iter_mut() {
        row[COL_KEY] = key;
        row[COL_IDX] = BaseElement::new(index);
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

    let mut acc = zero;

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
                // El acumulador incorpora el bit de ESTE nivel.
                if path.is_right[level] {
                    acc += BaseElement::new(1u64 << level);
                }
            }
        }
        rows[r + 1][..STATE_WIDTH].copy_from_slice(&state);
        rows[r + 1][COL_ACC] = acc;
    }

    // El acumulador permanece tras la última fila activa.
    for r in ROW_ROOT..TRACE_LENGTH {
        rows[r][COL_ACC] = acc;
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

/// Entradas públicas: la raíz del conjunto **y el índice del custodio**.
///
/// ⚠️ El índice es público a propósito: es lo que permite a la capa exigir
/// que las dos autorizaciones vengan de custodios distintos, papel que en
/// el circuito conjunto hacía el orden estricto (§51.2). El coste es que se
/// revela **quién** firmó.
#[derive(Clone, Debug)]
pub struct SingleThresholdPublicInputs {
    pub custodian_set_root: Digest,
    pub custodian_index: BaseElement,
}

impl ToElements<BaseElement> for SingleThresholdPublicInputs {
    fn to_elements(&self) -> Vec<BaseElement> {
        let mut v = self.custodian_set_root.to_vec();
        v.push(self.custodian_index);
        v
    }
}

pub struct SingleThresholdAir {
    context: AirContext<BaseElement>,
    pub_inputs: SingleThresholdPublicInputs,
}

impl Air for SingleThresholdAir {
    type BaseField = BaseElement;
    type PublicInputs = SingleThresholdPublicInputs;

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
        // C_ACC (1): **DOS columnas periódicas** (enlace y potencia de dos).
        // Declarar una sola cuando hay dos hace que winterfell calcule el
        // doble de grado — el mismo cuidado que el circuito de dos carriles.
        degrees.push(TransitionConstraintDegree::with_cycles(
            1,
            vec![TRACE_LENGTH, TRACE_LENGTH],
        ));
        // C_ACC_FINAL (1): un selector.
        degrees.push(TransitionConstraintDegree::with_cycles(1, full.clone()));
        // C_TRANSPORT (2): grado 1 sin ciclo.
        for _ in 0..2 {
            degrees.push(TransitionConstraintDegree::new(1));
        }

        assert_eq!(degrees.len(), NUM_CONSTRAINTS, "cuenta de grados");

        SingleThresholdAir {
            context: AirContext::new(trace_info, degrees, 14, options),
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

        let mut hash_flag = vec![zero; TRACE_LENGTH];
        for r in 0..=ROW_ROOT {
            if r % CYCLE_LENGTH < NUM_ROUNDS {
                hash_flag[r] = one;
            }
        }
        columns.push(hash_flag);

        for ark in [true, false] {
            for i in 0..STATE_WIDTH {
                let mut col = vec![zero; TRACE_LENGTH];
                for r in 0..=ROW_ROOT {
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
        let mut pow2 = vec![zero; TRACE_LENGTH];
        for level in 0..CUSTODIAN_DEPTH {
            let row = level * CYCLE_LENGTH + 7;
            tree_link[row] = one;
            pow2[row] = BaseElement::new(1u64 << level);
        }
        columns.push(tree_link);
        columns.push(pow2);

        let mut first_row = vec![zero; TRACE_LENGTH];
        first_row[0] = one;
        columns.push(first_row);

        let mut sel_root = vec![zero; TRACE_LENGTH];
        sel_root[ROW_ROOT] = one;
        columns.push(sel_root);

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
        let pow2 = periodic[P_POW2];
        let first_row = periodic[P_FIRST_ROW];
        let sel_root = periodic[P_SEL_ROOT];

        // La ronda de Rescue, un solo carril.
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

        // ===== EL ACUMULADOR ATA EL ÍNDICE AL CAMINO =====
        // Sin esto el índice público sería un número declarado sin relación
        // con la posición demostrada, y la comprobación de «custodios
        // distintos» que hace la capa no valdría nada.
        result[C_ACC] = tree_link * (next[COL_ACC] - (current[COL_ACC] + bit * pow2));

        // El acumulado final es el índice declarado.
        result[C_ACC_FINAL] = sel_root * (current[COL_ACC] - current[COL_IDX]);

        let transport = [COL_KEY, COL_IDX];
        for (k, col) in transport.iter().enumerate() {
            result[C_TRANSPORT + k] = next[*col] - current[*col];
        }
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        let zero = BaseElement::ZERO;
        let mut a = Vec::with_capacity(14);

        // Fila 0: capacidad, dominio anclado y relleno.
        for i in 0..4 {
            a.push(Assertion::single(i, 0, zero));
        }
        a.push(Assertion::single(4, 0, BaseElement::new(CUSTODIAN_DOMAIN)));
        for i in 5..8 {
            a.push(Assertion::single(i, 0, zero));
        }
        // Acumulador a cero.
        a.push(Assertion::single(COL_ACC, 0, zero));
        // El índice declarado es el público.
        a.push(Assertion::single(COL_IDX, 0, self.pub_inputs.custodian_index));
        // El carril llega a la raíz del conjunto autorizado.
        for i in 0..4 {
            a.push(Assertion::single(
                4 + i,
                ROW_ROOT,
                self.pub_inputs.custodian_set_root[i],
            ));
        }

        a
    }
}

pub struct SingleThresholdProver {
    options: ProofOptions,
}

impl SingleThresholdProver {
    pub fn new(options: ProofOptions) -> Self {
        Self { options }
    }
}

impl Prover for SingleThresholdProver {
    type BaseField = BaseElement;
    type Air = SingleThresholdAir;
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

    fn get_pub_inputs(&self, trace: &Self::Trace) -> SingleThresholdPublicInputs {
        SingleThresholdPublicInputs {
            custodian_set_root: [
                trace.get(4, ROW_ROOT),
                trace.get(5, ROW_ROOT),
                trace.get(6, ROW_ROOT),
                trace.get(7, ROW_ROOT),
            ],
            custodian_index: trace.get(COL_IDX, 0),
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
        index: u64,
        path: &CustodianPath,
        declared: SingleThresholdPublicInputs,
    ) -> bool {
        let trace = build_trace(key, index, path);
        let prover = SingleThresholdProver::new(default_options());
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| prover.prove(trace)));
        match r {
            Err(_) => false,
            Ok(Err(_)) => false,
            Ok(Ok(proof)) => {
                let min_opts = AcceptableOptions::OptionSet(vec![default_options()]);
                verify::<
                    SingleThresholdAir,
                    Blake3,
                    DefaultRandomCoin<Blake3>,
                    MerkleTree<Blake3>,
                >(proof, declared, &min_opts)
                .is_ok()
            }
        }
    }

    /// El caso honesto: un custodio del conjunto prueba su autorización.
    #[test]
    fn a_single_custodian_authorization_verifies() {
        let keys = custodian_keys();
        let (root, paths) = build_custodian_set(&keys);
        let declared = SingleThresholdPublicInputs {
            custodian_set_root: root,
            custodian_index: BaseElement::new(2),
        };
        assert!(prove_and_verify(keys[2], 2, &paths[2], declared));
    }

    /// Quien no está en el conjunto no puede autorizar: su hoja no sube a
    /// la raíz publicada.
    #[test]
    fn a_key_outside_the_set_cannot_authorize() {
        let keys = custodian_keys();
        let (root, paths) = build_custodian_set(&keys);
        let intruso = BaseElement::new(0xBADC0DE);
        let declared = SingleThresholdPublicInputs {
            custodian_set_root: root,
            custodian_index: BaseElement::new(2),
        };
        assert!(!prove_and_verify(intruso, 2, &paths[2], declared));
    }

    /// Distingue **por que** se rechaza: si `prove` falla, la traza viola
    /// una restriccion; si `verify` falla, la prueba no case con lo
    /// declarado. Disciplina §16.5: un test negativo que pasa por el motivo
    /// equivocado no prueba nada.
    #[derive(Debug, PartialEq)]
    enum Rechazo {
        Acepta,
        FallaAlProbar,
        FallaAlVerificar,
    }

    fn por_que_rechaza(
        key: BaseElement,
        index: u64,
        path: &CustodianPath,
        declared: SingleThresholdPublicInputs,
    ) -> Rechazo {
        let trace = build_trace(key, index, path);
        let prover = SingleThresholdProver::new(default_options());
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| prover.prove(trace)));
        match r {
            Err(_) | Ok(Err(_)) => Rechazo::FallaAlProbar,
            Ok(Ok(proof)) => {
                let min_opts = AcceptableOptions::OptionSet(vec![default_options()]);
                let v = verify::<
                    SingleThresholdAir,
                    Blake3,
                    DefaultRandomCoin<Blake3>,
                    MerkleTree<Blake3>,
                >(proof, declared, &min_opts);
                if v.is_ok() {
                    Rechazo::Acepta
                } else {
                    Rechazo::FallaAlVerificar
                }
            }
        }
    }

    /// §16.5 sobre el test del indice: la traza viola `C_ACC_FINAL` —el
    /// acumulado del camino no es el indice declarado— y por eso se rechaza.
    ///
    /// ⚠️ **La etapa es la verificacion, no la generacion.** Este test
    /// esperaba `FallaAlProbar` y fallo: en **release** winterfell no
    /// comprueba las restricciones al generar la prueba (esa es la
    /// comprobacion de depuracion de §20), asi que el probador emite
    /// alegremente una prueba de una traza invalida y es el **verificador**
    /// quien la tumba. El motivo del rechazo era el correcto; la etapa que
    /// predije, no. Queda escrito porque es la clase de suposicion que hay
    /// que comprobar, no dar por hecha.
    #[test]
    fn the_index_test_rejects_for_the_right_reason() {
        let keys = custodian_keys();
        let (root, paths) = build_custodian_set(&keys);
        let declared = SingleThresholdPublicInputs {
            custodian_set_root: root,
            custodian_index: BaseElement::new(3),
        };
        // ⚠️ **Quien lo caza depende del modo, y los dos estan bien.**
        //
        // En release el probador no valida restricciones, asi que la traza
        // se genera y la violacion de `C_ACC_FINAL` la caza el
        // **verificador**.
        //
        // En depuracion winterfell SI las valida al generar, y la caza el
        // **probador** -antes, y diciendo *"main transition constraint 23
        // did not evaluate to ZERO at step 39"*, que es mas preciso-.
        //
        // Este test afirmaba `FallaAlVerificar` en los dos modos y por eso
        // fallaba en depuracion **desde antes de la entrada 44**. Lo que no
        // puede ser en ningun modo es `Acepta`, y eso se sigue exigiendo.
        let esperado = if cfg!(debug_assertions) {
            Rechazo::FallaAlProbar
        } else {
            Rechazo::FallaAlVerificar
        };
        assert_eq!(
            por_que_rechaza(keys[2], 3, &paths[2], declared),
            esperado,
            "la violacion de C_ACC_FINAL debe cazarla el probador en \
             depuracion y el verificador en release, nunca pasar"
        );
    }

    /// §16.5 sobre el intruso: su hoja sube a OTRA raiz, asi que la prueba
    /// se genera bien y es la **verificacion** contra la raiz publicada la
    /// que la rechaza.
    #[test]
    fn the_outsider_test_rejects_for_the_right_reason() {
        let keys = custodian_keys();
        let (root, paths) = build_custodian_set(&keys);
        let declared = SingleThresholdPublicInputs {
            custodian_set_root: root,
            custodian_index: BaseElement::new(2),
        };
        assert_eq!(
            por_que_rechaza(BaseElement::new(0xBADC0DE), 2, &paths[2], declared),
            Rechazo::FallaAlVerificar,
            "la traza es coherente consigo misma; falla al contrastar la raiz"
        );
    }

    /// ⚠️ **LA MEDICION de §47.5.** Compara las dos variantes de carril unico
    /// con el circuito conjunto de dos carriles. Criterio: si dos pruebas
    /// single cuestan aproximadamente lo mismo que una conjunta, la via B es
    /// viable.
    ///
    /// `cargo test -p stark-experiment --release metrics_33 -- --nocapture`
    #[test]
    fn metrics_33() {
        use crate::circuit_threshold as conjunto;
        use crate::circuit_threshold_single_nullifier as nulif;
        use std::time::Instant;

        let keys = custodian_keys();
        let (_root, paths) = build_custodian_set(&keys);

        // --- Variante A: indice publico ---
        let t0 = Instant::now();
        let trace = build_trace(keys[2], 2, &paths[2]);
        let proof_a = SingleThresholdProver::new(default_options())
            .prove(trace)
            .expect("A deberia probar");
        let ms_a = t0.elapsed().as_secs_f64() * 1000.0;
        let size_a = proof_a.to_bytes().len();

        // --- Variante B: nulificador ---
        let t0 = Instant::now();
        // §55: la variante B ata la operacion al nulificador, asi que su
        // traza necesita el compromiso de la operacion. Se mide con uno
        // cualquiera: el coste no depende de su valor.
        let op_medida = [BaseElement::new(0xDEAD_0001), BaseElement::ZERO,
                         BaseElement::ZERO, BaseElement::ZERO];
        let trace = nulif::build_trace(
            BaseElement::new(crate::circuit_threshold::CUSTODIAN_DOMAIN),
            keys[2],
            &paths[2],
            op_medida,
        );
        let proof_b = nulif::NullifierThresholdProver::new(default_options())
            .prove(trace)
            .expect("B deberia probar");
        let ms_b = t0.elapsed().as_secs_f64() * 1000.0;
        let size_b = proof_b.to_bytes().len();

        // --- Conjunto: dos carriles en una traza ---
        let t0 = Instant::now();
        let trace = conjunto::build_trace(keys[1], 1, &paths[1], keys[2], 2, &paths[2]);
        let proof_c = conjunto::ThresholdProver::new(default_options())
            .prove(trace)
            .expect("conjunto deberia probar");
        let ms_c = t0.elapsed().as_secs_f64() * 1000.0;
        let size_c = proof_c.to_bytes().len();

        println!();
        println!("=== MEDICION ENTRADA 33 (§47.5) ===");
        println!(
            "{:<28} {:>6} {:>7} {:>8} {:>10} {:>10}",
            "circuito", "cols", "filas", "restr.", "ms", "bytes"
        );
        println!(
            "{:<28} {:>6} {:>7} {:>8} {:>10.1} {:>10}",
            "A single (indice publico)", TRACE_WIDTH, TRACE_LENGTH, NUM_CONSTRAINTS, ms_a, size_a
        );
        println!(
            "{:<28} {:>6} {:>7} {:>8} {:>10.1} {:>10}",
            "B single (nulificador)",
            nulif::TRACE_WIDTH,
            nulif::TRACE_LENGTH,
            nulif::NUM_CONSTRAINTS,
            ms_b,
            size_b
        );
        // ⚠️ Las tres cifras de restricciones se LEEN del codigo. Puestas a
        // mano fallaron dos veces: «29» donde eran 60 (§52.6) y «35» donde
        // eran 39 tras atar la operacion (§55). Una tabla de medicion con
        // numeros escritos a mano es una cifra sin fuente esperando su turno.
        println!(
            "{:<28} {:>6} {:>7} {:>8} {:>10.1} {:>10}",
            "conjunto (dos carriles)",
            34,
            64,
            conjunto::NUM_CONSTRAINTS,
            ms_c,
            size_c
        );
        println!();
        println!("DOS pruebas A: {:.1} ms, {} bytes", 2.0 * ms_a, 2 * size_a);
        println!("DOS pruebas B: {:.1} ms, {} bytes", 2.0 * ms_b, 2 * size_b);
        println!("UNA conjunta:  {:.1} ms, {} bytes", ms_c, size_c);
        println!();
        println!("Criterio §51.4: si dos single ~ una conjunta, la via B es viable.");

        // Que las pruebas se generan de verdad, no en cero.
        assert!(size_a > 1000, "prueba A sospechosamente pequena");
        assert!(size_b > 1000, "prueba B sospechosamente pequena");
    }

    /// ⚠️ **La restricción que sostiene toda la vía B.** El índice público
    /// debe ser la posición REALMENTE demostrada. Si un custodio pudiera
    /// declarar un índice distinto del suyo, dos pruebas del MISMO custodio
    /// pasarían por dos distintos y el umbral 2-de-N se caería a 1-de-N.
    #[test]
    fn a_custodian_cannot_claim_an_index_that_is_not_theirs() {
        let keys = custodian_keys();
        let (root, paths) = build_custodian_set(&keys);
        // El custodio 2 intenta pasar por el 3.
        let declared = SingleThresholdPublicInputs {
            custodian_set_root: root,
            custodian_index: BaseElement::new(3),
        };
        assert!(
            !prove_and_verify(keys[2], 3, &paths[2], declared),
            "SOLIDEZ: el indice publico debe ser la posicion demostrada; si no, \
             dos pruebas del mismo custodio superan el umbral (entrada 33)"
        );
    }

    /// Ninguna restriccion es vacua. Ver la nota en `circuit_frozen_climb`:
    /// esta herramienta y el barrido de disposiciones cubren defectos
    /// distintos.
    #[test]
    fn no_constraint_is_vacuous() {
        use crate::mutation::{buscar_vacias, rows_of};

        let keys = custodian_keys();
        let (root, paths) = build_custodian_set(&keys);
        let trace = build_trace(keys[2], 2, &paths[2]);
        let rows = rows_of(&trace, TRACE_WIDTH, TRACE_LENGTH);

        let air = SingleThresholdAir::new(
            TraceInfo::new(TRACE_WIDTH, TRACE_LENGTH),
            SingleThresholdPublicInputs {
                custodian_set_root: root,
                custodian_index: BaseElement::new(2),
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
}
