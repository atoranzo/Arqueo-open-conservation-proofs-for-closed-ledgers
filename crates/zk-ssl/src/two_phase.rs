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
use crate::log::OpKind;
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
    /// cero al cobrarse el pendiente, y el bucle de abajo **las reutiliza**:
    /// devuelve el primer hueco libre desde cero antes de `next_pending`.
    /// (Una version anterior de este comentario decia «Nadie las
    /// reutilizaba», contradiciendo al parrafo siguiente y al codigo.)
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
    // `pub(crate)` y no privado: `client.rs` la necesita para entregar
    // materiales de envio sin que el cliente hable dos veces con la capa.
    pub(crate) fn allocate_pending(&self) -> Result<u64, LayerError> {
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
    /// ⚠️ **No comprueba que el receptor exista, y no puede.**
    ///
    /// El receptor se identifica por su **identificador público**, no por un
    /// índice. Comprobar que alguien lo tenga exigiría que la capa revelara
    /// quién está en el árbol, que es lo que este diseño evita.
    ///
    /// **Consecuencia**: enviar a un identificador inventado funciona, el
    /// dinero sale, y queda en un pendiente que nadie puede cobrar. **No hay
    /// devolución.** Ver `AUDITORIA.md` §30.
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
        // ⚠️ **Deja constancia ANTES de persistir**, igual que las demas
        // operaciones: si el proceso muere en medio, el lote atomico
        // incluye o excluye las dos cosas.
        //
        // `two_phase.rs` era **el unico modulo que no registraba nada**.
        self.log
            .append(OpKind::Send, pi.root_old, pi.root_new, &receipt.proof);
        self.commit(&[sender_index], Some((pos, compromiso)))?;
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
        // ⚠️ **Deja constancia ANTES de persistir**, igual que las demas
        // operaciones: si el proceso muere en medio, el lote atomico
        // incluye o excluye las dos cosas.
        //
        // `two_phase.rs` era **el unico modulo que no registraba nada**.
        self.log
            .append(OpKind::Claim, pi.root_old, pi.root_new, &receipt.proof);
        self.commit(&[receiver_index], Some((notice.position, vacia)))?;
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
        // ⚠️ **Deja constancia ANTES de persistir**, igual que las demas
        // operaciones: si el proceso muere en medio, el lote atomico
        // incluye o excluye las dos cosas.
        //
        // `two_phase.rs` era **el unico modulo que no registraba nada**.
        self.log
            // ⚠️ **La misma raiz en los dos lados, y es correcto.**
            //
            // El registro encadena la raiz de CUENTAS, y una emision a un
            // pendiente **no la toca**: el dinero aparece en el arbol de
            // pendientes, no en ninguna cuenta.
            //
            // Declarar aqui las raices de pendientes romperia la cadena:
            // `verify` comprueba que cada entrada parta de donde acabo la
            // anterior, y mezclar arboles la haria fallar.
            //
            // ⚠️ **Coste declarado**: quien lea el registro ve que hubo una
            // emision a pendiente, pero **la raiz no le dice cual**. Para
            // eso haria falta encadenar tambien la de pendientes, y esta
            // capa no lo hace.
            .append(
                OpKind::MintToPending,
                self.accounts.root(),
                self.accounts.root(),
                &receipt.proof,
            );
        self.commit(&[], Some((pos, receipt.commitment)))?;
        Ok(())
    }

    // -----------------------------------------------------------------

    /// Emite a un pendiente **sin que las claves de custodio lleguen al
    /// operador**: la QUINTA y ultima de la entrada 32/33 (68).
    ///
    /// Con esta via, el fallo de la entrada 32 queda cerrado: ninguna de
    /// las cinco operaciones privilegiadas exige ya las claves en crudo.
    ///
    /// Tres pruebas: `climb_proof` de `circuit_mint_pending_climb` -que el
    /// suministro sube EXACTAMENTE el importe, que el compromiso entra en
    /// una posicion LIBRE y que no se pasa del tope- y dos de custodios
    /// distintos que autorizan ESTA emision.
    ///
    /// ## La posicion la asigna la capa, y eso no es un cabo suelto
    ///
    /// Quien genera la prueba necesita el camino del arbol, asi que tuvo
    /// que conocer la posicion antes. Si asigno otra, la raiz nueva no
    /// coincide y **la verificacion de la subida falla**: cierra en falso,
    /// no en abierto.
    ///
    /// ## El tope se comprueba DOS veces, y no es redundancia (67.1)
    ///
    /// Aqui porque la capa conoce el suministro real y puede rechazar
    /// antes de nada; en el circuito porque un auditor externo que solo ve
    /// el registro **no puede recomputar el suministro**.
    ///
    /// ⚠️ **Usa `SupplyCapExceeded`, no `OverRegulatoryLimit`.** La via
    /// antigua de arriba devuelve el segundo para esta misma condicion, que
    /// es el error del limite regulatorio de una transferencia, no el del
    /// tope de emision -y ademas suma sin saturar-. `mint` siempre uso
    /// `SupplyCapExceeded`. La divergencia se deja anotada en vez de
    /// corregirse de tapadillo: tocar el error de la via antigua cambia lo
    /// que ven sus tests y es otra tarea.
    #[allow(clippy::too_many_arguments)]
    pub fn apply_mint_pending_delegated(
        &mut self,
        climb_proof: winterfell::Proof,
        proof_a: winterfell::Proof,
        inputs_a: stark_experiment::circuit_threshold_single_nullifier::NullifierThresholdPublicInputs,
        proof_b: winterfell::Proof,
        inputs_b: stark_experiment::circuit_threshold_single_nullifier::NullifierThresholdPublicInputs,
        receiver_id: Digest,
        salt: Digest,
        amount: u64,
    ) -> Result<PendingNotice, LayerError> {
        use stark_experiment::circuit_mint_pending_climb::{
            MintPendingClimbAir, MintPendingClimbPublicInputs,
        };
        use stark_experiment::circuit_threshold::CUSTODIAN_DOMAIN;
        use stark_experiment::circuit_threshold_single_nullifier::{
            commit_operation, verify_threshold_pair, PairRejection, OP_MINT_PENDING,
        };

        let would_be = self.total_supply.saturating_add(amount);
        if would_be > self.max_supply {
            return Err(LayerError::SupplyCapExceeded {
                cap: self.max_supply,
                would_be,
            });
        }

        let position = self.allocate_pending()?;
        let commitment = pending_commitment(receiver_id, salt, amount);

        let root_old = self.pending.root();
        let supply_old = self.total_supply;
        let supply_new = supply_old + amount;

        // Sobre una COPIA, no sobre el estado. Si algo falla despues, el
        // arbol real no se ha tocado.
        let mut tentativo = self.pending.clone();
        tentativo.set_leaf(position, commitment);
        let root_new = tentativo.root();

        let accepted = AcceptableOptions::OptionSet(vec![self.options.clone()]);

        verify::<MintPendingClimbAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            climb_proof,
            MintPendingClimbPublicInputs {
                supply_old: BaseElement::new(supply_old),
                supply_new: BaseElement::new(supply_new),
                max_supply: BaseElement::new(self.max_supply),
                amount: BaseElement::new(amount),
                pending_root_old: root_old,
                pending_root_new: root_new,
            },
            &accepted,
        )
        .map_err(|e| LayerError::VerificationFailed(format!("subida del pendiente: {e:?}")))?;

        // El compromiso cubre TODO lo que la emision decide: de que raiz de
        // pendientes a cual -lo que fija posicion Y compromiso-, cuanto, y
        // contra que suministro y que tope.
        let mut params: Vec<BaseElement> = root_old.to_vec();
        params.extend_from_slice(&root_new);
        params.push(BaseElement::new(amount));
        params.push(BaseElement::new(supply_old));
        params.push(BaseElement::new(supply_new));
        params.push(BaseElement::new(self.max_supply));
        let operation = commit_operation(OP_MINT_PENDING, &params);

        verify_threshold_pair(
            proof_a,
            inputs_a,
            proof_b,
            inputs_b,
            BaseElement::new(CUSTODIAN_DOMAIN),
            self.custodian_set_root,
            operation,
            &accepted,
        )
        .map_err(|r| match r {
            PairRejection::SameCustodian
            | PairRejection::WrongCustodianSet
            | PairRejection::WrongIdentityDomain => LayerError::NotTheIssuer,
            PairRejection::WrongOperation => LayerError::StaleState,
            PairRejection::InvalidProof => {
                LayerError::VerificationFailed("autorizacion invalida".into())
            }
        })?;

        // Despues de verificar la autoridad: si fuera antes, cualquiera
        // podria agotar el cupo de los custodios sin serlo.
        self.consume_custodian_use()?;

        self.pending = tentativo;
        self.total_supply = supply_new;
        if position >= self.next_pending {
            self.next_pending = position + 1;
        }
        self.pending_amounts.insert(position, amount);

        // La MISMA raiz de cuentas en los dos lados, y es correcto: una
        // emision a un pendiente no toca ninguna cuenta. Mismo criterio que
        // la via antigua de arriba.
        self.log.append(
            OpKind::MintToPending,
            self.accounts.root(),
            self.accounts.root(),
            &[],
        );
        self.commit(&[], Some((position, commitment)))?;

        Ok(PendingNotice {
            position,
            salt,
            amount,
        })
    }
}

