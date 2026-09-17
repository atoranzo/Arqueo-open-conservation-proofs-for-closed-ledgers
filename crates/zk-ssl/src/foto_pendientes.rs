//! **RFC-0008 D-F: la FOTO de los pendientes que toma el latido.**
//!
//! El cobrador prueba su pendiente contra una cabeza v5 FIRMADA, y esa cabeza es la del ultimo
//! latido. El arbol de pendientes se mueve con cada pago, asi que un camino del estado de AHORA
//! deja de subir a la cabeza en cuanto un pago cae entre dos latidos. La foto es la copia de los
//! dos arboles -pendientes y meta- y del mapa de meta, tomada en la MISMA seccion critica en la que
//! el latido compone la cabeza; el nodo la guarda con la cabeza y sirve de ella.
//!
//! **Que se copia, y por que el mapa tambien** (D-F2): la hoja de meta es un hash de
//! `(emisor, nacido)` y no se lee del arbol; y las posiciones se reutilizan (`allocate_pending`
//! da el primer hueco libre), asi que la meta del mapa VIVO puede ser la de otro pendiente. Lo que
//! se sirve es el estado del ultimo latido, entero.
//!
//! **A quien se sirve** (la regla de D-F): a quien presenta un aviso cuya hoja, recompuesta con la
//! identidad publica del receptor, es la hoja de esa posicion en la foto. A cualquier otro, nada, y
//! sin decir que hay: una posicion libre, un aviso ajeno y un aviso v1 dan la misma respuesta.
//!
//! **Lo que la foto NO es.** No es historico: hay una por latido, en memoria, y tras un reinicio no
//! hay foto hasta el primer latido. La igualdad se juzga por las dos raices: dos fotos con las
//! mismas raices son el mismo par de arboles salvo colision del hash.

use std::collections::HashMap;

use crate::pending::pending_commitment;
use crate::prueba_cobro::FotoDelCobro;
use crate::sparse_tree::SparseTree;
use crate::two_phase::PendingNotice;
use crate::{Digest, SovereignLayer};
use stark_experiment::merkle::native_merge;

/// **La foto del latido**: los dos arboles de los pendientes y la meta de cada posicion ocupada.
#[derive(Clone, Debug)]
pub struct FotoPendientes {
    pendientes: SparseTree,
    meta: HashMap<u64, (u64, u64)>,
    arbol_meta: SparseTree,
}

impl PartialEq for FotoPendientes {
    fn eq(&self, otra: &Self) -> bool {
        self.raiz_pendientes() == otra.raiz_pendientes() && self.raiz_meta() == otra.raiz_meta()
    }
}

impl Eq for FotoPendientes {}

impl FotoPendientes {
    /// La raiz que la cabeza del mismo latido declara como `pendingRoot`.
    pub fn raiz_pendientes(&self) -> Digest {
        self.pendientes.root()
    }

    /// La raiz que la cabeza del mismo latido declara como `pmetaRoot`.
    pub fn raiz_meta(&self) -> Digest {
        self.arbol_meta.root()
    }

    /// **Lo que el cobrador recibe de la foto**, o nada.
    ///
    /// Recompone `C2 = M(H(H(receptor, sal), importe), X)` con el aviso y la compara con la hoja
    /// de la posicion del aviso. Un aviso sin `X` (v1) no tiene `C2`: E1 es del compromiso v2.
    pub fn cobro(&self, receptor: Digest, aviso: &PendingNotice) -> Option<FotoDelCobro> {
        let x = aviso.x?;
        let p = aviso.position;
        let hoja = native_merge(pending_commitment(receptor, aviso.salt, aviso.amount), x);
        if self.pendientes.leaf(p) != hoja {
            return None;
        }
        let (emisor, nacido) = *self.meta.get(&p)?;
        Some(FotoDelCobro {
            camino_pendiente: self.pendientes.path_for(p),
            hermanos_meta: self.arbol_meta.path_for(p).siblings,
            emisor,
            nacido,
        })
    }
}

