//! **Módulo de privacidad y cumplimiento**: revelación selectiva para
//! auditoría regulatoria.
//!
//! ## El hueco que cierra
//!
//! El sistema demuestra cumplimiento del límite regulatorio dentro del
//! circuito, pero **un supervisor no podía auditar nada**. No había forma
//! de que verificase el saldo de una cuenta concreta sin que la entidad
//! le enseñara todo su estado.
//!
//! Ese es el bloqueo real para adopción institucional: un sistema
//! perfectamente privado en el que el regulador no puede comprobar nada
//! no es adoptable, por mucha matemática que tenga detrás.
//!
//! ## El modelo elegido: revelación VOLUNTARIA, no custodia de claves
//!
//! Existen dos formas de resolver esto:
//!
//! **A) Clave de visualización en poder del supervisor.** El regulador
//! tiene una clave que le permite descifrar la actividad de las cuentas.
//! Es el modelo de las "view keys" con escrow.
//!
//! **B) Revelación voluntaria.** El titular produce una prueba de que su
//! saldo es exactamente X, dirigida a quien se la pida. El supervisor
//! aprende ese dato y **nada más**.
//!
//! **Este módulo implementa B**, y la razón es de riesgo sistémico: una
//! clave de visualización custodiada es un punto único de fallo. Quien la
//! robe —un atacante, un empleado, un estado hostil— ve la actividad de
//! todo el sistema, retroactivamente y sin dejar rastro. Con revelación
//! voluntaria **no hay nada que robar**: cada revelación es un acto
//! deliberado, puntual y trazable.
//!
//! La contrapartida honesta: **el supervisor depende de la cooperación
//! del titular**. Si una entidad se niega a revelar, la coerción tiene
//! que venir de fuera del sistema (requerimiento legal, sanción,
//! suspensión de licencia) — igual que hoy con el secreto bancario. Este
//! módulo no sustituye a la autoridad legal; le da una herramienta
//! criptográfica para verificar lo que se le entrega.
//!
//! ## Qué demuestra
//!
//! Sin revelar la clave de gasto ni ningún otro dato del árbol:
//!
//! 1. **Titularidad**: quien firma conoce la clave de la cuenta.
//! 2. **La cuenta está en el árbol** con la raíz declarada.
//! 3. **Su saldo es exactamente el revelado.**
//!
//! ## La variante de rango: revelar menos todavía
//!
//! `AuditRangeCircuit` demuestra `saldo >= umbral` **sin revelar el saldo
//! exacto**. Es lo que necesita un supervisor para comprobar requisitos
//! de capital o reservas mínimas: le basta saber que se cumple el
//! mínimo, no cuánto exactamente.
//!
//! Es revelación selectiva en su forma más estricta: se demuestra el
//! hecho regulatorio relevante y ni un bit más.

use ark_bls12_381::Fr;
use ark_crypto_primitives::sponge::Absorb;
use ark_ff::PrimeField;
use ark_r1cs_std::alloc::AllocVar;
use ark_r1cs_std::boolean::Boolean;
use ark_r1cs_std::eq::EqGadget;
use ark_r1cs_std::fields::fp::FpVar;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};

use crate::circuit::enforce_range;
use crate::merkle::{compute_merkle_root, MerklePath, TREE_DEPTH};
use crate::poseidon_hash::secure_hash_gadget;
use crate::proof_system::{
    prove_generic, setup_generic, verify_generic, ComplianceProof, ComplianceProvingKey,
    ComplianceVerifyingKey,
};
use crate::spend_authority::enforce_spend_authority;
use crate::ZkCoreError;

/// Testigos comunes a las dos formas de auditoría.
#[derive(Clone, Debug)]
pub struct AuditWitness<F: PrimeField> {
    pub spend_key: F,
    pub balance: u64,
    pub nonce: F,
    pub merkle_path: MerklePath<F>,
}

