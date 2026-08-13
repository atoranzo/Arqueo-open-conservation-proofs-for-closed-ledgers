//! # `zk-ssl-verify` — verificar una cabeza de época **sin el operador**
//!
//! Todo lo que un **tercero** necesita para comprobar que una cabeza de
//! época firmada la emitió quien dice. Nada más.
//!
//! ## ⚠️ Por qué es un crate aparte
//!
//! Hasta §243 la verificación vivía dentro del **binario del nodo**, junto a
//! `tokio` y `axum`. Eso significaba que **la única forma de verificar una
//! cabeza era compilar el código del operador** — exactamente la dependencia
//! que este aparato existe para eliminar.
//!
//! > La diferencia es entre *«publicamos algo verificable»* y *«publicamos
//! > algo verificable si te tragas nuestro servidor»*.
//!
//! ⚠️ **La dependencia va en UN SOLO SENTIDO.** Este crate **no depende de
//! la capa, ni del nodo, ni del cable**: solo de `xmss`. Si algún día
//! importa algo del proyecto, se habrá vuelto a caer en el problema.
//!
//! ⚠️ Y debería ser **el crate que menos cambie**: su superficie es el
//! dominio, la versión de formato y [`verificar_cabeza`]. Un pin que no se
//! mueve casi no cuesta.
//!
//! ## ⚠️ Lo que este crate NO da
//!
//! **Que exista un verificador independiente no hace las firmas oponibles.**
//! Sigue faltando **la custodia declarada de la clave** del operador: sin
//! ella, una firma no tiene valor probatorio.
//!
//! Esto hace **posible verificar**; no hace **válido lo verificado**.
//!
//! ## Uso
//!
//! ```no_run
//! use zk_ssl_verify::{verificar_cabeza, CabezaFirmada};
//! # let (clave, digest, firma) = (vec![], [0u8; 32], vec![]);
//! let c = CabezaFirmada { version_formato: 1, indice: 42, firma };
//! verificar_cabeza(&clave, &digest, &c)?;
//! # Ok::<(), zk_ssl_verify::VerificaError>(())
//! ```

use xmss::{Signature, VerifyingKey, XmssMtSha2_40_8_256};

// ⚠️ §256: la INCLUSION, el segundo eslabon que un tercero puede
//    comprobar sin el nodo. `verificar_cabeza` dice *quien* firmo; esto
//    dice *que contiene* lo firmado.
mod inclusion;
// ⚠️ §275 · **La superficie CRECE**, y la cabecera de este crate declara
// que deberia ser la que menos cambia. La razon por la que se paga: el
// modulo es PRIVADO, asi que un `pub` que no aparezca aqui **no existe
// para nadie de fuera** — el verificador del acuse no seria independiente
// de nada. `verificar_inclusion` NO se sustituye: v1 es el recompositor
// de las cabezas ya custodiadas, y esas no cambian de forma.
pub use inclusion::{
    verificar_acuse, verificar_acuse_v3, verificar_inclusion, verificar_inclusion_v2,
    verificar_inclusion_v3, InclusionError, ReciboAcuse, ReciboInclusion,
};

// ⚠️ §274 · Las reglas del árbol de acuses viven AQUÍ y en ningún otro
// sitio: el constructor (nodo, §274) y el verificador (§275) llaman LAS
// MISMAS. Ver la cabecera del módulo para el borde que lo justifica.
pub mod acuses;

/// El MMR de cabezas (§291): el objeto que prueba «esta cabeza contiene
/// aquella» sin descargar el registro — eslabon 2 de la nota 83, puro.
/// La atadura al formato firmado (v3) es decision aparte y llega despues.
pub mod mmr;

// §292: las composiciones del digest, reexportadas para que quien ya
// depende de verify (cli, bin) no gane una dependencia solo por ellas.
pub use zk_ssl_hash::{epoch_digest_v2, epoch_digest_v3};

// ⚠️ §279 · **La superficie CRECE otra vez**, y por la misma razon que en
// §275: el modulo es PRIVADO, asi que un `pub` que no aparezca aqui no
// existe para nadie de fuera — y un reverificador inalcanzable no
// reverifica nada. Lo que entra es la respuesta a la nota 79: que puede
// comprobar un tercero del registro **sin el nodo**.
mod reverificacion;
pub use reverificacion::{censo, EntradaLog, ReverificacionError, Veredicto, reverificar};

/// El conjunto de parámetros: 2⁴⁰ firmas, ~35.000 años a una por segundo.
pub type Conjunto = XmssMtSha2_40_8_256;

/// Separación de dominio. **Sin versión dentro**, a propósito: dos
/// marcadores de versión que pueden discrepar valen menos que uno (§236).
pub const DOMINIO: &[u8] = b"ZK-SSL-epoch-head";

