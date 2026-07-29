//! **Puente ISO 20022**: la capa habla el idioma de la mensajería
//! bancaria.
//!
//! Sin esto, el sistema es un motor sin conexión a nada. ISO 20022 es el
//! estándar sobre el que operan SWIFT, SEPA, TARGET2 y prácticamente
//! toda la infraestructura de pagos moderna.
//!
//! ## El flujo
//!
//! ```text
//! pacs.008  (orden de transferencia)
//!    │
//!    ├─→ resolver IBAN → índice de cuenta
//!    ├─→ validar divisa e importe
//!    ├─→ generar la prueba y aplicarla
//!    │
//!    └─→ pacs.002  (informe de estado)
//!          ACSC = liquidada, con la prueba adjunta
//!          RJCT = rechazada, con código de motivo ISO
//! ```
//!
//! ## Lo que aporta frente al puente de los backends comparativos
//!
//! Aquel traducía un mensaje a un circuito suelto. **Este opera sobre el
//! ledger**: resuelve cuentas, aplica la transición de estado, y devuelve
//! una respuesta en el propio estándar.
//!
//! Y traduce los errores de la capa a **códigos de motivo ISO reales**,
//! no a mensajes propios. Un sistema receptor entiende `AM04` sin saber
//! nada de esta implementación.
//!
//! ## ⚠️ Limitaciones honestas
//!
//! - **No es un parser XML.** `Pacs008` es una struct con un subconjunto
//!   de campos. Un puente de producción necesitaría validación de
//!   esquema, espacios de nombres y los cientos de campos opcionales del
//!   estándar.
//! - **La resolución IBAN → cuenta es un registro local**, fuera de la
//!   prueba. El circuito demuestra que la transferencia entre dos
//!   posiciones del árbol es válida; que esas posiciones correspondan a
//!   esos IBAN es responsabilidad del operador.
//! - **La clave de gasto no viaja en el mensaje.** ISO 20022 no
//!   transporta claves criptográficas: viene de un almacén aparte. Es lo
//!   correcto, pero significa que el puente por sí solo no autoriza nada.
//! - **Una sola divisa.** Multidivisa exigiría un ledger por divisa y un
//!   mecanismo de cambio, que es un problema distinto.
//! - **No hay pacs.004 (devolución) ni camt.05x (extractos).**

use std::collections::HashMap;
use winterfell::math::fields::f64::BaseElement;

use stark_experiment::merkle::Digest;

use crate::{AccountIndex, LayerError, Settlement, SovereignLayer};

/// Subconjunto de un mensaje **pacs.008** (`FIToFICustomerCreditTransfer`),
/// la orden de transferencia entre instituciones financieras.
#[derive(Clone, Debug)]
pub struct Pacs008 {
    /// Identificador del mensaje, asignado por el emisor.
    pub msg_id: String,
    /// Identificador extremo a extremo, que sobrevive a toda la cadena.
    /// Es el que permite conciliar.
    pub end_to_end_id: String,
    pub debtor_iban: String,
    pub creditor_iban: String,
    /// Importe en **unidades menores** (céntimos). ISO transporta
    /// decimales; convertirlos antes evita aritmética de coma flotante en
    /// un sistema de liquidación.
    pub amount_minor: u64,
    /// Código ISO 4217, por ejemplo "EUR".
    pub currency: String,
}

/// Estado de una transacción, según el conjunto `ExternalPaymentTransactionStatus`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TxStatus {
    /// `ACSC` — liquidada y firme.
    Settled,
    /// **`ACSP` — aceptada, liquidación EN CURSO.**
    ///
    /// El dinero ha salido de la cuenta del pagador y espera en un
    /// pendiente a que el receptor lo cobre. **No es firme todavía.**
    ///
    /// Existe porque el modelo en dos fases —el único que no filtra el
    /// saldo del receptor al pagador— no produce firmeza inmediata.
    /// Responder `ACSC` en ese momento sería afirmar algo falso.
    ///
    /// ⚠️ **ISO 20022 ya distingue aceptación de firmeza.** Usar `ACSP` no
    /// inventa un contrato nuevo: usa el código que el estándar tiene para
    /// exactamente esta situación. Ver `VISION.md` §3.11.
    InProcess,
    /// `RJCT` — rechazada.
    Rejected,
}

impl TxStatus {
    pub fn code(&self) -> &'static str {
        match self {
            TxStatus::Settled => "ACSC",
            TxStatus::InProcess => "ACSP",
            TxStatus::Rejected => "RJCT",
        }
    }
}

