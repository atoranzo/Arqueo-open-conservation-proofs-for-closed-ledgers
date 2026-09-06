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
use crate::pending::pending_commitment;
use stark_experiment::circuit_claim::{
    build_trace as build_claim_trace, ClaimAir, ClaimProver, ClaimPublicInputs,
};
use stark_experiment::circuit_send::{
    build_trace as build_send_trace, SendAir, SendProver, SendPublicInputs,
};
use stark_experiment::circuit_claim_v2::{
    build_trace as build_claim_trace_v2, ClaimAirV2, ClaimV2Prover,
};
use stark_experiment::circuit_send_v2::SendV2Air;

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
    /// Era 3 (RFC-0003, E3a / D-1): el sobre de reversion `X = M(f, d(delta))`,
    /// OPACO para el receptor. `None` = pendiente v1 (sin sobre); `Some(X)` =
    /// pendiente v2 -- y la propia opcion es la senal de dispatch del cobro.
    /// El receptor reconstruye `C2 = M(C1, X)` sin aprender `f` ni `delta`
    /// (la identidad esta probada en `pending::v2_compositor`).
    pub x: Option<Digest>,
}

#[derive(Debug)]
/// Materiales del reembolso (§178, R-2c): las DOS pruebas del doble
/// cerrojo — la apertura del compromiso (#27) y la subida de crédito del
/// EMISOR (#28, con su salt derivado de su clave §117).
/// Materiales de la DES-EMISIÓN (§178 §4, R-2d): solo la apertura #27 —
/// no hay cuenta que acreditar; caducar un pendiente de EMISIÓN destruye
/// exactamente lo comprometido y el suministro BAJA lo que subió.
pub struct DeissueReceipt {
    pub refund_proof: Vec<u8>,
    pub position: u64,
    pub amount: u64,
    pub commitment: Digest,
    /// Era 3 (RFC-0003, E3b-2 / D-2): la APERTURA del sobre de reversion,
    /// `(c1, f, delta)` con `c2 = M(c1, M(f, d(delta)))`. `None` = pendiente
    /// v1 (el juez es `refund_ttl`, D-4); `Some` = v2 -- la capa recompone la
    /// apertura contra el compromiso (el compromiso como juez, cero
    /// persistencia) y el plazo es `delta`, saturante: `u64::MAX` = nunca.
    pub apertura: Option<(Digest, Digest, u64)>,
}

pub struct RefundReceipt {
    pub refund_proof: Vec<u8>,
    pub credit_proof: Vec<u8>,
    pub position: u64,
    pub amount: u64,
    /// El compromiso que se abre; la capa comprueba `hoja[pos] == P`.
    pub commitment: Digest,
    /// Era 3 (RFC-0003, E3b-2 / D-2): la APERTURA `(c1, f, delta)`; ver
    /// `DeissueReceipt::apertura`. `None` = v1; `Some` = v2, y ademas
    /// `f` NOMBRA a quien vuelve el dinero: la capa exige que el
    /// `public_id` de la cuenta acreditada sea exactamente `f`.
    pub apertura: Option<(Digest, Digest, u64)>,
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

/// **Lo que una validacion de envio produce y una aplicacion consume**
/// (§214). Separar ambas fases es lo que permite validar N operaciones
/// contra una instantanea y aplicarlas despues (RFC-0002, etapa 2).
struct SendPlan {
    sender_index: AccountIndex,
    updated: ClientState,
    hoja_nueva: Digest,
    pos: u64,
    compromiso: Digest,
    amount: u64,
}

/// **Una operacion de un lote** (§215).
///
/// El lote existe para que N titulares generen sus pruebas **en paralelo
/// contra una misma raiz de arranque**, en vez de esperar cada uno a que
/// el anterior aplique. Es lo que ataca la contencion medida en §204:
/// 3,83 regeneraciones por pago, el 66 % del trabajo criptografico
/// tirado.
pub enum BatchOp<'a> {
    Send {
        receipt: &'a SendReceipt,
        sender_index: AccountIndex,
        sender_state: &'a ClientState,
        amount: u64,
    },
    Claim {
        receipt: &'a ClaimReceipt,
        receiver_index: AccountIndex,
        receiver_state: &'a ClientState,
        notice: &'a PendingNotice,
    },
}

impl BatchOp<'_> {
    /// La cuenta que la operacion toca. **Una por lote.**
    fn account(&self) -> AccountIndex {
        match self {
            BatchOp::Send { sender_index, .. } => *sender_index,
            BatchOp::Claim { receiver_index, .. } => *receiver_index,
        }
    }

    /// La posicion de pendiente que crea (envio) o consume (cobro).
    fn pending_position(&self) -> u64 {
        match self {
            BatchOp::Send { receipt, .. } => receipt.notice.position,
            BatchOp::Claim { notice, .. } => notice.position,
        }
    }
}

