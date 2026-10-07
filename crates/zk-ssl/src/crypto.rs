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
//! - La clave se deriva de la contraseña con **Argon2id** (RFC 9106), con
//!   sal y coste: ver «La derivación», abajo.
//!
//! ## La derivación de la clave (RFC-0001, §702)
//!
//! Hasta el §702 la clave era `SHA-256(dominio ‖ frase)`: sin sal y sin
//! coste. Quien robara el disco podía probar frases fuera de línea a la
//! velocidad de un hash, y una misma tabla servía para todos los libros.
//! Desde el §702 cada contenedor cifrado —el libro en `sled`, la
//! instantánea, el keystore del SDK— lleva **en claro** su [`Kdf`]: la
//! versión, el coste y una sal de 16 bytes del generador del sistema. La sal
//! y el coste no son secretos; sin ellos no se puede derivar.
//!
//! | versión | derivación | se escribe | se lee |
//! |---|---|---|---|
//! | 1 | `SHA-256(dominio_v1 ‖ frase)` | no | sí: para migrar |
//! | 2 | Argon2id v0x13, `P` = frase, `S` = sal (16 B), `X` = dominio_v2, `t` = 3, `m` = 64 MiB, `p` = 4, `T` = 32 B | sí | sí |
//!
//! El coste de la versión 2 es la segunda opción recomendada del RFC 9106
//! (§4): la que `argon2.profiles.RFC_9106_LOW_MEMORY` de argon2-cffi 25.1.0
//! toma por defecto. Cuesta unos 0,4 s por derivación en `--release`, con la
//! máquina cargada (medido en el §702), una vez por apertura: el nonce sigue
//! siendo uno por escritura. Un fichero de versión 2 con otro coste NO se abre: cambiar el
//! coste es una versión nueva, no un campo libre.
//!
//! El dominio va como dato asociado `X` del RFC 9106: la clave del libro y
//! la del keystore no coinciden aunque compartan frase y sal.
//!
//! ⚠️ **Lo que el KDF no cambia.** Una frase débil sigue siendo débil: el
//! coste multiplica el trabajo de cada intento, no el número de intentos. Y
//! quien tenga el proceso en marcha tiene la clave derivada en memoria.

use argon2::{Algorithm, Argon2, AssociatedData, Block, ParamsBuilder, Version};
use chacha20poly1305::aead::rand_core::RngCore;
use chacha20poly1305::aead::{Aead, KeyInit, OsRng};
use chacha20poly1305::{AeadCore, XChaCha20Poly1305, XNonce};
use sha2::{Digest as _, Sha256};
use zeroize::Zeroizing;

use crate::store::StoreError;

/// **Dominio de la clave del ledger en reposo** (registro de dominios:
/// `zk-ssl-hash/src/lib.rs`, §286). Era el unico literal `ZK-SSL-` suelto
/// del arbol. El keystore del SDK usa `ZK-SSL-keystore-v1`, y su test
/// exige que la clave de uno no abra al otro: la separacion que estas
/// cadenas garantizan se comprueba, no se supone.
///
/// §702: es el de la VERSION 1 del KDF, que solo se lee para migrar.
const DOMINIO_CLAVE_LEDGER: &[u8] = b"ZK-SSL-ledger-key-v1";

/// §702 (RFC-0001): el dominio de la clave del libro en la VERSION 2, el
/// dato asociado `X` de Argon2id. El keystore tiene el suyo,
/// `ZK-SSL-keystore-v2`, declarado en el SDK.
const DOMINIO_CLAVE_LEDGER_V2: &[u8] = b"ZK-SSL-ledger-key-v2";

/// Los dos dominios de un uso de la clave, uno por version del KDF. Cada
/// llamante declara los suyos: la capa, [`DOMINIOS_LIBRO`]; el keystore del
/// SDK, los de `keystore.rs`.
#[derive(Clone, Copy, Debug)]
pub struct Dominios {
    /// El de la version 1, prefijo de la frase en SHA-256.
    pub v1: &'static [u8],
    /// El de la version 2, dato asociado de Argon2id (hasta 32 bytes).
    pub v2: &'static [u8],
}

