//! **ZK-Sovereign Settlement Layer.**
//!
//! Los crates anteriores implementan **circuitos**: demuestran que una
//! transferencia concreta es válida. Este mantiene el **estado** y aplica
//! operaciones una tras otra. Es lo que convierte un conjunto de
//! primitivas criptográficas en una capa de liquidación.
//!
//! ## Qué mantiene
//!
//! - **Árbol de cuentas**: cada hoja es `H(H(pk, balance), nonce)`.
//! - **Árbol de nullifiers**: los gastados, para la no-pertenencia.
//! - **La cadena de raíces**: cada operación parte de la que dejó la
//!   anterior, y cada transición está respaldada por una prueba.
//!
//! ## Qué garantiza cada transferencia
//!
//! Sin revelar identidades, saldos ni importes:
//!
//! 1. El emisor conoce la clave de gasto de la cuenta de origen.
//! 2. `amount <= balance` y `amount <= limite regulatorio`.
//! 3. Ambas cuentas existen en el árbol.
//! 4. Lo debitado es exactamente lo acreditado (partida doble).
//! 5. El nullifier no se había gastado, y queda insertado.
//!
//! ## El modelo de operación
//!
//! ```text
//! layer.open_account(spend_key, saldo_inicial)  → índice de cuenta
//! layer.transfer(sk_emisor, idx_receptor, importe, limite) → Settlement
//! layer.apply(&settlement)                      → estado actualizado
//! ```
//!
//! `transfer` genera la prueba y **no toca el estado**; `apply` la
//! verifica y solo entonces aplica los cambios. Esa separación es
//! deliberada: permite que quien genera la prueba y quien la aplica sean
//! partes distintas, que es el caso de uso real entre bancos.
//!
//! ## ⚠️ Lo que esta capa NO es
//!
//! - **No hay red ni consenso.** Es un nodo único en memoria. Una
//!   federación real necesitaría acuerdo sobre el orden de las
//!   operaciones — y ese es un problema de sistemas distribuidos, no de
//!   criptografía.
//! - **No hay persistencia del árbol.** El estado vive en memoria.
//!   (`zk-core::persistent_nullifier_registry` persiste nullifiers, pero
//!   no el árbol de cuentas.)
//! - **La apertura de cuentas no está demostrada.** `open_account`
//!   modifica el árbol sin prueba: es una operación de administración,
//!   no de usuario. Un sistema real necesitaría un circuito de emisión.
//! - **No hay delegación de la prueba.** Quien la genera necesita la
//!   clave de gasto.

pub mod sparse_tree;

use ark_bls12_381::Fr;
use ark_ff::Zero;
use std::collections::HashMap;

use zk_core::circuit_mint::{derive_issuer_id, prove_mint, setup_mint, verify_mint, MintCircuit};
use zk_core::circuit_settlement::{
    account_id_from_key, account_leaf, prove_settlement, setup_settlement, verify_settlement,
    ReceiverWitness, SenderWitness, SettlementCircuit,
};
use zk_core::merkle::{MerklePath, TREE_DEPTH};
use zk_core::nullifier_tree::{self, NullifierPath, NULLIFIER_TREE_DEPTH};
use zk_core::proof_system::{ComplianceProof, ComplianceProvingKey, ComplianceVerifyingKey};
use zk_core::spend_authority::derive_nullifier;

use sparse_tree::SparseMerkleTree;

/// Índice de una cuenta dentro del árbol.
pub type AccountIndex = u64;

/// Semilla del generador para las pruebas de emisión.
const MINT_RNG_SEED: u64 = 0x1_5500;
/// Semilla del generador para las pruebas de transferencia.
const TRANSFER_RNG_SEED: u64 = 0xC0FFEE;

