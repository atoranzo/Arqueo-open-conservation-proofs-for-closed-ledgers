//! Tests de la capa. Se mantienen juntos porque comparten los
//! ayudantes `new_layer`, `open_and_fund` y `temp_path`.

// Estos tests ejercitan la via ANTIGUA a proposito: sigue siendo la unica
// para `mint` y `mint_pending`, y sus propiedades hay que comprobarlas
// igual. El aviso de obsolescencia se silencia AQUI, no en la definicion,
// para que siga saltando en codigo nuevo.
// Ampara `open_account` (64 bits), viva y opt-in por §97.4 (fuera de B
// por §160). La vía-recibo de custodios que antes también amparaba se
// retiró en B-3 (§171).
#![allow(deprecated)]

use super::*;

    use crate::tests_support::*;
    // `derive_public_id_wide` no llega por `use super::*`: lib.rs solo
    // reexporta la estrecha.
    use stark_experiment::native::derive_public_id_wide;

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
            l.records.values().map(|r| r.balance).sum()
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
        // El tope esta lleno, asi que este testigo afirma un enunciado
        // FALSO, y quien lo rechaza depende del modo: en depuracion el
        // probador al generar, en release la capa al aplicar. Los dos estan
        // bien (§77.1). Sin gancho de panico: el §45 midio su carrera.
        let subida = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            mint_climb_proof(&layer, alice, 1)
        }));
        let (pa, ia, pb, ib) = delegated_pair(op, 1, 3);
        match subida {
            Err(_) => {}
            Ok(subida) => assert!(matches!(
                layer.apply_mint_delegated(subida, pa, ia, pb, ib, alice, 1),
                Err(LayerError::SupplyCapExceeded { .. })
            )),
        }

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
        quitar_libro(&path);
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
        quitar_libro(&path);
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

        quitar_libro(&path);
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
        quitar_libro(&path);
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
        quitar_libro(&path);
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
        quitar_libro(&path);
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
        quitar_libro(&path);
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
        quitar_libro(&path);
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
        quitar_libro(&path);
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
        // El exceso es un enunciado FALSO, y quien lo rechaza depende del
        // modo: el probador al generar en depuracion, la capa al aplicar en
        // release. Los dos estan bien (§77.1), y lo que no puede pasar en
        // ningun modo es que se emita: lo exige el tope de abajo. Sin gancho
        // de panico: el §45 midio su carrera.
        let subida = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            mint_climb_proof(&layer, alice, exceso)
        }));
        let (pa, ia, pb, ib) = delegated_pair(op, 1, 3);
        match subida {
            Err(_) => {}
            Ok(subida) => {
                let r = layer.apply_mint_delegated(subida, pa, ia, pb, ib, alice, exceso);
                assert!(
                    matches!(r, Err(LayerError::SupplyCapExceeded { .. })),
                    "CRITICO: tras reiniciar, el tope debe seguir imponiendose sobre \
                     el suministro YA emitido. Salio: {r:?}"
                );
            }
        }

        // Y hasta el tope, si — por la via real.
        fund_delegated(&mut layer, alice, MAX_SUPPLY - emitido);
        assert_eq!(layer.total_supply(), MAX_SUPPLY, "clavado en el tope");
        quitar_libro(&path);
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
        quitar_libro(&path);
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
        quitar_libro(&path);
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
        quitar_libro(&path);
    }

    /// §390: descongelar tambien tiene que sobrevivir al reinicio. Antes,
    /// `commit` escribia las hojas ocupadas y nunca borraba la clave
    /// `froz:` de una cuenta descongelada: al reabrir, la clave huerfana
    /// volvia a congelarla.
    #[test]
    fn una_descongelacion_sobrevive_el_reinicio() {
        let path = temp_path("descongelar");
        let alice;
        {
            let mut layer = open_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
            )
            .expect("abrir");
            alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
            set_frozen_delegated(&mut layer, alice, true);
            set_frozen_delegated(&mut layer, alice, false);
            assert!(!layer.is_frozen(alice));
        }
        {
            let mut layer = open_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
            )
            .expect("recuperar");
            assert!(
                !layer.is_frozen(alice),
                "CRITICO: una cuenta descongelada volvio a estar congelada al \
                 reiniciar (clave froz: huerfana en disco)"
            );
            assert_eq!(layer.freeze_count(), 2);
            let bob = open_and_fund(&mut layer, SK_BOB, 0);
            let estado = state_of(&layer, alice);
            let receptor = layer.public_id_of(bob).expect("cuenta");
            let r = layer.send(
                BaseElement::new(SK_ALICE),
                alice,
                &estado,
                receptor,
                salt_de(0xF00E),
                1000,
            );
            assert!(
                r.is_ok(),
                "CRITICO: una cuenta descongelada debe poder gastar tras \
                 reiniciar. Salio: {:?}",
                r.err()
            );
        }
        quitar_libro(&path);
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
        quitar_libro(&path);
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
        quitar_libro(&path);
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
        let _bob = open_and_fund(&mut layer, SK_BOB, 500_000);

        let sum = |l: &SovereignLayer| -> u64 {
            l.records.values().map(|r| r.balance).sum()
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
            l.records.values().map(|r| r.balance).sum()
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

    /// **LA COTA NATIVA DE LA BANDA (§641).** El supervisor no acepta una
    /// banda con `lower > upper` ni con `upper` fuera del rango sano del
    /// campo (`> 2^62 - 1`): `verify_audit` lo rechaza ANTES de tocar la
    /// prueba, asi que cierra una banda forjada aunque la prueba fuera
    /// fabricada fuera de la casa.
    #[test]
    fn verify_audit_rechaza_una_banda_fuera_del_rango_sano() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let mut d = layer
            .audit(BaseElement::new(SK_ALICE), alice, &state_of(&layer, alice), 0, 2_000_000)
            .expect("banda");
        assert!(verify_audit(&d).is_ok(), "la banda valida verifica");
        // `upper` por encima de 2^62 - 1.
        d.public_inputs.upper =
            BaseElement::new(stark_experiment::circuit_audit::MAX_VALUE + 1);
        assert!(
            matches!(verify_audit(&d), Err(LayerError::BalanceOutsideBand { .. })),
            "una banda con upper > 2^62 - 1 debe rechazarse"
        );
        // `lower > upper`.
        let mut d2 = layer
            .audit(BaseElement::new(SK_ALICE), alice, &state_of(&layer, alice), 0, 2_000_000)
            .expect("banda");
        d2.public_inputs.lower = BaseElement::new(5);
        d2.public_inputs.upper = BaseElement::new(4);
        assert!(
            matches!(verify_audit(&d2), Err(LayerError::BalanceOutsideBand { .. })),
            "una banda con lower > upper debe rechazarse"
        );
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
        use stark_experiment::native::{native_climb, native_leaf_salted};
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

        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| prover.prove(trace)));

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

        quitar_libro(&path);
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

        quitar_libro(&path);
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

        quitar_libro(&path);
    }

    /// **UN SUMINISTRO QUE NO CUADRA CON LOS SALDOS SE DETECTA AL ABRIR.**
    ///
    /// Gemelo del de arriba, y el que faltaba. `root:state` cubre los saldos
    /// -la hoja lleva el balance-, pero `meta:supply` es un escalar que
    /// NINGUNA raiz cubre: se lee de disco y, hasta el §379, se creia.
    ///
    /// Sin esta puerta un ledger cuyo suministro miente se abre EN SILENCIO,
    /// y el nodo emite contra un tope calculado sobre una cifra falsa.
    ///
    /// La manipulacion NO toca `root:state`, asi que la puerta de cuentas
    /// pasa y solo puede cazarlo la de conservacion. Por eso el `matches!`
    /// exige el `what` LITERAL y no `{ .. }`: un verde que no pide
    /// explicacion es peor que un rojo.
    #[test]
    fn a_supply_that_does_not_match_the_balances_is_detected_at_startup() {
        let path = temp_path("supply_mentiroso");

        {
            let mut layer = open_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
            )
            .expect("abrir");
            open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        }

        // Sin clave, `seal` guarda el valor EN CLARO (`persistence.rs`), asi
        // que el suministro son ocho bytes little-endian legibles. Se LEE
        // antes de falsearlo: si esa premisa se cae, falla aqui y con su
        // nombre, no mas abajo y por otra razon.
        {
            let db = sled_open_retry(&path);
            let antes = db.get(b"meta:supply").expect("leer").expect("existe");
            let mut ocho = [0u8; 8];
            ocho.copy_from_slice(&antes[..8]);
            assert_eq!(
                u64::from_le_bytes(ocho),
                1_000_000,
                "un ledger sin cifrar guarda el suministro en claro"
            );
            db.insert(b"meta:supply", 1_000_001u64.to_le_bytes().to_vec())
                .expect("insertar");
            db.flush().expect("flush");
        }

        let r = open_retry(
            &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
        );
        assert!(
            matches!(
                r,
                Err(LayerError::Store(StoreError::IntegrityFailure {
                    what: "conservacion del suministro"
                }))
            ),
            "CRITICO: un suministro que no cuadra con los saldos debe pararse ANTES de operar"
        );

        quitar_libro(&path);
    }

    /// §387 — la raiz del arbol de pendientes se guarda y se comprueba.
    ///
    /// Se planta una hoja `pend:` que ninguna operacion creo. Antes del
    /// §387 el arranque la reconstruia y la creia; ahora la raiz guardada
    /// no cuadra y el arranque se detiene con su nombre.
    #[test]
    fn a_tampered_pending_leaf_is_detected_at_startup() {
        let path = temp_path("pend_mentiroso");

        {
            let mut layer = open_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
            )
            .expect("abrir");
            open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        }

        {
            let db = sled_open_retry(&path);
            assert!(
                db.get(b"root:pending").expect("leer").is_some(),
                "un ledger del §387 guarda la raiz del pendiente"
            );
            let fake: Digest = [BaseElement::new(0xBADBAD); 4];
            let mut key = b"pend:".to_vec();
            key.extend_from_slice(&0u64.to_le_bytes());
            db.insert(key, store::digest_to_bytes(&fake).to_vec())
                .expect("insertar");
            db.flush().expect("flush");
        }

        let r = open_retry(
            &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
        );
        assert!(
            matches!(
                r,
                Err(LayerError::Store(StoreError::IntegrityFailure {
                    what: "arbol de pendientes"
                }))
            ),
            "CRITICO: una hoja de pendiente que nadie creo debe pararse ANTES de operar"
        );

        quitar_libro(&path);
    }

    /// §387 — un libro sin `root:pending` no abre.
    ///
    /// Es el caso de un ledger anterior al §387: fail-closed, sin era
    /// silenciosa. `need` lo dice con su nombre, igual que con `root:state`.
    #[test]
    fn a_ledger_without_a_pending_root_does_not_open() {
        let path = temp_path("sin_raiz_pendiente");

        {
            let mut layer = open_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
            )
            .expect("abrir");
            open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        }

        {
            let db = sled_open_retry(&path);
            assert!(
                db.remove(b"root:pending").expect("borrar").is_some(),
                "habia una raiz del pendiente que borrar"
            );
            db.flush().expect("flush");
        }

        let r = open_retry(
            &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
        );
        assert!(
            matches!(
                r,
                Err(LayerError::Store(StoreError::Malformed(ref m))) if m.contains("root:pending")
            ),
            "CRITICO: un libro sin raiz del pendiente no debe abrir"
        );

        quitar_libro(&path);
    }

    /// §387 — un importe `pamt:` mentido A SOLAS lo caza la conservacion.
    ///
    /// La raiz del §387 cubre las hojas (`pend:`), no los importes: la hoja
    /// es H(H(receptor, sal), importe) y la capa no guarda receptor ni sal.
    /// Este test documenta el residuo: mentir `pamt:` a solas rompe la
    /// conservacion del §379 y se para ahi; mentir `pamt:` y `meta:supply`
    /// a la vez sigue sin falsador (candidata declarada en el asiento §387).
    #[test]
    fn a_lying_pending_amount_is_detected_at_startup() {
        let path = temp_path("pamt_mentiroso");

        {
            let mut layer = open_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
            )
            .expect("abrir");
            open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        }

        {
            let db = sled_open_retry(&path);
            let mut key = b"pamt:".to_vec();
            key.extend_from_slice(&0u64.to_le_bytes());
            db.insert(key, 5u64.to_le_bytes().to_vec()).expect("insertar");
            db.flush().expect("flush");
        }

        let r = open_retry(
            &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
        );
        assert!(
            matches!(
                r,
                Err(LayerError::Store(StoreError::IntegrityFailure {
                    what: "conservacion del suministro"
                }))
            ),
            "CRITICO: un importe de pendiente mentido debe pararse ANTES de operar"
        );

        quitar_libro(&path);
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

        quitar_libro(&path);
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

        quitar_libro(&path);
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
        l.records.values().map(|r| r.balance).sum()
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

    quitar_libro(&path);

}

// ===================================================================
// TESTIGOS DEL S487 (5.A-264): con la no-congelacion impuesta por el
// aplicador, el ataque del camino ajeno pasa de Ok(()) a AccountFrozen.
// Fabrican la prueba a mano (via honesta) con el camino de congelados de
// OTRA posicion libre -- el mismo ataque que midio la SONDA-264 -- y
// exigen que el apply lo RECHACE. Sin el arreglo, estos tres pasarian a
// Ok y el test caeria: son el falsador del arreglo.
// ===================================================================

fn t487_clave(sk: u64) -> Digest {
    [BaseElement::new(sk), BaseElement::ZERO, BaseElement::ZERO, BaseElement::ZERO]
}

fn t487_sal(layer: &SovereignLayer, idx: AccountIndex) -> Digest {
    layer
        .records
        .get(&idx)
        .map(|r| r.leaf_salt)
        .unwrap_or(crate::store::LEAF_SALT_LEGACY)
}

/// **5.A-264 / S487, ENVIO.** Una cuenta congelada NO envia reciclando el
/// camino de congelados de una posicion libre vecina.
#[test]
fn t487_una_congelada_no_envia_con_camino_ajeno() {
    let mut layer = new_layer();
    let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
    set_frozen_delegated(&mut layer, alice, true);
    let vecino = alice ^ 1;
    assert!(layer.is_frozen(alice) && !layer.is_frozen(vecino));

    let estado = state_of(&layer, alice);
    let destino = derive_public_id(BaseElement::new(SK_BOB));
    let sal = salt_de(0x487);
    let importe: u64 = 1000;
    let position = layer.reserve_pending().expect("reserva");
    let trace = stark_experiment::circuit_send::build_trace(
        t487_clave(SK_ALICE),
        estado.public_id,
        estado.balance,
        estado.nonce,
        t487_sal(&layer, alice),
        &layer.accounts.path_for(alice),
        &layer.frozen.path_for(vecino),
        importe,
        layer.regulatory_limit,
        layer.total_supply,
        0,
        destino,
        sal,
        &layer.pending.path_for(position),
    );
    let prover = stark_experiment::circuit_send::SendProver::new(layer.options.clone());
    let public_inputs = prover.get_pub_inputs(&trace);
    // E3b-2 (RFC-0009 D-AH): encendido, el probador COMPRUEBA la traza real antes de
    // ocultarla y el camino ajeno no la satisface: devuelve Err. Apagado, winterfell
    // emitia la prueba y la capa la rechazaba en el aplicador (S487). Las DOS puertas se
    // prueban: la del probador aqui, y la del aplicador con un recibo SIN prueba, que la
    // congelacion rechaza antes de leerla.
    let err = prover.prove(trace).expect_err("un camino ajeno no satisface el AIR (D-AH)");
    let que = format!("{err:?}");
    assert!(que.starts_with("UnsatisfiedTransitionConstraintError"), "otro error: {que}");
    let recibo = crate::two_phase::SendReceipt {
        proof: Vec::new(),
        public_inputs,
        commitment: crate::pending::pending_commitment(destino, sal, importe),
        notice: crate::two_phase::PendingNotice {
            position,
            salt: sal,
            amount: importe,
            x: None,
        },
    };
    let r = layer.apply_send(&recibo, alice, &estado, importe);
    assert!(
        matches!(r, Err(LayerError::AccountFrozen(_))),
        "S487: una congelada no debe enviar con camino ajeno: {r:?}"
    );
    assert_eq!(layer.balance_of(alice), Some(1_000_000), "el saldo no se movio");
}

/// **5.A-264 / S487, COBRO.**
#[test]
fn t487_una_congelada_no_cobra_con_camino_ajeno() {
    let mut layer = new_layer();
    let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
    // Un pendiente a nombre de alice, hecho por bob (via honesta).
    let bob = open_and_fund(&mut layer, SK_BOB, 1_000_000);
    let estado_bob = state_of(&layer, bob);
    let id_alice = layer.public_id_of(alice).expect("cuenta");
    let envio = layer
        .send(BaseElement::new(SK_BOB), bob, &estado_bob, id_alice, salt_de(0x4871), 1000)
        .expect("enviar");
    layer.apply_send(&envio, bob, &estado_bob, 1000).expect("aplicar el envio");
    let aviso = envio.notice;

    set_frozen_delegated(&mut layer, alice, true);
    let vecino = alice ^ 1;
    assert!(layer.is_frozen(alice) && !layer.is_frozen(vecino));

    let estado = state_of(&layer, alice);
    let trace = stark_experiment::circuit_claim::build_trace(
        t487_clave(SK_ALICE),
        estado.public_id,
        estado.balance,
        estado.nonce,
        t487_sal(&layer, alice),
        &layer.accounts.path_for(alice),
        &layer.frozen.path_for(vecino),
        aviso.amount,
        layer.total_supply,
        0,
        estado.public_id,
        aviso.salt,
        &layer.pending.path_for(aviso.position),
    );
    let prover = stark_experiment::circuit_claim::ClaimProver::new(layer.options.clone());
    let public_inputs = prover.get_pub_inputs(&trace);
    // E3b-2 (D-AH): el probador devuelve Err con el camino ajeno; el aplicador rechaza
    // la congelada antes de leer la prueba (S487).
    let err = prover.prove(trace).expect_err("un camino ajeno no satisface el AIR (D-AH)");
    let que = format!("{err:?}");
    assert!(que.starts_with("UnsatisfiedTransitionConstraintError"), "otro error: {que}");
    let recibo = crate::two_phase::ClaimReceipt {
        proof: Vec::new(),
        public_inputs,
    };
    let r = layer.apply_claim(&recibo, alice, &estado, &aviso);
    assert!(
        matches!(r, Err(LayerError::AccountFrozen(_))),
        "S487: una congelada no debe cobrar con camino ajeno: {r:?}"
    );
    assert_eq!(layer.balance_of(alice), Some(1_000_000), "el saldo no subio");
}

/// **5.A-264 / S487, DESTRUCCION.**
#[test]
fn t487_una_congelada_no_quema_con_camino_ajeno() {
    let mut layer = new_layer();
    let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
    set_frozen_delegated(&mut layer, alice, true);
    let vecino = alice ^ 1;
    assert!(layer.is_frozen(alice) && !layer.is_frozen(vecino));

    let estado = state_of(&layer, alice);
    let importe: u64 = 1000;
    let trace = stark_experiment::circuit_burn::build_trace(
        t487_clave(SK_ALICE),
        estado.public_id,
        estado.balance,
        estado.nonce,
        t487_sal(&layer, alice),
        &layer.accounts.path_for(alice),
        &layer.frozen.path_for(vecino),
        importe,
        layer.total_supply,
        importe,
    );
    let prover = stark_experiment::circuit_burn::BurnProver::new(layer.options.clone());
    let public_inputs = prover.get_pub_inputs(&trace);
    // E3b-2 (D-AH): el probador devuelve Err con el camino ajeno; el aplicador rechaza
    // la congelada antes de leer la prueba (S487).
    let err = prover.prove(trace).expect_err("un camino ajeno no satisface el AIR (D-AH)");
    let que = format!("{err:?}");
    assert!(que.starts_with("UnsatisfiedTransitionConstraintError"), "otro error: {que}");
    let recibo = crate::BurnReceipt {
        proof: Vec::new(),
        public_inputs,
    };
    let r = layer.apply_burn(&recibo, alice, &estado);
    assert!(
        matches!(r, Err(LayerError::AccountFrozen(_))),
        "S487: una congelada no debe quemar con camino ajeno: {r:?}"
    );
    assert_eq!(layer.balance_of(alice), Some(1_000_000), "el saldo no se destruyo");
    assert_eq!(layer.total_supply(), 1_000_000, "el suministro no bajo");
}

/// §668: el tope de suministro de la capa es `2^62 - 1`, el rango de los AIR. Con uno mayor
/// -antes valia hasta `u64::MAX`- una suma de saldo o de suministro podia dar la vuelta en `u64`.
/// `new` lo rechaza con un panico al arrancar; `open` con un error. Falsador: sin la
/// comprobacion, `new` arranca y `open` abre.
#[test]
#[should_panic(expected = "por encima del tope de la capa")]
fn un_tope_de_suministro_por_encima_de_2_a_la_62_no_arranca() {
    let _ = SovereignLayer::new(custodian_root(), governance_root(), LIMIT, MAX_SUMINISTRO + 1, MAX_ACCOUNTS);
}

#[test]
fn un_tope_de_suministro_por_encima_de_2_a_la_62_no_abre() {
    let ruta = temp_path("tope_suministro_668");
    let r = SovereignLayer::open(&ruta, custodian_root(), governance_root(), LIMIT, u64::MAX, MAX_ACCOUNTS);
    assert!(
        matches!(r, Err(LayerError::SupplyCapExceeded { cap: MAX_SUMINISTRO, would_be: u64::MAX })),
        "{:?}",
        r.as_ref().err()
    );
    let ok = SovereignLayer::open(&ruta, custodian_root(), governance_root(), LIMIT, MAX_SUMINISTRO, MAX_ACCOUNTS);
    assert!(ok.is_ok(), "el tope maximo si abre");
    quitar_libro(&ruta);
}

// -----------------------------------------------------------------
// Claves anchas de custodios y de gobernanza (RFC-0018 E2, §701)
// -----------------------------------------------------------------

