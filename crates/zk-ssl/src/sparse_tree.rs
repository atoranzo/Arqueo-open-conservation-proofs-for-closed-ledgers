//! Árbol de Merkle **disperso y mutable** sobre digests de Rescue.
//!
//! Equivalente del `settlement-layer::sparse_tree`, adaptado al backend
//! STARK: las hojas son digests de 4 elementos de Goldilocks, no un solo
//! escalar de BLS12-381.
//!
//! ## Por qué hace falta
//!
//! El árbol de `stark-experiment::merkle` construye la estructura entera
//! desde la lista completa de hojas. Con profundidad 32 eso son 2^32 =
//! **4.294 millones** de hojas: inviable incluso para un test, no digamos
//! para un ledger.
//!
//! Aquí solo se almacenan las hojas ocupadas. El resto se reconstruye con
//! los hashes de subárboles vacíos, precalculados una vez. Actualizar una
//! cuenta cuesta 32 hashes, no cuatro mil millones.
//!
//! ## Profundidad 32, no 20
//!
//! El backend STARK usa 32 niveles porque la traza debe tener longitud
//! potencia de dos y 20 ciclos no encajaban. Es una consecuencia del
//! paradigma, no una decisión de diseño — y hace el árbol de este backend
//! mayor que el de los otros, con más capacidad de cuentas a cambio de
//! más hashes por operación.
//!
//! ## Nodos internos en caché (§207, etapa 3 del RFC-0002)
//!
//! La primera versión recomputaba el árbol entero en cada `root()`, y en
//! cada nodo decidía la ocupación con un **barrido lineal de todas las
//! hojas**. Medido en `AUDITORIA.md` §204 (banco `etapa_a3_escala`): eso
//! NO dominaba el `apply` —exponente 0,18, plano— pero sí
//! `send_materials`, que construye caminos: **e = 1,08**, de 0,64 ms con
//! 4 cuentas a 11,84 con 60, y ~248 ms extrapolados a 1.000. Y corre
//! **en el nodo**, en cada envío.
//!
//! Ahora los nodos internos no vacíos se guardan y se actualizan **solo
//! en el camino de la hoja modificada**: `set_leaf` cuesta `O(profundidad)`
//! hashes, y `root()` y cada hermano de `path_for` son una consulta.
//!
//! ⚠️ **El precio, dicho:** se cambia tiempo por memoria. El mapa guarda
//! como mucho `hojas × profundidad` entradas —los subárboles vacíos NO se
//! almacenan—, así que un ledger de millones de cuentas ocupará memoria
//! proporcional. Es un intercambio deliberado, no un descuido.
//!
//! **La semántica NO cambia**: mismas raíces, mismos caminos. Lo
//! garantiza la compuerta de conformidad, que compara digests byte a byte
//! contra los vectores de `zkssl/0.1`.

use std::collections::HashMap;
use winterfell::math::fields::f64::BaseElement;
use winterfell::math::FieldElement;

use stark_experiment::merkle::{native_merge, Digest, MerklePath, TREE_DEPTH};

/// Árbol disperso de profundidad configurable.
///
/// El de cuentas y el de nullifiers usan `TREE_DEPTH` (32); el de
/// **congelados** usa 24, porque su subida tiene que caber en las filas
/// libres del circuito de liquidación.
#[derive(Clone, Debug)]
pub struct SparseTree {
    depth: usize,
    leaves: HashMap<u64, Digest>,
    /// Hash del subárbol vacío de cada altura. `empty[0]` es la hoja
    /// vacía; `empty[TREE_DEPTH]` es la raíz de un árbol vacío.
    empty: Vec<Digest>,
    /// Nodos internos NO vacíos, por `(nivel, índice)`.
    ///
    /// Invariante: una entrada ausente significa **subárbol vacío**, y su
    /// valor es `empty[nivel]`. Por eso `recompute_path` **borra** el nodo
    /// cuando su valor vuelve a ser el vacío: si no, el mapa crecería sin
    /// límite y la consulta devolvería basura tras un borrado.
    nodes: HashMap<(usize, u64), Digest>,
}

impl Default for SparseTree {
    fn default() -> Self {
        Self::new()
    }
}

impl SparseTree {
    /// Árbol de la profundidad habitual (`TREE_DEPTH`).
    pub fn new() -> Self {
        Self::with_depth(TREE_DEPTH)
    }

    /// Árbol de profundidad explícita.
    pub fn with_depth(depth: usize) -> Self {
        let zero: Digest = [BaseElement::ZERO; 4];
        let mut empty = vec![zero];
        for k in 1..=depth {
            let prev = empty[k - 1];
            empty.push(native_merge(prev, prev));
        }
        Self {
            depth,
            leaves: HashMap::new(),
            empty,
            nodes: HashMap::new(),
        }
    }

