//! Vectores de conformidad (nota 74, Fase 1): el escenario canonico —
//! DETERMINISTA de punta a punta, prueba STARK incluida (el prover de
//! winterfell no tira dados; §197 lo midio cruzando CLI↔RPC) — reducido
//! a los hechos que una SEGUNDA implementacion debe reproducir: por
//! operacion, raiz vieja y nueva, digest de la prueba y digest de
//! cadena; al final, la cabeza de epoca y el suministro. `--emit` los
//! fija en disco; `--check` re-ejecuta el escenario y compara campo a
//! campo. Los digests van en el hex canonico del proyecto (el de
//! `store::digest_to_bytes`): el mismo byte a byte que persiste la capa
//! y que define `spec/RPC.md`.

use clap::Args;
use serde::{Deserialize, Serialize};
use zk_ssl::SovereignLayer;

use crate::sandbox::{self, Params};
use crate::trace::{TraceEvent, Tracer};

/// El escenario canonico, en una linea (queda dentro del fichero).
pub const ESCENARIO: &str =
    "open+fund(seed 0xA11CE, +1000000) x2 · send 250000 (salt 7) · claim";

#[derive(Args)]
pub struct ConformanceArgs {
    /// Escribe los vectores del escenario canonico en esta ruta.
    #[arg(long, conflicts_with = "check")]
    emit: Option<String>,
    /// Re-ejecuta el escenario y compara contra esta ruta.
    #[arg(long)]
    check: Option<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Entrada {
    seq: String,
    kind: String,
    root_old: String,
    root_new: String,
    proof_digest: String,
    chain: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Vectores {
    spec: String,
    sellado: String,
    escenario: String,
    canon: [u64; 4],
    entradas: Vec<Entrada>,
    epoch_digest: String,
    supply: String,
    pending: String,
}

fn hexd(d: &crate::fmt::Digest) -> String {
    format!("0x{}", crate::fmt::hex(d))
}

fn q(n: u64) -> String {
    format!("0x{n:x}")
}

fn escenario(tr: &mut dyn Tracer) -> anyhow::Result<SovereignLayer> {
    let mut layer = sandbox::open_layer(None, Params::default())?;
    let seed = 0xA11CE;
    let a0 = sandbox::open_funded(&mut layer, sandbox::key_of(seed, 0), 1_000_000, tr)?;
    let a1 = sandbox::open_funded(&mut layer, sandbox::key_of(seed, 1), 1_000_000, tr)?;
    let envio =
        sandbox::run_send(&mut layer, a0, sandbox::key_of(seed, 0), a1, 250_000, 7, tr)?;
    sandbox::run_claim(&mut layer, a1, sandbox::key_of(seed, 1), &envio.notice, tr)?;
    Ok(layer)
}

fn recolectar(layer: &SovereignLayer) -> Vectores {
    let entradas = layer
        .transition_log()
        .entries()
        .iter()
        .map(|e| Entrada {
            seq: q(e.seq),
            kind: format!("{:?}", e.kind),
            root_old: hexd(&e.root_old),
            root_new: hexd(&e.root_new),
            proof_digest: hexd(&e.proof_digest),
            chain: hexd(&e.chain),
        })
        .collect();
    Vectores {
        spec: "zkssl/0.1".into(),
        sellado: "§197".into(),
        escenario: ESCENARIO.into(),
        canon: [297, 242, 40, 28],
        entradas,
        epoch_digest: hexd(&layer.epoch_head().digest()),
        supply: q(layer.total_supply()),
        pending: q(layer.total_pending()),
    }
}

pub fn conformance(a: ConformanceArgs, tr: &mut dyn Tracer) -> anyhow::Result<()> {
    tr.emit(&TraceEvent::Note {
        text: format!("escenario canonico: {ESCENARIO} (pruebas reales, unos segundos)"),
    });
    let layer = escenario(tr)?;
    let ahora = recolectar(&layer);
    match (a.emit, a.check) {
        (Some(ruta), None) => {
            let js = serde_json::to_string_pretty(&ahora)?;
            std::fs::write(&ruta, js + "\n")?;
            tr.emit(&TraceEvent::Note {
                text: format!(
                    "vectores emitidos: {} entradas + cabeza + suministro -> {ruta}",
                    ahora.entradas.len()
                ),
            });
            Ok(())
        }
        (None, Some(ruta)) => {
            let texto = std::fs::read_to_string(&ruta)?;
            let fijo: Vectores = serde_json::from_str(&texto)?;
            if fijo.spec != ahora.spec || fijo.canon != ahora.canon {
                anyhow::bail!(
                    "los vectores son de OTRA version: {} / {:?} (aqui: {} / {:?})",
                    fijo.spec, fijo.canon, ahora.spec, ahora.canon
                );
            }
            if fijo.entradas.len() != ahora.entradas.len() {
                anyhow::bail!(
                    "numero de entradas: fijo {} vs ahora {}",
                    fijo.entradas.len(), ahora.entradas.len()
                );
            }
            let mut malas = 0usize;
            for (f, n) in fijo.entradas.iter().zip(ahora.entradas.iter()) {
                if f != n {
                    malas += 1;
                    tr.emit(&TraceEvent::Note {
                        text: format!("DIFIERE seq {}:\n  fijo  {f:?}\n  ahora {n:?}", f.seq),
                    });
                }
            }
            let colas = [
                ("epoch_digest", &fijo.epoch_digest, &ahora.epoch_digest),
                ("supply", &fijo.supply, &ahora.supply),
                ("pending", &fijo.pending, &ahora.pending),
            ];
            for (nom, va, vb) in colas {
                if va != vb {
                    malas += 1;
                    tr.emit(&TraceEvent::Note { text: format!("DIFIERE {nom}: {va} vs {vb}") });
                }
            }
            if malas == 0 {
                tr.emit(&TraceEvent::Note {
                    text: format!(
                        "CONFORMIDAD: {} entradas + cabeza + suministro, todo IDENTICO",
                        ahora.entradas.len()
                    ),
                });
                Ok(())
            } else {
                anyhow::bail!("conformidad ROTA: {malas} diferencia(s)")
            }
        }
        _ => anyhow::bail!("usa exactamente uno: --emit RUTA o --check RUTA"),
    }
}
