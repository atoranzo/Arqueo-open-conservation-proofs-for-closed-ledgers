//! Extensión de `ComplianceCircuit` que además demuestra que `balance`
//! corresponde a una cuenta real registrada en el árbol de estado del
//! ledger (identificado por su raíz pública `state_root`), no un número
//! elegido libremente por el emisor.
//!
//! Esto cierra el hueco #1 documentado en el README original: "balance no
//! está vinculado a ningún estado real del ledger".

use ark_bls12_381::Fr;
use ark_crypto_primitives::sponge::Absorb;
use ark_ff::PrimeField;
use ark_r1cs_std::fields::fp::FpVar;
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};

use crate::circuit::enforce_range;
use crate::merkle::{enforce_merkle_membership, leaf_commitment, MerklePath, TREE_DEPTH};
use crate::nullifier::{compute_nullifier, enforce_nullifier_computation};
use crate::proof_system::{
    prove_generic, setup_generic, verify_generic, ComplianceProof, ComplianceProvingKey,
    ComplianceVerifyingKey, ZkCoreError,
};

/// Igual que `ComplianceCircuit`, pero con la cuenta ligada al árbol de
/// estado mediante un compromiso de hoja y su camino de Merkle, y con un
/// nullifier público que impide reutilizar la misma prueba dos veces.
#[derive(Clone)]
pub struct ComplianceCircuitWithState<F: PrimeField> {
    // --- Testigos privados de la cuenta y la transacción ---
    pub account_id: Option<F>,
    pub balance: Option<u64>,
    pub account_nonce: Option<F>,
    pub amount: Option<u64>,
    pub merkle_path: Option<MerklePath<F>>,

    // --- Entradas públicas ---
    pub state_root: F,
    pub regulatory_limit: F,
    /// Nullifier público de esta transacción. Se calcula automáticamente
    /// en `new()` a partir de `account_id` y `account_nonce`, para que sea
    /// imposible por construcción pasar un nullifier que no corresponda a
    /// esos testigos (evitando ese error por diseño, en vez de confiar en
    /// que el llamador lo calcule bien por su cuenta).
    pub nullifier: F,
}

impl<F: PrimeField + Absorb> ComplianceCircuitWithState<F> {
    pub fn new(
        account_id: F,
        balance: u64,
        account_nonce: F,
        amount: u64,
        merkle_path: MerklePath<F>,
        state_root: F,
        regulatory_limit: u64,
    ) -> Self {
        let nullifier = compute_nullifier(account_id, account_nonce);
        Self {
            account_id: Some(account_id),
            balance: Some(balance),
            account_nonce: Some(account_nonce),
            amount: Some(amount),
            merkle_path: Some(merkle_path),
            state_root,
            regulatory_limit: F::from(regulatory_limit),
            nullifier,
        }
    }

    pub fn empty_for_setup(state_root: F, regulatory_limit: u64) -> Self {
        Self {
            account_id: None,
            balance: None,
            account_nonce: None,
            amount: None,
            merkle_path: None,
            state_root,
            regulatory_limit: F::from(regulatory_limit),
            nullifier: F::zero(),
        }
    }
}

impl<F: PrimeField + Absorb> ConstraintSynthesizer<F> for ComplianceCircuitWithState<F> {
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        // --- Testigos privados ---
        let account_id_var = FpVar::<F>::new_witness(cs.clone(), || {
            self.account_id.ok_or(SynthesisError::AssignmentMissing)
        })?;
        let balance_var = FpVar::<F>::new_witness(cs.clone(), || {
            self.balance
                .map(F::from)
                .ok_or(SynthesisError::AssignmentMissing)
        })?;
        let nonce_var = FpVar::<F>::new_witness(cs.clone(), || {
            self.account_nonce.ok_or(SynthesisError::AssignmentMissing)
        })?;
        let amount_var = FpVar::<F>::new_witness(cs.clone(), || {
            self.amount
                .map(F::from)
                .ok_or(SynthesisError::AssignmentMissing)
        })?;

