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
    acuses, recibos, verificar_acuse, verificar_acuse_v3, indice_de_firma, verificar_cabeza, verificar_cofirma,
    CabezaFirmada, COFIRMA_V_MAX, ReciboAcuse, VersionCabeza,
};
use zk_ssl_air::{verificar_contra_cabeza, Afirmacion, CabezaEdad};
use winter_math::fields::f64::BaseElement;
use winter_math::FieldElement;
use zk_ssl_air::banda::{verificar as verificar_banda, BandaPublicInputs};
use zk_ssl_medio::hash::sha256;
use zk_ssl_medio::medio::{hoja, verificar_inclusion};
use zk_ssl_medio::nota::{origen_del_medio, verificar_nota, ClaveDeNota};
use zk_ssl_hash::{
    ancla_digest, digest_from_bytes, digest_of_proof, digest_to_bytes, epoch_digest_v2,
    epoch_digest_v3, epoch_digest_v4, epoch_digest_v5, epoch_digest_v6, huella_de_clave,
    params_digest, Digest,
};

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
    digest_from_bytes(&arr).map_err(|e| err(format!("{campo}: {e}")))
}

/// Un `u64` del sobre, **sólo si es canónico** (RFC-0016, D-B; §640): menor
/// que `p`, por `zk_ssl_hash::u64_canonico`, el único productor de la regla.
///
/// ⚠️ Todo `u64` que este mando lee entra en una composición o razona junto a
/// una -`n`, `seq`, `mmrSize`, los contadores, el índice-, y `as_digest`
/// reduce módulo `p`: sin esta lectura, una cabeza firmada con `n = 1440`
/// verificaba igual con `n = 1440 + p`, y el sobre de completitud pasaba de
/// «NO RESUELTA EN LA VENTANA» a «ventana ABIERTA» con la MISMA firma
/// (medido). Ningún productor honesto escribe un `u64` que no quepa: lo que
/// se rechaza aquí no lo emitió nunca un nodo de la casa.
fn u64_de(v: &serde_json::Value, campo: &str) -> Result<u64, String> {
    let s = v
        .get(campo)
        .and_then(|x| x.as_str())
        .ok_or_else(|| err(format!("falta {campo} o no es cadena 0x")))?;
    let h = s.strip_prefix("0x").ok_or_else(|| err(format!("{campo} sin 0x")))?;
    let x = u64::from_str_radix(h, 16).map_err(|e| err(format!("{campo}: {e}")))?;
    zk_ssl_hash::u64_canonico(x).map_err(|e| err(format!("{campo}: {e}")))
}

/// La familia de la cabeza v5 (RFC-0007 D-B; §451): los siete parametros en un digest, la
/// raiz del arbol de meta, las dos marcas de agua y el suministro. Se lee con los mismos
/// lectores que el resto de la cabeza: las cinco claves son obligatorias en una v5 y se
/// nombran al faltar, antes de tocar la firma.
#[derive(Clone, Copy)]
struct FamiliaV5 {
    params_digest: Digest,
    pmeta_root: Digest,
    next_pending: u64,
    next_index: u64,
    total_supply: u64,
}

fn familia_v5(c: &serde_json::Value) -> Result<FamiliaV5, String> {
    Ok(FamiliaV5 {
        params_digest: digest_de(c, "paramsDigest")?,
        pmeta_root: digest_de(c, "pmetaRoot")?,
        next_pending: u64_de(c, "nextPending")?,
        next_index: u64_de(c, "nextIndex")?,
        total_supply: u64_de(c, "totalSupply")?,
    })
}

fn correr(ruta: &str) -> Result<(), String> {
    let crudo = std::fs::read_to_string(ruta).map_err(|e| err(format!("no se puede leer {ruta}: {e}")))?;
    let p: serde_json::Value =
        serde_json::from_str(&crudo).map_err(|e| err(format!("JSON ilegible: {e}")))?;
    verificar_paquete(&p)
}

