//! Los tres subcomandos. Toda la salida de usuario pasa por el `Tracer`,
//! así `--json` produce JSON Lines puro sin tocar la lógica.

use clap::Args;
use zk_ssl::log::OpKind;
use zk_ssl::SovereignLayer;

use crate::fmt::hex_short;
use crate::sandbox::{self, Params};
use crate::trace::{TraceEvent, Tracer};

/// Parámetros de la capa, compartidos por los tres subcomandos.
/// Por defecto, los de la suite del proyecto.
#[derive(Args, Clone, Copy)]
pub struct LayerParams {
    /// Límite regulatorio por operación.
    #[arg(long, default_value_t = Params::default().regulatory_limit)]
    limit: u64,
    /// Tope de suministro.
    #[arg(long, default_value_t = Params::default().max_supply)]
    max_supply: u64,
    /// Máximo de cuentas.
    #[arg(long, default_value_t = Params::default().max_accounts)]
    max_accounts: u64,
}

impl From<LayerParams> for Params {
    fn from(p: LayerParams) -> Self {
        Params {
            regulatory_limit: p.limit,
            max_supply: p.max_supply,
            max_accounts: p.max_accounts,
        }
    }
}

// ───────────────────────────── simulate ─────────────────────────────

#[derive(Args)]
pub struct SimulateArgs {
    /// Ruta de un ledger persistido (sled). Sin ella, todo va en memoria.
    #[arg(long)]
    ledger: Option<String>,

    /// Índice de la cuenta pagadora.
    #[arg(long, default_value_t = 0)]
    from: u64,
    /// Índice de la cuenta receptora.
    #[arg(long, default_value_t = 1)]
    to: u64,
    /// Importe del envío.
    #[arg(long, default_value_t = 250_000)]
    amount: u64,

    /// Cuentas mínimas del escenario (se abren y fondean las que falten).
    #[arg(long, default_value_t = 2)]
    accounts: u64,
    /// Fondos iniciales por cuenta nueva (emisión delegada, dos custodios).
    #[arg(long, default_value_t = 1_000_000)]
    fund: u64,

    /// Deja el envío SIN cobrar, para ver el pendiente en tránsito.
    #[arg(long)]
    no_claim: bool,

    /// Semilla base de las claves deterministas del sandbox.
    #[arg(long, default_value_t = 0xA11CE)]
    key_seed: u64,
    /// Semilla del aleatorio (salt) del pendiente.
    #[arg(long, default_value_t = 7)]
    salt_seed: u64,

    #[command(flatten)]
    params: LayerParams,
}

