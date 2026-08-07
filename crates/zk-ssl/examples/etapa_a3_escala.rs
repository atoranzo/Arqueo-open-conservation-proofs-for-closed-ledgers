//! **Etapa A.3 del RFC-0002 — ¿el coste de los árboles es inherente o
//! es la implementación?**
//!
//! A midió que el `apply` cuesta ~38 ms y que la persistencia es el 3 %.
//! A.2 partió ese coste: **verificación 7 %, árboles 93 %**.
//!
//! Este banco pone a prueba una hipótesis concreta sobre ese 93 %.
//!
//! ## La hipótesis
//!
//! `SparseTree::root()` **no cachea**: recomputa el árbol entero en cada
//! llamada (`node(depth, 0)` recursivo). Y en cada nodo visitado decide
//! si el subárbol está ocupado con un **barrido lineal de todas las
//! hojas**:
//!
//! ```text
//! let occupied = self.leaves.keys().any(|k| *k >= start && *k < end);
//! ```
//!
//! El coste de una raíz sería entonces del orden de `2 · profundidad ·
//! k²`, con `k` = hojas ocupadas. Es decir: **cuadrático en el número de
//! cuentas**.
//!
//! ## La predicción, y qué la refuta
//!
//! Si la hipótesis es cierta, al multiplicar las cuentas por `m` el
//! tiempo de `apply` se multiplica por ~`m²` (exponente medido ≈ 2).
//!
//! - **exponente ≈ 2** → confirmada. El 93 % **no es coste inherente del
//!   diseño**: es la implementación del árbol, y se corrige con caché de
//!   raíz y nodos internos incrementales —**sin tocar circuitos, cable ni
//!   protocolo**—.
//! - **exponente ≈ 0** (tiempo plano) → **refutada**. Los 31 ms son otra
//!   cosa y hay que volver a medir antes de tocar nada.
//! - **exponente ≈ 1** → parcialmente: hay un término lineal dominante.
//!
//! ⚠️ Este banco **no demuestra** dónde está el coste: mide cómo escala.
//! Un exponente ≈ 2 es evidencia fuerte, no una prueba.
//!
//! Uso:
//! ```text
//! cargo run --release -p zk-ssl --features sandbox --example etapa_a3_escala -- 6
//! ```
//! (el argumento es el número de pagos cronometrados por tamaño)

use std::time::{Duration, Instant};

use zk_ssl::commitment::ClientState;
use zk_ssl::tests_support as ts;
use zk_ssl::{client, proof_options, AccountIndex, SovereignLayer};

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

/// Corrida con `cuentas` cuentas abiertas; cronometra `pagos` pagos
/// entre las dos primeras. Devuelve (media apply por operación,
/// media de `send_materials`, media de generación).
fn corrida(cuentas: u64, pagos: u32) -> (f64, f64, f64) {
    let mut layer = SovereignLayer::new(
        ts::custodian_root(),
        ts::governance_root(),
        ts::LIMIT,
        ts::MAX_SUPPLY,
        ts::MAX_ACCOUNTS,
    );

    // Montaje: `cuentas` cuentas abiertas. Solo las dos primeras se
    // fondean y operan; el resto existen para llenar el árbol.
    let mut indices = Vec::new();
    for i in 0..cuentas {
        let idx = layer.open_account_wide(ts::wide_key(0xA11CE + i));
        indices.push(idx);
    }
    let a = indices[0];
    let b = indices[1];
    fondear(&mut layer, a, 1_000_000);
    fondear(&mut layer, b, 1_000_000);

    let mut t_apply = Duration::ZERO;
    let mut t_mat = Duration::ZERO;
    let mut t_gen = Duration::ZERO;
    let mut ops = 0u32;

    for i in 0..pagos {
        let importe = 1_000u64;

        let est_a = estado(&layer, a);
        let receptor = layer.public_id_of(b).expect("receptor");

        let t = Instant::now();
        let materiales = layer
            .send_materials(a, receptor, importe, ts::salt_de(7 + i as u64))
            .expect("materiales de envio");
        t_mat += t.elapsed();

        let t = Instant::now();
        let envio =
            client::prove_send(&materiales, ts::wide_key(0xA11CE), proof_options())
                .expect("prueba de envio");
        t_gen += t.elapsed();

        let t = Instant::now();
        layer
            .apply_send(&envio, a, &est_a, importe)
            .expect("apply_send");
        t_apply += t.elapsed();
        ops += 1;

        let est_b = estado(&layer, b);
        let materiales = layer
            .claim_materials(b, &envio.notice)
            .expect("materiales de cobro");
        let cobro =
            client::prove_claim(&materiales, ts::wide_key(0xA11CE + 1), proof_options())
                .expect("prueba de cobro");

        let t = Instant::now();
        layer
            .apply_claim(&cobro, b, &est_b, &envio.notice)
            .expect("apply_claim");
        t_apply += t.elapsed();
        ops += 1;
    }

    let d = ops.max(1);
    (
        ms(t_apply / d),
        ms(t_mat / pagos.max(1)),
        ms(t_gen / pagos.max(1)),
    )
}

