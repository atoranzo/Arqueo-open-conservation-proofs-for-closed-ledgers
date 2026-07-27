//! Circuito de cumplimiento unificado: combina las cuatro piezas ya
//! verificadas por separado (range check, Poseidon, árbol de Merkle,
//! nullifier) en un único `Circuit`, equivalente en Halo2 a
//! `zk-core::circuit_with_state::ComplianceCircuitWithState`.
//!
//! ## Entradas públicas (en este orden, misma convención que zk-core)
//! 1. `state_root` — raíz del árbol de Merkle.
//! 2. `regulatory_limit` — límite normativo.
//! 3. `nullifier` — para prevenir doble gasto.
//!
//! ## Testigos privados
//! `account_id`, `balance`, `account_nonce`, `amount`, y el camino de
//! Merkle (`siblings`, `path_bits`).
//!
//! ## ⚠️ Riesgo de esta pieza: de integración, no de API nueva
//!
//! Cada gadget individual (range check, Poseidon, selección, nullifier)
//! ya está verificado. El riesgo aquí es de "cableado": que las
//! restricciones de igualdad entre piezas (p. ej. que `diff_balance`
//! realmente sea `balance - amount`, no un valor inventado) estén bien
//! puestas. Ver la puerta `s_sub` más abajo — sin ella, el range check de
//! las diferencias sería decorativo.

use ff::PrimeField;
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
pub const VALUE_BITS: usize = 64;
pub const NULLIFIER_DOMAIN: u64 = 0x4E554C4C;

fn value_to_bits_le(value: Fp) -> Vec<bool> {
    let repr = value.to_repr();
    let bytes: &[u8] = repr.as_ref();
    (0..VALUE_BITS)
        .map(|i| (bytes[i / 8] >> (i % 8)) & 1 == 1)
        .collect()
}

#[derive(Clone, Debug)]
pub struct ComplianceConfig {
    pow5_config: Pow5Config<Fp, WIDTH, RATE>,
    witness_column: Column<Advice>,

    // -- range check --
    rc_bit: Column<Advice>,
    rc_running_sum: Column<Advice>,
    rc_power_of_two: Column<halo2_proofs::plonk::Fixed>,
    rc_s_bool: Selector,
    rc_s_accumulate: Selector,

    // -- seleccion left/right del arbol --
    m_bit: Column<Advice>,
    m_current: Column<Advice>,
    m_sibling: Column<Advice>,
    m_left: Column<Advice>,
    m_right: Column<Advice>,
    m_s_bool: Selector,
    m_s_select: Selector,

    // -- resta atada (diff = a - b), para balance-amount y limit-amount --
    sub_a: Column<Advice>,
    sub_b: Column<Advice>,
    sub_diff: Column<Advice>,
    s_sub: Selector,

    instance: Column<Instance>, // [state_root, regulatory_limit, nullifier]
}

impl ComplianceConfig {
    fn hash_pair(
        &self,
        mut layouter: impl Layouter<Fp>,
        a: AssignedCell<Fp, Fp>,
        b: AssignedCell<Fp, Fp>,
    ) -> Result<AssignedCell<Fp, Fp>, Error> {
        let chip = Pow5Chip::construct(self.pow5_config.clone());
        let hasher = Hash::<Fp, Pow5Chip<Fp, WIDTH, RATE>, P128Pow5T3, ConstantLength<2>, WIDTH, RATE>::init(
            chip,
            layouter.namespace(|| "init poseidon"),
        )?;
        hasher.hash(layouter.namespace(|| "hash"), [a, b])
    }

    fn select_pair(
        &self,
        mut layouter: impl Layouter<Fp>,
        current_cell: &AssignedCell<Fp, Fp>,
        sibling_val: Value<Fp>,
        bit_val: Value<Fp>,
    ) -> Result<(AssignedCell<Fp, Fp>, AssignedCell<Fp, Fp>), Error> {
        layouter.assign_region(
            || "select left/right",
            |mut region| {
                self.m_s_bool.enable(&mut region, 0)?;
                self.m_s_select.enable(&mut region, 0)?;

                region.assign_advice(|| "bit", self.m_bit, 0, || bit_val)?;

                let current_copy = region.assign_advice(
                    || "current",
                    self.m_current,
                    0,
                    || current_cell.value().copied(),
                )?;
                region.constrain_equal(current_cell.cell(), current_copy.cell())?;

                let sibling_assigned =
                    region.assign_advice(|| "sibling", self.m_sibling, 0, || sibling_val)?;

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

                let left_cell =
                    region.assign_advice(|| "left", self.m_left, 0, || left_right.map(|(l, _)| l))?;
                let right_cell =
                    region.assign_advice(|| "right", self.m_right, 0, || left_right.map(|(_, r)| r))?;

                Ok((left_cell, right_cell))
            },
        )
    }

