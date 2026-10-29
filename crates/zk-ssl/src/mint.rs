//! Emisión de circulante.
//!
//! Requiere **DOS custodios distintos** y respeta un **tope inmutable** del
//! ledger. El suministro público sube exactamente en lo emitido, lo que
//! hace la conservación auditable globalmente.
use super::*;

impl SovereignLayer {
    // Emisión
    // -----------------------------------------------------------------

    // -----------------------------------------------------------------

    /// §716 (el reenvio de una emision delegada): el contador de emisiones, `n`. Cuenta
    /// las dos vias, `Mint` y `MintToPending`, y lo deriva el registro
    /// ([`crate::log::TransitionLog::emisiones_delegadas`]). La proxima emision se autoriza con
    /// `(n, n + 1)` en su compromiso.
    pub fn mint_count(&self) -> u64 {
        self.log.emisiones_delegadas()
    }

    /// Emite sin que las claves de custodio lleguen como dato al operador: la
    /// via de la entrada 32/33 (67). Hasta el §538 cada prueba publicaba la suya (§523).
    ///
    /// Es la mas critica de las cinco: `mint` **crea dinero**, asi que es
    /// donde el fallo de la entrada 32 mas pesa. Un operador con las claves
    /// podia emitir; hasta el §538, tras la primera emision delegada las tenia (§523).
    ///
    /// Tres pruebas: `climb_proof` de `circuit_mint_climb` -que el saldo y
    /// el suministro suben EXACTAMENTE en el importe y que no se pasa del
    /// tope- y dos de custodios distintos que autorizan ESTA emision.
    ///
    /// ⚠️ §716 (el reenvio de una emision delegada): el compromiso lleva el contador de
    /// emisiones, `(n, n + 1)`. Sin el, una autorizacion valia otra vez en cuanto la raiz de
    /// cuentas y el suministro volvian a los de antes de emitir, y una quema los devuelve: emitir,
    /// quemar y reenviar los mismos materiales volvia a emitir, en bucle, hasta el cupo de los
    /// custodios. El agregado cuadraba en cada paso; el fallo era de autorizacion, no de
    /// aritmetica.
    ///
    /// Limite declarado: una autorizacion vale para UN valor del contador. Si otra emision lo
    /// consume antes, por cualquiera de las dos vias, hay que firmar de nuevo; los materiales
    /// viejos caen con `StaleState`. Volver a autorizar la misma operacion despues de que el estado
    /// vuelva atras SI vale: el contador ya es otro, y con el, el compromiso y los nulificadores.
    pub fn apply_mint_delegated(
        &mut self,
        climb_proof: winterfell::Proof,
        proof_a: winterfell::Proof,
        inputs_a: stark_experiment::circuit_threshold_single_nullifier::NullifierThresholdPublicInputs,
        proof_b: winterfell::Proof,
        inputs_b: stark_experiment::circuit_threshold_single_nullifier::NullifierThresholdPublicInputs,
        account_index: AccountIndex,
        amount: u64,
    ) -> Result<(), LayerError> {
        use stark_experiment::circuit_mint_climb::{MintClimbAir, MintClimbPublicInputs};
        use stark_experiment::circuit_threshold::CUSTODIAN_DOMAIN;
        use stark_experiment::circuit_threshold_single_nullifier::{
            commit_operation, verify_threshold_pair, PairRejection, OP_MINT,
        };

        // §716 (el reenvio de una emision delegada): unos materiales ya aplicados traen en
        // sus entradas publicas el compromiso que se consumio. Se rechazan con su causa, antes de
        // verificar nada.
        for c in [&inputs_a.operation, &inputs_b.operation] {
            if self.log.autorizacion_consumida(c) {
                return Err(LayerError::AutorizacionConsumida { compromiso: *c });
            }
        }

        // El tope se comprueba AQUI y ademas en el circuito. Aqui porque la
        // capa conoce el suministro real; en el circuito porque un auditor
        // externo no puede recomputarlo.
        let would_be = self.total_supply.saturating_add(amount);
        if would_be > self.max_supply {
            return Err(LayerError::SupplyCapExceeded {
                cap: self.max_supply,
                would_be,
            });
        }

        let account = self
            .records
            .get(&account_index)
            .ok_or(LayerError::AccountNotFound(account_index))?
            .clone();

        let root_old = self.accounts.root();
        let supply_old = self.total_supply;
        // §668: comprobadas. Con el tope de la capa por debajo de 2^62 no fallan nunca; si un
        // estado roto las hiciera fallar, es un error y no una vuelta silenciosa.
        let desborde = || LayerError::SupplyCapExceeded { cap: self.max_supply, would_be: u64::MAX };
        let supply_new = supply_old.checked_add(amount).ok_or_else(desborde)?;
        let balance_new = account.balance.checked_add(amount).ok_or_else(desborde)?;

        let updated = AccountRecord {
            public_id: account.public_id,
            balance: balance_new,
            nonce: account.nonce,
            view_id: account.view_id,
            // opera sobre el record guardado -> preserva el salt.
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

        crate::sin_panico("subida de emision", || {
            // §913: la forma, la marca y la verificacion, por el juez unico
            juzgar::<MintClimbAir>(
                climb_proof,
                MintClimbPublicInputs {
                    root_old,
                    root_new,
                    amount: BaseElement::new(amount),
                    supply_old: BaseElement::new(supply_old),
                    supply_new: BaseElement::new(supply_new),
                    max_supply: BaseElement::new(self.max_supply),
                },
            )
            .map_err(|r| crate::rechazo("subida de emision: ", r))?;
            Ok(())
        })?;

        // El compromiso cubre TODO lo que la emision decide: de que raiz a
        // cual, cuanto, y contra que suministro y que tope.
        //
        // §716 (el reenvio de una emision delegada): y en que punto del contador de
        // emisiones, `(n, n + 1)`. Sin el, la raiz y el suministro de antes de emitir vuelven con
        // una quema, y la autorizacion vale otra vez. El formato viejo (doce elementos) y este
        // (catorce) no se confunden: `commit_operation` rellena con ceros, y `n + 1` nunca es cero.
        let count_old = self.mint_count();
        let mut params: Vec<BaseElement> = root_old.to_vec();
        params.extend_from_slice(&root_new);
        params.push(BaseElement::new(amount));
        params.push(BaseElement::new(supply_old));
        params.push(BaseElement::new(supply_new));
        params.push(BaseElement::new(self.max_supply));
        params.push(BaseElement::new(count_old));
        params.push(BaseElement::new(count_old + 1));
        let operation = commit_operation(OP_MINT, &params);

        crate::sin_panico("autorizacion", || {
            verify_threshold_pair(
                proof_a, inputs_a, proof_b, inputs_b,
                BaseElement::new(CUSTODIAN_DOMAIN),
                self.custodian_set_root,
                operation,
            )
            .map_err(|r| match r {
                PairRejection::WrongTraceWidth => LayerError::VerificationFailed(
                    "autorizacion: forma de traza equivocada".into(),
                ),
                PairRejection::SameCustodian
                | PairRejection::WrongCustodianSet
                | PairRejection::WrongIdentityDomain => LayerError::NotTheIssuer,
                PairRejection::WrongOperation => LayerError::StaleState,
                PairRejection::InvalidProof => {
                    LayerError::VerificationFailed("autorizacion invalida".into())
                }
            })?;
            Ok(())
        })?;

        // Despues de verificar la autoridad.
        self.consume_custodian_use()?;

        self.accounts = tentativo;
        self.records.insert(account_index, updated);
        self.total_supply = supply_new;

        // El registro sube el contador de emisiones y guarda el sello como consumido.
        self.log.append_con_compromiso(OpKind::Mint, root_old, root_new, &crate::log::sello_de_autorizacion(&operation), operation);
        self.commit(&[account_index], None)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests_delegada {
    use super::*;
    use crate::tests_support::*;
    use stark_experiment::circuit_mint_climb as climb;
    use stark_experiment::circuit_threshold::{build_custodian_set, CUSTODIAN_DOMAIN};
    use stark_experiment::circuit_threshold_single_nullifier as auth;

    const LIMIT: u64 = 1_000_000;
    const MAX_SUPPLY: u64 = 1_000_000_000;
    const MAX_ACCOUNTS: u64 = 1_000;
    const AMOUNT: u64 = 250_000;

    fn dominio() -> BaseElement {
        BaseElement::new(CUSTODIAN_DOMAIN)
    }

    fn compromiso(layer: &SovereignLayer, idx: AccountIndex, amount: u64) -> Digest {
        let rec = layer.records.get(&idx).expect("cuenta").clone();
        let mut t = layer.accounts.clone();
        t.set_leaf(idx, native_leaf_salted(rec.public_id,
                                    BaseElement::new(rec.balance + amount),
                                    rec.nonce, rec.leaf_salt));
        let mut v: Vec<BaseElement> = layer.accounts.root().to_vec();
        v.extend_from_slice(&t.root());
        v.push(BaseElement::new(amount));
        v.push(BaseElement::new(layer.total_supply()));
        v.push(BaseElement::new(layer.total_supply() + amount));
        v.push(BaseElement::new(MAX_SUPPLY));
        // §716 (el reenvio de una emision delegada): el contador de emisiones
        v.push(BaseElement::new(layer.mint_count()));
        v.push(BaseElement::new(layer.mint_count() + 1));
        auth::commit_operation(auth::OP_MINT, &v)
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

    fn prueba_subida(layer: &SovereignLayer, idx: AccountIndex, amount: u64) -> winterfell::Proof {
        let rec = layer.records.get(&idx).expect("cuenta").clone();
        let path = layer.accounts.path_for(idx);
        let trace = climb::build_trace(
            rec.public_id, rec.balance, rec.nonce, rec.leaf_salt, &path, amount,
            layer.total_supply(), amount, MAX_SUPPLY,
        );
        climb::MintClimbProver::new(proof_options()).prove(trace).expect("subida")
    }

    fn capa() -> (SovereignLayer, AccountIndex) {
        let mut l = SovereignLayer::new(
            custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS);
        // Post-F3 la posición ya no es secuencial: se captura la REAL.
        let idx = l.open_account_checked(BaseElement::new(SK_ALICE)).expect("abrir");
        (l, idx)
    }

    /// Dos custodios distintos emiten; sus claves viajaban en sus pruebas hasta el §538 (§523).
    #[test]
    fn a_delegated_mint_applies() {
        let (mut layer, idx) = capa();
        let ck = custodian_keys();
        let (_, cp) = build_custodian_set(&ck);
        let antes = layer.total_supply();

        let op = compromiso(&layer, idx, AMOUNT);
        let subida = prueba_subida(&layer, idx, AMOUNT);
        let (pa, ia) = autorizar(ck[1], &cp[1], op);
        let (pb, ib) = autorizar(ck[3], &cp[3], op);

        layer.apply_mint_delegated(subida, pa, ia, pb, ib, idx, AMOUNT)
            .expect("la emision legitima debe aplicarse");
        assert_eq!(layer.total_supply(), antes + AMOUNT, "el suministro debe subir");
        assert_eq!(layer.records.get(&idx).unwrap().balance, AMOUNT, "y el saldo");
    }

    /// **EL TOPE DELEGADO VIVE EN EL APPLY** — la generacion es del
    /// cliente y no conoce el suministro; la capa lo impone al aplicar
    /// (y el circuito lo re-declara para el auditor). Aqui la forma
    /// ACUMULADA, que subsume a la simple. Releva en B-3 la faceta
    /// apply de los cuatro tests gen-time de la via-recibo (§166).
    #[test]
    fn the_delegated_cap_is_enforced_at_apply() {
        let (mut layer, idx) = capa();
        let ck = custodian_keys();
        let (_, cp) = build_custodian_set(&ck);

        // Hasta casi el tope, por la delegada.
        let casi = MAX_SUPPLY - 1000;
        let op = compromiso(&layer, idx, casi);
        let subida = prueba_subida(&layer, idx, casi);
        let (pa, ia) = autorizar(ck[1], &cp[1], op);
        let (pb, ib) = autorizar(ck[3], &cp[3], op);
        layer
            .apply_mint_delegated(subida, pa, ia, pb, ib, idx, casi)
            .expect("hasta casi el tope");

        // Pequena por si sola; el acumulado la rechaza ANTES de verificar.
        //
        // QUIEN LA RECHAZA DEPENDE DEL MODO, Y LOS DOS ESTAN BIEN (§77.1).
        // El testigo afirma una subida por encima del tope: un enunciado
        // FALSO. En depuracion winterfell valida la traza al generar y se
        // niega; en release genera y rechaza la capa al aplicar. Lo que no
        // puede pasar en ningun modo es que se emita, y eso lo exige el
        // total de abajo. Sin gancho de panico: el §45 midio su carrera.
        let op = compromiso(&layer, idx, 2000);
        let subida = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            prueba_subida(&layer, idx, 2000)
        }));
        let (pa, ia) = autorizar(ck[1], &cp[1], op);
        let (pb, ib) = autorizar(ck[3], &cp[3], op);
        match subida {
            // El probador se nego: el enunciado es inprobable, que es la
            // propiedad. No hay nada que aplicar.
            Err(_) => {}
            Ok(subida) => {
                let r = layer.apply_mint_delegated(subida, pa, ia, pb, ib, idx, 2000);
                assert!(
                    matches!(r, Err(LayerError::SupplyCapExceeded { .. })),
                    "CRITICO: el tope debe aplicarse al acumulado tambien en la delegada: {r:?}"
                );
            }
        }
        assert_eq!(layer.total_supply(), casi, "y nada se emitio de mas");
    }

    /// **LOS MATERIALES ATAN LA CUENTA.** El compromiso sella el indice
    /// dentro del estado ANTES/DESPUES: aplicarlos a otra cuenta
    /// recompone otro compromiso y el par queda huerfano. Releva en B-3
    /// a `applying_a_receipt_to_the_wrong_account` (via-recibo, §166).
    #[test]
    fn materials_for_one_account_do_not_apply_to_another() {
        let (mut layer, alice) = capa();
        let bob = layer
            .open_account_checked(BaseElement::new(SK_BOB))
            .expect("abrir bob");
        let ck = custodian_keys();
        let (_, cp) = build_custodian_set(&ck);

        let op = compromiso(&layer, alice, AMOUNT);
        let subida = prueba_subida(&layer, alice, AMOUNT);
        let (pa, ia) = autorizar(ck[1], &cp[1], op);
        let (pb, ib) = autorizar(ck[3], &cp[3], op);

        let r = layer.apply_mint_delegated(subida, pa, ia, pb, ib, bob, AMOUNT);
        assert!(r.is_err(), "CRITICO: materiales de Alice sobre Bob: {r:?}");
        assert_eq!(layer.total_supply(), 0, "no se emitio nada");
        assert_eq!(layer.records.get(&bob).unwrap().balance, 0, "Bob intacto");
    }

    /// **REENVIAR una emision delegada NO emite dos veces.** El
    /// compromiso ata las raices del estado ANTES de aplicar: consumida
    /// la operacion, los mismos materiales quedan huerfanos del estado.
    /// Es el guardian delegado de la propiedad que
    /// `replaying_a_mint_is_rejected` guarda en la via-recibo (esa
    /// muere con la marca en B-3; esta la releva).
    ///
    /// ⚠️ §716 (el reenvio de una emision delegada): la segunda frase solo valia para el
    /// reenvio INMEDIATO. Una quema devuelve la raiz y el suministro a los de antes de emitir, y
    /// entonces los mismos materiales ya no eran huerfanos: volvian a emitir. Lo cierra el
    /// contador de emisiones del compromiso, y el reenvio cae con su causa,
    /// `AutorizacionConsumida`. Los reenvios tras la vuelta atras estan en `tests_reenvio`.
    #[test]
    fn replaying_a_delegated_mint_is_rejected() {
        let (mut layer, idx) = capa();
        let ck = custodian_keys();
        let (_, cp) = build_custodian_set(&ck);

        let op = compromiso(&layer, idx, AMOUNT);
        let subida = prueba_subida(&layer, idx, AMOUNT);
        let (pa, ia) = autorizar(ck[1], &cp[1], op);
        let (pb, ib) = autorizar(ck[3], &cp[3], op);
        layer
            .apply_mint_delegated(
                subida.clone(), pa.clone(), ia.clone(), pb.clone(), ib.clone(), idx, AMOUNT,
            )
            .expect("la primera aplicacion");
        let saldo = layer.records.get(&idx).unwrap().balance;
        let suministro = layer.total_supply();

        // Los MISMOS materiales, otra vez.
        let r = layer.apply_mint_delegated(subida, pa, ia, pb, ib, idx, AMOUNT);
        assert!(
            matches!(r, Err(LayerError::AutorizacionConsumida { compromiso }) if compromiso == op),
            "CRITICO: reenviar materiales crearia dinero: {r:?}"
        );
        assert_eq!(layer.records.get(&idx).unwrap().balance, saldo, "el saldo no cambia");
        assert_eq!(layer.total_supply(), suministro, "el suministro tampoco");
    }

    /// UNA SOLA CLAVE NO EMITE DINERO.
    #[test]
    fn the_same_custodian_twice_cannot_mint() {
        let (mut layer, idx) = capa();
        let ck = custodian_keys();
        let (_, cp) = build_custodian_set(&ck);
        let antes = layer.total_supply();

        let op = compromiso(&layer, idx, AMOUNT);
        let subida = prueba_subida(&layer, idx, AMOUNT);
        let (pa, ia) = autorizar(ck[2], &cp[2], op);
        let (pb, ib) = autorizar(ck[2], &cp[2], op);

        let r = layer.apply_mint_delegated(subida, pa, ia, pb, ib, idx, AMOUNT);
        assert!(matches!(r, Err(LayerError::NotTheIssuer)), "fue {r:?}");
        assert_eq!(layer.total_supply(), antes, "no debe emitirse nada");
    }

    /// UNA AUTORIZACION PARA EMITIR X NO SIRVE PARA EMITIR MAS.
    #[test]
    fn an_authorization_for_one_amount_does_not_mint_another() {
        let (mut layer, idx) = capa();
        let ck = custodian_keys();
        let (_, cp) = build_custodian_set(&ck);
        let antes = layer.total_supply();

        let op = compromiso(&layer, idx, AMOUNT);
        let subida = prueba_subida(&layer, idx, AMOUNT * 4);
        let (pa, ia) = autorizar(ck[1], &cp[1], op);
        let (pb, ib) = autorizar(ck[3], &cp[3], op);

        let r = layer.apply_mint_delegated(subida, pa, ia, pb, ib, idx, AMOUNT * 4);
        assert!(r.is_err(), "SOLIDEZ: autorizar 250k no autoriza emitir 1M, fue {r:?}");
        assert_eq!(layer.total_supply(), antes);
    }

    /// La jerarquia: gobernanza no emite.
    #[test]
    fn governance_keys_cannot_mint() {
        let (mut layer, idx) = capa();
        let gk = governance_keys();
        let (_, gp) = build_custodian_set(&gk);
        let antes = layer.total_supply();

        let op = compromiso(&layer, idx, AMOUNT);
        let subida = prueba_subida(&layer, idx, AMOUNT);
        let (pa, ia) = autorizar(gk[1], &gp[1], op);
        let (pb, ib) = autorizar(gk[3], &gp[3], op);

        let r = layer.apply_mint_delegated(subida, pa, ia, pb, ib, idx, AMOUNT);
        assert!(matches!(r, Err(LayerError::NotTheIssuer)), "fue {r:?}");
        assert_eq!(layer.total_supply(), antes);
    }
}

/// §716 (el reenvio de una emision delegada): los reenvios DESPUES de que el estado
/// vuelva atras. Cada test aserta su premisa -la raiz y el suministro que el compromiso viejo ata
/// son otra vez los de antes de emitir- y despues reenvia los mismos materiales. Medido por
/// mutacion: con la capa sin contador ni guarda, ninguno de estos tests pasa y cada reenvio da
/// `Ok(())`; con la guarda y sin contador, la reautorizacion legitima no vale; con el contador y
/// sin la guarda, los reenvios caen, pero con `StaleState` y no con su causa.
#[cfg(test)]
mod tests_reenvio {
    use super::*;
    use crate::tests_support::*;