fn main() {
    let pagos: u32 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(6);

    let tamanos: [u64; 3] = [4, 20, 60];

    eprintln!("== ETAPA A.3 · RFC-0002 · como escala el apply con las cuentas ==");
    eprintln!("   tamanos: {tamanos:?} cuentas · {pagos} pagos cronometrados cada uno");
    eprintln!("   hipotesis: root() recomputa el arbol y barre TODAS las hojas");
    eprintln!("              por nodo -> coste ~ k^2. Si el exponente sale ~2,");
    eprintln!("              el 93% de A.2 NO es inherente al diseno.\n");

    let mut filas = Vec::new();
    for &k in &tamanos {
        eprintln!("-- {k} cuentas --");
        let (ap, mat, gen) = corrida(k, pagos);
        eprintln!("   apply {ap:.2} ms · materiales {mat:.2} ms · generacion {gen:.0} ms");
        filas.push((k, ap, mat, gen));
    }

    println!();
    println!("| cuentas | apply/operacion | send_materials | generacion (referencia) |");
    println!("|---|---|---|---|");
    for (k, ap, mat, gen) in &filas {
        println!("| {k} | **{ap:.2} ms** | {mat:.2} ms | {gen:.0} ms |");
    }

    // Exponente medido entre el primer y el ultimo tamano:
    //   t ~ k^e   =>   e = ln(t2/t1) / ln(k2/k1)
    let (k1, t1, m1, _) = filas[0];
    let (k2, t2, m2, _) = filas[filas.len() - 1];
    let razon_k = (k2 as f64) / (k1 as f64);
    let e_apply = (t2 / t1).ln() / razon_k.ln();
    let e_mat = (m2 / m1).ln() / razon_k.ln();

    println!();
    println!("== EXPONENTE MEDIDO (t ~ cuentas^e, entre {k1} y {k2}) ==");
    println!("  apply ............ e = {e_apply:.2}   ({:.1}x mas lento)", t2 / t1);
    println!("  send_materials ... e = {e_mat:.2}   ({:.1}x mas lento)", m2 / m1);
    println!();

    if e_apply > 1.5 {
        println!("VEREDICTO: HIPOTESIS CONFIRMADA (e = {e_apply:.2}, cuadratico o peor).");
        println!("  -> El 93% de A.2 NO es coste inherente del diseno: es el arbol.");
        println!("  -> `SparseTree::root()` recomputa sin cache y barre todas las");
        println!("     hojas por nodo (sparse_tree.rs, linea del `any(...)`).");
        println!("  -> ARREGLO SIN TOCAR PROTOCOLO NI CIRCUITOS: cachear la raiz e");
        println!("     invalidarla al insertar; guardar nodos internos y actualizar");
        println!("     solo el camino de la hoja modificada. O(k^2*d) -> O(d).");
        println!("  -> Esto REDEFINE el RFC-0002: la etapa B pasa a ser el arbol");
        println!("     incremental, y las etapas C/D pueden no hacer falta para el");
        println!("     objetivo de media. Volver a medir A antes de decidirlas.");
        let proy = t2 * (1000.0f64 / k2 as f64).powf(e_apply);
        println!();
        println!("  Proyeccion a 1.000 cuentas con este exponente: ~{proy:.0} ms/operacion");
        println!("  (extrapolacion, NO medida: sirve para dimensionar la urgencia)");
    } else if e_apply < 0.5 {
        println!("VEREDICTO: HIPOTESIS REFUTADA (e = {e_apply:.2}, practicamente plano).");
        println!("  -> Los 31 ms de A.2 no vienen del tamano del arbol.");
        println!("  -> NO tocar el arbol. Hay que medir de nuevo que compone esos");
        println!("     31 ms antes de abrir ninguna etapa del RFC-0002.");
    } else {
        println!("VEREDICTO: PARCIAL (e = {e_apply:.2}).");
        println!("  -> Hay un termino que crece pero no es cuadratico limpio.");
        println!("  -> Repetir con mas tamanos antes de concluir.");
    }
    println!();
    println!("Anota esta tabla en AUDITORIA.md junto a las de A y A.2.");
}
