//! RFC-0007 E5, corte 3b (§475): las reglas del arbol de CUENTAS que el
//! verificador necesita para sostener, sin la capa, el rechazo
//! `AccountNotFound(i)`: que la hoja de `i` bajo el `accountsRoot` de una
//! cabeza firmada ES la vacia.
//!
//! Es el ESPEJO de `congelados`. Alli la hoja NO vacia prueba que la cuenta
//! esta congelada; aqui la hoja VACIA prueba que no hay cuenta. Las dos
//! primeras reglas son identicas, y por la misma razon:
//!
//! - **La profundidad la fija el verificador, no el camino.** El arbol se
//!   sube con `native_merge`, que hashea hojas y nodos internos igual, asi
//!   que un camino corto llegaria a la MISMA raiz. Aqui muerde por el otro
//!   lado: el nodo de nivel 1 NO es el cero ni en un arbol vacio, luego un
//!   camino truncado no diria <<no existe>> sino lo contrario. Falla cerrado
//!   por la regla de profundidad, y el testigo de abajo lo ensena.
//! - **La posicion es el indice de la cuenta**, y el camino se cruza contra
//!   sus bits: la convencion de `sparse_tree.rs`, la misma de `consumos` y
//!   de `congelados`.
//! - **No existir es hoja VACIA.** Es la regla de la capa: una cuenta se
//!   coloca por su identidad con sondeo lineal (F3) y `records.get(&index)`
//!   devuelve nada exactamente cuando su hoja esta a cero.
//!
//! ⚠️ `ACCOUNTS_DEPTH` vale HOY lo mismo que `FROZEN_DEPTH` y son dos hechos
//! distintos (§474): el arbol de cuentas nunca tuvo otra profundidad y el de
//! congelados si, 24 antes de v7. Reusar una por la otra saldria verde por
//! casualidad.
//!
//! ⚠️ La cabeza que se exige es la del `seq` EXACTO. Una POSTERIOR tambien
//! probaria la ausencia -el conjunto de cuentas solo crece, luego una hoja
//! vacia en un `seq` mayor lo estaba antes-, pero eso es una regla distinta
//! y pide su propio testigo: aqui se DECLARA y no se acepta.

// Los tipos y la subida viajan re-exportados desde aqui, como en `congelados`.
// La hoja vacia es LA MISMA que la del arbol de consumos: un solo productor.
pub use zk_ssl_hash::{path_root, Digest, ACCOUNTS_DEPTH};

use crate::consumos::hoja_vacia;

/// La convencion del camino para la cuenta `indice`: el nivel `i` va a la
/// derecha si el bit `i` del indice esta a uno (`sparse_tree.rs`:
/// `is_right.push(idx % 2 == 1); idx /= 2`).
pub fn is_right_de_indice(indice: u64) -> Vec<bool> {
    (0..ACCOUNTS_DEPTH).map(|i| (indice >> i) & 1 == 1).collect()
}

/// ¿El camino recibido es el de ESTA cuenta? Un indice que no cabe en el
/// arbol no cruza con nada: sus bits altos se perderian por el camino.
pub fn cruza_indice(indice: u64, is_right: &[bool]) -> bool {
    indice < (1u64 << ACCOUNTS_DEPTH) && is_right == is_right_de_indice(indice).as_slice()
}

/// Sube `hoja` por el camino, **o `None` si el camino no mide
/// `ACCOUNTS_DEPTH` en sus dos lados**. Es la regla que cierra el camino
/// truncado: la profundidad no la elige quien sirve el camino.
pub fn raiz_de_hoja(hoja: Digest, siblings: &[Digest], is_right: &[bool]) -> Option<Digest> {
    if siblings.len() != ACCOUNTS_DEPTH || is_right.len() != ACCOUNTS_DEPTH {
        return None;
    }
    Some(path_root(hoja, siblings, is_right))
}

