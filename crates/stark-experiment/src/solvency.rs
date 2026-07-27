//! Etapa 1 del circuito de cumplimiento en AIR: **solvencia con valores
//! privados** — demuestra `amount <= balance` y `amount <= regulatory_limit`
//! sin revelar `balance` ni `amount` (solo el límite es público).
//!
//! ## Hallazgo real que dio forma a este diseño (segunda versión)
//!
//! La primera versión usaba tres columnas de range check (`bit`, `power`,
//! `acc`) como el módulo `range_check`. El diagnóstico en debug de
//! winterfell reveló un desajuste de grados exacto de 3 en las cuatro
//! restricciones que tocaban `power` (249 vs 252, 504 vs 507): la columna
//! `power` es ESTRUCTURALMENTE PERIÓDICA (repite 1,2,4,...,2^63 en cada
//! segmento), así que su polinomio tiene grado 252 y no el 255 genérico
//! que winterfell asume al computar los grados esperados — y la API de
//! `TransitionConstraintDegree` no permite declarar ese caso.
//!
//! **Solución**: eliminar `power` por completo con acumulación de Horner
//! en big-endian: `acc_next = 2*acc_cur + bit_next`. Mismo resultado
//! (reconstruir el valor bit a bit), ninguna columna auxiliar periódica,
//! y el bit más significativo pasa a ser el PRIMERO de cada segmento, con
//! lo que forzarlo a cero es trivial (`first * bit = 0`). Todas las
//! columnas quedan de grado genérico y la contabilidad de grados cuadra.
//!
//! ## El problema estructural que esta pieza resuelve
//!
//! En Halo2 conectábamos el range check con la resta de solvencia
//! mediante `constrain_equal` entre celdas arbitrarias. **En AIR eso no
//! existe**: solo se relacionan filas adyacentes. La solución son
//! COLUMNAS DE TRANSPORTE constantes (`next - cur = 0`) que llevan
//! balance, amount y límite a todas las filas.
//!
//! ## Diseño de la traza (5 columnas × 256 filas)
//!
//! Columnas: `bit`, `acc` + `c_bal`, `c_amt`, `c_lim` (transporte).
//!
//! 4 segmentos de 64 filas (bits en BIG-ENDIAN, MSB primero):
//! - Segmento 0 (filas 0..63):    `balance`
//! - Segmento 1 (filas 64..127):  `amount`
//! - Segmento 2 (filas 128..191): `diff_balance = balance - amount`
//! - Segmento 3 (filas 192..255): `diff_limit = limit - amount`
//!
//! Que las dos diferencias quepan en 63 bits es EXACTAMENTE lo que
//! demuestra `amount <= balance` y `amount <= limit`: si amount fuera
//! mayor, la resta en el campo daría un valor enorme que no cabe.
//!
//! ## Los "links" (atar cada acumulador a la aritmética)
//!
//! Columnas periódicas de longitud 256 con un único 1 en la posición 62
//! de cada segmento, activando `next.acc = c_bal`, `= c_amt`,
//! `= c_bal - c_amt`, `= c_lim - c_amt` en las transiciones 62→63,
//! 126→127, 190→191, 254→255 — donde `next.acc` es el valor completo.
//! (No en la posición 63 con `cur`: la última transición de la traza está
//! exenta y el cuarto segmento quedaría sin atar.)
//!
//! ## El anclaje privado de fila 0, resuelto como se prometió
//!
//! En `range_check.rs` anclamos la fila 0 con aserciones sobre un valor
//! PÚBLICO y documentamos que no serviría con privados. Aquí: selector
//! periódico `first` (ciclo 64, 1 en posición 0) activa `acc = 0` y
//! `bit = 0` (el MSB) sobre la fila inicial de cada segmento — sin
//! revelar nada.

use winterfell::crypto::{hashers::Blake3_256, DefaultRandomCoin, MerkleTree};
use winterfell::math::{fields::f64::BaseElement, FieldElement, ToElements};
use winterfell::matrix::ColMatrix;
use winterfell::{
    Air, AirContext, Assertion, AuxRandElements, CompositionPoly, CompositionPolyTrace,
    ConstraintCompositionCoefficients, DefaultConstraintCommitment, DefaultConstraintEvaluator,
    DefaultTraceLde, EvaluationFrame, PartitionOptions, ProofOptions, Prover, StarkDomain,
    TraceInfo, TracePolyTable, TraceTable, TransitionConstraintDegree,
};

