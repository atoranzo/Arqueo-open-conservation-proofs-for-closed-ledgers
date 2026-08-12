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
/// ⚠️ **Declarado no es todavía normativo.** Lo será cuando viaje
/// **firmado en la cabeza** (§275) — uno por época, no uno por titular, y
/// con la serie auditable en los diarios. Hasta entonces, un `n` mentido
/// en la respuesta es ilegible: la hoja simplemente no aparecerá bajo la
/// raíz recomputada con el `n` de la cabeza, y el titular no puede
/// distinguir mentira de censura. **Tercera razón escrita de que raíz y
/// `N` van juntos en §275.**
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
