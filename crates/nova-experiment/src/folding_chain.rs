//! Cadena de plegado mínima: cada paso pliega **una transacción** sobre
//! un compromiso acumulado.
//!
//! ```text
//! z_{i+1} = Poseidon(z_i, importe_i)
//! ```
//!
//! Es el equivalente de `x³+42` en STARK o `a+b` en PLONK — la pieza más
//! simple posible para validar el ciclo completo — pero con una
//! diferencia: **ya demuestra lo que hace único a Nova**, porque pliega N
//! pasos y permite medir el coste marginal de cada uno.
//!
//! ## Lo que representa en el dominio bancario
//!
//! Un compromiso acumulado a la secuencia de transacciones liquidadas
//! durante la jornada. Al cierre, un solo valor `z_N` resume
//! criptográficamente todo el día, y la prueba comprimida certifica que
//! cada paso se ejecutó según las reglas.
//!
//! ## El pipeline completo de Nova
//!
//! 1. `PublicParams::setup` — una vez por circuito.
//! 2. `RecursiveSNARK::new` — arranca con el estado inicial `z_0`.
//! 3. `prove_step` × N — **aquí está lo interesante**: cada paso es
//!    barato e independiente del número de pasos anteriores.
//! 4. `verify` — comprueba el estado plegado.
//! 5. `CompressedSNARK::prove` — comprime a una prueba entregable.
//!
//! Los pasos 1 y 5 son caros; el 3 es el que se repite miles de veces.

use generic_array::typenum::U24;
use nova_snark::frontend::{
    gadgets::poseidon::{
        Elt, IOPattern, Simplex, Sponge, SpongeAPI, SpongeCircuit, SpongeOp, SpongeTrait, Strength,
    },
    num::AllocatedNum,
    ConstraintSystem, SynthesisError,
};
use nova_snark::provider::{Bn256EngineKZG, GrumpkinEngine};
use nova_snark::traits::{circuit::StepCircuit, Group};

/// Curva primaria, con compromiso HyperKZG.
pub type E1 = Bn256EngineKZG;
/// Curva secundaria del ciclo.
pub type E2 = GrumpkinEngine;
pub type EE1 = nova_snark::provider::hyperkzg::EvaluationEngine<E1>;
pub type EE2 = nova_snark::provider::ipa_pc::EvaluationEngine<E2>;
/// SNARK de compresión (Spartan) para cada curva.
pub type S1 = nova_snark::spartan::snark::RelaxedR1CSSNARK<E1, EE1>;
pub type S2 = nova_snark::spartan::snark::RelaxedR1CSSNARK<E2, EE2>;

/// Un paso de la cadena: pliega un importe sobre el compromiso acumulado.
#[derive(Clone, Debug)]
pub struct TransactionStep<G: Group> {
    /// Importe de la transacción de este paso (privado).
    pub amount: G::Scalar,
}

impl<G: Group> TransactionStep<G> {
    pub fn new(amount: u64) -> Self {
        Self {
            amount: G::Scalar::from(amount),
        }
    }
}

impl<G: Group> StepCircuit<G::Scalar> for TransactionStep<G> {
    /// Un solo elemento de estado: el compromiso acumulado.
    fn arity(&self) -> usize {
        1
    }

