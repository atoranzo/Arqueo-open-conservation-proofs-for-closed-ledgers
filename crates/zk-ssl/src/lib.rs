//! # ZK-Sovereign Settlement Layer
//!
//! Capa de liquidación con privacidad criptográfica y cumplimiento
//! demostrable, **sin ninguna ceremonia de confianza**.
//!
//! ## Por qué STARK y no otro paradigma
//!
//! El proyecto implementó el mismo circuito en cinco paradigmas (Groth16,
//! Halo2/IPA, STARK/FRI, PLONK/KZG y Nova) y midió sus trade-offs. La
//! elección para la capa se deduce del principio de soberanía:
//!
//! **Groth16 y PLONK-KZG exigen una ceremonia de confianza.** Si sus
//! participantes coluden, pueden falsificar pruebas y **crear dinero de
//! la nada**, sin que nadie lo detecte jamás. Para una infraestructura
//! soberana eso es una dependencia externa permanente e inauditable.
//!
//! Sin ceremonia quedan Halo2/IPA y STARK/FRI. De los dos, **STARK gana
//! en todo salvo el tamaño de prueba** y es el único con resistencia
//! cuántica — lo que importa en una infraestructura pensada para durar
//! décadas.
//!
//! El precio, medido: **36,7 KB por prueba** frente a los 192 bytes de
//! Groth16. Es el coste de no depender de nadie.
//!
//! ## Qué garantiza cada operación
//!
//! Sin revelar identidades, saldos ni importes:
//!
//! | Vía de crear dinero | Cerrada por |
//! |---|---|
//! | Transferir más de lo debitado | Conservación (partida doble) |
//! | Abrir cuenta con saldo | Apertura siempre a cero |
//! | Emitir sin autorización | Clave del emisor demostrada en circuito |
//! | Emisión encubierta | Suministro público atado en el circuito |
//! | Gastar dos veces | No-pertenencia demostrable |
//! | Gastar sin ser el titular | Autoridad de gasto |
//! | Reenviar una operación válida | Encadenamiento de raíces en la capa |
//!
//! ## El modelo de operación
//!
//! ```text
//! layer.open_account(sk)                    → cuenta con saldo CERO
//! layer.mint(&auth, cuenta, importe)        → EXIGE DOS custodios
//! layer.transfer(sk, origen, destino, imp)  → EXIGE clave de gasto
//!
//! // O SIN entregar la clave a la capa (ver `client`):
//! let m = layer.transfer_materials(origen, destino, imp, nullifier)?;
//! let s = client::prove_transfer(&m, sk)?;   // en la maquina del titular
//! layer.apply(...)                          → verifica y aplica
//! ```
//!
//! Generar la prueba y aplicarla están **separados a propósito**: permite
//! que quien produce la prueba y quien la acepta sean partes distintas,
//! que es el caso real entre entidades.
//!
//! ## ⚠️ Lo que esta capa NO es
//!
//! - **No hay red ni consenso.** Es un nodo único en memoria. Una
//!   federación real necesita acuerdo sobre el orden de las operaciones —
//!   un problema de sistemas distribuidos, no de criptografía.
//! - **No hay persistencia.** Reiniciar pierde el ledger.
//! - **No hay delegación de la prueba.** Quien la genera necesita la
//!   clave de gasto; en un banco, la clave estaría en un HSM y el cómputo
//!   en otro servicio.
//! - **No hay destrucción de circulante (burn)** ni política monetaria.
//! - **La clave del emisor es única**, no de umbral.
//! - **Nada de esto ha sido auditado por terceros.**

mod accounts;
mod audit;
pub mod commitment;
pub mod client;
pub mod crypto;
pub mod iso;
pub mod log;
use crate::log::{OpKind, TransitionLog};
mod burn;
mod mint;
mod freeze;
mod governance;
mod persistence;
mod recovery;
pub mod snapshot;
pub mod sparse_tree;
pub mod store;
mod transfer;

#[cfg(test)]
mod metrics;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_support;

use std::collections::HashMap;
use winterfell::math::fields::f64::BaseElement;
use winterfell::math::{FieldElement, StarkField};
use winterfell::{
    crypto::hashers::Blake3_256, crypto::DefaultRandomCoin, crypto::MerkleTree, verify,
    AcceptableOptions, BatchingMethod, FieldExtension, ProofOptions, Prover,
};

