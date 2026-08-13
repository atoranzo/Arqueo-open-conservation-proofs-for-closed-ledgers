//! Escenario sobre la capa REAL. Nada aquí es un mock: las pruebas se
//! generan con los circuitos STARK del proyecto y la capa las verifica al
//! aplicarlas.
//!
//! Reutiliza `zk_ssl::tests_support` (expuesto tras la feature `sandbox`,
//! ver PARCHES.md) para lo que exige internos de la capa: raíces de los
//! conjuntos de custodios/gobernanza y la vía de emisión delegada. Todo lo
//! demás —send/claim— va por la **vía de cliente pública**:
//! `send_materials → client::prove_send → apply_send`, sin que ninguna
//! clave llegue a la capa, que es el flujo de portada del README.

use std::time::Instant;

use zk_ssl::commitment::ClientState;
use zk_ssl::log::digest_of_proof;
use zk_ssl::tests_support as ts;
use zk_ssl::two_phase::{PendingNotice, SendReceipt};
use zk_ssl::{client, proof_options, AccountIndex, LayerError, SovereignLayer};

use crate::fmt::{hex, hex_short, Digest};
use crate::trace::{Phase, TraceEvent, Tracer};

/// Parámetros del escenario, con los mismos valores por defecto que la
/// suite del proyecto (`tests_support`): límite 500 000, tope 100 000 000,
/// 1 000 cuentas.
#[derive(Clone, Copy, Debug)]
pub struct Params {
    pub regulatory_limit: u64,
    pub max_supply: u64,
    pub max_accounts: u64,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            regulatory_limit: ts::LIMIT,
            max_supply: ts::MAX_SUPPLY,
            max_accounts: ts::MAX_ACCOUNTS,
        }
    }
}

/// Clave ancha determinista de la cuenta `i` del sandbox.
///
/// ⚠️ SOLO para pruebas: son claves derivadas de una semilla pública.
pub fn key_of(base_seed: u64, i: u64) -> Digest {
    ts::wide_key(base_seed.wrapping_add(i))
}

/// Abre la capa: en memoria, o persistida con `sled` si hay ruta.
///
/// ⚠️ Con `--ledger`, los parámetros deben coincidir con los de su
/// creación o la capa devuelve `ParameterMismatch` — es inmutabilidad,
/// no un fallo.
pub fn open_layer(ledger: Option<&str>, p: Params) -> anyhow::Result<SovereignLayer> {
    match ledger {
        Some(path) => SovereignLayer::open(
            path,
            ts::custodian_root(),
            ts::governance_root(),
            p.regulatory_limit,
            p.max_supply,
            p.max_accounts,
        )
        .map_err(|e| anyhow::anyhow!("abriendo {path}: {e:?}")),
        None => Ok(SovereignLayer::new(
            ts::custodian_root(),
            ts::governance_root(),
            p.regulatory_limit,
            p.max_supply,
            p.max_accounts,
        )),
    }
}

/// El estado que un titular conoce de su cuenta. En el sandbox se lee de
/// la capa por comodidad; en un despliegue lo custodia el cliente.
pub fn client_state(
    layer: &SovereignLayer,
    idx: AccountIndex,
) -> anyhow::Result<ClientState> {
    let missing = || anyhow::anyhow!("la cuenta #{idx} no existe");
    Ok(ClientState {
        public_id: layer.public_id_of(idx).ok_or_else(missing)?,
        balance: layer.balance_of(idx).ok_or_else(missing)?,
        nonce: layer.nonce_of(idx).ok_or_else(missing)?,
    })
}

/// Abre una cuenta (saldo CERO por diseño) y, si procede, la fondea por
/// la vía delegada: DOS custodios autorizan, cada uno consumiendo su
/// nullifier de umbral — que la traza muestra.
pub fn open_funded(
    layer: &mut SovereignLayer,
    key: Digest,
    amount: u64,
    tr: &mut dyn Tracer,
) -> anyhow::Result<AccountIndex> {
    let idx = layer.open_account_wide(key);
    tr.emit(&TraceEvent::Note {
        text: format!("cuenta #{idx} abierta — saldo CERO por diseño"),
    });
    if amount > 0 {
        fund_traced(layer, idx, amount, tr)?;
    }
    Ok(idx)
}