/// Filas por segmento de range check.
pub const SEGMENT_LENGTH: usize = 64;
/// Segmentos: balance, amount, diff_balance, diff_limit.
pub const NUM_SEGMENTS: usize = 4;
/// Longitud total: 256 filas (potencia de dos ✓).
pub const TRACE_LENGTH: usize = NUM_SEGMENTS * SEGMENT_LENGTH;
/// Bits efectivos por valor: 63 (límite del campo Goldilocks, ver
/// `range_check.rs`).
pub const EFFECTIVE_BITS: usize = 63;
/// Valor máximo demostrable: 2^63 - 1.
pub const MAX_VALUE: u64 = (1u64 << EFFECTIVE_BITS) - 1;

const TRACE_WIDTH: usize = 5; // bit, acc, c_bal, c_amt, c_lim
const COL_BIT: usize = 0;
const COL_ACC: usize = 1;
const COL_BAL: usize = 2;
const COL_AMT: usize = 3;
const COL_LIM: usize = 4;

type Blake3 = Blake3_256<BaseElement>;

/// Bits en BIG-ENDIAN (MSB primero): el bit en la posición p del segmento
/// es el bit (63 - p) del valor. La posición 0 es el bit 63, que siempre
/// es cero para valores <= MAX_VALUE.
fn value_to_bits_be(value: u64) -> Vec<bool> {
    (0..SEGMENT_LENGTH)
        .map(|p| (value >> (SEGMENT_LENGTH - 1 - p)) & 1 == 1)
        .collect()
}

/// Construye la traza. NO valida las entradas a propósito: la solidez del
/// circuito debe rechazar los casos inválidos (amount > balance, etc.)
/// por sí misma — y los tests lo comprueban exactamente así.
pub fn build_trace(balance: u64, amount: u64, limit: u64) -> TraceTable<BaseElement> {
    let c_bal = BaseElement::new(balance);
    let c_amt = BaseElement::new(amount);
    let c_lim = BaseElement::new(limit);

    // Las diferencias se calculan EN EL CAMPO (con wrap si amount es
    // mayor) y se descomponen en bits desde su representación entera. Si
    // no caben en 63 bits, la reconstrucción no cuadra y los "links"
    // fallan — precisamente el mecanismo de solidez.
    let diff_bal = (c_bal - c_amt).as_int();
    let diff_lim = (c_lim - c_amt).as_int();

    let segment_values = [balance, amount, diff_bal, diff_lim];
    let segment_bits: Vec<Vec<bool>> =
        segment_values.iter().map(|v| value_to_bits_be(*v)).collect();

    let mut trace = TraceTable::new(TRACE_WIDTH, TRACE_LENGTH);

    trace.fill(
        |state| {
            // Fila 0: MSB del balance (siempre 0 para valores validos),
            // acumulador de Horner arrancando en el bit.
            let bit0 = if segment_bits[0][0] {
                BaseElement::ONE
            } else {
                BaseElement::ZERO
            };
            state[COL_BIT] = bit0;
            state[COL_ACC] = bit0;
            state[COL_BAL] = c_bal;
            state[COL_AMT] = c_amt;
            state[COL_LIM] = c_lim;
        },
        |step, state| {
            let row = step + 1;
            let segment = row / SEGMENT_LENGTH;
            let pos = row % SEGMENT_LENGTH;

            let bit = if segment_bits[segment][pos] {
                BaseElement::ONE
            } else {
                BaseElement::ZERO
            };

            if pos == 0 {
                // Primera fila de un segmento nuevo: reiniciar Horner.
                state[COL_BIT] = bit;
                state[COL_ACC] = bit;
            } else {
                // Horner big-endian: acc = 2*acc + bit.
                state[COL_ACC] = state[COL_ACC] + state[COL_ACC] + bit;
                state[COL_BIT] = bit;
            }
            // Las columnas de transporte no cambian.
        },
    );

    trace
}

