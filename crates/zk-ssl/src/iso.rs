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

use crate::commitment::ClientState;
use crate::two_phase::PendingNotice;
use crate::{AccountIndex, LayerError, SovereignLayer};

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
            ("AG08", err.to_string())
        }
        // AM02 — el límite del sistema no coincide con el declarado
        LayerError::WrongRegulatoryLimit { .. } => ("AM02", err.to_string()),
        // AM09 — WrongAmount (la banda declarada no corresponde)
        LayerError::BalanceOutsideBand { .. } => ("AM09", err.to_string()),
        // AM12 — InvalidAmount
        LayerError::SupplyCapExceeded { .. } => ("AM13", err.to_string()),
        // DS0G — el estado ha cambiado: hay que reintentar sobre el actual
        LayerError::StaleState => ("DS0G", err.to_string()),
        // TECH — fallo técnico
        // ===== CODIGOS VERIFICADOS CONTRA EL CATALOGO REAL =====
        //
        // `ExternalStatusReason1Code`, publicado por ISO 20022. Tres
        // correcciones al contrastarlo:
        //
        // - **TECH no existe en el catalogo.** Se usaba en 7 variantes.
        //   El codigo correcto es FF10: *"File or transaction cannot be
        //   processed due to technical issues at the bank side"*.
        //
        // - **AG01 es sobre el TIPO DE CUENTA**, no sobre quien firma:
        //   *"Transaction forbidden on this type of account"*. Para una
        //   clave de gasto equivocada corresponde AG08: *"Transaction
        //   failed due to invalid or missing user or access right"*.
        //
        // - **AM12 es *"Amount is invalid or missing"***. El importe no es
        //   invalido cuando se excede el tope: AM13 es *"Transaction
        //   amount exceeds limits set by clearing system"*.
        //
        // ⚠️ Dos siguen siendo dudosos y se declaran en `AUDITORIA.md` §21:
        // DS0G para `StaleState` y AM09 para `BalanceOutsideBand`.

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
        LayerError::NullifierPositionCollision { .. } => ("FF10", err.to_string()),
        LayerError::PendingTreeExhausted { .. } => ("FF10", err.to_string()),
        LayerError::AccountLimitReached { .. } => ("FF10", err.to_string()),
        LayerError::CustodianSetExhausted { .. } => ("FF10", err.to_string()),
        LayerError::ProofFailed(_) => ("FF10", err.to_string()),
        LayerError::VerificationFailed(_) => ("FF10", err.to_string()),
        LayerError::Store(_) => ("FF10", err.to_string()),

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
    /// Un elemento obligatorio del mensaje no cumple el esquema.
    ///
    /// ⚠️ **Se rechaza ANTES de tocar el ledger.** Un mensaje malformado no
    /// debe mover dinero, y hasta que se anadio esta variante **lo movia**:
    /// un `MsgId` vacio pasaba la validacion y el `pacs.002` de respuesta
    /// salia sin identificador que correlacionar.
    MalformedMessage(String),
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
            // FF10 — File or transaction cannot be processed due to a
            // technical/format problem at the receiving agent.
            //
            // ⚠️ **Codigo por confirmar.** `FF01` (Invalid File Format)
            // podria ser mas preciso para un elemento que incumple el
            // esquema. Se usa `FF10` porque **ya esta verificado en este
            // codigo** (§21) y no se quiere inventar uno: es exactamente el
            // error que aquella seccion corrigio tres veces. Anotado para
            // que lo decida quien conozca el catalogo.
            BridgeError::MalformedMessage(detalle) => ("FF10", detalle.clone()),
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
        // ⚠️ **`MsgId` es `Max35Text` OBLIGATORIO en ISO 20022.**
        //
        // Sin esta comprobacion, un mensaje con el identificador vacio se
        // procesaba entero —**el dinero se movia**— y el `pacs.002` de
        // respuesta salia con `original_msg_id` vacio: **el emisor no podia
        // correlacionar la respuesta con su peticion**.
        //
        // Lo destapo contrastar los tests del crate `iso-bridge`, superado y
        // no usado por produccion, que si lo comprobaba
        // (`empty_message_id_is_rejected_before_touching_zk_core`). Ver
        // `AUDITORIA.md` §34.
        if msg.msg_id.is_empty() {
            return Err(BridgeError::MalformedMessage(
                "MsgId vacio: ISO 20022 lo exige con al menos un caracter".into(),
            ));
        }
        if msg.msg_id.chars().count() > 35 {
            return Err(BridgeError::MalformedMessage(format!(
                "MsgId de {} caracteres: ISO 20022 lo limita a 35 (Max35Text)",
                msg.msg_id.chars().count()
            )));
        }
        // ⚠️ **Mismo limite para `EndToEndId`**, que tambien es `Max35Text` y
        // viaja igual al `pacs.002`.
        if msg.end_to_end_id.is_empty() || msg.end_to_end_id.chars().count() > 35 {
            return Err(BridgeError::MalformedMessage(
                "EndToEndId vacio o de mas de 35 caracteres (Max35Text)".into(),
            ));
        }

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