        // --- Entradas públicas ---
        let state_root_var = FpVar::<F>::new_input(cs.clone(), || Ok(self.state_root))?;
        let limit_var = FpVar::<F>::new_input(cs.clone(), || Ok(self.regulatory_limit))?;
        let nullifier_var = FpVar::<F>::new_input(cs.clone(), || Ok(self.nullifier))?;

        // --- Camino de Merkle (privado: siblings + bits de dirección) ---
        //
        // CORRECCIÓN (hallazgo confirmado por ejecución real): durante el
        // `setup()` (generación de claves), el circuito se construye con
        // `merkle_path: None` (no hay testigo real todavía). Si aquí se
        // hiciera un `.ok_or(...)?` inmediato como en la versión anterior,
        // el setup fallaba con `AssignmentMissing` ANTES de llegar a
        // construir ninguna restricción — confirmado por
        // `SetupFailed("AssignmentMissing")` al ejecutar `setup_with_state()`
        // de verdad. La estructura de restricciones debe tener el MISMO
        // número de variables tanto en setup como al probar; el contenido
        // (ceros de relleno vs. valores reales) no importa en fase de
        // setup, solo la forma.
        let (siblings_native, is_right_native): (Vec<F>, Vec<bool>) = match &self.merkle_path {
            Some(p) => (p.siblings.clone(), p.is_right.clone()),
            None => (vec![F::zero(); TREE_DEPTH], vec![false; TREE_DEPTH]),
        };

        let siblings_var: Vec<FpVar<F>> = siblings_native
            .iter()
            .map(|s| FpVar::<F>::new_witness(cs.clone(), || Ok(*s)))
            .collect::<Result<_, _>>()?;

        let is_right_var: Vec<Boolean<F>> = is_right_native
            .iter()
            .map(|b| Boolean::new_witness(cs.clone(), || Ok(*b)))
            .collect::<Result<_, _>>()?;

        assert_eq!(siblings_var.len(), TREE_DEPTH);
        assert_eq!(is_right_var.len(), TREE_DEPTH);

        // --- 1. La hoja se calcula EN CIRCUITO a partir de los testigos,
        //         no se recibe como testigo aparte. Esto evita que alguien
        //         pase una hoja arbitraria que no corresponda a
        //         account_id/balance/nonce reales. Usa Poseidon real. ---
        let leaf_var = {
            use crate::poseidon_hash::secure_hash_gadget;
            let inner = secure_hash_gadget(cs.clone(), &account_id_var, &balance_var)?;
            secure_hash_gadget(cs.clone(), &inner, &nonce_var)?
        };

        // --- 2. Verificar que esa hoja pertenece al árbol con raíz pública
        //         state_root. Esto es lo que ata `balance` al estado real. ---
        enforce_merkle_membership(cs.clone(), &leaf_var, &siblings_var, &is_right_var, &state_root_var)?;

        // --- 2b. Verificar que el nullifier público declarado corresponde
        //          REALMENTE a account_id/account_nonce. Sin esto, alguien
        //          podría declarar un nullifier inventado (por ejemplo, uno
        //          aleatorio distinto cada vez) para esquivar el registro
        //          de gastados del ledger y reutilizar la prueba. ---
        enforce_nullifier_computation(cs.clone(), &account_id_var, &nonce_var, &nullifier_var)?;

        // --- 3. Las mismas comprobaciones de solvencia y límite regulatorio
        //         de ComplianceCircuit original. ---
        enforce_range(&balance_var)?;
        enforce_range(&amount_var)?;
        enforce_range(&limit_var)?;

        let diff_balance = &balance_var - &amount_var;
        enforce_range(&diff_balance)?;

        let diff_limit = &limit_var - &amount_var;
        enforce_range(&diff_limit)?;

        Ok(())
    }
}

