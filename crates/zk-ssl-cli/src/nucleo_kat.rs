//! # Los KAT del nucleo congelado (RFC-0005, E5; S411)
//!
//! Un vector de respuesta conocida (KAT: entrada -> salida) por cada `fn` NUCLEO de
//! `zk-ssl-hash` y `zk-ssl-verify`, en `spec/vectors/nucleo/`. Son **la norma de los bytes**:
//! `spec/NUCLEO.md` (seccion 6) nombra la permutacion, la serializacion, los dominios y los
//! preambulos; los ficheros de aqui fijan los bytes exactos. Una segunda implementacion
//! comprueba su hash contra ellos **sin leer winterfell ni este arbol**, y solo despues tiene
//! sentido pasarle el arnes de E4.
//!
//! Molde de `conformance` (`--emit` fija, `--check` reproduce): con `ZKSSL_KAT_EMITIR` en el
//! entorno el test ESCRIBE los ficheros (la referencia fija la foto); sin ella, los LEE y
//! COMPARA valor a valor. `ZKSSL_KAT_DIR` cambia el directorio, para emitir aparte y para
//! ensayar el gate contra una copia saboteada.
//!
//! Cada fichero es `{"fn": ..., "entradas": {...}, "salida": ...}`; los digests y los bytes van
//! en hex `0x…` con la serializacion del cable (`digest_to_bytes`: cuatro elementos, ocho bytes
//! little-endian cada uno); los `u64` como QUANTITY (`0x…`). Las entradas son pequenas y
//! deterministas —`as_digest(1..6)`, elementos 5 y 9—: lo que importa no es la entrada sino que
//! dos implementaciones den la misma salida.
//!
//! ⚠️ El test tambien exige que **cada fichero del directorio tenga su caso y cada caso su
//! fichero**: un vector huerfano o uno de menos es ROJO, igual que en `tools/conformidad.sh`.

use std::path::PathBuf;

use serde_json::{json, Value};
use zk_ssl_hash::{
    acuse_digest, as_digest, digest_from_bytes, digest_to_bytes, element_from_bytes, embeber,
    epoch_digest, epoch_digest_v2, epoch_digest_v3, epoch_digest_v4, mmr_hoja, mmr_nodo,
    native_leaf,
    native_leaf_salted, native_merge, path_root, Digest,
};
use zk_ssl_verify::{acuses::hoja_de_acuse, mmr::cima, preambulo, preambulo_cofirma};

/// `spec/vectors/nucleo/` relativo a este crate, salvo que `ZKSSL_KAT_DIR` diga otro.
fn directorio() -> PathBuf {
    match std::env::var("ZKSSL_KAT_DIR") {
        Ok(d) => PathBuf::from(d),
        Err(_) => PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../spec/vectors/nucleo"),
    }
}

fn hx(b: &[u8]) -> String {
    let mut s = String::with_capacity(2 + b.len() * 2);
    s.push_str("0x");
    for x in b {
        s.push_str(&format!("{x:02x}"));
    }
    s
}

fn dg(d: &Digest) -> String {
    hx(&digest_to_bytes(d))
}

fn q(x: u64) -> String {
    format!("{x:#x}")
}

