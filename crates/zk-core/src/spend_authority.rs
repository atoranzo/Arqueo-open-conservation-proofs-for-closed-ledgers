//! **Autoridad de gasto**: la pieza que separa una prueba de solvencia de
//! un sistema de liquidación.
//!
//! ## El agujero que esto cierra
//!
//! `circuit_with_state` y `circuit_double_entry` demuestran que quien
//! ejecuta el circuito **conoce** `account_id`, `balance`, `nonce` y el
//! camino de Merkle. **No demuestran que esté autorizado a gastar.**
//!
//! Cualquiera que averigüe esos valores —de una filtración, de un backup,
//! de un empleado— puede generar una prueba válida y mover el dinero. Es
//! el equivalente a un banco donde saber el número de cuenta y el saldo
//! basta para transferir.
//!
//! ## El diseño, siguiendo el modelo de Zcash Sapling
//!
//! ```text
//! sk        = clave de gasto (privada, nunca sale del titular)
//! pk        = H(DOMAIN_PK,   sk)
//! leaf      = H(H(pk, balance), nonce)
//! nullifier = H(H(DOMAIN_NULL, sk), nonce)
//! ```
//!
//! Dos propiedades nuevas, ambas críticas:
//!
//! **1. Autorización.** El circuito exige conocer `sk` tal que
//! `pk = H(DOMAIN_PK, sk)`. Sin la clave no hay prueba, aunque se conozca
//! todo lo demás. La identidad de la cuenta pasa a ser un COMPROMISO a la
//! clave, no un identificador arbitrario.
//!
//! **2. Inobservabilidad del nullifier.** Ahora se deriva de `sk`, no de
//! `pk`. Con el diseño anterior —nullifier derivado de un identificador
//! público— cualquiera podía **precomputar el nullifier de una cuenta
//! ajena** y vigilar el registro de gastados para saber exactamente
//! cuándo esa cuenta mueve dinero. Es una fuga de privacidad grave en un
//! sistema de liquidación: revela el momento de cada operación de
//! cualquier participante.
//!
//! Derivándolo de `sk`, **solo el titular puede calcularlo**. El registro
//! sigue funcionando igual (rechaza repetidos), pero deja de ser un
//! oráculo de vigilancia.
//!
//! ## Coste
//!
//! Dos hashes Poseidon adicionales por prueba. Medido en los tests.
//!
//! ## Lo que este módulo NO resuelve
//!
//! - **No hay rotación de claves ni revocación.** Si `sk` se compromete,
//!   la cuenta se pierde. Un sistema real necesitaría derivación
//!   jerárquica y un mecanismo de recuperación.
//! - **No hay firma sobre el destinatario ni el importe.** El circuito
//!   demuestra autorización para gastar, no un compromiso a los detalles
//!   concretos de esa transferencia. Sin eso, alguien con acceso al
//!   testigo podría reutilizarlo para otro destinatario.
//! - **No hay clave de visualización** para auditoría selectiva.

use ark_crypto_primitives::sponge::Absorb;
use ark_ff::PrimeField;
use ark_r1cs_std::fields::fp::FpVar;
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{ConstraintSystemRef, SynthesisError};

use crate::poseidon_hash::{secure_hash, secure_hash_gadget};

/// Dominio de derivación de la identidad pública desde la clave de gasto.
pub const SPEND_KEY_DOMAIN: u64 = 0x53504B59; // "SPKY"
/// Dominio del nullifier. Se conserva el mismo valor que en el diseño
/// anterior, pero ahora se aplica sobre `sk` en vez de sobre `pk`.
pub const NULLIFIER_DOMAIN: u64 = 0x4E554C4C; // "NULL"

/// Deriva la identidad pública de una cuenta a partir de su clave de
/// gasto: `pk = H(DOMAIN_PK, sk)`.
///
/// La identidad deja de ser un número elegido y pasa a ser un COMPROMISO
/// criptográfico a la clave. Nadie puede reclamar una cuenta sin conocer
/// la preimagen.
pub fn derive_public_id<F: PrimeField + Absorb>(spend_key: F) -> F {
    secure_hash(F::from(SPEND_KEY_DOMAIN), spend_key)
}

/// Nullifier derivado de la CLAVE PRIVADA, no del identificador público.
///
/// Es lo que impide que un observador precompute los nullifiers de
/// cuentas ajenas y vigile cuándo gastan.
pub fn derive_nullifier<F: PrimeField + Absorb>(spend_key: F, nonce: F) -> F {
    let inner = secure_hash(F::from(NULLIFIER_DOMAIN), spend_key);
    secure_hash(inner, nonce)
}

/// EN CIRCUITO: demuestra autoridad de gasto.
///
/// Impone `pk = H(DOMAIN_PK, sk)`, donde `sk` es un testigo privado. Sin
/// conocer `sk` es imposible satisfacer esta restricción, así que la
/// prueba solo puede generarla el titular.
pub fn enforce_spend_authority<F: PrimeField + Absorb>(
    cs: ConstraintSystemRef<F>,
    spend_key_var: &FpVar<F>,
    public_id_var: &FpVar<F>,
) -> Result<(), SynthesisError> {
    let domain = FpVar::<F>::Constant(F::from(SPEND_KEY_DOMAIN));
    let computed = secure_hash_gadget(cs, &domain, spend_key_var)?;
    computed.enforce_equal(public_id_var)?;
    Ok(())
}

