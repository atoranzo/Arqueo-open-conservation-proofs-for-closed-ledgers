//! **Banco B.3 — ¿aguanta el `apply` con el árbol grande?**
//!
//! B.2 mide **el árbol solo**. Este mide **el `apply` completo**:
//! verificación STARK, los tres árboles, `records`, el registro y la
//! persistencia. Son cosas distintas — el árbol puede quedar plano y el
//! `apply` no, porque toca piezas que B.2 no mira: `records` es un
//! `HashMap` con una entrada por cuenta, el registro crece, y `commit()`
//! reescribe ~20 claves de metadatos por operación.
//!
//! ## La cifra en juego
//!
//! **El techo de ~320 op/s de §209 se midió con DOS cuentas.** Si a 1e5
//! cuentas cae, todo lo que esta casa afirma sobre rendimiento —§209 y
//! §216 incluidos— vale para un juguete y no para un despliegue. Eso es
//! lo que este banco pone a prueba.
//!
//! ## El truco que lo hace viable
//!
//! Montar 1e5 cuentas **fondeadas** es imposible: cada emisión lleva una
//! prueba STARK (~250 ms → horas). Pero **abrir una cuenta no necesita
//! prueba**: `open_account_wide` solo hace `set_leaf` + `records.insert`.
//!
//! Así que: **se abren N cuentas vacías y se fondean solo las dos que
//! operan.** El árbol y los `records` quedan del tamaño real; el montaje
//! cuesta segundos.
//!
//! ⚠️ `tests_support::MAX_ACCOUNTS` es **1.000** — se agota en el primer
//! escalón. Este banco construye la capa con su propio tope.
//!
//! ## Lo que NO mide, y hay que saberlo
//!
//! - El **arranque desde disco**: `open()` reconstruye el árbol hoja a
//!   hoja; con 1e6 cuentas podrían ser minutos. Es la medida hermana y va
//!   aparte (B.4).
//! - El rendimiento **concurrente** a escala (B.1 lo midió con 4 pares).
//! - Todo aquí corre **en memoria**: la persistencia es el 3 % (§204 A).
//!
//! Uso:
//! ```text
//! cargo run --release -p zk-ssl --features sandbox --example etapa_b3_apply_a_escala -- 100000
//! ```
//! (tope de cuentas abiertas; por defecto 10.000)

use std::time::Instant;

use zk_ssl::commitment::ClientState;
use zk_ssl::tests_support as ts;
use zk_ssl::{client, proof_options, AccountIndex, SovereignLayer};

fn rss_mb() -> f64 {
    std::fs::read_to_string("/proc/self/statm")
        .ok()
        .and_then(|s| s.split_whitespace().nth(1).map(str::to_string))
        .and_then(|x| x.parse::<f64>().ok())
        .map(|p| p * 4096.0 / 1_048_576.0)
        .unwrap_or(f64::NAN)
}

fn estado(l: &SovereignLayer, i: AccountIndex) -> ClientState {
    ClientState {
        public_id: l.public_id_of(i).expect("cuenta abierta"),
        balance: l.balance_of(i).expect("cuenta abierta"),
        nonce: l.nonce_of(i).expect("cuenta abierta"),
    }
}

fn fondear(l: &mut SovereignLayer, i: AccountIndex, importe: u64) {
    let op = ts::mint_commitment(l, i, importe);
    let subida = ts::mint_climb_proof(l, i, importe);
    let (pa, ia, pb, ib) = ts::delegated_pair(op, 1, 3);
    l.apply_mint_delegated(subida, pa, ia, pb, ib, i, importe)
        .expect("fondear");
}

struct Punto {
    cuentas: u64,
    rss: f64,
    ms_apply: f64,
    ms_materiales: f64,
    ms_generar: f64,
}

