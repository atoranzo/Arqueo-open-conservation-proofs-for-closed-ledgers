//! iso-bridge: traduce un mensaje de transferencia de crédito ISO 20022
//! (subconjunto simplificado de `pacs.008.001.08`, FIToFICstmrCdtTrf) hacia
//! los testigos del circuito de cumplimiento ciego de `zk-core`, y genera
//! una prueba ZK real adjunta al paquete de liquidación.
//!
//! Este crate ofrece DOS flujos:
//! - `translate_and_prove` / `verify_package`: usa `ComplianceCircuit`
//!   (sin vinculación a un estado real del ledger). Se mantiene por
//!   compatibilidad con lo ya verificado; no debería usarse en nada real.
//! - `translate_and_prove_with_state` / `verify_package_with_state`
//!   (recomendado): usa `ComplianceCircuitWithState`, que ata `balance` a
//!   una cuenta real dentro de un árbol de Merkle y expone un nullifier
//!   que impide reutilizar la misma prueba dos veces. Este es el flujo que
//!   debería usarse de aquí en adelante.
//!
//! ## Alcance honesto de este módulo
//!
//! Esto NO es un parser XML/ISO 20022 completo. Un mensaje pacs.008 real
//! tiene decenas de campos opcionales, bloques anidados (GrpHdr, CdtTrfTxInf,
//! Dbtr/Cdtr/DbtrAgt/CdtrAgt, remittance information, etc.) definidos por un
//! esquema XSD oficial de ISO 20022. Aquí se modela solo el subconjunto de
//! campos estrictamente necesario para generar la prueba de cumplimiento:
//! identificador de mensaje, BIC del deudor/acreedor, divisa e importe.
//!
//! Para producción real haría falta:
//! - Un parser XML conforme al XSD real de pacs.008.001.08 (o a la variante
//!   de la red de pagos concreta: CBPR+, T2, Fedwire, CHIPS, etc., que
//!   además difieren entre sí en campos obligatorios).
//! - Conversión de importe decimal a unidades mínimas correcta POR DIVISA
//!   (la mayoría usa 2 decimales, pero JPY usa 0 y KWD usa 3, por ejemplo).
//!   Aquí se exige directamente el importe ya en unidades mínimas (u64)
//!   para no introducir un bug de redondeo/precisión disfrazado de detalle
//!   menor.

use ark_bls12_381::Fr;
use serde::{Deserialize, Serialize};
use zk_core::{
    prove, verify, ComplianceCircuitWithState, ComplianceProof, ComplianceProvingKey,
    ComplianceVerifyingKey, MerklePath, ZkCoreError,
};

/// Subconjunto simplificado de un mensaje ISO 20022 pacs.008
/// (FIToFICstmrCdtTrf) relevante para la generación de la prueba de
/// cumplimiento.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pacs008Message {
    /// GrpHdr/MsgId — identificador único del mensaje.
    pub message_id: String,
    /// CdtTrfTxInf/DbtrAgt/FinInstnId/BICFI
    pub debtor_bic: String,
    /// CdtTrfTxInf/CdtrAgt/FinInstnId/BICFI
    pub creditor_bic: String,
    /// CdtTrfTxInf/IntrBkSttlmAmt/@Ccy
    pub currency: String,
    /// CdtTrfTxInf/IntrBkSttlmAmt, YA expresado en unidades mínimas de la
    /// divisa (p. ej. céntimos para EUR/USD). Ver nota de alcance arriba.
    pub instructed_amount_minor_units: u64,
}

/// Paquete de liquidación soberana: el mensaje ISO 20022 original permanece
/// disponible solo para las partes autorizadas (no se incluye aquí en
/// claro más allá de los metadatos públicos necesarios), y la prueba ZK
/// certifica el cumplimiento sin revelar el saldo del emisor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SovereignSettlementPackage {
    pub message_id: String,
    pub currency: String,
    pub regulatory_limit_minor_units: u64,
    #[serde(skip)]
    pub proof: Option<ComplianceProof>,
}

#[derive(Debug)]
pub enum BridgeError {
    ZkCore(ZkCoreError),
    InvalidMessage(String),
}

impl std::fmt::Display for BridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BridgeError::ZkCore(e) => write!(f, "error del motor ZK: {e}"),
            BridgeError::InvalidMessage(e) => write!(f, "mensaje ISO 20022 invalido: {e}"),
        }
    }
}
impl std::error::Error for BridgeError {}
impl From<ZkCoreError> for BridgeError {
    fn from(e: ZkCoreError) -> Self {
        BridgeError::ZkCore(e)
    }
}

