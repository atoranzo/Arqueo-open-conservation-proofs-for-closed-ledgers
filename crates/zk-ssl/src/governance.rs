//! Gobernanza: actualización del conjunto de custodios.
//!
//! ## El cambio de fondo
//!
//! Hasta ahora `custodian_set_root` era **inmutable**: si un custodio se
//! comprometía, conservaba para siempre el poder de emitir y —desde la
//! recuperación— de reasignar cualquier cuenta. La única salida era crear
//! un ledger nuevo.
//!
//! Ahora es **mutable por gobernanza**, y el parámetro inmutable pasa a
//! ser el conjunto de gobernanza.
//!
//! | Conjunto | Puede | Mutabilidad |
//! |---|---|---|
//! | Custodios | Emitir, recuperar cuentas | **Cambiable** |
//! | Gobernanza | Cambiar los custodios | **Inmutable** |
//!
//! ## Dónde para la cadena, dicho sin adornos
//!
//! **La circularidad no desaparece: se traslada.** Si el conjunto de
//! gobernanza se compromete, no hay salida salvo crear un ledger nuevo.
//!
//! Pero se traslada a claves que se usan casi nunca y pueden guardarse
//! sin conexión, frente a claves operativas expuestas a diario. Eso es
//! una mejora real, y no es lo mismo que resolver el problema.

use super::*;

impl SovereignLayer {
    /// Raíz del conjunto de gobernanza. **Inmutable**: cambiarla exige
    /// crear un ledger nuevo.
    pub fn governance_set_root(&self) -> Digest {
        self.governance_set_root
    }

    /// Contador público de cambios de gobernanza.
    pub fn governance_change_count(&self) -> u64 {
        self.governance_change_count
    }

    /// Genera la prueba de un cambio del conjunto de custodios.
    /// **No modifica el estado.**
    ///
    /// Exige **dos miembros distintos del conjunto de gobernanza**, no de
    /// custodios: quien puede emitir y recuperar cuentas no puede cambiar
    /// quién tiene ese poder.
    pub fn update_custodians(
        &self,
        auth: &GovernanceAuth,
        new_custodian_root: Digest,
    ) -> Result<GovernanceReceipt, LayerError> {
        if auth.index_a >= auth.index_b {
            return Err(LayerError::NotTheIssuer);
        }
        if new_custodian_root == self.custodian_set_root {
            return Err(LayerError::RecoveryToSameIdentity);
        }

        let trace = build_governance_trace(auth, self.governance_change_count, 1);
        let prover = GovernanceProver::new(
            self.options.clone(),
            self.custodian_set_root,
            new_custodian_root,
        );
        let public_inputs = prover.get_pub_inputs(&trace);
        let proof = prover
            .prove(trace)
            .map_err(|e| LayerError::ProofFailed(format!("{e:?}")))?;

        Ok(GovernanceReceipt {
            proof: proof.to_bytes(),
            public_inputs,
        })
    }

    /// Verifica un cambio de gobernanza y, si es válido y parte del
    /// estado actual, lo aplica.
    pub fn apply_governance(&mut self, receipt: &GovernanceReceipt) -> Result<(), LayerError> {
        let pi = &receipt.public_inputs;

        // La autoridad de gobernanza es la del sistema, y es inmutable.
        if pi.governance_set_root != self.governance_set_root {
            return Err(LayerError::NotTheIssuer);
        }
        if pi.custodian_root_old != self.custodian_set_root
            || pi.change_count_old != BaseElement::new(self.governance_change_count)
        {
            return Err(LayerError::StaleState);
        }

        let proof = winterfell::Proof::from_bytes(&receipt.proof)
            .map_err(|e| LayerError::VerificationFailed(format!("prueba mal formada: {e:?}")))?;
        let min_opts = AcceptableOptions::OptionSet(vec![self.options.clone()]);
        verify::<GovernanceAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof,
            pi.clone(),
            &min_opts,
        )
        .map_err(|e| LayerError::VerificationFailed(format!("{e:?}")))?;

        self.custodian_set_root = pi.custodian_root_new;
        self.governance_change_count = pi.change_count_new.as_int();

        // El cambio de custodios no toca cuentas ni nullifiers, pero sí
        // los metadatos: se persiste en el mismo lote atómico.
        // La cadena va SIEMPRE sobre la raiz de CUENTAS, no sobre la del
        // arbol que esta operacion modifica.
        //
        // Encadenar raices de arboles distintos no funciona: la raiz de
        // custodios de una entrada no tiene por que ser la de cuentas de
        // la siguiente. Esta operacion no toca el arbol de cuentas, asi
        // que su raiz no cambia — y el detalle de lo que SI cambio queda
        // atado por el resumen de la prueba.
        let raiz = self.accounts.root();
        // Deja constancia en el registro ANTES de persistir: si el
        // proceso muere en medio, el lote atomico incluye o excluye
        // ambas cosas.
        self.log
            .append(OpKind::Governance, raiz, raiz, &receipt.proof);
        self.commit(&[], None)?;
        Ok(())
    }
}
