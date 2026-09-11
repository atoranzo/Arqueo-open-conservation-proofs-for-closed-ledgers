//! RFC-0007 E3b (§458): las reglas del arbol de CONGELADOS que el
//! verificador necesita para sostener, sin la capa, el rechazo
//! `AccountFrozen(i)`: que la hoja de `i` bajo el `frozenRoot` de una cabeza
//! firmada NO es la vacia.
//!
//! Tres reglas, y las tres son la mitad de la prueba:
//!
//! - **La profundidad la fija el verificador, no el camino.** El arbol se
//!   sube con `native_merge`, que hashea hojas y nodos internos igual. Un
//!   camino de `FROZEN_DEPTH - 1` niveles cuya «hoja» fuera el nodo de nivel
//!   1 subiria a la MISMA raiz, y ese nodo no es el cero -ni siquiera en un
//!   arbol vacio, donde vale `native_merge(0, 0)`-: un verificador que
//!   aceptara la profundidad que trae el camino daria por congelada
//!   CUALQUIER cuenta. El testigo de abajo lo ensena.
//! - **La posicion es el indice de la cuenta**, y el camino se cruza contra
//!   sus bits: la convencion de `sparse_tree.rs`, la misma de `consumos`.
//! - **Congelada es hoja NO vacia.** Es la regla de la capa (`is_frozen` es
//!   `is_occupied`), no la marca concreta: las hojas son valores libres
//!   (§58.3), y lo que la capa rechaza es cualquier hoja distinta del cero.
//!
//! ⚠️ La profundidad es la del mundo v7 (`meta:geometry_v7`): un libro no
//! migrado reconstruye el arbol a 24 niveles y su camino NO pasa por aqui.
//! Falla cerrado, y es lo declarado.

// Los tipos y la subida viajan re-exportados desde aqui, como en `consumos`.
// La hoja vacia es LA MISMA que la del arbol de consumos: un solo productor.
pub use zk_ssl_hash::{path_root, Digest, FROZEN_DEPTH};

use crate::consumos::hoja_vacia;

/// La convencion del camino para la cuenta `indice`: el nivel `i` va a la
/// derecha si el bit `i` del indice esta a uno (`sparse_tree.rs`:
/// `is_right.push(idx % 2 == 1); idx /= 2`).
pub fn is_right_de_indice(indice: u64) -> Vec<bool> {
    (0..FROZEN_DEPTH).map(|i| (indice >> i) & 1 == 1).collect()
}

/// ¿El camino recibido es el de ESTA cuenta? Un indice que no cabe en el
/// arbol no cruza con nada: sus bits altos se perderian por el camino.
pub fn cruza_indice(indice: u64, is_right: &[bool]) -> bool {
    indice < (1u64 << FROZEN_DEPTH) && is_right == is_right_de_indice(indice).as_slice()
}

/// Sube `hoja` por el camino, **o `None` si el camino no mide
/// `FROZEN_DEPTH` en sus dos lados**. Es la regla que cierra el camino
/// truncado: la profundidad no la elige quien sirve el camino.
pub fn raiz_de_hoja(hoja: Digest, siblings: &[Digest], is_right: &[bool]) -> Option<Digest> {
    if siblings.len() != FROZEN_DEPTH || is_right.len() != FROZEN_DEPTH {
        return None;
    }
    Some(path_root(hoja, siblings, is_right))
}

/// Congelada, para la capa, es cualquier hoja distinta de la vacia.
pub fn esta_congelada(hoja: Digest) -> bool {
    hoja != hoja_vacia()
}

#[cfg(test)]
mod tests {
    use super::*;
    use winter_math::fields::f64::BaseElement;
    use winter_math::FieldElement;
    use zk_ssl_hash::native_merge;

    /// Un arbol de mentira de profundidad `FROZEN_DEPTH` con UNA hoja puesta
    /// en `indice`: devuelve la raiz y el camino de esa posicion. Los
    /// hermanos son la cadena de vacios, como en el arbol disperso real.
    fn arbol_con(indice: u64, hoja: Digest) -> (Digest, Vec<Digest>, Vec<bool>) {
        let mut vacios = vec![hoja_vacia()];
        for k in 1..FROZEN_DEPTH {
            let anterior = vacios[k - 1];
            vacios.push(native_merge(anterior, anterior));
        }
        let is_right = is_right_de_indice(indice);
        let raiz = path_root(hoja, &vacios, &is_right);
        (raiz, vacios, is_right)
    }

