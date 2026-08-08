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
    fn nodo(ttl_segundos: u64) -> App {
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
        }
    }

    /// Abre una cuenta con semilla `s` y la fondea. Devuelve (indice, id).
    fn cuenta(app: &App, s: u64, saldo: u64) -> (u64, Value) {
        let r = dispatch(app, "dev_openSeeded", json!({ "seed": Q(s) })).expect("abrir");
        let idx = r["index"].as_str().expect("index").to_string();
        let idx = u64::from_str_radix(idx.trim_start_matches("0x"), 16).expect("hex");
        dispatch(app, "dev_fund", json!({ "index": Q(idx), "amount": Q(saldo) })).expect("fondear");
        (idx, r["publicId"].clone())
    }

    fn reservas_vivas(app: &App) -> usize {
        app.estado.lock().expect("mutex").reservas.len()
    }

    /// Un digest a partir de un u64, para las sales de los tests.
    fn sal(n: u64) -> [winterfell::math::fields::f64::BaseElement; 4] {
        use winterfell::math::fields::f64::BaseElement as E;
        [E::new(n), E::new(0), E::new(0), E::new(0)]
    }

    fn materiales(app: &App, emisor: u64, receptor: &Value, importe: u64, salt: u64) -> Value {
        dispatch(
            app,
            "zkssl_sendMaterials",
            json!({
                "sender": Q(emisor),
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
        let (a, _) = cuenta(&app, 1, 100_000);
        let (b, _) = cuenta(&app, 2, 100_000);
        let (_, id_c) = cuenta(&app, 3, 0);

        let m1 = materiales(&app, a, &id_c, 1_000, 7);
        let m2 = materiales(&app, b, &id_c, 1_000, 8);

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
        let (a, _) = cuenta(&app, 10, 1_000_000);
        let (_, id) = cuenta(&app, 11, 0);
        let mut vistas = std::collections::BTreeSet::new();
        for i in 0..15u64 {
            let m = materiales(&app, a, &id, 1_000, 100 + i);
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
        let (pobre, _) = cuenta(&app, 20, 10);
        let (_, id) = cuenta(&app, 21, 0);

        let e = dispatch(
            &app,
            "zkssl_sendMaterials",
            json!({
                "sender": Q(pobre),
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
        let (a, _) = cuenta(&app, 30, 100_000);
        let (_, id) = cuenta(&app, 31, 0);

        materiales(&app, a, &id, 1_000, 42);
        assert_eq!(reservas_vivas(&app), 1, "la reserva existe nada mas crearse");

        std::thread::sleep(Duration::from_millis(20));
        dispatch(&app, "zkssl_accountCount", json!({})).expect("una peticion cualquiera");
        assert_eq!(reservas_vivas(&app), 0, "el barrido no libero la reserva caducada");
    }

    #[test]
    fn con_caducidad_larga_el_barrido_no_toca_nada() {
        let app = nodo(3_600);
        let (a, _) = cuenta(&app, 40, 100_000);
        let (_, id) = cuenta(&app, 41, 0);
        materiales(&app, a, &id, 1_000, 43);
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
}
