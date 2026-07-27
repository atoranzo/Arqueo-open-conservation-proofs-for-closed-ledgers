//! Circuito de cumplimiento completo en PLONK-KZG — equivalente del
//! `zk-core::circuit_with_state`, del
//! `halo2-experiment::compliance_circuit` y del
//! `stark-experiment::compliance_circuit`.
//!
//! Demuestra, sin revelar saldo ni importe:
//!
//! 1. `amount <= balance` (solvencia).
//! 2. `amount <= regulatory_limit`.
//! 3. La cuenta existe en el árbol cuya raíz es pública.
//! 4. El nullifier se deriva correctamente de la cuenta y el nonce.
//!
//! ## Lo que dusk-plonk ahorra
//!
//! `component_range::<BIT_PAIRS>` es un range check **nativo**. Según la
//! documentación del crate, cuesta `(num_bits - 1)/8 + 9` puertas: para
//! 64 bits, unas **16**. En Groth16, Halo2 y STARK hubo que construirlo a
//! mano con descomposición en bits y acumuladores, y costaba del orden
//! de 64-256 restricciones cada uno.
//!
//! Es la contrapartida a que el hash sea más caro aquí (~997 puertas por
//! hash de aridad 2, frente a ~300 en Groth16).
//!
//! ## Cómo se demuestra `a <= b` sin revelar nada
//!
//! Igual que en los otros backends: se calcula `d = b - a` en el campo y
//! se comprueba que `d` cabe en 64 bits. Si `a > b`, la resta da la
//! vuelta y produce un valor enorme que no cabe, así que el range check
//! falla. Es el mismo mecanismo, con la diferencia de que aquí el range
//! check lo aporta la librería.
//!
//! ## Inputs públicos, EN ESTE ORDEN
//!
//! | posición | valor |
//! |---|---|
//! | 0 | `state_root` |
//! | 1 | `regulatory_limit` |
//! | 2 | `nullifier` |

use dusk_plonk::prelude::*;

use crate::merkle::{gadget_climb, native_climb, MerklePath};
use crate::poseidon_hash::{gadget_leaf, gadget_nullifier, native_leaf, native_nullifier};

/// Pares de bits del range check: 32 pares = 64 bits.
const RANGE_BIT_PAIRS: usize = 32;

/// Tamaño del SRS necesario. El árbol solo ya son ~19.950 puertas.
pub const CAPACITY: usize = 1 << 16;

#[derive(Default, Debug)]
pub struct ComplianceCircuit {
    // --- Testigos privados ---
    pub account_id: BlsScalar,
    pub balance: BlsScalar,
    pub nonce: BlsScalar,
    pub amount: BlsScalar,
    pub path: MerklePath,

    // --- Entradas públicas ---
    pub state_root: BlsScalar,
    pub regulatory_limit: BlsScalar,
    pub nullifier: BlsScalar,
}

impl ComplianceCircuit {
    /// Construye un circuito coherente a partir de los testigos: los
    /// valores públicos se DERIVAN, no se reciben. Así es imposible por
    /// construcción declarar una raíz o un nullifier que no correspondan.
    pub fn new(
        account_id: u64,
        balance: u64,
        nonce: u64,
        amount: u64,
        regulatory_limit: u64,
        path: MerklePath,
    ) -> Self {
        let account_id = BlsScalar::from(account_id);
        let balance = BlsScalar::from(balance);
        let nonce = BlsScalar::from(nonce);

        let leaf = native_leaf(account_id, balance, nonce);
        Self {
            account_id,
            balance,
            nonce,
            amount: BlsScalar::from(amount),
            state_root: native_climb(leaf, &path),
            regulatory_limit: BlsScalar::from(regulatory_limit),
            nullifier: native_nullifier(account_id, nonce),
            path,
        }
    }
}

