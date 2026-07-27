//! Sustituto real de `toy_hash`: Poseidon, un hash amigable para circuitos
//! ZK, implementado sobre `ark-crypto-primitives`.
//!
//! ## ⚠️ Esta es la pieza de mayor riesgo de todo el proyecto — léase antes de compilar
//!
//! De todos los módulos escritos en este proyecto, este es donde tengo
//! MENOS certeza de que la API coincida exactamente con lo que resuelva
//! `cargo` — más incluso que con `FpVar` (que ya tuvo que corregirse una
//! vez). Es razonable esperar 2-3 rondas de corrección aquí, no una.
//!
//! Lo que SÍ es una decisión deliberada y no un descuido: los parámetros
//! de Poseidon (matriz MDS, constantes de ronda) NO se escriben a mano en
//! ningún punto de este archivo. Eso sería exactamente el tipo de
//! "criptografía casera" que este proyecto llevaba meses arrastrando en
//! sus 57 crates anteriores. En su lugar, se generan con
//! `find_poseidon_ark_and_mds`, una función de `ark-crypto-primitives` que
//! sigue el algoritmo de generación determinista descrito en el paper
//! original de Poseidon (Grassi, Khovratovich, Rechberger, Roy, Schofnegger,
//! 2019) — el mismo mecanismo que usan los propios tests de esa librería
//! para generar parámetros de prueba. Si esta función no existe con esta
//! firma exacta en la versión que resuelva `cargo`, es un error de API que
//! se corrige con el compilador real, como todo lo demás en este proyecto.
//! Lo que NUNCA se hace es "solucionarlo" rellenando constantes a mano
//! como sustituto — si esta pieza falla, hay que arreglar la llamada a la
//! función, no inventar los números.

//! ## 🐌 Corrección de rendimiento aplicada tras ejecución real
//!
//! La primera versión de este archivo recalculaba los parámetros de
//! Poseidon (`find_poseidon_ark_and_mds`) en CADA llamada a
//! `secure_hash`/`secure_hash_gadget`. Como `enforce_merkle_membership`
//! llama a estas funciones 20 veces (una por nivel del árbol), y
//! `generate_constraints` se ejecuta varias veces por cada prueba (en el
//! pre-check de satisfacibilidad, en el setup, y en la generación de la
//! prueba), esto suponía docenas de recálculos completos por cada test —
//! confirmado en la práctica porque la ejecución real de los tests parecía
//! congelarse. Ahora los parámetros se calculan UNA SOLA VEZ por tipo de
//! campo y proceso, cacheados con `TypeId`, y se reutilizan después.

use ark_crypto_primitives::sponge::constraints::CryptographicSpongeVar;
use ark_crypto_primitives::sponge::poseidon::constraints::PoseidonSpongeVar;
use ark_crypto_primitives::sponge::poseidon::{find_poseidon_ark_and_mds, PoseidonConfig};
use ark_crypto_primitives::sponge::poseidon::PoseidonSponge;
use ark_crypto_primitives::sponge::{Absorb, CryptographicSponge};
use ark_ff::PrimeField;
use ark_r1cs_std::fields::fp::FpVar;
use ark_relations::r1cs::{ConstraintSystemRef, SynthesisError};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// Parámetros de seguridad estándar para ~128 bits de seguridad sobre un
/// cuerpo de ~255 bits (como el escalar de BLS12-381): 8 rondas completas,
/// 57 rondas parciales, alpha = 5 como exponente de la S-box, rate = 2
/// (2 elementos absorbidos por permutación), capacity = 1. Estos NÚMEROS
/// DE RONDAS (no las constantes en sí) son los parámetros ampliamente
/// documentados para este tamaño de cuerpo y nivel de seguridad — el mismo
/// conteo que usan implementaciones de referencia como Filecoin/Neptune
/// para BLS12-381. Las constantes derivadas de estos números sí se generan
/// por algoritmo, no se copian de ningún sitio.
const FULL_ROUNDS: usize = 8;
const PARTIAL_ROUNDS: usize = 57;
const ALPHA: u64 = 5;
const RATE: usize = 2;
const CAPACITY: usize = 1;

type ConfigCache = Mutex<HashMap<TypeId, Box<dyn Any + Send + Sync>>>;

fn config_cache() -> &'static ConfigCache {
    static CACHE: OnceLock<ConfigCache> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Genera (o recupera de caché) la configuración de Poseidon para el
