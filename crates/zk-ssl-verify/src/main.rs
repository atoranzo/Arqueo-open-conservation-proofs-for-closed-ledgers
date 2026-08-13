//! # `zk-ssl-verify` — el PAQUETE DE EVIDENCIA PORTABLE (§289, nota 83.1)
//!
//! Lo que sostiene la posicion del titular ante un tercero **cuando el
//! operador desaparece o miente**: las respuestas del cable que ya
//! custodia, reunidas en UN fichero, verificadas **sin el nodo, sin la
//! capa y sin el probador** (§243) — solo este binario y lo publicado.
//!
//! ## El formato, v1 — superficie declarada
//!
//! Un JSON con las respuestas del cable TAL CUAL el nodo las sirve, sin
//! reescribir (quien reescribe, adultera):
//!
//! ```text
//! {
//!   "v": 1,
//!   "cabeza": { …payload de zkssl_signedEpochHead con available:true… },
//!   "acuse": {                                  // OPCIONAL
//!     "seq": "0x…",                             // la entrada del titular
//!     "hashPrueba": "0x…64hex",                 // digest de SU prueba
//!     "s": "0x…",                               // de zkssl_ackPath
//!     "camino": { "siblings": […], "isRight": […] }
//!   }
//! }
//! ```
//!
//! ## Lo que se comprueba, en orden
//!
//! 1. los campos de la cabeza recomponen el digest DE SU VERSION == el
//!    `epochDigest` empaquetado — el digest no se cree: se recomputa;
//! 2. la firma XMSS verifica contra `publicKey` Y el preambulo recuperado
//!    es el esperado (verificar sin comparar no prueba nada, ver lib.rs);
//! 3. si hay acuse: `hoja_de_acuse(hashPrueba, seq, n)` sube por el
//!    camino hasta `acusesRoot`, y los siete vuelven a componer el digest
//!    firmado (`verificar_acuse`).
//!
//! ⚠️ Cabezas **v2 y v3** (`formatVersion`): la version que la firma
//! declara ELIGE RECOMPONEDOR — v2 con la pareja de acuses (§275), v3
//! ademas con la del MMR (`mmrRoot`/`mmrSize`, §292). Una cabeza v2
//! custodiada SIGUE verificando: el apagado de §290 no caduca. Una v1
//! se verifica con la biblioteca, no con este mando.
//!
//! ⚠️ Este binario tambien es **el procedimiento de apagado** (nota 91):
//! apaga el nodo, y una posicion sigue siendo demostrable sin el.
//!
//! Salida: VERDE y exit 0, o el primer fallo con nombre y exit 1.
use std::process::ExitCode;

use zk_ssl_verify::{acuses, verificar_acuse, verificar_acuse_v3, verificar_cabeza, CabezaFirmada, ReciboAcuse};
use zk_ssl_hash::{digest_from_bytes, epoch_digest_v2, epoch_digest_v3, Digest};

/// Punto unico de forma de error del binario (hoy identidad; el dia que
/// haga falta contexto comun, se anade AQUI y no en veinte sitios).
fn err(m: String) -> String {
    m
}

fn hex_a_bytes(s: &str) -> Result<Vec<u8>, String> {
    let h = s.strip_prefix("0x").ok_or_else(|| err(format!("sin 0x: {s:.18}")))?;
    if h.len() % 2 != 0 {
        return Err(err(format!("hex impar ({} chars)", h.len())));
    }
    (0..h.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&h[i..i + 2], 16).map_err(|e| err(format!("hex: {e}"))))
        .collect()
}

fn digest_de(v: &serde_json::Value, campo: &str) -> Result<Digest, String> {
    let s = v
        .get(campo)
        .and_then(|x| x.as_str())
        .ok_or_else(|| err(format!("falta {campo} o no es cadena")))?;
    let b = hex_a_bytes(s)?;
    let arr: [u8; 32] = b
        .as_slice()
        .try_into()
        .map_err(|_| err(format!("{campo}: {} bytes, se esperaban 32", b.len())))?;
    digest_from_bytes(&arr).map_err(|e| err(format!("{campo}: {e:?}")))
}

fn u64_de(v: &serde_json::Value, campo: &str) -> Result<u64, String> {
    let s = v
        .get(campo)
        .and_then(|x| x.as_str())
        .ok_or_else(|| err(format!("falta {campo} o no es cadena 0x")))?;
    let h = s.strip_prefix("0x").ok_or_else(|| err(format!("{campo} sin 0x")))?;
    u64::from_str_radix(h, 16).map_err(|e| err(format!("{campo}: {e}")))
}

