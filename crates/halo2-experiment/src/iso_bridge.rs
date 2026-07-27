//! Puente ISO 20022 → circuito de cumplimiento en Halo2, análogo a
//! `iso-bridge` (que hace lo mismo para Arkworks/Groth16 en `zk-core`).
//! Mismo alcance simplificado que el original: subconjunto de campos de
//! un mensaje pacs.008, no un parser XML completo.

use crate::compliance_circuit::ComplianceCircuit;
use halo2_gadgets::poseidon::primitives::{ConstantLength, Hash as PoseidonHashPrimitive, P128Pow5T3};
use halo2_proofs::circuit::Value;
use halo2_proofs::pasta::{EqAffine, Fp};
use halo2_proofs::plonk::{
    create_proof, verify_proof, ProvingKey, SingleVerifier, VerifyingKey,
};
use halo2_proofs::poly::commitment::Params;
use halo2_proofs::transcript::{Blake2bRead, Blake2bWrite, Challenge255};
use rand_core::OsRng;

const NULLIFIER_DOMAIN: u64 = 0x4E554C4C;

fn native_hash(a: Fp, b: Fp) -> Fp {
    PoseidonHashPrimitive::<Fp, P128Pow5T3, ConstantLength<2>, 3, 2>::init().hash([a, b])
}

fn native_nullifier(account_id: Fp, account_nonce: Fp) -> Fp {
    let domain = Fp::from(NULLIFIER_DOMAIN);
    native_hash(native_hash(domain, account_id), account_nonce)
}

/// Subconjunto simplificado de un mensaje ISO 20022 pacs.008, igual
/// alcance (y mismas limitaciones documentadas) que en `iso-bridge`.
#[derive(Debug, Clone)]
pub struct Pacs008Message {
    pub message_id: String,
    pub debtor_bic: String,
    pub creditor_bic: String,
    pub currency: String,
    pub instructed_amount_minor_units: u64,
}

#[derive(Debug)]
pub enum BridgeError {
    InvalidMessage(String),
    ProofError(String),
}

impl std::fmt::Display for BridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BridgeError::InvalidMessage(e) => write!(f, "mensaje ISO 20022 invalido: {e}"),
            BridgeError::ProofError(e) => write!(f, "error del motor de pruebas Halo2: {e}"),
        }
    }
}
impl std::error::Error for BridgeError {}

/// Paquete de liquidación con prueba Halo2/IPA real adjunta.
pub struct SovereignSettlementPackage {
    pub message_id: String,
    pub currency: String,
    pub state_root: Fp,
    pub regulatory_limit: Fp,
    pub nullifier: Fp,
    pub proof: Vec<u8>,
}

/// Traduce un mensaje pacs.008 en un paquete de liquidación con prueba
/// Halo2 real. `account_balance_minor_units` se recibe como parámetro
/// confiado (misma limitación documentada que en `iso-bridge`).
pub fn translate_and_prove(
    message: &Pacs008Message,
    account_id: Fp,
    account_balance_minor_units: u64,
    account_nonce: Fp,
    siblings: Vec<Fp>,
    path_bits: Vec<bool>,
    state_root: Fp,
    regulatory_limit_minor_units: u64,
    params: &Params<EqAffine>,
    pk: &ProvingKey<EqAffine>,
) -> Result<SovereignSettlementPackage, BridgeError> {
    if message.message_id.trim().is_empty() {
        return Err(BridgeError::InvalidMessage("message_id no puede estar vacio".into()));
    }
    if message.debtor_bic.trim().is_empty() || message.creditor_bic.trim().is_empty() {
        return Err(BridgeError::InvalidMessage(
            "debtor_bic y creditor_bic son obligatorios".into(),
        ));
    }
    if message.instructed_amount_minor_units == 0 {
        return Err(BridgeError::InvalidMessage("el importe instruido no puede ser cero".into()));
    }

    let balance = Fp::from(account_balance_minor_units);
    let amount = Fp::from(message.instructed_amount_minor_units);
    let regulatory_limit = Fp::from(regulatory_limit_minor_units);
    let nullifier = native_nullifier(account_id, account_nonce);

    let circuit = ComplianceCircuit {
        account_id: Value::known(account_id),
        balance: Value::known(balance),
        account_nonce: Value::known(account_nonce),
        amount: Value::known(amount),
        regulatory_limit: Value::known(regulatory_limit),
        siblings: siblings.into_iter().map(Value::known).collect(),
        path_bits: path_bits
            .into_iter()
            .map(|b| Value::known(if b { Fp::one() } else { Fp::zero() }))
            .collect(),
    };

    let public_input = vec![state_root, regulatory_limit, nullifier];

    let mut transcript = Blake2bWrite::<_, EqAffine, Challenge255<_>>::init(vec![]);
    create_proof(params, pk, &[circuit], &[&[&public_input]], OsRng, &mut transcript)
        .map_err(|e| BridgeError::ProofError(format!("fallo al generar la prueba: {e:?}")))?;
    let proof = transcript.finalize();

    Ok(SovereignSettlementPackage {
        message_id: message.message_id.clone(),
        currency: message.currency.clone(),
        state_root,
        regulatory_limit,
        nullifier,
        proof,
    })
}