/// Traduce un mensaje pacs.008 en un paquete de liquidación con prueba ZK
/// real adjunta.
///
/// `account_balance_minor_units` representa el saldo del emisor. En este
/// código se recibe como parámetro de entrada (confiado); en un sistema de
/// producción real debe derivarse de un compromiso verificado contra el
/// estado del ledger, no aceptarse tal cual venga del llamador.
pub fn translate_and_prove(
    message: &Pacs008Message,
    account_balance_minor_units: u64,
    regulatory_limit_minor_units: u64,
    proving_key: &ComplianceProvingKey,
    rng_seed: u64,
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

    let proof = prove(
        proving_key,
        account_balance_minor_units,
        message.instructed_amount_minor_units,
        regulatory_limit_minor_units,
        rng_seed,
    )?;

    Ok(SovereignSettlementPackage {
        message_id: message.message_id.clone(),
        currency: message.currency.clone(),
        regulatory_limit_minor_units,
        proof: Some(proof),
    })
}

/// Verifica el paquete de liquidación contra la clave de verificación
/// pública y el límite regulatorio declarado en el propio paquete.
pub fn verify_package(
    package: &SovereignSettlementPackage,
    verifying_key: &ComplianceVerifyingKey,
) -> Result<bool, BridgeError> {
    let proof = package
        .proof
        .as_ref()
        .ok_or_else(|| BridgeError::InvalidMessage("el paquete no contiene una prueba".into()))?;

    let is_valid = verify(verifying_key, proof, package.regulatory_limit_minor_units)?;
    Ok(is_valid)
}

// =======================================================================
// Flujo CON vinculación de estado y nullifier (recomendado). Ata
// `balance` a una cuenta real dentro de un árbol de Merkle del ledger, y
// expone un nullifier que el ledger debe registrar para impedir que la
// misma prueba se reutilice como si fuera una transacción nueva.
// =======================================================================

/// Paquete de liquidación con vinculación de estado. A diferencia de
/// `SovereignSettlementPackage`, este NO deriva `Serialize`/`Deserialize`:
/// los tipos de campo de Arkworks (`Fr`, `ComplianceProof`) no implementan
/// `serde::Serialize` por defecto en esta configuración del proyecto, y
/// preferí no asumir que sí (y arriesgarme a otro error de compilación
/// sin poder verificarlo) en vez de comprobarlo primero. Serializar este
/// paquete para enviarlo por red es un paso pendiente, no un descuido: usa
/// `ark_serialize::CanonicalSerialize` para `proof` y las representaciones
/// en bytes de `Fr` (`to_bytes_le()` vía `PrimeField`) para los campos de
/// campo finito, en vez de `serde` directamente sobre ellos.
#[derive(Debug, Clone)]
pub struct SovereignSettlementPackageWithState {
    pub message_id: String,
    pub currency: String,
    pub state_root: Fr,
    pub regulatory_limit_minor_units: u64,
    /// Nullifier público de esta transacción. El ledger debe comprobarlo
    /// contra su `NullifierRegistry` (o equivalente persistente) ANTES de
    /// aceptar la transacción como definitiva, incluso si `verify_package_with_state`
    /// devuelve `true` — ver la explicación completa en
    /// `zk_core::circuit_with_state::verify_with_state`.
    pub nullifier: Fr,
    pub proof: ComplianceProof,
}

/// Traduce un mensaje pacs.008 en un paquete de liquidación con prueba ZK
/// vinculada a estado real y protegida contra doble gasto.
///
/// `account_id`, `account_nonce`, `merkle_path` y `state_root` deben
/// corresponder a una cuenta REAL ya insertada en el árbol de estado del
/// ledger (típicamente obtenidos consultando el propio ledger antes de
/// llamar a esta función) — no son datos que el emisor pueda inventar
/// libremente, porque el circuito comprobará que encajan.
pub fn translate_and_prove_with_state(
    message: &Pacs008Message,
    account_id: Fr,
    account_balance_minor_units: u64,
    account_nonce: Fr,
    merkle_path: MerklePath<Fr>,
    state_root: Fr,
    regulatory_limit_minor_units: u64,
    proving_key: &ComplianceProvingKey,
    rng_seed: u64,
) -> Result<SovereignSettlementPackageWithState, BridgeError> {
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

    let circuit = ComplianceCircuitWithState::new(
        account_id,
        account_balance_minor_units,
        account_nonce,
        message.instructed_amount_minor_units,
        merkle_path,
        state_root,
        regulatory_limit_minor_units,
    );
    // El nullifier ya quedó calculado dentro de `new()`, a partir de
    // account_id/account_nonce reales — no se recalcula aparte, para que
    // sea imposible que el paquete exponga un nullifier distinto al que
    // el circuito realmente demostró.
    let nullifier = circuit.nullifier;

    let proof = zk_core::prove_with_state(proving_key, circuit, rng_seed)?;

    Ok(SovereignSettlementPackageWithState {
        message_id: message.message_id.clone(),
        currency: message.currency.clone(),
        state_root,
        regulatory_limit_minor_units,
        nullifier,
        proof,
    })
}

