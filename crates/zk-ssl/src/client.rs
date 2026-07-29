//! **Generación de pruebas en el cliente.** La clave de gasto nunca sale
//! de la máquina del titular.
//!
//! ## El problema que corrige
//!
//! `SovereignLayer::transfer` recibe la clave de gasto. Es decir: **para
//! transferir había que entregársela a quien opera el nodo**, y con ella
//! puede vaciar la cuenta cuando quiera.
//!
//! Eso no era una limitación de escala. Era que el sistema exigía
//! **confiar tu dinero al operador**, precisamente el intermediario que
//! el proyecto dice eliminar.
//!
//! ## El protocolo
//!
//! ```text
//! 1. Cliente: pide la vista de su cuenta        → nonce, saldo, identidad
//! 2. Cliente: calcula el nullifier LOCALMENTE   (necesita su clave)
//! 3. Cliente: pide los materiales               → caminos de Merkle
//! 4. Cliente: genera la prueba EN SU MÁQUINA    (la clave no sale)
//! 5. Cliente: envía la liquidación              → la capa verifica y aplica
//! ```
//!
//! El nullifier viaja a la capa en el paso 3, pero **no revela nada
//! nuevo**: es público y aparecería igualmente al aplicar la liquidación.
//!
//! ## Lo que esto NO resuelve
//!
//! **Generar la prueba cuesta ~600 ms y bastante memoria.** Si el cliente
//! es un dispositivo ligero y quiere que otro la genere por él, hace
//! falta que ese otro pueda probar **sin** la clave — lo que exige
//! verificar una firma dentro del circuito (Winternitz, ~8.000 filas
//! adicionales).
//!
//! Eso es una **optimización para clientes ligeros**, no una corrección
//! de seguridad. La custodia queda resuelta aquí.
//!
//! ## Y lo que sigue viendo el operador
//!
//! Los saldos. La capa mantiene el estado, así que los conoce. Esto
//! elimina que vea **claves**, no que vea **datos**. Lo segundo requiere
//! descentralización.

use super::*;
// `two_phase` es un modulo publico, pero sus tipos no estan en la raiz del
// crate: `use super::*` no los alcanza.
use crate::pending::pending_commitment;
use crate::two_phase::{ClaimReceipt, PendingNotice, SendReceipt};

/// Vista pública de una cuenta. **No incluye ninguna clave.**
///
/// El saldo aparece aquí porque el operador del nodo lo conoce de todos
/// modos —mantiene el estado—, así que exponerlo al titular no filtra
/// nada nuevo.
#[derive(Clone, Debug)]
pub struct AccountView {
    pub public_id: Digest,
    pub balance: u64,
    pub nonce: BaseElement,
}

/// Error al comprobar el destinatario de una transferencia.
#[derive(Debug, PartialEq, Eq)]
pub struct WrongRecipient {
    pub expected: Digest,
    pub found: Digest,
}

impl std::fmt::Display for WrongRecipient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "el destinatario de estos materiales NO es quien esperabas: \
             la capa devolvio otra cuenta"
        )
    }
}
impl std::error::Error for WrongRecipient {}



impl SovereignLayer {
    /// Vista pública de una cuenta, para que el cliente pueda calcular su
    /// nullifier antes de pedir materiales.
    pub fn account_view(&self, index: AccountIndex) -> Option<AccountView> {
        self.records.get(&index).map(|r| AccountView {
            public_id: r.public_id,
            balance: r.balance,
            nonce: r.nonce,
        })
    }

