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

/// Todo lo que el cliente necesita de la capa para generar la prueba
/// **sin entregarle su clave**.
///
/// Son caminos de Merkle y datos de cuenta: información de estado, no
/// secretos.
#[derive(Clone, Debug)]
pub struct TransferMaterials {
    pub sender: AccountView,
    pub receiver: AccountView,
    pub sender_path: MerklePath,
    /// Camino del receptor en el árbol INTERMEDIO, tras actualizar al
    /// emisor. Es lo que exige el encadenamiento de la partida doble.
    pub receiver_path: MerklePath,
    /// Camino en el árbol de nullifiers. Es un `MerklePath` como los
    /// demás: en este backend ambos árboles comparten estructura, solo
    /// difiere qué se coloca en la hoja.
    pub null_path: MerklePath,
    /// Camino de no-pertenencia al árbol de congelados. Sin él, el
    /// cliente no podría demostrar que no está congelado.
    pub frozen_path: MerklePath,
    pub regulatory_limit: u64,
    pub amount: u64,
}

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
    pub fn transfer_materials(
        &self,
        sender_index: AccountIndex,
        receiver_index: AccountIndex,
        amount: u64,
        nullifier: Digest,
    ) -> Result<TransferMaterials, LayerError> {
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

        let null_pos = nullifier_position(&nullifier);
        if self.is_frozen(sender_index) {
            return Err(LayerError::AccountFrozen(sender_index));
        }
        if self.nullifiers.is_occupied(null_pos) {
            return Err(LayerError::NullifierAlreadySpent);
        }

        let sender_path = self.accounts.path_for(sender_index);

        // Arbol INTERMEDIO: solo el emisor actualizado.
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

        Ok(TransferMaterials {
            sender: AccountView {
                public_id: sender.public_id,
                balance: sender.balance,
                nonce: sender.nonce,
            },
            receiver: AccountView {
                public_id: receiver.public_id,
                balance: receiver.balance,
                nonce: receiver.nonce,
            },
            sender_path,
            receiver_path,
            null_path,
            frozen_path: self.frozen.path_for(sender_index),
            regulatory_limit: self.regulatory_limit,
            amount,
        })
    }
}

/// **Calcula el nullifier de una operación.** Se ejecuta en el cliente.
///
/// Requiere la clave, y por eso solo el titular puede calcularlo — que es
/// lo que impide a un observador precomputar los nullifiers de cuentas
/// ajenas y vigilar cuándo gastan.
pub fn compute_nullifier(spend_key: BaseElement, nonce: BaseElement) -> Digest {
    stark_experiment::circuit_settlement::native_nullifier(spend_key, nonce)
}

/// **Genera la prueba de una transferencia EN LA MÁQUINA DEL CLIENTE.**
///
/// Es una función libre, no un método de la capa, y eso es deliberado:
/// **la capa no puede llamarla porque no tiene la clave**. Si fuera un
/// método, la API estaría sugiriendo lo contrario.
pub fn prove_transfer(
    materials: &TransferMaterials,
    spend_key: BaseElement,
) -> Result<Settlement, LayerError> {
    // La clave debe corresponder a la cuenta. El circuito lo impone
    // igualmente, pero en release no se valida al generar: sin esta
    // comprobacion se gastaria el computo de una prueba invalida.
    if derive_public_id(spend_key) != materials.sender.public_id {
        return Err(LayerError::NotTheAccountHolder);
    }

    let trace = build_settlement_trace(
        &SenderWitness {
            spend_key,
            balance: materials.sender.balance,
            nonce: materials.sender.nonce,
            path: materials.sender_path.clone(),
        },
        &ReceiverWitness {
            public_id: materials.receiver.public_id,
            balance: materials.receiver.balance,
            nonce: materials.receiver.nonce,
            path: materials.receiver_path.clone(),
        },
        materials.amount,
        materials.amount,
        materials.regulatory_limit,
        &materials.null_path,
        &materials.frozen_path,
    );

    let prover = SettlementProver::new(proof_options());
    let public_inputs = prover.get_pub_inputs(&trace);
    let proof = prover
        .prove(trace)
        .map_err(|e| LayerError::ProofFailed(format!("{e:?}")))?;

    Ok(Settlement {
        proof: proof.to_bytes(),
        public_inputs,
    })
}