    fn synthesize<CS: ConstraintSystem<G::Scalar>>(
        &self,
        cs: &mut CS,
        z_in: &[AllocatedNum<G::Scalar>],
    ) -> Result<Vec<AllocatedNum<G::Scalar>>, SynthesisError> {
        assert_eq!(z_in.len(), 1);

        // El importe es un testigo privado del paso.
        let amount = AllocatedNum::alloc(cs.namespace(|| "amount"), || Ok(self.amount))?;

        // z_{i+1} = Poseidon(z_i, amount)
        let elts = vec![
            Elt::Allocated(z_in[0].clone()),
            Elt::Allocated(amount),
        ];

        // El io-pattern (2 absorciones, 1 exprimido) queda codificado en
        // el tag de la esponja, así que actúa de separación de dominio
        // por estructura — igual que el `Domain` de dusk-poseidon.
        let parameter = IOPattern(vec![SpongeOp::Absorb(2), SpongeOp::Squeeze(1)]);
        let pc = Sponge::<G::Scalar, U24>::api_constants(Strength::Standard);

        let mut ns = cs.namespace(|| "poseidon");
        let z_out = {
            let mut sponge = SpongeCircuit::new_with_constants(&pc, Simplex);
            let acc = &mut ns;
            sponge.start(parameter, None, acc);
            SpongeAPI::absorb(&mut sponge, 2, &elts, acc);
            let output = SpongeAPI::squeeze(&mut sponge, 1, acc);
            // `Unsatisfiable` en el frontend de Nova NO es una variante
            // unitaria como en bellpepper: es `fn(String) -> SynthesisError`.
            sponge
                .finish(acc)
                .map_err(|e| SynthesisError::Unsatisfiable(format!("esponja: {e:?}")))?;
            Elt::ensure_allocated(&output[0], &mut ns.namespace(|| "ensure allocated"))?
        };

        Ok(vec![z_out])
    }
}

// Los tests requieren la feature `test-setup`: sin ella, `nova-snark`
// se niega a generar los parametros publicos localmente, que es
// exactamente el comportamiento que se quiere en produccion.
//
//   cargo test -p nova-experiment --release --features test-setup
#[cfg(all(test, feature = "test-setup"))]
mod tests {
    use super::*;
    use ff::Field;
    use nova_snark::nova::{CompressedSNARK, PublicParams, RecursiveSNARK};
    use nova_snark::traits::Engine;
    use nova_snark::traits::snark::RelaxedR1CSSNARKTrait;
    use std::time::Instant;

    type C = TransactionStep<<E1 as Engine>::GE>;

    /// EL TEST CLAVE, y el que mide lo que ningún otro backend puede:
    /// **el coste marginal de la transacción número N+1**.
    ///
    /// Los otros cuatro backends generan una prueba monolítica cuyo coste
    /// crece con el tamaño del circuito. Nova pliega paso a paso, y cada
    /// paso debería costar aproximadamente lo mismo independientemente de
    /// cuántos lo precedan. Este test lo comprueba con datos.
    #[test]
    fn folding_chain_full_cycle_with_marginal_cost() {
        let num_steps = 10usize;

        // Importes ficticios de las transacciones de la "jornada".
        let circuits: Vec<C> = (0..num_steps)
            .map(|i| TransactionStep::new(100_000 + i as u64 * 1_000))
            .collect();

        // --- 1. Parámetros públicos (una vez por circuito) ---
        let t0 = Instant::now();
        let pp = PublicParams::<E1, E2, C>::setup(&circuits[0], &*S1::ck_floor(), &*S2::ck_floor())
            .expect("el setup no deberia fallar");
        println!("PublicParams::setup       : {:?}", t0.elapsed());
        println!(
            "Restricciones por paso    : {} (primaria) / {} (secundaria)",
            pp.num_constraints().0,
            pp.num_constraints().1
        );

        // --- 2. Arranque de la cadena ---
        let z0 = vec![<E1 as Engine>::Scalar::ZERO];
        let mut recursive_snark = RecursiveSNARK::<E1, E2, C>::new(&pp, &circuits[0], &z0)
            .expect("RecursiveSNARK::new no deberia fallar");

        // --- 3. Plegado: AQUI ESTA LO INTERESANTE ---
        println!("--- coste marginal por paso ---");
        let mut step_times = Vec::new();
        for (i, circuit) in circuits.iter().enumerate() {
            let t = Instant::now();
            recursive_snark
                .prove_step(&pp, circuit)
                .expect("prove_step no deberia fallar");
            let elapsed = t.elapsed();
            step_times.push(elapsed);
            // NOTA: el paso 0 sale en nanosegundos porque
            // `RecursiveSNARK::new` YA ejecuta el primer paso; este
            // `prove_step` inicial es una operacion vacia. Por eso la
            // comparacion de mas abajo usa el paso 1, no el 0.
            println!("  paso {i:2} : {elapsed:?}");
        }

        // --- 4. Verificar el estado plegado ---
        let t = Instant::now();
        recursive_snark
            .verify(&pp, num_steps, &z0)
            .expect("el estado plegado deberia verificar");
        println!("RecursiveSNARK::verify    : {:?}", t.elapsed());

        // --- 5. Comprimir a una prueba entregable ---
        let t = Instant::now();
        let (pk, vk) = CompressedSNARK::<_, _, _, S1, S2>::setup(&pp)
            .expect("el setup de compresion no deberia fallar");
        println!("CompressedSNARK::setup    : {:?}", t.elapsed());

        let t = Instant::now();
        let compressed = CompressedSNARK::<_, _, _, S1, S2>::prove(&pp, &pk, &recursive_snark)
            .expect("la compresion no deberia fallar");
        println!("CompressedSNARK::prove    : {:?}", t.elapsed());

        let t = Instant::now();
        compressed
            .verify(&vk, num_steps, &z0)
            .expect("la prueba comprimida deberia verificar");
        println!("CompressedSNARK::verify   : {:?}", t.elapsed());

        // --- El dato que justifica todo el backend ---
        // El ultimo paso no deberia costar significativamente mas que el
        // primero: ese es el sentido del plegado. Se comprueba con un
        // margen amplio porque hay ruido de medicion y el primer paso
        // incluye inicializacion.
        let primero = step_times[1].as_micros() as f64;
        let ultimo = step_times[num_steps - 1].as_micros() as f64;
        println!(
            "--- coste del ultimo paso / segundo paso: {:.2}x ---",
            ultimo / primero
        );
        assert!(
            ultimo < primero * 3.0,
            "CRITICO: si el coste por paso creciera con la longitud de la \
             cadena, el plegado no estaria funcionando. primero={primero}us, \
             ultimo={ultimo}us"
        );
    }

