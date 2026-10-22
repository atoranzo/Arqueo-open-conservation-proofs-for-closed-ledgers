//! Vectores de conformidad (nota 74, Fase 1): el escenario canonico —
//! determinista en lo que el circuito fija y, desde zkssl/0.4 (RFC-0009
//! E3b, §538), NO en los bytes de la prueba: con la ocultacion encendida
//! cada prueba lleva sal y filas aleatorias (D-X), asi que proof_digest,
//! chain y epoch_digest cambian de una emision a otra y se DECLARAN no
//! reproducibles (D-AI; hasta la 0.3 el prover apagado no tiraba dados y
//! §197 lo midio cruzando CLI↔RPC) — reducido a los hechos que una
//! SEGUNDA implementacion debe reproducir: por operacion, raiz vieja y
//! nueva y el compromiso; al final, el suministro y el pendiente. `--emit`
//! los fija en disco, digests incluidos: un vector es lo que su version
//! produjo; `--check` re-ejecuta el escenario y compara lo que se fija.
//! Los digests van en el hex canonico del proyecto (el de
//! `store::digest_to_bytes`): el mismo byte a byte que persiste la capa
//! y que define `spec/RPC.md`.

use clap::Args;
use serde::{Deserialize, Serialize};
use zk_ssl::SovereignLayer;

use crate::sandbox::{self, Params};
use crate::trace::{TraceEvent, Tracer};

