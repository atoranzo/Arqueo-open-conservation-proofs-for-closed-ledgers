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

    /// Genera la prueba de una recuperación. **No modifica el estado.**
    ///
    /// Exige dos custodios distintos del conjunto autorizado. El circuito
    /// impone que **el saldo no cambie**: una recuperación reasigna el
    /// control, no mueve dinero.
    ///
    /// ⚠️ **El circuito no verifica que el nuevo titular sea legítimo.**
    /// Eso lo comprueban los custodios fuera de línea; la criptografía no
    /// puede saber de quién es una cuenta.
    pub fn recover(
        &self,
        auth: &ThresholdAuth,
        account_index: AccountIndex,
        new_public_id: Digest,
    ) -> Result<RecoveryReceipt, LayerError> {
        if auth.index_a >= auth.index_b {
            return Err(LayerError::NotTheIssuer);
        }
        let account = self
            .records
            .get(&account_index)
            .ok_or(LayerError::AccountNotFound(account_index))?
            .clone();

        if new_public_id == account.public_id {
            return Err(LayerError::RecoveryToSameIdentity);
        }

        let path = self.accounts.path_for(account_index);
        let trace = build_recovery_trace(
            auth,
            account.public_id,
            new_public_id,
            account.balance,
            account.balance,
            account.nonce,
            &path,
            self.recovery_count,
            1,
        );

        let prover = RecoveryProver::new(self.options.clone());
        let public_inputs = prover.get_pub_inputs(&trace);
        let proof = prover
            .prove(trace)
            .map_err(|e| LayerError::ProofFailed(format!("{e:?}")))?;

        Ok(RecoveryReceipt {
            proof: proof.to_bytes(),
            public_inputs,
            new_public_id,
        })
    }

    /// Verifica una recuperación y, si es válida y parte del estado
    /// actual, la aplica.
    pub fn apply_recovery(
        &mut self,
        receipt: &RecoveryReceipt,
        account_index: AccountIndex,
    ) -> Result<(), LayerError> {
        let pi = &receipt.public_inputs;

        if pi.custodian_set_root != self.custodian_set_root {
            return Err(LayerError::NotTheIssuer);
        }
        if pi.root_old != self.accounts.root()
            || pi.recovery_count_old != BaseElement::new(self.recovery_count)
        {
            return Err(LayerError::StaleState);
        }

        let proof = winterfell::Proof::from_bytes(&receipt.proof)
            .map_err(|e| LayerError::VerificationFailed(format!("prueba mal formada: {e:?}")))?;
        let min_opts = AcceptableOptions::OptionSet(vec![self.options.clone()]);
        verify::<RecoveryAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof,
            pi.clone(),
            &min_opts,
        )
        .map_err(|e| LayerError::VerificationFailed(format!("{e:?}")))?;

        let account = self
            .records
            .get(&account_index)
            .ok_or(LayerError::AccountNotFound(account_index))?
            .clone();

        // El saldo NO cambia. La identidad y el nonce, sí.
        let updated = AccountRecord {
            public_id: receipt.new_public_id,
            balance: account.balance,
            nonce: account.nonce + BaseElement::ONE,
        };
        // ===== ROTACIÓN: consume una intervención del conjunto =====
        //
        // Se consume **al aplicar**, no al generar la prueba: una prueba
        // que nunca se aplica no debe gastar cupo.
        //
        // Y va después de verificar la autoridad: si fuera antes,
        // cualquiera podría agotar el cupo de los custodios sin serlo.
        self.consume_custodian_use()?;

        // Se comprueba sobre una copia: ver el comentario de `mint`.
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
        self.recovery_count = pi.recovery_count_new.as_int();

        // Deja constancia en el registro ANTES de persistir: si el
        // proceso muere en medio, el lote atomico incluye o excluye
        // ambas cosas.
        self.log
            .append(OpKind::Recovery, pi.root_old, pi.root_new, &receipt.proof);
        self.commit(&[account_index], None, None)?;
        Ok(())
    }
}