pub fn simulate(a: SimulateArgs, tr: &mut dyn Tracer) -> anyhow::Result<()> {
    let mut layer = sandbox::open_layer(a.ledger.as_deref(), a.params.into())?;

    tr.emit(&TraceEvent::Note {
        text: format!(
            "⚠️ sandbox con claves DETERMINISTAS (semilla {:#x}); pruebas STARK reales, tarda unos segundos",
            a.key_seed
        ),
    });

    // Montaje: abre y fondea las cuentas que falten.
    // El arbol de cuentas es DISPERSO: open_account_wide devuelve la
    // POSICION DERIVADA de la hoja, no un contador. Guardamos el mapeo
    // logico->real de las cuentas abiertas en ESTA corrida; --from/--to
    // son posiciones logicas (0, 1, ...) sobre ese mapeo, o un indice
    // real ya existente (caso --ledger).
    let needed = a.accounts.max(a.from.max(a.to) + 1);
    let mut abiertas: Vec<u64> = Vec::new();
    while (layer.account_count() as u64) < needed {
        let i = abiertas.len() as u64;
        let idx =
            sandbox::open_funded(&mut layer, sandbox::key_of(a.key_seed, i), a.fund, tr)?;
        abiertas.push(idx);
    }
    let from_real = match abiertas.get(a.from as usize) {
        Some(&ix) => ix,
        None if layer.public_id_of(a.from).is_some() => a.from,
        None => anyhow::bail!(
            "la cuenta logica #{} no se abrio en esta corrida y tampoco es un \
             indice real del arbol (disperso: posiciones derivadas)",
            a.from
        ),
    };
    let to_real = match abiertas.get(a.to as usize) {
        Some(&ix) => ix,
        None if layer.public_id_of(a.to).is_some() => a.to,
        None => anyhow::bail!(
            "la cuenta logica #{} no se abrio en esta corrida y tampoco es un \
             indice real del arbol (disperso: posiciones derivadas)",
            a.to
        ),
    };

    // FASE 1 — enviar.
    let envio = sandbox::run_send(
        &mut layer,
        from_real,
        sandbox::key_of(a.key_seed, a.from),
        to_real,
        a.amount,
        a.salt_seed,
        tr,
    )?;

    // FASE 2 — cobrar (salvo que se pida ver el pendiente en tránsito).
    if a.no_claim {
        tr.emit(&TraceEvent::Note {
            text: "envío SIN cobrar: el importe queda inmovilizado hasta claim o refund (§29/§30)"
                .into(),
        });
    } else {
        sandbox::run_claim(
            &mut layer,
            to_real,
            sandbox::key_of(a.key_seed, a.to),
            &envio.notice,
            tr,
        )?;
    }

    verify_chain(&layer, tr);
    for idx in [from_real, to_real] {
        if let Some(balance) = layer.balance_of(idx) {
            tr.emit(&TraceEvent::Note {
                text: format!("saldo final #{idx}: {balance} (vista del operador)"),
            });
        }
    }
    sandbox::emit_summary(&layer, tr);
    Ok(())
}

// ───────────────────────────── trace-tx ─────────────────────────────

#[derive(Args)]
pub struct TraceTxArgs {
    /// Ruta de un ledger persistido. Sin ella se genera un escenario de
    /// demostración en memoria y se traza SU registro.
    #[arg(long)]
    ledger: Option<String>,

    /// Número de secuencia de la operación a detallar.
    #[arg(long)]
    seq: Option<u64>,

    /// Sin --seq: cuántas entradas recientes mostrar.
    #[arg(long, default_value_t = 10)]
    last: usize,

    #[command(flatten)]
    params: LayerParams,
}

pub fn trace_tx(a: TraceTxArgs, tr: &mut dyn Tracer) -> anyhow::Result<()> {
    let layer = layer_or_demo(a.ledger.as_deref(), a.params.into(), tr)?;
    let entries = layer.transition_log().entries();

    match a.seq {
        Some(seq) => {
            let e = entries
                .iter()
                .find(|e| e.seq == seq)
                .ok_or_else(|| anyhow::anyhow!("no hay entrada con seq {seq}"))?;
            emit_entry(e, tr);
        }
        None => {
            let skip = entries.len().saturating_sub(a.last);
            for e in &entries[skip..] {
                emit_entry(e, tr);
            }
        }
    }

    verify_chain(&layer, tr);
    Ok(())
}

fn emit_entry(e: &zk_ssl::log::LogEntry, tr: &mut dyn Tracer) {
    tr.emit(&TraceEvent::LogEntry {
        seq: e.seq,
        op: format!("{:?}", e.kind),
        circuit_hint: circuit_hint(e.kind).into(),
        root_old: hex_short(&e.root_old),
        root_new: hex_short(&e.root_new),
        proof_digest: hex_short(&e.proof_digest),
        chain: hex_short(&e.chain),
    });
}

