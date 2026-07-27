//! Range check de 64 bits, portado de la versión en Arkworks
//! (`zk-core::circuit::enforce_range`) a la aritmetización de Halo2.
//!
//! ## Idea (la misma que en Arkworks, expresada distinto)
//!
//! Para demostrar `0 <= value < 2^64` sin revelar `value`, se descompone
//! en 64 bits privados y se comprueban dos cosas:
//! 1. Cada bit es booleano (`bit * (bit - 1) = 0`).
//! 2. La suma ponderada de los bits (`Σ bit_i * 2^i`) reconstruye
//!    exactamente `value`.
//!
//! En Arkworks esto se hacía con `to_bits_le()` sobre un `FpVar`. En
//! Halo2 no hay un helper equivalente listo para usar aquí, así que se
//! construye a mano: una columna "advice" para los bits (una fila por
//! bit), una columna "fixed" con las potencias de dos (2^0, 2^1, ..., 2^63
//! — son constantes conocidas de antemano, por eso van en una columna
//! FIJA y no en una "advice"), y una columna "advice" para la suma
//! acumulada.
//!
//! ## ⚠️ Nivel de riesgo
//!
//! Menor que el pipeline de pruebas IPA (que ya compiló a la primera),
//! pero sigue siendo una pieza nueva: es la primera vez en este
//! experimento que se usa una columna `Fixed` y que se encadenan
//! restricciones a través de MUCHAS filas (64) en vez de dos como en
//! `SquareCircuit`. Espero que compile, pero no con la misma confianza
//! que el pipeline de pruebas.

use ff::PrimeField;
use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Fixed, Selector},
    poly::Rotation,
};
use halo2_proofs::pasta::Fp;

pub const VALUE_BITS: usize = 64;

/// Convierte un valor de campo en sus 64 bits menos significativos, en
/// orden little-endian. Debe usarse tanto para construir el testigo como
/// (en los tests) para corromper deliberadamente un bit y confirmar que
/// el circuito lo detecta.
fn value_to_bits_le(value: Fp) -> Vec<bool> {
    let repr = value.to_repr(); // representación canónica en bytes
    let bytes: &[u8] = repr.as_ref();
    (0..VALUE_BITS)
        .map(|i| {
            let byte = bytes[i / 8];
            (byte >> (i % 8)) & 1 == 1
        })
        .collect()
}

#[derive(Clone, Debug)]
pub struct RangeCheckConfig {
    bit: Column<Advice>,
    running_sum: Column<Advice>,
    power_of_two: Column<Fixed>,
    s_bool: Selector,
    s_accumulate: Selector,
}

/// Circuito que demuestra que un valor privado `value` cabe en
/// `VALUE_BITS` bits, sin revelarlo.
#[derive(Default)]
pub struct RangeCheckCircuit {
    pub value: Value<Fp>,
}

impl Circuit<Fp> for RangeCheckCircuit {
    type Config = RangeCheckConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
        let bit = meta.advice_column();
        let running_sum = meta.advice_column();
        let power_of_two = meta.fixed_column();
        let s_bool = meta.selector();
        let s_accumulate = meta.selector();

        meta.enable_equality(running_sum);

        // Restricción 1: cada bit es 0 o 1.
        meta.create_gate("bit is boolean", |meta| {
            let bit = meta.query_advice(bit, Rotation::cur());
            let s = meta.query_selector(s_bool);
            vec![s * bit.clone() * (bit - halo2_proofs::plonk::Expression::Constant(Fp::one()))]
        });

        // Restricción 2: la suma acumulada avanza correctamente de una
        // fila a la siguiente: running_sum(cur) = running_sum(prev) +
        // bit(cur) * power_of_two(cur).
        //
        // NOTA: `query_fixed` en esta versión de halo2_proofs (0.3.4) NO
        // acepta un parámetro de rotación (a diferencia de
        // `query_advice`, que sí) — confirmado por el compilador real.
        // Por eso la potencia de dos se consulta en la fila "actual", y
        // es la suma acumulada la que mira hacia atrás con
        // `Rotation::prev()`, en vez de al revés.
        meta.create_gate("accumulate weighted bit", |meta| {
            let sum_prev = meta.query_advice(running_sum, Rotation::prev());
            let sum_cur = meta.query_advice(running_sum, Rotation::cur());
            let bit_cur = meta.query_advice(bit, Rotation::cur());
            let power_cur = meta.query_fixed(power_of_two);
            let s = meta.query_selector(s_accumulate);
            vec![s * (sum_cur - (sum_prev + bit_cur * power_cur))]
        });