    /// Entrega los materiales para que el cliente genere la prueba.
    ///
    /// **No recibe la clave de gasto.** El `nullifier` lo calcula el
    /// cliente con su clave y lo envía porque la capa necesita su
    /// posición para dar el camino de no-pertenencia — y porque es
    /// público de todos modos.
    /// **Materiales para un envío, sin la clave y sin el saldo del receptor.**
    ///
    /// El equivalente, para la vía en dos fases, de los materiales que
    /// fases, con dos diferencias que son el diseño entero:
    ///
    /// | | Un paso | Envío |
    /// |---|---|---|
    /// | Del receptor entrega | **Su saldo y su camino** | **Solo su identificador** |
    /// | Reserva | Un nullificador | Una posición de pendiente |
    ///
    /// Todo lo que devuelve es público o derivable, así que la capa puede
    /// entregarlo sin conocer la clave de gasto y el cliente puede probar
    /// con [`prove_send`] sin volver a hablar con ella.
    /// **Materiales para cobrar, sin la clave.**
    ///
    /// El aviso lo aporta quien cobra: la capa no sabe qué pendiente es suyo
    /// —esa es la privacidad del diseño— así que **no puede entregarlo**.
    pub fn claim_materials(
        &self,
        receiver_index: AccountIndex,
        notice: &PendingNotice,
    ) -> Result<ClaimMaterials, LayerError> {
        let receiver = self
            .account_view(receiver_index)
            .ok_or(LayerError::AccountNotFound(receiver_index))?;

        // ⚠️ Una cuenta congelada no puede cobrar, y el dinero queda en el
        // limbo. Es una inversion de la decision original, documentada en
        // `AUDITORIA.md` §29.
        if self.is_frozen(receiver_index) {
            return Err(LayerError::AccountFrozen(receiver_index));
        }

        Ok(ClaimMaterials {
            receiver_path: self.accounts.path_for(receiver_index),
            frozen_path: self.frozen.path_for(receiver_index),
            pending_path: self.pending.path_for(notice.position),
            receiver,
            total_supply: self.total_supply,
            notice: notice.clone(),
        })
    }

    pub fn send_materials(
        &self,
        sender_index: AccountIndex,
        receiver_id: Digest,
        amount: u64,
        salt: Digest,
    ) -> Result<SendMaterials, LayerError> {
        let sender = self
            .account_view(sender_index)
            .ok_or(LayerError::AccountNotFound(sender_index))?;

        if self.is_frozen(sender_index) {
            return Err(LayerError::AccountFrozen(sender_index));
        }
        if amount > sender.balance {
            return Err(LayerError::InsufficientBalance {
                available: sender.balance,
                requested: amount,
            });
        }
        // ⚠️ El limite se comprueba aqui **y** lo prueba el circuito: la capa
        // aporta el suyo y `circuit_send` demuestra `importe <= limite`. Ver
        // `AUDITORIA.md` §25.
        if amount > self.regulatory_limit {
            return Err(LayerError::OverRegulatoryLimit {
                limit: self.regulatory_limit,
                requested: amount,
            });
        }

        let pending_position = self.allocate_pending()?;

        Ok(SendMaterials {
            sender_path: self.accounts.path_for(sender_index),
            frozen_path: self.frozen.path_for(sender_index),
            pending_path: self.pending.path_for(pending_position),
            pending_position,
            sender,
            receiver_id,
            regulatory_limit: self.regulatory_limit,
            total_supply: self.total_supply,
            amount,
            salt,
        })
    }

}

/// **Calcula el nullifier de una operación.** Se ejecuta en el cliente.
///
/// Requiere la clave, y por eso solo el titular puede calcularlo — que es
/// lo que impide a un observador precomputar los nullifiers de cuentas
/// ajenas y vigilar cuándo gastan.
/// **Materiales para un ENVÍO en dos fases.**
///
/// La diferencia con la vía de un paso —ya retirada— es la que da nombre al
/// diseño: **aquí no hay saldo del receptor**.
///
/// Aquella vía actualizaba las dos hojas en una transición, así que quien
/// probaba necesitaba el saldo del receptor para calcular su hoja nueva, y
/// sus materiales se lo entregaban. **Pagar a alguien revelaba cuánto
/// tiene.**
///
/// Un envío toca **una sola hoja**, la del pagador. Del receptor basta su
/// identificador público, que es lo que va en el compromiso. Ver
/// `AUDITORIA.md` §29 y el hallazgo 9 del preprint comparativo.
#[derive(Clone, Debug)]
pub struct SendMaterials {
    pub sender: AccountView,
    pub sender_path: MerklePath,
    pub frozen_path: MerklePath,
    pub pending_path: MerklePath,
    /// Posición libre del árbol de pendientes que la capa ha reservado.
    pub pending_position: u64,
    /// **Solo el identificador.** No el saldo, no la posición en el árbol.
    pub receiver_id: Digest,
    pub regulatory_limit: u64,
    pub total_supply: u64,
    pub amount: u64,
    pub salt: Digest,
}

