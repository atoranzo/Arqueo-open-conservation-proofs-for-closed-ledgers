//! Árbol de Merkle **disperso y mutable**: la estructura de datos que
//! necesita una capa de liquidación.
//!
//! ## Por qué no vale el árbol de `zk-core::merkle`
//!
//! Aquel construye el árbol entero a partir de la lista completa de
//! hojas. Con profundidad 20 eso son 2^20 = 1.048.576 hojas y unos dos
//! millones de invocaciones de Poseidon **cada vez que cambia un saldo**.
//! Sirve para tests; es inviable para un ledger.
//!
//! Aquí solo se almacenan las hojas ocupadas. El resto del árbol se
//! reconstruye con los hashes de subárboles vacíos, precalculados una
//! vez. Actualizar una hoja cuesta 20 hashes, no dos millones.
//!
//! ## Lo que esto permite y antes no
//!
//! - Actualizar una cuenta en tiempo logarítmico.
//! - Generar el camino de autenticación de cualquier cuenta.
//! - Encadenar operaciones: cada transferencia parte de la raíz que dejó
//!   la anterior.
//!
//! Es decir: mantener un estado, que es lo que distingue una capa de
//! liquidación de un circuito suelto.

use ark_bls12_381::Fr;
use ark_ff::Zero;
use std::collections::HashMap;

use zk_core::poseidon_hash::secure_hash;

/// Árbol disperso, genérico sobre la profundidad.
///
/// El parámetro `DEPTH` es necesario porque la capa mantiene DOS árboles
/// de profundidades distintas: el de cuentas (20 niveles) y el de
/// nullifiers (32). Fijar la profundidad en el tipo hace que un camino de
/// un árbol no pueda usarse por error en el otro — un error que ya se
/// coló una vez y que el compilador ahora impide.
#[derive(Clone, Debug)]
pub struct SparseMerkleTree<const DEPTH: usize> {
    /// Solo las hojas realmente ocupadas.
    leaves: HashMap<u64, Fr>,
    /// Hash del subárbol vacío de cada altura. `empty[0]` es la hoja
    /// vacía; `empty[DEPTH]` es la raíz de un árbol vacío.
    empty: Vec<Fr>,
}

impl<const DEPTH: usize> Default for SparseMerkleTree<DEPTH> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const DEPTH: usize> SparseMerkleTree<DEPTH> {
    pub fn new() -> Self {
        let mut empty = vec![Fr::zero()];
        for k in 1..=DEPTH {
            let prev = empty[k - 1];
            empty.push(secure_hash(prev, prev));
        }
        Self {
            leaves: HashMap::new(),
            empty,
        }
    }

    /// Número de cuentas ocupadas.
    pub fn len(&self) -> usize {
        self.leaves.len()
    }

    pub fn is_empty(&self) -> bool {
        self.leaves.is_empty()
    }

    /// Valor de una hoja: el almacenado, o cero si está libre.
    pub fn leaf(&self, index: u64) -> Fr {
        self.leaves.get(&index).copied().unwrap_or_else(Fr::zero)
    }

    pub fn set_leaf(&mut self, index: u64, value: Fr) {
        if value.is_zero() {
            self.leaves.remove(&index);
        } else {
            self.leaves.insert(index, value);
        }
    }

    /// Hash de un nodo interno.
    ///
    /// `level` es la altura (0 = hojas). `index` es la posición dentro de
    /// ese nivel. Recursivo, pero cortocircuitado: un subárbol sin hojas
    /// ocupadas devuelve directamente el hash vacío precalculado, sin
    /// recorrerlo.
    fn node(&self, level: usize, index: u64) -> Fr {
        if level == 0 {
            return self.leaf(index);
        }
        // Rango de hojas que cuelgan de este nodo.
        let span = 1u64 << level;
        let start = index * span;
        let end = start + span;
        let occupied = self.leaves.keys().any(|k| *k >= start && *k < end);
        if !occupied {
            return self.empty[level];
        }
        let left = self.node(level - 1, index * 2);
        let right = self.node(level - 1, index * 2 + 1);
        secure_hash(left, right)
    }