// ⚠️ **`settle_pacs008` clásico RETIRADO.**
//
// Llamaba a `transfer()`, que actualiza la hoja del receptor y por tanto
// **exige conocer su saldo**: el pagador lo aprendía. Era la fuga que la
// prioridad 0 pretendía cerrar, y seguía siendo la vía por la que un banco
// habría entrado.
//
// Lo sustituye `settle_pacs008_two_phase` + `claim_pacs008`, que
// materializan el ciclo `ACSP → ACSC` del propio estándar.
//
// **Una fuga presente y alcanzable es una fuga**, aunque exista una
// alternativa mejor al lado. Por eso se retira en vez de marcarse como
// desaconsejada.

/// **Liquida un pacs.008 por la vía en DOS FASES, sin filtrar el saldo del
/// receptor.**
///
/// ## Qué la distingue de `settle_pacs008`
///
/// La vía clásica llamaba a `transfer()`, que actualizaba la hoja del
/// receptor y por tanto **exigía conocer su saldo**. El pagador lo
/// aprendía. Era la fuga que la prioridad 0 pretendía cerrar — retirada
/// del árbol en la 32 (§161).
///
/// Ésta llama a `send()`: el dinero sale de la cuenta del pagador y queda
/// en un **pendiente**. El receptor lo cobra después con `claim`, y
/// **ninguna de las dos fases toca el saldo del otro**.
///
/// ## Por qué devuelve `ACSP` y no `ACSC`
///
/// El dinero ha salido, pero **el receptor todavía no lo tiene**. Responder
/// `ACSC` —*liquidación completada*— sería afirmar algo falso.
///
/// ISO 20022 ya distingue los dos estados: el ciclo estándar es
/// `RCVD → ACCP → ACSP → ACSC`. Esto **no inventa un contrato nuevo**; usa
/// el código que el estándar tiene para esta situación. Ver `VISION.md`
/// §3.11.
///
/// ## De dónde salen los parámetros que el mensaje no lleva
///
/// `sender_state` —el saldo y el nonce **declarados por el titular**— viene
/// del mismo sitio que `sender_key`: del banco del deudor, que conoce el
/// saldo de su cliente. **La capa no lo lee de su registro**, y ahí está la
/// diferencia.
///
/// `salt` lo elige el pagador: es lo que hace que el compromiso pendiente
/// no sea reconocible por terceros.
///
/// ## ⚠️ Lo que falta
///
/// **La segunda fase.** Cuando el receptor cobre, hace falta un segundo
/// `pacs.002` con `ACSC`. No está implementado.
pub fn settle_pacs008_two_phase(
    layer: &mut SovereignLayer,
    registry: &IbanRegistry,
    msg: &Pacs008,
    sender_key: BaseElement,
    sender_state: &ClientState,
    salt: Digest,
) -> (Pacs002, Option<PendingNotice>) {
    let (debtor, creditor) = match registry.validate(msg) {
        Ok(pair) => pair,
        Err(e) => {
            let (code, text) = e.iso_reason();
            return (rejected(msg, code, text), None);
        }
    };

    let receiver_id = match layer.public_id_of(creditor) {
        Some(id) => id,
        None => {
            let (code, text) = iso_reason(&LayerError::AccountNotFound(creditor));
            return (rejected(msg, code, text), None);
        }
    };

    let receipt = match layer.send(
        sender_key,
        debtor,
        sender_state,
        receiver_id,
        salt,
        msg.amount_minor,
    ) {
        Ok(r) => r,
        Err(e) => {
            let (code, text) = iso_reason(&e);
            return (rejected(msg, code, text), None);
        }
    };

    let root_old = receipt.public_inputs.root_old;
    let root_new = receipt.public_inputs.root_new;

    if let Err(e) = layer.apply_send(&receipt, debtor, sender_state, msg.amount_minor) {
        let (code, text) = iso_reason(&e);
        return (rejected(msg, code, text), None);
    }

    let respuesta = Pacs002 {
        original_msg_id: msg.msg_id.clone(),
        original_end_to_end_id: msg.end_to_end_id.clone(),
        // **ACSP, no ACSC**: el dinero salio, el receptor aun no lo tiene.
        status: TxStatus::InProcess,
        reason_code: None,
        reason_text: None,
        proof: Some(receipt.proof),
        root_old: Some(root_old),
        root_new: Some(root_new),
    };

    // ⚠️ **El aviso va FUERA del mensaje, y por eso va fuera del tipo.**
    //
    // El receptor necesita la posicion, el aleatorio y el importe para
    // cobrar. **ISO 20022 no tiene campo donde llevarlos**, y meterlos en
    // el de informacion de remesa seria forzarlo.
    //
    // Devolverlo aparte lo hace explicito: quien use este puente **tiene
    // que resolver como llega ese aviso al receptor**, y el tipo se lo
    // recuerda.
    (respuesta, Some(receipt.notice))
}

