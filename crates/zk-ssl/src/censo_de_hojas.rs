//! **§840: el censo de los `native_leaf` y su guarda** (la 51 de `BACKLOG.md`).
//!
//! La 51 decia que habia tres `native_leaf` con dominios de identidad de distinta anchura, y que
//! no estaba medido si las estrechas -identidad de un `BaseElement`, 64 bits- estaban en algun
//! camino de produccion. Las de produccion son anchas: identidad de un `Digest`, 256 bits. Este
//! modulo lo mide y lo ata en dos tests. Solo lee fuentes: corre tambien en depuracion.
//!
//! - **El censo:** cada `fn native_leaf…` de `crates/` (fuera de `target`, en `src/`, `tests/`,
//!   `examples/` y `benches/`), con el tipo de su primer parametro, la identidad, contra la lista
//!   escrita [`CENSO`]. Una funcion nueva con ese nombre, una que se va o una que cambia de
//!   anchura, cae, y la lista cambia con un asiento.
//! - **La guarda:** las cuatro raices de produccion de `tools/check_curvas.py` (R1) y los crates
//!   de la casa de su clausura, leidos de los `[dependencies]` de sus `Cargo.toml` -sin las de
//!   desarrollo-, son [`PRODUCCION`]. En `stark-experiment`, que esta en esa clausura, los modulos
//!   con una estrecha y los que nombran uno de ellos -como identificador entero, con su ruta o sin
//!   ella-, hasta el punto fijo, son [`APARTADOS`]. El codigo de un modulo `m` es su `m.rs` y todos
//!   los `.rs` bajo `m/`: `mod.rs` y los submodulos en directorio. Y:
//!   - ningun fuente de `src/` de los demas crates de [`PRODUCCION`] nombra un modulo apartado,
//!     ni tiene una estrecha;
//!   - los modulos de `stark-experiment` que esos fuentes nombran, y los que estos nombran con
//!     `crate::` o `super::`, son los del cable y sus ayudas; ninguno es un apartado -ni nombra
//!     uno, porque eso lo apartaria-, y la raiz de `stark-experiment` solo los nombra en su `mod`.
//!
//! Quedan fuera los ficheros SOLO de tests: los que la raiz de su crate declara con
//! `#[cfg(test)] mod x;`, y lo que cuelga de ellos. El instrumento de D-B
//! (`instrumento_revela.rs`) prueba las experimentales y nombra una a proposito. Los modulos de
//! test en linea de un fichero de produccion SI se leen: dan ROJO aunque no se enlacen.
//!
//! Un glob (`use crate::*;`, `use super::*;`) trae nombres sin su ruta: quien usa asi un modulo
//! apartado escribe su nombre, y ese nombre lo aparta a el; y un item que la raiz reexporte de un
//! apartado ya es ROJO por la regla de la raiz. Un renombre del crate da ROJO, porque la guarda
//! no lo sigue: un `package` en una dependencia de la casa, en la linea de su `path` o en su tabla
//! `[dependencies.x]`; y un alias escrito `stark_experiment as` -tambien con `extern crate`- o
//! `stark_experiment::{self as …}` en un fuente de produccion.
//!
//! Lo que NO ve: una estrecha con otro nombre; una funcion escrita con otro espaciado que
//! `fn native_leaf`; un nombre en un comentario de bloque o tras codigo en la misma linea (eso
//! da ROJO, no VERDE); un `#[path = ...]`, un `include!` o una macro que fabrique el nombre; un
//! fichero de `stark-experiment/src` fuera de `m.rs` y de `m/` de un `mod m;` de su raiz; una
//! dependencia de la casa por `workspace = true`; y los crates del fork (`winter-*`), que entran
//! por el `[patch]` del workspace y no por una ruta en un `Cargo.toml`: no dependen de
//! `stark-experiment`, y no pueden nombrarlo.
//! Las agujas se arman por partes para que este fuente no las contenga.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// Cada `fn native_leaf…` de `crates/`: (fichero bajo `crates/`, nombre, tipo de la identidad).
/// Ancha, `Digest`: la de produccion, en `zk-ssl-hash`, que `stark-experiment` reexporta
/// (`native.rs`). Estrechas, `BaseElement`: dos en `stark-experiment`. Las de los comparativos
/// llevan el campo de su sistema de pruebas.
const CENSO: &[(&str, &str, &str)] = &[
    ("halo2-experiment/src/circuit_double_entry.rs", "native_leaf", "Fp"),
    ("halo2-experiment/src/compliance_circuit.rs", "native_leaf", "Fp"),
    ("halo2-experiment/src/compliance_real_proof.rs", "native_leaf", "Fp"),
    ("halo2-experiment/src/iso_bridge.rs", "native_leaf", "Fp"),
    ("halo2-experiment/src/settlement_prover_impl.rs", "native_leaf", "Fp"),
    ("plonk-experiment/src/poseidon_hash.rs", "native_leaf", "BlsScalar"),
    ("stark-experiment/src/compliance_circuit.rs", "native_leaf", "BaseElement"),
    ("stark-experiment/src/double_entry.rs", "native_leaf", "BaseElement"),
    ("zk-ssl-hash/src/lib.rs", "native_leaf", "Digest"),
    ("zk-ssl-hash/src/lib.rs", "native_leaf_salted", "Digest"),
];

