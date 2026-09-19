//! **RFC-0008 E2, corte final: la BOCA del pagador** (S507; D-AI).
//!
//! El espejo del S497 por el otro lado del mismo pendiente. El cobrador tiene su aviso v2 y su
//! credencial; el PAGADOR tiene el MISMO aviso, su propia credencial y una cosa que nadie mas
//! tiene: la pareja `(refund_id, delta)` con la que compuso el sobre `X`, y que el receptor
//! recibe OPACA (RFC-0003). Esa pareja es el RETORNO y vive en SU fichero: tercer fichero,
//! tercer dueno, como la D-P hizo con la credencial.
//!
//! **La puerta barata, y por que va antes de la red.** `refund_envelope` es el UNICO productor
//! de `X`, asi que un retorno que no sea el de ESE aviso se caza recomponiendo su `x`: sin
//! pedirle nada al nodo y sin gastar una prueba. Medido en el PASTE-E2g-M (M2). Lo que NO
//! prueba, y va dicho donde se lee: `refund_envelope` mete `delta` en Goldilocks, asi que dos
//! deltas congruentes dan el MISMO sobre (D-AG). Descarta lo evidente; no fija el delta.
//!
//! **Lo que pide al nodo, y con que.** La cabeza firmada y la foto de SU pendiente, y la foto
//! con `receiverId` (D-AE, S505): la credencial que viaja es la del PAGADOR y el receptor va
//! nombrado, al reves que en el cobro. Un nodo anterior al S505 IGNORA ese campo y sirve la
//! nada; lo que dice si un nodo sabe de que habla es su `spec/openrpc.json`.
//!
//! **`--t` es ABSOLUTO** (D-AI-4): es la epoca hasta la que se afirma que el pago no revierte, y
//! es lo que el enunciado publica. Relativo haria que la misma orden diera sobres distintos
//! segun cuando se corre.
//!
//! Reunir, no recomponer: la cabeza se pega tal cual vino, y `seq` y las dos raices salen SOLO
//! de ella (D-J). Lo PURO -leer los tres ficheros y las dos respuestas, la puerta del retorno,
//! componer el sobre- vive en funciones sin red, con sus testigos; la red la ejercita el banco,
//! que es de otro corte (D-AJ).

use std::path::PathBuf;
use std::time::Duration;

use clap::Args;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use zk_ssl::pending::refund_envelope;
use zk_ssl::prueba_pago::{prueba_de_pago_en_curso, AperturaDelPago, SobrePago};
use zk_ssl_wire::{digest_from_wire, digest_to_wire, Blob, B32, Q};

use crate::cobro::{b32_de, escribir, leer_cabeza, leer_foto, notice_de, q_de, respuesta, AvisoV2};
use crate::fmt::Digest;

/// El retorno del pagador, en SU fichero (D-AI): la pareja del sobre, que es lo unico del pago
/// que no esta ni en el aviso ni en la credencial. No es el aviso: tres ficheros, tres duenos.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Retorno {
    pub refund_id: B32,
    pub delta: Q,
}

/// El retorno tal como lo escribe quien envia: la MISMA pareja que fue a `send_materials_v2`.
pub fn retorno_de(refund_id: Digest, delta: u64) -> Retorno {
    Retorno { refund_id: digest_to_wire(&refund_id), delta: Q(delta) }
}

/// **La apertura del pagador, o por que ese retorno no es de ese aviso.**
///
/// Falla cerrada ANTES de la red: si la pareja no recompone el `x` del aviso, no hay nada que
/// probar. El texto no dice cual de las dos mitades falla, porque quien lo lee tiene las dos.
pub fn apertura_de(a: &AvisoV2, r: &Retorno) -> Result<AperturaDelPago, String> {
    let n = notice_de(a)?;
    let refund_id = digest_from_wire(&r.refund_id).map_err(|e| format!("refundId: {e:?}"))?;
    if digest_to_wire(&refund_envelope(refund_id, r.delta.0)) != a.x {
        return Err("el retorno no es el de este aviso: (refundId, delta) no recompone su x; \
                    el sobre de un aviso solo lo abre quien lo compuso"
            .to_string());
    }
    Ok(AperturaDelPago {
        posicion: n.position,
        sal: n.salt,
        importe: n.amount,
        refund_id,
        delta: r.delta.0,
    })
}

