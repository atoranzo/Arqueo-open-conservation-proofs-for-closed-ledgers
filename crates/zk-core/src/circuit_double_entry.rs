//! Circuito de **partida doble** (double-entry bookkeeping): demuestra la
//! transición de estado completa de una transferencia, no solo que el
//! emisor tenga fondos.
//!
//! ## La carencia que esto cierra
//!
//! `ComplianceCircuitWithState` demuestra que el emisor tiene saldo
//! suficiente y está bajo el límite regulatorio. Pero **no demuestra qué
//! pasa con el dinero**: nada ata el adeudo del origen al abono del
//! destino. Un supervisor bancario preguntaría exactamente eso, porque la
//! invariante contable fundamental no es "el emisor podía pagar", sino
//! "el dinero se conserva".
//!
//! Este circuito lo demuestra:
//!
//! ```text
//! saldo_emisor_nuevo   = saldo_emisor   - importe   (ADEUDO)
//! saldo_receptor_nuevo = saldo_receptor + importe   (ABONO)
//! ```
//!
//! ## Por qué la conservación NO es vacua aquí
//!
//! Escribir `debit == credit` con una sola variable `importe` no
//! demostraría nada: sería una tautología. Lo que hace esto real es que
//! **ambos saldos están comprometidos en el árbol de Merkle** y **ambas
//! raíces son públicas**:
//!
//! - El saldo antiguo del emisor está atado a `root_old`.
//! - El saldo antiguo del receptor está atado a la raíz intermedia.
//! - Los saldos nuevos determinan `root_new`.
//!
//! Así que el mismo `importe` que se resta de un saldo comprometido se
//! suma a otro saldo comprometido, y el resultado se refleja en una raíz
//! pública. No hay forma de inflar dinero sin romper alguna de las tres
//! ataduras.
//!
//! ## La secuencia de actualización, y por qué el orden importa
//!
//! El árbol se actualiza en DOS pasos encadenados:
//!
//! 1. Verificar la hoja del emisor contra `root_old`.
//! 2. Recalcular con la hoja modificada → `root_mid`.
//! 3. Verificar la hoja del receptor contra **`root_mid`**, no contra
//!    `root_old`.
//! 4. Recalcular con la hoja modificada → debe dar `root_new`.
//!
//! El paso 3 es la sutileza: tras actualizar al emisor, el árbol ya
//! cambió. Si emisor y receptor comparten ancestros, los hermanos del
//! camino del receptor son distintos de los que había antes. Verificar
//! contra `root_old` sería un error que produciría un circuito
//! insatisfacible en la mitad de los casos (los que comparten subárbol) y
//! satisfacible en el resto — el peor tipo de bug: intermitente.
//!
//! ## Qué NO demuestra este circuito
//!
//! - **No demuestra que el receptor exista de antemano.** Demuestra que
//!   su hoja está en el árbol; crear cuentas es otra operación.
//! - **No impide una autotransferencia** de forma explícita. Si emisor y
//!   receptor son la misma cuenta, el paso 3 fallará (la hoja del receptor
//!   ya no coincide con la que se comprometió), así que se rechaza — pero
//!   por accidente estructural, no por una restricción dedicada.
//! - **No gestiona comisiones ni creación de dinero.** Un banco central
//!   emitiendo moneda rompería la conservación deliberadamente, y eso
//!   necesitaría un circuito distinto con una entrada pública de emisión.

use ark_bls12_381::Fr;
use ark_crypto_primitives::sponge::Absorb;
use ark_ff::PrimeField;
use ark_r1cs_std::alloc::AllocVar;
use ark_r1cs_std::boolean::Boolean;
use ark_r1cs_std::eq::EqGadget;
use ark_r1cs_std::fields::fp::FpVar;
use ark_r1cs_std::fields::FieldVar;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};

use crate::circuit::enforce_range;
use crate::merkle::{compute_merkle_root, MerklePath, TREE_DEPTH};
use crate::nullifier::{compute_nullifier, enforce_nullifier_computation};
use crate::poseidon_hash::secure_hash_gadget;
use crate::proof_system::{
    prove_generic, setup_generic, verify_generic, ComplianceProof, ComplianceProvingKey,
    ComplianceVerifyingKey,
};
use crate::ZkCoreError;

