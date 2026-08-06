//! Motor de traza: eventos estructurados y sumideros intercambiables.
//!
//! El diseño separa **qué pasó** (`TraceEvent`, un enum serializable) de
//! **cómo se muestra** (`Tracer`, un trait con dos implementaciones:
//! consola coloreada y JSON Lines). Es el equivalente casero del
//! `Inspector` de REVM, pero para las fases reales de ESTA capa:
//! materiales → prueba local → apply, más las autorizaciones por umbral
//! con su nullifier de custodio.

use colored::Colorize;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Fund,
    Send,
    Claim,
}

impl std::fmt::Display for Phase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Phase::Fund => "FUND",
            Phase::Send => "SEND",
            Phase::Claim => "CLAIM",
        };
        f.write_str(s)
    }
}

/// Un hecho observado durante la ejecución. Los digests van ya en hex
/// para que el evento sea serializable sin depender de winterfell.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum TraceEvent {
    PhaseStarted {
        phase: Phase,
        detail: String,
    },
    MaterialsBuilt {
        phase: Phase,
        /// Posición reservada en el árbol de pendientes (solo en SEND).
        pending_position: Option<u64>,
        ms: u128,
    },
    /// Autorización de umbral: el nullifier que ese custodio consume para
    /// ESTA operación. Es el anti-replay de `circuit_threshold_single_nullifier`.
    CustodianAuth {
        custodian_index: usize,
        nullifier: String,
        operation: String,
    },
    ProofGenerated {
        phase: Phase,
        bytes: usize,
        proof_digest: String,
        ms: u128,
    },
    /// La capa verificó y aplicó: entrada nueva en el TransitionLog.
    Applied {
        op: String,
        log_seq: u64,
        root_old: String,
        root_new: String,
        chain: String,
        ms: u128,
    },
    Rejected {
        phase: Phase,
        error: String,
    },
    /// Fila de `trace-tx`: una entrada del registro encadenado.
    LogEntry {
        seq: u64,
        op: String,
        /// Qué circuito verifica esta entrada (orientativo; la fuente de
        /// verdad es `OpKind` en zk-ssl/src/log.rs).
        circuit_hint: String,
        root_old: String,
        root_new: String,
        proof_digest: String,
        chain: String,
    },
    ChainVerified {
        ok: bool,
        entries: usize,
        error: Option<String>,
    },
    StateSummary {
        accounts: usize,
        total_supply: u64,
        total_pending: u64,
        state_root: String,
        pending_root: String,
        frozen_root: String,
        log_len: usize,
        log_head: String,
        epoch_digest: String,
    },
    /// Fila de `inspect-state --accounts` (vista del OPERADOR por diseño).
    AccountRow {
        index: u64,
        public_id: String,
        balance: u64,
        nonce: String,
    },
    Note {
        text: String,
    },
}

/// Sumidero de eventos. Aislar esto en un trait permite añadir mañana un
/// sumidero hacia fichero, hacia un colector, o hacia un nodo remoto, sin
/// tocar la lógica del sandbox.
pub trait Tracer {
    fn emit(&mut self, e: &TraceEvent);
}

pub fn make_tracer(json: bool) -> Box<dyn Tracer> {
    if json {
        Box::new(JsonlTracer)
    } else {
        Box::new(ConsoleTracer)
    }
}

/// JSON Lines por stdout: un evento por línea, apto para `jq`.
pub struct JsonlTracer;

impl Tracer for JsonlTracer {
    fn emit(&mut self, e: &TraceEvent) {
        // El evento es datos, no diagnóstico: va por stdout.
        match serde_json::to_string(e) {
            Ok(s) => println!("{s}"),
            Err(err) => tracing::error!(?err, "no se pudo serializar el evento"),
        }
    }
}

/// Consola coloreada, pensada para leerse fase a fase.
pub struct ConsoleTracer;

