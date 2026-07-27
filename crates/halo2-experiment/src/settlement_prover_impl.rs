//! Implementación de `SettlementProver` para el backend Halo2 (IPA).
//! Envuelve exactamente el mismo patrón ya verificado en
//! `compliance_real_proof.rs` — no añade lógica criptográfica nueva.
//!
//! ## Por qué `ProvingKey`/`VerifyingKey` incluyen `Params`
//!
//! A diferencia de Groth16 (donde el setup produce directamente unas
//! claves autocontenidas), en Halo2 los parámetros IPA (`Params<EqAffine>`)
//! se necesitan por separado en CADA llamada a `create_proof`/
//! `verify_proof`, no solo en el `keygen`. Para que este backend encaje
//! en la forma del trait (`prove`/`verify` solo reciben la clave, no los
//! parámetros aparte), se empaquetan juntos en `Halo2ProvingKey`/
//! `Halo2VerifyingKey`. Esto es una decisión de diseño explícita, no un
//! detalle escondido.
//!
//! ## Tamaño de circuito fijo
//!
//! `setup()` usa `k = 15`, el mismo tamaño ya validado en
//! `compliance_real_proof.rs`. Una versión de producción real
//! necesitaría exponer `k` como parámetro — se fija aquí para mantener
//! el trait mínimo, no por limitación técnica.

use crate::compliance_circuit::ComplianceCircuit;
use halo2_proofs::pasta::{EqAffine, Fp};
use halo2_proofs::plonk::{
    create_proof, keygen_pk, keygen_vk, verify_proof, ProvingKey, SingleVerifier, VerifyingKey,
};
use halo2_proofs::poly::commitment::Params;
use halo2_proofs::transcript::{Blake2bRead, Blake2bWrite, Challenge255};
use rand_core::OsRng;
use settlement_prover::SettlementProver;

const K: u32 = 15;

/// Marcador (sin estado propio) para seleccionar el backend Halo2 a
/// través del trait `SettlementProver`.
pub struct Halo2Backend;

pub struct Halo2ProvingKey {
    params: Params<EqAffine>,
    pk: ProvingKey<EqAffine>,
}

pub struct Halo2VerifyingKey {
    params: Params<EqAffine>,
    vk: VerifyingKey<EqAffine>,
}

#[derive(Debug)]
pub struct Halo2Error(String);

impl std::fmt::Display for Halo2Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error del backend Halo2: {}", self.0)
    }
}
impl std::error::Error for Halo2Error {}

impl SettlementProver for Halo2Backend {
    /// El circuito de Halo2 NO calcula/expone `state_root`/`nullifier`
    /// por sí mismo (a diferencia del de Groth16 — ver la nota de
    /// asimetría en `settlement-prover::lib`), así que el testigo incluye
    /// el input público ya calculado nativamente por el llamador.
    type Witness = (ComplianceCircuit, Vec<Fp>);
    type PublicInput = Vec<Fp>;
    type Proof = Vec<u8>;
    type ProvingKey = Halo2ProvingKey;
    type VerifyingKey = Halo2VerifyingKey;
    type Error = Halo2Error;

    fn setup(_rng_seed: u64) -> Result<(Self::ProvingKey, Self::VerifyingKey), Self::Error> {
        // El setup IPA es determinista: no usa aleatoriedad, así que
        // `_rng_seed` no se usa aquí (a diferencia del backend Groth16).
        let params: Params<EqAffine> = Params::new(K);
        let empty_circuit = ComplianceCircuit::default();

        let vk = keygen_vk(&params, &empty_circuit)
            .map_err(|e| Halo2Error(format!("keygen_vk: {e:?}")))?;
        let pk = keygen_pk(&params, vk.clone(), &empty_circuit)
            .map_err(|e| Halo2Error(format!("keygen_pk: {e:?}")))?;

        Ok((
            Halo2ProvingKey { params: params.clone(), pk },
            Halo2VerifyingKey { params, vk },
        ))
    }

    fn prove(
        pk: &Self::ProvingKey,
        witness: Self::Witness,
        _rng_seed: u64,
    ) -> Result<(Self::Proof, Self::PublicInput), Self::Error> {
        let (circuit, public_input) = witness;

        let mut transcript = Blake2bWrite::<_, EqAffine, Challenge255<_>>::init(vec![]);
        create_proof(
            &pk.params,
            &pk.pk,
            &[circuit],
            &[&[&public_input]],
            OsRng,
            &mut transcript,
        )
        .map_err(|e| Halo2Error(format!("create_proof: {e:?}")))?;
        let proof = transcript.finalize();

        Ok((proof, public_input))
    }

