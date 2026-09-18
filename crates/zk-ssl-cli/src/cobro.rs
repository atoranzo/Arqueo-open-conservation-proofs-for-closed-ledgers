//! **RFC-0008 E4, corte A: la BOCA del cobrador** (S497; D-M y D-P).
//!
//! El cobrador tiene su aviso v2 (posicion, sal, importe y el sobre `X` opaco, lo que el pagador
//! le dio) y su credencial (indice y clave de vista, lo que es suyo). Pide a un nodo VIVO la
//! cabeza firmada (`zkssl_signedEpochHead`) y la foto de su pendiente (`zkssl_pendingPath`),
//! exige que las dos sean del MISMO latido (`s == seq`, D-F: entre las dos llamadas puede caer
//! un latido, y entonces se vuelve a pedir), llama al productor de la capa -que no lee libro- y
//! escribe el sobre `cobro_pendiente` de `PAQUETE.md` 2.8 con la cabeza VERBATIM. Reunir, no
//! recomponer: la cabeza se pega tal cual vino y `Blob` es el unico productor de la forma `0x`.
//!
//! Lo PURO -leer los dos ficheros y las dos respuestas, la puerta del latido, componer el sobre-
//! vive en funciones sin red, con sus testigos; la red la ejercita el banco (E4, corte B). Dos
//! medidas viven aqui como tests, y no como prosa: la credencial que el sandbox escribe es la que
//! la capa acepta (D-P), y las dos formas del positivo se sostienen sobre un pendiente real
//! (D-N).

use std::path::PathBuf;
use std::time::Duration;

use clap::Args;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use stark_experiment::merkle::MerklePath;
use zk_ssl::prueba_cobro::{prueba_de_cobro_pendiente, CabezaDePendientes, FotoDelCobro, SobreCobro};
use zk_ssl::two_phase::PendingNotice;
use zk_ssl_wire::{
    digest_from_wire, digest_to_wire, Blob, MerklePathDto, SignedEpochHeadDto, B32, Q,
};

use crate::fmt::Digest;
use crate::witness::{pedir, Servido};

/// El aviso v2 en el fichero del CLIENTE (D-M): las cuatro piezas que `PendingNotice` lleva, en
/// las formas del cable. `x` es obligatorio: un fichero sin `x` es un aviso v1 y no se lee.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AvisoV2 {
    pub position: Q,
    pub salt: B32,
    pub amount: Q,
    pub x: B32,
}

/// La credencial del receptor, en SU fichero (D-P): la terna que `dev_openSeeded` entrega, y
/// que el sandbox deriva de su clave determinista. No es el aviso: dos ficheros, dos duenos.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Credencial {
    pub index: Q,
    pub public_id: B32,
    pub view_key: B32,
}

/// El aviso de un recibo, o por que no: E1 es del compromiso v2.
pub fn aviso_de(n: &PendingNotice) -> Result<AvisoV2, String> {
    let x = n.x.ok_or_else(|| "el aviso es v1 (sin sobre X): E1 es del compromiso v2".to_string())?;
    Ok(AvisoV2 {
        position: Q(n.position),
        salt: digest_to_wire(&n.salt),
        amount: Q(n.amount),
        x: digest_to_wire(&x),
    })
}

/// El camino de vuelta: el `PendingNotice` que el productor de la capa lee.
pub fn notice_de(a: &AvisoV2) -> Result<PendingNotice, String> {
    Ok(PendingNotice {
        position: a.position.0,
        salt: digest_from_wire(&a.salt).map_err(|e| format!("salt: {e:?}"))?,
        amount: a.amount.0,
        x: Some(digest_from_wire(&a.x).map_err(|e| format!("x: {e:?}"))?),
    })
}

/// La credencial del receptor a partir de su clave ancha: la misma derivacion que el nodo usa en
/// `dev_openSeeded` y la capa cruza en `account_view_authenticated`.
pub fn credencial_de(clave: Digest, index: u64) -> Credencial {
    Credencial {
        index: Q(index),
        public_id: digest_to_wire(&stark_experiment::native::derive_public_id_wide(clave)),
        view_key: digest_to_wire(&stark_experiment::native::derive_view_key_wide(clave)),
    }
}

/// Escribe un fichero del cliente: JSON con sangria y un salto final, como el modo del nodo.
pub fn escribir<T: Serialize>(ruta: &str, v: &T) -> anyhow::Result<()> {
    std::fs::write(ruta, format!("{}\n", serde_json::to_string_pretty(v)?))
        .map_err(|e| anyhow::anyhow!("{ruta}: no se puede escribir: {e}"))
}

