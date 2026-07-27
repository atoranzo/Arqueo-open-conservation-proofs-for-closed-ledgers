//! Destrucción de circulante.
//!
//! Requiere la clave del **titular**, no la del emisor: destruir no
//! puede crear dinero, así que no necesita autorización del emisor para
//! ser seguro. Exigirla sería política monetaria, e impediría que el
//! titular se deshiciera de su propio saldo.
use super::*;

impl SovereignLayer {
    // Destrucción de circulante
    // -----------------------------------------------------------------

    /// Genera la prueba de una destrucción. **No modifica el estado.**
    ///
    /// Requiere la clave del **titular**, no la del emisor: destruir no
    /// puede crear dinero, así que no necesita autorización del emisor
    /// para ser seguro. Exigirla sería política monetaria, y además
    /// impediría que el titular se deshiciera de su propio saldo.
    pub fn burn(
        &self,
        spend_key: BaseElement,
        account_index: AccountIndex,
        amount: u64,
    ) -> Result<BurnReceipt, LayerError> {
        let account = self
            .records
            .get(&account_index)
            .ok_or(LayerError::AccountNotFound(account_index))?
            .clone();

        if derive_public_id(spend_key) != account.public_id {
            return Err(LayerError::NotTheAccountHolder);
        }
        if amount > account.balance {
            return Err(LayerError::InsufficientBalance {
                available: account.balance,
                requested: amount,
            });
        }
        if amount > self.total_supply {
            // No deberia ocurrir si la invariante se mantiene, pero si
            // ocurriera indicaria corrupcion del estado.
            return Err(LayerError::StaleState);
        }

        let path = self.accounts.path_for(account_index);
        let trace = build_burn_trace(
            spend_key,
            account.public_id,
            account.balance,
            account.nonce,
            &path,
            amount,
            self.total_supply,
            amount,
        );

        let prover = BurnProver::new(self.options.clone());
        let public_inputs = prover.get_pub_inputs(&trace);
        let proof = prover
            .prove(trace)
            .map_err(|e| LayerError::ProofFailed(format!("{e:?}")))?;

        Ok(BurnReceipt {
            proof: proof.to_bytes(),
            public_inputs,
        })
    }

    /// Verifica una destrucción y, si es válida y parte del estado
    /// actual, la aplica.
    pub fn apply_burn(
        &mut self,
        receipt: &BurnReceipt,
        account_index: AccountIndex,
    ) -> Result<(), LayerError> {
        let pi = &receipt.public_inputs;

        if pi.root_old != self.accounts.root()
            || pi.supply_old != BaseElement::new(self.total_supply)
        {
            return Err(LayerError::StaleState);
        }

        let proof = winterfell::Proof::from_bytes(&receipt.proof)
            .map_err(|e| LayerError::VerificationFailed(format!("prueba mal formada: {e:?}")))?;
        let min_opts = AcceptableOptions::OptionSet(vec![self.options.clone()]);
        verify::<BurnAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof,
            pi.clone(),
            &min_opts,
        )
        .map_err(|e| LayerError::VerificationFailed(format!("{e:?}")))?;

        let amount = pi.amount.as_int();
        let account = self
            .records
            .get(&account_index)
            .ok_or(LayerError::AccountNotFound(account_index))?
            .clone();
        let updated = AccountRecord {
            public_id: account.public_id,
            balance: account.balance - amount,
            nonce: account.nonce,
        };
        self.accounts.set_leaf(
            account_index,
            native_leaf(
                updated.public_id,
                BaseElement::new(updated.balance),
                updated.nonce,
            ),
        );
        self.records.insert(account_index, updated);
        self.total_supply = pi.supply_new.as_int();

        if self.accounts.root() != pi.root_new {
            return Err(LayerError::StaleState);
        }

        // Deja constancia en el registro ANTES de persistir: si el
        // proceso muere en medio, el lote atomico incluye o excluye
        // ambas cosas.
        self.log
            .append(OpKind::Burn, pi.root_old, pi.root_new, &receipt.proof);
        self.commit(&[account_index], None)?;
        Ok(())
    }

    // -----------------------------------------------------------------
}
