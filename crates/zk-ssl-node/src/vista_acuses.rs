//! # La vista de acuses: el árbol de una época, computado del registro
//!
//! **No es una estructura: es una vista.** Las hojas se derivan del
//! registro —que el snapshot persiste entero y revalida al abrir— y los
//! límites de época los da el diario del nodo (§272). Nada nuevo se
//! persiste; el árbol se computa al pedirlo y se tira.
//!
//! Las reglas de pertenencia, posición, época y hoja **no viven aquí**:
//! viven en `zk_ssl_verify::acuses`, porque el verificador de §275 usará
//! exactamente las mismas.

use zk_ssl::sparse_tree::SparseTree;
use zk_ssl_verify::acuses::{hoja_de_acuse, indice_de_hoja, pertenece, Digest};

/// **N, declarado** — el techo de la promesa: *«no inclusión en N épocas
/// es censura»* (§121).
///
/// 1.440 cabezas = 24 h al latido de §115, con el precedente del MMD de
/// Certificate Transparency. A 70 ms por apply, 24 h absorben más de
/// 1,2 M de operaciones: a esa escala, *«era congestión»* deja de ser
/// defensa.
///
/// **Normativo desde §275**: viaja **firmado en la cabeza** —uno por
/// época, no uno por titular— y la serie queda auditable en los diarios
/// del nodo y del testigo. Un `n` mentido en una respuesta ya es legible:
/// la hoja no aparece bajo la raíz recomputada con el `n` de la cabeza
/// **firmada**.
pub const N_MAX_CABEZAS: u64 = 1_440;

/// La raíz del árbol de acuses de la época `[limite_anterior, limite)`.
///
/// Recibe pares `(seq, proof_digest)` —el llamante los saca de
/// `transition_log().entries()`— y aplica las reglas compartidas. **Toda
/// entrada tiene hoja**, delegadas incluidas: la suya sale del
/// `proof_digest` compartido (§271), y el día que la nota 78 les dé
/// pruebas reales sus acuses cobran sentido sin cambio de formato.
pub fn raiz_de_epoca(
    entradas: impl IntoIterator<Item = (u64, Digest)>,
    limite_anterior: u64,
    limite: u64,
    n: u64,
) -> Digest {
    let mut arbol = SparseTree::new();
    for (seq, hash_prueba) in entradas {
        if pertenece(seq, limite_anterior, limite) {
            arbol.set_leaf(
                indice_de_hoja(seq, limite_anterior),
                hoja_de_acuse(hash_prueba, seq, n),
            );
        }
    }
    arbol.root()
}

/// El **ÚNICO** mapeo del registro a pares `(seq, proof_digest)` (§275).
///
/// El test del latido «la cabeza del latido es la que sirve el RPC» hace
/// de compuerta: si `latir` y los arms del RPC mapearan cada uno a su
/// manera, sus cabezas divergirían y ese test lo diría.
pub fn pares(entradas: &[zk_ssl::log::LogEntry]) -> Vec<(u64, Digest)> {
    entradas.iter().map(|e| (e.seq, e.proof_digest)).collect()
}

/// La pareja que la cabeza **v2** firma: `(raíz de la época EN CURSO, N)`.
///
/// El límite superior de la época en curso es `len` — el `seq` que la
/// cabeza que se está componiendo va a llevar. `n` es [`N_MAX_CABEZAS`]:
/// el techo declarado, ahora **firmado**, uno por época.
pub fn pareja_de_ahora(entradas: &[(u64, Digest)], limite_anterior: u64) -> (Digest, u64) {
    let limite = entradas.len() as u64;
    (
        raiz_de_epoca(entradas.iter().copied(), limite_anterior, limite, N_MAX_CABEZAS),
        N_MAX_CABEZAS,
    )
}