/// Las raices de produccion: los cuatro de `tools/check_curvas.py` (R1).
const RAICES: &[&str] = &["zk-ssl-cli", "zk-ssl-node", "zk-ssl-sdk", "zk-ssl-verify"];

/// Las raices y los crates de la casa de su clausura, por los `[dependencies]` de los
/// `Cargo.toml`. Lo mismo que da `cargo tree -e normal` de las cuatro, sin el fork.
const PRODUCCION: &[&str] = &[
    "settlement-prover",
    "stark-experiment",
    "zk-ssl",
    "zk-ssl-air",
    "zk-ssl-cli",
    "zk-ssl-guardian",
    "zk-ssl-hash",
    "zk-ssl-medio",
    "zk-ssl-node",
    "zk-ssl-sdk",
    "zk-ssl-verify",
    "zk-ssl-wire",
];

/// Los modulos de `stark-experiment` con una estrecha, y los que los nombran, hasta el punto
/// fijo: los tres que usan la de `compliance_circuit`, y el registro de nulificadores, que nombra
/// ese modulo en sus tests.
const APARTADOS: &[&str] = &[
    "compliance_circuit",
    "compliance_real_proof",
    "double_entry",
    "iso_bridge",
    "persistent_nullifier_registry",
    "settlement_prover_impl",
];

/// Las lineas de CODIGO de un fuente, sin las de comentario, unidas con saltos de linea.
fn codigo(fuente: &str) -> String {
    fuente.lines().filter(|l| !l.trim_start().starts_with("//")).collect::<Vec<_>>().join("\n")
}

fn leer(ruta: &Path) -> String {
    std::fs::read_to_string(ruta).unwrap_or_else(|e| panic!("{}: {e}", ruta.display()))
}

