//! # El acta de clave: lo que un tercero comprueba de una rotacion (RFC-0015, E2; §643)
//!
//! Cada clave del operador entra con un **acta** que lleva la huella de la clave que la
//! sucedera (pre-rotacion, D-B, como el RFC 8649). Una rotacion vale si la clave que llega es
//! exactamente la comprometida y firma su propia acta; la que se va la firma tambien cuando su
//! estado es fiable, y un acta sin esa firma es la declaracion del operador de que la vieja esta
//! quemada (decision 5, §642). Aqui viven el preambulo que se firma y las reglas del verificador
//! de la D-C, las mismas que aplicaran el testigo (E4) y el kit sobre sus sobres (E5).
//!
//! ⚠️ **Solo el acta, no la cadena ni las cabezas.** La regla 4 de la D-C —una cabeza vale
//! entre el `desde` de su acta y el de la siguiente— necesita los sobres que comparan cabezas,
//! y es de la E5. Esto juzga un acta contra la previa, que es el eslabon con que se arma todo.
//!
//! ⚠️ **El `desde` es el indice EMBEBIDO** (RFC-0012, D-C; §399): el que la firma acredita. La
//! primera hoja de la clave que entra firma su acta, asi que su firma lleva dentro el `desde`.

use serde_json::{json, Value};
use xmss::Signature;
use zk_ssl_hash::{acta_digest, digest_from_bytes, digest_to_bytes};

// ⚠️ Reexportados para que quien arme un acta -el nodo- nombre la clave con la MISMA huella con
// que este juez la compara, sin depender de `zk-ssl-hash` por su cuenta.
pub use zk_ssl_hash::{huella_de_clave, Digest};

use crate::{clave_desde_bytes, indice_de_firma, Conjunto, VerificaError};

/// Separacion de dominio de **la firma de un acta**. Sin version dentro, por lo mismo que
/// [`crate::DOMINIO`] (§236): la version va en el preambulo. Distinto de la cabeza y de la
/// cofirma a proposito: una firma de cabeza nunca se lee como acta, ni al reves.
pub const DOMINIO_ACTA_FIRMA: &[u8] = b"ZK-SSL-key-act";

/// La version del acta, que entra en su preambulo.
pub const ACTA_VERSION: u8 = 1;

/// **El esquema de la clave de hoy**: XMSS^MT-SHA2_40/8_256. La familia en los 32 bits altos
/// (1, el registro de RFC 8391) y el OID de RFC 8391 en los bajos (`0x00000005`). Es el unico
/// que este verificador sabe comprobar: otro esquema se rechaza con su numero (la 87).
pub const ESQUEMA_XMSSMT_SHA2_40_8_256: u64 = (1 << 32) | 0x0000_0005;

/// El preambulo exacto que firma un acta. **Es superficie de conformidad**, con su KAT en
/// `spec/vectors/nucleo/`: dominio, version y los 32 bytes de [`acta_digest`], el molde de
/// [`crate::preambulo`] con otro dominio.
pub fn preambulo_acta(version: u8, acta_digest: &[u8; 32]) -> Vec<u8> {
    let mut v = Vec::with_capacity(DOMINIO_ACTA_FIRMA.len() + 1 + 32);
    v.extend_from_slice(DOMINIO_ACTA_FIRMA);
    v.push(version);
    v.extend_from_slice(acta_digest);
    v
}

/// De donde viene la clave que entra, en una rotacion: la huella de la que se va y la ultima
/// cabeza que firmo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Procedencia {
    pub anterior: Digest,
    pub epoch_digest: Digest,
    pub mmr_root: Digest,
    pub mmr_size: u64,
}

/// Un acta (RFC-0015, D-C). `procedencia` es `None` en el acta genesis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Acta {
    /// La clave publica que entra, entera, en el formato de RFC 8391.
    pub clave: Vec<u8>,
    pub esquema: u64,
    pub desde: u64,
    /// La huella de la clave que sucedera a esta: el compromiso de la pre-rotacion.
    pub siguiente: Digest,
    pub procedencia: Option<Procedencia>,
}

impl Acta {
    /// Su huella, con [`acta_digest`] y la [`huella_de_clave`] de su clave.
    pub fn digest(&self) -> Digest {
        acta_digest(
            huella_de_clave(&self.clave),
            self.esquema,
            self.desde,
            self.siguiente,
            self.procedencia
                .as_ref()
                .map(|p| (p.anterior, p.epoch_digest, p.mmr_root, p.mmr_size)),
        )
    }
}

