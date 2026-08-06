//! Formateo de digests. Usa los serializadores del propio proyecto
//! (`zk_ssl::store::digest_to_bytes`) para no inventar otra
//! representación: lo que se imprime es byte a byte lo que se persiste.

use winterfell::math::fields::f64::BaseElement;
use zk_ssl::store::digest_to_bytes;

/// El mismo alias del proyecto: `stark_experiment::merkle::Digest`.
pub type Digest = [BaseElement; 4];

pub fn hex(d: &Digest) -> String {
    digest_to_bytes(d).iter().map(|b| format!("{b:02x}")).collect()
}

/// `8+8` extremos, suficiente para seguir una traza a ojo.
pub fn hex_short(d: &Digest) -> String {
    let h = hex(d);
    format!("{}…{}", &h[..8], &h[h.len() - 8..])
}
