//! Circuito mínimo de prueba en Halo2, más el primer bloque real portado
//! desde `zk-core`: el range check de 64 bits.
//!
//! ## Estado verificado hasta ahora
//! - `SquareCircuit` (satisfacibilidad con `MockProver`): ✅ compiló y
//!   pasó a la primera.
//! - `real_proof` (pipeline completo con IPA: setup determinista, keygen,
//!   prove, verify): ✅ compiló y pasó a la primera.
//! - `range_check` (primer bloque del circuito real portado): ✅ compiló
//!   y pasó (tras una corrección real: `query_fixed` no acepta rotación
//!   en esta versión).
//! - `poseidon_hash` (Poseidon real vía `halo2_gadgets`): ✅ compiló y
//!   pasó (tras dos correcciones reales: columna privada del chip, y
//!   columna de constantes faltante).
//! - `merkle` (árbol de Merkle de 20 niveles): ✅ compiló y pasó a la
//!   primera (sin correcciones, la pieza más compleja hasta ahora).
//! - `nullifier` (Poseidon con separación de dominio): ✅ compiló tras
//!   una corrección (enable_constant solo acepta columnas fijas).
//! - `compliance_circuit` (todo unificado: range check + árbol +
//!   nullifier + solvencia): ✅ compiló y pasó A LA PRIMERA — sin
//!   correcciones, pese a ser la pieza de mayor integración.
//! - `compliance_real_proof` (pipeline IPA real, con medición de
//!   tiempos): ✅ Confirmado viable: setup 176s, keygen_vk 20s, keygen_pk
//!   15s, prove 53s, verify 1.3s, prueba de 4KB.
//! - `iso_bridge` (mensaje ISO 20022 → prueba Halo2 real): ✅ compiló y
//!   pasó a la primera. Cierra el circulo completo del experimento.
//! - `persistent_nullifier_registry`: ✅ mismo diseño ya probado en la
//!   version Groth16, adaptado a `ff::PrimeField`.
//! - `settlement_prover_impl`: implementación de `SettlementProver`
//!   (trait compartido con `zk-core`) para este backend. Pendiente de
//!   verificar.

pub mod circuit_double_entry;
pub mod compliance_circuit;
pub mod compliance_real_proof;
pub mod iso_bridge;
pub mod merkle;
pub mod nullifier;
pub mod persistent_nullifier_registry;
pub mod poseidon_hash;
pub mod range_check;
pub mod settlement_prover_impl;

use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Instance, Selector},
    poly::Rotation,
};
use halo2_proofs::pasta::Fp;

/// Configuración del circuito: una columna "advice" (donde viven los
/// valores privados/intermedios), una columna "instance" (el valor
/// público `y`), y un selector que activa la restricción de multiplicación.
#[derive(Clone, Debug)]
pub struct SquareConfig {
    advice: Column<Advice>,
    instance: Column<Instance>,
    s_square: Selector,
}

/// Circuito que demuestra conocimiento de `x` tal que `x * x = y`.
#[derive(Default)]
pub struct SquareCircuit {
    pub x: Value<Fp>,
}

impl Circuit<Fp> for SquareCircuit {
    type Config = SquareConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
        let advice = meta.advice_column();
        let instance = meta.instance_column();
        let s_square = meta.selector();

        meta.enable_equality(advice);
        meta.enable_equality(instance);

        // Restricción: en la fila donde se activa el selector, el valor
        // de la fila siguiente debe ser el cuadrado del valor de la fila
        // actual. Esto es lo que "demuestra" que el probador conoce x
        // tal que x*x = (valor de la fila siguiente).
        meta.create_gate("square", |meta| {
            let x = meta.query_advice(advice, Rotation::cur());
            let x_squared = meta.query_advice(advice, Rotation::next());
            let s = meta.query_selector(s_square);
            vec![s * (x.clone() * x - x_squared)]
        });

