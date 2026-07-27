//! zk-core: motor de cumplimiento con pruebas de conocimiento cero reales
//! sobre BLS12-381 / Groth16, verificado de extremo a extremo (compilado
//! y ejecutado, no solo escrito).
//!
//! ## Lo que este módulo garantiza (verificado con tests reales)
//! - Que una prueba válida implica matemáticamente `amount <= balance` y
//!   `amount <= regulatory_limit`, sin revelar `amount` ni `balance`.
//! - Que los operandos están acotados a 64 bits, evitando el ataque de
//!   desbordamiento de módulo típico en circuitos ZK mal diseñados.
//! - Que `balance` está vinculado a una cuenta real dentro de un árbol de
//!   Merkle de 20 niveles (`ComplianceCircuitWithState`), con Poseidon
//!   real (`poseidon_hash.rs`).
//! - Prevención de doble gasto vía nullifier con separación de dominio,
//!   persistido en disco (`persistent_nullifier_registry.rs`).
//! - `SettlementProver` (`settlement_prover_impl.rs`): implementación del
//!   trait de abstracción compartido con `halo2-experiment`, para poder
//!   escribir código de orquestación genérico sobre el backend.
//!
//! ## Lo que este módulo NO resuelve todavía (limitaciones honestas)
//! - No incluye una ceremonia de trusted setup multi-parte (MPC). El
//!   `setup()` de este código es de un solo participante y NO debe usarse
//!   en producción tal cual — ver README para las dos investigaciones
//!   reales (`ark-marlin`, `celo-org/snark-setup`) que confirmaron que no
//!   hay alternativa madura en el ecosistema Arkworks actual.
//! - `PersistentNullifierRegistry` es de un solo nodo, no distribuida.
//! - No ha sido auditado por terceros.

pub mod circuit;
pub mod circuit_audit;
pub mod circuit_double_entry;
pub mod circuit_mint;
pub mod circuit_settlement;
pub mod circuit_with_state;
pub mod merkle;
pub mod nullifier;
pub mod nullifier_tree;
pub mod performance;
pub mod persistent_nullifier_registry;
pub mod poseidon_hash;
pub mod proof_system;
pub mod spend_authority;
pub mod settlement_prover_impl;

pub use circuit::{ComplianceCircuit, VALUE_BITS};
pub use circuit_with_state::{
    compute_leaf, public_inputs_for_verification, prove_with_state, setup_with_state,
    verify_with_state, ComplianceCircuitWithState,
};
pub use merkle::{leaf_commitment, MerklePath, SimpleMerkleTree, TREE_DEPTH};
pub use nullifier::{compute_nullifier, NullifierError, NullifierRegistry};
pub use persistent_nullifier_registry::PersistentNullifierRegistry;
pub use proof_system::{
    prove, prove_generic, setup, setup_generic, verify, verify_generic, ComplianceProof,
    ComplianceProvingKey, ComplianceVerifyingKey, ZkCoreError,
};

// `circuit_with_state`, `merkle` y `nullifier` usan `secure_hash`/
// `secure_hash_gadget` (Poseidon real, ver `poseidon_hash.rs`) desde esta
// versión. `toy_hash` se conserva solo como referencia de comparación en
// un test, no en ninguna ruta funcional.
