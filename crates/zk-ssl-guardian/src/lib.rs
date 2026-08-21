//! # El guardián del índice de firma
//!
//! XMSS es un esquema **con estado**: cada firma consume un índice, y
//! **reusar uno filtra la clave** (§106.4). No es una degradación: es
//! compromiso.
//!
//! ## Por qué vive aquí y no en la capa
//!
//! §111.1 lo decidió: **un contador propio con `fsync`, aislado del
//! ledger** — no un WAL. Y va en el nodo por el mismo criterio que las
//! reservas de posición (§220): *«cuánto dura una reserva es política del
//! operador, no invariante de la capa»*. Firmar cabezas es deber del
//! operador; la liquidación no depende de ello.
//!
//! Aislado significa aislado: **no toca `sled`, no toca `persistence.rs`,
//! no reimplementa nada**. Un fichero, ocho bytes, y un orden.
//!
//! ## ⚠️ XMSS cambia la premisa de durabilidad de la capa
//!
//! `persistence.rs` justifica no tener WAL con esta frase:
//!
//! > *«Perder una operación es recuperable: se vuelve a enviar.»*
//!
//! **Con XMSS deja de ser cierto** (§110.2). Si el proceso muere tras
//! firmar con un índice y antes de persistirlo, ese índice está **quemado
//! en una firma publicada** y el contador no lo sabe: al reiniciar se
//! reusa. Por eso el orden se invierte a propósito: **persistir primero,
//! firmar después.**
//!
//! ## El invariante, en una línea
//!
//! > **Ninguna firma puede existir con un índice mayor que el contador
//! > persistido.**
//!
//! Lo contrario —contador por delante, índice quemado sin firma— es el
//! caso **seguro**, y es el que resuelve [`Reconciliacion`]. Medido en el
//! banco K.1: ocurre en **13 de 25** muertes del proceso. **No es la
//! excepción: es el camino normal tras una caída.**
//!
//! ## ⚠️ La autocomprobación, y por qué no es paranoia
//!
//! K.1 midió `fsync` en dos sistemas de ficheros de la misma máquina:
//!
//! | | coste de `fsync` | frente a no persistir |
//! |---|---|---|
//! | ext4 | 0,907 ms | **382×** |
//! | tmpfs (`/tmp`) | 0,002 ms | **1×** |
//!
//! En `tmpfs`, `fsync` **devuelve éxito sin persistir nada** — no hay
//! disco. Un guardián cuyo fichero acabe ahí es un **no-op**, y la clave
//! queda en riesgo con cada llamada devolviendo `Ok`.
//!
//! Y `/tmp` es un sitio perfectamente plausible para un fichero que
//! alguien considere auxiliar.
//!
//! Por eso [`GuardianIndice::abrir`] **mide su propio `fsync` al arrancar
//! y se niega a operar** si el coste es indistinguible de no persistir. Es
//! la única señal disponible desde dentro del proceso.
//!
//! ⚠️ **Los umbrales salen de UNA máquina** —WSL2 sobre un i5-1135G7— y
//! están declarados, no derivados. Un NVMe rápido puede dar `fsync` de
//! ~100 µs legítimos; por eso el discriminante principal es la **razón**
//! contra no persistir, no el valor absoluto.
//!
//! ## ⚠️ Lo que esto NO garantiza
//!
//! **Nada frente a un corte de corriente.** *«`fsync` puede mentir»* habla
//! de discos que confirman escrituras que siguen en caché volátil. K.1
//! midió durabilidad frente a **muerte del proceso** —25 de 25 sin una
//! sola firma por delante—, y eso **no es lo mismo**. Medirlo exige cortar
//! la corriente de verdad, y no se ha hecho.
//!
//! ## ⚠️ Y esta pieza NO tiene consumidor todavía
//!
//! Es el **eslabón 2 de cinco** (`BACKLOG.md`, «la cadena de la
//! oponibilidad»); el 3 —la cabeza firmada, emitida— no existe. Se
//! construye antes a propósito, porque es la pieza más difícil de
//! retroadaptar. **El riesgo está declarado**: se diseña una API sin su
//! consumidor.

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Bytes del OID al principio del SK, en el formato de referencia.
const OID_BYTES: usize = 4;
/// Longitud del hash del conjunto `_256`.
const N: usize = 32;
/// Ancho del índice en bytes: ⌈h/8⌉ = 5 para `h = 40`.
const fn ancho_indice() -> usize {
    5
}

