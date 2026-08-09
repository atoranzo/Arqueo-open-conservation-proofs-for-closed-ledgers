//! # El firmante de cabezas de época
//!
//! Eslabón 3 de la cadena de la oponibilidad (`BACKLOG.md`). Firma
//! `EpochHead` con XMSS y **da por fin consumidor al guardián de §234**.
//!
//! ## Qué se firma, y por qué no el digest a secas
//!
//! ```text
//! preámbulo = b"ZK-SSL-epoch-head" ‖ versión_de_formato ‖ epoch_digest
//! firma     = XMSS(preámbulo)
//! ```
//!
//! ⚠️ **La versión de formato existe porque la cabeza está incompleta y lo
//! sabemos.** `EpochHead` tiene dos extensiones pendientes —`verifier_hash`
//! (§104.3) y la raíz de recepción (§121)— y §121.2 avisaba: *«antes de que
//! exista un solo testigo»*. **Firmar crea el primer testigo.**
//!
//! Sin este byte, cuando entre la raíz de recepción el `epoch_digest`
//! cambiaría y un testigo con firmas guardadas **no podría distinguir una
//! cabeza vieja legítima de una falsificación**. Con él, la transición es
//! legible en vez de silenciosa. El aviso de §121.2 queda **atendido, no
//! ignorado**.
//!
//! ⚠️ Y no es lo mismo que un campo vacío. `verifier_hash` vacío **mentiría**
//! —diría que ata las reglas sin atarlas—; una versión de formato **dice la
//! verdad**: «esta cabeza tiene estos cinco campos». Es el criterio que mató
//! a `verifier_hash`, aplicado en la dirección contraria.
//!
//! ⚠️ **No toca `epoch_digest`.** Ese valor está congelado en los vectores de
//! conformidad de `zkssl/0.2`, y la versión vive en el **preámbulo de la
//! firma**, no en la cabeza. La conformidad no se mueve.
//!
//! ## El dominio, y por qué NO lleva la versión dentro
//!
//! Sin dominio, una firma de cabeza podría reinterpretarse como firma de
//! otra cosa el día que el operador firme algo más (§209, `digest_of_proof`).
//!
//! Pero el dominio va **sin** `-v1`: dos marcadores de versión que pueden
//! discrepar valen menos que uno. Si alguien sube el byte y no la cadena, el
//! preámbulo queda inconsistente y nadie lo nota. **Dominio fijo, versión que
//! avanza.**
//!
//! ## Dos ejes de versión, y cuál manda sobre qué
//!
//! | | gobierna |
//! |---|---|
//! | `zkssl/0.2` | **el cable**: qué viaja y qué significa |
//! | [`VERSION_FORMATO`] | **qué campos entran en la firma** |
//!
//! Son ejes distintos y **avanzan por separado**. Se dice aquí porque, sin
//! decirlo, la próxima persona pensará que uno implica al otro.
//!
//! ## El orden, que es lo único que hace segura la firma
//!
//! [`FirmanteCabeza::firmar`] **reserva el índice en el guardián —con
//! `fsync`— ANTES de firmar**. Si el proceso muere en medio, queda un índice
//! quemado sin firma: el caso seguro, y el normal —K.1 midió **13 de 25**.
//! Lo contrario —firmar y morir antes de persistir— **filtra la clave**.
//!
//! ## ⚠️ Lo que esta pieza NO da
//!
//! - **No prueba que la cabeza sea completa.** Prueba que el operador la
//!   emitió. Un testigo que guarde firmas v1 está guardando **cabezas
//!   incompletas por diseño, no por descuido**, y debe saberlo.
//! - **No hay custodia de clave.** El operador no tiene clave hoy, y *una
//!   firma sin custodia declarada de la clave es una firma sin valor
//!   probatorio*. Esta pieza toma una semilla; **de dónde sale y quién la
//!   guarda es decisión de despliegue**, y no está tomada.
//! - **No hay latido.** Nada emite cabezas periódicamente. Esto firma cuando
//!   se le pide.
//! - `xmss` es **`0.1.0-pre.0`, sin auditoría independiente**, declarado por
//!   el propio crate.

use crate::firma_indice::{GuardianIndice, GuardianError, Reconciliacion};
use std::path::Path;
use xmss::{KeyPair, Signature, VerifyingKey, XmssMtSha2_40_8_256};

