//! **Métricas de la capa**, medidas con la metodología común del
//! proyecto.
//!
//! ## Por qué este módulo existe
//!
//! Las cifras que el proyecto documentaba venían de **circuitos sueltos**
//! medidos en momentos distintos, y varias quedaron obsoletas al crecer
//! el sistema: el circuito de emisión pasó de 99 a 118 restricciones al
//! añadir el umbral de custodios.
//!
//! Medir la capa completa, con su configuración real, evita que la
//! documentación describa un sistema anterior al que existe.
//!
//! ## Metodología
//!
//! - **Todo en release.** Una comparativa que mezcle debug y release no
//!   vale nada: el proyecto ya cometió ese error una vez y lo documentó
//!   como corrección.
//! - **Misma máquina, misma ejecución.** Los tiempos relativos entre
//!   operaciones son fiables; los absolutos dependen del hardware.
//! - **Configuración real de la capa** (`proof_options`), no la
//!   configuración ligera de los tests de circuito.
//!
//! ## ⚠️ Lo que estas cifras NO son
//!
//! Una sola ejecución en una máquina. Sirven para comparar órdenes de
//! magnitud entre operaciones, **no como benchmark riguroso**: falta
//! repetición, control de varianza y hardware documentado.
//!
//! Ejecutar con:
//!
//! ```text
//! cargo test -p zk-ssl --release metrics -- --nocapture
//! ```

// ===== LA CIFRA PUBLICADA - LA PARTE QUE EL NODO CONSUME =====
//
// **PUBLICADA_PAGO_B sale de `mod tests` en el §318 y se hace publica.**
// La razon del §304 para tenerla dentro CADUCO: entonces no habia
// consumidor y una const a nivel de fichero habria sido codigo muerto en
// release. Ahora el latido del nodo la lee para poder decir cuantos BYTES
// son los pagos acumulados, y no solo cuantos pagos. Un aviso que dice la
// magnitud que importa cumple el §254; uno que dice un recuento, no.
//
// **`mod metrics` iba tras `#[cfg(test)]` y era ademas PRIVADO**, asi que
// en un build sin tests el modulo NO EXISTIA y reexportar de el daba E0432.
// El §318 le quita ese atributo: el CONTENIDO del fichero sigue entero
// tras cfg(test), de modo que en release este modulo compila vacio salvo
// por esta constante. Quien la hace API es el `pub use` de `lib.rs`. Y ese
// gate NO lo ve `cargo test` -que compila con cfg(test), donde la const si
// tiene usuario-: lo ve un build SIN tests, que es por lo que VIVA A va
// primero.
//
// **Las otras cinco se quedan DENTRO, y no por gusto.** Una const privada
// a nivel de fichero usada solo desde codigo cfg(test) es codigo muerto en
// release: el racimo lo parte el compilador, no una preferencia. Su
// documentacion entera -fecha, unidad, determinismo, y el aviso de que
// quien la mueva mueve los documentos- sigue abajo, en `mod tests`, y vale
// igual para esta.
//
// UNIDAD: bytes de UN pago, que son DOS pruebas (envio + cobro). Mil pagos
// son PUBLICADA_PAGO_B * 1000 / 2^20 MiB. Medida el 2026-08-14 y remedida
// el 2026-08-17 sin variacion. Quien la mueva mueve tambien los
// documentos: `tools/check_publicadas.py` los ata y dice cuales faltan.
pub const PUBLICADA_PAGO_B: usize = 132_311;

#[cfg(test)]
mod tests {
    // El arnes mide la via ANTIGUA a proposito: es la que las cifras
    // publicadas describen. Cuando se migre (entrada 32) las mediciones
    // cambian con ella, y eso hay que declararlo, no absorberlo. §65.3: el
    // permiso va aqui, no en la definicion.

    use crate::tests_support::*;
    use crate::*;
    use std::time::{Duration, Instant};
    use winterfell::math::fields::f64::BaseElement;