/// Qué verifica cada entrada. Orientativo: la fuente de verdad es
/// `OpKind` (zk-ssl/src/log.rs) y los circuitos de `stark-experiment`.
fn circuit_hint(k: OpKind) -> &'static str {
    match k {
        OpKind::OpenAccount => "sin prueba: apertura a CERO",
        OpKind::Mint => "mint_climb + 2× threshold_single_nullifier",
        OpKind::Transfer => "transferencia (vía histórica)",
        OpKind::Burn => "circuit_burn",
        OpKind::Recovery => "recovery_climb",
        OpKind::Governance => "circuit_governance",
        OpKind::Freeze => "frozen_climb",
        OpKind::Send => "circuit_send (fase 1)",
        OpKind::Claim => "circuit_claim (fase 2)",
        OpKind::MintToPending => "mint_pending_climb",
        OpKind::Migration => "sin prueba: compromiso replicable",
        OpKind::Refund => "reembolso de pendiente caducado",
        OpKind::Consumo => "sin prueba: consumo publicado (RFC-0006 E1)",
    }
}

// ─────────────────────────── inspect-state ──────────────────────────

#[derive(Args)]
pub struct InspectStateArgs {
    /// Ruta de un ledger persistido. Sin ella se inspecciona un escenario
    /// de demostración en memoria.
    #[arg(long)]
    ledger: Option<String>,

    /// Lista además las cuentas (índice, id público, saldo, nonce).
    #[arg(long)]
    accounts: bool,

    #[command(flatten)]
    params: LayerParams,
}

pub fn inspect_state(a: InspectStateArgs, tr: &mut dyn Tracer) -> anyhow::Result<()> {
    let layer = layer_or_demo(a.ledger.as_deref(), a.params.into(), tr)?;

    sandbox::emit_summary(&layer, tr);

    if a.accounts {
        tr.emit(&TraceEvent::Note {
            text: "⚠️ saldos y nonces: vista del OPERADOR por diseño (§129); el titular usa account_view_authenticated".into(),
        });
        for idx in 0..layer.account_count() as u64 {
            if let (Some(id), Some(balance), Some(nonce)) = (
                layer.public_id_of(idx),
                layer.balance_of(idx),
                layer.nonce_of(idx),
            ) {
                tr.emit(&TraceEvent::AccountRow {
                    index: idx,
                    public_id: hex_short(&id),
                    balance,
                    nonce: format!("{nonce}"),
                });
            }
        }
    }

    verify_chain(&layer, tr);
    Ok(())
}

// ───────────────────────────── comunes ──────────────────────────────

/// Abre el ledger indicado o, sin él, monta el escenario de demostración:
/// dos cuentas fondeadas y una transferencia completa en dos fases.
fn layer_or_demo(
    ledger: Option<&str>,
    p: Params,
    tr: &mut dyn Tracer,
) -> anyhow::Result<SovereignLayer> {
    if let Some(path) = ledger {
        return sandbox::open_layer(Some(path), p);
    }
    tr.emit(&TraceEvent::Note {
        text: "sin --ledger: generando escenario de demostración en memoria…".into(),
    });
    let mut layer = sandbox::open_layer(None, p)?;
    let seed = 0xA11CE;
    let a0 = sandbox::open_funded(&mut layer, sandbox::key_of(seed, 0), 1_000_000, tr)?;
    let a1 = sandbox::open_funded(&mut layer, sandbox::key_of(seed, 1), 1_000_000, tr)?;
    let envio = sandbox::run_send(
        &mut layer,
        a0,
        sandbox::key_of(seed, 0),
        a1,
        250_000,
        7,
        tr,
    )?;
    sandbox::run_claim(&mut layer, a1, sandbox::key_of(seed, 1), &envio.notice, tr)?;
    Ok(layer)
}

fn verify_chain(layer: &SovereignLayer, tr: &mut dyn Tracer) {
    let log = layer.transition_log();
    match log.verify_chain() {
        Ok(()) => tr.emit(&TraceEvent::ChainVerified {
            ok: true,
            entries: log.len(),
            error: None,
        }),
        Err(e) => tr.emit(&TraceEvent::ChainVerified {
            ok: false,
            entries: log.len(),
            error: Some(format!("{e:?}")),
        }),
    }
}
