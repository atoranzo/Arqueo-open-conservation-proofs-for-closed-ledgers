//! **§697: el nivel de seguridad de produccion, atado por tests** (ZK-1, paso (1), de
//! `doc/blueprint-v2.md`).
//!
//! Hasta este sello ningun test fijaba el nivel: fuera del fork, `proven_security` solo lo llamaba
//! `stark-experiment/src/compliance_real_proof.rs`, que imprime y no compara, y con otras opciones.
//! Un cambio de [`proof_options`] o de la longitud de un circuito podia bajar la seguridad
//! demostrable sin que nada se pusiera rojo. Este modulo la ata en dos tests:
//!
//! - **Las familias de longitud fija**, sobre pruebas OCULTAS reales que genera el camino de
//!   produccion con [`proof_options`]: el envio v2 (T = 1024, la traza mas larga de las familias
//!   de longitud fija, la misma que la del cobro y la quema), el credito del reembolso (T = 512) y
//!   la apertura del reembolso v2 (T = 64, la mas corta). Cada una exige conjeturada >= 127,
//!   LDR >= 80 y UDR = 59. La UDR se fija en su valor de hoy: solo cambia con un asiento.
//! - **Ningun verificador acepta por nivel minimo.** Fuera del fork, ningun fuente usa las dos
//!   variantes de `AcceptableOptions` que comparan bits calculados y no opciones. El batching y las
//!   particiones no entran en el transcript (`ProofOptions::to_elements` lleva la extension, el
//!   plegado, el resto, el blowup, la molienda y las consultas), y con esas variantes ninguna otra
//!   cosa los fijaria; `OptionSet`, que compara `ProofOptions` entero, si.
//!
//! La edad, cuya longitud depende de `m`, tiene su test en `zk-ssl-air` (`tests_nivel`): alli
//! viven su AIR y sus opciones, y alli se declara el piso de cada `m`.
//!
//! Las cifras son las de la funcion de `winter-air` (`proof/security.rs`) sobre la forma y las
//! opciones de cada prueba: el test fija lo que esa funcion devuelve, no la revisa. Con batching
//! lineal -lo que [`proof_options`] declara, y el test lo exige- la funcion no depende del ancho
//! ni del numero de restricciones, solo de las opciones y de la longitud; por eso basta una prueba
//! por longitud. El primer test prueba STARK reales: en depuracion se salta, como los del
//! instrumento de D-B. El segundo solo lee fuentes.

use crate::tests_support::{new_layer, open_and_fund, salt_de, state_of};
use crate::{client, proof_options, Blake3};
use std::path::{Path, PathBuf};
use winterfell::math::fields::f64::BaseElement;
use winterfell::math::FieldElement;
use winterfell::{BatchingMethod, Proof};

/// La conjeturada minima: el techo que pone el campo con extension cuadratica, 2 * 64 - 1.
const CONJETURADA_MINIMA: u32 = 127;
/// La LDR minima de las familias de longitud fija, la de la mas larga de ellas (§697).
const LDR_MINIMA: u32 = 80;
/// La UDR de hoy, fijada como valor y no como suelo: solo cambia con un asiento (§697).
const UDR_ACTUAL: u32 = 59;

// Claves estrechas y valores dentro del rango de la capa de test (limite 500_000, suministro
// 100_000_000), como el reembolso v2 del instrumento de D-B.
const SK_EMISOR: u64 = 0x697_0001;
const SK_RECEPTOR: u64 = 0x697_0002;
const FONDO: u64 = 61_803_398;
const IMPORTE: u64 = 314_159;
const DELTA: u64 = 1_414_213_562;

