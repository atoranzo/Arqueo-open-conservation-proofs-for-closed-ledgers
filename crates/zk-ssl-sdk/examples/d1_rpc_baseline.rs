//! **Banco D.1 — el nodo por RPC, bajo concurrencia.**
//!
//! Todo lo medido en esta casa sobre concurrencia —§204 banco A.5, §216
//! banco B.1— se hizo **a nivel de capa**, con un `Mutex` en el proceso
//! de pruebas. **Nadie ha medido el nodo real por RPC**, y ahí hay dos
//! cosas que no estaban:
//!
//! 1. **El `Mutex` que `dispatch` mantiene durante toda la petición.**
//!    `main.rs:163` ya lo anticipa —«con carga real, el paso siguiente es
//!    una cola de escritura»— pero nadie ha comprobado si el cuello es la
//!    regeneración de pruebas o el candado.
//! 2. **El coste del cable.** Una prueba de envío son **65.840 bytes**
//!    (§204 banco A.4). En hex dentro de JSON son **~132 KB por
//!    `applySend`**, más la serialización en ambos extremos. Eso no está
//!    medido en ninguna parte.
//!
//! ## Por qué esto va ANTES de acumular en el nodo
//!
//! La propuesta (b) —un método que encola operaciones para aplicarlas en
//! lote— ataca la **contención**: pruebas que llegan muertas porque otro
//! aplicó mientras se generaban. §216 midió que el lote la elimina hasta
//! el mínimo teórico… **a nivel de capa**.
//!
//! **Si por RPC el cuello resulta ser el `Mutex` o el cable, acumular no
//! arregla nada**: las operaciones seguirían serializándose una a una en
//! la aplicación. Escribir (b) primero y descubrirlo después sería la
//! peor forma de averiguarlo.
//!
//! ## La hipótesis, escrita antes del dato
//!
//! Referencia de capa (§216, cuatro pares, misma máquina): **3,50
//! regeneraciones por pago y 64 % de trabajo tirado** en modo secuencial.
//!
//! - Si por RPC sale **parecido** → el cuello es la contención, y (b)
//!   está justificado con la misma medida que lo justificó en la capa.
//! - Si sale **mucho menos desperdicio pero menos operaciones/s** → el
//!   cuello es el **`Mutex` o el cable**, y (b) **no lo toca**. Habría
//!   que atacar el candado (cola de escritura) antes que los lotes.
//! - Si el desperdicio es **cero** → las peticiones ya se serializan de
//!   tal modo que ninguna prueba muere. Entonces (b) no tendría nada que
//!   eliminar, y el trabajo iría a otro sitio.
//!
//! ## Cómo se usa
//!
//! Necesita un nodo **ya en marcha** y en modo `--dev` (custodios de
//! prueba, para poder fondear):
//!
//! ```text
//! cargo run --release -p zk-ssl-node -- --dev --listen 127.0.0.1:8645 &
//! cargo run --release -p zk-ssl-sdk --example d1_rpc_baseline -- http://127.0.0.1:8645 4 2
//! ```
//! (url, clientes concurrentes, pagos por cliente)
//!
//! ⚠️ **Muta el estado del nodo** (abre cuentas, fondea, paga). El banco
//! lo levanta **sin `--ledger`**, asi que la capa vive en memoria y no
//! deja nada en disco — pero sigue haciendo falta un nodo desechable.
//!
//! ⚠️ **UNA corrida NO basta.** La dispersion medida en §218 es del
//! ±11 %: cuatro corridas dieron 2,62 · 3,25 · 2,62 · 3,12 regen/pago,
//! con 1,68 · 1,55 · 1,85 · 1,79 pagos/s.
//! Correr tres veces con nodo NUEVO cada vez y quedarse con la media.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

use zk_ssl_sdk::{Account, Rpc, Wallet};

