//! # `zk-ssl-verify` — verificar una cabeza de época **sin el operador**
//!
//! Todo lo que un **tercero** necesita para comprobar **dos** cosas sobre
//! una cabeza de época firmada: **quién la emitió** —[`verificar_cabeza`],
//! desde §243— y **quién la atestiguó** —[`verificar_cofirma`], desde
//! §297—. Las dos afirmaciones son distintas y ninguna implica a la otra:
//! un operador puede firmar solo, y un testigo puede cofirmar una cabeza
//! que resulte estar mal.
//!
//! ⚠️ Aquí decía «Nada más» y **era verdad hasta que dejó de serlo**. Se
//! corrige nombrando lo que hay en vez de cerrando la puerta: quien añada
//! una tercera afirmación repara esta frase en el mismo corte.
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
//! la capa, ni del nodo, ni del cable**. Si algún día importa algo de
//! ellos, se habrá vuelto a caer en el problema.
//!
//! ⚠️ Aquí decía «solo de `xmss`», y **§292 lo dejó falso** al reexportar
//! `zk_ssl_hash` para que el cli y el bin no ganaran una dependencia por
//! las composiciones del digest. La letra se corrige; el espíritu —qué NO
//! puede entrar— se mantiene. **La verdad de hoy se mide en su
//! `[dependencies]`, no aquí**: una lista en prosa vuelve a caducar a la
//! primera dependencia legítima, y ya caducó una vez.
//!
//! ⚠️ Y debería ser **el crate que menos cambie**. Su superficie son hoy
//! **cuatro familias**: las FIRMAS (dominios, versión de formato,
//! `verificar_cabeza`, `verificar_cofirma`), las PRUEBAS de contenido
//! (inclusión, acuses, MMR), la REVERIFICACIÓN de un registro sin el nodo,
//! y las composiciones del digest **reexportadas** de `zk-ssl-hash`.
//!
//! ⚠️ **Las familias se nombran; los elementos NO se enumeran.** Aquí
//! decía «el dominio, la versión de formato y `verificar_cabeza`», y para
//! cuando alguien volvió a leerlo eran quince: §256 y §275 metieron la
//! inclusión, §274 los acuses, §279 la reverificación, §291 el MMR y §292
//! los reexports. **Dos de esos sellos escribieron al lado que la
//! superficie crecía —y no subieron la corrección a este párrafo**, que es
//! justo lo que §247 manda hacer. **La verdad se mide en los `pub` de este
//! fichero.** Un pin que no se mueve casi no cuesta; una prosa que no
//! puede caducar, tampoco.
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
    verificar_acuse_v4, verificar_inclusion_v4, verificar_acuse_v5, verificar_inclusion_v5,
    verificar_acuse_v6, verificar_inclusion_v6,
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
/// RFC-0006 E3b (§419): las reglas del arbol de CONSUMOS publicados —
/// que hoja se sube, que posicion le toca y el cruce que ata el camino a
/// ESE consumo. Puras, sin firmas, y sin la capa: el mando las usa para
/// sostener presencia y ausencia dentro de un sobre de evidencia.
pub mod consumos;

/// RFC-0007 E3b (§458): las reglas del arbol de CONGELADOS -la profundidad
/// que fija el verificador, el cruce con el indice de la cuenta y la hoja no
/// vacia-. Puras y sin la capa: el mando las usa para sostener el rechazo
/// `AccountFrozen` dentro de un sobre de evidencia.
pub mod congelados;

/// RFC-0007 E5 (§475): las reglas del arbol de CUENTAS -la profundidad que
/// fija el verificador, el cruce con el indice y la hoja VACIA-. El espejo de
/// `congelados`: puras y sin la capa, el mando las usa para sostener el
/// rechazo `AccountNotFound` dentro de un sobre de evidencia.
pub mod cuentas;

/// RFC-0010 E2b (S562): las reglas del arbol de RECIBOS DE RECEPCION -la era
/// que el recibo declara, su posicion densa dentro de ella, la hoja con
/// dominio propio y la ventana de la promesa-. Puras y sin la capa, como sus
/// cuatro hermanas: el nodo las usara para CONSTRUIR el arbol (E2c) y el mando
/// para recomponer su raiz sin nodo.
///
/// OJO: la era NO se computa como la epoca del acuse. Ver la cabecera del modulo.
pub mod recibos;

/// ⚠️ §643 · **El acta de clave** (RFC-0015, E2): el preambulo que firma un acta y las reglas
/// con que un tercero juzga una rotacion -la clave que entra es la que la previa comprometio, y
/// la cuenta sigue-. Sin la cadena ni las cabezas: eso es de los sobres (E5).
pub mod actas;

pub use zk_ssl_hash::{
    epoch_digest_v2, epoch_digest_v3, epoch_digest_v4, epoch_digest_v5, epoch_digest_v6,
};

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

/// Separación de dominio de **la cofirma del testigo** (§297).
///
/// ⚠️ **Sin versión dentro**, por lo mismo que el de arriba (§236): la
/// versión va en el preámbulo, y dos marcadores que pueden discrepar valen
/// menos que uno. Lo hace cumplir [`el_dominio_de_cofirma_no_lleva_la_version_dentro`].
///
/// ⚠️ Es un dominio **distinto** a propósito: una firma de operador nunca
/// puede presentarse como cofirma de testigo, ni al revés.
///
/// [`el_dominio_de_cofirma_no_lleva_la_version_dentro`]: #
pub const DOMINIO_COFIRMA: &[u8] = b"ZK-SSL-witness-cosign";

/// Versión del formato de cabeza que entra en la firma.
///
/// ⚠️ Sube cuando cambian **los campos de `EpochHead`**, no cuando cambia el
/// cable. Son ejes distintos: `zkssl/0.3` gobierna qué viaja; esto, qué
/// entra en la firma. 4 -> 5 en el §452 (RFC-0007 E1b): la familia del estado comprometido
/// entra en `EpochHead` y el nodo firma v5; el cable sigue en `zkssl/0.3`. 5 -> 6 en el §570
/// (RFC-0010 E2d): la pareja de recepcion entra en `EpochHead`, `digest()` compone v6 y el nodo
/// firma v6; el cable sigue en `zkssl/0.3`.
pub const VERSION_FORMATO: u8 = 6;

/// Las versiones de cabeza que un verificador del nucleo ACEPTA (RFC-0005, E2).
///
/// ⚠️ Es la UNICA puerta por la que el conjunto crece: una composicion nueva es
/// una variante nueva aqui, y el compilador marca cada `match` que la olvide.
/// El mando y el testigo NO repiten el conjunto: lo consumen, y derivan de el el
/// texto de sus rechazos. [`VERSION_FORMATO`] tiene que ser miembro (atado en
/// los tests). Un `TryFrom<u64>` que falla NO trunca: `0x103` no es un 3.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionCabeza {
    /// §275: la pareja `acusesRoot`/`n` viaja firmada.
    V2 = 2,
    /// §292: la cima y el tamano del MMR entran en el digest.
    V3 = 3,
    /// RFC-0006 E2 (§414): la raiz y la cuenta de consumos entran en el digest.
    V4 = 4,
    /// RFC-0007 E1 (§451): los siete parametros en un digest, la raiz de meta, las dos
    /// marcas de agua y el suministro entran en el digest.
    V5 = 5,
    /// RFC-0010 E2 (§558): la raiz y la cuenta de recepcion entran en el digest,
    /// con el molde exacto de la pareja de consumos que estreno la v4 (D-C).
    V6 = 6,
}

impl VersionCabeza {
    /// Todas, en orden: el conjunto que se enumera y del que se deriva el texto.
    pub const TODAS: [VersionCabeza; 5] = [
        VersionCabeza::V2,
        VersionCabeza::V3,
        VersionCabeza::V4,
        VersionCabeza::V5,
        VersionCabeza::V6,
    ];
    /// El byte que entra en el preambulo.
    pub fn as_u8(self) -> u8 {
        self as u8
    }
    /// `v2, v3, v4, v5 o v6`, DERIVADO de [`Self::TODAS`]: el texto que los rechazos citan.
    pub fn texto() -> String {
        Self::texto_de(Self::TODAS.iter().copied())
    }
    /// RFC-0006 E2a (§414): si la composicion lleva la pareja del MMR (`mmrRoot`,
    /// `mmrSize`). Es lo que la extension y el canal de la historia preguntan;
    /// antes preguntaban «¿es 3?», y una v4 los dejaba ciegos en silencio.
    pub fn lleva_mmr(self) -> bool {
        !matches!(self, VersionCabeza::V2)
    }
    /// `v3, v4, v5 o v6`: las versiones con pareja del MMR, DERIVADO de [`Self::TODAS`].
    pub fn texto_con_mmr() -> String {
        Self::texto_de(Self::TODAS.iter().copied().filter(|v| v.lleva_mmr()))
    }
    /// RFC-0007 E1a (§451): si la composicion lleva la pareja de consumos (`consRoot`,
    /// `consCount`). Es lo que el sobre de consumo y el de conflicto preguntan; antes
    /// preguntaban <<es 4?>>, y una v5 -que la lleva- se habria quedado fuera en silencio.
    pub fn lleva_consumos(self) -> bool {
        !matches!(self, VersionCabeza::V2 | VersionCabeza::V3)
    }
    /// `v4, v5 o v6`: las versiones con pareja de consumos, DERIVADO de [`Self::TODAS`].
    pub fn texto_con_consumos() -> String {
        Self::texto_de(Self::TODAS.iter().copied().filter(|v| v.lleva_consumos()))
    }
    /// RFC-0010 E2 (§558): si la composicion lleva la pareja de recepcion
    /// (`recepRoot`, `recepCount`). Mismo molde que [`Self::lleva_consumos`].
    pub fn lleva_recepcion(self) -> bool {
        matches!(self, VersionCabeza::V6)
    }
    /// `v6`: las versiones con pareja de recepcion, DERIVADO de [`Self::TODAS`].
    pub fn texto_con_recepcion() -> String {
        Self::texto_de(Self::TODAS.iter().copied().filter(|v| v.lleva_recepcion()))
    }
    /// RFC-0010 E2 (§558): si la composicion lleva la FAMILIA del estado comprometido
    /// que estreno la v5 (`paramsDigest`, `pmetaRoot`, las dos marcas y el suministro).
    /// Los sobres que la exigen preguntaban <<es exactamente V5?>>, y una v6 -que la
    /// lleva entera, porque `epoch_digest_v6` compone sobre la v5- se habria quedado
    /// fuera en silencio: la misma ceguera que la v4 tuvo con `lleva_mmr`.
    pub fn lleva_parametros(self) -> bool {
        !matches!(self, VersionCabeza::V2 | VersionCabeza::V3 | VersionCabeza::V4)
    }
    /// `v5 o v6`: las versiones con la familia del estado, DERIVADO de [`Self::TODAS`].
    pub fn texto_con_parametros() -> String {
        Self::texto_de(Self::TODAS.iter().copied().filter(|v| v.lleva_parametros()))
    }
    fn texto_de(vs: impl Iterator<Item = VersionCabeza>) -> String {
        let vs: Vec<String> = vs.map(|v| format!("v{}", v.as_u8())).collect();
        match vs.split_last() {
            Some((ult, resto)) if !resto.is_empty() => format!("{} o {}", resto.join(", "), ult),
            Some((ult, _)) => ult.clone(),
            None => String::new(),
        }
    }
}