/// Un plan validado con su recibo, listo para aplicar.
enum PlanListo<'a> {
    Send(SendPlan, &'a SendReceipt),
    Claim(ClaimPlan, &'a ClaimReceipt),
}

/// Gemela de [`SendPlan`] para el cobro.
struct ClaimPlan {
    receiver_index: AccountIndex,
    updated: ClientState,
    hoja_nueva: Digest,
    position: u64,
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
    /// comprobaba la invariante usaba `transfer()` —retirada en la 32
    /// (§161)—, que abonaba al receptor en el acto. Es el modo de fallo
    /// que este proyecto
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
    /// Metadato de caducidad de una posición: `(emisor, nacimiento)`.
    /// `None` en pendientes anteriores a R-2a (legado): sin meta no hay
    /// caducidad — solo `claim`, para siempre. Ausencia = inmunidad, no
    /// fabricación (§178).
    pub fn pending_meta_of(&self, pos: u64) -> Option<(u64, u64)> {
        self.pending_meta.get(&pos).copied()
    }

    /// §388: el meta y su arbol se mueven JUNTOS, o la raiz `root:pmeta`
    /// dejaria de describir el mapa. Unico escritor del par.
    pub(crate) fn meta_set(&mut self, pos: u64, sender: u64, born: u64) {
        self.pending_meta.insert(pos, (sender, born));
        self.pending_meta_tree
            .set_leaf(pos, zk_ssl_hash::meta_pendiente_hoja(sender, born));
    }

    /// §388: quitar el meta vacia tambien su hoja (digest cero = libre).
    pub(crate) fn meta_clear(&mut self, pos: u64) {
        self.pending_meta.remove(&pos);
        self.pending_meta_tree.set_leaf(pos, [BaseElement::ZERO; 4]);
    }

    /// La `T` vigente de la caducidad (línea sistémica, §178).
    pub fn refund_ttl(&self) -> u64 {
        self.refund_ttl
    }

    /// Ajusta `T`. El knob existe para operación y tests; el valor es
    /// parte del contrato público del nodo y se persiste con el lote.
    pub fn set_refund_ttl(&mut self, t: u64) {
        self.refund_ttl = t;
    }

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
            // §211: una posicion RESERVADA cuenta como ocupada aunque el
            // arbol aun no la tenga. Sin esto, dos peticiones contra la
            // misma raiz de arranque recibirian la misma (§210).
            if !self.pending.is_occupied(p) && !self.reserved_pending.contains(&p) {
                return Ok(p);
            }
        }
        let mut p = self.next_pending;
        while self.reserved_pending.contains(&p) {
            p += 1;
        }
        if p >= self.pending.capacity() {
            return Err(LayerError::PendingTreeExhausted {
                capacity: self.pending.capacity(),
            });
        }
        Ok(p)
    }

    /// **Reserva una posicion de pendiente** (§211, pieza 1 de la etapa 2).
    ///
    /// Devuelve la misma posicion que `allocate_pending` **y la marca**,
    /// de modo que una segunda llamada devuelve otra distinta aunque el
    /// arbol no haya cambiado. Es lo que permite entregar materiales a N
    /// clientes contra una sola raiz de arranque.
    ///
    /// Quien reserve y no aplique debe llamar a [`Self::release_pending`],
    /// o la posicion queda inmovilizada hasta reiniciar. **Las reservas no
    /// se persisten**: un reinicio las anula todas, que es lo correcto —
    /// si nada se aplico, nada hay que respetar.
    pub fn reserve_pending(&mut self) -> Result<u64, LayerError> {
        let p = self.allocate_pending()?;
        self.reserved_pending.insert(p);
        Ok(p)
    }

    /// Libera una reserva que no va a aplicarse. `true` si estaba reservada.
    pub fn release_pending(&mut self, position: u64) -> bool {
        self.reserved_pending.remove(&position)
    }

    /// Cuantas posiciones hay reservadas ahora mismo. Diagnostico: **debe
    /// volver a cero** cuando el lote termina.
    pub fn reserved_pending_count(&self) -> usize {
        self.reserved_pending.len()
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
    /// **Fabrica los materiales del reembolso** (§178, R-2c). Solo el
    /// EMISOR puede: la hoja se reconstruye con el salt DERIVADO de su
    /// clave (§117) y debe casar con el árbol — clave ajena, hoja ajena,
    /// rebote aquí mismo.
    /// Fabrica los materiales de la des-emisión: la apertura del
    /// compromiso, sin clave ni cuenta — quien tenga el aviso (emisor
    /// delegado, custodios, o el receptor que nunca cobró) puede pedirla;
    /// la compuerta del centinela decide si procede.
    pub fn deissue(
        &self,
        position: u64,
        receiver_id: Digest,
        salt: Digest,
        amount: u64,
    ) -> Result<DeissueReceipt, LayerError> {
        use stark_experiment::circuit_refund as refundc;
        let commitment = crate::pending::pending_commitment(receiver_id, salt, amount);
        let t = refundc::build_trace(receiver_id, salt, amount);
        let refund_proof = refundc::RefundProver::new(self.options.clone())
            .prove(t)
            .map_err(|e| LayerError::VerificationFailed(format!("apertura: {e:?}")))?
            .to_bytes();
        Ok(DeissueReceipt { refund_proof, position, amount, commitment, apertura: None })
    }

    /// Era 3 (RFC-0003, E3b-2 / D-2): la variante v2 de la des-emision.
    /// El compromiso es `pending_commitment_v2` y el recibo lleva la
    /// APERTURA; el juez temporal pasa a ser `delta` (via `apply_deissue`).
    pub fn deissue_v2(
        &self,
        position: u64,
        receiver_id: Digest,
        salt: Digest,
        amount: u64,
        refund_id: Digest,
        delta: u64,
    ) -> Result<DeissueReceipt, LayerError> {
        use stark_experiment::circuit_refund_v2 as refund2;
        let c1 = crate::pending::pending_commitment(receiver_id, salt, amount);
        let commitment =
            crate::pending::pending_commitment_v2(receiver_id, salt, amount, refund_id, delta);
        let t = refund2::build_trace(receiver_id, salt, amount, refund_id, delta);
        let refund_proof = refund2::RefundV2Prover::new(self.options.clone())
            .prove(t)
            .map_err(|e| LayerError::VerificationFailed(format!("apertura: {e:?}")))?
            .to_bytes();
        Ok(DeissueReceipt {
            refund_proof,
            position,
            amount,
            commitment,
            apertura: Some((c1, refund_id, delta)),
        })
    }

    /// **Aplica la des-emisión**: SOLO posiciones con centinela (nacidas
    /// por emisión). Las compuertas de tiempo y materiales son las del
    /// reembolso; la mutación es destruir — hoja vacía y suministro ABAJO.
    pub fn apply_deissue(&mut self, receipt: &DeissueReceipt) -> Result<(), LayerError> {
        use stark_experiment::circuit_refund::{RefundAir, RefundPublicInputs};

        let pos = receipt.position;
        let (sender_index, born) = self
            .pending_meta
            .get(&pos)
            .copied()
            .ok_or(LayerError::RefundUnavailable)?;
        if sender_index != crate::REFUND_SENDER_NONE {
            // Un pendiente de PAGO no se des-emite: su vía es el reembolso.
            return Err(LayerError::RefundUnavailable);
        }
        let now = self.log.len() as u64;
        match receipt.apertura {
            // Via v1, INTACTA (D-4): el juez temporal es el knob global.
            None => {
                if now.saturating_sub(born) < self.refund_ttl {
                    return Err(LayerError::RefundTooEarly { born, now, ttl: self.refund_ttl });
                }
            }
            // Via v2 (D-2): la apertura tiene que recomponer el compromiso
            // (el compromiso como juez) y el plazo es el del sobre,
            // saturante -- `u64::MAX` = nunca (el "nadie nunca" del S119).
            Some((c1, f, delta)) => {
                use stark_experiment::merkle::native_merge;
                if native_merge(c1, crate::pending::refund_envelope(f, delta))
                    != receipt.commitment
                {
                    return Err(LayerError::PendingMismatch);
                }
                if now.saturating_sub(born) < delta {
                    return Err(LayerError::RefundTooEarly { born, now, ttl: delta });
                }
            }
        }
        if self.pending.leaf(pos) != receipt.commitment {
            return Err(LayerError::PendingMismatch);
        }
        if self.pending_amounts.get(&pos).copied() != Some(receipt.amount) {
            return Err(LayerError::PendingMismatch);
        }
        let accepted = AcceptableOptions::OptionSet(vec![self.options.clone()]);
        let p_ref = winterfell::Proof::from_bytes(&receipt.refund_proof)
            .map_err(|e| LayerError::VerificationFailed(format!("apertura mal formada: {e:?}")))?;
        // D-7 a los gemelos: el ancho de la traza se compara ANTES de
        // construir el Air (su fn new lo exige con assert_eq!) -- el
        // rechazo es un Err, no un panico alcanzable desde la entrada.
        let ancho = p_ref.trace_info().width();
        match receipt.apertura {
            None => {
                if ancho != stark_experiment::circuit_refund::TRACE_WIDTH {
                    return Err(LayerError::VerificationFailed(format!(
                        "desemision: traza de {ancho} columnas y recibo sin apertura, via v1 (la via exige {})",
                        stark_experiment::circuit_refund::TRACE_WIDTH
                    )));
                }
                verify::<RefundAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
                    p_ref,
                    RefundPublicInputs {
                        commitment: receipt.commitment,
                        amount: BaseElement::new(receipt.amount),
                    },
                    &accepted,
                )
            }
            Some(_) => {
                // Los publics del v2 SON los del v1 (la X jamas se publica,
                // medido en el PASTE-E3a-M): RefundAirV2 reutiliza
                // RefundPublicInputs, ya importado arriba.
                use stark_experiment::circuit_refund_v2::RefundAirV2;
                if ancho != stark_experiment::circuit_refund_v2::TRACE_WIDTH {
                    return Err(LayerError::VerificationFailed(format!(
                        "desemision v2: traza de {ancho} columnas y recibo con apertura, via v2 (la via exige {})",
                        stark_experiment::circuit_refund_v2::TRACE_WIDTH
                    )));
                }
                verify::<RefundAirV2, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
                    p_ref,
                    RefundPublicInputs {
                        commitment: receipt.commitment,
                        amount: BaseElement::new(receipt.amount),
                    },
                    &accepted,
                )
            }
        }
        .map_err(|e| LayerError::VerificationFailed(format!("apertura: {e:?}")))?;

        let vacia: Digest = [BaseElement::ZERO; 4];
        let root = self.accounts.root();
        self.pending.set_leaf(pos, vacia);
        self.pending_amounts.remove(&pos);
        self.meta_clear(pos);
        // Lo emitido-a-pendiente subió el suministro al nacer (two_phase,
        // mint_to_pending): al caducar sin cobro, BAJA exactamente eso.
        self.total_supply -= receipt.amount;
        // La raíz de cuentas no se mueve: el log la repite a ambos lados.
        self.log
            .append(OpKind::Refund, root, root, &receipt.refund_proof);
        self.commit(&[], Some((pos, vacia)))?;
        Ok(())
    }

    pub fn refund(
        &self,
        spend_key: BaseElement,
        sender_index: AccountIndex,
        sender_state: &ClientState,
        position: u64,
        receiver_id: Digest,
        salt: Digest,
        amount: u64,
    ) -> Result<RefundReceipt, LayerError> {
        use stark_experiment::circuit_credit_climb as credit;
        use stark_experiment::circuit_refund as refundc;
        use stark_experiment::native::derive_leaf_salt;

        if derive_public_id(spend_key) != sender_state.public_id {
            return Err(LayerError::NotTheAccountHolder);
        }
        let leaf_salt = derive_leaf_salt(spend_key);
        let hoja = native_leaf_salted(
            sender_state.public_id,
            BaseElement::new(sender_state.balance),
            sender_state.nonce,
            leaf_salt,
        );
        if hoja != self.accounts.leaf(sender_index) {
            return Err(LayerError::StaleState);
        }
        let commitment = crate::pending::pending_commitment(receiver_id, salt, amount);
        let path = self.accounts.path_for(sender_index);

        let t_ref = refundc::build_trace(receiver_id, salt, amount);
        let refund_proof = refundc::RefundProver::new(self.options.clone())
            .prove(t_ref)
            .map_err(|e| LayerError::VerificationFailed(format!("apertura: {e:?}")))?
            .to_bytes();
        let t_cred = credit::build_trace(
            sender_state.public_id,
            sender_state.balance,
            sender_state.nonce,
            leaf_salt,
            &path,
            amount,
        );
        let credit_proof = credit::CreditClimbProver::new(self.options.clone())
            .prove(t_cred)
            .map_err(|e| LayerError::VerificationFailed(format!("credito: {e:?}")))?
            .to_bytes();
        Ok(RefundReceipt { refund_proof, credit_proof, position, amount, commitment, apertura: None })
    }

    /// Era 3 (RFC-0003, E3b-2 / D-2): la variante v2 del reembolso. El
    /// compromiso es `pending_commitment_v2`, la prueba de apertura es la
    /// del circuito v2, y el recibo lleva la APERTURA `(c1, f, delta)`;
    /// la pata del credito NO cambia (mismo `CreditClimbAir`).
    pub fn refund_v2(
        &self,
        spend_key: BaseElement,
        sender_index: AccountIndex,
        sender_state: &ClientState,
        position: u64,
        receiver_id: Digest,
        salt: Digest,
        amount: u64,
        refund_id: Digest,
        delta: u64,
    ) -> Result<RefundReceipt, LayerError> {
        use stark_experiment::circuit_credit_climb as credit;
        use stark_experiment::circuit_refund_v2 as refund2;
        use stark_experiment::native::derive_leaf_salt;

        if derive_public_id(spend_key) != sender_state.public_id {
            return Err(LayerError::NotTheAccountHolder);
        }
        let leaf_salt = derive_leaf_salt(spend_key);
        let hoja = native_leaf_salted(
            sender_state.public_id,
            BaseElement::new(sender_state.balance),
            sender_state.nonce,
            leaf_salt,
        );
        if hoja != self.accounts.leaf(sender_index) {
            return Err(LayerError::StaleState);
        }
        let c1 = crate::pending::pending_commitment(receiver_id, salt, amount);
        let commitment =
            crate::pending::pending_commitment_v2(receiver_id, salt, amount, refund_id, delta);
        let path = self.accounts.path_for(sender_index);

        let t_ref = refund2::build_trace(receiver_id, salt, amount, refund_id, delta);
        let refund_proof = refund2::RefundV2Prover::new(self.options.clone())
            .prove(t_ref)
            .map_err(|e| LayerError::VerificationFailed(format!("apertura: {e:?}")))?
            .to_bytes();
        let t_cred = credit::build_trace(
            sender_state.public_id,
            sender_state.balance,
            sender_state.nonce,
            leaf_salt,
            &path,
            amount,
        );
        let credit_proof = credit::CreditClimbProver::new(self.options.clone())
            .prove(t_cred)
            .map_err(|e| LayerError::VerificationFailed(format!("credito: {e:?}")))?
            .to_bytes();
        Ok(RefundReceipt {
            refund_proof,
            credit_proof,
            position,
            amount,
            commitment,
            apertura: Some((c1, refund_id, delta)),
        })
    }

    /// **Aplica el reembolso** con los DOS cerrojos de §178: el destino lo
    /// fijan los registros (`meta.sender_index`) — pruebe quien pruebe— y
    /// la subida solo casa si la fabricó el titular de ESA hoja.
    pub fn apply_refund(&mut self, receipt: &RefundReceipt) -> Result<(), LayerError> {
        use stark_experiment::circuit_credit_climb::{CreditClimbAir, CreditClimbPublicInputs};
        use stark_experiment::circuit_refund::{RefundAir, RefundPublicInputs};

        let pos = receipt.position;
        let (sender_index, born) = self
            .pending_meta
            .get(&pos)
            .copied()
            .ok_or(LayerError::RefundUnavailable)?;
        if sender_index == crate::REFUND_SENDER_NONE {
            return Err(LayerError::RefundUnavailable);
        }
        let now = self.log.len() as u64;
        match receipt.apertura {
            // Via v1, INTACTA (D-4): el juez temporal es el knob global.
            None => {
                if now.saturating_sub(born) < self.refund_ttl {
                    return Err(LayerError::RefundTooEarly { born, now, ttl: self.refund_ttl });
                }
            }
            // Via v2 (D-2): la apertura tiene que recomponer el compromiso
            // (el compromiso como juez) y el plazo es el del sobre,
            // saturante -- `u64::MAX` = nunca (el "nadie nunca" del S119).
            Some((c1, f, delta)) => {
                use stark_experiment::merkle::native_merge;
                if native_merge(c1, crate::pending::refund_envelope(f, delta))
                    != receipt.commitment
                {
                    return Err(LayerError::PendingMismatch);
                }
                if now.saturating_sub(born) < delta {
                    return Err(LayerError::RefundTooEarly { born, now, ttl: delta });
                }
            }
        }
        if self.pending.leaf(pos) != receipt.commitment {
            return Err(LayerError::PendingMismatch);
        }
        if self.pending_amounts.get(&pos).copied() != Some(receipt.amount) {
            return Err(LayerError::PendingMismatch);
        }
        let rec = self
            .records
            .get(&sender_index)
            .ok_or(LayerError::AccountNotFound(sender_index))?
            .clone();
        if let Some((_, f, _)) = receipt.apertura {
            // credito -> f (D-2): el sobre NOMBRA a quien vuelve el dinero.
            if rec.public_id != f {
                return Err(LayerError::PendingMismatch);
            }
        }
        let root_old = self.accounts.root();
        let updated_balance = rec.balance + receipt.amount;
        let mut tentativo = self.accounts.clone();
        tentativo.set_leaf(
            sender_index,
            native_leaf_salted(
                rec.public_id,
                BaseElement::new(updated_balance),
                rec.nonce,
                rec.leaf_salt,
            ),
        );
        let root_new = tentativo.root();
        let accepted = AcceptableOptions::OptionSet(vec![self.options.clone()]);
        let p_ref = winterfell::Proof::from_bytes(&receipt.refund_proof)
            .map_err(|e| LayerError::VerificationFailed(format!("apertura mal formada: {e:?}")))?;
        // D-7 a los gemelos: el ancho de la traza se compara ANTES de
        // construir el Air (su fn new lo exige con assert_eq!) -- el
        // rechazo es un Err, no un panico alcanzable desde la entrada.
        let ancho = p_ref.trace_info().width();
        match receipt.apertura {
            None => {
                if ancho != stark_experiment::circuit_refund::TRACE_WIDTH {
                    return Err(LayerError::VerificationFailed(format!(
                        "reembolso: traza de {ancho} columnas y recibo sin apertura, via v1 (la via exige {})",
                        stark_experiment::circuit_refund::TRACE_WIDTH
                    )));
                }
                verify::<RefundAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
                    p_ref,
                    RefundPublicInputs {
                        commitment: receipt.commitment,
                        amount: BaseElement::new(receipt.amount),
                    },
                    &accepted,
                )
            }
            Some(_) => {
                // Los publics del v2 SON los del v1 (la X jamas se publica,
                // medido en el PASTE-E3a-M): RefundAirV2 reutiliza
                // RefundPublicInputs, ya importado arriba.
                use stark_experiment::circuit_refund_v2::RefundAirV2;
                if ancho != stark_experiment::circuit_refund_v2::TRACE_WIDTH {
                    return Err(LayerError::VerificationFailed(format!(
                        "reembolso v2: traza de {ancho} columnas y recibo con apertura, via v2 (la via exige {})",
                        stark_experiment::circuit_refund_v2::TRACE_WIDTH
                    )));
                }
                verify::<RefundAirV2, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
                    p_ref,
                    RefundPublicInputs {
                        commitment: receipt.commitment,
                        amount: BaseElement::new(receipt.amount),
                    },
                    &accepted,
                )
            }
        }
        .map_err(|e| LayerError::VerificationFailed(format!("apertura: {e:?}")))?;
        let p_cred = winterfell::Proof::from_bytes(&receipt.credit_proof)
            .map_err(|e| LayerError::VerificationFailed(format!("credito mal formado: {e:?}")))?;
        // La ranura del credito tiene su propio juez de geometria.
        let ancho_credito = p_cred.trace_info().width();
        if ancho_credito != stark_experiment::circuit_credit_climb::TRACE_WIDTH {
            return Err(LayerError::VerificationFailed(format!(
                "credito: traza de {ancho_credito} columnas (la subida de credito exige {})",
                stark_experiment::circuit_credit_climb::TRACE_WIDTH
            )));
        }
        verify::<CreditClimbAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            p_cred,
            CreditClimbPublicInputs {
                root_old,
                root_new,
                amount: BaseElement::new(receipt.amount),
            },
            &accepted,
        )
        .map_err(|e| LayerError::VerificationFailed(format!("credito: {e:?}")))?;

        let vacia: Digest = [BaseElement::ZERO; 4];
        self.accounts = tentativo;
        let mut nuevo = rec;
        nuevo.balance = updated_balance;
        self.records.insert(sender_index, nuevo);
        self.pending.set_leaf(pos, vacia);
        self.pending_amounts.remove(&pos);
        self.meta_clear(pos);
        self.log
            .append(OpKind::Refund, root_old, root_new, &receipt.refund_proof);
        self.commit(&[sender_index], Some((pos, vacia)))?;
        Ok(())
    }

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
        let leaf_salt_rec = self
            .records
            .get(&sender_index)
            .map(|r| r.leaf_salt)
            .unwrap_or(crate::store::LEAF_SALT_LEGACY);
        let hoja = native_leaf_salted(
            sender_state.public_id,
            BaseElement::new(sender_state.balance),
            sender_state.nonce,
            leaf_salt_rec,
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

        // ⚠️ **La clave se RELLENA a cuatro elementos en el borde.**
        //
        // `circuit_send` la absorbe ancha desde §90, y esta via —la de EN
        // MEDIO: de produccion frente a `transfer`, que hoy solo vive en la
        // capa ANTERIOR (`settlement-layer`), y anterior al flujo por
        // MATERIALES que documenta PRINCIPIOS 4.2— sigue manejandola
        // estrecha. Cuadran porque §90 probo que rellenar con ceros da la
        // MISMA identidad.
        //
        // ⚠️ **Nada la marca.** No lleva `#[deprecated]` ni ningun otro
        // atributo, y por eso el compilador no avisa de su uso.
        //
        // ⚠️ Aqui SI se rellena y en `client::prove_send` NO: alli es donde
        // el cliente tiene que poder usar una clave ancha de verdad.
        let trace = build_send_trace(
            [
                spend_key,
                BaseElement::ZERO,
                BaseElement::ZERO,
                BaseElement::ZERO,
            ],
            sender.public_id,
            sender.balance,
            sender.nonce,
            leaf_salt_rec,
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
                x: None,
            },
        })
    }

    /// **Valida un envio SIN tocar nada** (§214).
    ///
    /// Toma los arboles por referencia en vez de leerlos de `self`: eso es
    /// lo que permite validarlo contra una **instantanea de arranque de
    /// lote** en vez de contra el estado actual (RFC-0002, etapa 2).
    ///
    /// Contiene, en el mismo orden, las cinco comprobaciones que
    /// `apply_send` hacia antes de mutar. **Ninguna muta.**
    #[allow(clippy::too_many_arguments)]
    fn validate_send(
        &self,
        accounts: &SparseTree,
        pending: &SparseTree,
        frozen_root: Digest,
        receipt: &SendReceipt,
        sender_index: AccountIndex,
        sender_state: &ClientState,
        amount: u64,
    ) -> Result<SendPlan, LayerError> {
        let pi = &receipt.public_inputs;
        if pi.root_old != accounts.root() || pi.pending_root_old != pending.root() {
            return Err(LayerError::StaleState);
        }
        if pi.frozen_root != frozen_root {
            return Err(LayerError::StaleState);
        }

        // ===== EL LIMITE REGULATORIO =====
        //
        // ⚠️ Se comprueba sobre el importe PROBADO, no sobre el parametro:
        // quien construya su propia traza puede llamar directamente al
        // apply. El circuito prueba `importe <= limite DECLARADO`; la capa
        // comprueba que ese limite sea el suyo. Ver `AUDITORIA.md` §25.
        //
        // ⚠️ Esto es de CAPA, no de circuito: `circuit_send` no lleva el
        // limite como entrada publica, asi que un tercero con solo la
        // prueba **no puede verificar que se respeto**. Cerrarlo exige
        // anadirlo al circuito, y no esta hecho.
        let limite_declarado = pi.regulatory_limit.as_int();
        if limite_declarado != self.regulatory_limit {
            return Err(LayerError::WrongRegulatoryLimit {
                expected: self.regulatory_limit,
                declared: limite_declarado,
            });
        }

        // ===== SE VERIFICA LA PRUEBA =====
        //
        // ⚠️ Esto faltaba, y era el fallo mas grave de la auditoria (§73).
        // Sin esta llamada, gastar no requeria la clave del titular: un
        // tercero debitaba una cuenta ajena y se dirigia el pendiente a si
        // mismo. Va ANTES de tocar el estado.
        let proof = winterfell::Proof::from_bytes(&receipt.proof)
            .map_err(|e| LayerError::VerificationFailed(format!("prueba mal formada: {e:?}")))?;
        let min_opts = AcceptableOptions::OptionSet(vec![self.options.clone()]);
        // E3c-1b (RFC-0003): el aviso del recibo decide el Air, como en
        // el cobro. Un recibo v1 con X colada, o un v2 sin ella, cae
        // aqui: el Air no corresponde a la prueba.
        // D-7: el ancho de la traza se compara ANTES de construir el Air.
        // El constructor de ambos Airs exige su ancho con assert_eq! (fn
        // new de circuit_send y circuit_send_v2): sin esta guarda, un
        // recibo con la X mal puesta ABORTARIA el hilo en vez de
        // rechazarse. El rechazo es un Err, no un panico alcanzable
        // desde la entrada (medido en la sesion 63).
        let ancho = proof.trace_info().width();
        match receipt.notice.x {
            None => {
                if ancho != stark_experiment::circuit_send::TRACE_WIDTH {
                    return Err(LayerError::VerificationFailed(format!(
                        "envio: traza de {ancho} columnas y aviso sin sobre (la via v1 exige {})",
                        stark_experiment::circuit_send::TRACE_WIDTH
                    )));
                }
                verify::<SendAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
                    proof,
                    pi.clone(),
                    &min_opts,
                )
                .map_err(|e| LayerError::VerificationFailed(format!("envio: {e:?}")))?
            }
            Some(_) => {
                if ancho != stark_experiment::circuit_send_v2::TRACE_WIDTH {
                    return Err(LayerError::VerificationFailed(format!(
                        "envio v2: traza de {ancho} columnas y aviso con sobre (la via v2 exige {})",
                        stark_experiment::circuit_send_v2::TRACE_WIDTH
                    )));
                }
                verify::<SendV2Air, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
                    proof,
                    pi.clone(),
                    &min_opts,
                )
                .map_err(|e| LayerError::VerificationFailed(format!("envio v2: {e:?}")))?
            }
        }

        // ⚠️ **El nonce NO se incrementa.** `circuit_send` lo heredo de
        // `circuit_burn`. Si la capa lo hiciera, la hoja resultante seria
        // otra y la raiz no cuadraria con la que la prueba acredita. La
        // proteccion contra reenvio viene del encadenamiento de raices.
        let updated = ClientState {
            balance: sender_state.balance - amount,
            ..sender_state.clone()
        };
        let leaf_salt_rec = self
            .records
            .get(&sender_index)
            .map(|r| r.leaf_salt)
            .unwrap_or(crate::store::LEAF_SALT_LEGACY);
        let hoja_nueva = native_leaf_salted(
            updated.public_id,
            BaseElement::new(updated.balance),
            updated.nonce,
            leaf_salt_rec,
        );
        let pos = receipt.notice.position;
        let compromiso = receipt.commitment;

        // La raiz hipotetica sale de `root_with` (§212): sin clonar, y sin
        // mutar. Se calcula **contra los arboles que se han pasado**, que
        // en un lote son los de la instantanea de arranque.
        if accounts.root_with(sender_index, hoja_nueva) != pi.root_new
            || pending.root_with(pos, compromiso) != pi.pending_root_new
        {
            return Err(LayerError::StaleState);
        }

        Ok(SendPlan {
            sender_index,
            updated,
            hoja_nueva,
            pos,
            compromiso,
            amount,
        })
    }

    /// **Aplica un envio ya validado.** Solo muta; no comprueba nada.
    ///
    /// ⚠️ El registro recibe las raices **REALES** —la de antes y la de
    /// despues de esta mutacion—, no las que la prueba declara. Con una
    /// sola operacion coinciden; en un lote no, y el registro necesita las
    /// reales para que `verify_chain` siga pasando. La decision y lo que
    /// se pierde con ella estan en `spec/RPC.md` y en §213.
    fn commit_send(&mut self, plan: &SendPlan, receipt: &SendReceipt) -> Result<(), LayerError> {
        let root_antes = self.accounts.root();
        self.accounts.set_leaf(plan.sender_index, plan.hoja_nueva);
        self.pending.set_leaf(plan.pos, plan.compromiso);
        let root_despues = self.accounts.root();

        // El registro se mantiene para los accessors del operador (§129,
        // §161): `balance_of` lee `records`, y el snapshot tambien.
        self.records.insert(
            plan.sender_index,
            AccountRecord {
                public_id: plan.updated.public_id,
                balance: plan.updated.balance,
                nonce: plan.updated.nonce,
                // view_id y salt del record GUARDADO, no del ClientState
                // entrante: el cliente no debe reescribir su credencial de
                // lectura al operar (49-A).
                view_id: self
                    .records
                    .get(&plan.sender_index)
                    .map(|r| r.view_id)
                    .unwrap_or(crate::store::VIEW_ID_LEGACY),
                leaf_salt: self
                    .records
                    .get(&plan.sender_index)
                    .map(|r| r.leaf_salt)
                    .unwrap_or(crate::store::LEAF_SALT_LEGACY),
            },
        );
        if plan.pos >= self.next_pending {
            self.next_pending = plan.pos + 1;
        }
        self.reserved_pending.remove(&plan.pos);
        self.pending_amounts.insert(plan.pos, plan.amount);
        let nacido = self.log.len() as u64;
        self.meta_set(plan.pos, plan.sender_index, nacido);
        // ⚠️ Deja constancia ANTES de persistir: el lote atomico incluye o
        // excluye las dos cosas.
        self.log
            .append(OpKind::Send, root_antes, root_despues, &receipt.proof);
        self.commit(&[plan.sender_index], Some((plan.pos, plan.compromiso)))?;
        Ok(())
    }

    /// **FASE 1 aplicada.** Validar contra el estado actual y aplicar.
    pub fn apply_send(
        &mut self,
        receipt: &SendReceipt,
        sender_index: AccountIndex,
        sender_state: &ClientState,
        amount: u64,
    ) -> Result<(), LayerError> {
        let plan = self.validate_send(
            &self.accounts,
            &self.pending,
            self.frozen.root(),
            receipt,
            sender_index,
            sender_state,
            amount,
        )?;
        self.commit_send(&plan, receipt)
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
        let leaf_salt_rec = self
            .records
            .get(&receiver_index)
            .map(|r| r.leaf_salt)
            .unwrap_or(crate::store::LEAF_SALT_LEGACY);
        let hoja = native_leaf_salted(
            receiver_state.public_id,
            BaseElement::new(receiver_state.balance),
            receiver_state.nonce,
            leaf_salt_rec,
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

        // ⚠️ Clave RELLENADA a cuatro elementos: via antigua, y §90
        // garantiza la misma identidad (§92.9).
        // E3b-1 (RFC-0003): el aviso decide la via. None = hoja v1;
        // Some(sobre) = hoja v2, la traza gana el cuarto merge y el
        // verificador sera ClaimAirV2. Los publics son LOS MISMOS.
        let clave = [
            spend_key,
            BaseElement::ZERO,
            BaseElement::ZERO,
            BaseElement::ZERO,
        ];
        let (public_inputs, proof) = match notice.x {
            None => {
                let trace = build_claim_trace(
                    clave,
                    receiver.public_id,
                    receiver.balance,
                    receiver.nonce,
                    leaf_salt_rec,
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
                (public_inputs, proof)
            }
            Some(sobre) => {
                let trace = build_claim_trace_v2(
                    clave,
                    receiver.public_id,
                    receiver.balance,
                    receiver.nonce,
                    leaf_salt_rec,
                    &path,
                    &frozen_path,
                    notice.amount,
                    self.total_supply,
                    0,
                    receiver.public_id,
                    notice.salt,
                    sobre,
                    &pending_path,
                );
                let prover = ClaimV2Prover::new(self.options.clone());
                let public_inputs = prover.get_pub_inputs(&trace);
                let proof = prover
                    .prove(trace)
                    .map_err(|e| LayerError::ProofFailed(format!("{e:?}")))?;
                (public_inputs, proof)
            }
        };

        Ok(ClaimReceipt {
            proof: proof.to_bytes(),
            public_inputs,
        })
    }

    /// Aplica una reclamación: acredita y consume el pendiente.
    /// **Valida un cobro SIN tocar nada** (§214). Gemela de
    /// [`Self::validate_send`]: arboles por referencia, ninguna mutacion.
    fn validate_claim(
        &self,
        accounts: &SparseTree,
        pending: &SparseTree,
        frozen_root: Digest,
        receipt: &ClaimReceipt,
        receiver_index: AccountIndex,
        receiver_state: &ClientState,
        notice: &PendingNotice,
    ) -> Result<ClaimPlan, LayerError> {
        let pi = &receipt.public_inputs;
        if pi.root_old != accounts.root() || pi.pending_root_old != pending.root() {
            return Err(LayerError::StaleState);
        }
        if pi.frozen_root != frozen_root {
            return Err(LayerError::StaleState);
        }

        // Se verifica la prueba ANTES de tocar el estado (§73). E3b-1:
        // el aviso decide el verificador - None = ClaimAir, Some = ClaimAirV2.
        let proof = winterfell::Proof::from_bytes(&receipt.proof)
            .map_err(|e| LayerError::VerificationFailed(format!("prueba mal formada: {e:?}")))?;
        let min_opts = AcceptableOptions::OptionSet(vec![self.options.clone()]);
        // D-7 a los gemelos: el ancho de la traza se compara ANTES de
        // construir el Air (su fn new lo exige con assert_eq!) -- el
        // rechazo es un Err, no un panico alcanzable desde la entrada.
        let ancho = proof.trace_info().width();
        match notice.x {
            None => {
                if ancho != stark_experiment::circuit_claim::TRACE_WIDTH {
                    return Err(LayerError::VerificationFailed(format!(
                        "cobro: traza de {ancho} columnas y aviso sin sobre, via v1 (la via exige {})",
                        stark_experiment::circuit_claim::TRACE_WIDTH
                    )));
                }
                verify::<ClaimAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
                    proof,
                    pi.clone(),
                    &min_opts,
                )
            }
            Some(_) => {
                if ancho != stark_experiment::circuit_claim_v2::TRACE_WIDTH {
                    return Err(LayerError::VerificationFailed(format!(
                        "cobro v2: traza de {ancho} columnas y aviso con sobre, via v2 (la via exige {})",
                        stark_experiment::circuit_claim_v2::TRACE_WIDTH
                    )));
                }
                verify::<ClaimAirV2, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
                    proof,
                    pi.clone(),
                    &min_opts,
                )
            }
        }
        .map_err(|e| LayerError::VerificationFailed(format!("cobro: {e:?}")))?;

        let updated = ClientState {
            balance: receiver_state.balance + notice.amount,
            ..receiver_state.clone()
        };
        let leaf_salt_rec = self
            .records
            .get(&receiver_index)
            .map(|r| r.leaf_salt)
            .unwrap_or(crate::store::LEAF_SALT_LEGACY);
        let hoja_nueva = native_leaf_salted(
            updated.public_id,
            BaseElement::new(updated.balance),
            updated.nonce,
            leaf_salt_rec,
        );
        // **Consumido**: la hoja vuelve a estar vacia. Sin esto, el mismo
        // pendiente se cobraria indefinidamente.
        let vacia: Digest = [BaseElement::ZERO; 4];

        if accounts.root_with(receiver_index, hoja_nueva) != pi.root_new
            || pending.root_with(notice.position, vacia) != pi.pending_root_new
        {
            return Err(LayerError::StaleState);
        }

        Ok(ClaimPlan {
            receiver_index,
            updated,
            hoja_nueva,
            position: notice.position,
        })
    }

    /// **Aplica un cobro ya validado.** Solo muta.
    ///
    /// ⚠️ Como en `commit_send`, el registro recibe las raices REALES
    /// (§213). Y las bajas de `pending_amounts`/`pending_meta` van aqui,
    /// **despues** de toda comprobacion: hasta §212 estaban antes, y una
    /// comprobacion fallida dejaba a la capa sin el importe de una nota
    /// que seguia existiendo en el arbol.
    fn commit_claim(&mut self, plan: &ClaimPlan, receipt: &ClaimReceipt) -> Result<(), LayerError> {
        let vacia: Digest = [BaseElement::ZERO; 4];
        let root_antes = self.accounts.root();
        self.pending_amounts.remove(&plan.position);
        self.meta_clear(plan.position);
        self.accounts.set_leaf(plan.receiver_index, plan.hoja_nueva);
        self.pending.set_leaf(plan.position, vacia);
        let root_despues = self.accounts.root();

        self.records.insert(
            plan.receiver_index,
            AccountRecord {
                public_id: plan.updated.public_id,
                balance: plan.updated.balance,
                nonce: plan.updated.nonce,
                view_id: self
                    .records
                    .get(&plan.receiver_index)
                    .map(|r| r.view_id)
                    .unwrap_or(crate::store::VIEW_ID_LEGACY),
                leaf_salt: self
                    .records
                    .get(&plan.receiver_index)
                    .map(|r| r.leaf_salt)
                    .unwrap_or(crate::store::LEAF_SALT_LEGACY),
            },
        );
        self.log
            .append(OpKind::Claim, root_antes, root_despues, &receipt.proof);
        self.commit(&[plan.receiver_index], Some((plan.position, vacia)))?;
        Ok(())
    }

    /// **FASE 2 aplicada.** Validar contra el estado actual y aplicar.
    pub fn apply_claim(
        &mut self,
        receipt: &ClaimReceipt,
        receiver_index: AccountIndex,
        receiver_state: &ClientState,
        notice: &PendingNotice,
    ) -> Result<(), LayerError> {
        let plan = self.validate_claim(
            &self.accounts,
            &self.pending,
            self.frozen.root(),
            receipt,
            receiver_index,
            receiver_state,
            notice,
        )?;
        self.commit_claim(&plan, receipt)
    }

    /// **Aplica N operaciones validadas contra una MISMA raiz de arranque**
    /// (§215, pieza 2 de la etapa 2 del RFC-0002).
    ///
    /// ## Por que existe
    ///
    /// Hoy cada titular genera su prueba contra la raiz que ve, y si otro
    /// aplica mientras tanto la prueba llega muerta. Medido en §204 con
    /// cuatro hilos: **3,83 regeneraciones por pago**, 70 generaciones
    /// para 24 operaciones —**el 66 % del trabajo criptografico tirado**—
    /// y el rendimiento BAJANDO al paralelizar. Un livelock.
    ///
    /// Con lote, N titulares generan **a la vez** contra la misma raiz y
    /// ninguna prueba muere.
    ///
    /// ## Que hace, en orden
    ///
    /// 1. **Rechaza el lote entero** si lleva dos operaciones de la misma
    ///    cuenta o sobre la misma posicion de pendiente.
    /// 2. Toma una **instantanea de arranque** de los tres arboles.
    /// 3. **Valida las N contra esa instantanea** —incluida la
    ///    verificacion de cada prueba— sin mutar nada. Si una falla, no se
    ///    aplica ninguna.
    /// 4. Aplica las N en orden.
    ///
    /// ## Lo que hay que saber antes de usarlo
    ///
    /// ⚠️ **Quien arma el lote debe reservar las posiciones** con
    /// [`Self::reserve_pending`] y pedir materiales con
    /// `send_materials_at` (§211, §214). Sin reservar, dos titulares
    /// reciben la misma posicion.
    ///
    /// ⚠️ **Una congelacion de gobernanza a mitad de lote lo invalida
    /// entero**: cambia `frozen_root` y todas las pruebas dejan de casar.
    /// Las congelaciones van en su propio lote.
    ///
    /// ⚠️ **El registro anota las raices REALES**, no las que cada prueba
    /// declara: en un lote no coinciden. Lo que eso implica —y lo que se
    /// pierde— esta declarado en `spec/RPC.md`, «Que afirma el registro de
    /// transiciones», y en §213.
    ///
    /// ⚠️ **La validacion es todo-o-nada; la aplicacion es secuencial.**
    /// Si fallara la persistencia a mitad, quedaria un lote parcial — la
    /// misma situacion que N llamadas sueltas fallando a la tercera.
    /// Agrupar la persistencia es trabajo posterior; §204 midio que es el
    /// 3 % del coste.
    pub fn apply_many(&mut self, ops: &[BatchOp<'_>]) -> Result<(), LayerError> {
        if ops.is_empty() {
            return Ok(());
        }

        // 1 · una operacion por cuenta, y posiciones distintas.
        let mut cuentas = std::collections::BTreeSet::new();
        let mut posiciones = std::collections::BTreeSet::new();
        for op in ops {
            let idx = op.account();
            if !cuentas.insert(idx) {
                return Err(LayerError::DuplicateAccountInBatch { index: idx });
            }
            let pos = op.pending_position();
            if !posiciones.insert(pos) {
                return Err(LayerError::DuplicatePendingInBatch { position: pos });
            }
        }

        // 2 · la instantanea de arranque. UN clon por lote, no por
        //     operacion: `root_with` (§212) hace el resto sin copiar.
        let snap_accounts = self.accounts.clone();
        let snap_pending = self.pending.clone();
        let snap_frozen = self.frozen.root();

        // 3 · validar TODAS. Nada se muta aqui.
        let mut planes: Vec<PlanListo<'_>> = Vec::with_capacity(ops.len());
        for op in ops {
            let plan = match op {
                BatchOp::Send {
                    receipt,
                    sender_index,
                    sender_state,
                    amount,
                } => PlanListo::Send(
                    self.validate_send(
                        &snap_accounts,
                        &snap_pending,
                        snap_frozen,
                        receipt,
                        *sender_index,
                        sender_state,
                        *amount,
                    )?,
                    receipt,
                ),
                BatchOp::Claim {
                    receipt,
                    receiver_index,
                    receiver_state,
                    notice,
                } => PlanListo::Claim(
                    self.validate_claim(
                        &snap_accounts,
                        &snap_pending,
                        snap_frozen,
                        receipt,
                        *receiver_index,
                        receiver_state,
                        notice,
                    )?,
                    receipt,
                ),
            };
            planes.push(plan);
        }

        // 4 · aplicar. A partir de aqui se muta.
        for plan in &planes {
            match plan {
                PlanListo::Send(p, r) => self.commit_send(p, r)?,
                PlanListo::Claim(p, r) => self.commit_claim(p, r)?,
            }
        }
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
        // D-7 a los gemelos: el ancho de la traza se compara ANTES de
        // construir el Air (su fn new lo exige con assert_eq!) -- el
        // rechazo es un Err, no un panico alcanzable desde la entrada.
        let ancho = climb_proof.trace_info().width();
        if ancho != stark_experiment::circuit_mint_pending_climb::TRACE_WIDTH {
            return Err(LayerError::VerificationFailed(format!(
                "subida del pendiente: traza de {ancho} columnas (la emision delegada exige {})",
                stark_experiment::circuit_mint_pending_climb::TRACE_WIDTH
            )));
        }

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
        // §211: aplicada deja de estar reservada.
        self.reserved_pending.remove(&position);
        self.pending_amounts.insert(position, amount);
        let nacido = self.log.len() as u64;
        self.meta_set(position, crate::REFUND_SENDER_NONE, nacido);

        // La MISMA raiz de cuentas en los dos lados, y es correcto: una
        // emision a un pendiente no toca ninguna cuenta. Mismo criterio que
        // la via antigua de arriba.
        self.log.append_con_compromiso(
            OpKind::MintToPending,
            self.accounts.root(),
            self.accounts.root(),
            &crate::log::sello_de_autorizacion(&operation),
         operation,
            );
        self.commit(&[], Some((position, commitment)))?;

        Ok(PendingNotice {
            position,
            salt,
            amount,
            x: None,
        })
    }
}

