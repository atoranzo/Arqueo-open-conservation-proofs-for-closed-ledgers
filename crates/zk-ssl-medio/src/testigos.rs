//! # Los testigos del medio: el cliente `add-checkpoint`, la retención de sus cofirmas y la nota cofirmada (RFC-0013 D-D; E3a, §808)
//!
//! El publicador envía a cada testigo su nota con una prueba de consistencia
//! desde el último tamaño que ese testigo cofirmó, y el testigo, si todo
//! cuadra, devuelve su cofirma (`tlog-witness` v1.1.0 de C2SP, commit
//! `b2dd989`, `add-checkpoint`). Este módulo es **el lado del publicador**
//! de ese protocolo:
//!
//! - [`peticion`] arma el cuerpo: `old N`, hasta 63 líneas de prueba en
//!   base64, una línea en blanco y la nota.
//! - [`leer_respuesta`] lee lo que contesta el testigo: las líneas de firma de
//!   un `200`, el tamaño que recuerda en un `409` (`text/x.tlog.size`), y los
//!   rechazos `400`, `403`, `404` y `422`.
//! - [`pedir_cofirma`] junta las dos con el árbol del medio. Si el testigo
//!   recuerda otro tamaño, reintenta UNA vez desde el suyo; si recuerda uno
//!   **mayor** que el de la nota, para y lo dice ([`Resultado::Retroceso`]):
//!   es la detección de la 103 que D-D le pide al medio, y no se esquiva.
//! - [`Retencion`] guarda la cofirma verificada de cada testigo, una por
//!   testigo, y escribe la **nota cofirmada**: la del publicador y detrás
//!   esas líneas, que es lo que el sobre `ancla-cofirmada` del kit lee
//!   (§633). [`publicar`] la deja en el fichero `checkpoint` de un
//!   directorio, por renombrado atómico, y no deja que lo publicado
//!   retroceda.
//! - [`HttpLocal`] es el transporte: HTTP/1.1 sobre TCP, **solo** a una
//!   dirección de bucle local y sin TLS.
//!
//! ## Las reglas, y de dónde sale cada una
//!
//! De `tlog-witness` v1.1.0: la línea `old` en decimal sin ceros a la
//! izquierda; como mucho 63 líneas de prueba; desde `old 0`, ninguna; el
//! tamaño viejo no pasa del de la nota (el testigo contestaría `400`); el
//! `409` lleva el tamaño recordado en decimal y un salto, con
//! `Content-Type: text/x.tlog.size`; el `200`, una o más líneas de firma que
//! empiezan por `—` y acaban en salto; y **el cliente ignora las firmas de
//! claves que no conoce**. Para leerlas, la especificación permite pegar la
//! respuesta a la nota y pasarla por el verificador de notas con las claves
//! de los testigos en los que se confía: es lo que se hace, con
//! [`verificar_nota`] y [`ClaveDeNota::verificar_cofirma`], así que las
//! reglas de cada línea son las de `signed-note` que ya aplica
//! [`crate::nota`].
//!
//! ⚠️ **Tres decisiones del cliente, más estrictas que la especificación, y
//! declaradas:**
//!
//! - **Al testigo se le envía la nota del publicador sola**, sin las cofirmas
//!   de otros testigos. La especificación lo permite, y pegadas caben poco:
//!   litewitness limita el cuerpo a 10 KiB (`MaxBytesHandler`, 10.240
//!   bytes), la nota del medio con su firma son 3.445 y cada cofirma
//!   ML-DSA-44, 3.277 con el nombre de `torchwood-a`. Medido con su motor
//!   (§808): con las dos de `torchwood-a` y `-b` pegadas, cabe `old 0`
//!   (10.006 bytes, `200`) y caben hasta 5 líneas de prueba (10.231, `200`);
//!   con una sexta (10.276), o con la línea Ed25519 de litewitness detrás
//!   (10.368), ya no: `500`. Con tres cofirmas ML-DSA-44 no cabe ni `old 0`
//!   (13.283). La prueba crece con el árbol: se envía sola.
//! - **Una línea de la clave del testigo que no verifica es un error**, no
//!   una línea más que ignorar: el testigo, o lo que hay entre los dos,
//!   miente, y el publicador lo tiene que ver.
//! - **El `409` sin su `Content-Type`, o con un tamaño no canónico, es un
//!   error**: el publicador no reintenta sobre un número mal escrito.
//!
//! ## ⚠️ Lo que este módulo NO hace
//!
//! - **No habla con testigos de verdad.** [`HttpLocal`] rechaza toda
//!   dirección que no sea de bucle local, y no hay TLS: con qué transporte se
//!   llega a un testigo público, qué testigos se eligen y con qué cadencia se
//!   publica es de otra etapa, y pide red.
//! - **No verifica cofirmas Ed25519.** El testigo de referencia,
//!   `litewitness`, cofirma con la Ed25519 de su agente SSH; la casa no
//!   enlaza curvas (§106, §705, `tools/check_curvas.py`), así que una
//!   cofirma así es una línea de una clave que el cliente no conoce, y se
//!   ignora como manda la especificación. Lo que se retiene son cofirmas
//!   `0x06`, que es lo que el kit sabe verificar.
//! - **No decide la frescura** de las marcas de tiempo: las devuelve.
//! - **No recuerda entre arranques** el tamaño que cofirmó cada testigo: lo
//!   aprende del `409`, que es para lo que la especificación lo deja.
//! - **No juzga al testigo que olvida.** Si recuerda MENOS de lo que el
//!   publicador creía, el reintento desde su tamaño cofirma, y
//!   [`Resultado::Cofirmada`] dice en `viejo` desde dónde: qué se hace con
//!   un testigo que retrocede es política de quien lo elige, no del cliente.
//! - **No usa keep-alive** (un SHOULD) **ni pide `sign-subtree`** (OPCIONAL).

