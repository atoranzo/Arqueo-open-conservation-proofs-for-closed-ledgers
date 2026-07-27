//! Circuito de **partida doble** en PLONK-KZG — equivalente del
//! `zk-core::circuit_double_entry`, del
//! `halo2-experiment::circuit_double_entry` y del
//! `stark-experiment::double_entry`.
//!
//! Demuestra la transición de estado completa de una transferencia,
//! conservando el dinero:
//!
//! ```text
//! saldo_emisor_nuevo   = saldo_emisor   - importe   (ADEUDO)
//! saldo_receptor_nuevo = saldo_receptor + importe   (ABONO)
//! ```
//!
//! ## Por qué aquí NO hace falta el diseño en lockstep
//!
//! El port a STARK reveló que AIR carece de restricciones de copia: nada
//! obliga a que las dos subidas del árbol (hoja antigua y hoja nueva)
//! usen los mismos hermanos, y eso abre un agujero silencioso. Allí hubo
//! que rediseñar con dos carriles en lockstep.
//!
//! **PLONK es Plonkish y sí tiene restricciones de copia**: reutilizar
//! los mismos `Witness` de hermanos en ambas subidas es sólido por
//! construcción, exactamente igual que en Halo2. El problema simplemente
//! no existe en este paradigma.
//!
//! Es una ilustración concreta de que la elección de aritmetización tiene
//! consecuencias de diseño que van mucho más allá del rendimiento.
//!
//! ## La secuencia, y por qué el orden importa
//!
//! 1. Hoja del emisor → verificar contra `root_old`.
//! 2. Adeudo, nonce+1, recalcular el camino → `root_mid`.
//! 3. Hoja del receptor → verificar contra **`root_mid`**, no contra
//!    `root_old`: el árbol ya cambió.
//! 4. Abono, recalcular → `root_new`.
//!
//! ## Inputs públicos, EN ESTE ORDEN
//!
//! | posición | valor |
//! |---|---|
//! | 0 | `root_old` |
//! | 1 | `root_new` |
//! | 2 | `regulatory_limit` |
//! | 3 | `nullifier` |

use dusk_plonk::prelude::*;

use crate::merkle::{gadget_climb, native_climb, MerklePath};
use crate::poseidon_hash::{gadget_leaf, gadget_nullifier, native_leaf, native_nullifier};

const RANGE_BIT_PAIRS: usize = 32; // 64 bits

/// Tamaño del SRS. Cuatro subidas del árbol (~20.000 puertas cada una)
/// más hojas, nullifier y range checks.
pub const CAPACITY: usize = 1 << 17;

/// Testigos de una de las dos partes.
#[derive(Clone, Debug, Default)]
pub struct PartyWitness {
    pub account_id: BlsScalar,
    pub balance: BlsScalar,
    pub nonce: BlsScalar,
    pub path: MerklePath,
}

#[derive(Default, Debug)]
pub struct DoubleEntryCircuit {
    pub sender: PartyWitness,
    pub receiver: PartyWitness,
    pub amount: BlsScalar,

    pub root_old: BlsScalar,
    pub root_new: BlsScalar,
    pub regulatory_limit: BlsScalar,
    pub nullifier: BlsScalar,
}

