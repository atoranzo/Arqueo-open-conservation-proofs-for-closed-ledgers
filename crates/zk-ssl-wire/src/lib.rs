//! # zk-ssl-wire — el formato de cable del protocolo
//!
//! DTOs JSON canónicos y sus conversiones **sin pérdida** a los tipos
//! reales de la capa. Todo digest viaja como los mismos 32 bytes que la
//! capa persiste (`store::digest_to_bytes`), así que lo que se ve en el
//! cable es byte a byte lo que hay en el árbol.
//!
//! Convenciones (de la práctica más aceptada, Ethereum JSON-RPC):
//! - `Q`    — QUANTITY: u64 como `"0x…"` sin ceros a la izquierda.
//! - `B32`  — DATA de 32 bytes: `"0x…"` (64 dígitos hex).
//! - `Blob` — DATA de longitud libre (pruebas STARK serializadas).
//!
//! La deserialización **valida**: un digest no canónico o una cantidad
//! fuera de rango se rechazan aquí, antes de tocar la capa.

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use winterfell::math::fields::f64::BaseElement;
use stark_experiment::circuit_claim::ClaimPublicInputs;
use stark_experiment::circuit_send::SendPublicInputs;
use stark_experiment::merkle::MerklePath;
use zk_ssl::client::{AccountView, ClaimMaterials, SendMaterials};
use zk_ssl::commitment::ClientState;
use zk_ssl::log::{EpochHead, LogEntry};
use zk_ssl::store::{digest_from_bytes, digest_to_bytes, element_from_bytes, element_to_bytes};
use zk_ssl::two_phase::{ClaimReceipt, PendingNotice, SendReceipt};

pub type Digest = [BaseElement; 4];

// ───────────────────────── errores de cable ─────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WireError {
    /// Hex mal formado o sin prefijo `0x`.
    BadHex,
    /// Longitud distinta de la esperada.
    BadLength { expected: usize, got: usize },
    /// Bytes que no son un digest/elemento canónico del cuerpo.
    NotCanonical,
    /// Los dos vectores de un camino Merkle no casan.
    PathMismatch,
}

impl std::fmt::Display for WireError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WireError::BadHex => write!(f, "hex mal formado (se espera \"0x…\")"),
            WireError::BadLength { expected, got } => {
                write!(f, "longitud {got}, se esperaban {expected} bytes")
            }
            WireError::NotCanonical => write!(f, "bytes no canónicos para el cuerpo"),
            WireError::PathMismatch => write!(f, "camino Merkle inconsistente"),
        }
    }
}

impl std::error::Error for WireError {}

// ───────────────────── hex sin dependencias extra ────────────────────

fn to_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(2 + bytes.len() * 2);
    s.push_str("0x");
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

fn from_hex(s: &str) -> Result<Vec<u8>, WireError> {
    let h = s.strip_prefix("0x").ok_or(WireError::BadHex)?;
    if h.len() % 2 != 0 {
        return Err(WireError::BadHex);
    }
    (0..h.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&h[i..i + 2], 16).map_err(|_| WireError::BadHex))
        .collect()
}

// ─────────────────────────── escalares ──────────────────────────────

/// QUANTITY: u64 en hex `"0x0"`, `"0x3d0900"`, …
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Q(pub u64);

impl Serialize for Q {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&format!("{:#x}", self.0))
    }
}

impl<'de> Deserialize<'de> for Q {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        let h = s.strip_prefix("0x").ok_or_else(|| D::Error::custom("QUANTITY sin 0x"))?;
        u64::from_str_radix(h, 16)
            .map(Q)
            .map_err(|_| D::Error::custom("QUANTITY inválida"))
    }
}

/// DATA de 32 bytes: la serialización canónica de un `Digest` de la capa.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct B32(pub [u8; 32]);

impl Serialize for B32 {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&to_hex(&self.0))
    }
}

impl<'de> Deserialize<'de> for B32 {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        let v = from_hex(&s).map_err(D::Error::custom)?;
        let arr: [u8; 32] = v
            .try_into()
            .map_err(|v: Vec<u8>| D::Error::custom(WireError::BadLength {
                expected: 32,
                got: v.len(),
            }))?;
        Ok(B32(arr))
    }
}

/// DATA de longitud libre (pruebas serializadas con `Proof::to_bytes`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Blob(pub Vec<u8>);

impl Serialize for Blob {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&to_hex(&self.0))
    }
}

impl<'de> Deserialize<'de> for Blob {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        from_hex(&s).map(Blob).map_err(D::Error::custom)
    }
}

// ─────────────── conversiones digest/elemento ↔ cable ───────────────

pub fn digest_to_wire(d: &Digest) -> B32 {
    B32(digest_to_bytes(d))
}

pub fn digest_from_wire(b: &B32) -> Result<Digest, WireError> {
    digest_from_bytes(&b.0).map_err(|_| WireError::NotCanonical)
}

pub fn elem_to_wire(e: BaseElement) -> Q {
    Q(u64::from_le_bytes(element_to_bytes(e)))
}

pub fn elem_from_wire(q: Q) -> Result<BaseElement, WireError> {
    element_from_bytes(&q.0.to_le_bytes()).map_err(|_| WireError::NotCanonical)
}

// ─────────────────────────────── DTOs ───────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MerklePathDto {
    pub siblings: Vec<B32>,
    pub is_right: Vec<bool>,
}

impl From<&MerklePath> for MerklePathDto {
    fn from(p: &MerklePath) -> Self {
        Self {
            siblings: p.siblings.iter().map(digest_to_wire).collect(),
            is_right: p.is_right.clone(),
        }
    }
}