    /// Fuerza `target` a estar en [0, 2^VALUE_BITS), atando la
    /// reconstrucción de bits al valor REAL de `target` mediante una
    /// restricción de igualdad — no solo comprobando algún valor
    /// arbitrario que "coincida por casualidad".
    fn enforce_range_tied_to(
        &self,
        mut layouter: impl Layouter<Fp>,
        target: &AssignedCell<Fp, Fp>,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "range check",
            |mut region| {
                let bits = target.value().copied().map(value_to_bits_le);
                let mut running_sum_value = Value::known(Fp::zero());
                let mut last_cell: Option<AssignedCell<Fp, Fp>> = None;

                for i in 0..VALUE_BITS {
                    let bit_i: Value<Fp> = bits
                        .as_ref()
                        .map(|bs| if bs[i] { Fp::one() } else { Fp::zero() });
                    region.assign_advice(|| format!("bit {i}"), self.rc_bit, i, || bit_i)?;

                    let power_i = Fp::from(1u64 << i.min(63));
                    region.assign_fixed(
                        || format!("2^{i}"),
                        self.rc_power_of_two,
                        i,
                        || Value::known(power_i),
                    )?;

                    self.rc_s_bool.enable(&mut region, i)?;

                    if i == 0 {
                        running_sum_value = bit_i.map(|b| b * power_i);
                        last_cell = Some(region.assign_advice(
                            || "running_sum[0]",
                            self.rc_running_sum,
                            0,
                            || running_sum_value,
                        )?);
                    } else {
                        self.rc_s_accumulate.enable(&mut region, i)?;
                        running_sum_value = running_sum_value
                            .zip(bit_i)
                            .map(|(sum, b)| sum + b * power_i);
                        last_cell = Some(region.assign_advice(
                            || format!("running_sum[{i}]"),
                            self.rc_running_sum,
                            i,
                            || running_sum_value,
                        )?);
                    }
                }

                region.constrain_equal(target.cell(), last_cell.unwrap().cell())?;
                Ok(())
            },
        )
    }

    /// Calcula `diff = a - b` y lo ATA a las celdas reales de `a` y `b`
    /// mediante restricciones de igualdad + una puerta aritmética — esto
    /// es lo que impide que alguien declare una diferencia inventada
    /// para pasar el range check sin que corresponda a la resta real.
    fn subtract_tied(
        &self,
        mut layouter: impl Layouter<Fp>,
        a_cell: &AssignedCell<Fp, Fp>,
        b_cell: &AssignedCell<Fp, Fp>,
    ) -> Result<AssignedCell<Fp, Fp>, Error> {
        layouter.assign_region(
            || "diff = a - b",
            |mut region| {
                self.s_sub.enable(&mut region, 0)?;

                let a_copy = region.assign_advice(|| "a", self.sub_a, 0, || a_cell.value().copied())?;
                region.constrain_equal(a_cell.cell(), a_copy.cell())?;

                let b_copy = region.assign_advice(|| "b", self.sub_b, 0, || b_cell.value().copied())?;
                region.constrain_equal(b_cell.cell(), b_copy.cell())?;

                let diff_val = a_cell
                    .value()
                    .copied()
                    .zip(b_cell.value().copied())
                    .map(|(a, b)| a - b);
                region.assign_advice(|| "diff", self.sub_diff, 0, || diff_val)
            },
        )
    }
}

/// Circuito de cumplimiento completo: equivalente en Halo2 de
/// `ComplianceCircuitWithState` en Arkworks.
pub struct ComplianceCircuit {
    pub account_id: Value<Fp>,
    pub balance: Value<Fp>,
    pub account_nonce: Value<Fp>,
    pub amount: Value<Fp>,
    pub regulatory_limit: Value<Fp>,
    pub siblings: Vec<Value<Fp>>,  // longitud TREE_DEPTH
    pub path_bits: Vec<Value<Fp>>, // longitud TREE_DEPTH, 0/1 como Fp
}

