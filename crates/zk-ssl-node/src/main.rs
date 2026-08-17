//! # zk-ssl-node — nodo de referencia JSON-RPC
//!
//! Un binario, un endpoint (`POST /`), JSON-RPC 2.0. La especificación
//! normativa está en `spec/RPC.md`; este código la implementa sobre
//! `zk_ssl::SovereignLayer` sin añadir semántica propia.
//!
//! Reparto de responsabilidades (el mismo del proyecto):
//! - el NODO entrega materiales (caminos y raíces: datos públicos) y
//!   aplica recibos verificando sus pruebas STARK;
//! - el CLIENTE (zk-ssl-sdk) deriva sus identificadores y PRUEBA en local.
//!   Ninguna clave de gasto viaja por este RPC.

// ⚠️ §296 · **El guardián del índice ya NO vive aquí.** Se mudó a
// `crates/zk-ssl-guardian` para que el TESTIGO pueda usar **la misma
// implementación** sin compilar el nodo. Es la regla de §243 (el
// verificador) y §254 (el hash) por tercera vez, y la razón es la de
// §253: dos implementaciones del mismo invariante pueden discrepar — y
// aquí discrepar significa **filtrar una clave**.
//
// ⚠️ Su doc viajó con él: cuando aquella decía «sin consumidor
// todavía» ya tenía dos (`firma_cabeza`, `recepcion`).

/// **El firmante de cabezas de época.** Eslabón 3 de la cadena de la
/// oponibilidad, y el **consumidor** que al guardián le faltaba (§234).
///
/// ⚠️ Sigue sin haber **latido**: esto firma cuando se le pide, y nadie
/// se lo pide todavía. Y **no hay custodia de clave declarada**.
mod firma_cabeza;

/// **El latido: emitir cabezas de época.** Cierra el eslabón 3.
///
/// ⚠️ **Sin `--clave` el nodo NO firma**: calcula la cabeza y lo dice.
/// Mismo criterio que `--dev` — el nodo separa capacidades por bandera
/// explícita, y firmar es algo que el operador habilita a sabiendas.
mod latido;

/// **El contador de recepción** (§253): en qué orden llegó cada operación
/// que el nodo **se puso a evaluar**.
///
/// ⚠️ `seq` no vale: sale de `entries.len()` y **solo existe si la
/// operación se aplicó**. **La censura vive en el hueco entre recibir y
/// aplicar.**
mod diario;
mod recepcion;
mod vista_acuses;

use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use clap::Parser;
use serde::Deserialize;
use serde_json::{json, Value};
use zk_ssl::commitment::ClientState;
use zk_ssl::two_phase::{BatchOp, ClaimReceipt, PendingNotice, SendReceipt};
use zk_ssl::{AccountIndex, LayerError, SovereignLayer};
use zk_ssl_wire as wire;
use zk_ssl_wire::{digest_from_wire, digest_to_wire, Q};

#[derive(Parser)]
#[command(name = "zk-ssl-node", version, about = "Nodo JSON-RPC de la ZK-Sovereign Settlement Layer")]
struct Args {
    /// Dirección de escucha.
    #[arg(long, default_value = "127.0.0.1:8545")]
    listen: SocketAddr,

    /// Ledger persistido (sled). Sin él, la capa vive en memoria.
    #[arg(long)]
    ledger: Option<String>,

    /// Habilita el espacio `dev_*` (fondos con custodios de PRUEBA).
    #[arg(long)]
    dev: bool,

    /// **Semilla de la clave de firma de cabezas** (96 bytes, en hex).
    ///
    /// ⚠️ **Sin esto el nodo NO FIRMA.** El latido sigue corriendo y la
    /// cabeza de época se calcula igual —su digest está en los vectores
    /// de conformidad de `zkssl/0.2`—; lo que falta es la firma.
    ///
    /// ⚠️ Y **una firma sin custodia declarada de la clave no tiene valor
    /// probatorio** (§236, §238). De dónde sale esta semilla y quién la
    /// guarda es **decisión de despliegue, y no está tomada**: pasarla por
    /// la línea de órdenes la deja en el historial del shell y en `ps`.
    /// Esta bandera existe para **poder ejercitar el mecanismo**, no como
    /// forma de operar.
    ///
    /// ⚠️ **Prefiere `--clave-fichero`.** Esta bandera existe para
    /// ejercitar el mecanismo, no para operar.
    /// ⚠️ Y **exige `--diario`** (nota 80; §285): quien firma, anota.
    ///
    #[arg(long, value_name = "HEX_96_BYTES", conflicts_with = "clave_fichero")]
    clave: Option<String>,

    /// **La semilla, leída de un fichero** (96 bytes en hex).
    ///
    /// ⚠️ **En Unix se comprueban los permisos al LEER**: si el fichero es
    /// legible por grupo u otros, el nodo **no arranca**. El keystore de
    /// §199 ya creaba con `0600`; esto añade **la mitad que faltaba**,
    /// porque crear bien no impide que alguien afloje después.
    ///
    /// ⚠️ Esto **no decide la custodia**: hace posible una decente.
    ///
    /// ⚠️ Y **exige `--diario`** (nota 80; §285): quien firma, anota.
    #[arg(long, value_name = "RUTA")]
    clave_fichero: Option<String>,

    /// **Dónde anota el nodo lo que firma** (una línea JSON por latido).
    ///
    /// ⚠️ Explícita, como `--clave`: el nodo no escribe en disco por su
    /// cuenta. Y desde §285 **un nodo con clave NO ARRANCA sin esto**
    /// (quien firma, anota, nota 80): firmar sin poder reconocer la
    /// propia firma era justo lo que §272 vino a arreglar. Ver la nota 80 del BACKLOG.
    ///
    /// ⚠️ **NO lleva el candado del guardián.** `PersistenciaFalsa`
    /// existe porque reusar un índice compromete la clave; perder este
    /// diario no compromete nada.
    #[arg(long, value_name = "RUTA")]
    diario: Option<String>,

    /// **Qué custodia AFIRMA el operador** para la clave de firma.
    ///
    /// ⚠️ **Es una afirmación del operador, no una comprobación del
    /// nodo** — salvo `fichero`, que sí se comprueba. El valor de la
    /// declaración **no está en que sea cierta, sino en que mentir en
    /// ella es oponible**.
    ///
    /// ⚠️ **`sin-declarar` es el valor por defecto y VIAJA IGUAL.** Si el
    /// campo se omitiera al no declarar nada, un consumidor no podría
    /// distinguir «no declara» de «versión vieja del nodo».
    #[arg(long, value_name = "MODELO", default_value = "sin-declarar",
          value_parser = ["sin-declarar", "fichero", "hsm", "kms", "otro"])]
    custodia: String,

    /// Fichero del **contador de recepción** (§253).
    ///
    /// ⚠️ Se persiste con `fsync` y **el nodo se niega a arrancar si el
    /// medio no persiste** (K.1, §234): un contador que vuelve a cero da
    /// **dos operaciones distintas con el mismo número**, y eso es **peor
    /// que no tener contador**.
    #[arg(long, value_name = "RUTA", default_value = "recepcion.bin")]
    contador_recepcion: String,

    /// Fichero del contador de índices de firma (§234).
    ///
    /// ⚠️ El guardián **se niega a arrancar si su `fsync` no persiste**:
    /// en `tmpfs` cuesta lo mismo que no hacerlo (K.1: 382× frente a 1×).
    #[arg(long, default_value = "zkssl-indice-firma.bin")]
    indice_firma: String,

    /// **Segundos entre cabezas de época.** §121 lo decidió: una por
    /// minuto, tras medir que a esa cadencia el almacenamiento cae 60
    /// veces, al precio declarado de dar al operador **una ventana de un
    /// minuto**. `0` apaga el latido.
    #[arg(long, default_value_t = crate::latido::LATIDO_POR_DEFECTO_S)]
    latido: u64,

    /// Límite regulatorio por operación.
    #[arg(long, default_value_t = 500_000)]
    limit: u64,
    /// Tope de suministro.
    #[arg(long, default_value_t = 100_000_000)]
    max_supply: u64,
    /// Máximo de cuentas.
    #[arg(long, default_value_t = 1_000)]
    max_accounts: u64,

    /// **Segundos que vive una reserva de posición de pendiente.**
    ///
    /// `zkssl_sendMaterials` reserva la posición que entrega (§211), de
    /// modo que dos titulares concurrentes reciben posiciones DISTINTAS
    /// contra la misma raíz. Quien pide materiales y no aplica dejaría
    /// la posición inmovilizada: esto la libera.
    ///
    /// ⚠️ **El valor por defecto está DECLARADO, no medido.** Lo que
    /// hay medido es la generación de una prueba —220-461 ms con ±50 %
    /// de ruido, §219— y eso NO incluye la red ni el tiempo que el
    /// titular tarda en decidirse. Treinta segundos son dos órdenes de
    /// magnitud de margen sobre lo único que se sabe. Medir el tramo
    /// materiales→entrega por RPC y ajustar está en la cola.
    ///
    /// Corta: un titular lento pierde su hueco y reintenta. Larga: quien
    /// pida materiales en bucle sin aplicar acumula reservas, y
    /// `allocate_pending` las recorre en cada llamada.
    #[arg(long, default_value_t = 30)]
    reserva_ttl: u64,

    /// **Cuantas cofirmas de testigo guarda el nodo por epoca** (§315).
    ///
    /// El almacen es una cota de RECURSO, no de confianza: guarda una sola
    /// cofirma viva por clave de testigo y solo de la epoca en curso, y esto
    /// pone el techo encima. Cada cofirma son **~37 KB** de hex
    /// (`FIRMA_RFC_BYTES = 18_469`), asi que 32 son ~1,2 MB por epoca.
    ///
    /// ⚠️ **El valor por defecto esta DECLARADO, no medido.** Lo que hay
    /// medido es el tamano de una firma; **cuantos testigos habra no lo sabe
    /// nadie todavia**, porque no hay ni uno operando de tercero. Treinta y
    /// dos es un techo que no estorba a un despliegue real y acota el gasto
    /// de uno hostil. Medir cuantos testigos aparecen y ajustar esta en la
    /// cola.
    ///
    /// Corto: un testigo legitimo se queda fuera y su cofirma no viaja.
    /// Largo: quien fabrique claves acumula ~37 KB por cada una.
    #[arg(long, default_value_t = 32)]
    max_cofirmas: usize,

    /// Filtro de tracing (stderr).
    #[arg(long, default_value = "info")]
    log: String,
}

/// Lo que el nodo guarda entre peticiones, bajo **un solo candado**.
///
/// Las reservas viven aquí y no en la capa a propósito: `reserve_pending`
/// y `release_pending` son la operación; **cuánto dura una reserva es
/// política del operador**, no invariante de la capa. Y un solo candado
/// para las dos cosas evita cualquier orden de bloqueo.
struct Estado {
    layer: SovereignLayer,
    /// posición reservada -> cuándo se entregó.
    reservas: BTreeMap<u64, Instant>,
}

struct App {
    estado: Mutex<Estado>,
    dev: bool,
    reserva_ttl: Duration,
    /// **La última cabeza firmada, en memoria.**
    ///
    /// ⚠️ Candado PROPIO, no el del estado: guardarla no debe volver a
    /// competir con las escrituras cuando el latido ya soltó el otro.
    ///
    /// ⚠️ **Se pierde al reiniciar**, y eso es el precio declarado de no
    /// tener histórico (§242).
    ///
    /// ⚠️ **CORRECCIÓN (§272): ese precio CADUCÓ.** Era razonable
    /// mientras nadie necesitara el histórico. Hoy sí: sin él **el nodo
    /// guarda el número de sus firmas y no las firmas** —el guardián
    /// persiste el contador de índice, no lo firmado— y el único con
    /// historia de lo que el operador firmó es **quien lo vigila**. Con
    /// `--diario` el nodo anota cada latido; esto sigue siendo la copia
    /// **en memoria** y **rápida**, no la memoria del nodo.
    ///
    /// ⚠️ **MEDIDO en L.1 (§247), y §242 lo decía impreciso.** Lo que un
    /// testigo ve al reiniciar **no es un hueco de índices**: es una
    /// **ventana de `SinFirma`**, y después el índice sigue **contiguo**.
    /// El guardián solo incrementa **al firmar**, así que morir entre
    /// latidos no gasta ninguno: se pierde **tiempo, no serie**.
    ///
    /// Hay hueco de índices **solo si se firmó una cabeza que nadie llegó
    /// a recoger** — y eso depende de la relación entre la cadencia del
    /// latido y la del testigo, **no del reinicio**.
    ultima_cabeza: Mutex<Option<latido::Latido>>,
    /// **Las hojas del MMR de cabezas** (§292): los digests de las
    /// cabezas ya emitidas, en orden. La memoria es cache — al arrancar
    /// se SIEMBRA del diario, y sin diario nace vacia: el `t` de la
    /// cabeza se resetea VISIBLEMENTE, y quien firma lleva diario
    /// obligado (§285), asi que toda cabeza FIRMADA lleva continuidad.
    hojas_mmr: Mutex<Vec<zk_ssl_verify::acuses::Digest>>,
    /// La clave pública de firma, en bytes del formato RFC. **Vacía** si el
    /// nodo arrancó sin `--clave`. Un testigo la necesita para verificar.
    clave_publica_firma: Vec<u8>,
    /// **Dónde anota el nodo lo que firma.** `None` si no se pasó
    /// `--diario`: entonces no hay memoria de lo emitido (§272).
    /// Desde §285, si hay clave esto nunca es `None`: quien firma, anota.
    diario: Option<std::path::PathBuf>,
    /// Cadencia del latido: cada cuánto esperar una cabeza nueva.
    latido_s: u64,
    /// Lo que el operador **afirma** sobre la custodia de la clave.
    custodia: String,
    /// Si el nodo **pudo comprobarlo**. Solo `fichero` es comprobable.
    custodia_comprobada: bool,
    /// **Las cofirmas de la epoca en curso** (§315), por clave de testigo.
    ///
    /// ⚠️ Candado PROPIO, como `ultima_cabeza` y por la misma razon: una
    /// submision no debe competir con las escrituras del estado.
    ///
    /// ⚠️ **Se AUTOLIMPIA y por eso no hay un segundo campo con la epoca:**
    /// cada cofirma guardada lleva su `epochDigest`, y la submision descarta
    /// las que no son de la cabeza en curso antes de insertar.
    ///
    /// ⚠️ **Se pierde al reiniciar**, como `ultima_cabeza`. Es coherente
    /// con no guardar historico: lo que un tercero quiera conservar, lo
    /// conserva el.
    cofirmas: Mutex<BTreeMap<Vec<u8>, wire::CofirmaDto>>,
    /// Techo de cofirmas por epoca. Cota de RECURSO, no de confianza.
    max_cofirmas: usize,
    /// **El contador de recepción** (§253), persistido con `fsync`.
    ///
    /// ⚠️ Candado propio, y se suelta enseguida: reservar cuesta un
    /// `fsync` —**0,907 ms medidos en ext4** (K.1)— y retenerlo mientras
    /// se aplica una operación pararía el nodo entero.
    recepcion: Mutex<recepcion::ContadorRecepcion>,
}

