//! # El diario del nodo: **lo que firmó**, no cuántas veces
//!
//! ## Por qué existe
//!
//! El guardián de §234 persiste **ocho bytes** —el contador de índice—
//! con `fsync` y autocomprobación de arranque, porque **reusar un índice
//! compromete la clave**. Junto a él, `App.ultima_cabeza` guardaba la
//! última cabeza en un `Mutex<Option<_>>`: cada latido pisaba al anterior
//! y un reinicio borraba la única memoria de lo emitido.
//!
//! Dicho de una vez: **el nodo guardaba el número de sus firmas y no las
//! firmas**. Sabía cuántas veces usó la clave, no para qué.
//!
//! Y el único que conservaba historia de lo que el operador firmó era
//! **quien vigila al operador**: el diario del testigo (§240). Para un
//! sistema construido sobre oponibilidad eso no es un detalle de
//! implementación — el firmante no podía reconocer su propia firma, ni
//! detectar que le atribuyeran una que no emitió.
//!
//! ## El formato no se inventa: ya viajaba
//!
//! ⚠️ El testigo **no comparte un struct** con el nodo: **transcribe**
//! las claves de lo que el nodo le sirve en `zkssl_signedEpochHead`. Así
//! que el núcleo comparable **ya estaba definido y las dos orillas ya lo
//! usaban**. Este diario escribe esas mismas claves.
//!
//! No se comparte la LÍNEA, sólo la CARGA: el testigo envuelve con su
//! `clase` y su `vistoUnix`; aquí no hay envoltura. Compartir la línea
//! entre dos cosas que **casi** son la misma garantizaría un campo que
//! miente en una de las dos orillas — el testigo anota lo que RECIBIÓ, el
//! nodo lo que FIRMÓ.
//!
//! ## Qué se anota y qué no
//!
//! Se anota **lo que se firmó**, no lo que el operador afirma sobre sí
//! mismo: `custody` y `beatSeconds` son aseveraciones suyas y **no
//! entran**.
//!
//! Se anotan **también las cabezas sin firmar**, y es seguro por una
//! razón medida: `comparar_lineas` **salta las líneas sin `signature`**,
//! así que no ensucian ninguna comparación. A cambio dejan registrado el
//! **límite de época**, que es lo que necesita quien quiera saber dónde
//! empieza y acaba una época.
//!
//! ## ⚠️ Esto NO lleva el candado del guardián
//!
//! `PersistenciaFalsa` existe porque **reusar un índice compromete la
//! clave**. Perder este diario **no compromete nada**: deja al nodo sin
//! reconocer su firma. Son categorías distintas, y copiar aquí aquel
//! candado sería ponerle una cerradura que no le toca.
//!
//! ## Lo que cae de regalo
//!
//! `comparar_lineas` ya detecta **contradicción interna** —mismo índice,
//! distinto digest—. Sobre este diario eso es **detección a posteriori de
//! reúso de índice XMSS**: el guardián lo evita a priori, el diario lo
//! delata después. Dos líneas independientes sobre lo único que
//! compromete la clave.

use crate::latido::Latido;
use serde_json::{json, Value};
use std::io::Write;
use std::path::Path;

/// Versión del formato de línea. Sube si cambia el significado de un
/// campo, no si se añade uno.
pub const DIARIO_VERSION: u64 = 1;

/// Cantidad en hexadecimal, **como el cable** (`{:#x}`).
///
/// ⚠️ No es cosmético: el testigo lee cantidades con
/// `u64::from_str_radix(s.trim_start_matches("0x"), 16)`. Escribir
/// decimales aquí haría ilegible el diario para la herramienta que
/// tiene que compararlo.
fn q(n: u64) -> String {
    format!("{:#x}", n)
}