/// Del `result` (o de la respuesta entera) de `zkssl_signedEpochHead`: la cabeza VERBATIM y lo
/// que el productor necesita de ella. Exige v5, la unica que firma `pmetaRoot`.
pub fn leer_cabeza(v: &Value) -> Result<(Value, CabezaDePendientes), String> {
    let obj = v.get("result").cloned().unwrap_or_else(|| v.clone());
    let dto: SignedEpochHeadDto = serde_json::from_value(obj.clone())
        .map_err(|e| format!("la cabeza no es una respuesta del cable: {e}"))?;
    let vista = dto
        .firmada()
        .map_err(|e| format!("cabeza malformada: {e}"))?
        .ok_or_else(|| "la respuesta no lleva cabeza firmada (available: false)".to_string())?;
    if vista.format_version.0 != 5 {
        return Err(format!(
            "formatVersion {}: el cobro pendiente exige una cabeza v5, la unica que firma \
             pmetaRoot",
            vista.format_version.0
        ));
    }
    let pmeta = vista.pmeta_root.ok_or_else(|| "cabeza v5 sin pmetaRoot".to_string())?;
    let cab = CabezaDePendientes {
        seq: vista.seq.0,
        pending_root: digest_from_wire(&vista.pending_root)
            .map_err(|e| format!("pendingRoot: {e:?}"))?,
        pmeta_root: digest_from_wire(&pmeta).map_err(|e| format!("pmetaRoot: {e:?}"))?,
    };
    Ok((obj, cab))
}

/// Del `result` de `zkssl_pendingPath`: la foto, SOLO si es del latido de la cabeza (`s == seq`).
pub fn leer_foto(v: &Value, seq: u64) -> Result<FotoDelCobro, String> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct R {
        available: bool,
        reason: Option<String>,
        s: Option<Q>,
        camino_pendiente: Option<MerklePathDto>,
        hermanos_meta: Option<Vec<B32>>,
        emisor: Option<Q>,
        nacido: Option<Q>,
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
    let camino_pendiente =
        MerklePath::try_from(&camino).map_err(|e| format!("caminoPendiente: {e:?}"))?;
    let hermanos_meta = r
        .hermanos_meta
        .ok_or_else(|| "falta hermanosMeta".to_string())?
        .iter()
        .map(|b| digest_from_wire(b).map_err(|e| format!("hermanosMeta: {e:?}")))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(FotoDelCobro {
        camino_pendiente,
        hermanos_meta,
        emisor: r.emisor.ok_or_else(|| "falta emisor".to_string())?.0,
        nacido: r.nacido.ok_or_else(|| "falta nacido".to_string())?.0,
    })
}

/// El sobre 2.8: la cabeza tal cual vino, el enunciado de ESTADO y la prueba. `seq` y las dos
/// raices NO se repiten: salen solo de la cabeza (D-J).
pub fn sobre(cabeza: Value, s: &SobreCobro) -> Value {
    json!({
        "v": 1,
        "tipo": "cobro_pendiente",
        "cabeza": cabeza,
        "enunciado": {
            "receptor": digest_to_wire(&s.receptor),
            "nacido": Q(s.nacido),
            "inferior": Q(s.inferior),
        },
        "prueba": Blob(s.prueba.clone()),
    })
}

fn q_de(s: &str) -> anyhow::Result<u64> {
    match s.strip_prefix("0x") {
        Some(h) => u64::from_str_radix(h, 16).map_err(|e| anyhow::anyhow!("{s}: {e}")),
        None => s.parse::<u64>().map_err(|e| anyhow::anyhow!("{s}: {e}")),
    }
}

fn b32_de(s: &str) -> anyhow::Result<B32> {
    serde_json::from_value(Value::String(s.to_string())).map_err(|e| anyhow::anyhow!("{s}: {e}"))
}

fn cuerpo(metodo: &str, params: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": 1, "method": metodo, "params": params })
}

fn respuesta(
    agente: &ureq::Agent,
    url: &str,
    metodo: &str,
    params: Value,
) -> anyhow::Result<Value> {
    match pedir(agente, url, cuerpo(metodo, params)) {
        Servido::Respuesta(v) => Ok(v),
        Servido::SinRespuesta { motivo } => anyhow::bail!("{metodo}: {motivo}"),
    }
}