fn fund_traced(
    layer: &mut SovereignLayer,
    idx: AccountIndex,
    amount: u64,
    tr: &mut dyn Tracer,
) -> anyhow::Result<()> {
    let span = tracing::info_span!("fund", account = idx, amount).entered();

    tr.emit(&TraceEvent::PhaseStarted {
        phase: Phase::Fund,
        detail: format!("emisión delegada de {amount} a #{idx}: exige DOS custodios"),
    });

    // Compromiso de la operación + prueba de subida, como en la suite.
    let t0 = Instant::now();
    let op = ts::mint_commitment(layer, idx, amount);
    let subida = ts::mint_climb_proof(layer, idx, amount);
    tr.emit(&TraceEvent::ProofGenerated {
        phase: Phase::Fund,
        bytes: subida.to_bytes().len(),
        proof_digest: hex_short(&digest_of_proof(&subida.to_bytes())),
        ms: t0.elapsed().as_millis(),
    });

    // Autorizaciones de umbral (custodios 1 y 3, índices estrictos §51).
    // Sus public inputs llevan el NULLIFIER que cada uno consume: el
    // anti-replay de `circuit_threshold_single_nullifier`.
    let (pa, ia, pb, ib) = ts::delegated_pair(op, 1, 3);
    tr.emit(&TraceEvent::CustodianAuth {
        custodian_index: 1,
        nullifier: hex(&ia.nullifier),
        operation: hex_short(&ia.operation),
    });
    tr.emit(&TraceEvent::CustodianAuth {
        custodian_index: 3,
        nullifier: hex(&ib.nullifier),
        operation: hex_short(&ib.operation),
    });

    let root_old = layer.state_root();
    let t1 = Instant::now();
    layer
        .apply_mint_delegated(subida, pa, ia, pb, ib, idx, amount)
        .map_err(|e| fail(tr, Phase::Fund, e))?;
    emit_applied(layer, &root_old, t1, tr);

    drop(span);
    Ok(())
}

/// FASE 1 — el pagador envía. La capa entrega materiales (caminos y
/// raíces, datos públicos), el titular prueba EN LOCAL con su clave, y la
/// capa verifica y aplica. Ninguna clave llega a la capa.
pub fn run_send(
    layer: &mut SovereignLayer,
    from: AccountIndex,
    from_key: Digest,
    to: AccountIndex,
    amount: u64,
    salt_seed: u64,
    tr: &mut dyn Tracer,
) -> anyhow::Result<SendReceipt> {
    let span = tracing::info_span!("send", from, to, amount).entered();

    tr.emit(&TraceEvent::PhaseStarted {
        phase: Phase::Send,
        detail: format!(
            "#{from} → #{to}, importe {amount}: materiales → prueba LOCAL → apply_send"
        ),
    });

    let estado = client_state(layer, from)?;
    let receptor = layer
        .public_id_of(to)
        .ok_or_else(|| anyhow::anyhow!("la cuenta receptora #{to} no existe"))?;

    let t0 = Instant::now();
    let m = layer
        .send_materials(from, receptor, amount, ts::salt_de(salt_seed))
        .map_err(|e| fail(tr, Phase::Send, e))?;
    tr.emit(&TraceEvent::MaterialsBuilt {
        phase: Phase::Send,
        pending_position: Some(m.pending_position),
        ms: t0.elapsed().as_millis(),
    });

    // En la máquina del titular: la clave no sale de aquí.
    let t1 = Instant::now();
    let envio = client::prove_send(&m, from_key, proof_options())
        .map_err(|e| fail(tr, Phase::Send, e))?;
    tr.emit(&TraceEvent::ProofGenerated {
        phase: Phase::Send,
        bytes: envio.proof.len(),
        proof_digest: hex_short(&digest_of_proof(&envio.proof)),
        ms: t1.elapsed().as_millis(),
    });

    let root_old = layer.state_root();
    let t2 = Instant::now();
    layer
        .apply_send(&envio, from, &estado, amount)
        .map_err(|e| fail(tr, Phase::Send, e))?;
    emit_applied(layer, &root_old, t2, tr);
    tr.emit(&TraceEvent::Note {
        text: "el dinero está EN TRÁNSITO: no es del receptor hasta que cobre (§29)".into(),
    });

    drop(span);
    Ok(envio)
}