use std::fs;
use std::io::{ErrorKind, Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::path::Path;
use std::time::{Duration, Instant};

use crate::base64::base64_encode;
use crate::hash::HashValue;
use crate::medio::{ArbolDelMedio, ErrorDelMedio};
use crate::nota::{verificar_nota, Checkpoint, ClaveDeNota, ErrorDeNota, FirmaAjena};

/// Líneas de prueba que un cliente envía como mucho (`tlog-witness`).
pub const MAX_LINEAS_DE_PRUEBA: usize = 63;
/// El tipo del cuerpo de un `409`.
pub const TIPO_DEL_TAMANO: &str = "text/x.tlog.size";
/// Lo que se añade al prefijo de envío de un testigo.
pub const RUTA: &str = "/add-checkpoint";
/// Bytes de cuerpo que se leen como mucho: 100 líneas ML-DSA-44, el máximo
/// de una nota, son unos 330 KB.
pub const MAX_CUERPO: usize = 512 * 1024;
/// Bytes de cabeceras que se leen como mucho.
const MAX_CABECERAS: usize = 16 * 1024;
/// El fichero donde se publica la nota cofirmada, en la convención de
/// `tlog-checkpoint` (RFC-0013, «Lo que se DESCARTÓ»: fuera del cable).
pub const FICHERO: &str = "checkpoint";
/// `— `: raya (U+2014) y espacio.
const PREFIJO_FIRMA: &str = "\u{2014} ";

/// Lo que puede fallar al pedir, leer, retener o publicar una cofirma.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorDeTestigo {
    /// Lo que el publicador iba a enviar no es una petición válida.
    Peticion(&'static str),
    /// La nota del publicador no verifica, o la respuesta pegada a ella no
    /// se lee como líneas de firma.
    Nota(ErrorDeNota),
    /// El árbol del publicador no llega al tamaño que se le pide.
    Arbol(ErrorDelMedio),
    /// El árbol del publicador tiene otra raíz en el tamaño de la nota.
    OtroArbol,
    /// La respuesta HTTP, o su cuerpo, no es lo que dice el protocolo.
    Respuesta(String),
    /// Una línea de la clave del testigo que no verifica sobre el
    /// checkpoint.
    CofirmaInvalida { testigo: String },
    /// La dirección no es `http://` a una dirección de bucle local.
    Direccion(String),
    /// Lo que pasó en el socket.
    Transporte(String),
    /// Lo que pasó en el disco.
    Fichero(String),
    /// Lo publicado tiene más anclas que lo que se iba a publicar.
    PublicadoMayor { publicado: u64, nuevo: u64 },
    /// Lo publicado tiene el mismo tamaño y otra raíz.
    PublicadoDistinto { tamano: u64 },
}

impl core::fmt::Display for ErrorDeTestigo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ErrorDeTestigo::Peticion(s) => write!(f, "petición: {s}"),
            ErrorDeTestigo::Nota(e) => write!(f, "{e}"),
            ErrorDeTestigo::Arbol(e) => write!(f, "árbol del medio: {e}"),
            ErrorDeTestigo::OtroArbol => {
                write!(f, "el árbol del medio no tiene la raíz de la nota")
            }
            ErrorDeTestigo::Respuesta(s) => write!(f, "respuesta del testigo: {s}"),
            ErrorDeTestigo::CofirmaInvalida { testigo } => {
                write!(f, "la cofirma de {testigo} no verifica")
            }
            ErrorDeTestigo::Direccion(s) => write!(f, "dirección: {s}"),
            ErrorDeTestigo::Transporte(s) => write!(f, "transporte: {s}"),
            ErrorDeTestigo::Fichero(s) => write!(f, "fichero: {s}"),
            ErrorDeTestigo::PublicadoMayor { publicado, nuevo } => write!(
                f,
                "lo publicado lleva {publicado} anclas y lo nuevo {nuevo}: no se retrocede"
            ),
            ErrorDeTestigo::PublicadoDistinto { tamano } => write!(
                f,
                "lo publicado lleva {tamano} anclas con otra raíz: es una bifurcación"
            ),
        }
    }
}

impl std::error::Error for ErrorDeTestigo {}

impl From<ErrorDeNota> for ErrorDeTestigo {
    fn from(e: ErrorDeNota) -> Self {
        ErrorDeTestigo::Nota(e)
    }
}

impl From<ErrorDelMedio> for ErrorDeTestigo {
    fn from(e: ErrorDelMedio) -> Self {
        ErrorDeTestigo::Arbol(e)
    }
}

/// **El cuerpo de `add-checkpoint`**: `old <viejo>`, cada hash de la prueba
/// en base64 en su línea, una línea en blanco y la nota.
pub fn peticion(viejo: u64, prueba: &[HashValue], nota: &str) -> Result<String, ErrorDeTestigo> {
    if prueba.len() > MAX_LINEAS_DE_PRUEBA {
        return Err(ErrorDeTestigo::Peticion("más de 63 líneas de prueba"));
    }
    if viejo == 0 && !prueba.is_empty() {
        return Err(ErrorDeTestigo::Peticion(
            "desde old 0 la prueba va vacía: el árbol vacío es consistente con todos",
        ));
    }
    if !nota.ends_with('\n') {
        return Err(ErrorDeTestigo::Peticion(
            "la nota no acaba en salto de línea",
        ));
    }
    let mut s = format!("old {viejo}\n");
    for h in prueba {
        s.push_str(&base64_encode(h));
        s.push('\n');
    }
    s.push('\n');
    s.push_str(nota);
    Ok(s)
}

/// Lo que devuelve un transporte: el estado, el `Content-Type` y el cuerpo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RespuestaHttp {
    pub estado: u16,
    pub tipo: Option<String>,
    pub cuerpo: Vec<u8>,
}

/// Lo que contesta un testigo, leído según `tlog-witness`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Respuesta {
    /// `200`: las líneas de firma, en su orden y **sin verificar**.
    Firmas(Vec<FirmaAjena>),
    /// `409`: el tamaño que el testigo recuerda haber cofirmado.
    Conflicto(u64),
    /// `400`: la petición no se entiende.
    PeticionMala,
    /// `403`: ninguna firma de una clave que el testigo tenga para el
    /// `origin`, o la que hay no verifica.
    Prohibida,
    /// `404`: el testigo no conoce el `origin`.
    OrigenDesconocido,
    /// `422`: la prueba de consistencia no verifica, o el tamaño es el que
    /// recuerda y la raíz es otra.
    Inconsistente,
    /// Cualquier otro estado.
    Otra(u16),
}

/// El tipo de medio de un `Content-Type`, sin parámetros y en minúsculas.
fn tipo_de_medio(tipo: &str) -> String {
    tipo.split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase()
}

/// **Lee la respuesta** de un testigo a la petición de la nota `nota`.
///
/// Las líneas de un `200` se leen pegadas a la nota con [`verificar_nota`],
/// como permite la especificación: así cada línea pasa las reglas de
/// `signed-note` de [`crate::nota`] (nombre, base64 estricta, al menos 5
/// bytes, como mucho 100 líneas), y una línea que se hiciera pasar por el
/// publicador es una firma repetida. Ninguna se verifica aquí: eso es de
/// [`pedir_cofirma`], que sabe en qué testigo se confía.
pub fn leer_respuesta(
    nota: &str,
    publicador: &ClaveDeNota,
    r: &RespuestaHttp,
) -> Result<Respuesta, ErrorDeTestigo> {
    match r.estado {
        200 => {
            let cuerpo = std::str::from_utf8(&r.cuerpo)
                .map_err(|_| ErrorDeTestigo::Respuesta("el 200 no es UTF-8".into()))?;
            // Vacío, o con otra cosa delante: la especificación pide una o más
            // líneas de firma, y pegado a la nota un cuerpo vacío la dejaría
            // tal cual, sin decir nada.
            if !cuerpo.starts_with(PREFIJO_FIRMA) {
                return Err(ErrorDeTestigo::Respuesta(
                    "el 200 no empieza por una línea de firma".into(),
                ));
            }
            let sola = verificar_nota(nota, publicador)?;
            let pegada = verificar_nota(&format!("{nota}{cuerpo}"), publicador)?;
            if pegada.checkpoint != sola.checkpoint || pegada.ajenas.len() < sola.ajenas.len() {
                return Err(ErrorDeTestigo::Respuesta(
                    "la respuesta cambia el texto de la nota".into(),
                ));
            }
            Ok(Respuesta::Firmas(
                pegada.ajenas[sola.ajenas.len()..].to_vec(),
            ))
        }
        409 => {
            if r.tipo.as_deref().map(tipo_de_medio).as_deref() != Some(TIPO_DEL_TAMANO) {
                return Err(ErrorDeTestigo::Respuesta(format!(
                    "el 409 no es {TIPO_DEL_TAMANO}: {:?}",
                    r.tipo
                )));
            }
            let t = std::str::from_utf8(&r.cuerpo)
                .ok()
                .and_then(|s| s.strip_suffix('\n'))
                .ok_or_else(|| ErrorDeTestigo::Respuesta("el 409 no acaba en salto".into()))?;
            let n: u64 = t
                .parse()
                .map_err(|_| ErrorDeTestigo::Respuesta("el 409 no lleva un tamaño".into()))?;
            if t != n.to_string() {
                return Err(ErrorDeTestigo::Respuesta(
                    "el tamaño del 409 no está en su forma canónica".into(),
                ));
            }
            Ok(Respuesta::Conflicto(n))
        }
        400 => Ok(Respuesta::PeticionMala),
        403 => Ok(Respuesta::Prohibida),
        404 => Ok(Respuesta::OrigenDesconocido),
        422 => Ok(Respuesta::Inconsistente),
        otro => Ok(Respuesta::Otra(otro)),
    }
}

