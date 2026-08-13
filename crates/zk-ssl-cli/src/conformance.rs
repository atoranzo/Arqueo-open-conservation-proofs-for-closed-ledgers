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
    /// Era 2 (§281): el compromiso autorizante, o el centinela
    /// declarado. Sin el, las cadenas v2 del vector serian
    /// irreproducibles por una segunda implementacion. Ausente solo en
    /// entradas de la era 1, que el escenario canonico no produce.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    compromiso: Option<String>,
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
            compromiso: e.compromiso.as_ref().map(hexd),
        })
        .collect();
    Vectores {
        spec: "zkssl/0.2".into(),
        // Re-emitido en §278: las cuatro entradas delegadas del escenario
        // —dos OpenAccount y dos Mint— dejan de asentar la prueba vacia,
        // asi que cambian sus digests, TODA la cadena y la cabeza.
        // Re-emitido en §281: el compromiso autorizante entra como
        // campo y en la cadena (v2) — cambian las seis cadenas, la
        // cabeza, y cada entrada publica su compromiso (las Mint el
        // real; el resto el centinela declarado).
        sellado: "§281".into(),
        escenario: ESCENARIO.into(),
        // §207 sumo tres tests al arbol disperso: 242 -> 245.
        canon: [297, 245, 40, 28],
        entradas,
        // ⚠️ §292: el vector SELLADO pina LA COMPOSICION V2 — es un artefacto
        // congelado y su significado no se mueve con el formato vivo. Por eso
        // aqui se compone v2 EXPLICITO en vez de llamar a digest(), que desde
        // §292 compone v3. Los campos son los mismos siete de siempre.
        epoch_digest: hexd(&{
            let h = layer.epoch_head(
                zk_ssl_verify::acuses::as_digest(0), 0,
                zk_ssl_verify::acuses::as_digest(0), 0,
            );
            zk_ssl_verify::epoch_digest_v2(
                h.seq, h.accounts_root, h.pending_root, h.frozen_root,
                h.chain_digest, h.acuses_root, h.n,
            )
        }),
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