impl SendMaterials {
    /// **Comprueba a quién van dirigidos estos materiales.**
    ///
    /// La capa entrega los materiales que se le piden. Si alguien
    /// interceptara la petición y cambiara el destinatario, el pagador
    /// firmaría un envío a otra cuenta sin notarlo.
    ///
    /// ⚠️ **Aquí es más simple que en la vía de un paso.** Allí había que
    /// comparar contra `receiver.public_id` —un campo de una vista que
    /// también traía el saldo—. Aquí el identificador **es** el único dato
    /// del receptor que existe.
    pub fn check_recipient(&self, expected: Digest) -> Result<(), WrongRecipient> {
        if self.receiver_id != expected {
            return Err(WrongRecipient {
                expected,
                found: self.receiver_id,
            });
        }
        Ok(())
    }
}

/// **Genera la prueba de un envío SIN tocar la capa.**
///
/// Es el equivalente, para la vía en dos fases, de la prueba local que
/// por la misma razón: demostrar que **la clave de gasto no necesita salir de
/// la máquina del cliente**.
///
/// `SovereignLayer::send` hace lo mismo, pero es un método de la capa que
/// recibe la clave. Esa forma no impide la separación —el cliente puede
/// ejecutar la capa en su máquina— pero **tampoco la enseña**. Ver
/// `AUDITORIA.md` §33.
pub fn prove_send(
    materials: &SendMaterials,
    spend_key: BaseElement,
    options: ProofOptions,
) -> Result<SendReceipt, LayerError> {
    // La clave debe corresponder a la cuenta. El circuito lo impone
    // igualmente, pero en release no se valida al generar: sin esta
    // comprobacion se gastaria el computo de una prueba invalida.
    if derive_public_id(spend_key) != materials.sender.public_id {
        return Err(LayerError::NotTheAccountHolder);
    }

    let trace = build_send_trace(
        spend_key,
        materials.sender.public_id,
        materials.sender.balance,
        materials.sender.nonce,
        &materials.sender_path,
        &materials.frozen_path,
        materials.amount,
        materials.regulatory_limit,
        materials.total_supply,
        0, // un envio no cambia el suministro
        materials.receiver_id,
        materials.salt,
        &materials.pending_path,
    );
    let prover = SendProver::new(options);
    let public_inputs = prover.get_pub_inputs(&trace);
    let proof = prover
        .prove(trace)
        .map_err(|e| LayerError::ProofFailed(format!("{e:?}")))?;

    Ok(SendReceipt {
        proof: proof.to_bytes(),
        public_inputs,
        commitment: pending_commitment(
            materials.receiver_id,
            materials.salt,
            materials.amount,
        ),
        notice: PendingNotice {
            position: materials.pending_position,
            salt: materials.salt,
            amount: materials.amount,
        },
    })
}

/// **Materiales para COBRAR un pendiente.**
///
/// La pieza que faltaba para que un pago entero se pueda probar sin dar la
/// clave a la capa. Ver `AUDITORIA.md` §33.
///
/// ⚠️ **No tiene precedente en la vía de un paso**, donde recibir era pasivo:
/// el pagador actualizaba las dos hojas y el receptor no hacía nada. Aquí
/// cobrar es una operación del receptor, con su propia prueba.
#[derive(Clone, Debug)]
pub struct ClaimMaterials {
    pub receiver: AccountView,
    pub receiver_path: MerklePath,
    pub frozen_path: MerklePath,
    pub pending_path: MerklePath,
    pub total_supply: u64,
    /// El aviso que el pagador tuvo que hacerle llegar.
    ///
    /// ⚠️ **ISO 20022 no lo transporta.** Cómo viaja del pagador al receptor
    /// sigue sin resolver; ver `AUDITORIA.md` §21 y el §3.5 de la nota de
    /// política.
    pub notice: PendingNotice,
}