fn correr(ruta: &str) -> Result<(), String> {
    let crudo = std::fs::read_to_string(ruta).map_err(|e| err(format!("no se puede leer {ruta}: {e}")))?;
    let p: serde_json::Value =
        serde_json::from_str(&crudo).map_err(|e| err(format!("JSON ilegible: {e}")))?;
    if p.get("v").and_then(|x| x.as_u64()) != Some(1) {
        return Err(err("el paquete no declara v:1".into()));
    }
    if p.get("tipo").and_then(|x| x.as_str()) == Some("extension") {
        return verificar_extension(&p);
    }
    let c = p.get("cabeza").ok_or_else(|| err("falta cabeza".into()))?;
    if c.get("available").and_then(|x| x.as_bool()) != Some(true) {
        return Err(err("la cabeza empaquetada no era available:true".into()));
    }
    let version = u64_de(c, "formatVersion")?;
    if version != 2 && version != 3 {
        return Err(err(format!(
            "formatVersion {version}: el paquete v1 empaqueta cabezas v2 o v3 \
             (la pareja acusesRoot/n viaja firmada desde §275; la del MMR, desde §292)"
        )));
    }
    let seq = u64_de(c, "seq")?;
    let n = u64_de(c, "n")?;
    let accounts = digest_de(c, "accountsRoot")?;
    let pending = digest_de(c, "pendingRoot")?;
    let frozen = digest_de(c, "frozenRoot")?;
    let chain = digest_de(c, "chainDigest")?;
    let acuses_root = digest_de(c, "acusesRoot")?;
    let epoch_digest = digest_de(c, "epochDigest")?;

    // 1 · el digest NO se cree: se recompone — y LA VERSION ELIGE RECOMPONEDOR
    let mmr = if version == 3 {
        Some((digest_de(c, "mmrRoot")?, u64_de(c, "mmrSize")?))
    } else {
        None
    };
    let compuesto = match mmr {
        None => epoch_digest_v2(seq, accounts, pending, frozen, chain, acuses_root, n),
        Some((cima, t)) => {
            epoch_digest_v3(seq, accounts, pending, frozen, chain, acuses_root, n, cima, t)
        }
    };
    if compuesto != epoch_digest {
        return Err(err(
            "los siete campos NO recomponen el epochDigest empaquetado: \
             o el paquete esta adulterado o la cabeza nunca fue esa"
                .into(),
        ));
    }
    println!("1/3 los campos de la cabeza (v{version}) recomponen el epochDigest — el digest no se ha creido");

    // 2 · la firma, contra la clave publicada, comparando el preambulo
    let clave = c
        .get("publicKey")
        .and_then(|x| x.as_str())
        .ok_or_else(|| err("falta publicKey".into()))?;
    let firma = c
        .get("signature")
        .and_then(|x| x.as_str())
        .ok_or_else(|| err("falta signature".into()))?;
    let cf = CabezaFirmada {
        version_formato: version as u8,
        indice: u64_de(c, "index")?,
        firma: hex_a_bytes(firma)?,
    };
    let mut ed = [0u8; 32];
    ed.copy_from_slice(&hex_a_bytes(c.get("epochDigest").and_then(|x| x.as_str()).unwrap())?);
    verificar_cabeza(&hex_a_bytes(clave)?, &ed, &cf).map_err(|e| err(format!("cabeza: {e}")))?;
    println!("2/3 la firma verifica y el preambulo ES el esperado (indice de firma {})", cf.indice);

    // 3 · el acuse, si viaja
    match p.get("acuse") {
        None => {
            println!("3/3 sin acuse en el paquete: la cabeza sola queda demostrada");
        }
        Some(a) => {
            let hash_prueba = digest_de(a, "hashPrueba")?;
            let seq_a = u64_de(a, "seq")?;
            let cam = a.get("camino").ok_or_else(|| err("acuse sin camino".into()))?;
            let sib = cam
                .get("siblings")
                .and_then(|x| x.as_array())
                .ok_or_else(|| err("camino sin siblings".into()))?;
            let der = cam
                .get("isRight")
                .and_then(|x| x.as_array())
                .ok_or_else(|| err("camino sin isRight".into()))?;
            let hermanos = sib
                .iter()
                .enumerate()
                .map(|(i, s)| {
                    let s = s.as_str().ok_or_else(|| err(format!("sibling {i} no es cadena")))?;
                    let b = hex_a_bytes(s)?;
                    let arr: [u8; 32] = b
                        .as_slice()
                        .try_into()
                        .map_err(|_| err(format!("sibling {i}: {} bytes", b.len())))?;
                    digest_from_bytes(&arr).map_err(|e| err(format!("sibling {i}: {e:?}")))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let derecha = der
                .iter()
                .map(|x| x.as_bool().ok_or_else(|| err("isRight no booleano".into())))
                .collect::<Result<Vec<_>, _>>()?;
            let recibo = ReciboAcuse {
                hoja: acuses::hoja_de_acuse(hash_prueba, seq_a, n),
                hermanos,
                derecha,
                seq,
                accounts_root: accounts,
                pending_root: pending,
                frozen_root: frozen,
                chain_digest: chain,
                acuses_root,
                n,
            };
            match mmr {
                None => verificar_acuse(&recibo, epoch_digest)
                    .map_err(|e| err(format!("acuse: {e:?}")))?,
                Some((cima, t)) => verificar_acuse_v3(&recibo, cima, t, epoch_digest)
                    .map_err(|e| err(format!("acuse: {e:?}")))?,
            }
            println!("3/3 el acuse sube hasta la raiz firmada: la entrada {seq_a} queda demostrada");
        }
    }
    println!("VERDE: el paquete se sostiene sin el nodo");
    Ok(())
}

/// Una cabeza **v3** del paquete de extension: se verifica ENTERA — el
/// digest se recompone (nunca se cree) y la firma se comprueba — y
/// devuelve lo que la consistencia necesita: su pareja del MMR y la
/// clave que la firmo.
fn cabeza_v3_verificada(c: &serde_json::Value, cual: &str) -> Result<(Digest, u64, String), String> {
    if c.get("available").and_then(|x| x.as_bool()) != Some(true) {
        return Err(err(format!("{cual}: la cabeza no era available:true")));
    }
    let version = u64_de(c, "formatVersion")?;
    if version != 3 {
        return Err(err(format!(
            "{cual}: formatVersion {version} — la extension exige cabezas v3: \
             una v2 no lleva la pareja del MMR que extender"
        )));
    }
    let seq = u64_de(c, "seq")?;
    let compuesto = epoch_digest_v3(
        seq,
        digest_de(c, "accountsRoot")?,
        digest_de(c, "pendingRoot")?,
        digest_de(c, "frozenRoot")?,
        digest_de(c, "chainDigest")?,
        digest_de(c, "acusesRoot")?,
        u64_de(c, "n")?,
        digest_de(c, "mmrRoot")?,
        u64_de(c, "mmrSize")?,
    );
    if compuesto != digest_de(c, "epochDigest")? {
        return Err(err(format!(
            "{cual}: los campos NO recomponen su epochDigest — adulterada o inventada"
        )));
    }
    let clave = c
        .get("publicKey")
        .and_then(|x| x.as_str())
        .ok_or_else(|| err(format!("{cual}: falta publicKey")))?;
    let firma = c
        .get("signature")
        .and_then(|x| x.as_str())
        .ok_or_else(|| err(format!("{cual}: falta signature")))?;
    let cf = CabezaFirmada {
        version_formato: version as u8,
        indice: u64_de(c, "index")?,
        firma: hex_a_bytes(firma)?,
    };
    let mut ed = [0u8; 32];
    ed.copy_from_slice(&hex_a_bytes(
        c.get("epochDigest").and_then(|x| x.as_str()).unwrap(),
    )?);
    verificar_cabeza(&hex_a_bytes(clave)?, &ed, &cf)
        .map_err(|e| err(format!("{cual}: cabeza: {e}")))?;
    Ok((digest_de(c, "mmrRoot")?, u64_de(c, "mmrSize")?, clave.to_string()))
}

/// El paquete de EXTENSION (§293): dos cabezas v3 firmadas y el camino
/// de consistencia entre sus cimas — el eslabon 2 entero, verificable
/// **sin el nodo**: quien custodia la vieja comprueba que la nueva la
/// EXTIENDE, con el objeto de §291 como juez.
fn verificar_extension(p: &serde_json::Value) -> Result<(), String> {
    let (cima_v, t_v, clave_v) = cabeza_v3_verificada(
        p.get("vieja").ok_or_else(|| err("falta vieja".into()))?,
        "vieja",
    )?;
    let (cima_n, t_n, clave_n) = cabeza_v3_verificada(
        p.get("nueva").ok_or_else(|| err("falta nueva".into()))?,
        "nueva",
    )?;
    println!("1/3 las DOS cabezas v3 recomponen su digest y sus firmas verifican");
    if clave_v != clave_n {
        return Err(err(
            "las cabezas llevan claves DISTINTAS: la continuidad es de UN firmante".into(),
        ));
    }
    println!("2/3 misma publicKey: el mismo firmante en los dos extremos");
    let cam = p
        .get("camino")
        .and_then(|x| x.as_array())
        .ok_or_else(|| err("falta camino (lista de digests)".into()))?;
    let camino = cam
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let s = s.as_str().ok_or_else(|| err(format!("camino[{i}] no es cadena")))?;
            let bts = hex_a_bytes(s)?;
            let arr: [u8; 32] = bts
                .as_slice()
                .try_into()
                .map_err(|_| err(format!("camino[{i}]: {} bytes", bts.len())))?;
            digest_from_bytes(&arr).map_err(|e| err(format!("camino[{i}]: {e:?}")))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if !zk_ssl_verify::mmr::verificar_consistencia(cima_v, t_v, cima_n, t_n, &camino) {
        return Err(err(format!(
            "la nueva (t={t_n}) NO extiende a la vieja (t={t_v}): historia \
             bifurcada, recortada, o camino que no es el suyo"
        )));
    }
    println!("3/3 la cima nueva EXTIENDE a la vieja: consistencia O(log N), sin el registro");
    println!("VERDE: la extension se sostiene sin el nodo");
    Ok(())
}

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let ruta = match (args.next(), args.next()) {
        (Some(r), None) => r,
        _ => {
            eprintln!("uso: zk-ssl-verify <paquete.json>");
            eprintln!("     (o de extension: {{v:1, tipo: extension, vieja, nueva, camino}})");
            eprintln!("     (formato v1 en la cabecera de este binario)");
            return ExitCode::from(2);
        }
    };
    match correr(&ruta) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("ROJO: {e}");
            ExitCode::FAILURE
        }
    }
}