/// Un acta con sus firmas: la de la clave que entra, siempre; la de la que se va, si su estado
/// era fiable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActaFirmada {
    pub acta: Acta,
    pub firma: Vec<u8>,
    pub firma_anterior: Option<Vec<u8>>,
}

/// Por que un acta no vale. Cada regla de la D-C tiene su nombre.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActaError {
    /// El esquema no es uno que este verificador sepa comprobar.
    EsquemaDesconocido(u64),
    /// La firma de la clave que entra no verifica sobre el preambulo del acta.
    Firma(VerificaError),
    /// La firma de la clave que entra no esta en su hoja `desde`.
    IndiceDeLaFirma { desde: u64, embebido: u64 },
    /// Un acta genesis presentada como continuacion de otra.
    GenesisConPrevia,
    /// Una rotacion sin el acta de la que viene.
    RotacionSinPrevia,
    /// Un acta genesis no tiene clave anterior que la firme.
    GenesisConFirmaAnterior,
    /// La `anterior` del acta no es la huella de la clave del acta previa.
    AnteriorDistinta,
    /// ⚠️ La clave que entra no es la que el acta previa comprometio: la pre-rotacion la
    /// rechaza aunque la firme la clave vieja.
    NoComprometida,
    /// El `desde` no crece: la cuenta es del operador (D-A), no de la clave.
    DesdeNoCrece { previo: u64, desde: u64 },
    /// La firma de la clave que se va no verifica sobre el mismo preambulo.
    FirmaDeLaAnterior(VerificaError),
    /// La firma de la clave que se va no esta entre su `desde` y el de la que entra.
    IndiceDeLaAnterior {
        previo: u64,
        embebido: u64,
        desde: u64,
    },
}

impl core::fmt::Display for ActaError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ActaError::EsquemaDesconocido(e) => {
                write!(f, "el acta presenta una clave de esquema {e:#x}, que este verificador no conoce")
            }
            ActaError::Firma(e) => write!(f, "la firma de la clave que entra: {e}"),
            ActaError::IndiceDeLaFirma { desde, embebido } => write!(
                f,
                "la clave que entra firma su acta en la hoja {embebido} y el acta dice desde {desde}"
            ),
            ActaError::GenesisConPrevia => {
                write!(f, "un acta genesis no continua a otra: no lleva procedencia")
            }
            ActaError::RotacionSinPrevia => {
                write!(f, "una rotacion se juzga contra el acta de la que viene, y no esta")
            }
            ActaError::GenesisConFirmaAnterior => {
                write!(f, "un acta genesis no tiene clave anterior que la firme")
            }
            ActaError::AnteriorDistinta => {
                write!(f, "la anterior del acta no es la clave del acta previa")
            }
            ActaError::NoComprometida => write!(
                f,
                "la clave que entra NO es la que el acta previa comprometio: la pre-rotacion la rechaza"
            ),
            ActaError::DesdeNoCrece { previo, desde } => write!(
                f,
                "el desde {desde} no supera el {previo} de la clave previa: la cuenta es del operador"
            ),
            ActaError::FirmaDeLaAnterior(e) => write!(f, "la firma de la clave que se va: {e}"),
            ActaError::IndiceDeLaAnterior { previo, embebido, desde } => write!(
                f,
                "la clave que se va firma en la hoja {embebido}, fuera de ({previo}, {desde})"
            ),
        }
    }
}

impl std::error::Error for ActaError {}

/// Verifica una firma sobre un mensaje exacto y devuelve el indice que lleva dentro.
fn firma_sobre(clave: &[u8], mensaje: &[u8], firma: &[u8]) -> Result<u64, VerificaError> {
    let vk = clave_desde_bytes(clave)?;
    let sig = Signature::<Conjunto>::try_from(firma)
        .map_err(|e| VerificaError::FirmaIlegible(format!("{e:?}")))?;
    let recuperado = vk
        .verify(&sig)
        .map_err(|e| VerificaError::NoVerifica(format!("{e:?}")))?;
    // ⚠️ El paso que no se puede saltar, como en `verificar_cabeza`: `verify()` devuelve el
    // mensaje que la firma lleva dentro, y una firma legitima de otra cosa pasaria sin esto.
    if recuperado != mensaje {
        return Err(VerificaError::PreambuloDistinto {
            esperado: mensaje.len(),
            recibido: recuperado.len(),
        });
    }
    indice_de_firma(firma)
}