/// `(P, S)` de la época **cerrada** que contiene `seq`, o `None` si esa
/// época sigue abierta. `limites` = los `seq` del diario, ascendentes.
///
/// `S` = primer límite `> seq`; `P` = último límite `<= seq`, **o 0**:
/// el borde documentado arriba en `acuses` — con `P` exclusivo la
/// entrada 0 no pertenecería a ninguna época.
pub fn limites_para(limites: &[u64], seq: u64) -> Option<(u64, u64)> {
    let s = limites.iter().copied().find(|&l| l > seq)?;
    let p = limites.iter().copied().filter(|&l| l <= seq).last().unwrap_or(0);
    Some((p, s))
}

/// El camino de `seq` en el árbol de su época cerrada `[P, S)`:
/// `Some((raíz, hermanos, derecha))`, o `None` si `seq` no pertenece o
/// su hoja no está entre los pares servidos.
///
/// ⚠️ Devuelve la raíz para los TESTS y las compuertas; el RPC **no la
/// sirve** (§248): el titular la recompone y la compara con la cabeza
/// que custodia.
pub fn camino_de_epoca(
    pares: &[(u64, Digest)],
    limite_anterior: u64,
    limite: u64,
    seq: u64,
    n: u64,
) -> Option<(Digest, Vec<Digest>, Vec<bool>)> {
    if !pertenece(seq, limite_anterior, limite) {
        return None;
    }
    let mut arbol = SparseTree::new();
    let mut vista = false;
    for (s, hash_prueba) in pares.iter().copied() {
        if pertenece(s, limite_anterior, limite) {
            arbol.set_leaf(
                indice_de_hoja(s, limite_anterior),
                hoja_de_acuse(hash_prueba, s, n),
            );
            if s == seq {
                vista = true;
            }
        }
    }
    if !vista {
        return None;
    }
    let camino = arbol.path_for(indice_de_hoja(seq, limite_anterior));
    Some((arbol.root(), camino.siblings, camino.is_right))
}

#[cfg(test)]
mod tests {
    use super::*;
    use zk_ssl_verify::acuses::as_digest;

    fn entradas(rango: std::ops::Range<u64>) -> Vec<(u64, Digest)> {
        rango.map(|s| (s, as_digest(0x1000 + s))).collect()
    }

    #[test]
    fn la_epoca_vacia_tiene_raiz_estable() {
        // La raiz del arbol vacio es un valor fijo: dos computos, iguales.
        let a = raiz_de_epoca(Vec::new(), 0, 5, N_MAX_CABEZAS);
        let b = raiz_de_epoca(Vec::new(), 0, 5, N_MAX_CABEZAS);
        assert_eq!(a, b);
        assert_eq!(a, SparseTree::new().root(), "no es la raiz del arbol vacio");
    }

    #[test]
    fn una_hoja_mueve_la_raiz() {
        let vacia = raiz_de_epoca(Vec::new(), 0, 5, N_MAX_CABEZAS);
        let una = raiz_de_epoca(vec![(2, as_digest(7))], 0, 5, N_MAX_CABEZAS);
        assert_ne!(vacia, una);
    }

    #[test]
    fn lo_que_no_pertenece_no_entra() {
        // Entradas de otras epocas no mueven esta raiz: la de la cabeza
        // (seq = limite) es de la SIGUIENTE.
        let dentro = raiz_de_epoca(entradas(5..9), 5, 9, N_MAX_CABEZAS);
        let con_ruido = raiz_de_epoca(
            entradas(3..12), // 3,4 son de antes; 9,10,11 de despues
            5, 9, N_MAX_CABEZAS,
        );
        assert_eq!(dentro, con_ruido, "algo de fuera de la epoca entro al arbol");
    }

    #[test]
    fn el_orden_de_insercion_no_importa() {
        // La posicion la da el seq, no la llegada: mismo conjunto, misma
        // raiz, en cualquier orden.
        let mut al_reves = entradas(5..9);
        al_reves.reverse();
        assert_eq!(
            raiz_de_epoca(entradas(5..9), 5, 9, N_MAX_CABEZAS),
            raiz_de_epoca(al_reves, 5, 9, N_MAX_CABEZAS),
        );
    }

