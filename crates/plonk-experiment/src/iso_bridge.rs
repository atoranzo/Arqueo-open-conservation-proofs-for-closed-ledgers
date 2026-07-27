//! Puente ISO 20022 pacs.008 → circuito PLONK-KZG, y la implementación
//! del trait `SettlementProver` para este backend.
//!
//! Mismo alcance simplificado que los otros tres puentes: un subconjunto
//! de campos de pacs.008, no un parser XML completo.
//!
//! ## Lo que la firma de la API revela de este backend
//!
//! En Groth16 hay que pasar `ProvingKey`/`VerifyingKey` de una ceremonia
//! por circuito. En Halo2, los `Params` del setup IPA. En STARK, nada
//! (solo configuración pública).
//!
//! Aquí se pasan `Prover` y `Verifier` **compilados a partir de un SRS
//! universal**. Ese SRS es la parte reutilizable: uno solo sirve para
//! todos los circuitos, y puede venir de una ceremonia pública ya
//! celebrada. La compilación por circuito, en cambio, es determinista y
//! sin secretos — no es una segunda ceremonia como la fase 2 de Groth16.
//!
//! Esa es la ventaja estructural que justificó añadir este backend.

use dusk_bytes::Serializable;
use dusk_plonk::prelude::*;
use settlement_prover::SettlementProver;

use crate::compliance_circuit::{ComplianceCircuit, CAPACITY};
use crate::merkle::MerklePath;

/// Subconjunto simplificado de un mensaje ISO 20022 pacs.008.
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
            BridgeError::ProofError(e) => write!(f, "error del motor PLONK-KZG: {e}"),
        }
    }
}
impl std::error::Error for BridgeError {}

/// Paquete de liquidación con prueba PLONK-KZG adjunta.
pub struct SovereignSettlementPackage {
    pub message_id: String,
    pub currency: String,
    pub state_root: BlsScalar,
    pub regulatory_limit: BlsScalar,
    pub nullifier: BlsScalar,
    /// Prueba serializada. Tamaño CONSTANTE: 1.008 bytes.
    pub proof: Vec<u8>,
}

fn validate(message: &Pacs008Message) -> Result<(), BridgeError> {
    if message.message_id.trim().is_empty() {
        return Err(BridgeError::InvalidMessage(
            "message_id no puede estar vacio".into(),
        ));
    }
    if message.debtor_bic.trim().is_empty() || message.creditor_bic.trim().is_empty() {
        return Err(BridgeError::InvalidMessage(
            "debtor_bic y creditor_bic son obligatorios".into(),
        ));
    }
    if message.instructed_amount_minor_units == 0 {
        return Err(BridgeError::InvalidMessage(
            "el importe instruido no puede ser cero".into(),
        ));
    }
    Ok(())
}

/// Traduce un mensaje pacs.008 en un paquete de liquidación con prueba
/// PLONK-KZG real.
#[allow(clippy::too_many_arguments)]
pub fn translate_and_prove<R: rand::RngCore + rand::CryptoRng>(
    message: &Pacs008Message,
    prover: &Prover,
    account_id: u64,
    account_balance_minor_units: u64,
    account_nonce: u64,
    regulatory_limit_minor_units: u64,
    path: MerklePath,
    rng: &mut R,
) -> Result<SovereignSettlementPackage, BridgeError> {
    validate(message)?;

    let circuit = ComplianceCircuit::new(
        account_id,
        account_balance_minor_units,
        account_nonce,
        message.instructed_amount_minor_units,
        regulatory_limit_minor_units,
        path,
    );
    let state_root = circuit.state_root;
    let regulatory_limit = circuit.regulatory_limit;
    let nullifier = circuit.nullifier;

    let (proof, _) = prover
        .prove(rng, &circuit)
        .map_err(|e| BridgeError::ProofError(format!("fallo al generar la prueba: {e:?}")))?;

    Ok(SovereignSettlementPackage {
        message_id: message.message_id.clone(),
        currency: message.currency.clone(),
        state_root,
        regulatory_limit,
        nullifier,
        proof: proof.to_bytes().to_vec(),
    })
}