/// Inputs públicos: SOLO el límite regulatorio. Balance y amount son
/// testigos privados.
#[derive(Clone, Debug)]
pub struct SolvencyPublicInputs {
    pub regulatory_limit: BaseElement,
}

impl ToElements<BaseElement> for SolvencyPublicInputs {
    fn to_elements(&self) -> Vec<BaseElement> {
        vec![self.regulatory_limit]
    }
}

pub struct SolvencyAir {
    context: AirContext<BaseElement>,
    regulatory_limit: BaseElement,
}

impl Air for SolvencyAir {
    type BaseField = BaseElement;
    type PublicInputs = SolvencyPublicInputs;

    fn new(trace_info: TraceInfo, pub_inputs: Self::PublicInputs, options: ProofOptions) -> Self {
        assert_eq!(TRACE_WIDTH, trace_info.width());

        let degrees = vec![
            // 0: cur.bit booleano (filas 0..254)
            TransitionConstraintDegree::new(2),
            // 1: next.bit booleano (filas 1..255)
            TransitionConstraintDegree::new(2),
            // 2..4: constancia de las columnas de transporte
            TransitionConstraintDegree::new(1),
            TransitionConstraintDegree::new(1),
            TransitionConstraintDegree::new(1),
            // 5: first * bit  (el MSB de cada segmento es cero)
            TransitionConstraintDegree::with_cycles(1, vec![SEGMENT_LENGTH]),
            // 6: first * acc  (Horner arranca en el MSB, que es cero)
            TransitionConstraintDegree::with_cycles(1, vec![SEGMENT_LENGTH]),
            // 7: cont * (acc_next - 2*acc_cur - bit_next)
            TransitionConstraintDegree::with_cycles(1, vec![SEGMENT_LENGTH]),
            // 8..11: links de cada segmento (ciclo = traza completa)
            TransitionConstraintDegree::with_cycles(1, vec![TRACE_LENGTH]),
            TransitionConstraintDegree::with_cycles(1, vec![TRACE_LENGTH]),
            TransitionConstraintDegree::with_cycles(1, vec![TRACE_LENGTH]),
            TransitionConstraintDegree::with_cycles(1, vec![TRACE_LENGTH]),
        ];

        SolvencyAir {
            // 1 asercion: el limite publico (ver get_assertions).
            context: AirContext::new(trace_info, degrees, 1, options),
            regulatory_limit: pub_inputs.regulatory_limit,
        }
    }

    fn context(&self) -> &AirContext<Self::BaseField> {
        &self.context
    }

    /// 7 columnas periódicas:
    /// - `first` (ciclo 64): 1 en la posición 0 — ancla la fila inicial
    ///   de cada segmento (acc = 0, bit = 0 al ser el MSB).
    /// - `cont` (ciclo 64): 1 en 0..62 — activa Horner dentro del
    ///   segmento, se apaga en el borde.
    /// - `link_*` (ciclo 256): un único 1 en la posición 62 del segmento
    ///   correspondiente — ata el acumulador completo.
    fn get_periodic_column_values(&self) -> Vec<Vec<Self::BaseField>> {
        let mut first = vec![BaseElement::ZERO; SEGMENT_LENGTH];
        first[0] = BaseElement::ONE;

        let mut cont = vec![BaseElement::ONE; SEGMENT_LENGTH];
        cont[SEGMENT_LENGTH - 1] = BaseElement::ZERO;

        let mut link_bal = vec![BaseElement::ZERO; TRACE_LENGTH];
        link_bal[SEGMENT_LENGTH - 2] = BaseElement::ONE; // pos 62

        let mut link_amt = vec![BaseElement::ZERO; TRACE_LENGTH];
        link_amt[2 * SEGMENT_LENGTH - 2] = BaseElement::ONE; // pos 126

        let mut link_db = vec![BaseElement::ZERO; TRACE_LENGTH];
        link_db[3 * SEGMENT_LENGTH - 2] = BaseElement::ONE; // pos 190

        let mut link_dl = vec![BaseElement::ZERO; TRACE_LENGTH];
        link_dl[4 * SEGMENT_LENGTH - 2] = BaseElement::ONE; // pos 254

        vec![first, cont, link_bal, link_amt, link_db, link_dl]
    }