/// Lo que la funcion de `winter-air` dice de una prueba: (longitud de la traza, conjeturada, LDR,
/// UDR). Antes, que la prueba sea de produccion: sus opciones son [`proof_options`] y su forma es la
/// OCULTA que la capa exige al verificarla (`comprobar_forma`, la misma regla que los quince
/// `verify` de la capa).
fn nivel(nombre: &str, bytes: &[u8], ancho: usize, longitud: usize) -> (usize, u32, u32, u32) {
    let p = Proof::from_bytes(bytes).unwrap_or_else(|e| panic!("{nombre}: no se lee: {e:?}"));
    assert_eq!(p.options(), &proof_options(), "{nombre}: no lleva las opciones de produccion");
    crate::comprobar_forma(p.trace_info(), ancho, 0, 0, longitud)
        .unwrap_or_else(|e| panic!("{nombre}: no es la forma oculta que la capa exige: {e:?}"));
    let conjeturada = p.conjectured_security::<Blake3>().bits();
    let demostrable = p.proven_security::<Blake3>();
    (p.trace_info().length(), conjeturada, demostrable.ldr_bits(), demostrable.udr_bits())
}

/// **Las familias de longitud fija, sobre la prueba oculta real.** Un envio v2 de una cuenta a
/// otra, aplicado, y el reembolso v2 de ese pendiente: tres pruebas del camino de produccion, de
/// la traza mas larga de las familias de longitud fija a la mas corta. Cada una, con conjeturada
/// >= 127, LDR >= 80 y UDR = 59.
#[test]
#[cfg_attr(debug_assertions, ignore = "winterfell valida grados en depuracion: juez release")]
fn el_nivel_de_las_familias_de_longitud_fija() {
    use stark_experiment::{circuit_credit_climb, circuit_refund_v2, circuit_send_v2};

    let o = proof_options();
    assert_eq!(
        (o.constraint_batching_method(), o.deep_poly_batching_method()),
        (BatchingMethod::Linear, BatchingMethod::Linear),
        "con otro batching el nivel depende del ancho y de las restricciones: hay que medir cada \
         familia, no una prueba por longitud"
    );

    let mut l = new_layer();
    let a = open_and_fund(&mut l, SK_EMISOR, FONDO);
    let b = open_and_fund(&mut l, SK_RECEPTOR, 0);
    let receptor = l.public_id_of(b).expect("receptor");
    let refund_f = salt_de(0x697_0100);
    let m = l
        .send_materials_v2(a, receptor, IMPORTE, salt_de(0x697_0200), refund_f, DELTA)
        .expect("materiales v2");
    let clave = [BaseElement::new(SK_EMISOR), BaseElement::ZERO, BaseElement::ZERO, BaseElement::ZERO];
    let envio = client::prove_send(&m, clave, o.clone()).expect("el envio v2");
    let estado = state_of(&l, a);
    l.apply_send(&envio, a, &estado, IMPORTE).expect("el envio v2 se aplica");
    let aviso = envio.notice.clone();
    let post = state_of(&l, a);
    let reembolso = l
        .refund_v2(
            BaseElement::new(SK_EMISOR),
            a,
            &post,
            aviso.position,
            receptor,
            aviso.salt,
            aviso.amount,
            refund_f,
            DELTA,
        )
        .expect("el reembolso v2");

    let pruebas = [
        (
            "envio v2 (SendV2Air)",
            &envio.proof,
            circuit_send_v2::TRACE_WIDTH,
            circuit_send_v2::TRACE_LENGTH,
        ),
        (
            "credito del reembolso (CreditClimbAir)",
            &reembolso.credit_proof,
            circuit_credit_climb::TRACE_WIDTH,
            circuit_credit_climb::TRACE_LENGTH,
        ),
        (
            "apertura del reembolso v2 (RefundAirV2)",
            &reembolso.refund_proof,
            circuit_refund_v2::TRACE_WIDTH,
            circuit_refund_v2::TRACE_LENGTH,
        ),
    ];
    let mut mal = Vec::new();
    for (nombre, bytes, ancho, longitud) in pruebas {
        let (filas, conjeturada, ldr, udr) = nivel(nombre, bytes, ancho, longitud);
        println!(
            "[§697] {nombre}: T = {longitud}, traza oculta de {filas} filas: conjeturada \
             {conjeturada}, LDR {ldr}, UDR {udr}"
        );
        if conjeturada < CONJETURADA_MINIMA {
            mal.push(format!("{nombre}: conjeturada {conjeturada} < {CONJETURADA_MINIMA}"));
        }
        if ldr < LDR_MINIMA {
            mal.push(format!("{nombre}: LDR {ldr} < {LDR_MINIMA}"));
        }
        if udr != UDR_ACTUAL {
            mal.push(format!("{nombre}: UDR {udr}, y la fijada es {UDR_ACTUAL}: cambia con asiento"));
        }
    }
    assert!(mal.is_empty(), "el nivel de produccion bajo: {}", mal.join("; "));
}

