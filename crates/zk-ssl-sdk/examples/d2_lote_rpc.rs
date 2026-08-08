//! **Banco D.2 — el lote por RPC, contra la línea base del nodo.**
//!
//! §216 midió que `apply_many` elimina el desperdicio de contención hasta
//! el mínimo teórico exacto — **a nivel de capa**. §218 midió la línea
//! base del nodo **por RPC**: `2,90 ± 0,33` regeneraciones por pago,
//! **59 %** de trabajo criptográfico tirado, `1,72 ± 0,13` pagos/s.
//!
//! §222 cableó `zkssl_applyMany`. Este banco comprueba si la mejora que
//! §216 vio en la capa **llega hasta el RPC**, que es donde vive el nodo.
//!
//! ## Qué hace, y por qué en este orden
//!
//! Por cada ronda de pagos, con `pares` titulares:
//!
//! 1. **Los `pares` `sendMaterials`, todos seguidos.** Nadie aplica en
//!    medio, así que todos reciben materiales contra **la misma raíz de
//!    arranque** — que es la condición que hace útil el lote. Y desde §220
//!    cada uno recibe una **posición distinta**: sin eso, `apply_many`
//!    rechazaría el lote entero con `DuplicatePendingInBatch`.
//! 2. **Los `pares` `prove_send` EN PARALELO**, un hilo por titular. Es lo
//!    que hace un despliegue real: N titulares probando a la vez.
//! 3. **UN `applyMany`** con las N operaciones.
//! 4. Lo mismo para los cobros.
//!
//! ## La clave no viaja, y eso importa aquí
//!
//! `Wallet` guarda su clave de gasto y **no la expone** — es deliberado.
//! Este banco crea las carteras con `Wallet::from_elements`, así que tiene
//! su propia copia y puede probar fuera de `Account::pay()`.
//!
//! ⚠️ **Un agregador real NO necesita claves: necesita RECIBOS.** Cada
//! prueba se genera en la máquina de su titular y lo que se junta para el
//! lote son los recibos ya firmados. Que aquí ambos papeles vivan en un
//! proceso es una comodidad del banco, no una propiedad del diseño.
//!
//! ## La hipótesis, escrita antes del dato
//!
//! - Las generaciones deben caer al **mínimo teórico exacto**: `2 × pagos`,
//!   ni una más. Cero `StaleState`, porque nadie aplica entre materiales
//!   y prueba.
//! - **3,7-4,2 pagos/s**, del orden del 2,28× que §216 midió en la capa.
//! - **Si sale por debajo de 3**, hay un coste del nodo que el lote no
//!   toca, y tocaría la cola de escritura que `main.rs` anticipa.
//!
//!   ⚠️ Salió 4,95, así que esta rama no se tomó. Y §230 midió que la
//!   cola de escritura no habría servido: lo que serializa es la raíz,
//!   no el candado.
//!
//! ## Cómo se usa
//!
//! Necesita un nodo en marcha con `--dev`:
//!
//! ```text
//! cargo run --release -p zk-ssl-node -- --dev --listen 127.0.0.1:8646 &
//! cargo run --release -p zk-ssl-sdk --example d2_lote_rpc -- http://127.0.0.1:8646 4 2
//! ```
//! (url, pares de titulares, pagos por par)
//!
//! ⚠️ **Muta el estado del nodo.** Usar uno desechable; sin `--ledger` la
//! capa vive en memoria y no deja nada en disco.
//!
//! ⚠️ **UNA corrida no basta.** La dispersión de este tipo de banco es del
//! ±11 % (§218). Correr tres veces con nodo NUEVO y quedarse con la media.

use std::time::Instant;

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

/// Un par pagador/receptor. Se guardan las SEMILLAS, no las carteras: de
/// una semilla salen las dos cosas que hacen falta —la `Wallet` para
/// hablar con el nodo y la clave de gasto para PROBAR— y guardarlas por
/// separado es como se equivoca uno de cartera.
struct Par {
    sa: [u64; 4],
    ia: u64,
    sb: [u64; 4],
    ib: u64,
    id_b: Digest,
}

