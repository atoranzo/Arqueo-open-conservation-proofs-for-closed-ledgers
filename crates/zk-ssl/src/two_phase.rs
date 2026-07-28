//! **Transferencia en dos fases**: enviar y reclamar.
//!
//! Sustituye a la liquidación de un solo paso para cerrar la fuga que
//! esta tenía: **el pagador necesitaba el saldo del receptor** porque la
//! prueba actualizaba las dos hojas.
//!
//! ```text
//! FASE 1: send                  FASE 2: claim
//! · debita la cuenta del        · el receptor demuestra que el
//!   pagador                       pendiente es suyo
//! · deposita un compromiso      · acredita su cuenta
//!   atado a la identidad del    · el pendiente queda consumido
//!   receptor
//! ```
//!
//! ## Qué necesita cada parte
//!
//! | Parte | Necesita | NO necesita |
//! |---|---|---|
//! | Pagador | La identidad pública del receptor, como dirección | **Su saldo. Ni su nonce.** |
//! | Receptor | Su estado y el aviso del pagador | Nada del pagador |
//!
//! ## ⚠️ El aviso viaja fuera del sistema
//!
//! El receptor necesita el **aleatorio y el importe** para reconstruir el
//! compromiso. El pagador se los envía por su cuenta: mensaje, código QR,
//! lo que sea.
//!
//! **Esta capa no lo transporta**, y perder el aviso significa no poder
//! reclamar aunque el dinero esté ahí.
//!
//! ## ⚠️ Y el residuo del diseño
//!
//! El pagador elige el aleatorio, así que **reconoce el compromiso y ve
//! cuándo desaparece del árbol**. Sabe *cuándo* cobra el receptor, no
//! cuánto tiene.
//!
//! Es mucho menor que revelar el saldo, pero es una fuga de
//! vinculabilidad. Zcash la cierra cifrando la nota para que el receptor
//! derive el aleatorio; **aquí no está resuelto**.

use super::*;
use crate::pending::pending_commitment;
use stark_experiment::circuit_claim::{
    build_trace as build_claim_trace, ClaimAir, ClaimProver, ClaimPublicInputs,
};
use stark_experiment::circuit_send::{
    build_trace as build_send_trace, SendAir, SendProver, SendPublicInputs,
};

/// Lo que el pagador envía al receptor **por otro canal**.
///
/// Sin esto el receptor no puede reclamar: necesita el aleatorio y el
/// importe para reconstruir el compromiso.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingNotice {
    /// Posición del compromiso en el árbol.
    pub position: u64,
    /// Aleatorio que eligió el pagador.
    pub salt: Digest,
    pub amount: u64,
}

#[derive(Debug)]
pub struct SendReceipt {
    pub proof: Vec<u8>,
    pub public_inputs: SendPublicInputs,
    /// El compromiso depositado.
    ///
    /// La capa lo coloca y **comprueba que produce la raíz declarada**: si
    /// el recibo mintiera, esa comprobación falla. No hace falta confiar
    /// en este campo.
    pub commitment: Digest,
    /// El aviso que hay que hacer llegar al receptor.
    pub notice: PendingNotice,
}

#[derive(Debug)]
pub struct ClaimReceipt {
    pub proof: Vec<u8>,
    pub public_inputs: ClaimPublicInputs,
}