    #[test]
    fn n_distinto_raiz_distinta() {
        // N va dentro de cada hoja (§270): cambiar el techo cambia el arbol
        // entero. Por eso un n mentido es visible cuando n viaje en la
        // cabeza (§275).
        assert_ne!(
            raiz_de_epoca(entradas(0..3), 0, 3, 1_440),
            raiz_de_epoca(entradas(0..3), 0, 3, 720),
        );
    }

    #[test]
    fn las_delegadas_entran_con_su_prueba_compartida() {
        // Dos delegadas comparten proof_digest (§271) pero no hoja: la
        // epoca de cada una va dentro. Toda entrada tiene hoja.
        let compartido = as_digest(0x74DE);
        let r = raiz_de_epoca(vec![(0, compartido), (1, compartido)], 0, 3, N_MAX_CABEZAS);
        let solo_una = raiz_de_epoca(vec![(0, compartido)], 0, 3, N_MAX_CABEZAS);
        assert_ne!(r, solo_una, "la segunda delegada no dejo hoja");
    }
}

#[cfg(test)]
mod tests_limites_y_camino {
    use super::*;
    use zk_ssl_verify::acuses::{as_digest, path_root};

    fn entradas(rango: std::ops::Range<u64>) -> Vec<(u64, Digest)> {
        rango.map(|s| (s, as_digest(0x1000 + s))).collect()
    }

    #[test]
    fn limites_para_encuentra_la_epoca_cerrada() {
        // S = primer limite > seq; P = ultimo limite <= seq.
        assert_eq!(limites_para(&[3, 7, 12], 8), Some((7, 12)));
        assert_eq!(limites_para(&[3, 7, 12], 3), Some((3, 7)), "P pertenece (P <= seq)");
    }

    #[test]
    fn la_primera_epoca_arranca_en_cero() {
        // El borde de §274, ahora en los limites: sin cabeza anterior,
        // P = 0 y la entrada 0 tiene epoca.
        assert_eq!(limites_para(&[5, 9], 2), Some((0, 5)));
    }

    #[test]
    fn la_epoca_abierta_no_tiene_limites() {
        // Sin cabeza que la cierre no hay S: el RPC respondera available
        // false con reason, no un camino a medias.
        assert_eq!(limites_para(&[5], 7), None);
        assert_eq!(limites_para(&[], 0), None);
    }

    #[test]
    fn el_camino_sube_hasta_la_raiz_de_su_epoca() {
        // La cadena entera con las reglas COMPARTIDAS: hoja -> path_root
        // (re-exportado por acuses) -> la raiz que raiz_de_epoca da.
        let pares = entradas(5..9);
        let (raiz, hermanos, derecha) =
            camino_de_epoca(&pares, 5, 9, 6, N_MAX_CABEZAS).expect("camino");
        assert_eq!(raiz, raiz_de_epoca(pares.iter().copied(), 5, 9, N_MAX_CABEZAS));
        let hoja = hoja_de_acuse(as_digest(0x1000 + 6), 6, N_MAX_CABEZAS);
        assert_eq!(path_root(hoja, &hermanos, &derecha), raiz, "el camino no sube");
    }

    #[test]
    fn sin_hoja_vista_no_hay_camino() {
        // Guardas: fuera de [P, S) -> None; dentro pero sin entrada
        // servida con ese seq -> None. Un camino de una hoja vacia
        // "verificaria" contra un arbol que no la contiene.
        let pares = entradas(5..9);
        assert_eq!(camino_de_epoca(&pares, 5, 9, 9, N_MAX_CABEZAS), None, "S es de la siguiente");
        let sin_el_7: Vec<_> = pares.iter().copied().filter(|(s, _)| *s != 7).collect();
        assert_eq!(camino_de_epoca(&sin_el_7, 5, 9, 7, N_MAX_CABEZAS), None, "hoja ausente");
    }
}
