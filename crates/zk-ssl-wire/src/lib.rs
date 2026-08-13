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

/// Documento OpenRPC del protocolo (nota 74): la tabla vive aqui.
pub mod openrpc;