impl Tracer for ConsoleTracer {
    fn emit(&mut self, e: &TraceEvent) {
        match e {
            TraceEvent::PhaseStarted { phase, detail } => {
                println!(
                    "\n{} {}",
                    format!("━━ FASE {phase} ━━").bold().cyan(),
                    detail.dimmed()
                );
            }
            TraceEvent::MaterialsBuilt { pending_position, ms, .. } => {
                let pos = pending_position
                    .map(|p| format!(" (pendiente@{p})"))
                    .unwrap_or_default();
                println!("  {} materiales listos{pos} {}", "·".cyan(), fmt_ms(*ms));
            }
            TraceEvent::CustodianAuth { custodian_index, nullifier, operation } => {
                println!(
                    "  {} custodio #{custodian_index} autoriza — nullifier consumido: {}  op: {}",
                    "·".yellow(),
                    nullifier.yellow(),
                    operation.dimmed()
                );
            }
            TraceEvent::ProofGenerated { bytes, proof_digest, ms, .. } => {
                println!(
                    "  {} prueba STARK generada: {} — digest {} {}",
                    "✔".green(),
                    fmt_kb(*bytes).bold(),
                    proof_digest,
                    fmt_ms(*ms)
                );
            }
            TraceEvent::Applied { op, log_seq, root_old, root_new, chain, ms } => {
                println!(
                    "  {} aplicado → log#{log_seq} {}  raíz {} → {}  cadena {} {}",
                    "✔".green(),
                    op.bold(),
                    root_old,
                    root_new.green(),
                    chain.dimmed(),
                    fmt_ms(*ms)
                );
            }
            TraceEvent::Rejected { phase, error } => {
                println!("  {} rechazado en {phase}: {}", "✘".red().bold(), error.red());
            }
            TraceEvent::LogEntry {
                seq, op, circuit_hint, root_old, root_new, proof_digest, chain,
            } => {
                println!(
                    "{} {:<14} {}  raíz {} → {}  prueba {}  cadena {}",
                    format!("#{seq:<4}").bold(),
                    op.cyan(),
                    format!("[{circuit_hint}]").dimmed(),
                    root_old,
                    root_new,
                    proof_digest.dimmed(),
                    chain.dimmed()
                );
            }
            TraceEvent::ChainVerified { ok, entries, error } => {
                if *ok {
                    println!(
                        "\n{} cadena de transiciones íntegra ({entries} entradas)",
                        "✔".green().bold()
                    );
                } else {
                    println!(
                        "\n{} cadena ROTA: {}",
                        "✘".red().bold(),
                        error.as_deref().unwrap_or("?").red()
                    );
                }
            }
            TraceEvent::StateSummary {
                accounts, total_supply, total_pending,
                state_root, pending_root, frozen_root,
                log_len, log_head, epoch_digest,
            } => {
                println!("\n{}", "── estado del libro mayor ──".bold());
                println!("  cuentas          {accounts}");
                println!("  suministro       {total_supply}");
                println!("  en tránsito      {total_pending}   (pendientes sin cobrar)");
                println!("  raíz cuentas     {state_root}");
                println!("  raíz pendientes  {pending_root}");
                println!("  raíz congelados  {frozen_root}");
                println!("  registro         {log_len} entradas, cabeza {log_head}");
                println!("  epoch head       {epoch_digest}");
            }
            TraceEvent::AccountRow { index, public_id, balance, nonce } => {
                println!(
                    "  #{index:<4} id {}  saldo {:<12} nonce {nonce}",
                    public_id,
                    balance.to_string().bold()
                );
            }
            TraceEvent::Note { text } => println!("  {}", text.dimmed()),
        }

        // Copia estructurada para el diagnóstico técnico (`--log debug`).
        tracing::debug!(event = ?e, "trace_event");
    }
}

fn fmt_ms(ms: u128) -> colored::ColoredString {
    format!("[{ms} ms]").dimmed()
}

fn fmt_kb(bytes: usize) -> String {
    format!("{:.1} KB", bytes as f64 / 1024.0)
}
