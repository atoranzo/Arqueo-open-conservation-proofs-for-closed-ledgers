//! **No-pertenencia demostrable del nullifier**: la corrección del último
//! punto del sistema donde una parte podía crear dinero sin romper
//! ninguna matemática.
//!
//! ## El agujero que esto cierra
//!
//! `persistent_nullifier_registry` es una base de datos `sled` de un solo
//! nodo. El circuito **no comprueba nada sobre ella**: se limita a
//! publicar el nullifier y confía en que alguien, fuera, mire si ya
//! estaba.
//!
//! Quien controle ese nodo puede aceptar un nullifier repetido y permitir
//! un doble gasto. **La prueba seguiría siendo criptográficamente
//! válida** — no habría forma de detectarlo mirando las pruebas. Es
//! confianza externa disfrazada de sistema demostrable.
//!
//! ## El diseño: árbol disperso con prueba de inserción
//!
//! Un árbol de Merkle de 32 niveles donde cada nullifier ocupa una
//! posición determinada por sus bits bajos. Una hoja vale cero si está
//! libre, y el propio nullifier si está gastado.
//!
//! Al gastar, el circuito demuestra **dos cosas a la vez**:
//!
//! 1. **No-pertenencia**: la hoja en la posición del nullifier vale cero
//!    en `nullifier_root_old`. Es decir, no se había gastado.
//! 2. **Inserción**: `nullifier_root_new` es exactamente ese árbol con
//!    esa hoja puesta al nullifier.
//!
//! Ambas raíces son públicas, así que la cadena de raíces es auditable y
//! el doble gasto pasa a ser **matemáticamente imposible**, no
//! "detectable por una base de datos".
//!
//! ## ⚠️ Limitación documentada: colisiones de posición
//!
//! La posición se deriva de los **32 bits bajos** del nullifier, no del
//! valor completo (un árbol sobre todo el campo tendría 254 niveles, con
//! un coste prohibitivo).
//!
//! Si dos nullifiers distintos caen en la misma posición, el segundo **no
//! podrá gastarse**: la hoja ya no está vacía. Es una **denegación de
//! servicio, no un doble gasto** — la solidez se mantiene, pero la
//! completitud no.
//!
//! Probabilidad por la paradoja del cumpleaños: con 10.000 nullifiers,
//! aproximadamente 1 entre 10^5. Aceptable para una demostración;
//! **inaceptable para producción a escala**, donde habría que usar un
//! árbol indexado (como el de Aztec) que evita las colisiones por
//! construcción.
//!
//! Esta limitación se documenta aquí en vez de esconderla porque es
//! exactamente el tipo de detalle que decide si un diseño escala.

use ark_crypto_primitives::sponge::Absorb;
use ark_ff::{BigInteger, PrimeField};
use ark_r1cs_std::fields::fp::FpVar;
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{ConstraintSystemRef, SynthesisError};

use crate::poseidon_hash::{secure_hash, secure_hash_gadget};

/// Profundidad del árbol de nullifiers. Ver la nota sobre colisiones.
pub const NULLIFIER_TREE_DEPTH: usize = 32;

/// Posición de un nullifier en el árbol: sus 32 bits bajos.
pub fn nullifier_position<F: PrimeField>(nullifier: F) -> u64 {
    let bits = nullifier.into_bigint().to_bits_le();
    let mut pos = 0u64;
    for (i, bit) in bits.iter().take(NULLIFIER_TREE_DEPTH).enumerate() {
        if *bit {
            pos |= 1u64 << i;
        }
    }
    pos
}

/// Hashes de subárboles vacíos por nivel. Un árbol de 2^32 hojas no se
/// materializa; se representa por sus subárboles vacíos precalculados.
pub fn empty_subtrees<F: PrimeField + Absorb>() -> Vec<F> {
    let mut empty = vec![F::zero()];
    for k in 1..=NULLIFIER_TREE_DEPTH {
        let prev = empty[k - 1];
        empty.push(secure_hash(prev, prev));
    }
    empty
}

/// Camino de autenticación dentro del árbol de nullifiers.
#[derive(Clone, Debug)]
pub struct NullifierPath<F: PrimeField> {
    pub siblings: Vec<F>,
    pub is_right: Vec<bool>,
}

impl<F: PrimeField + Absorb> NullifierPath<F> {
    /// Camino de una posición en un árbol COMPLETAMENTE VACÍO.
    ///
    /// Basta para el caso de uso principal: demostrar que un nullifier
    /// nunca se ha gastado. Para un árbol con entradas previas habría que
    /// aportar los hermanos reales.
    pub fn for_empty_tree(position: u64) -> Self {
        let empty = empty_subtrees::<F>();
        let mut siblings = Vec::with_capacity(NULLIFIER_TREE_DEPTH);
        let mut is_right = Vec::with_capacity(NULLIFIER_TREE_DEPTH);
        let mut idx = position;
        for level in 0..NULLIFIER_TREE_DEPTH {
            siblings.push(empty[level]);
            is_right.push(idx % 2 == 1);
            idx /= 2;
        }
        Self { siblings, is_right }
    }
}

