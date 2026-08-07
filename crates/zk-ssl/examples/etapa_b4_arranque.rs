//! **Banco B.4 — ¿cuánto tarda el nodo en arrancar?**
//!
//! Ni B.2 ni B.3 miran esto: los dos corren **en memoria**. Pero un nodo
//! real arranca desde disco, y `persistence::load()` recorre
//! `scan_prefix("acct:")` llamando a **`set_leaf` por cada cuenta**
//! —igual con `froz:`, `pend:`, `log:`—. **No hay caché persistida: el
//! árbol se reconstruye entero en cada arranque.**
//!
//! ## La sospecha que este banco pone a prueba
//!
//! **§207 pudo haber encarecido el arranque sin que nadie lo mirara.**
//!
//! Antes de §207, `set_leaf` era una inserción en un `HashMap`: `O(1)`.
//! Desde §207 mantiene los nodos internos y cuesta **`O(profundidad)` =
//! 32 merges de Rescue**. Cargar N cuentas pasó de N inserciones a
//! **N × 32 merges**.
//!
//! Con los ~7,5 µs por merge que se deducen de §204 (4.115 merges =
//! 30,99 ms), la aritmética da: **1e5 cuentas ≈ 24 s · 1e6 ≈ 4 minutos**,
//! solo para el árbol de cuentas.
//!
//! El asiento §207 declaró el precio en **memoria** —«se cambia tiempo
//! por memoria»— pero **no dijo nada del arranque**. Si esa aritmética se
//! confirma, es una deuda declarable de §207, no un defecto nuevo.
//!
//! ⚠️ **La aritmética de arriba es una ESTIMACIÓN.** Puede estar mal por
//! un factor de dos o de diez. Por eso se mide.
//!
//! ## Qué mide
//!
//! Por escalón (1e3, 1e4, …): tiempo de **crear** el ledger, tiempo de
//! **reabrirlo** (que es lo que importa), tamaño en disco, y RSS tras la
//! carga. Y contrasta el tiempo real con la estimación teórica.
//!
//! Usa un directorio temporal propio y **lo borra al terminar**.
//!
//! Uso:
//! ```text
//! cargo run --release -p zk-ssl --features sandbox --example etapa_b4_arranque -- 100000
//! ```
//! (tope de cuentas; por defecto 10.000)

use std::time::Instant;

use zk_ssl::tests_support as ts;
use zk_ssl::SovereignLayer;

/// Coste medido de un merge de Rescue, deducido de §204 (banco A.4):
/// 4.115 merges costaron 30,99 ms.
const US_POR_MERGE: f64 = 30_990.0 / 4_115.0;
/// Profundidad del árbol de cuentas.
const PROFUNDIDAD: f64 = 32.0;

fn rss_mb() -> f64 {
    std::fs::read_to_string("/proc/self/statm")
        .ok()
        .and_then(|s| s.split_whitespace().nth(1).map(str::to_string))
        .and_then(|x| x.parse::<f64>().ok())
        .map(|p| p * 4096.0 / 1_048_576.0)
        .unwrap_or(f64::NAN)
}

/// Tamaño en disco de un directorio, en MB.
fn tam_mb(ruta: &str) -> f64 {
    fn suma(p: &std::path::Path) -> u64 {
        let mut t = 0;
        if let Ok(rd) = std::fs::read_dir(p) {
            for e in rd.flatten() {
                let md = match e.metadata() {
                    Ok(m) => m,
                    Err(_) => continue,
                };
                t += if md.is_dir() { suma(&e.path()) } else { md.len() };
            }
        }
        t
    }
    suma(std::path::Path::new(ruta)) as f64 / 1_048_576.0
}

struct Punto {
    cuentas: u64,
    s_crear: f64,
    s_abrir: f64,
    disco: f64,
    rss: f64,
}

