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
use stark_experiment::merkle::MerklePath;
use stark_experiment::native;
use winterfell::math::fields::f64::BaseElement;
use zk_ssl::client::{ClaimMaterials, SendMaterials};
use zk_ssl::commitment::ClientState;
use zk_ssl::prueba_prenda::{self, CabezaDeLaPrenda, SobrePrenda};
use zk_ssl::two_phase::PendingNotice;
use zk_ssl::{client, proof_options, LayerError};
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

    /// Solo para el keystore del propio crate: por aquí la clave no sale del proceso.
    pub(crate) fn spend_key(&self) -> Digest {
        self.spend_key
    }

    pub(crate) fn from_spend_key(spend_key: Digest) -> Self {
        Self { spend_key }
    }

    /// **RFC-0008 E3 (D-BH): la prenda, con la clave que no sale de aquí.** Envuelve el productor
    /// de la capa (`zk_ssl::prueba_prenda::prueba_de_prenda`, §518): la cabeza, el aviso y el
    /// camino los trae quien llama —la boca los pide al nodo—, y el SDK pone la clave, que no sale
    /// del crate ni, desde el §538, de la prueba. Quien llama no nombra al receptor: se deriva de
    /// la clave (D-BD). Falla con el `LayerError` del productor, sin variantes nuevas.
    pub fn prueba_de_prenda(
        &self,
        cabeza: &CabezaDeLaPrenda,
        aviso: &PendingNotice,
        camino: &MerklePath,
    ) -> Result<SobrePrenda, LayerError> {
        prueba_prenda::prueba_de_prenda(cabeza, self.spend_key, aviso, camino)
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
                "viewKey": digest_to_wire(&self.wallet.view_key()),
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
                "viewKey": digest_to_wire(&self.wallet.view_key()),
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

pub mod keystore;

// ─────────────────────────────── tests ──────────────────────────────

/// **La prenda desde el Wallet (RFC-0008 E3, D-BH)**, medida primero por el PASTE-542-M en una
/// copia del árbol. El escenario se monta con los compositores PÚBLICOS de la capa y sin libro
/// —`tests_support` exige la feature `sandbox`, y este crate depende de `zk-ssl` sin ella—: la
/// hoja v2 del aviso con `pending_commitment_v2`, un camino de 32 niveles con hermanos
/// cualesquiera y la raíz que `native_root` saca de ahí. El productor no lee libro: sólo
/// necesita la cabeza, el aviso y el camino.
#[cfg(test)]
mod tests {
    use super::{keystore, Digest, Wallet};
    use stark_experiment::circuit_prenda::{
        verificar_contra_cabeza, AfirmacionPrenda, CabezaPrenda, PROFUNDIDAD,
    };
    use stark_experiment::merkle::{native_root, MerklePath};
    use winterfell::math::fields::f64::BaseElement;
    use zk_ssl::pending::{pending_commitment_v2, refund_envelope};
    use zk_ssl::prueba_prenda::CabezaDeLaPrenda;
    use zk_ssl::store::digest_to_bytes;
    use zk_ssl::two_phase::PendingNotice;

    fn d(a: u64, b: u64, c: u64, e: u64) -> Digest {
        [BaseElement::new(a), BaseElement::new(b), BaseElement::new(c), BaseElement::new(e)]
    }

    /// Clave de ALTA entropía: con elementos pequeños, un 4 o un 5 en ocho bytes aparece en
    /// cualquier prueba por azar, y el censo no diría nada (5.A-360).
    const CLAVE_ALTA: [u64; 4] = [
        0x9E37_79B9_7F4A_7C15,
        0xBF58_476D_1CE4_E5B9,
        0x94D0_49BB_1331_11EB,
        0x2545_F491_4F6C_DD1D,
    ];

    fn escenario(w: &Wallet) -> (CabezaDeLaPrenda, PendingNotice, MerklePath) {
        let receptor = w.public_id();
        let salt = d(0x5A17_0001, 0x5A17_0002, 0x5A17_0003, 0x5A17_0004);
        let (amount, delta) = (250_000u64, 96u64);
        let refund_id = d(0xF00D_0001, 0xF00D_0002, 0xF00D_0003, 0xF00D_0004);
        let x = refund_envelope(refund_id, delta);
        let hoja = pending_commitment_v2(receptor, salt, amount, refund_id, delta);
        let position = 5u64;
        // La MISMA regla que `bits_de` (pub(crate) en la capa): bit n de la posición, n < 64.
        let is_right: Vec<bool> =
            (0..PROFUNDIDAD).map(|n| n < 64 && (position >> n) & 1 == 1).collect();
        let siblings: Vec<Digest> = (0..PROFUNDIDAD as u64)
            .map(|i| d(0xAB00 + i, 0xCD00 + i, 0xEF00 + i, 0x1200 + i))
            .collect();
        let camino = MerklePath { siblings, is_right };
        let cab = CabezaDeLaPrenda { seq: 7, pending_root: native_root(hoja, &camino) };
        let aviso = PendingNotice { position, salt, amount, x: Some(x) };
        (cab, aviso, camino)
    }

    /// El operador de la suite de E2 del RFC-0009 (`instrumento_revela.rs`): ventana a ventana.
    fn cuenta(pr: &[u8], x: &[u8]) -> usize {
        pr.windows(x.len()).filter(|v| *v == x).count()
    }

    #[test]
    #[cfg_attr(debug_assertions, ignore = "winterfell valida grados en depuracion: juez release")]
    fn el_wallet_prenda_y_el_juez_del_mando_lo_acepta() {
        let w = Wallet::from_elements(CLAVE_ALTA);
        let (cab, aviso, camino) = escenario(&w);
        let s = w.prueba_de_prenda(&cab, &aviso, &camino).expect("el wallet prenda su aviso");
        assert_eq!(s.receptor, w.public_id(), "el receptor del sobre es el public_id del wallet");
        assert_eq!((s.seq, s.pending_root), (cab.seq, cab.pending_root));
        let af = AfirmacionPrenda { receptor: s.receptor, marca: s.marca };
        let cabeza = CabezaPrenda { pending_root: cab.pending_root };
        verificar_contra_cabeza(&s.prueba, &af, &cabeza).expect("el juez del mando lo acepta");
    }

    /// **La clave no sale del crate, ni en la prueba.** Entera y por elementos, con prueba de
    /// vida del contador y un control positivo: el receptor, que el sobre sí lleva.
    #[test]
    #[cfg_attr(debug_assertions, ignore = "winterfell valida grados en depuracion: juez release")]
    fn la_clave_de_gasto_no_esta_en_la_prueba() {
        let w = Wallet::from_elements(CLAVE_ALTA);
        let (cab, aviso, camino) = escenario(&w);
        let s = w.prueba_de_prenda(&cab, &aviso, &camino).expect("el wallet prenda su aviso");
        let clave = digest_to_bytes(&w.spend_key());
        let entera = cuenta(&s.prueba, &clave);
        let trozos: Vec<usize> = clave.chunks(8).map(|t| cuenta(&s.prueba, t)).collect();
        let vida = cuenta(&[s.prueba.as_slice(), &clave[..]].concat(), &clave);
        let mut sobre = s.prueba.clone();
        for dd in [s.receptor, s.marca, s.pending_root] {
            sobre.extend_from_slice(&digest_to_bytes(&dd));
        }
        let receptor = cuenta(&sobre, &digest_to_bytes(&s.receptor));
        assert_eq!(vida, entera + 1, "el contador no ve la clave pegada a proposito");
        assert!(receptor >= 1, "el contador no ve el receptor, que el sobre lleva");
        assert_eq!(entera, 0, "la clave de gasto aparece en la prueba");
        assert_eq!(trozos, vec![0, 0, 0, 0], "algun elemento de la clave aparece en la prueba");
    }

    /// **El viaje entero de D-BC: keystore -> wallet -> prenda.** Quien llama pasa una ruta y
    /// una frase, y nunca toca un `Digest` de gasto.
    #[test]
    #[cfg_attr(debug_assertions, ignore = "winterfell valida grados en depuracion: juez release")]
    fn del_keystore_a_la_prenda_sin_ver_la_clave() {
        let w = Wallet::from_elements(CLAVE_ALTA);
        let (cab, aviso, camino) = escenario(&w);
        let original = w.prueba_de_prenda(&cab, &aviso, &camino).expect("con el wallet original");
        let ruta =
            std::env::temp_dir().join(format!("zk-ssl-sdk-prenda-{}.json", std::process::id()));
        let frase = "una frase larga de prueba, y solo de prueba";
        keystore::save(&ruta, &w, frase).expect("guardar el keystore");
        let cargado = keystore::load(&ruta, frase).expect("cargar el keystore");
        let _ = std::fs::remove_file(&ruta);
        let s = cargado.prueba_de_prenda(&cab, &aviso, &camino).expect("con el del keystore");
        assert_eq!((s.receptor, s.marca), (original.receptor, original.marca));
    }

    /// Otro wallet no prenda el aviso ajeno, y rehúsa ANTES de gastar una prueba: la hoja
    /// recompuesta con otra clave no sube a la raíz.
    #[test]
    fn otro_wallet_no_prenda_el_aviso_ajeno() {
        let bob = Wallet::from_elements(CLAVE_ALTA);
        let (cab, aviso, camino) = escenario(&bob);
        let ladron = Wallet::from_elements([0x1AD0_0001, 0x1AD0_0002, 0x1AD0_0003, 0x1AD0_0004]);
        let e = ladron.prueba_de_prenda(&cab, &aviso, &camino).expect_err("tenia que rehusar");
        let t = format!("{e:?}");
        assert!(t.contains("no sube a la raiz de pendientes"), "{t}");
    }

    /// La guarda de la D-F, vista desde el Wallet: un aviso sin sobre `X` se rehúsa por su nombre.
    #[test]
    fn un_aviso_v1_se_rehusa_por_su_nombre() {
        let w = Wallet::from_elements(CLAVE_ALTA);
        let (cab, mut aviso, camino) = escenario(&w);
        aviso.x = None;
        let e = w.prueba_de_prenda(&cab, &aviso, &camino).expect_err("tenia que rehusar");
        let t = format!("{e:?}");
        assert!(t.contains("el aviso es v1"), "{t}");
    }
}