#[derive(Debug)]
pub enum LayerError {
    AccountNotFound(AccountIndex),
    InsufficientBalance { available: u64, requested: u64 },
    OverRegulatoryLimit { limit: u64, requested: u64 },
    NullifierAlreadySpent,
    ProofFailed(String),
    VerificationFailed,
    StaleState,
    /// Quien intenta emitir no es el emisor autorizado.
    NotTheIssuer,
    /// La liquidación declara un límite regulatorio distinto al del
    /// sistema. Sin esta comprobación, el regulado elegiría su propio
    /// límite y la restricción sería vacua.
    WrongRegulatoryLimit { expected: u64, declared: u64 },
}

impl std::fmt::Display for LayerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LayerError::AccountNotFound(i) => write!(f, "la cuenta {i} no existe"),
            LayerError::InsufficientBalance {
                available,
                requested,
            } => write!(
                f,
                "saldo insuficiente: hay {available}, se piden {requested}"
            ),
            LayerError::OverRegulatoryLimit { limit, requested } => write!(
                f,
                "importe {requested} por encima del limite regulatorio {limit}"
            ),
            LayerError::NullifierAlreadySpent => {
                write!(f, "el nullifier ya se gasto: doble gasto rechazado")
            }
            LayerError::ProofFailed(e) => write!(f, "no se pudo generar la prueba: {e}"),
            LayerError::VerificationFailed => write!(f, "la prueba no verifica"),
            LayerError::StaleState => write!(
                f,
                "la liquidacion parte de un estado que ya no es el actual"
            ),
            LayerError::NotTheIssuer => write!(
                f,
                "solo la autoridad emisora puede crear dinero"
            ),
            LayerError::WrongRegulatoryLimit { expected, declared } => write!(
                f,
                "limite regulatorio invalido: el sistema impone {expected}, \
                 la liquidacion declara {declared}"
            ),
        }
    }
}
impl std::error::Error for LayerError {}

/// Datos que la capa guarda de cada cuenta. **El saldo y el nonce son
/// conocidos por el operador del nodo**: esta capa no es privada frente a
/// sí misma, solo frente a terceros que ven las pruebas.
#[derive(Clone, Debug)]
struct AccountRecord {
    public_id: Fr,
    balance: u64,
    nonce: Fr,
}

/// Una liquidación: la prueba más los valores públicos que la
/// acompañan. Es lo que viaja entre partes.
pub struct Settlement {
    pub proof: ComplianceProof,
    pub root_old: Fr,
    pub root_new: Fr,
    pub nullifier_root_old: Fr,
    pub nullifier_root_new: Fr,
    pub regulatory_limit: u64,
    /// El nullifier gastado. **Público y necesario**: sin él, quien
    /// aplica la liquidación no puede insertarlo en su árbol y la
    /// no-pertenencia de operaciones futuras se vuelve vacua.
    ///
    /// Publicarlo es seguro: al derivarse de la clave de gasto es
    /// indistinguible y no se puede vincular a una cuenta.
    pub nullifier: Fr,
}

/// Árbol de cuentas: 20 niveles, el del circuito de cumplimiento.
type AccountTree = SparseMerkleTree<TREE_DEPTH>;
/// Árbol de nullifiers: 32 niveles. **Profundidad distinta**, y por eso
/// el tipo la fija: un desajuste entre ambas ya provocó un fallo, y
/// ahora el compilador lo impide.
type NullifierTree = SparseMerkleTree<NULLIFIER_TREE_DEPTH>;

pub struct SettlementLayer {
    accounts: AccountTree,
    nullifiers: NullifierTree,
    records: HashMap<AccountIndex, AccountRecord>,
    next_index: AccountIndex,
    pk: ComplianceProvingKey,
    vk: ComplianceVerifyingKey,
    mint_pk: ComplianceProvingKey,
    mint_vk: ComplianceVerifyingKey,
    /// Identidad pública de la autoridad emisora. Parámetro del sistema.
    issuer_id: Fr,
    /// Suministro total emitido. **Público y auditable**: solo crece
    /// mediante emisiones demostradas.
    total_supply: u64,
    /// Límite regulatorio por transacción. **Parámetro del sistema, no
    /// de la operación.**
    ///
    /// Una versión anterior lo recibía como argumento de `transfer` y lo
    /// verificaba contra el valor que la propia liquidación declaraba:
    /// el regulado elegía su propio límite. Poniendo `u64::MAX`, la
    /// restricción se volvía vacua.
    ///
    /// ⚠️ Esto es la corrección MÍNIMA. La completa es un circuito de
    /// gobernanza donde una autoridad reguladora compromete los
    /// parámetros en una raíz pública y cada liquidación demuestra haber
    /// usado los comprometidos. Mientras tanto, el límite lo fija quien
    /// arranca la capa, no quien opera en ella.
    regulatory_limit: u64,
}

