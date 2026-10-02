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
///
/// RFC-0010 E2d (§570): `recepCount` va en TODA línea, firmada o no, como `seq`: es el `Q`
/// de la cabeza siguiente, y tras un reinicio la memoria no lo tiene. Clave añadida, así que
/// `DIARIO_VERSION` no sube.
pub fn linea(l: &Latido, clave_publica: &[u8]) -> Value {
    let mut v = json!({
        "v": DIARIO_VERSION,
        "seq": q(l.seq),
        "epochDigest": format!("0x{}", crate::hex_de(&l.epoch_digest)),
        "emittedAtUnix": q(l.emitida_unix),
        "recepCount": q(l.cabeza.recep_count),
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

/// §666: las lineas del diario, leidas sobre BYTES, cada una por su cuenta.
///
/// ⚠️ Hasta aqui los cinco lectores usaban `read_to_string`, que falla ENTERO
/// si un solo byte no es UTF-8: un `0xFF` en una linea dejaba el diario sin
/// ninguna, y `maximo_indice` daba `None`, asi que el arranque perdia el tope
/// del contador XMSS (medido: `None` frente a `Some(16)`). La doctrina escrita
/// en cada lector -«una linea ilegible se SALTA»- no se cumplia para este
/// caso. Ahora una linea que no es UTF-8 se salta sola, como una que no es
/// JSON. `None` solo si el fichero no se puede leer.
fn lineas_del_diario(ruta: impl AsRef<Path>) -> Option<Vec<String>> {
    let bytes = std::fs::read(ruta).ok()?;
    Some(
        bytes
            .split(|b| *b == b'\n')
            .filter_map(|l| std::str::from_utf8(l).ok())
            .map(|l| l.trim_end_matches('\r').to_string())
            .collect(),
    )
}

/// Los límites de época que el diario conserva: los `seq` de sus líneas,
/// en el orden en que se anotaron.
///
/// **Las cabezas sin firmar también delimitan** — para el límite basta
/// el `seq`, y la nota de arriba ya lo decía: se anotan justo para esto.
/// Las líneas ilegibles se **saltan**: una línea corrupta cuesta una
/// época gorda en la lectura, no un pánico ni un diario inservible.
pub fn limites(ruta: impl AsRef<Path>) -> Vec<u64> {
    let texto = match lineas_del_diario(ruta) {
        Some(t) => t,
        None => return Vec::new(),
    };
    let mut v = Vec::new();
    for l in &texto {
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

/// El `recepCount` de la ÚLTIMA línea que lo lleva (RFC-0010 E2d, §570): el `Q` de la cabeza
/// siguiente cuando la memoria llega vacía tras un reinicio. Mismo criterio que
/// [`ultimo_seq`]: las líneas ilegibles se saltan —cuesta una era gorda en la lectura, no un
/// pánico—, y las anteriores al §570, que no lo llevan, también. Un diario solo de ellas da
/// `None`, y la primera era sale gorda: declarado, como la primera época de acuses.
pub fn ultimo_recep_count(ruta: impl AsRef<Path>) -> Option<u64> {
    let texto = lineas_del_diario(ruta)?;
    let mut ultimo = None;
    for l in &texto {
        let j: Value = match serde_json::from_str(l) {
            Ok(j) => j,
            Err(_) => continue,
        };
        if let Some(s) = j["recepCount"].as_str() {
            if let Ok(x) = u64::from_str_radix(s.trim_start_matches("0x"), 16) {
                ultimo = Some(x);
            }
        }
    }
    ultimo
}

/// Los CIERRES de las eras de recepción (RFC-0010 E3, §571): por cada línea que lleva
/// `recepCount` -desde el §570, todas-, esa cuenta y el `index` de su firma si lo lleva, en el
/// orden en que se anotaron. Las ilegibles y las anteriores al §570 se saltan, como en
/// [`limites`]: una línea perdida junta dos eras en la lectura -una era gorda-, no un pánico.
pub fn cierres_de_recepcion(ruta: impl AsRef<Path>) -> Vec<(u64, Option<u64>)> {
    let texto = match lineas_del_diario(ruta) {
        Some(t) => t,
        None => return Vec::new(),
    };
    let hex = |v: &Value| {
        v.as_str().and_then(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16).ok())
    };
    let mut v = Vec::new();
    for l in &texto {
        let j: Value = match serde_json::from_str(l) {
            Ok(j) => j,
            Err(_) => continue,
        };
        if let Some(c) = hex(&j["recepCount"]) {
            v.push((c, hex(&j["index"])));
        }
    }
    v
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
    let texto = match lineas_del_diario(ruta) {
        Some(t) => t,
        None => return None,
    };
    let mut max: Option<u64> = None;
    for l in &texto {
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

    /// §666: un byte que no es UTF-8 en UNA linea solo se lleva esa linea. Antes
    /// `read_to_string` fallaba entero y el maximo salia `None`: el arranque
    /// perdia el tope del contador XMSS. Falsador: con `read_to_string`, `None`.
    #[test]
    fn un_byte_ilegible_se_lleva_su_linea_y_no_el_diario() {
        let mut cuerpo = Vec::new();
        for (k, i) in [3u64, 16, 9].iter().enumerate() {
            cuerpo.extend_from_slice(linea_con(*i).as_bytes());
            if k == 1 {
                cuerpo.push(0xFF);
            }
            cuerpo.push(b'\n');
        }
        let mut basura = linea_con(40).into_bytes();
        basura[2] = 0xFF;
        cuerpo.extend_from_slice(&basura);
        cuerpo.push(b'\n');
        let p = std::env::temp_dir().join("zkssl_max_indice_ff.jsonl");
        std::fs::write(&p, &cuerpo).expect("escribir el diario de prueba");
        assert_eq!(maximo_indice(&p), Some(9), "las lineas sanas siguen contando; la rota no");
        assert_eq!(limites(&p).len(), 2, "solo se saltan las dos lineas con el byte ilegible");
        let _ = std::fs::remove_file(&p);
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

    /// ⚠️ §636: el diario SI dice de que clave es cada linea firmada, y el maximo NO lo mira.
    /// Los indices de una clave vieja cuentan para la nueva: por eso una clave nueva con el
    /// diario de la vieja no arranca (`politica_de_reconciliacion`). Es la consecuencia
    /// declarada en el §594, con su razon corregida; cambiarla es la entrada 84.
    /// ⚠️ PRECISADO (§638): eso vale con un contador NUEVO. Con el de la vieja, la clave nueva
    /// arranca y sigue su cuenta
    /// (`una_clave_nueva_con_el_contador_de_la_vieja_arranca_y_sigue_su_cuenta`).
    #[test]
    fn el_maximo_no_mira_de_que_clave_es_cada_linea() {
        let con_clave = |indice: u64, clave: &str| {
            json!({"seq": q(1), "index": q(indice), "publicKey": clave}).to_string()
        };
        let cuerpo = [con_clave(9, "0xaa"), con_clave(2, "0xbb")].join("\n");
        let p = en_disco("zkssl_max_indice_claves.jsonl", &(cuerpo + "\n"));
        assert_eq!(
            maximo_indice(&p),
            Some(9),
            "el 9 de la clave 0xaa cuenta para la 0xbb"
        );
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
    let texto = match lineas_del_diario(ruta) {
        Some(t) => t,
        None => return Vec::new(),
    };
    let mut v = Vec::new();
    for l in &texto {
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
        // §650: sobre bytes; una linea con un multibyte se salta, no panica.
        let b: [u8; 32] = match zk_ssl_hash::bytes_de_hex(h).ok().and_then(|v| v.try_into().ok()) {
            Some(b) => b,
            None => continue,
        };
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
                params_digest: as_digest(0),
                pmeta_root: as_digest(0),
                next_pending: 0,
                next_index: 0,
                total_supply: 0,
                recep_root: as_digest(0),
                recep_count: 0,
            },
            seq: 42,
            epoch_digest: [digest; 32],
            firma: if firmada {
                Some(CabezaFirmada { version_formato: 1, indice, firma: vec![0xAB, 0xCD] })
            } else {
                None
            },
            emitida_unix: 1_700_000_000,
            foto: std::sync::Arc::new(zk_ssl::tests_support::new_layer().foto_pendientes()),
        }
    }

    #[test]
    fn la_linea_lleva_las_claves_que_el_nodo_ya_sirve() {
        // El testigo TRANSCRIBE estas claves de la respuesta del nodo. Si
        // aqui se llamaran de otro modo, el nucleo dejaria de ser comun y
        // la comparacion cruzada seria imposible sin traducir.
        let v = linea(&cabeza(7, 0x11, true), &[0x01, 0x02]);
        for k in ["seq", "epochDigest", "emittedAtUnix", "recepCount", "formatVersion", "index", "signature", "publicKey"] {
            assert!(!v[k].is_null(), "falta la clave {k}");
        }
    }

    #[test]
    fn los_cierres_de_recepcion_salen_del_diario_con_su_indice() {
        // §571: la serie que `zkssl_recepPath` usa. Una cabeza firmada lleva su indice; una
        // sin firmar, no, y el cierre igual cuenta.
        let d = std::path::Path::new("target").join("diario_cierres_recep");
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).expect("dir");
        let ruta = d.join("diario.jsonl");
        let mut firmada = cabeza(7, 0x11, true);
        firmada.cabeza.recep_count = 4;
        anotar(&ruta, &firmada, &[]).expect("anotar");
        let mut sin = cabeza(0, 0x22, false);
        sin.cabeza.recep_count = 9;
        anotar(&ruta, &sin, &[]).expect("anotar");
        assert_eq!(cierres_de_recepcion(&ruta), vec![(4, Some(7)), (9, None)]);
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
        assert!(!v["recepCount"].is_null(), "y el de la era de recepcion, firmada o no (§570)");
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
                params_digest: as_digest(0),
                pmeta_root: as_digest(0),
                next_pending: 0,
                next_index: 0,
                total_supply: 0,
                recep_root: as_digest(0),
                recep_count: 0,
            },
            seq,
            epoch_digest: [0x33; 32],
            firma: None,
            emitida_unix: 1_700_000_000,
            foto: std::sync::Arc::new(zk_ssl::tests_support::new_layer().foto_pendientes()),
        }
    }

    #[test]
    fn el_ultimo_recep_count_sale_del_diario_y_las_lineas_viejas_no_lo_dan() {
        // §570: Q tras un reinicio. Una linea anterior al §570 no lo lleva y no cuenta; una
        // ilegible se salta; manda la ULTIMA que lo lleva.
        let d = std::path::Path::new("target").join("diario_recep_count");
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).expect("dir");
        let ruta = d.join("diario.jsonl");
        std::fs::write(&ruta, "{\"v\":1,\"seq\":\"0x1\",\"epochDigest\":\"0x00\",\"emittedAtUnix\":\"0x1\"}\n")
            .expect("una linea vieja");
        assert_eq!(ultimo_recep_count(&ruta), None, "un diario de antes del §570 no da Q");
        let mut l = latido_sin_firma(7);
        l.cabeza.recep_count = 3;
        anotar(&ruta, &l, &[]).expect("anotar");
        l.cabeza.recep_count = 9;
        anotar(&ruta, &l, &[]).expect("anotar");
        let mut f = std::fs::OpenOptions::new().append(true).open(&ruta).expect("abrir");
        writeln!(f, "esto no es json").expect("una linea rota");
        assert_eq!(ultimo_recep_count(&ruta), Some(9), "manda la ultima que lo lleva");
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
