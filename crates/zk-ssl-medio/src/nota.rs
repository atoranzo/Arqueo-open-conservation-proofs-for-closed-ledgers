//! # La nota del medio (RFC-0013 D-A y D-C; E2b, §632)
//!
//! El medio publica su árbol de anclas como una nota `checkpoint` de C2SP:
//! tres líneas —`origin`, `size` y la raíz en base64—, una línea en blanco
//! y las líneas de firma. La primera es la del publicador, firmada con
//! ML-DSA-44 en el tipo `0x06` de `tlog-cosignature`; detrás pueden ir las
//! cofirmas de los testigos (D-D), que aquí no se juzgan: se devuelven para
//! la política de umbral de E4.
//!
//! ```text
//! zkssl/v1/<huella de la clave XMSS, hex>
//! <size>
//! <raíz SHA-256 del árbol de anclas, base64>
//!
//! — zkssl/v1/<huella> base64(key_id || timestamp || firma ML-DSA-44)
//! — <testigo> base64(…)
//! ```
//!
//! Lo que se firma no es el texto de la nota sino el `cosigned_message` de
//! `tlog-cosignature`: la etiqueta `subtree/v1\n\0`, el nombre de quien
//! firma, la marca de tiempo, el `origin`, el subárbol `[0, size)` y la
//! raíz. El `key_id` es `SHA-256(nombre || "\n" || 0x06 || clave)[:4]`.
//!
//! ## Las reglas, y de dónde sale cada una
//!
//! Las de `signed-note` y `tlog-checkpoint`, como las aplican
//! `golang.org/x/mod/sumdb/note` (`note.Open`) y `filippo.io/torchwood`
//! (`ParseCheckpoint`, `NewLogVerifier`), con los que se contrastan los
//! vectores (`tests/vectores_notas.rs`): UTF-8 sin caracteres de control
//! salvo el salto de línea; el texto acaba en la ÚLTIMA línea en blanco;
//! cada línea de firma es `— nombre base64`, con un nombre sin blancos ni
//! `+` y una firma de al menos 5 bytes; como mucho 100 líneas; las de
//! claves que no son la del publicador se ignoran; y el texto es un
//! checkpoint canónico —tamaño decimal sin ceros a la izquierda y como
//! mucho `2^63-1`, raíz de 32 bytes en base64 estricto— **sin líneas de
//! extensión**, que el tipo `0x06` no cubre y torchwood rechaza.
//!
//! ⚠️ **Dos reglas más estrictas que las de Go, y declaradas:**
//!
//! - **El `origin` es el nombre del publicador** (D-A: la nota del medio
//!   la firma la clave que se llama como el medio). torchwood firma y
//!   verifica con `origin` y nombre distintos; aquí una nota así es
//!   [`ErrorDeNota::OrigenAjeno`].
//! - **Una sola línea del publicador.** `note.Open` verifica la primera
//!   línea de una clave y descarta las repetidas sin mirarlas; aquí dos
//!   líneas del publicador son [`ErrorDeNota::FirmaRepetida`]: no hay
//!   que elegir de cuál se lee la marca de tiempo.
//!
//! ## ⚠️ Lo que este módulo NO hace
//!
//! - **No juzga a los testigos.** Sus líneas se devuelven en
//!   [`NotaVerificada::ajenas`], sin verificar. [`ClaveDeNota::verificar_cofirma`]
//!   comprueba una si se le da la clave de su testigo; qué testigos valen y
//!   cuántos hacen falta es la política de umbral de E4.
//! - **No custodia la clave.** [`Publicador`] recibe la semilla y borra su
//!   copia; de dónde sale y dónde vive es del publicador (E3). La clave no
//!   tiene índice ni guardián: es lo que D-C eligió.
//! - **No decide la frescura.** La marca de tiempo se devuelve; si una
//!   marca del futuro se rechaza lo decide quien verifica (E4).

use getrandom::SysRng;
use ml_dsa::signature::Keypair;
use ml_dsa::{EncodedVerifyingKey, MlDsa44, Seed, Signature, SigningKey, VerifyingKey};
use zeroize::Zeroize;

