//! **RFC-0007 E4b-3 (§466): la CAPA produce la prueba de edad.**
//!
//! El AIR y su juez viven en `zk-ssl-air` (S463), el probador en
//! `stark-experiment::circuit_edad` (S463) y el kit la verifica contra una cabeza v5 firmada
//! (S465). Faltaba quien la PRODUZCA sobre un libro vivo: el testigo -las hojas del arbol de
//! pendientes en `0..next_pending` y su `(emisor, nacido)`- es del OPERADOR y no cruza el cable,
//! asi que nadie fuera de esta capa puede componerla. Aqui esta, y en un solo sitio.
//!
//! **La puerta que hace falsable lo que sale.** El productor no afirma sobre <<el libro>>: afirma
//! sobre la cabeza v5 FIRMADA que se le da. Si el libro en disco no reproduce su `seq`, su
//! `pendingRoot`, su `pmetaRoot` y su `nextPending`, RECHAZA antes de probar nada. Sin eso, un
//! sobre podria hablar de un libro que ya se movio, y lo que el circuito no restringe no se
//! afirma.
//!
//! **La cota no se pide, se deriva.** `k` es la cuenta que la traza suma, no un argumento: no hay
//! forma de pedir una cota mas floja de la verdadera ni de mentir por exceso de confianza.
//!
//! **Lo que NO prueba** esta en la cabecera de `zk_ssl_air` y no se repite: nada sobre importes
//! -el operador no guarda la apertura del compromiso (RFC-0003 D-2)-, nada sobre lo que nunca
//! entro en el arbol (eso es H5b) y nada de otra epoca que la de la cabeza que firma las raices.
//!
//! **El emisor nombrado se compara EN EL CAMPO** (medido en el S466). Un pendiente de emision
//! lleva `sender = REFUND_SENDER_NONE` (`u64::MAX`), que en Goldilocks reduce a `2^32 - 2`, y un
//! `emisor` nombrado con ese mismo valor no se distingue de el. La colision solo puede INFLAR la
//! cuenta -la emision entra en `k`-, nunca esconder una posicion, asi que el enunciado sigue
//! siendo cierto para el emisor real y no se pone guarda: se declara, y hay testigo que lo ensena.

use crate::{Digest, LayerError, SovereignLayer};
use stark_experiment::circuit_edad as edad;

/// **Lo que la cabeza v5 firmada declara del libro**, tal como el kit lo lee: los cuatro campos
/// que la prueba necesita y que el operador no puede mover sin que la firma deje de verificar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CabezaDeclarada {
    pub seq: u64,
    pub pending_root: Digest,
    pub pmeta_root: Digest,
    pub next_pending: u64,
}

/// **Lo que el productor entrega**: la prueba y todo lo que el sobre necesita declarar. `k` sale
/// de la traza; `m` y `n`, del enunciado que el probador fijo.
#[derive(Clone, Debug)]
pub struct SobreEdad {
    pub prueba: Vec<u8>,
    pub subraiz_pend: Digest,
    pub subraiz_meta: Digest,
    pub t: u64,
    pub k: u64,
    pub emisor: Option<u64>,
    pub m: u32,
    pub n: u64,
    pub seq: u64,
}