/// Cuántas escrituras usa la autocomprobación de arranque.
const MUESTRAS_AUTOCOMPROBACION: u32 = 20;

/// ⚠️ **Umbral DECLARADO, no derivado.** `fsync` tiene que costar al menos
/// esta razón frente a escribir sin persistir. Medido en K.1: ext4 dio
/// **382×** y tmpfs **1×**, así que 10 separa los dos casos con dos
/// órdenes de margen por el lado bueno.
const RAZON_MINIMA: f64 = 10.0;

/// Suelo absoluto, como segunda red. Un NVMe rápido hace `fsync` en
/// ~100 µs legítimos, así que esto se queda muy por debajo: solo caza el
/// caso «no hay disco».
const SUELO_MICROS: f64 = 20.0;

/// Lee el índice del SK: bytes `[4, 9)` en **big-endian**.
///
/// ⚠️ **El offset y el ancho están MEDIDOS** (S.3), no deducidos: el SK mide
/// **137 bytes** = OID(4) + índice(5) + 4×32, y al firmar cambia el byte 8
/// —el menos significativo de un entero big-endian que ocupa [4, 9)—.
///
/// ⚠️ La evaluación registró «SK = 136 B, índice de 4 bytes»: eso es del
/// conjunto de **árbol único**. Para el elegido son **137 y 5** (§236).
///
/// ⚠️⚠️ **Vive aquí desde §298, y no en el firmante del nodo.** Lo
/// compartible no era el contador sino **el invariante entero**: reservar,
/// comprobar el layout y reconciliar son la misma pieza. Dejarla partida
/// obligaba al TESTIGO a reimplementar la lectura del layout —dos lecturas
/// del mismo formato que pueden discrepar, que es justo lo que §253 evitó
/// reusando este guardián entero— o a firmar sin esa protección, que es el
/// agujero que la nota 92 tiene abierto.
///
/// ⚠️ No arrastra `xmss`: es aritmética de offsets sobre `&[u8]`. Este
/// crate sigue sin una sola dependencia.
pub fn indice_de_sk(sk: &[u8]) -> Result<u64, GuardianError> {
    let esperado = OID_BYTES + ancho_indice() + 4 * N;
    if sk.len() != esperado {
        return Err(GuardianError::LayoutInesperado { sk_len: sk.len(), esperado });
    }
    let mut v = 0u64;
    for b in &sk[OID_BYTES..OID_BYTES + ancho_indice()] {
        v = (v << 8) | *b as u64;
    }
    Ok(v)
}

/// Escribe el indice EN los bytes del SK. Espejo exacto de [`indice_de_sk`]:
/// mismo layout, mismo ancho, mismo orden de bytes.
///
/// ⚠️⚠️ **Es CONSERVADOR, no arriesgado.** El contador se persiste
/// ANTES de firmar, asi que como mucho se gasto la hoja `contador - 1`.
/// Poner la clave en `contador` usa una hoja que NUNCA se reservo: no puede
/// estar quemada. Las hojas de abajo quedan PERDIDAS, que es lo que la nota
/// 92 pide -un indice perdido es mejor que uno indeterminado-.
///
/// ⚠️ Falla CERRADA por el ANCHO DEL CAMPO: el techo es
/// `2^(8 * ancho_indice())`, DERIVADO y no tecleado.
///
/// ⚠️ No es una tercera copia del layout: usa [`ancho_indice`], la misma
/// fuente que el lector.
pub fn poner_indice_en_sk(sk: &mut [u8], indice: u64) -> Result<(), GuardianError> {
    let esperado = OID_BYTES + ancho_indice() + 4 * N;
    if sk.len() != esperado {
        return Err(GuardianError::LayoutInesperado { sk_len: sk.len(), esperado });
    }
    let ancho = ancho_indice();
    if ancho < 8 && indice >= (1u64 << (8 * ancho)) {
        return Err(GuardianError::IndiceFueraDeCampo { indice, ancho });
    }
    for i in 0..ancho {
        let desplazamiento = 8 * (ancho - 1 - i);
        sk[OID_BYTES + i] = ((indice >> desplazamiento) & 0xff) as u8;
    }
    Ok(())
}

/// ⚠️ Mod PROPIO y no dentro del de abajo: un item de Rust empieza en sus
/// ATRIBUTOS, y meter tests entre un `#[test]` y su `fn` los deja huerfanos
/// -el defecto que costo el cierre del S331-.
#[cfg(test)]
mod indice_en_el_sk {
    use super::*;