/// El conjunto elegido (entrada 53, §127.1): 2⁴⁰ firmas, ~35.000 años a una
/// por segundo. Medido en S.3 sobre esta máquina: keygen 17,9 ms, firmar
/// 144,5 ms, verificar 2,4 ms.
pub type Conjunto = XmssMtSha2_40_8_256;

/// Separación de dominio. **Sin versión dentro**: ver la cabecera.
pub const DOMINIO: &[u8] = b"ZK-SSL-epoch-head";

/// Versión del formato de cabeza que entra en la firma. Sube cuando cambian
/// **los campos de `EpochHead`**, no cuando cambia el cable.
pub const VERSION_FORMATO: u8 = 1;

/// Bytes del OID al principio del SK, en el formato de referencia.
const OID_BYTES: usize = 4;
/// Longitud del hash del conjunto `_256`.
const N: usize = 32;

#[derive(Debug)]
pub enum FirmaError {
    Guardian(GuardianError),
    /// El crate `xmss` rechazó la operación. Incluye `KeyExhausted`.
    Xmss(String),
    /// El SK no tiene la forma esperada: la serialización de upstream cambió.
    LayoutInesperado { sk_len: usize, esperado: usize },
    /// ⚠️ **La firma es válida, pero de OTRA cosa.** `verify()` devuelve el
    /// mensaje que llevaba dentro, no un booleano: que verifique dice «esta
    /// firma vale para el mensaje que contiene», **no** «para el que tú
    /// esperas». Sin comparar, un atacante presenta la firma legítima de
    /// otra cabeza y pasa.
    PreambuloDistinto { esperado: usize, recibido: usize },
}

impl std::fmt::Display for FirmaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FirmaError::Guardian(e) => write!(f, "firmante: {e}"),
            FirmaError::Xmss(e) => write!(f, "firmante: xmss rechazó: {e}"),
            FirmaError::PreambuloDistinto { esperado, recibido } => write!(
                f,
                "firmante: la firma es VALIDA pero de otro mensaje \
                 (preambulo esperado {esperado} bytes, recibido {recibido}). \
                 Verificar sin comparar no prueba nada: `verify()` acredita lo \
                 que la firma lleva dentro, no lo que se busca."
            ),
            FirmaError::LayoutInesperado { sk_len, esperado } => write!(
                f,
                "firmante: el SK mide {sk_len} bytes y se esperaban {esperado}. \
                 La serialización de `xmss` ha cambiado: el índice ya no está \
                 donde se midió, y leerlo daría un valor falso."
            ),
        }
    }
}

/// ⚠️ **Tercer tipo de error de este nodo al que le faltaba un trait
/// estandar**: `RpcError` no derivaba `Debug` (§228), `GuardianIndice`
/// tampoco (§234), y éste no implementaba `Error` — así que `?` no
/// convertía a `anyhow` y había que rodearlo con `map_err`.
///
/// La regla que lo evita: **un tipo de error lleva `Debug`, `Display` y
/// `Error` desde que nace.** Sin los tres, cada consumidor se inventa un
/// rodeo distinto — y en §241 el rodeo estaba en `main.rs` y el `?` en
/// `latido.rs`, **incoherentes dentro del mismo sello**.
impl std::error::Error for FirmaError {}

impl From<GuardianError> for FirmaError {
    fn from(e: GuardianError) -> Self {
        FirmaError::Guardian(e)
    }
}

/// Lo que produce una firma de cabeza.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CabezaFirmada {
    /// Qué campos de `EpochHead` entraron en la firma.
    pub version_formato: u8,
    /// **Cuántas firmas se han hecho con esta clave**, contando ésta. Coincide
    /// con el índice interno de la clave tras firmar. Un testigo que vea dos
    /// cabezas distintas con el mismo valor ha visto **reúso de índice**.
    pub indice: u64,
    /// La firma, con el preámbulo adjunto dentro.
    ///
    /// ⚠️ Se usa la firma **adjunta** y no la desprendida a propósito: el
    /// testigo puede extraer lo que se firmó sin reconstruirlo, así que la
    /// firma es autocontenida. Cuesta 50 bytes sobre 18.469 —el 0.3 %—.
    pub firma: Vec<u8>,
}

/// El preámbulo exacto que se firma. **Es superficie de conformidad**: una
/// segunda implementación tiene que producir estos bytes.
pub fn preambulo(version: u8, epoch_digest: &[u8; 32]) -> Vec<u8> {
    let mut v = Vec::with_capacity(DOMINIO.len() + 1 + 32);
    v.extend_from_slice(DOMINIO);
    v.push(version);
    v.extend_from_slice(epoch_digest);
    v
}