/// Una `formatVersion` fuera del conjunto. Lleva el valor tal como llego, en
/// `u64` y sin truncar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VersionCabezaDesconocida(pub u64);

impl core::fmt::Display for VersionCabezaDesconocida {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "formatVersion {}: se aceptan cabezas {}", self.0, VersionCabeza::texto())
    }
}

impl TryFrom<u64> for VersionCabeza {
    type Error = VersionCabezaDesconocida;
    fn try_from(v: u64) -> Result<Self, Self::Error> {
        Self::TODAS
            .iter()
            .copied()
            .find(|x| u64::from(x.as_u8()) == v)
            .ok_or(VersionCabezaDesconocida(v))
    }
}

/// Bytes del RFC 8391 que ocupa la firma de este conjunto, sin el mensaje.
pub const FIRMA_RFC_BYTES: usize = 18_469;

/// Version del formato de cofirma que **el testigo ESTAMPA** (§297, §315).
///
/// ⚠️ Vivio en `zk-ssl-cli` hasta el §324, y el binario de este crate
/// llevaba **un literal desnudo** como tope porque no podia importarla: la
/// dependencia va en un solo sentido (§243). Al mudarla aqui el numero pasa a
/// tener **una sola fuente**, y aquel literal desaparece.
///
/// ⚠️ Es `u64` y su vecina [`VERSION_FORMATO`] es `u8`, **a proposito**: son
/// dos ejes distintos y no se unifican. Aquella dice QUE CAMPOS de la cabeza
/// entran en la firma; esta, QUE FORMATO tiene la cofirma. Y es **propia**,
/// distinta de los dos `DIARIO_VERSION` (el del testigo y el del nodo, §314):
/// artefactos con destinatarios distintos evolucionan por separado.
///
/// ⚠️ El parrafo que justificaba el literal decia que este crate "no depende
/// de nadie del proyecto". Depende de `zk-ssl-hash`. Lo cierto, y lo que la
/// regla del §243 pide, es que **no depende de la capa, ni del nodo, ni del
/// cable**. Corregido al pasar (§324, §247).
pub const COFIRMA_VERSION: u64 = 1;

/// La version de cofirma mas alta que este arbol sabe **LEER**.
///
/// ⚠️⚠️ **No es la misma pregunta que [`COFIRMA_VERSION`]**, y por eso son dos
/// nombres: aquella es lo que se ESCRIBE, esta es el TOPE que se ACEPTA. Hoy
/// valen lo mismo. El dia que no, quien lee puede ir por delante de quien
/// escribe y **nunca al reves**; lo hace cumplir el test de aqui abajo.
pub const COFIRMA_V_MAX: u64 = 1;

#[cfg(test)]
mod atado_de_las_dos_versiones_de_cofirma {
    use super::{COFIRMA_VERSION, COFIRMA_V_MAX};

    /// ⚠️⚠️ **EL ATADO.** Dos constantes que hoy valen lo mismo son dos
    /// productores esperando a discrepar, y la casa ya pago tres veces por no
    /// atar dos listas (§292 -> §293, §294 -> §295, §297). Esto fija la
    /// unica relacion que no puede romperse, y lo dice con los dos numeros.
    #[test]
    fn el_testigo_no_estampa_una_version_que_el_lector_no_lea() {
        assert!(
            COFIRMA_VERSION <= COFIRMA_V_MAX,
            "el testigo estampa v{} y este arbol lee hasta la v{}",
            COFIRMA_VERSION,
            COFIRMA_V_MAX
        );
    }
}

/// Una cabeza firmada, tal como la publica el operador.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CabezaFirmada {
    /// Qué campos de la cabeza entraron en la firma.
    pub version_formato: u8,
    /// Cuántas firmas se han hecho con esta clave, contando ésta.
    ///
    /// ⚠️ **No entra en el preámbulo**: es metadato para detectar reúso, no
    /// algo que la firma acredite.
    /// Desde §399 se ata al índice de hoja que la firma lleva dentro
    /// (`embebido < indice`), en la cabeza igual que en la cofirma (§332).
    pub indice: u64,
    /// La firma, en bytes del formato RFC 8391, con el preámbulo adjunto.
    pub firma: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// ⚠️ **La clave del operador no cabe en el prefijo de longitud.**
    ///
    /// El preámbulo de cofirma lleva la longitud en `u16`: 65 535 bytes de
    /// techo, holgadísimo para XMSS (decenas) y para ML-DSA (1-2,6 KB). Lo
    /// que NO se hace es `len() as u16`: eso truncaría **en silencio** y el
    /// testigo firmaría un preámbulo que miente sobre su propio contenido.
    /// El día que el techo estorbe, esto se pone rojo y se ve.
    ClaveDemasiadoLarga { bytes: usize },
    /// ⚠️⚠️ **El indice declarado y el que va dentro de la firma no
    /// coinciden.** Dice lo que puede afirmar —los dos numeros no
    /// cuadran— y **no acusa a nadie de mentir**.
    ///
    /// ⚠️ Mira el indice de HOJA, el que la firma acredita. No confundir
    /// con la clase `indice-repetido` del tercero, que mira la REPETICION
    /// sobre ese mismo numero: son dos cosas distintas.
    IndiceDiscordante { declarado: u64, embebido: u64 },
    /// La firma no llega ni al ancho del indice, asi que no lo lleva.
    FirmaSinIndice { bytes: usize, esperado: usize },
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
            VerificaError::ClaveDemasiadoLarga { bytes } => write!(
                f,
                "la clave del operador mide {bytes} bytes y el prefijo de \
                 longitud del preambulo es u16 (techo 65535)"
            ),
            VerificaError::IndiceDiscordante { declarado, embebido } => write!(
                f,
                "el indice declarado ({declarado}) no cuadra con el que va \
                 dentro de la firma ({embebido}): el declarado no entra en el \
                 preambulo, asi que la firma no lo acredita"
            ),
            VerificaError::FirmaSinIndice { bytes, esperado } => write!(
                f,
                "la firma mide {bytes} bytes y el indice de hoja ocupa \
                 {esperado}: no lleva indice que comprobar"
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
/// Los bytes exactos: `spec/NUCLEO.md`, sección 6, y su KAT en `spec/vectors/nucleo/` (§411).
pub fn preambulo(version: u8, epoch_digest: &[u8; 32]) -> Vec<u8> {
    let mut v = Vec::with_capacity(DOMINIO.len() + 1 + 32);
    v.extend_from_slice(DOMINIO);
    v.push(version);
    v.extend_from_slice(epoch_digest);
    v
}

/// El preámbulo exacto de **una cofirma de testigo**. Como el de arriba,
/// **es superficie de conformidad**: una segunda implementación tiene que
/// producir estos bytes.
///
/// Los bytes exactos: `spec/NUCLEO.md`, sección 6, y su KAT en `spec/vectors/nucleo/` (§411).
///
/// ⚠️ **La clave del operador va DENTRO, y esa es la razón de ser de esta
/// función.** Sin ella, una cofirma sería **transferible**: valdría para
/// cualquiera que emitiese ese mismo `epoch_digest`. Lo que el testigo
/// atestigua no es «este digest existe», sino «**este operador** publicó
/// este digest». Lo hace cumplir [`una_cofirma_bajo_otra_clave_de_operador_se_rechaza`].
///
/// ⚠️ **En BYTES, no en hex.** El testigo custodia la clave como hex del
/// cable, pero el verificador tercero la recibe en bytes: pedirle que
/// reconstruya la misma cadena hex —capitalización, prefijo, ceros a la
/// izquierda— es fabricar discrepancias entre implementaciones que no
/// mienten. Los bytes tienen una representación; el hex, muchas.
///
/// ⚠️ **El prefijo de longitud NO es adorno.** El cuarto campo es de
/// longitud variable, y `zk-ssl-hash` ya dejó escrito el precedente para
/// `commit_operation`: sin relleno ni prefijo, dos mensajes del mismo
/// dominio con longitudes distintas **podrían colisionar**. Allí la
/// suposición de longitud fija se cumplía y se declaró; aquí NO se cumple
/// —la nota 87 (ML-DSA) trae claves de otro tamaño— así que se resuelve en
/// vez de suponerse. Un `assert` de longitud fija sería una trampa armada:
/// funciona hoy y el día que estorbe alguien lo relaja para que pase.
///
/// ⚠️ **El campo con prefijo va el ÚLTIMO por diseño.** Si algún día entra
/// un quinto campo, lo gobierna el byte de versión (§236), no la posición.
pub fn preambulo_cofirma(
    version: u8,
    epoch_digest: &[u8; 32],
    clave_del_operador: &[u8],
) -> Result<Vec<u8>, VerificaError> {
    let n: u16 = clave_del_operador.len().try_into().map_err(|_| {
        VerificaError::ClaveDemasiadoLarga { bytes: clave_del_operador.len() }
    })?;
    let mut v = Vec::with_capacity(DOMINIO_COFIRMA.len() + 1 + 32 + 2 + n as usize);
    v.extend_from_slice(DOMINIO_COFIRMA);
    v.push(version);
    v.extend_from_slice(epoch_digest);
    v.extend_from_slice(&n.to_be_bytes());
    v.extend_from_slice(clave_del_operador);
    Ok(v)
}

/// **El indice de hoja que va DENTRO de la firma.**
///
/// ⚠️ **Lectura de material PUBLICADO**, y por eso vive aqui y no en el
/// guardian: `zk_ssl_guardian::indice_de_sk` lee el **SK**, material
/// secreto del firmante. Sacar esta funcion alli obligaria a un tercero a
/// compilar el crate del firmante para verificar, que es justo lo que el
/// §243 deshizo. Comparten el ancho del campo; no comparten el invariante.
///
/// ```text
/// firma := indice(ANCHO_INDICE bytes, big-endian) ‖ R(n) ‖ ...
/// ```
///
/// ⚠️ Falla **CERRADA**: una firma que no llega al ancho no da indice, da
/// error. `Signature::try_from` no valida longitud —es casi un envoltorio—
/// asi que esta comprobacion no la hace nadie mas.
pub fn indice_de_firma(firma: &[u8]) -> Result<u64, VerificaError> {
    if firma.len() < ANCHO_INDICE {
        return Err(VerificaError::FirmaSinIndice {
            bytes: firma.len(),
            esperado: ANCHO_INDICE,
        });
    }
    let mut v = 0u64;
    for b in &firma[..ANCHO_INDICE] {
        v = (v << 8) | *b as u64;
    }
    Ok(v)
}

/// Ancho del indice de hoja en bytes: ⌈h/8⌉ = 5 para `h = 40`.
///
/// ⚠️⚠️ **ES LA SEGUNDA COPIA, NO UNA TERCERA.** `xmss` **no expone**
/// `index_bytes` —es `pub(crate)` en `params.rs`— asi que el ancho esta
/// DECLARADO en dos sitios: aqui y en `zk-ssl-guardian::ancho_indice()`,
/// que es la primera. **Las ata un test**, no la buena fe:
/// `el_ancho_del_indice_esta_atado_al_guardian`. Un censo que cuente tres
/// fuentes se estara equivocando; si algun dia hay una tercera, sobra.
///
/// Y desde el §586 esta atada a `xmss`: no expone `index_bytes`, pero SI las
/// constantes de su `XmssParameter` -`NAME`, `SK_LEN`, `SIG_LEN`-, y de ellas
/// se DERIVA el ancho en `el_ancho_del_indice_sale_del_conjunto_de_xmss`.
pub const ANCHO_INDICE: usize = 5;

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
/// Aplica el apano del OID sobre los CUATRO primeros bytes, EN SU SITIO.
///
/// ⚠️⚠️ **Este es el UNICO sitio donde el apano se aplica.** Entran por
/// aqui [`clave_desde_bytes`] -que lee lo publicado- y el FIRMANTE, cuando
/// resincroniza su clave. Dos copias del mismo apano podrian discrepar, y
/// discrepar aqui significa no poder leer una clave legitima (S243).
///
/// ⚠️ Su centinela sigue siendo el de siempre:
/// `el_apano_del_oid_sigue_haciendo_falta` se pone ROJO el dia que upstream
/// arregle `parse_oid_and_params`, y entonces esto sobra entero.
pub fn aplicar_apano_del_oid(bytes: &mut [u8]) -> Result<(), VerificaError> {
    if bytes.len() < 4 {
        return Err(VerificaError::ClaveIlegible(format!(
            "{} bytes: no caben ni los 4 del OID",
            bytes.len()
        )));
    }
    let raw = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) | OFFSET_MT_UPSTREAM;
    bytes[..4].copy_from_slice(&raw.to_be_bytes());
    Ok(())
}

