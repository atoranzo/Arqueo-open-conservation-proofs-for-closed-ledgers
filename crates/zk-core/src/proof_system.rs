//! Capa de generación y verificación de pruebas SNARK reales (Groth16)
//! sobre `ComplianceCircuit`.
//!
//! Esto es lo que en todas las versiones anteriores del proyecto faltaba
//! por completo: aquí sí se genera una `ProvingKey`/`VerifyingKey` mediante
//! un "trusted setup" específico del circuito (Groth16 requiere esto; es
//! una limitación conocida del esquema, documentada más abajo), y se
//! genera/verifica una prueba criptográfica real, no un hash.

use ark_bls12_381::{Bls12_381, Fr};
use ark_groth16::{Groth16, PreparedVerifyingKey, Proof, ProvingKey, VerifyingKey};
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystem};
use ark_snark::SNARK;
use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::SeedableRng;

use crate::circuit::ComplianceCircuit;

pub type ComplianceProof = Proof<Bls12_381>;
pub type ComplianceProvingKey = ProvingKey<Bls12_381>;
pub type ComplianceVerifyingKey = VerifyingKey<Bls12_381>;

/// Errores propios de esta capa, para no filtrar el tipo de error interno
/// de ark-relations/ark-groth16 a quien use esta librería.
#[derive(Debug)]
pub enum ZkCoreError {
    SetupFailed(String),
    ProvingFailed(String),
    VerificationFailed(String),
}

impl std::fmt::Display for ZkCoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZkCoreError::SetupFailed(e) => write!(f, "fallo en el setup del circuito: {e}"),
            ZkCoreError::ProvingFailed(e) => write!(f, "fallo al generar la prueba: {e}"),
            ZkCoreError::VerificationFailed(e) => write!(f, "fallo al verificar la prueba: {e}"),
        }
    }
}
impl std::error::Error for ZkCoreError {}

/// Genera las claves de prueba/verificación para el circuito de
/// cumplimiento SIN vinculación de estado. Esto se ejecuta UNA VEZ por
/// circuito (no por transacción) y las claves resultantes se distribuyen:
/// `ProvingKey` a los emisores autorizados a generar pruebas,
/// `VerifyingKey` a los validadores.
///
/// ## Nota importante sobre el trusted setup
///
/// Groth16 requiere un "trusted setup" cuyo parámetro aleatorio (el
/// "toxic waste") debe destruirse tras la generación. Si se conserva o se
/// filtra, quien lo tenga puede falsificar pruebas sin que nadie lo note.
/// Para un sistema de producción real esto normalmente se resuelve con una
/// ceremonia MPC (multi-party computation) donde ningún participante
/// individual conoce el secreto completo — no está implementado aquí, y es
/// un requisito de seguridad real que no se puede omitir antes de
/// cualquier uso en producción. Alternativas sin trusted setup por
/// circuito (PLONK, Halo2, STARKs) evitan este problema a cambio de
/// pruebas algo más grandes o más lentas de generar.
pub fn setup(
    rng_seed: u64,
) -> Result<(ComplianceProvingKey, ComplianceVerifyingKey), ZkCoreError> {
    // El límite regulatorio usado aquí es arbitrario: la estructura de
    // restricciones (y por tanto las claves) no depende de su valor
    // concreto, solo de que el circuito tenga testigos `None`.
    let circuit = ComplianceCircuit::<Fr>::empty_for_setup(0);
    setup_generic(circuit, rng_seed)
}

/// Genera una prueba de que `amount <= balance` y `amount <= regulatory_limit`,
/// sin revelar `balance` ni `amount`. Sin vinculación de estado (usa
/// `ComplianceCircuit`, no `ComplianceCircuitWithState`).
pub fn prove(
    pk: &ComplianceProvingKey,
    balance: u64,
    amount: u64,
    regulatory_limit: u64,
    rng_seed: u64,
) -> Result<ComplianceProof, ZkCoreError> {
    let circuit = ComplianceCircuit::<Fr>::new(balance, amount, regulatory_limit);
    prove_generic(pk, circuit, rng_seed)
}

/// Verifica una prueba contra el límite regulatorio público. Si esto
/// devuelve `Ok(true)`, matemáticamente se garantiza que quien generó la
/// prueba conocía un `balance` y un `amount` tales que
/// `amount <= balance` y `amount <= regulatory_limit`, sin que el
/// verificador aprenda esos valores.
pub fn verify(
    vk: &ComplianceVerifyingKey,
    proof: &ComplianceProof,
    regulatory_limit: u64,
) -> Result<bool, ZkCoreError> {
    verify_generic(vk, proof, &[Fr::from(regulatory_limit)])
}

/// Versión genérica de `setup`, válida para CUALQUIER circuito de Arkworks
/// sobre el cuerpo escalar de BLS12-381 (`Fr`). Se introdujo al integrar
/// `ComplianceCircuitWithState`, para no duplicar esta lógica una segunda
/// vez: los wrappers específicos de cada circuito (`setup`/`prove`/`verify`
/// arriba, y `setup_with_state`/`prove_with_state`/`verify_with_state` en
/// `circuit_with_state.rs`) llaman a estas funciones por dentro.
pub fn setup_generic<C: ConstraintSynthesizer<Fr> + Clone>(
    empty_circuit: C,
    rng_seed: u64,
) -> Result<(ComplianceProvingKey, ComplianceVerifyingKey), ZkCoreError> {
    let mut rng = ChaCha20Rng::seed_from_u64(rng_seed);
    let (pk, vk) = Groth16::<Bls12_381>::circuit_specific_setup(empty_circuit, &mut rng)
        .map_err(|e| ZkCoreError::SetupFailed(format!("{e:?}")))?;
    Ok((pk, vk))
}

