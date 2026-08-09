//! # El hash nativo, **fuera del probador**
//!
//! Una sola función —`native_merge`— y la constante que necesita. Nada más.
//!
//! ## ⚠️ Por qué está separado: no es organización
//!
//! **§243** estableció que **un verificador independiente no compila el
//! servidor**: `zk-ssl-verify` solo depende de `xmss`, y hay compuerta que
//! lo vigila.
//!
//! Este crate **extiende esa regla un nivel más abajo: tampoco compila el
//! probador.**
//!
//! El problema apareció al diseñar el **recibo de inclusión**: cualquier
//! cosa con forma *hoja → camino Merkle → raíz → cabeza firmada* necesita
//! **el mismo hash que el nodo**, y hasta ahora ese hash **solo existía
//! dentro de `stark-experiment`**, que arrastra `winterfell` entero,
//! `sled` y `settlement-prover`.
//!
//! > **La primitiva de verificación independiente no estaba donde tenía que
//! > estar**, y no se sabía porque **hasta ahora nadie fuera del nodo había
//! > necesitado recomputar una raíz**.
//!
//! Las dos salidas obvias eran malas:
//!
//! | salida | por qué no |
//! |---|---|
//! | reimplementar `native_merge` en el verificador | **dos implementaciones del mismo hash**, que divergirían **en silencio** — un recibo válido declarado inválido, o peor, al revés. Es lo que §253 evitó reusando `GuardianIndice` **entero** |
//! | que el verificador dependa de `stark-experiment` | **mata la propiedad de §243**, que tiene compuerta |
//!
//! ## Qué necesita de verdad, y qué no
//!
//! `native_merge` usa **tres cosas**, y **ninguna es del AIR**:
//! `Rp64_256` (de `winter-crypto`), `BaseElement` (de `winter-math`) y
//! `STATE_WIDTH`, que es **una constante del propio hasher** —
//! `Rp64_256::STATE_WIDTH`—, no del circuito.
//!
//! ⚠️ **No usa `apply_sbox`, ni `NUM_ROUNDS`, ni `MerkleTree`, ni
//! `ColMatrix`.** El AIR del circuito de hash **se queda entero donde
//! está**: aquí no se mueve nada de la maquinaria de pruebas.
//!
//! ## ⚠️ Las versiones van fijadas con `=`
//!
//! `winter-math` y `winter-crypto` se toman **sueltos y clavados a
//! `=0.13.1`**, que es lo que el `Cargo.lock` ya tenía resuelto vía
//! `winterfell 0.13`.
//!
//! Si se tomaran por rango, cargo podría resolver **dos versiones del mismo
//! subcrate** en el árbol — y **dos `BaseElement` de versiones distintas no
//! son el mismo tipo**. El buen caso sería que no compilara; el malo, que
//! compilara con conversiones y **divergiera en silencio**.
//!
//! ## Este sello no cambia ni un byte
//!
//! Es un **refactor puro**: `stark-experiment` reexporta `native_merge`
//! desde aquí, y **los 172 usos en 31 ficheros siguen igual**.
//!
//! ⚠️ Y la corrección **no la demuestra un argumento, la demuestran las
//! compuertas que ya existen**: 297 tests de `stark-experiment`, 256 de la
//! capa, los seis censos y **la conformidad `zkssl/0.2`, que pincha el
//! `epoch_digest`**.
//!
//! Como `native_merge` es **la primitiva del árbol y de `chain_digest`**,
//! un corte mal hecho **revienta la conformidad inmediatamente**. No hay
//! forma de que pase inadvertido.

use winter_crypto::hashers::Rp64_256;
use winter_math::fields::f64::BaseElement;
use winter_math::FieldElement;

/// Cuatro elementos de campo: el resumen que circula por todo el proyecto.
pub type Digest = [BaseElement; 4];

/// Anchura del estado de la permutación, **tomada del propio hasher**.
///
/// ⚠️ No es una constante del circuito: es `Rp64_256::STATE_WIDTH`. Copiarla
/// a mano la desligaría de la implementación que de verdad se usa.
pub const STATE_WIDTH: usize = Rp64_256::STATE_WIDTH;

/// Hash 2-a-1 nativo, con la implementación **real** de `winter-crypto`.
///
/// ⚠️ Es **la primitiva del árbol disperso y de `chain_digest`**: si esto
/// cambiara, cambiarían todas las raíces del proyecto y la conformidad
/// `zkssl/0.2` lo diría en el acto.
pub fn native_merge(left: Digest, right: Digest) -> Digest {
    let mut state = [BaseElement::ZERO; STATE_WIDTH];
    state[4..8].copy_from_slice(&left);
    state[8..12].copy_from_slice(&right);
    Rp64_256::apply_permutation(&mut state);
    [state[4], state[5], state[6], state[7]]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(n: u64) -> Digest {
        [
            BaseElement::new(n),
            BaseElement::new(n + 1),
            BaseElement::new(n + 2),
            BaseElement::new(n + 3),
        ]
    }

    #[test]
    fn el_hash_es_determinista() {
        assert_eq!(native_merge(d(1), d(5)), native_merge(d(1), d(5)));
    }

    #[test]
    fn el_orden_de_los_hijos_importa() {
        // ⚠️ Si no importara, un camino Merkle no distinguiria izquierda de
        // derecha y CUALQUIER hoja probaria CUALQUIER posicion.
        assert_ne!(native_merge(d(1), d(5)), native_merge(d(5), d(1)));
    }

    #[test]
    fn entradas_distintas_dan_salidas_distintas() {
        let a = native_merge(d(0), d(0));
        assert_ne!(a, native_merge(d(0), d(1)));
        assert_ne!(a, native_merge(d(1), d(0)));
    }

    #[test]
    fn la_anchura_del_estado_es_la_del_hasher() {
        // ⚠️ Copiar 12 a mano desligaria la constante de la implementacion.
        assert_eq!(STATE_WIDTH, Rp64_256::STATE_WIDTH);
        assert!(STATE_WIDTH >= 12, "native_merge escribe hasta el indice 11");
    }
}
