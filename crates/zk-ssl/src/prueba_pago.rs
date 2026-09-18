//! **RFC-0008 E2: el PAGADOR produce la prueba de su pago en curso.**
//!
//! El AIR vive en `zk-ssl-air::pago_en_curso` y el probador en
//! `stark_experiment::circuit_pago_en_curso` (S503). Faltaba quien la PRODUZCA con lo que el
//! pagador tiene de verdad: su APERTURA -la posicion, la sal, el importe y la pareja
//! `(refund_id, delta)` que el eligio y que `send_materials_v2` le devolvio en `SendMaterials`-,
//! la identidad publica del receptor y lo que el nodo le sirva de la foto del ultimo latido -los
//! dos caminos de ESA posicion y su meta `(emisor, nacido)`- (D-AE). Por eso esto es una funcion
//! LIBRE y no un metodo del libro: no lee el libro, y se puede correr en un cliente sin el.
//!
//! **El espejo de E1, y donde no lo es.** El cobrador afirma <<me deben AL MENOS `inferior`>> con
//! el sobre `X` opaco; el pagador afirma <<pague `importe` EXACTO y no lo puedo revertir antes de
//! `T`>>, y para eso abre el sobre por dentro del circuito: conoce la pareja, y el AIR la compone
//! en el carril B sin publicarla (D-AD, D-AF). El plazo NO viaja: lo que se prueba es
//! `delta >= T - nacido`, nunca `delta`.
//!
//! **Sin clave.** El enunciado es de ESTADO, como en E1: nada de lo que entra aqui es un secreto
//! de firma, y el receptor -que conoce la apertura menos la pareja- no puede producirla.
//!
//! **La puerta que hace falsable lo que sale.** El productor no afirma sobre <<un libro>>: afirma
//! sobre las dos raices de la cabeza v5 FIRMADA que se le dan. Antes de probar recompone en
//! nativo la hoja v2 de su apertura y la de la meta y las SUBE por los caminos, con los bits que
//! salen de la posicion; si algo no llega a las raices de esa cabeza, RECHAZA y dice que. Y lo
//! que sale ya esta verificado por el mismo juez que corre el tercero.
//!
//! **Lo que comprueba ANTES de gastar una prueba, y el molde no.** `nacido < seq` lo exige el
//! enlace (S503), que corre al final; aqui se comprueba primero, porque una cabeza que no puede
//! firmar ese pendiente no merece una prueba, y porque un rc que dice <<no se enlaza>> esconde la
//! causa. Es la misma regla, adelantada.

use crate::pending::pending_commitment_v2;
use crate::prueba_cobro::{bits_de, CabezaDePendientes, FotoDelCobro};
use crate::{Digest, LayerError};
use stark_experiment::circuit_pago_en_curso as pago;
use stark_experiment::merkle::{native_root, MerklePath};

/// **Lo que el PAGADOR tiene de su propio pago**, y que nadie mas tiene entero: la pareja del
/// sobre es suya (RFC-0003) y el receptor la recibe OPACA. El `receptor` va aparte, como en el
/// molde: es lo unico publico de las dos partes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AperturaDelPago {
    pub posicion: u64,
    pub sal: Digest,
    pub importe: u64,
    pub refund_id: Digest,
    pub delta: u64,
}

/// **Lo que el productor entrega**: la prueba y lo que el sobre declara. El importe SI sale, y
/// exacto -es lo que el pagador afirma haber pagado (D-AD)-; la sal, la pareja y el emisor no.
#[derive(Clone, Debug)]
pub struct SobrePago {
    pub prueba: Vec<u8>,
    pub receptor: Digest,
    pub importe: u64,
    pub t: u64,
    pub nacido: u64,
    pub seq: u64,
    pub pending_root: Digest,
    pub pmeta_root: Digest,
}

fn falla(que: String) -> LayerError {
    LayerError::VerificationFailed(format!("prueba de pago: {que}"))
}

