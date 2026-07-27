//! Implementación de `SettlementProver` para el backend STARK/AIR.
//! Envuelve el circuito ya verificado en `compliance_circuit` — no añade
//! ninguna lógica criptográfica nueva.
//!
//! ## Lo que este backend revela sobre el trait
//!
//! Aquí **no hay claves**. `setup()` no genera ningún material secreto:
//! devuelve la CONFIGURACIÓN de la prueba (`ProofOptions`), que es
//! pública, publicable y verificable por cualquiera. Los tipos asociados
//! `ProvingKey` y `VerifyingKey` son ambos `ProofOptions`, y son
//! idénticos entre sí.
//!
//! Esa es la asimetría real que el trait deja ver sin esconderla:
//! - Groth16: `setup` produce claves de una ceremonia; si el "residuo
//!   tóxico" no se destruye, se pueden falsificar pruebas.
//! - Halo2/IPA: `setup` produce parámetros deterministas, sin secreto,
//!   pero caros de generar (~176 s medidos) y necesarios en cada llamada.
//! - STARK: `setup` es instantáneo y su salida es una elección de
//!   parámetros, no un artefacto criptográfico.
//!
//! Un trait que ocultara esta diferencia bajo una interfaz uniforme
//! estaría mintiendo sobre la propiedad más importante que distingue a
//! estos sistemas.
//!
//! ## Configuración por defecto
//!
//! `setup()` ignora `rng_seed` (no hay aleatoriedad que sembrar) y
//! devuelve la configuración de **128 bits DEMOSTRABLES**, la misma que
//! usa `iso_bridge`. Cuesta ~125 KB y ~45 ms; se elige la garantía fuerte
//! por defecto. Para otras configuraciones, usar directamente
//! `ComplianceProver` en vez del trait.
//!
//! ## Nota sobre testigos inválidos
//!
//! Con un testigo que viola las restricciones (por ejemplo, gastar más
//! del saldo), en compilaciones de DEBUG winterfell hace panic desde una
//! assertion interna en vez de devolver `Err`. En release devuelve una
//! prueba que después no verifica. Ambas son detecciones correctas, pero
//! la primera no pasa por `Result` — está documentado aquí porque un
//! llamador podría esperar lo contrario.

use settlement_prover::SettlementProver;
use winterfell::crypto::{hashers::Blake3_256, DefaultRandomCoin, MerkleTree};
use winterfell::math::fields::f64::BaseElement;
use winterfell::{verify, AcceptableOptions, Proof, ProofOptions, Prover};

use crate::compliance_circuit::{
    build_trace, native_leaf, native_nullifier, native_root, ComplianceAir, ComplianceProver,
    CompliancePublicInputs,
};
use crate::iso_bridge::default_proof_options;
use crate::merkle::MerklePath;

type Blake3 = Blake3_256<BaseElement>;

/// Marcador (sin estado propio) para seleccionar el backend STARK a
/// través del trait `SettlementProver`.
pub struct StarkBackend;

/// Testigo completo del circuito de cumplimiento. Todos estos valores son
/// PRIVADOS: no aparecen en los inputs públicos.
#[derive(Clone, Debug)]
pub struct StarkWitness {
    pub account_id: BaseElement,
    pub balance_minor_units: u64,
    pub nonce: BaseElement,
    pub amount_minor_units: u64,
    pub regulatory_limit_minor_units: u64,
    pub path: MerklePath,
}

#[derive(Debug)]
pub struct StarkError(String);

impl std::fmt::Display for StarkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error del backend STARK: {}", self.0)
    }
}
impl std::error::Error for StarkError {}

impl SettlementProver for StarkBackend {
    type Witness = StarkWitness;
    type PublicInput = CompliancePublicInputs;
    /// La prueba serializada, lista para viajar por la red.
    type Proof = Vec<u8>;
    /// No es una clave: es configuración pública. Ver la nota de cabecera.
    type ProvingKey = ProofOptions;
    /// Idéntica a la de prueba, y publicable sin riesgo.
    type VerifyingKey = ProofOptions;
    type Error = StarkError;

    /// Instantáneo y sin secretos. `rng_seed` se ignora: no hay
    /// aleatoriedad que sembrar en un sistema transparente.
    fn setup(_rng_seed: u64) -> Result<(Self::ProvingKey, Self::VerifyingKey), Self::Error> {
        let options = default_proof_options();
        Ok((options.clone(), options))
    }

    /// `rng_seed` también se ignora: winterfell obtiene su aleatoriedad
    /// del propio transcript (Fiat-Shamir), no de una semilla externa.
    fn prove(
        pk: &Self::ProvingKey,
        witness: Self::Witness,
        _rng_seed: u64,
    ) -> Result<(Self::Proof, Self::PublicInput), Self::Error> {
        // Los inputs públicos se derivan del testigo con las funciones
        // nativas ya verificadas — no se reciben del llamador, para que
        // no puedan declararse valores que no correspondan al testigo.
        let leaf = native_leaf(
            witness.account_id,
            BaseElement::new(witness.balance_minor_units),
            witness.nonce,
        );
        let public_input = CompliancePublicInputs {
            state_root: native_root(leaf, &witness.path),
            regulatory_limit: BaseElement::new(witness.regulatory_limit_minor_units),
            nullifier: native_nullifier(witness.account_id, witness.nonce),
        };

        let trace = build_trace(
            witness.account_id,
            witness.balance_minor_units,
            witness.nonce,
            witness.amount_minor_units,
            witness.regulatory_limit_minor_units,
            &witness.path,
        );

        let prover = ComplianceProver::new(pk.clone());
        let proof = prover
            .prove(trace)
            .map_err(|e| StarkError(format!("fallo al generar la prueba: {e:?}")))?;

        Ok((proof.to_bytes(), public_input))
    }

