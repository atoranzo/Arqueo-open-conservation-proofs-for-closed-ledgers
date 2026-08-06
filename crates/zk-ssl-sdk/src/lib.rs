//! # zk-ssl-sdk — el lado del titular
//!
//! ```no_run
//! use zk_ssl_sdk::{Rpc, Wallet, Account};
//!
//! let rpc = Rpc::new("http://127.0.0.1:8545");
//! let alice = Account::open(&rpc, Wallet::random())?;   // la clave NO viaja
//! let bob = Account::open(&rpc, Wallet::random())?;
//!
//! // (en un nodo --dev: rpc.dev_fund(alice.index, 1_000_000)?)
//!
//! // FASE 1 — Alice paga: materiales del nodo, prueba EN LOCAL, recibo.
//! let notice = alice.pay(&bob.public_id(), 250_000)?;
//!
//! // El aviso viaja FUERA de banda (§21) hasta Bob, que cobra igual:
//! bob.claim(&notice)?;
//! # Ok::<(), anyhow::Error>(())
//! ```
//!
//! Todo lo que cruza la red está en `zk-ssl-wire` y `spec/RPC.md`.

use rand::RngCore;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{json, Value};
use stark_experiment::native;
use winterfell::math::fields::f64::BaseElement;
use zk_ssl::client::{ClaimMaterials, SendMaterials};
use zk_ssl::commitment::ClientState;
use zk_ssl::two_phase::PendingNotice;
use zk_ssl::{client, proof_options};
use zk_ssl_wire as wire;
use zk_ssl_wire::{digest_to_wire, Q};

pub type Digest = [BaseElement; 4];

// ─────────────────────────────── RPC ────────────────────────────────

pub struct Rpc {
    url: String,
    agent: ureq::Agent,
    next_id: std::sync::atomic::AtomicU64,
}

impl Rpc {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            agent: ureq::Agent::new(),
            next_id: std::sync::atomic::AtomicU64::new(1),
        }
    }

    pub fn call<P: Serialize, R: DeserializeOwned>(
        &self,
        method: &str,
        params: P,
    ) -> anyhow::Result<R> {
        let id = self.next_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let body = json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params });
        let resp: Value = self
            .agent
            .post(&self.url)
            .send_json(body)
            .map_err(|e| anyhow::anyhow!("{method}: transporte: {e}"))?
            .into_json()?;

        if let Some(err) = resp.get("error") {
            anyhow::bail!("{method}: el nodo rechazó: {err}");
        }
        let result = resp
            .get("result")
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("{method}: respuesta sin result"))?;
        Ok(serde_json::from_value(result)?)
    }

    /// Grifo de un nodo `--dev`: emisión delegada con custodios de prueba.
    pub fn dev_fund(&self, index: u64, amount: u64) -> anyhow::Result<Value> {
        self.call("dev_fund", json!({ "index": Q(index), "amount": Q(amount) }))
    }
}

// ────────────────────────────── Wallet ──────────────────────────────

/// La clave ancha del titular y sus derivaciones. **Nunca sale de aquí.**
#[derive(Clone, Copy)]
pub struct Wallet {
    spend_key: Digest,
}

impl Wallet {
    /// Clave ancha desde entropía del sistema (los CUATRO elementos con
    /// entropía real, que es lo que ejercita los 256 bits del circuito).
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        let mut el = || BaseElement::new(rng.next_u64());
        Self { spend_key: [el(), el(), el(), el()] }
    }

    pub fn from_elements(sk: [u64; 4]) -> Self {
        Self { spend_key: sk.map(BaseElement::new) }
    }

    pub fn public_id(&self) -> Digest {
        native::derive_public_id_wide(self.spend_key)
    }

    /// Clave de VISTA (49-A): lo único con forma de clave que viaja, y
    /// solo autoriza a LEER la propia cuenta.
    pub fn view_key(&self) -> Digest {
        native::derive_view_key_wide(self.spend_key)
    }

    pub fn view_id(&self) -> Digest {
        native::view_id_of_wide(self.spend_key)
    }

    pub fn leaf_salt(&self) -> Digest {
        native::derive_leaf_salt_wide(self.spend_key)
    }
}

// ────────────────────────────── Account ─────────────────────────────

pub struct Account<'a> {
    rpc: &'a Rpc,
    wallet: Wallet,
    pub index: u64,
}

impl<'a> Account<'a> {
    /// Abre la cuenta enviando SOLO identificadores derivados
    /// (`publicId`, `viewId`, `leafSalt`). Saldo CERO por diseño.
    pub fn open(rpc: &'a Rpc, wallet: Wallet) -> anyhow::Result<Self> {
        #[derive(serde::Deserialize)]
        struct R { index: Q }
        let r: R = rpc.call(
            "zkssl_openAccount",
            json!({
                "publicId": digest_to_wire(&wallet.public_id()),
                "viewId": digest_to_wire(&wallet.view_id()),
                "leafSalt": digest_to_wire(&wallet.leaf_salt()),
            }),
        )?;
        Ok(Self { rpc, wallet, index: r.index.0 })
    }

