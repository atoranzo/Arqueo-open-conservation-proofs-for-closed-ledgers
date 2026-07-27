//! Árbol de Merkle de 20 niveles verificado dentro del circuito
//! PLONK-KZG.
//!
//! ## Mucho más corto que en los otros backends
//!
//! `dusk-plonk` trae `component_select(bit, a, b)` (con semántica
//! documentada: `bit == 1 => a`) y `component_boolean(bit)`. La selección
//! izquierda/derecha de cada nivel, que en Groth16, Halo2 y STARK hubo
//! que construir a mano con puertas propias, aquí son dos llamadas.
//!
//! ## Por qué NO se usa `poseidon-merkle`
//!
//! Existe (`poseidon-merkle 0.9.0-rc.0`), pero implementa el árbol
//! **nativo** de Dusk. Lo que necesita un circuito de cumplimiento es
//! verificar un camino de autenticación DENTRO del circuito, que es otra
//! cosa. Además solo está publicado como release candidate.
//!
//! ## Profundidad 20, igual que Groth16 y Halo2
//!
//! A diferencia del backend STARK, que tuvo que usar 32 niveles por el
//! requisito de que la traza sea potencia de dos, aquí no hay tal
//! restricción y se mantiene la profundidad de los otros dos backends.
//! Eso hace la comparación de tamaños más limpia.

use dusk_plonk::prelude::*;

use crate::poseidon_hash::{gadget_hash_pair, native_hash_pair};

/// Profundidad del árbol, igual que en `zk-core` y `halo2-experiment`.
pub const TREE_DEPTH: usize = 20;

/// Camino de autenticación: un hermano y una dirección por nivel.
#[derive(Clone, Debug)]
pub struct MerklePath {
    pub siblings: Vec<BlsScalar>,
    /// `false` = el nodo actual va a la izquierda; `true` = a la derecha.
    pub is_right: Vec<bool>,
}

impl Default for MerklePath {
    fn default() -> Self {
        Self {
            siblings: vec![BlsScalar::zero(); TREE_DEPTH],
            is_right: vec![false; TREE_DEPTH],
        }
    }
}

/// Sube una hoja por el camino, de forma nativa.
pub fn native_climb(leaf: BlsScalar, path: &MerklePath) -> BlsScalar {
    let mut current = leaf;
    for level in 0..TREE_DEPTH {
        current = if path.is_right[level] {
            native_hash_pair(path.siblings[level], current)
        } else {
            native_hash_pair(current, path.siblings[level])
        };
    }
    current
}

/// Sube una hoja por el camino DENTRO del circuito, y devuelve la raíz.
///
/// Cada bit de dirección se restringe a booleano antes de usarlo:
/// `component_select` asume esa restricción y la documentación de
/// `dusk-plonk` lo advierte explícitamente. Sin ella, un bit arbitrario
/// permitiría interpolar entre las dos ramas y fabricar caminos
/// inexistentes.
pub fn gadget_climb(
    composer: &mut Composer,
    leaf: Witness,
    siblings: &[Witness],
    bits: &[Witness],
) -> Witness {
    assert_eq!(siblings.len(), TREE_DEPTH);
    assert_eq!(bits.len(), TREE_DEPTH);

    let mut current = leaf;
    for level in 0..TREE_DEPTH {
        let bit = bits[level];
        let sibling = siblings[level];

        composer.component_boolean(bit);

        // bit = 1 (el nodo va a la derecha) → hermano a la izquierda.
        // bit = 0 → nodo a la izquierda, hermano a la derecha.
        let left = composer.component_select(bit, sibling, current);
        let right = composer.component_select(bit, current, sibling);

        current = gadget_hash_pair(composer, left, right);
    }
    current
}

/// Demuestra pertenencia al árbol: conocimiento de una hoja y un camino
/// que llevan a la raíz pública. Hoja y camino son PRIVADOS.
#[derive(Default, Debug)]
pub struct MerkleMembershipCircuit {
    pub leaf: BlsScalar,
    pub path: MerklePath,
    pub root: BlsScalar,
}

impl Circuit for MerkleMembershipCircuit {
    fn circuit(&self, composer: &mut Composer) -> Result<(), Error> {
        let w_leaf = composer.append_witness(self.leaf);

        let siblings: Vec<Witness> = self
            .path
            .siblings
            .iter()
            .map(|s| composer.append_witness(*s))
            .collect();
        let bits: Vec<Witness> = self
            .path
            .is_right
            .iter()
            .map(|b| {
                composer.append_witness(if *b {
                    BlsScalar::one()
                } else {
                    BlsScalar::zero()
                })
            })
            .collect();

        let computed = gadget_climb(composer, w_leaf, &siblings, &bits);

        let w_root = composer.append_public(self.root);
        composer.assert_equal(computed, w_root);

        Ok(())
    }
}

#[cfg(test)]
pub mod test_support_paths {
    use super::*;

