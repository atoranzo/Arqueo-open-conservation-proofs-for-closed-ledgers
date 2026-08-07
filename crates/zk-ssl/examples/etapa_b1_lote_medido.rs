//! **Banco B.1 — ¿sirvió de algo el lote?**
//!
//! El RFC-0002 se abrió midiendo y se cierra midiendo. `apply_many`
//! existe desde §215 y sus tests pasan, pero **nadie había medido su
//! rendimiento**: que una función haga lo que dice no significa que
//! arregle lo que se escribió que arreglaría.
//!
//! ## Lo que se compara
//!
//! Dos modos, la misma carga, la misma máquina, en la misma corrida:
//!
//! | modo | qué hace |
//! |---|---|
//! | **secuencial** | como `etapa_a5_concurrencia` (§204): cada hilo pide materiales, genera y aplica; si sale `StaleState`, **regenera** |
//! | **lote** | el nodo reserva N posiciones y reparte materiales contra **una misma raíz**; los N generan **en paralelo**; el nodo aplica con `apply_many` |
//!
//! ## La cifra que decide
//!
//! **Regeneraciones por pago.** §204 midió **3,83** con cuatro hilos —70
//! generaciones para 24 operaciones, el **66 % del trabajo criptográfico
//! tirado**— y el rendimiento **bajando** al paralelizar: un livelock.
//!
//! Si el lote sirve, esa cifra debe caer a **cero**: por construcción no
//! puede haber `StaleState`, porque todas las pruebas se generan contra
//! la raíz que el lote va a usar y nada se aplica entremedias.
//!
//! ⚠️ **Aviso de método, el mismo de §204.** Todos los "clientes"
//! comparten una máquina, así que si generar ya usa varios núcleos, el
//! tiempo de pared no se multiplicará aunque el desperdicio desaparezca.
//! **La medida que decide es el recuento de generaciones**, no los
//! pagos/s. Un despliegue real tiene los clientes en máquinas distintas.
//!
//! Uso:
//! ```text
//! cargo run --release -p zk-ssl --features sandbox --example etapa_b1_lote_medido -- 4 2
//! ```
//! (hilos/lote, rondas)

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use winterfell::math::fields::f64::BaseElement;

use zk_ssl::commitment::ClientState;
use zk_ssl::tests_support as ts;
use zk_ssl::two_phase::BatchOp;
use zk_ssl::{client, proof_options, AccountIndex, LayerError, SovereignLayer};

fn estado(layer: &SovereignLayer, idx: AccountIndex) -> ClientState {
    ClientState {
        public_id: layer.public_id_of(idx).expect("cuenta abierta"),
        balance: layer.balance_of(idx).expect("cuenta abierta"),
        nonce: layer.nonce_of(idx).expect("cuenta abierta"),
    }
}

fn fondear(layer: &mut SovereignLayer, idx: AccountIndex, importe: u64) {
    let op = ts::mint_commitment(layer, idx, importe);
    let subida = ts::mint_climb_proof(layer, idx, importe);
    let (pa, ia, pb, ib) = ts::delegated_pair(op, 1, 3);
    layer
        .apply_mint_delegated(subida, pa, ia, pb, ib, idx, importe)
        .expect("fondear");
}

/// Un par pagador→receptor con sus claves.
/// ⚠️ Las claves son **anchas**: `[BaseElement; 4]`, no un escalar. Es
/// lo que `prove_send`/`prove_claim` esperan, y lo que devuelve
/// `wide_key`.
type Par = (
    AccountIndex,
    AccountIndex,
    [BaseElement; 4],
    [BaseElement; 4],
);

fn montar(n: usize) -> (SovereignLayer, Vec<Par>) {
    let mut layer = SovereignLayer::new(
        ts::custodian_root(),
        ts::governance_root(),
        ts::LIMIT,
        ts::MAX_SUPPLY,
        ts::MAX_ACCOUNTS,
    );
    let mut pares = Vec::new();
    for h in 0..n as u64 {
        let ka = ts::wide_key(0xA11CE + h * 2);
        let kb = ts::wide_key(0xA11CE + h * 2 + 1);
        let a = layer.open_account_wide(ka);
        let b = layer.open_account_wide(kb);
        fondear(&mut layer, a, 1_000_000);
        fondear(&mut layer, b, 1_000_000);
        pares.push((a, b, ka, kb));
    }
    (layer, pares)
}