/// Versión del formato de cabeza que entra en la firma.
///
/// ⚠️ Sube cuando cambian **los campos de `EpochHead`**, no cuando cambia el
/// cable. Son ejes distintos: `zkssl/0.2` gobierna qué viaja; esto, qué
/// entra en la firma.
pub const VERSION_FORMATO: u8 = 3;

/// Bytes del RFC 8391 que ocupa la firma de este conjunto, sin el mensaje.
pub const FIRMA_RFC_BYTES: usize = 18_469;

/// Una cabeza firmada, tal como la publica el operador.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CabezaFirmada {
    /// Qué campos de la cabeza entraron en la firma.
    pub version_formato: u8,
    /// Cuántas firmas se han hecho con esta clave, contando ésta.
    ///
    /// ⚠️ **No entra en el preámbulo**: es metadato para detectar reúso, no
    /// algo que la firma acredite.
    pub indice: u64,
    /// La firma, en bytes del formato RFC 8391, con el preámbulo adjunto.
    pub firma: Vec<u8>,
}

#[derive(Debug)]
pub enum VerificaError {
    /// La clave pública no se pudo leer.
    ClaveIlegible(String),
    /// La firma no se pudo leer de sus bytes.
    FirmaIlegible(String),
    /// La firma no valida contra esa clave.
    NoVerifica(String),
    /// ⚠️ **La firma es válida, pero de OTRA cosa.**
    ///
    /// `verify()` devuelve el mensaje que la firma lleva dentro, no un
    /// booleano: que verifique dice *«esta firma vale para su contenido»*,
    /// **no** *«para lo que tú esperas»*. Sin comparar, un atacante presenta
    /// la firma legítima de otra cabeza y pasa.
    PreambuloDistinto { esperado: usize, recibido: usize },
}

impl core::fmt::Display for VerificaError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            VerificaError::ClaveIlegible(e) => write!(f, "clave publica ilegible: {e}"),
            VerificaError::FirmaIlegible(e) => write!(f, "firma ilegible: {e}"),
            VerificaError::NoVerifica(e) => write!(f, "la firma no verifica: {e}"),
            VerificaError::PreambuloDistinto { esperado, recibido } => write!(
                f,
                "la firma es VALIDA pero de otro mensaje (preambulo esperado \
                 {esperado} bytes, recibido {recibido}). Verificar sin comparar \
                 no prueba nada."
            ),
        }
    }
}

// ⚠️ `Debug`, `Display` y `Error` desde que nace. Sin los tres, cada
// consumidor se inventa un rodeo distinto (§228, §234, §241 — tres veces).
impl std::error::Error for VerificaError {}

/// El preámbulo exacto que se firma. **Es superficie de conformidad**: una
/// segunda implementación tiene que producir estos bytes.
///
/// ```text
/// b"ZK-SSL-epoch-head" ‖ version ‖ epoch_digest     (17 + 1 + 32 = 50)
/// ```
pub fn preambulo(version: u8, epoch_digest: &[u8; 32]) -> Vec<u8> {
    let mut v = Vec::with_capacity(DOMINIO.len() + 1 + 32);
    v.extend_from_slice(DOMINIO);
    v.push(version);
    v.extend_from_slice(epoch_digest);
    v
}

/// ⚠️ **APAÑO SOBRE UN FALLO DE `xmss 0.1.0-pre.0`** (§240, sondas S.5/S.6).
///
/// Una clave pública XMSS^MT **no se puede releer de sus propios bytes**:
///
/// ```text
/// // xmss.rs
/// let oid = XmssOid::try_from(raw).or_else(|_| XmssOid::from_xmssmt_raw_oid(raw))?;
///
/// // params.rs:1031 — hace justo lo que hace falta…
/// fn from_xmssmt_raw_oid(oid: u32) { Self::try_from(oid + XMSSMT_OID_OFFSET) }
/// ```
///
/// El RFC 8391 tiene **dos registros de OID separados** —XMSS y XMSS^MT— y
/// **los dos empiezan en 1**. `XMSSMT-SHA2_40/8_256` es el 5, y
/// `try_from(5)` **acierta** porque 5 también es un OID válido de árbol
/// único. **El `or_else` nunca corre.**
///
/// El apaño: sumar el offset **antes de parsear**.
///
/// ⚠️ **Vive aquí, con el verificador, porque es un apaño de LECTURA y quien
/// lee es quien verifica** (§243).
///
/// ⚠️ Y su centinela viaja con él: [`el_apano_del_oid_sigue_haciendo_falta`]
/// se pone **rojo el día que upstream lo arregle**, y esa señal debe llegar
/// al crate que la sufre, no al que la heredó.
///
/// [`el_apano_del_oid_sigue_haciendo_falta`]: #
pub const OFFSET_MT_UPSTREAM: u32 = 0x0001_0000;

