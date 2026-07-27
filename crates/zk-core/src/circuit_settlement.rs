//! **El circuito de liquidación.** Partida doble con autoridad de gasto.
//!
//! Sustituye a `circuit_double_entry`, que demostraba la conservación del
//! dinero pero **no que quien firma esté autorizado a gastar**. Ese
//! circuito se conserva como referencia de la evolución del diseño; este
//! es el que debe usarse.
//!
//! ## Lo que demuestra
//!
//! Sin revelar identidades, saldos ni importes:
//!
//! 1. **Autoridad**: el emisor conoce la clave de gasto `sk` de la cuenta
//!    de origen (`pk = H(DOMAIN_PK, sk)`).
//! 2. **Solvencia**: `amount <= balance` y `amount <= limite`.
//! 3. **Pertenencia**: ambas cuentas existen en el árbol.
//! 4. **Conservación**: lo debitado al emisor es exactamente lo
//!    acreditado al receptor.
//! 5. **Transición**: el árbol pasa de `root_old` a `root_new`.
//! 6. **Unicidad**: el nullifier se deriva de `sk`, así que solo el
//!    titular puede calcularlo y el registro puede rechazar repeticiones
//!    sin revelar de quién son.
//!
//! ## La asimetría emisor/receptor es deliberada
//!
//! **Solo el emisor demuestra autoridad.** El receptor aparece con su
//! identidad pública `pk_receiver`, sin clave.
//!
//! No es un descuido: **recibir dinero no requiere permiso**. Exigir la
//! firma del receptor rompería el modelo — no se puede transferir a
//! alguien que no está presente para autorizar. Es la misma asimetría que
//! en Zcash y, de hecho, en cualquier sistema de pagos real: se firma
//! para gastar, no para cobrar.
//!
//! ## Lo que sigue faltando para producción
//!
//! - **La prueba no se compromete al destinatario ni al importe.**
//!   Demuestra autoridad para gastar y que la transición es coherente,
//!   pero alguien con acceso al testigo completo podría construir otra
//!   transferencia distinta con la misma clave. Un sistema real ataría la
//!   autorización a los detalles concretos de ESA operación.
//! - **No hay rotación ni revocación de claves.** Si `sk` se compromete,
//!   la cuenta se pierde.
//! - **La no-pertenencia del nullifier no es demostrable en circuito**:
//!   depende del registro externo (`persistent_nullifier_registry`), que
//!   es de un solo nodo.
//! - **No hay revelación selectiva** para auditoría del supervisor.

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
use crate::poseidon_hash::secure_hash_gadget;
use crate::proof_system::{
    prove_generic, setup_generic, verify_generic, ComplianceProof, ComplianceProvingKey,
    ComplianceVerifyingKey,
};
use crate::nullifier_tree::{
    enforce_insert_unspent, NullifierPath, NULLIFIER_TREE_DEPTH,
};
use crate::spend_authority::{
    derive_nullifier, derive_public_id, enforce_nullifier_from_key, enforce_spend_authority,
};
use crate::ZkCoreError;

/// Testigos del emisor. Incluye la CLAVE DE GASTO.
#[derive(Clone, Debug)]
pub struct SenderWitness<F: PrimeField> {
    /// Clave de gasto. Nunca sale del titular y nunca se revela.
    pub spend_key: F,
    pub balance: u64,
    pub nonce: F,
    pub merkle_path: MerklePath<F>,
}

/// Testigos del receptor. **Sin clave**: recibir no requiere autorización.
#[derive(Clone, Debug)]
pub struct ReceiverWitness<F: PrimeField> {
    /// Identidad pública del receptor, ya derivada de SU clave (que él
    /// conserva y que aquí no aparece).
    pub public_id: F,
    pub balance: u64,
    pub nonce: F,
    pub merkle_path: MerklePath<F>,
}

#[derive(Clone)]
pub struct SettlementCircuit<F: PrimeField> {
    pub sender: Option<SenderWitness<F>>,
    pub receiver: Option<ReceiverWitness<F>>,
    pub amount: Option<u64>,

