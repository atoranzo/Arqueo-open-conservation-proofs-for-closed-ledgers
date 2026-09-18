//! **RFC-0008 E1: el COBRADOR produce la prueba de su cobro pendiente.**
//!
//! El AIR vive en `zk-ssl-air::cobro_pendiente` y el probador en
//! `stark_experiment::circuit_cobro_pendiente` (S490). Faltaba quien la PRODUZCA con lo que el
//! cobrador tiene de verdad, y a diferencia de la banda y de la edad ese alguien NO es el
//! operador: es el receptor, con su aviso (`PendingNotice`: posicion, sal, importe y el sobre `X`
//! opaco), su identidad publica y lo que el nodo le sirva de la foto del ultimo latido -los dos
//! caminos de ESA posicion y su meta `(emisor, nacido)`- (D-F). Por eso esto es una funcion
//! LIBRE y no un metodo del libro: no lee el libro, y se puede correr en un cliente sin el.
//!
//! **Sin clave.** El enunciado es de ESTADO (D-G): el pagador, que conoce la apertura, produce la
//! misma prueba. Nada de lo que entra aqui es secreto del receptor; lo que la prueba no dice va en
//! la cabecera del AIR y no se repite.
//!
//! **La puerta que hace falsable lo que sale.** El productor no afirma sobre <<un libro>>: afirma
//! sobre las dos raices de la cabeza v5 FIRMADA que se le dan. Antes de probar recompone en
//! nativo la hoja del aviso y la de la meta y las SUBE por los caminos, con los bits que salen de
//! la posicion del aviso; si algo no llega a las raices de esa cabeza, RECHAZA y dice que. Y lo que
//! sale ya esta verificado por el mismo juez que corre el tercero.
//!
//! **La cota superior no se pide.** El sobre dice <<al menos `inferior`>>, asi que el techo es el
//! del campo (`MAX_VALOR`) y no un argumento: no hay forma de pedir una banda que diga mas de lo
//! que el RFC promete. `X` no sale en el sobre (D-I).

use crate::two_phase::PendingNotice;
use crate::{Digest, LayerError};
use stark_experiment::circuit_cobro_pendiente as cobro;
use stark_experiment::merkle::{native_merge, native_root, MerklePath};
use winterfell::math::fields::f64::BaseElement;

/// **Lo que la cabeza v5 firmada declara de los dos arboles**: lo que la prueba necesita y lo que
/// el sobre tiene que llevar para que un tercero la enlace a la firma.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CabezaDePendientes {
    pub seq: u64,
    pub pending_root: Digest,
    pub pmeta_root: Digest,
}

/// **Lo que el nodo sirve de su foto** a quien presenta un aviso que recompone la hoja (D-F): el
/// camino de pendientes de esa posicion, los hermanos del camino de meta -los bits son los mismos,
/// porque la posicion es la misma (D-H)- y la meta que vive ahi.
#[derive(Clone, Debug)]
pub struct FotoDelCobro {
    pub camino_pendiente: MerklePath,
    pub hermanos_meta: Vec<Digest>,
    pub emisor: u64,
    pub nacido: u64,
}

/// **Lo que el productor entrega**: la prueba y lo que el sobre declara. Ni el importe, ni la sal,
/// ni `X`, ni el emisor (D-B, D-I).
#[derive(Clone, Debug)]
pub struct SobreCobro {
    pub prueba: Vec<u8>,
    pub receptor: Digest,
    pub nacido: u64,
    pub inferior: u64,
    pub seq: u64,
    pub pending_root: Digest,
    pub pmeta_root: Digest,
}

fn falla(que: String) -> LayerError {
    LayerError::VerificationFailed(format!("prueba de cobro: {que}"))
}

/// Los bits de una posicion, en el orden en que `SparseTree::path_for` los escribe. Es
/// `pub(crate)` desde el §504: el productor del pago hace la MISMA comprobacion, y dos
/// listas de bits serian dos productores.
pub(crate) fn bits_de(posicion: u64, niveles: usize) -> Vec<bool> {
    (0..niveles).map(|n| n < 64 && (posicion >> n) & 1 == 1).collect()
}