impl Default for ComplianceCircuit {
    fn default() -> Self {
        Self {
            account_id: Value::unknown(),
            balance: Value::unknown(),
            account_nonce: Value::unknown(),
            amount: Value::unknown(),
            regulatory_limit: Value::unknown(),
            siblings: vec![Value::unknown(); TREE_DEPTH],
            path_bits: vec![Value::unknown(); TREE_DEPTH],
        }
    }
}

impl Circuit<Fp> for ComplianceCircuit {
    type Config = ComplianceConfig;
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

        let witness_column = meta.advice_column();
        let rc_bit = meta.advice_column();
        let rc_running_sum = meta.advice_column();
        let rc_power_of_two = meta.fixed_column();
        let m_bit = meta.advice_column();
        let m_current = meta.advice_column();
        let m_sibling = meta.advice_column();
        let m_left = meta.advice_column();
        let m_right = meta.advice_column();
        let sub_a = meta.advice_column();
        let sub_b = meta.advice_column();
        let sub_diff = meta.advice_column();
        let instance = meta.instance_column();

        meta.enable_equality(instance);
        meta.enable_equality(witness_column);
        meta.enable_equality(rc_running_sum);
        meta.enable_equality(m_current);
        meta.enable_equality(m_left);
        meta.enable_equality(m_right);
        meta.enable_equality(sub_a);
        meta.enable_equality(sub_b);
        meta.enable_equality(sub_diff);
        for column in state.iter() {
            meta.enable_equality(*column);
        }
        meta.enable_constant(rc_b[0]);

        let rc_s_bool = meta.selector();
        let rc_s_accumulate = meta.selector();
        let m_s_bool = meta.selector();
        let m_s_select = meta.selector();
        let s_sub = meta.selector();

        meta.create_gate("rc: bit is boolean", |meta| {
            let bit = meta.query_advice(rc_bit, Rotation::cur());
            let s = meta.query_selector(rc_s_bool);
            vec![s * bit.clone() * (bit - Expression::Constant(Fp::one()))]
        });

        meta.create_gate("rc: accumulate weighted bit", |meta| {
            let sum_prev = meta.query_advice(rc_running_sum, Rotation::prev());
            let sum_cur = meta.query_advice(rc_running_sum, Rotation::cur());
            let bit_cur = meta.query_advice(rc_bit, Rotation::cur());
            let power_cur = meta.query_fixed(rc_power_of_two);
            let s = meta.query_selector(rc_s_accumulate);
            vec![s * (sum_cur - (sum_prev + bit_cur * power_cur))]
        });

        meta.create_gate("merkle: bit is boolean", |meta| {
            let bit = meta.query_advice(m_bit, Rotation::cur());
            let s = meta.query_selector(m_s_bool);
            vec![s * bit.clone() * (bit - Expression::Constant(Fp::one()))]
        });

        meta.create_gate("merkle: select left/right", |meta| {
            let bit = meta.query_advice(m_bit, Rotation::cur());
            let current = meta.query_advice(m_current, Rotation::cur());
            let sibling = meta.query_advice(m_sibling, Rotation::cur());
            let left = meta.query_advice(m_left, Rotation::cur());
            let right = meta.query_advice(m_right, Rotation::cur());
            let s = meta.query_selector(m_s_select);
            let expected_left = current.clone() + bit * (sibling.clone() - current.clone());
            let expected_right = current + sibling - left.clone();
            vec![
                s.clone() * (left - expected_left),
                s * (right - expected_right),
            ]
        });

        meta.create_gate("sub: diff = a - b", |meta| {
            let a = meta.query_advice(sub_a, Rotation::cur());
            let b = meta.query_advice(sub_b, Rotation::cur());
            let diff = meta.query_advice(sub_diff, Rotation::cur());
            let s = meta.query_selector(s_sub);
            vec![s * (diff - (a - b))]
        });

        let pow5_config = Pow5Chip::configure::<P128Pow5T3>(meta, state, partial_sbox, rc_a, rc_b);