/// Un testigo en el que el publicador confía: su clave ML-DSA-44 (tipo
/// `0x06`) y su prefijo de envío.
#[derive(Clone, Debug)]
pub struct Testigo {
    clave: ClaveDeNota,
    prefijo: String,
}

impl Testigo {
    /// `vkey` de `signed-note` de tipo `0x06` y prefijo de envío (lo que va
    /// delante de `/add-checkpoint`).
    pub fn nuevo(vkey: &str, prefijo: &str) -> Result<Self, ErrorDeTestigo> {
        Ok(Testigo {
            clave: ClaveDeNota::leer_vkey(vkey)?,
            prefijo: prefijo.trim_end_matches('/').to_string(),
        })
    }

    pub fn clave(&self) -> &ClaveDeNota {
        &self.clave
    }

    pub fn prefijo(&self) -> &str {
        &self.prefijo
    }
}

/// Quien lleva la petición al testigo y trae su respuesta.
pub trait Transporte {
    fn add_checkpoint(
        &mut self,
        testigo: &Testigo,
        cuerpo: &[u8],
    ) -> Result<RespuestaHttp, ErrorDeTestigo>;
}

/// Una cofirma de un testigo: su línea, ya verificada sobre un checkpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cofirma {
    pub linea: FirmaAjena,
    /// La marca de tiempo que el testigo firmó.
    pub marca: u64,
}

impl Cofirma {
    /// La línea de firma, con su salto final: `— nombre base64(key_id ||
    /// marca || firma)`.
    pub fn texto(&self) -> String {
        let mut bytes = self.linea.id.to_vec();
        bytes.extend_from_slice(&self.linea.firma);
        format!(
            "{PREFIJO_FIRMA}{} {}\n",
            self.linea.nombre,
            base64_encode(&bytes)
        )
    }
}

/// Lo que se sabe al acabar de pedirle a un testigo su cofirma.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resultado {
    /// El testigo cofirmó, y su línea verifica con su clave. `viejo` es el
    /// tamaño que aceptó como suyo; `ignoradas`, las líneas de claves que
    /// no son la suya.
    Cofirmada {
        cofirma: Cofirma,
        viejo: u64,
        ignoradas: usize,
    },
    /// El testigo contestó `200` y ninguna línea es de su clave: avanzó su
    /// estado, y no dio nada que el publicador pueda retener. Es lo que pasa
    /// con un testigo que cofirma con una clave de otro tipo.
    SinSuCofirma { viejo: u64, ignoradas: usize },
    /// ⚠️ **El testigo recuerda un tamaño MAYOR que el de la nota.** O el
    /// medio retrocedió —el directorio restaurado de la 103—, o alguien con la
    /// clave del publicador publicó más anclas. No se reintenta: se dice.
    Retroceso { recordado: u64, nota: u64 },
    /// El testigo no cofirma. Un `409` que se repite tras el reintento
    /// también acaba aquí.
    Rechazada(Respuesta),
}

/// La prueba de consistencia que se envía desde `viejo`.
fn prueba_desde(
    arbol: &ArbolDelMedio,
    viejo: u64,
    tamano: u64,
) -> Result<Vec<HashValue>, ErrorDeTestigo> {
    if viejo > tamano {
        return Err(ErrorDeTestigo::Peticion(
            "el tamaño viejo pasa del de la nota: el testigo contestaría 400",
        ));
    }
    if viejo == 0 {
        return Ok(Vec::new());
    }
    Ok(arbol.prueba_de_consistencia(viejo, tamano)?)
}

/// De las líneas de un `200`, la del testigo, verificada.
fn cofirma_del_testigo(
    testigo: &Testigo,
    checkpoint: &Checkpoint,
    lineas: Vec<FirmaAjena>,
    viejo: u64,
) -> Result<Resultado, ErrorDeTestigo> {
    let clave = &testigo.clave;
    let mut suya = None;
    let mut ignoradas = 0;
    for linea in lineas {
        if linea.nombre != clave.nombre() || linea.id != clave.id() {
            ignoradas += 1;
            continue;
        }
        let marca = clave.verificar_cofirma(checkpoint, &linea).map_err(|_| {
            ErrorDeTestigo::CofirmaInvalida {
                testigo: clave.nombre().to_string(),
            }
        })?;
        // Dos líneas válidas de la misma clave: se retiene la primera, y
        // la otra cuenta como ignorada (el kit rechaza dos líneas del mismo
        // testigo en una nota, §633).
        if suya.is_none() {
            suya = Some(Cofirma { linea, marca });
        } else {
            ignoradas += 1;
        }
    }
    Ok(match suya {
        Some(cofirma) => Resultado::Cofirmada {
            cofirma,
            viejo,
            ignoradas,
        },
        None => Resultado::SinSuCofirma { viejo, ignoradas },
    })
}

/// **Pide su cofirma a un testigo** para la nota `nota` del publicador,
/// cuyo árbol es `arbol`, empezando por el tamaño `viejo` que se cree que
/// el testigo recuerda (0 si no se sabe).
///
/// Antes de enviar nada comprueba que la nota es del publicador, que va
/// sola, y que el árbol tiene en su tamaño la raíz que la nota dice: una
/// prueba de consistencia de otro árbol no se manda.
///
/// Un `409` con un tamaño que no pasa del de la nota se reintenta UNA vez
/// desde ese tamaño; con uno mayor, es [`Resultado::Retroceso`] y no se
/// reintenta.
pub fn pedir_cofirma<T: Transporte>(
    transporte: &mut T,
    testigo: &Testigo,
    publicador: &ClaveDeNota,
    nota: &str,
    arbol: &ArbolDelMedio,
    viejo: u64,
) -> Result<Resultado, ErrorDeTestigo> {
    let v = verificar_nota(nota, publicador)?;
    if !v.ajenas.is_empty() {
        return Err(ErrorDeTestigo::Peticion(
            "la nota lleva otras firmas: al testigo se le envía la del publicador sola",
        ));
    }
    let c = v.checkpoint;
    if arbol.raiz_en(c.tamano)? != c.raiz {
        return Err(ErrorDeTestigo::OtroArbol);
    }
    let mut viejo = viejo;
    let mut reintentado = false;
    loop {
        let cuerpo = peticion(viejo, &prueba_desde(arbol, viejo, c.tamano)?, nota)?;
        let r = transporte.add_checkpoint(testigo, cuerpo.as_bytes())?;
        match leer_respuesta(nota, publicador, &r)? {
            Respuesta::Firmas(lineas) => return cofirma_del_testigo(testigo, &c, lineas, viejo),
            Respuesta::Conflicto(m) if m > c.tamano => {
                return Ok(Resultado::Retroceso {
                    recordado: m,
                    nota: c.tamano,
                })
            }
            Respuesta::Conflicto(m) if !reintentado && m != viejo => {
                viejo = m;
                reintentado = true;
            }
            otra => return Ok(Resultado::Rechazada(otra)),
        }
    }
}

/// **Las cofirmas retenidas de una nota**: la nota del publicador, sola, y
/// la cofirma de cada testigo que la cofirmó, una por testigo y todas
/// verificadas sobre su checkpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Retencion {
    nota: String,
    checkpoint: Checkpoint,
    cofirmas: Vec<Cofirma>,
}