/// **La prueba de que el pendiente del aviso esta bajo la cabeza, a nombre de `receptor`, por al
/// menos `inferior`** (RFC-0008 E1).
///
/// Falla cerrado y por su nombre en cada paso, y NUNCA con una variante nueva de `LayerError`:
/// una variante nueva rompe todo `match` exhaustivo de fuera.
pub fn prueba_de_cobro_pendiente(
    cab: &CabezaDePendientes,
    receptor: Digest,
    aviso: &PendingNotice,
    foto: &FotoDelCobro,
    inferior: u64,
) -> Result<SobreCobro, LayerError> {
    let x = aviso.x.ok_or_else(|| {
        falla("el aviso es v1 (sin sobre X): E1 es del compromiso v2 (D-F)".to_string())
    })?;
    let techo = cobro::MAX_VALOR;
    if aviso.amount > techo {
        return Err(falla(format!(
            "el importe {} pasa el techo del campo ({techo}): su banda no es probable",
            aviso.amount
        )));
    }
    if inferior > aviso.amount {
        return Err(falla(format!(
            "la banda NO se sostiene: el aviso lleva {} y se pide al menos {inferior}",
            aviso.amount
        )));
    }
    let niveles = cobro::PROFUNDIDAD;
    let cp = &foto.camino_pendiente;
    if cp.siblings.len() != niveles || cp.is_right.len() != niveles
        || foto.hermanos_meta.len() != niveles
    {
        return Err(falla(format!("los caminos no tienen {niveles} niveles")));
    }
    if cp.is_right != bits_de(aviso.position, niveles) {
        return Err(falla(format!(
            "el camino servido no es el de la posicion {} del aviso",
            aviso.position
        )));
    }

    // La hoja que el cobrador recompone con su aviso, en nativo, y la meta de esa posicion.
    let importe = zk_ssl_hash::embeber(BaseElement::new(aviso.amount));
    let hoja = native_merge(native_merge(native_merge(receptor, aviso.salt), importe), x);
    if native_root(hoja, cp) != cab.pending_root {
        return Err(falla(
            "la hoja del aviso no sube a la raiz de pendientes de esa cabeza (otro receptor, \
             otro aviso, o un camino de otra foto)"
                .to_string(),
        ));
    }
    let meta = zk_ssl_hash::meta_pendiente_hoja(foto.emisor, foto.nacido);
    let camino_meta =
        MerklePath { siblings: foto.hermanos_meta.clone(), is_right: cp.is_right.clone() };
    if native_root(meta, &camino_meta) != cab.pmeta_root {
        return Err(falla(
            "la meta servida no sube a la raiz de meta de esa cabeza por la misma posicion"
                .to_string(),
        ));
    }

    let w = cobro::CobroPendienteWitness {
        receptor,
        sal: aviso.salt,
        importe: aviso.amount,
        x,
        emisor: foto.emisor,
        nacido: foto.nacido,
        camino_pendiente: cp.clone(),
        hermanos_meta: foto.hermanos_meta.clone(),
    };
    let (prueba, pi) = cobro::probar(cobro::trazar(&w, inferior, techo))
        .map_err(|e| falla(format!("probar: {e}")))?;
    if pi.pending_root != cab.pending_root
        || pi.pmeta_root != cab.pmeta_root
        || pi.receptor != receptor
    {
        return Err(falla(
            "el enunciado derivado no es el de esta cabeza y este receptor".to_string(),
        ));
    }
    // El molde del S466: lo que sale ya esta verificado por la MISMA regla que corre el tercero,
    // la que lo ENLAZA a la cabeza (RFC-0008 D-K, S495): sus raices, su `seq` y el techo.
    let af = cobro::AfirmacionCobro { receptor, nacido: foto.nacido, inferior };
    let cabeza = cobro::CabezaCobro {
        seq: cab.seq,
        pending_root: cab.pending_root,
        pmeta_root: cab.pmeta_root,
    };
    let enlazado = cobro::verificar_contra_cabeza(&prueba, &af, &cabeza)
        .map_err(|e| falla(format!("la prueba recien producida no se enlaza: {e}")))?;
    if enlazado != pi {
        return Err(falla("el enlace compuso otro enunciado que el probador".to_string()));
    }
    Ok(SobreCobro {
        prueba,
        receptor,
        nacido: foto.nacido,
        inferior,
        seq: cab.seq,
        pending_root: cab.pending_root,
        pmeta_root: cab.pmeta_root,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests_support as ts;
    use crate::{AccountIndex, SovereignLayer};
    use winterfell::math::FieldElement;

    const IMPORTE: u64 = 250_000;
    const FONDO: u64 = 1_000_000;

    fn key(sk: u64) -> [BaseElement; 4] {
        [BaseElement::new(sk), BaseElement::ZERO, BaseElement::ZERO, BaseElement::ZERO]
    }

    /// Un envio v2 de Alice a Bob aplicado sobre un libro VIVO: el aviso de Bob. El `refund_id` es
    /// el `public_id` de Alice, como en los tres productores del arbol. Los indices de cuenta son
    /// los que `open_and_fund` devuelve, no 0 y 1: la capa los asigna salados (medido en el
    /// PASTE-491-PRE, que los supuso secuenciales).
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
        let aviso = envio(&mut l, alice, bob, 0xE1C0);
        let id_bob = l.public_id_of(bob).expect("bob");
        Libro { l, alice, bob, aviso, id_bob }
    }

    /// Lo que el latido guardaria (D-F): la cabeza y la foto de ESTE libro, AHORA. En los tests
    /// se lee del libro directamente; el metodo del nodo es otro corte.
    fn foto_de(l: &SovereignLayer, pos: u64) -> (CabezaDePendientes, FotoDelCobro) {
        let (emisor, nacido) = l.pending_meta_of(pos).expect("el pendiente tiene meta");
        let cab = CabezaDePendientes {
            seq: l.log.len() as u64,
            pending_root: l.pending.root(),
            pmeta_root: l.pending_meta_tree.root(),
        };
        let foto = FotoDelCobro {
            camino_pendiente: l.pending.path_for(pos),
            hermanos_meta: l.pending_meta_tree.path_for(pos).siblings,
            emisor,
            nacido,
        };
        (cab, foto)
    }

    fn enunciado(s: &SobreCobro) -> cobro::CobroPendientePublicInputs {
        cobro::CobroPendientePublicInputs {
            pending_root: s.pending_root,
            pmeta_root: s.pmeta_root,
            receptor: s.receptor,
            nacido: BaseElement::new(s.nacido),
            inferior: BaseElement::new(s.inferior),
            superior: BaseElement::new(cobro::MAX_VALOR),
        }
    }

    fn rehusa(r: Result<SobreCobro, LayerError>, texto: &str) {
        let e = r.expect_err("tenia que rehusar");
        assert!(format!("{e:?}").contains(texto), "se esperaba <<{texto}>>: {e:?}");
    }

    /// **El positivo, contra un libro vivo**: Bob prueba su cobro con su aviso y la foto, y el
    /// juez del kit lo acepta con el enunciado que el sobre declara.
    #[test]
    fn el_cobrador_prueba_su_pendiente_y_el_juez_lo_acepta() {
        let Libro { l, aviso, id_bob, .. } = libro();
        let (cab, foto) = foto_de(&l, aviso.position);
        let s = prueba_de_cobro_pendiente(&cab, id_bob, &aviso, &foto, 100_000)
            .expect("el cobrador prueba");
        assert_eq!((s.seq, s.inferior, s.nacido), (cab.seq, 100_000, foto.nacido));
        assert!(cobro::verificar(&s.prueba, &enunciado(&s)).is_ok(), "el juez no la acepta");
        let mut otro = enunciado(&s);
        otro.inferior = BaseElement::new(IMPORTE + 1);
        assert!(cobro::verificar(&s.prueba, &otro).is_err(), "la prueba vale para otra banda");
    }

    /// **La foto (D-F)**: tras otro pago el arbol vivo se mueve; con la cabeza y la foto de antes
    /// la prueba sale, y con la cabeza de despues y la foto de antes, rehusa.
    #[test]
    fn un_pago_entre_la_foto_y_la_prueba_no_la_invalida_pero_no_se_mezclan() {
        let Libro { mut l, alice, bob, aviso, id_bob } = libro();
        let (cab, foto) = foto_de(&l, aviso.position);
        let _otro = envio(&mut l, alice, bob, 0xE1C1);
        assert_ne!(l.pending.root(), cab.pending_root, "el libro tenia que moverse");
        assert!(prueba_de_cobro_pendiente(&cab, id_bob, &aviso, &foto, 1).is_ok());
        let (cab_nueva, _) = foto_de(&l, aviso.position);
        rehusa(
            prueba_de_cobro_pendiente(&cab_nueva, id_bob, &aviso, &foto, 1),
            "no sube a la raiz de pendientes",
        );
    }

    #[test]
    fn un_aviso_v1_no_tiene_prueba_de_cobro() {
        let Libro { l, mut aviso, id_bob, .. } = libro();
        let (cab, foto) = foto_de(&l, aviso.position);
        aviso.x = None;
        rehusa(prueba_de_cobro_pendiente(&cab, id_bob, &aviso, &foto, 1), "el aviso es v1");
    }

    #[test]
    fn una_banda_que_no_se_sostiene_rehusa_antes_de_probar() {
        let Libro { l, aviso, id_bob, .. } = libro();
        let (cab, foto) = foto_de(&l, aviso.position);
        rehusa(
            prueba_de_cobro_pendiente(&cab, id_bob, &aviso, &foto, IMPORTE + 1),
            "NO se sostiene",
        );
    }

    /// El aviso de Bob no prueba nada a nombre de Alice: la hoja recompuesta no sube.
    #[test]
    fn otro_receptor_no_sube_a_la_cabeza() {
        let Libro { l, alice, aviso, .. } = libro();
        let (cab, foto) = foto_de(&l, aviso.position);
        let id_alice = l.public_id_of(alice).expect("alice");
        rehusa(
            prueba_de_cobro_pendiente(&cab, id_alice, &aviso, &foto, 1),
            "no sube a la raiz de pendientes",
        );
    }

    /// Un camino de otra posicion se rechaza por sus bits, antes de subir nada.
    #[test]
    fn un_camino_de_otra_posicion_se_rechaza() {
        let Libro { mut l, alice, bob, aviso, id_bob } = libro();
        let otro = envio(&mut l, alice, bob, 0xE1C2);
        let (cab, foto_otra) = foto_de(&l, otro.position);
        rehusa(
            prueba_de_cobro_pendiente(&cab, id_bob, &aviso, &foto_otra, 1),
            "no es el de la posicion",
        );
    }

    /// Una meta mentida (otro nacido) no sube a la raiz de meta.
    #[test]
    fn una_meta_mentida_no_sube_a_la_cabeza() {
        let Libro { l, aviso, id_bob, .. } = libro();
        let (cab, mut foto) = foto_de(&l, aviso.position);
        foto.nacido += 1;
        rehusa(
            prueba_de_cobro_pendiente(&cab, id_bob, &aviso, &foto, 1),
            "la meta servida no sube",
        );
    }

    /// **D-K (S495), la regla MEDIDA antes de escribirse**: en un libro vivo la meta de un
    /// pendiente nace antes que la cabeza que lo firma (`nacido = log.len()` al nacer, `seq =
    /// log.len()` al componer la cabeza), y con otro pago de por medio lo sigue siendo.
    #[test]
    fn el_nacido_de_un_pendiente_es_anterior_a_su_cabeza() {
        let Libro { mut l, alice, bob, aviso, .. } = libro();
        let (cab, foto) = foto_de(&l, aviso.position);
        let (n1, s1) = (foto.nacido, cab.seq);
        println!("D-K| primero: nacido {n1} . seq {s1}");
        assert!(n1 < s1, "D-K DESMENTIDA: nacido {n1} y seq {s1}");
        let otro = envio(&mut l, alice, bob, 0xE1C3);
        let (cab2, foto2) = foto_de(&l, otro.position);
        let (n2, s2) = (foto2.nacido, cab2.seq);
        println!("D-K| segundo: nacido {n2} . seq {s2}");
        assert!(n2 < s2, "D-K DESMENTIDA: nacido {n2} y seq {s2}");
        assert!(n1 < n2, "el segundo tenia que nacer despues");
    }

    /// Dos listas son dos productores: las opciones del probador y las de la capa se atan aqui.
    #[test]
    fn las_opciones_del_probador_son_las_de_la_capa() {
        assert_eq!(cobro::opciones(), crate::proof_options());
    }
}