/// La línea que se anota por cada latido.
///
/// Las cuatro claves de firma —`formatVersion`, `index`, `signature`,
/// `publicKey`— **faltan** si el nodo arrancó sin `--clave`. Esa ausencia
/// es lo que hace que `comparar_lineas` salte la línea, y es deliberada.
pub fn linea(l: &Latido, clave_publica: &[u8]) -> Value {
    let mut v = json!({
        "v": DIARIO_VERSION,
        "seq": q(l.seq),
        "epochDigest": format!("0x{}", crate::hex_de(&l.epoch_digest)),
        "emittedAtUnix": q(l.emitida_unix),
    });
    if let Some(c) = l.firma.as_ref() {
        v["formatVersion"] = json!(q(u64::from(c.version_formato)));
        v["index"] = json!(q(c.indice));
        v["signature"] = json!(format!("0x{}", crate::hex_de(&c.firma)));
        v["publicKey"] = json!(format!("0x{}", crate::hex_de(clave_publica)));
    }
    v
}

/// Añade una línea al final del diario. **Nunca reescribe.**
///
/// ⚠️ Sin `fsync`, y a propósito: ver la nota de arriba sobre el candado
/// del guardián. Un latido que no llegue al disco cuesta una línea, no
/// una clave.
pub fn anotar(ruta: impl AsRef<Path>, l: &Latido, clave_publica: &[u8]) -> std::io::Result<()> {
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(ruta)?;
    writeln!(f, "{}", linea(l, clave_publica))
}

