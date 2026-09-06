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

/// **MUDADOS al nucleo en el S416** (`zk-ssl-hash`): un verificador que no
/// compila la capa tiene que recomponer la posicion para subir el camino.
/// Se re-exportan aqui para que el nombre siga viviendo donde se usa; el
/// PRODUCTOR es uno solo, y esta en el nucleo.
pub use zk_ssl_hash::{posicion_de_consumo, CONS_DEPTH};

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

    /// El arbol de consumos **tal como lo firmo la cabeza de `seq_cabeza`**.
    ///
    /// Se RECONSTRUYE del registro: el consumo viaja como compromiso de su
    /// entrada (S413), asi que las entradas `OpKind::Consumo` con
    /// `entrada.seq < seq_cabeza` son el conjunto exacto que esa cabeza
    /// acredito. No hay historico que guardar. Molde:
    /// `vista_acuses::camino_de_epoca` del nodo (S275), sin diario.
    ///
    /// **`None` si una entrada `Consumo` no lleva su compromiso.** Saltarla
    /// daria un arbol a medias y una raiz distinta SIN DECIR POR QUE: se para.
    fn consumos_hasta(&self, seq_cabeza: u64) -> Option<SparseTree> {
        let mut hojas: Vec<(u64, Digest)> = Vec::new();
        for e in self.log.entries() {
            if e.seq >= seq_cabeza || e.kind != OpKind::Consumo {
                continue;
            }
            let c = e.compromiso?;
            hojas.push((posicion_de_consumo(&c), c));
        }
        let mut t = SparseTree::with_depth(CONS_DEPTH);
        t.rebuild_from(hojas);
        Some(t)
    }

    /// **La raiz de consumos que firmo la cabeza de `seq_cabeza`, y el camino
    /// de autenticacion de `consumo` bajo ella.** `None` si esa cabeza no
    /// existe todavia, o si el registro no permite reconstruir el tramo.
    ///
    /// Sirve para las DOS direcciones y por eso NO son dos funciones:
    /// `path_for` da el mismo camino para una posicion ocupada y para una
    /// libre, y **quien verifica elige la hoja** - el digest del consumo
    /// prueba que esta bajo la raiz; el digest CERO prueba que no estaba.
    /// Esa pareja es lo que E3 empaqueta (presencia bajo la cabeza nueva,
    /// ausencia bajo la anterior), y se sube con `zk_ssl_hash::path_root`.
    ///
    /// **La raiz se devuelve a proposito**, como en `camino_de_epoca`:
    /// reconstruir y mantener son dos productores del mismo objeto, y quien
    /// llama tiene que poder cruzar lo reconstruido con lo que la cabeza
    /// FIRMO. Esta funcion no dice nada sobre esa firma: eso es del mando.
    pub fn cons_path(
        &self,
        seq_cabeza: u64,
        consumo: &Digest,
    ) -> Option<(Digest, MerklePath)> {
        if seq_cabeza > self.log.len() as u64 {
            return None;
        }
        let t = self.consumos_hasta(seq_cabeza)?;
        Some((t.root(), t.path_for(posicion_de_consumo(consumo))))
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

    /// S416: el camino de un consumo publicado SUBE hasta la raiz que la
    /// cabeza firma. Si esto fallara, ninguna prueba portable verificaria.
    #[test]
    fn rfc0006_el_camino_de_un_consumo_sube_a_la_raiz() {
        let mut layer = new_layer();
        let a = consumo(11);
        layer.apply_consumo(a).expect("a");
        layer.apply_consumo(consumo(12)).expect("b");
        let s = layer.transition_log().len() as u64;
        let (raiz, cam) = layer.cons_path(s, &a).expect("camino");
        assert_eq!(raiz, layer.cons_root(), "la raiz reconstruida es la viva");
        assert_eq!(
            zk_ssl_hash::path_root(a, &cam.siblings, &cam.is_right),
            layer.cons_root(),
            "CRITICO: el camino de un consumo publicado tiene que dar la raiz firmada"
        );
    }

    /// S416: el camino de un consumo que NO esta sube hasta la MISMA raiz
    /// con la hoja CERO. Es la no-pertenencia, y es la mitad de E3.
    #[test]
    fn rfc0006_la_ausencia_sube_a_la_misma_raiz_con_la_hoja_cero() {
        let mut layer = new_layer();
        layer.apply_consumo(consumo(21)).expect("uno");
        let ausente = consumo(22);
        assert!(!layer.is_consumido(&ausente));
        let s = layer.transition_log().len() as u64;
        let (_, cam) = layer.cons_path(s, &ausente).expect("camino");
        let cero: Digest = [BaseElement::ZERO; 4];
        assert_eq!(
            zk_ssl_hash::path_root(cero, &cam.siblings, &cam.is_right),
            layer.cons_root(),
            "CRITICO: la hoja CERO tiene que subir a la raiz: es la NO-pertenencia"
        );
        assert_ne!(
            zk_ssl_hash::path_root(ausente, &cam.siblings, &cam.is_right),
            layer.cons_root(),
            "y el consumo ausente NO puede dar la raiz"
        );
    }

    /// S416, **el falsador de E3**: contra la cabeza ANTERIOR, un consumo
    /// publicado DESPUES no estaba - y el camino de entonces lo demuestra.
    #[test]
    fn rfc0006_bajo_la_cabeza_anterior_el_consumo_nuevo_no_estaba() {
        let mut layer = new_layer();
        layer.apply_consumo(consumo(31)).expect("previo");
        let s_vieja = layer.transition_log().len() as u64;
        let raiz_vieja = layer.cons_root();
        let nuevo = consumo(32);
        layer.apply_consumo(nuevo).expect("nuevo");
        let cero: Digest = [BaseElement::ZERO; 4];
        let (raiz_r, ausencia) = layer.cons_path(s_vieja, &nuevo).expect("camino viejo");
        assert_eq!(raiz_r, raiz_vieja, "el tramo reconstruido es el de ENTONCES");
        assert_eq!(
            zk_ssl_hash::path_root(cero, &ausencia.siblings, &ausencia.is_right),
            raiz_vieja,
            "CRITICO: bajo la cabeza anterior el consumo NO estaba"
        );
        let s_nueva = layer.transition_log().len() as u64;
        let (_, presencia) = layer.cons_path(s_nueva, &nuevo).expect("camino nuevo");
        assert_eq!(
            zk_ssl_hash::path_root(nuevo, &presencia.siblings, &presencia.is_right),
            layer.cons_root(),
            "y bajo la nueva SI esta"
        );
        assert_ne!(raiz_vieja, layer.cons_root(), "las dos raices son distintas");
    }

    /// S416: una cabeza que aun no existe no tiene camino, y se dice.
    #[test]
    fn rfc0006_una_seq_del_futuro_no_tiene_camino() {
        let layer = new_layer();
        let s = layer.transition_log().len() as u64;
        assert!(layer.cons_path(s, &consumo(41)).is_some());
        assert!(
            layer.cons_path(s + 1, &consumo(41)).is_none(),
            "una seq mayor que el registro no puede tener camino"
        );
    }

    /// S416, **el cruce**: reconstruir del registro y mantener el arbol vivo
    /// son DOS productores del mismo objeto. Este test es lo que los ata.
    #[test]
    fn rfc0006_la_raiz_reconstruida_es_la_raiz_viva() {
        let mut layer = new_layer();
        for n in 51..55 {
            layer.apply_consumo(consumo(n)).expect("publicar");
        }
        let s = layer.transition_log().len() as u64;
        let (raiz, _) = layer.cons_path(s, &consumo(51)).expect("camino");
        assert_eq!(
            raiz,
            layer.cons_root(),
            "CRITICO: reconstruir del registro tiene que dar la MISMA raiz que el arbol vivo"
        );
        assert_eq!(layer.cons_count(), 4);
    }
}
