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
//! entorno el test ESCRIBE los ficheros que FALTAN (la referencia fija la foto de un KAT nuevo);
//! sin ella, los LEE y COMPARA valor a valor. `ZKSSL_KAT_DIR` cambia el directorio, para emitir
//! aparte y para ensayar el gate contra una copia saboteada.
//!
//! ⚠️ Desde el §692 **emitir no reescribe un KAT publicado**: un fichero que ya esta con los
//! mismos bytes se deja, y uno que esta con OTROS bytes hace que la emision se niegue entera,
//! sin escribir nada, y nombre el fichero. Un KAT es un vector de `spec/vectors/` como los demas
//! (regla 2 de `spec/rfc/PROCESO.md`): si una `fn` NUCLEO cambia, su KAT nuevo entra con otro
//! nombre o bajo una version nueva, y el viejo se conserva. `tools/check_vectores.py` lo vigila
//! en cada canon; esto lo dice antes, en la herramienta que lo escribiria.
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
    acta_digest, acuse_digest, ancla_digest, as_digest, digest_from_bytes, digest_pi, digest_to_bytes, element_from_bytes,
    embeber, epoch_digest, epoch_digest_v2, epoch_digest_v3, epoch_digest_v4, epoch_digest_v5,
    epoch_digest_v6, hash_del_lote, huella_de_clave, mmr_hoja, mmr_nodo, native_leaf, native_leaf_salted,
    native_merge, params_digest, path_root, recibo_digest, recibo_digest_v2, Digest,
};
use zk_ssl_verify::{
    actas::preambulo_acta, acuses::hoja_de_acuse, mmr::cima, preambulo, preambulo_cofirma,
};

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
        ("epoch_digest_v5", json!({"fn": "epoch_digest_v5",
            "entradas": {"seq": q(0x10), "accounts_root": dg(&a), "pending_root": dg(&b),
                         "frozen_root": dg(&c), "chain_digest": dg(&d), "acuses_root": dg(&e),
                         "n": q(0x11), "cima_mmr": dg(&f), "t": q(0x12), "cons_root": dg(&b),
                         "cons_count": q(0x13), "params_digest": dg(&c), "pmeta_root": dg(&d),
                         "next_pending": q(0x14), "next_index": q(0x15), "total_supply": q(0x16)},
            "salida": dg(&epoch_digest_v5(0x10, a, b, c, d, e, 0x11, f, 0x12, b, 0x13, c, d,
                                          0x14, 0x15, 0x16))})),
        ("epoch_digest_v6", json!({"fn": "epoch_digest_v6",
            "entradas": {"seq": q(0x10), "accounts_root": dg(&a), "pending_root": dg(&b),
                         "frozen_root": dg(&c), "chain_digest": dg(&d), "acuses_root": dg(&e),
                         "n": q(0x11), "cima_mmr": dg(&f), "t": q(0x12), "cons_root": dg(&b),
                         "cons_count": q(0x13), "params_digest": dg(&c), "pmeta_root": dg(&d),
                         "next_pending": q(0x14), "next_index": q(0x15), "total_supply": q(0x16),
                         "recep_root": dg(&e), "recep_count": q(0x17)},
            "salida": dg(&epoch_digest_v6(0x10, a, b, c, d, e, 0x11, f, 0x12, b, 0x13, c, d,
                                          0x14, 0x15, 0x16, e, 0x17))})),
        ("params_digest", json!({"fn": "params_digest",
            "entradas": {"regulatory_limit": q(0x20), "max_supply": q(0x21),
                         "max_accounts": q(0x22), "custodian_set_root": dg(&a),
                         "governance_set_root": dg(&b), "refund_ttl": q(0x23),
                         "max_custodian_uses": q(0x24)},
            "salida": dg(&params_digest(0x20, 0x21, 0x22, a, b, 0x23, 0x24))})),
        ("acuse_digest", json!({"fn": "acuse_digest",
            "entradas": {"hash_prueba": dg(&a), "epoca": q(2), "n": q(3)},
            "salida": dg(&acuse_digest(a, 2, 3))})),
        ("recibo_digest", json!({"fn": "recibo_digest",
            "entradas": {"hash_prueba": dg(&a), "era": q(2), "n": q(3)},
            "salida": dg(&recibo_digest(a, 2, 3))})),
        ("recibo_digest_v2", json!({"fn": "recibo_digest_v2",
            "entradas": {"hash_prueba": dg(&a), "digest_pi": dg(&b), "era": q(2), "n": q(3)},
            "salida": dg(&recibo_digest_v2(a, b, 2, 3))})),
        ("ancla_digest", json!({"fn": "ancla_digest",
            "entradas": {"huella_clave": dg(&a), "indice": q(7), "epoch_digest": dg(&b),
                         "mmr_root": dg(&c), "mmr_size": q(9)},
            "salida": dg(&ancla_digest(a, 7, b, c, 9))})),
        ("acta_digest_genesis", json!({"fn": "acta_digest",
            "entradas": {"huella_clave": dg(&a), "esquema": q(0x1_0000_0005), "desde": q(0),
                         "siguiente": dg(&b), "procedencia": null},
            "salida": dg(&acta_digest(a, 0x1_0000_0005, 0, b, None))})),
        ("acta_digest_rotacion", json!({"fn": "acta_digest",
            "entradas": {"huella_clave": dg(&b), "esquema": q(0x1_0000_0005), "desde": q(0x28),
                         "siguiente": dg(&c),
                         "procedencia": {"anterior": dg(&a), "epoch_digest": dg(&d),
                                         "mmr_root": dg(&e), "mmr_size": q(0x27)}},
            "salida": dg(&acta_digest(b, 0x1_0000_0005, 0x28, c, Some((a, d, e, 0x27))))})),
        ("huella_de_clave", json!({"fn": "huella_de_clave",
            "entradas": {"clave": hx(&clave_op)},
            "salida": dg(&huella_de_clave(&clave_op))})),
        ("hash_del_lote", json!({"fn": "hash_del_lote",
            "entradas": {"operaciones": [
                {"hash_prueba": dg(&a), "cuenta": q(1), "posicion": q(2)},
                {"hash_prueba": dg(&b), "cuenta": q(3), "posicion": q(4)}]},
            "salida": dg(&hash_del_lote(&[(a, 1, 2), (b, 3, 4)]))})),
        ("digest_pi", json!({"fn": "digest_pi",
            "entradas": {"familia": q(1), "version_air": q(1),
                         "entradas_publicas": [hx(&5u64.to_le_bytes()), hx(&9u64.to_le_bytes())]},
            "salida": dg(&digest_pi(1, 1, &[saldo, nonce]))})),
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
        ("preambulo_acta", json!({"fn": "preambulo_acta",
            "entradas": {"version": 1, "acta_digest": dg(&a)},
            "salida": hx(&preambulo_acta(1, &digest_to_bytes(&a)))})),
    ]
}