/// FASE 2 — el receptor cobra el pendiente con SU clave, también en local.
pub fn run_claim(
    layer: &mut SovereignLayer,
    to: AccountIndex,
    to_key: Digest,
    notice: &PendingNotice,
    tr: &mut dyn Tracer,
) -> anyhow::Result<()> {
    let span = tracing::info_span!("claim", to).entered();

    tr.emit(&TraceEvent::PhaseStarted {
        phase: Phase::Claim,
        detail: format!("#{to} cobra el aviso: materiales → prueba LOCAL → apply_claim"),
    });

    let estado = client_state(layer, to)?;

    let t0 = Instant::now();
    let m = layer
        .claim_materials(to, notice)
        .map_err(|e| fail(tr, Phase::Claim, e))?;
    tr.emit(&TraceEvent::MaterialsBuilt {
        phase: Phase::Claim,
        pending_position: None,
        ms: t0.elapsed().as_millis(),
    });

    let t1 = Instant::now();
    let cobro = client::prove_claim(&m, to_key, proof_options())
        .map_err(|e| fail(tr, Phase::Claim, e))?;
    tr.emit(&TraceEvent::ProofGenerated {
        phase: Phase::Claim,
        bytes: cobro.proof.len(),
        proof_digest: hex_short(&digest_of_proof(&cobro.proof)),
        ms: t1.elapsed().as_millis(),
    });

    let root_old = layer.state_root();
    let t2 = Instant::now();
    layer
        .apply_claim(&cobro, to, &estado, notice)
        .map_err(|e| fail(tr, Phase::Claim, e))?;
    emit_applied(layer, &root_old, t2, tr);

    drop(span);
    Ok(())
}

/// Resumen del estado, leído entero de la API pública de la capa.
pub fn emit_summary(layer: &SovereignLayer, tr: &mut dyn Tracer) {
    let head = layer.epoch_head(zk_ssl_verify::acuses::as_digest(0), 0, zk_ssl_verify::acuses::as_digest(0), 0);
    tr.emit(&TraceEvent::StateSummary {
        accounts: layer.account_count(),
        total_supply: layer.total_supply(),
        total_pending: layer.total_pending(),
        state_root: hex_short(&layer.state_root()),
        pending_root: hex_short(&layer.pending_root()),
        frozen_root: hex_short(&layer.frozen_root()),
        log_len: layer.transition_log().len(),
        log_head: hex_short(&layer.log_head()),
        epoch_digest: hex_short(&head.digest()),
    });
}

fn emit_applied(
    layer: &SovereignLayer,
    root_old: &Digest,
    t0: Instant,
    tr: &mut dyn Tracer,
) {
    if let Some(e) = layer.transition_log().entries().last() {
        tr.emit(&TraceEvent::Applied {
            op: format!("{:?}", e.kind),
            log_seq: e.seq,
            root_old: hex_short(root_old),
            root_new: hex_short(&e.root_new),
            chain: hex_short(&e.chain),
            ms: t0.elapsed().as_millis(),
        });
    }
}

fn fail(tr: &mut dyn Tracer, phase: Phase, e: LayerError) -> anyhow::Error {
    tr.emit(&TraceEvent::Rejected { phase, error: format!("{e:?}") });
    anyhow::anyhow!("{phase}: {e:?}")
}
