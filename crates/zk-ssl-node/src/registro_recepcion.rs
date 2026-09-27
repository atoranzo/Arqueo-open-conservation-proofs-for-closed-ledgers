//! # El registro de recepción: qué operación llevó cada número
//!
//! [`crate::recepcion::ContadorRecepcion`] dice **cuántas** operaciones
//! evaluó el nodo. Esto dice **cuáles**: por cada número `rx` reservado, el
//! digest de la prueba que llegó con él y la **era** que ese instante fijó.
//!
//! Sin esto no hay hoja. `zk_ssl_verify::recibos::hoja_de_recibo` compone con
//! `(hash_prueba, era, n)`, y de esos tres el contador no guarda ninguno.
//!
//! ## ⚠️ Por qué vive en el NODO y no en la capa (D-I)
//!
//! Por el criterio de §220, el mismo que puso aquí el contador y las reservas
//! de posición: **cuánto retiene el operador su registro de lo que evaluó es
//! política del operador, no invariante de la liquidación**. Meterlo en `sled`
//! lo haría estado del ledger, y no lo es: un nodo que pierde su registro no
//! ha perdido dinero de nadie — ha perdido su capacidad de responder.
//!
//! Y por eso **no toca `sled` ni `persistence.rs`**, igual que el guardián:
//! un directorio, ficheros de ancho fijo y un orden.
//!
//! ## La forma: un fichero por ERA (D-K)
//!
//! `era-<era>.bin`, y dentro entradas de [`ANCHO_ENTRADA`] bytes: `rx` en
//! ocho little-endian y el digest de la prueba en treinta y dos.
//!
//! Un fichero único indexado por `rx` sería más compacto y **no se podría
//! podar por delante** sin un offset base que habría que persistir y
//! reconciliar — otra pieza de estado, y con ella otra forma de divergir. Con
//! un fichero por era, podar es borrar ficheros: la operación más simple que
//! existe y la única que no puede dejar el índice a medias.
//!
//! La era se conoce **al recibir** (`era_de_recibo` sobre el `seq` de la
//! última cabeza firmada, D-D), así que el destino está decidido antes de
//! escribir. El límite inferior `Q` NO se guarda: se deriva de dos cabezas
//! firmadas al componer, que es lo que dice `recibos`.
//!
//! ## ⚠️ El orden, y qué se persiste antes de responder (D-J)
//!
//! El mismo de §234, heredado del contador: **la entrada está en disco antes
//! de que el nodo responda**. Se sincroniza el fichero Y el directorio —sin lo
//! segundo el nombre nuevo puede no sobrevivir—, y sólo entonces se vuelve.
//!
//! Si el proceso muere entre reservar y anotar, queda un `rx` **sin hoja**: un
//! hueco, del que la era da cuenta. Es el caso seguro, y es el mismo precio
//! que §242 declaró benigno para los huecos de firma.
//!
//! ## ⚠️⚠️ Lo que NO es benigno: el registro por delante del contador
//!
//! Un hueco es declarable. Lo que rompe la propiedad es que el registro
//! contenga números que el contador no reservó, porque entonces **dos
//! operaciones distintas pueden llevar el mismo `rx`** — exactamente lo que
//! §253 declaró peor que no tener contador.
//!
//! El caso real no es una caída: es **restaurar uno de los dos ficheros sin el
//! otro**. Y eso **no nombra un veredicto: nombra dos**, porque los dos ficheros
//! no son simétricos. Restaurar viejo el CONTADOR deja el registro por delante
//! —`ClaveAdelantada`, que es esta rama—. Restaurar viejo el REGISTRO deja el
//! contador por delante —`ContadorAdelantado`, el que la sección anterior acaba
//! de declarar benigno—.
//!
//! ⚠️ CORRECCIÓN (§565-B), citada y no borrada. Este párrafo decía: «Es el mismo
//! defecto que el banco de HBS-STATE mide en LMS (su M5): restaurada una clave
//! vieja, la librería vuelve a firmar con hojas ya usadas, y las dos firmas
//! verifican.» La mecánica es correcta y **la etiqueta estaba colgada de la rama
//! equivocada**: restaurada una clave VIEJA, el material queda DETRÁS del
//! contador, y eso es `ContadorAdelantado`. El M5 cae en la rama benigna, no en
//! ésta; el que cae aquí es su espejo.
//!
//! ⚠️ Y LO QUE ESTA RAMA NO PARA, dicho junto a lo que para: restaurar el
//! REGISTRO viejo con el contador vivo es `ContadorAdelantado`, y es
//! **indistinguible de un hueco legítimo por caída** —los dos casos son el MISMO
//! par de números—. El spec de HBS-STATE lo dice verbatim, «No datum on disk
//! separates the innocent case from the dangerous one», y anota su vector `A4`
//! —`counter=7 key=5 -> CounterAhead, fatal=False`— como «the normal case».
//! Fallar cerrado ante cualquier discrepancia hacia atrás pararía también la
//! operativa normal. La salida no es una política de este módulo: pide un DATO
//! MÁS ALLÁ DEL PAR —comprobación al ABRIR el material y un testigo negativo de
//! la hoja quemada—, y eso es otra etapa. **La D-J cubre media restauración.**
//!
//! Por eso el arranque **no lo interpreta aquí**: se lo pregunta a
//! [`zk_ssl_guardian::Reconciliacion`] a través de
//! [`crate::recepcion::ContadorRecepcion::reconciliar`], que es la máquina que
//! §296 y §298 ya midieron para el índice de firma. Dos implementaciones del
//! mismo problema pueden discrepar; ésta no se escribe dos veces.
//!
//! ## ⚠️ Lo que este módulo NO afirma
//!
//! Nada de lo que hay aquí es evidencia oponible. El registro es **un
//! directorio que el operador escribe y que nada ata**: sirve para COMPONER la
//! hoja, y lo que la hace oponible es la raíz firmada en la cabeza, que es
//! otra etapa. Venderlo como prueba sería el mismo falso «limpio» que §250
//! encontró un piso más arriba.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use zk_ssl_guardian::GuardianError;
use zk_ssl_verify::acuses::Digest;
// ⚠️ La conversion `Digest` <-> 32 bytes viene del CABLE, y es a proposito:
// `zk-ssl-hash` NO es dependencia directa de este crate -medido en su
// `Cargo.toml`- y `zk-ssl-verify` no la re-exporta. `zk-ssl-wire` si lo es y ya
// la tiene resuelta. Escribir aqui una segunda conversion seria un segundo
// productor del mismo formato, que es como dos composiciones divergen en
// silencio. Un fichero local que usa el mismo mapeo que el cable no es impuro:
// es el MISMO mapeo, y por eso no puede discrepar de el.
use zk_ssl_wire::{digest_from_wire, digest_to_wire, B32};

