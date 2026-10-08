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
//! Por eso [`GuardianIndice::abrir`] **se niega a operar** donde ve que
//! `fsync` no persiste, con dos redes (§740):
//!
//! 1. **En Linux, el tipo del sistema de ficheros**, el que da
//!    `/proc/self/mountinfo` para el montaje del descriptor: `tmpfs`, `ramfs`,
//!    `devtmpfs` y `rootfs` viven en memoria, y se rechazan sin medir nada
//!    ([`GuardianError::SistemaEnMemoria`]). Se mira dos veces: en la carpeta,
//!    con el fichero de prueba, y en el fichero del contador ya abierto, que
//!    puede ser un montaje propio (`mount --bind` de un fichero).
//! 2. **El coste de `fsync`**, medido al arrancar en la carpeta: la mediana por
//!    escritura con `fsync` frente a sin él ([`GuardianError::PersistenciaFalsa`]).
//!    Cubre lo que la primera no alcanza: fuera de Linux o sin `/proc`, y un
//!    sistema que no se llama como los de la lista y donde `fsync` cuesta lo
//!    mismo que no hacerlo (un `overlay` con la capa de arriba en `tmpfs`: 22 000
//!    de 22 000 aperturas negadas en el contenedor de §740).
//!
//! Lo que ninguna de las dos ve —derivado, no medido—: fuera de Linux o sin
//! `/proc`, un contador que sea un montaje propio en memoria, porque la segunda
//! mide la carpeta y no el fichero; en Linux, ese montaje propio si su sistema
//! no se llama como los de la lista; y un sistema de red o de paso —NFS, 9p,
//! `virtiofs`, FUSE— cuyo otro lado viva en memoria, donde `fsync` cuesta un
//! viaje y pasa el suelo.
//!
//! ⚠️ **Hasta §740 sólo existía la segunda, y con la media de veinte
//! escrituras**, y fallaba ABIERTA, en reposo y con carga: para negarse hacían
//! falta la razón bajo el mínimo **y** el coste bajo el suelo, y una sola
//! escritura lenta sacaba la media de uno de los dos (con las cifras de
//! `/dev/shm`, bastaba una de 200 µs). Medido con `GuardianIndice::abrir` en el
//! contenedor de §740 (Linux, 4 CPU, `/dev/shm` en tmpfs): de 2 a 11 aperturas
//! de cada 2000 arrancaban, en reposo como con cuatro bucles ocupando las CPU,
//! y 265 de 76 000 mientras corría la suite de `zk-ssl-cli`; la escritura más
//! lenta fue de 995 µs, y en 2000 rondas la media llegó a 51,2 µs y la mediana
//! no pasó de 3,1 µs. Por qué hay escrituras lentas no está medido. Con §740,
//! esas aperturas se niegan por el tipo, 8000 de 8000; y la mediana sola, con
//! el tipo apagado en una copia, no dejó arrancar ninguna de 538 000.
//!
//! ⚠️ **Los umbrales salen de UNA máquina** —WSL2 sobre un i5-1135G7—, con
//! medias, y están declarados, no derivados; con medianas sólo se han medido en
//! el contenedor de §740. Un NVMe rápido puede dar `fsync` de ~100 µs
//! legítimos; por eso el discriminante principal es la **razón** contra no
//! persistir, no el valor absoluto. Con la mediana, un disco con `fsync` de 10
//! a 20 µs queda más cerca de los dos umbrales que con la media, que arrastraba
//! sus escrituras lentas: si se niega, se niega de más, que es el lado seguro.
//! No se ha medido.
//!
//! ## ⚠️ Lo que esto NO garantiza
//!
//! **Nada frente a un corte de corriente.** *«`fsync` puede mentir»* habla
//! de discos que confirman escrituras que siguen en caché volátil. K.1
//! midió el **orden** frente a la **muerte del proceso** —25 de 25 sin una
//! sola firma por delante—, y eso **no es durabilidad**: lo escrito sin
//! `fsync` también sobrevive a la muerte del proceso, porque queda en la
//! caché del núcleo y no en la del proceso (§709 corrige aquí «midió
//! durabilidad»). Medir la durabilidad exige cortar la corriente de verdad,
//! y no se ha hecho.
//!
//! ## ⚠️ Un solo proceso por contador (§709, SEC-1)
//!
//! [`GuardianIndice::abrir`] toma un **cerrojo exclusivo** sobre el fichero
//! del contador ([`File::try_lock`], Rust 1.89) **antes** de leerlo y de
//! reescribirlo, y lo guarda en un campo mientras viva el guardián. Un
//! segundo guardián sobre el mismo fichero —en otro proceso o en este— **no
//! abre**, y el error dice qué proceso lo tiene, si el sistema lo dice. Cubre
//! a la vez la firma de cabeza, la recepción y el cofirmante del testigo,
//! porque los tres abren aquí. El cerrojo muere con el proceso, también con
//! `kill -9`. Cada reserva escribe y sincroniza **por ese mismo descriptor**,
//! sin abrir otro, y antes, en unix, comprueba que la ruta sigue siendo ese
//! fichero: si se borró, se movió o se cambió por otro, **no reserva**.
//!
//! **Lo que el cerrojo NO cubre**, y no es un descuido sino lo que un cerrojo
//! de aviso puede ver:
//!
//! - un sistema de ficheros de red (NFS y parecidos), donde el cerrojo puede
//!   no llegar de una máquina a otra;
//! - otra máquina, o una copia del contador en otro disco;
//! - **la misma semilla con otro contador**: dos ficheros distintos son dos
//!   cerrojos distintos, y los dos firmantes reutilizan hojas igual;
//! - un proceso que abra el fichero sin pedir el cerrojo: en unix es de
//!   aviso, no obligatorio.
//!
//! Y si el fichero del contador es un **enlace simbólico**, no se sigue: se
//! rechaza ([`GuardianError::EnlaceSimbolico`]). La carpeta sí se
//! canonicaliza, y la comprobación de `fsync` se hace en la de verdad. Un
//! enlace en el fichero podría llevar los ocho bytes a un sitio que no
//! persiste mientras la carpeta del enlace, que es la que se medía, sí. En
//! unix el rechazo se comprueba dos veces: antes de abrir, y después, sobre
//! lo abierto —el dispositivo y el inodo de la ruta, sin seguir el enlace,
//! tienen que ser los del descriptor—, porque `open` sigue los enlaces y
//! entre las dos cosas alguien con permiso en la carpeta podría poner uno.
//! Fuera de unix, sólo antes.
//!
//! ## ⚠️ Y esta pieza NO tiene consumidor todavía
//!
//! Es el **eslabón 2 de cinco** (`BACKLOG.md`, «la cadena de la
//! oponibilidad»); el 3 —la cabeza firmada, emitida— no existe. Se
//! construye antes a propósito, porque es la pieza más difícil de
//! retroadaptar. **El riesgo está declarado**: se diseña una API sin su
//! consumidor.

use std::fs::{File, OpenOptions, TryLockError};
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
// La mediana de una muestra vacía no existe: que no compile.
const _: () = assert!(MUESTRAS_AUTOCOMPROBACION > 0);

/// ⚠️ **Umbral DECLARADO, no derivado.** `fsync` tiene que costar al menos
/// esta razón frente a escribir sin persistir. Medido en K.1: ext4 dio
/// **382×** y tmpfs **1×**, así que 10 separa los dos casos con 38 veces de
/// margen por el lado bueno (decía «dos órdenes»; corregido en el §740).
const RAZON_MINIMA: f64 = 10.0;

/// Suelo absoluto de la segunda red: sólo se niega si, además de la razón, la
/// mediana con `fsync` queda por debajo. Un NVMe rápido hace `fsync` en
/// ~100 µs legítimos, así que esto se queda muy por debajo: solo caza el
/// caso «no hay disco».
const SUELO_MICROS: f64 = 20.0;

