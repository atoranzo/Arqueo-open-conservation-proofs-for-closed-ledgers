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

use xmss::{KeyPair, SigningKey};
use zeroize::Zeroize;

// ⚠️ §296: el guardian vive en su propio crate. El nodo y el TESTIGO
// comparten LA MISMA implementacion — dos del mismo invariante pueden
// discrepar, y aqui discrepar significa FILTRAR UNA CLAVE (§253, §243).
use zk_ssl_guardian::{
    indice_de_sk, poner_indice_en_sk, GuardianError, GuardianIndice, Reconciliacion,
};

// ⚠️ Reexportado, no reimplementado: la verificación vive en `zk-ssl-verify`
// y quien la usaba desde aquí (main.rs, latido.rs) no cambia.
pub use zk_ssl_verify::{
    aplicar_apano_del_oid, indice_de_firma, preambulo, verificar_cabeza,
    CabezaFirmada, Conjunto, VerificaError, DOMINIO, FIRMA_RFC_BYTES,
    VERSION_FORMATO,
};


/// ⚠️⚠️ **El invariante que el `&mut` deja huerfano.**
/// `KeyPair::signing_key()` devuelve `&mut`, asi que nada impide asignar un SK
/// de OTRA semilla y romper la pareja `sk`/`vk` sin que el tipo lo note. Este
/// mod lo ATA: se resincroniza, se firma, y se verifica **con la clave publica
/// que NO se toco**. Sin el, el S335 abre una puerta que el tipo cerraba.
///
/// ⚠️ No pasa por `FirmanteCabeza` a proposito: asi no necesita guardian ni
/// fichero en disco, y aisla exactamente lo que se quiere probar.
#[cfg(test)]
mod el_par_sigue_atado {
    use super::*;
    use zk_ssl_guardian::{indice_de_sk, poner_indice_en_sk};

    #[test]
    fn resincronizar_no_rompe_la_pareja_y_usa_la_hoja_pedida() {
        let semilla = [7u8; 96];
        let mut par = KeyPair::<Conjunto>::from_seed(&semilla).expect("keygen");
        let pk_antes = par.verifying_key().as_ref().to_vec();

        let mut sk = par.signing_key().as_ref().to_vec();
        poner_indice_en_sk(&mut sk, 5).expect("poner el indice");
        aplicar_apano_del_oid(&mut sk).expect("el apano del OID");
        *par.signing_key() =
            SigningKey::<Conjunto>::try_from(sk.as_slice()).expect("rehacer el SK");
        sk.zeroize();

        assert_eq!(
            indice_de_sk(par.signing_key().as_ref()).expect("leer"),
            5,
            "la clave tiene que quedar EN el indice pedido"
        );

        let digest = [9u8; 32];
        let pre = preambulo(VERSION_FORMATO, &digest);
        let sig = par.signing_key().sign(&pre).expect("firmar");
        let c = CabezaFirmada {
            version_formato: VERSION_FORMATO,
            indice: 6,
            firma: sig.as_ref().to_vec(),
        };

        verificar_cabeza(&pk_antes, &digest, &c)
            .expect("la clave publica que NO se toco tiene que verificar la firma");

        assert_eq!(
            indice_de_firma(&c.firma).expect("indice embebido"),
            5,
            "la hoja gastada es la resincronizada, no otra"
        );
    }
}

/// **El presupuesto de la clave**: 2^(8·ancho del índice) hojas, derivado y no tecleado.
pub const PRESUPUESTO_DE_LA_CLAVE: u64 = 1 << (8 * zk_ssl_verify::ANCHO_INDICE);

#[derive(Debug)]
pub enum FirmaError {
    Guardian(GuardianError),
    /// El crate `xmss` rechazó la operación. Incluye `KeyExhausted`.
    Xmss(String),
    /// La firma recién hecha no verifica, o verifica contra otra cosa.
    Verifica(VerificaError),
    /// El acta no se puede firmar o, firmada, no la acepta su juez (RFC-0015, §644).
    Acta(String),
    /// ⚠️ §645 · La clave está en el techo: no queda hoja que firmar, y no se reserva ninguna.
    Agotada {
        hoja: u64,
    },
}