/// Escribe en `dir` los KAT que FALTAN y devuelve cuantos escribio. Un KAT que ya esta con los
/// mismos bytes se deja; si alguno esta con OTROS bytes, no escribe NINGUNO y devuelve el
/// porque, con el fichero (§692): primero se comprueban todos, despues se escribe.
fn emitir(dir: &std::path::Path, casos: &[(&str, Value)]) -> Result<usize, String> {
    let mut nuevos = Vec::new();
    for (nombre, v) in casos {
        let texto = serde_json::to_string_pretty(v).map_err(|e| format!("json de {nombre}: {e}"))? + "\n";
        let ruta = dir.join(format!("{nombre}.json"));
        match std::fs::read(&ruta) {
            Ok(publicado) if publicado == texto.as_bytes() => {}
            Ok(_) => {
                return Err(format!(
                    "KAT {nombre}: {} ya esta publicado con otros bytes, y un KAT publicado no se \
                     reescribe (regla 2 de spec/rfc/PROCESO.md): si la fn cambio, su KAT nuevo va \
                     con otro nombre o bajo una version nueva. No se ha escrito nada.",
                    ruta.display()
                ))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => nuevos.push((ruta, texto)),
            Err(e) => return Err(format!("KAT {nombre}: no se puede leer {}: {e}", ruta.display())),
        }
    }
    std::fs::create_dir_all(dir).map_err(|e| format!("crear {}: {e}", dir.display()))?;
    for (ruta, texto) in &nuevos {
        std::fs::write(ruta, texto).map_err(|e| format!("escribir {}: {e}", ruta.display()))?;
    }
    Ok(nuevos.len())
}

/// Los KAT se reproducen byte a byte, y el directorio y los casos son el mismo conjunto.
#[test]
fn los_kat_del_nucleo_se_reproducen_byte_a_byte() {
    let casos = casos();
    let dir = directorio();
    if std::env::var("ZKSSL_KAT_EMITIR").is_ok() {
        let n = emitir(&dir, &casos).unwrap_or_else(|e| panic!("{e}"));
        eprintln!("KAT escritos: {n} nuevos de {} casos en {}", casos.len(), dir.display());
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

/// §692: emitir solo AÑADE. Lo que falta se escribe; lo que esta igual se deja; un KAT publicado
/// con otros bytes hace que la emision se niegue entera, sin escribir nada. Y los KAT publicados
/// son, byte a byte, lo que el emisor escribiria hoy: una emision honesta no choca con ellos.
#[test]
fn emitir_no_reescribe_un_kat_publicado() {
    let dir = std::env::temp_dir().join(format!("zkssl-kat-emitir-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let leer = |n: &str| std::fs::read(dir.join(n)).unwrap_or_else(|e| panic!("leer {n}: {e}"));
    let dos = [("a", json!({"fn": "a", "salida": "0x01"})), ("b", json!({"fn": "b", "salida": "0x02"}))];

    assert_eq!(emitir(&dir, &dos), Ok(2), "en un directorio vacio se escriben los dos");
    let a = leer("a.json");
    assert_eq!(emitir(&dir, &dos), Ok(0), "con los dos ya publicados igual, no se escribe nada");
    assert_eq!(leer("a.json"), a, "y el publicado no se toca");

    std::fs::write(dir.join("a.json"), "{}\n").unwrap_or_else(|e| panic!("sabotear a: {e}"));
    let tres = [dos[0].clone(), dos[1].clone(), ("c", json!({"fn": "c", "salida": "0x03"}))];
    let e = emitir(&dir, &tres).err().unwrap_or_else(|| panic!("a.json con otros bytes tenia que negarse"));
    assert!(e.contains("a.json") && e.contains("no se reescribe"), "el porque nombra el fichero: {e}");
    assert_eq!(leer("a.json"), b"{}\n", "el KAT con otros bytes NO se reescribe");
    assert!(!dir.join("c.json").exists(), "y la emision negada no escribe ni el que faltaba");

    let publicados = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../spec/vectors/nucleo");
    let copia = dir.join("nucleo");
    std::fs::create_dir_all(&copia).unwrap_or_else(|e| panic!("crear la copia: {e}"));
    for x in std::fs::read_dir(&publicados).unwrap_or_else(|e| panic!("leer {}: {e}", publicados.display())) {
        let x = x.unwrap_or_else(|e| panic!("entrada: {e}"));
        std::fs::copy(x.path(), copia.join(x.file_name())).unwrap_or_else(|e| panic!("copiar: {e}"));
    }
    assert_eq!(emitir(&copia, &casos()), Ok(0), "los KAT publicados son los bytes que el emisor escribe hoy");
    let _ = std::fs::remove_dir_all(&dir);
}