/// Respuesta **pacs.002** (`FIToFIPaymentStatusReport`).
///
/// Cuando la liquidación tiene éxito lleva adjunta **la prueba y las
/// raíces**: el receptor puede verificar criptográficamente que la
/// transición ocurrió, sin confiar en el emisor del mensaje.
///
/// Eso es lo que ISO 20022 no puede dar por sí solo, y esta capa sí.
#[derive(Debug)]
pub struct Pacs002 {
    pub original_msg_id: String,
    pub original_end_to_end_id: String,
    pub status: TxStatus,
    /// Código de motivo ISO cuando se rechaza. `None` si se liquidó.
    pub reason_code: Option<&'static str>,
    /// Descripción legible del motivo.
    pub reason_text: Option<String>,
    /// Prueba de la liquidación. Solo presente si `status == Settled`.
    pub proof: Option<Vec<u8>>,
    pub root_old: Option<Digest>,
    pub root_new: Option<Digest>,
}

/// Traduce un error de la capa a un **código de motivo ISO real**.
///
/// Usar los códigos del estándar en vez de mensajes propios es lo que
/// permite que un sistema receptor entienda el rechazo sin conocer esta
/// implementación.
fn iso_reason(err: &LayerError) -> (&'static str, String) {
    match err {
        // AM04 — InsufficientFunds
        LayerError::InsufficientBalance { .. } => ("AM04", err.to_string()),
        // AM02 — NotAllowedAmount
        LayerError::OverRegulatoryLimit { .. } => ("AM02", err.to_string()),
        // AC01 — IncorrectAccountNumber
        LayerError::AccountNotFound(_) => ("AC01", err.to_string()),
        // AM05 — Duplication
        LayerError::NullifierAlreadySpent => ("AM05", err.to_string()),
        // AG01 — TransactionForbidden
        LayerError::NotTheAccountHolder | LayerError::NotTheIssuer => {
            ("AG01", err.to_string())
        }
        // AM02 — el límite del sistema no coincide con el declarado
        LayerError::WrongRegulatoryLimit { .. } => ("AM02", err.to_string()),
        // AM09 — WrongAmount (la banda declarada no corresponde)
        LayerError::BalanceOutsideBand { .. } => ("AM09", err.to_string()),
        // AM12 — InvalidAmount
        LayerError::SupplyCapExceeded { .. } => ("AM12", err.to_string()),
        // DS0G — el estado ha cambiado: hay que reintentar sobre el actual
        LayerError::StaleState => ("DS0G", err.to_string()),
        // TECH — fallo técnico
        // ===== CUENTA BLOQUEADA =====
        //
        // ⚠️ Antes caia en el comodin y se reportaba como "TECH", es decir
        // **problema tecnico**. Decirle eso a un banco cuando la cuenta
        // esta congelada es falso: es un rechazo de negocio, y AC06 es su
        // codigo. Ver `AUDITORIA.md` §13.
        LayerError::AccountFrozen(_) => ("AC06", err.to_string()),

        // ===== LIMITACIONES DEL SISTEMA, DECLARADAS COMO TALES =====
        //
        // Estas SI son tecnicas, pero se mapean **explicitamente** para que
        // se vea que la decision se tomo.
        LayerError::NullifierPositionCollision { .. } => ("TECH", err.to_string()),
        LayerError::PendingTreeExhausted { .. } => ("TECH", err.to_string()),
        LayerError::AccountLimitReached { .. } => ("TECH", err.to_string()),
        LayerError::CustodianSetExhausted { .. } => ("TECH", err.to_string()),
        LayerError::ProofFailed(_) => ("TECH", err.to_string()),
        LayerError::VerificationFailed(_) => ("TECH", err.to_string()),
        LayerError::Store(_) => ("TECH", err.to_string()),

        // ===== PETICIONES MAL FORMADAS =====
        LayerError::RecoveryToSameIdentity => ("MS03", err.to_string()),
        LayerError::AlreadyInThatFreezeState { .. } => ("MS03", err.to_string()),

        // ⚠️ **NO HAY COMODIN, Y ES DELIBERADO.**
        //
        // Habia un `_ => ("TECH", ...)` que absorbia **9 de las 19
        // variantes** sin que nadie lo decidiera, incluida `AccountFrozen`.
        //
        // Sin comodin, anadir un error nuevo **no compila** hasta que
        // alguien elija su codigo. Es preferible a que se reporte mal en
        // silencio.
    }
}