impl SovereignLayer {
    /// **La prueba de edad de este libro contra una cabeza v5 firmada** (RFC-0007 E4b-3).
    ///
    /// `t` es la edad del enunciado (`edad = seq - born`); `emisor` nombra a uno o es `None` para
    /// todos. Devuelve el sobre con la cota `k` DERIVADA de la traza.
    ///
    /// Falla cerrado, y por su nombre, en cada paso: si el libro no es el que la cabeza firma, si
    /// un pendiente vivo no tiene meta (decision D-2 de E4b-1: su edad no esta comprometida), si
    /// el enunciado no cabe en el AIR, o si la prueba que acaba de producir no se ENLAZA a la
    /// cabeza por la misma regla que el kit corre (`verificar_contra_cabeza`). Esa ultima
    /// comprobacion cuesta unos 2 ms y convierte cada sobre emitido en uno ya verificado.
    pub fn prueba_de_edad(
        &self,
        cab: &CabezaDeclarada,
        t: u64,
        emisor: Option<u64>,
    ) -> Result<SobreEdad, LayerError> {
        let mio = CabezaDeclarada {
            seq: self.log.len() as u64,
            pending_root: self.pending_root(),
            pmeta_root: self.pending_meta_tree.root(),
            next_pending: self.next_pending,
        };
        if mio != *cab {
            return Err(LayerError::VerificationFailed(format!(
                "prueba de edad: el libro no es el que esa cabeza firma (libro seq {} \
                 nextPending {}; cabeza seq {} nextPending {}); las raices y la marca tienen que \
                 coincidir con la firma o el sobre hablaria de otro estado",
                mio.seq, mio.next_pending, cab.seq, cab.next_pending
            )));
        }

        let n = cab.next_pending;
        let mut hojas: Vec<Digest> = Vec::with_capacity(n as usize);
        let mut meta: Vec<Option<(u64, u64)>> = Vec::with_capacity(n as usize);
        for p in 0..n {
            hojas.push(self.pending_at(p));
            meta.push(self.pending_meta_of(p));
        }

        let en = edad::Enunciado {
            seq: cab.seq,
            t,
            emisor: emisor.unwrap_or(0),
            todos: emisor.is_none(),
        };
        let traza = edad::construir(&hojas, &meta, &en)
            .map_err(|e| LayerError::VerificationFailed(format!("prueba de edad: {e}")))?;
        let (prueba, pi) = edad::probar(traza)
            .map_err(|e| LayerError::VerificationFailed(format!("prueba de edad: probar: {e}")))?;

        let af = edad::Afirmacion {
            t: pi.t,
            k: pi.k,
            emisor,
            subraiz_pend: pi.subraiz_pend,
            subraiz_meta: pi.subraiz_meta,
        };
        let suya = edad::CabezaEdad {
            seq: cab.seq,
            pending_root: cab.pending_root,
            pmeta_root: cab.pmeta_root,
            next_pending: cab.next_pending,
        };
        edad::verificar_contra_cabeza(&prueba, &af, &suya).map_err(|e| {
            LayerError::VerificationFailed(format!(
                "prueba de edad: la prueba recien producida no se enlaza a su cabeza: {e}"
            ))
        })?;

        Ok(SobreEdad {
            prueba,
            subraiz_pend: pi.subraiz_pend,
            subraiz_meta: pi.subraiz_meta,
            t: pi.t,
            k: pi.k,
            emisor,
            m: pi.m,
            n: pi.n,
            seq: pi.seq,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests_support as ts;
    use crate::AccountIndex;
    use winterfell::math::{fields::f64::BaseElement, FieldElement};

    const CERO: Digest = [BaseElement::ZERO; 4];

    /// El centinela del emisor de una EMISION (`REFUND_SENDER_NONE`, `u64::MAX`) reducido al
    /// campo: `p = 2^64 - 2^32 + 1`, luego `u64::MAX = p + (2^32 - 2)`. No se teclea: se deriva.
    fn centinela_en_el_campo() -> u64 {
        (1u64 << 32) - 2
    }

    /// Un envio que se queda EN VUELO: `send` + `apply_send` y ningun cobro, que es lo que deja
    /// una posicion viva con su meta `(emisor, nacido)`.
    fn envia(
        l: &mut SovereignLayer,
        de: AccountIndex,
        sk: u64,
        a: AccountIndex,
        cuanto: u64,
        s: u64,
    ) {
        let estado = ts::state_of(l, de);
        let receptor = l.public_id_of(a).expect("la cuenta receptora existe");
        let recibo = l
            .send(BaseElement::new(sk), de, &estado, receptor, ts::salt_de(s), cuanto)
            .expect("el envio prueba");
        l.apply_send(&recibo, de, &estado, cuanto).expect("el envio se aplica");
    }

    /// **Un libro REAL con tres posiciones vivas**: dos envios en vuelo del emisor de Alice,
    /// nacidos a dos alturas distintas, y una EMISION a pendiente, cuyo emisor es el centinela.
    /// Es el material minimo que discrimina: dos edades y dos clases de emisor.
    fn libro() -> (SovereignLayer, AccountIndex, AccountIndex) {
        let mut l = ts::new_layer();
        let a = ts::open_and_fund(&mut l, ts::SK_ALICE, 10_000);
        let b = ts::open_and_fund(&mut l, ts::SK_BOB, 0);
        envia(&mut l, a, ts::SK_ALICE, b, 100, 1);
        envia(&mut l, a, ts::SK_ALICE, b, 200, 2);
        let receptor = l.public_id_of(b).expect("la cuenta receptora existe");
        ts::mint_to_pending_delegated(&mut l, receptor, ts::salt_de(3), 50);
        (l, a, b)
    }

    /// La cabeza que este libro firmaria AHORA, por los mismos cuatro campos que la v5 lleva.
    fn cabeza_de(l: &SovereignLayer) -> CabezaDeclarada {
        CabezaDeclarada {
            seq: l.log.len() as u64,
            pending_root: l.pending_root(),
            pmeta_root: l.pending_meta_tree.root(),
            next_pending: l.next_pending,
        }
    }

    /// La cuenta A MANO: posiciones vivas con `seq - nacido >= t` y, si se nombra, del emisor
    /// nombrado COMPARADO EN EL CAMPO, que es como lo compara el circuito.
    fn a_mano(l: &SovereignLayer, seq: u64, t: u64, emisor: Option<u64>) -> u64 {
        (0..l.next_pending)
            .filter(|&p| l.pending_at(p) != CERO)
            .filter(|&p| {
                let (s, nacido) = l.pending_meta_of(p).expect("un vivo tiene meta");
                let suyo = match emisor {
                    None => true,
                    Some(x) => BaseElement::new(s) == BaseElement::new(x),
                };
                suyo && seq - nacido >= t
            })
            .count() as u64
    }

    /// **El positivo (E4b-3):** la capa produce sobre su propio libro y el juez del kit
    /// ENLAZA la prueba a la cabeza que ese libro firma, por la misma regla que corre un
    /// tercero. La cota que sale es la cuenta a mano: `k` no se pide, se deriva.
    #[test]
    #[cfg_attr(debug_assertions, ignore = "winterfell valida grados en depuracion: juez release")]
    fn la_capa_produce_la_prueba_de_su_libro_y_el_juez_la_enlaza() {
        let (l, _a, _b) = libro();
        let cab = cabeza_de(&l);
        let t = 2;
        let sobre = l.prueba_de_edad(&cab, t, None).expect("la capa produce");
        assert_eq!(sobre.k, a_mano(&l, cab.seq, t, None), "la cota no es la cuenta a mano");
        assert_eq!(sobre.n, cab.next_pending);
        assert_eq!(sobre.seq, cab.seq);
        let af = edad::Afirmacion {
            t: sobre.t,
            k: sobre.k,
            emisor: None,
            subraiz_pend: sobre.subraiz_pend,
            subraiz_meta: sobre.subraiz_meta,
        };
        let suya = edad::CabezaEdad {
            seq: cab.seq,
            pending_root: cab.pending_root,
            pmeta_root: cab.pmeta_root,
            next_pending: cab.next_pending,
        };
        edad::verificar_contra_cabeza(&sobre.prueba, &af, &suya).expect("el juez no la enlazo");
    }

    /// **El falsador de la puerta:** con la cabeza de ANTES y el libro movido una operacion, el
    /// productor RECHAZA antes de probar nada. Sin esta puerta, un sobre podria hablar de un
    /// estado que la firma no cubre.
    #[test]
    #[cfg_attr(debug_assertions, ignore = "winterfell valida grados en depuracion: juez release")]
    fn un_libro_que_se_movio_no_produce_contra_la_cabeza_vieja() {
        let (mut l, _a, b) = libro();
        let vieja = cabeza_de(&l);
        let receptor = l.public_id_of(b).expect("la cuenta receptora existe");
        ts::mint_to_pending_delegated(&mut l, receptor, ts::salt_de(4), 7);
        assert_ne!(cabeza_de(&l), vieja, "el sabotaje no movio el libro");
        let e = l.prueba_de_edad(&vieja, 2, None).expect_err("produjo contra otra cabeza");
        let texto = format!("{e}");
        assert!(texto.contains("no es el que esa cabeza firma"), "{texto}");
    }

    /// **La colision del emisor de emision, MEDIDA (5.A-183).** Un pendiente de emision lleva
    /// `sender = REFUND_SENDER_NONE` (`u64::MAX`), que en el campo es `2^32 - 2`: nombrar ese
    /// valor lo cuenta como si fuera la cuenta de ese indice. La cota sale INFLADA, nunca corta,
    /// asi que el enunciado sigue siendo cierto para el emisor real; se declara y aqui se ensena.
    #[test]
    #[cfg_attr(debug_assertions, ignore = "winterfell valida grados en depuracion: juez release")]
    fn nombrar_el_centinela_en_el_campo_cuenta_la_emision() {
        let (l, _a, _b) = libro();
        let cab = cabeza_de(&l);
        let s = centinela_en_el_campo();
        assert_eq!(
            BaseElement::new(s),
            BaseElement::new(crate::REFUND_SENDER_NONE),
            "el centinela no reduce a 2^32 - 2: la premisa del testigo es falsa"
        );
        let sobre = l.prueba_de_edad(&cab, 0, Some(s)).expect("la capa produce");
        assert_eq!(sobre.k, a_mano(&l, cab.seq, 0, Some(s)));
        assert!(sobre.k >= 1, "la emision no entro en la cuenta del emisor nombrado");
    }

    /// **Nombrar a un emisor REAL no cuenta lo ajeno:** con el indice de Alice salen sus dos
    /// envios y no la emision. Es el discriminante del testigo de arriba: sin este, aquel
    /// pasaria con un circuito que contara todo.
    #[test]
    #[cfg_attr(debug_assertions, ignore = "winterfell valida grados en depuracion: juez release")]
    fn nombrar_un_emisor_real_deja_fuera_la_emision() {
        let (l, a, _b) = libro();
        let cab = cabeza_de(&l);
        let sobre = l.prueba_de_edad(&cab, 0, Some(a)).expect("la capa produce");
        assert_eq!(sobre.k, a_mano(&l, cab.seq, 0, Some(a)));
        assert!(
            sobre.k < a_mano(&l, cab.seq, 0, None),
            "nombrar un emisor no dejo fuera nada: el testigo no discrimina"
        );
    }
}
