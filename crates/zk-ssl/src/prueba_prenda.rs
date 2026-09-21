//! **RFC-0008 E3: el PRENDADOR produce su media prenda.**
//!
//! El AIR vive en `zk-ssl-air::prenda` (S516) y el probador en `stark_experiment::circuit_prenda`
//! (S517). Faltaba quien la PRODUZCA con lo que el prendador tiene de verdad: su aviso
//! (`PendingNotice`: posicion, sal, importe y el sobre `X` opaco), **su clave de gasto** y el
//! camino que el nodo le sirva de la foto del ultimo latido. Por eso esto es una funcion LIBRE y
//! no un metodo del libro: no lee el libro, y se puede correr en un cliente sin el.
//!
//! **CON clave, y ahi se separa del hermano de E1.** El enunciado del cobro es de ESTADO (D-G) y
//! el pagador produce la misma prueba; el de la prenda es de AUTORIZACION (D-AV) y solo lo
//! produce quien tiene la clave. Eso no mete esta pieza en el libro: la pone del lado del
//! CLIENTE, que es donde `client::prove_send` ya vive. **La clave no se guarda**: entra por
//! argumento, compone la traza y se va con ella -la prueba la publica, 42 veces (§521)-.
//!
//! **El receptor no se declara: se DERIVA.** Quien llama no puede decir a nombre de quien
//! prenda, porque la identidad es funcion de la clave (`derive_public_id_wide`). Si la clave no
//! es la del pendiente, la hoja recompuesta no sube a la raiz y esto rehusa ANTES de gastar una
//! prueba.
//!
//! **Lo que sale es MEDIA PRENDA.** La prenda es el PAR: la marca bajo la raiz firmada MAS este
//! sobre (D-AS). Aqui se COMPONE la marca y **no se publica**: escribirla en el arbol de
//! consumos es otro corte, y sin ella en el arbol este sobre no afirma nada por si solo.
//!
//! **La puerta que hace falsable lo que sale.** No se afirma sobre <<un libro>>: se afirma sobre
//! la raiz de la cabeza v5 FIRMADA que se da. Antes de probar se recompone en nativo la hoja del
//! aviso y se SUBE por el camino, con los bits que salen de la posicion del aviso; si no llega a
//! la raiz de esa cabeza, RECHAZA y dice que puede ser. Y lo que sale ya esta verificado por el
//! mismo juez que corre el tercero.

use crate::prueba_cobro::bits_de;
use crate::two_phase::PendingNotice;
use crate::{Digest, LayerError};
use stark_experiment::circuit_prenda as prenda;
use stark_experiment::merkle::{native_merge, native_root, MerklePath};
use stark_experiment::native::derive_public_id_wide;
use winterfell::math::fields::f64::BaseElement;

/// **Lo que la cabeza v5 firmada declara y la prenda necesita**: una sola raiz, porque la prenda
/// no lleva la meta (D-AY). El `seq` no entra en el enunciado y viaja igual: sin el, un tercero
/// no sabe QUE cabeza pedir.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CabezaDeLaPrenda {
    pub seq: u64,
    pub pending_root: Digest,
}

/// **Lo que el prendador entrega**: la prueba, lo que el sobre declara y la marca que el par
/// necesita. Ni el importe, ni la sal, ni `X`, ni `C2`, ni la clave (D-AW).
#[derive(Clone, Debug)]
pub struct SobrePrenda {
    pub prueba: Vec<u8>,
    pub receptor: Digest,
    pub marca: Digest,
    pub seq: u64,
    pub pending_root: Digest,
}

fn falla(que: String) -> LayerError {
    LayerError::VerificationFailed(format!("prenda: {que}"))
}

