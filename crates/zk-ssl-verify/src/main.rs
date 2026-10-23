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
//! el catalogo del documento cubre CADA llamada de rechazo —censo por
//! llamada, no por linea: `tools/check_catalogo.py`, en el canon desde el
//! §777— y CADA clave que este fichero lee (esa, todavia a mano).
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
//! Salida: VERDE y exit 0; el primer fallo con nombre (`ROJO: …`) y exit 1; uso —ningun argumento,
//! o mas de uno— y exit 2; el cuarto estado del sobre de completitud, sin `ROJO`, y exit 3 (§807:
//! decia hasta el 2; el 3 es del §573). Los textos y los codigos, en `spec/PAQUETE.md`, 5 y 6.
use std::process::ExitCode;

use zk_ssl_verify::actas::{
    acta_de_json, juzgar_continuidad, tramos_de, verificar_cadena, ActaFirmada, Rotacion, Tramo,
};
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

/// §650: lee con `zk_ssl_hash::hex_canonico`, sobre bytes. La copia de aqui
/// troceaba el `&str` y un caracter multibyte en un paquete hacia salir al
/// mando con 101 (panico) en vez de ROJO. Los tres textos de rechazo son los de
/// siempre: los manifiestos de `spec/vectors/paquete/` los fijan.
fn hex_a_bytes(s: &str) -> Result<Vec<u8>, String> {
    use zk_ssl_hash::HexError;
    zk_ssl_hash::hex_canonico(s).map_err(|e| match e {
        HexError::SinPrefijo => err(format!("sin 0x: {s:.18}")),
        HexError::LongitudImpar(n) => err(format!("hex impar ({n} chars)")),
        HexError::NoHex(i) => err(format!("hex: cifra no admitida en la posicion {i}")),
        HexError::NoMinima => err(format!("hex no minimo: {s:.18}")),
    })
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
    if !s.starts_with("0x") {
        return Err(err(format!("{campo} sin 0x")));
    }
    // §662: la escritura minima y ninguna otra (sin `+`, sin mayusculas, sin
    // ceros a la izquierda); despues, la cota `< p` del §640.
    let x = zk_ssl_hash::cantidad_canonica(s).map_err(|e| err(format!("{campo}: {e}")))?;
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
        // RFC-0015 E5c (§686): una cabeza firmada fuera de los tramos de su clave en la cadena.
        Some("solapamiento") => return verificar_solapamiento(&p),
        Some(otro) => {
            return Err(err(format!(
                "tipo desconocido: {otro} - se lee un paquete de posicion (sin `tipo`), \
                 `tipo: \"extension\"`, `tipo: \"consumo\"`, `tipo: \"conflicto\"`, \
                 `tipo: \"rechazo\"`, `tipo: \"edad\"`, `tipo: \"cobro_pendiente\"`, \
                 `tipo: \"pago_en_curso\"`, `tipo: \"prenda\"`, `tipo: \"completitud\"`, \
                 `tipo: \"ancla\"`, `tipo: \"ancla-cofirmada\"` o `tipo: \"solapamiento\"`"
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
    // §663: un testigo, una cofirma. Sin esto, la misma cofirma copiada cinco
    // veces salia como «cofirmas: 5 verifican», y quien decide su politica por
    // ese numero contaba cinco testigos donde habia uno.
    let mut testigos: Vec<Vec<u8>> = Vec::with_capacity(lista.len());
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
        if let Some(m) = testigos.iter().position(|t| *t == kt) {
            return Err(err(format!("cofirma {n}: repite el testigo de la cofirma {}", m + 1)));
        }
        testigos.push(kt.clone());
        let firma = hex_de_cofirma(c, "firma", n)?;
        // ⚠️ §780 · El preambulo de la cofirma lleva la version en UN byte. Hasta el §780 se
        //    truncaba (`as u8`): `0x103` verificaba como `0x3` -un valor, dos escrituras, lo que
        //    el RFC-0016 cierra- y la segunda implementacion se caia en el mismo vector.
        let vf = u64_de(c, "versionFormato")?;
        let version_formato = u8::try_from(vf).map_err(|_| {
            err(format!(
                "cofirma {n}: versionFormato {vf:#x} no cabe en el byte que el preambulo le reserva"
            ))
        })?;
        let cf = CabezaFirmada {
            version_formato,
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

/// RFC-0019 E1 (§681): el indice XMSS que va DENTRO de la firma de una cabeza. El `index`
/// declarado no entra en lo firmado (§399: solo se ata por abajo, `embebido < declarado`), asi que
/// cualquiera puede inflarlo en un sobre; el embebido no se cambia sin romper la firma. La ventana
/// de completitud se mide con este, en los dos extremos.
fn indice_embebido(c: &serde_json::Value, cual: &str) -> Result<u64, String> {
    let firma = c
        .get("signature")
        .and_then(|x| x.as_str())
        .ok_or_else(|| err(format!("{cual}: falta signature")))?;
    indice_de_firma(&hex_a_bytes(firma)?).map_err(|e| err(format!("{cual}: {e}")))
}

/// RFC-0019 E1 (§681): el indice que la firma ACREDITA, `embebido + 1`: el de la cabeza si ningun
/// indice se quemo sin firmar. Es el `S` de la ventana.
fn indice_acreditado(c: &serde_json::Value, cual: &str) -> Result<u64, String> {
    Ok(indice_embebido(c, cual)? + 1)
}

/// El paquete de EXTENSION (§293): dos cabezas v3 firmadas y el camino
/// de consistencia entre sus cimas — el eslabon 2 entero, verificable
/// **sin el nodo**: quien custodia la vieja comprueba que la nueva la
/// EXTIENDE, con el objeto de §291 como juez.
///
/// ⚠️ §712: y despues de la consistencia, el reloj ([`el_reloj_avanza`]). Con el mismo
/// `mmrSize`, el mismo digest; con la misma clave, la hoja de la nueva por encima.
fn verificar_extension(p: &serde_json::Value) -> Result<(), String> {
    let vieja = p.get("vieja").ok_or_else(|| err("falta vieja".into()))?;
    let (cima_v, t_v, clave_v, _) = cabeza_v3_verificada(vieja, "vieja")?;
    let nueva = p.get("nueva").ok_or_else(|| err("falta nueva".into()))?;
    let (cima_n, t_n, clave_n, _) = cabeza_v3_verificada(nueva, "nueva")?;
    // §649: decia «v3» a fuego, y la extension lee v3 a v6; la medida de la segunda implementacion
    // sobre la familia de la rotacion, de cabezas v6, lo destapo. Sin version, como el consumo.
    println!("1/3 las DOS cabezas recomponen su digest y sus firmas verifican");
    match misma_continuidad(p, (vieja, &clave_v), (nueva, &clave_n))? {
        None => println!("2/3 misma publicKey: el mismo firmante en los dos extremos"),
        Some(r) => println!(
            "2/3 claves distintas que las actas unen ({} eslabon(es), la posterior desde la \
             hoja {}): el mismo OPERADOR en los dos extremos",
            r.eslabones, r.desde
        ),
    }
    let camino = camino_mmr(p)?;
    if !zk_ssl_verify::mmr::verificar_consistencia(cima_v, t_v, cima_n, t_n, &camino) {
        return Err(err(format!(
            "la nueva (t={t_n}) NO extiende a la vieja (t={t_v}): historia \
             bifurcada, recortada, o camino que no es el suyo"
        )));
    }
    println!("3/3 la cima nueva EXTIENDE a la vieja: consistencia O(log N), sin el registro");
    // §712: despues de la consistencia, y no antes: sus rojos siguen siendo los suyos.
    println!("{}", el_reloj_avanza(dos_lados((vieja, &clave_v, t_v), (nueva, &clave_n, t_n))?)?);
    println!("VERDE: la extension se sostiene sin el nodo");
    Ok(())
}

/// Un lado de [`el_reloj_avanza`]: quien es, para el texto, y lo que el reloj compara -su
/// `mmrSize`, su `epochDigest`, su clave y su indice EMBEBIDO-.
///
/// ⚠️ §712: los datos sueltos, y no la cabeza entera, porque el ancla del modo 3 (PAQUETE
/// 2.12) no es una cabeza: trae el digest y el indice escritos. Asi el juez es UNO para la
/// extension, el consumo y el ancla.
struct Lado<'a> {
    nombre: &'a str,
    t: u64,
    digest: Digest,
    clave: &'a str,
    embebido: u64,
}

/// Los dos lados de la extension y del consumo: dos cabezas YA verificadas, con su clave y su
/// `mmrSize`. La firma verifico, asi que su indice embebido se lee: aqui no nace un rojo nuevo.
fn dos_lados<'a>(
    (vieja, kv, tv): (&serde_json::Value, &'a str, u64),
    (nueva, kn, tn): (&serde_json::Value, &'a str, u64),
) -> Result<(Lado<'a>, Lado<'a>), String> {
    Ok((
        Lado {
            nombre: "la vieja",
            t: tv,
            digest: digest_de(vieja, "epochDigest")?,
            clave: kv,
            embebido: indice_embebido(vieja, "vieja")?,
        },
        Lado {
            nombre: "la nueva",
            t: tn,
            digest: digest_de(nueva, "epochDigest")?,
            clave: kn,
            embebido: indice_embebido(nueva, "nueva")?,
        },
    ))
}

/// ⚠️ §712 · RFC-0012 D-C · **El reloj es el índice EMBEBIDO.** La consistencia del MMR
/// prueba que la cima nueva contiene a la vieja. No ve dos cabezas en el mismo sitio de la
/// historia, ni una cabeza posterior que firma por debajo de una anterior. Con el mismo `mmrSize`
/// el juez de consistencia es la identidad (`mmr::verificar_consistencia`, el caso identidad, que
/// no se toca): dos cabezas DISTINTAS con la misma cima pasaban, y una bifurcación firmada salía
/// VERDE como extensión. Medido: los dos sentidos de la vista dividida del ancla y una bifurcación
/// con índices embebidos distintos daban VERDE en la extensión y en el consumo.
///
/// Va DESPUÉS de la consistencia: un rojo de consistencia sigue siendo el suyo. Un productor para
/// la extensión, el consumo y el ancla del modo 3 (PAQUETE 2.12); la refutación del conflicto
/// (PAQUETE 2.5) es una extensión.
///
/// - **Mismo `mmrSize`**: el mismo `epochDigest`. La misma cabeza dos veces sigue VERDE.
/// - **`mmrSize` mayor y la misma clave**: el índice embebido de la nueva, estrictamente mayor.
///   Un índice huérfano —reservado y no firmado— solo lo adelanta: no da rojo.
/// - ⚠️ **Con claves distintas el índice no se compara AQUÍ**: la cuenta es una por operador
///   (RFC-0015 D-A, PAQUETE 2.14), y el cruce entre sus claves lo juzgan los tramos de la cadena
///   de actas (`juzgar_continuidad`, reglas 3 y 4), no este reloj.
///
/// ⚠️ El ancla del modo 3 también: con el mismo `mmrSize` su consistencia es la identidad, y el
/// ancla de una rama contra la cabeza de la otra salía VERDE, «la historia anclada es un prefijo»
/// (medido con el kit de antes del §712 y con la segunda).
///
/// Devuelve la línea que lo dice; el rojo nombra su causa, la misma en los tres sobres.
fn el_reloj_avanza((v, n): (Lado, Lado)) -> Result<String, String> {
    let (tv, tn) = (v.t, n.t);
    if tv == tn {
        if v.digest != n.digest {
            return Err(err(format!(
                "extension: mismo mmrSize y digest distinto - {} y {} firman la misma historia \
                 de {tv} cabeza(s) y no son la misma cabeza: una bifurcacion firmada, no una \
                 extension",
                v.nombre, n.nombre
            )));
        }
        return Ok(format!(
            "   {} y {} son la MISMA cabeza: mismo mmrSize y mismo digest",
            v.nombre, n.nombre
        ));
    }
    if v.clave != n.clave {
        return Ok("   claves distintas: el orden de sus hojas lo juzgan los tramos de la cadena, \
                   no el reloj de una clave"
            .into());
    }
    let (ev, en) = (v.embebido, n.embebido);
    if en <= ev {
        return Err(err(format!(
            "extension: el indice embebido no crece - {} (t={tn}) firma en la hoja {en} y {} \
             (t={tv}) en la {ev}: con la misma clave, una cabeza posterior firma por encima \
             (RFC-0012 D-C)",
            n.nombre, v.nombre
        )));
    }
    Ok(format!(
        "   el reloj de la clave avanza con la historia: de la hoja {ev} a la {en} (RFC-0012 D-C)"
    ))
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
    let vieja = p.get("vieja").ok_or_else(|| err("falta vieja".into()))?;
    let (cima_v, t_v, clave_v, cons_v) = cabeza_v3_verificada(vieja, "vieja")?;
    let nueva = p.get("nueva").ok_or_else(|| err("falta nueva".into()))?;
    let (cima_n, t_n, clave_n, cons_n) = cabeza_v3_verificada(nueva, "nueva")?;
    println!("1/5 las DOS cabezas recomponen su digest y sus firmas verifican");
    let rotada = misma_continuidad(p, (vieja, &clave_v), (nueva, &clave_n))?;
    let (raiz_v, raiz_n) = match (cons_v, cons_n) {
        (Some(v), Some(n)) => (v, n),
        _ => return Err(exige_consumos("consumo")),
    };
    match rotada {
        None => println!(
            "2/5 misma publicKey y las dos cabezas llevan consRoot ({}) a los dos lados",
            VersionCabeza::texto_con_consumos()
        ),
        Some(r) => println!(
            "2/5 claves distintas que las actas unen ({} eslabon(es), la posterior desde la \
             hoja {}), y las dos cabezas llevan consRoot ({}) a los dos lados",
            r.eslabones,
            r.desde,
            VersionCabeza::texto_con_consumos()
        ),
    }
    if !zk_ssl_verify::mmr::verificar_consistencia(cima_v, t_v, cima_n, t_n, &camino_mmr(p)?) {
        return Err(err(format!(
            "la nueva (t={t_n}) NO extiende a la vieja (t={t_v}): historia \
             bifurcada, recortada, o camino que no es el suyo"
        )));
    }
    println!("3/5 la cima nueva EXTIENDE a la vieja: el «antes» es de esta historia");
    // §712: el mismo reloj que la extension. Sin el, una vieja de OTRA rama -el mismo mmrSize, sin
    // el consumo- «probaba» que el consumo se publico despues de ella.
    println!("{}", el_reloj_avanza(dos_lados((vieja, &clave_v, t_v), (nueva, &clave_n, t_n))?)?);

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

/// ⚠️ §648 · RFC-0015 E5a · **¿La misma continuidad?** Dos cabezas YA verificadas, cada una con su
/// `publicKey`: con la misma, nada que juzgar (`None`). Con claves distintas y sin `actas` en el
/// sobre, el texto de siempre, byte a byte (`claves_distintas`): ningún vector se mueve. Con
/// `actas` —la cadena que sirve `zkssl_keyActs`, cada una en el JSON de `acta_a_json`—, la
/// continuidad es de UN operador si `juzgar_continuidad` lo dice: la cadena entera vale, lleva de
/// la clave de la cabeza de índice EMBEBIDO menor a la de la otra, la anterior firmó por debajo del
/// `desde` de su sucesora y la posterior en su tramo. El embebido lo lee `indice_embebido` (§681),
/// el mismo que mide la ventana de completitud. Un productor para los ocho sitios que exigen
/// continuidad; el conflicto exige claves distintas y la vista dividida la misma, y no leen
/// `actas`. Con la misma clave, `actas` no se lee: no hay cambio que explicar.
fn misma_continuidad(
    sobre: &serde_json::Value,
    una: (&serde_json::Value, &str),
    otra: (&serde_json::Value, &str),
) -> Result<Option<Rotacion>, String> {
    if una.1 == otra.1 {
        return Ok(None);
    }
    let actas = actas_del_sobre(sobre)?.ok_or_else(claves_distintas)?;
    let lado = |(c, clave): (&serde_json::Value, &str)| -> Result<(Vec<u8>, u64), String> {
        Ok((hex_a_bytes(clave)?, indice_embebido(c, "cabeza")?))
    };
    let ((k1, e1), (k2, e2)) = (lado(una)?, lado(otra)?);
    juzgar_continuidad(&actas, (&k1, e1), (&k2, e2))
        .map(Some)
        .map_err(|e| {
            err(format!(
                "las cabezas llevan claves DISTINTAS y las actas no las unen: {e}"
            ))
        })
}

/// ⚠️ §686 · Las `actas` de un sobre, LEÍDAS —leer no es juzgar—: `None` si no las lleva, y cada
/// una con `acta_de_json` y su posición en el texto. Un productor para los sobres que las leen.
fn actas_del_sobre(sobre: &serde_json::Value) -> Result<Option<Vec<ActaFirmada>>, String> {
    let lista = match sobre.get("actas") {
        None => return Ok(None),
        Some(l) => l
            .as_array()
            .ok_or_else(|| err("actas no es lista (la cadena de zkssl_keyActs)".into()))?,
    };
    lista
        .iter()
        .enumerate()
        .map(|(k, a)| acta_de_json(a).map_err(|e| err(format!("actas[{k}]: {e}"))))
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
}

/// ⚠️ §648 · La cabeza contra la que se comparan las de una resolución —el cierre del sobre de
/// completitud—, con su clave y el sobre, que es quien lleva las `actas` si las lleva.
struct Cierre<'a> {
    sobre: &'a serde_json::Value,
    cabeza: &'a serde_json::Value,
    clave: &'a str,
}

impl Cierre<'_> {
    /// `misma_continuidad` con el cierre como uno de los lados; `que` nombra la otra cabeza en la
    /// línea que dice la rotación.
    fn exige(&self, c: &serde_json::Value, clave: &str, que: &str) -> Result<(), String> {
        if let Some(r) = misma_continuidad(self.sobre, (self.cabeza, self.clave), (c, clave))? {
            println!(
                "   {que}: otra clave del MISMO operador, unida al cierre por las actas ({} \
                 eslabon(es), la posterior desde la hoja {})",
                r.eslabones, r.desde
            );
        }
        Ok(())
    }
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
///
/// ⚠️ §815 (kit 0.4.3, arreglo de seguridad redactado en privado; RFC-0021 R6 f): hasta aqui la lista
/// era de CUATRO y faltaban `ProofFailed` y `VerificationFailed`, que la D-I del RFC-0007 DECLARA sin
/// prueba portable (`spec/rfc/0007-pruebas-sobre-el-estado-comprometido.md`, D-I y su nota del §481)
/// y que la D-G del RFC-0010 manda al cuarto estado. La lista salio de las filas de la tabla D-D
/// marcadas «declarada sin prueba» -`CustodianSetExhausted`, `NotTheIssuer`, `NotTheAccountHolder`- y
/// de la correccion que el §460 escribio tras ella, que dejo `PendingTreeExhausted` declarada sin
/// prueba portable (en la tabla, «E3 la marca; E4 el hueco»); no de la D-I.
/// MEDIDO en privado: un tercero sin cuenta manda una prueba basura con las raices publicas, el nodo
/// la rechaza con `VerificationFailed` y le da su recibo, y el operador honrado no tenia resolucion
/// que el kit aceptara -el sobre de rechazo no la prueba y la declarada salia ROJO-: pasada la
/// ventana, «NO RESUELTA EN LA VENTANA» contra el. Desde §815 se declaran, salida 3, y siguen SIN
/// probarse: que la prueba no verificara es la palabra del nodo hasta que el kit la re-verifique
/// (el tren 0.5, RFC-0019 E3).
const CAUSAS_SIN_PRUEBA_PORTABLE: [&str; 6] = [
    "CustodianSetExhausted",
    "PendingTreeExhausted",
    "NotTheIssuer",
    "NotTheAccountHolder",
    "ProofFailed",
    "VerificationFailed",
];

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
    inicio: u64,
    n: u64,
    cierre: &Cierre,
    que: &str,
) -> Result<u64, String> {
    let c = x.get("cabeza").ok_or_else(|| err(format!("{que}: falta cabeza")))?;
    let a = x.get("acuse").ok_or_else(|| err(format!("{que}: falta acuse")))?;
    let (_, _, clave_r, _) = cabeza_v3_verificada(c, &format!("{que}.cabeza"))?;
    cierre.exige(c, &clave_r, &format!("{que}.cabeza"))?;
    if digest_de(a, "hashPrueba")? != hash {
        return Err(err(format!("{que}: el acuse es de OTRA prueba: no resuelve este recibo")));
    }
    let s = indice_acreditado(c, &format!("{que}.cabeza"))?;
    if !recibos::dentro_de_ventana(inicio, s, n) {
        return Err(err(format!(
            "{que}: llega FUERA de la ventana (indice acreditado {s}, inicio {inicio}, n {n})"
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
    inicio: u64,
    n: u64,
    cierre: &Cierre,
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
    cierre.exige(c, &clave_r, &format!("{que}.cabeza"))?;
    let s = indice_acreditado(c, &format!("{que}.cabeza"))?;
    if !recibos::dentro_de_ventana(inicio, s, n) {
        return Err(err(format!(
            "resolucion: llega FUERA de la ventana (indice acreditado {s}, inicio {inicio}, n {n})"
        )));
    }
    println!("   la resolucion, como sobre de rechazo:");
    verificar_rechazo(sobre)?;
    Ok((s, d))
}

/// RFC-0019 E2 (§682): un `StaleState` no resuelve solo. Su causa se sostiene -las raices que el
/// `recibo` del rechazo declaro no son las de la cabeza-, pero ni ese `recibo` ni el `data` van
/// firmados: que esas raices sean las de la prueba que el operador RECIBIO es su palabra, y el
/// acusado podia responder con un `StaleState` inventado y el sobre decia VERDE. Sale como el cuarto
/// estado. Cada brazo lo pregunta DESPUES de sus propias comprobaciones, para que un sobre mal
/// formado siga siendo ROJO por lo suyo.
fn stale_no_resuelve(d: &serde_json::Value, s: u64, que: &str) -> Result<(), String> {
    if d.get("causa").and_then(|c| c.as_str()) == Some("StaleState") {
        return Err(format!(
            "{DECLARADA_NO_PROBADA}{que}: el operador resuelve por StaleState. La causa se sostiene \
             sobre el estado comprometido (cabeza de indice acreditado {s}), pero su atadura a \
             ESTA operacion -los publicInputs del recibo del rechazo y su data- no va firmada: es \
             la palabra del nodo (RFC-0019 E2)"
        ));
    }
    Ok(())
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
    inicio: u64,
    n: u64,
    cierre: &Cierre,
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
                resolver_por_acuse(
                    par,
                    op.0,
                    inicio,
                    n,
                    cierre,
                    &format!("resolucion.acuses[{i}]"),
                )?;
            }
            println!(
                "3/3 RESUELTA como LOTE aplicado: sus {k} operacion(es), cada una con el acuse de \
                 SU prueba dentro de la ventana (inicio {inicio}, n {n})"
            );
        }
        (false, true, false) => {
            let (s, d) =
                resolver_por_rechazo(&x["sobre"], hash, inicio, n, cierre, "resolucion.sobre")?;
            let j = operacion_nombrada(d, k, "resolucion.sobre.data")?;
            stale_no_resuelve(d, s, "resolucion.sobre")?;
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
    inicio: u64,
    n: u64,
    cierre: &Cierre,
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
            cierre.exige(c, &clave_s, "resolucion.sobre.cabeza")?;
            println!("   la resolucion, como sobre de prenda:");
            verificar_prenda(sobre)?;
            let k = &x["consumo"];
            let cc = k
                .get("cabeza")
                .ok_or_else(|| err("resolucion.consumo: falta cabeza".into()))?;
            let (_, _, clave_c, cons) = cabeza_v3_verificada(cc, "resolucion.consumo.cabeza")?;
            cierre.exige(cc, &clave_c, "resolucion.consumo.cabeza")?;
            let raiz = cons.ok_or_else(|| exige_consumos("prenda"))?;
            let s = indice_acreditado(cc, "resolucion.consumo.cabeza")?;
            if !recibos::dentro_de_ventana(inicio, s, n) {
                return Err(err(format!(
                    "resolucion: llega FUERA de la ventana (indice acreditado {s}, inicio {inicio}, n {n})"
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
                 ventana (inicio {inicio}, n {n})-"
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
            cierre.exige(j, &clave_j, "resolucion.juzgada")?;
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
                resolver_por_rechazo(&x["rechazo"], hash, inicio, n, cierre, "resolucion.rechazo")?;
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
/// un camino que no cruza. La ventana no se declara, se mide (D2), y desde el §681 (RFC-0019 E1) en
/// la unidad de la firma en sus dos extremos: empieza en el indice EMBEBIDO del cierre y `S` es el
/// ACREDITADO (`embebido + 1`) de la cabeza que la mide. El `index` declarado no va firmado y solo
/// se ata por abajo (§399): medida con el, una cabeza honrada con el `index` inflado en el sobre
/// daba un ROJO contra el operador; y la `era` de la hoja va en la unidad del contador del nodo, en
/// la que un indice quemado sin firmar retrasaria la ventana.
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
    let contra = Cierre {
        sobre: p,
        cabeza: cierre,
        clave: &clave,
    };
    // RFC-0019 E1 (§681): la ventana EMPIEZA en el indice embebido del cierre -el acreditado de la
    // ultima cabeza firmada antes de el, la era en la unidad de la firma- y no en la `era` de la
    // hoja, que el nodo cuenta con su contador (`anotar`: el declarado mas uno). En esa unidad un
    // indice quemado sin firmar -gratis para el operador- retrasaria todas las ventanas.
    let inicio = indice_embebido(cierre, "cierre")?;
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
            contra.exige(vig, &clave_v, "vigente")?;
            let s = indice_acreditado(vig, "vigente")?;
            if recibos::dentro_de_ventana(inicio, s, n) {
                return Err(err(format!(
                    "ventana ABIERTA: la cabeza vigente tiene indice acreditado {s} y la ventana \
                     empieza en {inicio}; con n {n} la promesa sigue viva y el sobre es prematuro"
                )));
            }
            return Err(err(format!(
                "NO RESUELTA EN LA VENTANA: el operador recibio la operacion bajo su firma \
                 (rx {rx}, era {era}) y su cabeza vigente de indice acreditado {s} -{} cabezas \
                 firmadas despues del inicio {inicio}, n {n}- llega sin resolucion: se \
                 comprometio a resolver y no lo hizo",
                s - inicio
            )));
        }
        Some(Some("acuse")) => {
            let s = resolver_por_acuse(
                res.expect("hay resolucion"),
                hash,
                inicio,
                n,
                &contra,
                "resolucion",
            )?;
            println!(
                "3/3 RESUELTA como transicion aplicada, dentro de la ventana (indice acreditado \
                 {s}, inicio {inicio}, n {n})"
            );
        }
        Some(Some("rechazo")) => {
            let sobre = res
                .expect("hay resolucion")
                .get("sobre")
                .ok_or_else(|| err("resolucion: falta sobre (el de rechazo)".into()))?;
            let (s, d) = resolver_por_rechazo(sobre, hash, inicio, n, &contra, "resolucion.sobre")?;
            stale_no_resuelve(d, s, "resolucion.sobre")?;
            println!(
                "3/3 RESUELTA como rechazo con prueba, dentro de la ventana (indice {s}); la \
                 atadura al recibo es la palabra del nodo en su data"
            );
        }
        Some(Some("lote")) => {
            resolver_lote(res.expect("hay resolucion"), hash, inicio, n, &contra)?;
        }
        Some(Some("prenda")) => {
            resolver_prenda(res.expect("hay resolucion"), hash, era, inicio, n, &contra)?;
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
/// Cuatro modos; desde el §714, cinco:
///
/// 1. La cabeza SOLA: el mando DERIVA el ancla y su huella y las imprime —
///    el productor de B10.6. Lo impreso es lo que se publica; quien
///    comprueba contra el medio compara la huella byte a byte, porque el
///    mando no tiene red y no puede leer el medio por nadie. VERDE.
/// 2. Con `ancla` y sin `camino`: el ancla ES esta cabeza — los cinco
///    campos iguales, el indice contra el EMBEBIDO en la firma. VERDE.
/// 3. Con `ancla` y `camino`: el ancla es ANTERIOR — misma clave, indice
///    anterior, y la consistencia del MMR (§291, RFC 6962) del lote
///    anclado a la cabeza; despues, el reloj de la extension
///    ([`el_reloj_avanza`]): con el mismo `mmrSize`, el mismo digest (§712). VERDE: la
///    historia anclada es un prefijo.
/// 4. Con `contraria`: la VISTA DIVIDIDA — dos cabezas firmadas de la
///    misma clave con el MISMO indice embebido y contenidos distintos.
///    DETECCION con salida 0, el molde del conflicto (§430): el sobre que
///    la exhibe no falla — delata, y solo el operador pudo producirla.
/// 5. ⚠️ §714 · Con `rama`: la BIFURCACION ([`bifurcacion_del_ancla`]) — dos cabezas
///    firmadas del MISMO operador (la misma clave, o claves que las `actas` unen) con el MISMO
///    `mmrSize` y digests distintos. DETECCION con salida 0, como el modo 4.
///
/// El indice del ancla es el EMBEBIDO (D-C): el declarado solo esta
/// acotado por abajo (§399), y dos cabezas honestas pueden compartirlo.
fn verificar_ancla(p: &serde_json::Value) -> Result<(), String> {
    // Las reglas de FORMA, antes de tocar la criptografia: que combinacion
    // de claves es un sobre y cual no.
    // ⚠️ §714: decide la PRESENCIA de la clave, tambien con `null`: `get` da
    //    `Some(Null)`. Lo fijan los vectores `*-null` de `ancla/`; la segunda lo imita con `in`.
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
    // §714: la rama va sola con la cabeza, como la contraria. Despues de las dos de arriba:
    // sus rojos siguen siendo los suyos.
    if p.get("rama").is_some() && (p.get("ancla").is_some() || p.get("contraria").is_some()) {
        return Err(err(
            "un sobre con rama no lleva ancla ni contraria: la bifurcacion se demuestra con las \
             dos cabezas solas"
                .into(),
        ));
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
    if let Some(rama) = p.get("rama") {
        return bifurcacion_del_ancla(p, (c, &clave_hex, t, cima, epoch_digest, embebido), rama);
    }

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
                    // ⚠️ §712: el reloj de la extension, tambien aqui y despues de la
                    //    consistencia. Con `a_size == t` la consistencia es la identidad y bastaba
                    //    compartir la cima: el ancla de una rama contra la cabeza de la otra -el
                    //    mismo mmrSize, otro digest- salia VERDE. La clave es una por construccion:
                    //    la huella ya se comparo.
                    println!(
                        "{}",
                        el_reloj_avanza((
                            Lado {
                                nombre: "el ancla",
                                t: a_size,
                                digest: a_digest,
                                clave: &clave_hex,
                                embebido: a_indice,
                            },
                            Lado {
                                nombre: "la cabeza",
                                t,
                                digest: epoch_digest,
                                clave: &clave_hex,
                                embebido,
                            },
                        ))?
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

/// ⚠️ §714 · **El modo 5 del ancla: la BIFURCACION.** Dos cabezas firmadas del MISMO
/// operador con el MISMO `mmrSize` y digests distintos: dos cabezas en el mismo sitio de la
/// historia. Un nodo honesto con su diario no repite un `mmrSize`: la hoja del MMR entra en el mismo
/// sitio que anota el diario, y al arrancar el diario siembra el MMR (`zk-ssl-node`, `latido.rs`,
/// `conservar`, y la siembra en `main.rs`). Las dos solo salen de una bifurcacion o de un reinicio
/// sin el diario -o con uno que perdio lineas-. DETECTAR NO ES DISTINGUIR: este sobre no dice cual
/// de las dos, y no hace falta. Las dos ramas existen firmadas, y quien guardaba una tiene una
/// historia que el operador ya no sirve.
///
/// Hasta aqui no habia sobre que lo DELATARA. El de extension la RECHAZA con su causa (§712: «mismo
/// mmrSize y digest distinto»), y un rechazo no es una delacion. El modo 4 solo la ve si ademas se
/// repite la hoja. DETECCION con salida 0, el molde del modo 4 y del conflicto.
///
/// - La `rama` se verifica ENTERA, como la cabeza: dos firmas de verdad o no hay delacion.
/// - **El mismo operador**: la misma `publicKey`, o claves distintas que las `actas` unen, con el
///   juez de los ocho sitios que exigen continuidad ([`misma_continuidad`]). Dos operadores en el
///   mismo `mmrSize` no bifurcan nada: cada uno lleva su historia.
/// - **El mismo `mmrSize`**, con cualquier cima: con la misma, las dos ramas divergen en esta
///   cabeza; con otra, antes. El genesis (`mmrSize` 0) cuenta: dos cabezas que dicen «ninguna
///   cabeza antes» con contenidos distintos son dos historias desde el principio.
/// - **Digests distintos**: el mismo digest es la misma cabeza, aunque la firmen dos hojas.
fn bifurcacion_del_ancla(
    p: &serde_json::Value,
    (c, clave_hex, t, cima, epoch_digest, embebido): (&serde_json::Value, &str, u64, Digest, Digest, u64),
    rama: &serde_json::Value,
) -> Result<(), String> {
    // ⚠️ §714: la rama es el SUJETO de sus rojos. Un `"rama": null`, una cadena o
    //    una rama sin `formatVersion` decian «falta formatVersion o no es cadena 0x», sin nombrarla, y
    //    la segunda decia otra cosa, o revento con la cadena (medido). El orden es el de siempre: que
    //    sea un objeto, su version, y despues la cabeza entera (available, recomposicion, firma).
    if !rama.is_object() {
        return Err(err("rama: no es una cabeza (objeto)".into()));
    }
    let version_r = u64_de(rama, "formatVersion").map_err(|e| format!("rama: {e}"))?;
    match VersionCabeza::try_from(version_r) {
        Ok(v) if v.lleva_mmr() => {}
        _ => {
            return Err(err(format!(
                "rama: formatVersion {version_r}: el ancla lee cabezas {}: la pareja del MMR \
                 viaja firmada desde ellas",
                VersionCabeza::texto_con_mmr()
            )))
        }
    }
    let (cima_r, t_r, clave_r, _) = cabeza_v3_verificada(rama, "rama")?;
    let embebido_r = indice_embebido(rama, "rama")?;
    let operador = match misma_continuidad(p, (c, clave_hex), (rama, &clave_r))? {
        None => "misma publicKey: el mismo firmante".to_string(),
        Some(r) => format!(
            "claves distintas que las actas unen ({} eslabon(es), la posterior desde la hoja {}): \
             el mismo OPERADOR",
            r.eslabones, r.desde
        ),
    };
    println!(
        "2/3 la rama recompone su digest y su firma verifica (indice embebido {embebido_r}); \
         {operador}"
    );
    if t_r != t {
        return Err(err(format!(
            "bifurcacion: mmrSize distinto ({t}, {t_r}) - dos cabezas en sitios distintos de la \
             historia no la bifurcan: el orden entre ellas lo juzga la extension"
        )));
    }
    let digest_r = digest_de(rama, "epochDigest")?;
    if digest_r == epoch_digest {
        return Err(err(format!(
            "bifurcacion: las dos cabezas son LA MISMA (mmrSize {t} y el mismo digest): no hay \
             bifurcacion"
        )));
    }
    let donde = if cima_r == cima {
        "la MISMA cima: las dos ramas divergen en esta cabeza"
    } else {
        "OTRA cima: las dos ramas divergen antes de esta cabeza"
    };
    let tambien = if embebido_r == embebido {
        format!(" (y la misma hoja {embebido}: tambien es la vista dividida del modo 4)")
    } else {
        String::new()
    };
    println!("3/3 el mismo mmrSize {t} y digests DISTINTOS, con {donde}{tambien}");
    println!(
        "VERDE: BIFURCACION - el operador firmo DOS cabezas distintas con el mmrSize {t}: \
         bifurcacion o reinicio sin diario. Es DETECCION del operador: un nodo con su diario no \
         repite un mmrSize, y solo quien tiene la clave pudo firmarlas"
    );
    Ok(())
}

/// ⚠️ §686 · **El sobre del SOLAPAMIENTO** (RFC-0015 E5c; D-C, reglas 3 y 4): una cabeza firmada y
/// la cadena de actas de su operador. La cuenta de índices es una por operador (D-A): cada clave
/// firma cabezas solo en sus tramos, y una hoja fuera de TODOS los de su clave ya era de otra clave
/// suya o de un acta. DETECCIÓN con salida 0, el molde de la vista dividida (§586): el sobre que la
/// exhibe no falla —delata—, y solo quien tiene la clave pudo producirla.
///
/// ⚠️ La cadena la firman las claves del operador: quien delata no puede añadir un acta, y un
/// prefijo de la cadena solo quita tramos superiores —delata menos, nunca de más—. Lo que la
/// génesis firmó antes de su acta no se juzga: es el residuo de la D-I.
fn verificar_solapamiento(p: &serde_json::Value) -> Result<(), String> {
    let c = p
        .get("cabeza")
        .ok_or_else(|| err("falta cabeza (la firmada que se juzga contra su tramo)".into()))?;
    let actas = actas_del_sobre(p)?.ok_or_else(|| {
        err("falta actas (la cadena de zkssl_keyActs: sin ella no hay tramo que cruzar)".into())
    })?;
    let version = u64_de(c, "formatVersion")?;
    match VersionCabeza::try_from(version) {
        Ok(v) if v.lleva_mmr() => {}
        _ => {
            return Err(err(format!(
                "cabeza: formatVersion {version}: el sobre del solapamiento lee cabezas {}: las \
                 que firma un nodo con actas",
                VersionCabeza::texto_con_mmr()
            )))
        }
    }
    let (_, _, clave_hex, _) = cabeza_v3_verificada(c, "cabeza")?;
    let e = indice_embebido(c, "cabeza")?;
    println!(
        "1/3 la cabeza (v{version}) recompone su digest y su firma verifica (indice embebido {e})"
    );
    verificar_cadena(&actas)
        .map_err(|(i, x)| err(format!("el acta {i} de la cadena no vale: {x}")))?;
    let tramos = tramos_de(&actas, &hex_a_bytes(&clave_hex)?);
    if tramos.is_empty() {
        return Err(err(
            "la clave de la cabeza no esta en la cadena: nadie la comprometio".into(),
        ));
    }
    println!(
        "2/3 la cadena vale entera, {} acta(s), y la clave de la cabeza entra en {} de ellas",
        actas.len(),
        tramos.len()
    );
    if let Some(t) = tramos.iter().find(|t| t.contiene(e)) {
        return Err(err(format!(
            "la cabeza firma en la hoja {e}, dentro de su tramo {}: no hay solapamiento",
            texto_de_tramo(t)
        )));
    }
    let lista: Vec<String> = tramos.iter().map(texto_de_tramo).collect();
    println!(
        "3/3 la hoja {e} cae fuera de los tramos de su clave: {}",
        lista.join(", ")
    );
    println!(
        "VERDE: SOLAPAMIENTO - la clave firmo una cabeza en la hoja {e}, que la cuenta del \
         operador (RFC-0015 D-A) daba a otra clave suya o a un acta. Es DETECCION del operador: \
         solo quien tiene la clave pudo firmarla"
    );
    Ok(())
}

/// Un tramo dicho para leerlo: el acta de la que sale y sus bordes.
fn texto_de_tramo(t: &Tramo) -> String {
    let abajo = if t.eslabon == 0 {
        format!(
            "acta 0, la genesis: cualquier hoja salvo la {} de su acta",
            t.desde
        )
    } else {
        format!("acta {}: por encima de la hoja {}", t.eslabon, t.desde)
    };
    match t.hasta {
        Some(h) => format!("({abajo}, por debajo de la {h})"),
        None => format!("({abajo})"),
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

/// **§711 · La frontera del mando.** Un panico al verificar no sale con 101: es ROJO, con el texto
/// de `zk_ssl_air::PANICO_RECOGIDO` y el mensaje del panico, y salida 1. Es la defensa de
/// ESTRUCTURA: el fork rechaza sin panico lo que se midio, y esto cubre lo que no se midio. Los
/// tests llaman a `correr` sin la red, para que un panico se siga viendo como panico.
fn correr_sin_panico(correr: impl FnOnce() -> Result<(), String>) -> Result<(), String> {
    zk_ssl_air::sin_panico(correr, |m| err(format!("el verificador entro en panico: {m}")))
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
    let r = correr_sin_panico(|| correr(&ruta));
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

    /// §662: cada `u64` del sobre se lee en su escritura minima. Falsador: con
    /// `u64::from_str_radix`, `0x+2` y `0x02` leian 2 y la sonda del enjambre
    /// con `"0x+2"` en un `u64` salia VERDE.
    #[test]
    fn un_u64_del_sobre_es_la_escritura_minima() {
        assert_eq!(u64_de(&json!({"n": "0x2"}), "n"), Ok(2));
        for mal in ["0x+2", "0x02", "0xA", "0x", "2"] {
            assert!(u64_de(&json!({"n": mal}), "n").is_err(), "{mal} no es un u64 canonico");
        }
    }
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

    /// §686 · el desconocido enumera el brazo nuevo, el último: `tipo: "solapamiento"`.
    #[test]
    fn el_tipo_desconocido_enumera_el_solapamiento() {
        let e = verificar_paquete(&json!({ "v": 1, "tipo": "otra" })).unwrap_err();
        assert!(e.ends_with("o `tipo: \"solapamiento\"`"), "{e}");
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
    ///
    /// ⚠️ §815: son SEIS -las cuatro y `ProofFailed` y `VerificationFailed`, que la D-I del RFC-0007
    /// declara-; hasta aqui el test se llamaba `..._son_las_cuatro_del_rfc_0007` y fijaba la lista
    /// de cuatro, la que dejaba sin salida al operador honrado (ver la constante).
    #[test]
    fn las_causas_sin_prueba_portable_son_las_seis_del_rfc_0007() {
        assert_eq!(
            CAUSAS_SIN_PRUEBA_PORTABLE,
            [
                "CustodianSetExhausted",
                "PendingTreeExhausted",
                "NotTheIssuer",
                "NotTheAccountHolder",
                "ProofFailed",
                "VerificationFailed",
            ]
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
        let (nada, sin) = (json!({}), json!(null));
        let cierre = Cierre {
            sobre: &nada,
            cabeza: &sin,
            clave: "0xaa",
        };
        let r = |x: serde_json::Value| resolver_prenda(&x, h, 5, 4, 1440, &cierre).unwrap_err();
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
        let e =
            resolver_prenda(&json!({ "sobre": sobre }), dg(9), 5, 4, 1440, &cierre).unwrap_err();
        assert!(e.contains("la prenda es de OTRA prueba"), "{e}");
    }

    /// ⚠️ §648 · RFC-0015 E5a · **con claves distintas, `actas` se lee y se juzga; sin ella, el
    /// texto de siempre.** Sobre los vectores de claves DISTINTAS del catálogo —dos nodos de
    /// verdad, sin acta que los una—, en el consumo y en los tres sitios del cierre: sin `actas`,
    /// el rojo de siempre, byte a byte; con una que no es lista, con un acta que no se lee y con
    /// una cadena vacía, cada uno el suyo. La vista dividida es de UNA clave y no lee `actas`.
    #[test]
    fn con_claves_distintas_las_actas_se_leen_y_se_juzgan() {
        let leer = |t: &str| -> serde_json::Value { serde_json::from_str(t).unwrap() };
        for (nombre, sobre) in [
            (
                "consumo",
                leer(include_str!(
                    "../../../spec/vectors/consumo/rechazo-cons-claves-distintas.json"
                )),
            ),
            (
                "vigente",
                leer(include_str!(
                    "../../../spec/vectors/completitud/neg-vigente-de-otra-clave.json"
                )),
            ),
            (
                "acuse",
                leer(include_str!(
                    "../../../spec/vectors/completitud/neg-acuse-de-otra-clave.json"
                )),
            ),
            (
                "rechazo",
                leer(include_str!(
                    "../../../spec/vectors/completitud/neg-rechazo-de-otra-clave.json"
                )),
            ),
        ] {
            assert_eq!(
                verificar_paquete(&sobre),
                Err(claves_distintas()),
                "{nombre}"
            );
            for (actas, texto) in [
                (
                    json!("0x00"),
                    "actas no es lista (la cadena de zkssl_keyActs)",
                ),
                (json!([{}]), "actas[0]: "),
                (
                    json!([]),
                    "las cabezas llevan claves DISTINTAS y las actas no las unen: la clave que \
                     llega no esta en la cadena",
                ),
            ] {
                let mut con = sobre.clone();
                con["actas"] = actas;
                let e = verificar_paquete(&con).unwrap_err();
                assert!(e.starts_with(texto), "{nombre}: {e}");
            }
        }
        let mut vista = leer(include_str!(
            "../../../spec/vectors/ancla/neg-contraria-de-otra-clave.json"
        ));
        vista["actas"] = json!([]);
        assert_eq!(verificar_paquete(&vista), Err(claves_distintas()));

        // ⚠️ §771 · y con claves que SÍ son de un operador: los sobres de un nodo que rota de A a
        // B. Con la cadena, el consumo entre las dos sale VERDE y la completitud con la vigente
        // de B mide su ventana —abierta: el sobre es prematuro—; sin ella, el rojo de siempre.
        let consumo = leer(include_str!(
            "../../../spec/vectors/consumo/consumo-rotado.json"
        ));
        assert!(verificar_paquete(&consumo).is_ok(), "consumo rotado");
        let completitud = leer(include_str!(
            "../../../spec/vectors/completitud/rotada-ventana-abierta.json"
        ));
        let e = verificar_paquete(&completitud).unwrap_err();
        assert!(e.starts_with("ventana ABIERTA"), "{e}");
        for (nombre, mut sobre) in [("consumo", consumo), ("vigente", completitud)] {
            sobre.as_object_mut().unwrap().remove("actas");
            assert_eq!(
                verificar_paquete(&sobre),
                Err(claves_distintas()),
                "{nombre} rotado sin actas"
            );
        }
    }

    /// ⚠️ §686 · RFC-0015 E5c · **el sobre del solapamiento delata, y solo fuera de los tramos.**
    /// Sobre los vectores de `spec/vectors/rotacion/`, de un nodo que rotó: la clave vieja firmando
    /// después del `desde` de su sucesora y la nueva antes del suyo salen VERDE —detección, salida
    /// 0—; la misma clave en su tramo, ROJO; y una cadena sin la clave, o sin génesis, también. El
    /// veredicto lo da la hoja de la cabeza contra los tramos de su clave, y nada más.
    #[test]
    fn el_sobre_del_solapamiento_delata_solo_fuera_de_los_tramos() {
        let leer = |t: &str| -> serde_json::Value { serde_json::from_str(t).unwrap() };
        for sobre in [
            leer(include_str!(
                "../../../spec/vectors/rotacion/solapamiento-la-vieja-firma-despues.json"
            )),
            leer(include_str!(
                "../../../spec/vectors/rotacion/solapamiento-la-nueva-firma-antes.json"
            )),
        ] {
            assert_eq!(verificar_paquete(&sobre), Ok(()));
        }
        for (sobre, texto) in [
            (
                leer(include_str!(
                    "../../../spec/vectors/rotacion/neg-solap-la-vieja-en-su-tramo.json"
                )),
                "dentro de su tramo (acta 0, la genesis",
            ),
            (
                leer(include_str!(
                    "../../../spec/vectors/rotacion/neg-solap-la-ultima-en-su-tramo.json"
                )),
                "dentro de su tramo (acta 2: por encima de la hoja 10): no hay solapamiento",
            ),
            (
                leer(include_str!(
                    "../../../spec/vectors/rotacion/neg-solap-clave-fuera-de-la-cadena.json"
                )),
                "la clave de la cabeza no esta en la cadena",
            ),
            (
                leer(include_str!(
                    "../../../spec/vectors/rotacion/neg-solap-cadena-sin-genesis.json"
                )),
                "el acta 0 de la cadena no vale",
            ),
        ] {
            let e = verificar_paquete(&sobre).unwrap_err();
            assert!(e.contains(texto), "{texto}: {e}");
        }
    }

    /// ⚠️ §712 · RFC-0012 D-C · **el reloj de la extensión y del consumo, sobre los
    /// vectores.** Los siete negativos del §712 -una bifurcación con hojas distintas, los dos
    /// sentidos de la vista dividida del ancla, la vista dividida y la bifurcación en el consumo, y
    /// el reloj que retrocede en los dos sobres- caen con su causa; antes del §712 los siete daban
    /// VERDE (medido con `tools/banco_bifurcacion.sh` y el kit de antes del §712). Los positivos siguen
    /// VERDE -la extensión por encima de índices huérfanos, las tres rotaciones que cruzan claves,
    /// el consumo-, la misma cabeza a los dos lados también, y con los papeles cambiados sigue
    /// mandando la consistencia: el reloj va DESPUÉS.
    #[test]
    fn la_extension_y_el_consumo_exigen_el_reloj() {
        let leer = |t: &str| -> serde_json::Value { serde_json::from_str(t).unwrap() };
        let (gemelas, reloj) = (
            "extension: mismo mmrSize y digest distinto",
            "extension: el indice embebido no crece",
        );
        for (sobre, causa) in [
            (include_str!("../../../spec/vectors/paquete/rechazo-ext-bifurcada-mismo-mmrSize.json"), gemelas),
            (include_str!("../../../spec/vectors/paquete/rechazo-ext-vista-dividida-ab.json"), gemelas),
            (include_str!("../../../spec/vectors/paquete/rechazo-ext-vista-dividida-ba.json"), gemelas),
            (include_str!("../../../spec/vectors/paquete/rechazo-ext-reloj-retrocede.json"), reloj),
            (include_str!("../../../spec/vectors/consumo/rechazo-cons-bifurcada-mismo-mmrSize.json"), gemelas),
            (include_str!("../../../spec/vectors/consumo/rechazo-cons-vista-dividida.json"), gemelas),
            (include_str!("../../../spec/vectors/consumo/rechazo-cons-reloj-retrocede.json"), reloj),
        ] {
            let e = verificar_paquete(&leer(sobre)).unwrap_err();
            assert!(e.starts_with(causa), "se esperaba «{causa}» y salio: {e}");
        }
        for sobre in [
            include_str!("../../../spec/vectors/paquete/extension.json"),
            include_str!("../../../spec/vectors/paquete/extension-sobre-huerfanos.json"),
            include_str!("../../../spec/vectors/rotacion/rotacion-un-eslabon.json"),
            include_str!("../../../spec/vectors/rotacion/rotacion-sin-firma-de-la-vieja.json"),
            include_str!("../../../spec/vectors/rotacion/rotacion-dos-eslabones.json"),
            include_str!("../../../spec/vectors/consumo/consumo.json"),
        ] {
            assert_eq!(verificar_paquete(&leer(sobre)), Ok(()));
        }
        let mut misma = leer(include_str!("../../../spec/vectors/paquete/extension.json"));
        misma["vieja"] = misma["nueva"].clone();
        misma["camino"] = json!([]);
        assert_eq!(verificar_paquete(&misma), Ok(()), "el caso identidad sigue VERDE");
        let e = verificar_paquete(&leer(include_str!(
            "../../../spec/vectors/rotacion/neg-papeles-cambiados.json"
        )))
        .unwrap_err();
        assert!(e.contains("NO extiende a la vieja"), "la consistencia va antes: {e}");
    }

    /// ⚠️ §712 · **el reloj, puro**, sobre cabezas que solo llevan lo que lee: el digest y
    /// la firma, con su hoja en los cinco primeros bytes. Con el mismo tamaño, el mismo digest -y
    /// las claves no lo salvan-; con tamaño mayor y la misma clave, una hoja estrictamente mayor;
    /// con claves distintas, la hoja no se compara.
    #[test]
    fn el_reloj_puro() {
        let d = |n: u8| format!("0x{}{:02x}", "00".repeat(31), n);
        let c = |dg: &str, hoja: u64| json!({"epochDigest": dg, "signature": format!("0x{hoja:010x}00")});
        let (d1, d2) = (d(1), d(2));
        fn r<'a>(
            v: (&serde_json::Value, &'a str, u64),
            n: (&serde_json::Value, &'a str, u64),
        ) -> Result<String, String> {
            el_reloj_avanza(dos_lados(v, n)?)
        }
        assert!(r((&c(&d1, 1), "k", 5), (&c(&d1, 1), "k", 5)).is_ok(), "la misma cabeza");
        let e = r((&c(&d1, 1), "k", 5), (&c(&d2, 2), "k", 5)).unwrap_err();
        assert!(e.starts_with("extension: mismo mmrSize y digest distinto"), "{e}");
        let e = r((&c(&d1, 9), "k1", 4), (&c(&d2, 2), "k2", 4)).unwrap_err();
        assert!(e.starts_with("extension: mismo mmrSize y digest distinto"), "{e}");
        for hoja in [2, 3] {
            let e = r((&c(&d1, 3), "k", 4), (&c(&d2, hoja), "k", 6)).unwrap_err();
            assert!(e.starts_with("extension: el indice embebido no crece"), "{e}");
        }
        assert!(r((&c(&d1, 3), "k", 4), (&c(&d2, 4), "k", 6)).is_ok());
        assert!(r((&c(&d1, 9), "k1", 4), (&c(&d2, 2), "k2", 6)).is_ok(), "claves distintas");
    }

    /// ⚠️ §712 · **el ancla del modo 3 y el mismo reloj.** El ancla que
    /// el mando deriva de la cabeza vieja de la bifurcación (mmrSize 5, hoja 11) contra la cabeza
    /// nueva (mmrSize 5, la misma cima, hoja 17), con el camino vacío: la consistencia es la
    /// identidad y salía VERDE, «la historia anclada es un prefijo» (medido con el kit de antes del §712 y
    /// con la segunda). Ahora cae con la causa de la extensión, DESPUÉS de la consistencia. El
    /// positivo del modo 3 sigue VERDE, y el juez es el mismo: con la misma clave, la hoja avanza.
    #[test]
    fn el_ancla_del_modo_3_exige_el_reloj() {
        let leer = |t: &str| -> serde_json::Value { serde_json::from_str(t).unwrap() };
        let e = verificar_paquete(&leer(include_str!(
            "../../../spec/vectors/ancla/neg-ancla-bifurcada-mismo-mmrSize.json"
        )))
        .unwrap_err();
        assert!(e.starts_with("extension: mismo mmrSize y digest distinto - el ancla y la cabeza"), "{e}");
        assert_eq!(
            verificar_paquete(&leer(include_str!("../../../spec/vectors/ancla/ancla-extendida.json"))),
            Ok(())
        );
        // Con la vieja de otra cima, manda la consistencia: su rojo sigue siendo el suyo.
        let mut otra = leer(include_str!(
            "../../../spec/vectors/ancla/neg-ancla-bifurcada-mismo-mmrSize.json"
        ));
        otra["ancla"]["mmrRoot"] = otra["ancla"]["epochDigest"].clone();
        let e = verificar_paquete(&otra).unwrap_err();
        assert!(e.contains("NO extiende el ancla"), "la consistencia va antes: {e}");
    }

    /// ⚠️ §714 · **el modo 5 del ancla DELATA la bifurcacion.** Los doce sobres nuevos de
    /// `ancla/`, derivados de cabezas ya capturadas: los tres positivos -la misma cima, otra cima y
    /// el genesis- son VERDE, y cada negativo cae con su causa. Y lo que los vectores no dicen: la
    /// delacion no depende de que cabeza vaya en `cabeza` y cual en `rama`, tampoco con las actas
    /// (la continuidad la ordena el indice embebido).
    #[test]
    fn el_modo_5_del_ancla_delata_la_bifurcacion() {
        let leer = |t: &str| -> serde_json::Value { serde_json::from_str(t).unwrap() };
        let cambiada = |t: &str| {
            let mut s = leer(t);
            let (c, r) = (s["cabeza"].clone(), s["rama"].clone());
            s["cabeza"] = r;
            s["rama"] = c;
            s
        };
        for sobre in [
            include_str!("../../../spec/vectors/ancla/bifurcacion-misma-cima.json"),
            include_str!("../../../spec/vectors/ancla/bifurcacion-otra-cima.json"),
            include_str!("../../../spec/vectors/ancla/bifurcacion-genesis.json"),
        ] {
            assert_eq!(verificar_paquete(&leer(sobre)), Ok(()));
            assert_eq!(verificar_paquete(&cambiada(sobre)), Ok(()), "con los papeles cambiados");
        }
        let distinto = "bifurcacion: mmrSize distinto";
        for (sobre, causa) in [
            (
                include_str!("../../../spec/vectors/ancla/neg-rama-con-ancla.json"),
                "un sobre con rama no lleva ancla ni contraria",
            ),
            (
                include_str!("../../../spec/vectors/ancla/neg-rama-con-contraria.json"),
                "un sobre con rama no lleva ancla ni contraria",
            ),
            (
                include_str!("../../../spec/vectors/ancla/neg-rama-v2.json"),
                "rama: formatVersion 2: el ancla lee cabezas",
            ),
            (
                include_str!("../../../spec/vectors/ancla/neg-rama-firma-de-otra-cabeza.json"),
                "rama: cabeza: la firma es VALIDA pero de otro mensaje",
            ),
            (
                include_str!("../../../spec/vectors/ancla/neg-rama-de-otra-clave.json"),
                "las cabezas llevan claves DISTINTAS: la continuidad es de UN firmante",
            ),
            (
                include_str!("../../../spec/vectors/ancla/neg-rama-actas-que-no-unen.json"),
                "las cabezas llevan claves DISTINTAS y las actas no las unen",
            ),
            (include_str!("../../../spec/vectors/ancla/neg-rama-otro-mmrSize.json"), distinto),
            (include_str!("../../../spec/vectors/ancla/neg-rama-otro-mmrSize-con-actas.json"), distinto),
            (
                include_str!("../../../spec/vectors/ancla/neg-rama-la-misma-cabeza.json"),
                "bifurcacion: las dos cabezas son LA MISMA",
            ),
        ] {
            let e = verificar_paquete(&leer(sobre)).unwrap_err();
            assert!(e.starts_with(causa), "se esperaba «{causa}» y salio: {e}");
        }
        // Con las actas y los papeles cambiados, el operador se sigue aceptando y cae el mmrSize.
        let e = verificar_paquete(&cambiada(include_str!(
            "../../../spec/vectors/ancla/neg-rama-otro-mmrSize-con-actas.json"
        )))
        .unwrap_err();
        assert!(e.starts_with("bifurcacion: mmrSize distinto (4, 2)"), "{e}");
    }

    /// ⚠️ §714 · **la forma la decide la PRESENCIA, y la rama
    /// es el sujeto de sus rojos.** Los nueve sobres nuevos de `ancla/`. Cinco con una clave en
    /// `null`: la referencia la cuenta como presente (`get` da `Some(Null)`), y la segunda la tomaba
    /// por ausente y daba VERDE. Cuatro con la rama mal formada: `null`, una cadena, sin
    /// `formatVersion` y una v2 no disponible -la version antes que `available`-; los tres primeros
    /// decian «falta formatVersion o no es cadena 0x», sin nombrar la rama.
    #[test]
    fn la_forma_por_presencia_y_la_rama_como_sujeto() {
        let leer = |t: &str| -> serde_json::Value { serde_json::from_str(t).unwrap() };
        let rama_con = "un sobre con rama no lleva ancla ni contraria";
        let no_objeto = "rama: no es una cabeza (objeto)";
        for (sobre, causa) in [
            (include_str!("../../../spec/vectors/ancla/neg-rama-con-ancla-null.json"), rama_con),
            (include_str!("../../../spec/vectors/ancla/neg-rama-con-contraria-null.json"), rama_con),
            (
                include_str!("../../../spec/vectors/ancla/neg-contraria-con-ancla-null.json"),
                "un sobre con contraria no lleva ancla",
            ),
            (
                include_str!("../../../spec/vectors/ancla/neg-camino-null-sin-ancla.json"),
                "camino sin ancla: no hay nada que extender",
            ),
            (
                include_str!("../../../spec/vectors/ancla/neg-ancla-exacta-con-camino-null.json"),
                "el ancla declara un indice (6) que no es ANTERIOR",
            ),
            (include_str!("../../../spec/vectors/ancla/neg-rama-null.json"), no_objeto),
            (include_str!("../../../spec/vectors/ancla/neg-rama-cadena.json"), no_objeto),
            (
                include_str!("../../../spec/vectors/ancla/neg-rama-sin-formatVersion.json"),
                "rama: falta formatVersion o no es cadena 0x",
            ),
            (
                include_str!("../../../spec/vectors/ancla/neg-rama-v2-no-disponible.json"),
                "rama: formatVersion 2: el ancla lee cabezas",
            ),
        ] {
            let e = verificar_paquete(&leer(sobre)).unwrap_err();
            assert!(e.starts_with(causa), "se esperaba «{causa}» y salio: {e}");
        }
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

/// §650: un paquete con un caracter multibyte en un campo hex es ROJO con su
/// texto, no un panico (salida 101). Falsador: con la copia que troceaba el
/// `&str`, los dos casos entran en panico «not a char boundary».
#[cfg(test)]
mod tests_hex_650 {
    use super::*;

    fn paquete() -> serde_json::Value {
        serde_json::from_str(include_str!("../../../spec/vectors/paquete/posicion-v2.json")).unwrap()
    }

    #[test]
    fn un_multibyte_en_un_digest_del_paquete_es_rojo_y_no_panico() {
        let mut p = paquete();
        assert!(verificar_paquete(&p).is_ok(), "el positivo va primero y solo (§66.2)");
        let raiz = p["cabeza"]["accountsRoot"].as_str().unwrap().to_string();
        // Mismo largo en BYTES: dos cifras sustituidas por «é» (2 bytes).
        p["cabeza"]["accountsRoot"] = serde_json::Value::String(format!("0x\u{e9}{}", &raiz[4..]));
        let e = verificar_paquete(&p).unwrap_err();
        assert!(e.starts_with("hex: "), "{e}");
    }

    #[test]
    fn un_multibyte_en_la_firma_del_paquete_es_rojo_y_no_panico() {
        let mut p = paquete();
        let firma = p["cabeza"]["signature"].as_str().unwrap().to_string();
        p["cabeza"]["signature"] = serde_json::Value::String(format!("0xa\u{e9}{}", &firma[5..]));
        let r = verificar_paquete(&p);
        assert!(r.is_err(), "una firma con un multibyte no puede verificar");
        assert_eq!(codigo_de_salida(&r), 1, "ROJO, no 101: {r:?}");
    }

    #[test]
    fn el_hex_del_kit_es_el_canonico() {
        assert_eq!(hex_a_bytes("0x0aff"), Ok(vec![0x0a, 0xff]));
        assert_eq!(hex_a_bytes("0x0AFF").unwrap_err(), "hex: cifra no admitida en la posicion 1");
        assert_eq!(hex_a_bytes("0x+f").unwrap_err(), "hex: cifra no admitida en la posicion 0");
        assert_eq!(hex_a_bytes("0xabc").unwrap_err(), "hex impar (3 chars)");
        assert!(hex_a_bytes("abcd").unwrap_err().starts_with("sin 0x: "));
    }
}

/// §651 (D3-META): en cada una de las cinco familias con prueba STARK, la prueba de un vector
/// positivo con el meta VACIADO. Antes, la forma oculta pasaba la guarda y `AIR::new` entraba en
/// panico por el ancho: el mando salia con 101. Ahora es ROJO, salida 1. Las cinco guardas son
/// copias independientes (edad, banda, cobro pendiente, pago en curso, prenda): un vector cada una.
#[cfg(test)]
mod tests_marca_651 {
    use super::*;

    fn sin_meta(hex: &str) -> String {
        let b = hex_a_bytes(hex).unwrap();
        let largo = u16::from_le_bytes([b[4], b[5]]) as usize;
        assert_eq!(largo, 19, "la prueba de la casa lleva la marca de 19 bytes");
        let mut c = b[0..4].to_vec();
        c.extend_from_slice(&[0, 0]);
        c.extend_from_slice(&b[6 + largo..]);
        format!("0x{}", c.iter().map(|x| format!("{x:02x}")).collect::<String>())
    }

    fn caso(vector: &str, ruta_prueba: &[&str]) {
        let base = format!("{}/../../spec/vectors/{vector}", env!("CARGO_MANIFEST_DIR"));
        assert_eq!(codigo_de_salida(&correr(&base)), 0, "{vector}: el positivo va primero y solo");
        let mut v: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&base).unwrap()).unwrap();
        let mut p = &mut v;
        for k in &ruta_prueba[..ruta_prueba.len() - 1] {
            p = &mut p[*k];
        }
        let campo = ruta_prueba[ruta_prueba.len() - 1];
        let mala = sin_meta(p[campo].as_str().unwrap());
        p[campo] = serde_json::Value::String(mala);
        let tmp = std::env::temp_dir()
            .join(format!("d3meta-{}-{}", std::process::id(), vector.replace('/', "-")));
        std::fs::write(&tmp, serde_json::to_string(&v).unwrap()).unwrap();
        let r = std::panic::catch_unwind(|| correr(tmp.to_str().unwrap()));
        let _ = std::fs::remove_file(&tmp);
        let r = r.unwrap_or_else(|_| panic!("CRITICO: {vector} sin marca hace entrar en panico al kit"));
        assert_eq!(codigo_de_salida(&r), 1, "{vector}: ROJO, no otra cosa: {r:?}");
    }

    /// §681 (RFC-0019 E1): la ventana de completitud se mide con el indice que la FIRMA acredita,
    /// no con el `index` declarado, que no va firmado. Sobre `completitud/neg-ventana-abierta.json`
    /// -la vigente es el propio cierre-: con el `index` de la vigente inflado a 1444, o el del cierre,
    /// el veredicto sigue siendo «ventana ABIERTA». Falsador, ensayado: con `S` leido del `index`
    /// declarado, como hasta el §681, la vigente inflada da «NO RESUELTA EN LA VENTANA», un ROJO que
    /// cualquiera fabrica contra un operador honrado (H10).
    #[test]
    fn la_ventana_se_mide_con_el_indice_que_la_firma_acredita() {
        let base = format!(
            "{}/../../spec/vectors/completitud/neg-ventana-abierta.json",
            env!("CARGO_MANIFEST_DIR")
        );
        let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&base).unwrap()).unwrap();
        for campo in ["vigente", "cierre"] {
            let mut w = v.clone();
            w[campo]["index"] = serde_json::json!("0x5a4");
            let e = verificar_paquete(&w).expect_err("la ventana sigue abierta: ROJO de prematuro");
            assert!(e.contains("ventana ABIERTA"), "{campo} inflado: {e}");
        }
    }

    /// §682 (RFC-0019 E2): un rechazo `StaleState` no resuelve solo, ni en la via directa ni en el
    /// lote: el cuarto estado, salida 3. Los mismos bytes que, con el kit 0.4, daban VERDE
    /// (`spec/vectors/0.4/completitud/`). Control: un `StaleState` mal atado sigue siendo ROJO por lo
    /// suyo (`neg-rechazo-de-otra-operacion`). Falsador, ensayado: sin la pregunta en los dos brazos,
    /// los dos dan VERDE y cae.
    #[test]
    fn un_stale_state_no_resuelve_solo() {
        let raiz = format!("{}/../../spec/vectors", env!("CARGO_MANIFEST_DIR"));
        for v in ["rechazo-stale-declarada", "lote-rechazado-stale-declarada"] {
            let r = correr(&format!("{raiz}/completitud/{v}.json"));
            assert_eq!(codigo_de_salida(&r), 3, "{v}: {r:?}");
            assert!(r.unwrap_err().contains("resuelve por StaleState"), "{v}");
        }
        let r = correr(&format!("{raiz}/completitud/neg-rechazo-de-otra-operacion.json"));
        assert_eq!(codigo_de_salida(&r), 1, "mal atado, ROJO: {r:?}");
    }

    /// §815 (kit 0.4.3; RFC-0007 D-I, RFC-0010 D-G): un recibo que el nodo rechazo por
    /// `VerificationFailed` o `ProofFailed` se resuelve DECLARADO, el cuarto estado, salida 3, en la
    /// via directa y en el lote, sobre los tres vectores nuevos de `completitud/` y, en el lote, con
    /// `ProofFailed` mutada aqui sobre el del lote, que no tiene vector propio. Y sigue SIN
    /// probarse: el mismo sobre de rechazo, con esas causas, es ROJO por la causa, como el de las
    /// otras cuatro. Controles: la declarada con `StaleState` sigue ROJO (`neg-declarada-con-prueba`),
    /// y la de otra operacion tambien. Falsador, ensayado: sin `VerificationFailed` en
    /// `CAUSAS_SIN_PRUEBA_PORTABLE`, la declarada sale 1, «no es de las que el RFC-0007 dejo sin
    /// prueba portable», y este test cae; sin `ProofFailed`, igual, y sin ella solo en el brazo del
    /// lote, tambien (el K1 de la revision rev1 del §815, que antes sobrevivia).
    #[test]
    fn verification_failed_y_proof_failed_se_declaran_y_no_se_prueban() {
        let raiz = format!("{}/../../spec/vectors/completitud", env!("CARGO_MANIFEST_DIR"));
        for (v, quiere) in [
            ("declarada-verification-failed", "el operador declara VerificationFailed, una causa"),
            ("declarada-proof-failed", "el operador declara ProofFailed, una causa"),
            (
                "lote-declarada-verification-failed",
                "el operador declara VerificationFailed en la operacion 0 del lote",
            ),
        ] {
            let r = correr(&format!("{raiz}/{v}.json"));
            assert_eq!(codigo_de_salida(&r), 3, "{v}: {r:?}");
            let e = r.unwrap_err();
            assert!(e.starts_with(DECLARADA_NO_PROBADA) && e.contains(quiere), "{v}: {e}");
        }
        // el lote con `ProofFailed`, que no tiene vector propio: el mismo sobre con la causa mutada
        // aqui (la revision del §815 lo midio: sin esto, el brazo del lote sin `ProofFailed` no caia)
        let mut lote: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(format!("{raiz}/lote-declarada-verification-failed.json")).unwrap(),
        )
        .unwrap();
        lote["resolucion"]["data"]["causa"] = serde_json::json!("ProofFailed");
        let r = verificar_paquete(&lote);
        assert_eq!(codigo_de_salida(&r), 3, "lote con ProofFailed: {r:?}");
        let e = r.unwrap_err();
        assert!(
            e.starts_with(DECLARADA_NO_PROBADA)
                && e.contains("el operador declara ProofFailed en la operacion 0 del lote"),
            "lote con ProofFailed: {e}"
        );
        // sin probarse: el sobre de rechazo con esas causas no lo prueba este mando
        let base: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(format!("{raiz}/rechazo-stale-declarada.json")).unwrap(),
        )
        .unwrap();
        for causa in ["VerificationFailed", "ProofFailed"] {
            let mut w = base.clone();
            w["resolucion"]["sobre"]["data"]["causa"] = serde_json::json!(causa);
            let r = verificar_paquete(&w);
            assert_eq!(codigo_de_salida(&r), 1, "{causa}: {r:?}");
            let e = r.unwrap_err();
            assert!(
                e.contains(&format!("la causa {causa} no la prueba este mando")),
                "{causa}: {e}"
            );
        }
        for v in ["neg-declarada-con-prueba", "neg-declarada-de-otra-operacion"] {
            let r = correr(&format!("{raiz}/{v}.json"));
            assert_eq!(codigo_de_salida(&r), 1, "{v}: {r:?}");
        }
    }

    #[test]
    fn la_prueba_de_edad_sin_marca_es_rojo() {
        caso("edad/edad-todos.json", &["prueba"]);
    }
    #[test]
    fn la_prueba_de_banda_sin_marca_es_rojo() {
        caso("rechazo/saldo-insuficiente.json", &["banda", "prueba"]);
    }
    #[test]
    fn la_prueba_del_cobro_pendiente_sin_marca_es_rojo() {
        caso("pendiente/cobro-inferior-0.json", &["prueba"]);
    }
    #[test]
    fn la_prueba_del_pago_en_curso_sin_marca_es_rojo() {
        caso("pago/pago-t-seq.json", &["prueba"]);
    }
    #[test]
    fn la_prueba_de_la_prenda_sin_marca_es_rojo() {
        caso("prenda/prenda.json", &["prueba"]);
    }
}

/// §653 (SEG-04): una prueba con un byte de cola. Antes se leia igual y verificaba, con otra huella;
/// ahora el lector del fork la rechaza (`UnconsumedBytes`) y el mando dice ROJO.
#[cfg(test)]
mod tests_cola_653 {
    use super::*;

    #[test]
    fn una_prueba_con_bytes_de_cola_es_rojo() {
        let base = format!("{}/../../spec/vectors/prenda/prenda.json", env!("CARGO_MANIFEST_DIR"));
        assert_eq!(codigo_de_salida(&correr(&base)), 0, "el positivo va primero y solo");
        let mut v: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&base).unwrap()).unwrap();
        let p = format!("{}00", v["prueba"].as_str().unwrap());
        v["prueba"] = serde_json::Value::String(p);
        let tmp = std::env::temp_dir().join(format!("cola-653-{}", std::process::id()));
        std::fs::write(&tmp, serde_json::to_string(&v).unwrap()).unwrap();
        let r = correr(tmp.to_str().unwrap());
        let _ = std::fs::remove_file(&tmp);
        assert_eq!(codigo_de_salida(&r), 1, "ROJO: {r:?}");
    }
}

/// Lo que comparten las puertas de la prueba STARK del mando: las cuatro pruebas de `spec/vectors/`
/// que barren -edad, pago, prenda y cobro pendiente: el mismo fork y cuatro jueces- y el juez que
/// el mando llama con cada una. La firma de la cabeza no toca la prueba: la comprueban los
/// manifiestos, con el vector entero.
#[cfg(test)]
mod tests_puerta {
    use super::*;
    use zk_ssl_air::pago_en_curso::verificar_contra_cabeza as juez_pago;

    pub(super) const VECTORES: [&str; 4] = [
        "edad/edad-todos.json",
        "pago/pago-t-frontera.json",
        "prenda/prenda.json",
        "pendiente/cobro-inferior-importe.json",
    ];

    pub(super) fn vector(nombre: &str) -> serde_json::Value {
        let ruta = format!("{}/../../spec/vectors/{nombre}", env!("CARGO_MANIFEST_DIR"));
        serde_json::from_str(&std::fs::read_to_string(ruta).unwrap()).unwrap()
    }

    /// El juez de un sobre con prueba, armado con los mismos lectores que el mando.
    pub(super) enum Juez {
        Edad(Afirmacion, CabezaEdad),
        Pago(AfirmacionPago, CabezaPago),
        Prenda(AfirmacionPrenda, CabezaPrenda),
        Cobro(AfirmacionCobro, CabezaCobro),
    }

    pub(super) fn juez_de(p: &serde_json::Value) -> Juez {
        let e = &p["enunciado"];
        let c = &p["cabeza"];
        let seq = u64_de(c, "seq").unwrap();
        let pending_root = digest_de(c, "pendingRoot").unwrap();
        let pmeta_root = digest_de(c, "pmetaRoot").unwrap();
        match p["tipo"].as_str() {
            Some("edad") => Juez::Edad(
                Afirmacion {
                    t: u64_de(e, "t").unwrap(),
                    k: u64_de(e, "k").unwrap(),
                    emisor: e.get("emisor").map(|_| u64_de(e, "emisor").unwrap()),
                    subraiz_pend: digest_de(&p["subraices"], "pendientes").unwrap(),
                    subraiz_meta: digest_de(&p["subraices"], "meta").unwrap(),
                },
                CabezaEdad {
                    seq,
                    pending_root,
                    pmeta_root,
                    next_pending: u64_de(c, "nextPending").unwrap(),
                },
            ),
            Some("pago_en_curso") => Juez::Pago(
                AfirmacionPago {
                    receptor: digest_de(e, "receptor").unwrap(),
                    importe: u64_de(e, "importe").unwrap(),
                    t: u64_de(e, "t").unwrap(),
                    nacido: u64_de(e, "nacido").unwrap(),
                },
                CabezaPago { seq, pending_root, pmeta_root },
            ),
            Some("prenda") => {
                Juez::Prenda(enunciado_de_prenda(p).unwrap().0, CabezaPrenda { pending_root })
            }
            Some("cobro_pendiente") => Juez::Cobro(
                AfirmacionCobro {
                    receptor: digest_de(e, "receptor").unwrap(),
                    nacido: u64_de(e, "nacido").unwrap(),
                    inferior: u64_de(e, "inferior").unwrap(),
                },
                CabezaCobro { seq, pending_root, pmeta_root },
            ),
            otro => panic!("sin juez para {otro:?}"),
        }
    }

    pub(super) fn juzgar(j: &Juez, prueba: &[u8]) -> Result<(), String> {
        match j {
            Juez::Edad(af, c) => verificar_contra_cabeza(prueba, af, c).map(|_| ()),
            Juez::Pago(af, c) => juez_pago(prueba, af, c).map(|_| ()),
            Juez::Prenda(af, c) => enlazar_prenda(prueba, af, c).map(|_| ()),
            Juez::Cobro(af, c) => enlazar_cobro(prueba, af, c).map(|_| ()),
        }
    }
}

/// **§711 · LA PUERTA DE REGRESION: una prueba ajena se rechaza con `Err`, nunca con un panico.**
///
/// Medido antes del arreglo: el byte 34 de las pruebas de edad y de pago a cero hacia salir al
/// kit con 101 en `winter-air/src/options.rs:143`, y el byte 46 a cero, en `proof/queries.rs:97`,
/// dentro de `verify`. Tres barridos sobre cuatro pruebas de `spec/vectors/` -edad, pago, prenda y
/// cobro pendiente: el mismo fork y cuatro jueces-, contra el juez que el mando llama con cada una
/// -la firma de la cabeza no toca la prueba: la comprueban los manifiestos, con el vector entero-,
/// y cada mutacion tiene que ser `Err`, sin panico y sin la red de `zk_ssl_air::sin_panico` (que no
/// puede esconder lo que tiene que arreglar el verificador):
///
/// 1. la cabecera: los bytes 0 a 49 con cada uno de `00 01 03 40 FF`, el 46 a cero incluido;
/// 2. la forma: cada byte que da forma a la prueba -cuentas, longitudes, profundidades, el marco,
///    las capas FRI y su particion-, leido de sus propios bytes, con los mismos cinco valores;
/// 3. el azar: de uno a tres bytes de CUALQUIER sitio de la prueba, con semilla fija y presupuesto
///    (`ARQUEO_SIN_PANICO_PRESUPUESTO`, 1.000 por prueba si no se dice).
///
/// Una mutacion de esas, de uno a tres bytes, que VERIFICA tambien cae: seria una prueba con otros
/// bytes y la misma validez. ⚠️ Lo que cambia la estructura de la prueba no se barre aqui: la
/// puerta mide el panico.
#[cfg(test)]
mod tests_sin_panico {
    use super::tests_puerta::{juez_de, juzgar, vector, VECTORES};
    use super::*;
    use std::sync::{Mutex, Once};

    const VALORES: [u8; 5] = [0x00, 0x01, 0x03, 0x40, 0xFF];
    const SEMILLA: u64 = 0x701701;
    const PRESUPUESTO: usize = 1_000;

    /// Donde entro en panico cada hilo: lo apunta un gancho que se pone una vez y llama despues al
    /// que habia, para que los panicos de los demas tests se sigan viendo.
    static DONDE: Mutex<Vec<(String, String)>> = Mutex::new(Vec::new());
    static GANCHO: Once = Once::new();

    fn apuntar_panicos() {
        GANCHO.call_once(|| {
            let previo = std::panic::take_hook();
            std::panic::set_hook(Box::new(move |info| {
                let hilo = std::thread::current().name().unwrap_or("").to_string();
                let lugar =
                    info.location().map(|l| format!("{}:{}", l.file(), l.line())).unwrap_or_default();
                if let Ok(mut d) = DONDE.lock() {
                    d.push((hilo, lugar));
                }
                previo(info)
            }));
        });
    }

    fn ultimo_panico() -> String {
        let hilo = std::thread::current().name().unwrap_or("").to_string();
        let d = DONDE.lock().unwrap();
        d.iter().rev().find(|(h, _)| *h == hilo).map(|(_, l)| l.clone()).unwrap_or_default()
    }

    /// Lo que salio de un barrido: cuantas mutaciones, y las que no fueron un `Err` limpio.
    #[derive(Default)]
    struct Cuenta {
        mutaciones: usize,
        panicos: Vec<String>,
        verifican: Vec<String>,
        recogidos: Vec<String>,
    }

    impl Cuenta {
        fn exigir_limpio(&self, barrido: &str) {
            let total = self.panicos.len() + self.verifican.len() + self.recogidos.len();
            assert!(
                total == 0,
                "{barrido}: {} de {} mutaciones no son un Err limpio\n  panicos: {:?}\n  \
                 verifican: {:?}\n  recogidos por la red: {:?}",
                total,
                self.mutaciones,
                self.panicos,
                self.verifican,
                self.recogidos
            );
        }
    }

    /// Aplica cada mutacion (una lista de pares byte, valor) a la prueba y la juzga.
    fn barrer(nombre: &str, mutaciones: &[Vec<(usize, u8)>], cuenta: &mut Cuenta) {
        apuntar_panicos();
        let p = vector(nombre);
        let j = juez_de(&p);
        let buena = hex_a_bytes(p["prueba"].as_str().unwrap()).unwrap();
        assert_eq!(juzgar(&j, &buena), Ok(()), "{nombre}: la prueba sin tocar verifica");
        for m in mutaciones {
            let mut b = buena.clone();
            for &(i, v) in m {
                b[i] = v;
            }
            if b == buena {
                continue; // la mutacion que no cambia nada no es una mutacion
            }
            cuenta.mutaciones += 1;
            let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| juzgar(&j, &b)));
            let que = format!("{nombre} {m:?}");
            match r {
                Err(_) => cuenta.panicos.push(format!("{que} en {}", ultimo_panico())),
                Ok(Ok(())) => cuenta.verifican.push(que),
                Ok(Err(e)) if e.contains(zk_ssl_air::PANICO_RECOGIDO) => {
                    cuenta.recogidos.push(format!("{que}: {e}"))
                }
                Ok(Err(_)) => {}
            }
        }
    }

    /// Los offsets de los bytes que dan FORMA a una prueba de winterfell 0.13.1 con el compromiso
    /// de la casa, en el orden de `Proof::write_into`: los de la cabecera, el primer byte de cada
    /// longitud, y en cada lote su profundidad, su cuenta de vectores, las dos primeras cuentas de
    /// nodos y su cuenta de sales. Recorre la prueba entera y exige acabar en su ultimo byte.
    fn bytes_de_forma(b: &[u8]) -> Vec<usize> {
        recorrer(b).forma
    }

    /// Lo que deja el recorrido de una prueba: los bytes de forma, el offset del byte que cuenta
    /// las capas FRI, y de cada capa su tramo `[desde, hasta)` y la profundidad de su lote; desde
    /// el §713, tambien donde va el resto (lo usa `tests_resto`).
    pub(super) struct Recorrido {
        forma: Vec<usize>,
        capas_en: usize,
        capas: Vec<(usize, usize, u8)>,
        /// §713: donde va la longitud del resto FRI (`u16`); el resto empieza dos bytes despues.
        pub(super) resto_en: usize,
    }

    pub(super) fn recorrer(b: &[u8]) -> Recorrido {
        struct L<'a> {
            b: &'a [u8],
            i: usize,
            forma: Vec<usize>,
        }
        impl L<'_> {
            fn marca(&mut self, n: usize) -> &[u8] {
                self.forma.push(self.i);
                self.i += n;
                &self.b[self.i - n..self.i]
            }
            fn u8(&mut self) -> usize {
                self.marca(1)[0] as usize
            }
            fn u16(&mut self) -> usize {
                let x = self.marca(2);
                u16::from_le_bytes([x[0], x[1]]) as usize
            }
            fn u32(&mut self) -> usize {
                let x = self.marca(4);
                u32::from_le_bytes([x[0], x[1], x[2], x[3]]) as usize
            }
            fn vint(&mut self) -> usize {
                let n = self.b[self.i].trailing_zeros() as usize + 1;
                if n == 9 {
                    let x = self.marca(9);
                    return u64::from_le_bytes(x[1..9].try_into().unwrap()) as usize;
                }
                let mut a = [0u8; 8];
                a[..n].copy_from_slice(self.marca(n));
                (u64::from_le_bytes(a) >> n) as usize
            }
            fn lote(&mut self) {
                self.u8();
                let vectores = self.vint();
                for k in 0..vectores {
                    let nodos = if k < 2 { self.vint() } else { self.saltar_vint() };
                    self.i += 32 * nodos;
                }
                let sales = self.u32();
                self.i += 32 * sales;
            }
            fn saltar_vint(&mut self) -> usize {
                let antes = self.forma.len();
                let v = self.vint();
                self.forma.truncate(antes);
                v
            }
        }
        let mut l = L { b, i: 0, forma: Vec::new() };
        l.u8();
        let aux = l.u8();
        l.u8();
        l.u8();
        let meta = l.u16();
        l.i += meta;
        let modulo = l.u8();
        l.i += modulo;
        for _ in 0..10 {
            l.u8(); // las opciones, byte a byte
        }
        l.vint(); // las restricciones
        l.u8(); // las consultas unicas
        let compromisos = l.u16();
        l.i += compromisos;
        for _ in 0..(if aux > 0 { 3 } else { 2 }) {
            let valores = l.vint();
            l.i += valores;
            let apertura = l.vint();
            let fin = l.i + apertura;
            l.lote();
            assert_eq!(l.i, fin, "la apertura de unas consultas acaba donde dice");
        }
        for _ in 0..2 {
            let n = l.u16();
            l.u8(); // las filas del marco
            l.i += n - 1;
        }
        let capas_en = l.i;
        let mut tramos = Vec::new();
        for _ in 0..l.u8() {
            let desde = l.i;
            let valores = l.u32();
            l.i += valores;
            let caminos = l.u32();
            let fin = l.i + caminos;
            let profundidad = b[l.i];
            l.lote();
            assert_eq!(l.i, fin, "el lote de una capa FRI acaba donde dice");
            tramos.push((desde, l.i, profundidad));
        }
        let resto_en = l.i;
        let resto = l.u16();
        l.i += resto;
        l.u8(); // las particiones FRI
        l.i += 8; // el nonce
        assert_eq!(l.i, b.len(), "el recorrido acaba en el ultimo byte de la prueba");
        Recorrido { forma: l.forma, capas_en, capas: tramos, resto_en }
    }

    /// SplitMix64: el azar del barrido, con semilla fija para que el mismo barrido se repita.
    struct Azar(u64);
    impl Azar {
        fn siguiente(&mut self) -> u64 {
            self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
            let mut z = self.0;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
            z ^ (z >> 31)
        }
    }

    /// La frontera del mando: un panico es ROJO, salida 1, con la marca de la red; lo demas pasa
    /// tal cual. Falsador: sin `correr_sin_panico` en `main`, el mando salia con 101.
    #[test]
    fn un_panico_en_la_frontera_del_mando_es_rojo_con_salida_1() {
        let r = correr_sin_panico(|| panic!("panico inyectado"));
        assert_eq!(codigo_de_salida(&r), 1, "{r:?}");
        let e = r.unwrap_err();
        assert!(e.starts_with("el verificador entro en panico: "), "{e}");
        assert!(e.contains(zk_ssl_air::PANICO_RECOGIDO) && e.contains("panico inyectado"), "{e}");
        let bueno = format!("{}/../../spec/vectors/{}", env!("CARGO_MANIFEST_DIR"), VECTORES[0]);
        assert_eq!(correr_sin_panico(|| correr(&bueno)), Ok(()));
    }

    #[test]
    fn la_cabecera_byte_a_byte_es_err_y_nunca_panico() {
        let mut c = Cuenta::default();
        for nombre in VECTORES {
            let mut m: Vec<Vec<(usize, u8)>> =
                (0..50).flat_map(|i| VALORES.iter().map(move |&v| vec![(i, v)])).collect();
            m.push(vec![(46, 0)]);
            barrer(nombre, &m, &mut c);
        }
        eprintln!("sin panico cabecera: {} mutaciones", c.mutaciones);
        c.exigir_limpio("la cabecera");
    }

    #[test]
    fn cada_byte_de_forma_es_err_y_nunca_panico() {
        let mut c = Cuenta::default();
        for nombre in VECTORES {
            let p = vector(nombre);
            let b = hex_a_bytes(p["prueba"].as_str().unwrap()).unwrap();
            let forma = bytes_de_forma(&b);
            let m: Vec<Vec<(usize, u8)>> =
                forma.iter().flat_map(|&i| VALORES.iter().map(move |&v| vec![(i, v)])).collect();
            barrer(nombre, &m, &mut c);
        }
        eprintln!("sin panico forma: {} mutaciones", c.mutaciones);
        c.exigir_limpio("la forma");
    }

    /// Las capas FRI son las que el dominio exige, ni una menos ni una mas, y lo dice la causa
    /// del fork. Medido antes del arreglo: con la ultima capa quitada -bytes bien formados-, el
    /// verificador hacia `remove(0)` sobre un vector vacio dentro de `verify`, un panico; y con una
    /// capa de mas, bien formada y nunca leida, la prueba VERIFICABA con otros bytes y otra huella.
    #[test]
    fn una_capa_fri_de_menos_o_de_mas_es_err_con_causa_y_no_panico() {
        for nombre in VECTORES {
            let p = vector(nombre);
            let j = juez_de(&p);
            let b = hex_a_bytes(p["prueba"].as_str().unwrap()).unwrap();
            let r = recorrer(&b);
            let (desde, hasta, profundidad) = *r.capas.last().expect("hay capas FRI");
            // una de menos: la ultima fuera, y la cuenta un byte por debajo
            let mut menos = b[..desde].to_vec();
            menos.extend_from_slice(&b[hasta..]);
            menos[r.capas_en] -= 1;
            // una de mas: tras la ultima, una capa de un solo grupo de 8 valores de 16 bytes, con
            // un lote vacio de la profundidad que toca (el plegado de la casa es 8: tres niveles
            // menos) y ninguna sal
            let mut capa = Vec::new();
            capa.extend_from_slice(&128u32.to_le_bytes());
            capa.extend_from_slice(&[0u8; 128]);
            let lote = [profundidad - 3, 0x01, 0, 0, 0, 0];
            capa.extend_from_slice(&(lote.len() as u32).to_le_bytes());
            capa.extend_from_slice(&lote);
            let mut mas = b[..hasta].to_vec();
            mas.extend_from_slice(&capa);
            mas.extend_from_slice(&b[hasta..]);
            mas[r.capas_en] += 1;
            for (que, prueba) in [("de menos", menos), ("de mas", mas)] {
                let v = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| juzgar(&j, &prueba)))
                    .unwrap_or_else(|_| panic!("CRITICO: {nombre}, una capa FRI {que}: panico"));
                let e = v.expect_err(&format!("{nombre}: una capa FRI {que} no puede verificar"));
                assert!(e.contains(zk_ssl_air::causa::FRI), "{nombre}, una capa {que}: {e}");
            }
        }
    }

    #[test]
    fn la_prueba_entera_al_azar_es_err_y_nunca_panico() {
        let presupuesto = std::env::var("ARQUEO_SIN_PANICO_PRESUPUESTO")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(PRESUPUESTO);
        let mut c = Cuenta::default();
        let mut azar = Azar(SEMILLA);
        for nombre in VECTORES {
            let p = vector(nombre);
            let largo = hex_a_bytes(p["prueba"].as_str().unwrap()).unwrap().len();
            let m: Vec<Vec<(usize, u8)>> = (0..presupuesto)
                .map(|_| {
                    let k = 1 + (azar.siguiente() % 3) as usize;
                    (0..k)
                        .map(|_| {
                            let i = (azar.siguiente() % largo as u64) as usize;
                            (i, azar.siguiente() as u8)
                        })
                        .collect()
                })
                .collect();
            barrer(nombre, &m, &mut c);
        }
        eprintln!("sin panico azar: {} mutaciones (semilla {SEMILLA:#x})", c.mutaciones);
        c.exigir_limpio("el azar");
    }
}

/// **§713 · EL RESTO FRI, ATADO A SU COMPROMISO.**
///
/// El probador se compromete con el resto: el ultimo compromiso FRI es `hash_elements(resto)`.
/// `winter-fri` 0.13.1 no lo comprueba al verificar: ese compromiso solo sembraba la moneda, y una
/// prueba que se compromete con OTRO resto y lleva el de verdad salia VERDE. Lo comprueba ahora el
/// canal del fork (`crates/winter-verifier/src/channel.rs`, `read_remainder`), antes del grado y
/// del plegado, con la variante que `winter-fri` define y no usaba. Los falsadores que fabrican esa
/// prueba sin tocar el probador viven en `stark-experiment` (`falsadores_resto.rs`); aqui, el kit.
#[cfg(test)]
mod tests_resto {
    use super::tests_puerta::{juez_de, juzgar, vector, VECTORES};
    use super::tests_sin_panico::recorrer;
    use super::*;

    const CAUSA: &str = "FriVerificationFailed(RemainderCommitmentMismatch)";

    /// El vector capturado (`tools/vectores/resto_ajeno.sh`): el ultimo compromiso FRI es el hash
    /// de OTRO resto. Con el kit de antes del §713 salia VERDE, salida 0; ahora es ROJO con su
    /// causa, y no por la red de los panicos.
    #[test]
    fn un_resto_que_no_es_el_comprometido_es_rojo_con_su_causa() {
        let ruta = format!(
            "{}/../../spec/vectors/edad/neg-prueba-resto-ajeno.json",
            env!("CARGO_MANIFEST_DIR")
        );
        let e = correr(&ruta).unwrap_err();
        assert_eq!(e, format!("edad: {CAUSA}"));
        assert_eq!(codigo_de_salida(&Err(e)), 1);
    }

    /// Un bit del resto de cada prueba de la puerta del §711 -edad, pago, prenda y cobro-: cae por
    /// su compromiso, que se mira antes que el plegado. Con el kit de antes del §713, la de edad
    /// caia por `InvalidRemainderFolding` (medido, y lo mismo en la segunda implementacion).
    #[test]
    fn un_bit_del_resto_cae_por_su_compromiso_antes_que_por_el_plegado() {
        for nombre in VECTORES {
            let p = vector(nombre);
            let j = juez_de(&p);
            let mut b = hex_a_bytes(p["prueba"].as_str().unwrap()).unwrap();
            assert_eq!(juzgar(&j, &b), Ok(()), "{nombre}: la de partida no verifica");
            let i = recorrer(&b).resto_en + 2;
            b[i] ^= 1;
            assert_eq!(juzgar(&j, &b), Err(CAUSA.to_string()), "{nombre}");
        }
    }
}

/// §715 (la maleabilidad de los bytes de la prueba): una prueba tiene UNA escritura. Sin
/// el testigo, de los bytes de una prueba salian otros que verificaban igual: cada `usize` escrito
/// de mas (el `read_usize` de winter-utils acepta cualquier escritura del mismo valor), una fila
/// de mas en las tablas o un grupo de mas en una capa FRI (`get_root` no mira si sobran hojas).
/// Medido con el kit de antes de este arreglo: VERDE y salida 0, con otros bytes y otra huella.
/// Estos testigos reescriben las cuatro pruebas de la puerta (`tests_puerta`) pieza a pieza -con
/// un lector que recuerda cuantos bytes ocupa cada `usize`- y exigen ROJO con su causa en cada
/// forma.
#[cfg(test)]
mod tests_maleabilidad {
    use super::tests_puerta::{juez_de, juzgar, vector, Juez, VECTORES};
    use super::*;

    /// Un `usize` de la prueba: su valor y cuantos bytes de mas lleva sobre su escritura minima.
    #[derive(Clone)]
    struct Vint {
        v: u64,
        de_mas: usize,
    }

    fn largo_minimo(v: u64) -> usize {
        (1..9).find(|n| v < (1u64 << (7 * n))).unwrap_or(9)
    }

    impl Vint {
        fn de(v: u64) -> Self {
            Vint { v, de_mas: 0 }
        }

        fn leer(b: &[u8], i: &mut usize) -> Self {
            let n = b[*i].trailing_zeros() as usize + 1;
            let v = if n == 9 {
                u64::from_le_bytes(b[*i + 1..*i + 9].try_into().unwrap())
            } else {
                let mut a = [0u8; 8];
                a[..n].copy_from_slice(&b[*i..*i + n]);
                u64::from_le_bytes(a) >> n
            };
            *i += n;
            Vint { v, de_mas: n - largo_minimo(v) }
        }

        fn escribir(&self, o: &mut Vec<u8>) {
            let n = (largo_minimo(self.v) + self.de_mas).min(9);
            if n == 9 {
                o.push(0);
                o.extend_from_slice(&self.v.to_le_bytes());
            } else {
                o.extend_from_slice(&((((self.v << 1) | 1) << (n - 1)).to_le_bytes())[..n]);
            }
        }
    }

    fn tomar<'a>(b: &'a [u8], i: &mut usize, n: usize) -> &'a [u8] {
        *i += n;
        &b[*i - n..*i]
    }

    fn u16_de(b: &[u8], i: &mut usize) -> usize {
        u16::from_le_bytes(tomar(b, i, 2).try_into().unwrap()) as usize
    }

    fn u32_de(b: &[u8], i: &mut usize) -> usize {
        u32::from_le_bytes(tomar(b, i, 4).try_into().unwrap()) as usize
    }

    /// El lote con sal (`VariasConSal`): `depth`, los vectores de nodos con sus cuentas y las sales.
    #[derive(Clone)]
    struct Lote {
        depth: u8,
        vectores: Vint,
        nodos: Vec<(Vint, Vec<u8>)>,
        sales: Vec<u8>,
    }

    impl Lote {
        fn leer(b: &[u8]) -> Self {
            let mut i = 0;
            let depth = tomar(b, &mut i, 1)[0];
            let vectores = Vint::leer(b, &mut i);
            let mut nodos = Vec::new();
            for _ in 0..vectores.v {
                let k = Vint::leer(b, &mut i);
                let n = 32 * k.v as usize;
                nodos.push((k, tomar(b, &mut i, n).to_vec()));
            }
            let n = u32_de(b, &mut i);
            let sales = tomar(b, &mut i, 32 * n).to_vec();
            assert_eq!(i, b.len(), "el lote acaba donde dice");
            Lote { depth, vectores, nodos, sales }
        }

        fn escribir(&self) -> Vec<u8> {
            let mut o = vec![self.depth];
            let mut vectores = self.vectores.clone();
            vectores.v = self.nodos.len() as u64;
            vectores.escribir(&mut o);
            for (k, nodos) in &self.nodos {
                let mut k = k.clone();
                k.v = (nodos.len() / 32) as u64;
                k.escribir(&mut o);
                o.extend_from_slice(nodos);
            }
            o.extend_from_slice(&((self.sales.len() / 32) as u32).to_le_bytes());
            o.extend_from_slice(&self.sales);
            o
        }

        fn sitios(&mut self) -> Vec<&mut Vint> {
            let mut s = vec![&mut self.vectores];
            s.extend(self.nodos.iter_mut().map(|(k, _)| k));
            s
        }
    }

    /// Unas consultas (`Queries`): los valores y la apertura, cada uno con su longitud.
    #[derive(Clone)]
    struct Consultas {
        largo_valores: Vint,
        valores: Vec<u8>,
        largo_apertura: Vint,
        lote: Lote,
    }

    /// Una prueba de la casa, en el orden de `Proof::write_into` del fork.
    #[derive(Clone)]
    struct Prueba {
        cabeza: Vec<u8>,
        restricciones: Vint,
        unicas: u8,
        compromisos: Vec<u8>,
        consultas: Vec<Consultas>,
        ood: Vec<u8>,
        capas: Vec<(Vec<u8>, Lote)>,
        cola: Vec<u8>,
    }

    impl Prueba {
        fn leer(b: &[u8]) -> Self {
            let mut i = 4;
            i += 2 + u16::from_le_bytes([b[4], b[5]]) as usize; // meta
            i += 1 + b[i] as usize; // el modulo
            i += 10; // las opciones
            let cabeza = b[..i].to_vec();
            let restricciones = Vint::leer(b, &mut i);
            let unicas = tomar(b, &mut i, 1)[0];
            let desde = i;
            let n = u16_de(b, &mut i);
            tomar(b, &mut i, n);
            let compromisos = b[desde..i].to_vec();
            let mut consultas = Vec::new();
            for _ in 0..2 + usize::from(b[1] != 0) {
                let largo_valores = Vint::leer(b, &mut i);
                let valores = tomar(b, &mut i, largo_valores.v as usize).to_vec();
                let largo_apertura = Vint::leer(b, &mut i);
                let lote = Lote::leer(tomar(b, &mut i, largo_apertura.v as usize));
                consultas.push(Consultas { largo_valores, valores, largo_apertura, lote });
            }
            let desde = i;
            for _ in 0..2 {
                let n = u16_de(b, &mut i);
                tomar(b, &mut i, n);
            }
            let ood = b[desde..i].to_vec();
            let mut capas = Vec::new();
            for _ in 0..tomar(b, &mut i, 1)[0] {
                let n = u32_de(b, &mut i);
                let valores = tomar(b, &mut i, n).to_vec();
                let n = u32_de(b, &mut i);
                capas.push((valores, Lote::leer(tomar(b, &mut i, n))));
            }
            let cola = b[i..].to_vec();
            Prueba { cabeza, restricciones, unicas, compromisos, consultas, ood, capas, cola }
        }

        fn escribir(&self) -> Vec<u8> {
            let mut o = self.cabeza.clone();
            self.restricciones.escribir(&mut o);
            o.push(self.unicas);
            o.extend_from_slice(&self.compromisos);
            for c in &self.consultas {
                let mut lv = c.largo_valores.clone();
                lv.v = c.valores.len() as u64;
                lv.escribir(&mut o);
                o.extend_from_slice(&c.valores);
                let lote = c.lote.escribir();
                let mut la = c.largo_apertura.clone();
                la.v = lote.len() as u64;
                la.escribir(&mut o);
                o.extend(lote);
            }
            o.extend_from_slice(&self.ood);
            o.push(self.capas.len() as u8);
            for (valores, lote) in &self.capas {
                o.extend_from_slice(&(valores.len() as u32).to_le_bytes());
                o.extend_from_slice(valores);
                let lote = lote.escribir();
                o.extend_from_slice(&(lote.len() as u32).to_le_bytes());
                o.extend(lote);
            }
            o.extend_from_slice(&self.cola);
            o
        }

        /// Todos los `usize` de la prueba, en su orden: las restricciones, las dos longitudes de
        /// cada `Queries` con las cuentas de su lote, y las cuentas del lote de cada capa FRI.
        fn sitios(&mut self) -> Vec<&mut Vint> {
            let mut s = vec![&mut self.restricciones];
            for c in &mut self.consultas {
                let Consultas { largo_valores, largo_apertura, lote, .. } = c;
                s.push(largo_valores);
                s.push(largo_apertura);
                s.extend(lote.sitios());
            }
            for (_, lote) in &mut self.capas {
                s.extend(lote.sitios());
            }
            s
        }
    }

    /// Cada prueba de la puerta, con su juez; y el lector de arriba, fiel: escribe los mismos
    /// bytes que lee, y la prueba sin tocar verifica.
    fn pruebas() -> Vec<(&'static str, Juez, Vec<u8>, Prueba)> {
        VECTORES
            .iter()
            .map(|nombre| {
                let p = vector(nombre);
                let j = juez_de(&p);
                let b = hex_a_bytes(p["prueba"].as_str().unwrap()).unwrap();
                let leida = Prueba::leer(&b);
                assert_eq!(leida.escribir(), b, "{nombre}: el lector escribe lo que lee");
                assert_eq!(juzgar(&j, &b), Ok(()), "{nombre}: la prueba sin tocar verifica");
                (*nombre, j, b, leida)
            })
            .collect()
    }

    /// Juzga `m` y exige ROJO con `causa`, sin panico y sin la marca de la red.
    fn exigir_rojo(j: &Juez, m: &[u8], buena: &[u8], que: &str, causa: &str) {
        assert_ne!(m, buena, "{que}: la mutacion cambia los bytes");
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| juzgar(j, m)))
            .unwrap_or_else(|_| panic!("CRITICO: {que} entra en panico"));
        match r {
            Ok(()) => panic!("{que}: VERIFICA, con otros bytes"),
            Err(e) => {
                assert!(e.contains(causa), "{que}: pide {causa}, da {e}");
                assert!(!e.contains(zk_ssl_air::PANICO_RECOGIDO), "{que}: lo recogio la red: {e}");
            }
        }
    }

    /// Cada `usize` de las cuatro pruebas, escrito con un byte de mas y con nueve: ROJO con la
    /// causa del `usize`. Con el kit de 161a233, todas VERDE (medido con su binario: 1.320 de
    /// 1.320 recodificaciones de un sitio en `edad-todos.json`, 1.363 de 1.363 en `pago-t-seq`).
    /// Falsador: con `read_usize` de upstream en el lector del fork y en el lote con sal, todas
    /// verifican.
    #[test]
    fn cada_usize_escrito_de_mas_es_rojo_con_su_causa() {
        let mut n = 0;
        for (nombre, j, buena, leida) in pruebas() {
            let sitios = leida.clone().sitios().len();
            for k in 0..sitios {
                for de_mas in [1usize, 8] {
                    let mut p = leida.clone();
                    {
                        let mut sitios = p.sitios();
                        let s = &mut sitios[k];
                        if largo_minimo(s.v) == 9 {
                            continue;
                        }
                        s.de_mas = de_mas;
                    }
                    let que = format!("{nombre}, usize {k} con {de_mas} bytes de mas");
                    exigir_rojo(&j, &p.escribir(), &buena, &que, zk_ssl_air::causa::USIZE);
                    n += 1;
                }
            }
        }
        eprintln!("maleabilidad: {n} usize escritos de mas, todos ROJO con su causa");
        assert!(n > 1000, "{n}");
    }

    /// El numero de restricciones mas 2^32, en su escritura minima: `Context::to_elements` lo
    /// escribe `as u32`, y daba el mismo transcript. ROJO con la causa del contexto, que ya puso
    /// el §711; aqui se vigila como forma de la maleabilidad.
    #[test]
    fn las_restricciones_mas_dos_a_la_32_son_rojo_con_su_causa() {
        for (nombre, j, buena, leida) in pruebas() {
            let mut p = leida.clone();
            p.restricciones = Vint::de(p.restricciones.v + (1u64 << 32));
            let que = format!("{nombre}, restricciones + 2^32");
            exigir_rojo(&j, &p.escribir(), &buena, &que, zk_ssl_air::causa::CONTEXTO);
        }
    }

    /// Una fila de mas -una copia de la primera- en cada tabla, con su sal, y una consulta unica
    /// mas, donde cabe (las unicas no pasan de las 42 de las opciones). ROJO: la fila no casa con
    /// ninguna posicion consultada. Antes, VERDE en `edad-todos.json` (40 unicas): medido con el
    /// kit de antes de este arreglo, 75.947 -> 77.051 bytes.
    #[test]
    fn una_fila_de_mas_es_rojo() {
        let mut n = 0;
        for (nombre, j, buena, leida) in pruebas() {
            if leida.unicas >= 42 {
                continue;
            }
            let mut p = leida.clone();
            p.unicas += 1;
            for c in &mut p.consultas {
                let fila = c.valores.len() / leida.unicas as usize;
                let primera = c.valores[..fila].to_vec();
                c.valores.extend(primera);
                let sal = c.lote.sales[..32].to_vec();
                c.lote.sales.extend(sal);
            }
            let que = format!("{nombre}, una fila de mas");
            exigir_rojo(&j, &p.escribir(), &buena, &que, "TraceQueryDoesNotMatchCommitment");
            n += 1;
        }
        assert!(n >= 1, "al menos edad-todos.json tiene sitio para una fila");
    }

    /// Un grupo de mas -una copia del primero- en cada capa FRI, con su sal. ROJO con la causa de
    /// una capa que no casa con su compromiso. Antes, VERDE en edad y en pago (medido).
    #[test]
    fn un_grupo_fri_de_mas_es_rojo() {
        for (nombre, j, buena, leida) in pruebas() {
            for capa in 0..leida.capas.len() {
                let mut p = leida.clone();
                let (valores, lote) = &mut p.capas[capa];
                let grupo = valores[..8 * 16].to_vec(); // el plegado de la casa, en la cuadratica
                valores.extend(grupo);
                let sal = lote.sales[..32].to_vec();
                lote.sales.extend(sal);
                let que = format!("{nombre}, un grupo de mas en la capa FRI {capa}");
                exigir_rojo(&j, &p.escribir(), &buena, &que, "LayerCommitmentMismatch");
            }
        }
    }

    /// Un nodo de mas al final del primer vector de cada lote (§659, SEG-03b), en las tablas y en
    /// las capas FRI: ROJO, cada uno con la causa de lo que abre.
    #[test]
    fn un_nodo_de_mas_en_cualquier_lote_es_rojo() {
        for (nombre, j, buena, leida) in pruebas() {
            let causas = ["TraceQueryDoesNotMatchCommitment", "ConstraintQueryDoesNotMatchCommitment"];
            let n = leida.consultas.len();
            for k in 0..n + leida.capas.len() {
                let mut p = leida.clone();
                let lote = if k < n { &mut p.consultas[k].lote } else { &mut p.capas[k - n].1 };
                let Some((_, nodos)) = lote.nodos.first_mut() else { continue };
                if nodos.is_empty() {
                    continue;
                }
                let otro = nodos[..32].to_vec();
                nodos.extend(otro);
                let causa = if k + 1 < n { causas[0] } else if k + 1 == n { causas[1] } else {
                    "LayerCommitmentMismatch"
                };
                let que = format!("{nombre}, un nodo de mas en el lote {k}");
                exigir_rojo(&j, &p.escribir(), &buena, &que, causa);
            }
        }
    }
}

/// **§717, el coste del juez de la edad.** El mando no fija la clave del operador: verifica
/// la cabeza con la que ella trae. Un paquete de edad con una cabeza firmada con una clave propia y
/// una prueba que solo tiene la forma elige `m` (por `nextPending`), y el juez pagaba un coste
/// lineal en `2^m` antes de la primera comprobacion contra la prueba. Los cuatro negativos de
/// `spec/vectors/edad/` que lo cubren (tres del §717 y uno del §721) se rechazan con su causa y en
/// el tiempo de uno honesto.
#[cfg(test)]
mod tests_coste_edad {
    use std::time::{Duration, Instant};

    use super::tests_puerta::vector;
    use super::*;

    /// El menor de tres rechazos del mando (el menor resiste la carga de la maquina) y su error.
    fn por_el_mando(p: &serde_json::Value) -> (Duration, Result<(), String>) {
        let mut mejor = Duration::MAX;
        let mut r = Ok(());
        for _ in 0..3 {
            let t0 = Instant::now();
            r = verificar_edad(p);
            mejor = mejor.min(t0.elapsed());
        }
        (mejor, r)
    }

    /// Los cuatro paquetes de atacante caen con su causa: sobre el tope de `m` (`m = 17`, y
    /// `m = 23`, la mayor que el lector de la prueba admite), por el enunciado y antes de leer la
    /// prueba; en el tope (`m = 13`), en la coherencia OOD. Las `m` las fija cada vector, no el
    /// tope: las tres primeras causas valen con cualquier tope de 13 a 16. La cuarta (§721),
    /// `m = 14` (`nextPending` = `2^13 + 1`, la marca mas pequena sobre el tope), cae por el tope
    /// solo si el tope es 13: con 14 llegaria a la coherencia OOD. Con la de `m = 13`, fija el tope
    /// en 13. Y ninguno cuesta mas de cinco veces lo que el paquete honesto de `edad-todos.json`
    /// (`m = 3`, una prueba real y su firma): con el kit de antes de este arreglo, el de `m = 17`
    /// costaba mas de un segundo de CPU y el de `m = 23` mas de un minuto (medido con el mando). El
    /// tiempo solo se exige sin las aserciones de depuracion: con ellas el test no sabe si corre
    /// optimizado -el perfil `aserciones` del «1 bis» del canon (§864), con los pines de la fila del
    /// kit- o en depuracion, donde el cronometro no mide el juez de release; ahi comprueba las
    /// causas, y no se ignora.
    #[test]
    fn un_paquete_de_edad_de_m_alto_se_rechaza_en_milisegundos() {
        let (t_honesto, r) = por_el_mando(&vector("edad/edad-todos.json"));
        assert_eq!(r, Ok(()), "el honesto");
        let casos = [
            ("edad/neg-m-sobre-tope.json", "edad: m = 17 fuera de 3..="),
            ("edad/neg-m-mayor-legible.json", "edad: m = 23 fuera de 3..="),
            ("edad/neg-m-13-fabricada.json", "edad: InconsistentOodConstraintEvaluations"),
            ("edad/neg-m-14-sobre-tope.json", "edad: m = 14 fuera de 3..="),
        ];
        for (nombre, causa) in casos {
            let (t, r) = por_el_mando(&vector(nombre));
            let e = r.expect_err(nombre);
            assert!(e.starts_with(causa), "{nombre}: {e}");
            let cociente = t.as_secs_f64() / t_honesto.as_secs_f64();
            eprintln!("{nombre}: {t:?} frente a {t_honesto:?} del honesto ({cociente:.1})");
            if !cfg!(debug_assertions) {
                assert!(cociente < 5.0, "{nombre}: {t:?} frente a {t_honesto:?} ({cociente:.1})");
            }
        }
    }
}

/// **§718, la canonicidad u64 -> campo, con las pruebas REALES.** Los sobres firmados de
/// `spec/vectors/` -pago, cobro pendiente y edad-, sus jueces armados con los mismos lectores que
/// el mando, y cada entero de la afirmacion cambiado por su alias `x + p` DESPUES del lector: es
/// lo que recibe la API de `zk-ssl-air` cuando la llama un integrador sin el lector del kit.
/// Medido antes del arreglo: cada alias daba `Ok` con la prueba del honesto, y el enunciado
/// devuelto llevaba el valor reducido mientras el llamante tenia el alias en la mano. Ahora cada
/// uno es `Err` con su causa, ANTES de leer la prueba; el honesto sigue `Ok` y el control `+1`
/// sigue cayendo en la prueba (el alias cae por la cota, no por otra cosa).
///
/// ⚠️ Al portarlo (§718): se escribieron con `cfg_attr(debug_assertions, ignore = …)`. Solo
/// verifican pruebas ya hechas, sin el probador, y desde el §864 el canon corre el kit tambien con
/// el perfil `aserciones`, con los pines de su fila: corren con las aserciones encendidas, y no se
/// ignoran.
#[cfg(test)]
mod tests_canonicidad {
    use super::tests_puerta::{juez_de, juzgar, vector, Juez};
    use super::*;
    use zk_ssl_air::pago_en_curso::MAX_VALOR;
    use zk_ssl_air::EdadPublicInputs;

    const P: u64 = 0xffff_ffff_0000_0001;
    const OOD: &str = "InconsistentOodConstraintEvaluations";

    fn prueba_de(p: &serde_json::Value) -> Vec<u8> {
        hex_a_bytes(p["prueba"].as_str().unwrap()).unwrap()
    }

    #[test]
    fn pago_importe_y_t_alias_son_err_con_la_prueba_real() {
        let p = vector("pago/pago-t-frontera.json");
        let prueba = prueba_de(&p);
        let Juez::Pago(af, c) = juez_de(&p) else { panic!("no es un pago") };
        assert_eq!(enlazar_pago(&prueba, &af, &c).map(|pi| pi.importe.as_int()), Ok(af.importe));
        let casos = [
            (AfirmacionPago { importe: af.importe + P, ..af }, "el importe"),
            (AfirmacionPago { t: af.t + P, ..af }, "T "),
            (AfirmacionPago { importe: af.importe + P, t: af.t + P, ..af }, "el importe"),
        ];
        for (malo, causa) in casos {
            let e = enlazar_pago(&prueba, &malo, &c).expect_err("el alias verifico");
            assert!(e.starts_with(causa) && e.ends_with(&format!("techo {MAX_VALOR}")), "{e}");
        }
        let control = AfirmacionPago { importe: af.importe + 1, ..af };
        assert_eq!(enlazar_pago(&prueba, &control, &c).unwrap_err(), OOD);
    }

    #[test]
    fn cobro_inferior_alias_es_err_con_la_prueba_real() {
        for nombre in ["pendiente/cobro-inferior-importe.json", "pendiente/cobro-inferior-0.json"] {
            let p = vector(nombre);
            let prueba = prueba_de(&p);
            let Juez::Cobro(af, c) = juez_de(&p) else { panic!("no es un cobro") };
            assert!(enlazar_cobro(&prueba, &af, &c).is_ok(), "{nombre}: el honesto");
            let malo = AfirmacionCobro { inferior: af.inferior + P, ..af };
            let e = enlazar_cobro(&prueba, &malo, &c).expect_err("el alias verifico");
            let causa = format!("las cotas {} y {MAX_VALOR} pasan del techo", af.inferior + P);
            assert!(e.starts_with(&causa), "{nombre}: {e}");
            let control = AfirmacionCobro { inferior: af.inferior + 1, ..af };
            assert_eq!(enlazar_cobro(&prueba, &control, &c).unwrap_err(), OOD, "{nombre}");
        }
    }

    /// La prueba de todos con `emisor = p` (el alias de 0) por `zk_ssl_air::verificar` directo, y
    /// el sobre de todos con un emisor nombrado que no es un indice de cuenta.
    #[test]
    fn edad_alias_del_emisor_es_err_con_la_prueba_real() {
        let p = vector("edad/edad-todos.json");
        let prueba = prueba_de(&p);
        let Juez::Edad(af, c) = juez_de(&p) else { panic!("no es de edad") };
        let pi = verificar_contra_cabeza(&prueba, &af, &c).expect("el honesto");
        assert!(pi.todos && pi.emisor == 0);
        let alias = EdadPublicInputs { emisor: P, ..pi.clone() };
        let e = zk_ssl_air::verificar(&prueba, &alias).expect_err("todos con emisor = p");
        assert!(e.starts_with(&format!("emisor = {P} no es un indice de cuenta")), "{e}");
        let control = EdadPublicInputs { emisor: 1, ..pi };
        let e = zk_ssl_air::verificar(&prueba, &control).expect_err("todos con emisor = 1");
        assert!(e.starts_with("todos con emisor = 1"), "{e}");
        for x in [P, 1 << 32] {
            let malo = Afirmacion { emisor: Some(x), ..af };
            let e = verificar_contra_cabeza(&prueba, &malo, &c).expect_err("emisor fuera");
            assert!(e.starts_with(&format!("emisor = {x} no es un indice de cuenta")), "{e}");
        }
        assert!(juzgar(&Juez::Edad(af, c), &prueba).is_ok());
    }
}
