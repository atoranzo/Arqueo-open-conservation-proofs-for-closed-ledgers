//! **RFC-0008 E3: la BOCA del prendador** (S543). `zk-ssl-cli prueba-prenda` escribe el sobre
//! `prenda` (spec/PAQUETE.md, 2.10) con la cabeza firmada y la foto del ultimo latido de un nodo
//! VIVO, y con `--publicar` pide ademas a `zkssl_pledge` que escriba la marca bajo ese latido.
//!
//! D-BC..D-BI: la clave entra por el keystore del SDK y el cli no ve un `Digest` de gasto; la
//! frase, por fichero; sin `--receptor`, que se deriva de la clave; `--publicar` opcional, con el
//! sobre escrito SIEMPRE; y lectores PROPIOS de la cabeza y de la foto, con la razon de la prenda.

use std::path::PathBuf;
use std::time::Duration;

use clap::Args;
use serde::Deserialize;
use serde_json::{json, Value};
use stark_experiment::merkle::MerklePath;
use zk_ssl::prueba_prenda::{CabezaDeLaPrenda, SobrePrenda};
use zk_ssl_sdk::{keystore, Wallet};
use zk_ssl_verify::VersionCabeza;
use zk_ssl_wire::{digest_from_wire, digest_to_wire, Blob, MerklePathDto, SignedEpochHeadDto, Q};

use crate::cobro::{escribir, notice_de, q_de, respuesta, AvisoV2};

/// La frase del keystore, de SU fichero (D-BG): sin los saltos del final, y vacia no vale.
pub fn leer_frase(ruta: &PathBuf) -> anyhow::Result<String> {
    let crudo = std::fs::read_to_string(ruta)
        .map_err(|e| anyhow::anyhow!("{}: no se puede leer la frase: {e}", ruta.display()))?;
    let frase = crudo.trim_end_matches(&['\n', '\r'][..]);
    if frase.is_empty() {
        anyhow::bail!("{}: la frase esta vacia", ruta.display());
    }
    Ok(frase.to_string())
}

/// El wallet, del keystore del SDK con la frase de su fichero (D-BC): el cli nunca ve la clave.
pub fn wallet_de(keystore_ruta: &PathBuf, frase_ruta: &PathBuf) -> anyhow::Result<Wallet> {
    let frase = leer_frase(frase_ruta)?;
    keystore::load(keystore_ruta, &frase)
}

/// De `zkssl_signedEpochHead`: la cabeza VERBATIM y lo que la prenda necesita, con SU razon (D-BF,
/// D-BI): la FAMILIA del estado (`lleva_parametros`) porque es la que el nodo sirve y contra la
/// que juzga `zkssl_pledge`, no por la meta.
pub fn leer_cabeza(v: &Value) -> Result<(Value, CabezaDeLaPrenda), String> {
    let obj = v.get("result").cloned().unwrap_or_else(|| v.clone());
    let dto: SignedEpochHeadDto = serde_json::from_value(obj.clone())
        .map_err(|e| format!("la cabeza no es una respuesta del cable: {e}"))?;
    let vista = dto
        .firmada()
        .map_err(|e| format!("cabeza malformada: {e}"))?
        .ok_or_else(|| "la respuesta no lleva cabeza firmada (available: false)".to_string())?;
    // RFC-0010 E2a (§559 en el binario, §563 aqui): la boca exige la FAMILIA del estado, no
    // la variante. Preguntaba <<es exactamente 5?>> y una v6 -que la lleva entera, porque
    // epoch_digest_v6 compone sobre la v5- se habria quedado fuera en silencio.
    // ⚠️ MOLDE: `zk-ssl-verify/src/main.rs` ya lleva este predicado y ESTE MISMO texto para
    // la misma boca. Los dos productores se CRUZAN en el bloque que los junto.
    if !matches!(
        VersionCabeza::try_from(vista.format_version.0),
        Ok(v) if v.lleva_parametros()
    ) {
        return Err(format!(
            "formatVersion {}: la prenda exige una cabeza {} - no por la meta, que no lleva \
             (D-AY), sino porque es la que el nodo sirve y contra la que juzga zkssl_pledge",
            vista.format_version.0,
            VersionCabeza::texto_con_parametros()
        ));
    }
    let cab = CabezaDeLaPrenda {
        seq: vista.seq.0,
        pending_root: digest_from_wire(&vista.pending_root)
            .map_err(|e| format!("pendingRoot: {e:?}"))?,
    };
    Ok((obj, cab))
}

