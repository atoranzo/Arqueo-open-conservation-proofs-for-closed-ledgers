//! **Keystore del wallet: la clave de gasto, dormida y cifrada.**
//!
//! La MISMA construccion de reposo que la capa (`zk_ssl::crypto`):
//! XChaCha20-Poly1305 con nonce de 24 bytes aleatorio por escritura, y
//! clave derivada de la contrasena por `zk_ssl::crypto::Kdf` sobre un
//! DOMINIO propio, distinto del dominio del ledger, de modo que **ledger y
//! wallet nunca comparten clave aunque compartan contrasena** (hay un test
//! que lo exige). Una sola ley de reposo en el proyecto; escribir
//! criptografia propia aqui seria un error grave.
//!
//! ## Las dos versiones (RFC-0001, §702)
//!
//! - **`zkssl-keystore/2`**, la que se escribe: Argon2id con sal de 16 bytes
//!   y el coste de `zk_ssl::crypto::COSTE_V2`, los dos EN CLARO en el
//!   fichero (`kdf_sal`, `kdf_coste`), con el dominio `ZK-SSL-keystore-v2`.
//!   Un fichero de version 2 con otro coste no se abre.
//! - **`zkssl-keystore/1`**, la de antes: `SHA-256(ZK-SSL-keystore-v1 ‖
//!   frase)`, sin sal ni coste, atacable fuera de linea a la velocidad de un
//!   hash. Se sigue abriendo, y **abrirla con su frase la migra**: `load`
//!   reescribe el fichero en la version 2, con la misma frase y la misma
//!   clave de gasto, antes de devolver el wallet. Con otra frase no se toca.
//!
//! El fichero guarda EN CLARO el `public_id` (es publico por diseno) y
//! cifrado el material de gasto en los 32 bytes canonicos de
//! `store::digest_to_bytes`. Al cargar se verifica que la clave
//! descifrada DERIVA ese `public_id`: un fichero cambiado de sitio o
//! editado no pasa por wallet ajeno.

use chacha20poly1305::aead::{Aead, AeadCore, OsRng};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use serde::{Deserialize, Serialize};
use std::path::Path;
use zk_ssl::crypto::{Dominios, Kdf, COSTE_V2, LARGO_CABECERA, LARGO_SAL};
use zk_ssl_guardian::semilla::comprobar_permisos;
use zk_ssl::store::{digest_from_bytes, digest_to_bytes, StoreError};

use crate::Wallet;

/// El dominio de la version 1: solo se lee, para migrar.
const DOMINIO: &[u8] = b"ZK-SSL-keystore-v1";
/// §702: el dominio de la version 2, dato asociado de Argon2id.
const DOMINIO_V2: &[u8] = b"ZK-SSL-keystore-v2";
/// Los dominios del keystore, uno por version del KDF.
const DOMINIOS: Dominios = Dominios { v1: DOMINIO, v2: DOMINIO_V2 };
/// La version 1, de antes del §702: solo se lee.
const VERSION_V1: &str = "zkssl-keystore/1";
/// §702: la version que se escribe.
const VERSION: &str = "zkssl-keystore/2";
/// §702: el nombre del KDF de la version 2, tal cual en el fichero.
const KDF_V2: &str = "argon2id";

#[derive(Serialize, Deserialize)]
struct Fichero {
    version: String,
    kdf: String,
    /// §702: la sal del KDF, en hex. Solo en la version 2.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    kdf_sal: Option<String>,
    /// §702: el coste del KDF. Solo en la version 2.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    kdf_coste: Option<CosteFichero>,
    aead: String,
    public_id: String,
    sealed: String,
}

#[derive(Serialize, Deserialize)]
struct CosteFichero {
    m_kib: u32,
    t: u32,
    p: u32,
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
    // §650: sobre bytes, sin trocear el `&str` (un multibyte era un panico).
    zk_ssl_hash::bytes_de_hex(h).map_err(|e| anyhow::anyhow!("{e}"))
}

