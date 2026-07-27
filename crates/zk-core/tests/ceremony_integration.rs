//! Ceremonia MPC real sobre el circuito de cumplimiento de este proyecto.
//!
//! Este test es la respuesta a la limitación más grave que el README ha
//! documentado desde el principio: *"trusted setup de un solo
//! participante, sin resolver"*. Aquí las claves de prueba se generan
//! mediante una ceremonia de varias contribuciones, y se demuestra que la
//! `ProvingKey` resultante produce pruebas Groth16 verificables del
//! circuito real — no de un circuito de juguete.
//!
//! ## ⚠️ Qué demuestra esto, y qué NO
//!
//! **Demuestra**: que el mecanismo de ceremonia funciona de extremo a
//! extremo con nuestro circuito y nuestra curva, y que sus claves son
//! válidas.
//!
//! **NO demuestra que exista seguridad real.** Las tres contribuciones se
//! generan aquí en el mismo proceso y con el mismo generador de números
//! aleatorios. La garantía de una ceremonia MPC es *"basta con que UN
//! participante sea honesto y destruya su aleatoriedad"* — y con un solo
//! proceso no hay ningún participante independiente. Una ceremonia real
//! exige personas distintas, en máquinas distintas, publicando el
//! transcript de contribuciones para que cualquiera pueda verificarlo.
//!
//! Esa distinción es la diferencia entre "el código para hacer una
//! ceremonia existe y funciona" y "se ha celebrado una ceremonia". Lo
//! primero es lo que este test establece.
//!
//! ## Aviso de tiempo
//!
//! La fase 1 construye vectores proporcionales al tamaño del circuito, y
//! cada contribución los recorre completos. Con el circuito de
//! cumplimiento (árbol de Merkle de 20 niveles con Poseidon) esto puede
//! tardar bastantes minutos. Ejecutar SIEMPRE en release:
//!
//! ```text
//! cargo test -p zk-core --release --test ceremony_integration -- --nocapture
//! ```

use ark_bls12_381::Fr;
use ark_relations::r1cs::{
    ConstraintSynthesizer, ConstraintSystem, OptimizationGoal, SynthesisMode,
};
use rand::SeedableRng;

use ceremony::{
    circuit_degree, combine, log::Hashable, transition, Phase1CRSElements, Phase1Contribution,
    Phase2Contribution,
};
use zk_core::circuit_with_state::{compute_leaf, prove_with_state, verify_with_state, ComplianceCircuitWithState};
use zk_core::merkle::SimpleMerkleTree;

/// Construye un escenario válido: una cuenta con saldo dentro de un árbol
/// de Merkle real, y una transacción solvente y dentro del límite.
fn build_scenario() -> (ComplianceCircuitWithState<Fr>, Fr, u64, Fr) {
    let account_id = Fr::from(42u64);
    let nonce = Fr::from(1u64);
    let balance: u64 = 1_000_000;
    let amount: u64 = 250_000;
    let regulatory_limit: u64 = 500_000;

    let leaf = compute_leaf(account_id, balance, nonce);
    let mut leaves = vec![Fr::from(1u64), Fr::from(2u64), Fr::from(3u64), leaf];
    leaves.resize(8, Fr::from(0u64));
    let tree = SimpleMerkleTree::build(leaves);
    let path = tree.path_for(3);
    let root = tree.root();

    let circuit = ComplianceCircuitWithState::new(
        account_id,
        balance,
        nonce,
        amount,
        path,
        root,
        regulatory_limit,
    );
    let nullifier = circuit.nullifier;

    (circuit, root, regulatory_limit, nullifier)
}