/// ⚠️ §740 · Los sistemas de ficheros que viven en memoria: `fsync` devuelve
/// éxito en ellos y nada sobrevive a un apagado. `devtmpfs` es el `tmpfs` de
/// `/dev`, y `rootfs`, el del arranque (el initramfs), que es `ramfs` o
/// `tmpfs`. La lista es declarada, no exhaustiva: un `overlay` cuya capa de
/// arriba esté en memoria no se llama así, y ése lo coge, si lo coge, la medida
/// de `fsync` (22 000 de 22 000 en el contenedor de §740).
const SISTEMAS_EN_MEMORIA: [&str; 4] = ["tmpfs", "ramfs", "devtmpfs", "rootfs"];

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
    /// ⚠️ `fsync` no cuesta nada: un montaje sin persistencia real. **Operar
    /// aquí pondría la clave en riesgo.** En Linux, `tmpfs` y los demás de
    /// [`SISTEMAS_EN_MEMORIA`] se rechazan antes por su nombre
    /// ([`Self::SistemaEnMemoria`]); aquí llega lo que no se llama así —un
    /// `overlay` con la capa de arriba en memoria— y, fuera de Linux o sin
    /// `/proc`, el propio `tmpfs`. Los dos tiempos son MEDIANAS por escritura
    /// desde §740; hasta entonces, medias. Si la mediana sin `fsync` es 0, la
    /// razón es infinita y decide el suelo.
    PersistenciaFalsa { con_fsync_us: f64, sin_fsync_us: f64, razon: f64 },
    /// ⚠️ §740 · El fichero vive en un sistema de ficheros **en memoria**
    /// ([`SISTEMAS_EN_MEMORIA`]), según `/proc/self/mountinfo`: `fsync`
    /// devuelve éxito sin persistir nada, y el contador se pierde al apagar.
    /// Sólo en Linux, y sin medir nada. `ruta` es lo que se miró: la carpeta,
    /// o el fichero del contador si es él el que está en memoria.
    SistemaEnMemoria { ruta: String, tipo: String },
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
    /// ⚠️ §709 · SEC-1: **otro guardián tiene ya este contador**, en otro
    /// proceso o en este. Dos procesos sobre un mismo contador reparten los
    /// mismos números: en el índice de firma, la misma hoja XMSS para dos
    /// mensajes, y eso compromete la clave. `pid` y `nombre` dicen quién lo
    /// tiene cuando el sistema lo dice (en Linux, `/proc/locks`); `None` si no.
    ContadorOcupado { ruta: String, pid: Option<u32>, nombre: Option<String> },
    /// ⚠️ §709 · La ruta del contador es un **enlace simbólico**. No se sigue:
    /// el cerrojo, la comprobación de `fsync` y el nombre tienen que ser los
    /// del fichero de verdad, y un enlace puede llevarlo a un sitio que no
    /// persiste mientras la carpeta del enlace sí lo hace.
    EnlaceSimbolico { ruta: String, destino: String },
    /// ⚠️ §709 · **Al firmar**, la ruta de un fichero de estado falta o es
    /// relativa. Sin bandera, el fichero lo elegía el directorio desde el que
    /// se lanzaba el proceso; con una ruta relativa, también. `sugerida` es la
    /// absoluta que la relativa nombra desde aquí, si se puede saber.
    RutaDeEstado { bandera: String, ruta: Option<String>, sugerida: Option<String> },
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
            GuardianError::PersistenciaFalsa { con_fsync_us, sin_fsync_us, razon } => {
                write!(
                    f,
                    "guardián del índice: `fsync` no persiste nada aquí \
                     ({con_fsync_us:.1} µs con fsync frente a {sin_fsync_us:.1} µs sin él, \
                     mediana por escritura, "
                )?;
                if razon.is_finite() {
                    write!(
                        f,
                        "razón {razon:.1}×, mínimo {RAZON_MINIMA:.0}×, y suelo \
                         {SUELO_MICROS:.0} µs"
                    )?;
                } else {
                    write!(
                        f,
                        "sin razón: el reloj no midió la escritura sin fsync, y decide \
                         el suelo de {SUELO_MICROS:.0} µs"
                    )?;
                }
                write!(
                    f,
                    "). Casi seguro es memoria —tmpfs, o un overlay \
                     con la capa de arriba en tmpfs— o un montaje sin disco. \
                     Reusar un índice XMSS filtra la clave: el nodo NO arranca así."
                )
            }
            GuardianError::SistemaEnMemoria { ruta, tipo } => write!(
                f,
                "guardián del índice: {ruta} está en un sistema de ficheros `{tipo}`, \
                 que vive en memoria: `fsync` devuelve éxito sin persistir nada, y el \
                 contador se pierde al apagar. Reusar un índice XMSS filtra la clave: \
                 el nodo NO arranca así. Pon el contador en un disco"
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
            GuardianError::ContadorOcupado { ruta, pid, nombre } => {
                write!(f, "guardián del índice: el contador {ruta} ya lo tiene abierto ")?;
                match (pid, nombre.as_deref()) {
                    (Some(p), Some(n)) if !n.is_empty() => write!(f, "el proceso {p} ({n})")?,
                    (Some(p), _) => write!(f, "el proceso {p}")?,
                    (None, _) => write!(f, "otro proceso (este sistema no dice cuál)")?,
                }
                write!(
                    f,
                    ". Dos procesos sobre un mismo contador reparten los mismos números: \
                     en el índice de firma, la misma hoja XMSS para dos mensajes, y eso \
                     compromete la clave. NO se arranca. Si es un proceso anterior que \
                     sigue vivo, páralo antes; si es otro firmante, necesita su propio \
                     contador y su propia semilla: la misma semilla con otro contador \
                     también reutiliza hojas, y eso el cerrojo no lo ve"
                )
            }
            GuardianError::EnlaceSimbolico { ruta, destino } => write!(
                f,
                "guardián del índice: {ruta} es un enlace simbólico (a {destino}), y el \
                 contador no se sigue por un enlace: el cerrojo y la comprobación de \
                 `fsync` tienen que ser los del fichero de verdad. Pasa la ruta del destino"
            ),
            GuardianError::RutaDeEstado { bandera, ruta: None, .. } => write!(
                f,
                "al firmar, {bandera} es obligatoria y no tiene valor por defecto: nombra \
                 con su ruta ABSOLUTA el fichero de este contador. Si ya firmabas sin \
                 ella, el contador está en el directorio desde el que lanzabas el \
                 proceso: pasa ESE fichero, no uno nuevo, porque un contador nuevo \
                 empieza en 0"
            ),
            GuardianError::RutaDeEstado { bandera, ruta: Some(r), sugerida } => {
                write!(
                    f,
                    "al firmar, {bandera} tiene que ser una ruta ABSOLUTA, y `{r}` es \
                     relativa: el fichero dependería del directorio desde el que se lance \
                     el proceso, y lanzado desde otro, el mismo firmante abriría otro \
                     contador, que empieza en 0"
                )?;
                if let Some(s) = sugerida {
                    write!(f, ". Desde aquí es `{s}`: si es el que venías usando, pásalo así")?;
                }
                Ok(())
            }
        }
    }
}

/// Ver la nota de `firma_cabeza::FirmaError`: **un tipo de error lleva
/// `Debug`, `Display` y `Error` desde que nace**. A éste le faltaba el
/// tercero, y no había fallado todavía solo porque nadie lo había usado
/// con `?` sobre `anyhow` (§241).
impl std::error::Error for GuardianError {}