/// Verifica el paquete con estado. Devuelve `Ok(true)` si la prueba
/// criptográfica es válida — lo cual NO incluye comprobar si el nullifier
/// ya se usó antes. Esa comprobación (contra `NullifierRegistry` o el
/// almacenamiento persistente real) debe hacerla quien reciba `true` de
/// aquí, como paso siguiente obligatorio antes de dar la transacción por
/// definitiva.
pub fn verify_package_with_state(
    package: &SovereignSettlementPackageWithState,
    verifying_key: &ComplianceVerifyingKey,
) -> Result<bool, BridgeError> {
    let is_valid = zk_core::verify_with_state(
        verifying_key,
        &package.proof,
        package.state_root,
        package.regulatory_limit_minor_units,
        package.nullifier,
    )?;
    Ok(is_valid)
}

// El tipo `Fr` se importa para dejar explícito el cuerpo escalar usado por
// el circuito (BLS12-381), evitando que quien lea este crate tenga que ir
// a buscarlo en zk-core. No se usa directamente en este archivo más allá
// de esta referencia documental.
#[allow(dead_code)]
type ScalarField = Fr;

#[cfg(test)]
mod tests {
    use super::*;
    use zk_core::setup;

    fn sample_message(amount: u64) -> Pacs008Message {
        Pacs008Message {
            message_id: "ISO-PAC-2026-000123".to_string(),
            debtor_bic: "BKESESMMXXX".to_string(),
            creditor_bic: "CHASUS33XXX".to_string(),
            currency: "EUR".to_string(),
            instructed_amount_minor_units: amount,
        }
    }

    #[test]
    fn valid_iso_message_produces_verifiable_proof() {
        let (pk, vk) = setup(1).expect("setup no deberia fallar");

        let message = sample_message(250_000); // 2.500,00 EUR en centimos
        let package = translate_and_prove(
            &message,
            1_000_000, // saldo: 10.000,00 EUR
            500_000,   // limite regulatorio: 5.000,00 EUR
            &pk,
            99,
        )
        .expect("la traduccion y generacion de prueba no deberia fallar con datos validos");

        let is_valid =
            verify_package(&package, &vk).expect("la verificacion no deberia devolver error");
        assert!(is_valid);
    }

    #[test]
    fn empty_message_id_is_rejected_before_touching_zk_core() {
        let (pk, _vk) = setup(1).expect("setup no deberia fallar");
        let mut message = sample_message(1_000);
        message.message_id = "".to_string();

        let result = translate_and_prove(&message, 10_000, 5_000, &pk, 1);
        assert!(matches!(result, Err(BridgeError::InvalidMessage(_))));
    }

    /// Prueba de integración completa del flujo CON estado: desde un
    /// mensaje ISO 20022 hasta una prueba Groth16 real vinculada a una
    /// cuenta concreta del árbol de estado, y demostración de que el
    /// registro de nullifiers (no la criptografía por sí sola) es lo que
    /// bloquea la reutilización de la misma prueba como si fuera una
    /// transacción nueva.
    #[test]
    fn valid_iso_message_with_state_produces_verifiable_proof_and_blocks_replay() {
        use zk_core::{compute_leaf, setup_with_state, NullifierRegistry, SimpleMerkleTree};

        let account_id = Fr::from(12345u64);
        let account_nonce = Fr::from(1u64);
        let balance: u64 = 1_000_000;

        // Construir un arbol de estado de prueba con esa cuenta real en
        // la posicion 3 (igual que en los tests de zk-core).
        let leaf = compute_leaf(account_id, balance, account_nonce);
        let mut leaves = vec![Fr::from(1u64), Fr::from(2u64), Fr::from(3u64), leaf];
        leaves.resize(8, Fr::from(0u64));
        let tree = SimpleMerkleTree::build(leaves);
        let path = tree.path_for(3);
        let root = tree.root();

        let (pk, vk) = setup_with_state(1).expect("setup_with_state no deberia fallar");

        let message = sample_message(250_000);
        let package = translate_and_prove_with_state(
            &message, account_id, balance, account_nonce, path, root, 500_000, &pk, 42,
        )
        .expect("la generacion de la prueba con estado no deberia fallar con datos validos");

        let is_valid = verify_package_with_state(&package, &vk)
            .expect("la verificacion no deberia devolver error");
        assert!(is_valid, "una transaccion valida vinculada a estado debe verificar como verdadera");

        // La prueba criptografica sigue siendo valida si se verifica de
        // nuevo (Groth16 no "caduca" por reutilizacion): es el registro de
        // nullifiers, del lado del ledger, quien debe bloquear el reenvio.
        let mut registry = NullifierRegistry::<Fr>::new();
        assert!(
            registry.check_and_mark_spent(package.nullifier).is_ok(),
            "el primer uso de la transaccion debe aceptarse"
        );
        let replay_attempt = registry.check_and_mark_spent(package.nullifier);
        assert!(
            replay_attempt.is_err(),
            "CRITICO: el registro debio rechazar el reenvio del mismo paquete de liquidacion \
             (intento de doble gasto)."
        );
    }
}