impl SovereignLayer {
    /// **FASE 1.** El pagador debita su cuenta y deposita el compromiso.
    ///
    /// `receiver_id` es la **identidad pública** del receptor, que
    /// funciona como dirección.
    ///
    /// ⚠️ **Obtenla del propio receptor, no de esta capa.** Si el operador
    /// te da otra identidad, el dinero irá a otra cuenta y la prueba será
    /// válida: las entradas públicas no dicen quién recibe.
    pub fn send(
        &self,
        spend_key: BaseElement,
        sender_index: AccountIndex,
        receiver_id: Digest,
        salt: Digest,
        amount: u64,
    ) -> Result<SendReceipt, LayerError> {
        let sender = self
            .records
            .get(&sender_index)
            .ok_or(LayerError::AccountNotFound(sender_index))?
            .clone();

        if derive_public_id(spend_key) != sender.public_id {
            return Err(LayerError::NotTheAccountHolder);
        }

        // Después de la autoridad: antes filtraría el estado de congelación
        // a quien no es el titular.
        if self.is_frozen(sender_index) {
            return Err(LayerError::AccountFrozen(sender_index));
        }
        if sender.balance < amount {
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

        let position = self.next_pending;
        let path = self.accounts.path_for(sender_index);
        let frozen_path = self.frozen.path_for(sender_index);
        let pending_path = self.pending.path_for(position);

        let trace = build_send_trace(
            spend_key,
            sender.public_id,
            sender.balance,
            sender.nonce,
            &path,
            &frozen_path,
            amount,
            self.total_supply,
            0, // un envío no cambia el suministro
            receiver_id,
            salt,
            &pending_path,
        );
        let prover = SendProver::new(self.options.clone());
        let public_inputs = prover.get_pub_inputs(&trace);
        let proof = prover
            .prove(trace)
            .map_err(|e| LayerError::ProofFailed(format!("{e:?}")))?;

        let commitment = pending_commitment(receiver_id, salt, amount);

        Ok(SendReceipt {
            proof: proof.to_bytes(),
            public_inputs,
            commitment,
            notice: PendingNotice {
                position,
                salt,
                amount,
            },
        })
    }

    /// Aplica un envío: debita y deposita el compromiso.
    pub fn apply_send(
        &mut self,
        receipt: &SendReceipt,
        sender_index: AccountIndex,
        amount: u64,
    ) -> Result<(), LayerError> {
        let pi = &receipt.public_inputs;
        if pi.root_old != self.accounts.root() || pi.pending_root_old != self.pending.root() {
            return Err(LayerError::StaleState);
        }
        if pi.frozen_root != self.frozen.root() {
            return Err(LayerError::StaleState);
        }

        let sender = self
            .records
            .get(&sender_index)
            .ok_or(LayerError::AccountNotFound(sender_index))?
            .clone();
        // ⚠️ **El nonce NO se incrementa.**
        //
        // `circuit_send` lo heredó de `circuit_burn`, que tampoco lo
        // incrementa. Si la capa lo hiciera, la hoja resultante seria otra
        // y la raiz no cuadraria con la que la prueba acredita.
        //
        // La proteccion contra reenvio viene del encadenamiento de raices:
        // un segundo intento tendria `root_old` obsoleta. Es lo mismo que
        // hace la destruccion.
        //
        // `circuit_settlement` SI lo incrementa. La diferencia es
        // deliberada aqui, pero conviene saberla.
        let updated = AccountRecord {
            balance: sender.balance - amount,
            ..sender
        };

        // Sobre copias: si las raíces no cuadran, el estado queda intacto.
        let mut cuentas = self.accounts.clone();
        cuentas.set_leaf(
            sender_index,
            native_leaf(
                updated.public_id,
                BaseElement::new(updated.balance),
                updated.nonce,
            ),
        );
        let pos = receipt.notice.position;
        let compromiso = receipt.commitment;
        let mut pend = self.pending.clone();
        pend.set_leaf(pos, compromiso);

        if cuentas.root() != pi.root_new || pend.root() != pi.pending_root_new {
            return Err(LayerError::StaleState);
        }

        self.accounts = cuentas;
        self.pending = pend;
        self.records.insert(sender_index, updated);
        self.next_pending = pos + 1;
        self.commit(&[sender_index], None, Some((pos, compromiso)))?;
        Ok(())
    }

    /// **FASE 2.** El receptor demuestra que el pendiente es suyo y cobra.
    pub fn claim(
        &self,
        spend_key: BaseElement,
        receiver_index: AccountIndex,
        notice: &PendingNotice,
    ) -> Result<ClaimReceipt, LayerError> {
        let receiver = self
            .records
            .get(&receiver_index)
            .ok_or(LayerError::AccountNotFound(receiver_index))?
            .clone();

        if derive_public_id(spend_key) != receiver.public_id {
            return Err(LayerError::NotTheAccountHolder);
        }
        if self.is_frozen(receiver_index) {
            return Err(LayerError::AccountFrozen(receiver_index));
        }

        let path = self.accounts.path_for(receiver_index);
        let frozen_path = self.frozen.path_for(receiver_index);
        let pending_path = self.pending.path_for(notice.position);

        let trace = build_claim_trace(
            spend_key,
            receiver.public_id,
            receiver.balance,
            receiver.nonce,
            &path,
            &frozen_path,
            notice.amount,
            self.total_supply,
            0,
            receiver.public_id,
            notice.salt,
            &pending_path,
        );
        let prover = ClaimProver::new(self.options.clone());
        let public_inputs = prover.get_pub_inputs(&trace);
        let proof = prover
            .prove(trace)
            .map_err(|e| LayerError::ProofFailed(format!("{e:?}")))?;

        Ok(ClaimReceipt {
            proof: proof.to_bytes(),
            public_inputs,
        })
    }

    /// Aplica una reclamación: acredita y consume el pendiente.
    pub fn apply_claim(
        &mut self,
        receipt: &ClaimReceipt,
        receiver_index: AccountIndex,
        notice: &PendingNotice,
    ) -> Result<(), LayerError> {
        let pi = &receipt.public_inputs;
        if pi.root_old != self.accounts.root() || pi.pending_root_old != self.pending.root() {
            return Err(LayerError::StaleState);
        }

        let receiver = self
            .records
            .get(&receiver_index)
            .ok_or(LayerError::AccountNotFound(receiver_index))?
            .clone();
        // El nonce tampoco se incrementa: ver el comentario de `apply_send`.
        let updated = AccountRecord {
            balance: receiver.balance + notice.amount,
            ..receiver
        };

        let mut cuentas = self.accounts.clone();
        cuentas.set_leaf(
            receiver_index,
            native_leaf(
                updated.public_id,
                BaseElement::new(updated.balance),
                updated.nonce,
            ),
        );
        // **Consumido**: la hoja vuelve a estar vacía. Sin esto, el mismo
        // pendiente se cobraría indefinidamente.
        let vacia: Digest = [BaseElement::ZERO; 4];
        let mut pend = self.pending.clone();
        pend.set_leaf(notice.position, vacia);

        if cuentas.root() != pi.root_new || pend.root() != pi.pending_root_new {
            return Err(LayerError::StaleState);
        }

        self.accounts = cuentas;
        self.pending = pend;
        self.records.insert(receiver_index, updated);
        self.commit(&[receiver_index], None, Some((notice.position, vacia)))?;
        Ok(())
    }

}