/// **§211 — la reserva de posiciones de pendiente.**
///
/// Es la pieza 1 de la etapa 2 del RFC-0002, y la que desbloquea las
/// otras dos: sin ella, un lote reparte la misma posicion dos veces
/// (§210).
/// **§215 — el lote.** Lo que estos tests demuestran es que dos titulares
/// pueden generar contra la MISMA raiz y que las dos pruebas viven.
#[cfg(test)]
mod tests_lote {
    use super::*;
    use crate::tests_support::*;

    fn capa_con_cuentas(n: u64) -> (SovereignLayer, Vec<AccountIndex>) {
        let mut c = SovereignLayer::new(
            custodian_root(),
            governance_root(),
            LIMIT,
            MAX_SUPPLY,
            MAX_ACCOUNTS,
        );
        let mut idx = Vec::new();
        for i in 0..n {
            let a = c.open_account_wide(wide_key(0xA11CE + i));
            idx.push(a);
        }
        for &a in &idx {
            let op = mint_commitment(&c, a, 1_000_000);
            let subida = mint_climb_proof(&c, a, 1_000_000);
            let (pa, ia, pb, ib) = delegated_pair(op, 1, 3);
            c.apply_mint_delegated(subida, pa, ia, pb, ib, a, 1_000_000)
                .expect("fondear");
        }
        (c, idx)
    }