/// Estado del cliente, tal y como lo arma el SDK (`Account::state`, que es
/// privado). Se reconstruye desde `view()`, que sí es público.
fn estado(a: &Account) -> anyhow::Result<ClientState> {
    let v = a.view()?;
    Ok(ClientState { public_id: v.public_id, balance: v.balance, nonce: v.nonce })
}

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let url = args.next().unwrap_or_else(|| "http://127.0.0.1:8646".into());
    let pares: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(4);
    let pagos: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(2);

    eprintln!("== BANCO D.2 · el lote por RPC ==");
    eprintln!("   nodo: {url} · {pares} pares · {pagos} pagos por par");
    eprintln!("   linea base del NODO (§218, cuatro corridas):");
    eprintln!("     2,90 +- 0,33 regen/pago · 59 % tirado · 1,72 +- 0,13 pagos/s");
    eprintln!("   referencia de CAPA (§216, modo lote): 0 % tirado, minimo teorico");
    eprintln!("   ⚠️ escribe en el ledger del nodo: usar uno desechable\n");

    let rpc = Rpc::new(url.clone());
    let version: Value = rpc.call("zkssl_protocolVersion", json!([]))?;
    eprintln!("   protocolo del nodo: {version}");

    // ── Montaje ───────────────────────────────────────────────────────
    eprintln!("-- montaje --");
    let mut pares_v: Vec<Par> = Vec::new();
    for i in 0..pares {
        let sa: [u64; 4] = [0xA11CE + i as u64 * 7, 11, 22, 33];
        let sb: [u64; 4] = [0xB0B + i as u64 * 7, 44, 55, 66];
        let wa = Wallet::from_elements(sa);
        let wb = Wallet::from_elements(sb);
        let a = Account::open(&rpc, wa)?;
        let b = Account::open(&rpc, wb)?;
        rpc.dev_fund(a.index, 1_000_000)?;
        rpc.dev_fund(b.index, 1_000_000)?;
        pares_v.push(Par { sa, ia: a.index, sb, ib: b.index, id_b: wb.public_id() });
    }
    eprintln!("   {} pares abiertos y fondeados\n", pares_v.len());

    // ── La medida ─────────────────────────────────────────────────────
    let mut generaciones = 0u64;
    let mut hechos = 0u64;
    let mut lotes = 0u64;
    let t0 = Instant::now();

    for ronda in 0..pagos {
        // ─── FASE A · los envios, en lote ───────────────────────────
        // 1 · TODOS los materiales primero: nadie aplica en medio, asi
        //     que todos salen contra la MISMA raiz de arranque.
        let mut trabajos = Vec::new();
        for p in &pares_v {
            // ⚠️ La cartera REAL: `estado()` pide la vista AUTENTICADA y una
            // cartera equivocada da `AccountNotFound`, no un error claro.
            let cuenta = Account::attach(&rpc, Wallet::from_elements(p.sa), p.ia);
            let est = estado(&cuenta)?;
            let m_dto: wire::SendMaterialsDto = rpc.call(
                "zkssl_sendMaterials",
                json!({
                    "sender": Q(p.ia),
                    "receiverId": digest_to_wire(&p.id_b),
                    "amount": Q(1_000u64),
                    "salt": digest_to_wire(&[
                        BaseElement::new(7 + ronda as u64),
                        BaseElement::new(p.ia),
                        // Se usa `new(0)` y no la constante del trait
                        // `FieldElement`, que obligaria a importarlo solo
                        // para esto y arriesgaria un warning si dejara de
                        // usarse. La compuerta exige warnings a cero.
                        BaseElement::new(0),
                        BaseElement::new(0),
                    ]),
                }),
            )?;
            let m: SendMaterials = (&m_dto).try_into()?;
            m.check_recipient(p.id_b)
                .map_err(|e| anyhow::anyhow!("materiales con otro destinatario: {e:?}"))?;
            trabajos.push((p.sa.map(BaseElement::new), p.ia, est, m));
        }

        // 2 · las pruebas, EN PARALELO: un hilo por titular.
        let recibos: Vec<_> = std::thread::scope(|s| {
            let hs: Vec<_> = trabajos
                .iter()
                .map(|(sk, ia, est, m)| {
                    s.spawn(move || {
                        let r = client::prove_send(m, *sk, proof_options())
                            .expect("prove_send");
                        (r, *ia, est.clone())
                    })
                })
                .collect();
            hs.into_iter().map(|h| h.join().expect("hilo")).collect()
        });
        generaciones += recibos.len() as u64;

        // 3 · UN solo applyMany.
        let ops: Vec<Value> = recibos
            .iter()
            .map(|(r, ia, est)| {
                json!({
                    "kind": "send",
                    "receipt": wire::SendReceiptDto::from(r),
                    "sender": Q(*ia),
                    "senderState": wire::ClientStateDto::from(est),
                    "amount": Q(1_000u64),
                })
            })
            .collect();
        let res: Value = rpc.call("zkssl_applyMany", json!({ "ops": ops }))?;
        lotes += 1;
        let n_apl = res["applied"].as_array().map(|a| a.len()).unwrap_or(0);
        eprintln!(
            "   ronda {} · ENVIOS en lote: {} ops · batch.size {} · applied {}",
            ronda + 1,
            ops.len(),
            res["batch"]["size"],
            n_apl
        );
        anyhow::ensure!(n_apl == ops.len(), "el lote de envios no aplico todo");

        let avisos: Vec<PendingNotice> = recibos.iter().map(|(r, ..)| r.notice.clone()).collect();

        // ─── FASE B · los cobros, en lote ───────────────────────────
        let mut trabajos_c = Vec::new();
        for (p, aviso) in pares_v.iter().zip(avisos.iter()) {
            let cuenta = Account::attach(&rpc, Wallet::from_elements(p.sb), p.ib);
            let est = estado(&cuenta)?;
            let m_dto: wire::ClaimMaterialsDto = rpc.call(
                "zkssl_claimMaterials",
                json!({
                    "receiver": Q(p.ib),
                    "notice": wire::PendingNoticeDto::from(aviso),
                }),
            )?;
            let m: ClaimMaterials = (&m_dto).try_into()?;
            trabajos_c.push((p.sb.map(BaseElement::new), p.ib, est, m, aviso.clone()));
        }

        let recibos_c: Vec<_> = std::thread::scope(|s| {
            let hs: Vec<_> = trabajos_c
                .iter()
                .map(|(sk, ib, est, m, av)| {
                    s.spawn(move || {
                        let r = client::prove_claim(m, *sk, proof_options())
                            .expect("prove_claim");
                        (r, *ib, est.clone(), av.clone())
                    })
                })
                .collect();
            hs.into_iter().map(|h| h.join().expect("hilo")).collect()
        });
        generaciones += recibos_c.len() as u64;

        let ops_c: Vec<Value> = recibos_c
            .iter()
            .map(|(r, ib, est, av)| {
                json!({
                    "kind": "claim",
                    "receipt": wire::ClaimReceiptDto::from(r),
                    "receiver": Q(*ib),
                    "receiverState": wire::ClientStateDto::from(est),
                    "notice": wire::PendingNoticeDto::from(av),
                })
            })
            .collect();
        let res_c: Value = rpc.call("zkssl_applyMany", json!({ "ops": ops_c }))?;
        lotes += 1;
        let n_c = res_c["applied"].as_array().map(|a| a.len()).unwrap_or(0);
        eprintln!(
            "   ronda {} · COBROS en lote: {} ops · batch.size {} · applied {}",
            ronda + 1,
            ops_c.len(),
            res_c["batch"]["size"],
            n_c
        );
        anyhow::ensure!(n_c == ops_c.len(), "el lote de cobros no aplico todo");

        hechos += ops.len() as u64;
    }

    let segundos = t0.elapsed().as_secs_f64();
    let minimo = hechos * 2;
    let desperdicio =
        100.0 * (generaciones.saturating_sub(minimo)) as f64 / generaciones.max(1) as f64;
    let regen = (generaciones.saturating_sub(minimo)) as f64 / hechos.max(1) as f64;
    let tps = hechos as f64 / segundos.max(1e-9);

    println!();
    println!("| via | pares | pagos | generaciones | minimo | desperdicio | regen/pago | pagos/s |");
    println!("|---|---|---|---|---|---|---|---|");
    println!(
        "| **LOTE por RPC** | {pares} | {hechos} | {generaciones} | {minimo} | **{desperdicio:.0} %** | **{regen:.2}** | **{tps:.2}** |"
    );
    println!("| suelto por RPC (§218, media de 4) | 4 | 8 | 39,25 | 16 | 59 % | 2,90 | 1,72 |");
    println!("| capa, secuencial (§216) | 4 | 8 | 44 | 16 | 64 % | 3,50 | 1,62 |");
    println!("| capa, lote (§216) | 4 | 8 | 16 | 16 | 0 % | 0,00 | 3,70 |");

    println!();
    println!("== LECTURA ==");
    println!("  lotes enviados: {lotes} · segundos: {segundos:.1}");
    println!("  linea base del nodo (§218): 1,72 ± 0,13 pagos/s");
    println!();
    if generaciones == minimo {
        println!("  ✅ CERO desperdicio: {generaciones} generaciones para un minimo de");
        println!("     {minimo}. Ninguna prueba llego muerta — nadie aplico entre los");
        println!("     materiales y la prueba, que es exactamente lo que el lote compra.");
    } else {
        println!("  ⚠️ {} generaciones sobrantes. El lote NO llego al minimo teorico,", generaciones - minimo);
        println!("     y eso no deberia pasar: investigar antes de creerse el resto.");
    }
    println!();
    if tps >= 3.7 {
        println!("VEREDICTO: ✅ EL LOTE LLEGA HASTA EL RPC.");
        println!("  {tps:.2} pagos/s frente a los 1,72 de la via suelta: la mejora que");
        println!("  §216 midio en la capa NO se la come el nodo.");
    } else if tps >= 3.0 {
        println!("VEREDICTO: el lote mejora, pero por debajo de lo predicho.");
        println!("  {tps:.2} pagos/s (se predijo 3,7-4,2). Hay un coste del nodo que el");
        println!("  lote no toca; medirlo antes de prometer nada.");
    } else {
        println!("VEREDICTO: ⚠️ EL LOTE NO BASTA POR RPC ({tps:.2} pagos/s).");
        println!("  Predicho 3,7-4,2, y por debajo de 3 el cuello NO es la contencion:");
        println!("  es el Mutex de `dispatch` o el cable. Tocaria la COLA DE ESCRITURA");
        println!("  que `main.rs` anticipa, y el lote solo, no lo arregla.");
    }
    println!();
    println!("⚠️ UNA corrida no decide: la dispersion de este tipo de banco es del");
    println!("   ±11 % (§218). Correr tres veces con nodo NUEVO y usar la media.");
    println!("⚠️ Lo que NO se mide aqui: la latencia de JUNTAR los recibos. Un");
    println!("   agregador real espera al titular mas lento antes de poder enviar");
    println!("   el lote, y eso no entra en pagos/s.");
    Ok(())
}