/// Errores propios del puente, previos a tocar la capa.
#[derive(Debug)]
pub enum BridgeError {
    UnknownIban(String),
    /// La divisa del mensaje no es la del ledger.
    CurrencyMismatch { expected: String, found: String },
    ZeroAmount,
}

impl BridgeError {
    fn iso_reason(&self) -> (&'static str, String) {
        match self {
            // AC01 — IncorrectAccountNumber
            BridgeError::UnknownIban(iban) => {
                ("AC01", format!("IBAN no registrado: {iban}"))
            }
            // AM03 — NotAllowedCurrency
            BridgeError::CurrencyMismatch { expected, found } => (
                "AM03",
                format!("divisa {found} no admitida; este ledger opera en {expected}"),
            ),
            // AM01 — ZeroAmount
            BridgeError::ZeroAmount => ("AM01", "importe cero".to_string()),
        }
    }
}

impl std::fmt::Display for BridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.iso_reason().1)
    }
}
impl std::error::Error for BridgeError {}

/// Registro que resuelve IBAN a posiciones del árbol de cuentas.
///
/// ⚠️ **Está fuera de la prueba.** El circuito demuestra que la
/// transferencia entre dos posiciones es válida; que esas posiciones
/// correspondan a esos IBAN lo garantiza el operador, no la
/// criptografía. Un sistema real ataría la correspondencia a un registro
/// firmado o la incluiría en el compromiso de la cuenta.
#[derive(Debug, Default)]
pub struct IbanRegistry {
    map: HashMap<String, AccountIndex>,
    currency: String,
}

impl IbanRegistry {
    pub fn new(currency: &str) -> Self {
        Self {
            map: HashMap::new(),
            currency: currency.to_string(),
        }
    }

    pub fn register(&mut self, iban: &str, index: AccountIndex) {
        self.map.insert(iban.to_string(), index);
    }

    pub fn resolve(&self, iban: &str) -> Option<AccountIndex> {
        self.map.get(iban).copied()
    }

    pub fn currency(&self) -> &str {
        &self.currency
    }

    /// Validaciones que no requieren tocar la capa.
    fn validate(&self, msg: &Pacs008) -> Result<(AccountIndex, AccountIndex), BridgeError> {
        if msg.currency != self.currency {
            return Err(BridgeError::CurrencyMismatch {
                expected: self.currency.clone(),
                found: msg.currency.clone(),
            });
        }
        if msg.amount_minor == 0 {
            return Err(BridgeError::ZeroAmount);
        }
        let debtor = self
            .resolve(&msg.debtor_iban)
            .ok_or_else(|| BridgeError::UnknownIban(msg.debtor_iban.clone()))?;
        let creditor = self
            .resolve(&msg.creditor_iban)
            .ok_or_else(|| BridgeError::UnknownIban(msg.creditor_iban.clone()))?;
        Ok((debtor, creditor))
    }
}

fn rejected(msg: &Pacs008, code: &'static str, text: String) -> Pacs002 {
    Pacs002 {
        original_msg_id: msg.msg_id.clone(),
        original_end_to_end_id: msg.end_to_end_id.clone(),
        status: TxStatus::Rejected,
        reason_code: Some(code),
        reason_text: Some(text),
        proof: None,
        root_old: None,
        root_new: None,
    }
}