impl TryFrom<&MerklePathDto> for MerklePath {
    type Error = WireError;
    fn try_from(d: &MerklePathDto) -> Result<Self, WireError> {
        if d.siblings.len() != d.is_right.len() {
            return Err(WireError::PathMismatch);
        }
        Ok(MerklePath {
            siblings: d
                .siblings
                .iter()
                .map(digest_from_wire)
                .collect::<Result<_, _>>()?,
            is_right: d.is_right.clone(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AccountViewDto {
    pub public_id: B32,
    pub balance: Q,
    pub nonce: Q,
    pub leaf_salt: B32,
}

impl From<&AccountView> for AccountViewDto {
    fn from(v: &AccountView) -> Self {
        Self {
            public_id: digest_to_wire(&v.public_id),
            balance: Q(v.balance),
            nonce: elem_to_wire(v.nonce),
            leaf_salt: digest_to_wire(&v.leaf_salt),
        }
    }
}

impl TryFrom<&AccountViewDto> for AccountView {
    type Error = WireError;
    fn try_from(d: &AccountViewDto) -> Result<Self, WireError> {
        Ok(AccountView {
            public_id: digest_from_wire(&d.public_id)?,
            balance: d.balance.0,
            nonce: elem_from_wire(d.nonce)?,
            leaf_salt: digest_from_wire(&d.leaf_salt)?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ClientStateDto {
    pub public_id: B32,
    pub balance: Q,
    pub nonce: Q,
}

impl From<&ClientState> for ClientStateDto {
    fn from(s: &ClientState) -> Self {
        Self {
            public_id: digest_to_wire(&s.public_id),
            balance: Q(s.balance),
            nonce: elem_to_wire(s.nonce),
        }
    }
}

impl TryFrom<&ClientStateDto> for ClientState {
    type Error = WireError;
    fn try_from(d: &ClientStateDto) -> Result<Self, WireError> {
        Ok(ClientState {
            public_id: digest_from_wire(&d.public_id)?,
            balance: d.balance.0,
            nonce: elem_from_wire(d.nonce)?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PendingNoticeDto {
    pub position: Q,
    pub salt: B32,
    pub amount: Q,
}

impl From<&PendingNotice> for PendingNoticeDto {
    fn from(n: &PendingNotice) -> Self {
        Self {
            position: Q(n.position),
            salt: digest_to_wire(&n.salt),
            amount: Q(n.amount),
        }
    }
}

impl TryFrom<&PendingNoticeDto> for PendingNotice {
    type Error = WireError;
    fn try_from(d: &PendingNoticeDto) -> Result<Self, WireError> {
        Ok(PendingNotice {
            position: d.position.0,
            salt: digest_from_wire(&d.salt)?,
            amount: d.amount.0,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SendMaterialsDto {
    pub sender: AccountViewDto,
    pub sender_path: MerklePathDto,
    pub frozen_path: MerklePathDto,
    pub pending_path: MerklePathDto,
    pub pending_position: Q,
    pub receiver_id: B32,
    pub regulatory_limit: Q,
    pub total_supply: Q,
    pub amount: Q,
    pub salt: B32,
}

impl From<&SendMaterials> for SendMaterialsDto {
    fn from(m: &SendMaterials) -> Self {
        Self {
            sender: (&m.sender).into(),
            sender_path: (&m.sender_path).into(),
            frozen_path: (&m.frozen_path).into(),
            pending_path: (&m.pending_path).into(),
            pending_position: Q(m.pending_position),
            receiver_id: digest_to_wire(&m.receiver_id),
            regulatory_limit: Q(m.regulatory_limit),
            total_supply: Q(m.total_supply),
            amount: Q(m.amount),
            salt: digest_to_wire(&m.salt),
        }
    }
}

impl TryFrom<&SendMaterialsDto> for SendMaterials {
    type Error = WireError;
    fn try_from(d: &SendMaterialsDto) -> Result<Self, WireError> {
        Ok(SendMaterials {
            sender: (&d.sender).try_into()?,
            sender_path: (&d.sender_path).try_into()?,
            frozen_path: (&d.frozen_path).try_into()?,
            pending_path: (&d.pending_path).try_into()?,
            pending_position: d.pending_position.0,
            receiver_id: digest_from_wire(&d.receiver_id)?,
            regulatory_limit: d.regulatory_limit.0,
            total_supply: d.total_supply.0,
            amount: d.amount.0,
            salt: digest_from_wire(&d.salt)?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ClaimMaterialsDto {
    pub receiver: AccountViewDto,
    pub receiver_path: MerklePathDto,
    pub frozen_path: MerklePathDto,
    pub pending_path: MerklePathDto,
    pub total_supply: Q,
    pub notice: PendingNoticeDto,
}

impl From<&ClaimMaterials> for ClaimMaterialsDto {
    fn from(m: &ClaimMaterials) -> Self {
        Self {
            receiver: (&m.receiver).into(),
            receiver_path: (&m.receiver_path).into(),
            frozen_path: (&m.frozen_path).into(),
            pending_path: (&m.pending_path).into(),
            total_supply: Q(m.total_supply),
            notice: (&m.notice).into(),
        }
    }
}

impl TryFrom<&ClaimMaterialsDto> for ClaimMaterials {
    type Error = WireError;
    fn try_from(d: &ClaimMaterialsDto) -> Result<Self, WireError> {
        Ok(ClaimMaterials {
            receiver: (&d.receiver).try_into()?,
            receiver_path: (&d.receiver_path).try_into()?,
            frozen_path: (&d.frozen_path).try_into()?,
            pending_path: (&d.pending_path).try_into()?,
            total_supply: d.total_supply.0,
            notice: (&d.notice).try_into()?,
        })
    }
}

/// Public inputs de `circuit_send`. Espejo campo a campo.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SendPublicInputsDto {
    pub root_old: B32,
    pub root_new: B32,
    pub frozen_root: B32,
    pub pending_root_old: B32,
    pub pending_root_new: B32,
    pub amount: Q,
    pub regulatory_limit: Q,
    pub supply_old: Q,
    pub supply_new: Q,
}

impl From<&SendPublicInputs> for SendPublicInputsDto {
    fn from(p: &SendPublicInputs) -> Self {
        Self {
            root_old: digest_to_wire(&p.root_old),
            root_new: digest_to_wire(&p.root_new),
            frozen_root: digest_to_wire(&p.frozen_root),
            pending_root_old: digest_to_wire(&p.pending_root_old),
            pending_root_new: digest_to_wire(&p.pending_root_new),
            amount: elem_to_wire(p.amount),
            regulatory_limit: elem_to_wire(p.regulatory_limit),
            supply_old: elem_to_wire(p.supply_old),
            supply_new: elem_to_wire(p.supply_new),
        }
    }
}

impl TryFrom<&SendPublicInputsDto> for SendPublicInputs {
    type Error = WireError;
    fn try_from(d: &SendPublicInputsDto) -> Result<Self, WireError> {
        Ok(SendPublicInputs {
            root_old: digest_from_wire(&d.root_old)?,
            root_new: digest_from_wire(&d.root_new)?,
            frozen_root: digest_from_wire(&d.frozen_root)?,
            pending_root_old: digest_from_wire(&d.pending_root_old)?,
            pending_root_new: digest_from_wire(&d.pending_root_new)?,
            amount: elem_from_wire(d.amount)?,
            regulatory_limit: elem_from_wire(d.regulatory_limit)?,
            supply_old: elem_from_wire(d.supply_old)?,
            supply_new: elem_from_wire(d.supply_new)?,
        })
    }
}

/// Public inputs de `circuit_claim`. Espejo campo a campo.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ClaimPublicInputsDto {
    pub root_old: B32,
    pub root_new: B32,
    pub frozen_root: B32,
    pub pending_root_old: B32,
    pub pending_root_new: B32,
    pub amount: Q,
    pub supply_old: Q,
    pub supply_new: Q,
}

impl From<&ClaimPublicInputs> for ClaimPublicInputsDto {
    fn from(p: &ClaimPublicInputs) -> Self {
        Self {
            root_old: digest_to_wire(&p.root_old),
            root_new: digest_to_wire(&p.root_new),
            frozen_root: digest_to_wire(&p.frozen_root),
            pending_root_old: digest_to_wire(&p.pending_root_old),
            pending_root_new: digest_to_wire(&p.pending_root_new),
            amount: elem_to_wire(p.amount),
            supply_old: elem_to_wire(p.supply_old),
            supply_new: elem_to_wire(p.supply_new),
        }
    }
}

impl TryFrom<&ClaimPublicInputsDto> for ClaimPublicInputs {
    type Error = WireError;
    fn try_from(d: &ClaimPublicInputsDto) -> Result<Self, WireError> {
        Ok(ClaimPublicInputs {
            root_old: digest_from_wire(&d.root_old)?,
            root_new: digest_from_wire(&d.root_new)?,
            frozen_root: digest_from_wire(&d.frozen_root)?,
            pending_root_old: digest_from_wire(&d.pending_root_old)?,
            pending_root_new: digest_from_wire(&d.pending_root_new)?,
            amount: elem_from_wire(d.amount)?,
            supply_old: elem_from_wire(d.supply_old)?,
            supply_new: elem_from_wire(d.supply_new)?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SendReceiptDto {
    /// `winterfell::Proof::to_bytes`. **66.164 bytes (64,6 KB)** para
    /// `circuit_send`, medido en §218 (banco C0.1).
    ///
    /// ⚠️ Los 36,7 KB que citan las tablas comparativas son del
    /// **circuito de comparación** (blowup 16, ext. cuadrática), no de
    /// los circuitos de esta capa. Ver `crates/zk-ssl/src/lib.rs`.
    pub proof: Blob,
    pub public_inputs: SendPublicInputsDto,
    pub commitment: B32,
    pub notice: PendingNoticeDto,
}

impl From<&SendReceipt> for SendReceiptDto {
    fn from(r: &SendReceipt) -> Self {
        Self {
            proof: Blob(r.proof.clone()),
            public_inputs: (&r.public_inputs).into(),
            commitment: digest_to_wire(&r.commitment),
            notice: (&r.notice).into(),
        }
    }
}

impl TryFrom<&SendReceiptDto> for SendReceipt {
    type Error = WireError;
    fn try_from(d: &SendReceiptDto) -> Result<Self, WireError> {
        Ok(SendReceipt {
            proof: d.proof.0.clone(),
            public_inputs: (&d.public_inputs).try_into()?,
            commitment: digest_from_wire(&d.commitment)?,
            notice: (&d.notice).try_into()?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ClaimReceiptDto {
    pub proof: Blob,
    pub public_inputs: ClaimPublicInputsDto,
}

impl From<&ClaimReceipt> for ClaimReceiptDto {
    fn from(r: &ClaimReceipt) -> Self {
        Self {
            proof: Blob(r.proof.clone()),
            public_inputs: (&r.public_inputs).into(),
        }
    }
}

impl TryFrom<&ClaimReceiptDto> for ClaimReceipt {
    type Error = WireError;
    fn try_from(d: &ClaimReceiptDto) -> Result<Self, WireError> {
        Ok(ClaimReceipt {
            proof: d.proof.0.clone(),
            public_inputs: (&d.public_inputs).try_into()?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LogEntryDto {
    pub seq: Q,
    /// `OpKind` como texto estable (`"Send"`, `"Claim"`, `"Mint"`, …).
    pub kind: String,
    pub root_old: B32,
    pub root_new: B32,
    pub proof_digest: B32,
    pub chain: B32,
    /// Era 2 (§281): el compromiso autorizante, o el centinela declarado.
    /// Ausente en las entradas de la era 1. `deny_unknown_fields` hace
    /// que un cliente viejo que reciba esto **rompa en voz alta** — el
    /// fallo honesto ya diseñado —; `default` deja al nuevo leer v1.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub compromiso: Option<B32>,
}

impl From<&LogEntry> for LogEntryDto {
    fn from(e: &LogEntry) -> Self {
        Self {
            seq: Q(e.seq),
            kind: format!("{:?}", e.kind),
            root_old: digest_to_wire(&e.root_old),
            root_new: digest_to_wire(&e.root_new),
            proof_digest: digest_to_wire(&e.proof_digest),
            chain: digest_to_wire(&e.chain),
            compromiso: e.compromiso.as_ref().map(digest_to_wire),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EpochHeadDto {
    pub seq: Q,
    pub accounts_root: B32,
    pub pending_root: B32,
    pub frozen_root: B32,
    pub chain_digest: B32,
    /// Raíz del árbol de acuses de la época (§275). Clave nueva,
    /// aditiva: `zkssl/0.2` no sube; la versión de FORMATO viaja en la
    /// firma (§236) y es la que separa v1 de v2.
    pub acuses_root: B32,
    /// Techo de retención que la cabeza declara y firma (§275).
    pub n: Q,
    /// La cima del MMR de cabezas (§292). Clave nueva, aditiva como
    /// `acusesRoot` en su dia: `zkssl/0.2` no sube; la version de
    /// FORMATO viaja en la firma y es la que separa v2 de v3.
    pub mmr_root: B32,
    /// Cuantas cabezas acumula la cima. Genesis: 0.
    pub mmr_size: Q,
    /// `EpochHead::digest()`: la cabeza entera en un solo digest.
    pub epoch_digest: B32,
}

impl From<&EpochHead> for EpochHeadDto {
    fn from(h: &EpochHead) -> Self {
        Self {
            seq: Q(h.seq),
            accounts_root: digest_to_wire(&h.accounts_root),
            pending_root: digest_to_wire(&h.pending_root),
            frozen_root: digest_to_wire(&h.frozen_root),
            chain_digest: digest_to_wire(&h.chain_digest),
            acuses_root: digest_to_wire(&h.acuses_root),
            n: Q(h.n),
            mmr_root: digest_to_wire(&h.mmr_cima),
            mmr_size: Q(h.mmr_t),
            epoch_digest: digest_to_wire(&h.digest()),
        }
    }
}

/// **Recibo de inclusión** (§259): lo que un tercero necesita para
/// comprobar, **sin el nodo**, que una hoja estaba en una cabeza firmada.
///
/// ⚠️ `leaf_format` es **una observación, no una afirmación**: el nodo
/// compone la hoja de las dos formas y declara la que casó con el árbol.
/// Si mintiera, el titular compondría mal y el recibo **fallaría** — una
/// forma equivocada no puede hacer que un recibo falso verifique, solo que
/// uno legítimo falle. Está para que el fallo sea **legible** (§254).
///
/// ⚠️ **No lleva el `leafSalt`**: es lo único que impide enumerar el saldo
/// desde un camino (§117).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InclusionReceiptDto {
    pub index: Q,
    pub leaf: B32,
    pub path: MerklePathDto,
    /// `"salted"` o `"unsalted"`.
    pub leaf_format: String,
    pub head: EpochHeadDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ParamsDto {
    pub regulatory_limit: Q,
    pub max_supply: Q,
    pub max_accounts: Q,
    pub custodian_root: B32,
}

/// **La cabeza firmada, tal como `zkssl_signedEpochHead` la sirve** (nota 95).
///
/// El decimoquinto DTO del cable, y el primero que describe una respuesta con
/// MAS DE UNA FORMA. El dispatch del nodo la ensambla hoy a mano con `json!`
/// en tres brazos; esto es el molde que los sustituye.
///
/// ## Las tres formas, y por que es un struct plano
///
/// Hay TRES respuestas, discriminadas por `available`, que en realidad son
/// **dos condiciones independientes** —ha emitido cabeza, hay firma— con una
/// de las cuatro combinaciones inalcanzable. Los campos se reparten en cuatro
/// grupos y la cuenta cuadra exacta:
///
/// ```text
/// cara minima, en las TRES (4)  available · beatSeconds · custody · custodyChecked
/// no hay firma, en 1 y 2  (1)   reason
/// hay cabeza, en 2 y 3    (3)   seq · epochDigest · emittedAtUnix
/// hay firma, solo en 3   (13)   domain · formatVersion · mmrRoot · mmrSize ·
///                               index · accountsRoot · pendingRoot ·
///                               frozenRoot · chainDigest · acusesRoot · n ·
///                               signature · publicKey
///
/// 4+1 = 5     4+3+1 = 8     4+3+13 = 20
/// ```
///
/// De ahi que **`reason` equivalga a la negacion de `available`**, y que la
/// cadena este anidada del caso 1 al 2 pero **se bifurque** en el 3.
///
/// **Es un struct plano y no un tipo suma, y la eleccion se declara entera.**
/// Un enum etiquetado haria los estados ilegales INCONSTRUIBLES; el struct
/// plano solo los hace DETECTABLES. A cambio: catorce precedentes en este
/// mismo fichero, el cable sigue con **cero enums**, y no nace un segundo
/// productor del discriminante —`serde` no puede etiquetar por un booleano,
/// asi que un enum con `tag` exigiria un campo `status` que codificaria el
/// mismo hecho que `available` **dos veces**—. El coste del `status` seria
/// permanente y viajaria en el cable; el del struct plano es local.
///
/// ⚠️ **Lo que falta a proposito y llega con su consumidor:** los tres
/// constructores nombrados (`sin_latido`, `sin_clave`, `firmada`) y el
/// accesor falible que distingue *no disponible* de *malformada* con error
/// nombrado (§254). Se escriben cuando exista quien los llame, no antes:
/// disenar una firma sin el llamante delante es como se eligen los ordenes
/// de argumentos equivocados.
///
/// ⚠️ **Lleva `deny_unknown_fields`, como sus catorce hermanos, y eso tiene
/// precio declarado:** el dia que las cofirmas entren como campo aditivo,
/// **todo consumidor tipado viejo dejara de deserializar**. Se acepta porque
/// hoy el unico consumidor tipado posible es de esta casa —`zk-ssl-verify` no
/// puede depender del cable sin arrastrar la capa y el probador— y porque un
/// tercero **no puede generar tipos aunque quiera**: el esquema
/// `SignedEpochHead` figura en el documento publicado **sin definicion**.
/// La rotura futura ocurrira dentro del mismo commit que la causa.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SignedEpochHeadDto {
    // ── la cara minima: en las TRES formas ──
    /// Hay cabeza FIRMADA que servir. Es el discriminante.
    pub available: bool,
    /// Cadencia del latido, para que el consumidor sepa cuando volver.
    pub beat_seconds: Q,
    /// ⚠️ AFIRMADO por el operador. Que este comprobado lo dice el campo
    /// de al lado, y son dos cosas distintas.
    pub custody: String,
    /// COMPROBADO por el nodo, solo posible con custodia `fichero`.
    pub custody_checked: bool,

    // ── cuando NO hay firma: en las formas 1 y 2 ──
    /// Por que no hay firma, en prosa. Equivale a `!available`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,

    // ── cuando ya ha emitido cabeza: en las formas 2 y 3 ──
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seq: Option<Q>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub epoch_digest: Option<B32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emitted_at_unix: Option<Q>,

    // ── cuando hay firma: solo en la forma 3 ──
    /// El dominio de separacion con el que se firmo (§286).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    /// ⚠️ **La version elige recompositor** (§292): sin esto el consumidor
    /// no sabe si `mmrRoot`/`mmrSize` entran en lo que la firma cubre.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format_version: Option<Q>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mmr_root: Option<B32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mmr_size: Option<Q>,
    /// ⚠️ Cuantas firmas lleva la clave, contando esta. **No entra en el
    /// preambulo**: es metadato para detectar reuso, no algo que la firma
    /// acredite (ver `zk_ssl_verify::CabezaFirmada`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<Q>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accounts_root: Option<B32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pending_root: Option<B32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frozen_root: Option<B32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_digest: Option<B32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acuses_root: Option<B32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<Q>,
    /// ⚠️ ~37 KB de hex: XMSS^MT 40/8 da firmas de 18.469 bytes. Va por
    /// `Blob` —hex de longitud libre, `DATA` en el documento publicado— y no
    /// por `B32`, que son 32 bytes clavados.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<Blob>,
    /// La clave publica del operador, en bytes del formato RFC.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key: Option<Blob>,
}

// ───────────── el accesor falible de la cabeza firmada (§312) ─────────────

/// Lo que el struct plano **no puede impedir** y aquí se detecta: con
/// `available: true`, los diecinueve campos restantes tienen que estar.
///
/// ⚠️ Es la mitad del precio de D1 (§311). El struct plano hace los estados
/// ilegales DETECTABLES, no inconstruibles; esto es la detección — y **dice
/// QUÉ falta**, no que algo falta (§254).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CabezaMalformada {
    /// `available: true` y sin el campo que la forma firmada exige.
    FaltaCampo(&'static str),
}

impl std::fmt::Display for CabezaMalformada {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CabezaMalformada::FaltaCampo(k) => {
                write!(f, "cabeza firmada sin {k}: el productor la sirvio incompleta")
            }
        }
    }
}

impl std::error::Error for CabezaMalformada {}

/// Vista **sin `Option`** de una cabeza firmada bien formada: los diecinueve
/// campos que acompañan a `available`, ya comprobados.
///
/// ⚠️ El nombre **no** es `CabezaFirmada` a propósito: ese lo ocupa
/// `zk_ssl_verify::CabezaFirmada`, que es el preámbulo de la firma y no esto.
/// El testigo importa los dos en el mismo fichero.
///
/// 💡 Son **exactamente** los diecinueve que el diario del testigo captura
/// por su lista escrita a mano. Cuando esa lista se derive de aquí (etapa 2
/// del §309), dejarán de ser dos productores del mismo conjunto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VistaFirmada<'a> {
    pub beat_seconds: Q,
    pub custody: &'a str,
    pub custody_checked: bool,
    pub seq: Q,
    pub epoch_digest: B32,
    pub emitted_at_unix: Q,
    pub domain: &'a str,
    pub format_version: Q,
    pub mmr_root: B32,
    pub mmr_size: Q,
    pub index: Q,
    pub accounts_root: B32,
    pub pending_root: B32,
    pub frozen_root: B32,
    pub chain_digest: B32,
    pub acuses_root: B32,
    pub n: Q,
    pub signature: &'a Blob,
    pub public_key: &'a Blob,
}

impl SignedEpochHeadDto {
    // ─────────── §313 · los TRES constructores nombrados ───────────
    //
    // ⚠️ Con struct plano los estados ilegales eran DETECTABLES pero no
    // inconstruibles (D1, §311). Estos tres no lo arreglan del todo —el
    // struct sigue siendo construible a mano— pero **hacen que construir
    // uno ilegal deje de ser el camino fácil**: el productor pasa de tres
    // `json!` a tres llamadas, y el invariante se vigila **en la
    // construcción y en la lectura**, no sólo en la lectura.
    //
    // ⚠️ El de la forma firmada se llama `con_firma` y **no `firmada`**:
    // ese nombre lo ocupa el accesor de arriba (§312), que lee. El nombre
    // tiene precedente en el árbol —`con_firma_el_metodo_da_todo_lo_que_
    // un_testigo_necesita`, el test del nodo—.

    /// Forma 1: **no ha habido latido todavía**. Cinco claves.
    pub fn sin_latido(beat_seconds: Q, custody: String, custody_checked: bool, reason: String) -> Self {
        Self {
            available: false,
            beat_seconds,
            custody,
            custody_checked,
            reason: Some(reason),
            seq: None,
            epoch_digest: None,
            emitted_at_unix: None,
            domain: None,
            format_version: None,
            mmr_root: None,
            mmr_size: None,
            index: None,
            accounts_root: None,
            pending_root: None,
            frozen_root: None,
            chain_digest: None,
            acuses_root: None,
            n: None,
            signature: None,
            public_key: None,
        }
    }

    /// Forma 2: **hay cabeza pero no hay firma** —el nodo arrancó sin
    /// clave—. Ocho claves: las cinco de arriba más las tres de la cabeza
    /// que sí se pueden servir.
    #[allow(clippy::too_many_arguments)]
    pub fn sin_clave(
        beat_seconds: Q,
        custody: String,
        custody_checked: bool,
        reason: String,
        seq: Q,
        epoch_digest: B32,
        emitted_at_unix: Q,
    ) -> Self {
        Self {
            seq: Some(seq),
            epoch_digest: Some(epoch_digest),
            emitted_at_unix: Some(emitted_at_unix),
            ..Self::sin_latido(beat_seconds, custody, custody_checked, reason)
        }
    }

    /// Forma 3: **la cabeza firmada**. Veinte claves.
    ///
    /// ⚠️ Toma la cabeza sin firmar ENTERA y no sus diez campos sueltos:
    /// el §311 midió que **la forma firmada contiene entera a
    /// `EpochHeadDto`**, así que el `impl From<&EpochHead>` de más arriba
    /// queda como **único productor de la forma de cable de la cabeza** y
    /// el llamante deja de escribir a mano seis `digest_to_wire`.
    #[allow(clippy::too_many_arguments)]
    pub fn con_firma(
        cabeza: &EpochHeadDto,
        domain: String,
        format_version: Q,
        index: Q,
        signature: Blob,
        public_key: Blob,
        emitted_at_unix: Q,
        beat_seconds: Q,
        custody: String,
        custody_checked: bool,
    ) -> Self {
        Self {
            available: true,
            beat_seconds,
            custody,
            custody_checked,
            reason: None,
            seq: Some(cabeza.seq),
            epoch_digest: Some(cabeza.epoch_digest),
            emitted_at_unix: Some(emitted_at_unix),
            domain: Some(domain),
            format_version: Some(format_version),
            mmr_root: Some(cabeza.mmr_root),
            mmr_size: Some(cabeza.mmr_size),
            index: Some(index),
            accounts_root: Some(cabeza.accounts_root),
            pending_root: Some(cabeza.pending_root),
            frozen_root: Some(cabeza.frozen_root),
            chain_digest: Some(cabeza.chain_digest),
            acuses_root: Some(cabeza.acuses_root),
            n: Some(cabeza.n),
            signature: Some(signature),
            public_key: Some(public_key),
        }
    }

    /// **Tres desenlaces, y ninguno colapsado** (§254):
    ///
    /// - `Ok(None)` — no hay cabeza firmada que servir. Es una respuesta
    ///   legítima del operador, no un defecto suyo.
    /// - `Ok(Some(v))` — la hay y está completa.
    /// - `Err(..)` — dice `available: true` y **le falta un campo, nombrado**.
    ///
    /// ⚠️ Un `Option` a secas juntaría el primero con el tercero, que es la
    /// figura que el §254 persigue: «no disponible» y «disponible y rota» no
    /// son la misma noticia.
    ///
    /// ⚠️ El orden de los campos aquí decide **cuál falta se reporta
    /// primero**, y es determinista: se evalúan en el orden escrito.
    pub fn firmada(&self) -> Result<Option<VistaFirmada<'_>>, CabezaMalformada> {
        if !self.available {
            return Ok(None);
        }
        // ⚠️ Macro y no closure, con el precedente de `recomponer` en el
        // testigo: un closure tendría que nombrar el tipo de retorno de cada
        // campo, y son cuatro tipos distintos.
        macro_rules! exige {
            ($campo:ident, $nombre:literal) => {
                match &self.$campo {
                    Some(x) => x,
                    None => return Err(CabezaMalformada::FaltaCampo($nombre)),
                }
            };
        }
        Ok(Some(VistaFirmada {
            beat_seconds: self.beat_seconds,
            custody: self.custody.as_str(),
            custody_checked: self.custody_checked,
            seq: *exige!(seq, "seq"),
            epoch_digest: *exige!(epoch_digest, "epochDigest"),
            emitted_at_unix: *exige!(emitted_at_unix, "emittedAtUnix"),
            domain: exige!(domain, "domain").as_str(),
            format_version: *exige!(format_version, "formatVersion"),
            mmr_root: *exige!(mmr_root, "mmrRoot"),
            mmr_size: *exige!(mmr_size, "mmrSize"),
            index: *exige!(index, "index"),
            accounts_root: *exige!(accounts_root, "accountsRoot"),
            pending_root: *exige!(pending_root, "pendingRoot"),
            frozen_root: *exige!(frozen_root, "frozenRoot"),
            chain_digest: *exige!(chain_digest, "chainDigest"),
            acuses_root: *exige!(acuses_root, "acusesRoot"),
            n: *exige!(n, "n"),
            signature: exige!(signature, "signature"),
            public_key: exige!(public_key, "publicKey"),
        }))
    }
}

/// Documento OpenRPC del protocolo (nota 74): la tabla vive aqui.
pub mod openrpc;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    // ────────────── §313 · los tres constructores nombrados ──────────────
    //
    // ⚠️ No nacen helpers ni fixtures nuevos: los tres cuerpos ya están
    // arriba (`SIN_LATIDO`, `SIN_CLAVE`, `FIRMADA`) y son los que el §311
    // pinó. Lo que estos tres tests atan es que **el constructor produce
    // EXACTAMENTE el cuerpo que el nodo sirve** — y los dos lados se
    // derivan serializando, ninguno se escribe a mano.

    #[test]
    fn sin_latido_produce_exactamente_el_cuerpo_que_el_nodo_sirve() {
        let d = SignedEpochHeadDto::sin_latido(
            Q(30),
            "fichero".into(),
            true,
            "aun no ha habido latido: el nodo acaba de arrancar".into(),
        );
        let del_fixture: SignedEpochHeadDto =
            serde_json::from_str(SIN_LATIDO).expect("el cuerpo del brazo 1 deserializa");
        assert_eq!(
            serde_json::to_value(&d).expect("serializa"),
            serde_json::to_value(&del_fixture).expect("serializa"),
            "el constructor y el cuerpo que sirve el nodo han divergido"
        );
    }

    #[test]
    fn sin_clave_produce_exactamente_el_cuerpo_que_el_nodo_sirve() {
        let d = SignedEpochHeadDto::sin_clave(
            Q(30),
            "fichero".into(),
            true,
            "el nodo arranco SIN --clave".into(),
            Q(7),
            B32([0x11; 32]),
            Q(0x66c0),
        );
        let del_fixture: SignedEpochHeadDto =
            serde_json::from_str(SIN_CLAVE).expect("el cuerpo del brazo 2 deserializa");
        assert_eq!(
            serde_json::to_value(&d).expect("serializa"),
            serde_json::to_value(&del_fixture).expect("serializa"),
            "el constructor y el cuerpo que sirve el nodo han divergido"
        );
        // Y la forma 2 contiene entera a la 1 POR CONSTRUCCION: `sin_clave`
        // se construye sobre `sin_latido`, no copiando sus campos.
        assert!(!d.available);
        assert!(d.reason.is_some(), "sin firma, `reason` siempre viaja");
    }

    #[test]
    fn con_firma_produce_exactamente_el_cuerpo_que_el_nodo_sirve() {
        // ⚠️ La cabeza sin firmar entra ENTERA: es el hallazgo del §311, y es
        // lo que deja el `impl From<&EpochHead>` como unico productor de la
        // forma de cable de la cabeza.
        let cabeza = EpochHeadDto {
            seq: Q(7),
            accounts_root: B32([0x33; 32]),
            pending_root: B32([0x44; 32]),
            frozen_root: B32([0x55; 32]),
            chain_digest: B32([0x66; 32]),
            acuses_root: B32([0x77; 32]),
            n: Q(100),
            mmr_root: B32([0x22; 32]),
            mmr_size: Q(5),
            epoch_digest: B32([0x11; 32]),
        };
        let d = SignedEpochHeadDto::con_firma(
            &cabeza,
            "ZK-SSL-epoch-head".into(),
            Q(3),
            Q(2),
            Blob(vec![0xde, 0xad, 0xbe, 0xef]),
            Blob(vec![0xab, 0xcd]),
            Q(0x66c0),
            Q(30),
            "fichero".into(),
            true,
        );
        let del_fixture: SignedEpochHeadDto =
            serde_json::from_str(FIRMADA).expect("el cuerpo del brazo 3 deserializa");
        assert_eq!(
            serde_json::to_value(&d).expect("serializa"),
            serde_json::to_value(&del_fixture).expect("serializa"),
            "el constructor y el cuerpo que sirve el nodo han divergido"
        );
        // Y lo construido pasa el accesor: los diecinueve estan.
        let vista = d.firmada().expect("bien formada").expect("hay cabeza firmada");
        assert_eq!(vista.index, Q(2));
        assert_eq!(vista.signature, &Blob(vec![0xde, 0xad, 0xbe, 0xef]));
    }

    // ─────────────── §312 · el accesor falible de la cabeza ───────────────

    /// La forma firmada COMPLETA, con las veinte claves del dispatch.
    ///
    /// ⚠️ **Fuente única de las tres pruebas del accesor**: la que necesita
    /// un campo de menos se lo quita a ESTA, no escribe una segunda copia.
    const FIRMADA_JSON: &str = r#"{"available":true,"beatSeconds":"0x1e","custody":"fichero","custodyChecked":true,"seq":"0x5","epochDigest":"0x1111111111111111111111111111111111111111111111111111111111111111","emittedAtUnix":"0x64","domain":"ZK-SSL-EPOCH-HEAD","formatVersion":"0x3","mmrRoot":"0x2222222222222222222222222222222222222222222222222222222222222222","mmrSize":"0x9","index":"0x7","accountsRoot":"0x3333333333333333333333333333333333333333333333333333333333333333","pendingRoot":"0x4444444444444444444444444444444444444444444444444444444444444444","frozenRoot":"0x5555555555555555555555555555555555555555555555555555555555555555","chainDigest":"0x6666666666666666666666666666666666666666666666666666666666666666","acusesRoot":"0x7777777777777777777777777777777777777777777777777777777777777777","n":"0x3","signature":"0xaabb","publicKey":"0xccdd"}"#;

    #[test]
    fn sin_cabeza_firmada_el_accesor_dice_que_no_hay_y_eso_no_es_un_error() {
        let d: SignedEpochHeadDto = serde_json::from_str(
            r#"{"available":false,"reason":"el nodo arranco SIN --clave","beatSeconds":"0x1e","custody":"memoria","custodyChecked":false}"#,
        )
        .expect("la cara minima deserializa");
        assert_eq!(
            d.firmada(),
            Ok(None),
            "no disponible es una respuesta legitima, no un defecto del productor"
        );
    }

    #[test]
    fn la_forma_firmada_completa_da_la_vista_con_los_diecinueve() {
        let d: SignedEpochHeadDto = serde_json::from_str(FIRMADA_JSON).expect("la forma 3 deserializa");
        // ⚠️ El conjunto de claves NO se compara contra una lista escrita a
        // mano: se DERIVA serializando el propio DTO.
        let servido: Value = serde_json::to_value(&d).expect("serializa");
        assert_eq!(
            servido.as_object().expect("objeto").len(),
            20,
            "la forma firmada son veinte claves"
        );
        let vista = d.firmada().expect("bien formada").expect("hay cabeza firmada");
        // Que la vista EXISTA ya prueba que los diecinueve estaban: si alguno
        // fuera `None`, el accesor habria devuelto `Err` con su nombre.
        assert_eq!(vista.index, Q(7));
        assert_eq!(vista.n, Q(3));
        assert_eq!(vista.epoch_digest, B32([0x11; 32]));
        assert_eq!(vista.custody, "fichero");
        assert_eq!(vista.signature, &Blob(vec![0xaa, 0xbb]));
    }

    #[test]
    fn si_falta_un_campo_el_accesor_dice_cual_y_no_solo_que_falta() {
        let mut v: Value = serde_json::from_str(FIRMADA_JSON).expect("json");
        v.as_object_mut().expect("objeto").remove("signature");
        let d: SignedEpochHeadDto =
            serde_json::from_value(v).expect("quitar un Option no impide deserializar");
        assert_eq!(d.firmada(), Err(CabezaMalformada::FaltaCampo("signature")));
        // §254 · el mensaje NOMBRA el campo; un gate que solo dice cuantos
        // fallan no es auditable.
        let texto = format!("{}", CabezaMalformada::FaltaCampo("signature"));
        assert!(texto.contains("signature"), "el error no nombra el campo: {texto}");
    }
    use std::collections::BTreeSet;

    // Los TRES cuerpos que el dispatch sirve hoy, uno por brazo del `match`
    // de `main.rs`. Son un FIXTURE del contrato, no una lista de claves: lo
    // que el test compara se DERIVA serializando (ver abajo).
    const SIN_LATIDO: &str = r#"{"available":false,
        "reason":"aun no ha habido latido: el nodo acaba de arrancar",
        "beatSeconds":"0x1e","custody":"fichero","custodyChecked":true}"#;

    const SIN_CLAVE: &str = r#"{"available":false,
        "reason":"el nodo arranco SIN --clave",
        "seq":"0x7",
        "epochDigest":"0x1111111111111111111111111111111111111111111111111111111111111111",
        "emittedAtUnix":"0x66c0",
        "beatSeconds":"0x1e","custody":"fichero","custodyChecked":true}"#;

    const FIRMADA: &str = r#"{"available":true,
        "seq":"0x7",
        "epochDigest":"0x1111111111111111111111111111111111111111111111111111111111111111",
        "domain":"ZK-SSL-epoch-head","formatVersion":"0x3",
        "mmrRoot":"0x2222222222222222222222222222222222222222222222222222222222222222",
        "mmrSize":"0x5","index":"0x2",
        "accountsRoot":"0x3333333333333333333333333333333333333333333333333333333333333333",
        "pendingRoot":"0x4444444444444444444444444444444444444444444444444444444444444444",
        "frozenRoot":"0x5555555555555555555555555555555555555555555555555555555555555555",
        "chainDigest":"0x6666666666666666666666666666666666666666666666666666666666666666",
        "acusesRoot":"0x7777777777777777777777777777777777777777777777777777777777777777",
        "n":"0x64","signature":"0xdeadbeef","publicKey":"0xabcd",
        "emittedAtUnix":"0x66c0",
        "beatSeconds":"0x1e","custody":"fichero","custodyChecked":true}"#;

    /// Las claves que el DTO EMITE, derivadas serializandolo.
    fn claves(j: &str) -> BTreeSet<String> {
        let d: SignedEpochHeadDto = serde_json::from_str(j).expect("deserializa");
        let v = serde_json::to_value(&d).expect("serializa");
        v.as_object().expect("objeto").keys().cloned().collect()
    }

    /// ⚠️ Las cuentas NO se comparan contra una lista escrita a mano: se
    /// DERIVAN serializando el DTO. Una lista aqui recrearia exactamente la
    /// figura que este tipo viene a quitar — cuatro listas parciales sin
    /// atar, repartidas por dos crates.
    #[test]
    fn las_tres_formas_dan_cinco_ocho_y_veinte_claves() {
        assert_eq!(claves(SIN_LATIDO).len(), 5, "la forma sin latido son cinco");
        assert_eq!(claves(SIN_CLAVE).len(), 8, "la forma sin clave son ocho");
        assert_eq!(claves(FIRMADA).len(), 20, "la forma firmada son veinte");
        for j in &[SIN_LATIDO, SIN_CLAVE, FIRMADA] {
            let d: SignedEpochHeadDto = serde_json::from_str(j).expect("deserializa");
            assert_eq!(
                d.reason.is_some(),
                !d.available,
                "`reason` equivale a la negacion de `available`, y aqui no: {j}"
            );
        }
    }

    /// ⚠️ **EL PRECIO DE `deny_unknown_fields`, EJECUTABLE.** El dia que las
    /// cofirmas entren como campo aditivo, este test se pondra ROJO — y eso
    /// es lo que tiene que pasar: la rotura esta declarada, no escondida.
    #[test]
    fn un_campo_aditivo_futuro_rompe_a_este_consumidor_y_asi_esta_declarado() {
        let con_cofirmas = FIRMADA.replace(
            "\"available\":true,",
            "\"available\":true,\"cofirmas\":[],",
        );
        let r: Result<SignedEpochHeadDto, _> = serde_json::from_str(&con_cofirmas);
        assert!(
            r.is_err(),
            "con deny_unknown_fields un campo nuevo TIENE que romper la deserializacion"
        );
    }

    /// La firma son ~37 KB de hex y no caben en `B32`: van por `Blob`, que es
    /// `DATA` en el documento publicado. Se comprueba ida y vuelta, prefijo
    /// incluido: el patron de `DATA` exige minusculas.
    #[test]
    fn la_firma_y_la_clave_van_por_blob_y_conservan_el_hex() {
        let d: SignedEpochHeadDto = serde_json::from_str(FIRMADA).expect("deserializa");
        assert_eq!(
            d.signature.as_ref().expect("hay firma").0,
            vec![0xde, 0xad, 0xbe, 0xef],
            "el Blob guarda los BYTES, no la cadena"
        );
        let v = serde_json::to_value(&d).expect("serializa");
        assert_eq!(v["signature"], Value::String("0xdeadbeef".into()));
        assert_eq!(v["publicKey"], Value::String("0xabcd".into()));
    }

    /// ⚠️ **MEDIDO, no supuesto: la forma firmada CONTIENE entera la cabeza
    /// sin firmar.** Los diez campos de `EpochHeadDto` estan los diez en la
    /// respuesta firmada, con el mismo nombre de cable. Los dos conjuntos se
    /// derivan serializando; ninguno se escribe a mano.
    #[test]
    fn la_forma_firmada_contiene_entera_la_cabeza_sin_firmar() {
        let cabeza = EpochHeadDto {
            seq: Q(7),
            accounts_root: B32([0x33; 32]),
            pending_root: B32([0x44; 32]),
            frozen_root: B32([0x55; 32]),
            chain_digest: B32([0x66; 32]),
            acuses_root: B32([0x77; 32]),
            n: Q(100),
            mmr_root: B32([0x22; 32]),
            mmr_size: Q(5),
            epoch_digest: B32([0x11; 32]),
        };
        let v = serde_json::to_value(&cabeza).expect("serializa");
        let suyas: BTreeSet<String> =
            v.as_object().expect("objeto").keys().cloned().collect();
        let firmadas = claves(FIRMADA);
        assert_eq!(suyas.len(), 10, "la cabeza sin firmar son diez campos");
        let fuera: Vec<&String> = suyas.difference(&firmadas).collect();
        assert!(
            fuera.is_empty(),
            "estos campos de la cabeza NO estan en la firmada: {fuera:?}"
        );
    }
}