/// **Juzga un acta**, sola si es la genesis o contra el acta previa si es una rotacion, con las
/// reglas 1 a 3 de la D-C del RFC-0015.
///
/// ⚠️ **Las reglas van antes que las firmas.** Lo que el acta afirma -de donde viene, que clave
/// entra, desde donde- se juzga primero: una clave no comprometida se rechaza aunque la firmen
/// la nueva y la vieja, porque ninguna firma la rescata. Despues, las firmas.
pub fn verificar_acta(a: &ActaFirmada, previa: Option<&Acta>) -> Result<(), ActaError> {
    let acta = &a.acta;
    if acta.esquema != ESQUEMA_XMSSMT_SHA2_40_8_256 {
        return Err(ActaError::EsquemaDesconocido(acta.esquema));
    }
    match (&acta.procedencia, previa) {
        (None, None) if a.firma_anterior.is_some() => {
            return Err(ActaError::GenesisConFirmaAnterior)
        }
        (None, None) => {}
        (None, Some(_)) => return Err(ActaError::GenesisConPrevia),
        (Some(_), None) => return Err(ActaError::RotacionSinPrevia),
        (Some(p), Some(previa)) => {
            if p.anterior != huella_de_clave(&previa.clave) {
                return Err(ActaError::AnteriorDistinta);
            }
            if huella_de_clave(&acta.clave) != previa.siguiente {
                return Err(ActaError::NoComprometida);
            }
            if acta.desde <= previa.desde {
                return Err(ActaError::DesdeNoCrece {
                    previo: previa.desde,
                    desde: acta.desde,
                });
            }
        }
    }
    let pre = preambulo_acta(ACTA_VERSION, &digest_to_bytes(&acta.digest()));
    let embebido = firma_sobre(&acta.clave, &pre, &a.firma).map_err(ActaError::Firma)?;
    if embebido != acta.desde {
        return Err(ActaError::IndiceDeLaFirma {
            desde: acta.desde,
            embebido,
        });
    }
    if let (Some(f), Some(previa)) = (&a.firma_anterior, previa) {
        let e = firma_sobre(&previa.clave, &pre, f).map_err(ActaError::FirmaDeLaAnterior)?;
        if e <= previa.desde || e >= acta.desde {
            return Err(ActaError::IndiceDeLaAnterior {
                previo: previa.desde,
                embebido: e,
                desde: acta.desde,
            });
        }
    }
    Ok(())
}

/// ⚠️ §647 · **La cadena entera**, eslabón a eslabón (D-C, reglas 1 a 3): la génesis sola y cada
/// rotación contra la previa. Un productor para el nodo que arranca, el testigo que rota y los
/// sobres de la E5. El error dice qué eslabón, contando desde 0, y por qué.
pub fn verificar_cadena(actas: &[ActaFirmada]) -> Result<(), (usize, ActaError)> {
    for (i, a) in actas.iter().enumerate() {
        let previa = i.checked_sub(1).map(|j| &actas[j].acta);
        verificar_acta(a, previa).map_err(|e| (i, e))?;
    }
    Ok(())
}

/// Lo que una rotación juzgada deja fijado: el tramo de índices EMBEBIDOS de la clave que llega
/// (D-C, regla 4) —por encima de su `desde`, que es la hoja de su acta, y por debajo del `desde`
/// de la siguiente, si la cadena la trae— y cuántos eslabones se cruzaron.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rotacion {
    pub desde: u64,
    pub hasta: Option<u64>,
    pub eslabones: usize,
}

impl Rotacion {
    /// Si una cabeza de la clave, con este índice EMBEBIDO, cae en su tramo.
    pub fn en_su_tramo(&self, embebido: u64) -> bool {
        embebido > self.desde && self.hasta.map_or(true, |h| embebido < h)
    }
}

/// Por qué una clave no rota a otra con esta cadena.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RotacionError {
    /// Un eslabón de la cadena no vale: cuál, contando desde 0, y su regla.
    Cadena { eslabon: usize, error: ActaError },
    /// La clave que llega no está en la cadena: nadie la comprometió.
    RecibidaFuera,
    /// La clave que se tenía no está en la cadena antes de la que llega.
    FijadaFuera,
    /// ⚠️ **Solapamiento** (D-C, reglas 3 y 4): una firma de la clave que se va con un índice
    /// embebido que alcanza el `desde` de su sucesora. Evidencia oponible con nombre, como la
    /// vista dividida: rotar no escapa de lo ya firmado.
    Solapamiento { indice: u64, desde: u64 },
}