    const X: u64 = 250_000;
    const SK_CAROL: u64 = 0xCA201;
    const SK_DAVE: u64 = 0xDA7E;

    type Par = stark_experiment::circuit_threshold_single_nullifier::NullifierThresholdPublicInputs;

    /// Lo que viaja de los custodios al operador: la subida y el par.
    struct Materiales {
        op: Digest,
        subida: winterfell::Proof,
        pa: winterfell::Proof,
        ia: Par,
        pb: winterfell::Proof,
        ib: Par,
    }

    fn clave(sk: u64) -> [BaseElement; 4] {
        [BaseElement::new(sk), BaseElement::ZERO, BaseElement::ZERO, BaseElement::ZERO]
    }

    /// Los custodios 1 y 3 autorizan emitir `x` en `idx` desde el estado de ahora.
    fn autorizar(l: &SovereignLayer, idx: AccountIndex, x: u64) -> Materiales {
        let op = mint_commitment(l, idx, x);
        let subida = mint_climb_proof(l, idx, x);
        let (pa, ia, pb, ib) = delegated_pair(op, 1, 3);
        Materiales { op, subida, pa, ia, pb, ib }
    }

    fn emitir(l: &mut SovereignLayer, m: &Materiales, idx: AccountIndex, x: u64) -> Result<(), LayerError> {
        l.apply_mint_delegated(
            m.subida.clone(), m.pa.clone(), m.ia.clone(), m.pb.clone(), m.ib.clone(), idx, x,
        )
    }