/// **La prueba de que el pendiente del aviso esta bajo la cabeza a nombre de quien tiene esta
/// clave, y de que su marca es la que se publica** (RFC-0008 E3).
///
/// Falla cerrado y por su nombre en cada paso, y NUNCA con una variante nueva de `LayerError`:
/// una variante nueva rompe todo `match` exhaustivo de fuera.
pub fn prueba_de_prenda(
    cab: &CabezaDeLaPrenda,
    clave: Digest,
    aviso: &PendingNotice,
    camino: &MerklePath,
) -> Result<SobrePrenda, LayerError> {
    let x = aviso.x.ok_or_else(|| {
        falla("el aviso es v1 (sin sobre X): la prenda es del compromiso v2 (D-F)".to_string())
    })?;
    let niveles = prenda::PROFUNDIDAD;
    if camino.siblings.len() != niveles || camino.is_right.len() != niveles {
        return Err(falla(format!("el camino no tiene {niveles} niveles")));
    }
    if camino.is_right != bits_de(aviso.position, niveles) {
        return Err(falla(format!(
            "el camino servido no es el de la posicion {} del aviso",
            aviso.position
        )));
    }

    // La identidad es FUNCION de la clave; nadie la declara. La hoja que el prendador recompone
    // con su aviso, en nativo, y su marca.
    let receptor = derive_public_id_wide(clave);
    let importe = zk_ssl_hash::embeber(BaseElement::new(aviso.amount));
    let hoja = native_merge(native_merge(native_merge(receptor, aviso.salt), importe), x);
    if native_root(hoja, camino) != cab.pending_root {
        return Err(falla(
            "la hoja del aviso no sube a la raiz de pendientes de esa cabeza (otra clave, otro \
             aviso, o un camino de otra foto)"
                .to_string(),
        ));
    }
    let marca = zk_ssl_hash::marca_prenda(hoja);

    let w = prenda::PrendaWitness {
        clave,
        sal: aviso.salt,
        importe: aviso.amount,
        x,
        camino_pendiente: camino.clone(),
    };
    let (prueba, pi) =
        prenda::probar(prenda::trazar(&w)).map_err(|e| falla(format!("probar: {e}")))?;
    if pi.pending_root != cab.pending_root || pi.receptor != receptor || pi.marca != marca {
        return Err(falla(
            "el enunciado derivado no es el de esta cabeza, esta clave y esta marca".to_string(),
        ));
    }
    // El molde del S466: lo que sale ya esta verificado por la MISMA regla que corre el tercero,
    // la que lo ENLAZA a la cabeza (D-AV).
    let af = prenda::AfirmacionPrenda { receptor, marca };
    let cabeza = prenda::CabezaPrenda { pending_root: cab.pending_root };
    let enlazado = prenda::verificar_contra_cabeza(&prueba, &af, &cabeza)
        .map_err(|e| falla(format!("la prueba recien producida no se enlaza: {e}")))?;
    if enlazado != pi {
        return Err(falla("el enlace compuso otro enunciado que el probador".to_string()));
    }
    Ok(SobrePrenda { prueba, receptor, marca, seq: cab.seq, pending_root: cab.pending_root })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests_support as ts;
    use crate::{AccountIndex, SovereignLayer};
    use winterfell::math::FieldElement;

    const IMPORTE: u64 = 250_000;
    const FONDO: u64 = 1_000_000;

    fn key(sk: u64) -> Digest {
        [BaseElement::new(sk), BaseElement::ZERO, BaseElement::ZERO, BaseElement::ZERO]
    }

    /// Un envio v2 de Alice a Bob aplicado sobre un libro VIVO: el aviso de Bob, con el molde del
    /// S504. El `refund_id` es el `public_id` de Alice, como en los tres productores del arbol.
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
        assert_ne!(alice, bob);
        let aviso = envio(&mut l, alice, bob, 0xE3C0);
        let id_bob = l.public_id_of(bob).expect("bob");
        Libro { l, alice, bob, aviso, id_bob }
    }

    /// Lo que el latido guardaria: la cabeza y el camino de ESTE libro, AHORA. La prenda no pide
    /// meta, asi que la foto es un camino y nada mas (D-AY).
    fn foto_de(l: &SovereignLayer, pos: u64) -> (CabezaDeLaPrenda, MerklePath) {
        let cab = CabezaDeLaPrenda { seq: l.log.len() as u64, pending_root: l.pending.root() };
        (cab, l.pending.path_for(pos))
    }

    fn enunciado(s: &SobrePrenda) -> prenda::PrendaPublicInputs {
        prenda::PrendaPublicInputs {
            pending_root: s.pending_root,
            receptor: s.receptor,
            marca: s.marca,
        }
    }

    fn hoja_del(aviso: &PendingNotice, receptor: Digest) -> Digest {
        let importe = zk_ssl_hash::embeber(BaseElement::new(aviso.amount));
        let x = aviso.x.expect("el aviso es v2");
        native_merge(native_merge(native_merge(receptor, aviso.salt), importe), x)
    }

    fn rehusa(r: Result<SobrePrenda, LayerError>, texto: &str) {
        let e = r.expect_err("tenia que rehusar");
        assert!(format!("{e:?}").contains(texto), "se esperaba <<{texto}>>: {e:?}");
    }

    /// **El positivo, contra un libro vivo**: Bob prenda su pendiente con su clave, su aviso y el
    /// camino, y el juez del kit lo acepta con el enunciado que el sobre declara.
    #[test]
    fn el_prendador_prueba_su_pendiente_y_el_juez_lo_acepta() {
        let Libro { l, aviso, .. } = libro();
        let (cab, camino) = foto_de(&l, aviso.position);
        let s = prueba_de_prenda(&cab, key(ts::SK_BOB), &aviso, &camino).expect("el prendador");
        assert_eq!((s.seq, s.pending_root), (cab.seq, cab.pending_root));
        assert!(prenda::verificar(&s.prueba, &enunciado(&s)).is_ok(), "el juez no la acepta");
        let mut otra = enunciado(&s);
        otra.marca[0] += BaseElement::ONE;
        assert!(prenda::verificar(&s.prueba, &otra).is_err(), "la prueba vale para otra marca");
    }

    /// **La marca del sobre es la NATIVA de la hoja del aviso** (D-AW): se compone, no se copia,
    /// y quien tenga el aviso la precomputa sin la clave y sin el probador.
    #[test]
    fn la_marca_del_sobre_es_la_nativa_de_la_hoja_del_aviso() {
        let Libro { l, aviso, id_bob, .. } = libro();
        let (cab, camino) = foto_de(&l, aviso.position);
        let s = prueba_de_prenda(&cab, key(ts::SK_BOB), &aviso, &camino).expect("el prendador");
        let hoja = hoja_del(&aviso, id_bob);
        assert_eq!(s.marca, zk_ssl_hash::marca_prenda(hoja), "la marca no es la de esta hoja");
        assert_ne!(s.marca, hoja, "la marca no es la hoja: lleva su dominio");
    }

    /// **La clave de otro no sube a la cabeza, y rehusa ANTES de probar.** El receptor no se
    /// declara: se deriva, asi que una clave ajena recompone OTRA hoja, que no esta en el arbol.
    #[test]
    fn la_clave_de_otro_no_sube_a_la_cabeza() {
        let Libro { l, aviso, .. } = libro();
        let (cab, camino) = foto_de(&l, aviso.position);
        rehusa(
            prueba_de_prenda(&cab, key(ts::SK_ALICE), &aviso, &camino),
            "no sube a la raiz de pendientes",
        );
    }

    #[test]
    fn un_aviso_v1_no_tiene_prenda() {
        let Libro { l, mut aviso, .. } = libro();
        let (cab, camino) = foto_de(&l, aviso.position);
        aviso.x = None;
        rehusa(prueba_de_prenda(&cab, key(ts::SK_BOB), &aviso, &camino), "el aviso es v1");
    }

    /// Un camino de otra posicion se rechaza por sus bits, antes de subir nada.
    #[test]
    fn un_camino_de_otra_posicion_se_rechaza() {
        let Libro { mut l, alice, bob, aviso, .. } = libro();
        let otro = envio(&mut l, alice, bob, 0xE3C2);
        let (cab, camino_otro) = foto_de(&l, otro.position);
        rehusa(
            prueba_de_prenda(&cab, key(ts::SK_BOB), &aviso, &camino_otro),
            "no es el de la posicion",
        );
    }

    /// **La foto**: tras otro pago el arbol vivo se mueve; con la cabeza y el camino de antes la
    /// prenda sale, y con la cabeza de despues y el camino de antes, rehusa.
    #[test]
    fn un_pago_entre_la_foto_y_la_prenda_no_la_invalida_pero_no_se_mezclan() {
        let Libro { mut l, alice, bob, aviso, .. } = libro();
        let (cab, camino) = foto_de(&l, aviso.position);
        let _otro = envio(&mut l, alice, bob, 0xE3C1);
        assert_ne!(l.pending.root(), cab.pending_root, "el libro tenia que moverse");
        assert!(prueba_de_prenda(&cab, key(ts::SK_BOB), &aviso, &camino).is_ok());
        let (cab_nueva, _) = foto_de(&l, aviso.position);
        rehusa(
            prueba_de_prenda(&cab_nueva, key(ts::SK_BOB), &aviso, &camino),
            "no sube a la raiz de pendientes",
        );
    }

    /// **El enlace del tercero**: el sobre se enlaza con SU cabeza y no con otra.
    #[test]
    fn el_sobre_se_enlaza_con_su_cabeza_y_no_con_otra() {
        let Libro { l, aviso, .. } = libro();
        let (cab, camino) = foto_de(&l, aviso.position);
        let s = prueba_de_prenda(&cab, key(ts::SK_BOB), &aviso, &camino).expect("el prendador");
        let af = prenda::AfirmacionPrenda { receptor: s.receptor, marca: s.marca };
        let buena = prenda::CabezaPrenda { pending_root: s.pending_root };
        assert_eq!(
            prenda::verificar_contra_cabeza(&s.prueba, &af, &buena),
            Ok(enunciado(&s)),
            "no se enlaza con la suya"
        );
        let mut otra = s.pending_root;
        otra[0] += BaseElement::ONE;
        let mala = prenda::CabezaPrenda { pending_root: otra };
        assert!(
            prenda::verificar_contra_cabeza(&s.prueba, &af, &mala).is_err(),
            "se enlazo con otra raiz"
        );
    }

    /// **El receptor del sobre es el `public_id` que la capa tiene para esa cuenta**: ata la
    /// derivacion del productor con la que el libro uso al abrirla.
    #[test]
    fn el_receptor_del_sobre_es_el_public_id_de_la_cuenta() {
        let Libro { l, aviso, id_bob, .. } = libro();
        let (cab, camino) = foto_de(&l, aviso.position);
        let s = prueba_de_prenda(&cab, key(ts::SK_BOB), &aviso, &camino).expect("el prendador");
        assert_eq!(s.receptor, id_bob, "el receptor derivado no es el de la cuenta");
        assert_eq!(s.receptor, derive_public_id_wide(key(ts::SK_BOB)));
    }

    /// Dos listas son dos productores: las opciones del probador y las de la capa se atan aqui.
    #[test]
    fn las_opciones_del_probador_son_las_de_la_capa() {
        assert_eq!(prenda::opciones(), crate::proof_options());
    }
}
