//! # zk-ssl-cli — sandbox y trazador de la capa desde la terminal
//!
//! Ocho subcomandos sobre la capa REAL (`zk_ssl::SovereignLayer`) y el nodo:
//!
//! - `simulate`      — pago en dos fases (send + claim) con pruebas STARK
//!                     reales, en memoria o contra un ledger persistido.
//! - `trace-tx`      — paso a paso de una operación según el
//!                     `TransitionLog` encadenado de la capa.
//! - `inspect-state` — raíces, suministro, cuentas y cabeza del registro.
//! - `conformance`   — los vectores de conformidad (--emit / --check).
//! - `witness`       — el TESTIGO de las cabezas firmadas de un nodo (§245).
//! - `prueba-cobro`  — la BOCA del cobrador: el sobre `cobro_pendiente` (RFC-0008, S497).
//! - `prueba-pago`   — la BOCA del pagador: el sobre `pago_en_curso` (RFC-0008, S507).
//! - `prueba-prenda` — la BOCA del prendador: el sobre `prenda` (RFC-0008, S543).
//!
//! Convención de salida: **datos por stdout, diagnóstico por stderr**.
//! Con `--json`, stdout es JSON Lines puro (un evento por línea).

mod cobro;
mod commands;
mod conformance;
mod fmt;
#[cfg(test)]
mod nucleo_kat;
mod pago;
mod prenda;
mod sandbox;
mod trace;
mod witness;

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
    /// Vectores de conformidad: --emit los fija, --check los reproduce.
    Conformance(conformance::ConformanceArgs),
    /// **El TESTIGO** (§245): consulta las cabezas firmadas de un nodo,
    /// las verifica, y **fija la clave que ve la primera vez**.
    ///
    /// ⚠️ Un testigo que opera el propio operador **no prueba nada**: esto
    /// es la implementación de referencia de lo que correría un TERCERO.
    Witness(witness::WitnessArgs),
    /// **La BOCA del cobrador** (RFC-0008 E4, S497): con su aviso v2 y su credencial pide la
    /// cabeza firmada y la foto de su pendiente a un nodo VIVO, y escribe el sobre
    /// `cobro_pendiente` de `PAQUETE.md` 2.8 con la cabeza verbatim.
    PruebaCobro(cobro::PruebaCobroArgs),
    /// **La BOCA del pagador** (RFC-0008 E2, S507): con su aviso v2, su RETORNO y su
    /// credencial pide la cabeza firmada y la foto de su pendiente a un nodo VIVO -con
    /// `receiverId`, S505-, y escribe el sobre `pago_en_curso` de `PAQUETE.md` 2.9.
    PruebaPago(pago::PruebaPagoArgs),
    /// **La BOCA del prendador** (RFC-0008 E3, S543): con su aviso v2 y su keystore pide la
    /// cabeza firmada y la foto de su pendiente a un nodo VIVO, escribe el sobre `prenda` de
    /// `PAQUETE.md` 2.10 y, con `--publicar`, pide a `zkssl_pledge` que escriba la marca.
    PruebaPrenda(prenda::PruebaPrendaArgs),
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
        Command::Conformance(a) => conformance::conformance(a, tracer.as_mut()),
        Command::Witness(a) => witness::run(a),
        Command::PruebaCobro(a) => cobro::run(a),
        Command::PruebaPago(a) => pago::run(a),
        Command::PruebaPrenda(a) => prenda::run(a),
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