#[cfg(test)]
mod tests {
    use crate::tests_support::*;
    use crate::*;
    use winterfell::math::fields::f64::BaseElement;

    /// **EL TEST QUE JUSTIFICA LA PIEZA.**
    ///
    /// El ciclo completo sin que la clave llegue nunca a la capa.
    #[test]
    fn a_transfer_without_giving_the_key_to_the_layer() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        let key = BaseElement::new(SK_ALICE);

        // 1. El cliente pide la vista de su cuenta.
        let view = layer.account_view(alice).expect("vista");

        // 2. Calcula el nullifier LOCALMENTE, con su clave.
        let nullifier = client::compute_nullifier(key, view.nonce);

        // 3. Pide los materiales. La capa NO recibe la clave.
        let materials = layer
            .transfer_materials(alice, bob, 250_000, nullifier)
            .expect("materiales");

        // 4. Genera la prueba EN SU MAQUINA.
        let settlement = client::prove_transfer(&materials, key).expect("prueba");

        // 5. La capa verifica y aplica.
        layer
            .apply(&settlement, alice, bob, 250_000)
            .expect("aplicar");

        assert_eq!(layer.balance_of(alice), Some(750_000));
        assert_eq!(layer.balance_of(bob), Some(250_000));
    }

    /// **Los materiales no contienen ninguna clave.**
    ///
    /// Es la propiedad que define la pieza: lo que viaja de la capa al
    /// cliente es estado, no secretos.
    #[test]
    fn materials_contain_no_keys() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        let key = BaseElement::new(SK_ALICE);
        let view = layer.account_view(alice).expect("vista");
        let nullifier = client::compute_nullifier(key, view.nonce);

        let m = layer
            .transfer_materials(alice, bob, 1000, nullifier)
            .expect("materiales");

        // La identidad publica NO es la clave: es su hash.
        assert_ne!(
            m.sender.public_id,
            [key, BaseElement::ZERO, BaseElement::ZERO, BaseElement::ZERO],
            "los materiales no deben contener la clave"
        );
        assert_eq!(m.sender.public_id, derive_public_id(key));
    }

    /// **Sin la clave correcta no se puede generar la prueba**, aunque se
    /// tengan todos los materiales.
    ///
    /// Es lo que impide que quien intercepte los materiales pueda gastar.
    #[test]
    fn materials_alone_are_not_enough_to_spend() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 0);
        let key = BaseElement::new(SK_ALICE);
        let view = layer.account_view(alice).expect("vista");
        let nullifier = client::compute_nullifier(key, view.nonce);
        let m = layer
            .transfer_materials(alice, bob, 250_000, nullifier)
            .expect("materiales");

        // Un atacante con TODOS los materiales pero sin la clave.
        let r = client::prove_transfer(&m, BaseElement::new(0x1337));
        assert!(
            matches!(r, Err(LayerError::NotTheAccountHolder)),
            "CRITICO: quien intercepte los materiales NO debe poder gastar. \
             Resultado: {r:?}"
        );
    }

    /// El nullifier solo lo puede calcular el titular: es lo que impide
    /// vigilar cuándo gasta una cuenta ajena.
    #[test]
    fn only_the_holder_can_compute_the_nullifier() {
        let nonce = BaseElement::new(3);
        let real = client::compute_nullifier(BaseElement::new(SK_ALICE), nonce);
        let guess = client::compute_nullifier(BaseElement::new(0x1337), nonce);
        assert_ne!(real, guess);
    }
}