/// Aloja los testigos y devuelve la hoja calculada, tras comprobar
/// titularidad y pertenencia.
///
/// Compartido por los dos circuitos: la diferencia entre revelar el saldo
/// exacto y revelar solo que supera un umbral es únicamente lo que se
/// hace DESPUÉS con el saldo.
fn common_audit_constraints<F: PrimeField + Absorb>(
    cs: ConstraintSystemRef<F>,
    witness: &Option<AuditWitness<F>>,
    public_id_var: &FpVar<F>,
    root_var: &FpVar<F>,
) -> Result<FpVar<F>, SynthesisError> {
    let (key, balance, nonce) = match witness {
        Some(w) => (w.spend_key, F::from(w.balance), w.nonce),
        None => (F::zero(), F::zero(), F::zero()),
    };
    let key_var = FpVar::new_witness(cs.clone(), || Ok(key))?;
    let balance_var = FpVar::new_witness(cs.clone(), || Ok(balance))?;
    let nonce_var = FpVar::new_witness(cs.clone(), || Ok(nonce))?;

    let (siblings, is_right) = match witness {
        Some(w) => (
            w.merkle_path.siblings.clone(),
            w.merkle_path.is_right.clone(),
        ),
        None => (vec![F::zero(); TREE_DEPTH], vec![false; TREE_DEPTH]),
    };
    let siblings_var = siblings
        .iter()
        .map(|s| FpVar::new_witness(cs.clone(), || Ok(*s)))
        .collect::<Result<Vec<_>, _>>()?;
    let bits_var = is_right
        .iter()
        .map(|b| Boolean::new_witness(cs.clone(), || Ok(*b)))
        .collect::<Result<Vec<_>, _>>()?;

    // 1. TITULARIDAD: sin la clave no hay revelación posible. Impide que
    //    un tercero fabrique revelaciones sobre cuentas ajenas.
    enforce_spend_authority(cs.clone(), &key_var, public_id_var)?;

    // 2. PERTENENCIA: la cuenta está en el árbol con la raíz declarada.
    let leaf = {
        let inner = secure_hash_gadget(cs.clone(), public_id_var, &balance_var)?;
        secure_hash_gadget(cs.clone(), &inner, &nonce_var)?
    };
    let computed = compute_merkle_root(cs.clone(), &leaf, &siblings_var, &bits_var)?;
    computed.enforce_equal(root_var)?;

    enforce_range(&balance_var)?;
    Ok(balance_var)
}

// =====================================================================
// Revelación del saldo exacto
// =====================================================================

/// Demuestra que el saldo de una cuenta es **exactamente** el revelado.
#[derive(Clone)]
pub struct AuditDisclosureCircuit<F: PrimeField> {
    pub witness: Option<AuditWitness<F>>,
    /// Raíz del estado auditado.
    pub root: F,
    /// Identidad pública de la cuenta. El supervisor sabe A QUIÉN audita.
    pub public_id: F,
    /// El saldo que se revela.
    pub disclosed_balance: F,
}

impl<F: PrimeField + Absorb> AuditDisclosureCircuit<F> {
    pub fn new(witness: AuditWitness<F>, root: F, public_id: F) -> Self {
        let disclosed_balance = F::from(witness.balance);
        Self {
            witness: Some(witness),
            root,
            public_id,
            disclosed_balance,
        }
    }

    pub fn empty_for_setup() -> Self {
        Self {
            witness: None,
            root: F::zero(),
            public_id: F::zero(),
            disclosed_balance: F::zero(),
        }
    }
}

impl<F: PrimeField + Absorb> ConstraintSynthesizer<F> for AuditDisclosureCircuit<F> {
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        let root_var = FpVar::new_input(cs.clone(), || Ok(self.root))?;
        let id_var = FpVar::new_input(cs.clone(), || Ok(self.public_id))?;
        let disclosed_var = FpVar::new_input(cs.clone(), || Ok(self.disclosed_balance))?;

        let balance_var = common_audit_constraints(cs, &self.witness, &id_var, &root_var)?;

        // 3. El saldo revelado es EXACTAMENTE el real.
        balance_var.enforce_equal(&disclosed_var)?;
        Ok(())
    }
}

// =====================================================================
// Revelación de rango: "tengo al menos X"
// =====================================================================

/// Demuestra que el saldo **supera un umbral**, sin revelar cuánto.
///
/// Es lo que necesita un supervisor para comprobar requisitos de capital
/// o reservas mínimas: le basta saber que se cumple el mínimo.
#[derive(Clone)]
pub struct AuditRangeCircuit<F: PrimeField> {
    pub witness: Option<AuditWitness<F>>,
    pub root: F,
    pub public_id: F,
    /// Umbral que debe superarse. Público.
    pub threshold: F,
}

impl<F: PrimeField + Absorb> AuditRangeCircuit<F> {
    pub fn new(witness: AuditWitness<F>, root: F, public_id: F, threshold: u64) -> Self {
        Self {
            witness: Some(witness),
            root,
            public_id,
            threshold: F::from(threshold),
        }
    }

