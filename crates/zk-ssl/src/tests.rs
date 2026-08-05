//! Tests de la capa. Se mantienen juntos porque comparten los
//! ayudantes `new_layer`, `open_and_fund` y `temp_path`.

// Estos tests ejercitan la via ANTIGUA a proposito: sigue siendo la unica
// para `mint` y `mint_pending`, y sus propiedades hay que comprobarlas
// igual. El aviso de obsolescencia se silencia AQUI, no en la definicion,
// para que siga saltando en codigo nuevo.
#![allow(deprecated)]

use super::*;

    use crate::tests_support::*;
    // `derive_public_id_wide` no llega por `use super::*`: lib.rs solo
    // reexporta la estrecha.
    use stark_experiment::circuit_settlement::derive_public_id_wide;

    /// **No hay setup de claves.** Es la propiedad que distingue este
    /// paradigma: arrancar la capa es instantáneo y no genera ningún
    /// secreto que haya que destruir después.
    #[test]
    fn starting_the_layer_requires_no_ceremony() {
        let start = std::time::Instant::now();
        let layer = new_layer();
        let elapsed = start.elapsed();
        println!("Arranque de la capa: {elapsed:?} (sin ceremonia, sin claves)");
        assert_eq!(layer.total_supply(), 0);
        assert_eq!(layer.account_count(), 0);
    }

    /// **Abrir una cuenta NO crea dinero.**
    #[test]
    fn opening_an_account_creates_no_money() {
        let mut layer = new_layer();
        let alice = layer.open_account(BaseElement::new(SK_ALICE));
        assert_eq!(layer.balance_of(alice), Some(0));
        assert_eq!(layer.total_supply(), 0);
    }

    /// La emisión aumenta el suministro exactamente en lo emitido.
    #[test]
    fn minting_increases_supply_exactly() {
        let mut layer = new_layer();
        let alice = layer.open_account(BaseElement::new(SK_ALICE));
        let op = mint_commitment(&layer, alice, 500_000);
        let subida = mint_climb_proof(&layer, alice, 500_000);
        let (pa, ia, pb, ib) = delegated_pair(op, 1, 3);
        println!("Tamano de la subida de EMISION: {} bytes", subida.to_bytes().len());

        assert_eq!(layer.total_supply(), 0, "generar materiales no muta el estado");
        layer
            .apply_mint_delegated(subida, pa, ia, pb, ib, alice, 500_000)
            .expect("aplicar");
        assert_eq!(layer.total_supply(), 500_000);
        assert_eq!(layer.balance_of(alice), Some(500_000));
    }

    /// **EL TEST CLAVE DE LA CAPA**: ciclo completo de transferencia.
    #[test]
    fn full_two_phase_cycle_updates_state() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 50_000);

        let root_before = layer.state_root();

        // ===== FASE 1: EL PAGADOR ENVIA =====
        let estado_alice = state_of(&layer, alice);
        let receptor = layer.public_id_of(bob).expect("cuenta");
        let recibo = layer
            .send(
                BaseElement::new(SK_ALICE),
                alice,
                &estado_alice,
                receptor,
                salt_de(0xC1C10),
                250_000,
            )
            .expect("un envio valido deberia generar prueba");
        println!("Tamano de la prueba de ENVIO: {} bytes", recibo.proof.len());

        // `send` NO toca el estado: solo genera la prueba.
        assert_eq!(layer.state_root(), root_before);

        layer
            .apply_send(&recibo, alice, &estado_alice, 250_000)
            .expect("un envio valido deberia aplicarse");

        assert_eq!(layer.balance_of(alice), Some(750_000), "el dinero salio");
        assert_eq!(
            layer.balance_of(bob),
            Some(50_000),
            "pero el receptor aun no lo tiene: esta en un pendiente"
        );
        assert_eq!(layer.total_pending(), 250_000, "y esta contabilizado");

        // ===== FASE 2: EL RECEPTOR COBRA =====
        let estado_bob = state_of(&layer, bob);
        let cobro = layer
            .claim(BaseElement::new(SK_BOB), bob, &estado_bob, &recibo.notice)
            .expect("el receptor legitimo deberia poder cobrar");
        layer
            .apply_claim(&cobro, bob, &estado_bob, &recibo.notice)
            .expect("el cobro deberia aplicarse");

        assert_eq!(layer.balance_of(alice), Some(750_000));
        assert_eq!(layer.balance_of(bob), Some(300_000), "ahora si");
        assert_eq!(layer.total_pending(), 0, "nada en transito");
        assert_ne!(layer.state_root(), root_before);

        // ⚠️ **Ya no se comprueba la raiz de nullificadores.**
        //
        // `circuit_send` y `circuit_claim` **no los usan**: un reenvio
        // tendria la raiz de cuentas obsoleta y se rechazaria. Es una
        // decision documentada del circuito, no un olvido. Ver
        // `AUDITORIA.md` §13.
    }

    /// **LA INVARIANTE GLOBAL**: la suma de saldos equivale siempre al
    /// suministro emitido, y transferir no lo altera.
    #[test]
    fn total_balances_always_equal_total_supply() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 50_000);

        let sum = |l: &SovereignLayer| -> u64 {
            [alice, bob].iter().map(|i| l.balance_of(*i).unwrap()).sum()
        };
        assert_eq!(sum(&layer), layer.total_supply());

        two_phase_transfer(&mut layer, alice, SK_ALICE, bob, SK_BOB, 250_000, salt_de(0xA1A1))
            .expect("transferencia en dos fases");

        assert_eq!(
            sum(&layer),
            layer.total_supply(),
            "transferir NO debe alterar el suministro total"
        );
        assert_eq!(layer.total_supply(), 1_050_000);
    }

    /// **EL TEST QUE IMPIDE LA REPETICIÓN.**
    ///
    /// Reenviar una liquidación válida duplicaría el dinero. El circuito
    /// no lo impide —la prueba sigue siendo válida—; lo bloquea la capa
    /// al comprobar que parte del estado actual.
    #[test]
    fn replaying_a_send_is_rejected() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 50_000);

        let estado = state_of(&layer, alice);
        let receptor = layer.public_id_of(bob).expect("cuenta");
        let recibo = layer
            .send(
                BaseElement::new(SK_ALICE),
                alice,
                &estado,
                receptor,
                salt_de(0x2E2E),
                250_000,
            )
            .expect("prueba");
        layer
            .apply_send(&recibo, alice, &estado, 250_000)
            .expect("primera");

        // ⚠️ **Aquí no hay nullificador, y el reenvío se bloquea igual.**
        //
        // `circuit_settlement` insertaba una marca pública de gasto.
        // `circuit_send` **no la lleva, por decisión documentada**: un envío
        // cambia el saldo, luego la hoja, luego la raíz, así que el segundo
        // intento parte de una `root_old` que ya no es la actual.
        //
        // El mecanismo cambia; **la propiedad no**.
        assert!(
            matches!(
                layer.apply_send(&recibo, alice, &estado, 250_000),
                Err(LayerError::StaleState)
            ),
            "CRITICO: reaplicar un envio duplicaria el dinero"
        );
        assert_eq!(layer.balance_of(alice), Some(750_000));
        assert_eq!(
            layer.total_pending(),
            250_000,
            "y solo hay UN pendiente, no dos"
        );
    }

    /// Dos transferencias encadenadas: la segunda parte de la raíz que
    /// dejó la primera, con nonce y nullifier distintos.
    #[test]
    fn consecutive_transfers_chain_correctly() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 50_000);

        two_phase_transfer(&mut layer, alice, SK_ALICE, bob, SK_BOB, 100_000, salt_de(0xB2B2))
            .expect("transferencia en dos fases");
        let root_mid = layer.state_root();

        let s2 = layer.send(
                BaseElement::new(SK_ALICE),
                alice,
                &state_of(&layer, alice),
                layer.public_id_of(bob).expect("cuenta"),
                salt_de(0xC4A1),
                200_000,
            )
            .expect("segunda prueba");
        assert_eq!(
            s2.public_inputs.root_old, root_mid,
            "la segunda debe partir de la raiz que dejo la primera"
        );
        let estado = state_of(&layer, alice);
        layer
            .apply_send(&s2, alice, &estado, 200_000)
            .expect("segunda");

        // ⚠️ **Bob todavia no tiene el dinero.**
        //
        // El encadenamiento de raices es lo que este test comprueba, y ocurre
        // en el ENVIO: la segunda prueba parte de la raiz que dejo la
        // primera. Que Bob cobre es otra operacion y otra raiz.
        assert_eq!(layer.balance_of(alice), Some(700_000), "salieron las dos");
        assert_eq!(
            layer.balance_of(bob),
            Some(150_000),
            "solo tiene lo de la PRIMERA, que si se cobro"
        );
        assert_eq!(layer.total_pending(), 200_000, "la segunda esta en transito");
    }

    /// Emitir exactamente hasta el tope sí vale: un límite efectivo menor
    /// que el declarado sería un error de una unidad difícil de detectar.
    #[test]
    fn minting_exactly_to_the_cap_is_allowed() {
        let mut layer = new_layer();
        let alice = layer.open_account(BaseElement::new(SK_ALICE));

        fund_delegated(&mut layer, alice, MAX_SUPPLY);
        assert_eq!(layer.total_supply(), MAX_SUPPLY);
    }

    /// **DESTRUIR LIBERA CAPACIDAD DE EMISIÓN.**
    ///
    /// Tras retirar circulante, el emisor puede volver a emitir hasta el
    /// tope. Es lo que hace del tope un límite de circulante y no un
    /// contador acumulado histórico.
    #[test]
    fn burning_frees_up_minting_capacity() {
        let mut layer = new_layer();
        let alice = layer.open_account(BaseElement::new(SK_ALICE));

        fund_delegated(&mut layer, alice, MAX_SUPPLY);
        let op = mint_commitment(&layer, alice, 1);
        let subida = mint_climb_proof(&layer, alice, 1);
        let (pa, ia, pb, ib) = delegated_pair(op, 1, 3);
        assert!(matches!(
            layer.apply_mint_delegated(subida, pa, ia, pb, ib, alice, 1),
            Err(LayerError::SupplyCapExceeded { .. })
        ));

        let b = layer
            .burn(BaseElement::new(SK_ALICE), alice, &state_of(&layer, alice), 1_000_000)
            .expect("destruccion");
        let estado_alice = state_of(&layer, alice);
        layer.apply_burn(&b, alice, &estado_alice).expect("aplicar");

        // Ahora vuelve a haber margen — y se comprueba por la via real.
        let op = mint_commitment(&layer, alice, 500_000);
        let subida = mint_climb_proof(&layer, alice, 500_000);
        let (pa, ia, pb, ib) = delegated_pair(op, 1, 3);
        let r2 = layer.apply_mint_delegated(subida, pa, ia, pb, ib, alice, 500_000);
        assert!(
            r2.is_ok(),
            "destruir deberia liberar capacidad de emision: {r2:?}"
        );
    }

    /// Cambiar el tope sobre un ledger existente falla.
    #[test]
    fn changing_the_cap_on_an_existing_ledger_fails() {
        let path = temp_path("cap");
        {
            let mut layer =
                open_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS)
                    .expect("abrir");
            open_and_fund(&mut layer, SK_ALICE, 1000);
        }
        let r = open_retry(
                &path, custodian_root(), governance_root(), LIMIT, u64::MAX >> 2, MAX_ACCOUNTS);
        assert!(
            matches!(
                r,
                Err(LayerError::Store(StoreError::ParameterMismatch { .. }))
            ),
            "CRITICO: no debe poder elevarse el tope de un ledger en curso"
        );
        let _ = std::fs::remove_dir_all(&path);
    }

    /// **EL TOPE DE CUENTAS ACOTA LA DENEGACIÓN DE SERVICIO.**
    ///
    /// `open_account` no exige autorización de ningún tipo, así que sin
    /// tope cualquiera podría crear cuentas hasta agotar la memoria del
    /// nodo.
    ///
    /// ⚠️ Esto **acota el daño, no impide el abuso**: un atacante puede
    /// agotar el cupo y dejar sin sitio a usuarios legítimos.
    #[test]
    fn opening_accounts_is_bounded() {
        // Capa con un tope minusculo para que el test sea rapido.
        let mut layer = SovereignLayer::new(
            custodian_root(),
            governance_root(),
            LIMIT,
            MAX_SUPPLY,
            3,
        );
        for i in 0..3 {
            layer
                .open_account_checked(BaseElement::new(100 + i))
                .unwrap_or_else(|e| panic!("la cuenta {i} deberia caber: {e}"));
        }
        let r = layer.open_account_checked(BaseElement::new(999));
        assert!(
            matches!(r, Err(LayerError::AccountLimitReached { limit: 3 })),
            "CRITICO: sin tope, crear cuentas agotaria la memoria del nodo. \
             Resultado: {r:?}"
        );
        assert_eq!(layer.account_count(), 3);
    }

    /// El tope es inmutable: cambiarlo sobre un ledger existente falla.
    #[test]
    fn changing_the_account_limit_on_an_existing_ledger_fails() {
        let path = temp_path("acctlimit");
        {
            let mut layer = open_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
            )
            .expect("abrir");
            layer.open_account(BaseElement::new(SK_ALICE));
        }
        let r = open_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, 999_999,
        );
        assert!(matches!(
            r,
            Err(LayerError::Store(StoreError::ParameterMismatch { .. }))
        ));
        let _ = std::fs::remove_dir_all(&path);
    }

    // -----------------------------------------------------------------
    // Cifrado en reposo
    // -----------------------------------------------------------------

    /// **EL TEST QUE JUSTIFICA EL CIFRADO.**
    ///
    /// Con la contraseña correcta el ledger se recupera; con otra, no.
    #[test]
    fn an_encrypted_ledger_needs_the_right_passphrase() {
        let path = temp_path("encrypted");
        let good = crypto::LedgerKey::from_passphrase("la correcta");

        let alice = {
            let mut layer = open_encrypted_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
                Some(good.clone()),
            )
            .expect("abrir cifrado");
            open_and_fund(&mut layer, SK_ALICE, 1_000_000)
        };

        // Con la contrasena correcta: se recupera.
        {
            let layer = open_encrypted_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
                Some(good),
            )
            .expect("recuperar con la contrasena correcta");
            assert_eq!(layer.balance_of(alice), Some(1_000_000));
        }

        // Con OTRA contrasena: falla.
        let bad = crypto::LedgerKey::from_passphrase("la incorrecta");
        let r = open_encrypted_retry(
            &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
            Some(bad),
        );
        let ok = r.is_ok();
        assert!(
            !ok,
            "CRITICO: otra contrasena no debe poder leer el ledger cifrado"
        );

        // Y SIN contrasena tampoco: los datos estan cifrados en disco.
        let r2 = open_encrypted_retry(
            &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
            None,
        );
        assert!(
            r2.is_err(),
            "CRITICO: sin la clave, los datos en disco no deben ser legibles"
        );

        let _ = std::fs::remove_dir_all(&path);
    }

    /// **Los saldos NO están en claro en el disco.**
    ///
    /// Es la propiedad que define la pieza: quien robe el fichero no
    /// puede leerlos.
    #[test]
    fn balances_are_not_readable_on_disk() {
        let path = temp_path("ondisk");
        // Valor distintivo y POR DEBAJO del tope de emision.
        //
        // La primera version usaba 0x1234_5678_9ABC = 20 billones, muy por
        // encima de MAX_SUPPLY: la emision fallaba y el saldo nunca se
        // creaba, asi que el test no comprobaba nada sobre el cifrado.
        // Habria fallado igual con un cifrado perfecto.
        const SALDO: u64 = 0x05A3_B7C9; // 94.615.497
        {
            let mut layer = open_encrypted_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
                Some(crypto::LedgerKey::from_passphrase("clave")),
            )
            .expect("abrir");
            open_and_fund(&mut layer, SK_ALICE, SALDO);
        }

        // Buscar el saldo en claro entre TODOS los ficheros del directorio.
        //
        // Se buscan los 4 bytes significativos, no los 8: cualquier valor
        // por debajo del tope tiene 4 bytes altos a cero, y buscarlos
        // haria el test menos sensible sin ganar especificidad.
        let le = SALDO.to_le_bytes();
        let patron = &le[..4];
        let mut encontrado = false;
        for entry in std::fs::read_dir(&path).expect("leer dir") {
            let p = entry.expect("entrada").path();
            if p.is_file() {
                if let Ok(bytes) = std::fs::read(&p) {
                    if bytes.windows(4).any(|w| w == patron) {
                        encontrado = true;
                    }
                }
            }
        }
        assert!(
            !encontrado,
            "CRITICO: el saldo aparece EN CLARO en el disco pese al cifrado"
        );
        let _ = std::fs::remove_dir_all(&path);
    }

    /// **EL TEST QUE VALIDA AL TEST ANTERIOR.**
    ///
    /// Sin cifrado, el saldo **sí** debe aparecer en el disco. Si no
    /// apareciera, la búsqueda del test anterior no serviría para nada y
    /// pasaría siempre.
    ///
    /// Es la misma disciplina que con los tests negativos de los
    /// circuitos: comprobar que la prueba puede fallar.
    #[test]
    fn without_encryption_the_balance_is_readable() {
        let path = temp_path("plain");
        const SALDO: u64 = 0x05A3_B7C9;
        {
            let mut layer = open_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
            )
            .expect("abrir sin cifrar");
            open_and_fund(&mut layer, SK_ALICE, SALDO);
        }
        let le = SALDO.to_le_bytes();
        let patron = &le[..4];
        let mut encontrado = false;
        for entry in std::fs::read_dir(&path).expect("leer dir") {
            let p = entry.expect("entrada").path();
            if p.is_file() {
                if let Ok(bytes) = std::fs::read(&p) {
                    if bytes.windows(4).any(|w| w == patron) {
                        encontrado = true;
                    }
                }
            }
        }
        assert!(
            encontrado,
            "sin cifrado el saldo DEBE aparecer en disco; si no, la busqueda \
             del test de cifrado no comprueba nada"
        );
        let _ = std::fs::remove_dir_all(&path);
    }

    // -----------------------------------------------------------------
    // Registro de transiciones
    // -----------------------------------------------------------------

    /// **EL REGISTRO ENCADENA TODA LA ACTIVIDAD.**
    ///
    /// Y se verifica desde el génesis: cada raíz antigua es la raíz nueva
    /// de la anterior, sin huecos.
    #[test]
    fn the_log_chains_every_operation() {
        let mut layer = new_layer();
        let genesis = layer.state_root();

        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        two_phase_transfer(&mut layer, alice, SK_ALICE, bob, SK_BOB, 250_000, salt_de(0x106))
            .expect("transferencia en dos fases");
        let b = layer.burn(BaseElement::new(SK_BOB), bob, &state_of(&layer, bob), 1000).expect("destruir");
        let estado_bob = state_of(&layer, bob);
        layer.apply_burn(&b, bob, &estado_bob).expect("aplicar");

        // Dos aperturas + una emision (bob se abre con cero y no emite)
        // + **un envio + un cobro** + una destruccion.
        //
        // Que `open_account` cuente es lo correcto: mueve la raiz de
        // estado, asi que tiene que dejar rastro.
        //
        // **SEIS, no cinco.** Una transferencia en dos fases deja **dos**
        // entradas donde `transfer` dejaba una. No es contabilidad: el
        // registro refleja que **son dos operaciones distintas, en momentos
        // distintos y con actores distintos**. Quien audite la cadena ve
        // cuando salio el dinero y cuando se cobro.
        assert_eq!(layer.transition_log().len(), 6);
        layer
            .transition_log()
            .verify(genesis)
            .expect("el registro debe verificar desde el genesis");
    }

    /// **LA CABEZA COMPROMETE EL HISTORIAL.**
    ///
    /// Dos capas con la misma actividad tienen la misma cabeza; una
    /// operación más la cambia.
    #[test]
    fn the_log_head_commits_to_the_history() {
        let mut a = new_layer();
        let mut b = new_layer();
        let cuenta_a = open_and_fund(&mut a, SK_ALICE, 1_000_000);
        let _cuenta_b = open_and_fund(&mut b, SK_ALICE, 1_000_000);
        assert_eq!(a.log_head(), b.log_head(), "misma actividad, misma cabeza");

        let extra = a.burn(BaseElement::new(SK_ALICE), cuenta_a, &state_of(&a, cuenta_a), 1).expect("destruir");
        let estado_cuenta_a = state_of(&a, cuenta_a);
        a.apply_burn(&extra, cuenta_a, &estado_cuenta_a).expect("aplicar");
        assert_ne!(a.log_head(), b.log_head(), "una operacion mas la cambia");
    }

    /// **EL REGISTRO SOBREVIVE AL REINICIO.**
    ///
    /// Si se perdiera, el operador borraría el historial reiniciando el
    /// nodo — que es justo lo que el registro existe para impedir.
    #[test]
    fn the_log_survives_restart() {
        let path = temp_path("log");
        let cabeza;
        let genesis;
        {
            let mut layer = open_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
            )
            .expect("abrir");
            genesis = layer.state_root();
            let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
            let bob = open_and_fund(&mut layer, SK_BOB, 0);
            two_phase_transfer(&mut layer, alice, SK_ALICE, bob, SK_BOB, 1000, salt_de(0xC3C3))
                .expect("transferencia en dos fases");
            cabeza = layer.log_head();
        }
        {
            let layer = open_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
            )
            .expect("recuperar");
            // Dos aperturas + una emision + **un envio y un cobro**.
            //
            // Cinco, no cuatro: la via en dos fases deja **dos** entradas
            // donde `transfer` dejaba una, y el registro refleja que son dos
            // operaciones distintas, en momentos distintos y con actores
            // distintos.
            assert_eq!(layer.transition_log().len(), 5);
            assert_eq!(
                layer.log_head(),
                cabeza,
                "CRITICO: si el registro no sobrevive al reinicio, el operador \
                 borraria el historial reiniciando"
            );
            layer
                .transition_log()
                .verify(genesis)
                .expect("debe seguir verificando tras el reinicio");
        }
        let _ = std::fs::remove_dir_all(&path);
    }

    // -----------------------------------------------------------------
    // Reenvío de recibos
    // -----------------------------------------------------------------

    // -----------------------------------------------------------------
    // Transferencia en dos fases
    // -----------------------------------------------------------------

    /// **EL CICLO COMPLETO: ENVIAR Y RECLAMAR.**
    ///
    /// Y lo que cierra la fuga está **en la firma**: `send` recibe la
    /// identidad pública del receptor. **No hay parámetro donde pudiera
    /// entrar su saldo.**
    #[test]
    fn the_full_two_phase_cycle() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 50_000);
        let id_bob = derive_public_id(BaseElement::new(SK_BOB));

        // FASE 1: Alice envia sin conocer el saldo de Bob.
        let r = layer
            .send(BaseElement::new(SK_ALICE), alice, &state_of(&layer, alice), id_bob, salt_de(0x5EED), 250_000)
            .expect("enviar");
        let estado_alice = state_of(&layer, alice);
        layer.apply_send(&r, alice, &estado_alice, 250_000).expect("aplicar envio");

        assert_eq!(layer.balance_of(alice), Some(750_000), "Alice debitada");
        assert_eq!(layer.balance_of(bob), Some(50_000), "Bob aun no cobra");

        // FASE 2: Bob reclama con el aviso.
        let cr = layer
            .claim(BaseElement::new(SK_BOB), bob, &state_of(&layer, bob), &r.notice)
            .expect("reclamar");
        let estado_bob = state_of(&layer, bob);
        layer.apply_claim(&cr, bob, &estado_bob, &r.notice).expect("aplicar reclamacion");

        assert_eq!(layer.balance_of(bob), Some(300_000), "Bob cobrado");
        assert_eq!(layer.balance_of(alice), Some(750_000), "Alice sin cambios");
    }

    /// **NADIE MÁS PUEDE RECLAMARLO.**
    ///
    /// Mallory tiene el aviso —pudo interceptarlo— pero no la clave de
    /// Bob. Sin esto, quien viera el mensaje cobraría el pago.
    #[test]
    fn nobody_else_can_claim_a_pending_transfer() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        let mallory = open_and_fund(&mut layer, 0xBADCAFE, 0);
        let id_bob = derive_public_id(BaseElement::new(SK_BOB));

        let r = layer
            .send(BaseElement::new(SK_ALICE), alice, &state_of(&layer, alice), id_bob, salt_de(0x5EED), 250_000)
            .expect("enviar");
        let estado_alice = state_of(&layer, alice);
        layer.apply_send(&r, alice, &estado_alice, 250_000).expect("aplicar");

        // Mallory lo intenta con SU clave y SU cuenta.
        let intento = layer.claim(BaseElement::new(0xBADCAFE), mallory, &state_of(&layer, mallory), &r.notice);
        if let Ok(cr) = intento {
            let estado_mallory = state_of(&layer, mallory);
            assert!(
                layer.apply_claim(&cr, mallory, &estado_mallory, &r.notice).is_err(),
                "CRITICO: quien intercepte el aviso no debe poder cobrarlo"
            );
        }
        assert_eq!(layer.balance_of(mallory), Some(0), "Mallory no cobra");
        let _ = bob;
    }

    /// **NO SE RECLAMA DOS VECES.**
    ///
    /// El pendiente queda consumido. Sin esto, el mismo pago se cobraría
    /// indefinidamente.
    #[test]
    fn a_pending_transfer_cannot_be_claimed_twice() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        let id_bob = derive_public_id(BaseElement::new(SK_BOB));

        let r = layer
            .send(BaseElement::new(SK_ALICE), alice, &state_of(&layer, alice), id_bob, salt_de(0x5EED), 250_000)
            .expect("enviar");
        let estado_alice = state_of(&layer, alice);
        layer.apply_send(&r, alice, &estado_alice, 250_000).expect("aplicar");

        let cr = layer.claim(BaseElement::new(SK_BOB), bob, &state_of(&layer, bob), &r.notice).expect("reclamar");
        let estado_bob = state_of(&layer, bob);
        layer.apply_claim(&cr, bob, &estado_bob, &r.notice).expect("primera");
        assert_eq!(layer.balance_of(bob), Some(250_000));

        let estado_bob = state_of(&layer, bob);
        assert!(
            layer.apply_claim(&cr, bob, &estado_bob, &r.notice).is_err(),
            "CRITICO: reclamar dos veces seria cobrar dos veces"
        );
        assert_eq!(layer.balance_of(bob), Some(250_000), "el saldo no sube");
    }

    /// **UN TITULAR QUE MIENTA SOBRE SU SALDO ES DETECTADO.**
    ///
    /// La vía nueva **no lee el registro**: el saldo lo aporta el titular
    /// y la capa comprueba que produce la hoja que tiene en el árbol.
    ///
    /// Eso es lo que permite que un operador **no necesite conocer los
    /// saldos** — la prioridad 1 del documento de visión. Si esta
    /// comprobación no bastara, el modelo entero se caería.
    #[test]
    fn a_holder_lying_about_their_balance_is_caught() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let id_bob = derive_public_id(BaseElement::new(SK_BOB));

        // Alice declara tener mas de lo que su hoja acredita.
        let mut mentira = state_of(&layer, alice);
        mentira.balance = 99_999_999;

        let r = layer.send(
            BaseElement::new(SK_ALICE),
            alice,
            &mentira,
            id_bob,
            salt_de(0x5EED),
            250_000,
        );
        assert!(
            matches!(r, Err(LayerError::StaleState)),
            "CRITICO: un saldo declarado que no produce la hoja del arbol debe \
             rechazarse: {r:?}"
        );
    }

    /// **Y el que valida al anterior**: con el saldo real, funciona.
    ///
    /// Sin esto, el anterior pasaría aunque `send` fallara siempre.
    #[test]
    fn a_truthful_holder_can_send() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let id_bob = derive_public_id(BaseElement::new(SK_BOB));
        let verdad = state_of(&layer, alice);

        assert!(layer
            .send(BaseElement::new(SK_ALICE), alice, &verdad, id_bob, salt_de(0x5EED), 250_000)
            .is_ok());
    }

    /// **UNA CUENTA CONGELADA NO PUEDE ENVIAR.**
    #[test]
    fn a_frozen_account_cannot_send() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let id_bob = derive_public_id(BaseElement::new(SK_BOB));

        set_frozen_delegated(&mut layer, alice, true);

        let r = layer.send(BaseElement::new(SK_ALICE), alice, &state_of(&layer, alice), id_bob, salt_de(0x5EED), 1000);
        assert!(matches!(r, Err(LayerError::AccountFrozen(_))));
    }

    /// **LOS PENDIENTES SOBREVIVEN AL REINICIO.**
    ///
    /// Si no lo hicieran, un reinicio borraría las transferencias sin
    /// reclamar: el dinero saldría de la cuenta del pagador y **no
    /// llegaría a ninguna parte**.
    #[test]
    fn pending_transfers_survive_restart() {
        let path = temp_path("pending");
        let aviso;
        let (alice, bob);
        {
            let mut layer = open_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
            )
            .expect("abrir");
            alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
            bob = open_and_fund(&mut layer, SK_BOB, 0);
            let id_bob = derive_public_id(BaseElement::new(SK_BOB));
            let r = layer
                .send(BaseElement::new(SK_ALICE), alice, &state_of(&layer, alice), id_bob, salt_de(0x5EED), 250_000)
                .expect("enviar");
            let estado_alice = state_of(&layer, alice);
            layer.apply_send(&r, alice, &estado_alice, 250_000).expect("aplicar");
            aviso = r.notice.clone();
        }

        let mut layer = open_retry(
            &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
        )
        .expect("reabrir");

        // Bob reclama DESPUES del reinicio.
        let cr = layer
            .claim(BaseElement::new(SK_BOB), bob, &state_of(&layer, bob), &aviso)
            .expect("reclamar tras reiniciar");
        let estado_bob = state_of(&layer, bob);
        layer.apply_claim(&cr, bob, &estado_bob, &aviso).expect("aplicar");
        assert_eq!(
            layer.balance_of(bob),
            Some(250_000),
            "CRITICO: si los pendientes no sobrevivieran, el dinero se perderia"
        );
        let _ = std::fs::remove_dir_all(&path);
    }

    // -----------------------------------------------------------------
    // Emisión a un pendiente
    // -----------------------------------------------------------------

    /// **EL CICLO COMPLETO: EMITIR A UN PENDIENTE Y RECLAMARLO.**
    ///
    /// Los custodios crean dinero **sin tocar ninguna cuenta**, así que no
    /// necesitan el saldo de nadie. El destinatario lo reclama después.
    ///
    /// La emisión clásica acredita una cuenta directamente, y para
    /// calcular su hoja nueva **necesita su saldo**.
    #[test]
    fn the_full_mint_to_pending_cycle() {
        let mut layer = new_layer();
        let bob = open_and_fund(&mut layer, SK_BOB, 50_000);
        let id_bob = derive_public_id(BaseElement::new(SK_BOB));
        let suministro = layer.total_supply();

        // Los custodios emiten a un pendiente, por la vía delegada.
        let op = mint_pending_commitment(&layer, id_bob, salt_de(0xA11), 300_000);
        let subida = mint_pending_climb_proof(&layer, id_bob, salt_de(0xA11), 300_000);
        let (pa, ia, pb, ib) = delegated_pair(op, 1, 3);
        let aviso = layer
            .apply_mint_pending_delegated(subida, pa, ia, pb, ib, id_bob, salt_de(0xA11), 300_000)
            .expect("emitir a pendiente");

        assert_eq!(
            layer.total_supply(),
            suministro + 300_000,
            "el suministro sube"
        );
        assert_eq!(layer.balance_of(bob), Some(50_000), "Bob aun no cobra");

        // Bob lo reclama.
        let estado_bob = state_of(&layer, bob);
        let cr = layer
            .claim(BaseElement::new(SK_BOB), bob, &estado_bob, &aviso)
            .expect("reclamar");
        let estado_bob = state_of(&layer, bob);
        layer.apply_claim(&cr, bob, &estado_bob, &aviso).expect("aplicar");

        assert_eq!(layer.balance_of(bob), Some(350_000), "Bob cobrado");
        assert_eq!(layer.total_supply(), suministro + 300_000, "y el suministro no cambia al reclamar");
    }

    /// **EMITIR CONSUME CUPO DE CUSTODIOS.**
    ///
    /// Es una intervención como las demás, y la rotación la cuenta.
    #[test]
    fn minting_to_pending_consumes_custodian_quota() {
        let mut layer = new_layer();
        let id_bob = derive_public_id(BaseElement::new(SK_BOB));
        assert_eq!(layer.custodian_uses(), 0);

        mint_to_pending_delegated(&mut layer, id_bob, salt_de(0xA11), 1000);
        assert_eq!(layer.custodian_uses(), 1);
    }

    // -----------------------------------------------------------------
    // Rotación de privilegios
    // -----------------------------------------------------------------

    /// **CADA INTERVENCIÓN CONSUME CUPO.**
    ///
    /// La rotación se expresa por **uso**, no por tiempo: esta capa no
    /// tiene noción de tiempo. Sin rotación, una clave comprometida sirve
    /// para siempre.
    #[test]
    fn each_custodian_intervention_consumes_quota() {
        let mut layer = new_layer();
        assert_eq!(layer.custodian_uses(), 0);

        let alice = layer.open_account_checked(BaseElement::new(SK_ALICE)).expect("abrir");
        fund_delegated(&mut layer, alice, 100_000);
        assert_eq!(layer.custodian_uses(), 1, "emitir consume una");

        set_frozen_delegated(&mut layer, alice, true);
        assert_eq!(layer.custodian_uses(), 2, "congelar consume otra");
    }

    /// **AGOTADO EL CUPO, LOS CUSTODIOS NO PUEDEN ACTUAR.**
    ///
    /// Es la rotación funcionando: no es un fallo, es la exigencia de
    /// renovar.
    #[test]
    fn an_exhausted_custodian_set_cannot_act() {
        let mut layer = new_layer_with_quota(2);
        let alice = layer.open_account_checked(BaseElement::new(SK_ALICE)).expect("abrir");

        for i in 0..2 {
            fund_delegated(&mut layer, alice, 1000);
            assert_eq!(layer.custodian_uses(), i + 1);
        }

        // La tercera intervencion: materiales validos, cupo agotado. El
        // cupo se consume DESPUES de verificar la autoridad (:290), asi
        // que la autorizacion pasa y el gasto es lo que rebota.
        let op = mint_commitment(&layer, alice, 1000);
        let subida = mint_climb_proof(&layer, alice, 1000);
        let (pa, ia, pb, ib) = delegated_pair(op, 1, 3);
        assert!(
            matches!(
                layer.apply_mint_delegated(subida, pa, ia, pb, ib, alice, 1000),
                Err(LayerError::CustodianSetExhausted { .. })
            ),
            "CRITICO: un conjunto agotado no debe poder seguir actuando"
        );
        assert_eq!(layer.balance_of(alice), Some(2000), "y no se emitio nada mas");
    }

    /// **ROTAR EL CONJUNTO RENUEVA EL CUPO.**
    ///
    /// Es lo que hace útil la rotación: agotarse no bloquea el sistema,
    /// **obliga a renovar**.
    #[test]
    fn rotating_the_custodian_set_renews_the_quota() {
        let mut layer = new_layer_with_quota(1);
        let alice = layer.open_account_checked(BaseElement::new(SK_ALICE)).expect("abrir");

        fund_delegated(&mut layer, alice, 1000);
        assert_eq!(layer.custodian_uses(), 1);

        update_custodians_delegated(&mut layer, new_custodian_root());
        assert_eq!(layer.custodian_uses(), 0, "rotar reinicia el cupo");
    }

    /// **EL CUPO SOBREVIVE AL REINICIO.**
    ///
    /// Si no lo hiciera, **bastaría reiniciar el nodo para seguir usando
    /// un conjunto agotado**: la rotación no serviría de nada.
    ///
    /// Es el mismo razonamiento que con nullificadores, congelaciones y
    /// suministro. Cuarta vez que aparece el patrón.
    #[test]
    fn the_custodian_quota_survives_restart() {
        let path = temp_path("quota");
        {
            let mut layer = open_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
            )
            .expect("abrir");
            let alice = layer.open_account_checked(BaseElement::new(SK_ALICE)).expect("abrir");
            fund_delegated(&mut layer, alice, 100_000);
            assert_eq!(layer.custodian_uses(), 1);
        }
        let layer = open_retry(
            &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
        )
        .expect("reabrir");
        assert_eq!(
            layer.custodian_uses(),
            1,
            "CRITICO: si el cupo se renovara al reiniciar, bastaria reiniciar \
             para seguir usando un conjunto agotado"
        );
        let _ = std::fs::remove_dir_all(&path);
    }

    // -----------------------------------------------------------------
    // Qué sobrevive a un reinicio
    // -----------------------------------------------------------------

    /// **EL CONTADOR DE CUENTAS SOBREVIVE.**
    ///
    /// Si no lo hiciera, abrir una cuenta tras reiniciar devolvería un
    /// índice **ya ocupado**, y la cuenta nueva **sobrescribiría a otra
    /// existente**: su titular perdería su saldo sin que nada fallara.
    ///
    /// De los ocho campos de estado, había cinco con test de reinicio.
    /// Este era uno de los que faltaban.
    #[test]
    fn the_account_counter_survives_restart() {
        let path = temp_path("nextidx");
        let (alice, bob);
        {
            let mut layer = open_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
            )
                .expect("abrir");
            alice = open_and_fund(&mut layer, SK_ALICE, 500_000);
            bob = open_and_fund(&mut layer, SK_BOB, 300_000);
        }
        let mut layer = open_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
            )
            .expect("reabrir");

        let nueva = layer.open_account_checked(BaseElement::new(0xC0FFEE)).expect("abrir");
        assert_ne!(nueva, alice, "CRITICO: la cuenta nueva pisaria a Alice");
        assert_ne!(nueva, bob, "CRITICO: la cuenta nueva pisaria a Bob");
        assert_eq!(layer.balance_of(alice), Some(500_000), "Alice intacta");
        assert_eq!(layer.balance_of(bob), Some(300_000), "Bob intacto");
        let _ = std::fs::remove_dir_all(&path);
    }

    /// **EL SUMINISTRO TOTAL SOBREVIVE.**
    ///
    /// Si se reiniciara a cero, se podría emitir de nuevo hasta el tope:
    /// **reiniciar el nodo sería una forma de crear dinero**.
    #[test]
    fn the_total_supply_survives_restart() {
        let path = temp_path("supply");
        let emitido = 750_000u64;
        let alice = {
            let mut layer = open_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
            )
                .expect("abrir");
            let a = open_and_fund(&mut layer, SK_ALICE, emitido);
            assert_eq!(layer.total_supply(), emitido);
            a
        };
        let mut layer = open_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
            )
            .expect("reabrir");
        assert_eq!(
            layer.total_supply(),
            emitido,
            "CRITICO: si el suministro se reiniciara, reiniciar el nodo \
             permitiria emitir de nuevo hasta el tope"
        );

        // ⚠️ **Y AHORA EL ATAQUE.** El contador es el indicio; el tope es la
        // propiedad, y comprobarla exige intentar pasarse.
        //
        // Que el contador se restaure no basta: podria restaurarse y no
        // usarse en la comprobacion del tope. Ver `AUDITORIA.md` §27. Índice REAL capturado arriba (post-F3).
        let exceso = MAX_SUPPLY - emitido + 1;
        let op = mint_commitment(&layer, alice, exceso);
        let subida = mint_climb_proof(&layer, alice, exceso);
        let (pa, ia, pb, ib) = delegated_pair(op, 1, 3);
        let r = layer.apply_mint_delegated(subida, pa, ia, pb, ib, alice, exceso);
        assert!(
            matches!(r, Err(LayerError::SupplyCapExceeded { .. })),
            "CRITICO: tras reiniciar, el tope debe seguir imponiendose sobre \
             el suministro YA emitido. Salio: {r:?}"
        );

        // Y hasta el tope, si — por la via real.
        fund_delegated(&mut layer, alice, MAX_SUPPLY - emitido);
        assert_eq!(layer.total_supply(), MAX_SUPPLY, "clavado en el tope");
        let _ = std::fs::remove_dir_all(&path);
    }

    /// **LOS NULLIFICADORES SOBREVIVEN.**
    ///
    /// Ya estaba comprobado en su día, pero **no en combinación con un
    /// gasto real**: aquí se transfiere, se reinicia, y se intenta gastar
    /// el mismo nullificador.
    ///
    /// Si no sobrevivieran, **bastaría reiniciar para gastar dos veces**.
    #[test]
    fn a_restart_does_not_revive_an_applied_receipt() {
        let path = temp_path("nulls");
        let (alice, bob, raiz_cuentas);
        let recibo;
        let estado_usado;
        {
            let mut layer = open_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
            )
                .expect("abrir");
            alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
            bob = open_and_fund(&mut layer, SK_BOB, 0);
            let estado = state_of(&layer, alice);
            let receptor = layer.public_id_of(bob).expect("cuenta");
            recibo = layer
                .send(
                    BaseElement::new(SK_ALICE),
                    alice,
                    &estado,
                    receptor,
                    salt_de(0x5EE),
                    250_000,
                )
                .expect("enviar");
            layer.apply_send(&recibo, alice, &estado, 250_000).expect("aplicar");
            estado_usado = estado;
            raiz_cuentas = layer.state_root();
        }
        let mut layer = open_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
            )
            .expect("reabrir");

        // ⚠️ **Lo que sobrevive ya no es el árbol de nullificadores.**
        //
        // `send`/`claim` no los usan: el reenvío se bloquea porque la
        // `root_old` del recibo deja de ser la raíz actual. Así que lo que
        // hay que comprobar tras reiniciar es **que esa raíz se restaure**
        // — si volviera a la anterior, el recibo antiguo valdría otra vez.
        assert_eq!(
            layer.state_root(),
            raiz_cuentas,
            "CRITICO: si la raiz de cuentas no sobreviviera, bastaria \
             reiniciar para reenviar el mismo recibo"
        );

        // Y la propiedad, comprobada de verdad: el recibo antiguo no cuela.
        assert!(
            matches!(
                layer.apply_send(&recibo, alice, &estado_usado, 250_000),
                Err(LayerError::StaleState)
            ),
            "CRITICO: reiniciar no debe revivir un recibo ya aplicado"
        );

        assert_eq!(layer.balance_of(alice), Some(750_000));
        assert_eq!(
            layer.balance_of(bob),
            Some(0),
            "bob no ha cobrado: el dinero sigue en un pendiente"
        );
        assert_eq!(layer.total_pending(), 250_000, "y el pendiente sobrevivio");
        let _ = std::fs::remove_dir_all(&path);
    }

    /// **EL CONJUNTO DE GOBERNANZA SOBREVIVE.**
    ///
    /// Es inmutable por diseño, pero si al reabrir se restaurara otro,
    /// **quien controlara ese otro conjunto podría cambiar los
    /// custodios**.
    #[test]
    // ⚠️ **Se salta en depuracion por el negativo que lleva dentro.**
    //
    // El nombre suena a camino legitimo, pero la propiedad esta en el
    // impostor: claves de custodio con caminos de gobernanza. Esa traza no
    // cumple `main_trace(16, 39)` y en depuracion panica al generar.
    //
    // Mismo caso que `a_custodian_cannot_change_the_custodian_set`, y
    // **quedo escondido** hasta que se clasificaron los ochenta fallos por
    // clase de panico en vez de contarlos (§78).
    #[cfg_attr(
        debug_assertions,
        ignore = "escenario de rechazo: la traza es invalida a proposito y en depuracion winterfell lo caza al generar (§78)"
    )]
    fn the_governance_set_survives_restart() {
        let path = temp_path("gov");
        {
            let _ = open_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
            ).expect("abrir");
        }
        let mut layer = open_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
            )
            .expect("reabrir");
        // El conjunto legitimo, tras el reinicio, PUEDE actuar — y actua.
        update_custodians_delegated(&mut layer, new_custodian_root());
        assert_eq!(layer.custodian_set_root(), new_custodian_root());

        // ⚠️ **Y el negativo, que es la propiedad.**
        //
        // Que el conjunto legitimo siga funcionando comprueba que el estado
        // se restauro. Lo que hay que comprobar es que **el ilegitimo NO
        // funcione**: si al reiniciar la raiz de gobernanza volviera a un
        // valor por defecto o quedara vacia, cualquiera podria cambiar el
        // conjunto de custodios.
        //
        // Es el mismo modo que el cupo de §28: un valor que se restaura y
        // otro que no.
        // El impostor: custodios firmando en SU dominio contra la
        // gobernanza — el par llega con el dominio equivocado y rebota
        // al aplicar, que es donde la autoridad se comprueba.
        let claves = custodian_keys();
        let otra = governance_commitment(&layer, custodian_root());
        let (pa, ia, pb, ib) = custodian_pair_with(&claves, otra, 1, 3);
        let r = layer.apply_governance_delegated(pa, ia, pb, ib, custodian_root());
        assert!(
            r.is_err(),
            "CRITICO: reiniciar no debe permitir que un no-gobernador cambie \
             el conjunto de custodios: {r:?}"
        );
        assert_eq!(
            layer.custodian_set_root(),
            new_custodian_root(),
            "solo el cambio legitimo se aplico; el del impostor rebota"
        );
        assert_eq!(layer.governance_change_count(), 1);
        let _ = std::fs::remove_dir_all(&path);
    }

    // -----------------------------------------------------------------
    // Poderes del operador que no estaban declarados
    // -----------------------------------------------------------------

    /// **EL OPERADOR PUEDE DESVIAR UN PAGO SI NO COMPRUEBAS EL DESTINO.**
    ///
    /// Encontrado preguntando *"¿qué puede el operador que no esté
    /// declarado?"*. Lo declarado era: ve saldos, ordena y censura.
    ///
    /// El índice de una cuenta ajena **viene de fuera del sistema**, y lo
    /// natural es preguntárselo a la capa. Si la capa devuelve el índice
    /// equivocado, **el pago va a otra cuenta y la prueba es válida**: las
    /// entradas públicas de la liquidación no dicen quién recibe, solo
    /// raíces, nullificador y límites.
    ///
    /// Se cierra comparando la identidad del destinatario con la que
    /// esperas, obtenida **por otro canal** — el propio destinatario.
    #[test]
    fn materials_for_the_wrong_recipient_are_detectable() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        // La identidad esperada viene POR OTRO CANAL -del propio Bob-,
        // que es justo lo que el test comprueba: el indice no se usa.
        let _bob = open_and_fund(&mut layer, SK_BOB, 0);
        let mallory = open_and_fund(&mut layer, 0xBADCAFE, 0);

        let id_bob = derive_public_id(BaseElement::new(SK_BOB));

        // Alice cree pagar a Bob, pero la capa le da el indice de Mallory.
        let id_mallory = layer.public_id_of(mallory).expect("cuenta");
        let m = layer
            .send_materials(alice, id_mallory, 1000, salt_de(0xDE51))
            .expect("materiales");

        assert!(
            m.check_recipient(id_bob).is_err(),
            "CRITICO: comprobar el destinatario debe detectar el desvio"
        );
    }

    /// **Y con el destinatario correcto, la comprobación pasa.**
    ///
    /// Sin esto, el test anterior pasaría aunque `check_recipient`
    /// rechazara siempre.
    #[test]
    fn materials_for_the_right_recipient_check_out() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        // Igual que arriba: la identidad viene por otro canal.
        let _bob = open_and_fund(&mut layer, SK_BOB, 0);

        let id_bob = derive_public_id(BaseElement::new(SK_BOB));
        let m = layer
            .send_materials(alice, id_bob, 1000, salt_de(0xDE52))
            .expect("materiales");

        assert!(
            m.check_recipient(id_bob).is_ok(),
            "el destinatario correcto debe pasar la comprobacion"
        );
    }

    // -----------------------------------------------------------------
    // Combinaciones de operaciones
    // -----------------------------------------------------------------

    /// **UNA CUENTA CONGELADA NO PUEDE DESTRUIR SU DINERO.**
    ///
    /// Una versión anterior de este test documentaba lo contrario: la
    /// liquidación comprobaba la congelación y la destrucción no, así que
    /// **un titular bajo investigación podía vaciar su cuenta a cero**.
    ///
    /// Aquel test terminaba diciendo: *"si se decide que la congelación
    /// debe bloquear también la destrucción, este test falla y señala
    /// dónde"*. Se decidió, y este es el resultado.
    ///
    /// El razonamiento: congelar existe para que una cuenta bajo
    /// investigación **no mueva fondos**. Destruirlos los mueve — los saca
    /// del sistema. Que sea público e irreversible no los devuelve.
    #[test]
    fn a_frozen_account_cannot_burn() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);

        set_frozen_delegated(&mut layer, alice, true);
        assert!(layer.is_frozen(alice));

        // Transferir NO puede.
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        let estado = state_of(&layer, alice);
        let receptor = layer.public_id_of(bob).expect("cuenta");
        assert!(layer
            .send(
                BaseElement::new(SK_ALICE),
                alice,
                &estado,
                receptor,
                salt_de(0xB0B0),
                1000
            )
            .is_err());

        // Y destruir TAMPOCO.
        let b = layer.burn(BaseElement::new(SK_ALICE), alice, &state_of(&layer, alice), 1_000_000);
        assert!(
            matches!(b, Err(LayerError::AccountFrozen(_))),
            "CRITICO: una cuenta congelada no debe poder destruir su dinero: {b:?}"
        );
        assert_eq!(
            layer.balance_of(alice),
            Some(1_000_000),
            "el saldo investigado sigue intacto"
        );
    }

    /// **Y el que valida al anterior**: descongelada, sí puede.
    ///
    /// Sin esto, el test anterior pasaría aunque `burn` fallara siempre
    /// por cualquier otra razón.
    #[test]
    fn an_unfrozen_account_can_burn_again() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);

        set_frozen_delegated(&mut layer, alice, true);
        assert!(layer.burn(BaseElement::new(SK_ALICE), alice, &state_of(&layer, alice), 1000).is_err());

        set_frozen_delegated(&mut layer, alice, false);

        let b = layer
            .burn(BaseElement::new(SK_ALICE), alice, &state_of(&layer, alice), 1000)
            .expect("descongelada deberia poder destruir");
        let estado_alice = state_of(&layer, alice);
        layer.apply_burn(&b, alice, &estado_alice).expect("aplicar");
        assert_eq!(layer.balance_of(alice), Some(999_000));
    }

    /// **LA RECUPERACIÓN NO LEVANTA LA CONGELACIÓN.**
    ///
    /// Si la levantara, bastaría con perder la clave —o decir que se
    /// perdió— para escapar de una investigación.
    ///
    /// Se sostiene porque el árbol de congelados se indexa por **posición
    /// de cuenta**, no por identidad. Pero eso era una consecuencia del
    /// diseño, no una decisión probada.
    #[test]
    fn recovery_does_not_lift_a_freeze() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);

        set_frozen_delegated(&mut layer, alice, true);

        // Los custodios recuperan la cuenta a una identidad nueva.
        let nueva = derive_public_id(BaseElement::new(0xC0FFEE));
        recover_delegated(&mut layer, alice, nueva);

        assert!(
            layer.is_frozen(alice),
            "CRITICO: recuperar una cuenta no debe levantar su congelacion, \
             o bastaria con decir que se perdio la clave para escapar"
        );
    }

    // -----------------------------------------------------------------
    // Casos límite
    // -----------------------------------------------------------------

    /// **TRANSFERIR A LA PROPIA CUENTA.**
    ///
    /// El caso que más me preocupaba y **no estaba probado**.
    ///
    /// La capa lee los dos registros al empezar. Si son el mismo, ambos
    /// llevan el saldo original `B`. Entonces calcularía:
    ///
    /// ```text
    /// saldo_emisor_nuevo  = B − X
    /// saldo_receptor_nuevo = B + X      ← desde B, no desde B−X
    /// ```
    ///
    /// Si eso se aceptara, la cuenta **acabaría con B + X**: dinero creado
    /// de la nada.
    ///
    /// El circuito debería detectarlo, porque el receptor parte de una
    /// hoja que el emisor ya cambió y su subida no alcanzaría la raíz
    /// intermedia. **Este test lo comprueba en vez de suponerlo.**
    #[test]
    fn sending_to_yourself_conserves_value() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let antes = layer.balance_of(alice);
        let suministro = layer.total_supply();

        // ⚠️ **En la via nueva esto ya no es una anomalia.**
        //
        // `transfer` actualizaba las dos hojas a la vez, asi que pagarse a
        // uno mismo era una contradiccion: la misma hoja con dos valores.
        //
        // `send` y `claim` tocan **una hoja cada uno, en momentos
        // distintos**. Enviarse a uno mismo es salir y volver: no hay
        // contradiccion que detectar, y el resultado neto es cero.
        let propio = layer.public_id_of(alice).expect("cuenta");
        let estado = state_of(&layer, alice);
        let enviado = layer
            .send(BaseElement::new(SK_ALICE), alice, &estado, propio, salt_de(0x5E1F), 250_000)
            .and_then(|r| {
                layer.apply_send(&r, alice, &estado, 250_000)?;
                Ok(r)
            });

        if let Ok(recibo) = enviado {
            assert_eq!(
                layer.balance_of(alice),
                Some(750_000),
                "el dinero salio, como en cualquier envio"
            );
            let estado2 = state_of(&layer, alice);
            let cobro = layer.claim(BaseElement::new(SK_ALICE), alice, &estado2, &recibo.notice);
            if let Ok(cr) = cobro {
                let _ = layer.apply_claim(&cr, alice, &estado2, &recibo.notice);
            }
        }

        // **La invariante es lo que importa**, no si la operacion se
        // permite: pase lo que pase, ni se crea ni se destruye valor.
        // Post-F3 los índices no son un rango 0..censo: la suma vive en
        // los RECORDS, que son la verdad de quién existe y dónde.
        let total: u64 = layer.records.values().map(|r| r.balance).sum();
        assert_eq!(
            total + layer.total_pending(),
            suministro,
            "CRITICO: enviarse a uno mismo no puede crear ni destruir dinero"
        );
        assert!(
            layer.balance_of(alice) == antes || layer.total_pending() > 0,
            "o vuelve entero, o esta en transito: {:?} / {}",
            layer.balance_of(alice), layer.total_pending()
        );
    }

    /// **TRANSFERIR CERO.**
    ///
    /// No crea ni destruye dinero, pero **consume una posición del árbol
    /// de pendientes**. Si se permitiera sin coste, sería una forma de
    /// agotar posiciones.
    #[test]
    fn transferring_zero() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        let antes_a = layer.balance_of(alice);
        let antes_b = layer.balance_of(bob);

        // Se permita o no, los saldos no pueden moverse.
        let receptor = layer.public_id_of(bob).expect("cuenta");
        let estado = state_of(&layer, alice);
        if let Ok(r) = layer.send(
            BaseElement::new(SK_ALICE), alice, &estado, receptor, salt_de(0x0000), 0,
        ) {
            let _ = layer.apply_send(&r, alice, &estado, 0);
        }
        assert_eq!(layer.balance_of(alice), antes_a, "el emisor no cambia");
        assert_eq!(layer.balance_of(bob), antes_b, "el receptor tampoco");
    }

    /// **TRANSFERIR TODO EL SALDO.**
    ///
    /// La cuenta queda a cero. Es la frontera del rango: un error de una
    /// unidad lo rechazaría.
    #[test]
    fn transferring_the_entire_balance() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 400_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);

        two_phase_transfer(&mut layer, alice, SK_ALICE, bob, SK_BOB, 400_000, salt_de(0xD4D4))
            .expect("transferencia en dos fases");

        assert_eq!(layer.balance_of(alice), Some(0), "la cuenta queda a cero");
        assert_eq!(layer.balance_of(bob), Some(400_000));
    }

    /// **Y desde una cuenta a cero no se puede sacar nada.**
    #[test]
    fn spending_from_an_empty_account_is_rejected() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 0);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        let receptor = layer.public_id_of(bob).expect("cuenta");
        let estado = state_of(&layer, alice);
        assert!(
            layer
                .send(BaseElement::new(SK_ALICE), alice, &estado, receptor, salt_de(0x0E11), 1)
                .is_err(),
            "no se puede enviar lo que no se tiene"
        );
    }

    /// **TRANSFERIR A UNA CUENTA QUE NO EXISTE.**
    #[test]
    fn sending_to_a_nonexistent_recipient_loses_the_money() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        // ⚠️ **LA VIA NUEVA NO LO IMPIDE, Y ESO ES EL HALLAZGO.**
        //
        // `transfer` recibia un INDICE y devolvia `AccountNotFound`. `send`
        // recibe un IDENTIFICADOR PUBLICO —un hash— y **no comprueba que
        // alguien lo tenga**: no puede, sin revelar quien esta en el arbol.
        //
        // El envio funciona, el dinero sale, y queda en un pendiente que
        // **nadie puede cobrar jamas**. Un digito mal en el identificador
        // pierde el pago sin ningun aviso.
        let inexistente = derive_public_id(BaseElement::new(0xDEADBEEF));
        let estado = state_of(&layer, alice);
        let recibo = layer
            .send(BaseElement::new(SK_ALICE), alice, &estado, inexistente, salt_de(0x404), 1000)
            .expect("la via nueva NO puede detectarlo");
        layer
            .apply_send(&recibo, alice, &estado, 1000)
            .expect("y lo aplica");

        assert_eq!(layer.balance_of(alice), Some(999_000), "el dinero salio");
        assert_eq!(
            layer.total_pending(),
            1000,
            "y esta en un pendiente que nadie reclamara"
        );

        // ⚠️ **No hay devolucion.** El importe queda fuera de circulacion sin
        // dejar de contar en el suministro: la invariante global se cumple y
        // el dinero es inalcanzable. Ver `AUDITORIA.md` §30.
        //
        // **Esto no es un defecto de implementacion**: comprobar la
        // existencia del receptor exigiria que la capa revelara quien tiene
        // cuenta, que es justo lo que el diseno evita. Es un **coste del
        // modelo**, y hasta ahora no estaba declarado.
    }

    // -----------------------------------------------------------------
    // Congelación de cuentas
    // -----------------------------------------------------------------

    /// **NADA SE FILTRA ANTES DE COMPROBAR LA AUTORIDAD.**
    ///
    /// Cualquier comprobación anterior a la autorización **revela su
    /// resultado a quien no es el titular**.
    ///
    /// Con la congelación comprobada antes —como estuvo—, un cliente de la
    /// API podría sondear qué cuentas están congeladas, es decir **quién
    /// está bajo investigación**, sin ser dueño de ninguna.
    #[test]
    fn nothing_leaks_before_authority_is_checked() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);

        set_frozen_delegated(&mut layer, alice, true);

        // Un intruso, sin la clave de Alice, intenta transferir desde su
        // cuenta. Debe recibir "no eres el titular", NO "esta congelada".
        // ⚠️ **La via nueva pide el estado del pagador**, que un intruso no
        // tendria. El test se lo da: lo que se comprueba es **el orden de las
        // comprobaciones**, no que el intruso pudiera llegar hasta aqui.
        let estado = state_of(&layer, alice);
        let receptor = layer.public_id_of(bob).expect("cuenta");
        let r = layer.send(
            BaseElement::new(0x1337),
            alice,
            &estado,
            receptor,
            salt_de(0x1337),
            1000,
        );
        assert!(
            matches!(r, Err(LayerError::NotTheAccountHolder)),
            "CRITICO: un intruso no debe poder deducir que la cuenta esta \
             congelada. Resultado: {r:?}"
        );
    }

    /// **Y el titular SÍ ve el motivo real.**
    ///
    /// Sin esto, el test anterior pasaría aunque `transfer` devolviera
    /// siempre el mismo error y la congelación no se comprobara.
    #[test]
    fn the_holder_does_see_the_freeze() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        set_frozen_delegated(&mut layer, alice, true);

        let estado = state_of(&layer, alice);
        let receptor = layer.public_id_of(bob).expect("cuenta");
        let r = layer.send(
            BaseElement::new(SK_ALICE),
            alice,
            &estado,
            receptor,
            salt_de(0xF2EE),
            1000,
        );
        assert!(
            matches!(r, Err(LayerError::AccountFrozen(_))),
            "el titular debe saber que su cuenta esta congelada: {r:?}"
        );
    }

    /// **Ni el saldo se filtra antes de la autoridad.**
    ///
    /// `InsufficientBalance` lleva el saldo disponible. Si se comprobara
    /// antes que la clave, cualquiera podría **sondear saldos ajenos**
    /// pidiendo transferencias imposibles.
    ///
    /// `burn` y `audit` ya lo tenían en el orden correcto; este test lo
    /// fija para los tres.
    #[test]
    fn the_balance_does_not_leak_before_authority() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);

        // Pide mas de lo que Alice tiene, sin su clave.
        let estado = state_of(&layer, alice);
        let receptor = layer.public_id_of(bob).expect("cuenta");
        let r = layer.send(
            BaseElement::new(0x1337),
            alice,
            &estado,
            receptor,
            salt_de(0xBA1A),
            99_999_999,
        );
        assert!(
            matches!(r, Err(LayerError::NotTheAccountHolder)),
            "CRITICO: el error no debe revelar el saldo a quien no es titular"
        );

        // Y con burn, que tambien lo lleva.
        let r = layer.burn(BaseElement::new(0x1337), alice, &state_of(&layer, alice), 99_999_999);
        assert!(matches!(r, Err(LayerError::NotTheAccountHolder)));
    }

    /// **EL TEST QUE JUSTIFICA TODA LA PIEZA.**
    ///
    /// Una cuenta congelada no puede gastar. Y no lo impide la capa: lo
    /// impide el circuito, que acredita que el emisor no está en el árbol
    /// de congelados.
    #[test]
    fn a_frozen_account_cannot_transfer() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);

        // Antes de congelar: puede gastar.
        two_phase_transfer(&mut layer, alice, SK_ALICE, bob, SK_BOB, 1000, salt_de(0xE5E5))
            .expect("transferencia en dos fases");

        // Dos custodios la congelan.
        set_frozen_delegated(&mut layer, alice, true);
        assert!(layer.is_frozen(alice));

        // Ahora NO puede.
        let estado = state_of(&layer, alice);
        let receptor = layer.public_id_of(bob).expect("cuenta");
        let r = layer.send(
            BaseElement::new(SK_ALICE),
            alice,
            &estado,
            receptor,
            salt_de(0xF3EE),
            1000,
        );
        assert!(
            matches!(r, Err(LayerError::AccountFrozen(_))),
            "CRITICO: una cuenta congelada NO debe poder gastar. Resultado: {r:?}"
        );
        assert_eq!(layer.balance_of(alice), Some(999_000), "sin cambios");
    }

    /// **Y SIGUE PUDIENDO RECIBIR.**
    ///
    /// Es deliberado: impedirlo dejaría fondos en el limbo y rompería
    /// pagos legítimos hacia una cuenta bajo investigación.
    #[test]
    fn a_frozen_account_receives_into_limbo() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);

        set_frozen_delegated(&mut layer, bob, true);

        // ===== EL DINERO SALE, Y LLEGA A UN PENDIENTE =====
        let estado = state_of(&layer, alice);
        let receptor = layer.public_id_of(bob).expect("cuenta");
        let recibo = layer
            .send(BaseElement::new(SK_ALICE), alice, &estado, receptor, salt_de(0xF6F6), 5000)
            .expect("enviar a una cuenta congelada SI se puede");
        layer
            .apply_send(&recibo, alice, &estado, 5000)
            .expect("y aplicarlo tambien");

        assert_eq!(layer.balance_of(alice), Some(995_000), "el dinero salio");
        assert_eq!(layer.total_pending(), 5000, "y esta en un pendiente");

        // ===== PERO NO PUEDE COBRARLO =====
        let estado_bob = state_of(&layer, bob);
        let r = layer.claim(BaseElement::new(SK_BOB), bob, &estado_bob, &recibo.notice);
        assert!(
            matches!(r, Err(LayerError::AccountFrozen(_))),
            "una cuenta congelada no puede cobrar: {:?}",
            r.map(|_| "recibo")
        );
        assert_eq!(layer.balance_of(bob), Some(0), "sigue sin el dinero");

        // ⚠️ **ESTO INVIERTE UNA DECISION DE DISENO DOCUMENTADA.**
        //
        // `freeze.rs` dice: *«Una cuenta congelada no puede gastar, pero si
        // seguir recibiendo. Impedirlo exigiria comprobar tambien al receptor
        // y **dejaria fondos en el limbo**»*.
        //
        // La via en dos fases hace justo eso: el cobro es una accion del
        // receptor, y tanto la capa como `circuit_claim` —que lleva
        // `frozen_root`— la rechazan si esta congelado.
        //
        // **El dinero queda en el limbo que el diseno queria evitar**: salio
        // del pagador, no llego al receptor, y solo se libera si alguien
        // levanta la congelacion. Ver `AUDITORIA.md` §29.
    }

    /// **DESCONGELAR DEVUELVE LA CAPACIDAD DE GASTO.**
    #[test]
    fn unfreezing_restores_the_ability_to_spend() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);

        set_frozen_delegated(&mut layer, alice, true);
        assert!(layer.send(
                BaseElement::new(SK_ALICE),
                alice,
                &state_of(&layer, alice),
                layer.public_id_of(bob).expect("cuenta"),
                salt_de(0xF3EF),
                1000,
            ).is_err());

        set_frozen_delegated(&mut layer, alice, false);
        assert!(!layer.is_frozen(alice));

        two_phase_transfer(&mut layer, alice, SK_ALICE, bob, SK_BOB, 1000, salt_de(0x0707))
            .expect("transferencia en dos fases");
        assert_eq!(layer.balance_of(bob), Some(1000));
    }

    /// Cada congelación y descongelación queda contada.
    #[test]
    fn every_freeze_is_counted() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        assert_eq!(layer.freeze_count(), 0);

        set_frozen_delegated(&mut layer, alice, true);
        assert_eq!(layer.freeze_count(), 1);

        set_frozen_delegated(&mut layer, alice, false);
        assert_eq!(
            layer.freeze_count(),
            2,
            "descongelar tambien es una intervencion y debe contarse"
        );
    }

    /// **LA CONGELACIÓN SOBREVIVE AL REINICIO.**
    ///
    /// Si se perdiera, bastaría reiniciar el nodo para que una cuenta
    /// bajo investigación volviera a poder gastar.
    #[test]
    fn a_freeze_survives_restart() {
        let path = temp_path("freeze");
        let alice;
        {
            let mut layer = open_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
            )
            .expect("abrir");
            alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
            set_frozen_delegated(&mut layer, alice, true);
        }
        {
            let mut layer = open_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
            )
            .expect("recuperar");
            assert!(
                layer.is_frozen(alice),
                "CRITICO: si la congelacion no sobrevive al reinicio, bastaria \
                 reiniciar el nodo para levantarla"
            );
            assert_eq!(layer.freeze_count(), 1);

            // ⚠️ **Y AHORA EL ATAQUE, no solo el indicio.**
            //
            // Que la marca sobreviva es una comprobacion de estado. La
            // propiedad es que **la cuenta no pueda gastar**, y eso exige
            // intentarlo.
            //
            // Son cosas distintas: la marca podria restaurarse y la raiz de
            // congelados no, y entonces `is_frozen` diria que si mientras el
            // circuito acepta la prueba. Ver `AUDITORIA.md` §27.
            let bob = open_and_fund(&mut layer, SK_BOB, 0);
            let estado = state_of(&layer, alice);
            let receptor = layer.public_id_of(bob).expect("cuenta");
            let r = layer.send(
                BaseElement::new(SK_ALICE),
                alice,
                &estado,
                receptor,
                salt_de(0xF00D),
                1000,
            );
            assert!(
                matches!(r, Err(LayerError::AccountFrozen(_))),
                "CRITICO: una cuenta congelada NO debe poder gastar tras \
                 reiniciar. Salio: {:?}",
                r.map(|_| "recibo generado")
            );
        }
        let _ = std::fs::remove_dir_all(&path);
    }

    // -----------------------------------------------------------------
    // Gobernanza
    // -----------------------------------------------------------------

    /// Conjunto de custodios alternativo, para los cambios de gobernanza.
    fn new_custodian_root() -> Digest {
        let keys: Vec<BaseElement> = (0..5)
            .map(|i| BaseElement::new(0xD0_0D_00 + i))
            .collect();
        stark_experiment::circuit_threshold::build_custodian_set(&keys).0
    }

    fn new_custodian_auth() -> ThresholdAuth {
        let keys: Vec<BaseElement> = (0..5)
            .map(|i| BaseElement::new(0xD0_0D_00 + i))
            .collect();
        let (_, paths) = stark_experiment::circuit_threshold::build_custodian_set(&keys);
        ThresholdAuth {
            key_a: keys[1],
            index_a: 1,
            path_a: paths[1].clone(),
            key_b: keys[3],
            index_b: 3,
            path_b: paths[3].clone(),
        }
    }

    /// **EL TEST QUE JUSTIFICA LA PIEZA.**
    ///
    /// Tras el cambio, **los custodios antiguos ya no pueden emitir** y
    /// los nuevos sí. Sin esto, el cambio sería cosmético.
    #[test]
    fn changing_custodians_revokes_the_old_ones() {
        let mut layer = new_layer();
        let alice = layer.open_account(BaseElement::new(SK_ALICE));

        // Los custodios actuales pueden emitir.
        fund_delegated(&mut layer, alice, 1000);

        // La gobernanza cambia el conjunto.
        update_custodians_delegated(&mut layer, new_custodian_root());
        assert_eq!(layer.custodian_set_root(), new_custodian_root());

        // Los ANTIGUOS ya no pueden — ni con materiales frescos.
        let op = mint_commitment(&layer, alice, 1000);
        let subida = mint_climb_proof(&layer, alice, 1000);
        let (pa, ia, pb, ib) = delegated_pair(op, 1, 3); // claves VIEJAS
        let old = layer.apply_mint_delegated(subida, pa, ia, pb, ib, alice, 1000);
        assert!(
            old.is_err(),
            "CRITICO: tras el cambio, los custodios antiguos NO deben poder \
             crear dinero. Si pueden, el cambio es cosmetico: {old:?}"
        );

        // Los NUEVOS sí — con su propio par.
        let nuevas: Vec<BaseElement> = (0..5).map(|i| BaseElement::new(0xD0_0D_00 + i)).collect();
        let op2 = mint_commitment(&layer, alice, 5000);
        let subida2 = mint_climb_proof(&layer, alice, 5000);
        let (pa2, ia2, pb2, ib2) = custodian_pair_with(&nuevas, op2, 1, 3);
        layer
            .apply_mint_delegated(subida2, pa2, ia2, pb2, ib2, alice, 5000)
            .expect("los nuevos custodios deberian poder emitir");
        assert_eq!(layer.balance_of(alice), Some(6000));
    }

    /// **UN CUSTODIO NO PUEDE CAMBIAR EL CONJUNTO DE CUSTODIOS.**
    ///
    /// Es la prueba de que la jerarquía funciona: quien puede emitir y
    /// recuperar cuentas no puede cambiar quién tiene ese poder.
    #[test]
    // ⚠️ **Se salta en depuracion, y no por estar mal.**
    //
    // El impostor lleva claves de custodio por caminos de gobernanza, asi
    // que su carril **no llega a la raiz del conjunto** y la asercion
    // `main_trace(16, 39)` no se cumple. Eso es justo lo que el test
    // comprueba, y en release lo caza el verificador.
    //
    // En depuracion winterfell comprueba las aserciones **al generar** y
    // panica dentro de `update_custodians`. No se puede esperar «el rechazo
    // de cada modo» como en §77.1: **la capa no puede capturar un panico**
    // para devolver un `Err`.
    //
    // Medido el 31-07-2026 (§78). Release SI lo ejecuta.
    #[cfg_attr(
        debug_assertions,
        ignore = "escenario de rechazo: la traza es invalida a proposito y en depuracion winterfell lo caza al generar (§78)"
    )]

    /// Cada cambio queda contado.
    #[test]
    fn every_governance_change_is_counted() {
        let mut layer = new_layer();
        assert_eq!(layer.governance_change_count(), 0);

        update_custodians_delegated(&mut layer, new_custodian_root());
        assert_eq!(layer.governance_change_count(), 1);
    }

    /// **LA ROTACIÓN DEJA HUÉRFANOS LOS MATERIALES DELEGADOS** (§54.2
    /// en vivo: la capa pone SU raíz al verificar). Ni los materiales
    /// generados bajo el conjunto saliente ni sus claves sobreviven al
    /// cambio. Releva en B-3 al clúster vía-recibo de rotación (§166:
    /// pending-receipts, revokes-old, revoked-survives-restart).
    #[test]
    fn rotation_orphans_pregenerated_delegated_materials() {
        let mut layer = new_layer();
        let alice = layer.open_account_checked(BaseElement::new(SK_ALICE)).expect("abrir");

        // Materiales de emisión, generados BAJO el conjunto vigente.
        let op = mint_commitment(&layer, alice, 100_000);
        let subida = mint_climb_proof(&layer, alice, 100_000);
        let (pa, ia, pb, ib) = delegated_pair(op, 1, 3);

        // La gobernanza rota el conjunto, por la delegada.
        update_custodians_delegated(&mut layer, new_custodian_root());

        // Los materiales del conjunto saliente no aplican...
        let r = layer.apply_mint_delegated(subida, pa, ia, pb, ib, alice, 100_000);
        assert!(r.is_err(), "CRITICO: materiales del conjunto revocado: {r:?}");

        // ...y sus claves tampoco autorizan nada nuevo.
        let op2 = mint_commitment(&layer, alice, 100_000);
        let s2 = mint_climb_proof(&layer, alice, 100_000);
        let (pa2, ia2, pb2, ib2) = delegated_pair(op2, 1, 3); // claves VIEJAS
        let r2 = layer.apply_mint_delegated(s2, pa2, ia2, pb2, ib2, alice, 100_000);
        assert!(r2.is_err(), "CRITICO: claves revocadas contra raiz nueva: {r2:?}");
        assert_eq!(layer.total_supply(), 0, "nada se emitio");
    }

    /// **EL CONJUNTO DE CUSTODIOS VIGENTE SOBREVIVE AL REINICIO.**
    ///
    /// Si al reabrir se restaurara el conjunto original, un cambio de
    /// gobernanza se desharía solo — y un custodio revocado volvería a
    /// tener poder con solo reiniciar el nodo.
    #[test]
    fn the_current_custodian_set_survives_restart() {
        let path = temp_path("governance");
        {
            let mut layer =
                open_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS)
                    .expect("abrir");
            update_custodians_delegated(&mut layer, new_custodian_root());
        }
        {
            // Se abre con el conjunto ORIGINAL como argumento, pero el
            // vigente es el nuevo.
            let layer =
                open_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS)
                    .expect("recuperar");
            assert_eq!(
                layer.custodian_set_root(),
                new_custodian_root(),
                "CRITICO: el conjunto vigente debe sobrevivir al reinicio, o un \
                 custodio revocado recuperaria su poder reiniciando el nodo"
            );
            assert_eq!(layer.governance_change_count(), 1);
        }

        // ⚠️ **Y AHORA EL ATAQUE que el mensaje de arriba describe.**
        //
        // Comparar la raiz comprueba que el ESTADO se restaure. La propiedad
        // es que **los custodios revocados no puedan actuar**, y eso exige
        // intentarlo con sus credenciales.
        //
        // La autoridad se comprueba al APLICAR — en las dos eras: generar
        // materiales es del cliente y no consume nada. Ver `AUDITORIA.md` §28.
        {
            let mut layer =
                open_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS)
                    .expect("reabrir para el ataque");
            let alice = layer
                .open_account_checked(BaseElement::new(SK_ALICE))
                .expect("abrir cuenta");

            // Los custodios ANTIGUOS, ya revocados — materiales frescos
            // bajo su conjunto, contra la raiz nueva persistida.
            let op = mint_commitment(&layer, alice, 100_000);
            let subida = mint_climb_proof(&layer, alice, 100_000);
            let (pa, ia, pb, ib) = delegated_pair(op, 1, 3);
            let r = layer.apply_mint_delegated(subida, pa, ia, pb, ib, alice, 100_000);
            assert!(
                r.is_err(),
                "CRITICO: un custodio revocado NO debe recuperar su poder \
                 reiniciando el nodo. Salio: {r:?}"
            );
            assert_eq!(layer.total_supply(), 0, "y no se emitio nada");
        }
        let _ = std::fs::remove_dir_all(&path);
    }

    // -----------------------------------------------------------------
    // Recuperación de cuenta
    // -----------------------------------------------------------------

    /// **EL TEST QUE JUSTIFICA TODA LA PIEZA.**
    ///
    /// Tras la recuperación, **la clave comprometida deja de servir** y
    /// la nueva funciona. Sin esto, la recuperación sería un cambio
    /// cosmético.
    #[test]
    fn recovery_locks_out_the_compromised_key() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);

        // La clave de Alice se ha comprometido. Genera una nueva.
        const SK_ALICE_NEW: u64 = 0xA11CE_2;
        let new_id = derive_public_id(BaseElement::new(SK_ALICE_NEW));

        recover_delegated(&mut layer, alice, new_id);

        // La clave ANTIGUA ya no sirve.
        assert!(
            matches!(
                layer.send(
                BaseElement::new(SK_ALICE),
                alice,
                &state_of(&layer, alice),
                layer.public_id_of(bob).expect("cuenta"),
                salt_de(0xC0FF),
                1000,
            ),
                Err(LayerError::NotTheAccountHolder)
            ),
            "CRITICO: tras recuperar, la clave comprometida NO debe poder gastar"
        );

        // La NUEVA sí.
        two_phase_transfer(&mut layer, alice, SK_ALICE_NEW, bob, SK_BOB, 1000, salt_de(0x1818))
            .expect("transferencia en dos fases");
        assert_eq!(layer.balance_of(bob), Some(1000));
    }

    /// **EL SALDO SE CONSERVA.**
    ///
    /// Una recuperación reasigna el control, no mueve dinero. Sin esta
    /// garantía, dos custodios podrían vaciar cuentas bajo apariencia de
    /// recuperación.
    #[test]
    fn recovery_preserves_the_balance_and_the_supply() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let supply_before = layer.total_supply();

        let new_id = derive_public_id(BaseElement::new(0xA11CE_2));
        recover_delegated(&mut layer, alice, new_id);

        assert_eq!(layer.balance_of(alice), Some(1_000_000), "el saldo no cambia");
        assert_eq!(layer.total_supply(), supply_before, "el suministro tampoco");
    }

    /// **EL CONTADOR HACE CONTABLES LAS INTERVENCIONES.**
    ///
    /// Sin él, los custodios podrían reasignar cuentas en silencio.
    #[test]
    fn every_recovery_increments_the_public_counter() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 1_000_000);
        assert_eq!(layer.recovery_count(), 0);

        recover_delegated(&mut layer, alice, derive_public_id(BaseElement::new(0xA1)));
        assert_eq!(layer.recovery_count(), 1);

        recover_delegated(&mut layer, bob, derive_public_id(BaseElement::new(0xB1)));
        assert_eq!(
            layer.recovery_count(),
            2,
            "cada intervencion de los custodios debe quedar contada"
        );
    }

    /// **El contador sobrevive al reinicio.**
    ///
    /// Si se perdiera, las intervenciones de los custodios dejarían de
    /// ser contables entre arranques — que es justo lo que el contador
    /// existe para evitar.
    #[test]
    fn the_recovery_counter_survives_restart() {
        let path = temp_path("recoveries");
        let materiales;
        let cuenta;
        {
            let mut layer =
                open_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS).expect("abrir");
            let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
            let nueva = derive_public_id(BaseElement::new(0xA1));
            let op = recovery_commitment(&layer, alice, nueva);
            let subida = recovery_climb_proof(&layer, alice, nueva);
            let (pa, ia, pb, ib) = delegated_pair(op, 1, 3);
            layer
                .apply_recovery_delegated(
                    subida.clone(), pa.clone(), ia.clone(), pb.clone(), ib.clone(), alice, nueva,
                )
                .expect("recuperar");
            assert_eq!(layer.recovery_count(), 1);
            materiales = (subida, pa, ia, pb, ib, nueva);
            cuenta = alice;
        }
        {
            let layer =
                open_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS).expect("recuperar");
            assert_eq!(
                layer.recovery_count(),
                1,
                "CRITICO: el contador debe sobrevivir al reinicio, o las \
                 intervenciones dejarian de ser contables"
            );
        }

        // ⚠️ **Y el ataque, que el contador no cubre.**
        //
        // Que el contador se restaure hace las intervenciones CONTABLES. La
        // propiedad distinta es que **una recuperacion ya aplicada no se
        // pueda repetir**: reaplicarla volveria a poner la clave que los
        // custodios eligieron, deshaciendo cualquier cambio posterior del
        // titular legitimo.
        //
        // `replaying_a_delegated_recovery_is_rejected` lo comprueba sin reiniciar. Lo
        // que faltaba es comprobar que **sobrevive al reinicio**.
        {
            let mut layer =
                open_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS)
                .expect("reabrir para el ataque");
            let (subida, pa, ia, pb, ib, nueva) = materiales;
            let r = layer.apply_recovery_delegated(subida, pa, ia, pb, ib, cuenta, nueva);
            assert!(
                r.is_err(),
                "CRITICO: reiniciar no debe permitir reaplicar una \
                 recuperacion ya aplicada. Salio: {r:?}"
            );
            assert_eq!(layer.recovery_count(), 1, "y sigue contando una");
        }
        let _ = std::fs::remove_dir_all(&path);
    }

    // -----------------------------------------------------------------
    // Destrucción de circulante
    // -----------------------------------------------------------------

    /// **EL TEST QUE CIERRA EL CICLO MONETARIO.**
    ///
    /// El dinero puede retirarse, no solo crearse. Y el suministro baja
    /// exactamente en lo destruido.
    #[test]
    fn burning_reduces_balance_and_supply() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        assert_eq!(layer.total_supply(), 1_000_000);

        let receipt = layer
            .burn(BaseElement::new(SK_ALICE), alice, &state_of(&layer, alice), 300_000)
            .expect("destruccion");
        println!(
            "Tamano de la prueba de DESTRUCCION: {} bytes",
            receipt.proof.len()
        );
        assert_eq!(layer.total_supply(), 1_000_000, "burn no debe mutar el estado");

        let estado_alice = state_of(&layer, alice);
        layer.apply_burn(&receipt, alice, &estado_alice).expect("aplicar");
        assert_eq!(layer.balance_of(alice), Some(700_000));
        assert_eq!(layer.total_supply(), 700_000);
    }

    /// **LA INVARIANTE GLOBAL SE MANTIENE TRAS DESTRUIR.**
    ///
    /// La suma de saldos sigue equivaliendo al suministro. Es lo que
    /// distingue una destrucción legítima de dinero que simplemente
    /// desaparece.
    #[test]
    fn invariant_holds_after_burning() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 500_000);

        let sum = |l: &SovereignLayer| -> u64 {
            [alice, bob].iter().map(|i| l.balance_of(*i).unwrap()).sum()
        };
        assert_eq!(sum(&layer), layer.total_supply());

        let r = layer
            .burn(BaseElement::new(SK_ALICE), alice, &state_of(&layer, alice), 400_000)
            .expect("destruccion");
        let estado_alice = state_of(&layer, alice);
        layer.apply_burn(&r, alice, &estado_alice).expect("aplicar");

        assert_eq!(
            sum(&layer),
            layer.total_supply(),
            "la suma de saldos debe seguir equivaliendo al suministro"
        );
        assert_eq!(layer.total_supply(), 1_100_000);
    }

    /// **NADIE PUEDE DESTRUIR EL DINERO DE OTRO.**
    #[test]
    fn third_party_cannot_burn_someone_elses_money() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);

        let r = layer.burn(BaseElement::new(0x1337), alice, &state_of(&layer, alice), 100_000);
        assert!(
            matches!(r, Err(LayerError::NotTheAccountHolder)),
            "CRITICO: sin la clave del titular no debe poder destruirse su saldo. \
             Resultado: {r:?}"
        );
        assert_eq!(layer.balance_of(alice), Some(1_000_000));
    }

    /// No se puede destruir más de lo que hay.
    #[test]
    fn cannot_burn_more_than_the_balance() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 100_000);
        let r = layer.burn(BaseElement::new(SK_ALICE), alice, &state_of(&layer, alice), 500_000);
        assert!(matches!(r, Err(LayerError::InsufficientBalance { .. })));
    }

    /// Reaplicar una destrucción debe rechazarse.
    #[test]
    fn replaying_a_burn_is_rejected() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let r = layer
            .burn(BaseElement::new(SK_ALICE), alice, &state_of(&layer, alice), 200_000)
            .expect("destruccion");

        let estado_alice = state_of(&layer, alice);
        layer.apply_burn(&r, alice, &estado_alice).expect("primera");
        let estado_alice = state_of(&layer, alice);
        assert!(
            matches!(layer.apply_burn(&r, alice, &estado_alice), Err(LayerError::StaleState)),
            "CRITICO: reaplicar una destruccion descuadraria el suministro"
        );
        assert_eq!(layer.total_supply(), 800_000);
    }

    /// **EL CICLO COMPLETO**: emitir, transferir, destruir. La invariante
    /// se mantiene en cada paso.
    #[test]
    fn full_monetary_cycle() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);

        let sum = |l: &SovereignLayer| -> u64 {
            [alice, bob].iter().map(|i| l.balance_of(*i).unwrap()).sum()
        };
        assert_eq!(sum(&layer), layer.total_supply());

        // Transferir: el suministro NO cambia.
        two_phase_transfer(&mut layer, alice, SK_ALICE, bob, SK_BOB, 300_000, salt_de(0xA1A1))
            .expect("transferencia en dos fases");
        assert_eq!(layer.total_supply(), 1_000_000);
        assert_eq!(sum(&layer), layer.total_supply());

        // Destruir: el suministro SI baja.
        let b = layer
            .burn(BaseElement::new(SK_BOB), bob, &state_of(&layer, bob), 100_000)
            .expect("destruccion");
        let estado_bob = state_of(&layer, bob);
        layer.apply_burn(&b, bob, &estado_bob).expect("aplicar");
        assert_eq!(layer.total_supply(), 900_000);
        assert_eq!(sum(&layer), layer.total_supply());

        assert_eq!(layer.balance_of(alice), Some(700_000));
        assert_eq!(layer.balance_of(bob), Some(200_000));
    }

    // -----------------------------------------------------------------
    // Auditoría
    // -----------------------------------------------------------------

    /// **EL TEST QUE HACE AUDITABLE LA CAPA.**
    ///
    /// El titular revela su saldo exacto a un supervisor, que lo verifica
    /// **sin acceso al ledger** — solo con la prueba.
    #[test]
    fn holder_can_disclose_exact_balance_to_a_supervisor() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);

        let disclosure = layer
            .disclose_exact(BaseElement::new(SK_ALICE), alice, &state_of(&layer, alice))
            .expect("revelacion");
        println!(
            "Tamano de la prueba de AUDITORIA: {} bytes",
            disclosure.proof.len()
        );

        // El supervisor verifica SIN la capa.
        assert!(verify_audit(&disclosure).is_ok());
        assert_eq!(disclosure.public_inputs.lower, BaseElement::new(1_000_000));
        assert_eq!(disclosure.public_inputs.upper, BaseElement::new(1_000_000));
        assert_eq!(
            disclosure.public_inputs.root,
            layer.state_root(),
            "la revelacion debe referirse al estado actual"
        );
    }

    /// **SOLVENCIA SIN REVELAR LA CIFRA.**
    ///
    /// Un banco acredita reservas mínimas sin exponer su posición.
    #[test]
    fn holder_can_prove_a_minimum_without_revealing_the_amount() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);

        let d = layer
            .prove_minimum(BaseElement::new(SK_ALICE), alice, &state_of(&layer, alice), 500_000)
            .expect("prueba de minimo");
        assert!(verify_audit(&d).is_ok());
        assert_eq!(d.public_inputs.lower, BaseElement::new(500_000));
        // El saldo real NO aparece: solo el minimo y el techo maximo.
        assert_ne!(d.public_inputs.upper, BaseElement::new(1_000_000));
    }

    /// **LA BANDA**: "estoy entre X e Y", que expone menos que revelar
    /// la cifra exacta y suele bastar a un supervisor.
    #[test]
    fn holder_can_prove_a_band() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);

        let d = layer
            .audit(BaseElement::new(SK_ALICE), alice, &state_of(&layer, alice), 900_000, 1_100_000)
            .expect("banda");
        assert!(verify_audit(&d).is_ok());
    }

    /// **NO SE PUEDE FINGIR SOLVENCIA.**
    #[test]
    fn cannot_prove_a_minimum_that_is_not_met() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 100_000);

        let r = layer.prove_minimum(BaseElement::new(SK_ALICE), alice, &state_of(&layer, alice), 500_000);
        assert!(
            matches!(r, Err(LayerError::BalanceOutsideBand { .. })),
            "CRITICO: no debe poder demostrarse un minimo que no se cumple. \
             Resultado: {r:?}"
        );
    }

    /// **NADIE PUEDE REVELAR POR OTRO.**
    ///
    /// Sin la clave del titular no hay revelación posible, aunque se
    /// conozca todo lo demás.
    #[test]
    fn third_party_cannot_disclose_for_someone_else() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);

        let r = layer.disclose_exact(BaseElement::new(0x1337), alice, &state_of(&layer, alice));
        assert!(
            matches!(r, Err(LayerError::NotTheAccountHolder)),
            "CRITICO: solo el titular puede revelar su saldo. Resultado: {r:?}"
        );
    }

    /// **LA GARANTÍA DE FONDO.**
    ///
    /// Las precondiciones de `audit()` son una comodidad: dan un error
    /// legible y evitan gastar el cómputo. Pero la garantía real está en
    /// el circuito, y este test lo comprueba saltándose la capa.
    ///
    /// Se construye la traza directamente con un saldo fuera de banda. En
    /// release Winterfell no valida al generar, así que produce una
    /// prueba — que **no verifica**.
    #[test]
    fn the_circuit_rejects_out_of_band_even_bypassing_the_layer() {
        use stark_experiment::circuit_audit::{
            build_trace as build_audit, AuditProver, AuditWitness,
        };
        use stark_experiment::circuit_settlement::{native_climb, native_leaf_salted};
        use stark_experiment::merkle::{native_merge, MerklePath, TREE_DEPTH};

        let mut empty = vec![[BaseElement::ZERO; 4]];
        for k in 1..=TREE_DEPTH {
            let prev = empty[k - 1];
            empty.push(native_merge(prev, prev));
        }
        // ⚠️ Ancha de verdad: este test **fabrica el arbol**, no abre
        // cuenta por la capa, asi que no le aplica el limite de §92.14 y
        // puede ejercitar los tres elementos nuevos (§90.3).
        let key = [
            BaseElement::new(SK_ALICE),
            BaseElement::new(0xA0D17),
            BaseElement::new(0x0DDBA11),
            BaseElement::new(0x5EA51DE),
        ];
        let id = derive_public_id_wide(key);
        let nonce = BaseElement::ZERO;
        let mut siblings = Vec::new();
        let mut is_right = Vec::new();
        for level in 0..TREE_DEPTH {
            siblings.push(empty[level]);
            is_right.push(level % 3 == 0);
        }
        let path = MerklePath { siblings, is_right };
        let leaf_salt = [BaseElement::new(0x5A17_0B13); 4];
        let root = native_climb(
            native_leaf_salted(id, BaseElement::new(100_000), nonce, leaf_salt),
            &path,
        );

        let w = AuditWitness {
            spend_key: key,
            balance: 100_000,
            nonce,
            leaf_salt,
            path,
        };
        // Se afirma un minimo de 500.000 con un saldo de 100.000.
        let trace = build_audit(&w, 500_000, MAX_VALUE);
        let prover = AuditProver::new(proof_options());
        let public_inputs = prover.get_pub_inputs(&trace);

        let hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| prover.prove(trace)));
        std::panic::set_hook(hook);

        match r {
            Err(_) | Ok(Err(_)) => { /* rechazado al generar */ }
            Ok(Ok(proof)) => {
                let d = AuditDisclosure {
                    proof: proof.to_bytes(),
                    public_inputs,
                };
                assert!(
                    verify_audit(&d).is_err(),
                    "CRITICO: una revelacion de un minimo que no se cumple NO debe \
                     verificar, aunque se salten las comprobaciones de la capa"
                );
                assert_eq!(d.public_inputs.root, root, "misma raiz");
            }
        }
    }

    // -----------------------------------------------------------------
    // Persistencia
    // -----------------------------------------------------------------

    fn temp_path(name: &str) -> String {
        let mut p = std::env::temp_dir();
        p.push(format!("zkssl_{}_{}", name, std::process::id()));
        let s = p.to_str().unwrap().to_string();
        let _ = std::fs::remove_dir_all(&s);
        s
    }

    /// **EL TEST QUE JUSTIFICA LA PERSISTENCIA.**
    ///
    /// El estado sobrevive al reinicio del nodo: cuentas, saldos,
    /// suministro y pendientes. Sin esto, apagar el proceso
    /// borraría el ledger entero.
    #[test]
    fn ledger_survives_restart() {
        let path = temp_path("restart");
        let (alice, bob);

        {
            let mut layer =
                open_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS).expect("abrir");
            alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
            bob = open_and_fund(&mut layer, SK_BOB, 50_000);
            two_phase_transfer(&mut layer, alice, SK_ALICE, bob, SK_BOB, 250_000, salt_de(0x1ED9E4))
                .expect("transferencia en dos fases");
        } // el nodo se apaga

        {
            let mut layer = open_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS)
                .expect("el ledger deberia recuperarse");
            assert_eq!(layer.balance_of(alice), Some(750_000));
            assert_eq!(layer.balance_of(bob), Some(300_000));
            assert_eq!(layer.total_supply(), 1_050_000);
            assert_eq!(layer.account_count(), 2);
            assert_eq!(layer.total_pending(), 0, "nada quedo en transito");

            // ⚠️ **Y el ledger recuperado debe SEGUIR FUNCIONANDO.**
            //
            // Los saldos comprobados arriba dicen que el estado se leyo. No
            // dicen que se leyera **entero**: si faltara el nonce, la raiz de
            // congelados o el contador de cuentas, los saldos cuadrarian y la
            // siguiente operacion fallaria.
            //
            // Es el modo de §28: un valor que se restaura y otro que no.
            two_phase_transfer(&mut layer, bob, SK_BOB, alice, SK_ALICE, 1000, salt_de(0x5EC0))
                .expect("CRITICO: el ledger recuperado debe poder operar");
            assert_eq!(layer.balance_of(alice), Some(751_000));
            assert_eq!(layer.balance_of(bob), Some(299_000));
            assert_eq!(
                layer.total_supply(),
                1_050_000,
                "y el suministro no cambia: una transferencia no crea dinero"
            );
        }

        let _ = std::fs::remove_dir_all(&path);
    }

    /// **El pendiente cobrado sobrevive al reinicio.**
    ///
    /// Sin esto, reiniciar el nodo permitiría cobrar dos veces: la hoja
    /// consumida del árbol de pendientes volvería a aparecer ocupada.
    #[test]
    fn a_restart_does_not_allow_claiming_twice() {
        let path = temp_path("cobro_doble");
        let aviso;
        let (alice, bob);

        {
            let mut layer = open_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS)
                .expect("abrir");
            alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
            bob = open_and_fund(&mut layer, SK_BOB, 0);

            let estado = state_of(&layer, alice);
            let receptor = layer.public_id_of(bob).expect("cuenta");
            let recibo = layer
                .send(
                    BaseElement::new(SK_ALICE),
                    alice,
                    &estado,
                    receptor,
                    salt_de(0xC0B0),
                    100_000,
                )
                .expect("enviar");
            layer.apply_send(&recibo, alice, &estado, 100_000).expect("aplicar");

            let estado_bob = state_of(&layer, bob);
            let cobro = layer
                .claim(BaseElement::new(SK_BOB), bob, &estado_bob, &recibo.notice)
                .expect("cobrar");
            layer
                .apply_claim(&cobro, bob, &estado_bob, &recibo.notice)
                .expect("aplicar cobro");
            aviso = recibo.notice.clone();
        }

        let mut layer = open_retry(
            &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS)
            .expect("recuperar");

        // **Intenta el ataque COMPLETO, no solo la primera mitad.**
        //
        // ⚠️ Una version anterior comprobaba solo `claim()`, que **unicamente
        // genera la prueba**. Que se genere no significa que se pueda
        // cobrar: el estado cambia en `apply_claim`, y ahi es donde hay que
        // intentar el ataque.
        //
        // El pendiente ya se cobro. Si reiniciar lo devolviera al arbol,
        // Bob podria cobrarlo otra vez y **se crearia dinero**.
        let estado_bob = state_of(&layer, bob);
        let intento = layer.claim(BaseElement::new(SK_BOB), bob, &estado_bob, &aviso);
        let bloqueado = match intento {
            Err(_) => true,
            Ok(recibo) => layer
                .apply_claim(&recibo, bob, &estado_bob, &aviso)
                .is_err(),
        };
        assert!(
            bloqueado,
            "CRITICO: reiniciar no debe permitir cobrar dos veces el mismo \
             pendiente"
        );

        assert_eq!(layer.balance_of(bob), Some(100_000), "cobrado UNA vez");
        assert_eq!(layer.total_pending(), 0, "y nada en transito");

        let _ = std::fs::remove_dir_all(&path);
    }

    /// **EL TEST QUE DETECTA UN LEDGER CORRUPTO.**
    ///
    /// Se manipula la raíz guardada. Al arrancar, la reconstruida no
    /// coincide y el nodo **se niega a operar**.
    ///
    /// Sin esta comprobación, el nodo generaría pruebas válidas de
    /// transiciones sobre un estado que no es el real —
    /// criptográficamente indetectable desde fuera, porque las pruebas
    /// verificarían perfectamente.
    #[test]
    fn corrupted_ledger_is_detected_at_startup() {
        let path = temp_path("corrupt");

        {
            let mut layer =
                open_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS).expect("abrir");
            open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        }

        // Manipulacion: se altera la raiz guardada.
        {
            let db = sled_open_retry(&path);
            let fake: Digest = [BaseElement::new(0xBADBAD); 4];
            db.insert(b"root:state", store::digest_to_bytes(&fake).to_vec())
                .expect("insertar");
            db.flush().expect("flush");
        }

        let r = open_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS);
        assert!(
            matches!(
                r,
                Err(LayerError::Store(StoreError::IntegrityFailure { .. }))
            ),
            "CRITICO: un ledger corrupto debe detectarse ANTES de operar sobre el"
        );

        let _ = std::fs::remove_dir_all(&path);
    }

    /// Cambiar la identidad del emisor sobre un ledger existente falla.
    ///
    /// Silenciarlo permitiría sustituir al banco central sin dejar rastro.
    #[test]
    fn changing_the_governance_set_on_an_existing_ledger_fails() {
        let path = temp_path("issuer");

        {
            let mut layer =
                open_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS).expect("abrir");
            open_and_fund(&mut layer, SK_ALICE, 1000);
        }

        let r = open_retry(
                &path, custodian_root(), [BaseElement::new(0xDEAD); 4], LIMIT, MAX_SUPPLY, MAX_ACCOUNTS);
        assert!(
            matches!(
                r,
                Err(LayerError::Store(StoreError::ParameterMismatch { .. }))
            ),
            "CRITICO: no debe poder sustituirse la autoridad emisora de un \
             ledger en curso"
        );

        let _ = std::fs::remove_dir_all(&path);
    }

    /// Cambiar el límite regulatorio sobre un ledger existente también.
    #[test]
    fn changing_the_limit_on_an_existing_ledger_fails() {
        let path = temp_path("limit");

        {
            let mut layer =
                open_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS).expect("abrir");
            open_and_fund(&mut layer, SK_ALICE, 1000);
        }

        let r = open_retry(
                &path, custodian_root(), governance_root(), u64::MAX, MAX_SUPPLY, MAX_ACCOUNTS);
        assert!(matches!(
            r,
            Err(LayerError::Store(StoreError::ParameterMismatch { .. }))
        ));

        let _ = std::fs::remove_dir_all(&path);
    }

    /// Saldo insuficiente y límite regulatorio se reportan con claridad,
    /// antes de intentar generar la prueba.
    #[test]
    fn invalid_transfers_are_reported_clearly() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 100_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);

        assert!(matches!(
            layer.send(
                BaseElement::new(SK_ALICE),
                alice,
                &state_of(&layer, alice),
                layer.public_id_of(bob).expect("cuenta"),
                salt_de(0x1BAD),
                250_000,
            ),
            Err(LayerError::InsufficientBalance { .. })
        ));

        // ⚠️ **La segunda afirmacion de este test se ha RETIRADO.**
        //
        // Comprobaba que enviar a la cuenta 999 diera `AccountNotFound`.
        // `send` no recibe indices sino identificadores publicos, y **no
        // puede comprobar que alguien los tenga** sin revelar quien esta en
        // el arbol.
        //
        // No es que el error haya cambiado: **la comprobacion ya no es
        // posible**, y lo que ocurre en su lugar —el dinero se pierde— lo
        // fija `sending_to_a_nonexistent_recipient_loses_the_money`. Ver
        // `AUDITORIA.md` §30.
    
}