/// **Segunda fase: el receptor cobra, y el pago pasa a firme.**
///
/// Devuelve el segundo `pacs.002` con `ACSC` —*AcceptedSettlementCompleted*—
/// que cierra el ciclo abierto por `settle_pacs008_two_phase` con `ACSP`.
///
/// ## El ciclo completo
///
/// ```text
/// pacs.008  →  settle_pacs008_two_phase  →  pacs.002 (ACSP)
///                el dinero sale, queda pendiente
///
///           →  claim_pacs008             →  pacs.002 (ACSC)
///                el receptor cobra, el pago es firme
/// ```
///
/// Es el ciclo `RCVD → ACCP → ACSP → ACSC` del estándar, con las dos
/// últimas etapas materializadas. **No inventa nada**: ISO 20022 ya
/// distingue aceptación de firmeza.
///
/// ## Por qué el receptor tiene que actuar
///
/// Es el precio de no filtrar. Para abonar en el acto habría que actualizar
/// la hoja del receptor, y eso **exige conocer su saldo** — que es la fuga
/// que esta vía evita.
///
/// ## Los parámetros que el mensaje no lleva
///
/// `receiver_state` viene del banco del acreedor, igual que `sender_state`
/// venía del banco del deudor. `notice` lo hace llegar el pagador junto con
/// el aviso de pago: contiene la posición del pendiente, el aleatorio y el
/// importe.
///
/// ⚠️ **Cómo viaja ese aviso queda fuera de este puente.** ISO 20022 no
/// tiene campo para él, y usar el de información de remesa sería forzarlo.
/// Es una pieza real que falta.
pub fn claim_pacs008(
    layer: &mut SovereignLayer,
    previous: &Pacs002,
    receiver_index: AccountIndex,
    receiver_key: BaseElement,
    receiver_state: &ClientState,
    notice: &PendingNotice,
) -> Pacs002 {
    let ids = (
        previous.original_msg_id.clone(),
        previous.original_end_to_end_id.clone(),
    );

    let receipt = match layer.claim(receiver_key, receiver_index, receiver_state, notice) {
        Ok(r) => r,
        Err(e) => {
            let (code, text) = iso_reason(&e);
            return Pacs002 {
                original_msg_id: ids.0,
                original_end_to_end_id: ids.1,
                status: TxStatus::Rejected,
                reason_code: Some(code),
                reason_text: Some(text),
                proof: None,
                root_old: None,
                root_new: None,
            };
        }
    };

    let root_old = receipt.public_inputs.root_old;
    let root_new = receipt.public_inputs.root_new;

    if let Err(e) = layer.apply_claim(&receipt, receiver_index, receiver_state, notice) {
        let (code, text) = iso_reason(&e);
        return Pacs002 {
            original_msg_id: ids.0,
            original_end_to_end_id: ids.1,
            status: TxStatus::Rejected,
            reason_code: Some(code),
            reason_text: Some(text),
            proof: None,
            root_old: None,
            root_new: None,
        };
    }

    Pacs002 {
        original_msg_id: ids.0,
        original_end_to_end_id: ids.1,
        // **ACSC**: ahora si. El receptor tiene el dinero.
        status: TxStatus::Settled,
        reason_code: None,
        reason_text: None,
        proof: Some(receipt.proof),
        root_old: Some(root_old),
        root_new: Some(root_new),
    }
}