    fn evaluate_transition<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        frame: &EvaluationFrame<E>,
        periodic_values: &[E],
        result: &mut [E],
    ) {
        let current = frame.current();
        let next = frame.next();

        let first = periodic_values[0];
        let cont = periodic_values[1];
        let link_bal = periodic_values[2];
        let link_amt = periodic_values[3];
        let link_db = periodic_values[4];
        let link_dl = periodic_values[5];

        let bit_cur = current[COL_BIT];
        let bit_next = next[COL_BIT];
        let acc_cur = current[COL_ACC];
        let acc_next = next[COL_ACC];

        // 0-1: los bits son booleanos (cur cubre 0..254, next cubre 1..255).
        result[0] = bit_cur * (bit_cur - E::ONE);
        result[1] = bit_next * (bit_next - E::ONE);

        // 2-4: las columnas de transporte son constantes.
        result[2] = next[COL_BAL] - current[COL_BAL];
        result[3] = next[COL_AMT] - current[COL_AMT];
        result[4] = next[COL_LIM] - current[COL_LIM];

        // 5-6: anclaje de la fila inicial de cada segmento — ¡con valores
        // privados, sin aserciones! El MSB (primer bit en big-endian)
        // debe ser cero (rango de 63 bits sobre Goldilocks), y Horner
        // arranca exactamente en ese bit.
        result[5] = first * bit_cur;
        result[6] = first * acc_cur;

        // 7: Horner dentro del segmento: acc = 2*acc + bit.
        result[7] = cont * (acc_next - (acc_cur + acc_cur + bit_next));

        // 8-11: los acumuladores completos se atan a la aritmética de
        // solvencia a través de las columnas de transporte.
        result[8] = link_bal * (acc_next - current[COL_BAL]);
        result[9] = link_amt * (acc_next - current[COL_AMT]);
        result[10] = link_db * (acc_next - (current[COL_BAL] - current[COL_AMT]));
        result[11] = link_dl * (acc_next - (current[COL_LIM] - current[COL_AMT]));
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        // El límite regulatorio es público y se ancla en la fila 0; la
        // restricción de constancia lo propaga a toda la traza.
        vec![Assertion::single(COL_LIM, 0, self.regulatory_limit)]
    }
}

pub struct SolvencyProver {
    options: ProofOptions,
}

impl SolvencyProver {
    pub fn new(options: ProofOptions) -> Self {
        Self { options }
    }
}

impl Prover for SolvencyProver {
    type BaseField = BaseElement;
    type Air = SolvencyAir;
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