fn corrida(cuentas: u64, base: &str) -> Punto {
    let ruta = format!("{base}/n{cuentas}");
    let _ = std::fs::remove_dir_all(&ruta);
    let tope = (cuentas + 16).max(1_000);

    // ── 1 · crear y llenar ──
    let t0 = Instant::now();
    {
        let mut l = SovereignLayer::open(
            &ruta,
            ts::custodian_root(),
            ts::governance_root(),
            ts::LIMIT,
            ts::MAX_SUPPLY,
            tope,
        )
        .expect("abrir ledger nuevo");
        // Cuentas VACIAS: abrir no lleva prueba (§B.3).
        for k in 0..cuentas {
            l.open_account_wide(ts::wide_key(0xC0FFEE_0000 + k));
        }
        // `l` se cierra al salir del bloque.
    }
    let s_crear = t0.elapsed().as_secs_f64();
    let disco = tam_mb(&ruta);

    // ── 2 · REABRIR: esto es lo que mide el banco ──
    let rss_antes = rss_mb();
    let t0 = Instant::now();
    let l = SovereignLayer::open(
        &ruta,
        ts::custodian_root(),
        ts::governance_root(),
        ts::LIMIT,
        ts::MAX_SUPPLY,
        tope,
    )
    .expect("reabrir ledger");
    let s_abrir = t0.elapsed().as_secs_f64();
    let rss = rss_mb() - rss_antes;

    // Comprobacion de cordura: **se cargaron las cuentas que se
    // escribieron**. Sin esto, un `load` que fallara en silencio daria un
    // arranque rapidisimo y una tabla mentirosa.
    let cargadas = l.account_count() as u64;
    assert_eq!(
        cargadas,
        cuentas,
        "se escribieron {cuentas} cuentas y se cargaron {cargadas}: el banco mediria una carga incompleta"
    );
    drop(l);
    let _ = std::fs::remove_dir_all(&ruta);

    Punto {
        cuentas,
        s_crear,
        s_abrir,
        disco,
        rss,
    }
}

fn exponente(a: &Punto, b: &Punto, f: impl Fn(&Punto) -> f64) -> f64 {
    let (ya, yb) = (f(a), f(b));
    if ya <= 0.0 || yb <= 0.0 || a.cuentas == 0 || b.cuentas == 0 {
        return f64::NAN;
    }
    (yb / ya).ln() / ((b.cuentas as f64) / (a.cuentas as f64)).ln()
}