/// `zk-ssl-cli prueba-cobro`: la prueba portable del cobro pendiente, escrita por su cobrador.
#[derive(Args)]
pub struct PruebaCobroArgs {
    /// URL del nodo VIVO (JSON-RPC): sirve la cabeza firmada y la foto del ultimo latido.
    #[arg(long, default_value = "http://127.0.0.1:8545")]
    nodo: String,
    /// El aviso v2 del pagador: el fichero que `simulate --v2 --aviso` escribio.
    #[arg(long)]
    aviso: PathBuf,
    /// El indice de la cuenta del cobrador (decimal o `0x`), de SU credencial.
    #[arg(long)]
    index: String,
    /// La identidad publica del cobrador (`0x` + 64 hex), el `receptor` del enunciado.
    #[arg(long)]
    receptor: String,
    /// La clave de vista del cobrador (`0x` + 64 hex): autoriza la foto, no autoriza a gastar.
    #[arg(long)]
    view_key: String,
    /// La cota inferior del enunciado, <<al menos `inferior`>> (D-N: 0 es la existencia).
    #[arg(long)]
    inferior: u64,
    /// Donde escribir el sobre `cobro_pendiente` (spec/PAQUETE.md, 2.8).
    #[arg(long)]
    salida: PathBuf,
}

