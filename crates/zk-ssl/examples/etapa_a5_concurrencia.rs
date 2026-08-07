//! **Etapa A.5 del RFC-0002 — ¿existe la contención, o solo falta paralelismo?**
//!
//! Los 1,5-1,9 TPS de `AUDITORIA.md` §123 admiten **dos explicaciones
//! distintas que dan el mismo número**, y el RFC-0002 se apoya en una de
//! ellas sin haberla separado de la otra:
//!
//! - **Historia A (la escrita).** Bajo concurrencia las pruebas mueren
//!   porque la raíz cambia mientras se generan, y hay regeneraciones en
//!   cascada. Habría un desperdicio de ~7×, y los lotes lo eliminarían.
//! - **Historia B (la aritmética de hoy).** 250 ms de envío + 33 de
//!   `apply` + 237 de cobro + 33 = **553 ms en SERIE** = 1,81 pagos/s.
//!   Sin desperdicio ninguno: sencillamente nadie va en paralelo.
//!
//! Este banco las separa contando **cuántas veces una prueba llega
//! muerta** (`StaleState`).
//!
//! ## Diseño
//!
//! `T` hilos sobre una capa compartida (`Arc<Mutex<…>>`, como el nodo).
//! **Cada hilo opera su propio par de cuentas**, así que no hay conflicto
//! lógico entre ellos: lo único que pueden disputarse es la RAÍZ.
//!
//! Cada pago: materiales (con candado) → **generar la prueba SIN
//! candado** → aplicar (con candado). Si sale `StaleState`, se rehacen
//! materiales y se regenera, contándolo.
//!
//! ## Cómo se lee
//!
//! | resultado | lectura |
//! |---|---|
//! | **muchos `StaleState`** | historia A confirmada: hay desperdicio real y los lotes lo eliminan |
//! | **casi ninguno**, y el tiempo total cae al añadir hilos | historia B: no hay contención, solo faltaba paralelismo — el arreglo es más barato de lo previsto |
//! | casi ninguno, y el tiempo **no** mejora con hilos | hay un cuello distinto (probablemente CPU: la generación ya usa todos los núcleos) |
//!
//! ⚠️ **Aviso de método.** Todos los "clientes" comparten una sola
//! máquina, así que si la generación de pruebas ya usa varios núcleos,
//! añadir hilos **no** multiplicará el rendimiento aunque no haya
//! contención. Por eso la medida que decide es el **recuento de
//! `StaleState`**, no los TPS observados.
//!
//! Uso:
//! ```text
//! cargo run --release -p zk-ssl --features sandbox --example etapa_a5_concurrencia -- 4 3
//! ```
//! (hilos, pagos por hilo)

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use zk_ssl::commitment::ClientState;
use zk_ssl::tests_support as ts;
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

/// Resultado de una corrida con `hilos` hilos.
struct Corrida {
    hilos: usize,
    pagos: u64,
    stale_send: u64,
    stale_claim: u64,
    segundos: f64,
}