/// Los dominios de la clave del libro en reposo.
pub const DOMINIOS_LIBRO: Dominios =
    Dominios { v1: DOMINIO_CLAVE_LEDGER, v2: DOMINIO_CLAVE_LEDGER_V2 };

/// El coste de Argon2id: memoria en KiB, pasadas y carriles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Coste {
    pub m_kib: u32,
    pub t: u32,
    pub p: u32,
}

/// **El coste de la version 2**: la segunda opcion recomendada del RFC 9106,
/// 64 MiB, tres pasadas y cuatro carriles. Es el UNICO que se lee en v2.
pub const COSTE_V2: Coste = Coste { m_kib: 65_536, t: 3, p: 4 };

/// La sal de la version 2: 16 bytes, los que recomienda el RFC 9106.
pub const LARGO_SAL: usize = 16;

/// La cabecera de la version 2: version (1 B), `m_kib`, `t` y `p` (u32 LE
/// cada uno) y la sal.
pub const LARGO_CABECERA: usize = 1 + 4 + 4 + 4 + LARGO_SAL;

/// **El KDF de un contenedor cifrado, con su version** (RFC-0001, §702).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kdf {
    /// Version 1, la de antes del §702: `SHA-256(dominio ‖ frase)`, sin sal
    /// ni coste. No lleva cabecera: es lo que hay cuando no hay cabecera. Se
    /// lee para MIGRAR; nada la escribe.
    Sha256V1,
    /// Version 2: Argon2id con [`COSTE_V2`] y su sal.
    Argon2idV2 {
        sal: [u8; LARGO_SAL],
    },
}

impl Kdf {
    /// Un KDF de version 2 con sal nueva, del generador del sistema.
    pub fn nuevo() -> Self {
        let mut sal = [0u8; LARGO_SAL];
        OsRng.fill_bytes(&mut sal);
        Kdf::Argon2idV2 { sal }
    }

    /// 1 o 2.
    pub fn version(&self) -> u8 {
        match self {
            Kdf::Sha256V1 => 1,
            Kdf::Argon2idV2 { .. } => 2,
        }
    }

    /// La cabecera que se guarda en claro junto a lo cifrado. La version 1
    /// no tiene: su ausencia ES la version 1.
    pub fn cabecera(&self) -> Option<[u8; LARGO_CABECERA]> {
        match self {
            Kdf::Sha256V1 => None,
            Kdf::Argon2idV2 { sal } => {
                let mut c = [0u8; LARGO_CABECERA];
                c[0] = 2;
                c[1..5].copy_from_slice(&COSTE_V2.m_kib.to_le_bytes());
                c[5..9].copy_from_slice(&COSTE_V2.t.to_le_bytes());
                c[9..13].copy_from_slice(&COSTE_V2.p.to_le_bytes());
                c[13..].copy_from_slice(sal);
                Some(c)
            }
        }
    }