    /// Los custodios 1 y 3 autorizan emitir `x` a un pendiente de `receptor`.
    fn autorizar_a_pendiente(l: &SovereignLayer, receptor: Digest, sal: Digest, x: u64) -> Materiales {
        let op = mint_pending_commitment(l, receptor, sal, x);
        let subida = mint_pending_climb_proof(l, receptor, sal, x);
        let (pa, ia, pb, ib) = delegated_pair(op, 1, 3);
        Materiales { op, subida, pa, ia, pb, ib }
    }

    fn emitir_a_pendiente(
        l: &mut SovereignLayer,
        m: &Materiales,
        receptor: Digest,
        sal: Digest,
        x: u64,
    ) -> Result<crate::two_phase::PendingNotice, LayerError> {
        l.apply_mint_pending_delegated(
            m.subida.clone(), m.pa.clone(), m.ia.clone(), m.pb.clone(), m.ib.clone(), receptor, sal, x,
        )
    }

    /// El titular quema `x`: una prueba nueva, con bytes nuevos, que la guarda de las pruebas del
    /// titular no puede parar.
    fn quemar(l: &mut SovereignLayer, sk: u64, idx: AccountIndex, x: u64) {
        let st = state_of(l, idx);
        let q = l.burn(BaseElement::new(sk), idx, &st, x).expect("quema");
        l.apply_burn(&q, idx, &st).expect("aplicar la quema");
    }