/// Recibo de una emisión: la prueba y sus valores públicos.
pub struct MintReceipt {
    pub proof: ComplianceProof,
    pub root_old: Fr,
    pub root_new: Fr,
    pub amount: u64,
    pub supply_old: u64,
    pub supply_new: u64,
}

impl SettlementLayer {
    /// Arranca la capa generando las claves del circuito.
    ///
    /// ⚠️ El setup es de UNA SOLA PARTE. En producción habría que usar
    /// las claves de una ceremonia MPC real — el mecanismo está
    /// implementado y verificado en `crates/ceremony`.
    pub fn new(
        rng_seed: u64,
        issuer_key: Fr,
        regulatory_limit: u64,
    ) -> Result<Self, LayerError> {
        let (pk, vk) =
            setup_settlement(rng_seed).map_err(|e| LayerError::ProofFailed(e.to_string()))?;
        let (mint_pk, mint_vk) =
            setup_mint(rng_seed).map_err(|e| LayerError::ProofFailed(e.to_string()))?;
        Ok(Self {
            accounts: AccountTree::new(),
            nullifiers: NullifierTree::new(),
            records: HashMap::new(),
            next_index: 0,
            pk,
            vk,
            mint_pk,
            mint_vk,
            issuer_id: derive_issuer_id(issuer_key),
            total_supply: 0,
            regulatory_limit,
        })
    }

    /// Suministro total emitido. Auditable: la suma de todos los saldos
    /// debe coincidir con esta cifra.
    pub fn total_supply(&self) -> u64 {
        self.total_supply
    }

    /// Identidad pública de la autoridad emisora.
    pub fn issuer_id(&self) -> Fr {
        self.issuer_id
    }

    /// Límite regulatorio del sistema.
    pub fn regulatory_limit(&self) -> u64 {
        self.regulatory_limit
    }

    /// Raíz actual del árbol de cuentas.
    pub fn state_root(&self) -> Fr {
        self.accounts.root()
    }

    /// Raíz actual del árbol de nullifiers.
    pub fn nullifier_root(&self) -> Fr {
        self.nullifiers.root()
    }

    pub fn account_count(&self) -> usize {
        self.records.len()
    }

    /// Saldo de una cuenta, tal como lo ve el operador del nodo.
    pub fn balance_of(&self, index: AccountIndex) -> Option<u64> {
        self.records.get(&index).map(|r| r.balance)
    }

    /// Abre una cuenta **con saldo CERO**.
    ///
    /// No necesita prueba porque **no crea dinero**. Una versión anterior
    /// permitía abrir con saldo inicial arbitrario, y eso era una puerta
    /// trasera: toda la conservación demostrada en las transferencias era
    /// irrelevante si el operador podía crear cuentas con mil millones.
    ///
    /// Para que una cuenta tenga fondos hay que EMITIR (`mint`), y eso sí
    /// exige la clave de la autoridad emisora y deja rastro en el
    /// suministro público.
    pub fn open_account(&mut self, spend_key: Fr) -> AccountIndex {
        let index = self.next_index;
        self.next_index += 1;

        let public_id = account_id_from_key(spend_key);
        let nonce = Fr::zero();
        self.accounts
            .set_leaf(index, account_leaf(public_id, 0, nonce));
        self.records.insert(
            index,
            AccountRecord {
                public_id,
                balance: 0,
                nonce,
            },
        );
        index
    }