/// **LA INVARIANTE GLOBAL CON DINERO EN TRÁNSITO.**
///
/// `total_balances_always_equal_total_supply` comprueba que la suma de
/// los saldos iguala al suministro. **Con la vía en dos fases eso deja
/// de ser cierto**: el dinero sale de la cuenta del pagador y espera en
/// un pendiente que **no está en ningún saldo**.
///
/// La invariante correcta es:
///
/// ```text
/// suma de saldos + suma de pendientes == suministro
/// ```
///
/// ⚠️ **Este test se escribió para verlo fallar.** La capa no lleva la
/// suma de los pendientes, así que el descuadre existe y **ningún test
/// lo detectaba**: el que comprobaba la invariante usaba `transfer()`
/// —hoy retirada (§161)—, que abonaba al receptor en el acto.
///
/// Es el mismo modo de fallo que este proyecto documenta en otros
/// sitios: **una propiedad que se cree comprobada porque hay un test
/// con ese nombre**, y el test ejercita otro camino.
#[test]
fn balances_plus_pending_always_equal_total_supply() {
    let mut layer = new_layer();
    let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
    let bob = open_and_fund(&mut layer, SK_BOB, 50_000);

    let suma_saldos = |l: &SovereignLayer| -> u64 {
        [alice, bob].iter().map(|i| l.balance_of(*i).unwrap()).sum()
    };
    assert_eq!(
        suma_saldos(&layer),
        layer.total_supply(),
        "sin pendientes, la invariante clasica se cumple"
    );

    // Envio por la via en dos fases: el dinero sale y queda en transito.
    let estado = state_of(&layer, alice);
    let receptor = layer.public_id_of(bob).expect("cuenta");
    let recibo = layer
        .send(
            BaseElement::new(SK_ALICE),
            alice,
            &estado,
            receptor,
            salt_de(0x7E57),
            250_000,
        )
        .expect("envio");
    layer
        .apply_send(&recibo, alice, &estado, 250_000)
        .expect("aplicar");

    assert_eq!(
        suma_saldos(&layer) + layer.total_pending(),
        layer.total_supply(),
        "CRITICO: el dinero en transito debe contarse. Sin sumar los \
         pendientes, {} + 0 != {}",
        suma_saldos(&layer),
        layer.total_supply()
    );

    // Y al cobrarse, el pendiente desaparece y vuelve a un saldo.
    let estado_bob = state_of(&layer, bob);
    let cobro = layer
        .claim(BaseElement::new(SK_BOB), bob, &estado_bob, &recibo.notice)
        .expect("cobro");
    layer
        .apply_claim(&cobro, bob, &estado_bob, &recibo.notice)
        .expect("aplicar cobro");

    assert_eq!(layer.total_pending(), 0, "ya no hay nada en transito");
    assert_eq!(
        suma_saldos(&layer),
        layer.total_supply(),
        "y la invariante clasica vuelve a cumplirse"
    );
}

