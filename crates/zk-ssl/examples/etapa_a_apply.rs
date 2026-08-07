//! **Etapa A del RFC-0002 — instrumentación del `apply`.**
//!
//! Responde la única pregunta que el RFC declara previa a todo lo demás:
//! **¿dónde se van los ~175 ms de un `apply`?**
//!
//! No toca la biblioteca. Mide la MISMA carga sobre tres soportes de
//! persistencia distintos y resta:
//!
//! | soporte | qué incluye |
//! |---|---|
//! | memoria (`SovereignLayer::new`) | verificación de la prueba + árboles |
//! | tmpfs (`open` sobre `/dev/shm`) | lo anterior + serializar y `sled`, sin disco físico |
//! | disco (`open` sobre el sistema de ficheros) | lo anterior + durabilidad física (`flush`) |
//!
//! Las restas dan la tabla que decide el RFC:
//!
//! - `memoria`                 → verificación + árboles
//! - `tmpfs − memoria`         → serialización y `sled`
//! - `disco − tmpfs`           → **coste de la durabilidad por operación**
//!
//! Si la tercera fila domina, la etapa B (group commit) es el múltiplo
//! barato y las etapas C/D pueden no hacer falta para el objetivo de
//! media. Si domina la primera, hay que ir directo a la D.
//!
//! **La generación de pruebas NO se cronometra**: es del cliente y ya
//! escala sola (RFC-0002, motivación). Aquí solo se mide `apply_*`.
//!
//! Uso:
//! ```text
//! cargo run --release -p zk-ssl --features sandbox --example etapa_a_apply -- 20
//! ```

use std::time::{Duration, Instant};

use zk_ssl::commitment::ClientState;
use zk_ssl::tests_support as ts;
use zk_ssl::{client, proof_options, AccountIndex, SovereignLayer};

/// Lo que un titular conoce de su cuenta. Aquí se lee de la capa por
/// comodidad del banco; en un despliegue lo custodia el cliente.
fn estado(layer: &SovereignLayer, idx: AccountIndex) -> ClientState {
    ClientState {
        public_id: layer.public_id_of(idx).expect("cuenta abierta"),
        balance: layer.balance_of(idx).expect("cuenta abierta"),
        nonce: layer.nonce_of(idx).expect("cuenta abierta"),
    }
}

/// Emisión delegada: DOS custodios, como en la suite.
fn fondear(layer: &mut SovereignLayer, idx: AccountIndex, importe: u64) {
    let op = ts::mint_commitment(layer, idx, importe);
    let subida = ts::mint_climb_proof(layer, idx, importe);
    let (pa, ia, pb, ib) = ts::delegated_pair(op, 1, 3);
    layer
        .apply_mint_delegated(subida, pa, ia, pb, ib, idx, importe)
        .expect("fondear");
}

#[derive(Default)]
struct Medida {
    send: Duration,
    claim: Duration,
    n: u32,
}

impl Medida {
    /// Media por operación de capa (send y claim son operaciones distintas).
    fn media_por_operacion(&self) -> Duration {
        let total = self.send + self.claim;
        total / (2 * self.n.max(1))
    }
    fn ms(d: Duration) -> f64 {
        d.as_secs_f64() * 1000.0
    }
}