    // ===== LA CIFRA PUBLICADA - FUENTE UNICA =====
    //
    // Medida el 2026-08-14, en release, con la configuracion real de la
    // capa (proof_options) y el montaje de metrics_of_the_layer.
    //
    // UNIDAD: MiB = 2^20. NO son MB de 10^6. Un lector que tome MB
    // decimal lee un 9,9 % menos de lo real. La etiqueta ya la cazo
    // AUDITORIA.md §83.3 y nadie la arreglo.
    //
    // El tamano de una prueba es DETERMINISTA para las MISMAS entradas
    // (cuatro corridas, byte a byte), pero varia ~4 % entre entradas
    // distintas: lo mide proof_size_does_not_correlate_with_amount. Por
    // eso el gate exacto exige un montaje GEMELO, y por eso existe
    // medir_el_pago_publicado.
    //
    // Quien mueva esta constante mueve tambien los documentos:
    // tools/check_publicadas.py los ata y dice cuales faltan.
    const PUBLICADA_FECHA: &str = "2026-08-14";
    const PUBLICADA_ENVIO_B: usize = 66_998;
    const PUBLICADA_COBRO_B: usize = 65_313;
    // PUBLICADA_PAGO_B vive AHORA a nivel de fichero y es publica
    // (§318): el latido del nodo la consume. Llega hasta aqui por el
    // `use crate::*` de arriba, via el `pub use` de lib.rs.
    const PUBLICADA_MIL_MIB: &str = "126,2";

    // La RELACION va con BANDA y no con valor: los bytes no dependen de
    // la maquina, los tiempos SI. Medido cuatro veces: envio 260,7-286,8
    // ms, cobro 159,4-189,6 ms. Las bandas no se solapan. Se afirma el
    // SENTIDO con margen, nunca un valor.
    //
    // Esto importa mas que los bytes: los preprints afirman que la mitad
    // cara cae en el RECEPTOR (cobro ~500 ms contra 283 ms de envio) y
    // construyen sobre ello un argumento normativo. Hoy es al reves. Los
    // preprints los suspende la entrada 28; este gate impide que la
    // inversion vuelva a pasar inadvertida.
    const PUBLICADA_ENVIO_SOBRE_COBRO_MIN: f64 = 1.20;

    /// El montaje que produce la cifra publicada.
    ///
    /// **GEMELO del de metrics_of_the_layer**: mismas cuentas, mismos
    /// importes, misma semilla. Tiene que serlo, porque el tamano de una
    /// prueba solo es exacto para las mismas entradas. Si uno de los dos
    /// cambia y el otro no, este atado miente.
    fn medir_el_pago_publicado() -> (usize, usize, f64, f64) {
        let mut layer = new_layer();
        #[allow(deprecated)]
        let alice = layer.open_account(BaseElement::new(SK_ALICE));
        #[allow(deprecated)]
        let bob = layer.open_account(BaseElement::new(SK_BOB));
        let op = mint_commitment(&layer, alice, 1_000_000);
        let subida = mint_climb_proof(&layer, alice, 1_000_000);
        let (pa, ia, pb, ib) = delegated_pair(op, 1, 3);
        layer
            .apply_mint_delegated(subida, pa, ia, pb, ib, alice, 1_000_000)
            .expect("aplicar emision");
        fund_delegated(&mut layer, bob, 50_000);
        let estado_a = state_of(&layer, alice);
        let receptor = layer.public_id_of(bob).expect("cuenta");
        let t = Instant::now();
        let envio = layer
            .send(
                BaseElement::new(SK_ALICE),
                alice,
                &estado_a,
                receptor,
                salt_de(0x11E7),
                250_000,
            )
            .expect("envio");
        let envio_ms = t.elapsed().as_secs_f64() * 1000.0;
        layer
            .apply_send(&envio, alice, &estado_a, 250_000)
            .expect("aplicar envio");
        let estado_b = state_of(&layer, bob);
        let t = Instant::now();
        let cobro = layer
            .claim(BaseElement::new(SK_BOB), bob, &estado_b, &envio.notice)
            .expect("cobro");
        let cobro_ms = t.elapsed().as_secs_f64() * 1000.0;
        layer
            .apply_claim(&cobro, bob, &estado_b, &envio.notice)
            .expect("aplicar cobro");
        (envio.proof.len(), cobro.proof.len(), envio_ms, cobro_ms)
    }

    fn ms(d: Duration) -> f64 {
        d.as_secs_f64() * 1000.0
    }

    fn line(label: &str, gen: Duration, apply: Duration, bytes: usize) {
        println!(
            "{label:<22} generar {:>8.1} ms | aplicar {:>7.1} ms | prueba {:>7} B",
            ms(gen),
            ms(apply),
            bytes
        );
    }

