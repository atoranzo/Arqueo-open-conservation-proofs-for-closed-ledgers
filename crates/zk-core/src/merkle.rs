//! Compromiso de estado: vincula `balance` a un valor real dentro de un
//! árbol de Merkle del ledger, en vez de aceptarlo como un testigo
//! "confiado" sin más (que era la limitación explícita documentada en
//! `circuit.rs`).
//!
//! ## Historial de esta pieza
//!
//! Este módulo usaba `toy_hash`, una función de compresión deliberadamente
//! insegura, como marcador de posición. **Ya no es así**: el árbol y la
//! verificación de pertenencia ahora usan `secure_hash`/`secure_hash_gadget`
//! (Poseidon real, ver `poseidon_hash.rs`). `toy_hash` se conserva en este
//! archivo únicamente como referencia de comparación en un test (para
//! confirmar que Poseidon produce resultados distintos a la función de
//! juguete) — no se usa en ninguna ruta funcional del árbol.

use ark_crypto_primitives::sponge::Absorb;
use ark_ff::PrimeField;
use ark_r1cs_std::fields::fp::FpVar;
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{ConstraintSystemRef, SynthesisError};

use crate::poseidon_hash::{secure_hash, secure_hash_gadget};

/// Profundidad del árbol de Merkle. 2^TREE_DEPTH cuentas máximas.
/// 20 niveles = hasta ~1 millón de cuentas; ajustar según necesidad real.
pub const TREE_DEPTH: usize = 20;

/// Función de compresión de dos elementos en uno. **NO SEGURA — solo se
/// conserva como referencia de comparación en tests** (ver
/// `poseidon_hash::tests::secure_hash_differs_from_toy_hash`). Ninguna
/// ruta funcional del árbol la usa ya.
#[cfg(test)]
pub fn toy_hash<F: PrimeField>(x: F, y: F) -> F {
    let x5 = x * x * x * x * x;
    let y5 = y * y * y * y * y;
    x5 + y5 + (x * y)
}

/// Hoja del árbol: compromiso de una cuenta, con Poseidon real. `nonce`
/// evita que dos cuentas con el mismo `account_id` y `balance` casual
/// coincidan en el hash (y, más adelante, sirve de base para nullifiers).
pub fn leaf_commitment<F: PrimeField + Absorb>(account_id: F, balance: F, nonce: F) -> F {
    secure_hash(secure_hash(account_id, balance), nonce)
}

/// Testigo de pertenencia (Merkle path) para una hoja concreta: en cada
/// nivel, el valor del hermano y si la hoja actual es el operando
/// izquierdo (`false`) o derecho (`true`) de la compresión.
#[derive(Clone, Debug)]
pub struct MerklePath<F: PrimeField> {
    pub siblings: Vec<F>,
    pub is_right: Vec<bool>,
}

/// Árbol de Merkle simple, solo para pruebas/demostración de wiring. Un árbol
/// disperso (sparse Merkle tree) sería lo apropiado para un ledger real con
/// cuentas identificadas por hash de dirección, pero añade complejidad no
/// esencial para validar la estructura del circuito. (⚠️ §822: ese árbol
/// disperso es el que pone cada cuenta en la hoja del hash de su dirección, y
/// este sigue sin serlo: lo es solo en que no guarda el relleno, abajo.)
///
/// ⚠️ §822 · **Disperso: no materializa el relleno** (la 75 del `BACKLOG.md`).
/// Decía «Árbol de Merkle simple y completo (no disperso)», y lo era: hasta el
/// §822 rellenaba hasta 2^TREE_DEPTH hojas y hasheaba el árbol entero:
/// 1.048.575 hashes para usar ocho, en cada `build`. Ahora el nivel `k` guarda
/// los nodos que cubren alguna hoja dada, y cualquier otro es `vacios[k]`, el
/// hash de un subárbol vacío de altura `k`: `vacios[0]` es el relleno de
/// siempre, `F::zero()`, y `vacios[k + 1]` es `secure_hash(vacios[k],
/// vacios[k])`, como `empty_subtrees` en los tests de
/// `plonk-experiment/src/merkle.rs`. Un subárbol vacío tiene ese hash lo
/// calcule quien lo calcule, así que la raíz y los caminos son los del árbol
/// denso. Lo atan los tests de abajo: la raíz, contra un KAT sacado del árbol
/// denso antes de cambiarlo; los bits del camino, contra los de su índice, que
/// son los del denso; y los hermanos, porque con esos bits el camino tiene que
/// subir a esa raíz, y con otros hermanos sería una colisión de Poseidon. Ni el
/// circuito ni `TREE_DEPTH` cambian.
pub struct SimpleMerkleTree<F: PrimeField> {
    // ⚠️ §822: decía «levels[0] = hojas, levels[ultimo] = [root]». Con alguna hoja,
    // el último nivel sigue siendo [root]; sin ninguna, todos los niveles quedan
    // vacíos y la raíz es vacios[TREE_DEPTH].
    levels: Vec<Vec<F>>, // levels[0] = las hojas dadas, levels[k] = los nodos del nivel k que las cubren
    vacios: Vec<F>,      // vacios[k] = el hash de un subárbol vacío de altura k
}