    fn entradas(l: &SovereignLayer, kind: OpKind) -> usize {
        l.log.entries().iter().filter(|e| e.kind == kind).count()
    }

    fn consumida(r: &Result<(), LayerError>, op: Digest) -> bool {
        matches!(r, Err(LayerError::AutorizacionConsumida { compromiso }) if *compromiso == op)
    }

    /// Ataque 1: emitir, quemar y reenviar los mismos materiales.
    #[test]
    fn reenviar_una_emision_tras_quemarla_no_vuelve_a_emitir() {
        let mut l = new_layer_with_quota(1_000);
        let a = l.open_account_wide(clave(SK_ALICE));
        let (raiz0, s0) = (l.accounts.root(), l.total_supply());

        let m = autorizar(&l, a, X);
        emitir(&mut l, &m, a, X).expect("la emision autorizada");
        quemar(&mut l, SK_ALICE, a, X);
        assert_eq!(l.accounts.root(), raiz0, "premisa: la raiz de cuentas vuelve");
        assert_eq!(l.total_supply(), s0, "premisa: y el suministro");

        let r = emitir(&mut l, &m, a, X);
        assert!(consumida(&r, m.op), "SOLIDEZ: el reenvio volveria a emitir: {r:?}");
        assert_eq!(l.balance_of(a), Some(0), "el saldo no sube");
        assert_eq!(l.total_supply(), s0, "ni el suministro");
        assert_eq!(entradas(&l, OpKind::Mint), 1, "una autorizacion, una entrada");
        assert_eq!(l.custodian_uses(), 1, "el reenvio no consume cupo");
    }