    /// Lee una cabecera. Rechaza, diciendo por que, lo que no es
    /// EXACTAMENTE una cabecera de version 2 con [`COSTE_V2`]: otro largo,
    /// otra version u otro coste.
    pub fn de_cabecera(b: &[u8]) -> Result<Self, StoreError> {
        if b.len() != LARGO_CABECERA {
            return Err(StoreError::Malformed(format!(
                "cabecera del KDF de {} bytes; la de la version 2 tiene {LARGO_CABECERA}",
                b.len()
            )));
        }
        if b[0] != 2 {
            return Err(StoreError::Malformed(format!(
                "cabecera del KDF de version {}; solo la 2 lleva cabecera",
                b[0]
            )));
        }
        let u = |i: usize| u32::from_le_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]);
        let coste = Coste { m_kib: u(1), t: u(5), p: u(9) };
        if coste != COSTE_V2 {
            return Err(StoreError::Malformed(format!(
                "coste del KDF {coste:?}; la version 2 es {COSTE_V2:?} y no se lee otro"
            )));
        }
        let mut sal = [0u8; LARGO_SAL];
        sal.copy_from_slice(&b[13..]);
        Ok(Kdf::Argon2idV2 { sal })
    }

    /// **La clave de 32 bytes**, con el dominio de esta version. Se borra al
    /// soltarse. Es lo que fijan los KAT; para cifrar, [`Kdf::cifra`].
    pub fn clave(
        &self,
        frase: &[u8],
        dominios: &Dominios,
    ) -> Result<Zeroizing<[u8; 32]>, StoreError> {
        let mut clave = Zeroizing::new([0u8; 32]);
        match self {
            Kdf::Sha256V1 => {
                let mut h = Sha256::new();
                h.update(dominios.v1);
                h.update(frase);
                clave.copy_from_slice(&h.finalize());
            }
            Kdf::Argon2idV2 { sal } => {
                let kdf = |e: argon2::Error| StoreError::Malformed(format!("argon2id: {e}"));
                let params = ParamsBuilder::new()
                    .m_cost(COSTE_V2.m_kib)
                    .t_cost(COSTE_V2.t)
                    .p_cost(COSTE_V2.p)
                    .output_len(32)
                    .data(AssociatedData::new(dominios.v2).map_err(kdf)?)
                    .build()
                    .map_err(kdf)?;
                // La memoria la reserva esto y no el crate (sin `alloc`): asi
                // se borra al soltarse, que la del crate no se borra.
                let mut memoria = Zeroizing::new(vec![Block::default(); params.block_count()]);
                Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
                    .hash_password_into_with_memory(frase, sal, &mut clave[..], &mut memoria[..])
                    .map_err(kdf)?;
            }
        }
        Ok(clave)
    }

    /// El cifrado autenticado con la clave de [`Kdf::clave`].
    pub fn cifra(&self, frase: &[u8], dominios: &Dominios) -> Result<XChaCha20Poly1305, StoreError> {
        let clave = self.clave(frase, dominios)?;
        Ok(XChaCha20Poly1305::new((&clave[..]).into()))
    }
}

/// Clave de cifrado del ledger.
///
/// No se serializa ni se guarda: vive solo en memoria mientras el nodo
/// está en marcha.
///
/// §702 (RFC-0001): nace de la FRASE ([`LedgerKey::from_passphrase`]) y no
/// cifra hasta que se FIJA a un [`Kdf`], porque la sal vive en el
/// contenedor: `open_encrypted` la fija con la cabecera del libro, y
/// `import_snapshot_with_key` con la de la instantánea. Fijada, ya no lleva
/// la frase.
#[derive(Clone)]
pub struct LedgerKey {
    frase: Option<Zeroizing<String>>,
    fijada: Option<(Kdf, XChaCha20Poly1305)>,
}

impl std::fmt::Debug for LedgerKey {
    /// No imprime el material de la clave.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LedgerKey(<oculta>)")
    }
}

impl LedgerKey {
    /// La clave de una contraseña, SIN derivar todavía.
    ///
    /// §702: hasta aquí derivaba con SHA-256, que **no es una función de
    /// derivación de contraseñas**. Ahora guarda la frase, que se borra al
    /// soltarse, y la deriva con Argon2id cuando se fija a la sal de un
    /// contenedor ([`LedgerKey::fijar`]).
    pub fn from_passphrase(passphrase: &str) -> Self {
        Self { frase: Some(Zeroizing::new(passphrase.to_string())), fijada: None }
    }

    /// La clave ya fijada a `kdf`, sin la frase.
    pub fn derivar(passphrase: &str, kdf: &Kdf) -> Result<Self, StoreError> {
        let cifra = kdf.cifra(passphrase.as_bytes(), &DOMINIOS_LIBRO)?;
        Ok(Self { frase: None, fijada: Some((*kdf, cifra)) })
    }