/// Versión genérica de `prove`, con el mismo pre-check de satisfacibilidad
/// (ver hallazgo documentado más abajo) aplicado a cualquier circuito.
pub fn prove_generic<C: ConstraintSynthesizer<Fr> + Clone>(
    pk: &ComplianceProvingKey,
    circuit: C,
    rng_seed: u64,
) -> Result<ComplianceProof, ZkCoreError> {
    let mut rng = ChaCha20Rng::seed_from_u64(rng_seed);

    // Pre-check de satisfacibilidad. Ver la nota completa sobre por qué es
    // necesario (ark-groth16 0.4 hace panic, no devuelve Err, con un
    // testigo inválido; confirmado por ejecución real de este proyecto).
    let check_cs = ConstraintSystem::<Fr>::new_ref();
    circuit
        .clone()
        .generate_constraints(check_cs.clone())
        .map_err(|e| ZkCoreError::ProvingFailed(format!("{e:?}")))?;
    let satisfied = check_cs
        .is_satisfied()
        .map_err(|e| ZkCoreError::ProvingFailed(format!("{e:?}")))?;
    if !satisfied {
        return Err(ZkCoreError::ProvingFailed(
            "el testigo no satisface las restricciones del circuito".to_string(),
        ));
    }

    Groth16::<Bls12_381>::prove(pk, circuit, &mut rng)
        .map_err(|e| ZkCoreError::ProvingFailed(format!("{e:?}")))
}

/// Versión genérica de `verify`: recibe la lista completa de inputs
/// públicos, EN EL MISMO ORDEN en que el circuito los asignó con
/// `new_input(...)` dentro de `generate_constraints`. Si el orden no
/// coincide, la verificación fallará (devolverá `Ok(false)`) aunque los
/// valores en sí sean "correctos" — el orden importa tanto como el valor.
pub fn verify_generic(
    vk: &ComplianceVerifyingKey,
    proof: &ComplianceProof,
    public_inputs: &[Fr],
) -> Result<bool, ZkCoreError> {
    let pvk: PreparedVerifyingKey<Bls12_381> = Groth16::<Bls12_381>::process_vk(vk)
        .map_err(|e| ZkCoreError::VerificationFailed(format!("{e:?}")))?;

    Groth16::<Bls12_381>::verify_with_processed_vk(&pvk, public_inputs, proof)
        .map_err(|e| ZkCoreError::VerificationFailed(format!("{e:?}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Prueba de extremo a extremo: setup -> prove -> verify, con una
    /// transacción válida. Esta es la prueba que demuestra que el sistema
    /// completo funciona, no solo que las restricciones del circuito se
    /// satisfacen en memoria.
    #[test]
    fn end_to_end_valid_transaction() {
        let (pk, vk) = setup(42).expect("setup no debería fallar");

        let proof = prove(&pk, 1_000_000, 250_000, 500_000, 7)
            .expect("generar la prueba no debería fallar con una transacción válida");

        let is_valid = verify(&vk, &proof, 500_000).expect("la verificación no debería devolver error");
        assert!(is_valid, "la prueba de una transacción válida debe verificar como verdadera");
    }

    /// Prueba de extremo a extremo con un límite regulatorio incorrecto en
    /// la verificación: debe fallar, porque el input público no coincide
    /// con el que se usó al generar la prueba.
    #[test]
    fn verification_fails_with_wrong_public_input() {
        let (pk, vk) = setup(42).expect("setup no debería fallar");

        let proof = prove(&pk, 1_000_000, 250_000, 500_000, 7)
            .expect("generar la prueba no debería fallar");

        // Verificamos contra un límite regulatorio distinto al usado en la prueba.
        let is_valid = verify(&vk, &proof, 999_999).expect("la verificación no debería devolver error");
        assert!(
            !is_valid,
            "la prueba no debe verificar como válida si el input público no coincide"
        );
    }

    /// Prueba de extremo a extremo con un testigo que NO satisface las
    /// restricciones (amount > balance).
    ///
    /// CONFIRMADO POR EJECUCIÓN REAL: sin el pre-check añadido en `prove()`,
    /// este caso hacía panic dentro de ark-groth16
    /// (`prover.rs:197: assertion failed: cs.is_satisfied().unwrap()`),
    /// no devolvía un `Err`. Con el pre-check, debe devolver `Err` de forma
    /// limpia, sin crashear el proceso.
    #[test]
    fn end_to_end_insufficient_balance_is_rejected_without_panicking() {
        let (pk, _vk) = setup(42).expect("setup no deberia fallar");

        let result = prove(&pk, 100_000, 250_000, 500_000, 7);
        assert!(
            result.is_err(),
            "un testigo con amount > balance debe ser rechazado con Err, \
             sin llegar a invocar (y hacer panicar) al backend Groth16."
        );
    }

    /// Mismo caso que arriba, pero excediendo el límite regulatorio en vez
    /// del saldo, para confirmar que el pre-check cubre ambas restricciones,
    /// no solo la de balance.
    #[test]
    fn end_to_end_exceeding_limit_is_rejected_without_panicking() {
        let (pk, _vk) = setup(42).expect("setup no deberia fallar");

        let result = prove(&pk, 10_000_000, 600_000, 500_000, 7);
        assert!(
            result.is_err(),
            "un testigo con amount > regulatory_limit debe ser rechazado con Err."
        );
    }
}