/// «Quien firma, anota» (nota 80, segunda mitad; §285): la decision de
/// arranque, PURA para poder probarse en frio — los tests de este binario
/// no ejercitan `Args`, asi que el predicado se prueba solo y el cableado
/// lo cubre el molde de `--custodia fichero`, tres lineas encima del bail.
fn firma_sin_diario(firmara: bool, con_diario: bool) -> bool {
    firmara && !con_diario
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    init_tracing(&args.log);

    let layer = open_layer(&args)?;

    if args.dev {
        tracing::warn!("modo --dev: dev_* habilitado con custodios de PRUEBA; no usar en producción");
    }

    // ── el latido, y la firma como capacidad SEPARADA ──
    // ⚠️ El firmante se crea ANTES de `App`: su clave pública es un campo
    // de `App`, porque **un testigo la necesita para verificar** y tiene
    // que poder pedirla por el cable.
    // ⚠️ `fichero` es **la unica declaracion que el nodo puede COMPROBAR**.
    // Las demas son afirmaciones puras, y se dice al arrancar.
    let custodia_comprobada = args.custodia == "fichero" && args.clave_fichero.is_some();
    if args.custodia == "fichero" && args.clave_fichero.is_none() {
        anyhow::bail!(
            "--custodia fichero exige --clave-fichero: el nodo NO afirma lo que no puede comprobar"
        );
    }
    let semilla_hex = match (&args.clave, &args.clave_fichero) {
        (Some(h), None) => {
            tracing::warn!(
                "--clave en la linea de ordenes: la semilla queda en el HISTORIAL \
                 del shell y en `ps`. Para operar, --clave-fichero"
            );
            Some(h.clone())
        }
        (None, Some(ruta)) => Some(leer_semilla_de_fichero(ruta)?),
        _ => None,
    };
    // ── «quien firma, anota» (nota 80, segunda mitad; §285) ──
    // ⚠️ Mismo molde que --custodia fichero, unas lineas arriba: el nodo
    // NO arranca. Un nodo que firma sin --diario no puede reconocer su
    // propia firma despues ni negar una que no emitio — y el mando
    // `--ausentes` del testigo (§283) compararia un diario que este nodo
    // nunca habria escrito.
    if firma_sin_diario(semilla_hex.is_some(), args.diario.is_some()) {
        anyhow::bail!(
            "quien firma, anota: --clave/--clave-fichero exige --diario. \
             Un nodo que firma sin diario no puede reconocer su propia firma"
        );
    }
    let firmante = match &semilla_hex {
        Some(hex) => {
            let semilla = descodificar_semilla(hex)?;
            let f = firma_cabeza::FirmanteCabeza::desde_semilla(&semilla, &args.indice_firma)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            if custodia_comprobada {
                tracing::info!(custodia = args.custodia, "custodia COMPROBADA por el nodo");
            } else if args.custodia == "sin-declarar" {
                tracing::warn!(
                    "las cabezas se FIRMARAN con custodia SIN DECLARAR: una firma sin \
                     custodia declarada NO tiene valor probatorio (SECURITY.md)"
                );
            } else {
                // ⚠️ La declaracion no comprobada se dice en voz alta, o el
                //    operador se acostumbra a que suene bien y ya.
                tracing::warn!(
                    custodia = args.custodia,
                    "custodia AFIRMADA pero NO COMPROBADA por el nodo: es una \
                     afirmacion suya, y mentir en ella es oponible"
                );
            }
            Some(f)
        }
        None => {
            // ⚠️ EN VOZ ALTA. El silencio equivale a que nadie se entere.
            tracing::warn!(
                "sin clave: las cabezas de epoca se CALCULAN pero NO se firman"
            );
            None
        }
    };

    let clave_publica_firma = firmante
        .as_ref()
        .map(|f| f.clave_publica())
        .unwrap_or_default();
    // §292: la siembra del MMR — el diario manda, la memoria es cache.
    let hojas_mmr_iniciales = args
        .diario
        .as_ref()
        .map(crate::diario::digests)
        .unwrap_or_default();
    let app = std::sync::Arc::new(App {
        estado: Mutex::new(Estado { layer, reservas: BTreeMap::new() }),
        dev: args.dev,
        reserva_ttl: Duration::from_secs(args.reserva_ttl),
        ultima_cabeza: Mutex::new(None),
        hojas_mmr: Mutex::new(hojas_mmr_iniciales),
        clave_publica_firma,
        diario: args.diario.as_ref().map(std::path::PathBuf::from),
        latido_s: args.latido,
        custodia: args.custodia.clone(),
        custodia_comprobada,
        cofirmas: Mutex::new(BTreeMap::new()),
        max_cofirmas: args.max_cofirmas,
        recepcion: Mutex::new(
            recepcion::ContadorRecepcion::abrir(&args.contador_recepcion)
                .map_err(|e| anyhow::anyhow!("{e}"))?,
        ),
    });

    if args.latido > 0 {
        tracing::info!(cada_s = args.latido, "latido de cabezas de epoca en marcha");
        latido::arrancar(app.clone(), firmante, Duration::from_secs(args.latido));
    } else {
        tracing::warn!("--latido 0: NO se emiten cabezas de epoca");
    }

    let router = Router::new().route("/", post(handle)).with_state(app);
    tracing::info!(listen = %args.listen, "zk-ssl-node escuchando (JSON-RPC 2.0, spec/RPC.md)");

    let listener = tokio::net::TcpListener::bind(args.listen).await?;
    axum::serve(listener, router).await?;
    Ok(())
}

fn open_layer(a: &Args) -> anyhow::Result<SovereignLayer> {
    // Las raíces de los conjuntos: en dev, las de la suite; en producción
    // vendrían de la ceremonia de constitución de custodios/gobernanza.
    #[cfg(feature = "dev")]
    let (c_root, g_root) = (
        zk_ssl::tests_support::custodian_root(),
        zk_ssl::tests_support::governance_root(),
    );
    #[cfg(not(feature = "dev"))]
    let (c_root, g_root) = anyhow::bail!(
        "build sin feature `dev`: pasar raíces de custodios/gobernanza reales (pendiente de flags --custodian-root/--governance-root)"
    );

    Ok(match &a.ledger {
        Some(path) => SovereignLayer::open(path, c_root, g_root, a.limit, a.max_supply, a.max_accounts)
            .map_err(|e| anyhow::anyhow!("abriendo {path}: {e:?}"))?,
        None => SovereignLayer::new(c_root, g_root, a.limit, a.max_supply, a.max_accounts),
    })
}

// ─────────────────────────── JSON-RPC 2.0 ───────────────────────────

#[derive(Deserialize)]
struct RpcRequest {
    #[allow(dead_code)]
    jsonrpc: Option<String>,
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

async fn handle(
    State(app): State<std::sync::Arc<App>>,
    Json(req): Json<RpcRequest>,
) -> Json<Value> {
    let t0 = Instant::now();
    let id = req.id.clone().unwrap_or(Value::Null);
    let out = dispatch(&app, &req.method, req.params);
    let ms = t0.elapsed().as_millis() as u64;

    Json(match out {
        Ok(result) => {
            tracing::info!(method = %req.method, ms, "ok");
            json!({ "jsonrpc": "2.0", "id": id, "result": result })
        }
        Err(e) => {
            tracing::warn!(method = %req.method, ms, code = e.code, msg = %e.message, "error");
            json!({ "jsonrpc": "2.0", "id": id,
                    "error": { "code": e.code, "message": e.message } })
        }
    })
}

/// Lee la semilla de un fichero, **comprobando los permisos**.
///
/// ⚠️ El keystore de §199 creaba con `0600`; **nadie comprobaba al leer**.
/// Crear bien no impide que alguien afloje despues, y un secreto legible
/// por el grupo es un secreto de todos.
fn leer_semilla_de_fichero(ruta: &str) -> anyhow::Result<String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let modo = std::fs::metadata(ruta)?.permissions().mode() & 0o777;
        anyhow::ensure!(
            modo & 0o077 == 0,
            "{ruta} tiene permisos {modo:04o}: es legible por grupo u otros. \
             Un secreto legible por el grupo es un secreto de todos. `chmod 600`"
        );
    }
    Ok(std::fs::read_to_string(ruta)?)
}