/// **Un libro constituido con claves de cuatro elementos ejerce las cinco
/// autoridades** (RFC-0018 E2, §701): emitir, emitir a un pendiente,
/// recuperar, congelar y, con la gobernanza, rotar los custodios a otro
/// conjunto ancho, desde el que emiten los entrantes. Cada clave tiene sus
/// cuatro elementos no nulos: con [`custodian_keys`] el camino es el mismo y
/// no ejercita los 256 bits.
#[test]
fn un_libro_de_claves_anchas_ejerce_las_cinco_autoridades() {
    let ck = custodian_keys_wide();
    let gk = governance_keys_wide();
    let mut layer = new_layer_wide();
    assert_ne!(layer.custodian_set_root(), custodian_root(), "otro conjunto de custodios");
    assert_ne!(layer.governance_set_root(), governance_root(), "otra gobernanza");
    let alice = layer.open_account(BaseElement::new(SK_ALICE));
    let bob = layer.open_account(BaseElement::new(SK_BOB));

    // 1. Emitir.
    let op = mint_commitment(&layer, alice, 1000);
    let subida = mint_climb_proof(&layer, alice, 1000);
    let (pa, ia, pb, ib) = custodian_pair_wide_with(&ck, op, 1, 3);
    layer
        .apply_mint_delegated(subida, pa, ia, pb, ib, alice, 1000)
        .expect("dos custodios de clave ancha emiten");
    assert_eq!(layer.balance_of(alice), Some(1000));

    // 2. Emitir a un pendiente de Bob.
    let id_bob = layer.public_id_of(bob).expect("cuenta");
    let op = mint_pending_commitment(&layer, id_bob, salt_de(0x701), 500);
    let subida = mint_pending_climb_proof(&layer, id_bob, salt_de(0x701), 500);
    let (pa, ia, pb, ib) = custodian_pair_wide_with(&ck, op, 0, 4);
    layer
        .apply_mint_pending_delegated(subida, pa, ia, pb, ib, id_bob, salt_de(0x701), 500)
        .expect("y emiten a un pendiente");
    assert_eq!(layer.total_supply(), 1500);

    // 3. Recuperar la cuenta de Alice hacia una identidad nueva.
    let nueva_id = derive_public_id_wide(wide_key(0xA11CE_2));
    let op = recovery_commitment(&layer, alice, nueva_id);
    let subida = recovery_climb_proof(&layer, alice, nueva_id);
    let (pa, ia, pb, ib) = custodian_pair_wide_with(&ck, op, 2, 4);
    layer
        .apply_recovery_delegated(subida, pa, ia, pb, ib, alice, nueva_id)
        .expect("y recuperan");
    assert_eq!(layer.public_id_of(alice), Some(nueva_id));
    assert_eq!(layer.balance_of(alice), Some(1000), "la recuperacion no mueve dinero");

    // 4. Congelar a Bob.
    let op = freeze_commitment(&layer, bob, true);
    let subida = freeze_climb_proof(&layer, bob, true);
    let (pa, ia, pb, ib) = custodian_pair_wide_with(&ck, op, 0, 1);
    layer
        .apply_freeze_delegated(subida, pa, ia, pb, ib, bob, true)
        .expect("y congelan");
    assert!(layer.is_frozen(bob));

    // 5. La gobernanza ancha rota los custodios a otro conjunto ancho...
    let entrantes: Vec<Digest> = (0..5).map(|i| wide_key(0xD0_0D_00 + i)).collect();
    let raiz_entrante =
        stark_experiment::circuit_threshold::build_custodian_set_wide(&entrantes).0;
    let op = governance_commitment(&layer, raiz_entrante);
    let (pa, ia, pb, ib) = governance_pair_wide_with(&gk, op, 0, 2);
    layer
        .apply_governance_delegated(pa, ia, pb, ib, raiz_entrante)
        .expect("dos miembros de clave ancha rotan los custodios");
    assert_eq!(layer.custodian_set_root(), raiz_entrante);

    // ...y los entrantes emiten.
    let op = mint_commitment(&layer, alice, 2000);
    let subida = mint_climb_proof(&layer, alice, 2000);
    let (pa, ia, pb, ib) = custodian_pair_wide_with(&entrantes, op, 1, 3);
    layer
        .apply_mint_delegated(subida, pa, ia, pb, ib, alice, 2000)
        .expect("los custodios entrantes emiten");
    assert_eq!(layer.balance_of(alice), Some(3000));
    assert_eq!(layer.total_supply(), 3500);
}

/// ⚠️ **FALSADOR: sobre un conjunto ancho, una clave de un solo elemento no
/// autoriza** (RFC-0018 E2, §701; `SECURITY.md` 3.10). Es el ataque que esa
/// sección declaraba: agotar los 64 bits de un elemento contra la identidad
/// pública. Una clave de un solo elemento es lo único que alcanza esa
/// búsqueda; aquí el atacante tiene incluso más: el primer elemento verdadero
/// de las claves de dos custodios y el camino de cada uno en el conjunto. Con
/// los otros tres a cero la identidad es otra y la subida no
/// llega a la raíz de la capa (`NotTheIssuer`); con la raíz de la capa puesta
/// a mano en las entradas, la prueba no verifica. Lo mismo en la gobernanza.
/// Con las claves enteras, las dos operaciones se aplican: el rechazo es por
/// la clave y no por el montaje.
#[test]
fn una_clave_de_un_solo_elemento_no_autoriza_sobre_un_conjunto_ancho() {
    use stark_experiment::circuit_governance::{build_governance_set_wide, GOVERNANCE_DOMAIN};
    use stark_experiment::circuit_threshold::{build_custodian_set_wide, CUSTODIAN_DOMAIN};
    let z = BaseElement::ZERO;
    let solo_el_primero = |k: &Digest| -> Digest { [k[0], z, z, z] };

    let ck = custodian_keys_wide();
    let gk = governance_keys_wide();
    let (_, cp) = build_custodian_set_wide(&ck);
    let (_, gp) = build_governance_set_wide(&gk);
    let mut layer = new_layer_wide();
    let alice = layer.open_account(BaseElement::new(SK_ALICE));

    // --- Custodios: emitir con el primer elemento de dos claves.
    let op = mint_commitment(&layer, alice, 1000);
    let subida = mint_climb_proof(&layer, alice, 1000);
    let (pa, ia) = autorizacion_ancha(CUSTODIAN_DOMAIN, solo_el_primero(&ck[1]), &cp[1], op);
    let (pb, ib) = autorizacion_ancha(CUSTODIAN_DOMAIN, solo_el_primero(&ck[3]), &cp[3], op);
    assert_ne!(ia.custodian_set_root, layer.custodian_set_root(), "la subida va a otra raiz");
    let r = layer.apply_mint_delegated(
        subida.clone(), pa.clone(), ia.clone(), pb.clone(), ib.clone(), alice, 1000,
    );
    assert!(
        matches!(r, Err(LayerError::NotTheIssuer)),
        "CRITICO: un elemento de la clave de un custodio ancho emite: {r:?}"
    );
    let (mut ia2, mut ib2) = (ia, ib);
    ia2.custodian_set_root = layer.custodian_set_root();
    ib2.custodian_set_root = layer.custodian_set_root();
    let r = layer.apply_mint_delegated(subida.clone(), pa, ia2, pb, ib2, alice, 1000);
    assert!(
        matches!(r, Err(LayerError::VerificationFailed(_))),
        "CRITICO: con la raiz de la capa puesta a mano, la prueba verifica: {r:?}"
    );
    assert_eq!(layer.total_supply(), 0, "no se emitio nada");

    // La clave entera, con los mismos materiales: emite.
    let (pa, ia, pb, ib) = custodian_pair_wide_with(&ck, op, 1, 3);
    layer
        .apply_mint_delegated(subida, pa, ia, pb, ib, alice, 1000)
        .expect("con la clave entera, los mismos custodios emiten");
    assert_eq!(layer.total_supply(), 1000);

    // --- Gobernanza: rotar los custodios con el primer elemento de dos claves.
    let entrantes: Vec<Digest> = (0..5).map(|i| wide_key(0xD0_0D_00 + i)).collect();
    let raiz_entrante = build_custodian_set_wide(&entrantes).0;
    let saliente = layer.custodian_set_root();
    let op = governance_commitment(&layer, raiz_entrante);
    let (pa, ia) = autorizacion_ancha(GOVERNANCE_DOMAIN, solo_el_primero(&gk[0]), &gp[0], op);
    let (pb, ib) = autorizacion_ancha(GOVERNANCE_DOMAIN, solo_el_primero(&gk[2]), &gp[2], op);
    let r = layer.apply_governance_delegated(
        pa.clone(), ia.clone(), pb.clone(), ib.clone(), raiz_entrante,
    );
    assert!(
        matches!(r, Err(LayerError::NotTheIssuer)),
        "CRITICO: un elemento de la clave de un gobernador ancho rota los custodios: {r:?}"
    );
    let (mut ia2, mut ib2) = (ia, ib);
    ia2.custodian_set_root = layer.governance_set_root();
    ib2.custodian_set_root = layer.governance_set_root();
    let r = layer.apply_governance_delegated(pa, ia2, pb, ib2, raiz_entrante);
    assert!(
        matches!(r, Err(LayerError::VerificationFailed(_))),
        "CRITICO: con la raiz de la gobernanza puesta a mano, la prueba verifica: {r:?}"
    );
    assert_eq!(layer.custodian_set_root(), saliente, "los custodios no cambiaron");
    assert_eq!(layer.governance_change_count(), 0);

    // La clave entera: rota.
    let (pa, ia, pb, ib) = governance_pair_wide_with(&gk, op, 0, 2);
    layer
        .apply_governance_delegated(pa, ia, pb, ib, raiz_entrante)
        .expect("con la clave entera, la gobernanza rota");
    assert_eq!(layer.custodian_set_root(), raiz_entrante);
}

/// **Un libro de claves estrechas rota sus custodios a claves anchas**
/// (RFC-0018, la corrección del §683, en la capa: §701). La gobernanza es
/// inmutable y sigue siendo la de un elemento: firma con su clave rellenada y
/// lleva el conjunto de custodios a uno de claves anchas. Desde ahí emiten los
/// entrantes, con la clave entera, y no los salientes. Lo que sólo da un libro
/// nuevo es la gobernanza ancha.
#[test]
fn un_libro_de_claves_estrechas_rota_sus_custodios_a_claves_anchas() {
    let mut layer = new_layer();
    let alice = open_and_fund(&mut layer, SK_ALICE, 1000);
    let gobernanza = layer.governance_set_root();
    assert_eq!(gobernanza, governance_root(), "la gobernanza de la suite, estrecha");

    // La gobernanza estrecha, por la delegada, lleva los custodios a claves anchas.
    let entrantes = custodian_keys_wide();
    update_custodians_delegated(&mut layer, custodian_root_wide());
    assert_eq!(layer.custodian_set_root(), custodian_root_wide());
    assert_eq!(layer.governance_set_root(), gobernanza, "la gobernanza no cambia");

    // Los salientes, de un elemento, ya no emiten.
    let op = mint_commitment(&layer, alice, 1000);
    let subida = mint_climb_proof(&layer, alice, 1000);
    let (pa, ia, pb, ib) = delegated_pair(op, 1, 3);
    let r = layer.apply_mint_delegated(subida.clone(), pa, ia, pb, ib, alice, 1000);
    assert!(
        matches!(r, Err(LayerError::NotTheIssuer)),
        "CRITICO: los custodios salientes emiten tras la rotacion: {r:?}"
    );

    // Los entrantes, con la clave entera, sí.
    let (pa, ia, pb, ib) = custodian_pair_wide_with(&entrantes, op, 1, 3);
    layer
        .apply_mint_delegated(subida, pa, ia, pb, ib, alice, 1000)
        .expect("los custodios de clave ancha emiten en un libro de gobernanza estrecha");
    assert_eq!(layer.balance_of(alice), Some(2000));
}

// -----------------------------------------------------------------
// §702 (RFC-0001): el KDF del libro cifrado, con su version
// -----------------------------------------------------------------
// Al final del fichero, como los del §701: ninguna cita `tests.rs:linea` se mueve.

/// Lleva un libro cifrado de la version 2 a la 1, como lo habria dejado el codigo de antes del
/// §702: cada valor sellado se abre con su clave y se vuelve a sellar con la de SHA-256 sin sal,
/// y la cabecera se borra. Existe porque este codigo ya no ESCRIBE la version 1: la unica forma de
/// probar aqui que un libro viejo abre es fabricar sus bytes con la regla documentada, como
/// `v3_snapshot_bytes` en las instantaneas. Un libro escrito por el codigo de `590caae` se abrio
/// y se migro, fuera del arbol, en el §702.
fn rebajar_a_v1(path: &str, frase: &str) {
    use crate::crypto::{Kdf, LedgerKey};
    use crate::persistence::{CLAVE_KDF, EN_CLARO};
    let db = sled_open_retry(path);
    let kdf = Kdf::de_cabecera(&db.get(CLAVE_KDF).unwrap().expect("un libro v2 lleva cabecera"))
        .expect("cabecera v2");
    let v2 = LedgerKey::derivar(frase, &kdf).unwrap();
    let v1 = LedgerKey::derivar(frase, &Kdf::Sha256V1).unwrap();
    let mut lote = sled::Batch::default();
    for item in db.iter() {
        let (k, v) = item.unwrap();
        if EN_CLARO.contains(&k.as_ref()) {
            continue;
        }
        lote.insert(k, v1.seal(&v2.open(&v).expect("abre con la v2")).unwrap());
    }
    lote.remove(CLAVE_KDF);
    db.apply_batch(lote).unwrap();
    db.flush().unwrap();
}

/// Todas las parejas del `sled` del libro, para comparar un antes y un despues.
fn volcado(path: &str) -> Vec<(Vec<u8>, Vec<u8>)> {
    let db = sled_open_retry(path);
    db.iter().map(|i| i.unwrap()).map(|(k, v)| (k.to_vec(), v.to_vec())).collect()
}

/// **§702: un libro cifrado de la version 1 abre con su frase y queda en la 2.** Antes de
/// abrirlo no tiene cabecera y sus valores abren con la clave de SHA-256; despues tiene la
/// cabecera de la version 2, ningun valor sellado abre ya con la clave vieja y todos abren con la
/// nueva; y se reabre por la version 2 con el mismo saldo.
#[test]
fn un_libro_cifrado_de_la_version_1_se_migra_al_abrirlo() {
    use crate::crypto::{Kdf, LedgerKey};
    use crate::persistence::{CLAVE_KDF, EN_CLARO};
    let path = temp_path("kdf_v1_migra");
    let frase = "la frase del libro";
    let abrir = |f: &str| {
        open_encrypted_retry(
            &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
            Some(LedgerKey::from_passphrase(f)),
        )
    };
    let alice = {
        let mut layer = abrir(frase).expect("abrir cifrado");
        open_and_fund(&mut layer, SK_ALICE, 777_777)
    };
    rebajar_a_v1(&path, frase);
    let v1 = LedgerKey::derivar(frase, &Kdf::Sha256V1).unwrap();
    {
        let db = sled_open_retry(&path);
        assert!(db.get(CLAVE_KDF).unwrap().is_none(), "la v1 no lleva cabecera");
        assert!(v1.open(&db.get(b"meta:supply").unwrap().unwrap()).is_ok(), "no es la v1");
    }

    {
        let layer = abrir(frase).expect("la v1 abre con su frase");
        assert_eq!(layer.balance_of(alice), Some(777_777));
    }

    let kdf = {
        let db = sled_open_retry(&path);
        let c = db.get(CLAVE_KDF).unwrap().expect("la migracion escribe la cabecera");
        Kdf::de_cabecera(&c).expect("cabecera v2")
    };
    let v2 = LedgerKey::derivar(frase, &kdf).unwrap();
    let mut sellados = 0;
    for (k, v) in volcado(&path) {
        if EN_CLARO.contains(&k.as_slice()) {
            continue;
        }
        let nombre = String::from_utf8_lossy(&k).into_owned();
        assert!(v1.open(&v).is_err(), "CRITICO: {nombre} sigue sellado con la clave v1");
        assert!(v2.open(&v).is_ok(), "{nombre} no abre con la clave v2");
        sellados += 1;
    }
    assert!(sellados >= 20, "la migracion tenia que tocar el libro entero: {sellados}");

    let layer = abrir(frase).expect("reabre en la version 2");
    assert_eq!(layer.balance_of(alice), Some(777_777));
    drop(layer);
    quitar_libro(&path);
}

/// **§702: con otra frase, un libro de la version 1 no abre y NO SE TOCA**: el `sled` queda
/// pareja a pareja como estaba, sin cabecera. La migracion solo empieza despues de que `load`
/// verifique el libro entero con la clave vieja.
#[test]
fn un_libro_de_la_version_1_con_otra_frase_no_se_toca() {
    use crate::crypto::LedgerKey;
    let path = temp_path("kdf_v1_otra");
    {
        let mut layer = open_encrypted_retry(
            &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
            Some(LedgerKey::from_passphrase("la buena")),
        )
        .expect("abrir cifrado");
        open_and_fund(&mut layer, SK_ALICE, 1_000);
    }
    rebajar_a_v1(&path, "la buena");
    let antes = volcado(&path);
    let r = open_encrypted_retry(
        &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
        Some(LedgerKey::from_passphrase("la mala")),
    );
    assert!(r.is_err(), "CRITICO: otra frase abrio un libro de la version 1");
    drop(r);
    assert_eq!(volcado(&path), antes, "CRITICO: una frase mala toco el libro de la version 1");
    quitar_libro(&path);
}

/// **§702: la cabecera va en claro, cada libro tiene su sal, y sin frase no se abre en claro.**
/// Dos libros con la misma frase llevan dos sales; la cabecera es la de la version 2, con su
/// coste; y abrir uno sin clave es `ParameterMismatch` de «cifrado en reposo», no un error de
/// formato del primer valor sellado.
#[test]
fn cada_libro_cifrado_lleva_su_cabecera_y_su_sal() {
    use crate::crypto::{Kdf, LedgerKey, COSTE_V2};
    use crate::persistence::CLAVE_KDF;
    let (a, b) = (temp_path("kdf_sal_a"), temp_path("kdf_sal_b"));
    for p in [&a, &b] {
        let layer = open_encrypted_retry(
            p, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
            Some(LedgerKey::from_passphrase("la misma frase")),
        )
        .expect("abrir cifrado");
        drop(layer);
    }
    let cabecera = |p: &str| sled_open_retry(p).get(CLAVE_KDF).unwrap().expect("cabecera").to_vec();
    let (ca, cb) = (cabecera(&a), cabecera(&b));
    assert_eq!(ca.len(), 29);
    assert_eq!(ca[0], 2, "version");
    assert_eq!(ca[1..5], COSTE_V2.m_kib.to_le_bytes());
    assert_eq!(ca[5..9], COSTE_V2.t.to_le_bytes());
    assert_eq!(ca[9..13], COSTE_V2.p.to_le_bytes());
    assert_ne!(ca[13..], cb[13..], "dos libros con la misma sal");
    assert!(matches!(Kdf::de_cabecera(&ca), Ok(Kdf::Argon2idV2 { .. })));

    let r = open_encrypted_retry(
        &a, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS, None,
    );
    assert!(
        matches!(r, Err(LayerError::Store(StoreError::ParameterMismatch { what: "cifrado en reposo" }))),
        "un libro cifrado sin su frase: {:?}",
        r.err()
    );
    quitar_libro(&a);
    quitar_libro(&b);
}

/// **§704: el montaje comparte sus pruebas, y nada más.** Dentro de `de_montaje`, la misma subida
/// sobre el mismo libro sale con los mismos bytes, y otra subida —otro importe— con otra prueba;
/// la compartida la aplica la capa, que la verifica entera, en dos libros distintos. Fuera, el
/// mismo ayudante prueba de nuevo, y dos pruebas del mismo enunciado no comparten bytes: es lo
/// que necesitan los tests que llaman a los ayudantes a mano para medir o para repetir una prueba.
/// Y el hilo sale del montaje también cuando dentro hay un pánico.
#[test]
fn el_montaje_comparte_sus_pruebas_y_nada_mas() {
    const SK: u64 = 0x703_0001;
    const IMPORTE: u64 = 1_703;
    let mut l = new_layer();
    let a = open_and_fund(&mut l, SK, 0);

    let dentro = de_montaje(|| mint_climb_proof(&l, a, IMPORTE)).to_bytes();
    let otra_vez = de_montaje(|| mint_climb_proof(&l, a, IMPORTE)).to_bytes();
    assert_eq!(dentro, otra_vez, "dentro del montaje, el mismo enunciado se probo dos veces");
    let otro_importe = de_montaje(|| mint_climb_proof(&l, a, IMPORTE + 1)).to_bytes();
    assert_ne!(dentro, otro_importe, "dos enunciados distintos salieron con la misma prueba");

    let fuera = mint_climb_proof(&l, a, IMPORTE).to_bytes();
    let fuera_otra = mint_climb_proof(&l, a, IMPORTE).to_bytes();
    assert_ne!(fuera, fuera_otra, "fuera del montaje se reutilizo una prueba");
    assert_ne!(fuera, dentro, "fuera del montaje salio la prueba compartida");

    let r = std::panic::catch_unwind(|| {
        de_montaje(|| panic!("§704: un panico dentro del montaje, a proposito"))
    });
    assert!(r.is_err());
    let tras_el_panico = mint_climb_proof(&l, a, IMPORTE).to_bytes();
    assert_ne!(tras_el_panico, dentro, "tras un panico, el hilo siguio en el montaje");

    // La compartida se aplica: `fund_delegated` pide esta misma subida dentro del montaje.
    fund_delegated(&mut l, a, IMPORTE);
    assert_eq!(l.balance_of(a), Some(IMPORTE));
    let mut otro_libro = new_layer();
    let b = open_and_fund(&mut otro_libro, SK, 0);
    fund_delegated(&mut otro_libro, b, IMPORTE);
    assert_eq!(otro_libro.balance_of(b), Some(IMPORTE));
    assert_eq!(otro_libro.total_supply(), IMPORTE);
}

// -----------------------------------------------------------------
// §802: las identidades retiradas, en reposo
// -----------------------------------------------------------------
// Al final del fichero, como los del §702: ninguna cita `tests.rs:linea` se mueve.

/// La identidad de la clave estrecha `sk`.
fn id_de(sk: u64) -> Digest {
    derive_public_id(BaseElement::new(sk))
}

/// Intenta abrir `id` con una vista y una sal ajenas: lo que haria quien solo la conoce.
fn abrir_ajena(l: &mut SovereignLayer, id: Digest) -> Result<AccountIndex, LayerError> {
    l.open_with_id(id, [BaseElement::new(9); 4], [BaseElement::new(7); 4])
}

/// La clave en sled de la identidad retirada de ordinal `k`.
fn clave_retir(k: u64) -> Vec<u8> {
    let mut v = b"retir:".to_vec();
    v.extend_from_slice(&k.to_le_bytes());
    v
}

/// Un libro en `path` con Alice recuperada de X hacia Y: una retirada en disco.
fn libro_con_una_recuperacion(path: &str, key: Option<crate::crypto::LedgerKey>) {
    let mut l = open_encrypted_retry(
        path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS, key,
    )
    .expect("abrir");
    let alice = open_and_fund(&mut l, SK_ALICE, 1_000);
    recover_delegated(&mut l, alice, id_de(0xA11CE_2));
}

/// Es el `IntegrityFailure` de las identidades retiradas, y no otro.
fn es_de_las_retiradas<T>(r: &Result<T, LayerError>) -> bool {
    matches!(
        r,
        Err(LayerError::Store(StoreError::IntegrityFailure { what: "identidades retiradas" }))
    )
}

/// **§802: la retirada sobrevive al reinicio, y un rechazo no escribe.** Libro cifrado con X->Y,
/// cerrado y reabierto: ni X ni Y abren otra cuenta. El `sled` queda pareja a pareja como
/// estaba tras los dos rechazos -cada sello lleva un nonce aleatorio, asi que cualquier `commit`
/// cambiaria bytes-, y el libro vuelve a abrir.
#[test]
fn la_retirada_sobrevive_al_reinicio_y_un_rechazo_no_escribe() {
    use crate::crypto::LedgerKey;
    let path = temp_path("retir_reinicio");
    let frase = "la frase del libro de las retiradas";
    let abrir = || {
        open_encrypted_retry(
            &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
            Some(LedgerKey::from_passphrase(frase)),
        )
    };
    libro_con_una_recuperacion(&path, Some(LedgerKey::from_passphrase(frase)));
    let antes = volcado(&path);
    {
        let mut l = abrir().expect("CRITICO: el libro con una recuperacion no reabre");
        for (nombre, id) in [("X, la retirada", id_de(SK_ALICE)), ("Y, la recuperada", id_de(0xA11CE_2))] {
            let r = abrir_ajena(&mut l, id);
            assert!(
                matches!(r, Err(LayerError::IdentityUnavailable)),
                "CRITICO: tras reiniciar, {nombre} abrio otra cuenta: {r:?}"
            );
        }
    }
    assert_eq!(volcado(&path), antes, "CRITICO: un rechazo escribio en el libro");
    let l = abrir().expect("y vuelve a abrir");
    assert_eq!(l.recovery_count(), 1);
    drop(l);
    quitar_libro(&path);
}