    pub fn empty_for_setup() -> Self {
        Self {
            witness: None,
            root: F::zero(),
            public_id: F::zero(),
            threshold: F::zero(),
        }
    }
}

impl<F: PrimeField + Absorb> ConstraintSynthesizer<F> for AuditRangeCircuit<F> {
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        let root_var = FpVar::new_input(cs.clone(), || Ok(self.root))?;
        let id_var = FpVar::new_input(cs.clone(), || Ok(self.public_id))?;
        let threshold_var = FpVar::new_input(cs.clone(), || Ok(self.threshold))?;

        let balance_var = common_audit_constraints(cs, &self.witness, &id_var, &root_var)?;

        // 3. saldo >= umbral, por el mismo mecanismo que la solvencia: si
        //    la resta diera la vuelta en el campo, no cabría en 64 bits.
        let excess = &balance_var - &threshold_var;
        enforce_range(&excess)?;
        Ok(())
    }
}

// ---------------------------------------------------------------------
// Wrappers Groth16
// ---------------------------------------------------------------------

pub fn setup_disclosure(
    rng_seed: u64,
) -> Result<(ComplianceProvingKey, ComplianceVerifyingKey), ZkCoreError> {
    setup_generic(AuditDisclosureCircuit::<Fr>::empty_for_setup(), rng_seed)
}

pub fn prove_disclosure(
    pk: &ComplianceProvingKey,
    circuit: AuditDisclosureCircuit<Fr>,
    rng_seed: u64,
) -> Result<ComplianceProof, ZkCoreError> {
    prove_generic(pk, circuit, rng_seed)
}

pub fn verify_disclosure(
    vk: &ComplianceVerifyingKey,
    proof: &ComplianceProof,
    root: Fr,
    public_id: Fr,
    disclosed_balance: u64,
) -> Result<bool, ZkCoreError> {
    verify_generic(
        vk,
        proof,
        &[root, public_id, Fr::from(disclosed_balance)],
    )
}

pub fn setup_range_audit(
    rng_seed: u64,
) -> Result<(ComplianceProvingKey, ComplianceVerifyingKey), ZkCoreError> {
    setup_generic(AuditRangeCircuit::<Fr>::empty_for_setup(), rng_seed)
}

pub fn prove_range_audit(
    pk: &ComplianceProvingKey,
    circuit: AuditRangeCircuit<Fr>,
    rng_seed: u64,
) -> Result<ComplianceProof, ZkCoreError> {
    prove_generic(pk, circuit, rng_seed)
}

