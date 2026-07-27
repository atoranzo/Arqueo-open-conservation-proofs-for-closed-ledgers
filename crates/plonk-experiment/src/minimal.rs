//! Circuito mínimo PLONK-KZG: demuestra conocimiento de dos valores
//! privados `a` y `b` tales que `a + b` es igual a un valor público.
//!
//! Es el equivalente de `x³+42` en STARK o de `SquareCircuit` en Halo2:
//! la pieza más simple posible para validar el ciclo completo
//! `compile → prove → verify` antes de tocar nada de lógica de
//! cumplimiento.
//!
//! ## El modelo de dusk-plonk, en una nota
//!
//! Una puerta PLONK tiene la forma
//! `q_m·a·b + q_l·a + q_r·b + q_o·c + q_4·d + q_c + PI = 0`.
//! `Constraint` es el constructor de esos coeficientes: `.mult()` fija
//! `q_m`, `.left()` fija `q_l`, `.right()` fija `q_r`, etc., y `.a()`,
//! `.b()`, `.d()` asignan los testigos a las posiciones.
//!
//! `gate_add` devuelve el testigo de salida que satisface la puerta.

use dusk_plonk::prelude::*;

/// Tamaño del SRS para este circuito. Con un circuito tan pequeño basta
/// de sobra; los circuitos reales necesitarán bastante más.
pub const CAPACITY: usize = 1 << 8;

#[derive(Default, Debug)]
pub struct MinimalCircuit {
    /// Sumando privado.
    pub a: BlsScalar,
    /// Sumando privado.
    pub b: BlsScalar,
    /// Suma, que se expone como entrada pública.
    pub sum: BlsScalar,
}

impl MinimalCircuit {
    pub fn new(a: u64, b: u64) -> Self {
        Self {
            a: BlsScalar::from(a),
            b: BlsScalar::from(b),
            sum: BlsScalar::from(a + b),
        }
    }
}

impl Circuit for MinimalCircuit {
    fn circuit(&self, composer: &mut Composer) -> Result<(), Error> {
        let w_a = composer.append_witness(self.a);
        let w_b = composer.append_witness(self.b);

        // Puerta de suma: con q_l = q_r = 1 y solo a, b asignados, la
        // salida es a + b.
        let constraint = Constraint::new().left(1).right(1).a(w_a).b(w_b);
        let w_sum = composer.gate_add(constraint);

        // La suma se declara pública y se ata al resultado calculado.
        let w_public = composer.append_public(self.sum);
        composer.assert_equal(w_sum, w_public);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dusk_bytes::Serializable;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    /// EL TEST CLAVE: valida el ciclo completo del backend nuevo.
    ///
    /// Si esto pasa, la maquinaria de dusk-plonk (SRS, compilación,
    /// generación y verificación) funciona, y podemos empezar a portar
    /// las piezas reales del circuito de cumplimiento.
    #[test]
    fn minimal_circuit_full_cycle() {
        let mut rng = StdRng::seed_from_u64(0xdecafbad);

        // El setup de un solo participante NO debe usarse en producción;
        // aquí sirve para validar el mecanismo. La ceremonia real es el
        // siguiente paso (ver la nota de cabecera de `lib.rs`).
        let pp = PublicParameters::setup(CAPACITY, &mut rng)
            .expect("el setup del SRS no deberia fallar");

        let (prover, verifier) = Compiler::compile::<MinimalCircuit>(&pp, b"zk-ssl-minimal")
            .expect("la compilacion del circuito no deberia fallar");

        let circuit = MinimalCircuit::new(300_000, 200_000);
        let (proof, public_inputs) = prover
            .prove(&mut rng, &circuit)
            .expect("la generacion de la prueba no deberia fallar");

        println!("Inputs publicos devueltos: {}", public_inputs.len());
        println!("Tamano de la prueba: {} bytes", proof.to_bytes().len());

        verifier
            .verify(&proof, &public_inputs)
            .expect("una prueba valida deberia verificar");
    }

    /// Declarar una suma pública distinta a la real debe hacer fallar la
    /// verificación.
    #[test]
    fn wrong_declared_sum_fails_verification() {
        let mut rng = StdRng::seed_from_u64(0xdecafbad);
        let pp = PublicParameters::setup(CAPACITY, &mut rng).expect("setup");
        let (prover, verifier) =
            Compiler::compile::<MinimalCircuit>(&pp, b"zk-ssl-minimal").expect("compile");

        let circuit = MinimalCircuit::new(300_000, 200_000);
        let (proof, _) = prover.prove(&mut rng, &circuit).expect("prove");

        // Se declara una suma que no corresponde.
        let forged = vec![BlsScalar::from(999_999u64)];
        assert!(
            verifier.verify(&proof, &forged).is_err(),
            "CRITICO: una suma publica falsificada no deberia verificar"
        );
    }

    /// Un testigo que no satisface el circuito no debe producir prueba.
    /// Aquí `sum` no es `a + b`, así que `assert_equal` falla.
    #[test]
    fn unsatisfied_witness_fails_to_prove() {
        let mut rng = StdRng::seed_from_u64(0xdecafbad);
        let pp = PublicParameters::setup(CAPACITY, &mut rng).expect("setup");
        let (prover, _) =
            Compiler::compile::<MinimalCircuit>(&pp, b"zk-ssl-minimal").expect("compile");

        let broken = MinimalCircuit {
            a: BlsScalar::from(1u64),
            b: BlsScalar::from(1u64),
            sum: BlsScalar::from(999u64), // 1 + 1 != 999
        };

        assert!(
            prover.prove(&mut rng, &broken).is_err(),
            "CRITICO: un testigo que no satisface el circuito no deberia producir prueba"
        );
    }
}