    fn sk_de_prueba() -> Vec<u8> {
        vec![0u8; OID_BYTES + ancho_indice() + 4 * N]
    }

    /// ⚠️⚠️ Lector y escritor son DOS productores del mismo layout: se
    /// ATAN aqui, no se confia en que coincidan.
    #[test]
    fn lo_que_se_escribe_es_lo_que_se_lee() {
        let ancho = ancho_indice();
        let tope = 1u64 << (8 * ancho);
        for n in [0u64, 1, 2, 255, 256, tope - 1] {
            let mut sk = sk_de_prueba();
            poner_indice_en_sk(&mut sk, n).expect("escribir");
            let leido = indice_de_sk(&sk).expect("leer");
            assert_eq!(leido, n, "el lector no devuelve lo que el escritor puso");
        }
    }

    /// ⚠️⚠️ EL ROJO del techo. El limite se DERIVA de `ancho_indice()`.
    #[test]
    fn un_indice_que_no_cabe_falla_cerrada_y_no_toca_nada() {
        let ancho = ancho_indice();
        let tope = 1u64 << (8 * ancho);
        let mut sk = sk_de_prueba();
        match poner_indice_en_sk(&mut sk, tope) {
            Err(GuardianError::IndiceFueraDeCampo { indice, ancho: a }) => {
                assert_eq!(indice, tope, "el error nombra el indice que no cupo");
                assert_eq!(a, ancho, "y el ancho contra el que no cupo");
            }
            otro => panic!("un indice que no cabe NO puede escribirse: {otro:?}"),
        }
        assert!(sk.iter().all(|b| *b == 0), "y no toca un solo byte al fallar");
    }

    #[test]
    fn un_sk_con_otra_longitud_no_se_toca() {
        let mut corto = vec![0u8; 10];
        assert!(matches!(
            poner_indice_en_sk(&mut corto, 1),
            Err(GuardianError::LayoutInesperado { .. })
        ));
    }
}

#[derive(Debug)]
pub enum GuardianError {
    Io(String),
    /// ⚠️ El indice no cabe en el CAMPO del SK. El ancho lo fija
    /// [`ancho_indice`], y aqui coincide con la altura del arbol porque
    /// h = 40 = 8 x 5; con una `h` que no fuera multiplo de 8 el techo
    /// quedaria laxo por arriba y habria que atarlo a la altura real.
    /// Falla CERRADA: un contador corrupto no se cuela dentro de un SK.
    IndiceFueraDeCampo { indice: u64, ancho: usize },
    /// El fichero existe pero no tiene ocho bytes.
    Corrupto { bytes: usize },
    /// ⚠️ `fsync` no cuesta nada: casi seguro `tmpfs` o un montaje sin
    /// persistencia real. **Operar aquí pondría la clave en riesgo.**
    PersistenciaFalsa { con_fsync_us: f64, sin_fsync_us: f64, razon: f64 },
    /// ⚠️ **El SK no tiene la forma esperada: la serialización de upstream
    /// cambió.** Vive aquí desde §298, con [`indice_de_sk`]: quien custodia
    /// el índice es quien tiene que saber leerlo.
    /// ⚠️ El fichero de la semilla es legible por grupo u otros. Crear con
    /// `0600` no impide que alguien afloje despues: se comprueba AL LEER.
    PermisosAbiertos { ruta: String, modo: u32 },
    /// ⚠️ La semilla en BINARIO crudo no mide lo que debe. `parece_hex`
    /// distingue el error que de verdad comete la gente: darle a `--cofirmar`
    /// el fichero HEX del nodo.
    SemillaLongitud { esperado: usize, encontrado: usize, parece_hex: bool },
    /// ⚠️ La semilla en HEX no mide lo que debe. Se cuentan CARACTERES: un
    /// byte derivado con division entera hacia que 193 dijera «96».
    SemillaHexLongitud { esperado_car: usize, encontrado_car: usize },
    /// La semilla en HEX trae algo que no es un digito hexadecimal.
    SemillaNoHex { detalle: String },
    LayoutInesperado { sk_len: usize, esperado: usize },
}