impl Circuit for DoubleEntryCircuit {
    fn circuit(&self, composer: &mut Composer) -> Result<(), Error> {
        // --- Testigos ---
        let s_id = composer.append_witness(self.sender.account_id);
        let s_bal = composer.append_witness(self.sender.balance);
        let s_nonce = composer.append_witness(self.sender.nonce);
        let r_id = composer.append_witness(self.receiver.account_id);
        let r_bal = composer.append_witness(self.receiver.balance);
        let r_nonce = composer.append_witness(self.receiver.nonce);
        let amount = composer.append_witness(self.amount);
        let limit = composer.append_witness(self.regulatory_limit);

        let alloc_path = |composer: &mut Composer, p: &MerklePath| {
            let siblings: Vec<Witness> =
                p.siblings.iter().map(|s| composer.append_witness(*s)).collect();
            let bits: Vec<Witness> = p
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
            (siblings, bits)
        };
        let (s_siblings, s_bits) = alloc_path(composer, &self.sender.path);
        let (r_siblings, r_bits) = alloc_path(composer, &self.receiver.path);

        // ===== 1. El emisor está en el árbol ANTIGUO =====
        let s_leaf_old = gadget_leaf(composer, s_id, s_bal, s_nonce);
        let computed_old = gadget_climb(composer, s_leaf_old, &s_siblings, &s_bits);
        let w_root_old = composer.append_public(self.root_old);
        composer.assert_equal(computed_old, w_root_old);

        // ===== 2. ADEUDO =====
        let s_bal_new = composer.gate_add(
            Constraint::new()
                .left(1)
                .right(-BlsScalar::one())
                .a(s_bal)
                .b(amount),
        );
        let s_nonce_new =
            composer.gate_add(Constraint::new().left(1).constant(1).a(s_nonce));

        let s_leaf_new = gadget_leaf(composer, s_id, s_bal_new, s_nonce_new);
        // MISMOS testigos de hermanos y bits: las restricciones de copia
        // de PLONK garantizan que es el mismo camino, sin necesidad del
        // diseño en lockstep que exigió el backend STARK.
        let root_mid = gadget_climb(composer, s_leaf_new, &s_siblings, &s_bits);

        // ===== 3. El receptor está en el árbol INTERMEDIO =====
        let r_leaf_old = gadget_leaf(composer, r_id, r_bal, r_nonce);
        let computed_mid = gadget_climb(composer, r_leaf_old, &r_siblings, &r_bits);
        composer.assert_equal(computed_mid, root_mid);

        // ===== 4. ABONO: el MISMO importe =====
        let r_bal_new =
            composer.gate_add(Constraint::new().left(1).right(1).a(r_bal).b(amount));
        let r_leaf_new = gadget_leaf(composer, r_id, r_bal_new, r_nonce);
        let computed_new = gadget_climb(composer, r_leaf_new, &r_siblings, &r_bits);
        let w_root_new = composer.append_public(self.root_new);
        composer.assert_equal(computed_new, w_root_new);

        // ===== 5. Límite público y nullifier =====
        let w_limit_public = composer.append_public(self.regulatory_limit);
        composer.assert_equal(limit, w_limit_public);

        let computed_null = gadget_nullifier(composer, s_id, s_nonce);
        let w_null = composer.append_public(self.nullifier);
        composer.assert_equal(computed_null, w_null);

        // ===== 6. Solvencia, límite y no desbordamiento =====
        composer.component_range::<RANGE_BIT_PAIRS>(s_bal);
        composer.component_range::<RANGE_BIT_PAIRS>(r_bal);
        composer.component_range::<RANGE_BIT_PAIRS>(amount);
        composer.component_range::<RANGE_BIT_PAIRS>(limit);
        // amount <= saldo del emisor
        composer.component_range::<RANGE_BIT_PAIRS>(s_bal_new);
        // el abono no desborda
        composer.component_range::<RANGE_BIT_PAIRS>(r_bal_new);
        // amount <= limite regulatorio
        let diff_limit = composer.gate_add(
            Constraint::new()
                .left(1)
                .right(-BlsScalar::one())
                .a(limit)
                .b(amount),
        );
        composer.component_range::<RANGE_BIT_PAIRS>(diff_limit);

        Ok(())
    }
}