    /// **Fija la clave a un KDF**: la deriva de la frase con su sal. Una
    /// clave ya fijada a ese mismo KDF se devuelve tal cual; fijada a otro y
    /// sin frase, no puede derivar, y lo dice.
    pub fn fijar(&self, kdf: &Kdf) -> Result<Self, StoreError> {
        match (&self.fijada, &self.frase) {
            (Some((k, c)), _) if k == kdf => Ok(Self { frase: None, fijada: Some((*k, c.clone())) }),
            (_, Some(f)) => Self::derivar(f, kdf),
            _ => Err(StoreError::Malformed(
                "la clave del libro esta fijada a otro KDF y ya no lleva la frase".into(),
            )),
        }
    }

    /// El KDF al que está fijada, o `None` si todavía no lo está.
    pub fn kdf(&self) -> Option<Kdf> {
        self.fijada.as_ref().map(|(k, _)| *k)
    }

    fn cifra(&self) -> Result<&XChaCha20Poly1305, StoreError> {
        self.fijada.as_ref().map(|(_, c)| c).ok_or_else(|| {
            StoreError::Malformed(
                "clave del libro sin fijar a un KDF: la sal la da el contenedor (RFC-0001)".into(),
            )
        })
    }

    /// Cifra un valor. El nonce va delante del texto cifrado.
    pub fn seal(&self, plaintext: &[u8]) -> Result<Vec<u8>, StoreError> {
        let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
        let ct = self
            .cifra()?
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
        let cifra = self.cifra()?;
        if sealed.len() < 24 {
            return Err(StoreError::Malformed(
                "dato cifrado demasiado corto".into(),
            ));
        }
        let (nonce_bytes, ct) = sealed.split_at(24);
        let nonce = XNonce::from_slice(nonce_bytes);
        cifra.decrypt(nonce, ct).map_err(|_| {
            StoreError::IntegrityFailure {
                what: "dato cifrado: contrasena incorrecta o manipulacion",
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Una clave fijada a una sal nueva: lo que hace `open_encrypted` con un
    /// libro nuevo.
    fn clave(frase: &str) -> LedgerKey {
        LedgerKey::from_passphrase(frase).fijar(&Kdf::nuevo()).expect("fijar")
    }

    fn hex(b: &[u8]) -> String {
        b.iter().map(|x| format!("{x:02x}")).collect()
    }

    /// Ida y vuelta con la contraseña correcta.
    #[test]
    fn seal_and_open_roundtrip() {
        let k = clave("una contrasena");
        let msg = b"saldo: 1000000";
        let sealed = k.seal(msg).expect("cifrar");
        assert_ne!(&sealed[24..], msg, "el texto cifrado no debe ser el claro");
        assert_eq!(k.open(&sealed).expect("descifrar"), msg);
    }

    /// **Con otra contraseña no se puede leer.**
    ///
    /// §702: con la MISMA sal, para que lo que distinga sea la frase.
    #[test]
    fn a_wrong_passphrase_cannot_read() {
        let kdf = Kdf::nuevo();
        let good = LedgerKey::derivar("correcta", &kdf).expect("derivar");
        let bad = LedgerKey::derivar("incorrecta", &kdf).expect("derivar");
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
        let k = clave("clave");
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
        let k = clave("clave");
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
        let k = clave("clave");
        assert!(k.open(&[0u8; 10]).is_err());
    }

    /// La clave no se imprime en los diagnósticos.
    #[test]
    fn the_key_is_not_printed() {
        let k = LedgerKey::from_passphrase("secreto-que-no-debe-aparecer");
        let s = format!("{k:?}");
        assert!(!s.contains("secreto"), "la clave no debe aparecer en {s}");
        let s = format!("{:?}", k.fijar(&Kdf::nuevo()).expect("fijar"));
        assert!(!s.contains("secreto"), "la clave fijada no debe aparecer en {s}");
    }

    /// **§702: el crate clavado reproduce el vector de Argon2id del RFC 9106**
    /// (§5.3: contraseña 32 x 0x01, sal 16 x 0x02, secreto 8 x 0x03, dato
    /// asociado 12 x 0x04, t = 3, m = 32 KiB, p = 4). La etiqueta es la del
    /// fichero `kats/argon2id` de la implementacion de referencia
    /// (P-H-C/phc-winner-argon2), linea 12304.
    #[test]
    fn argon2_reproduce_el_vector_del_rfc_9106() {
        let params = ParamsBuilder::new()
            .m_cost(32)
            .t_cost(3)
            .p_cost(4)
            .output_len(32)
            .data(AssociatedData::new(&[0x04; 12]).unwrap())
            .build()
            .unwrap();
        let a = Argon2::new_with_secret(&[0x03; 8], Algorithm::Argon2id, Version::V0x13, params)
            .unwrap();
        let mut out = [0u8; 32];
        let mut memoria = vec![Block::default(); 32];
        a.hash_password_into_with_memory(&[0x01; 32], &[0x02; 16], &mut out, &mut memoria[..])
            .unwrap();
        assert_eq!(
            hex(&out),
            "0d640df58d78766c08c037a34a8b53c9d01ef0452d75b65eb52520e96b01e659"
        );
    }

    /// **§702: los KAT de la clave del libro, de las dos versiones.** La v2,
    /// con la frase `la frase del KAT` y la sal `00 01 .. 0f`, la calculo
    /// FUERA de Rust la implementacion de referencia en C (la `libargon2` de
    /// argon2-cffi-bindings 26.1.0) con el dominio como dato asociado; la v1
    /// es `sha256(ZK-SSL-ledger-key-v1 ‖ frase)` de `hashlib`, la formula de
    /// antes del §702: si se moviera, un libro viejo dejaria de abrir.
    #[test]
    fn la_clave_del_libro_es_la_de_la_referencia() {
        let frase = b"la frase del KAT";
        let mut sal = [0u8; LARGO_SAL];
        for (i, b) in sal.iter_mut().enumerate() {
            *b = i as u8;
        }
        let v2 = Kdf::Argon2idV2 { sal }.clave(frase, &DOMINIOS_LIBRO).unwrap();
        assert_eq!(
            hex(&v2[..]),
            "a74cb56e8665090375ee41b8d8bd4c5f653239c189ea109a2cffd89fb8a33c68"
        );
        let v1 = Kdf::Sha256V1.clave(frase, &DOMINIOS_LIBRO).unwrap();
        assert_eq!(
            hex(&v1[..]),
            "8abf298a5ba883a0c27342cbbb39dd0cf2b783441113062c180d97876ae43027"
        );
    }

    /// **La cabecera va y vuelve, y no se lee nada que no sea v2 con su
    /// coste.** Cada rechazo dice por que.
    #[test]
    fn la_cabecera_va_y_vuelve_y_rechaza_lo_que_no_es_v2() {
        let kdf = Kdf::nuevo();
        let c = kdf.cabecera().expect("v2 lleva cabecera");
        assert_eq!(Kdf::de_cabecera(&c).expect("leer"), kdf);
        assert_eq!(Kdf::Sha256V1.cabecera(), None, "la v1 no lleva cabecera");

        let rojo = |b: &[u8]| match Kdf::de_cabecera(b) {
            Err(StoreError::Malformed(m)) => m,
            otro => panic!("tenia que rechazarse: {otro:?}"),
        };
        assert!(rojo(&c[..LARGO_CABECERA - 1]).contains("28 bytes"));
        let mut v = c;
        v[0] = 1;
        assert!(rojo(&v).contains("version 1"));
        v[0] = 3;
        assert!(rojo(&v).contains("version 3"));
        // Un coste distinto, aunque sea MAYOR, no se lee: es otra version.
        for (desde, nuevo) in [(1usize, 65_537u32), (5, 2), (9, 1)] {
            let mut v = c;
            v[desde..desde + 4].copy_from_slice(&nuevo.to_le_bytes());
            assert!(rojo(&v).contains("coste del KDF"), "coste en {desde}");
        }
    }

    /// **Cada libro su sal, y una clave sin fijar no cifra.** Dos KDF
    /// nuevos no comparten sal, y con la misma frase dan claves distintas:
    /// lo que cifra una no lo abre la otra.
    #[test]
    fn cada_libro_su_sal_y_sin_fijar_no_cifra() {
        let (a, b) = (Kdf::nuevo(), Kdf::nuevo());
        assert_ne!(a, b, "dos sales nuevas iguales");
        let ka = LedgerKey::derivar("la misma", &a).unwrap();
        let kb = LedgerKey::derivar("la misma", &b).unwrap();
        assert!(kb.open(&ka.seal(b"saldo").unwrap()).is_err(), "otra sal abrio");

        let sin = LedgerKey::from_passphrase("la misma");
        assert_eq!(sin.kdf(), None);
        assert!(sin.seal(b"saldo").is_err(), "una clave sin fijar cifro");
        let fijada = sin.fijar(&a).unwrap();
        assert_eq!(fijada.kdf(), Some(a));
        assert_eq!(fijada.open(&ka.seal(b"saldo").unwrap()).unwrap(), b"saldo");
        // Fijada, ya no lleva la frase: a su KDF vuelve; a otro, no.
        assert!(fijada.fijar(&a).is_ok());
        assert!(fijada.fijar(&b).is_err(), "derivo sin frase");
    }

    /// **§702: `argon2` entra clavada con `=` y solo por la capa.** En el
    /// lock, un solo `argon2`, el 0.5.3 de crates.io; en el manifiesto de la
    /// capa, `=0.5.3`; y ningun otro manifiesto del workspace la declara: el
    /// keystore del SDK deriva por `Kdf`, una sola ley de reposo. Que el kit
    /// no la arrastra lo dice su lista cerrada (§694), que este sello no toca.
    #[test]
    fn argon2_entra_clavada_y_solo_por_la_capa() {
        let lock = include_str!("../../../Cargo.lock");
        let bloques: Vec<&str> = lock
            .split("[[package]]")
            .filter(|b| b.contains("\nname = \"argon2\"\n"))
            .collect();
        assert_eq!(bloques.len(), 1, "argon2 en el lock {} veces", bloques.len());
        assert!(bloques[0].contains("\nversion = \"0.5.3\"\n"), "argon2 no es la 0.5.3");
        assert!(bloques[0].contains("source = \"registry+https://github.com/rust-lang/crates.io-index\""));

        let raiz = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
        let ws = std::fs::read_to_string(format!("{raiz}/Cargo.toml")).unwrap();
        let miembros: Vec<&str> = ws
            .lines()
            .map(str::trim)
            .filter_map(|l| l.strip_prefix('"').and_then(|l| l.strip_suffix("\",")))
            .filter(|l| l.starts_with("crates/"))
            .collect();
        assert!(miembros.len() >= 20, "no se leyeron los miembros: {miembros:?}");
        let mut declaran = Vec::new();
        for m in &miembros {
            let t = std::fs::read_to_string(format!("{raiz}/{m}/Cargo.toml")).unwrap();
            for l in t.lines().filter(|l| l.trim_start().starts_with("argon2")) {
                declaran.push((*m, l.trim().to_string()));
            }
        }
        assert_eq!(
            declaran,
            [(
                "crates/zk-ssl",
                "argon2 = { version = \"=0.5.3\", default-features = false, features = [\"zeroize\"] }"
                    .to_string()
            )],
            "argon2 la declara alguien mas, o sin `=`"
        );
    }
}
