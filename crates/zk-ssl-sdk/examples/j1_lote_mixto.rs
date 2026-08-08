//! **Banco J.1 — ¿admite `applyMany` un lote MIXTO?**
//!
//! ## Por qué importa, y no es una curiosidad
//!
//! §230 midió que **lo que serializa es la raíz**: un recibo solo vale
//! contra la raíz exacta contra la que se probó, así que **aplica un lote
//! por raíz** y los demás mueren.
//!
//! §231 destapó una mitigación de observación: repartir **envíos y cobros
//! entre agregadores distintos**, porque ninguno de los dos ve la arista
//! por sí solo —hace falta correlacionar por `notice.position`—.
//!
//! Las dos cosas chocan en un punto que **nadie ha comprobado**: si el
//! lote mixto NO se admite, repartir observación obliga a **dos peticiones
//! y por tanto dos turnos**, y el reparto sale más caro. Si SÍ se admite,
//! un solo agregador podría aplicar ambas mitades de una vez —y entonces
//! el reparto de observación cuesta un turno extra a propósito, que es una
//! decisión distinta.
//!
//! ## Lo que dice la lectura, y por qué no basta
//!
//! `apply_many` comprueba dos cosas y **ninguna mira la clase**:
//!
//! ```text
//! if !cuentas.insert(idx)     { DuplicateAccountInBatch }
//! if !posiciones.insert(pos)  { DuplicatePendingInBatch }
//! ```
//!
//! Un cobro consume una posición **vieja**; un envío crea una **nueva**.
//! No deberían chocar. Pero en esta serie la lectura ha fallado nueve
//! veces, así que se mide.
//!
//! ## Hipótesis, escritas ANTES del dato
//!
//! - **H1** · un lote de `n` envíos + `m` cobros **aplica**, con
//!   `applied.len() == n + m`.
//! - **H2** · un lote donde un cobro consume el pendiente que crea un
//!   envío **del mismo lote** se rechaza con `DuplicatePendingInBatch`
//!   —las dos operaciones citan la misma posición—.
//! - **H3** · un lote con la misma cuenta dos veces se rechaza con
//!   `DuplicateAccountInBatch`.
//! - **H4** · ⚠️ si H1 falla, la mitigación de §231 cuesta un turno extra
//!   **por diseño**, no por elección, y eso cambia la decisión de mesa.
//!
//! ## Cómo se usa
//!
//! ```text
//! cargo run --release -p zk-ssl-node -- --dev --listen 127.0.0.1:8649 &
//! cargo run --release -p zk-ssl-sdk --example j1_lote_mixto -- http://127.0.0.1:8649 4
//! ```
//! (url, pares)
//!
//! ⚠️ **Muta el estado del nodo.** Usar uno desechable.

use serde_json::{json, Value};
use winterfell::math::fields::f64::BaseElement;

use zk_ssl::client::{self, ClaimMaterials, SendMaterials};
use zk_ssl::commitment::ClientState;
use zk_ssl::proof_options;
use zk_ssl::two_phase::PendingNotice;
use zk_ssl_sdk::{Account, Rpc, Wallet};
use zk_ssl_wire as wire;
use zk_ssl_wire::{digest_to_wire, Q};

type Digest = [BaseElement; 4];

struct Par {
    sa: [u64; 4],
    ia: u64,
    sb: [u64; 4],
    ib: u64,
    id_b: Digest,
}

fn estado(a: &Account) -> anyhow::Result<ClientState> {
    let v = a.view()?;
    Ok(ClientState { public_id: v.public_id, balance: v.balance, nonce: v.nonce })
}

fn sal(n: u64) -> Digest {
    [BaseElement::new(n), BaseElement::new(0), BaseElement::new(0), BaseElement::new(0)]
}

/// Materiales + prueba de un envío. Devuelve la operación lista para el lote.
fn envio(rpc: &Rpc, p: &Par, salt: u64) -> anyhow::Result<(Value, PendingNotice)> {
    let cuenta = Account::attach(rpc, Wallet::from_elements(p.sa), p.ia);
    let est = estado(&cuenta)?;
    let m_dto: wire::SendMaterialsDto = rpc.call(
        "zkssl_sendMaterials",
        json!({
            "sender": Q(p.ia),
            "receiverId": digest_to_wire(&p.id_b),
            "amount": Q(1_000u64),
            "salt": digest_to_wire(&sal(salt)),
        }),
    )?;
    let m: SendMaterials = (&m_dto).try_into()?;
    let rec = client::prove_send(&m, p.sa.map(BaseElement::new), proof_options())?;
    let aviso = rec.notice.clone();
    Ok((
        json!({
            "kind": "send",
            "receipt": wire::SendReceiptDto::from(&rec),
            "sender": Q(p.ia),
            "senderState": wire::ClientStateDto::from(&est),
            "amount": Q(1_000u64),
        }),
        aviso,
    ))
}