impl<F: PrimeField + Absorb> SimpleMerkleTree<F> {
    /// Construye el árbol a partir de una lista de hojas. Las que faltan hasta
    /// 2^TREE_DEPTH son el relleno (F::zero()), y no se materializan (⚠️ §822:
    /// decía «Rellena hasta 2^TREE_DEPTH con un valor de relleno (F::zero())
    /// si hace falta»).
    pub fn build(leaves: Vec<F>) -> Self {
        let target_len = 1usize << TREE_DEPTH;
        assert!(
            leaves.len() <= target_len,
            "demasiadas hojas para la profundidad configurada"
        );

        let mut vacios = Vec::with_capacity(TREE_DEPTH + 1);
        vacios.push(F::zero());
        for k in 0..TREE_DEPTH {
            let v = vacios[k];
            vacios.push(secure_hash(v, v));
        }

        let mut levels = vec![leaves];
        for k in 0..TREE_DEPTH {
            let prev = levels.last().unwrap();
            let mut next = Vec::with_capacity((prev.len() + 1) / 2);
            for pair in prev.chunks(2) {
                let derecho = if pair.len() == 2 { pair[1] } else { vacios[k] };
                next.push(secure_hash(pair[0], derecho));
            }
            levels.push(next);
        }
        Self { levels, vacios }
    }

    /// El nodo `idx` del nivel `level`: el guardado, o el de un subárbol vacío.
    fn nodo(&self, level: usize, idx: usize) -> F {
        self.levels[level]
            .get(idx)
            .copied()
            .unwrap_or(self.vacios[level])
    }

    pub fn root(&self) -> F {
        self.nodo(TREE_DEPTH, 0)
    }

    /// Genera el camino de Merkle para la hoja en `index`.
    pub fn path_for(&self, index: usize) -> MerklePath<F> {
        assert!(
            index < 1usize << TREE_DEPTH,
            "indice fuera del arbol: la profundidad configurada no lo alcanza"
        );
        let mut siblings = Vec::with_capacity(TREE_DEPTH);
        let mut is_right = Vec::with_capacity(TREE_DEPTH);
        let mut idx = index;

        for level in 0..TREE_DEPTH {
            let sibling_idx = idx ^ 1;
            siblings.push(self.nodo(level, sibling_idx));
            is_right.push(idx % 2 == 1);
            idx /= 2;
        }

        MerklePath { siblings, is_right }
    }
}