    /// **Cuántas posiciones admite este árbol.**
    ///
    /// ⚠️ `path_for` con un índice mayor produce un camino que **no llega a
    /// la raíz**: la prueba no verificaría, y sin decir por qué. Quien
    /// asigne posiciones debe comprobar este límite.
    pub fn capacity(&self) -> u64 {
        1u64 << self.depth
    }

    pub fn len(&self) -> usize {
        self.leaves.len()
    }

    pub fn is_empty(&self) -> bool {
        self.leaves.is_empty()
    }

    /// Valor de una hoja: el almacenado, o el digest cero si está libre.
    pub fn leaf(&self, index: u64) -> Digest {
        self.leaves
            .get(&index)
            .copied()
            .unwrap_or([BaseElement::ZERO; 4])
    }

    /// ¿Está esta posición ocupada?
    pub fn is_occupied(&self, index: u64) -> bool {
        self.leaves.contains_key(&index)
    }

    pub fn set_leaf(&mut self, index: u64, value: Digest) {
        if value == [BaseElement::ZERO; 4] {
            self.leaves.remove(&index);
        } else {
            self.leaves.insert(index, value);
        }
        self.recompute_path(index);
    }

    /// Rehace los nodos internos del camino de una hoja hasta la raíz.
    ///
    /// `O(profundidad)` hashes. Es lo único que cuesta una escritura.
    fn recompute_path(&mut self, index: u64) {
        let mut idx = index;
        for level in 1..=self.depth {
            let parent = idx >> 1;
            let left = self.node(level - 1, parent * 2);
            let right = self.node(level - 1, parent * 2 + 1);
            let value = native_merge(left, right);
            if value == self.empty[level] {
                self.nodes.remove(&(level, parent));
            } else {
                self.nodes.insert((level, parent), value);
            }
            idx = parent;
        }
    }

    /// Hash de un nodo interno: **una consulta**, no un recorrido.
    ///
    /// Un subárbol sin hojas ocupadas no está en el mapa y devuelve el
    /// hash vacío precalculado. Es lo que hace viable un árbol de 2^32
    /// posiciones — y ahora también lo que lo hace barato.
    fn node(&self, level: usize, index: u64) -> Digest {
        if level == 0 {
            return self.leaf(index);
        }
        self.nodes
            .get(&(level, index))
            .copied()
            .unwrap_or(self.empty[level])
    }

    /// Hojas ocupadas, para exportar el estado.
    ///
    /// Devuelve pares (posición, valor). El orden **no** está definido:
    /// quien exporte debe ordenarlos si necesita determinismo.
    pub fn occupied(&self) -> Vec<(u64, Digest)> {
        self.leaves.iter().map(|(k, v)| (*k, *v)).collect()
    }

    /// Raíz del árbol. Ahora es **una consulta**, no una reconstrucción.
    pub fn root(&self) -> Digest {
        self.node(self.depth, 0)
    }

    /// Cuántos nodos internos no vacíos hay en caché.
    ///
    /// Diagnóstico: crece como `hojas × profundidad` en el peor caso, y
    /// **debe volver a cero** si se vacía el árbol. Hay un test que lo
    /// exige.
    pub fn cached_nodes(&self) -> usize {
        self.nodes.len()
    }

