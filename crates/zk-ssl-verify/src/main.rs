//! # `zk-ssl-verify` — el PAQUETE DE EVIDENCIA PORTABLE (§289, nota 83.1)
//!
//! Lo que sostiene la posicion del titular ante un tercero **cuando el
//! operador desaparece o miente**: las respuestas del cable que ya
//! custodia, reunidas en UN fichero, verificadas **sin el nodo, sin la
//! capa y sin el probador** (§243) — solo este binario y lo publicado.
//!
//! ## Donde esta el contrato (§397)
//!
//! **El formato del paquete y el contrato de este mando se especifican en
//! `spec/PAQUETE.md`, y SOLO ahi**: las tres formas (v1, v2 con las
//! cofirmas dentro, extension), el sobre y cada clave que este binario lee,
//! el orden de comprobacion, el catalogo de rechazos y los codigos de
//! salida. Hasta §397 todo eso vivia AQUI (lineas 1..90, sha de region
//! `293990fedc785833`) y `spec/RPC.md` delegaba en esta cabecera: dos
//! productores del mismo contrato, y el que caduco fue el de dentro (abajo).
//!
//! ⚠️ Esta cabecera **ya no enumera**, por la misma razon que `Cargo.toml`
//! no enumera la superficie: una lista en prosa vuelve a caducar a la
//! primera forma nueva, y ya caduco una vez. La verdad del sobre se lee en
//! el documento; la del codigo, en el codigo. Al sellar se comprueba que
//! el catalogo del documento cubre CADA llamada de rechazo y CADA clave
//! que este fichero lee — censo por llamada, no por linea.
//!
//! ⚠️ **El paquete REPORTA, no juzga.** Dice cuantas cofirmas verifican
//! contra ESTA cabeza y ESTE operador. **Que testigos valen y cuantos hacen
//! falta lo decide el CLIENTE** con su politica (§319): quien arma el
//! paquete puede ser el operador, y dejarle elegir su propia k le devolveria
//! justo lo que la cofirma le quita.
//!
//! ## La tercera forma, que esta cabecera NO declaraba (§247)
//!
//! §397: lo que sigue es HISTORIA y se conserva citada, no borrada. «El
//! bloque de arriba» era la superficie declarada aqui hasta §397; hoy vive
//! en `spec/PAQUETE.md`, con el esqueleto de la extension incluido.
//!
//! Ademas del paquete de posicion, este binario verifica desde el
//! §293 el **paquete de EXTENSION**, y el bloque de arriba nunca lo dijo:
//!
//! No es una contradiccion: es una superficie declarada **como si fuera
//! completa**, que se lee peor que una incompleta que se sabe incompleta.
//! Estaba publicada en `spec/RPC.md` y ausente aqui: **el productor rancio
//! era el que mas cerca queda del codigo**.
//!
//! ⚠️ Este binario tambien es **el procedimiento de apagado** (nota 91):
//! apaga el nodo, y una posicion sigue siendo demostrable sin el.
//!
//! Salida: VERDE y exit 0; el primer fallo con nombre (`ROJO: …`) y exit 1;
//! uso —ningun argumento, o mas de uno— y exit 2. Los textos, en el
//! catalogo de `spec/PAQUETE.md`.
use std::process::ExitCode;