/// Verifica en-circuito que `leaf_var` pertenece al árbol cuya raíz es
/// `root_var`, siguiendo `path`. Devuelve `Ok(())` si las restricciones se
/// añadieron correctamente (no implica que se satisfagan: eso se comprueba
/// aparte, igual que el resto del circuito, en `cs.is_satisfied()`).
/// Calcula la raíz que resulta de subir `leaf_var` por el camino dado, y
/// la DEVUELVE en vez de imponer igualdad con una raíz esperada.
///
/// Existe para poder encadenar actualizaciones del árbol: en una
/// transferencia de partida doble hay que verificar la hoja del emisor
/// contra la raíz antigua, recalcular la raíz con la hoja modificada, y
/// verificar la hoja del receptor contra ESA raíz intermedia. Con
/// `enforce_merkle_membership` (que impone igualdad y no devuelve nada)
/// eso no se puede expresar.
pub fn compute_merkle_root<F: PrimeField + Absorb>(
    cs: ConstraintSystemRef<F>,
    leaf_var: &FpVar<F>,
    siblings_var: &[FpVar<F>],
    is_right_var: &[Boolean<F>],
) -> Result<FpVar<F>, SynthesisError> {
    assert_eq!(siblings_var.len(), TREE_DEPTH);
    assert_eq!(is_right_var.len(), TREE_DEPTH);

    let mut current = leaf_var.clone();

    for level in 0..TREE_DEPTH {
        let sibling = &siblings_var[level];
        let bit = &is_right_var[level];

        // Si is_right = true, la hoja actual es el operando derecho:
        // left = sibling, right = current.
        // Si is_right = false: left = current, right = sibling.
        let left = bit.select(sibling, &current)?;
        let right = bit.select(&current, sibling)?;

        current = secure_hash_gadget(cs.clone(), &left, &right)?;
    }

    Ok(current)
}

