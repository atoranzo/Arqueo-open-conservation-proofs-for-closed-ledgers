//! El medio de un banco (RFC-0013 E4b, §634): lee por la entrada estándar un JSON con las
//! hojas del medio y las claves de prueba, y escribe por la salida estándar la nota firmada,
//! las vkeys y el camino de inclusión de una posición.
//!
//! ```text
//! { "origen": "zkssl/v1/…", "hojas": ["0x…", …], "posicion": 1,
//!   "publicador": { "semilla": "0x…", "marca": 1790000000 },
//!   "testigos": [ { "nombre": "…", "semilla": "0x…", "marca": 1790000060 }, … ] }
//! ```
//!
//! ⚠️ **No es el publicador** (eso es E3): es lo que `tools/banco_ancla_cofirmada.sh` necesita
//! para armar sobres del kit con una nota de verdad. Firma CON SAL, como firmaría el publicador,
//! y cada testigo cofirma con su propia clave: en el medio los testigos son ajenos y no corren
//! este código. Las semillas son de prueba y viajan en claro por la entrada estándar.

use std::io::Read;

use serde_json::{json, Value};
use zk_ssl_medio::medio::ArbolDelMedio;
use zk_ssl_medio::nota::{Checkpoint, Cofirmante, Publicador};

fn hex32(v: &Value, que: &str) -> [u8; 32] {
    let s = v.as_str().unwrap_or_else(|| panic!("{que}: no es cadena"));
    let h = s
        .strip_prefix("0x")
        .unwrap_or_else(|| panic!("{que}: sin 0x"));
    assert_eq!(h.len(), 64, "{que}: 32 bytes");
    core::array::from_fn(|i| u8::from_str_radix(&h[2 * i..2 * i + 2], 16).expect(que))
}

fn hex(b: &[u8]) -> String {
    format!(
        "0x{}",
        b.iter().map(|x| format!("{x:02x}")).collect::<String>()
    )
}

fn main() {
    let mut entrada = String::new();
    std::io::stdin().read_to_string(&mut entrada).unwrap();
    let e: Value = serde_json::from_str(&entrada).expect("JSON de entrada");
    let origen = e["origen"].as_str().expect("origen");
    let mut arbol = ArbolDelMedio::nuevo();
    for h in e["hojas"].as_array().expect("hojas") {
        arbol.anadir(&hex32(h, "hoja"));
    }
    let posicion = e["posicion"].as_u64().expect("posicion");
    let checkpoint = Checkpoint {
        origen: origen.to_string(),
        tamano: arbol.tamano(),
        raiz: arbol.raiz(),
    };
    let p = &e["publicador"];
    let publicador =
        Publicador::desde_semilla(origen, hex32(&p["semilla"], "semilla del publicador")).unwrap();
    let mut nota = publicador
        .firmar(&checkpoint, p["marca"].as_u64().expect("marca"))
        .unwrap();
    let mut vkeys = Vec::new();
    for t in e["testigos"].as_array().cloned().unwrap_or_default() {
        let testigo = Cofirmante::desde_semilla(
            t["nombre"].as_str().expect("nombre"),
            hex32(&t["semilla"], "semilla del testigo"),
        )
        .unwrap();
        nota += &testigo
            .cofirmar(&checkpoint, t["marca"].as_u64().expect("marca"))
            .unwrap();
        vkeys.push(testigo.clave_publica().vkey());
    }
    let inclusion: Vec<String> = arbol
        .prueba_de_inclusion(posicion, arbol.tamano())
        .unwrap()
        .iter()
        .map(|h| hex(h))
        .collect();
    println!(
        "{}",
        json!({
            "nota": nota,
            "publicador": publicador.clave_publica().vkey(),
            "testigos": vkeys,
            "inclusion": inclusion,
            "tamano": arbol.tamano(),
            "raiz": hex(&arbol.raiz()),
        })
    );
}