    fn verify(
        vk: &Self::VerifyingKey,
        public_input: &Self::PublicInput,
        proof: &Self::Proof,
    ) -> Result<bool, Self::Error> {
        let parsed = Proof::from_bytes(proof)
            .map_err(|e| StarkError(format!("prueba mal formada: {e:?}")))?;

        let acceptable = AcceptableOptions::OptionSet(vec![vk.clone()]);
        match verify::<ComplianceAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            parsed,
            public_input.clone(),
            &acceptable,
        ) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compliance_circuit::TREE_DEPTH;
    use crate::merkle::Digest;

    fn digest_from(n: u64) -> Digest {
        [
            BaseElement::new(n),
            BaseElement::new(n + 1),
            BaseElement::new(n + 2),
            BaseElement::new(n + 3),
        ]
    }

    fn sample_witness(balance: u64, amount: u64, limit: u64) -> StarkWitness {
        StarkWitness {
            account_id: BaseElement::new(12345),
            balance_minor_units: balance,
            nonce: BaseElement::new(1),
            amount_minor_units: amount,
            regulatory_limit_minor_units: limit,
            path: MerklePath {
                siblings: (0..TREE_DEPTH).map(|i| digest_from(i as u64 * 10)).collect(),
                is_right: (0..TREE_DEPTH).map(|i| i % 3 == 0).collect(),
            },
        }
    }

    /// EL TEST CLAVE: el mismo flujo setup/prove/verify que ya
    /// verificamos directamente, ahora pasando exclusivamente por la
    /// interfaz genérica `SettlementProver` — confirma que la abstracción
    /// no cambia el comportamiento real.
    #[test]
    fn stark_backend_valid_transaction_via_trait() {
        let (pk, vk) = StarkBackend::setup(1).expect("setup no deberia fallar");

        let (proof, public_input) =
            StarkBackend::prove(&pk, sample_witness(1_000_000, 250_000, 500_000), 2)
                .expect("prove no deberia fallar");

        let is_valid = StarkBackend::verify(&vk, &public_input, &proof)
            .expect("verify no deberia devolver error");

        assert!(
            is_valid,
            "una transaccion valida debe verificar como verdadera via el trait"
        );
    }

    /// El setup de este backend no produce secretos: sus dos salidas son
    /// la misma configuración pública. Este test documenta esa propiedad
    /// en código ejecutable, no solo en un comentario.
    #[test]
    fn setup_produces_no_secret_material() {
        let (pk, vk) = StarkBackend::setup(1).unwrap();
        assert_eq!(
            pk.num_queries(),
            vk.num_queries(),
            "en STARK, clave de prueba y de verificacion son la misma configuracion publica"
        );
        assert_eq!(pk.blowup_factor(), vk.blowup_factor());

        // Y es determinista: dos setups distintos dan lo mismo, porque no
        // hay aleatoriedad involucrada (a diferencia de Groth16).
        let (pk2, _) = StarkBackend::setup(999).unwrap();
        assert_eq!(pk.num_queries(), pk2.num_queries());
        assert_eq!(pk.blowup_factor(), pk2.blowup_factor());
    }

    /// SOLIDEZ vía el trait: un testigo insolvente no debe producir una
    /// prueba verificable. Tres formas legítimas de detección, igual que
    /// en el resto del crate.
    #[test]
    fn insolvent_witness_does_not_verify_via_trait() {
        let (pk, vk) = StarkBackend::setup(1).unwrap();

        let previous_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let attempt = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            StarkBackend::prove(&pk, sample_witness(100_000, 250_000, 500_000), 2)
        }));
        std::panic::set_hook(previous_hook);

        match attempt {
            Err(_) => { /* panic: detectado en debug */ }
            Ok(Err(_)) => { /* Err: detectado */ }
            Ok(Ok((proof, public_input))) => {
                let is_valid = StarkBackend::verify(&vk, &public_input, &proof).unwrap();
                assert!(
                    !is_valid,
                    "CRITICO: un testigo insolvente no deberia verificar via el trait"
                );
            }
        }
    }

    /// Manipular los inputs públicos declarados debe hacer fallar la
    /// verificación, también a través del trait.
    #[test]
    fn tampered_public_input_fails_via_trait() {
        let (pk, vk) = StarkBackend::setup(1).unwrap();
        let (proof, mut public_input) =
            StarkBackend::prove(&pk, sample_witness(1_000_000, 250_000, 500_000), 2).unwrap();

        public_input.state_root = digest_from(999_999);

        let is_valid = StarkBackend::verify(&vk, &public_input, &proof).unwrap();
        assert!(
            !is_valid,
            "CRITICO: unos inputs publicos manipulados no deberian verificar"
        );
    }
}
