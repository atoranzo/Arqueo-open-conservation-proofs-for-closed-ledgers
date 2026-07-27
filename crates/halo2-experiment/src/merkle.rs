//! Árbol de Merkle de 20 niveles en Halo2, usando Poseidon real
//! (`halo2_gadgets`) y una puerta de selección condicional para elegir el
//! orden izquierda/derecha según el bit de dirección — el equivalente en
//! Halo2 de `zk-core::merkle::enforce_merkle_membership`.
//!
//! ## ⚠️ La pieza más compleja de todo el experimento con Halo2
//!
//! Combina dos gadgets (selección condicional + Poseidon) encadenados 20
//! veces, con restricciones de igualdad (`constrain_equal`) uniendo la
//! salida de cada nivel con la entrada del siguiente. Es razonable
//! esperar más rondas de corrección que en cualquier pieza anterior de
//! este experimento.
//!
//! ## El truco para ahorrar una puerta
//!
//! En vez de seleccionar `left` Y `right` por separado (dos selecciones
//! completas), se aprovecha que `{left, right} = {current, sibling}` como
//! conjunto: solo se selecciona `left` con el bit
//! (`left = current + bit*(sibling - current)`), y `right` sale de una
//! resta simple (`right = current + sibling - left`) — más barato que una
//! segunda selección condicional completa.

use halo2_gadgets::poseidon::{
    primitives::{ConstantLength, P128Pow5T3},
    Hash, Pow5Chip, Pow5Config,
};
use halo2_proofs::pasta::Fp;
use halo2_proofs::{
    circuit::{AssignedCell, Layouter, SimpleFloorPlanner, Value},
    plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Expression, Instance, Selector},
    poly::Rotation,
};

const WIDTH: usize = 3;
const RATE: usize = 2;
pub const TREE_DEPTH: usize = 20;

#[derive(Clone, Debug)]
pub struct MerkleConfig {
    pow5_config: Pow5Config<Fp, WIDTH, RATE>,
    bit: Column<Advice>,
    current: Column<Advice>,
    sibling: Column<Advice>,
    left: Column<Advice>,
    right: Column<Advice>,
    s_bool: Selector,
    s_select: Selector,
    instance: Column<Instance>,
}

/// Circuito que demuestra que `leaf` pertenece a un árbol de Merkle de
/// `TREE_DEPTH` niveles cuya raíz es pública, sin revelar `leaf`, el
/// camino, ni los bits de dirección.
pub struct MerkleCircuit {
    pub leaf: Value<Fp>,
    /// Longitud TREE_DEPTH. `siblings[i]` es el hermano en el nivel i.
    pub siblings: Vec<Value<Fp>>,
    /// Longitud TREE_DEPTH. Cada elemento es 0 o 1 (como `Fp`), no `bool`,
    /// para simplificar la aritmética dentro del circuito.
    pub path_bits: Vec<Value<Fp>>,
}

impl Default for MerkleCircuit {
    fn default() -> Self {
        Self {
            leaf: Value::unknown(),
            siblings: vec![Value::unknown(); TREE_DEPTH],
            path_bits: vec![Value::unknown(); TREE_DEPTH],
        }
    }
}

impl Circuit<Fp> for MerkleCircuit {
    type Config = MerkleConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
        let state: [Column<Advice>; WIDTH] = std::array::from_fn(|_| meta.advice_column());
        let partial_sbox = meta.advice_column();
        let rc_a: [Column<halo2_proofs::plonk::Fixed>; WIDTH] =
            std::array::from_fn(|_| meta.fixed_column());
        let rc_b: [Column<halo2_proofs::plonk::Fixed>; WIDTH] =
            std::array::from_fn(|_| meta.fixed_column());

        let bit = meta.advice_column();
        let current = meta.advice_column();
        let sibling = meta.advice_column();
        let left = meta.advice_column();
        let right = meta.advice_column();
        let instance = meta.instance_column();

        meta.enable_equality(instance);
        meta.enable_equality(current);
        meta.enable_equality(left);
        meta.enable_equality(right);
        for column in state.iter() {
            meta.enable_equality(*column);
        }
        meta.enable_constant(rc_b[0]);

        let s_bool = meta.selector();
        let s_select = meta.selector();