#[cfg(test)]
mod tests {
    // Estos tests ejercitan la via ANTIGUA a proposito: sigue siendo la
    // unica para `mint` y `mint_pending`, y sus propiedades hay que
    // comprobarlas igual. El aviso de obsolescencia se silencia aqui, no en
    // la definicion, para que siga saltando en codigo nuevo.
    #![allow(deprecated)]

    use super::*;
    use crate::tests_support::*;

    const IBAN_ALICE: &str = "ES9121000418450200051332";
    const IBAN_BOB: &str = "DE89370400440532013000";
    const SK_ALICE: u64 = 0xA11CE;
    const SK_BOB: u64 = 0xB0B;

    /// **Envuelve la vía en dos fases para los tests de mapeo de errores.**
    ///
    /// El estado del pagador y el aleatorio no vienen del mensaje: los
    /// aporta el banco del deudor. Los tests que solo comprueban códigos de
    /// rechazo no tienen por qué repetirlo once veces.
    fn settle(
        layer: &mut SovereignLayer,
        registry: &IbanRegistry,
        msg: &Pacs008,
        key: u64,
        pagador: AccountIndex,
    ) -> Pacs002 {
        let estado = state_of(layer, pagador);
        settle_pacs008_two_phase(
            layer,
            registry,
            msg,
            BaseElement::new(key),
            &estado,
            salt_iso(),
        )
        .0
    }

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

    /// **UN MENSAJE MALFORMADO NO DEBE MOVER DINERO.**
    ///
    /// `MsgId` es `Max35Text` **obligatorio** en ISO 20022. Hasta que se
    /// añadió la comprobación, un mensaje con el identificador vacío se
    /// liquidaba entero y el `pacs.002` salía con `original_msg_id` vacío:
    /// **el emisor no podía correlacionar la respuesta con su petición**.
    ///
    /// Lo destapó contrastar los tests de `iso-bridge` —crate superado que
    /// no usa producción— contra la vía real. Ver `AUDITORIA.md` §34.
    #[test]
    fn a_malformed_message_is_rejected_before_moving_money() {
        let (mut layer, registry, alice, _) = setup();
        let saldo_antes = layer.balance_of(alice);

        for (msg_id, e2e, caso) in [
            ("", "E2E-ABC-123", "MsgId vacio"),
            (&"X".repeat(36) as &str, "E2E-ABC-123", "MsgId de 36 caracteres"),
            ("MSG-2026-0001", "", "EndToEndId vacio"),
            ("MSG-2026-0001", &"Y".repeat(36) as &str, "EndToEndId de 36"),
        ] {
            let mut msg = message(100_000, "EUR");
            msg.msg_id = msg_id.into();
            msg.end_to_end_id = e2e.into();

            let r = settle(&mut layer, &registry, &msg, SK_ALICE, alice);
            assert_eq!(
                r.reason_code,
                Some("FF10"),
                "{caso}: deberia rechazarse por esquema"
            );
            assert!(r.proof.is_none(), "{caso}: no debe haber prueba");
        }

        // **Y lo que de verdad importa**: cuatro mensajes malformados y el
        // saldo intacto. El rechazo ocurre **antes de tocar el ledger**.
        assert_eq!(
            layer.balance_of(alice),
            saldo_antes,
            "CRITICO: un mensaje malformado no debe mover dinero"
        );
    }