/// El escenario canonico, en una linea (queda dentro del fichero).
pub const ESCENARIO: &str =
    "open+fund(seed 0xA11CE, +1000000) x2 · send v2 250000 (salt 7, delta 96) · claim";

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
    // v2 (RFC-0003, E3c-2): el emisor fija la pareja (f, delta). f = el
    // public_id del emisor (determinista via key_of); delta = 96, A
    // PROPOSITO distinto de 64 (DEFAULT_REFUND_TTL global v1): son ejes
    // distintos y que se lea.
    let f = layer.public_id_of(a0).expect("a0 acaba de abrirse");
    let envio = sandbox::run_send_v2(
        &mut layer, a0, sandbox::key_of(seed, 0), a1, 250_000, 7, f, 96, tr,
    )?;
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
        spec: "zkssl/0.5".into(),
        // Re-emitido en §278: las cuatro entradas delegadas del escenario
        // —dos OpenAccount y dos Mint— dejan de asentar la prueba vacia,
        // asi que cambian sus digests, TODA la cadena y la cabeza.
        // Re-emitido en §281: el compromiso autorizante entra como
        // campo y en la cadena (v2) — cambian las seis cadenas, la
        // cabeza, y cada entrada publica su compromiso (las Mint el
        // real; el resto el centinela declarado).
        // Re-emitido como zkssl/0.3 (RFC-0003, E3c-2, §354): el envio del
        // escenario pasa a la VIA v2 -- el sobre entra por los materiales
        // y el aviso viaja con la X opaca; cambian el compromiso de la
        // hoja pendiente, la prueba del Send, la cadena desde ahi y la
        // cabeza. El campo `canon` estrena semantica DECLARADA (foto al
        // emitir): [tests de circuitos, tests de la capa, suma de la
        // compuerta de sello, circuitos con impl Air]. Los antiguos 40
        // (sin fuente nombrada en el registro) y 28: ver el asiento.
        // Re-emitido como zkssl/0.4 (RFC-0009 E3b, §538): los 23 probadores con
        // fila salen OCULTOS (D-Z), con sal y filas aleatorias de la entropia del
        // sistema (D-X). Dos emisiones del MISMO escenario ya no dan los mismos bytes
        // de prueba: proof_digest, chain y epoch_digest no se reproducen; seq, kind,
        // root_old, root_new, compromiso, supply y pending si. Lo mide el
        // PASTE-538-M (dos --emit, campo a campo) y lo que un --check puede exigir en
        // 0.4 lo decide el asiento.
        // Re-emitido como zkssl/0.5 en el §716 (el reenvio de una emision delegada): el
        // compromiso de `Mint` y de `MintToPending` lleva el contador de emisiones, `(n, n + 1)`,
        // y cambia el de las dos Mint del escenario (seq 1 y 3); el resto de lo que se fija, no
        // (medido con --check contra la 0.4). La 0.4 se conserva y se rechaza por version. El
        // escenario sube antes que el cable: el cable sigue diciendo zkssl/0.4 hasta el corte del
        // tren (RFC-0018 D-E). `canon` es la foto al emitir: circuitos, capa, suma de la
        // compuerta de sello y el censo de `impl Air for` en crates/.
        // ⚠️ Por eso el corte del tren NO re-emite este vector: los vectores no se reescriben, y
        // `--check` exige este `canon`. Si el corte cambia algo de lo que el escenario fija, el
        // escenario sube a zkssl/0.6; si no, este vector sigue valiendo para la 0.5.
        sellado: "§716".into(),
        escenario: ESCENARIO.into(),
        // §207 sumo tres tests al arbol disperso: 242 -> 245.
        canon: [427, 530, 1955, 40],
        entradas,
        // ⚠️ §292: el vector SELLADO pina LA COMPOSICION V2 — es un artefacto
        // congelado y su significado no se mueve con el formato vivo. Por eso
        // aqui se compone v2 EXPLICITO en vez de llamar a digest(), que desde
        // §292 compone v3. Los campos son los mismos siete de siempre.
        epoch_digest: hexd(&{
            let h = layer.epoch_head(
                zk_ssl_verify::acuses::as_digest(0), 0,
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

/// ⚠️ §781 · **Un vector publicado no se pisa** (regla 2 de `spec/rfc/PROCESO.md`). Busca el
/// `HUELLAS.sha256` más cercano subiendo desde el directorio de `ruta` y, si `ruta` está
/// registrada en él, exige que `bytes` den su misma huella: lo idéntico se deja escribir, lo
/// distinto se niega nombrando el fichero y su registro. Sin `HUELLAS.sha256` en el camino, o con
/// `ruta` sin registrar, no hay nada que proteger. La usan `conformance --emit` y el emisor de los
/// KAT del núcleo (`nucleo_kat.rs`), que hasta el §781 solo miraba el fichero en disco: uno
/// borrado se rehacía con otros bytes sin que nada lo dijera hasta el canon.
pub(crate) fn vigilar_huella(ruta: &std::path::Path, bytes: &[u8]) -> Result<(), String> {
    let absoluta = if ruta.is_absolute() {
        ruta.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| format!("directorio actual: {e}"))?
            .join(ruta)
    };
    let (Some(padre), Some(nombre)) = (absoluta.parent(), absoluta.file_name()) else {
        return Ok(());
    };
    // El padre resuelto (sin `..` ni enlaces), si existe: la ruta relativa al registro sale limpia.
    let padre = std::fs::canonicalize(padre).unwrap_or_else(|_| padre.to_path_buf());
    let fichero = padre.join(nombre);
    for dir in padre.ancestors() {
        let registro = dir.join("HUELLAS.sha256");
        let Ok(texto) = std::fs::read_to_string(&registro) else {
            continue;
        };
        let Some(relativa) = fichero.strip_prefix(dir).ok().and_then(|r| r.to_str()) else {
            return Ok(());
        };
        let relativa = relativa.replace(std::path::MAIN_SEPARATOR, "/");
        let registrada = texto
            .lines()
            .filter(|l| !l.starts_with('#'))
            .filter_map(|l| l.split_once("  "))
            .find(|(_, r)| *r == relativa)
            .map(|(h, _)| h.to_string());
        let Some(registrada) = registrada else {
            return Ok(());
        };
        let ahora: String = zk_ssl_medio::hash::sha256(bytes)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        return if ahora == registrada {
            Ok(())
        } else {
            Err(format!(
                "{}: es un vector publicado ({} lo registra con la huella {}…) y estos \
                 bytes dan {}…: un vector publicado no se reescribe (regla 2 de \
                 spec/rfc/PROCESO.md); se emite a otra ruta y, si su veredicto cambia, \
                 entra con otro nombre o bajo una version nueva. No se ha escrito nada.",
                ruta.display(),
                registro.display(),
                &registrada[..registrada.len().min(16)],
                &ahora[..16]
            ))
        };
    }
    Ok(())
}

pub fn conformance(a: ConformanceArgs, tr: &mut dyn Tracer) -> anyhow::Result<()> {
    tr.emit(&TraceEvent::Note {
        text: format!("escenario canonico: {ESCENARIO} (pruebas reales, unos segundos)"),
    });
    let layer = escenario(tr)?;
    let ahora = recolectar(&layer);
    match (a.emit, a.check) {
        (Some(ruta), None) => {
            let js = serde_json::to_string_pretty(&ahora)? + "\n";
            vigilar_huella(std::path::Path::new(&ruta), js.as_bytes())
                .map_err(anyhow::Error::msg)?;
            std::fs::write(&ruta, js)?;
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
            // D-AI (RFC-0009, zkssl/0.4): se cruza lo que el circuito fija; los digests que
            // llevan bytes de prueba no se reproducen con la ocultacion encendida.
            fn fija(e: &Entrada) -> (&str, &str, &str, &str, Option<&str>) {
                (&e.seq, &e.kind, &e.root_old, &e.root_new, e.compromiso.as_deref())
            }
            for (f, n) in fijo.entradas.iter().zip(ahora.entradas.iter()) {
                if fija(f) != fija(n) {
                    malas += 1;
                    tr.emit(&TraceEvent::Note {
                        text: format!("DIFIERE seq {}:\n  fijo  {f:?}\n  ahora {n:?}", f.seq),
                    });
                }
            }
            let colas = [
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
                        "CONFORMIDAD: {} entradas + suministro + pendiente, todo IDENTICO en lo \
                         que el circuito fija (proof_digest, chain y epoch_digest no se \
                         reproducen con la ocultacion: D-AI)",
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

#[cfg(test)]
mod tests {
    use super::vigilar_huella;

    /// ⚠️ §781 · Lo registrado en un `HUELLAS.sha256` del camino solo se escribe con sus mismos
    /// bytes; lo no registrado, o sin registro en el camino, se escribe.
    #[test]
    fn un_vector_registrado_solo_se_escribe_con_su_huella() {
        let raiz = std::env::temp_dir().join(format!("zkssl-huellas-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&raiz);
        std::fs::create_dir_all(raiz.join("sub")).unwrap_or_else(|e| panic!("crear: {e}"));
        let huella: String = zk_ssl_medio::hash::sha256(b"vale\n")
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        std::fs::write(
            raiz.join("HUELLAS.sha256"),
            format!("# comentario\n{huella}  sub/v.json\n"),
        )
        .unwrap_or_else(|e| panic!("registro: {e}"));
        let v = raiz.join("sub").join("v.json");
        assert_eq!(
            vigilar_huella(&v, b"vale\n"),
            Ok(()),
            "los mismos bytes pasan"
        );
        let e = vigilar_huella(&v, b"otro\n")
            .err()
            .unwrap_or_else(|| panic!("otros bytes pasaron"));
        assert!(
            e.contains("v.json") && e.contains("no se reescribe"),
            "nombra el fichero: {e}"
        );
        // A traves de `..`, el mismo fichero registrado.
        let rodeo = raiz.join("sub").join("..").join("sub").join("v.json");
        assert!(
            vigilar_huella(&rodeo, b"otro\n").is_err(),
            "el `..` no lo esconde"
        );
        assert_eq!(
            vigilar_huella(&raiz.join("sub").join("nuevo.json"), b"x"),
            Ok(())
        );
        let _ = std::fs::remove_dir_all(&raiz);
    }

    /// El vector de conformidad publicado está registrado: emitir encima con otros bytes se niega.
    #[test]
    fn el_vector_de_conformidad_publicado_esta_protegido() {
        let v = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../spec/vectors/zkssl-0.4.json");
        let publicado = std::fs::read(&v).unwrap_or_else(|e| panic!("leer {}: {e}", v.display()));
        assert_eq!(
            vigilar_huella(&v, &publicado),
            Ok(()),
            "lo publicado se reconoce"
        );
        assert!(
            vigilar_huella(&v, b"{}\n").is_err(),
            "otros bytes encima de zkssl-0.4.json se niegan"
        );
    }
}