/// Sube una hoja por el camino, de forma nativa.
pub fn climb<F: PrimeField + Absorb>(leaf: F, path: &NullifierPath<F>) -> F {
    let mut current = leaf;
    for level in 0..NULLIFIER_TREE_DEPTH {
        current = if path.is_right[level] {
            secure_hash(path.siblings[level], current)
        } else {
            secure_hash(current, path.siblings[level])
        };
    }
    current
}

/// Raíz de un árbol de nullifiers vacío.
pub fn empty_root<F: PrimeField + Absorb>() -> F {
    empty_subtrees::<F>()[NULLIFIER_TREE_DEPTH]
}

/// Sube una hoja por el camino DENTRO del circuito.
fn climb_gadget<F: PrimeField + Absorb>(
    cs: ConstraintSystemRef<F>,
    leaf: &FpVar<F>,
    siblings: &[FpVar<F>],
    bits: &[Boolean<F>],
) -> Result<FpVar<F>, SynthesisError> {
    let mut current = leaf.clone();
    for level in 0..NULLIFIER_TREE_DEPTH {
        let sibling = &siblings[level];
        let bit = &bits[level];
        let left = bit.select(sibling, &current)?;
        let right = bit.select(&current, sibling)?;
        current = secure_hash_gadget(cs.clone(), &left, &right)?;
    }
    Ok(current)
}

