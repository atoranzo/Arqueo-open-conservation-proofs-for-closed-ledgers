//! Range check en Winterfell/AIR sobre el campo **Goldilocks (f64)** —
//! migrado desde `f128` para ser compatible con `rescue_hash.rs`, ya que
//! `Rp64_256` (el hash Rescue de `winter-crypto`) está definido sobre este
//! campo y el circuito de cumplimiento integrado necesita que todas las
//! piezas compartan campo.
//!
//! ## ⚠️ Hallazgo real de la migración: el rango efectivo es de 63 bits
//!
//! El campo Goldilocks es **p = 2^64 - 2^32 + 1**, que es MENOR que 2^64.
//! Consecuencia: `u64::MAX = 2^64 - 1` NO cabe en el campo (se reduce
//! módulo p y da 2^32 - 2). Eso rompería la solidez de un range check de
//! 64 bits completos: dos patrones de bits distintos podrían dar el mismo
//! elemento de campo, y la reconstrucción dejaría de probar unívocamente
//! que el valor está en [0, 2^64).
//!
//! En `f128` esto no ocurría porque el campo era mucho mayor que el rango.
//! Es un cambio de solidez REAL introducido por la migración, no un
//! detalle de implementación.
//!
//! **Solución aplicada**: forzar el bit 63 a cero mediante una aserción,
//! de modo que el rango demostrado es [0, 2^63), cómodamente por debajo
//! de p. La traza conserva 64 filas (requisito de potencia de dos de
//! Winterfell). Para el caso de uso no hay pérdida práctica: 2^63 es
//! aproximadamente 9,2 x 10^18 unidades mínimas, un techo muy por encima
//! de cualquier importe monetario real.
//!
//! ## Diseño de la traza (3 columnas x 64 filas)
//! - `bit`: el bit en la fila i (0 o 1)
//! - `power`: 2^i (empieza en 1, se duplica cada fila)
//! - `acc`: suma acumulada de `bit_k * 2^k` para k=0..=i
//!
//! ## Nota sobre trazas degeneradas (heredada de la versión f128)
//!
//! Con `value = 0` la traza es degenerada (polinomio de restricción
//! idénticamente nulo) y una comprobación de depuración de Winterfell da
//! un falso positivo. Los tests de este módulo deben ejecutarse en
//! release: `cargo test -p stark-experiment range_check --release`.
//!
//! ## Limitación pendiente para el circuito integrado
//!
//! Anclar `bit_0` con una aserción sobre un valor público solo funciona
//! porque aquí `value` ES público. Cuando esto se integre en el circuito
//! de cumplimiento (donde `balance` y `amount` son PRIVADOS), habrá que
//! usar una columna periódica como selector de "primera fila" para
//! expresar `acc_0 = bit_0 * 1` como restricción sin revelar el bit.
//! Pendiente real, no resuelto.

use winterfell::crypto::{hashers::Blake3_256, DefaultRandomCoin, MerkleTree};
use winterfell::math::{fields::f64::BaseElement, FieldElement, ToElements};
use winterfell::matrix::ColMatrix;
use winterfell::{
    Air, AirContext, Assertion, AuxRandElements, CompositionPoly, CompositionPolyTrace,
    ConstraintCompositionCoefficients, DefaultConstraintCommitment, DefaultConstraintEvaluator,
    DefaultTraceLde, EvaluationFrame, PartitionOptions, ProofOptions, Prover, StarkDomain, Trace,
    TraceInfo, TracePolyTable, TraceTable, TransitionConstraintDegree,
};

/// Filas de la traza. Debe ser potencia de dos (requisito de Winterfell).
pub const TRACE_ROWS: usize = 64;
/// Bits efectivamente demostrados: 63, no 64. Ver la advertencia sobre el
/// tamaño del campo Goldilocks en la cabecera de este módulo.
pub const EFFECTIVE_BITS: usize = 63;
/// Valor máximo que este range check puede demostrar: 2^63 - 1.
pub const MAX_VALUE: u64 = (1u64 << EFFECTIVE_BITS) - 1;

const TRACE_WIDTH: usize = 3; // bit, power, acc

type Blake3 = Blake3_256<BaseElement>;

/// Descompone `value` en bits little-endian, ocupando las 64 filas de la
/// traza. El bit 63 siempre es cero (garantizado por `MAX_VALUE`).
fn value_to_bits_le(value: u64) -> Vec<bool> {
    assert!(
        value <= MAX_VALUE,
        "este range check solo cubre valores hasta 2^63 - 1 sobre el campo Goldilocks"
    );
    (0..TRACE_ROWS).map(|i| (value >> i) & 1 == 1).collect()
}

