//! E2E contra un nodo `--dev` vivo: el flujo de portada, con claves
//! ALEATORIAS (la via de produccion, no las deterministas del sandbox).
//!
//!   abrir x2 (solo ids derivados) -> dev_fund(alice) -> alice.pay ->
//!   bob.claim -> saldos por clave de VISTA.
//!
//! La clave de gasto no sale de este proceso: la unica linea donde
//! interviene es dentro de prove_send/prove_claim, en local.

use zk_ssl_sdk::{Account, Rpc, Wallet};

fn main() -> anyhow::Result<()> {
    let url = std::env::var("ZKSSL_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8545".into());
    let rpc = Rpc::new(url);

    let alice = Account::open(&rpc, Wallet::random())?;
    let bob = Account::open(&rpc, Wallet::random())?;
    println!("E2E: cuentas abiertas — alice #{} · bob #{}", alice.index, bob.index);

    rpc.dev_fund(alice.index, 1_000_000)?;
    println!("E2E: alice fondeada (grifo dev, dos custodios)");

    let notice = alice.pay(&bob.public_id(), 250_000)?;
    println!("E2E: FASE 1 aplicada — aviso en mano (viaja fuera de banda, §21)");

    bob.claim(&notice)?;
    println!("E2E: FASE 2 aplicada — bob cobro");

    let sa = alice.balance()?;
    let sb = bob.balance()?;
    println!("E2E: saldos por clave de VISTA — alice {sa} · bob {sb}");
    anyhow::ensure!(sa == 750_000, "alice deberia tener 750000, tiene {sa}");
    anyhow::ensure!(sb == 250_000, "bob deberia tener 250000, tiene {sb}");

    println!("E2E OK: pay+claim con claves aleatorias; la clave de gasto no viajo");
    Ok(())
}