        meta.create_gate("bit is boolean", |meta| {
            let bit = meta.query_advice(bit, Rotation::cur());
            let s = meta.query_selector(s_bool);
            vec![s * bit.clone() * (bit - Expression::Constant(Fp::one()))]
        });

        meta.create_gate("select left/right by direction bit", |meta| {
            let bit = meta.query_advice(bit, Rotation::cur());
            let current = meta.query_advice(current, Rotation::cur());
            let sibling = meta.query_advice(sibling, Rotation::cur());
            let left = meta.query_advice(left, Rotation::cur());
            let right = meta.query_advice(right, Rotation::cur());
            let s = meta.query_selector(s_select);

            // left = current + bit * (sibling - current)
            let expected_left = current.clone() + bit * (sibling.clone() - current.clone());
            // right = current + sibling - left  (el "resto" del par)
            let expected_right = current + sibling - left.clone();

            vec![
                s.clone() * (left - expected_left),
                s * (right - expected_right),
            ]
        });

        let pow5_config =
            Pow5Chip::configure::<P128Pow5T3>(meta, state, partial_sbox, rc_a, rc_b);

        MerkleConfig {
            pow5_config,
            bit,
            current,
            sibling,
            left,
            right,
            s_bool,
            s_select,
            instance,
        }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<Fp>,
    ) -> Result<(), Error> {
        // Cargar la hoja como punto de partida de "current".
        let mut current_cell: AssignedCell<Fp, Fp> = layouter.assign_region(
            || "cargar hoja",
            |mut region| region.assign_advice(|| "leaf", config.current, 0, || self.leaf),
        )?;

        for level in 0..TREE_DEPTH {
            let bit_val = self.path_bits[level];
            let sibling_val = self.siblings[level];

            let (left_cell, right_cell) = layouter.assign_region(
                || format!("seleccionar left/right, nivel {level}"),
                |mut region| {
                    config.s_bool.enable(&mut region, 0)?;
                    config.s_select.enable(&mut region, 0)?;

                    region.assign_advice(|| "bit", config.bit, 0, || bit_val)?;

                    // "current" se vuelve a testificar aquí, pero atado
                    // mediante restricción de igualdad a la celda REAL de
                    // la iteración anterior — no es un valor inventado de
                    // nuevo, es el mismo valor, solo con una copia formal
                    // hacia esta región.
                    let current_copy = region.assign_advice(
                        || "current",
                        config.current,
                        0,
                        || current_cell.value().copied(),
                    )?;
                    region.constrain_equal(current_cell.cell(), current_copy.cell())?;

                    let sibling_assigned =
                        region.assign_advice(|| "sibling", config.sibling, 0, || sibling_val)?;

                    let combined = current_copy
                        .value()
                        .copied()
                        .zip(sibling_assigned.value().copied())
                        .zip(bit_val);

                    let left_right: Value<(Fp, Fp)> = combined.map(|((cur, sib), b)| {
                        let left = cur + b * (sib - cur);
                        let right = cur + sib - left;
                        (left, right)
                    });

                    let left_cell = region.assign_advice(
                        || "left",
                        config.left,
                        0,
                        || left_right.map(|(l, _)| l),
                    )?;
                    let right_cell = region.assign_advice(
                        || "right",
                        config.right,
                        0,
                        || left_right.map(|(_, r)| r),
                    )?;

                    Ok((left_cell, right_cell))
                },
            )?;

            // Hashear (left, right) con Poseidon real -> nuevo "current".
            let pow5_chip = Pow5Chip::construct(config.pow5_config.clone());
            let hasher = Hash::<Fp, Pow5Chip<Fp, WIDTH, RATE>, P128Pow5T3, ConstantLength<2>, WIDTH, RATE>::init(
                pow5_chip,
                layouter.namespace(|| format!("init poseidon nivel {level}")),
            )?;
            current_cell = hasher.hash(
                layouter.namespace(|| format!("hash nivel {level}")),
                [left_cell, right_cell],
            )?;
        }

        // La última "current" es la raíz reconstruida; debe coincidir con
        // la raíz pública declarada.
        layouter.constrain_instance(current_cell.cell(), config.instance, 0)
    }
}

