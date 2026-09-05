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
/// entra en la firma.
pub const VERSION_FORMATO: u8 = 3;

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
}

impl VersionCabeza {
    /// Todas, en orden: el conjunto que se enumera y del que se deriva el texto.
    pub const TODAS: [VersionCabeza; 2] = [VersionCabeza::V2, VersionCabeza::V3];
    /// El byte que entra en el preambulo.
    pub fn as_u8(self) -> u8 {
        self as u8
    }
    /// `v2 o v3`, DERIVADO de [`Self::TODAS`]: el texto que los rechazos citan.
    pub fn texto() -> String {
        let vs: Vec<String> = Self::TODAS.iter().map(|v| format!("v{}", v.as_u8())).collect();
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
            ("dev-dependencies", "zk-ssl-guardian"),
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
    fn el_conjunto_es_exactamente_v2_y_v3_y_su_texto_se_deriva() {
        assert_eq!(crate::VersionCabeza::TODAS.map(|v| v.as_u8()), [2, 3]);
        assert_eq!(crate::VersionCabeza::texto(), "v2 o v3");
    }

    /// Cinco valores fuera del conjunto, el `0x103` entre ellos: el valor viaja
    /// entero en el error y el texto nombra el conjunto.
    #[test]
    fn una_version_fuera_del_conjunto_se_rechaza_sin_truncar() {
        for v in [0u64, 1, 4, 0x103, u64::MAX] {
            let e = crate::VersionCabeza::try_from(v).expect_err("fuera del conjunto");
            assert_eq!(e.0, v, "el valor tiene que viajar entero");
            let t = e.to_string();
            assert!(t.contains(&format!("formatVersion {v}")) && t.contains("v2 o v3"), "{t}");
        }
    }
}