/// Construye un escenario coherente, con las raíces derivadas de los
/// testigos.
///
/// `credited` permite acreditar al receptor una cantidad distinta de la
/// debitada, para construir los tests que rompen la conservación. En uso
/// normal debe ser igual a `amount`.
#[allow(clippy::too_many_arguments)]
pub fn build_scenario(
    sender_id: u64,
    sender_balance: u64,
    sender_nonce: u64,
    receiver_id: u64,
    receiver_balance: u64,
    receiver_nonce: u64,
    amount: u64,
    credited: u64,
    limit: u64,
    sender_path_of: impl Fn(BlsScalar) -> MerklePath,
    receiver_path_of: impl Fn(BlsScalar) -> MerklePath,
) -> DoubleEntryCircuit {
    let s_id = BlsScalar::from(sender_id);
    let s_bal = BlsScalar::from(sender_balance);
    let s_nonce = BlsScalar::from(sender_nonce);
    let r_id = BlsScalar::from(receiver_id);
    let r_bal = BlsScalar::from(receiver_balance);
    let r_nonce = BlsScalar::from(receiver_nonce);
    let amt = BlsScalar::from(amount);

    let s_leaf_old = native_leaf(s_id, s_bal, s_nonce);
    let s_leaf_new = native_leaf(s_id, s_bal - amt, s_nonce + BlsScalar::one());
    let r_leaf_old = native_leaf(r_id, r_bal, r_nonce);
    let r_leaf_new = native_leaf(r_id, r_bal + BlsScalar::from(credited), r_nonce);

    // El camino del emisor tiene como hermano de nivel 0 la hoja ANTIGUA
    // del receptor; el del receptor, la hoja NUEVA del emisor — porque su
    // camino se toma del árbol ya actualizado.
    let sender_path = sender_path_of(r_leaf_old);
    let receiver_path = receiver_path_of(s_leaf_new);

    DoubleEntryCircuit {
        root_old: native_climb(s_leaf_old, &sender_path),
        root_new: native_climb(r_leaf_new, &receiver_path),
        regulatory_limit: BlsScalar::from(limit),
        nullifier: native_nullifier(s_id, s_nonce),
        sender: PartyWitness {
            account_id: s_id,
            balance: s_bal,
            nonce: s_nonce,
            path: sender_path,
        },
        receiver: PartyWitness {
            account_id: r_id,
            balance: r_bal,
            nonce: r_nonce,
            path: receiver_path,
        },
        amount: amt,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::merkle::test_support_paths::{sparse_path_index_0, sparse_path_index_1};
    use crate::test_support::shared_pp;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    fn scenario(balance: u64, amount: u64, credited: u64, limit: u64) -> DoubleEntryCircuit {
        // Emisor en el indice 0, receptor en el 1: hermanos en el nivel 0
        // de un arbol disperso. Asi los dos caminos son coherentes con un
        // arbol REAL, que es lo que exige el encadenamiento
        // root_old -> root_mid -> root_new.
        build_scenario(
            1001,
            balance,
            7,
            2002,
            50_000,
            3,
            amount,
            credited,
            limit,
            sparse_path_index_0,
            sparse_path_index_1,
        )
    }

    fn compile_once() -> (Prover, Verifier) {
        Compiler::compile::<DoubleEntryCircuit>(shared_pp(), b"zk-ssl-double-entry")
            .expect("la compilacion no deberia fallar")
    }

    /// EL TEST CLAVE: una transferencia válida produce prueba verificable.
    #[test]
    fn valid_transfer_verifies() {
        let mut rng = StdRng::seed_from_u64(0xdeadbeef);
        let (prover, verifier) = compile_once();

        let circuit = scenario(1_000_000, 250_000, 250_000, 500_000);
        println!("Puertas del circuito de partida doble: {}", circuit.size());

        let (proof, pi) = prover.prove(&mut rng, &circuit).expect("prove");
        verifier.verify(&proof, &pi).expect("deberia verificar");
    }

    /// EL TEST QUE DA SENTIDO A LA PIEZA: el receptor recibe 10.000 más
    /// de lo que el emisor perdió. Creación de dinero de la nada.
    #[test]
    fn money_creation_is_rejected() {
        let mut rng = StdRng::seed_from_u64(0xdeadbeef);
        let (prover, _) = compile_once();

        let circuit = scenario(1_000_000, 250_000, 260_000, 500_000);
        assert!(
            prover.prove(&mut rng, &circuit).is_err(),
            "CRITICO: acreditar mas de lo debitado debe rechazarse"
        );
    }

    /// El caso simétrico: destrucción de dinero.
    #[test]
    fn money_destruction_is_rejected() {
        let mut rng = StdRng::seed_from_u64(0xdeadbeef);
        let (prover, _) = compile_once();

        let circuit = scenario(1_000_000, 250_000, 240_000, 500_000);
        assert!(
            prover.prove(&mut rng, &circuit).is_err(),
            "CRITICO: acreditar menos de lo debitado debe rechazarse"
        );
    }

    /// Gastar más del saldo.
    #[test]
    fn insufficient_balance_is_rejected() {
        let mut rng = StdRng::seed_from_u64(0xdeadbeef);
        let (prover, _) = compile_once();

        let circuit = scenario(100_000, 250_000, 250_000, 500_000);
        assert!(
            prover.prove(&mut rng, &circuit).is_err(),
            "CRITICO: gastar mas del saldo debe rechazarse"
        );
    }

    /// Superar el límite regulatorio.
    #[test]
    fn over_regulatory_limit_is_rejected() {
        let mut rng = StdRng::seed_from_u64(0xdeadbeef);
        let (prover, _) = compile_once();

        let circuit = scenario(1_000_000, 750_000, 750_000, 500_000);
        assert!(
            prover.prove(&mut rng, &circuit).is_err(),
            "CRITICO: superar el limite debe rechazarse"
        );
    }

    /// Raíz final declarada incorrecta.
    #[test]
    fn wrong_declared_new_root_is_rejected() {
        let mut rng = StdRng::seed_from_u64(0xdeadbeef);
        let (prover, _) = compile_once();

        let mut circuit = scenario(1_000_000, 250_000, 250_000, 500_000);
        circuit.root_new = BlsScalar::from(999_999u64);
        assert!(
            prover.prove(&mut rng, &circuit).is_err(),
            "CRITICO: una raiz final incorrecta debe rechazarse"
        );
    }

    /// Nullifier falsificado.
    #[test]
    fn forged_nullifier_is_rejected() {
        let mut rng = StdRng::seed_from_u64(0xdeadbeef);
        let (prover, _) = compile_once();

        let mut circuit = scenario(1_000_000, 250_000, 250_000, 500_000);
        circuit.nullifier = BlsScalar::from(31_337u64);
        assert!(
            prover.prove(&mut rng, &circuit).is_err(),
            "CRITICO: un nullifier falsificado debe rechazarse"
        );
    }
}
