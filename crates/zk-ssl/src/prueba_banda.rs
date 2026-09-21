//! **RFC-0007 E5, corte 4b: la CAPA produce la prueba de `InsufficientBalance`.**
//!
//! El AIR de la banda vive en `zk-ssl-air` (S477), el probador en
//! `stark-experiment::circuit_banda` (S477) y el mando la verifica sin nodo. Faltaba quien la
//! PRODUZCA sobre un libro vivo: el saldo, el nonce, el salt y el camino de la cuenta son del
//! OPERADOR y no cruzan el cable, asi que nadie fuera de esta capa puede componerla. Aqui esta,
//! y en un solo sitio. Es el molde de `prueba_edad.rs` (S466).
//!
//! **Sin clave, y por eso existe.** El testigo es el de `audit()` menos el `spend_key`, y por eso
//! el 4a recorto el ciclo de titularidad. Con el dentro, el rechazo no seria producible por
//! quien lo emite. `account_view` es `pub(crate)`: este modulo vive DENTRO del crate a
//! proposito, porque su material no debe salir de el.
//!
//! **La puerta que hace falsable lo que sale.** El productor no afirma sobre <<el libro>>:
//! afirma sobre la cabeza v5 FIRMADA que se le da. Si el libro en disco no reproduce su `seq` y
//! su `accountsRoot`, RECHAZA antes de probar nada.
//!
//! **La cota no se pide, se deriva**: la banda es `[0, pedido - 1]` y el `pedido` es el de la
//! peticion que se rechazo. No hay forma de pedir una banda mas floja.
//!
//! **Lo que NO prueba** esta en la cabecera de `zk_ssl_air::banda` y no se repite. Y lo que este
//! corte decidio: el `data` del sobre NO trae el saldo, pero la PRUEBA si lo publica -abre sus
//! filas en claro con el saldo y el `leaf_salt`, medido en el S521-, y el CABLE lo manda desde el
//! S454 a quien hizo la peticion. <<Probar sin el>> es del enunciado, no de lo que se ve.

use crate::{AccountIndex, Digest, LayerError, SovereignLayer};
use stark_experiment::circuit_banda as banda;

/// **Lo que la cabeza v5 firmada declara del arbol de cuentas**: los dos campos que la prueba
/// necesita y que el operador no puede mover sin que la firma deje de verificar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CabezaDeCuentas {
    pub seq: u64,
    pub accounts_root: Digest,
}

/// **Lo que el productor entrega**: la prueba y lo que el sobre declara. El saldo no va como
/// CAMPO (D-0 del corte), pero va dentro de la prueba, que no oculta su testigo (S521).
#[derive(Clone, Debug)]
pub struct SobreBanda {
    pub prueba: Vec<u8>,
    pub public_id: Digest,
    pub requested: u64,
    pub seq: u64,
}