pub fn run(a: PruebaCobroArgs) -> anyhow::Result<()> {
    let crudo = std::fs::read_to_string(&a.aviso)
        .map_err(|e| anyhow::anyhow!("{}: no se puede leer: {e}", a.aviso.display()))?;
    let aviso: AvisoV2 = serde_json::from_str(&crudo)
        .map_err(|e| anyhow::anyhow!("{}: no es un aviso v2: {e}", a.aviso.display()))?;
    let notice = notice_de(&aviso).map_err(|e| anyhow::anyhow!("{e}"))?;
    let index = q_de(&a.index)?;
    let vk = b32_de(&a.view_key)?;
    let receptor = digest_from_wire(&b32_de(&a.receptor)?)
        .map_err(|e| anyhow::anyhow!("receptor: {e:?}"))?;

    let agente = ureq::AgentBuilder::new().timeout(Duration::from_secs(20)).build();
    let cab_v = respuesta(&agente, &a.nodo, "zkssl_signedEpochHead", json!({}))?;
    let (cabeza, cab) = leer_cabeza(&cab_v).map_err(|e| anyhow::anyhow!("{e}"))?;
    let foto_v = respuesta(
        &agente,
        &a.nodo,
        "zkssl_pendingPath",
        json!({
            "index": Q(index), "viewKey": vk,
            "position": aviso.position, "salt": aviso.salt, "amount": aviso.amount, "x": aviso.x,
        }),
    )?;
    let foto = leer_foto(&foto_v, cab.seq).map_err(|e| anyhow::anyhow!("{e}"))?;
    let s = prueba_de_cobro_pendiente(&cab, receptor, &notice, &foto, a.inferior)
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    let salida = a.salida.to_string_lossy().to_string();
    escribir(&salida, &sobre(cabeza, &s))?;
    eprintln!(
        "sobre cobro_pendiente escrito en {salida}: seq {}, nacido {}, al menos {} \
         ({} B de prueba)",
        s.seq, s.nacido, s.inferior, s.prueba.len()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sandbox;
    use zk_ssl::tests_support as ts;

    const IMPORTE: u64 = 250_000;

    fn cero() -> String {
        format!("0x{}", "00".repeat(32))
    }

    #[test]
    fn un_aviso_v1_no_se_escribe() {
        let n = PendingNotice { position: 3, salt: ts::salt_de(7), amount: IMPORTE, x: None };
        let e = aviso_de(&n).unwrap_err();
        assert!(e.contains("v1"), "{e}");
    }

    #[test]
    fn un_aviso_sin_x_no_se_lee() {
        let crudo = format!(r#"{{"position":"0x3","salt":"{}","amount":"0x3d090"}}"#, cero());
        let r: Result<AvisoV2, _> = serde_json::from_str(&crudo);
        assert!(r.is_err(), "un aviso sin x es v1 y no se lee");
    }

    #[test]
    fn la_foto_de_otro_latido_se_rechaza() {
        let v = json!({
            "available": true, "s": "0x5", "caminoPendiente": { "siblings": [], "isRight": [] },
            "hermanosMeta": [], "emisor": "0x0", "nacido": "0x1"
        });
        let e = leer_foto(&v, 6).unwrap_err();
        assert!(e.contains("latido 5") && e.contains("cabeza del 6"), "{e}");
    }

    #[test]
    fn una_foto_no_disponible_dice_su_razon() {
        let v = json!({ "available": false, "reason": "aun no ha habido latido" });
        let e = leer_foto(&v, 1).unwrap_err();
        assert!(e.contains("aun no ha habido latido"), "{e}");
    }

    #[test]
    fn una_respuesta_sin_cabeza_firmada_se_nombra() {
        let e = leer_cabeza(&json!({ "result": { "available": false } })).unwrap_err();
        assert!(e.contains("cabeza firmada") || e.contains("cable"), "{e}");
    }

    /// D-P, MEDIDA: la credencial que el sandbox deriva es la que la capa acepta para esa cuenta.
    #[test]
    fn la_credencial_del_sandbox_es_la_que_la_capa_acepta() {
        let mut l = sandbox::open_layer(None, sandbox::Params::default()).expect("capa");
        let mut tr = crate::trace::make_tracer(true);
        let k = sandbox::key_of(0xA11CE, 1);
        let idx = sandbox::open_funded(&mut l, k, 0, tr.as_mut()).expect("abrir");
        let c = credencial_de(k, idx);
        let vk = digest_from_wire(&c.view_key).expect("clave de vista");
        println!("D-P| cuenta {idx}: la capa acepta la clave de vista derivada");
        assert!(l.account_view_authenticated(idx, vk).is_some(), "D-P DESMENTIDA");
        assert_eq!(digest_from_wire(&c.public_id).expect("id"), l.public_id_of(idx).expect("id"));
        let otra = credencial_de(sandbox::key_of(0xA11CE, 2), idx).view_key;
        let otra = digest_from_wire(&otra).unwrap();
        assert!(l.account_view_authenticated(idx, otra).is_none(), "otra clave no vale");
    }

    /// D-N, MEDIDA de punta a punta SIN nodo: un pendiente v2 real, el aviso por fichero, la foto
    /// de la capa, y las DOS formas del positivo (0 y el importe) se prueban y se enlazan.
    #[test]
    fn las_dos_formas_de_d_n_se_sostienen_sobre_un_pendiente_real() {
        let mut l = sandbox::open_layer(None, sandbox::Params::default()).expect("capa");
        let mut tr = crate::trace::make_tracer(true);
        let seed = 0xA11CE;
        let a0 = sandbox::open_funded(&mut l, sandbox::key_of(seed, 0), 1_000_000, tr.as_mut())
            .unwrap();
        let a1 = sandbox::open_funded(&mut l, sandbox::key_of(seed, 1), 0, tr.as_mut()).unwrap();
        let f = l.public_id_of(a0).expect("a0");
        let envio = sandbox::run_send_v2(
            &mut l, a0, sandbox::key_of(seed, 0), a1, IMPORTE, 7, f, 96, tr.as_mut(),
        )
        .expect("envio v2");
        let av = aviso_de(&envio.notice).expect("aviso v2");
        let ida: AvisoV2 = serde_json::from_str(&serde_json::to_string(&av).unwrap()).unwrap();
        assert_eq!(ida, av, "ida y vuelta del aviso");
        let notice = notice_de(&ida).expect("notice");
        assert_eq!(notice.x, envio.notice.x);
        let foto = l.foto_pendientes();
        let cab = CabezaDePendientes {
            seq: l.transition_log().entries().len() as u64,
            pending_root: foto.raiz_pendientes(),
            pmeta_root: foto.raiz_meta(),
        };
        let id_bob = l.public_id_of(a1).expect("bob");
        let fc = foto.cobro(id_bob, &notice).expect("la foto sirve el pendiente del aviso");
        for inferior in [0u64, IMPORTE] {
            let s =
                prueba_de_cobro_pendiente(&cab, id_bob, &notice, &fc, inferior).expect("prueba");
            println!("D-N| inferior {inferior}: seq {} nacido {} enlazado", s.seq, s.nacido);
            assert_eq!((s.seq, s.inferior, s.receptor), (cab.seq, inferior, id_bob));
            assert!(s.nacido < s.seq);
            let v = sobre(json!({ "seq": Q(cab.seq) }), &s);
            let claves: Vec<&str> = v.as_object().unwrap().keys().map(|k| k.as_str()).collect();
            assert_eq!(claves, ["cabeza", "enunciado", "prueba", "tipo", "v"]);
            let seq_hex = json!(format!("{:#x}", cab.seq));
            assert_eq!(v["cabeza"]["seq"], seq_hex, "la cabeza va verbatim");
        }
        let e = prueba_de_cobro_pendiente(&cab, id_bob, &notice, &fc, IMPORTE + 1).unwrap_err();
        assert!(format!("{e:?}").contains("NO se sostiene"), "{e:?}");
    }
}