/// **EL LÍMITE REGULATORIO LO IMPONE EL SISTEMA, NO QUIEN ENVÍA.**
///
/// Son **dos comprobaciones que se componen**:
///
/// | Dónde | Qué prueba |
/// |---|---|
/// | El circuito | `importe ≤ límite declarado` |
/// | La capa | El límite declarado **es el del sistema** |
///
/// Juntas dan `importe ≤ límite del sistema`, y —a diferencia de una
/// comprobación solo de capa— **un tercero con la prueba puede verificar la
/// primera mitad**.
///
/// La capa antigua tenía `settlement_with_foreign_limit_is_rejected`:
/// manipulaba el límite declarado en el recibo y comprobaba que `apply` lo
/// rechazara. **La vía nueva no tenía equivalente**, y tampoco la
/// comprobación.
///
/// | | Límite regulatorio |
/// |---|---|
/// | `circuit_settlement` + `apply` | Entrada pública **y** comprobación al aplicar |
/// | `circuit_send` + `apply_send` | ⚠️ **Ninguna de las dos** |
///
/// **Al sustituir una vía por otra se perdió la comprobación**, y como la
/// nueva es ahora la única de ISO, el límite quedaba impuesto solo al
/// generar — evitable construyendo la propia traza.
///
/// Se encontró contrastando los tests de `crates/settlement-layer`, la capa
/// superada, con la que se ejecuta. Ver `AUDITORIA.md` §25.
///
/// ⚠️ **Sigue siendo una comprobación de capa.** `circuit_send` no lleva el
/// límite como entrada pública, así que un tercero que solo tenga la prueba
/// **no puede verificar que se respetó**. La vía antigua sí lo permitía.
#[test]
fn a_send_declaring_more_than_the_limit_is_rejected() {
    let mut layer = new_layer();
    let alice = open_and_fund(&mut layer, SK_ALICE, 5_000_000);
    let bob = open_and_fund(&mut layer, SK_BOB, 50_000);

    let estado = state_of(&layer, alice);
    let receptor = layer.public_id_of(bob).expect("cuenta");
    let mut recibo = layer
        .send(
            BaseElement::new(SK_ALICE),
            alice,
            &estado,
            receptor,
            salt_de(0x11017),
            250_000,
        )
        .expect("un envio dentro del limite se genera bien");

    // Quien quisiera esquivar el limite declararia OTRO LIMITE, no otro
    // importe: el circuito prueba `importe <= limite declarado`, asi que
    // subir el importe sin subir el limite no daria una prueba valida.
    recibo.public_inputs.regulatory_limit = BaseElement::new(u64::MAX / 2);

    let r = layer.apply_send(&recibo, alice, &estado, 250_000);
    assert!(
        matches!(r, Err(LayerError::WrongRegulatoryLimit { .. })),
        "CRITICO: el limite declarado en la prueba debe ser EL DEL SISTEMA. \
         El circuito prueba `importe <= limite declarado`; sin esta \
         comprobacion bastaria con declarar uno enorme. Salio: {r:?}"
    );
}