pub fn enforce_merkle_membership<F: PrimeField + Absorb>(
    cs: ConstraintSystemRef<F>,
    leaf_var: &FpVar<F>,
    siblings_var: &[FpVar<F>],
    is_right_var: &[Boolean<F>],
    root_var: &FpVar<F>,
) -> Result<(), SynthesisError> {
    let computed = compute_merkle_root(cs, leaf_var, siblings_var, is_right_var)?;
    computed.enforce_equal(root_var)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bls12_381::Fr;

    #[test]
    fn native_path_verifies_against_root() {
        let leaves: Vec<Fr> = (0..8u64).map(Fr::from).collect();
        let tree = SimpleMerkleTree::build(leaves.clone());
        let root = tree.root();

        for (i, leaf) in leaves.iter().enumerate() {
            let path = tree.path_for(i);
            // Reconstrucción manual de la raíz a partir del path, para
            // validar la lógica nativa antes de tocar el circuito.
            let mut current = *leaf;
            for level in 0..TREE_DEPTH {
                current = if path.is_right[level] {
                    secure_hash(path.siblings[level], current)
                } else {
                    secure_hash(current, path.siblings[level])
                };
            }
            assert_eq!(current, root, "el path reconstruido no coincide con la raiz para la hoja {i}");
        }
    }

    // ⚠️ §822 · Las cuatro raíces del árbol DENSO, el de antes del §822, en
    // `serialize_compressed` y hexadecimal. Medido el 15-10-2026 sobre 0315b14
    // (zk-core idéntico a 4d8017d): un test temporal, que no se queda, imprimió
    // con el `build` denso la raíz y los caminos -hermanos y bits- de los
    // índices 0, 3, 4, 100 y 2^20 - 1 de estos cuatro casos (213,14 s en
    // `--release`); el mismo test sobre el disperso imprimió lo mismo byte a
    // byte (md5 150177c071544af39422455482a2cdb9 las dos salidas). Los caminos
    // no se copian: en esa salida los bits de cada camino son los de su índice,
    // y aquí se exigen así, y cada camino tiene que subir a su raíz de aquí.
    const KAT_RAIZ_OCHO: &str = "08fc742690deecd06310fed5feafef610265cc7bc92a633b78de8083ec382869"; // hojas 0..8
    const KAT_RAIZ_CINCO: &str = "ce41bb85254f7b38c0c5ffde39cb609636e1bab424acc89c8f67cc85c68baa5c"; // hojas 1..6
    const KAT_RAIZ_VACIO: &str = "77b4c587eacd0344f92b4ddbd3d63caec93a72ed6b88b164225ec03a958a3c67"; // ninguna hoja
    const KAT_RAIZ_UNA: &str = "5657ca70456bc0c8afbf4bdba8eec3b9b5dc4019ace023c56171c973a6887623"; // la hoja 7

    fn hex_comprimido(x: Fr) -> String {
        use ark_serialize::CanonicalSerialize;
        let mut bytes = Vec::new();
        x.serialize_compressed(&mut bytes).unwrap();
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }

    /// ⚠️ §822 · El disperso da la raíz del denso en los cuatro casos del KAT, y
    /// el camino de cada índice -dentro de las hojas dadas, en el relleno y en
    /// la última hoja del árbol, 2^20 - 1- lleva los bits de su índice y sube
    /// a ella. La hoja de un índice sin hoja dada es el relleno, `F::zero()`,
    /// como en el denso.
    #[test]
    fn el_arbol_disperso_da_las_raices_del_denso() {
        use ark_ff::Zero;
        let casos: [(&str, Vec<Fr>, &str); 4] = [
            ("ocho", (0..8u64).map(Fr::from).collect(), KAT_RAIZ_OCHO),
            ("cinco", (1..6u64).map(Fr::from).collect(), KAT_RAIZ_CINCO),
            ("vacio", Vec::new(), KAT_RAIZ_VACIO),
            ("una", vec![Fr::from(7u64)], KAT_RAIZ_UNA),
        ];
        for (nombre, hojas, kat) in casos {
            let tree = SimpleMerkleTree::build(hojas.clone());
            assert_eq!(
                hex_comprimido(tree.root()),
                kat,
                "la raiz del caso {nombre} no es la del arbol denso"
            );
            for i in [0, 3, 4, 100, (1usize << TREE_DEPTH) - 1] {
                let path = tree.path_for(i);
                assert_eq!(path.siblings.len(), TREE_DEPTH);
                assert_eq!(path.is_right.len(), TREE_DEPTH);
                let bits: Vec<bool> = (0..TREE_DEPTH).map(|l| (i >> l) & 1 == 1).collect();
                assert_eq!(
                    path.is_right, bits,
                    "los bits del camino del indice {i} del caso {nombre} no son los de su indice"
                );
                let mut current = hojas.get(i).copied().unwrap_or_else(Fr::zero);
                for level in 0..TREE_DEPTH {
                    current = if path.is_right[level] {
                        secure_hash(path.siblings[level], current)
                    } else {
                        secure_hash(current, path.siblings[level])
                    };
                }
                assert_eq!(
                    hex_comprimido(current),
                    kat,
                    "el camino del indice {i} del caso {nombre} no sube a la raiz del denso"
                );
            }
        }
    }

    /// ⚠️ §822 · Un índice que la profundidad no alcanza no tiene camino. El
    /// denso caía por el índice de su vector; el disperso respondería con
    /// subárboles vacíos, y por eso lo dice su `assert`.
    #[test]
    #[should_panic(expected = "indice fuera del arbol: la profundidad configurada no lo alcanza")]
    fn el_camino_fuera_del_arbol_entra_en_panico() {
        let tree = SimpleMerkleTree::build((0..8u64).map(Fr::from).collect::<Vec<Fr>>());
        let _ = tree.path_for(1usize << TREE_DEPTH);
    }

    /// ⚠️ §822 · Más hojas de las que caben en 2^TREE_DEPTH no hacen árbol. El
    /// `assert` de `build` es el de antes del §822, sin cambiar, y ningún test lo
    /// pedía. Que el árbol lleno, 2^20 hojas, sí cabe no se prueba aquí:
    /// construirlo cuesta más de un millón de hashes, como el denso.
    #[test]
    #[should_panic(expected = "demasiadas hojas para la profundidad configurada")]
    fn el_arbol_con_mas_hojas_de_las_que_caben_entra_en_panico() {
        use ark_ff::Zero;
        let _ = SimpleMerkleTree::build(vec![Fr::zero(); (1usize << TREE_DEPTH) + 1]);
    }
}