    /// Ataque 2: el bucle. Un tercero quieto en el libro, y tres vueltas de quemar y reenviar.
    /// En cada vuelta los custodios autorizan DE NUEVO la misma operacion desde el mismo estado:
    /// eso es legitimo y tiene que valer, y con el contador da otro compromiso. Todos los
    /// materiales ya aplicados, los primeros y los de cada vuelta, caen.
    #[test]
    fn el_bucle_de_quema_y_reenvio_se_corta_y_la_reautorizacion_vale() {
        let mut l = new_layer_with_quota(1_000);
        let _carol = open_and_fund(&mut l, SK_CAROL, 777);
        let a = l.open_account_wide(clave(SK_ALICE));
        let (raiz0, s0) = (l.accounts.root(), l.total_supply());

        let mut aplicadas = vec![autorizar(&l, a, X)];
        emitir(&mut l, &aplicadas[0], a, X).expect("la emision autorizada");
        let mut redimido = 0u64;
        for vuelta in 0..3 {
            quemar(&mut l, SK_ALICE, a, X);
            redimido += X;
            assert_eq!(l.accounts.root(), raiz0, "premisa, vuelta {vuelta}: la raiz vuelve");
            assert_eq!(l.total_supply(), s0, "premisa, vuelta {vuelta}: y el suministro");

            for m in &aplicadas {
                let r = emitir(&mut l, m, a, X);
                assert!(consumida(&r, m.op), "SOLIDEZ, vuelta {vuelta}: reenvio aceptado: {r:?}");
            }

            // La reautorizacion legitima: la misma operacion, el mismo estado, otro contador.
            let nueva = autorizar(&l, a, X);
            assert!(
                aplicadas.iter().all(|m| m.op != nueva.op && m.ia.nullifier != nueva.ia.nullifier),
                "el contador da otro compromiso y otros nulificadores"
            );
            emitir(&mut l, &nueva, a, X).expect("una autorizacion nueva emite");
            aplicadas.push(nueva);
        }

        // Cuatro autorizaciones de X: tres redimidas y una en saldo. Ni una unidad sin firma.
        assert_eq!(l.balance_of(a), Some(X));
        assert_eq!(redimido + X, aplicadas.len() as u64 * X, "lo emitido es lo autorizado");
        assert_eq!(entradas(&l, OpKind::Mint), 1 + aplicadas.len(), "la de Carol y las cuatro");
        assert_eq!(entradas(&l, OpKind::Burn), 3);
        assert_eq!(l.mint_count(), 1 + aplicadas.len() as u64);
    }

