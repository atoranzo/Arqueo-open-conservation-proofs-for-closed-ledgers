//! Recuperación de cuenta asistida por custodios.
//!
//! Corrige la única carencia del sistema que causaba **pérdida
//! irreversible**: si una clave de gasto se comprometía, el dinero de esa
//! cuenta se perdía para siempre.
//!
//! ## ⚠️ El intercambio
//!
//! Se cambia *"pérdida irreversible si te roban la clave"* por *"dos
//! custodios pueden reasignar cualquier cuenta"*.
//!
//! Es el intercambio correcto para un sistema bancario —un banco puede
//! reasignar bajo orden judicial— **pero solo si es visible**. Por eso la
//! capa mantiene un **contador público de recuperaciones**, atado en el
//! circuito, que incrementa en cada una.
//!
//! El contador no impide el abuso: nada en un circuito puede. Lo hace
//! **contable**, que es la condición para que exista rendición de
//! cuentas.
//!
//! ## La API pide la identidad nueva, no la clave nueva
//!
//! `recover` recibe el `public_id` del nuevo titular, no su clave. El
//! nuevo dueño genera su clave y entrega solo la identidad derivada — la
//! capa nunca la ve, igual que no ve ninguna otra clave de gasto.

use super::*;

impl SovereignLayer {
    /// Contador público de recuperaciones.
    ///
    /// Cada intervención de los custodios lo incrementa. Una discrepancia
    /// entre este número y las recuperaciones justificadas es detectable.
    pub fn recovery_count(&self) -> u64 {
        self.recovery_count
    }

    /// Recupera una cuenta **sin que las claves de custodio lleguen al
    /// operador**: la via de la entrada 32/33 (65).
    ///
    /// Tres pruebas: `climb_proof` de `circuit_recovery_climb` -que la hoja
    /// vieja y la nueva suben a las dos raices con el mismo camino, con el
    /// MISMO saldo y el nonce incrementado- y dos de custodios distintos que
    /// autorizan ESTA transicion.
    ///
    /// # Que prueba el circuito y que comprueba la capa
    ///
    /// El circuito prueba lo que un auditor externo no puede recomputar: que
    /// el saldo se conserva. La capa recomputa la raiz por su cuenta -tiene
    /// el arbol- asi que para ella la prueba es redundante, igual que en
    /// `freeze` (60.3). Su valor es para el registro.
    pub fn apply_recovery_delegated(
        &mut self,
        climb_proof: winterfell::Proof,
        proof_a: winterfell::Proof,
        inputs_a: stark_experiment::circuit_threshold_single_nullifier::NullifierThresholdPublicInputs,
        proof_b: winterfell::Proof,
        inputs_b: stark_experiment::circuit_threshold_single_nullifier::NullifierThresholdPublicInputs,
        account_index: AccountIndex,
        new_public_id: Digest,
    ) -> Result<(), LayerError> {
        use stark_experiment::circuit_recovery_climb::{
            RecoveryClimbAir, RecoveryClimbPublicInputs,
        };
        use stark_experiment::circuit_threshold::CUSTODIAN_DOMAIN;
        use stark_experiment::circuit_threshold_single_nullifier::{
            commit_operation, verify_threshold_pair, PairRejection, OP_RECOVERY,
        };

        let account = self
            .records
            .get(&account_index)
            .ok_or(LayerError::AccountNotFound(account_index))?
            .clone();

        // ⚠️ La vía-recibo rechaza esto en la GENERACIÓN (:72). La
        // delegada NO tenía puerta — ni aquí ni en el circuito: §169 lo
        // demostró con el rojo del relevo — y aceptaba un no-cambio que
        // gasta cupo, avanza contador y quema nonce. La puerta, calcada
        // de la de gobernanza: rechaza ANTES de verificar.
        if new_public_id == account.public_id {
            return Err(LayerError::RecoveryToSameIdentity);
        }

        let root_old = self.accounts.root();
        let count_old = self.recovery_count;
        let count_new = count_old + 1;

        let updated = AccountRecord {
            public_id: new_public_id,
            balance: account.balance,
            nonce: account.nonce + BaseElement::ONE,
                // ⚠️ COSTURA 49-A <-> 52 (rotación): se copia el view_id
                // ANTERIOR, pero la clave de gasto acaba de rotar y el
                // view_id deriva de ella (§127), así que el recuperado
                // conserva una credencial de vista que su clave NUEVA ya
                // no reproduce. La capa no puede recalcularlo: no tiene la
                // clave (§93.4). El cierre correcto —traer el view_id nuevo
                // en el receipt de recuperación— es diseño de la rotación
                // (entrada 52), NO de 49-A. Limitación declarada y con test
                // (`recovery_deja_view_id_viejo`), no agujero silencioso.
                view_id: account.view_id,
                // MISMA costura para el salt: deriva de la clave que rota,
                // asi que el salt viejo ya no corresponde. Se copia con el
                // view_id; el cierre es la entrada 52.
                leaf_salt: account.leaf_salt,
        };
        let mut tentativo = self.accounts.clone();
        tentativo.set_leaf(
            account_index,
            native_leaf_salted(
                updated.public_id,
                BaseElement::new(updated.balance),
                updated.nonce,
                updated.leaf_salt,
            ),
        );
        let root_new = tentativo.root();

        let accepted = AcceptableOptions::OptionSet(vec![self.options.clone()]);

        verify::<RecoveryClimbAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            climb_proof,
            RecoveryClimbPublicInputs {
                root_old,
                root_new,
                recovery_count_old: BaseElement::new(count_old),
                recovery_count_new: BaseElement::new(count_new),
            },
            &accepted,
        )
        .map_err(|e| LayerError::VerificationFailed(format!("subida de recuperacion: {e:?}")))?;

