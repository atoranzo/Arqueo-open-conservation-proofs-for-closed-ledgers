//! El publicador del medio contra testigos LOCALES (RFC-0013 E3a, §808): lee por la entrada
//! estándar un JSON con las hojas del medio, la semilla del publicador y los testigos, firma la
//! nota, pide a cada testigo su cofirma con `zk_ssl_medio::testigos` y escribe por la salida
//! estándar lo que pasó, la nota cofirmada y cada intercambio HTTP tal cual.
//!
//! ```text
//! { "origen": "zkssl/v1/…", "hojas": ["0x…", …],
//!   "publicador": { "semilla": "0x…", "marca": 1790000000 },
//!   "testigos": [ { "vkey": "…", "prefijo": "http://127.0.0.1:…", "viejo": 0 },
//!                 { "nombre": "…", "semilla": "0x…", "prefijo": "…", "viejo": 0 }, … ] }
//! ```
//!
//! Un testigo va por su `vkey` `0x06`, o por un nombre y una semilla de los que sale la `vkey`
//! en la que el cliente confía: así se le habla a litewitness, que cofirma con Ed25519, con una
//! clave suya que no tiene.
//!
//! ⚠️ **Es el cliente de `tests/vectores/testigos/rehacer.sh`, no el publicador de un
//! operador**: la semilla viaja en claro por la entrada estándar, y la nota se firma en
//! DETERMINISTA para que `tests/testigos_torchwood.rs` rehaga byte a byte cada petición que se
//! grabó. Un publicador firma con sal (FIPS 204, la variante recomendada).

use std::io::Read;

use serde_json::{json, Value};
use zk_ssl_medio::medio::ArbolDelMedio;
use zk_ssl_medio::nota::{Checkpoint, Cofirmante, Publicador};
use zk_ssl_medio::testigos::{
    pedir_cofirma, ErrorDeTestigo, HttpLocal, RespuestaHttp, Resultado, Retencion, Testigo,
    Transporte,
};

fn hex32(v: &Value, que: &str) -> [u8; 32] {
    let s = v.as_str().unwrap_or_else(|| panic!("{que}: no es cadena"));
    let h = s
        .strip_prefix("0x")
        .unwrap_or_else(|| panic!("{que}: sin 0x"));
    assert_eq!(h.len(), 64, "{que}: 32 bytes");
    // §650: sobre bytes, nunca troceando el `&str` (la puerta de zk-ssl-hash).
    let c = |b: u8| (b as char).to_digit(16).expect(que) as u8;
    core::array::from_fn(|i| c(h.as_bytes()[2 * i]) << 4 | c(h.as_bytes()[2 * i + 1]))
}

/// El transporte local, y lo que pasó por él.
struct Grabado {
    http: HttpLocal,
    intercambios: Vec<Value>,
}

impl Transporte for Grabado {
    fn add_checkpoint(
        &mut self,
        testigo: &Testigo,
        cuerpo: &[u8],
    ) -> Result<RespuestaHttp, ErrorDeTestigo> {
        let r = self.http.add_checkpoint(testigo, cuerpo)?;
        self.intercambios.push(json!({
            "peticion": String::from_utf8_lossy(cuerpo),
            "estado": r.estado,
            "tipo": r.tipo,
            "respuesta": String::from_utf8_lossy(&r.cuerpo),
        }));
        Ok(r)
    }
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
    let checkpoint = Checkpoint {
        origen: origen.to_string(),
        tamano: arbol.tamano(),
        raiz: arbol.raiz(),
    };
    let p = &e["publicador"];
    let publicador =
        Publicador::determinista(origen, hex32(&p["semilla"], "semilla del publicador")).unwrap();
    let nota = publicador
        .firmar(&checkpoint, p["marca"].as_u64().expect("marca"))
        .unwrap();
    let k = publicador.clave_publica();
    let mut retencion = Retencion::nueva(&nota, k).unwrap();
    let mut salidas = Vec::new();
    for t in e["testigos"].as_array().expect("testigos") {
        let vkey = match t["vkey"].as_str() {
            Some(v) => v.to_string(),
            None => Cofirmante::determinista(
                t["nombre"].as_str().expect("nombre"),
                hex32(&t["semilla"], "semilla del testigo"),
            )
            .unwrap()
            .clave_publica()
            .vkey(),
        };
        let testigo = Testigo::nuevo(&vkey, t["prefijo"].as_str().expect("prefijo")).unwrap();
        let mut g = Grabado {
            http: HttpLocal::default(),
            intercambios: Vec::new(),
        };
        let viejo = t["viejo"].as_u64().expect("viejo");
        let mut s = json!({ "vkey": vkey });
        match pedir_cofirma(&mut g, &testigo, k, &nota, &arbol, viejo) {
            Ok(Resultado::Cofirmada {
                cofirma,
                viejo,
                ignoradas,
            }) => {
                let retenida = retencion.retener(testigo.clave(), &cofirma).unwrap();
                s["resultado"] = json!("cofirmada");
                s["viejo"] = json!(viejo);
                s["ignoradas"] = json!(ignoradas);
                s["marca"] = json!(cofirma.marca);
                s["retenida"] = json!(retenida);
            }
            Ok(Resultado::SinSuCofirma { viejo, ignoradas }) => {
                s["resultado"] = json!("sin-su-cofirma");
                s["viejo"] = json!(viejo);
                s["ignoradas"] = json!(ignoradas);
            }
            Ok(Resultado::Retroceso { recordado, nota }) => {
                s["resultado"] = json!("retroceso");
                s["recordado"] = json!(recordado);
                s["nota"] = json!(nota);
            }
            Ok(Resultado::Rechazada(r)) => {
                s["resultado"] = json!("rechazada");
                s["respuesta"] = json!(format!("{r:?}"));
            }
            Err(err) => {
                s["resultado"] = json!("error");
                s["error"] = json!(err.to_string());
            }
        }
        s["intercambios"] = json!(g.intercambios);
        salidas.push(s);
    }
    println!(
        "{}",
        json!({
            "publicador": k.vkey(),
            "tamano": arbol.tamano(),
            "nota": nota,
            "nota_cofirmada": retencion.nota_cofirmada(),
            "testigos": salidas,
        })
    );
}
