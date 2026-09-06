//! El consumo publicado (RFC-0006, E1).
//!
//! ## Que es
//!
//! Una etiqueta PUBLICA y PRECOMPUTABLE —`H(dominio, identificador acordado)`,
//! derivada fuera de la capa— que el libro acumula en un arbol disperso propio
//! y cuya raiz vive en reposo como `root:cons`, la SEXTA raiz que `load` exige
//! (§387 pendientes, §388 meta, §391 congelados, §392 registro; esta, §413).
//! Publicar el mismo consumo dos veces se rechaza con nombre.
//!
//! ## Lo que prueba y lo que NO
//!
//! Prueba exactamente esto: que un consumo esta bajo la raiz de la cabeza y
//! no estaba bajo la de una anterior. **No prueba que el consumo corresponda
//! a un pago**: lo que el circuito no restringe no existe, y aqui el circuito
//! no restringe nada (E5 del RFC-0006, fuera de este corte). Tampoco autentica
//! a quien publica: quien tiene acceso al nodo publica, y quien publica primero
//! bloquea. Es denegacion de servicio, no doble uso, y esta declarada (D-4).
//!
//! ## La posicion
//!
//! `SparseTree` se indexa por `u64`, asi que la posicion NO puede ser el digest
//! entero: son los 63 bits bajos de sus primeros ocho bytes (`CONS_DEPTH`).
//! La hoja SI es el digest entero, de modo que una colision de prefijo —dos
//! consumos distintos con la misma posicion, probabilidad 2^-63 por pareja— se
//! DETECTA y se rechaza con su propio nombre, no se confunde con un repetido.
//! Es la clase de limite que `nullifier_tree.rs` declara en su cabecera, con
//! 63 bits en vez de los bits bajos de un digest de circuito.

use super::*;

/// Profundidad del arbol de consumos: 63 bits de posicion. `1u64 << 63` es
/// la ultima capacidad representable en `u64`.
pub const CONS_DEPTH: usize = 63;

/// Posicion de un consumo en el arbol: los 63 bits bajos de sus primeros
/// ocho bytes (little-endian, la serializacion de la capa).
pub fn posicion_de_consumo(consumo: &Digest) -> u64 {
    let b = digest_to_bytes(consumo);
    let mut ocho = [0u8; 8];
    ocho.copy_from_slice(&b[0..8]);
    u64::from_le_bytes(ocho) & (u64::MAX >> 1)
}

impl SovereignLayer {
    /// Raiz del arbol de consumos. Publica: es la sexta raiz en reposo y, con
    /// E2 del RFC-0006, entrara firmada en la cabeza v4.
    pub fn cons_root(&self) -> Digest {
        self.consumos.root()
    }

    /// Cuantos consumos hay publicados (la `k` de la cabeza v4).
    pub fn cons_count(&self) -> u64 {
        self.consumos_orden.len() as u64
    }

    /// Si un consumo ya esta publicado (la hoja de su posicion es EL consumo).
    pub fn is_consumido(&self, consumo: &Digest) -> bool {
        let pos = posicion_de_consumo(consumo);
        self.consumos.is_occupied(pos) && self.consumos.leaf(pos) == *consumo
    }

