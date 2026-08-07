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

    /// **Carga N hojas y reconstruye los nodos internos DE ABAJO
    /// ARRIBA** (§221). Sustituye a llamar `set_leaf` N veces al
    /// arrancar.
    ///
    /// ## Por qué existe
    ///
    /// `set_leaf` cuesta `O(profundidad)` = **32 merges por hoja**, y
    /// recomputa cada nodo interno **una vez por descendiente**.
    /// Construir por niveles lo computa **una sola vez**.
    ///
    /// El ahorro no es constante: es `32 / (nodos por hoja)`, y §217
    /// midió que los nodos por hoja siguen `32 - log2(n)`. A cien mil
    /// hojas dispersas son 15,5 → **2,1× menos merges**; a un millón,
    /// 12,1 → 2,6×. **Mejora cuanto mayor es el ledger.**
    ///
    /// Es la salida barata de la deuda que §207 declaró en memoria y no
    /// en arranque (§217, banco B.4): no toca el formato en disco, no
    /// persiste nada nuevo y no exige RFC.
    ///
    /// ## Por qué da EXACTAMENTE el mismo árbol
    ///
    /// El invariante de `nodes` es «entrada ausente = subárbol vacío».
    /// Tras N `set_leaf`, el mapa contiene exactamente los nodos
    /// internos NO vacíos: cada `recompute_path` rehace su camino con
    /// los valores vigentes, y el último que toca un ancestro lo deja
    /// correcto. Esto construye ese mismo conjunto, y aplica el mismo
    /// corte —un nodo cuyo valor sea el vacío NO se guarda—.
    /// `insertion_order_does_not_change_the_root` ya afirmaba que la
    /// caché no depende del historial; hay un test que compara las dos
    /// vías hoja a hoja.
    ///
    /// ⚠️ **Descarta el contenido previo.** Es para arrancar, no para
    /// añadir: quien tenga hojas y quiera una más usa `set_leaf`.
    ///
    /// ⚠️ **Precio en memoria transitoria**: mantiene el nivel actual y
    /// el siguiente en mapas aparte. En el nivel bajo son ~N entradas,
    /// y a partir de ahí encogen. Se libera al terminar.
    pub fn rebuild_from(&mut self, hojas: impl IntoIterator<Item = (u64, Digest)>) {
        let cero: Digest = [BaseElement::ZERO; 4];
        self.leaves.clear();
        self.nodes.clear();
        for (idx, val) in hojas {
            // El digest cero es hoja libre, igual que en `set_leaf`.
            if val != cero {
                self.leaves.insert(idx, val);
            }
        }
        let mut actual: HashMap<u64, Digest> = self.leaves.clone();
        for level in 1..=self.depth {
            let vacio_hijo = self.empty[level - 1];
            let vacio_padre = self.empty[level];
            let mut padres: HashMap<u64, Digest> = HashMap::new();
            for idx in actual.keys() {
                let p = idx >> 1;
                if padres.contains_key(&p) {
                    continue; // el hermano ya lo computo
                }
                let izq = actual.get(&(p * 2)).copied().unwrap_or(vacio_hijo);
                let der = actual.get(&(p * 2 + 1)).copied().unwrap_or(vacio_hijo);
                let v = native_merge(izq, der);
                if v != vacio_padre {
                    padres.insert(p, v);
                }
            }
            for (p, v) in &padres {
                self.nodes.insert((level, *p), *v);
            }
            actual = padres;
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

    /// **Qué raíz saldría si esta posición valiera `value`** — sin tocar
    /// el árbol y **sin clonarlo** (§212).
    ///
    /// Sube por el camino de la hoja combinando con los hermanos que ya
    /// están en caché: `O(profundidad)` hashes, cero copias.
    ///
    /// ## Por qué existe
    ///
    /// `apply_send` y `apply_claim` calculaban la raíz hipotética
    /// **clonando el árbol entero** —dos clones por operación, uno de
    /// cuentas y otro de pendientes— solo para comprobar que coincidía con
    /// la que la prueba declara. Desde §207 esos clones copian también el
    /// mapa de nodos internos, así que dejaron de ser gratis; quedó
    /// anotado en aquel asiento.
    ///
    /// Y es además el ladrillo que `apply_many` necesita (etapa 2 del
    /// RFC-0002, pieza 2): permite preguntar «¿qué raíz saldría desde la
    /// INSTANTÁNEA DE ARRANQUE si esta hoja valiera X?» sin clonar por
    /// operación.
    ///
    /// El digest cero se trata como hoja vacía, igual que en `set_leaf`:
    /// `root_with(i, [0;4])` da la raíz que quedaría al borrarla.
    pub fn root_with(&self, index: u64, value: Digest) -> Digest {
        let mut actual = value;
        let mut idx = index;
        for level in 0..self.depth {
            let hermano = self.node(level, idx ^ 1);
            actual = if idx % 2 == 1 {
                native_merge(hermano, actual)
            } else {
                native_merge(actual, hermano)
            };
            idx /= 2;
        }
        actual
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

    /// **`root_with` coincide con clonar-y-escribir**, que es lo que
    /// sustituye. Si esto fallara, `apply_send` aceptaría o rechazaría
    /// raíces distintas de las que aceptaba antes.
    #[test]
    fn root_with_equivale_a_clonar_y_escribir() {
        let mut t = SparseTree::new();
        for i in [3u64, 5, 1_000_000] {
            t.set_leaf(i, d(i));
        }
        for (idx, val) in [
            (7u64, d(999)),                    // posicion libre
            (5, d(1234)),                      // sobrescribir una ocupada
            (3, [BaseElement::ZERO; 4]),       // borrar una ocupada
            (1u64 << (TREE_DEPTH - 1), d(77)), // la mitad opuesta del arbol
        ] {
            let mut copia = t.clone();
            copia.set_leaf(idx, val);
            assert_eq!(
                t.root_with(idx, val),
                copia.root(),
                "root_with difiere de clonar-y-escribir en {idx}"
            );
        }
        // Y no ha tocado el arbol original.
        assert_eq!(t.len(), 3);
    }

    /// `root_with` con el valor que ya tiene devuelve la raíz actual.
    #[test]
    fn root_with_del_valor_actual_es_la_raiz() {
        let mut t = SparseTree::new();
        t.set_leaf(11, d(11));
        t.set_leaf(22, d(22));
        assert_eq!(t.root_with(11, t.leaf(11)), t.root());
        assert_eq!(t.root_with(999, t.leaf(999)), t.root(), "posicion vacia");
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

    /// **EL TEST DE §221**: reconstruir por niveles da EXACTAMENTE el
    /// mismo árbol que insertar hoja a hoja.
    ///
    /// No basta la raíz: se compara también el número de nodos en caché
    /// —el invariante «ausente = vacío» se rompe si sobra o falta uno—
    /// y el camino de autenticación de cada hoja, que es lo que viaja
    /// dentro de las pruebas.
    ///
    /// Los índices imitan la colocación REAL de `accounts.rs`
    /// —dispersión pseudoaleatoria, no consecutiva— porque el patrón
    /// decide cuántos nodos altos se comparten y por tanto qué se está
    /// probando. Y se incluye el caso vacío y la mitad opuesta del
    /// árbol de 2^32.
    #[test]
    fn reconstruir_por_niveles_da_el_mismo_arbol() {
        let dispersa = |i: u64| -> u64 {
            i.wrapping_mul(0x9E37_79B9_7F4A_7C15)
                .rotate_left(31)
                .wrapping_mul(0xBF58_476D_1CE4_E5B9)
                % (1u64 << TREE_DEPTH)
        };
        let mut hojas: Vec<(u64, Digest)> =
            (0..500u64).map(|i| (dispersa(i), d(i + 1))).collect();
        // Vecinas, la mitad opuesta y el cero: los bordes del invariante.
        hojas.push((0, d(7)));
        hojas.push((1, d(8)));
        hojas.push((1u64 << (TREE_DEPTH - 1), d(9)));

        let mut una = SparseTree::new();
        for (i, v) in &hojas {
            una.set_leaf(*i, *v);
        }
        let mut otra = SparseTree::new();
        otra.rebuild_from(hojas.clone());

        assert_eq!(una.root(), otra.root(), "la raiz difiere");
        assert_eq!(una.len(), otra.len(), "numero de hojas");
        assert_eq!(
            una.cached_nodes(),
            otra.cached_nodes(),
            "CRITICO: el mapa de nodos internos no coincide"
        );
        for (i, _) in &hojas {
            let a = una.path_for(*i);
            let b = otra.path_for(*i);
            assert_eq!(a.siblings, b.siblings, "camino de la hoja {i}");
            assert_eq!(a.is_right, b.is_right, "lados de la hoja {i}");
        }

        // Sin hojas: arbol vacio canonico y cache a cero.
        let mut vacio = SparseTree::new();
        vacio.rebuild_from(Vec::new());
        assert_eq!(vacio.root(), SparseTree::new().root());
        assert_eq!(vacio.cached_nodes(), 0);

        // Una hoja CERO no ocupa, igual que en `set_leaf`.
        let mut con_cero = SparseTree::new();
        con_cero.rebuild_from(vec![(5u64, [BaseElement::ZERO; 4])]);
        assert!(con_cero.is_empty());
        assert_eq!(con_cero.cached_nodes(), 0);
    }
}
