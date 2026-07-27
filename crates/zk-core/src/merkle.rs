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

/// Árbol de Merkle simple y completo (no disperso), solo para
/// pruebas/demostración de wiring. Un árbol disperso (sparse Merkle tree)
/// sería lo apropiado para un ledger real con cuentas identificadas por
/// hash de dirección, pero añade complejidad no esencial para validar la
/// estructura del circuito.
pub struct SimpleMerkleTree<F: PrimeField> {
    levels: Vec<Vec<F>>, // levels[0] = hojas, levels[ultimo] = [root]
}

impl<F: PrimeField + Absorb> SimpleMerkleTree<F> {
    /// Construye el árbol a partir de una lista de hojas. Rellena hasta
    /// 2^TREE_DEPTH con un valor de relleno (F::zero()) si hace falta.
    pub fn build(mut leaves: Vec<F>) -> Self {
        let target_len = 1usize << TREE_DEPTH;
        assert!(
            leaves.len() <= target_len,
            "demasiadas hojas para la profundidad configurada"
        );
        leaves.resize(target_len, F::zero());

        let mut levels = vec![leaves];
        for _ in 0..TREE_DEPTH {
            let prev = levels.last().unwrap();
            let mut next = Vec::with_capacity(prev.len() / 2);
            for pair in prev.chunks(2) {
                next.push(secure_hash(pair[0], pair[1]));
            }
            levels.push(next);
        }
        Self { levels }
    }

    pub fn root(&self) -> F {
        self.levels.last().unwrap()[0]
    }

    /// Genera el camino de Merkle para la hoja en `index`.
    pub fn path_for(&self, index: usize) -> MerklePath<F> {
        let mut siblings = Vec::with_capacity(TREE_DEPTH);
        let mut is_right = Vec::with_capacity(TREE_DEPTH);
        let mut idx = index;

        for level in 0..TREE_DEPTH {
            let sibling_idx = idx ^ 1;
            siblings.push(self.levels[level][sibling_idx]);
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
}