/// Bytes por entrada: `rx` en ocho little-endian y el digest en treinta y dos.
pub const ANCHO_ENTRADA: usize = 40;

/// El prefijo del nombre de cada fichero de era. El nombre lleva la era y
/// nada más: **el `Q` no se guarda** porque se deriva de dos cabezas firmadas.
const PREFIJO: &str = "era-";
const SUFIJO: &str = ".bin";

fn io<E: std::fmt::Display>(e: E) -> GuardianError {
    GuardianError::Io(e.to_string())
}

/// El registro de lo que el nodo evaluó, por eras y en disco.
pub struct RegistroRecepcion {
    dir: PathBuf,
}

impl RegistroRecepcion {
    /// Abre —y crea si hace falta— el directorio del registro.
    ///
    /// ⚠️ No comprueba que el medio persista: eso ya lo hace el contador con
    /// el que se reconcilia, y comprobarlo dos veces daría dos veredictos.
    pub fn abrir(dir: impl AsRef<Path>) -> Result<Self, GuardianError> {
        let dir = dir.as_ref().to_path_buf();
        fs::create_dir_all(&dir).map_err(io)?;
        Ok(RegistroRecepcion { dir })
    }

    fn ruta_de_era(&self, era: u64) -> PathBuf {
        self.dir.join(format!("{PREFIJO}{era}{SUFIJO}"))
    }