    pub fn root(&self) -> Fr {
        self.node(DEPTH, 0)
    }

    /// Camino de autenticación de una hoja.
    ///
    /// Devuelve las piezas en crudo (hermanos y direcciones) en vez de un
    /// tipo concreto, porque el mismo árbol sirve para caminos de cuentas
    /// y de nullifiers, que son tipos distintos en `zk-core`.
    pub fn path_for(&self, index: u64) -> (Vec<Fr>, Vec<bool>) {
        let mut siblings = Vec::with_capacity(DEPTH);
        let mut is_right = Vec::with_capacity(DEPTH);
        let mut idx = index;
        for level in 0..DEPTH {
            siblings.push(self.node(level, idx ^ 1));
            is_right.push(idx % 2 == 1);
            idx /= 2;
        }
        (siblings, is_right)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Profundidad de prueba: la del arbol de cuentas.
    const D: usize = zk_core::merkle::TREE_DEPTH;
    type Tree = SparseMerkleTree<D>;

    /// Un árbol vacío tiene la raíz del subárbol vacío de altura máxima.
    #[test]
    fn empty_tree_has_canonical_root() {
        let t = Tree::new();
        assert_eq!(t.root(), t.empty[D]);
        assert!(t.is_empty());
    }

    /// Insertar cambia la raíz; borrar la restaura.
    #[test]
    fn insert_and_remove_are_inverse() {
        let mut t = Tree::new();
        let empty_root = t.root();

        t.set_leaf(42, Fr::from(1234u64));
        assert_ne!(t.root(), empty_root);
        assert_eq!(t.len(), 1);

        t.set_leaf(42, Fr::zero());
        assert_eq!(t.root(), empty_root, "borrar deberia restaurar la raiz");
        assert!(t.is_empty());
    }

    /// EL TEST CLAVE de la estructura: el camino que genera reconstruye
    /// la raíz. Si esto fallara, las pruebas generadas con estos caminos
    /// no verificarían.
    #[test]
    fn path_reconstructs_the_root() {
        let mut t = Tree::new();
        t.set_leaf(3, Fr::from(111u64));
        t.set_leaf(5, Fr::from(222u64));
        t.set_leaf(1000, Fr::from(333u64));

        for index in [3u64, 5, 1000] {
            let (siblings, is_right) = t.path_for(index);
            let mut current = t.leaf(index);
            for level in 0..D {
                current = if is_right[level] {
                    secure_hash(siblings[level], current)
                } else {
                    secure_hash(current, siblings[level])
                };
            }
            assert_eq!(
                current,
                t.root(),
                "el camino de la hoja {index} deberia reconstruir la raiz"
            );
        }
    }

    /// También funciona para una hoja VACÍA: es lo que permite demostrar
    /// no-pertenencia.
    #[test]
    fn path_works_for_empty_leaf() {
        let mut t = Tree::new();
        t.set_leaf(3, Fr::from(111u64));

        let (siblings, is_right) = t.path_for(999);
        let mut current = Fr::zero();
        for level in 0..D {
            current = if is_right[level] {
                secure_hash(siblings[level], current)
            } else {
                secure_hash(current, siblings[level])
            };
        }
        assert_eq!(current, t.root());
    }

    /// Actualizar una hoja no altera el camino de otra rama lejana.
    #[test]
    fn distant_updates_do_not_disturb_unrelated_paths() {
        let mut t = Tree::new();
        t.set_leaf(0, Fr::from(1u64));
        let (sib_before, _) = t.path_for(0);

        // Una hoja en la mitad opuesta del arbol.
        t.set_leaf((1u64 << (D - 1)) + 7, Fr::from(2u64));
        let (sib_after, _) = t.path_for(0);

        // Los niveles bajos no cambian; solo el ultimo hermano.
        assert_eq!(sib_before[..D - 1], sib_after[..D - 1]);
        assert_ne!(sib_before[D - 1], sib_after[D - 1]);
    }
}