/// ⚠️ §709 · **Al firmar, el fichero de un contador se nombra entero.** La regla
/// que comparten el nodo (`--indice-firma`, `--contador-recepcion`) y el
/// cofirmante del testigo (`--indice-cofirma`): la bandera es obligatoria, sin
/// valor por defecto, y su ruta tiene que ser absoluta.
///
/// ⚠️ Vive aquí y no en cada binario por la razón del §296: dos copias de la
/// misma regla pueden discrepar, y aquí discrepar es que un firmante la cumpla y
/// el otro no. Y va en la FRONTERA de la línea de órdenes, no dentro de
/// [`GuardianIndice::abrir`]: los nodos que no firman, y los tests, abren sus
/// contadores con rutas relativas, y eso no compromete ninguna clave.
///
/// ⚠️ Lo que cierra es el accidente: dos procesos lanzados desde el mismo
/// directorio con el valor por defecto comparten contador, y uno lanzado desde
/// otro abre un contador nuevo, en 0. **No cierra** la misma semilla con dos
/// rutas absolutas distintas: eso es otro fichero, y otro cerrojo.
pub fn exigir_ruta_absoluta(bandera: &str, ruta: Option<&Path>) -> Result<PathBuf, GuardianError> {
    match ruta {
        None => Err(GuardianError::RutaDeEstado {
            bandera: bandera.to_string(),
            ruta: None,
            sugerida: None,
        }),
        Some(r) if r.is_absolute() => Ok(r.to_path_buf()),
        Some(r) => Err(GuardianError::RutaDeEstado {
            bandera: bandera.to_string(),
            ruta: Some(r.display().to_string()),
            sugerida: std::env::current_dir().ok().map(|d| d.join(r).display().to_string()),
        }),
    }
}

#[cfg(test)]
mod ruta_al_firmar {
    use super::*;

    #[test]
    fn sin_bandera_no_hay_valor_por_defecto() {
        match exigir_ruta_absoluta("--indice-firma", None) {
            Err(e @ GuardianError::RutaDeEstado { ruta: None, .. }) => {
                let m = e.to_string();
                assert!(m.contains("--indice-firma"), "nombra la bandera: {m}");
                assert!(m.contains("obligatoria"), "dice que es obligatoria: {m}");
            }
            otro => panic!("sin bandera, al firmar, no hay contador por defecto: {otro:?}"),
        }
    }

    /// ⚠️ La relativa se rechaza aunque el fichero exista, y el error da la absoluta que
    /// nombraba desde aquí: el operador que la venía usando pasa ESA, no una nueva.
    #[test]
    fn una_ruta_relativa_se_rechaza_y_se_dice_cual_era() {
        for r in ["recepcion.bin", "./contador.bin", "../x/indice.bin", ""] {
            match exigir_ruta_absoluta("--contador-recepcion", Some(Path::new(r))) {
                Err(e @ GuardianError::RutaDeEstado { ruta: Some(_), .. }) => {
                    let m = e.to_string();
                    assert!(m.contains("relativa"), "{r}: {m}");
                    if let GuardianError::RutaDeEstado { sugerida: Some(s), .. } = &e {
                        assert!(Path::new(s).is_absolute(), "{r}: la sugerida es absoluta: {s}");
                    }
                }
                otro => panic!("{r:?} es relativa y se acepto: {otro:?}"),
            }
        }
    }

    #[test]
    fn una_ruta_absoluta_pasa_tal_cual() {
        let r = Path::new("/var/lib/arqueo/indice-firma.bin");
        assert_eq!(exigir_ruta_absoluta("--indice-cofirma", Some(r)).expect("absoluta"), r);
    }
}

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
    /// ⚠️ §709 · SEC-1 — **EL CERROJO**: el fichero del contador, abierto y
    /// bloqueado en exclusiva con [`File::try_lock`] en [`Self::abrir`], antes de
    /// leer el valor y de reescribirlo. Vive en un campo porque el cerrojo dura
    /// lo que dura su `File`. Se lee y se escribe por él: [`Self::persistir`] no
    /// abre otro descriptor. Se suelta al soltar el guardián, o cuando muere el
    /// proceso. (El nombre, con `_`, es el de SEC-1 en `doc/blueprint-v2.md`.)
    _cerrojo: File,
}

impl GuardianIndice {
    /// Abre —o crea— el contador, **y comprueba que `fsync` persiste de
    /// verdad** en ese sistema de ficheros.
    ///
    /// ⚠️ §709, en este orden: se rechaza un enlace simbólico; la autocomprobación
    /// mide la carpeta CANÓNICA, la del fichero de verdad; se abre el contador y se
    /// mira su sistema de ficheros (§740); se toma el cerrojo; y sólo entonces se
    /// lee y se reescribe el valor. Un contador recién creado se persiste con
    /// `fsync` del fichero y de su carpeta, para que su NOMBRE también sobreviva a
    /// un corte.
    pub fn abrir(ruta: impl AsRef<Path>) -> Result<Self, GuardianError> {
        let io = |e: std::io::Error| GuardianError::Io(e.to_string());
        let pedida = ruta.as_ref();
        let nombre = pedida.file_name().ok_or_else(|| {
            GuardianError::Io(format!("{} no nombra un fichero", pedida.display()))
        })?;
        // ⚠️ `Path::new("recepcion.bin").parent()` es `Some("")`, no `None`.
        let carpeta = match pedida.parent() {
            Some(c) if !c.as_os_str().is_empty() => c,
            _ => Path::new("."),
        };
        std::fs::create_dir_all(carpeta).map_err(io)?;

        // ── §709 (3) · un enlace simbólico NO se sigue ──
        let existia = match std::fs::symlink_metadata(pedida) {
            Ok(m) if m.file_type().is_symlink() => {
                return Err(GuardianError::EnlaceSimbolico {
                    ruta: pedida.display().to_string(),
                    destino: std::fs::read_link(pedida)
                        .map(|d| d.display().to_string())
                        .unwrap_or_else(|e| format!("ilegible: {e}")),
                });
            }
            Ok(_) => true,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => false,
            Err(e) => return Err(io(e)),
        };
        // ⚠️ La carpeta CANÓNICA: la autocomprobación se hace donde vive de verdad
        //    el fichero, y los mensajes dicen esa ruta.
        let carpeta = std::fs::canonicalize(carpeta).map_err(io)?;
        let ruta = carpeta.join(nombre);

        Self::comprobar_persistencia(&carpeta)?;

        // ── §709 (1) · EL CERROJO, antes de leer y de reescribir ──
        let cerrojo = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&ruta)
            .map_err(io)?;
        // ⚠️ El rechazo del enlace, otra vez y sobre lo que se ABRIÓ: `open` sigue los
        //    enlaces, y entre el `symlink_metadata` de arriba y él alguien con permiso
        //    en la carpeta pudo poner uno. (Si apuntaba a un fichero que no existía,
        //    este `open` lo ha creado vacío; se rechaza igual, y no se usa.)
        if !sigue_siendo_el_mismo(&ruta, &cerrojo).map_err(io)? {
            return Err(match std::fs::read_link(&ruta) {
                Ok(d) => GuardianError::EnlaceSimbolico {
                    ruta: ruta.display().to_string(),
                    destino: d.display().to_string(),
                },
                Err(_) => GuardianError::Io(format!(
                    "{} cambió de fichero mientras se abría: no se usa",
                    ruta.display()
                )),
            });
        }
        // ⚠️ §740 · Y el sistema de ficheros, otra vez y sobre lo ABIERTO: la
        //    autocomprobación miró la carpeta, y el contador puede ser un montaje
        //    propio (`mount --bind` de un fichero de `/dev/shm`), que no es un enlace
        //    y que nada de lo de arriba ve.
        rechazar_si_en_memoria(&cerrojo, &ruta)?;
        match cerrojo.try_lock() {
            Ok(()) => {}
            Err(TryLockError::WouldBlock) => {
                let (pid, nombre) = quien_lo_tiene(&cerrojo);
                return Err(GuardianError::ContadorOcupado {
                    ruta: ruta.display().to_string(),
                    pid,
                    nombre,
                });
            }
            Err(TryLockError::Error(e)) => return Err(io(e)),
        }

