//! Las notas del medio contra una implementación ajena (RFC-0013 E2b, §632).
//!
//! `tests/vectores/notas/` lleva 33 notas y lo que dice de cada una
//! `filippo.io/torchwood` v0.10.0 —`note.Open` con `NewLogVerifier`, sobre el
//! `crypto/mldsa` de Go 1.27—, escrito por `torchwood/main.go` en
//! `veredictos-torchwood.txt`. Las `pos-*` y las `neg-*` las firmó torchwood;
//! las `rust-*`, este crate. El canon no corre Go: lee lo que Go dejó escrito
//! y comprueba aquí, en cada pasada:
//!
//! - que las claves salen iguales de la misma semilla en las dos
//!   implementaciones de ML-DSA-44, hasta la `vkey`;
//! - que las notas de torchwood verifican, y que la línea del publicador
//!   **se reproduce byte a byte** firmando aquí en determinista con su
//!   misma marca de tiempo: el `cosigned_message`, el `key_id` y la base64
//!   son los mismos;
//! - que la raíz de cada nota, que torchwood calculó con el árbol de
//!   `golang.org/x/mod/sumdb/tlog`, es la de [`ArbolDelMedio`];
//! - que la cofirma del testigo, también de torchwood, verifica con
//!   [`ClaveDeNota::verificar_cofirma`], y que [`Cofirmante`] la reproduce
//!   byte a byte (§633);
//! - y que el `MANIFIESTO.txt` dice lo que dicen los dos verificadores, con
//!   las cuatro negativas que torchwood acepta y el medio no, cada una con
//!   su motivo, y ninguna al revés.

use std::collections::BTreeMap;
use std::path::PathBuf;

use zk_ssl_medio::hash::sha256;
use zk_ssl_medio::medio::ArbolDelMedio;
use zk_ssl_medio::nota::{
    origen_del_medio, verificar_nota, ClaveDeNota, Cofirmante, ErrorDeNota, NotaVerificada,
    Publicador,
};

/// La huella de la clave del operador de `spec/vectors/ancla/`.
const HUELLA: &str = "8c40b55b1d40cf7f71f9ea99c770f701db95da5876e6ce889f86af12b0def53a";
const TESTIGO: &str = "testigo.invalid/ajeno";

fn dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/vectores/notas")
}

fn leer(nombre: &str) -> String {
    std::fs::read_to_string(dir().join(nombre)).unwrap_or_else(|e| panic!("{nombre}: {e}"))
}

fn huella() -> [u8; 32] {
    // §650: sobre bytes, nunca troceando el `&str` (la puerta de zk-ssl-hash).
    let c = |b: u8| (b as char).to_digit(16).unwrap() as u8;
    let h = HUELLA.as_bytes();
    core::array::from_fn(|i| c(h[2 * i]) << 4 | c(h[2 * i + 1]))
}

/// Las semillas de prueba de `main.go`: `desde, desde+1, …`.
fn semilla(desde: u8) -> [u8; 32] {
    core::array::from_fn(|i| desde + i as u8)
}

fn publicador() -> ClaveDeNota {
    ClaveDeNota::leer_vkey(leer("publicador.vkey").trim()).unwrap()
}

fn testigo() -> ClaveDeNota {
    ClaveDeNota::leer_vkey(leer("testigo.vkey").trim()).unwrap()
}

/// El árbol de `main.go`: la huella del ancla `i` es `SHA-256(u64be(i))`.
fn arbol(n: u64) -> ArbolDelMedio {
    let mut a = ArbolDelMedio::nuevo();
    for i in 0..n {
        a.anadir(&sha256(&i.to_be_bytes()));
    }
    a
}

fn clase(e: &ErrorDeNota) -> &'static str {
    match e {
        ErrorDeNota::Malformada(_) => "Malformada",
        ErrorDeNota::Checkpoint(_) => "Checkpoint",
        ErrorDeNota::OrigenAjeno { .. } => "OrigenAjeno",
        ErrorDeNota::SinFirmaDelPublicador => "SinFirmaDelPublicador",
        ErrorDeNota::FirmaRepetida => "FirmaRepetida",
        ErrorDeNota::FirmaInvalida => "FirmaInvalida",
        ErrorDeNota::Mensaje(_) => "Mensaje",
        ErrorDeNota::Clave(_) => "Clave",
        ErrorDeNota::Firmando(_) => "Firmando",
    }
}

#[test]
fn las_mismas_claves_desde_la_misma_semilla() {
    let origen = origen_del_medio(&huella());
    // El medio es el del operador de los vectores del RFC-0012.
    let ancla = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../spec/vectors/ancla/ancla-exacta.json"),
    )
    .unwrap();
    assert!(ancla.contains(&format!("\"clave\": \"0x{HUELLA}\"")));
    let p = Publicador::determinista(&origen, semilla(0x00)).unwrap();
    assert_eq!(p.clave_publica().vkey(), leer("publicador.vkey").trim());
    let t = Publicador::determinista(TESTIGO, semilla(0x20)).unwrap();
    assert_eq!(t.clave_publica().vkey(), leer("testigo.vkey").trim());
    assert_eq!(publicador().nombre(), origen);
}