/// **§802: una retirada borrada en reposo no abre.** Libro en claro con una recuperacion: sin
/// `retir:0` no abre, y tampoco si ademas se reescribe `root:retir` con la cabeza de la lista
/// vacia, que en claro se puede: la lista tiene una por recuperacion del registro.
#[test]
fn una_retirada_borrada_en_reposo_no_abre() {
    let path = temp_path("retir_borrada");
    libro_con_una_recuperacion(&path, None);
    let abrir = || open_retry(&path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS);
    {
        let db = sled_open_retry(&path);
        db.remove(clave_retir(0)).expect("borrar").expect("estaba");
        db.flush().expect("flush");
    }
    let r = abrir();
    assert!(es_de_las_retiradas(&r), "(a) sin retir:0: {:?}", r.as_ref().err());
    drop(r);
    {
        let db = sled_open_retry(&path);
        db.insert(b"root:retir", store::digest_to_bytes(&[BaseElement::ZERO; 4]).to_vec())
            .expect("reescribir la raiz");
        db.flush().expect("flush");
    }
    let r = abrir();
    assert!(
        es_de_las_retiradas(&r),
        "CRITICO: (b) la lista vacia con su raiz vacia abrio un libro con una recuperacion: {:?}",
        r.as_ref().err()
    );
    drop(r);
    quitar_libro(&path);
}

/// **§802: una retirada sustituida en reposo no abre.** El sellado no lleva AAD: el valor sellado
/// de `meta:governance` -32 B, con la misma clave- se pega en `retir:0` sin la frase, y la
/// cuenta seguiria cuadrando. Lo caza `root:retir`, que es la razon de que exista.
#[test]
fn una_retirada_sustituida_en_reposo_no_abre() {
    use crate::crypto::LedgerKey;
    let path = temp_path("retir_sustituida");
    let frase = "la frase del libro de las retiradas";
    libro_con_una_recuperacion(&path, Some(LedgerKey::from_passphrase(frase)));
    {
        let db = sled_open_retry(&path);
        let otra = db.get(b"meta:governance").expect("leer").expect("sellada");
        db.insert(clave_retir(0), otra).expect("pegar");
        db.flush().expect("flush");
    }
    let r = open_encrypted_retry(
        &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
        Some(LedgerKey::from_passphrase(frase)),
    );
    assert!(
        es_de_las_retiradas(&r),
        "CRITICO: un valor sellado de otra clave paso por la identidad retirada: {:?}",
        r.as_ref().err()
    );
    drop(r);
    quitar_libro(&path);
}

/// **§802: sin `root:retir`, un libro abre si no tuvo recuperaciones y no abre si las tuvo.** (a)
/// Un libro anterior al §802, sin recuperaciones, no tiene ninguna retirada: abre, y el indice se
/// rehace de los registros. (b) Con una recuperacion y sin la raiz, no abre aunque `retir:0`
/// siga: sin raiz no hay contra que mirarla, y una sustituida pasaria. (b') Sin sus dos claves
/// tampoco: se ve igual que un libro al que se las han borrado. (c) Sin recuperaciones y con una
/// `retir:` sin raiz, no abre.
#[test]
fn sin_root_retir_abre_si_no_hubo_recuperaciones_y_no_si_las_hubo() {
    let abrir = |p: &str| open_retry(p, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS);
    let quitar = |p: &str, claves: &[Vec<u8>]| {
        let db = sled_open_retry(p);
        for k in claves {
            db.remove(k).expect("borrar").expect("estaba");
        }
        db.flush().expect("flush");
    };

    let a = temp_path("retir_sin_raiz_a");
    {
        let mut l = abrir(&a).expect("abrir");
        open_and_fund(&mut l, SK_ALICE, 1_000);
    }
    quitar(&a, &[b"root:retir".to_vec()]);
    {
        let mut l = abrir(&a).expect("(a) un libro sin recuperaciones y sin root:retir abre");
        let r = abrir_ajena(&mut l, id_de(SK_ALICE));
        assert!(matches!(r, Err(LayerError::IdentityUnavailable)), "(a) el indice: {r:?}");
    }
    quitar_libro(&a);

    let b = temp_path("retir_sin_raiz_b");
    libro_con_una_recuperacion(&b, None);
    quitar(&b, &[b"root:retir".to_vec()]);
    let r = abrir(&b);
    assert!(
        es_de_las_retiradas(&r),
        "CRITICO: (b) un libro con una recuperacion abrio sin la raiz de sus retiradas: {:?}",
        r.as_ref().err()
    );
    drop(r);
    quitar(&b, &[clave_retir(0)]);
    let r = abrir(&b);
    assert!(
        es_de_las_retiradas(&r),
        "CRITICO: (b') un libro con una recuperacion abrio sin sus retiradas: {:?}",
        r.as_ref().err()
    );
    drop(r);
    quitar_libro(&b);

    let c = temp_path("retir_sin_raiz_c");
    {
        let mut l = abrir(&c).expect("abrir");
        open_and_fund(&mut l, SK_ALICE, 1_000);
    }
    quitar(&c, &[b"root:retir".to_vec()]);
    {
        let db = sled_open_retry(&c);
        db.insert(clave_retir(0), store::digest_to_bytes(&id_de(SK_BOB)).to_vec()).expect("plantar");
        db.flush().expect("flush");
    }
    let r = abrir(&c);
    assert!(es_de_las_retiradas(&r), "(c) una retir: sin raiz: {:?}", r.as_ref().err());
    drop(r);
    quitar_libro(&c);
}

/// **§802: la migracion del KDF conserva las retiradas.** Un libro cifrado con una recuperacion,
/// llevado a la version 1 y abierto -que lo migra-: X sigue sin abrir, y `retir:0` y
/// `root:retir` abren con la clave de la version 2 y no con la de la 1.
#[test]
fn la_migracion_del_kdf_conserva_las_retiradas() {
    use crate::crypto::{Kdf, LedgerKey};
    use crate::persistence::CLAVE_KDF;
    let path = temp_path("retir_kdf");
    let frase = "la frase del libro de las retiradas";
    libro_con_una_recuperacion(&path, Some(LedgerKey::from_passphrase(frase)));
    rebajar_a_v1(&path, frase);
    {
        let mut l = open_encrypted_retry(
            &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
            Some(LedgerKey::from_passphrase(frase)),
        )
        .expect("la v1 abre y se migra");
        let r = abrir_ajena(&mut l, id_de(SK_ALICE));
        assert!(
            matches!(r, Err(LayerError::IdentityUnavailable)),
            "CRITICO: tras migrar el KDF, la identidad retirada abrio: {r:?}"
        );
    }
    let kdf = {
        let db = sled_open_retry(&path);
        let c = db.get(CLAVE_KDF).unwrap().expect("la migracion escribe la cabecera");
        Kdf::de_cabecera(&c).expect("cabecera v2")
    };
    let v1 = LedgerKey::derivar(frase, &Kdf::Sha256V1).unwrap();
    let v2 = LedgerKey::derivar(frase, &kdf).unwrap();
    let mut vistas = 0;
    for (k, v) in volcado(&path) {
        if k.starts_with(b"retir:") || k.as_slice() == b"root:retir" {
            let nombre = String::from_utf8_lossy(&k).into_owned();
            assert!(v1.open(&v).is_err(), "CRITICO: {nombre} sigue sellado con la clave v1");
            assert!(v2.open(&v).is_ok(), "{nombre} no abre con la clave v2");
            vistas += 1;
        }
    }
    assert_eq!(vistas, 2, "retir:0 y root:retir");
    quitar_libro(&path);
}

/// **§802: varias retiradas, varios reinicios, en su orden.** Libro cifrado: Alice X->Y, reabrir;
/// Bob B->W, reabrir; Alice Y->Z, reabrir. Tras el ultimo, la lista sale en el orden de las
/// recuperaciones, la cadena de tres eslabones es la de `root:retir` y ninguna de las cinco
/// identidades -las tres retiradas y las dos vigentes- abre otra cuenta. Y cada lote escribio solo
/// su retirada, a su ordinal: en disco quedan `retir:0`, `:1` y `:2`, ninguna mas. Con una sola
/// recuperacion por libro, los demas tests no ven ni el orden, ni la cadena larga, ni la marca de
/// lo escrito despues de reabrir.
#[test]
fn varias_retiradas_sobreviven_a_varios_reinicios_en_su_orden() {
    use crate::crypto::LedgerKey;
    let path = temp_path("retir_varias");
    let frase = "la frase del libro de las retiradas";
    let abrir = || {
        open_encrypted_retry(
            &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
            Some(LedgerKey::from_passphrase(frase)),
        )
        .expect("CRITICO: el libro con sus retiradas no reabre")
    };
    let (x, y, z) = (id_de(SK_ALICE), id_de(0xA11CE_2), id_de(0xA11CE_3));
    let (b, w) = (id_de(SK_BOB), id_de(0xB0B_2));
    let (alice, bob) = {
        let mut l = abrir();
        let alice = open_and_fund(&mut l, SK_ALICE, 1_000);
        let bob = open_and_fund(&mut l, SK_BOB, 0);
        recover_delegated(&mut l, alice, y);
        (alice, bob)
    };
    recover_delegated(&mut abrir(), bob, w);
    recover_delegated(&mut abrir(), alice, z);
    let mut l = abrir();
    assert_eq!(l.recovery_count(), 3);
    assert_eq!(l.retiradas, vec![x, b, y], "las retiradas, en el orden de las recuperaciones");
    assert_eq!(l.retir_cabeza, crate::accounts::cabeza_de_retiradas(&[x, b, y]));
    for (nombre, id) in [("X", x), ("B", b), ("Y", y), ("Z", z), ("W", w)] {
        let r = abrir_ajena(&mut l, id);
        assert!(
            matches!(r, Err(LayerError::IdentityUnavailable)),
            "CRITICO: tras tres reinicios, {nombre} abrio otra cuenta: {r:?}"
        );
    }
    drop(l);
    let claves: Vec<Vec<u8>> =
        volcado(&path).into_iter().map(|(k, _)| k).filter(|k| k.starts_with(b"retir:")).collect();
    assert_eq!(claves, vec![clave_retir(0), clave_retir(1), clave_retir(2)], "una por recuperacion");
    quitar_libro(&path);
}

// -----------------------------------------------------------------
// §812 (RFC-0001, E4): el recifrado, y lo que queda en los ficheros
// -----------------------------------------------------------------
// Al final del fichero, como los del §702: ninguna cita `tests.rs:linea` se mueve. Los que buscan
// en los ficheros del directorio no miran lo que sled devuelve (`volcado`), sino los BYTES de cada
// fichero: sled es un log, y lo sobrescrito sigue en ellos hasta que reaprovecha su segmento.

const FRASE_VIEJA_E4: &str = "la frase vieja del libro";
const FRASE_NUEVA_E4: &str = "la frase nueva del libro";

/// Los dos hermanos del libro, formados como los forma la capa: `<libro>.recifrando` y
/// `<libro>.retirado`, en su mismo directorio.
fn hermanos_de(path: &str) -> (String, String) {
    let p = std::path::Path::new(path);
    let nombre = p.file_name().expect("nombre").to_string_lossy().into_owned();
    let padre = p.parent().expect("padre");
    let con = |s: &str| padre.join(format!("{nombre}{s}")).to_string_lossy().into_owned();
    (con(".recifrando"), con(".retirado"))
}

fn hay(p: &str) -> bool {
    std::fs::symlink_metadata(p).is_ok()
}

fn ficheros_bajo(dir: &std::path::Path, out: &mut Vec<Vec<u8>>) {
    let Ok(entradas) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entradas.flatten() {
        let tipo = e.file_type().expect("tipo");
        if tipo.is_dir() {
            ficheros_bajo(&e.path(), out);
        } else if tipo.is_file() {
            out.push(std::fs::read(e.path()).expect("leer un fichero del libro"));
        }
    }
}

/// **Cuantas de las `agujas` aparecen, byte a byte, en algun fichero** bajo esos directorios; los
/// que no existen se saltan. Cada aguja es un valor sellado: 24 bytes de nonce aleatorio delante y
/// la etiqueta detras, mas de 40 bytes que no aparecen por azar.
fn apariciones(dirs: &[&str], agujas: &[Vec<u8>]) -> usize {
    let mut contenidos = Vec::new();
    for d in dirs {
        ficheros_bajo(std::path::Path::new(d), &mut contenidos);
    }
    agujas
        .iter()
        .filter(|a| {
            contenidos
                .iter()
                .any(|c| c.windows(a.len()).any(|w| w[0] == a[0] && w == a.as_slice()))
        })
        .count()
}

/// Los valores sellados del libro, tal como estan en disco: todo lo que no va en claro.
fn sellados(path: &str) -> Vec<Vec<u8>> {
    use crate::persistence::EN_CLARO;
    volcado(path)
        .into_iter()
        .filter(|(k, _)| !EN_CLARO.contains(&k.as_slice()))
        .map(|(_, v)| v)
        .collect()
}

fn cabecera_de(path: &str) -> Vec<u8> {
    use crate::persistence::CLAVE_KDF;
    sled_open_retry(path).get(CLAVE_KDF).unwrap().expect("un libro v2 lleva cabecera").to_vec()
}

/// El libro EN CLARO: cada valor sellado abierto con la frase fijada a la cabecera del libro, lo
/// que va en claro tal cual, y sin la cabecera. Es lo que el recifrado no puede cambiar.
fn en_claro(path: &str, frase: &str) -> Vec<(Vec<u8>, Vec<u8>)> {
    use crate::crypto::{Kdf, LedgerKey};
    use crate::persistence::{CLAVE_KDF, EN_CLARO};
    let clave = LedgerKey::derivar(frase, &Kdf::de_cabecera(&cabecera_de(path)).unwrap()).unwrap();
    volcado(path)
        .into_iter()
        .filter(|(k, _)| k.as_slice() != CLAVE_KDF)
        .map(|(k, v)| {
            if EN_CLARO.contains(&k.as_slice()) {
                (k, v)
            } else {
                let c = clave.open(&v).expect("abre con la frase del libro");
                (k, c)
            }
        })
        .collect()
}

fn copiar_dir(de: &str, a: &str) {
    std::fs::create_dir_all(a).unwrap();
    for e in std::fs::read_dir(de).unwrap().flatten() {
        let destino = std::path::Path::new(a).join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            copiar_dir(e.path().to_str().unwrap(), destino.to_str().unwrap());
        } else {
            std::fs::copy(e.path(), destino).unwrap();
        }
    }
}

fn abrir_e4(path: &str, frase: &str) -> Result<SovereignLayer, LayerError> {
    use crate::crypto::LedgerKey;
    open_encrypted_retry(
        path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
        Some(LedgerKey::from_passphrase(frase)),
    )
}

/// `SovereignLayer::recifrar`, reintentando como `open_encrypted_retry` solo el Io de un cerrojo
/// que sled no ha soltado todavia: nunca el que llega pasado el compromiso.
fn recifrar_e4(path: &str, vieja: &str, nueva: &str) -> Result<SovereignLayer, LayerError> {
    use crate::crypto::LedgerKey;
    for intento in 0..10 {
        match SovereignLayer::recifrar(
            path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
            LedgerKey::from_passphrase(vieja), LedgerKey::from_passphrase(nueva),
        ) {
            Err(LayerError::Store(StoreError::Io(m))) if intento < 9 && !m.contains("compromiso") => {
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            otro => return otro,
        }
    }
    unreachable!()
}

/// Un libro cifrado con la frase vieja, con una cuenta fondeada y otra vacia, sin hermanos. Las
/// pruebas de la emision las comparte el montaje del §704: cada test paga su Argon2id, no su STARK.
fn libro_e4(nombre: &str) -> (String, AccountIndex, AccountIndex) {
    let path = temp_path(nombre);
    let (r, v) = hermanos_de(&path);
    let _ = std::fs::remove_dir_all(&r);
    let _ = std::fs::remove_dir_all(&v);
    let mut layer = abrir_e4(&path, FRASE_VIEJA_E4).expect("abrir cifrado");
    let alice = open_and_fund(&mut layer, SK_ALICE, 777_777);
    let bob = open_and_fund(&mut layer, SK_BOB, 0);
    (path, alice, bob)
}

fn quitar_e4(path: &str) {
    let (r, v) = hermanos_de(path);
    for d in [path, r.as_str(), v.as_str()] {
        let _ = std::fs::remove_dir_all(d);
    }
    // §816: y el cerrojo del libro, que la capa no borra nunca.
    let _ = std::fs::remove_file(cerrojo_de(path));
}

/// Corre `f` con una muerte inyectada al acabar el paso `paso` del recifrado
/// (`persistence::MORIR_EN`). Dice si murio. Es una aproximacion: el panico corre los `Drop`, y el
/// de sled hace `flush` de lo que tenga abierto, que una muerte de verdad no hace. En cada punto lo
/// unico abierto es el libro viejo, que el recifrado solo lee, asi que no cambia lo que se mide (lo
/// dice `persistence::punto`, con lo medido fuera de la suite con muertes de verdad).
fn morir_en<T>(paso: u8, f: impl FnOnce() -> T) -> bool {
    use crate::persistence::MORIR_EN;
    MORIR_EN.with(|m| m.set(Some(paso)));
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
    MORIR_EN.with(|m| m.set(None));
    r.is_err()
}

/// Corre `f` con un ERROR inyectado al acabar el paso `paso` del recifrado
/// (`persistence::FALLAR_EN`): sigue el camino de un error de verdad en ese paso.
fn fallar_en<T>(paso: u8, f: impl FnOnce() -> T) -> T {
    use crate::persistence::FALLAR_EN;
    FALLAR_EN.with(|m| m.set(Some(paso)));
    let r = f();
    FALLAR_EN.with(|m| m.set(None));
    r
}

/// **§812: la migracion de la version 1 no deja en los ficheros lo sellado con la clave v1.** Hasta
/// el §812 la migracion sobrescribia cada valor en un lote sobre el mismo `sled`, y los valores v1
/// -SHA-256 sin sal- seguian en los ficheros del directorio: el test del §702 solo miraba lo que
/// sled devuelve. Aqui se buscan sus bytes en los ficheros del libro y de sus dos hermanos.
#[test]
fn la_migracion_v1_no_deja_en_los_ficheros_lo_sellado_con_v1() {
    let (path, alice, bob) = libro_e4("e4_migra");
    let (r, v) = hermanos_de(&path);
    rebajar_a_v1(&path, FRASE_VIEJA_E4);
    let agujas = sellados(&path);
    assert!(agujas.len() >= 20, "el libro tenia que llevar valores sellados: {}", agujas.len());
    // Prueba de vida del instrumento: antes de abrir, todos estan en los ficheros.
    assert_eq!(apariciones(&[&path], &agujas), agujas.len(), "el instrumento no ve lo que hay");
    {
        let layer = abrir_e4(&path, FRASE_VIEJA_E4).expect("la v1 abre con su frase");
        assert_eq!(layer.balance_of(alice), Some(777_777));
        assert_eq!(layer.balance_of(bob), Some(0));
        assert_eq!(layer.total_supply(), 777_777);
    }
    let quedan = apariciones(&[&path, &r, &v], &agujas);
    assert_eq!(
        quedan,
        0,
        "CRITICO: tras la migracion siguen en los ficheros {quedan} de {} valores sellados con la clave v1",
        agujas.len()
    );
    assert!(!hay(&r) && !hay(&v), "la migracion dejo un hermano");
    let layer = abrir_e4(&path, FRASE_VIEJA_E4).expect("reabre en la version 2");
    assert_eq!(layer.balance_of(alice), Some(777_777));
    drop(layer);
    quitar_e4(&path);
}

/// **§812 (E4): `recifrar` cambia la frase y la sal del libro entero, sin cambiar lo que dice, y no
/// deja en los ficheros nada sellado con la vieja.** El contenido en claro es el mismo, pareja a
/// pareja; la sal es otra, con el coste de la version 2; la frase vieja ya no abre; no queda
/// ningun hermano; y el directorio conserva sus permisos, y `db` los suyos.
#[test]
fn recifrar_cambia_frase_y_sal_y_no_deja_lo_sellado_con_la_vieja() {
    use crate::crypto::Kdf;
    let (path, alice, bob) = libro_e4("e4_recifra");
    let (r, v) = hermanos_de(&path);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = sled_open_retry(&path); // que sled haya soltado el libro
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o710)).unwrap();
        std::fs::set_permissions(format!("{path}/db"), std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    let claro_antes = en_claro(&path, FRASE_VIEJA_E4);
    let agujas = sellados(&path);
    let sal_antes = cabecera_de(&path)[13..].to_vec();
    assert_eq!(apariciones(&[&path], &agujas), agujas.len(), "el instrumento no ve lo que hay");

    {
        let layer = recifrar_e4(&path, FRASE_VIEJA_E4, FRASE_NUEVA_E4).expect("recifrar");
        assert_eq!(layer.balance_of(alice), Some(777_777));
        assert_eq!(layer.balance_of(bob), Some(0));
        assert_eq!(layer.total_supply(), 777_777);
    }

    let quedan = apariciones(&[&path, &r, &v], &agujas);
    assert_eq!(
        quedan,
        0,
        "CRITICO: tras recifrar siguen en los ficheros {quedan} de {} valores sellados con la frase vieja",
        agujas.len()
    );
    assert!(!hay(&r) && !hay(&v), "el recifrado dejo un hermano");
    assert_eq!(en_claro(&path, FRASE_NUEVA_E4), claro_antes, "el recifrado cambio lo que dice el libro");
    let cabecera = cabecera_de(&path);
    assert!(matches!(Kdf::de_cabecera(&cabecera), Ok(Kdf::Argon2idV2 { .. })), "cabecera v2");
    assert_ne!(cabecera[13..], sal_antes[..], "la sal no cambio");
    let vieja = abrir_e4(&path, FRASE_VIEJA_E4);
    assert!(
        matches!(vieja, Err(LayerError::Store(StoreError::IntegrityFailure { .. }))),
        "CRITICO: la frase vieja abre el libro recifrado: {:?}",
        vieja.err()
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let modo = |p: &str| std::fs::metadata(p).unwrap().permissions().mode() & 0o7777;
        assert_eq!(modo(&path), 0o710, "el directorio del libro perdio sus permisos");
        assert_eq!(modo(&format!("{path}/db")), 0o600, "`db` perdio sus permisos");
    }
    quitar_e4(&path);
}

/// **§812 (E4): un recifrado que falla antes del compromiso no toca el libro**: el `sled` queda
/// pareja a pareja como estaba y no queda ningun hermano. Tres fallos: (a) un valor que no abre
/// con la vieja -una clave ajena, metida con sled en crudo, que `load` no lee-: el error la
/// nombra, y descartarla sin verificar no se hace; (b) la frase vieja equivocada; y (c) una clave
/// nueva ya fijada, sin frase, que no puede tomar una sal nueva.
#[test]
fn recifrar_que_falla_antes_del_compromiso_no_toca_el_libro() {
    use crate::crypto::{Kdf, LedgerKey};
    let (path, _, _) = libro_e4("e4_falla");
    let (r, v) = hermanos_de(&path);
    {
        let db = sled_open_retry(&path);
        let ajena: Vec<u8> = (0..64u32).map(|i| (i.wrapping_mul(97) ^ 0x5A) as u8).collect();
        db.insert(b"zz:ajena", ajena).unwrap();
        db.flush().unwrap();
    }
    let antes = volcado(&path);

    let a = recifrar_e4(&path, FRASE_VIEJA_E4, FRASE_NUEVA_E4);
    match &a {
        Err(LayerError::Store(StoreError::Malformed(m))) if m.contains("zz:ajena") => {}
        otro => panic!("CRITICO: un valor que no abre con la vieja no paro el recifrado: {:?}", otro.as_ref().err()),
    }
    drop(a);
    assert_eq!(volcado(&path), antes, "(a) el recifrado fallido toco el libro");
    assert!(!hay(&r) && !hay(&v), "(a) el recifrado fallido dejo un hermano");

    let b = recifrar_e4(&path, "otra frase", FRASE_NUEVA_E4);
    assert!(b.is_err(), "CRITICO: otra frase recifro el libro");
    drop(b);
    assert_eq!(volcado(&path), antes, "(b) una frase mala toco el libro");
    assert!(!hay(&r) && !hay(&v), "(b) una frase mala dejo un hermano");

    let fijada = LedgerKey::derivar(FRASE_NUEVA_E4, &Kdf::nuevo()).unwrap();
    let c = SovereignLayer::recifrar(
        &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
        LedgerKey::from_passphrase(FRASE_VIEJA_E4), fijada,
    );
    assert!(
        matches!(c, Err(LayerError::Store(StoreError::Malformed(_)))),
        "una clave nueva sin frase: {:?}",
        c.as_ref().err()
    );
    drop(c);
    assert_eq!(volcado(&path), antes, "(c) una clave sin frase toco el libro");
    assert!(!hay(&r) && !hay(&v), "(c) una clave sin frase dejo un hermano");
    quitar_e4(&path);
}

