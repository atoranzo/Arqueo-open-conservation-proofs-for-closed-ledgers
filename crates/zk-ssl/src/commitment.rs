//! **Capa por compromisos**: demostración de que el operador no necesita
//! ver los saldos.
//!
//! ## Por qué existe este módulo
//!
//! El principio **P2 — recipiente limpio** dice que el sistema *no debe ver
//! más de lo necesario*. Hoy no se cumple: la capa guarda
//! `(identidad, saldo, nonce)` de cada cuenta y **el operador los ve en
//! memoria**. El cifrado en reposo protege ante el robo del disco, no ante
//! él.
//!
//! Cerrarlo exige un refactor de la capa entera —101 referencias en nueve
//! módulos—. Este módulo **demuestra primero que el diseño funciona**,
//! antes de comprometerse con ese trabajo.
//!
//! ## La observación
//!
//! La capa guarda `(identidad, saldo, nonce)`. **No lo necesita.** Para
//! mantener el árbol y verificar transiciones le basta con **el digest de
//! la hoja**.
//!
//! | | Hoy | Por compromisos |
//! |---|---|---|
//! | La capa guarda | id, saldo, nonce | **Solo `H(H(id,saldo),nonce)`** |
//! | Calcula raíces y caminos | Sí | **Sí** |
//! | **Puede leer un saldo** | **Sí** ⚠️ | **No** |
//!
//! ## Por qué sigue siendo sólido
//!
//! El cliente aporta la posición y **el digest de la hoja nueva**. La capa:
//!
//! 1. Verifica la prueba: acredita `raíz_antigua → raíz_nueva`.
//! 2. Comprueba que `raíz_antigua` es la vigente.
//! 3. Coloca la hoja nueva en esa posición.
//! 4. **Comprueba que la raíz resultante es `raíz_nueva`.**
//!
//! Si el cliente mintiera sobre la hoja o la posición, **el paso 4 falla**.
//! La capa no necesita entender el contenido para comprobar que la
//! transición es la que la prueba acredita.
//!
//! ## ⚠️ El coste, que es real
//!
//! **El titular lleva su propio estado.** Si pierde su copia local no sabe
//! cuánto tiene — aunque el dinero siga ahí y la recuperación por custodios
//! siga funcionando.
//!
//! Es el modelo de Zcash, y es un cambio de usabilidad serio: hoy la capa
//! puede responder *"tienes X"* y con este diseño **no puede**.
//!
//! Un despliegue real lo resolvería con el proveedor de servicios de pago
//! guardando el estado del cliente — con lo que la legibilidad se
//! re-concentra, aunque repartida y no en el operador central.

use std::collections::HashMap;
use winterfell::math::fields::f64::BaseElement;

use crate::sparse_tree::SparseTree;
use crate::{AccountIndex, Digest};

/// Lo que la capa guarda de una cuenta: **solo el digest de su hoja**.
///
/// No hay identidad, ni saldo, ni nonce. El operador no puede leer nada
/// aunque quiera.
pub type LeafCommitment = Digest;

/// Lo que el **titular** guarda de su propia cuenta.
///
/// Vive en el cliente, no en la capa. Es lo que le permite construir
/// pruebas, y perderlo significa no saber cuánto tiene.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientState {
    pub public_id: Digest,
    pub balance: u64,
    pub nonce: BaseElement,
}

/// Capa que solo conoce compromisos.
///
/// Mantiene el árbol de cuentas sin poder leer ninguna.
#[derive(Clone, Debug, Default)]
pub struct CommitmentLayer {
    tree: SparseTree,
    /// Compromisos por posición. **No hay saldos aquí.**
    leaves: HashMap<AccountIndex, LeafCommitment>,
}

/// Una transición que el cliente propone y la capa comprueba.
#[derive(Clone, Debug)]
pub struct LeafUpdate {
    pub index: AccountIndex,
    pub new_leaf: LeafCommitment,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CommitmentError {
    /// La raíz declarada no es la vigente.
    StaleRoot,
    /// Aplicar las hojas propuestas no produce la raíz declarada.
    ///
    /// Es lo que detecta a un cliente que miente sobre su hoja o su
    /// posición.
    RootMismatch,
}

impl std::fmt::Display for CommitmentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommitmentError::StaleRoot => {
                write!(f, "la raiz declarada no es la vigente")
            }
            CommitmentError::RootMismatch => write!(
                f,
                "las hojas propuestas no producen la raiz declarada: el \
                 cliente miente sobre su hoja o su posicion"
            ),
        }
    }
}
impl std::error::Error for CommitmentError {}

impl CommitmentLayer {
    pub fn new() -> Self {
        Self {
            tree: SparseTree::new(),
            leaves: HashMap::new(),
        }
    }

    pub fn root(&self) -> Digest {
        self.tree.root()
    }

