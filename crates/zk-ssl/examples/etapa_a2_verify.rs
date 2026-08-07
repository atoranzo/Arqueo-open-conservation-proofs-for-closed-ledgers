//! **Etapa A.2 del RFC-0002 — ¿verificación o árboles?**
//!
//! La etapa A midió que el `apply` cuesta ~38 ms y que **la persistencia
//! es solo el 3 %**: el cómputo domina. Este banco parte ese cómputo en
//! dos, porque de la respuesta depende TODO el plan:
//!
//! | si domina | consecuencia para el RFC |
//! |---|---|
//! | **verificación** | los lotes quitan la contención pero **no** el coste: N pruebas siguen siendo N verificaciones. El múltiplo barato pasa a ser **verificación en paralelo**, sin circuitos nuevos |
//! | **árboles** | los lotes dan un salto grande: N operaciones, **una sola** actualización de árbol. La etapa D se justifica sola |
//!
//! Método: se cronometra la verificación STARK **exactamente igual que la
//! hace la capa** —mismo `verify`, mismo AIR, mismas opciones— y se resta
//! del `apply` completo medido en la misma corrida y la misma máquina.
//!
//! No modifica la biblioteca ni `Cargo.toml`: `stark-experiment` y
//! `winterfell` ya son dependencias de `zk-ssl`, y los ejemplos del
//! paquete pueden usarlas.
//!
//! Uso:
//! ```text
//! cargo run --release -p zk-ssl --features sandbox --example etapa_a2_verify -- 15
//! ```

use std::time::{Duration, Instant};

use winterfell::crypto::hashers::Blake3_256;
use winterfell::crypto::{DefaultRandomCoin, MerkleTree};
use winterfell::math::fields::f64::BaseElement;
use winterfell::{verify, AcceptableOptions};

use stark_experiment::circuit_claim::ClaimAir;
use stark_experiment::circuit_send::SendAir;

use zk_ssl::commitment::ClientState;
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

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

fn main() {
    let n: u32 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(15);

    eprintln!("== ETAPA A.2 · RFC-0002 · verificacion vs arboles ==");
    eprintln!("   pagos: {n} · en memoria (la persistencia ya se descarto en A: 3%)\n");

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

    let opciones = AcceptableOptions::OptionSet(vec![proof_options()]);

    let mut v_send = Duration::ZERO;
    let mut v_claim = Duration::ZERO;
    let mut a_send = Duration::ZERO;
    let mut a_claim = Duration::ZERO;

    for i in 0..n {
        let importe = 1_000u64;

        // ───── FASE 1 ─────
        let est_a = estado(&layer, a);
        let receptor = layer.public_id_of(b).expect("receptor");
        let materiales = layer
            .send_materials(a, receptor, importe, ts::salt_de(7 + i as u64))
            .expect("materiales de envio");
        let envio = client::prove_send(&materiales, clave_a, proof_options())
            .expect("prueba de envio");

        // (1) SOLO la verificacion, igual que la hace la capa.
        let proof = winterfell::Proof::from_bytes(&envio.proof).expect("prueba bien formada");
        let t = Instant::now();
        verify::<SendAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof,
            envio.public_inputs.clone(),
            &opciones,
        )
        .expect("verificacion de envio");
        v_send += t.elapsed();

        // (2) El apply COMPLETO (que vuelve a verificar: es lo que mide A).
        let t = Instant::now();
        layer
            .apply_send(&envio, a, &est_a, importe)
            .expect("apply_send");
        a_send += t.elapsed();

        // ───── FASE 2 ─────
        let est_b = estado(&layer, b);
        let materiales = layer
            .claim_materials(b, &envio.notice)
            .expect("materiales de cobro");
        let cobro = client::prove_claim(&materiales, clave_b, proof_options())
            .expect("prueba de cobro");

        let proof = winterfell::Proof::from_bytes(&cobro.proof).expect("prueba bien formada");
        let t = Instant::now();
        verify::<ClaimAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof,
            cobro.public_inputs.clone(),
            &opciones,
        )
        .expect("verificacion de cobro");
        v_claim += t.elapsed();

        let t = Instant::now();
        layer
            .apply_claim(&cobro, b, &est_b, &envio.notice)
            .expect("apply_claim");
        a_claim += t.elapsed();

        if (i + 1) % 5 == 0 {
            eprintln!("    … {}/{}", i + 1, n);
        }
    }

    let d = n.max(1);
    let vs = ms(v_send / d);
    let vc = ms(v_claim / d);
    let as_ = ms(a_send / d);
    let ac = ms(a_claim / d);

    println!();
    println!("| fase | verificacion sola | apply completo | resto (arboles + log) |");
    println!("|---|---|---|---|");
    println!("| send  | {vs:.2} ms | {as_:.2} ms | **{:.2} ms** |", as_ - vs);
    println!("| claim | {vc:.2} ms | {ac:.2} ms | **{:.2} ms** |", ac - vc);

    let verif = (vs + vc) / 2.0;
    let total = (as_ + ac) / 2.0;
    let resto = total - verif;

    println!();
    println!("== DESCOMPOSICION del apply (media por operacion) ==");
    println!("  verificacion STARK ........... {verif:.2} ms  ({:.0}%)", 100.0 * verif / total);
    println!("  arboles + log + resto ........ {resto:.2} ms  ({:.0}%)", 100.0 * resto / total);
    println!("  TOTAL ........................ {total:.2} ms");
    println!();

    if verif > resto {
        println!("VEREDICTO: domina la VERIFICACION ({:.0}%).", 100.0 * verif / total);
        println!("  -> Los lotes NO bajan este coste: N pruebas son N verificaciones.");
        println!("  -> El multiplo barato es VERIFICACION EN PARALELO (x nucleos),");
        println!("     sin circuitos nuevos. Redefinir la etapa B del RFC-0002.");
        println!("  -> La etapa D sigue justificada, pero por QUITAR CONTENCION,");
        println!("     no por bajar el coste por operacion.");
    } else {
        println!("VEREDICTO: dominan los ARBOLES ({:.0}%).", 100.0 * resto / total);
        println!("  -> Los lotes SI bajan el coste: N operaciones, una actualizacion.");
        println!("  -> La etapa D del RFC-0002 se justifica sola. Dimensionarla con");
        println!("     el circuito de lote (N ascensos de Merkle en una traza).");
    }
    println!();
    println!("⚠️ Nota de metodo: el apply verifica TAMBIEN, asi que 'resto' ya");
    println!("   descuenta una verificacion. Si 'resto' saliera negativo, la");
    println!("   medida esta mal y no debe usarse.");
    println!();
    println!("Anota esta tabla en AUDITORIA.md junto a la de la etapa A.");
}