    /// **Las métricas de la capa.**
    ///
    /// Separa deliberadamente **generar** de **aplicar**, porque son
    /// operaciones de partes distintas: quien produce la prueba puede no
    /// ser quien la acepta. Confundirlas en un solo número escondería que
    /// la verificación es dos órdenes de magnitud más barata.
    /// **Cuanto cuesta VERIFICAR una prueba, sola.**
    ///
    /// ⚠️ `apply` no responde a esto: verifica, muta el arbol **y escribe a
    /// disco**. Los tres juntos son lo que mide `metrics_of_the_layer`.
    ///
    /// Lo pregunta la propuesta de sharding —entrada 47 del backlog,
    /// corregida en §89—, que dimensiona el shard sobre **4 ms por
    /// prueba** presentandolos como medidos —y no lo estaban—. De ese numero
    /// cuelga la primera etapa del cuello de botella:
    ///
    /// ```text
    ///   4 ms -> 8.000 TPS/shard ->   64 shards
    ///  20 ms -> 1.600 TPS/shard -> ~310 shards
    ///  74 ms ->   430 TPS/shard -> ~1150 shards
    /// ```
    ///
    /// **INSTRUMENTO, no comprobacion.** Correr en release:
    ///
    /// ```text
    /// cargo test --release -p zk-ssl el_coste_de_verificar \
    ///     -- --ignored --nocapture
    /// ```
    #[test]
    #[ignore = "instrumento de medida, no comprobacion: correr a mano"]
    fn el_coste_de_verificar_una_prueba() {
        use stark_experiment::circuit_claim::ClaimAir;
        use stark_experiment::circuit_send::SendAir;
        use std::time::Instant;

        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        let estado_a = state_of(&layer, alice);
        let receptor = layer.public_id_of(bob).expect("cuenta");

        let envio = layer
            .send(
                BaseElement::new(SK_ALICE),
                alice,
                &estado_a,
                receptor,
                salt_de(0x11E7),
                250_000,
            )
            .expect("envio");

        // El `apply` completo, para poder repartirlo despues.
        let t = Instant::now();
        layer
            .apply_send(&envio, alice, &estado_a, 250_000)
            .expect("aplicar envio");
        let send_apply = t.elapsed().as_secs_f64() * 1000.0;

        let estado_b = state_of(&layer, bob);
        let cobro = layer
            .claim(BaseElement::new(SK_BOB), bob, &estado_b, &envio.notice)
            .expect("cobro");
        let t = Instant::now();
        layer
            .apply_claim(&cobro, bob, &estado_b, &envio.notice)
            .expect("aplicar cobro");
        let claim_apply = t.elapsed().as_secs_f64() * 1000.0;

        let accepted = AcceptableOptions::OptionSet(vec![proof_options()]);

        // ===== VERIFICAR, Y SOLO VERIFICAR =====
        //
        // Ni arbol, ni disco, ni las comprobaciones de estado de la capa.
        // Es lo unico que un shard paraleliza.
        const N: usize = 50;

        let mut send_ver = 0.0;
        for i in 0..=N {
            let p = winterfell::Proof::from_bytes(&envio.proof).expect("prueba");
            let t = Instant::now();
            verify::<SendAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
                p,
                envio.public_inputs.clone(),
                &accepted,
            )
            .expect("el envio honesto debe verificar");
            // La primera es calentamiento y no cuenta.
            if i > 0 {
                send_ver += t.elapsed().as_secs_f64() * 1000.0;
            }
        }
        send_ver /= N as f64;

