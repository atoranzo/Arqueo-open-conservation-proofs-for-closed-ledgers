//! Puente ISO 20022 → circuito de cumplimiento STARK, análogo a
//! `iso-bridge` (Groth16) y `halo2-experiment::iso_bridge` (Halo2).
//! Mismo alcance simplificado que los originales: subconjunto de campos
//! de un mensaje pacs.008, no un parser XML completo.
//!
//! ## La diferencia práctica con los otros dos puentes
//!
//! Ni `translate_and_prove` ni `verify_package` reciben claves. En los
//! otros dos backends hay que pasar `ProvingKey`/`VerifyingKey` (y en
//! Halo2 además los `Params` del setup IPA); aquí basta con la
//! configuración de `ProofOptions`, que es pública y no secreta. Es la
//! ausencia de trusted setup manifestándose en la propia firma de las
//! funciones, no solo en la teoría.
//!
//! ## Limitaciones honestas (idénticas a las de los otros puentes)
//!
//! - **No es un parser XML de ISO 20022.** Es una struct con un
//!   subconjunto de campos de pacs.008. Un puente de producción
//!   necesitaría validación de esquema real, manejo de espacios de
//!   nombres y los cientos de campos opcionales del estándar.
//! - **El saldo llega como parámetro confiado.** El circuito demuestra
//!   que el saldo declarado está en el árbol, pero quién alimenta ese
//!   dato al puente queda fuera del alcance de la prueba.
//! - **`verify_package` confirma validez criptográfica, NO frescura.**
//!   Que el nullifier no se haya gastado antes es responsabilidad del
//!   registro persistente, no de esta función.

use winterfell::crypto::{hashers::Blake3_256, DefaultRandomCoin, MerkleTree};
use winterfell::math::fields::f64::BaseElement;
use winterfell::{
    verify, AcceptableOptions, BatchingMethod, FieldExtension, Proof, ProofOptions, Prover,
};

use crate::compliance_circuit::{
    build_trace, native_leaf, native_nullifier, native_root, ComplianceAir, ComplianceProver,
    CompliancePublicInputs,
};
use crate::merkle::{Digest, MerklePath};

type Blake3 = Blake3_256<BaseElement>;

/// Configuración por defecto del puente: la que alcanza **128 bits de
/// seguridad DEMOSTRABLE** (no solo conjeturada), medida en
/// `compliance_real_proof`. Cuesta ~125 KB y ~45 ms — se elige la
/// garantía fuerte por defecto, y quien quiera pruebas más pequeñas debe
/// pedirlo explícitamente y saber qué está cambiando.
pub fn default_proof_options() -> ProofOptions {
    ProofOptions::new(
        120,
        16,
        20,
        FieldExtension::Cubic,
        8,
        31,
        BatchingMethod::Linear,
        BatchingMethod::Linear,
    )
}

/// Subconjunto simplificado de un mensaje ISO 20022 pacs.008, mismo
/// alcance que en los otros dos puentes.
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
            BridgeError::ProofError(e) => write!(f, "error del motor de pruebas STARK: {e}"),
        }
    }
}
impl std::error::Error for BridgeError {}

/// Paquete de liquidación con prueba STARK real adjunta.
pub struct SovereignSettlementPackage {
    pub message_id: String,
    pub currency: String,
    pub state_root: Digest,
    pub regulatory_limit: BaseElement,
    pub nullifier: Digest,
    /// Prueba serializada, lista para viajar por el bus de mensajería.
    pub proof: Vec<u8>,
}

/// Traduce un mensaje pacs.008 en un paquete de liquidación con prueba
/// STARK real. Sin claves: no hay trusted setup que gestionar.
#[allow(clippy::too_many_arguments)]
pub fn translate_and_prove(
    message: &Pacs008Message,
    account_id: BaseElement,
    account_balance_minor_units: u64,
    account_nonce: BaseElement,
    path: &MerklePath,
    regulatory_limit_minor_units: u64,
    options: ProofOptions,
) -> Result<SovereignSettlementPackage, BridgeError> {
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

    let amount = message.instructed_amount_minor_units;
    let leaf = native_leaf(
        account_id,
        BaseElement::new(account_balance_minor_units),
        account_nonce,
    );
    let state_root = native_root(leaf, path);
    let nullifier = native_nullifier(account_id, account_nonce);
    let regulatory_limit = BaseElement::new(regulatory_limit_minor_units);

    let trace = build_trace(
        account_id,
        account_balance_minor_units,
        account_nonce,
        amount,
        regulatory_limit_minor_units,
        path,
    );

    let prover = ComplianceProver::new(options);
    let proof = prover
        .prove(trace)
        .map_err(|e| BridgeError::ProofError(format!("fallo al generar la prueba: {e:?}")))?;

    Ok(SovereignSettlementPackage {
        message_id: message.message_id.clone(),
        currency: message.currency.clone(),
        state_root,
        regulatory_limit,
        nullifier,
        proof: proof.to_bytes(),
    })
}