#[cfg(test)]
mod el_apano_tiene_un_solo_dueno {
    use super::*;

    #[test]
    fn deja_el_oid_que_xmss_espera_y_no_toca_nada_mas() {
        let mut b = vec![0u8; 68];
        b[3] = 5;
        b[10] = 0xab;
        aplicar_apano_del_oid(&mut b).expect("aplicar");
        assert_eq!(
            u32::from_be_bytes([b[0], b[1], b[2], b[3]]),
            0x0001_0005,
            "el OID tiene que pasar al registro de XMSS^MT"
        );
        assert_eq!(b[10], 0xab, "y no toca un byte mas alla del OID");
    }

    /// El apano es un OR de un bit, asi que aplicarlo dos veces da lo mismo.
    /// No es una excusa para aplicarlo dos veces: es que no puede corromper.
    #[test]
    fn aplicarlo_dos_veces_da_lo_mismo() {
        let mut una = vec![0u8; 68];
        una[3] = 5;
        aplicar_apano_del_oid(&mut una).expect("una");
        let mut dos = una.clone();
        aplicar_apano_del_oid(&mut dos).expect("dos");
        assert_eq!(una, dos, "el apano es idempotente");
    }

    #[test]
    fn un_buffer_corto_falla_y_no_se_toca() {
        let mut corto = vec![9u8, 9, 9];
        assert!(aplicar_apano_del_oid(&mut corto).is_err());
        assert_eq!(corto, vec![9u8, 9, 9], "y no se toca al fallar");
    }
}

/// ⚠️⚠️ S335 CORRIGE, SIN BORRARLO, el "Solo al leer" de arriba: el
///    FIRMANTE tambien entra por el apano, al resincronizar su clave. Lo que
///    SIGUE siendo cierto es lo que importa: **el apano no toca el cable**. El
///    SK no se publica jamas, y lo publicado sigue llevando su OID del RFC,
///    `0x00000005`, sin apano ninguno.
pub fn clave_desde_bytes(rfc: &[u8]) -> Result<VerifyingKey<Conjunto>, VerificaError> {
    // §664: lo publicado lleva el OID de RFC 8391, sin el bit del apano. Sin
    // esto, `0x00010005` -el apano ya aplicado- leia la MISMA clave que
    // `0x00000005`: una clave con dos escrituras, y un «conflicto entre dos
    // libros» con un solo libro escrito dos veces salia VERDE. La segunda
    // implementacion ya lo rechazaba, con este mismo texto.
    if rfc.len() >= 4 {
        let oid = u32::from_be_bytes([rfc[0], rfc[1], rfc[2], rfc[3]]);
        if oid & OFFSET_MT_UPSTREAM != 0 {
            return Err(VerificaError::ClaveIlegible(format!(
                "OID {oid:#010x} no es un XMSS^MT de RFC 8391"
            )));
        }
    }
    let mut b = rfc.to_vec();
    aplicar_apano_del_oid(&mut b)?;
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
///
/// ⚠️ Desde §399 ata además el `indice` declarado al que va dentro de la
/// firma, con el invariante del §332 (`embebido < declarado`): el declarado
/// queda acotado **por abajo**. Declarar más de lo firmado pasa, y se dice.
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
    // ⚠️⚠️ EL ATADO (§399), calcado del de la cofirma (§332). El `indice`
    // declarado no entra en el preambulo, asi que la firma no lo acredita;
    // aqui se compara con el que va DENTRO, que no se puede falsear sin
    // romper la firma. Mismo invariante y por la misma razon:
    // `embebido < declarado`, porque el productor (`firma_cabeza::firmar`)
    // hace `reservar()` -> firmar, y un indice huerfano ensancha el desfase
    // para siempre. Exigir el +1 daria rojo sobre material legitimo.
    //
    // ⚠️ Esto acota el declarado POR ABAJO. Un sobre puede declarar mas de
    // lo que firmo y pasa: lo que la firma acredita es el embebido, y es el
    // que el mando imprime. Va despues de comparar el preambulo, para que
    // una cofirma presentada como cabeza siga cayendo por el DOMINIO.
    let embebido = indice_de_firma(&c.firma)?;
    if embebido >= c.indice {
        return Err(VerificaError::IndiceDiscordante { declarado: c.indice, embebido });
    }
    Ok(())
}