    fn verify(
        vk: &Self::VerifyingKey,
        public_input: &Self::PublicInput,
        proof: &Self::Proof,
    ) -> Result<bool, Self::Error> {
        let strategy = SingleVerifier::new(&vk.params);
        let mut transcript_reader = Blake2bRead::<_, EqAffine, Challenge255<_>>::init(&proof[..]);

        match verify_proof(&vk.params, &vk.vk, strategy, &[&[public_input.as_slice()]], &mut transcript_reader) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compliance_circuit::{NULLIFIER_DOMAIN, TREE_DEPTH};
    use halo2_gadgets::poseidon::primitives::{ConstantLength, Hash as PoseidonHashPrimitive, P128Pow5T3};
    use halo2_proofs::circuit::Value;

    fn native_hash(a: Fp, b: Fp) -> Fp {
        PoseidonHashPrimitive::<Fp, P128Pow5T3, ConstantLength<2>, 3, 2>::init().hash([a, b])
    }
    fn native_leaf(account_id: Fp, balance: Fp, nonce: Fp) -> Fp {
        native_hash(native_hash(account_id, balance), nonce)
    }
    fn native_nullifier(account_id: Fp, nonce: Fp) -> Fp {
        native_hash(native_hash(Fp::from(NULLIFIER_DOMAIN), account_id), nonce)
    }

    struct NativeTree {
        levels: Vec<Vec<Fp>>,
    }
    impl NativeTree {
        fn build(mut leaves: Vec<Fp>) -> Self {
            let target = 1usize << TREE_DEPTH;
            leaves.resize(target, Fp::zero());
            let mut levels = vec![leaves];
            for _ in 0..TREE_DEPTH {
                let prev = levels.last().unwrap();
                let next: Vec<Fp> = prev.chunks(2).map(|p| native_hash(p[0], p[1])).collect();
                levels.push(next);
            }
            Self { levels }
        }
        fn root(&self) -> Fp {
            self.levels.last().unwrap()[0]
        }
        fn path_for(&self, index: usize) -> (Vec<Fp>, Vec<bool>) {
            let mut siblings = Vec::with_capacity(TREE_DEPTH);
            let mut is_right = Vec::with_capacity(TREE_DEPTH);
            let mut idx = index;
            for level in 0..TREE_DEPTH {
                siblings.push(self.levels[level][idx ^ 1]);
                is_right.push(idx % 2 == 1);
                idx /= 2;
            }
            (siblings, is_right)
        }
    }

    /// EL TEST CLAVE: el mismo flujo setup/prove/verify que ya
    /// verificamos directamente en `compliance_real_proof.rs`, ahora
    /// pasando por la interfaz genérica `SettlementProver`.
    ///
    /// AVISO DE TIEMPO: repite el ciclo completo (k=15), esperar varios
    /// minutos — mismo orden de magnitud que las rondas anteriores.
    #[test]
    fn halo2_backend_valid_transaction_via_trait() {
        let account_id = Fp::from(42u64);
        let nonce = Fp::from(1u64);
        let balance = Fp::from(1_000_000u64);
        let amount = Fp::from(250_000u64);
        let regulatory_limit = Fp::from(500_000u64);

        let leaf = native_leaf(account_id, balance, nonce);
        let mut leaves = vec![Fp::from(1), Fp::from(2), Fp::from(3), leaf];
        leaves.resize(8, Fp::zero());
        let tree = NativeTree::build(leaves);
        let root = tree.root();
        let (siblings, is_right) = tree.path_for(3);
        let nullifier = native_nullifier(account_id, nonce);

        let circuit = ComplianceCircuit {
            account_id: Value::known(account_id),
            balance: Value::known(balance),
            account_nonce: Value::known(nonce),
            amount: Value::known(amount),
            regulatory_limit: Value::known(regulatory_limit),
            siblings: siblings.into_iter().map(Value::known).collect(),
            path_bits: is_right.into_iter().map(|b| Value::known(if b { Fp::one() } else { Fp::zero() })).collect(),
        };
        let public_input = vec![root, regulatory_limit, nullifier];

        let (pk, vk) = Halo2Backend::setup(1).expect("setup no deberia fallar");
        let (proof, returned_public_input) =
            Halo2Backend::prove(&pk, (circuit, public_input), 2).expect("prove no deberia fallar");
        let is_valid = Halo2Backend::verify(&vk, &returned_public_input, &proof)
            .expect("verify no deberia devolver error");

        assert!(is_valid, "una transaccion valida debe verificar como verdadera via el trait");
    }
}
