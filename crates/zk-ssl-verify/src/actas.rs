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

use xmss::Signature;
use zk_ssl_hash::{acta_digest, digest_to_bytes, huella_de_clave, Digest};

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
}