/// **Genera la prueba de un cobro SIN tocar la capa.**
///
/// Con esto y [`prove_send`], **un pago completo se prueba en el cliente**:
/// la capa entrega caminos y raíces, y verifica; la clave de gasto no sale
/// de la máquina de quien paga ni de la de quien cobra.
pub fn prove_claim(
    materials: &ClaimMaterials,
    spend_key: BaseElement,
    options: ProofOptions,
) -> Result<ClaimReceipt, LayerError> {
    if derive_public_id(spend_key) != materials.receiver.public_id {
        return Err(LayerError::NotTheAccountHolder);
    }

    let trace = build_claim_trace(
        spend_key,
        materials.receiver.public_id,
        materials.receiver.balance,
        materials.receiver.nonce,
        &materials.receiver_path,
        &materials.frozen_path,
        materials.notice.amount,
        materials.total_supply,
        0,
        // El destinatario del compromiso es el propio receptor: cobrar es
        // demostrar que el pendiente estaba a su nombre.
        materials.receiver.public_id,
        materials.notice.salt,
        &materials.pending_path,
    );
    let prover = ClaimProver::new(options);
    let public_inputs = prover.get_pub_inputs(&trace);
    let proof = prover
        .prove(trace)
        .map_err(|e| LayerError::ProofFailed(format!("{e:?}")))?;

    Ok(ClaimReceipt {
        proof: proof.to_bytes(),
        public_inputs,
    })
}



#[cfg(test)]
mod tests {
    use crate::tests_support::*;
    use crate::*;
    use winterfell::math::fields::f64::BaseElement;


    /// **Los materiales no contienen ninguna clave.**
    ///
    /// Es la propiedad que define la pieza: lo que viaja de la capa al
    /// cliente es estado, no secretos.
    #[test]
    fn send_materials_contain_no_keys() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        let key = BaseElement::new(SK_ALICE);
        let receptor = layer.public_id_of(bob).expect("cuenta");

        let m = layer
            .send_materials(alice, receptor, 1000, salt_de(0x0C1A))
            .expect("materiales");

        assert_ne!(
            m.sender.public_id,
            [key, BaseElement::ZERO, BaseElement::ZERO, BaseElement::ZERO],
            "los materiales no deben contener la clave"
        );
        assert_eq!(m.sender.public_id, derive_public_id(key));