/// Verifica el paquete desde sus BYTES, como haría un nodo receptor.
///
/// `Ok(true)` confirma validez criptográfica, **no** que el nullifier
/// esté sin gastar — eso corresponde al registro persistente.
pub fn verify_package(
    package: &SovereignSettlementPackage,
    verifier: &Verifier,
) -> Result<bool, BridgeError> {
    let bytes: [u8; Proof::SIZE] = package
        .proof
        .as_slice()
        .try_into()
        .map_err(|_| BridgeError::ProofError("la prueba no tiene el tamano esperado".into()))?;
    let proof = Proof::from_bytes(&bytes)
        .map_err(|e| BridgeError::ProofError(format!("prueba mal formada: {e:?}")))?;

    let public_inputs = vec![
        package.state_root,
        package.regulatory_limit,
        package.nullifier,
    ];

    Ok(verifier.verify(&proof, &public_inputs).is_ok())
}

// ---------------------------------------------------------------------
// SettlementProver
// ---------------------------------------------------------------------

/// Marcador para seleccionar el backend PLONK-KZG a través del trait.
pub struct PlonkBackend;

/// Testigo completo. Todos los valores son PRIVADOS.
#[derive(Clone, Debug)]
pub struct PlonkWitness {
    pub account_id: u64,
    pub balance_minor_units: u64,
    pub nonce: u64,
    pub amount_minor_units: u64,
    pub regulatory_limit_minor_units: u64,
    pub path: MerklePath,
}

#[derive(Debug)]
pub struct PlonkError(String);

impl std::fmt::Display for PlonkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error del backend PLONK-KZG: {}", self.0)
    }
}
impl std::error::Error for PlonkError {}

impl SettlementProver for PlonkBackend {
    type Witness = PlonkWitness;
    type PublicInput = Vec<BlsScalar>;
    type Proof = Vec<u8>;
    /// El `Prover` compilado. Se deriva del SRS universal, que es lo
    /// reutilizable; esta compilación es determinista y sin secretos.
    type ProvingKey = Prover;
    type VerifyingKey = Verifier;
    type Error = PlonkError;

    fn setup(rng_seed: u64) -> Result<(Self::ProvingKey, Self::VerifyingKey), Self::Error> {
        use rand::rngs::StdRng;
        use rand::SeedableRng;

        // ⚠️ SRS generado por UNA SOLA PARTE. En producción debe cargarse
        // el de una ceremonia real: dusk publicó la suya sobre BLS12-381,
        // con herramienta de conversión al formato `PublicParameters`.
        let mut rng = StdRng::seed_from_u64(rng_seed);
        let pp = PublicParameters::setup(CAPACITY, &mut rng)
            .map_err(|e| PlonkError(format!("setup del SRS: {e:?}")))?;

        Compiler::compile::<ComplianceCircuit>(&pp, b"zk-ssl-settlement")
            .map_err(|e| PlonkError(format!("compilacion del circuito: {e:?}")))
    }

    fn prove(
        pk: &Self::ProvingKey,
        witness: Self::Witness,
        rng_seed: u64,
    ) -> Result<(Self::Proof, Self::PublicInput), Self::Error> {
        use rand::rngs::StdRng;
        use rand::SeedableRng;
        let mut rng = StdRng::seed_from_u64(rng_seed);

        // Los inputs públicos se DERIVAN del testigo, no se reciben: así
        // es imposible declarar valores que no correspondan.
        let circuit = ComplianceCircuit::new(
            witness.account_id,
            witness.balance_minor_units,
            witness.nonce,
            witness.amount_minor_units,
            witness.regulatory_limit_minor_units,
            witness.path,
        );

        let (proof, public_inputs) = pk
            .prove(&mut rng, &circuit)
            .map_err(|e| PlonkError(format!("fallo al generar la prueba: {e:?}")))?;

        Ok((proof.to_bytes().to_vec(), public_inputs))
    }