impl std::fmt::Display for RotacionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RotacionError::Cadena { eslabon, error } => {
                write!(f, "el acta {eslabon} de la cadena no vale: {error}")
            }
            RotacionError::RecibidaFuera => {
                write!(
                    f,
                    "la clave que llega no esta en la cadena: nadie la comprometio"
                )
            }
            RotacionError::FijadaFuera => {
                write!(
                    f,
                    "la clave que se tenia no esta en la cadena antes de la que llega"
                )
            }
            RotacionError::Solapamiento { indice, desde } => write!(
                f,
                "SOLAPAMIENTO: la clave que se va firmo en la hoja {indice}, y su sucesora \
                 empieza en la {desde}"
            ),
        }
    }
}

/// ⚠️ §647 · **Juzga el paso de la clave `de` a la clave `a` con una cadena de actas** (RFC-0015,
/// D-C y D-E): la cadena entera vale; `a` está en ella y `de` antes; y lo que el verificador vio
/// firmar a `de` —su mayor índice EMBEBIDO, si vio alguno— queda por debajo del `desde` de su
/// sucesora. Puede cruzar varios eslabones: un testigo que estuvo apagado ve la clave de hoy, no
/// las de en medio. Si una clave vuelve a la cadena, cuenta su última entrada.
pub fn juzgar_rotacion(
    actas: &[ActaFirmada],
    de: &[u8],
    a: &[u8],
    ultimo_de_la_vieja: Option<u64>,
) -> Result<Rotacion, RotacionError> {
    verificar_cadena(actas).map_err(|(eslabon, error)| RotacionError::Cadena { eslabon, error })?;
    let j = actas
        .iter()
        .rposition(|x| x.acta.clave == a)
        .ok_or(RotacionError::RecibidaFuera)?;
    let i = actas[..j]
        .iter()
        .rposition(|x| x.acta.clave == de)
        .ok_or(RotacionError::FijadaFuera)?;
    if let Some(u) = ultimo_de_la_vieja {
        let sucesora = actas[i + 1].acta.desde;
        if u >= sucesora {
            return Err(RotacionError::Solapamiento {
                indice: u,
                desde: sucesora,
            });
        }
    }
    Ok(Rotacion {
        desde: actas[j].acta.desde,
        hasta: actas.get(j + 1).map(|x| x.acta.desde),
        eslabones: j - i,
    })
}

/// ⚠️ §646 · **El acta en JSON, con UN productor** (RFC-0015 D-D: en el diario, en el cable y en
/// los sobres, tal cual). Con las convenciones de `spec/RPC.md`: la clave y las firmas en `DATA`,
/// el esquema, el `desde` y el tamaño del MMR en `Q`, y los digests en `Digest`, con la
/// serialización que persiste la capa. `procedencia` y `firmaAnterior` van SIEMPRE, `null` cuando
/// no hay: un campo ausente no se lee como uno vacío.
pub fn acta_a_json(a: &ActaFirmada) -> Value {
    let acta = &a.acta;
    let procedencia = match &acta.procedencia {
        None => Value::Null,
        Some(p) => json!({
            "anterior": hex_de_digest(&p.anterior),
            "epochDigest": hex_de_digest(&p.epoch_digest),
            "mmrRoot": hex_de_digest(&p.mmr_root),
            "mmrSize": q(p.mmr_size),
        }),
    };
    json!({
        "acta": {
            "clave": hex(&acta.clave),
            "esquema": q(acta.esquema),
            "desde": q(acta.desde),
            "siguiente": hex_de_digest(&acta.siguiente),
            "procedencia": procedencia,
        },
        "firma": hex(&a.firma),
        "firmaAnterior": a.firma_anterior.as_ref().map(|f| hex(f)),
    })
}

/// Lee lo que [`acta_a_json`] escribe, con el nombre del campo que falla. Las claves de más se
/// ignoran -la línea del diario lleva las suyas: `v`, `tipo` e `index`-; un campo que falta, o
/// que no es lo que dice ser, no. Leer NO es juzgar: eso es [`verificar_acta`].
pub fn acta_de_json(v: &Value) -> Result<ActaFirmada, String> {
    let a = v
        .get("acta")
        .filter(|x| x.is_object())
        .ok_or("falta acta, o no es un objeto")?;
    let procedencia = match a.get("procedencia") {
        None => return Err("falta acta.procedencia: null en la genesis".into()),
        Some(Value::Null) => None,
        Some(p) => Some(Procedencia {
            anterior: digest_de(p, "anterior")?,
            epoch_digest: digest_de(p, "epochDigest")?,
            mmr_root: digest_de(p, "mmrRoot")?,
            mmr_size: q_de(p, "mmrSize")?,
        }),
    };
    let firma_anterior = match v.get("firmaAnterior") {
        None => return Err("falta firmaAnterior: null si la clave que se va no firma".into()),
        Some(Value::Null) => None,
        Some(_) => Some(data_de(v, "firmaAnterior")?),
    };
    Ok(ActaFirmada {
        acta: Acta {
            clave: data_de(a, "clave")?,
            esquema: q_de(a, "esquema")?,
            desde: q_de(a, "desde")?,
            siguiente: digest_de(a, "siguiente")?,
            procedencia,
        },
        firma: data_de(v, "firma")?,
        firma_anterior,
    })
}

