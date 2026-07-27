//! Poseidon real vía `halo2_gadgets::poseidon` — el mismo gadget que usa
//! Zcash en producción para Orchard. Primer paso, deliberadamente
//! pequeño: un circuito que hashea DOS elementos de campo y expone el
//! resultado como público, sin construir el árbol de Merkle todavía.
//!
//! ## ⚠️ Nivel de riesgo: el más alto de todo el experimento con Halo2
//!
//! La API de `Pow5Chip`/`Pow5Config` es la más cargada de genéricos que
//! hemos tocado en Halo2 (`Spec`, constantes de tipo `WIDTH`/`RATE`,
//! `ConstantLength`). Es razonable esperar más rondas de corrección aquí
//! que en el range check.
//!
//! `P128Pow5T3` es la especificación estándar de Poseidon con ancho 3
//! (rate=2, capacity=1) que usa Orchard — coherente con el diseño que ya
//! usamos en `zk-core::poseidon_hash` (rate=2, capacity=1 también).

use halo2_gadgets::poseidon::{
    primitives::{ConstantLength, P128Pow5T3},
    Hash, Pow5Chip, Pow5Config,
};
use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Instance},
};
use halo2_proofs::pasta::Fp;

const WIDTH: usize = 3;
const RATE: usize = 2;

#[derive(Clone, Debug)]
pub struct PoseidonHashConfig {
    pow5_config: Pow5Config<Fp, WIDTH, RATE>,
    /// Columna PROPIA (no las columnas internas, privadas, del chip de
    /// Poseidon) para cargar los valores de entrada x, y antes de
    /// pasárselos al gadget.
    message_column: Column<Advice>,
    instance: Column<Instance>,
}

/// Circuito que demuestra conocimiento de `x`, `y` tales que
/// `Poseidon(x, y) = resultado`, con `resultado` público.
#[derive(Default)]
pub struct PoseidonHashCircuit {
    pub x: Value<Fp>,
    pub y: Value<Fp>,
}

impl Circuit<Fp> for PoseidonHashCircuit {
    type Config = PoseidonHashConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
        let state: [Column<Advice>; WIDTH] = std::array::from_fn(|_| meta.advice_column());
        let partial_sbox = meta.advice_column();
        let rc_a: [Column<halo2_proofs::plonk::Fixed>; WIDTH] =
            std::array::from_fn(|_| meta.fixed_column());
        let rc_b: [Column<halo2_proofs::plonk::Fixed>; WIDTH] =
            std::array::from_fn(|_| meta.fixed_column());
        let instance = meta.instance_column();
        let message_column = meta.advice_column();

        meta.enable_equality(instance);
        meta.enable_equality(message_column);
        for column in state.iter() {
            meta.enable_equality(*column);
        }

        // El chip de Poseidon necesita al menos una columna habilitada
        // para cargar CONSTANTES (p. ej. el relleno de la capacidad del
        // esponja) — sin esto, la configuración falla en tiempo de
        // ejecución con `NotEnoughColumnsForConstants` (confirmado por
        // MockProver real, no supuesto). Se reutiliza la primera columna
        // fija de `rc_b` para esto, el patrón habitual en los propios
        // ejemplos de `halo2_gadgets`.
        meta.enable_constant(rc_b[0]);

        let pow5_config = Pow5Chip::configure::<P128Pow5T3>(
            meta,
            state,
            partial_sbox,
            rc_a,
            rc_b,
        );

        PoseidonHashConfig {
            pow5_config,
            message_column,
            instance,
        }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<Fp>,
    ) -> Result<(), Error> {
        let pow5_chip = Pow5Chip::construct(config.pow5_config.clone());

        // Asignar x, y como celdas iniciales dentro de una región propia,
        // para poder pasárselas al gadget de Poseidon como mensaje.
        let (x_cell, y_cell) = layouter.assign_region(
            || "cargar x, y",
            |mut region| {
                let x_cell = region.assign_advice(
                    || "x",
                    config.message_column,
                    0,
                    || self.x,
                )?;
                let y_cell = region.assign_advice(
                    || "y",
                    config.message_column,
                    1,
                    || self.y,
                )?;
                Ok((x_cell, y_cell))
            },
        )?;

        let hasher = Hash::<Fp, Pow5Chip<Fp, WIDTH, RATE>, P128Pow5T3, ConstantLength<2>, WIDTH, RATE>::init(
            pow5_chip,
            layouter.namespace(|| "init poseidon"),
        )?;

        let result_cell = hasher.hash(layouter.namespace(|| "hash"), [x_cell, y_cell])?;

        layouter.constrain_instance(result_cell.cell(), config.instance, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use halo2_gadgets::poseidon::primitives::Hash as PoseidonHashPrimitive;
    use halo2_proofs::dev::MockProver;

    /// Calcula Poseidon(x, y) de forma NATIVA (fuera de circuito), usando
    /// las primitivas de `halo2_gadgets`, para saber qué valor público
    /// esperar sin tener que adivinarlo.
    fn native_poseidon_hash(x: Fp, y: Fp) -> Fp {
        PoseidonHashPrimitive::<Fp, P128Pow5T3, ConstantLength<2>, WIDTH, RATE>::init()
            .hash([x, y])
    }

    /// EL TEST CLAVE: el circuito acepta el resultado correcto de
    /// Poseidon(x, y) como input público.
    #[test]
    fn correct_hash_satisfies_circuit() {
        let k = 7; // Poseidon necesita más filas que los circuitos anteriores.
        let x = Fp::from(3);
        let y = Fp::from(5);
        let expected = native_poseidon_hash(x, y);

        let circuit = PoseidonHashCircuit {
            x: Value::known(x),
            y: Value::known(y),
        };

        let prover = MockProver::run(k, &circuit, vec![vec![expected]]).unwrap();
        assert_eq!(prover.verify(), Ok(()));
    }

    /// Confirma que declarar un resultado INCORRECTO como público hace
    /// fallar el circuito.
    #[test]
    fn wrong_hash_fails_circuit() {
        let k = 7;
        let x = Fp::from(3);
        let y = Fp::from(5);
        let wrong_result = Fp::from(999_999);

        let circuit = PoseidonHashCircuit {
            x: Value::known(x),
            y: Value::known(y),
        };

        let prover = MockProver::run(k, &circuit, vec![vec![wrong_result]]).unwrap();
        assert!(prover.verify().is_err());
    }
}