/// Una corrida completa sobre un soporte. `ruta = None` → memoria.
fn corrida(ruta: Option<&str>, n: u32) -> Medida {
    if let Some(p) = ruta {
        let _ = std::fs::remove_dir_all(p);
    }

    let mut layer = match ruta {
        Some(p) => SovereignLayer::open(
            p,
            ts::custodian_root(),
            ts::governance_root(),
            ts::LIMIT,
            ts::MAX_SUPPLY,
            ts::MAX_ACCOUNTS,
        )
        .expect("abrir ledger"),
        None => SovereignLayer::new(
            ts::custodian_root(),
            ts::governance_root(),
            ts::LIMIT,
            ts::MAX_SUPPLY,
            ts::MAX_ACCOUNTS,
        ),
    };

    // Montaje (no se cronometra).
    let clave_a = ts::wide_key(0xA11CE);
    let clave_b = ts::wide_key(0xA11CE + 1);
    let a = layer.open_account_wide(clave_a);
    let b = layer.open_account_wide(clave_b);
    fondear(&mut layer, a, 1_000_000);
    fondear(&mut layer, b, 1_000_000);

    let mut m = Medida { n, ..Default::default() };

    for i in 0..n {
        let importe = 1_000u64;

        // --- FASE 1: enviar ---
        let est_a = estado(&layer, a);
        let receptor = layer.public_id_of(b).expect("receptor");
        let materiales = layer
            .send_materials(a, receptor, importe, ts::salt_de(7 + i as u64))
            .expect("materiales de envio");
        // La prueba es del cliente: NO se cronometra.
        let envio = client::prove_send(&materiales, clave_a, proof_options())
            .expect("prueba de envio");

        let t = Instant::now();
        layer
            .apply_send(&envio, a, &est_a, importe)
            .expect("apply_send");
        m.send += t.elapsed();

        // --- FASE 2: cobrar ---
        let est_b = estado(&layer, b);
        let materiales = layer
            .claim_materials(b, &envio.notice)
            .expect("materiales de cobro");
        let cobro = client::prove_claim(&materiales, clave_b, proof_options())
            .expect("prueba de cobro");

        let t = Instant::now();
        layer
            .apply_claim(&cobro, b, &est_b, &envio.notice)
            .expect("apply_claim");
        m.claim += t.elapsed();

        if (i + 1) % 5 == 0 {
            eprintln!("    … {}/{}", i + 1, n);
        }
    }

    if let Some(p) = ruta {
        let _ = std::fs::remove_dir_all(p);
    }
    m
}

fn main() {
    let n: u32 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(20);

    eprintln!("== ETAPA A · RFC-0002 · instrumentacion del apply ==");
    eprintln!("   pagos por soporte: {n} (cada pago son DOS operaciones)");
    eprintln!("   la generacion de pruebas NO se cronometra\n");

    let tmpfs = if std::path::Path::new("/dev/shm").is_dir() {
        Some("/dev/shm/zkssl_etapa_a")
    } else {
        None
    };

    eprintln!("-- 1/3 memoria --");
    let mem = corrida(None, n);
    eprintln!("-- 2/3 tmpfs --");
    let tm = tmpfs.map(|p| corrida(Some(p), n));
    eprintln!("-- 3/3 disco --");
    let disco = corrida(Some("./target/zkssl_etapa_a"), n);

    println!();
    println!("| soporte | apply_send | apply_claim | media/operacion |");
    println!("|---|---|---|---|");
    let fila = |nombre: &str, m: &Medida| {
        println!(
            "| {} | {:.2} ms | {:.2} ms | **{:.2} ms** |",
            nombre,
            Medida::ms(m.send / m.n.max(1)),
            Medida::ms(m.claim / m.n.max(1)),
            Medida::ms(m.media_por_operacion()),
        );
    };
    fila("memoria", &mem);
    if let Some(t) = &tm {
        fila("tmpfs", t);
    } else {
        println!("| tmpfs | (no hay /dev/shm) | | |");
    }
    fila("disco", &disco);

    println!();
    println!("== DESCOMPOSICION ==");
    let ms_mem = Medida::ms(mem.media_por_operacion());
    let ms_disco = Medida::ms(disco.media_por_operacion());
    println!("  verificacion + arboles ....... {ms_mem:.2} ms");
    if let Some(t) = &tm {
        let ms_tm = Medida::ms(t.media_por_operacion());
        println!("  serializacion + sled ......... {:.2} ms", ms_tm - ms_mem);
        println!("  durabilidad fisica (flush) ... {:.2} ms", ms_disco - ms_tm);
    } else {
        println!("  persistencia completa ........ {:.2} ms", ms_disco - ms_mem);
    }
    println!("  TOTAL en disco ............... {ms_disco:.2} ms");
    println!();
    let techo = 1000.0 / ms_disco;
    println!("  techo implicito del apply .... {techo:.1} operaciones/s");
    println!();

    // El veredicto que decide el RFC, dicho por el propio banco.
    let persistencia = ms_disco - ms_mem;
    if persistencia > ms_mem {
        println!("VEREDICTO: la PERSISTENCIA domina ({:.0}% del coste).", 100.0 * persistencia / ms_disco);
        println!("  -> La etapa B (group commit) es el multiplo barato. Hacerla primero.");
    } else {
        println!("VEREDICTO: el COMPUTO domina ({:.0}% del coste).", 100.0 * ms_mem / ms_disco);
        println!("  -> La etapa B rinde poco; el trabajo esta en la D (lotes).");
    }
    println!();
    println!("Anota esta tabla en AUDITORIA.md: es el entregable de la etapa A.");
}
