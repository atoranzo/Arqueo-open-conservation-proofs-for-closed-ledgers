//! Circuito de **partida doble** en Halo2 — equivalente del
//! `zk-core::circuit_double_entry` de Groth16.
//!
//! Demuestra la transición de estado completa de una transferencia:
//!
//! ```text
//! saldo_emisor_nuevo   = saldo_emisor   - importe   (ADEUDO)
//! saldo_receptor_nuevo = saldo_receptor + importe   (ABONO)
//! ```
//!
//! La conservación no es una tautología porque **ambos saldos están
//! comprometidos en el árbol de Merkle** y **ambas raíces son públicas**.
//! Ver la explicación completa en `zk-core/src/circuit_double_entry.rs`.
//!
//! ## La secuencia de actualización, y por qué el orden importa
//!
//! El árbol se actualiza en dos pasos encadenados: emisor contra
//! `root_old`, se recalcula `root_mid`, y el receptor se verifica contra
//! **`root_mid`**, no contra `root_old`. Si emisor y receptor comparten
//! ancestros —y con profundidad 20 comparten los niveles altos— usar la
//! raíz antigua produciría un fallo intermitente.
//!
//! ## Coste: este circuito es mucho más grande
//!
//! Sube de ~24 invocaciones de Poseidon a ~90: cuatro recorridos del
//! árbol (emisor antes/después, receptor antes/después) en vez de uno.
//! Eso obliga a un `k` mayor que el del circuito de solvencia. El valor
//! concreto lo determina la evidencia, no la estimación — ver los tests.
//!
//! ## Inputs públicos (columna `instance`, EN ESTE ORDEN)
//!
//! | fila | valor |
//! |---|---|
//! | 0 | `root_old` |
//! | 1 | `root_new` |
//! | 2 | `regulatory_limit` |
//! | 3 | `nullifier` |

use halo2_gadgets::poseidon::primitives::{ConstantLength, P128Pow5T3};
use halo2_gadgets::poseidon::{Hash as PoseidonHash, Pow5Chip, Pow5Config};
use ff::PrimeField;
use halo2_proofs::arithmetic::Field;
use halo2_proofs::circuit::{AssignedCell, Layouter, SimpleFloorPlanner, Value};
use halo2_proofs::pasta::Fp;
use halo2_proofs::plonk::{
    Advice, Circuit, Column, ConstraintSystem, Error, Expression, Fixed, Instance, Selector,
};
use halo2_proofs::poly::Rotation;

use crate::compliance_circuit::{NULLIFIER_DOMAIN, TREE_DEPTH};

const WIDTH: usize = 3;
const RATE: usize = 2;
/// Bits del range check, igual que en el circuito de solvencia.
const RANGE_BITS: usize = 64;

#[derive(Clone, Debug)]
pub struct DoubleEntryConfig {
    pow5_config: Pow5Config<Fp, WIDTH, RATE>,
    witness_column: Column<Advice>,

    // -- range check --
    rc_bit: Column<Advice>,
    rc_running_sum: Column<Advice>,
    rc_power_of_two: Column<Fixed>,
    rc_s_bool: Selector,
    rc_s_accumulate: Selector,

    // -- seleccion left/right del arbol --
    m_bit: Column<Advice>,
    m_current: Column<Advice>,
    m_sibling: Column<Advice>,
    m_left: Column<Advice>,
    m_right: Column<Advice>,
    m_s_select: Selector,

    // -- aritmetica atada --
    ar_a: Column<Advice>,
    ar_b: Column<Advice>,
    ar_out: Column<Advice>,
    s_sub: Selector,
    s_add: Selector,

    instance: Column<Instance>,
}

impl DoubleEntryConfig {
    fn hash_pair(
        &self,
        mut layouter: impl Layouter<Fp>,
        a: AssignedCell<Fp, Fp>,
        b: AssignedCell<Fp, Fp>,
    ) -> Result<AssignedCell<Fp, Fp>, Error> {
        let chip = Pow5Chip::construct(self.pow5_config.clone());
        let hasher = PoseidonHash::<_, _, P128Pow5T3, ConstantLength<2>, WIDTH, RATE>::init(
            chip,
            layouter.namespace(|| "init poseidon"),
        )?;
        hasher.hash(layouter.namespace(|| "hash"), [a, b])
    }