/// Genera la traza de ejecución del range check para `value`.
pub fn build_trace(value: u64) -> TraceTable<BaseElement> {
    let bits = value_to_bits_le(value);
    let mut trace = TraceTable::new(TRACE_WIDTH, TRACE_ROWS);

    trace.fill(
        |state| {
            let bit0 = if bits[0] {
                BaseElement::ONE
            } else {
                BaseElement::ZERO
            };
            state[0] = bit0;
            state[1] = BaseElement::ONE; // power_0 = 2^0 = 1
            state[2] = bit0; // acc_0 = bit_0 * 1
        },
        |step, state| {
            let bit_next = if bits[step + 1] {
                BaseElement::ONE
            } else {
                BaseElement::ZERO
            };
            let power_next = state[1] + state[1];
            let acc_next = state[2] + bit_next * power_next;

            state[0] = bit_next;
            state[1] = power_next;
            state[2] = acc_next;
        },
    );

    trace
}

#[derive(Clone, Debug)]
pub struct RangeCheckPublicInputs {
    pub value: BaseElement,
    pub first_bit: BaseElement,
}

impl ToElements<BaseElement> for RangeCheckPublicInputs {
    fn to_elements(&self) -> Vec<BaseElement> {
        vec![self.value, self.first_bit]
    }
}

pub struct RangeCheckAir {
    context: AirContext<BaseElement>,
    value: BaseElement,
    first_bit: BaseElement,
}

impl Air for RangeCheckAir {
    type BaseField = BaseElement;
    type PublicInputs = RangeCheckPublicInputs;

    fn new(trace_info: TraceInfo, pub_inputs: Self::PublicInputs, options: ProofOptions) -> Self {
        assert_eq!(TRACE_WIDTH, trace_info.width());
        let degrees = vec![
            TransitionConstraintDegree::new(2), // bit booleano
            TransitionConstraintDegree::new(1), // power se duplica
            TransitionConstraintDegree::new(2), // acc acumula
        ];
        RangeCheckAir {
            // 5 aserciones (ver get_assertions). Este numero DEBE coincidir
            // con la longitud del vector devuelto por `get_assertions`.
            context: AirContext::new(trace_info, degrees, 5, options),
            value: pub_inputs.value,
            first_bit: pub_inputs.first_bit,
        }
    }

    fn context(&self) -> &AirContext<Self::BaseField> {
        &self.context
    }

    fn evaluate_transition<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        frame: &EvaluationFrame<E>,
        _periodic_values: &[E],
        result: &mut [E],
    ) {
        let current = frame.current();
        let next = frame.next();

        let bit_next = next[0];
        let power_cur = current[1];
        let power_next = next[1];
        let acc_cur = current[2];
        let acc_next = next[2];

        result[0] = bit_next * (bit_next - E::ONE);
        result[1] = power_next - (power_cur + power_cur);
        result[2] = acc_next - (acc_cur + bit_next * power_next);
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        let last_step = TRACE_ROWS - 1;
        vec![
            // Anclar la fila 0 por completo (cierra los huecos de solidez
            // documentados en la version f128):
            Assertion::single(0, 0, self.first_bit),
            Assertion::single(1, 0, BaseElement::ONE),
            Assertion::single(2, 0, self.first_bit),
            // El bit mas significativo DEBE ser cero: es lo que limita el
            // rango a [0, 2^63) y mantiene la solidez sobre Goldilocks.
            Assertion::single(0, last_step, BaseElement::ZERO),
            // Y el resultado acumulado final:
            Assertion::single(2, last_step, self.value),
        ]
    }
}

pub struct RangeCheckProver {
    options: ProofOptions,
}

impl RangeCheckProver {
    pub fn new(options: ProofOptions) -> Self {
        Self { options }
    }
}

impl Prover for RangeCheckProver {
    type BaseField = BaseElement;
    type Air = RangeCheckAir;
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

    fn get_pub_inputs(&self, trace: &Self::Trace) -> RangeCheckPublicInputs {
        let last_step = trace.length() - 1;
        RangeCheckPublicInputs {
            value: trace.get(2, last_step),
            first_bit: trace.get(0, 0),
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

    fn public_inputs_for(value: u64) -> RangeCheckPublicInputs {
        RangeCheckPublicInputs {
            value: BaseElement::new(value),
            first_bit: if value & 1 == 1 {
                BaseElement::ONE
            } else {
                BaseElement::ZERO
            },
        }
    }

    #[test]
    fn value_within_range_produces_verifiable_proof() {
        let value: u64 = 123_456_789;
        let trace = build_trace(value);

        let prover = RangeCheckProver::new(default_options());
        let proof: Proof = prover
            .prove(trace)
            .expect("la generacion de la prueba no deberia fallar");

        let min_opts = AcceptableOptions::OptionSet(vec![prover.options().clone()]);
        let verification =
            verify::<RangeCheckAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
                proof,
                public_inputs_for(value),
                &min_opts,
            );

        assert!(
            verification.is_ok(),
            "un valor dentro de rango deberia producir una prueba verificable: {verification:?}"
        );
    }

    /// Valores límite, ajustados al rango real de 63 bits sobre Goldilocks
    /// (ya no se puede usar `u64::MAX`, que no cabe en el campo).
    ///
    /// NOTA: `value = 0` NO está en esta lista — ver
    /// `zero_value_only_works_in_release_mode` justo debajo.
    #[test]
    fn boundary_values_produce_verifiable_proofs() {
        for value in [1u64, MAX_VALUE / 2, (1u64 << 62), MAX_VALUE] {
            let trace = build_trace(value);
            let prover = RangeCheckProver::new(default_options());
            let proof = prover
                .prove(trace)
                .expect("la generacion de la prueba no deberia fallar");

            let min_opts = AcceptableOptions::OptionSet(vec![prover.options().clone()]);
            let verification =
                verify::<RangeCheckAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
                    proof,
                    public_inputs_for(value),
                    &min_opts,
                );

            assert!(
                verification.is_ok(),
                "el valor {value} deberia producir una prueba valida"
            );
        }
    }