        // Se lee por el MISMO descriptor que tiene el cerrojo.
        let mut buf = Vec::new();
        (&cerrojo).read_to_end(&mut buf).map_err(io)?;
        let actual = match buf.len() {
            8 => u64::from_le_bytes(buf.try_into().expect("8 bytes")),
            // Lo acaba de crear este `open`: un contador nuevo empieza en 0.
            0 if !existia => 0,
            n => return Err(GuardianError::Corrupto { bytes: n }),
        };

        let g = GuardianIndice { ruta, actual, _cerrojo: cerrojo };
        // Se escribe el valor de arranque para que el fichero exista y
        // quede sincronizado, incluso si es 0.
        g.persistir(actual)?;
        // ── §709 (4) · el NOMBRE del fichero nuevo, persistido ──
        // ⚠️ El molde de `registro_recepcion.rs` (`anotar`): sin el `fsync` de la
        //    carpeta, un corte puede dejar los ocho bytes en disco y el nombre no.
        if !existia {
            File::open(&carpeta).and_then(|d| d.sync_all()).map_err(io)?;
        }
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

    /// ⚠️ §690 · **Adelanta el contador hasta `hasta` sin firmar nada**, con `fsync`: las hojas de
    /// en medio quedan QUEMADAS, sin firma —perdidas, nunca indeterminadas—. Si ya está ahí o por
    /// encima, no hace nada: **nunca retrocede**. Es el salto del `desde` de la D-G del RFC-0015:
    /// rotar por encima de toda hoja que la clave que se va pudo firmar.
    pub fn adelantar(&mut self, hasta: u64) -> Result<(), GuardianError> {
        if hasta > self.actual {
            self.persistir(hasta)?;
            self.actual = hasta;
        }
        Ok(())
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

    /// ⚠️ §709 · Escribe **por el descriptor que tiene el cerrojo**, sin abrir la
    /// ruta otra vez: así el cerrojo no depende de lo que el sistema haga con otro
    /// descriptor del mismo fichero —donde el cerrojo es obligatorio, como el de
    /// `LockFileEx` en Windows, escribir por otro chocaría con él—, y una reserva
    /// cuesta un `open` menos.
    ///
    /// ⚠️ Y antes comprueba que la ruta **sigue siendo ese fichero**. Si se borró,
    /// se movió o se cambió por otro, NO reserva: escribir en un fichero que ya no
    /// tiene nombre es perder la cuenta al reiniciar, y abrir otro por la ruta
    /// sería un contador sin cerrojo.
    fn persistir(&self, valor: u64) -> Result<(), GuardianError> {
        let io = |e: std::io::Error| GuardianError::Io(e.to_string());
        if !sigue_siendo_el_mismo(&self.ruta, &self._cerrojo).map_err(io)? {
            return Err(GuardianError::Io(format!(
                "el contador {} ya no es el fichero que este guardián tiene abierto y \
                 bloqueado: se borró, se movió o se cambió por otro. NO se reserva",
                self.ruta.display()
            )));
        }
        let mut f = &self._cerrojo;
        f.seek(SeekFrom::Start(0)).map_err(io)?;
        f.write_all(&valor.to_le_bytes()).map_err(io)?;
        // `sync_all` es `fsync(2)`: datos Y metadatos. Es lo que K.1 midió.
        f.sync_all().map_err(io)?;
        Ok(())
    }

    /// Decide si este sitio persiste, con las dos redes de la doc del crate
    /// (§740): primero el sistema de ficheros de un fichero de prueba en la
    /// carpeta, y sólo si no lo descarta, el coste de `fsync` en él.
    fn comprobar_persistencia(carpeta: &Path) -> Result<(), GuardianError> {
        comprobar_persistencia_con(carpeta, medir)
    }
}

/// ⚠️ §740 · [`GuardianIndice::comprobar_persistencia`] con la medida como
/// argumento, para que un test pueda darle tiempos de `tmpfs` en un disco y ver
/// que la segunda red, cableada, se niega: sin eso, cambiar la llamada a
/// [`decidir`] por un `Ok` no lo cazaba ningún test, porque en Linux la primera
/// red llega antes.
///
/// ⚠️ El fichero de prueba lleva un nombre propio —el PID, la hora en ns y una
/// serie— y se crea con `create_new` (`O_CREAT|O_EXCL`), que no sigue un enlace
/// simbólico ni abre lo que ya existe. Hasta §740 se llamaba siempre
/// `.guardian-autocomprobacion` y se abría con `create` y `truncate`: un enlace
/// puesto ahí por quien pudiera escribir en la carpeta se seguía —se medía otro
/// sitio, y se truncaba el fichero al que apuntara—, y dos guardianes que se
/// comprobaran a la vez en la misma carpeta compartían el fichero. Se borra en
/// todos los casos; si el proceso muere en medio, queda uno, con su nombre.
fn comprobar_persistencia_con(
    carpeta: &Path,
    mut medir: impl FnMut(&mut File, bool) -> Result<Vec<f64>, GuardianError>,
) -> Result<(), GuardianError> {
    static SERIE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let ns = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let serie = SERIE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let prueba =
        carpeta.join(format!(".guardian-autocomprobacion-{}-{ns}-{serie}", std::process::id()));
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&prueba)
        .map_err(|e| GuardianError::Io(format!("{}: {e}", prueba.display())))?;
    let veredicto = rechazar_si_en_memoria(&f, carpeta).and_then(|()| {
        let sin = medir(&mut f, false)?;
        let con = medir(&mut f, true)?;
        decidir(&con, &sin)
    });
    drop(f);
    let _ = std::fs::remove_file(&prueba);
    veredicto
}

/// ⚠️ §740 · **La primera red**: si el descriptor está en un sistema de ficheros
/// en memoria, según `/proc/self/mountinfo`, se rechaza, y el error nombra
/// `ruta`. Si el sistema no lo dice —fuera de Linux, sin `/proc`, un núcleo sin
/// `mnt_id` en `fdinfo`—, no rechaza. En la carpeta decide después la segunda
/// red; en el fichero del contador no hay otra, y un montaje propio en memoria
/// pasa.
fn rechazar_si_en_memoria(f: &File, ruta: &Path) -> Result<(), GuardianError> {
    match sistema_de_ficheros(f) {
        Some(tipo) if SISTEMAS_EN_MEMORIA.contains(&tipo.as_str()) => {
            Err(GuardianError::SistemaEnMemoria { ruta: ruta.display().to_string(), tipo })
        }
        _ => Ok(()),
    }
}

/// ⚠️ §740 · Lo que tarda cada una de las [`MUESTRAS_AUTOCOMPROBACION`]
/// escrituras de ocho bytes, en µs, con `fsync` detrás de cada una o sin él.
/// Una cifra por escritura, y no el total, porque lo que se compara es la
/// mediana.
fn medir(f: &mut File, con_fsync: bool) -> Result<Vec<f64>, GuardianError> {
    let io = |e: std::io::Error| GuardianError::Io(e.to_string());
    let mut tiempos = Vec::with_capacity(MUESTRAS_AUTOCOMPROBACION as usize);
    for n in 0..MUESTRAS_AUTOCOMPROBACION {
        let t0 = Instant::now();
        f.seek(SeekFrom::Start(0)).map_err(io)?;
        f.write_all(&(n as u64).to_le_bytes()).map_err(io)?;
        if con_fsync {
            f.sync_all().map_err(io)?;
        }
        tiempos.push(t0.elapsed().as_secs_f64() * 1e6);
    }
    Ok(tiempos)
}