/// Directorio de trabajo de los tests.
///
/// ⚠️ **`crates/X/target/` NO estaba ignorado**: `/target` en el
/// `.gitignore` esta anclado a la raiz. Los tests del nodo escriben ahi
/// desde §234 y se libraron porque `*.bin` cubria los contadores — hasta
/// que §244 escribio un `semilla.hex` y git lo cogio.
///
/// Se centraliza aqui para que **un test nuevo no tenga que acordarse**.
#[cfg(test)]
/// Un nombre de directorio **por instancia**.
///
/// ⚠️ `tests_dir` **BORRA lo que encuentra**, y cargo corre los tests
/// **en paralelo**: con un nombre compartido, un test borraba el contador
/// de otro mientras escribía. **Diecisiete tests llaman a `nodo(30)`** — y
/// el resultado era INTERMITENTE: **46 tests una vez y 45 con fallo la
/// siguiente**.
///
/// ⚠️⚠️ **Lo cazó el canon, no la compuerta del bloque**: esa los corrió
/// UNA vez y ganó la carrera. **Un test intermitente pasa la mitad de las
/// veces, y una compuerta que ejecuta una sola vez lo deja pasar la mitad
/// de las veces.**
#[cfg(test)]
pub(crate) fn proximo_nodo() -> u64 {
    static N: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    N.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

pub(crate) fn tests_dir(nombre: &str) -> std::path::PathBuf {
    let d = std::path::Path::new("target").join(format!("t_{nombre}"));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("crear el directorio de trabajo del test");
    d
}

/// Bytes a hexadecimal, para el cable.
fn hex_de(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// Lee la semilla de 96 bytes en hexadecimal.
///
/// ⚠️ 96 = 3×32: `SK_SEED ‖ SK_PRF ‖ PUB_SEED`. Se rechaza cualquier otra
/// longitud **en el arranque**, no al firmar: un nodo que arranca y luego
/// no puede firmar es peor que uno que no arranca.
fn descodificar_semilla(hex: &str) -> anyhow::Result<Vec<u8>> {
    let h = hex.trim().trim_start_matches("0x");
    anyhow::ensure!(
        h.len() == 192,
        "la semilla debe tener 96 bytes (192 caracteres hex) y tiene {}",
        h.len() / 2
    );
    (0..96)
        .map(|i| u8::from_str_radix(&h[i * 2..i * 2 + 2], 16))
        .collect::<Result<Vec<u8>, _>>()
        .map_err(|e| anyhow::anyhow!("la semilla no es hexadecimal: {e}"))
}

/// ⚠️ `Debug` no estaba, y eso es un defecto por derecho propio: un tipo
/// de error sin `Debug` obliga a cualquiera que lo use a inventarse
/// rodeos —`.expect()` no compila sobre `Result<_, RpcError>`—. Se
/// añadió en §228, al escribir los primeros tests del nodo. No toca el
/// cable: `handle` construye el JSON del error a mano.
#[derive(Debug)]
struct RpcError {
    code: i64,
    message: String,
}

/// **Exige la credencial de la cuenta ANTES de entregar un camino** (§261).
///
/// ⚠️ Comprueba contra **el indice pedido**, no solo que la clave sea bien
/// formada: `account_view_authenticated` devuelve `None` si la clave es de
/// **otra** cuenta. Hay test de ese caso exacto, y es **el unico que prueba
/// el sello** — ausente y malformada las caza cualquier cosa.
///
/// ⚠️ Esta comprobacion **ya existia, del lado equivocado**: el SDK miraba
/// `materials.receiver.public_id != wallet.public_id()` **despues** de
/// recibir los caminos. Una comprobacion del lado del cliente no protege al
/// sistema: **protege al cliente que la ejecuta**.
fn exige_credencial(l: &SovereignLayer, indice: u64, vk: &wire::B32) -> Result<(), RpcError> {
    let clave = digest_from_wire(vk).map_err(RpcError::wire)?;
    l.account_view_authenticated(indice, clave)
        .map(|_| ())
        .ok_or_else(|| RpcError::credencial(indice))
}

impl RpcError {
    fn invalid_params(e: impl std::fmt::Display) -> Self {
        Self { code: -32602, message: format!("parámetros inválidos: {e}") }
    }
    fn method_not_found(m: &str) -> Self {
        Self { code: -32601, message: format!("método desconocido: {m}") }
    }
    fn layer(e: LayerError) -> Self {
        // -32000: rechazo de la capa. El mensaje es el Debug del error,
        // que en este proyecto es autoexplicativo.
        Self { code: -32000, message: format!("{e:?}") }
    }
    /// ⚠️ §261: la credencial **no cuadra con ESA cuenta**.
    ///
    /// Codigo propio —-32004— y no `layer`: quien pregunta tiene que poder
    /// distinguir «no autorizado» de «la capa rechazo la operacion». Un
    /// instrumento que falla dice QUE fallo (§254).
    fn credencial(indice: u64) -> Self {
        Self { code: -32004, message: format!("credencial invalida para la cuenta {indice}") }
    }
    /// ⚠️ El número de recepción **también en el error**: el caso que
    /// importa es el rechazo, y ahí es donde un censor se escondería.
    fn con_recepcion(mut self, rx: u64) -> Self {
        self.message = format!("{} [receptionSeq={rx:#x}]", self.message);
        self
    }
    fn wire(e: wire::WireError) -> Self {
        Self::invalid_params(e)
    }
}

fn parse<T: serde::de::DeserializeOwned>(params: Value) -> Result<T, RpcError> {
    serde_json::from_value(params).map_err(RpcError::invalid_params)
}

fn dispatch(app: &App, method: &str, params: Value) -> Result<Value, RpcError> {
    // Las operaciones bloquean el Mutex mientras verifican pruebas
    // (~decenas de ms). Correcto para un nodo único.
    //
    // ⚠️ **Aquí decía que el paso siguiente era «una cola de escritura
    // (ver ROADMAP)». Las dos mitades estaban mal** (§230):
    //
    // - **No hay tal ROADMAP.** El único es `ROADMAP-ECOSISTEMA.md`, que
    //   trata de especificación, SDKs y vectores de conformidad, y no
    //   menciona el candado ni la concurrencia. Referencia rota.
    // - **Y una cola de escritura no arreglaría nada.** El banco I.1
    //   midió cuatro agregadores concurrentes: aplica UNO por ronda y
    //   los otros tres reciben `StaleState`. Lo que serializa no es
    //   este candado: es que **un recibo solo vale contra la raíz
    //   exacta contra la que se probó** (`two_phase.rs`, la
    //   comprobación de `root_old`). Quitar el `Mutex` no cambiaría un
    //   solo resultado de ese banco.
    //
    // Lo que el nodo SÍ hace bien bajo esa carga, y estaba sin medir:
    // **rechaza barato**. Un lote muerto cuesta 3,1 ms frente a los
    // 32 de uno aplicado —el 9 %—, porque la raíz se comprueba ANTES de
    // verificar las pruebas. El precio de la contención lo paga el
    // agregador que pierde, no el nodo.
    let mut guardia = app.estado.lock().expect("mutex del estado envenenado");
    let Estado { layer: l, reservas } = &mut *guardia;

    // ── Barrido PEREZOSO de reservas caducadas ────────────────────
    // Sin tarea de fondo a propósito: un temporizador disputaría este
    // mismo candado con las escrituras, y §218 midió que la contención
    // es el cuello. Barrer aquí cuesta O(caducadas) y no añade candados.
    let ahora = Instant::now();
    let ttl = app.reserva_ttl;
    let caducadas: Vec<u64> = reservas
        .iter()
        .filter(|(_, t)| ahora.duration_since(**t) > ttl)
        .map(|(p, _)| *p)
        .collect();
    for p in caducadas {
        l.release_pending(p);
        reservas.remove(&p);
        tracing::warn!(posicion = p, "reserva caducada y liberada");
    }

    match method {
        // ── lectura ────────────────────────────────────────────────
        "zkssl_protocolVersion" => Ok(json!("zkssl/0.2")),

        "zkssl_params" => Ok(serde_json::to_value(wire::ParamsDto {
            regulatory_limit: Q(l.regulatory_limit()),
            max_supply: Q(l.max_supply()),
            max_accounts: Q(l.max_accounts()),
            custodian_root: digest_to_wire(&l.custodian_set_root()),
        })
        .unwrap()),

        "zkssl_epochHead" => {
            // §275: la pareja se computa aqui IGUAL que en el latido —
            // mismas funciones, mismo P—; el test del latido «la cabeza
            // del latido es la que sirve el RPC» es la compuerta.
            let p_epoca = crate::latido::limite_de_epoca(app);
            let pares = crate::vista_acuses::pares(l.transition_log().entries());
            let (r, n) = crate::vista_acuses::pareja_de_ahora(&pares, p_epoca);
            {
                let (cm, tm) = crate::latido::pareja_mmr(app);
                Ok(serde_json::to_value(wire::EpochHeadDto::from(&l.epoch_head(r, n, cm, tm))).unwrap())
            }
        }

        // ⚠️ **El método del TESTIGO** (§242). Aditivo: no toca
        // `zkssl_epochHead`, que sigue sirviendo la cabeza SIN firma y está
        // en los vectores de conformidad de 0.2. **La versión no se mueve.**
        //
        // ⚠️ Tres respuestas, y **ninguna es un error genérico**: sin latido
        // todavía, sin `--clave`, o la cabeza firmada. Es la forma de §241
        // llevada al cable: **la pieza que falta se nota también aquí**.
        "zkssl_signedEpochHead" => {
            let u = app.ultima_cabeza.lock().map_err(|_| RpcError {
                code: -32603,
                message: "candado de la ultima cabeza envenenado".into(),
            })?;
            // ⚠️ §313 · las tres formas las monta el TIPO, no un `json!`.
            // El invariante «si `available`, esos trece son todos `Some`»
            // se vigila ahora en la construccion y en la lectura.
            let dto = match u.as_ref() {
                None => wire::SignedEpochHeadDto::sin_latido(
                    Q(app.latido_s),
                    app.custodia.clone(),
                    app.custodia_comprobada,
                    "aun no ha habido latido: el nodo acaba de arrancar".into(),
                ),
                Some(l) if l.firma.is_none() => wire::SignedEpochHeadDto::sin_clave(
                    Q(app.latido_s),
                    app.custodia.clone(),
                    app.custodia_comprobada,
                    "el nodo arranco SIN --clave: las cabezas se calculan pero NO se firman".into(),
                    Q(l.seq),
                    wire::B32(l.epoch_digest),
                    Q(l.emitida_unix),
                ),
                Some(l) => {
                    let c = l.firma.as_ref().expect("comprobado en el brazo anterior");
                    // ⚠️ §275 sigue vigente: campos+digest+firma del MISMO
                    // latido. Lo que cambia es QUIEN convierte: el `From` del
                    // cable, en vez de seis `digest_to_wire` a mano aqui.
                    //
                    // ⚠️ `epochDigest` pasa a salir de `cabeza.digest()` en vez
                    // del campo del latido. NO es un valor distinto: `latido.rs`
                    // construye ese campo con `digest_to_wire(&cabeza.digest()).0`
                    // —la misma expresion— y `la_cabeza_viaja_entera_y_su_digest_
                    // es_el_del_latido` ya lo ata. El cambio es de RUTA, no de
                    // valor, y va declarado.
                    wire::SignedEpochHeadDto::con_firma(
                        &wire::EpochHeadDto::from(&l.cabeza),
                        String::from_utf8_lossy(firma_cabeza::DOMINIO).into_owned(),
                        Q(u64::from(c.version_formato)),
                        Q(c.indice),
                        wire::Blob(c.firma.clone()),
                        wire::Blob(app.clave_publica_firma.clone()),
                        Q(l.emitida_unix),
                        Q(app.latido_s),
                        // ⚠️ AFIRMADO por el operador; COMPROBADO solo si es
                        // `fichero`. El consumidor distingue las dos cosas.
                        app.custodia.clone(),
                        app.custodia_comprobada,
                    )
                }
            };
            serde_json::to_value(dto).map_err(|e| RpcError {
                code: -32603,
                message: format!("la cabeza firmada no serializa: {e}"),
            })
        }
        // ── §315 · EL TRANSPORTE DE LA COFIRMA: el testigo la deja aqui ──
        //
        // ⚠️ El nodo pasa a ser el transporte —lo que decidio la nota 83— y
        // **no** la autoridad. Lo que comprueba y lo que no está escrito en
        // `spec/RPC.md`, y el resumen es: la forma sí, el testigo no.
        "zkssl_submitCosig" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct P {
                cosig: wire::CofirmaDto,
            }
            let p: P = parse(params)?;

            // ── 1 · la epoca EN CURSO ──
            // Sin latido no hay nada que cofirmar, y eso NO es un error del
            // que llama: se dice, como hacen `ackPath` y `consistencyProof`.
            let actual = {
                let u = app.ultima_cabeza.lock().map_err(|_| RpcError {
                    code: -32603,
                    message: "candado de la ultima cabeza envenenado".into(),
                })?;
                u.as_ref().map(|l| l.epoch_digest)
            };
            let actual = match actual {
                Some(d) => d,
                None => {
                    return Ok(json!({ "accepted": false, "stored": Q(0),
                        "reason": "aun no ha habido latido: no hay epoca que cofirmar" }))
                }
            };
            if p.cosig.epoch_digest.0 != actual {
                return Ok(json!({ "accepted": false, "stored": Q(0),
                    "reason": "la cofirma no es de la epoca en curso" }));
            }

            // ── 2 · VERIFICAR, con el mismo verificador que usara el tercero ──
            //
            // ⚠️⚠️ Y lo que esto NO acota, declarado aqui y no escondido: la
            // clave del testigo viaja DENTRO del objeto y nada la acredita,
            // asi que cualquiera puede firmar con la suya y esto pasara. Esto
            // es cota de FORMA. La cota de CONFIANZA es la politica del
            // cliente, que nombra testigos — y tiene que vivir alli, porque
            // el nodo es el operador.
            let c = zk_ssl_verify::CabezaFirmada {
                version_formato: p.cosig.version_formato.0 as u8,
                indice: p.cosig.indice.0,
                firma: p.cosig.firma.0.clone(),
            };
            if let Err(e) = zk_ssl_verify::verificar_cofirma(
                &p.cosig.clave_publica_testigo.0,
                &actual,
                &p.cosig.clave_publica_operador.0,
                &c,
            ) {
                return Ok(json!({ "accepted": false, "stored": Q(0),
                    "reason": format!("la cofirma no verifica: {e}") }));
            }

            // ── 3 · guardar, con las dos cotas de RECURSO ──
            //
            // La clave del mapa es la del TESTIGO: la serie es por testigo
            // (§310), asi que reenviar SUSTITUYE en vez de acumular. Y el
            // almacen se autolimpia aqui: cada cofirma lleva su epoca.
            let mut g = app.cofirmas.lock().map_err(|_| RpcError {
                code: -32603,
                message: "candado de las cofirmas envenenado".into(),
            })?;
            g.retain(|_, v| v.epoch_digest.0 == actual);
            let nueva = !g.contains_key(&p.cosig.clave_publica_testigo.0);
            if nueva && g.len() >= app.max_cofirmas {
                let n = g.len() as u64;
                return Ok(json!({ "accepted": false, "stored": Q(n),
                    "reason": "tope de cofirmas por epoca alcanzado: ver --max-cofirmas" }));
            }
            let clave = p.cosig.clave_publica_testigo.0.clone();
            g.insert(clave, p.cosig);
            Ok(json!({ "accepted": true, "stored": Q(g.len() as u64) }))
        }
        // ── §315 · y el cliente se las lleva cuando las necesita ──
        "zkssl_cosigs" => {
            #[derive(Deserialize, Default)]
            #[serde(rename_all = "camelCase")]
            struct P {
                epoch_digest: Option<wire::B32>,
            }
            let p: P = if params.is_null() { P::default() } else { parse(params)? };
            let actual = {
                let u = app.ultima_cabeza.lock().map_err(|_| RpcError {
                    code: -32603,
                    message: "candado de la ultima cabeza envenenado".into(),
                })?;
                u.as_ref().map(|l| l.epoch_digest)
            };
            let pedida = p.epoch_digest.map(|b| b.0).or(actual);
            let g = app.cofirmas.lock().map_err(|_| RpcError {
                code: -32603,
                message: "candado de las cofirmas envenenado".into(),
            })?;
            // ⚠️ El nodo NO guarda historico: pedir una epoca que no es la
            // actual devuelve CERO, y eso no es un error — es la misma
            // promesa que ya hace con la cabeza.
            let lista: Vec<&wire::CofirmaDto> = match pedida {
                Some(d) => g.values().filter(|v| v.epoch_digest.0 == d).collect(),
                None => Vec::new(),
            };
            Ok(json!({
                "epochDigest": pedida.map(wire::B32),
                "n": Q(lista.len() as u64),
                "cosigs": lista,
            }))
        }
        // ⚠️ **§259 · EL RECIBO DE INCLUSION.** Del arbol `accounts`, que es
        // el que firma la cabeza. `leafFormat` va OBSERVADO: la capa
        // compone la hoja de las dos formas y declara la que caso.
        //
        // ⚠️ **Aditivo**: no toca `zkssl_epochHead` ni los valores que
        // viajan. **La version NO sube**, por la misma razon que no subio
        // con `zkssl_applyMany` (§222) ni con `zkssl_signedEpochHead`.
        "zkssl_inclusionReceipt" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct P { index: Q, view_key: wire::B32 }
            let p: P = parse(params)?;
            // ⚠️ §261: el recibo cambia de promesa. VERIFICARLO sigue sin
            //    necesitar al nodo; OBTENERLO pasa a ser del titular, que
            //    puede reenviarlo a quien quiera. Ver spec/RPC.md.
            exige_credencial(l, p.index.0, &p.view_key)?;
            let m = l.inclusion_materials(p.index.0).map_err(RpcError::layer)?;
            Ok(serde_json::to_value(wire::InclusionReceiptDto {
                index: Q(m.index),
                leaf: digest_to_wire(&m.leaf),
                path: wire::MerklePathDto::from(&m.path),
                leaf_format: m.forma.como_cable().to_string(),
                head: wire::EpochHeadDto::from(&{ let p_epoca = crate::latido::limite_de_epoca(app); let pares = crate::vista_acuses::pares(l.transition_log().entries()); let (r, n) = crate::vista_acuses::pareja_de_ahora(&pares, p_epoca); let (cm, tm) = crate::latido::pareja_mmr(app); l.epoch_head(r, n, cm, tm) }),
            })
            .unwrap())
        }

        // ⚠️ **§275 · EL CAMINO DE UNA EPOCA CERRADA.** Lo que le falta
        // al acuse de §274 para atarse a una firma. La cabeza NO viaja
        // (§248): el camino se verifica contra la que el titular YA
        // custodia — servirla aqui seria dejar que el operador fabrique
        // la vara con la que se le mide.
        //
        // ⚠️ Los limites de epoca salen del DIARIO (§272). Sin `--diario`
        // no hay limites que servir, y la respuesta LO DICE — la forma
        // de §241: la pieza que falta se nota, no se disfraza de error.
        // ⚠️ Aditivo: `zkssl/0.2` no sube, como con los tres anteriores.
        "zkssl_ackPath" => {
            #[derive(Deserialize)]
            struct P { seq: Q }
            let p: P = parse(params)?;
            match app.diario.as_ref() {
                None => Ok(json!({
                    "available": false,
                    "reason": "el nodo corre sin --diario: los limites de epoca no se conservan",
                })),
                Some(ruta) => {
                    let limites = crate::diario::limites(ruta);
                    match crate::vista_acuses::limites_para(&limites, p.seq.0) {
                        None => Ok(json!({
                            "available": false,
                            "reason": "la epoca de esa entrada sigue ABIERTA: vuelve tras el proximo latido",
                            "beatSeconds": Q(app.latido_s),
                        })),
                        Some((p_epoca, s)) => {
                            let pares = crate::vista_acuses::pares(l.transition_log().entries());
                            match crate::vista_acuses::camino_de_epoca(
                                &pares, p_epoca, s, p.seq.0, crate::vista_acuses::N_MAX_CABEZAS,
                            ) {
                                None => Ok(json!({
                                    "available": false,
                                    "reason": "esa entrada no existe en el registro dentro de esa epoca",
                                })),
                                Some((_raiz, hermanos, derecha)) => Ok(json!({
                                    "available": true,
                                    "s": Q(s),
                                    "camino": {
                                        "siblings": hermanos.iter().map(digest_to_wire).collect::<Vec<_>>(),
                                        "isRight": derecha,
                                    },
                                })),
                            }
                        }
                    }
                }
            }
        }

        "zkssl_consistencyProof" => {
            // §293: el eslabon 2 como SERVICIO. El camino que prueba que la
            // cima ACTUAL extiende a la de una cabeza custodiada de tamano
            // oldSize. Doctrina §248: la cabeza NO viaja aqui. ⚠️ La pareja
            // FIRMADA es el acumulador ANTES de cada cabeza (el push va tras
            // el emit): el camino de tamano t lo firma la cabeza SIGUIENTE
            // en emitirse — el cliente espera a la que firme el mmrSize de
            // esta respuesta, a lo sumo un latido.
            let viejo = params
                .get("oldSize")
                .and_then(|v| serde_json::from_value::<Q>(v.clone()).ok())
                .ok_or_else(|| RpcError {
                    code: -32602,
                    message: "falta oldSize (Q): el mmrSize de la cabeza custodiada".into(),
                })?
                .0;
            let h = app.hojas_mmr.lock().map_err(|_| RpcError {
                code: -32603,
                message: "candado de las hojas del MMR envenenado".into(),
            })?;
            let t = h.len() as u64;
            if viejo == 0 {
                Ok(json!({
                    "available": false,
                    "reason": "oldSize 0: no hay historia que extender",
                    "mmrSize": Q(t),
                }))
            } else if viejo > t {
                // §292 prometio el reseteo VISIBLE; esta es la promesa en el
                // cable: un nodo que rearranco sin diario lo DICE.
                Ok(json!({
                    "available": false,
                    "reason": format!(
                        "el acumulador de este nodo va POR DETRAS: lleva t={t} y se piden {viejo} — \
                         reinicio sin diario, o no es el mismo nodo"
                    ),
                    "mmrSize": Q(t),
                }))
            } else {
                let camino = zk_ssl_verify::mmr::prueba_de_consistencia(&h, viejo)
                    .expect("0 < viejo <= t: la prueba existe por construccion");
                Ok(json!({
                    "available": true,
                    "mmrSize": Q(t),
                    "camino": camino.iter().map(digest_to_wire).collect::<Vec<_>>(),
                }))
            }
        }
        "zkssl_supply" => Ok(json!({
            "total": Q(l.total_supply()),
            "pending": Q(l.total_pending()),
        })),

        "zkssl_accountCount" => Ok(serde_json::to_value(Q(l.account_count() as u64)).unwrap()),

        "zkssl_publicId" => {
            #[derive(Deserialize)]
            struct P { index: Q }
            let p: P = parse(params)?;
            let id = l
                .public_id_of(p.index.0)
                .ok_or_else(|| RpcError::layer(LayerError::AccountNotFound(p.index.0)))?;
            Ok(serde_json::to_value(digest_to_wire(&id)).unwrap())
        }

        // Vista AUTENTICADA (49-A): el titular presenta su clave de VISTA
        // (derivada, no la de gasto) y recibe su AccountView completo.
        "zkssl_accountView" => {
            #[derive(Deserialize)]
            struct P { index: Q, #[serde(rename = "viewKey")] view_key: wire::B32 }
            let p: P = parse(params)?;
            let vk = digest_from_wire(&p.view_key).map_err(RpcError::wire)?;
            let v = l
                .account_view_authenticated(p.index.0, vk)
                .ok_or_else(|| RpcError::layer(LayerError::AccountNotFound(p.index.0)))?;
            Ok(serde_json::to_value(wire::AccountViewDto::from(&v)).unwrap())
        }

        "zkssl_logEntry" => {
            #[derive(Deserialize)]
            struct P { seq: Q }
            let p: P = parse(params)?;
            let e = l
                .transition_log()
                .entries()
                .iter()
                .find(|e| e.seq == p.seq.0)
                .ok_or_else(|| RpcError::invalid_params(format!("no hay seq {}", p.seq.0)))?;
            Ok(serde_json::to_value(wire::LogEntryDto::from(e)).unwrap())
        }

        "zkssl_logEntries" => {
            #[derive(Deserialize, Default)]
            #[serde(default)]
            struct P { #[serde(rename = "fromSeq")] from_seq: Option<Q>, limit: Option<Q> }
            let p: P = if params.is_null() { P::default() } else { parse(params)? };
            let from = p.from_seq.map(|q| q.0).unwrap_or(0);
            let limit = p.limit.map(|q| q.0 as usize).unwrap_or(100).min(1_000);
            let out: Vec<wire::LogEntryDto> = l
                .transition_log()
                .entries()
                .iter()
                .filter(|e| e.seq >= from)
                .take(limit)
                .map(wire::LogEntryDto::from)
                .collect();
            Ok(serde_json::to_value(out).unwrap())
        }

        "zkssl_verifyChain" => match l.transition_log().verify_chain() {
            Ok(()) => Ok(json!({ "ok": true, "entries": l.transition_log().len() })),
            Err(e) => Ok(json!({ "ok": false, "error": format!("{e:?}") })),
        },

        // ── apertura: los identificadores viajan, la clave NO ──────
        "zkssl_openAccount" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct P { public_id: wire::B32, view_id: wire::B32, leaf_salt: wire::B32 }
            let p: P = parse(params)?;
            let index = l
                .open_with_id(
                    digest_from_wire(&p.public_id).map_err(RpcError::wire)?,
                    digest_from_wire(&p.view_id).map_err(RpcError::wire)?,
                    digest_from_wire(&p.leaf_salt).map_err(RpcError::wire)?,
                )
                .map_err(RpcError::layer)?;
            Ok(json!({ "index": Q(index) }))
        }

        // ── pago en dos fases ──────────────────────────────────────
        "zkssl_sendMaterials" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct P { sender: Q, receiver_id: wire::B32, amount: Q, salt: wire::B32,
                       view_key: wire::B32 }
            let p: P = parse(params)?;
            // ⚠️ §261: ANTES de tocar la capa. Este brazo repartia el camino
            //    del remitente SIN CREDENCIAL NINGUNA desde que existe.
            exige_credencial(l, p.sender.0, &p.view_key)?;
            let receptor = digest_from_wire(&p.receiver_id).map_err(RpcError::wire)?;
            let salt = digest_from_wire(&p.salt).map_err(RpcError::wire)?;

            // ⚠️ **RESERVAR, no solo consultar.** `allocate_pending` es PURA:
            // no muta. El candado serializa las peticiones pero no cambia su
            // resultado, así que dos titulares recibían la MISMA posición y
            // el segundo moría al aplicar. Es el fallo que §211 arregló en la
            // capa con `reserve_pending` y que este nodo no usaba.
            //
            // Y es condición necesaria del lote: `apply_many` rechaza el lote
            // entero con `DuplicatePendingInBatch` si dos comparten posición.
            let pos = l.reserve_pending().map_err(RpcError::layer)?;
            let m = match l.send_materials_at(p.sender.0, receptor, p.amount.0, salt, pos) {
                Ok(m) => m,
                Err(e) => {
                    // ⚠️ Soltar en el camino de ERROR no es un detalle: la capa
                    // rechaza por cuenta congelada, saldo o límite regulatorio, y
                    // sin esto cada rechazo dejaría una reserva muerta. Un
                    // atacante ni siquiera necesitaría saldo para provocarlas.
                    l.release_pending(pos);
                    return Err(RpcError::layer(e));
                }
            };
            reservas.insert(pos, Instant::now());
            Ok(serde_json::to_value(wire::SendMaterialsDto::from(&m)).unwrap())
        }

        "zkssl_applySend" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct P {
                receipt: wire::SendReceiptDto,
                sender: Q,
                sender_state: wire::ClientStateDto,
                amount: Q,
            }
            let p: P = parse(params)?;
            let receipt = (&p.receipt).try_into().map_err(RpcError::wire)?;
            let state = (&p.sender_state).try_into().map_err(RpcError::wire)?;
            // ⚠️ **Toda reserva necesita una salida que NO sea el reloj.**
            // Con exito la suelta `commit_send` en la capa; con FALLO no la
            // soltaba nadie, y en D.1 cada reintento abandonaba una: 16-18
            // huerfanas por corrida, contadas por el propio nodo. Como
            // `allocate_pending` las recorre en cada llamada, el arreglo de
            // la reserva costaba un 17 % de rendimiento (1,72 -> 1,42
            // pagos/s, t=4,2). Medido en §220 antes de sellarlo.
            //
            // Un recibo que no se aplico esta MUERTO: su prueba esta atada a
            // una raiz que ya se movio, y el titular regenerara con
            // materiales nuevos. Soltar su posicion es correcto.
            //
            // La posicion se lee DESPUES de `apply_send`: el tipo de
            // `receipt` lo fija esa llamada, y tocar un campo antes deja la
            // inferencia coja.
            // ⚠️⚠️ EL CONTADOR VA AQUI (§253): despues del parseo y la
            // conversion —eso es ruido, no una operacion— y ANTES de la
            // capa. Cuenta LO QUE EL NODO LLEGA A EVALUAR, y por eso lo
            // consume TAMBIEN si la capa rechaza: ahi es donde se
            // esconderia un censor, alegando prueba invalida.
            let rx = recibir(app)?;
            let r = l.apply_send(&receipt, p.sender.0, &state, p.amount.0);
            let pos = receipt.notice.position;
            reservas.remove(&pos);
            if let Err(e) = r {
                l.release_pending(pos);
                // ⚠️ El numero viaja TAMBIEN en el error: el caso que
                // importa es justo el rechazo.
                return Err(RpcError::layer(e).con_recepcion(rx));
            }
            Ok(con_rx(con_acuse(applied(l), l), rx))
        }

        "zkssl_claimMaterials" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct P { receiver: Q, notice: wire::PendingNoticeDto, view_key: wire::B32 }
            let p: P = parse(params)?;
            // ⚠️ §261: el aviso NO autenticaba —`claim_materials` solo usa
            //    `notice.position`—, asi que este brazo entregaba el camino
            //    del receptor a quien inventara un aviso. §259 dijo lo
            //    contrario y estaba equivocado.
            exige_credencial(l, p.receiver.0, &p.view_key)?;
            let notice = (&p.notice).try_into().map_err(RpcError::wire)?;
            let m = l.claim_materials(p.receiver.0, &notice).map_err(RpcError::layer)?;
            Ok(serde_json::to_value(wire::ClaimMaterialsDto::from(&m)).unwrap())
        }

        "zkssl_applyClaim" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct P {
                receipt: wire::ClaimReceiptDto,
                receiver: Q,
                receiver_state: wire::ClientStateDto,
                notice: wire::PendingNoticeDto,
            }
            let p: P = parse(params)?;
            let receipt = (&p.receipt).try_into().map_err(RpcError::wire)?;
            let state = (&p.receiver_state).try_into().map_err(RpcError::wire)?;
            let notice = (&p.notice).try_into().map_err(RpcError::wire)?;
            let rx = recibir(app)?;
            l.apply_claim(&receipt, p.receiver.0, &state, &notice)
                .map_err(|e| RpcError::layer(e).con_recepcion(rx))?;
            Ok(con_rx(con_acuse(applied(l), l), rx))
        }

        // ── lote: N operaciones contra UNA raiz de arranque ──────
        //
        // ⚠️ **Aditivo a proposito.** `applySend` y `applyClaim` no se
        // tocan: cambiar su respuesta sincrona subiria de version, y
        // esto no. Quien quiera la respuesta operacion a operacion la
        // sigue teniendo; quien quiera lote usa esto.
        //
        // ⚠️ **El lote lo arma QUIEN LLAMA.** El nodo no acumula: se
        // midio (§222, banco E.2) que juntar los recibos fuera o dentro
        // del nodo se diferencia en **0,08 % del ciclo**, y acumular
        // dentro exigiria estado, vales, otra caducidad y rehacer la
        // concurrencia de `handle`/`dispatch`. Si alguna vez hace falta
        // que titulares SIN coordinar se beneficien, sera una capa
        // encima de esto, no en su lugar.
        //
        // ⚠️ **El lote MIXTO —envíos y cobros juntos— SE ADMITE.** Lo
        // midió J.1 (§232): `apply_many` solo exige cuentas distintas y
        // posiciones distintas, y **ninguna de las dos comprobaciones mira
        // la clase**. Un cobro consume una posición vieja; un envío crea
        // una nueva. Un cobro del pendiente que crea un envío del MISMO
        // lote sí se rechaza, con `DuplicatePendingInBatch`.
        //
        // Y sin embargo el modelo de despliegue recomendado los SEPARA en
        // dos agregadores (`SECURITY.md` §2.ter). **No es por rendimiento
        // ni un descuido**: es porque cada mitad revela media arista
        // —emisor+importe, o receptor+importe— y separarlas hace que
        // ninguna pieza fuera del nodo vea el grafo de pagos (§231).
        // El código admite las dos formas a propósito: la spec manda
        // sobre el cable, no sobre quién opera qué.
        //
        // ⚠️ **Cabe sin tocar el transporte.** El muro del cuerpo son
        // 2.097.152 bytes (§218, banco C0.2) y una operacion ronda los
        // 132.728 en hex: entran **15**. La carga que §216 midio —8
        // pagos— son 8 envios y 8 cobros, y no pueden ir juntos de todas
        // formas: un cobro necesita que su pendiente exista, y
        // `apply_many` los rechazaria por `DuplicatePendingInBatch`.
        // Van en dos lotes de 8, que son 1,06 MB: la mitad del muro.
        "zkssl_applyMany" => {
            #[derive(Deserialize)]
            #[serde(tag = "kind", rename_all = "camelCase")]
            enum OpDto {
                #[serde(rename_all = "camelCase")]
                Send {
                    receipt: wire::SendReceiptDto,
                    sender: Q,
                    sender_state: wire::ClientStateDto,
                    amount: Q,
                },
                #[serde(rename_all = "camelCase")]
                Claim {
                    receipt: wire::ClaimReceiptDto,
                    receiver: Q,
                    receiver_state: wire::ClientStateDto,
                    notice: wire::PendingNoticeDto,
                },
            }
            #[derive(Deserialize)]
            struct P { ops: Vec<OpDto> }

            // Los tipos PROPIOS viven aqui porque `BatchOp` los presta:
            // hay que tenerlos vivos mientras dure `apply_many`.
            enum Propia {
                Send(SendReceipt, AccountIndex, ClientState, u64),
                Claim(ClaimReceipt, AccountIndex, ClientState, PendingNotice),
            }

            let p: P = parse(params)?;
            // ⚠️ El lote VACIO se rechaza: `apply_many` devuelve Ok(())
            // con cero operaciones y el sobre de respuesta no tendria
            // ni `fromSeq` ni `rootOld` que dar.
            if p.ops.is_empty() {
                return Err(RpcError::invalid_params("lote vacio"));
            }

            let mut propias: Vec<Propia> = Vec::with_capacity(p.ops.len());
            for op in &p.ops {
                propias.push(match op {
                    OpDto::Send { receipt, sender, sender_state, amount } => Propia::Send(
                        receipt.try_into().map_err(RpcError::wire)?,
                        sender.0,
                        sender_state.try_into().map_err(RpcError::wire)?,
                        amount.0,
                    ),
                    OpDto::Claim { receipt, receiver, receiver_state, notice } => Propia::Claim(
                        receipt.try_into().map_err(RpcError::wire)?,
                        receiver.0,
                        receiver_state.try_into().map_err(RpcError::wire)?,
                        notice.try_into().map_err(RpcError::wire)?,
                    ),
                });
            }

            // Las posiciones que el lote toca, para soltarlas si falla.
            let posiciones: Vec<u64> = propias
                .iter()
                .map(|o| match o {
                    Propia::Send(r, ..) => r.notice.position,
                    Propia::Claim(_, _, _, n) => n.position,
                })
                .collect();

            let ops: Vec<BatchOp<'_>> = propias
                .iter()
                .map(|o| match o {
                    Propia::Send(receipt, sender_index, sender_state, amount) => {
                        BatchOp::Send {
                            receipt,
                            sender_index: *sender_index,
                            sender_state,
                            amount: *amount,
                        }
                    }
                    Propia::Claim(receipt, receiver_index, receiver_state, notice) => {
                        BatchOp::Claim {
                            receipt,
                            receiver_index: *receiver_index,
                            receiver_state,
                            notice,
                        }
                    }
                })
                .collect();

            let antes = l.transition_log().len();
            let raiz_antes = digest_to_wire(&l.state_root());
            let r = l.apply_many(&ops);
            drop(ops);

            // ⚠️ **Soltar las reservas pase lo que pase.** `apply_many`
            // valida TODO o nada: un lote rechazado dejaria N posiciones
            // reservadas sin dueno, y `allocate_pending` las recorre en
            // cada llamada. Es la leccion de §220 —donde el mismo
            // descuido costo un 17 % de rendimiento— aplicada ANTES de
            // que la cobre un banco. Con exito las quita `commit_send`;
            // aqui se retiran del reloj en los dos casos.
            for pos in &posiciones {
                reservas.remove(pos);
            }
            if let Err(e) = r {
                for pos in &posiciones {
                    l.release_pending(*pos);
                }
                return Err(RpcError::layer(e));
            }

            // El sobre: `applied` en ORDEN DE ENTRADA, que es el mismo en
            // que `apply_many` aplica (two_phase.rs, paso 4). Y
            // `batch.rootOld` es lo unico que el lote puede devolver a
            // cambio de lo que quita: dentro de un lote cada prueba
            // acredita una transicion contra la RAIZ DE ARRANQUE, no
            // contra la que el registro anota (spec/RPC.md, §213).
            let entradas = l.transition_log().entries();
            let nuevas = &entradas[antes..];
            let aplicadas: Vec<Value> = nuevas
                .iter()
                .map(|e| {
                    json!({
                        "logSeq": Q(e.seq),
                        "kind": format!("{:?}", e.kind),
                        "accountsRoot": digest_to_wire(&e.root_new),
                        "chain": digest_to_wire(&e.chain),
                    })
                })
                .collect();
            Ok(json!({
                "batch": {
                    "size": Q(nuevas.len() as u64),
                    "fromSeq": nuevas.first().map(|e| Q(e.seq)),
                    "toSeq": nuevas.last().map(|e| Q(e.seq)),
                    "rootOld": raiz_antes,
                    "rootNew": digest_to_wire(&l.state_root()),
                    "chain": digest_to_wire(&l.log_head()),
                },
                "applied": aplicadas,
            }))
        }

        // ── dev_* : SOLO con --dev (custodios de prueba) ───────────
        m if m.starts_with("dev_") => dispatch_dev(app, l, m, params),

        other => Err(RpcError::method_not_found(other)),
    }
}