/// Lee el índice del SK: bytes `[4, 9)` en **big-endian**, cinco bytes para
/// `h = 40`.
///
/// ⚠️ **El offset y el ancho están MEDIDOS** (S.3), no deducidos: el SK mide
/// **137 bytes** = OID(4) + índice(5) + 4×32, y al firmar cambia el byte 8 —el
/// menos significativo de un entero big-endian que ocupa [4, 9)—.
///
/// ⚠️ La evaluación registró «SK = 136 B, índice de 4 bytes»: eso es del
/// conjunto de **árbol único**. Para el elegido son **137 y 5** (§236).
pub fn indice_de_sk(sk: &[u8]) -> Result<u64, FirmaError> {
    let esperado = OID_BYTES + ancho_indice() + 4 * N;
    if sk.len() != esperado {
        return Err(FirmaError::LayoutInesperado { sk_len: sk.len(), esperado });
    }
    let mut v = 0u64;
    for b in &sk[OID_BYTES..OID_BYTES + ancho_indice()] {
        v = (v << 8) | *b as u64;
    }
    Ok(v)
}

/// Ancho del índice en bytes: ⌈h/8⌉ = 5 para `h = 40`.
const fn ancho_indice() -> usize {
    5
}

/// Firma cabezas, con el guardián del índice delante.
pub struct FirmanteCabeza {
    par: KeyPair<Conjunto>,
    guardian: GuardianIndice,
}

impl FirmanteCabeza {
    /// ⚠️ **La semilla es material de clave.** De dónde sale y quién la
    /// guarda es **decisión de despliegue**, y no está tomada. Esta función
    /// no la genera ni la persiste.
    pub fn desde_semilla(
        semilla: &[u8],
        ruta_contador: impl AsRef<Path>,
    ) -> Result<Self, FirmaError> {
        let guardian = GuardianIndice::abrir(ruta_contador)?;
        let par = KeyPair::<Conjunto>::from_seed(semilla)
            .map_err(|e| FirmaError::Xmss(format!("{e:?}")))?;
        let mut f = FirmanteCabeza { par, guardian };
        // Se comprueba el layout AL ABRIR, no al firmar: si upstream cambió
        // la serialización, es mejor no arrancar que firmar y anotar mal.
        let _ = f.indice_de_la_clave()?;
        Ok(f)
    }

    /// **Reserva el índice y luego firma. Ese orden es la pieza.**
    pub fn firmar(&mut self, epoch_digest: &[u8; 32]) -> Result<CabezaFirmada, FirmaError> {
        // ── 1 · persistir con fsync ANTES de firmar ──
        let indice = self.guardian.reservar()?;
        // ── 2 · y solo entonces gastar el índice de la clave ──
        let pre = preambulo(VERSION_FORMATO, epoch_digest);
        let sig = self
            .par
            .signing_key()
            .sign(&pre)
            .map_err(|e| FirmaError::Xmss(format!("{e:?}")))?;
        // ── 3 · ⚠️ verificar la propia salida ANTES de devolverla ──
        // No se emite una firma que no verifica. Cuesta 2,4 ms sobre 144,5
        // —el 1,7 %— y cierra la clase de fallo en que se publica una firma
        // invalida y nadie lo nota hasta que un testigo la rechaza.
        let esperado = pre.clone();
        match self.par.verifying_key().verify(&sig) {
            Ok(m) if m == esperado => {}
            Ok(_) => return Err(FirmaError::Xmss(
                "la firma verifica pero devuelve otro mensaje".into())),
            Err(e) => return Err(FirmaError::Xmss(
                format!("la firma recien hecha NO verifica: {e:?}"))),
        }
        Ok(CabezaFirmada {
            version_formato: VERSION_FORMATO,
            indice,
            firma: sig.as_ref().to_vec(),
        })
    }

    /// El índice que la clave dice tener, leído de su SK.
    pub fn indice_de_la_clave(&mut self) -> Result<u64, FirmaError> {
        indice_de_sk(self.par.signing_key().as_ref())
    }