        ComplianceConfig {
            pow5_config,
            witness_column,
            rc_bit,
            rc_running_sum,
            rc_power_of_two,
            rc_s_bool,
            rc_s_accumulate,
            m_bit,
            m_current,
            m_sibling,
            m_left,
            m_right,
            m_s_bool,
            m_s_select,
            sub_a,
            sub_b,
            sub_diff,
            s_sub,
            instance,
        }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<Fp>,
    ) -> Result<(), Error> {
        // --- 1. Cargar testigos escalares ---
        let (account_id_cell, balance_cell, nonce_cell, amount_cell, limit_cell) = layouter
            .assign_region(
                || "cargar testigos",
                |mut region| {
                    let account_id_cell = region.assign_advice(
                        || "account_id",
                        config.witness_column,
                        0,
                        || self.account_id,
                    )?;
                    let balance_cell =
                        region.assign_advice(|| "balance", config.witness_column, 1, || self.balance)?;
                    let nonce_cell = region.assign_advice(
                        || "account_nonce",
                        config.witness_column,
                        2,
                        || self.account_nonce,
                    )?;
                    let amount_cell =
                        region.assign_advice(|| "amount", config.witness_column, 3, || self.amount)?;
                    let limit_cell = region.assign_advice(
                        || "regulatory_limit",
                        config.witness_column,
                        4,
                        || self.regulatory_limit,
                    )?;
                    Ok((account_id_cell, balance_cell, nonce_cell, amount_cell, limit_cell))
                },
            )?;

        // regulatory_limit es publico: atarlo al instance (fila 1).
        layouter.constrain_instance(limit_cell.cell(), config.instance, 1)?;

        // --- 2. Hoja del arbol: leaf = Poseidon(Poseidon(account_id, balance), nonce) ---
        let inner_leaf = config.hash_pair(
            layouter.namespace(|| "leaf inner"),
            account_id_cell.clone(),
            balance_cell.clone(),
        )?;
        let leaf_cell =
            config.hash_pair(layouter.namespace(|| "leaf"), inner_leaf, nonce_cell.clone())?;

        // --- 3. Camino de Merkle: 20 niveles ---
        let mut current_cell = leaf_cell;
        for level in 0..TREE_DEPTH {
            let (left, right) = config.select_pair(
                layouter.namespace(|| format!("select nivel {level}")),
                &current_cell,
                self.siblings[level],
                self.path_bits[level],
            )?;
            current_cell =
                config.hash_pair(layouter.namespace(|| format!("hash nivel {level}")), left, right)?;
        }
        // La raiz reconstruida debe coincidir con la publica (fila 0).
        layouter.constrain_instance(current_cell.cell(), config.instance, 0)?;

        // --- 4. Nullifier: Poseidon(Poseidon(DOMAIN, account_id), nonce) ---
        let domain_cell = layouter.assign_region(
            || "cargar domain",
            |mut region| {
                region.assign_advice_from_constant(
                    || "domain",
                    config.witness_column,
                    0,
                    Fp::from(NULLIFIER_DOMAIN),
                )
            },
        )?;
        let inner_null = config.hash_pair(
            layouter.namespace(|| "nullifier inner"),
            domain_cell,
            account_id_cell,
        )?;
        let nullifier_cell =
            config.hash_pair(layouter.namespace(|| "nullifier"), inner_null, nonce_cell)?;
        layouter.constrain_instance(nullifier_cell.cell(), config.instance, 2)?;

        // --- 5. Range checks: balance, amount, limit ---
        config.enforce_range_tied_to(layouter.namespace(|| "range balance"), &balance_cell)?;
        config.enforce_range_tied_to(layouter.namespace(|| "range amount"), &amount_cell)?;
        config.enforce_range_tied_to(layouter.namespace(|| "range limit"), &limit_cell)?;

        // --- 6. Solvencia: amount <= balance, amount <= regulatory_limit ---
        let diff_balance =
            config.subtract_tied(layouter.namespace(|| "diff_balance"), &balance_cell, &amount_cell)?;
        config.enforce_range_tied_to(layouter.namespace(|| "range diff_balance"), &diff_balance)?;

        let diff_limit =
            config.subtract_tied(layouter.namespace(|| "diff_limit"), &limit_cell, &amount_cell)?;
        config.enforce_range_tied_to(layouter.namespace(|| "range diff_limit"), &diff_limit)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use halo2_proofs::dev::MockProver;

    fn native_hash(a: Fp, b: Fp) -> Fp {
        halo2_gadgets::poseidon::primitives::Hash::<Fp, P128Pow5T3, ConstantLength<2>, WIDTH, RATE>::init()
            .hash([a, b])
    }

    fn native_leaf(account_id: Fp, balance: Fp, nonce: Fp) -> Fp {
        native_hash(native_hash(account_id, balance), nonce)
    }

    fn native_nullifier(account_id: Fp, nonce: Fp) -> Fp {
        let domain = Fp::from(NULLIFIER_DOMAIN);
        native_hash(native_hash(domain, account_id), nonce)
    }

    struct NativeTree {
        levels: Vec<Vec<Fp>>,
    }
    impl NativeTree {
        fn build(mut leaves: Vec<Fp>) -> Self {
            let target = 1usize << TREE_DEPTH;
            leaves.resize(target, Fp::zero());
            let mut levels = vec![leaves];
            for _ in 0..TREE_DEPTH {
                let prev = levels.last().unwrap();
                let next: Vec<Fp> = prev.chunks(2).map(|p| native_hash(p[0], p[1])).collect();
                levels.push(next);
            }
            Self { levels }
        }
        fn root(&self) -> Fp {
            self.levels.last().unwrap()[0]
        }
        fn path_for(&self, index: usize) -> (Vec<Fp>, Vec<bool>) {
            let mut siblings = Vec::with_capacity(TREE_DEPTH);
            let mut is_right = Vec::with_capacity(TREE_DEPTH);
            let mut idx = index;
            for level in 0..TREE_DEPTH {
                siblings.push(self.levels[level][idx ^ 1]);
                is_right.push(idx % 2 == 1);
                idx /= 2;
            }
            (siblings, is_right)
        }
    }

    fn bool_to_fp(b: bool) -> Fp {
        if b {
            Fp::one()
        } else {
            Fp::zero()
        }
    }

    /// EL TEST CLAVE: una transaccion completamente valida (cuenta real
    /// en el arbol, fondos suficientes, dentro del limite, nullifier
    /// correcto) satisface el circuito unificado completo.
    #[test]
    fn fully_valid_transaction_satisfies_circuit() {
        let k = 15;

        let account_id = Fp::from(12345);
        let balance = Fp::from(1_000_000u64);
        let nonce = Fp::from(1);
        let amount = Fp::from(250_000u64);
        let regulatory_limit = Fp::from(500_000u64);

        let leaf = native_leaf(account_id, balance, nonce);
        let mut leaves = vec![Fp::from(1), Fp::from(2), Fp::from(3), leaf];
        leaves.resize(8, Fp::zero());
        let tree = NativeTree::build(leaves);
        let root = tree.root();
        let (siblings, is_right) = tree.path_for(3);
        let nullifier = native_nullifier(account_id, nonce);

        let circuit = ComplianceCircuit {
            account_id: Value::known(account_id),
            balance: Value::known(balance),
            account_nonce: Value::known(nonce),
            amount: Value::known(amount),
            regulatory_limit: Value::known(regulatory_limit),
            siblings: siblings.into_iter().map(Value::known).collect(),
            path_bits: is_right.into_iter().map(|b| Value::known(bool_to_fp(b))).collect(),
        };

        let public_input = vec![root, regulatory_limit, nullifier];
        let prover = MockProver::run(k, &circuit, vec![public_input]).unwrap();
        assert_eq!(prover.verify(), Ok(()));
    }

    /// EL TEST DE SOLIDEZ MAS IMPORTANTE: gastar mas de lo que se tiene
    /// (amount > balance) debe romper el circuito, gracias a la puerta
    /// de resta atada + el range check de diff_balance.
    #[test]
    fn insufficient_balance_fails_circuit() {
        let k = 15;

        let account_id = Fp::from(12345);
        let balance = Fp::from(100_000u64); // saldo insuficiente
        let nonce = Fp::from(1);
        let amount = Fp::from(250_000u64); // amount > balance
        let regulatory_limit = Fp::from(500_000u64);

        let leaf = native_leaf(account_id, balance, nonce);
        let mut leaves = vec![Fp::from(1), Fp::from(2), Fp::from(3), leaf];
        leaves.resize(8, Fp::zero());
        let tree = NativeTree::build(leaves);
        let root = tree.root();
        let (siblings, is_right) = tree.path_for(3);
        let nullifier = native_nullifier(account_id, nonce);

        let circuit = ComplianceCircuit {
            account_id: Value::known(account_id),
            balance: Value::known(balance),
            account_nonce: Value::known(nonce),
            amount: Value::known(amount),
            regulatory_limit: Value::known(regulatory_limit),
            siblings: siblings.into_iter().map(Value::known).collect(),
            path_bits: is_right.into_iter().map(|b| Value::known(bool_to_fp(b))).collect(),
        };

        let public_input = vec![root, regulatory_limit, nullifier];
        let prover = MockProver::run(k, &circuit, vec![public_input]).unwrap();
        assert!(
            prover.verify().is_err(),
            "CRITICO: gastar mas del saldo real no deberia satisfacer el circuito"
        );
    }
}