        let mut params: Vec<BaseElement> = root_old.to_vec();
        params.extend_from_slice(&root_new);
        params.push(BaseElement::new(count_old));
        params.push(BaseElement::new(count_new));
        let operation = commit_operation(OP_RECOVERY, &params);

        verify_threshold_pair(
            proof_a, inputs_a, proof_b, inputs_b,
            // Custodios: recuperar es custodia, no cambiar quien custodia.
            BaseElement::new(CUSTODIAN_DOMAIN),
            self.custodian_set_root,
            operation,
            &accepted,
        )
        .map_err(|r| match r {
            PairRejection::SameCustodian
            | PairRejection::WrongCustodianSet
            | PairRejection::WrongIdentityDomain => LayerError::NotTheIssuer,
            PairRejection::WrongOperation => LayerError::StaleState,
            PairRejection::InvalidProof => {
                LayerError::VerificationFailed("autorizacion invalida".into())
            }
        })?;

        // Despues de verificar la autoridad: si fuera antes, cualquiera
        // podria agotar el cupo sin ser custodio.
        self.consume_custodian_use()?;

        self.accounts = tentativo;
        self.records.insert(account_index, updated);
        self.recovery_count = count_new;

        self.log.append(OpKind::Recovery, root_old, root_new, &[]);
        // ⚠️ **El registro VIAJA en el lote, como en la vía-recibo.** Con
        // `&[]` la raíz rotada llegaba al disco sin el AccountRecord: al
        // reabrir, la reconstrucción no casaba con `root:state` y la
        // integridad —fail-closed— detenía un ledger LEGÍTIMO. Lo destapó
        // el test persistente girado en B-3a-ii (§171).
        self.commit(&[account_index], None)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests_delegada {
    use super::*;
    use crate::tests_support::*;
    use stark_experiment::circuit_recovery_climb as climb;
    use stark_experiment::circuit_threshold::{build_custodian_set, CUSTODIAN_DOMAIN};
    use stark_experiment::circuit_threshold_single_nullifier as auth;

    const LIMIT: u64 = 1_000_000;
    const MAX_SUPPLY: u64 = 1_000_000_000;
    const MAX_ACCOUNTS: u64 = 1_000;

    /// ⚠️ **UNA CUENTA ESTRECHA PUEDE ROTAR A CLAVE DE 256 BITS.**
    ///
    /// Es lo que decide si la entrada 15 se puede cerrar para las cuentas
    /// **que ya existen**. `open_account_wide` sirve para las nuevas; esta
    /// via es la unica salida para las viejas.
    ///
    /// Y funciona porque `recover` toma la identidad **ya derivada** —un
    /// `Digest`— y **no comprueba su formato**: al titular le basta derivar
    /// `derive_public_id_wide(sk_ancha)` en su maquina.
    ///
    /// ⚠️ **Pero exige DOS CUSTODIOS.** Rotar a clave ancha **no es una
    /// accion soberana del titular**: necesita autorizacion de terceros. Eso
    /// contradice el espiritu del resto del diseño —la clave nunca sale de
    /// su maquina, pero **cambiarla depende de otros**— y no se resuelve
    /// aqui: se registra (§98).
    #[test]
    fn a_narrow_account_can_rotate_to_a_256_bit_key() {
        use stark_experiment::circuit_settlement::derive_public_id_wide;

        let mut layer = new_layer();
        #[allow(deprecated)]
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund_wide(&mut layer, wide_key(SK_BOB), 0);

        // La clave nueva, ancha de verdad. **El titular la genera y la
        // deriva; la capa solo ve la identidad.**
        let sk_ancha = wide_key(0x1207A);
        let id_nueva = derive_public_id_wide(sk_ancha);

        // ⚠️ **Por la via DELEGADA**, no la antigua.
        //
        // El primer intento uso `recover`, que esta marcada `#[deprecated]`
        // desde §65: exige las claves de custodio EN EL OPERADOR, que es el
        // fallo de la entrada 32. Habria demostrado que se puede rotar **por
        // un camino que el proyecto quiere retirar**, y al retirarlo se
        // habria perdido la evidencia de que las cuentas viejas tienen
        // salida.
        //
        // Las dos vias toman `new_public_id: Digest`, asi que la propiedad
        // es la misma — pero la que se demuestra debe ser la que se queda.
        let ck = custodian_keys();
        let (_, cp) = build_custodian_set(&ck);
        let op = compromiso(
            layer.accounts.root(),
            raiz_nueva(&layer, alice, id_nueva),
            layer.recovery_count(),
        );
        let subida = prueba_subida(&layer, alice, id_nueva);
        let (pa, ia) = autorizar(ck[1], &cp[1], op);
        let (pb, ib) = autorizar(ck[3], &cp[3], op);
        layer
            .apply_recovery_delegated(subida, pa, ia, pb, ib, alice, id_nueva)
            .expect("la recuperacion delegada debe aplicarse");

        assert_eq!(
            layer.public_id_of(alice),
            Some(id_nueva),
            "la cuenta debe tener ya la identidad ancha"
        );

        // ⚠️ Y lo que de verdad importa: que DESPUES pueda pagar con ella.
        let receptor = layer.public_id_of(bob).expect("cuenta");
        let m = layer
            .send_materials(alice, receptor, 250_000, salt_de(0x1207))
            .expect("materiales");
        let recibo = crate::client::prove_send(&m, sk_ancha, proof_options())
            .expect("tras rotar, el titular DEBE poder probar con su clave ancha");
        let estado = state_of(&layer, alice);
        layer
            .apply_send(&recibo, alice, &estado, 250_000)
            .expect("aplicar envio");

        assert_eq!(layer.balance_of(alice), Some(750_000));
    }

    /// **REENVIAR una recuperación delegada se rechaza**: el compromiso
    /// ata raíces y contador; aplicado una vez, ambos avanzaron y los
    /// mismos materiales quedan huérfanos. Releva en B-3 a
    /// `replaying_a_recovery_is_rejected` y el mecanismo de
    /// `the_recovery_counter_survives_restart` (vía-recibo, §169).
    #[test]
    fn replaying_a_delegated_recovery_is_rejected() {
        let (mut layer, idx, nueva) = capa();
        let ck = custodian_keys();
        let (_, cp) = build_custodian_set(&ck);

        let op = compromiso(
            layer.accounts.root(), raiz_nueva(&layer, idx, nueva), layer.recovery_count(),
        );
        let subida = prueba_subida(&layer, idx, nueva);
        let (pa, ia) = autorizar(ck[1], &cp[1], op);
        let (pb, ib) = autorizar(ck[3], &cp[3], op);
        layer
            .apply_recovery_delegated(
                subida.clone(), pa.clone(), ia.clone(), pb.clone(), ib.clone(), idx, nueva,
            )
            .expect("la primera recuperacion");
        let cuenta = layer.recovery_count();

        // Los MISMOS materiales, otra vez.
        let r = layer.apply_recovery_delegated(subida, pa, ia, pb, ib, idx, nueva);
        assert!(r.is_err(), "CRITICO: reenviar rotaria la identidad de nuevo: {r:?}");
        assert_eq!(layer.recovery_count(), cuenta, "el contador no avanza");
        assert_eq!(layer.public_id_of(idx), Some(nueva), "la identidad queda donde estaba");
    }

    /// **RECUPERAR A LA MISMA IDENTIDAD, por la delegada — el rojo que
    /// valió una puerta**: la teoría decía «el circuito la impone»; el
    /// relevo la desmintió con un Ok(()) — ni capa ni circuito la
    /// tenían, y el no-cambio gastaba cupo, contador y nonce. La puerta
    /// vive ahora en el apply (calco de la de gobernanza) y este test
    /// la vigila: rechazo ANTES de verificar, sin gastar nada. Releva
    /// en B-3 a `recovery_to_the_same_identity_is_rejected` (§169).
    #[test]
    fn delegated_recovery_to_the_same_identity_is_rejected() {
        let (mut layer, idx, _) = capa();
        let misma = layer.public_id_of(idx).expect("cuenta");
        let ck = custodian_keys();
        let (_, cp) = build_custodian_set(&ck);

        let op = compromiso(
            layer.accounts.root(), raiz_nueva(&layer, idx, misma), layer.recovery_count(),
        );
        let subida = prueba_subida(&layer, idx, misma);
        let (pa, ia) = autorizar(ck[1], &cp[1], op);
        let (pb, ib) = autorizar(ck[3], &cp[3], op);

        let r = layer.apply_recovery_delegated(subida, pa, ia, pb, ib, idx, misma);
        assert!(
            matches!(r, Err(LayerError::RecoveryToSameIdentity)),
            "CRITICO: rotar hacia la misma identidad: {r:?}"
        );
        assert_eq!(layer.recovery_count(), 0, "el contador no se gasta");
        assert_eq!(layer.custodian_uses(), 0, "ni el cupo");
    }

    fn dominio() -> BaseElement {
        BaseElement::new(CUSTODIAN_DOMAIN)
    }

    fn compromiso(root_old: Digest, root_new: Digest, count_old: u64) -> Digest {
        let mut v: Vec<BaseElement> = root_old.to_vec();
        v.extend_from_slice(&root_new);
        v.push(BaseElement::new(count_old));
        v.push(BaseElement::new(count_old + 1));
        auth::commit_operation(auth::OP_RECOVERY, &v)
    }

    fn autorizar(
        key: BaseElement,
        path: &stark_experiment::circuit_threshold::CustodianPath,
        op: Digest,
    ) -> (winterfell::Proof, auth::NullifierThresholdPublicInputs) {
        let trace = auth::build_trace(dominio(), key, path, op);
        let prover = auth::NullifierThresholdProver::new(proof_options());
        let inputs = prover.get_pub_inputs(&trace);
        (prover.prove(trace).expect("autorizacion"), inputs)
    }

    /// Monta la capa con una cuenta y devuelve (capa, indice, id_nueva).
    fn capa() -> (SovereignLayer, AccountIndex, Digest) {
        let mut l = SovereignLayer::new(
            custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS);
        // Post-F3 la posición ya no es secuencial: se captura la REAL.
        let idx = l.open_account_checked(BaseElement::new(SK_ALICE)).expect("abrir");
        let nueva = stark_experiment::circuit_settlement::derive_public_id(
            BaseElement::new(0xBEEF_CAFE));
        (l, idx, nueva)
    }

    fn prueba_subida(layer: &SovereignLayer, idx: AccountIndex, nueva: Digest) -> winterfell::Proof {
        let rec = layer.records.get(&idx).expect("cuenta").clone();
        let path = layer.accounts.path_for(idx);
        let trace = climb::build_trace(
            rec.public_id, nueva, rec.balance, rec.balance, rec.nonce,
            rec.leaf_salt, &path,
            layer.recovery_count(), 1,
        );
        climb::RecoveryClimbProver::new(proof_options())
            .prove(trace)
            .expect("subida")
    }

    fn raiz_nueva(layer: &SovereignLayer, idx: AccountIndex, nueva: Digest) -> Digest {
        let rec = layer.records.get(&idx).expect("cuenta").clone();
        let mut t = layer.accounts.clone();
        t.set_leaf(idx, native_leaf_salted(nueva, BaseElement::new(rec.balance),
                                    rec.nonce + BaseElement::ONE,
                                    rec.leaf_salt));
        t.root()
    }

    /// Dos custodios distintos recuperan una cuenta sin entregar sus claves.
    #[test]
    fn a_delegated_recovery_applies() {
        let (mut layer, idx, nueva) = capa();
        let ck = custodian_keys();
        let (_, cp) = build_custodian_set(&ck);

        let op = compromiso(layer.accounts.root(), raiz_nueva(&layer, idx, nueva),
                            layer.recovery_count());
        let subida = prueba_subida(&layer, idx, nueva);
        let (pa, ia) = autorizar(ck[1], &cp[1], op);
        let (pb, ib) = autorizar(ck[3], &cp[3], op);

        layer.apply_recovery_delegated(subida, pa, ia, pb, ib, idx, nueva)
            .expect("la recuperacion legitima debe aplicarse");
        assert_eq!(layer.records.get(&idx).unwrap().public_id, nueva,
                   "la identidad debe haber cambiado");
    }

    #[test]
    fn the_same_custodian_twice_cannot_recover() {
        let (mut layer, idx, nueva) = capa();
        let ck = custodian_keys();
        let (_, cp) = build_custodian_set(&ck);
        let antes = layer.records.get(&idx).unwrap().public_id;

        let op = compromiso(layer.accounts.root(), raiz_nueva(&layer, idx, nueva),
                            layer.recovery_count());
        let subida = prueba_subida(&layer, idx, nueva);
        let (pa, ia) = autorizar(ck[2], &cp[2], op);
        let (pb, ib) = autorizar(ck[2], &cp[2], op);

        let r = layer.apply_recovery_delegated(subida, pa, ia, pb, ib, idx, nueva);
        assert!(matches!(r, Err(LayerError::NotTheIssuer)), "fue {r:?}");
        assert_eq!(layer.records.get(&idx).unwrap().public_id, antes, "no debe cambiar");
    }

    /// Una autorizacion para recuperar hacia una identidad no sirve para otra.
    #[test]
    fn an_authorization_for_another_identity_does_not_apply() {
        let (mut layer, idx, nueva) = capa();
        let ck = custodian_keys();
        let (_, cp) = build_custodian_set(&ck);
        let antes = layer.records.get(&idx).unwrap().public_id;
        let otra = stark_experiment::circuit_settlement::derive_public_id(
            BaseElement::new(0xDEAD_BEEF));

        let op = compromiso(layer.accounts.root(), raiz_nueva(&layer, idx, otra),
                            layer.recovery_count());
        let subida = prueba_subida(&layer, idx, nueva);
        let (pa, ia) = autorizar(ck[1], &cp[1], op);
        let (pb, ib) = autorizar(ck[3], &cp[3], op);

        let r = layer.apply_recovery_delegated(subida, pa, ia, pb, ib, idx, nueva);
        assert!(r.is_err(), "una autorizacion para otra identidad no vale, fue {r:?}");
        assert_eq!(layer.records.get(&idx).unwrap().public_id, antes);
    }

    /// La jerarquia: gobernanza no recupera cuentas.
    #[test]
    fn governance_keys_cannot_recover() {
        let (mut layer, idx, nueva) = capa();
        let gk = governance_keys();
        let (_, gp) = build_custodian_set(&gk);
        let antes = layer.records.get(&idx).unwrap().public_id;

        let op = compromiso(layer.accounts.root(), raiz_nueva(&layer, idx, nueva),
                            layer.recovery_count());
        let subida = prueba_subida(&layer, idx, nueva);
        let (pa, ia) = autorizar(gk[1], &gp[1], op);
        let (pb, ib) = autorizar(gk[3], &gp[3], op);

        let r = layer.apply_recovery_delegated(subida, pa, ia, pb, ib, idx, nueva);
        assert!(matches!(r, Err(LayerError::NotTheIssuer)), "fue {r:?}");
        assert_eq!(layer.records.get(&idx).unwrap().public_id, antes);
    }
}