use crate::base64::{base64_decode, base64_encode};
use crate::hash::{hash_empty, sha256, HashValue, HASH_SIZE};
use crate::subtree::is_valid_subtree;

/// El byte de tipo de `tlog-cosignature` para ML-DSA-44.
pub const TIPO_MLDSA44: u8 = 0x06;
/// Bytes de una clave pública ML-DSA-44 (`pkEncode`).
pub const BYTES_CLAVE_PUBLICA: usize = 1312;
/// Bytes de una firma ML-DSA-44.
pub const BYTES_FIRMA: usize = 2420;
/// `uint8 label[12] = "subtree/v1\n\0"`.
pub const ETIQUETA: &[u8; 12] = b"subtree/v1\n\0";
/// Prefijo del `origin` del medio (D-A).
pub const PREFIJO_ORIGEN: &str = "zkssl/v1/";
/// Líneas de firma que se leen como mucho, como `note.Open`.
pub const MAX_LINEAS_DE_FIRMA: usize = 100;
/// `— `: raya (U+2014) y espacio.
const PREFIJO_FIRMA: &str = "\u{2014} ";

/// El `origin` del medio de un operador: `zkssl/v1/` y la huella de su clave
/// XMSS (RFC-0012 D-B) en hexadecimal. La identidad que un testigo recuerda
/// es la misma que un titular verifica en cada cabeza.
pub fn origen_del_medio(huella_de_clave: &[u8; 32]) -> String {
    let mut s = String::from(PREFIJO_ORIGEN);
    for b in huella_de_clave {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// Lo que puede fallar al escribir, firmar o verificar una nota.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorDeNota {
    /// La nota no es una `signed-note`: caracteres de control, sin línea en
    /// blanco, una línea de firma mal formada o demasiadas.
    Malformada(&'static str),
    /// El texto no es un checkpoint canónico de tres líneas.
    Checkpoint(&'static str),
    /// El `origin` no es el nombre del publicador (D-A).
    OrigenAjeno { esperado: String, visto: String },
    /// Ninguna línea de la clave del publicador.
    SinFirmaDelPublicador,
    /// Más de una línea de la clave del publicador.
    FirmaRepetida,
    /// La línea del publicador está, y no verifica.
    FirmaInvalida,
    /// El `cosigned_message` no se puede formar con esos valores.
    Mensaje(&'static str),
    /// Una clave o una `vkey` que no lo es.
    Clave(&'static str),
    /// La firma no se pudo hacer (sin entropía, o no verifica al salir).
    Firmando(String),
}

impl core::fmt::Display for ErrorDeNota {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ErrorDeNota::Malformada(s) => write!(f, "nota mal formada: {s}"),
            ErrorDeNota::Checkpoint(s) => write!(f, "checkpoint mal formado: {s}"),
            ErrorDeNota::OrigenAjeno { esperado, visto } => {
                write!(f, "el origin es {visto:?} y el publicador {esperado:?}")
            }
            ErrorDeNota::SinFirmaDelPublicador => write!(f, "ninguna firma del publicador"),
            ErrorDeNota::FirmaRepetida => write!(f, "más de una firma del publicador"),
            ErrorDeNota::FirmaInvalida => write!(f, "la firma del publicador no verifica"),
            ErrorDeNota::Mensaje(s) => write!(f, "mensaje cofirmado: {s}"),
            ErrorDeNota::Clave(s) => write!(f, "clave: {s}"),
            ErrorDeNota::Firmando(s) => write!(f, "firmando: {s}"),
        }
    }
}

impl std::error::Error for ErrorDeNota {}

/// Un nombre de clave de `signed-note`: no vacío, sin blancos Unicode y
/// sin `+`.
fn nombre_valido(nombre: &str) -> bool {
    !nombre.is_empty() && !nombre.chars().any(|c| c.is_whitespace() || c == '+')
}

/// Lo que dice una nota: sus tres líneas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Checkpoint {
    pub origen: String,
    pub tamano: u64,
    pub raiz: HashValue,
}

impl Checkpoint {
    fn comprobar(&self) -> Result<(), ErrorDeNota> {
        if self.origen.is_empty() || self.origen.len() > 255 {
            return Err(ErrorDeNota::Checkpoint("el origin mide de 1 a 255 bytes"));
        }
        if self.origen.contains('\n') || self.origen.chars().any(|c| c.is_control()) {
            return Err(ErrorDeNota::Checkpoint("el origin no es una línea"));
        }
        if self.tamano > i64::MAX as u64 {
            return Err(ErrorDeNota::Checkpoint("el tamaño pasa de 2^63-1"));
        }
        Ok(())
    }

    /// El texto de la nota: las tres líneas, cada una con su salto.
    pub fn texto(&self) -> Result<String, ErrorDeNota> {
        self.comprobar()?;
        Ok(format!(
            "{}\n{}\n{}\n",
            self.origen,
            self.tamano,
            base64_encode(&self.raiz)
        ))
    }

    /// Lee el texto de una nota, y solo si es la escritura canónica de un
    /// checkpoint de tres líneas.
    pub fn leer(texto: &str) -> Result<Checkpoint, ErrorDeNota> {
        let cuerpo = texto.strip_suffix('\n').ok_or(ErrorDeNota::Checkpoint(
            "el texto no acaba en salto de línea",
        ))?;
        let lineas: Vec<&str> = cuerpo.split('\n').collect();
        if lineas.len() < 3 {
            return Err(ErrorDeNota::Checkpoint("menos de tres líneas"));
        }
        if lineas.len() > 3 {
            return Err(ErrorDeNota::Checkpoint(
                "líneas de extensión: el tipo 0x06 no las cubre",
            ));
        }
        let tamano: u64 = lineas[1]
            .parse()
            .map_err(|_| ErrorDeNota::Checkpoint("el tamaño no es un número"))?;
        if lineas[1] != tamano.to_string() {
            return Err(ErrorDeNota::Checkpoint(
                "el tamaño no está en su forma canónica",
            ));
        }
        let raiz: HashValue = base64_decode(lineas[2])
            .map_err(|_| ErrorDeNota::Checkpoint("la raíz no es base64 estricto"))?
            .try_into()
            .map_err(|_| ErrorDeNota::Checkpoint("la raíz no mide 32 bytes"))?;
        let c = Checkpoint {
            origen: lineas[0].to_string(),
            tamano,
            raiz,
        };
        c.comprobar()?;
        Ok(c)
    }
}

/// El `cosigned_message` de `tlog-cosignature` para ML-DSA-44: **una sola
/// definición** para quien firma y quien verifica.
pub fn mensaje_cofirmado(
    nombre: &str,
    marca: u64,
    origen: &str,
    inicio: u64,
    fin: u64,
    hash: &HashValue,
) -> Result<Vec<u8>, ErrorDeNota> {
    if nombre.is_empty() || nombre.len() > 255 {
        return Err(ErrorDeNota::Mensaje("el nombre mide de 1 a 255 bytes"));
    }
    if origen.is_empty() || origen.len() > 255 {
        return Err(ErrorDeNota::Mensaje("el origin mide de 1 a 255 bytes"));
    }
    if marca > i64::MAX as u64 {
        return Err(ErrorDeNota::Mensaje("la marca de tiempo pasa de 2^63-1"));
    }
    if marca != 0 && inicio != 0 {
        return Err(ErrorDeNota::Mensaje(
            "una marca de tiempo solo en un subárbol que empieza en 0",
        ));
    }
    if !is_valid_subtree(inicio, fin) {
        return Err(ErrorDeNota::Mensaje("el intervalo no es un subárbol"));
    }
    if inicio == fin && *hash != hash_empty() {
        return Err(ErrorDeNota::Mensaje(
            "el hash de un subárbol vacío es SHA-256 de nada",
        ));
    }
    let mut m = Vec::with_capacity(12 + 1 + nombre.len() + 8 + 1 + origen.len() + 16 + HASH_SIZE);
    m.extend_from_slice(ETIQUETA);
    m.push(nombre.len() as u8);
    m.extend_from_slice(nombre.as_bytes());
    m.extend_from_slice(&marca.to_be_bytes());
    m.push(origen.len() as u8);
    m.extend_from_slice(origen.as_bytes());
    m.extend_from_slice(&inicio.to_be_bytes());
    m.extend_from_slice(&fin.to_be_bytes());
    m.extend_from_slice(hash);
    Ok(m)
}

/// El `key_id` de una clave ML-DSA-44:
/// `SHA-256(nombre || "\n" || 0x06 || clave)[:4]`.
pub fn id_de_clave(nombre: &str, clave_publica: &[u8]) -> [u8; 4] {
    let mut entrada = Vec::with_capacity(nombre.len() + 2 + clave_publica.len());
    entrada.extend_from_slice(nombre.as_bytes());
    entrada.push(b'\n');
    entrada.push(TIPO_MLDSA44);
    entrada.extend_from_slice(clave_publica);
    let h = sha256(&entrada);
    [h[0], h[1], h[2], h[3]]
}

/// Una clave pública ML-DSA-44 con su nombre: la del publicador, que un
/// testigo o un tercero tiene configurada, o la de un testigo.
#[derive(Clone)]
pub struct ClaveDeNota {
    nombre: String,
    bytes: Vec<u8>,
    clave: VerifyingKey<MlDsa44>,
    id: [u8; 4],
}

impl core::fmt::Debug for ClaveDeNota {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ClaveDeNota({})", self.vkey())
    }
}

impl ClaveDeNota {
    pub fn nueva(nombre: &str, clave_publica: &[u8]) -> Result<Self, ErrorDeNota> {
        if !nombre_valido(nombre) {
            return Err(ErrorDeNota::Clave("nombre vacío, con blancos o con +"));
        }
        let enc = EncodedVerifyingKey::<MlDsa44>::try_from(clave_publica)
            .map_err(|_| ErrorDeNota::Clave("la clave ML-DSA-44 mide 1312 bytes"))?;
        Ok(ClaveDeNota {
            nombre: nombre.to_string(),
            bytes: clave_publica.to_vec(),
            clave: VerifyingKey::<MlDsa44>::decode(&enc),
            id: id_de_clave(nombre, clave_publica),
        })
    }

    /// Lee una `vkey` de `signed-note`: `nombre+hex(key_id)+base64(0x06 ||
    /// clave)`, y exige que el `key_id` sea el que sale de las otras dos.
    pub fn leer_vkey(vkey: &str) -> Result<Self, ErrorDeNota> {
        // El nombre no lleva `+`, y la base64 sí puede: se parte en los
        // dos primeros, como `strings.Cut` en Go.
        let mut partes = vkey.splitn(3, '+');
        let (Some(nombre), Some(id_hex), Some(b64)) = (partes.next(), partes.next(), partes.next())
        else {
            return Err(ErrorDeNota::Clave("una vkey tiene tres partes"));
        };
        if id_hex.len() != 8 || !id_hex.bytes().all(|c| c.is_ascii_hexdigit()) {
            return Err(ErrorDeNota::Clave("el key_id son 8 cifras hexadecimales"));
        }
        let id = u32::from_str_radix(id_hex, 16)
            .map_err(|_| ErrorDeNota::Clave("el key_id no es hexadecimal"))?
            .to_be_bytes();
        let material =
            base64_decode(b64).map_err(|_| ErrorDeNota::Clave("la clave no es base64"))?;
        let (tipo, clave) = material
            .split_first()
            .ok_or(ErrorDeNota::Clave("la vkey no lleva clave"))?;
        if *tipo != TIPO_MLDSA44 {
            return Err(ErrorDeNota::Clave("la vkey no es de tipo 0x06"));
        }
        let c = ClaveDeNota::nueva(nombre, clave)?;
        if c.id != id {
            return Err(ErrorDeNota::Clave("el key_id no es el de esa clave"));
        }
        Ok(c)
    }

    /// La `vkey` de `signed-note`.
    pub fn vkey(&self) -> String {
        let mut material = vec![TIPO_MLDSA44];
        material.extend_from_slice(&self.bytes);
        format!(
            "{}+{:08x}+{}",
            self.nombre,
            u32::from_be_bytes(self.id),
            base64_encode(&material)
        )
    }

    pub fn nombre(&self) -> &str {
        &self.nombre
    }

    pub fn id(&self) -> [u8; 4] {
        self.id
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// ML-DSA.Verify de FIPS 204, con el contexto vacío.
    fn verifica(&self, mensaje: &[u8], firma: &[u8]) -> bool {
        match Signature::<MlDsa44>::try_from(firma) {
            Ok(sig) => self.clave.verify_with_context(mensaje, &[], &sig),
            Err(_) => false,
        }
    }

    /// `timestamp || firma` (lo que va detrás del `key_id`) sobre el
    /// checkpoint, con el nombre de esta clave: devuelve la marca si
    /// verifica. Es la comprobación de la línea del publicador, y la que
    /// E4 hará con cada testigo de su política.
    fn verifica_linea(&self, checkpoint: &Checkpoint, resto: &[u8]) -> Result<u64, ErrorDeNota> {
        if resto.len() != 8 + BYTES_FIRMA {
            return Err(ErrorDeNota::FirmaInvalida);
        }
        let marca = u64::from_be_bytes(resto[..8].try_into().expect("8 bytes"));
        let mensaje = mensaje_cofirmado(
            &self.nombre,
            marca,
            &checkpoint.origen,
            0,
            checkpoint.tamano,
            &checkpoint.raiz,
        )
        .map_err(|_| ErrorDeNota::FirmaInvalida)?;
        if !self.verifica(&mensaje, &resto[8..]) {
            return Err(ErrorDeNota::FirmaInvalida);
        }
        Ok(marca)
    }

    /// **Verifica una cofirma** de esta clave —la línea de un testigo que
    /// [`verificar_nota`] devolvió sin mirar— sobre el checkpoint de la
    /// nota. Devuelve su marca de tiempo. Una línea de otro nombre o de otro
    /// `key_id` no es de esta clave: [`ErrorDeNota::Clave`].
    pub fn verificar_cofirma(
        &self,
        checkpoint: &Checkpoint,
        linea: &FirmaAjena,
    ) -> Result<u64, ErrorDeNota> {
        if linea.nombre != self.nombre || linea.id != self.id {
            return Err(ErrorDeNota::Clave("la línea no es de esta clave"));
        }
        self.verifica_linea(checkpoint, &linea.firma)
    }
}

/// Cómo firma un [`Publicador`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Modo {
    /// La variante que FIPS 204 recomienda: 32 bytes del sistema por firma.
    ConSal,
    /// La variante opcional: misma entrada, misma firma. Solo para
    /// reproducir vectores; FIPS 204 avisa de que resiste peor los ataques
    /// por fallos.
    Determinista,
}

/// Quien firma las notas del medio.
pub struct Publicador {
    clave: SigningKey<MlDsa44>,
    publica: ClaveDeNota,
    modo: Modo,
}

impl Publicador {
    /// ⚠️ La semilla es material de clave: de dónde sale y dónde vive es del
    /// publicador (E3). La copia de aquí se borra; la del llamador es suya.
    pub fn desde_semilla(nombre: &str, semilla: [u8; 32]) -> Result<Self, ErrorDeNota> {
        Self::con_modo(nombre, semilla, Modo::ConSal)
    }

    /// Firma determinista: solo para vectores (ver [`Modo::Determinista`]).
    pub fn determinista(nombre: &str, semilla: [u8; 32]) -> Result<Self, ErrorDeNota> {
        Self::con_modo(nombre, semilla, Modo::Determinista)
    }

    fn con_modo(nombre: &str, mut semilla: [u8; 32], modo: Modo) -> Result<Self, ErrorDeNota> {
        let clave = SigningKey::<MlDsa44>::from_seed(&Seed::from(semilla));
        semilla.zeroize();
        let bytes = clave.verifying_key().encode().to_vec();
        let publica = ClaveDeNota::nueva(nombre, &bytes)?;
        Ok(Publicador {
            clave,
            publica,
            modo,
        })
    }

    pub fn clave_publica(&self) -> &ClaveDeNota {
        &self.publica
    }

    pub fn modo(&self) -> Modo {
        self.modo
    }

    /// Firma el checkpoint con la marca de tiempo `marca` y devuelve la nota
    /// entera: el texto, la línea en blanco y la línea del publicador.
    ///
    /// ⚠️ Antes de devolverla la verifica con el mismo verificador que usará
    /// un tercero (la regla del §299).
    pub fn firmar(&self, checkpoint: &Checkpoint, marca: u64) -> Result<String, ErrorDeNota> {
        if checkpoint.origen != self.publica.nombre {
            return Err(ErrorDeNota::OrigenAjeno {
                esperado: self.publica.nombre.clone(),
                visto: checkpoint.origen.clone(),
            });
        }
        let texto = checkpoint.texto()?;
        let mensaje = mensaje_cofirmado(
            &self.publica.nombre,
            marca,
            &checkpoint.origen,
            0,
            checkpoint.tamano,
            &checkpoint.raiz,
        )?;
        let expandida = self.clave.expanded_key();
        let firma = match self.modo {
            Modo::Determinista => expandida.sign_deterministic(&mensaje, &[]),
            Modo::ConSal => expandida.sign_randomized(&mensaje, &[], &mut SysRng),
        }
        .map_err(|e| ErrorDeNota::Firmando(e.to_string()))?;
        let mut bytes = self.publica.id.to_vec();
        bytes.extend_from_slice(&marca.to_be_bytes());
        bytes.extend_from_slice(&firma.encode());
        let nota = format!(
            "{texto}\n{PREFIJO_FIRMA}{} {}\n",
            self.publica.nombre,
            base64_encode(&bytes)
        );
        match verificar_nota(&nota, &self.publica) {
            Ok(v) if v.checkpoint == *checkpoint && v.marca == marca => Ok(nota),
            Ok(_) => Err(ErrorDeNota::Firmando(
                "la nota recién hecha no dice lo que se firmó".into(),
            )),
            Err(e) => Err(ErrorDeNota::Firmando(format!(
                "la nota recién hecha no verifica: {e}"
            ))),
        }
    }
}

/// Una línea de firma de una clave que no es la del publicador: la de un
/// testigo, o la de cualquiera. **No se ha verificado.**
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FirmaAjena {
    pub nombre: String,
    pub id: [u8; 4],
    /// Lo que va detrás del `key_id`.
    pub firma: Vec<u8>,
}

/// Una nota cuya línea del publicador verifica.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotaVerificada {
    pub checkpoint: Checkpoint,
    /// La marca de tiempo de la línea del publicador.
    pub marca: u64,
    /// Las demás líneas, en su orden y sin verificar (E4).
    pub ajenas: Vec<FirmaAjena>,
}

