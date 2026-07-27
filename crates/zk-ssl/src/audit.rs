//! Revelación selectiva para auditoría regulatoria.
//!
//! Demuestra `inferior <= saldo <= superior`, lo que cubre revelación
//! exacta, mínimo de reservas y **banda** con un solo circuito.
//!
//! Es revelación **voluntaria**, no custodia de claves: no hay ninguna
//! clave maestra que robar. La contrapartida es que el supervisor
//! depende de la cooperación del titular.
use super::*;

impl SovereignLayer {
    // Auditoría: revelación selectiva
    // -----------------------------------------------------------------

    /// Produce una revelación dirigida a un supervisor.
    ///
    /// Demuestra `inferior <= saldo <= superior` sobre el estado actual,
    /// **sin revelar la clave ni ningún otro dato del árbol**. Los tres
    /// usos:
    ///
    /// - `inferior = superior = saldo` → revelación exacta.
    /// - `inferior = X`, `superior = MAX_VALUE` → "supero X".
    /// - `inferior = X`, `superior = Y` → **"estoy entre X e Y"**, que es
    ///   lo que suele bastar a un supervisor y expone menos.
    ///
    /// Requiere la clave de gasto: **solo el titular puede revelar**. Es
    /// revelación voluntaria, no custodia de claves — no hay ninguna
    /// clave maestra que robar.
    pub fn audit(
        &self,
        spend_key: BaseElement,
        account_index: AccountIndex,
        lower: u64,
        upper: u64,
    ) -> Result<AuditDisclosure, LayerError> {
        let account = self
            .records
            .get(&account_index)
            .ok_or(LayerError::AccountNotFound(account_index))?
            .clone();

        // Comprobaciones tempranas.
        //
        // El circuito las volvería a imponer, pero en RELEASE Winterfell
        // no verifica las restricciones al generar: produciría una prueba
        // que luego no verifica, tras gastar el tiempo de generarla.
        // Fallar aquí da un error legible y no desperdicia el cómputo.
        if derive_public_id(spend_key) != account.public_id {
            return Err(LayerError::NotTheAccountHolder);
        }
        if account.balance < lower || account.balance > upper {
            return Err(LayerError::BalanceOutsideBand { lower, upper });
        }

        let witness = AuditWitness {
            spend_key,
            balance: account.balance,
            nonce: account.nonce,
            path: self.accounts.path_for(account_index),
        };
        let trace = build_audit_trace(&witness, lower, upper);

        let prover = AuditProver::new(self.options.clone());
        let public_inputs = prover.get_pub_inputs(&trace);
        let proof = prover
            .prove(trace)
            .map_err(|e| LayerError::ProofFailed(format!("{e:?}")))?;

        Ok(AuditDisclosure {
            proof: proof.to_bytes(),
            public_inputs,
        })
    }

    /// Atajo: revelar el saldo exacto.
    pub fn disclose_exact(
        &self,
        spend_key: BaseElement,
        account_index: AccountIndex,
    ) -> Result<AuditDisclosure, LayerError> {
        let balance = self
            .balance_of(account_index)
            .ok_or(LayerError::AccountNotFound(account_index))?;
        self.audit(spend_key, account_index, balance, balance)
    }

    /// Atajo: demostrar que se supera un mínimo sin revelar cuánto.
    pub fn prove_minimum(
        &self,
        spend_key: BaseElement,
        account_index: AccountIndex,
        threshold: u64,
    ) -> Result<AuditDisclosure, LayerError> {
        self.audit(spend_key, account_index, threshold, MAX_VALUE)
    }

    // -----------------------------------------------------------------
}