    /// Hashes de subárboles vacíos por nivel, para construir árboles
    /// DISPERSOS: con profundidad 20 no se materializan 2^20 hojas en un
    /// test.
    pub fn empty_subtrees() -> Vec<BlsScalar> {
        let mut empty = vec![BlsScalar::zero()];
        for k in 1..=TREE_DEPTH {
            let prev = empty[k - 1];
            empty.push(native_hash_pair(prev, prev));
        }
        empty
    }

    /// Camino de una hoja situada en el índice 0 de un árbol disperso
    /// cuyo hermano de nivel 0 es `sibling0`.
    pub fn sparse_path_index_0(sibling0: BlsScalar) -> MerklePath {
        let empty = empty_subtrees();
        let mut siblings = vec![sibling0];
        let mut is_right = vec![false];
        for level in 1..TREE_DEPTH {
            siblings.push(empty[level]);
            is_right.push(false);
        }
        MerklePath { siblings, is_right }
    }

    /// Camino de una hoja situada en el índice 1 (hermana de la 0).
    pub fn sparse_path_index_1(sibling0: BlsScalar) -> MerklePath {
        let empty = empty_subtrees();
        let mut siblings = vec![sibling0];
        let mut is_right = vec![true];
        for level in 1..TREE_DEPTH {
            siblings.push(empty[level]);
            is_right.push(false);
        }
        MerklePath { siblings, is_right }
    }
}

#[cfg(test)]
mod tests {
    use super::test_support_paths::*;
    use super::*;
    use crate::test_support::shared_pp;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    /// EL TEST CLAVE: pertenencia al árbol demostrada con una prueba
    /// PLONK-KZG real.
    #[test]
    fn valid_merkle_path_verifies() {
        let mut rng = StdRng::seed_from_u64(0xabcdef);
        let pp = shared_pp();
        let (prover, verifier) =
            Compiler::compile::<MerkleMembershipCircuit>(pp, b"zk-ssl-merkle").expect("compile");

        let leaf = BlsScalar::from(1000u64);
        let path = sparse_path_index_0(BlsScalar::from(2000u64));
        let root = native_climb(leaf, &path);

        let circuit = MerkleMembershipCircuit {
            leaf,
            path: path.clone(),
            root,
        };
        println!("Puertas del circuito de Merkle (20 niveles): {}", circuit.size());

        let (proof, pi) = prover.prove(&mut rng, &circuit).expect("prove");
        verifier.verify(&proof, &pi).expect("deberia verificar");
    }

    /// Declarar una raíz que no corresponde debe impedir la prueba.
    #[test]
    fn wrong_declared_root_fails() {
        let mut rng = StdRng::seed_from_u64(0xabcdef);
        let pp = shared_pp();
        let (prover, _) =
            Compiler::compile::<MerkleMembershipCircuit>(pp, b"zk-ssl-merkle").expect("compile");

        let leaf = BlsScalar::from(1000u64);
        let path = sparse_path_index_0(BlsScalar::from(2000u64));

        let broken = MerkleMembershipCircuit {
            leaf,
            path,
            root: BlsScalar::from(999_999u64), // no es la raiz real
        };

        assert!(
            prover.prove(&mut rng, &broken).is_err(),
            "CRITICO: una raiz incorrecta no deberia producir prueba"
        );
    }

    /// Un hermano alterado cambia la raíz: el camino ata de verdad.
    #[test]
    fn tampered_sibling_changes_root() {
        let leaf = BlsScalar::from(1000u64);
        let path_a = sparse_path_index_0(BlsScalar::from(2000u64));
        let path_b = sparse_path_index_0(BlsScalar::from(2001u64));
        assert_ne!(
            native_climb(leaf, &path_a),
            native_climb(leaf, &path_b),
            "cambiar un hermano deberia cambiar la raiz"
        );
    }

    /// La dirección importa: la misma hoja con el mismo hermano pero en
    /// el lado contrario da otra raíz.
    #[test]
    fn direction_bit_matters() {
        let leaf = BlsScalar::from(1000u64);
        let sibling = BlsScalar::from(2000u64);
        assert_ne!(
            native_climb(leaf, &sparse_path_index_0(sibling)),
            native_climb(leaf, &sparse_path_index_1(sibling)),
            "el bit de direccion deberia cambiar la raiz"
        );
    }

    /// Dos hojas hermanas comparten los hermanos de los niveles altos y
    /// llegan a la MISMA raíz — la propiedad que hace consistente un
    /// árbol disperso, y que necesitará la partida doble.
    #[test]
    fn sibling_leaves_reach_same_root() {
        let leaf_a = BlsScalar::from(1000u64);
        let leaf_b = BlsScalar::from(2000u64);
        let root_from_a = native_climb(leaf_a, &sparse_path_index_0(leaf_b));
        let root_from_b = native_climb(leaf_b, &sparse_path_index_1(leaf_a));
        assert_eq!(
            root_from_a, root_from_b,
            "dos hojas hermanas deben llegar a la misma raiz"
        );
    }
}