/// EN CIRCUITO: calcula el nullifier desde la clave privada y lo ata al
/// valor público declarado.
pub fn enforce_nullifier_from_key<F: PrimeField + Absorb>(
    cs: ConstraintSystemRef<F>,
    spend_key_var: &FpVar<F>,
    nonce_var: &FpVar<F>,
    claimed_nullifier_var: &FpVar<F>,
) -> Result<(), SynthesisError> {
    let domain = FpVar::<F>::Constant(F::from(NULLIFIER_DOMAIN));
    let inner = secure_hash_gadget(cs.clone(), &domain, spend_key_var)?;
    let computed = secure_hash_gadget(cs, &inner, nonce_var)?;
    computed.enforce_equal(claimed_nullifier_var)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bls12_381::Fr;
    use ark_relations::r1cs::ConstraintSystem;

    /// Claves distintas producen identidades distintas.
    #[test]
    fn distinct_keys_yield_distinct_identities() {
        let a = derive_public_id(Fr::from(111u64));
        let b = derive_public_id(Fr::from(222u64));
        assert_ne!(a, b);
    }

    /// LA PROPIEDAD QUE JUSTIFICA EL REDISEÑO DEL NULLIFIER.
    ///
    /// Conocer la identidad pública NO permite calcular el nullifier.
    /// Con el diseño anterior (nullifier derivado de `account_id`, que es
    /// público) cualquiera podía precomputarlo y vigilar el registro para
    /// saber cuándo gasta una cuenta ajena.
    #[test]
    fn nullifier_is_not_derivable_from_public_identity() {
        let sk = Fr::from(31_337u64);
        let nonce = Fr::from(1u64);
        let pk = derive_public_id(sk);

        let real = derive_nullifier(sk, nonce);
        // Un observador solo conoce `pk`. Lo mejor que puede hacer es
        // aplicar la misma construccion sobre `pk`:
        let guess = {
            let inner = secure_hash(Fr::from(NULLIFIER_DOMAIN), pk);
            secure_hash(inner, nonce)
        };
        assert_ne!(
            real, guess,
            "CRITICO: si el nullifier fuera derivable de la identidad publica, \
             cualquiera podria vigilar cuando gasta una cuenta ajena"
        );
    }

    /// Nonces distintos dan nullifiers distintos: la misma cuenta puede
    /// gastar varias veces sin reutilizar el nullifier.
    #[test]
    fn distinct_nonces_yield_distinct_nullifiers() {
        let sk = Fr::from(7u64);
        assert_ne!(
            derive_nullifier(sk, Fr::from(1u64)),
            derive_nullifier(sk, Fr::from(2u64))
        );
    }

    /// EL TEST CLAVE: la restricción de autoridad se satisface con la
    /// clave correcta.
    #[test]
    fn correct_key_satisfies_spend_authority() {
        let sk = Fr::from(999u64);
        let pk = derive_public_id(sk);

        let cs = ConstraintSystem::<Fr>::new_ref();
        let sk_var = FpVar::new_witness(cs.clone(), || Ok(sk)).unwrap();
        let pk_var = FpVar::new_input(cs.clone(), || Ok(pk)).unwrap();

        enforce_spend_authority(cs.clone(), &sk_var, &pk_var).unwrap();
        assert!(cs.is_satisfied().unwrap());
        println!("Restricciones de la autoridad de gasto: {}", cs.num_constraints());
    }

    /// EL TEST DE SOLIDEZ: una clave incorrecta NO satisface la
    /// restricción. Este es el agujero que la pieza cierra.
    #[test]
    fn wrong_key_fails_spend_authority() {
        let real_sk = Fr::from(999u64);
        let pk = derive_public_id(real_sk);
        let attacker_sk = Fr::from(1000u64);

        let cs = ConstraintSystem::<Fr>::new_ref();
        let sk_var = FpVar::new_witness(cs.clone(), || Ok(attacker_sk)).unwrap();
        let pk_var = FpVar::new_input(cs.clone(), || Ok(pk)).unwrap();

        enforce_spend_authority(cs.clone(), &sk_var, &pk_var).unwrap();
        assert!(
            !cs.is_satisfied().unwrap(),
            "CRITICO: conocer la identidad publica de una cuenta NO debe bastar \
             para gastar. Sin la clave privada no debe haber prueba valida."
        );
    }

    /// El nullifier en-circuito coincide con el nativo.
    #[test]
    fn gadget_nullifier_matches_native() {
        let sk = Fr::from(555u64);
        let nonce = Fr::from(3u64);
        let expected = derive_nullifier(sk, nonce);

        let cs = ConstraintSystem::<Fr>::new_ref();
        let sk_var = FpVar::new_witness(cs.clone(), || Ok(sk)).unwrap();
        let nonce_var = FpVar::new_witness(cs.clone(), || Ok(nonce)).unwrap();
        let null_var = FpVar::new_input(cs.clone(), || Ok(expected)).unwrap();

        enforce_nullifier_from_key(cs.clone(), &sk_var, &nonce_var, &null_var).unwrap();
        assert!(cs.is_satisfied().unwrap());
    }

    /// Declarar un nullifier arbitrario no cuela.
    #[test]
    fn forged_nullifier_fails() {
        let sk = Fr::from(555u64);
        let nonce = Fr::from(3u64);

        let cs = ConstraintSystem::<Fr>::new_ref();
        let sk_var = FpVar::new_witness(cs.clone(), || Ok(sk)).unwrap();
        let nonce_var = FpVar::new_witness(cs.clone(), || Ok(nonce)).unwrap();
        let forged = FpVar::new_input(cs.clone(), || Ok(Fr::from(31_337u64))).unwrap();

        enforce_nullifier_from_key(cs.clone(), &sk_var, &nonce_var, &forged).unwrap();
        assert!(!cs.is_satisfied().unwrap());
    }
}