/// Testigos de una de las dos partes de la transferencia.
#[derive(Clone, Debug)]
pub struct AccountWitness<F: PrimeField> {
    pub account_id: F,
    pub balance: u64,
    pub nonce: F,
    pub merkle_path: MerklePath<F>,
}

#[derive(Clone)]
pub struct DoubleEntryCircuit<F: PrimeField> {
    // --- Testigos privados: emisor ---
    pub sender: Option<AccountWitness<F>>,
    // --- Testigos privados: receptor ---
    pub receiver: Option<AccountWitness<F>>,
    // --- Importe de la transferencia (privado) ---
    pub amount: Option<u64>,

    // --- Entradas públicas ---
    /// Raíz del árbol ANTES de la transferencia.
    pub root_old: F,
    /// Raíz del árbol DESPUÉS de la transferencia.
    pub root_new: F,
    pub regulatory_limit: F,
    /// Nullifier del emisor. Se calcula en `new()` a partir de sus
    /// testigos, para que sea imposible por construcción declarar uno
    /// que no corresponda.
    pub nullifier: F,
}

impl<F: PrimeField + Absorb> DoubleEntryCircuit<F> {
    pub fn new(
        sender: AccountWitness<F>,
        receiver: AccountWitness<F>,
        amount: u64,
        root_old: F,
        root_new: F,
        regulatory_limit: u64,
    ) -> Self {
        let nullifier = compute_nullifier(sender.account_id, sender.nonce);
        Self {
            sender: Some(sender),
            receiver: Some(receiver),
            amount: Some(amount),
            root_old,
            root_new,
            regulatory_limit: F::from(regulatory_limit),
            nullifier,
        }
    }

    /// Circuito vacío con la MISMA forma, para el setup de claves. Igual
    /// que en `circuit_with_state`: en fase de setup no hay testigos
    /// reales, pero la estructura de restricciones debe ser idéntica.
    pub fn empty_for_setup() -> Self {
        Self {
            sender: None,
            receiver: None,
            amount: None,
            root_old: F::zero(),
            root_new: F::zero(),
            regulatory_limit: F::zero(),
            nullifier: F::zero(),
        }
    }
}

/// Aloja los testigos de una cuenta, con relleno de ceros si no hay
/// testigo (fase de setup).
fn alloc_account<F: PrimeField>(
    cs: ConstraintSystemRef<F>,
    witness: &Option<AccountWitness<F>>,
) -> Result<(FpVar<F>, FpVar<F>, FpVar<F>, Vec<FpVar<F>>, Vec<Boolean<F>>), SynthesisError> {
    let (id, balance, nonce, siblings, is_right) = match witness {
        Some(w) => (
            w.account_id,
            F::from(w.balance),
            w.nonce,
            w.merkle_path.siblings.clone(),
            w.merkle_path.is_right.clone(),
        ),
        None => (
            F::zero(),
            F::zero(),
            F::zero(),
            vec![F::zero(); TREE_DEPTH],
            vec![false; TREE_DEPTH],
        ),
    };

    let id_var = FpVar::new_witness(cs.clone(), || Ok(id))?;
    let balance_var = FpVar::new_witness(cs.clone(), || Ok(balance))?;
    let nonce_var = FpVar::new_witness(cs.clone(), || Ok(nonce))?;
    let siblings_var: Vec<FpVar<F>> = siblings
        .iter()
        .map(|s| FpVar::new_witness(cs.clone(), || Ok(*s)))
        .collect::<Result<_, _>>()?;
    let is_right_var: Vec<Boolean<F>> = is_right
        .iter()
        .map(|b| Boolean::new_witness(cs.clone(), || Ok(*b)))
        .collect::<Result<_, _>>()?;

    Ok((id_var, balance_var, nonce_var, siblings_var, is_right_var))
}

