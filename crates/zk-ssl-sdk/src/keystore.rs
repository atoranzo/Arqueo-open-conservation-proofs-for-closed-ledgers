//! **Keystore del wallet: la clave de gasto, dormida y cifrada.**
//!
//! La MISMA construccion de reposo que la capa (`zk_ssl::crypto`):
//! XChaCha20-Poly1305 con nonce de 24 bytes aleatorio por escritura, y
//! clave derivada de la contrasena con SHA-256 sobre un DOMINIO — aqui
//! `ZK-SSL-keystore-v1`, distinto del dominio del ledger, de modo que
//! **ledger y wallet nunca comparten clave aunque compartan contrasena**
//! (hay un test que lo exige). Una sola ley de reposo en el proyecto;
//! escribir criptografia propia aqui seria un error grave.
//!
//! ⚠️ La advertencia de la capa aplica INTEGRA: SHA-256 **no es una
//! funcion de derivacion de contrasenas** (sin coste ajustable, una
//! contrasena debil es forzable). Endurecer a Argon2/scrypt es materia
//! del proceso RFC (`spec/rfc/`), no de un parche silencioso.
//!
//! El fichero guarda EN CLARO el `public_id` (es publico por diseno) y
//! cifrado el material de gasto en los 32 bytes canonicos de
//! `store::digest_to_bytes`. Al cargar se verifica que la clave
//! descifrada DERIVA ese `public_id`: un fichero cambiado de sitio o
//! editado no pasa por wallet ajeno.

use chacha20poly1305::aead::{Aead, KeyInit, OsRng};
use chacha20poly1305::{AeadCore, XChaCha20Poly1305, XNonce};
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};
use std::path::Path;
use zk_ssl::store::{digest_from_bytes, digest_to_bytes};

use crate::Wallet;

const DOMINIO: &[u8] = b"ZK-SSL-keystore-v1";
const VERSION: &str = "zkssl-keystore/1";

#[derive(Serialize, Deserialize)]
struct Fichero {
    version: String,
    kdf: String,
    aead: String,
    public_id: String,
    sealed: String,
}

fn cifra(passphrase: &str) -> XChaCha20Poly1305 {
    let mut h = Sha256::new();
    h.update(DOMINIO);
    h.update(passphrase.as_bytes());
    let key = h.finalize();
    XChaCha20Poly1305::new((&key[..]).into())
}

fn hex(b: &[u8]) -> String {
    let mut s = String::with_capacity(2 + b.len() * 2);
    s.push_str("0x");
    for x in b {
        s.push_str(&format!("{x:02x}"));
    }
    s
}

fn des_hex(s: &str) -> anyhow::Result<Vec<u8>> {
    let h = s
        .strip_prefix("0x")
        .ok_or_else(|| anyhow::anyhow!("hex sin 0x"))?;
    anyhow::ensure!(h.len() % 2 == 0, "hex de longitud impar");
    (0..h.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&h[i..i + 2], 16).map_err(Into::into))
        .collect()
}

/// Guarda el wallet cifrado. En Unix, el fichero nace con permisos 0600.
pub fn save(path: &Path, wallet: &Wallet, passphrase: &str) -> anyhow::Result<()> {
    let claro = digest_to_bytes(&wallet.spend_key());
    let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
    let ct = cifra(passphrase)
        .encrypt(&nonce, claro.as_slice())
        .map_err(|_| anyhow::anyhow!("fallo al cifrar"))?;
    let mut sealed = Vec::with_capacity(nonce.len() + ct.len());
    sealed.extend_from_slice(&nonce);
    sealed.extend_from_slice(&ct);

    let f = Fichero {
        version: VERSION.into(),
        kdf: "sha256(ZK-SSL-keystore-v1 || passphrase) — ver advertencia en zk-ssl::crypto"
            .into(),
        aead: "xchacha20poly1305, nonce 24B aleatorio antepuesto".into(),
        public_id: hex(&digest_to_bytes(&wallet.public_id())),
        sealed: hex(&sealed),
    };
    let js = serde_json::to_string_pretty(&f)? + "\n";

    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let mut fh = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(path)?;
        fh.write_all(js.as_bytes())?;
        return Ok(());
    }
    #[cfg(not(unix))]
    {
        std::fs::write(path, js)?;
        Ok(())
    }
}