impl std::fmt::Display for GuardianError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GuardianError::IndiceFueraDeCampo { indice, ancho } => write!(
                f,
                "guardian del indice: el indice {indice} no cabe en un campo \
                 de {ancho} byte(s). El contador esta corrupto o el conjunto \
                 de parametros cambio"
            ),
            GuardianError::Io(e) => write!(f, "guardián del índice: {e}"),
            GuardianError::Corrupto { bytes } => write!(
                f,
                "guardián del índice: el fichero del contador tiene {bytes} bytes, no 8"
            ),
            GuardianError::PersistenciaFalsa { con_fsync_us, sin_fsync_us, razon } => write!(
                f,
                "guardián del índice: `fsync` no persiste nada aquí \
                 ({con_fsync_us:.1} µs con fsync frente a {sin_fsync_us:.1} µs sin él, \
                 razón {razon:.1}×, mínimo {RAZON_MINIMA:.0}×). \
                 Casi seguro es tmpfs o un montaje sin disco. \
                 Reusar un índice XMSS filtra la clave: el nodo NO arranca así."
            ),
            GuardianError::PermisosAbiertos { ruta, modo } => write!(
                f,
                "{ruta} tiene permisos {modo:04o}: es legible por grupo u otros. \
                 Un secreto legible por el grupo es un secreto de todos. `chmod 600`"
            ),
            GuardianError::SemillaLongitud { esperado, encontrado, parece_hex } => {
                write!(
                    f,
                    "la semilla en BINARIO crudo debe tener {esperado} bytes y tiene {encontrado}"
                )?;
                if *parece_hex {
                    write!(
                        f,
                        ". Son {encontrado} caracteres hexadecimales: parece el fichero \
                         HEX del nodo, y este mando quiere los bytes CRUDOS"
                    )?;
                }
                Ok(())
            }
            GuardianError::SemillaHexLongitud { esperado_car, encontrado_car } => write!(
                f,
                "la semilla debe tener {} bytes ({esperado_car} caracteres hex) y tiene \
                 {encontrado_car} caracteres",
                esperado_car / 2
            ),
            GuardianError::SemillaNoHex { detalle } => {
                write!(f, "la semilla no es hexadecimal: {detalle}")
            }
            GuardianError::LayoutInesperado { sk_len, esperado } => write!(
                f,
                "guardián del índice: el SK mide {sk_len} bytes y se esperaban \
                 {esperado}. La serialización de `xmss` cambió: NO se lee el \
                 índice a ciegas."
            ),
        }
    }
}

/// Ver la nota de `firma_cabeza::FirmaError`: **un tipo de error lleva
/// `Debug`, `Display` y `Error` desde que nace**. A éste le faltaba el
/// tercero, y no había fallado todavía solo porque nadie lo había usado
/// con `?` sobre `anyhow` (§241).
impl std::error::Error for GuardianError {}

/// **La semilla del firmante, leida y comprobada en un solo sitio** (§330).
pub mod semilla;

/// Lo que se encuentra al comparar el contador con el índice real de la
/// clave, tras un reinicio.
#[derive(Debug, PartialEq, Eq)]
pub enum Reconciliacion {
    /// Todo cuadra.
    Coincide { indice: u64 },
    /// ⚠️ **El caso normal tras una caída** —13 de 25 en K.1—: se persistió
    /// el índice y el proceso murió antes de firmar. Hay índices
    /// **quemados sin firma**. No es un fallo: es el precio del orden.
    ///
    /// ⚠️ **K.1 midió esto DENTRO de un proceso** -un hijo que persiste y
    /// firma, matado en un instante aleatorio-, **no tras un REINICIO**. Al
    /// reiniciar la clave vuelve a cero y el caso es [`Reconciliacion::ClaveEnCero`].
    /// La cifra es correcta; lo que no cubre es el reinicio.
    ContadorAdelantado { contador: u64, clave: u64, huerfanos: u64 },
    /// ⚠️⚠️ **LO QUE NUNCA DEBE PASAR.** La clave ha firmado con índices
    /// que el contador no registró: o el orden se invirtió, o `fsync` no
    /// hizo lo que dijo. **La clave debe considerarse comprometida.**
    /// ⚠️⚠️ **La clave viene de la semilla y el contador dice que ya se
    /// firmó.** El SK **no se persiste**: al rearrancar, `from_seed` la devuelve
    /// en CERO. No faltan firmas: lo que hay es que **0..contador-1 quedan
    /// INDETERMINADOS**, y volver a firmar los reutilizaría -curva QRL: a la
    /// segunda repetición, ~2^34 hashes-.
    ///
    /// ⚠️ **Falla cerrada, y no por prudencia sino porque no se puede
    /// discriminar**: con contador 1 y clave 0, morir en la ventana de
    /// [`GuardianIndice::reservar`] y morir tras firmar dejan el **mismo estado en
    /// disco**. Es la nota 92: «un índice indeterminado es peor que uno perdido:
    /// invita a reutilizar».
    ClaveEnCero { contador: u64, indeterminados: u64 },
    ClaveAdelantada { contador: u64, clave: u64, sin_registrar: u64 },
}