    /// Ataque 3: emitir a pendiente, cobrar, trafico ajeno, quemar y reenviar. Sin desemision, y
    /// sin que la raiz de CUENTAS vuelva: esta via no la ata.
    #[test]
    fn reenviar_una_emision_a_pendiente_tras_cobrarla_y_quemarla_no_vuelve_a_emitir() {
        let o = proof_options();
        let mut l = new_layer_with_quota(1_000);
        let c = open_and_fund(&mut l, SK_CAROL, 50_000);
        let d = l.open_account_wide(clave(SK_DAVE));
        let b = l.open_account_wide(clave(SK_BOB));
        let id_b = l.public_id_of(b).expect("bob");
        let id_d = l.public_id_of(d).expect("dave");
        let (sal, x) = (salt_de(0x6400), 9_999u64);
        let (praiz0, raiz0, s0) = (l.pending.root(), l.accounts.root(), l.total_supply());

        let m = autorizar_a_pendiente(&l, id_b, sal, x);
        let aviso = emitir_a_pendiente(&mut l, &m, id_b, sal, x).expect("la emision a pendiente");
        let mc = l.claim_materials(b, &aviso).expect("materiales del cobro");
        let cobro = crate::client::prove_claim(&mc, clave(SK_BOB), o.clone()).expect("cobro");
        l.apply_claim(&cobro, b, &state_of(&l, b), &aviso).expect("Bob cobra");

        // Trafico ajeno: Carol paga 1.234 a Dave y Dave cobra.
        let pos = l.reserve_pending().expect("reserva");
        let me = l.send_materials_at(c, id_d, 1_234, salt_de(0x6410), pos).expect("materiales");
        let envio = crate::client::prove_send(&me, clave(SK_CAROL), o.clone()).expect("envio");
        l.apply_send(&envio, c, &state_of(&l, c), 1_234).expect("Carol envia");
        let mc = l.claim_materials(d, &envio.notice).expect("materiales del cobro de Dave");
        let cobro = crate::client::prove_claim(&mc, clave(SK_DAVE), o).expect("cobro de Dave");
        l.apply_claim(&cobro, d, &state_of(&l, d), &envio.notice).expect("Dave cobra");

        quemar(&mut l, SK_BOB, b, x);
        assert_eq!(l.pending.root(), praiz0, "premisa: la raiz de pendientes vuelve");
        assert_eq!(l.total_supply(), s0, "premisa: y el suministro");
        assert_ne!(l.accounts.root(), raiz0, "y la de cuentas no hace falta");

        let r = emitir_a_pendiente(&mut l, &m, id_b, sal, x).map(|_| ());
        assert!(consumida(&r, m.op), "SOLIDEZ: el reenvio volveria a emitir: {r:?}");
        assert_eq!(l.total_supply(), s0, "el suministro no sube");
        assert_eq!(l.total_pending(), 0, "y no nace ningun pendiente");
        assert_eq!(entradas(&l, OpKind::MintToPending), 1);
    }

