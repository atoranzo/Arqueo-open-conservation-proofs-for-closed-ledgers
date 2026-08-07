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
use zk_ssl::{LayerError, SovereignLayer};
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
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    init_tracing(&args.log);

    let layer = open_layer(&args)?;
    let app = std::sync::Arc::new(App {
        estado: Mutex::new(Estado { layer, reservas: BTreeMap::new() }),
        dev: args.dev,
        reserva_ttl: Duration::from_secs(args.reserva_ttl),
    });

    if args.dev {
        tracing::warn!("modo --dev: dev_* habilitado con custodios de PRUEBA; no usar en producción");
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

struct RpcError {
    code: i64,
    message: String,
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
    fn wire(e: wire::WireError) -> Self {
        Self::invalid_params(e)
    }
}

fn parse<T: serde::de::DeserializeOwned>(params: Value) -> Result<T, RpcError> {
    serde_json::from_value(params).map_err(RpcError::invalid_params)
}

fn dispatch(app: &App, method: &str, params: Value) -> Result<Value, RpcError> {
    // ⚠️ Las operaciones bloquean el Mutex mientras verifican pruebas
    // (~decenas de ms). Correcto para un nodo único; con carga real, el
    // paso siguiente es una cola de escritura (ver ROADMAP).
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
            Ok(serde_json::to_value(wire::EpochHeadDto::from(&l.epoch_head())).unwrap())
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
            struct P { sender: Q, receiver_id: wire::B32, amount: Q, salt: wire::B32 }
            let p: P = parse(params)?;
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
            let r = l.apply_send(&receipt, p.sender.0, &state, p.amount.0);
            let pos = receipt.notice.position;
            reservas.remove(&pos);
            if let Err(e) = r {
                l.release_pending(pos);
                return Err(RpcError::layer(e));
            }
            Ok(applied(l))
        }

        "zkssl_claimMaterials" => {
            #[derive(Deserialize)]
            struct P { receiver: Q, notice: wire::PendingNoticeDto }
            let p: P = parse(params)?;
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
            l.apply_claim(&receipt, p.receiver.0, &state, &notice)
                .map_err(RpcError::layer)?;
            Ok(applied(l))
        }

        // ── dev_* : SOLO con --dev (custodios de prueba) ───────────
        m if m.starts_with("dev_") => dispatch_dev(app, l, m, params),

        other => Err(RpcError::method_not_found(other)),
    }
}

/// Respuesta común de las escrituras: qué entrada quedó en el registro.
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