/// El único caso que **NO ADMITE MATIZ**, con independencia de quién pregunte.
///
/// ⚠️⚠️ **El invariante es del guardián; la política, de cada dueño.** El nodo
/// y el cofirmante deciden cosas distintas ante `ContadorAdelantado` o ante
/// `ClaveEnCero` —y hacen bien: eso es política—, pero ninguno de los dos
/// puede arrancar con la clave por delante del contador. Esa parte no es
/// suya: se decide aquí, en el crate que las dos comparten (§296, §298).
///
/// ⚠️ **La producción no lo llama, y es a propósito.** Para construir su
/// mensaje cada política necesita los CAMPOS de la variante, así que su
/// `match` es inevitable y un `if` delante sugeriría una restricción que no
/// existe. Quien lo consume es **el test de cada crate**, y ese test enumera
/// las variantes con un `match` sin comodín: el día que nazca una quinta,
/// los dos crates dejan de compilar hasta que alguien decida.
pub fn no_admite_matiz(r: &Reconciliacion) -> bool {
    match r {
        Reconciliacion::Coincide { .. } => false,
        Reconciliacion::ContadorAdelantado { .. } => false,
        Reconciliacion::ClaveEnCero { .. } => false,
        Reconciliacion::ClaveAdelantada { .. } => true,
    }
}

#[cfg(test)]
mod invariante_del_arranque {
    use super::*;

    /// Los cuatro valores, escritos como literales y no derivados del propio
    /// `match`: un test que reproduce la implementación no prueba nada.
    #[test]
    fn solo_la_clave_adelantada_no_admite_matiz() {
        assert!(!no_admite_matiz(&Reconciliacion::Coincide { indice: 7 }));
        assert!(!no_admite_matiz(&Reconciliacion::ContadorAdelantado {
            contador: 9,
            clave: 7,
            huerfanos: 2
        }));
        assert!(!no_admite_matiz(&Reconciliacion::ClaveEnCero {
            contador: 5,
            indeterminados: 5
        }));
        assert!(no_admite_matiz(&Reconciliacion::ClaveAdelantada {
            contador: 3,
            clave: 9,
            sin_registrar: 6
        }));
    }
}

/// Contador monótono de índices de firma, persistido antes de cada uso.
///
/// ⚠️ `Debug` porque aparece en un `Result` que los tests inspeccionan con
/// `{:?}`: **si el error lo deriva y el éxito no, el `Result` sigue sin
/// derivarlo**. Es el mismo olvido de §228 con `RpcError`, y la regla que
/// lo evita es mirar LAS DOS mitades del `Result`, no solo la que falla.
#[derive(Debug)]
pub struct GuardianIndice {
    ruta: PathBuf,
    actual: u64,
}

impl GuardianIndice {
    /// Abre —o crea— el contador, **y comprueba que `fsync` persiste de
    /// verdad** en ese sistema de ficheros.
    pub fn abrir(ruta: impl AsRef<Path>) -> Result<Self, GuardianError> {
        let ruta = ruta.as_ref().to_path_buf();
        let carpeta = ruta.parent().unwrap_or(Path::new(".")).to_path_buf();
        std::fs::create_dir_all(&carpeta).map_err(|e| GuardianError::Io(e.to_string()))?;

        Self::comprobar_persistencia(&carpeta)?;

        let actual = if ruta.exists() {
            let mut buf = Vec::new();
            File::open(&ruta)
                .and_then(|mut f| f.read_to_end(&mut buf))
                .map_err(|e| GuardianError::Io(e.to_string()))?;
            if buf.len() != 8 {
                return Err(GuardianError::Corrupto { bytes: buf.len() });
            }
            u64::from_le_bytes(buf.try_into().expect("8 bytes"))
        } else {
            0
        };

        let g = GuardianIndice { ruta, actual };
        // Se escribe el valor de arranque para que el fichero exista y
        // quede sincronizado, incluso si es 0.
        g.persistir(actual)?;
        Ok(g)
    }