/// Los casos, en el orden del censo de `spec/NUCLEO.md`. Un caso por `fn` NUCLEO que un
/// tercero recompone; las entradas son bytes, nunca tipos de este arbol.
fn casos() -> Vec<(&'static str, Value)> {
    let (a, b, c, d, e, f) = (
        as_digest(1),
        as_digest(2),
        as_digest(3),
        as_digest(4),
        as_digest(5),
        as_digest(6),
    );
    let saldo = element_from_bytes(&5u64.to_le_bytes()).unwrap_or_else(|_| panic!("saldo"));
    let nonce = element_from_bytes(&9u64.to_le_bytes()).unwrap_or_else(|_| panic!("nonce"));
    let siete = element_from_bytes(&7u64.to_le_bytes()).unwrap_or_else(|_| panic!("siete"));
    let clave_op: [u8; 2] = [0xcc, 0xcc];
    vec![
        ("as_digest", json!({"fn": "as_digest", "entradas": {"x": q(7)},
            "salida": dg(&as_digest(7))})),
        ("embeber", json!({"fn": "embeber", "entradas": {"x": hx(&7u64.to_le_bytes())},
            "salida": dg(&embeber(siete))})),
        ("digest_to_bytes", json!({"fn": "digest_to_bytes",
            "entradas": {"d": "as_digest(x)", "x": q(0x102)},
            "salida": hx(&digest_to_bytes(&as_digest(0x102)))})),
        ("digest_from_bytes", json!({"fn": "digest_from_bytes", "entradas": {"bytes": dg(&a)},
            "salida": dg(&digest_from_bytes(&digest_to_bytes(&a))
                .unwrap_or_else(|_| panic!("32 bytes")))})),
        ("native_merge", json!({"fn": "native_merge",
            "entradas": {"left": dg(&a), "right": dg(&b)},
            "salida": dg(&native_merge(a, b))})),
        ("native_leaf", json!({"fn": "native_leaf",
            "entradas": {"public_id": dg(&a), "balance": hx(&5u64.to_le_bytes()),
                         "nonce": hx(&9u64.to_le_bytes())},
            "salida": dg(&native_leaf(a, saldo, nonce))})),
        ("native_leaf_salted", json!({"fn": "native_leaf_salted",
            "entradas": {"public_id": dg(&a), "balance": hx(&5u64.to_le_bytes()),
                         "nonce": hx(&9u64.to_le_bytes()), "leaf_salt": dg(&c)},
            "salida": dg(&native_leaf_salted(a, saldo, nonce, c))})),
        ("path_root", json!({"fn": "path_root",
            "entradas": {"leaf": dg(&a), "siblings": [dg(&b), dg(&c)], "is_right": [true, false]},
            "salida": dg(&path_root(a, &[b, c], &[true, false]))})),
        ("epoch_digest", json!({"fn": "epoch_digest",
            "entradas": {"seq": q(0x10), "accounts_root": dg(&a), "pending_root": dg(&b),
                         "frozen_root": dg(&c), "chain_digest": dg(&d)},
            "salida": dg(&epoch_digest(0x10, a, b, c, d))})),
        ("epoch_digest_v2", json!({"fn": "epoch_digest_v2",
            "entradas": {"seq": q(0x10), "accounts_root": dg(&a), "pending_root": dg(&b),
                         "frozen_root": dg(&c), "chain_digest": dg(&d), "acuses_root": dg(&e),
                         "n": q(0x11)},
            "salida": dg(&epoch_digest_v2(0x10, a, b, c, d, e, 0x11))})),
        ("epoch_digest_v3", json!({"fn": "epoch_digest_v3",
            "entradas": {"seq": q(0x10), "accounts_root": dg(&a), "pending_root": dg(&b),
                         "frozen_root": dg(&c), "chain_digest": dg(&d), "acuses_root": dg(&e),
                         "n": q(0x11), "cima_mmr": dg(&f), "t": q(0x12)},
            "salida": dg(&epoch_digest_v3(0x10, a, b, c, d, e, 0x11, f, 0x12))})),
        ("epoch_digest_v4", json!({"fn": "epoch_digest_v4",
            "entradas": {"seq": q(0x10), "accounts_root": dg(&a), "pending_root": dg(&b),
                         "frozen_root": dg(&c), "chain_digest": dg(&d), "acuses_root": dg(&e),
                         "n": q(0x11), "cima_mmr": dg(&f), "t": q(0x12), "cons_root": dg(&b),
                         "cons_count": q(0x13)},
            "salida": dg(&epoch_digest_v4(0x10, a, b, c, d, e, 0x11, f, 0x12, b, 0x13))})),
        ("acuse_digest", json!({"fn": "acuse_digest",
            "entradas": {"hash_prueba": dg(&a), "epoca": q(2), "n": q(3)},
            "salida": dg(&acuse_digest(a, 2, 3))})),
        ("hoja_de_acuse", json!({"fn": "hoja_de_acuse",
            "entradas": {"hash_prueba": dg(&a), "seq": q(0x2b), "n": q(3)},
            "salida": dg(&hoja_de_acuse(a, 0x2b, 3))})),
        ("mmr_hoja", json!({"fn": "mmr_hoja", "entradas": {"cabeza": dg(&a)},
            "salida": dg(&mmr_hoja(a))})),
        ("mmr_nodo", json!({"fn": "mmr_nodo",
            "entradas": {"izquierda": dg(&a), "derecha": dg(&b)},
            "salida": dg(&mmr_nodo(a, b))})),
        ("cima", json!({"fn": "cima", "entradas": {"hojas": [dg(&a), dg(&b), dg(&c)]},
            "salida": dg(&cima(&[a, b, c]).unwrap_or_else(|| panic!("tres hojas")))})),
        ("preambulo", json!({"fn": "preambulo",
            "entradas": {"version": 3, "epoch_digest": dg(&a)},
            "salida": hx(&preambulo(3, &digest_to_bytes(&a)))})),
        ("preambulo_cofirma", json!({"fn": "preambulo_cofirma",
            "entradas": {"version": 3, "epoch_digest": dg(&a), "clave_del_operador": hx(&clave_op)},
            "salida": hx(&preambulo_cofirma(3, &digest_to_bytes(&a), &clave_op)
                .unwrap_or_else(|_| panic!("clave corta")))})),
    ]
}

/// Los KAT se reproducen byte a byte, y el directorio y los casos son el mismo conjunto.
#[test]
fn los_kat_del_nucleo_se_reproducen_byte_a_byte() {
    let casos = casos();
    let dir = directorio();
    if std::env::var("ZKSSL_KAT_EMITIR").is_ok() {
        std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("crear {}: {e}", dir.display()));
        for (nombre, v) in &casos {
            let texto = serde_json::to_string_pretty(v).unwrap_or_else(|e| panic!("json: {e}")) + "\n";
            std::fs::write(dir.join(format!("{nombre}.json")), texto)
                .unwrap_or_else(|e| panic!("escribir {nombre}: {e}"));
        }
        eprintln!("KAT escritos: {} en {}", casos.len(), dir.display());
        return;
    }
    let mut en_disco: Vec<String> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("KAT: no se puede leer {}: {e}", dir.display()))
        .filter_map(|x| x.ok())
        .filter_map(|x| x.file_name().to_str().map(|s| s.to_string()))
        .filter(|s| s.ends_with(".json"))
        .collect();
    en_disco.sort();
    let mut esperados: Vec<String> = casos.iter().map(|(n, _)| format!("{n}.json")).collect();
    esperados.sort();
    assert_eq!(en_disco, esperados, "KAT: los ficheros de {} y los casos NO coinciden", dir.display());
    for (nombre, calculado) in &casos {
        let ruta = dir.join(format!("{nombre}.json"));
        let texto = std::fs::read_to_string(&ruta).unwrap_or_else(|e| panic!("KAT {nombre}: {e}"));
        let fijado: Value = serde_json::from_str(&texto)
            .unwrap_or_else(|e| panic!("KAT {nombre}: JSON ilegible: {e}"));
        assert_eq!(
            &fijado, calculado,
            "KAT {nombre}: el arbol NO reproduce los bytes fijados en {}",
            ruta.display()
        );
    }
}
