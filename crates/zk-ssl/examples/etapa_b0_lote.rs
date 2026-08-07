//! **Etapa 2 · fase 0 del RFC-0002 — ¿cuánto costaría el circuito de lote?**
//!
//! El RFC-0002 exige medir esto **antes de escribir código de
//! producción**, porque un circuito de lote demasiado caro convierte al
//! nodo en el nuevo cuello de botella: *«si probar 100 cuesta 10 s, son
//! ~10 op/s y no compensa»*.
//!
//! ## Qué se mide, y por qué estos dos puntos
//!
//! No se puede cronometrar un circuito que aún no existe. Lo que sí se
//! puede es **medir el coste de probar con dos geometrías reales** y
//! ajustar la recta:
//!
//! | circuito | filas | columnas | celdas | qué es |
//! |---|---|---|---|---|
//! | `circuit_frozen_climb` | 256 | 25 | 6.400 | **DOS ascensos de Merkle de 32 niveles** — el ladrillo exacto del lote |
//! | `circuit_send` | 1.024 | 56 | 57.344 | el pago completo de hoy (~250 ms medidos en §204) |
//!
//! Con dos puntos se ajusta `t = a + b · celdas` —coste fijo más parte
//! proporcional— y se proyecta el lote.
//!
//! ⚠️ **Dos puntos son dos puntos.** La proyección a lotes grandes es una
//! **extrapolación**, no una medida: el término `n log n` de las FFT y la
//! presión de memoria harán que el coste real quede **por encima** de la
//! recta. Se publica como cota optimista, y así hay que leerla.
//!
//! ## Qué decide
//!
//! Si el techo proyectado del lote queda **muy por debajo de las
//! 265-320 op/s** que el `apply` alcanza (banda medida en §217; §209
//! citó solo el extremo bueno), entonces el circuito de lote
//! **se convierte en el cuello nuevo** y hay que preguntarse si hace
//! falta — porque la raíz nueva es **determinista** dadas las hojas, y un
//! verificador que replique el árbol la recomputa sin prueba ninguna.
//!
//! Uso:
//! ```text
//! cargo run --release -p zk-ssl --features sandbox --example etapa_b0_lote
//! ```

use std::time::Instant;

use winterfell::math::fields::f64::BaseElement;
use winterfell::Prover;

use stark_experiment::circuit_frozen_climb::{build_trace, FrozenClimbProver};

use zk_ssl::commitment::ClientState;
use zk_ssl::sparse_tree::SparseTree;
use zk_ssl::tests_support as ts;
use zk_ssl::{client, proof_options, AccountIndex, SovereignLayer};