/// Lee una clave pública publicada, aplicando el apaño del OID.
///
/// ⚠️ **Solo al leer.** Lo que el operador publica lleva su OID `0x00000005`
/// y es **RFC 8391 correcto**: el apaño no toca el cable.
pub fn clave_desde_bytes(rfc: &[u8]) -> Result<VerifyingKey<Conjunto>, VerificaError> {
    if rfc.len() < 4 {
        return Err(VerificaError::ClaveIlegible(format!(
            "{} bytes: no caben ni los 4 del OID",
            rfc.len()
        )));
    }
    let mut b = rfc.to_vec();
    let raw = u32::from_be_bytes([b[0], b[1], b[2], b[3]]) | OFFSET_MT_UPSTREAM;
    b[..4].copy_from_slice(&raw.to_be_bytes());
    VerifyingKey::<Conjunto>::try_from(b.as_slice())
        .map_err(|e| VerificaError::ClaveIlegible(format!("{e:?}")))
}

/// **La función del testigo.** Verifica una cabeza firmada **sin la clave
/// privada, sin el nodo y sin el operador**: solo con lo publicado.
///
/// ⚠️ **Verificar con éxito NO basta, y esta función es la razón.**
/// `verify()` devuelve **el mensaje que la firma lleva dentro**. Un atacante
/// puede presentar la firma **legítima de otra cabeza** y pasaría el
/// `verify()` a secas. Lo que cierra esa puerta es **comparar el mensaje
/// recuperado con el preámbulo esperado**.
///
/// ⚠️ Y no lo cierra el parseo: `Signature::try_from` **no valida OID ni
/// longitud** —las firmas adjuntas son de longitud variable—, así que es
/// casi un envoltorio. **Toda la validación real ocurre en `verify()` y en
/// la comparación de abajo.**
pub fn verificar_cabeza(
    clave_publica: &[u8],
    epoch_digest: &[u8; 32],
    c: &CabezaFirmada,
) -> Result<(), VerificaError> {
    let vk = clave_desde_bytes(clave_publica)?;
    let sig = Signature::<Conjunto>::try_from(c.firma.as_slice())
        .map_err(|e| VerificaError::FirmaIlegible(format!("{e:?}")))?;
    let recuperado = vk
        .verify(&sig)
        .map_err(|e| VerificaError::NoVerifica(format!("{e:?}")))?;
    // ⚠️ EL PASO QUE NO SE PUEDE SALTAR.
    let esperado = preambulo(c.version_formato, epoch_digest);
    if recuperado != esperado {
        return Err(VerificaError::PreambuloDistinto {
            esperado: esperado.len(),
            recibido: recuperado.len(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use xmss::KeyPair;

    /// Un par de claves determinista, y una cabeza firmada con él.
    ///
    /// ⚠️ Este crate **no firma en producción** —solo verifica—, pero sus
    /// tests necesitan firmas de verdad, y `xmss` ya es dependencia.
    fn firmado(digest: &[u8; 32]) -> (Vec<u8>, CabezaFirmada) {
        let mut s = [0u8; 96];
        for (i, b) in s.iter_mut().enumerate() {
            *b = (i as u8).wrapping_mul(17).wrapping_add(4);
        }
        let mut kp = KeyPair::<Conjunto>::from_seed(&s).expect("keygen");
        let pk = kp.verifying_key().as_ref().to_vec();
        let pre = preambulo(VERSION_FORMATO, digest);
        let sig = kp.signing_key().sign(&pre).expect("firmar");
        (
            pk,
            CabezaFirmada {
                version_formato: VERSION_FORMATO,
                indice: 1,
                firma: sig.as_ref().to_vec(),
            },
        )
    }

    // ── el preambulo: superficie de conformidad, sin clave ──

    #[test]
    fn el_preambulo_lleva_dominio_y_version_en_ese_orden() {
        let d = [0xABu8; 32];
        let p = preambulo(1, &d);
        assert_eq!(p.len(), DOMINIO.len() + 1 + 32);
        assert_eq!(p.len(), 50);
        assert_eq!(&p[..DOMINIO.len()], DOMINIO);
        assert_eq!(p[DOMINIO.len()], 1);
        assert_eq!(&p[DOMINIO.len() + 1..], &d);
    }

    #[test]
    fn el_dominio_no_lleva_la_version_dentro() {
        // Dos marcadores de version que pueden discrepar valen menos que uno.
        let s = String::from_utf8(DOMINIO.to_vec()).expect("utf8");
        assert!(!s.contains("v1"), "el dominio no debe llevar version: {s}");
        assert!(!s.contains("-v"), "el dominio no debe llevar version: {s}");
    }

    #[test]
    fn cambiar_la_version_cambia_lo_que_se_firma() {
        let d = [1u8; 32];
        assert_ne!(preambulo(1, &d), preambulo(2, &d), "la version debe entrar en la firma");
    }

    #[test]
    fn dos_cabezas_distintas_dan_preambulos_distintos() {
        assert_ne!(preambulo(1, &[1u8; 32]), preambulo(1, &[2u8; 32]));
    }

    // ── la verificacion, contra firmas de verdad ──

    #[test]
    fn una_cabeza_bien_firmada_verifica() {
        let d = [0x5Au8; 32];
        let (pk, c) = firmado(&d);
        verificar_cabeza(&pk, &d, &c).expect("debe verificar");
        assert_eq!(c.firma.len(), FIRMA_RFC_BYTES + 50, "RFC + preambulo adjunto");
    }

    #[test]
    fn una_firma_valida_de_otra_cabeza_se_rechaza() {
        // ⚠️⚠️ EL TEST QUE JUSTIFICA LA FUNCION. `verify()` devuelve el
        // MENSAJE, no un booleano: esta firma es PERFECTAMENTE VALIDA, solo
        // que de otra cosa. Sin comparar el preambulo, pasaria.
        let (pk, c) = firmado(&[0xAAu8; 32]);
        match verificar_cabeza(&pk, &[0xBBu8; 32], &c) {
            Err(VerificaError::PreambuloDistinto { .. }) => {}
            otro => panic!("una firma de OTRA cabeza debe rechazarse, y dio: {otro:?}"),
        }
    }

    #[test]
    fn una_version_de_formato_cambiada_se_rechaza() {
        let d = [0x11u8; 32];
        let (pk, mut c) = firmado(&d);
        c.version_formato = VERSION_FORMATO + 1;
        assert!(verificar_cabeza(&pk, &d, &c).is_err(), "otra version debe fallar");
    }

    #[test]
    fn un_byte_cambiado_en_la_firma_se_rechaza() {
        let d = [0x22u8; 32];
        let (pk, mut c) = firmado(&d);
        c.firma[100] ^= 0x01;
        assert!(verificar_cabeza(&pk, &d, &c).is_err(), "un bit cambiado debe fallar");
    }

    #[test]
    fn basura_se_rechaza_sin_reventar() {
        // ⚠️ Un testigo recibe lo que le manden. Debe dar Err, no panic.
        let d = [0x33u8; 32];
        let (pk, c0) = firmado(&d);
        for firma in [
            vec![],
            vec![0u8; 10],
            vec![0xFFu8; FIRMA_RFC_BYTES + 50],
            c0.firma[..100].to_vec(),
        ] {
            let c = CabezaFirmada { firma, ..c0.clone() };
            assert!(verificar_cabeza(&pk, &d, &c).is_err(), "la basura debe dar Err");
        }
        for clave in [vec![], vec![0u8; 3], vec![0u8; 68], vec![0xFFu8; 200]] {
            assert!(verificar_cabeza(&clave, &d, &c0).is_err(), "una clave rota debe dar Err");
        }
    }

    // ── el apaño, y su centinela ──

    #[test]
    fn la_clave_publicada_lleva_el_oid_del_rfc_sin_el_apano() {
        // ⚠️ Lo que se PUBLICA es RFC 8391 correcto: OID 0x00000005, SIN el
        // offset. El apaño vive en la lectura, no en el cable.
        let (pk, _) = firmado(&[0u8; 32]);
        assert_eq!(&pk[..4], &[0x00, 0x00, 0x00, 0x05], "el OID publicado debe ser el del RFC");
        assert_eq!(pk.len(), 68, "OID(4) + root(32) + pub_seed(32)");
    }

    #[test]
    fn el_apano_del_oid_sigue_haciendo_falta() {
        // ⚠️⚠️ EL CENTINELA, que viaja CON el apaño (§243).
        //
        // Comprueba que SIN sumar el offset la clave NO se puede releer. El
        // dia que upstream arregle `parse_oid_and_params`, esto se pone ROJO
        // y avisa de que `OFFSET_MT_UPSTREAM` hay que quitarlo.
        //
        // Un apaño que no sabe cuando estorba se queda para siempre — y
        // ademas ENMASCARA el cambio de formato que venga despues.
        let (pk, _) = firmado(&[0u8; 32]);
        assert!(
            VerifyingKey::<Conjunto>::try_from(pk.as_slice()).is_err(),
            "⚠️ `xmss` YA RELEE la clave multiarbol sin el apaño: quitar \
             OFFSET_MT_UPSTREAM y clave_desde_bytes, y cerrar el hallazgo en \
             doc/issue-rustcrypto.md"
        );
        // Y con el apaño, vuelve.
        assert!(clave_desde_bytes(&pk).is_ok(), "con el offset la clave debe volver");
    }
}
