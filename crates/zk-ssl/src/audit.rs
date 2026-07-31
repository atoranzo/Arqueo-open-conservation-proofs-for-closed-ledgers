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
        // El estado lo aporta el titular: ver `burn.rs`.
        account_state: &crate::commitment::ClientState,
        lower: u64,
        upper: u64,
    ) -> Result<AuditDisclosure, LayerError> {
        let hoja = native_leaf(
            account_state.public_id,
            BaseElement::new(account_state.balance),
            account_state.nonce,
        );
        if hoja != self.accounts.leaf(account_index) {
            return Err(LayerError::StaleState);
        }
        let account = account_state.clone();

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
            // ⚠️ Clave RELLENADA a cuatro elementos (§92.9): la capa la
            // maneja estrecha y §90 garantiza la misma identidad.
            spend_key: [
                spend_key,
                BaseElement::ZERO,
                BaseElement::ZERO,
                BaseElement::ZERO,
            ],
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
        // El estado lo aporta el titular: ver `burn.rs`.
        account_state: &crate::commitment::ClientState,
    ) -> Result<AuditDisclosure, LayerError> {
        // **El saldo lo aporta el titular**, no la capa.
        //
        // Antes se leia con `balance_of`, que es justo lo que el modelo por
        // compromisos elimina: la capa no debe poder responder cuanto
        // tiene alguien.
        let balance = account_state.balance;
        self.audit(spend_key, account_index, account_state, balance, balance)
    }

    /// Atajo: demostrar que se supera un mínimo sin revelar cuánto.
    pub fn prove_minimum(
        &self,
        spend_key: BaseElement,
        account_index: AccountIndex,
        // El estado lo aporta el titular: ver `burn.rs`.
        account_state: &crate::commitment::ClientState,
        threshold: u64,
    ) -> Result<AuditDisclosure, LayerError> {
        self.audit(spend_key, account_index, account_state, threshold, MAX_VALUE)
    }

    // -----------------------------------------------------------------
}
