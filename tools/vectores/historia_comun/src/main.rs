//! §722, la historia comun. Los negativos del RELEVO que una mutacion no produce: cada uno
//! lleva un acta firmada de nuevo, con otra `procedencia`, por las claves de los dos operadores de
//! acuerdo del banco (`tools/banco_historia_comun.sh`). Parte de la refutacion capturada -sus dos
//! cabezas, su camino y la genesis de A tal cual- y solo cambia lo que el relevo juzga.
//!
//! Uso: `historia-comun <refutacion.json> <dir de salida>`. Determinista: XMSS lo es, y cada acta
//! se firma en una hoja fija que el banco quemo sin firmar (B salto de la 7 a la 1000000).
//!
//! Tras la revision, dos sobres mas con UNA cadena de tres: la de la refutacion y un acta de C (la
//! clave que B compromete, semilla '63') que releva en el mmrSize de la cabeza de B. Con esa cadena
//! la extension de la refutacion cae por el borde de arriba de la posterior, y el solapamiento de
//! esa misma cabeza de B delata: una cadena, un veredicto. C no firmo nunca: cualquier hoja vale.

use std::path::Path;

use serde_json::{json, Value};
use xmss::{KeyPair, SigningKey};
use zk_ssl_hash::{digest_from_bytes, hex_canonico, Digest};
use zk_ssl_verify::actas::{
    acta_a_json, acta_de_json, huella_de_clave, preambulo_acta, Acta, ActaFirmada, Procedencia,
    ACTA_VERSION, ESQUEMA_XMSSMT_SHA2_40_8_256,
};
use zk_ssl_verify::{aplicar_apano_del_oid, Conjunto};

fn clave(semilla: u8) -> KeyPair<Conjunto> {
    KeyPair::<Conjunto>::from_seed(&[semilla; 96]).expect("keygen")
}

fn publica(kp: &KeyPair<Conjunto>) -> Vec<u8> {
    kp.verifying_key().as_ref().to_vec()
}

/// Firma el acta con la hoja `desde` de la clave, como el nodo: el `desde` ES la hoja de su firma.
fn firmar(kp: &mut KeyPair<Conjunto>, acta: Acta) -> ActaFirmada {
    let mut sk = kp.signing_key().as_ref().to_vec();
    zk_ssl_guardian::poner_indice_en_sk(&mut sk, acta.desde).expect("indice");
    aplicar_apano_del_oid(&mut sk).expect("apano del OID");
    *kp.signing_key() = SigningKey::<Conjunto>::try_from(sk.as_slice()).expect("sk");
    let pre = preambulo_acta(ACTA_VERSION, &zk_ssl_hash::digest_to_bytes(&acta.digest()));
    let firma = kp.signing_key().sign(&pre).expect("firmar").as_ref().to_vec();
    ActaFirmada {
        acta,
        firma,
        firma_anterior: None,
    }
}

fn digest(v: &Value, campo: &str) -> Digest {
    let b = hex_canonico(v[campo].as_str().expect(campo)).expect(campo);
    digest_from_bytes(&b).expect(campo)
}

fn q(v: &Value, campo: &str) -> u64 {
    u64::from_str_radix(v[campo].as_str().expect(campo).trim_start_matches("0x"), 16).expect(campo)
}

fn rotacion(
    entra: &[u8],
    sale: &[u8],
    desde: u64,
    siguiente: Digest,
    (mmr_size, mmr_root, epoch_digest): (u64, Digest, Digest),
) -> Acta {
    Acta {
        clave: entra.to_vec(),
        esquema: ESQUEMA_XMSSMT_SHA2_40_8_256,
        desde,
        siguiente,
        procedencia: Some(Procedencia {
            anterior: huella_de_clave(sale),
            epoch_digest,
            mmr_root,
            mmr_size,
        }),
    }
}

fn guardar(dir: &Path, nombre: &str, base: &Value, actas: &[ActaFirmada]) {
    let mut s = base.clone();
    s["actas"] = json!(actas.iter().map(acta_a_json).collect::<Vec<_>>());
    let ruta = dir.join(format!("{nombre}.json"));
    std::fs::write(&ruta, serde_json::to_string_pretty(&s).expect("json") + "\n").expect("escribir");
    eprintln!("historia-comun: {}", ruta.display());
}