impl Circuit for ComplianceCircuit {
    fn circuit(&self, composer: &mut Composer) -> Result<(), Error> {
        // --- Testigos ---
        let w_account = composer.append_witness(self.account_id);
        let w_balance = composer.append_witness(self.balance);
        let w_nonce = composer.append_witness(self.nonce);
        let w_amount = composer.append_witness(self.amount);
        let w_limit = composer.append_witness(self.regulatory_limit);

        let siblings: Vec<Witness> = self
            .path
            .siblings
            .iter()
            .map(|s| composer.append_witness(*s))
            .collect();
        let bits: Vec<Witness> = self
            .path
            .is_right
            .iter()
            .map(|b| {
                composer.append_witness(if *b {
                    BlsScalar::one()
                } else {
                    BlsScalar::zero()
                })
            })
            .collect();

        // ===== 1. Pertenencia al árbol =====
        let leaf = gadget_leaf(composer, w_account, w_balance, w_nonce);
        let computed_root = gadget_climb(composer, leaf, &siblings, &bits);
        let w_root = composer.append_public(self.state_root);
        composer.assert_equal(computed_root, w_root);

        // ===== 2. El límite regulatorio es público =====
        let w_limit_public = composer.append_public(self.regulatory_limit);
        composer.assert_equal(w_limit, w_limit_public);

        // ===== 3. Nullifier =====
        let computed_null = gadget_nullifier(composer, w_account, w_nonce);
        let w_null = composer.append_public(self.nullifier);
        composer.assert_equal(computed_null, w_null);

        // ===== 4. Solvencia y límite =====
        // Los operandos deben estar acotados: sin esto, valores enormes
        // podrían dar la vuelta en el campo y falsear las comparaciones.
        composer.component_range::<RANGE_BIT_PAIRS>(w_balance);
        composer.component_range::<RANGE_BIT_PAIRS>(w_amount);
        composer.component_range::<RANGE_BIT_PAIRS>(w_limit);

        // amount <= balance  ⟺  (balance - amount) cabe en 64 bits.
        let diff_balance =
            composer.gate_add(Constraint::new().left(1).right(-BlsScalar::one()).a(w_balance).b(w_amount));
        composer.component_range::<RANGE_BIT_PAIRS>(diff_balance);

        // amount <= limite regulatorio.
        let diff_limit =
            composer.gate_add(Constraint::new().left(1).right(-BlsScalar::one()).a(w_limit).b(w_amount));
        composer.component_range::<RANGE_BIT_PAIRS>(diff_limit);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::merkle::test_support_paths::sparse_path_index_0;
    use crate::test_support::shared_pp;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    fn compile_once() -> (Prover, Verifier) {
        Compiler::compile::<ComplianceCircuit>(shared_pp(), b"zk-ssl-compliance")
            .expect("la compilacion no deberia fallar")
    }

    fn valid_circuit(balance: u64, amount: u64, limit: u64) -> ComplianceCircuit {
        ComplianceCircuit::new(
            42,
            balance,
            1,
            amount,
            limit,
            sparse_path_index_0(BlsScalar::from(999u64)),
        )
    }

    /// EL TEST CLAVE: una transacción válida produce una prueba
    /// verificable.
    #[test]
    fn valid_transaction_verifies() {
        let mut rng = StdRng::seed_from_u64(0xc0ffee);
        let (prover, verifier) = compile_once();

        let circuit = valid_circuit(1_000_000, 250_000, 500_000);
        println!("Puertas del circuito de cumplimiento: {}", circuit.size());

        let (proof, pi) = prover.prove(&mut rng, &circuit).expect("prove");
        verifier.verify(&proof, &pi).expect("deberia verificar");

        use dusk_bytes::Serializable;
        println!("Tamano de la prueba: {} bytes", proof.to_bytes().len());
    }

    /// SOLIDEZ: gastar más del saldo debe impedir la prueba. La resta da
    /// la vuelta en el campo y el range check la rechaza.
    #[test]
    fn insufficient_balance_fails() {
        let mut rng = StdRng::seed_from_u64(0xc0ffee);
        let (prover, _) = compile_once();

        let circuit = valid_circuit(100_000, 250_000, 500_000); // amount > balance
        assert!(
            prover.prove(&mut rng, &circuit).is_err(),
            "CRITICO: gastar mas del saldo no deberia producir prueba"
        );
    }

    /// SOLIDEZ: superar el límite regulatorio debe impedir la prueba.
    #[test]
    fn over_regulatory_limit_fails() {
        let mut rng = StdRng::seed_from_u64(0xc0ffee);
        let (prover, _) = compile_once();

        let circuit = valid_circuit(1_000_000, 750_000, 500_000); // amount > limite
        assert!(
            prover.prove(&mut rng, &circuit).is_err(),
            "CRITICO: superar el limite no deberia producir prueba"
        );
    }

    /// Caso frontera legítimo: amount == balance.
    #[test]
    fn boundary_amount_equals_balance_verifies() {
        let mut rng = StdRng::seed_from_u64(0xc0ffee);
        let (prover, verifier) = compile_once();

        let circuit = valid_circuit(250_000, 250_000, 500_000);
        let (proof, pi) = prover.prove(&mut rng, &circuit).expect("prove");
        verifier
            .verify(&proof, &pi)
            .expect("amount == balance es legitimo y deberia verificar");
    }

    /// SOLIDEZ: declarar una raíz que no corresponde debe fallar.
    #[test]
    fn wrong_declared_root_fails() {
        let mut rng = StdRng::seed_from_u64(0xc0ffee);
        let (prover, _) = compile_once();

        let mut circuit = valid_circuit(1_000_000, 250_000, 500_000);
        circuit.state_root = BlsScalar::from(999_999u64);
        assert!(
            prover.prove(&mut rng, &circuit).is_err(),
            "CRITICO: una raiz incorrecta no deberia producir prueba"
        );
    }

    /// SOLIDEZ: un nullifier falsificado debe fallar. Sin esto, el
    /// registro contra doble gasto sería esquivable.
    #[test]
    fn forged_nullifier_fails() {
        let mut rng = StdRng::seed_from_u64(0xc0ffee);
        let (prover, _) = compile_once();

        let mut circuit = valid_circuit(1_000_000, 250_000, 500_000);
        circuit.nullifier = BlsScalar::from(31_337u64);
        assert!(
            prover.prove(&mut rng, &circuit).is_err(),
            "CRITICO: un nullifier falsificado no deberia producir prueba"
        );
    }
}