    fn verify(
        vk: &Self::VerifyingKey,
        public_input: &Self::PublicInput,
        proof: &Self::Proof,
    ) -> Result<bool, Self::Error> {
        let bytes: [u8; Proof::SIZE] = proof
            .as_slice()
            .try_into()
            .map_err(|_| PlonkError("la prueba no tiene el tamano esperado".into()))?;
        let parsed = Proof::from_bytes(&bytes)
            .map_err(|e| PlonkError(format!("prueba mal formada: {e:?}")))?;

        Ok(vk.verify(&parsed, public_input).is_ok())
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
        Compiler::compile::<ComplianceCircuit>(shared_pp(), b"zk-ssl-iso")
            .expect("la compilacion no deberia fallar")
    }

    fn sample_message(amount: u64) -> Pacs008Message {
        Pacs008Message {
            message_id: "MSG-2026-0001".into(),
            debtor_bic: "BBVAESMMXXX".into(),
            creditor_bic: "CAIXESBBXXX".into(),
            currency: "EUR".into(),
            instructed_amount_minor_units: amount,
        }
    }

    /// EL TEST QUE CIERRA EL CÍRCULO: mensaje ISO 20022 → prueba
    /// PLONK-KZG → bytes → verificación.
    #[test]
    fn valid_iso_message_produces_verifiable_proof() {
        let mut rng = StdRng::seed_from_u64(0x150);
        let (prover, verifier) = compile_once();

        let package = translate_and_prove(
            &sample_message(250_000),
            &prover,
            42,
            1_000_000,
            1,
            500_000,
            sparse_path_index_0(BlsScalar::from(999u64)),
            &mut rng,
        )
        .expect("un mensaje valido deberia producir paquete");

        assert_eq!(package.message_id, "MSG-2026-0001");
        assert_eq!(package.proof.len(), 1008, "el tamano de prueba es constante");

        assert!(
            verify_package(&package, &verifier).expect("verificacion"),
            "el paquete generado deberia verificar"
        );
    }

    /// Manipular el paquete en tránsito debe hacer fallar la
    /// verificación — el escenario del intermediario malicioso.
    #[test]
    fn tampered_package_fails_verification() {
        let mut rng = StdRng::seed_from_u64(0x150);
        let (prover, verifier) = compile_once();

        let mut package = translate_and_prove(
            &sample_message(250_000),
            &prover,
            42,
            1_000_000,
            1,
            500_000,
            sparse_path_index_0(BlsScalar::from(999u64)),
            &mut rng,
        )
        .expect("paquete valido");

        package.state_root = BlsScalar::from(999_999u64);
        assert!(
            !verify_package(&package, &verifier).expect("verificacion"),
            "CRITICO: un paquete manipulado no deberia verificar"
        );
    }

    /// Los mensajes mal formados se rechazan antes de tocar el motor.
    #[test]
    fn malformed_messages_are_rejected_before_proving() {
        let mut rng = StdRng::seed_from_u64(0x150);
        let (prover, _) = compile_once();

        let cases = vec![
            Pacs008Message {
                message_id: "   ".into(),
                ..sample_message(1000)
            },
            Pacs008Message {
                debtor_bic: "".into(),
                ..sample_message(1000)
            },
            sample_message(0),
        ];

        for message in cases {
            let r = translate_and_prove(
                &message,
                &prover,
                42,
                1_000_000,
                1,
                500_000,
                sparse_path_index_0(BlsScalar::from(999u64)),
                &mut rng,
            );
            assert!(matches!(r, Err(BridgeError::InvalidMessage(_))));
        }
    }

    /// El backend a través del trait genérico `SettlementProver`.
    ///
    /// Usa el SRS compartido en vez de `setup()`, que generaría uno
    /// nuevo y tardaría minutos.
    #[test]
    fn plonk_backend_valid_transaction_via_trait() {
        let (pk, vk) = compile_once();

        let witness = PlonkWitness {
            account_id: 42,
            balance_minor_units: 1_000_000,
            nonce: 1,
            amount_minor_units: 250_000,
            regulatory_limit_minor_units: 500_000,
            path: sparse_path_index_0(BlsScalar::from(999u64)),
        };

        let (proof, pi) = PlonkBackend::prove(&pk, witness, 7).expect("prove via trait");
        assert!(
            PlonkBackend::verify(&vk, &pi, &proof).expect("verify via trait"),
            "una transaccion valida debe verificar via el trait"
        );
    }
}