    /// El compromiso de una cuenta. **No revela nada de su contenido.**
    pub fn commitment_of(&self, index: AccountIndex) -> Option<LeafCommitment> {
        self.leaves.get(&index).copied()
    }

    /// Camino de autenticación, que el cliente necesita para probar.
    pub fn path_for(&self, index: AccountIndex) -> crate::MerklePath {
        self.tree.path_for(index)
    }

    /// Abre una cuenta colocando el compromiso que el cliente aporta.
    ///
    /// La capa **no sabe qué contiene**. Que nazca a cero lo garantiza el
    /// circuito de liquidación, no esta función.
    pub fn open(&mut self, index: AccountIndex, leaf: LeafCommitment) {
        self.tree.set_leaf(index, leaf);
        self.leaves.insert(index, leaf);
    }

    /// **Aplica una transición sin entender su contenido.**
    ///
    /// El cliente aporta las hojas nuevas; la capa comprueba que producen
    /// la raíz que la prueba acredita.
    ///
    /// Si el cliente mintiera sobre una hoja o una posición, la raíz
    /// resultante no coincidiría. **La capa no necesita leer nada para
    /// detectarlo.**
    pub fn apply(
        &mut self,
        root_old: Digest,
        root_new: Digest,
        updates: &[LeafUpdate],
    ) -> Result<(), CommitmentError> {
        if self.tree.root() != root_old {
            return Err(CommitmentError::StaleRoot);
        }

        // Se aplican sobre una copia: si la raíz no cuadra, el estado
        // queda intacto.
        let mut tentativo = self.tree.clone();
        for u in updates {
            tentativo.set_leaf(u.index, u.new_leaf);
        }
        if tentativo.root() != root_new {
            return Err(CommitmentError::RootMismatch);
        }

        self.tree = tentativo;
        for u in updates {
            self.leaves.insert(u.index, u.new_leaf);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sparse_tree::SparseTree;
    use stark_experiment::circuit_settlement::{derive_public_id, native_leaf};
    use winterfell::math::FieldElement;

    fn estado(clave: u64, saldo: u64, nonce: u64) -> ClientState {
        ClientState {
            public_id: derive_public_id(BaseElement::new(clave)),
            balance: saldo,
            nonce: BaseElement::new(nonce),
        }
    }

    fn hoja(s: &ClientState) -> Digest {
        native_leaf(s.public_id, BaseElement::new(s.balance), s.nonce)
    }

    /// **LA CAPA NO GUARDA NINGÚN SALDO.**
    ///
    /// Es la propiedad central. Se comprueba buscando el saldo entre los
    /// bytes de todo lo que la capa retiene.
    #[test]
    fn the_layer_holds_no_balances() {
        const SALDO: u64 = 0x05A3_B7C9; // 94.615.497, distintivo
        let mut capa = CommitmentLayer::new();
        let s = estado(0xDEADBEEF, SALDO, 7);
        capa.open(0, hoja(&s));

        // Todo lo que la capa retiene, en bytes.
        let mut retenido = Vec::new();
        for (_, c) in &capa.leaves {
            for e in c {
                retenido.extend_from_slice(&e.as_int().to_le_bytes());
            }
        }
        for e in capa.root() {
            retenido.extend_from_slice(&e.as_int().to_le_bytes());
        }

        let patron = &SALDO.to_le_bytes()[..4];
        assert!(
            !retenido.windows(4).any(|w| w == patron),
            "CRITICO: el saldo aparece en lo que la capa retiene"
        );
    }

    /// **Y el que valida al anterior**: si la capa guardara el saldo, SÍ
    /// aparecería.
    ///
    /// Sin esto, el test anterior pasaría aunque la búsqueda estuviera mal
    /// construida.
    #[test]
    fn a_layer_that_stored_balances_would_show_them() {
        const SALDO: u64 = 0x05A3_B7C9;
        // Simulacion de la capa ACTUAL: guarda el saldo en claro.
        let mut retenido = Vec::new();
        retenido.extend_from_slice(&SALDO.to_le_bytes());

        let patron = &SALDO.to_le_bytes()[..4];
        assert!(
            retenido.windows(4).any(|w| w == patron),
            "la busqueda debe encontrar un saldo cuando lo hay, o el test \
             anterior no comprueba nada"
        );
    }

    /// **LA CAPA APLICA TRANSICIONES SIN ENTENDERLAS.**
    ///
    /// Una transferencia de 250.000 entre dos cuentas. La capa comprueba
    /// que las hojas propuestas producen la raíz declarada, y no lee
    /// ningún importe.
    #[test]
    fn the_layer_applies_a_transfer_without_reading_amounts() {
        let mut capa = CommitmentLayer::new();
        let emisor = estado(0xA11CE, 1_000_000, 7);
        let receptor = estado(0xB0B, 50_000, 3);
        capa.open(0, hoja(&emisor));
        capa.open(1, hoja(&receptor));

        let raiz_antes = capa.root();

        // El cliente calcula su estado nuevo y las hojas.
        let emisor_nuevo = ClientState { balance: 750_000, ..emisor.clone() };
        let receptor_nuevo = ClientState { balance: 300_000, ..receptor.clone() };

        // Y la raiz que resultara, que la prueba acredita.
        let mut previsto = SparseTree::new();
        previsto.set_leaf(0, hoja(&emisor_nuevo));
        previsto.set_leaf(1, hoja(&receptor_nuevo));
        let raiz_despues = previsto.root();

        let r = capa.apply(
            raiz_antes,
            raiz_despues,
            &[
                LeafUpdate { index: 0, new_leaf: hoja(&emisor_nuevo) },
                LeafUpdate { index: 1, new_leaf: hoja(&receptor_nuevo) },
            ],
        );
        assert!(r.is_ok(), "{r:?}");
        assert_eq!(capa.root(), raiz_despues);
    }

    /// **UN CLIENTE QUE MIENTE SOBRE SU HOJA SE DETECTA.**
    ///
    /// Es lo que sostiene la solidez del diseño: la capa no entiende el
    /// contenido, pero comprueba que la raíz resultante es la que la
    /// prueba acredita.
    #[test]
    fn a_client_lying_about_the_new_leaf_is_caught() {
        let mut capa = CommitmentLayer::new();
        let emisor = estado(0xA11CE, 1_000_000, 7);
        capa.open(0, hoja(&emisor));
        let raiz_antes = capa.root();

        // La raiz que la prueba acredita: saldo 750.000.
        let honrado = ClientState { balance: 750_000, ..emisor.clone() };
        let mut previsto = SparseTree::new();
        previsto.set_leaf(0, hoja(&honrado));
        let raiz_despues = previsto.root();

        // Pero el cliente entrega la hoja de un saldo MAYOR.
        let mentiroso = ClientState { balance: 999_999, ..emisor.clone() };
        let r = capa.apply(
            raiz_antes,
            raiz_despues,
            &[LeafUpdate { index: 0, new_leaf: hoja(&mentiroso) }],
        );
        assert_eq!(
            r,
            Err(CommitmentError::RootMismatch),
            "CRITICO: un cliente que entregue una hoja distinta de la que \
             la prueba acredita debe ser rechazado"
        );
        assert_eq!(capa.root(), raiz_antes, "y el estado debe quedar intacto");
    }

    /// **MENTIR SOBRE LA POSICIÓN TAMPOCO SIRVE.**
    #[test]
    fn a_client_lying_about_the_position_is_caught() {
        let mut capa = CommitmentLayer::new();
        let emisor = estado(0xA11CE, 1_000_000, 7);
        capa.open(0, hoja(&emisor));
        let raiz_antes = capa.root();

        let nuevo = ClientState { balance: 750_000, ..emisor.clone() };
        let mut previsto = SparseTree::new();
        previsto.set_leaf(0, hoja(&nuevo));
        let raiz_despues = previsto.root();

        // La hoja correcta, pero en la posicion equivocada.
        let r = capa.apply(
            raiz_antes,
            raiz_despues,
            &[LeafUpdate { index: 5, new_leaf: hoja(&nuevo) }],
        );
        assert_eq!(r, Err(CommitmentError::RootMismatch));
    }

    /// Una raíz antigua obsoleta se rechaza.
    #[test]
    fn a_stale_root_is_rejected() {
        let mut capa = CommitmentLayer::new();
        capa.open(0, hoja(&estado(0xA11CE, 1_000_000, 7)));
        let r = capa.apply(
            [BaseElement::new(999); 4],
            capa.root(),
            &[],
        );
        assert_eq!(r, Err(CommitmentError::StaleRoot));
    }

    /// **EL CLIENTE SÍ PUEDE RECONSTRUIR SU ESTADO.**
    ///
    /// La contrapartida del diseño: el titular lleva su propio estado y la
    /// capa solo confirma que corresponde.
    ///
    /// Si perdiera su copia, el dinero sigue ahí pero **no sabría cuánto
    /// tiene**. Es el modelo de Zcash, y un cambio de usabilidad serio.
    #[test]
    fn the_client_can_check_its_state_against_the_layer() {
        let mut capa = CommitmentLayer::new();
        let s = estado(0xA11CE, 1_000_000, 7);
        capa.open(0, hoja(&s));

        // El cliente comprueba que su estado corresponde al compromiso.
        assert_eq!(capa.commitment_of(0), Some(hoja(&s)));

        // Y un estado equivocado no cuadra.
        let mal = ClientState { balance: 999_999, ..s.clone() };
        assert_ne!(capa.commitment_of(0), Some(hoja(&mal)));
    }
}