/// Compromiso de hoja EN CIRCUITO: H(H(id, balance), nonce).
fn leaf_gadget<F: PrimeField + Absorb>(
    cs: ConstraintSystemRef<F>,
    id: &FpVar<F>,
    balance: &FpVar<F>,
    nonce: &FpVar<F>,
) -> Result<FpVar<F>, SynthesisError> {
    let inner = secure_hash_gadget(cs.clone(), id, balance)?;
    secure_hash_gadget(cs, &inner, nonce)
}

impl<F: PrimeField + Absorb> ConstraintSynthesizer<F> for DoubleEntryCircuit<F> {
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        // --- Testigos ---
        let (s_id, s_bal, s_nonce, s_siblings, s_bits) = alloc_account(cs.clone(), &self.sender)?;
        let (r_id, r_bal, r_nonce, r_siblings, r_bits) = alloc_account(cs.clone(), &self.receiver)?;

        let amount_var = FpVar::new_witness(cs.clone(), || {
            self.amount
                .map(F::from)
                .ok_or(SynthesisError::AssignmentMissing)
                .or(Ok(F::zero()))
        })?;

        // --- Entradas públicas, EN ESTE ORDEN (ver
        //     `public_inputs_for_double_entry`) ---
        let root_old_var = FpVar::new_input(cs.clone(), || Ok(self.root_old))?;
        let root_new_var = FpVar::new_input(cs.clone(), || Ok(self.root_new))?;
        let limit_var = FpVar::new_input(cs.clone(), || Ok(self.regulatory_limit))?;
        let nullifier_var = FpVar::new_input(cs.clone(), || Ok(self.nullifier))?;

        // ============ 1. El emisor está en el árbol antiguo ============
        let s_leaf_old = leaf_gadget(cs.clone(), &s_id, &s_bal, &s_nonce)?;
        let computed_root_old =
            compute_merkle_root(cs.clone(), &s_leaf_old, &s_siblings, &s_bits)?;
        computed_root_old.enforce_equal(&root_old_var)?;

        // ============ 2. ADEUDO: el emisor pierde el importe ============
        let s_bal_new = &s_bal - &amount_var;
        let s_nonce_new = &s_nonce + FpVar::constant(F::one());
        let s_leaf_new = leaf_gadget(cs.clone(), &s_id, &s_bal_new, &s_nonce_new)?;

        // Raíz INTERMEDIA: el árbol tras actualizar solo al emisor.
        let root_mid = compute_merkle_root(cs.clone(), &s_leaf_new, &s_siblings, &s_bits)?;

        // ============ 3. El receptor está en el árbol INTERMEDIO ============
        // Contra root_mid, NO contra root_old: el árbol ya cambió. Ver la
        // nota de cabecera sobre por qué esto importa.
        let r_leaf_old = leaf_gadget(cs.clone(), &r_id, &r_bal, &r_nonce)?;
        let computed_root_mid =
            compute_merkle_root(cs.clone(), &r_leaf_old, &r_siblings, &r_bits)?;
        computed_root_mid.enforce_equal(&root_mid)?;

        // ============ 4. ABONO: el receptor gana el MISMO importe ============
        // Aquí está la conservación: es la misma variable `amount_var` que
        // se restó arriba. No puede haber discrepancia entre adeudo y
        // abono porque son literalmente el mismo valor del circuito.
        let r_bal_new = &r_bal + &amount_var;
        let r_leaf_new = leaf_gadget(cs.clone(), &r_id, &r_bal_new, &r_nonce)?;

        let computed_root_new =
            compute_merkle_root(cs.clone(), &r_leaf_new, &r_siblings, &r_bits)?;
        computed_root_new.enforce_equal(&root_new_var)?;

        // ============ 5. Nullifier del emisor ============
        enforce_nullifier_computation(cs.clone(), &s_id, &s_nonce, &nullifier_var)?;

        // ============ 6. Solvencia, límite y rangos ============
        enforce_range(&s_bal)?;
        enforce_range(&r_bal)?;
        enforce_range(&amount_var)?;
        enforce_range(&limit_var)?;

