//! Emisión de circulante.
//!
//! Requiere la clave del **emisor** y respeta un **tope inmutable** del
//! ledger. El suministro público sube exactamente en lo emitido, lo que
//! hace la conservación auditable globalmente.
use super::*;

impl SovereignLayer {
    // Emisión
    // -----------------------------------------------------------------

    /// Genera la prueba de una emisión. **No modifica el estado.**
    ///
    /// Exige **dos custodios distintos** del conjunto autorizado, con
    /// índices en orden estricto. Un 2-de-N en el que un custodio pudiera
    /// contar dos veces sería un 1-de-N disfrazado.
    pub fn mint(
        &self,
        auth: &ThresholdAuth,
        account_index: AccountIndex,
        amount: u64,
    ) -> Result<MintReceipt, LayerError> {
        // Comprobación temprana de distinción: en release Winterfell no
        // valida las restricciones al generar, así que sin esto se
        // gastaría el cómputo de una prueba que luego no verifica.
        if auth.index_a >= auth.index_b {
            return Err(LayerError::NotTheIssuer);
        }

        let account = self
            .records
            .get(&account_index)
            .ok_or(LayerError::AccountNotFound(account_index))?
            .clone();

        // El tope lo impone el SISTEMA. Sin esta comprobacion, la
        // autoridad emisora podria inflar sin limite: tendria la clave, y
        // nada mas se lo impediria.
        let would_be = self.total_supply.saturating_add(amount);
        if would_be > self.max_supply {
            return Err(LayerError::SupplyCapExceeded {
                cap: self.max_supply,
                would_be,
            });
        }

        let path = self.accounts.path_for(account_index);
        let trace = build_mint_trace(
            auth,
            account.public_id,
            account.balance,
            account.nonce,
            &path,
            amount,
            self.total_supply,
            amount,
            self.max_supply,
        );

        let prover = MintProver::new(self.options.clone());
        let public_inputs = prover.get_pub_inputs(&trace);
        let proof = prover
            .prove(trace)
            .map_err(|e| LayerError::ProofFailed(format!("{e:?}")))?;

        Ok(MintReceipt {
            proof: proof.to_bytes(),
            public_inputs,
        })
    }

    /// Verifica una emisión y, si es válida y parte del estado actual, la
    /// aplica.
    pub fn apply_mint(
        &mut self,
        receipt: &MintReceipt,
        account_index: AccountIndex,
    ) -> Result<(), LayerError> {
        let pi = &receipt.public_inputs;

        if pi.custodian_set_root != self.custodian_set_root {
            return Err(LayerError::NotTheIssuer);
        }
        // El tope declarado debe ser el del sistema.
        let declared_cap = pi.max_supply.as_int();
        if declared_cap != self.max_supply {
            return Err(LayerError::SupplyCapExceeded {
                cap: self.max_supply,
                would_be: declared_cap,
            });
        }
        if pi.root_old != self.accounts.root()
            || pi.supply_old != BaseElement::new(self.total_supply)
        {
            return Err(LayerError::StaleState);
        }

        let proof = winterfell::Proof::from_bytes(&receipt.proof)
            .map_err(|e| LayerError::VerificationFailed(format!("prueba mal formada: {e:?}")))?;
        let min_opts = AcceptableOptions::OptionSet(vec![self.options.clone()]);
        verify::<MintAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof,
            pi.clone(),
            &min_opts,
        )
        .map_err(|e| LayerError::VerificationFailed(format!("{e:?}")))?;

        // ===== ROTACIÓN: consume una intervención del conjunto =====
        //
        // Se consume **al aplicar**, no al generar la prueba: una prueba
        // que nunca se aplica no debe gastar cupo.
        //
        // Y va después de verificar la autoridad: si fuera antes,
        // cualquiera podría agotar el cupo de los custodios sin serlo.
        self.consume_custodian_use()?;

        let amount = pi.amount.as_int();
        let account = self
            .records
            .get(&account_index)
            .ok_or(LayerError::AccountNotFound(account_index))?
            .clone();
        let updated = AccountRecord {
            public_id: account.public_id,
            balance: account.balance + amount,
            nonce: account.nonce,
        };
        // ===== SE COMPRUEBA SOBRE UNA COPIA, NO SOBRE EL ESTADO =====
        //
        // Una versión anterior mutaba y comprobaba después: el error se
        // devolvía, pero **el estado ya había cambiado en memoria**.
        //
        // Un recibo de emisión para una cuenta, aplicado sobre otra,
        // dejaba a esa otra con el importe sumado. No se persistía —
        // `commit` no llegaba— pero el nodo quedaba con un estado que no
        // correspondía a su disco hasta reiniciar.
        let mut tentativo = self.accounts.clone();
        tentativo.set_leaf(
            account_index,
            native_leaf(
                updated.public_id,
                BaseElement::new(updated.balance),
                updated.nonce,
            ),
        );
        if tentativo.root() != pi.root_new {
            return Err(LayerError::StaleState);
        }

