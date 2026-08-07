//! **Etapa A.4 del RFC-0002 — el coste plano del `apply`, aislado.**
//!
//! Cadena de mediciones hasta aquí:
//!
//! | etapa | resultado |
//! |---|---|
//! | A | `apply` ~38 ms · persistencia **3 %** (hipótesis del `flush`: refutada) |
//! | A.2 | verificación **7 %** · árboles y resto **93 %** |
//! | A.3 | exponente en cuentas **0,18** → plano (hipótesis del árbol cuadrático: refutada) |
//!
//! Queda un coste de ~31 ms que **no** depende del número de cuentas.
//! Este banco pone a prueba el último candidato.
//!
//! ## La hipótesis
//!
//! `log::digest_of_proof` recorre la prueba en bloques de 16 bytes y hace
//! **una permutación Rescue (`native_merge`) por bloque**. Una prueba de
//! ~36-65 KB son miles de permutaciones.
//!
//! Y `proof_digest` **no entra en ningún circuito**: no aparece en
//! `stark-experiment`. Se estaría pagando un hash algebraico —caro, y
//! elegido por ser amigable con circuitos— para un resumen que nunca va a
//! demostrarse dentro de uno.
//!
//! ## Qué mide, y qué la refuta
//!
//! Cronometra `digest_of_proof` sobre **los bytes de una prueba real**, y
//! Blake3 sobre **los mismos bytes**, en la misma máquina.
//!
//! - `digest_of_proof` ≈ **30 ms** → confirmada: es el coste que falta.
//! - `digest_of_proof` ≪ 10 ms → **refutada**: el coste está en otro
//!   sitio y hay que seguir midiendo antes de tocar nada.
//!
//! ## Este banco ya cumplió su encargo — léelo antes de interpretarlo
//!
//! **En §204 confirmó la hipótesis**: `digest_of_proof` era el **93 %**
//! del `apply` (30,99 ms de 33,27), **2.915×** lo que costaba Blake3
//! sobre los mismos bytes, para un resumen que **no entra en ningún
//! circuito**.
//!
//! **En §209 se hizo el cambio** (etapa 1 del RFC-0002): `digest_of_proof`
//! pasó a Blake3 con dominio y longitud explícita, la versión del
//! protocolo subió a `zkssl/0.2` y se emitieron vectores nuevos
//! conservando los de `0.1`. El `apply` bajó de 33,27 a ~3,3 ms.
//!
//! **Desde §209 este banco es una COMPUERTA DE REGRESIÓN**: si alguien
//! devolviera un hash algebraico a `digest_of_proof`, su coste volvería a
//! dominar el `apply` y esta salida lo diría.
//!
//! ⚠️ La columna «Blake3 sobre los mismos bytes» es la **referencia**:
//! desde §209 debería parecerse mucho a la primera, porque ambas hacen
//! ya el mismo trabajo. Si se separan otra vez, algo cambió.
//!
//! Uso:
//! ```text
//! cargo run --release -p zk-ssl --features sandbox --example etapa_a4_hash
//! ```

use std::time::Instant;

use winterfell::crypto::hashers::Blake3_256;
use winterfell::crypto::Hasher;
use winterfell::math::fields::f64::BaseElement;

use zk_ssl::commitment::ClientState;
use zk_ssl::log::digest_of_proof;
use zk_ssl::tests_support as ts;
use zk_ssl::{client, proof_options, AccountIndex, SovereignLayer};

type Blake3 = Blake3_256<BaseElement>;

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