    fn estado(c: &SovereignLayer, i: AccountIndex) -> ClientState {
        ClientState {
            public_id: c.public_id_of(i).expect("cuenta"),
            balance: c.balance_of(i).expect("cuenta"),
            nonce: c.nonce_of(i).expect("cuenta"),
        }
    }

    /// **EL TEST QUE JUSTIFICA LA ETAPA 2.**
    ///
    /// Dos titulares piden materiales contra el MISMO estado —cada uno
    /// con su posicion reservada (§211)— generan sus pruebas, y las dos
    /// se aplican en un lote. Hoy, sin lote, la segunda llegaria muerta.
    #[test]
    fn dos_envios_generados_contra_la_misma_raiz_viven_los_dos() {
        let (mut c, idx) = capa_con_cuentas(4);
        let (a1, b1, a2, b2) = (idx[0], idx[1], idx[2], idx[3]);
        let k1 = wide_key(0xA11CE);
        let k2 = wide_key(0xA11CE + 2);

        // Estado de arranque: nadie ha aplicado nada todavia.
        let est1 = estado(&c, a1);
        let est2 = estado(&c, a2);
        let raiz_arranque = c.accounts.root();
        let entradas_antes = c.log.len();

        // Dos posiciones RESERVADAS: sin esto ambas serian la misma.
        let p1 = c.reserve_pending().expect("reserva 1");
        let p2 = c.reserve_pending().expect("reserva 2");
        assert_ne!(p1, p2, "sin reserva distinta no hay lote posible");

        let m1 = c
            .send_materials_at(a1, c.public_id_of(b1).unwrap(), 1_000, salt_de(11), p1)
            .expect("materiales 1");
        let m2 = c
            .send_materials_at(a2, c.public_id_of(b2).unwrap(), 2_000, salt_de(22), p2)
            .expect("materiales 2");

        // AMBAS pruebas se generan contra la misma raiz de arranque.
        let e1 = crate::client::prove_send(&m1, k1, crate::proof_options()).expect("prueba 1");
        let e2 = crate::client::prove_send(&m2, k2, crate::proof_options()).expect("prueba 2");
        assert_eq!(e1.public_inputs.root_old, raiz_arranque);
        assert_eq!(e2.public_inputs.root_old, raiz_arranque);

        c.apply_many(&[
            BatchOp::Send {
                receipt: &e1,
                sender_index: a1,
                sender_state: &est1,
                amount: 1_000,
            },
            BatchOp::Send {
                receipt: &e2,
                sender_index: a2,
                sender_state: &est2,
                amount: 2_000,
            },
        ])
        .expect("el lote debe aplicarse entero");

        // Los dos saldos bajaron, y el registro encadena.
        assert_eq!(c.balance_of(a1).unwrap(), 999_000);
        assert_eq!(c.balance_of(a2).unwrap(), 998_000);
        // No se fija un numero absoluto: el montaje (aperturas y emisiones)
        // tambien deja entradas, y cuantas es cosa suya. Lo que este test
        // afirma es que el LOTE anade exactamente DOS.
        assert_eq!(
            c.log.len(),
            entradas_antes + 2,
            "el lote debe anadir dos entradas, una por operacion"
        );
        c.log.verify_chain().expect("la cadena debe encadenar");
        assert_eq!(c.reserved_pending_count(), 0, "las reservas se consumieron");
    }

    /// El lote rechaza dos operaciones de la misma cuenta, **antes** de
    /// validar nada.
    #[test]
    fn el_lote_rechaza_una_cuenta_repetida() {
        let (mut c, idx) = capa_con_cuentas(2);
        let (a, b) = (idx[0], idx[1]);
        let est = estado(&c, a);
        let p = c.reserve_pending().expect("reserva");
        let m = c
            .send_materials_at(a, c.public_id_of(b).unwrap(), 1_000, salt_de(7), p)
            .expect("materiales");
        let e = crate::client::prove_send(&m, wide_key(0xA11CE), crate::proof_options())
            .expect("prueba");

        let r = c.apply_many(&[
            BatchOp::Send {
                receipt: &e,
                sender_index: a,
                sender_state: &est,
                amount: 1_000,
            },
            BatchOp::Send {
                receipt: &e,
                sender_index: a,
                sender_state: &est,
                amount: 1_000,
            },
        ]);
        assert!(
            matches!(r, Err(LayerError::DuplicateAccountInBatch { index }) if index == a),
            "esperaba cuenta repetida, salio {r:?}"
        );
        // Y NADA se aplico.
        assert_eq!(c.balance_of(a).unwrap(), 1_000_000);
    }

    /// Si una validacion falla, **no se aplica ninguna**.
    #[test]
    fn una_validacion_fallida_deja_el_lote_entero_sin_aplicar() {
        let (mut c, idx) = capa_con_cuentas(4);
        let (a1, b1, a2) = (idx[0], idx[1], idx[2]);
        let est1 = estado(&c, a1);
        // Estado MENTIROSO para la segunda: su prueba no casara.
        let est2_falso = ClientState {
            balance: 999_999,
            ..estado(&c, a2)
        };
        let p1 = c.reserve_pending().expect("r1");
        let p2 = c.reserve_pending().expect("r2");
        let m1 = c
            .send_materials_at(a1, c.public_id_of(b1).unwrap(), 1_000, salt_de(31), p1)
            .expect("m1");
        let e1 = crate::client::prove_send(&m1, wide_key(0xA11CE), crate::proof_options())
            .expect("p1");
        let m2 = c
            .send_materials_at(a2, c.public_id_of(b1).unwrap(), 1_000, salt_de(32), p2)
            .expect("m2");
        let e2 = crate::client::prove_send(&m2, wide_key(0xA11CE + 2), crate::proof_options())
            .expect("p2");

        let saldo_antes = c.balance_of(a1).unwrap();
        let raiz_antes = c.accounts.root();
        let r = c.apply_many(&[
            BatchOp::Send {
                receipt: &e1,
                sender_index: a1,
                sender_state: &est1,
                amount: 1_000,
            },
            BatchOp::Send {
                receipt: &e2,
                sender_index: a2,
                sender_state: &est2_falso,
                amount: 1_000,
            },
        ]);
        assert!(r.is_err(), "la segunda deberia fallar la validacion");
        assert_eq!(c.balance_of(a1).unwrap(), saldo_antes, "la PRIMERA no debe haberse aplicado");
        assert_eq!(c.accounts.root(), raiz_antes, "el arbol debe estar intacto");
    }

    /// Un lote de UNA operacion deja el mismo estado que `apply_send`.
    #[test]
    fn un_lote_de_uno_equivale_a_apply_send() {
        let hacer = |por_lote: bool| -> (Digest, Digest, usize) {
            let (mut c, idx) = capa_con_cuentas(2);
            let (a, b) = (idx[0], idx[1]);
            let est = estado(&c, a);
            let m = c
                .send_materials(a, c.public_id_of(b).unwrap(), 5_000, salt_de(99))
                .expect("materiales");
            let e = crate::client::prove_send(&m, wide_key(0xA11CE), crate::proof_options())
                .expect("prueba");
            if por_lote {
                c.apply_many(&[BatchOp::Send {
                    receipt: &e,
                    sender_index: a,
                    sender_state: &est,
                    amount: 5_000,
                }])
                .expect("lote de uno");
            } else {
                c.apply_send(&e, a, &est, 5_000).expect("apply_send");
            }
            (c.accounts.root(), c.log.head(), c.log.len())
        };
        assert_eq!(hacer(true), hacer(false), "lote de uno != apply_send");
    }
}

#[cfg(test)]
mod tests_reserva {
    use super::*;
    use crate::tests_support::*;

    fn capa() -> SovereignLayer {
        SovereignLayer::new(
            custodian_root(),
            governance_root(),
            LIMIT,
            MAX_SUPPLY,
            MAX_ACCOUNTS,
        )
    }

    /// **EL TEST QUE JUSTIFICA TODO**: sin reservar, dos peticiones contra
    /// el mismo estado dan la MISMA posicion. Reservando, dan distintas.
    #[test]
    fn dos_reservas_no_colisionan() {
        let mut c = capa();

        // Sin reservar: el comportamiento viejo, que es el bug de §210.
        let a = c.allocate_pending().expect("primera");
        let b = c.allocate_pending().expect("segunda");
        assert_eq!(a, b, "sin reservar, allocate reparte la misma dos veces");

        // Reservando: distintas.
        let a = c.reserve_pending().expect("primera");
        let b = c.reserve_pending().expect("segunda");
        let d = c.reserve_pending().expect("tercera");
        assert_ne!(a, b);
        assert_ne!(b, d);
        assert_ne!(a, d);
        assert_eq!(c.reserved_pending_count(), 3);
    }

    /// Liberar devuelve la posicion al fondo comun.
    #[test]
    fn liberar_devuelve_la_posicion() {
        let mut c = capa();
        let p = c.reserve_pending().expect("reserva");
        assert_eq!(c.reserved_pending_count(), 1);

        assert!(c.release_pending(p), "estaba reservada");
        assert_eq!(c.reserved_pending_count(), 0);
        assert!(!c.release_pending(p), "ya no lo estaba");

        // Y vuelve a entregarse.
        assert_eq!(c.reserve_pending().expect("otra vez"), p);
    }

    /// Las reservas **no se persisten**: una capa nueva no hereda ninguna.
    /// Es lo correcto — si nada se aplico, nada hay que respetar.
    #[test]
    fn una_capa_nueva_no_hereda_reservas() {
        let mut c = capa();
        for _ in 0..5 {
            c.reserve_pending().expect("reserva");
        }
        assert_eq!(c.reserved_pending_count(), 5);

        let limpia = capa();
        assert_eq!(limpia.reserved_pending_count(), 0);
        assert_eq!(limpia.allocate_pending().expect("libre"), 0);
    }

    /// El agotamiento del arbol sigue detectandose con reservas de por
    /// medio: no se puede reservar mas alla de la capacidad.
    #[test]
    fn las_reservas_respetan_la_capacidad() {
        let mut c = capa();
        // No se agota un arbol de 2^32 en un test; se comprueba que la
        // cuenta de reservas avanza y que ninguna se repite.
        let mut vistas = std::collections::BTreeSet::new();
        for _ in 0..64 {
            let p = c.reserve_pending().expect("reserva");
            assert!(vistas.insert(p), "posicion {p} repetida");
        }
        assert_eq!(c.reserved_pending_count(), 64);
        assert_eq!(vistas.len(), 64);
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
        stark_experiment::native::derive_public_id(BaseElement::new(SK_BOB))
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

    /// GEMELOS D-7 (5/5): una prueba de AUTORIZACION en la ranura de la
    /// SUBIDA rebota con Err antes del Air -- sin la guarda, el assert_eq!
    /// del ancho en fn new del MintPendingClimbAir abortaria el hilo.
    #[test]
    fn una_subida_de_ancho_ajeno_no_valida() {
        assert_ne!(
            stark_experiment::circuit_threshold::TRACE_WIDTH,
            stark_experiment::circuit_mint_pending_climb::TRACE_WIDTH,
            "premisa del testigo: anchos distintos"
        );
        let mut layer = new_layer();
        let ck = custodian_keys();
        let (_, cp) = build_custodian_set(&ck);
        let antes = layer.total_supply();
        let op = compromiso(&layer, AMOUNT);
        let (mala, _) = autorizar(ck[1], &cp[1], op);
        let (pa, ia) = autorizar(ck[1], &cp[1], op);
        let (pb, ib) = autorizar(ck[3], &cp[3], op);
        let r = layer.apply_mint_pending_delegated(
            mala, pa, ia, pb, ib, receptor(), salt_de(0x5EED), AMOUNT,
        );
        assert!(r.is_err(), "la subida de ancho ajeno rebota con Err");
        assert_eq!(layer.total_supply(), antes, "nada emitido");
        assert_eq!(layer.total_pending(), 0, "nada en transito");
    }
}

/// **¿VERIFICA LA CAPA LAS PRUEBAS DE LA VIA DE DOS FASES?**
///
/// En este fichero `verify::<...>` vive hoy en CINCO funciones -- diez
/// sitios: `apply_deissue`, `apply_refund` (la apertura y el credito),
/// `validate_send`, `validate_claim` y `apply_mint_pending_delegated`,
/// cada uno con la guarda de ancho D-7 delante (§355). Los demas
/// modulos de la capa -`burn`, `freeze`, `governance`, `audit`, `mint`,
/// `recovery`- si verifican, y `log::verify` declara que **no** valida
/// pruebas.
///
/// Eso es lectura. Estos testigos lo miden, porque la via de dos fases es
/// **la via de pago del sistema** desde que §36 retiro la de un paso.
///
/// ⚠️ **Correr en release.** El tercero usa la posicion 0, que degenera el
/// grado en depuracion (entrada 6, §71.3): `prove` panicaria por una razon
/// distinta de la que se pregunta, y el testigo pasaria por el motivo
/// equivocado.
#[cfg(test)]
mod tests_verificacion {
    use super::*;
    use crate::tests_support::*;
    use stark_experiment::native::derive_public_id;

    const IMPORTE: u64 = 250_000;
    const FONDO: u64 = 1_000_000;
    /// Mallory. **No conoce ninguna clave ajena.**
    const SK_MALLORY: u64 = 0xBADCAFE;

    /// **TESTIGO 5: ¿hace falta la clave del receptor para COBRAR?**
    ///
    /// `apply_claim` acredita `receiver_state.balance + notice.amount` con
    /// **las dos cosas puestas por quien llama**, borra el pendiente de
    /// `notice.position` **sin comprobar que el compromiso guardado
    /// corresponda al aviso**, y no verificaba la prueba.
    ///
    /// Si eso es asi, cobrar el pendiente de otro solo exige saber su
    /// posicion —que es publica: esta en el arbol—.
    ///
    /// ## El escenario
    ///
    /// Alice paga a Bob por la via honesta. Mallory, que **no conoce
    /// `SK_BOB`**, cobra ese pendiente en su propia cuenta.
    /// R-2a: el envío registra `(emisor, nacimiento)` y el cobro lo borra.
    #[test]
    fn send_registra_el_meta_y_claim_lo_borra() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        let id_bob = layer.public_id_of(bob).expect("bob");
        let ea = state_of(&layer, alice);
        let seq_antes = layer.epoch_head(zk_ssl_hash::as_digest(0), 0, zk_ssl_hash::as_digest(0), 0).seq;

        let recibo = layer
            .send(BaseElement::new(SK_ALICE), alice, &ea, id_bob, salt_de(0xB2), 250_000)
            .expect("send");
        layer.apply_send(&recibo, alice, &ea, 250_000).expect("apply");

        let pos = recibo.notice.position;
        let (emisor, nacido) = layer.pending_meta_of(pos).expect("meta debe existir");
        assert_eq!(emisor, alice, "el meta apunta al EMISOR");
        assert_eq!(nacido, seq_antes, "nacido = seq del lote que lo creó");

        let eb = state_of(&layer, bob);
        let cobro = layer
            .claim(BaseElement::new(SK_BOB), bob, &eb, &recibo.notice)
            .expect("claim");
        layer.apply_claim(&cobro, bob, &eb, &recibo.notice).expect("apply claim");
        assert!(layer.pending_meta_of(pos).is_none(), "el cobro borra el meta");
    }

    /// R-2a: la emisión-a-pendiente lleva el CENTINELA (des-emisión, no
    /// reembolso: no hay emisor-cuenta).
    #[test]
    fn mint_a_pendiente_lleva_el_centinela() {
        let mut layer = new_layer();
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        let id_bob = layer.public_id_of(bob).expect("bob");
        mint_to_pending_delegated(&mut layer, id_bob, salt_de(0xA1), 500_000);
        // Capa fresca: allocate_pending entrega la posición 0.
        let (emisor, _) = layer.pending_meta_of(0).expect("meta");
        assert_eq!(emisor, crate::REFUND_SENDER_NONE, "centinela de emisión");
    }

    /// R-2a: `T` es línea sistémica — con defecto declarado y knob.
    #[test]
    fn la_t_de_caducidad_tiene_defecto_y_knob() {
        let mut layer = new_layer();
        assert_eq!(layer.refund_ttl(), crate::DEFAULT_REFUND_TTL);
        layer.set_refund_ttl(7);
        assert_eq!(layer.refund_ttl(), 7);
    }

    fn refund_de(
        layer: &SovereignLayer,
        idx: AccountIndex,
        sk: u64,
        recibo: &SendReceipt,
        receptor: Digest,
        amount: u64,
    ) -> RefundReceipt {
        let estado = state_of(layer, idx);
        layer
            .refund(
                BaseElement::new(sk), idx, &estado,
                recibo.notice.position, receptor, recibo.notice.salt, amount,
            )
            .expect("materiales de reembolso")
    }

