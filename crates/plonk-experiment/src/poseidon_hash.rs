//! Hash Poseidon (permutación Hades) para el backend PLONK-KZG, en sus
//! dos formas: nativa y en-circuito.
//!
//! ## Lo que dusk trae y los otros backends no
//!
//! `dusk-poseidon` incluye **separación de dominio en el propio
//! framework de esponja**, vía el enum `Domain`:
//!
//! - `Domain::Merkle2`: exactamente 2 entradas, 1 salida. Hecho a medida
//!   para un árbol binario, y el propio framework rechaza cualquier otra
//!   aridad.
//! - `Domain::Other`: entrada libre. Su separador vale cero, pero la
//!   documentación del crate señala que **el io-pattern queda codificado
//!   en el tag**, así que dos hashes con distinto número de entradas ya
//!   están separados aunque compartan dominio.
//!
//! En Groth16, Halo2 y STARK esa separación la implementamos nosotros
//! anteponiendo una constante. Aquí el framework la aporta.
//!
//! **Aun así usamos constantes de dominio explícitas** para la hoja y el
//! nullifier: mantiene la coherencia con los otros tres backends y no
//! hace depender la seguridad de una sutileza del framework que un
//! lector podría pasar por alto.
//!
//! ## Diferencia de diseño respecto a los otros backends
//!
//! Allí la hoja es `H(H(id, balance), nonce)` — dos hashes anidados,
//! porque el gadget disponible era de aridad 2. Aquí la esponja acepta
//! cualquier número de entradas, así que la hoja es **un solo hash de 4
//! elementos**: más barato y más directo.
//!
//! Las pruebas de los cuatro backends no son intercambiables en ningún
//! caso (parámetros de Poseidon distintos), así que esta diferencia no
//! rompe nada que ya funcionara.

use dusk_plonk::prelude::*;
use dusk_poseidon::{Domain, Hash, HashGadget};

/// Constante de dominio de las hojas del árbol de cuentas.
pub const LEAF_DOMAIN: u64 = 0x4C454146; // "LEAF"
/// Constante de dominio de los nullifiers. La misma que en los otros
/// tres backends, para que la correspondencia sea evidente.
pub const NULLIFIER_DOMAIN: u64 = 0x4E554C4C; // "NULL"

// ---------------------------------------------------------------------
// Versión nativa: para calcular los valores esperados fuera del circuito
// ---------------------------------------------------------------------

/// Hash de un nivel del árbol: `Domain::Merkle2`, aridad fija 2.
pub fn native_hash_pair(left: BlsScalar, right: BlsScalar) -> BlsScalar {
    Hash::digest(Domain::Merkle2, &[left, right])[0]
}

/// Compromiso de una hoja de cuenta.
pub fn native_leaf(account_id: BlsScalar, balance: BlsScalar, nonce: BlsScalar) -> BlsScalar {
    Hash::digest(
        Domain::Other,
        &[BlsScalar::from(LEAF_DOMAIN), account_id, balance, nonce],
    )[0]
}

/// Nullifier con separación de dominio.
pub fn native_nullifier(account_id: BlsScalar, nonce: BlsScalar) -> BlsScalar {
    Hash::digest(
        Domain::Other,
        &[BlsScalar::from(NULLIFIER_DOMAIN), account_id, nonce],
    )[0]
}

// ---------------------------------------------------------------------
// Versión en-circuito
// ---------------------------------------------------------------------

/// Hash de un nivel del árbol, dentro del circuito.
pub fn gadget_hash_pair(composer: &mut Composer, left: Witness, right: Witness) -> Witness {
    let inputs = [left, right];
    let mut hasher = HashGadget::new(Domain::Merkle2);
    hasher.update(&inputs);
    hasher.finalize(composer)[0]
}

/// Compromiso de hoja, dentro del circuito.
pub fn gadget_leaf(
    composer: &mut Composer,
    account_id: Witness,
    balance: Witness,
    nonce: Witness,
) -> Witness {
    let domain = composer.append_constant(BlsScalar::from(LEAF_DOMAIN));
    let inputs = [domain, account_id, balance, nonce];
    let mut hasher = HashGadget::new(Domain::Other);
    hasher.update(&inputs);
    hasher.finalize(composer)[0]
}

/// Nullifier, dentro del circuito.
pub fn gadget_nullifier(composer: &mut Composer, account_id: Witness, nonce: Witness) -> Witness {
    let domain = composer.append_constant(BlsScalar::from(NULLIFIER_DOMAIN));
    let inputs = [domain, account_id, nonce];
    let mut hasher = HashGadget::new(Domain::Other);
    hasher.update(&inputs);
    hasher.finalize(composer)[0]
}