#[cfg(test)]
mod tests_delegada {
    use super::*;
    use crate::tests_support::*;
    use stark_experiment::circuit_mint_pending_climb as climb;
    use stark_experiment::circuit_threshold::{build_custodian_set, CUSTODIAN_DOMAIN};
    use stark_experiment::circuit_threshold_single_nullifier as auth;

    const AMOUNT: u64 = 250_000;

    // ===== LOS CINCO SE SALTAN EN DEPURACION, Y ESTA MEDIDO POR QUE =====
    //
    // `allocate_pending` devuelve 0 en una capa nueva, y **la posicion 0
    // tiene el camino de Merkle todo a la izquierda**: `COL_PBIT` queda
    // identicamente nula, los veinte terminos `pbit * X` se anulan y las
    // restricciones `C_PEND_ENTRY_A/B`, `C_PEND_PLACE` y `C_PEND_SIBLING`
    // caen de grado 2 a 1 -indices 50-69, de 1022 a 511-, y `C_PBIT_BOOL`
    // de 511 a 0. Winterfell comprueba en depuracion que el grado declarado
    // se realice, y rechaza.
    //
    // **No es un fallo de solidez.** En release generan y verifican: lo
    // prueban estos mismos cinco y, en el circuito,
    // `the_all_left_path_of_position_zero_still_verifies`, que usa el
    // camino degenerado a proposito.
    //
    // ⚠️ **No se cambia el test a otra posicion para que pase.** En
    // produccion la posicion 0 SE USA -`allocate_pending` reutiliza huecos
    // y un ledger recae en ella (46.1)-, asi que un test en la posicion 1
    // pasaria sin ejercitar el caso comun. Vale mas un test fiel saltado
    // que uno verde que mira a otro lado.
    //
    // Es la decision de la entrada 6, tomada en 46: **se declara, no se
    // migra**. Limite conocido de winterfell (entradas 24, 25, 34).
    //
    // ⚠️ **Los DOCE fallos de depuracion de `mint`, `freeze` y `recovery`
    // NO llevan esta marca.** Divergen en otros indices -44 y 73-88- y su
    // causa **no esta medida**. Ponerles este motivo seria atribuirles una
    // causa sin comprobar. Van en su propia entrada de backlog.