/// EL TEST QUE CIERRA LA LIMITACIÓN MÁS GRAVE DEL PROYECTO.
#[test]
fn ceremony_produces_working_keys_for_the_compliance_circuit() -> anyhow::Result<()> {
    // --- 1. Extraer las matrices R1CS del circuito real ---
    // En modo Setup los valores del testigo son irrelevantes; lo que
    // importa es la ESTRUCTURA del circuito.
    let (circuit_for_setup, _, _, _) = build_scenario();

    let cs = ConstraintSystem::new_ref();
    cs.set_optimization_goal(OptimizationGoal::Constraints);
    cs.set_mode(SynthesisMode::Setup);
    circuit_for_setup.generate_constraints(cs.clone())?;
    cs.finalize();
    let matrices = cs
        .to_matrices()
        .ok_or_else(|| anyhow::anyhow!("no se pudieron generar las matrices del circuito"))?;

    let degree = circuit_degree(&matrices)?;
    println!("Restricciones del circuito : {}", matrices.num_constraints);
    println!("Grado del dominio          : {degree}");

    // --- 2. Fase 1: Powers of Tau, con tres contribuciones ---
    // Cada participante usa su propia semilla. En una ceremonia real
    // serían personas distintas en máquinas distintas (ver la nota de
    // cabecera sobre lo que esto NO demuestra).
    let phase1_root = Phase1CRSElements::root(degree);

    let mut rng_a = rand_chacha::ChaChaRng::seed_from_u64(1001);
    let contrib_1 = Phase1Contribution::make(&mut rng_a, phase1_root.hash(), &phase1_root);

    let mut rng_b = rand_chacha::ChaChaRng::seed_from_u64(2002);
    let contrib_2 =
        Phase1Contribution::make(&mut rng_b, contrib_1.hash(), &contrib_1.new_elements);

    let mut rng_c = rand_chacha::ChaChaRng::seed_from_u64(3003);
    let contrib_3 =
        Phase1Contribution::make(&mut rng_c, contrib_2.hash(), &contrib_2.new_elements);

    println!("Fase 1: 3 contribuciones completadas");

    // Cada contribución debe estar encadenada a la anterior: es lo que
    // impide que alguien inserte una contribución fuera del transcript.
    assert!(
        contrib_3.is_linked_to(&contrib_2.new_elements),
        "la tercera contribucion debe estar encadenada a la segunda"
    );

    // --- 3. Transición: especializar al circuito ---
    let (extra, phase2_root) = transition(&contrib_3.new_elements, &matrices)?;
    println!("Transicion fase 1 -> fase 2 completada");

    // --- 4. Fase 2: tres contribuciones más ---
    let mut rng_d = rand_chacha::ChaChaRng::seed_from_u64(4004);
    let p2_1 = Phase2Contribution::make(&mut rng_d, phase2_root.hash(), &phase2_root);

    let mut rng_e = rand_chacha::ChaChaRng::seed_from_u64(5005);
    let p2_2 = Phase2Contribution::make(&mut rng_e, p2_1.hash(), &p2_1.new_elements);

    let mut rng_f = rand_chacha::ChaChaRng::seed_from_u64(6006);
    let p2_3 = Phase2Contribution::make(&mut rng_f, p2_2.hash(), &p2_2.new_elements);

    println!("Fase 2: 3 contribuciones completadas");

    assert!(
        p2_3.is_linked_to(&p2_2.new_elements),
        "la tercera contribucion de fase 2 debe estar encadenada a la segunda"
    );

    // --- 5. Combinar en una ProvingKey ---
    let pk = combine(&matrices, &contrib_3.new_elements, &p2_3.new_elements, &extra);
    println!("ProvingKey generada por ceremonia");

    // --- 6. LA PRUEBA DE FUEGO: usar esas claves con el circuito real ---
    let (circuit, root, limit, nullifier) = build_scenario();
    let proof = prove_with_state(&pk, circuit, 7)?;
    let is_valid = verify_with_state(&pk.vk, &proof, root, limit, nullifier)?;

    assert!(
        is_valid,
        "CRITICO: las claves generadas por la ceremonia deben producir pruebas verificables"
    );

    println!("=== Prueba Groth16 verificada con claves de ceremonia ===");
    Ok(())
}