    /// Genera la prueba de una emisión. **No modifica el estado.**
    ///
    /// Solo funciona si `issuer_key` corresponde a la autoridad emisora
    /// del sistema.
    pub fn mint(
        &self,
        issuer_key: Fr,
        account_index: AccountIndex,
        amount: u64,
    ) -> Result<MintReceipt, LayerError> {
        if derive_issuer_id(issuer_key) != self.issuer_id {
            return Err(LayerError::NotTheIssuer);
        }
        let account = self
            .records
            .get(&account_index)
            .ok_or(LayerError::AccountNotFound(account_index))?
            .clone();

        let root_old = self.accounts.root();
        let path = {
            let (siblings, is_right) = self.accounts.path_for(account_index);
            MerklePath { siblings, is_right }
        };

        let mut new_tree = self.accounts.clone();
        new_tree.set_leaf(
            account_index,
            account_leaf(account.public_id, account.balance + amount, account.nonce),
        );
        let root_new = new_tree.root();

        let circuit = MintCircuit::new(
            issuer_key,
            account.public_id,
            account.balance,
            account.nonce,
            path,
            amount,
            root_old,
            root_new,
            self.total_supply,
        );

        let proof = prove_mint(&self.mint_pk, circuit, MINT_RNG_SEED)
            .map_err(|e| LayerError::ProofFailed(e.to_string()))?;

        Ok(MintReceipt {
            proof,
            root_old,
            root_new,
            amount,
            supply_old: self.total_supply,
            supply_new: self.total_supply + amount,
        })
    }

    /// Verifica una emisión y, si es válida y parte del estado actual, la
    /// aplica.
    pub fn apply_mint(
        &mut self,
        receipt: &MintReceipt,
        account_index: AccountIndex,
    ) -> Result<(), LayerError> {
        if receipt.root_old != self.accounts.root() || receipt.supply_old != self.total_supply {
            return Err(LayerError::StaleState);
        }

        let valid = verify_mint(
            &self.mint_vk,
            &receipt.proof,
            receipt.root_old,
            receipt.root_new,
            self.issuer_id,
            receipt.amount,
            receipt.supply_old,
            receipt.supply_new,
        )
        .map_err(|e| LayerError::ProofFailed(e.to_string()))?;

        if !valid {
            return Err(LayerError::VerificationFailed);
        }

        let account = self
            .records
            .get(&account_index)
            .ok_or(LayerError::AccountNotFound(account_index))?
            .clone();
        let updated = AccountRecord {
            public_id: account.public_id,
            balance: account.balance + receipt.amount,
            nonce: account.nonce,
        };
        self.accounts.set_leaf(
            account_index,
            account_leaf(updated.public_id, updated.balance, updated.nonce),
        );
        self.records.insert(account_index, updated);
        self.total_supply = receipt.supply_new;

        if self.accounts.root() != receipt.root_new {
            return Err(LayerError::StaleState);
        }
        Ok(())
    }