    pub root_old: F,
    pub root_new: F,
    /// Raíz del árbol de nullifiers ANTES de esta operación.
    pub nullifier_root_old: F,
    /// Raíz DESPUÉS de insertar el nullifier de esta operación.
    pub nullifier_root_new: F,
    pub regulatory_limit: F,
    /// Camino del nullifier en su árbol. PRIVADO.
    pub nullifier_path: Option<NullifierPath<F>>,
}

impl<F: PrimeField + Absorb> SettlementCircuit<F> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        sender: SenderWitness<F>,
        receiver: ReceiverWitness<F>,
        amount: u64,
        root_old: F,
        root_new: F,
        nullifier_root_old: F,
        nullifier_root_new: F,
        nullifier_path: NullifierPath<F>,
        regulatory_limit: u64,
    ) -> Self {
        Self {
            sender: Some(sender),
            receiver: Some(receiver),
            amount: Some(amount),
            root_old,
            root_new,
            nullifier_root_old,
            nullifier_root_new,
            regulatory_limit: F::from(regulatory_limit),
            nullifier_path: Some(nullifier_path),
        }
    }

    pub fn empty_for_setup() -> Self {
        Self {
            sender: None,
            receiver: None,
            amount: None,
            root_old: F::zero(),
            root_new: F::zero(),
            nullifier_root_old: F::zero(),
            nullifier_root_new: F::zero(),
            regulatory_limit: F::zero(),
            nullifier_path: None,
        }
    }
}

/// Compromiso de hoja: `H(H(public_id, balance), nonce)`.
fn leaf_gadget<F: PrimeField + Absorb>(
    cs: ConstraintSystemRef<F>,
    public_id: &FpVar<F>,
    balance: &FpVar<F>,
    nonce: &FpVar<F>,
) -> Result<FpVar<F>, SynthesisError> {
    let inner = secure_hash_gadget(cs.clone(), public_id, balance)?;
    secure_hash_gadget(cs, &inner, nonce)
}