/// De `zkssl_pendingPath`: el camino de la foto, SOLO si es del latido de la cabeza. La prenda no
/// lleva la meta (D-AY): `hermanosMeta`, `emisor` y `nacido` ni se exigen ni se leen.
pub fn leer_camino(v: &Value, seq: u64) -> Result<MerklePath, String> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct R {
        available: bool,
        reason: Option<String>,
        s: Option<Q>,
        camino_pendiente: Option<MerklePathDto>,
    }
    let obj = v.get("result").cloned().unwrap_or_else(|| v.clone());
    let r: R = serde_json::from_value(obj)
        .map_err(|e| format!("la foto no es una respuesta de zkssl_pendingPath: {e}"))?;
    if !r.available {
        return Err(format!(
            "el nodo no sirve la foto: {}",
            r.reason.unwrap_or_else(|| "sin reason".to_string())
        ));
    }
    let s = r.s.ok_or_else(|| "available sin s".to_string())?.0;
    if s != seq {
        return Err(format!(
            "la foto es del latido {s} y la cabeza del {seq}: cayo un latido entre las dos \
             llamadas, se vuelve a pedir"
        ));
    }
    let camino = r.camino_pendiente.ok_or_else(|| "falta caminoPendiente".to_string())?;
    MerklePath::try_from(&camino).map_err(|e| format!("caminoPendiente: {e:?}"))
}

/// El sobre 2.10: la cabeza tal cual vino, el enunciado de AUTORIZACION y la prueba (D-AW).
pub fn sobre(cabeza: Value, s: &SobrePrenda) -> Value {
    json!({
        "v": 1,
        "tipo": "prenda",
        "cabeza": cabeza,
        "enunciado": {
            "receptor": digest_to_wire(&s.receptor),
            "marca": digest_to_wire(&s.marca),
        },
        "prueba": Blob(s.prueba.clone()),
    })
}

/// `zk-ssl-cli prueba-prenda`: la media prenda que viaja, escrita por su prendador.
#[derive(Args)]
pub struct PruebaPrendaArgs {
    /// URL del nodo VIVO (JSON-RPC): la cabeza firmada, la foto y, con --publicar, la prenda.
    #[arg(long, default_value = "http://127.0.0.1:8545")]
    nodo: String,
    /// El aviso v2 del pendiente que se prenda: el fichero que `simulate --v2 --aviso` escribio.
    #[arg(long)]
    aviso: PathBuf,
    /// El keystore `zkssl-keystore/1` del prendador (D-BC): la clave no sale del SDK.
    #[arg(long)]
    keystore: PathBuf,
    /// El fichero con la frase del keystore (D-BG): nunca en la linea de ordenes.
    #[arg(long)]
    frase_fichero: PathBuf,
    /// El indice de la cuenta del prendador (decimal o `0x`), para pedir su foto.
    #[arg(long)]
    index: String,
    /// Donde escribir el sobre `prenda` (spec/PAQUETE.md, 2.10). Se escribe SIEMPRE (D-BE).
    #[arg(long)]
    salida: PathBuf,
    /// Publicar la marca con `zkssl_pledge` bajo el mismo latido, e imprimir lo que devuelve.
    #[arg(long)]
    publicar: bool,
}