/// **§812 (E4): una muerte ANTES del compromiso deja el libro con la frase vieja.** La muerte se
/// inyecta al acabar cada uno de los pasos 1 a 4 -crear el hermano, copiar, verificar,
/// sincronizar-, y deja el libro y su `.recifrando`. La apertura siguiente borra el hermano, que
/// solo lleva lo sellado con la nueva, y el libro abre con la vieja, pareja a pareja como estaba;
/// la nueva no lo abre. Con el codigo de antes del §812 el hermano se quedaba en el disco. Y mientras
/// el hermano lleva los ficheros que sled crea con la umask -pasos 1 a 3-, esta en 0700: solo al
/// final del paso 4, con los modos de los ficheros ya copiados, toma el del libro.
#[test]
fn una_muerte_antes_del_compromiso_deja_el_libro_con_la_frase_vieja() {
    let (base, alice, _) = libro_e4("e4_muerte_antes");
    let antes = volcado(&base);
    for paso in 1..=4u8 {
        let path = temp_path(&format!("e4_muerte_antes_{paso}"));
        quitar_e4(&path);
        let (r, v) = hermanos_de(&path);
        copiar_dir(&base, &path);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o751)).unwrap();
            std::fs::set_permissions(format!("{path}/db"), std::fs::Permissions::from_mode(0o600)).unwrap();
        }
        let murio = morir_en(paso, || recifrar_e4(&path, FRASE_VIEJA_E4, FRASE_NUEVA_E4));
        assert!(murio, "paso {paso}: el recifrado no murio donde se le pidio");
        assert!(hay(&path) && hay(&r) && !hay(&v), "paso {paso}: la muerte tenia que dejar el libro y su .recifrando");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let modo = |p: &str| std::fs::metadata(p).unwrap().permissions().mode() & 0o7777;
            if paso < 4 {
                assert_eq!(modo(&r), 0o700, "CRITICO: paso {paso}: el .recifrando se abrio antes de tener los modos del libro");
            } else {
                assert_eq!(modo(&r), 0o751, "paso 4: el .recifrando no tomo el modo del libro");
                assert_eq!(modo(&format!("{r}/db")), 0o600, "paso 4: el `db` de la copia no tomo el del libro");
            }
        }

        let layer = abrir_e4(&path, FRASE_VIEJA_E4)
            .unwrap_or_else(|e| panic!("CRITICO: paso {paso}: el libro no abre con la frase vieja: {e}"));
        assert_eq!(layer.balance_of(alice), Some(777_777), "paso {paso}");
        assert_eq!(layer.total_supply(), 777_777, "paso {paso}");
        drop(layer);
        assert!(!hay(&r) && !hay(&v), "paso {paso}: la apertura dejo un hermano");
        assert_eq!(volcado(&path), antes, "paso {paso}: la vuelta atras toco el libro");
        assert!(abrir_e4(&path, FRASE_NUEVA_E4).is_err(), "CRITICO: paso {paso}: la frase nueva abre");
        quitar_e4(&path);
    }
    quitar_e4(&base);
}

/// **§812 (E4): una muerte DESPUES del compromiso deja el libro con la frase nueva.** La muerte se
/// inyecta al acabar el paso 5 (el libro pasa a `.retirado`: quedan el retirado y el
/// `.recifrando`, sin libro), el 6 (el `.recifrando` pasa a ser el libro: quedan el libro y el
/// retirado) y el 7 (el retirado, borrado). La apertura siguiente termina hacia delante: el libro
/// abre con la nueva, dice lo mismo que antes, no queda ningun hermano ni nada sellado con la
/// vieja, y la vieja no abre. Con el codigo de antes del §812, tras el paso 5 nacia un libro VACIO.
#[test]
fn una_muerte_despues_del_compromiso_deja_el_libro_con_la_frase_nueva() {
    let (base, alice, _) = libro_e4("e4_muerte_despues");
    let agujas = sellados(&base);
    let claro = en_claro(&base, FRASE_VIEJA_E4);
    for paso in 5..=7u8 {
        let path = temp_path(&format!("e4_muerte_despues_{paso}"));
        quitar_e4(&path);
        let (r, v) = hermanos_de(&path);
        copiar_dir(&base, &path);
        let murio = morir_en(paso, || recifrar_e4(&path, FRASE_VIEJA_E4, FRASE_NUEVA_E4));
        assert!(murio, "paso {paso}: el recifrado no murio donde se le pidio");
        let en_disco = (hay(&path), hay(&r), hay(&v));
        let esperado = match paso {
            5 => (false, true, true),
            6 => (true, false, true),
            _ => (true, false, false),
        };
        assert_eq!(en_disco, esperado, "paso {paso}: (libro, .recifrando, .retirado)");

        let layer = abrir_e4(&path, FRASE_NUEVA_E4)
            .unwrap_or_else(|e| panic!("CRITICO: paso {paso}: el libro no abre con la frase nueva: {e}"));
        assert_eq!(layer.balance_of(alice), Some(777_777), "CRITICO: paso {paso}: el libro perdio el saldo");
        assert_eq!(layer.total_supply(), 777_777, "CRITICO: paso {paso}: el libro perdio el suministro");
        drop(layer);
        assert!(!hay(&r) && !hay(&v), "paso {paso}: la apertura dejo un hermano");
        let quedan = apariciones(&[&path, &r, &v], &agujas);
        assert_eq!(quedan, 0, "CRITICO: paso {paso}: quedan {quedan} valores sellados con la vieja");
        assert_eq!(en_claro(&path, FRASE_NUEVA_E4), claro, "paso {paso}: el libro no dice lo mismo");
        assert!(abrir_e4(&path, FRASE_VIEJA_E4).is_err(), "CRITICO: paso {paso}: la frase vieja abre");
        quitar_e4(&path);
    }
    quitar_e4(&base);
}

/// **§812: el remedio de un libro que migro el codigo del §702.** Esa migracion sobrescribia en un
/// lote sobre el mismo `sled`, y aqui se reproduce tal cual: el libro abre en la version 2 y lo
/// sellado con la clave v1 sigue en sus ficheros -el defecto, medido-. Recifrarlo con una frase
/// nueva limpia el directorio: ni lo sellado con la v1 ni lo sellado con la v2 de la frase vieja
/// quedan. Y la capa que devuelve `recifrar` escribe con la clave nueva.
#[test]
fn un_libro_migrado_por_el_702_se_limpia_recifrandolo() {
    use crate::crypto::{Kdf, LedgerKey};
    use crate::persistence::{CLAVE_KDF, EN_CLARO};
    let (path, alice, _) = libro_e4("e4_remedio");
    let (r, v) = hermanos_de(&path);
    rebajar_a_v1(&path, FRASE_VIEJA_E4);
    let agujas_v1 = sellados(&path);
    {
        // El cuerpo de `migrar_a_kdf_v2` del §702: cada valor, de la v1 a una v2 de sal nueva, en
        // UN lote sobre el mismo sled.
        let db = sled_open_retry(&path);
        let v1 = LedgerKey::derivar(FRASE_VIEJA_E4, &Kdf::Sha256V1).unwrap();
        let kdf = Kdf::nuevo();
        let v2 = LedgerKey::derivar(FRASE_VIEJA_E4, &kdf).unwrap();
        let mut lote = sled::Batch::default();
        for item in db.iter() {
            let (k, val) = item.unwrap();
            if EN_CLARO.contains(&k.as_ref()) {
                continue;
            }
            lote.insert(k, v2.seal(&v1.open(&val).unwrap()).unwrap());
        }
        lote.insert(CLAVE_KDF, kdf.cabecera().unwrap().to_vec());
        db.apply_batch(lote).unwrap();
        db.flush().unwrap();
    }
    let agujas_v2 = sellados(&path);
    let restos = apariciones(&[&path], &agujas_v1);
    println!("§812: tras la migracion en sitio, {restos} de {} valores v1 en los ficheros", agujas_v1.len());
    assert!(restos > 0, "la migracion en sitio ya no deja el resto: el test no mide el defecto");

    {
        let mut layer = recifrar_e4(&path, FRASE_VIEJA_E4, FRASE_NUEVA_E4).expect("recifrar");
        layer.open_account(BaseElement::new(0xE4));
    }
    let v1 = apariciones(&[&path, &r, &v], &agujas_v1);
    let v2 = apariciones(&[&path, &r, &v], &agujas_v2);
    assert_eq!(v1, 0, "CRITICO: tras el remedio quedan {v1} valores sellados con la clave v1");
    assert_eq!(v2, 0, "CRITICO: tras el remedio quedan {v2} valores sellados con la frase vieja");
    let layer = abrir_e4(&path, FRASE_NUEVA_E4).expect("abre con la nueva");
    assert_eq!(layer.balance_of(alice), Some(777_777));
    assert_eq!(layer.account_count(), 3, "la capa recifrada no escribio con la clave nueva");
    drop(layer);
    quitar_e4(&path);
}

/// **§812 (E4): la apertura rehusa los estados que el recifrado no deja**, sin tocar nada y sin
/// crear un libro vacio: (a) el libro con sus dos hermanos -lo deja, por ejemplo, el codigo de
/// antes del §812, que crea un libro vacio donde solo estaban los hermanos-; (b) un `.recifrando`
/// sin libro; (c) un `.retirado` sin libro.
#[test]
fn abrir_rehusa_los_estados_que_el_recifrado_no_deja() {
    let basura = |d: &str| {
        let db = sled::open(d).unwrap();
        db.insert(b"basura", b"x".to_vec()).unwrap();
        db.flush().unwrap();
    };
    let rehusa = |r: Result<SovereignLayer, LayerError>, que: &str| match r {
        Err(LayerError::Store(StoreError::Malformed(_))) => {}
        otro => panic!("CRITICO: {que}: {:?}", otro.err()),
    };

    let (path, _, _) = libro_e4("e4_estados");
    let (r, v) = hermanos_de(&path);
    basura(&r);
    basura(&v);
    let (vp, vr, vv) = (volcado(&path), volcado(&r), volcado(&v));
    rehusa(abrir_e4(&path, FRASE_VIEJA_E4), "(a) abrio con sus dos hermanos al lado");
    assert_eq!((volcado(&path), volcado(&r), volcado(&v)), (vp, vr, vv), "(a) toco algo");
    quitar_e4(&path);

    for (que, cual) in [("(b) un .recifrando sin libro", 0), ("(c) un .retirado sin libro", 1)] {
        let solo = temp_path(&format!("e4_estados_solo_{cual}"));
        quitar_e4(&solo);
        let (sr, sv) = hermanos_de(&solo);
        let hermano = if cual == 0 { &sr } else { &sv };
        basura(hermano);
        let antes = volcado(hermano);
        rehusa(
            open_retry(&solo, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS),
            que,
        );
        assert!(!hay(&solo), "CRITICO: {que}: nacio un libro vacio");
        assert_eq!(volcado(hermano), antes, "{que}: toco el hermano");
        quitar_e4(&solo);
    }
}

/// **§812 (E4): `recifrar` rehusa una ruta que es un enlace.** Renombrar el enlace y borrarlo
/// quitaria solo el enlace, y el libro de verdad, al que apunta, se quedaria con todo lo sellado
/// con la vieja mientras `recifrar` decia que si.
#[cfg(unix)]
#[test]
fn recifrar_rehusa_una_ruta_que_es_un_enlace() {
    let (real, _, _) = libro_e4("e4_enlace_real");
    let enlace = temp_path("e4_enlace");
    quitar_e4(&enlace);
    let _ = std::fs::remove_file(&enlace);
    std::os::unix::fs::symlink(&real, &enlace).unwrap();
    let (r, v) = hermanos_de(&enlace);
    let antes = volcado(&real);
    let agujas = sellados(&real);
    let res = recifrar_e4(&enlace, FRASE_VIEJA_E4, FRASE_NUEVA_E4);
    assert!(
        matches!(res, Err(LayerError::Store(StoreError::Malformed(_)))),
        "CRITICO: se recifro por un enlace: {:?}",
        res.as_ref().err()
    );
    drop(res);
    assert_eq!(volcado(&real), antes, "el libro de verdad cambio");
    assert_eq!(apariciones(&[&real], &agujas), agujas.len());
    assert!(!hay(&r) && !hay(&v), "quedo un hermano");
    assert!(std::fs::symlink_metadata(&enlace).unwrap().file_type().is_symlink());
    quitar_e4(&enlace); // ⚠️ §816: el enlace y el `.cerrojo` que `recifrar` deja a su lado
    quitar_e4(&real);
}

/// **§812 (E4): `recifrar` no borra lo que no es de sled.** Un fichero del operador dentro del
/// directorio del libro se iria con el `.retirado`; el recifrado rehusa, lo nombra y no toca nada.
#[test]
fn recifrar_no_borra_lo_que_no_es_de_sled() {
    let (path, _, _) = libro_e4("e4_ajeno");
    let (r, v) = hermanos_de(&path);
    let nota = format!("{path}/nota.txt");
    std::fs::write(&nota, b"del operador").unwrap();
    let antes = volcado(&path);
    let res = recifrar_e4(&path, FRASE_VIEJA_E4, FRASE_NUEVA_E4);
    match &res {
        Err(LayerError::Store(StoreError::Malformed(m))) if m.contains("nota.txt") => {}
        otro => panic!("CRITICO: el recifrado no rehuso un fichero ajeno: {:?}", otro.as_ref().err()),
    }
    drop(res);
    assert_eq!(std::fs::read(&nota).ok().as_deref(), Some(&b"del operador"[..]), "CRITICO: se borro la nota");
    assert_eq!(volcado(&path), antes, "se toco el libro");
    assert!(!hay(&r) && !hay(&v), "quedo un hermano");
    quitar_e4(&path);
}