    fn dominio() -> BaseElement {
        BaseElement::new(CUSTODIAN_DOMAIN)
    }

    /// El compromiso de operacion, calculado como lo calculara la capa.
    fn compromiso(layer: &SovereignLayer, amount: u64) -> Digest {
        let position = layer.allocate_pending().expect("posicion libre");
        let c = pending_commitment(receptor(), salt_de(0x5EED), amount);
        let mut t = layer.pending.clone();
        t.set_leaf(position, c);

        let mut v: Vec<BaseElement> = layer.pending.root().to_vec();
        v.extend_from_slice(&t.root());
        v.push(BaseElement::new(amount));
        v.push(BaseElement::new(layer.total_supply()));
        v.push(BaseElement::new(layer.total_supply() + amount));
        v.push(BaseElement::new(MAX_SUPPLY));
        auth::commit_operation(auth::OP_MINT_PENDING, &v)
    }

    fn autorizar(
        key: BaseElement,
        path: &stark_experiment::circuit_threshold::CustodianPath,
        op: Digest,
    ) -> (winterfell::Proof, auth::NullifierThresholdPublicInputs) {
        let trace = auth::build_trace(dominio(), key, path, op);
        let prover = auth::NullifierThresholdProver::new(proof_options());
        let inputs = prover.get_pub_inputs(&trace);
        (prover.prove(trace).expect("autorizacion"), inputs)
    }

