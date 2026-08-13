//! **Cifrado en reposo.** Quien robe el disco no puede leer los saldos.
//!
//! ## ⚠️ El alcance, dicho antes que nada
//!
//! **Protege contra**: robo del disco, de una copia de seguridad, o de
//! una instantánea exportada.
//!
//! **NO protege contra**: el operador del nodo —que ve los saldos en
//! memoria—, ni contra alguien con acceso al proceso en marcha, ni contra
//! quien obtenga la contraseña.
//!
//! Es una protección **real pero estrecha**. Leerla como "los saldos son
//! privados" sería un error: el operador sigue viéndolo todo, y eso solo
//! lo corrige la descentralización.
//!
//! ## De dónde sale la clave, y su consecuencia operativa
//!
//! **La aporta el operador al arrancar**, no se guarda junto a los datos.
//! Guardarla al lado no protegería nada: quien robe el disco se llevaría
//! ambos.
//!
//! Eso tiene una **consecuencia operativa real**: el nodo **no puede
//! reiniciar solo**. Alguien tiene que introducir la contraseña. En un
//! sistema que deba levantarse sin intervención, esto no sirve — habría
//! que usar un módulo de seguridad hardware o un servicio de claves, que
//! son otras piezas.
//!
//! ## Qué se cifra y qué no
//!
//! Se cifra **el valor**, no la clave de almacenamiento. Es decir: quien
//! tenga el disco ve **cuántas cuentas hay y cuántos nullifiers se han
//! gastado**, pero no los saldos ni las identidades.
//!
//! Ocultar también las claves exigiría cifrarlas de forma determinista
//! —para poder buscar— lo que filtra igualdad entre valores y suele ser
//! peor que no cifrar. Se prefiere ser explícito sobre qué queda expuesto.
//!
//! ## La construcción
//!
//! **XChaCha20-Poly1305**, cifrado autenticado de RustCrypto. Escribir
//! criptografía propia aquí sería un error grave.
//!
//! - **Nonce de 24 bytes aleatorio por escritura.** Con nonces de 12
//!   bytes habría que llevar un contador y un reinicio mal gestionado
//!   reutilizaría uno, lo que rompe la confidencialidad. Con 24 bytes
//!   aleatorios la colisión es despreciable sin llevar estado.
//! - **Autenticado**: un valor manipulado se detecta al descifrar, no
//!   produce datos plausibles pero falsos.
//! - La clave se deriva de la contraseña con SHA-256.
//!
//! ⚠️ **SHA-256 no es una función de derivación de contraseñas.** No
//! tiene coste ajustable, así que una contraseña débil es vulnerable a
//! fuerza bruta. Un despliegue real debería usar Argon2 o scrypt. Se
//! documenta en vez de fingir que basta.

use chacha20poly1305::aead::{Aead, KeyInit, OsRng};
use chacha20poly1305::{AeadCore, XChaCha20Poly1305, XNonce};
use sha2::{Digest as _, Sha256};

use crate::store::StoreError;

/// **Dominio de la clave del ledger en reposo** (registro de dominios:
/// `zk-ssl-hash/src/lib.rs`, §286). Era el unico literal `ZK-SSL-` suelto
/// del arbol. El keystore del SDK usa `ZK-SSL-keystore-v1`, y su test
/// exige que la clave de uno no abra al otro: la separacion que estas
/// cadenas garantizan se comprueba, no se supone.
const DOMINIO_CLAVE_LEDGER: &[u8] = b"ZK-SSL-ledger-key-v1";

/// Clave de cifrado del ledger.
///
/// No se serializa ni se guarda: vive solo en memoria mientras el nodo
/// está en marcha.
#[derive(Clone)]
pub struct LedgerKey {
    cipher: XChaCha20Poly1305,
}

impl std::fmt::Debug for LedgerKey {
    /// No imprime el material de la clave.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LedgerKey(<oculta>)")
    }
}

impl LedgerKey {
    /// Deriva la clave de una contraseña.
    ///
    /// ⚠️ Usa SHA-256, que **no es una función de derivación de
    /// contraseñas**: no tiene coste ajustable. Una contraseña débil es
    /// vulnerable a fuerza bruta. Un despliegue real debería usar Argon2
    /// o scrypt.
    pub fn from_passphrase(passphrase: &str) -> Self {
        let mut h = Sha256::new();
        h.update(DOMINIO_CLAVE_LEDGER);
        h.update(passphrase.as_bytes());
        let key = h.finalize();
        Self {
            cipher: XChaCha20Poly1305::new((&key[..]).into()),
        }
    }