/// **§812: una muerte a mitad de la MIGRACION de la version 1 no deja en el directorio nada sellado
/// con la clave v1.** La migracion es la via que cierra el fallo de la §3.14 de `SECURITY.md`, y
/// las muertes de arriba solo recorren `recifrar`. Aqui se inyecta la muerte al acabar cada uno de
/// los pasos 1 a 7 de la migracion que hace la apertura, y se reabre con la misma frase: antes del
/// compromiso, la apertura borra el `.recifrando` y vuelve a migrar; despues, termina hacia delante.
/// El instrumento es mas ancho que el de los otros tests: busca en TODOS los ficheros del
/// directorio que guarda el libro, que al final solo puede llevar el libro, y no solo lo vivo de un
/// momento, sino tambien lo que el libro llevo antes -la v2 de la que se rebajo, la v1 del rebaje y
/// tres reescrituras con la v1- y lo que la migracion muerta dejo sellado con su sal en el
/// `.recifrando`.
#[test]
fn una_muerte_en_la_migracion_v1_no_deja_en_el_directorio_lo_sellado_con_v1() {
    use crate::crypto::{Kdf, LedgerKey};
    use crate::persistence::EN_CLARO;
    let base_dir = temp_path("e4_migra_muerte_base");
    std::fs::create_dir_all(&base_dir).unwrap();
    let base = format!("{base_dir}/libro");
    let (alice, bob) = {
        let mut layer = abrir_e4(&base, FRASE_VIEJA_E4).expect("abrir cifrado");
        (open_and_fund(&mut layer, SK_ALICE, 777_777), open_and_fund(&mut layer, SK_BOB, 0))
    };
    // Las agujas: lo sellado con la v2 de la que se rebaja, y cada ronda con la v1.
    let mut agujas = sellados(&base);
    rebajar_a_v1(&base, FRASE_VIEJA_E4);
    let v1 = LedgerKey::derivar(FRASE_VIEJA_E4, &Kdf::Sha256V1).unwrap();
    for _ in 0..3 {
        agujas.extend(sellados(&base));
        let db = sled_open_retry(&base);
        let mut lote = sled::Batch::default();
        for item in db.iter() {
            let (k, val) = item.unwrap();
            if EN_CLARO.contains(&k.as_ref()) {
                continue;
            }
            lote.insert(k, v1.seal(&v1.open(&val).unwrap()).unwrap());
        }
        db.apply_batch(lote).unwrap();
        db.flush().unwrap();
    }
    agujas.extend(sellados(&base));
    // Prueba de vida del instrumento: antes de migrar, todas estan en los ficheros.
    assert_eq!(apariciones(&[&base_dir], &agujas), agujas.len(), "el instrumento no ve lo que hay");
    println!(
        "§812: {} agujas en los ficheros antes de migrar: la v2 de la que se rebajo, el rebaje y tres reescrituras v1",
        agujas.len()
    );

    for paso in 1..=7u8 {
        let dir = temp_path(&format!("e4_migra_muerte_{paso}"));
        std::fs::create_dir_all(&dir).unwrap();
        let path = format!("{dir}/libro");
        let (r, v) = hermanos_de(&path);
        copiar_dir(&base, &path);
        let murio = morir_en(paso, || abrir_e4(&path, FRASE_VIEJA_E4));
        assert!(murio, "paso {paso}: la migracion no murio donde se le pidio");
        let mut todas = agujas.clone();
        if paso <= 4 {
            assert!(hay(&path) && hay(&r) && !hay(&v), "paso {paso}: la muerte tenia que dejar el libro y su .recifrando");
            if hay(&format!("{r}/db")) {
                // Lo que la migracion muerta sello con su sal: la apertura migra con otra.
                todas.extend(sellados(&r));
            }
        }

        let layer = abrir_e4(&path, FRASE_VIEJA_E4)
            .unwrap_or_else(|e| panic!("CRITICO: paso {paso}: el libro no abre con su frase: {e}"));
        assert_eq!(layer.balance_of(alice), Some(777_777), "CRITICO: paso {paso}: el libro perdio el saldo");
        assert_eq!(layer.balance_of(bob), Some(0), "paso {paso}");
        assert_eq!(layer.total_supply(), 777_777, "CRITICO: paso {paso}: el libro perdio el suministro");
        drop(layer);
        assert!(
            matches!(Kdf::de_cabecera(&cabecera_de(&path)), Ok(Kdf::Argon2idV2 { .. })),
            "paso {paso}: el libro no quedo en la version 2"
        );
        // ⚠️ §816: decia `vec!["libro".to_string()]`. Desde el §816 la apertura deja al lado el cerrojo
        // del libro, un fichero vacio que la capa no borra; vacio, no guarda nada sellado.
        let mut en_dir: Vec<String> = std::fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        en_dir.sort();
        assert_eq!(
            en_dir,
            vec!["libro".to_string(), "libro.cerrojo".to_string()],
            "paso {paso}: el directorio lleva algo mas que el libro y su cerrojo"
        );
        assert_eq!(std::fs::metadata(format!("{dir}/libro.cerrojo")).unwrap().len(), 0, "paso {paso}: el cerrojo no esta vacio");
        let quedan = apariciones(&[&dir], &todas);
        assert_eq!(
            quedan,
            0,
            "CRITICO: paso {paso}: quedan en el directorio {quedan} de {} valores sellados con la v1 o con una sal anterior",
            todas.len()
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
    let _ = std::fs::remove_dir_all(&base_dir);
}

/// **§812: un error pasado el compromiso dice con que frase queda el libro, y es verdad.** La pieza
/// la usan el recifrado y la migracion de la version 1, y no dejan el libro igual: el recifrado,
/// con la frase NUEVA; la migracion, con la MISMA con que se abrio, ya en la version 2, tambien
/// cuando la hace `recifrar` antes de recifrar un libro v1. Se inyecta un error al acabar cada uno
/// de los pasos 5 a 7, en los tres caminos: (a) `recifrar` de un libro v2, (b) la apertura de un
/// libro v1 y (c) `recifrar` de un libro v1, que falla en su migracion. Quien siga el mensaje
/// encuentra el libro con la frase que dice. Hasta este arreglo el mensaje decia «la frase NUEVA»
/// en los tres, y en (b) y (c) el libro quedaba con la vieja. Y dice que, hasta la apertura
/// siguiente, el `.retirado` -si sigue- guarda lo sellado con la clave de antes.
#[test]
fn un_error_pasado_el_compromiso_dice_con_que_frase_queda_el_libro() {
    use crate::crypto::{Kdf, LedgerKey};
    let mensaje = |r: Result<SovereignLayer, LayerError>, que: &str| -> String {
        match r {
            Err(LayerError::Store(StoreError::Io(m))) if m.contains("pasado el compromiso") => m,
            otro => panic!("{que}: tenia que dar el Io de pasado el compromiso: {:?}", otro.err()),
        }
    };
    // La apertura sin reintentar el Io que llega pasado el compromiso: `open_encrypted_retry` lo
    // reintentaria, y el reintento lo terminaria sin que nadie viera el mensaje.
    let abrir_sin_tapar = |path: &str, frase: &str| -> Result<SovereignLayer, LayerError> {
        for intento in 0..10 {
            match SovereignLayer::open_encrypted(
                path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
                Some(LedgerKey::from_passphrase(frase)),
            ) {
                Err(LayerError::Store(StoreError::Io(m))) if intento < 9 && !m.contains("compromiso") => {
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
                otro => return otro,
            }
        }
        unreachable!()
    };
    let (base_v2, alice, _) = libro_e4("e4_mensaje_v2");
    let base_v1 = temp_path("e4_mensaje_v1");
    copiar_dir(&base_v2, &base_v1);
    rebajar_a_v1(&base_v1, FRASE_VIEJA_E4);
    let es_v2 = |path: &str| matches!(Kdf::de_cabecera(&cabecera_de(path)), Ok(Kdf::Argon2idV2 { .. }));

    for paso in 5..=7u8 {
        // (a) recifrar un libro v2: queda con la NUEVA.
        let path = temp_path(&format!("e4_mensaje_a_{paso}"));
        quitar_e4(&path);
        copiar_dir(&base_v2, &path);
        let m = mensaje(
            fallar_en(paso, || recifrar_e4(&path, FRASE_VIEJA_E4, FRASE_NUEVA_E4)),
            &format!("(a) paso {paso}"),
        );
        assert!(m.contains("frase NUEVA"), "(a) paso {paso}: el mensaje no dice la nueva: {m}");
        assert!(
            m.contains("`.retirado`, si sigue en el disco, guarda lo sellado con la clave de antes"),
            "(a) paso {paso}: el mensaje no dice que el retirado, si sigue, guarda lo de la vieja: {m}"
        );
        let layer = abrir_e4(&path, FRASE_NUEVA_E4)
            .unwrap_or_else(|e| panic!("CRITICO: (a) paso {paso}: el mensaje dice la nueva y no abre: {e}"));
        assert_eq!(layer.balance_of(alice), Some(777_777), "(a) paso {paso}");
        drop(layer);
        quitar_e4(&path);

        // (b) abrir un libro v1, que se migra: queda con la MISMA, en la version 2.
        let path = temp_path(&format!("e4_mensaje_b_{paso}"));
        quitar_e4(&path);
        copiar_dir(&base_v1, &path);
        let m = mensaje(fallar_en(paso, || abrir_sin_tapar(&path, FRASE_VIEJA_E4)), &format!("(b) paso {paso}"));
        assert!(
            m.contains("MISMA") && !m.contains("NUEVA"),
            "CRITICO: (b) paso {paso}: la migracion dice otra frase que la que deja: {m}"
        );
        let layer = abrir_e4(&path, FRASE_VIEJA_E4)
            .unwrap_or_else(|e| panic!("CRITICO: (b) paso {paso}: el mensaje dice la misma y no abre: {e}"));
        assert_eq!(layer.balance_of(alice), Some(777_777), "(b) paso {paso}");
        drop(layer);
        assert!(es_v2(&path), "(b) paso {paso}: no quedo en la version 2");
        quitar_e4(&path);

        // (c) recifrar un libro v1: falla la migracion de antes, y queda con la VIEJA, en la 2.
        let path = temp_path(&format!("e4_mensaje_c_{paso}"));
        quitar_e4(&path);
        copiar_dir(&base_v1, &path);
        let m = mensaje(
            fallar_en(paso, || recifrar_e4(&path, FRASE_VIEJA_E4, FRASE_NUEVA_E4)),
            &format!("(c) paso {paso}"),
        );
        assert!(
            m.contains("MISMA") && !m.contains("NUEVA"),
            "CRITICO: (c) paso {paso}: el mensaje manda a la frase nueva y el libro queda con la vieja: {m}"
        );
        let layer = abrir_e4(&path, FRASE_VIEJA_E4)
            .unwrap_or_else(|e| panic!("CRITICO: (c) paso {paso}: el libro no abre con la vieja: {e}"));
        assert_eq!(layer.balance_of(alice), Some(777_777), "(c) paso {paso}");
        drop(layer);
        assert!(es_v2(&path), "(c) paso {paso}: no quedo en la version 2");
        assert!(abrir_e4(&path, FRASE_NUEVA_E4).is_err(), "(c) paso {paso}: la nueva abre un libro que no se recifro");
        quitar_e4(&path);
    }
    quitar_e4(&base_v2);
    quitar_e4(&base_v1);
}

/// **§812 (E4) con el §802: el recifrado conserva las identidades retiradas.** `retir:{k}` y su raiz
/// `root:retir` van selladas, asi que el paso 2 las copia con la clave nueva y el paso 3 las
/// compara como a las demas. Un libro con una recuperacion (X -> Y), en la version 2 y en la 1 -que
/// `recifrar` migra antes-: tras recifrar, la capa que devuelve y la que se reabre del disco siguen
/// con su recuperacion y su retirada, y ni X ni Y abren otra cuenta; las dos claves dicen en claro
/// lo mismo que antes, abren con la nueva y no con la vieja; y en los ficheros no queda nada sellado
/// con la vieja ni ningun hermano. Si el recifrado las perdiera, el libro no reabriria -sin raiz,
/// un libro con recuperaciones no abre (§802)-, y si las perdiera tambien la carga, X volveria a
/// abrir.
#[test]
fn recifrar_conserva_las_identidades_retiradas() {
    use crate::crypto::{Kdf, LedgerKey};
    use crate::persistence::CLAVE_KDF;
    let es_retir = |k: &[u8]| k.starts_with(b"retir:") || k == b"root:retir";
    let clave_de = |path: &str, frase: &str| {
        let kdf = match sled_open_retry(path).get(CLAVE_KDF).unwrap() {
            Some(c) => Kdf::de_cabecera(&c).unwrap(),
            None => Kdf::Sha256V1,
        };
        LedgerKey::derivar(frase, &kdf).unwrap()
    };
    let (x, y) = (id_de(SK_ALICE), id_de(0xA11CE_2));
    let rechaza = |l: &mut SovereignLayer, que: &str| {
        assert_eq!(l.recovery_count(), 1, "{que}: la recuperacion");
        assert_eq!(l.retiradas, vec![x], "CRITICO: {que}: la retirada no siguio");
        for (nombre, id) in [("X, la retirada", x), ("Y, la recuperada", y)] {
            let r = abrir_ajena(l, id);
            assert!(
                matches!(r, Err(LayerError::IdentityUnavailable)),
                "CRITICO: {que}: tras recifrar, {nombre} abrio otra cuenta: {r:?}"
            );
        }
    };
    for v1 in [false, true] {
        let que = if v1 { "(v1)" } else { "(v2)" };
        let path = temp_path(if v1 { "e4_retir_v1" } else { "e4_retir_v2" });
        quitar_e4(&path);
        let (r, v) = hermanos_de(&path);
        libro_con_una_recuperacion(&path, Some(LedgerKey::from_passphrase(FRASE_VIEJA_E4)));
        if v1 {
            rebajar_a_v1(&path, FRASE_VIEJA_E4);
        }
        let vieja = clave_de(&path, FRASE_VIEJA_E4);
        let retir_antes: Vec<(Vec<u8>, Vec<u8>)> = volcado(&path)
            .into_iter()
            .filter(|(k, _)| es_retir(k))
            .map(|(k, s)| {
                let c = vieja.open(&s).expect("abre con la vieja");
                (k, c)
            })
            .collect();
        assert_eq!(retir_antes.len(), 2, "{que}: retir:0 y root:retir");
        let agujas = sellados(&path);

        {
            let mut l = recifrar_e4(&path, FRASE_VIEJA_E4, FRASE_NUEVA_E4)
                .unwrap_or_else(|e| panic!("CRITICO: {que}: un libro con recuperaciones no se recifra: {e}"));
            rechaza(&mut l, &format!("{que} la capa que devuelve"));
        }
        {
            let mut l = abrir_e4(&path, FRASE_NUEVA_E4)
                .unwrap_or_else(|e| panic!("CRITICO: {que}: el libro recifrado no reabre: {e}"));
            rechaza(&mut l, &format!("{que} reabierto"));
        }
        let nueva = clave_de(&path, FRASE_NUEVA_E4);
        let vieja_v2 = LedgerKey::derivar(FRASE_VIEJA_E4, &Kdf::de_cabecera(&cabecera_de(&path)).unwrap()).unwrap();
        let retir_ahora: Vec<(Vec<u8>, Vec<u8>)> = volcado(&path)
            .into_iter()
            .filter(|(k, _)| es_retir(k))
            .map(|(k, s)| {
                let nombre = String::from_utf8_lossy(&k).into_owned();
                assert!(vieja.open(&s).is_err(), "CRITICO: {que}: {nombre} sigue sellado con la vieja");
                assert!(vieja_v2.open(&s).is_err(), "CRITICO: {que}: {nombre} abre con la frase vieja");
                let c = nueva.open(&s).unwrap_or_else(|_| panic!("{que}: {nombre} no abre con la nueva"));
                (k, c)
            })
            .collect();
        assert_eq!(retir_ahora, retir_antes, "CRITICO: {que}: las retiradas no dicen lo mismo que antes");
        let quedan = apariciones(&[&path, &r, &v], &agujas);
        assert_eq!(quedan, 0, "CRITICO: {que}: quedan {quedan} valores sellados con la vieja");
        assert!(!hay(&r) && !hay(&v), "{que}: quedo un hermano");
        quitar_e4(&path);
    }
}

/// **§812 (E4): `recifrar` rehusa una ruta sin libro, y no crea ninguno.** (a) Una ruta que no
/// existe y (b) un directorio vacio: `Malformed`, y ni el libro, ni su `.recifrando`, ni su
/// `.retirado`; el directorio vacio sigue vacio. Sin el paso c de `recifrar`, `open_encrypted`
/// crearia ahi un libro vacio con la frase vieja, lo recifraria y daria `Ok`. ⚠️ §816: y tampoco el
/// cerrojo del libro, `<libro>.cerrojo`: el arreglo, hasta sus revisores, lo tomaba antes de mirar
/// si habia libro, y lo dejaba creado junto a una ruta sin libro (uno lo vio leyendo el codigo; este
/// test no lo miraba, porque `quitar_e4` lo borra).
#[test]
fn recifrar_rehusa_una_ruta_sin_libro() {
    let path = temp_path("e4_sin_libro");
    quitar_e4(&path);
    let (r, v) = hermanos_de(&path);
    let rehusa = |que: &str| {
        let res = recifrar_e4(&path, FRASE_VIEJA_E4, FRASE_NUEVA_E4);
        assert!(
            matches!(res, Err(LayerError::Store(StoreError::Malformed(_)))),
            "CRITICO: {que}: recifrar no rehuso una ruta sin libro: {:?}",
            res.as_ref().err()
        );
    };
    rehusa("(a) una ruta que no existe");
    assert!(!hay(&path), "CRITICO: (a) nacio un libro donde no lo habia");
    assert!(!hay(&r) && !hay(&v), "(a) quedo un hermano");
    assert!(!hay(&cerrojo_de(&path)), "(a) quedo un `.cerrojo` donde no hay libro");

    std::fs::create_dir_all(&path).unwrap();
    rehusa("(b) un directorio vacio");
    assert!(
        std::fs::read_dir(&path).unwrap().next().is_none(),
        "CRITICO: (b) se escribio un libro en el directorio vacio"
    );
    assert!(!hay(&r) && !hay(&v), "(b) quedo un hermano");
    assert!(!hay(&cerrojo_de(&path)), "(b) quedo un `.cerrojo` junto a un directorio que no es un libro");
    quitar_e4(&path);
}

/// **§812: la importacion de una instantanea cifrada barre el claro que dejaba una importacion
/// muerta.** Entre `5d2833e` y `c87d7d6` (§774), `import_snapshot_with_key` escribia el estado
/// entero en claro en `{instantanea}.tmp-descifrada` y lo borraba despues: un proceso muerto en
/// medio lo dejaba en el disco. (a) Con ese fichero al lado, la importacion lo borra y el estado
/// importado es el mismo; (b) `barrer_descifrada` dice si lo borro; (c) un enlace con ese nombre no
/// es lo que aquel codigo escribia: no se sigue, el enlace se queda y su destino no se toca. Que un
/// DIRECTORIO con ese nombre tampoco se toca lo mide el test del §774
/// (`una_instantanea_cifrada_se_importa_sin_escribir_el_claro_en_disco`).
#[test]
fn la_importacion_barre_el_claro_que_dejaba_una_importacion_muerta() {
    use crate::crypto::LedgerKey;
    use crate::snapshot::barrer_descifrada;
    let (path, alice, _) = libro_e4("e4_barre");
    let file = format!("{path}.instantanea");
    let resto = format!("{file}.tmp-descifrada");
    let _ = std::fs::remove_file(&file);
    let _ = std::fs::remove_file(&resto);
    let key = LedgerKey::from_passphrase(FRASE_VIEJA_E4);
    let raiz = {
        let layer = abrir_e4(&path, FRASE_VIEJA_E4).expect("abrir");
        layer.export_snapshot(&file).expect("exportar");
        layer.state_root()
    };

    std::fs::write(&resto, b"el estado del libro en claro, de mentira").unwrap();
    let l = SovereignLayer::import_snapshot_with_key(&file, &key).expect("importar");
    assert_eq!(l.state_root(), raiz, "(a) el estado importado no es el exportado");
    assert_eq!(l.balance_of(alice), Some(777_777), "(a)");
    assert!(!hay(&resto), "CRITICO: (a) la importacion dejo el claro de una importacion muerta");

    assert!(!barrer_descifrada(&file).expect("(b) sin resto"), "(b) dice que borro lo que no habia");
    std::fs::write(&resto, b"otra vez").unwrap();
    assert!(barrer_descifrada(&file).expect("(b) con resto"), "(b) no dice que lo borro");
    assert!(!hay(&resto), "(b) dice que lo borro y sigue");

    #[cfg(unix)]
    {
        let destino = format!("{path}.destino");
        std::fs::write(&destino, b"del operador").unwrap();
        std::os::unix::fs::symlink(&destino, &resto).unwrap();
        SovereignLayer::import_snapshot_with_key(&file, &key).expect("(c) importar");
        assert!(
            std::fs::symlink_metadata(&resto).map(|m| m.file_type().is_symlink()).unwrap_or(false),
            "CRITICO: (c) se borro un enlace que no escribio la importacion"
        );
        assert_eq!(std::fs::read(&destino).ok().as_deref(), Some(&b"del operador"[..]), "(c) se toco el destino");
        let _ = std::fs::remove_file(&resto);
        let _ = std::fs::remove_file(&destino);
    }
    let _ = std::fs::remove_file(&file);
    quitar_e4(&path);
}

// ---------------------------------------------------------------------------------------------
// §816 (RFC-0001, E4): la carrera entre una apertura y un recifrado, y el cerrojo del libro
// ---------------------------------------------------------------------------------------------
// Medida sobre b6994c3 por un revisor, con dos hilos y pausas inyectadas: la apertura miraba los
// hermanos -el cerrojo de sled de cada uno, tomado y SOLTADO en el acto; sin hermanos, ninguno- y
// despues actuaba y llamaba a `sled::open`, que CREA el libro si no existe. Nada retenia un cerrojo
// entre mirar y actuar, y un recifrado que llegaba a su compromiso en medio dejaba a la apertura
// crear un libro VACIO donde estaba el libro. Desde el §816 la apertura y el recifrado toman el
// cerrojo del libro, `<libro>.cerrojo`, un fichero que la capa no renombra ni borra. Los ganchos de pausa
// (`persistence::pausa`) paran un hilo en un punto con nombre hasta que el otro llega al suyo o
// acaba; las carreras van de una en una, porque el gancho es uno para todo el proceso.

/// `<libro>.cerrojo`, formado como lo forma la capa.
fn cerrojo_de(path: &str) -> String {
    let p = std::path::Path::new(path);
    let nombre = p.file_name().expect("nombre").to_string_lossy().into_owned();
    p.parent().expect("padre").join(format!("{nombre}.cerrojo")).to_string_lossy().into_owned()
}

/// Lo que hay con el nombre del libro en su directorio -el libro, sus hermanos y su cerrojo-,
/// ordenado.
fn junto_al_libro(path: &str) -> Vec<String> {
    let p = std::path::Path::new(path);
    let nombre = p.file_name().expect("nombre").to_string_lossy().into_owned();
    let con_punto = format!("{nombre}.");
    let mut v: Vec<String> = std::fs::read_dir(p.parent().expect("padre"))
        .expect("leer el directorio del libro")
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| *n == nombre || n.starts_with(&con_punto))
        .collect();
    v.sort();
    v
}

/// Los hitos de una carrera: cada hilo marca los suyos y espera alguno del otro, con un tope de
/// 30 s para que un hito que no llega no cuelgue la suite.
struct Hitos(std::sync::Mutex<std::collections::BTreeSet<&'static str>>, std::sync::Condvar);

impl Hitos {
    fn nuevos() -> std::sync::Arc<Self> {
        std::sync::Arc::new(Hitos(Default::default(), Default::default()))
    }

    fn marcar(&self, hito: &'static str) {
        self.0.lock().unwrap_or_else(|e| e.into_inner()).insert(hito);
        self.1.notify_all();
    }

    /// Espera a que este marcado alguno de `hitos`; dice si llego antes del tope.
    fn esperar(&self, hitos: &[&'static str]) -> bool {
        let g = self.0.lock().unwrap_or_else(|e| e.into_inner());
        let (_g, tope) = self
            .1
            .wait_timeout_while(g, std::time::Duration::from_secs(30), |m| !hitos.iter().any(|h| m.contains(h)))
            .unwrap_or_else(|e| e.into_inner());
        !tope.timed_out()
    }
}

/// Las carreras van de una en una: el gancho de pausa es uno para todo el proceso.
static CARRERAS: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// El gancho puesto. Al soltarse lo quita -tambien si el test cae- y deja pasar a la carrera
/// siguiente.
struct ConGancho(#[allow(dead_code)] std::sync::MutexGuard<'static, ()>);

impl Drop for ConGancho {
    fn drop(&mut self) {
        *crate::persistence::PAUSA.lock().unwrap_or_else(|e| e.into_inner()) = None;
    }
}

/// Pone el gancho: `f(hilo, punto)` se llama en cada punto de pausa de cada hilo con nombre; los
/// de los otros tests, que corren a la vez, no llevan el nombre de ningun hilo de una carrera.
fn con_gancho(f: impl Fn(&str, &'static str) + Send + Sync + 'static) -> ConGancho {
    let turno = CARRERAS.lock().unwrap_or_else(|e| e.into_inner());
    let g: crate::persistence::Gancho = std::sync::Arc::new(move |donde: &'static str| {
        if let Some(hilo) = std::thread::current().name() {
            f(hilo, donde)
        }
    });
    *crate::persistence::PAUSA.lock().unwrap_or_else(|e| e.into_inner()) = Some(g);
    ConGancho(turno)
}

/// Lo que sale de una apertura o de un recifrado en un hilo de la carrera: el saldo de alice y el
/// suministro, o el error. La capa se suelta dentro del hilo.
type Salida = Result<(Option<u64>, u64), LayerError>;

fn salida(r: Result<SovereignLayer, LayerError>, alice: AccountIndex) -> Salida {
    r.map(|l| (l.balance_of(alice), l.total_supply()))
}

/// Marca su hito al soltarse: tambien si el hilo cae, para que el otro no espere al tope.
struct AlAcabar(std::sync::Arc<Hitos>, &'static str);

impl Drop for AlAcabar {
    fn drop(&mut self) {
        self.0.marcar(self.1);
    }
}

/// Corre `f` en un hilo llamado `nombre`, y marca `fin` al acabar.
fn en_hilo(
    nombre: &str,
    hitos: &std::sync::Arc<Hitos>,
    fin: &'static str,
    f: impl FnOnce() -> Salida + Send + 'static,
) -> std::thread::JoinHandle<Salida> {
    let al_acabar = AlAcabar(hitos.clone(), fin);
    std::thread::Builder::new()
        .name(nombre.into())
        .spawn(move || {
            let _al_acabar = al_acabar;
            f()
        })
        .expect("lanzar el hilo")
}

/// La apertura cifrada, SIN reintentar el Io: en una carrera, el Io es lo que se mide.
fn abrir_una_vez(path: &str, frase: &str) -> Result<SovereignLayer, LayerError> {
    use crate::crypto::LedgerKey;
    SovereignLayer::open_encrypted(
        path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
        Some(LedgerKey::from_passphrase(frase)),
    )
}

/// `recifrar`, SIN reintentar el Io.
fn recifrar_una_vez(path: &str, vieja: &str, nueva: &str) -> Result<SovereignLayer, LayerError> {
    use crate::crypto::LedgerKey;
    SovereignLayer::recifrar(
        path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
        LedgerKey::from_passphrase(vieja), LedgerKey::from_passphrase(nueva),
    )
}

/// Si la salida es el Io del cerrojo del libro, con su nombre.
fn es_io_del_cerrojo(r: &Salida, path: &str) -> bool {
    matches!(r, Err(LayerError::Store(StoreError::Io(m))) if m.contains(&cerrojo_de(path)))
}

/// **Despues de una carrera, el libro sigue entero**: cada frase de `otras` -que no son la suya-
/// da `IntegrityFailure` y deja el libro en su sitio; no queda ningun hermano; y la suya lo abre con
/// el saldo y el suministro enteros. Ninguna apertura crea un libro vacio ni borra la unica copia.
fn el_libro_sigue(path: &str, alice: AccountIndex, suya: &str, otras: &[&str], que: &str) {
    let (r, v) = hermanos_de(path);
    for otra in otras {
        let res = abrir_e4(path, otra);
        assert!(
            matches!(res, Err(LayerError::Store(StoreError::IntegrityFailure { .. }))),
            "CRITICO: {que}: «{otra}» no es la frase del libro, y su apertura no dio IntegrityFailure: {:?}",
            res.as_ref().map(|l| (l.balance_of(alice), l.total_supply()))
        );
        drop(res);
        assert!(hay(path), "CRITICO: {que}: la apertura con «{otra}» se llevo el libro");
    }
    assert!(!hay(&r) && !hay(&v), "{que}: quedo un hermano: {:?}", junto_al_libro(path));
    let l = abrir_e4(path, suya).unwrap_or_else(|e| panic!("CRITICO: {que}: el libro no abre con su frase: {e}"));
    assert_eq!(l.balance_of(alice), Some(777_777), "CRITICO: {que}: el libro perdio el saldo");
    assert_eq!(l.total_supply(), 777_777, "CRITICO: {que}: el libro perdio el suministro");
}

/// **§816: una apertura y un recifrado a la vez no crean un libro vacio** (la carrera (a), sin
/// hermanos). Medido sobre b6994c3: la apertura A miraba si habia hermanos -no los habia, y sin
/// ellos no miraba ningun cerrojo- y se paraba antes de `sled::open`; el recifrado B llegaba a su
/// compromiso, `rename(P, V)`; A creaba un libro VACIO en P y abria `Ok`, sin saldo; el paso 6 de B
/// fallaba diciendo «la apertura siguiente lo termina», y la apertura siguiente rehusaba: el libro
/// entero se quedaba en V y en R, y ya no abria. Con el cerrojo del libro, A lo tiene desde antes de
/// mirar hasta que vuelve con el libro abierto: B no lo toma en su tope y da `Io` con su nombre, sin
/// tocar nada; A abre el libro entero; y despues ninguna apertura crea un libro vacio ni borra la
/// unica copia: la frase nueva no lo abre y la vieja lo abre con el saldo.
#[test]
fn una_apertura_y_un_recifrado_a_la_vez_no_crean_un_libro_vacio() {
    let (path, alice, _) = libro_e4("carrera_a");
    drop(sled_open_retry(&path)); // que sled haya soltado el libro
    let hitos = Hitos::nuevos();
    let h = hitos.clone();
    let gancho = con_gancho(move |hilo, donde| match (hilo, donde) {
        ("carrera-a-A", "abrir:antes_de_sled") => {
            h.marcar("A:tope");
            h.esperar(&["B:5", "B:fin"]);
        }
        ("carrera-a-A", "abrir:tras_sled") => h.marcar("A:sled"),
        ("carrera-a-B", "recifrar:5") => {
            h.marcar("B:5");
            h.esperar(&["A:sled", "A:fin"]);
        }
        _ => {}
    });
    let pa = path.clone();
    let a = en_hilo("carrera-a-A", &hitos, "A:fin", move || salida(abrir_una_vez(&pa, FRASE_VIEJA_E4), alice));
    assert!(hitos.esperar(&["A:tope"]), "(a) la apertura no llego a su tope");
    let pb = path.clone();
    let b = en_hilo("carrera-a-B", &hitos, "B:fin", move || {
        salida(recifrar_una_vez(&pb, FRASE_VIEJA_E4, FRASE_NUEVA_E4), alice)
    });
    let (ra, rb) = (a.join().expect("hilo A"), b.join().expect("hilo B"));
    drop(gancho);
    eprintln!("§816 (a): A = {ra:?}; B = {rb:?}; junto al libro: {:?}", junto_al_libro(&path));
    assert!(
        matches!(ra, Ok((Some(777_777), 777_777))),
        "CRITICO: (a) la apertura A no abrio el libro entero: {ra:?}"
    );
    assert!(es_io_del_cerrojo(&rb, &path), "(a) el recifrado B, en su tope, tenia que dar el Io del cerrojo: {rb:?}");
    el_libro_sigue(&path, alice, FRASE_VIEJA_E4, &[FRASE_NUEVA_E4], "(a)");
    quitar_e4(&path);
}

/// **§816: una apertura que resuelve lo que dejo una muerte y un recifrado a la vez no se llevan el
/// libro** (las carreras (b) y (c)). Medido sobre b6994c3, en los dos estados de partida:
///
/// - (b) Una muerte tras el paso 6 deja el libro, con la frase nueva, y su `.retirado`. La apertura
///   A miraba los cerrojos -libres- y se paraba antes de borrar el retirado; el recifrado B resolvia
///   lo mismo, abria y recifraba hasta su compromiso, que pone el libro en un `.retirado` NUEVO; A
///   borraba ese, el de B, y creaba un libro vacio en P; el paso 6 de B fallaba, y la apertura
///   siguiente con la frase que nombraba su error borraba el `.recifrando` -la unica copia que
///   quedaba- como resto de antes del compromiso.
/// - (c) Una muerte antes del compromiso deja el libro, con la frase vieja, y su `.recifrando`. A
///   miraba y se paraba antes de borrarlo; B lo borraba, recifraba hasta su compromiso y dejaba su
///   `.recifrando` NUEVO; A borraba ese y creaba un libro vacio en P; el paso 6 de B fallaba diciendo
///   que el libro quedaba con la frase NUEVA, y no quedaba; y la apertura siguiente, con cualquier
///   frase, borraba el `.retirado` -la unica copia- y la vieja abria `Ok` un libro vacio, sin error.
///
/// El libro, con sus saldos, se perdia entero. Con el cerrojo, B da `Io` en su tope sin tocar nada,
/// A resuelve lo de la muerte y abre el libro entero, y despues ni la frase de B, que no llego a
/// nada, ni la otra lo abren ni lo tocan.
#[test]
fn una_apertura_que_resuelve_y_un_recifrado_a_la_vez_no_se_llevan_el_libro() {
    const TERCERA: &str = "la tercera frase del libro";
    // (que, paso de la muerte, frase con que queda el libro, la otra)
    for (que, paso, suya, otra) in [("(b)", 6u8, FRASE_NUEVA_E4, FRASE_VIEJA_E4), ("(c)", 3, FRASE_VIEJA_E4, FRASE_NUEVA_E4)] {
        let (path, alice, _) = libro_e4(if paso == 6 { "carrera_b" } else { "carrera_c" });
        let (r, v) = hermanos_de(&path);
        assert!(morir_en(paso, || recifrar_e4(&path, FRASE_VIEJA_E4, FRASE_NUEVA_E4)), "{que}: tenia que morir en el paso {paso}");
        if paso == 6 {
            assert!(hay(&path) && !hay(&r) && hay(&v), "(b) el estado de partida tenia que ser el libro y su .retirado");
            drop(sled_open_retry(&v));
        } else {
            assert!(hay(&path) && hay(&r) && !hay(&v), "(c) el estado de partida tenia que ser el libro y su .recifrando");
            drop(sled_open_retry(&r));
        }
        drop(sled_open_retry(&path));
        let hitos = Hitos::nuevos();
        let h = hitos.clone();
        let gancho = con_gancho(move |hilo, donde| match (hilo, donde) {
            ("carrera-b-A", "resolver:tras_cerrojos") => {
                h.marcar("A:tope");
                h.esperar(&["B:5", "B:fin"]);
            }
            ("carrera-b-A", "abrir:tras_sled") => h.marcar("A:sled"),
            ("carrera-b-B", "recifrar:5") => {
                h.marcar("B:5");
                h.esperar(&["A:sled", "A:fin"]);
            }
            _ => {}
        });
        let pa = path.clone();
        let a = en_hilo("carrera-b-A", &hitos, "A:fin", move || salida(abrir_una_vez(&pa, suya), alice));
        assert!(hitos.esperar(&["A:tope"]), "{que} la apertura no llego a mirar los cerrojos");
        let pb = path.clone();
        let b = en_hilo("carrera-b-B", &hitos, "B:fin", move || salida(recifrar_una_vez(&pb, suya, TERCERA), alice));
        let (ra, rb) = (a.join().expect("hilo A"), b.join().expect("hilo B"));
        drop(gancho);
        eprintln!("§816 {que}: A = {ra:?}; B = {rb:?}; junto al libro: {:?}", junto_al_libro(&path));
        assert!(
            matches!(ra, Ok((Some(777_777), 777_777))),
            "CRITICO: {que} la apertura A no abrio el libro entero: {ra:?}"
        );
        assert!(es_io_del_cerrojo(&rb, &path), "{que} el recifrado B, en su tope, tenia que dar el Io del cerrojo: {rb:?}");
        el_libro_sigue(&path, alice, suya, &[TERCERA, otra], que);
        quitar_e4(&path);
    }
}

/// **§816: dos recifrados a la vez: uno acaba y el otro da `Io` sin tocar nada.** B2 empieza primero
/// y se para en su apertura con la frase vieja, despues de mirar los hermanos y antes de
/// `sled::open`; B1 llega entonces. Sobre b6994c3, B1 recifraba hasta su compromiso y B2 creaba un
/// libro vacio en P: el paso 6 de B1 fallaba, B2 recifraba el libro vacio hasta toparse con el
/// `.retirado` de B1, y el libro quedaba repartido en tres nombres que la apertura siguiente rehusa.
/// Con el cerrojo, B2 lo tiene desde antes de mirar los hermanos hasta el final; B1 no lo toma en su
/// tope y da `Io` con su nombre;
/// B2 acaba con el libro entero; y el libro abre con la frase de B2, no con la de B1 ni con la vieja.
#[test]
fn dos_recifrados_a_la_vez_uno_acaba_y_el_otro_no_toca_nada() {
    const TERCERA: &str = "la tercera frase del libro";
    let (path, alice, _) = libro_e4("carrera_dos");
    drop(sled_open_retry(&path));
    let hitos = Hitos::nuevos();
    let h = hitos.clone();
    let gancho = con_gancho(move |hilo, donde| match (hilo, donde) {
        ("carrera-dos-B2", "abrir:antes_de_sled") => {
            h.marcar("B2:tope");
            h.esperar(&["B1:5", "B1:fin"]);
        }
        ("carrera-dos-B2", "abrir:tras_sled") => h.marcar("B2:sled"),
        ("carrera-dos-B1", "recifrar:5") => {
            h.marcar("B1:5");
            h.esperar(&["B2:sled", "B2:fin"]);
        }
        _ => {}
    });
    let p2 = path.clone();
    let b2 = en_hilo("carrera-dos-B2", &hitos, "B2:fin", move || {
        salida(recifrar_una_vez(&p2, FRASE_VIEJA_E4, FRASE_NUEVA_E4), alice)
    });
    assert!(hitos.esperar(&["B2:tope"]), "B2 no llego a su apertura");
    let p1 = path.clone();
    let b1 = en_hilo("carrera-dos-B1", &hitos, "B1:fin", move || salida(recifrar_una_vez(&p1, FRASE_VIEJA_E4, TERCERA), alice));
    let (r2, r1) = (b2.join().expect("hilo B2"), b1.join().expect("hilo B1"));
    drop(gancho);
    eprintln!("§816 (dos): B2 = {r2:?}; B1 = {r1:?}; junto al libro: {:?}", junto_al_libro(&path));
    assert!(matches!(r2, Ok((Some(777_777), 777_777))), "CRITICO: el primer recifrado no acabo con el libro entero: {r2:?}");
    assert!(es_io_del_cerrojo(&r1, &path), "el segundo recifrado, en su tope, tenia que dar el Io del cerrojo: {r1:?}");
    el_libro_sigue(&path, alice, FRASE_NUEVA_E4, &[TERCERA, FRASE_VIEJA_E4], "(dos)");
    quitar_e4(&path);
}

/// **§816: una apertura que no toma el cerrojo en su tope da `Io` con su nombre, sin tocar nada**,
/// mientras otro recifra o migra el libro. B ya tiene el libro abierto y va a empezar a copiarlo
/// -en `recifrar`, y en la apertura que migra un libro v1-, y se para ahi; A llega entonces: espera
/// el tope, diez veces cada 50 ms, y da `Io` con el nombre del cerrojo; junto al libro no cambia
/// nada; B acaba, y el libro abre con la frase con que B lo deja. Sin el cerrojo de B en ese tramo,
/// A no veia hermanos, se paraba antes de `sled::open`, B llegaba a su compromiso y A creaba un
/// libro vacio en P: es lo que cazan, en el §816, los mutantes que no toman el cerrojo en el
/// recifrado o que lo sueltan en la apertura antes de migrar.
#[test]
fn una_apertura_en_el_tope_del_cerrojo_da_io_sin_tocar_nada() {
    for migra in [false, true] {
        let que = if migra { "(migracion)" } else { "(recifrado)" };
        let (path, alice, _) = libro_e4(if migra { "carrera_tope_m" } else { "carrera_tope_r" });
        if migra {
            rebajar_a_v1(&path, FRASE_VIEJA_E4);
        }
        drop(sled_open_retry(&path));
        let hitos = Hitos::nuevos();
        let h = hitos.clone();
        let gancho = con_gancho(move |hilo, donde| match (hilo, donde) {
            ("carrera-tope-B", "recifrar:0") => {
                h.marcar("B:abierto");
                h.esperar(&["A:tope", "visto"]);
            }
            ("carrera-tope-B", "recifrar:5") => {
                h.marcar("B:5");
                h.esperar(&["A:sled", "A:fin"]);
            }
            ("carrera-tope-A", "abrir:antes_de_sled") => {
                h.marcar("A:tope");
                h.esperar(&["B:5", "B:fin"]);
            }
            ("carrera-tope-A", "abrir:tras_sled") => h.marcar("A:sled"),
            _ => {}
        });
        let pb = path.clone();
        let b = en_hilo("carrera-tope-B", &hitos, "B:fin", move || {
            let r = if migra {
                abrir_una_vez(&pb, FRASE_VIEJA_E4)
            } else {
                recifrar_una_vez(&pb, FRASE_VIEJA_E4, FRASE_NUEVA_E4)
            };
            salida(r, alice)
        });
        assert!(hitos.esperar(&["B:abierto"]), "{que}: B no llego a abrir el libro");
        let antes = junto_al_libro(&path);
        let pa = path.clone();
        let ra = en_hilo("carrera-tope-A", &hitos, "A:fin", move || salida(abrir_una_vez(&pa, FRASE_VIEJA_E4), alice))
            .join()
            .expect("hilo A");
        let despues = junto_al_libro(&path);
        hitos.marcar("visto");
        let rb = b.join().expect("hilo B");
        drop(gancho);
        eprintln!("§816 {que}: A = {ra:?}; B = {rb:?}; junto al libro antes {antes:?}, despues {despues:?}");
        assert!(es_io_del_cerrojo(&ra, &path), "CRITICO: {que}: la apertura en el tope tenia que dar el Io del cerrojo: {ra:?}");
        assert_eq!(despues, antes, "CRITICO: {que}: la apertura que no tomo el cerrojo cambio lo que hay junto al libro");
        assert!(matches!(rb, Ok((Some(777_777), 777_777))), "{que}: B no acabo con el libro entero: {rb:?}");
        if migra {
            el_libro_sigue(&path, alice, FRASE_VIEJA_E4, &[FRASE_NUEVA_E4], que);
            assert!(
                matches!(crate::crypto::Kdf::de_cabecera(&cabecera_de(&path)), Ok(crate::crypto::Kdf::Argon2idV2 { .. })),
                "{que}: el libro no quedo en la version 2"
            );
        } else {
            el_libro_sigue(&path, alice, FRASE_NUEVA_E4, &[FRASE_VIEJA_E4], que);
        }
        quitar_e4(&path);
    }
}

/// **§816: el cerrojo del libro es un fichero que nada renombra ni borra**, y uno que no es un fichero
/// de verdad no se toma. (a) La primera apertura lo crea vacio y, en Unix, con 0600: solo su dueno
/// puede tomarlo. (b) Un recifrado que acaba, una muerte pasado el compromiso, la apertura que la
/// termina y la migracion de un libro v1 lo dejan donde estaba: el mismo inodo, vacio. Un cerrojo
/// que se renombrara o se borrara no excluiria a nadie: quien espera en el viejo y quien crea uno
/// nuevo lo tendrian los dos. (c) Un `<libro>.cerrojo` que es un directorio o un enlace no se toma:
/// la apertura rehusa con `Malformed`, no crea el libro y no toca lo que hay, ni el destino del
/// enlace.
#[test]
fn el_cerrojo_del_libro_es_un_fichero_que_nada_renombra_ni_borra() {
    const TERCERA: &str = "la tercera frase del libro";
    let (path, alice, _) = libro_e4("cerrojo_fijo");
    let c = cerrojo_de(&path);
    let marca = |que: &str| -> (u64, u64) {
        let md = std::fs::symlink_metadata(&c).unwrap_or_else(|e| panic!("CRITICO: {que}: no esta el cerrojo: {e}"));
        assert!(md.file_type().is_file(), "{que}: el cerrojo no es un fichero");
        #[cfg(unix)]
        let ino = {
            use std::os::unix::fs::MetadataExt;
            md.ino()
        };
        #[cfg(not(unix))]
        let ino = 0;
        (ino, md.len())
    };
    let nacio = marca("(a)");
    assert_eq!(nacio.1, 0, "(a) el cerrojo no nacio vacio");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let modo = std::fs::metadata(&c).unwrap().permissions().mode() & 0o7777;
        assert_eq!(modo, 0o600, "(a) el cerrojo no nacio con 0600");
    }
    drop(recifrar_e4(&path, FRASE_VIEJA_E4, FRASE_NUEVA_E4).expect("(b) recifrar"));
    assert_eq!(marca("(b) recifrado"), nacio, "CRITICO: (b) el recifrado renombro o borro el cerrojo");
    assert!(morir_en(6, || recifrar_e4(&path, FRASE_NUEVA_E4, TERCERA)), "(b) tenia que morir en el paso 6");
    assert_eq!(marca("(b) muerte"), nacio, "CRITICO: (b) la muerte renombro o borro el cerrojo");
    drop(abrir_e4(&path, TERCERA).expect("(b) la apertura que termina el recifrado"));
    assert_eq!(marca("(b) apertura"), nacio, "CRITICO: (b) la apertura que resuelve renombro o borro el cerrojo");
    rebajar_a_v1(&path, TERCERA);
    {
        let l = abrir_e4(&path, TERCERA).expect("(b) la apertura que migra");
        assert_eq!(l.balance_of(alice), Some(777_777), "(b) la migracion");
    }
    assert_eq!(marca("(b) migracion"), nacio, "CRITICO: (b) la migracion renombro o borro el cerrojo");
    quitar_e4(&path);

    let ajeno = temp_path("cerrojo_ajeno");
    quitar_e4(&ajeno);
    let ca = cerrojo_de(&ajeno);
    let rehusa = |que: &str| {
        let r = open_retry(&ajeno, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS);
        assert!(
            matches!(r, Err(LayerError::Store(StoreError::Malformed(ref m))) if m.contains(&ca)),
            "CRITICO: {que}: la apertura tomo un cerrojo que no es un fichero: {:?}",
            r.as_ref().err()
        );
        drop(r);
        assert!(!hay(&ajeno), "CRITICO: {que}: la apertura creo el libro");
    };
    std::fs::create_dir_all(&ca).unwrap();
    rehusa("(c) un directorio");
    assert!(std::fs::read_dir(&ca).unwrap().next().is_none(), "(c) se escribio dentro del directorio");
    std::fs::remove_dir_all(&ca).unwrap();
    #[cfg(unix)]
    {
        let destino = format!("{ajeno}.destino");
        std::fs::write(&destino, b"del operador").unwrap();
        std::os::unix::fs::symlink(&destino, &ca).unwrap();
        rehusa("(c) un enlace");
        assert!(std::fs::symlink_metadata(&ca).unwrap().file_type().is_symlink(), "(c) se toco el enlace");
        assert_eq!(std::fs::read(&destino).ok().as_deref(), Some(&b"del operador"[..]), "(c) se toco el destino");
        let _ = std::fs::remove_file(&ca);
        let _ = std::fs::remove_file(&destino);
    }
    quitar_e4(&ajeno);
}

// ---------------------------------------------------------------------------------------------
// §816: lo que midieron los dos revisores del arreglo, y su atado
// ---------------------------------------------------------------------------------------------

/// **§816: un libro nuevo en un directorio que no existe se abre como antes del cerrojo.** `sled::open`
/// crea la ruta entera del libro; el cerrojo del libro, que se abre antes, no encontraba su
/// directorio, y la apertura daba `Io` con el nombre del `.cerrojo` sin crear nada (medido por los
/// dos revisores del arreglo: el nodo no arrancaba, y el cli no abria, con `--ledger` en un
/// directorio nuevo). La apertura, en claro y cifrada, crea antes el directorio del libro y deja el
/// cerrojo a su lado, vacio y con 0600; `recifrar`, que exige un libro que ya existe, no crea ningun
/// directorio.
#[test]
fn un_libro_nuevo_en_un_directorio_que_no_existe_se_abre_como_antes() {
    let base = temp_path("sin_padre");
    let _ = std::fs::remove_dir_all(&base);
    let en = |que: &str| format!("{base}/{que}/a/b/libro");
    let claro = en("claro");
    let l = SovereignLayer::open(&claro, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS)
        .unwrap_or_else(|e| panic!("CRITICO: en claro, un libro nuevo en un directorio que no existe no abre: {e}"));
    assert_eq!(l.total_supply(), 0, "en claro: el libro nuevo no nacio vacio");
    drop(l);
    let cifrado = en("cifrado");
    let l = abrir_una_vez(&cifrado, FRASE_VIEJA_E4)
        .unwrap_or_else(|e| panic!("CRITICO: cifrado, un libro nuevo en un directorio que no existe no abre: {e}"));
    assert_eq!(l.total_supply(), 0, "cifrado: el libro nuevo no nacio vacio");
    drop(l);
    for p in [&claro, &cifrado] {
        assert!(std::fs::symlink_metadata(p).map(|m| m.is_dir()).unwrap_or(false), "{p}: no se creo el libro");
        let c = std::fs::symlink_metadata(cerrojo_de(p)).unwrap_or_else(|e| panic!("{p}: no se creo su cerrojo: {e}"));
        assert!(c.is_file() && c.len() == 0, "{p}: el cerrojo no es un fichero vacio");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(c.permissions().mode() & 0o7777, 0o600, "{p}: el cerrojo no nacio con 0600");
        }
    }
    let nada = en("recifrar");
    let r = recifrar_una_vez(&nada, FRASE_VIEJA_E4, FRASE_NUEVA_E4).map(|_| ());
    assert!(r.is_err(), "recifrar un libro que no existe no fallo");
    assert!(!hay(&format!("{base}/recifrar")), "CRITICO: recifrar un libro que no existe creo su directorio: {r:?}");
    let _ = std::fs::remove_dir_all(&base);
}

/// **§816: la reentrada del cerrojo no se queda tras volver.** El cerrojo es reentrante en el hilo que
/// lo tiene -la apertura que hace `recifrar` dentro lo encuentra tomado-, y lo que lo hace reentrante
/// es la lista del hilo, que se vacia al soltarlo. Un hilo abre el libro y lo suelta; otro recifra y
/// se para con el cerrojo tomado; la segunda apertura del primer hilo tiene que esperar el tope y dar
/// el `Io` del cerrojo. Si la entrada se quedara en la lista -el mutante M11 de un revisor, que
/// pasaba los cinco tests de la carrera-, esa apertura no lo tomaria, se pararia antes de
/// `sled::open`, el recifrado llegaria a su compromiso y ella crearia un libro vacio: la carrera (a)
/// otra vez, en ese hilo (medido por el revisor: `Ok((None, 0))`, y el recifrado «pasado el
/// compromiso»).
#[test]
fn la_reentrada_del_cerrojo_no_se_queda_tras_volver() {
    let (path, alice, _) = libro_e4("reentrada");
    drop(sled_open_retry(&path));
    let segunda = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let hitos = Hitos::nuevos();
    let (h, s2) = (hitos.clone(), segunda.clone());
    let gancho = con_gancho(move |hilo, donde| {
        let en_segunda = s2.load(std::sync::atomic::Ordering::SeqCst);
        match (hilo, donde) {
            ("reentrada-A", "abrir:antes_de_sled") if en_segunda => {
                h.marcar("A:tope");
                h.esperar(&["B:5", "B:fin"]);
            }
            ("reentrada-A", "abrir:tras_sled") if en_segunda => h.marcar("A:sled"),
            ("reentrada-B", "recifrar:0") => {
                h.marcar("B:0");
                h.esperar(&["A:tope", "A:fin"]);
            }
            ("reentrada-B", "recifrar:5") => {
                h.marcar("B:5");
                h.esperar(&["A:sled", "A:fin"]);
            }
            _ => {}
        }
    });
    let (pa, h2, s3) = (path.clone(), hitos.clone(), segunda.clone());
    let a = en_hilo("reentrada-A", &hitos, "A:fin", move || {
        drop(abrir_e4(&pa, FRASE_VIEJA_E4).expect("la primera apertura del hilo"));
        drop(sled_open_retry(&pa));
        s3.store(true, std::sync::atomic::Ordering::SeqCst);
        h2.marcar("A:abrio");
        assert!(h2.esperar(&["B:0", "B:fin"]), "el recifrado no llego a tener el libro abierto");
        salida(abrir_una_vez(&pa, FRASE_VIEJA_E4), alice)
    });
    assert!(hitos.esperar(&["A:abrio", "A:fin"]), "la primera apertura del hilo no acabo");
    let pb = path.clone();
    let b = en_hilo("reentrada-B", &hitos, "B:fin", move || {
        salida(recifrar_una_vez(&pb, FRASE_VIEJA_E4, FRASE_NUEVA_E4), alice)
    });
    let (ra, rb) = (a.join().expect("hilo A"), b.join().expect("hilo B"));
    drop(gancho);
    eprintln!("§816 reentrada: A, su segunda apertura = {ra:?}; B = {rb:?}; junto al libro: {:?}", junto_al_libro(&path));
    assert!(es_io_del_cerrojo(&ra, &path), "CRITICO: la segunda apertura del hilo no espero el cerrojo: {ra:?}");
    assert!(matches!(rb, Ok((Some(777_777), 777_777))), "el recifrado no acabo con el libro entero: {rb:?}");
    el_libro_sigue(&path, alice, FRASE_NUEVA_E4, &[FRASE_VIEJA_E4], "reentrada");
    quitar_e4(&path);
}

/// **§816: un cerrojo que cambia mientras se toma no se sigue.** Entre mirar el nombre
/// (`symlink_metadata`) y abrirlo, un revisor cambiaba `<libro>.cerrojo` por un enlace en bucle, y la
/// apertura, que lo abria con `create`, seguia el enlace y creaba su DESTINO, un fichero fuera del
/// directorio del libro (medido: en el intento 64 de un bucle, y en el 126 en otra corrida; con
/// b6994c3, ninguno en 9071); y el `flock` quedaba sobre otro inodo. Con el gancho entre mirar y
/// abrir, sin bucle:
///
/// - (i) No estaba. Se crea con `create_new`, que no sigue el enlace: el destino no se crea, la
///   apertura vuelve a mirar y rehusa el enlace, y no crea el libro.
/// - (ii) Estaba, junto a un libro con el `.retirado` de una muerte. El enlace que aparece entre mirar
///   y abrir se abre sin escribir en su destino, y con el `flock` tomado la apertura ve que el nombre
///   ya no es ese fichero y rehusa ANTES de resolver: el `.retirado` sigue, y el destino tambien.
#[cfg(unix)]
#[test]
fn un_cerrojo_que_cambia_mientras_se_toma_no_se_sigue() {
    for estaba in [false, true] {
        let que = if estaba { "(ii) estaba" } else { "(i) no estaba" };
        let (path, alice) = if estaba {
            let (path, alice, _) = libro_e4("toctou_estaba");
            assert!(morir_en(6, || recifrar_e4(&path, FRASE_VIEJA_E4, FRASE_NUEVA_E4)), "{que}: tenia que morir en el paso 6");
            drop(sled_open_retry(&path));
            drop(sled_open_retry(&hermanos_de(&path).1));
            (path, alice)
        } else {
            let path = temp_path("toctou_nuevo");
            quitar_e4(&path);
            (path, 0)
        };
        let (_, v) = hermanos_de(&path);
        let c = cerrojo_de(&path);
        let destino = format!("{path}.destino");
        let _ = std::fs::remove_file(&destino);
        if estaba {
            std::fs::write(&destino, b"del operador").unwrap();
        }
        let hitos = Hitos::nuevos();
        let h = hitos.clone();
        let gancho = con_gancho(move |hilo, donde| {
            if hilo == "toctou-A" && donde == "cerrojo:tras_mirar" {
                h.marcar("A:miro");
                h.esperar(&["cambiado"]);
            }
        });
        let pa = path.clone();
        let a = en_hilo("toctou-A", &hitos, "A:fin", move || salida(abrir_una_vez(&pa, FRASE_NUEVA_E4), alice));
        assert!(hitos.esperar(&["A:miro"]), "{que}: la apertura no llego a mirar el cerrojo");
        let _ = std::fs::remove_file(&c);
        std::os::unix::fs::symlink(&destino, &c).unwrap();
        hitos.marcar("cambiado");
        let ra = a.join().expect("hilo A");
        drop(gancho);
        eprintln!("§816 toctou {que}: A = {ra:?}; destino: {:?}; junto al libro: {:?}", std::fs::read(&destino).ok(), junto_al_libro(&path));
        assert!(
            matches!(ra, Err(LayerError::Store(StoreError::Malformed(ref m))) if m.contains(&c)),
            "CRITICO: {que}: la apertura siguio el enlace: {ra:?}"
        );
        assert!(std::fs::symlink_metadata(&c).unwrap().file_type().is_symlink(), "{que}: se toco el enlace");
        std::fs::remove_file(&c).unwrap();
        if estaba {
            assert_eq!(std::fs::read(&destino).ok().as_deref(), Some(&b"del operador"[..]), "CRITICO: {que}: se toco el destino");
            assert!(hay(&v), "CRITICO: {que}: la apertura resolvio los hermanos sin tener el cerrojo");
            let _ = std::fs::remove_file(&destino);
            el_libro_sigue(&path, alice, FRASE_NUEVA_E4, &[FRASE_VIEJA_E4], que);
        } else {
            assert!(!hay(&destino), "CRITICO: {que}: la apertura creo el destino del enlace");
            assert!(!hay(&path), "{que}: la apertura creo el libro");
        }
        quitar_e4(&path);
    }
}

/// **§816: un cerrojo que se borra o se cambia mientras se tiene no deja seguir a quien lo tenia.** El
/// cerrojo es un fichero, y a mano se puede borrar o sustituir -restaurar una copia, un `rsync`-
/// mientras alguien lo tiene; quien llega despues crea o toma uno nuevo y entra. Medido por un
/// revisor sobre el primer arreglo del §816: con la apertura A parada antes de `sled::open` y su
/// cerrojo borrado o cambiado por otro fichero, el recifrado B tomaba el nuevo y llegaba a su
/// compromiso, y A creaba un libro vacio, la carrera (a) otra vez. Desde el §816 quien lo tiene mira,
/// antes de lo que no se deshace -`sled::open` en la apertura, el compromiso en el recifrado-, que el
/// nombre siga siendo el fichero que tomo (`Cerrojo::sigue`), y si no, rehusa sin tocar nada:
///
/// - (A) La apertura lo tiene y se para antes de `sled::open`; se borra, o se cambia por otro fichero
///   o por un enlace; B recifra hasta su compromiso, o rehusa el enlace. A rehusa, B acaba, y el libro
///   abre con la frase con que lo deja B.
/// - (B) El recifrado lo tiene, con el libro abierto, y se para antes de copiarlo; se borra; A toma
///   uno nuevo y se para antes de `sled::open`. B rehusa antes de su compromiso, borra su copia y deja
///   el libro con la vieja; A sigue y abre el libro entero, o da el `Io` de sled si este aun no lo ha
///   soltado.
#[test]
fn un_cerrojo_borrado_o_cambiado_mientras_se_tiene_no_deja_seguir() {
    for como in ["borrado", "otro fichero", "enlace"] {
        if como == "enlace" && !cfg!(unix) {
            continue;
        }
        let que = format!("(A) {como}");
        let (path, alice, _) = libro_e4(&format!("cambiado_a_{}", como.len()));
        drop(sled_open_retry(&path));
        let c = cerrojo_de(&path);
        let destino = format!("{path}.destino");
        let _ = std::fs::remove_file(&destino);
        let hitos = Hitos::nuevos();
        let h = hitos.clone();
        let gancho = con_gancho(move |hilo, donde| match (hilo, donde) {
            ("cambiado-A", "abrir:antes_de_sled") => {
                h.marcar("A:tope");
                h.esperar(&["B:5", "B:fin"]);
            }
            ("cambiado-A", "abrir:tras_sled") => h.marcar("A:sled"),
            ("cambiado-B", "recifrar:5") => {
                h.marcar("B:5");
                h.esperar(&["A:sled", "A:fin"]);
            }
            _ => {}
        });
        let pa = path.clone();
        let a = en_hilo("cambiado-A", &hitos, "A:fin", move || salida(abrir_una_vez(&pa, FRASE_VIEJA_E4), alice));
        assert!(hitos.esperar(&["A:tope"]), "{que}: la apertura no llego a su tope");
        std::fs::remove_file(&c).expect("borrar el cerrojo");
        if como == "otro fichero" {
            std::fs::write(&c, b"").unwrap();
        }
        #[cfg(unix)]
        if como == "enlace" {
            std::os::unix::fs::symlink(&destino, &c).unwrap();
        }
        let pb = path.clone();
        let b = en_hilo("cambiado-B", &hitos, "B:fin", move || {
            salida(recifrar_una_vez(&pb, FRASE_VIEJA_E4, FRASE_NUEVA_E4), alice)
        });
        let (ra, rb) = (a.join().expect("hilo A"), b.join().expect("hilo B"));
        drop(gancho);
        eprintln!("§816 {que}: A = {ra:?}; B = {rb:?}; junto al libro: {:?}", junto_al_libro(&path));
        assert!(
            matches!(ra, Err(LayerError::Store(StoreError::Malformed(ref m))) if m.contains(&c)),
            "CRITICO: {que}: la apertura siguio sin su cerrojo: {ra:?}"
        );
        if como == "enlace" {
            assert!(
                matches!(rb, Err(LayerError::Store(StoreError::Malformed(ref m))) if m.contains(&c)),
                "{que}: el recifrado tomo un enlace: {rb:?}"
            );
            assert!(!hay(&destino), "CRITICO: {que}: se creo el destino del enlace");
            std::fs::remove_file(&c).unwrap();
            el_libro_sigue(&path, alice, FRASE_VIEJA_E4, &[FRASE_NUEVA_E4], &que);
        } else {
            assert!(matches!(rb, Ok((Some(777_777), 777_777))), "{que}: el recifrado no acabo con el libro entero: {rb:?}");
            el_libro_sigue(&path, alice, FRASE_NUEVA_E4, &[FRASE_VIEJA_E4], &que);
        }
        quitar_e4(&path);
    }

    let que = "(B) borrado";
    let (path, alice, _) = libro_e4("cambiado_b");
    drop(sled_open_retry(&path));
    let c = cerrojo_de(&path);
    let hitos = Hitos::nuevos();
    let h = hitos.clone();
    let gancho = con_gancho(move |hilo, donde| match (hilo, donde) {
        ("cambiado2-B", "recifrar:0") => {
            h.marcar("B:0");
            h.esperar(&["A:tope", "A:fin"]);
        }
        ("cambiado2-B", "recifrar:5") => {
            h.marcar("B:5");
            h.esperar(&["A:sled", "A:fin"]);
        }
        ("cambiado2-A", "abrir:antes_de_sled") => {
            h.marcar("A:tope");
            h.esperar(&["B:5", "B:fin"]);
        }
        ("cambiado2-A", "abrir:tras_sled") => h.marcar("A:sled"),
        _ => {}
    });
    let pb = path.clone();
    let b = en_hilo("cambiado2-B", &hitos, "B:fin", move || {
        salida(recifrar_una_vez(&pb, FRASE_VIEJA_E4, FRASE_NUEVA_E4), alice)
    });
    assert!(hitos.esperar(&["B:0"]), "{que}: el recifrado no llego a tener el libro abierto");
    std::fs::remove_file(&c).expect("borrar el cerrojo");
    let pa = path.clone();
    let a = en_hilo("cambiado2-A", &hitos, "A:fin", move || salida(abrir_una_vez(&pa, FRASE_VIEJA_E4), alice));
    let (rb, ra) = (b.join().expect("hilo B"), a.join().expect("hilo A"));
    drop(gancho);
    eprintln!("§816 {que}: A = {ra:?}; B = {rb:?}; junto al libro: {:?}", junto_al_libro(&path));
    assert!(
        matches!(rb, Err(LayerError::Store(StoreError::Malformed(ref m))) if m.contains(&c)),
        "CRITICO: {que}: el recifrado siguio sin su cerrojo: {rb:?}"
    );
    assert!(
        matches!(ra, Ok((Some(777_777), 777_777)) | Err(LayerError::Store(StoreError::Io(_)))),
        "CRITICO: {que}: la apertura no abrio el libro entero: {ra:?}"
    );
    el_libro_sigue(&path, alice, FRASE_VIEJA_E4, &[FRASE_NUEVA_E4], que);
    quitar_e4(&path);
}

/// **§816: una ruta sin nombre no abre el libro sin su cerrojo.** Una ruta que acaba en `..` no tiene
/// nombre del que formar `<libro>.cerrojo`, y aun asi puede nombrar un libro: `<libro>/blobs/..` es el
/// libro mismo, porque sled le crea `blobs`. El arreglo, hasta sus revisores, la abria sin cerrojo
/// -uno lo vio leyendo el codigo-, y una apertura por ahi quedaba fuera de la exclusion, como antes
/// del §816. La apertura, cifrada y en claro, y `recifrar` la rehusan con `Malformed`, sin tocar nada:
/// junto al libro queda lo mismo que habia, y el libro abre por su nombre, entero.
#[test]
fn una_ruta_sin_nombre_no_abre_el_libro_sin_su_cerrojo() {
    let rehusa = |que: &str, libro: &str, r: Result<SovereignLayer, LayerError>, antes: &[String]| {
        assert!(
            matches!(r, Err(LayerError::Store(StoreError::Malformed(ref m))) if m.contains("sin nombre no hay cerrojo")),
            "CRITICO: {que}: una ruta sin nombre no se rehuso: {:?}",
            r.as_ref().map(|l| l.total_supply())
        );
        drop(r);
        assert_eq!(junto_al_libro(libro), antes, "CRITICO: {que}: se toco algo junto al libro");
    };
    let (path, alice, _) = libro_e4("sin_nombre");
    let alias = format!("{path}/blobs/..");
    assert!(std::fs::metadata(&alias).map(|m| m.is_dir()).unwrap_or(false), "el libro no tiene `blobs`: {alias}");
    let antes = junto_al_libro(&path);
    rehusa("cifrada", &path, abrir_una_vez(&alias, FRASE_VIEJA_E4), &antes);
    rehusa("recifrar", &path, recifrar_una_vez(&alias, FRASE_VIEJA_E4, FRASE_NUEVA_E4), &antes);
    el_libro_sigue(&path, alice, FRASE_VIEJA_E4, &[FRASE_NUEVA_E4], "sin nombre");
    quitar_e4(&path);

    let claro = temp_path("sin_nombre_claro");
    quitar_e4(&claro);
    drop(
        SovereignLayer::open(&claro, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS)
            .expect("un libro en claro"),
    );
    let antes = junto_al_libro(&claro);
    let r = SovereignLayer::open(
        &format!("{claro}/blobs/.."), custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
    );
    rehusa("en claro", &claro, r, &antes);
    quitar_e4(&claro);
}

/// **§817: con clave, una instantanea en claro no se importa.** Hasta el §817,
/// `import_snapshot_with_key` importaba la marca `0x00` sin mirar la clave (`rehusar_en_claro`, al
/// final de `snapshot.rs`). (a) Una capa cifrada exporta su instantanea; quien puede escribir donde
/// se guarda pone en su sitio una en claro, hecha con la biblioteca, con otro saldo: con la frase
/// buena, la importacion falla con `Malformed`, y su error dice que esta en claro y nombra
/// `import_snapshot`. (b) Con otra frase, o con una clave ya fijada a su KDF, tampoco. (c) La misma,
/// sin clave, por `import_snapshot`, si: es la via a sabiendas. (d) Con la frase, el mismo cuerpo en
/// claro tras una marca que no es de cifrado (0x03, 0xff) no se importa. (e) Y una v8 en claro, que
/// sin clave se importa (la lectura de v3..v8 sigue, la 76), con la frase tampoco.
#[test]
fn con_clave_una_instantanea_en_claro_no_se_importa() {
    use crate::crypto::LedgerKey;
    const MARCA_EN_CLARO: u8 = 0x00; // `SNAPSHOT_PLAIN`, privada de snapshot.rs
    let path = temp_path("claro_con_clave_817");
    let file = format!("{path}.instantanea");
    let _ = std::fs::remove_file(&file);
    let frase = "una contrasena larga de prueba";
    {
        let mut layer = open_encrypted_retry(
            &path,
            custodian_root(),
            governance_root(),
            LIMIT,
            MAX_SUPPLY,
            MAX_ACCOUNTS,
            Some(LedgerKey::from_passphrase(frase)),
        )
        .expect("abrir cifrada");
        open_and_fund(&mut layer, SK_ALICE, 250_000);
        layer.export_snapshot(&file).expect("exportar cifrada");
    }
    assert_ne!(std::fs::read(&file).expect("leer")[0], MARCA_EN_CLARO, "la exportada va cifrada");

    let mut otra = new_layer();
    let alice = open_and_fund(&mut otra, SK_ALICE, 999_999);
    otra.export_snapshot(&file).expect("(a) la sustituye una en claro");
    assert_eq!(std::fs::read(&file).expect("leer")[0], MARCA_EN_CLARO);
    match SovereignLayer::import_snapshot_with_key(&file, &LedgerKey::from_passphrase(frase)) {
        Ok(l) => panic!(
            "CRITICO: (a) con la frase se importo una instantanea en claro: alice tiene {:?}",
            l.balance_of(alice)
        ),
        Err(e) => assert!(
            matches!(e, LayerError::Store(crate::store::StoreError::Malformed(ref m))
                if m.contains("en claro") && m.contains("import_snapshot")),
            "(a) el error no es Malformed, o no dice que esta en claro y la via sin clave: {e}"
        ),
    }
    assert!(
        SovereignLayer::import_snapshot_with_key(&file, &LedgerKey::from_passphrase("otra")).is_err(),
        "CRITICO: (b) con otra frase se importo una instantanea en claro"
    );
    let fijada = LedgerKey::derivar(frase, &crate::crypto::Kdf::Sha256V1).expect("derivar");
    assert!(
        SovereignLayer::import_snapshot_with_key(&file, &fijada).is_err(),
        "CRITICO: (b) con una clave ya fijada se importo una instantanea en claro"
    );
    let r = SovereignLayer::import_snapshot(&file).expect("(c) sin clave se importa");
    assert_eq!(r.balance_of(alice), Some(999_999), "(c)");

    let claro = std::fs::read(&file).expect("leer");
    for marca in [0x03u8, 0xff] {
        let mut v = claro.clone();
        v[0] = marca;
        std::fs::write(&file, &v).expect("escribir");
        assert!(
            SovereignLayer::import_snapshot_with_key(&file, &LedgerKey::from_passphrase(frase)).is_err(),
            "CRITICO: (d) con la frase se importo un cuerpo en claro tras la marca {marca:#04x}"
        );
    }
    // (e) La v8 es la v9 sin su cola de retiradas, que aqui va vacia: ocho ceros al final.
    let n = claro.len();
    assert_eq!(&claro[n - 8..], &[0u8; 8], "(e) sin retiradas");
    let mut v8 = vec![MARCA_EN_CLARO];
    v8.extend_from_slice(b"ZKSSL8\0\0");
    v8.extend_from_slice(&claro[9..n - 8]);
    std::fs::write(&file, &v8).expect("escribir la v8");
    let r = SovereignLayer::import_snapshot(&file).expect("(e) la v8 en claro se importa sin clave");
    assert_eq!(r.balance_of(alice), Some(999_999), "(e)");
    assert!(
        SovereignLayer::import_snapshot_with_key(&file, &LedgerKey::from_passphrase(frase)).is_err(),
        "CRITICO: (e) con la frase se importo una v8 en claro"
    );
    let _ = std::fs::remove_file(&file);
    quitar_libro(&path);
}

/// Los nombres del directorio de `file` que empiezan por el de `file`, ordenados: lo que una
/// importacion o una exportacion podria dejar a su lado. Solo esos, porque el directorio temporal
/// lo comparten los tests que corren a la vez.
fn junto_a(file: &str) -> Vec<String> {
    let p = std::path::Path::new(file);
    let nombre = p.file_name().expect("nombre").to_string_lossy().into_owned();
    let mut v: Vec<String> = std::fs::read_dir(p.parent().expect("padre"))
        .expect("leer el directorio")
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with(&nombre))
        .collect();
    v.sort();
    v
}

/// ⚠️ §819 · **Si no puede borrar el claro que dejaba una importacion muerta, la importacion falla y
/// no importa nada.** El borrado se hace fallar a el mismo (`snapshot::FALLAR_BORRADO`: se le pide
/// a `remove_file` el resto con una barra al final, y lo rehusa): el `Result` que la importacion
/// propaga es el del borrado, y un mutante que lo ignore cae aqui. Se exige que (a) la importacion
/// falle, (b) el error nombre el fichero, (c) la instantanea y el resto sigan byte a byte como
/// estaban, no aparezca nada a su lado y no se avise de un borrado que no se hizo, y (d) sin el
/// fallo, la misma importacion lo barra, lo avise -una vez, nombrandolo y diciendo que estaba en
/// claro (`snapshot::AVISOS_DEL_BARRIDO`)- e importe: la negativa era del borrado, y nada mas lo
/// impedia.
#[test]
fn si_no_puede_barrer_el_claro_la_importacion_falla_sin_importar_nada() {
    use crate::crypto::LedgerKey;
    use crate::snapshot::{AVISOS_DEL_BARRIDO, FALLAR_BORRADO};
    let avisos = || AVISOS_DEL_BARRIDO.with(|a| a.borrow_mut().split_off(0));
    let (path, alice, _) = libro_e4("e4_no_barre");
    let file = format!("{path}.instantanea");
    let resto = format!("{file}.tmp-descifrada");
    let _ = std::fs::remove_file(&file);
    let _ = std::fs::remove_file(&resto);
    let key = LedgerKey::from_passphrase(FRASE_VIEJA_E4);
    let raiz = {
        let layer = abrir_e4(&path, FRASE_VIEJA_E4).expect("abrir");
        layer.export_snapshot(&file).expect("exportar");
        layer.state_root()
    };
    const CLARO: &[u8] = b"el estado del libro en claro, de mentira";
    std::fs::write(&resto, CLARO).unwrap();
    let instantanea = std::fs::read(&file).unwrap();
    let al_lado = junto_a(&file);
    avisos();

    FALLAR_BORRADO.with(|f| f.set(true));
    let r = SovereignLayer::import_snapshot_with_key(&file, &key).map(|l| l.state_root());
    FALLAR_BORRADO.with(|f| f.set(false));
    let e = match r {
        Err(e) => e.to_string(),
        Ok(_) => panic!("CRITICO: (a) importo con el claro al lado sin poder borrarlo"),
    };
    assert!(e.contains(&resto), "(b) el error no nombra el fichero: {e}");
    assert_eq!(std::fs::read(&resto).ok().as_deref(), Some(CLARO), "(c) el resto se toco");
    assert_eq!(std::fs::read(&file).unwrap(), instantanea, "(c) la instantanea se toco");
    assert_eq!(junto_a(&file), al_lado, "(c) aparecio algo junto a la instantanea");
    assert_eq!(avisos(), Vec::<String>::new(), "CRITICO: (c) aviso de un borrado que no hizo");

    let l = SovereignLayer::import_snapshot_with_key(&file, &key).expect("(d) importar");
    assert_eq!(l.state_root(), raiz, "(d)");
    assert_eq!(l.balance_of(alice), Some(777_777), "(d)");
    assert!(!hay(&resto), "(d) sin el fallo, la importacion no lo barrio");
    let dichos = avisos();
    assert_eq!(dichos.len(), 1, "(d) el borrado no se aviso una vez: {dichos:?}");
    assert!(
        dichos[0].contains(&resto) && dichos[0].contains("EN CLARO"),
        "(d) el aviso no nombra el fichero o no dice que estaba en claro: {}",
        dichos[0]
    );
    let _ = std::fs::remove_file(&file);
    quitar_e4(&path);
}

/// El temporal de la exportacion a `file`, formado como lo forma la capa.
fn exportando(file: &str) -> String {
    format!("{file}{}", crate::snapshot::SUFIJO_EXPORTANDO)
}

/// Corre `f` con un ERROR inyectado en el punto `k` de la exportacion
/// (`snapshot::FALLAR_EN_EXPORTACION`): 1, con la mitad del temporal escrita; 2, entero y
/// sincronizado, antes del cambio de nombre; 3, despues, al sincronizar el directorio.
fn fallar_exportacion_en<T>(k: u8, f: impl FnOnce() -> T) -> T {
    use crate::snapshot::FALLAR_EN_EXPORTACION;
    FALLAR_EN_EXPORTACION.with(|m| m.set(Some(k)));
    let r = f();
    FALLAR_EN_EXPORTACION.with(|m| m.set(None));
    r
}

/// Corre `f` con una MUERTE inyectada en el punto `k` de la exportacion
/// (`snapshot::MORIR_EN_EXPORTACION`). Dice si murio. Es la aproximacion de `morir_en`: el panico
/// corre los `Drop`, y lo unico abierto es el temporal, que se cierra sin mas.
fn morir_exportacion_en(k: u8, f: impl FnOnce()) -> bool {
    use crate::snapshot::MORIR_EN_EXPORTACION;
    MORIR_EN_EXPORTACION.with(|m| m.set(Some(k)));
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
    MORIR_EN_EXPORTACION.with(|m| m.set(None));
    r.is_err()
}

/// ⚠️ §820 · **Una exportacion que falla a mitad deja la instantanea anterior entera y legible, y
/// una que acaba deja la nueva** (la 120 del `BACKLOG.md`). Hasta el §820 `export_snapshot` truncaba
/// el destino y escribia encima: un fallo a mitad dejaba una a medias y la anterior ya no estaba.
/// Con un error inyectado (1) con la mitad del temporal escrita y (2) con el temporal entero, antes
/// del cambio de nombre, el destino sigue byte a byte siendo la anterior, importa con su raiz, y no
/// queda nada a su lado. (3) Uno al sincronizar el directorio, ya cambiado el nombre, deja la nueva
/// y lo dice. (4) Sin fallos, el destino es la nueva y el temporal no queda.
#[test]
fn una_exportacion_que_falla_a_mitad_deja_la_anterior_entera() {
    let file = format!("{}.instantanea", temp_path("export_falla"));
    let tmp = exportando(&file);
    let _ = std::fs::remove_file(&file);
    let _ = std::fs::remove_file(&tmp);
    let mut layer = new_layer();
    open_and_fund(&mut layer, SK_ALICE, 1_000_000);
    layer.export_snapshot(&file).expect("la anterior");
    let anterior = std::fs::read(&file).unwrap();
    let raiz_anterior = layer.state_root();
    open_and_fund(&mut layer, SK_BOB, 500);
    let raiz_nueva = layer.state_root();
    assert_ne!(raiz_nueva, raiz_anterior, "premisa: la nueva es otra");
    let solo = junto_a(&file);
    assert_eq!(solo.len(), 1, "premisa: solo el destino");

    for k in [1u8, 2] {
        let r = fallar_exportacion_en(k, || layer.export_snapshot(&file).map(|_| ()));
        assert!(r.is_err(), "({k}) el fallo inyectado no llego a la exportacion");
        assert_eq!(
            std::fs::read(&file).ok().as_deref(),
            Some(&anterior[..]),
            "CRITICO: ({k}) una exportacion que fallo a mitad toco la anterior"
        );
        let l = SovereignLayer::import_snapshot(&file).expect("la anterior importa");
        assert_eq!(l.state_root(), raiz_anterior, "({k})");
        assert!(!hay(&tmp), "({k}) el fallo dejo el temporal");
        assert_eq!(junto_a(&file), solo, "({k}) quedo algo junto al destino");
    }

    let e = fallar_exportacion_en(3, || layer.export_snapshot(&file).map(|_| ()))
        .expect_err("(3) el fallo inyectado no llego a la exportacion")
        .to_string();
    assert!(e.contains("en su sitio"), "(3) no dice que la nueva esta en su sitio: {e}");
    let l = SovereignLayer::import_snapshot(&file).expect("(3) la nueva importa");
    assert_eq!(l.state_root(), raiz_nueva, "(3)");
    assert!(!hay(&tmp), "(3)");

    std::fs::write(&file, &anterior).unwrap();
    layer.export_snapshot(&file).expect("(4) exportar");
    let l = SovereignLayer::import_snapshot(&file).expect("(4) la nueva importa");
    assert_eq!(l.state_root(), raiz_nueva, "(4) la exportacion no dejo la nueva");
    assert_eq!(junto_a(&file), solo, "(4) quedo algo junto al destino");
    let _ = std::fs::remove_file(&file);
}

/// ⚠️ §820 · **Una exportacion muerta a mitad deja la anterior entera, y su temporal no se confunde
/// con ella.** Con una capa cifrada: (a) una muerte con la mitad del temporal escrita deja el
/// destino byte a byte como estaba, importando con la frase; el temporal queda, con la misma marca y
/// cabecera que el destino -cifrado como el, sin el saldo en claro-, 0600 en Unix, y no importa.
/// (b) La exportacion siguiente lo encuentra, rehusa nombrandolo y no toca ni el destino ni el
/// temporal: no sabe si es de una viva. (c) Borrado a mano, exporta; y una muerte con el temporal
/// entero, antes del cambio de nombre, tampoco toca el destino.
#[test]
fn una_exportacion_muerta_a_mitad_deja_la_anterior_y_su_temporal_no_se_confunde() {
    use crate::crypto::LedgerKey;
    let (path, _, _) = libro_e4("export_muere");
    let file = format!("{path}.instantanea");
    let tmp = exportando(&file);
    let _ = std::fs::remove_file(&file);
    let _ = std::fs::remove_file(&tmp);
    let key = LedgerKey::from_passphrase(FRASE_VIEJA_E4);
    let mut layer = abrir_e4(&path, FRASE_VIEJA_E4).expect("abrir");
    layer.export_snapshot(&file).expect("la anterior");
    let anterior = std::fs::read(&file).unwrap();
    let raiz_anterior = layer.state_root();
    const SALDO: u64 = 0x05A3_B7C9;
    open_and_fund(&mut layer, 0xC4A0, SALDO);

    assert!(morir_exportacion_en(1, || drop(layer.export_snapshot(&file))), "(a) no murio");
    assert_eq!(
        std::fs::read(&file).ok().as_deref(),
        Some(&anterior[..]),
        "CRITICO: (a) una exportacion muerta a mitad toco la anterior"
    );
    let l = SovereignLayer::import_snapshot_with_key(&file, &key).expect("(a) la anterior importa");
    assert_eq!(l.state_root(), raiz_anterior, "(a)");
    let medio = std::fs::read(&tmp).expect("(a) la muerte no dejo el temporal");
    let cabecera = 1 + crate::crypto::LARGO_CABECERA;
    assert_eq!(medio[..cabecera], anterior[..cabecera], "(a) el temporal no va cifrado como el destino");
    let patron = &SALDO.to_le_bytes()[..4];
    assert!(!medio.windows(4).any(|w| w == patron), "CRITICO: (a) el temporal lleva el saldo en claro");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let modo = std::fs::metadata(&tmp).unwrap().permissions().mode() & 0o777;
        assert_eq!(modo, 0o600, "(a) el temporal no nacio 0600");
    }
    assert!(
        SovereignLayer::import_snapshot_with_key(&tmp, &key).is_err(),
        "CRITICO: (a) el temporal a medias importo"
    );

    let e = layer.export_snapshot(&file).map(|_| ()).expect_err("(b) exporto con el temporal al lado");
    let e = e.to_string();
    assert!(e.contains(&tmp), "(b) el error no nombra el temporal: {e}");
    assert_eq!(std::fs::read(&file).unwrap(), anterior, "(b) toco el destino");
    assert_eq!(std::fs::read(&tmp).unwrap(), medio, "(b) toco el temporal");

    std::fs::remove_file(&tmp).unwrap();
    layer.export_snapshot(&file).expect("(c) exportar");
    let actual = std::fs::read(&file).unwrap();
    assert!(!hay(&tmp), "(c)");
    open_and_fund(&mut layer, 0xC4A1, 1);
    assert!(morir_exportacion_en(2, || drop(layer.export_snapshot(&file))), "(c) no murio");
    assert_eq!(std::fs::read(&file).unwrap(), actual, "CRITICO: (c) la muerte antes del cambio de nombre toco el destino");
    drop(layer);
    let _ = std::fs::remove_file(&tmp);
    let _ = std::fs::remove_file(&file);
    quitar_e4(&path);
}

/// ⚠️ §820 · **La exportacion no abre lo que la escritura en sitio cerraba, salvo lo que cerrara una
/// ACL.** Cambiar de nombre un
/// temporal por el destino le daria el dueno, el grupo y los permisos del temporal, y sustituiria
/// un enlace -o un FIFO, un socket o un dispositivo- en vez de escribir a traves de el; un
/// directorio ya lo rehusa el cambio de nombre (`EISDIR`). En Unix: (a) una instantanea nueva nace
/// 0600; (b) una que sustituye a otra se queda con los bits de modo de la anterior, como en sitio; (c)
/// un enlace como destino se rehusa, el enlace se queda y su destino no se toca; (d) un directorio
/// tampoco -lo rehusaria igual el cambio de nombre: (d) no distingue la guarda, y (c) si-; (e) con
/// privilegios, la nueva se queda con el dueno y el grupo de la anterior; y (f) si no puede tomar su
/// grupo (`snapshot::FALLAR_DUENO`), sin los bits de grupo ni de otros. (e) pide cambiar el dueno
/// de un fichero: sin privilegios no se puede preparar, y lo dice por la salida de errores. No mide
/// las ACL POSIX de la anterior, que la nueva no toma (`dar_lo_de_la_anterior`).
#[test]
fn la_exportacion_guarda_los_permisos_de_la_anterior_y_no_sustituye_lo_que_no_es_un_fichero() {
    let file = format!("{}.instantanea", temp_path("export_modos"));
    let _ = std::fs::remove_file(&file);
    let _ = std::fs::remove_file(exportando(&file));
    let mut layer = new_layer();
    open_and_fund(&mut layer, SK_ALICE, 1_000);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let modo = |p: &str| std::fs::metadata(p).unwrap().permissions().mode() & 0o777;
        layer.export_snapshot(&file).expect("(a) exportar");
        assert_eq!(modo(&file), 0o600, "(a) una instantanea nueva no nacio 0600");
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o640)).unwrap();
        layer.export_snapshot(&file).expect("(b) exportar");
        assert_eq!(modo(&file), 0o640, "(b) la nueva no conserva los permisos de la anterior");

        let destino = format!("{file}.destino");
        let enlace = format!("{file}.enlace");
        let _ = std::fs::remove_file(&enlace);
        std::fs::write(&destino, b"del operador").unwrap();
        std::os::unix::fs::symlink(&destino, &enlace).unwrap();
        let e = layer.export_snapshot(&enlace).map(|_| ()).expect_err("CRITICO: (c) exporto sobre un enlace");
        assert!(e.to_string().contains(&enlace), "(c) el error no nombra el enlace: {e}");
        assert!(
            std::fs::symlink_metadata(&enlace).map(|m| m.file_type().is_symlink()).unwrap_or(false),
            "(c) el enlace no se quedo"
        );
        assert_eq!(std::fs::read(&destino).ok().as_deref(), Some(&b"del operador"[..]), "(c) se toco su destino");
        assert!(!hay(&exportando(&enlace)), "(c) quedo un temporal");
        let _ = std::fs::remove_file(&enlace);
        let _ = std::fs::remove_file(&destino);
    }
    let dir = format!("{file}.dir");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir(&dir).unwrap();
    let e = layer.export_snapshot(&dir).map(|_| ()).expect_err("(d) exporto sobre un directorio");
    assert!(e.to_string().contains(&dir), "(d) el error no nombra el directorio: {e}");
    assert!(std::fs::read_dir(&dir).map(|mut d| d.next().is_none()).unwrap_or(false), "(d) se toco el directorio");
    assert!(!hay(&exportando(&dir)), "(d) quedo un temporal");
    let _ = std::fs::remove_dir_all(&dir);
    #[cfg(unix)]
    {
        use crate::snapshot::FALLAR_DUENO;
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        let modo = |p: &str| std::fs::metadata(p).unwrap().permissions().mode() & 0o777;
        let dueno = |p: &str| {
            let m = std::fs::metadata(p).unwrap();
            (m.uid(), m.gid())
        };
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o640)).unwrap();
        if std::os::unix::fs::chown(&file, Some(4242), Some(4343)).is_ok() {
            layer.export_snapshot(&file).expect("(e) exportar");
            assert_eq!(dueno(&file), (4242, 4343), "CRITICO: (e) la nueva no es del dueno y del grupo de la anterior");
            assert_eq!(modo(&file), 0o640, "(e)");
        } else {
            eprintln!("(e) sin medir: cambiar el dueno de un fichero pide privilegios");
        }
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o644)).unwrap();
        FALLAR_DUENO.with(|c| c.set(true));
        let r = layer.export_snapshot(&file).map(|_| ());
        FALLAR_DUENO.with(|c| c.set(false));
        r.expect("(f) exportar");
        assert_eq!(
            modo(&file),
            0o600,
            "CRITICO: (f) sin poder tomar el grupo de la anterior, la nueva conserva bits de grupo u otros"
        );
        assert!(!hay(&exportando(&file)), "(f) quedo un temporal");
    }
    let _ = std::fs::remove_file(&file);
}

