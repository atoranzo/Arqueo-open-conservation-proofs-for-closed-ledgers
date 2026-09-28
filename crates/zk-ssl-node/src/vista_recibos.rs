//! # La vista de recibos: el árbol de una era, computado del registro
//!
//! **No es una estructura: es una vista**, como [`crate::vista_acuses`]. Las
//! hojas se derivan de lo que [`crate::registro_recepcion`] anotó con `fsync`
//! al recibir —por cada `rx`, la era que declaró y el digest de su prueba—, y
//! los límites de la era son dos `recep_count` de cabezas consecutivas (D-C).
//! Nada nuevo se persiste; el árbol se computa al pedirlo y se tira.
//!
//! Las reglas de pertenencia, posición y hoja **no viven aquí**: viven en
//! `zk_ssl_verify::recibos`, porque el verificador del sobre de completitud
//! (E4) usará exactamente las mismas. Es el argumento de `acuses` desde §274.
//!
//! ## ⚠️ La era de la hoja NO es la de la cabeza que la contiene
//!
//! La era la fija el recibo AL RECIBIR —el índice XMSS de la última cabeza
//! firmada, más uno (D-D, §567)— y va DENTRO de la hoja. La posición, en
//! cambio, la fija el `rx` contra los dos `recep_count` de la cabeza que
//! cierra (D-C). Casi siempre coinciden; no siempre: una recepción que entra
//! entre que una cabeza se COMPONE y se FIRMA ve la anterior como última
//! firmada y declara su era, pero su `rx` ya es posterior al `recep_count`
//! compuesto y cae en el árbol de la SIGUIENTE. Por eso la vista recibe la
//! era de cada entrada como DATO y no la recalcula: recalcularla sería un
//! segundo productor de un número que ya se prometió.
//!
//! ## ⚠️ Falla CERRADA ante un `rx` repetido
//!
//! En el árbol de acuses cada `seq` sale del registro de la capa y no se
//! repite por construcción. Aquí las entradas salen de un directorio que el
//! operador escribe (D-I): dos entradas con el mismo `rx` en la misma era son
//! un registro roto —restaurado a medias, o escrito dos veces—, y `set_leaf`
//! se quedaría en silencio con la última. La vista lo NOMBRA y no compone.

use zk_ssl::sparse_tree::SparseTree;
use zk_ssl_verify::acuses::Digest;
use zk_ssl_verify::recibos::{hoja_de_recibo, indice_de_recibo, pertenece_a_era};

/// Una entrada del registro, tal como la vista la consume: `(rx, era, hash_prueba)`.
pub type Entrada = (u64, u64, Digest);

/// Lo que impide componer la raíz de una era.
#[derive(Debug, PartialEq, Eq)]
pub enum ErrorVista {
    /// Dos entradas con el mismo `rx` dentro de la era: el registro está roto.
    RxRepetido { rx: u64 },
}

/// El árbol de la era `(limite_anterior, limite]`, o el error que lo impide.
fn arbol_de_era(
    entradas: &[Entrada],
    limite_anterior: u64,
    limite: u64,
    n: u64,
) -> Result<SparseTree, ErrorVista> {
    let mut arbol = SparseTree::new();
    let mut vistos = std::collections::BTreeSet::new();
    for &(rx, era, hash_prueba) in entradas {
        if !pertenece_a_era(rx, limite_anterior, limite) {
            continue;
        }
        if !vistos.insert(rx) {
            return Err(ErrorVista::RxRepetido { rx });
        }
        arbol.set_leaf(indice_de_recibo(rx, limite_anterior), hoja_de_recibo(hash_prueba, era, n));
    }
    Ok(arbol)
}

/// La raíz de recepción de la era `(limite_anterior, limite]`.
///
/// Toda recepción anotada dentro de la era tiene hoja, se aplicara o no: es
/// lo que distingue este árbol del de acuses (D-A). Una era sin entradas
/// tiene la raíz del árbol vacío, y **el vacío también se firma**.
pub fn raiz_de_era(
    entradas: &[Entrada],
    limite_anterior: u64,
    limite: u64,
    n: u64,
) -> Result<Digest, ErrorVista> {
    Ok(arbol_de_era(entradas, limite_anterior, limite, n)?.root())
}