use stark_experiment::circuit_audit::{
    build_trace as build_audit_trace, AuditAir, AuditProver, AuditPublicInputs, AuditWitness,
    MAX_VALUE,
};
use stark_experiment::circuit_burn::{
    build_trace as build_burn_trace, BurnAir, BurnProver, BurnPublicInputs,
};
use stark_experiment::circuit_freeze::{
    build_trace as build_freeze_trace, frozen_leaf, FreezeAir, FreezeProver, FreezePublicInputs,
    FROZEN_DEPTH,
};
use stark_experiment::circuit_governance::{
    build_trace as build_governance_trace, GovernanceAir, GovernanceProver,
    GovernancePublicInputs,
};
/// Autorización de dos miembros del conjunto de gobernanza.
pub use stark_experiment::circuit_governance::{build_governance_set, GovernanceAuth};
use stark_experiment::circuit_mint::{
    build_trace as build_mint_trace, MintAir, MintProver, MintPublicInputs,
};
/// Autorización de dos custodios, necesaria para emitir.
pub use stark_experiment::circuit_mint::ThresholdAuth;
/// Construcción del conjunto de custodios: devuelve su raíz y los
/// caminos de cada miembro.
pub use stark_experiment::circuit_threshold::build_custodian_set;
use stark_experiment::circuit_recovery::{
    build_trace as build_recovery_trace, RecoveryAir, RecoveryProver, RecoveryPublicInputs,
};
use stark_experiment::circuit_settlement::{
    build_trace as build_settlement_trace, derive_public_id, native_leaf, ReceiverWitness,
    SenderWitness, SettlementAir, SettlementProver, SettlementPublicInputs,
};
use stark_experiment::merkle::{Digest, MerklePath};
use stark_experiment::nullifier_tree::nullifier_position;

use sparse_tree::SparseTree;
use store::{digest_from_bytes, digest_to_bytes, record_from_bytes, record_to_bytes, StoreError};

type Blake3 = Blake3_256<BaseElement>;

/// Índice de una cuenta dentro del árbol.
pub type AccountIndex = u64;

/// Opciones de prueba del sistema.
///
/// **Configuración de 127 bits conjeturados** (blowup 16, extensión
/// cuadrática). Ver `FIVE_BACKENDS.md` sobre la diferencia entre
/// seguridad conjeturada y demostrable: alcanzar 128 bits DEMOSTRABLES
/// costaría 125,6 KB por prueba en vez de 36,7.
pub fn proof_options() -> ProofOptions {
    ProofOptions::new(
        42,
        16,
        21,
        FieldExtension::Quadratic,
        8,
        31,
        BatchingMethod::Linear,
        BatchingMethod::Linear,
    )
}

#[derive(Debug)]
pub enum LayerError {
    AccountNotFound(AccountIndex),
    InsufficientBalance { available: u64, requested: u64 },
    OverRegulatoryLimit { limit: u64, requested: u64 },
    NullifierAlreadySpent,
    NotTheIssuer,
    ProofFailed(String),
    VerificationFailed(String),
    StaleState,
    WrongRegulatoryLimit { expected: u64, declared: u64 },
    /// Fallo de persistencia, incluida la corrupción del ledger.
    Store(StoreError),
    /// Quien pide la revelación no es el titular de la cuenta.
    NotTheAccountHolder,
    /// El saldo no está en la banda que se pretende demostrar.
    BalanceOutsideBand { lower: u64, upper: u64 },
    /// La emisión superaría el tope del sistema.
    SupplyCapExceeded { cap: u64, would_be: u64 },
    /// La recuperación asigna la misma identidad que ya tenía: no
    /// cambiaría nada y solo incrementaría el contador.
    RecoveryToSameIdentity,
    /// Se alcanzó el tope de cuentas del sistema.
    AccountLimitReached { limit: u64 },
    /// La cuenta está congelada: no puede gastar.
    AccountFrozen(AccountIndex),
    /// La congelación no cambiaría nada.
    AlreadyInThatFreezeState,
}

impl From<StoreError> for LayerError {
    fn from(e: StoreError) -> Self {
        LayerError::Store(e)
    }
}

impl std::fmt::Display for LayerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use LayerError::*;
        match self {
            AccountNotFound(i) => write!(f, "la cuenta {i} no existe"),
            InsufficientBalance {
                available,
                requested,
            } => write!(f, "saldo insuficiente: hay {available}, se piden {requested}"),
            OverRegulatoryLimit { limit, requested } => write!(
                f,
                "importe {requested} por encima del limite regulatorio {limit}"
            ),
            NullifierAlreadySpent => {
                write!(f, "el nullifier ya se gasto: doble gasto rechazado")
            }
            NotTheIssuer => write!(
                f,
                "crear dinero exige dos custodios distintos del conjunto autorizado"
            ),
            ProofFailed(e) => write!(f, "no se pudo generar la prueba: {e}"),
            VerificationFailed(e) => write!(f, "la prueba no verifica: {e}"),
            StaleState => write!(
                f,
                "la operacion parte de un estado que ya no es el actual"
            ),
            WrongRegulatoryLimit { expected, declared } => write!(
                f,
                "limite regulatorio invalido: el sistema impone {expected}, se declara {declared}"
            ),
            Store(e) => write!(f, "{e}"),
            NotTheAccountHolder => write!(
                f,
                "solo el titular puede revelar el saldo de su cuenta"
            ),
            BalanceOutsideBand { lower, upper } => write!(
                f,
                "el saldo no esta en la banda [{lower}, {upper}] que se pretende demostrar"
            ),
            SupplyCapExceeded { cap, would_be } => write!(
                f,
                "la emision llevaria el suministro a {would_be}, por encima del tope {cap}"
            ),
            RecoveryToSameIdentity => write!(
                f,
                "la recuperacion asignaria la misma identidad que ya tenia"
            ),
            AccountLimitReached { limit } => write!(
                f,
                "se alcanzo el tope de {limit} cuentas del sistema"
            ),
            AccountFrozen(i) => write!(f, "la cuenta {i} esta congelada y no puede gastar"),
            AlreadyInThatFreezeState => {
                write!(f, "la cuenta ya esta en ese estado de congelacion")
            }
        }
    }
}
impl std::error::Error for LayerError {}

