//! Nullifier en Halo2: `Poseidon(Poseidon(domain, account_id), nonce)`,
//! con separación de dominio — análogo a `zk-core::nullifier`, con la
//! misma justificación (evitar que el nullifier se confunda con la hoja
//! del árbol u otro hash del sistema).
//!
//! Reutiliza el mismo patrón que `poseidon_hash.rs` (ya verificado, con
//! sus dos correcciones ya aplicadas: columna propia en vez del campo
//! privado del chip, y `enable_constant` para las constantes internas del
//! chip). La única pieza nueva aquí es cargar la constante de dominio
//! como una celda del circuito mediante `assign_advice_from_constant`,
//! para lo cual también hace falta `enable_constant` mas en nuestra
//! propia columna, no solo en la interna del chip de Poseidon.

use halo2_gadgets::poseidon::{
    primitives::{ConstantLength, P128Pow5T3},
    Hash, Pow5Chip, Pow5Config,
};
use halo2_proofs::pasta::Fp;
use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Instance},
};

const WIDTH: usize = 3;
const RATE: usize = 2;

/// Misma constante de separación de dominio que en `zk-core::nullifier`.
pub const NULLIFIER_DOMAIN: u64 = 0x4E554C4C; // "NULL", solo mnemónico

#[derive(Clone, Debug)]
pub struct NullifierConfig {
    pow5_config: Pow5Config<Fp, WIDTH, RATE>,
    message_column: Column<Advice>,
    instance: Column<Instance>,
}

/// Circuito que demuestra conocimiento de `account_id`, `account_nonce`
/// tales que `nullifier = Poseidon(Poseidon(DOMAIN, account_id), account_nonce)`,
/// con `nullifier` público.
#[derive(Default)]
pub struct NullifierCircuit {
    pub account_id: Value<Fp>,
    pub account_nonce: Value<Fp>,
}

impl Circuit<Fp> for NullifierCircuit {
    type Config = NullifierConfig;
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
        // Necesario para las constantes internas del chip de Poseidon.
        // Una única columna fija habilitada como constante basta para
        // todo el circuito (incluida la carga de NULLIFIER_DOMAIN más
        // abajo vía `assign_advice_from_constant`) — no hace falta (y de
        // hecho `enable_constant` solo acepta columnas FIJAS, no
        // "advice", confirmado por el compilador real) una segunda
        // llamada sobre `message_column`.
        meta.enable_constant(rc_b[0]);

        let pow5_config = Pow5Chip::configure::<P128Pow5T3>(meta, state, partial_sbox, rc_a, rc_b);

        NullifierConfig {
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
        // --- Primer hash: Poseidon(domain, account_id) ---
        let (domain_cell, account_id_cell) = layouter.assign_region(
            || "cargar domain, account_id",
            |mut region| {
                let domain_cell = region.assign_advice_from_constant(
                    || "domain",
                    config.message_column,
                    0,
                    Fp::from(NULLIFIER_DOMAIN),
                )?;
                let account_id_cell = region.assign_advice(
                    || "account_id",
                    config.message_column,
                    1,
                    || self.account_id,
                )?;
                Ok((domain_cell, account_id_cell))
            },
        )?;

        let pow5_chip_1 = Pow5Chip::construct(config.pow5_config.clone());
        let hasher_1 = Hash::<Fp, Pow5Chip<Fp, WIDTH, RATE>, P128Pow5T3, ConstantLength<2>, WIDTH, RATE>::init(
            pow5_chip_1,
            layouter.namespace(|| "init poseidon 1"),
        )?;
        let inner_cell = hasher_1.hash(
            layouter.namespace(|| "hash 1: poseidon(domain, account_id)"),
            [domain_cell, account_id_cell],
        )?;

        // --- Segundo hash: Poseidon(inner, account_nonce) ---
        let nonce_cell = layouter.assign_region(
            || "cargar account_nonce",
            |mut region| {
                region.assign_advice(|| "account_nonce", config.message_column, 0, || self.account_nonce)
            },
        )?;

        let pow5_chip_2 = Pow5Chip::construct(config.pow5_config.clone());
        let hasher_2 = Hash::<Fp, Pow5Chip<Fp, WIDTH, RATE>, P128Pow5T3, ConstantLength<2>, WIDTH, RATE>::init(
            pow5_chip_2,
            layouter.namespace(|| "init poseidon 2"),
        )?;
        let nullifier_cell = hasher_2.hash(
            layouter.namespace(|| "hash 2: poseidon(inner, account_nonce)"),
            [inner_cell, nonce_cell],
        )?;

        layouter.constrain_instance(nullifier_cell.cell(), config.instance, 0)
    }
}

/// Cálculo nativo (fuera de circuito) del nullifier, para saber qué valor
/// público esperar en los tests sin adivinarlo.
pub fn native_nullifier(account_id: Fp, account_nonce: Fp) -> Fp {
    let native_hash = |a: Fp, b: Fp| {
        halo2_gadgets::poseidon::primitives::Hash::<Fp, P128Pow5T3, ConstantLength<2>, WIDTH, RATE>::init()
            .hash([a, b])
    };
    let domain = Fp::from(NULLIFIER_DOMAIN);
    let inner = native_hash(domain, account_id);
    native_hash(inner, account_nonce)
}

#[cfg(test)]
mod tests {
    use super::*;
    use halo2_proofs::dev::MockProver;

    /// EL TEST CLAVE: el nullifier calculado correctamente (nativo)
    /// satisface el circuito cuando se declara como input público.
    #[test]
    fn correct_nullifier_satisfies_circuit() {
        let k = 8;
        let account_id = Fp::from(12345);
        let account_nonce = Fp::from(1);
        let expected_nullifier = native_nullifier(account_id, account_nonce);

        let circuit = NullifierCircuit {
            account_id: Value::known(account_id),
            account_nonce: Value::known(account_nonce),
        };

        let prover = MockProver::run(k, &circuit, vec![vec![expected_nullifier]]).unwrap();
        assert_eq!(prover.verify(), Ok(()));
    }

    /// EL TEST DE SEGURIDAD: declarar un nullifier FALSIFICADO (que no
    /// corresponde a account_id/account_nonce reales) hace fallar el
    /// circuito. Sin esto, el nullifier seria decorativo — cualquiera
    /// podria declarar uno inventado para evadir el registro de gastados.
    #[test]
    fn forged_nullifier_fails_circuit() {
        let k = 8;
        let account_id = Fp::from(12345);
        let account_nonce = Fp::from(1);
        let forged_nullifier = Fp::from(999_999_999);

        let circuit = NullifierCircuit {
            account_id: Value::known(account_id),
            account_nonce: Value::known(account_nonce),
        };

        let prover = MockProver::run(k, &circuit, vec![vec![forged_nullifier]]).unwrap();
        assert!(
            prover.verify().is_err(),
            "CRITICO: un nullifier falsificado no deberia satisfacer el circuito"
        );
    }

    /// Confirma que avanzar el nonce cambia el nullifier (misma propiedad
    /// que se comprobó en zk-core).
    #[test]
    fn different_nonce_produces_different_nullifier() {
        let account_id = Fp::from(12345);
        let n1 = native_nullifier(account_id, Fp::from(1));
        let n2 = native_nullifier(account_id, Fp::from(2));
        assert_ne!(n1, n2);
    }
}