/// **La función del tercero, para la cofirma.** Comprueba que **este
/// testigo** atestiguó **esta cabeza de este operador**, sin el nodo, sin
/// el testigo y sin el operador: solo con lo publicado.
///
/// ⚠️ Mismo paso que no se puede saltar que en [`verificar_cabeza`]:
/// `verify()` devuelve **el mensaje que la firma lleva dentro**, así que
/// una cofirma legítima de OTRA cabeza —o de la misma bajo OTRO operador—
/// pasaría el `verify()` a secas. Lo que cierra la puerta es comparar con
/// el preámbulo esperado.
///
/// ⚠️ La clave del operador que se pasa aquí es **la que el tercero tiene
/// por buena**. Si no es la que el testigo ancló, la cofirma no verifica —
/// y eso es exactamente lo que debe pasar.
pub fn verificar_cofirma(
    clave_del_testigo: &[u8],
    epoch_digest: &[u8; 32],
    clave_del_operador: &[u8],
    c: &CabezaFirmada,
) -> Result<(), VerificaError> {
    let vk = clave_desde_bytes(clave_del_testigo)?;
    let sig = Signature::<Conjunto>::try_from(c.firma.as_slice())
        .map_err(|e| VerificaError::FirmaIlegible(format!("{e:?}")))?;
    let recuperado = vk
        .verify(&sig)
        .map_err(|e| VerificaError::NoVerifica(format!("{e:?}")))?;
    // ⚠️ EL PASO QUE NO SE PUEDE SALTAR.
    let esperado = preambulo_cofirma(c.version_formato, epoch_digest, clave_del_operador)?;
    if recuperado != esperado {
        return Err(VerificaError::PreambuloDistinto {
            esperado: esperado.len(),
            recibido: recuperado.len(),
        });
    }
    // ⚠️⚠️ EL ATADO (§332). El `indice` declarado **no entra en el
    // preambulo** —lo dice el doc de `CabezaFirmada`— asi que un tercero se
    // estaba creyendo un numero que la firma no acredita. Aqui se compara
    // con el que va DENTRO, que no se puede falsear sin romper la firma.
    //
    // ⚠️⚠️ El invariante es `embebido < declarado`, **NO**
    // `declarado == embebido + 1`. `GuardianIndice::reservar` persiste
    // `actual + 1` ANTES de firmar y el contador **nunca retrocede**, asi
    // que un indice HUERFANO —proceso muerto entre la reserva y la firma,
    // «correcto y esperado» segun su propio doc— ensancha el desfase para
    // siempre. Exigir el +1 daria rojo sobre material legitimo.
    //
    // ⚠️ Convencion del PRODUCTOR, y se dice: `reservar()` -> `sign`. Con
    // `declarado == 0` no cuadra nunca, y es correcto: el contador empieza
    // a devolver en 1.
    //
    // ⚠️ Esto NO caza el reinicio —contador 6 y clave 0 dan 0 < 7— y no
    // pretende hacerlo: de eso se ocupa la REPETICION del indice embebido.
    let embebido = indice_de_firma(&c.firma)?;
    if embebido >= c.indice {
        return Err(VerificaError::IndiceDiscordante { declarado: c.indice, embebido });
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

    /// Firma DOS veces con la misma clave y devuelve la segunda: la hoja
    /// embebida es la 1, y el `indice` declarado que le corresponde es 2
    /// (`reservar()` devuelve `actual + 1`: la convencion del productor).
    fn firmado_en_la_hoja_1(digest: &[u8; 32]) -> (Vec<u8>, CabezaFirmada) {
        let mut s = [0u8; 96];
        for (i, b) in s.iter_mut().enumerate() {
            *b = (i as u8).wrapping_mul(17).wrapping_add(4);
        }
        let mut kp = KeyPair::<Conjunto>::from_seed(&s).expect("keygen");
        let pk = kp.verifying_key().as_ref().to_vec();
        let pre = preambulo(VERSION_FORMATO, digest);
        let _hoja_0 = kp.signing_key().sign(&pre).expect("firmar la hoja 0");
        let sig = kp.signing_key().sign(&pre).expect("firmar la hoja 1");
        assert_eq!(
            indice_de_firma(sig.as_ref()).expect("la firma lleva indice"),
            1,
            "la segunda firma va en la hoja 1"
        );
        (
            pk,
            CabezaFirmada { version_formato: VERSION_FORMATO, indice: 2, firma: sig.as_ref().to_vec() },
        )
    }

    /// §399 · el declarado IGUAL al embebido no cuadra: es lo que `atrasado` ensena.
    #[test]
    fn el_indice_declarado_igual_al_embebido_se_rechaza() {
        let d = [0x63u8; 32];
        let (pk, mut c) = firmado_en_la_hoja_1(&d);
        c.indice = 1;
        match verificar_cabeza(&pk, &d, &c) {
            Err(VerificaError::IndiceDiscordante { declarado: 1, embebido: 1 }) => {}
            otro => panic!("declarado 1 sobre la hoja 1 debe dar IndiceDiscordante, y dio: {otro:?}"),
        }
    }

    /// §399 · el cero no cuadra nunca: el contador empieza a devolver en 1.
    #[test]
    fn el_indice_declarado_a_cero_se_rechaza() {
        let d = [0x64u8; 32];
        let (pk, mut c) = firmado_en_la_hoja_1(&d);
        c.indice = 0;
        match verificar_cabeza(&pk, &d, &c) {
            Err(VerificaError::IndiceDiscordante { declarado: 0, embebido: 1 }) => {}
            otro => panic!("declarado 0 debe dar IndiceDiscordante, y dio: {otro:?}"),
        }
    }

    /// §399 · LA COTA, escrita como test: el declarado solo esta acotado por
    /// abajo. Declarar mas de lo firmado PASA, y no es un descuido: exigir el
    /// +1 daria rojo sobre el indice huerfano, que es legitimo (§332).
    #[test]
    fn el_indice_declarado_por_encima_pasa_y_es_la_cota_declarada() {
        let d = [0x65u8; 32];
        let (pk, mut c) = firmado_en_la_hoja_1(&d);
        verificar_cabeza(&pk, &d, &c).expect("2 sobre la hoja 1: el productor honesto");
        c.indice = 1_002;
        verificar_cabeza(&pk, &d, &c)
            .expect("1002 sobre la hoja 1 PASA: el declarado solo esta acotado por abajo");
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

    /// §664: la clave publicada con el OID ya «apanado» (`0x00010005`) leia la
    /// misma clave que la de RFC 8391. Ahora no se lee, y la firma legitima
    /// tampoco verifica con ella. Falsador: sin la comprobacion, las dos `is_err`
    /// fallan.
    #[test]
    fn una_clave_con_el_oid_ya_apanado_no_se_lee() {
        let (pk, cf) = firmado(&[7u8; 32]);
        assert!(clave_desde_bytes(&pk).is_ok(), "la publicada se lee");
        let mut alterna = pk.clone();
        alterna[1] |= 0x01;
        assert_eq!(u32::from_be_bytes([alterna[0], alterna[1], alterna[2], alterna[3]]), 0x0001_0005);
        assert!(clave_desde_bytes(&alterna).is_err(), "la escritura alterna no es una clave publicada");
        assert!(verificar_cabeza(&alterna, &[7u8; 32], &cf).is_err());
    }

    // ── la COFIRMA del testigo (§297) ──

    /// Un testigo con clave propia, cofirmando una cabeza de un operador.
    fn cofirmado(digest: &[u8; 32], clave_op: &[u8]) -> (Vec<u8>, CabezaFirmada) {
        let mut s = [0u8; 96];
        for (i, b) in s.iter_mut().enumerate() {
            // ⚠️ Semilla DISTINTA de la de `firmado`: el testigo no es el
            //    operador, y un test que los confunda no probaria nada.
            *b = (i as u8).wrapping_mul(31).wrapping_add(9);
        }
        let mut kp = KeyPair::<Conjunto>::from_seed(&s).expect("keygen");
        let pk = kp.verifying_key().as_ref().to_vec();
        let pre = preambulo_cofirma(VERSION_FORMATO, digest, clave_op).expect("preambulo");
        let sig = kp.signing_key().sign(&pre).expect("firmar");
        (pk, CabezaFirmada { version_formato: VERSION_FORMATO, indice: 1, firma: sig.as_ref().to_vec() })
    }

    #[test]
    fn el_preambulo_de_cofirma_lleva_los_cinco_campos_en_ese_orden() {
        let d = [0xABu8; 32];
        let k = vec![0xCDu8; 68];
        let p = preambulo_cofirma(3, &d, &k).expect("cabe");
        assert_eq!(p.len(), DOMINIO_COFIRMA.len() + 1 + 32 + 2 + 68);
        assert_eq!(p.len(), 21 + 1 + 32 + 2 + 68);
        let o = DOMINIO_COFIRMA.len();
        assert_eq!(&p[..o], DOMINIO_COFIRMA);
        assert_eq!(p[o], 3);
        assert_eq!(&p[o + 1..o + 33], &d);
        assert_eq!(&p[o + 33..o + 35], &68u16.to_be_bytes(), "la longitud, big-endian");
        assert_eq!(&p[o + 35..], &k[..]);
    }

    #[test]
    fn el_dominio_de_cofirma_no_lleva_la_version_dentro() {
        // Dos marcadores de version que pueden discrepar valen menos que uno.
        let s = String::from_utf8(DOMINIO_COFIRMA.to_vec()).expect("utf8");
        assert!(!s.contains("v1"), "el dominio no debe llevar version: {s}");
        assert!(!s.contains("-v"), "el dominio no debe llevar version: {s}");
    }

    #[test]
    fn los_dos_dominios_son_distintos_y_ninguno_prefija_al_otro() {
        // ⚠️ Una firma de operador no puede presentarse como cofirma, ni al
        //    reves. Si uno fuera prefijo del otro, el preambulo dejaria de
        //    separarlos por si solo.
        assert_ne!(DOMINIO, DOMINIO_COFIRMA);
        assert!(!DOMINIO_COFIRMA.starts_with(DOMINIO));
        assert!(!DOMINIO.starts_with(DOMINIO_COFIRMA));
    }

    #[test]
    fn dos_claves_de_longitudes_distintas_no_dan_el_mismo_preambulo() {
        // ⚠️⚠️ EL TEST DEL PREFIJO. Es la respuesta al precedente que
        //    `zk-ssl-hash` dejo escrito para `commit_operation`: sin relleno
        //    ni prefijo, dos mensajes del mismo dominio con longitudes
        //    distintas podrian colisionar. Con el prefijo, no.
        let d = [7u8; 32];
        let a = preambulo_cofirma(3, &d, &[0xAA; 4]).expect("cabe");
        let b = preambulo_cofirma(3, &d, &[0xAA; 5]).expect("cabe");
        assert_ne!(a, b);
        assert!(!b.starts_with(&a), "uno no puede ser prefijo del otro");
    }

    #[test]
    fn una_clave_que_no_cabe_en_el_prefijo_da_error_en_vez_de_truncar() {
        // ⚠️⚠️ `len() as u16` daria 4464 para 70000 bytes: el testigo
        //    firmaria un preambulo que MIENTE sobre su propio contenido.
        let d = [0u8; 32];
        let enorme = vec![0u8; 70_000];
        match preambulo_cofirma(3, &d, &enorme) {
            Err(VerificaError::ClaveDemasiadoLarga { bytes }) => assert_eq!(bytes, 70_000),
            otro => panic!("debia decir que no cabe, no truncar: {otro:?}"),
        }
        // Y el techo exacto SI cabe.
        assert!(preambulo_cofirma(3, &d, &vec![0u8; 65_535]).is_ok());
    }

    #[test]
    fn una_cofirma_bien_hecha_verifica() {
        let d = [0x5Au8; 32];
        let (pk_op, _) = firmado(&d);
        let (pk_testigo, c) = cofirmado(&d, &pk_op);
        verificar_cofirma(&pk_testigo, &d, &pk_op, &c).expect("debe verificar");
        assert_eq!(c.firma.len(), FIRMA_RFC_BYTES + 21 + 1 + 32 + 2 + 68);
    }

    #[test]
    fn una_cofirma_bajo_otra_clave_de_operador_se_rechaza() {
        // ⚠️⚠️ EL TEST QUE JUSTIFICA EL CUARTO CAMPO. Sin la clave del
        //    operador dentro, esta cofirma seria TRANSFERIBLE: valdria para
        //    cualquiera que emitiese el mismo digest. Con ella, no.
        let d = [0x5Au8; 32];
        let (pk_op, _) = firmado(&d);
        let (pk_testigo, c) = cofirmado(&d, &pk_op);
        let mut otro_op = pk_op.clone();
        otro_op[10] ^= 0x01;
        assert!(
            verificar_cofirma(&pk_testigo, &d, &otro_op, &c).is_err(),
            "una cofirma NO puede valer para otro operador"
        );
        // Y una cofirma de OTRA cabeza tampoco.
        assert!(verificar_cofirma(&pk_testigo, &[0x11u8; 32], &pk_op, &c).is_err());
        // Ni la firma del OPERADOR puede pasar por cofirma: otro dominio.
        let (_, c_op) = firmado(&d);
        assert!(verificar_cofirma(&pk_testigo, &d, &pk_op, &c_op).is_err());
    }

    // ── el indice que va DENTRO de la firma (§332) ──

    #[test]
    fn el_ancho_del_indice_sale_del_conjunto_de_xmss() {
        // §586, entrada 101 del BACKLOG: `xmss` no expone `index_bytes` ni `full_height`,
        // pero SI las constantes de su `XmssParameter`: el nombre del conjunto y las
        // longitudes del SK y de la firma. De ahi se DERIVA el ancho con la regla de la
        // propia `xmss` (con d > 1, ⌈h/8⌉) y se ata a ANCHO_INDICE -y, por el test de
        // abajo, al guardian-. Cambiar `Conjunto` por uno de otro ancho tumba esto en
        // vez de validar mal en silencio.
        use xmss::XmssParameter;
        let nombre = <Conjunto as XmssParameter>::NAME;
        // «XMSSMT-SHA2_<h>/<d>_<bits de n>»
        let resto = nombre
            .strip_prefix("XMSSMT-SHA2_")
            .unwrap_or_else(|| panic!("{nombre}: no es un conjunto XMSS^MT con SHA2"));
        let (hd, bits) = resto.split_once('_').expect(nombre);
        let (h, d) = hd.split_once('/').expect(nombre);
        let num = |x: &str| -> usize { x.parse().expect(nombre) };
        let (h, d, n) = (num(h), num(d), num(bits) / 8);
        let ancho = if d == 1 { 4 } else { h.div_ceil(8) };
        assert_eq!(ANCHO_INDICE, ancho, "{nombre}: xmss dice un ancho de {ancho} bytes");
        // Las longitudes publicas lo confirman por dos lados, con las formulas del RFC 8391.
        assert_eq!(
            <Conjunto as XmssParameter>::SK_LEN,
            4 + ANCHO_INDICE + 4 * n,
            "{nombre}: el SK es OID + indice + cuatro valores de N bytes"
        );
        let wots = (2 * n + 3) * n;
        assert_eq!(
            <Conjunto as XmssParameter>::SIG_LEN,
            ANCHO_INDICE + n + d * wots + h * n,
            "{nombre}: la firma es indice + r + d firmas WOTS + h nodos"
        );
        // Y el techo del guardian, 2^(8*ancho) (§335), solo es 2^h si h llena el campo.
        assert_eq!(8 * ANCHO_INDICE, h, "{nombre}: el techo 2^(8*ancho) no seria 2^h");
    }

    #[test]
    fn el_ancho_del_indice_esta_atado_al_guardian() {
        // ⚠️⚠️ DOS PRODUCTORES DEL MISMO CONTRATO, y llevaban sin atar desde
        // el §298. `xmss` no expone `index_bytes`, asi que el ancho vive
        // declarado aqui y en `zk-ssl-guardian`. Esto los ata.
        //
        // ⚠️ Se ata por el ANCHO, no por el total: un cambio que
        // redistribuyera OID e indice dejando 137 bytes pasaria limpio.
        const OID: usize = 4;
        let mut sk = vec![0u8; OID + ANCHO_INDICE + 4 * 32];
        for i in 0..ANCHO_INDICE {
            sk[OID + i] = (i as u8) + 1;
        }
        let mut esperado = 0u64;
        for i in 0..ANCHO_INDICE {
            esperado = (esperado << 8) | (i as u64 + 1);
        }
        match zk_ssl_guardian::indice_de_sk(&sk) {
            Ok(v) => assert_eq!(
                v, esperado,
                "el ancho o el orden de bytes han discrepado: zk-ssl-verify::\
                 ANCHO_INDICE = {ANCHO_INDICE} espera {esperado}, y \
                 zk-ssl-guardian::indice_de_sk lee {v}"
            ),
            Err(e) => panic!(
                "las DOS fuentes del ancho del indice han discrepado. \
                 zk-ssl-verify::ANCHO_INDICE = {ANCHO_INDICE} (verify/src/lib.rs) \
                 y zk-ssl-guardian::ancho_indice() (guardian/src/lib.rs) \
                 rechaza un SK de ese ancho: {e}"
            ),
        }
    }

    #[test]
    fn el_indice_embebido_de_una_clave_recien_nacida_es_cero() {
        // ⚠️ El INSTRUMENTO se valida contra lo que ya se sabe cierto: una
        // clave que acaba de nacer firma con la hoja 0, y el guardian
        // declara 1 porque `reservar()` persiste `actual + 1`.
        let (_, c) = cofirmado(&[0x5Au8; 32], &[0xAAu8; 68]);
        assert_eq!(indice_de_firma(&c.firma).expect("la firma lleva indice"), 0);
        assert_eq!(c.indice, 1, "y el declarado va uno por delante");
    }

    #[test]
    fn una_firma_mas_corta_que_el_ancho_no_da_indice() {
        // Falla CERRADA: sin bytes no hay numero, y no se inventa uno.
        match indice_de_firma(&[0xAAu8; ANCHO_INDICE - 1]) {
            Err(VerificaError::FirmaSinIndice { bytes, esperado }) => {
                assert_eq!((bytes, esperado), (ANCHO_INDICE - 1, ANCHO_INDICE));
            }
            otro => panic!("una firma corta no puede dar indice: {otro:?}"),
        }
    }

    #[test]
    fn un_indice_declarado_que_no_cuadra_con_la_firma_se_rechaza() {
        // ⚠️⚠️ EL ROJO DEL §332. La firma es PERFECTAMENTE VALIDA; lo que
        // esta reescrito es el numero de al lado, que nadie firmaba y nadie
        // miraba.
        let d = [0x5Au8; 32];
        let (pk_op, _) = firmado(&d);
        let (pk_t, mut c) = cofirmado(&d, &pk_op);
        verificar_cofirma(&pk_t, &d, &pk_op, &c).expect("intacta, debe verificar");
        c.indice = 0;
        match verificar_cofirma(&pk_t, &d, &pk_op, &c) {
            Err(VerificaError::IndiceDiscordante { declarado, embebido }) => {
                assert_eq!((declarado, embebido), (0, 0), "dice los DOS numeros");
            }
            otro => panic!("el ordinal reescrito tiene que verse: {otro:?}"),
        }
    }

    // ---------- S395: la autonomia del verificador, como invariante ----------
    //
    // El Cargo.toml y la cabecera de este fichero afirman CATORCE veces que la
    // dependencia va en UN SOLO SENTIDO -este crate no depende de la capa, ni
    // del nodo, ni del cable (S243)- y hasta aqui NADA lo comprobaba.
    //
    // El operador es el CONJUNTO EXACTO y no una lista de prohibidos: un censo
    // de tres nombres es ciego al cuarto crate que nazca manana, y la propia
    // cabecera dice "si algun dia importa algo del proyecto". Dos listas, dos
    // productores, con difference por los DOS lados, como el atado del cable.
    //
    // include_str! vive bajo cfg(test): NO viaja al binario, asi que este gate
    // no ata el artefacto al arbol.
    fn deps_por_ruta_del_manifiesto(
        toml: &str,
    ) -> std::collections::BTreeSet<(String, String)> {
        let mut fuera = std::collections::BTreeSet::new();
        let mut seccion = String::new();
        for linea in toml.lines() {
            let s = linea.trim();
            if s.starts_with('#') {
                continue;
            }
            if s.starts_with('[') && s.ends_with(']') {
                seccion = s[1..s.len() - 1].to_string();
                continue;
            }
            if !seccion.ends_with("dependencies") || !s.contains("path") {
                continue;
            }
            let nombre = s.split('=').next().unwrap_or("").trim().to_string();
            if !nombre.is_empty() {
                fuera.insert((seccion.clone(), nombre));
            }
        }
        fuera
    }

    #[test]
    fn el_cierre_del_verificador_es_el_declarado() {
        let derivadas = deps_por_ruta_del_manifiesto(include_str!("../Cargo.toml"));
        let declaradas: std::collections::BTreeSet<(String, String)> = [
            ("dependencies", "zk-ssl-hash"),
            // S465 (RFC-0007 E4b-2): el juez de la prueba de edad, sin el probador.
            ("dependencies", "zk-ssl-air"),
            // §633 (RFC-0013 E4a): el medio del ancla, sin `firmar`; con ella, solo en los tests.
            ("dependencies", "zk-ssl-medio"),
            ("dev-dependencies", "zk-ssl-guardian"),
            ("dev-dependencies", "zk-ssl-medio"),
        ]
        .iter()
        .map(|(s, n)| (s.to_string(), n.to_string()))
        .collect();
        let sobran: Vec<_> = derivadas.difference(&declaradas).collect();
        let faltan: Vec<_> = declaradas.difference(&derivadas).collect();
        assert!(
            sobran.is_empty(),
            "dependencias por ruta NUEVAS y sin declarar: {sobran:?}"
        );
        assert!(
            faltan.is_empty(),
            "dependencias declaradas que ya no estan: {faltan:?}"
        );
    }

    // ---------- §694: la clausura es una LISTA CERRADA, y las fijaciones se comprueban ----------
    //
    // Hasta aqui la puerta de la clausura prohibia DOS nombres, `winter-prover` y `winterfell`:
    // un paquete nuevo cualquiera, o un probador con otro nombre, pasaba sin que nadie lo viera.
    // Y los `=` de los manifiestos se sostenian por convencion: `winter-fri` y `winter-utils` no
    // lo llevaban en ningun sitio. La 0.13.1 es la ultima version de winterfell en crates.io, del
    // 19-07-2025, y el fork no esta auditado: un cambio de dependencias tiene que poner rojo el
    // canon, no pasar en silencio.

    /// El `source` de crates.io en el `Cargo.lock`.
    const CRATES_IO: &str = "registry+https://github.com/rust-lang/crates.io-index";

    /// Un paquete del `Cargo.lock`: nombre, version, `source` (vacio sin el: el workspace y el
    /// fork) y sus aristas tal como el lock las escribe.
    struct PaqueteDelLock {
        nombre: String,
        version: String,
        source: String,
        aristas: Vec<String>,
    }

    impl PaqueteDelLock {
        /// `nombre version origen`, la forma de la lista cerrada: `ruta` sin `source`, `crates.io`
        /// del registry, y el `source` entero si viene de cualquier otro sitio.
        fn id(&self) -> String {
            let origen = match self.source.as_str() {
                "" => "ruta",
                CRATES_IO => "crates.io",
                otro => otro,
            };
            format!("{} {} {}", self.nombre, self.version, origen)
        }
    }

    fn paquetes_del_lock(lock: &str) -> Vec<PaqueteDelLock> {
        let mut fuera = Vec::new();
        for bloque in lock.split("[[package]]").skip(1) {
            let mut p = PaqueteDelLock {
                nombre: String::new(),
                version: String::new(),
                source: String::new(),
                aristas: Vec::new(),
            };
            let mut dentro = false;
            for linea in bloque.lines() {
                let s = linea.trim();
                if dentro {
                    if s == "]" {
                        dentro = false;
                    } else {
                        p.aristas.push(s.trim_end_matches(',').trim_matches('"').to_string());
                    }
                } else if let Some(v) = s.strip_prefix("name = ") {
                    p.nombre = v.trim_matches('"').to_string();
                } else if let Some(v) = s.strip_prefix("version = ") {
                    p.version = v.trim_matches('"').to_string();
                } else if let Some(v) = s.strip_prefix("source = ") {
                    p.source = v.trim_matches('"').to_string();
                } else if s == "dependencies = [" {
                    dentro = true;
                }
            }
            fuera.push(p);
        }
        fuera
    }

    /// La clausura de `raiz` en el lock, como `nombre version origen`. El lock escribe cada
    /// arista como `nombre`, `nombre version` o `nombre version (source)`, lo justo para que sea
    /// unica, y se resuelve al paquete, no al nombre: la puerta de antes juntaba las aristas de
    /// todas las versiones de un nombre. Una arista que no casa con UN paquete hace fallar el
    /// test: un caminante que pierde una arista es una puerta ciega.
    fn clausura_del_lock(lock: &str, raiz: &str) -> std::collections::BTreeSet<String> {
        let paquetes = paquetes_del_lock(lock);
        let resolver = |arista: &str| -> usize {
            let mut t = arista.splitn(3, ' ');
            let (nombre, version, source) = (t.next().unwrap_or(""), t.next(), t.next());
            let casan: Vec<usize> = paquetes
                .iter()
                .enumerate()
                .filter(|(_, p)| {
                    p.nombre == nombre
                        && version.map_or(true, |v| p.version == v)
                        && source.map_or(true, |s| {
                            s.trim_start_matches('(').trim_end_matches(')') == p.source
                        })
                })
                .map(|(i, _)| i)
                .collect();
            assert_eq!(casan.len(), 1, "la arista `{arista}` del lock casa con {casan:?}");
            casan[0]
        };
        let mut vistos = std::collections::BTreeSet::new();
        let mut cola = vec![resolver(raiz)];
        while let Some(i) = cola.pop() {
            if vistos.insert(i) {
                cola.extend(paquetes[i].aristas.iter().map(|a| resolver(a)));
            }
        }
        vistos.into_iter().map(|i| paquetes[i].id()).collect()
    }

    /// **§694: la clausura PERMITIDA del kit.** Leida del `Cargo.lock` desde `zk-ssl-verify`,
    /// con las dev-dependencias de los crates del arbol (el lock no las separa: la lista es mas
    /// estricta que la clausura normal, no mas laxa). Un paquete que entre, salga, cambie de
    /// version o de origen la pone roja. Se edita a mano, en el mismo sello que mueve el lock, y
    /// el asiento dice por que. `ruta` es el arbol: el kit, sus crates y el FORK (`winter-air`,
    /// `winter-verifier`), que entra por el `[patch]` del `Cargo.toml` raiz.
    const CLAUSURA_DEL_KIT: &str = "
        arrayref 0.3.9 crates.io
        arrayvec 0.7.8 crates.io
        autocfg 1.5.1 crates.io
        blake3 1.8.5 crates.io
        block-buffer 0.10.4 crates.io
        bumpalo 3.20.3 crates.io
        cc 1.4.0 crates.io
        cfg-if 1.0.4 crates.io
        chacha20 0.10.1 crates.io
        cmov 0.5.4 crates.io
        constant_time_eq 0.4.2 crates.io
        cpufeatures 0.2.17 crates.io
        cpufeatures 0.3.0 crates.io
        crossbeam-deque 0.8.7 crates.io
        crossbeam-epoch 0.9.20 crates.io
        crossbeam-utils 0.8.22 crates.io
        crypto-common 0.1.7 crates.io
        crypto-common 0.2.2 crates.io
        ctutils 0.4.2 crates.io
        digest 0.10.7 crates.io
        digest 0.11.3 crates.io
        either 1.17.0 crates.io
        find-msvc-tools 0.1.9 crates.io
        futures-core 0.3.33 crates.io
        futures-task 0.3.33 crates.io
        futures-util 0.3.33 crates.io
        generic-array 0.14.7 crates.io
        getrandom 0.2.17 crates.io
        getrandom 0.3.4 crates.io
        getrandom 0.4.3 crates.io
        hybrid-array 0.2.3 crates.io
        hybrid-array 0.4.15 crates.io
        itoa 1.0.18 crates.io
        js-sys 0.3.103 crates.io
        keccak 0.1.6 crates.io
        keccak 0.2.2 crates.io
        libc 0.2.189 crates.io
        libm 0.2.16 crates.io
        memchr 2.8.3 crates.io
        ml-dsa 0.1.1 crates.io
        module-lattice 0.2.3 crates.io
        num-traits 0.2.19 crates.io
        once_cell 1.21.4 crates.io
        pin-project-lite 0.2.17 crates.io
        ppv-lite86 0.2.21 crates.io
        proc-macro2 1.0.107 crates.io
        quote 1.0.47 crates.io
        r-efi 5.3.0 crates.io
        r-efi 6.0.0 crates.io
        rand 0.10.2 crates.io
        rand 0.9.5 crates.io
        rand_chacha 0.9.0 crates.io
        rand_core 0.10.1 crates.io
        rand_core 0.6.4 crates.io
        rand_core 0.9.5 crates.io
        rayon 1.12.0 crates.io
        rayon-core 1.13.0 crates.io
        rustversion 1.0.23 crates.io
        serde 1.0.229 crates.io
        serde_core 1.0.229 crates.io
        serde_derive 1.0.229 crates.io
        serde_json 1.0.151 crates.io
        sha2 0.10.9 crates.io
        sha3 0.10.9 crates.io
        shake 0.1.0 crates.io
        shlex 2.0.1 crates.io
        signature 2.2.0 crates.io
        signature 3.0.0 crates.io
        slab 0.4.12 crates.io
        sponge-cursor 0.1.0 crates.io
        subtle 2.6.1 crates.io
        syn 2.0.119 crates.io
        syn 3.0.3 crates.io
        thiserror 2.0.19 crates.io
        thiserror-impl 2.0.19 crates.io
        typenum 1.20.1 crates.io
        unicode-ident 1.0.24 crates.io
        version_check 0.9.5 crates.io
        wasi 0.11.1+wasi-snapshot-preview1 crates.io
        wasip2 1.0.4+wasi-0.2.12 crates.io
        wasm-bindgen 0.2.126 crates.io
        wasm-bindgen-macro 0.2.126 crates.io
        wasm-bindgen-macro-support 0.2.126 crates.io
        wasm-bindgen-shared 0.2.126 crates.io
        winter-air 0.13.1 ruta
        winter-crypto 0.13.1 crates.io
        winter-fri 0.13.1 crates.io
        winter-math 0.13.1 crates.io
        winter-rand-utils 0.13.1 crates.io
        winter-utils 0.13.1 crates.io
        winter-verifier 0.13.1 ruta
        wit-bindgen 0.57.1 crates.io
        xmss 0.1.0-pre.0 crates.io
        zerocopy 0.8.55 crates.io
        zerocopy-derive 0.8.55 crates.io
        zeroize 1.9.0 crates.io
        zeroize_derive 1.5.0 crates.io
        zk-ssl-air 0.1.0 ruta
        zk-ssl-guardian 0.1.0 ruta
        zk-ssl-hash 0.1.0 ruta
        zk-ssl-medio 0.1.0 ruta
        zk-ssl-verify 0.4.2 ruta
        zmij 1.0.23 crates.io
    ";

    /// **S465 (RFC-0007 E4b-2): el kit verifica STARK y NO compila al probador.** Desde que
    /// `zk-ssl-air` entra, la propiedad de S243 es esa, y se lee del `Cargo.lock` que el
    /// repositorio versiona.
    ///
    /// §694: hasta aqui prohibia `winter-prover` y `winterfell` por nombre y exigia ver cuatro
    /// paquetes. Ahora la clausura tiene que ser EXACTAMENTE `CLAUSURA_DEL_KIT`, con version y
    /// origen; y la lista, que se edita, no puede llevar al probador, ni la capa, el nodo, el
    /// cable o sus clientes: la propiedad no se edita con ella.
    #[test]
    fn la_clausura_del_kit_no_lleva_el_probador() {
        let lock = include_str!("../../../Cargo.lock");
        let permitida: std::collections::BTreeSet<String> = CLAUSURA_DEL_KIT
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(str::to_string)
            .collect();
        for prohibido in [
            "winter-prover",
            "winterfell",
            "stark-experiment",
            "zk-ssl",
            "zk-ssl-node",
            "zk-ssl-wire",
            "zk-ssl-cli",
            "zk-ssl-sdk",
        ] {
            assert!(
                !permitida.iter().any(|p| p.split(' ').next() == Some(prohibido)),
                "la lista cerrada del kit lleva {prohibido}: la propiedad de S243 no se edita"
            );
        }
        let vista = clausura_del_lock(lock, "zk-ssl-verify");
        let entran: Vec<_> = vista.difference(&permitida).collect();
        let salen: Vec<_> = permitida.difference(&vista).collect();
        assert!(
            entran.is_empty() && salen.is_empty(),
            "la clausura del kit no es la lista cerrada (§694)\n  entran: {entran:?}\n  \
             salen: {salen:?}"
        );

        // Prueba de vida, sobre el lock de verdad: una puerta que no ve lo que entra no mide nada.
        // (1) un crate del arbol que lleva al probador con OTRO nombre, colgado del verificador;
        let bloque = "name = \"winter-verifier\"\nversion = \"0.13.1\"\ndependencies = [\n";
        let con_probador = lock.replacen(bloque, &format!("{bloque} \"stark-experiment\",\n"), 1);
        assert_ne!(con_probador, lock, "el falsador no encontro winter-verifier en el lock");
        let c = clausura_del_lock(&con_probador, "zk-ssl-verify");
        assert!(c.contains("stark-experiment 0.1.0 ruta"));
        assert!(c.contains("winter-prover 0.13.1 ruta"));
        // (2) el fork que deja de entrar por ruta;
        let fork_de_fuera = lock.replacen(
            "name = \"winter-air\"\nversion = \"0.13.1\"\n",
            &format!("name = \"winter-air\"\nversion = \"0.13.1\"\nsource = \"{CRATES_IO}\"\n"),
            1,
        );
        assert_ne!(fork_de_fuera, lock, "el falsador no encontro winter-air en el lock");
        let c = clausura_del_lock(&fork_de_fuera, "zk-ssl-verify");
        assert!(c.contains("winter-air 0.13.1 crates.io"));
        // (3) y una version que se mueve sin que nadie la pida.
        let movida = lock.replacen(
            "name = \"winter-fri\"\nversion = \"0.13.1\"\n",
            "name = \"winter-fri\"\nversion = \"0.13.2\"\n",
            1,
        );
        assert_ne!(movida, lock, "el falsador no encontro winter-fri en el lock");
        let c = clausura_del_lock(&movida, "zk-ssl-verify");
        assert!(c.contains("winter-fri 0.13.2 crates.io"));
    }

    /// **Las fijaciones (§694).** La version EXACTA que el lock tiene que llevar y que cada
    /// manifiesto tiene que pedir con `=`: la familia de winterfell entera -el paraguas y todo
    /// `winter-*`, el fork incluido- a 0.13.1, `xmss` a 0.1.0-pre.0 y `ml-dsa` a 0.1.1. Son la
    /// criptografia del kit: `xmss` y `ml-dsa` declaran ellas mismas que no tienen auditoria
    /// independiente, y el fork no esta auditado (RFC-0009, H7).
    fn fijacion(paquete: &str) -> Option<&'static str> {
        match paquete {
            "winterfell" => Some("0.13.1"),
            p if p.starts_with("winter-") => Some("0.13.1"),
            "xmss" => Some("0.1.0-pre.0"),
            "ml-dsa" => Some("0.1.1"),
            _ => None,
        }
    }

    /// Los tres crates del FORK (RFC-0009 E3a-1, §533): entran por `[patch]`, por ruta.
    const FORK: [&str; 3] = ["winter-air", "winter-prover", "winter-verifier"];

    /// El valor de `campo = "..."` en una linea de TOML: una tabla en linea o una linea de una
    /// tabla. Solo como clave entera: `rust-version` no es `version`.
    fn campo_toml(s: &str, campo: &str) -> Option<String> {
        let mut desde = 0;
        while let Some(i) = s[desde..].find(campo).map(|i| i + desde) {
            let antes = s[..i].chars().next_back();
            let tras = s[i + campo.len()..].trim_start();
            if !antes.is_some_and(|c| c.is_alphanumeric() || c == '-' || c == '_') {
                if let Some(v) = tras.strip_prefix('=').map(str::trim_start) {
                    if let Some(v) = v.strip_prefix('"') {
                        return v.split('"').next().map(str::to_string);
                    }
                }
            }
            desde = i + campo.len();
        }
        None
    }

    /// Una dependencia declarada: (seccion, clave, paquete, requisito).
    type Declarada = (String, String, String, Option<String>);

    /// El contenido de una cadena de TOML al principio de `s`, sin las comillas.
    fn cadena_toml(s: &str) -> Option<String> {
        s.strip_prefix('"').and_then(|r| r.split('"').next()).map(str::to_string)
    }

    /// Cada dependencia que declara un manifiesto. Lee las formas que usa el arbol: `clave =
    /// "req"` y `clave = { version = "req", package = ".." }` bajo una seccion que acaba en
    /// `dependencies` -tambien la de `target.'cfg(..)'` y la de `[workspace]`-, y la tabla
    /// `[dependencies.clave]` del fork, con su `version` y su `package` en lineas propias. Sin
    /// `version`, el requisito es `None`; una clave con punto (`x.workspace = true`) cuenta como
    /// `x`, con requisito solo si es `x.version`: falla cerrada.
    fn dependencias_del_manifiesto(toml: &str) -> Vec<Declarada> {
        type Abierta = (String, String, Option<String>, Option<String>);
        fn cerrar(tabla: Option<Abierta>, fuera: &mut Vec<Declarada>) {
            if let Some((seccion, clave, paquete, req)) = tabla {
                let paquete = paquete.unwrap_or_else(|| clave.clone());
                fuera.push((seccion, clave, paquete, req));
            }
        }
        let mut fuera = Vec::new();
        let mut seccion = String::new();
        // La tabla de una dependencia, abierta: (seccion, clave, paquete, requisito).
        let mut tabla: Option<Abierta> = None;
        for linea in toml.lines() {
            let s = linea.trim();
            if s.is_empty() || s.starts_with('#') {
                continue;
            }
            if s.starts_with('[') && s.ends_with(']') {
                cerrar(tabla.take(), &mut fuera);
                seccion = s[1..s.len() - 1].to_string();
                if let Some((antes, clave)) = seccion.rsplit_once('.') {
                    if antes.ends_with("dependencies") {
                        tabla = Some((antes.to_string(), clave.to_string(), None, None));
                    }
                }
                continue;
            }
            if let Some((_, _, paquete, req)) = tabla.as_mut() {
                if s.starts_with("version") {
                    *req = campo_toml(s, "version");
                } else if s.starts_with("package") {
                    *paquete = campo_toml(s, "package");
                }
            } else if seccion.ends_with("dependencies") {
                let Some((clave, valor)) = s.split_once('=') else { continue };
                let (clave, valor) = (clave.trim().trim_matches('"'), valor.trim());
                let (clave, req, paquete) = match clave.split_once('.') {
                    Some((base, "version")) => (base, cadena_toml(valor), None),
                    Some((base, _)) => (base, None, None),
                    None if valor.starts_with('"') => (clave, cadena_toml(valor), None),
                    None => (clave, campo_toml(valor, "version"), campo_toml(valor, "package")),
                };
                let paquete = paquete.unwrap_or_else(|| clave.to_string());
                fuera.push((seccion.clone(), clave.to_string(), paquete, req));
            }
        }
        cerrar(tabla.take(), &mut fuera);
        fuera
    }

    /// Los miembros del workspace, leidos del `Cargo.toml` raiz como los lee el canon.
    fn miembros_del_workspace(toml: &str) -> Vec<String> {
        let mut dentro = false;
        let mut fuera = Vec::new();
        for linea in toml.lines() {
            let s = linea.trim();
            if s.starts_with('[') {
                dentro = s == "[workspace]";
            } else if dentro && s.starts_with("\"crates/") {
                fuera.push(s.trim_end_matches(',').trim_matches('"').to_string());
            }
        }
        fuera
    }

    /// Los fallos de fijacion del lock y de los manifiestos, cada uno con su fichero. Vacio es
    /// VERDE. En el lock: cada paquete de una familia fijada lleva su version, el fork viene por
    /// ruta y lo demas de crates.io, y algun manifiesto lo clava. En los manifiestos: cada
    /// declaracion de una familia fijada pide `=` y la version, sin excepcion.
    fn fallos_de_fijacion(lock: &str, manifiestos: &[(String, String)]) -> Vec<String> {
        let mut fallos = Vec::new();
        let mut clavados = std::collections::BTreeSet::new();
        for (ruta, toml) in manifiestos {
            for (seccion, clave, paquete, req) in dependencias_del_manifiesto(toml) {
                let Some(v) = fijacion(&paquete) else { continue };
                if req.as_deref() == Some(format!("={v}").as_str()) {
                    clavados.insert(paquete);
                } else {
                    fallos.push(format!(
                        "{ruta}: [{seccion}] {clave} ({paquete}) pide {req:?}; \
                         la fijacion es \"={v}\""
                    ));
                }
            }
        }
        for p in paquetes_del_lock(lock) {
            let Some(v) = fijacion(&p.nombre) else { continue };
            if p.version != v {
                fallos.push(format!("Cargo.lock: {} {}; la fijacion es {v}", p.nombre, p.version));
            }
            let del_fork = FORK.contains(&p.nombre.as_str());
            if del_fork && !p.source.is_empty() {
                fallos.push(format!(
                    "Cargo.lock: {} no entra por el fork del arbol: {}",
                    p.id(),
                    p.source
                ));
            }
            if !del_fork && p.source != CRATES_IO {
                fallos.push(format!("Cargo.lock: {} no viene de crates.io", p.id()));
            }
            if !clavados.contains(&p.nombre) {
                fallos.push(format!(
                    "Cargo.lock: {} esta en el lock y ningun manifiesto lo clava con `=`",
                    p.id()
                ));
            }
        }
        fallos
    }

    /// **§694: las fijaciones se comprueban, no se recuerdan.** Lee el `Cargo.lock` y el
    /// `Cargo.toml` de la raiz y de cada miembro del workspace, y exige lo que dice `fijacion`.
    /// Hasta aqui los `=` se sostenian por convencion: sobre el arbol de antes, este test da 23
    /// declaraciones sin `=` (`winterfell` en seis crates y diecisiete en los manifiestos del
    /// fork) y cinco paquetes del lock que no clavaba nadie -`winter-fri`, `winter-utils`,
    /// `winter-maybe-async`, `winter-rand-utils` y `winterfell`-: el dia que hubiera una 0.13.2,
    /// un `cargo update` los moveria sin que ninguna puerta lo viera.
    #[test]
    fn las_fijaciones_son_exactas() {
        let raiz = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
        let toml_raiz = include_str!("../../../Cargo.toml");
        let miembros = miembros_del_workspace(toml_raiz);
        assert!(
            miembros.iter().any(|m| m == "crates/zk-ssl-verify")
                && FORK.iter().all(|f| miembros.contains(&format!("crates/{f}"))),
            "prueba de vida: el lector de miembros no ve el kit o el fork: {miembros:?}"
        );
        let mut manifiestos = vec![("Cargo.toml".to_string(), toml_raiz.to_string())];
        for m in &miembros {
            let ruta = format!("{m}/Cargo.toml");
            let texto = std::fs::read_to_string(format!("{raiz}/{ruta}"))
                .unwrap_or_else(|e| panic!("{ruta}: {e}"));
            manifiestos.push((ruta, texto));
        }
        let lock = include_str!("../../../Cargo.lock");
        let fallos = fallos_de_fijacion(lock, &manifiestos);
        assert!(fallos.is_empty(), "fijaciones rotas (§694):\n{}", fallos.join("\n"));

        // Prueba de vida: la puerta ve cada familia en el lock, y los dos que nadie clavaba.
        let fijados: std::collections::BTreeSet<String> = paquetes_del_lock(lock)
            .into_iter()
            .filter(|p| fijacion(&p.nombre).is_some())
            .map(|p| p.nombre)
            .collect();
        for vivo in ["winter-fri", "winter-utils", "winterfell", "xmss", "ml-dsa"] {
            assert!(fijados.contains(vivo), "prueba de vida: el lock no tiene {vivo}");
        }

        // Y los falsadores, sobre los ficheros de verdad: cada uno tiene que salir con su nombre.
        let ve = |lock: &str, manifiestos: &[(String, String)], que: &str| {
            let f = fallos_de_fijacion(lock, manifiestos);
            assert!(f.iter().any(|l| l.contains(que)), "la puerta no ve {que}: {f:?}");
        };
        let mutado = |ruta: &str, de: &str, a: &str| {
            let mut m = manifiestos.clone();
            let t = &mut m.iter_mut().find(|(r, _)| r == ruta).expect(ruta).1;
            let nuevo = t.replacen(de, a, 1);
            assert!(nuevo != *t, "el falsador no encontro `{de}` en {ruta}");
            *t = nuevo;
            m
        };
        // (1) el lock mueve `winter-fri`;
        let movido = lock.replacen(
            "name = \"winter-fri\"\nversion = \"0.13.1\"\n",
            "name = \"winter-fri\"\nversion = \"0.13.2\"\n",
            1,
        );
        assert_ne!(movido, lock);
        ve(&movido, &manifiestos, "winter-fri 0.13.2");
        // (2) el fork deja de entrar por ruta;
        let bloque = "name = \"winter-verifier\"\nversion = \"0.13.1\"\n";
        let de_fuera = lock.replacen(bloque, &format!("{bloque}source = \"{CRATES_IO}\"\n"), 1);
        assert_ne!(de_fuera, lock);
        ve(&de_fuera, &manifiestos, "no entra por el fork");
        // (3) un `winter-*` nuevo en el lock, que no clava nadie;
        let nuevo = format!(
            "{lock}\n[[package]]\nname = \"winter-nuevo\"\nversion = \"0.13.1\"\nsource = \"{CRATES_IO}\"\n"
        );
        ve(&nuevo, &manifiestos, "winter-nuevo 0.13.1 crates.io esta en el lock y ningun");
        // (4) un manifiesto que vuelve al caret: en tabla, en linea y con `package`;
        let m = mutado(
            "crates/winter-air/Cargo.toml",
            "[dependencies.fri]\nversion = \"=0.13.1\"",
            "[dependencies.fri]\nversion = \"0.13\"",
        );
        ve(lock, &m, "winter-air/Cargo.toml: [dependencies] fri (winter-fri) pide Some(\"0.13\")");
        let m = mutado("crates/zk-ssl/Cargo.toml", "winterfell = \"=0.13.1\"", "winterfell = \"0.13\"");
        ve(lock, &m, "crates/zk-ssl/Cargo.toml: [dependencies] winterfell");
        let m = mutado(
            "crates/zk-ssl-verify/Cargo.toml",
            "\n[dev-dependencies]\n",
            "\n[dev-dependencies]\nutils = { package = \"winter-utils\", version = \"0.13\" }\n",
        );
        ve(lock, &m, "[dev-dependencies] utils (winter-utils)");
        // (5) y las otras dos familias: `xmss` sin `=`, y `ml-dsa` heredado sin version.
        let m = mutado(
            "crates/zk-ssl-verify/Cargo.toml",
            "xmss = \"=0.1.0-pre.0\"",
            "xmss = \"0.1.0-pre.0\"",
        );
        ve(lock, &m, "crates/zk-ssl-verify/Cargo.toml: [dependencies] xmss");
        let m = mutado(
            "crates/zk-ssl-medio/Cargo.toml",
            "ml-dsa = { version = \"=0.1.1\", default-features = false }",
            "ml-dsa.workspace = true",
        );
        ve(lock, &m, "(ml-dsa) pide None");
    }

    /// **§633 (RFC-0013 E4a): el kit VERIFICA la nota del medio y no firma ninguna.** La
    /// dependencia normal de `zk-ssl-medio` va sin sus features por defecto, que traen
    /// `firmar` (la sal del sistema, el borrado de la semilla y el firmado de `ml-dsa`); la
    /// dev-dependency la lleva, y no entra en el binario. Si alguien quita la linea, este
    /// test lo dice antes que el binario.
    #[test]
    fn el_kit_verifica_el_medio_sin_firmar() {
        let toml = include_str!("../Cargo.toml");
        let mut seccion = "";
        let mut vista = false;
        for linea in toml.lines() {
            let s = linea.trim();
            if s.starts_with('[') {
                seccion = s;
            } else if s.starts_with("zk-ssl-medio") {
                if seccion == "[dependencies]" {
                    assert!(
                        s.contains("default-features = false") && !s.contains("firmar"),
                        "el kit depende del medio CON firmar: {s}"
                    );
                    vista = true;
                }
            }
        }
        assert!(vista, "prueba de vida: el kit no depende del medio");
    }

    #[test]
    fn una_path_dep_a_la_capa_no_se_le_escapa_al_parser() {
        let mentira = concat!(
            "[dependencies]\n",
            "zk-ssl-hash = { path = \"../zk-ssl-hash\" }\n",
            "zk-ssl = { path = \"../zk-ssl\" }\n"
        );
        let d = deps_por_ruta_del_manifiesto(mentira);
        assert!(
            d.contains(&("dependencies".to_string(), "zk-ssl".to_string())),
            "el parser no ve una path-dep a la capa: seria una puerta ciega"
        );
    }

    #[test]
    fn el_parser_ve_tambien_las_dev_dependencies() {
        let mentira = concat!(
            "[dev-dependencies]\n",
            "zk-ssl-node = { path = \"../zk-ssl-node\" }\n"
        );
        let d = deps_por_ruta_del_manifiesto(mentira);
        assert_eq!(
            d.len(),
            1,
            "una dependencia por ruta en dev tiene que verse: el gate afirma las DOS secciones"
        );
    }

    // ── §406 · RFC-0005 E2: EL CONJUNTO DE VERSIONES TIENE UN SOLO PRODUCTOR ──

    /// Subir `VERSION_FORMATO` sin anadir la variante se pone rojo aqui: la
    /// puerta por la que el nucleo crece es una sola.
    #[test]
    fn la_version_vigente_es_miembro_del_conjunto() {
        let vf = crate::VERSION_FORMATO;
        assert!(
            crate::VersionCabeza::TODAS.iter().any(|v| v.as_u8() == vf),
            "VERSION_FORMATO {vf} no esta en {:?}",
            crate::VersionCabeza::TODAS
        );
        assert_eq!(
            crate::VersionCabeza::try_from(u64::from(vf)).map(|v| v.as_u8()),
            Ok(vf)
        );
    }

    #[test]
    fn el_conjunto_es_exactamente_v2_v3_v4_v5_y_v6_y_su_texto_se_deriva() {
        assert_eq!(crate::VersionCabeza::TODAS.map(|v| v.as_u8()), [2, 3, 4, 5, 6]);
        assert_eq!(crate::VersionCabeza::texto(), "v2, v3, v4, v5 o v6");
    }

    /// RFC-0006 E2a (§414): la pareja del MMR la llevan v3 y v4 -y desde §451 v5, y
    /// desde §558 v6-, y el texto que la extension y el canal de la historia citan se
    /// DERIVA de aqui.
    #[test]
    fn la_pareja_del_mmr_la_llevan_v3_v4_v5_y_v6() {
        assert!(!crate::VersionCabeza::V2.lleva_mmr());
        assert!(crate::VersionCabeza::V3.lleva_mmr());
        assert!(crate::VersionCabeza::V4.lleva_mmr());
        assert!(crate::VersionCabeza::V5.lleva_mmr());
        assert!(crate::VersionCabeza::V6.lleva_mmr());
        assert_eq!(crate::VersionCabeza::texto_con_mmr(), "v3, v4, v5 o v6");
    }

    /// RFC-0007 E1a (§451): la pareja de consumos la llevan v4 y v5 -y desde §558 v6-,
    /// y el texto que el sobre de consumo y el de conflicto citan se DERIVA de aqui.
    #[test]
    fn la_pareja_de_consumos_la_llevan_v4_v5_y_v6() {
        assert!(!crate::VersionCabeza::V2.lleva_consumos());
        assert!(!crate::VersionCabeza::V3.lleva_consumos());
        assert!(crate::VersionCabeza::V4.lleva_consumos());
        assert!(crate::VersionCabeza::V5.lleva_consumos());
        assert!(crate::VersionCabeza::V6.lleva_consumos());
        assert_eq!(crate::VersionCabeza::texto_con_consumos(), "v4, v5 o v6");
    }

    /// RFC-0010 E2 (§558): la pareja de recepcion la lleva SOLO la v6, y la familia del
    /// estado comprometido la llevan v5 y v6. Los dos textos se DERIVAN del conjunto.
    #[test]
    fn la_pareja_de_recepcion_la_lleva_v6_y_la_familia_la_llevan_v5_y_v6() {
        assert!(!crate::VersionCabeza::V5.lleva_recepcion());
        assert!(crate::VersionCabeza::V6.lleva_recepcion());
        assert_eq!(crate::VersionCabeza::texto_con_recepcion(), "v6");
        assert!(!crate::VersionCabeza::V4.lleva_parametros());
        assert!(crate::VersionCabeza::V5.lleva_parametros());
        assert!(crate::VersionCabeza::V6.lleva_parametros());
        assert_eq!(crate::VersionCabeza::texto_con_parametros(), "v5 o v6");
    }

    /// Cinco valores fuera del conjunto, el `0x103` entre ellos: el valor viaja
    /// entero en el error y el texto nombra el conjunto.
    #[test]
    fn una_version_fuera_del_conjunto_se_rechaza_sin_truncar() {
        let fuera = u64::from(crate::VersionCabeza::TODAS.last().expect("no vacio").as_u8()) + 1;
        for v in [0u64, 1, fuera, 0x103, u64::MAX] {
            let e = crate::VersionCabeza::try_from(v).expect_err("fuera del conjunto");
            assert_eq!(e.0, v, "el valor tiene que viajar entero");
            let t = e.to_string();
            assert!(
                t.contains(&format!("formatVersion {v}"))
                    && t.contains(&crate::VersionCabeza::texto()),
                "{t}"
            );
        }
    }
}