/// ⚠️ §821 · Lo que una importacion tiene que conservar de la capa que exporto la instantanea: sus
/// cinco raices -cuentas, congeladas, consumos, custodios y gobernanza-, su suministro y los saldos
/// de `cuentas`.
fn lo_que_conserva(l: &SovereignLayer, cuentas: &[AccountIndex]) -> (Vec<Digest>, u64, Vec<Option<u64>>) {
    (
        vec![l.state_root(), l.frozen_root(), l.cons_root(), l.custodian_set_root(), l.governance_set_root()],
        l.total_supply(),
        cuentas.iter().map(|&c| l.balance_of(c)).collect(),
    )
}

/// ⚠️ §821 · **Una capa importada de una instantanea cifrada conserva su clave, y se reexporta
/// cifrada, con la misma cabecera** (la 121 del `BACKLOG.md`). Hasta el §821 nacia sin clave, y
/// `export_snapshot` la reexportaba en claro, con la marca `0x00`. Un libro cifrado exporta su
/// instantanea, con la marca `0x02` y la cabecera del libro; importada con su frase y vuelta a
/// exportar, (a) sale con la marca `0x02` y la MISMA cabecera, byte a byte -la del libro: la clave
/// sigue fijada a su sal-; (b) sin clave no se importa, ni con otra frase; (c) con la misma frase se
/// importa, con los saldos, el suministro y las raices de la capa que la exporto; y (d) la cadena
/// sigue: la importada de la reexportada se reexporta otra vez cifrada y con la misma cabecera, e
/// importa con la frase. (e) Con la clave ya fijada a ese KDF (`LedgerKey::derivar`), que no lleva la
/// frase, tambien: no le hace falta, porque la clave que conserva la capa es la que abrio la
/// instantanea. (f) Y la clave no se pierde al operar: la capa importada abre una cuenta -lo que
/// pasa por `commit`, que sin libro no sella nada- y se reexporta otra vez cifrada, con la misma
/// cabecera, e importa con la frase con la cuenta nueva.
#[test]
fn una_capa_importada_de_una_cifrada_se_reexporta_cifrada_con_su_cabecera() {
    use crate::crypto::{LedgerKey, LARGO_CABECERA};
    const MARCA_V2: u8 = 0x02; // `SNAPSHOT_ENCRYPTED_V2`, privada de snapshot.rs
    let (path, alice, bob) = libro_e4("e4_reexporta_121");
    let file = format!("{path}.instantanea");
    let otra = format!("{path}.reexportada");
    for f in [&file, &otra] {
        let _ = std::fs::remove_file(f);
        let _ = std::fs::remove_file(exportando(f));
    }
    let de_partida = {
        let layer = abrir_e4(&path, FRASE_VIEJA_E4).expect("abrir");
        layer.export_snapshot(&file).expect("exportar");
        lo_que_conserva(&layer, &[alice, bob])
    };
    let cabecera = cabecera_de(&path);
    let original = std::fs::read(&file).expect("leer");
    assert_eq!(original[0], MARCA_V2, "la del libro va con la marca 0x02");
    assert_eq!(&original[1..1 + LARGO_CABECERA], &cabecera[..], "y con la cabecera del libro");
    let frase = || LedgerKey::from_passphrase(FRASE_VIEJA_E4);

    let importada = SovereignLayer::import_snapshot_with_key(&file, &frase()).expect("importar");
    assert_eq!(lo_que_conserva(&importada, &[alice, bob]), de_partida, "la importada");
    importada.export_snapshot(&otra).expect("(a) reexportar");
    let re = std::fs::read(&otra).expect("(a) leer");
    assert_eq!(re[0], MARCA_V2, "CRITICO: (a) la reexportada no va cifrada en la version 2: marca {:#04x}", re[0]);
    assert_eq!(
        &re[1..1 + LARGO_CABECERA],
        &cabecera[..],
        "CRITICO: (a) la reexportada no lleva la cabecera de la instantanea: la clave no siguio fijada a su sal"
    );
    assert_ne!(re, original, "(a) la reexportada es la misma copia: el sellado lleva nonce nuevo");

    assert!(SovereignLayer::import_snapshot(&otra).is_err(), "CRITICO: (b) la reexportada se importo sin clave");
    assert!(
        SovereignLayer::import_snapshot_with_key(&otra, &LedgerKey::from_passphrase(FRASE_NUEVA_E4)).is_err(),
        "CRITICO: (b) la reexportada se importo con otra frase"
    );
    let r = SovereignLayer::import_snapshot_with_key(&otra, &frase()).expect("(c) con la frase se importa");
    assert_eq!(lo_que_conserva(&r, &[alice, bob]), de_partida, "(c)");

    r.export_snapshot(&otra).expect("(d) reexportar la importada de la reexportada");
    let otra_vez = std::fs::read(&otra).expect("(d) leer");
    assert_eq!(otra_vez[0], MARCA_V2, "CRITICO: (d) marca {:#04x}", otra_vez[0]);
    assert_eq!(&otra_vez[1..1 + LARGO_CABECERA], &cabecera[..], "CRITICO: (d) otra cabecera");
    let r = SovereignLayer::import_snapshot_with_key(&otra, &frase()).expect("(d) importar");
    assert_eq!(lo_que_conserva(&r, &[alice, bob]), de_partida, "(d)");

    let fijada = LedgerKey::derivar(FRASE_VIEJA_E4, &crate::crypto::Kdf::de_cabecera(&cabecera).unwrap()).unwrap();
    let r = SovereignLayer::import_snapshot_with_key(&file, &fijada).expect("(e) con la clave ya fijada se importa");
    r.export_snapshot(&otra).expect("(e) reexportar");
    let con_fijada = std::fs::read(&otra).expect("(e) leer");
    assert_eq!(con_fijada[0], MARCA_V2, "CRITICO: (e) marca {:#04x}", con_fijada[0]);
    assert_eq!(&con_fijada[1..1 + LARGO_CABECERA], &cabecera[..], "CRITICO: (e) otra cabecera");
    let r = SovereignLayer::import_snapshot_with_key(&otra, &frase()).expect("(e) importar con la frase");
    assert_eq!(lo_que_conserva(&r, &[alice, bob]), de_partida, "(e)");

    let mut op = SovereignLayer::import_snapshot_with_key(&file, &frase()).expect("(f) importar");
    let carol = op.open_account(BaseElement::new(0xC121));
    let tras = lo_que_conserva(&op, &[alice, bob, carol]);
    assert_ne!(tras.0, de_partida.0, "(f) la cuenta nueva no movio ninguna raiz");
    op.export_snapshot(&otra).expect("(f) reexportar tras operar");
    let tras_operar = std::fs::read(&otra).expect("(f) leer");
    assert_eq!(tras_operar[0], MARCA_V2, "CRITICO: (f) tras operar, la reexportada no va cifrada: marca {:#04x}", tras_operar[0]);
    assert_eq!(&tras_operar[1..1 + LARGO_CABECERA], &cabecera[..], "CRITICO: (f) tras operar, otra cabecera");
    let r = SovereignLayer::import_snapshot_with_key(&otra, &frase()).expect("(f) importar con la frase");
    assert_eq!((lo_que_conserva(&r, &[alice, bob, carol]), r.account_count()), (tras, 3), "(f)");
    for f in [&file, &otra] {
        let _ = std::fs::remove_file(f);
    }
    quitar_e4(&path);
}