fn alloc_path<F: PrimeField>(
    cs: ConstraintSystemRef<F>,
    path: Option<&MerklePath<F>>,
) -> Result<(Vec<FpVar<F>>, Vec<Boolean<F>>), SynthesisError> {
    let (siblings, is_right) = match path {
        Some(p) => (p.siblings.clone(), p.is_right.clone()),
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
    Ok((siblings_var, bits_var))
}

impl<F: PrimeField + Absorb> ConstraintSynthesizer<F> for SettlementCircuit<F> {
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        // ===== Testigos del emisor, incluida la clave =====
        let (s_key, s_bal, s_nonce) = match &self.sender {
            Some(s) => (s.spend_key, F::from(s.balance), s.nonce),
            None => (F::zero(), F::zero(), F::zero()),
        };
        let s_key_var = FpVar::new_witness(cs.clone(), || Ok(s_key))?;
        let s_bal_var = FpVar::new_witness(cs.clone(), || Ok(s_bal))?;
        let s_nonce_var = FpVar::new_witness(cs.clone(), || Ok(s_nonce))?;
        let (s_siblings, s_bits) =
            alloc_path(cs.clone(), self.sender.as_ref().map(|s| &s.merkle_path))?;

        // ===== Testigos del receptor, SIN clave =====
        let (r_id, r_bal, r_nonce) = match &self.receiver {
            Some(r) => (r.public_id, F::from(r.balance), r.nonce),
            None => (F::zero(), F::zero(), F::zero()),
        };
        let r_id_var = FpVar::new_witness(cs.clone(), || Ok(r_id))?;
        let r_bal_var = FpVar::new_witness(cs.clone(), || Ok(r_bal))?;
        let r_nonce_var = FpVar::new_witness(cs.clone(), || Ok(r_nonce))?;
        let (r_siblings, r_bits) =
            alloc_path(cs.clone(), self.receiver.as_ref().map(|r| &r.merkle_path))?;

        let amount_var = FpVar::new_witness(cs.clone(), || {
            Ok(self.amount.map(F::from).unwrap_or_else(F::zero))
        })?;

        // ===== Entradas públicas, EN ESTE ORDEN =====
        //
        // El NULLIFIER ES PÚBLICO, y debe serlo. Una versión anterior lo
        // hizo testigo privado pensando que exponía menos información;
        // fue un error de diseño que un test destapó: **la capa necesita
        // conocerlo para mantener su propio árbol** y poder generar los
        // caminos de no-pertenencia de operaciones futuras. Sin él, el
        // árbol nunca crece y la no-pertenencia se vuelve vacua.
        //
        // Publicarlo es seguro: al derivarse de `sk`, es indistinguible y
        // nadie puede vincularlo a una cuenta. Es el mismo diseño de
        // Zcash, donde los nullifiers son públicos por esta razón.
        let root_old_var = FpVar::new_input(cs.clone(), || Ok(self.root_old))?;
        let root_new_var = FpVar::new_input(cs.clone(), || Ok(self.root_new))?;
        let null_root_old_var = FpVar::new_input(cs.clone(), || Ok(self.nullifier_root_old))?;
        let null_root_new_var = FpVar::new_input(cs.clone(), || Ok(self.nullifier_root_new))?;
        let limit_var = FpVar::new_input(cs.clone(), || Ok(self.regulatory_limit))?;

        let nullifier_value = match &self.sender {
            Some(s) => derive_nullifier(s.spend_key, s.nonce),
            None => F::zero(),
        };
        let nullifier_var = FpVar::new_input(cs.clone(), || Ok(nullifier_value))?;

        // Camino del nullifier en su árbol.
        let (null_siblings, null_bits) = {
            let (sibs, bits) = match &self.nullifier_path {
                Some(p) => (p.siblings.clone(), p.is_right.clone()),
                None => (
                    vec![F::zero(); NULLIFIER_TREE_DEPTH],
                    vec![false; NULLIFIER_TREE_DEPTH],
                ),
            };
            let s_var = sibs
                .iter()
                .map(|s| FpVar::new_witness(cs.clone(), || Ok(*s)))
                .collect::<Result<Vec<_>, _>>()?;
            let b_var = bits
                .iter()
                .map(|b| Boolean::new_witness(cs.clone(), || Ok(*b)))
                .collect::<Result<Vec<_>, _>>()?;
            (s_var, b_var)
        };

        // ===== 1. AUTORIDAD DE GASTO =====
        // La identidad del emisor se DERIVA de su clave. Sin conocer `sk`
        // es imposible satisfacer esto, por mucho que se conozcan el
        // saldo, el nonce y el camino de Merkle.
        let s_id_var = secure_hash_gadget(
            cs.clone(),
            &FpVar::Constant(F::from(crate::spend_authority::SPEND_KEY_DOMAIN)),
            &s_key_var,
        )?;
        enforce_spend_authority(cs.clone(), &s_key_var, &s_id_var)?;

        // ===== 2. El emisor está en el árbol antiguo =====
        let s_leaf_old = leaf_gadget(cs.clone(), &s_id_var, &s_bal_var, &s_nonce_var)?;
        let computed_old = compute_merkle_root(cs.clone(), &s_leaf_old, &s_siblings, &s_bits)?;
        computed_old.enforce_equal(&root_old_var)?;

        // ===== 3. ADEUDO =====
        let s_bal_new = &s_bal_var - &amount_var;
        let s_nonce_new = &s_nonce_var + FpVar::constant(F::one());
        let s_leaf_new = leaf_gadget(cs.clone(), &s_id_var, &s_bal_new, &s_nonce_new)?;
        let root_mid = compute_merkle_root(cs.clone(), &s_leaf_new, &s_siblings, &s_bits)?;

        // ===== 4. El receptor está en el árbol INTERMEDIO =====
        let r_leaf_old = leaf_gadget(cs.clone(), &r_id_var, &r_bal_var, &r_nonce_var)?;
        let computed_mid = compute_merkle_root(cs.clone(), &r_leaf_old, &r_siblings, &r_bits)?;
        computed_mid.enforce_equal(&root_mid)?;

        // ===== 5. ABONO: el MISMO importe =====
        let r_bal_new = &r_bal_var + &amount_var;
        let r_leaf_new = leaf_gadget(cs.clone(), &r_id_var, &r_bal_new, &r_nonce_var)?;
        let computed_new = compute_merkle_root(cs.clone(), &r_leaf_new, &r_siblings, &r_bits)?;
        computed_new.enforce_equal(&root_new_var)?;

        // ===== 6. Nullifier DESDE LA CLAVE =====
        // Solo el titular puede calcularlo: el registro de gastados deja
        // de ser un oráculo de vigilancia.
        enforce_nullifier_from_key(cs.clone(), &s_key_var, &s_nonce_var, &nullifier_var)?;

        // ===== 7. NO-PERTENENCIA + INSERCIÓN =====
        // Demuestra que este nullifier NO se había gastado y que la nueva
        // raíz refleja su inserción. El doble gasto pasa de ser
        // "detectable por una base de datos externa" a ser
        // MATEMÁTICAMENTE IMPOSIBLE.
        enforce_insert_unspent(
            cs.clone(),
            &nullifier_var,
            &null_siblings,
            &null_bits,
            &null_root_old_var,
            &null_root_new_var,
        )?;

        // ===== 8. Solvencia, límite y rangos =====
        enforce_range(&s_bal_var)?;
        enforce_range(&r_bal_var)?;
        enforce_range(&amount_var)?;
        enforce_range(&limit_var)?;
        enforce_range(&s_bal_new)?; // amount <= balance
        enforce_range(&r_bal_new)?; // el abono no desborda
        let diff_limit = &limit_var - &amount_var;
        enforce_range(&diff_limit)?; // amount <= limite

        Ok(())
    }
}