/// No hay cuenta, para la capa, es exactamente la hoja vacia.
pub fn no_existe(hoja: Digest) -> bool {
    hoja == hoja_vacia()
}

#[cfg(test)]
mod tests {
    use super::*;
    use winter_math::fields::f64::BaseElement;
    use winter_math::FieldElement;
    use zk_ssl_hash::native_merge;

    /// Un arbol de mentira de profundidad `ACCOUNTS_DEPTH` con UNA hoja
    /// puesta en `indice`: devuelve la raiz y el camino de esa posicion. Los
    /// hermanos son la cadena de vacios, como en el arbol disperso real.
    fn arbol_con(indice: u64, hoja: Digest) -> (Digest, Vec<Digest>, Vec<bool>) {
        let mut vacios = vec![hoja_vacia()];
        for k in 1..ACCOUNTS_DEPTH {
            let anterior = vacios[k - 1];
            vacios.push(native_merge(anterior, anterior));
        }
        let is_right = is_right_de_indice(indice);
        let raiz = path_root(hoja, &vacios, &is_right);
        (raiz, vacios, is_right)
    }

    /// Una hoja de cuenta OCUPADA. El verificador NO necesita su forma -juzga
    /// hoja vacia contra hoja no vacia-; el test solo quiere una que no sea
    /// el cero.
    fn ocupada() -> Digest {
        [BaseElement::new(7), BaseElement::ZERO, BaseElement::ZERO, BaseElement::ZERO]
    }

    #[test]
    fn una_hoja_vacia_sube_a_su_raiz_y_la_cuenta_no_existe() {
        let i = 0x3C09_AA89; // el indice REAL de la captura del §475
        let (raiz, siblings, is_right) = arbol_con(i, hoja_vacia());
        assert_eq!(raiz_de_hoja(hoja_vacia(), &siblings, &is_right), Some(raiz));
        assert!(cruza_indice(i, &is_right), "el camino es el de la cuenta");
        assert!(no_existe(hoja_vacia()), "la hoja vacia es la cuenta que no esta");
    }

    #[test]
    fn una_hoja_ocupada_desmiente_la_causa() {
        // EL DISFRAZ, y es el testigo que NO tiene vector: el nodo dice que
        // la cuenta no existe y la hoja bajo el accountsRoot esta OCUPADA.
        // Fabricarlo en JSON pediria el camino real de una cuenta viva, que
        // ningun metodo del cable sirve (D-G): se prueba aqui.
        let i = 11;
        let (raiz, siblings, is_right) = arbol_con(i, ocupada());
        assert!(!no_existe(ocupada()), "una hoja ocupada no puede decir <<no existe>>");
        assert_eq!(raiz_de_hoja(ocupada(), &siblings, &is_right), Some(raiz));
        assert_ne!(
            raiz_de_hoja(hoja_vacia(), &siblings, &is_right),
            Some(raiz),
            "la hoja vacia no puede subir a la raiz de un arbol que tiene la cuenta"
        );
    }

    #[test]
    fn un_camino_truncado_sube_a_la_misma_raiz_y_aqui_no_pasa() {
        // El ataque de la hoja-nodo, que es la razon de fijar la profundidad.
        let i = 0x2A;
        let (raiz, siblings, is_right) = arbol_con(i, hoja_vacia());
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
        // ...y su «hoja» NO es la vacia, asi que por aqui el truncado no
        // colaria un <<no existe>>: falla por la profundidad, y ademas por la
        // regla. Los dos lados tienen que medir lo mismo.
        assert!(!no_existe(nodo1), "el nodo de nivel 1 no es el cero");
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
        let fuera = i | (1u64 << ACCOUNTS_DEPTH);
        assert_eq!(is_right_de_indice(fuera), is_right_de_indice(i));
        assert!(!cruza_indice(fuera, &is_right_de_indice(i)));
        assert!(!cruza_indice(i, &[]), "ni uno vacio");
    }
}