    /// Compara el contador con el índice real de la clave.
    ///
    /// ⚠️ [`Reconciliacion::ContadorAdelantado`] es **el caso normal tras una
    /// caída**, no la excepción: K.1 lo midió en 13 de 25.
    pub fn reconciliar(&mut self) -> Result<Reconciliacion, FirmaError> {
        let de_la_clave = self.indice_de_la_clave()?;
        Ok(self.guardian.reconciliar(de_la_clave))
    }

    /// Cuántas firmas ha registrado el guardián.
    pub fn indice_del_guardian(&self) -> u64 {
        self.guardian.actual()
    }

    /// La clave pública **en bytes del formato RFC 8391**, para publicarla.
    ///
    /// ⚠️ Sale tal cual, con su OID `0x00000005`. **El apaño de
    /// [`OFFSET_MT_UPSTREAM`] NO se aplica aquí**: lo que se publica es
    /// correcto según el RFC, y el rodeo vive solo en la lectura.
    pub fn clave_publica(&self) -> Vec<u8> {
        self.par.verifying_key().as_ref().to_vec()
    }
}

/// ⚠️ **APAÑO SOBRE UN FALLO DE `xmss 0.1.0-pre.0`** (§240, sondas S.5/S.6).
///
/// Una clave pública XMSS^MT **no se puede releer de sus propios bytes**.
/// El mecanismo, leído del fuente:
///
/// ```text
/// // xmss.rs
/// let oid = XmssOid::try_from(raw).or_else(|_| XmssOid::from_xmssmt_raw_oid(raw))?;
///
/// // params.rs:1031 — hace justo lo que hace falta...
/// fn from_xmssmt_raw_oid(oid: u32) { Self::try_from(oid + XMSSMT_OID_OFFSET) }
/// ```
///
/// El RFC 8391 tiene **dos registros de OID separados** —XMSS y XMSS^MT— y
/// **los dos empiezan en 1**. `XMSSMT-SHA2_40/8_256` es el 5, y
/// `try_from(5)` **acierta** porque 5 también es un OID válido de árbol
/// único (`XmssSha2_16_512`). El `or_else` **nunca corre**.
///
/// ⚠️ **Cinco de los ocho OID multiárbol de SHA2-256 colisionan** con OID
/// válidos de árbol único. Todas esas claves son irrecuperables.
///
/// El apaño: sumar el offset **antes de parsear**. Medido en S.6: con
/// `0x00010005` la clave vuelve **y la firma verifica**.
///
/// ⚠️ **Se aplica SOLO al leer.** Ver [`FirmanteCabeza::clave_publica`].
///
/// ⚠️ Y hay un test que **comprobará el día que sobre**:
/// `el_apano_del_oid_sigue_haciendo_falta`. Si upstream lo arregla, ese
/// test se pone rojo y avisa de que esto hay que quitarlo — un apaño que
/// no sabe cuándo estorba se queda para siempre y **enmascara el cambio de
/// formato que venga después**.
///
/// Las otras dos vías **no existen**: `pkcs8` es un módulo privado y
/// `xmssmt_core_sign_open` no está reexportado (S.6).
pub const OFFSET_MT_UPSTREAM: u32 = 0x0001_0000;

/// Lee una clave pública publicada, aplicando el apaño del OID.
fn clave_desde_bytes(rfc: &[u8]) -> Result<VerifyingKey<Conjunto>, FirmaError> {
    if rfc.len() < 4 {
        return Err(FirmaError::Xmss(format!(
            "clave publica de {} bytes: no caben ni los 4 del OID",
            rfc.len()
        )));
    }
    let mut b = rfc.to_vec();
    let raw = u32::from_be_bytes([b[0], b[1], b[2], b[3]]) | OFFSET_MT_UPSTREAM;
    b[..4].copy_from_slice(&raw.to_be_bytes());
    VerifyingKey::<Conjunto>::try_from(b.as_slice())
        .map_err(|e| FirmaError::Xmss(format!("clave publica ilegible: {e:?}")))
}