/// Helper nativo (fuera de circuito) para construir el compromiso de hoja
/// exactamente como lo calcula el circuito. Debe usarse al insertar cuentas
/// en el árbol para que los dos caminos (nativo y en-circuito) coincidan
/// SIEMPRE. Si un solo bit de esto se implementa distinto entre ambos
/// lados, el circuito nunca será satisfacible para ningún testigo real.
pub fn compute_leaf<F: PrimeField + Absorb>(account_id: F, balance: u64, nonce: F) -> F {
    leaf_commitment(account_id, F::from(balance), nonce)
}

// ---------------------------------------------------------------------
// Wrappers Groth16 específicos de ComplianceCircuitWithState. Antes de
// esto, `proof_system.rs` solo sabía generar/verificar pruebas para
// `ComplianceCircuit` (la versión sin estado); esta sección cierra ese
// hueco reutilizando las funciones genéricas (`setup_generic`,
// `prove_generic`, `verify_generic`) en vez de duplicar la lógica.
// ---------------------------------------------------------------------

/// Genera las claves de prueba/verificación para `ComplianceCircuitWithState`.
/// Distinta de `setup()` (que es para `ComplianceCircuit`, sin estado):
/// las claves de un circuito NO sirven para verificar pruebas del otro,
/// porque tienen un número y una estructura de restricciones diferentes.
pub fn setup_with_state(
    rng_seed: u64,
) -> Result<(ComplianceProvingKey, ComplianceVerifyingKey), ZkCoreError> {
    let empty = ComplianceCircuitWithState::<Fr>::empty_for_setup(Fr::from(0u64), 0);
    setup_generic(empty, rng_seed)
}

/// Genera una prueba real para `ComplianceCircuitWithState`.
pub fn prove_with_state(
    pk: &ComplianceProvingKey,
    circuit: ComplianceCircuitWithState<Fr>,
    rng_seed: u64,
) -> Result<ComplianceProof, ZkCoreError> {
    prove_generic(pk, circuit, rng_seed)
}

/// Construye el vector de inputs públicos EN EL ORDEN EXACTO en que
/// `generate_constraints` los allocó: `state_root`, `regulatory_limit`,
/// `nullifier` — en ese orden y no otro. Si alguna vez se reordenan las
/// líneas `new_input(...)` dentro de `generate_constraints`, hay que
/// actualizar esta función a la vez, o toda verificación empezará a fallar
/// de forma silenciosamente confusa (la prueba "no verifica" sin motivo
/// aparente, porque el orden de los inputs no coincide).
pub fn public_inputs_for_verification(
    state_root: Fr,
    regulatory_limit: u64,
    nullifier: Fr,
) -> Vec<Fr> {
    vec![state_root, Fr::from(regulatory_limit), nullifier]
}