#[test]
fn las_de_torchwood_verifican_y_la_linea_del_publicador_sale_igual() {
    let clave = publicador();
    let propio = Publicador::determinista(clave.nombre(), semilla(0x00)).unwrap();
    for n in [0u64, 1, 2, 5, 13] {
        let nota = leer(&format!("pos-{n:02}-anclas.txt"));
        let v = verificar_nota(&nota, &clave).unwrap();
        assert_eq!(v.checkpoint.tamano, n);
        assert_eq!(
            v.checkpoint.raiz,
            arbol(n).raiz(),
            "x/mod/sumdb/tlog, {n} hojas"
        );
        // La línea del publicador, firmada aquí con su misma marca.
        let aqui = propio.firmar(&v.checkpoint, v.marca).unwrap();
        assert!(nota.starts_with(&aqui), "{n} anclas: no sale byte a byte");
        // Y detrás, la cofirma del testigo, que verifica con su clave.
        assert_eq!(v.ajenas.len(), 1);
        let marca = testigo()
            .verificar_cofirma(&v.checkpoint, &v.ajenas[0])
            .unwrap();
        assert!(marca > 1_700_000_000);
        // §633: y el cofirmante de aquí, con la misma semilla y la misma
        // marca, escribe la misma línea.
        let linea = Cofirmante::determinista(TESTIGO, semilla(0x20))
            .unwrap()
            .cofirmar(&v.checkpoint, marca)
            .unwrap();
        assert!(
            nota.ends_with(&linea),
            "{n} anclas: la cofirma no sale byte a byte"
        );
        // La del testigo no es del publicador, ni al revés.
        assert!(clave
            .verificar_cofirma(&v.checkpoint, &v.ajenas[0])
            .is_err());
    }
    let sin = verificar_nota(&leer("pos-sin-testigo.txt"), &clave).unwrap();
    assert!(sin.ajenas.is_empty());
    let primero: NotaVerificada = verificar_nota(&leer("pos-testigo-primero.txt"), &clave).unwrap();
    assert_eq!(primero.ajenas.len(), 1);
    assert_eq!(primero.checkpoint, sin.checkpoint);
}

#[test]
fn una_cofirma_tocada_no_verifica() {
    let v = verificar_nota(&leer("pos-05-anclas.txt"), &publicador()).unwrap();
    let mut linea = v.ajenas[0].clone();
    linea.firma[100] ^= 1;
    assert_eq!(
        testigo().verificar_cofirma(&v.checkpoint, &linea),
        Err(ErrorDeNota::FirmaInvalida)
    );
    let mut otro = v.checkpoint.clone();
    otro.tamano += 1;
    assert!(testigo().verificar_cofirma(&otro, &v.ajenas[0]).is_err());
}

#[test]
fn el_manifiesto_dice_lo_que_dicen_los_dos() {
    let mut manifiesto = BTreeMap::new();
    for linea in leer("MANIFIESTO.txt").lines() {
        if linea.starts_with('#') || linea.trim().is_empty() {
            continue;
        }
        let (campos, motivo) = match linea.split_once('#') {
            Some((c, m)) => (c, Some(m.trim().to_string())),
            None => (linea, None),
        };
        let c: Vec<&str> = campos.split_whitespace().collect();
        assert_eq!(c.len(), 3, "{linea}");
        manifiesto.insert(
            c[0].to_string(),
            (c[1].to_string(), c[2].to_string(), motivo),
        );
    }
    let mut torchwood = BTreeMap::new();
    for linea in leer("veredictos-torchwood.txt").lines() {
        if linea.starts_with('#') {
            continue;
        }
        let mut c = linea.split_whitespace();
        torchwood.insert(c.next().unwrap().to_string(), c.next().unwrap().to_string());
    }
    let mut ficheros: Vec<String> = std::fs::read_dir(dir())
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .filter(|f| f.ends_with(".txt") && f != "MANIFIESTO.txt" && !f.starts_with("veredictos"))
        .collect();
    ficheros.sort();
    assert_eq!(
        ficheros,
        manifiesto.keys().cloned().collect::<Vec<_>>(),
        "una nota sin fila o una fila sin nota"
    );
    assert_eq!(ficheros, torchwood.keys().cloned().collect::<Vec<_>>());

    let clave = publicador();
    let (mut aceptadas, mut rechazadas, mut divergencias) = (0, 0, Vec::new());
    for f in &ficheros {
        let (medio, suyo, motivo) = &manifiesto[f];
        let aqui = match verificar_nota(&leer(f), &clave) {
            Ok(_) => "ACEPTA",
            Err(e) => clase(&e),
        };
        assert_eq!(aqui, medio, "{f}: el medio dice {aqui}");
        assert_eq!(
            &torchwood[f], suyo,
            "{f}: el manifiesto no copia a torchwood"
        );
        match (aqui == "ACEPTA", suyo.as_str()) {
            (true, "ACEPTA") => aceptadas += 1,
            (false, "RECHAZA") => rechazadas += 1,
            (false, "ACEPTA") => {
                assert!(motivo.is_some(), "{f}: una divergencia sin motivo");
                divergencias.push(f.as_str());
            }
            (true, _) => panic!("{f}: el medio acepta lo que torchwood rechaza"),
            _ => unreachable!(),
        }
    }
    assert_eq!((aceptadas, rechazadas), (10, 19));
    assert_eq!(
        divergencias,
        [
            "neg-firma-no-canonica.txt",
            "neg-firma-repetida-identica.txt",
            "neg-firma-repetida-y-basura.txt",
            "neg-origen-ajeno-firmado.txt",
        ]
    );
}