        // amount <= balance del emisor
        enforce_range(&s_bal_new)?;
        // amount <= limite regulatorio
        let diff_limit = &limit_var - &amount_var;
        enforce_range(&diff_limit)?;
        // El saldo del receptor no desborda los 64 bits tras el abono.
        enforce_range(&r_bal_new)?;

        Ok(())
    }
}

// ---------------------------------------------------------------------
// Wrappers Groth16
// ---------------------------------------------------------------------

pub fn setup_double_entry(
    rng_seed: u64,
) -> Result<(ComplianceProvingKey, ComplianceVerifyingKey), ZkCoreError> {
    setup_generic(DoubleEntryCircuit::<Fr>::empty_for_setup(), rng_seed)
}

pub fn prove_double_entry(
    pk: &ComplianceProvingKey,
    circuit: DoubleEntryCircuit<Fr>,
    rng_seed: u64,
) -> Result<ComplianceProof, ZkCoreError> {
    prove_generic(pk, circuit, rng_seed)
}

/// Inputs públicos EN EL ORDEN EXACTO en que `generate_constraints` los
/// alloca: `root_old`, `root_new`, `regulatory_limit`, `nullifier`.
/// Reordenar las líneas `new_input(...)` sin actualizar esto rompería
/// toda verificación de forma silenciosa y confusa.
pub fn public_inputs_for_double_entry(
    root_old: Fr,
    root_new: Fr,
    regulatory_limit: u64,
    nullifier: Fr,
) -> Vec<Fr> {
    vec![
        root_old,
        root_new,
        Fr::from(regulatory_limit),
        nullifier,
    ]
}