impl Retencion {
    /// Empieza la retención de la nota `nota`, que tiene que ser del
    /// publicador y no llevar otras firmas.
    pub fn nueva(nota: &str, publicador: &ClaveDeNota) -> Result<Self, ErrorDeTestigo> {
        let v = verificar_nota(nota, publicador)?;
        if !v.ajenas.is_empty() {
            return Err(ErrorDeTestigo::Peticion(
                "la retención empieza con la nota del publicador sola",
            ));
        }
        Ok(Retencion {
            nota: nota.to_string(),
            checkpoint: v.checkpoint,
            cofirmas: Vec::new(),
        })
    }

    /// **Retiene la cofirma** de `testigo`. La vuelve a verificar sobre el
    /// checkpoint de ESTA nota —una cofirma de otro checkpoint no se
    /// retiene, venga de donde venga— y devuelve `false` sin tocar nada si
    /// ese testigo ya tiene una: la primera se queda.
    pub fn retener(
        &mut self,
        testigo: &ClaveDeNota,
        cofirma: &Cofirma,
    ) -> Result<bool, ErrorDeTestigo> {
        let marca = testigo
            .verificar_cofirma(&self.checkpoint, &cofirma.linea)
            .map_err(|_| ErrorDeTestigo::CofirmaInvalida {
                testigo: testigo.nombre().to_string(),
            })?;
        if marca != cofirma.marca {
            return Err(ErrorDeTestigo::CofirmaInvalida {
                testigo: testigo.nombre().to_string(),
            });
        }
        if self
            .cofirmas
            .iter()
            .any(|c| c.linea.nombre == cofirma.linea.nombre && c.linea.id == cofirma.linea.id)
        {
            return Ok(false);
        }
        self.cofirmas.push(cofirma.clone());
        Ok(true)
    }

    pub fn checkpoint(&self) -> &Checkpoint {
        &self.checkpoint
    }

    /// La nota del publicador, sola.
    pub fn nota(&self) -> &str {
        &self.nota
    }

    pub fn cofirmas(&self) -> &[Cofirma] {
        &self.cofirmas
    }

    /// **La nota cofirmada**: la del publicador y, detrás, cada cofirma
    /// retenida en el orden en que se retuvo.
    pub fn nota_cofirmada(&self) -> String {
        let mut s = self.nota.clone();
        for c in &self.cofirmas {
            s.push_str(&c.texto());
        }
        s
    }

    /// **Relee una nota cofirmada** —la que [`publicar`] dejó, por ejemplo—
    /// con la clave del publicador y las de los testigos en los que se
    /// confía. La línea del publicador tiene que ser la primera, y cada una
    /// de las demás, de uno de esos testigos y verificar: una línea de otra
    /// clave en lo que el publicador publicó no la puso él.
    pub fn leer(
        nota_cofirmada: &str,
        publicador: &ClaveDeNota,
        testigos: &[ClaveDeNota],
    ) -> Result<Self, ErrorDeTestigo> {
        let corte =
            nota_cofirmada
                .rfind("\n\n")
                .ok_or(ErrorDeTestigo::Nota(ErrorDeNota::Malformada(
                    "sin línea en blanco",
                )))?;
        let firmas = &nota_cofirmada[corte + 2..];
        let primera = firmas.find('\n').map(|i| i + 1).unwrap_or(firmas.len());
        let mut r = Retencion::nueva(&nota_cofirmada[..corte + 2 + primera], publicador)?;
        let v = verificar_nota(nota_cofirmada, publicador)?;
        for linea in v.ajenas {
            let clave = testigos
                .iter()
                .find(|t| t.nombre() == linea.nombre && t.id() == linea.id)
                .ok_or_else(|| {
                    ErrorDeTestigo::Respuesta(format!(
                        "una línea de {} con una clave que no es de ningún testigo",
                        linea.nombre
                    ))
                })?;
            let marca = clave
                .verificar_cofirma(&r.checkpoint, &linea)
                .map_err(|_| ErrorDeTestigo::CofirmaInvalida {
                    testigo: linea.nombre.clone(),
                })?;
            if !r.retener(clave, &Cofirma { linea, marca })? {
                return Err(ErrorDeTestigo::Respuesta(format!(
                    "dos líneas de {}",
                    clave.nombre()
                )));
            }
        }
        Ok(r)
    }
}

/// **Publica la nota cofirmada** en `<dir>/checkpoint`: la escribe en un
/// temporal del mismo directorio, la lleva al disco y la renombra encima.
///
/// ⚠️ **Lo publicado no retrocede.** Si ya hay un `checkpoint`, tiene que
/// verificar con la clave del publicador —uno que no verifica no lo puso él,
/// y no se pisa—, no puede llevar más anclas que el nuevo, y con las mismas
/// tiene que llevar la misma raíz. Es la propiedad que D-B le pide al medio,
/// aplicada a lo que el propio publicador sirve.
pub fn publicar(
    dir: &Path,
    retencion: &Retencion,
    publicador: &ClaveDeNota,
) -> Result<(), ErrorDeTestigo> {
    let ruta = dir.join(FICHERO);
    match fs::read_to_string(&ruta) {
        Ok(previo) => {
            let v = verificar_nota(&previo, publicador).map_err(|e| {
                ErrorDeTestigo::Fichero(format!(
                    "el checkpoint publicado no verifica con el publicador ({e}): no se pisa"
                ))
            })?;
            let nuevo = &retencion.checkpoint;
            if v.checkpoint.tamano > nuevo.tamano {
                return Err(ErrorDeTestigo::PublicadoMayor {
                    publicado: v.checkpoint.tamano,
                    nuevo: nuevo.tamano,
                });
            }
            if v.checkpoint.tamano == nuevo.tamano && v.checkpoint.raiz != nuevo.raiz {
                return Err(ErrorDeTestigo::PublicadoDistinto {
                    tamano: nuevo.tamano,
                });
            }
        }
        Err(e) if e.kind() == ErrorKind::NotFound => {}
        Err(e) => return Err(ErrorDeTestigo::Fichero(e.to_string())),
    }
    let temporal = dir.join(format!("{FICHERO}.nuevo"));
    let escribir = || -> std::io::Result<()> {
        let mut f = fs::File::create(&temporal)?;
        f.write_all(retencion.nota_cofirmada().as_bytes())?;
        f.sync_all()?;
        fs::rename(&temporal, &ruta)?;
        fs::File::open(dir)?.sync_all()
    };
    escribir().map_err(|e| ErrorDeTestigo::Fichero(e.to_string()))
}

/// Un prefijo de envío local: `http://` a una dirección de bucle.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Local {
    direccion: SocketAddr,
    anfitrion: String,
    ruta: String,
}