    /// Identidad publica del destinatario. **La dan los custodios, no la
    /// capa**: la capa no sabe de quien es cada pendiente.
    fn receptor() -> Digest {
        stark_experiment::circuit_settlement::derive_public_id(BaseElement::new(SK_BOB))
    }

    fn prueba_subida(layer: &SovereignLayer, amount: u64) -> winterfell::Proof {
        let position = layer.allocate_pending().expect("posicion libre");
        let path = layer.pending.path_for(position);
        let trace = climb::build_trace(
            layer.total_supply(),
            amount,
            MAX_SUPPLY,
            amount,
            receptor(),
            salt_de(0x5EED),
            &path,
        );
        climb::MintPendingClimbProver::new(proof_options())
            .prove(trace)
            .expect("subida")
    }

    // =================================================================
    // EL POSITIVO VA PRIMERO (guion, 66.2).
    // =================================================================

    /// **Dos custodios distintos emiten a un pendiente sin entregar sus
    /// claves.** La quinta y ultima: cierra la entrada 32.
    #[test]
    #[cfg_attr(
        debug_assertions,
        ignore = "grado dependiente del testigo: la posicion 0 anula COL_PBIT (entrada 6, 46, 68.3)"
    )]
    fn a_delegated_mint_to_pending_applies() {
        let mut layer = new_layer();
        let ck = custodian_keys();
        let (_, cp) = build_custodian_set(&ck);
        let antes = layer.total_supply();

        let op = compromiso(&layer, AMOUNT);
        let subida = prueba_subida(&layer, AMOUNT);
        let (pa, ia) = autorizar(ck[1], &cp[1], op);
        let (pb, ib) = autorizar(ck[3], &cp[3], op);

        let aviso = layer
            .apply_mint_pending_delegated(
                subida, pa, ia, pb, ib, receptor(), salt_de(0x5EED), AMOUNT,
            )
            .expect("la emision legitima debe aplicarse");

        assert_eq!(aviso.amount, AMOUNT);
        assert_eq!(
            layer.total_supply(),
            antes + AMOUNT,
            "el suministro debe subir"
        );
        assert_eq!(
            layer.total_pending(),
            AMOUNT,
            "y el dinero debe quedar en transito, no en una cuenta"
        );
    }

    /// **UNA SOLA CLAVE NO CREA DINERO.**
    ///
    /// Un 2-de-N en el que un custodio contara dos veces seria un 1-de-N
    /// disfrazado.
    #[test]
    #[cfg_attr(
        debug_assertions,
        ignore = "grado dependiente del testigo: la posicion 0 anula COL_PBIT (entrada 6, 46, 68.3)"
    )]
    fn the_same_custodian_twice_cannot_mint_to_pending() {
        let mut layer = new_layer();
        let ck = custodian_keys();
        let (_, cp) = build_custodian_set(&ck);
        let antes = layer.total_supply();
        let raiz_antes = layer.pending.root();

        let op = compromiso(&layer, AMOUNT);
        let subida = prueba_subida(&layer, AMOUNT);
        let (pa, ia) = autorizar(ck[2], &cp[2], op);
        let (pb, ib) = autorizar(ck[2], &cp[2], op);

        let r = layer.apply_mint_pending_delegated(
            subida, pa, ia, pb, ib, receptor(), salt_de(0x5EED), AMOUNT,
        );
        assert!(matches!(r, Err(LayerError::NotTheIssuer)), "fue {r:?}");
        assert_eq!(layer.total_supply(), antes, "no debe emitirse nada");
        assert_eq!(layer.pending.root(), raiz_antes, "ni depositarse nada");
    }

    /// **UNA AUTORIZACION PARA EMITIR X NO SIRVE PARA EMITIR MAS.**
    ///
    /// Es la propiedad de 67.2 aplicada al pendiente, y aqui cubre ademas
    /// la POSICION: el compromiso ata las dos raices del arbol, asi que una
    /// autorizacion tampoco vale para otro hueco.
    #[test]
    #[cfg_attr(
        debug_assertions,
        ignore = "grado dependiente del testigo: la posicion 0 anula COL_PBIT (entrada 6, 46, 68.3)"
    )]
    fn an_authorization_for_one_amount_does_not_mint_another_to_pending() {
        let mut layer = new_layer();
        let ck = custodian_keys();
        let (_, cp) = build_custodian_set(&ck);
        let antes = layer.total_supply();
        let raiz_antes = layer.pending.root();

        let op = compromiso(&layer, AMOUNT);
        let subida = prueba_subida(&layer, AMOUNT * 4);
        let (pa, ia) = autorizar(ck[1], &cp[1], op);
        let (pb, ib) = autorizar(ck[3], &cp[3], op);

        let r = layer.apply_mint_pending_delegated(
            subida, pa, ia, pb, ib, receptor(), salt_de(0x5EED), AMOUNT * 4,
        );
        assert!(
            r.is_err(),
            "SOLIDEZ: autorizar 250k no autoriza emitir 1M, fue {r:?}"
        );
        assert_eq!(layer.total_supply(), antes);
        assert_eq!(layer.pending.root(), raiz_antes);
    }

    /// **LA JERARQUIA: gobernanza no emite.**
    ///
    /// Las claves de gobernanza cambian quienes son los custodios; no
    /// pueden hacer el trabajo de los custodios. La separacion de dominio
    /// es lo que la hace real.
    #[test]
    #[cfg_attr(
        debug_assertions,
        ignore = "grado dependiente del testigo: la posicion 0 anula COL_PBIT (entrada 6, 46, 68.3)"
    )]
    fn governance_keys_cannot_mint_to_pending() {
        let mut layer = new_layer();
        let gk = governance_keys();
        let (_, gp) = build_custodian_set(&gk);
        let antes = layer.total_supply();
        let raiz_antes = layer.pending.root();

        let op = compromiso(&layer, AMOUNT);
        let subida = prueba_subida(&layer, AMOUNT);
        let (pa, ia) = autorizar(gk[1], &gp[1], op);
        let (pb, ib) = autorizar(gk[3], &gp[3], op);

        let r = layer.apply_mint_pending_delegated(
            subida, pa, ia, pb, ib, receptor(), salt_de(0x5EED), AMOUNT,
        );
        assert!(matches!(r, Err(LayerError::NotTheIssuer)), "fue {r:?}");
        assert_eq!(layer.total_supply(), antes);
        assert_eq!(layer.pending.root(), raiz_antes);
    }

    /// **EL TOPE, EN LA CAPA.** El circuito lo comprueba tambien, y no es
    /// redundancia: destinatarios distintos (67.1).
    #[test]
    #[cfg_attr(
        debug_assertions,
        ignore = "grado dependiente del testigo: la posicion 0 anula COL_PBIT (entrada 6, 46, 68.3)"
    )]
    fn the_layer_rejects_minting_over_the_cap() {
        let mut layer = new_layer();
        let ck = custodian_keys();
        let (_, cp) = build_custodian_set(&ck);

        let exceso = MAX_SUPPLY + 1;
        let op = compromiso(&layer, AMOUNT);
        let subida = prueba_subida(&layer, AMOUNT);
        let (pa, ia) = autorizar(ck[1], &cp[1], op);
        let (pb, ib) = autorizar(ck[3], &cp[3], op);

        let r = layer.apply_mint_pending_delegated(
            subida, pa, ia, pb, ib, receptor(), salt_de(0x5EED), exceso,
        );
        assert!(
            matches!(r, Err(LayerError::SupplyCapExceeded { .. })),
            "la capa debe rechazar antes de generar nada, fue {r:?}"
        );
        assert_eq!(layer.total_supply(), 0);
    }
}
