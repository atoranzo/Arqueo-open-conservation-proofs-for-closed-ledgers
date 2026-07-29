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
use crate::commitment::ClientState;
use stark_experiment::circuit_mint_pending::{
    build_trace as build_mint_pending_trace, MintPendingProver,
    MintPendingPublicInputs,
};
use crate::pending::pending_commitment;
use stark_experiment::circuit_claim::{
    build_trace as build_claim_trace, ClaimProver, ClaimPublicInputs,
};
use stark_experiment::circuit_send::{
    build_trace as build_send_trace, SendProver, SendPublicInputs,
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
pub struct MintPendingReceipt {
    pub proof: Vec<u8>,
    pub public_inputs: MintPendingPublicInputs,
    pub commitment: Digest,
    /// El aviso que hay que hacer llegar al destinatario.
    pub notice: PendingNotice,
}

#[derive(Debug)]
pub struct ClaimReceipt {
    pub proof: Vec<u8>,
    pub public_inputs: ClaimPublicInputs,
}

impl SovereignLayer {
    /// **Cuánto dinero hay en tránsito: la suma de los pendientes sin
    /// cobrar.**
    ///
    /// ## Por qué hace falta
    ///
    /// La invariante global del sistema era `suma de saldos == suministro`.
    /// **Con la vía en dos fases deja de ser cierta**: el dinero sale de la
    /// cuenta del pagador y espera en un pendiente que no está en ningún
    /// saldo.
    ///
    /// La correcta es:
    ///
    /// ```text
    /// suma de saldos + total_pending() == suministro
    /// ```
    ///
    /// ⚠️ **El descuadre existía y nada lo detectaba.** El test que
    /// comprueba la invariante usa `transfer()`, la vía antigua, que abona
    /// al receptor en el acto. Es el modo de fallo que este proyecto
    /// documenta en otros sitios: **una propiedad que se cree comprobada
    /// porque hay un test con ese nombre, y el test ejercita otro camino.**
    ///
    /// ## ⚠️ Qué revela y qué no
    ///
    /// Revela **cuánto** hay en tránsito en total. **No revela de quién ni
    /// para quién**: eso sigue en los compromisos, que no se abren.
    ///
    /// Que el total sea visible es coherente con el modelo declarado —el
    /// suministro y el tope ya son escalares públicos— pero **es
    /// información que antes no existía**, y conviene decirlo.
    pub fn total_pending(&self) -> u64 {
        self.pending_amounts.values().sum()
    }

    /// **Asigna una posición en el árbol de pendientes, REUTILIZANDO las
    /// que quedaron libres al cobrarse.**
    ///
    /// ## Por qué existe
    ///
    /// Antes el contador solo subía, así que el límite no era de
    /// transferencias **simultáneas** sino **totales desde el inicio**: a
    /// mil pagos por segundo, 2³² posiciones se agotaban en unos cincuenta
    /// días. Ver `AUDITORIA.md` §13.
    ///
    /// Y las posiciones **ya se liberaban**: `apply_claim` pone la hoja a
    /// cero al cobrarse el pendiente. Nadie las reutilizaba.
    ///
    /// **No hizo falta tocar el circuito.** Éste demuestra que la posición
    /// estaba vacía y pasa a contener el compromiso, y eso vale igual para
    /// una posición nueva que para una reciclada.
    ///
    /// ## ⚠️ Coste
    ///
    /// Busca linealmente desde cero, así que es **O(pendientes creados)**
    /// en el peor caso. Es aceptable para una demostración y **no lo sería
    /// en producción**: ahí haría falta una lista de libres persistida.
    ///
    /// Se prefirió lo simple y comprobable a lo rápido y sutil.
    /// Toma `&self`: **no muta nada**. Es coherente con `send`, que solo
    /// genera la prueba; el estado lo cambia `apply_send`.
    fn allocate_pending(&self) -> Result<u64, LayerError> {
        for p in 0..self.next_pending {
            if !self.pending.is_occupied(p) {
                return Ok(p);
            }
        }
        if self.next_pending >= self.pending.capacity() {
            return Err(LayerError::PendingTreeExhausted {
                capacity: self.pending.capacity(),
            });
        }
        Ok(self.next_pending)
    }

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
        // **El estado lo aporta el titular, no la capa.**
        //
        // Es el modelo por compromisos aplicado a esta via: la capa
        // **comprueba** que el estado declarado produce la hoja que tiene
        // en el arbol, y **no necesita conocer el saldo** para ello.
        //
        // Ver `commitment.rs`, donde el diseño está demostrado.
        sender_state: &ClientState,
        receiver_id: Digest,
        salt: Digest,
        amount: u64,
    ) -> Result<SendReceipt, LayerError> {
        // La hoja que el estado declarado produce debe ser la que está en
        // el árbol. Si el titular mintiera sobre su saldo, no coincidiría.
        let hoja = native_leaf(
            sender_state.public_id,
            BaseElement::new(sender_state.balance),
            sender_state.nonce,
        );
        if hoja != self.accounts.leaf(sender_index) {
            return Err(LayerError::StaleState);
        }
        let sender = sender_state.clone();

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

        let position = self.allocate_pending()?;
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
            // **El límite lo aporta la capa, y ahora el circuito lo
            // demuestra.** Antes solo se comprobaba aquí, al generar: quien
            // construyera su propia traza podía saltárselo. Ver
            // `AUDITORIA.md` §25.
            self.regulatory_limit,
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
        sender_state: &ClientState,
        amount: u64,
    ) -> Result<(), LayerError> {
        let pi = &receipt.public_inputs;
        if pi.root_old != self.accounts.root() || pi.pending_root_old != self.pending.root() {
            return Err(LayerError::StaleState);
        }
        if pi.frozen_root != self.frozen.root() {
            return Err(LayerError::StaleState);
        }

        // ===== EL LIMITE REGULATORIO =====
        //
        // ⚠️ **Se comprueba sobre el importe PROBADO, no sobre el
        // parametro.** `send()` ya lo comprueba al generar, pero eso solo
        // ata a quien use esa funcion: quien construya su propia traza y su
        // propia prueba puede llamar directamente a `apply_send`.
        //
        // **Sin esto, el limite regulatorio no se imponia en esta via.**
        // `circuit_settlement` —la via antigua— lo lleva como entrada
        // publica, y su `apply` comprobaba que la declarada fuera la del
        // sistema. Al sustituir una via por otra **se perdio esa
        // comprobacion**. Ver `AUDITORIA.md` §25.
        //
        // ⚠️ **Esto es de CAPA, no de circuito.** `circuit_send` no lleva
        // el limite como entrada publica, asi que un tercero que solo tenga
        // la prueba **no puede verificar que se respeto**. La via antigua
        // si lo permitia. **Cerrarlo exige anadir el limite al circuito**, y
        // no esta hecho.
        // El circuito prueba `importe <= limite DECLARADO en la traza`.
        // La capa comprueba que ese limite declarado **sea el suyo**.
        //
        // Las dos juntas dan `importe <= limite del sistema`, y a
        // diferencia de una comprobacion solo de capa, **un tercero con la
        // prueba puede verificar la primera mitad**.
        //
        // Es la misma composicion que tenia `circuit_settlement` + `apply`
        // en la via antigua, y que se perdio al sustituirla.
        let limite_declarado = pi.regulatory_limit.as_int();
        if limite_declarado != self.regulatory_limit {
            return Err(LayerError::WrongRegulatoryLimit {
                expected: self.regulatory_limit,
                declared: limite_declarado,
            });
        }

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
        let updated = ClientState {
            balance: sender_state.balance - amount,
            ..sender_state.clone()
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
        // ⚠️ **Se mantiene el registro por compatibilidad con la vía
        // antigua**, que sí lo necesita. La vía nueva **no lo lee en
        // ningún punto**: el saldo viene del titular y se verifica contra
        // la hoja.
        //
        // Cuando `transfer()` desaparezca, esto también.
        self.records.insert(
            sender_index,
            AccountRecord {
                public_id: updated.public_id,
                balance: updated.balance,
                nonce: updated.nonce,
            },
        );
        // Solo avanza si la posicion era nueva. Si se reutilizo una
        // liberada, el contador ya estaba mas adelante.
        if pos >= self.next_pending {
            self.next_pending = pos + 1;
        }
        // El dinero sale del saldo y pasa a estar en transito.
        self.pending_amounts.insert(pos, amount);
        self.commit(&[sender_index], None, Some((pos, compromiso)))?;
        Ok(())
    }

    /// **FASE 2.** El receptor demuestra que el pendiente es suyo y cobra.
    pub fn claim(
        &self,
        spend_key: BaseElement,
        receiver_index: AccountIndex,
        // El estado lo aporta el titular: ver el comentario de `send`.
        receiver_state: &ClientState,
        notice: &PendingNotice,
    ) -> Result<ClaimReceipt, LayerError> {
        let hoja = native_leaf(
            receiver_state.public_id,
            BaseElement::new(receiver_state.balance),
            receiver_state.nonce,
        );
        if hoja != self.accounts.leaf(receiver_index) {
            return Err(LayerError::StaleState);
        }
        let receiver = receiver_state.clone();

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
        receiver_state: &ClientState,
        notice: &PendingNotice,
    ) -> Result<(), LayerError> {
        let pi = &receipt.public_inputs;
        if pi.root_old != self.accounts.root() || pi.pending_root_old != self.pending.root() {
            return Err(LayerError::StaleState);
        }

        // El nonce tampoco se incrementa: ver el comentario de `apply_send`.
        let updated = ClientState {
            balance: receiver_state.balance + notice.amount,
            ..receiver_state.clone()
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
        // Y deja de estar en transito al cobrarse.
        self.pending_amounts.remove(&notice.position);

        if cuentas.root() != pi.root_new || pend.root() != pi.pending_root_new {
            return Err(LayerError::StaleState);
        }

        self.accounts = cuentas;
        self.pending = pend;
        // Ver el comentario de `apply_send`: compatibilidad, no lectura.
        self.records.insert(
            receiver_index,
            AccountRecord {
                public_id: updated.public_id,
                balance: updated.balance,
                nonce: updated.nonce,
            },
        );
        self.commit(&[receiver_index], None, Some((notice.position, vacia)))?;
        Ok(())
    }


    /// **EMISIÓN A UN PENDIENTE.**
    ///
    /// Dos custodios crean dinero **sin tocar ninguna cuenta**: depositan
    /// un compromiso atado a la identidad del destinatario, que este
    /// reclama con `claim`.
    ///
    /// La emisión clásica acredita una cuenta directamente, y para ello
    /// **necesita su saldo**. Aquí no.
    ///
    /// ⚠️ **Si el destinatario nunca reclama, el dinero queda en el
    /// limbo**: el suministro subió y no está en ninguna cuenta. Haría
    /// falta devolución tras un plazo, y esta capa **no tiene noción de
    /// tiempo**.
    pub fn mint_to_pending(
        &self,
        auth: &ThresholdAuth,
        receiver_id: Digest,
        salt: Digest,
        amount: u64,
    ) -> Result<MintPendingReceipt, LayerError> {
        if auth.index_a >= auth.index_b {
            return Err(LayerError::NotTheIssuer);
        }
        if self.total_supply + amount > self.max_supply {
            return Err(LayerError::OverRegulatoryLimit {
                limit: self.max_supply,
                requested: amount,
            });
        }

        let position = self.allocate_pending()?;
        let pending_path = self.pending.path_for(position);
        let trace = build_mint_pending_trace(
            auth.key_a,
            auth.index_a,
            &auth.path_a,
            auth.key_b,
            auth.index_b,
            &auth.path_b,
            self.total_supply,
            amount,
            self.max_supply,
            amount,
            receiver_id,
            salt,
            &pending_path,
        );
        let prover = MintPendingProver::new(self.options.clone());
        let public_inputs = prover.get_pub_inputs(&trace);
        let proof = prover
            .prove(trace)
            .map_err(|e| LayerError::ProofFailed(format!("{e:?}")))?;

        Ok(MintPendingReceipt {
            proof: proof.to_bytes(),
            public_inputs,
            commitment: pending_commitment(receiver_id, salt, amount),
            notice: PendingNotice {
                position,
                salt,
                amount,
            },
        })
    }

    /// Aplica una emisión a pendiente: sube el suministro y deposita.
    pub fn apply_mint_to_pending(
        &mut self,
        receipt: &MintPendingReceipt,
    ) -> Result<(), LayerError> {
        let pi = &receipt.public_inputs;
        if pi.custodian_set_root != self.custodian_set_root {
            return Err(LayerError::StaleState);
        }
        if pi.pending_root_old != self.pending.root() {
            return Err(LayerError::StaleState);
        }
        if pi.supply_old.as_int() != self.total_supply {
            return Err(LayerError::StaleState);
        }

        // Sobre una copia: si la raíz no cuadra, el estado queda intacto.
        let pos = receipt.notice.position;
        let mut pend = self.pending.clone();
        pend.set_leaf(pos, receipt.commitment);
        if pend.root() != pi.pending_root_new {
            return Err(LayerError::StaleState);
        }

        // ===== ROTACIÓN: consume una intervención del conjunto =====
        self.consume_custodian_use()?;

        self.pending = pend;
        self.total_supply = pi.supply_new.as_int();
        // Solo avanza si la posicion era nueva. Si se reutilizo una
        // liberada, el contador ya estaba mas adelante.
        if pos >= self.next_pending {
            self.next_pending = pos + 1;
        }
        // Dinero recien emitido, en transito hasta que se cobre.
        self.pending_amounts.insert(pos, receipt.notice.amount);
        self.commit(&[], None, Some((pos, receipt.commitment)))?;
        Ok(())
    }
}