pub fn run(a: PruebaPrendaArgs) -> anyhow::Result<()> {
    let crudo = std::fs::read_to_string(&a.aviso)
        .map_err(|e| anyhow::anyhow!("{}: no se puede leer: {e}", a.aviso.display()))?;
    let aviso: AvisoV2 = serde_json::from_str(&crudo)
        .map_err(|e| anyhow::anyhow!("{}: no es un aviso v2: {e}", a.aviso.display()))?;
    let notice = notice_de(&aviso).map_err(|e| anyhow::anyhow!("{e}"))?;
    let wallet = wallet_de(&a.keystore, &a.frase_fichero)?;
    let index = q_de(&a.index)?;

    let agente = ureq::AgentBuilder::new().timeout(Duration::from_secs(20)).build();
    let cab_v = respuesta(&agente, &a.nodo, "zkssl_signedEpochHead", json!({}))?;
    let (cabeza, cab) = leer_cabeza(&cab_v).map_err(|e| anyhow::anyhow!("{e}"))?;
    let foto_v = respuesta(
        &agente,
        &a.nodo,
        "zkssl_pendingPath",
        json!({
            "index": Q(index), "viewKey": digest_to_wire(&wallet.view_key()),
            "position": aviso.position, "salt": aviso.salt, "amount": aviso.amount, "x": aviso.x,
        }),
    )?;
    let camino = leer_camino(&foto_v, cab.seq).map_err(|e| anyhow::anyhow!("{e}"))?;
    let s = wallet
        .prueba_de_prenda(&cab, &notice, &camino)
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    let salida = a.salida.to_string_lossy().to_string();
    escribir(&salida, &sobre(cabeza, &s))?;
    eprintln!("sobre prenda escrito en {salida}: seq {} ({} B de prueba)", s.seq, s.prueba.len());
    if a.publicar {
        let r = respuesta(
            &agente,
            &a.nodo,
            "zkssl_pledge",
            json!({
                "prueba": Blob(s.prueba.clone()),
                "receptor": digest_to_wire(&s.receptor),
                "marca": digest_to_wire(&s.marca),
                "seq": Q(s.seq),
            }),
        )?;
        let res = r.get("result").cloned().unwrap_or(r);
        println!("{}", serde_json::to_string(&res)?);
        if res.get("accepted") != Some(&Value::Bool(true)) {
            anyhow::bail!("zkssl_pledge no la acepto: {res}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use winterfell::math::fields::f64::BaseElement;

    fn cero() -> String {
        format!("0x{}", "00".repeat(32))
    }

    #[test]
    fn una_respuesta_sin_cabeza_firmada_se_nombra() {
        let e = leer_cabeza(&json!({ "result": { "available": false } })).unwrap_err();
        assert!(e.contains("cabeza firmada") || e.contains("cable"), "{e}");
    }

    #[test]
    fn la_foto_de_otro_latido_se_rechaza() {
        let v = json!({
            "available": true, "s": "0x5", "caminoPendiente": { "siblings": [], "isRight": [] }
        });
        let e = leer_camino(&v, 6).unwrap_err();
        assert!(e.contains("latido 5") && e.contains("del 6"), "{e}");
    }

    #[test]
    fn la_foto_sin_la_meta_se_lee() {
        let sib: Vec<String> = (0..32).map(|_| cero()).collect();
        let v = json!({ "result": {
            "available": true, "s": "0x6",
            "caminoPendiente": { "siblings": sib, "isRight": vec![false; 32] }
        }});
        let c = leer_camino(&v, 6).expect("sin hermanosMeta, emisor ni nacido se lee");
        assert_eq!((c.siblings.len(), c.is_right.len()), (32, 32));
    }

    #[test]
    fn el_sobre_2_10_lleva_la_cabeza_verbatim_y_el_enunciado() {
        let (r, m) = ([BaseElement::new(7); 4], [BaseElement::new(9); 4]);
        let s = SobrePrenda {
            prueba: vec![1, 2, 3],
            receptor: r,
            marca: m,
            seq: 4,
            pending_root: [BaseElement::new(5); 4],
        };
        let cabeza = json!({ "cabeza": "tal cual vino" });
        let v = sobre(cabeza.clone(), &s);
        assert_eq!(v["v"], json!(1));
        assert_eq!(v["tipo"], json!("prenda"));
        assert_eq!(v["cabeza"], cabeza);
        assert_eq!(v["enunciado"]["receptor"], serde_json::to_value(digest_to_wire(&r)).unwrap());
        assert_eq!(v["enunciado"]["marca"], serde_json::to_value(digest_to_wire(&m)).unwrap());
        assert_eq!(v["prueba"], json!("0x010203"));
        assert_eq!(v.as_object().map(|o| o.len()), Some(5), "ni el seq ni la raiz se repiten");
    }

    #[test]
    fn la_frase_se_lee_sin_el_salto_y_vacia_no_vale() {
        let dir = std::env::temp_dir();
        let (f1, f2) = (
            dir.join(format!("m543-frase-{}.txt", std::process::id())),
            dir.join(format!("m543-vacia-{}.txt", std::process::id())),
        );
        std::fs::write(&f1, "una frase de prueba\n").unwrap();
        std::fs::write(&f2, "\n").unwrap();
        assert_eq!(leer_frase(&f1).unwrap(), "una frase de prueba");
        assert!(leer_frase(&f2).is_err(), "una frase vacia no vale");
        let _ = (std::fs::remove_file(&f1), std::fs::remove_file(&f2));
    }

    #[test]
    fn el_keystore_abre_con_su_frase_y_no_con_otra() {
        let w = Wallet::from_elements([0x9E37_79B9_7F4A_7C15, 2, 3, 4]);
        let dir = std::env::temp_dir();
        let id = std::process::id();
        let ks = dir.join(format!("m543-ks-{id}.json"));
        let (buena, otra) =
            (dir.join(format!("m543-buena-{id}.txt")), dir.join(format!("m543-otra-{id}.txt")));
        keystore::save(&ks, &w, "la frase buena").unwrap();
        std::fs::write(&buena, "la frase buena\n").unwrap();
        std::fs::write(&otra, "otra frase\n").unwrap();
        let abierto = wallet_de(&ks, &buena).expect("abre con su frase");
        assert_eq!(abierto.public_id(), w.public_id());
        assert!(wallet_de(&ks, &otra).is_err(), "no abre con otra frase");
        for f in [&ks, &buena, &otra] {
            let _ = std::fs::remove_file(f);
        }
    }

    /// **La puerta de clausura del cli (5.A-359).** El kit tiene la suya
    /// (`la_clausura_del_kit_no_lleva_el_probador`); el cli no tenia ninguna, y este corte le mete
    /// `zk-ssl-sdk`. Camina el `Cargo.lock` desde `zk-ssl-cli` como la del kit y exige que los
    /// crates SIN `source` -los del arbol: el workspace y el fork- sean EXACTAMENTE estos trece, y
    /// que el NODO no este. Los de crates.io los fija el propio lock.
    #[test]
    fn la_clausura_del_cli_son_trece_crates_del_arbol_y_no_el_nodo() {
        let lock = include_str!("../../../Cargo.lock");
        let mut deps: std::collections::BTreeMap<String, Vec<String>> = Default::default();
        let mut del_arbol = std::collections::BTreeSet::new();
        for bloque in lock.split("[[package]]").skip(1) {
            let (mut nombre, mut lista, mut dentro) = (String::new(), Vec::new(), false);
            let mut con_source = false;
            for linea in bloque.lines() {
                let s = linea.trim();
                if let Some(v) = s.strip_prefix("name = ") {
                    nombre = v.trim_matches('"').to_string();
                } else if s.starts_with("source = ") {
                    con_source = true;
                } else if s == "dependencies = [" {
                    dentro = true;
                } else if dentro && s == "]" {
                    dentro = false;
                } else if dentro {
                    let d = s.trim_end_matches(',').trim_matches('"');
                    lista.push(d.split(' ').next().unwrap_or("").to_string());
                }
            }
            if !con_source {
                del_arbol.insert(nombre.clone());
            }
            deps.entry(nombre).or_default().extend(lista);
        }
        let mut vistos = std::collections::BTreeSet::new();
        let mut cola = vec!["zk-ssl-cli".to_string()];
        while let Some(n) = cola.pop() {
            if vistos.insert(n.clone()) {
                cola.extend(deps.get(&n).cloned().unwrap_or_default());
            }
        }
        assert!(!vistos.contains("zk-ssl-node"), "la clausura del cli no puede llevar el nodo");
        let casa: Vec<&str> =
            vistos.iter().filter(|n| del_arbol.contains(*n)).map(|n| n.as_str()).collect();
        assert_eq!(
            casa,
            [
                "settlement-prover", "stark-experiment", "winter-air", "winter-prover",
                "winter-verifier", "zk-ssl", "zk-ssl-air", "zk-ssl-cli", "zk-ssl-guardian",
                "zk-ssl-hash", "zk-ssl-sdk", "zk-ssl-verify", "zk-ssl-wire",
            ],
            "un crate del arbol entra en la clausura del cli o sale de ella: que pase por aqui"
        );
    }
}