/// Verifica el paquete de liquidación. `Ok(true)` confirma que la prueba
/// es criptográficamente válida — NO que el nullifier no se haya usado
/// antes (esa comprobación sigue siendo responsabilidad de un registro
/// aparte, igual que en la versión Groth16).
pub fn verify_package(
    package: &SovereignSettlementPackage,
    params: &Params<EqAffine>,
    vk: &VerifyingKey<EqAffine>,
) -> Result<bool, BridgeError> {
    let public_input = vec![package.state_root, package.regulatory_limit, package.nullifier];
    let strategy = SingleVerifier::new(params);
    let mut transcript_reader = Blake2bRead::<_, EqAffine, Challenge255<_>>::init(&package.proof[..]);

    match verify_proof(params, vk, strategy, &[&[&public_input]], &mut transcript_reader) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compliance_circuit::TREE_DEPTH;
    use halo2_proofs::plonk::{keygen_pk, keygen_vk};

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

    fn native_leaf(account_id: Fp, balance: Fp, nonce: Fp) -> Fp {
        native_hash(native_hash(account_id, balance), nonce)
    }

    /// EL TEST FINAL DE TODO EL EXPERIMENTO: mensaje ISO 20022 real →
    /// prueba Halo2/IPA real → verificación real. Cierra el círculo
    /// completo, igual que hicimos en la versión Groth16.
    ///
    /// AVISO DE TIEMPO: esto repite el setup/keygen/prove/verify
    /// completo (~5-10 minutos según los datos ya medidos), porque cada
    /// test necesita su propio par de claves. Es esperado, no un error.
    #[test]
    fn valid_iso_message_produces_verifiable_halo2_proof() {
        let k = 15;

        let account_id = Fp::from(12345u64);
        let account_nonce = Fp::from(1u64);
        let balance: u64 = 1_000_000;

        let leaf = native_leaf(account_id, Fp::from(balance), account_nonce);
        let mut leaves = vec![Fp::from(1), Fp::from(2), Fp::from(3), leaf];
        leaves.resize(8, Fp::zero());
        let tree = NativeTree::build(leaves);
        let root = tree.root();
        let (siblings, is_right) = tree.path_for(3);

        let params: Params<EqAffine> = Params::new(k);
        let empty_circuit = ComplianceCircuit::default();
        let vk = keygen_vk(&params, &empty_circuit).expect("keygen_vk no deberia fallar");
        let pk = keygen_pk(&params, vk.clone(), &empty_circuit).expect("keygen_pk no deberia fallar");

        let message = Pacs008Message {
            message_id: "ISO-PAC-2026-000123".to_string(),
            debtor_bic: "BKESESMMXXX".to_string(),
            creditor_bic: "CHASUS33XXX".to_string(),
            currency: "EUR".to_string(),
            instructed_amount_minor_units: 250_000,
        };

        let package = translate_and_prove(
            &message,
            account_id,
            balance,
            account_nonce,
            siblings,
            is_right,
            root,
            500_000,
            &params,
            &pk,
        )
        .expect("la generacion de la prueba no deberia fallar con datos validos");

        let is_valid =
            verify_package(&package, &params, &vk).expect("la verificacion no deberia devolver error");
        assert!(is_valid, "una transaccion ISO 20022 valida debe producir una prueba Halo2 verificable");
    }
}