        let mut claim_ver = 0.0;
        for i in 0..=N {
            let p = winterfell::Proof::from_bytes(&cobro.proof).expect("prueba");
            let t = Instant::now();
            verify::<ClaimAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
                p,
                cobro.public_inputs.clone(),
                &accepted,
            )
            .expect("el cobro honesto debe verificar");
            if i > 0 {
                claim_ver += t.elapsed().as_secs_f64() * 1000.0;
            }
        }
        claim_ver /= N as f64;

        println!("\n=== Coste de VERIFICAR, aislado ===\n");
        println!("  {:<10} {:>10} {:>10} {:>10}", "", "verificar", "apply", "resto");
        println!(
            "  {:<10} {:>8.2} ms {:>8.1} ms {:>8.1} ms",
            "envio", send_ver, send_apply, send_apply - send_ver
        );
        println!(
            "  {:<10} {:>8.2} ms {:>8.1} ms {:>8.1} ms",
            "cobro", claim_ver, claim_apply, claim_apply - claim_ver
        );
        println!();
        let media = (send_ver + claim_ver) / 2.0;
        let por_seg = 1000.0 / media;
        println!("  media por prueba        {media:>8.2} ms");
        println!("  pruebas/s por nucleo    {por_seg:>8.0}");
        println!("  64 nucleos              {:>8.0} TPS", por_seg * 64.0);
        println!(
            "  con margen del 50 %     {:>8.0} TPS/shard",
            por_seg * 64.0 * 0.5
        );
        let shards = 498_000.0 / (por_seg * 64.0 * 0.5);
        println!("  shards para 498.000 TPS {shards:>8.0}");
        println!();
        println!("  ⚠️ La propuesta de sharding (entrada 47, §89) supone 4 ms");
        println!("     presenta como MEDIDOS. Este es el numero real, y §11");
        println!("     de ese documento no lo lista entre sus incertidumbres.");
        println!();
        println!("  ⚠️ Esto mide UN nucleo de esta maquina con estas");
        println!("     `proof_options`. No mide un shard.");
    }

    #[test]
    fn metrics_of_the_layer() {
        println!("\n=== ZK-SSL — metricas de la capa (release) ===\n");

        // --- Arranque ---
        let t = Instant::now();
        let mut layer = new_layer();
        let startup = t.elapsed();
        println!(
            "Arranque de la capa    {:.3} ms  (sin ceremonia, sin claves que generar)",
            ms(startup)
        );

        // `open_account` (64 bits) sigue viva y opt-in (§97.4, fuera de B
        // por §160); aquí se usa a propósito para medir la capa real.
        #[allow(deprecated)]
        let alice = layer.open_account(BaseElement::new(SK_ALICE));
        #[allow(deprecated)]
        let bob = layer.open_account(BaseElement::new(SK_BOB));

        // --- Emision con umbral 2-de-N ---
        let t = Instant::now();
        let op = mint_commitment(&layer, alice, 1_000_000);
        let subida = mint_climb_proof(&layer, alice, 1_000_000);
        let (pa, ia, pb, ib) = delegated_pair(op, 1, 3);
        let mint_gen = t.elapsed();
        let mint_bytes = subida.to_bytes().len();

        let t = Instant::now();
        layer
            .apply_mint_delegated(subida, pa, ia, pb, ib, alice, 1_000_000)
            .expect("aplicar emision");
        let mint_apply = t.elapsed();

        // Fondos para el receptor, sin medir.
        fund_delegated(&mut layer, bob, 50_000);

        // --- Transferencia ---
        // ⚠️ **UN PAGO SON DOS PRUEBAS, NO UNA.**
        //
        // La via de produccion es `send` + `claim`. Medir solo `transfer`
        // —la via retirada— daba **la mitad** del coste real por pago y de
        // la acumulacion de pruebas. Ver `AUDITORIA.md` §31.
        let estado_a = state_of(&layer, alice);
        let receptor = layer.public_id_of(bob).expect("cuenta");

        let t = Instant::now();
        let envio = layer
            .send(
                BaseElement::new(SK_ALICE),
                alice,
                &estado_a,
                receptor,
                salt_de(0x11E7),
                250_000,
            )
            .expect("envio");
        let send_gen = t.elapsed();
        let send_bytes = envio.proof.len();

        let t = Instant::now();
        layer
            .apply_send(&envio, alice, &estado_a, 250_000)
            .expect("aplicar envio");
        let send_apply = t.elapsed();

        let estado_b = state_of(&layer, bob);
        let t = Instant::now();
        let cobro = layer
            .claim(BaseElement::new(SK_BOB), bob, &estado_b, &envio.notice)
            .expect("cobro");
        let claim_gen = t.elapsed();
        let claim_bytes = cobro.proof.len();

        let t = Instant::now();
        layer
            .apply_claim(&cobro, bob, &estado_b, &envio.notice)
            .expect("aplicar cobro");
        let claim_apply = t.elapsed();

        // El coste de UN PAGO completo: las dos fases sumadas.
        let tx_gen = send_gen + claim_gen;
        let tx_apply = send_apply + claim_apply;
        let tx_bytes = send_bytes + claim_bytes;

        // --- Destruccion ---
        let t = Instant::now();
        let burn = layer
            .burn(BaseElement::new(SK_BOB), bob, &state_of(&layer, bob), 100_000)
            .expect("destruccion");
        let burn_gen = t.elapsed();
        let burn_bytes = burn.proof.len();

        let t = Instant::now();
        let estado_bob = state_of(&layer, bob);
        layer.apply_burn(&burn, bob, &estado_bob).expect("aplicar destruccion");
        let burn_apply = t.elapsed();

        // --- Auditoria ---
        let t = Instant::now();
        let disclosure = layer
            .audit(BaseElement::new(SK_ALICE), alice, &state_of(&layer, alice), 700_000, 800_000)
            .expect("auditoria");
        let audit_gen = t.elapsed();
        let audit_bytes = disclosure.proof.len();

        let t = Instant::now();
        verify_audit(&disclosure).expect("verificar auditoria");
        let audit_verify = t.elapsed();

        println!();
        line("Emision (2-de-N)", mint_gen, mint_apply, mint_bytes);
        line("Transferencia", tx_gen, tx_apply, tx_bytes);
        line("Destruccion", burn_gen, burn_apply, burn_bytes);
        line("Auditoria (banda)", audit_gen, audit_verify, audit_bytes);

        line("  ├─ envio (send)", send_gen, send_apply, send_bytes);
        line("  └─ cobro (claim)", claim_gen, claim_apply, claim_bytes);

        println!("\n--- Lecturas ---");
        println!(
            "Verificar / generar (pago completo): {:.1}%",
            100.0 * tx_apply.as_secs_f64() / tx_gen.as_secs_f64()
        );
        println!(
            "Auditar / transferir (generacion):   {:.1}%",
            100.0 * audit_gen.as_secs_f64() / tx_gen.as_secs_f64()
        );
        println!(
            "Jornada de 1.000 pagos (envio+cobro): {:.1} s de prueba, {:.1} MiB acumulados",
            tx_gen.as_secs_f64() * 1000.0,
            (tx_bytes * 1000) as f64 / 1_048_576.0
        );

        // --- Comprobaciones de coherencia ---
        //
        // Sin esto el test seria solo un impresor de numeros y podria
        // pasar aunque el sistema estuviera roto.
        assert!(
            startup < Duration::from_millis(500),
            "el arranque debe ser inmediato: no hay ceremonia ni claves que generar"
        );
        assert!(
            tx_apply < tx_gen,
            "verificar debe ser mas barato que generar"
        );
        assert!(
            audit_gen < tx_gen,
            "auditar debe ser mas barato que transferir: una subida de arbol \
             frente a las del envio (cuentas dual, congelados y pendientes)"
        );
        // ===== EL TAMAÑO SÍ SE COMPRUEBA =====
        //
        // Los tiempos dependen de la maquina y por eso arriba solo se
        // afirman RELACIONES. **El tamaño de la prueba no depende de la
        // maquina**: es determinista, funcion del circuito y de las
        // opciones de prueba.
        //
        // Y es la cifra que los documentos publican. Sin esta comprobacion,
        // un cambio que engordara la prueba dejaria esas cifras falsas **sin
        // que nada lo detectara**.
        //
        // ⚠️ El margen es amplio a proposito: se trata de detectar un cambio
        // de orden de magnitud, no de fijar el byte exacto.
        //
        // ⚠️⚠️ **ESTA GUARDA SALTO AL MIGRAR A LA VIA EN DOS FASES**, y tenia
        // razon. Cada prueba medía ~62 KB; §304 remidio 65,4 y 63,8 KB —eso no ha cambiado— pero
        // **un pago son ahora DOS pruebas**, asi que la acumulacion por mil
        // pagos paso de 59,1 MB a 120,4 MB, y en §304 a **126,2 MiB**.
        //
        // La cifra vieja no era un error de medicion: medía una operacion
        // que dejo de ser la de produccion. Ver `AUDITORIA.md` §31.
        assert!(
            (50_000..80_000).contains(&send_bytes),
            "la prueba de ENVIO mide {send_bytes} bytes; se esperan ~62 KB"
        );
        assert!(
            (50_000..80_000).contains(&claim_bytes),
            "la prueba de COBRO mide {claim_bytes} bytes; se esperan ~62 KB"
        );
        assert!(
            (100_000..160_000).contains(&tx_bytes),
            "un PAGO COMPLETO mide {tx_bytes} bytes. Los documentos publican \
             ~129 KiB por pago y 126,2 MiB por cada mil: si el tamaño ha \
             cambiado de orden, esas cifras son falsas"
        );

        assert_eq!(layer.total_supply(), 950_000);
        let sum: u64 = [alice, bob]
            .iter()
            .map(|i| layer.balance_of(*i).unwrap())
            .sum();
        assert_eq!(
            sum,
            layer.total_supply(),
            "la invariante global debe mantenerse tras la secuencia completa"
        );
    }

    /// **El coste de una jornada.**
    ///
    /// Encadena transferencias reales y mide si el coste por operación se
    /// mantiene estable. Si creciera —por los árboles llenándose o por
    /// otra causa— el sistema no escalaría linealmente y eso hay que
    /// saberlo.
    /// **ATADO A - el instrumento contra la cifra publicada.**
    ///
    /// Falla si el SISTEMA se movio y la constante no.
    ///
    /// Su gemelo vive en tools/check_publicadas.py y falla si la
    /// CONSTANTE se movio y los documentos no. Dos eslabones, dos
    /// mensajes distintos: cada rojo dice cual de las dos cosas paso.
    ///
    /// Sin atributo de ignorar: es una COMPROBACION, no un instrumento
    /// de medida. Un gate ignorado es un gate que no puede hablar.
    #[test]
    fn la_cifra_publicada_sigue_siendo_la_medida() {
        let (envio_b, cobro_b, envio_ms, cobro_ms) = medir_el_pago_publicado();

        assert_eq!(
            envio_b, PUBLICADA_ENVIO_B,
            "el ENVIO mide {} B; la constante dice {} (medida el {}). El \
             sistema se movio: remide, mueve la constante y deja que \
             check_publicadas.py senale los documentos que se quedan atras",
            envio_b, PUBLICADA_ENVIO_B, PUBLICADA_FECHA
        );
        assert_eq!(
            cobro_b, PUBLICADA_COBRO_B,
            "el COBRO mide {} B; la constante dice {} (medida el {})",
            cobro_b, PUBLICADA_COBRO_B, PUBLICADA_FECHA
        );
        assert_eq!(
            envio_b + cobro_b,
            PUBLICADA_PAGO_B,
            "un PAGO mide {} B; la constante dice {}. Un pago son DOS \
             pruebas: envio + cobro",
            envio_b + cobro_b,
            PUBLICADA_PAGO_B
        );

        // La jornada de mil pagos, DERIVADA aqui y no recordada.
        // MiB = 2^20, con una decimal y coma, como la escriben los
        // documentos en castellano.
        let mil = (PUBLICADA_PAGO_B * 1000) as f64 / 1_048_576.0;
        let escrito = format!("{:.1}", mil).replace('.', ",");
        assert_eq!(
            escrito, PUBLICADA_MIL_MIB,
            "la jornada de mil pagos deriva {} MiB y la constante dice {} \
             MiB. La unidad es 2^20: quien escriba MB de 10^6 publica un \
             9,9 % menos",
            escrito, PUBLICADA_MIL_MIB
        );

        // La RELACION, con banda. Afirma el sentido, no el valor.
        assert!(
            envio_ms > cobro_ms * PUBLICADA_ENVIO_SOBRE_COBRO_MIN,
            "generar el ENVIO tardo {:.1} ms y el COBRO {:.1} ms. Se exige \
             envio > cobro x {:.2}. Si esto se invierte, la mitad cara pasa \
             al receptor y el argumento normativo de los preprints (entrada \
             28) cambia de sentido: no se absorbe, se declara",
            envio_ms, cobro_ms, PUBLICADA_ENVIO_SOBRE_COBRO_MIN
        );
    }

    #[test]
    fn cost_per_transfer_stays_stable() {
        const N: usize = 5;
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);

        println!("\n=== Coste por envio encadenado ===\n");
        let mut times = Vec::with_capacity(N);
        for i in 0..N {
            let estado = state_of(&layer, alice);
            let receptor = layer.public_id_of(bob).expect("cuenta");
            let t = Instant::now();
            let s = layer
                .send(
                    BaseElement::new(SK_ALICE),
                    alice,
                    &estado,
                    receptor,
                    salt_de(0xE57A + i as u64),
                    10_000,
                )
                .expect("envio");
            let gen = t.elapsed();
            layer.apply_send(&s, alice, &estado, 10_000).expect("aplicar");
            // El receptor cobra en el acto: lo que se mide es que el coste no
            // crezca con el numero de operaciones, y para eso hacen falta las
            // dos fases o el arbol de pendientes creceria sin vaciarse nunca.
            let estado_b = state_of(&layer, bob);
            let cr = layer
                .claim(BaseElement::new(SK_BOB), bob, &estado_b, &s.notice)
                .expect("cobro");
            layer
                .apply_claim(&cr, bob, &estado_b, &s.notice)
                .expect("aplicar cobro");
            times.push(gen);
            println!("  envio {:>2}: {:>7.1} ms", i + 1, ms(gen));
        }

        let first = times[0].as_secs_f64();
        let last = times[N - 1].as_secs_f64();
        println!("\n  ultima / primera: {:.2}x", last / first);

        assert!(
            last < first * 2.0,
            "el coste por envio no debe crecer con el numero de \
             operaciones. primera={first:.3}s ultima={last:.3}s"
        );
        assert_eq!(layer.balance_of(bob), Some(50_000));
    }

    /// **El tamaño de la prueba varía, y eso mereció investigarse.**
    ///
    /// ## Lo que suponía y era falso
    ///
    /// La primera versión de este test asertaba que el tamaño era
    /// idéntico para cualquier importe. Falló. Con más muestras, la
    /// variación resultó ser de **unos 700 bytes sobre 63.000 (1,1%)**.
    ///
    /// ## Por qué varía
    ///
    /// Una prueba STARK incluye caminos de autenticación de Merkle para
    /// las posiciones consultadas. Esas posiciones salen de Fiat-Shamir
    /// —del hash del compromiso de la traza— y cuántos nodos comparten
    /// entre sí cambia de una prueba a otra. Al deduplicarlos, el tamaño
    /// oscila.
    ///
    /// ## Por qué importa
    ///
    /// Si el tamaño **correlacionara** con el importe, filtraría
    /// información sobre una operación que se supone privada. Un
    /// observador que solo viera el tamaño de las pruebas podría inferir
    /// magnitudes.
    ///
    /// ## Qué mide este test, y qué fuerza tiene
    ///
    /// Genera pruebas con importes que abarcan cuatro órdenes de
    /// magnitud y calcula la **correlación de Pearson** entre importe y
    /// tamaño.
    ///
    /// ⚠️ **Es evidencia débil, no demostración.** Con esta cantidad de
    /// muestras, una correlación espuria de magnitud moderada es
    /// perfectamente posible por azar. Descartar la fuga exigiría
    /// centenares de muestras y un análisis estadístico serio, que **no
    /// se ha hecho**. Lo que este test descarta es una fuga *grosera*.
    #[test]
    fn proof_size_does_not_correlate_with_amount() {
        const N: usize = 16;
        let mut layer = new_layer();
        // Cada transferencia va capada al limite regulatorio, asi que el
        // fondo necesario es N por ese limite — no una cifra arbitraria,
        // que ademas chocaria con el tope de emision.
        let needed = LIMIT * N as u64;
        assert!(needed <= MAX_SUPPLY, "el fondo cabe en el tope de emision");
        let alice = open_and_fund(&mut layer, SK_ALICE, needed);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);

        // Importes repartidos por varios ordenes de magnitud, todos
        // dentro del limite.
        let amounts: Vec<u64> = (0..N)
            .map(|i| (1u64 << (i + 3)).min(LIMIT))
            .collect();

        println!("\n=== Tamano de prueba frente a importe ===");
        let mut samples: Vec<(f64, f64)> = Vec::with_capacity(N);
        // ⚠️ **Se mide `send`, no `transfer`.**
        //
        // Lo que este bucle demuestra —que el tamaño de la prueba **no
        // depende del importe**— hay que demostrarlo sobre la via que se
        // ejecuta. Sobre la retirada no dice nada de produccion.
        for (i, amount) in amounts.iter().enumerate() {
            let capped = (*amount).min(layer.regulatory_limit());
            let estado = state_of(&layer, alice);
            let receptor = layer.public_id_of(bob).expect("cuenta");
            let s = layer
                .send(
                    BaseElement::new(SK_ALICE),
                    alice,
                    &estado,
                    receptor,
                    salt_de(0x7A11 + i as u64),
                    capped,
                )
                .expect("envio");
            println!("  importe {:>9} → {} B", capped, s.proof.len());
            samples.push((capped as f64, s.proof.len() as f64));
        }

        let n = samples.len() as f64;
        let mean_x = samples.iter().map(|(x, _)| x).sum::<f64>() / n;
        let mean_y = samples.iter().map(|(_, y)| y).sum::<f64>() / n;
        let cov: f64 = samples
            .iter()
            .map(|(x, y)| (x - mean_x) * (y - mean_y))
            .sum::<f64>();
        let var_x: f64 = samples.iter().map(|(x, _)| (x - mean_x).powi(2)).sum();
        let var_y: f64 = samples.iter().map(|(_, y)| (y - mean_y).powi(2)).sum();
        let r = if var_x > 0.0 && var_y > 0.0 {
            cov / (var_x.sqrt() * var_y.sqrt())
        } else {
            0.0
        };

        // Correlacion tambien en escala LOGARITMICA, que es la natural
        // cuando los importes abarcan varios ordenes de magnitud: una
        // fuga plausible dependeria del numero de bits del importe, no de
        // su valor absoluto.
        let log_samples: Vec<(f64, f64)> =
            samples.iter().map(|(x, y)| (x.log2(), *y)).collect();
        let mean_lx = log_samples.iter().map(|(x, _)| x).sum::<f64>() / n;
        let cov_l: f64 = log_samples
            .iter()
            .map(|(x, y)| (x - mean_lx) * (y - mean_y))
            .sum();
        let var_lx: f64 = log_samples
            .iter()
            .map(|(x, _)| (x - mean_lx).powi(2))
            .sum();
        let r_log = if var_lx > 0.0 && var_y > 0.0 {
            cov_l / (var_lx.sqrt() * var_y.sqrt())
        } else {
            0.0
        };

        let sizes: Vec<usize> = samples.iter().map(|(_, y)| *y as usize).collect();
        let min = *sizes.iter().min().unwrap();
        let max = *sizes.iter().max().unwrap();
        println!(
            "\n  variacion: {} B sobre {} B ({:.2}%)",
            max - min,
            min,
            100.0 * (max - min) as f64 / min as f64
        );
        println!("  correlacion de Pearson importe/tamano:      {r:+.3}");
        println!("  correlacion log2(importe)/tamano:            {r_log:+.3}");
        println!(
            "  ⚠️  evidencia DEBIL con {N} muestras: descarta una fuga grosera, \
             no demuestra ausencia de fuga"
        );

        assert!(
            r.abs() < 0.7,
            "CRITICO: correlacion {r:+.3} entre importe y tamano de prueba. \
             Una correlacion fuerte significaria que el tamano filtra \
             informacion sobre una operacion que se supone privada."
        );
        // La correlacion en escala logaritmica es la comprobacion
        // principal: una fuga plausible dependeria del numero de bits del
        // importe.
        assert!(
            r_log.abs() < 0.7,
            "CRITICO: correlacion {r_log:+.3} entre log2(importe) y tamano"
        );

        // NOTA: **no** se comprueba que la variacion absoluta sea pequena.
        //
        // Una version anterior de este test lo hacia, con un umbral
        // elegido a ojo, y fallo dos veces por adivinar el numero en vez
        // de medir lo que importa. La magnitud de la variacion (~5%) no
        // es una fuga si no correlaciona con el secreto; lo que habria que
        // comprobar, y aqui se comprueba, es la correlacion.
    }
}

