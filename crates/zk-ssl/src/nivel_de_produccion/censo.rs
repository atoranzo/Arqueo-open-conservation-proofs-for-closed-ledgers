//! **§839: la traza oculta de cada AIR del cable, en una longitud con nivel atado.**
//!
//! El nivel de las familias de longitud fija lo fija el primer test de [`super`], sobre pruebas
//! ocultas reales, y con lote lineal la funcion de `winter-air` solo depende de las opciones y de
//! la longitud: basta una prueba por longitud T. Pero nada exigia que cada T del cable tuviera la
//! suya. La de la subida de congelados (T = 256) no la tenia: su fila de `SECURITY.md` §3.11 era
//! derivada, no medida. Este censo lee de los fuentes que AIR verifica cada camino, toma su T del
//! compilador, y exige que este en [`LONGITUDES_MEDIDAS`]:
//!
//! - **la capa:** cada llamada `crate::comprobar_forma(...)` de `crates/zk-ssl/src`, fuera de este
//!   modulo, con su longitud escrita como `stark_experiment::circuit_<x>::TRACE_LENGTH`, y la AIR
//!   del primer `verify::<...>` que la sigue. Otra forma de escribirla, o un circuito que
//!   [`longitud_de_la_capa`] no conoce, es ROJO: un probador nuevo entra en la tabla, y su T, en
//!   el primer test;
//! - **la pareja umbral:** `verify_threshold_pair` compara la longitud con `2 * TRACE_LENGTH` de
//!   su circuito;
//! - **el kit:** cada `impl Air for` de `crates/zk-ssl-air/src`, por su fichero, con la `TRAZA` de
//!   ese fichero (⚠️ §723: y cada AIR con version, `XDe<V>`, por su familia `X`). La edad
//!   (`EdadAir`, en `lib.rs`), cuya longitud depende de `m`, queda fuera:
//!   tiene su test en `zk-ssl-air` (`tests_nivel`), con el piso de cada `m`.
//!
//! Las AIR vistas, con la edad, son las del cable: las filas `true` del censo D-L
//! (`FILAS` en `instrumento_revela.rs`), ni mas ni menos. Una AIR que entra en el cable por otro
//! camino cae aqui. Y al reves: las familias del primer test, leidas de su fuente como las lee el
//! ATADO F de `tools/check_publicadas.py`, dan exactamente [`LONGITUDES_MEDIDAS`]. El primer test
//! lo exige tambien sobre sus pruebas, pero solo en release; esto corre tambien en depuracion.
//!
//! La prueba de vida: al menos las quince llamadas de la capa (`tests_sal` en `lib.rs` cuenta los
//! quince `verify`), la pareja umbral y las cuatro AIR de longitud fija del kit. Lo que NO ve: una
//! llamada escrita sin el prefijo `crate::`, y una AIR del cable que no este en `FILAS`. Las agujas
//! se arman por partes para que este fuente no las contenga.

use super::LONGITUDES_MEDIDAS;
use std::path::{Path, PathBuf};

/// La T de un circuito de `stark-experiment` que la capa verifica, por el nombre de su modulo.
/// La resuelve el compilador: un `TRACE_LENGTH` cambiado llega aqui tal cual. Un nombre que no
/// esta es un probador nuevo, y el censo cae hasta que entra en esta tabla.
fn longitud_de_la_capa(circuito: &str) -> Option<usize> {
    use stark_experiment as s;
    Some(match circuito {
        "circuit_audit" => s::circuit_audit::TRACE_LENGTH,
        "circuit_burn" => s::circuit_burn::TRACE_LENGTH,
        "circuit_claim" => s::circuit_claim::TRACE_LENGTH,
        "circuit_claim_v2" => s::circuit_claim_v2::TRACE_LENGTH,
        "circuit_credit_climb" => s::circuit_credit_climb::TRACE_LENGTH,
        "circuit_frozen_climb" => s::circuit_frozen_climb::TRACE_LENGTH,
        "circuit_mint_climb" => s::circuit_mint_climb::TRACE_LENGTH,
        "circuit_mint_pending_climb" => s::circuit_mint_pending_climb::TRACE_LENGTH,
        "circuit_recovery_climb" => s::circuit_recovery_climb::TRACE_LENGTH,
        "circuit_refund" => s::circuit_refund::TRACE_LENGTH,
        "circuit_refund_v2" => s::circuit_refund_v2::TRACE_LENGTH,
        "circuit_send" => s::circuit_send::TRACE_LENGTH,
        "circuit_send_v2" => s::circuit_send_v2::TRACE_LENGTH,
        _ => return None,
    })
}

/// La `TRAZA` de una AIR de longitud fija del kit, por su fichero en `crates/zk-ssl-air/src`.
fn longitud_del_kit(fichero: &str) -> Option<usize> {
    Some(match fichero {
        "banda.rs" => zk_ssl_air::banda::TRAZA,
        "cobro_pendiente.rs" => zk_ssl_air::cobro_pendiente::TRAZA,
        "pago_en_curso.rs" => zk_ssl_air::pago_en_curso::TRAZA,
        "prenda.rs" => zk_ssl_air::prenda::TRAZA,
        _ => return None,
    })
}