    /// Publica un consumo.
    ///
    /// Sin prueba y sin autorizacion (D-4 del RFC-0006, declarado). Un consumo
    /// ya publicado se rechaza con `ConsumoRepetido`; una posicion ocupada por
    /// OTRO consumo se rechaza con `ConsumoColision`. La raiz de cuentas no se
    /// mueve: el registro la repite a ambos lados y lleva el consumo como
    /// compromiso de la entrada (`OpKind::Consumo`), asi que la cadena del
    /// registro tambien lo ata.
    pub fn apply_consumo(&mut self, consumo: Digest) -> Result<(), LayerError> {
        let pos = posicion_de_consumo(&consumo);
        if self.consumos.is_occupied(pos) {
            let ocupante = self.consumos.leaf(pos);
            if ocupante == consumo {
                return Err(LayerError::ConsumoRepetido { consumo });
            }
            return Err(LayerError::ConsumoColision { consumo, ocupante });
        }
        self.consumos.set_leaf(pos, consumo);
        self.consumos_orden.push((pos, consumo));
        let raiz = self.accounts.root();
        self.log
            .append_con_compromiso(OpKind::Consumo, raiz, raiz, &[], consumo);
        self.commit(&[], None)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests_support::*;

    fn consumo(n: u64) -> Digest {
        [
            BaseElement::new(n),
            BaseElement::new(7),
            BaseElement::new(11),
            BaseElement::new(13),
        ]
    }

    /// Un consumo publicado cambia la raiz, cuenta uno, se ve como consumido
    /// y deja su entrada en el registro con el consumo como compromiso.
    #[test]
    fn rfc0006_un_consumo_se_publica_y_la_raiz_cambia() {
        let mut layer = new_layer();
        let vacia = layer.cons_root();
        let n = layer.log_head();
        let c = consumo(1);
        assert!(!layer.is_consumido(&c));
        layer.apply_consumo(c).expect("publicar");
        assert_ne!(layer.cons_root(), vacia, "la raiz de consumos debe moverse");
        assert_eq!(layer.cons_count(), 1);
        assert!(layer.is_consumido(&c));
        assert_ne!(layer.log_head(), n, "el registro debe llevar la entrada");
        let ultima = layer.transition_log().entries().last().expect("entrada").clone();
        assert_eq!(ultima.kind, OpKind::Consumo);
        assert_eq!(ultima.compromiso, Some(c), "el consumo viaja como compromiso");
        assert_eq!(ultima.root_old, ultima.root_new, "la raiz de cuentas no se mueve");
    }

    /// El mismo consumo dos veces: la segunda se rechaza con nombre y nada
    /// se mueve (ni raiz, ni cuenta, ni registro).
    #[test]
    fn rfc0006_un_consumo_repetido_se_rechaza() {
        let mut layer = new_layer();
        let c = consumo(2);
        layer.apply_consumo(c).expect("la primera vez");
        let raiz = layer.cons_root();
        let n = layer.log_head();
        let r = layer.apply_consumo(c);
        assert!(
            matches!(&r, Err(LayerError::ConsumoRepetido { consumo }) if *consumo == c),
            "CRITICO: un consumo repetido debe rechazarse con nombre: {r:?}"
        );
        assert_eq!(layer.cons_root(), raiz, "la raiz no se mueve en un rechazo");
        assert_eq!(layer.cons_count(), 1);
        assert_eq!(layer.log_head(), n, "un rechazo no escribe en el registro");
    }

    /// Dos consumos distintos conviven, cada uno en su posicion.
    #[test]
    fn rfc0006_dos_consumos_distintos_conviven() {
        let mut layer = new_layer();
        let a = consumo(3);
        let b = consumo(4);
        assert_ne!(posicion_de_consumo(&a), posicion_de_consumo(&b));
        layer.apply_consumo(a).expect("a");
        layer.apply_consumo(b).expect("b");
        assert_eq!(layer.cons_count(), 2);
        assert!(layer.is_consumido(&a) && layer.is_consumido(&b));
    }

    /// Una colision de prefijo (misma posicion, otro digest) NO es un
    /// repetido: se rechaza con su propio nombre y el ocupante.
    #[test]
    fn rfc0006_una_colision_de_prefijo_se_rechaza_con_su_nombre() {
        let mut layer = new_layer();
        let a = consumo(5);
        // Mismo primer elemento -> misma posicion; el resto distinto.
        let b = [a[0], BaseElement::new(99), a[2], a[3]];
        assert_eq!(posicion_de_consumo(&a), posicion_de_consumo(&b));
        assert_ne!(a, b);
        layer.apply_consumo(a).expect("a");
        let r = layer.apply_consumo(b);
        assert!(
            matches!(&r, Err(LayerError::ConsumoColision { consumo, ocupante }) if *consumo == b && *ocupante == a),
            "CRITICO: una colision no es un repetido: {r:?}"
        );
        assert!(!layer.is_consumido(&b), "el colisionado no cuenta como consumido");
        assert_eq!(layer.cons_count(), 1);
    }

    /// La clase nueva del registro tiene tag propio (13) y hace la ida y vuelta.
    #[test]
    fn rfc0006_el_tag_del_consumo_hace_la_ida_y_vuelta() {
        assert_eq!(OpKind::Consumo.tag_byte(), 13);
        assert_eq!(OpKind::from_tag_byte(13), Some(OpKind::Consumo));
        assert_eq!(OpKind::from_tag_byte(14), None, "el 14 sigue libre");
    }
}