fn es_ident(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Los `.rs` de `dir` y sus subdirectorios, en orden, sin `target` y sin los de `fuera`.
fn fuentes(dir: &Path, fuera: &[PathBuf]) -> Vec<PathBuf> {
    let mut hijos: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .map(|e| e.expect("entrada").path())
        .collect();
    hijos.sort();
    let mut out = Vec::new();
    for h in hijos {
        if fuera.contains(&h) || h.file_name().is_some_and(|n| n == "target") {
            continue;
        }
        if h.is_dir() {
            out.extend(fuentes(&h, fuera));
        } else if h.extension().is_some_and(|x| x == "rs") {
            out.push(h);
        }
    }
    out
}

/// Si `palabra` sale en `texto` como identificador entero.
fn nombra(texto: &str, palabra: &str) -> bool {
    texto.match_indices(palabra).any(|(i, _)| {
        !texto[..i].chars().next_back().is_some_and(es_ident)
            && !texto[i + palabra.len()..].chars().next().is_some_and(es_ident)
    })
}

/// Lo que sigue a cada `prefijo` de `texto`, tal cual: `prefijo::a::b` da `a::b…` hasta el fin
/// del identificador, y `prefijo::{a, b::c}` da `a` y `b::c`, un elemento por cada uno del grupo.
fn tras_prefijo(texto: &str, prefijo: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (i, _) in texto.match_indices(prefijo) {
        if texto[..i].chars().next_back().is_some_and(es_ident) {
            continue;
        }
        let resto = texto[i + prefijo.len()..].trim_start();
        if let Some(grupo) = resto.strip_prefix('{') {
            let (mut hondo, mut actual) = (0usize, String::new());
            for c in grupo.chars() {
                match c {
                    '{' => hondo += 1,
                    '}' if hondo == 0 => break,
                    '}' => hondo -= 1,
                    ',' if hondo == 0 => {
                        out.push(std::mem::take(&mut actual));
                        continue;
                    }
                    _ => {}
                }
                actual.push(c);
            }
            out.push(actual);
        } else {
            out.push(resto.chars().take_while(|c| es_ident(*c)).collect());
        }
    }
    out
}

/// Los nombres que siguen a `prefijo` en `texto`: `prefijo::a` da `a`, y `prefijo::{a, b::c}` da
/// `a` y `b`.
fn nombrados(texto: &str, prefijo: &str) -> BTreeSet<String> {
    tras_prefijo(texto, prefijo)
        .iter()
        .map(|it| it.trim_start().chars().take_while(|c| es_ident(*c)).collect::<String>())
        .filter(|n| !n.is_empty())
        .collect()
}

/// Si `src` renombra el crate `stark_experiment`: `stark_experiment as x`, con o sin `extern
/// crate`, o `stark_experiment::{self as x}`.
fn renombra_stark(src: &str) -> bool {
    src.contains(concat!("stark_experiment", " as "))
        || tras_prefijo(src, "stark_experiment::").iter().any(|it| {
            let t: Vec<&str> = it.split_whitespace().collect();
            t.len() >= 2 && t[0] == "self" && t[1] == "as"
        })
}

/// Cada `fn native_leaf…` de `dir`: (fichero relativo a `base`, nombre, tipo de la identidad), y
/// cuantos ficheros leyo.
fn censar(base: &Path, dir: &Path) -> (Vec<(String, String, String)>, usize) {
    let aguja = concat!("fn native", "_leaf");
    let ficheros = fuentes(dir, &[]);
    let mut out = Vec::new();
    for f in &ficheros {
        let src = codigo(&leer(f));
        for (i, _) in src.match_indices(aguja) {
            if src[..i].chars().next_back().is_some_and(es_ident) {
                continue;
            }
            let tras = &src[i + 3..];
            let nombre: String = tras.chars().take_while(|c| es_ident(*c)).collect();
            let tipo = tras
                .find('(')
                .map(|j| {
                    let p = &tras[j + 1..];
                    let fin = p.find([',', ')']).unwrap_or(p.len());
                    let param = &p[..fin];
                    param.split_once(':').map_or("?".to_string(), |(_, t)| {
                        t.chars().filter(|c| !c.is_whitespace()).collect()
                    })
                })
                .unwrap_or_else(|| "?".into());
            let rel = f.strip_prefix(base).unwrap_or(f).display().to_string();
            out.push((rel, nombre, tipo));
        }
    }
    out.sort();
    (out, ficheros.len())
}

/// Los crates de la casa que `krate` enlaza: las dependencias con `path = "../x"` de sus
/// secciones `[dependencies]`, `[dependencies.x]`, `[target.….dependencies]` y
/// `[target.….dependencies.x]`. Las de desarrollo y
/// de compilacion no. Un `package` en una de ellas va a `mal`, en la misma linea que su `path` o
/// en la tabla `[dependencies.x]` que lo lleva: el crate se nombraria de otro modo.
fn de_la_casa(crates: &Path, krate: &str, mal: &mut Vec<String>) -> Vec<String> {
    let toml = leer(&crates.join(krate).join("Cargo.toml"));
    let (mut seccion, mut out) = (String::new(), Vec::new());
    // La tabla `[dependencies.x]` en curso: si lleva un `path` de la casa, y si un `package`.
    let (mut tabla_casa, mut tabla_package) = (false, false);
    let cerrar = |seccion: &str, casa: bool, package: bool, mal: &mut Vec<String>| {
        if casa && package {
            mal.push(format!("{krate}/Cargo.toml: `[{seccion}]` renombra un crate de la casa"));
        }
    };
    for linea in toml.lines() {
        let l = linea.split('#').next().unwrap_or("").trim();
        if l.starts_with('[') {
            cerrar(&seccion, tabla_casa, tabla_package, mal);
            (tabla_casa, tabla_package) = (false, false);
            seccion = l.trim_matches(|c| c == '[' || c == ']').to_string();
            continue;
        }
        let normal = seccion == "dependencies"
            || seccion.starts_with("dependencies.")
            || (seccion.starts_with("target.") && seccion.contains(".dependencies"));
        if !normal {
            continue;
        }
        let tabla = seccion != "dependencies" && !seccion.ends_with(".dependencies");
        if tabla && l.starts_with("package") {
            tabla_package = true;
        }
        let Some(j) = l.find("path") else { continue };
        let ruta = l[j..].split('"').nth(1).unwrap_or("");
        if let Some(dep) = ruta.strip_prefix("../") {
            out.push(dep.trim_end_matches('/').to_string());
            tabla_casa |= tabla;
            if l.contains("package") {
                mal.push(format!("{krate}/Cargo.toml: `{l}` renombra un crate de la casa"));
            }
        }
    }
    cerrar(&seccion, tabla_casa, tabla_package, mal);
    out
}

/// Los ficheros y directorios SOLO de tests de `src`: los `mod x;` que `lib.rs` o `main.rs`
/// declaran justo tras una linea `#[cfg(test)]`.
fn solo_de_tests(src: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for raiz in ["lib.rs", "main.rs"] {
        let Ok(texto) = std::fs::read_to_string(src.join(raiz)) else { continue };
        let lineas: Vec<&str> = texto.lines().map(str::trim).collect();
        for par in lineas.windows(2) {
            let decl = par[1].strip_prefix("pub ").unwrap_or(par[1]);
            if par[0] == "#[cfg(test)]" {
                if let Some(m) = decl.strip_prefix("mod ").and_then(|d| d.strip_suffix(';')) {
                    out.push(src.join(format!("{m}.rs")));
                    out.push(src.join(m));
                }
            }
        }
    }
    out
}

/// **Cada `native_leaf` de `crates/` esta en el censo, con la anchura de su identidad.**
#[test]
fn cada_native_leaf_esta_en_el_censo() {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let base = crates.canonicalize().expect("crates/");
    let (vistas, n_ficheros) = censar(&base, &base);
    let mut escritas: Vec<(String, String, String)> =
        CENSO.iter().map(|(f, n, t)| (f.to_string(), n.to_string(), t.to_string())).collect();
    escritas.sort();
    let mut mal = Vec::new();
    for v in &vistas {
        if !escritas.contains(v) {
            mal.push(format!("{} en {}, identidad {}: no esta en el censo", v.1, v.0, v.2));
        }
    }
    for e in &escritas {
        if !vistas.contains(e) {
            mal.push(format!("{} en {}, identidad {}: el censo la tiene y no esta", e.1, e.0, e.2));
        }
    }
    let cuenta = |t: &str| vistas.iter().filter(|v| v.2 == t).count();
    println!(
        "[§840] censo: {} funciones native_leaf… en {n_ficheros} ficheros de crates/: {} anchas (Digest), \
         {} estrechas (BaseElement), {} de halo2 (Fp) y {} de plonk (BlsScalar)",
        vistas.len(),
        cuenta("Digest"),
        cuenta("BaseElement"),
        cuenta("Fp"),
        cuenta("BlsScalar")
    );
    if n_ficheros < 250 {
        mal.push(format!("prueba de vida: {n_ficheros} ficheros (al menos 250): no ve el arbol"));
    }
    assert!(mal.is_empty(), "el censo de los native_leaf no cuadra:\n{}", mal.join("\n"));
}

/// **Ninguna estrecha llega a produccion.** Ver la cabecera.
#[test]
fn ninguna_estrecha_llega_a_produccion() {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let base = crates.canonicalize().expect("crates/");
    let mut mal = Vec::new();

    // La clausura de las raices, por sus `Cargo.toml`.
    let mut clausura: BTreeSet<String> = RAICES.iter().map(|r| r.to_string()).collect();
    let mut pendientes: Vec<String> = clausura.iter().cloned().collect();
    while let Some(k) = pendientes.pop() {
        for d in de_la_casa(&base, &k, &mut mal) {
            if clausura.insert(d.clone()) {
                pendientes.push(d);
            }
        }
    }
    let escrita: BTreeSet<String> = PRODUCCION.iter().map(|p| p.to_string()).collect();
    if clausura != escrita {
        mal.push(format!("la clausura de las raices es {clausura:?}, y la escrita {escrita:?}"));
    }

    // `stark-experiment`: sus modulos, y los que tienen una estrecha, del censo vivo.
    let stark = base.join("stark-experiment/src");
    let raiz_stark = codigo(&leer(&stark.join("lib.rs")));
    let modulos: BTreeSet<String> = raiz_stark
        .lines()
        .map(str::trim)
        .filter_map(|l| l.strip_prefix("pub ").unwrap_or(l).strip_prefix("mod "))
        .filter_map(|l| l.strip_suffix(';'))
        .map(str::to_string)
        .collect();
    // El codigo de cada modulo: `m.rs`, y todos los `.rs` bajo `m/`, `mod.rs` y submodulos.
    let codigo_de: BTreeMap<String, String> = modulos
        .iter()
        .map(|m| {
            let mut fs: Vec<PathBuf> =
                [stark.join(format!("{m}.rs"))].into_iter().filter(|f| f.is_file()).collect();
            if stark.join(m).is_dir() {
                fs.extend(fuentes(&stark.join(m), &[]));
            }
            assert!(!fs.is_empty(), "stark-experiment/src: `mod {m}` sin fuente");
            (m.clone(), fs.iter().map(|f| codigo(&leer(f))).collect::<Vec<_>>().join("\n"))
        })
        .collect();
    let n_stark: usize = modulos
        .iter()
        .map(|m| {
            usize::from(stark.join(format!("{m}.rs")).is_file())
                + if stark.join(m).is_dir() { fuentes(&stark.join(m), &[]).len() } else { 0 }
        })
        .sum();
    let nombres_de = |m: &str| {
        let src = &codigo_de[m];
        let mut n = nombrados(src, "crate::");
        n.extend(nombrados(src, "super::"));
        n.retain(|x| modulos.contains(x));
        n
    };
    let (censo_stark, _) = censar(&base, &stark);
    let mut apartados: BTreeSet<String> = BTreeSet::new();
    for (f, _, tipo) in &censo_stark {
        let m = f.trim_start_matches("stark-experiment/src/").trim_end_matches(".rs");
        if tipo != "Digest" {
            apartados.insert(m.split('/').next().unwrap_or(m).to_string());
        }
    }
    let estrechas = apartados.len();
    loop {
        let nuevos: Vec<String> = modulos
            .iter()
            .filter(|m| !apartados.contains(*m))
            .filter(|m| apartados.iter().any(|a| nombra(&codigo_de[*m], a)))
            .cloned()
            .collect();
        if nuevos.is_empty() {
            break;
        }
        apartados.extend(nuevos);
    }
    let escritos: BTreeSet<String> = APARTADOS.iter().map(|a| a.to_string()).collect();
    if apartados != escritos {
        mal.push(format!("los apartados son {apartados:?}, y los escritos {escritos:?}"));
    }

    // Los demas crates de produccion: ni un apartado, ni una estrecha, ni un alias.
    let mut semilla: BTreeSet<String> = BTreeSet::new();
    let mut n_prod = 0;
    for k in clausura.iter().filter(|k| *k != "stark-experiment") {
        let src_dir = base.join(k).join("src");
        let (censo_k, _) = censar(&base, &src_dir);
        for (f, n, t) in censo_k.iter().filter(|c| c.2 != "Digest") {
            mal.push(format!("{f}: {n} con identidad {t}, una estrecha en produccion"));
        }
        for f in fuentes(&src_dir, &solo_de_tests(&src_dir)) {
            n_prod += 1;
            let rel = f.strip_prefix(&base).unwrap_or(&f).display().to_string();
            let src = codigo(&leer(&f));
            for a in &apartados {
                if nombra(&src, a) {
                    mal.push(format!("{rel}: nombra `{a}`, un modulo apartado de stark-experiment"));
                }
            }
            if renombra_stark(&src) {
                mal.push(format!("{rel}: un alias de stark_experiment, que la guarda no sigue"));
            }
            semilla.extend(nombrados(&src, "stark_experiment::"));
        }
    }
    semilla.retain(|m| modulos.contains(m));

    // Lo que la produccion enlaza de `stark-experiment`, hasta el punto fijo.
    let mut enlazados = semilla.clone();
    let mut pendientes: Vec<String> = semilla.iter().cloned().collect();
    pendientes.extend(nombrados(&raiz_stark, "crate::").into_iter().filter(|m| modulos.contains(m)));
    while let Some(m) = pendientes.pop() {
        enlazados.insert(m.clone());
        for x in nombres_de(&m) {
            if !enlazados.contains(&x) {
                pendientes.push(x);
            }
        }
    }
    // Un modulo que nombra un apartado ya es apartado: basta con que ninguno se enlace.
    for m in enlazados.iter().filter(|m| apartados.contains(*m)) {
        mal.push(format!("stark-experiment: `{m}`, apartado, lo enlaza la produccion"));
    }
    for l in raiz_stark.lines() {
        let t = l.trim();
        let t = t.strip_prefix("pub ").unwrap_or(t);
        if t.strip_prefix("mod ").is_some_and(|d| d.ends_with(';')) {
            continue;
        }
        for a in apartados.iter().filter(|a| nombra(l, a)) {
            mal.push(format!("stark-experiment/src/lib.rs: `{}` nombra `{a}` fuera de su mod", t));
        }
    }

    println!(
        "[§840] guarda: {} crates de produccion; {n_prod} fuentes de src/ fuera de stark-experiment, \
         sin los de tests; {} modulos de stark-experiment en {n_stark} fuentes, {} con una estrecha y {} apartados; la \
         produccion nombra {} y enlaza {}: {:?}",
        clausura.len(),
        modulos.len(),
        estrechas,
        apartados.len(),
        semilla.len(),
        enlazados.len(),
        enlazados
    );
    if n_prod < 80 || semilla.len() < 20 || enlazados.len() < 25 || estrechas < 2 {
        mal.push(format!(
            "prueba de vida: {n_prod} fuentes de produccion (al menos 80), {} modulos nombrados \
             (al menos 20), {} enlazados (al menos 25) y {estrechas} modulos con estrecha (al \
             menos 2): la guarda no ve el arbol",
            semilla.len(),
            enlazados.len()
        ));
    }
    assert!(mal.is_empty(), "una estrecha al alcance de la produccion:\n{}", mal.join("\n"));
}
