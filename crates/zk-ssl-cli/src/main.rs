//! # zk-ssl-cli — sandbox y trazador de la capa desde la terminal
//!
//! Tres subcomandos sobre la capa REAL (`zk_ssl::SovereignLayer`):
//!
//! - `simulate`      — pago en dos fases (send + claim) con pruebas STARK
//!                     reales, en memoria o contra un ledger persistido.
//! - `trace-tx`      — paso a paso de una operación según el
//!                     `TransitionLog` encadenado de la capa.
//! - `inspect-state` — raíces, suministro, cuentas y cabeza del registro.
//!
//! Convención de salida: **datos por stdout, diagnóstico por stderr**.
//! Con `--json`, stdout es JSON Lines puro (un evento por línea).

mod commands;
mod fmt;
mod sandbox;
mod trace;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "zk-ssl-cli",
    version,
    about = "Sandbox, trazas e inspección de la ZK-Sovereign Settlement Layer"
)]
struct Cli {
    /// Filtro de `tracing` (error|warn|info|debug|trace o directivas EnvFilter).
    /// Con `debug` se ven los spans y tiempos internos de cada fase.
    #[arg(long, global = true, default_value = "info")]
    log: String,

    /// Desactiva colores ANSI.
    #[arg(long, global = true)]
    no_color: bool,

    /// Emite los eventos de traza como JSON Lines por stdout.
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Ejecuta un envío en dos fases (y su cobro) con pruebas STARK reales.
    Simulate(commands::SimulateArgs),
    /// Muestra el paso a paso registrado de operaciones del TransitionLog.
    TraceTx(commands::TraceTxArgs),
    /// Inspecciona el estado del libro mayor.
    InspectState(commands::InspectStateArgs),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if cli.no_color {
        colored::control::set_override(false);
    }
    init_tracing(&cli.log, !cli.no_color);

    // El sumidero de eventos es intercambiable: consola coloreada o JSONL.
    let mut tracer = trace::make_tracer(cli.json);

    match cli.command {
        Command::Simulate(a) => commands::simulate(a, tracer.as_mut()),
        Command::TraceTx(a) => commands::trace_tx(a, tracer.as_mut()),
        Command::InspectState(a) => commands::inspect_state(a, tracer.as_mut()),
    }
}

/// Diagnóstico técnico por **stderr**, para no contaminar la salida de
/// datos (crítico con `--json`).
fn init_tracing(filter: &str, ansi: bool) {
    use tracing_subscriber::EnvFilter;

    let filter =
        EnvFilter::try_new(filter).unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .with_ansi(ansi)
        .with_target(false)
        .compact()
        .init();
}