    /// Genera la prueba de una transferencia. **No modifica el estado.**
    ///
    /// La separación entre generar y aplicar es deliberada: permite que
    /// quien produce la prueba y quien la acepta sean partes distintas,
    /// que es el caso real entre bancos.
    pub fn transfer(
        &self,
        sender_key: Fr,
        sender_index: AccountIndex,
        receiver_index: AccountIndex,
        amount: u64,
    ) -> Result<Settlement, LayerError> {
        // El límite lo impone el SISTEMA, no quien transfiere.
        let regulatory_limit = self.regulatory_limit;
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

        // Comprobaciones tempranas: el circuito las volveria a imponer,
        // pero fallar aqui da un error legible en vez de una prueba que
        // no se puede generar.
        if amount > sender.balance {
            return Err(LayerError::InsufficientBalance {
                available: sender.balance,
                requested: amount,
            });
        }
        if amount > regulatory_limit {
            return Err(LayerError::OverRegulatoryLimit {
                limit: regulatory_limit,
                requested: amount,
            });
        }

        let nullifier = derive_nullifier(sender_key, sender.nonce);
        let null_pos = nullifier_tree::nullifier_position(nullifier);
        if !self.nullifiers.leaf(null_pos).is_zero() {
            return Err(LayerError::NullifierAlreadySpent);
        }

        // --- Raices y caminos, en el orden que exige el circuito ---
        let root_old = self.accounts.root();
        let sender_path = {
            let (siblings, is_right) = self.accounts.path_for(sender_index);
            MerklePath { siblings, is_right }
        };

        // Arbol INTERMEDIO: solo el emisor actualizado. El camino del
        // receptor debe salir de AQUI, no del arbol antiguo.
        let mut mid = self.accounts.clone();
        mid.set_leaf(
            sender_index,
            account_leaf(
                sender.public_id,
                sender.balance - amount,
                sender.nonce + Fr::from(1u64),
            ),
        );
        let receiver_path = {
            let (siblings, is_right) = mid.path_for(receiver_index);
            MerklePath { siblings, is_right }
        };

        let mut new_tree = mid;
        new_tree.set_leaf(
            receiver_index,
            account_leaf(
                receiver.public_id,
                receiver.balance + amount,
                receiver.nonce,
            ),
        );
        let root_new = new_tree.root();

        // --- Arbol de nullifiers ---
        let nullifier_root_old = self.nullifiers.root();
        let null_path = self.nullifier_path(null_pos);
        let mut null_new = self.nullifiers.clone();
        null_new.set_leaf(null_pos, nullifier);
        let nullifier_root_new = null_new.root();

        let circuit = SettlementCircuit::new(
            SenderWitness {
                spend_key: sender_key,
                balance: sender.balance,
                nonce: sender.nonce,
                merkle_path: sender_path,
            },
            ReceiverWitness {
                public_id: receiver.public_id,
                balance: receiver.balance,
                nonce: receiver.nonce,
                merkle_path: receiver_path,
            },
            amount,
            root_old,
            root_new,
            nullifier_root_old,
            nullifier_root_new,
            null_path,
            regulatory_limit,
        );

        let proof = prove_settlement(&self.pk, circuit, TRANSFER_RNG_SEED)
            .map_err(|e| LayerError::ProofFailed(e.to_string()))?;

        Ok(Settlement {
            proof,
            root_old,
            root_new,
            nullifier_root_old,
            nullifier_root_new,
            regulatory_limit,
            nullifier,
        })
    }

    /// Camino de una posición en el árbol de nullifiers.
    ///
    /// El árbol de nullifiers tiene profundidad distinta a la de cuentas,
    /// así que se construye aparte.
    fn nullifier_path(&self, position: u64) -> NullifierPath<Fr> {
        let (siblings, is_right) = self.nullifiers.path_for(position);
        NullifierPath { siblings, is_right }
    }

    /// Verifica una liquidación y, solo si es válida y parte del estado
    /// actual, la aplica.
    ///
    /// La comprobación de `root_old` es lo que impide aplicar dos veces
    /// la misma operación o aplicarla sobre un estado que ya cambió.
    pub fn apply(
        &mut self,
        settlement: &Settlement,
        sender_index: AccountIndex,
        receiver_index: AccountIndex,
        amount: u64,
    ) -> Result<(), LayerError> {
        // El límite declarado debe ser el del sistema. Sin esto, una
        // liquidación podría venir con un límite inventado y la prueba
        // verificaría contra él, dejando la restricción sin efecto.
        if settlement.regulatory_limit != self.regulatory_limit {
            return Err(LayerError::WrongRegulatoryLimit {
                expected: self.regulatory_limit,
                declared: settlement.regulatory_limit,
            });
        }

        if settlement.root_old != self.accounts.root()
            || settlement.nullifier_root_old != self.nullifiers.root()
        {
            return Err(LayerError::StaleState);
        }

        let valid = verify_settlement(
            &self.vk,
            &settlement.proof,
            settlement.root_old,
            settlement.root_new,
            settlement.nullifier_root_old,
            settlement.nullifier_root_new,
            settlement.regulatory_limit,
            settlement.nullifier,
        )
        .map_err(|e| LayerError::ProofFailed(e.to_string()))?;

        if !valid {
            return Err(LayerError::VerificationFailed);
        }

        // La prueba es valida: aplicar la transicion.
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
            nonce: sender.nonce + Fr::from(1u64),
        };
        let new_receiver = AccountRecord {
            public_id: receiver.public_id,
            balance: receiver.balance + amount,
            nonce: receiver.nonce,
        };