        self.accounts = tentativo;
        self.records.insert(account_index, updated);
        self.total_supply = pi.supply_new.as_int();

        // Deja constancia en el registro ANTES de persistir: si el
        // proceso muere en medio, el lote atomico incluye o excluye
        // ambas cosas.
        self.log
            .append(OpKind::Mint, pi.root_old, pi.root_new, &receipt.proof);
        self.commit(&[account_index], None)?;
        Ok(())
    }

    // -----------------------------------------------------------------

    /// Emite **sin que las claves de custodio lleguen al operador**: la via
    /// de la entrada 32/33 (67).
    ///
    /// Es la mas critica de las cinco: `mint` **crea dinero**, asi que es
    /// donde el fallo de la entrada 32 mas pesa. Un operador comprometido
    /// con las claves podia emitir; con esta via no.
    ///
    /// Tres pruebas: `climb_proof` de `circuit_mint_climb` -que el saldo y
    /// el suministro suben EXACTAMENTE en el importe y que no se pasa del
    /// tope- y dos de custodios distintos que autorizan ESTA emision.
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
        let supply_new = supply_old + amount;

        let updated = AccountRecord {
            public_id: account.public_id,
            balance: account.balance + amount,
            nonce: account.nonce,
        };
        let mut tentativo = self.accounts.clone();
        tentativo.set_leaf(
            account_index,
            native_leaf(
                updated.public_id,
                BaseElement::new(updated.balance),
                updated.nonce,
            ),
        );
        let root_new = tentativo.root();

        let accepted = AcceptableOptions::OptionSet(vec![self.options.clone()]);

        verify::<MintClimbAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            climb_proof,
            MintClimbPublicInputs {
                root_old,
                root_new,
                amount: BaseElement::new(amount),
                supply_old: BaseElement::new(supply_old),
                supply_new: BaseElement::new(supply_new),
                max_supply: BaseElement::new(self.max_supply),
            },
            &accepted,
        )
        .map_err(|e| LayerError::VerificationFailed(format!("subida de emision: {e:?}")))?;

        // El compromiso cubre TODO lo que la emision decide: de que raiz a
        // cual, cuanto, y contra que suministro y que tope.
        let mut params: Vec<BaseElement> = root_old.to_vec();
        params.extend_from_slice(&root_new);
        params.push(BaseElement::new(amount));
        params.push(BaseElement::new(supply_old));
        params.push(BaseElement::new(supply_new));
        params.push(BaseElement::new(self.max_supply));
        let operation = commit_operation(OP_MINT, &params);

        verify_threshold_pair(
            proof_a, inputs_a, proof_b, inputs_b,
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

        // Despues de verificar la autoridad.
        self.consume_custodian_use()?;

        self.accounts = tentativo;
        self.records.insert(account_index, updated);
        self.total_supply = supply_new;

        self.log.append(OpKind::Mint, root_old, root_new, &[]);
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
        t.set_leaf(idx, native_leaf(rec.public_id,
                                    BaseElement::new(rec.balance + amount), rec.nonce));
        let mut v: Vec<BaseElement> = layer.accounts.root().to_vec();
        v.extend_from_slice(&t.root());
        v.push(BaseElement::new(amount));
        v.push(BaseElement::new(layer.total_supply()));
        v.push(BaseElement::new(layer.total_supply() + amount));
        v.push(BaseElement::new(MAX_SUPPLY));
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
            rec.public_id, rec.balance, rec.nonce, &path, amount,
            layer.total_supply(), amount, MAX_SUPPLY,
        );
        climb::MintClimbProver::new(proof_options()).prove(trace).expect("subida")
    }

    fn capa() -> (SovereignLayer, AccountIndex) {
        let mut l = SovereignLayer::new(
            custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS);
        l.open_account_checked(BaseElement::new(SK_ALICE)).expect("abrir");
        (l, 0)
    }

    /// Dos custodios distintos emiten sin entregar sus claves.
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