impl SovereignLayer {
    /// **La foto, tomada ahora.** Quien la llama decide el instante: el latido la toma bajo el
    /// mismo candado con el que compone la cabeza (D-F).
    pub fn foto_pendientes(&self) -> FotoPendientes {
        FotoPendientes {
            pendientes: self.pending.clone(),
            meta: self.pending_meta.clone(),
            arbol_meta: self.pending_meta_tree.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prueba_cobro::{prueba_de_cobro_pendiente, CabezaDePendientes};
    use crate::tests_support as ts;
    use crate::AccountIndex;
    use stark_experiment::merkle::native_root;
    use winterfell::math::fields::f64::BaseElement;
    use winterfell::math::FieldElement;

    const IMPORTE: u64 = 250_000;
    const FONDO: u64 = 1_000_000;

    fn key(sk: u64) -> [BaseElement; 4] {
        [BaseElement::new(sk), BaseElement::ZERO, BaseElement::ZERO, BaseElement::ZERO]
    }

    /// Un envio v2 de Alice a Bob sobre un libro vivo, con el molde de `prueba_cobro`: el aviso
    /// de Bob.
    fn envio(
        layer: &mut SovereignLayer,
        alice: AccountIndex,
        bob: AccountIndex,
        semilla: u64,
    ) -> PendingNotice {
        let id_bob = layer.public_id_of(bob).expect("bob");
        let f = layer.public_id_of(alice).expect("alice");
        let m = layer
            .send_materials_v2(alice, id_bob, IMPORTE, ts::salt_de(semilla), f, 96)
            .expect("materiales v2");
        let recibo = crate::client::prove_send(&m, key(ts::SK_ALICE), crate::proof_options())
            .expect("probar el envio");
        let ea = ts::state_of(layer, alice);
        layer.apply_send(&recibo, alice, &ea, IMPORTE).expect("aplicar el envio");
        recibo.notice
    }

    struct Libro {
        l: SovereignLayer,
        alice: AccountIndex,
        bob: AccountIndex,
        aviso: PendingNotice,
        id_bob: Digest,
    }

    fn libro() -> Libro {
        let mut l = ts::new_layer();
        let alice = ts::open_and_fund(&mut l, ts::SK_ALICE, FONDO);
        let bob = ts::open_and_fund(&mut l, ts::SK_BOB, 0);
        let aviso = envio(&mut l, alice, bob, 0xF070);
        let id_bob = l.public_id_of(bob).expect("bob");
        Libro { l, alice, bob, aviso, id_bob }
    }

    /// La cabeza que el latido compondria con esta foto.
    fn cabeza(l: &SovereignLayer, foto: &FotoPendientes) -> CabezaDePendientes {
        CabezaDePendientes {
            seq: l.log.len() as u64,
            pending_root: foto.raiz_pendientes(),
            pmeta_root: foto.raiz_meta(),
        }
    }

    /// **El positivo**: la foto sirve el cobro de Bob y el productor del cobrador prueba con ello.
    ///
    /// Con DOS pendientes, y no con uno: con una sola hoja, los hermanos de los dos arboles son los
    /// digests de los subarboles vacios, IGUALES, y servir los de un arbol por los del otro no se
    /// ve. La r1 del PASTE-492-PRE lo cazo: su F2 no discriminaba con un pendiente. La ultima
    /// asercion es la prueba de vida del escenario.
    #[test]
    fn la_foto_sirve_el_cobro_y_el_cobrador_prueba_con_ella() {
        let Libro { mut l, alice, bob, aviso, id_bob } = libro();
        let _otro = envio(&mut l, alice, bob, 0xF073);
        let foto = l.foto_pendientes();
        assert_eq!(foto.raiz_pendientes(), l.pending.root());
        assert_eq!(foto.raiz_meta(), l.pending_meta_tree.root());
        let servido = foto.cobro(id_bob, &aviso).expect("la foto sirve el cobro de Bob");
        assert_eq!(Some((servido.emisor, servido.nacido)), l.pending_meta_of(aviso.position));
        let cab = cabeza(&l, &foto);
        prueba_de_cobro_pendiente(&cab, id_bob, &aviso, &servido, 1)
            .expect("el cobrador prueba con lo que la foto sirve");
        assert_ne!(
            servido.hermanos_meta, servido.camino_pendiente.siblings,
            "el escenario tiene que distinguir los dos arboles"
        );
    }

    /// **La razon de la foto**: un pago posterior mueve el libro y no mueve lo que la foto sirve.
    #[test]
    fn un_pago_despues_de_la_foto_no_mueve_lo_que_la_foto_sirve() {
        let Libro { mut l, alice, bob, aviso, id_bob } = libro();
        let foto = l.foto_pendientes();
        let raiz = foto.raiz_pendientes();
        let _otro = envio(&mut l, alice, bob, 0xF071);
        assert_ne!(l.pending.root(), raiz, "el libro tenia que moverse");
        assert_eq!(foto.raiz_pendientes(), raiz, "la foto no se mueve con el libro");
        let servido = foto.cobro(id_bob, &aviso).expect("la foto sigue sirviendo");
        let hoja = foto.pendientes.leaf(aviso.position);
        assert_eq!(native_root(hoja, &servido.camino_pendiente), raiz);
        assert_ne!(native_root(hoja, &servido.camino_pendiente), l.pending.root());
    }

    /// **La regla de D-F**: un aviso que no recompone la hoja no recibe nada.
    #[test]
    fn un_aviso_que_no_recompone_la_hoja_no_recibe_nada() {
        let Libro { l, alice, aviso, id_bob, .. } = libro();
        let foto = l.foto_pendientes();
        let id_alice = l.public_id_of(alice).expect("alice");
        assert!(foto.cobro(id_alice, &aviso).is_none(), "otro receptor");
        let mut a = aviso.clone();
        a.amount += 1;
        assert!(foto.cobro(id_bob, &a).is_none(), "otro importe");
        let mut a = aviso.clone();
        a.salt = ts::salt_de(0xBAD);
        assert!(foto.cobro(id_bob, &a).is_none(), "otra sal");
        let mut a = aviso.clone();
        a.x = Some(ts::salt_de(0xBAD));
        assert!(foto.cobro(id_bob, &a).is_none(), "otro sobre");
    }

    #[test]
    fn un_aviso_v1_no_recibe_nada() {
        let Libro { l, mut aviso, id_bob, .. } = libro();
        let foto = l.foto_pendientes();
        aviso.x = None;
        assert!(foto.cobro(id_bob, &aviso).is_none());
    }

    #[test]
    fn una_posicion_libre_no_recibe_nada() {
        let Libro { l, mut aviso, id_bob, .. } = libro();
        let foto = l.foto_pendientes();
        aviso.position += 1;
        assert!(foto.cobro(id_bob, &aviso).is_none());
    }

    /// **Por que la foto lleva el mapa** (D-F2): el cobro borra la meta VIVA de esa posicion, y la
    /// foto sigue sirviendo la del estado en que se tomo.
    #[test]
    fn el_cobro_despues_de_la_foto_no_borra_lo_que_la_foto_sirve() {
        let Libro { mut l, bob, aviso, id_bob, .. } = libro();
        let foto = l.foto_pendientes();
        let meta = l.pending_meta_of(aviso.position).expect("meta");
        let eb = ts::state_of(&l, bob);
        let cobro = l.claim(BaseElement::new(ts::SK_BOB), bob, &eb, &aviso).expect("claim");
        l.apply_claim(&cobro, bob, &eb, &aviso).expect("apply claim");
        assert!(l.pending_meta_of(aviso.position).is_none(), "el cobro borra la meta viva");
        let servido = foto.cobro(id_bob, &aviso).expect("la foto sirve su estado");
        assert_eq!((servido.emisor, servido.nacido), meta);
    }

    /// La igualdad es por las dos raices: el mismo estado da la misma foto, y un pago la distingue.
    #[test]
    fn dos_fotos_del_mismo_estado_son_iguales_y_un_pago_las_distingue() {
        let Libro { mut l, alice, bob, .. } = libro();
        let a = l.foto_pendientes();
        assert_eq!(a, l.foto_pendientes());
        let _otro = envio(&mut l, alice, bob, 0xF072);
        assert_ne!(a, l.foto_pendientes());
    }
}
