//! Circuito de cumplimiento ("Blind Compliance") real, en R1CS.
//!
//! A diferencia de las versiones anteriores del proyecto (que solo hasheaban
//! valores con SHA-256 sin imponer ninguna restricción algebraica), este
//! circuito impone matemáticamente, dentro del sistema de restricciones
//! rank-1 (R1CS), dos condiciones sobre valores privados:
//!
//!   1. amount <= balance           (el emisor tiene fondos suficientes)
//!   2. amount <= regulatory_limit  (la transferencia no excede el límite AML)
//!
//! sin revelar `balance` ni `amount` al verificador. Solo `regulatory_limit`
//! es una entrada pública (el regulador ya conoce el límite normativo).
//!
//! ## Por qué range checks explícitos
//!
//! Un cuerpo finito (field) no tiene un orden natural: F_p "envuelve"
//! (wraps around) al llegar al módulo p. Comparar `a <= b` mediante una resta
//! `b - a` e interpretarla como "no negativa" solo es válido si podemos
//! garantizar que el resultado, de ser negativo en los enteros, se traduce
//! en un valor de campo *enorme* (cercano a p) y no en un valor pequeño por
//! casualidad. Para BLS12-381 (p ~ 2^255) esto se cumple siempre que
//! acotemos los operandos a un rango razonable (aquí, 64 bits), evitando
//! así el "ataque de desbordamiento de módulo" clásico en circuitos ZK mal
//! diseñados.
//!
//! Por eso el circuito acota explícitamente `balance`, `amount` y
//! `regulatory_limit` a 64 bits, y solo entonces resta y vuelve a acotar
//! las diferencias.

use ark_ff::PrimeField;
use ark_r1cs_std::fields::fp::FpVar;
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};

/// Número de bits al que se acota cada valor. u64 es suficiente para
/// representar cualquier importe monetario en la unidad mínima (céntimos)
/// hasta ~184 billones de unidades — más que suficiente para liquidación
/// institucional, y muy por debajo del módulo del cuerpo escalar de
/// BLS12-381 (~2^255), que es lo que garantiza que la técnica de
/// range-check por resta sea segura.
pub const VALUE_BITS: usize = 64;

/// Circuito de cumplimiento de liquidación institucional.
///
/// Testigos privados (nunca se revelan al verificador): `balance`, `amount`.
/// Entrada pública (el verificador la conoce): `regulatory_limit`.
#[derive(Clone)]
pub struct ComplianceCircuit<F: PrimeField> {
    /// Saldo del emisor. `None` se usa únicamente durante la generación de
    /// claves (setup), donde no existe un testigo real todavía.
    pub balance: Option<u64>,
    /// Monto de la transferencia.
    pub amount: Option<u64>,
    /// Límite regulatorio (AML / macroprudencial). Es información pública:
    /// el regulador define el límite, no es un secreto.
    pub regulatory_limit: F,
}

impl<F: PrimeField> ComplianceCircuit<F> {
    /// Construye una instancia completa (con testigos) para generar una prueba.
    pub fn new(balance: u64, amount: u64, regulatory_limit: u64) -> Self {
        Self {
            balance: Some(balance),
            amount: Some(amount),
            regulatory_limit: F::from(regulatory_limit),
        }
    }

    /// Construye una instancia "vacía" (sin testigos) para la generación de
    /// claves. La estructura de restricciones debe ser idéntica a la de una
    /// instancia real; por eso `regulatory_limit` debe fijarse a un valor
    /// representativo (no afecta al número de restricciones, pero conviene
    /// no dejarlo en cero para evitar confusiones al depurar).
    pub fn empty_for_setup(regulatory_limit: u64) -> Self {
        Self {
            balance: None,
            amount: None,
            regulatory_limit: F::from(regulatory_limit),
        }
    }
}

/// Fuerza que `value` sea representable en `VALUE_BITS` bits, es decir,
/// que `0 <= value < 2^VALUE_BITS`. Esto añade restricciones que enlazan
/// la descomposición en bits con el valor de campo original (la conversión
/// `to_bits_le` ya impone esa consistencia internamente).
pub(crate) fn enforce_range<F: PrimeField>(value: &FpVar<F>) -> Result<(), SynthesisError> {
    let bits = value.to_bits_le()?;
    for bit in bits.iter().skip(VALUE_BITS) {
        bit.enforce_equal(&Boolean::FALSE)?;
    }
    Ok(())
}