/// Las lineas de CODIGO de un fuente, sin las de comentario, unidas con saltos de linea.
fn codigo(fuente: &str) -> String {
    fuente.lines().filter(|l| !l.trim_start().starts_with("//")).collect::<Vec<_>>().join("\n")
}

/// Los argumentos de la llamada que empieza en `texto` (justo detras de su `(`), hasta su `)`,
/// separados por las comas de primer nivel. None si la llamada no se cierra.
fn argumentos(texto: &str) -> Option<Vec<String>> {
    let (mut hondo, mut actual, mut args) = (0usize, String::new(), Vec::new());
    for c in texto.chars() {
        match c {
            '(' | '[' | '{' => hondo += 1,
            ')' | ']' | '}' if hondo == 0 => {
                if !actual.trim().is_empty() {
                    args.push(actual.trim().to_string());
                }
                return Some(args);
            }
            ')' | ']' | '}' => hondo -= 1,
            ',' if hondo == 0 => {
                args.push(actual.trim().to_string());
                actual.clear();
                continue;
            }
            _ => {}
        }
        actual.push(c);
    }
    None
}

/// Los modulos `circuit_<x>` nombrados como `circuit_<x>::TRACE_LENGTH` en `texto`, en orden.
fn familias_nombradas(texto: &str) -> Vec<String> {
    let aguja = concat!("::TRACE", "_LENGTH");
    let mut out = Vec::new();
    for (i, _) in texto.match_indices(aguja) {
        let antes = &texto[..i];
        let ini = antes
            .rfind(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .map_or(0, |j| j + 1);
        let nombre = &antes[ini..];
        if nombre.starts_with("circuit_") && !out.iter().any(|n| n == nombre) {
            out.push(nombre.to_string());
        }
    }
    out
}

/// Los `.rs` de `dir` y sus subdirectorios, en orden.
fn fuentes(dir: &Path) -> Vec<PathBuf> {
    let mut hijos: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .map(|e| e.expect("entrada").path())
        .collect();
    hijos.sort();
    let mut out = Vec::new();
    for h in hijos {
        if h.is_dir() {
            out.extend(fuentes(&h));
        } else if h.extension().is_some_and(|x| x == "rs") {
            out.push(h);
        }
    }
    out
}

fn leer(ruta: &Path) -> String {
    std::fs::read_to_string(ruta).unwrap_or_else(|e| panic!("{}: {e}", ruta.display()))
}

/// **La T de cada AIR del cable esta entre las que mide el primer test.** Ver la cabecera.
#[test]
fn cada_longitud_del_cable_tiene_su_nivel_medido() {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR"));
    let crates = raiz.join("..");
    let medidas = LONGITUDES_MEDIDAS;
    let mut mal = Vec::new();
    // (camino, AIR, T) de cada verificador visto.
    let mut vistas: Vec<(String, String, usize)> = Vec::new();

    // La capa: las llamadas a la guarda de forma, fuera del modulo del nivel.
    let llamada = concat!("crate::comprobar", "_forma(");
    let propio = raiz.join("src").join("nivel_de_produccion");
    let mut n_capa = 0;
    for f in fuentes(&raiz.join("src")) {
        if f.starts_with(&propio) || f.file_name().is_some_and(|n| n == "nivel_de_produccion.rs") {
            continue;
        }
        let nombre = f.strip_prefix(&raiz).unwrap_or(&f).display().to_string();
        let src = codigo(&leer(&f));
        for (i, _) in src.match_indices(llamada) {
            n_capa += 1;
            let args = argumentos(&src[i + llamada.len()..]);
            let longitud =
                args.as_ref().and_then(|a| a.get(4)).map(|a| a.replace(char::is_whitespace, ""));
            let circuito = longitud.as_deref().and_then(|l| {
                l.strip_prefix("stark_experiment::")?.strip_suffix(concat!("::TRACE", "_LENGTH"))
            });
            let air: Option<String> = src[i..].find(concat!("verify", "::<")).map(|j| {
                src[i + j + 9..].chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect()
            });
            match circuito.and_then(|c| longitud_de_la_capa(c).map(|t| (c, t))) {
                Some((c, t)) => vistas.push((
                    format!("{nombre}, {c}"),
                    air.unwrap_or_else(|| format!("{c} sin verify")),
                    t,
                )),
                None => mal.push(format!(
                    "{nombre}: una guarda de forma con longitud {longitud:?}, que el censo no \
                     resuelve: un probador nuevo entra en `longitud_de_la_capa` y su T en el \
                     primer test"
                )),
            }
        }
    }

    // La pareja umbral.
    let umbral = crates.join("stark-experiment/src/circuit_threshold_single_nullifier.rs");
    let src = codigo(&leer(&umbral));
    let cuerpo = src
        .find(concat!("pub fn verify_threshold", "_pair("))
        .map(|ini| &src[ini..ini + src[ini..].find("\n}").unwrap_or(src.len() - ini)]);
    let n_umbral = match cuerpo {
        Some(c) if c.contains(concat!("i.length() == 2 * TRACE", "_LENGTH")) => {
            let t = stark_experiment::circuit_threshold_single_nullifier::TRACE_LENGTH;
            vistas.push(("la pareja umbral".into(), "NullifierThresholdAir".into(), t));
            1
        }
        _ => {
            mal.push("verify_threshold_pair: no se ve que compare la longitud con 2 * su T".into());
            0
        }
    };

    // El kit: cada AIR, por su fichero.
    let mut n_kit = 0;
    for f in fuentes(&crates.join("zk-ssl-air/src")) {
        let src = codigo(&leer(&f));
        let fichero = f.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
        let mut implementadas: Vec<String> = src
            .match_indices(concat!("impl Air", " for "))
            .map(|(i, _)| src[i + 13..].chars().take_while(|c| c.is_alphanumeric()).collect())
            .collect();
        // §723: la banda y el cobro pendiente implementan la AIR con version, `XDe<V>`, y cuentan
        // por su familia `X`, con la regla de `air_implementadas` de `instrumento_revela.rs`: la
        // v1 y la v2 tienen la misma traza.
        let con_version = concat!("impl<const V: u8> Air", " for ");
        for (i, _) in src.match_indices(con_version) {
            let n: String =
                src[i + con_version.len()..].chars().take_while(|c| c.is_alphanumeric()).collect();
            implementadas.push(n.strip_suffix("De").map(str::to_string).unwrap_or(n));
        }
        for air in implementadas {
            if fichero == "lib.rs" && air == "EdadAir" {
                continue; // su longitud depende de `m`: `tests_nivel`
            }
            match longitud_del_kit(&fichero) {
                Some(t) => {
                    n_kit += 1;
                    vistas.push((format!("el kit, {fichero}"), air, t));
                }
                None => mal.push(format!(
                    "zk-ssl-air/src/{fichero}: {air} sin T en el censo (`longitud_del_kit`)"
                )),
            }
        }
    }

    // Cada T vista, entre las medidas.
    for (camino, air, t) in &vistas {
        if !medidas.contains(t) {
            mal.push(format!(
                "{camino}: {air} con T = {t}, una traza oculta de {} filas sin nivel medido \
                 (medidas: {medidas:?})",
                2 * t
            ));
        }
    }

    // Las AIR vistas, con la edad, son las del cable de D-L.
    let revela = leer(&raiz.join("src/instrumento_revela.rs"));
    let filas = revela.find(concat!("const FI", "LAS: &[(&str, &str, bool)] = &[")).map(|ini| {
        let fin = revela[ini..].find("\n];").unwrap_or(revela.len() - ini);
        &revela[ini..ini + fin]
    });
    let mut del_cable: Vec<String> = filas
        .unwrap_or("")
        .lines()
        .filter(|l| l.trim_end().ends_with(", true),"))
        .filter_map(|l| l.split('"').nth(1).map(str::to_string))
        .collect();
    del_cable.sort();
    let mut airs: Vec<String> = vistas.iter().map(|v| v.1.clone()).collect();
    airs.push("EdadAir".into());
    airs.sort();
    airs.dedup();
    if airs != del_cable {
        mal.push(format!(
            "las AIR vistas, con la edad, {airs:?}, no son las del cable de D-L, {del_cable:?}"
        ));
    }

    // Y al reves: las familias del primer test dan las medidas, ni mas ni menos.
    let mut del_test: Vec<usize> = Vec::new();
    for c in familias_nombradas(&codigo(&leer(&raiz.join("src/nivel_de_produccion.rs")))) {
        match longitud_de_la_capa(&c) {
            Some(t) => del_test.push(t),
            None => mal.push(format!("el primer test nombra {c}, que el censo no resuelve")),
        }
    }
    del_test.sort_unstable();
    del_test.dedup();
    if del_test != medidas {
        mal.push(format!(
            "las familias del primer test dan T = {del_test:?}, y las fijadas son {medidas:?}"
        ));
    }

    let mut por_t: Vec<(usize, usize)> = medidas.iter().map(|m| (*m, 0)).collect();
    for (_, _, t) in &vistas {
        if let Some(x) = por_t.iter_mut().find(|x| x.0 == *t) {
            x.1 += 1;
        }
    }
    println!(
        "[§839] censo: {n_capa} guardas de forma en la capa, {n_umbral} pareja umbral y {n_kit} \
         AIR del kit; {} AIR con la edad, las {} del cable de D-L; por T (T, verificadores): \
         {por_t:?}; las del primer test: {del_test:?}",
        airs.len(),
        del_cable.len()
    );
    if n_capa < 15 || n_umbral != 1 || n_kit < 4 {
        mal.push(format!(
            "prueba de vida: {n_capa} guardas de la capa (al menos 15), {n_umbral} pareja umbral \
             (1) y {n_kit} AIR del kit (al menos 4): el censo no ve el arbol"
        ));
    }
    assert!(mal.is_empty(), "longitudes del cable sin nivel medido:\n{}", mal.join("\n"));
}
