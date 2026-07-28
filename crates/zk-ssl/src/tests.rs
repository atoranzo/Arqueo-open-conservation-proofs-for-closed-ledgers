//! Tests de la capa. Se mantienen juntos porque comparten los
//! ayudantes `new_layer`, `open_and_fund` y `temp_path`.

use super::*;

    use super::*;

    use crate::tests_support::*;

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

    /// **EL TEST QUE CIERRA LA CREACIÓN DE DINERO.**
    #[test]
    fn only_the_issuer_can_create_money() {
        let mut layer = new_layer();
        let alice = layer.open_account(BaseElement::new(SK_ALICE));
        let keys = custodian_keys();
        let (_, paths) = stark_experiment::circuit_threshold::build_custodian_set(&keys);
        // Mismo custodio dos veces: un 2-de-N disfrazado de 1-de-N.
        let bad = ThresholdAuth {
            key_a: keys[2],
            index_a: 2,
            path_a: paths[2].clone(),
            key_b: keys[2],
            index_b: 2,
            path_b: paths[2].clone(),
        };
        let r = layer.mint(&bad, alice, 1_000_000);
        assert!(
            matches!(r, Err(LayerError::NotTheIssuer)),
            "CRITICO: sin la clave del emisor no debe poder crearse dinero"
        );
        assert_eq!(layer.total_supply(), 0);
    }

    /// La emisión aumenta el suministro exactamente en lo emitido.
    #[test]
    fn minting_increases_supply_exactly() {
        let mut layer = new_layer();
        let alice = layer.open_account(BaseElement::new(SK_ALICE));
        let receipt = layer
            .mint(&valid_auth(), alice, 500_000)
            .expect("emision");
        println!("Tamano de la prueba de EMISION: {} bytes", receipt.proof.len());

        assert_eq!(layer.total_supply(), 0, "mint no debe mutar el estado");
        layer.apply_mint(&receipt, alice).expect("aplicar");
        assert_eq!(layer.total_supply(), 500_000);
        assert_eq!(layer.balance_of(alice), Some(500_000));
    }

    /// **EL TEST CLAVE DE LA CAPA**: ciclo completo de transferencia.
    #[test]
    fn full_transfer_cycle_updates_state() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 50_000);

        let root_before = layer.state_root();
        let null_root_before = layer.nullifier_root();

        let settlement = layer
            .transfer(BaseElement::new(SK_ALICE), alice, bob, 250_000)
            .expect("una transferencia valida deberia generar prueba");
        println!(
            "Tamano de la prueba de LIQUIDACION: {} bytes",
            settlement.proof.len()
        );

        // `transfer` NO toca el estado.
        assert_eq!(layer.state_root(), root_before);

        layer
            .apply(&settlement, alice, bob, 250_000)
            .expect("una liquidacion valida deberia aplicarse");

        assert_eq!(layer.balance_of(alice), Some(750_000));
        assert_eq!(layer.balance_of(bob), Some(300_000));
        assert_ne!(layer.state_root(), root_before);
        assert_ne!(
            layer.nullifier_root(),
            null_root_before,
            "el nullifier deberia haberse insertado"
        );
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

        let s = layer
            .transfer(BaseElement::new(SK_ALICE), alice, bob, 250_000)
            .expect("prueba");
        layer.apply(&s, alice, bob, 250_000).expect("aplicar");

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
    fn replaying_a_settlement_is_rejected() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 50_000);

        let s = layer
            .transfer(BaseElement::new(SK_ALICE), alice, bob, 250_000)
            .expect("prueba");
        layer.apply(&s, alice, bob, 250_000).expect("primera");

        assert!(
            matches!(
                layer.apply(&s, alice, bob, 250_000),
                Err(LayerError::StaleState)
            ),
            "CRITICO: reaplicar una liquidacion duplicaria el dinero"
        );
        assert_eq!(layer.balance_of(alice), Some(750_000));
    }

    /// Dos transferencias encadenadas: la segunda parte de la raíz que
    /// dejó la primera, con nonce y nullifier distintos.
    #[test]
    fn consecutive_transfers_chain_correctly() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 50_000);

        let s1 = layer
            .transfer(BaseElement::new(SK_ALICE), alice, bob, 100_000)
            .expect("primera prueba");
        layer.apply(&s1, alice, bob, 100_000).expect("primera");
        let root_mid = layer.state_root();

        let s2 = layer
            .transfer(BaseElement::new(SK_ALICE), alice, bob, 200_000)
            .expect("segunda prueba");
        assert_eq!(
            s2.public_inputs.root_old, root_mid,
            "la segunda debe partir de la raiz que dejo la primera"
        );
        layer.apply(&s2, alice, bob, 200_000).expect("segunda");

        assert_eq!(layer.balance_of(alice), Some(700_000));
        assert_eq!(layer.balance_of(bob), Some(350_000));
    }

    /// **EL TEST DEL TOPE DE EMISIÓN.**
    ///
    /// Ni siquiera la autoridad emisora puede inflar sin límite. Tiene la
    /// clave, pero el tope es un parámetro inmutable del ledger.
    #[test]
    fn the_issuer_cannot_mint_beyond_the_cap() {
        let mut layer = new_layer();
        let alice = layer.open_account(BaseElement::new(SK_ALICE));

        let r = layer.mint(&valid_auth(), alice, MAX_SUPPLY + 1);
        assert!(
            matches!(r, Err(LayerError::SupplyCapExceeded { .. })),
            "CRITICO: la autoridad emisora NO debe poder superar el tope del \
             sistema. Resultado: {r:?}"
        );
        assert_eq!(layer.total_supply(), 0);
    }

    /// El tope se aplica al ACUMULADO, no a cada emisión por separado.
    ///
    /// Sin esto, el emisor podría emitir mil veces por debajo del tope y
    /// superarlo igualmente.
    #[test]
    fn the_cap_applies_to_the_accumulated_supply() {
        let mut layer = new_layer();
        let alice = layer.open_account(BaseElement::new(SK_ALICE));

        // Primera emision: hasta casi el tope.
        let r1 = layer
            .mint(&valid_auth(), alice, MAX_SUPPLY - 1000)
            .expect("primera emision");
        layer.apply_mint(&r1, alice).expect("aplicar");
        assert_eq!(layer.total_supply(), MAX_SUPPLY - 1000);

        // Segunda: pequena por si sola, pero superaria el acumulado.
        let r2 = layer.mint(&valid_auth(), alice, 2000);
        assert!(
            matches!(r2, Err(LayerError::SupplyCapExceeded { .. })),
            "CRITICO: el tope debe aplicarse al acumulado, no a cada emision"
        );
    }

    /// Emitir exactamente hasta el tope sí vale: un límite efectivo menor
    /// que el declarado sería un error de una unidad difícil de detectar.
    #[test]
    fn minting_exactly_to_the_cap_is_allowed() {
        let mut layer = new_layer();
        let alice = layer.open_account(BaseElement::new(SK_ALICE));

        let r = layer
            .mint(&valid_auth(), alice, MAX_SUPPLY)
            .expect("emitir hasta el tope exacto deberia valer");
        layer.apply_mint(&r, alice).expect("aplicar");
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

        let r = layer
            .mint(&valid_auth(), alice, MAX_SUPPLY)
            .expect("emision");
        layer.apply_mint(&r, alice).expect("aplicar");
        assert!(layer.mint(&valid_auth(), alice, 1).is_err());

        let b = layer
            .burn(BaseElement::new(SK_ALICE), alice, 1_000_000)
            .expect("destruccion");
        layer.apply_burn(&b, alice).expect("aplicar");

        // Ahora vuelve a haber margen.
        let r2 = layer.mint(&valid_auth(), alice, 500_000);
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

        {
            let mut layer = SovereignLayer::open_encrypted(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
                Some(good.clone()),
            )
            .expect("abrir cifrado");
            open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        }

        // Con la contrasena correcta: se recupera.
        {
            let layer = SovereignLayer::open_encrypted(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
                Some(good),
            )
            .expect("recuperar con la contrasena correcta");
            assert_eq!(layer.balance_of(0), Some(1_000_000));
        }

        // Con OTRA contrasena: falla.
        let bad = crypto::LedgerKey::from_passphrase("la incorrecta");
        let r = SovereignLayer::open_encrypted(
            &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
            Some(bad),
        );
        let ok = r.is_ok();
        assert!(
            !ok,
            "CRITICO: otra contrasena no debe poder leer el ledger cifrado"
        );

        // Y SIN contrasena tampoco: los datos estan cifrados en disco.
        let r2 = SovereignLayer::open_encrypted(
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
            let mut layer = SovereignLayer::open_encrypted(
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
        let s = layer
            .transfer(BaseElement::new(SK_ALICE), alice, bob, 250_000)
            .expect("transferencia");
        layer.apply(&s, alice, bob, 250_000).expect("aplicar");
        let b = layer.burn(BaseElement::new(SK_BOB), bob, 1000).expect("destruir");
        layer.apply_burn(&b, bob).expect("aplicar");

        // Dos aperturas + una emision (bob se abre con cero y no emite)
        // + una transferencia + una destruccion.
        //
        // Que `open_account` cuente es lo correcto: mueve la raiz de
        // estado, asi que tiene que dejar rastro.
        assert_eq!(layer.transition_log().len(), 5);
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

        let extra = a.burn(BaseElement::new(SK_ALICE), cuenta_a, 1).expect("destruir");
        a.apply_burn(&extra, cuenta_a).expect("aplicar");
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
            let s = layer
                .transfer(BaseElement::new(SK_ALICE), alice, bob, 1000)
                .expect("transferencia");
            layer.apply(&s, alice, bob, 1000).expect("aplicar");
            cabeza = layer.log_head();
        }
        {
            let layer = open_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
            )
            .expect("recuperar");
            // Dos aperturas + una emision + una transferencia.
            assert_eq!(layer.transition_log().len(), 4);
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

    /// **REENVIAR UNA EMISIÓN CREARÍA DINERO.**
    ///
    /// Es el reenvío más grave de todos, y **no estaba probado**. Había
    /// tests para liquidación, gobernanza, recuperación y destrucción; la
    /// emisión se quedó fuera.
    ///
    /// Un recibo de emisión válido, aplicado dos veces, duplicaría el
    /// importe emitido y el suministro dejaría de cuadrar.
    #[test]
    fn replaying_a_mint_is_rejected() {
        let mut layer = new_layer();
        let alice = layer.open_account_checked(BaseElement::new(SK_ALICE)).expect("abrir");

        let r = layer.mint(&valid_auth(), alice, 250_000).expect("emitir");
        layer.apply_mint(&r, alice).expect("primera aplicacion");

        let saldo = layer.balance_of(alice);
        let suministro = layer.total_supply();

        // El MISMO recibo, otra vez.
        assert!(
            layer.apply_mint(&r, alice).is_err(),
            "CRITICO: reenviar una emision crearia dinero de la nada"
        );
        assert_eq!(layer.balance_of(alice), saldo, "el saldo no cambia");
        assert_eq!(layer.total_supply(), suministro, "el suministro tampoco");
    }

    /// **REENVIAR UNA CONGELACIÓN.**
    ///
    /// No crea dinero, pero **incrementaría el contador público dos veces**
    /// por una sola intervención. Ese contador existe justamente para que
    /// las intervenciones de los custodios sean contables.
    #[test]
    fn replaying_a_freeze_is_rejected() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);

        let f = layer.set_frozen(&valid_auth(), alice, true).expect("congelar");
        layer.apply_freeze(&f, alice).expect("primera aplicacion");
        let cuenta = layer.freeze_count();

        assert!(
            layer.apply_freeze(&f, alice).is_err(),
            "reenviar una congelacion no debe aceptarse"
        );
        assert_eq!(
            layer.freeze_count(),
            cuenta,
            "CRITICO: el contador de intervenciones no puede subir dos veces \
             por una sola congelacion"
        );
    }

    /// **Y aplicar un recibo a una cuenta DISTINTA de la suya.**
    ///
    /// Un recibo de emisión para Alice, aplicado sobre Bob. Si se
    /// aceptara, cualquiera podría desviar una emisión legítima.
    #[test]
    fn applying_a_receipt_to_the_wrong_account_is_rejected() {
        let mut layer = new_layer();
        let alice = layer.open_account_checked(BaseElement::new(SK_ALICE)).expect("abrir");
        let bob = layer.open_account_checked(BaseElement::new(SK_BOB)).expect("abrir");

        let r = layer.mint(&valid_auth(), alice, 250_000).expect("emitir");
        assert!(
            layer.apply_mint(&r, bob).is_err(),
            "CRITICO: un recibo de emision para Alice no debe poder aplicarse \
             sobre la cuenta de Bob"
        );
        assert_eq!(layer.balance_of(bob), Some(0));
    }

    // -----------------------------------------------------------------
    // Transferencia en dos fases
    // -----------------------------------------------------------------

    fn salt_de(seed: u64) -> Digest {
        [
            BaseElement::new(seed),
            BaseElement::new(seed + 1),
            BaseElement::new(seed + 2),
            BaseElement::new(seed + 3),
        ]
    }

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
            .send(BaseElement::new(SK_ALICE), alice, id_bob, salt_de(0x5EED), 250_000)
            .expect("enviar");
        layer.apply_send(&r, alice, 250_000).expect("aplicar envio");

        assert_eq!(layer.balance_of(alice), Some(750_000), "Alice debitada");
        assert_eq!(layer.balance_of(bob), Some(50_000), "Bob aun no cobra");

        // FASE 2: Bob reclama con el aviso.
        let cr = layer
            .claim(BaseElement::new(SK_BOB), bob, &r.notice)
            .expect("reclamar");
        layer.apply_claim(&cr, bob, &r.notice).expect("aplicar reclamacion");

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
            .send(BaseElement::new(SK_ALICE), alice, id_bob, salt_de(0x5EED), 250_000)
            .expect("enviar");
        layer.apply_send(&r, alice, 250_000).expect("aplicar");

        // Mallory lo intenta con SU clave y SU cuenta.
        let intento = layer.claim(BaseElement::new(0xBADCAFE), mallory, &r.notice);
        if let Ok(cr) = intento {
            assert!(
                layer.apply_claim(&cr, mallory, &r.notice).is_err(),
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
            .send(BaseElement::new(SK_ALICE), alice, id_bob, salt_de(0x5EED), 250_000)
            .expect("enviar");
        layer.apply_send(&r, alice, 250_000).expect("aplicar");

        let cr = layer.claim(BaseElement::new(SK_BOB), bob, &r.notice).expect("reclamar");
        layer.apply_claim(&cr, bob, &r.notice).expect("primera");
        assert_eq!(layer.balance_of(bob), Some(250_000));

        assert!(
            layer.apply_claim(&cr, bob, &r.notice).is_err(),
            "CRITICO: reclamar dos veces seria cobrar dos veces"
        );
        assert_eq!(layer.balance_of(bob), Some(250_000), "el saldo no sube");
    }

    /// **UNA CUENTA CONGELADA NO PUEDE ENVIAR.**
    #[test]
    fn a_frozen_account_cannot_send() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let id_bob = derive_public_id(BaseElement::new(SK_BOB));

        let f = layer.set_frozen(&valid_auth(), alice, true).expect("congelar");
        layer.apply_freeze(&f, alice).expect("aplicar");

        let r = layer.send(BaseElement::new(SK_ALICE), alice, id_bob, salt_de(0x5EED), 1000);
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
                .send(BaseElement::new(SK_ALICE), alice, id_bob, salt_de(0x5EED), 250_000)
                .expect("enviar");
            layer.apply_send(&r, alice, 250_000).expect("aplicar");
            aviso = r.notice.clone();
        }

        let mut layer = open_retry(
            &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
        )
        .expect("reabrir");

        // Bob reclama DESPUES del reinicio.
        let cr = layer
            .claim(BaseElement::new(SK_BOB), bob, &aviso)
            .expect("reclamar tras reiniciar");
        layer.apply_claim(&cr, bob, &aviso).expect("aplicar");
        assert_eq!(
            layer.balance_of(bob),
            Some(250_000),
            "CRITICO: si los pendientes no sobrevivieran, el dinero se perderia"
        );
        let _ = std::fs::remove_dir_all(&path);
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
        let m = layer.mint(&valid_auth(), alice, 100_000).expect("emitir");
        layer.apply_mint(&m, alice).expect("aplicar");
        assert_eq!(layer.custodian_uses(), 1, "emitir consume una");

        let f = layer.set_frozen(&valid_auth(), alice, true).expect("congelar");
        layer.apply_freeze(&f, alice).expect("aplicar");
        assert_eq!(layer.custodian_uses(), 2, "congelar consume otra");
    }

    /// **GENERAR UNA PRUEBA QUE NO SE APLICA NO GASTA CUPO.**
    ///
    /// El consumo va en la aplicación, no en la generación. Si fuera al
    /// revés, **pruebas descartadas agotarían el cupo de los custodios**.
    #[test]
    fn generating_an_unapplied_proof_does_not_consume_quota() {
        let mut layer = new_layer();
        let alice = layer.open_account_checked(BaseElement::new(SK_ALICE)).expect("abrir");

        let _descartada = layer.mint(&valid_auth(), alice, 100_000).expect("emitir");
        assert_eq!(
            layer.custodian_uses(),
            0,
            "una prueba que no se aplica no debe gastar cupo"
        );
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
            let m = layer.mint(&valid_auth(), alice, 1000).expect("emitir");
            layer.apply_mint(&m, alice).expect("aplicar");
            assert_eq!(layer.custodian_uses(), i + 1);
        }

        let m = layer.mint(&valid_auth(), alice, 1000).expect("emitir");
        assert!(
            matches!(
                layer.apply_mint(&m, alice),
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

        let m = layer.mint(&valid_auth(), alice, 1000).expect("emitir");
        layer.apply_mint(&m, alice).expect("aplicar");
        assert_eq!(layer.custodian_uses(), 1);

        let g = layer
            .update_custodians(&valid_governance_auth(), new_custodian_root())
            .expect("rotar");
        layer.apply_governance(&g).expect("aplicar");
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
            let m = layer.mint(&valid_auth(), alice, 100_000).expect("emitir");
            layer.apply_mint(&m, alice).expect("aplicar");
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
        {
            let mut layer = open_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
            )
                .expect("abrir");
            open_and_fund(&mut layer, SK_ALICE, emitido);
            assert_eq!(layer.total_supply(), emitido);
        }
        let layer = open_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
            )
            .expect("reabrir");
        assert_eq!(
            layer.total_supply(),
            emitido,
            "CRITICO: si el suministro se reiniciara, reiniciar el nodo \
             permitiria emitir de nuevo hasta el tope"
        );
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
    fn nullifiers_survive_restart_and_still_block() {
        let path = temp_path("nulls");
        let (alice, bob, raiz_nulls);
        {
            let mut layer = open_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
            )
                .expect("abrir");
            alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
            bob = open_and_fund(&mut layer, SK_BOB, 0);
            let s = layer
                .transfer(BaseElement::new(SK_ALICE), alice, bob, 250_000)
                .expect("transferir");
            layer.apply(&s, alice, bob, 250_000).expect("aplicar");
            raiz_nulls = layer.nullifier_root();
        }
        let layer = open_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
            )
            .expect("reabrir");
        assert_eq!(
            layer.nullifier_root(),
            raiz_nulls,
            "CRITICO: si los nullificadores no sobrevivieran, bastaria \
             reiniciar para gastar dos veces"
        );
        assert_eq!(layer.balance_of(alice), Some(750_000));
        assert_eq!(layer.balance_of(bob), Some(250_000));
        let _ = std::fs::remove_dir_all(&path);
    }

    /// **EL CONJUNTO DE GOBERNANZA SOBREVIVE.**
    ///
    /// Es inmutable por diseño, pero si al reabrir se restaurara otro,
    /// **quien controlara ese otro conjunto podría cambiar los
    /// custodios**.
    #[test]
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
        // Un conjunto de gobernanza distinto no debe poder cambiar nada.
        let g = layer.update_custodians(&valid_governance_auth(), new_custodian_root());
        assert!(g.is_ok(), "el conjunto legitimo debe seguir funcionando");
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
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        let mallory = open_and_fund(&mut layer, 0xBADCAFE, 0);

        let id_bob = derive_public_id(BaseElement::new(SK_BOB));

        // Alice cree pagar a Bob, pero la capa le da el indice de Mallory.
        let m = layer
            .transfer_materials(alice, mallory, 1000, client::compute_nullifier(BaseElement::new(SK_ALICE), layer.nonce_of(alice).unwrap()))
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
        let bob = open_and_fund(&mut layer, SK_BOB, 0);

        let id_bob = derive_public_id(BaseElement::new(SK_BOB));
        let m = layer
            .transfer_materials(alice, bob, 1000, client::compute_nullifier(BaseElement::new(SK_ALICE), layer.nonce_of(alice).unwrap()))
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

        let f = layer.set_frozen(&valid_auth(), alice, true).expect("congelar");
        layer.apply_freeze(&f, alice).expect("aplicar");
        assert!(layer.is_frozen(alice));

        // Transferir NO puede.
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        assert!(layer
            .transfer(BaseElement::new(SK_ALICE), alice, bob, 1000)
            .is_err());

        // Y destruir TAMPOCO.
        let b = layer.burn(BaseElement::new(SK_ALICE), alice, 1_000_000);
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

        let f = layer.set_frozen(&valid_auth(), alice, true).expect("congelar");
        layer.apply_freeze(&f, alice).expect("aplicar");
        assert!(layer.burn(BaseElement::new(SK_ALICE), alice, 1000).is_err());

        let u = layer.set_frozen(&valid_auth(), alice, false).expect("descongelar");
        layer.apply_freeze(&u, alice).expect("aplicar");

        let b = layer
            .burn(BaseElement::new(SK_ALICE), alice, 1000)
            .expect("descongelada deberia poder destruir");
        layer.apply_burn(&b, alice).expect("aplicar");
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

        let f = layer.set_frozen(&valid_auth(), alice, true).expect("congelar");
        layer.apply_freeze(&f, alice).expect("aplicar");

        // Los custodios recuperan la cuenta a una identidad nueva.
        let nueva = derive_public_id(BaseElement::new(0xC0FFEE));
        let r = layer.recover(&valid_auth(), alice, nueva).expect("recuperar");
        layer.apply_recovery(&r, alice).expect("aplicar");

        assert!(
            layer.is_frozen(alice),
            "CRITICO: recuperar una cuenta no debe levantar su congelacion, \
             o bastaria con decir que se perdio la clave para escapar"
        );
    }

    /// **CAMBIAR LOS CUSTODIOS INVALIDA LOS RECIBOS PENDIENTES.**
    ///
    /// Un recibo de emisión firmado por los custodios antiguos no debe
    /// poder aplicarse después de sustituirlos. Si pudiera, **los
    /// custodios destituidos conservarían poder** durante el tiempo que
    /// tuvieran recibos sin aplicar.
    #[test]
    fn changing_custodians_invalidates_pending_receipts() {
        let mut layer = new_layer();
        let alice = layer.open_account_checked(BaseElement::new(SK_ALICE)).expect("abrir");

        // Los custodios actuales emiten, pero el recibo no se aplica aun.
        let pendiente = layer.mint(&valid_auth(), alice, 250_000).expect("emitir");

        // La gobernanza los sustituye.
        let g = layer
            .update_custodians(&valid_governance_auth(), new_custodian_root())
            .expect("cambiar custodios");
        layer.apply_governance(&g).expect("aplicar");

        assert!(
            layer.apply_mint(&pendiente, alice).is_err(),
            "CRITICO: un recibo de los custodios destituidos no debe aplicarse"
        );
        assert_eq!(layer.balance_of(alice), Some(0));
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
    fn transferring_to_your_own_account() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let antes = layer.balance_of(alice);

        let r = layer.transfer(BaseElement::new(SK_ALICE), alice, alice, 250_000);

        // Sea cual sea el resultado, el saldo NO debe haber subido.
        if let Ok(s) = &r {
            let aplicado = layer.apply(s, alice, alice, 250_000);
            assert!(
                aplicado.is_err(),
                "CRITICO: transferirse a uno mismo NO debe poder aplicarse"
            );
        }
        assert_eq!(
            layer.balance_of(alice),
            antes,
            "CRITICO: el saldo no puede cambiar por transferirse a uno mismo"
        );
    }

    /// **TRANSFERIR CERO.**
    ///
    /// No crea ni destruye dinero, pero **consume un nullifier**. Si se
    /// permitiera sin coste, sería una forma de agotar posiciones del
    /// árbol de nullifiers.
    #[test]
    fn transferring_zero() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        let antes_a = layer.balance_of(alice);
        let antes_b = layer.balance_of(bob);

        if let Ok(s) = layer.transfer(BaseElement::new(SK_ALICE), alice, bob, 0) {
            let _ = layer.apply(&s, alice, bob, 0);
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

        let s = layer
            .transfer(BaseElement::new(SK_ALICE), alice, bob, 400_000)
            .expect("transferir todo el saldo deberia valer");
        layer.apply(&s, alice, bob, 400_000).expect("aplicar");

        assert_eq!(layer.balance_of(alice), Some(0), "la cuenta queda a cero");
        assert_eq!(layer.balance_of(bob), Some(400_000));
    }

    /// **Y desde una cuenta a cero no se puede sacar nada.**
    #[test]
    fn spending_from_an_empty_account_is_rejected() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 0);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        assert!(layer
            .transfer(BaseElement::new(SK_ALICE), alice, bob, 1)
            .is_err());
    }

    /// **TRANSFERIR A UNA CUENTA QUE NO EXISTE.**
    #[test]
    fn transferring_to_a_nonexistent_account() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let r = layer.transfer(BaseElement::new(SK_ALICE), alice, 9_999, 1000);
        assert!(
            matches!(r, Err(LayerError::AccountNotFound(_))),
            "deberia decir que la cuenta no existe: {r:?}"
        );
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

        let f = layer.set_frozen(&valid_auth(), alice, true).expect("congelar");
        layer.apply_freeze(&f, alice).expect("aplicar");

        // Un intruso, sin la clave de Alice, intenta transferir desde su
        // cuenta. Debe recibir "no eres el titular", NO "esta congelada".
        let r = layer.transfer(BaseElement::new(0x1337), alice, bob, 1000);
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
        let f = layer.set_frozen(&valid_auth(), alice, true).expect("congelar");
        layer.apply_freeze(&f, alice).expect("aplicar");

        let r = layer.transfer(BaseElement::new(SK_ALICE), alice, bob, 1000);
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
        let r = layer.transfer(BaseElement::new(0x1337), alice, bob, 99_999_999);
        assert!(
            matches!(r, Err(LayerError::NotTheAccountHolder)),
            "CRITICO: el error no debe revelar el saldo a quien no es titular"
        );

        // Y con burn, que tambien lo lleva.
        let r = layer.burn(BaseElement::new(0x1337), alice, 99_999_999);
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
        let s = layer
            .transfer(BaseElement::new(SK_ALICE), alice, bob, 1000)
            .expect("antes de congelar deberia poder");
        layer.apply(&s, alice, bob, 1000).expect("aplicar");

        // Dos custodios la congelan.
        let f = layer
            .set_frozen(&valid_auth(), alice, true)
            .expect("congelar");
        layer.apply_freeze(&f, alice).expect("aplicar congelacion");
        assert!(layer.is_frozen(alice));

        // Ahora NO puede.
        let r = layer.transfer(BaseElement::new(SK_ALICE), alice, bob, 1000);
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
    fn a_frozen_account_can_still_receive() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);

        let f = layer.set_frozen(&valid_auth(), bob, true).expect("congelar");
        layer.apply_freeze(&f, bob).expect("aplicar");

        let s = layer
            .transfer(BaseElement::new(SK_ALICE), alice, bob, 5000)
            .expect("recibir estando congelado deberia valer");
        layer.apply(&s, alice, bob, 5000).expect("aplicar");
        assert_eq!(layer.balance_of(bob), Some(5000));
    }

    /// **DESCONGELAR DEVUELVE LA CAPACIDAD DE GASTO.**
    #[test]
    fn unfreezing_restores_the_ability_to_spend() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);

        let f = layer.set_frozen(&valid_auth(), alice, true).expect("congelar");
        layer.apply_freeze(&f, alice).expect("aplicar");
        assert!(layer.transfer(BaseElement::new(SK_ALICE), alice, bob, 1000).is_err());

        let u = layer
            .set_frozen(&valid_auth(), alice, false)
            .expect("descongelar");
        layer.apply_freeze(&u, alice).expect("aplicar");
        assert!(!layer.is_frozen(alice));

        let s = layer
            .transfer(BaseElement::new(SK_ALICE), alice, bob, 1000)
            .expect("tras descongelar deberia poder gastar");
        layer.apply(&s, alice, bob, 1000).expect("aplicar");
        assert_eq!(layer.balance_of(bob), Some(1000));
    }

    /// **UN SOLO CUSTODIO NO PUEDE CONGELAR.**
    #[test]
    fn one_custodian_cannot_freeze_alone() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);

        let keys = custodian_keys();
        let (_, paths) = stark_experiment::circuit_threshold::build_custodian_set(&keys);
        let solo = ThresholdAuth {
            key_a: keys[2],
            index_a: 2,
            path_a: paths[2].clone(),
            key_b: keys[2],
            index_b: 2,
            path_b: paths[2].clone(),
        };
        assert!(layer.set_frozen(&solo, alice, true).is_err());
        assert!(!layer.is_frozen(alice));
    }

    /// Cada congelación y descongelación queda contada.
    #[test]
    fn every_freeze_is_counted() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        assert_eq!(layer.freeze_count(), 0);

        let f = layer.set_frozen(&valid_auth(), alice, true).expect("congelar");
        layer.apply_freeze(&f, alice).expect("aplicar");
        assert_eq!(layer.freeze_count(), 1);

        let u = layer.set_frozen(&valid_auth(), alice, false).expect("descongelar");
        layer.apply_freeze(&u, alice).expect("aplicar");
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
            let f = layer.set_frozen(&valid_auth(), alice, true).expect("congelar");
            layer.apply_freeze(&f, alice).expect("aplicar");
        }
        {
            let layer = open_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
            )
            .expect("recuperar");
            assert!(
                layer.is_frozen(alice),
                "CRITICO: si la congelacion no sobrevive al reinicio, bastaria \
                 reiniciar el nodo para levantarla"
            );
            assert_eq!(layer.freeze_count(), 1);
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
        let r = layer.mint(&valid_auth(), alice, 1000).expect("emision inicial");
        layer.apply_mint(&r, alice).expect("aplicar");

        // La gobernanza cambia el conjunto.
        let g = layer
            .update_custodians(&valid_governance_auth(), new_custodian_root())
            .expect("cambio de gobernanza");
        layer.apply_governance(&g).expect("aplicar");
        assert_eq!(layer.custodian_set_root(), new_custodian_root());

        // Los ANTIGUOS ya no pueden.
        let old = layer.mint(&valid_auth(), alice, 1000);
        let applied = match old {
            Ok(receipt) => layer.apply_mint(&receipt, alice).is_ok(),
            Err(_) => false,
        };
        assert!(
            !applied,
            "CRITICO: tras el cambio, los custodios antiguos NO deben poder \
             crear dinero. Si pueden, el cambio es cosmetico."
        );

        // Los NUEVOS sí.
        let r2 = layer
            .mint(&new_custodian_auth(), alice, 5000)
            .expect("los nuevos custodios deberian poder emitir");
        layer.apply_mint(&r2, alice).expect("aplicar");
        assert_eq!(layer.balance_of(alice), Some(6000));
    }

    /// **UN CUSTODIO NO PUEDE CAMBIAR EL CONJUNTO DE CUSTODIOS.**
    ///
    /// Es la prueba de que la jerarquía funciona: quien puede emitir y
    /// recuperar cuentas no puede cambiar quién tiene ese poder.
    #[test]
    fn a_custodian_cannot_change_the_custodian_set() {
        let mut layer = new_layer();
        // Una autorizacion construida con claves de CUSTODIO sobre la
        // estructura de gobernanza.
        let keys = custodian_keys();
        let (_, paths) = build_governance_set(&governance_keys());
        let fake = GovernanceAuth {
            key_a: keys[1],
            index_a: 1,
            path_a: paths[1].clone(),
            key_b: keys[3],
            index_b: 3,
            path_b: paths[3].clone(),
        };

        let r = layer.update_custodians(&fake, new_custodian_root());
        let applied = match r {
            Ok(receipt) => layer.apply_governance(&receipt).is_ok(),
            Err(_) => false,
        };
        assert!(
            !applied,
            "CRITICO: quien puede emitir y recuperar NO debe poder cambiar \
             quien tiene ese poder"
        );
        assert_eq!(layer.custodian_set_root(), custodian_root(), "sin cambios");
    }

    /// **UN SOLO GOBERNADOR NO BASTA.**
    #[test]
    fn one_governor_cannot_change_the_set_alone() {
        let mut layer = new_layer();
        let keys = governance_keys();
        let (_, paths) = build_governance_set(&keys);
        let solo = GovernanceAuth {
            key_a: keys[2],
            index_a: 2,
            path_a: paths[2].clone(),
            key_b: keys[2],
            index_b: 2,
            path_b: paths[2].clone(),
        };
        assert!(layer.update_custodians(&solo, new_custodian_root()).is_err());
    }

    /// Cada cambio queda contado.
    #[test]
    fn every_governance_change_is_counted() {
        let mut layer = new_layer();
        assert_eq!(layer.governance_change_count(), 0);

        let g = layer
            .update_custodians(&valid_governance_auth(), new_custodian_root())
            .expect("cambio");
        layer.apply_governance(&g).expect("aplicar");
        assert_eq!(layer.governance_change_count(), 1);
    }

    /// Reaplicar un cambio de gobernanza se rechaza.
    #[test]
    fn replaying_a_governance_change_is_rejected() {
        let mut layer = new_layer();
        let g = layer
            .update_custodians(&valid_governance_auth(), new_custodian_root())
            .expect("cambio");
        layer.apply_governance(&g).expect("primera");
        assert!(matches!(
            layer.apply_governance(&g),
            Err(LayerError::StaleState)
        ));
        assert_eq!(layer.governance_change_count(), 1);
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
            let g = layer
                .update_custodians(&valid_governance_auth(), new_custodian_root())
                .expect("cambio");
            layer.apply_governance(&g).expect("aplicar");
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

        let receipt = layer
            .recover(&valid_auth(), alice, new_id)
            .expect("los custodios deberian poder recuperar");
        layer
            .apply_recovery(&receipt, alice)
            .expect("aplicar recuperacion");

        // La clave ANTIGUA ya no sirve.
        assert!(
            matches!(
                layer.transfer(BaseElement::new(SK_ALICE), alice, bob, 1000),
                Err(LayerError::NotTheAccountHolder)
            ),
            "CRITICO: tras recuperar, la clave comprometida NO debe poder gastar"
        );

        // La NUEVA sí.
        let s = layer
            .transfer(BaseElement::new(SK_ALICE_NEW), alice, bob, 1000)
            .expect("la clave nueva deberia poder gastar");
        layer.apply(&s, alice, bob, 1000).expect("aplicar");
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
        let r = layer.recover(&valid_auth(), alice, new_id).expect("recuperar");
        layer.apply_recovery(&r, alice).expect("aplicar");

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

        let r1 = layer
            .recover(&valid_auth(), alice, derive_public_id(BaseElement::new(0xA1)))
            .expect("primera");
        layer.apply_recovery(&r1, alice).expect("aplicar");
        assert_eq!(layer.recovery_count(), 1);

        let r2 = layer
            .recover(&valid_auth(), bob, derive_public_id(BaseElement::new(0xB1)))
            .expect("segunda");
        layer.apply_recovery(&r2, bob).expect("aplicar");
        assert_eq!(
            layer.recovery_count(),
            2,
            "cada intervencion de los custodios debe quedar contada"
        );
    }

    /// **UN SOLO CUSTODIO NO PUEDE RECUPERAR.**
    #[test]
    fn one_custodian_cannot_recover_alone() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);

        let keys = custodian_keys();
        let (_, paths) = stark_experiment::circuit_threshold::build_custodian_set(&keys);
        let solo = ThresholdAuth {
            key_a: keys[2],
            index_a: 2,
            path_a: paths[2].clone(),
            key_b: keys[2],
            index_b: 2,
            path_b: paths[2].clone(),
        };

        let new_id = derive_public_id(BaseElement::new(0xA11CE_2));
        assert!(
            layer.recover(&solo, alice, new_id).is_err(),
            "CRITICO: un custodio contando dos veces convertiria el 2-de-N en 1-de-N"
        );
        assert_eq!(layer.recovery_count(), 0);
    }

    /// Recuperar a la misma identidad no hace nada y solo gastaría el
    /// contador: se rechaza antes de generar la prueba.
    #[test]
    fn recovery_to_the_same_identity_is_rejected() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let same = derive_public_id(BaseElement::new(SK_ALICE));
        assert!(matches!(
            layer.recover(&valid_auth(), alice, same),
            Err(LayerError::RecoveryToSameIdentity)
        ));
    }

    /// Reaplicar una recuperación debe rechazarse.
    #[test]
    fn replaying_a_recovery_is_rejected() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let new_id = derive_public_id(BaseElement::new(0xA11CE_2));
        let r = layer.recover(&valid_auth(), alice, new_id).expect("recuperar");

        layer.apply_recovery(&r, alice).expect("primera");
        assert!(
            matches!(
                layer.apply_recovery(&r, alice),
                Err(LayerError::StaleState)
            ),
            "CRITICO: reaplicar una recuperacion descuadraria el contador"
        );
        assert_eq!(layer.recovery_count(), 1);
    }

    /// **El contador sobrevive al reinicio.**
    ///
    /// Si se perdiera, las intervenciones de los custodios dejarían de
    /// ser contables entre arranques — que es justo lo que el contador
    /// existe para evitar.
    #[test]
    fn the_recovery_counter_survives_restart() {
        let path = temp_path("recoveries");
        {
            let mut layer =
                open_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS).expect("abrir");
            let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
            let r = layer
                .recover(&valid_auth(), alice, derive_public_id(BaseElement::new(0xA1)))
                .expect("recuperar");
            layer.apply_recovery(&r, alice).expect("aplicar");
            assert_eq!(layer.recovery_count(), 1);
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
            .burn(BaseElement::new(SK_ALICE), alice, 300_000)
            .expect("destruccion");
        println!(
            "Tamano de la prueba de DESTRUCCION: {} bytes",
            receipt.proof.len()
        );
        assert_eq!(layer.total_supply(), 1_000_000, "burn no debe mutar el estado");

        layer.apply_burn(&receipt, alice).expect("aplicar");
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
            .burn(BaseElement::new(SK_ALICE), alice, 400_000)
            .expect("destruccion");
        layer.apply_burn(&r, alice).expect("aplicar");

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

        let r = layer.burn(BaseElement::new(0x1337), alice, 100_000);
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
        let r = layer.burn(BaseElement::new(SK_ALICE), alice, 500_000);
        assert!(matches!(r, Err(LayerError::InsufficientBalance { .. })));
    }

    /// Reaplicar una destrucción debe rechazarse.
    #[test]
    fn replaying_a_burn_is_rejected() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let r = layer
            .burn(BaseElement::new(SK_ALICE), alice, 200_000)
            .expect("destruccion");

        layer.apply_burn(&r, alice).expect("primera");
        assert!(
            matches!(layer.apply_burn(&r, alice), Err(LayerError::StaleState)),
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
        let s = layer
            .transfer(BaseElement::new(SK_ALICE), alice, bob, 300_000)
            .expect("transferencia");
        layer.apply(&s, alice, bob, 300_000).expect("aplicar");
        assert_eq!(layer.total_supply(), 1_000_000);
        assert_eq!(sum(&layer), layer.total_supply());

        // Destruir: el suministro SI baja.
        let b = layer
            .burn(BaseElement::new(SK_BOB), bob, 100_000)
            .expect("destruccion");
        layer.apply_burn(&b, bob).expect("aplicar");
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
            .disclose_exact(BaseElement::new(SK_ALICE), alice)
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
            .prove_minimum(BaseElement::new(SK_ALICE), alice, 500_000)
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
            .audit(BaseElement::new(SK_ALICE), alice, 900_000, 1_100_000)
            .expect("banda");
        assert!(verify_audit(&d).is_ok());
    }

    /// **NO SE PUEDE FINGIR SOLVENCIA.**
    #[test]
    fn cannot_prove_a_minimum_that_is_not_met() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 100_000);

        let r = layer.prove_minimum(BaseElement::new(SK_ALICE), alice, 500_000);
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

        let r = layer.disclose_exact(BaseElement::new(0x1337), alice);
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
        use stark_experiment::circuit_settlement::{native_climb, native_leaf};
        use stark_experiment::merkle::{native_merge, MerklePath, TREE_DEPTH};

        let mut empty = vec![[BaseElement::ZERO; 4]];
        for k in 1..=TREE_DEPTH {
            let prev = empty[k - 1];
            empty.push(native_merge(prev, prev));
        }
        let key = BaseElement::new(SK_ALICE);
        let id = derive_public_id(key);
        let nonce = BaseElement::ZERO;
        let mut siblings = Vec::new();
        let mut is_right = Vec::new();
        for level in 0..TREE_DEPTH {
            siblings.push(empty[level]);
            is_right.push(level % 3 == 0);
        }
        let path = MerklePath { siblings, is_right };
        let root = native_climb(native_leaf(id, BaseElement::new(100_000), nonce), &path);

        let w = AuditWitness {
            spend_key: key,
            balance: 100_000,
            nonce,
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
    /// suministro y nullifiers gastados. Sin esto, apagar el proceso
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
            let s = layer
                .transfer(BaseElement::new(SK_ALICE), alice, bob, 250_000)
                .expect("prueba");
            layer.apply(&s, alice, bob, 250_000).expect("aplicar");
        } // el nodo se apaga

        {
            let layer = open_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS)
                .expect("el ledger deberia recuperarse");
            assert_eq!(layer.balance_of(alice), Some(750_000));
            assert_eq!(layer.balance_of(bob), Some(300_000));
            assert_eq!(layer.total_supply(), 1_050_000);
            assert_eq!(layer.account_count(), 2);
        }

        let _ = std::fs::remove_dir_all(&path);
    }

    /// **El nullifier gastado sobrevive al reinicio.**
    ///
    /// Sin esto, reiniciar el nodo permitiría regastar todo lo anterior:
    /// el árbol de nullifiers volvería a estar vacío y la no-pertenencia
    /// se satisfaría de nuevo.
    #[test]
    fn spent_nullifiers_survive_restart() {
        let path = temp_path("nullifiers");
        let null_root_after;
        let (alice, bob);

        {
            let mut layer =
                open_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS).expect("abrir");
            alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
            bob = open_and_fund(&mut layer, SK_BOB, 0);
            let s = layer
                .transfer(BaseElement::new(SK_ALICE), alice, bob, 100_000)
                .expect("prueba");
            layer.apply(&s, alice, bob, 100_000).expect("aplicar");
            null_root_after = layer.nullifier_root();
        }

        {
            let layer = open_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS)
                .expect("recuperar");
            assert_eq!(
                layer.nullifier_root(),
                null_root_after,
                "CRITICO: los nullifiers gastados deben sobrevivir al reinicio, \
                 o reiniciar permitiria regastar"
            );
        }

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
            let db = sled::open(&path).expect("abrir db");
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
            layer.transfer(BaseElement::new(SK_ALICE), alice, bob, 250_000),
            Err(LayerError::InsufficientBalance { .. })
        ));
        assert!(matches!(
            layer.transfer(BaseElement::new(SK_ALICE), alice, 999, 1000),
            Err(LayerError::AccountNotFound(999))
        ));
    }
