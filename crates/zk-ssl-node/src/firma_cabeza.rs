//! # El firmante de cabezas de época
//!
//! Eslabón 3 de la cadena de la oponibilidad. Firma `EpochHead` con XMSS,
//! con el guardián del índice (§234) delante.
//!
//! ## ⚠️ La VERIFICACIÓN ya no vive aquí (§243)
//!
//! El preámbulo, [`CabezaFirmada`], [`verificar_cabeza`] y el apaño del OID
//! están en **`zk-ssl-verify`**, un crate que **solo depende de `xmss`**.
//!
//! Hasta §242 vivían dentro de este binario, junto a `tokio` y `axum`: **la
//! única forma de verificar una cabeza era compilar el código del
//! operador** — exactamente la dependencia que el aparato existe para
//! eliminar. Aquí se reexportan para que quien ya los usaba siga igual.
//!
//! **Lo que queda en este módulo es solo lo que hace falta para FIRMAR**: el
//! par de claves, el guardián, y la lectura del índice del SK.
//!
//! ## Lo que se firma
//!
//! ```text
//! preámbulo = b"ZK-SSL-epoch-head" ‖ versión_de_formato ‖ epoch_digest
//! ```
//!
//! Ver `zk-ssl-verify` para por qué lleva versión de formato, por qué el
//! dominio no la lleva dentro, y por qué no toca `epoch_digest`.
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
//! - **No hay custodia de clave.** *Una firma sin custodia declarada no
//!   tiene valor probatorio.* Toma una semilla; **de dónde sale y quién la
//!   guarda es decisión de despliegue**, y no está tomada.
//! - `xmss` es **`0.1.0-pre.0`, sin auditoría independiente**.

use std::path::Path;

use xmss::KeyPair;

// ⚠️ §296: el guardian vive en su propio crate. El nodo y el TESTIGO
// comparten LA MISMA implementacion — dos del mismo invariante pueden
// discrepar, y aqui discrepar significa FILTRAR UNA CLAVE (§253, §243).
use zk_ssl_guardian::{GuardianError, GuardianIndice, Reconciliacion};

// ⚠️ Reexportado, no reimplementado: la verificación vive en `zk-ssl-verify`
// y quien la usaba desde aquí (main.rs, latido.rs) no cambia.
pub use zk_ssl_verify::{
    preambulo, verificar_cabeza, CabezaFirmada, Conjunto, VerificaError, DOMINIO,
    FIRMA_RFC_BYTES, VERSION_FORMATO,
};

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
    /// La firma recién hecha no verifica, o verifica contra otra cosa.
    Verifica(VerificaError),
}

impl std::fmt::Display for FirmaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FirmaError::Guardian(e) => write!(f, "firmante: {e}"),
            FirmaError::Xmss(e) => write!(f, "firmante: xmss rechazó: {e}"),
            FirmaError::Verifica(e) => write!(f, "firmante: {e}"),
            FirmaError::LayoutInesperado { sk_len, esperado } => write!(
                f,
                "firmante: el SK mide {sk_len} bytes y se esperaban {esperado}. \
                 La serialización de `xmss` ha cambiado: el índice ya no está \
                 donde se midió, y leerlo daría un valor falso."
            ),
        }
    }
}

// ⚠️ `Debug`, `Display` y `Error` desde que nace (§241).
impl std::error::Error for FirmaError {}

impl From<GuardianError> for FirmaError {
    fn from(e: GuardianError) -> Self {
        FirmaError::Guardian(e)
    }
}

impl From<VerificaError> for FirmaError {
    fn from(e: VerificaError) -> Self {
        FirmaError::Verifica(e)
    }
}

/// Ancho del índice en bytes: ⌈h/8⌉ = 5 para `h = 40`.
const fn ancho_indice() -> usize {
    5
}

/// Lee el índice del SK: bytes `[4, 9)` en **big-endian**.
///
/// ⚠️ **El offset y el ancho están MEDIDOS** (S.3), no deducidos: el SK mide
/// **137 bytes** = OID(4) + índice(5) + 4×32, y al firmar cambia el byte 8
/// —el menos significativo de un entero big-endian que ocupa [4, 9)—.
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

/// Firma cabezas, con el guardián del índice delante.
pub struct FirmanteCabeza {
    par: KeyPair<Conjunto>,
    guardian: GuardianIndice,
}

impl FirmanteCabeza {
    /// ⚠️ **La semilla es material de clave.** De dónde sale y quién la
    /// guarda es **decisión de despliegue**, y no está tomada.
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
        let c = CabezaFirmada {
            version_formato: VERSION_FORMATO,
            indice,
            firma: sig.as_ref().to_vec(),
        };
        // ── 3 · ⚠️ verificar la propia salida ANTES de devolverla ──
        // No se emite una firma que no verifica. Cuesta 2,4 ms sobre 144,5
        // —el 1,7 %— y cierra la clase de fallo en que se publica una firma
        // invalida y nadie lo nota hasta que un testigo la rechaza.
        //
        // ⚠️ Y usa **el mismo verificador que usara el tercero**, no otro:
        // si el firmante y el testigo no comparten codigo, pueden discrepar.
        verificar_cabeza(&self.clave_publica(), epoch_digest, &c)?;
        Ok(c)
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
    /// `zk_ssl_verify::OFFSET_MT_UPSTREAM` NO se aplica aquí**: lo que se
    /// publica es correcto según el RFC, y el rodeo vive en la lectura.
    pub fn clave_publica(&self) -> Vec<u8> {
        self.par.verifying_key().as_ref().to_vec()
    }
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

    // ── el layout del SK: MEDIDO, y probado sin gastar 37 s ──

    #[test]
    fn el_lector_del_indice_maneja_el_acarreo() {
        // ⚠️ EL TEST DE LAYOUT, mitad sintetica. Firmar 256 veces contra la
        // clave real costaria **37 s medidos** (256 x 144,5 ms). El acarreo
        // es una propiedad del LECTOR, y aqui se prueba exhaustivamente.
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
        let p = en_disco("layout_real");
        let mut f = FirmanteCabeza::desde_semilla(&semilla(), &p).expect("abrir");
        assert_eq!(f.indice_de_la_clave().expect("indice"), 0, "una clave nueva empieza en 0");
        f.firmar(&[9u8; 32]).expect("firmar");
        assert_eq!(f.indice_de_la_clave().expect("indice"), 1, "tras una firma, 1");
    }

    #[test]
    fn firma_y_un_testigo_la_verifica_con_lo_publicado() {
        // ⚠️ Sin la clave privada, sin el guardian: solo con lo publicado, y
        // con **el mismo verificador que usaria un tercero**.
        let p = en_disco("verifica");
        let mut f = FirmanteCabeza::desde_semilla(&semilla(), &p).expect("abrir");
        let d = [0x5Au8; 32];
        let c = f.firmar(&d).expect("firmar");
        assert_eq!(c.version_formato, VERSION_FORMATO);
        assert_eq!(c.indice, 1);
        verificar_cabeza(&f.clave_publica(), &d, &c).expect("un testigo debe poder verificar");
        assert_eq!(c.firma.len(), FIRMA_RFC_BYTES + DOMINIO.len() + 1 + 32);
        assert_eq!(c.firma.len(), 18_519);
    }

    #[test]
    fn el_guardian_persiste_antes_y_los_dos_indices_avanzan_juntos() {
        // ⚠️ El invariante de §234, con su consumidor: ninguna firma puede
        // existir con indice mayor que el contador persistido.
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