/// Lee un prefijo de envío y **exige que sea de bucle local**: `http://`,
/// una IPv4 de `127.0.0.0/8`, `[::1]` o `localhost`, el puerto, y la ruta si
/// la hay. Sin DNS: `localhost` es `127.0.0.1`.
fn local(prefijo: &str) -> Result<Local, ErrorDeTestigo> {
    if prefijo.starts_with("https://") {
        return Err(ErrorDeTestigo::Direccion(
            "https: no hay TLS en E3a, que solo habla con testigos locales".into(),
        ));
    }
    let resto = prefijo
        .strip_prefix("http://")
        .ok_or_else(|| ErrorDeTestigo::Direccion(format!("{prefijo:?} no empieza por http://")))?;
    let (autoridad, ruta) = match resto.find('/') {
        Some(i) => (&resto[..i], &resto[i..]),
        None => (resto, ""),
    };
    let (anfitrion, puerto) = if let Some(v6) = autoridad.strip_prefix('[') {
        let (ip, p) = v6
            .split_once("]:")
            .ok_or_else(|| ErrorDeTestigo::Direccion(format!("{autoridad:?} sin puerto")))?;
        (ip, p)
    } else {
        autoridad
            .rsplit_once(':')
            .ok_or_else(|| ErrorDeTestigo::Direccion(format!("{autoridad:?} sin puerto")))?
    };
    let puerto: u16 = puerto
        .parse()
        .map_err(|_| ErrorDeTestigo::Direccion(format!("{puerto:?} no es un puerto")))?;
    let ip: IpAddr = if anfitrion == "localhost" {
        IpAddr::V4(Ipv4Addr::LOCALHOST)
    } else {
        anfitrion
            .parse()
            .map_err(|_| ErrorDeTestigo::Direccion(format!("{anfitrion:?} no es una IP")))?
    };
    if !ip.is_loopback() {
        return Err(ErrorDeTestigo::Direccion(format!(
            "{ip} no es de bucle local: E3a no habla con testigos de verdad"
        )));
    }
    if ruta.contains(|c: char| c.is_whitespace() || c.is_control()) {
        return Err(ErrorDeTestigo::Direccion(format!(
            "{ruta:?} no es una ruta"
        )));
    }
    Ok(Local {
        direccion: SocketAddr::new(ip, puerto),
        anfitrion: autoridad.to_string(),
        ruta: ruta.to_string(),
    })
}

/// Busca `aguja` en `pajar` desde `desde`.
fn buscar(pajar: &[u8], aguja: &[u8], desde: usize) -> Option<usize> {
    pajar
        .get(desde..)?
        .windows(aguja.len())
        .position(|w| w == aguja)
        .map(|i| i + desde)
}

/// El cuerpo en `Transfer-Encoding: chunked` (RFC 9112 §7.1), sin las
/// cabeceras finales.
fn sin_trozos(mut b: &[u8]) -> Result<Vec<u8>, ErrorDeTestigo> {
    let mal = |s: &str| ErrorDeTestigo::Respuesta(format!("cuerpo por trozos: {s}"));
    let mut cuerpo = Vec::new();
    loop {
        let fin = buscar(b, b"\r\n", 0).ok_or_else(|| mal("un tamaño sin CRLF"))?;
        let linea = std::str::from_utf8(&b[..fin]).map_err(|_| mal("un tamaño no ASCII"))?;
        let hex = linea.split(';').next().unwrap_or("").trim();
        if hex.is_empty() || hex.len() > 8 {
            return Err(mal("un tamaño vacío o enorme"));
        }
        let n = usize::from_str_radix(hex, 16).map_err(|_| mal("un tamaño no hexadecimal"))?;
        b = &b[fin + 2..];
        if n == 0 {
            return Ok(cuerpo);
        }
        if cuerpo.len() + n > MAX_CUERPO {
            return Err(mal("pasa del máximo"));
        }
        if b.len() < n + 2 || &b[n..n + 2] != b"\r\n" {
            return Err(mal("un trozo cortado"));
        }
        cuerpo.extend_from_slice(&b[..n]);
        b = &b[n + 2..];
    }
}

/// **Lee una respuesta HTTP/1.1 entera** (la conexión se cerró): la línea de
/// estado, las cabeceras y el cuerpo, por `Content-Length`, por trozos o
/// hasta el final.
fn leer_http(b: &[u8]) -> Result<RespuestaHttp, ErrorDeTestigo> {
    let mal = |s: String| ErrorDeTestigo::Respuesta(s);
    let fin = buscar(b, b"\r\n\r\n", 0)
        .filter(|&i| i <= MAX_CABECERAS)
        .ok_or_else(|| mal("sin el final de las cabeceras".into()))?;
    let cabeza = std::str::from_utf8(&b[..fin]).map_err(|_| mal("cabeceras no ASCII".into()))?;
    let mut lineas = cabeza.split("\r\n");
    let estado_linea = lineas.next().unwrap_or("");
    let mut partes = estado_linea.splitn(3, ' ');
    let version = partes.next().unwrap_or("");
    let codigo = partes.next().unwrap_or("");
    if !version.starts_with("HTTP/1.") || codigo.len() != 3 {
        return Err(mal(format!("línea de estado {estado_linea:?}")));
    }
    let estado: u16 = codigo
        .parse()
        .map_err(|_| mal(format!("estado {codigo:?}")))?;
    let (mut tipo, mut largo, mut trozos) = (None, None, false);
    for l in lineas {
        let (nombre, valor) = l
            .split_once(':')
            .ok_or_else(|| mal(format!("cabecera {l:?}")))?;
        let valor = valor.trim();
        match nombre.trim().to_ascii_lowercase().as_str() {
            "content-type" => tipo = Some(valor.to_string()),
            "content-length" => {
                let n: usize = valor
                    .parse()
                    .map_err(|_| mal(format!("Content-Length {valor:?}")))?;
                if largo.is_some_and(|m| m != n) {
                    return Err(mal("dos Content-Length distintos".into()));
                }
                largo = Some(n);
            }
            "transfer-encoding" => trozos = valor.to_ascii_lowercase().ends_with("chunked"),
            _ => {}
        }
    }
    let resto = &b[fin + 4..];
    let cuerpo = if trozos {
        sin_trozos(resto)?
    } else if let Some(n) = largo {
        if n > MAX_CUERPO || resto.len() < n {
            return Err(mal(format!(
                "Content-Length {n} y llegan {} bytes",
                resto.len()
            )));
        }
        resto[..n].to_vec()
    } else {
        resto.to_vec()
    };
    if cuerpo.len() > MAX_CUERPO {
        return Err(mal("el cuerpo pasa del máximo".into()));
    }
    Ok(RespuestaHttp {
        estado,
        tipo,
        cuerpo,
    })
}

/// **El transporte local**: un `POST` HTTP/1.1 por conexión, con
/// `Connection: close`, a un prefijo de bucle local y sin TLS.
#[derive(Clone, Debug)]
pub struct HttpLocal {
    /// El plazo de conexión, de escritura y de lectura.
    pub plazo: Duration,
}

impl Default for HttpLocal {
    fn default() -> Self {
        HttpLocal {
            plazo: Duration::from_secs(10),
        }
    }
}

