//! Congelación de cuentas.
//!
//! ## Lo que la hace real
//!
//! La congelación **no la impone esta capa**: la impone el circuito de
//! liquidación, que acredita que el emisor no está en el árbol de
//! congelados.
//!
//! Si solo la impusiera la capa, sería el operador negándose a procesar —
//! y **el operador ya puede censurar cualquier operación**. No añadiría
//! ninguna garantía verificable.
//!
//! Aquí la comprobación previa existe solo para fallar pronto con un
//! error claro, en vez de gastar ~600 ms generando una prueba que no
//! verificará.
//!
//! ## ⚠️ Lo que NO impide
//!
//! - **Recibir.** Una cuenta congelada no puede gastar, pero sí seguir
//!   recibiendo. Impedirlo exigiría comprobar también al receptor y
//!   dejaría fondos en el limbo.
//!
//!   ⚠️ **Esto ya NO es cierto por la vía en dos fases.** Cobrar un pendiente
//!   es una acción del receptor, y `circuit_claim` lleva `frozen_root`: una
//!   cuenta congelada **recibe hacia un pendiente que no puede cobrar**. El
//!   dinero queda en el limbo que esta nota decía evitar. Ver
//!   `AUDITORIA.md` §29.
//! - **Nada justifica la congelación en el circuito.** Demuestra que dos
//!   custodios la autorizaron, no que tuvieran razón.
//! - **No hay caducidad.** Dura hasta que alguien la levante.

use super::*;

impl SovereignLayer {
    /// Raíz del árbol de congelados. Pública: entra en cada prueba de
    /// liquidación.
    pub fn frozen_root(&self) -> Digest {
        self.frozen.root()
    }

    /// Contador público de congelaciones y descongelaciones.
    pub fn freeze_count(&self) -> u64 {
        self.freeze_count
    }

    /// Si una cuenta está congelada.
    pub fn is_frozen(&self, account_index: AccountIndex) -> bool {
        self.frozen.is_occupied(account_index)
    }

    /// Genera la prueba de una congelación o descongelación.
    /// **No modifica el estado.**
    ///
    /// Exige **dos custodios distintos**. Descongelar cuesta lo mismo que
    /// congelar: si levantar una congelación fuese más fácil que
    /// imponerla, no valdría de nada.
    pub fn set_frozen(
        &self,
        auth: &ThresholdAuth,
        account_index: AccountIndex,
        frozen: bool,
    ) -> Result<FreezeReceipt, LayerError> {
        if auth.index_a >= auth.index_b {
            return Err(LayerError::NotTheIssuer);
        }
        if !self.records.contains_key(&account_index) {
            return Err(LayerError::AccountNotFound(account_index));
        }
        if self.is_frozen(account_index) == frozen {
            return Err(LayerError::AlreadyInThatFreezeState);
        }

        let path = self.frozen.path_for(account_index);
        let trace = build_freeze_trace(auth, !frozen, frozen, &path, self.freeze_count, 1);

        let prover = FreezeProver::new(self.options.clone());
        let public_inputs = prover.get_pub_inputs(&trace);
        let proof = prover
            .prove(trace)
            .map_err(|e| LayerError::ProofFailed(format!("{e:?}")))?;

        Ok(FreezeReceipt {
            proof: proof.to_bytes(),
            public_inputs,
            now_frozen: frozen,
        })
    }

    /// Verifica y aplica una congelación.
    pub fn apply_freeze(
        &mut self,
        receipt: &FreezeReceipt,
        account_index: AccountIndex,
    ) -> Result<(), LayerError> {
        let pi = &receipt.public_inputs;

        if pi.custodian_set_root != self.custodian_set_root {
            return Err(LayerError::NotTheIssuer);
        }
        if pi.frozen_root_old != self.frozen.root()
            || pi.freeze_count_old != BaseElement::new(self.freeze_count)
        {
            return Err(LayerError::StaleState);
        }

        let proof = winterfell::Proof::from_bytes(&receipt.proof)
            .map_err(|e| LayerError::VerificationFailed(format!("prueba mal formada: {e:?}")))?;
        let min_opts = AcceptableOptions::OptionSet(vec![self.options.clone()]);
        verify::<FreezeAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
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

        // Se comprueba sobre una copia: ver el comentario de `mint`.
        let mut tentativo = self.frozen.clone();
        tentativo.set_leaf(account_index, frozen_leaf(receipt.now_frozen));
        if tentativo.root() != pi.frozen_root_new {
            return Err(LayerError::StaleState);
        }

        self.frozen = tentativo;
        self.freeze_count = pi.freeze_count_new.as_int();

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
            .append(OpKind::Freeze, raiz, raiz, &receipt.proof);
        self.commit(&[], None, None)?;
        Ok(())
    }
}