    /// ⚠️ **La única función que debe usarse antes de firmar.** Persiste
    /// `actual + 1`, lo devuelve, y **solo entonces** el llamante puede
    /// firmar con él.
    ///
    /// Si el proceso muere entre esta llamada y la firma, el índice queda
    /// **huérfano** — quemado sin firma. Eso es correcto y esperado; lo
    /// resuelve [`Self::reconciliar`].
    pub fn reservar(&mut self) -> Result<u64, GuardianError> {
        let siguiente = self.actual.checked_add(1).ok_or_else(|| {
            GuardianError::Io("el contador de índices se ha desbordado".into())
        })?;
        self.persistir(siguiente)?;
        self.actual = siguiente;
        Ok(siguiente)
    }

    /// El último índice persistido. **Nunca retrocede.**
    pub fn actual(&self) -> u64 {
        self.actual
    }

    /// Compara el contador con el índice que la clave dice tener.
    ///
    /// ⚠️ El índice de la clave **lo lee el llamante**, porque hoy la API
    /// de `xmss` **no lo expone**: hay que interpretar el byte del SK en el
    /// offset del formato de referencia, y en multiárbol ese offset depende
    /// del conjunto (⌈h/8⌉). El issue upstream que pide `index()` está
    /// redactado en `doc/issue-rustcrypto.md` **y sin enviar**.
    pub fn reconciliar(&self, indice_de_la_clave: u64) -> Reconciliacion {
        use std::cmp::Ordering::*;
        match self.actual.cmp(&indice_de_la_clave) {
            Equal => Reconciliacion::Coincide { indice: self.actual },
            Greater if indice_de_la_clave == 0 => Reconciliacion::ClaveEnCero {
                contador: self.actual,
                indeterminados: self.actual,
            },
            Greater => Reconciliacion::ContadorAdelantado {
                contador: self.actual,
                clave: indice_de_la_clave,
                huerfanos: self.actual - indice_de_la_clave,
            },
            Less => Reconciliacion::ClaveAdelantada {
                contador: self.actual,
                clave: indice_de_la_clave,
                sin_registrar: indice_de_la_clave - self.actual,
            },
        }
    }

    fn persistir(&self, valor: u64) -> Result<(), GuardianError> {
        let mut f = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(&self.ruta)
            .map_err(|e| GuardianError::Io(e.to_string()))?;
        f.seek(SeekFrom::Start(0)).map_err(|e| GuardianError::Io(e.to_string()))?;
        f.write_all(&valor.to_le_bytes()).map_err(|e| GuardianError::Io(e.to_string()))?;
        // `sync_all` es `fsync(2)`: datos Y metadatos. Es lo que K.1 midió.
        f.sync_all().map_err(|e| GuardianError::Io(e.to_string()))?;
        Ok(())
    }

    /// Mide `fsync` contra no-`fsync` y decide si este sitio persiste.
    fn comprobar_persistencia(carpeta: &Path) -> Result<(), GuardianError> {
        let prueba = carpeta.join(".guardian-autocomprobacion");
        let medir = |con_fsync: bool| -> Result<f64, GuardianError> {
            let mut f = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&prueba)
                .map_err(|e| GuardianError::Io(e.to_string()))?;
            let t0 = Instant::now();
            for n in 0..MUESTRAS_AUTOCOMPROBACION {
                f.seek(SeekFrom::Start(0)).map_err(|e| GuardianError::Io(e.to_string()))?;
                f.write_all(&(n as u64).to_le_bytes())
                    .map_err(|e| GuardianError::Io(e.to_string()))?;
                if con_fsync {
                    f.sync_all().map_err(|e| GuardianError::Io(e.to_string()))?;
                }
            }
            Ok(t0.elapsed().as_secs_f64() * 1e6 / MUESTRAS_AUTOCOMPROBACION as f64)
        };
        let sin = medir(false)?;
        let con = medir(true)?;
        let _ = std::fs::remove_file(&prueba);