/// Respuesta común de las escrituras: qué entrada quedó en el registro.
/// Reserva el siguiente número de recepción, con `fsync`.
///
/// ⚠️ El candado se toma y **se suelta aquí dentro**: reservar cuesta un
/// `fsync` y retenerlo durante la operación pararía el nodo. Mismo
/// criterio que el latido (§241, medido en §252).
fn recibir(app: &App) -> Result<u64, RpcError> {
    app.recepcion
        .lock()
        .map_err(|_| RpcError {
            code: -32603,
            message: "candado del contador de recepcion envenenado".into(),
        })?
        .recibir()
        .map_err(|e| RpcError { code: -32603, message: format!("{e}") })
}

/// Añade el número de recepción a una respuesta.
///
/// ⚠️ **Un contador que el titular no puede ver no le sirve para detectar
/// nada**, así que viaja en el cable — pero **NO está firmado**: `chain`
/// autentica `seq`, `kind`, las raíces, el digest de prueba y el anterior,
/// **y nada más**. Es un número que el nodo dice y **que nada ata**.
/// §274 · **El acuse en la respuesta del titular** — la instrucción
/// autosuficiente: el titular guarda una cosa y esa cosa dice cómo
/// completarse.
///
/// - `epoca` = `seq + 1`: la **primera cabeza que puede contener** la
///   operación (regla en `zk_ssl_verify::acuses`, la misma que usará el
///   verificador de §275).
/// - `n` = el techo declarado (`vista_acuses::N_MAX_CABEZAS`).
/// - `hashPrueba` = el `proof_digest` asentado — real en las vías del
///   titular (§273).
///
/// ⚠️ **Sólo lo llaman `applySend` y `applyClaim`**: las delegadas no
/// emiten acuse (el corte, §273). Y la respuesta **no va firmada** — el
/// acuse hereda la firma al cerrar la época, bajo la raíz (§275); el
/// límite está declarado en `spec/RPC.md` y en el asiento §274.
fn con_acuse(mut v: Value, l: &SovereignLayer) -> Value {
    if let Some(e) = l.transition_log().entries().last() {
        v["acuse"] = json!({
            "epoca": Q(zk_ssl_verify::acuses::epoca_de_acuse(e.seq)),
            "n": Q(vista_acuses::N_MAX_CABEZAS),
            "hashPrueba": digest_to_wire(&e.proof_digest),
        });
    }
    v
}

