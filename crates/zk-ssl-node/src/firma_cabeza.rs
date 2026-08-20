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
use zk_ssl_guardian::{indice_de_sk, GuardianError, GuardianIndice, Reconciliacion};

// ⚠️ Reexportado, no reimplementado: la verificación vive en `zk-ssl-verify`
// y quien la usaba desde aquí (main.rs, latido.rs) no cambia.
pub use zk_ssl_verify::{
    preambulo, verificar_cabeza, CabezaFirmada, Conjunto, VerificaError, DOMINIO,
    FIRMA_RFC_BYTES, VERSION_FORMATO,
};


#[derive(Debug)]
pub enum FirmaError {
    Guardian(GuardianError),
    /// El crate `xmss` rechazó la operación. Incluye `KeyExhausted`.
    Xmss(String),
    /// La firma recién hecha no verifica, o verifica contra otra cosa.
    Verifica(VerificaError),
}

impl std::fmt::Display for FirmaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FirmaError::Guardian(e) => write!(f, "firmante: {e}"),
            FirmaError::Xmss(e) => write!(f, "firmante: xmss rechazó: {e}"),
            FirmaError::Verifica(e) => write!(f, "firmante: {e}"),
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

// ⚠️ §298 · **La lectura del índice YA NO VIVE AQUÍ.** Se mudó a
// `zk-ssl-guardian`, junto al contador que protege: reservar, comprobar el
// layout y reconciliar son **la misma pieza**, y el TESTIGO (§299) necesita
// las tres. Tenerla partida le habría obligado a reimplementar la lectura
// del layout —dos lecturas del mismo formato que pueden discrepar (§253)—.

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
            .map_err(|e| FirmaError::Xmss(format!("{e}")))?;
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
            .map_err(|e| FirmaError::Xmss(format!("{e}")))?;
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
        // ⚠️ §298: la lectura vive en el guardián y su error es el suyo. El
        // `?` lo convierte con el `From` de arriba — sin él no compila.
        Ok(indice_de_sk(self.par.signing_key().as_ref())?)
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

    // ⚠️ §298 · los dos tests del LAYOUT se mudaron con la pieza, a
    // `zk-ssl-guardian`. El de abajo se queda: necesita una clave XMSS de
    // verdad, y el guardian no depende de `xmss` — ni debe.

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
