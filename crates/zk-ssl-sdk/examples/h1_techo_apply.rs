//! **Banco H.1 — cuántas operaciones aplica el NODO por RPC.**
//!
//! ## Por qué existe, y qué error corrige
//!
//! D.2 midió **4,95 pagos/s** y de ahí se dedujo, durante dos sellos, que
//! el proyecto estaba a un factor 2 de su objetivo declarado (21 op/s de
//! media para un RTGS). **La resta no se había hecho.**
//!
//! De los 1.616 ms que cuestan ocho pagos en D.2:
//!
//! ```text
//!   aplicar 16 operaciones ...  59 ms   (3,67 ms cada una, §219)
//!   cuatro peticiones .......    6 ms   (0,255 ms fijo + bytes, §222)
//!   ───────────────────────────────────
//!   el NODO ................    65 ms  =  4 % del ciclo
//!   generar las pruebas ..... 1.552 ms = 96 %
//! ```
//!
//! **El nodo está al 4 % de ocupación.** Los 4,95 pagos/s no son una
//! propiedad suya: son una propiedad de generar las pruebas de las dos
//! partes en el mismo portátil de cuatro núcleos.
//!
//! Y en un despliegue real **cada titular prueba en su propia máquina**,
//! así que más titulares es más capacidad de prueba, no menos. Con 220-461
//! ms por prueba (§219), bastan **entre 5 y 10 titulares probando de
//! continuo** para saturar 21 op/s.
//!
//! ## Lo que este banco mide, y por qué solo él puede
//!
//! El techo de 272 op/s viene de B.3, que mide `apply` **en la capa, en
//! proceso**. Nadie ha medido cuántas operaciones aplica **el nodo por
//! RPC** —con verificación STARK, candado, deserialización y transporte—
//! cuando los probadores no son el cuello.
//!
//! `zkssl_applyMany` lo hace posible: todos los recibos de un lote van
//! contra **la misma raíz de arranque**, así que se pueden generar antes,
//! tardando lo que haga falta, y **cronometrar solo la petición**.
//!
//! Se miden lotes de 1, 4, 8 y 15 para separar el coste FIJO del coste POR
//! OPERACIÓN, igual que hizo E.2 con el transporte.
//!
//! ## Hipótesis, escritas ANTES del dato
//!
//! - **H1** · el coste por operación ronda los **3,7 ms** de B.3 más los
//!   ~0,15 ms de transportar sus 130 KB: **~3,85 ms/op**. B.3 midió que
//!   `apply` es PLANO en el número de cuentas (e=0,01), así que el árbol
//!   pequeño de este banco no debería halagar el resultado.
//! - **H2** · el coste fijo por petición es pequeño, **0,3-1 ms**.
//! - **H3** · quince operaciones en **~58 ms**, o sea **~250 op/s**. Con
//!   eso, el objetivo RTGS de 21 op/s estaría al **8 % del techo**, no a
//!   un factor 2.
//! - **H4** · ⚠️ si sale **por debajo de 150 op/s**, hay un coste del nodo
//!   que B.3 no ve —verificación más cara por RPC, deserialización, o
//!   contención del candado— y hay que buscarlo antes de afirmar nada.
//!
//! ## Cómo se usa
//!
//! ```text
//! cargo run --release -p zk-ssl-node -- --dev --listen 127.0.0.1:8647 &
//! cargo run --release -p zk-ssl-sdk --example h1_techo_apply -- http://127.0.0.1:8647 3
//! ```
//! (url, repeticiones por tamaño)
//!
//! ⚠️ **Muta el estado del nodo.** Usar uno desechable.
//! ⚠️ **Generar las pruebas NO entra en la medida** — y eso es todo el
//! propósito. El banco informa aparte de cuánto tardó, para que se vea el
//! contraste.

use std::time::Instant;

use serde_json::{json, Value};
use winterfell::math::fields::f64::BaseElement;

use zk_ssl::client::{self, SendMaterials};
use zk_ssl::commitment::ClientState;
use zk_ssl::proof_options;
use zk_ssl_sdk::{Account, Rpc, Wallet};
use zk_ssl_wire as wire;
use zk_ssl_wire::{digest_to_wire, Q};

type Digest = [BaseElement; 4];

struct Par {
    sa: [u64; 4],
    ia: u64,
    id_b: Digest,
}

fn estado(a: &Account) -> anyhow::Result<ClientState> {
    let v = a.view()?;
    Ok(ClientState { public_id: v.public_id, balance: v.balance, nonce: v.nonce })
}

