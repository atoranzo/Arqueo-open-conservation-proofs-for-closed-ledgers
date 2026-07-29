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

#[cfg(test)]
mod tests {
    use crate::tests_support::*;
    use crate::*;
    use std::time::{Duration, Instant};
    use winterfell::math::fields::f64::BaseElement;

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

        let alice = layer.open_account(BaseElement::new(SK_ALICE));
        let bob = layer.open_account(BaseElement::new(SK_BOB));

        // --- Emision con umbral 2-de-N ---
        let t = Instant::now();
        let receipt = layer
            .mint(&valid_auth(), alice, 1_000_000)
            .expect("emision");
        let mint_gen = t.elapsed();
        let mint_bytes = receipt.proof.len();

        let t = Instant::now();
        layer.apply_mint(&receipt, alice).expect("aplicar emision");
        let mint_apply = t.elapsed();

        // Fondos para el receptor, sin medir.
        let r2 = layer.mint(&valid_auth(), bob, 50_000).expect("emision 2");
        layer.apply_mint(&r2, bob).expect("aplicar");

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
            "Jornada de 1.000 pagos (envio+cobro): {:.1} s de prueba, {:.1} MB acumulados",
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
             frente a cuatro mas el arbol de nullifiers"
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
        // razon. Cada prueba sigue midiendo ~62 KB —eso no ha cambiado— pero
        // **un pago son ahora DOS pruebas**, asi que la acumulacion por mil
        // pagos pasa de 59,1 MB a **120,4 MB**.
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
             ~126 KB por pago y 120,4 MB por cada mil: si el tamaño ha \
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
    /// mantiene estable. Si creciera —por el árbol de nullifiers llenándose
    /// o por otra causa— el sistema no escalaría linealmente y eso hay que
    /// saberlo.
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
        for amount in &amounts {
            let capped = (*amount).min(layer.regulatory_limit());
            let s = layer
                .transfer(BaseElement::new(SK_ALICE), alice, bob, capped)
                .expect("transferencia");
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