/// El paquete ya leido, de cualquier forma. Vivia dentro de `correr`; sale a su funcion en el §573
/// para que el sobre de completitud verifique su resolucion por acuse COMO el paquete de posicion
/// que es -una cabeza y su acuse-, con el mismo codigo y no con una copia.
fn verificar_paquete(p: &serde_json::Value) -> Result<(), String> {
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
    // §418 · RFC-0006 E3b (D-12) · EL `tipo` DESCONOCIDO GANA RECHAZO CON NOMBRE.
    //   Esto era un `if` contra "extension": cualquier otro `tipo` caia por el
    //   brazo de POSICION y moria abajo en `falta cabeza`, que nombra OTRA
    //   regla. Un rechazo que nombra otra cosa es peor que ninguno, y "seguir
    //   por compatibilidad" es justo lo que la ley prohibe (fail-closed).
    //   El `tipo` AUSENTE sigue siendo el paquete de posicion (v1 y v2): ese es
    //   el contrato de §289 y §322 y NO se toca.
    match p.get("tipo").map(|t| t.as_str().unwrap_or("(no es una cadena)")) {
        None => {}
        Some("extension") => return verificar_extension(&p),
        Some("consumo") => return verificar_consumo(&p),
        Some("conflicto") => return verificar_conflicto(&p),
        // RFC-0007 E3a (§455): la causa de un rechazo, probada sobre el estado comprometido.
        Some("rechazo") => return verificar_rechazo(&p),
        // RFC-0007 E4b-2 (S465): la prueba de edad contra una cabeza v5.
        Some("edad") => return verificar_edad(&p),
        // RFC-0008 E1 (S495): el cobro pendiente contra una cabeza v5.
        Some("cobro_pendiente") => return verificar_cobro_pendiente(&p),
        // RFC-0008 E2 (S506): el pago en curso contra una cabeza v5, la otra mitad.
        Some("pago_en_curso") => return verificar_pago_en_curso(&p),
        // RFC-0008 E3 (S520): la prenda, la unica de la familia que prueba AUTORIZACION.
        Some("prenda") => return verificar_prenda(&p),
        // RFC-0010 E4 (§573): la completitud de un recibo de recepcion, y sus tres veredictos.
        Some("completitud") => return verificar_completitud(&p),
        // RFC-0012 E3 (§586): el ancla de cabezas, sus tres modos y la vista dividida.
        Some("ancla") => return verificar_ancla(&p),
        // RFC-0013 E4a (§633): el ancla publicada en el medio y cofirmada por testigos ajenos.
        Some("ancla-cofirmada") => return verificar_ancla_cofirmada(&p),
        Some(otro) => {
            return Err(err(format!(
                "tipo desconocido: {otro} - se lee un paquete de posicion (sin `tipo`), \
                 `tipo: \"extension\"`, `tipo: \"consumo\"`, `tipo: \"conflicto\"`, \
                 `tipo: \"rechazo\"`, `tipo: \"edad\"`, `tipo: \"cobro_pendiente\"`, \
                 `tipo: \"pago_en_curso\"`, `tipo: \"prenda\"`, `tipo: \"completitud\"`, \
                 `tipo: \"ancla\"` o `tipo: \"ancla-cofirmada\"`"
            )))
        }
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
        VersionCabeza::V3 | VersionCabeza::V4 | VersionCabeza::V5 | VersionCabeza::V6 => {
            Some((digest_de(c, "mmrRoot")?, u64_de(c, "mmrSize")?))
        }
    };
    let cons = match version {
        VersionCabeza::V2 | VersionCabeza::V3 => None,
        VersionCabeza::V4 | VersionCabeza::V5 | VersionCabeza::V6 => {
            Some((digest_de(c, "consRoot")?, u64_de(c, "consCount")?))
        }
    };
    // RFC-0007 E1a (§451): la familia de v5, que el compilador exige en cada `match`
    // igual que exigio la pareja de consumos en el §414.
    let estado = match version {
        VersionCabeza::V2 | VersionCabeza::V3 | VersionCabeza::V4 => None,
        VersionCabeza::V5 | VersionCabeza::V6 => Some(familia_v5(c)?),
    };
    // RFC-0010 E2 (§558): la pareja de recepcion, con el molde de la de consumos.
    let recep = match version {
        VersionCabeza::V2 | VersionCabeza::V3 | VersionCabeza::V4 | VersionCabeza::V5 => None,
        VersionCabeza::V6 => Some((digest_de(c, "recepRoot")?, u64_de(c, "recepCount")?)),
    };
    let compuesto = match (mmr, cons, estado, recep) {
        (None, _, _, _) => epoch_digest_v2(seq, accounts, pending, frozen, chain, acuses_root, n),
        (Some((cima, t)), None, _, _) => {
            epoch_digest_v3(seq, accounts, pending, frozen, chain, acuses_root, n, cima, t)
        }
        (Some((cima, t)), Some((raiz, k)), None, _) => epoch_digest_v4(
            seq, accounts, pending, frozen, chain, acuses_root, n, cima, t, raiz, k,
        ),
        (Some((cima, t)), Some((raiz, k)), Some(f), None) => epoch_digest_v5(
            seq, accounts, pending, frozen, chain, acuses_root, n, cima, t, raiz, k,
            f.params_digest, f.pmeta_root, f.next_pending, f.next_index, f.total_supply,
        ),
        (Some((cima, t)), Some((raiz, k)), Some(f), Some((rr, rc))) => epoch_digest_v6(
            seq, accounts, pending, frozen, chain, acuses_root, n, cima, t, raiz, k,
            f.params_digest, f.pmeta_root, f.next_pending, f.next_index, f.total_supply,
            rr, rc,
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
                    digest_from_bytes(&arr).map_err(|e| err(format!("sibling {i}: {e}")))
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
            acuse_contra_cabeza(&recibo, mmr, cons, estado, recep, epoch_digest)?;
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
/// 3/3 · el acuse contra la cabeza empaquetada: la VERSION elige recomponedor
/// con el MISMO `match` exhaustivo de CUATRO piezas que el paso 1 (§566).
///
/// ⚠️ Hasta el §566 este brazo casaba TRES piezas -`(mmr, cons, estado)`- y
/// dejaba fuera la pareja de recepcion, asi que una cabeza v6 caia en el brazo
/// de la v5: `CabezaDistinta` sobre una cabeza legitima. El paso 1 ya casaba
/// cuatro desde el §558; el paso 3 se quedo atras y ningun compilador podia
/// verlo, porque el brazo de tres cubria la v6 sin nombrarla. Con cuatro, el
/// brazo que falte lo marca el compilador.
fn acuse_contra_cabeza(
    recibo: &ReciboAcuse,
    mmr: Option<(Digest, u64)>,
    cons: Option<(Digest, u64)>,
    estado: Option<FamiliaV5>,
    recep: Option<(Digest, u64)>,
    epoch_digest: Digest,
) -> Result<(), String> {
    match (mmr, cons, estado, recep) {
        (None, _, _, _) => verificar_acuse(recibo, epoch_digest),
        (Some((cima, t)), None, _, _) => verificar_acuse_v3(recibo, cima, t, epoch_digest),
        (Some((cima, t)), Some((raiz, k)), None, _) => {
            zk_ssl_verify::verificar_acuse_v4(recibo, cima, t, raiz, k, epoch_digest)
        }
        (Some((cima, t)), Some((raiz, k)), Some(f), None) => zk_ssl_verify::verificar_acuse_v5(
            recibo, cima, t, raiz, k, f.params_digest, f.pmeta_root, f.next_pending,
            f.next_index, f.total_supply, epoch_digest,
        ),
        (Some((cima, t)), Some((raiz, k)), Some(f), Some((rr, rc))) => {
            zk_ssl_verify::verificar_acuse_v6(
                recibo, cima, t, raiz, k, f.params_digest, f.pmeta_root, f.next_pending,
                f.next_index, f.total_supply, rr, rc, epoch_digest,
            )
        }
    }
    .map_err(|e| err(format!("acuse: {e:?}")))
}

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
fn cabeza_v3_verificada(
    c: &serde_json::Value,
    cual: &str,
) -> Result<(Digest, u64, String, Option<Digest>), String> {
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
        Ok(VersionCabeza::V5) => {
            let f = familia_v5(c)?;
            epoch_digest_v5(
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
                f.params_digest,
                f.pmeta_root,
                f.next_pending,
                f.next_index,
                f.total_supply,
            )
        }
        Ok(VersionCabeza::V6) => {
            let f = familia_v5(c)?;
            epoch_digest_v6(
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
                f.params_digest,
                f.pmeta_root,
                f.next_pending,
                f.next_index,
                f.total_supply,
                digest_de(c, "recepRoot")?,
                u64_de(c, "recepCount")?,
            )
        }
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
    // El `consRoot` existe desde la v4 (`lleva_consumos`, el productor unico; RFC-0007 E1a,
    // §451): una v3 no lo lleva y el sobre de consumo la rechaza por su cuenta, con su texto.
    let cons = if VersionCabeza::try_from(version).map_or(false, VersionCabeza::lleva_consumos) {
        Some(digest_de(c, "consRoot")?)
    } else {
        None
    };
    Ok((
        digest_de(c, "mmrRoot")?,
        u64_de(c, "mmrSize")?,
        clave.to_string(),
        cons,
    ))
}

/// El paquete de EXTENSION (§293): dos cabezas v3 firmadas y el camino
/// de consistencia entre sus cimas — el eslabon 2 entero, verificable
/// **sin el nodo**: quien custodia la vieja comprueba que la nueva la
/// EXTIENDE, con el objeto de §291 como juez.
fn verificar_extension(p: &serde_json::Value) -> Result<(), String> {
    let (cima_v, t_v, clave_v, _) = cabeza_v3_verificada(
        p.get("vieja").ok_or_else(|| err("falta vieja".into()))?,
        "vieja",
    )?;
    let (cima_n, t_n, clave_n, _) = cabeza_v3_verificada(
        p.get("nueva").ok_or_else(|| err("falta nueva".into()))?,
        "nueva",
    )?;
    println!("1/3 las DOS cabezas v3 recomponen su digest y sus firmas verifican");
    if clave_v != clave_n {
        return Err(claves_distintas());
    }
    println!("2/3 misma publicKey: el mismo firmante en los dos extremos");
    let camino = camino_mmr(p)?;
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

/// El paquete de CONSUMO (§419, RFC-0006 E3b): dos cabezas **v4**
/// firmadas, la consistencia entre sus cimas y los dos caminos del
/// consumo — que ESTA bajo la nueva y que NO estaba bajo la vieja.
///
/// El sobre es un superconjunto estricto del de extension: mismas dos
/// cabezas, misma `publicKey`, mismo juez de consistencia. Lo que anade
/// son los caminos, y **la posicion se DERIVA aqui y se CRUZA** contra el
/// `isRight` recibido: sin ese cruce la mitad de AUSENCIA es falsificable.
fn verificar_consumo(p: &serde_json::Value) -> Result<(), String> {
    let (cima_v, t_v, clave_v, cons_v) = cabeza_v3_verificada(
        p.get("vieja").ok_or_else(|| err("falta vieja".into()))?,
        "vieja",
    )?;
    let (cima_n, t_n, clave_n, cons_n) = cabeza_v3_verificada(
        p.get("nueva").ok_or_else(|| err("falta nueva".into()))?,
        "nueva",
    )?;
    println!("1/5 las DOS cabezas recomponen su digest y sus firmas verifican");
    if clave_v != clave_n {
        return Err(claves_distintas());
    }
    let (raiz_v, raiz_n) = match (cons_v, cons_n) {
        (Some(v), Some(n)) => (v, n),
        _ => return Err(exige_consumos("consumo")),
    };
    println!(
        "2/5 misma publicKey y las dos cabezas llevan consRoot ({}) a los dos lados",
        VersionCabeza::texto_con_consumos()
    );
    if !zk_ssl_verify::mmr::verificar_consistencia(cima_v, t_v, cima_n, t_n, &camino_mmr(p)?) {
        return Err(err(format!(
            "la nueva (t={t_n}) NO extiende a la vieja (t={t_v}): historia \
             bifurcada, recortada, o camino que no es el suyo"
        )));
    }
    println!("3/5 la cima nueva EXTIENDE a la vieja: el «antes» es de esta historia");

    let consumo = digest_de(p, "consumo")?;
    let pos = zk_ssl_verify::consumos::posicion_de_consumo(&consumo);
    let (herm_n, der_n) = camino_de(p, "presencia", "presencia")?;
    let (herm_v, der_v) = camino_de(p, "ausencia", "ausencia")?;
    for (cual, der) in [("presencia", &der_n), ("ausencia", &der_v)] {
        if !zk_ssl_verify::consumos::cruza_posicion(pos, der) {
            return Err(cruce_fallado(cual, pos));
        }
    }
    println!("4/5 los dos caminos son los de la posicion {pos}, DERIVADA del consumo");

    match zk_ssl_verify::consumos::raiz_de_presencia(consumo, &herm_n, &der_n) {
        Some(r) if r == raiz_n => {}
        Some(_) => return Err(err("presencia: el camino NO sube al consRoot de la nueva".into())),
        None => return Err(err(camino_descuadrado("presencia"))),
    }
    match zk_ssl_verify::consumos::raiz_de_ausencia(&herm_v, &der_v) {
        Some(r) if r == raiz_v => {}
        Some(_) => {
            return Err(err(
                "ausencia: la hoja vacia NO sube al consRoot de la vieja - el consumo YA estaba"
                    .into(),
            ))
        }
        None => return Err(err(camino_descuadrado("ausencia"))),
    }
    println!("5/5 el consumo ESTA bajo la nueva y NO estaba bajo la vieja");
    println!("VERDE: el consumo se publico entre las dos cabezas, sin el nodo");
    Ok(())
}

fn camino_descuadrado(cual: &str) -> String {
    err(format!(
        "{cual}: el camino no tiene los {} niveles del arbol de consumos",
        zk_ssl_verify::consumos::CONS_DEPTH
    ))
}

/// UN productor del texto de las claves distintas. Vivia DUPLICADO byte a byte
/// en `verificar_extension` y en `verificar_consumo` -medido en la sesion 108-,
/// y el catalogo de `spec/PAQUETE.md` seccion 5 lo declara UNA sola vez: dos
/// productores del mismo contrato, y el documento contando uno. Aqui queda uno.
fn claves_distintas() -> String {
    err("las cabezas llevan claves DISTINTAS: la continuidad es de UN firmante".into())
}

/// UN productor del texto de la version del sobre, con su SUJETO como hueco y el
/// conjunto DERIVADO de `VersionCabeza::texto_con_consumos()` (RFC-0007 E1a, §451):
/// <<v4, v5 o v6>>. Los fragmentos que los manifiestos pinan (`exige cabezas v4`) siguen
/// dentro del texto, byte a byte, asi que ningun vector del catalogo se mueve: lo
/// gatea el arnes en cada canon.
fn exige_consumos(cual: &str) -> String {
    err(format!(
        "el sobre de {cual} exige cabezas {}: una v2 o v3 no lleva consRoot contra el \
         que comprobar",
        VersionCabeza::texto_con_consumos()
    ))
}


/// El camino de consistencia del MMR: lista PLANA de digests, como en el
/// sobre de extension.
fn camino_mmr(p: &serde_json::Value) -> Result<Vec<Digest>, String> {
    let cam = p
        .get("camino")
        .and_then(|x| x.as_array())
        .ok_or_else(|| err("falta camino (lista de digests)".into()))?;
    cam.iter()
        .enumerate()
        .map(|(i, s)| {
            let s = s.as_str().ok_or_else(|| err(format!("camino[{i}] no es cadena")))?;
            let bts = hex_a_bytes(s)?;
            let arr: [u8; 32] = bts
                .as_slice()
                .try_into()
                .map_err(|_| err(format!("camino[{i}]: {} bytes", bts.len())))?;
            digest_from_bytes(&arr).map_err(|e| err(format!("camino[{i}]: {e}")))
        })
        .collect()
}

/// Un camino del arbol de consumos: `{siblings, isRight}`, la respuesta de
/// `zkssl_consumoPath` TAL CUAL.
fn camino_de(
    p: &serde_json::Value,
    clave: &str,
    mote: &str,
) -> Result<(Vec<Digest>, Vec<bool>), String> {
    let c = p
        .get(clave)
        .ok_or_else(|| err(format!("falta {mote} (camino del consumo)")))?;
    let sib = c
        .get("siblings")
        .and_then(|x| x.as_array())
        .ok_or_else(|| err(format!("{mote}: falta siblings")))?;
    let der = c
        .get("isRight")
        .and_then(|x| x.as_array())
        .ok_or_else(|| err(format!("{mote}: falta isRight")))?;
    let hermanos = sib
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let s = s
                .as_str()
                .ok_or_else(|| err(format!("{mote}: siblings[{i}] no es cadena")))?;
            let bts = hex_a_bytes(s)?;
            let arr: [u8; 32] = bts
                .as_slice()
                .try_into()
                .map_err(|_| err(format!("{mote}: siblings[{i}]: {} bytes", bts.len())))?;
            digest_from_bytes(&arr).map_err(|e| err(format!("{mote}: siblings[{i}]: {e}")))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let derecha = der
        .iter()
        .enumerate()
        .map(|(i, b)| {
            b.as_bool()
                .ok_or_else(|| err(format!("{mote}: isRight[{i}] no es booleano")))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok((hermanos, derecha))
}

/// UN productor del texto del cruce de posicion, con su SUJETO como hueco. Con
/// `cual` = "presencia" o "ausencia" emite la MISMA cadena que hasta hoy, byte a
/// byte, asi que ningun vector del catalogo se mueve; lo gatea el arnes en cada
/// canon. Vivia INLINE en `verificar_consumo`, y el conflicto necesita nombrar
/// un sujeto distinto: dos sitios con el mismo texto serian dos productores.
fn cruce_fallado(cual: &str, pos: u64) -> String {
    err(format!(
        "{cual}: el isRight recibido NO es el de la posicion {pos} que el consumo \
         DERIVA - un camino de otra posicion no prueba nada de este consumo"
    ))
}

/// **El sobre de CONFLICTO** (RFC-0006, E4a): el MISMO consumo bajo el `consRoot`
/// de DOS cabezas firmadas por operadores DISTINTOS.
///
/// NO es una variante del sobre de consumo. Aquel prueba que un consumo se
/// publico ENTRE dos cabezas de UN firmante, y para eso necesita la consistencia
/// del MMR y la `ausencia`. Entre DOS libros no hay historia comun que extender,
/// asi que aqui no hay `camino` ni `ausencia`: hay una lista `libros` de
/// exactamente dos, SIN orden, porque la prueba no lo tiene.
///
/// Y la regla de las claves va AL REVES que en las otras dos formas: alli se
/// exige la MISMA `publicKey` -la continuidad es de un firmante-; aqui se exige
/// que sean DISTINTAS, o dos cabezas del mismo operador pasarian por conflicto.
///
/// Lo que demuestra es DETECCION y llega despues. No previene nada -prevenir
/// seria ORDENAR entre libros, y nadie ordena- ni dice que la unidad consumida
/// sea la misma a los dos lados: eso es gobernanza (RFC-0006, D-4).
fn verificar_conflicto(p: &serde_json::Value) -> Result<(), String> {
    let libros = p
        .get("libros")
        .ok_or_else(|| err("falta libros".into()))?
        .as_array()
        .ok_or_else(|| err("libros no es una lista".into()))?;
    if libros.len() != 2 {
        return Err(err(format!(
            "el sobre de conflicto exige DOS libros: se recibieron {}",
            libros.len()
        )));
    }
    let consumo = digest_de(p, "consumo")?;
    let pos = zk_ssl_verify::consumos::posicion_de_consumo(&consumo);
    let mut claves: Vec<String> = Vec::new();
    let mut raices: Vec<Digest> = Vec::new();
    for (i, libro) in libros.iter().enumerate() {
        let cual = format!("libro[{i}]");
        let c = libro
            .get("cabeza")
            .ok_or_else(|| err(format!("{cual}: falta cabeza")))?;
        let (_, _, clave, cons) = cabeza_v3_verificada(c, &cual)?;
        claves.push(clave);
        raices.push(cons.ok_or_else(|| exige_consumos("conflicto"))?);
    }
    println!("1/4 las DOS cabezas recomponen su digest y sus firmas verifican");
    if claves[0] == claves[1] {
        return Err(err(
            "las cabezas llevan la MISMA clave: un conflicto es entre DOS firmantes".into(),
        ));
    }
    println!(
        "2/4 las dos cabezas son de operadores DISTINTOS, y las dos llevan consRoot ({})",
        VersionCabeza::texto_con_consumos()
    );
    for (i, libro) in libros.iter().enumerate() {
        let cual = format!("libro[{i}]");
        let (herm, der) = camino_de(libro, "presencia", &cual)?;
        if !zk_ssl_verify::consumos::cruza_posicion(pos, &der) {
            return Err(cruce_fallado(&cual, pos));
        }
        match zk_ssl_verify::consumos::raiz_de_presencia(consumo, &herm, &der) {
            Some(r) if r == raices[i] => {}
            Some(_) => {
                return Err(err(format!(
                    "{cual}: el camino NO sube al consRoot de su cabeza"
                )))
            }
            None => return Err(err(camino_descuadrado(&cual))),
        }
    }
    println!("3/4 los dos caminos son los de la posicion {pos}, DERIVADA del consumo");
    println!("4/4 el MISMO consumo esta bajo el consRoot de los DOS libros");
    println!("VERDE: dos libros aceptaron el mismo consumo. Es DETECCION, no prevencion:");
    println!("       nadie ordena entre libros, y que la unidad sea la misma es gobernanza");
    Ok(())
}

/// **El sobre de RECHAZO** (RFC-0007 E3a, §455): la causa que el nodo dio al rechazar, y el
/// material que la PRUEBA sobre el estado que una cabeza firmada compromete, sin el nodo.
///
/// Todo viaja TAL CUAL lo sirvio el cable -reunir, no recomponer-: `data` es el objeto `data` del
/// rechazo (`spec/RPC.md`, §454), `{causa, campos, seq}`; `cabeza`, una respuesta de
/// `zkssl_signedEpochHead`; `parametros`, la de `zkssl_params`; `presencia`, el camino de
/// `zkssl_consumoPath`; `congelados`, la respuesta de `zkssl_frozenPath` (§459); `cuenta`, el
/// camino del arbol de cuentas que el propio rechazo trae (§475, SIN metodo del cable);
/// `peticion`, los
/// `params` de la emision rechazada tal cual los envio el solicitante (§460). QUE cabeza
/// sirve depende de la causa y se exige con el `seq`: una
/// ANTERIOR al rechazo, o la misma, para lo que solo crece -`nextIndex`, los consumos-; cualquiera
/// del libro para lo que no tiene setter -el limite regulatorio-. El veredicto es de la CAUSA:
/// VERDE si se sostiene sobre el estado comprometido; si no, ROJO nombrando por que, y entonces el
/// sobre es la prueba de que la regla fue un disfraz.
fn verificar_rechazo(p: &serde_json::Value) -> Result<(), String> {
    let d = p
        .get("data")
        .ok_or_else(|| err("falta data (el objeto del rechazo)".into()))?;
    let causa = d
        .get("causa")
        .and_then(|x| x.as_str())
        .ok_or_else(|| err("data: falta causa".into()))?;
    let campos = d
        .get("campos")
        .ok_or_else(|| err("data: falta campos".into()))?;
    let s_rechazo = u64_de(d, "seq")?;
    let c = p.get("cabeza").ok_or_else(|| err("falta cabeza".into()))?;
    let (_, _, _, cons) = cabeza_v3_verificada(c, "cabeza")?;
    let s_cabeza = u64_de(c, "seq")?;
    println!("1/3 la cabeza recompone su digest y su firma verifica (seq {s_cabeza})");
    match causa {
        "OverRegulatoryLimit" => {
            let (limite, _, _) = parametros_comprometidos(p, c, causa)?;
            println!("2/3 los parametros recomponen el paramsDigest: el limite es {limite}");
            let dicho = u64_de(campos, "limit")?;
            let pedido = u64_de(campos, "requested")?;
            if dicho != limite {
                return Err(no_es_el_comprometido("limite", dicho, limite));
            }
            if pedido <= limite {
                return Err(err(format!(
                    "la causa NO se sostiene: el importe pedido ({pedido}) no supera el \
                     limite ({limite})"
                )));
            }
            println!("3/3 el importe pedido ({pedido}) supera el limite comprometido ({limite})");
        }
        "AccountLimitReached" => {
            let (_, tope, _) = parametros_comprometidos(p, c, causa)?;
            let n = familia_v5(c)?.next_index;
            println!("2/3 los parametros recomponen el paramsDigest: el tope de cuentas es {tope}");
            let dicho = u64_de(campos, "limit")?;
            if dicho != tope {
                return Err(no_es_el_comprometido("tope", dicho, tope));
            }
            exige_anterior(s_cabeza, s_rechazo, "nextIndex solo sube")?;
            if n < tope {
                return Err(err(format!(
                    "la causa NO se sostiene: nextIndex ({n}) no alcanza el tope de cuentas \
                     ({tope})"
                )));
            }
            println!("3/3 una cabeza anterior al rechazo ya tenia nextIndex {n}, el tope");
        }
        "ConsumoRepetido" | "ConsumoColision" => {
            let raiz = cons.ok_or_else(|| exige_consumos("rechazo"))?;
            let consumo = digest_de(campos, "consumo")?;
            let pos = zk_ssl_verify::consumos::posicion_de_consumo(&consumo);
            // lo que tiene que estar YA en la posicion: el mismo consumo, o su ocupante
            let presente = if causa == "ConsumoRepetido" {
                consumo
            } else {
                let o = digest_de(campos, "ocupante")?;
                if o == consumo {
                    return Err(err("la causa NO se sostiene: ocupante y consumo son el MISMO \
                                    - eso seria ConsumoRepetido"
                        .into()));
                }
                let po = zk_ssl_verify::consumos::posicion_de_consumo(&o);
                if po != pos {
                    return Err(err(format!(
                        "la causa NO se sostiene: el ocupante vive en la posicion {po}, no en \
                         la {pos} del consumo"
                    )));
                }
                o
            };
            let (herm, der) = camino_de(p, "presencia", "presencia")?;
            if !zk_ssl_verify::consumos::cruza_posicion(pos, &der) {
                return Err(cruce_fallado("presencia", pos));
            }
            println!("2/3 el camino es el de la posicion {pos}, DERIVADA del consumo");
            match zk_ssl_verify::consumos::raiz_de_presencia(presente, &herm, &der) {
                Some(r) if r == raiz => {}
                Some(_) => {
                    return Err(err(
                        "presencia: el camino NO sube al consRoot de la cabeza".into()
                    ))
                }
                None => return Err(err(camino_descuadrado("presencia"))),
            }
            exige_anterior(s_cabeza, s_rechazo, "un consumo publicado no se quita")?;
            println!("3/3 la posicion ya estaba ocupada bajo una cabeza anterior al rechazo");
        }
        "StaleState" => {
            // §454: el mensaje de una StaleState puede llevar `[receptionSeq=..]`, pero
            // la CAUSA no tiene campos: su prueba son las tres raices que el recibo declaro,
            // contra las de la cabeza. Basta con que UNA no sea la comprometida.
            let recibo = p
                .get("recibo")
                .ok_or_else(|| err("falta recibo (los publicInputs del rechazado)".into()))?;
            if !campos.as_object().map_or(false, |o| o.is_empty()) {
                return Err(err("StaleState no lleva campos: su material es el recibo".into()));
            }
            exige_misma(s_cabeza, s_rechazo)?;
            let root_new = digest_de(c, "accountsRoot")?;
            let pend_new = digest_de(c, "pendingRoot")?;
            let froz = digest_de(c, "frozenRoot")?;
            let r_rec = digest_de(recibo, "rootOld")?;
            let p_rec = digest_de(recibo, "pendingRootOld")?;
            let f_rec = digest_de(recibo, "frozenRoot")?;
            println!("2/3 el recibo declaro rootOld, pendingRootOld y frozenRoot");
            let cual = if r_rec != root_new {
                "accountsRoot"
            } else if p_rec != pend_new {
                "pendingRoot"
            } else if f_rec != froz {
                "frozenRoot"
            } else {
                return Err(err(
                    "la causa NO se sostiene: las tres raices del recibo son las de la cabeza \
                     - ese estado NO estaba atras"
                        .into(),
                ));
            };
            println!("3/3 el recibo se probo contra otro {cual}: su estado quedo atras");
        }
        "WrongRegulatoryLimit" => {
            // El mismo modelo que OverRegulatoryLimit: los parametros recomponen, y lo que
            // el nodo dice haber recibido (`declared`) NO es el comprometido (`expected`). Lo
            // que el cliente puso en su recibo es palabra del nodo, como el `requested`; su
            // refutacion es el recibo que el propio cliente guarda (spec/PAQUETE.md 2.6).
            let (limite, _, _) = parametros_comprometidos(p, c, causa)?;
            println!("2/3 los parametros recomponen el paramsDigest: el limite es {limite}");
            let esperado = u64_de(campos, "expected")?;
            let declarado = u64_de(campos, "declared")?;
            if esperado != limite {
                return Err(no_es_el_comprometido("limite esperado", esperado, limite));
            }
            if declarado == limite {
                return Err(err(format!(
                    "la causa NO se sostiene: el limite declarado ({declarado}) ES el \
                     comprometido ({limite})"
                )));
            }
            println!("3/3 el limite declarado ({declarado}) no es el comprometido ({limite})");
        }
        "DuplicateAccountInBatch" | "DuplicatePendingInBatch" => {
            // El lote viaja TAL CUAL (`ops`, como en zkssl_applyMany); el mando aplica la
            // MISMA regla y en el MISMO orden que `apply_many`: una cuenta por operacion y
            // posiciones distintas, y el PRIMER choque tiene que ser el que el nodo nombro.
            let ops = p
                .get("lote")
                .and_then(|x| x.as_array())
                .ok_or_else(|| err("falta lote (las ops de applyMany)".into()))?;
            exige_misma(s_cabeza, s_rechazo)?;
            let (mut cuentas, mut posiciones) = (Vec::new(), Vec::new());
            let mut choque: Option<(&str, u64)> = None;
            for (i, op) in ops.iter().enumerate() {
                let (cuenta, pos) = op_cuenta_y_posicion(op, i)?;
                if cuentas.contains(&cuenta) {
                    choque = Some(("DuplicateAccountInBatch", cuenta));
                    break;
                }
                cuentas.push(cuenta);
                if posiciones.contains(&pos) {
                    choque = Some(("DuplicatePendingInBatch", pos));
                    break;
                }
                posiciones.push(pos);
            }
            println!("2/3 el lote lleva {} operaciones, leidas en orden", ops.len());
            match choque {
                Some((n, v)) if n == causa => {
                    let clave =
                        if causa == "DuplicateAccountInBatch" { "index" } else { "position" };
                    let dicho = u64_de(campos, clave)?;
                    if dicho != v {
                        return Err(err(format!(
                            "data: el {clave} que el nodo dice ({dicho}) no es el del primer \
                             choque del lote ({v})"
                        )));
                    }
                    println!("3/3 el primer choque del lote es {causa} en {clave} {v}");
                }
                Some((n, _)) => {
                    return Err(err(format!(
                        "la causa NO se sostiene: el primer choque del lote es {n}, no {causa}"
                    )))
                }
                None => {
                    return Err(err(
                        "la causa NO se sostiene: el lote no tiene cuentas ni posiciones \
                         repetidas"
                            .into(),
                    ))
                }
            }
        }
        "AccountFrozen" => {
            // RFC-0007 E3b (§459): la hoja de la cuenta bajo el `frozenRoot` de la cabeza del
            // `seq` EXACTO -las congelaciones van y vuelven-, con las reglas del modulo
            // `congelados` del nucleo (§458): profundidad FIJA, cruce con el indice y hoja no
            // vacia. El material es la respuesta de `zkssl_frozenPath` TAL CUAL (`index`,
            // `leaf`, `camino`); su `s` no va bajo firma y no se mira: se juzga contra la raiz.
            let g = p
                .get("congelados")
                .ok_or_else(|| err("falta congelados (el camino de zkssl_frozenPath)".into()))?;
            exige_misma(s_cabeza, s_rechazo)?;
            let dicho = u64_de(campos, "index")?;
            let del_camino = u64_de(g, "index")?;
            if dicho != del_camino {
                return Err(err(format!(
                    "data: el index que el nodo dice ({dicho}) no es el del camino ({del_camino})"
                )));
            }
            let hoja = digest_de(g, "leaf")?;
            if g.get("camino").is_none() {
                return Err(err("congelados: falta camino".into()));
            }
            let (herm, der) = camino_de(g, "camino", "congelados")?;
            if !zk_ssl_verify::congelados::cruza_indice(dicho, &der) {
                return Err(err(format!(
                    "congelados: el isRight recibido NO es el de la cuenta {dicho} - un camino \
                     de otra cuenta no prueba nada de esta"
                )));
            }
            println!("2/3 el camino es el de la cuenta {dicho}, la que el nodo nombro");
            let raiz = digest_de(c, "frozenRoot")?;
            match zk_ssl_verify::congelados::raiz_de_hoja(hoja, &herm, &der) {
                Some(r) if r == raiz => {}
                Some(_) => {
                    return Err(err(
                        "congelados: el camino NO sube al frozenRoot de la cabeza".into()
                    ))
                }
                None => {
                    return Err(err(format!(
                        "congelados: el camino no tiene los {} niveles del arbol de congelados",
                        zk_ssl_verify::congelados::FROZEN_DEPTH
                    )))
                }
            }
            if !zk_ssl_verify::congelados::esta_congelada(hoja) {
                return Err(err(format!(
                    "la causa NO se sostiene: la hoja de la cuenta {dicho} bajo el frozenRoot es \
                     la vacia - no estaba congelada"
                )));
            }
            println!("3/3 la hoja de la cuenta {dicho} bajo el frozenRoot de la cabeza no es la vacia");
        }
        "AccountNotFound" => {
            // RFC-0007 E5, corte 3b (§475): la hoja VACIA de la cuenta bajo el
            // `accountsRoot` de la cabeza del `seq` EXACTO, con las reglas del modulo `cuentas`
            // (§475): profundidad FIJA, cruce con el indice y hoja vacia. El material es el
            // bloque `cuenta` que el propio rechazo trae -`index`, `leaf`, `camino`-, y NO sale
            // de ningun metodo del cable: solo lo recibe quien hizo la peticion (D-G). Su `s` no
            // va bajo firma y no se mira: se juzga contra la raiz.
            // ⚠️ Una cabeza POSTERIOR tambien probaria la ausencia -el conjunto de cuentas solo
            // crece-, pero eso es otra regla y pide su testigo: aqui se exige la MISMA.
            let g = p
                .get("cuenta")
                .ok_or_else(|| err("falta cuenta (el camino del arbol de cuentas)".into()))?;
            exige_misma(s_cabeza, s_rechazo)?;
            let dicho = u64_de(campos, "index")?;
            let del_camino = u64_de(g, "index")?;
            if dicho != del_camino {
                return Err(err(format!(
                    "data: el index que el nodo dice ({dicho}) no es el del camino ({del_camino})"
                )));
            }
            let hoja = digest_de(g, "leaf")?;
            if g.get("camino").is_none() {
                return Err(err("cuenta: falta camino".into()));
            }
            let (herm, der) = camino_de(g, "camino", "cuenta")?;
            if !zk_ssl_verify::cuentas::cruza_indice(dicho, &der) {
                return Err(err(format!(
                    "cuenta: el isRight recibido NO es el de la cuenta {dicho} - un camino de \
                     otra cuenta no prueba nada de esta"
                )));
            }
            println!("2/3 el camino es el de la cuenta {dicho}, la que el nodo nombro");
            let raiz = digest_de(c, "accountsRoot")?;
            match zk_ssl_verify::cuentas::raiz_de_hoja(hoja, &herm, &der) {
                Some(r) if r == raiz => {}
                Some(_) => {
                    return Err(err(
                        "cuenta: el camino NO sube al accountsRoot de la cabeza".into()
                    ))
                }
                None => {
                    return Err(err(format!(
                        "cuenta: el camino no tiene los {} niveles del arbol de cuentas",
                        zk_ssl_verify::cuentas::ACCOUNTS_DEPTH
                    )))
                }
            }
            if !zk_ssl_verify::cuentas::no_existe(hoja) {
                return Err(err(format!(
                    "la causa NO se sostiene: la hoja de la cuenta {dicho} bajo el accountsRoot \
                     NO es la vacia - la cuenta existe"
                )));
            }
            println!("3/3 la hoja de la cuenta {dicho} bajo el accountsRoot de la cabeza es la VACIA");
        }
        "SupplyCapExceeded" => {
            // RFC-0007 E3 (§460): el tope de suministro. Los parametros recomponen el
            // `paramsDigest` y dan el `maxSupply` comprometido; la cabeza es la del `seq` EXACTO
            // -el suministro sube con cada emision y baja con cada quema-; y el importe NO es
            // palabra del nodo: viaja en `peticion`, los `params` de la emision rechazada TAL
            // CUAL los envio el solicitante. `wouldBe` tiene que ser el suministro de la cabeza
            // mas ese importe, con la suma saturada de la capa (`mint.rs`), y pasar el tope.
            let (_, _, tope) = parametros_comprometidos(p, c, causa)?;
            println!(
                "2/3 los parametros recomponen el paramsDigest: el tope de suministro es {tope}"
            );
            let dicho = u64_de(campos, "cap")?;
            if dicho != tope {
                return Err(no_es_el_comprometido("tope de suministro", dicho, tope));
            }
            exige_misma(s_cabeza, s_rechazo)?;
            let pet = p
                .get("peticion")
                .ok_or_else(|| err("falta peticion (los params de la emision rechazada)".into()))?;
            let importe = u64_de(pet, "amount")?;
            let suministro = familia_v5(c)?.total_supply;
            let seria = suministro.saturating_add(importe);
            let dice = u64_de(campos, "wouldBe")?;
            if dice != seria {
                return Err(err(format!(
                    "data: el wouldBe que el nodo dice ({dice}) no es el suministro de la cabeza \
                     mas el importe pedido ({suministro} + {importe} = {seria})"
                )));
            }
            if seria <= tope {
                return Err(err(format!(
                    "la causa NO se sostiene: el suministro resultante ({seria}) no supera el \
                     tope ({tope})"
                )));
            }
            println!(
                "3/3 el suministro de la cabeza ({suministro}) mas el importe pedido ({importe}) \
                 es {seria}, y pasa el tope comprometido ({tope})"
            );
        }
        "InsufficientBalance" => {
            // RFC-0007 E5, corte 4b: la BANDA del saldo. Es el primer brazo de este mando que
            // verifica un STARK; la regla vive en `zk_ssl_air::banda::verificar`, un solo
            // productor en el crate que el tercero compila, y aqui solo se le pasa lo que la
            // cabeza FIRMA. El `data` no trae el saldo y este brazo lo RECHAZA si aparece; la
            // prueba tampoco lo lleva en claro desde el §538 (hasta entonces si: §521).
            if campos.get("available").is_some() {
                return Err(err(
                    "data: esta causa no publica el saldo: la banda lo prueba sin el".into(),
                ));
            }
            let g = p
                .get("banda")
                .ok_or_else(|| err("falta banda (la prueba de la desigualdad)".into()))?;
            exige_misma(s_cabeza, s_rechazo)?;
            let pedido = u64_de(campos, "requested")?;
            if pedido != u64_de(g, "requested")? {
                return Err(err(
                    "data: el importe que el nodo dice no es el que la prueba acota".into(),
                ));
            }
            if pedido == 0 {
                return Err(err(
                    "la causa NO se sostiene: pedir 0 no puede pasar de ningun saldo".into(),
                ));
            }
            let pid = digest_de(g, "publicId")?;
            let prueba = hex_a_bytes(
                g.get("prueba")
                    .and_then(|x| x.as_str())
                    .ok_or_else(|| err("banda: falta prueba o no es cadena 0x".into()))?,
            )?;
            let pi = BandaPublicInputs {
                root: digest_de(c, "accountsRoot")?,
                public_id: pid,
                lower: BaseElement::ZERO,
                upper: BaseElement::new(pedido - 1),
            };
            println!("2/3 el enunciado es el de la cuenta nombrada bajo el accountsRoot firmado");
            verificar_banda(&prueba, &pi).map_err(|e| err(format!("banda: {e}")))?;
            println!(
                "3/3 la prueba verifica: el saldo de esa cuenta esta en [0, {}] bajo esa raiz, y \
                 el importe pedido era {pedido}",
                pedido - 1
            );
        }
        otra => {
            return Err(err(format!(
                "data: la causa {otra} no la prueba este mando (spec/PAQUETE.md, seccion 2.6)"
            )))
        }
    }
    println!("VERDE: {causa} se sostiene sobre el estado comprometido. Dice que la regla se");
    println!("       aplico, no que sea justa (RFC-0007)");
    Ok(())
}

/// **El paquete de EDAD** (RFC-0007 E4b-2, S465): la prueba de edad de `zk-ssl-air` contra una
/// cabeza v5 firmada. Este mando lee el sobre, exige la v5 antes de tocar la firma, verifica la
/// cabeza y le pasa al juez lo que ella firma; la regla que ENLAZA la prueba a la cabeza vive en
/// `zk_ssl_air::verificar_contra_cabeza`, un solo productor en el crate que el tercero compila.
fn verificar_edad(p: &serde_json::Value) -> Result<(), String> {
    let e = p.get("enunciado").ok_or_else(|| err("falta enunciado".into()))?;
    let t = u64_de(e, "t")?;
    let k = u64_de(e, "k")?;
    let emisor = match e.get("emisor") {
        None => None,
        Some(_) => Some(u64_de(e, "emisor")?),
    };
    let sub = p.get("subraices").ok_or_else(|| err("falta subraices".into()))?;
    let subraiz_pend = digest_de(sub, "pendientes")?;
    let subraiz_meta = digest_de(sub, "meta")?;
    let prueba = hex_a_bytes(
        p.get("prueba")
            .and_then(|x| x.as_str())
            .ok_or_else(|| err("falta prueba o no es cadena 0x".into()))?,
    )?;
    let c = p.get("cabeza").ok_or_else(|| err("falta cabeza".into()))?;
    let version = u64_de(c, "formatVersion")?;
    if !VersionCabeza::try_from(version).map_or(false, VersionCabeza::lleva_parametros) {
        return Err(err(format!(
            "formatVersion {version}: la prueba de edad exige una cabeza {}, las que firman \
             pmetaRoot y nextPending",
            VersionCabeza::texto_con_parametros()
        )));
    }
    let _ = cabeza_v3_verificada(c, "cabeza")?;
    let seq = u64_de(c, "seq")?;
    let f = familia_v5(c)?;
    println!(
        "1/3 la cabeza v{} recompone su digest y su firma verifica (seq {seq})",
        u64_de(c, "formatVersion")?
    );
    let cabeza = CabezaEdad {
        seq,
        pending_root: digest_de(c, "pendingRoot")?,
        pmeta_root: f.pmeta_root,
        next_pending: f.next_pending,
    };
    let af = Afirmacion { t, k, emisor, subraiz_pend, subraiz_meta };
    let pi = verificar_contra_cabeza(&prueba, &af, &cabeza).map_err(|e| err(format!("edad: {e}")))?;
    println!(
        "2/3 las dos subraices suben a pendingRoot y pmetaRoot (m {}, nextPending {})",
        pi.m, pi.n
    );
    println!("3/3 la prueba verifica contra ese enunciado con las opciones de la casa");
    let quien = match emisor {
        None => "de cualquier emisor".to_string(),
        Some(s) => format!("del emisor {s}"),
    };
    println!("VERDE: bajo la cabeza de seq {seq}, a lo sumo {k} posiciones vivas {quien}");
    println!("       tienen edad >= {t}. Nada sobre importes ni sobre lo que nunca entro");
    Ok(())
}

use zk_ssl_air::cobro_pendiente::{
    verificar_contra_cabeza as enlazar_cobro, AfirmacionCobro, CabezaCobro,
};

/// **El paquete de COBRO PENDIENTE** (RFC-0008 E1, S495): la prueba del cobrador, de
/// `zk-ssl-air`, contra una cabeza v5 firmada. El molde es el de la edad: la forma del sobre y la
/// v5 antes de la firma, la cabeza verificada, y la regla que ENLAZA -las dos raices y el `seq` de
/// la cabeza, el techo del campo y el nacido anterior- en `zk_ssl_air::cobro_pendiente`, un solo
/// productor en el crate que el tercero compila (D-J, D-K).
fn verificar_cobro_pendiente(p: &serde_json::Value) -> Result<(), String> {
    let e = p.get("enunciado").ok_or_else(|| err("falta enunciado".into()))?;
    let receptor = digest_de(e, "receptor")?;
    let nacido = u64_de(e, "nacido")?;
    let inferior = u64_de(e, "inferior")?;
    let prueba = hex_a_bytes(
        p.get("prueba")
            .and_then(|x| x.as_str())
            .ok_or_else(|| err("falta prueba o no es cadena 0x".into()))?,
    )?;
    let c = p.get("cabeza").ok_or_else(|| err("falta cabeza".into()))?;
    let version = u64_de(c, "formatVersion")?;
    if !VersionCabeza::try_from(version).map_or(false, VersionCabeza::lleva_parametros) {
        return Err(err(format!(
            "formatVersion {version}: el cobro pendiente exige una cabeza {}, las que firman \
             pmetaRoot",
            VersionCabeza::texto_con_parametros()
        )));
    }
    let _ = cabeza_v3_verificada(c, "cabeza")?;
    let seq = u64_de(c, "seq")?;
    let f = familia_v5(c)?;
    println!(
        "1/3 la cabeza v{} recompone su digest y su firma verifica (seq {seq})",
        u64_de(c, "formatVersion")?
    );
    let cabeza = CabezaCobro {
        seq,
        pending_root: digest_de(c, "pendingRoot")?,
        pmeta_root: f.pmeta_root,
    };
    let af = AfirmacionCobro { receptor, nacido, inferior };
    enlazar_cobro(&prueba, &af, &cabeza).map_err(|e| err(format!("cobro: {e}")))?;
    println!("2/3 el enunciado toma pendingRoot, pmetaRoot y el techo del campo; nacido {nacido}");
    println!("    es anterior al seq {seq}");
    println!("3/3 la prueba verifica contra ese enunciado con las opciones de la casa");
    println!("VERDE: bajo la cabeza de seq {seq} hay un pendiente a nombre del receptor,");
    println!("       por al menos {inferior}, nacido en {nacido}. Nada sobre quien lo pago");
    println!("       ni cuando caduca (RFC-0008)");
    Ok(())
}

use zk_ssl_air::pago_en_curso::{
    verificar_contra_cabeza as enlazar_pago, AfirmacionPago, CabezaPago,
};

/// **RFC-0008 E2 (S506): el PAGO EN CURSO.** El espejo del cobro, y con el enunciado al reves de
/// exigente: el cobrador afirma <<me deben AL MENOS `inferior`>>, y el pagador <<pague `importe`
/// EXACTO y no puedo revertirlo antes de `T`>>. El juez es el mismo que la capa usa para
/// re-verificar lo que produce (`zk_ssl_air::pago_en_curso`, S503), que el kit compila SIN el
/// probador; el plazo `delta` NO viaja, y por eso lo que se lee del enunciado es `t` y no un
/// vencimiento. Todo sale de la cabeza firmada o del enunciado: nada del libro, nada de un nodo.
fn verificar_pago_en_curso(p: &serde_json::Value) -> Result<(), String> {
    let e = p.get("enunciado").ok_or_else(|| err("falta enunciado".into()))?;
    let receptor = digest_de(e, "receptor")?;
    let importe = u64_de(e, "importe")?;
    let t = u64_de(e, "t")?;
    let nacido = u64_de(e, "nacido")?;
    let prueba = hex_a_bytes(
        p.get("prueba")
            .and_then(|x| x.as_str())
            .ok_or_else(|| err("falta prueba o no es cadena 0x".into()))?,
    )?;
    let c = p.get("cabeza").ok_or_else(|| err("falta cabeza".into()))?;
    let version = u64_de(c, "formatVersion")?;
    if !VersionCabeza::try_from(version).map_or(false, VersionCabeza::lleva_parametros) {
        return Err(err(format!(
            "formatVersion {version}: el pago en curso exige una cabeza {}, las que firman \
             pmetaRoot",
            VersionCabeza::texto_con_parametros()
        )));
    }
    let _ = cabeza_v3_verificada(c, "cabeza")?;
    let seq = u64_de(c, "seq")?;
    let f = familia_v5(c)?;
    println!(
        "1/3 la cabeza v{} recompone su digest y su firma verifica (seq {seq})",
        u64_de(c, "formatVersion")?
    );
    let cabeza = CabezaPago {
        seq,
        pending_root: digest_de(c, "pendingRoot")?,
        pmeta_root: f.pmeta_root,
    };
    let af = AfirmacionPago { receptor, importe, t, nacido };
    enlazar_pago(&prueba, &af, &cabeza).map_err(|e| err(format!("pago: {e}")))?;
    println!("2/3 el enunciado toma las dos raices, el importe EXACTO y T; nacido {nacido}");
    println!("    es anterior al seq {seq}");
    println!("3/3 la prueba verifica contra ese enunciado con las opciones de la casa");
    println!("VERDE: bajo la cabeza de seq {seq} hay un pendiente a nombre del receptor por");
    println!("       {importe} EXACTO, nacido en {nacido}, que quien lo pago no puede revertir");
    println!("       antes de {t}. Nada sobre quien lo cobrara, ni el plazo, que no viaja");
    println!("       (RFC-0008)");
    Ok(())
}

use zk_ssl_air::prenda::{
    verificar_contra_cabeza as enlazar_prenda, AfirmacionPrenda, CabezaPrenda,
};

/// **RFC-0008 E3 (S520): la PRENDA.** La mitad que un tercero juzga sin nodo y sin libro, y la
/// unica de la familia cuyo enunciado es de AUTORIZACION y no de estado: el cobro y el pago los
/// produce cualquiera que tenga la apertura; esta, solo quien tiene la clave (D-AV). Es tambien
/// el enunciado mas corto -- `{receptor, marca}`, dos campos donde el cobro lleva tres y el pago
/// cuatro -- porque la prenda NO lleva la meta (D-AY): su cabeza aporta UNA raiz y no hay
/// `nacido` que comparar contra el `seq`. El juez es `zk_ssl_air::prenda` (S516), el MISMO con el
/// que la capa re-verifica lo que produce (S518) y con el que `zkssl_pledge` juzga antes de
/// escribir (S519), y el kit lo compila SIN el probador.
///
/// La cabeza se exige **v5** (D-BF) y el texto dice SU razon: no la de los hermanos -- «la unica
/// que firma pmetaRoot» --, que aqui seria falsa, sino que es la que el nodo sirve desde el S452
/// y contra la que `zkssl_pledge` juzga.
///
/// Lo que este brazo NO dice, y va impreso donde se lee: que la `marca` este publicada en el
/// arbol de consumos. El juez lo declara en su propia doc -- «es la puerta de quien escribe en
/// el, no la del juez» -- y este binario no tiene arbol que mirar. Un VERDE aqui es MEDIA prenda:
/// el par es la marca bajo la raiz firmada MAS este sobre (D-AS).
fn verificar_prenda(p: &serde_json::Value) -> Result<(), String> {
    let (af, prueba, marca_hex) = enunciado_de_prenda(p)?;
    let c = p.get("cabeza").ok_or_else(|| err("falta cabeza".into()))?;
    let version = u64_de(c, "formatVersion")?;
    if !VersionCabeza::try_from(version).map_or(false, VersionCabeza::lleva_parametros) {
        return Err(err(format!(
            "formatVersion {version}: la prenda exige una cabeza {} - no por la meta, que no \
             lleva (D-AY), sino porque es la que el nodo sirve y contra la que juzga \
             zkssl_pledge",
            VersionCabeza::texto_con_parametros()
        )));
    }
    let _ = cabeza_v3_verificada(c, "cabeza")?;
    let seq = u64_de(c, "seq")?;
    println!(
        "1/3 la cabeza v{} recompone su digest y su firma verifica (seq {seq})",
        u64_de(c, "formatVersion")?
    );
    let cabeza = CabezaPrenda { pending_root: digest_de(c, "pendingRoot")? };
    enlazar_prenda(&prueba, &af, &cabeza).map_err(|e| err(format!("prenda: {e}")))?;
    println!("2/3 el enunciado toma la raiz de pendientes y la marca; ni importe, ni sal, ni");
    println!("    nacido: la prenda no lleva la meta");
    println!("3/3 la prueba verifica contra ese enunciado con las opciones de la casa");
    println!("VERDE: bajo la cabeza de seq {seq} hay un pendiente que solo puede cobrar quien");
    println!("       tiene la clave del receptor, y su marca es {marca_hex}. Es MEDIA prenda:");
    println!("       esto NO dice que la marca este publicada, que es del arbol de consumos y");
    println!("       se pide con zkssl_consumoPath (RFC-0008 D-AS)");
    Ok(())
}

/// Lo que un sobre de prenda AFIRMA -`{receptor, marca}`- y la prueba que lo sostiene, leidos SIN
/// su cabeza, con los textos de siempre. Lo comparten el paquete de prenda y la resolucion de la
/// prenda en el sobre de completitud (§613), que la juzga contra OTRA cabeza: la que juzgo el nodo.
/// Devuelve tambien la marca en el hex en que llego, para imprimirla.
fn enunciado_de_prenda(p: &serde_json::Value) -> Result<(AfirmacionPrenda, Vec<u8>, String), String> {
    let e = p.get("enunciado").ok_or_else(|| err("falta enunciado".into()))?;
    let receptor = digest_de(e, "receptor")?;
    let marca = digest_de(e, "marca")?;
    // `Digest` es `[BaseElement; 4]` y no se imprime solo. El hex que se ensena es el que YA
    // paso por `digest_de`, no una segunda lectura sin puerta.
    let marca_hex = e.get("marca").and_then(|x| x.as_str()).unwrap_or_default().to_string();
    let prueba = hex_a_bytes(
        p.get("prueba")
            .and_then(|x| x.as_str())
            .ok_or_else(|| err("falta prueba o no es cadena 0x".into()))?,
    )?;
    Ok((AfirmacionPrenda { receptor, marca }, prueba, marca_hex))
}

/// Los siete parametros de `zkssl_params` contra el `paramsDigest` de una cabeza **v5** (RFC-0007
/// D-B): si recomponen, lo que dicen es lo comprometido. Devuelve los tres que las causas citan:
/// el limite regulatorio, el tope de cuentas y el tope de suministro (§460).
fn parametros_comprometidos(
    p: &serde_json::Value,
    c: &serde_json::Value,
    causa: &str,
) -> Result<(u64, u64, u64), String> {
    if !VersionCabeza::try_from(u64_de(c, "formatVersion")?)
        .map_or(false, VersionCabeza::lleva_parametros)
    {
        return Err(err(format!(
            "la causa {causa} exige una cabeza {}: sus parametros viajan en paramsDigest",
            VersionCabeza::texto_con_parametros()
        )));
    }
    let f = familia_v5(c)?;
    let pr = p
        .get("parametros")
        .ok_or_else(|| err("falta parametros (zkssl_params)".into()))?;
    let limite = u64_de(pr, "regulatoryLimit")?;
    let tope = u64_de(pr, "maxAccounts")?;
    let suministro_max = u64_de(pr, "maxSupply")?;
    let compuesto = params_digest(
        limite,
        suministro_max,
        tope,
        digest_de(pr, "custodianRoot")?,
        digest_de(pr, "governanceRoot")?,
        u64_de(pr, "refundTtl")?,
        u64_de(pr, "maxCustodianUses")?,
    );
    if compuesto != f.params_digest {
        return Err(err(
            "parametros: NO recomponen el paramsDigest de la cabeza - no son los de este libro"
                .into(),
        ));
    }
    Ok((limite, tope, suministro_max))
}

/// UN productor del texto de un campo de la causa que no es el comprometido.
fn no_es_el_comprometido(que: &str, dicho: u64, comprometido: u64) -> String {
    err(format!(
        "data: el {que} que el nodo dice ({dicho}) no es el comprometido ({comprometido})"
    ))
}

/// UN productor de la regla del `seq` para lo que solo crece: la cabeza tiene que ser ANTERIOR al
/// rechazo, o la misma -lo que ya estaba en ella seguia estando al juzgar-.
fn exige_anterior(s_cabeza: u64, s_rechazo: u64, porque: &str) -> Result<(), String> {
    if s_cabeza > s_rechazo {
        return Err(err(format!(
            "la cabeza (seq {s_cabeza}) es POSTERIOR al rechazo (seq {s_rechazo}): {porque}, y \
             solo una cabeza anterior lo prueba"
        )));
    }
    Ok(())
}

/// La cabeza tiene que ser la MISMA en que se juzgo (`StaleState`, los duplicados de lote): lo que
/// se prueba es un estado instantaneo -las raices de ese `seq`, el lote contra ese registro-, no
/// algo que solo crezca. Una anterior o posterior probaria otro estado.
fn exige_misma(s_cabeza: u64, s_rechazo: u64) -> Result<(), String> {
    if s_cabeza != s_rechazo {
        return Err(err(format!(
            "la cabeza (seq {s_cabeza}) no es la del rechazo (seq {s_rechazo}): esta causa se \
             juzga sobre un estado instantaneo, no sobre lo que crece"
        )));
    }
    Ok(())
}

/// La cuenta y la posicion de una operacion de un lote, con la MISMA regla que `BatchOp` en la
/// capa: la cuenta es `sender` (envio) o `receiver` (cobro); la posicion, `receipt.notice.position`
/// en el envio o `notice.position` en el cobro (`crates/zk-ssl/src/two_phase.rs`).
fn op_cuenta_y_posicion(op: &serde_json::Value, i: usize) -> Result<(u64, u64), String> {
    let kind = op
        .get("kind")
        .and_then(|x| x.as_str())
        .ok_or_else(|| err(format!("lote[{i}]: falta kind")))?;
    match kind {
        "send" => {
            let cuenta = u64_de(op, "sender")?;
            let notice = op
                .get("receipt")
                .and_then(|r| r.get("notice"))
                .ok_or_else(|| err(format!("lote[{i}]: falta receipt.notice")))?;
            Ok((cuenta, u64_de(notice, "position")?))
        }
        "claim" => {
            let cuenta = u64_de(op, "receiver")?;
            let notice = op
                .get("notice")
                .ok_or_else(|| err(format!("lote[{i}]: falta notice")))?;
            Ok((cuenta, u64_de(notice, "position")?))
        }
        otro => Err(err(format!("lote[{i}]: kind desconocido: {otro}"))),
    }
}

/// Las causas que el RFC-0007 DECLARO sin prueba portable -su tabla de causas y la correccion del
/// §476-: `CustodianSetExhausted` y `PendingTreeExhausted` (declaradas sin prueba portable) y
/// `NotTheIssuer`, `NotTheAccountHolder` (la autorizacion ausente no se puede exhibir). Un recibo que
/// se resuelve por una de ellas no puede exhibir el veredicto 2: es la grieta de la D-G del RFC-0010,
/// y el sobre la NOMBRA en vez de callarla.
const CAUSAS_SIN_PRUEBA_PORTABLE: [&str; 4] =
    ["CustodianSetExhausted", "PendingTreeExhausted", "NotTheIssuer", "NotTheAccountHolder"];

/// El prefijo del CUARTO estado (RFC-0010 D-G; decision D4 del autor, sesion 193): «resolucion
/// declarada, no probada». Ni VERDE ni ROJO: `codigo_de_salida` lo cuenta aparte, con el 3.
const DECLARADA_NO_PROBADA: &str = "DECLARADA, NO PROBADA: ";

/// El codigo de salida de un veredicto: 0 VERDE, 3 el cuarto estado del sobre de completitud, 1
/// cualquier otro fallo con nombre. El 2 -uso- lo decide `main` antes de leer nada.
fn codigo_de_salida(r: &Result<(), String>) -> u8 {
    match r {
        Ok(()) => 0,
        Err(e) if e.starts_with(DECLARADA_NO_PROBADA) => 3,
        Err(_) => 1,
    }
}

/// D3 del autor: un rechazo se ata a ESTE recibo por el `recepcion.hashPrueba` que el nodo puso en
/// su `data` (§571). ⚠️ Es la PALABRA del nodo: el `error` del cable no va firmado. Lo que la firma
/// cubre es la causa sobre el estado comprometido; que fuera ESTA operacion lo dice el acusado.
fn exige_mismo_recibo(data: &serde_json::Value, hash: Digest, que: &str) -> Result<(), String> {
    let rp = data
        .get("recepcion")
        .ok_or_else(|| err(format!("{que}: su data no lleva recepcion: no se ata a este recibo")))?;
    if digest_de(rp, "hashPrueba")? != hash {
        return Err(err(format!(
            "{que}: es de OTRA operacion: su hashPrueba no es el del recibo"
        )));
    }
    Ok(())
}

/// La resolucion por ACUSE (veredicto 1 del RFC-0010): la cabeza del mismo operador dentro de la
/// ventana, el acuse de la MISMA prueba y el par verificado COMO el paquete de posicion que es.
/// `que` nombra el sitio: `resolucion` en la via directa, `resolucion.acuses[i]` en el lote (§612).
/// Devuelve el indice XMSS de la cabeza.
fn resolver_por_acuse(
    x: &serde_json::Value,
    hash: Digest,
    era: u64,
    n: u64,
    clave: &str,
    que: &str,
) -> Result<u64, String> {
    let c = x.get("cabeza").ok_or_else(|| err(format!("{que}: falta cabeza")))?;
    let a = x.get("acuse").ok_or_else(|| err(format!("{que}: falta acuse")))?;
    let (_, _, clave_r, _) = cabeza_v3_verificada(c, &format!("{que}.cabeza"))?;
    if clave_r != clave {
        return Err(claves_distintas());
    }
    if digest_de(a, "hashPrueba")? != hash {
        return Err(err(format!("{que}: el acuse es de OTRA prueba: no resuelve este recibo")));
    }
    let s = u64_de(c, "index")?;
    if !recibos::dentro_de_ventana(era, s, n) {
        return Err(err(format!(
            "{que}: llega FUERA de la ventana (indice {s}, era {era}, n {n})"
        )));
    }
    println!("   la resolucion, como paquete de posicion con su acuse:");
    verificar_paquete(&serde_json::json!({ "v": 1, "cabeza": c, "acuse": a }))?;
    Ok(s)
}

/// La resolucion por RECHAZO (veredicto 2 del RFC-0010): el sobre de la seccion 2.6, atado al
/// recibo por su `data.recepcion` (D3), sobre una cabeza del mismo operador dentro de la ventana, y
/// verificado por sus reglas. `que` nombra el sitio: `resolucion.sobre` en la via directa y en el
/// lote, `resolucion.rechazo` en la prenda (§613). Devuelve el indice XMSS de su cabeza y su `data`.
fn resolver_por_rechazo<'a>(
    sobre: &'a serde_json::Value,
    hash: Digest,
    era: u64,
    n: u64,
    clave: &str,
    que: &str,
) -> Result<(u64, &'a serde_json::Value), String> {
    if sobre.get("tipo").and_then(|t| t.as_str()) != Some("rechazo") {
        return Err(err("resolucion: el sobre no es de tipo rechazo".into()));
    }
    let d = sobre
        .get("data")
        .ok_or_else(|| err(format!("{que}: falta data")))?;
    exige_mismo_recibo(d, hash, que)?;
    let c = sobre
        .get("cabeza")
        .ok_or_else(|| err(format!("{que}: falta cabeza")))?;
    let (_, _, clave_r, _) = cabeza_v3_verificada(c, &format!("{que}.cabeza"))?;
    if clave_r != clave {
        return Err(claves_distintas());
    }
    let s = u64_de(c, "index")?;
    if !recibos::dentro_de_ventana(era, s, n) {
        return Err(err(format!(
            "resolucion: llega FUERA de la ventana (indice {s}, era {era}, n {n})"
        )));
    }
    println!("   la resolucion, como sobre de rechazo:");
    verificar_rechazo(sobre)?;
    Ok((s, d))
}

/// Una operacion de un lote tal como la compone quien lo arma: el digest de su prueba, su cuenta y
/// la posicion de su pendiente (RFC-0014 D-A). Los mismos tres campos que `hash_del_lote` escribe.
type OperacionDelLote = (Digest, u64, u64);

/// **La composicion de un lote** (RFC-0014 D-B, §612): lo que el agregador reenvia a cada titular,
/// `[{hashPrueba, cuenta, posicion}]` en el orden del lote (D-C). Se lee ENTERA antes de nada, y
/// vacia no es un lote: el nodo no evalua uno vacio.
fn composicion_de(x: &serde_json::Value) -> Result<Vec<OperacionDelLote>, String> {
    let a = x
        .get("composicion")
        .and_then(|c| c.as_array())
        .ok_or_else(|| err("resolucion: falta composicion (la lista del lote, D-C)".into()))?;
    if a.is_empty() {
        return Err(err("resolucion: composicion VACIA: el nodo no evalua un lote vacio".into()));
    }
    a.iter()
        .enumerate()
        .map(|(i, o)| {
            let en = |e: String| err(format!("resolucion.composicion[{i}]: {e}"));
            Ok((
                digest_de(o, "hashPrueba").map_err(en)?,
                u64_de(o, "cuenta").map_err(en)?,
                u64_de(o, "posicion").map_err(en)?,
            ))
        })
        .collect()
}

/// La operacion `j` que el nodo nombra en el `data` de un lote rechazado (§611), dentro del lote.
fn operacion_nombrada(d: &serde_json::Value, k: usize, que: &str) -> Result<usize, String> {
    let j = u64_de(d, "operacion").map_err(|e| err(format!("{que}: {e}: el rechazo de un lote \
                                                            nombra su operacion (§611)")))?;
    if j >= k as u64 {
        return Err(err(format!("{que}: la operacion {j} no esta en un lote de {k}")));
    }
    Ok(j as usize)
}

/// El campo de `data.campos` que nombra lo repetido, por causa de FORMA: la capa lo escribe asi
/// (`LayerError::causa`, §454).
fn campo_de_forma(causa: &str) -> Option<&'static str> {
    match causa {
        "DuplicateAccountInBatch" => Some("index"),
        "DuplicatePendingInBatch" => Some("position"),
        _ => None,
    }
}

/// **La FORMA del lote, juzgada OTRA VEZ y sin el nodo** (RFC-0014 D-B, §612).
/// `DuplicateAccountInBatch` y `DuplicatePendingInBatch` se prueban con la composicion sola: la
/// operacion `j` que el nodo nombra tiene que llevar la cuenta -o la posicion- `valor` que su
/// `campos` dice, y alguna ANTERIOR tiene que llevarla tambien. Devuelve la primera del par.
///
/// ⚠️ `Err` aqui no es un fallo de lectura -lo leido ya se leyo-: es el juicio del operador
/// REPETIDO y contradicho, y quien llama lo nombra (la regla de la decision 3 del §609).
fn juzgar_forma(
    comp: &[OperacionDelLote],
    j: usize,
    causa: &str,
    valor: u64,
) -> Result<usize, String> {
    let (por_cuenta, que) = match campo_de_forma(causa) {
        Some("index") => (true, "cuenta"),
        Some(_) => (false, "posicion"),
        None => return Err(format!("{causa} no es una causa de la forma del lote")),
    };
    let de = |o: &OperacionDelLote| if por_cuenta { o.1 } else { o.2 };
    let op = comp.get(j).ok_or_else(|| format!("no hay operacion {j} en un lote de {}", comp.len()))?;
    if de(op) != valor {
        return Err(format!(
            "la operacion {j} lleva la {que} {} y la causa nombra la {valor}",
            de(op)
        ));
    }
    comp[..j]
        .iter()
        .position(|o| de(o) == valor)
        .ok_or_else(|| format!("ninguna operacion anterior a la {j} lleva la {que} {valor}"))
}

/// **La resolucion de un LOTE** (RFC-0014 E4a, §612; D-B). El recibo del lote lleva como
/// `hashPrueba` la huella de su COMPOSICION, y lo primero es recomponerla: sin eso nada de lo demas
/// habla de este recibo. Despues, UNA de tres cosas:
///
/// - `acuses`: el lote se APLICO -uno por operacion, en su orden, cada uno el de SU prueba y
///   resuelto como en la via directa-. VERDE.
/// - `sobre`: se RECHAZO con prueba -el sobre de la seccion 2.6 de la operacion que el nodo nombra,
///   atado al recibo por su `data.recepcion` y su `data.operacion`, que son la PALABRA del nodo
///   (D3)-. Las companeras quedan resueltas por ella: el lote es la unidad. VERDE.
/// - `data`: el `error.data` del rechazo, tal cual. Con una causa de FORMA, el mando repite el
///   juicio con la composicion: si la sostiene, VERDE sin cabeza ni ventana -el rechazo fue en el
///   acto-; si no, **ROJO NOMBRADO, «RECHAZO SIN FUNDAMENTO»**. Con una causa sin prueba portable,
///   el cuarto estado, como en la via directa.
fn resolver_lote(
    x: &serde_json::Value,
    hash: Digest,
    era: u64,
    n: u64,
    clave: &str,
) -> Result<(), String> {
    let comp = composicion_de(x)?;
    let k = comp.len();
    if recibos::hash_del_lote(&comp) != hash {
        return Err(err(
            "resolucion: la composicion NO es la del recibo: su huella no es su hashPrueba".into(),
        ));
    }
    println!("   la composicion, {k} operacion(es), recompone el hashPrueba del recibo");
    let hay = |c: &str| x.get(c).is_some();
    match (hay("acuses"), hay("sobre"), hay("data")) {
        (true, false, false) => {
            let acuses = x["acuses"]
                .as_array()
                .ok_or_else(|| err("resolucion.acuses: no es una lista".into()))?;
            if acuses.len() != k {
                return Err(err(format!(
                    "resolucion: {} acuse(s) para un lote de {k}: el lote se aplica ENTERO o no \
                     se aplica",
                    acuses.len()
                )));
            }
            for (i, (par, op)) in acuses.iter().zip(&comp).enumerate() {
                resolver_por_acuse(par, op.0, era, n, clave, &format!("resolucion.acuses[{i}]"))?;
            }
            println!(
                "3/3 RESUELTA como LOTE aplicado: sus {k} operacion(es), cada una con el acuse de \
                 SU prueba dentro de la ventana (era {era}, n {n})"
            );
        }
        (false, true, false) => {
            let (s, d) = resolver_por_rechazo(&x["sobre"], hash, era, n, clave, "resolucion.sobre")?;
            let j = operacion_nombrada(d, k, "resolucion.sobre.data")?;
            println!(
                "3/3 RESUELTA como LOTE rechazado con prueba, dentro de la ventana (indice {s}): \
                 la operacion {j} no se sostenia, y sus companeras quedan resueltas por ella -el \
                 lote es la unidad-; la atadura al recibo y a la operacion {j} es la palabra del \
                 nodo en su data"
            );
        }
        (false, false, true) => {
            let d = &x["data"];
            exige_mismo_recibo(d, hash, "resolucion.data")?;
            let j = operacion_nombrada(d, k, "resolucion.data")?;
            let causa = d
                .get("causa")
                .and_then(|c| c.as_str())
                .ok_or_else(|| err("resolucion.data: falta causa".into()))?;
            if let Some(campo) = campo_de_forma(causa) {
                let valor = d
                    .get("campos")
                    .ok_or_else(|| err("resolucion.data: falta campos".into()))
                    .and_then(|c| u64_de(c, campo).map_err(|e| format!("resolucion.data.campos: {e}")))?;
                match juzgar_forma(&comp, j, causa, valor) {
                    Ok(i) => println!(
                        "3/3 RESUELTA como LOTE rechazado por su FORMA: {causa} en la operacion \
                         {j}, que repite la de la {i} -lo prueba la composicion sola, en el acto y \
                         sin cabeza-"
                    ),
                    Err(por_que) => {
                        return Err(err(format!(
                            "RECHAZO SIN FUNDAMENTO: el nodo dijo {causa} en la operacion {j} del \
                             lote, y la composicion que su recibo firma no lo sostiene: {por_que}. \
                             El juicio se repite sin el nodo; su negativa es su palabra, en su data \
                             (D3)"
                        )))
                    }
                }
            } else if CAUSAS_SIN_PRUEBA_PORTABLE.contains(&causa) {
                return Err(format!(
                    "{DECLARADA_NO_PROBADA}el operador declara {causa} en la operacion {j} del \
                     lote, una causa que el RFC-0007 dejo sin prueba portable: se cuenta aparte \
                     (RFC-0010, D-G)"
                ));
            } else {
                return Err(err(format!(
                    "resolucion del lote con la causa {causa} y sin sobre: no es de forma ni de \
                     las que el RFC-0007 dejo sin prueba portable: se exhibe su sobre de rechazo"
                )));
            }
        }
        _ => {
            return Err(err(
                "resolucion del lote: lleva UNA de tres, acuses, sobre o data".into(),
            ))
        }
    }
    Ok(())
}

/// **La resolucion de una PRENDA** (RFC-0014 E4b, §613; D-E). El recibo de una prenda lleva como
/// `hashPrueba` el digest de la prueba que llego (§611), y lo primero es atar a el el `sobre` de la
/// seccion 2.10: su prueba tiene que tener ESE digest. Despues, UNA de tres:
///
/// - `consumo`, `{cabeza, camino}`: la prenda se ACEPTO -el sobre verifica, y su marca esta bajo el
///   `consRoot` de una cabeza del mismo operador dentro de la ventana, por el camino de
///   `zkssl_consumoPath`-. Es el PAR entero (2.10, D-AS). VERDE.
/// - `respuesta`, con `juzgada`: la negativa del nodo, tal cual, y la cabeza contra la que juzgo
///   -la ultima firmada al recibir, de indice `era - 1` (§567)-. El mando repite el juicio con EL
///   MISMO juez y la `pendingRoot` de esa cabeza: si el sobre no verifica, VERDE, rechazada con
///   prueba; si verifica, **ROJO NOMBRADO, «RECHAZO SIN FUNDAMENTO»** (decision 3 del §609). Una
///   negativa con causa en su `data` no es de esta rama: se exhibe su sobre de rechazo.
/// - `rechazo`: la capa la rechazo con causa al escribir la marca (`apply_consumo`, p. ej.
///   `ConsumoColision`): el sobre de la 2.6, resuelto como el veredicto 2 y atado al recibo por su
///   `data.recepcion`, y el consumo que rechaza es la marca de ESTE sobre. VERDE.
///
/// ⚠️ La negativa es la palabra del nodo (D3): si alguien la inventara, el operador la desmiente con
/// el PAR. ⚠️ Si un latido firmo ENTRE la comprobacion del `seq` y la reserva, la `era - 1` es una
/// cabeza posterior a la juzgada; con el mismo `seq` su raiz es la misma, y sin el, un sobre que
/// verificara contra ella tendria que haberse probado contra una raiz que aun no existia.
fn resolver_prenda(
    x: &serde_json::Value,
    hash: Digest,
    era: u64,
    n: u64,
    clave: &str,
) -> Result<(), String> {
    let sobre = x
        .get("sobre")
        .ok_or_else(|| err("resolucion: falta sobre (el de la prenda, 2.10)".into()))?;
    if sobre.get("tipo").and_then(|t| t.as_str()) != Some("prenda") {
        return Err(err("resolucion: el sobre no es de tipo prenda".into()));
    }
    let (af, prueba, marca_hex) =
        enunciado_de_prenda(sobre).map_err(|e| err(format!("resolucion.sobre: {e}")))?;
    if digest_of_proof(&prueba) != hash {
        return Err(err(
            "resolucion: la prenda es de OTRA prueba: su digest no es el hashPrueba del recibo"
                .into(),
        ));
    }
    println!("   la prueba del sobre de prenda es la del recibo: su digest es su hashPrueba");
    let hay = |c: &str| x.get(c).is_some();
    match (hay("consumo"), hay("respuesta"), hay("rechazo")) {
        (true, false, false) => {
            let c = sobre
                .get("cabeza")
                .ok_or_else(|| err("resolucion.sobre: falta cabeza".into()))?;
            let (_, _, clave_s, _) = cabeza_v3_verificada(c, "resolucion.sobre.cabeza")?;
            if clave_s != clave {
                return Err(claves_distintas());
            }
            println!("   la resolucion, como sobre de prenda:");
            verificar_prenda(sobre)?;
            let k = &x["consumo"];
            let cc = k
                .get("cabeza")
                .ok_or_else(|| err("resolucion.consumo: falta cabeza".into()))?;
            let (_, _, clave_c, cons) = cabeza_v3_verificada(cc, "resolucion.consumo.cabeza")?;
            if clave_c != clave {
                return Err(claves_distintas());
            }
            let raiz = cons.ok_or_else(|| exige_consumos("prenda"))?;
            let s = u64_de(cc, "index")?;
            if !recibos::dentro_de_ventana(era, s, n) {
                return Err(err(format!(
                    "resolucion: llega FUERA de la ventana (indice {s}, era {era}, n {n})"
                )));
            }
            let pos = zk_ssl_verify::consumos::posicion_de_consumo(&af.marca);
            let (herm, der) = camino_de(k, "camino", "resolucion.consumo.camino")?;
            if !zk_ssl_verify::consumos::cruza_posicion(pos, &der) {
                return Err(cruce_fallado("resolucion.consumo.camino", pos));
            }
            match zk_ssl_verify::consumos::raiz_de_presencia(af.marca, &herm, &der) {
                Some(r) if r == raiz => {}
                Some(_) => {
                    return Err(err(
                        "resolucion.consumo: la marca NO esta bajo el consRoot de su cabeza".into(),
                    ))
                }
                None => return Err(err(camino_descuadrado("resolucion.consumo.camino"))),
            }
            println!(
                "3/3 RESUELTA como PRENDA aceptada: el PAR -el sobre verifica y su marca \
                 {marca_hex} esta bajo el consRoot de la cabeza de indice {s}, dentro de la \
                 ventana (era {era}, n {n})-"
            );
        }
        (false, true, false) => {
            let r = &x["respuesta"];
            if r.get("accepted") != Some(&serde_json::Value::Bool(false)) {
                return Err(err(
                    "resolucion.respuesta: no es una negativa (accepted false): una prenda \
                     aceptada se resuelve con su consumo"
                        .into(),
                ));
            }
            exige_mismo_recibo(r, hash, "resolucion.respuesta")?;
            if r.get("data").is_some() {
                return Err(err(
                    "resolucion.respuesta: la negativa lleva causa en su data: se exhibe su sobre \
                     de rechazo (resolucion.rechazo)"
                        .into(),
                ));
            }
            let j = x.get("juzgada").ok_or_else(|| {
                err("resolucion: falta juzgada (la cabeza que el nodo juzgo: la ultima firmada al \
                     recibir, de indice era - 1)"
                    .into())
            })?;
            let version = u64_de(j, "formatVersion")?;
            if !VersionCabeza::try_from(version).map_or(false, VersionCabeza::lleva_parametros) {
                return Err(err(format!(
                    "resolucion.juzgada: formatVersion {version}: la prenda se juzga contra una \
                     cabeza {}",
                    VersionCabeza::texto_con_parametros()
                )));
            }
            let (_, _, clave_j, _) = cabeza_v3_verificada(j, "resolucion.juzgada")?;
            if clave_j != clave {
                return Err(claves_distintas());
            }
            let sj = u64_de(j, "index")?;
            if sj.checked_add(1) != Some(era) {
                return Err(err(format!(
                    "resolucion.juzgada: su indice {sj} no es el de la cabeza que el nodo juzgo: \
                     la ultima firmada al recibir es la de la era menos uno ({})",
                    era.saturating_sub(1)
                )));
            }
            let cab = CabezaPrenda { pending_root: digest_de(j, "pendingRoot")? };
            match enlazar_prenda(&prueba, &af, &cab) {
                Err(e) => println!(
                    "3/3 RESUELTA como PRENDA rechazada con prueba: el sobre NO verifica contra la \
                     cabeza que el nodo juzgo (indice {sj}), y cualquiera lo comprueba sin el \
                     nodo: {e}"
                ),
                Ok(_) => {
                    let dijo = r.get("reason").and_then(|x| x.as_str()).unwrap_or("(sin razon)");
                    return Err(err(format!(
                        "RECHAZO SIN FUNDAMENTO: el sobre de la prenda VERIFICA contra la cabeza \
                         que el nodo juzgo (indice {sj}), con el mismo juez y sin el nodo, y el \
                         nodo dijo que no: {dijo}. Su negativa es su palabra, en su respuesta \
                         (D3); si alguien la inventara, el operador la desmiente con el PAR"
                    )));
                }
            }
        }
        (false, false, true) => {
            let (s, d) =
                resolver_por_rechazo(&x["rechazo"], hash, era, n, clave, "resolucion.rechazo")?;
            let rechazado = d.get("campos").and_then(|c| digest_de(c, "consumo").ok());
            if rechazado != Some(af.marca) {
                return Err(err(
                    "resolucion.rechazo: el consumo que la capa rechazo no es la marca de este \
                     sobre"
                        .into(),
                ));
            }
            println!(
                "3/3 RESUELTA como PRENDA rechazada por la capa, con prueba, dentro de la ventana \
                 (indice {s}): la causa se sostiene sobre el estado comprometido, y el consumo \
                 rechazado es su marca"
            );
        }
        _ => {
            return Err(err(
                "resolucion de la prenda: lleva UNA de tres, consumo, respuesta o rechazo".into(),
            ))
        }
    }
    Ok(())
}

/// **El sobre de COMPLETITUD** (RFC-0010 E4, §573): un recibo de recepcion bajo la `recepRoot` de
/// una cabeza v6 firmada -el `cierre` de su era-, y lo que el operador hizo con el. Tres veredictos
/// (D-F) y un cuarto estado (D-G):
///
/// 1. `resolucion.tipo = "acuse"`: la transicion se APLICO dentro de la ventana -el acuse de la
///    MISMA prueba bajo una cabeza del mismo operador cuyo indice XMSS `S` cumple `S - era <= n`-.
///    VERDE.
/// 2. `resolucion.tipo = "rechazo"`: un sobre de rechazo del RFC-0007, verificado por sus reglas,
///    sobre una cabeza dentro de la ventana, y atado al recibo por su `data.recepcion` (D3). VERDE.
/// 3. Sin resolucion, con una cabeza `vigente` del mismo operador fuera de la ventana: **ROJO
///    NOMBRADO**, «NO RESUELTA EN LA VENTANA». Es el producto: un objeto portable que dice, con la
///    firma del propio operador dentro, que se comprometio a resolver y no lo hizo. ⚠️ No es una
///    prueba criptografica de ausencia (D-F): es evidencia oponible.
/// 4. `resolucion.tipo = "declarada"` con una causa sin prueba portable: el cuarto estado, salida 3.
///
/// `limiteAnterior` (Q) va DECLARADO (D1): la hoja no lleva `rx`, asi que un Q mentido solo produce
/// un camino que no cruza. `S` es siempre el indice de una cabeza FIRMADA, que `verificar_cabeza` ata
/// al embebido en la firma (§399): la ventana no se declara, se mide (D2).
fn verificar_completitud(p: &serde_json::Value) -> Result<(), String> {
    // 1 · el cierre: una cabeza v6 firmada, entera
    let cierre = p
        .get("cierre")
        .ok_or_else(|| err("falta cierre (la cabeza v6 que cierra la era del recibo)".into()))?;
    let version = VersionCabeza::try_from(u64_de(cierre, "formatVersion")?).map_err(|e| {
        err(format!(
            "cierre: formatVersion {}: el sobre de completitud lee cabezas {}",
            e.0,
            VersionCabeza::texto_con_recepcion()
        ))
    })?;
    if !version.lleva_recepcion() {
        return Err(err(format!(
            "cierre: el sobre de completitud exige una cabeza {}: la pareja de recepcion viaja \
             firmada desde ella",
            VersionCabeza::texto_con_recepcion()
        )));
    }
    let (_, _, clave, _) = cabeza_v3_verificada(cierre, "cierre")?;
    let raiz = digest_de(cierre, "recepRoot")?;
    let r = u64_de(cierre, "recepCount")?;
    let n_firmada = u64_de(cierre, "n")?;
    println!(
        "1/3 el cierre es una cabeza v{} que recompone su digest y cuya firma verifica \
         (indice {}, recepCount {r})",
        version.as_u8(),
        u64_de(cierre, "index")?
    );

    // 2 · el recibo, bajo la firma del cierre
    let rec = p
        .get("recepcion")
        .ok_or_else(|| err("falta recepcion (el recibo del cable, §571)".into()))?;
    let rx = u64_de(rec, "rx")?;
    let era = u64_de(rec, "era")?;
    let n = u64_de(rec, "n")?;
    let hash = digest_de(rec, "hashPrueba")?;
    if n != n_firmada {
        return Err(err(format!(
            "recepcion: n {n} no es la que el cierre firma ({n_firmada}): una n mentida produce \
             una hoja que no verifica"
        )));
    }
    let q = u64_de(p, "limiteAnterior")?;
    if !recibos::pertenece_a_era(rx, q, r) {
        return Err(err(format!(
            "recepcion: el rx {rx} no pertenece a la era ({q}, {r}] que el cierre cierra"
        )));
    }
    let pos = recibos::indice_de_recibo(rx, q);
    if p.get("camino").is_none() {
        return Err(err("falta camino (el de zkssl_recepPath)".into()));
    }
    let (hermanos, derecha) = camino_de(p, "camino", "camino")?;
    let hoja = recibos::hoja_de_recibo(hash, era, n);
    match recibos::raiz_de_camino_de_recibo(pos, hoja, &hermanos, &derecha) {
        None => {
            return Err(err(format!(
                "camino: no mide {} niveles o sus lados no son los de la posicion {pos} \
                 (rx {rx} - Q {q} - 1)",
                recibos::RECEP_DEPTH
            )))
        }
        Some(x) if x != raiz => {
            return Err(err("el recibo NO sube a la recepRoot que el cierre firma".into()))
        }
        Some(_) => {}
    }
    println!(
        "2/3 el recibo (rx {rx}, era {era}) esta bajo la recepRoot que el cierre firma: \
         el operador lo RECIBIO"
    );

    // 3 · el veredicto
    let res = p.get("resolucion");
    match res.map(|x| x.get("tipo").and_then(|t| t.as_str())) {
        None => {
            let vig = p.get("vigente").ok_or_else(|| {
                err("sin resolucion y sin cabeza vigente: no hay ventana que medir".into())
            })?;
            let (_, _, clave_v, _) = cabeza_v3_verificada(vig, "vigente")?;
            if clave_v != clave {
                return Err(claves_distintas());
            }
            let s = u64_de(vig, "index")?;
            if recibos::dentro_de_ventana(era, s, n) {
                return Err(err(format!(
                    "ventana ABIERTA: la cabeza vigente tiene indice {s} y la era es {era}; con \
                     n {n} la promesa sigue viva y el sobre es prematuro"
                )));
            }
            return Err(err(format!(
                "NO RESUELTA EN LA VENTANA: el operador recibio la operacion bajo su firma \
                 (rx {rx}, era {era}) y su cabeza vigente de indice {s} -{} cabezas despues, \
                 n {n}- llega sin resolucion: se comprometio a resolver y no lo hizo",
                s - era
            )));
        }
        Some(Some("acuse")) => {
            let s = resolver_por_acuse(res.expect("hay resolucion"), hash, era, n, &clave, "resolucion")?;
            println!(
                "3/3 RESUELTA como transicion aplicada, dentro de la ventana (indice {s}, \
                 era {era}, n {n})"
            );
        }
        Some(Some("rechazo")) => {
            let sobre = res
                .expect("hay resolucion")
                .get("sobre")
                .ok_or_else(|| err("resolucion: falta sobre (el de rechazo)".into()))?;
            let (s, _) = resolver_por_rechazo(sobre, hash, era, n, &clave, "resolucion.sobre")?;
            println!(
                "3/3 RESUELTA como rechazo con prueba, dentro de la ventana (indice {s}); la \
                 atadura al recibo es la palabra del nodo en su data"
            );
        }
        Some(Some("lote")) => {
            resolver_lote(res.expect("hay resolucion"), hash, era, n, &clave)?;
        }
        Some(Some("prenda")) => {
            resolver_prenda(res.expect("hay resolucion"), hash, era, n, &clave)?;
        }
        Some(Some("declarada")) => {
            let d = res
                .expect("hay resolucion")
                .get("data")
                .ok_or_else(|| err("resolucion: falta data (la del rechazo declarado)".into()))?;
            let causa = d
                .get("causa")
                .and_then(|x| x.as_str())
                .ok_or_else(|| err("resolucion.data: falta causa".into()))?;
            if !CAUSAS_SIN_PRUEBA_PORTABLE.contains(&causa) {
                return Err(err(format!(
                    "resolucion declarada con la causa {causa}, que no es de las que el RFC-0007 \
                     dejo sin prueba portable: se exhibe su sobre de rechazo, no se declara"
                )));
            }
            exige_mismo_recibo(d, hash, "resolucion.data")?;
            return Err(format!(
                "{DECLARADA_NO_PROBADA}el operador declara {causa}, una causa que el RFC-0007 \
                 dejo sin prueba portable: se cuenta aparte (RFC-0010, D-G)"
            ));
        }
        Some(otro) => {
            return Err(err(format!(
                "resolucion: tipo {otro:?} desconocido: se lee acuse, rechazo o declarada, y lote o \
                 prenda para los recibos del RFC-0014"
            )))
        }
    }
    println!("VERDE: el recibo se resolvio dentro de la ventana, y se sostiene sin el nodo");
    Ok(())
}

/// Un digest, en el hex del cable (`0x…`, 64 nibbles): lo que el sobre del
/// ancla IMPRIME para que el que comprueba lo compare con el medio.
fn hex_de_digest(d: &Digest) -> String {
    let b = digest_to_bytes(d);
    let mut s = String::with_capacity(2 + 64);
    s.push_str("0x");
    for x in b {
        s.push_str(&format!("{x:02x}"));
    }
    s
}

/// **El sobre del ANCLA** (RFC-0012 E3, §586): una cabeza firmada y, si
/// viajan, el ancla publicada en el medio y lo que la une a la cabeza.
/// Cuatro modos:
///
/// 1. La cabeza SOLA: el mando DERIVA el ancla y su huella y las imprime —
///    el productor de B10.6. Lo impreso es lo que se publica; quien
///    comprueba contra el medio compara la huella byte a byte, porque el
///    mando no tiene red y no puede leer el medio por nadie. VERDE.
/// 2. Con `ancla` y sin `camino`: el ancla ES esta cabeza — los cinco
///    campos iguales, el indice contra el EMBEBIDO en la firma. VERDE.
/// 3. Con `ancla` y `camino`: el ancla es ANTERIOR — misma clave, indice
///    anterior, y la consistencia del MMR (§291, RFC 6962) del lote
///    anclado a la cabeza. VERDE: la historia anclada es un prefijo.
/// 4. Con `contraria`: la VISTA DIVIDIDA — dos cabezas firmadas de la
///    misma clave con el MISMO indice embebido y contenidos distintos.
///    DETECCION con salida 0, el molde del conflicto (§430): el sobre que
///    la exhibe no falla — delata, y solo el operador pudo producirla.
///
/// El indice del ancla es el EMBEBIDO (D-C): el declarado solo esta
/// acotado por abajo (§399), y dos cabezas honestas pueden compartirlo.
fn verificar_ancla(p: &serde_json::Value) -> Result<(), String> {
    // Las reglas de FORMA, antes de tocar la criptografia: que combinacion
    // de claves es un sobre y cual no.
    if p.get("ancla").is_some() && p.get("contraria").is_some() {
        return Err(err(
            "un sobre con contraria no lleva ancla: la vista dividida se demuestra con las \
             dos cabezas solas"
                .into(),
        ));
    }
    if p.get("camino").is_some() && p.get("ancla").is_none() {
        return Err(err("camino sin ancla: no hay nada que extender".into()));
    }
    let c = p
        .get("cabeza")
        .ok_or_else(|| err("falta cabeza (la firmada que el ancla compromete)".into()))?;
    let version = u64_de(c, "formatVersion")?;
    match VersionCabeza::try_from(version) {
        Ok(v) if v.lleva_mmr() => {}
        _ => {
            return Err(err(format!(
                "cabeza: formatVersion {version}: el ancla lee cabezas {}: la pareja del MMR \
                 viaja firmada desde ellas",
                VersionCabeza::texto_con_mmr()
            )))
        }
    }
    let (cima, t, clave_hex, _) = cabeza_v3_verificada(c, "cabeza")?;
    let epoch_digest = digest_de(c, "epochDigest")?;
    let firma = hex_a_bytes(c.get("signature").and_then(|x| x.as_str()).unwrap_or(""))?;
    let embebido = indice_de_firma(&firma).map_err(|e| err(format!("cabeza: {e}")))?;
    let huella_clave = huella_de_clave(&hex_a_bytes(&clave_hex)?);
    println!(
        "1/3 la cabeza (v{version}) recompone su digest y su firma verifica \
         (indice embebido {embebido})"
    );

    match (p.get("ancla"), p.get("contraria")) {
        (None, None) => {
            // Modo 1: DERIVAR. El unico modo sin nada que creer: todo sale
            // de la cabeza verificada.
            let huella = ancla_digest(huella_clave, embebido, epoch_digest, cima, t);
            println!("2/3 el ancla, derivada de la cabeza sola: lo que se publica en el medio");
            println!(
                "   {{ \"v\": 1, \"clave\": \"{}\", \"indice\": \"{:#x}\", \
                 \"epochDigest\": \"{}\", \"mmrRoot\": \"{}\", \"mmrSize\": \"{:#x}\" }}",
                hex_de_digest(&huella_clave),
                embebido,
                hex_de_digest(&epoch_digest),
                hex_de_digest(&cima),
                t
            );
            println!(
                "3/3 la huella del ancla: {} - comparala con el medio: este mando no tiene red",
                hex_de_digest(&huella)
            );
            println!("VERDE: el ancla se deriva de la cabeza firmada, y se sostiene sin el nodo");
            Ok(())
        }
        (Some(a), None) => {
            if a.get("v").and_then(|x| x.as_u64()) != Some(1) {
                return Err(err("el ancla no declara v 1: este binario lee ancla v1".into()));
            }
            let a_clave = digest_de(a, "clave")?;
            if a_clave != huella_clave {
                return Err(err(
                    "el ancla es de OTRA clave: su clave no es la huella de la publicKey de \
                     la cabeza"
                        .into(),
                ));
            }
            let a_indice = u64_de(a, "indice")?;
            let a_digest = digest_de(a, "epochDigest")?;
            let a_root = digest_de(a, "mmrRoot")?;
            let a_size = u64_de(a, "mmrSize")?;
            let huella = ancla_digest(a_clave, a_indice, a_digest, a_root, a_size);
            match p.get("camino") {
                None => {
                    // Modo 2: el ancla ES esta cabeza. Campo a campo, con el
                    // nombre del que no casa: un ancla es una afirmacion
                    // publicada, y el que no casa es EL dato manipulado.
                    for (campo, casa) in [
                        ("indice", a_indice == embebido),
                        ("epochDigest", a_digest == epoch_digest),
                        ("mmrRoot", a_root == cima),
                        ("mmrSize", a_size == t),
                    ] {
                        if !casa {
                            return Err(err(format!(
                                "el ancla no ES esta cabeza: su {campo} no casa"
                            )));
                        }
                    }
                    println!("2/3 los cinco campos del ancla son los de la cabeza firmada");
                    println!("3/3 la huella del ancla: {}", hex_de_digest(&huella));
                    println!("VERDE: el ancla ES esta cabeza firmada, y se sostiene sin el nodo");
                }
                Some(_) => {
                    // Modo 3: el ancla es ANTERIOR y la cabeza la extiende.
                    if a_size == 0 {
                        return Err(err(
                            "el ancla del genesis (mmrSize 0) no tiene historia que extender: \
                             se compara entera, sin camino"
                                .into(),
                        ));
                    }
                    if a_indice >= embebido {
                        return Err(err(format!(
                            "el ancla declara un indice ({a_indice}) que no es ANTERIOR al \
                             embebido de la cabeza ({embebido})"
                        )));
                    }
                    let camino = camino_mmr(p)?;
                    if !zk_ssl_verify::mmr::verificar_consistencia(a_root, a_size, cima, t, &camino)
                    {
                        return Err(err(format!(
                            "la cabeza (t={t}) NO extiende el ancla (t={a_size}): historia \
                             bifurcada, recortada, o camino que no es el suyo"
                        )));
                    }
                    println!(
                        "2/3 la cima de la cabeza EXTIENDE el lote anclado: consistencia \
                         O(log N), sin el registro"
                    );
                    println!("3/3 la huella del ancla: {}", hex_de_digest(&huella));
                    println!(
                        "VERDE: la cabeza extiende el ancla: la historia anclada es un \
                         prefijo, y se sostiene sin el nodo"
                    );
                }
            }
            Ok(())
        }
        (None, Some(otra)) => {
            // Modo 4: la VISTA DIVIDIDA. La contraria se verifica ENTERA,
            // como la cabeza: dos firmas de verdad o no hay delacion.
            let version_o = u64_de(otra, "formatVersion")?;
            match VersionCabeza::try_from(version_o) {
                Ok(v) if v.lleva_mmr() => {}
                _ => {
                    return Err(err(format!(
                        "contraria: formatVersion {version_o}: el ancla lee cabezas {}: la \
                         pareja del MMR viaja firmada desde ellas",
                        VersionCabeza::texto_con_mmr()
                    )))
                }
            }
            let (_, _, clave_o, _) = cabeza_v3_verificada(otra, "contraria")?;
            if clave_o != clave_hex {
                return Err(claves_distintas());
            }
            let firma_o =
                hex_a_bytes(otra.get("signature").and_then(|x| x.as_str()).unwrap_or(""))?;
            let embebido_o = indice_de_firma(&firma_o).map_err(|e| err(format!("contraria: {e}")))?;
            if embebido_o != embebido {
                return Err(err(format!(
                    "los indices embebidos son DISTINTOS ({embebido}, {embebido_o}): dos \
                     firmas con su indice propio no dividen la vista"
                )));
            }
            let digest_o = digest_de(otra, "epochDigest")?;
            if digest_o == epoch_digest && version_o == version {
                return Err(err(
                    "las dos cabezas son LA MISMA: no hay vista que dividir".into(),
                ));
            }
            println!(
                "2/3 la contraria recompone y su firma verifica: misma clave, mismo indice \
                 embebido {embebido}"
            );
            println!(
                "3/3 los contenidos DIFIEREN: dos preambulos bajo un indice de un solo uso"
            );
            println!(
                "VERDE: VISTA DIVIDIDA - la clave firmo DOS cabezas con el indice embebido \
                 {embebido}. Es DETECCION del operador: dos historias, y solo quien tiene la \
                 clave pudo producirlas"
            );
            Ok(())
        }
        (Some(_), Some(_)) => unreachable!("la regla de forma corta antes"),
    }
}

/// Una cadena del sobre, con su nombre al faltar.
fn cadena_de<'a>(p: &'a serde_json::Value, campo: &str, que: &str) -> Result<&'a str, String> {
    p.get(campo)
        .and_then(|x| x.as_str())
        .ok_or_else(|| err(format!("falta {campo} o no es cadena ({que})")))
}

/// Un hash SHA-256 del medio: `0x` y 64 hex, los bytes TAL CUAL. No es un digest de
/// Goldilocks: `digest_de` rechazaria con razon un elemento no canonico, y aqui no hay campo.
fn sha256_de(s: &str, campo: &str) -> Result<[u8; 32], String> {
    let b = hex_a_bytes(s).map_err(|e| err(format!("{campo}: {e}")))?;
    b.as_slice()
        .try_into()
        .map_err(|_| err(format!("{campo}: {} bytes, se esperaban 32", b.len())))
}

/// **El sobre del ANCLA COFIRMADA** (RFC-0013 E4a, §633): una cabeza firmada, la nota del
/// medio que publica su ancla, las claves para leerla y el camino que las une.
///
/// El ancla no viaja: se DERIVA de la cabeza (el modo 1 del sobre del ancla), y por eso ES la
/// cabeza firmada que dice ser, que es el ultimo paso de la D-E. El orden de esa D-E es nota,
/// cofirmas, inclusion y ancla; aqui la cabeza va PRIMERO, porque el `origin` de la nota se
/// deriva de su clave XMSS (D-A) y sin ella no hay medio con que comparar.
///
/// ⚠️ **REPORTA, NO JUZGA** (decidido en el §633, como el paquete v2 con sus cofirmas): lista
/// los testigos cuya cofirma verifica con las claves que trae el sobre, cada uno con la huella
/// SHA-256 de su clave ENTERA —el `key_id` de 4 bytes de la nota es un identificador, no una
/// garantia, y se fabrica—. Que testigos valen y cuantos hacen falta lo decide quien verifica
/// con su politica (D-D): quien arma el sobre puede ser el operador. Una cofirma de una clave
/// del sobre que NO verifica es ROJO, como pide `signed-note`; una linea sin clave en el
/// sobre se cuenta y no se juzga.
fn verificar_ancla_cofirmada(p: &serde_json::Value) -> Result<(), String> {
    // Las reglas de FORMA, antes de tocar la criptografia.
    for ajeno in ["ancla", "camino", "contraria"] {
        if p.get(ajeno).is_some() {
            return Err(err(format!(
                "el sobre del ancla cofirmada no lleva {ajeno}: el ancla se deriva de la \
                 cabeza, y la extension y la vista dividida son del sobre del ancla"
            )));
        }
    }
    let c = p
        .get("cabeza")
        .ok_or_else(|| err("falta cabeza (la firmada cuya ancla se publico)".into()))?;
    let nota = cadena_de(p, "nota", "la nota checkpoint del medio, entera")?;
    let vkey_publicador = cadena_de(p, "publicador", "la vkey del publicador del medio")?;
    let posicion = u64_de(p, "posicion")?;
    let inclusion: Vec<[u8; 32]> = p
        .get("inclusion")
        .and_then(|x| x.as_array())
        .ok_or_else(|| err("falta inclusion o no es lista (el camino del ancla a la raiz)".into()))?
        .iter()
        .map(|x| sha256_de(x.as_str().unwrap_or(""), "inclusion"))
        .collect::<Result<_, _>>()?;
    let testigos: Vec<ClaveDeNota> = match p.get("testigos") {
        None => Vec::new(),
        Some(t) => t
            .as_array()
            .ok_or_else(|| err("testigos no es lista (las vkeys de los testigos)".into()))?
            .iter()
            .map(|x| {
                ClaveDeNota::leer_vkey(x.as_str().unwrap_or(""))
                    .map_err(|e| err(format!("testigos: {e}")))
            })
            .collect::<Result<_, _>>()?,
    };
    for (i, a) in testigos.iter().enumerate() {
        if testigos[..i]
            .iter()
            .any(|b| b.nombre() == a.nombre() && b.id() == a.id())
        {
            return Err(err(format!("testigos: {} esta dos veces", a.nombre())));
        }
    }

    // 1. La cabeza, ENTERA, como en el sobre del ancla.
    let version = u64_de(c, "formatVersion")?;
    match VersionCabeza::try_from(version) {
        Ok(v) if v.lleva_mmr() => {}
        _ => {
            return Err(err(format!(
                "cabeza: formatVersion {version}: el ancla lee cabezas {}: la pareja del MMR \
                 viaja firmada desde ellas",
                VersionCabeza::texto_con_mmr()
            )))
        }
    }
    let (cima, t, clave_hex, _) = cabeza_v3_verificada(c, "cabeza")?;
    let epoch_digest = digest_de(c, "epochDigest")?;
    let firma = hex_a_bytes(c.get("signature").and_then(|x| x.as_str()).unwrap_or(""))?;
    let embebido = indice_de_firma(&firma).map_err(|e| err(format!("cabeza: {e}")))?;
    let huella_clave = huella_de_clave(&hex_a_bytes(&clave_hex)?);
    let huella_ancla = ancla_digest(huella_clave, embebido, epoch_digest, cima, t);
    println!(
        "1/4 la cabeza (v{version}) recompone su digest y su firma verifica (indice embebido \
         {embebido}); su ancla: {}",
        hex_de_digest(&huella_ancla)
    );

    // 2. La nota: del medio de ESTA clave, y firmada por su publicador.
    let origen = origen_del_medio(&digest_to_bytes(&huella_clave));
    let publicador =
        ClaveDeNota::leer_vkey(vkey_publicador).map_err(|e| err(format!("publicador: {e}")))?;
    if publicador.nombre() != origen {
        return Err(err(format!(
            "el publicador es {:?} y el medio de esta cabeza es {origen:?}: el origin lleva la \
             huella de la clave XMSS (RFC-0013 D-A)",
            publicador.nombre()
        )));
    }
    let v = verificar_nota(nota, &publicador).map_err(|e| err(format!("la nota: {e}")))?;
    println!(
        "2/4 la nota del medio verifica con su publicador (ML-DSA-44, tipo 0x06): {} anclas, \
         marca {}",
        v.checkpoint.tamano, v.marca
    );

    // 3. Las cofirmas: las que tienen clave en el sobre se verifican; las demas se cuentan.
    let mut cofirman: Vec<(&ClaveDeNota, u64)> = Vec::new();
    let mut sin_clave = 0;
    for linea in &v.ajenas {
        match testigos
            .iter()
            .find(|k| k.nombre() == linea.nombre && k.id() == linea.id)
        {
            None => sin_clave += 1,
            Some(k) => {
                if cofirman.iter().any(|(o, _)| std::ptr::eq(*o, k)) {
                    return Err(err(format!(
                        "la nota lleva dos cofirmas de {}",
                        k.nombre()
                    )));
                }
                let marca = k.verificar_cofirma(&v.checkpoint, linea).map_err(|e| {
                    err(format!("la cofirma de {} no verifica: {e}", k.nombre()))
                })?;
                cofirman.push((k, marca));
            }
        }
    }
    println!(
        "3/4 cofirmas que verifican con las claves del sobre: {}; lineas sin clave en el sobre, \
         sin juzgar: {sin_clave}; testigos del sobre sin cofirma: {}",
        cofirman.len(),
        testigos.len() - cofirman.len()
    );
    for (k, marca) in &cofirman {
        let h: String = sha256(k.bytes()).iter().map(|b| format!("{b:02x}")).collect();
        println!("   testigo {} clave sha256:{h} marca {marca}", k.nombre());
    }

    // 4. La inclusion: el ancla de ESTA cabeza esta en la raiz de la nota, donde dice.
    let tamano = v.checkpoint.tamano;
    verificar_inclusion(
        &hoja(&digest_to_bytes(&huella_ancla)),
        posicion,
        tamano,
        &inclusion,
        &v.checkpoint.raiz,
    )
    .map_err(|e| {
        err(format!(
            "el ancla de esta cabeza no esta en la posicion {posicion} de las {tamano} de la \
             nota: {e}"
        ))
    })?;
    println!("4/4 el ancla esta en la posicion {posicion} de las {tamano} anclas de la nota");
    println!(
        "VERDE: la cabeza estaba publicada en el medio {origen}, en la posicion {posicion}, y la \
         cofirman {} testigo(s) con clave en el sobre. Que testigos valen y cuantos hacen falta \
         lo decide quien verifica (RFC-0013 D-D): este mando reporta, no juzga",
        cofirman.len()
    );
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
    let r = correr(&ruta);
    match &r {
        Ok(()) => {}
        // §573: el cuarto estado se imprime con su nombre, sin «ROJO»: no lo es.
        Err(e) if codigo_de_salida(&r) == 3 => eprintln!("{e}"),
        Err(e) => eprintln!("ROJO: {e}"),
    }
    ExitCode::from(codigo_de_salida(&r))
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

    /// §573 · el `tipo` completitud se despacha, y sin cierre se rechaza con su nombre.
    #[test]
    fn el_sobre_de_completitud_se_despacha_y_sin_cierre_lo_dice() {
        let e = verificar_paquete(&json!({ "v": 1, "tipo": "completitud" })).unwrap_err();
        assert!(e.starts_with("falta cierre"), "{e}");
    }

    /// §573 · el cierre tiene que llevar la pareja de recepcion: una v5 se rechaza ANTES de tocar
    /// la criptografia, con el texto DERIVADO del conjunto.
    #[test]
    fn un_cierre_sin_la_pareja_de_recepcion_no_cierra_nada() {
        let e = verificar_completitud(&json!({
            "v": 1, "tipo": "completitud", "cierre": { "formatVersion": "0x5" }
        }))
        .unwrap_err();
        assert!(e.contains("exige una cabeza v6"), "{e}");
        let e = verificar_completitud(&json!({
            "v": 1, "tipo": "completitud", "cierre": { "formatVersion": "0x9" }
        }))
        .unwrap_err();
        assert!(e.contains("formatVersion 9"), "{e}");
    }

    /// §586 · el `tipo` ancla se despacha, y sin cabeza se rechaza con su nombre.
    #[test]
    fn el_sobre_del_ancla_se_despacha_y_sin_cabeza_lo_dice() {
        let e = verificar_paquete(&json!({ "v": 1, "tipo": "ancla" })).unwrap_err();
        assert!(e.starts_with("falta cabeza"), "{e}");
    }

    /// §586 · el ancla exige la pareja del MMR: una v2 y una version desconocida caen ANTES de
    /// tocar la criptografia, con el texto DERIVADO del conjunto.
    #[test]
    fn un_ancla_sin_la_pareja_del_mmr_no_ancla_nada() {
        let e = verificar_ancla(&json!({
            "v": 1, "tipo": "ancla", "cabeza": { "formatVersion": "0x2" }
        }))
        .unwrap_err();
        assert!(e.contains("el ancla lee cabezas"), "{e}");
        let e = verificar_ancla(&json!({
            "v": 1, "tipo": "ancla", "cabeza": { "formatVersion": "0x9" }
        }))
        .unwrap_err();
        assert!(e.contains("formatVersion 9"), "{e}");
    }

    /// §586 · las dos reglas de forma cortan antes que la criptografia: contraria y ancla no
    /// conviven, y un camino sin ancla no tiene nada que extender.
    #[test]
    fn la_forma_del_sobre_del_ancla_corta_antes_de_creer_nada() {
        let e = verificar_ancla(&json!({
            "v": 1, "tipo": "ancla", "cabeza": {}, "ancla": {}, "contraria": {}
        }))
        .unwrap_err();
        assert!(e.starts_with("un sobre con contraria no lleva ancla"), "{e}");
        let e = verificar_ancla(&json!({
            "v": 1, "tipo": "ancla", "cabeza": {}, "camino": []
        }))
        .unwrap_err();
        assert!(e.starts_with("camino sin ancla"), "{e}");
    }

    /// §586 · el desconocido enumera el brazo nuevo: el texto del catalogo lleva `tipo: "ancla"`.
    #[test]
    fn el_tipo_desconocido_enumera_el_ancla() {
        let e = verificar_paquete(&json!({ "v": 1, "tipo": "otra" })).unwrap_err();
        assert!(e.contains("`tipo: \"ancla\"`"), "{e}");
    }

    /// §633 · el desconocido enumera el brazo nuevo.
    #[test]
    fn el_tipo_desconocido_enumera_el_ancla_cofirmada() {
        let e = verificar_paquete(&json!({ "v": 1, "tipo": "otra" })).unwrap_err();
        assert!(e.contains("`tipo: \"ancla-cofirmada\"`"), "{e}");
    }

    /// §633 · la FORMA del ancla cofirmada corta antes de tocar la criptografia: las claves del
    /// sobre del ancla no van aqui, y cada campo que falta se nombra.
    #[test]
    fn la_forma_del_ancla_cofirmada_corta_antes_de_creer_nada() {
        let e = verificar_paquete(&json!({ "v": 1, "tipo": "ancla-cofirmada", "ancla": {} }))
            .unwrap_err();
        assert!(e.starts_with("el sobre del ancla cofirmada no lleva ancla"), "{e}");
        let e = verificar_paquete(&json!({ "v": 1, "tipo": "ancla-cofirmada" })).unwrap_err();
        assert!(e.starts_with("falta cabeza"), "{e}");
        for (sobre, falta) in [
            (json!({ "v": 1, "tipo": "ancla-cofirmada", "cabeza": {} }), "falta nota"),
            (
                json!({ "v": 1, "tipo": "ancla-cofirmada", "cabeza": {}, "nota": "x" }),
                "falta publicador",
            ),
            (
                json!({ "v": 1, "tipo": "ancla-cofirmada", "cabeza": {}, "nota": "x",
                        "publicador": "x" }),
                "falta posicion",
            ),
            (
                json!({ "v": 1, "tipo": "ancla-cofirmada", "cabeza": {}, "nota": "x",
                        "publicador": "x", "posicion": "0x0" }),
                "falta inclusion",
            ),
            (
                json!({ "v": 1, "tipo": "ancla-cofirmada", "cabeza": {}, "nota": "x",
                        "publicador": "x", "posicion": "0x0", "inclusion": ["0x00"] }),
                "inclusion: 1 bytes",
            ),
            (
                json!({ "v": 1, "tipo": "ancla-cofirmada", "cabeza": {}, "nota": "x",
                        "publicador": "x", "posicion": "0x0", "inclusion": [],
                        "testigos": ["no es una vkey"] }),
                "testigos: clave",
            ),
        ] {
            let e = verificar_paquete(&sobre).unwrap_err();
            assert!(e.starts_with(falta), "{falta}: {e}");
        }
    }

    /// Lo que arma un sobre de ancla cofirmada con la cabeza firmada DE VERDAD de
    /// `spec/vectors/ancla/ancla-exacta.json`: su ancla en la posicion 1 de un medio de tres,
    /// la nota de su publicador y la cofirma de un testigo, con claves de prueba.
    mod cofirmada {
        use super::*;
        use zk_ssl_medio::medio::ArbolDelMedio;
        use zk_ssl_medio::nota::{Checkpoint, Cofirmante, Publicador};

        pub const TESTIGO: &str = "testigo.invalid/ajeno";

        pub fn cabeza() -> serde_json::Value {
            let v: serde_json::Value = serde_json::from_str(include_str!(
                "../../../spec/vectors/ancla/ancla-exacta.json"
            ))
            .unwrap();
            v["cabeza"].clone()
        }

        /// La huella del ancla de la cabeza, por el mismo camino que el sobre del ancla.
        pub fn huella_del_ancla(c: &serde_json::Value) -> [u8; 32] {
            let (cima, t, clave_hex, _) = cabeza_v3_verificada(c, "cabeza").unwrap();
            let firma = hex_a_bytes(c["signature"].as_str().unwrap()).unwrap();
            let h = ancla_digest(
                huella_de_clave(&hex_a_bytes(&clave_hex).unwrap()),
                indice_de_firma(&firma).unwrap(),
                digest_de(c, "epochDigest").unwrap(),
                cima,
                t,
            );
            digest_to_bytes(&h)
        }

        pub fn origen(c: &serde_json::Value) -> String {
            let (_, _, clave_hex, _) = cabeza_v3_verificada(c, "cabeza").unwrap();
            origen_del_medio(&digest_to_bytes(&huella_de_clave(&hex_a_bytes(&clave_hex).unwrap())))
        }

        pub fn hex(b: &[u8]) -> String {
            format!("0x{}", b.iter().map(|x| format!("{x:02x}")).collect::<String>())
        }

        /// `(sobre, arbol, checkpoint)`: el medio tiene tres anclas y la de la cabeza es la 1.
        pub fn sobre() -> (serde_json::Value, ArbolDelMedio, Checkpoint) {
            let c = cabeza();
            let mut arbol = ArbolDelMedio::nuevo();
            arbol.anadir(&[0x11; 32]);
            arbol.anadir(&huella_del_ancla(&c));
            arbol.anadir(&[0x33; 32]);
            let checkpoint = Checkpoint {
                origen: origen(&c),
                tamano: 3,
                raiz: arbol.raiz(),
            };
            let publicador = Publicador::determinista(&checkpoint.origen, [1; 32]).unwrap();
            let testigo = Cofirmante::determinista(TESTIGO, [2; 32]).unwrap();
            let nota = publicador.firmar(&checkpoint, 1_790_000_000).unwrap()
                + &testigo.cofirmar(&checkpoint, 1_790_000_060).unwrap();
            let inclusion: Vec<String> = arbol
                .prueba_de_inclusion(1, 3)
                .unwrap()
                .iter()
                .map(|h| hex(h))
                .collect();
            let sobre = json!({
                "v": 1, "tipo": "ancla-cofirmada", "cabeza": c, "nota": nota,
                "publicador": publicador.clave_publica().vkey(),
                "testigos": [testigo.clave_publica().vkey()],
                "posicion": "0x1", "inclusion": inclusion,
            });
            (sobre, arbol, checkpoint)
        }
    }

    /// §633 · el ancla cofirmada de una cabeza firmada de verdad: VERDE con la cofirma del
    /// testigo, y VERDE tambien sin su clave en el sobre —la linea se cuenta y no se juzga:
    /// el mando reporta, no juzga—.
    #[test]
    fn el_ancla_cofirmada_de_una_cabeza_real() {
        let (sobre, _, _) = cofirmada::sobre();
        verificar_paquete(&sobre).unwrap();
        let mut sin_testigos = sobre.clone();
        sin_testigos.as_object_mut().unwrap().remove("testigos");
        verificar_paquete(&sin_testigos).unwrap();
    }

    /// §633 · cada mentira cae por su nombre: la posicion, el camino, la cofirma tocada, el
    /// publicador de otro medio, el testigo dos veces y una nota de un medio sin el ancla.
    #[test]
    fn el_ancla_cofirmada_rechaza_cada_mentira_por_su_nombre() {
        use zk_ssl_medio::medio::ArbolDelMedio;
        use zk_ssl_medio::nota::{Checkpoint, Publicador};
        let (sobre, _, checkpoint) = cofirmada::sobre();
        let rojo = |s: &serde_json::Value| verificar_paquete(s).unwrap_err();

        let mut s = sobre.clone();
        s["posicion"] = json!("0x2");
        assert!(rojo(&s).contains("no esta en la posicion 2"), "{}", rojo(&s));

        let mut s = sobre.clone();
        s["inclusion"][0] = json!(cofirmada::hex(&[0x44; 32]));
        assert!(rojo(&s).contains("no esta en la posicion 1"), "{}", rojo(&s));

        // La cofirma, con un caracter de su base64 cambiado.
        let mut s = sobre.clone();
        let nota = s["nota"].as_str().unwrap().to_string();
        let i = nota.rfind(cofirmada::TESTIGO).unwrap() + cofirmada::TESTIGO.len() + 100;
        let c = if &nota[i..i + 1] == "A" { "B" } else { "A" };
        s["nota"] = json!(format!("{}{c}{}", &nota[..i], &nota[i + 1..]));
        assert!(rojo(&s).starts_with("la cofirma de testigo.invalid/ajeno no verifica"), "{}", rojo(&s));

        // El ataque de verdad: el ancla publicada en el medio de OTRA clave, bien firmado por
        // el publicador de ese medio y con el ancla dentro. Todo verifica menos el atado.
        let ajeno = Checkpoint {
            origen: "zkssl/v1/00".into(),
            ..checkpoint.clone()
        };
        let suyo = Publicador::determinista(&ajeno.origen, [9; 32]).unwrap();
        let mut s = sobre.clone();
        s["nota"] = json!(suyo.firmar(&ajeno, 1_790_000_000).unwrap());
        s["publicador"] = json!(suyo.clave_publica().vkey());
        assert!(rojo(&s).starts_with("el publicador es \"zkssl/v1/00\""), "{}", rojo(&s));

        let mut s = sobre.clone();
        let otro = Publicador::determinista("zkssl/v1/00", [1; 32]).unwrap();
        s["publicador"] = json!(otro.clave_publica().vkey());
        assert!(rojo(&s).starts_with("el publicador es \"zkssl/v1/00\""), "{}", rojo(&s));

        let mut s = sobre.clone();
        let t = s["testigos"][0].clone();
        s["testigos"] = json!([t.clone(), t]);
        assert!(rojo(&s).contains("esta dos veces"), "{}", rojo(&s));

        // Un medio de tres anclas en el que la de la cabeza NO esta, bien firmado.
        let mut ajeno = ArbolDelMedio::nuevo();
        for b in [0x11, 0x22, 0x33] {
            ajeno.anadir(&[b; 32]);
        }
        let ck = Checkpoint { raiz: ajeno.raiz(), ..checkpoint };
        let mut s = sobre.clone();
        s["nota"] = json!(Publicador::determinista(&ck.origen, [1; 32])
            .unwrap()
            .firmar(&ck, 1_790_000_000)
            .unwrap());
        s["inclusion"] = json!(ajeno
            .prueba_de_inclusion(1, 3)
            .unwrap()
            .iter()
            .map(|h| cofirmada::hex(h))
            .collect::<Vec<_>>());
        assert!(rojo(&s).contains("no esta en la posicion 1"), "{}", rojo(&s));
    }

    /// §573 · la grieta de la D-G, nombrada: las cuatro causas que el RFC-0007 dejo sin prueba
    /// portable, escritas como literales y no derivadas del propio `match`.
    #[test]
    fn las_causas_sin_prueba_portable_son_las_cuatro_del_rfc_0007() {
        assert_eq!(
            CAUSAS_SIN_PRUEBA_PORTABLE,
            ["CustodianSetExhausted", "PendingTreeExhausted", "NotTheIssuer", "NotTheAccountHolder"]
        );
    }

    /// §573 · D4 del autor: el cuarto estado se cuenta APARTE, con su propio codigo.
    #[test]
    fn el_cuarto_estado_sale_con_el_tres_y_el_resto_no() {
        assert_eq!(codigo_de_salida(&Ok(())), 0);
        assert_eq!(codigo_de_salida(&Err(format!("{DECLARADA_NO_PROBADA}x"))), 3);
        assert_eq!(codigo_de_salida(&Err("NO RESUELTA EN LA VENTANA: x".into())), 1);
        assert_eq!(codigo_de_salida(&Err("falta cierre".into())), 1);
    }

    /// §573 · D3 del autor: el rechazo se ata al recibo por su `data.recepcion`, y de otra
    /// operacion NO se ata.
    #[test]
    fn un_rechazo_se_ata_al_recibo_por_su_data_y_de_otra_operacion_no() {
        let h = dg(70);
        let hex = |d: &Digest| {
            let b = zk_ssl_hash::digest_to_bytes(d);
            format!("0x{}", b.iter().map(|x| format!("{x:02x}")).collect::<String>())
        };
        let data = json!({ "causa": "X", "recepcion": { "hashPrueba": hex(&h) } });
        assert_eq!(exige_mismo_recibo(&data, h, "r"), Ok(()));
        let e = exige_mismo_recibo(&data, dg(71), "r").unwrap_err();
        assert!(e.contains("OTRA operacion"), "{e}");
        let e = exige_mismo_recibo(&json!({ "causa": "X" }), h, "r").unwrap_err();
        assert!(e.contains("no lleva recepcion"), "{e}");
    }

    /// §612 (RFC-0014 D-B): la composicion de un lote se lee ENTERA, nombrando el sitio de lo
    /// que falta; vacia no es un lote; y leida, recompone con la MISMA funcion que el nodo anota.
    #[test]
    fn la_composicion_de_un_lote_se_lee_entera_y_vacia_no_es_un_lote() {
        let hex = |d: &Digest| {
            let b = zk_ssl_hash::digest_to_bytes(d);
            format!("0x{}", b.iter().map(|x| format!("{x:02x}")).collect::<String>())
        };
        let e = composicion_de(&json!({})).unwrap_err();
        assert!(e.contains("falta composicion"), "{e}");
        let e = composicion_de(&json!({ "composicion": [] })).unwrap_err();
        assert!(e.contains("VACIA"), "{e}");
        let bien = json!({ "hashPrueba": hex(&dg(80)), "cuenta": "0x3", "posicion": "0x7" });
        let sin_cuenta = json!({ "hashPrueba": hex(&dg(81)), "posicion": "0x8" });
        let e = composicion_de(&json!({ "composicion": [bien, sin_cuenta] })).unwrap_err();
        assert!(e.contains("resolucion.composicion[1]: falta cuenta"), "{e}");
        let otra = json!({ "hashPrueba": hex(&dg(81)), "cuenta": "0x4", "posicion": "0x8" });
        let c = composicion_de(&json!({ "composicion": [bien, otra] })).expect("se lee");
        assert_eq!(c, vec![(dg(80), 3, 7), (dg(81), 4, 8)]);
        assert_eq!(
            recibos::hash_del_lote(&c),
            zk_ssl_hash::hash_del_lote(&[(dg(80), 3, 7), (dg(81), 4, 8)]),
            "la huella del mando es la del nucleo"
        );
    }

    /// §612 (RFC-0014 D-B, y la regla de la decision 3 del §609): la FORMA se juzga otra vez con la
    /// composicion sola. Lo que la sostiene devuelve la primera del par; lo que no -otro valor en
    /// la operacion nombrada, o ninguna anterior que lo repita- vuelve como juicio contradicho, que
    /// quien llama nombra «RECHAZO SIN FUNDAMENTO». Y la operacion nombrada tiene que estar en el lote.
    #[test]
    fn la_forma_del_lote_se_juzga_otra_vez_y_la_que_no_se_sostiene_vuelve_contradicha() {
        let c = [(dg(1), 5, 10), (dg(2), 6, 11), (dg(3), 5, 12), (dg(4), 7, 11)];
        assert_eq!(campo_de_forma("DuplicateAccountInBatch"), Some("index"));
        assert_eq!(campo_de_forma("DuplicatePendingInBatch"), Some("position"));
        assert_eq!(campo_de_forma("StaleState"), None);
        assert_eq!(juzgar_forma(&c, 2, "DuplicateAccountInBatch", 5), Ok(0));
        assert_eq!(juzgar_forma(&c, 3, "DuplicatePendingInBatch", 11), Ok(1));
        let e = juzgar_forma(&c, 2, "DuplicateAccountInBatch", 6).unwrap_err();
        assert!(e.contains("lleva la cuenta 5 y la causa nombra la 6"), "{e}");
        let e = juzgar_forma(&c, 1, "DuplicateAccountInBatch", 6).unwrap_err();
        assert!(e.contains("ninguna operacion anterior a la 1 lleva la cuenta 6"), "{e}");
        let e = juzgar_forma(&c, 2, "DuplicatePendingInBatch", 12).unwrap_err();
        assert!(e.contains("ninguna operacion anterior a la 2 lleva la posicion 12"), "{e}");
        let e = operacion_nombrada(&json!({ "causa": "X" }), 4, "r").unwrap_err();
        assert!(e.contains("nombra su operacion"), "{e}");
        let e = operacion_nombrada(&json!({ "operacion": "0x4" }), 4, "r").unwrap_err();
        assert!(e.contains("la operacion 4 no esta en un lote de 4"), "{e}");
        assert_eq!(operacion_nombrada(&json!({ "operacion": "0x3" }), 4, "r"), Ok(3));
    }

    /// §613 (RFC-0014 D-E): la resolucion de una prenda se ata al recibo por el digest de SU
    /// prueba -el de la casa, §116- antes de nada; lleva UNA de tres; y la negativa es una
    /// negativa, atada al recibo, sin causa -la que la lleva se exhibe como rechazo- y con la
    /// cabeza que el nodo juzgo. Todo esto se decide sin tocar una cabeza.
    #[test]
    fn la_resolucion_de_una_prenda_se_ata_por_su_prueba_y_lleva_una_de_tres() {
        let hex = |d: &Digest| {
            let b = zk_ssl_hash::digest_to_bytes(d);
            format!("0x{}", b.iter().map(|x| format!("{x:02x}")).collect::<String>())
        };
        let prueba = [1u8, 2, 3, 4];
        let h = zk_ssl_hash::digest_of_proof(&prueba);
        let sobre = json!({ "v": 1, "tipo": "prenda", "prueba": "0x01020304",
                            "enunciado": { "receptor": hex(&dg(1)), "marca": hex(&dg(2)) } });
        let rec = json!({ "hashPrueba": hex(&h) });
        let r = |x: serde_json::Value| resolver_prenda(&x, h, 5, 1440, "0xaa").unwrap_err();
        let casos = [
            (json!({}), "falta sobre (el de la prenda"),
            (json!({ "sobre": { "tipo": "rechazo" } }), "el sobre no es de tipo prenda"),
            (json!({ "sobre": { "tipo": "prenda" } }), "resolucion.sobre: falta enunciado"),
            (json!({ "sobre": sobre }), "lleva UNA de tres"),
            (json!({ "sobre": sobre, "consumo": {}, "respuesta": {} }), "lleva UNA de tres"),
            (json!({ "sobre": sobre, "respuesta": { "accepted": true } }), "no es una negativa"),
            (json!({ "sobre": sobre, "respuesta": { "accepted": false } }), "no lleva recepcion"),
            (json!({ "sobre": sobre, "respuesta": { "accepted": false, "recepcion": { "hashPrueba": hex(&dg(9)) } } }),
             "es de OTRA operacion"),
            (json!({ "sobre": sobre, "respuesta": { "accepted": false, "recepcion": rec, "data": {} } }),
             "la negativa lleva causa"),
            (json!({ "sobre": sobre, "respuesta": { "accepted": false, "recepcion": rec } }), "falta juzgada"),
        ];
        for (x, texto) in casos {
            let e = r(x);
            assert!(e.contains(texto), "se esperaba «{texto}» y salio: {e}");
        }
        let e = resolver_prenda(&json!({ "sobre": sobre }), dg(9), 5, 1440, "0xaa").unwrap_err();
        assert!(e.contains("la prenda es de OTRA prueba"), "{e}");
    }

    /// Un paquete sin la clave no es un error: es un paquete sin cofirmas.
    fn dg(n: u64) -> Digest {
        [
            BaseElement::new(n),
            BaseElement::new(n + 1),
            BaseElement::new(n + 2),
            BaseElement::new(n + 3),
        ]
    }

    /// §566 · El acuse de una cabeza v6 se recompone con la v6. FALSADOR del
    /// brazo viejo: sin la pareja de recepcion -que es por donde el paso 3 la
    /// mandaba a la v5- el MISMO acuse contra el MISMO digest cae.
    #[test]
    fn el_acuse_de_una_cabeza_v6_se_recompone_con_la_v6() {
        let hoja = dg(40);
        let recibo = ReciboAcuse {
            hoja,
            hermanos: Vec::new(),
            derecha: Vec::new(),
            seq: 9,
            accounts_root: dg(1),
            pending_root: dg(2),
            frozen_root: dg(3),
            chain_digest: dg(4),
            acuses_root: hoja,
            n: 5,
        };
        let f = FamiliaV5 {
            params_digest: dg(110),
            pmeta_root: dg(120),
            next_pending: 7,
            next_index: 8,
            total_supply: 9,
        };
        let (cima, cons, recep) = (dg(60), dg(90), dg(130));
        let firmado = epoch_digest_v6(
            9, dg(1), dg(2), dg(3), dg(4), hoja, 5, cima, 2, cons, 1, f.params_digest,
            f.pmeta_root, 7, 8, 9, recep, 3,
        );
        let mmr = Some((cima, 2));
        let cn = Some((cons, 1));
        acuse_contra_cabeza(&recibo, mmr, cn, Some(f), Some((recep, 3)), firmado)
            .expect("el acuse de una v6 debe verificar con la v6");
        let viejo = acuse_contra_cabeza(&recibo, mmr, cn, Some(f), None, firmado);
        assert!(
            viejo.as_ref().is_err_and(|e| e.contains("CabezaDistinta")),
            "el brazo de la v5 no puede aceptar un digest v6: {viejo:?}"
        );
    }

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

    // RFC-0006 E4a. Estos cuatro falsan la FORMA de `libros`, que es lo unico
    // del conflicto alcanzable sin firmas validas: la regla de las claves y la
    // de los caminos exigen dos cabezas reales de dos operadores, y aqui la
    // firma es de mentira a proposito (ver la cabecera de este modulo). Su
    // testigo vive en `tools/banco_dos_libros.sh`, que las produce de verdad.

    /// Sin `libros` no hay conflicto que juzgar, y se dice con su nombre.
    #[test]
    fn un_conflicto_sin_libros_se_nombra() {
        let p = json!({ "v": 1, "tipo": "conflicto" });
        assert_eq!(verificar_conflicto(&p), Err("falta libros".into()));
    }

    /// Y si esta pero no es una lista, tampoco: el fallo nombra la clave.
    #[test]
    fn unos_libros_que_no_son_lista_se_nombran() {
        let p = json!({ "v": 1, "tipo": "conflicto", "libros": 3 });
        assert_eq!(verificar_conflicto(&p), Err("libros no es una lista".into()));
    }

    /// Con UNO no hay conflicto: un consumo bajo una sola cabeza es el caso
    /// normal, y confundirlo con un conflicto seria acusar sin prueba.
    #[test]
    fn un_solo_libro_se_rechaza_con_su_cuenta() {
        let p = json!({ "v": 1, "tipo": "conflicto", "libros": [{}] });
        let e = verificar_conflicto(&p).unwrap_err();
        assert!(e.contains("exige DOS libros: se recibieron 1"), "{e}");
    }

    /// Y con TRES tampoco: el sobre dice DOS, y la puerta cierra por los dos
    /// lados. Un `>= 2` dejaria entrar un sobre cuya afirmacion nadie escribio.
    #[test]
    fn tres_libros_tambien_se_rechazan() {
        let p = json!({ "v": 1, "tipo": "conflicto", "libros": [{}, {}, {}] });
        let e = verificar_conflicto(&p).unwrap_err();
        assert!(e.contains("exige DOS libros: se recibieron 3"), "{e}");
    }
    /// RFC-0007 E1a (§451): una cabeza v5 sin una de las cinco piezas se rechaza NOMBRANDO
    /// la clave que falta, antes de tocar la firma; y con las cinco, la version elige el
    /// recomponedor v5: lo que falla entonces es la firma de mentira, no la recomposicion.
    #[test]
    fn una_cabeza_v5_sin_una_de_las_cinco_piezas_se_nombra_y_con_ellas_recompone() {
        use zk_ssl_hash::{as_digest, digest_to_bytes};
        fn hx(d: zk_ssl_hash::Digest) -> String {
            let cuerpo: String = digest_to_bytes(&d).iter().map(|b| format!("{b:02x}")).collect();
            format!("0x{cuerpo}")
        }
        let d = as_digest(7);
        let ed = zk_ssl_hash::epoch_digest_v5(1, d, d, d, d, d, 5, d, 2, d, 3, d, d, 4, 6, 8);
        let c = json!({
            "available": true, "formatVersion": "0x5", "seq": "0x1", "n": "0x5",
            "accountsRoot": hx(d), "pendingRoot": hx(d), "frozenRoot": hx(d),
            "chainDigest": hx(d), "acusesRoot": hx(d), "mmrRoot": hx(d), "mmrSize": "0x2",
            "consRoot": hx(d), "consCount": "0x3", "paramsDigest": hx(d), "pmetaRoot": hx(d),
            "nextPending": "0x4", "nextIndex": "0x6", "totalSupply": "0x8",
            "epochDigest": hx(ed), "index": "0x1", "publicKey": "0xaa", "signature": "0xbb"
        });
        let ruta = std::env::temp_dir().join("zk-ssl-verify-e1a-v5.json");
        std::fs::write(&ruta, json!({ "v": 1, "cabeza": c.clone() }).to_string()).unwrap();
        let e = correr(ruta.to_str().unwrap()).unwrap_err();
        assert!(e.starts_with("cabeza: "), "con las cinco piezas recompone y cae en la firma: {e}");
        for k in ["paramsDigest", "pmetaRoot", "nextPending", "nextIndex", "totalSupply"] {
            let mut sin = c.clone();
            let _ = sin.as_object_mut().expect("objeto").remove(k);
            std::fs::write(&ruta, json!({ "v": 1, "cabeza": sin }).to_string()).unwrap();
            let e = correr(ruta.to_str().unwrap()).unwrap_err();
            assert!(e.contains(&format!("falta {k}")), "{k}: {e}");
        }
        let _ = std::fs::remove_file(&ruta);
    }

    // RFC-0007 E4b-2 (S465). Como en el conflicto, lo alcanzable sin una cabeza firmada es la
    // FORMA del sobre y la VERSION de la cabeza, que se juzga antes de la firma. La regla que
    // enlaza la prueba a la cabeza tiene sus testigos con pruebas reales en `stark-experiment` y
    // en la capa; el positivo de punta a punta por este mando, con una cabeza de un nodo, es E4b-3.

    /// Sin `enunciado` no hay afirmacion que juzgar, y se dice con su nombre.
    #[test]
    fn un_sobre_de_edad_sin_enunciado_se_nombra() {
        let p = json!({ "v": 1, "tipo": "edad" });
        assert_eq!(verificar_edad(&p), Err("falta enunciado".into()));
    }

    /// Un `emisor` presente tiene que ser una cantidad: su AUSENCIA es la que dice «todos».
    #[test]
    fn un_emisor_que_no_es_una_cantidad_se_nombra() {
        let en = json!({ "t": "0x1", "k": "0x0", "emisor": 7 });
        let p = json!({ "v": 1, "tipo": "edad", "enunciado": en });
        let e = verificar_edad(&p).unwrap_err();
        assert!(e.contains("falta emisor o no es cadena 0x"), "{e}");
    }

    /// Una cabeza que no es v5 se rechaza por su VERSION, antes de tocar la firma: solo la v5
    /// firma `pmetaRoot` y `nextPending`.
    #[test]
    fn una_cabeza_que_no_es_v5_se_rechaza_antes_de_la_firma() {
        let p = json!({
            "v": 1, "tipo": "edad",
            "enunciado": { "t": "0x1", "k": "0x0" },
            "subraices": { "pendientes": DIG, "meta": DIG },
            "prueba": "0x00",
            "cabeza": { "available": true, "formatVersion": "0x4" }
        });
        let e = verificar_edad(&p).unwrap_err();
        assert!(e.contains("formatVersion 4: la prueba de edad exige una cabeza v5"), "{e}");
    }

    // RFC-0008 E1 (S495). Como en la edad, lo alcanzable sin una cabeza firmada es la FORMA del
    // sobre y la VERSION de la cabeza. El enlace tiene sus testigos con pruebas reales en
    // `stark-experiment`, y el productor que lo usa, en la capa y en el nodo.

    /// Sin `enunciado` no hay afirmacion que juzgar, y se dice con su nombre.
    #[test]
    fn un_sobre_de_cobro_sin_enunciado_se_nombra() {
        let p = json!({ "v": 1, "tipo": "cobro_pendiente" });
        assert_eq!(verificar_cobro_pendiente(&p), Err("falta enunciado".into()));
    }

    /// Sin `prueba` no hay nada que enlazar, y se dice antes de mirar la cabeza.
    #[test]
    fn un_sobre_de_cobro_sin_prueba_se_nombra() {
        let en = json!({ "receptor": DIG, "nacido": "0x1", "inferior": "0x1" });
        let p = json!({ "v": 1, "tipo": "cobro_pendiente", "enunciado": en });
        assert_eq!(verificar_cobro_pendiente(&p), Err("falta prueba o no es cadena 0x".into()));
    }

    /// Una cabeza que no es v5 se rechaza por su VERSION, antes de tocar la firma.
    #[test]
    fn un_cobro_con_cabeza_que_no_es_v5_se_rechaza_antes_de_la_firma() {
        let p = json!({
            "v": 1, "tipo": "cobro_pendiente",
            "enunciado": { "receptor": DIG, "nacido": "0x1", "inferior": "0x1" },
            "prueba": "0x00",
            "cabeza": { "available": true, "formatVersion": "0x4" }
        });
        let e = verificar_cobro_pendiente(&p).unwrap_err();
        assert!(e.contains("formatVersion 4: el cobro pendiente exige una cabeza v5"), "{e}");
    }

    /// El `tipo` se despacha por el mando: `cobro_pendiente` llega a su brazo, y el desconocido
    /// nombra ya los seis.
    #[test]
    fn el_mando_despacha_el_cobro_pendiente_y_lo_nombra() {
        let ruta = std::env::temp_dir().join("zk-ssl-verify-s495-tipo.json");
        std::fs::write(&ruta, json!({ "v": 1, "tipo": "cobro_pendiente" }).to_string()).unwrap();
        let brazo = correr(ruta.to_str().unwrap()).unwrap_err();
        std::fs::write(&ruta, json!({ "v": 1, "tipo": "otra" }).to_string()).unwrap();
        let otro = correr(ruta.to_str().unwrap()).unwrap_err();
        let _ = std::fs::remove_file(&ruta);
        assert_eq!(brazo, "falta enunciado");
        assert!(otro.starts_with("tipo desconocido: otra"), "{otro}");
        assert!(otro.contains("`tipo: \"cobro_pendiente\"`"), "{otro}");
    }

    // RFC-0008 E2 (S506). Como en el cobro: lo alcanzable sin una cabeza firmada es la FORMA del
    // sobre y la VERSION de la cabeza. El enlace tiene sus testigos con pruebas reales en
    // `stark-experiment` y en la capa, y el PASTE-E2f-M midio que estos campos bastan.

    /// Sin `enunciado` no hay afirmacion que juzgar, y se dice con su nombre.
    #[test]
    fn un_sobre_de_pago_sin_enunciado_se_nombra() {
        let p = json!({ "v": 1, "tipo": "pago_en_curso" });
        assert_eq!(verificar_pago_en_curso(&p), Err("falta enunciado".into()));
    }

    /// Sin `prueba` no hay nada que enlazar, y se dice antes de mirar la cabeza. Y el enunciado
    /// del pago lleva `importe` y `t` donde el del cobro lleva `inferior`.
    #[test]
    fn un_sobre_de_pago_sin_prueba_se_nombra() {
        let en = json!({ "receptor": DIG, "nacido": "0x1", "importe": "0x2", "t": "0x3" });
        let p = json!({ "v": 1, "tipo": "pago_en_curso", "enunciado": en });
        assert_eq!(verificar_pago_en_curso(&p), Err("falta prueba o no es cadena 0x".into()));
    }

    /// Una cabeza que no es v5 se rechaza por su VERSION, antes de tocar la firma.
    #[test]
    fn un_pago_con_cabeza_que_no_es_v5_se_rechaza_antes_de_la_firma() {
        let p = json!({
            "v": 1, "tipo": "pago_en_curso",
            "enunciado": { "receptor": DIG, "nacido": "0x1", "importe": "0x2", "t": "0x3" },
            "prueba": "0x00",
            "cabeza": { "available": true, "formatVersion": "0x4" }
        });
        let e = verificar_pago_en_curso(&p).unwrap_err();
        assert!(e.contains("formatVersion 4: el pago en curso exige una cabeza v5"), "{e}");
    }

    /// El `tipo` se despacha por el mando: `pago_en_curso` llega a su brazo, y el desconocido
    /// nombra ya los SIETE -esta es la parte que el S506 ENSANCHA del testigo del S495-.
    #[test]
    fn el_mando_despacha_el_pago_en_curso_y_nombra_los_siete() {
        let ruta = std::env::temp_dir().join("zk-ssl-verify-s506-tipo.json");
        std::fs::write(&ruta, json!({ "v": 1, "tipo": "pago_en_curso" }).to_string()).unwrap();
        let brazo = correr(ruta.to_str().unwrap()).unwrap_err();
        std::fs::write(&ruta, json!({ "v": 1, "tipo": "otra" }).to_string()).unwrap();
        let otro = correr(ruta.to_str().unwrap()).unwrap_err();
        let _ = std::fs::remove_file(&ruta);
        assert_eq!(brazo, "falta enunciado");
        assert!(otro.starts_with("tipo desconocido: otra"), "{otro}");
        for t in ["extension", "consumo", "conflicto", "rechazo", "edad", "cobro_pendiente",
                  "pago_en_curso"] {
            assert!(otro.contains(&format!("`tipo: \"{t}\"`")), "no nombra {t}: {otro}");
        }
    }
    // RFC-0008 E3 (S520). Como en el pago: lo alcanzable sin una cabeza firmada es la FORMA del
    // sobre y la VERSION de la cabeza. El enlace tiene sus testigos con pruebas REALES en
    // `stark-experiment` (S517) y en la capa (S518), que es donde vive el productor.

    /// Sin `enunciado` no hay afirmacion que juzgar, y se dice con su nombre.
    #[test]
    fn un_sobre_de_prenda_sin_enunciado_se_nombra() {
        let p = json!({ "v": 1, "tipo": "prenda" });
        assert_eq!(verificar_prenda(&p), Err("falta enunciado".into()));
    }

    /// Sin `prueba` no hay nada que enlazar, y se dice antes de mirar la cabeza. El enunciado de
    /// la prenda es el mas corto de la familia: `receptor` y `marca`, y nada mas.
    #[test]
    fn un_sobre_de_prenda_sin_prueba_se_nombra() {
        let en = json!({ "receptor": DIG, "marca": DIG });
        let p = json!({ "v": 1, "tipo": "prenda", "enunciado": en });
        assert_eq!(verificar_prenda(&p), Err("falta prueba o no es cadena 0x".into()));
    }

    /// Una cabeza que no es v5 se rechaza por su VERSION, antes de tocar la firma -- y el texto
    /// NO puede decir «la unica que firma pmetaRoot», que es la razon de los hermanos y en la
    /// prenda seria falsa (D-AY, D-BF). El segundo aserto es el que lo gatea.
    #[test]
    fn una_prenda_con_cabeza_que_no_es_v5_se_rechaza_con_su_razon() {
        let p = json!({
            "v": 1, "tipo": "prenda",
            "enunciado": { "receptor": DIG, "marca": DIG },
            "prueba": "0x00",
            "cabeza": { "available": true, "formatVersion": "0x4" }
        });
        let e = verificar_prenda(&p).unwrap_err();
        assert!(e.contains("formatVersion 4: la prenda exige una cabeza v5"), "{e}");
        assert!(!e.contains("pmetaRoot"), "la razon de los hermanos no vale aqui: {e}");
    }

    /// El `tipo` se despacha por el mando: `prenda` llega a su brazo, y el desconocido nombra ya
    /// los OCHO -- esta es la parte que el S520 ENSANCHA del testigo del S506.
    #[test]
    fn el_mando_despacha_la_prenda_y_nombra_los_ocho() {
        let ruta = std::env::temp_dir().join("zk-ssl-verify-s520-tipo.json");
        std::fs::write(&ruta, json!({ "v": 1, "tipo": "prenda" }).to_string()).unwrap();
        let brazo = correr(ruta.to_str().unwrap()).unwrap_err();
        std::fs::write(&ruta, json!({ "v": 1, "tipo": "otra" }).to_string()).unwrap();
        let otro = correr(ruta.to_str().unwrap()).unwrap_err();
        let _ = std::fs::remove_file(&ruta);
        assert_eq!(brazo, "falta enunciado");
        assert!(otro.starts_with("tipo desconocido: otra"), "{otro}");
        for t in ["extension", "consumo", "conflicto", "rechazo", "edad", "cobro_pendiente",
                  "pago_en_curso", "prenda"] {
            assert!(otro.contains(&format!("`tipo: \"{t}\"`")), "no nombra {t}: {otro}");
        }
    }

    // ── RFC-0016 (§640): un valor, una escritura ──────────────────────────

    /// El lector de `u64` del sobre rechaza lo que no escribe un elemento, y
    /// dice el valor tal como llego.
    #[test]
    fn un_u64_que_no_es_canonico_se_rechaza_con_su_valor() {
        let p = zk_ssl_hash::MODULO;
        assert_eq!(u64_de(&json!({ "n": format!("{:#x}", p - 1) }), "n"), Ok(p - 1));
        let e = u64_de(&json!({ "n": format!("{:#x}", 0x5a0 + p) }), "n").unwrap_err();
        assert_eq!(e, "n: 0xffffffff000005a1 no es canonico: no es menor que p = 2^64 - 2^32 + 1");
        let e = u64_de(&json!({ "n": "0xffffffffffffffff" }), "n").unwrap_err();
        assert!(e.contains("no es canonico"), "{e}");
    }

    /// El lector de digests dice la regla, no un `Debug`: el cero escrito como
    /// `p` en el primer elemento.
    #[test]
    fn un_digest_que_no_es_canonico_se_rechaza_con_su_regla() {
        let mut b = [0u8; 32];
        b[..8].copy_from_slice(&zk_ssl_hash::MODULO.to_le_bytes());
        let hex: String = b.iter().map(|x| format!("{x:02x}")).collect();
        let e = digest_de(&json!({ "chainDigest": format!("0x{hex}") }), "chainDigest").unwrap_err();
        assert_eq!(e, "chainDigest: 0xffffffff00000001 no es canonico: no es menor que p = 2^64 - 2^32 + 1");
    }

    /// ⚠️ EL CASO MEDIDO, de punta a punta y sobre un vector REAL: la cabeza
    /// de `posicion-v2.json` con `n + p`. La firma es la misma y el digest
    /// recompone igual, porque `as_digest` reduce; hasta el §640 el mando decia
    /// VERDE. Ahora la lectura lo para ANTES de la firma, con el nombre del campo.
    #[test]
    fn la_misma_firma_ya_no_acredita_dos_n() {
        let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../spec/vectors/paquete/posicion-v2.json");
        let mut p: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&base).expect("el vector existe")).unwrap();
        assert_eq!(verificar_paquete(&p), Ok(()), "el vector honesto verifica");
        let n = u64::from_str_radix(p["cabeza"]["n"].as_str().unwrap().trim_start_matches("0x"), 16).unwrap();
        p["cabeza"]["n"] = json!(format!("{:#x}", n + zk_ssl_hash::MODULO));
        let e = verificar_paquete(&p).unwrap_err();
        assert!(e.starts_with("n: ") && e.contains("no es canonico"), "{e}");
    }
}