/// El sobre 2.9: la cabeza tal cual vino y el enunciado de ESTADO. El importe SI sale, y exacto
/// (D-AD); la sal, la pareja y el emisor no. `seq` y las dos raices, solo en la cabeza (D-J).
pub fn sobre(cabeza: Value, s: &SobrePago) -> Value {
    json!({
        "v": 1,
        "tipo": "pago_en_curso",
        "cabeza": cabeza,
        "enunciado": {
            "receptor": digest_to_wire(&s.receptor),
            "importe": Q(s.importe),
            "t": Q(s.t),
            "nacido": Q(s.nacido),
        },
        "prueba": Blob(s.prueba.clone()),
    })
}

fn leer_json<T: for<'a> Deserialize<'a>>(ruta: &PathBuf, que: &str) -> anyhow::Result<T> {
    let crudo = std::fs::read_to_string(ruta)
        .map_err(|e| anyhow::anyhow!("{}: no se puede leer: {e}", ruta.display()))?;
    serde_json::from_str(&crudo)
        .map_err(|e| anyhow::anyhow!("{}: no es {que}: {e}", ruta.display()))
}

/// `zk-ssl-cli prueba-pago`: la prueba portable del pago en curso, escrita por su pagador.
#[derive(Args)]
pub struct PruebaPagoArgs {
    /// URL del nodo VIVO (JSON-RPC): sirve la cabeza firmada y la foto del ultimo latido.
    /// Tiene que ser del S505 o posterior, o servira la nada.
    #[arg(long, default_value = "http://127.0.0.1:8545")]
    nodo: String,
    /// El aviso v2 de ESTE pago: el mismo fichero que `simulate --v2 --aviso` escribio.
    #[arg(long)]
    aviso: PathBuf,
    /// El retorno del pagador (`refundId`, `delta`): el fichero de `simulate --v2 --retorno`.
    #[arg(long)]
    retorno: PathBuf,
    /// El indice de la cuenta del PAGADOR (decimal o `0x`), de SU credencial.
    #[arg(long)]
    index: String,
    /// La identidad publica del RECEPTOR (`0x` + 64 hex): el `receptor` del enunciado, y el
    /// `receiverId` con el que el nodo sabe que quien pide es el pagador.
    #[arg(long)]
    receptor: String,
    /// La clave de vista del PAGADOR (`0x` + 64 hex): autoriza la foto, no autoriza a gastar.
    #[arg(long)]
    view_key: String,
    /// La epoca, ABSOLUTA, hasta la que se afirma que el pago no revierte (D-AI-4).
    #[arg(long)]
    t: u64,
    /// Donde escribir el sobre `pago_en_curso` (spec/PAQUETE.md, 2.9).
    #[arg(long)]
    salida: PathBuf,
}