/// Una corrida con `cuentas` cuentas abiertas. Solo dos se fondean y
/// operan; el resto existen para que el árbol y `records` pesen lo real.
fn corrida(cuentas: u64, pagos: u32, rss0: f64) -> Punto {
    // ⚠️ Tope propio: `ts::MAX_ACCOUNTS` es 1.000 y se agotaria enseguida.
    let tope = (cuentas + 16).max(1_000);
    let mut l = SovereignLayer::new(
        ts::custodian_root(),
        ts::governance_root(),
        ts::LIMIT,
        ts::MAX_SUPPLY,
        tope,
    );

    // Las dos que operan, primero: sus indices no dependen del relleno
    // porque la colocacion es por `public_id[0] mod capacidad`.
    let ka = ts::wide_key(0xA11CE);
    let kb = ts::wide_key(0xA11CE + 1);
    let a = l.open_account_wide(ka);
    let b = l.open_account_wide(kb);
    fondear(&mut l, a, 1_000_000);
    fondear(&mut l, b, 1_000_000);

    // El relleno: cuentas VACIAS, sin prueba ninguna.
    for k in 0..cuentas {
        l.open_account_wide(ts::wide_key(0xBEEF_0000 + k));
    }

    let mut t_apply = 0.0f64;
    let mut t_mat = 0.0f64;
    let mut t_gen = 0.0f64;
    let mut ops = 0u32;

    for i in 0..pagos {
        // ── envio ──
        let est = estado(&l, a);
        let receptor = l.public_id_of(b).expect("receptor");
        let t0 = Instant::now();
        let m = l
            .send_materials(a, receptor, 1_000, ts::salt_de(7 + i as u64))
            .expect("materiales");
        t_mat += t0.elapsed().as_secs_f64() * 1e3;

        let t0 = Instant::now();
        let envio = client::prove_send(&m, ka, proof_options()).expect("prueba");
        t_gen += t0.elapsed().as_secs_f64() * 1e3;

        let t0 = Instant::now();
        l.apply_send(&envio, a, &est, 1_000).expect("apply_send");
        t_apply += t0.elapsed().as_secs_f64() * 1e3;
        ops += 1;

        // ── cobro ──
        let est = estado(&l, b);
        let m = l.claim_materials(b, &envio.notice).expect("materiales");
        let cobro = client::prove_claim(&m, kb, proof_options()).expect("prueba");
        let t0 = Instant::now();
        l.apply_claim(&cobro, b, &est, &envio.notice)
            .expect("apply_claim");
        t_apply += t0.elapsed().as_secs_f64() * 1e3;
        ops += 1;
    }

    let d = ops.max(1) as f64;
    Punto {
        cuentas,
        rss: rss_mb() - rss0,
        ms_apply: t_apply / d,
        ms_materiales: t_mat / pagos.max(1) as f64,
        ms_generar: t_gen / pagos.max(1) as f64,
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
    let pagos: u32 = std::env::args()
        .nth(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(3);

    eprintln!("== BANCO B.3 · el apply con el arbol GRANDE ==");
    eprintln!("   tope: {tope} cuentas · {pagos} pagos cronometrados por escalon");
    eprintln!("   referencia: §209 midio ~3,1 ms de apply (~320 op/s) con DOS cuentas");
    eprintln!("   ⚠️ el relleno son cuentas VACIAS: abrir no lleva prueba\n");

    let rss0 = rss_mb();
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
        let t0 = Instant::now();
        let p = corrida(c, pagos, rss0);
        eprintln!(
            "   apply {:.2} ms · materiales {:.2} ms · generar {:.0} ms · RSS +{:.0} MB · montaje+medida {:.1} s",
            p.ms_apply,
            p.ms_materiales,
            p.ms_generar,
            p.rss,
            t0.elapsed().as_secs_f64()
        );
        puntos.push(p);
    }

    println!();
    println!("| cuentas | apply/operacion | techo implicito | send_materials | generar (ref) | RSS (MB) |");
    println!("|---|---|---|---|---|---|");
    for p in &puntos {
        println!(
            "| {} | **{:.2} ms** | **{:.0} op/s** | {:.2} ms | {:.0} ms | {:.0} |",
            p.cuentas,
            p.ms_apply,
            1000.0 / p.ms_apply.max(1e-9),
            p.ms_materiales,
            p.ms_generar,
            p.rss
        );
    }

    let primero = &puntos[0];
    let ultimo = &puntos[puntos.len() - 1];
    let e_apply = exponente(primero, ultimo, |p| p.ms_apply);
    let e_mat = exponente(primero, ultimo, |p| p.ms_materiales);
    let e_rss = exponente(primero, ultimo, |p| p.rss);

    println!();
    println!("== EXPONENTES (t ~ cuentas^e, entre {} y {}) ==", primero.cuentas, ultimo.cuentas);
    println!("  apply .......... e = {e_apply:.2}  ({:.1}x)", ultimo.ms_apply / primero.ms_apply.max(1e-9));
    println!("  send_materials . e = {e_mat:.2}  ({:.1}x)", ultimo.ms_materiales / primero.ms_materiales.max(1e-9));
    println!("  RSS ............ e = {e_rss:.2}");
    println!();
    println!("== LECTURA ==");
    println!("  §209 (2 cuentas) ....... ~3,1 ms · ~320 op/s");
    println!(
        "  aqui ({} cuentas) ... {:.2} ms · {:.0} op/s",
        ultimo.cuentas,
        ultimo.ms_apply,
        1000.0 / ultimo.ms_apply.max(1e-9)
    );
    println!("  objetivo RTGS de media (Fedwire, §204): ~21 op/s");
    println!();

    let techo = 1000.0 / ultimo.ms_apply.max(1e-9);
    if e_apply.is_finite() && e_apply > 0.35 {
        println!("VEREDICTO: ⚠️ EL APPLY CRECE CON EL NUMERO DE CUENTAS (e={e_apply:.2}).");
        println!("  El techo de ~320 op/s de §209 se midio con DOS cuentas y NO VALE");
        println!("  a escala. Hay que corregir §209 y §216 con la misma vara con la");
        println!("  que §205 corrigio ESCALADO §2.1: 'no derivar cifras, medirlas'.");
        println!("  Antes de tocar nada, averiguar QUE crece: el arbol (lo dice B.2),");
        println!("  `records`, el registro, o `commit`.");
    } else if e_mat.is_finite() && e_mat > 0.5 {
        println!("VEREDICTO: ⚠️ EL APPLY AGUANTA PERO `send_materials` NO (e={e_mat:.2}).");
        println!("  La cache de §207 no cubre el caso real a esta escala: aquel");
        println!("  arreglo se midio hasta 60 cuentas. Y `send_materials` corre EN EL");
        println!("  NODO en cada envio, asi que es techo igual.");
    } else if techo < 21.0 {
        println!("VEREDICTO: ⚠️ PLANO, PERO POR DEBAJO DEL OBJETIVO ({techo:.0} op/s).");
        println!("  No crece con las cuentas —bien— pero no llega a las ~21 op/s de");
        println!("  media que pide un RTGS. El limite esta en el coste fijo, no en la");
        println!("  escala: mirar que compone esos {:.2} ms.", ultimo.ms_apply);
    } else {
        println!("VEREDICTO: ✅ EL APPLY AGUANTA A {} CUENTAS.", ultimo.cuentas);
        println!("  e={e_apply:.2} (plano) y {techo:.0} op/s, por encima de las ~21 de");
        println!("  media que pide un RTGS. **Se declara medido HASTA AQUI**, no mas:");
        println!("  este banco no dice nada de 1e6 cuentas, ni del arranque desde");
        println!("  disco, ni del rendimiento concurrente a escala.");
    }
    println!();
    println!("⚠️ Todo en memoria. La persistencia es el 3 % (§204 A), pero `commit`");
    println!("   reescribe ~20 claves de metadatos por operacion y eso NO se mide");
    println!("   aqui. El arranque desde disco tampoco: es B.4.");
    println!();
    println!("Anota esta tabla en AUDITORIA.md junto a las de §204 y B.2.");
}