    /// Selecciona (left, right) según el bit de dirección, atando el
    /// resultado a la celda real de `current`.
    fn select_pair(
        &self,
        mut layouter: impl Layouter<Fp>,
        current: &AssignedCell<Fp, Fp>,
        sibling: Value<Fp>,
        bit: Value<Fp>,
    ) -> Result<(AssignedCell<Fp, Fp>, AssignedCell<Fp, Fp>), Error> {
        layouter.assign_region(
            || "select left/right",
            |mut region| {
                self.m_s_select.enable(&mut region, 0)?;

                let bit_cell = region.assign_advice(|| "bit", self.m_bit, 0, || bit)?;
                let cur_copy = region.assign_advice(
                    || "current",
                    self.m_current,
                    0,
                    || current.value().copied(),
                )?;
                region.constrain_equal(current.cell(), cur_copy.cell())?;
                let sib_cell = region.assign_advice(|| "sibling", self.m_sibling, 0, || sibling)?;

                // bit = 0 -> left = current, right = sibling
                // bit = 1 -> left = sibling, right = current
                let left_val = bit
                    .zip(cur_copy.value().copied())
                    .zip(sib_cell.value().copied())
                    .map(|((b, c), s)| if b == Fp::zero() { c } else { s });
                let right_val = bit
                    .zip(cur_copy.value().copied())
                    .zip(sib_cell.value().copied())
                    .map(|((b, c), s)| if b == Fp::zero() { s } else { c });

                let left = region.assign_advice(|| "left", self.m_left, 0, || left_val)?;
                let right = region.assign_advice(|| "right", self.m_right, 0, || right_val)?;
                let _ = bit_cell;
                Ok((left, right))
            },
        )
    }