pub fn verify_range_audit(
    vk: &ComplianceVerifyingKey,
    proof: &ComplianceProof,
    root: Fr,
    public_id: Fr,
    threshold: u64,
) -> Result<bool, ZkCoreError> {
    verify_generic(vk, proof, &[root, public_id, Fr::from(threshold)])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuit_settlement::{account_id_from_key, account_leaf};
    use crate::merkle::SimpleMerkleTree;
    use ark_ff::Zero;
    use ark_relations::r1cs::ConstraintSystem;

    const ACCOUNT_IDX: usize = 3;
    const SK: u64 = 0xA11CE;

    fn scenario(balance: u64) -> (AuditWitness<Fr>, Fr, Fr) {
        let key = Fr::from(SK);
        let public_id = account_id_from_key(key);
        let nonce = Fr::zero();

        let mut leaves: Vec<Fr> = (0..8u64).map(Fr::from).collect();
        leaves[ACCOUNT_IDX] = account_leaf(public_id, balance, nonce);
        let tree = SimpleMerkleTree::build(leaves);

        let witness = AuditWitness {
            spend_key: key,
            balance,
            nonce,
            merkle_path: tree.path_for(ACCOUNT_IDX),
        };
        (witness, tree.root(), public_id)
    }

    fn satisfied<C: ConstraintSynthesizer<Fr>>(circuit: C) -> bool {
        let cs = ConstraintSystem::<Fr>::new_ref();
        match circuit.generate_constraints(cs.clone()) {
            Ok(()) => cs.is_satisfied().unwrap_or(false),
            Err(_) => false,
        }
    }

    /// EL TEST CLAVE: revelar el saldo real satisface el circuito.
    #[test]
    fn honest_disclosure_satisfies() {
        let (w, root, id) = scenario(1_000_000);
        let circuit = AuditDisclosureCircuit::new(w, root, id);
        let cs = ConstraintSystem::<Fr>::new_ref();
        circuit.clone().generate_constraints(cs.clone()).unwrap();
        println!("Restricciones de revelacion: {}", cs.num_constraints());
        assert!(cs.is_satisfied().unwrap());
    }

    /// **EL TEST QUE HACE ÚTIL LA AUDITORÍA.**
    ///
    /// Revelar un saldo distinto del real debe fallar. Sin esto, una
    /// entidad podría declarar al supervisor el saldo que le conviniera y
    /// la "auditoría" no comprobaría nada.
    #[test]
    fn lying_about_the_balance_is_rejected() {
        let (w, root, id) = scenario(1_000_000);
        let mut circuit = AuditDisclosureCircuit::new(w, root, id);
        circuit.disclosed_balance = Fr::from(9_999_999u64);
        assert!(
            !satisfied(circuit),
            "CRITICO: declarar un saldo falso al supervisor debe rechazarse"
        );
    }

    /// **NADIE PUEDE REVELAR POR OTRO.**
    ///
    /// Un tercero que conoce la identidad pública y el saldo de una
    /// cuenta ajena no puede fabricar una revelación sobre ella. Sin la
    /// restricción de titularidad, cualquiera podría generar
    /// "revelaciones" falsas sobre terceros — o filtrar datos ajenos con
    /// apariencia de prueba.
    #[test]
    fn third_party_cannot_disclose_someone_elses_balance() {
        let (mut w, root, id) = scenario(1_000_000);
        w.spend_key = Fr::from(0x1337u64); // no es el titular
        let circuit = AuditDisclosureCircuit::new(w, root, id);
        assert!(
            !satisfied(circuit),
            "CRITICO: solo el titular puede revelar su saldo"
        );
    }

    /// Declarar una raíz distinta a la del estado auditado debe fallar:
    /// impide revelar sobre un estado antiguo favorable.
    #[test]
    fn disclosure_against_wrong_root_is_rejected() {
        let (w, _, id) = scenario(1_000_000);
        let circuit = AuditDisclosureCircuit::new(w, Fr::from(999u64), id);
        assert!(!satisfied(circuit));
    }

    /// La prueba de rango: superar el umbral satisface.
    #[test]
    fn balance_above_threshold_satisfies() {
        let (w, root, id) = scenario(1_000_000);
        let circuit = AuditRangeCircuit::new(w, root, id, 500_000);
        let cs = ConstraintSystem::<Fr>::new_ref();
        circuit.clone().generate_constraints(cs.clone()).unwrap();
        println!("Restricciones de rango: {}", cs.num_constraints());
        assert!(cs.is_satisfied().unwrap());
    }

    /// **NO SE PUEDE FINGIR SOLVENCIA.**
    ///
    /// Un saldo por debajo del umbral no puede producir la prueba. Es lo
    /// que permite a un supervisor comprobar reservas mínimas sin ver el
    /// saldo exacto.
    #[test]
    fn balance_below_threshold_is_rejected() {
        let (w, root, id) = scenario(100_000);
        let circuit = AuditRangeCircuit::new(w, root, id, 500_000);
        assert!(
            !satisfied(circuit),
            "CRITICO: no debe poder demostrarse un minimo de reservas que no se cumple"
        );
    }

    /// El caso frontera: saldo exactamente igual al umbral, que sí
    /// cumple el requisito.
    #[test]
    fn balance_exactly_at_threshold_satisfies() {
        let (w, root, id) = scenario(500_000);
        let circuit = AuditRangeCircuit::new(w, root, id, 500_000);
        assert!(satisfied(circuit));
    }

    /// PRUEBA REAL de revelación.
    #[test]
    fn disclosure_end_to_end_proof() {
        let (w, root, id) = scenario(1_000_000);
        let circuit = AuditDisclosureCircuit::new(w, root, id);
        let (pk, vk) = setup_disclosure(1).expect("setup");
        let proof = prove_disclosure(&pk, circuit, 2).expect("prove");
        assert!(verify_disclosure(&vk, &proof, root, id, 1_000_000).expect("verify"));
    }

    /// PRUEBA REAL de rango.
    #[test]
    fn range_audit_end_to_end_proof() {
        let (w, root, id) = scenario(1_000_000);
        let circuit = AuditRangeCircuit::new(w, root, id, 500_000);
        let (pk, vk) = setup_range_audit(1).expect("setup");
        let proof = prove_range_audit(&pk, circuit, 2).expect("prove");
        assert!(verify_range_audit(&vk, &proof, root, id, 500_000).expect("verify"));
    }
}