/// cuerpo `F`. Solo se calcula de verdad la PRIMERA vez que se pide para
/// cada tipo de campo en todo el proceso; las siguientes llamadas
/// devuelven una copia de la ya calculada.
pub fn poseidon_config<F: PrimeField + 'static>() -> PoseidonConfig<F> {
    let type_id = TypeId::of::<F>();

    {
        let cache = config_cache().lock().expect("el mutex de cache de Poseidon no deberia estar envenenado");
        if let Some(boxed) = cache.get(&type_id) {
            if let Some(config) = boxed.downcast_ref::<PoseidonConfig<F>>() {
                return config.clone();
            }
        }
    }

    let (ark, mds) = find_poseidon_ark_and_mds::<F>(
        F::MODULUS_BIT_SIZE as u64,
        RATE,
        FULL_ROUNDS as u64,
        PARTIAL_ROUNDS as u64,
        0,
    );
    let config: PoseidonConfig<F> =
        PoseidonConfig::new(FULL_ROUNDS, PARTIAL_ROUNDS, ALPHA, mds, ark, RATE, CAPACITY);

    let mut cache = config_cache().lock().expect("el mutex de cache de Poseidon no deberia estar envenenado");
    cache.insert(type_id, Box::new(config.clone()));

    config
}

/// Comprime dos elementos de campo en uno usando Poseidon (fuera de
/// circuito). Firma compatible con `toy_hash`, para poder sustituirla
/// directamente en `merkle.rs` y `nullifier.rs`.
pub fn secure_hash<F: PrimeField + Absorb + 'static>(x: F, y: F) -> F {
    let config = poseidon_config::<F>();
    let mut sponge = PoseidonSponge::new(&config);
    sponge.absorb(&x);
    sponge.absorb(&y);
    let out: Vec<F> = sponge.squeeze_field_elements(1);
    out[0]
}

/// Versión en-circuito de `secure_hash`. A diferencia de `toy_hash_gadget`,
/// necesita acceso al `ConstraintSystemRef` para construir el sponge.
pub fn secure_hash_gadget<F: PrimeField + Absorb + 'static>(
    cs: ConstraintSystemRef<F>,
    x: &FpVar<F>,
    y: &FpVar<F>,
) -> Result<FpVar<F>, SynthesisError> {
    let config = poseidon_config::<F>();
    let mut sponge = PoseidonSpongeVar::new(cs, &config);
    sponge.absorb(x)?;
    sponge.absorb(y)?;
    let out = sponge.squeeze_field_elements(1)?;
    Ok(out[0].clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bls12_381::Fr;

    #[test]
    fn secure_hash_is_deterministic() {
        let a = Fr::from(3u64);
        let b = Fr::from(5u64);
        assert_eq!(
            secure_hash(a, b),
            secure_hash(a, b),
            "el mismo par de entradas debe producir siempre la misma salida"
        );
    }

    #[test]
    fn secure_hash_is_sensitive_to_input_order() {
        // Propiedad basica de un hash de compresion serio: hash(a, b) no
        // deberia coincidir con hash(b, a) en general.
        let a = Fr::from(3u64);
        let b = Fr::from(5u64);
        assert_ne!(secure_hash(a, b), secure_hash(b, a));
    }

    #[test]
    fn secure_hash_differs_from_toy_hash() {
        // No es en si mismo un requisito de seguridad, pero confirma que
        // realmente se esta usando una funcion distinta de toy_hash, no
        // la misma logica camuflada con otro nombre.
        let a = Fr::from(3u64);
        let b = Fr::from(5u64);
        assert_ne!(
            secure_hash(a, b),
            crate::merkle::toy_hash(a, b),
            "secure_hash y toy_hash deben dar resultados distintos: son funciones diferentes"
        );
    }

    #[test]
    fn secure_hash_gadget_matches_native_computation() {
        use ark_r1cs_std::alloc::AllocVar;
        use ark_r1cs_std::R1CSVar;
        use ark_relations::r1cs::ConstraintSystem;

        let a = Fr::from(7u64);
        let b = Fr::from(11u64);
        let expected = secure_hash(a, b);

        let cs = ConstraintSystem::<Fr>::new_ref();
        let a_var = FpVar::new_witness(cs.clone(), || Ok(a)).unwrap();
        let b_var = FpVar::new_witness(cs.clone(), || Ok(b)).unwrap();

        let result_var = secure_hash_gadget(cs.clone(), &a_var, &b_var)
            .expect("secure_hash_gadget no deberia fallar al construir las restricciones");

        assert_eq!(
            result_var.value().unwrap(),
            expected,
            "el resultado en-circuito debe coincidir con el calculo nativo"
        );
        assert!(cs.is_satisfied().unwrap());
    }
}