/// ⚠️ §821 · **Una capa importada de una instantanea cifrada de la version 1 se reexporta en la
/// version 2, con sal nueva y la misma frase; con su clave ya fijada, que no lleva la frase, no se
/// importa** (la 121 del `BACKLOG.md`; llevarla a la v2, y fallar sin la frase, es de la sesion,
/// reversible). La v1 se fabrica con la regla documentada, como en
/// `una_instantanea_cifrada_de_la_version_1_se_sigue_importando`: la marca `0x01` y el cuerpo sellado
/// con la clave de SHA-256 sin sal. Importada con su frase y vuelta a exportar, (a) sale con la marca
/// `0x02` y una cabecera de la version 2 que se lee; (b) sin clave no se importa, ni con otra frase;
/// (c) con la misma frase se importa, con los saldos, el suministro y las raices de la de partida; y
/// (d) otra exportacion de la misma capa lleva la misma cabecera: la sal se fija al importar, no en
/// cada exportacion. (e) Con la clave de la v1 ya fijada (`LedgerKey::derivar`), que no lleva la
/// frase, la importacion falla, lo dice, y no toca la instantanea. (f) Y la sal es NUEVA en cada
/// importacion: otra importacion de la misma v1, reexportada, lleva otra cabecera, y abre con la
/// frase; con una sal fija, todas las v1 llevadas a la v2 compartirian sal.
#[test]
fn una_capa_importada_de_una_v1_se_reexporta_en_la_version_2() {
    use crate::crypto::{Kdf, LedgerKey, LARGO_CABECERA};
    const MARCA_EN_CLARO: u8 = 0x00; // `SNAPSHOT_PLAIN`, privada de snapshot.rs
    const MARCA_V1: u8 = 0x01; // `SNAPSHOT_ENCRYPTED`
    const MARCA_V2: u8 = 0x02; // `SNAPSHOT_ENCRYPTED_V2`
    const FRASE: &str = "la frase de una instantanea v1";
    let file = format!("{}.instantanea", temp_path("v1_reexporta_121"));
    let otra = format!("{file}.reexportada");
    for f in [&file, &otra] {
        let _ = std::fs::remove_file(f);
        let _ = std::fs::remove_file(exportando(f));
    }
    let mut layer = new_layer();
    let alice = open_and_fund(&mut layer, SK_ALICE, 121_121);
    let bob = open_and_fund(&mut layer, SK_BOB, 0);
    let de_partida = lo_que_conserva(&layer, &[alice, bob]);
    layer.export_snapshot(&file).expect("exportar en claro");
    let claro = std::fs::read(&file).expect("leer");
    assert_eq!(claro[0], MARCA_EN_CLARO);
    let v1 = LedgerKey::derivar(FRASE, &Kdf::Sha256V1).unwrap();
    let mut viejo = vec![MARCA_V1];
    viejo.extend_from_slice(&v1.seal(&claro[1..]).unwrap());
    std::fs::write(&file, &viejo).unwrap();
    let frase = || LedgerKey::from_passphrase(FRASE);

    let importada = SovereignLayer::import_snapshot_with_key(&file, &frase()).expect("la v1 se importa con su frase");
    assert_eq!(lo_que_conserva(&importada, &[alice, bob]), de_partida, "la importada");
    importada.export_snapshot(&otra).expect("(a) reexportar");
    let re = std::fs::read(&otra).expect("(a) leer");
    assert_eq!(re[0], MARCA_V2, "CRITICO: (a) la reexportada de una v1 no va en la version 2: marca {:#04x}", re[0]);
    let kdf = Kdf::de_cabecera(&re[1..1 + LARGO_CABECERA]).expect("(a) la cabecera no es de la version 2");
    assert_eq!(kdf.version(), 2, "(a)");

    assert!(SovereignLayer::import_snapshot(&otra).is_err(), "CRITICO: (b) la reexportada se importo sin clave");
    assert!(
        SovereignLayer::import_snapshot_with_key(&otra, &LedgerKey::from_passphrase("otra frase")).is_err(),
        "CRITICO: (b) la reexportada se importo con otra frase"
    );
    let r = SovereignLayer::import_snapshot_with_key(&otra, &frase()).expect("(c) con la frase se importa");
    assert_eq!(lo_que_conserva(&r, &[alice, bob]), de_partida, "(c)");

    importada.export_snapshot(&otra).expect("(d) reexportar otra vez");
    let otra_vez = std::fs::read(&otra).expect("(d) leer");
    assert_eq!(otra_vez[..1 + LARGO_CABECERA], re[..1 + LARGO_CABECERA], "(d) otra exportacion de la misma capa, otra sal");

    match SovereignLayer::import_snapshot_with_key(&file, &v1) {
        Ok(l) => {
            let marca = l.export_snapshot(&otra).ok().and_then(|_| std::fs::read(&otra).ok()).map(|v| v[0]);
            panic!("CRITICO: (e) con la clave de la v1 ya fijada, sin la frase, se importo; reexportada, con la marca {marca:?}")
        }
        Err(e) => {
            let e = e.to_string();
            assert!(e.contains("version 1") && e.contains("frase"), "(e) el error no dice que es la v1 ni que falta la frase: {e}");
        }
    }
    assert_eq!(std::fs::read(&file).expect("(e) leer"), viejo, "(e) la instantanea se toco");

    let segunda = SovereignLayer::import_snapshot_with_key(&file, &frase()).expect("(f) importar otra vez");
    segunda.export_snapshot(&otra).expect("(f) reexportar");
    let de_la_segunda = std::fs::read(&otra).expect("(f) leer");
    assert_eq!(de_la_segunda[0], MARCA_V2, "(f) marca {:#04x}", de_la_segunda[0]);
    assert_ne!(
        de_la_segunda[1..1 + LARGO_CABECERA],
        re[1..1 + LARGO_CABECERA],
        "CRITICO: (f) dos importaciones de la misma v1, la misma cabecera: la sal no es nueva"
    );
    let r = SovereignLayer::import_snapshot_with_key(&otra, &frase()).expect("(f) importar con la frase");
    assert_eq!(lo_que_conserva(&r, &[alice, bob]), de_partida, "(f)");
    for f in [&file, &otra] {
        let _ = std::fs::remove_file(f);
    }
}