// ---- ningun verificador acepta por nivel minimo ----

/// Los `.rs` de `crates/` menos el fork (`crates/winter-*`, que define las variantes) y los
/// `target`, recorridos con sus subdirectorios.
fn fuentes_fuera_del_fork(crates: &Path) -> Vec<PathBuf> {
    fn recorre(dir: &Path, out: &mut Vec<PathBuf>) {
        let mut hijos: Vec<PathBuf> = std::fs::read_dir(dir)
            .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
            .map(|e| e.expect("entrada").path())
            .collect();
        hijos.sort();
        for h in hijos {
            let nombre = h.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if h.is_dir() {
                if nombre != "target" && !nombre.starts_with('.') {
                    recorre(&h, out);
                }
            } else if nombre.ends_with(".rs") {
                out.push(h);
            }
        }
    }
    let mut out = Vec::new();
    let mut miembros: Vec<PathBuf> = std::fs::read_dir(crates)
        .expect("crates/")
        .map(|e| e.expect("entrada").path())
        .filter(|p| p.is_dir())
        .collect();
    miembros.sort();
    for m in miembros {
        let nombre = m.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !nombre.starts_with("winter-") {
            recorre(&m, &mut out);
        }
    }
    out
}

/// **Ningun verificador acepta por nivel minimo.** Recorre los fuentes de `crates/` fuera del fork
/// y exige que ninguna linea de codigo -las de comentario no cuentan- nombre
/// `MinConjecturedSecurity` ni `MinProvenSecurity`. Las agujas se arman por partes para que este
/// mismo fuente no las contenga. La prueba de vida: al menos 200 ficheros y 146 lineas con
/// `AcceptableOptions::OptionSet`, lo medido en el §697; un universo vacio no pasa.
#[test]
fn ningun_verificador_acepta_por_nivel_minimo() {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let agujas = [concat!("MinConjectured", "Security"), concat!("MinProven", "Security")];
    let opcion = concat!("AcceptableOptions::", "OptionSet");
    let ficheros = fuentes_fuera_del_fork(&crates);
    let (mut con_opciones, mut mal) = (0usize, Vec::new());
    for f in &ficheros {
        let fuente = std::fs::read_to_string(f).unwrap_or_else(|e| panic!("{}: {e}", f.display()));
        for (i, linea) in fuente.lines().enumerate() {
            if linea.trim_start().starts_with("//") {
                continue;
            }
            con_opciones += linea.matches(opcion).count();
            if agujas.iter().any(|a| linea.contains(a)) {
                mal.push(format!("{}:{}: {}", f.display(), i + 1, linea.trim()));
            }
        }
    }
    println!(
        "[§697] {} ficheros fuera del fork; {con_opciones} usos de OptionSet; {} de Min*",
        ficheros.len(),
        mal.len()
    );
    assert!(ficheros.len() >= 200, "solo {} ficheros: el recorrido no ve el arbol", ficheros.len());
    assert!(con_opciones >= 146, "solo {con_opciones} usos de OptionSet: el recorrido no ve el arbol");
    assert!(mal.is_empty(), "verificadores que aceptan por nivel minimo:\n{}", mal.join("\n"));
}