/// Verifica una prueba de `ComplianceCircuitWithState` contra los tres
/// valores públicos: la raíz de estado esperada, el límite regulatorio, y
/// el nullifier declarado. Devolver `Ok(true)` aquí SOLO confirma que la
/// prueba es criptográficamente válida — NO confirma que el nullifier no
/// se haya usado antes. Esa comprobación es responsabilidad de
/// `NullifierRegistry` (o el ledger real), aparte de esta función. Ver
/// el test `full_flow_valid_proof_then_registry_blocks_replay` para una
/// demostración explícita de por qué ambas capas son necesarias.
pub fn verify_with_state(
    vk: &ComplianceVerifyingKey,
    proof: &ComplianceProof,
    state_root: Fr,
    regulatory_limit: u64,
    nullifier: Fr,
) -> Result<bool, ZkCoreError> {
    let public_inputs = public_inputs_for_verification(state_root, regulatory_limit, nullifier);
    verify_generic(vk, proof, &public_inputs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::merkle::SimpleMerkleTree;
    use crate::nullifier::compute_nullifier;
    use ark_bls12_381::Fr;
    use ark_relations::r1cs::ConstraintSystem;

    /// Construye un árbol de prueba con una única cuenta real en la
    /// posición 3, y devuelve (path, root) — helper compartido por varios
    /// tests para no repetir el mismo boilerplate.
    fn build_test_tree(account_id: Fr, balance: u64, nonce: Fr) -> (crate::merkle::MerklePath<Fr>, Fr) {
        let leaf = compute_leaf(account_id, balance, nonce);
        let mut leaves = vec![Fr::from(1u64), Fr::from(2u64), Fr::from(3u64), leaf];
        leaves.resize(8, Fr::from(0u64));
        let tree = SimpleMerkleTree::build(leaves);
        (tree.path_for(3), tree.root())
    }

    #[test]
    fn valid_state_linked_transaction_satisfies_constraints() {
        let account_id = Fr::from(12345u64);
        let nonce = Fr::from(1u64);
        let balance: u64 = 1_000_000;
        let amount: u64 = 250_000;
        let regulatory_limit: u64 = 500_000;

        let (path, root) = build_test_tree(account_id, balance, nonce);

        let circuit = ComplianceCircuitWithState::new(
            account_id, balance, nonce, amount, path, root, regulatory_limit,
        );

        let cs = ConstraintSystem::<Fr>::new_ref();
        circuit.generate_constraints(cs.clone()).unwrap();
        assert!(
            cs.is_satisfied().unwrap(),
            "una transaccion valida con cuenta real en el arbol deberia satisfacer el circuito"
        );
    }

    #[test]
    fn wrong_balance_not_matching_leaf_fails_constraints() {
        let account_id = Fr::from(12345u64);
        let nonce = Fr::from(1u64);
        let real_balance: u64 = 1_000_000;

        let (path, root) = build_test_tree(account_id, real_balance, nonce);

        // El circuito se construye declarando un balance MAYOR al real
        // (mintiendo sobre el saldo para poder gastar mas).
        let claimed_balance: u64 = 50_000_000;

        let circuit = ComplianceCircuitWithState::new(
            account_id, claimed_balance, nonce, 250_000, path, root, 500_000,
        );

        let cs = ConstraintSystem::<Fr>::new_ref();
        circuit.generate_constraints(cs.clone()).unwrap();
        assert!(
            !cs.is_satisfied().unwrap(),
            "declarar un balance distinto al comprometido en el arbol debe romper la prueba de pertenencia"
        );
    }

    /// EL TEST CLAVE DE ESTA FASE: si alguien intenta declarar un
    /// nullifier que NO corresponde a su account_id/nonce real (por
    /// ejemplo, uno aleatorio para intentar esquivar el registro de
    /// gastados), el circuito debe rechazarlo. Sin esta restriccion, el
    /// nullifier seria decorativo: cualquiera podria ponerle un valor
    /// distinto cada vez y reutilizar la prueba indefinidamente.
    #[test]
    fn forged_nullifier_not_derived_from_witnesses_fails_constraints() {
        let account_id = Fr::from(12345u64);
        let nonce = Fr::from(1u64);
        let balance: u64 = 1_000_000;

        let (path, root) = build_test_tree(account_id, balance, nonce);

        let mut circuit = ComplianceCircuitWithState::new(
            account_id, balance, nonce, 250_000, path, root, 500_000,
        );

        // El nullifier correcto ya se calculo dentro de new(). Lo
        // sobrescribimos con un valor inventado, como haria un atacante
        // que intenta declarar un nullifier distinto en cada intento.
        circuit.nullifier = Fr::from(999_999_999u64);

        let cs = ConstraintSystem::<Fr>::new_ref();
        circuit.generate_constraints(cs.clone()).unwrap();
        assert!(
            !cs.is_satisfied().unwrap(),
            "CRITICO: un nullifier inventado (no derivado de account_id/nonce) \
             fue aceptado. Esto anularia por completo la proteccion contra doble gasto."
        );
    }

    /// Confirma que el nullifier calculado automáticamente por `new()`
    /// coincide con lo que calcularía el ledger de forma independiente
    /// (usando `compute_nullifier` directamente) — así el ledger puede
    /// predecir el nullifier esperado ANTES de recibir la prueba, para
    /// comprobar de antemano si ya fue gastado.
    #[test]
    fn nullifier_matches_independent_native_computation() {
        let account_id = Fr::from(777u64);
        let nonce = Fr::from(5u64);
        let (path, root) = build_test_tree(account_id, 1_000_000, nonce);

        let circuit =
            ComplianceCircuitWithState::new(account_id, 1_000_000, nonce, 100_000, path, root, 500_000);

        let expected = compute_nullifier(account_id, nonce);
        assert_eq!(
            circuit.nullifier, expected,
            "el nullifier expuesto por el circuito debe coincidir con el que calcula el ledger de forma independiente"
        );
    }

    /// Prueba de extremo a extremo REAL con Groth16 (no solo satisfacibilidad
    /// en memoria): setup -> prove -> verify sobre `ComplianceCircuitWithState`,
    /// y demostración explícita de por qué hacen falta DOS capas de defensa
    /// contra el doble gasto, no solo una:
    ///
    /// 1. La prueba criptográfica (Groth16) demuestra que la transacción es
    ///    VÁLIDA (fondos suficientes, cuenta real, nullifier bien derivado).
    /// 2. Pero la MISMA prueba, verificada una segunda vez, sigue siendo
    ///    criptográficamente válida — es la misma prueba, no ha dejado de
    ///    serlo. Groth16 no sabe nada sobre "ya se usó antes".
    /// 3. Por eso el `NullifierRegistry` es una capa aparte, del lado del
    ///    ledger: la que decide si ACEPTA una prueba válida o la rechaza
    ///    por ser una reutilización.
    #[test]
    fn end_to_end_groth16_valid_proof_then_registry_blocks_replay() {
        let account_id = Fr::from(555u64);
        let nonce = Fr::from(9u64);
        let balance: u64 = 2_000_000;
        let amount: u64 = 300_000;
        let regulatory_limit: u64 = 500_000;

        let (path, root) = build_test_tree(account_id, balance, nonce);

        let circuit = ComplianceCircuitWithState::new(
            account_id, balance, nonce, amount, path, root, regulatory_limit,
        );
        let nullifier = circuit.nullifier;

        let (pk, vk) = setup_with_state(11).expect("setup_with_state no deberia fallar");
        let proof = prove_with_state(&pk, circuit, 22)
            .expect("prove_with_state no deberia fallar con una transaccion valida");

        // 1. La prueba criptografica es valida.
        let is_valid = verify_with_state(&vk, &proof, root, regulatory_limit, nullifier)
            .expect("la verificacion no deberia devolver error");
        assert!(is_valid, "una transaccion valida con estado real debe verificar como verdadera");

        // 2. La MISMA prueba, verificada de nuevo, sigue siendo valida
        //    criptograficamente. Esto NO es un bug: es exactamente por lo
        //    que el nullifier registry existe como capa aparte.
        let is_still_cryptographically_valid =
            verify_with_state(&vk, &proof, root, regulatory_limit, nullifier)
                .expect("la verificacion no deberia devolver error");
        assert!(
            is_still_cryptographically_valid,
            "Groth16 por si solo no rechaza una prueba reutilizada: \
             la validez criptografica no caduca con el uso."
        );

        // 3. El registro de nullifiers es quien SI bloquea la reutilizacion.
        let mut registry = crate::nullifier::NullifierRegistry::<Fr>::new();
        assert!(
            registry.check_and_mark_spent(nullifier).is_ok(),
            "el primer uso del nullifier debe aceptarse"
        );
        let second_attempt = registry.check_and_mark_spent(nullifier);
        assert!(
            second_attempt.is_err(),
            "CRITICO: el registro debio rechazar la reutilizacion del mismo nullifier \
             (intento de doble gasto)."
        );
    }
}