struct Medida {
    pagos: u64,
    generaciones: u64,
    stale: u64,
    segundos: f64,
}

impl Medida {
    fn regeneraciones_por_pago(&self) -> f64 {
        // Un pago "limpio" son DOS generaciones: envio y cobro.
        let limpias = self.pagos * 2;
        (self.generaciones.saturating_sub(limpias)) as f64 / self.pagos.max(1) as f64
    }
}

// ───────────────────────── MODO SECUENCIAL ─────────────────────────

fn modo_secuencial(hilos: usize, rondas: u32) -> Medida {
    let (layer, pares) = montar(hilos);
    let capa = Arc::new(Mutex::new(layer));
    let gen = Arc::new(AtomicU64::new(0));
    let stale = Arc::new(AtomicU64::new(0));
    let hechos = Arc::new(AtomicU64::new(0));
    let t0 = Instant::now();

    std::thread::scope(|s| {
        for (h, &(a, b, ka, kb)) in pares.iter().enumerate() {
            let capa = Arc::clone(&capa);
            let (g, st, hh) = (Arc::clone(&gen), Arc::clone(&stale), Arc::clone(&hechos));
            s.spawn(move || {
                for i in 0..rondas {
                    let salt = ts::salt_de(1_000 * (h as u64 + 1) + i as u64);
                    // FASE 1
                    let envio = loop {
                        let (m, est) = {
                            let l = capa.lock().unwrap();
                            let est = estado(&l, a);
                            let r = l.public_id_of(b).expect("receptor");
                            (l.send_materials(a, r, 1_000, salt).expect("materiales"), est)
                        };
                        g.fetch_add(1, Ordering::Relaxed);
                        let e = client::prove_send(&m, ka, proof_options()).expect("prueba");
                        let r = {
                            let mut l = capa.lock().unwrap();
                            l.apply_send(&e, a, &est, 1_000)
                        };
                        match r {
                            Ok(()) => break e,
                            Err(LayerError::StaleState) => {
                                st.fetch_add(1, Ordering::Relaxed);
                            }
                            Err(e) => panic!("apply_send: {e:?}"),
                        }
                    };
                    // FASE 2
                    loop {
                        let (m, est) = {
                            let l = capa.lock().unwrap();
                            let est = estado(&l, b);
                            (l.claim_materials(b, &envio.notice).expect("materiales"), est)
                        };
                        g.fetch_add(1, Ordering::Relaxed);
                        let c = client::prove_claim(&m, kb, proof_options()).expect("prueba");
                        let r = {
                            let mut l = capa.lock().unwrap();
                            l.apply_claim(&c, b, &est, &envio.notice)
                        };
                        match r {
                            Ok(()) => break,
                            Err(LayerError::StaleState) => {
                                st.fetch_add(1, Ordering::Relaxed);
                            }
                            Err(e) => panic!("apply_claim: {e:?}"),
                        }
                    }
                    hh.fetch_add(1, Ordering::Relaxed);
                }
            });
        }
    });

    Medida {
        pagos: hechos.load(Ordering::Relaxed),
        generaciones: gen.load(Ordering::Relaxed),
        stale: stale.load(Ordering::Relaxed),
        segundos: t0.elapsed().as_secs_f64(),
    }
}

// ─────────────────────────── MODO LOTE ───────────────────────────