    /// Cifra un valor. El nonce va delante del texto cifrado.
    pub fn seal(&self, plaintext: &[u8]) -> Result<Vec<u8>, StoreError> {
        let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
        let ct = self
            .cipher
            .encrypt(&nonce, plaintext)
            .map_err(|_| StoreError::Io("fallo al cifrar".into()))?;
        let mut out = Vec::with_capacity(nonce.len() + ct.len());
        out.extend_from_slice(&nonce);
        out.extend_from_slice(&ct);
        Ok(out)
    }

    /// Descifra un valor.
    ///
    /// Falla si el dato fue manipulado: el cifrado es **autenticado**, así
    /// que una alteración se detecta en vez de producir datos plausibles
    /// pero falsos.
    pub fn open(&self, sealed: &[u8]) -> Result<Vec<u8>, StoreError> {
        if sealed.len() < 24 {
            return Err(StoreError::Malformed(
                "dato cifrado demasiado corto".into(),
            ));
        }
        let (nonce_bytes, ct) = sealed.split_at(24);
        let nonce = XNonce::from_slice(nonce_bytes);
        self.cipher.decrypt(nonce, ct).map_err(|_| {
            StoreError::IntegrityFailure {
                what: "dato cifrado: contrasena incorrecta o manipulacion",
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ida y vuelta con la contraseña correcta.
    #[test]
    fn seal_and_open_roundtrip() {
        let k = LedgerKey::from_passphrase("una contrasena");
        let msg = b"saldo: 1000000";
        let sealed = k.seal(msg).expect("cifrar");
        assert_ne!(&sealed[24..], msg, "el texto cifrado no debe ser el claro");
        assert_eq!(k.open(&sealed).expect("descifrar"), msg);
    }

    /// **Con otra contraseña no se puede leer.**
    #[test]
    fn a_wrong_passphrase_cannot_read() {
        let good = LedgerKey::from_passphrase("correcta");
        let bad = LedgerKey::from_passphrase("incorrecta");
        let sealed = good.seal(b"secreto").expect("cifrar");
        assert!(
            bad.open(&sealed).is_err(),
            "CRITICO: otra contrasena no debe poder leer el ledger"
        );
    }

    /// **UN DATO MANIPULADO SE DETECTA.**
    ///
    /// El cifrado es autenticado: alterar un byte produce un error, no un
    /// valor plausible pero falso. Sin autenticación, un atacante con
    /// acceso al disco podría alterar saldos sin conocer la contraseña.
    #[test]
    fn tampering_is_detected_not_silently_accepted() {
        let k = LedgerKey::from_passphrase("clave");
        let mut sealed = k.seal(b"saldo: 1000").expect("cifrar");
        let last = sealed.len() - 1;
        sealed[last] ^= 0x01;
        assert!(
            matches!(k.open(&sealed), Err(StoreError::IntegrityFailure { .. })),
            "CRITICO: un dato manipulado debe detectarse, no producir un valor \
             plausible pero falso"
        );
    }

    /// **Cifrar dos veces el mismo dato da textos distintos.**
    ///
    /// Sin nonce aleatorio, dos cuentas con el mismo saldo produzcan el
    /// mismo cifrado — y un observador del disco podría agruparlas.
    #[test]
    fn identical_plaintexts_produce_different_ciphertexts() {
        let k = LedgerKey::from_passphrase("clave");
        let a = k.seal(b"mismo valor").expect("cifrar");
        let b = k.seal(b"mismo valor").expect("cifrar");
        assert_ne!(
            a, b,
            "CRITICO: sin nonce aleatorio, dos saldos iguales darian el mismo \
             cifrado y serian agrupables desde el disco"
        );
        assert_eq!(k.open(&a).unwrap(), k.open(&b).unwrap());
    }

    /// Un dato demasiado corto se rechaza en vez de interpretarse.
    #[test]
    fn a_truncated_value_is_rejected() {
        let k = LedgerKey::from_passphrase("clave");
        assert!(k.open(&[0u8; 10]).is_err());
    }

    /// La clave no se imprime en los diagnósticos.
    #[test]
    fn the_key_is_not_printed() {
        let k = LedgerKey::from_passphrase("secreto-que-no-debe-aparecer");
        let s = format!("{k:?}");
        assert!(!s.contains("secreto"), "la clave no debe aparecer en {s}");
    }
}
