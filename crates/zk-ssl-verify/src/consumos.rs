//! # Las reglas del arbol de consumos: la hoja, la posicion y el cruce
//!
//! ## Por que esto vive AQUI, y en un solo sitio
//!
//! El **constructor** del arbol es de la capa (§413, `consumo.rs`) y el
//! **camino** lo sirve el nodo (§417, `zkssl_consumoPath`). El
//! **verificador** que sube ese camino **sin la capa** es de este crate: si
//! la regla de que hoja se sube —o de que posicion ocupa— viviera solo en
//! la capa, aqui se escribiria otra version y habria **dos**. Es el mismo
//! argumento que sostiene `acuses` desde §274.
//!
//! ## La hoja decide QUE se prueba (RFC-0006, D-11)
//!
//! `SparseTree::path_for` devuelve **el mismo camino** para una posicion
//! este ocupada o vacia. Quien verifica elige la hoja: el **digest del
//! consumo** prueba que ESTA bajo la raiz; la **hoja vacia** prueba que NO
//! estaba. Una sola funcion en la capa, dos afirmaciones distintas aqui.
//!
//! ## El cruce, y por que sin el la mitad de la prueba es falsificable
//!
//! ⚠️ `raiz_de_ausencia(siblings, is_right) == consRoot` demuestra solo que
//! **hay ALGUNA posicion vacia bajo esa raiz**. Sin atar el camino a la
//! posicion del consumo, quien empaqueta elige cualquiera de las 2^63
//! libres y «prueba» la ausencia de CUALQUIER cosa. El unico enlace
//! consumo -> posicion es la derivacion, asi que el mando DERIVA la
//! posicion del consumo y CRUZA sus bits contra el `is_right` recibido.
//!
//! La mitad de PRESENCIA si es solida sin el cruce: no se fabrican
//! hermanos que suban a una raiz real.
//!
//! ## La longitud se comprueba aqui, y no con un `debug_assert`
//!
//! `path_root` lleva un `debug_assert_eq!` de longitudes y **el canon corre
//! en RELEASE**: alli no dispara. Y `zip` trunca en silencio al mas corto,
//! de modo que un camino de un solo nivel subiria un nivel y devolveria
//! algo con cara de raiz. Por eso la profundidad es un `Option`, no un
//! aviso de depuracion.

use zk_ssl_hash::as_digest;
// El mando no importa `zk-ssl-hash` por su cuenta: los tipos y las piezas
// que estas reglas usan viajan re-exportados desde aqui — un solo cable,
// tambien para tipos (el molde de `acuses`).
pub use zk_ssl_hash::{path_root, posicion_de_consumo, Digest, CONS_DEPTH};

/// La **hoja vacia** del arbol de consumos: el cero del campo.
///
/// La capa la teclea `[BaseElement::ZERO; 4]` (`sparse_tree.rs`), un tipo
/// que este crate no nombra fuera de los tests. `as_digest(0)` es
/// `embeber(BaseElement::new(0))` = `[0, 0, 0, 0]`: **el mismo valor por
/// otro camino**, y el testigo de abajo lo cruza contra los DOS
/// constructores independientes del nucleo.
pub fn hoja_vacia() -> Digest {
    as_digest(0)
}

/// La convencion del camino, **leida del productor** y no deducida
/// (`sparse_tree.rs`: `is_right.push(idx % 2 == 1); idx /= 2`): el nivel
/// `i` va a la derecha si el bit `i` de la posicion esta a uno.
pub fn is_right_de_posicion(pos: u64) -> Vec<bool> {
    (0..CONS_DEPTH).map(|i| (pos >> i) & 1 == 1).collect()
}

/// ¿El camino recibido es **el de esta posicion**? Es el cruce de D-17, y
/// lo que separa «esta posicion esta vacia» de «alguna posicion lo esta».
pub fn cruza_posicion(pos: u64, is_right: &[bool]) -> bool {
    is_right == is_right_de_posicion(pos).as_slice()
}

/// Sube `hoja` por el camino, **o `None` si el camino no tiene la
/// profundidad del arbol**. Privada: fuera se nombran las dos
/// afirmaciones, no la mecanica.
fn raiz_desde(hoja: Digest, siblings: &[Digest], is_right: &[bool]) -> Option<Digest> {
    if siblings.len() != CONS_DEPTH || is_right.len() != CONS_DEPTH {
        return None;
    }
    Some(path_root(hoja, siblings, is_right))
}

/// La raiz que prueba **presencia**: la hoja es el digest del consumo.
pub fn raiz_de_presencia(
    consumo: Digest,
    siblings: &[Digest],
    is_right: &[bool],
) -> Option<Digest> {
    raiz_desde(consumo, siblings, is_right)
}

/// La raiz que prueba **ausencia**: la hoja es la vacia.
pub fn raiz_de_ausencia(siblings: &[Digest], is_right: &[bool]) -> Option<Digest> {
    raiz_desde(hoja_vacia(), siblings, is_right)
}

#[cfg(test)]
mod tests {
    use super::*;
    use winter_math::fields::f64::BaseElement;
    use winter_math::FieldElement;
    use zk_ssl_hash::{digest_from_bytes, native_merge};

