//! **Banco B.2 — ¿cuánto pesa el árbol a escala real?**
//!
//! Todo lo medido hasta hoy sobre el árbol se hizo con **cuatro a sesenta
//! cuentas**. El techo del `apply` (~320 op/s, §209) y el exponente plano
//! de §207 se midieron ahí. Nadie ha mirado qué pasa con **cien mil o un
//! millón**, y desde §207 el árbol guarda nodos internos en caché: la
//! memoria crece con `hojas × profundidad`.
//!
//! Este banco **no toca la capa, ni genera pruebas, ni abre cuentas**.
//! Solo `SparseTree`. Por eso puede llegar a millones en segundos, y por
//! eso responde la pregunta grande antes de gastar en las pequeñas.
//!
//! ## Los dos patrones, y por qué importan
//!
//! | patrón | de dónde sale |
//! |---|---|
//! | **disperso** | el REAL: `accounts.rs` coloca por `public_id[0] mod capacidad`, dispersión pseudoaleatoria **deliberada** para que los índices no sean enumerables ni un atacante elija vecino |
//! | **consecutivo** | el hipotético: 0, 1, 2… Comparte casi todos los nodos altos |
//!
//! Medir los dos pone precio a esa decisión de seguridad. Si el disperso
//! cuesta N veces más memoria, **eso es lo que se paga por no ser
//! enumerable**, y merece estar escrito.
//!
//! ## Qué se mide en cada escalón
//!
//! - `cached_nodes()` — entradas del mapa de nodos internos
//! - **RSS** — memoria residente real del proceso, no una estimación
//! - `set_leaf`, `root_with`, `path_for` — deben seguir siendo
//!   `O(profundidad)`: **planos**, no crecientes
//!
//! ## Qué puede salir
//!
//! - **Nodos ≈ hojas × 32 y RSS manejable** → el diseño aguanta; se
//!   declara el techo de memoria y a otra cosa.
//! - **Tiempos creciendo** → la caché no hace lo que §207 dice.
//! - **RSS desbocado** → hace falta caché parcial o persistir los nodos
//!   internos. **Es rediseño**, y mejor saberlo ahora.
//!
//! ⚠️ Este banco puede pedir **varios GB**. Empieza por el escalón bajo.
//!
//! Uso:
//! ```text
//! cargo run --release -p zk-ssl --features sandbox --example etapa_b2_arbol_a_escala -- 1000000
//! ```
//! (tope de hojas; por defecto 100.000)

use std::time::Instant;

use winterfell::math::fields::f64::BaseElement;

use zk_ssl::sparse_tree::SparseTree;

/// Memoria residente del proceso, en MB. Lectura real de `/proc`, no una
/// estimación: lo que interesa es lo que el sistema operativo cree que
/// ocupamos, incluida la sobrecarga del mapa.
fn rss_mb() -> f64 {
    std::fs::read_to_string("/proc/self/statm")
        .ok()
        .and_then(|s| s.split_whitespace().nth(1).map(str::to_string))
        .and_then(|x| x.parse::<f64>().ok())
        .map(|paginas| paginas * 4096.0 / 1_048_576.0)
        .unwrap_or(f64::NAN)
}

/// Hoja determinista y no trivial para una posición.
fn hoja(i: u64) -> [BaseElement; 4] {
    [
        BaseElement::new(i.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1),
        BaseElement::new(i ^ 0x5851_F42D_4C95_7F2D),
        BaseElement::new(i.wrapping_add(0xD1B5_4A32_D192_ED03)),
        BaseElement::new(i.rotate_left(17) | 1),
    ]
}

/// Índice **disperso**, imitando `accounts.rs`: `public_id[0] mod cap`.
/// Es el patrón REAL del sistema.
fn indice_disperso(i: u64, cap: u64) -> u64 {
    let mezcla = i
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .rotate_left(31)
        .wrapping_mul(0xBF58_476D_1CE4_E5B9);
    mezcla % cap
}

struct Punto {
    hojas: u64,
    nodos: usize,
    rss: f64,
    us_set: f64,
    us_root_with: f64,
    us_path: f64,
}