use zk_ssl_verify::{
    acuses, verificar_acuse, verificar_acuse_v3, indice_de_firma, verificar_cabeza, verificar_cofirma,
    CabezaFirmada, COFIRMA_V_MAX, ReciboAcuse, VersionCabeza,
};
use zk_ssl_hash::{digest_from_bytes, epoch_digest_v2, epoch_digest_v3, epoch_digest_v4, Digest};

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
    // §322 · v1 Y v2: lo custodiado no caduca. Un v1 con `cofirmas` se
    //          rechaza, porque subir la version es lo que las hace contrato.
    let v_paquete = p
        .get("v")
        .and_then(|x| x.as_u64())
        .ok_or_else(|| err("el paquete no declara su version en `v`".into()))?;
    if v_paquete != 1 && v_paquete != 2 {
        return Err(err(format!(
            "el paquete declara v:{v_paquete} — este binario lee v1 y v2"
        )));
    }
    if v_paquete == 1 && p.get("cofirmas").is_some() {
        return Err(err(
            "un paquete v1 con `cofirmas`: subir la version es lo que las hace \
             parte del contrato — declaralo v2, o quitalas"
                .into(),
        ));
    }
    if p.get("tipo").and_then(|x| x.as_str()) == Some("extension") {
        return verificar_extension(&p);
    }
    let c = p.get("cabeza").ok_or_else(|| err("falta cabeza".into()))?;
    if c.get("available").and_then(|x| x.as_bool()) != Some(true) {
        return Err(err("la cabeza empaquetada no era available:true".into()));
    }
    // §406 · RFC-0005 E2: el conjunto lo produce `VersionCabeza` y aqui se CONSUME;
    //        el texto de este rechazo lo fija el MANIFIESTO del paquete.
    let version = VersionCabeza::try_from(u64_de(c, "formatVersion")?).map_err(|e| {
        err(format!(
            "formatVersion {}: el paquete v1 empaqueta cabezas {} \
             (la pareja acusesRoot/n viaja firmada desde §275; la del MMR, desde §292)",
            e.0,
            VersionCabeza::texto()
        ))
    })?;
    let seq = u64_de(c, "seq")?;
    let n = u64_de(c, "n")?;
    let accounts = digest_de(c, "accountsRoot")?;
    let pending = digest_de(c, "pendingRoot")?;
    let frozen = digest_de(c, "frozenRoot")?;
    let chain = digest_de(c, "chainDigest")?;
    let acuses_root = digest_de(c, "acusesRoot")?;
    let epoch_digest = digest_de(c, "epochDigest")?;

    // 1 · el digest NO se cree: se recompone — y LA VERSION ELIGE RECOMPONEDOR
    //     (RFC-0006 E2a, §414: cada pareja la decide un `match` EXHAUSTIVO sobre
    //     `VersionCabeza`, y el compilador marca el brazo que falte; el 3/3 de
    //     abajo elegia por un `Option`, y una v4 habria pasado por v3 en silencio).
    let mmr = match version {
        VersionCabeza::V2 => None,
        VersionCabeza::V3 | VersionCabeza::V4 => {
            Some((digest_de(c, "mmrRoot")?, u64_de(c, "mmrSize")?))
        }
    };
    let cons = match version {
        VersionCabeza::V2 | VersionCabeza::V3 => None,
        VersionCabeza::V4 => Some((digest_de(c, "consRoot")?, u64_de(c, "consCount")?)),
    };
    let compuesto = match (mmr, cons) {
        (None, _) => epoch_digest_v2(seq, accounts, pending, frozen, chain, acuses_root, n),
        (Some((cima, t)), None) => {
            epoch_digest_v3(seq, accounts, pending, frozen, chain, acuses_root, n, cima, t)
        }
        (Some((cima, t)), Some((raiz, k))) => epoch_digest_v4(
            seq, accounts, pending, frozen, chain, acuses_root, n, cima, t, raiz, k,
        ),
    };
    if compuesto != epoch_digest {
        return Err(err(
            "los siete campos NO recomponen el epochDigest empaquetado: \
             o el paquete esta adulterado o la cabeza nunca fue esa"
                .into(),
        ));
    }
    println!(
        "1/3 los campos de la cabeza (v{}) recomponen el epochDigest — el digest no se ha creido",
        version.as_u8()
    );

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
        version_formato: version.as_u8(),
        indice: u64_de(c, "index")?,
        firma: hex_a_bytes(firma)?,
    };
    let mut ed = [0u8; 32];
    ed.copy_from_slice(&hex_a_bytes(c.get("epochDigest").and_then(|x| x.as_str()).unwrap())?);
    // §322 · los bytes de la clave se atan a un nombre: las cofirmas los
    //          necesitan, y recomputarlos seria un segundo productor.
    let clave_op = hex_a_bytes(clave)?;
    verificar_cabeza(&clave_op, &ed, &cf).map_err(|e| err(format!("cabeza: {e}")))?;
    // §399 · se imprime el indice EMBEBIDO, el unico que la firma acredita;
    //        el declarado ya quedo atado a el dentro de verificar_cabeza.
    let hoja = indice_de_firma(&cf.firma).map_err(|e| err(format!("cabeza: {e}")))?;
    println!("2/3 la firma verifica y el preambulo ES el esperado (indice de firma {hoja})");

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
            match (mmr, cons) {
                (None, _) => verificar_acuse(&recibo, epoch_digest)
                    .map_err(|e| err(format!("acuse: {e:?}")))?,
                (Some((cima, t)), None) => verificar_acuse_v3(&recibo, cima, t, epoch_digest)
                    .map_err(|e| err(format!("acuse: {e:?}")))?,
                (Some((cima, t)), Some((raiz, k))) => {
                    zk_ssl_verify::verificar_acuse_v4(&recibo, cima, t, raiz, k, epoch_digest)
                        .map_err(|e| err(format!("acuse: {e:?}")))?
                }
            }
            println!("3/3 el acuse sube hasta la raiz firmada: la entrada {seq_a} queda demostrada");
        }
    }

    // 4 · las cofirmas, si el paquete es v2 (§322)
    if v_paquete == 2 {
        let k = verificar_cofirmas_del_paquete(&p, &ed, &clave_op)?;
        if k == 0 {
            println!("cofirmas: el paquete v2 no trae ninguna: la cabeza queda sola");
        } else {
            println!(
                "cofirmas: {k} verifican contra ESTA cabeza y ESTE operador \
                 (cuantas hacen falta lo decide TU politica, no el paquete)"
            );
        }
    }
    println!("VERDE: el paquete se sostiene sin el nodo");
    Ok(())
}