    /// Un arbol de mentira de profundidad `CONS_DEPTH` con UNA hoja puesta:
    /// devuelve la raiz y el camino de esa posicion. Los hermanos son la
    /// cadena de vacios, como en el arbol disperso real.
    fn arbol_con(pos: u64, hoja: Digest) -> (Digest, Vec<Digest>, Vec<bool>) {
        let mut vacios = vec![hoja_vacia()];
        for k in 1..CONS_DEPTH {
            let anterior = vacios[k - 1];
            vacios.push(native_merge(anterior, anterior));
        }
        let is_right = is_right_de_posicion(pos);
        let siblings: Vec<Digest> = vacios.clone();
        let raiz = path_root(hoja, &siblings, &is_right);
        (raiz, siblings, is_right)
    }

    #[test]
    fn la_hoja_vacia_es_el_cero_del_campo_por_dos_caminos() {
        // El cruce que sostiene TODO este modulo: la capa escribe la hoja
        // vacia con `BaseElement::ZERO`, que aqui solo se puede nombrar en
        // los tests. Si `as_digest(0)` no fuera ese valor, la ausencia se
        // comprobaria contra otro arbol y nada lo diria.
        assert_eq!(hoja_vacia(), [BaseElement::ZERO; 4], "la hoja vacia NO es el cero del campo");
        assert_eq!(
            hoja_vacia(),
            digest_from_bytes(&[0u8; 32]).expect("32 bytes"),
            "los dos constructores del nucleo discrepan"
        );
        // Y la cota que el propio nucleo pina: el digest cero vive en la 0.
        assert_eq!(posicion_de_consumo(&hoja_vacia()), 0);
    }

    #[test]
    fn la_convencion_sale_de_los_bits_de_la_posicion() {
        let v = is_right_de_posicion(0b1011);
        assert_eq!(v.len(), CONS_DEPTH, "el camino tiene la profundidad del arbol");
        assert_eq!(&v[..4], &[true, true, false, true], "bit i -> nivel i");
        assert!(v[4..].iter().all(|b| !*b), "y ceros por encima");
        assert!(is_right_de_posicion(0).iter().all(|b| !*b));
    }

    #[test]
    fn el_cruce_caza_un_camino_de_otra_posicion() {
        let pos = 0x2A_BC_DE;
        assert!(cruza_posicion(pos, &is_right_de_posicion(pos)));
        // UN bit distinto ya es otra posicion: es el falsador de D-17.
        assert!(
            !cruza_posicion(pos, &is_right_de_posicion(pos ^ 1)),
            "un camino de otra posicion NO puede pasar por el de esta"
        );
        assert!(!cruza_posicion(pos, &[]), "ni uno vacio");
    }

    #[test]
    fn un_camino_descuadrado_no_da_raiz() {
        // `zip` trunca en silencio y el `debug_assert` de `path_root` no
        // dispara en RELEASE: si esto devolviera algo, un camino de un
        // nivel «probaria» lo que quisiera.
        let corto = vec![hoja_vacia(); 3];
        let bits = vec![false; 3];
        assert!(raiz_de_ausencia(&corto, &bits).is_none(), "3 niveles no son 63");
        assert!(raiz_de_presencia(as_digest(7), &corto, &bits).is_none());
        let siblings = vec![hoja_vacia(); CONS_DEPTH];
        assert!(
            raiz_de_ausencia(&siblings, &vec![false; CONS_DEPTH - 1]).is_none(),
            "los dos lados del camino tienen que medir lo mismo"
        );
        assert!(raiz_de_ausencia(&siblings, &vec![false; CONS_DEPTH]).is_some());
    }

    #[test]
    fn presencia_y_ausencia_no_pueden_subir_a_la_misma_raiz() {
        let consumo = as_digest(0xC0_A9_2281);
        let pos = posicion_de_consumo(&consumo);
        let (raiz, siblings, is_right) = arbol_con(pos, consumo);
        assert_eq!(raiz_de_presencia(consumo, &siblings, &is_right), Some(raiz));
        assert_ne!(
            raiz_de_ausencia(&siblings, &is_right),
            Some(raiz),
            "si la ausencia subiera a la misma raiz, la prueba no diria nada"
        );
    }

    #[test]
    fn la_ausencia_en_posicion_ajena_no_prueba_la_del_consumo() {
        // EL testigo de D-17. La raiz de un arbol VACIO acepta la ausencia
        // en CUALQUIER posicion —eso es correcto y es justo el problema—,
        // asi que lo que ata la prueba a ESTE consumo es el cruce.
        let consumo = as_digest(0xDEAD_BEEF);
        let pos = posicion_de_consumo(&consumo);
        let ajena = pos ^ 1;
        let (raiz_vacia, siblings, _) = arbol_con(pos, hoja_vacia());
        let is_right_ajeno = is_right_de_posicion(ajena);

        // Sin el cruce, el camino ajeno sube a la MISMA raiz: la ausencia
        // «se prueba» con una posicion que no es la del consumo.
        assert_eq!(
            raiz_de_ausencia(&siblings, &is_right_ajeno),
            Some(raiz_vacia),
            "el arbol vacio acepta cualquier posicion: por eso hace falta el cruce"
        );
        // Con el cruce, no pasa.
        assert!(
            !cruza_posicion(pos, &is_right_ajeno),
            "CRITICO: sin este rechazo la mitad de AUSENCIA es falsificable"
        );
    }
}