    /// Camino de autenticación de una posición. Funciona igual para hojas
    /// ocupadas y libres — esto último es lo que permite demostrar
    /// no-pertenencia.
    pub fn path_for(&self, index: u64) -> MerklePath {
        let mut siblings = Vec::with_capacity(self.depth);
        let mut is_right = Vec::with_capacity(self.depth);
        let mut idx = index;
        for level in 0..self.depth {
            siblings.push(self.node(level, idx ^ 1));
            is_right.push(idx % 2 == 1);
            idx /= 2;
        }
        MerklePath { siblings, is_right }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(n: u64) -> Digest {
        [
            BaseElement::new(n),
            BaseElement::new(n + 1),
            BaseElement::new(n + 2),
            BaseElement::new(n + 3),
        ]
    }

    #[test]
    fn empty_tree_has_canonical_root() {
        let t = SparseTree::new();
        assert_eq!(t.root(), t.empty[TREE_DEPTH]);
        assert!(t.is_empty());
    }

    #[test]
    fn insert_and_remove_are_inverse() {
        let mut t = SparseTree::new();
        let empty_root = t.root();
        t.set_leaf(42, d(1234));
        assert_ne!(t.root(), empty_root);
        assert!(t.is_occupied(42));
        t.set_leaf(42, [BaseElement::ZERO; 4]);
        assert_eq!(t.root(), empty_root);
        assert!(!t.is_occupied(42));
    }

    /// EL TEST CLAVE: el camino generado reconstruye la raíz. Si fallara,
    /// las pruebas construidas con estos caminos no verificarían.
    #[test]
    fn path_reconstructs_the_root() {
        let mut t = SparseTree::new();
        t.set_leaf(3, d(111));
        t.set_leaf(5, d(222));
        t.set_leaf(1_000_000, d(333));

        for index in [3u64, 5, 1_000_000] {
            let path = t.path_for(index);
            let mut current = t.leaf(index);
            for level in 0..TREE_DEPTH {
                current = if path.is_right[level] {
                    native_merge(path.siblings[level], current)
                } else {
                    native_merge(current, path.siblings[level])
                };
            }
            assert_eq!(current, t.root(), "camino de la hoja {index}");
        }
    }

    /// También funciona para una posición VACÍA: es lo que permite
    /// demostrar no-pertenencia de un nullifier.
    #[test]
    fn path_works_for_empty_position() {
        let mut t = SparseTree::new();
        t.set_leaf(3, d(111));
        let path = t.path_for(999_999);
        let mut current = [BaseElement::ZERO; 4];
        for level in 0..TREE_DEPTH {
            current = if path.is_right[level] {
                native_merge(path.siblings[level], current)
            } else {
                native_merge(current, path.siblings[level])
            };
        }
        assert_eq!(current, t.root());
    }

    /// **El invariante de la caché**: vaciar el árbol debe dejar el mapa
    /// de nodos internos a CERO. Si no, un borrado dejaría basura y
    /// `node()` devolvería un valor obsoleto en vez del hash vacío.
    #[test]
    fn emptying_the_tree_clears_the_cache() {
        let mut t = SparseTree::new();
        let empty_root = t.root();
        assert_eq!(t.cached_nodes(), 0);

        for i in [3u64, 5, 1_000_000, 1u64 << (TREE_DEPTH - 1)] {
            t.set_leaf(i, d(i));
        }
        assert!(t.cached_nodes() > 0, "la cache deberia tener nodos");
        assert_ne!(t.root(), empty_root);

        for i in [3u64, 5, 1_000_000, 1u64 << (TREE_DEPTH - 1)] {
            t.set_leaf(i, [BaseElement::ZERO; 4]);
        }
        assert_eq!(t.root(), empty_root, "la raiz debe volver a la vacia");
        assert_eq!(t.cached_nodes(), 0, "CRITICO: la cache no se limpio");
    }

    /// Reescribir la misma hoja no acumula nodos: el camino se rehace, no
    /// se anade.
    #[test]
    fn rewriting_a_leaf_does_not_grow_the_cache() {
        let mut t = SparseTree::new();
        t.set_leaf(7, d(1));
        let n1 = t.cached_nodes();
        for k in 2..20u64 {
            t.set_leaf(7, d(k));
        }
        assert_eq!(t.cached_nodes(), n1, "la cache crecio al reescribir");
    }

    /// El orden de insercion no cambia la raiz: la caché no introduce
    /// dependencia del historial.
    #[test]
    fn insertion_order_does_not_change_the_root() {
        let mut a = SparseTree::new();
        for i in [1u64, 500, 99_999, 7] {
            a.set_leaf(i, d(i));
        }
        let mut b = SparseTree::new();
        for i in [99_999u64, 7, 1, 500] {
            b.set_leaf(i, d(i));
        }
        assert_eq!(a.root(), b.root());
        assert_eq!(a.cached_nodes(), b.cached_nodes());
    }

    /// Posiciones muy separadas no interfieren: comprueba que el
    /// cortocircuito de subárboles vacíos no rompe nada a gran escala.
    #[test]
    fn distant_positions_are_independent() {
        let mut t = SparseTree::new();
        t.set_leaf(0, d(1));
        let path_before = t.path_for(0);
        // Una posicion en la mitad opuesta de un arbol de 2^32.
        t.set_leaf(1u64 << (TREE_DEPTH - 1), d(2));
        let path_after = t.path_for(0);

        assert_eq!(
            path_before.siblings[..TREE_DEPTH - 1],
            path_after.siblings[..TREE_DEPTH - 1]
        );
        assert_ne!(
            path_before.siblings[TREE_DEPTH - 1],
            path_after.siblings[TREE_DEPTH - 1]
        );
    }
}
