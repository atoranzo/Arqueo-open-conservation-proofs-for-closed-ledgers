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

/// Los límites de época que el diario conserva: los `seq` de sus líneas,
/// en el orden en que se anotaron.
///
/// **Las cabezas sin firmar también delimitan** — para el límite basta
/// el `seq`, y la nota de arriba ya lo decía: se anotan justo para esto.
/// Las líneas ilegibles se **saltan**: una línea corrupta cuesta una
/// época gorda en la lectura, no un pánico ni un diario inservible.
pub fn limites(ruta: impl AsRef<Path>) -> Vec<u64> {
    let texto = match std::fs::read_to_string(ruta) {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };
    let mut v = Vec::new();
    for l in texto.lines() {
        let j: Value = match serde_json::from_str(l) {
            Ok(j) => j,
            Err(_) => continue,
        };
        if let Some(s) = j["seq"].as_str() {
            if let Ok(x) = u64::from_str_radix(s.trim_start_matches("0x"), 16) {
                v.push(x);
            }
        }
    }
    v
}

/// El último `seq` anotado, para `limite_de_epoca` cuando la memoria
/// (`ultima_cabeza`) llega vacía tras un reinicio: **P sale del diario;
/// la memoria es caché** (§275).
pub fn ultimo_seq(ruta: impl AsRef<Path>) -> Option<u64> {
    limites(ruta).into_iter().last()
}

/// El MAXIMO `index` anotado, o `None` si el diario no tiene ni una firma.
///
/// ⚠️⚠️ **MAXIMO y no ULTIMO**, a diferencia de [`ultimo_seq`]: el caso
/// del que esto defiende -un contador restaurado hacia atras- hace que el nodo
/// escriba indices MENORES detras de mayores, asi que agregar por el ultimo
/// seria medir con un instrumento que el propio caso desarma.
///
/// ⚠️ Este maximo puede quedar POR DEBAJO del real por dos vias, las dos a
/// proposito: [`anotar`] no hace `fsync`, y las lineas ilegibles se SALTAN como
/// en [`limites`]. Las dos empujan al mismo lado: quien lo use falla hacia el
/// lado PERMISIVO -deja arrancar-, nunca hacia un rojo falso.
pub fn maximo_indice(ruta: impl AsRef<Path>) -> Option<u64> {
    let texto = match std::fs::read_to_string(ruta) {
        Ok(t) => t,
        Err(_) => return None,
    };
    let mut max: Option<u64> = None;
    for l in texto.lines() {
        let j: Value = match serde_json::from_str(l) {
            Ok(j) => j,
            Err(_) => continue,
        };
        if let Some(s) = j["index"].as_str() {
            if let Ok(x) = u64::from_str_radix(s.trim_start_matches("0x"), 16) {
                if max.map_or(true, |m| x > m) {
                    max = Some(x);
                }
            }
        }
    }
    max
}

#[cfg(test)]
mod maximo_del_diario {
    use super::*;

    fn linea_con(indice: u64) -> String {
        json!({"seq": q(1), "index": q(indice)}).to_string()
    }

    fn en_disco(nombre: &str, cuerpo: &str) -> std::path::PathBuf {
        let p = std::env::temp_dir().join(nombre);
        let _ = std::fs::remove_file(&p);
        std::fs::write(&p, cuerpo).expect("escribir el diario de prueba");
        p
    }