    /// Se ata a una cuenta ya abierta con esta misma wallet.
    pub fn attach(rpc: &'a Rpc, wallet: Wallet, index: u64) -> Self {
        Self { rpc, wallet, index }
    }

    pub fn public_id(&self) -> Digest {
        self.wallet.public_id()
    }

    /// Vista autenticada: presenta la clave de VISTA, no la de gasto.
    pub fn view(&self) -> anyhow::Result<zk_ssl::client::AccountView> {
        let dto: wire::AccountViewDto = self.rpc.call(
            "zkssl_accountView",
            json!({
                "index": Q(self.index),
                "viewKey": digest_to_wire(&self.wallet.view_key()),
            }),
        )?;
        Ok((&dto).try_into()?)
    }

    pub fn balance(&self) -> anyhow::Result<u64> {
        Ok(self.view()?.balance)
    }

    fn state(&self) -> anyhow::Result<ClientState> {
        let v = self.view()?;
        Ok(ClientState { public_id: v.public_id, balance: v.balance, nonce: v.nonce })
    }

    /// FASE 1 completa: materiales → `prove_send` EN LOCAL → `applySend`.
    /// Devuelve el aviso que hay que hacer llegar al receptor (fuera de
    /// banda: ISO 20022 no lo transporta, §21).
    pub fn pay(&self, receiver_id: &Digest, amount: u64) -> anyhow::Result<PendingNotice> {
        self.pay_with_salt(receiver_id, amount, random_salt())
    }

    pub fn pay_with_salt(
        &self,
        receiver_id: &Digest,
        amount: u64,
        salt: Digest,
    ) -> anyhow::Result<PendingNotice> {
        let estado = self.state()?;

        let m_dto: wire::SendMaterialsDto = self.rpc.call(
            "zkssl_sendMaterials",
            json!({
                "sender": Q(self.index),
                "receiverId": digest_to_wire(receiver_id),
                "amount": Q(amount),
                "salt": digest_to_wire(&salt),
            }),
        )?;
        let materials: SendMaterials = (&m_dto).try_into()?;

        // Defensa del pagador: que la capa no cambió el destinatario.
        materials
            .check_recipient(*receiver_id)
            .map_err(|e| anyhow::anyhow!("materiales con otro destinatario: {e:?}"))?;

        // La única línea donde interviene la clave de gasto: aquí, en
        // la máquina del titular.
        let receipt = client::prove_send(&materials, self.wallet.spend_key, proof_options())
            .map_err(|e| anyhow::anyhow!("prove_send: {e:?}"))?;
        let notice = receipt.notice.clone();

        let _applied: Value = self.rpc.call(
            "zkssl_applySend",
            json!({
                "receipt": wire::SendReceiptDto::from(&receipt),
                "sender": Q(self.index),
                "senderState": wire::ClientStateDto::from(&estado),
                "amount": Q(amount),
            }),
        )?;
        Ok(notice)
    }

    /// FASE 2 completa: materiales → `prove_claim` EN LOCAL → `applyClaim`.
    pub fn claim(&self, notice: &PendingNotice) -> anyhow::Result<()> {
        let estado = self.state()?;

        let m_dto: wire::ClaimMaterialsDto = self.rpc.call(
            "zkssl_claimMaterials",
            json!({
                "receiver": Q(self.index),
                "notice": wire::PendingNoticeDto::from(notice),
            }),
        )?;
        let materials: ClaimMaterials = (&m_dto).try_into()?;

        // Defensa del receptor: que estos materiales son de MI cuenta.
        if materials.receiver.public_id != self.wallet.public_id() {
            anyhow::bail!("los materiales de cobro no corresponden a esta wallet");
        }

        let receipt = client::prove_claim(&materials, self.wallet.spend_key, proof_options())
            .map_err(|e| anyhow::anyhow!("prove_claim: {e:?}"))?;

        let _applied: Value = self.rpc.call(
            "zkssl_applyClaim",
            json!({
                "receipt": wire::ClaimReceiptDto::from(&receipt),
                "receiver": Q(self.index),
                "receiverState": wire::ClientStateDto::from(&estado),
                "notice": wire::PendingNoticeDto::from(notice),
            }),
        )?;
        Ok(())
    }
}

/// Aleatorio del pendiente, con entropía del sistema.
pub fn random_salt() -> Digest {
    let mut rng = rand::thread_rng();
    let mut el = || BaseElement::new(rng.next_u64());
    [el(), el(), el(), el()]
}

/// Reexports útiles para quien construya sobre el SDK.
pub mod reexports {
    pub use zk_ssl_wire as wire;
    pub use {serde_json, winterfell};
}

pub use wire::WireError;