/// Un campo hex de una cofirma, con su numero en el error: un instrumento
/// que falla dice QUE fallo, no cuantos (§254).
fn hex_de_cofirma(c: &serde_json::Value, campo: &str, n: usize) -> Result<Vec<u8>, String> {
    let s = c
        .get(campo)
        .and_then(|x| x.as_str())
        .ok_or_else(|| err(format!("cofirma {n}: falta {campo}")))?;
    hex_a_bytes(s)
}

/// **Las cofirmas del paquete v2** (§322). Cada una acredita que UN testigo
/// vio ESTA cabeza de ESTE operador, y nada mas.
///
/// ⚠️⚠️ **REPORTA, NO JUZGA.** Devuelve CUANTAS verifican. Que testigos
/// valen y cuantos hacen falta lo decide el CLIENTE (§319, `--testigos` y
/// `--k` del mando del testigo), NO el paquete: quien lo arma puede ser el
/// operador. Una cofirma que NO verifica si es un fallo del paquete y para el
/// binario en rojo; cuantas hacen falta no es asunto suyo.
///
/// ⚠️ **Las respuestas del cable TAL CUAL.** `cofirmas` es el contenido de
/// `zkssl_cosigs` sin reescribir, asi que sus cantidades vienen en convencion
/// `Q` (cadena hex con `0x`) y sus bytes en hex: justo lo que `u64_de` y
/// `hex_a_bytes` ya leen. **No se recompone nada**, y por eso aqui no puede
/// haber blanqueo de version (§320).
///
/// ⚠️ El atado sale del propio paquete: `epoch_digest` es el de la cabeza
/// empaquetada y `clave_operador` es su `publicKey`. Una cofirma que nombre
/// otra cabeza u otro operador se rechaza ANTES de tocar la criptografia.
fn verificar_cofirmas_del_paquete(
    p: &serde_json::Value,
    epoch_digest: &[u8; 32],
    clave_operador: &[u8],
) -> Result<usize, String> {
    let lista = match p.get("cofirmas") {
        None => return Ok(0),
        Some(x) => x
            .as_array()
            .ok_or_else(|| err("cofirmas no es una lista".into()))?,
    };
    for (i, c) in lista.iter().enumerate() {
        let n = i + 1;
        let cv = u64_de(c, "v")?;
        if cv > COFIRMA_V_MAX {
            return Err(err(format!(
                "cofirma {n}: version {cv} desconocida, este binario lee hasta la {}",
                COFIRMA_V_MAX
            )));
        }
        let d = hex_de_cofirma(c, "epochDigest", n)?;
        if d.as_slice() != &epoch_digest[..] {
            return Err(err(format!(
                "cofirma {n}: acredita OTRA cabeza, no la empaquetada"
            )));
        }
        let ko = hex_de_cofirma(c, "clavePublicaOperador", n)?;
        if ko.as_slice() != clave_operador {
            return Err(err(format!(
                "cofirma {n}: acredita a OTRO operador, no al que firmo la cabeza"
            )));
        }
        let kt = hex_de_cofirma(c, "clavePublicaTestigo", n)?;
        let firma = hex_de_cofirma(c, "firma", n)?;
        let cf = CabezaFirmada {
            version_formato: u64_de(c, "versionFormato")? as u8,
            indice: u64_de(c, "indice")?,
            firma,
        };
        verificar_cofirma(&kt, epoch_digest, &ko, &cf)
            .map_err(|e| err(format!("cofirma {n}: {e}")))?;
    }
    Ok(lista.len())
}