/// Calcula la hoja de una cuenta a partir de su identidad pública.
pub fn account_leaf<F: PrimeField + Absorb>(public_id: F, balance: u64, nonce: F) -> F {
    use crate::poseidon_hash::secure_hash;
    secure_hash(secure_hash(public_id, F::from(balance)), nonce)
}

/// Identidad pública de una cuenta desde su clave de gasto.
pub fn account_id_from_key<F: PrimeField + Absorb>(spend_key: F) -> F {
    derive_public_id(spend_key)
}

// ---------------------------------------------------------------------
// Wrappers Groth16
// ---------------------------------------------------------------------

pub fn setup_settlement(
    rng_seed: u64,
) -> Result<(ComplianceProvingKey, ComplianceVerifyingKey), ZkCoreError> {
    setup_generic(SettlementCircuit::<Fr>::empty_for_setup(), rng_seed)
}

pub fn prove_settlement(
    pk: &ComplianceProvingKey,
    circuit: SettlementCircuit<Fr>,
    rng_seed: u64,
) -> Result<ComplianceProof, ZkCoreError> {
    prove_generic(pk, circuit, rng_seed)
}

/// Inputs públicos en el orden exacto en que se allocan.
///
/// El nullifier NO aparece: queda comprometido dentro del árbol de
/// nullifiers, cuyas dos raíces sí son públicas.
#[allow(clippy::too_many_arguments)]
pub fn public_inputs_for_settlement(
    root_old: Fr,
    root_new: Fr,
    nullifier_root_old: Fr,
    nullifier_root_new: Fr,
    regulatory_limit: u64,
    nullifier: Fr,
) -> Vec<Fr> {
    vec![
        root_old,
        root_new,
        nullifier_root_old,
        nullifier_root_new,
        Fr::from(regulatory_limit),
        nullifier,
    ]
}