/// **REINICIAR NO DEBE RENOVAR EL CUPO DE CUSTODIOS.**
///
/// `the_custodian_quota_survives_restart` comprueba que el **contador**
/// sobreviva. Pero el cupo son **dos cosas**: el contador y el máximo.
///
/// | | ¿Persiste? |
/// |---|---|
/// | `custodian_uses` — el contador | ✅ `meta:cust_uses` |
/// | `max_custodian_uses` — el máximo | ⚠️ **No**: vuelve al valor por defecto |
///
/// Si alguien restringió el cupo —para limitar a un conjunto de custodios
/// bajo sospecha— **reiniciar el nodo lo renovaría**.
///
/// ⚠️ **Este test se escribió para verlo fallar.** Es el mismo modo que §27:
/// once tests de reinicio comparan un valor, y el que no comparaban era
/// justo el que faltaba.
#[test]
fn a_restart_does_not_renew_an_exhausted_custodian_quota() {
    let path = temp_path("cupo_reinicio");
    let alice;
    {
        let mut layer = open_retry(
            &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
        )
        .expect("abrir");
        // Un solo uso permitido, y se gasta.
        layer.set_max_custodian_uses(1);
        alice = layer
            .open_account_checked(BaseElement::new(SK_ALICE))
            .expect("abrir cuenta");
        fund_delegated(&mut layer, alice, 100_000);

        // ⚠️ **El cupo se consume en el APPLY, no al generar** — en las
        // dos vias (:125 vieja, :290 delegada). Es una decision
        // documentada: materiales generados y NO aplicados no gastan
        // cupo; una version anterior comprobaba la generacion y fallaba
        // por eso.
        let op = mint_commitment(&layer, alice, 1000);
        let subida = mint_climb_proof(&layer, alice, 1000);
        let (pa, ia, pb, ib) = delegated_pair(op, 1, 3);
        let r = layer.apply_mint_delegated(subida, pa, ia, pb, ib, alice, 1000);
        assert!(
            r.is_err(),
            "el cupo esta agotado antes de reiniciar: {r:?}"
        );
    }

    let mut layer = open_retry(
        &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
    )
    .expect("reabrir");

    // **El ataque: reiniciar y APLICAR, que es donde se gasta el cupo.**
    let op = mint_commitment(&layer, alice, 1000);
    let subida = mint_climb_proof(&layer, alice, 1000);
    let (pa, ia, pb, ib) = delegated_pair(op, 1, 3);
    let r = layer.apply_mint_delegated(subida, pa, ia, pb, ib, alice, 1000);
    assert!(
        r.is_err(),
        "CRITICO: reiniciar el nodo NO debe renovar un cupo agotado. \
         Salio: {r:?}"
    );

    let _ = std::fs::remove_dir_all(&path);

}