fn con_rx(mut v: Value, rx: u64) -> Value {
    v["receptionSeq"] = json!(Q(rx));
    v
}

fn applied(l: &SovereignLayer) -> Value {
    let last = l.transition_log().entries().last();
    json!({
        "logSeq": last.map(|e| Q(e.seq)),
        "kind": last.map(|e| format!("{:?}", e.kind)),
        "accountsRoot": digest_to_wire(&l.state_root()),
        "chain": digest_to_wire(&l.log_head()),
    })
}

#[cfg(feature = "dev")]
fn dispatch_dev(
    app: &App,
    l: &mut SovereignLayer,
    method: &str,
    params: Value,
) -> Result<Value, RpcError> {
    use zk_ssl::tests_support as ts;

    if !app.dev {
        return Err(RpcError {
            code: -32601,
            message: "dev_* deshabilitado: arrancar con --dev".into(),
        });
    }

    match method {
        // Grifo de anvil, versión ZK-SSL: emisión delegada real con los
        // DOS custodios de la suite, nullifiers de umbral incluidos.
        "dev_fund" => {
            #[derive(Deserialize)]
            struct P { index: Q, amount: Q }
            let p: P = parse(params)?;
            let op = ts::mint_commitment(l, p.index.0, p.amount.0);
            let subida = ts::mint_climb_proof(l, p.index.0, p.amount.0);
            let (pa, ia, pb, ib) = ts::delegated_pair(op, 1, 3);
            let nullifiers = [digest_to_wire(&ia.nullifier), digest_to_wire(&ib.nullifier)];
            l.apply_mint_delegated(subida, pa, ia, pb, ib, p.index.0, p.amount.0)
                .map_err(RpcError::layer)?;
            let mut out = applied(l);
            out["custodianNullifiers"] = serde_json::to_value(nullifiers).unwrap();
            Ok(out)
        }

        // Comodidad de sandbox: abre desde una clave DETERMINISTA de la
        // suite. En producción se abre con zkssl_openAccount.
        "dev_openSeeded" => {
            #[derive(Deserialize)]
            struct P { seed: Q }
            let p: P = parse(params)?;
            let sk = ts::wide_key(p.seed.0);
            let index = l.open_account_wide(sk);
            Ok(json!({
                "index": Q(index),
                "publicId": digest_to_wire(&stark_experiment::native::derive_public_id_wide(sk)),
                "viewKey": digest_to_wire(&stark_experiment::native::derive_view_key_wide(sk)),
            }))
        }

        other => Err(RpcError::method_not_found(other)),
    }
}

#[cfg(not(feature = "dev"))]
fn dispatch_dev(
    _app: &App,
    _l: &mut SovereignLayer,
    method: &str,
    _params: Value,
) -> Result<Value, RpcError> {
    Err(RpcError::method_not_found(method))
}

fn init_tracing(filter: &str) {
    use tracing_subscriber::EnvFilter;
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_new(filter).unwrap_or_else(|_| EnvFilter::new("info")))
        .with_writer(std::io::stderr)
        .with_target(false)
        .compact()
        .init();
}

// ─────────────────────────── TESTS DEL NODO ───────────────────────────
//
// §228. **El nodo llevaba 672 líneas y dieciocho métodos sin un solo
// test.** Su pin en `tools/canon.sh` era 0, declarado como hueco, no como
// aprobación. Lo único que lo verificaba era una prueba de humo por RPC
// —que el método responde— y las suites de la capa, que no tocan el nodo.
//
// ## Qué se prueba aquí, y por qué esto y no otra cosa
//
// `dispatch` es una función libre sobre `&App`: se puede llamar sin
// levantar HTTP, sin puertos y sin tokio. Eso permite probar la LÓGICA DEL
// NODO —despacho, reservas, barrido, errores— en milisegundos.
//
// Lo que NO se prueba aquí, y se dice para que nadie lo suponga:
// - **Nada que exija una prueba STARK.** `applySend` y `applyClaim` con
//   recibos reales cuestan ~250 ms de generación por operación y viven en
//   los bancos D.1 y D.2, no en la suite.
// - **El transporte**: axum, el muro del cuerpo, la serialización HTTP.
//   Eso lo miden C0.2 y E.2.
//
// ## Los que importan de verdad
//
// Tres de estos tests corresponden a fallos que ESTA CASA YA TUVO:
// - `sendMaterials` repartía posiciones REPETIDAS (§220), porque
//   `allocate_pending` es pura y el candado no cambia su resultado.
// - un `applySend` fallido NO soltaba su reserva (§220), y eso costó un
//   **17 % de rendimiento** medido.
// - el lote vacío devolvía un sobre sin `fromSeq` ni `rootOld` (§222).
//
// Un test que reproduce un fallo ya ocurrido vale más que diez inventados.
#[cfg(test)]
mod tests {
    use super::*;

    /// Un nodo en memoria, con `--dev`, y la caducidad que pida el test.
    ///
    /// ⚠️ `pub(crate)` para que **el módulo del latido** pueda usarlo
    /// (§241). Un helper duplicado en dos sitios se desincroniza; uno
    /// compartido no.
    pub(crate) fn nodo(ttl_segundos: u64) -> App {
        let layer = SovereignLayer::new(
            zk_ssl::tests_support::custodian_root(),
            zk_ssl::tests_support::governance_root(),
            500_000,
            100_000_000,
            1_000,
        );
        App {
            estado: Mutex::new(Estado { layer, reservas: BTreeMap::new() }),
            dev: true,
            reserva_ttl: Duration::from_secs(ttl_segundos),
            ultima_cabeza: Mutex::new(None),
            hojas_mmr: Mutex::new(Vec::new()),
            clave_publica_firma: Vec::new(),
            diario: None,
            latido_s: crate::latido::LATIDO_POR_DEFECTO_S,
            custodia: "sin-declarar".into(),
            custodia_comprobada: false,
            cofirmas: Mutex::new(BTreeMap::new()),
            max_cofirmas: 32,
            recepcion: Mutex::new(
                recepcion::ContadorRecepcion::abrir(
                    tests_dir(&format!("rec_{}", proximo_nodo())).join("recepcion.bin"),
                )
                .expect("contador de recepcion"),
            ),
        }
    }

    /// Abre una cuenta con semilla `s` y la fondea. Devuelve (indice, id).
    /// ⚠️ §261: DEVUELVE TAMBIEN LA CLAVE DE VISTA, que `dev_openSeeded`
    /// siempre entrego y este ayudante TIRABA. Mientras la tiraba, los
    /// tests del nodo median **una idea del cliente mas pobre que la
    /// real** — y por eso nueve llamadas pedian caminos sin credencial: no
    /// porque el modelo lo permitiera, sino porque el andamio la
    /// descartaba. Es la familia de §250.
    pub(crate) fn cuenta(app: &App, s: u64, saldo: u64) -> (u64, Value, Value) {
        let r = dispatch(app, "dev_openSeeded", json!({ "seed": Q(s) })).expect("abrir");
        let idx = r["index"].as_str().expect("index").to_string();
        let idx = u64::from_str_radix(idx.trim_start_matches("0x"), 16).expect("hex");
        dispatch(app, "dev_fund", json!({ "index": Q(idx), "amount": Q(saldo) })).expect("fondear");
        (idx, r["publicId"].clone(), r["viewKey"].clone())
    }

    fn reservas_vivas(app: &App) -> usize {
        app.estado.lock().expect("mutex").reservas.len()
    }

    /// Un digest a partir de un u64, para las sales de los tests.
    fn sal(n: u64) -> [winterfell::math::fields::f64::BaseElement; 4] {
        use winterfell::math::fields::f64::BaseElement as E;
        [E::new(n), E::new(0), E::new(0), E::new(0)]
    }

    fn materiales(app: &App, emisor: u64, vk: &Value, receptor: &Value, importe: u64, salt: u64) -> Value {
        dispatch(
            app,
            "zkssl_sendMaterials",
            json!({
                "sender": Q(emisor),
                "viewKey": vk,
                "receiverId": receptor,
                "amount": Q(importe),
                "salt": digest_to_wire(&sal(salt)),
            }),
        )
        .expect("materiales")
    }