/// **La prueba de que el pendiente de esta apertura esta bajo la cabeza, a nombre de `receptor`,
/// por `importe` EXACTO, y de que no puede revertir antes de `T`** (RFC-0008 E2).
///
/// Falla cerrado y por su nombre en cada paso, y NUNCA con una variante nueva de `LayerError`:
/// una variante nueva rompe todo `match` exhaustivo de fuera.
pub fn prueba_de_pago_en_curso(
    cab: &CabezaDePendientes,
    receptor: Digest,
    apertura: &AperturaDelPago,
    foto: &FotoDelCobro,
    t: u64,
) -> Result<SobrePago, LayerError> {
    let techo = pago::MAX_VALOR;
    if apertura.importe > techo {
        return Err(falla(format!(
            "el importe {} pasa el techo del campo ({techo}): su pago no es probable",
            apertura.importe
        )));
    }
    if t > techo || foto.nacido > techo {
        return Err(falla(format!(
            "T {t} o nacido {} pasan el techo del campo ({techo})",
            foto.nacido
        )));
    }
    if foto.nacido > t {
        return Err(falla(format!(
            "el plazo iria hacia atras: nacido {} es posterior a T {t}",
            foto.nacido
        )));
    }
    // D-AD: lo que se afirma es que el pago no revierte antes de `T`, y eso es
    // `T <= nacido + delta`. La resta va en u64 y los dos ya estan bajo el techo.
    if t - foto.nacido > apertura.delta {
        return Err(falla(format!(
            "el pago NO se sostiene hasta T: de nacido {} a T {t} van {} epocas y el sobre \
             lleva {}",
            foto.nacido,
            t - foto.nacido,
            apertura.delta
        )));
    }
    if foto.nacido >= cab.seq {
        return Err(falla(format!(
            "nacido {} no es anterior a la cabeza de seq {}: esa cabeza no firma este pendiente",
            foto.nacido, cab.seq
        )));
    }
    let niveles = pago::PROFUNDIDAD;
    let cp = &foto.camino_pendiente;
    if cp.siblings.len() != niveles
        || cp.is_right.len() != niveles
        || foto.hermanos_meta.len() != niveles
    {
        return Err(falla(format!("los caminos no tienen {niveles} niveles")));
    }
    if cp.is_right != bits_de(apertura.posicion, niveles) {
        return Err(falla(format!(
            "el camino servido no es el de la posicion {} de la apertura",
            apertura.posicion
        )));
    }

    // La hoja que el pagador recompone con su apertura ENTERA, en nativo, y la meta de esa
    // posicion. El compositor v2 es el mismo que la capa uso al escribirla (RFC-0003, E1a).
    let hoja = pending_commitment_v2(
        receptor,
        apertura.sal,
        apertura.importe,
        apertura.refund_id,
        apertura.delta,
    );
    if native_root(hoja, cp) != cab.pending_root {
        return Err(falla(
            "la hoja de la apertura no sube a la raiz de pendientes de esa cabeza (otro \
             receptor, otra pareja, otro importe, o un camino de otra foto)"
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

    let w = pago::PagoEnCursoWitness {
        receptor,
        sal: apertura.sal,
        importe: apertura.importe,
        refund_id: apertura.refund_id,
        delta: apertura.delta,
        emisor: foto.emisor,
        nacido: foto.nacido,
        camino_pendiente: cp.clone(),
        hermanos_meta: foto.hermanos_meta.clone(),
    };
    let (prueba, pi) =
        pago::probar(pago::trazar(&w, t)).map_err(|e| falla(format!("probar: {e}")))?;
    if pi.pending_root != cab.pending_root
        || pi.pmeta_root != cab.pmeta_root
        || pi.receptor != receptor
    {
        return Err(falla(
            "el enunciado derivado no es el de esta cabeza y este receptor".to_string(),
        ));
    }
    // El molde del S466 y del S491: lo que sale ya esta verificado por la MISMA regla que corre
    // el tercero, la que lo ENLAZA a la cabeza firmada (S503).
    let af = pago::AfirmacionPago { receptor, importe: apertura.importe, t, nacido: foto.nacido };
    let cabeza = pago::CabezaPago {
        seq: cab.seq,
        pending_root: cab.pending_root,
        pmeta_root: cab.pmeta_root,
    };
    let enlazado = pago::verificar_contra_cabeza(&prueba, &af, &cabeza)
        .map_err(|e| falla(format!("la prueba recien producida no se enlaza: {e}")))?;
    if enlazado != pi {
        return Err(falla("el enlace compuso otro enunciado que el probador".to_string()));
    }
    Ok(SobrePago {
        prueba,
        receptor,
        importe: apertura.importe,
        t,
        nacido: foto.nacido,
        seq: cab.seq,
        pending_root: cab.pending_root,
        pmeta_root: cab.pmeta_root,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pending::refund_envelope;
    use crate::prueba_cobro::{prueba_de_cobro_pendiente, SobreCobro};
    use crate::tests_support as ts;
    use crate::two_phase::PendingNotice;
    use crate::{AccountIndex, SovereignLayer};
    use winterfell::math::fields::f64::BaseElement;
    use winterfell::math::FieldElement;

    const IMPORTE: u64 = 250_000;
    const FONDO: u64 = 1_000_000;
    const DELTA: u64 = 96;

    fn key(sk: u64) -> [BaseElement; 4] {
        [BaseElement::new(sk), BaseElement::ZERO, BaseElement::ZERO, BaseElement::ZERO]
    }

    /// Un envio v2 de Alice a Bob aplicado sobre un libro VIVO. Devuelve lo que el PAGADOR se
    /// queda -su apertura- y el aviso del receptor, que aqui es CONTROL: el pagador no lo tiene.
    /// El `refund_id` es el `public_id` de Alice, como en los tres productores del arbol.
    fn envio(
        layer: &mut SovereignLayer,
        alice: AccountIndex,
        bob: AccountIndex,
        semilla: u64,
    ) -> (AperturaDelPago, PendingNotice) {
        let id_bob = layer.public_id_of(bob).expect("bob");
        let f = layer.public_id_of(alice).expect("alice");
        let sal = ts::salt_de(semilla);
        let m = layer
            .send_materials_v2(alice, id_bob, IMPORTE, sal, f, DELTA)
            .expect("materiales v2");
        assert_eq!(m.sobre, Some((f, DELTA)), "la capa no guarda la pareja del pagador");
        let apertura = AperturaDelPago {
            posicion: m.pending_position,
            sal,
            importe: IMPORTE,
            refund_id: f,
            delta: DELTA,
        };
        let recibo = crate::client::prove_send(&m, key(ts::SK_ALICE), crate::proof_options())
            .expect("probar el envio");
        let ea = ts::state_of(layer, alice);
        layer.apply_send(&recibo, alice, &ea, IMPORTE).expect("aplicar el envio");
        (apertura, recibo.notice)
    }

    struct Libro {
        l: SovereignLayer,
        alice: AccountIndex,
        bob: AccountIndex,
        id_bob: Digest,
        apertura: AperturaDelPago,
        aviso: PendingNotice,
    }

    fn libro() -> Libro {
        let mut l = ts::new_layer();
        let alice = ts::open_and_fund(&mut l, ts::SK_ALICE, FONDO);
        let bob = ts::open_and_fund(&mut l, ts::SK_BOB, 0);
        assert_ne!(alice, bob);
        let (apertura, aviso) = envio(&mut l, alice, bob, 0xE2D0);
        let id_bob = l.public_id_of(bob).expect("bob");
        Libro { l, alice, bob, id_bob, apertura, aviso }
    }

    /// Lo que el latido guardaria (D-F): la cabeza y la foto de ESTE libro, AHORA. La foto la
    /// sirve el nodo con la credencial del pagador (D-AE); ese metodo es otro corte.
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

    fn enunciado(s: &SobrePago) -> pago::PagoEnCursoPublicInputs {
        pago::PagoEnCursoPublicInputs {
            pending_root: s.pending_root,
            pmeta_root: s.pmeta_root,
            receptor: s.receptor,
            importe: BaseElement::new(s.importe),
            t: BaseElement::new(s.t),
            nacido: BaseElement::new(s.nacido),
        }
    }

    fn rehusa(r: Result<SobrePago, LayerError>, texto: &str) {
        let e = r.expect_err("tenia que rehusar");
        assert!(format!("{e:?}").contains(texto), "se esperaba <<{texto}>>: {e:?}");
    }

    /// **El positivo, contra un libro vivo**: Alice prueba su pago con su apertura y la foto, y
    /// el juez del kit lo acepta con el enunciado que el sobre declara. Con el importe EXACTO:
    /// una prueba de este pago no vale para otro importe.
    #[test]
    fn el_pagador_prueba_su_pago_y_el_juez_lo_acepta() {
        let Libro { l, id_bob, apertura, .. } = libro();
        let (cab, foto) = foto_de(&l, apertura.posicion);
        let t = foto.nacido + DELTA;
        let s = prueba_de_pago_en_curso(&cab, id_bob, &apertura, &foto, t)
            .expect("el pagador prueba");
        assert_eq!((s.seq, s.importe, s.t, s.nacido), (cab.seq, IMPORTE, t, foto.nacido));
        assert!(pago::verificar(&s.prueba, &enunciado(&s)).is_ok(), "el juez no la acepta");
        let mut otro = enunciado(&s);
        otro.importe = BaseElement::new(IMPORTE + 1);
        assert!(pago::verificar(&s.prueba, &otro).is_err(), "la prueba vale para otro importe");
    }

    /// **La foto (D-F)**: tras otro pago el arbol vivo se mueve; con la cabeza y la foto de antes
    /// la prueba sale, y con la cabeza de despues y la foto de antes, rehusa.
    #[test]
    fn un_pago_posterior_no_invalida_la_prueba_del_pagador() {
        let Libro { mut l, alice, bob, id_bob, apertura, .. } = libro();
        let (cab, foto) = foto_de(&l, apertura.posicion);
        let _otro = envio(&mut l, alice, bob, 0xE2D1);
        assert_ne!(l.pending.root(), cab.pending_root, "el libro tenia que moverse");
        let t = foto.nacido + DELTA;
        assert!(prueba_de_pago_en_curso(&cab, id_bob, &apertura, &foto, t).is_ok());
        let (cab_nueva, _) = foto_de(&l, apertura.posicion);
        rehusa(
            prueba_de_pago_en_curso(&cab_nueva, id_bob, &apertura, &foto, t),
            "no sube a la raiz de pendientes",
        );
    }

    /// **D-AD, la frontera, en el productor**: en `T = nacido + delta` prueba; una epoca mas
    /// alla rehusa ANTES de gastar una prueba, y lo dice con sus cifras.
    #[test]
    fn la_frontera_de_t_en_el_productor_del_pago() {
        let Libro { l, id_bob, apertura, .. } = libro();
        let (cab, foto) = foto_de(&l, apertura.posicion);
        assert!(prueba_de_pago_en_curso(&cab, id_bob, &apertura, &foto, foto.nacido + DELTA)
            .is_ok());
        rehusa(
            prueba_de_pago_en_curso(&cab, id_bob, &apertura, &foto, foto.nacido + DELTA + 1),
            "NO se sostiene hasta T",
        );
    }

    /// Un `T` anterior al nacimiento no es una banda estrecha: es un plazo hacia atras, y la
    /// resta daria la vuelta en el campo. Rehusa por su nombre.
    #[test]
    fn un_plazo_hacia_atras_rehusa_antes_de_probar() {
        let Libro { l, id_bob, apertura, .. } = libro();
        let (cab, foto) = foto_de(&l, apertura.posicion);
        assert!(foto.nacido > 0, "el escenario necesita un nacido > 0");
        rehusa(
            prueba_de_pago_en_curso(&cab, id_bob, &apertura, &foto, foto.nacido - 1),
            "el plazo iria hacia atras",
        );
    }

    /// La apertura de Alice no prueba nada a nombre de Alice: la hoja recompuesta no sube.
    #[test]
    fn otro_receptor_no_sube_a_la_cabeza_en_el_pago() {
        let Libro { l, alice, apertura, .. } = libro();
        let (cab, foto) = foto_de(&l, apertura.posicion);
        let id_alice = l.public_id_of(alice).expect("alice");
        rehusa(
            prueba_de_pago_en_curso(&cab, id_alice, &apertura, &foto, foto.nacido + DELTA),
            "no sube a la raiz de pendientes",
        );
    }

    /// **El par ATA**: otro `refund_id` con el mismo `delta` da otra hoja, y no sube. Es lo que
    /// impide presentar el pago de otro como propio.
    #[test]
    fn otra_pareja_no_sube_a_la_cabeza() {
        let Libro { l, id_bob, apertura, .. } = libro();
        let (cab, foto) = foto_de(&l, apertura.posicion);
        let mut ajena = apertura;
        ajena.refund_id = ts::salt_de(0x0DD0);
        assert_ne!(ajena.refund_id, apertura.refund_id, "el escenario no cambia la pareja");
        rehusa(
            prueba_de_pago_en_curso(&cab, id_bob, &ajena, &foto, foto.nacido + DELTA),
            "no sube a la raiz de pendientes",
        );
    }

    /// Un camino de otra posicion se rechaza por sus bits, antes de subir nada.
    #[test]
    fn un_camino_de_otra_posicion_se_rechaza_en_el_pago() {
        let Libro { mut l, alice, bob, id_bob, apertura, .. } = libro();
        let (otra, _) = envio(&mut l, alice, bob, 0xE2D2);
        let (cab, foto_otra) = foto_de(&l, otra.posicion);
        assert_ne!(otra.posicion, apertura.posicion, "el escenario necesita dos posiciones");
        rehusa(
            prueba_de_pago_en_curso(&cab, id_bob, &apertura, &foto_otra, foto_otra.nacido + DELTA),
            "no es el de la posicion",
        );
    }

    /// Una meta mentida -otro EMISOR, que es mentir quien pago- no sube a la raiz de meta. Se
    /// miente el emisor y no el `nacido` a proposito: mover el `nacido` chocaria antes con la
    /// puerta de la cabeza, y un falsador que da el rojo por otra causa no discrimina.
    #[test]
    fn un_emisor_mentido_no_sube_a_la_raiz_de_meta() {
        let Libro { l, id_bob, apertura, .. } = libro();
        let (cab, mut foto) = foto_de(&l, apertura.posicion);
        foto.emisor ^= 1;
        rehusa(
            prueba_de_pago_en_curso(&cab, id_bob, &apertura, &foto, foto.nacido + DELTA),
            "la meta servida no sube",
        );
    }

    /// Una cabeza que no puede firmar este pendiente se rehusa por su nombre, y NO gastando una
    /// prueba para descubrirlo en el enlace: es la regla del S503, adelantada.
    #[test]
    fn una_cabeza_anterior_al_nacimiento_rehusa_antes_de_probar() {
        let Libro { l, id_bob, apertura, .. } = libro();
        let (mut cab, foto) = foto_de(&l, apertura.posicion);
        cab.seq = foto.nacido;
        rehusa(
            prueba_de_pago_en_curso(&cab, id_bob, &apertura, &foto, foto.nacido + DELTA),
            "no es anterior a la cabeza",
        );
    }

    /// **Las dos mitades de un pendiente disputado** (5.B-70): sobre el MISMO libro y la MISMA
    /// cabeza, el pagador prueba su pago y el cobrador su cobro; las dos raices y el receptor son
    /// los mismos, y cada mitad dice lo suyo -el importe exacto y el plazo, o la banda-.
    #[test]
    fn las_dos_mitades_del_mismo_pendiente_se_prueban_contra_la_misma_cabeza() {
        let Libro { l, id_bob, apertura, aviso, .. } = libro();
        let (cab, foto) = foto_de(&l, apertura.posicion);
        assert_eq!(
            aviso.x,
            Some(refund_envelope(apertura.refund_id, apertura.delta)),
            "el sobre del aviso no es el de la apertura"
        );
        let p: SobrePago =
            prueba_de_pago_en_curso(&cab, id_bob, &apertura, &foto, foto.nacido + DELTA)
                .expect("el pagador prueba");
        let c: SobreCobro = prueba_de_cobro_pendiente(&cab, id_bob, &aviso, &foto, 1)
            .expect("el cobrador prueba");
        assert_eq!(p.pending_root, c.pending_root, "dos mitades, una raiz de pendientes");
        assert_eq!(p.pmeta_root, c.pmeta_root, "dos mitades, una raiz de meta");
        assert_eq!(p.receptor, c.receptor, "dos mitades, un receptor");
        assert_eq!((p.seq, p.nacido), (c.seq, c.nacido));
        assert_eq!(p.importe, IMPORTE, "el pago dice el importe exacto");
        assert!(c.inferior <= p.importe, "el cobro dice una cota, no el importe");
        assert_ne!(p.prueba, c.prueba, "son dos pruebas de dos enunciados");
    }

    /// Dos listas son dos productores: las opciones del probador y las de la capa se atan aqui.
    #[test]
    fn las_opciones_del_probador_son_las_de_la_capa() {
        assert_eq!(pago::opciones(), crate::proof_options());
    }
}