/// Verifica el paquete de liquidación desde sus BYTES, como haría un nodo
/// receptor. `Ok(true)` confirma que la prueba es criptográficamente
/// válida — NO que el nullifier esté sin gastar (eso corresponde al
/// registro persistente).
pub fn verify_package(
    package: &SovereignSettlementPackage,
    options: ProofOptions,
) -> Result<bool, BridgeError> {
    let proof = Proof::from_bytes(&package.proof)
        .map_err(|e| BridgeError::ProofError(format!("prueba mal formada: {e:?}")))?;

    let public_inputs = CompliancePublicInputs {
        state_root: package.state_root,
        regulatory_limit: package.regulatory_limit,
        nullifier: package.nullifier,
    };

    let acceptable = AcceptableOptions::OptionSet(vec![options]);
    match verify::<ComplianceAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
        proof,
        public_inputs,
        &acceptable,
    ) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compliance_circuit::TREE_DEPTH;

    fn digest_from(n: u64) -> Digest {
        [
            BaseElement::new(n),
            BaseElement::new(n + 1),
            BaseElement::new(n + 2),
            BaseElement::new(n + 3),
        ]
    }

    fn sample_path() -> MerklePath {
        MerklePath {
            siblings: (0..TREE_DEPTH).map(|i| digest_from(i as u64 * 10)).collect(),
            is_right: (0..TREE_DEPTH).map(|i| i % 3 == 0).collect(),
        }
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

    /// EL TEST FINAL DEL PORT COMPLETO: mensaje ISO 20022 → prueba STARK
    /// real → serialización → verificación desde bytes. Cierra el círculo
    /// igual que en los backends Groth16 y Halo2.
    #[test]
    fn valid_iso_message_produces_verifiable_stark_proof() {
        let message = sample_message(250_000);
        let package = translate_and_prove(
            &message,
            BaseElement::new(12345),
            1_000_000,
            BaseElement::new(1),
            &sample_path(),
            500_000,
            default_proof_options(),
        )
        .expect("un mensaje valido deberia producir un paquete");

        assert_eq!(package.message_id, "MSG-2026-0001");
        assert_eq!(package.currency, "EUR");
        assert!(!package.proof.is_empty());

        let valid = verify_package(&package, default_proof_options())
            .expect("la verificacion no deberia devolver error");
        assert!(valid, "el paquete generado deberia verificar correctamente");
    }

    /// Un importe que supera el saldo no debe producir un paquete
    /// utilizable. La detección puede llegar por tres vías legítimas
    /// (panic en debug, `Err` del prover, o prueba que no verifica);
    /// lo inaceptable sería un paquete que verificara.
    #[test]
    fn insufficient_balance_does_not_yield_a_verifiable_package() {
        let attempt = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            translate_and_prove(
                &sample_message(2_000_000), // mas que el saldo
                BaseElement::new(12345),
                1_000_000,
                BaseElement::new(1),
                &sample_path(),
                5_000_000, // limite alto: el fallo debe venir del saldo
                default_proof_options(),
            )
        }));

        match attempt {
            Err(_) => { /* panic: la traza invalida se detecto en debug */ }
            Ok(Err(_)) => { /* el prover la rechazo */ }
            Ok(Ok(package)) => {
                let valid = verify_package(&package, default_proof_options())
                    .expect("la verificacion no deberia devolver error");
                assert!(
                    !valid,
                    "CRITICO: un importe superior al saldo no deberia producir un paquete verificable"
                );
            }
        }
    }

    /// Validación de los campos del mensaje, antes de tocar el motor.
    #[test]
    fn malformed_messages_are_rejected_before_proving() {
        let cases = vec![
            (
                Pacs008Message {
                    message_id: "   ".into(),
                    ..sample_message(1000)
                },
                "message_id vacio",
            ),
            (
                Pacs008Message {
                    debtor_bic: "".into(),
                    ..sample_message(1000)
                },
                "debtor_bic vacio",
            ),
            (sample_message(0), "importe cero"),
        ];

        for (message, description) in cases {
            let result = translate_and_prove(
                &message,
                BaseElement::new(12345),
                1_000_000,
                BaseElement::new(1),
                &sample_path(),
                500_000,
                default_proof_options(),
            );
            assert!(
                matches!(result, Err(BridgeError::InvalidMessage(_))),
                "deberia rechazarse por mensaje invalido: {description}"
            );
        }
    }

    /// Manipular el paquete en tránsito (cambiar la raíz declarada) debe
    /// hacer fallar la verificación — el escenario de un intermediario
    /// malicioso en el bus de mensajería.
    #[test]
    fn tampered_package_fails_verification() {
        let mut package = translate_and_prove(
            &sample_message(250_000),
            BaseElement::new(12345),
            1_000_000,
            BaseElement::new(1),
            &sample_path(),
            500_000,
            default_proof_options(),
        )
        .expect("el paquete valido deberia generarse");

        package.state_root = digest_from(999_999);

        let valid = verify_package(&package, default_proof_options())
            .expect("la verificacion no deberia devolver error");
        assert!(
            !valid,
            "CRITICO: un paquete manipulado en transito no deberia verificar"
        );
    }
}