fn corrida(hilos: usize, pagos_por_hilo: u32) -> Corrida {
    // ── Montaje: un par de cuentas por hilo, todas fondeadas ──
    let mut layer = SovereignLayer::new(
        ts::custodian_root(),
        ts::governance_root(),
        ts::LIMIT,
        ts::MAX_SUPPLY,
        ts::MAX_ACCOUNTS,
    );

    let mut pares = Vec::new();
    for h in 0..hilos as u64 {
        let ka = ts::wide_key(0xA11CE + h * 2);
        let kb = ts::wide_key(0xA11CE + h * 2 + 1);
        let a = layer.open_account_wide(ka);
        let b = layer.open_account_wide(kb);
        fondear(&mut layer, a, 1_000_000);
        fondear(&mut layer, b, 1_000_000);
        pares.push((a, b, ka, kb));
    }

    let capa = Arc::new(Mutex::new(layer));
    let stale_send = Arc::new(AtomicU64::new(0));
    let stale_claim = Arc::new(AtomicU64::new(0));
    let hechos = Arc::new(AtomicU64::new(0));

    let t0 = Instant::now();

    std::thread::scope(|s| {
        for (h, &(a, b, ka, kb)) in pares.iter().enumerate() {
            let capa = Arc::clone(&capa);
            let ss = Arc::clone(&stale_send);
            let sc = Arc::clone(&stale_claim);
            let hh = Arc::clone(&hechos);
            s.spawn(move || {
                for i in 0..pagos_por_hilo {
                    let importe = 1_000u64;
                    let salt = ts::salt_de(1_000 * (h as u64 + 1) + i as u64);

                    // ── FASE 1: enviar, con reintentos si llega muerta ──
                    let envio = loop {
                        let (mats, est) = {
                            let g = capa.lock().unwrap();
                            let est = estado(&g, a);
                            let receptor = g.public_id_of(b).expect("receptor");
                            let m = g
                                .send_materials(a, receptor, importe, salt)
                                .expect("materiales de envio");
                            (m, est)
                        };
                        // ⚠️ SIN candado: es donde se va el tiempo, y es
                        // donde la raiz puede cambiarle debajo.
                        let envio = client::prove_send(&mats, ka, proof_options())
                            .expect("prueba de envio");

                        let r = {
                            let mut g = capa.lock().unwrap();
                            g.apply_send(&envio, a, &est, importe)
                        };
                        match r {
                            Ok(()) => break envio,
                            Err(LayerError::StaleState) => {
                                ss.fetch_add(1, Ordering::Relaxed);
                                continue;
                            }
                            Err(e) => panic!("apply_send: {e:?}"),
                        }
                    };

                    // ── FASE 2: cobrar, igual ──
                    loop {
                        let (mats, est) = {
                            let g = capa.lock().unwrap();
                            let est = estado(&g, b);
                            let m = g
                                .claim_materials(b, &envio.notice)
                                .expect("materiales de cobro");
                            (m, est)
                        };
                        let cobro = client::prove_claim(&mats, kb, proof_options())
                            .expect("prueba de cobro");
                        let r = {
                            let mut g = capa.lock().unwrap();
                            g.apply_claim(&cobro, b, &est, &envio.notice)
                        };
                        match r {
                            Ok(()) => break,
                            Err(LayerError::StaleState) => {
                                sc.fetch_add(1, Ordering::Relaxed);
                                continue;
                            }
                            Err(e) => panic!("apply_claim: {e:?}"),
                        }
                    }

                    hh.fetch_add(1, Ordering::Relaxed);
                }
            });
        }
    });

    Corrida {
        hilos,
        pagos: hechos.load(Ordering::Relaxed),
        stale_send: stale_send.load(Ordering::Relaxed),
        stale_claim: stale_claim.load(Ordering::Relaxed),
        segundos: t0.elapsed().as_secs_f64(),
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let hilos_max: usize = args
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(4);
    let pagos_por_hilo: u32 = args
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3);

    eprintln!("== ETAPA A.5 · RFC-0002 · ¿contencion real o falta de paralelismo? ==");
    eprintln!("   nucleos disponibles: {}", std::thread::available_parallelism()
        .map(|n| n.get()).unwrap_or(0));
    eprintln!("   cada hilo opera SU PROPIO par de cuentas: sin conflicto logico.");
    eprintln!("   lo unico disputable es la RAIZ.\n");

    let mut filas = Vec::new();
    for hilos in [1usize, hilos_max] {
        if hilos == 0 {
            continue;
        }
        eprintln!("-- {hilos} hilo(s), {pagos_por_hilo} pagos cada uno --");
        let c = corrida(hilos, pagos_por_hilo);
        eprintln!(
            "   {} pagos en {:.1} s · stale send {} · stale claim {}",
            c.pagos, c.segundos, c.stale_send, c.stale_claim
        );
        filas.push(c);
        if hilos_max == 1 {
            break;
        }
    }

    println!();
    println!("| hilos | pagos | segundos | pagos/s | StaleState (send) | StaleState (claim) | regeneraciones por pago |");
    println!("|---|---|---|---|---|---|---|");
    for c in &filas {
        let tps = c.pagos as f64 / c.segundos.max(1e-9);
        let regen = (c.stale_send + c.stale_claim) as f64 / c.pagos.max(1) as f64;
        println!(
            "| {} | {} | {:.1} | **{tps:.2}** | {} | {} | {regen:.2} |",
            c.hilos, c.pagos, c.segundos, c.stale_send, c.stale_claim
        );
    }

    println!();
    println!("== LECTURA ==");
    let total_stale: u64 = filas.iter().map(|c| c.stale_send + c.stale_claim).sum();
    let total_pagos: u64 = filas.iter().map(|c| c.pagos).sum();
    let regen = total_stale as f64 / total_pagos.max(1) as f64;
    println!("  regeneraciones por pago (global): {regen:.2}");

    if filas.len() == 2 {
        let uno = &filas[0];
        let varios = &filas[1];
        let tps1 = uno.pagos as f64 / uno.segundos.max(1e-9);
        let tpsn = varios.pagos as f64 / varios.segundos.max(1e-9);
        println!("  pagos/s con 1 hilo ....... {tps1:.2}");
        println!("  pagos/s con {} hilos ...... {tpsn:.2}  ({:.2}x)", varios.hilos, tpsn / tps1);
        println!();

        if regen > 0.5 {
            println!("VEREDICTO: HISTORIA A — la contencion EXISTE ({regen:.2} regeneraciones/pago).");
            println!("  -> Hay desperdicio real: pruebas generadas que llegan muertas.");
            println!("  -> La etapa D (lotes) esta justificada por lo que dice el RFC.");
        } else if tpsn / tps1 > 1.5 {
            println!("VEREDICTO: HISTORIA B — casi no hay contencion, faltaba PARALELISMO.");
            println!("  -> Los hilos escalan sin que las pruebas mueran.");
            println!("  -> Los lotes NO 'quitan contencion' (no la hay): habilitan");
            println!("     generacion concurrente. Se justifican distinto, y puede");
            println!("     bastar con servir peticiones en paralelo. REESCRIBIR el RFC.");
        } else {
            println!("VEREDICTO: NI A NI B — sin apenas StaleState y sin ganancia por hilos.");
            println!("  -> El cuello es otro. Sospechoso principal: la generacion ya");
            println!("     usa todos los nucleos, asi que varios 'clientes' en UNA");
            println!("     maquina no se suman. Repetir con clientes en maquinas");
            println!("     distintas, o medir el uso de CPU de una sola generacion.");
        }
    }

    println!();
    println!("⚠️ Recordatorio: todos los clientes comparten esta maquina. Los");
    println!("   pagos/s de arriba NO son el rendimiento de un despliegue real;");
    println!("   la medida que decide es el recuento de StaleState.");
    println!();
    println!("Anota esta tabla en AUDITORIA.md junto a las de A, A.2, A.3 y A.4.");
}
