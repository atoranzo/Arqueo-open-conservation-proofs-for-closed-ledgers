//! Ejercicio vivo del keystore: guardar, cargar, y que lo malo FALLE.
//! Corre sin nodo: cargo run --release -p zk-ssl-sdk --example keystore

use zk_ssl_sdk::{keystore, Wallet};

fn main() -> anyhow::Result<()> {
    let ruta = std::env::temp_dir().join("zkssl_keystore_demo.json");
    let w = Wallet::random();

    keystore::save(&ruta, &w, "contrasena de demo")?;
    println!("KS: wallet guardado cifrado en {}", ruta.display());

    let w2 = keystore::load(&ruta, "contrasena de demo")?;
    anyhow::ensure!(w.public_id() == w2.public_id(), "ids distintos tras cargar");
    println!("KS: cargado — public_id identico (la clave volvio intacta)");

    match keystore::load(&ruta, "contrasena MALA") {
        Err(e) => println!("KS: contrasena mala rechazada: {e}"),
        Ok(_) => anyhow::bail!("CRITICO: una contrasena mala abrio el keystore"),
    }

    let _ = std::fs::remove_file(&ruta);
    println!("KEYSTORE OK: la clave duerme con la ley de reposo del ledger y dominio propio");
    Ok(())
}