impl std::fmt::Display for FirmaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FirmaError::Guardian(e) => write!(f, "firmante: {e}"),
            FirmaError::Xmss(e) => write!(f, "firmante: xmss rechazó: {e}"),
            FirmaError::Verifica(e) => write!(f, "firmante: {e}"),
            FirmaError::Acta(e) => write!(f, "firmante, acta de clave: {e}"),
            FirmaError::Agotada { hoja } => write!(
                f,
                "firmante: la clave esta AGOTADA en la hoja {hoja} de {PRESUPUESTO_DE_LA_CLAVE}: \
                 no se reserva ni se firma; toca rotar a la sucesora comprometida (RFC-0015)"
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
        // ── 0 · ⚠️ §645: en el techo NO se reserva ──
        self.hoja_con_presupuesto()?;
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

    /// **Firma un acta de clave** (RFC-0015, E3a; §644) por el MISMO camino que una cabeza:
    /// reservar con `fsync`, firmar, y verificar la propia salida con el juez del tercero
    /// (`verificar_acta`) antes de devolverla. Devuelve el acta firmada y el índice DECLARADO
    /// que reservó, el que su línea del diario lleva para la puerta del contador.
    ///
    /// ⚠️ El `desde` del acta tiene que ser la hoja en que la clave está: la primera firma de la
    /// clave que entra es su acta, y lleva dentro el `desde` (D-C). Si no cuadra, no se reserva
    /// ni se firma nada.
    pub fn firmar_acta(
        &mut self,
        acta: zk_ssl_verify::actas::Acta,
        previa: Option<&zk_ssl_verify::actas::Acta>,
        firma_anterior: Option<Vec<u8>>,
    ) -> Result<(zk_ssl_verify::actas::ActaFirmada, u64), FirmaError> {
        use zk_ssl_verify::actas::{preambulo_acta, verificar_acta, ActaFirmada, ACTA_VERSION};
        use zk_ssl_wire::digest_to_wire;
        let hoja = self.hoja_con_presupuesto()?;
        if acta.desde != hoja {
            return Err(FirmaError::Acta(format!(
                "el acta dice desde {} y la clave esta en la hoja {hoja}",
                acta.desde
            )));
        }
        let indice = self.guardian.reservar()?;
        let pre = preambulo_acta(ACTA_VERSION, &digest_to_wire(&acta.digest()).0);
        let sig = self
            .par
            .signing_key()
            .sign(&pre)
            .map_err(|e| FirmaError::Xmss(format!("{e}")))?;
        let firmada = ActaFirmada {
            acta,
            firma: sig.as_ref().to_vec(),
            firma_anterior,
        };
        verificar_acta(&firmada, previa).map_err(|e| FirmaError::Acta(e.to_string()))?;
        Ok((firmada, indice))
    }

    /// **La clave que se va firma el acta de su sucesora** (RFC-0015, decisión 5; §645), con
    /// la hoja que el contador da -la siguiente a todo lo que firmó-, reservada ANTES de firmar
    /// por el MISMO guardián. Después, la clave que entra salta a la hoja siguiente: el `desde`
    /// del acta tiene que ser esa, y su hoja de abajo queda perdida, sin firmar nunca.
    ///
    /// ⚠️ Comprueba ANTES de reservar que la semilla es la de la clave del acta en vigor: una
    /// semilla equivocada no gasta nada. Y que el estado de la vieja es fiable lo afirma el
    /// operador al darla; la puerta del contador (§594) ya garantiza que la hoja no se usó.
    pub fn firmar_con_la_anterior(
        &mut self,
        semilla: &[u8],
        clave_anterior: &[u8],
        acta: &zk_ssl_verify::actas::Acta,
    ) -> Result<Vec<u8>, FirmaError> {
        use zk_ssl_verify::actas::{preambulo_acta, ACTA_VERSION};
        use zk_ssl_wire::digest_to_wire;
        let mut vieja = KeyPair::<Conjunto>::from_seed(semilla)
            .map_err(|e| FirmaError::Xmss(format!("{e}")))?;
        if vieja.verifying_key().as_ref() != clave_anterior {
            return Err(FirmaError::Acta(
                "la semilla anterior no es la de la clave del acta en vigor".into(),
            ));
        }
        let hoja = self.hoja_con_presupuesto()?;
        if acta.desde != hoja + 1 {
            return Err(FirmaError::Acta(format!(
                "con la firma de la anterior en la hoja {hoja}, el acta tiene que decir desde {}",
                hoja + 1
            )));
        }
        let mut sk = vieja.signing_key().as_ref().to_vec();
        poner_indice_en_sk(&mut sk, hoja)?;
        aplicar_apano_del_oid(&mut sk).map_err(|e| FirmaError::Xmss(format!("{e:?}")))?;
        *vieja.signing_key() = SigningKey::<Conjunto>::try_from(sk.as_slice())
            .map_err(|e| FirmaError::Xmss(format!("{e}")))?;
        sk.zeroize();
        self.guardian.reservar()?;
        let pre = preambulo_acta(ACTA_VERSION, &digest_to_wire(&acta.digest()).0);
        let sig = vieja
            .signing_key()
            .sign(&pre)
            .map_err(|e| FirmaError::Xmss(format!("{e}")))?;
        self.resincronizar_a(hoja + 1)?;
        Ok(sig.as_ref().to_vec())
    }

    /// ⚠️ §645 · **En el techo no se reserva.** La hoja en que la clave está, si queda
    /// presupuesto; si no, `Agotada` SIN tocar el contador. Antes el guardián reservaba y el
    /// `xmss` fallaba después, así que cada latido en el techo quemaba un índice más.
    ///
    /// ⚠️⚠️ **El techo lo dice el CONTADOR, no el SK. MEDIDO en el §645**: tras firmar con la
    /// última hoja, el índice del SK sigue leyendo 2^40 − 1 -su campo de cinco bytes no
    /// representa 2^40-, así que la clave en su última hoja y la clave agotada se leen igual. Y
    /// una segunda firma no da `KeyExhausted`: da una firma que NO verifica, y solo la
    /// autoverificación de `firmar` impide publicarla. El contador es un `u64` y sí llega a 2^40.
    fn hoja_con_presupuesto(&mut self) -> Result<u64, FirmaError> {
        let hoja = self.indice_de_la_clave()?;
        let contador = self.guardian.actual();
        if contador >= PRESUPUESTO_DE_LA_CLAVE || hoja >= PRESUPUESTO_DE_LA_CLAVE {
            return Err(FirmaError::Agotada {
                hoja: contador.max(hoja),
            });
        }
        Ok(hoja)
    }

    /// El índice que la clave dice tener, leído de su SK.
    pub fn indice_de_la_clave(&mut self) -> Result<u64, FirmaError> {
        // ⚠️ §298: la lectura vive en el guardián y su error es el suyo. El
        // `?` lo convierte con el `From` de arriba — sin él no compila.
        Ok(indice_de_sk(self.par.signing_key().as_ref())?)
    }

    /// Pone la clave EN el indice que el guardian tiene registrado.
    ///
    /// ⚠️⚠️ Es CONSERVADOR: como el contador se persiste ANTES de
    /// firmar, la hoja `indice` NUNCA se reservo y no puede estar quemada. Las
    /// de abajo quedan PERDIDAS -no reutilizables-, que es lo que la nota 92
    /// pide frente a dejarlas indeterminadas.
    ///
    /// ⚠️ El SK viejo se ZEROIZA solo al asignar: `SigningKey` tiene `Drop`.
    /// El buffer temporal se borra a mano y es **BEST-EFFORT**: un `Vec` pudo
    /// reubicarse mientras se construia, asi que borrar el ultimo puntero no
    /// promete nada sobre copias intermedias. Y `KeyPair::from_seed` de upstream
    /// ya deja una copia sin borrar en cada arranque: eso es del crate ajeno y
    /// va DECLARADO, no arreglado aqui.
    pub fn resincronizar_a(&mut self, indice: u64) -> Result<(), FirmaError> {
        let mut sk = self.par.signing_key().as_ref().to_vec();
        poner_indice_en_sk(&mut sk, indice)?;
        // ⚠️⚠️ EL APANO DEL OID, y es EL MISMO que usa el verificador.
        //    `xmss` prueba los OID de arbol unico ANTES que los de XMSS^MT, y
        //    el 5 acierta con el significado equivocado, asi que sin esto
        //    `try_from` rechaza un SK que el propio crate acaba de producir.
        //    Solo para volver a ENTRAR: el SK no se publica jamas.
        aplicar_apano_del_oid(&mut sk).map_err(|e| FirmaError::Xmss(format!("{e:?}")))?;
        let nueva = SigningKey::<Conjunto>::try_from(sk.as_slice())
            .map_err(|e| FirmaError::Xmss(format!("{e}")))?;
        *self.par.signing_key() = nueva;
        sk.zeroize();
        // ⚠️ Se AUTOCOMPRUEBA antes de devolver, como el firmar del S299:
        //    no se afirma que la clave esta donde se pidio sin releerlo.
        let leido = self.indice_de_la_clave()?;
        if leido != indice {
            return Err(FirmaError::Xmss(format!(
                "resincronizar pidio el indice {indice} y la clave dice {leido}"
            )));
        }
        Ok(())
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

    /// ⚠️ §645: **en el techo no se reserva.** Con el contador y la clave en la ultima hoja, la
    /// ultima firma sale; la siguiente es `Agotada` y el contador NO se mueve. Antes, el guardian
    /// reservaba y el `xmss` devolvia una firma que no verifica: cada latido en el techo quemaba
    /// un indice mas. Y el SK no sirve de juez: tras la ultima hoja sigue leyendo 2^40 - 1.
    #[test]
    fn en_el_techo_no_se_reserva_ni_se_firma() {
        let ruta = en_disco("techo");
        let ultima = PRESUPUESTO_DE_LA_CLAVE - 1;
        std::fs::write(&ruta, ultima.to_le_bytes()).expect("contador en la ultima hoja");
        let mut f = FirmanteCabeza::desde_semilla(&semilla(), &ruta).expect("abrir");
        f.resincronizar_a(ultima)
            .expect("la clave en la ultima hoja");
        let c = f.firmar(&[3u8; 32]).expect("la ultima hoja firma");
        assert_eq!(indice_de_firma(&c.firma).expect("embebido"), ultima);
        assert_eq!(f.indice_del_guardian(), PRESUPUESTO_DE_LA_CLAVE);
        assert_eq!(
            f.indice_de_la_clave().expect("sk"),
            ultima,
            "el SK no representa 2^40"
        );
        match f.firmar(&[4u8; 32]) {
            Err(FirmaError::Agotada { hoja }) => assert_eq!(hoja, PRESUPUESTO_DE_LA_CLAVE),
            otra => panic!("en el techo tiene que ser Agotada: {otra:?}"),
        }
        assert_eq!(
            f.indice_del_guardian(),
            PRESUPUESTO_DE_LA_CLAVE,
            "y el contador no se mueve"
        );
    }
}