/// §702: el KDF que declara el fichero. La version 1 no declara nada (su
/// `kdf` era una descripcion); la 2 se recompone en la cabecera de la capa y
/// la lee `Kdf::de_cabecera`, el mismo juez que el libro: otro KDF, otro
/// coste o una sal que no es de 16 bytes no se abren.
fn kdf_del_fichero(f: &Fichero) -> anyhow::Result<Kdf> {
    if f.version == VERSION_V1 {
        return Ok(Kdf::Sha256V1);
    }
    anyhow::ensure!(f.version == VERSION, "version de keystore desconocida: {}", f.version);
    anyhow::ensure!(f.kdf == KDF_V2, "KDF de keystore desconocido: {}", f.kdf);
    let sal = des_hex(f.kdf_sal.as_deref().ok_or_else(|| anyhow::anyhow!("falta kdf_sal"))?)?;
    anyhow::ensure!(sal.len() == LARGO_SAL, "kdf_sal de {} bytes; son {LARGO_SAL}", sal.len());
    let c = f.kdf_coste.as_ref().ok_or_else(|| anyhow::anyhow!("falta kdf_coste"))?;
    let mut cabecera = Vec::with_capacity(LARGO_CABECERA);
    cabecera.push(2u8);
    for x in [c.m_kib, c.t, c.p] {
        cabecera.extend_from_slice(&x.to_le_bytes());
    }
    cabecera.extend_from_slice(&sal);
    Kdf::de_cabecera(&cabecera).map_err(|e| match e {
        StoreError::Malformed(m) => anyhow::anyhow!("{m}"),
        otro => anyhow::anyhow!("{otro}"),
    })
}

fn cifra(kdf: &Kdf, passphrase: &str) -> anyhow::Result<XChaCha20Poly1305> {
    kdf.cifra(passphrase.as_bytes(), &DOMINIOS).map_err(|e| anyhow::anyhow!("{e}"))
}

/// El fichero de la version 2, con una sal nueva.
fn fichero_v2(wallet: &Wallet, passphrase: &str) -> anyhow::Result<String> {
    let kdf = Kdf::nuevo();
    let sal = match kdf {
        Kdf::Argon2idV2 { sal } => sal,
        Kdf::Sha256V1 => anyhow::bail!("Kdf::nuevo no dio la version 2"),
    };
    let claro = digest_to_bytes(&wallet.spend_key());
    let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
    let ct = cifra(&kdf, passphrase)?
        .encrypt(&nonce, claro.as_slice())
        .map_err(|_| anyhow::anyhow!("fallo al cifrar"))?;
    let mut sealed = Vec::with_capacity(nonce.len() + ct.len());
    sealed.extend_from_slice(&nonce);
    sealed.extend_from_slice(&ct);

    let f = Fichero {
        version: VERSION.into(),
        kdf: KDF_V2.into(),
        kdf_sal: Some(hex(&sal)),
        kdf_coste: Some(CosteFichero { m_kib: COSTE_V2.m_kib, t: COSTE_V2.t, p: COSTE_V2.p }),
        aead: "xchacha20poly1305, nonce 24B aleatorio antepuesto".into(),
        public_id: hex(&digest_to_bytes(&wallet.public_id())),
        sealed: hex(&sealed),
    };
    Ok(serde_json::to_string_pretty(&f)? + "\n")
}

/// §702: escribe el fichero ENTERO o no lo toca. Se escribe al lado, con un
/// nombre propio del proceso, y se renombra encima: un corte a mitad deja el
/// keystore anterior, no uno truncado. En Unix nace con permisos 0600, y
/// despues del rename se sincroniza el directorio, que es donde vive el
/// nombre: sin eso, tras un corte de corriente el nombre podria volver al
/// keystore anterior aunque esta llamada hubiera dicho que si.
///
/// Un proceso que muere entre crear el temporal y renombrarlo lo deja en
/// disco: `<keystore>.escribiendo-<pid>`, con 0600 y el mismo contenido
/// cifrado que el keystore nuevo. Se puede borrar a mano; aqui solo lo quita
/// otra escritura del mismo fichero con el mismo pid.
fn escribir_entero(path: &Path, js: &str) -> anyhow::Result<()> {
    let nombre = path
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("{}: la ruta no nombra un fichero", path.display()))?;
    let mut tmp_nombre = nombre.to_os_string();
    tmp_nombre.push(format!(".escribiendo-{}", std::process::id()));
    let tmp = path.with_file_name(tmp_nombre);
    let r = (|| -> anyhow::Result<()> {
        use std::io::Write;
        // Uno que dejara un corte anterior de este mismo pid se quita: el
        // fichero se CREA aqui, con su modo, o no se escribe.
        let _ = std::fs::remove_file(&tmp);
        let mut o = std::fs::OpenOptions::new();
        o.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            o.mode(0o600);
        }
        let mut fh = o.open(&tmp)?;
        fh.write_all(js.as_bytes())?;
        fh.sync_all()?;
        drop(fh);
        std::fs::rename(&tmp, path)?;
        #[cfg(unix)]
        {
            // Una ruta sin directorio (`wallet.json`) es del directorio actual.
            let dir = match path.parent() {
                Some(d) if !d.as_os_str().is_empty() => d,
                _ => Path::new("."),
            };
            std::fs::File::open(dir).and_then(|d| d.sync_all()).map_err(|e| {
                anyhow::anyhow!(
                    "{}: renombrado, pero no se pudo sincronizar el directorio: {e}",
                    dir.display()
                )
            })?;
        }
        Ok(())
    })();
    if r.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    r
}