    /// Range check de 64 bits atado a la celda real del valor.
    fn enforce_range_tied_to(
        &self,
        mut layouter: impl Layouter<Fp>,
        value: &AssignedCell<Fp, Fp>,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "range check",
            |mut region| {
                let val = value.value().copied();

                let mut running = region.assign_advice(
                    || "running_sum inicial",
                    self.rc_running_sum,
                    0,
                    || Value::known(Fp::zero()),
                )?;

                for i in 0..RANGE_BITS {
                    self.rc_s_bool.enable(&mut region, i)?;
                    self.rc_s_accumulate.enable(&mut region, i)?;

                    let bit_val = val.map(|v| {
                        let repr = v.to_repr();
                        let byte = repr.as_ref()[i / 8];
                        if (byte >> (i % 8)) & 1 == 1 {
                            Fp::one()
                        } else {
                            Fp::zero()
                        }
                    });
                    region.assign_advice(|| format!("bit {i}"), self.rc_bit, i, || bit_val)?;
                    region.assign_fixed(
                        || format!("2^{i}"),
                        self.rc_power_of_two,
                        i,
                        || Value::known(Fp::from(2u64).pow_vartime([i as u64])),
                    )?;

                    let acc = running
                        .value()
                        .copied()
                        .zip(bit_val)
                        .map(|(a, b)| a + b * Fp::from(2u64).pow_vartime([i as u64]));
                    running = region.assign_advice(
                        || format!("running {i}"),
                        self.rc_running_sum,
                        i + 1,
                        || acc,
                    )?;
                }

                // El acumulado final debe ser el valor real: esto ata la
                // descomposicion en bits al valor, sin lo cual el range
                // check no comprobaria nada del valor original.
                region.constrain_equal(running.cell(), value.cell())?;
                Ok(())
            },
        )
    }

    /// `out = a - b`, atado a las celdas reales de `a` y `b`.
    fn subtract_tied(
        &self,
        mut layouter: impl Layouter<Fp>,
        a_cell: &AssignedCell<Fp, Fp>,
        b_cell: &AssignedCell<Fp, Fp>,
    ) -> Result<AssignedCell<Fp, Fp>, Error> {
        layouter.assign_region(
            || "out = a - b",
            |mut region| {
                self.s_sub.enable(&mut region, 0)?;
                let a_copy =
                    region.assign_advice(|| "a", self.ar_a, 0, || a_cell.value().copied())?;
                region.constrain_equal(a_cell.cell(), a_copy.cell())?;
                let b_copy =
                    region.assign_advice(|| "b", self.ar_b, 0, || b_cell.value().copied())?;
                region.constrain_equal(b_cell.cell(), b_copy.cell())?;

                let out_val = a_cell
                    .value()
                    .copied()
                    .zip(b_cell.value().copied())
                    .map(|(a, b)| a - b);
                region.assign_advice(|| "out", self.ar_out, 0, || out_val)
            },
        )
    }

    /// `out = a + b`, atado a las celdas reales. Es la puerta que faltaba
    /// respecto al circuito de solvencia: el ABONO del receptor.
    fn add_tied(
        &self,
        mut layouter: impl Layouter<Fp>,
        a_cell: &AssignedCell<Fp, Fp>,
        b_cell: &AssignedCell<Fp, Fp>,
    ) -> Result<AssignedCell<Fp, Fp>, Error> {
        layouter.assign_region(
            || "out = a + b",
            |mut region| {
                self.s_add.enable(&mut region, 0)?;
                let a_copy =
                    region.assign_advice(|| "a", self.ar_a, 0, || a_cell.value().copied())?;
                region.constrain_equal(a_cell.cell(), a_copy.cell())?;
                let b_copy =
                    region.assign_advice(|| "b", self.ar_b, 0, || b_cell.value().copied())?;
                region.constrain_equal(b_cell.cell(), b_copy.cell())?;

                let out_val = a_cell
                    .value()
                    .copied()
                    .zip(b_cell.value().copied())
                    .map(|(a, b)| a + b);
                region.assign_advice(|| "out", self.ar_out, 0, || out_val)
            },
        )
    }

    /// Sube una hoja por un camino de Merkle y devuelve la raíz.
    fn climb(
        &self,
        mut layouter: impl Layouter<Fp>,
        leaf: AssignedCell<Fp, Fp>,
        siblings: &[Value<Fp>],
        bits: &[Value<Fp>],
        tag: &str,
    ) -> Result<AssignedCell<Fp, Fp>, Error> {
        let mut current = leaf;
        for level in 0..TREE_DEPTH {
            let (left, right) = self.select_pair(
                layouter.namespace(|| format!("{tag} select {level}")),
                &current,
                siblings[level],
                bits[level],
            )?;
            current = self.hash_pair(
                layouter.namespace(|| format!("{tag} hash {level}")),
                left,
                right,
            )?;
        }
        Ok(current)
    }
}

/// Testigos de una de las dos partes.
#[derive(Clone, Debug)]
pub struct PartyWitness {
    pub account_id: Value<Fp>,
    pub balance: Value<Fp>,
    pub nonce: Value<Fp>,
    pub siblings: Vec<Value<Fp>>,
    pub path_bits: Vec<Value<Fp>>,
}