/// Una cabeza **v3** del paquete de extension: se verifica ENTERA — el
/// digest se recompone (nunca se cree) y la firma se comprueba — y
/// devuelve lo que la consistencia necesita: su pareja del MMR y la
/// clave que la firmo.
fn cabeza_v3_verificada(c: &serde_json::Value, cual: &str) -> Result<(Digest, u64, String), String> {
    if c.get("available").and_then(|x| x.as_bool()) != Some(true) {
        return Err(err(format!("{cual}: la cabeza no era available:true")));
    }
    // RFC-0006 E2a (§414): la extension exige la pareja del MMR, que llevan v3 y
    // v4; el conjunto lo produce `VersionCabeza`, el texto se DERIVA de el y el
    // compilador marca el brazo que falte. Antes preguntaba «¿es 3?».
    let version = u64_de(c, "formatVersion")?;
    let seq = u64_de(c, "seq")?;
    let compuesto = match VersionCabeza::try_from(version) {
        Ok(VersionCabeza::V3) => epoch_digest_v3(
            seq,
            digest_de(c, "accountsRoot")?,
            digest_de(c, "pendingRoot")?,
            digest_de(c, "frozenRoot")?,
            digest_de(c, "chainDigest")?,
            digest_de(c, "acusesRoot")?,
            u64_de(c, "n")?,
            digest_de(c, "mmrRoot")?,
            u64_de(c, "mmrSize")?,
        ),
        Ok(VersionCabeza::V4) => epoch_digest_v4(
            seq,
            digest_de(c, "accountsRoot")?,
            digest_de(c, "pendingRoot")?,
            digest_de(c, "frozenRoot")?,
            digest_de(c, "chainDigest")?,
            digest_de(c, "acusesRoot")?,
            u64_de(c, "n")?,
            digest_de(c, "mmrRoot")?,
            u64_de(c, "mmrSize")?,
            digest_de(c, "consRoot")?,
            u64_de(c, "consCount")?,
        ),
        Ok(VersionCabeza::V2) | Err(_) => {
            return Err(err(format!(
                "{cual}: formatVersion {version} — la extension exige cabezas {}: \
                 una v2 no lleva la pareja del MMR que extender",
                VersionCabeza::texto_con_mmr()
            )))
        }
    };
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
            eprintln!("     (el formato y el catalogo de rechazos: spec/PAQUETE.md)");
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// ⚠️ Lo que estos tests prueban es **lo que este binario aporta**: el
    /// atado de la cofirma a la cabeza empaquetada, la version y la forma.
    /// La criptografia la prueban los tests del lib, que tienen con que
    /// firmar; aqui la firma es de mentira A PROPOSITO y por eso ningun test
    /// llega a `verificar_cofirma`.
    const DIG: &str = "0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20";
    const OTRO: &str = "0x0000000000000000000000000000000000000000000000000000000000000000";
    const OP: &str = "0xaa";

    fn ed() -> [u8; 32] {
        let mut a = [0u8; 32];
        for (i, b) in a.iter_mut().enumerate() {
            *b = (i + 1) as u8;
        }
        a
    }

    fn cofirma(dig: &str, op: &str) -> serde_json::Value {
        json!({
            "v": format!("0x{:x}", zk_ssl_verify::COFIRMA_VERSION),
            "epochDigest": dig,
            "clavePublicaOperador": op,
            "clavePublicaTestigo": "0xbb",
            "versionFormato": "0x3",
            "indice": "0x0",
            "firma": "0xcc",
            "vistoUnix": "0x0"
        })
    }

    /// Un paquete sin la clave no es un error: es un paquete sin cofirmas.
    #[test]
    fn sin_la_clave_no_hay_cofirmas_y_no_es_un_error() {
        let p = json!({ "v": 2 });
        assert_eq!(verificar_cofirmas_del_paquete(&p, &ed(), &[0xaa_u8]), Ok(0));
    }

    /// Y una lista vacia tampoco: cero cofirmas es una respuesta legitima
    /// del nodo (S317), no un fallo.
    #[test]
    fn una_lista_vacia_es_cero_y_no_un_error() {
        let p = json!({ "v": 2, "cofirmas": [] });
        assert_eq!(verificar_cofirmas_del_paquete(&p, &ed(), &[0xaa_u8]), Ok(0));
    }

    /// ⚠️⚠️ **EL TEST QUE JUSTIFICA EL ATADO.** Una cofirma legitima de OTRA
    /// cabeza pasaria su propia verificacion criptografica: lo que la hace
    /// inservible aqui es que no acredita LA cabeza del paquete.
    #[test]
    fn una_cofirma_de_otra_cabeza_no_ata() {
        let p = json!({ "v": 2, "cofirmas": [cofirma(OTRO, OP)] });
        let e = verificar_cofirmas_del_paquete(&p, &ed(), &[0xaa_u8]).unwrap_err();
        assert!(e.contains("OTRA cabeza"), "{e}");
    }

    /// Misma cabeza, otro operador: tampoco vale, y por la misma razon que
    /// el operador va DENTRO del preambulo firmado.
    #[test]
    fn una_cofirma_de_otro_operador_no_ata() {
        let p = json!({ "v": 2, "cofirmas": [cofirma(DIG, "0xab")] });
        let e = verificar_cofirmas_del_paquete(&p, &ed(), &[0xaa_u8]).unwrap_err();
        assert!(e.contains("OTRO operador"), "{e}");
    }

    /// Una version que este binario no sabe leer se RECHAZA, no se supone.
    #[test]
    fn una_version_de_cofirma_desconocida_se_rechaza() {
        let mut c = cofirma(DIG, OP);
        c["v"] = json!("0x2");
        let p = json!({ "v": 2, "cofirmas": [c] });
        let e = verificar_cofirmas_del_paquete(&p, &ed(), &[0xaa_u8]).unwrap_err();
        assert!(e.contains("desconocida"), "{e}");
    }

    /// Y un campo que falta se NOMBRA: un instrumento que falla dice QUE
    /// fallo, no cuantos (§254).
    #[test]
    fn un_campo_ausente_se_nombra() {
        let mut c = cofirma(DIG, OP);
        let _ = c.as_object_mut().expect("objeto").remove("firma");
        let p = json!({ "v": 2, "cofirmas": [c] });
        let e = verificar_cofirmas_del_paquete(&p, &ed(), &[0xaa_u8]).unwrap_err();
        assert!(e.contains("falta firma"), "{e}");
    }

    /// §406 · RFC-0005 E2: el rechazo por version del mando CONSUME el conjunto de
    /// `VersionCabeza` y conserva el texto que el MANIFIESTO del paquete fija.
    #[test]
    fn el_rechazo_por_version_consume_el_conjunto_y_conserva_su_texto() {
        let dir = std::env::temp_dir();
        let fuera = format!("{:#x}", u64::from(VersionCabeza::TODAS.last().expect("no vacio").as_u8()) + 1);
        for v in ["0x1", fuera.as_str(), "0x103"] {
            let n = u64::from_str_radix(v.trim_start_matches("0x"), 16).unwrap();
            let ruta = dir.join(format!("zk-ssl-verify-406-{n}.json"));
            let p = json!({ "v": 1, "cabeza": { "available": true, "formatVersion": v } });
            std::fs::write(&ruta, p.to_string()).unwrap();
            let e = correr(ruta.to_str().unwrap()).unwrap_err();
            let _ = std::fs::remove_file(&ruta);
            let esperado = format!(
                "formatVersion {n}: el paquete v1 empaqueta cabezas {}",
                VersionCabeza::texto()
            );
            assert!(e.contains(&esperado), "{v}: {e}");
        }
    }
}