/// Datos que el nodo guarda de cada cuenta.
///
/// **El operador del nodo ve estos valores.** La privacidad es frente a
/// terceros que solo ven las pruebas, no frente a quien mantiene el
/// estado. Una federación real repartiría el estado entre nodos.
#[derive(Clone, Debug)]
struct AccountRecord {
    public_id: Digest,
    balance: u64,
    nonce: BaseElement,
}

/// Liquidación: la prueba y los valores públicos que la acompañan.
#[derive(Debug)]
pub struct Settlement {
    pub proof: Vec<u8>,
    pub public_inputs: SettlementPublicInputs,
}

/// Revelación dirigida a un supervisor.
///
/// Viaja sola: el supervisor **no necesita acceso a la capa** para
/// verificarla, solo la raíz del estado que audita.
#[derive(Debug)]
pub struct AuditDisclosure {
    pub proof: Vec<u8>,
    pub public_inputs: AuditPublicInputs,
}

/// Verifica una revelación. **Función libre a propósito**: un supervisor
/// la usa sin tener el ledger ni ningún estado del sistema.
///
/// Devuelve `Ok(())` si la prueba es válida para la raíz declarada. Queda
/// a cargo de quien verifica comprobar que esa raíz es la del estado que
/// pretende auditar — igual que en las liquidaciones, el circuito prueba
/// la afirmación y quien la recibe ancla el contexto.
pub fn verify_audit(disclosure: &AuditDisclosure) -> Result<(), LayerError> {
    let proof = winterfell::Proof::from_bytes(&disclosure.proof)
        .map_err(|e| LayerError::VerificationFailed(format!("prueba mal formada: {e:?}")))?;
    let min_opts = AcceptableOptions::OptionSet(vec![proof_options()]);
    verify::<AuditAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
        proof,
        disclosure.public_inputs.clone(),
        &min_opts,
    )
    .map_err(|e| LayerError::VerificationFailed(format!("{e:?}")))
}

/// Recibo de una congelación o descongelación.
#[derive(Debug)]
pub struct FreezeReceipt {
    pub proof: Vec<u8>,
    pub public_inputs: FreezePublicInputs,
    /// Estado al que pasa la cuenta. Lo necesita quien aplica el recibo;
    /// el circuito solo demuestra que la transición es válida.
    pub now_frozen: bool,
}

/// Recibo de un cambio de gobernanza.
#[derive(Debug)]
pub struct GovernanceReceipt {
    pub proof: Vec<u8>,
    pub public_inputs: GovernancePublicInputs,
}

/// Recibo de una recuperación de cuenta.
///
/// Lleva la identidad nueva porque quien aplica la recuperación necesita
/// saber a quién reasignar la cuenta; el circuito solo demuestra que la
/// transición es válida.
#[derive(Debug)]
pub struct RecoveryReceipt {
    pub proof: Vec<u8>,
    pub public_inputs: RecoveryPublicInputs,
    pub new_public_id: Digest,
}

/// Recibo de una emisión.
/// Recibo de una destrucción de circulante.
///
/// **La identidad de la cuenta no aparece**: desde fuera se sabe que
/// alguien con autoridad destruyó ese importe y que el suministro bajó en
/// consecuencia, pero no de quién era.
#[derive(Debug)]
pub struct BurnReceipt {
    pub proof: Vec<u8>,
    pub public_inputs: BurnPublicInputs,
}

#[derive(Debug)]
pub struct MintReceipt {
    pub proof: Vec<u8>,
    pub public_inputs: MintPublicInputs,
}