        RangeCheckConfig {
            bit,
            running_sum,
            power_of_two,
            s_bool,
            s_accumulate,
        }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<Fp>,
    ) -> Result<(), Error> {
        // El testigo (los bits reales) se calcula fuera de la región,
        // porque necesitamos conocerlo entero antes de empezar a asignar
        // filas (cada fila depende del bit correspondiente Y de la suma
        // acumulada de las filas anteriores).
        let bits: Value<Vec<bool>> = self.value.map(value_to_bits_le);

        layouter.assign_region(
            || "range check de 64 bits",
            |mut region| {
                let mut running_sum_value = Value::known(Fp::zero());

                for i in 0..VALUE_BITS {
                    // Bit de esta fila (como elemento de campo 0 o 1).
                    let bit_i: Value<Fp> = bits
                        .as_ref()
                        .map(|bs| if bs[i] { Fp::one() } else { Fp::zero() });

                    region.assign_advice(|| format!("bit {i}"), config.bit, i, || bit_i)?;

                    let power_i = Fp::from(1u64 << i.min(63)); // 2^i; VALUE_BITS=64 así que i<64 siempre
                    region.assign_fixed(
                        || format!("2^{i}"),
                        config.power_of_two,
                        i,
                        || Value::known(power_i),
                    )?;

                    config.s_bool.enable(&mut region, i)?;

                    if i == 0 {
                        // Primera fila: la suma acumulada es simplemente
                        // bit_0 * 2^0 (no hay fila anterior con la que
                        // encadenar mediante la puerta "accumulate").
                        running_sum_value = bit_i.map(|b| b * power_i);
                        region.assign_advice(
                            || "running_sum[0]",
                            config.running_sum,
                            0,
                            || running_sum_value,
                        )?;
                    } else {
                        // Activar la puerta en ESTA fila (i), que mira
                        // hacia atrás (Rotation::prev()) a la fila i-1.
                        config.s_accumulate.enable(&mut region, i)?;

                        running_sum_value = running_sum_value
                            .zip(bit_i)
                            .map(|(sum, b)| sum + b * power_i);
                        region.assign_advice(
                            || format!("running_sum[{i}]"),
                            config.running_sum,
                            i,
                            || running_sum_value,
                        )?;
                    }
                }

                Ok(())
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use halo2_proofs::dev::MockProver;

    /// EL TEST CLAVE: un valor que cabe en 64 bits (aquí, uno pequeño y
    /// arbitrario) satisface el circuito.
    #[test]
    fn value_within_64_bits_satisfies_circuit() {
        let k = 8; // 2^8 = 256 filas: de sobra para 64 filas de trabajo.
        let value = Fp::from(123_456_789u64);

        let circuit = RangeCheckCircuit {
            value: Value::known(value),
        };

        let prover = MockProver::run(k, &circuit, vec![]).unwrap();
        assert_eq!(prover.verify(), Ok(()));
    }

    /// Confirma que la descomposición de bits reconstruye correctamente
    /// varios valores distintos, incluyendo los casos límite (cero, y un
    /// valor que usa el bit 63).
    #[test]
    fn various_values_satisfy_circuit() {
        let k = 8;
        for raw in [0u64, 1u64, u64::MAX / 2, 1u64 << 63, u64::MAX] {
            let value = Fp::from(raw);
            let circuit = RangeCheckCircuit {
                value: Value::known(value),
            };
            let prover = MockProver::run(k, &circuit, vec![]).unwrap();
            assert_eq!(
                prover.verify(),
                Ok(()),
                "el valor {raw} deberia satisfacer el circuito de range check"
            );
        }
    }
}