// ---------------------------------------------------------------------
// Circuito de prueba: conocimiento de preimagen
// ---------------------------------------------------------------------

/// Tamaño del SRS. Poseidon consume bastantes más puertas que una suma.
pub const CAPACITY: usize = 1 << 12;

/// Demuestra conocimiento de `(left, right)` cuyo hash es el público.
#[derive(Default, Debug)]
pub struct HashPreimageCircuit {
    pub left: BlsScalar,
    pub right: BlsScalar,
    pub expected: BlsScalar,
}

impl HashPreimageCircuit {
    pub fn new(left: u64, right: u64) -> Self {
        let l = BlsScalar::from(left);
        let r = BlsScalar::from(right);
        Self {
            left: l,
            right: r,
            expected: native_hash_pair(l, r),
        }
    }
}

impl Circuit for HashPreimageCircuit {
    fn circuit(&self, composer: &mut Composer) -> Result<(), Error> {
        let w_left = composer.append_witness(self.left);
        let w_right = composer.append_witness(self.right);

        let computed = gadget_hash_pair(composer, w_left, w_right);

        let w_expected = composer.append_public(self.expected);
        composer.assert_equal(computed, w_expected);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    /// LO PRIMERO QUE HAY QUE COMPROBAR: que el hash en-circuito y el
    /// nativo dan lo mismo. Si difirieran, todo lo demás sería ruido.
    ///
    /// Se comprueba indirectamente: el circuito calcula el hash con el
    /// gadget y lo compara con el público, que se calculó de forma
    /// nativa. Si no coincidieran, `prove` fallaría.
    #[test]
    fn gadget_and_native_hash_agree() {
        let mut rng = StdRng::seed_from_u64(0xfeedface);
        let pp = PublicParameters::setup(CAPACITY, &mut rng).expect("setup");
        let (prover, verifier) =
            Compiler::compile::<HashPreimageCircuit>(&pp, b"zk-ssl-hash").expect("compile");

        let circuit = HashPreimageCircuit::new(12345, 67890);
        let (proof, pi) = prover
            .prove(&mut rng, &circuit)
            .expect("el hash del gadget deberia coincidir con el nativo");
        verifier.verify(&proof, &pi).expect("deberia verificar");

        println!("Puertas del circuito de hash: {}", circuit.size());
    }

    /// Declarar un hash público que no corresponde debe fallar.
    #[test]
    fn wrong_declared_hash_fails() {
        let mut rng = StdRng::seed_from_u64(0xfeedface);
        let pp = PublicParameters::setup(CAPACITY, &mut rng).expect("setup");
        let (prover, _) =
            Compiler::compile::<HashPreimageCircuit>(&pp, b"zk-ssl-hash").expect("compile");

        let broken = HashPreimageCircuit {
            left: BlsScalar::from(1u64),
            right: BlsScalar::from(2u64),
            expected: BlsScalar::from(999u64), // no es el hash real
        };

        assert!(
            prover.prove(&mut rng, &broken).is_err(),
            "CRITICO: un hash declarado incorrecto no deberia producir prueba"
        );
    }

    /// El hash es sensible al orden de los operandos.
    #[test]
    fn hash_is_order_sensitive() {
        let a = BlsScalar::from(111u64);
        let b = BlsScalar::from(222u64);
        assert_ne!(
            native_hash_pair(a, b),
            native_hash_pair(b, a),
            "invertir los operandos deberia cambiar el hash"
        );
    }

    /// La separación de dominio funciona: una hoja y un nullifier con los
    /// mismos valores de entrada dan hashes distintos.
    #[test]
    fn domain_separation_works() {
        let id = BlsScalar::from(42u64);
        let nonce = BlsScalar::from(7u64);

        // Mismo id y nonce; solo cambia el propósito.
        let leaf = native_leaf(id, BlsScalar::from(0u64), nonce);
        let null = native_nullifier(id, nonce);
        assert_ne!(
            leaf, null,
            "CRITICO: hoja y nullifier deben estar separados por dominio"
        );
    }

    /// Cuentas o nonces distintos dan nullifiers distintos.
    #[test]
    fn nullifiers_are_distinct() {
        let n1 = native_nullifier(BlsScalar::from(1u64), BlsScalar::from(1u64));
        let n2 = native_nullifier(BlsScalar::from(2u64), BlsScalar::from(1u64));
        let n3 = native_nullifier(BlsScalar::from(1u64), BlsScalar::from(2u64));
        assert_ne!(n1, n2);
        assert_ne!(n1, n3);
        assert_ne!(n2, n3);
    }
}