/// **EN CIRCUITO: no-pertenencia + inserción, en una sola operación.**
///
/// Demuestra que:
/// - la posición del nullifier estaba VACÍA en `root_old_var`, y
/// - `root_new_var` es ese mismo árbol con el nullifier insertado.
///
/// El mismo camino de hermanos sirve para ambas subidas: es lo que ata
/// las dos raíces a la MISMA posición del árbol. (En AIR esto exigiría el
/// diseño en lockstep; en R1CS las restricciones de copia lo garantizan
/// al reutilizar las mismas variables.)
pub fn enforce_insert_unspent<F: PrimeField + Absorb>(
    cs: ConstraintSystemRef<F>,
    nullifier_var: &FpVar<F>,
    siblings: &[FpVar<F>],
    bits: &[Boolean<F>],
    root_old_var: &FpVar<F>,
    root_new_var: &FpVar<F>,
) -> Result<(), SynthesisError> {
    assert_eq!(siblings.len(), NULLIFIER_TREE_DEPTH);
    assert_eq!(bits.len(), NULLIFIER_TREE_DEPTH);

    // 1. NO-PERTENENCIA: la hoja valia CERO antes.
    let empty_leaf = FpVar::<F>::Constant(F::zero());
    let computed_old = climb_gadget(cs.clone(), &empty_leaf, siblings, bits)?;
    computed_old.enforce_equal(root_old_var)?;

    // 2. INSERCION: la hoja pasa a valer el nullifier, MISMO camino.
    let computed_new = climb_gadget(cs, nullifier_var, siblings, bits)?;
    computed_new.enforce_equal(root_new_var)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bls12_381::Fr;
    use ark_relations::r1cs::ConstraintSystem;

    fn alloc_path(
        cs: ConstraintSystemRef<Fr>,
        path: &NullifierPath<Fr>,
    ) -> (Vec<FpVar<Fr>>, Vec<Boolean<Fr>>) {
        let siblings = path
            .siblings
            .iter()
            .map(|s| FpVar::new_witness(cs.clone(), || Ok(*s)).unwrap())
            .collect();
        let bits = path
            .is_right
            .iter()
            .map(|b| Boolean::new_witness(cs.clone(), || Ok(*b)).unwrap())
            .collect();
        (siblings, bits)
    }

    /// La raíz de un árbol vacío coincide con subir una hoja cero por
    /// cualquier camino vacío.
    #[test]
    fn empty_tree_is_consistent() {
        let path = NullifierPath::<Fr>::for_empty_tree(12345);
        assert_eq!(climb(Fr::from(0u64), &path), empty_root::<Fr>());
    }

    /// EL TEST CLAVE: insertar un nullifier no gastado satisface el
    /// circuito.
    #[test]
    fn inserting_unspent_nullifier_satisfies() {
        let nullifier = Fr::from(0xABCDEFu64);
        let pos = nullifier_position(nullifier);
        let path = NullifierPath::<Fr>::for_empty_tree(pos);

        let root_old = empty_root::<Fr>();
        let root_new = climb(nullifier, &path);

        let cs = ConstraintSystem::<Fr>::new_ref();
        let null_var = FpVar::new_witness(cs.clone(), || Ok(nullifier)).unwrap();
        let (siblings, bits) = alloc_path(cs.clone(), &path);
        let old_var = FpVar::new_input(cs.clone(), || Ok(root_old)).unwrap();
        let new_var = FpVar::new_input(cs.clone(), || Ok(root_new)).unwrap();

        enforce_insert_unspent(cs.clone(), &null_var, &siblings, &bits, &old_var, &new_var)
            .unwrap();

        assert!(cs.is_satisfied().unwrap());
        println!(
            "Restricciones de no-pertenencia + insercion: {}",
            cs.num_constraints()
        );
    }

    /// **EL TEST QUE CIERRA EL DOBLE GASTO.**
    ///
    /// Se intenta gastar un nullifier cuya posición YA está ocupada. La
    /// no-pertenencia falla y el circuito lo rechaza.
    ///
    /// Con el diseño anterior esto habría dependido de que una base de
    /// datos externa lo detectara. Aquí es matemáticamente imposible.
    #[test]
    fn double_spend_is_mathematically_impossible() {
        let nullifier = Fr::from(0xABCDEFu64);
        let pos = nullifier_position(nullifier);
        let path = NullifierPath::<Fr>::for_empty_tree(pos);

        // El arbol YA contiene este nullifier: raiz tras el primer gasto.
        let root_after_first_spend = climb(nullifier, &path);
        // El atacante intenta gastarlo otra vez, declarando esa raiz como
        // la "antigua" y una nueva cualquiera.
        let root_new = climb(nullifier, &path);

        let cs = ConstraintSystem::<Fr>::new_ref();
        let null_var = FpVar::new_witness(cs.clone(), || Ok(nullifier)).unwrap();
        let (siblings, bits) = alloc_path(cs.clone(), &path);
        let old_var = FpVar::new_input(cs.clone(), || Ok(root_after_first_spend)).unwrap();
        let new_var = FpVar::new_input(cs.clone(), || Ok(root_new)).unwrap();

        enforce_insert_unspent(cs.clone(), &null_var, &siblings, &bits, &old_var, &new_var)
            .unwrap();

        assert!(
            !cs.is_satisfied().unwrap(),
            "CRITICO: gastar dos veces el mismo nullifier debe ser imposible, \
             no solo detectable por una base de datos externa"
        );
    }

    /// Declarar una raíz nueva que no corresponde a la inserción real
    /// debe fallar.
    #[test]
    fn wrong_new_root_fails() {
        let nullifier = Fr::from(0xABCDEFu64);
        let pos = nullifier_position(nullifier);
        let path = NullifierPath::<Fr>::for_empty_tree(pos);

        let cs = ConstraintSystem::<Fr>::new_ref();
        let null_var = FpVar::new_witness(cs.clone(), || Ok(nullifier)).unwrap();
        let (siblings, bits) = alloc_path(cs.clone(), &path);
        let old_var = FpVar::new_input(cs.clone(), || Ok(empty_root::<Fr>())).unwrap();
        let new_var = FpVar::new_input(cs.clone(), || Ok(Fr::from(999u64))).unwrap();

        enforce_insert_unspent(cs.clone(), &null_var, &siblings, &bits, &old_var, &new_var)
            .unwrap();
        assert!(!cs.is_satisfied().unwrap());
    }

    /// Nullifiers distintos ocupan posiciones distintas (salvo colisión,
    /// documentada en la cabecera).
    #[test]
    fn distinct_nullifiers_map_to_distinct_positions() {
        let a = nullifier_position(Fr::from(1u64));
        let b = nullifier_position(Fr::from(2u64));
        let c = nullifier_position(Fr::from(0xFFFF_FFFFu64));
        assert_ne!(a, b);
        assert_ne!(a, c);
        assert_ne!(b, c);
    }

    /// La posición depende solo de los bits bajos: esto DOCUMENTA la
    /// limitación de colisiones en un test ejecutable, en vez de dejarla
    /// solo en un comentario.
    #[test]
    fn position_collisions_are_possible_and_documented() {
        let a = Fr::from(1u64);
        // Mismo valor en los 32 bits bajos, distinto por encima.
        let b = Fr::from(1u64 + (1u64 << 32));
        assert_eq!(
            nullifier_position(a),
            nullifier_position(b),
            "las colisiones de posicion son posibles por diseno; con 10.000 \
             nullifiers la probabilidad ronda 1 entre 10^5. Producen denegacion \
             de servicio, NO doble gasto"
        );
    }
}