/// La pareja que la cabeza **v6** firmará: `(recep_root, recep_count)`.
///
/// `recep_count` es `limite`: las recepciones que el nodo evaluó al componer,
/// que es el último `rx` reservado (§567). `limite_anterior` es el de la cabeza
/// anterior. `n` es el techo que la cabeza ya firma para los acuses, y sale
/// del MISMO productor: [`crate::vista_acuses::N_MAX_CABEZAS`].
pub fn pareja_de_ahora(
    entradas: &[Entrada],
    limite_anterior: u64,
    limite: u64,
) -> Result<(Digest, u64), ErrorVista> {
    let raiz = raiz_de_era(
        entradas,
        limite_anterior,
        limite,
        crate::vista_acuses::N_MAX_CABEZAS,
    )?;
    Ok((raiz, limite))
}

/// El camino de `rx` en el árbol de su era `(limite_anterior, limite]`:
/// `Ok(Some((raíz, hermanos, derecha)))`, `Ok(None)` si `rx` no pertenece a la
/// era o no está entre las entradas, o el error que impide componerla.
///
/// ⚠️ Devuelve la raíz para los TESTS y las compuertas; el RPC **no la
/// sirve** (§248): el titular la recompone y la compara con la cabeza que
/// custodia.
pub fn camino_de_era(
    entradas: &[Entrada],
    limite_anterior: u64,
    limite: u64,
    rx: u64,
    n: u64,
) -> Result<Option<(Digest, Vec<Digest>, Vec<bool>)>, ErrorVista> {
    let arbol = arbol_de_era(entradas, limite_anterior, limite, n)?;
    if !pertenece_a_era(rx, limite_anterior, limite)
        || !entradas.iter().any(|&(r, _, _)| r == rx)
    {
        return Ok(None);
    }
    let camino = arbol.path_for(indice_de_recibo(rx, limite_anterior));
    Ok(Some((arbol.root(), camino.siblings, camino.is_right)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vista_acuses::N_MAX_CABEZAS;
    use zk_ssl_verify::acuses::{as_digest, path_root};

    /// Entradas `rx` de `rango`, todas de la era `era`, con una prueba distinta cada una.
    fn entradas(rango: std::ops::RangeInclusive<u64>, era: u64) -> Vec<Entrada> {
        rango.map(|rx| (rx, era, as_digest(0x2000 + rx))).collect()
    }

    #[test]
    fn la_era_vacia_tiene_la_raiz_del_arbol_vacio() {
        let r = raiz_de_era(&[], 5, 9, N_MAX_CABEZAS).expect("vacia");
        assert_eq!(r, SparseTree::new().root(), "el vacio no es la raiz del arbol vacio");
    }

    #[test]
    fn lo_que_no_pertenece_no_entra_y_el_borde_es_q_exclusivo_r_inclusivo() {
        // (Q, R] con Q = 5 y R = 9: el 5 es de la era anterior y el 10 de la
        // siguiente; el 9 -el ultimo rx reservado al componer- SI entra (§567).
        let dentro = raiz_de_era(&entradas(6..=9, 7), 5, 9, N_MAX_CABEZAS).expect("dentro");
        let con_ruido = raiz_de_era(&entradas(3..=12, 7), 5, 9, N_MAX_CABEZAS).expect("ruido");
        assert_eq!(dentro, con_ruido, "algo de fuera de la era entro al arbol");
        let sin_el_9 = raiz_de_era(&entradas(6..=8, 7), 5, 9, N_MAX_CABEZAS).expect("sin 9");
        assert_ne!(dentro, sin_el_9, "el rx = R no entro: el borde sigue siendo [Q, R)");
    }

    #[test]
    fn el_orden_de_llegada_no_importa() {
        // La posicion la da el rx, no el orden en que el registro lo devuelve.
        let mut al_reves = entradas(6..=9, 7);
        al_reves.reverse();
        assert_eq!(
            raiz_de_era(&entradas(6..=9, 7), 5, 9, N_MAX_CABEZAS).expect("a"),
            raiz_de_era(&al_reves, 5, 9, N_MAX_CABEZAS).expect("b"),
        );
    }

    #[test]
    fn la_era_declarada_va_dentro_de_la_hoja() {
        // Mismo rx, misma prueba, otra era declarada: otra raiz. Por eso la
        // era se recibe como DATO y no se recalcula.
        let a = raiz_de_era(&entradas(6..=9, 7), 5, 9, N_MAX_CABEZAS).expect("a");
        let b = raiz_de_era(&entradas(6..=9, 8), 5, 9, N_MAX_CABEZAS).expect("b");
        assert_ne!(a, b, "la era no entra en la hoja");
    }

    #[test]
    fn n_distinto_raiz_distinta() {
        assert_ne!(
            raiz_de_era(&entradas(1..=3, 1), 0, 3, 1_440).expect("a"),
            raiz_de_era(&entradas(1..=3, 1), 0, 3, 720).expect("b"),
        );
    }

    #[test]
    fn un_rx_repetido_en_la_era_se_nombra_y_no_compone() {
        let mut rotas = entradas(6..=9, 7);
        rotas.push((8, 7, as_digest(0xDEAD)));
        assert_eq!(
            raiz_de_era(&rotas, 5, 9, N_MAX_CABEZAS),
            Err(ErrorVista::RxRepetido { rx: 8 })
        );
        // Un repetido FUERA de la era no es asunto de esta era.
        let mut fuera = entradas(6..=9, 7);
        fuera.push((3, 6, as_digest(1)));
        fuera.push((3, 6, as_digest(2)));
        assert!(raiz_de_era(&fuera, 5, 9, N_MAX_CABEZAS).is_ok());
    }

    #[test]
    fn la_pareja_de_ahora_firma_la_raiz_y_el_ultimo_rx() {
        let e = entradas(1..=4, 1);
        let (raiz, cuenta) = pareja_de_ahora(&e, 0, 4).expect("pareja");
        assert_eq!(cuenta, 4, "recep_count es el ultimo rx reservado");
        assert_eq!(raiz, raiz_de_era(&e, 0, 4, N_MAX_CABEZAS).expect("raiz"));
        // El genesis: sin recepciones, el arbol vacio y 0.
        assert_eq!(pareja_de_ahora(&[], 0, 0).expect("genesis"), (SparseTree::new().root(), 0));
    }

    #[test]
    fn el_camino_sube_hasta_la_raiz_de_su_era() {
        // La cadena entera con las reglas COMPARTIDAS: hoja -> path_root -> la
        // raiz que raiz_de_era da.
        let e = entradas(6..=9, 7);
        let (raiz, hermanos, derecha) = camino_de_era(&e, 5, 9, 9, N_MAX_CABEZAS)
            .expect("compone")
            .expect("camino");
        assert_eq!(raiz, raiz_de_era(&e, 5, 9, N_MAX_CABEZAS).expect("raiz"));
        let hoja = hoja_de_recibo(as_digest(0x2000 + 9), 7, N_MAX_CABEZAS);
        assert_eq!(path_root(hoja, &hermanos, &derecha), raiz, "el camino no sube");
    }

    #[test]
    fn sin_hoja_no_hay_camino() {
        // Fuera de (Q, R] -> None; dentro pero sin entrada -> None. Un camino
        // de una hoja vacia "verificaria" contra un arbol que no la contiene.
        let e = entradas(6..=9, 7);
        assert_eq!(camino_de_era(&e, 5, 9, 5, N_MAX_CABEZAS), Ok(None), "Q es de la anterior");
        assert_eq!(camino_de_era(&e, 5, 9, 10, N_MAX_CABEZAS), Ok(None), "R+1 es de la siguiente");
        let sin_el_7: Vec<_> = e.iter().copied().filter(|&(r, _, _)| r != 7).collect();
        assert_eq!(camino_de_era(&sin_el_7, 5, 9, 7, N_MAX_CABEZAS), Ok(None), "hoja ausente");
    }
}