/// Guarda el wallet cifrado, en la version 2. En Unix, el fichero nace con
/// permisos 0600.
pub fn save(path: &Path, wallet: &Wallet, passphrase: &str) -> anyhow::Result<()> {
    escribir_entero(path, &fichero_v2(wallet, passphrase)?)
}

/// Carga el wallet. Falla con contrasena incorrecta, fichero manipulado,
/// o un fichero que no corresponde a su `public_id` declarado.
///
/// Falla tambien, y DICIENDO CUAL, si el fichero no se puede leer (5.A-403), y si sus
/// permisos lo dejan legible por grupo u otros (5.A-404): es material de clave, y esa
/// regla se comprueba al LEER, no al escribir (S199).
///
/// §702 (RFC-0001): **un keystore de la version 1 abierto con su frase se
/// migra**: se reescribe en la version 2, entero o nada, antes de devolver el
/// wallet. Si no se puede reescribir, falla y lo dice: un wallet que se abre
/// y se queda en la version 1 sin que nadie lo vea es lo que esto evita. Por
/// eso un keystore de la version 1 en un medio de solo lectura, o en un
/// directorio donde no se puede escribir, ya no se abre con ninguna frase:
/// se copia a un directorio escribible y se abre alli.
pub fn load(path: &Path, passphrase: &str) -> anyhow::Result<Wallet> {
    // 5.A-403: un lector que falla NOMBRA el fichero. Su hermana `leer_frase`, en el
    // mismo paso del cli, ya lo hacia; esta moria con un `os error 2` pelado.
    let crudo = std::fs::read_to_string(path).map_err(|e| {
        anyhow::anyhow!("{}: no se puede leer el keystore: {e}", path.display())
    })?;
    // 5.A-404: se comprueba al LEER, y la regla tiene UN productor, el guardian.
    comprobar_permisos(path).map_err(|e| anyhow::anyhow!("{e}"))?;
    let f: Fichero = serde_json::from_str(&crudo)
        .map_err(|e| anyhow::anyhow!("{}: no es un keystore: {e}", path.display()))?;
    let kdf = kdf_del_fichero(&f)?;

    let sealed = des_hex(&f.sealed)?;
    anyhow::ensure!(sealed.len() >= 24 + 32, "keystore demasiado corto");
    let (nonce_bytes, ct) = sealed.split_at(24);
    let claro = cifra(&kdf, passphrase)?
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
    if kdf == Kdf::Sha256V1 {
        escribir_entero(path, &fichero_v2(&wallet, passphrase)?).map_err(|e| {
            anyhow::anyhow!(
                "{}: keystore {VERSION_V1} abierto, pero no se pudo migrar a {VERSION}: {e}",
                path.display()
            )
        })?;
    }
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
        // El fichero conserva el 0600 que le puso `save`: `fs::write` sobre uno que ya
        // existe no cambia el modo, asi que la puerta de permisos (5.A-404) no se
        // dispara aqui y este testigo sigue probando lo suyo. Medido en el ENSAYO-550-r2;
        // si algun dia esto escribiera en una ruta NUEVA, habria que mirarlo.
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
        // El fichero conserva el 0600 que le puso `save`: `fs::write` sobre uno que ya
        // existe no cambia el modo, asi que la puerta de permisos (5.A-404) no se
        // dispara aqui y este testigo sigue probando lo suyo. Medido en el ENSAYO-550-r2;
        // si algun dia esto escribiera en una ruta NUEVA, habria que mirarlo.
        assert!(load(&p, "clave").is_err(), "CRITICO: public_id ajeno acepto");
        let _ = std::fs::remove_file(&p);
    }

    /// **DOMINIOS SEPARADOS: la clave del ledger NO abre el keystore.**
    ///
    /// Misma contrasena, dominios distintos => claves distintas. Si este
    /// test fallara, ledger y wallet compartirian clave simetrica.
    ///
    /// §702: en las DOS versiones, y en la 2 con la MISMA sal y el mismo
    /// coste que el keystore, para que lo unico distinto sea el dominio. Y con
    /// el testigo de vida: los dominios del keystore si lo abren, asi que el
    /// rojo de la del ledger no viene de un montaje roto.
    #[test]
    fn la_clave_del_ledger_no_abre_el_keystore() {
        let p = tmp("dom");
        let v1 = keystore_v1_en(&p);
        let w = Wallet::random();
        let p2 = tmp("dom2");
        save(&p2, &w, FRASE_V1).expect("guardar");
        let v2: Fichero = serde_json::from_str(&std::fs::read_to_string(&p2).unwrap()).unwrap();
        for f in [&v1, &v2] {
            let kdf = kdf_del_fichero(f).expect("kdf del fichero");
            let sealed = des_hex(&f.sealed).unwrap();
            let ledger = zk_ssl::crypto::LedgerKey::derivar(FRASE_V1, &kdf).expect("derivar");
            assert!(
                ledger.open(&sealed).is_err(),
                "CRITICO: el dominio del ledger abrio el keystore {}",
                f.version
            );
            let (n, ct) = sealed.split_at(24);
            assert!(
                cifra(&kdf, FRASE_V1).unwrap().decrypt(XNonce::from_slice(n), ct).is_ok(),
                "los dominios del keystore no abren su propio fichero {}",
                f.version
            );
        }
        let _ = std::fs::remove_file(&p);
        let _ = std::fs::remove_file(&p2);
    }

    /// **5.A-403: un lector que falla dice QUE fichero.** Antes moria con `os error 2` pelado.
    #[test]
    fn un_keystore_que_no_existe_dice_su_ruta() {
        let p = tmp("nohay");
        let _ = std::fs::remove_file(&p);
        // Sin `expect_err`: exige `Wallet: Debug`, y el struct que guarda la clave de gasto
        // NO se imprime. El compilador sugiere derivarlo; aqui eso es que no.
        let e = match load(&p, "clave") {
            Ok(_) => panic!("CRITICO: abrio un keystore que no existe"),
            Err(e) => e,
        };
        let msg = format!("{e}");
        assert!(msg.contains(&p.display().to_string()), "el error no nombra el fichero: {msg}");
        assert!(
            msg.contains("no se puede leer el keystore"),
            "el error no dice de que hablaba: {msg}"
        );
    }

    /// **5.A-404: material de clave legible por otros NO se abre.** Con 0600 SI abre: el mismo
    /// testigo prueba los dos lados, para que un verde no pueda venir de que la puerta no este.
    #[cfg(unix)]
    #[test]
    fn un_keystore_legible_por_otros_no_se_abre() {
        use std::os::unix::fs::PermissionsExt;
        let w = Wallet::random();
        let p = tmp("modo");
        save(&p, &w, "clave").expect("guardar");
        load(&p, "clave").expect("con 0600 tiene que abrir");
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o644)).unwrap();
        let e = match load(&p, "clave") {
            Ok(_) => panic!("CRITICO: abrio un keystore legible por otros"),
            Err(e) => e,
        };
        let msg = format!("{e}");
        assert!(msg.contains("0644"), "el error no dice el modo: {msg}");
        assert!(msg.contains("chmod 600"), "el error no dice como arreglarlo: {msg}");
        let _ = std::fs::remove_file(&p);
    }

    /// §702: un keystore de la version 1 escrito por el codigo de `590caae` (el §701), con
    /// `Wallet::from_elements([1, 2, 3, 4])` y la frase `FRASE_V1`, tal cual salio de
    /// `keystore::save`. Fuera del arbol se escribio con ese codigo; aqui va literal.
    const KEYSTORE_V1_DE_590CAAE: &str = r#"{
  "version": "zkssl-keystore/1",
  "kdf": "sha256(ZK-SSL-keystore-v1 || passphrase) — ver advertencia en zk-ssl::crypto",
  "aead": "xchacha20poly1305, nonce 24B aleatorio antepuesto",
  "public_id": "0x1df5b4432d786ae5ccbd02d559b1ffaa3489f2bdd842a33a41356b1df653e0b7",
  "sealed": "0xf8221312baa0fde209f6c6bbe424784ba26e70fead828060a82c62cb3b141c8c16a9bbf6d53ce24bf6f370aa8137410a3eec6d8f8b370cd742580738c1675e8590d0d584ddf82ba5"
}
"#;
    const FRASE_V1: &str = "la frase de la foto v1";

    /// Escribe la foto v1 en `p`, con el 0600 que exige el lector, y la devuelve leida.
    fn keystore_v1_en(p: &std::path::Path) -> Fichero {
        escribir_entero(p, KEYSTORE_V1_DE_590CAAE).expect("escribir la foto v1");
        serde_json::from_str(KEYSTORE_V1_DE_590CAAE).unwrap()
    }

    /// **§702: lo que se escribe es la version 2, con sal y coste en claro, y cada vez su sal.**
    /// El mismo wallet con la misma frase, guardado dos veces, da dos sales y dos sellos.
    #[test]
    fn guardar_escribe_la_version_2_con_su_sal_y_su_coste() {
        let w = Wallet::random();
        let (p, q) = (tmp("v2a"), tmp("v2b"));
        save(&p, &w, "clave").expect("guardar");
        save(&q, &w, "clave").expect("guardar");
        let a: Fichero = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
        let b: Fichero = serde_json::from_str(&std::fs::read_to_string(&q).unwrap()).unwrap();
        assert_eq!((a.version.as_str(), a.kdf.as_str()), ("zkssl-keystore/2", "argon2id"));
        let c = a.kdf_coste.as_ref().expect("lleva el coste");
        assert_eq!((c.m_kib, c.t, c.p), (65_536, 3, 4));
        assert_eq!(des_hex(a.kdf_sal.as_ref().expect("lleva la sal")).unwrap().len(), 16);
        assert_ne!(a.kdf_sal, b.kdf_sal, "dos keystores con la misma sal");
        assert_ne!(a.sealed, b.sealed);
        assert_eq!(load(&q, "clave").expect("abre").public_id(), w.public_id());
        let _ = std::fs::remove_file(&p);
        let _ = std::fs::remove_file(&q);
    }

    /// **§702: el keystore v1 que escribio el codigo de `590caae` se abre con su frase y queda
    /// en la version 2.** El wallet es el de la foto; el fichero, despues, es otro y de la
    /// version 2; y se vuelve a abrir, ya por Argon2id, con la misma frase y el mismo wallet.
    #[test]
    fn un_keystore_v1_de_590caae_se_abre_y_queda_migrado() {
        let p = tmp("migra");
        keystore_v1_en(&p);
        let esperado = Wallet::from_elements([1, 2, 3, 4]).public_id();
        assert_eq!(load(&p, FRASE_V1).expect("abre la v1").public_id(), esperado);
        let despues = std::fs::read_to_string(&p).unwrap();
        assert_ne!(despues, KEYSTORE_V1_DE_590CAAE, "abrir la v1 no la migro");
        let f: Fichero = serde_json::from_str(&despues).unwrap();
        assert_eq!(f.version, "zkssl-keystore/2");
        assert_eq!(kdf_del_fichero(&f).unwrap().version(), 2);
        assert_eq!(load(&p, FRASE_V1).expect("abre la v2").public_id(), esperado);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let modo = std::fs::metadata(&p).unwrap().permissions().mode() & 0o777;
            assert_eq!(modo, 0o600, "la migracion dejo el keystore con {modo:o}");
        }
        let _ = std::fs::remove_file(&p);
    }

    /// **§702: con otra frase, la v1 no se abre y NO SE TOCA**: el fichero queda byte a byte.
    #[test]
    fn un_keystore_v1_con_otra_frase_no_se_toca() {
        let p = tmp("noto");
        keystore_v1_en(&p);
        assert!(load(&p, "otra frase").is_err(), "CRITICO: otra frase abrio la v1");
        assert_eq!(std::fs::read_to_string(&p).unwrap(), KEYSTORE_V1_DE_590CAAE);
        let _ = std::fs::remove_file(&p);
    }

    /// **§702: lo que la version 2 declara se comprueba.** Otro coste -aunque sea mayor-, otro
    /// KDF, una sal de otro largo o sin sal no se abren, y cada uno dice que; una sal cambiada en
    /// un bit deriva otra clave y no abre.
    #[test]
    fn un_v2_con_otro_kdf_otro_coste_u_otra_sal_no_abre() {
        let w = Wallet::random();
        let p = tmp("decl");
        save(&p, &w, "clave").expect("guardar");
        let bueno = std::fs::read_to_string(&p).unwrap();
        let prueba = |cambia: &dyn Fn(&mut Fichero), dice: &str| {
            let mut f: Fichero = serde_json::from_str(&bueno).unwrap();
            cambia(&mut f);
            std::fs::write(&p, serde_json::to_string(&f).unwrap()).unwrap();
            let e = match load(&p, "clave") {
                Ok(_) => panic!("CRITICO: abrio un keystore cambiado ({dice})"),
                Err(e) => format!("{e}"),
            };
            assert!(e.contains(dice), "el error no dice {dice}: {e}");
        };
        prueba(&|f| f.kdf_coste.as_mut().unwrap().t = 4, "coste del KDF");
        prueba(&|f| f.kdf_coste.as_mut().unwrap().m_kib = 19_456, "coste del KDF");
        prueba(&|f| f.kdf = "scrypt".into(), "KDF de keystore desconocido");
        prueba(&|f| f.kdf_sal = Some("0x00".into()), "kdf_sal de 1 bytes");
        prueba(&|f| f.kdf_sal = None, "falta kdf_sal");
        prueba(&|f| f.kdf_coste = None, "falta kdf_coste");
        prueba(
            &|f| {
                let mut s = des_hex(f.kdf_sal.as_ref().unwrap()).unwrap();
                s[0] ^= 1;
                f.kdf_sal = Some(hex(&s));
            },
            "contrasena incorrecta o fichero manipulado",
        );
        // Y el fichero bueno, en su sitio, abre: el rojo era por el cambio.
        std::fs::write(&p, &bueno).unwrap();
        assert_eq!(load(&p, "clave").expect("el bueno abre").public_id(), w.public_id());
        let _ = std::fs::remove_file(&p);
    }

    /// **§702: los KAT de la clave del keystore, de las dos versiones**, calculados FUERA de
    /// Rust: la v2 con la `libargon2` de referencia (argon2-cffi-bindings 26.1.0) y el dominio
    /// `ZK-SSL-keystore-v2` como dato asociado; la v1 con `hashlib`, `sha256(ZK-SSL-keystore-v1
    /// ‖ frase)`, la formula de antes del §702. Frase `la frase del KAT`, sal `00 01 .. 0f`.
    #[test]
    fn la_clave_del_keystore_es_la_de_la_referencia() {
        let frase = b"la frase del KAT";
        let mut sal = [0u8; LARGO_SAL];
        for (i, b) in sal.iter_mut().enumerate() {
            *b = i as u8;
        }
        let v2 = Kdf::Argon2idV2 { sal }.clave(frase, &DOMINIOS).unwrap();
        assert_eq!(
            hex(&v2[..]),
            "0xedf21e3f7452c0f6237bfa1d589426c1a7cef35db9dd7beb5d95d5152d3794df"
        );
        let v1 = Kdf::Sha256V1.clave(frase, &DOMINIOS).unwrap();
        assert_eq!(
            hex(&v1[..]),
            "0x91674eb08f79c74b2844cf2975752b67beb96047bace47f3841f4d6bb12cb881"
        );
    }
}