/// Materiales + prueba de un cobro sobre un aviso ya aplicado.
fn cobro(rpc: &Rpc, p: &Par, aviso: &PendingNotice) -> anyhow::Result<Value> {
    let cuenta = Account::attach(rpc, Wallet::from_elements(p.sb), p.ib);
    let est = estado(&cuenta)?;
    let m_dto: wire::ClaimMaterialsDto = rpc.call(
        "zkssl_claimMaterials",
        json!({ "receiver": Q(p.ib), "notice": wire::PendingNoticeDto::from(aviso) }),
    )?;
    let m: ClaimMaterials = (&m_dto).try_into()?;
    let rec = client::prove_claim(&m, p.sb.map(BaseElement::new), proof_options())?;
    Ok(json!({
        "kind": "claim",
        "receipt": wire::ClaimReceiptDto::from(&rec),
        "receiver": Q(p.ib),
        "receiverState": wire::ClientStateDto::from(&est),
        "notice": wire::PendingNoticeDto::from(aviso),
    }))
}

fn aplicar(rpc: &Rpc, ops: Vec<Value>) -> (bool, String, usize) {
    let r: anyhow::Result<Value> = rpc.call("zkssl_applyMany", json!({ "ops": ops }));
    match r {
        Ok(v) => (true, String::new(), v["applied"].as_array().map(|a| a.len()).unwrap_or(0)),
        Err(e) => (false, format!("{e}"), 0),
    }
}

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let url = args.next().unwrap_or_else(|| "http://127.0.0.1:8649".into());
    let n: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(4);

    eprintln!("== BANCO J.1 · el lote MIXTO ==");
    eprintln!("   nodo: {url} · {n} pares");
    eprintln!("   apply_many solo exige cuentas distintas y posiciones distintas,");
    eprintln!("   y NINGUNA de las dos mira la clase. Se predice que el mixto SI.");
    eprintln!("   ⚠️ si NO, la mitigacion de §231 cuesta un turno extra POR DISEÑO\n");

    let rpc = Rpc::new(url.clone());
    let v: Value = rpc.call("zkssl_protocolVersion", json!([]))?;
    eprintln!("   protocolo: {v}");

    eprintln!("-- montaje: {n} pares --");
    let mut pares = Vec::new();
    for i in 0..n {
        let sa: [u64; 4] = [0xE1A0 + i as u64 * 11, 1, 2, 3];
        let sb: [u64; 4] = [0xF1B0 + i as u64 * 11, 4, 5, 6];
        let wa = Wallet::from_elements(sa);
        let wb = Wallet::from_elements(sb);
        let a = Account::open(&rpc, wa)?;
        let b = Account::open(&rpc, wb)?;
        rpc.dev_fund(a.index, 500_000)?;
        pares.push(Par { sa, ia: a.index, sb, ib: b.index, id_b: wb.public_id() });
    }
    eprintln!("   {} cuentas\n", n * 2);

    // ── Ronda 0: envios, para tener pendientes que cobrar ──────────
    eprintln!("-- ronda 0: {n} envios, para crear pendientes --");
    let mut ops = Vec::new();
    let mut avisos = Vec::new();
    for p in &pares {
        let (op, av) = envio(&rpc, p, 100 + p.ia)?;
        ops.push(op);
        avisos.push(av);
    }
    let (ok, err, k) = aplicar(&rpc, ops);
    anyhow::ensure!(ok, "el lote de envios no aplico: {err}");
    eprintln!("   aplicadas {k}\n");

    let mut veredictos: Vec<(&str, bool, String)> = Vec::new();

    // ── A · el lote MIXTO ──────────────────────────────────────────
    // Cobros de los pendientes de la ronda 0 + envios NUEVOS de las
    // mismas cuentas emisoras. Cuentas todas distintas: los emisores son
    // las `ia` y los receptores las `ib`.
    eprintln!("-- A · lote MIXTO: {n} cobros + {n} envios --");
    let mut mixto = Vec::new();
    for (p, av) in pares.iter().zip(avisos.iter()) {
        mixto.push(cobro(&rpc, p, av)?);
    }
    let mut avisos2 = Vec::new();
    for p in &pares {
        let (op, av) = envio(&rpc, p, 200 + p.ia)?;
        mixto.push(op);
        avisos2.push(av);
    }
    let esperadas = mixto.len();
    let (ok, err, k) = aplicar(&rpc, mixto);
    eprintln!("   -> {} · aplicadas {k} de {esperadas}{}",
              if ok { "APLICA" } else { "RECHAZA" },
              if ok { String::new() } else { format!(" · {err}") });
    veredictos.push(("H1 · mixto de cobros + envios", ok && k == esperadas, err.clone()));

    // ── B · cobro del pendiente que crea un envio del MISMO lote ───
    eprintln!("\n-- B · un cobro del pendiente que crea un envio del MISMO lote --");
    let p = &pares[0];
    let (op_env, av_nuevo) = envio(&rpc, p, 300)?;
    // El cobro cita la MISMA posicion que el envio va a crear.
    let op_cob = cobro(&rpc, p, &av_nuevo);
    match op_cob {
        Ok(op_cob) => {
            let (ok, err, _) = aplicar(&rpc, vec![op_env, op_cob]);
            eprintln!("   -> {} · {err}", if ok { "APLICA (inesperado)" } else { "RECHAZA" });
            veredictos.push((
                "H2 · cobro del pendiente del mismo lote",
                !ok && err.contains("DuplicatePendingInBatch"),
                err,
            ));
        }
        Err(e) => {
            // Que los materiales del cobro no existan todavia es OTRA forma
            // de que el sistema lo impida, y hay que decirlo tal cual.
            eprintln!("   -> ni siquiera hay materiales: {e}");
            veredictos.push((
                "H2 · cobro del pendiente del mismo lote",
                true,
                format!("impedido antes, al pedir materiales: {e}"),
            ));
        }
    }

    // ── C · la misma cuenta dos veces ──────────────────────────────
    eprintln!("\n-- C · la misma cuenta emisora dos veces en un lote --");
    let (o1, _) = envio(&rpc, &pares[1], 401)?;
    let (o2, _) = envio(&rpc, &pares[1], 402)?;
    let (ok, err, _) = aplicar(&rpc, vec![o1, o2]);
    eprintln!("   -> {} · {err}", if ok { "APLICA (inesperado)" } else { "RECHAZA" });
    veredictos.push((
        "H3 · misma cuenta dos veces",
        !ok && err.contains("DuplicateAccountInBatch"),
        err,
    ));

    // ── lectura ────────────────────────────────────────────────────
    println!();
    println!("| hipotesis | esperado | resultado |");
    println!("|---|---|---|");
    for (h, bien, _) in &veredictos {
        println!("| {h} | — | **{}** |", if *bien { "ACIERTA" } else { "FALLA" });
    }

    println!();
    println!("== LECTURA ==");
    for (h, bien, msg) in &veredictos {
        println!("  {} {h}", if *bien { "OK  " } else { "XX  " });
        if !msg.is_empty() {
            println!("      {msg}");
        }
    }

    let mixto_ok = veredictos[0].1;
    println!();
    println!("== VEREDICTO ==");
    if mixto_ok {
        println!("  **El lote MIXTO se admite.** `apply_many` no mira la clase: solo");
        println!("  exige cuentas distintas y posiciones distintas, y un cobro consume");
        println!("  una posicion VIEJA mientras un envio crea una NUEVA.");
        println!();
        println!("  Para la decision de mesa:");
        println!("    · un SOLO agregador puede aplicar ambas mitades en UNA peticion,");
        println!("      o sea en UN turno.");
        println!("    · repartir observacion entre dos agregadores (§231) cuesta");
        println!("      entonces **un turno extra POR ELECCION**, no por diseño.");
        println!("    · y ese turno extra no es gratis: §230 midio que dos que salen");
        println!("      a la vez pierden uno de los dos. Habria que TURNARSE, no");
        println!("      competir.");
    } else {
        println!("  ⚠️ **El lote mixto NO se admite.** H4: la mitigacion de §231 cuesta");
        println!("     un turno extra POR DISEÑO, no por eleccion. Eso encarece el");
        println!("     reparto de observacion y hay que decidirlo sabiendolo.");
    }
    println!();
    println!("⚠️ Lo que este banco NO mide:");
    println!("   · el COSTE del lote mixto frente a dos lotes separados.");
    println!("   · si el orden de las operaciones dentro del lote importa.");
    println!("   · nada de concurrencia: aqui hay un solo cliente.");
    Ok(())
}