/// Hash nativo (fuera de circuito) usando las primitivas de
/// `halo2_gadgets`, para construir árboles de prueba y calcular caminos
/// esperados sin tener que adivinarlos.
fn native_hash(a: Fp, b: Fp) -> Fp {
    halo2_gadgets::poseidon::primitives::Hash::<Fp, P128Pow5T3, ConstantLength<2>, WIDTH, RATE>::init()
        .hash([a, b])
}

/// Árbol de Merkle nativo simple, análogo a `zk_core::merkle::SimpleMerkleTree`
/// pero con el Poseidon real de `halo2_gadgets`, solo para generar datos
/// de prueba.
pub struct NativeMerkleTree {
    levels: Vec<Vec<Fp>>,
}

impl NativeMerkleTree {
    pub fn build(mut leaves: Vec<Fp>) -> Self {
        let target_len = 1usize << TREE_DEPTH;
        assert!(leaves.len() <= target_len);
        leaves.resize(target_len, Fp::zero());

        let mut levels = vec![leaves];
        for _ in 0..TREE_DEPTH {
            let prev = levels.last().unwrap();
            let next: Vec<Fp> = prev.chunks(2).map(|pair| native_hash(pair[0], pair[1])).collect();
            levels.push(next);
        }
        Self { levels }
    }

    pub fn root(&self) -> Fp {
        self.levels.last().unwrap()[0]
    }

    pub fn path_for(&self, index: usize) -> (Vec<Fp>, Vec<bool>) {
        let mut siblings = Vec::with_capacity(TREE_DEPTH);
        let mut is_right = Vec::with_capacity(TREE_DEPTH);
        let mut idx = index;
        for level in 0..TREE_DEPTH {
            let sibling_idx = idx ^ 1;
            siblings.push(self.levels[level][sibling_idx]);
            is_right.push(idx % 2 == 1);
            idx /= 2;
        }
        (siblings, is_right)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use halo2_proofs::dev::MockProver;

    fn bool_to_fp(b: bool) -> Fp {
        if b { Fp::one() } else { Fp::zero() }
    }

    /// EL TEST CLAVE: una hoja real, con su camino real hacia la raíz,
    /// satisface el circuito completo (20 niveles, Poseidon real,
    /// selección condicional real).
    #[test]
    fn valid_leaf_and_path_satisfy_circuit() {
        let k = 13; // 20 niveles * (~select + ~poseidon completo) necesita bastantes filas.

        let leaf = Fp::from(42);
        let mut leaves = vec![Fp::from(1), Fp::from(2), Fp::from(3), leaf];
        leaves.resize(8, Fp::zero());
        // El resto se rellena con ceros hasta 2^TREE_DEPTH dentro de `build`.
        let tree = NativeMerkleTree::build(leaves);
        let root = tree.root();
        let (siblings, is_right) = tree.path_for(3); // la hoja "leaf" quedo en el indice 3

        let circuit = MerkleCircuit {
            leaf: Value::known(leaf),
            siblings: siblings.into_iter().map(Value::known).collect(),
            path_bits: is_right.into_iter().map(|b| Value::known(bool_to_fp(b))).collect(),
        };

        let prover = MockProver::run(k, &circuit, vec![vec![root]]).unwrap();
        assert_eq!(prover.verify(), Ok(()));
    }

    /// Confirma que declarar una raíz pública INCORRECTA hace fallar el
    /// circuito, incluso con un camino real y válido por lo demás.
    #[test]
    fn wrong_root_fails_circuit() {
        let k = 13;

        let leaf = Fp::from(42);
        let mut leaves = vec![Fp::from(1), Fp::from(2), Fp::from(3), leaf];
        leaves.resize(8, Fp::zero());
        let tree = NativeMerkleTree::build(leaves);
        let (siblings, is_right) = tree.path_for(3);
        let wrong_root = Fp::from(999_999_999);

        let circuit = MerkleCircuit {
            leaf: Value::known(leaf),
            siblings: siblings.into_iter().map(Value::known).collect(),
            path_bits: is_right.into_iter().map(|b| Value::known(bool_to_fp(b))).collect(),
        };

        let prover = MockProver::run(k, &circuit, vec![vec![wrong_root]]).unwrap();
        assert!(prover.verify().is_err());
    }
}