/// **La función del testigo.** Verifica una cabeza firmada **sin la clave
/// privada, sin el guardián y sin este proceso**: solo con lo publicado.
///
/// ⚠️ **Verificar con éxito NO basta, y esta función es la razón.**
/// `verify()` devuelve **el mensaje que la firma lleva dentro**, no un
/// booleano. Un atacante puede presentar la firma **legítima de otra
/// cabeza** y pasaría el `verify()` a secas. Lo que cierra esa puerta es
/// **comparar el mensaje recuperado con el preámbulo esperado**.
///
/// ⚠️ Y no lo cierra el parseo: `Signature::try_from` **no valida OID ni
/// longitud** —las firmas adjuntas son de longitud variable—, así que es
/// casi un envoltorio. **Toda la validación real ocurre en `verify()` y en
/// la comparación de abajo.**
///
/// ## Por qué los bytes del RFC y no `postcard`
///
/// Medido en S.4: **18.475 B** frente a 18.478 por `postcard` y **36.952
/// por `serde_json`**. Gana en tamaño **y en interoperabilidad**.
pub fn verificar_cabeza(
    clave_publica: &[u8],
    epoch_digest: &[u8; 32],
    c: &CabezaFirmada,
) -> Result<(), FirmaError> {
    let vk = clave_desde_bytes(clave_publica)?;
    let sig = Signature::<Conjunto>::try_from(c.firma.as_slice())
        .map_err(|e| FirmaError::Xmss(format!("firma ilegible: {e:?}")))?;
    let recuperado = vk
        .verify(&sig)
        .map_err(|e| FirmaError::Xmss(format!("la firma no verifica: {e:?}")))?;
    // ⚠️ EL PASO QUE NO SE PUEDE SALTAR.
    let esperado = preambulo(c.version_formato, epoch_digest);
    if recuperado != esperado {
        return Err(FirmaError::PreambuloDistinto {
            esperado: esperado.len(),
            recibido: recuperado.len(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn semilla() -> [u8; 96] {
        let mut s = [0u8; 96];
        for (i, b) in s.iter_mut().enumerate() {
            *b = (i as u8).wrapping_mul(7).wrapping_add(3);
        }
        s
    }

    /// ⚠️ En disco de verdad: el guardián se niega a operar en `tmpfs`, y
    /// `std::env::temp_dir()` suele serlo.
    fn en_disco(nombre: &str) -> std::path::PathBuf {
        let d = std::path::Path::new("target").join(format!("firmante_{nombre}"));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).expect("crear");
        d.join("indice.bin")
    }

    // ── el preambulo: superficie de conformidad, sin clave ──

    #[test]
    fn el_preambulo_lleva_dominio_y_version_en_ese_orden() {
        let d = [0xABu8; 32];
        let p = preambulo(1, &d);
        assert_eq!(p.len(), DOMINIO.len() + 1 + 32);
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

    // ── el layout del SK: MEDIDO, y probado sin gastar 37 s ──

    #[test]
    fn el_lector_del_indice_maneja_el_acarreo() {
        // ⚠️ EL TEST DE LAYOUT, mitad sintetica. Firmar 256 veces contra la
        // clave real costaria **37 s medidos** (256 x 144,5 ms) sobre los 4 s
        // que tarda hoy el nodo entero. El acarreo es una propiedad del
        // LECTOR, y aqui se prueba exhaustivamente y en microsegundos.
        let largo = OID_BYTES + ancho_indice() + 4 * N;
        let mut sk = vec![0u8; largo];
        for (bytes, esperado) in [
            ([0, 0, 0, 0, 1u8], 1u64),
            ([0, 0, 0, 1, 0], 256),
            ([0, 0, 1, 0, 0], 65_536),
            ([0, 1, 0, 0, 0], 16_777_216),
            ([1, 0, 0, 0, 0], 4_294_967_296),
            ([0xff, 0xff, 0xff, 0xff, 0xff], (1u64 << 40) - 1),
        ] {
            sk[OID_BYTES..OID_BYTES + 5].copy_from_slice(&bytes);
            assert_eq!(indice_de_sk(&sk).expect("leer"), esperado, "bytes {bytes:02x?}");
        }
        // El maximo cuadra con 2^40: el horizonte del conjunto elegido.
        assert_eq!((1u64 << 40) - 1, 1_099_511_627_775);
    }

    #[test]
    fn un_sk_de_otro_tamano_se_rechaza_en_vez_de_leerse() {
        // ⚠️ La otra mitad del test de layout: si upstream cambia la
        // serializacion, **falla aqui y no en produccion**.
        match indice_de_sk(&[0u8; 136]) {
            Err(FirmaError::LayoutInesperado { sk_len, esperado }) => {
                assert_eq!(sk_len, 136);
                assert_eq!(esperado, 137, "OID(4) + indice(5) + 4x32 = 137");
            }
            otro => panic!("un SK de 136 bytes debe rechazarse, y dio: {otro:?}"),
        }
    }

    // ── contra la clave real ──

    #[test]
    fn el_sk_real_mide_137_y_el_indice_esta_donde_se_midio() {
        // ⚠️ Contra la clave DE VERDAD: confirma el offset y el ancho que S.3
        // midio, y que la evaluacion habia registrado con las cifras del
        // conjunto de arbol unico (136 y 4).
        let p = en_disco("layout_real");
        let mut f = FirmanteCabeza::desde_semilla(&semilla(), &p).expect("abrir");
        assert_eq!(f.indice_de_la_clave().expect("indice"), 0, "una clave nueva empieza en 0");
        f.firmar(&[9u8; 32]).expect("firmar");
        assert_eq!(f.indice_de_la_clave().expect("indice"), 1, "tras una firma, 1");
    }

    #[test]
    fn firma_y_verifica_lo_que_dice_firmar() {
        let p = en_disco("verifica");
        let mut f = FirmanteCabeza::desde_semilla(&semilla(), &p).expect("abrir");
        let d = [0x5Au8; 32];
        let c = f.firmar(&d).expect("firmar");
        assert_eq!(c.version_formato, VERSION_FORMATO);
        assert_eq!(c.indice, 1);

        // ⚠️ `firmar` ya verifico su propia salida contra el preambulo antes
        // de devolverla: si hubiera firmado otra cosa, habria dado error.
        //
        // ⚠️ RELEER una firma desde sus bytes —lo que necesita un TESTIGO— no
        // se prueba aqui porque **no se sondeo esa parte de la API**. Va
        // declarado en la cola, no fingido con un `try_from` supuesto.
        assert_eq!(c.firma.len(), 18_469 + preambulo(VERSION_FORMATO, &d).len(),
                   "firma adjunta = RFC + preambulo");
    }

    // ── el TESTIGO ──

    #[test]
    fn un_testigo_verifica_con_la_clave_publica_y_los_bytes() {
        let p = en_disco("testigo");
        let mut f = FirmanteCabeza::desde_semilla(&semilla(), &p).expect("abrir");
        let d = [0x77u8; 32];
        let c = f.firmar(&d).expect("firmar");
        let pk = f.clave_publica();
        // ⚠️ Sin la clave privada, sin el guardian, sin el firmante.
        verificar_cabeza(&pk, &d, &c).expect("un testigo debe poder verificar");
    }

    #[test]
    fn una_firma_valida_de_otra_cabeza_se_rechaza() {
        // ⚠️⚠️ EL TEST QUE JUSTIFICA LA FUNCION. `verify()` devuelve el
        // MENSAJE, no un booleano: esta firma es PERFECTAMENTE VALIDA, solo
        // que de otra cosa. Sin comparar el preambulo, pasaria.
        let p = en_disco("otra_cabeza");
        let mut f = FirmanteCabeza::desde_semilla(&semilla(), &p).expect("abrir");
        let c = f.firmar(&[0xAAu8; 32]).expect("firmar");
        let pk = f.clave_publica();
        match verificar_cabeza(&pk, &[0xBBu8; 32], &c) {
            Err(FirmaError::PreambuloDistinto { .. }) => {}
            otro => panic!("una firma de OTRA cabeza debe rechazarse, y dio: {otro:?}"),
        }
    }

    #[test]
    fn una_version_de_formato_cambiada_se_rechaza() {
        let p = en_disco("version");
        let mut f = FirmanteCabeza::desde_semilla(&semilla(), &p).expect("abrir");
        let d = [0x11u8; 32];
        let mut c = f.firmar(&d).expect("firmar");
        let pk = f.clave_publica();
        c.version_formato = VERSION_FORMATO + 1;
        assert!(verificar_cabeza(&pk, &d, &c).is_err(), "otra version debe fallar");
    }

    #[test]
    fn un_byte_cambiado_en_la_firma_se_rechaza() {
        let p = en_disco("byte");
        let mut f = FirmanteCabeza::desde_semilla(&semilla(), &p).expect("abrir");
        let d = [0x22u8; 32];
        let mut c = f.firmar(&d).expect("firmar");
        let pk = f.clave_publica();
        c.firma[100] ^= 0x01;
        assert!(verificar_cabeza(&pk, &d, &c).is_err(), "un bit cambiado debe fallar");
    }

    #[test]
    fn basura_se_rechaza_sin_reventar() {
        // ⚠️ Un testigo recibe lo que le manden. Debe dar Err, no panic.
        let p = en_disco("basura");
        let mut f = FirmanteCabeza::desde_semilla(&semilla(), &p).expect("abrir");
        let d = [0x33u8; 32];
        let c0 = f.firmar(&d).expect("firmar");
        let pk = f.clave_publica();
        for firma in [vec![], vec![0u8; 10], vec![0xFFu8; 18_519], c0.firma[..100].to_vec()] {
            let c = CabezaFirmada { firma, ..c0.clone() };
            assert!(verificar_cabeza(&pk, &d, &c).is_err(), "la basura debe dar Err");
        }
        for clave in [vec![], vec![0u8; 3], vec![0u8; 68], vec![0xFFu8; 200]] {
            assert!(verificar_cabeza(&clave, &d, &c0).is_err(), "una clave rota debe dar Err");
        }
    }

    #[test]
    fn la_clave_publicada_lleva_el_oid_del_rfc_sin_el_apano() {
        // ⚠️ Lo que se PUBLICA es RFC 8391 correcto: OID 0x00000005, SIN el
        // offset. El apaño vive en la lectura, no en el cable.
        let p = en_disco("oid_publicado");
        let f = FirmanteCabeza::desde_semilla(&semilla(), &p).expect("abrir");
        let pk = f.clave_publica();
        assert_eq!(&pk[..4], &[0x00, 0x00, 0x00, 0x05], "el OID publicado debe ser el del RFC");
        assert_eq!(pk.len(), 68, "OID(4) + root(32) + pub_seed(32)");
    }

    #[test]
    fn el_apano_del_oid_sigue_haciendo_falta() {
        // ⚠️⚠️ EL TEST QUE MATARA EL APAÑO CUANDO SOBRE.
        //
        // Comprueba que SIN sumar el offset la clave NO se puede releer. El
        // dia que upstream arregle `parse_oid_and_params`, esto se pondra
        // ROJO y avisara de que `OFFSET_MT_UPSTREAM` hay que quitarlo.
        //
        // Un apaño que no sabe cuando estorba se queda para siempre — y
        // ademas ENMASCARA el cambio de formato que venga despues.
        let p = en_disco("apano");
        let f = FirmanteCabeza::desde_semilla(&semilla(), &p).expect("abrir");
        let pk = f.clave_publica();
        assert!(
            VerifyingKey::<Conjunto>::try_from(pk.as_slice()).is_err(),
            "⚠️ `xmss` YA RELEE la clave multiarbol sin el apaño: quitar \
             `OFFSET_MT_UPSTREAM` y `clave_desde_bytes`, y cerrar el hallazgo \
             en doc/issue-rustcrypto.md"
        );
    }

    #[test]
    fn la_firma_publicada_son_los_bytes_del_rfc() {
        let p = en_disco("rfc");
        let mut f = FirmanteCabeza::desde_semilla(&semilla(), &p).expect("abrir");
        let c = f.firmar(&[0x44u8; 32]).expect("firmar");
        assert_eq!(c.firma.len(), 18_469 + DOMINIO.len() + 1 + 32);
        assert_eq!(c.firma.len(), 18_519);
    }

    #[test]
    fn el_guardian_persiste_antes_y_los_dos_indices_avanzan_juntos() {
        // ⚠️ El invariante de §234, ahora con su consumidor: ninguna firma
        // puede existir con indice mayor que el contador persistido.
        let p = en_disco("juntos");
        let mut f = FirmanteCabeza::desde_semilla(&semilla(), &p).expect("abrir");
        for esperado in 1..=3u64 {
            let c = f.firmar(&[esperado as u8; 32]).expect("firmar");
            assert_eq!(c.indice, esperado);
            assert_eq!(f.indice_del_guardian(), esperado);
            assert_eq!(f.indice_de_la_clave().expect("indice"), esperado);
            assert_eq!(
                f.reconciliar().expect("reconciliar"),
                Reconciliacion::Coincide { indice: esperado },
                "el contador y la clave deben ir juntos tras cada firma"
            );
            // Y en DISCO, no solo en memoria.
            let en_disco = std::fs::read(&p).expect("leer el contador");
            assert_eq!(
                u64::from_le_bytes(en_disco.try_into().expect("8 bytes")),
                esperado,
                "CRITICO: el contador no esta persistido tras firmar"
            );
        }
    }
}