#[allow(clippy::too_many_arguments)]
pub fn verify_settlement(
    vk: &ComplianceVerifyingKey,
    proof: &ComplianceProof,
    root_old: Fr,
    root_new: Fr,
    nullifier_root_old: Fr,
    nullifier_root_new: Fr,
    regulatory_limit: u64,
    nullifier: Fr,
) -> Result<bool, ZkCoreError> {
    let inputs = public_inputs_for_settlement(
        root_old,
        root_new,
        nullifier_root_old,
        nullifier_root_new,
        regulatory_limit,
        nullifier,
    );
    verify_generic(vk, proof, &inputs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::merkle::SimpleMerkleTree;
    use ark_relations::r1cs::ConstraintSystem;

    const SENDER_IDX: usize = 3;
    const RECEIVER_IDX: usize = 5;

    struct Scenario {
        circuit: SettlementCircuit<Fr>,
        root_old: Fr,
        root_new: Fr,
        null_root_old: Fr,
        null_root_new: Fr,
        nullifier: Fr,
        limit: u64,
    }

    /// Construye una transferencia. El camino del receptor sale del árbol
    /// INTERMEDIO (tras actualizar al emisor) — ver la nota en
    /// `circuit_double_entry`.
    fn build(sender_balance: u64, amount: u64, credited: u64, limit: u64) -> Scenario {
        let s_key = Fr::from(0xDEADBEEFu64);
        let s_id = account_id_from_key(s_key);
        let s_nonce = Fr::from(7u64);
        let r_id = account_id_from_key(Fr::from(0xCAFEu64));
        let r_bal = 50_000u64;
        let r_nonce = Fr::from(3u64);

        let mut leaves: Vec<Fr> = (0..8u64).map(Fr::from).collect();
        leaves[SENDER_IDX] = account_leaf(s_id, sender_balance, s_nonce);
        leaves[RECEIVER_IDX] = account_leaf(r_id, r_bal, r_nonce);

        let tree_old = SimpleMerkleTree::build(leaves.clone());
        let root_old = tree_old.root();
        let sender_path = tree_old.path_for(SENDER_IDX);

        let mut leaves_mid = leaves.clone();
        leaves_mid[SENDER_IDX] = account_leaf(
            s_id,
            sender_balance.wrapping_sub(amount),
            s_nonce + Fr::from(1u64),
        );
        let tree_mid = SimpleMerkleTree::build(leaves_mid.clone());
        let receiver_path = tree_mid.path_for(RECEIVER_IDX);

        let mut leaves_new = leaves_mid;
        leaves_new[RECEIVER_IDX] = account_leaf(r_id, r_bal + credited, r_nonce);
        let root_new = SimpleMerkleTree::build(leaves_new).root();

        // --- Arbol de nullifiers: parte vacio, se inserta el de esta
        //     operacion ---
        let nullifier = crate::spend_authority::derive_nullifier(s_key, s_nonce);
        let pos = crate::nullifier_tree::nullifier_position(nullifier);
        let null_path = NullifierPath::<Fr>::for_empty_tree(pos);
        let null_root_old = crate::nullifier_tree::empty_root::<Fr>();
        let null_root_new = crate::nullifier_tree::climb(nullifier, &null_path);

        let circuit = SettlementCircuit::new(
            SenderWitness {
                spend_key: s_key,
                balance: sender_balance,
                nonce: s_nonce,
                merkle_path: sender_path,
            },
            ReceiverWitness {
                public_id: r_id,
                balance: r_bal,
                nonce: r_nonce,
                merkle_path: receiver_path,
            },
            amount,
            root_old,
            root_new,
            null_root_old,
            null_root_new,
            null_path,
            limit,
        );

        Scenario {
            circuit,
            root_old,
            root_new,
            null_root_old,
            null_root_new,
            nullifier,
            limit,
        }
    }

    fn satisfied(circuit: SettlementCircuit<Fr>) -> bool {
        let cs = ConstraintSystem::<Fr>::new_ref();
        match circuit.generate_constraints(cs.clone()) {
            Ok(()) => cs.is_satisfied().unwrap_or(false),
            Err(_) => false,
        }
    }

    /// EL TEST CLAVE: una transferencia autorizada y coherente satisface
    /// el circuito.
    #[test]
    fn authorized_valid_transfer_satisfies() {
        let s = build(1_000_000, 250_000, 250_000, 500_000);
        let cs = ConstraintSystem::<Fr>::new_ref();
        s.circuit.clone().generate_constraints(cs.clone()).unwrap();
        println!("Restricciones del circuito de liquidacion: {}", cs.num_constraints());
        assert!(cs.is_satisfied().unwrap());
    }

    /// **EL TEST QUE JUSTIFICA TODA LA PIEZA.**
    ///
    /// Un atacante conoce TODO de la cuenta —identidad pública, saldo,
    /// nonce, camino de Merkle— pero no la clave de gasto. Intenta
    /// transferir con una clave inventada.
    ///
    /// Con `circuit_double_entry` esto FUNCIONARÍA: aquel circuito no
    /// pedía ninguna clave. Aquí debe fallar.
    #[test]
    fn attacker_without_spend_key_cannot_transfer() {
        let mut s = build(1_000_000, 250_000, 250_000, 500_000);

        // El atacante sustituye la clave por una suya. Todo lo demas
        // (saldo, nonce, caminos, raices) sigue siendo correcto.
        if let Some(sender) = s.circuit.sender.as_mut() {
            sender.spend_key = Fr::from(0x1337u64);
        }

        assert!(
            !satisfied(s.circuit),
            "CRITICO: conocer los datos publicos de una cuenta NO debe bastar \
             para gastar. Sin la clave de gasto no debe haber prueba valida."
        );
    }

    /// Creación de dinero: el receptor recibe más de lo debitado.
    #[test]
    fn money_creation_is_rejected() {
        let s = build(1_000_000, 250_000, 260_000, 500_000);
        assert!(!satisfied(s.circuit));
    }

    /// Destrucción de dinero.
    #[test]
    fn money_destruction_is_rejected() {
        let s = build(1_000_000, 250_000, 240_000, 500_000);
        assert!(!satisfied(s.circuit));
    }

    /// Saldo insuficiente.
    #[test]
    fn insufficient_balance_is_rejected() {
        let s = build(100_000, 250_000, 250_000, 500_000);
        assert!(!satisfied(s.circuit));
    }

    /// Límite regulatorio superado.
    #[test]
    fn over_regulatory_limit_is_rejected() {
        let s = build(1_000_000, 750_000, 750_000, 500_000);
        assert!(!satisfied(s.circuit));
    }

    /// Declarar una raíz de nullifiers nueva que no corresponde a la
    /// inserción real debe fallar.
    #[test]
    fn forged_nullifier_root_is_rejected() {
        let mut s = build(1_000_000, 250_000, 250_000, 500_000);
        s.circuit.nullifier_root_new = Fr::from(31_337u64);
        assert!(!satisfied(s.circuit));
    }

    /// **EL TEST QUE CIERRA EL DOBLE GASTO EN EL CIRCUITO COMPLETO.**
    ///
    /// Se intenta gastar declarando como raíz "antigua" una en la que el
    /// nullifier YA está insertado. La no-pertenencia falla.
    ///
    /// Antes esto dependía de que una base de datos externa lo
    /// detectara; ahora es imposible sin romper la matemática.
    #[test]
    fn double_spend_is_rejected_by_the_circuit() {
        let mut s = build(1_000_000, 250_000, 250_000, 500_000);
        // La raiz "antigua" pasa a ser la que YA contiene el nullifier.
        s.circuit.nullifier_root_old = s.null_root_new;
        assert!(
            !satisfied(s.circuit),
            "CRITICO: gastar dos veces debe ser imposible en el circuito, \
             no solo detectable por un registro externo"
        );
    }

    /// PRUEBA REAL de extremo a extremo. Ejecutar en release.
    #[test]
    fn settlement_end_to_end_proof() {
        let s = build(1_000_000, 250_000, 250_000, 500_000);
        let (pk, vk) = setup_settlement(1).expect("setup");
        let proof = prove_settlement(&pk, s.circuit, 2).expect("prove");
        let ok = verify_settlement(
            &vk,
            &proof,
            s.root_old,
            s.root_new,
            s.null_root_old,
            s.null_root_new,
            s.limit,
            s.nullifier,
        )
        .expect("verify");
        assert!(ok, "una liquidacion autorizada y valida deberia verificar");
    }
}