    /// **Y el limite es de 35, no de 34 ni de 36.**
    ///
    /// El par positivo del test anterior: sin el, un cambio que rechazara
    /// TODO pasaria igual.
    #[test]
    fn identifiers_of_exactly_35_characters_are_accepted() {
        let (mut layer, registry, alice, _) = setup();
        let mut msg = message(100_000, "EUR");
        msg.msg_id = "X".repeat(35);
        msg.end_to_end_id = "Y".repeat(35);

        let r = settle(&mut layer, &registry, &msg, SK_ALICE, alice);
        assert_ne!(
            r.reason_code,
            Some("FF10"),
            "35 caracteres es el maximo PERMITIDO, no el primero prohibido"
        );
    }

    /// **EL TEST CLAVE**: un pacs.008 válido se liquida y devuelve un
    /// pacs.002 con la prueba adjunta.
    #[test]
    fn a_valid_pacs008_is_accepted_and_returns_a_proof() {
        let (mut layer, registry, alice, bob) = setup();
        let msg = message(250_000, "EUR");

        let response = settle(&mut layer, &registry, &msg, SK_ALICE, alice);

        // **ACSP, no ACSC.** El pago se acepta y el dinero sale, pero no
        // es firme hasta que el receptor cobre. Antes este test esperaba
        // ACSC porque el puente usaba la via que filtra el saldo.
        assert_eq!(response.status, TxStatus::InProcess);
        assert_eq!(response.status.code(), "ACSP");
        assert!(response.reason_code.is_none());
        assert!(response.proof.is_some(), "la respuesta debe llevar la prueba");

        // Los identificadores se conservan: es lo que permite conciliar.
        assert_eq!(response.original_msg_id, "MSG-2026-0001");
        assert_eq!(response.original_end_to_end_id, "E2E-ABC-123");

        // Y el estado cambió de verdad.
        assert_eq!(
            layer.balance_of(alice),
            Some(750_000),
            "el dinero SI salio de la cuenta del pagador"
        );
        assert_eq!(
            layer.balance_of(bob),
            Some(50_000),
            "pero el receptor sigue con lo que tenia: el dinero esta en un \
             pendiente que aun no ha cobrado. **Esta es la propiedad que \
             cierra la fuga**: para abonarselo en el acto habria que \
             actualizar su hoja, y eso exige conocer su saldo"
        );
        assert_eq!(response.root_new, Some(layer.state_root()));
    }

    /// **Saldo insuficiente → AM04**, el código ISO estándar.
    ///
    /// Un sistema receptor entiende `AM04` sin saber nada de esta
    /// implementación.
    #[test]
    fn insufficient_funds_maps_to_am04() {
        let (mut layer, registry, alice, _) = setup();
        let msg = message(9_000_000, "EUR");

        let r = settle(&mut layer, &registry, &msg, SK_ALICE, alice);
        assert_eq!(r.status, TxStatus::Rejected);
        assert_eq!(r.status.code(), "RJCT");
        assert_eq!(r.reason_code, Some("AM04"));
        assert!(r.proof.is_none(), "un rechazo no lleva prueba");
    }

    /// Importe por encima del límite regulatorio → AM02.
    #[test]
    fn over_regulatory_limit_maps_to_am02() {
        let (mut layer, registry, alice, _) = setup();
        let msg = message(LIMIT + 1, "EUR");
        let r = settle(&mut layer, &registry, &msg, SK_ALICE, alice);
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
        let r = settle(&mut layer, &registry, &msg, SK_ALICE, alice);

        assert_eq!(
            r.reason_code,
            Some("AC06"),
            "una cuenta bloqueada es un rechazo de NEGOCIO, no un fallo tecnico"
        );
    }

    /// Divisa no admitida → AM03, antes de tocar la capa.
    #[test]
    fn wrong_currency_maps_to_am03() {
        let (mut layer, registry, alice, _) = setup();
        let msg = message(1000, "USD");
        let r = settle(&mut layer, &registry, &msg, SK_ALICE, alice);
        assert_eq!(r.reason_code, Some("AM03"));
    }

    /// IBAN desconocido → AC01.
    #[test]
    fn unknown_iban_maps_to_ac01() {
        let (mut layer, registry, alice, _) = setup();
        let mut msg = message(1000, "EUR");
        msg.creditor_iban = "FR7630006000011234567890189".into();
        let r = settle(&mut layer, &registry, &msg, SK_ALICE, alice);
        assert_eq!(r.reason_code, Some("AC01"));
    }