fn medir(disperso: bool, tope: u64) -> Vec<Punto> {
    let mut t = SparseTree::new();
    let cap = t.capacity();
    let rss0 = rss_mb();
    let mut puntos = Vec::new();

    // Escalones: 1e3, 1e4, 1e5, 1e6 … hasta el tope.
    let mut hitos: Vec<u64> = Vec::new();
    let mut h = 1_000u64;
    while h <= tope {
        hitos.push(h);
        h *= 10;
    }
    if hitos.is_empty() {
        hitos.push(tope);
    }

    let mut puestas = 0u64;
    for &hito in &hitos {
        while puestas < hito {
            let idx = if disperso {
                indice_disperso(puestas, cap)
            } else {
                puestas
            };
            t.set_leaf(idx, hoja(puestas));
            puestas += 1;
        }

        // ── Coste de una ESCRITURA, ya con el árbol de este tamaño ──
        const REPS: u64 = 200;
        let base = puestas + 1_000_000_000;
        let t0 = Instant::now();
        for r in 0..REPS {
            let idx = if disperso {
                indice_disperso(base + r, cap)
            } else {
                puestas + r
            };
            t.set_leaf(idx, hoja(base + r));
        }
        let us_set = t0.elapsed().as_secs_f64() * 1e6 / REPS as f64;

        // ── Coste de una LECTURA hipotética (`root_with`) ──
        let t0 = Instant::now();
        for r in 0..REPS {
            let idx = if disperso {
                indice_disperso(r, cap)
            } else {
                r
            };
            std::hint::black_box(t.root_with(idx, hoja(r)));
        }
        let us_root_with = t0.elapsed().as_secs_f64() * 1e6 / REPS as f64;

        // ── Coste de un CAMINO (lo que paga `send_materials`) ──
        let t0 = Instant::now();
        for r in 0..REPS {
            let idx = if disperso {
                indice_disperso(r, cap)
            } else {
                r
            };
            std::hint::black_box(t.path_for(idx));
        }
        let us_path = t0.elapsed().as_secs_f64() * 1e6 / REPS as f64;

        puntos.push(Punto {
            hojas: t.len() as u64,
            nodos: t.cached_nodes(),
            rss: rss_mb() - rss0,
            us_set,
            us_root_with,
            us_path,
        });
        eprintln!(
            "   {:>9} hojas · {:>11} nodos · RSS +{:>7.0} MB · set {:.1} us · root_with {:.1} us · path {:.1} us",
            t.len(),
            t.cached_nodes(),
            rss_mb() - rss0,
            us_set,
            us_root_with,
            us_path
        );
    }
    puntos
}

fn exponente(a: &Punto, b: &Punto, f: impl Fn(&Punto) -> f64) -> f64 {
    let (ya, yb) = (f(a), f(b));
    if ya <= 0.0 || yb <= 0.0 || a.hojas == 0 || b.hojas == 0 {
        return f64::NAN;
    }
    (yb / ya).ln() / ((b.hojas as f64) / (a.hojas as f64)).ln()
}