/// ⚠️ **No implementa `Debug` deliberadamente.** Contiene los saldos de
/// todas las cuentas, y un `{:?}` en un registro de diagnóstico los
/// expondría. Para inspeccionar el estado están los accesores
/// (`balance_of`, `state_root`, `total_supply`...), que devuelven lo
/// concreto que se pide.
pub struct SovereignLayer {
    accounts: SparseTree,
    nullifiers: SparseTree,
    records: HashMap<AccountIndex, AccountRecord>,
    next_index: AccountIndex,
    /// **Raíz del conjunto de custodios autorizados.** Crear dinero y
    /// recuperar cuentas exige dos custodios distintos del conjunto.
    ///
    /// **Mutable por gobernanza**: si un custodio se compromete, el
    /// conjunto puede cambiarse sin crear un ledger nuevo.
    ///
    /// ⚠️ La garantía es "dos claves comprometidas en vez de una", **no**
    /// "dos voluntades independientes": en un nodo único, quien genera la
    /// prueba necesita las dos claves a la vez.
    custodian_set_root: Digest,
    /// **Raíz del conjunto de gobernanza. INMUTABLE.**
    ///
    /// Es la única autoridad que puede cambiar el conjunto de custodios,
    /// y cambiarla a ella exige crear un ledger nuevo.
    ///
    /// La circularidad no desaparece —quien la controle controla todo—
    /// pero se traslada a claves que se usan casi nunca y pueden
    /// guardarse sin conexión.
    governance_set_root: Digest,
    /// Contador público de cambios de gobernanza.
    governance_change_count: u64,
    total_supply: u64,
    /// **Contador público de recuperaciones.** Cada intervención de los
    /// custodios lo incrementa.
    ///
    /// Sin él, los custodios podrían reasignar cuentas en silencio: desde
    /// fuera, una recuperación es indistinguible de cualquier otra
    /// transición de estado. No impide el abuso —nada en un circuito
    /// puede— pero lo hace **contable**.
    /// **Árbol de congelados.** Profundidad 24 —no 32— porque su subida
    /// tiene que caber en las filas libres del circuito de liquidación.
    ///
    /// Una cuenta congelada no puede gastar, y **eso lo impone el
    /// circuito**: la prueba de liquidación acredita que el emisor no
    /// está en este árbol.
    frozen: SparseTree,
    /// Contador público de congelaciones y descongelaciones.
    freeze_count: u64,
    /// **Registro encadenado de transiciones.**
    ///
    /// Hace que el operador no pueda reescribir el historial en secreto.
    /// No impide que vea los saldos ni que censure: eso exige consenso.
    log: TransitionLog,
    recovery_count: u64,
    regulatory_limit: u64,
    /// **Tope de emisión.** Parámetro inmutable del ledger: ni siquiera
    /// el conjunto completo de custodios puede superarlo sin crear un
    /// ledger nuevo, y eso dejaría rastro evidente.
    max_supply: u64,
    /// **Tope de cuentas.** Parámetro inmutable.
    ///
    /// `open_account` no exige autorización de ningún tipo, así que sin
    /// un tope cualquiera podría crear cuentas hasta agotar la memoria
    /// del nodo. No crea dinero —nacen a cero— pero es denegación de
    /// servicio trivial.
    ///
    /// ⚠️ **Acota el daño, no impide el abuso.** Un atacante puede agotar
    /// el cupo y dejar sin sitio a usuarios legítimos. Cerrarlo de verdad
    /// exigiría autorización de custodio para abrir, y eso requiere un
    /// circuito nuevo: abrir hoy no genera ninguna prueba.
    max_accounts: u64,
    options: ProofOptions,
    /// Almacenamiento persistente. `None` para una capa en memoria.
    db: Option<sled::Db>,
    /// Clave de cifrado en reposo. `None` = sin cifrar.
    ///
    /// **No se guarda junto a los datos**: la aporta el operador al
    /// arrancar. Guardarla al lado no protegería nada.
    key: Option<crate::crypto::LedgerKey>,
}

impl SovereignLayer {
    /// Arranca la capa.
    ///
    /// **No hay setup de claves.** Es la propiedad que distingue este
    /// paradigma: no existe ninguna ceremonia que celebrar ni ningún
    /// secreto que destruir. Los parámetros son públicos y verificables.
    pub fn new(
        custodian_set_root: Digest,
        governance_set_root: Digest,
        regulatory_limit: u64,
        max_supply: u64,
        max_accounts: u64,
    ) -> Self {
        Self {
            accounts: SparseTree::new(),
            nullifiers: SparseTree::new(),
            records: HashMap::new(),
            next_index: 0,
            custodian_set_root,
            governance_set_root,
            governance_change_count: 0,
            total_supply: 0,
            frozen: SparseTree::with_depth(FROZEN_DEPTH),
            freeze_count: 0,
            log: TransitionLog::new(),
            recovery_count: 0,
            regulatory_limit,
            max_supply,
            max_accounts,
            options: proof_options(),
            db: None,
            key: None,
        }
    }

}