        let razon = if sin > 0.0 { con / sin } else { f64::INFINITY };
        if razon < RAZON_MINIMA && con < SUELO_MICROS {
            return Err(GuardianError::PersistenciaFalsa {
                con_fsync_us: con,
                sin_fsync_us: sin,
                razon,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ⚠️ Los tests que NO son de la autocomprobación fuerzan el sitio de
    // trabajo, porque `std::env::temp_dir()` suele ser **tmpfs** y ahí
    // `abrir` se niega —con razón—. Se usa un directorio bajo el propio
    // árbol del proyecto, que está en disco.
    fn en_disco(nombre: &str) -> PathBuf {
        let d = std::path::Path::new("target").join(format!("guardian_{nombre}"));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).expect("crear");
        d.join("indice.bin")
    }

    #[test]
    fn arranca_en_cero_y_avanza_de_uno_en_uno() {
        let p = en_disco("avanza");
        let mut g = GuardianIndice::abrir(&p).expect("abrir");
        assert_eq!(g.actual(), 0, "un contador nuevo empieza en 0");
        assert_eq!(g.reservar().expect("reservar"), 1);
        assert_eq!(g.reservar().expect("reservar"), 2);
        assert_eq!(g.actual(), 2);
    }

    #[test]
    fn el_contador_sobrevive_al_cierre_y_nunca_retrocede() {
        // ⚠️ El test que da sentido a la pieza: si esto falla, un reinicio
        // reusa indices y **filtra la clave** (§106.4).
        let p = en_disco("sobrevive");
        {
            let mut g = GuardianIndice::abrir(&p).expect("abrir");
            for _ in 0..5 {
                g.reservar().expect("reservar");
            }
            assert_eq!(g.actual(), 5);
        }
        let g2 = GuardianIndice::abrir(&p).expect("reabrir");
        assert_eq!(g2.actual(), 5, "CRITICO: el contador retrocedio al reabrir");
    }

    #[test]
    fn reabrir_muchas_veces_no_pierde_ni_una() {
        let p = en_disco("muchas");
        for esperado in 1..=6u64 {
            let mut g = GuardianIndice::abrir(&p).expect("abrir");
            assert_eq!(g.reservar().expect("reservar"), esperado);
        }
        assert_eq!(GuardianIndice::abrir(&p).expect("abrir").actual(), 6);
    }

    #[test]
    fn el_contador_adelantado_es_el_caso_normal_y_se_reconoce() {
        // K.1: 13 de 25 muertes del proceso dejan el contador por delante.
        let p = en_disco("adelantado");
        let mut g = GuardianIndice::abrir(&p).expect("abrir");
        for _ in 0..7 {
            g.reservar().expect("reservar");
        }
        // La clave solo llego a firmar 5 de los 7 indices reservados.
        assert_eq!(
            g.reconciliar(5),
            Reconciliacion::ContadorAdelantado { contador: 7, clave: 5, huerfanos: 2 }
        );
    }

    #[test]
    fn la_clave_adelantada_se_distingue_y_es_lo_grave() {
        // ⚠️ Esto significa que la clave firmo con indices que el contador
        // no registro: el orden se invirtio o `fsync` mintio.
        let p = en_disco("clave_adelantada");
        let mut g = GuardianIndice::abrir(&p).expect("abrir");
        g.reservar().expect("reservar");
        assert_eq!(
            g.reconciliar(9),
            Reconciliacion::ClaveAdelantada { contador: 1, clave: 9, sin_registrar: 8 }
        );
    }

    #[test]
    fn la_clave_en_cero_con_el_contador_vivo_no_es_un_huerfano() {
        // ⚠⚠ El SK no se persiste: al rearrancar, la clave vuelve a CERO y
        // 0..contador-1 quedan INDETERMINADOS, no huerfanos.
        // ⚠ Se afirma la RELACION, no un numero: el contador se DERIVA de
        // `actual()`. Tecleado, dependeria del estado que traiga el fixture -el
        // r1 murio por eso: pidio 4 y el fichero venia con 4 puestos-.
        let p = en_disco("clave_en_cero");
        let mut g = GuardianIndice::abrir(&p).expect("abrir");
        for _ in 0..4 {
            g.reservar().expect("reservar");
        }
        let n = g.actual();
        assert!(n >= 4, "el contador tiene que haber avanzado y esta en {n}");
        match g.reconciliar(0) {
            Reconciliacion::ClaveEnCero { contador, indeterminados } => {
                assert_eq!(contador, n, "reporta el contador que hay");
                assert_eq!(indeterminados, contador, "TODOS quedan indeterminados");
            }
            otro => panic!("la clave en cero con el contador vivo NO es un huerfano: {otro:?}"),
        }
    }

    #[test]
    fn con_el_contador_en_uno_y_la_clave_en_cero_falla_cerrada() {
        // ⚠⚠ EL CASO QUE NO SE PUEDE DISCRIMINAR: morir en la ventana de
        // `reservar` y morir tras firmar dejan el MISMO estado en disco. Por eso
        // no se adivina: se resuelve por el lado seguro.
        let p = en_disco("cerrada");
        let mut g = GuardianIndice::abrir(&p).expect("abrir");
        let antes = g.actual();
        g.reservar().expect("reservar");
        let n = g.actual();
        assert_eq!(n, antes + 1, "una reserva avanza exactamente uno");
        match g.reconciliar(0) {
            Reconciliacion::ClaveEnCero { contador, indeterminados } => {
                assert_eq!(contador, n);
                assert_eq!(indeterminados, n, "con contador 1 y clave 0 tampoco se adivina");
            }
            otro => panic!("no se adivina: se falla cerrada. Y dio: {otro:?}"),
        }
    }

    #[test]
    fn el_arranque_limpio_no_cae_en_la_variante_nueva() {
        // ⚠ Contador 0 y clave 0 COINCIDEN: la guarda no puede robarle el caso
        // bueno al arranque de siempre.
        let p = en_disco("cero_cero");
        let g = GuardianIndice::abrir(&p).expect("abrir");
        assert_eq!(g.actual(), 0, "el fixture tiene que dar un contador limpio");
        assert_eq!(g.reconciliar(0), Reconciliacion::Coincide { indice: 0 });
    }

    #[test]
    fn coincidir_es_coincidir() {
        let p = en_disco("coincide");
        let mut g = GuardianIndice::abrir(&p).expect("abrir");
        g.reservar().expect("reservar");
        g.reservar().expect("reservar");
        assert_eq!(g.reconciliar(2), Reconciliacion::Coincide { indice: 2 });
    }

    #[test]
    fn un_fichero_de_otro_tamano_se_rechaza_en_vez_de_interpretarse() {
        let p = en_disco("corrupto");
        std::fs::write(&p, b"esto no son ocho bytes").expect("escribir");
        match GuardianIndice::abrir(&p) {
            Err(GuardianError::Corrupto { bytes }) => assert_eq!(bytes, 22),
            otro => panic!("deberia rechazarse por corrupto, y dio: {otro:?}"),
        }
    }

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
            Err(GuardianError::LayoutInesperado { sk_len, esperado }) => {
                assert_eq!(sk_len, 136);
                assert_eq!(esperado, 137, "OID(4) + indice(5) + 4x32 = 137");
            }
            otro => panic!("un SK de 136 bytes debe rechazarse, y dio: {otro:?}"),
        }
    }