fn main() {
    let tope: u64 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(100_000);

    eprintln!("== BANCO B.2 · el arbol a escala real ==");
    eprintln!("   tope: {tope} hojas · profundidad 32 · RSS inicial {:.0} MB", rss_mb());
    eprintln!("   ⚠️ el escalon de 1.000.000 puede pedir varios GB\n");

    eprintln!("-- patron DISPERSO (el REAL: public_id[0] mod capacidad) --");
    let disp = medir(true, tope);
    eprintln!("\n-- patron CONSECUTIVO (hipotetico: 0,1,2...) --");
    let cons = medir(false, tope);

    println!();
    println!("| patron | hojas | nodos en cache | nodos/hoja | RSS (MB) | bytes/nodo | set (us) | root_with (us) | path (us) |");
    println!("|---|---|---|---|---|---|---|---|---|");
    for (nombre, serie) in [("disperso", &disp), ("consecutivo", &cons)] {
        for p in serie {
            let por_hoja = p.nodos as f64 / p.hojas.max(1) as f64;
            let por_nodo = if p.nodos > 0 {
                p.rss * 1_048_576.0 / p.nodos as f64
            } else {
                f64::NAN
            };
            println!(
                "| {nombre} | {} | {} | {por_hoja:.1} | {:.0} | {por_nodo:.0} | {:.1} | {:.1} | {:.1} |",
                p.hojas, p.nodos, p.rss, p.us_set, p.us_root_with, p.us_path
            );
        }
    }

    println!();
    println!("== EXPONENTES (entre el primer y el ultimo escalon) ==");
    for (nombre, serie) in [("disperso", &disp), ("consecutivo", &cons)] {
        if serie.len() < 2 {
            continue;
        }
        let (a, b) = (&serie[0], &serie[serie.len() - 1]);
        println!(
            "  {nombre:<12} nodos e={:.2} · RSS e={:.2} · set e={:.2} · root_with e={:.2} · path e={:.2}",
            exponente(a, b, |p| p.nodos as f64),
            exponente(a, b, |p| p.rss),
            exponente(a, b, |p| p.us_set),
            exponente(a, b, |p| p.us_root_with),
            exponente(a, b, |p| p.us_path),
        );
    }

    println!();
    println!("== LECTURA ==");
    let ud = &disp[disp.len() - 1];
    let uc = &cons[cons.len() - 1];
    println!("  Con {} hojas:", ud.hojas);
    println!("    disperso (REAL) ... {} nodos · {:.0} MB", ud.nodos, ud.rss);
    println!("    consecutivo ....... {} nodos · {:.0} MB", uc.nodos, uc.rss);
    if uc.rss > 0.0 {
        println!(
            "    PRECIO DE LA DISPERSION: {:.1}x en memoria",
            ud.rss / uc.rss
        );
        println!("    (la dispersion NO es un descuido: `accounts.rs` coloca por");
        println!("     public_id[0] mod capacidad para que los indices no sean");
        println!("     enumerables ni un atacante elija vecino. Esto le pone precio.)");
    }
    let e_tiempo = exponente(&disp[0], ud, |p| p.us_root_with);
    let proy_1m = ud.rss * (1_000_000.0 / ud.hojas as f64);
    println!();
    println!("    proyeccion a 1.000.000 de cuentas (disperso): ~{proy_1m:.0} MB");
    println!("    ⚠️ extrapolacion lineal, NO medida, salvo que el tope ya sea 1e6.");

    println!();
    if e_tiempo.is_finite() && e_tiempo > 0.35 {
        println!("VEREDICTO: ⚠️ EL TIEMPO CRECE CON EL TAMANO (e={e_tiempo:.2}).");
        println!("  La cache de §207 deberia dar O(profundidad): PLANO. Si crece,");
        println!("  el techo del apply (~320 op/s) y el exponente plano de §207 se");
        println!("  midieron con 4-60 cuentas y NO valen a escala. Volver a medir");
        println!("  el apply con 1e5 cuentas antes de afirmar nada de rendimiento.");
    } else if proy_1m > 4_000.0 {
        println!("VEREDICTO: ⚠️ LA MEMORIA ES EL LIMITE, no el tiempo.");
        println!("  Los tiempos se mantienen planos (e={e_tiempo:.2}: la cache");
        println!("  funciona), pero a un millon de cuentas harian falta ~{proy_1m:.0} MB");
        println!("  solo para los nodos internos del arbol de cuentas — y hay TRES");
        println!("  arboles. Eso es un techo de despliegue, no un detalle:");
        println!("   - o se acota el numero de cuentas y se DECLARA;");
        println!("   - o los nodos internos se persisten y se cachea solo una parte;");
        println!("   - o se reduce la profundidad (rompe circuitos: seria RFC).");
        println!("  Ninguna es trivial. Decidir en mesa ANTES de prometer escala.");
    } else {
        println!("VEREDICTO: ✅ EL ARBOL AGUANTA.");
        println!("  Tiempos planos (e={e_tiempo:.2}) y ~{proy_1m:.0} MB proyectados a un");
        println!("  millon de cuentas. Se DECLARA ese techo de memoria en la");
        println!("  documentacion y se sigue. Siguiente medida pendiente: el apply");
        println!("  completo a 1e5 cuentas, y el arranque desde disco (§204: `open`");
        println!("  reconstruye el arbol hoja a hoja).");
    }
    println!();
    println!("Anota esta tabla en AUDITORIA.md. NO afirma nada sobre el apply");
    println!("completo ni sobre la persistencia: solo mide el arbol.");
}