/// Carga el wallet. Falla con contrasena incorrecta, fichero manipulado,
/// o un fichero que no corresponde a su `public_id` declarado.
pub fn load(path: &Path, passphrase: &str) -> anyhow::Result<Wallet> {
    let f: Fichero = serde_json::from_str(&std::fs::read_to_string(path)?)?;
    anyhow::ensure!(f.version == VERSION, "version de keystore desconocida: {}", f.version);

    let sealed = des_hex(&f.sealed)?;
    anyhow::ensure!(sealed.len() >= 24 + 32, "keystore demasiado corto");
    let (nonce_bytes, ct) = sealed.split_at(24);
    let claro = cifra(passphrase)
        .decrypt(XNonce::from_slice(nonce_bytes), ct)
        .map_err(|_| anyhow::anyhow!("contrasena incorrecta o fichero manipulado"))?;
    let sk = digest_from_bytes(&claro).map_err(|e| anyhow::anyhow!("{e:?}"))?;
    let wallet = Wallet::from_spend_key(sk);

    let declarado = des_hex(&f.public_id)?;
    let derivado = digest_to_bytes(&wallet.public_id());
    anyhow::ensure!(
        declarado == derivado,
        "el fichero no corresponde: public_id declarado != derivado"
    );
    Ok(wallet)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(nombre: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("zkssl_ks_{nombre}_{}.json", std::process::id()))
    }

    /// Ida y vuelta: el wallet cargado deriva los MISMOS identificadores.
    #[test]
    fn roundtrip_preserva_el_wallet() {
        let w = Wallet::random();
        let p = tmp("rt");
        save(&p, &w, "una contrasena").expect("guardar");
        let w2 = load(&p, "una contrasena").expect("cargar");
        assert_eq!(w.public_id(), w2.public_id());
        assert_eq!(w.view_id(), w2.view_id());
        let _ = std::fs::remove_file(&p);
    }

    /// **Con otra contrasena no se abre.**
    #[test]
    fn otra_contrasena_no_abre() {
        let w = Wallet::random();
        let p = tmp("wp");
        save(&p, &w, "correcta").expect("guardar");
        assert!(load(&p, "incorrecta").is_err(), "CRITICO: otra contrasena abrio");
        let _ = std::fs::remove_file(&p);
    }

    /// **Un byte alterado se detecta** (cifrado autenticado).
    #[test]
    fn manipulacion_detectada() {
        let w = Wallet::random();
        let p = tmp("tp");
        save(&p, &w, "clave").expect("guardar");
        let mut f: Fichero =
            serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
        let mut b = des_hex(&f.sealed).unwrap();
        let ultimo = b.len() - 1;
        b[ultimo] ^= 0x01;
        f.sealed = hex(&b);
        std::fs::write(&p, serde_json::to_string(&f).unwrap()).unwrap();
        assert!(load(&p, "clave").is_err(), "CRITICO: manipulacion no detectada");
        let _ = std::fs::remove_file(&p);
    }

    /// **Un public_id ajeno no cuela**: el binding declarado==derivado.
    #[test]
    fn public_id_ajeno_no_cuela() {
        let w = Wallet::random();
        let otro = Wallet::random();
        let p = tmp("pid");
        save(&p, &w, "clave").expect("guardar");
        let mut f: Fichero =
            serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
        f.public_id = hex(&digest_to_bytes(&otro.public_id()));
        std::fs::write(&p, serde_json::to_string(&f).unwrap()).unwrap();
        assert!(load(&p, "clave").is_err(), "CRITICO: public_id ajeno acepto");
        let _ = std::fs::remove_file(&p);
    }

    /// **DOMINIOS SEPARADOS: la clave del ledger NO abre el keystore.**
    ///
    /// Misma contrasena, dominios distintos => claves distintas. Si este
    /// test fallara, ledger y wallet compartirian clave simetrica.
    #[test]
    fn la_clave_del_ledger_no_abre_el_keystore() {
        let w = Wallet::random();
        let p = tmp("dom");
        save(&p, &w, "compartida").expect("guardar");
        let f: Fichero =
            serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
        let sealed = des_hex(&f.sealed).unwrap();
        let ledger = zk_ssl::crypto::LedgerKey::from_passphrase("compartida");
        assert!(
            ledger.open(&sealed).is_err(),
            "CRITICO: el dominio del ledger abrio el keystore"
        );
        let _ = std::fs::remove_file(&p);
    }
}