        self.accounts.set_leaf(
            sender_index,
            account_leaf(new_sender.public_id, new_sender.balance, new_sender.nonce),
        );
        self.accounts.set_leaf(
            receiver_index,
            account_leaf(
                new_receiver.public_id,
                new_receiver.balance,
                new_receiver.nonce,
            ),
        );
        self.records.insert(sender_index, new_sender);
        self.records.insert(receiver_index, new_receiver);

        // INSERTAR EL NULLIFIER. Una version anterior lo olvidaba: el
        // arbol nunca crecia y la no-pertenencia era vacua en la
        // practica. Lo destapo el test `full_transfer_cycle_updates_state`
        // al comprobar que la raiz de nullifiers cambia.
        let null_pos = nullifier_tree::nullifier_position(settlement.nullifier);
        self.nullifiers.set_leaf(null_pos, settlement.nullifier);

        if self.nullifiers.root() != settlement.nullifier_root_new {
            return Err(LayerError::StaleState);
        }

        // Comprobacion de coherencia: si la raiz resultante no coincide
        // con la que la prueba declara, el estado del nodo y el del
        // circuito han divergido — un fallo grave que debe detenerlo todo.
        if self.accounts.root() != settlement.root_new {
            return Err(LayerError::StaleState);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Cada test que crea una capa paga el setup del circuito (~2,6 s) y
    /// cada transferencia una prueba (~2 s). Los tests son pocos y
    /// deliberados; ejecutar en release.
    const SK_ALICE: u64 = 0xA11CE;
    const SK_BOB: u64 = 0xB0B;
    const SK_ISSUER: u64 = 0xBA1CE47;
    const LIMIT: u64 = 500_000;

    fn new_layer() -> SettlementLayer {
        SettlementLayer::new(42, Fr::from(SK_ISSUER), LIMIT)
            .expect("el arranque de la capa no deberia fallar")
    }

    /// Abre una cuenta y le emite fondos, que es el unico camino legitimo
    /// para que una cuenta tenga saldo.
    fn open_and_fund(layer: &mut SettlementLayer, sk: u64, amount: u64) -> AccountIndex {
        let idx = layer.open_account(Fr::from(sk));
        if amount > 0 {
            let receipt = layer
                .mint(Fr::from(SK_ISSUER), idx, amount)
                .expect("la emision autorizada deberia generar prueba");
            layer.apply_mint(&receipt, idx).expect("aplicar emision");
        }
        idx
    }

    /// **Abrir una cuenta NO crea dinero**: siempre nace con saldo cero.
    #[test]
    fn opening_an_account_creates_no_money() {
        let mut layer = new_layer();
        let alice = layer.open_account(Fr::from(SK_ALICE));
        assert_eq!(layer.balance_of(alice), Some(0));
        assert_eq!(
            layer.total_supply(),
            0,
            "abrir cuentas no debe aumentar el suministro"
        );
    }

    /// **EL TEST QUE CIERRA LA PUERTA TRASERA EN LA CAPA.**
    ///
    /// Quien no es la autoridad emisora no puede crear dinero. Con la
    /// versión anterior de `open_account`, que aceptaba un saldo inicial
    /// arbitrario, esto era trivial.
    #[test]
    fn only_the_issuer_can_create_money() {
        let mut layer = new_layer();
        let alice = layer.open_account(Fr::from(SK_ALICE));

        let r = layer.mint(Fr::from(0x1337u64), alice, 1_000_000);
        assert!(
            matches!(r, Err(LayerError::NotTheIssuer)),
            "CRITICO: sin la clave del emisor no debe poder crearse dinero"
        );
        assert_eq!(layer.total_supply(), 0);
        assert_eq!(layer.balance_of(alice), Some(0));
    }

    /// **LA INVARIANTE GLOBAL**: la suma de todos los saldos equivale
    /// siempre al suministro emitido.
    ///
    /// Es lo que convierte la conservación de local (por transferencia) a
    /// global y auditable.
    #[test]
    fn total_balances_always_equal_total_supply() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 50_000);

