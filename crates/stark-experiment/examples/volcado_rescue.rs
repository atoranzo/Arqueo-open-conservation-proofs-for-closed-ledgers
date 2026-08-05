//! Volcado de las constantes de Rescue-Prime (Rp64_256) a JSON — FV-2 (§190).
//!
//! La aritmética AIR la canta winterfell (rito): el exportador SMT
//! (`doc/fv/fv2_exporta_smt.py`) consume este JSON en vez de copiar
//! números a mano. Uso:
//!   cargo run --release -p stark-experiment --example volcado_rescue \
//!       > /tmp/rescue_ctes.json

use winterfell::crypto::hashers::Rp64_256;
use winterfell::math::{fields::f64::BaseElement, StarkField};

fn fila(v: &[BaseElement]) -> String {
    let xs: Vec<String> = v.iter().map(|e| e.as_int().to_string()).collect();
    format!("[{}]", xs.join(","))
}

fn matriz(m: &[[BaseElement; 12]]) -> String {
    let fs: Vec<String> = m.iter().map(|f| fila(f)).collect();
    format!("[{}]", fs.join(",\n "))
}

fn main() {
    println!("{{");
    println!("\"p\": {},", BaseElement::MODULUS);
    println!("\"MDS\": {},", matriz(&Rp64_256::MDS));
    println!("\"INV_MDS\": {},", matriz(&Rp64_256::INV_MDS));
    println!("\"ARK1\": {},", matriz(&Rp64_256::ARK1));
    println!("\"ARK2\": {}", matriz(&Rp64_256::ARK2));
    println!("}}");
}