    /// El caso `value = 0` produce una traza DEGENERADA (todos los bits a
    /// cero → polinomio de restricción idénticamente nulo, grado 0 en vez
    /// de 63). La prueba en sí es correcta y verifica bien, pero una
    /// comprobación de depuración de winterfell
    /// (`evaluation_table.rs:214`) da un falso positivo en debug:
    /// "transition constraint degrees didn't match: expected [63,0,63],
    /// actual [0,0,0]".
    ///
    /// ⚠️ **Estaba marcado `#[ignore]` sin condición**, así que no se
    /// ejecutaba **tampoco en release**, donde funciona: había que acordarse
    /// de lanzarlo a mano con `-- --ignored`. Un test que depende de que
    /// alguien recuerde ejecutarlo no protege nada.
    ///
    /// Ahora se salta **solo en depuración**, y `--release` lo corre con el
    /// resto.
    ///
    /// **Por qué importa este caso y no se descarta sin más**: el circuito
    /// de cumplimiento necesita `amount == balance` (que produce
    /// `diff = 0`), verificado explícitamente en los backends Groth16 y
    /// Halo2. Es un caso legítimo del dominio, no una curiosidad.
    #[test]
    #[cfg_attr(
        debug_assertions,
        ignore = "traza degenerada: grado 0 en depuracion, correcto en release"
    )]
    fn zero_value_only_works_in_release_mode() {
        let value = 0u64;
        let trace = build_trace(value);
        let prover = RangeCheckProver::new(default_options());
        let proof = prover
            .prove(trace)
            .expect("la generacion de la prueba no deberia fallar");

        let min_opts = AcceptableOptions::OptionSet(vec![prover.options().clone()]);
        let verification =
            verify::<RangeCheckAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
                proof,
                public_inputs_for(value),
                &min_opts,
            );

        assert!(
            verification.is_ok(),
            "el valor cero deberia producir una prueba valida en release"
        );
    }

    /// TEST DE SOLIDEZ: declarar un valor público distinto del realmente
    /// descompuesto debe hacer fallar la verificación.
    #[test]
    fn wrong_declared_value_fails_verification() {
        let real_value: u64 = 123_456_789;
        let trace = build_trace(real_value);

        let prover = RangeCheckProver::new(default_options());
        let proof = prover.prove(trace).expect("la generacion no deberia fallar");

        let wrong_value = real_value + 2; // mismo bit 0, distinto acumulado
        let min_opts = AcceptableOptions::OptionSet(vec![prover.options().clone()]);
        let verification =
            verify::<RangeCheckAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
                proof,
                public_inputs_for(wrong_value),
                &min_opts,
            );

        assert!(
            verification.is_err(),
            "CRITICO: declarar un valor publico distinto al real deberia fallar"
        );
    }

    /// TEST DISCRIMINANTE (el equivalente al de `rescue_hash`): corromper
    /// una fila INTERMEDIA de la traza debe detectarse. Sin este test, no
    /// sabríamos distinguir restricciones reales de restricciones vacuas.
    #[test]
    fn corrupted_intermediate_row_is_detected() {
        let value: u64 = 123_456_789;
        let mut trace = build_trace(value);

        // Corromper la columna `power` en una fila intermedia.
        let original = trace.get(1, 10);
        trace.set(1, 10, original + BaseElement::ONE);

        let prover = RangeCheckProver::new(default_options());

        // Tres formas legitimas de deteccion; ver la nota en `merkle.rs`.
        let prove_result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| prover.prove(trace)));

        match prove_result {
            Err(_) => { /* panic: detectado en modo debug */ }
            Ok(Err(_)) => { /* Err: detectado */ }
            Ok(Ok(proof)) => {
                let min_opts = AcceptableOptions::OptionSet(vec![prover.options().clone()]);
                let verification = verify::<
                    RangeCheckAir,
                    Blake3,
                    DefaultRandomCoin<Blake3>,
                    MerkleTree<Blake3>,
                >(proof, public_inputs_for(value), &min_opts);
                assert!(
                    verification.is_err(),
                    "CRITICO: una traza con una fila intermedia corrompida no deberia \
                     producir una prueba verificable"
                );
            }
        }
    }
}
