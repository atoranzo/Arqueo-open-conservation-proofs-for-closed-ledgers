//! **Banco I.1 — varios agregadores a la vez: ¿candado o raíz?**
//!
//! ## La pregunta, y por qué no es la que parecía
//!
//! Desde §218 se viene diciendo que «con carga real el paso siguiente es
//! una cola de escritura», señalando al `Mutex` de `dispatch`. §229 midió
//! el techo del nodo —**248 op/s**— con **un solo cliente en serie**, y
//! declaró que la concurrencia seguía sin medir.
//!
//! Pero al leer `two_phase.rs:675` aparece algo que puede hacer irrelevante
//! el candado:
//!
//! ```text
//! if pi.root_old != accounts.root() || pi.pending_root_old != pending.root() {
//!     return Err(LayerError::StaleState);
//! }
//! ```
//!
//! **Un recibo solo vale contra la raíz exacta contra la que se probó.** Si
//! un agregador aplica su lote, la raíz se mueve y los lotes que otros
//! probaron contra la raíz anterior **mueren enteros**.
//!
//! Si eso es así, el cuello no es el `Mutex` —que solo serializa— sino la
//! **raíz avanzando**: la misma contención que D.1 midió operación a
//! operación, pero a granularidad de LOTE, donde perder la carrera cuesta
//! N pruebas en vez de una.
//!
//! ⚠️ Eso es una **lectura del código, no una medida**. Este banco existe
//! para comprobarla, y está construido para poder REFUTARLA.
//!
//! ## Qué hace
//!
//! `agregadores` clientes, cada uno con **sus propias cuentas** —así que no
//! compiten por cuenta ni por posición de pendiente, solo por la raíz—:
//!
//! 1. todos piden materiales contra **la misma raíz**;
//! 2. todos generan sus pruebas en paralelo;
//! 3. **todos envían su `applyMany` a la vez**, en hilos.
//!
//! Se cuenta cuántos lo consiguen, cuántos reciben `StaleState`, y cuántas
//! pruebas se tiran.
//!
//! ## Hipótesis, escritas ANTES del dato
//!
//! - **H1** · exactamente **uno** lo consigue por ronda; los demás reciben
//!   `StaleState`.
//! - **H2** · el rendimiento **no sube con más agregadores**: el nodo sigue
//!   en el entorno de los 248 op/s de §229, porque lo que serializa no es
//!   el candado sino la raíz.
//! - **H3** · el desperdicio es `(N-1)/N` de todo lo generado. Con cuatro
//!   agregadores de ocho operaciones: **24 de 32 pruebas a la basura**, y
//!   cada una costó ~250 ms.
//! - **H4** · ⚠️ si **más de uno** lo consigue, mi lectura de la línea 675
//!   está mal y hay que releerla antes de escribir nada.
//!
//! ## Lo que este banco NO puede decidir
//!
//! Si H1 acierta, la conclusión **no** es «el diseño está roto». Es que
//! `applyMany` asume **un agregador**, y eso ya se intuía en §223 por otra
//! vía. Aquí aparecería una segunda razón, técnica: **dos no pueden ganar
//! la misma raíz.**
//!
//! ⚠️ Aquella vía estaba MAL enunciada y §231 la corrigió: el agregador
//! **no** ve quién paga a quién por procesar envíos. El receptor no viaja
//! en el recibo de envío. Solo ve la arista quien procesa **las dos
//! mitades**, correlacionando por `notice.position`.
//!
//! Qué hacer con eso —cola de escritura en el nodo, agregador único por
//! diseño, o encadenar lotes— es **decisión de mesa**, y este banco no la
//! toma.
//!
//! ## Cómo se usa
//!
//! ```text
//! cargo run --release -p zk-ssl-node -- --dev --listen 127.0.0.1:8648 &
//! cargo run --release -p zk-ssl-sdk --example i1_concurrencia -- http://127.0.0.1:8648 4 8 3
//! ```
//! (url, agregadores, operaciones por lote, rondas)
//!
//! ⚠️ **Muta el estado del nodo.** Usar uno desechable.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use serde_json::{json, Value};
use winterfell::math::fields::f64::BaseElement;

use zk_ssl::client::{self, SendMaterials};
use zk_ssl::commitment::ClientState;
use zk_ssl::proof_options;
use zk_ssl_sdk::{Account, Rpc, Wallet};
use zk_ssl_wire as wire;
use zk_ssl_wire::{digest_to_wire, Q};

type Digest = [BaseElement; 4];

struct Par {
    sa: [u64; 4],
    ia: u64,
    id_b: Digest,
}

fn estado(a: &Account) -> anyhow::Result<ClientState> {
    let v = a.view()?;
    Ok(ClientState { public_id: v.public_id, balance: v.balance, nonce: v.nonce })
}