        SquareConfig {
            advice,
            instance,
            s_square,
        }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<Fp>,
    ) -> Result<(), Error> {
        let x_squared_cell = layouter.assign_region(
            || "calcular x al cuadrado",
            |mut region| {
                config.s_square.enable(&mut region, 0)?;

                region.assign_advice(|| "x", config.advice, 0, || self.x)?;

                let x_squared = self.x.map(|x| x * x);
                region.assign_advice(|| "x al cuadrado", config.advice, 1, || x_squared)
            },
        )?;

        // Ata la celda calculada (x al cuadrado) a la entrada pública `y`
        // en la fila 0 de la columna instance — esto es lo que hace que
        // el verificador, que solo conoce `y`, pueda comprobar la prueba.
        layouter.constrain_instance(x_squared_cell.cell(), config.instance, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use halo2_proofs::dev::MockProver;

    /// EL TEST CLAVE de este primer intento: confirma que el circuito
    /// está bien formado y que, con un testigo válido (x=3, y=9), las
    /// restricciones se satisfacen. Esto es el equivalente de Halo2 a
    /// `cs.is_satisfied()` en Arkworks — no genera una prueba real
    /// todavía, solo confirma que la lógica del circuito es correcta.
    #[test]
    fn valid_witness_satisfies_circuit() {
        let k = 4; // tamaño del circuito: 2^k filas: de sobra para esto.
        let x = Fp::from(3);
        let y = Fp::from(9); // 3 * 3 = 9

        let circuit = SquareCircuit { x: Value::known(x) };
        let public_input = vec![y];

        let prover = MockProver::run(k, &circuit, vec![public_input]).unwrap();
        assert_eq!(prover.verify(), Ok(()));
    }

    /// Confirma que un testigo INVÁLIDO (x=3 pero declarando y=10, que no
    /// es 3*3) hace que el circuito falle su propia comprobación interna.
    #[test]
    fn invalid_witness_fails_circuit() {
        let k = 4;
        let x = Fp::from(3);
        let wrong_y = Fp::from(10); // 3 * 3 = 9, no 10

        let circuit = SquareCircuit { x: Value::known(x) };
        let public_input = vec![wrong_y];

        let prover = MockProver::run(k, &circuit, vec![public_input]).unwrap();
        assert!(
            prover.verify().is_err(),
            "un testigo con y incorrecto NO deberia satisfacer el circuito"
        );
    }
}

/// Pipeline de prueba REAL con IPA: parámetros, claves, generación y
/// verificación de una prueba criptográfica de verdad — no solo
/// satisfacibilidad con `MockProver`.
///
/// ## ⚠️ La pieza de mayor incertidumbre de todo el proyecto
///
/// Los transcripts (`Blake2bWrite`/`Blake2bRead`), el tipo `SingleVerifier`
/// como estrategia de verificación, y la firma exacta de
/// `create_proof`/`verify_proof` (incluido el anidamiento de los inputs
/// públicos en tres niveles: por circuito, por columna instance, por
/// valor) son la parte de la API de Halo2 en la que tengo MENOS confianza
/// de todo este proyecto — más que `poseidon_hash.rs` en su momento. Si
/// esto falla, no me sorprendería que necesitara varias rondas.
///
/// ## Por qué NO hay trusted setup aquí
///
/// `Params::new(k)` genera los parámetros IPA de forma completamente
/// DETERMINISTA (a partir de `k` y constantes fijas del esquema, sin
/// ningún secreto aleatorio que deba destruirse después). Esto es
/// exactamente lo que perseguíamos: cualquiera puede regenerar los mismos
/// parámetros de forma independiente y comprobar que coinciden — no hace
/// falta confiar en que nadie "destruyó" nada.
pub mod real_proof {
    use super::{Fp, SquareCircuit};
    use halo2_proofs::pasta::EqAffine;
    use halo2_proofs::plonk::{create_proof, keygen_pk, keygen_vk, verify_proof, SingleVerifier};
    use halo2_proofs::poly::commitment::Params;
    use halo2_proofs::transcript::{Blake2bRead, Blake2bWrite, Challenge255};
    use halo2_proofs::circuit::Value;
    use rand_core::OsRng;

    /// Ejecuta el flujo completo: setup determinista -> keygen -> prove ->
    /// verify, con un testigo válido. Devuelve `Ok(())` si la prueba real
    /// (no solo `MockProver`) se genera y verifica correctamente.
    pub fn run_end_to_end(k: u32, x: u64, y: u64) -> Result<(), String> {
        let params: Params<EqAffine> = Params::new(k);

        // El circuito "vacío" (sin testigo) se usa para derivar las claves
        // — la estructura de restricciones no depende del valor de x.
        let empty_circuit = SquareCircuit { x: Value::unknown() };
        let vk = keygen_vk(&params, &empty_circuit)
            .map_err(|e| format!("fallo en keygen_vk: {e:?}"))?;
        let pk = keygen_pk(&params, vk.clone(), &empty_circuit)
            .map_err(|e| format!("fallo en keygen_pk: {e:?}"))?;

        let circuit = SquareCircuit {
            x: Value::known(Fp::from(x)),
        };
        let public_value = Fp::from(y);

        let mut transcript = Blake2bWrite::<_, EqAffine, Challenge255<_>>::init(vec![]);
        create_proof(
            &params,
            &pk,
            &[circuit],
            &[&[&[public_value]]],
            OsRng,
            &mut transcript,
        )
        .map_err(|e| format!("fallo al generar la prueba: {e:?}"))?;
        let proof = transcript.finalize();

        let strategy = SingleVerifier::new(&params);
        let mut transcript_reader = Blake2bRead::<_, EqAffine, Challenge255<_>>::init(&proof[..]);
        verify_proof(
            &params,
            &vk,
            strategy,
            &[&[&[public_value]]],
            &mut transcript_reader,
        )
        .map_err(|e| format!("fallo al verificar la prueba: {e:?}"))?;

        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        /// EL TEST CLAVE de esta pieza: una prueba IPA real (no MockProver)
        /// se genera y verifica correctamente para un testigo válido.
        #[test]
        fn end_to_end_real_ipa_proof_valid_witness() {
            let result = run_end_to_end(4, 3, 9); // x=3, y=9=3*3
            assert!(
                result.is_ok(),
                "el flujo completo de prueba IPA deberia funcionar con un testigo valido: {result:?}"
            );
        }

        /// Confirma que un testigo inválido (x=3, pero declarando y=10)
        /// hace fallar la VERIFICACIÓN real, no solo MockProver.
        #[test]
        fn end_to_end_real_ipa_proof_invalid_witness_fails_verification() {
            let result = run_end_to_end(4, 3, 10); // x=3, pero y declarado = 10, no 9
            assert!(
                result.is_err(),
                "CRITICO: una prueba con testigo invalido no deberia verificar correctamente"
            );
        }
    }
}