fn main() {
    const REPS: u32 = 20;

    eprintln!("== ETAPA A.4 · RFC-0002 · el coste plano, aislado ==");
    eprintln!("   generando UNA prueba real para medir sobre sus bytes…\n");

    // Una prueba real, del tamaño real. Generarla no se cronometra.
    let mut layer = SovereignLayer::new(
        ts::custodian_root(),
        ts::governance_root(),
        ts::LIMIT,
        ts::MAX_SUPPLY,
        ts::MAX_ACCOUNTS,
    );
    let clave_a = ts::wide_key(0xA11CE);
    let clave_b = ts::wide_key(0xA11CE + 1);
    let a = layer.open_account_wide(clave_a);
    let b = layer.open_account_wide(clave_b);
    fondear(&mut layer, a, 1_000_000);
    fondear(&mut layer, b, 1_000_000);

    let est_a = estado(&layer, a);
    let receptor = layer.public_id_of(b).expect("receptor");
    let materiales = layer
        .send_materials(a, receptor, 1_000, ts::salt_de(7))
        .expect("materiales");
    let envio = client::prove_send(&materiales, clave_a, proof_options()).expect("prueba");
    let bytes = envio.proof.clone();

    // El apply completo, como referencia de la misma corrida.
    let t = Instant::now();
    layer
        .apply_send(&envio, a, &est_a, 1_000)
        .expect("apply_send");
    let ms_apply = t.elapsed().as_secs_f64() * 1000.0;

    let n_bloques = bytes.len().div_ceil(16);

    // ── 1. digest_of_proof, tal como este arbol lo implemente HOY ──
    let t = Instant::now();
    let mut d1 = [BaseElement::from(0u32); 4];
    for _ in 0..REPS {
        d1 = digest_of_proof(&bytes);
    }
    let ms_rescue = t.elapsed().as_secs_f64() * 1000.0 / REPS as f64;

    // ── 2. Blake3 sobre LOS MISMOS bytes (solo referencia) ──
    let t = Instant::now();
    let mut d2 = Blake3::hash(&bytes);
    for _ in 0..REPS {
        d2 = Blake3::hash(&bytes);
    }
    let ms_blake = t.elapsed().as_secs_f64() * 1000.0 / REPS as f64;

    // Que el optimizador no elimine el trabajo.
    std::hint::black_box((&d1, &d2));

    println!();
    println!("| medida | valor |");
    println!("|---|---|");
    println!("| tamano de la prueba | **{} bytes** ({n_bloques} bloques de 16) |", bytes.len());
    println!("| `digest_of_proof` (implementacion actual) | **{ms_rescue:.3} ms** |");
    println!("| Blake3 sobre los mismos bytes | **{ms_blake:.3} ms** |");
    println!("| `apply_send` completo (esta corrida) | {ms_apply:.2} ms |");

    println!();
    println!("== LECTURA ==");
    let pct = 100.0 * ms_rescue / ms_apply;
    println!("  digest_of_proof / apply .... {pct:.0}% del coste del apply");
    println!("  actual / Blake3 ............ {:.1}x", ms_rescue / ms_blake.max(1e-9));
    let ahorro = ms_rescue - ms_blake;
    let apply_nuevo = ms_apply - ahorro;
    println!("  apply si se cambiara ....... {apply_nuevo:.2} ms (de {ms_apply:.2})");
    if apply_nuevo > 0.0 {
        println!("  techo implicito ............ {:.0} operaciones/s (de {:.0})",
                 1000.0 / apply_nuevo, 1000.0 / ms_apply);
    }
    println!();

    println!("== DIAGNOSTICO ==");
    println!();
    if pct > 50.0 {
        println!("⚠️ REGRESION: digest_of_proof es el {pct:.0}% del apply.");
        println!("   Referencia: en §204 era el 93% (Rescue, 30,99 ms) y en §209 se");
        println!("   cambio a Blake3 con dominio y longitud (etapa 1 del RFC-0002).");
        println!("   Si esta cifra vuelve a estar arriba, alguien devolvio un hash");
        println!("   ALGEBRAICO a `log::digest_of_proof`. Mirar ahi.");
        println!("   Recordatorio: `chain_digest` SI debe seguir en Rescue.");
    } else if pct < 10.0 {
        println!("✅ El cambio de §209 esta en su sitio: {pct:.0}% del apply.");
        println!("   Referencia: 93% en §204, antes de la etapa 1.");
        println!("   El apply de esta corrida ({ms_apply:.2} ms) ya no lo domina el");
        println!("   resumen de la prueba. Lo que queda del RFC-0002 es la etapa 2");
        println!("   (transicion de hoja y lote), que ataca la CONTENCION — otra");
        println!("   cosa distinta, y NO se arregla desde aqui.");
    } else {
        println!("⚠️ INTERMEDIO: {pct:.0}% del apply.");
        println!("   Ni el 93% de §204 ni el residuo esperado tras §209. Medir de");
        println!("   nuevo antes de concluir: puede ser ruido de una corrida corta.");
    }
    println!();
    println!("Referencias: AUDITORIA.md §204 (las cinco tablas) y §209 (etapa 1).");
    println!();
    println!("Anota esta tabla en AUDITORIA.md junto a las de A, A.2 y A.3.");
}
