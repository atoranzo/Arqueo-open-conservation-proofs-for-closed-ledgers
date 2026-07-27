//! Trait mínimo de abstracción sobre backends de prueba de cumplimiento.
//!
//! ## Alcance deliberado — leer antes de extender
//!
//! Este trait abstrae ÚNICAMENTE la forma de la llamada (setup/prove/verify
//! con claves tipadas), siguiendo el mismo patrón que `ark_snark::SNARK`
//! (el trait que ya usa `ark-groth16` internamente) — no un formato de
//! datos común entre backends. `Witness`, `PublicInput`, `Proof`,
//! `ProvingKey`, `VerifyingKey` y `Error` son tipos ASOCIADOS, no
//! concretos: cada backend define los suyos.
//!
//! **Por qué no hay un `SettlementWitness` compartido**: se intentó
//! diseñar uno (con `account_id`/`balance`/etc. en tipos primitivos como
//! `u64`) y se descubrió un límite real al hacerlo: el árbol de Merkle y
//! el nullifier dependen del CUERPO FINITO del backend (BLS12-381 `Fr`
//! para Groth16, Pallas `Fp` para Halo2 — cuerpos matemáticamente
//! distintos). Un árbol construido con Poseidon-sobre-Fr no es el mismo
//! árbol que uno construido con Poseidon-sobre-Fp para "las mismas"
//! cuentas — cada backend necesita su propio árbol y su propio espacio
//! de nullifiers. Esto contradice la promesa de "conmutar el motor sin
//! tocar una línea" que a veces se hace sobre este tipo de arquitecturas;
//! aquí se prefiere ser honestos sobre esa limitación real, no ocultarla
//! en una abstracción que finja no tenerla.
//!
//! Lo que SÍ gana este trait: código de orquestación (medir tiempos,
//! reintentar, loguear, elegir backend en tiempo de ejecución mediante
//! Cargo features) puede escribirse genérico sobre `P: SettlementProver`,
//! sin ligarse a los tipos concretos de un backend concreto.
//!
//! ## Asimetría real entre los tres backends, documentada aquí
//!
//! **Qué expone cada circuito por su cuenta:**
//! - `zk-core::ComplianceCircuitWithState` calcula y expone `state_root`,
//!   `regulatory_limit` y `nullifier` como campos públicos de sí mismo
//!   (el nullifier se deriva automáticamente en `new()`).
//! - `halo2-experiment::ComplianceCircuit` NO expone estos valores — el
//!   llamador debe calcularlos nativamente aparte (ver
//!   `compliance_real_proof.rs`) y pasarlos junto con el circuito.
//! - `stark-experiment` los DERIVA del testigo dentro de `prove`, de modo
//!   que es imposible declarar unos inputs públicos que no correspondan.
//!
//! **Y lo que `setup()` significa en cada uno — la diferencia que más
//! importa y la que un trait mal diseñado escondería:**
//! - Groth16: produce claves de una ceremonia con "residuo tóxico". Si
//!   ese residuo no se destruye, se pueden falsificar pruebas.
//! - Halo2/IPA: produce parámetros deterministas, sin secreto, pero caros
//!   (~176 s medidos) y necesarios en cada llamada posterior.
//! - STARK: es instantáneo y su salida es una elección de PARÁMETROS
//!   PÚBLICOS, no un artefacto criptográfico. `ProvingKey` y
//!   `VerifyingKey` son el mismo tipo y el mismo valor.
//!
//! Este trait unifica la FORMA de la llamada precisamente para que esas
//! diferencias sigan siendo visibles en los tipos, en vez de quedar
//! ocultas tras una interfaz que finja que los tres son equivalentes.
//! Ver `stark-experiment::settlement_prover_impl`, cuyo test
//! `setup_produces_no_secret_material` convierte esa propiedad en una
//! aserción ejecutable.
//!
//! Por eso `Witness` en cada implementación incluye lo que haga falta
//! para que `prove` tenga todo lo necesario, sin forzar una forma común
//! que no reflejaría la realidad de cada backend.

/// Backend de pruebas de cumplimiento intercambiable.
pub trait SettlementProver {
    type Witness;
    type PublicInput;
    type Proof;
    type ProvingKey;
    type VerifyingKey;
    type Error: std::error::Error;

    /// Genera (o deriva) las claves de prueba/verificación. La semántica
    /// exacta ("trusted setup por circuito" vs. "setup universal
    /// determinista") depende del backend — este trait no la esconde,
    /// solo unifica la FORMA de la llamada.
    fn setup(rng_seed: u64) -> Result<(Self::ProvingKey, Self::VerifyingKey), Self::Error>;

    /// Genera una prueba a partir de un testigo.
    fn prove(
        pk: &Self::ProvingKey,
        witness: Self::Witness,
        rng_seed: u64,
    ) -> Result<(Self::Proof, Self::PublicInput), Self::Error>;

    /// Verifica una prueba contra los inputs públicos declarados.
    fn verify(
        vk: &Self::VerifyingKey,
        public_input: &Self::PublicInput,
        proof: &Self::Proof,
    ) -> Result<bool, Self::Error>;
}