impl<F: PrimeField> ConstraintSynthesizer<F> for ComplianceCircuit<F> {
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        // --- 1. Asignación de testigos privados ---
        let balance_var = FpVar::<F>::new_witness(cs.clone(), || {
            self.balance
                .map(F::from)
                .ok_or(SynthesisError::AssignmentMissing)
        })?;

        let amount_var = FpVar::<F>::new_witness(cs.clone(), || {
            self.amount
                .map(F::from)
                .ok_or(SynthesisError::AssignmentMissing)
        })?;

        // --- 2. Asignación de la entrada pública ---
        let limit_var = FpVar::<F>::new_input(cs.clone(), || Ok(self.regulatory_limit))?;

        // --- 3. Acotar cada valor a VALUE_BITS bits ---
        // Esto es lo que faltaba en todas las versiones anteriores del
        // proyecto: sin esto, un probador malicioso podría elegir
        // `balance`/`amount` como elementos de campo arbitrarios (incluyendo
        // valores "negativos" en el sentido modular) y falsificar la prueba.
        enforce_range(&balance_var)?;
        enforce_range(&amount_var)?;
        enforce_range(&limit_var)?;

        // --- 4. amount <= balance ---
        // diff_balance = balance - amount
        // Si amount > balance en los enteros, diff_balance (como elemento
        // de campo) es p - (amount - balance): un número cercano a p,
        // muchísimo mayor que 2^VALUE_BITS. enforce_range lo rechaza.
        let diff_balance = &balance_var - &amount_var;
        enforce_range(&diff_balance)?;

        // --- 5. amount <= regulatory_limit ---
        let diff_limit = &limit_var - &amount_var;
        enforce_range(&diff_limit)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bls12_381::Fr;
    use ark_relations::r1cs::ConstraintSystem;

    /// Comprueba que una transacción válida (fondos suficientes, dentro del
    /// límite) satisface todas las restricciones del circuito.
    #[test]
    fn valid_transaction_satisfies_constraints() {
        let cs = ConstraintSystem::<Fr>::new_ref();
        let circuit = ComplianceCircuit::new(
            1_000_000, // balance
            250_000,   // amount
            500_000,   // regulatory_limit
        );
        circuit.generate_constraints(cs.clone()).unwrap();
        assert!(cs.is_satisfied().unwrap(), "el circuito debería ser satisfacible con una transacción válida");
    }

    /// Comprueba que un intento de gastar más de lo que se tiene NO
    /// satisface el circuito. Esta es la prueba negativa que faltaba en
    /// todas las versiones anteriores de `zk-core`: aquí sí falla de verdad.
    #[test]
    fn insufficient_balance_fails_constraints() {
        let cs = ConstraintSystem::<Fr>::new_ref();
        let circuit = ComplianceCircuit::new(
            100_000, // balance
            250_000, // amount > balance
            500_000, // regulatory_limit
        );
        circuit.generate_constraints(cs.clone()).unwrap();
        assert!(
            !cs.is_satisfied().unwrap(),
            "el circuito NO debería ser satisfacible cuando amount > balance"
        );
    }

    /// Comprueba que exceder el límite regulatorio NO satisface el circuito,
    /// incluso si hay saldo de sobra.
    #[test]
    fn exceeding_regulatory_limit_fails_constraints() {
        let cs = ConstraintSystem::<Fr>::new_ref();
        let circuit = ComplianceCircuit::new(
            10_000_000, // balance de sobra
            600_000,    // amount > regulatory_limit
            500_000,    // regulatory_limit
        );
        circuit.generate_constraints(cs.clone()).unwrap();
        assert!(
            !cs.is_satisfied().unwrap(),
            "el circuito NO debería ser satisfacible cuando amount > regulatory_limit"
        );
    }

    /// Caso límite: amount == balance y amount == limit deben ser válidos
    /// (la condición es <=, no <).
    #[test]
    fn boundary_equal_values_satisfy_constraints() {
        let cs = ConstraintSystem::<Fr>::new_ref();
        let circuit = ComplianceCircuit::new(500_000, 500_000, 500_000);
        circuit.generate_constraints(cs.clone()).unwrap();
        assert!(cs.is_satisfied().unwrap());
    }
}