    /// ⚠️⚠️ EL CASO ADVERSARIO: un indice MENOR detras de uno mayor, que
    /// es justo lo que escribe un nodo con el contador retrocedido. Agregar por
    /// el ultimo daria 2 y taparia el retroceso.
    #[test]
    fn se_agrega_por_el_maximo_y_no_por_el_ultimo() {
        let cuerpo = [linea_con(1), linea_con(2), linea_con(3), linea_con(2)].join("\n");
        let p = en_disco("zkssl_max_indice.jsonl", &(cuerpo + "\n"));
        assert_eq!(maximo_indice(&p), Some(3), "tiene que dar el MAXIMO, no el ultimo");
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn sin_firmas_no_hay_indice_que_leer() {
        let cuerpo = json!({"seq": q(1)}).to_string() + "\n";
        let p = en_disco("zkssl_max_indice_vacio.jsonl", &cuerpo);
        assert_eq!(maximo_indice(&p), None, "sin firma no hay indice anotado");
        let _ = std::fs::remove_file(&p);
    }
}

/// Los `epochDigest` que el diario conserva, en orden de anotacion:
/// **las hojas del MMR de cabezas** (§292). Misma doctrina que
/// [`limites`]: el diario manda, la memoria es cache, y una linea
/// ilegible o sin digest legible se SALTA — cuesta una hoja en la
/// lectura, no un panico.
pub fn digests(ruta: impl AsRef<Path>) -> Vec<zk_ssl_verify::acuses::Digest> {
    let texto = match std::fs::read_to_string(ruta) {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };
    let mut v = Vec::new();
    for l in texto.lines() {
        let j: Value = match serde_json::from_str(l) {
            Ok(j) => j,
            Err(_) => continue,
        };
        let s = match j["epochDigest"].as_str() {
            Some(s) => s,
            None => continue,
        };
        let h = s.trim_start_matches("0x");
        if h.len() != 64 {
            continue;
        }
        let mut b = [0u8; 32];
        let mut mal = false;
        for (i, par) in (0..64).step_by(2).enumerate() {
            match u8::from_str_radix(&h[par..par + 2], 16) {
                Ok(x) => b[i] = x,
                Err(_) => {
                    mal = true;
                    break;
                }
            }
        }
        if mal {
            continue;
        }
        if let Some(dig) = zk_ssl_verify::mmr::hoja_desde_bytes(&b) {
            v.push(dig);
        }
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use zk_ssl_verify::CabezaFirmada;
    use zk_ssl_verify::acuses::as_digest;

    fn cabeza(indice: u64, digest: u8, firmada: bool) -> Latido {
        Latido {
            cabeza: zk_ssl::log::EpochHead {
                seq: 42,
                accounts_root: as_digest(0),
                pending_root: as_digest(0),
                frozen_root: as_digest(0),
                chain_digest: as_digest(0),
                acuses_root: as_digest(0),
                n: 0,
                mmr_cima: as_digest(0),
                mmr_t: 0,
                cons_root: as_digest(0),
                cons_count: 0,
            },
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

}

#[cfg(test)]
mod tests_limites {
    use super::*;
    use zk_ssl_verify::acuses::as_digest;

    fn latido_sin_firma(seq: u64) -> Latido {
        Latido {
            cabeza: zk_ssl::log::EpochHead {
                seq,
                accounts_root: as_digest(0),
                pending_root: as_digest(0),
                frozen_root: as_digest(0),
                chain_digest: as_digest(0),
                acuses_root: as_digest(0),
                n: 0,
                mmr_cima: as_digest(0),
                mmr_t: 0,
                cons_root: as_digest(0),
                cons_count: 0,
            },
            seq,
            epoch_digest: [0x33; 32],
            firma: None,
            emitida_unix: 1_700_000_000,
        }
    }

    #[test]
    fn los_limites_salen_del_diario_y_los_ilegibles_se_saltan() {
        let d = std::path::Path::new("target").join("diario_limites");
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).expect("carpeta");
        let r = d.join("diario.jsonl");
        anotar(&r, &latido_sin_firma(5), &[]).expect("primera");
        {
            use std::io::Write;
            let mut f = std::fs::OpenOptions::new().append(true).open(&r).expect("abrir");
            writeln!(f, "esto no es json").expect("basura");
        }
        anotar(&r, &latido_sin_firma(9), &[]).expect("segunda");
        assert_eq!(limites(&r), vec![5, 9], "la linea corrupta debe SALTARSE");
        assert_eq!(ultimo_seq(&r), Some(9));
    }

    #[test]
    fn los_digests_salen_del_diario_en_orden_y_lo_ilegible_se_salta() {
        // §292: la siembra del MMR lee EXACTAMENTE lo que el diario
        // conserva. Una linea basura cuesta una hoja, no un panico.
        let dir = std::path::Path::new("target").join("diario_digests");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("carpeta");
        let r = dir.join("diario.jsonl");
        let mut a = latido_sin_firma(1);
        a.epoch_digest = [0x11; 32];
        let mut b = latido_sin_firma(2);
        b.epoch_digest = [0x22; 32];
        anotar(&r, &a, &[]).expect("primera");
        {
            use std::io::Write;
            let mut f = std::fs::OpenOptions::new().append(true).open(&r).expect("abrir");
            writeln!(f, "esto no es json").expect("basura");
        }
        anotar(&r, &b, &[]).expect("segunda");
        let v = digests(&r);
        assert_eq!(v.len(), 2, "dos hojas, la basura saltada");
        assert_ne!(v[0], v[1], "el orden y el contenido deben conservarse");
    }

    #[test]
    fn sin_diario_no_hay_limites_ni_panico() {
        // El RPC preguntara por rutas que pueden no existir todavia: la
        // respuesta correcta es vacio, y el arm lo convierte en reason.
        let r = std::path::Path::new("target").join("diario_inexistente.jsonl");
        let _ = std::fs::remove_file(&r);
        assert!(limites(&r).is_empty());
        assert_eq!(ultimo_seq(&r), None);
    }
}