    /// R-2c FELIZ: tras T, el reembolso devuelve al emisor — con cronómetro.
    #[test]
    fn un_refund_tras_t_devuelve_al_emisor() {
        use std::time::Instant;
        let mut layer = new_layer();
        layer.set_refund_ttl(1);
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        let receptor = layer.public_id_of(bob).expect("bob");
        let ea = state_of(&layer, alice);
        let recibo = layer
            .send(BaseElement::new(SK_ALICE), alice, &ea, receptor, salt_de(0xF1), 250_000)
            .expect("send");
        layer.apply_send(&recibo, alice, &ea, 250_000).expect("apply send");
        assert_eq!(state_of(&layer, alice).balance, 750_000);

        let t = Instant::now();
        let materiales = refund_de(&layer, alice, SK_ALICE, &recibo, receptor, 250_000);
        let t_gen = t.elapsed();
        let t = Instant::now();
        layer.apply_refund(&materiales).expect("apply refund");
        eprintln!(
            "REFUND — generar: {:?} | aplicar: {:?} | apertura {} B + credito {} B",
            t_gen, t.elapsed(),
            materiales.refund_proof.len(), materiales.credit_proof.len()
        );
        assert_eq!(state_of(&layer, alice).balance, 1_000_000, "el dinero VUELVE");
        assert!(layer.pending_meta_of(recibo.notice.position).is_none());
        assert_eq!(layer.total_pending(), 0);
    }