        let sum = |l: &SettlementLayer| -> u64 {
            [alice, bob].iter().map(|i| l.balance_of(*i).unwrap()).sum()
        };

        assert_eq!(sum(&layer), layer.total_supply(), "tras la emision");

        let s = layer
            .transfer(Fr::from(SK_ALICE), alice, bob, 250_000)
            .expect("prueba");
        layer.apply(&s, alice, bob, 250_000).expect("aplicar");

        assert_eq!(
            sum(&layer),
            layer.total_supply(),
            "transferir NO debe alterar el suministro total"
        );
        assert_eq!(layer.total_supply(), 1_050_000);
    }

    /// Emitir aumenta el suministro exactamente en lo emitido.
    #[test]
    fn minting_increases_supply_exactly() {
        let mut layer = new_layer();
        let alice = layer.open_account(Fr::from(SK_ALICE));

        let receipt = layer
            .mint(Fr::from(SK_ISSUER), alice, 500_000)
            .expect("emision");
        assert_eq!(layer.total_supply(), 0, "mint no debe mutar el estado");

        layer.apply_mint(&receipt, alice).expect("aplicar");
        assert_eq!(layer.total_supply(), 500_000);
        assert_eq!(layer.balance_of(alice), Some(500_000));
    }

    /// **EL TEST QUE CIERRA EL AGUJERO DEL LIMITE REGULATORIO.**
    ///
    /// Una liquidación que declara un límite distinto al del sistema debe
    /// rechazarse. Antes, `transfer` recibía el límite como argumento y
    /// `apply` lo verificaba contra ese mismo valor: **el regulado
    /// elegía su propio límite**, y con `u64::MAX` la restricción era
    /// vacua.
    #[test]
    fn settlement_with_foreign_limit_is_rejected() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 50_000);

        let mut s = layer
            .transfer(Fr::from(SK_ALICE), alice, bob, 250_000)
            .expect("prueba");

        // Se manipula el limite declarado, como haria quien quisiera
        // esquivar la restriccion regulatoria.
        s.regulatory_limit = u64::MAX;

        let r = layer.apply(&s, alice, bob, 250_000);
        assert!(
            matches!(r, Err(LayerError::WrongRegulatoryLimit { .. })),
            "CRITICO: el limite regulatorio lo impone el sistema, no quien \
             transfiere. Resultado: {r:?}"
        );
        assert_eq!(layer.balance_of(alice), Some(1_000_000), "sin cambios");
    }

    /// Reaplicar una emisión debe rechazarse, igual que una transferencia.
    #[test]
    fn replaying_a_mint_is_rejected() {
        let mut layer = new_layer();
        let alice = layer.open_account(Fr::from(SK_ALICE));
        let receipt = layer.mint(Fr::from(SK_ISSUER), alice, 500_000).expect("emision");

        layer.apply_mint(&receipt, alice).expect("primera");
        assert!(
            matches!(layer.apply_mint(&receipt, alice), Err(LayerError::StaleState)),
            "CRITICO: reaplicar una emision duplicaria el dinero"
        );
        assert_eq!(layer.total_supply(), 500_000);
    }

    /// EL TEST CLAVE DE LA CAPA: una transferencia completa, con prueba,
    /// verificación y aplicación del estado.
    #[test]
    fn full_transfer_cycle_updates_state() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 50_000);

        let root_before = layer.state_root();
        let null_root_before = layer.nullifier_root();

        let settlement = layer
            .transfer(Fr::from(SK_ALICE), alice, bob, 250_000)
            .expect("una transferencia valida deberia generar prueba");

        // `transfer` NO toca el estado: esa separacion permite que quien
        // genera la prueba y quien la aplica sean partes distintas.
        assert_eq!(layer.state_root(), root_before, "transfer no debe mutar");

        layer
            .apply(&settlement, alice, bob, 250_000)
            .expect("una liquidacion valida deberia aplicarse");

        assert_eq!(layer.balance_of(alice), Some(750_000));
        assert_eq!(layer.balance_of(bob), Some(300_000));
        assert_ne!(layer.state_root(), root_before);
        assert_ne!(
            layer.nullifier_root(),
            null_root_before,
            "el nullifier deberia haberse insertado"
        );

        // La conservacion se cumple tambien a nivel de capa.
        let total: u64 = [alice, bob]
            .iter()
            .map(|i| layer.balance_of(*i).unwrap())
            .sum();
        assert_eq!(total, 1_050_000, "el dinero total no debe cambiar");
    }

    /// **EL TEST QUE IMPIDE LA REPETICIÓN.**
    ///
    /// Aplicar dos veces la misma liquidación debe fallar: la segunda
    /// parte de un estado que ya no es el actual. Sin esta comprobación,
    /// reenviar una liquidación válida duplicaría el dinero.
    #[test]
    fn replaying_a_settlement_is_rejected() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 50_000);

        let settlement = layer
            .transfer(Fr::from(SK_ALICE), alice, bob, 250_000)
            .expect("prueba");

        layer.apply(&settlement, alice, bob, 250_000).expect("primera");

        let result = layer.apply(&settlement, alice, bob, 250_000);
        assert!(
            matches!(result, Err(LayerError::StaleState)),
            "CRITICO: reaplicar una liquidacion debe rechazarse, o el dinero \
             se duplicaria. Resultado: {result:?}"
        );
        assert_eq!(layer.balance_of(alice), Some(750_000), "sin cambios");
    }

    /// Gastar más del saldo se rechaza con un error legible, antes de
    /// intentar generar la prueba.
    #[test]
    fn insufficient_balance_is_reported_clearly() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 100_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 50_000);

        let r = layer.transfer(Fr::from(SK_ALICE), alice, bob, 250_000);
        assert!(matches!(
            r,
            Err(LayerError::InsufficientBalance {
                available: 100_000,
                requested: 250_000
            })
        ));
    }

    /// Superar el límite regulatorio también.
    #[test]
    fn over_regulatory_limit_is_reported_clearly() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 50_000);

        let r = layer.transfer(Fr::from(SK_ALICE), alice, bob, 750_000);
        assert!(matches!(r, Err(LayerError::OverRegulatoryLimit { .. })));
    }

    /// Una cuenta inexistente da error, no un pánico.
    #[test]
    fn unknown_account_is_reported() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let r = layer.transfer(Fr::from(SK_ALICE), alice, 999, 1000);
        assert!(matches!(r, Err(LayerError::AccountNotFound(999))));
    }

    /// **DOS TRANSFERENCIAS ENCADENADAS.**
    ///
    /// La segunda parte de la raíz que dejó la primera, y usa un nonce
    /// distinto (por tanto un nullifier distinto). Es la prueba de que la
    /// capa mantiene estado de verdad y no solo produce pruebas sueltas.
    #[test]
    fn consecutive_transfers_chain_correctly() {
        let mut layer = new_layer();
        let alice = open_and_fund(&mut layer, SK_ALICE, 1_000_000);
        let bob = open_and_fund(&mut layer, SK_BOB, 50_000);

        let s1 = layer
            .transfer(Fr::from(SK_ALICE), alice, bob, 100_000)
            .expect("primera prueba");
        layer.apply(&s1, alice, bob, 100_000).expect("primera");
        let root_mid = layer.state_root();

        let s2 = layer
            .transfer(Fr::from(SK_ALICE), alice, bob, 200_000)
            .expect("segunda prueba");
        assert_eq!(
            s2.root_old, root_mid,
            "la segunda liquidacion debe partir de la raiz que dejo la primera"
        );
        layer.apply(&s2, alice, bob, 200_000).expect("segunda");

        assert_eq!(layer.balance_of(alice), Some(700_000));
        assert_eq!(layer.balance_of(bob), Some(350_000));
    }
}