/// **Liquida un pacs.008 y devuelve el pacs.002 correspondiente.**
///
/// Nunca falla con `Err`: un rechazo es una respuesta válida del
/// protocolo, no un error del programa. Eso es deliberado — un sistema de
/// mensajería espera **siempre** un informe de estado, y tratar el
/// rechazo como excepción llevaría a mensajes perdidos.
///
/// `sender_key` viene de un almacén de claves, no del mensaje: ISO 20022
/// no transporta claves criptográficas.
pub fn settle_pacs008(
    layer: &mut SovereignLayer,
    registry: &IbanRegistry,
    msg: &Pacs008,
    sender_key: BaseElement,
) -> Pacs002 {
    let (debtor, creditor) = match registry.validate(msg) {
        Ok(pair) => pair,
        Err(e) => {
            let (code, text) = e.iso_reason();
            return rejected(msg, code, text);
        }
    };

    let settlement: Settlement =
        match layer.transfer(sender_key, debtor, creditor, msg.amount_minor) {
            Ok(s) => s,
            Err(e) => {
                let (code, text) = iso_reason(&e);
                return rejected(msg, code, text);
            }
        };

    let root_old = settlement.public_inputs.root_old;
    let root_new = settlement.public_inputs.root_new;

    if let Err(e) = layer.apply(&settlement, debtor, creditor, msg.amount_minor) {
        let (code, text) = iso_reason(&e);
        return rejected(msg, code, text);
    }

    Pacs002 {
        original_msg_id: msg.msg_id.clone(),
        original_end_to_end_id: msg.end_to_end_id.clone(),
        status: TxStatus::Settled,
        reason_code: None,
        reason_text: None,
        proof: Some(settlement.proof),
        root_old: Some(root_old),
        root_new: Some(root_new),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests_support::*;

    const IBAN_ALICE: &str = "ES9121000418450200051332";
    const IBAN_BOB: &str = "DE89370400440532013000";
    const SK_ALICE: u64 = 0xA11CE;
    const SK_BOB: u64 = 0xB0B;

    fn setup() -> (SovereignLayer, IbanRegistry, AccountIndex, AccountIndex) {
        let mut layer = SovereignLayer::new(custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS);
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 50_000);

        let mut registry = IbanRegistry::new("EUR");
        registry.register(IBAN_ALICE, alice);
        registry.register(IBAN_BOB, bob);
        (layer, registry, alice, bob)
    }

    fn message(amount: u64, currency: &str) -> Pacs008 {
        Pacs008 {
            msg_id: "MSG-2026-0001".into(),
            end_to_end_id: "E2E-ABC-123".into(),
            debtor_iban: IBAN_ALICE.into(),
            creditor_iban: IBAN_BOB.into(),
            amount_minor: amount,
            currency: currency.into(),
        }
    }

    /// **EL TEST CLAVE**: un pacs.008 válido se liquida y devuelve un
    /// pacs.002 con la prueba adjunta.
    #[test]
    fn a_valid_pacs008_settles_and_returns_a_proof() {
        let (mut layer, registry, alice, bob) = setup();
        let msg = message(250_000, "EUR");

        let response = settle_pacs008(&mut layer, &registry, &msg, BaseElement::new(SK_ALICE));

        assert_eq!(response.status, TxStatus::Settled);
        assert_eq!(response.status.code(), "ACSC");
        assert!(response.reason_code.is_none());
        assert!(response.proof.is_some(), "la respuesta debe llevar la prueba");

        // Los identificadores se conservan: es lo que permite conciliar.
        assert_eq!(response.original_msg_id, "MSG-2026-0001");
        assert_eq!(response.original_end_to_end_id, "E2E-ABC-123");

        // Y el estado cambió de verdad.
        assert_eq!(layer.balance_of(alice), Some(750_000));
        assert_eq!(layer.balance_of(bob), Some(300_000));
        assert_eq!(response.root_new, Some(layer.state_root()));
    }

    /// **Saldo insuficiente → AM04**, el código ISO estándar.
    ///
    /// Un sistema receptor entiende `AM04` sin saber nada de esta
    /// implementación.
    #[test]
    fn insufficient_funds_maps_to_am04() {
        let (mut layer, registry, _, _) = setup();
        let msg = message(9_000_000, "EUR");

        let r = settle_pacs008(&mut layer, &registry, &msg, BaseElement::new(SK_ALICE));
        assert_eq!(r.status, TxStatus::Rejected);
        assert_eq!(r.status.code(), "RJCT");
        assert_eq!(r.reason_code, Some("AM04"));
        assert!(r.proof.is_none(), "un rechazo no lleva prueba");
    }

    /// Importe por encima del límite regulatorio → AM02.
    #[test]
    fn over_regulatory_limit_maps_to_am02() {
        let (mut layer, registry, _, _) = setup();
        let msg = message(LIMIT + 1, "EUR");
        let r = settle_pacs008(&mut layer, &registry, &msg, BaseElement::new(SK_ALICE));
        assert_eq!(r.reason_code, Some("AM02"));
    }

    /// **CUENTA CONGELADA → AC06, NO "PROBLEMA TÉCNICO".**
    ///
    /// Antes caía en un comodín `_ => ("TECH", ...)` que absorbía **9 de
    /// las 19 variantes de error**. Un banco que reciba *TECH* reintenta;
    /// uno que reciba *AC06* sabe que la cuenta está bloqueada.
    ///
    /// Decirle "problema técnico" a un rechazo de negocio **es falso**, y
    /// en un contexto de cumplimiento puede tener consecuencias.
    #[test]
    fn a_frozen_account_maps_to_ac06_not_tech() {
        let (mut layer, registry, alice, _) = setup();
        // Dos pasos: generar el recibo y APLICARLO. `set_frozen` toma
        // `&self` y solo produce la prueba; sin `apply_freeze` la cuenta
        // seguiria libre y el test comprobaria otra cosa.
        let recibo = layer
            .set_frozen(&valid_auth(), alice, true)
            .expect("generar la congelacion");
        layer
            .apply_freeze(&recibo, alice)
            .expect("aplicar la congelacion");

        let msg = message(1000, "EUR");
        let r = settle_pacs008(&mut layer, &registry, &msg, BaseElement::new(SK_ALICE));

        assert_eq!(
            r.reason_code,
            Some("AC06"),
            "una cuenta bloqueada es un rechazo de NEGOCIO, no un fallo tecnico"
        );
    }

    /// Divisa no admitida → AM03, antes de tocar la capa.
    #[test]
    fn wrong_currency_maps_to_am03() {
        let (mut layer, registry, _, _) = setup();
        let msg = message(1000, "USD");
        let r = settle_pacs008(&mut layer, &registry, &msg, BaseElement::new(SK_ALICE));
        assert_eq!(r.reason_code, Some("AM03"));
    }

    /// IBAN desconocido → AC01.
    #[test]
    fn unknown_iban_maps_to_ac01() {
        let (mut layer, registry, _, _) = setup();
        let mut msg = message(1000, "EUR");
        msg.creditor_iban = "FR7630006000011234567890189".into();
        let r = settle_pacs008(&mut layer, &registry, &msg, BaseElement::new(SK_ALICE));
        assert_eq!(r.reason_code, Some("AC01"));
    }

    /// Importe cero → AM01.
    #[test]
    fn zero_amount_maps_to_am01() {
        let (mut layer, registry, _, _) = setup();
        let msg = message(0, "EUR");
        let r = settle_pacs008(&mut layer, &registry, &msg, BaseElement::new(SK_ALICE));
        assert_eq!(r.reason_code, Some("AM01"));
    }

    /// **Sin la clave correcta no hay liquidación → AG01.**
    ///
    /// El mensaje ISO puede ser perfectamente válido; la autorización
    /// viene de otro sitio.
    #[test]
    fn wrong_spend_key_maps_to_ag01() {
        let (mut layer, registry, alice, _) = setup();
        let msg = message(250_000, "EUR");

        let r = settle_pacs008(&mut layer, &registry, &msg, BaseElement::new(0x1337));
        assert_eq!(r.status, TxStatus::Rejected);
        assert_eq!(
            r.reason_code,
            Some("AG01"),
            "un mensaje valido con clave incorrecta debe rechazarse"
        );
        assert_eq!(layer.balance_of(alice), Some(1_000_000), "sin cambios");
    }

    /// **Un rechazo NUNCA es un `Err`.**
    ///
    /// Un sistema de mensajería espera siempre un informe de estado.
    /// Tratar el rechazo como excepción llevaría a mensajes perdidos y a
    /// operaciones en limbo.
    #[test]
    fn every_message_gets_a_status_report() {
        let (mut layer, registry, _, _) = setup();
        for msg in [
            message(250_000, "EUR"),
            message(9_000_000, "EUR"),
            message(0, "EUR"),
            message(1000, "USD"),
        ] {
            let r = settle_pacs008(&mut layer, &registry, &msg, BaseElement::new(SK_ALICE));
            assert_eq!(r.original_end_to_end_id, "E2E-ABC-123");
            assert!(matches!(
                r.status,
                TxStatus::Settled | TxStatus::Rejected
            ));
        }
    }

    /// **La prueba adjunta es verificable por el receptor.**
    ///
    /// Es lo que ISO 20022 no puede dar por sí solo: el receptor no tiene
    /// que confiar en quien le envía el informe.
    #[test]
    fn the_receiver_can_verify_the_attached_proof() {
        let (mut layer, registry, _, _) = setup();
        let msg = message(250_000, "EUR");
        let r = settle_pacs008(&mut layer, &registry, &msg, BaseElement::new(SK_ALICE));

        let proof = r.proof.expect("liquidada");
        assert!(!proof.is_empty());
        // La raiz nueva declarada es la del ledger tras aplicar.
        assert_eq!(r.root_new, Some(layer.state_root()));
        assert_ne!(r.root_old, r.root_new, "el estado debe haber cambiado");
    }
}