fn sal(n: u64) -> Digest {
    [BaseElement::new(n), BaseElement::new(0), BaseElement::new(0), BaseElement::new(0)]
}

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let url = args.next().unwrap_or_else(|| "http://127.0.0.1:8648".into());
    let n_agg: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(4);
    let n_ops: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(8);
    let rondas: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(3);

    eprintln!("== BANCO I.1 · varios agregadores a la vez ==");
    eprintln!("   nodo: {url} · {n_agg} agregadores · {n_ops} ops por lote · {rondas} rondas");
    eprintln!("   §229 midio con UN cliente en serie: 248 op/s");
    eprintln!("   two_phase.rs:675 rechaza con StaleState si la raiz se movio");
    eprintln!("   se predice: UNO lo consigue por ronda, {} pruebas tiradas de {}",
              (n_agg - 1) * n_ops, n_agg * n_ops);
    eprintln!("   ⚠️ si lo consigue MAS DE UNO, mi lectura del codigo esta mal\n");

    let rpc = Rpc::new(url.clone());
    let v: Value = rpc.call("zkssl_protocolVersion", json!([]))?;
    eprintln!("   protocolo: {v}");

    // ── Montaje: cada agregador con SUS cuentas ────────────────────
    eprintln!("-- montaje: {} pares --", n_agg * n_ops);
    let mut grupos: Vec<Vec<Par>> = Vec::new();
    for g in 0..n_agg {
        let mut pares = Vec::new();
        for i in 0..n_ops {
            let s = (g * 1_000 + i) as u64;
            let sa: [u64; 4] = [0xC0A0 + s * 7, 1, 2, 3];
            let sb: [u64; 4] = [0xD0B0 + s * 7, 4, 5, 6];
            let wa = Wallet::from_elements(sa);
            let wb = Wallet::from_elements(sb);
            let a = Account::open(&rpc, wa)?;
            Account::open(&rpc, wb)?;
            rpc.dev_fund(a.index, 500_000)?;
            pares.push(Par { sa, ia: a.index, id_b: wb.public_id() });
        }
        grupos.push(pares);
    }
    eprintln!("   {} cuentas abiertas\n", n_agg * n_ops * 2);

    let mut total_gen = 0usize;
    let mut total_ok = 0usize;
    let mut total_stale = 0usize;
    let mut total_otro = 0usize;
    let mut segundos = 0.0f64;

    for r in 0..rondas {
        // 1 · TODOS los materiales, contra la MISMA raiz
        let mut trabajos: Vec<Vec<(Digest, u64, ClientState, SendMaterials)>> = Vec::new();
        for pares in &grupos {
            let mut t = Vec::new();
            for p in pares {
                let cuenta = Account::attach(&rpc, Wallet::from_elements(p.sa), p.ia);
                let est = estado(&cuenta)?;
                let m_dto: wire::SendMaterialsDto = rpc.call(
                    "zkssl_sendMaterials",
                    json!({
                        "sender": Q(p.ia),
                        "receiverId": digest_to_wire(&p.id_b),
                        "amount": Q(1_000u64),
                        "salt": digest_to_wire(&sal(r as u64 * 10_000 + p.ia)),
                    }),
                )?;
                let m: SendMaterials = (&m_dto).try_into()?;
                t.push((p.sa.map(BaseElement::new), p.ia, est, m));
            }
            trabajos.push(t);
        }

        // 2 · todas las pruebas, en paralelo
        let t_gen = Instant::now();
        let lotes: Vec<Vec<Value>> = std::thread::scope(|s| {
            let hs: Vec<_> = trabajos
                .iter()
                .map(|grupo| {
                    s.spawn(move || {
                        grupo
                            .iter()
                            .map(|(sk, ia, est, m)| {
                                let rec =
                                    client::prove_send(m, *sk, proof_options()).expect("prove");
                                json!({
                                    "kind": "send",
                                    "receipt": wire::SendReceiptDto::from(&rec),
                                    "sender": Q(*ia),
                                    "senderState": wire::ClientStateDto::from(est),
                                    "amount": Q(1_000u64),
                                })
                            })
                            .collect::<Vec<Value>>()
                    })
                })
                .collect();
            hs.into_iter().map(|h| h.join().expect("hilo")).collect()
        });
        let ms_gen = t_gen.elapsed().as_secs_f64() * 1000.0;
        total_gen += n_agg * n_ops;

        // 3 · TODOS envian a la vez
        let ok = AtomicUsize::new(0);
        let stale = AtomicUsize::new(0);
        let otro = AtomicUsize::new(0);
        let t = Instant::now();
        std::thread::scope(|s| {
            for lote in &lotes {
                let url = url.clone();
                let ok = &ok;
                let stale = &stale;
                let otro = &otro;
                s.spawn(move || {
                    let mio = Rpc::new(url);
                    // `call<P, R>` toma DOS genericos —parametros y
                    // resultado—; se anota el tipo del resultado en vez de
                    // usar turbofish, que ahi solo fijaria el primero.
                    let r: anyhow::Result<Value> =
                        mio.call("zkssl_applyMany", json!({ "ops": lote }));
                    match r {
                        Ok(_) => {
                            ok.fetch_add(1, Ordering::Relaxed);
                        }
                        Err(e) => {
                            let msg = format!("{e}");
                            if msg.contains("StaleState") {
                                stale.fetch_add(1, Ordering::Relaxed);
                            } else {
                                eprintln!("      otro error: {msg}");
                                otro.fetch_add(1, Ordering::Relaxed);
                            }
                        }
                    }
                });
            }
        });
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        segundos += ms / 1000.0;

        let (o, st, ot) = (ok.load(Ordering::Relaxed), stale.load(Ordering::Relaxed), otro.load(Ordering::Relaxed));
        total_ok += o;
        total_stale += st;
        total_otro += ot;
        eprintln!(
            "   ronda {} · aplicaron {o}/{n_agg} · StaleState {st} · otros {ot} · \
             enviar {ms:.0} ms · generar {ms_gen:.0} ms",
            r + 1
        );
    }

    // ── lectura ────────────────────────────────────────────────────
    let aplicadas = total_ok * n_ops;
    let tiradas = (total_stale + total_otro) * n_ops;
    let ops_s = aplicadas as f64 / segundos.max(1e-9);

    println!();
    println!("| agregadores | ops/lote | rondas | lotes OK | StaleState | pruebas tiradas | op/s |");
    println!("|---|---|---|---|---|---|---|");
    println!(
        "| {n_agg} | {n_ops} | {rondas} | **{total_ok}** | **{total_stale}** | **{tiradas} de {total_gen}** | **{ops_s:.0}** |"
    );

    println!();
    println!("== LECTURA ==");
    println!("  lotes enviados a la vez: {}", n_agg * rondas);
    println!("  lotes que aplicaron .... {total_ok}");
    println!("  rechazados por raiz .... {total_stale}");
    println!("  otros errores .......... {total_otro}");
    println!("  desperdicio ............ {:.0} % de las pruebas generadas",
             100.0 * tiradas as f64 / total_gen.max(1) as f64);
    println!("  op/s aplicadas ......... {ops_s:.0}   (§229 con UN cliente: 248)");
    println!();
    let uno_por_ronda = total_ok == rondas;
    println!("  H1 (uno por ronda): {}", if uno_por_ronda { "ACIERTA" } else { "FALLA" });
    println!("  H3 ({}% de desperdicio): {:.0} % medido",
             (100 * (n_agg - 1)) / n_agg,
             100.0 * tiradas as f64 / total_gen.max(1) as f64);
    println!();
    println!("== VEREDICTO ==");
    if total_ok > rondas {
        println!("  ⚠️ H4: APLICARON MAS DE UNO POR RONDA ({total_ok} en {rondas}).");
        println!("     Mi lectura de two_phase.rs:675 esta MAL. Releerla antes de");
        println!("     escribir nada: el nodo admite agregadores concurrentes y la");
        println!("     pregunta del candado vuelve a estar abierta.");
    } else if uno_por_ronda {
        println!("  Uno por ronda. **Lo que serializa NO es el Mutex: es la RAIZ.**");
        println!("  Un recibo solo vale contra la raiz contra la que se probo, asi");
        println!("  que el primer lote que aplica mata a todos los demas.");
        println!();
        println!("  Consecuencias, dichas con cuidado:");
        println!("    · una cola de escritura en el nodo NO arreglaria esto: el");
        println!("      candado no es el cuello, y quitarlo no cambiaria nada.");
        println!("    · `applyMany` asume UN agregador. §223 lo intuia por otra");
        println!("      via —MAL enunciada, corregida en §231—; aqui aparece una");
        println!("      segunda razon, tecnica: **dos no pueden ganar la misma raiz**.");
        println!("    · y perder la carrera cuesta ahora {n_ops} pruebas, no una.");
        println!("      El lote MULTIPLICA el precio de la contencion que evita.");
    } else {
        println!("  ⚠️ No aplico ninguno o el reparto es raro: revisar los errores");
        println!("     de arriba antes de concluir.");
    }
    println!();
    println!("⚠️ Lo que este banco NO decide:");
    println!("   · Que hacer al respecto —cola de escritura, agregador unico por");
    println!("     diseño, o encadenar lotes— es decision de MESA, no de banco.");
    println!("   · Ni mide la LATENCIA por peticion, ni el reparto entre quien gana");
    println!("     y quien pierde: aqui todos salen a la vez y desde el mismo");
    println!("     proceso.");
    println!("   · Solo ENVIOS, nodo en memoria, y este hardware.");
    Ok(())
}