pub fn run(a: PruebaPagoArgs) -> anyhow::Result<()> {
    let aviso: AvisoV2 = leer_json(&a.aviso, "un aviso v2")?;
    let retorno: Retorno = leer_json(&a.retorno, "un retorno del pagador")?;
    let apertura = apertura_de(&aviso, &retorno).map_err(|e| anyhow::anyhow!("{e}"))?;
    let index = q_de(&a.index)?;
    let vk = b32_de(&a.view_key)?;
    let receptor_wire = b32_de(&a.receptor)?;
    let receptor =
        digest_from_wire(&receptor_wire).map_err(|e| anyhow::anyhow!("receptor: {e:?}"))?;

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
            "receiverId": receptor_wire,
        }),
    )?;
    let foto = leer_foto(&foto_v, cab.seq).map_err(|e| anyhow::anyhow!("{e}"))?;
    let s = prueba_de_pago_en_curso(&cab, receptor, &apertura, &foto, a.t)
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    let salida = a.salida.to_string_lossy().to_string();
    escribir(&salida, &sobre(cabeza, &s))?;
    eprintln!(
        "sobre pago_en_curso escrito en {salida}: seq {}, nacido {}, importe {} hasta t {} \
         ({} B de prueba)",
        s.seq, s.nacido, s.importe, s.t, s.prueba.len()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cobro::{aviso_de, credencial_de};
    use crate::sandbox;
    use zk_ssl::prueba_cobro::CabezaDePendientes;
    use zk_ssl::tests_support as ts;

    const IMPORTE: u64 = 250_000;
    const DELTA: u64 = 96;
    const SEMILLA: u64 = 0xA11CE;

    /// Un libro vivo con un pendiente v2 sin cobrar, y lo que sus dos duenos se llevan.
    fn escena() -> (zk_ssl::SovereignLayer, u64, u64, AvisoV2, Retorno) {
        let mut l = sandbox::open_layer(None, sandbox::Params::default()).expect("capa");
        let mut tr = crate::trace::make_tracer(true);
        let a0 = sandbox::open_funded(&mut l, sandbox::key_of(SEMILLA, 0), 1_000_000, tr.as_mut())
            .expect("pagador");
        let a1 = sandbox::open_funded(&mut l, sandbox::key_of(SEMILLA, 1), 0, tr.as_mut())
            .expect("receptor");
        let f = l.public_id_of(a0).expect("id del pagador");
        let envio = sandbox::run_send_v2(
            &mut l, a0, sandbox::key_of(SEMILLA, 0), a1, IMPORTE, 7, f, DELTA, tr.as_mut(),
        )
        .expect("envio v2");
        let av = aviso_de(&envio.notice).expect("aviso v2");
        (l, a0, a1, av, retorno_de(f, DELTA))
    }

    #[test]
    fn un_retorno_con_campos_de_mas_no_se_lee() {
        let crudo = format!(
            r#"{{"refundId":"0x{}","delta":"0x60","sal":"0x1"}}"#,
            "00".repeat(32)
        );
        let r: Result<Retorno, _> = serde_json::from_str(&crudo);
        assert!(r.is_err(), "un retorno con un campo de mas no se lee");
    }

    #[test]
    fn el_retorno_va_y_vuelve_por_el_fichero() {
        let r = retorno_de(ts::salt_de(3), DELTA);
        let ida: Retorno = serde_json::from_str(&serde_json::to_string(&r).unwrap()).unwrap();
        assert_eq!(ida, r, "ida y vuelta del retorno");
        let crudo = serde_json::to_string(&r).unwrap();
        assert!(crudo.contains("refundId"), "el fichero del retorno es camelCase: {crudo}");
    }

    /// El sobre 2.9 y nada mas que el 2.9: cinco claves, la cabeza verbatim y los cuatro del
    /// enunciado. `seq` y las raices NO se repiten fuera de la cabeza (D-J).
    #[test]
    fn el_sobre_es_la_forma_2_9() {
        let s = SobrePago {
            prueba: vec![1, 2, 3],
            receptor: ts::salt_de(11),
            importe: IMPORTE,
            t: 99,
            nacido: 3,
            seq: 4,
            pending_root: ts::salt_de(12),
            pmeta_root: ts::salt_de(13),
        };
        let v = sobre(json!({ "seq": Q(s.seq) }), &s);
        let claves: Vec<&str> = v.as_object().unwrap().keys().map(|k| k.as_str()).collect();
        assert_eq!(claves, ["cabeza", "enunciado", "prueba", "tipo", "v"]);
        assert_eq!(v["tipo"], json!("pago_en_curso"));
        let en: Vec<&str> =
            v["enunciado"].as_object().unwrap().keys().map(|k| k.as_str()).collect();
        assert_eq!(en, ["importe", "nacido", "receptor", "t"]);
        assert_eq!(v["cabeza"]["seq"], json!(format!("{:#x}", s.seq)), "la cabeza va verbatim");
        assert_eq!(v["enunciado"]["importe"], json!(format!("{:#x}", IMPORTE)));
    }

    /// D-AI-3, MEDIDA: el retorno del pagador abre SU aviso, y uno que no es el suyo muere
    /// antes de que nadie toque el probador.
    #[test]
    fn la_puerta_del_retorno_caza_al_que_no_es() {
        let (_l, _a0, _a1, av, r) = escena();
        let ap = apertura_de(&av, &r).expect("el retorno abre su aviso");
        assert_eq!((ap.importe, ap.delta), (IMPORTE, DELTA));
        assert_eq!(ap.posicion, av.position.0);
        let otro_delta = Retorno { delta: Q(DELTA + 1), ..r };
        let e = apertura_de(&av, &otro_delta).unwrap_err();
        assert!(e.contains("no recompone su x"), "{e}");
        let otro_id = Retorno { refund_id: digest_to_wire(&ts::salt_de(99)), ..r };
        let e = apertura_de(&av, &otro_id).unwrap_err();
        assert!(e.contains("no recompone su x"), "{e}");
    }

    /// D-AI, MEDIDA: la credencial del PAGADOR sale de la MISMA funcion que la del receptor
    /// (S497) y la capa la acepta para SU indice. Para el receptor estaba medido; para el
    /// pagador estaba RAZONADO hasta el PASTE-E2g-M.
    #[test]
    fn la_credencial_del_pagador_es_la_que_la_capa_acepta() {
        let (l, a0, _a1, _av, _r) = escena();
        let c = credencial_de(sandbox::key_of(SEMILLA, 0), a0);
        let vk = digest_from_wire(&c.view_key).expect("clave de vista");
        assert!(l.account_view_authenticated(a0, vk).is_some(), "D-AI DESMENTIDA");
        assert_eq!(digest_from_wire(&c.public_id).expect("id"), l.public_id_of(a0).expect("id"));
        let ajena = digest_from_wire(&credencial_de(sandbox::key_of(SEMILLA, 1), a0).view_key)
            .expect("ajena");
        assert!(l.account_view_authenticated(a0, ajena).is_none(), "otra clave no vale");
    }

    /// De punta a punta SIN nodo: los tres ficheros del pagador, la foto de la capa, el sobre
    /// 2.9 sobre un pendiente REAL, y los falsadores del plazo y de la pareja.
    #[test]
    fn de_punta_a_punta_sin_nodo_el_pagador_prueba_su_pago() {
        let (l, _a0, a1, av, r) = escena();
        let ap = apertura_de(&av, &r).expect("apertura");
        let notice = notice_de(&av).expect("notice");
        let foto = l.foto_pendientes();
        let receptor = l.public_id_of(a1).expect("id del receptor");
        let fc = foto.cobro(receptor, &notice).expect("la foto sirve el pendiente del aviso");
        let cab = CabezaDePendientes {
            seq: l.transition_log().entries().len() as u64,
            pending_root: foto.raiz_pendientes(),
            pmeta_root: foto.raiz_meta(),
        };
        let t = fc.nacido + DELTA;
        let s = prueba_de_pago_en_curso(&cab, receptor, &ap, &fc, t).expect("prueba del pago");
        println!("S507| prueba {} B, importe {}, t {}, nacido {}", s.prueba.len(), s.importe,
                 s.t, s.nacido);
        assert_eq!((s.importe, s.t, s.receptor, s.seq), (IMPORTE, t, receptor, cab.seq));
        let v = sobre(json!({ "seq": Q(cab.seq) }), &s);
        assert_eq!(v["enunciado"]["t"], json!(format!("{t:#x}")));
        let e = prueba_de_pago_en_curso(&cab, receptor, &ap, &fc, t + 1).unwrap_err();
        assert!(format!("{e:?}").contains("NO se sostiene"), "{e:?}");
        let pareja_mentida = AperturaDelPago { delta: DELTA + 1, ..ap };
        let e = prueba_de_pago_en_curso(&cab, receptor, &pareja_mentida, &fc, fc.nacido)
            .unwrap_err();
        assert!(format!("{e:?}").contains("no sube a la raiz"), "{e:?}");
    }
}