pub fn verify_double_entry(
    vk: &ComplianceVerifyingKey,
    proof: &ComplianceProof,
    root_old: Fr,
    root_new: Fr,
    regulatory_limit: u64,
    nullifier: Fr,
) -> Result<bool, ZkCoreError> {
    let inputs =
        public_inputs_for_double_entry(root_old, root_new, regulatory_limit, nullifier);
    verify_generic(vk, proof, &inputs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::merkle::{leaf_commitment, SimpleMerkleTree};
    use ark_relations::r1cs::ConstraintSystem;

    const SENDER_IDX: usize = 3;
    const RECEIVER_IDX: usize = 5;

    /// Escenario de transferencia completo.
    struct Scenario {
        circuit: DoubleEntryCircuit<Fr>,
        root_old: Fr,
        root_new: Fr,
        nullifier: Fr,
        limit: u64,
    }

    /// Construye un escenario de transferencia.
    ///
    /// ⚠️ EL PUNTO DELICADO: el camino del receptor se toma del árbol
    /// INTERMEDIO (tras actualizar al emisor), no del original. Emisor y
    /// receptor comparten ancestros en los niveles altos del árbol, así
    /// que actualizar al emisor cambia hermanos del camino del receptor.
    /// Tomarlos del árbol original haría fallar el circuito de forma
    /// intermitente — solo cuando los caminos se cruzan.
    ///
    /// `credited` permite acreditar al receptor una cantidad DISTINTA de
    /// la debitada, para poder construir el test que rompe la
    /// conservación deliberadamente.
    fn build_scenario(
        sender_balance: u64,
        receiver_balance: u64,
        amount: u64,
        credited: u64,
        limit: u64,
    ) -> Scenario {
        let sender_id = Fr::from(1001u64);
        let sender_nonce = Fr::from(7u64);
        let receiver_id = Fr::from(2002u64);
        let receiver_nonce = Fr::from(3u64);

        // --- Estado inicial: 8 cuentas ---
        let mut leaves: Vec<Fr> = (0..8u64).map(Fr::from).collect();
        leaves[SENDER_IDX] =
            leaf_commitment(sender_id, Fr::from(sender_balance), sender_nonce);
        leaves[RECEIVER_IDX] =
            leaf_commitment(receiver_id, Fr::from(receiver_balance), receiver_nonce);

        let tree_old = SimpleMerkleTree::build(leaves.clone());
        let root_old = tree_old.root();
        let sender_path = tree_old.path_for(SENDER_IDX);

        // --- Árbol intermedio: solo el emisor actualizado ---
        let mut leaves_mid = leaves.clone();
        leaves_mid[SENDER_IDX] = leaf_commitment(
            sender_id,
            Fr::from(sender_balance - amount),
            sender_nonce + Fr::from(1u64),
        );
        let tree_mid = SimpleMerkleTree::build(leaves_mid.clone());
        // El camino del receptor sale de AQUI, no del arbol original.
        let receiver_path = tree_mid.path_for(RECEIVER_IDX);

        // --- Árbol final: receptor acreditado ---
        let mut leaves_new = leaves_mid;
        leaves_new[RECEIVER_IDX] = leaf_commitment(
            receiver_id,
            Fr::from(receiver_balance + credited),
            receiver_nonce,
        );
        let tree_new = SimpleMerkleTree::build(leaves_new);
        let root_new = tree_new.root();

        let sender = AccountWitness {
            account_id: sender_id,
            balance: sender_balance,
            nonce: sender_nonce,
            merkle_path: sender_path,
        };
        let receiver = AccountWitness {
            account_id: receiver_id,
            balance: receiver_balance,
            nonce: receiver_nonce,
            merkle_path: receiver_path,
        };

        let circuit =
            DoubleEntryCircuit::new(sender, receiver, amount, root_old, root_new, limit);
        let nullifier = circuit.nullifier;

        Scenario {
            circuit,
            root_old,
            root_new,
            nullifier,
            limit,
        }
    }

    /// Comprueba si el sistema de restricciones se satisface con estos
    /// testigos. Mucho más rápido que generar una prueba real, y suficiente
    /// para los casos negativos.
    fn is_satisfied(circuit: DoubleEntryCircuit<Fr>) -> bool {
        let cs = ConstraintSystem::<Fr>::new_ref();
        match circuit.generate_constraints(cs.clone()) {
            Ok(()) => cs.is_satisfied().unwrap_or(false),
            Err(_) => false,
        }
    }

    /// Una transferencia legítima satisface el circuito, e informa del
    /// tamaño real del circuito.
    #[test]
    fn valid_transfer_satisfies_constraints() {
        let s = build_scenario(1_000_000, 50_000, 250_000, 250_000, 500_000);

        let cs = ConstraintSystem::<Fr>::new_ref();
        s.circuit
            .generate_constraints(cs.clone())
            .expect("la sintesis no deberia fallar");
        println!("Restricciones del circuito de partida doble: {}", cs.num_constraints());

        assert!(
            cs.is_satisfied().unwrap(),
            "una transferencia valida deberia satisfacer el circuito"
        );
    }

    /// EL TEST QUE DA SENTIDO A TODA LA PIEZA.
    ///
    /// El receptor recibe 10.000 más de lo que el emisor perdió. Es
    /// creación de dinero de la nada — exactamente lo que la partida doble
    /// existe para impedir, y lo que el circuito anterior NO detectaba
    /// porque ni siquiera miraba al receptor.
    #[test]
    fn money_creation_is_rejected() {
        let s = build_scenario(1_000_000, 50_000, 250_000, 260_000, 500_000);
        assert!(
            !is_satisfied(s.circuit),
            "CRITICO: acreditar mas de lo debitado es creacion de dinero y debe rechazarse"
        );
    }

    /// El caso simétrico: el receptor recibe menos de lo debitado. Es
    /// destrucción de dinero, igual de inaceptable en una cámara de
    /// compensación (ese descuadre tendría que ir a alguna cuenta).
    #[test]
    fn money_destruction_is_rejected() {
        let s = build_scenario(1_000_000, 50_000, 250_000, 240_000, 500_000);
        assert!(
            !is_satisfied(s.circuit),
            "CRITICO: acreditar menos de lo debitado descuadra el balance y debe rechazarse"
        );
    }

    /// Gastar más del saldo disponible.
    #[test]
    fn insufficient_balance_is_rejected() {
        // El emisor tiene 100.000 e intenta enviar 250.000.
        let sender_id = Fr::from(1001u64);
        let sender_nonce = Fr::from(7u64);
        let receiver_id = Fr::from(2002u64);
        let receiver_nonce = Fr::from(3u64);
        let (sender_balance, amount) = (100_000u64, 250_000u64);

        let mut leaves: Vec<Fr> = (0..8u64).map(Fr::from).collect();
        leaves[SENDER_IDX] =
            leaf_commitment(sender_id, Fr::from(sender_balance), sender_nonce);
        leaves[RECEIVER_IDX] = leaf_commitment(receiver_id, Fr::from(50_000u64), receiver_nonce);

        let tree_old = SimpleMerkleTree::build(leaves.clone());
        let root_old = tree_old.root();
        let sender_path = tree_old.path_for(SENDER_IDX);

        // El saldo nuevo del emisor da la vuelta en el campo (wrap).
        let mut leaves_mid = leaves.clone();
        leaves_mid[SENDER_IDX] = leaf_commitment(
            sender_id,
            Fr::from(sender_balance) - Fr::from(amount),
            sender_nonce + Fr::from(1u64),
        );
        let tree_mid = SimpleMerkleTree::build(leaves_mid.clone());
        let receiver_path = tree_mid.path_for(RECEIVER_IDX);

        let mut leaves_new = leaves_mid;
        leaves_new[RECEIVER_IDX] =
            leaf_commitment(receiver_id, Fr::from(50_000u64 + amount), receiver_nonce);
        let root_new = SimpleMerkleTree::build(leaves_new).root();

        let circuit = DoubleEntryCircuit::new(
            AccountWitness {
                account_id: sender_id,
                balance: sender_balance,
                nonce: sender_nonce,
                merkle_path: sender_path,
            },
            AccountWitness {
                account_id: receiver_id,
                balance: 50_000,
                nonce: receiver_nonce,
                merkle_path: receiver_path,
            },
            amount,
            root_old,
            root_new,
            500_000,
        );

        assert!(
            !is_satisfied(circuit),
            "CRITICO: gastar mas del saldo debe rechazarse"
        );
    }

    /// Superar el límite regulatorio.
    #[test]
    fn over_regulatory_limit_is_rejected() {
        let s = build_scenario(1_000_000, 50_000, 750_000, 750_000, 500_000);
        assert!(
            !is_satisfied(s.circuit),
            "CRITICO: superar el limite regulatorio debe rechazarse"
        );
    }

    /// Declarar una raíz final que no corresponde a la transición real.
    #[test]
    fn wrong_declared_new_root_is_rejected() {
        let mut s = build_scenario(1_000_000, 50_000, 250_000, 250_000, 500_000);
        s.circuit.root_new = Fr::from(999_999u64);
        assert!(
            !is_satisfied(s.circuit),
            "CRITICO: una raiz final incorrecta debe rechazarse"
        );
    }

    /// Declarar un nullifier falsificado.
    #[test]
    fn forged_nullifier_is_rejected() {
        let mut s = build_scenario(1_000_000, 50_000, 250_000, 250_000, 500_000);
        s.circuit.nullifier = Fr::from(31_337u64);
        assert!(
            !is_satisfied(s.circuit),
            "CRITICO: un nullifier falsificado debe rechazarse"
        );
    }

    /// PRUEBA REAL de extremo a extremo. Lenta: ejecutar en release.
    ///
    /// `cargo test -p zk-core --release double_entry_end_to_end -- --nocapture`
    #[test]
    fn double_entry_end_to_end_proof() {
        let s = build_scenario(1_000_000, 50_000, 250_000, 250_000, 500_000);

        let (pk, vk) = setup_double_entry(1).expect("setup no deberia fallar");
        let proof = prove_double_entry(&pk, s.circuit, 2).expect("prove no deberia fallar");
        let ok = verify_double_entry(&vk, &proof, s.root_old, s.root_new, s.limit, s.nullifier)
            .expect("verify no deberia devolver error");

        assert!(
            ok,
            "una transferencia de partida doble valida deberia verificar"
        );
    }
}