    /// Y por la desemision: emitir a pendiente, desemitir y reenviar.
    #[test]
    fn reenviar_una_emision_a_pendiente_tras_desemitirla_no_vuelve_a_emitir() {
        let mut l = new_layer_with_quota(1_000);
        l.set_refund_ttl(0);
        let b = l.open_account_wide(clave(SK_BOB));
        let id_b = l.public_id_of(b).expect("bob");
        let (sal, x) = (salt_de(0x6300), 9_999u64);
        let (praiz0, s0) = (l.pending.root(), l.total_supply());

        let m = autorizar_a_pendiente(&l, id_b, sal, x);
        let aviso = emitir_a_pendiente(&mut l, &m, id_b, sal, x).expect("la emision a pendiente");
        let des = l.deissue(aviso.position, id_b, sal, x).expect("desemision");
        l.apply_deissue(&des).expect("aplicar la desemision");
        assert_eq!(l.pending.root(), praiz0, "premisa: la raiz de pendientes vuelve");
        assert_eq!(l.total_supply(), s0, "premisa: y el suministro");

        let r = emitir_a_pendiente(&mut l, &m, id_b, sal, x).map(|_| ());
        assert!(consumida(&r, m.op), "SOLIDEZ: la emision vencida reviviria: {r:?}");
        assert_eq!(l.total_supply(), s0);
        assert_eq!(entradas(&l, OpKind::MintToPending), 1);
    }

    /// El limite declarado: una autorizacion vale para UN valor del contador, y el contador es de
    /// las dos vias. Los custodios firman una emision a Alicia; antes de aplicarla, otra emision
    /// -a pendiente- se aplica y se desemite. La raiz de cuentas y el suministro son otra vez los
    /// que se firmaron, pero el contador ya no: la de Alicia hay que firmarla de nuevo.
    #[test]
    fn una_autorizacion_vale_para_un_solo_valor_del_contador() {
        let mut l = new_layer_with_quota(1_000);
        l.set_refund_ttl(0);
        let a = l.open_account_wide(clave(SK_ALICE));
        let b = l.open_account_wide(clave(SK_BOB));
        let id_b = l.public_id_of(b).expect("bob");
        let (sal, x) = (salt_de(0x6500), 1_000u64);
        let (raiz0, s0) = (l.accounts.root(), l.total_supply());

        let a_cuenta = autorizar(&l, a, X);
        let a_pendiente = autorizar_a_pendiente(&l, id_b, sal, x);
        let aviso = emitir_a_pendiente(&mut l, &a_pendiente, id_b, sal, x).expect("la primera que llega");
        let des = l.deissue(aviso.position, id_b, sal, x).expect("desemision");
        l.apply_deissue(&des).expect("aplicar la desemision");
        assert_eq!(l.accounts.root(), raiz0, "premisa: la raiz de cuentas es la firmada");
        assert_eq!(l.total_supply(), s0, "premisa: y el suministro");

        let r = emitir(&mut l, &a_cuenta, a, X);
        assert!(matches!(r, Err(LayerError::StaleState)), "firmada para un contador ya gastado: {r:?}");
        assert_eq!(l.total_supply(), s0);
        // Firmada de nuevo, vale.
        let otra = autorizar(&l, a, X);
        emitir(&mut l, &otra, a, X).expect("firmada otra vez, emite");
        assert_eq!(l.mint_count(), 2);
        assert_eq!(l.balance_of(a), Some(X));
    }