#[cfg(test)]
mod remedicion_89_1 {
    //! Re-medicion §89.1 (prerrequisito bloqueante del informe de sesion):
    //! caracterizar el instrumento ANTES de creer numeros. Una ejecucion
    //! del proceso = UNA muestra (correr 5 veces, en release, a mano).
    //! Cada muestra imprime gen, apply y el tamaño REAL de la prueba —
    //! el paso 1 del protocolo — para send y para claim.
    use crate::tests_support::*;
    use std::time::Instant;
    use winterfell::math::fields::f64::BaseElement;

    const SK_A: u64 = 0x89A1;
    const SK_B: u64 = 0x89B1;

    #[test]
    #[ignore = "instrumento de medida, no comprobacion: correr a mano, 5x, en release"]
    fn muestra() {
        let mut layer = new_layer();
        let a = open_and_fund(&mut layer, SK_A, 1_000_000);
        let b = open_and_fund(&mut layer, SK_B, 0);
        let ea = state_of(&layer, a);
        let receptor = layer.public_id_of(b).expect("cuenta");

        let t = Instant::now();
        let envio = layer
            .send(BaseElement::new(SK_A), a, &ea, receptor, salt_de(0x891), 250_000)
            .expect("envio");
        let gen_s = t.elapsed().as_secs_f64() * 1e3;
        let t = Instant::now();
        layer.apply_send(&envio, a, &ea, 250_000).expect("aplicar envio");
        let ap_s = t.elapsed().as_secs_f64() * 1e3;
        eprintln!(
            "MUESTRA send  gen={gen_s:7.1}ms apply={ap_s:6.1}ms proof={} B",
            envio.proof.len()
        );

        let eb = state_of(&layer, b);
        let t = Instant::now();
        let cobro = layer
            .claim(BaseElement::new(SK_B), b, &eb, &envio.notice)
            .expect("cobro");
        let gen_c = t.elapsed().as_secs_f64() * 1e3;
        let t = Instant::now();
        layer
            .apply_claim(&cobro, b, &eb, &envio.notice)
            .expect("aplicar cobro");
        let ap_c = t.elapsed().as_secs_f64() * 1e3;
        eprintln!(
            "MUESTRA claim gen={gen_c:7.1}ms apply={ap_c:6.1}ms proof={} B",
            cobro.proof.len()
        );
    }
}