/// Lectura aproximada de memoria residente (Linux). Diagnóstico, no medida
/// fina: sirve para ver si un lote grande se va de las manos.
fn rss_mb() -> f64 {
    std::fs::read_to_string("/proc/self/statm")
        .ok()
        .and_then(|s| s.split_whitespace().nth(1).map(|x| x.to_string()))
        .and_then(|x| x.parse::<f64>().ok())
        .map(|paginas| paginas * 4096.0 / 1_048_576.0)
        .unwrap_or(0.0)
}

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
    const REPS: u32 = 5;

    eprintln!("== ETAPA 2 · FASE 0 · RFC-0002 · ¿cuanto costaria el lote? ==");
    eprintln!("   RSS al arrancar: {:.0} MB\n", rss_mb());

    // ── Punto 1: el ascenso puro (el ladrillo del lote) ──
    eprintln!("-- punto 1: circuit_frozen_climb (256 filas x 25 col = DOS ascensos) --");
    let mut arbol = SparseTree::with_depth(32);
    arbol.set_leaf(7, [BaseElement::new(11), BaseElement::new(22),
                       BaseElement::new(33), BaseElement::new(44)]);
    let camino = arbol.path_for(7);
    let hoja_a = arbol.leaf(7);
    let hoja_b = arbol.leaf(7);

    let prover = FrozenClimbProver::new(proof_options());
    // Una en frio, que no se cronometra.
    let traza = build_trace(hoja_a, hoja_b, &camino);
    let _ = prover.prove(traza).expect("prueba de calentamiento");

    let t = Instant::now();
    for _ in 0..REPS {
        let traza = build_trace(hoja_a, hoja_b, &camino);
        let p = prover.prove(traza).expect("prueba de ascenso");
        std::hint::black_box(&p);
    }
    let ms_climb = t.elapsed().as_secs_f64() * 1000.0 / REPS as f64;
    let celdas_climb = 256.0 * 25.0;
    eprintln!("   {ms_climb:.1} ms  ({celdas_climb:.0} celdas) · RSS {:.0} MB", rss_mb());

    // ── Punto 2: el pago completo de hoy ──
    eprintln!("-- punto 2: circuit_send (1.024 filas x 56 col) --");
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

    let mut total = 0.0f64;
    for i in 0..REPS {
        let est = estado(&layer, a);
        let receptor = layer.public_id_of(b).expect("receptor");
        let m = layer
            .send_materials(a, receptor, 1_000, ts::salt_de(7 + i as u64))
            .expect("materiales");
        let t = Instant::now();
        let envio = client::prove_send(&m, clave_a, proof_options()).expect("prueba");
        total += t.elapsed().as_secs_f64() * 1000.0;
        layer.apply_send(&envio, a, &est, 1_000).expect("apply");
    }
    let ms_send = total / REPS as f64;
    let celdas_send = 1024.0 * 56.0;
    eprintln!("   {ms_send:.1} ms  ({celdas_send:.0} celdas) · RSS {:.0} MB\n", rss_mb());

    // ── La recta: t = a + b · celdas ──
    let b_pend = (ms_send - ms_climb) / (celdas_send - celdas_climb);
    let a_fijo = ms_climb - b_pend * celdas_climb;

    println!();
    println!("| punto | filas | col | celdas | prueba |");
    println!("|---|---|---|---|---|");
    println!("| frozen_climb (2 ascensos) | 256 | 25 | {celdas_climb:.0} | **{ms_climb:.1} ms** |");
    println!("| send (pago completo) | 1024 | 56 | {celdas_send:.0} | **{ms_send:.1} ms** |");
    println!();
    println!("Recta ajustada: t(ms) = {a_fijo:.1} + {:.5} x celdas", b_pend);
    println!("  (coste fijo {a_fijo:.1} ms · {:.1} us por cada 1.000 celdas)", b_pend * 1000.0);

    // ── Proyeccion del lote ──
    // Un lote de N hojas son N ascensos. frozen_climb hace DOS por 256
    // filas, luego N ascensos = N/2 * 256 = N*128 filas, redondeado a
    // potencia de dos, a 25 columnas.
    println!();
    println!("== PROYECCION del circuito de lote (25 columnas) ==");
    println!();
    println!("| N (hojas) | filas | pot. de 2 | celdas | prueba proyectada | techo |");
    println!("|---|---|---|---|---|---|");
    let mut techo_100 = 0.0f64;
    for n in [10u64, 50, 100, 500] {
        let filas = (n as f64) * 128.0;
        let pot = (filas as u64).next_power_of_two() as f64;
        let celdas = pot * 25.0;
        let ms = a_fijo + b_pend * celdas;
        let ops = (n as f64) / (ms / 1000.0);
        println!("| {n} | {filas:.0} | {pot:.0} | {celdas:.0} | **{ms:.0} ms** | **{ops:.0} op/s** |");
        if n == 100 {
            techo_100 = ops;
        }
    }

    println!();
    println!("⚠️ EXTRAPOLACION, no medida: el termino n·log(n) de las FFT y la");
    println!("   presion de memoria empujan el coste real POR ENCIMA de la recta.");
    println!("   Leelo como cota OPTIMISTA.");
    println!();
    println!("== LECTURA ==");
    println!("  techo del apply (banda, §217) ....... 265-320 op/s");
    println!("  techo proyectado del lote (N=100) ... ~{techo_100:.0} op/s");
    println!("  efectivo hoy (contencion) ........... ~3,8 op/s");
    println!();

    if techo_100 < 160.0 {
        println!("VEREDICTO: el circuito de lote SERIA EL CUELLO NUEVO.");
        println!("  Aun siendo mucho mejor que los 3,8 op/s de hoy, dejaria el nodo");
        println!("  muy por debajo de las 265-320 op/s que el apply ya alcanza.");
        println!();
        println!("  -> Pregunta obligada: ¿hace falta el circuito de lote?");
        println!("     Su unico trabajo es que un verificador que SOLO tiene raices");
        println!("     pueda comprobar root_old -> root_new. Pero si las pruebas de");
        println!("     cliente ya afirman (hoja vieja en root_old, hoja nueva = X),");
        println!("     la raiz nueva es DETERMINISTA: quien replique el arbol la");
        println!("     recomputa sin prueba ninguna — y la replica verificable es lo");
        println!("     que SECURITY.md §6 y §121 ya declaran como camino.");
        println!();
        println!("  -> Opcion (b): sin circuito de lote. Quita la contencion, no");
        println!("     anade circuitos (ni superficie de sub-restringimiento §3.1),");
        println!("     y el techo sigue siendo el del apply.");
    } else {
        println!("VEREDICTO: el lote NO seria el cuello ({techo_100:.0} op/s proyectados).");
        println!("  El circuito de lote sigue sobre la mesa. Antes de escribirlo,");
        println!("  medir RAM real a N=100 y N=500: la extrapolacion no la cubre.");
    }
    println!();
    println!("Anota esta tabla en AUDITORIA.md. Referencias: RFC-0002 etapa 2, §204, §209.");
}