fn modo_lote(hilos: usize, rondas: u32) -> Medida {
    let (layer, pares) = montar(hilos);
    let capa = Arc::new(Mutex::new(layer));
    let mut generaciones = 0u64;
    let mut stale = 0u64;
    let mut pagos = 0u64;
    let t0 = Instant::now();

    for i in 0..rondas {
        // ── FASE 1: N envíos en un lote ──
        // 1a · materiales para los N, contra la MISMA raíz, con posición
        //      reservada (§211): sin reservar, dos recibirían la misma.
        let mut trabajo = Vec::new();
        {
            let mut l = capa.lock().unwrap();
            for (h, &(a, b, ka, _)) in pares.iter().enumerate() {
                let est = estado(&l, a);
                let r = l.public_id_of(b).expect("receptor");
                let p = l.reserve_pending().expect("reserva");
                let salt = ts::salt_de(1_000 * (h as u64 + 1) + i as u64);
                let m = l
                    .send_materials_at(a, r, 1_000, salt, p)
                    .expect("materiales");
                trabajo.push((m, est, ka, a));
            }
        }
        // 1b · generar EN PARALELO, sin candado.
        let envios: Vec<_> = std::thread::scope(|s| {
            let hs: Vec<_> = trabajo
                .iter()
                .map(|(m, _, k, _)| {
                    s.spawn(move || client::prove_send(m, *k, proof_options()).expect("prueba"))
                })
                .collect();
            hs.into_iter().map(|h| h.join().unwrap()).collect()
        });
        generaciones += envios.len() as u64;

        // 1c · aplicar el lote entero.
        {
            let mut l = capa.lock().unwrap();
            let ops: Vec<BatchOp> = envios
                .iter()
                .zip(trabajo.iter())
                .map(|(e, (_, est, _, a))| BatchOp::Send {
                    receipt: e,
                    sender_index: *a,
                    sender_state: est,
                    amount: 1_000,
                })
                .collect();
            match l.apply_many(&ops) {
                Ok(()) => {}
                Err(LayerError::StaleState) => {
                    stale += ops.len() as u64;
                    panic!("el lote de envios no deberia quedar obsoleto");
                }
                Err(e) => panic!("apply_many (envios): {e:?}"),
            }
        }

        // ── FASE 2: N cobros en otro lote ──
        let mut trabajo2 = Vec::new();
        {
            let mut l = capa.lock().unwrap();
            for (j, &(_, b, _, kb)) in pares.iter().enumerate() {
                let est = estado(&l, b);
                let m = l
                    .claim_materials(b, &envios[j].notice)
                    .expect("materiales");
                trabajo2.push((m, est, kb, b, envios[j].notice.clone()));
            }
        }
        let cobros: Vec<_> = std::thread::scope(|s| {
            let hs: Vec<_> = trabajo2
                .iter()
                .map(|(m, _, k, _, _)| {
                    s.spawn(move || client::prove_claim(m, *k, proof_options()).expect("prueba"))
                })
                .collect();
            hs.into_iter().map(|h| h.join().unwrap()).collect()
        });
        generaciones += cobros.len() as u64;

        {
            let mut l = capa.lock().unwrap();
            let ops: Vec<BatchOp> = cobros
                .iter()
                .zip(trabajo2.iter())
                .map(|(c, (_, est, _, b, notice))| BatchOp::Claim {
                    receipt: c,
                    receiver_index: *b,
                    receiver_state: est,
                    notice,
                })
                .collect();
            match l.apply_many(&ops) {
                Ok(()) => {}
                Err(LayerError::StaleState) => {
                    stale += ops.len() as u64;
                    panic!("el lote de cobros no deberia quedar obsoleto");
                }
                Err(e) => panic!("apply_many (cobros): {e:?}"),
            }
        }
        pagos += pares.len() as u64;
    }

    // El registro debe encadenar despues de todos los lotes.
    {
        let l = capa.lock().unwrap();
        l.transition_log()
            .verify_chain()
            .expect("la cadena debe encadenar tras los lotes");
    }

    Medida {
        pagos,
        generaciones,
        stale,
        segundos: t0.elapsed().as_secs_f64(),
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let hilos: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(4);
    let rondas: u32 = args.next().and_then(|s| s.parse().ok()).unwrap_or(2);

    eprintln!("== BANCO B.1 · ¿sirvio de algo el lote? ==");
    eprintln!("   nucleos: {}", std::thread::available_parallelism().map(|n| n.get()).unwrap_or(0));
    eprintln!("   {hilos} pares de cuentas x {rondas} rondas = {} pagos por modo\n", hilos as u32 * rondas);

    eprintln!("-- modo SECUENCIAL (como hoy) --");
    let sec = modo_secuencial(hilos, rondas);
    eprintln!(
        "   {} pagos · {} generaciones · {} StaleState · {:.1} s",
        sec.pagos, sec.generaciones, sec.stale, sec.segundos
    );

    eprintln!("-- modo LOTE (apply_many) --");
    let lote = modo_lote(hilos, rondas);
    eprintln!(
        "   {} pagos · {} generaciones · {} StaleState · {:.1} s",
        lote.pagos, lote.generaciones, lote.stale, lote.segundos
    );

    let tps_s = sec.pagos as f64 / sec.segundos.max(1e-9);
    let tps_l = lote.pagos as f64 / lote.segundos.max(1e-9);
    let limpias = sec.pagos * 2;

    println!();
    println!("| modo | pagos | generaciones | minimo posible | desperdicio | StaleState | regen/pago | pagos/s |");
    println!("|---|---|---|---|---|---|---|---|");
    for (n, m, t) in [("secuencial", &sec, tps_s), ("**lote**", &lote, tps_l)] {
        let minimo = m.pagos * 2;
        let desperdicio = 100.0 * (m.generaciones.saturating_sub(minimo)) as f64
            / m.generaciones.max(1) as f64;
        println!(
            "| {n} | {} | {} | {minimo} | **{desperdicio:.0} %** | {} | **{:.2}** | {t:.2} |",
            m.pagos,
            m.generaciones,
            m.stale,
            m.regeneraciones_por_pago()
        );
    }

    println!();
    println!("== LECTURA ==");
    println!("  referencia §204 (secuencial, 4 hilos): 3,83 regen/pago · 66 % tirado");
    println!("  minimo posible en ambos modos: {limpias} generaciones ({} pagos x 2)", sec.pagos);
    println!(
        "  generaciones ahorradas por el lote: {}",
        sec.generaciones as i64 - lote.generaciones as i64
    );
    println!("  pagos/s: {tps_s:.2} -> {tps_l:.2}  ({:.2}x)", tps_l / tps_s.max(1e-9));
    println!();

    if lote.stale == 0 && lote.generaciones == limpias {
        println!("VEREDICTO: EL LOTE HACE LO QUE SE ESCRIBIO QUE HARIA.");
        println!("  Cero StaleState y cero regeneraciones: **ni una sola prueba");
        println!("  tirada**. Cada pago costo exactamente dos generaciones, que es");
        println!("  el minimo teorico.");
        if sec.generaciones > limpias {
            println!();
            println!("  El modo secuencial de esta misma corrida tiro {} generaciones",
                     sec.generaciones - limpias);
            println!("  ({:.0} % de su trabajo). Eso es lo que el lote elimina.",
                     100.0 * (sec.generaciones - limpias) as f64 / sec.generaciones as f64);
        } else {
            println!();
            println!("  ⚠️ Pero el modo secuencial NO desperdicio nada en esta corrida:");
            println!("     con esta carga no hubo contencion que quitar. La ventaja del");
            println!("     lote no se ve aqui — subir hilos y rondas para provocarla.");
        }
        println!();
        println!("  ⚠️ Los pagos/s NO son el rendimiento de un despliegue: todos los");
        println!("     clientes comparten esta maquina. La medida que decide es el");
        println!("     recuento de generaciones (§204, mismo aviso).");
    } else if lote.stale > 0 {
        println!("VEREDICTO: ⚠️ EL LOTE TUVO {} StaleState, Y NO DEBERIA TENER NINGUNO.", lote.stale);
        println!("  Por construccion todas las pruebas se generan contra la raiz que");
        println!("  el lote va a usar. Si aparece, hay algo aplicandose entremedias:");
        println!("  revisar el candado en `modo_lote`.");
    } else {
        println!("VEREDICTO: ⚠️ el lote gasto {} generaciones para {} pagos,",
                 lote.generaciones, lote.pagos);
        println!("  cuando el minimo son {limpias}. Sin StaleState, asi que el exceso");
        println!("  viene de otro sitio: revisar el banco antes de concluir nada.");
    }
    println!();
    println!("Anota esta tabla en AUDITORIA.md. Cierra el ciclo del RFC-0002.");
}