impl Transporte for HttpLocal {
    fn add_checkpoint(
        &mut self,
        testigo: &Testigo,
        cuerpo: &[u8],
    ) -> Result<RespuestaHttp, ErrorDeTestigo> {
        let d = local(testigo.prefijo())?;
        let io = |e: std::io::Error| ErrorDeTestigo::Transporte(e.to_string());
        let mut s = TcpStream::connect_timeout(&d.direccion, self.plazo).map_err(io)?;
        s.set_read_timeout(Some(self.plazo)).map_err(io)?;
        s.set_write_timeout(Some(self.plazo)).map_err(io)?;
        let cabeza = format!(
            "POST {}{RUTA} HTTP/1.1\r\nHost: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            d.ruta,
            d.anfitrion,
            cuerpo.len()
        );
        s.write_all(cabeza.as_bytes()).map_err(io)?;
        s.write_all(cuerpo).map_err(io)?;
        s.flush().map_err(io)?;
        // Hasta que el testigo cierre, con un tope de bytes y un plazo TOTAL:
        // el de lectura vale para cada `read`, y un testigo que suelta un
        // byte cada poco lo renovaría sin fin.
        let mut leido = Vec::new();
        let mut trozo = [0u8; 8192];
        let tope = MAX_CABECERAS + 2 * MAX_CUERPO;
        let limite = Instant::now() + 3 * self.plazo;
        loop {
            let n = s.read(&mut trozo).map_err(io)?;
            if n == 0 {
                break;
            }
            leido.extend_from_slice(&trozo[..n]);
            if leido.len() > tope {
                return Err(ErrorDeTestigo::Respuesta(
                    "la respuesta pasa del máximo".into(),
                ));
            }
            if Instant::now() > limite {
                return Err(ErrorDeTestigo::Transporte(
                    "el testigo no acaba de contestar".into(),
                ));
            }
        }
        leer_http(&leido)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hash::sha256;
    use crate::nota::{origen_del_medio, Cofirmante, Publicador};

    const HUELLA: [u8; 32] = [0x5a; 32];

    fn publicador() -> Publicador {
        Publicador::determinista(&origen_del_medio(&HUELLA), [7; 32]).unwrap()
    }

    fn arbol(n: u64) -> ArbolDelMedio {
        let mut a = ArbolDelMedio::nuevo();
        for i in 0..n {
            a.anadir(&sha256(&i.to_be_bytes()));
        }
        a
    }

    fn nota(p: &Publicador, a: &ArbolDelMedio) -> String {
        let c = Checkpoint {
            origen: p.clave_publica().nombre().to_string(),
            tamano: a.tamano(),
            raiz: a.raiz(),
        };
        p.firmar(&c, 1_790_000_000).unwrap()
    }

    fn http(estado: u16, tipo: Option<&str>, cuerpo: &str) -> RespuestaHttp {
        RespuestaHttp {
            estado,
            tipo: tipo.map(str::to_string),
            cuerpo: cuerpo.as_bytes().to_vec(),
        }
    }

    #[test]
    fn la_peticion_byte_a_byte_y_sus_reglas() {
        let p = publicador();
        let a = arbol(7);
        let n = nota(&p, &a);
        let prueba = a.prueba_de_consistencia(3, 7).unwrap();
        let mut esperado = String::from("old 3\n");
        for h in &prueba {
            esperado.push_str(&base64_encode(h));
            esperado.push('\n');
        }
        esperado.push('\n');
        esperado.push_str(&n);
        assert_eq!(peticion(3, &prueba, &n).unwrap(), esperado);
        // old 0: sin prueba, y la línea es «old 0», no «old ».
        assert!(peticion(0, &[], &n)
            .unwrap()
            .starts_with("old 0\n\nzkssl/v1/"));
        assert!(peticion(0, &prueba, &n).is_err());
        assert!(peticion(3, &vec![[0; 32]; 64], &n).is_err());
        assert!(peticion(3, &vec![[0; 32]; 63], &n).is_ok());
        assert!(peticion(3, &prueba, n.trim_end()).is_err());
        // El tamaño viejo no pasa del de la nota: el cliente no lo envía.
        assert!(prueba_desde(&a, 8, 7).is_err());
        assert!(prueba_desde(&a, 7, 7).unwrap().is_empty());
    }

    #[test]
    fn el_409_solo_con_su_tipo_y_su_tamano_canonico() {
        let p = publicador();
        let n = nota(&p, &arbol(5));
        let k = p.clave_publica();
        let leer = |r: RespuestaHttp| leer_respuesta(&n, k, &r);
        assert_eq!(
            leer(http(409, Some("text/x.tlog.size"), "9\n")),
            Ok(Respuesta::Conflicto(9))
        );
        assert_eq!(
            leer(http(409, Some("Text/X.Tlog.Size; charset=utf-8"), "0\n")),
            Ok(Respuesta::Conflicto(0))
        );
        for (tipo, cuerpo) in [
            (None, "9\n"),
            (Some("text/plain"), "9\n"),
            (Some(TIPO_DEL_TAMANO), "9"),
            (Some(TIPO_DEL_TAMANO), "09\n"),
            (Some(TIPO_DEL_TAMANO), "+9\n"),
            (Some(TIPO_DEL_TAMANO), " 9\n"),
            (Some(TIPO_DEL_TAMANO), "\n"),
            (Some(TIPO_DEL_TAMANO), "18446744073709551616\n"),
        ] {
            assert!(
                leer(http(409, tipo, cuerpo)).is_err(),
                "{tipo:?} {cuerpo:?}"
            );
        }
        assert_eq!(leer(http(400, None, "x")), Ok(Respuesta::PeticionMala));
        assert_eq!(leer(http(403, None, "")), Ok(Respuesta::Prohibida));
        assert_eq!(leer(http(404, None, "")), Ok(Respuesta::OrigenDesconocido));
        assert_eq!(leer(http(422, None, "")), Ok(Respuesta::Inconsistente));
        assert_eq!(leer(http(500, None, "")), Ok(Respuesta::Otra(500)));
    }

    /// Las líneas del `200` pasan las reglas de la nota: una línea del
    /// propio publicador es una firma repetida, una línea en blanco cambia
    /// el texto, y un cuerpo vacío o sin raya no es una respuesta.
    #[test]
    fn el_200_son_lineas_de_firma_y_nada_mas() {
        let p = publicador();
        let a = arbol(5);
        let n = nota(&p, &a);
        let k = p.clave_publica();
        let t = Cofirmante::determinista("testigo.invalid/t", [9; 32]).unwrap();
        let c = verificar_nota(&n, k).unwrap().checkpoint;
        let linea = t.cofirmar(&c, 1_790_000_100).unwrap();
        match leer_respuesta(&n, k, &http(200, None, &linea)).unwrap() {
            Respuesta::Firmas(l) => {
                assert_eq!(l.len(), 1);
                assert_eq!(l[0].nombre, "testigo.invalid/t");
            }
            otra => panic!("{otra:?}"),
        }
        let del_publicador = &n[n.rfind("\n\n").unwrap() + 2..];
        for malo in [
            String::new(),
            linea.trim_end().to_string(),
            format!("x{linea}"),
            format!("{linea}\n{linea}"),
            format!("{linea}{del_publicador}"),
            linea.replace('\n', "\r\n"),
        ] {
            assert!(
                leer_respuesta(&n, k, &http(200, None, &malo)).is_err(),
                "{malo:?}"
            );
        }
    }

    #[test]
    fn solo_bucle_local_y_sin_tls() {
        let l = local("http://127.0.0.1:7380").unwrap();
        assert_eq!(l.direccion, "127.0.0.1:7380".parse().unwrap());
        assert_eq!(
            (l.anfitrion.as_str(), l.ruta.as_str()),
            ("127.0.0.1:7380", "")
        );
        let l = local("http://localhost:8080/testigo").unwrap();
        assert_eq!(l.direccion, "127.0.0.1:8080".parse().unwrap());
        assert_eq!(l.ruta, "/testigo");
        assert!(local("http://[::1]:7380").is_ok());
        assert!(local("http://127.8.9.10:1").is_ok());
        for malo in [
            "https://127.0.0.1:443",
            "http://10.0.0.1:7380",
            "http://192.168.1.1:7380",
            "http://[2001:db8::1]:7380",
            "http://testigo.example:7380",
            "http://127.0.0.1",
            "http://127.0.0.1:99999",
            "ftp://127.0.0.1:21",
            "http://127.0.0.1:7380/con espacio",
        ] {
            assert!(local(malo).is_err(), "{malo}");
        }
    }

    #[test]
    fn la_respuesta_http_por_largo_por_trozos_y_hasta_el_final() {
        let r = leer_http(
            b"HTTP/1.1 409 Conflict\r\nContent-Type: text/x.tlog.size\r\nContent-Length: 2\r\n\r\n5\n",
        )
        .unwrap();
        assert_eq!(
            (r.estado, r.tipo.as_deref(), r.cuerpo.as_slice()),
            (409, Some("text/x.tlog.size"), &b"5\n"[..])
        );
        let r = leer_http(
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n4\r\nhola\r\n3;x=y\r\n, m\r\n0\r\n\r\n",
        )
        .unwrap();
        assert_eq!(r.cuerpo, b"hola, m");
        let r = leer_http(b"HTTP/1.0 200 OK\r\n\r\nhasta el final").unwrap();
        assert_eq!(r.cuerpo, b"hasta el final");
        for malo in [
            &b"HTTP/1.1 200 OK\r\nContent-Length: 9\r\n\r\ncorto"[..],
            b"HTTP/1.1 200 OK\r\nContent-Length: 1\r\nContent-Length: 2\r\n\r\nab",
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n4\r\nhol",
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\nzz\r\n",
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n4\r\nholaXX0\r\n\r\n",
            b"HTTP/2 200\r\n\r\n",
            b"HTTP/1.1 20 OK\r\n\r\n",
            b"HTTP/1.1 200 OK\r\nsin dos puntos\r\n\r\n",
            b"HTTP/1.1 200 OK\r\n",
        ] {
            assert!(
                leer_http(malo).is_err(),
                "{:?}",
                String::from_utf8_lossy(malo)
            );
        }
    }

    /// Un testigo de mentira, en memoria, que contesta lo que se le diga y
    /// guarda lo que se le pidió.
    struct Guion {
        respuestas: Vec<RespuestaHttp>,
        peticiones: Vec<String>,
    }

    impl Transporte for Guion {
        fn add_checkpoint(
            &mut self,
            _: &Testigo,
            cuerpo: &[u8],
        ) -> Result<RespuestaHttp, ErrorDeTestigo> {
            self.peticiones
                .push(String::from_utf8(cuerpo.to_vec()).unwrap());
            Ok(self.respuestas.remove(0))
        }
    }

    fn guion(respuestas: Vec<RespuestaHttp>) -> Guion {
        Guion {
            respuestas,
            peticiones: Vec::new(),
        }
    }

    /// El reintento del `409` es UNO y desde el tamaño que el testigo dice;
    /// un tamaño mayor que el de la nota no se reintenta.
    #[test]
    fn el_409_se_reintenta_una_vez_y_el_retroceso_ninguna() {
        let p = publicador();
        let a = arbol(9);
        let n = nota(&p, &a);
        let k = p.clave_publica();
        let t = Cofirmante::determinista("testigo.invalid/t", [9; 32]).unwrap();
        let testigo = Testigo::nuevo(&t.clave_publica().vkey(), "http://127.0.0.1:1").unwrap();
        let c = verificar_nota(&n, k).unwrap().checkpoint;
        let linea = t.cofirmar(&c, 1_790_000_100).unwrap();
        let tamano = |m: u64| http(409, Some(TIPO_DEL_TAMANO), &format!("{m}\n"));

        let mut g = guion(vec![tamano(7), http(200, None, &linea)]);
        let r = pedir_cofirma(&mut g, &testigo, k, &n, &a, 0).unwrap();
        assert!(
            matches!(
                r,
                Resultado::Cofirmada {
                    viejo: 7,
                    ignoradas: 0,
                    ..
                }
            ),
            "{r:?}"
        );
        assert!(g.peticiones[0].starts_with("old 0\n\n"));
        let mut esperado = String::from("old 7\n");
        for h in a.prueba_de_consistencia(7, 9).unwrap() {
            esperado.push_str(&base64_encode(&h));
            esperado.push('\n');
        }
        assert!(g.peticiones[1].starts_with(&format!("{esperado}\n")));

        // Dos 409 seguidos: no hay un tercer intento.
        let mut g = guion(vec![tamano(7), tamano(3)]);
        let r = pedir_cofirma(&mut g, &testigo, k, &n, &a, 0).unwrap();
        assert_eq!(r, Resultado::Rechazada(Respuesta::Conflicto(3)));
        assert_eq!(g.peticiones.len(), 2);

        // El testigo recuerda MÁS anclas que la nota: la 103, sin reintento.
        let mut g = guion(vec![tamano(12)]);
        let r = pedir_cofirma(&mut g, &testigo, k, &n, &a, 0).unwrap();
        assert_eq!(
            r,
            Resultado::Retroceso {
                recordado: 12,
                nota: 9
            }
        );
        assert_eq!(g.peticiones.len(), 1);
        // Y el mismo número que se envió no es una pista: no se reintenta.
        let mut g = guion(vec![tamano(4)]);
        let r = pedir_cofirma(&mut g, &testigo, k, &n, &a, 4).unwrap();
        assert_eq!(r, Resultado::Rechazada(Respuesta::Conflicto(4)));
    }

    /// De las líneas de un `200` se retiene la del testigo, verificada; las
    /// de otras claves —también la de su mismo nombre con otra clave— se
    /// ignoran, y una suya que no verifica es un error.
    #[test]
    fn del_200_se_queda_la_suya_verificada() {
        let p = publicador();
        let a = arbol(5);
        let n = nota(&p, &a);
        let k = p.clave_publica();
        let c = verificar_nota(&n, k).unwrap().checkpoint;
        let t = Cofirmante::determinista("testigo.invalid/t", [9; 32]).unwrap();
        let otra_clave = Cofirmante::determinista("testigo.invalid/t", [8; 32]).unwrap();
        let ajeno = Cofirmante::determinista("testigo.invalid/otro", [6; 32]).unwrap();
        let testigo = Testigo::nuevo(&t.clave_publica().vkey(), "http://127.0.0.1:1").unwrap();
        let suya = t.cofirmar(&c, 1_790_000_100).unwrap();
        let cuerpo = format!(
            "{}{}{suya}",
            otra_clave.cofirmar(&c, 1).unwrap(),
            ajeno.cofirmar(&c, 2).unwrap()
        );
        let mut g = guion(vec![http(200, None, &cuerpo)]);
        match pedir_cofirma(&mut g, &testigo, k, &n, &a, 5).unwrap() {
            Resultado::Cofirmada {
                cofirma,
                viejo,
                ignoradas,
            } => {
                assert_eq!((viejo, ignoradas, cofirma.marca), (5, 2, 1_790_000_100));
                assert_eq!(cofirma.texto(), suya);
            }
            otro => panic!("{otro:?}"),
        }
        // Sin ninguna suya: el testigo avanzó y no hay nada que retener.
        let mut g = guion(vec![http(200, None, &ajeno.cofirmar(&c, 2).unwrap())]);
        assert_eq!(
            pedir_cofirma(&mut g, &testigo, k, &n, &a, 5).unwrap(),
            Resultado::SinSuCofirma {
                viejo: 5,
                ignoradas: 1
            }
        );
        // Su línea, sobre OTRO checkpoint: no verifica, y es un error.
        let otro = Checkpoint {
            tamano: 4,
            raiz: a.raiz_en(4).unwrap(),
            ..c.clone()
        };
        let mut g = guion(vec![http(200, None, &t.cofirmar(&otro, 3).unwrap())]);
        assert_eq!(
            pedir_cofirma(&mut g, &testigo, k, &n, &a, 5),
            Err(ErrorDeTestigo::CofirmaInvalida {
                testigo: "testigo.invalid/t".into()
            })
        );
    }

    /// Antes de enviar: la nota es del publicador, va sola, y el árbol tiene
    /// su raíz. Si no, no sale ninguna petición.
    #[test]
    fn no_se_envia_una_nota_ajena_acompanada_o_de_otro_arbol() {
        let p = publicador();
        let a = arbol(5);
        let n = nota(&p, &a);
        let k = p.clave_publica();
        let t = Cofirmante::determinista("testigo.invalid/t", [9; 32]).unwrap();
        let testigo = Testigo::nuevo(&t.clave_publica().vkey(), "http://127.0.0.1:1").unwrap();
        let c = verificar_nota(&n, k).unwrap().checkpoint;
        let acompanada = format!("{n}{}", t.cofirmar(&c, 1).unwrap());
        let impostor = Publicador::determinista(k.nombre(), [3; 32]).unwrap();
        let mut otro = ArbolDelMedio::nuevo();
        for i in 0..5u64 {
            otro.anadir(&sha256(&(i + 100).to_be_bytes()));
        }
        let mut g = guion(vec![]);
        assert!(matches!(
            pedir_cofirma(&mut g, &testigo, k, &acompanada, &a, 0),
            Err(ErrorDeTestigo::Peticion(_))
        ));
        assert!(matches!(
            pedir_cofirma(&mut g, &testigo, impostor.clave_publica(), &n, &a, 0),
            Err(ErrorDeTestigo::Nota(_))
        ));
        assert_eq!(
            pedir_cofirma(&mut g, &testigo, k, &n, &otro, 0),
            Err(ErrorDeTestigo::OtroArbol)
        );
        assert!(matches!(
            pedir_cofirma(&mut g, &testigo, k, &n, &arbol(4), 0),
            Err(ErrorDeTestigo::Arbol(_))
        ));
        assert!(g.peticiones.is_empty());
    }

    /// La retención: una por testigo, verificada sobre SU checkpoint, y la
    /// nota cofirmada se relee igual.
    #[test]
    fn la_retencion_una_por_testigo_y_la_nota_cofirmada_se_relee() {
        let p = publicador();
        let a = arbol(5);
        let n = nota(&p, &a);
        let k = p.clave_publica();
        let c = verificar_nota(&n, k).unwrap().checkpoint;
        let uno = Cofirmante::determinista("testigo.invalid/uno", [9; 32]).unwrap();
        let dos = Cofirmante::determinista("testigo.invalid/dos", [8; 32]).unwrap();
        // La línea de una cofirma, leída como la lee el verificador de notas.
        let linea_de =
            |texto: &str| verificar_nota(&format!("{n}{texto}"), k).unwrap().ajenas[0].clone();
        let cofirma = |t: &Cofirmante, c: &Checkpoint, m: u64| Cofirma {
            linea: linea_de(&t.cofirmar(c, m).unwrap()),
            marca: m,
        };
        let mut r = Retencion::nueva(&n, k).unwrap();
        assert!(r
            .retener(uno.clave_publica(), &cofirma(&uno, &c, 11))
            .unwrap());
        assert!(!r
            .retener(uno.clave_publica(), &cofirma(&uno, &c, 12))
            .unwrap());
        assert!(r
            .retener(dos.clave_publica(), &cofirma(&dos, &c, 13))
            .unwrap());
        assert_eq!(r.cofirmas().len(), 2);
        assert_eq!(r.cofirmas()[0].marca, 11);
        // La cofirma de otro checkpoint no se retiene, ni con la clave de
        // otro testigo, ni con la marca cambiada.
        let otro = Checkpoint {
            tamano: 4,
            raiz: a.raiz_en(4).unwrap(),
            ..c.clone()
        };
        // La línea de otro checkpoint se lee igual —el texto de la nota no
        // entra en la línea—, y no verifica sobre este.
        let de_otro = Cofirma {
            linea: linea_de(&uno.cofirmar(&otro, 14).unwrap()),
            marca: 14,
        };
        let mut s = Retencion::nueva(&n, k).unwrap();
        assert!(s.retener(uno.clave_publica(), &de_otro).is_err());
        assert!(s
            .retener(dos.clave_publica(), &cofirma(&uno, &c, 15))
            .is_err());
        let mut marca_cambiada = cofirma(&uno, &c, 16);
        marca_cambiada.marca = 17;
        assert!(s.retener(uno.clave_publica(), &marca_cambiada).is_err());
        assert!(s.cofirmas().is_empty());

        // La nota cofirmada la lee el verificador de notas, y cada línea
        // verifica con su testigo; y se relee a la misma retención.
        let nc = r.nota_cofirmada();
        let v = verificar_nota(&nc, k).unwrap();
        assert_eq!(v.ajenas.len(), 2);
        assert_eq!(
            uno.clave_publica().verificar_cofirma(&c, &v.ajenas[0]),
            Ok(11)
        );
        assert_eq!(
            dos.clave_publica().verificar_cofirma(&c, &v.ajenas[1]),
            Ok(13)
        );
        let testigos = [uno.clave_publica().clone(), dos.clave_publica().clone()];
        assert_eq!(Retencion::leer(&nc, k, &testigos).unwrap(), r);
        // Releída con un solo testigo, la línea del otro no es de nadie.
        assert!(Retencion::leer(&nc, k, &testigos[..1]).is_err());
        // Con una línea repetida, tampoco.
        let repetida = format!("{nc}{}", r.cofirmas()[0].texto());
        assert!(Retencion::leer(&repetida, k, &testigos).is_err());
        // Ni con una nota que ya lleva otras firmas como punto de partida.
        assert!(Retencion::nueva(&nc, k).is_err());
    }

    /// Lo publicado no retrocede, no se bifurca, y no se pisa si no es del
    /// publicador.
    #[test]
    fn publicar_no_retrocede_ni_se_bifurca() {
        /// El directorio temporal se borra también si el test cae.
        struct Temporal(std::path::PathBuf);
        impl Drop for Temporal {
            fn drop(&mut self) {
                let _ = fs::remove_dir_all(&self.0);
            }
        }
        let t = Temporal(std::env::temp_dir().join(format!(
            "zk-ssl-medio-publicar-{}-{}",
            std::process::id(),
            line!()
        )));
        let dir = t.0.clone();
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let p = publicador();
        let k = p.clave_publica();
        let r = |n: u64| Retencion::nueva(&nota(&p, &arbol(n)), k).unwrap();
        publicar(&dir, &r(5), k).unwrap();
        assert_eq!(
            fs::read_to_string(dir.join(FICHERO)).unwrap(),
            r(5).nota_cofirmada()
        );
        publicar(&dir, &r(5), k).unwrap();
        publicar(&dir, &r(9), k).unwrap();
        assert_eq!(
            publicar(&dir, &r(5), k),
            Err(ErrorDeTestigo::PublicadoMayor {
                publicado: 9,
                nuevo: 5
            })
        );
        let mut b = ArbolDelMedio::nuevo();
        for i in 0..9u64 {
            b.anadir(&sha256(&(i + 100).to_be_bytes()));
        }
        let bifurcada = Retencion::nueva(&nota(&p, &b), k).unwrap();
        assert_eq!(
            publicar(&dir, &bifurcada, k),
            Err(ErrorDeTestigo::PublicadoDistinto { tamano: 9 })
        );
        // Un checkpoint que no es del publicador no se pisa.
        let ajeno = Publicador::determinista(k.nombre(), [3; 32]).unwrap();
        fs::write(dir.join(FICHERO), nota(&ajeno, &arbol(2))).unwrap();
        assert!(matches!(
            publicar(&dir, &r(9), k),
            Err(ErrorDeTestigo::Fichero(_))
        ));
        assert!(!dir.join(format!("{FICHERO}.nuevo")).exists());
    }
}
