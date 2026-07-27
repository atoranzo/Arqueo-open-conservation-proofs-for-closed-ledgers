//! Utilidades compartidas por los tests del backend PLONK-KZG.
//!
//! ## Por qué existe el SRS compartido
//!
//! Generar los parámetros públicos cuesta segundos y crece con el
//! tamaño del circuito. Con un `OnceLock` se paga una sola vez por
//! ejecución de la suite en vez de una vez por test.
//!
//! **Y una nota sobre los tiempos**: los circuitos de este backend son
//! grandes (el árbol de Merkle solo ya son ~20.000 puertas). Ejecutar
//! los tests en debug tarda varios minutos; **usar siempre `--release`**.

use dusk_plonk::prelude::*;
use std::sync::OnceLock;

/// Tamaño del SRS compartido. Debe cubrir el MAYOR de los circuitos:
/// el de partida doble hace cuatro subidas del árbol (~20.000 puertas
/// cada una) y ronda las 90.000, así que hace falta 2^17 = 131.072.
///
/// Generarlo es lo más lento de la suite; por eso el `OnceLock`.
pub const SHARED_CAPACITY: usize = 1 << 17;

static SHARED_PP: OnceLock<PublicParameters> = OnceLock::new();

/// Parámetros públicos compartidos.
///
/// ⚠️ Generados por UNA SOLA PARTE. Válido para tests; en producción
/// habría que usar el SRS de una ceremonia real — que en el caso de
/// dusk-plonk existe y es pública (ver la nota de cabecera de `lib.rs`).
pub fn shared_pp() -> &'static PublicParameters {
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    SHARED_PP.get_or_init(|| {
        let mut rng = StdRng::seed_from_u64(0x5E7);
        PublicParameters::setup(SHARED_CAPACITY, &mut rng)
            .expect("el setup del SRS no deberia fallar")
    })
}