    fn posicion(m: &Value) -> u64 {
        let p = m["pendingPosition"].as_str().expect("pendingPosition");
        u64::from_str_radix(p.trim_start_matches("0x"), 16).expect("hex")
    }

    // ── el despacho ───────────────────────────────────────────────

    #[test]
    fn un_metodo_desconocido_da_32601_y_no_toca_nada() {
        let app = nodo(30);
        let e = dispatch(&app, "zkssl_loQueSea", json!({})).expect_err("deberia fallar");
        assert_eq!(e.code, -32601, "metodo desconocido debe ser MethodNotFound");
        assert_eq!(reservas_vivas(&app), 0);
    }

    // ── §253: el contador de recepcion ──

    #[test]
    fn el_numero_de_recepcion_viaja_al_titular() {
        // ⚠️ Un contador que el titular NO PUEDE VER no le sirve para
        // detectar nada.
        let v = con_rx(json!({"logSeq": Q(3)}), 7);
        assert_eq!(v["receptionSeq"], json!("0x7"), "y en el hex del cable");
        assert_eq!(v["logSeq"], json!("0x3"), "sin tocar lo que ya habia");
    }

    #[test]
    fn el_numero_de_recepcion_viaja_tambien_en_el_error() {
        // ⚠️⚠️ EL CASO QUE IMPORTA: un censor se esconderia RECHAZANDO, asi
        // que el titular necesita su numero JUSTO cuando le dicen que no.
        let e = RpcError { code: -32000, message: "prueba invalida".into() }
            .con_recepcion(9);
        assert!(e.message.contains("receptionSeq=0x9"), "{}", e.message);
        assert!(e.message.contains("prueba invalida"), "sin perder el motivo");
    }

    // ── §244: la custodia, afirmada frente a comprobada ──

    #[test]
    fn la_custodia_viaja_siempre_y_por_defecto_es_sin_declarar() {
        // ⚠️ Si el campo se omitiera al no declarar nada, un consumidor no
        // distinguiria "no declara" de "version vieja del nodo".
        let app = nodo(30);
        let v = dispatch(&app, "zkssl_signedEpochHead", json!({})).expect("no debe fallar");
        assert_eq!(v["custody"], json!("sin-declarar"), "presente y honesto por defecto");
        assert_eq!(v["custodyChecked"], json!(false));
    }

    #[test]
    fn la_custodia_viaja_tambien_cuando_hay_firma() {
        let app = nodo(30);
        let l = crate::latido::latir(&app, None).expect("latir");
        crate::latido::conservar(&app, l);
        let v = dispatch(&app, "zkssl_signedEpochHead", json!({})).expect("no debe fallar");
        assert!(!v["custody"].is_null(), "la custodia viaja aunque no haya firma");
        assert!(!v["custodyChecked"].is_null());
    }