    /// Lo que rompe el arreglo: una autorizacion firmada con el compromiso de antes, sin contador,
    /// ya no emite. Y los dos formatos no se confunden ni con el contador a cero.
    #[test]
    fn una_autorizacion_sin_contador_no_emite() {
        use stark_experiment::circuit_threshold_single_nullifier as auth;
        let mut l = new_layer_with_quota(1_000);
        let a = l.open_account_wide(clave(SK_ALICE));
        assert_eq!(l.mint_count(), 0);

        // El compromiso de antes: doce elementos, sin `(n, n + 1)`.
        let rec = l.records.get(&a).expect("cuenta").clone();
        let mut t = l.accounts.clone();
        t.set_leaf(
            a,
            native_leaf_salted(rec.public_id, BaseElement::new(X), rec.nonce, rec.leaf_salt),
        );
        let mut v: Vec<BaseElement> = l.accounts.root().to_vec();
        v.extend_from_slice(&t.root());
        for e in [X, 0, X, l.max_supply()] {
            v.push(BaseElement::new(e));
        }
        let viejo = auth::commit_operation(auth::OP_MINT, &v);
        let nuevo = mint_commitment(&l, a, X);
        assert_ne!(viejo, nuevo, "el contador a cero tambien cambia el compromiso");

        let subida = mint_climb_proof(&l, a, X);
        let (pa, ia, pb, ib) = delegated_pair(viejo, 1, 3);
        let r = l.apply_mint_delegated(subida, pa, ia, pb, ib, a, X);
        assert!(matches!(r, Err(LayerError::StaleState)), "fue {r:?}");
        assert_eq!(l.total_supply(), 0);
    }

    /// El contador y los sellos consumidos los deriva el registro: sobreviven al reinicio. Tras
    /// abrir, la premisa se sostiene, el reenvio cae con su causa y una autorizacion nueva emite:
    /// el contador derivado al abrir es el mismo que el vivo.
    #[test]
    fn el_reenvio_tras_el_reinicio_tambien_cae() {
        let mut ruta = std::env::temp_dir();
        ruta.push(format!("zkssl_reenvio_emision_{}", std::process::id()));
        let ruta = ruta.to_str().expect("ruta").to_string();
        quitar_libro(&ruta);
        let abrir = || {
            open_retry(&ruta, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS)
                .expect("abrir")
        };

        let (a, m, raiz0, s0, vivo) = {
            let mut l = abrir();
            let a = l.open_account_wide(clave(SK_ALICE));
            let (raiz0, s0) = (l.accounts.root(), l.total_supply());
            let m = autorizar(&l, a, X);
            emitir(&mut l, &m, a, X).expect("la emision autorizada");
            quemar(&mut l, SK_ALICE, a, X);
            (a, m, raiz0, s0, l.mint_count())
        };

        let mut l = abrir();
        assert_eq!(l.mint_count(), vivo, "el contador derivado al abrir es el vivo");
        assert_eq!(l.mint_count(), 1, "el contador se deriva del registro");
        assert_eq!(l.accounts.root(), raiz0, "premisa tras abrir: la raiz de cuentas vuelve");
        assert_eq!(l.total_supply(), s0, "premisa tras abrir: y el suministro");
        let r = emitir(&mut l, &m, a, X);
        assert!(consumida(&r, m.op), "SOLIDEZ: tras el reinicio el reenvio volveria a emitir: {r:?}");
        assert_eq!(l.total_supply(), s0);

        let nueva = autorizar(&l, a, X);
        assert_ne!(nueva.op, m.op, "otro contador, otro compromiso");
        emitir(&mut l, &nueva, a, X).expect("una autorizacion nueva emite tras reabrir");
        assert_eq!(l.mint_count(), 2);
        assert_eq!(l.balance_of(a), Some(X));
        drop(l);
        quitar_libro(&ruta);
    }

    /// El otro lector del registro, la instantanea: emitir, quemar, exportar, importar y
    /// reenviar. El contador y los sellos viajan en el registro, sin campo propio.
    #[test]
    fn el_reenvio_tras_la_instantanea_tambien_cae() {
        let mut ruta = std::env::temp_dir();
        ruta.push(format!("zkssl_reenvio_emision_snap_{}.bin", std::process::id()));
        let ruta = ruta.to_str().expect("ruta").to_string();
        let _ = std::fs::remove_file(&ruta);

        let mut l = new_layer_with_quota(1_000);
        let a = l.open_account_wide(clave(SK_ALICE));
        let (raiz0, s0) = (l.accounts.root(), l.total_supply());
        let m = autorizar(&l, a, X);
        emitir(&mut l, &m, a, X).expect("la emision autorizada");
        quemar(&mut l, SK_ALICE, a, X);
        l.export_snapshot(&ruta).expect("exportar");

        let mut r = SovereignLayer::import_snapshot(&ruta).expect("importar");
        let _ = std::fs::remove_file(&ruta);
        assert_eq!(r.mint_count(), l.mint_count(), "el contador importado es el vivo");
        assert_eq!(r.accounts.root(), raiz0, "premisa tras importar: la raiz de cuentas vuelve");
        assert_eq!(r.total_supply(), s0, "premisa tras importar: y el suministro");
        let res = emitir(&mut r, &m, a, X);
        assert!(consumida(&res, m.op), "SOLIDEZ: tras importar el reenvio volveria a emitir: {res:?}");
        assert_eq!(r.total_supply(), s0);
        assert_eq!(entradas(&r, OpKind::Mint), 1);

        let nueva = autorizar(&r, a, X);
        emitir(&mut r, &nueva, a, X).expect("una autorizacion nueva emite tras importar");
        assert_eq!(r.mint_count(), 2);
    }
}