/// ⚠️ §740 · **La segunda red**, sin reloj para poder probarla: se niega si la
/// mediana con `fsync` cuesta menos de [`RAZON_MINIMA`] veces la mediana sin él
/// **y** menos de [`SUELO_MICROS`] µs. Los umbrales son los de siempre; lo que
/// cambia en §740 es la mediana en lugar de la media. Con la media, una sola
/// escritura lenta bastaba para arrancar en `tmpfs`. La mediana de veinte es la
/// media de la décima y la undécima: mientras once sean rápidas sale de ellas, y
/// nueve lentas no la mueven; con diez, arranca si son lo bastante lentas.
///
/// ⚠️ Si la mediana sin `fsync` es 0 —un reloj más grueso que una escritura, que
/// es más probable ahora que se mide cada una—, la razón no dice nada y decide el
/// suelo solo. Con la razón infinita, eso arrancaba.
fn decidir(con: &[f64], sin: &[f64]) -> Result<(), GuardianError> {
    let (con, sin) = (mediana(con), mediana(sin));
    let razon = if sin > 0.0 { con / sin } else { f64::INFINITY };
    if con < SUELO_MICROS && (sin <= 0.0 || razon < RAZON_MINIMA) {
        return Err(GuardianError::PersistenciaFalsa {
            con_fsync_us: con,
            sin_fsync_us: sin,
            razon,
        });
    }
    Ok(())
}

/// La mediana de una muestra no vacía: con un número par de valores, la media de
/// los dos de en medio.
fn mediana(muestra: &[f64]) -> f64 {
    let mut v = muestra.to_vec();
    v.sort_by(f64::total_cmp);
    let m = v.len() / 2;
    if v.len().is_multiple_of(2) {
        (v[m - 1] + v[m]) / 2.0
    } else {
        v[m]
    }
}

/// ⚠️ §709 · **Quién tiene el cerrojo**: el proceso y su nombre, si el sistema lo
/// dice. En Linux, `/proc/locks` da una línea por cerrojo, con el PID de quien lo
/// tomó y el fichero como `MAYOR:MENOR:INODO` (los dos primeros en hexadecimal):
/// `1: FLOCK  ADVISORY  WRITE 2051 fe:00:624583 0 EOF`. Las líneas con `->` son
/// procesos que esperan, no el que lo tiene.
///
/// ⚠️ Del proceso se da el NOMBRE (`/proc/PID/comm`), nunca la línea de órdenes:
/// la del nodo puede llevar la semilla (`--clave`), y esto acaba en un log.
///
/// ⚠️ El `MAYOR:MENOR` de `/proc/locks` es el del SUPERBLOQUE del sistema de
/// ficheros, y el `st_dev` de `fstat` no siempre lo es: en btrfs es el del
/// subvolumen, y en un overlay cuyas capas están en sistemas distintos, el de la
/// capa. Por eso el fichero se busca con los dos: el de `fstat` y el del montaje
/// del descriptor (su `mnt_id` en `/proc/self/fdinfo`, y el `MAYOR:MENOR` de ese
/// montaje en `/proc/self/mountinfo`, que es el del superbloque). Si casa más de
/// un proceso —en btrfs, dos subvolúmenes comparten superbloque y pueden repetir
/// inodo—, no se da ninguno.
///
/// ⚠️ Es la mejor información disponible, no una prueba: entre el `WouldBlock` y
/// esta lectura el cerrojo puede cambiar de manos, y en otro espacio de PID el
/// número puede no ser visible (sale 0, y se da como desconocido).
#[cfg(target_os = "linux")]
fn quien_lo_tiene(f: &File) -> (Option<u32>, Option<String>) {
    use std::os::unix::fs::MetadataExt;
    let Ok(m) = f.metadata() else { return (None, None) };
    let (dev, ino) = (m.dev(), m.ino());
    // La codificación de `dev_t` de glibc (`gnu_dev_major`, `gnu_dev_minor`).
    let mut dispositivos = vec![(
        ((dev >> 8) & 0xfff) | ((dev >> 32) & 0xffff_f000),
        (dev & 0xff) | ((dev >> 12) & 0xffff_ff00),
    )];
    dispositivos.extend(dispositivo_del_montaje(f));
    let Ok(cerrojos) = std::fs::read_to_string("/proc/locks") else { return (None, None) };
    let mut quienes: Vec<u32> = Vec::new();
    for linea in cerrojos.lines() {
        let campos: Vec<&str> = linea.split_whitespace().collect();
        if campos.contains(&"->") || campos.get(1) != Some(&"FLOCK") {
            continue;
        }
        let Some(pos) = campos.iter().position(|c| c.matches(':').count() == 2) else {
            continue;
        };
        let mut partes = campos[pos].split(':');
        let (Some(ma), Some(me), Some(i)) = (partes.next(), partes.next(), partes.next()) else {
            continue;
        };
        let es_este = match (u64::from_str_radix(ma, 16), u64::from_str_radix(me, 16)) {
            (Ok(a), Ok(b)) => dispositivos.contains(&(a, b)) && i.parse::<u64>().ok() == Some(ino),
            _ => false,
        };
        if !es_este {
            continue;
        }
        match pos.checked_sub(1).and_then(|p| campos[p].parse::<u32>().ok()) {
            Some(p) if !quienes.contains(&p) => quienes.push(p),
            Some(_) => {}
            None => return (None, None),
        }
    }
    let pid = match quienes[..] {
        [p] if p != 0 => p,
        _ => return (None, None),
    };
    let nombre = std::fs::read_to_string(format!("/proc/{pid}/comm")).ok();
    (Some(pid), nombre.map(|n| n.trim().to_string()))
}

/// El `MAYOR:MENOR` del montaje de un descriptor, que es el de su superbloque: el
/// `mnt_id` de `/proc/self/fdinfo/FD`, buscado en `/proc/self/mountinfo` (el
/// tercer campo, en decimal). `None` si el sistema no lo dice.
#[cfg(target_os = "linux")]
fn dispositivo_del_montaje(f: &File) -> Option<(u64, u64)> {
    let montaje = montaje_del_descriptor(f)?;
    let montajes = leer_mountinfo()?;
    montajes.lines().find_map(|l| {
        let mut campos = l.split_whitespace();
        if campos.next()? != montaje {
            return None;
        }
        let (mayor, menor) = campos.nth(1)?.split_once(':')?;
        Some((mayor.parse().ok()?, menor.parse().ok()?))
    })
}

/// ⚠️ §740 · `/proc/self/mountinfo` entero, con `from_utf8_lossy` y no con
/// `read_to_string`: el núcleo sólo escapa el espacio, el tabulador, el salto de
/// línea y la barra invertida, y cualquier otro byte de una ruta sale tal cual.
/// Un solo byte que no fuera UTF-8, en cualquier montaje, hacía fallar la
/// lectura entera, y con ella la primera red. Los campos que se leen —el id, el
/// dispositivo y el tipo— son ASCII.
#[cfg(target_os = "linux")]
fn leer_mountinfo() -> Option<String> {
    let bytes = std::fs::read("/proc/self/mountinfo").ok()?;
    Some(String::from_utf8_lossy(&bytes).into_owned())
}

/// El `mnt_id` del montaje de un descriptor, de `/proc/self/fdinfo/FD` (desde
/// Linux 3.15). `None` si el sistema no lo dice.
#[cfg(target_os = "linux")]
fn montaje_del_descriptor(f: &File) -> Option<String> {
    use std::os::fd::AsRawFd;
    let info = std::fs::read_to_string(format!("/proc/self/fdinfo/{}", f.as_raw_fd())).ok()?;
    Some(info.lines().find_map(|l| l.strip_prefix("mnt_id:"))?.trim().to_string())
}

