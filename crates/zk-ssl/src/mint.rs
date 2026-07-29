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
}