    #[test]
    fn un_fichero_de_clave_legible_por_otros_se_rechaza() {
        // ⚠️ El keystore de §199 creaba con 0600; NADIE comprobaba al leer.
        // Crear bien no impide que alguien afloje despues.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let d = crate::tests_dir("clave_permisos");
            let p = d.join("semilla.hex");
            std::fs::write(&p, "00".repeat(96)).expect("escribir");

            std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o644)).expect("chmod");
            let e = leer_semilla_de_fichero(p.to_str().expect("ruta"))
                .expect_err("un fichero legible por otros debe rechazarse");
            assert!(format!("{e}").contains("legible por grupo u otros"), "{e}");

            std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o600)).expect("chmod");
            let s = leer_semilla_de_fichero(p.to_str().expect("ruta")).expect("con 0600 debe leer");
            assert_eq!(s.len(), 192);
        }
    }

    // ── §242: el metodo del TESTIGO ──

    /// Lee una QUANTITY del cable: hexadecimal con 0x, como {:#x}.
    ///
    /// ⚠️ Se escribio primero asumiendo DECIMAL y el test cayo con
    /// left: \"0x1\" · right: \"1\" (§242). **El formato del cable se LEE,
    /// no se supone**, aunque sea el de una cifra de un digito.
    fn q_de(v: &Value) -> u64 {
        let s = v.as_str().expect("QUANTITY es una cadena");
        u64::from_str_radix(s.trim_start_matches("0x"), 16).expect("hex")
    }

    #[test]
    fn sin_latido_todavia_el_metodo_lo_dice_y_no_falla() {
        // ⚠️ Ninguna de las tres respuestas es un error generico.
        let app = nodo(30);
        let v = dispatch(&app, "zkssl_signedEpochHead", json!({})).expect("no debe fallar");
        assert_eq!(v["available"], json!(false));
        assert!(v["reason"].as_str().expect("reason").contains("aun no ha habido latido"));
    }

    #[test]
    fn sin_clave_el_metodo_sirve_la_cabeza_y_dice_que_no_hay_firma() {
        let app = nodo(30);
        let l = crate::latido::latir(&app, None).expect("latir");
        crate::latido::conservar(&app, l);
        let v = dispatch(&app, "zkssl_signedEpochHead", json!({})).expect("no debe fallar");
        assert_eq!(v["available"], json!(false));
        assert!(v["reason"].as_str().expect("reason").contains("SIN --clave"));
        assert!(v["epochDigest"].is_string(), "la CABEZA si viaja, aunque no haya firma");
        assert!(v["signature"].is_null(), "y la firma NO");
    }

    #[test]
    fn con_firma_el_metodo_da_todo_lo_que_un_testigo_necesita() {
        use crate::firma_cabeza::{verificar_cabeza, CabezaFirmada, FirmanteCabeza};
        let app = nodo(30);
        let d = crate::tests_dir("rpc_testigo");
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).expect("crear");
        let mut s = [0u8; 96];
        for (i, b) in s.iter_mut().enumerate() {
            *b = (i as u8).wrapping_mul(13).wrapping_add(2);
        }
        let mut f = FirmanteCabeza::desde_semilla(&s, d.join("indice.bin")).expect("abrir");
        let pk = f.clave_publica();
        let l = crate::latido::latir(&app, Some(&mut f)).expect("latir");
        crate::latido::conservar(&app, l);

        // ⚠️ El nodo de prueba se construye SIN clave publica; se sirve la
        // suya, asi que aqui se comprueba el resto y se verifica con `pk`.
        let v = dispatch(&app, "zkssl_signedEpochHead", json!({})).expect("no debe fallar");
        assert_eq!(v["available"], json!(true));
        for k in ["seq", "epochDigest", "domain", "formatVersion", "index",
                  "signature", "publicKey", "emittedAtUnix", "beatSeconds"] {
            assert!(!v[k].is_null(), "falta el campo {k}");
        }
        assert_eq!(v["domain"], json!("ZK-SSL-epoch-head"));
        assert_eq!(v["index"], json!("0x1"), "Q es una cantidad HEX del cable");

        // ⚠️ LO QUE IMPORTA: lo servido se VERIFICA, sin el nodo.
        let hex = v["signature"].as_str().expect("signature").trim_start_matches("0x");
        let firma: Vec<u8> = (0..hex.len() / 2)
            .map(|i| u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).expect("hex"))
            .collect();
        let dig = v["epochDigest"].as_str().expect("digest").trim_start_matches("0x");
        let mut digest = [0u8; 32];
        for i in 0..32 {
            digest[i] = u8::from_str_radix(&dig[i * 2..i * 2 + 2], 16).expect("hex");
        }
        let c = CabezaFirmada {
            version_formato: q_de(&v["formatVersion"]) as u8,
            indice: q_de(&v["index"]),
            firma,
        };
        verificar_cabeza(&pk, &digest, &c).expect("lo que sirve el RPC debe verificar");
    }

    #[test]
    fn el_metodo_nuevo_no_toca_la_version_ni_la_cabeza() {
        // ⚠️ ADITIVO: `zkssl_epochHead` y la version siguen igual. Los
        // vectores de conformidad de 0.2 no se mueven.
        let app = nodo(30);
        assert_eq!(
            dispatch(&app, "zkssl_protocolVersion", json!([])).expect("version"),
            json!("zkssl/0.2")
        );
        let h = dispatch(&app, "zkssl_epochHead", json!({})).expect("epochHead");
        assert!(h["signature"].is_null(), "epochHead NO debe llevar firma");
        assert!(h["epochDigest"].is_string());
    }

    #[test]
    fn la_version_del_protocolo_es_la_declarada() {
        let app = nodo(30);
        let v = dispatch(&app, "zkssl_protocolVersion", json!([])).expect("version");
        // ⚠️ Si esto cambia, cambia el CABLE. spec/RPC.md §versionado: lo
        // que sube version es que cambien los VALORES que viajan.
        assert_eq!(v, json!("zkssl/0.2"));
    }

    #[test]
    fn sin_dev_los_metodos_dev_se_rechazan() {
        let mut app = nodo(30);
        app.dev = false;
        let e = dispatch(&app, "dev_fund", json!({ "index": Q(0), "amount": Q(1) }))
            .expect_err("dev_* sin --dev debe rechazarse");
        assert_eq!(e.code, -32601, "sin --dev, dev_* no existe");
    }

    // ── EL FALLO DE §220: posiciones repetidas ────────────────────

    #[test]
    fn dos_peticiones_de_materiales_reciben_posiciones_distintas() {
        // ⚠️ EL TEST QUE HABRIA CAZADO EL FALLO DE §220.
        // `allocate_pending` es PURA: mira el estado y no muta. El candado
        // de `dispatch` serializa las peticiones pero NO cambia su
        // resultado, asi que dos titulares recibian la MISMA posicion y el
        // segundo moria al aplicar. El nodo llevaba nueve sellos sin usar
        // `reserve_pending`, que la capa tenia desde §211.
        let app = nodo(30);
        let (a, _, vk_a) = cuenta(&app, 1, 100_000);
        let (b, _, vk_b) = cuenta(&app, 2, 100_000);
        let (_, id_c, _) = cuenta(&app, 3, 0);

        let m1 = materiales(&app, a, &vk_a, &id_c, 1_000, 7);
        let m2 = materiales(&app, b, &vk_b, &id_c, 1_000, 8);

        assert_ne!(
            posicion(&m1),
            posicion(&m2),
            "CRITICO: dos titulares han recibido la MISMA posicion de pendiente. \
             Un lote con las dos seria rechazado entero por DuplicatePendingInBatch, \
             y aplicandolas de una en una la segunda muere."
        );
        assert_eq!(reservas_vivas(&app), 2, "cada materiales deja SU reserva");
    }

    #[test]
    fn muchas_peticiones_seguidas_no_repiten_ninguna_posicion() {
        let app = nodo(30);
        let (a, _, vk_a) = cuenta(&app, 10, 1_000_000);
        let (_, id, _) = cuenta(&app, 11, 0);
        let mut vistas = std::collections::BTreeSet::new();
        for i in 0..15u64 {
            let m = materiales(&app, a, &vk_a, &id, 1_000, 100 + i);
            assert!(vistas.insert(posicion(&m)), "posicion repetida en la iteracion {i}");
        }
        assert_eq!(vistas.len(), 15);
        assert_eq!(reservas_vivas(&app), 15);
    }

    // ── EL FALLO DE §220: reservas que nadie suelta ───────────────

    #[test]
    fn si_la_capa_rechaza_los_materiales_la_reserva_se_suelta() {
        // ⚠️ Sin esto, CADA peticion rechazada —cuenta congelada, saldo
        // insuficiente, limite regulatorio— dejaria una posicion muerta.
        // Y un atacante no necesitaria ni saldo para provocarlas.
        let app = nodo(30);
        let (pobre, _, vk_pobre) = cuenta(&app, 20, 10);
        let (_, id, _) = cuenta(&app, 21, 0);

        let e = dispatch(
            &app,
            "zkssl_sendMaterials",
            json!({
                "sender": Q(pobre),
                "viewKey": vk_pobre,
                "receiverId": id,
                "amount": Q(999_999u64),
                "salt": digest_to_wire(&sal(1)),
            }),
        )
        .expect_err("saldo insuficiente debe rechazarse");
        assert_eq!(e.code, -32000, "un rechazo de la capa es error de capa");
        assert_eq!(
            reservas_vivas(&app),
            0,
            "CRITICO: la peticion fue rechazada y la reserva se quedo colgada"
        );
    }

    // ── EL BARRIDO PEREZOSO ───────────────────────────────────────

    #[test]
    fn el_barrido_libera_las_reservas_caducadas_en_la_siguiente_peticion() {
        // Caducidad de 0 s: todo lo reservado esta caduco al instante.
        // El barrido corre AL ENTRAR en dispatch, asi que hace falta UNA
        // peticion mas para que se note — y eso es correcto: un nodo
        // ocioso mantiene sus reservas hasta que alguien llama.
        let app = nodo(0);
        let (a, _, vk_a) = cuenta(&app, 30, 100_000);
        let (_, id, _) = cuenta(&app, 31, 0);

        materiales(&app, a, &vk_a, &id, 1_000, 42);
        assert_eq!(reservas_vivas(&app), 1, "la reserva existe nada mas crearse");

        std::thread::sleep(Duration::from_millis(20));
        dispatch(&app, "zkssl_accountCount", json!({})).expect("una peticion cualquiera");
        assert_eq!(reservas_vivas(&app), 0, "el barrido no libero la reserva caducada");
    }

    #[test]
    fn con_caducidad_larga_el_barrido_no_toca_nada() {
        let app = nodo(3_600);
        let (a, _, vk_a) = cuenta(&app, 40, 100_000);
        let (_, id, _) = cuenta(&app, 41, 0);
        materiales(&app, a, &vk_a, &id, 1_000, 43);
        for _ in 0..5 {
            dispatch(&app, "zkssl_accountCount", json!({})).expect("peticion");
        }
        assert_eq!(reservas_vivas(&app), 1, "una reserva viva no debe barrerse");
    }

    // ── EL LOTE (§222) ────────────────────────────────────────────

    #[test]
    fn el_lote_vacio_se_rechaza_y_dice_por_que() {
        // `apply_many` devuelve Ok(()) con cero operaciones, asi que sin
        // esta guarda el sobre no tendria ni `fromSeq` ni `rootOld`.
        let app = nodo(30);
        let e = dispatch(&app, "zkssl_applyMany", json!({ "ops": [] }))
            .expect_err("el lote vacio debe rechazarse");
        assert_eq!(e.code, -32602);
        assert!(
            e.message.contains("lote vacio"),
            "el rechazo debe decir POR QUE, y dijo: {}",
            e.message
        );
    }

    #[test]
    fn una_operacion_de_clase_desconocida_rechaza_el_lote() {
        let app = nodo(30);
        let e = dispatch(&app, "zkssl_applyMany", json!({ "ops": [{ "kind": "vuelo" }] }))
            .expect_err("clase desconocida debe rechazarse");
        assert_eq!(e.code, -32602);
    }

    #[test]
    fn apply_many_es_aditivo_los_sueltos_siguen_existiendo() {
        // ⚠️ Esto es lo que hace legitimo NO subir de version: la
        // superficie es aditiva y los valores de cable no se movieron.
        // Si alguien "simplificara" quitando applySend, esto se pone rojo.
        let app = nodo(30);
        for m in [
            "zkssl_applySend",
            "zkssl_applyClaim",
            "zkssl_sendMaterials",
            "zkssl_claimMaterials",
            "zkssl_applyMany",
        ] {
            let e = dispatch(&app, m, json!({})).expect_err("params vacios: debe fallar");
            assert_ne!(e.code, -32601, "{m} ha desaparecido del despacho");
        }
    }

    // --- el contrato PUBLICADO y el despacho, atados ---
    /// CEGUERA DECLARADA de este censo (302). Lee la FUENTE de este mismo
    /// fichero y reconoce un brazo como un literal entre comillas seguido,
    /// en la MISMA linea, de => o de |. NO ve: un brazo construido
    /// dinamicamente, un multipatron partido por un salto de linea, ni un
    /// literal con comillas escapadas. Y por el lado del nombre censa los
    /// brazos que empiezan por zkssl_ mas los de EXCEPCIONES: un metodo con
    /// un prefijo NUEVO seria invisible aqui, y por eso la otra direccion
    /// (cable subconjunto del despacho) se comprueba nombre a nombre y SIN
    /// filtrar por prefijo, y hay un assert que exige que todo nombre del
    /// cable lleve un prefijo conocido.
    #[test]
    fn el_despacho_y_el_documento_publican_los_mismos_metodos() {
        const EXCEPCIONES: &[&str] = &["dev_fund", "dev_openSeeded"];
        let fuente = include_str!("main.rs");
        let mut brazos: Vec<String> = Vec::new();
        for linea in fuente.lines() {
            let cs: Vec<char> = linea.chars().collect();
            let mut i = 0usize;
            while i < cs.len() {
                if cs[i] != '"' { i += 1; continue; }
                let ini = i + 1;
                let mut j = ini;
                while j < cs.len() && cs[j] != '"' { j += 1; }
                if j >= cs.len() { break; }
                let lit: String = cs[ini..j].iter().collect();
                let resto: String = cs[j + 1..].iter().collect();
                let r = resto.trim_start();
                if r.starts_with("=>") || r.starts_with('|') { brazos.push(lit); }
                i = j + 1;
            }
        }
        let del_despacho: std::collections::BTreeSet<String> = brazos
            .into_iter()
            .filter(|n| n.starts_with("zkssl_") || EXCEPCIONES.contains(&n.as_str()))
            .collect();
        let del_cable: std::collections::BTreeSet<String> =
            zk_ssl_wire::openrpc::method_names().iter().map(|s| s.to_string()).collect();
        for n in &del_cable {
            assert!(
                n.starts_with("zkssl_") || EXCEPCIONES.contains(&n.as_str()),
                "{n}: prefijo desconocido, el censo del despacho no lo veria"
            );
        }
        let solo_despacho: Vec<&String> = del_despacho.difference(&del_cable).collect();
        let solo_cable: Vec<&String> = del_cable.difference(&del_despacho).collect();
        assert!(
            solo_despacho.is_empty(),
            "DESPACHADOS y no publicados en el OpenRPC: {solo_despacho:?}"
        );
        assert!(
            solo_cable.is_empty(),
            "PUBLICADOS en el OpenRPC y no despachados: {solo_cable:?}"
        );
    }

    // ─────────── §315 · el transporte de la cofirma ───────────

    /// Una cofirma con la forma del cable. La firma es de mentira a
    /// proposito: estos tres tests miden los caminos de RECHAZO, que son los
    /// que se pueden construir sin fabricar una cofirma XMSS valida.
    ///
    /// ⚠️ **DECLARADO Y NO CUBIERTO: el camino de ACEPTACION no tiene test
    /// aqui.** Exige un cofirmante real —clave, guardian y preambulo de
    /// cofirma— que vive en el testigo, no en el nodo. Lo cierra el §316,
    /// cuando el testigo submita de verdad.
    fn cofirma_de(digest: &str) -> Value {
        json!({
            "v": "0x1",
            "epochDigest": digest,
            "clavePublicaOperador": "0xabcd",
            "clavePublicaTestigo": "0xbeef",
            "versionFormato": "0x3",
            "indice": "0x2",
            "firma": "0xdeadbeef",
            "vistoUnix": "0x66c0"
        })
    }

    #[test]
    fn sin_latido_una_cofirma_no_se_acepta_y_el_nodo_dice_por_que() {
        let app = nodo(30);
        let d = format!("0x{}", "11".repeat(32));
        let v = dispatch(&app, "zkssl_submitCosig", json!({ "cosig": cofirma_de(&d) }))
            .expect("no debe fallar: no hay epoca NO es un error del que llama");
        assert_eq!(v["accepted"], json!(false));
        assert_eq!(v["stored"], json!("0x0"));
        assert!(
            v["reason"].as_str().expect("reason").contains("latido"),
            "dio: {}", v["reason"]
        );
    }

    #[test]
    fn una_cofirma_de_otra_epoca_se_rechaza_antes_de_verificar_nada() {
        let app = nodo(30);
        let l = crate::latido::latir(&app, None).expect("latir");
        crate::latido::conservar(&app, l);
        // Un digest que NO es el de la cabeza en curso.
        let otro = format!("0x{}", "22".repeat(32));
        let v = dispatch(&app, "zkssl_submitCosig", json!({ "cosig": cofirma_de(&otro) }))
            .expect("no debe fallar");
        assert_eq!(v["accepted"], json!(false));
        assert!(
            v["reason"].as_str().expect("reason").contains("epoca en curso"),
            "dio: {}", v["reason"]
        );
    }

    #[test]
    fn pedir_cofirmas_de_una_epoca_que_no_es_la_actual_da_cero_y_no_es_error() {
        let app = nodo(30);
        let otro = format!("0x{}", "33".repeat(32));
        let v = dispatch(&app, "zkssl_cosigs", json!({ "epochDigest": otro }))
            .expect("no debe fallar: el nodo NO guarda historico y lo dice con un cero");
        assert_eq!(v["n"], json!("0x0"));
        assert_eq!(v["cosigs"].as_array().expect("cosigs").len(), 0);
        // Y sin parametro, con nodo recien arrancado, tampoco hay epoca.
        let v2 = dispatch(&app, "zkssl_cosigs", json!({})).expect("no debe fallar");
        assert_eq!(v2["n"], json!("0x0"));
    }

    // --- el brazo NO monta JSON a mano: lo monta el TIPO (309 -> 313) ---
    /// El brazo servia VEINTE campos en el caso firmado y el test de al lado
    /// asertaba NUEVE con `!is_null()`: once claves del contrato SIN gate del
    /// lado del productor. Y el conjunto mas completo vivia en un test del
    /// CONSUMIDOR (`cli/src/witness.rs`, diecinueve claves), que es la figura
    /// del §304 otra vez: la verdad mas completa, lejos de donde se produce.
    ///
    /// CORRECCION §247 — la frase de abajo dejo de ser cierta y se CITA, no se
    /// borra. Decia: *«TEMPORAL POR DISENO: cuando exista `SignedEpochHeadDto`,
    /// estas tres listas se derivan serializando el DTO y dejan de estar
    /// escritas a mano»*. El DTO existe desde §311 y el §313 lo pone a
    /// producir — pero **derivar las listas de el habria sido un ESPEJO**: si
    /// el dispatch construye y serializa el DTO, comparar su salida contra la
    /// serializacion del DTO no puede fallar nunca, y un banco sin su rojo es
    /// un adorno. El conjunto de claves lo pina ahora el CABLE, donde el test
    /// del §311 lo deriva serializando (5 / 8 / 20). Lo que el tipo NO puede
    /// decir de si mismo —y es lo que este gate afirma desde §313— es que
    /// **nadie vuelva a montar la respuesta a mano en el brazo**.
    ///
    /// CEGUERA DECLARADA, distinta de la anterior. Lee la FUENTE de este mismo
    /// fichero y delimita el brazo por dos marcas al PRINCIPIO de linea. NO
    /// ve: un `json!` construido en una funcion auxiliar llamada desde el
    /// brazo, ni una clave insertada despues con `v["x"] = ...`.
    ///
    /// Y lleva los dos endurecimientos que el gate anterior no tenia, que son
    /// la misma figura dos veces. **Un cero solo vale si antes se demuestra
    /// que se miro donde habia algo**: por eso las dos marcas se exigen UNA
    /// sola vez y la region NO VACIA *antes* de contar nada — si la
    /// delimitacion se desplaza y acaba mirando una region vacia, el gate se
    /// pondria verde por nada (caso 24 y 67 de la familia). Y **prohibir sin
    /// exigir dejaria verde un brazo borrado**: por eso junto al cero `json!`
    /// va el positivo de las TRES construcciones por el tipo.
    #[test]
    fn el_brazo_no_monta_json_a_mano_y_lo_monta_el_tipo() {
        // ⚠️ §315 · EL CIERRE CAMBIA PORQUE EL VECINO CAMBIO: los dos
        // brazos de la cofirma entraron entre este y `inclusionReceipt`, y
        // el gate lo CAZO — que es exactamente para lo que esta. Los brazos
        // nuevos SI montan `json!` a mano y DEBEN: sus respuestas son formas
        // ad-hoc de dos y tres claves, no el DTO. Lo que este gate vigila es
        // que la CABEZA FIRMADA no vuelva a montarse a pelo, y eso sigue.
        const APERTURA: &str = "\"zkssl_signedEpochHead\" =>";
        const CIERRE: &str = "\"zkssl_submitCosig\" =>";

        let fuente = include_str!("main.rs");
        let lineas: Vec<&str> = fuente.lines().map(|l| l.trim_start()).collect();
        let aperturas: Vec<usize> = lineas
            .iter()
            .enumerate()
            .filter(|(_, t)| t.starts_with(APERTURA))
            .map(|(i, _)| i)
            .collect();
        let cierres: Vec<usize> = lineas
            .iter()
            .enumerate()
            .filter(|(_, t)| t.starts_with(CIERRE))
            .map(|(i, _)| i)
            .collect();

        // ── 1 · PRIMERO se demuestra que la region existe ──
        assert_eq!(
            aperturas.len(),
            1,
            "la marca de APERTURA aparece {} veces: sin delimitacion, el cero de abajo no vale nada",
            aperturas.len()
        );
        assert_eq!(
            cierres.len(),
            1,
            "la marca de CIERRE aparece {} veces",
            cierres.len()
        );
        let (abre, cierra) = (aperturas[0], cierres[0]);
        assert!(
            cierra > abre + 1,
            "la region del brazo esta VACIA o invertida: de {abre} a {cierra}"
        );
        let region = &lineas[abre + 1..cierra];

        // ── 2 · y SOLO entonces se cuenta ──
        let a_mano = region.iter().filter(|t| t.contains("json!(")).count();
        assert_eq!(
            a_mano, 0,
            "el brazo vuelve a montar la respuesta a mano: {a_mano} usos de json!"
        );
        let por_el_tipo = region
            .iter()
            .filter(|t| t.contains("SignedEpochHeadDto::"))
            .count();
        assert_eq!(
            por_el_tipo, 3,
            "el brazo tiene que construir las TRES formas con el tipo; vi {por_el_tipo}"
        );
    }

    // ── consultas de solo lectura ─────────────────────────────────

    #[test]
    fn las_consultas_no_dejan_reservas() {
        let app = nodo(30);
        cuenta(&app, 50, 1_000);
        for m in ["zkssl_accountCount", "zkssl_supply", "zkssl_epochHead", "zkssl_params"] {
            dispatch(&app, m, json!({})).unwrap_or_else(|e| panic!("{m}: {}", e.message));
        }
        assert_eq!(reservas_vivas(&app), 0, "una consulta no debe reservar nada");
    }

    #[test]
    fn el_registro_arranca_vacio_y_crece_con_las_altas() {
        let app = nodo(30);
        let antes = dispatch(&app, "zkssl_accountCount", json!({})).expect("cuenta");
        assert_eq!(antes, json!("0x0"), "un nodo nuevo no tiene cuentas");
        cuenta(&app, 60, 100);
        cuenta(&app, 61, 100);
        let despues = dispatch(&app, "zkssl_accountCount", json!({})).expect("cuenta");
        assert_eq!(despues, json!("0x2"));
    }

    // ── §259 · el recibo de inclusion ─────────────────────────────
    //
    // ⚠️ AQUI VIVE EL CRUCE, y por eso los tests de §259 estan en el nodo
    // y no en la capa: es el unico crate que ve LA CAPA y EL VERIFICADOR a
    // la vez. Los once tests de `inclusion.rs` construyen su raiz CON
    // `path_root`: son autoconsistentes y no pueden probar que un camino
    // de un `SparseTree` DE VERDAD verifique.

    /// Traduce el recibo del cable a lo que el verificador entiende —
    /// exactamente lo que haria un tercero.
    fn recibo_desde_cable(v: &Value) -> zk_ssl_verify::ReciboInclusion {
        let d = |x: &Value| {
            digest_from_wire(&serde_json::from_value::<wire::B32>(x.clone()).expect("B32"))
                .expect("digest")
        };
        let q = |x: &Value| {
            u64::from_str_radix(x.as_str().expect("Q").trim_start_matches("0x"), 16).expect("hex")
        };
        let h = &v["head"];
        zk_ssl_verify::ReciboInclusion {
            indice: q(&v["index"]),
            hoja: d(&v["leaf"]),
            hermanos: v["path"]["siblings"].as_array().expect("siblings").iter().map(d).collect(),
            derecha: v["path"]["isRight"]
                .as_array()
                .expect("isRight")
                .iter()
                .map(|b| b.as_bool().expect("bool"))
                .collect(),
            seq: q(&h["seq"]),
            accounts_root: d(&h["accountsRoot"]),
            pending_root: d(&h["pendingRoot"]),
            frozen_root: d(&h["frozenRoot"]),
            chain_digest: d(&h["chainDigest"]),
        }
    }

    /// La pareja de §275, leida del MISMO `head` que el recibo.
    ///
    /// ⚠️ Aparte del recibo a proposito: lo que crecio es LA CABEZA, no
    /// el camino. `ReciboInclusion` sigue teniendo la forma que las
    /// cabezas v1 custodiadas necesitan.
    fn pareja_desde_cable(v: &Value) -> (zk_ssl_verify::acuses::Digest, u64) {
        let h = &v["head"];
        let raiz = digest_from_wire(
            &serde_json::from_value::<wire::B32>(h["acusesRoot"].clone()).expect("B32"),
        )
        .expect("digest");
        let n = u64::from_str_radix(h["n"].as_str().expect("Q").trim_start_matches("0x"), 16)
            .expect("hex");
        (raiz, n)
    }

    #[test]
    fn el_recibo_de_inclusion_verifica_contra_la_cabeza() {
        // ⚠️⚠️ EL TEST QUE JUSTIFICA TODA LA CADENA §256-§259: un camino
        // sacado de un `SparseTree` REAL, subido por el verificador
        // INDEPENDIENTE, hasta la cabeza. Si el convenio de `is_right` del
        // arbol y el de `path_root` divergieran, los once de
        // `inclusion.rs` seguirian verdes y ESTE se pondria rojo.
        let app = nodo(30);
        let (idx, _, vk) = cuenta(&app, 70, 1_000);
        let r = dispatch(&app, "zkssl_inclusionReceipt", json!({ "index": Q(idx), "viewKey": vk }))
            .expect("recibo");
        let firmado = digest_from_wire(
            &serde_json::from_value::<wire::B32>(r["head"]["epochDigest"].clone()).expect("B32"),
        )
        .expect("digest");
        // §275: la cabeza servida ya es v2. Recomponer con el v1 daria
        // CabezaDistinta aunque el camino fuese perfecto — que es
        // exactamente lo que la corrida 2 puso en rojo.
        let (acuses_root, n) = pareja_desde_cable(&r);
        assert_eq!(
            zk_ssl_verify::verificar_inclusion_v3(&recibo_desde_cable(&r), acuses_root, n, zk_ssl_verify::acuses::as_digest(0), 0, firmado),
            Ok(())
        );
    }

    #[test]
    fn el_recibo_declara_la_forma_que_el_arbol_tiene() {
        // En caliente toda cuenta nueva sale SALADA (§258): la forma no se
        // declara de memoria, se mide contra el arbol.
        let app = nodo(30);
        let (idx, _, vk) = cuenta(&app, 71, 500);
        let r = dispatch(&app, "zkssl_inclusionReceipt", json!({ "index": Q(idx), "viewKey": vk }))
            .expect("recibo");
        assert_eq!(r["leafFormat"], json!("salted"));
    }

    #[test]
    fn el_recibo_no_entrega_el_salt() {
        // ⚠️ El salt es lo UNICO que impide enumerar el saldo desde un
        // camino (§117). Si algun dia se colara en el DTO, esto lo dice.
        let app = nodo(30);
        let (idx, _, vk) = cuenta(&app, 72, 500);
        let r = dispatch(&app, "zkssl_inclusionReceipt", json!({ "index": Q(idx), "viewKey": vk }))
            .expect("recibo");
        // ⚠️ Por CLAVES, no por subcadena: el valor de `leafFormat` es
        // literalmente "salted", asi que buscar "salt" en el texto entero
        // mediria la respuesta y no la estructura.
        let claves: Vec<String> = r
            .as_object()
            .expect("el recibo es un objeto")
            .keys()
            .map(|k| k.to_lowercase())
            .collect();
        assert!(
            claves.iter().all(|k| !k.contains("salt")),
            "el recibo no debe llevar el salt: {claves:?}"
        );
    }

    #[test]
    fn un_recibo_de_otra_epoca_no_verifica() {
        // ⚠️ EL TERCER ESLABON, contra un arbol de verdad: el camino era
        // correcto CUANDO SE SIRVIO, y deja de valer en cuanto la raiz se
        // mueve. Sin la comprobacion de cabeza, un operador serviria un
        // recibo de una epoca en la que la hoja SI estaba.
        let app = nodo(30);
        let (idx, _, vk) = cuenta(&app, 73, 500);
        let r = dispatch(&app, "zkssl_inclusionReceipt", json!({ "index": Q(idx), "viewKey": vk }))
            .expect("recibo");
        cuenta(&app, 74, 500); // la raiz se mueve
        let nueva = dispatch(&app, "zkssl_epochHead", json!({})).expect("cabeza");
        let firmado = digest_from_wire(
            &serde_json::from_value::<wire::B32>(nueva["epochDigest"].clone()).expect("B32"),
        )
        .expect("digest");
        // ⚠️ §275 · PRIMERO contra SU cabeza, y esto es una CORRECCION.
        // En la corrida 2 este negativo siguio VERDE con el recompositor
        // equivocado: media la VERSION, no la epoca, y habria pasado
        // igual con un recibo de la epoca correcta. Un test que cuadra y
        // miente (§266). El positivo de al lado es lo que le devuelve el
        // sentido: si el recibo no vale contra su propia cabeza, el
        // negativo no prueba nada.
        let (acuses_root, n) = pareja_desde_cable(&r);
        let suya = digest_from_wire(
            &serde_json::from_value::<wire::B32>(r["head"]["epochDigest"].clone()).expect("B32"),
        )
        .expect("digest");
        assert_eq!(
            zk_ssl_verify::verificar_inclusion_v3(&recibo_desde_cable(&r), acuses_root, n, zk_ssl_verify::acuses::as_digest(0), 0, suya),
            Ok(()),
            "el recibo debe valer contra la cabeza de SU epoca"
        );
        assert_eq!(
            zk_ssl_verify::verificar_inclusion_v3(&recibo_desde_cable(&r), acuses_root, n, zk_ssl_verify::acuses::as_digest(0), 0, firmado),
            Err(zk_ssl_verify::InclusionError::CabezaDistinta)
        );
    }

    // ── §293 · la prueba de extension, servida ─────────────────

    #[test]
    fn la_prueba_de_extension_se_sirve_y_verifica() {
        // El eslabon 2 como servicio: lo que el cable da, el objeto de
        // §291 lo juzga — las MISMAS funciones que usaria un tercero.
        let app = nodo(30);
        for _ in 0..3 {
            let l = crate::latido::latir(&app, None).expect("latir");
            crate::latido::conservar(&app, l);
        }
        let (vieja, nueva, t) = {
            let h = app.hojas_mmr.lock().expect("hojas");
            (
                zk_ssl_verify::mmr::cima(&h[..1]).expect("cima vieja"),
                zk_ssl_verify::mmr::cima(&h).expect("cima nueva"),
                h.len() as u64,
            )
        };
        let r = dispatch(&app, "zkssl_consistencyProof", json!({ "oldSize": Q(1u64) }))
            .expect("consistencyProof");
        assert_eq!(r["available"], json!(true));
        assert_eq!(r["mmrSize"], serde_json::to_value(Q(t)).unwrap());
        let camino: Vec<_> = r["camino"]
            .as_array()
            .expect("camino")
            .iter()
            .map(|s| {
                digest_from_wire(&serde_json::from_value::<wire::B32>(s.clone()).expect("B32"))
                    .expect("digest")
            })
            .collect();
        assert!(
            zk_ssl_verify::mmr::verificar_consistencia(vieja, 1, nueva, t, &camino),
            "el camino servido debe probar la extension"
        );
    }

    #[test]
    fn un_acumulador_que_va_por_detras_lo_dice() {
        // §292 prometio el reseteo VISIBLE; esto es la promesa en el cable.
        let app = nodo(30);
        let l = crate::latido::latir(&app, None).expect("latir");
        crate::latido::conservar(&app, l);
        let r = dispatch(&app, "zkssl_consistencyProof", json!({ "oldSize": Q(9u64) }))
            .expect("consistencyProof");
        assert_eq!(r["available"], json!(false));
        assert!(r["reason"].as_str().expect("reason").contains("POR DETRAS"));
        let r0 = dispatch(&app, "zkssl_consistencyProof", json!({ "oldSize": Q(0u64) }))
            .expect("consistencyProof oldSize 0");
        assert_eq!(r0["available"], json!(false));
    }

    /// Una clave de vista que no es de nadie, para los negativos.
    ///
    /// ⚠️ Devuelve `B32`, no `Value`: `digest_to_wire` da `B32` y `json!`
    /// ya lo serializa. Declararlo `Value` costo una corrida.
    fn vk_falsa() -> wire::B32 {
        digest_to_wire(&sal(9_999))
    }

    // ── §261 · la credencial para los caminos ─────────────────────

    #[test]
    fn la_clave_de_otra_cuenta_no_abre_el_camino() {
        // ⚠️⚠️ EL UNICO NEGATIVO QUE PRUEBA EL SELLO. Ausente y
        // malformada las caza cualquier cosa; esta distingue si el nodo
        // comprueba la credencial CONTRA EL INDICE PEDIDO o solo que sea
        // una clave bien formada. Si `account_view_authenticated`
        // comprobara lo segundo, todo lo demas seguiria verde.
        let app = nodo(30);
        let (a, _, _vk_a) = cuenta(&app, 80, 1_000);
        let (_b, _, vk_b) = cuenta(&app, 81, 1_000);

        for (m, params) in [
            ("zkssl_inclusionReceipt", json!({ "index": Q(a), "viewKey": vk_b })),
            ("zkssl_sendMaterials", json!({
                "sender": Q(a), "viewKey": vk_b, "receiverId": digest_to_wire(&sal(1)),
                "amount": Q(1u64), "salt": digest_to_wire(&sal(2)) })),
            ("zkssl_claimMaterials", json!({
                "receiver": Q(a), "viewKey": vk_b,
                "notice": json!({ "position": Q(0u64), "salt": digest_to_wire(&sal(3)),
                                  "amount": Q(1u64) }) })),
        ] {
            let e = dispatch(&app, m, params).expect_err("una clave ajena no puede abrir");
            assert_eq!(e.code, -32004, "{m} acepto la clave de OTRA cuenta");
        }
    }

    #[test]
    fn sin_credencial_no_hay_camino() {
        // Falta el campo: el despacho ni siquiera llega a la capa.
        let app = nodo(30);
        let (a, _, _) = cuenta(&app, 82, 1_000);
        let e = dispatch(&app, "zkssl_inclusionReceipt", json!({ "index": Q(a) }))
            .expect_err("sin viewKey no hay recibo");
        assert_eq!(e.code, -32602, "deberia ser parametros invalidos");
    }

    #[test]
    fn una_credencial_malformada_se_rechaza_sin_reventar() {
        let app = nodo(30);
        let (a, _, _) = cuenta(&app, 83, 1_000);
        let e = dispatch(&app, "zkssl_inclusionReceipt",
                         json!({ "index": Q(a), "viewKey": "0xno-es-hex" }))
            .expect_err("basura no puede pasar");
        assert_ne!(e.code, -32601, "el metodo existe");
    }

    #[test]
    fn con_su_propia_clave_el_titular_sigue_pudiendo() {
        // ⚠️ Que el sello CIERRA no basta: hay que probar que NO cerro de
        // mas. Un control que rechaza todo tambien pasaria los negativos.
        let app = nodo(30);
        let (a, _, vk_a) = cuenta(&app, 84, 1_000);
        let r = dispatch(&app, "zkssl_inclusionReceipt",
                         json!({ "index": Q(a), "viewKey": vk_a.clone() }))
            .expect("con su clave, si");
        assert_eq!(r["index"], json!(Q(a)));
        let m = dispatch(&app, "zkssl_sendMaterials", json!({
            "sender": Q(a), "viewKey": vk_a, "receiverId": digest_to_wire(&sal(1)),
            "amount": Q(10u64), "salt": digest_to_wire(&sal(2)) }))
            .expect("materiales con su clave");
        assert!(m["senderPath"].is_object(), "el camino sigue llegando");
    }

    #[test]
    fn el_camino_del_receptor_ya_no_lo_da_un_aviso_inventado() {
        // ⚠️ §259 AFIRMO QUE ESTA PUERTA EXIGIA EL AVISO DEL PAGADOR. No
        // lo exigia: `claim_materials` solo usa `notice.position`. Este
        // test fija que ahora si hace falta ser el receptor.
        let app = nodo(30);
        let (a, _, _) = cuenta(&app, 85, 1_000);
        let inventado = json!({ "position": Q(0u64), "salt": digest_to_wire(&sal(7)),
                                "amount": Q(1u64) });
        let e = dispatch(&app, "zkssl_claimMaterials",
                         json!({ "receiver": Q(a), "notice": inventado, "viewKey": vk_falsa() }))
            .expect_err("un aviso inventado ya no basta");
        assert_eq!(e.code, -32004);
    }

    #[test]
    fn una_cuenta_que_no_existe_no_da_recibo() {
        let app = nodo(30);
        let e = dispatch(&app, "zkssl_inclusionReceipt", json!({ "index": Q(9_999), "viewKey": vk_falsa() }))
            .expect_err("no deberia haber recibo");
        assert_ne!(e.code, -32601, "el metodo existe: no puede ser MethodNotFound");
    }

    // ── §285 / nota 80, segunda mitad: quien firma, anota ──

    #[test]
    fn quien_firma_anota_firmar_sin_diario_se_rechaza() {
        // El molde de --custodia fichero. El cableado real es el bail de
        // main(); esto prueba la DECISION, que es lo que este binario
        // puede probar en frio.
        assert!(firma_sin_diario(true, false), "clave sin diario tiene que rechazarse");
    }

    #[test]
    fn quien_firma_anota_las_otras_tres_combinaciones_arrancan() {
        // Sin clave, el diario sigue siendo opcional: anota limites de
        // epoca (§272) sin que nadie firme nada que haya que recordar.
        assert!(!firma_sin_diario(true, true));
        assert!(!firma_sin_diario(false, true));
        assert!(!firma_sin_diario(false, false));
    }
}

#[cfg(test)]
mod tests_ack_path {
    // §275: seguro A CIEGAS — no fija cual de las dos razones sale
    // (sin --diario, o epoca abierta con diario vacio): en ambas,
    // available == false y la respuesta DICE por que (forma de §241).
    #[test]
    fn ack_path_sin_epoca_cerrada_dice_por_que() {
        let app = crate::tests::nodo(30);
        let v = crate::dispatch(&app, "zkssl_ackPath", serde_json::json!({ "seq": "0x0" }))
            .expect("ackPath");
        assert_eq!(v["available"], false, "sin epoca cerrada no puede haber camino");
        assert!(v["reason"].as_str().map(|s| !s.is_empty()).unwrap_or(false),
            "la respuesta debe DECIR por que: {v}");
    }
}