/// ⚠️ §740 · El tipo del sistema de ficheros de un descriptor: el de su montaje
/// en `/proc/self/mountinfo`. Por el montaje del descriptor, y no por la ruta,
/// porque en una ruta puede haber montajes apilados —en el contenedor donde se
/// escribió §740, `/dev/shm` tenía dos— y un fichero puede ser un montaje
/// propio. `None` si el sistema no lo dice.
#[cfg(target_os = "linux")]
fn sistema_de_ficheros(f: &File) -> Option<String> {
    let montaje = montaje_del_descriptor(f)?;
    let montajes = leer_mountinfo()?;
    tipo_en_mountinfo(&montajes, &montaje).map(str::to_string)
}

#[cfg(not(target_os = "linux"))]
fn sistema_de_ficheros(_: &File) -> Option<String> {
    None
}

/// ⚠️ §740 · El tipo del montaje `montaje` en un texto de `/proc/self/mountinfo`.
/// Cada línea es `ID PADRE MAYOR:MENOR RAÍZ PUNTO OPCIONES [OPCIONALES...] - TIPO
/// ORIGEN SUPEROPCIONES` (`proc_pid_mountinfo(5)`). Los campos opcionales son
/// cero o más, así que el tipo no está en una columna fija, sino detrás del campo
/// `-`, que ningún otro puede ser: un opcional es `etiqueta[:valor]`, y un
/// espacio en una ruta va escapado como `\040`.
///
/// ⚠️ Los campos se separan por UN espacio ASCII, y sólo por él. Con
/// `split_whitespace` se partía también por los espacios de Unicode —U+00A0, por
/// ejemplo—, que el núcleo no escapa: un punto de montaje con `\u{a0}-` sacaba un
/// campo `-` falso, y el tipo leído eran las opciones.
#[cfg(any(target_os = "linux", test))]
fn tipo_en_mountinfo<'a>(mountinfo: &'a str, montaje: &str) -> Option<&'a str> {
    mountinfo.lines().find_map(|l| {
        let mut campos = l.split(' ');
        if campos.next()? != montaje {
            return None;
        }
        campos.skip_while(|c| *c != "-").nth(1)
    })
}

/// ⚠️ §709 · **¿La ruta sigue nombrando el fichero de este descriptor?** El
/// dispositivo y el inodo de la ruta, sin seguir un enlace en su último
/// componente, tienen que ser los del descriptor; una ruta que ya no existe, o que
/// es un enlace, no lo es. Lo usan [`GuardianIndice::abrir`], justo después de
/// abrir, y [`GuardianIndice::persistir`], antes de cada escritura.
#[cfg(unix)]
fn sigue_siendo_el_mismo(ruta: &Path, f: &File) -> std::io::Result<bool> {
    use std::os::unix::fs::MetadataExt;
    let en_la_ruta = match std::fs::symlink_metadata(ruta) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(e),
    };
    let abierto = f.metadata()?;
    Ok(!en_la_ruta.file_type().is_symlink()
        && en_la_ruta.dev() == abierto.dev()
        && en_la_ruta.ino() == abierto.ino())
}

/// Fuera de unix, `std` estable no da la identidad de un fichero abierto: no se
/// comprueba, y se dice en la doc del crate.
#[cfg(not(unix))]
fn sigue_siendo_el_mismo(_: &Path, _: &File) -> std::io::Result<bool> {
    Ok(true)
}