fn main() {
    let tope: u64 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(10_000);

    let base = std::env::var("TMPDIR").unwrap_or_else(|_| "/tmp".into());
    let base = format!("{base}/zkssl_b4");
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).expect("directorio de trabajo");

    eprintln!("== BANCO B.4 · ¿cuanto tarda el nodo en ARRANCAR? ==");
    eprintln!("   tope: {tope} cuentas · ledger en {base}");
    eprintln!(
        "   estimacion previa: {:.1} us por merge x 32 por cuenta",
        US_POR_MERGE
    );
    eprintln!("   ⚠️ es una ESTIMACION deducida de §204, no una medida\n");

    let mut escalones: Vec<u64> = Vec::new();
    let mut e = 1_000u64;
    while e <= tope {
        escalones.push(e);
        e *= 10;
    }
    if escalones.is_empty() {
        escalones.push(tope);
    }

    let mut puntos = Vec::new();
    for &c in &escalones {
        eprintln!("-- {c} cuentas --");
        let p = corrida(c, &base);
        eprintln!(
            "   crear {:.1} s · ABRIR {:.2} s · disco {:.0} MB · RSS +{:.0} MB",
            p.s_crear, p.s_abrir, p.disco, p.rss
        );
        puntos.push(p);
    }
    let _ = std::fs::remove_dir_all(&base);

    println!();
    println!("| cuentas | crear (s) | **ABRIR (s)** | us/cuenta al abrir | estimado (s) | disco (MB) | RSS (MB) |");
    println!("|---|---|---|---|---|---|---|");
    for p in &puntos {
        let us = p.s_abrir * 1e6 / p.cuentas.max(1) as f64;
        let est = p.cuentas as f64 * PROFUNDIDAD * US_POR_MERGE / 1e6;
        println!(
            "| {} | {:.1} | **{:.2}** | {us:.1} | {est:.1} | {:.0} | {:.0} |",
            p.cuentas, p.s_crear, p.s_abrir, p.disco, p.rss
        );
    }

    let pa = &puntos[0];
    let pu = &puntos[puntos.len() - 1];
    let e_abrir = exponente(pa, pu, |p| p.s_abrir);
    let est_u = pu.cuentas as f64 * PROFUNDIDAD * US_POR_MERGE / 1e6;
    let us_cuenta = pu.s_abrir * 1e6 / pu.cuentas.max(1) as f64;

    println!();
    println!("== EXPONENTES ==");
    println!("  abrir ... e = {e_abrir:.2}   (lineal seria 1,00)");
    println!(
        "  disco ... e = {:.2} · RSS ... e = {:.2}",
        exponente(pa, pu, |p| p.disco),
        exponente(pa, pu, |p| p.rss)
    );
    println!();
    println!("== LECTURA ==");
    println!(
        "  con {} cuentas: **{:.2} s de arranque** ({us_cuenta:.1} us por cuenta)",
        pu.cuentas, pu.s_abrir
    );
    println!("  estimacion teorica (N x 32 x {:.1} us): {est_u:.1} s", US_POR_MERGE);
    // §219: extrapolar con el exponente MEDIDO, no linealmente. Con tres
    // escalones e=1,03, y la diferencia a 1e6 es de 297 a 318 s: un 7 %
    // que se perdia por usar un numero resumido teniendo el dato al lado.
    let factor = 1_000_000.0 / pu.cuentas as f64;
    let proy_1m = if e_abrir.is_finite() && e_abrir > 0.0 {
        pu.s_abrir * factor.powf(e_abrir)
    } else {
        pu.s_abrir * factor
    };
    println!("  proyeccion a 1.000.000 de cuentas: **~{proy_1m:.0} s**");
    println!("  ⚠️ extrapolada con el exponente MEDIDO (e={e_abrir:.2}), no es una");
    println!("     medida. Con UN solo escalon no hay exponente y cae a lineal.");
    println!();

    let cerca = est_u > 0.0 && (pu.s_abrir / est_u) > 0.4 && (pu.s_abrir / est_u) < 2.5;
    if proy_1m > 60.0 {
        println!("VEREDICTO: ⚠️ EL ARRANQUE ES UN PROBLEMA OPERATIVO.");
        println!("  ~{proy_1m:.0} s para reconstruir un ledger de un millon de cuentas.");
        println!("  Un nodo de liquidacion que tarda minutos en volver tras un");
        println!("  reinicio NO cumple un objetivo de recuperacion razonable, y eso");
        println!("  es exactamente lo que un comite de riesgos mira (RTO).");
        if cerca {
            println!();
            println!("  Y la causa esta identificada: el tiempo casa con N x 32 merges,");
            println!("  es decir con el `set_leaf` de §207. **§207 declaro su precio en");
            println!("  MEMORIA pero no en ARRANQUE.** Es una deuda de aquel sello, no");
            println!("  un defecto nuevo, y hay que anotarla como tal.");
        }
        println!();
        println!("  Salidas, ninguna trivial:");
        println!("   - PERSISTIR los nodos internos y no reconstruirlos (mas disco,");
        println!("     arranque casi instantaneo). Es la simetrica del intercambio");
        println!("     que §207 ya hizo: alli tiempo por memoria, aqui disco por tiempo.");
        println!("   - carga PEREZOSA: reconstruir solo lo que se toca.");
        println!("   - instantanea binaria del arbol al cerrar limpiamente, con");
        println!("     reconstruccion solo tras caida.");
    } else if e_abrir.is_finite() && e_abrir > 1.35 {
        println!("VEREDICTO: ⚠️ EL ARRANQUE CRECE PEOR QUE LINEAL (e={e_abrir:.2}).");
        println!("  No deberia: son N inserciones de coste fijo. Algo escala mal en");
        println!("  la carga —¿el `scan_prefix`? ¿la deserializacion?— y hay que");
        println!("  mirarlo antes de proyectar nada.");
    } else {
        println!("VEREDICTO: ✅ EL ARRANQUE AGUANTA ({:.2} s a {} cuentas).", pu.s_abrir, pu.cuentas);
        println!("  e={e_abrir:.2} y ~{proy_1m:.0} s proyectados a un millon. **Se declara");
        println!("  medido HASTA AQUI.** Anotar el numero en la documentacion");
        println!("  operativa: es lo que un RTO tiene que contemplar.");
    }
    println!();
    println!("⚠️ Este banco NO mide: el `apply` (B.3), el arbol solo (B.2), ni el");
    println!("   arranque con REGISTRO grande — aqui el log solo tiene las altas.");
    println!("   Un ledger con millones de ENTRADAS de registro es otro escalon.");
    println!();
    println!("Anota esta tabla en AUDITORIA.md. Si el veredicto es rojo, la deuda");
    println!("va referida a §207, que declaro el precio en memoria y no en arranque.");
}