    /// Una hoja real de la capa: `circuit_freeze::FROZEN_MARK` ("FROZ") en
    /// el primer limbo. El verificador NO la necesita -juzga hoja no vacia-;
    /// el test solo quiere la hoja que el libro escribe.
    fn marca() -> Digest {
        [BaseElement::new(0x4652_4F5A), BaseElement::ZERO, BaseElement::ZERO, BaseElement::ZERO]
    }

    #[test]
    fn una_hoja_marcada_sube_a_su_raiz_y_esta_congelada() {
        let i = 0x00C0_FFEE;
        let (raiz, siblings, is_right) = arbol_con(i, marca());
        assert_eq!(raiz_de_hoja(marca(), &siblings, &is_right), Some(raiz));
        assert!(cruza_indice(i, &is_right), "el camino es el de la cuenta");
        assert!(esta_congelada(marca()), "la marca de la capa es una hoja no vacia");
    }

    #[test]
    fn la_hoja_vacia_no_esta_congelada_y_prueba_lo_contrario() {
        // El disfraz de D-5: si el nodo dijera `AccountFrozen` de una cuenta
        // libre, el camino de la hoja VACIA a la misma raiz lo desmiente.
        let i = 7;
        let (raiz, siblings, is_right) = arbol_con(i, hoja_vacia());
        assert!(!esta_congelada(hoja_vacia()));
        assert_eq!(raiz_de_hoja(hoja_vacia(), &siblings, &is_right), Some(raiz));
        assert_ne!(
            raiz_de_hoja(marca(), &siblings, &is_right),
            Some(raiz),
            "la marca no puede subir a la raiz de un arbol que no la tiene"
        );
    }

    #[test]
    fn un_camino_truncado_sube_a_la_misma_raiz_y_aqui_no_pasa() {
        // EL ataque de la hoja-nodo, que es la razon de fijar la profundidad.
        let i = 0x2A;
        let (raiz, siblings, is_right) = arbol_con(i, hoja_vacia());
        // El nodo de nivel 1 sobre la cuenta `i`: su hoja (vacia) y su hermano.
        let nodo1 = if is_right[0] {
            native_merge(siblings[0], hoja_vacia())
        } else {
            native_merge(hoja_vacia(), siblings[0])
        };
        // Sin la regla, un camino de un nivel menos sube a la MISMA raiz...
        assert_eq!(
            path_root(nodo1, &siblings[1..], &is_right[1..]),
            raiz,
            "native_merge no separa hoja de nodo: el camino truncado llega"
        );
        // ...y su «hoja» no es la vacia: pasaria por congelada.
        assert!(esta_congelada(nodo1), "el nodo de nivel 1 no es el cero");
        // Con la regla, no hay raiz, y los dos lados miden lo mismo.
        assert!(raiz_de_hoja(nodo1, &siblings[1..], &is_right[1..]).is_none());
        assert!(raiz_de_hoja(hoja_vacia(), &siblings, &is_right[1..]).is_none());
    }

    #[test]
    fn un_camino_de_otra_cuenta_no_cruza() {
        let i = 0x1234_5678;
        assert!(cruza_indice(i, &is_right_de_indice(i)));
        assert!(!cruza_indice(i, &is_right_de_indice(i ^ 1)), "un bit distinto es otra cuenta");
        // Un indice que no cabe en el arbol tiene los mismos bits bajos que
        // otro que si: sin la cota, cruzaria con su camino.
        let fuera = i | (1u64 << FROZEN_DEPTH);
        assert_eq!(is_right_de_indice(fuera), is_right_de_indice(i));
        assert!(!cruza_indice(fuera, &is_right_de_indice(i)));
        assert!(!cruza_indice(i, &[]), "ni uno vacio");
    }
}
