//! Transferencias con partida doble.
//!
//! La operación central: conserva el dinero, exige la clave de gasto del
//! emisor, demuestra que el nullifier no se había gastado, y encadena
//! las raíces para que una liquidación no pueda reenviarse.
use super::*;

impl SovereignLayer {
    // Transferencia
    // -----------------------------------------------------------------

    /// Genera la prueba de una transferencia. **No modifica el estado.**
    /// ⚠️ **Recibe la clave de gasto**, es decir, se la entrega a quien
    /// opera el nodo — que con ella puede vaciar la cuenta.
    ///
    /// Se conserva por comodidad y para los tests, pero **el camino
    /// correcto es `client::prove_transfer`**, donde la clave nunca sale
    /// de la máquina del titular.
    pub fn transfer(
        &self,
        sender_key: BaseElement,
        sender_index: AccountIndex,
        receiver_index: AccountIndex,
        amount: u64,
    ) -> Result<Settlement, LayerError> {
        let sender = self
            .records
            .get(&sender_index)
            .ok_or(LayerError::AccountNotFound(sender_index))?
            .clone();
        let receiver = self
            .records
            .get(&receiver_index)
            .ok_or(LayerError::AccountNotFound(receiver_index))?
            .clone();

        // Quien transfiere debe ser el titular.
        //
        // El circuito lo impone igualmente, pero en RELEASE Winterfell no
        // valida las restricciones al generar: se gastaría el cómputo de
        // una prueba que luego no verifica, y el error resultante sería
        // técnico en vez de decir lo que pasa. `burn` y `audit` ya hacían
        // esta comprobación; `transfer` no, y era la operación más usada.
        // Congelada: el circuito lo impone igualmente, pero fallar aqui
        // evita gastar ~600 ms en una prueba que no verificara.
        if self.is_frozen(sender_index) {
            return Err(LayerError::AccountFrozen(sender_index));
        }
        if derive_public_id(sender_key) != sender.public_id {
            return Err(LayerError::NotTheAccountHolder);
        }
        if amount > sender.balance {
            return Err(LayerError::InsufficientBalance {
                available: sender.balance,
                requested: amount,
            });
        }
        if amount > self.regulatory_limit {
            return Err(LayerError::OverRegulatoryLimit {
                limit: self.regulatory_limit,
                requested: amount,
            });
        }

        // El nullifier se deriva de la CLAVE: solo el titular puede
        // calcularlo.
        let nullifier = stark_experiment::circuit_settlement::native_nullifier(
            sender_key,
            sender.nonce,
        );
        let null_pos = nullifier_position(&nullifier);
        if self.nullifiers.is_occupied(null_pos) {
            return Err(LayerError::NullifierAlreadySpent);
        }

        let sender_path = self.accounts.path_for(sender_index);

        // Arbol INTERMEDIO: solo el emisor actualizado. El camino del
        // receptor sale de AQUI, no del arbol antiguo.
        let mut mid = self.accounts.clone();
        mid.set_leaf(
            sender_index,
            native_leaf(
                sender.public_id,
                BaseElement::new(sender.balance - amount),
                sender.nonce + BaseElement::ONE,
            ),
        );
        let receiver_path = mid.path_for(receiver_index);
        let null_path = self.nullifiers.path_for(null_pos);

        let trace = build_settlement_trace(
            &SenderWitness {
                spend_key: sender_key,
                balance: sender.balance,
                nonce: sender.nonce,
                path: sender_path,
            },
            &ReceiverWitness {
                public_id: receiver.public_id,
                balance: receiver.balance,
                nonce: receiver.nonce,
                path: receiver_path,
            },
            amount,
            amount,
            self.regulatory_limit,
            &null_path,
            &self.frozen.path_for(sender_index),
        );

        let prover = SettlementProver::new(self.options.clone());
        let public_inputs = prover.get_pub_inputs(&trace);
        let proof = prover
            .prove(trace)
            .map_err(|e| LayerError::ProofFailed(format!("{e:?}")))?;

        Ok(Settlement {
            proof: proof.to_bytes(),
            public_inputs,
        })
    }

    /// Verifica una liquidación y, si es válida y parte del estado
    /// actual, la aplica.
    pub fn apply(
        &mut self,
        settlement: &Settlement,
        sender_index: AccountIndex,
        receiver_index: AccountIndex,
        amount: u64,
    ) -> Result<(), LayerError> {
        let pi = &settlement.public_inputs;

        // El límite lo impone el SISTEMA, no quien transfiere.
        let declared_limit = pi.regulatory_limit.as_int();
        if declared_limit != self.regulatory_limit {
            return Err(LayerError::WrongRegulatoryLimit {
                expected: self.regulatory_limit,
                declared: declared_limit,
            });
        }

        // Encadenamiento: la operación debe partir del estado actual. Es
        // lo que impide reenviar una liquidación válida.
        if pi.root_old != self.accounts.root()
            || pi.nullifier_root_old != self.nullifiers.root()
        {
            return Err(LayerError::StaleState);
        }

        let proof = winterfell::Proof::from_bytes(&settlement.proof)
            .map_err(|e| LayerError::VerificationFailed(format!("prueba mal formada: {e:?}")))?;
        let min_opts = AcceptableOptions::OptionSet(vec![self.options.clone()]);
        verify::<SettlementAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof,
            pi.clone(),
            &min_opts,
        )
        .map_err(|e| LayerError::VerificationFailed(format!("{e:?}")))?;

        let sender = self
            .records
            .get(&sender_index)
            .ok_or(LayerError::AccountNotFound(sender_index))?
            .clone();
        let receiver = self
            .records
            .get(&receiver_index)
            .ok_or(LayerError::AccountNotFound(receiver_index))?
            .clone();

        let new_sender = AccountRecord {
            public_id: sender.public_id,
            balance: sender.balance - amount,
            nonce: sender.nonce + BaseElement::ONE,
        };
        let new_receiver = AccountRecord {
            public_id: receiver.public_id,
            balance: receiver.balance + amount,
            nonce: receiver.nonce,
        };

        self.accounts.set_leaf(
            sender_index,
            native_leaf(
                new_sender.public_id,
                BaseElement::new(new_sender.balance),
                new_sender.nonce,
            ),
        );
        self.accounts.set_leaf(
            receiver_index,
            native_leaf(
                new_receiver.public_id,
                BaseElement::new(new_receiver.balance),
                new_receiver.nonce,
            ),
        );
        self.records.insert(sender_index, new_sender);
        self.records.insert(receiver_index, new_receiver);

        // INSERTAR EL NULLIFIER. Sin esto el árbol nunca crece y la
        // no-pertenencia sería vacua en la práctica.
        let null_pos = nullifier_position(&pi.nullifier);
        self.nullifiers.set_leaf(null_pos, pi.nullifier);

        if self.accounts.root() != pi.root_new
            || self.nullifiers.root() != pi.nullifier_root_new
        {
            return Err(LayerError::StaleState);
        }

        // Las dos cuentas, el nullifier y los metadatos en UN SOLO lote
        // atomico. Antes eran cuatro llamadas con nueve escrituras: si el
        // proceso moria en medio, el ledger quedaba a medias.
        // Deja constancia en el registro ANTES de persistir: si el
        // proceso muere en medio, el lote atomico incluye o excluye
        // ambas cosas.
        self.log
            .append(OpKind::Transfer, pi.root_old, pi.root_new, &settlement.proof);
        self.commit(&[sender_index, receiver_index], Some((null_pos, pi.nullifier)))?;
        Ok(())
    }
}