    /// Anota `(rx, hash_prueba)` en el fichero de su `era`, **con `fsync` del
    /// fichero y del directorio ANTES de volver**.
    ///
    /// ⚠️ El orden es el de §234 y la razón es la misma: devolver sin
    /// persistir deja una hoja que el siguiente arranque no encuentra, y la
    /// cabeza la habría firmado.
    pub fn anotar(&mut self, rx: u64, era: u64, hash_prueba: Digest) -> Result<(), GuardianError> {
        let mut entrada = [0u8; ANCHO_ENTRADA];
        entrada[0..8].copy_from_slice(&rx.to_le_bytes());
        entrada[8..40].copy_from_slice(&digest_to_wire(&hash_prueba).0);

        let ruta = self.ruta_de_era(era);
        let nuevo = !ruta.exists();
        let mut f = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&ruta)
            .map_err(io)?;
        f.write_all(&entrada).map_err(io)?;
        f.sync_all().map_err(io)?;
        // El directorio TAMBIEN: sin esto el nombre del fichero nuevo puede no
        // sobrevivir a un corte, y la era entera quedaria invisible.
        if nuevo {
            fs::File::open(&self.dir).map_err(io)?.sync_all().map_err(io)?;
        }
        Ok(())
    }

    /// Los pares `(rx, hash_prueba)` de una era, en el orden en que se
    /// anotaron. Una era sin fichero devuelve el vector vacío: **no es un
    /// error**, es una era en la que el nodo no evaluó nada.
    pub fn pares_de_era(&self, era: u64) -> Result<Vec<(u64, Digest)>, GuardianError> {
        let ruta = self.ruta_de_era(era);
        if !ruta.exists() {
            return Ok(Vec::new());
        }
        let bytes = fs::read(&ruta).map_err(io)?;
        if bytes.len() % ANCHO_ENTRADA != 0 {
            // Falla CERRADA y NOMBRANDO: media entrada en disco no se
            // completa adivinando, porque la hoja que saliera de ahi
            // "verificaria" contra una raiz que nadie firmo.
            return Err(GuardianError::Corrupto { bytes: bytes.len() });
        }
        let mut fuera = Vec::with_capacity(bytes.len() / ANCHO_ENTRADA);
        for trozo in bytes.chunks(ANCHO_ENTRADA) {
            let mut ocho = [0u8; 8];
            ocho.copy_from_slice(&trozo[0..8]);
            let rx = u64::from_le_bytes(ocho);
            let mut treinta_y_dos = [0u8; 32];
            treinta_y_dos.copy_from_slice(&trozo[8..40]);
            let d = digest_from_wire(&B32(treinta_y_dos))
                .map_err(|e| GuardianError::Io(format!("digest de rx {rx}: {e:?}")))?;
            fuera.push((rx, d));
        }
        Ok(fuera)
    }

    /// El mayor `rx` anotado en TODO el registro, o 0 si no hay ninguno.
    ///
    /// ⚠️ Es lo que se le pasa a la reconciliación como «lo que el registro
    /// dice», y por eso es el MAYOR y no la cuenta: con eras podadas la
    /// cuenta baja y el mayor no, y lo que no puede retroceder es el mayor.
    pub fn mayor_anotado(&self) -> Result<u64, GuardianError> {
        let mut mayor = 0u64;
        for e in fs::read_dir(&self.dir).map_err(io)? {
            let e = e.map_err(io)?;
            let nombre = e.file_name().to_string_lossy().to_string();
            if !(nombre.starts_with(PREFIJO) && nombre.ends_with(SUFIJO)) {
                continue;
            }
            let bytes = fs::read(e.path()).map_err(io)?;
            if bytes.len() % ANCHO_ENTRADA != 0 {
                return Err(GuardianError::Corrupto { bytes: bytes.len() });
            }
            for trozo in bytes.chunks(ANCHO_ENTRADA) {
                let mut ocho = [0u8; 8];
                ocho.copy_from_slice(&trozo[0..8]);
                let rx = u64::from_le_bytes(ocho);
                if rx > mayor {
                    mayor = rx;
                }
            }
        }
        Ok(mayor)
    }

    /// Borra las eras cuya ventana ya expiró y devuelve cuántos ficheros
    /// quitó. La regla es la del RFC, no una propia:
    /// `zk_ssl_verify::recibos::dentro_de_ventana`.
    ///
    /// ⚠️ Fuera de la ventana el veredicto ya no depende del camino —
    /// `dentro_de_ventana` devuelve `false` y la promesa expiró —, así que
    /// retener más no sirve a ningún veredicto. Lo que se poda es lo que ya
    /// no puede responder nada.
    pub fn podar(&mut self, seq_cierre: u64, n: u64) -> Result<usize, GuardianError> {
        let mut quitados = 0usize;
        for e in fs::read_dir(&self.dir).map_err(io)? {
            let e = e.map_err(io)?;
            let nombre = e.file_name().to_string_lossy().to_string();
            if !(nombre.starts_with(PREFIJO) && nombre.ends_with(SUFIJO)) {
                continue;
            }
            let medio = &nombre[PREFIJO.len()..nombre.len() - SUFIJO.len()];
            let era: u64 = match medio.parse() {
                Ok(v) => v,
                // Un nombre que no es una era NO se borra: podar es destructivo
                // y lo que no se entiende se deja quieto.
                Err(_) => continue,
            };
            if !zk_ssl_verify::recibos::dentro_de_ventana(era, seq_cierre, n) {
                fs::remove_file(e.path()).map_err(io)?;
                quitados += 1;
            }
        }
        if quitados > 0 {
            fs::File::open(&self.dir).map_err(io)?.sync_all().map_err(io)?;
        }
        Ok(quitados)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zk_ssl_guardian::{no_admite_matiz, Reconciliacion};
    use zk_ssl_verify::acuses::as_digest;

    fn dir(nombre: &str) -> PathBuf {
        crate::tests_dir(nombre).join("recepciones")
    }

    #[test]
    fn lo_anotado_se_relee_igual_y_en_orden() {
        let mut r = RegistroRecepcion::abrir(dir("reg_relee")).expect("abrir");
        r.anotar(1, 7, as_digest(0xAA)).expect("anotar 1");
        r.anotar(2, 7, as_digest(0xBB)).expect("anotar 2");
        let p = r.pares_de_era(7).expect("leer");
        assert_eq!(p, vec![(1, as_digest(0xAA)), (2, as_digest(0xBB))]);
    }

    #[test]
    fn cada_era_en_su_fichero_y_no_se_mezclan() {
        let mut r = RegistroRecepcion::abrir(dir("reg_eras")).expect("abrir");
        r.anotar(1, 7, as_digest(1)).expect("a");
        r.anotar(2, 8, as_digest(2)).expect("b");
        assert_eq!(r.pares_de_era(7).expect("7").len(), 1, "la 8 se colo en la 7");
        assert_eq!(r.pares_de_era(8).expect("8").len(), 1, "la 7 se colo en la 8");
    }

    #[test]
    fn una_era_sin_fichero_no_es_un_error() {
        // Una era en la que el nodo no evaluo nada tiene raiz de arbol vacio,
        // no un fallo: la diferencia importa porque el vacio SI se firma.
        let r = RegistroRecepcion::abrir(dir("reg_vacia")).expect("abrir");
        assert_eq!(r.pares_de_era(99).expect("vacia"), Vec::new());
    }

    #[test]
    fn la_entrada_esta_en_disco_antes_de_volver() {
        // ⚠️ El orden de §234, y aqui medido sobre los BYTES: si `anotar`
        // volviera sin persistir, la cabeza firmaria una hoja que el siguiente
        // arranque no encuentra.
        let d = dir("reg_fsync");
        let mut r = RegistroRecepcion::abrir(&d).expect("abrir");
        r.anotar(3, 4, as_digest(0xC0FE)).expect("anotar");
        let bytes = fs::read(d.join("era-4.bin")).expect("leer el fichero de la era");
        assert_eq!(bytes.len(), ANCHO_ENTRADA, "la entrada no esta entera en disco");
        assert_eq!(u64::from_le_bytes(bytes[0..8].try_into().unwrap()), 3);
    }

    #[test]
    fn sobrevive_al_reinicio() {
        let d = dir("reg_reinicio");
        let mut primero = RegistroRecepcion::abrir(&d).expect("abrir");
        primero.anotar(1, 2, as_digest(9)).expect("anotar");
        drop(primero);
        let segundo = RegistroRecepcion::abrir(&d).expect("reabrir");
        assert_eq!(segundo.pares_de_era(2).expect("leer").len(), 1);
        assert_eq!(segundo.mayor_anotado().expect("mayor"), 1);
    }

    #[test]
    fn media_entrada_falla_cerrada_y_nombrando() {
        // Un fichero con 40 + 7 bytes es una caida a mitad de escritura. NO se
        // completa adivinando: la hoja que saliera de ahi "verificaria" contra
        // una raiz que nadie firmo.
        let d = dir("reg_corrupto");
        let mut r = RegistroRecepcion::abrir(&d).expect("abrir");
        r.anotar(1, 5, as_digest(1)).expect("anotar");
        let mut f = fs::OpenOptions::new()
            .append(true)
            .open(d.join("era-5.bin"))
            .expect("abrir para ensuciar");
        f.write_all(&[0u8; 7]).expect("escribir media entrada");
        drop(f);
        match r.pares_de_era(5) {
            Err(GuardianError::Corrupto { bytes }) => assert_eq!(bytes, ANCHO_ENTRADA + 7),
            otro => panic!("una entrada a medias tiene que fallar NOMBRANDO: {otro:?}"),
        }
    }

    #[test]
    fn podar_quita_lo_que_ya_no_puede_responder_y_deja_lo_vivo() {
        // La regla es la del RFC: fuera de la ventana `S - e > N`.
        let d = dir("reg_poda");
        let mut r = RegistroRecepcion::abrir(&d).expect("abrir");
        r.anotar(1, 10, as_digest(1)).expect("vieja");
        r.anotar(2, 90, as_digest(2)).expect("viva");
        let quitados = r.podar(100, 20).expect("podar");
        assert_eq!(quitados, 1, "no quito exactamente la era expirada");
        assert!(r.pares_de_era(10).expect("10").is_empty(), "la vieja sigue ahi");
        assert_eq!(r.pares_de_era(90).expect("90").len(), 1, "la poda se llevo una viva");
    }

    #[test]
    fn podar_no_toca_lo_que_no_entiende() {
        // Un nombre ajeno en el directorio no se borra: podar es destructivo.
        let d = dir("reg_poda_ajeno");
        let mut r = RegistroRecepcion::abrir(&d).expect("abrir");
        fs::write(d.join("era-no-es-un-numero.bin"), b"").expect("sembrar");
        fs::write(d.join("otra-cosa.txt"), b"").expect("sembrar");
        assert_eq!(r.podar(1000, 1).expect("podar"), 0, "borro algo que no era una era");
        assert!(d.join("otra-cosa.txt").exists(), "borro un fichero ajeno");
    }

    #[test]
    fn el_mayor_anotado_no_retrocede_al_podar() {
        // ⚠️ Es la razon por la que la reconciliacion usa el MAYOR y no la
        // cuenta: podar baja la cuenta, y si eso bajara el veredicto, una
        // poda legitima se leeria como un contador adelantado.
        let d = dir("reg_mayor");
        let mut r = RegistroRecepcion::abrir(&d).expect("abrir");
        r.anotar(1, 10, as_digest(1)).expect("a");
        r.anotar(5, 90, as_digest(5)).expect("b");
        assert_eq!(r.mayor_anotado().expect("antes"), 5);
        r.podar(100, 20).expect("podar");
        assert_eq!(r.mayor_anotado().expect("despues"), 5, "el mayor retrocedio con la poda");
    }

    #[test]
    fn el_registro_por_delante_del_contador_no_admite_matiz() {
        // ⚠️⚠️ EL ESPEJO DEL M5, traducido: restaurar el CONTADOR de un
        // respaldo viejo con el registro vivo. (Decia <<EL CASO M5>> y era
        // falso: el M5 restaura el MATERIAL viejo, que deja el contador por
        // delante -- `ContadorAdelantado`, la rama benigna. Corregido en el
        // §565-B, citando y sin borrar.) La maquina que lo dice es la del
        // guardian, no una propia -- dos implementaciones del mismo problema
        // pueden discrepar.
        let adelantado = Reconciliacion::ClaveAdelantada { contador: 3, clave: 9, sin_registrar: 6 };
        assert!(no_admite_matiz(&adelantado), "el registro por delante tiene que parar");
        let huecos = Reconciliacion::ContadorAdelantado { contador: 9, clave: 3, huerfanos: 6 };
        assert!(!no_admite_matiz(&huecos), "un hueco es declarable, no un muro");
    }
}