/// **Verifica una nota del medio** contra la clave del publicador.
pub fn verificar_nota(nota: &str, publicador: &ClaveDeNota) -> Result<NotaVerificada, ErrorDeNota> {
    if nota.chars().any(|c| c < ' ' && c != '\n') {
        return Err(ErrorDeNota::Malformada("caracteres de control"));
    }
    let corte = nota
        .rfind("\n\n")
        .ok_or(ErrorDeNota::Malformada("sin línea en blanco"))?;
    let (texto, firmas) = (&nota[..corte + 1], &nota[corte + 2..]);
    let firmas = firmas.strip_suffix('\n').ok_or(ErrorDeNota::Malformada(
        "las firmas no acaban en salto de línea",
    ))?;

    let mut del_publicador = Vec::new();
    let mut ajenas = Vec::new();
    for (n, linea) in firmas.split('\n').enumerate() {
        if n >= MAX_LINEAS_DE_FIRMA {
            return Err(ErrorDeNota::Malformada("más de 100 líneas de firma"));
        }
        let resto = linea
            .strip_prefix(PREFIJO_FIRMA)
            .ok_or(ErrorDeNota::Malformada(
                "una línea de firma no empieza por «— »",
            ))?;
        let (nombre, b64) = resto.split_once(' ').unwrap_or((resto, ""));
        if !nombre_valido(nombre) {
            return Err(ErrorDeNota::Malformada("nombre de clave inválido"));
        }
        let bytes = base64_decode(b64)
            .map_err(|_| ErrorDeNota::Malformada("una firma no es base64 estricto"))?;
        if bytes.len() < 5 {
            return Err(ErrorDeNota::Malformada("una firma de menos de 5 bytes"));
        }
        let id = [bytes[0], bytes[1], bytes[2], bytes[3]];
        if nombre == publicador.nombre && id == publicador.id {
            del_publicador.push(bytes);
        } else {
            ajenas.push(FirmaAjena {
                nombre: nombre.to_string(),
                id,
                firma: bytes[4..].to_vec(),
            });
        }
    }

    let checkpoint = Checkpoint::leer(texto)?;
    if checkpoint.origen != publicador.nombre {
        return Err(ErrorDeNota::OrigenAjeno {
            esperado: publicador.nombre.clone(),
            visto: checkpoint.origen,
        });
    }
    let bytes = match del_publicador.as_slice() {
        [] => return Err(ErrorDeNota::SinFirmaDelPublicador),
        [una] => una,
        _ => return Err(ErrorDeNota::FirmaRepetida),
    };
    let marca = publicador.verifica_linea(&checkpoint, &bytes[4..])?;
    Ok(NotaVerificada {
        checkpoint,
        marca,
        ajenas,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const HUELLA: [u8; 32] = [0x5a; 32];

    fn publicador(modo: Modo) -> Publicador {
        let nombre = origen_del_medio(&HUELLA);
        match modo {
            Modo::ConSal => Publicador::desde_semilla(&nombre, [7; 32]).unwrap(),
            Modo::Determinista => Publicador::determinista(&nombre, [7; 32]).unwrap(),
        }
    }

    fn checkpoint(tamano: u64) -> Checkpoint {
        Checkpoint {
            origen: origen_del_medio(&HUELLA),
            tamano,
            raiz: [tamano as u8; 32],
        }
    }

    #[test]
    fn el_origen_es_zkssl_v1_y_la_huella_en_hex() {
        let o = origen_del_medio(&[0xab; 32]);
        assert_eq!(o, format!("zkssl/v1/{}", "ab".repeat(32)));
        assert_eq!(o.len(), 9 + 64);
        assert!(nombre_valido(&o));
    }

    #[test]
    fn el_mensaje_cofirmado_byte_a_byte() {
        let m = mensaje_cofirmado("w", 0x0102, "o", 0, 5, &[0xee; 32]).unwrap();
        let mut esperado = b"subtree/v1\n\0".to_vec();
        esperado.extend_from_slice(&[1, b'w']);
        esperado.extend_from_slice(&[0, 0, 0, 0, 0, 0, 1, 2]);
        esperado.extend_from_slice(&[1, b'o']);
        esperado.extend_from_slice(&0u64.to_be_bytes());
        esperado.extend_from_slice(&5u64.to_be_bytes());
        esperado.extend_from_slice(&[0xee; 32]);
        assert_eq!(m, esperado);
        // Las reglas de tlog-cosignature, una a una.
        assert!(mensaje_cofirmado("", 0, "o", 0, 5, &[0; 32]).is_err());
        assert!(mensaje_cofirmado(&"n".repeat(256), 0, "o", 0, 5, &[0; 32]).is_err());
        assert!(mensaje_cofirmado("w", 1 << 63, "o", 0, 5, &[0; 32]).is_err());
        assert!(mensaje_cofirmado("w", 1, "o", 4, 8, &[0; 32]).is_err());
        assert!(mensaje_cofirmado("w", 0, "o", 4, 8, &[0; 32]).is_ok());
        assert!(mensaje_cofirmado("w", 0, "o", 1, 5, &[0; 32]).is_err());
        assert!(mensaje_cofirmado("w", 0, "o", 0, 0, &[0; 32]).is_err());
        assert!(mensaje_cofirmado("w", 0, "o", 0, 0, &hash_empty()).is_ok());
    }

    #[test]
    fn el_checkpoint_solo_en_su_forma_canonica() {
        let c = checkpoint(5);
        let t = c.texto().unwrap();
        assert_eq!(Checkpoint::leer(&t).unwrap(), c);
        let raiz = base64_encode(&c.raiz);
        for (malo, por) in [
            (format!("{}\n05\n{raiz}\n", c.origen), "cero a la izquierda"),
            (format!("{}\n+5\n{raiz}\n", c.origen), "signo"),
            (format!("{}\n5\n{raiz}\nextra\n", c.origen), "extensión"),
            (format!("{}\n5\n{raiz}", c.origen), "sin salto final"),
            (format!("\n5\n{raiz}\n"), "origin vacío"),
            (
                format!("{}\n9223372036854775808\n{raiz}\n", c.origen),
                "2^63",
            ),
            (format!("{}\n5\n{}\n", c.origen, &raiz[..40]), "raíz corta"),
            (format!("{}\n5\n {raiz}\n", c.origen), "blanco en la raíz"),
        ] {
            assert!(Checkpoint::leer(&malo).is_err(), "{por}");
        }
    }

    #[test]
    fn la_vkey_va_y_vuelve_y_su_key_id_se_comprueba() {
        let p = publicador(Modo::Determinista);
        let v = p.clave_publica().vkey();
        let leida = ClaveDeNota::leer_vkey(&v).unwrap();
        assert_eq!(leida.bytes(), p.clave_publica().bytes());
        assert_eq!(leida.id(), p.clave_publica().id());
        // Otro key_id, otro tipo, otra longitud: no.
        let (nombre, resto) = v.split_once('+').unwrap();
        let (_, b64) = resto.split_once('+').unwrap();
        assert!(ClaveDeNota::leer_vkey(&format!("{nombre}+00000000+{b64}")).is_err());
        let mut material = base64_decode(b64).unwrap();
        material[0] = 0x04;
        let id = &resto[..8];
        assert!(
            ClaveDeNota::leer_vkey(&format!("{nombre}+{id}+{}", base64_encode(&material))).is_err()
        );
        assert!(ClaveDeNota::nueva(nombre, &[0; 100]).is_err());
        assert!(ClaveDeNota::nueva("con espacio", p.clave_publica().bytes()).is_err());
    }

    #[test]
    fn lo_firmado_verifica_con_sal_y_sin_ella() {
        for modo in [Modo::ConSal, Modo::Determinista] {
            let p = publicador(modo);
            for tamano in [0, 1, 5] {
                let mut c = checkpoint(tamano);
                if tamano == 0 {
                    c.raiz = hash_empty();
                }
                let nota = p.firmar(&c, 1_790_000_000).unwrap();
                let v = verificar_nota(&nota, p.clave_publica()).unwrap();
                assert_eq!(v.checkpoint, c);
                assert_eq!(v.marca, 1_790_000_000);
                assert!(v.ajenas.is_empty());
            }
        }
        // Con sal, dos firmas del mismo checkpoint difieren; sin ella, no.
        let c = checkpoint(3);
        let s = publicador(Modo::ConSal);
        assert_ne!(s.firmar(&c, 9).unwrap(), s.firmar(&c, 9).unwrap());
        let d = publicador(Modo::Determinista);
        assert_eq!(d.firmar(&c, 9).unwrap(), d.firmar(&c, 9).unwrap());
    }

    #[test]
    fn el_publicador_no_firma_otro_origen_ni_un_vacio_con_raiz() {
        let p = publicador(Modo::Determinista);
        let mut c = checkpoint(4);
        c.origen = origen_del_medio(&[1; 32]);
        assert!(matches!(
            p.firmar(&c, 1),
            Err(ErrorDeNota::OrigenAjeno { .. })
        ));
        assert!(p.firmar(&checkpoint(0), 1).is_err());
    }
}