fn hex(b: &[u8]) -> String {
    let mut s = String::with_capacity(2 + 2 * b.len());
    s.push_str("0x");
    for x in b {
        s.push_str(&format!("{x:02x}"));
    }
    s
}

fn hex_de_digest(d: &Digest) -> String {
    hex(&digest_to_bytes(d))
}

fn q(n: u64) -> String {
    format!("{n:#x}")
}

fn texto<'a>(v: &'a Value, campo: &str) -> Result<&'a str, String> {
    v.get(campo)
        .and_then(|x| x.as_str())
        .ok_or_else(|| format!("falta {campo}, o no es una cadena"))
}

/// El hex del cable —`0x`, minúscula, por pares—, con el lector de `zk-ssl-hash` (§650).
fn data_de(v: &Value, campo: &str) -> Result<Vec<u8>, String> {
    zk_ssl_hash::hex_canonico(texto(v, campo)?).map_err(|e| format!("{campo}: {e}"))
}

/// ⚠️ **Un elemento fuera del campo no se lee**: ocho bytes que valen `p` o más no son un
/// elemento, y `digest_from_bytes` los rechaza desde el §640 (RFC-0016). El lector del acta lo
/// hereda y no lo repite.
fn digest_de(v: &Value, campo: &str) -> Result<Digest, String> {
    let b = data_de(v, campo)?;
    if b.len() != 32 {
        return Err(format!("{campo}: {} bytes, se esperaban 32", b.len()));
    }
    digest_from_bytes(&b).map_err(|e| format!("{campo}: {e:?}"))
}