impl Default for PartyWitness {
    fn default() -> Self {
        Self {
            account_id: Value::unknown(),
            balance: Value::unknown(),
            nonce: Value::unknown(),
            siblings: vec![Value::unknown(); TREE_DEPTH],
            path_bits: vec![Value::unknown(); TREE_DEPTH],
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct DoubleEntryCircuit {
    pub sender: PartyWitness,
    pub receiver: PartyWitness,
    pub amount: Value<Fp>,
    pub regulatory_limit: Value<Fp>,
}

impl Circuit<Fp> for DoubleEntryCircuit {
    type Config = DoubleEntryConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
        let state: [Column<Advice>; WIDTH] = std::array::from_fn(|_| meta.advice_column());
        let partial_sbox = meta.advice_column();
        let rc_a: [Column<Fixed>; WIDTH] = std::array::from_fn(|_| meta.fixed_column());
        let rc_b: [Column<Fixed>; WIDTH] = std::array::from_fn(|_| meta.fixed_column());

        let witness_column = meta.advice_column();
        let rc_bit = meta.advice_column();
        let rc_running_sum = meta.advice_column();
        let rc_power_of_two = meta.fixed_column();
        let m_bit = meta.advice_column();
        let m_current = meta.advice_column();
        let m_sibling = meta.advice_column();
        let m_left = meta.advice_column();
        let m_right = meta.advice_column();
        let ar_a = meta.advice_column();
        let ar_b = meta.advice_column();
        let ar_out = meta.advice_column();

        let instance = meta.instance_column();

        meta.enable_equality(witness_column);
        meta.enable_equality(rc_running_sum);
        meta.enable_equality(m_current);
        meta.enable_equality(m_left);
        meta.enable_equality(m_right);
        meta.enable_equality(ar_a);
        meta.enable_equality(ar_b);
        meta.enable_equality(ar_out);
        meta.enable_equality(instance);
        meta.enable_constant(rc_power_of_two);
        for c in state.iter() {
            meta.enable_equality(*c);
        }

        let rc_s_bool = meta.selector();
        let rc_s_accumulate = meta.selector();
        let m_s_select = meta.selector();
        let s_sub = meta.selector();
        let s_add = meta.selector();

        meta.create_gate("bit booleano", |meta| {
            let bit = meta.query_advice(rc_bit, Rotation::cur());
            let s = meta.query_selector(rc_s_bool);
            vec![s * bit.clone() * (bit - Expression::Constant(Fp::one()))]
        });

        meta.create_gate("acumulacion de bits", |meta| {
            let bit = meta.query_advice(rc_bit, Rotation::cur());
            let pow = meta.query_fixed(rc_power_of_two);
            let acc_cur = meta.query_advice(rc_running_sum, Rotation::cur());
            let acc_next = meta.query_advice(rc_running_sum, Rotation::next());
            let s = meta.query_selector(rc_s_accumulate);
            vec![s * (acc_next - (acc_cur + bit * pow))]
        });

        meta.create_gate("seleccion left/right", |meta| {
            let bit = meta.query_advice(m_bit, Rotation::cur());
            let cur = meta.query_advice(m_current, Rotation::cur());
            let sib = meta.query_advice(m_sibling, Rotation::cur());
            let left = meta.query_advice(m_left, Rotation::cur());
            let right = meta.query_advice(m_right, Rotation::cur());
            let s = meta.query_selector(m_s_select);
            let one = Expression::Constant(Fp::one());
            vec![
                s.clone() * bit.clone() * (bit.clone() - one.clone()),
                s.clone()
                    * (left - ((one.clone() - bit.clone()) * cur.clone() + bit.clone() * sib.clone())),
                s * (right - ((one - bit.clone()) * sib + bit * cur)),
            ]
        });

        meta.create_gate("sub: out = a - b", |meta| {
            let a = meta.query_advice(ar_a, Rotation::cur());
            let b = meta.query_advice(ar_b, Rotation::cur());
            let out = meta.query_advice(ar_out, Rotation::cur());
            let s = meta.query_selector(s_sub);
            vec![s * (out - (a - b))]
        });

        meta.create_gate("add: out = a + b", |meta| {
            let a = meta.query_advice(ar_a, Rotation::cur());
            let b = meta.query_advice(ar_b, Rotation::cur());
            let out = meta.query_advice(ar_out, Rotation::cur());
            let s = meta.query_selector(s_add);
            vec![s * (out - (a + b))]
        });

        let pow5_config = Pow5Chip::configure::<P128Pow5T3>(meta, state, partial_sbox, rc_a, rc_b);

        DoubleEntryConfig {
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
            m_s_select,
            ar_a,
            ar_b,
            ar_out,
            s_sub,
            s_add,
            instance,
        }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<Fp>,
    ) -> Result<(), Error> {
        // --- Cargar testigos escalares ---
        let (s_id, s_bal, s_nonce, r_id, r_bal, r_nonce, amount, limit, one_cell) = layouter
            .assign_region(
                || "cargar testigos",
                |mut region| {
                    let s_id =
                        region.assign_advice(|| "s_id", config.witness_column, 0, || self.sender.account_id)?;
                    let s_bal =
                        region.assign_advice(|| "s_bal", config.witness_column, 1, || self.sender.balance)?;
                    let s_nonce =
                        region.assign_advice(|| "s_nonce", config.witness_column, 2, || self.sender.nonce)?;
                    let r_id = region.assign_advice(
                        || "r_id",
                        config.witness_column,
                        3,
                        || self.receiver.account_id,
                    )?;
                    let r_bal = region.assign_advice(
                        || "r_bal",
                        config.witness_column,
                        4,
                        || self.receiver.balance,
                    )?;
                    let r_nonce = region.assign_advice(
                        || "r_nonce",
                        config.witness_column,
                        5,
                        || self.receiver.nonce,
                    )?;
                    let amount =
                        region.assign_advice(|| "amount", config.witness_column, 6, || self.amount)?;
                    let limit = region.assign_advice(
                        || "limit",
                        config.witness_column,
                        7,
                        || self.regulatory_limit,
                    )?;
                    let one_cell = region.assign_advice_from_constant(
                        || "uno",
                        config.witness_column,
                        8,
                        Fp::one(),
                    )?;
                    Ok((s_id, s_bal, s_nonce, r_id, r_bal, r_nonce, amount, limit, one_cell))
                },
            )?;

        // El limite regulatorio es publico (fila 2).
        layouter.constrain_instance(limit.cell(), config.instance, 2)?;

        // ===== 1. El emisor esta en el arbol ANTIGUO =====
        let s_inner = config.hash_pair(
            layouter.namespace(|| "s leaf inner"),
            s_id.clone(),
            s_bal.clone(),
        )?;
        let s_leaf_old =
            config.hash_pair(layouter.namespace(|| "s leaf"), s_inner, s_nonce.clone())?;
        let root_old = config.climb(
            layouter.namespace(|| "s climb old"),
            s_leaf_old,
            &self.sender.siblings,
            &self.sender.path_bits,
            "s_old",
        )?;
        layouter.constrain_instance(root_old.cell(), config.instance, 0)?;

        // ===== 2. ADEUDO =====
        let s_bal_new =
            config.subtract_tied(layouter.namespace(|| "adeudo"), &s_bal, &amount)?;
        let s_nonce_new =
            config.add_tied(layouter.namespace(|| "nonce+1"), &s_nonce, &one_cell)?;

        let s_inner_new = config.hash_pair(
            layouter.namespace(|| "s leaf inner nueva"),
            s_id.clone(),
            s_bal_new.clone(),
        )?;
        let s_leaf_new = config.hash_pair(
            layouter.namespace(|| "s leaf nueva"),
            s_inner_new,
            s_nonce_new,
        )?;
        let root_mid = config.climb(
            layouter.namespace(|| "s climb new"),
            s_leaf_new,
            &self.sender.siblings,
            &self.sender.path_bits,
            "s_new",
        )?;

        // ===== 3. El receptor esta en el arbol INTERMEDIO =====
        let r_inner = config.hash_pair(
            layouter.namespace(|| "r leaf inner"),
            r_id.clone(),
            r_bal.clone(),
        )?;
        let r_leaf_old =
            config.hash_pair(layouter.namespace(|| "r leaf"), r_inner, r_nonce.clone())?;
        let root_mid_computed = config.climb(
            layouter.namespace(|| "r climb old"),
            r_leaf_old,
            &self.receiver.siblings,
            &self.receiver.path_bits,
            "r_old",
        )?;
        // Contra root_mid, NO contra root_old.
        layouter.assign_region(
            || "coherencia root_mid",
            |mut region| region.constrain_equal(root_mid.cell(), root_mid_computed.cell()),
        )?;

        // ===== 4. ABONO: el MISMO importe =====
        let r_bal_new = config.add_tied(layouter.namespace(|| "abono"), &r_bal, &amount)?;
        let r_inner_new = config.hash_pair(
            layouter.namespace(|| "r leaf inner nueva"),
            r_id,
            r_bal_new.clone(),
        )?;
        let r_leaf_new = config.hash_pair(
            layouter.namespace(|| "r leaf nueva"),
            r_inner_new,
            r_nonce,
        )?;
        let root_new = config.climb(
            layouter.namespace(|| "r climb new"),
            r_leaf_new,
            &self.receiver.siblings,
            &self.receiver.path_bits,
            "r_new",
        )?;
        layouter.constrain_instance(root_new.cell(), config.instance, 1)?;

        // ===== 5. Nullifier del emisor =====
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
        let null_inner =
            config.hash_pair(layouter.namespace(|| "null inner"), domain_cell, s_id)?;
        let nullifier =
            config.hash_pair(layouter.namespace(|| "nullifier"), null_inner, s_nonce)?;
        layouter.constrain_instance(nullifier.cell(), config.instance, 3)?;

        // ===== 6. Rangos y solvencia =====
        config.enforce_range_tied_to(layouter.namespace(|| "range s_bal"), &s_bal)?;
        config.enforce_range_tied_to(layouter.namespace(|| "range r_bal"), &r_bal)?;
        config.enforce_range_tied_to(layouter.namespace(|| "range amount"), &amount)?;
        config.enforce_range_tied_to(layouter.namespace(|| "range limit"), &limit)?;
        // amount <= s_bal
        config.enforce_range_tied_to(layouter.namespace(|| "range s_bal_new"), &s_bal_new)?;
        // el abono no desborda
        config.enforce_range_tied_to(layouter.namespace(|| "range r_bal_new"), &r_bal_new)?;
        // amount <= limit
        let diff_limit =
            config.subtract_tied(layouter.namespace(|| "diff limit"), &limit, &amount)?;
        config.enforce_range_tied_to(layouter.namespace(|| "range diff limit"), &diff_limit)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use halo2_gadgets::poseidon::primitives::Hash as PoseidonHashPrimitive;
    use halo2_proofs::dev::MockProver;

    /// `k` necesario para este circuito. Es mayor que el del circuito de
    /// solvencia (k=15) porque pasamos de ~24 invocaciones de Poseidon a
    /// ~90: cuatro recorridos del árbol en vez de uno.
    const K: u32 = 17;

    fn native_hash(a: Fp, b: Fp) -> Fp {
        PoseidonHashPrimitive::<Fp, P128Pow5T3, ConstantLength<2>, WIDTH, RATE>::init().hash([a, b])
    }

    fn native_leaf(id: Fp, balance: Fp, nonce: Fp) -> Fp {
        native_hash(native_hash(id, balance), nonce)
    }

    fn native_nullifier(id: Fp, nonce: Fp) -> Fp {
        native_hash(native_hash(Fp::from(NULLIFIER_DOMAIN), id), nonce)
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
        fn path_for(&self, index: usize) -> (Vec<Value<Fp>>, Vec<Value<Fp>>) {
            let mut siblings = Vec::with_capacity(TREE_DEPTH);
            let mut bits = Vec::with_capacity(TREE_DEPTH);
            let mut idx = index;
            for level in 0..TREE_DEPTH {
                siblings.push(Value::known(self.levels[level][idx ^ 1]));
                bits.push(Value::known(if idx % 2 == 1 { Fp::one() } else { Fp::zero() }));
                idx /= 2;
            }
            (siblings, bits)
        }
    }

    const SENDER_IDX: usize = 3;
    const RECEIVER_IDX: usize = 5;

    struct Scenario {
        circuit: DoubleEntryCircuit,
        public_inputs: Vec<Fp>,
    }

    /// ⚠️ EL PUNTO DELICADO: el camino del receptor se toma del árbol
    /// INTERMEDIO (tras actualizar al emisor), no del original. Emisor y
    /// receptor comparten ancestros en los niveles altos, así que
    /// actualizar al emisor cambia hermanos del camino del receptor.
    ///
    /// `credited` permite acreditar una cantidad distinta a la debitada,
    /// para construir los tests que rompen la conservación.
    fn build_scenario(
        sender_balance: u64,
        receiver_balance: u64,
        amount: u64,
        credited: u64,
        limit: u64,
    ) -> Scenario {
        let s_id = Fp::from(1001u64);
        let s_nonce = Fp::from(7u64);
        let r_id = Fp::from(2002u64);
        let r_nonce = Fp::from(3u64);

        let mut leaves: Vec<Fp> = (0..8u64).map(Fp::from).collect();
        leaves[SENDER_IDX] = native_leaf(s_id, Fp::from(sender_balance), s_nonce);
        leaves[RECEIVER_IDX] = native_leaf(r_id, Fp::from(receiver_balance), r_nonce);

        let tree_old = NativeTree::build(leaves.clone());
        let root_old = tree_old.root();
        let (s_siblings, s_bits) = tree_old.path_for(SENDER_IDX);

        let mut leaves_mid = leaves.clone();
        leaves_mid[SENDER_IDX] = native_leaf(
            s_id,
            Fp::from(sender_balance) - Fp::from(amount),
            s_nonce + Fp::one(),
        );
        let tree_mid = NativeTree::build(leaves_mid.clone());
        let (r_siblings, r_bits) = tree_mid.path_for(RECEIVER_IDX);

        let mut leaves_new = leaves_mid;
        leaves_new[RECEIVER_IDX] =
            native_leaf(r_id, Fp::from(receiver_balance + credited), r_nonce);
        let root_new = NativeTree::build(leaves_new).root();

        let circuit = DoubleEntryCircuit {
            sender: PartyWitness {
                account_id: Value::known(s_id),
                balance: Value::known(Fp::from(sender_balance)),
                nonce: Value::known(s_nonce),
                siblings: s_siblings,
                path_bits: s_bits,
            },
            receiver: PartyWitness {
                account_id: Value::known(r_id),
                balance: Value::known(Fp::from(receiver_balance)),
                nonce: Value::known(r_nonce),
                siblings: r_siblings,
                path_bits: r_bits,
            },
            amount: Value::known(Fp::from(amount)),
            regulatory_limit: Value::known(Fp::from(limit)),
        };

        Scenario {
            circuit,
            public_inputs: vec![
                root_old,
                root_new,
                Fp::from(limit),
                native_nullifier(s_id, s_nonce),
            ],
        }
    }

    fn is_satisfied(s: Scenario) -> bool {
        match MockProver::run(K, &s.circuit, vec![s.public_inputs]) {
            Ok(prover) => prover.verify().is_ok(),
            Err(e) => {
                println!("MockProver::run fallo: {e:?}");
                false
            }
        }
    }

    #[test]
    fn valid_transfer_satisfies_circuit() {
        let s = build_scenario(1_000_000, 50_000, 250_000, 250_000, 500_000);
        let prover = MockProver::run(K, &s.circuit, vec![s.public_inputs])
            .expect("MockProver deberia arrancar; si falla por filas, subir K");
        assert_eq!(
            prover.verify(),
            Ok(()),
            "una transferencia valida deberia satisfacer el circuito"
        );
    }

    /// EL TEST QUE DA SENTIDO A LA PIEZA: el receptor recibe 10.000 más de
    /// lo que el emisor perdió. Creación de dinero de la nada.
    #[test]
    fn money_creation_is_rejected() {
        let s = build_scenario(1_000_000, 50_000, 250_000, 260_000, 500_000);
        assert!(
            !is_satisfied(s),
            "CRITICO: acreditar mas de lo debitado debe rechazarse"
        );
    }

    /// El caso simétrico: destrucción de dinero.
    #[test]
    fn money_destruction_is_rejected() {
        let s = build_scenario(1_000_000, 50_000, 250_000, 240_000, 500_000);
        assert!(
            !is_satisfied(s),
            "CRITICO: acreditar menos de lo debitado debe rechazarse"
        );
    }

    #[test]
    fn over_regulatory_limit_is_rejected() {
        let s = build_scenario(1_000_000, 50_000, 750_000, 750_000, 500_000);
        assert!(
            !is_satisfied(s),
            "CRITICO: superar el limite regulatorio debe rechazarse"
        );
    }

    #[test]
    fn wrong_declared_new_root_is_rejected() {
        let mut s = build_scenario(1_000_000, 50_000, 250_000, 250_000, 500_000);
        s.public_inputs[1] = Fp::from(999_999u64);
        assert!(
            !is_satisfied(s),
            "CRITICO: una raiz final incorrecta debe rechazarse"
        );
    }

    #[test]
    fn forged_nullifier_is_rejected() {
        let mut s = build_scenario(1_000_000, 50_000, 250_000, 250_000, 500_000);
        s.public_inputs[3] = Fp::from(31_337u64);
        assert!(
            !is_satisfied(s),
            "CRITICO: un nullifier falsificado debe rechazarse"
        );
    }
}