    /// Importe cero → AM01.
    #[test]
    fn zero_amount_maps_to_am01() {
        let (mut layer, registry, alice, _) = setup();
        let msg = message(0, "EUR");
        let r = settle(&mut layer, &registry, &msg, SK_ALICE, alice);
        assert_eq!(r.reason_code, Some("AM01"));
    }

    /// **Sin la clave correcta no hay liquidación → AG01.**
    ///
    /// El mensaje ISO puede ser perfectamente válido; la autorización
    /// viene de otro sitio.
    #[test]
    fn wrong_spend_key_maps_to_ag08() {
        let (mut layer, registry, alice, _) = setup();
        let msg = message(250_000, "EUR");

        let r = settle(&mut layer, &registry, &msg, 0x1337, alice);
        assert_eq!(r.status, TxStatus::Rejected);
        assert_eq!(
            r.reason_code,
            Some("AG08"),
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
        let (mut layer, registry, alice, _) = setup();
        for msg in [
            message(250_000, "EUR"),
            message(9_000_000, "EUR"),
            message(0, "EUR"),
            message(1000, "USD"),
        ] {
            let r = settle(&mut layer, &registry, &msg, SK_ALICE, alice);
            assert_eq!(r.original_end_to_end_id, "E2E-ABC-123");
            assert!(matches!(
                r.status,
                TxStatus::InProcess | TxStatus::Rejected
            ));
        }
    }

    /// **La prueba adjunta es verificable por el receptor.**
    ///
    /// Es lo que ISO 20022 no puede dar por sí solo: el receptor no tiene
    /// que confiar en quien le envía el informe.
    #[test]
    fn the_receiver_can_verify_the_attached_proof() {
        let (mut layer, registry, alice, _) = setup();
        let msg = message(250_000, "EUR");
        let r = settle(&mut layer, &registry, &msg, SK_ALICE, alice);

        let proof = r.proof.expect("liquidada");
        assert!(!proof.is_empty());
        // La raiz nueva declarada es la del ledger tras aplicar.
        assert_eq!(r.root_new, Some(layer.state_root()));
        assert_ne!(r.root_old, r.root_new, "el estado debe haber cambiado");
    }

    /// **LA VÍA EN DOS FASES DEVUELVE `ACSP`, NO `ACSC`.**
    ///
    /// El dinero ha salido de la cuenta del pagador y espera en un
    /// pendiente. Responder *"liquidación completada"* sería afirmar algo
    /// falso: el receptor todavía no lo tiene.
    #[test]
    fn the_two_phase_bridge_reports_acsp_not_acsc() {
        let (mut layer, registry, alice, _bob) = setup();
        let estado = state_of(&layer, alice);
        let msg = message(250_000, "EUR");

        let (r, _aviso) = settle_pacs008_two_phase(
            &mut layer,
            &registry,
            &msg,
            BaseElement::new(SK_ALICE),
            &estado,
            salt_iso(),
        );

        assert_eq!(r.status, TxStatus::InProcess, "aceptada, no firme");
        assert_eq!(r.status.code(), "ACSP");
        assert!(r.proof.is_some(), "la prueba se adjunta igual");
    }

    /// **EL SALDO DEL RECEPTOR NO INTERVIENE.**
    ///
    /// Es la propiedad que justifica toda la vía nueva, y **va en el
    /// tipo**: `settle_pacs008_two_phase` recibe el estado del **pagador**
    /// y nada del receptor. No hay parámetro donde entrara su saldo.
    ///
    /// La vía clásica, en cambio, llamaba a `transfer()`, que actualizaba
    /// la hoja del receptor y por tanto **exigía conocerlo** (retirada,
    /// §161).
    ///
    /// Este test lo comprueba por su efecto: el saldo del receptor **no
    /// cambia** tras el envío, porque el dinero está en un pendiente que
    /// aún no ha cobrado.
    #[test]
    fn the_two_phase_bridge_does_not_touch_the_recipient() {
        let (mut layer, registry, alice, bob) = setup();
        let antes = layer.balance_of(bob).expect("cuenta");
        let estado = state_of(&layer, alice);

        let (r, _aviso) = settle_pacs008_two_phase(
            &mut layer,
            &registry,
            &message(250_000, "EUR"),
            BaseElement::new(SK_ALICE),
            &estado,
            salt_iso(),
        );
        assert_eq!(r.status, TxStatus::InProcess);

        assert_eq!(
            layer.balance_of(bob),
            Some(antes),
            "el saldo del receptor NO cambia: el dinero esta en un pendiente \
             que todavia no ha cobrado"
        );
        assert_eq!(
            layer.balance_of(alice),
            Some(1_000_000 - 250_000),
            "pero el del pagador SI: el dinero ya salio"
        );
    }


    fn salt_iso() -> Digest {
        [
            BaseElement::new(0x150),
            BaseElement::new(0x151),
            BaseElement::new(0x152),
            BaseElement::new(0x153),
        ]
    }

    /// **EL CICLO ISO COMPLETO, SIN FILTRAR NINGÚN SALDO.**
    ///
    /// ```text
    /// pacs.008  →  ACSP   el dinero sale, queda pendiente
    ///           →  ACSC   el receptor cobra, el pago es firme
    /// ```
    ///
    /// Y en ninguna de las dos fases interviene el saldo del otro: el
    /// pagador aporta el suyo, el receptor el suyo.
    #[test]
    fn the_full_two_phase_iso_cycle() {
        let (mut layer, registry, alice, bob) = setup();
        let bob_antes = layer.balance_of(bob).expect("cuenta");

        // --- Fase 1: el pagador envia ---
        let estado_alice = state_of(&layer, alice);
        let (acsp, aviso) = settle_pacs008_two_phase(
            &mut layer,
            &registry,
            &message(250_000, "EUR"),
            BaseElement::new(SK_ALICE),
            &estado_alice,
            salt_iso(),
        );
        assert_eq!(acsp.status.code(), "ACSP");
        let aviso = aviso.expect("el aviso para el receptor");

        assert_eq!(
            layer.balance_of(bob),
            Some(bob_antes),
            "tras ACSP el receptor NO tiene el dinero todavia"
        );

        // --- Fase 2: el receptor cobra ---
        let estado_bob = state_of(&layer, bob);
        let acsc = claim_pacs008(
            &mut layer,
            &acsp,
            bob,
            BaseElement::new(SK_BOB),
            &estado_bob,
            &aviso,
        );

        assert_eq!(acsc.status.code(), "ACSC", "ahora si es firme");
        assert!(acsc.proof.is_some(), "con su prueba adjunta");
        assert_eq!(
            acsc.original_end_to_end_id, acsp.original_end_to_end_id,
            "los dos informes se refieren al mismo pago original"
        );

        assert_eq!(
            layer.balance_of(bob),
            Some(bob_antes + 250_000),
            "y ahora si tiene el dinero"
        );
        assert_eq!(
            layer.balance_of(alice),
            Some(1_000_000 - 250_000),
            "el pagador no recupera nada"
        );
    }

    /// **NADIE MÁS PUEDE COBRAR ESE PENDIENTE.**
    ///
    /// Aunque tuviera el aviso completo —posición, aleatorio e importe—,
    /// sin la clave del receptor la prueba no se genera.
    #[test]
    fn a_third_party_cannot_claim_the_pending() {
        let (mut layer, registry, alice, bob) = setup();
        let estado_alice = state_of(&layer, alice);
        let (acsp, aviso) = settle_pacs008_two_phase(
            &mut layer,
            &registry,
            &message(250_000, "EUR"),
            BaseElement::new(SK_ALICE),
            &estado_alice,
            salt_iso(),
        );
        let aviso = aviso.expect("aviso");

        // Alice intenta cobrar el pendiente que ella misma envio.
        let estado_alice2 = state_of(&layer, alice);
        let r = claim_pacs008(
            &mut layer,
            &acsp,
            alice,
            BaseElement::new(SK_ALICE),
            &estado_alice2,
            &aviso,
        );

        assert_eq!(
            r.status,
            TxStatus::Rejected,
            "el compromiso se formo con la identidad de BOB: nadie mas lo cobra"
        );
        assert_eq!(
            layer.balance_of(bob),
            Some(50_000),
            "y el pendiente sigue sin cobrarse"
        );
    }
}