/// El sobre del solapamiento de una cabeza con una cadena: la forma de los capturados.
fn solapamiento(cabeza: &Value) -> Value {
    json!({ "v": 1, "tipo": "solapamiento", "cabeza": cabeza.clone() })
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (entrada, salida) = match args.as_slice() {
        [_, e, s] => (e, Path::new(s)),
        _ => {
            eprintln!("uso: historia-comun <refutacion.json> <dir de salida>");
            std::process::exit(2)
        }
    };
    let base: Value =
        serde_json::from_str(&std::fs::read_to_string(entrada).expect("leer")).expect("json");
    let (h_a, h_b) = (&base["vieja"], &base["nueva"]);
    let genesis = acta_de_json(&base["actas"][0]).expect("la genesis capturada");
    let rot_b = acta_de_json(&base["actas"][1]).expect("la rotacion capturada");
    let (t_a, t_b) = (q(h_a, "mmrSize"), q(h_b, "mmrSize"));
    let (cima_a, cima_b) = (digest(h_a, "mmrRoot"), digest(h_b, "mmrRoot"));
    let (d_a, d_b) = (digest(h_a, "epochDigest"), digest(h_b, "epochDigest"));
    let (mut ka, mut kb, mut kc, mut kx) = std::thread::scope(|s| {
        let h = [0x61u8, 0x62, 0x63, 0x64].map(|x| s.spawn(move || clave(x)));
        let [a, b, c, x] = h.map(|h| h.join().expect("keygen"));
        (a, b, c, x)
    });
    let (pa, pb, pc, px) = (publica(&ka), publica(&kb), publica(&kc), publica(&kx));
    assert_eq!(genesis.acta.clave, pa, "la genesis capturada es la de la semilla 61");
    assert_eq!(rot_b.acta.clave, pb, "la rotacion capturada es la de la semilla 62");
    assert_eq!(rot_b.acta.siguiente, huella_de_clave(&pc), "B compromete la de la semilla 63");
    let siguiente_b = rot_b.acta.siguiente;
    // Hojas que el banco dejo quemadas sin firma: A firmo por debajo de la 20; B salto de la 7 a
    // la 1000000. Una por acta, para no firmar dos mensajes con la misma hoja.
    let mut b_en = |desde: u64, relevo| firmar(&mut kb, rotacion(&pb, &pa, desde, siguiente_b, relevo));
    // 1. El relevo en el tamano de la vieja: la vieja ya no es anterior a el.
    let antes_de_la_vieja = b_en(500_001, (t_a, cima_a, d_a));
    // 2. El relevo por encima de la nueva: la nueva no ha entrado.
    let tras_la_nueva = b_en(500_002, (t_b + 1, cima_b, d_a));
    // 3. El relevo en el tamano de la nueva, con otra cima que la suya.
    let otra_cima = b_en(500_003, (t_b, cima_a, d_a));
    // 4. Una cadena de tres que retrocede: A -> X (relevo en t_b) -> B (relevo en t_b - 1).
    let genesis_ax = firmar(
        &mut ka,
        Acta {
            clave: pa.clone(),
            esquema: ESQUEMA_XMSSMT_SHA2_40_8_256,
            desde: 200_000,
            siguiente: huella_de_clave(&px),
            procedencia: None,
        },
    );
    let rot_x = firmar(
        &mut kx,
        rotacion(&px, &pa, 300_000, huella_de_clave(&pb), (t_b, cima_b, d_a)),
    );
    let rot_xb = firmar(&mut kb, rotacion(&pb, &px, 500_004, siguiente_b, (t_b - 1, cima_a, d_a)));
    // 5. Tras la revision: C entra por encima de la hoja de la cabeza de B, que queda en su tramo
    //    por indice, y releva en el mmrSize de esa misma cabeza. La extension de la refutacion cae por
    //    el borde de arriba de la posterior, y su solapamiento, con la misma cadena, delata.
    let rot_c = firmar(
        &mut kc,
        rotacion(&pc, &pb, 2_000_000, huella_de_clave(&px), (t_b, cima_b, d_b)),
    );
    let tres = [genesis.clone(), rot_b.clone(), rot_c];
    std::fs::create_dir_all(salida).expect("dir");
    guardar(salida, "neg-historia-comun-relevo-en-la-vieja", &base, &[genesis.clone(), antes_de_la_vieja]);
    guardar(salida, "neg-historia-comun-relevo-tras-la-nueva", &base, &[genesis.clone(), tras_la_nueva]);
    guardar(salida, "neg-historia-comun-relevo-con-otra-cima", &base, &[genesis, otra_cima]);
    guardar(salida, "neg-historia-comun-relevo-retrocede", &base, &[genesis_ax, rot_x, rot_xb]);
    guardar(salida, "neg-historia-comun-la-nueva-tras-su-relevo", &base, &tres);
    guardar(salida, "historia-comun-solap-la-nueva-tras-su-relevo", &solapamiento(h_b), &tres);
}