impl SovereignLayer {
    /// **La prueba de que el saldo de `index` no llega a `pedido`, contra una cabeza v5
    /// firmada** (RFC-0007 E5, corte 4b).
    ///
    /// Falla cerrado y por su nombre en cada paso, y NUNCA con una variante nueva de
    /// `LayerError`: una variante nueva rompe todo `match` exhaustivo de fuera.
    pub fn prueba_de_banda(
        &self,
        cab: &CabezaDeCuentas,
        index: AccountIndex,
        pedido: u64,
    ) -> Result<SobreBanda, LayerError> {
        let mio = CabezaDeCuentas { seq: self.log.len() as u64, accounts_root: self.state_root() };
        if mio != *cab {
            return Err(LayerError::VerificationFailed(format!(
                "prueba de banda: el libro no es el que esa cabeza firma (libro seq {}; cabeza \
                 seq {}); el seq y el accountsRoot tienen que coincidir con la firma o el sobre \
                 hablaria de otro estado",
                mio.seq, cab.seq
            )));
        }
        let vista = self.account_view(index).ok_or_else(|| {
            LayerError::VerificationFailed(format!(
                "prueba de banda: la cuenta {index} no existe en este libro: su rechazo seria \
                 AccountNotFound, que es otra causa y otro corte"
            ))
        })?;
        if pedido <= vista.balance {
            return Err(LayerError::VerificationFailed(format!(
                "prueba de banda: la causa NO se sostiene: pedir {pedido} a la cuenta {index} es \
                 legitimo en este libro"
            )));
        }
        // La banda es [0, pedido - 1]. `pedido >= 1` se deriva de la linea de arriba y no lleva
        // guarda: con `pedido = 0` la causa no puede nacer (`amount > balance` seria `0 > b`).
        let techo = banda::MAX_VALOR;
        if pedido - 1 > techo {
            return Err(LayerError::VerificationFailed(format!(
                "prueba de banda: el pedido {pedido} pasa el techo del campo ({techo}): esta \
                 causa es producible por el cable y su prueba de banda NO"
            )));
        }
        let (_, camino) = self.accounts_path_of(index);
        let w = banda::BandaWitness {
            public_id: vista.public_id,
            balance: vista.balance,
            nonce: vista.nonce,
            leaf_salt: vista.leaf_salt,
            path: camino,
        };
        let (prueba, pi) = banda::probar(banda::trazar(&w, 0, pedido - 1))
            .map_err(|e| LayerError::VerificationFailed(format!("prueba de banda: probar: {e}")))?;
        if pi.root != cab.accounts_root || pi.public_id != vista.public_id {
            return Err(LayerError::VerificationFailed(
                "prueba de banda: el enunciado derivado no es el de esta cabeza y esta cuenta"
                    .to_string(),
            ));
        }
        // El molde del S466: lo que sale ya esta verificado por la MISMA regla que corre el
        // tercero, no por una propia. Cuesta unos 2 ms y deja de haber sobres muertos.
        banda::verificar(&prueba, &pi).map_err(|e| {
            LayerError::VerificationFailed(format!(
                "prueba de banda: la prueba recien producida no verifica: {e}"
            ))
        })?;
        Ok(SobreBanda { prueba, public_id: vista.public_id, requested: pedido, seq: cab.seq })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests_support as ts;

    /// Un libro con una cuenta fondeada y su cabeza declarada, la que el libro reproduce.
    fn libro() -> (SovereignLayer, AccountIndex, CabezaDeCuentas) {
        let mut l = ts::new_layer();
        let a = ts::open_and_fund(&mut l, ts::SK_ALICE, 1_000);
        let cab = CabezaDeCuentas { seq: l.log.len() as u64, accounts_root: l.state_root() };
        (l, a, cab)
    }

    #[test]
    fn el_productor_emite_una_prueba_que_el_juez_del_kit_acepta() {
        let (l, a, cab) = libro();
        let s = l.prueba_de_banda(&cab, a, 1_001).expect("el productor prueba");
        assert_eq!(s.requested, 1_001);
        assert_eq!(s.seq, cab.seq);
        assert!(!s.prueba.is_empty(), "la prueba salio vacia");
    }

    #[test]
    fn un_libro_que_no_es_el_que_la_cabeza_firma_no_prueba() {
        let (l, a, mut cab) = libro();
        cab.seq += 1;
        let e = l.prueba_de_banda(&cab, a, 1_001).expect_err("tenia que rehusar");
        assert!(format!("{e:?}").contains("no es el que esa cabeza firma"), "{e:?}");
    }

    #[test]
    fn un_pedido_por_encima_del_techo_del_campo_rehusa_nombrandolo() {
        let (l, a, cab) = libro();
        let e = l.prueba_de_banda(&cab, a, banda::MAX_VALOR + 2).expect_err("tenia que rehusar");
        assert!(format!("{e:?}").contains("pasa el techo del campo"), "{e:?}");
    }

    #[test]
    fn un_envio_legitimo_no_tiene_sobre_que_producir() {
        let (l, a, cab) = libro();
        let e = l.prueba_de_banda(&cab, a, 10).expect_err("tenia que rehusar");
        assert!(format!("{e:?}").contains("NO se sostiene"), "{e:?}");
    }

    /// Dos listas son dos productores: las opciones del probador y las de la capa se atan aqui,
    /// en el arco que las usa, como `instrumento_edad.rs` las ata por la edad.
    #[test]
    fn las_opciones_del_probador_son_las_de_la_capa() {
        assert_eq!(banda::opciones(), crate::proof_options());
    }
}