/// **La comprobación DIRIGIDA**: índices del testigo ausentes del diario
/// del nodo.
///
/// ⚠️ **Y sólo esa dirección.** El diario del nodo es COMPLETO —uno por
/// latido— y el del testigo es MUESTREADO —sólo lo que pidió—, así que el
/// nodo tendrá siempre líneas que el testigo no tiene y eso **no es un
/// hallazgo, es lo normal**. Contarlo daría rojo en cada corrida y
/// acabaría siendo paisaje.
///
/// La ausencia que sí importa significa una de dos cosas, y las dos son
/// graves: **o el nodo firmó algo que no recuerda, o alguien sirvió una
/// firma que el nodo no emitió**.
///
/// ⚠️ Esto es lo que `comparar_lineas` NO hace: mapea por `index` y sólo
/// recorre los presentes en ambos, así que una ausencia le pasa en
/// silencio.
pub fn ausentes(testigo: &[String], nodo: &[String]) -> Vec<u64> {
    let indices = |ls: &[String]| -> std::collections::BTreeSet<u64> {
        let mut s = std::collections::BTreeSet::new();
        for l in ls {
            let v: Value = match serde_json::from_str(l) {
                Ok(v) => v,
                Err(_) => continue,
            };
            if v["signature"].is_null() {
                continue;
            }
            if let Some(t) = v["index"].as_str() {
                if let Ok(i) = u64::from_str_radix(t.trim_start_matches("0x"), 16) {
                    s.insert(i);
                }
            }
        }
        s
    };
    let (t, n) = (indices(testigo), indices(nodo));
    t.difference(&n).copied().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use zk_ssl_verify::CabezaFirmada;

    fn cabeza(indice: u64, digest: u8, firmada: bool) -> Latido {
        Latido {
            seq: 42,
            epoch_digest: [digest; 32],
            firma: if firmada {
                Some(CabezaFirmada { version_formato: 1, indice, firma: vec![0xAB, 0xCD] })
            } else {
                None
            },
            emitida_unix: 1_700_000_000,
        }
    }

    #[test]
    fn la_linea_lleva_las_claves_que_el_nodo_ya_sirve() {
        // El testigo TRANSCRIBE estas claves de la respuesta del nodo. Si
        // aqui se llamaran de otro modo, el nucleo dejaria de ser comun y
        // la comparacion cruzada seria imposible sin traducir.
        let v = linea(&cabeza(7, 0x11, true), &[0x01, 0x02]);
        for k in ["seq", "epochDigest", "emittedAtUnix", "formatVersion", "index", "signature", "publicKey"] {
            assert!(!v[k].is_null(), "falta la clave {k}");
        }
    }

    #[test]
    fn las_cantidades_van_en_hexadecimal_como_el_cable() {
        // El testigo lee con from_str_radix sobre "0x...": decimales aqui
        // harian el diario ilegible para quien tiene que compararlo.
        let v = linea(&cabeza(255, 0x11, true), &[]);
        assert_eq!(v["index"], "0xff", "el indice no va en hexadecimal");
        assert_eq!(v["seq"], "0x2a", "el seq no va en hexadecimal");
    }

    #[test]
    fn una_cabeza_sin_firmar_se_anota_igual_pero_sin_las_claves_de_firma() {
        // Se anota para dejar el LIMITE DE EPOCA registrado. Es seguro
        // porque comparar_lineas salta las lineas sin signature.
        let v = linea(&cabeza(0, 0x22, false), &[]);
        assert!(!v["seq"].is_null(), "el limite de epoca tiene que quedar");
        assert!(v["signature"].is_null(), "sin clave no puede haber firma");
        assert!(v["index"].is_null(), "sin firma no hay indice que anotar");
    }

    #[test]
    fn el_diario_no_anota_lo_que_el_operador_afirma_de_si_mismo() {
        // custody y beatSeconds son aseveraciones del operador, no parte
        // de lo que se firmo. El diario registra lo firmado.
        let v = linea(&cabeza(1, 0x33, true), &[]);
        assert!(v["custody"].is_null(), "custody no es algo firmado");
        assert!(v["beatSeconds"].is_null(), "beatSeconds no es algo firmado");
    }

    #[test]
    fn anotar_anade_y_nunca_reescribe() {
        let d = std::path::Path::new("target").join("diario_anade");
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).expect("carpeta");
        let r = d.join("diario.jsonl");
        anotar(&r, &cabeza(1, 0x11, true), &[]).expect("primera");
        anotar(&r, &cabeza(2, 0x22, true), &[]).expect("segunda");
        let t = std::fs::read_to_string(&r).expect("leer");
        assert_eq!(t.lines().count(), 2, "cada latido es una linea");
        assert!(t.lines().next().expect("primera").contains("0x1"), "la primera no sobrevivio");
    }

    #[test]
    fn cada_linea_es_json_valido_y_de_una_sola_linea() {
        // Un diario que se lee linea a linea no tolera saltos dentro.
        let v = linea(&cabeza(3, 0x44, true), &[0xFF]);
        let s = v.to_string();
        assert!(!s.contains('\n'), "la linea lleva un salto dentro");
        serde_json::from_str::<Value>(&s).expect("no es JSON valido");
    }

    #[test]
    fn ausentes_nombra_la_linea_del_testigo_que_el_nodo_no_tiene() {
        // El caso grave: o el nodo firmo algo que no recuerda, o alguien
        // sirvio una firma que el nodo no emitio.
        let t: Vec<String> = [5u64, 6, 7]
            .iter()
            .map(|i| linea(&cabeza(*i, 0x11, true), &[]).to_string())
            .collect();
        let n: Vec<String> = [5u64, 7]
            .iter()
            .map(|i| linea(&cabeza(*i, 0x11, true), &[]).to_string())
            .collect();
        assert_eq!(ausentes(&t, &n), vec![6], "no nombro el indice ausente");
    }

    #[test]
    fn la_direccion_contraria_no_es_un_hallazgo() {
        // El diario del nodo es COMPLETO y el del testigo MUESTREADO.
        // Contar esta direccion daria rojo en cada corrida.
        let t: Vec<String> = [5u64]
            .iter()
            .map(|i| linea(&cabeza(*i, 0x11, true), &[]).to_string())
            .collect();
        let n: Vec<String> = [5u64, 6, 7]
            .iter()
            .map(|i| linea(&cabeza(*i, 0x11, true), &[]).to_string())
            .collect();
        assert!(ausentes(&t, &n).is_empty(), "el muestreo del testigo no es un hallazgo");
    }
}