        // ⚠️ **Y tampoco el saldo del receptor.**
        //
        // Los materiales de la via retirada llevaban una vista completa del
        // receptor, asi que quien pagaba veia cuanto tenia el otro. Aqui el
        // tipo solo tiene
        // `receiver_id: Digest`: **no hay campo por donde el saldo pudiera
        // entrar**. Ver `AUDITORIA.md` §29.
        assert_eq!(m.receiver_id, receptor);
    }

    /// **Sin la clave correcta no se puede generar la prueba**, aunque se
    /// tengan todos los materiales.
    ///
    /// Es lo que impide que quien intercepte los materiales pueda gastar.
    #[test]
    fn send_materials_alone_are_not_enough_to_spend() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        let receptor = layer.public_id_of(bob).expect("cuenta");

        let m = layer
            .send_materials(alice, receptor, 250_000, salt_de(0x1073))
            .expect("materiales");

        let r = client::prove_send(&m, BaseElement::new(0x1337), proof_options());
        assert!(
            matches!(r, Err(LayerError::NotTheAccountHolder)),
            "CRITICO: quien intercepte los materiales NO debe poder gastar. \
             Resultado: {:?}",
            r.map(|_| "recibo")
        );
    }


    /// **UN PAGO ENTERO SIN DAR NINGUNA CLAVE A LA CAPA.**
    ///
    /// El equivalente de `a_transfer_without_giving_the_key_to_the_layer`
    /// para la vía en dos fases, y la razón de que `send_materials` y
    /// `prove_send` existan.
    ///
    /// `SovereignLayer::send` hace lo mismo en una llamada, pero **recibe la
    /// clave como argumento de un método de la capa**. Eso no impide la
    /// separación —el cliente puede ejecutar la capa en su máquina— pero
    /// tampoco la enseña, y los tres preprints citan esta propiedad como el
    /// argumento institucional central. Ver `AUDITORIA.md` §33.
    #[test]
    fn a_whole_payment_without_giving_any_key_to_the_layer() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        let key = BaseElement::new(SK_ALICE);
        let receptor = layer.public_id_of(bob).expect("cuenta");
        let salt = salt_de(0xC11E);

        // ===== 1. LA CAPA ENTREGA MATERIALES. NO VE LA CLAVE. =====
        let materials = layer
            .send_materials(alice, receptor, 250_000, salt)
            .expect("materiales");

        // ⚠️ **Y NO ENTREGA EL SALDO DEL RECEPTOR.**
        //
        // Los materiales de la vía retirada llevaban una vista completa del
        // receptor, porque esa vía actualizaba las dos hojas y necesitaba el
        // saldo del otro.
        // `SendMaterials` lleva `receiver_id: Digest` y nada más: **la fuga
        // hacia la contraparte está cerrada en el tipo**, no en un comentario.
        assert_eq!(materials.receiver_id, receptor);

        // ===== 2. EL CLIENTE PRUEBA EN LOCAL, CON SU CLAVE. =====
        let recibo =
            client::prove_send(&materials, key, proof_options()).expect("prueba local");

        // ===== 3. LA CAPA VERIFICA Y APLICA. =====
        let estado = state_of(&layer, alice);
        layer
            .apply_send(&recibo, alice, &estado, 250_000)
            .expect("aplicar");

        assert_eq!(layer.balance_of(alice), Some(750_000), "el dinero salio");
        assert_eq!(layer.total_pending(), 250_000, "y esta en un pendiente");

        // ===== 4. EL RECEPTOR COBRA, TAMBIEN SIN DAR SU CLAVE. =====
        //
        // ⚠️ **El aviso lo aporta el, no la capa.** La capa no sabe que
        // pendiente es suyo —esa es la privacidad del diseno— asi que no
        // podria entregarselo. Como le llega es la pieza que ISO 20022 no
        // transporta; ver `AUDITORIA.md` §21.
        let mat_cobro = layer
            .claim_materials(bob, &recibo.notice)
            .expect("materiales de cobro");
        let cobro = client::prove_claim(
            &mat_cobro,
            BaseElement::new(SK_BOB),
            proof_options(),
        )
        .expect("prueba local del cobro");

        let estado_bob = state_of(&layer, bob);
        layer
            .apply_claim(&cobro, bob, &estado_bob, &recibo.notice)
            .expect("aplicar cobro");

        assert_eq!(layer.balance_of(bob), Some(250_000));
        assert_eq!(layer.total_pending(), 0);

        // **UN PAGO ENTERO, Y LA CAPA NO HA VISTO NINGUNA CLAVE.**
        //
        // Ni la del pagador ni la del receptor. Lo unico que la capa aporta
        // son caminos y raices —datos publicos— y lo unico que recibe son
        // pruebas que verifica.
    }

    /// **La clave equivocada no genera prueba, y falla ANTES de gastar cómputo.**
    #[test]
    fn prove_send_rejects_a_key_that_is_not_the_holders() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        let receptor = layer.public_id_of(bob).expect("cuenta");

        let materials = layer
            .send_materials(alice, receptor, 1000, salt_de(0xBAD1))
            .expect("materiales");

        let r = client::prove_send(&materials, BaseElement::new(0x1337), proof_options());
        assert!(
            matches!(r, Err(LayerError::NotTheAccountHolder)),
            "el circuito lo impondria igual, pero en release no se valida al \
             generar: sin esta comprobacion se gastaria el computo de una \
             prueba invalida. Salio: {:?}",
            r.map(|_| "recibo")
        );
    }
}