    fn get_pub_inputs(&self, trace: &Self::Trace) -> SolvencyPublicInputs {
        SolvencyPublicInputs {
            regulatory_limit: trace.get(COL_LIM, 0),
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

    fn run_proof(balance: u64, amount: u64, limit: u64) -> Result<(), String> {
        let trace = build_trace(balance, amount, limit);
        let prover = SolvencyProver::new(default_options());

        // Capturar el posible panic de la assertion de depuración de
        // winterfell en debug (trazas inválidas), y tratarlo como rechazo.
        let previous_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let prove_result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| prover.prove(trace)));
        std::panic::set_hook(previous_hook);

        let proof: Proof = match prove_result {
            Err(_) => return Err("prove hizo panic (traza invalida detectada en debug)".into()),
            Ok(Err(e)) => return Err(format!("prove devolvio Err: {e:?}")),
            Ok(Ok(p)) => p,
        };

        let min_opts = AcceptableOptions::OptionSet(vec![prover.options().clone()]);
        verify::<SolvencyAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof,
            SolvencyPublicInputs {
                regulatory_limit: BaseElement::new(limit),
            },
            &min_opts,
        )
        .map_err(|e| format!("la verificacion fallo: {e:?}"))
    }

    /// La traza debe reconstruir cada valor en el acumulador final de su
    /// segmento — comprobación estructural previa a cualquier prueba.
    #[test]
    fn trace_accumulators_match_values() {
        let (balance, amount, limit) = (1_000_000u64, 250_000u64, 500_000u64);
        let trace = build_trace(balance, amount, limit);

        let expected = [
            balance,
            amount,
            balance - amount,
            limit - amount,
        ];
        for (seg, exp) in expected.iter().enumerate() {
            let last_row = (seg + 1) * SEGMENT_LENGTH - 1;
            assert_eq!(
                trace.get(COL_ACC, last_row),
                BaseElement::new(*exp),
                "el acumulador final del segmento {seg} no reconstruye su valor"
            );
        }
    }

    /// EL TEST CLAVE: una transacción solvente y dentro del límite
    /// produce una prueba verificable — con balance y amount PRIVADOS.
    #[test]
    fn valid_solvent_transaction_verifies() {
        let result = run_proof(1_000_000, 250_000, 500_000);
        assert!(result.is_ok(), "una transaccion valida deberia verificar: {result:?}");
    }

    /// EL TEST DE SOLIDEZ MÁS IMPORTANTE: gastar más del saldo debe
    /// fallar. La resta en el campo produce un valor gigante que no cabe
    /// en 63 bits, y el link del segmento diff_balance no cuadra.
    #[test]
    fn insufficient_balance_fails() {
        let result = run_proof(100_000, 250_000, 500_000); // amount > balance
        assert!(
            result.is_err(),
            "CRITICO: gastar mas del saldo no deberia producir una prueba verificable"
        );
    }

    /// Y el límite regulatorio: superarlo también debe fallar.
    #[test]
    fn amount_over_regulatory_limit_fails() {
        let result = run_proof(1_000_000, 750_000, 500_000); // amount > limit
        assert!(
            result.is_err(),
            "CRITICO: superar el limite regulatorio no deberia verificar"
        );
    }

    /// Declarar un límite público distinto al de la traza debe fallar.
    #[test]
    fn wrong_declared_limit_fails() {
        let trace = build_trace(1_000_000, 250_000, 500_000);
        let prover = SolvencyProver::new(default_options());
        let proof = prover.prove(trace).expect("prove no deberia fallar");

        let min_opts = AcceptableOptions::OptionSet(vec![prover.options().clone()]);
        let verification =
            verify::<SolvencyAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
                proof,
                SolvencyPublicInputs {
                    regulatory_limit: BaseElement::new(999_999),
                },
                &min_opts,
            );
        assert!(verification.is_err());
    }

    /// TEST DISCRIMINANTE: corromper una fila intermedia debe detectarse.
    #[test]
    fn corrupted_intermediate_row_is_detected() {
        let mut trace = build_trace(1_000_000, 250_000, 500_000);
        let original = trace.get(COL_ACC, 100);
        trace.set(COL_ACC, 100, original + BaseElement::ONE);

        let prover = SolvencyProver::new(default_options());

        let previous_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let prove_result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| prover.prove(trace)));
        std::panic::set_hook(previous_hook);

        match prove_result {
            Err(_) => { /* panic: detectado */ }
            Ok(Err(_)) => { /* Err: detectado */ }
            Ok(Ok(proof)) => {
                let min_opts = AcceptableOptions::OptionSet(vec![prover.options().clone()]);
                let verification = verify::<
                    SolvencyAir,
                    Blake3,
                    DefaultRandomCoin<Blake3>,
                    MerkleTree<Blake3>,
                >(
                    proof,
                    SolvencyPublicInputs {
                        regulatory_limit: BaseElement::new(500_000),
                    },
                    &min_opts,
                );
                assert!(
                    verification.is_err(),
                    "CRITICO: una traza corrompida no deberia verificar"
                );
            }
        }
    }

    /// El caso frontera `amount == balance` (diff_balance = 0). Con el
    /// diseño de Horner las columnas `bit` y `acc` siguen siendo de grado
    /// genérico (los otros tres segmentos no son cero), así que —a
    /// diferencia del range_check aislado— este caso podría pasar incluso
    /// en debug. Se deja como test NORMAL para que la evidencia decida;
    /// si la assertion de depuración salta, se reclasificará como
    /// #[ignore] con release, igual que en range_check.
    #[test]
    fn boundary_amount_equals_balance_verifies() {
        let result = run_proof(250_000, 250_000, 500_000);
        assert!(
            result.is_ok(),
            "amount == balance es legitimo y deberia verificar: {result:?}"
        );
    }
}
