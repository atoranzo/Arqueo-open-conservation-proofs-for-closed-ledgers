//! # Las primitivas de FORMATO, fuera del probador
//!
//! ⚠️ **El nombre del crate dice `hash`, y ya no es solo el hash.** Lo que
//! vive aquí es **lo que el nodo y un verificador independiente tienen que
//! componer IGUAL**: el hash 2-a-1, cómo se embebe un `u64`, cómo se sube
//! un camino Merkle, y **cómo se compone el digest de una cabeza**.
//!
//! > **Una decisión de formato tiene que tener UNA SOLA DEFINICIÓN**, por
//! > la misma razón que el hash: si el nodo y el verificador la componen
//! > distinto, **divergen en silencio**.
//!
//! §254 movió `native_merge` aquí y creyó cerrar el problema. Al escribir
//! el **recibo de inclusión** aparecieron **tres piezas más** que el
//! verificador necesitaba y no tenía: `as_digest` era **privada** en
//! `log.rs`, subir el camino vivía en `stark-experiment` **atado a
//! `TREE_DEPTH`**, y **la composición de `EpochHead::digest()`** estaba
//! solo en el nodo. **El problema no estaba resuelto: estaba destapado a
//! medias.**
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

/// Embebe un `u64` como digest: el valor en el primer elemento, ceros el
/// resto.
///
/// ⚠️ Era **privada** en `zk-ssl/src/log.rs`. Es una **decisión de
/// formato** —dónde va el número y con qué se rellena—, así que un
/// verificador independiente tiene que usar **esta misma**, no una copia.
pub fn as_digest(x: u64) -> Digest {
    [
        BaseElement::new(x),
        BaseElement::ZERO,
        BaseElement::ZERO,
        BaseElement::ZERO,
    ]
}

/// Sube un camino Merkle desde la hoja y devuelve la raíz.
///
/// ⚠️ **Itera sobre la LONGITUD DEL CAMINO, no sobre una constante.** La
/// versión de `stark-experiment` usaba `TREE_DEPTH` fijo, mientras
/// `SparseTree::path_for` genera caminos de `self.depth`: con un árbol de
/// otra profundidad, la constante habría leído fuera del camino o dejado
/// niveles sin subir.
///
/// `is_right[i] == true` significa que **el nodo actual va a la derecha** y
/// el hermano a la izquierda.
pub fn path_root(leaf: Digest, siblings: &[Digest], is_right: &[bool]) -> Digest {
    debug_assert_eq!(siblings.len(), is_right.len(), "camino descuadrado");
    let mut current = leaf;
    for (hermano, derecha) in siblings.iter().zip(is_right) {
        current = if *derecha {
            native_merge(*hermano, current)
        } else {
            native_merge(current, *hermano)
        };
    }
    current
}

/// Compone el digest de una cabeza de época.
///
/// ⚠️ **Esta es LA composición**, y `EpochHead::digest()` la llama. Un
/// verificador que quiera comprobar que una raíz de cuentas pertenece a una
/// cabeza firmada **necesita componerla exactamente igual** — y la única
/// forma segura de garantizarlo es que **sea la misma función**.
pub fn epoch_digest(
    seq: u64,
    accounts_root: Digest,
    pending_root: Digest,
    frozen_root: Digest,
    chain_digest: Digest,
) -> Digest {
    let a = native_merge(as_digest(seq), accounts_root);
    let b = native_merge(pending_root, frozen_root);
    native_merge(native_merge(a, b), chain_digest)
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
    fn as_digest_pone_el_valor_delante_y_ceros_detras() {
        let v = as_digest(7);
        assert_eq!(v[0], BaseElement::new(7));
        assert!(v[1..].iter().all(|x| *x == BaseElement::ZERO));
    }

    #[test]
    fn un_camino_de_un_nivel_sube_en_los_dos_sentidos() {
        // ⚠️ Si el orden no importara, un camino no distinguiria izquierda
        // de derecha y CUALQUIER hoja probaria CUALQUIER posicion.
        let (hoja, hermano) = (d(1), d(9));
        assert_eq!(path_root(hoja, &[hermano], &[false]), native_merge(hoja, hermano));
        assert_eq!(path_root(hoja, &[hermano], &[true]), native_merge(hermano, hoja));
    }

    #[test]
    fn path_root_itera_sobre_el_camino_no_sobre_una_constante() {
        // ⚠️ La version de stark-experiment usaba TREE_DEPTH FIJO. Con un
        // camino mas corto habria leido fuera; con uno mas largo, habria
        // dejado niveles sin subir.
        let hoja = d(1);
        assert_eq!(path_root(hoja, &[], &[]), hoja, "camino vacio: la hoja ES la raiz");
        let dos = path_root(hoja, &[d(9), d(11)], &[false, true]);
        assert_eq!(dos, native_merge(d(11), native_merge(hoja, d(9))));
        let tres = path_root(hoja, &[d(9), d(11), d(13)], &[false, true, false]);
        assert_ne!(dos, tres, "cada nivel cuenta");
    }

    #[test]
    fn el_digest_de_epoca_depende_de_sus_cinco_partes() {
        // ⚠️ Si alguna no entrara, el operador podria cambiarla sin que la
        // cabeza firmada lo delatara.
        let base = epoch_digest(1, d(10), d(20), d(30), d(40));
        assert_ne!(base, epoch_digest(2, d(10), d(20), d(30), d(40)), "seq");
        assert_ne!(base, epoch_digest(1, d(11), d(20), d(30), d(40)), "accounts_root");
        assert_ne!(base, epoch_digest(1, d(10), d(21), d(30), d(40)), "pending_root");
        assert_ne!(base, epoch_digest(1, d(10), d(20), d(31), d(40)), "frozen_root");
        assert_ne!(base, epoch_digest(1, d(10), d(20), d(30), d(41)), "chain_digest");
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