    #[test]
    fn en_tmpfs_se_niega_a_operar() {
        // ⚠️ EL TEST QUE JUSTIFICA LA AUTOCOMPROBACION. K.1 midio que en
        // tmpfs `fsync` cuesta lo MISMO que no hacerlo (razon 1x, frente a
        // 382x en ext4): devuelve exito sin persistir nada.
        //
        // Si la maquina de pruebas no tiene /dev/shm ni un temp_dir en
        // tmpfs, el test se salta EN VOZ ALTA en vez de fingir que paso.
        let candidatos = [PathBuf::from("/dev/shm"), std::env::temp_dir()];
        let mut probado = false;
        for base in candidatos {
            if !base.is_dir() {
                continue;
            }
            let salida = std::process::Command::new("df")
                .args(["-T", "--output=fstype"])
                .arg(&base)
                .output();
            let es_tmpfs = match salida {
                Ok(o) => String::from_utf8_lossy(&o.stdout).contains("tmpfs"),
                Err(_) => false,
            };
            if !es_tmpfs {
                continue;
            }
            probado = true;
            let d = base.join(format!("guardian_tmpfs_{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&d);
            std::fs::create_dir_all(&d).expect("crear");
            let r = GuardianIndice::abrir(d.join("indice.bin"));
            let _ = std::fs::remove_dir_all(&d);
            match r {
                Err(GuardianError::PersistenciaFalsa { razon, .. }) => {
                    assert!(razon < RAZON_MINIMA, "razon {razon} deberia estar bajo el minimo");
                }
                otro => panic!(
                    "en {} —que es tmpfs— el guardian DEBE negarse, y dio: {otro:?}",
                    base.display()
                ),
            }
            break;
        }
        if !probado {
            eprintln!(
                "AVISO: no se encontro ningun tmpfs donde probar el rechazo. \
                 La autocomprobacion NO se ha ejercitado en este entorno."
            );
        }
    }

    #[test]
    fn en_disco_de_verdad_si_opera() {
        // La otra mitad: donde `fsync` cuesta, el guardian arranca.
        let p = en_disco("disco_real");
        GuardianIndice::abrir(&p).expect("en disco real el guardian debe arrancar");
    }
}