    /// R-2c: antes de T, la compuerta rebota con su error nominal.
    #[test]
    fn antes_de_t_el_reembolso_rebota() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        let receptor = layer.public_id_of(bob).expect("bob");
        let ea = state_of(&layer, alice);
        let recibo = layer
            .send(BaseElement::new(SK_ALICE), alice, &ea, receptor, salt_de(0xF2), 100_000)
            .expect("send");
        layer.apply_send(&recibo, alice, &ea, 100_000).expect("apply");
        let materiales = refund_de(&layer, alice, SK_ALICE, &recibo, receptor, 100_000);
        assert!(
            matches!(layer.apply_refund(&materiales), Err(LayerError::RefundTooEarly { .. })),
            "con T=64 por defecto, un latido no basta"
        );
    }

    /// R-2c EL LADRÓN: con el aviso entero, Mallory fabrica materiales —
    /// pero su clímber acredita SU hoja, y el destino lo fijan los
    /// registros: la raíz tentativa (que acredita a ALICE) no casa.
    #[test]
    fn el_ladron_con_aviso_no_puede_reembolsarse() {
        let mut layer = new_layer();
        layer.set_refund_ttl(1);
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let mallory = open_and_fund(&mut layer, SK_MALLORY, 0);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        let receptor = layer.public_id_of(bob).expect("bob");
        let ea = state_of(&layer, alice);
        let recibo = layer
            .send(BaseElement::new(SK_ALICE), alice, &ea, receptor, salt_de(0xF3), 200_000)
            .expect("send");
        layer.apply_send(&recibo, alice, &ea, 200_000).expect("apply");

        let robado = refund_de(&layer, mallory, SK_MALLORY, &recibo, receptor, 200_000);
        assert!(
            matches!(layer.apply_refund(&robado), Err(LayerError::VerificationFailed(_))),
            "el credito del ladron no casa la raiz que acredita al EMISOR"
        );
        assert_eq!(state_of(&layer, mallory).balance, 0, "el ladron no cobra");
        assert!(layer.pending_meta_of(recibo.notice.position).is_some(), "el pendiente sigue");
    }

    /// R-2c LA CARRERA, ambos órdenes: el primero gana, el segundo rebota.
    #[test]
    fn la_carrera_post_t_el_primero_gana() {
        // Orden A: claim primero → refund rebota (hoja vacía).
        let mut layer = new_layer();
        layer.set_refund_ttl(1);
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        let receptor = layer.public_id_of(bob).expect("bob");
        let ea = state_of(&layer, alice);
        let recibo = layer
            .send(BaseElement::new(SK_ALICE), alice, &ea, receptor, salt_de(0xF4), 50_000)
            .expect("send");
        layer.apply_send(&recibo, alice, &ea, 50_000).expect("apply");
        let materiales = refund_de(&layer, alice, SK_ALICE, &recibo, receptor, 50_000);
        let eb = state_of(&layer, bob);
        let cobro = layer
            .claim(BaseElement::new(SK_BOB), bob, &eb, &recibo.notice)
            .expect("claim");
        layer.apply_claim(&cobro, bob, &eb, &recibo.notice).expect("apply claim");
        assert!(
            matches!(layer.apply_refund(&materiales), Err(LayerError::RefundUnavailable)),
            "el cobro se lleva el meta consigo: el reembolso ya NI CALIFICA \
             (RefundUnavailable, la primera compuerta) — mejor que encontrar \
             la hoja vacia"
        );

        // Orden B: refund primero → claim rebota (raíz de pendientes movida).
        let mut layer = new_layer();
        layer.set_refund_ttl(1);
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        let receptor = layer.public_id_of(bob).expect("bob");
        let ea = state_of(&layer, alice);
        let recibo = layer
            .send(BaseElement::new(SK_ALICE), alice, &ea, receptor, salt_de(0xF5), 50_000)
            .expect("send");
        layer.apply_send(&recibo, alice, &ea, 50_000).expect("apply");
        let eb = state_of(&layer, bob);
        let cobro = layer
            .claim(BaseElement::new(SK_BOB), bob, &eb, &recibo.notice)
            .expect("claim generado ANTES del refund");
        let materiales = refund_de(&layer, alice, SK_ALICE, &recibo, receptor, 50_000);
        layer.apply_refund(&materiales).expect("refund gana");
        assert!(
            layer.apply_claim(&cobro, bob, &eb, &recibo.notice).is_err(),
            "tras el reembolso, el cobro rebota"
        );
        assert_eq!(state_of(&layer, alice).balance, 1_000_000);
        assert_eq!(state_of(&layer, bob).balance, 0);
    }

    /// R-2c PERSISTENCIA: el meta sobrevive al reinicio, y el reembolso
    /// funciona sobre la capa reabierta (patrón open_retry).
    #[test]
    fn el_meta_sobrevive_al_reinicio_y_el_refund_funciona() {
        let path = std::env::temp_dir()
            .join(format!(
                "zkssl_meta_reinicio_{}_{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ))
            .to_string_lossy()
            .into_owned();
        let (alice, recibo, receptor, born);
        {
            let mut layer = open_retry(
                &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS)
                .expect("abrir");
            alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
            let bob = open_and_fund(&mut layer, SK_BOB, 0);
            receptor = layer.public_id_of(bob).expect("bob");
            let ea = state_of(&layer, alice);
            recibo = layer
                .send(BaseElement::new(SK_ALICE), alice, &ea, receptor, salt_de(0xF6), 300_000)
                .expect("send");
            layer.apply_send(&recibo, alice, &ea, 300_000).expect("apply");
            born = layer.pending_meta_of(recibo.notice.position).expect("meta").1;
        }
        let mut layer = open_retry(
            &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS)
            .expect("reabrir");
        assert_eq!(
            layer.pending_meta_of(recibo.notice.position),
            Some((alice, born)),
            "el meta VIAJÓ en el lote y sobrevivió"
        );
        layer.set_refund_ttl(1);
        let materiales = refund_de(&layer, alice, SK_ALICE, &recibo, receptor, 300_000);
        layer.apply_refund(&materiales).expect("refund tras reinicio");
        assert_eq!(state_of(&layer, alice).balance, 1_000_000);
    }

    /// **TESTIGO §388 (punto 41, T-B): el reloj del reembolso no es fe del
    /// disco.** `born` decide cuando un pendiente es reembolsable
    /// (`apply_refund` / `apply_deissue`: `now - born < plazo`) y hasta el
    /// §387 vivia SOLO en `pmeta:`, sin raiz. Se miente `born` en sled y el
    /// libro NO ABRE: la raiz de meta (`root:pmeta`) no cuadra.
    #[test]
    fn un_born_mentido_en_reposo_no_abre() {
        let path = ruta_temporal("meta_born_mentido");
        let (alice, pos, born) = ledger_con_un_pendiente(&path);
        {
            let db = sled_open_retry(&path);
            let mut key = b"pmeta:".to_vec();
            key.extend_from_slice(&pos.to_le_bytes());
            let mut v = alice.to_le_bytes().to_vec();
            v.extend_from_slice(&born.wrapping_add(1_000_000).to_le_bytes());
            db.insert(key, v).expect("insertar");
            db.flush().expect("flush");
        }
        let r = open_retry(
            &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
        );
        assert!(
            matches!(
                r,
                Err(LayerError::Store(crate::store::StoreError::IntegrityFailure {
                    what: "meta de pendientes"
                }))
            ),
            "CRITICO: un nacimiento mentido en pmeta: debe pararse ANTES de operar"
        );
        let _ = std::fs::remove_dir_all(&path);
    }

    /// **TESTIGO §388 (T-B bis): la AUSENCIA de meta tambien es una mentira.**
    /// `load` salta un `pmeta:` vacio ("vaciada o legado"); vaciarlo a mano
    /// dejaria el pendiente sin reembolso posible (`RefundUnavailable`). La
    /// hoja de esa posicion pasa a cero y la raiz ya no cuadra.
    #[test]
    fn un_meta_borrado_en_reposo_no_abre() {
        let path = ruta_temporal("meta_borrado");
        let (_alice, pos, _born) = ledger_con_un_pendiente(&path);
        {
            let db = sled_open_retry(&path);
            let mut key = b"pmeta:".to_vec();
            key.extend_from_slice(&pos.to_le_bytes());
            db.insert(key, Vec::new()).expect("insertar");
            db.flush().expect("flush");
        }
        let r = open_retry(
            &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
        );
        assert!(
            matches!(
                r,
                Err(LayerError::Store(crate::store::StoreError::IntegrityFailure {
                    what: "meta de pendientes"
                }))
            ),
            "CRITICO: un meta borrado en pmeta: debe pararse ANTES de operar"
        );
        let _ = std::fs::remove_dir_all(&path);
    }

    /// Ayudantes de los dos testigos: una ruta propia, y un ledger con UN
    /// pendiente v1 de alice a bob ya cerrado (el lote esta en disco).
    fn ruta_temporal(nombre: &str) -> String {
        std::env::temp_dir()
            .join(format!(
                "zkssl_{}_{}_{}",
                nombre,
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ))
            .to_string_lossy()
            .into_owned()
    }

    fn ledger_con_un_pendiente(path: &str) -> (AccountIndex, u64, u64) {
        let mut layer = open_retry(
            path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
        )
        .expect("abrir");
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        let receptor = layer.public_id_of(bob).expect("bob");
        let ea = state_of(&layer, alice);
        let recibo = layer
            .send(BaseElement::new(SK_ALICE), alice, &ea, receptor, salt_de(0x388), 300_000)
            .expect("send");
        layer.apply_send(&recibo, alice, &ea, 300_000).expect("apply");
        let pos = recibo.notice.position;
        let born = layer.pending_meta_of(pos).expect("meta").1;
        (alice, pos, born)
    }

    /// §391: un ledger con alice CONGELADA por dos custodios y bob libre; el lote
    /// (froz: y root:froz) esta en disco.
    fn ledger_con_una_congelada(path: &str) -> (AccountIndex, AccountIndex) {
        let mut layer = open_retry(
            path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
        )
        .expect("abrir");
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        set_frozen_delegated(&mut layer, alice, true);
        assert!(layer.is_frozen(alice) && !layer.is_frozen(bob));
        (alice, bob)
    }

    fn clave_froz(idx: AccountIndex) -> Vec<u8> {
        let mut key = b"froz:".to_vec();
        key.extend_from_slice(&idx.to_le_bytes());
        key
    }

    /// §391 (punto 45): el adversario del disco DESCONGELA borrando la clave
    /// `froz:` de la cuenta. Sin raiz en reposo, `load` reconstruia el arbol de
    /// lo que quedaba y se lo creia.
    #[test]
    fn una_congelada_borrada_en_reposo_no_abre() {
        let path = ruta_temporal("froz_borrada");
        let (alice, _bob) = ledger_con_una_congelada(&path);
        {
            let db = sled_open_retry(&path);
            assert!(
                db.remove(clave_froz(alice)).expect("borrar").is_some(),
                "habia una froz: de alice que borrar"
            );
            db.flush().expect("flush");
        }
        let r = open_retry(
            &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
        );
        assert!(
            matches!(
                r,
                Err(LayerError::Store(crate::store::StoreError::IntegrityFailure {
                    what: "arbol de congelados"
                }))
            ),
            "CRITICO: una congelacion levantada desde el disco debe pararse ANTES de operar"
        );
        let _ = std::fs::remove_dir_all(&path);
    }

    /// §391: el reves. El adversario del disco CONGELA a bob copiando la hoja
    /// de alice a la posicion de bob, sin custodios.
    #[test]
    fn una_congelada_anadida_en_reposo_no_abre() {
        let path = ruta_temporal("froz_anadida");
        let (alice, bob) = ledger_con_una_congelada(&path);
        {
            let db = sled_open_retry(&path);
            let hoja = db.get(clave_froz(alice)).expect("leer").expect("habia una froz: de alice");
            db.insert(clave_froz(bob), hoja.to_vec()).expect("insertar");
            db.flush().expect("flush");
        }
        let r = open_retry(
            &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
        );
        assert!(
            matches!(
                r,
                Err(LayerError::Store(crate::store::StoreError::IntegrityFailure {
                    what: "arbol de congelados"
                }))
            ),
            "CRITICO: una congelacion impuesta desde el disco debe pararse ANTES de operar"
        );
        let _ = std::fs::remove_dir_all(&path);
    }

    /// §391: un libro sin `root:froz` (anterior a este sello) NO ABRE: fail-closed,
    /// sin era silenciosa, como `root:pending` (§387) y `root:pmeta` (§388).
    #[test]
    fn un_libro_sin_root_froz_no_abre() {
        let path = ruta_temporal("sin_root_froz");
        let _cuentas = ledger_con_una_congelada(&path);
        {
            let db = sled_open_retry(&path);
            assert!(
                db.remove(b"root:froz").expect("borrar").is_some(),
                "habia una raiz de congelados que borrar"
            );
            db.flush().expect("flush");
        }
        let r = open_retry(
            &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
        );
        assert!(
            matches!(
                r,
                Err(LayerError::Store(crate::store::StoreError::Malformed(ref m))) if m.contains("root:froz")
            ),
            "CRITICO: un libro sin raiz de congelados no debe abrir"
        );
        let _ = std::fs::remove_dir_all(&path);
    }

    fn clave_log(seq: u64) -> Vec<u8> {
        let mut key = b"log:".to_vec();
        key.extend_from_slice(&seq.to_le_bytes());
        key
    }

    /// §392: una entrada del registro ALTERADA en reposo no abre. Se toca el
    /// `seq` de la ultima entrada (byte 0 del formato 137/169 del store): la
    /// cadena recomputada por `verify_chain` ya no cuadra y la capa se para.
    #[test]
    fn una_entrada_del_registro_alterada_en_reposo_no_abre() {
        let path = ruta_temporal("log_alterada");
        let _cuentas = ledger_con_una_congelada(&path);
        {
            let db = sled_open_retry(&path);
            let n = db.scan_prefix(b"log:").count() as u64;
            assert!(n > 0, "habia entradas del registro que alterar");
            let key = clave_log(n - 1);
            let mut v = db.get(&key).expect("leer").expect("existe la ultima entrada").to_vec();
            v[0] ^= 1;
            db.insert(key, v).expect("insertar");
            db.flush().expect("flush");
        }
        let r = open_retry(
            &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
        );
        assert!(
            matches!(
                r,
                Err(LayerError::Store(crate::store::StoreError::IntegrityFailure {
                    what: "registro de transiciones"
                }))
            ),
            "CRITICO: una entrada del registro alterada en reposo no debe abrir. Salio: {:?}",
            r.err()
        );
        let _ = std::fs::remove_dir_all(&path);
    }

    /// §392: una entrada del registro BORRADA en reposo no abre. Se borra la
    /// ultima: la cadena que queda verifica (una copia atrasada no es una
    /// bifurcacion), y por eso hace falta la raiz: la cabeza ya no cuadra.
    #[test]
    fn una_entrada_del_registro_borrada_en_reposo_no_abre() {
        let path = ruta_temporal("log_borrada");
        let _cuentas = ledger_con_una_congelada(&path);
        {
            let db = sled_open_retry(&path);
            let n = db.scan_prefix(b"log:").count() as u64;
            assert!(n > 0, "habia entradas del registro que borrar");
            assert!(
                db.remove(clave_log(n - 1)).expect("borrar").is_some(),
                "la ultima entrada existia"
            );
            db.flush().expect("flush");
        }
        let r = open_retry(
            &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
        );
        assert!(
            matches!(
                r,
                Err(LayerError::Store(crate::store::StoreError::IntegrityFailure {
                    what: "registro de transiciones"
                }))
            ),
            "CRITICO: un registro truncado en reposo no debe abrir. Salio: {:?}",
            r.err()
        );
        let _ = std::fs::remove_dir_all(&path);
    }

    /// §392: un libro sin `root:log` NO abre (fail-closed, como sin las otras raices).
    #[test]
    fn un_libro_sin_root_log_no_abre() {
        let path = ruta_temporal("sin_root_log");
        let _cuentas = ledger_con_una_congelada(&path);
        {
            let db = sled_open_retry(&path);
            assert!(
                db.remove(b"root:log").expect("borrar").is_some(),
                "habia una raiz del registro que borrar"
            );
            db.flush().expect("flush");
        }
        let r = open_retry(
            &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
        );
        assert!(
            matches!(
                r,
                Err(LayerError::Store(crate::store::StoreError::Malformed(ref m))) if m.contains("root:log")
            ),
            "CRITICO: un libro sin raiz del registro no debe abrir"
        );
        let _ = std::fs::remove_dir_all(&path);
    }

    /// §393: pone un contador custodiado en `v` por el disco. El valor va sin
    /// sellar, como en los testigos del §388, §391 y §392: con `key: None`,
    /// `seal` es la identidad.
    fn poner_contador(path: &str, clave: &[u8], v: u64) {
        let db = sled_open_retry(path);
        assert!(
            db.get(clave).expect("leer").is_some(),
            "la clave del contador existia antes de tocarla"
        );
        db.insert(clave, v.to_le_bytes().to_vec()).expect("insertar");
        db.flush().expect("flush");
    }

    fn no_abre_por_contadores(path: &str) {
        let r = open_retry(
            path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
        );
        assert!(
            matches!(
                r,
                Err(LayerError::Store(crate::store::StoreError::IntegrityFailure {
                    what: "contadores custodiados"
                }))
            ),
            "CRITICO: un contador que no cuadra con el registro no debe abrir. Salio: {:?}",
            r.err()
        );
    }

    fn no_abre_por_cuota(path: &str) {
        let r = open_retry(
            path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
        );
        assert!(
            matches!(
                r,
                Err(LayerError::Store(crate::store::StoreError::IntegrityFailure {
                    what: "cuota de custodios"
                }))
            ),
            "CRITICO: una cuota que no cuadra con el registro no debe abrir. Salio: {:?}",
            r.err()
        );
    }

    /// §394: REBOBINAR `meta:cust_uses` es el ataque de verdad: le devuelve cupo a
    /// un conjunto de custodios que ya lo gasto. El registro tiene una entrada
    /// `Freeze` y la cuota diria cero: no debe abrir.
    #[test]
    fn una_cuota_de_custodios_rebobinada_no_abre() {
        let path = ruta_temporal("rebobinar_cuota");
        let _cuentas = ledger_con_una_congelada(&path);
        poner_contador(&path, b"meta:cust_uses", 0);
        no_abre_por_cuota(&path);
        let _ = std::fs::remove_dir_all(&path);
    }

    /// §394: una cuota ADELANTADA tambien miente -gasta cupo que nadie uso- y la
    /// misma puerta la caza.
    #[test]
    fn una_cuota_de_custodios_adelantada_no_abre() {
        let path = ruta_temporal("adelantar_cuota");
        let _cuentas = ledger_con_una_congelada(&path);
        poner_contador(&path, b"meta:cust_uses", 7);
        no_abre_por_cuota(&path);
        let _ = std::fs::remove_dir_all(&path);
    }

    /// §393: REBOBINAR `meta:freezes` es el ataque de verdad — revive una
    /// autorizacion de custodios ya gastada, porque el compromiso que ellos
    /// firmaron lleva (count_old, count_new). El registro tiene una entrada
    /// `Freeze` y el contador diria cero: no debe abrir.
    #[test]
    fn un_contador_de_congelaciones_rebobinado_no_abre() {
        let path = ruta_temporal("rebobinar_freezes");
        let _cuentas = ledger_con_una_congelada(&path);
        poner_contador(&path, b"meta:freezes", 0);
        no_abre_por_contadores(&path);
        let _ = std::fs::remove_dir_all(&path);
    }

    /// §393: un contador ADELANTADO tambien miente. El registro no tiene
    /// ninguna entrada `Recovery` y el contador diria una: no debe abrir.
    #[test]
    fn un_contador_de_recuperaciones_adelantado_no_abre() {
        let path = ruta_temporal("adelantar_recoveries");
        let _cuentas = ledger_con_una_congelada(&path);
        poner_contador(&path, b"meta:recoveries", 1);
        no_abre_por_contadores(&path);
        let _ = std::fs::remove_dir_all(&path);
    }

    /// §393: lo mismo con `meta:gov_changes`, que es el contador del conjunto
    /// de custodios: adelantarlo revive una autorizacion de gobernanza.
    #[test]
    fn un_contador_de_gobernanza_adelantado_no_abre() {
        let path = ruta_temporal("adelantar_gov");
        let _cuentas = ledger_con_una_congelada(&path);
        poner_contador(&path, b"meta:gov_changes", 1);
        no_abre_por_contadores(&path);
        let _ = std::fs::remove_dir_all(&path);
    }

    /// R-2d FELIZ: el mint-pendiente caducado se DES-EMITE — el
    /// suministro baja exactamente lo que subió al emitir. Con cronómetro.
    #[test]
    fn un_mint_pendiente_caducado_se_desemite() {
        use std::time::Instant;
        let mut layer = new_layer();
        layer.set_refund_ttl(1);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        let receptor = layer.public_id_of(bob).expect("bob");
        let supply_antes = layer.total_supply();
        mint_to_pending_delegated(&mut layer, receptor, salt_de(0xD1), 500_000);
        assert_eq!(layer.total_supply(), supply_antes + 500_000, "emitir SUBE");
        let t = Instant::now();
        let materiales = layer
            .deissue(0, receptor, salt_de(0xD1), 500_000)
            .expect("materiales");
        let t_gen = t.elapsed();
        let t = Instant::now();
        layer.apply_deissue(&materiales).expect("des-emitir");
        eprintln!(
            "DESEMISION — generar: {:?} | aplicar: {:?} | apertura {} B",
            t_gen, t.elapsed(), materiales.refund_proof.len()
        );
        assert_eq!(layer.total_supply(), supply_antes, "caducar BAJA lo emitido");
        assert!(layer.pending_meta_of(0).is_none());
        assert_eq!(layer.total_pending(), 0);
    }

    /// R-2d: antes de T, la des-emisión rebota igual que el reembolso.
    #[test]
    fn la_desemision_pre_t_rebota() {
        let mut layer = new_layer();
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        let receptor = layer.public_id_of(bob).expect("bob");
        mint_to_pending_delegated(&mut layer, receptor, salt_de(0xD2), 100_000);
        let materiales = layer
            .deissue(0, receptor, salt_de(0xD2), 100_000)
            .expect("materiales");
        assert!(matches!(
            layer.apply_deissue(&materiales),
            Err(LayerError::RefundTooEarly { .. })
        ));
    }

    /// R-2d LAS VÍAS NO SE CRUZAN: el centinela no acepta reembolso-con-
    /// crédito, y el pendiente de pago no acepta des-emisión.
    #[test]
    fn las_dos_vias_de_caducidad_no_se_cruzan() {
        let mut layer = new_layer();
        layer.set_refund_ttl(1);
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        let receptor = layer.public_id_of(bob).expect("bob");

        // Un mint-pendiente (centinela) en pos 0…
        mint_to_pending_delegated(&mut layer, receptor, salt_de(0xD3), 40_000);
        // …y un pendiente de PAGO de alice en pos 1.
        let ea = state_of(&layer, alice);
        let recibo = layer
            .send(BaseElement::new(SK_ALICE), alice, &ea, receptor, salt_de(0xD4), 60_000)
            .expect("send");
        layer.apply_send(&recibo, alice, &ea, 60_000).expect("apply");

        // Vía cruzada 1: refund-con-crédito sobre el CENTINELA → rebota.
        // (Directo, sin recibo sintético: alice apunta su refund a la
        // posición 0 del mint-pendiente — lo que un atacante haría.)
        let ea2 = state_of(&layer, alice);
        let robado = layer
            .refund(
                BaseElement::new(SK_ALICE), alice, &ea2,
                0, receptor, salt_de(0xD3), 40_000,
            )
            .expect("materiales contra el centinela");
        assert!(matches!(
            layer.apply_refund(&robado),
            Err(LayerError::RefundUnavailable)
        ), "el centinela NO admite credito a nadie");

        // Vía cruzada 2: des-emisión sobre el pendiente de PAGO → rebota.
        let destruir = layer
            .deissue(recibo.notice.position, receptor, salt_de(0xD4), 60_000)
            .expect("materiales");
        assert!(matches!(
            layer.apply_deissue(&destruir),
            Err(LayerError::RefundUnavailable)
        ), "el pago NO se destruye: su via es el reembolso al emisor");
    }

    #[test]
    fn claiming_a_pending_requires_knowing_the_recipients_key() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, FONDO);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        let mallory = open_and_fund(&mut layer, SK_MALLORY, 0);

        // Un pago honesto de Alice a Bob.
        let estado_alice = state_of(&layer, alice);
        let recibo = layer
            .send(
                BaseElement::new(SK_ALICE),
                alice,
                &estado_alice,
                layer.public_id_of(bob).expect("cuenta"),
                salt_de(0x5EED),
                IMPORTE,
            )
            .expect("envio honesto");
        layer
            .apply_send(&recibo, alice, &estado_alice, IMPORTE)
            .expect("el envio honesto debe aplicarse");
        assert_eq!(layer.total_pending(), IMPORTE, "el pendiente esta ahi");

        // Mallory cobra lo de Bob. No conoce SK_BOB.
        let estado_mallory = state_of(&layer, mallory);
        let salt_mallory = layer
            .records
            .get(&mallory)
            .map(|r| r.leaf_salt)
            .unwrap_or(crate::store::LEAF_SALT_LEGACY);
        let mut cuentas = layer.accounts.clone();
        cuentas.set_leaf(
            mallory,
            native_leaf_salted(
                estado_mallory.public_id,
                BaseElement::new(estado_mallory.balance + IMPORTE),
                estado_mallory.nonce,
                salt_mallory,
            ),
        );
        let mut pend = layer.pending.clone();
        pend.set_leaf(recibo.notice.position, [BaseElement::ZERO; 4]);

        let cobro = ClaimReceipt {
            proof: vec![0u8; 32],
            public_inputs: ClaimPublicInputs {
                root_old: layer.accounts.root(),
                root_new: cuentas.root(),
                frozen_root: layer.frozen.root(),
                pending_root_old: layer.pending.root(),
                pending_root_new: pend.root(),
                amount: BaseElement::new(IMPORTE),
                supply_old: BaseElement::new(layer.total_supply()),
                supply_new: BaseElement::new(layer.total_supply()),
            },
        };

        let r = layer.apply_claim(&cobro, mallory, &estado_mallory, &recibo.notice);
        let saldo_m = layer.balance_of(mallory);
        let saldo_b = layer.balance_of(bob);

        assert!(
            r.is_err(),
            "SOLIDEZ: cobrar NO requiere la clave del receptor. Mallory, sin \
             conocer SK_BOB, ha cobrado en su cuenta un pendiente dirigido a \
             Bob. Fue {r:?}. Mallory: {saldo_m:?}, Bob: {saldo_b:?}."
        );
        assert_eq!(saldo_m, Some(0), "Mallory no debe haber cobrado nada");
        assert_eq!(
            layer.total_pending(),
            IMPORTE,
            "y el pendiente de Bob debe seguir ahi"
        );
    }

    /// **TESTIGO 4: ¿hace falta la clave del titular para gastar?**
    ///
    /// Los testigos 1-3 midieron que la capa no verifica las pruebas. De
    /// ahi se **sigue** que `apply_send` no comprueba que quien gasta
    /// conozca la clave. Seguirse no es medirse.
    ///
    /// ## El escenario
    ///
    /// Mallory **no conoce `SK_ALICE`**. Conoce el estado de la cuenta de
    /// Alice, que la capa expone por `balance_of`, `nonce_of` y
    /// `public_id_of` —las tres `pub`—. Con eso fabrica un recibo entero:
    /// prueba a ceros, raices coherentes, y un pendiente dirigido a **su
    /// propia identidad**, no a la de Alice.
    ///
    /// ## Por que no puede rechazar por otra cosa
    ///
    /// `apply_send` tiene **exactamente cuatro** condiciones de rechazo,
    /// contadas sobre el codigo: las dos raices vigentes, la de congelados,
    /// el limite regulatorio declarado, y que las raices nuevas cuadren con
    /// lo que la propia capa recomputa. Las cuatro se satisfacen aqui a
    /// proposito. **Ninguna es una clave. Ninguna es una prueba.**
    ///
    /// El importe va por debajo del limite regulatorio para que ese camino
    /// tampoco pueda dar un rechazo por el motivo equivocado.
    ///
    /// ## Y el pendiente es COBRABLE por ella
    ///
    /// El compromiso se forma con la identidad de Mallory, asi que es ella
    /// —y solo ella— quien puede reclamarlo despues. No es un pago perdido
    /// en el limbo: es una transferencia de Alice a Mallory que Alice no
    /// autorizo.
    #[test]
    fn spending_from_an_account_requires_knowing_its_key() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, FONDO);
        let _mallory = open_and_fund(&mut layer, SK_MALLORY, 0);

        // Lo unico que Mallory necesita saber, y la capa se lo da.
        let victima = state_of(&layer, alice);

        // Un pendiente dirigido a ELLA MISMA.
        let id_mallory = derive_public_id(BaseElement::new(SK_MALLORY));
        let salt = salt_de(0xBADC0DE);
        let commitment = pending_commitment(id_mallory, salt, IMPORTE);

        // Raices coherentes con el robo.
        let salt_alice = layer
            .records
            .get(&alice)
            .map(|r| r.leaf_salt)
            .unwrap_or(crate::store::LEAF_SALT_LEGACY);
        let mut cuentas = layer.accounts.clone();
        cuentas.set_leaf(
            alice,
            native_leaf_salted(
                victima.public_id,
                BaseElement::new(victima.balance - IMPORTE),
                victima.nonce,
                salt_alice,
            ),
        );
        let position = layer.allocate_pending().expect("posicion libre");
        let mut pend = layer.pending.clone();
        pend.set_leaf(position, commitment);

        let recibo = SendReceipt {
            // Ni siquiera se molesta en generar una.
            proof: vec![0u8; 32],
            public_inputs: SendPublicInputs {
                root_old: layer.accounts.root(),
                root_new: cuentas.root(),
                frozen_root: layer.frozen.root(),
                pending_root_old: layer.pending.root(),
                pending_root_new: pend.root(),
                amount: BaseElement::new(IMPORTE),
                regulatory_limit: BaseElement::new(layer.regulatory_limit),
                supply_old: BaseElement::new(layer.total_supply()),
                supply_new: BaseElement::new(layer.total_supply()),
            },
            commitment,
            notice: PendingNotice {
                position,
                salt,
                amount: IMPORTE,
                x: None,
            },
        };

        let r = layer.apply_send(&recibo, alice, &victima, IMPORTE);
        let saldo = layer.balance_of(alice);
        let en_transito = layer.total_pending();

        assert!(
            r.is_err(),
            "SOLIDEZ: gastar NO requiere la clave del titular. Mallory, sin \
             conocer SK_ALICE, ha debitado {IMPORTE} de la cuenta de Alice y \
             ha creado un pendiente dirigido a su propia identidad, con una \
             prueba de 32 ceros. Fue {r:?}. Saldo de la victima: {saldo:?}. \
             En transito: {en_transito}."
        );
        assert_eq!(
            saldo,
            Some(FONDO),
            "el saldo de la victima no debe haberse tocado"
        );
        assert_eq!(en_transito, 0, "ni haberse creado ningun pendiente");
    }

    fn id_bob() -> Digest {
        derive_public_id(BaseElement::new(SK_BOB))
    }

    /// **TESTIGO 1: una prueba que no prueba nada.**
    ///
    /// Recibo honesto en todo salvo en la prueba, que se pone a ceros. Si
    /// la capa la verificara, esto se rechaza; si no la mira, la longitud
    /// del vector es lo unico que cambia y la operacion se aplica igual.
    ///
    /// Nada mas se toca: las raices, el compromiso y el aviso son los que
    /// genero `send`. Si rechaza, rechaza por la prueba.
    #[test]
    fn apply_send_rejects_a_receipt_whose_proof_is_garbage() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, FONDO);
        let estado = state_of(&layer, alice);

        let mut recibo = layer
            .send(
                BaseElement::new(SK_ALICE),
                alice,
                &estado,
                id_bob(),
                salt_de(0x5EED),
                IMPORTE,
            )
            .expect("el recibo honesto debe generarse");

        // La prueba, a ceros. Misma longitud, cero contenido.
        recibo.proof = vec![0u8; recibo.proof.len()];

        let r = layer.apply_send(&recibo, alice, &estado, IMPORTE);

        assert!(
            r.is_err(),
            "SOLIDEZ: `apply_send` aplica un pago cuya prueba es basura. La \
             capa NO verifica la prueba de la via de dos fases -la unica via \
             de pago desde §36-, asi que el circuito no protege nada aqui. \
             Fue {r:?}"
        );
        assert_eq!(
            layer.balance_of(alice),
            Some(FONDO),
            "y el saldo no debe haberse movido"
        );
        assert_eq!(layer.total_pending(), 0, "ni haberse creado el pendiente");
    }

    /// **TESTIGO 2: el titular miente sobre su propio saldo.**
    ///
    /// `apply_send` toma `sender_state` **del que llama** y escribe
    /// `saldo - importe` en la hoja, comprobando solo que la raiz declarada
    /// cuadre con lo que el mismo recomputa. Si nada ata ese estado a la
    /// hoja real -y la prueba es lo unico que podria hacerlo-, declarar un
    /// saldo de diez millones sobre una cuenta de uno **escribe diez
    /// millones menos el importe**.
    ///
    /// Las raices declaradas se construyen coherentes **con la mentira**, a
    /// proposito: si la capa las comprobara contra el estado real en vez de
    /// contra lo recomputado, el test lo detecta.
    #[test]
    fn apply_send_rejects_a_lied_holder_state() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, FONDO);
        let verdad = state_of(&layer, alice);

        let honesto = layer
            .send(
                BaseElement::new(SK_ALICE),
                alice,
                &verdad,
                id_bob(),
                salt_de(0x5EED),
                IMPORTE,
            )
            .expect("recibo honesto");

        // Diez veces el saldo real.
        let mentira = ClientState {
            balance: FONDO * 10,
            ..verdad.clone()
        };

        // Raices coherentes CON LA MENTIRA.
        let salt_alice = layer
            .records
            .get(&alice)
            .map(|r| r.leaf_salt)
            .unwrap_or(crate::store::LEAF_SALT_LEGACY);
        let mut cuentas = layer.accounts.clone();
        cuentas.set_leaf(
            alice,
            native_leaf_salted(
                mentira.public_id,
                BaseElement::new(mentira.balance - IMPORTE),
                mentira.nonce,
                salt_alice,
            ),
        );
        let mut pend = layer.pending.clone();
        pend.set_leaf(honesto.notice.position, honesto.commitment);

        let recibo = SendReceipt {
            proof: honesto.proof.clone(),
            public_inputs: SendPublicInputs {
                root_new: cuentas.root(),
                pending_root_new: pend.root(),
                ..honesto.public_inputs.clone()
            },
            commitment: honesto.commitment,
            notice: honesto.notice.clone(),
        };

        let r = layer.apply_send(&recibo, alice, &mentira, IMPORTE);

        assert!(
            r.is_err(),
            "SOLIDEZ: `apply_send` acepta un estado de titular MENTIDO. \
             Declarando {} de saldo sobre una cuenta de {}, la capa escribe \
             la mentira menos el importe: dinero de la nada, sin clave y sin \
             prueba valida. Fue {r:?}",
            FONDO * 10,
            FONDO
        );
        assert_eq!(
            layer.balance_of(alice),
            Some(FONDO),
            "el saldo real no debe haberse tocado"
        );
    }


    /// E3b-1: LA IDENTIDAD DEL AVISO. Some(sobre) permite al receptor
    /// recomponer C2 = M(C1, X) sin aprender f ni delta.
    #[test]
    fn el_aviso_v2_reconstruye_c2_con_el_sobre_opaco() {
        use crate::pending::{pending_commitment_v2, refund_envelope};
        use stark_experiment::merkle::native_merge;
        let (r, s, fk) = (salt_de(1), salt_de(2), salt_de(9));
        let (amt, delta) = (250_000u64, 40u64);
        let sobre = refund_envelope(fk, delta);
        assert_eq!(
            native_merge(pending_commitment(r, s, amt), sobre),
            pending_commitment_v2(r, s, amt, fk, delta),
            "el aviso con el sobre opaco recompone C2 exacto"
        );
    }

    /// E3b-1: UN COBRO v2 ATRAVIESA LA CAPA ENTERA. El C2 se PLANTA
    /// (nadie lo produce aun: el envio v2 es de otro corte; los tests de
    /// este fichero tocan el arbol, molde del test de Mallory). El aviso
    /// lleva Some(sobre) y el cobro prueba y verifica con ClaimAirV2.
    #[test]
    fn un_cobro_v2_atraviesa_la_capa() {
        use crate::pending::{pending_commitment_v2, refund_envelope};
        let mut layer = new_layer();
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        let id_bob = layer.public_id_of(bob).expect("cuenta");
        let (salt, fk, delta) = (salt_de(0x5EED), salt_de(0xF00D), 40u64);

        let pos = layer.next_pending;
        let c2 = pending_commitment_v2(id_bob, salt, IMPORTE, fk, delta);
        layer.pending.set_leaf(pos, c2);
        layer.next_pending += 1;
        layer.pending_amounts.insert(pos, IMPORTE);
        layer
            .pending_meta
            .insert(pos, (crate::REFUND_SENDER_NONE, layer.log.len() as u64));

        let aviso = PendingNotice {
            position: pos,
            salt,
            amount: IMPORTE,
            x: Some(refund_envelope(fk, delta)),
        };
        let estado = state_of(&layer, bob);
        let cobro = layer
            .claim(BaseElement::new(SK_BOB), bob, &estado, &aviso)
            .expect("el productor v2 debe probar");
        layer
            .apply_claim(&cobro, bob, &estado, &aviso)
            .expect("el cobro v2 debe verificar y aplicarse");
        assert_eq!(layer.balance_of(bob), Some(IMPORTE), "Bob cobra su v2");
        assert_eq!(
            layer.pending.leaf(pos),
            [BaseElement::ZERO; 4],
            "el pendiente v2 queda consumido"
        );
    }

    /// E3c-1b: EL PRIMER ENVIO v2 DE PUNTA A PUNTA, SIN PLANTAR. La via
    /// viva produce C2 (SendV2Air prueba el cuarto merge), el aviso viaja
    /// con Some(sobre), y el cobro de E3b-1 lo cobra. Nadie toca el arbol
    /// a mano.
    #[test]
    fn un_envio_v2_de_punta_a_punta_se_cobra() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, FONDO);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        let id_bob = layer.public_id_of(bob).expect("bob");
        let f = layer.public_id_of(alice).expect("alice: el retorno");
        let (salt, delta) = (salt_de(0xE3C1), 40u64);

        let m = layer
            .send_materials_v2(alice, id_bob, IMPORTE, salt, f, delta)
            .expect("materiales v2");
        let key_a = [
            BaseElement::new(SK_ALICE),
            BaseElement::ZERO,
            BaseElement::ZERO,
            BaseElement::ZERO,
        ];
        let recibo = crate::client::prove_send(&m, key_a, proof_options())
            .expect("el productor v2 debe probar (SendV2Air)");
        assert!(recibo.notice.x.is_some(), "el aviso viaja con el sobre");

        let ea = state_of(&layer, alice);
        layer
            .apply_send(&recibo, alice, &ea, IMPORTE)
            .expect("el envio v2 debe verificar y aplicarse");
        assert_eq!(layer.balance_of(alice), Some(FONDO - IMPORTE));
        assert_eq!(layer.total_pending(), IMPORTE, "C2 en transito");

        let eb = state_of(&layer, bob);
        let cobro = layer
            .claim(BaseElement::new(SK_BOB), bob, &eb, &recibo.notice)
            .expect("el cobro v2 debe probar");
        layer
            .apply_claim(&cobro, bob, &eb, &recibo.notice)
            .expect("el cobro v2 debe aplicarse");
        assert_eq!(layer.balance_of(bob), Some(IMPORTE), "Bob cobra el v2");
        assert_eq!(layer.total_pending(), 0, "nada queda en transito");
    }

    /// E3c-1b, dominio EN LA CAPA (ida): al recibo v2 se le arranca la X
    /// del aviso y validate_send lo rechaza ANTES del Air (D-7): la
    /// traza de 60 columnas no corresponde a la via v1. El estado no se
    /// toca.
    #[test]
    fn un_recibo_v2_sin_su_x_no_valida() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, FONDO);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        let id_bob = layer.public_id_of(bob).expect("bob");
        let f = layer.public_id_of(alice).expect("alice");
        let m = layer
            .send_materials_v2(alice, id_bob, IMPORTE, salt_de(0xE3C2), f, 40)
            .expect("materiales v2");
        let key_a = [
            BaseElement::new(SK_ALICE),
            BaseElement::ZERO,
            BaseElement::ZERO,
            BaseElement::ZERO,
        ];
        let mut recibo = crate::client::prove_send(&m, key_a, proof_options())
            .expect("prueba v2");
        recibo.notice.x = None;
        let ea = state_of(&layer, alice);
        let r = layer.apply_send(&recibo, alice, &ea, IMPORTE);
        assert!(r.is_err(), "sin la X, la via v1 no acepta una traza v2");
        assert_eq!(layer.balance_of(alice), Some(FONDO), "nada debitado");
    }

    /// E3c-1b, dominio EN LA CAPA (vuelta): a un recibo v1 se le cuela
    /// una X y validate_send lo rechaza ANTES del Air (D-7): la traza
    /// de 56 columnas no corresponde a la via v2. Inmune en ambos
    /// sentidos.
    #[test]
    fn un_recibo_v1_con_x_colada_no_valida() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, FONDO);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        let id_bob = layer.public_id_of(bob).expect("bob");
        let m = layer
            .send_materials(alice, id_bob, IMPORTE, salt_de(0xE3C3))
            .expect("materiales v1");
        let key_a = [
            BaseElement::new(SK_ALICE),
            BaseElement::ZERO,
            BaseElement::ZERO,
            BaseElement::ZERO,
        ];
        let mut recibo = crate::client::prove_send(&m, key_a, proof_options())
            .expect("prueba v1");
        recibo.notice.x = Some(salt_de(0xFA15E));
        let ea = state_of(&layer, alice);
        let r = layer.apply_send(&recibo, alice, &ea, IMPORTE);
        assert!(r.is_err(), "con X colada, la via v2 no acepta una traza v1");
        assert_eq!(layer.balance_of(alice), Some(FONDO), "nada debitado");
    }

    /// E3b-1, dominio y no marca EN LA CAPA: el mismo C2 no se cobra por
    /// la via v1 (aviso sin sobre) - la raiz que la prueba acredita no es
    /// la del arbol real, y el estado no se toca.
    #[test]
    fn un_pendiente_v2_no_se_cobra_sin_el_sobre() {
        use crate::pending::pending_commitment_v2;
        let mut layer = new_layer();
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        let id_bob = layer.public_id_of(bob).expect("cuenta");
        let (salt, fk, delta) = (salt_de(0x5EED), salt_de(0xF00D), 40u64);
        let pos = layer.next_pending;
        layer
            .pending
            .set_leaf(pos, pending_commitment_v2(id_bob, salt, IMPORTE, fk, delta));
        layer.next_pending += 1;
        layer.pending_amounts.insert(pos, IMPORTE);
        layer
            .pending_meta
            .insert(pos, (crate::REFUND_SENDER_NONE, layer.log.len() as u64));

        let aviso_v1 = PendingNotice { position: pos, salt, amount: IMPORTE, x: None };
        let estado = state_of(&layer, bob);
        let r = layer
            .claim(BaseElement::new(SK_BOB), bob, &estado, &aviso_v1)
            .and_then(|c| layer.apply_claim(&c, bob, &estado, &aviso_v1));
        assert!(r.is_err(), "sin el sobre no hay C2: el cobro debe rebotar");
        assert_eq!(layer.balance_of(bob), Some(0), "nada acreditado");
    }

    /// E3b-2 T1 FELIZ: un pendiente v2 (C2 PLANTADO, molde del S350) se
    /// reembolsa tras `delta` por la via de la APERTURA -- el compromiso
    /// como juez, sin tocar `refund_ttl`.
    #[test]
    fn un_refund_v2_tras_delta_devuelve_al_emisor() {
        use crate::pending::pending_commitment_v2;
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        let receptor = layer.public_id_of(bob).expect("bob");
        let f = layer.public_id_of(alice).expect("alice");
        let ea = state_of(&layer, alice);
        let recibo = layer
            .send(BaseElement::new(SK_ALICE), alice, &ea, receptor, salt_de(0xE3B2), 300_000)
            .expect("send");
        layer.apply_send(&recibo, alice, &ea, 300_000).expect("apply");
        let pos = recibo.notice.position;
        let delta = 1u64;
        layer
            .pending
            .set_leaf(pos, pending_commitment_v2(receptor, salt_de(0xE3B2), 300_000, f, delta));
        layer.pending_meta.insert(pos, (alice, 0));
        let ea2 = state_of(&layer, alice);
        let materiales = layer
            .refund_v2(
                BaseElement::new(SK_ALICE), alice, &ea2, pos,
                receptor, salt_de(0xE3B2), 300_000, f, delta,
            )
            .expect("materiales v2");
        layer.apply_refund(&materiales).expect("refund v2 tras delta");
        assert_eq!(state_of(&layer, alice).balance, 1_000_000);
    }

    /// E3b-2 T2: antes de `delta` el reembolso v2 REBOTA -- y el caso
    /// particular `u64::MAX` es el "nadie nunca" del S119.
    #[test]
    fn antes_de_delta_el_reembolso_v2_rebota() {
        use crate::pending::pending_commitment_v2;
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        let receptor = layer.public_id_of(bob).expect("bob");
        let f = layer.public_id_of(alice).expect("alice");
        let ea = state_of(&layer, alice);
        let recibo = layer
            .send(BaseElement::new(SK_ALICE), alice, &ea, receptor, salt_de(0xE3B2), 300_000)
            .expect("send");
        layer.apply_send(&recibo, alice, &ea, 300_000).expect("apply");
        let pos = recibo.notice.position;
        for delta in [1_000_000u64, u64::MAX] {
            layer
                .pending
                .set_leaf(pos, pending_commitment_v2(receptor, salt_de(0xE3B2), 300_000, f, delta));
            let ea2 = state_of(&layer, alice);
            let materiales = layer
                .refund_v2(
                    BaseElement::new(SK_ALICE), alice, &ea2, pos,
                    receptor, salt_de(0xE3B2), 300_000, f, delta,
                )
                .expect("materiales v2");
            assert!(
                matches!(
                    layer.apply_refund(&materiales),
                    Err(LayerError::RefundTooEarly { .. })
                ),
                "con delta={delta} el reembolso tiene que rebotar"
            );
        }
        assert_eq!(state_of(&layer, alice).balance, 700_000);
    }

    /// E3b-2 T3 -- el test que la RECOMPOSICION estrena: un recibo cuyo
    /// compromiso cuadra con la hoja pero cuya apertura MIENTE sobre
    /// `delta` muere en `M(c1, M(f, d(delta))) != c2`, antes de mirar
    /// ninguna prueba.
    #[test]
    fn un_delta_mentiroso_rebota_en_la_recomposicion() {
        use crate::pending::{pending_commitment, pending_commitment_v2};
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        let receptor = layer.public_id_of(bob).expect("bob");
        let f = layer.public_id_of(alice).expect("alice");
        let ea = state_of(&layer, alice);
        let recibo = layer
            .send(BaseElement::new(SK_ALICE), alice, &ea, receptor, salt_de(0xE3B2), 300_000)
            .expect("send");
        layer.apply_send(&recibo, alice, &ea, 300_000).expect("apply");
        let pos = recibo.notice.position;
        layer
            .pending
            .set_leaf(pos, pending_commitment_v2(receptor, salt_de(0xE3B2), 300_000, f, 1));
        layer.pending_meta.insert(pos, (alice, 0));
        let ea2 = state_of(&layer, alice);
        let mut materiales = layer
            .refund_v2(
                BaseElement::new(SK_ALICE), alice, &ea2, pos,
                receptor, salt_de(0xE3B2), 300_000, f, 1,
            )
            .expect("materiales v2");
        let c1 = pending_commitment(receptor, salt_de(0xE3B2), 300_000);
        materiales.apertura = Some((c1, f, 999));
        assert!(
            matches!(layer.apply_refund(&materiales), Err(LayerError::PendingMismatch)),
            "la apertura mentirosa tiene que morir en la recomposicion"
        );
    }

    /// E3b-2 T4 (calca del ladron con aviso): el sobre NOMBRA el retorno.
    /// Si `f` es un tercero, la cuenta acreditada (el emisor del meta) no
    /// es `f` y la capa rebota: nadie redirige un reembolso en la capa.
    #[test]
    fn el_sobre_no_devuelve_a_un_tercero() {
        use crate::pending::pending_commitment_v2;
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        let mallory = open_and_fund(&mut layer, SK_MALLORY, 0);
        let receptor = layer.public_id_of(bob).expect("bob");
        let id_mallory = layer.public_id_of(mallory).expect("mallory");
        let ea = state_of(&layer, alice);
        let recibo = layer
            .send(BaseElement::new(SK_ALICE), alice, &ea, receptor, salt_de(0xE3B2), 300_000)
            .expect("send");
        layer.apply_send(&recibo, alice, &ea, 300_000).expect("apply");
        let pos = recibo.notice.position;
        layer.pending.set_leaf(
            pos,
            pending_commitment_v2(receptor, salt_de(0xE3B2), 300_000, id_mallory, 1),
        );
        layer.pending_meta.insert(pos, (alice, 0));
        let ea2 = state_of(&layer, alice);
        let materiales = layer
            .refund_v2(
                BaseElement::new(SK_ALICE), alice, &ea2, pos,
                receptor, salt_de(0xE3B2), 300_000, id_mallory, 1,
            )
            .expect("materiales v2");
        assert!(
            matches!(layer.apply_refund(&materiales), Err(LayerError::PendingMismatch)),
            "el credito solo vuelve a quien el sobre nombra"
        );
        assert_eq!(state_of(&layer, alice).balance, 700_000);
    }

    /// GEMELOS D-7 (1/5): al reembolso v2 se le arranca la apertura y la
    /// via v1 lo rechaza ANTES del Air -- la traza ancha del RefundAirV2
    /// no corresponde a RefundAir. Sin la guarda, el assert_eq! del ancho
    /// en fn new abortaria el hilo. El estado no se toca.
    #[test]
    fn un_reembolso_v2_sin_su_apertura_no_valida() {
        use crate::pending::pending_commitment_v2;
        assert_ne!(
            stark_experiment::circuit_refund_v2::TRACE_WIDTH,
            stark_experiment::circuit_refund::TRACE_WIDTH,
            "premisa del testigo: anchos distintos"
        );
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        let receptor = layer.public_id_of(bob).expect("bob");
        let f = layer.public_id_of(alice).expect("alice");
        let ea = state_of(&layer, alice);
        let recibo = layer
            .send(BaseElement::new(SK_ALICE), alice, &ea, receptor, salt_de(0xD7A1), 300_000)
            .expect("send");
        layer.apply_send(&recibo, alice, &ea, 300_000).expect("apply");
        let pos = recibo.notice.position;
        layer
            .pending
            .set_leaf(pos, pending_commitment_v2(receptor, salt_de(0xD7A1), 300_000, f, 1));
        layer.pending_meta.insert(pos, (alice, 0));
        let ea2 = state_of(&layer, alice);
        let mut materiales = layer
            .refund_v2(
                BaseElement::new(SK_ALICE), alice, &ea2, pos,
                receptor, salt_de(0xD7A1), 300_000, f, 1,
            )
            .expect("materiales v2");
        layer.set_refund_ttl(0);
        materiales.apertura = None;
        let r = layer.apply_refund(&materiales);
        assert!(r.is_err(), "sin apertura, la via v1 no acepta una traza v2");
        assert_eq!(state_of(&layer, alice).balance, 700_000, "nada acreditado");
    }

    /// GEMELOS D-7 (2/5): la apertura VERIFICA y aun asi nada muta -- la
    /// ranura del credito tiene su propio juez de geometria y habla antes
    /// de tocar el estado (el orden del doble cerrojo, demostrado).
    #[test]
    fn un_credito_de_ancho_ajeno_no_valida() {
        use crate::pending::pending_commitment_v2;
        assert_ne!(
            stark_experiment::circuit_refund_v2::TRACE_WIDTH,
            stark_experiment::circuit_credit_climb::TRACE_WIDTH,
            "premisa del testigo: anchos distintos"
        );
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        let receptor = layer.public_id_of(bob).expect("bob");
        let f = layer.public_id_of(alice).expect("alice");
        let ea = state_of(&layer, alice);
        let recibo = layer
            .send(BaseElement::new(SK_ALICE), alice, &ea, receptor, salt_de(0xD7A2), 300_000)
            .expect("send");
        layer.apply_send(&recibo, alice, &ea, 300_000).expect("apply");
        let pos = recibo.notice.position;
        layer
            .pending
            .set_leaf(pos, pending_commitment_v2(receptor, salt_de(0xD7A2), 300_000, f, 1));
        layer.pending_meta.insert(pos, (alice, 0));
        let ea2 = state_of(&layer, alice);
        let mut materiales = layer
            .refund_v2(
                BaseElement::new(SK_ALICE), alice, &ea2, pos,
                receptor, salt_de(0xD7A2), 300_000, f, 1,
            )
            .expect("materiales v2");
        materiales.credit_proof = materiales.refund_proof.clone();
        let r = layer.apply_refund(&materiales);
        assert!(r.is_err(), "el credito de ancho ajeno rebota con Err");
        assert_eq!(
            state_of(&layer, alice).balance,
            700_000,
            "la apertura verifico y aun asi nada muta"
        );
    }

    /// GEMELOS D-7 (3/5): una desemision cuyo recibo no lleva apertura
    /// pero cuya prueba es de traza v2 rebota en la guarda (via v1 =
    /// RefundAir). El recibo se construye a mano sobre el pendiente
    /// re-plantado con el centinela de emision. Nada se destruye.
    #[test]
    fn una_desemision_de_ancho_ajeno_no_valida() {
        use crate::pending::pending_commitment_v2;
        assert_ne!(
            stark_experiment::circuit_refund_v2::TRACE_WIDTH,
            stark_experiment::circuit_refund::TRACE_WIDTH,
            "premisa del testigo: anchos distintos"
        );
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        let receptor = layer.public_id_of(bob).expect("bob");
        let f = layer.public_id_of(alice).expect("alice");
        let ea = state_of(&layer, alice);
        let recibo = layer
            .send(BaseElement::new(SK_ALICE), alice, &ea, receptor, salt_de(0xD7A3), 300_000)
            .expect("send");
        layer.apply_send(&recibo, alice, &ea, 300_000).expect("apply");
        let pos = recibo.notice.position;
        let c2 = pending_commitment_v2(receptor, salt_de(0xD7A3), 300_000, f, 1);
        layer.pending.set_leaf(pos, c2);
        layer.pending_meta.insert(pos, (alice, 0));
        let ea2 = state_of(&layer, alice);
        let materiales = layer
            .refund_v2(
                BaseElement::new(SK_ALICE), alice, &ea2, pos,
                receptor, salt_de(0xD7A3), 300_000, f, 1,
            )
            .expect("materiales v2");
        // El mismo pendiente pasa a EMISION: solo el centinela cambia.
        layer.pending_meta.insert(pos, (crate::REFUND_SENDER_NONE, 0));
        layer.set_refund_ttl(0);
        let d = DeissueReceipt {
            refund_proof: materiales.refund_proof.clone(),
            position: pos,
            amount: 300_000,
            commitment: c2,
            apertura: None,
        };
        let supply = layer.total_supply();
        let r = layer.apply_deissue(&d);
        assert!(r.is_err(), "sin apertura, la via v1 no acepta una traza v2");
        assert_eq!(layer.total_supply(), supply, "nada destruido");
        assert_eq!(layer.pending.leaf(pos), c2, "la hoja sigue");
    }

    /// DEUDA DEL S351, SALDADA (S357): el testigo FUNCIONAL de la rama
    /// `Some` de `apply_deissue`. Un pendiente de EMISION (centinela) con
    /// compromiso v2 (C2 PLANTADO, molde del S350) se des-emite tras
    /// `delta` por la via de la APERTURA: la hoja queda vacia, el
    /// suministro BAJA exactamente lo comprometido y NADIE cobra -- la
    /// raiz de cuentas y el saldo del emisor no se mueven (destruir no
    /// es devolver: el contraste exacto con el reembolso).
    #[test]
    fn una_desemision_v2_tras_delta_destruye_el_pendiente() {
        use crate::pending::pending_commitment_v2;
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        let receptor = layer.public_id_of(bob).expect("bob");
        let f = layer.public_id_of(alice).expect("alice");
        let ea = state_of(&layer, alice);
        let recibo = layer
            .send(BaseElement::new(SK_ALICE), alice, &ea, receptor, salt_de(0xDE15), 300_000)
            .expect("send");
        layer.apply_send(&recibo, alice, &ea, 300_000).expect("apply");
        let pos = recibo.notice.position;
        let delta = 1u64;
        let c2 = pending_commitment_v2(receptor, salt_de(0xDE15), 300_000, f, delta);
        layer.pending.set_leaf(pos, c2);
        // El pendiente pasa a EMISION: el centinela es la llave del deissue.
        layer.pending_meta.insert(pos, (crate::REFUND_SENDER_NONE, 0));
        let saldo_emisor = state_of(&layer, alice).balance;
        let supply = layer.total_supply();
        let raiz = layer.accounts.root();
        let materiales = layer
            .deissue_v2(pos, receptor, salt_de(0xDE15), 300_000, f, delta)
            .expect("materiales deissue v2");
        layer.apply_deissue(&materiales).expect("desemision v2 tras delta");
        assert_eq!(layer.pending.leaf(pos), [BaseElement::ZERO; 4], "la hoja queda vacia");
        assert_eq!(layer.total_supply(), supply - 300_000, "el suministro baja lo comprometido");
        assert_eq!(state_of(&layer, alice).balance, saldo_emisor, "nadie cobra");
        assert_eq!(layer.accounts.root(), raiz, "la raiz de cuentas no se mueve");
        assert!(layer.pending_meta.get(&pos).is_none(), "el meta se va con la hoja");
    }

    /// GEMELOS D-7 (4/5): a un cobro v2 se le presenta el aviso sin sobre
    /// y la via v1 lo rechaza ANTES del Air -- la traza del ClaimAirV2 no
    /// corresponde a ClaimAir. El pendiente sigue en el arbol.
    #[test]
    fn un_cobro_v2_sin_su_sobre_no_valida() {
        use crate::pending::{pending_commitment_v2, refund_envelope};
        assert_ne!(
            stark_experiment::circuit_claim_v2::TRACE_WIDTH,
            stark_experiment::circuit_claim::TRACE_WIDTH,
            "premisa del testigo: anchos distintos"
        );
        let mut layer = new_layer();
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        let id_bob = layer.public_id_of(bob).expect("cuenta");
        let (salt, fk, delta) = (salt_de(0xD7A4), salt_de(0xF00D), 40u64);

        let pos = layer.next_pending;
        let c2 = pending_commitment_v2(id_bob, salt, IMPORTE, fk, delta);
        layer.pending.set_leaf(pos, c2);
        layer.next_pending += 1;
        layer.pending_amounts.insert(pos, IMPORTE);
        layer
            .pending_meta
            .insert(pos, (crate::REFUND_SENDER_NONE, layer.log.len() as u64));

        let aviso = PendingNotice {
            position: pos,
            salt,
            amount: IMPORTE,
            x: Some(refund_envelope(fk, delta)),
        };
        let estado = state_of(&layer, bob);
        let cobro = layer
            .claim(BaseElement::new(SK_BOB), bob, &estado, &aviso)
            .expect("el productor v2 debe probar");
        let aviso_sin = PendingNotice {
            position: pos,
            salt,
            amount: IMPORTE,
            x: None,
        };
        let r = layer.apply_claim(&cobro, bob, &estado, &aviso_sin);
        assert!(r.is_err(), "sin el sobre, la via v1 no acepta una traza v2");
        assert_eq!(layer.balance_of(bob), Some(0), "nada cobrado");
        assert_eq!(layer.pending.leaf(pos), c2, "el pendiente sigue");
    }
    // ------------------------------------------------------------------
    // RFC-0006, E1 (§413): la SEXTA raiz en reposo, `root:cons`, calcada de
    // la quinta (§391). Tres testigos negativos, dos positivos.
    // ------------------------------------------------------------------

    fn consumo_de(n: u64) -> Digest {
        [
            BaseElement::new(n),
            BaseElement::new(7),
            BaseElement::new(11),
            BaseElement::new(13),
        ]
    }

    /// Abre un libro en `path`, publica un consumo y lo devuelve.
    fn ledger_con_un_consumo(path: &str) -> Digest {
        let mut layer = open_retry(
            path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
        )
        .expect("abrir");
        let c = consumo_de(41);
        layer.apply_consumo(c).expect("publicar");
        assert!(layer.is_consumido(&c));
        c
    }

    /// §413: un consumo sobrevive al reinicio y sigue rechazandose repetido.
    #[test]
    fn rfc0006_un_consumo_sobrevive_el_reinicio() {
        let path = ruta_temporal("consumo_reinicio");
        let c = ledger_con_un_consumo(&path);
        let mut layer = open_retry(
            &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
        )
        .expect("reabrir");
        assert!(layer.is_consumido(&c), "CRITICO: el consumo debe sobrevivir al reinicio");
        assert_eq!(layer.cons_count(), 1);
        assert!(
            matches!(layer.apply_consumo(c), Err(LayerError::ConsumoRepetido { .. })),
            "tras reiniciar, el repetido se sigue rechazando"
        );
        let _ = std::fs::remove_dir_all(&path);
    }

    /// §413: un libro sin `root:cons` (anterior a este sello) NO ABRE: fail-closed,
    /// sin era silenciosa, como `root:froz` (§391) y `root:log` (§392).
    #[test]
    fn rfc0006_un_libro_sin_root_cons_no_abre() {
        let path = ruta_temporal("sin_root_cons");
        let _c = ledger_con_un_consumo(&path);
        {
            let db = sled_open_retry(&path);
            assert!(
                db.remove(b"root:cons").expect("borrar").is_some(),
                "habia una raiz de consumos que borrar"
            );
            db.flush().expect("flush");
        }
        let r = open_retry(
            &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
        );
        assert!(
            matches!(
                r,
                Err(LayerError::Store(crate::store::StoreError::Malformed(ref m))) if m.contains("root:cons")
            ),
            "CRITICO: un libro sin raiz de consumos no debe abrir"
        );
        let _ = std::fs::remove_dir_all(&path);
    }

    /// §413: un consumo BORRADO en reposo no abre: la raiz guardada no cuadra.
    #[test]
    fn rfc0006_un_consumo_borrado_en_reposo_no_abre() {
        let path = ruta_temporal("consumo_borrado");
        let c = ledger_con_un_consumo(&path);
        {
            let db = sled_open_retry(&path);
            let mut key = b"cons:".to_vec();
            key.extend_from_slice(&crate::consumo::posicion_de_consumo(&c).to_le_bytes());
            assert!(db.remove(key).expect("borrar").is_some(), "habia un consumo que borrar");
            db.flush().expect("flush");
        }
        let r = open_retry(
            &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
        );
        assert!(
            matches!(
                r,
                Err(LayerError::Store(crate::store::StoreError::IntegrityFailure { what })) if what == "arbol de consumos"
            ),
            "CRITICO: borrar un consumo en disco debe impedir abrir"
        );
        let _ = std::fs::remove_dir_all(&path);
    }

    /// §413: un consumo ANADIDO en reposo, sin pasar por `apply_consumo`, no abre.
    #[test]
    fn rfc0006_un_consumo_anadido_en_reposo_no_abre() {
        let path = ruta_temporal("consumo_anadido");
        let _c = ledger_con_un_consumo(&path);
        let intruso = consumo_de(42);
        {
            let db = sled_open_retry(&path);
            let mut key = b"cons:".to_vec();
            key.extend_from_slice(&crate::consumo::posicion_de_consumo(&intruso).to_le_bytes());
            assert!(
                db.insert(key, crate::store::digest_to_bytes(&intruso).to_vec()).expect("anadir").is_none(),
                "la posicion del intruso estaba libre"
            );
            db.flush().expect("flush");
        }
        let r = open_retry(
            &path, custodian_root(), governance_root(), LIMIT, MAX_SUPPLY, MAX_ACCOUNTS,
        );
        assert!(
            matches!(
                r,
                Err(LayerError::Store(crate::store::StoreError::IntegrityFailure { what })) if what == "arbol de consumos"
            ),
            "CRITICO: anadir un consumo en disco debe impedir abrir"
        );
        let _ = std::fs::remove_dir_all(&path);
    }

    /// §413: la instantanea (v8) conserva los consumos: restaurar una copia
    /// no los pierde y la raiz coincide.
    #[test]
    fn rfc0006_una_instantanea_conserva_los_consumos() {
        let mut layer = new_layer();
        let c = consumo_de(43);
        layer.apply_consumo(c).expect("publicar");
        let file = format!("{}.snap", ruta_temporal("instantanea_consumos"));
        layer.export_snapshot(&file).expect("exportar");
        let restaurada = SovereignLayer::import_snapshot(&file).expect("importar");
        assert!(
            restaurada.is_consumido(&c),
            "CRITICO: restaurar una copia no debe perder los consumos"
        );
        assert_eq!(restaurada.cons_root(), layer.cons_root());
        assert_eq!(restaurada.cons_count(), 1);
        let _ = std::fs::remove_file(&file);
    }
}