#[cfg(not(target_os = "linux"))]
fn quien_lo_tiene(_: &File) -> (Option<u32>, Option<String>) {
    (None, None)
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

    /// §690 · adelantar persiste, sobrevive al cierre, y nunca retrocede: pedirle menos no hace
    /// nada, y la reserva siguiente es la de encima.
    #[test]
    fn adelantar_salta_hacia_delante_y_nunca_retrocede() {
        let p = en_disco("adelantar");
        {
            let mut g = GuardianIndice::abrir(&p).expect("abrir");
            g.reservar().expect("reservar");
            g.adelantar(40).expect("adelantar");
            assert_eq!(g.actual(), 40);
            g.adelantar(7).expect("pedir menos");
            assert_eq!(g.actual(), 40, "CRITICO: adelantar retrocedio");
        }
        let mut g = GuardianIndice::abrir(&p).expect("reabrir");
        assert_eq!(g.actual(), 40, "el salto se persistio");
        assert_eq!(g.reservar().expect("reservar"), 41);
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
    fn un_contador_borrado_reabre_en_cero_y_coincide_con_la_clave_en_cero() {
        // ⚠⚠ ECST §8.1 (doc/ecst/ECST.md): la PREMISA del hallazgo, medida
        // aqui. La pieza no distingue "no habia fichero" de "lo borraron": un
        // contador que ya firmo y cuyo fichero desaparece reabre en 0, y con la
        // clave de la semilla -que al rearrancar esta en 0- el par COINCIDE. R
        // solo ve el par, asi que la pieza NO puede cazarlo: lo caza la politica
        // del arranque consultando el segundo testigo -el diario del nodo, las
        // cofirmas del testigo- en todo estado, no solo en `ClaveEnCero`.
        let p = en_disco("borrado");
        {
            let mut g = GuardianIndice::abrir(&p).expect("abrir");
            for _ in 0..3 {
                g.reservar().expect("reservar");
            }
            assert_eq!(g.actual(), 3, "el fixture tiene que haber reservado");
        }
        std::fs::remove_file(&p).expect("borrar el contador");
        let g = GuardianIndice::abrir(&p).expect("reabrir");
        assert_eq!(g.actual(), 0, "reabre en cero sin saber que hubo reservas");
        assert_eq!(
            g.reconciliar(0),
            Reconciliacion::Coincide { indice: 0 },
            "y con la clave en cero COINCIDE: el caso que la politica tiene que cazar"
        );
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

    /// El tipo del sistema de ficheros de `ruta` segun `/proc/self/mounts`: el del
    /// punto de montaje mas largo que la contiene, y entre iguales el ultimo, que es
    /// el de arriba. Otro camino que el del guardian —la ruta y no el `mnt_id` del
    /// descriptor—, a proposito. `None` fuera de Linux o si no se puede leer.
    fn tipo_segun_mounts(ruta: &Path) -> Option<String> {
        let ruta = std::fs::canonicalize(ruta).ok()?;
        let texto = String::from_utf8_lossy(&std::fs::read("/proc/self/mounts").ok()?).into_owned();
        let mut mejor: Option<(usize, String)> = None;
        for linea in texto.lines() {
            let campos: Vec<&str> = linea.split(' ').collect();
            let (Some(punto), Some(tipo)) = (campos.get(1), campos.get(2)) else { continue };
            let largo = punto.len();
            if ruta.starts_with(punto) && mejor.as_ref().is_none_or(|(n, _)| largo >= *n) {
                mejor = Some((largo, tipo.to_string()));
            }
        }
        mejor.map(|(_, t)| t)
    }

    /// ¿Da este sistema el `mnt_id` de un descriptor abierto en `ruta`? Es lo que
    /// necesita la primera red (Linux desde 3.15). Se lee `fdinfo` aqui mismo, sin
    /// pasar por el guardian: si su lector se rompiera, este test no debe creer que
    /// el sistema no lo da.
    #[cfg(target_os = "linux")]
    fn da_mnt_id(ruta: &Path) -> bool {
        use std::os::fd::AsRawFd;
        let Ok(d) = File::open(ruta) else { return false };
        std::fs::read_to_string(format!("/proc/self/fdinfo/{}", d.as_raw_fd()))
            .is_ok_and(|info| info.lines().any(|l| l.starts_with("mnt_id:")))
    }

    #[cfg(not(target_os = "linux"))]
    fn da_mnt_id(_: &Path) -> bool {
        false
    }

    #[test]
    fn en_tmpfs_se_niega_a_operar() {
        // ⚠️ EL TEST QUE JUSTIFICA LA AUTOCOMPROBACION. K.1 midio que en
        // tmpfs `fsync` cuesta lo MISMO que no hacerlo (razon 1x, frente a
        // 382x en ext4): devuelve exito sin persistir nada.
        //
        // ⚠️⚠️ §740 · Hasta §740 buscaba el tmpfs con `df -T --output=fstype`, y en
        //    el `df` de GNU `-T` y `--output` no van juntos: salia con error, sin
        //    nada en la salida, y el test se saltaba SIEMPRE, con un aviso por
        //    stderr que el arnes solo enseña con `--nocapture`. Decia saltarse «EN
        //    VOZ ALTA», y en Linux no probaba nada. Ahora el tipo lo dice
        //    `/proc/self/mounts`, sin herramientas de fuera; y en Linux no
        //    encontrar ningun tmpfs es un fallo, no un salto. Fuera de Linux se
        //    salta, con el aviso.
        let candidatos = [PathBuf::from("/dev/shm"), std::env::temp_dir()];
        let base = candidatos
            .into_iter()
            .find(|b| b.is_dir() && tipo_segun_mounts(b).as_deref() == Some("tmpfs"));
        let Some(base) = base else {
            if cfg!(target_os = "linux") {
                panic!(
                    "en Linux tiene que haber un tmpfs donde probar el rechazo, y segun \
                     /proc/self/mounts no lo son ni /dev/shm ni el temp_dir"
                );
            }
            eprintln!(
                "AVISO: no se encontro ningun tmpfs donde probar el rechazo. \
                 La autocomprobacion NO se ha ejercitado en este entorno."
            );
            return;
        };
        let d = base.join(format!("guardian_tmpfs_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).expect("crear");
        let r = GuardianIndice::abrir(d.join("indice.bin"));
        let _ = std::fs::remove_dir_all(&d);
        // ⚠️ §740 · Donde el sistema da el `mnt_id` del descriptor, lo dice el tipo,
        //    sin medir, y el resultado es siempre el mismo; donde no, la medida de
        //    `fsync`. Hasta §740 era siempre la medida, que en tmpfs arrancaba de 2 a
        //    11 veces de cada 2000, tambien en reposo.
        let con_mnt_id = da_mnt_id(&base);
        match r {
            Err(GuardianError::SistemaEnMemoria { tipo, .. }) if con_mnt_id => {
                assert_eq!(tipo, "tmpfs")
            }
            Err(GuardianError::PersistenciaFalsa { .. }) if !con_mnt_id => {}
            otro => panic!(
                "en {} —que es tmpfs— el guardian DEBE negarse (y por el tipo si hay \
                 mnt_id: {con_mnt_id}), y dio: {otro:?}",
                base.display()
            ),
        }
    }

    #[test]
    fn en_disco_de_verdad_si_opera() {
        // La otra mitad: donde `fsync` cuesta, el guardian arranca.
        let p = en_disco("disco_real");
        GuardianIndice::abrir(&p).expect("en disco real el guardian debe arrancar");
    }

    /// ⚠️⚠️ §740 · EL FALSADOR DE LA MEDIANA. Diecinueve escrituras con `fsync` de
    /// tmpfs (1,2 µs) y una lenta, de 5 ms, que es una cifra de ejemplo: la media es
    /// 251 µs, y con ella el guardian ARRANCABA (con estas cifras bastan unos
    /// 180 µs). Es la forma que midio la sonda de §740: la escritura mas lenta, de
    /// 995 µs, y medias de ronda de hasta 51,2 µs con ninguna mediana por encima de
    /// 3,1 µs. Con la mediana, 1,2 µs: se niega, y sigue negandose con nueve lentas
    /// de veinte. Con diez, la mediana ya sale de las lentas, y se dice.
    #[test]
    fn unas_escrituras_lentas_no_hacen_persistente_un_tmpfs() {
        let sin = [1.0; 20];
        let mut con = [1.2; 20];
        con[7] = 5000.0;
        let media = con.iter().sum::<f64>() / 20.0;
        assert!(
            media / sin[0] >= RAZON_MINIMA || media >= SUELO_MICROS,
            "con la media de antes, esto arrancaba: {media} µs"
        );
        match decidir(&con, &sin) {
            Err(GuardianError::PersistenciaFalsa { con_fsync_us, razon, .. }) => {
                assert_eq!(con_fsync_us, 1.2, "el error da la mediana, no la media");
                assert!(razon < RAZON_MINIMA, "razon {razon}");
            }
            otro => panic!("una escritura lenta NO hace persistente un tmpfs: {otro:?}"),
        }
        for c in con.iter_mut().take(9) {
            *c = 5000.0;
        }
        assert!(decidir(&con, &sin).is_err(), "nueve lentas de veinte tampoco");
        con[9] = 5000.0;
        assert!(mediana(&con) > SUELO_MICROS, "con diez, la mediana sale de las lentas");
    }

    /// §740 · La otra cara: la mediana no rechaza un disco. Las cifras son las del
    /// contenedor donde se escribio §740 (ext4): en 200 rondas, la mediana con
    /// `fsync` mas baja fue de 114 µs, y sin el, de 0,80 a 2,19 µs. Con 114 µs pasa
    /// por el suelo, sea cual sea la razon. Y unas pocas escrituras con `fsync`
    /// rapidas, que la media diluia, tampoco lo tumban.
    #[test]
    fn la_mediana_no_rechaza_un_disco() {
        let sin = [1.0; 20];
        let mut con = [114.0; 20];
        assert!(decidir(&con, &sin).is_ok(), "ext4 medido");
        for c in con.iter_mut().take(9) {
            *c = 1.0;
        }
        assert!(decidir(&con, &sin).is_ok(), "nueve rapidas de veinte no lo tumban");
    }

    /// ⚠️ §740 · Un reloj mas grueso que una escritura da una mediana sin `fsync` de
    /// 0, y la razon sale infinita. Con ella el guardian arrancaba aunque `fsync`
    /// tampoco costara nada; ahora decide el suelo solo.
    #[test]
    fn un_reloj_que_no_mide_la_escritura_no_abre_la_puerta() {
        let sin = [0.0; 20];
        match decidir(&[0.0; 20], &sin) {
            Err(e @ GuardianError::PersistenciaFalsa { .. }) => {
                assert!(e.to_string().contains("el reloj no midió"), "y lo dice: {e}")
            }
            otro => panic!("sin nada medido, no se arranca: {otro:?}"),
        }
        assert!(decidir(&[5.0; 20], &sin).is_err(), "5 µs con fsync, bajo el suelo");
        assert!(decidir(&[114.0; 20], &sin).is_ok(), "114 µs, por encima: un disco");
    }

    /// ⚠️⚠️ §740 · La segunda red, CABLEADA. Con tiempos de tmpfs inyectados en una
    /// carpeta de disco, la comprobacion se niega por `PersistenciaFalsa`; con los de
    /// ext4, pasa; y el fichero de prueba no queda. Sin este test, cambiar la llamada
    /// a `decidir` por un `Ok` no lo cazaba ninguno: en Linux la primera red llega
    /// antes, y los demas tests de `decidir` lo llaman directamente.
    #[test]
    fn la_segunda_red_se_niega_con_tiempos_de_tmpfs() {
        let carpeta = en_disco("segunda_red").parent().expect("carpeta").to_path_buf();
        let tmpfs = |_: &mut File, con: bool| Ok(vec![if con { 1.2 } else { 1.0 }; 20]);
        match comprobar_persistencia_con(&carpeta, tmpfs) {
            Err(GuardianError::PersistenciaFalsa { con_fsync_us, .. }) => {
                assert_eq!(con_fsync_us, 1.2)
            }
            otro => panic!("con tiempos de tmpfs, la segunda red se niega: {otro:?}"),
        }
        let ext4 = |_: &mut File, con: bool| Ok(vec![if con { 114.0 } else { 1.0 }; 20]);
        comprobar_persistencia_con(&carpeta, ext4).expect("con tiempos de ext4, pasa");
        let restos: Vec<_> =
            std::fs::read_dir(&carpeta).expect("leer").filter_map(|e| e.ok()).collect();
        assert!(restos.is_empty(), "el fichero de prueba no queda: {restos:?}");
    }

    /// ⚠️ §740 · El lector de `/proc/self/mountinfo`: el tipo esta detras del campo
    /// `-`, haya campos opcionales o no, y el montaje se casa por su id ENTERO.
    #[test]
    fn el_tipo_del_montaje_se_lee_detras_del_separador() {
        let texto = "\
23 28 0:22 / /proc rw,relatime - proc proc rw
26 25 0:24 / /dev/shm rw,relatime - tmpfs tmpfs rw,size=16480952k
28 1 254:0 / / rw,relatime shared:1 master:2 - ext4 /dev/vda rw,discard
36 26 0:28 / /dev/shm rw,relatime - tmpfs tmpfs rw
260 28 0:50 / /mnt/con\\040espacio rw - ramfs none rw
2 1 0:2 / / rw - rootfs rootfs rw
29 28 254:16 / /roto rw sin-separador ext4
61 28 0:52 / /srv/x\u{a0}- rw,relatime - tmpfs tmpfs rw
";
        assert_eq!(tipo_en_mountinfo(texto, "36"), Some("tmpfs"));
        assert_eq!(tipo_en_mountinfo(texto, "28"), Some("ext4"), "con campos opcionales");
        assert_eq!(tipo_en_mountinfo(texto, "260"), Some("ramfs"), "con un espacio escapado");
        assert_eq!(tipo_en_mountinfo(texto, "2"), Some("rootfs"), "el id entero, no un prefijo");
        assert_eq!(tipo_en_mountinfo(texto, "29"), None, "sin `-` no hay tipo");
        assert_eq!(tipo_en_mountinfo(texto, "99"), None, "un montaje que no esta");
        assert_eq!(
            tipo_en_mountinfo(texto, "61"),
            Some("tmpfs"),
            "un espacio de Unicode en la ruta no parte un campo: el nucleo no lo escapa"
        );
        for tipo in ["tmpfs", "ramfs", "devtmpfs", "rootfs"] {
            assert!(SISTEMAS_EN_MEMORIA.contains(&tipo), "{tipo} vive en memoria");
        }
        for tipo in ["ext4", "xfs", "btrfs", "overlay", "9p", "proc"] {
            assert!(!SISTEMAS_EN_MEMORIA.contains(&tipo), "{tipo} no se rechaza por nombre");
        }
    }

    /// ⚠️⚠️ §709 · SEC-1, EL FALSADOR: dos guardianes sobre la misma ruta, y el
    /// segundo NO abre. Sobre `54fe931` abria y leia el contador del primero
    /// (`Ok(1)`): los dos habrian repartido los mismos numeros. El cerrojo es del
    /// descriptor, asi que dos guardianes en el MISMO proceso se excluyen igual que
    /// en dos: por eso basta un test, y el de dos procesos de verdad es el del nodo.
    #[test]
    fn dos_guardianes_sobre_la_misma_ruta_el_segundo_no_abre() {
        let p = en_disco("dos_guardianes");
        let mut primero = GuardianIndice::abrir(&p).expect("el primero abre");
        primero.reservar().expect("reservar");
        match GuardianIndice::abrir(&p) {
            Err(e @ GuardianError::ContadorOcupado { .. }) => {
                let GuardianError::ContadorOcupado { ref ruta, pid, .. } = e else { unreachable!() };
                assert!(Path::new(ruta).is_absolute(), "nombra la ruta canonica: {ruta}");
                // ⚠️ Quien lo tiene es ESTE proceso: si el error da un PID, es el
                //    propio, nunca otro. Que lo dé no se exige aquí, porque depende
                //    de lo que el sistema enseñe en `/proc` (otro espacio de PID, un
                //    `/proc` recortado); lo exige `tools/banco_un_proceso.sh`, con
                //    dos procesos de verdad.
                assert!(
                    pid.map_or(true, |p| p == std::process::id()),
                    "si dice quien lo tiene, es este proceso: {e}"
                );
                assert!(e.to_string().contains("NO se arranca"), "{e}");
            }
            otro => panic!("CRITICO: un segundo guardian abrio el mismo contador: {otro:?}"),
        }
        assert_eq!(primero.reservar().expect("el primero sigue"), 2, "el segundo no lo toco");
        drop(primero);
        let tercero = GuardianIndice::abrir(&p).expect("suelto el primero, el cerrojo se va con el");
        assert_eq!(tercero.actual(), 2, "y el valor es el que dejo el primero");
    }

    /// ⚠️ §709 · Lo que el cerrojo CUBRE y lo que NO, atado. Es del FICHERO, no del
    /// nombre: un enlace duro es el mismo fichero y no abre. Y dos ficheros son dos
    /// cerrojos: la misma semilla con otro contador abre los dos, y eso el cerrojo no
    /// lo ve —es lo que `SECURITY.md` §2 y `doc/CONFIANZA_RESIDUAL.md` declaran—.
    /// Y como es del fichero, el guardián escribe por él y no por el nombre: si el
    /// nombre deja de ser ese fichero —borrado, o cambiado por otro—, NO reserva.
    /// Antes del arreglo de la revisión, `persistir` abría la ruta en cada reserva y
    /// habría creado un contador nuevo, sin cerrojo, con el valor de este.
    #[test]
    fn el_cerrojo_es_del_fichero_y_dos_ficheros_son_dos_cerrojos() {
        let p = en_disco("cerrojo_del_fichero");
        let mut g = GuardianIndice::abrir(&p).expect("abrir");
        let duro = p.with_file_name("enlace_duro.bin");
        std::fs::hard_link(&p, &duro).expect("enlace duro");
        assert!(
            matches!(GuardianIndice::abrir(&duro), Err(GuardianError::ContadorOcupado { .. })),
            "un enlace duro es el mismo fichero: el mismo cerrojo"
        );
        let otro = p.with_file_name("otro_contador.bin");
        GuardianIndice::abrir(&otro).expect("otro fichero es otro cerrojo: NO lo cubre");

        assert_eq!(g.reservar().expect("con su nombre, reserva"), 1);
        std::fs::remove_file(&p).expect("borrar el nombre");
        assert!(g.reservar().is_err(), "sin su nombre, NO reserva");
        std::fs::write(&p, 7u64.to_le_bytes()).expect("otro fichero con el mismo nombre");
        assert!(g.reservar().is_err(), "con el nombre de otro fichero, tampoco");
        assert_eq!(g.actual(), 1, "y la cuenta no se mueve");
        assert_eq!(std::fs::read(&p).expect("leer"), 7u64.to_le_bytes(), "ni toca el otro");
        assert_eq!(std::fs::read(&duro).expect("leer"), 1u64.to_le_bytes(), "lo suyo sigue");
    }

    /// ⚠️ §709 · Un contador que es un ENLACE SIMBOLICO no se sigue. Sobre `54fe931`,
    /// el enlace a un fichero de `/dev/shm` abria —la autocomprobacion medía la
    /// carpeta del enlace, en disco— y los ocho bytes vivian en tmpfs.
    #[cfg(unix)]
    #[test]
    fn un_enlace_simbolico_no_se_sigue() {
        let p = en_disco("enlace_simbolico");
        let destino = p.with_file_name("destino_de_verdad.bin");
        std::os::unix::fs::symlink(&destino, &p).expect("enlace");
        match GuardianIndice::abrir(&p) {
            Err(GuardianError::EnlaceSimbolico { destino: d, .. }) => {
                assert!(d.contains("destino_de_verdad.bin"), "dice a donde apunta: {d}");
            }
            otro => panic!("un enlace simbolico no se sigue: {otro:?}"),
        }
        assert!(!destino.exists(), "y no se crea nada al otro lado");
    }
}