/// Un `Q` del cable, en su escritura mínima, con el lector de `zk-ssl-hash` (§662).
fn q_de(v: &Value, campo: &str) -> Result<u64, String> {
    zk_ssl_hash::cantidad_canonica(texto(v, campo)?).map_err(|e| format!("{campo}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::OnceLock;
    use xmss::{KeyPair, SigningKey};
    use zk_ssl_hash::as_digest;

    // ⚠️ Cada clave y cada firma XMSS^MT 40/8 cuestan segundos en un build de depuracion. La
    // escena se firma UNA vez, en paralelo, y la comparten los tests; las reglas van antes que
    // las firmas, asi que sus rojos no necesitan firmar nada nuevo. Cinco firmas en el modulo.

    fn clave(semilla: u8) -> KeyPair<Conjunto> {
        KeyPair::<Conjunto>::from_seed(&[semilla; 96]).expect("keygen")
    }

    fn publica(kp: &KeyPair<Conjunto>) -> Vec<u8> {
        kp.verifying_key().as_ref().to_vec()
    }

    /// Firma `mensaje` con la hoja `hoja` de la clave, como el nodo tras resincronizar (§638).
    fn firmar_en(kp: &mut KeyPair<Conjunto>, hoja: u64, mensaje: &[u8]) -> Vec<u8> {
        let mut sk = kp.signing_key().as_ref().to_vec();
        zk_ssl_guardian::poner_indice_en_sk(&mut sk, hoja).expect("indice");
        crate::aplicar_apano_del_oid(&mut sk).expect("apano del OID");
        *kp.signing_key() = SigningKey::<Conjunto>::try_from(sk.as_slice()).expect("sk");
        kp.signing_key()
            .sign(mensaje)
            .expect("firmar")
            .as_ref()
            .to_vec()
    }

    fn pre(acta: &Acta) -> Vec<u8> {
        preambulo_acta(ACTA_VERSION, &digest_to_bytes(&acta.digest()))
    }

    /// La genesis de la clave 1, comprometiendo a la 2; la rotacion a la 2 desde 40,
    /// comprometiendo a la 3, firmada por la 2 y por la 1 en su hoja 39. Y dos firmas mal
    /// puestas de la 1: su genesis en la hoja 1, y la de la rotacion en la hoja 40.
    struct Escena {
        genesis: ActaFirmada,
        rotacion: ActaFirmada,
        genesis_en_la_hoja_1: Vec<u8>,
        vieja_en_la_hoja_40: Vec<u8>,
    }

    fn escena() -> &'static Escena {
        static E: OnceLock<Escena> = OnceLock::new();
        E.get_or_init(|| {
            // Las claves y las firmas, en paralelo: son independientes, y cada una cuesta segundos.
            let (k0, k1, k2, k0b) = std::thread::scope(|s| {
                let h = [1u8, 2, 3, 1].map(|x| s.spawn(move || clave(x)));
                let [a, b, c, d] = h.map(|h| h.join().expect("keygen"));
                (a, b, c, d)
            });
            let g = Acta {
                clave: publica(&k0),
                esquema: ESQUEMA_XMSSMT_SHA2_40_8_256,
                desde: 0,
                siguiente: huella_de_clave(&publica(&k1)),
                procedencia: None,
            };
            let r = Acta {
                clave: publica(&k1),
                esquema: ESQUEMA_XMSSMT_SHA2_40_8_256,
                desde: 40,
                siguiente: huella_de_clave(&publica(&k2)),
                procedencia: Some(Procedencia {
                    anterior: huella_de_clave(&publica(&k0)),
                    epoch_digest: as_digest(7),
                    mmr_root: as_digest(8),
                    mmr_size: 39,
                }),
            };
            let (pg, pr) = (pre(&g), pre(&r));
            let ((firma_g, vieja), firma_r, (en_1, en_40)) = std::thread::scope(|s| {
                let a = s.spawn(|| {
                    let mut k = k0;
                    (firmar_en(&mut k, 0, &pg), firmar_en(&mut k, 39, &pr))
                });
                let b = s.spawn(|| {
                    let mut k = k1;
                    firmar_en(&mut k, 40, &pr)
                });
                let c = s.spawn(|| {
                    let mut k = k0b;
                    (firmar_en(&mut k, 1, &pg), firmar_en(&mut k, 40, &pr))
                });
                (
                    a.join().expect("k0"),
                    b.join().expect("k1"),
                    c.join().expect("k0b"),
                )
            });
            Escena {
                genesis: ActaFirmada {
                    acta: g,
                    firma: firma_g,
                    firma_anterior: None,
                },
                rotacion: ActaFirmada {
                    acta: r,
                    firma: firma_r,
                    firma_anterior: Some(vieja),
                },
                genesis_en_la_hoja_1: en_1,
                vieja_en_la_hoja_40: en_40,
            }
        })
    }

    #[test]
    fn la_genesis_y_una_rotacion_comprometida_valen_con_y_sin_la_firma_de_la_vieja() {
        let e = escena();
        assert_eq!(verificar_acta(&e.genesis, None), Ok(()));
        assert_eq!(
            verificar_acta(&e.rotacion, Some(&e.genesis.acta)),
            Ok(()),
            "con la vieja en su hoja 39"
        );
        let mut sin = e.rotacion.clone();
        sin.firma_anterior = None;
        assert_eq!(
            verificar_acta(&sin, Some(&e.genesis.acta)),
            Ok(()),
            "sin la vieja: la vieja, quemada"
        );
    }

    #[test]
    fn una_clave_no_comprometida_no_rota_aunque_traiga_la_firma_de_la_vieja() {
        // El ataque que la pre-rotacion cierra: quien robo la vieja arma un acta hacia una clave
        // suya. La regla va antes que las firmas, asi que ninguna firma la rescata.
        let e = escena();
        let mut r = e.rotacion.clone();
        r.acta.clave = publica(&clave(9));
        assert_eq!(
            verificar_acta(&r, Some(&e.genesis.acta)),
            Err(ActaError::NoComprometida)
        );
    }

    #[test]
    fn cada_regla_de_la_rotacion_tiene_su_rojo() {
        let e = escena();
        let g = &e.genesis.acta;
        let mut otra = e.rotacion.clone();
        otra.acta.procedencia.as_mut().expect("rotacion").anterior = as_digest(99);
        assert_eq!(
            verificar_acta(&otra, Some(g)),
            Err(ActaError::AnteriorDistinta)
        );
        let mut baja = e.rotacion.clone();
        baja.acta.desde = 0;
        assert_eq!(
            verificar_acta(&baja, Some(g)),
            Err(ActaError::DesdeNoCrece {
                previo: 0,
                desde: 0
            })
        );
        // la firma de la vieja sobre OTRA cosa: la de la genesis, que es de la vieja y de otra acta
        let mut ajena = e.rotacion.clone();
        ajena.firma_anterior = Some(e.genesis.firma.clone());
        assert!(matches!(
            verificar_acta(&ajena, Some(g)),
            Err(ActaError::FirmaDeLaAnterior(
                VerificaError::PreambuloDistinto { .. }
            ))
        ));
        // la firma de la que entra, cambiada por la de la vieja: no verifica con la nueva
        let mut cambiada = e.rotacion.clone();
        cambiada.firma = cambiada.firma_anterior.clone().expect("vieja");
        assert!(matches!(
            verificar_acta(&cambiada, Some(g)),
            Err(ActaError::Firma(_))
        ));
        // genesis y rotacion fuera de sitio
        assert_eq!(
            verificar_acta(&e.genesis, Some(g)),
            Err(ActaError::GenesisConPrevia)
        );
        assert_eq!(
            verificar_acta(&e.rotacion, None),
            Err(ActaError::RotacionSinPrevia)
        );
        let mut g2 = e.genesis.clone();
        g2.firma_anterior = Some(e.genesis.firma.clone());
        assert_eq!(
            verificar_acta(&g2, None),
            Err(ActaError::GenesisConFirmaAnterior)
        );
        // un esquema que no se conoce se rechaza con su numero, antes de mirar nada mas
        let mut ajeno = e.genesis.clone();
        ajeno.acta.esquema = 0x2_0000_0001;
        assert_eq!(
            verificar_acta(&ajeno, None),
            Err(ActaError::EsquemaDesconocido(0x2_0000_0001))
        );
    }

    #[test]
    fn las_firmas_estan_en_su_hoja_y_en_su_dominio() {
        let e = escena();
        // la clave que entra firma su acta en otra hoja
        let mut corrida = e.genesis.clone();
        corrida.firma = e.genesis_en_la_hoja_1.clone();
        assert_eq!(
            verificar_acta(&corrida, None),
            Err(ActaError::IndiceDeLaFirma {
                desde: 0,
                embebido: 1
            })
        );
        // la vieja firma en la hoja del desde de la nueva: fuera de su tramo
        let mut fuera = e.rotacion.clone();
        fuera.firma_anterior = Some(e.vieja_en_la_hoja_40.clone());
        assert_eq!(
            verificar_acta(&fuera, Some(&e.genesis.acta)),
            Err(ActaError::IndiceDeLaAnterior {
                previo: 0,
                embebido: 40,
                desde: 40
            })
        );
        // el dominio: los mismos 32 bytes con el preambulo de CABEZA no son un acta
        let d = digest_to_bytes(&e.genesis.acta.digest());
        assert_ne!(
            preambulo_acta(ACTA_VERSION, &d),
            crate::preambulo(ACTA_VERSION, &d)
        );
        // el dominio, sin version dentro (§236) y con la longitud del preambulo de NUCLEO.md
        let texto = String::from_utf8(DOMINIO_ACTA_FIRMA.to_vec()).expect("utf8");
        assert!(
            !texto.contains("-v"),
            "el dominio no lleva version: {texto}"
        );
        assert_eq!(preambulo_acta(ACTA_VERSION, &d).len(), 47);
        assert_ne!(DOMINIO_ACTA_FIRMA, crate::DOMINIO);
        assert_ne!(DOMINIO_ACTA_FIRMA, crate::DOMINIO_COFIRMA);
    }

    /// ⚠️ §647 · **la rotación juzgada con la cadena.** De la clave 1 a la 2 vale, con su tramo
    /// por encima de 40 y sin techo; la cadena al revés no la explica; una clave que nadie
    /// comprometió tampoco; la cadena sin su génesis no vale, y dice el eslabón. Y la clave que
    /// se va con una firma en la hoja 40 —el `desde` de su sucesora— es SOLAPAMIENTO; en la 39 no.
    #[test]
    fn la_rotacion_se_juzga_con_la_cadena_y_el_solapamiento_tiene_nombre() {
        let e = escena();
        let cadena = [e.genesis.clone(), e.rotacion.clone()];
        let (k1, k2) = (&e.genesis.acta.clave, &e.rotacion.acta.clave);
        let r = juzgar_rotacion(&cadena, k1, k2, Some(39)).expect("rota");
        assert_eq!(
            r,
            Rotacion {
                desde: 40,
                hasta: None,
                eslabones: 1
            }
        );
        assert!(
            !r.en_su_tramo(40),
            "la hoja 40 es del acta, no de una cabeza"
        );
        assert!(r.en_su_tramo(41) && r.en_su_tramo(u64::MAX));
        assert_eq!(
            juzgar_rotacion(&cadena, k1, k2, None).map(|r| r.desde),
            Ok(40)
        );
        assert_eq!(
            juzgar_rotacion(&cadena, k1, k2, Some(40)),
            Err(RotacionError::Solapamiento {
                indice: 40,
                desde: 40
            })
        );
        assert_eq!(
            juzgar_rotacion(&cadena, k2, k1, None),
            Err(RotacionError::FijadaFuera),
            "la cadena no rota hacia atras"
        );
        assert_eq!(
            juzgar_rotacion(&cadena, k1, &[9u8; 4], None),
            Err(RotacionError::RecibidaFuera)
        );
        assert!(matches!(
            juzgar_rotacion(&cadena[1..], k1, k2, None),
            Err(RotacionError::Cadena { eslabon: 0, .. })
        ));
        assert_eq!(verificar_cadena(&cadena), Ok(()));
        let tramo = Rotacion {
            desde: 40,
            hasta: Some(90),
            eslabones: 1,
        };
        assert!(tramo.en_su_tramo(89) && !tramo.en_su_tramo(90));
    }

    /// ⚠️ §646 · **el acta en JSON va y vuelve, y dice qué campo falla.** Sin firmar nada: leer
    /// no es juzgar. La génesis lleva sus dos `null`; la rotación, todo. Un campo que falta, un
    /// `Q` sin `0x`, un digest corto, un elemento que vale `p` y una firma impar se rechazan con
    /// su nombre; una clave de más, como las de la línea del diario, no. El `p` es el borde:
    /// `p − 1` se lee.
    #[test]
    fn el_acta_en_json_va_y_vuelve_y_dice_que_campo_falla() {
        let genesis = ActaFirmada {
            acta: Acta {
                clave: vec![1, 2, 3],
                esquema: ESQUEMA_XMSSMT_SHA2_40_8_256,
                desde: 0,
                siguiente: as_digest(7),
                procedencia: None,
            },
            firma: vec![0xab; 5],
            firma_anterior: None,
        };
        let rotacion = ActaFirmada {
            acta: Acta {
                clave: vec![4, 5],
                esquema: ESQUEMA_XMSSMT_SHA2_40_8_256,
                desde: 41,
                siguiente: as_digest(8),
                procedencia: Some(Procedencia {
                    anterior: as_digest(9),
                    epoch_digest: as_digest(10),
                    mmr_root: as_digest(11),
                    mmr_size: 40,
                }),
            },
            firma: vec![0xcd; 3],
            firma_anterior: Some(vec![0xef; 4]),
        };
        for a in [&genesis, &rotacion] {
            assert_eq!(acta_de_json(&acta_a_json(a)).as_ref(), Ok(a));
        }
        let j = acta_a_json(&genesis);
        assert!(j["acta"]["procedencia"].is_null() && j["firmaAnterior"].is_null());
        assert_eq!(j["acta"]["esquema"], "0x100000005");
        assert_eq!(j["acta"]["desde"], "0x0");
        assert_eq!(j["firma"], "0xababababab");

        let mut con_mas = acta_a_json(&rotacion);
        con_mas["v"] = json!(1);
        con_mas["tipo"] = json!("acta");
        assert_eq!(acta_de_json(&con_mas).as_ref(), Ok(&rotacion));

        let rojo = |cambio: &dyn Fn(&mut Value), campo: &str| {
            let mut v = acta_a_json(&rotacion);
            cambio(&mut v);
            let e = acta_de_json(&v).expect_err(campo);
            assert!(e.contains(campo), "{campo}: {e}");
        };
        rojo(
            &|v| drop(v.as_object_mut().expect("o").remove("firmaAnterior")),
            "firmaAnterior",
        );
        rojo(
            &|v| drop(v["acta"].as_object_mut().expect("o").remove("procedencia")),
            "procedencia",
        );
        rojo(&|v| v["acta"]["desde"] = json!("29"), "desde");
        rojo(
            &|v| v["acta"]["procedencia"]["mmrRoot"] = json!("0x00"),
            "mmrRoot",
        );
        let p_le = hex(&0xFFFF_FFFF_0000_0001u64.to_le_bytes());
        let con = |x: &str| json!(format!("{x}{}", "00".repeat(24)));
        rojo(&|v| v["acta"]["siguiente"] = con(&p_le), "siguiente");
        let mut borde = acta_a_json(&rotacion);
        borde["acta"]["siguiente"] = con(&hex(&0xFFFF_FFFF_0000_0000u64.to_le_bytes()));
        assert!(acta_de_json(&borde).is_ok(), "p - 1 es un elemento");
        rojo(&|v| v["firma"] = json!("0xabc"), "firma");
        rojo(&|v| v["acta"]["clave"] = json!(7), "clave");
        assert!(acta_de_json(&json!({"firma": "0x00"})).is_err(), "sin acta");
    }
}