    /// Verificar con un número de pasos distinto al real debe fallar.
    #[test]
    fn wrong_step_count_fails_verification() {
        let num_steps = 3usize;
        let circuits: Vec<C> = (0..num_steps).map(|_| TransactionStep::new(1000)).collect();

        let pp = PublicParams::<E1, E2, C>::setup(&circuits[0], &*S1::ck_floor(), &*S2::ck_floor())
            .expect("setup");
        let z0 = vec![<E1 as Engine>::Scalar::ZERO];
        let mut rs = RecursiveSNARK::<E1, E2, C>::new(&pp, &circuits[0], &z0).expect("new");
        for circuit in circuits.iter() {
            rs.prove_step(&pp, circuit).expect("prove_step");
        }

        assert!(
            rs.verify(&pp, num_steps + 1, &z0).is_err(),
            "CRITICO: declarar mas pasos de los plegados no deberia verificar"
        );
    }

    /// Verificar con un estado inicial distinto debe fallar.
    #[test]
    fn wrong_initial_state_fails_verification() {
        let num_steps = 3usize;
        let circuits: Vec<C> = (0..num_steps).map(|_| TransactionStep::new(1000)).collect();

        let pp = PublicParams::<E1, E2, C>::setup(&circuits[0], &*S1::ck_floor(), &*S2::ck_floor())
            .expect("setup");
        let z0 = vec![<E1 as Engine>::Scalar::ZERO];
        let mut rs = RecursiveSNARK::<E1, E2, C>::new(&pp, &circuits[0], &z0).expect("new");
        for circuit in circuits.iter() {
            rs.prove_step(&pp, circuit).expect("prove_step");
        }

        let wrong_z0 = vec![<E1 as Engine>::Scalar::from(999u64)];
        assert!(
            rs.verify(&pp, num_steps, &wrong_z0).is_err(),
            "CRITICO: un estado inicial distinto no deberia verificar"
        );
    }
}