struct Cuenta {
    reintentos: u64,
    generaciones: u64,
    pagos: u64,
}

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let url = args.next().unwrap_or_else(|| "http://127.0.0.1:8645".into());
    let clientes: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(4);
    let pagos: u32 = args.next().and_then(|s| s.parse().ok()).unwrap_or(2);

    eprintln!("== BANCO D.1 · el nodo por RPC, bajo concurrencia ==");
    eprintln!("   nodo: {url} · {clientes} clientes · {pagos} pagos cada uno");
    eprintln!("   referencia de CAPA (§216, secuencial): 3,50 regen/pago · 64 % tirado");
    eprintln!("   ⚠️ escribe en el ledger del nodo: usar uno desechable\n");

    // ── Montaje: un par de cuentas por cliente, fondeadas via dev_fund ──
    eprintln!("-- montaje --");
    let rpc = Rpc::new(url.clone());
    let version: serde_json::Value = rpc.call("zkssl_protocolVersion", serde_json::json!([]))?;
    eprintln!("   protocolo del nodo: {version}");

    let mut pares = Vec::new();
    for _ in 0..clientes {
        let wa = Wallet::random();
        let wb = Wallet::random();
        let a = Account::open(&rpc, wa)?;
        let b = Account::open(&rpc, wb)?;
        rpc.dev_fund(a.index, 1_000_000)?;
        rpc.dev_fund(b.index, 1_000_000)?;
        // `index` es un CAMPO publico, no un metodo (comprobado en lib.rs).
        pares.push((wa, a.index, wb, b.index, b.public_id()));
    }
    eprintln!("   {} pares abiertos y fondeados\n", pares.len());

    // ── La medida ──
    let gen = Arc::new(AtomicU64::new(0));
    let rei = Arc::new(AtomicU64::new(0));
    let hechos = Arc::new(AtomicU64::new(0));
    let t0 = Instant::now();

    std::thread::scope(|s| {
        for (wa, ia, wb, ib, id_b) in &pares {
            let (g, r, h) = (Arc::clone(&gen), Arc::clone(&rei), Arc::clone(&hechos));
            let url = url.clone();
            s.spawn(move || {
                // Cada cliente con su propia conexion: es lo que hace un
                // titular real, y lo que pone al Mutex del nodo a prueba.
                let rpc = Rpc::new(url);
                // `Wallet` es Clone+Copy (comprobado): se copia sin problema.
                let cuenta_a = Account::attach(&rpc, *wa, *ia);
                let cuenta_b = Account::attach(&rpc, *wb, *ib);

                for _ in 0..pagos {
                    // FASE 1 · pagar, reintentando si llega muerta.
                    let aviso = loop {
                        g.fetch_add(1, Ordering::Relaxed);
                        match cuenta_a.pay(id_b, 1_000) {
                            Ok(n) => break n,
                            Err(e) => {
                                // Un fallo por estado obsoleto es EL dato:
                                // la prueba se genero y llego muerta.
                                let msg = format!("{e}");
                                if msg.contains("Stale") || msg.contains("stale") {
                                    r.fetch_add(1, Ordering::Relaxed);
                                    continue;
                                }
                                panic!("pay: {msg}");
                            }
                        }
                    };
                    // FASE 2 · cobrar, igual.
                    loop {
                        g.fetch_add(1, Ordering::Relaxed);
                        match cuenta_b.claim(&aviso) {
                            Ok(()) => break,
                            Err(e) => {
                                let msg = format!("{e}");
                                if msg.contains("Stale") || msg.contains("stale") {
                                    r.fetch_add(1, Ordering::Relaxed);
                                    continue;
                                }
                                panic!("claim: {msg}");
                            }
                        }
                    }
                    h.fetch_add(1, Ordering::Relaxed);
                }
            });
        }
    });

    let m = Cuenta {
        reintentos: rei.load(Ordering::Relaxed),
        generaciones: gen.load(Ordering::Relaxed),
        pagos: hechos.load(Ordering::Relaxed),
    };
    let segundos = t0.elapsed().as_secs_f64();
    let minimo = m.pagos * 2;
    let desperdicio = 100.0 * (m.generaciones.saturating_sub(minimo)) as f64
        / m.generaciones.max(1) as f64;
    let regen = (m.generaciones.saturating_sub(minimo)) as f64 / m.pagos.max(1) as f64;
    let tps = m.pagos as f64 / segundos.max(1e-9);

    println!();
    println!("| via | clientes | pagos | generaciones | minimo | desperdicio | regen/pago | pagos/s |");
    println!("|---|---|---|---|---|---|---|---|");
    println!(
        "| **RPC** | {clientes} | {} | {} | {minimo} | **{desperdicio:.0} %** | **{regen:.2}** | **{tps:.2}** |",
        m.pagos, m.generaciones
    );
    println!("| capa (§216, ref) | 4 | 8 | 44 | 16 | 64 % | 3,50 | 1,62 |");

    println!();
    println!("== LECTURA ==");
    println!("  reintentos por estado obsoleto: {}", m.reintentos);
    println!("  segundos: {segundos:.1}");
    println!("  ⚠️ el cable NO se mide aparte: una prueba son 65.840 bytes,");
    println!("     ~132 KB en hex dentro del JSON de cada applySend.");
    println!();

    println!("== LINEA BASE MEDIDA (§218, media de cuatro corridas) ==");
    println!("  2,90 ± 0,33 regen/pago · 59 % de desperdicio · 1,72 ± 0,13 pagos/s");
    println!("  Referencia de CAPA (§216, secuencial): 3,50 regen/pago · 1,62 pagos/s.");
    println!("  La diferencia entre las dos NO es demostrable: cae dentro del ±11 %.");
    println!();
    println!("  Esta corrida: {regen:.2} regen/pago · {tps:.2} pagos/s.");
    if regen > 1.5 {
        println!("  Consistente con la contencion medida. (b) sigue justificado.");
    } else {
        println!("  ⚠️ POR DEBAJO de la linea base. Con UNA corrida eso no decide");
        println!("     nada: repetir tres veces con nodo nuevo antes de concluir.");
    }
    println!();
    println!("  ⚠️ NO se concluye de una sola corrida. El banco imprime la fila;");
    println!("     el veredicto sale de la MEDIA. Correr:");
    println!("       for i in 1 2 3; do <nodo nuevo>; <este banco>; done");
    println!();
    println!("Anota esta tabla en AUDITORIA.md. Es la linea base del nodo por RPC:");
    println!("no existia, y sin ella (b) se construiria sin nada contra que comparar.");
    Ok(())
}