fn sal(n: u64) -> Digest {
    [BaseElement::new(n), BaseElement::new(0), BaseElement::new(0), BaseElement::new(0)]
}

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let url = args.next().unwrap_or_else(|| "http://127.0.0.1:8647".into());
    let reps: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(3);

    // Quince es el maximo que cabe bajo el muro del cuerpo (§218, C0.2):
    // 15 x 132.728 = 1.990.920 B contra 2.097.152.
    let tamanos = [1usize, 4, 8, 15];
    let maximo = 15usize;

    eprintln!("== BANCO H.1 · el techo del NODO por RPC ==");
    eprintln!("   nodo: {url} · {reps} repeticiones por tamaño");
    eprintln!("   B.3 midio `apply` EN LA CAPA: 3,67 ms -> 272 op/s");
    eprintln!("   E.2 midio la peticion: 0,255 ms fijo · 808 MB/s");
    eprintln!("   se predice: ~3,85 ms/op, 15 ops en ~58 ms, ~250 op/s");
    eprintln!("   ⚠️ generar las pruebas NO entra en la medida\n");

    let rpc = Rpc::new(url.clone());
    let v: Value = rpc.call("zkssl_protocolVersion", json!([]))?;
    eprintln!("   protocolo: {v}");

    // ── Montaje: quince pares ──────────────────────────────────────
    eprintln!("-- montaje: {maximo} pares --");
    let mut pares: Vec<Par> = Vec::new();
    for i in 0..maximo {
        // ⚠️ Semillas hexadecimales de verdad, y fondeo por DEBAJO del tope
        // de suministro: quince cuentas a 10 M serian 150 M contra los
        // 100 M por defecto, y el fondeo fallaria a mitad de montaje.
        let sa: [u64; 4] = [0xA1A0 + i as u64 * 13, 1, 2, 3];
        let sb: [u64; 4] = [0xB1B0 + i as u64 * 13, 4, 5, 6];
        let wa = Wallet::from_elements(sa);
        let wb = Wallet::from_elements(sb);
        let a = Account::open(&rpc, wa)?;
        // El receptor se abre para que exista; su cuenta no se fondea
        // porque aqui solo se miden ENVIOS.
        Account::open(&rpc, wb)?;
        rpc.dev_fund(a.index, 1_000_000)?;
        pares.push(Par { sa, ia: a.index, id_b: wb.public_id() });
    }
    eprintln!("   {} pares abiertos y fondeados\n", pares.len());

    let mut filas: Vec<(usize, f64, f64)> = Vec::new(); // (n, ms_apply, ms_generar)

    for &n in &tamanos {
        for r in 0..reps {
            let ronda = (r * 100 + n) as u64;

            // 1 · materiales para los N primeros, TODOS contra la misma raiz
            let mut trabajos = Vec::new();
            for p in pares.iter().take(n) {
                let cuenta = Account::attach(&rpc, Wallet::from_elements(p.sa), p.ia);
                let est = estado(&cuenta)?;
                let m_dto: wire::SendMaterialsDto = rpc.call(
                    "zkssl_sendMaterials",
                    json!({
                        "sender": Q(p.ia),
                        "receiverId": digest_to_wire(&p.id_b),
                        "amount": Q(1_000u64),
                        "salt": digest_to_wire(&sal(ronda * 1_000 + p.ia)),
                    }),
                )?;
                let m: SendMaterials = (&m_dto).try_into()?;
                trabajos.push((p.sa.map(BaseElement::new), p.ia, est, m));
            }

            // 2 · las pruebas, EN PARALELO y FUERA del cronometro
            let t_gen = Instant::now();
            let recibos: Vec<_> = std::thread::scope(|s| {
                let hs: Vec<_> = trabajos
                    .iter()
                    .map(|(sk, ia, est, m)| {
                        s.spawn(move || {
                            let rec = client::prove_send(m, *sk, proof_options()).expect("prove");
                            (rec, *ia, est.clone())
                        })
                    })
                    .collect();
                hs.into_iter().map(|h| h.join().expect("hilo")).collect()
            });
            let ms_gen = t_gen.elapsed().as_secs_f64() * 1000.0;

            let ops: Vec<Value> = recibos
                .iter()
                .map(|(rec, ia, est)| {
                    json!({
                        "kind": "send",
                        "receipt": wire::SendReceiptDto::from(rec),
                        "sender": Q(*ia),
                        "senderState": wire::ClientStateDto::from(est),
                        "amount": Q(1_000u64),
                    })
                })
                .collect();
            let cuerpo = json!({ "ops": ops });
            let bytes = serde_json::to_vec(&cuerpo)?.len();

            // 3 · SOLO ESTO se cronometra
            let t = Instant::now();
            let res: Value = rpc.call("zkssl_applyMany", cuerpo)?;
            let ms = t.elapsed().as_secs_f64() * 1000.0;

            let aplicadas = res["applied"].as_array().map(|a| a.len()).unwrap_or(0);
            anyhow::ensure!(aplicadas == n, "el lote de {n} aplico {aplicadas}");

            eprintln!(
                "   n={n:>2} r={r} · APLICAR {ms:>7.2} ms ({:>6.2} ms/op) · \
                 cuerpo {:>9} B · generar {ms_gen:>7.0} ms",
                ms / n as f64,
                bytes
            );
            filas.push((n, ms, ms_gen));
        }
    }

    // ── lectura ────────────────────────────────────────────────────
    println!();
    println!("| ops | aplicar (ms) | ms/op | generar (ms) | el NODO es el |");
    println!("|---|---|---|---|---|");
    for &n in &tamanos {
        let de: Vec<&(usize, f64, f64)> = filas.iter().filter(|f| f.0 == n).collect();
        let m: f64 = de.iter().map(|f| f.1).sum::<f64>() / de.len() as f64;
        let g: f64 = de.iter().map(|f| f.2).sum::<f64>() / de.len() as f64;
        println!(
            "| {n} | {m:.2} | {:.2} | {g:.0} | **{:.1} %** |",
            m / n as f64,
            100.0 * m / (m + g)
        );
    }

    // recta t = a + b*n por minimos cuadrados
    let pts: Vec<(f64, f64)> = tamanos
        .iter()
        .map(|&n| {
            let de: Vec<&(usize, f64, f64)> = filas.iter().filter(|f| f.0 == n).collect();
            (n as f64, de.iter().map(|f| f.1).sum::<f64>() / de.len() as f64)
        })
        .collect();
    let k = pts.len() as f64;
    let sx: f64 = pts.iter().map(|p| p.0).sum();
    let sy: f64 = pts.iter().map(|p| p.1).sum();
    let sxx: f64 = pts.iter().map(|p| p.0 * p.0).sum();
    let sxy: f64 = pts.iter().map(|p| p.0 * p.1).sum();
    let b = (k * sxy - sx * sy) / (k * sxx - sx * sx);
    let a = (sy - b * sx) / k;
    let techo = 1000.0 / b;

    println!();
    println!("== LECTURA ==");
    println!("  coste FIJO por peticion ..... {a:.2} ms   (E.2 midio 0,255 en vacio)");
    println!("  coste POR OPERACION ......... {b:.2} ms   (B.3 midio 3,67 en la capa)");
    println!("  TECHO DEL NODO POR RPC ...... {techo:.0} op/s");
    println!();
    println!("  H1 (~3,85 ms/op): {}", if (2.5..5.5).contains(&b) { "ACIERTA" } else { "FALLA" });
    println!("  H2 (fijo 0,3-1 ms): {}", if (0.0..2.0).contains(&a) { "ACIERTA" } else { "FALLA" });
    let q15 = pts.last().map(|p| p.1).unwrap_or(0.0);
    println!("  H3 (15 ops en ~58 ms): {q15:.1} ms medidos — {}",
             if (40.0..90.0).contains(&q15) { "ACIERTA" } else { "FALLA" });
    println!();
    println!("== VEREDICTO ==");
    if techo < 150.0 {
        println!("  ⚠️ H4: {techo:.0} op/s, POR DEBAJO de 150. Hay un coste del nodo que");
        println!("     B.3 no ve —verificacion por RPC, deserializacion o contencion—.");
        println!("     Buscarlo ANTES de afirmar nada sobre el objetivo.");
    } else {
        println!("  El nodo aplica {techo:.0} op/s por RPC.");
        println!("  El objetivo RTGS de 21 op/s de media es el {:.1} % de ese techo.", 100.0 * 21.0 / techo);
        println!();
        println!("  ⚠️ Y LO QUE ESO SIGNIFICA, dicho con cuidado:");
        println!("     · el NODO no es el cuello, y no lo era cuando se dijo que si.");
        println!("     · los 4,95 pagos/s de D.2 son el portatil generando pruebas");
        println!("       para las DOS partes; el nodo estaba al 4 %.");
        println!("     · en despliegue real cada titular prueba en SU maquina: mas");
        println!("       titulares es MAS capacidad de prueba, no menos.");
    }
    println!();
    println!("⚠️ Lo que este banco NO mide, y no se debe suponer:");
    println!("   · CONCURRENCIA: un solo cliente manda los lotes en serie. Con N");
    println!("     agregadores a la vez, el Mutex de `dispatch` es el siguiente");
    println!("     sospechoso, y no esta medido.");
    println!("   · La ESCALA: el arbol de este banco tiene 30 cuentas. B.3 midio");
    println!("     que `apply` es PLANO (e=0,01) hasta 1e5, asi que no deberia");
    println!("     importar — pero eso es de B.3, no de aqui.");
    println!("   · El DISCO: el nodo corre en memoria, sin `--ledger`.");
    println!("   · Solo ENVIOS. Un cobro verifica otro circuito.");
    Ok(())
}
