//! # OpenRPC del protocolo — generado DESDE este crate (nota 74, Fase 1).
//!
//! La tabla de metodos vive AQUI, junto a los DTOs que describe: una
//! sola fuente. `gen_openrpc` (src/bin) la vuelca a `spec/openrpc.json`;
//! una herramienta o una SEGUNDA implementacion la consume sin leer el
//! codigo del nodo. v0 deliberadamente conciso: nombre, resumen,
//! parametros y resultado con esquemas por referencia a los tipos de
//! `spec/RPC.md` (Q, DATA, Digest) — el contraste campo a campo lo dan
//! los vectores de conformidad, no este documento.
//!
//! **Toda referencia RESUELVE (§585, entrada 95 del BACKLOG).** Hasta el §585 el
//! documento referenciaba 35 esquemas y declaraba 5: un validador OpenRPC lo
//! habria rechazado. Ahora cada nombre referenciado tiene su esquema -un puntero a
//! `spec/RPC.md`, con el DTO del cable que lo tipa cuando lo hay y los metodos que
//! lo usan-, y un test exige que lo referenciado y lo declarado sean lo MISMO. Lo
//! que el documento sigue sin dar, por diseno: la FORMA campo a campo.

use std::collections::BTreeMap;

use serde_json::{json, Value};

/// Los metodos del protocolo, en el ORDEN DE INCORPORACION, con los dos
/// `dev_*` cerrando la lista.
///
/// > « Los metodos del protocolo, en el orden de `spec/RPC.md`. »
///
/// Esa linea dejo de ser cierta y se CITA, no se borra (247). MEDIDO en
/// el 302: `spec/RPC.md` lista `inclusionReceipt`, `ackPath` y
/// `consistencyProof` ANTES de `openAccount`, y aqui van DESPUES de
/// `applyMany`. Donde vive la verdad: en el registro cronologico del
/// test de mas abajo (223 -> 242 -> 259 -> 275 -> 293/302), que crece
/// con cada metodo; no en un censo nuevo que envejezca igual.
pub fn method_names() -> Vec<&'static str> {
    vec![
        "zkssl_protocolVersion",
        "zkssl_params",
        "zkssl_epochHead",
        "zkssl_supply",
        "zkssl_accountCount",
        "zkssl_publicId",
        "zkssl_accountView",
        "zkssl_logEntry",
        "zkssl_logEntries",
        "zkssl_verifyChain",
        "zkssl_openAccount",
        "zkssl_sendMaterials",
        "zkssl_applySend",
        "zkssl_claimMaterials",
        "zkssl_applyClaim",
        "zkssl_applyMany",
        "zkssl_signedEpochHead",
        "zkssl_inclusionReceipt",
        "zkssl_ackPath",
        "zkssl_consistencyProof",
        "zkssl_submitCosig",
        "zkssl_cosigs",
        "zkssl_publishConsumo",
        "zkssl_consumoPath",
        "zkssl_frozenPath",
        "zkssl_pendingPath",
        "zkssl_pledge",
        "zkssl_recepPath",
        "dev_fund",
        "dev_openSeeded",
        "dev_freeze",
    ]
}

fn p(name: &str, tipo: &str) -> Value {
    json!({ "name": name, "required": true,
            "schema": { "$ref": format!("#/components/schemas/{tipo}") } })
}

fn p_opt(name: &str, tipo: &str) -> Value {
    json!({ "name": name, "required": false,
            "schema": { "$ref": format!("#/components/schemas/{tipo}") } })
}

fn m(name: &str, summary: &str, params: Value, result_tipo: &str) -> Value {
    json!({ "name": name, "summary": summary, "params": params,
            "result": { "name": "result",
                        "schema": { "$ref": format!("#/components/schemas/{result_tipo}") } } })
}

/// **Los esquemas que el documento declara por referencia (§585).** Cada nombre que un metodo
/// referencia y que no es un tipo base (`Q`, `DATA`, `Digest`, `ProtocolVersion`, `Bool`) tiene
/// aqui su fila, con el DTO de este crate que lo tipa cuando lo hay. Los emparejamientos se
/// MIDIERON contra el nodo -el tipo que usa el manejador de cada metodo-; los que dicen `None` los
/// compone el nodo con `json!`, y `BatchOp` tambien: su `OpDto` vive en el nodo, no en el cable.
/// Un metodo nuevo con un nombre nuevo tumba el test hasta que alguien le ponga fila.
fn esquemas_por_referencia() -> Vec<(&'static str, Option<&'static str>)> {
    vec![
        ("AccountView", Some("AccountViewDto")),
        ("AckPath", None),
        ("Applied", None),
        ("BatchApplied", None),
        ("BatchOp", None),
        ("ClaimMaterials", Some("ClaimMaterialsDto")),
        ("ClaimReceipt", Some("ClaimReceiptDto")),
        ("ClientState", Some("ClientStateDto")),
        ("ConsistencyProof", None),
        ("ConsumoPath", None),
        ("ConsumoPublicado", None),
        ("Cosig", Some("CofirmaDto")),
        ("CosigAccepted", None),
        ("Cosigs", None),
        ("EpochHead", Some("EpochHeadDto")),
        ("FrozenPath", None),
        ("InclusionReceipt", Some("InclusionReceiptDto")),
        ("LogEntries", None),
        ("LogEntry", Some("LogEntryDto")),
        ("Opened", None),
        ("Params", Some("ParamsDto")),
        ("PendingNotice", Some("PendingNoticeDto")),
        ("PendingPath", None),
        ("PrendaPublicada", None),
        ("RecepPath", None),
        ("SendMaterials", Some("SendMaterialsDto")),
        ("SendReceipt", Some("SendReceiptDto")),
        ("SignedEpochHead", Some("SignedEpochHeadDto")),
        ("Supply", None),
        ("VerifyChain", None),
    ]
}

/// Los nombres de esquema que `v` referencia (`#/components/schemas/<nombre>`), en orden de
/// aparicion y con repeticiones.
fn referencias_en(v: &Value, fuera: &mut Vec<String>) {
    match v {
        Value::Object(o) => {
            if let Some(Value::String(r)) = o.get("$ref") {
                if let Some(n) = r.strip_prefix("#/components/schemas/") {
                    fuera.push(n.to_string());
                }
            }
            for x in o.values() {
                referencias_en(x, fuera);
            }
        }
        Value::Array(a) => {
            for x in a {
                referencias_en(x, fuera);
            }
        }
        _ => {}
    }
}

pub fn document() -> Value {
    let methods = vec![
        m("zkssl_protocolVersion", "Version del protocolo.", json!([]), "ProtocolVersion"),
        m("zkssl_params", "Parametros inmutables del ledger.", json!([]), "Params"),
        m("zkssl_epochHead", "Cabeza de epoca: seq, raices y digests.", json!([]), "EpochHead"),
        m("zkssl_supply", "Suministro total y en transito.", json!([]), "Supply"),
        m("zkssl_accountCount", "Numero de cuentas abiertas.", json!([]), "Q"),
        m("zkssl_publicId", "Id publico de una cuenta por indice.",
          json!([p("index", "Q")]), "Digest"),
        m("zkssl_accountView", "Vista AUTENTICADA: exige la clave de VISTA (49-A).",
          json!([p("index", "Q"), p("viewKey", "Digest")]), "AccountView"),
        m("zkssl_logEntry", "Una entrada del registro encadenado.",
          json!([p("seq", "Q")]), "LogEntry"),
        m("zkssl_logEntries", "Entradas desde fromSeq (limite <= 1000).",
          json!([p_opt("fromSeq", "Q"), p_opt("limit", "Q")]), "LogEntries"),
        m("zkssl_verifyChain", "Reverifica la cadena completa del registro.",
          json!([]), "VerifyChain"),
        m("zkssl_openAccount", "Abre con ids DERIVADOS: la clave de gasto no viaja.",
          json!([p("publicId", "Digest"), p("viewId", "Digest"), p("leafSalt", "Digest")]),
          "Opened"),
        m("zkssl_sendMaterials", "Materiales publicos para probar el envio EN LOCAL.",
          json!([p("sender", "Q"), p("viewKey", "Digest"), p("receiverId", "Digest"),
                 p("amount", "Q"), p("salt", "Digest")]), "SendMaterials"),
        m("zkssl_applySend", "Aplica un recibo de envio verificando su prueba STARK.",
          json!([p("receipt", "SendReceipt"), p("sender", "Q"),
                 p("senderState", "ClientState"), p("amount", "Q")]), "Applied"),
        m("zkssl_claimMaterials", "Materiales publicos para probar el cobro EN LOCAL.",
          json!([p("receiver", "Q"), p("viewKey", "Digest"), p("notice", "PendingNotice")]), "ClaimMaterials"),
        m("zkssl_applyClaim", "Aplica un recibo de cobro verificando su prueba STARK.",
          json!([p("receipt", "ClaimReceipt"), p("receiver", "Q"),
                 p("receiverState", "ClientState"), p("notice", "PendingNotice")]),
          "Applied"),
        m("zkssl_applyMany", "Aplica N operaciones contra UNA raiz de arranque: todo o nada.",
          json!([p("ops", "BatchOp")]), "BatchApplied"),
        m("zkssl_signedEpochHead",
          "La ULTIMA cabeza de epoca firmada, para un TESTIGO. Aditivo: no toca zkssl_epochHead.",
          json!([]), "SignedEpochHead"),
        m("zkssl_inclusionReceipt",
          "Recibo de inclusion de una cuenta: hoja, camino y cabeza. leafFormat es OBSERVADO.",
          json!([p("index", "Q"), p("viewKey", "Digest")]), "InclusionReceipt"),
        m("zkssl_ackPath",
          "Camino de acuse de una epoca CERRADA. La cabeza NO viaja: se verifica contra la custodiada.",
          json!([p("seq", "Q")]), "AckPath"),
        m("zkssl_consistencyProof",
          "Prueba de consistencia del MMR entre un tamano antiguo y la cima actual (eslabon 2 como SERVICIO).",
          json!([p("oldSize", "Q")]), "ConsistencyProof"),
        m("zkssl_submitCosig",
          "Submision de una cofirma de testigo para la epoca EN CURSO. El nodo VERIFICA la firma y deduplica por clave de testigo; NO acredita al testigo: eso es politica del cliente.",
          json!([p("cosig", "Cosig")]), "CosigAccepted"),
        m("zkssl_cosigs",
          "Las cofirmas que el nodo tiene de una epoca. Sin parametro, la epoca en curso. El nodo NO guarda historico: otra epoca da cero.",
          json!([p_opt("epochDigest", "Digest")]), "Cosigs"),
        m("zkssl_publishConsumo",
          "Publica un consumo (RFC-0006). SIN prueba y SIN autorizacion: quien publica primero bloquea, y eso es denegacion de servicio, no doble uso. El repetido y la colision se rechazan con nombre.",
          json!([p("consumo", "Digest")]), "ConsumoPublicado"),
        m("zkssl_consumoPath",
          "Camino de autenticacion de un consumo bajo la cabeza de ese seq. La cabeza NO viaja y la raiz tampoco: quien verifica elige la hoja (el consumo prueba presencia; el digest cero, ausencia).",
          json!([p("consumo", "Digest"), p("seq", "Q")]), "ConsumoPath"),
        m("zkssl_frozenPath",
          "Camino de la cuenta en el arbol de CONGELADOS, para su TITULAR (exige la clave de VISTA). Hoja vacia o no; s es el seq del estado. La profundidad la fija quien verifica (RFC-0007 E3b).",
          json!([p("index", "Q"), p("viewKey", "Digest")]), "FrozenPath"),
        m("zkssl_pendingPath",
          "Lo que el COBRADOR necesita de la FOTO del ultimo latido firmado (RFC-0008 D-F): el camino de su pendiente, los hermanos de su meta y la meta, en el seq s de esa cabeza. Exige la clave de VISTA del receptor y un aviso v2 que recomponga la hoja; si no, available false sin decir que hay. Con receiverId (RFC-0008 D-AE) quien pide es el PAGADOR: trae SU clave de VISTA, nombra al receptor y recibe lo mismo, solo si la meta de esa posicion le nombra.",
          json!([p("index", "Q"), p("viewKey", "Digest"), p("position", "Q"), p("salt", "Digest"),
                 p("amount", "Q"), p("x", "Digest"), p_opt("receiverId", "Digest")]), "PendingPath"),
        m("zkssl_pledge",
          "Publica la MARCA de una prenda (RFC-0008 E3) EXIGIENDO su sobre: el nodo compone el enunciado con la raiz de pendientes de SU cabeza firmada y verifica la prueba ANTES de escribir. NO es una puerta del arbol de consumos: la marca sola sigue entrando por `zkssl_publishConsumo`, que no pide nada (D-AT). NO exige credencial: la autorizacion es la prueba. Una marca ya publicada cuyo sobre verifica no es un fallo, y la respuesta lo dice.",
          json!([p("prueba", "DATA"), p("receptor", "Digest"), p("marca", "Digest"),
                 p("seq", "Q")]), "PrendaPublicada"),
        m("zkssl_recepPath",
          "Camino del recibo de recepcion `rx` en la era CERRADA que lo contiene (RFC-0010 E3). La cabeza NO viaja, ni la raiz: el titular sube su hoja hasta la recepRoot de la cabeza que custodia, que la respuesta identifica por su recepCount y su index.",
          json!([p("rx", "Q")]), "RecepPath"),
        m("dev_fund", "SOLO --dev: emision delegada con custodios de PRUEBA.",
          json!([p("index", "Q"), p("amount", "Q")]), "Applied"),
        m("dev_openSeeded", "SOLO --dev: abre desde una clave determinista de la suite.",
          json!([p("seed", "Q")]), "Opened"),
        m("dev_freeze", "SOLO --dev: congelacion delegada con custodios de PRUEBA.",
          json!([p("index", "Q"), p("frozen", "Bool")]), "Applied"),
    ];
    let mut schemas = json!({
        "Q": { "type": "string", "pattern": "^0x[0-9a-f]+$",
               "description": "u64 en hex, sin ceros a la izquierda" },
        "DATA": { "type": "string", "pattern": "^0x([0-9a-f][0-9a-f])*$" },
        "Digest": { "type": "string", "pattern": "^0x[0-9a-f]{64}$",
                    "description": "32 bytes: la MISMA serializacion que persiste la capa (store::digest_to_bytes)" },
        "ProtocolVersion": { "type": "string", "const": "zkssl/0.4" },
        "Bool": { "type": "boolean" }
    });
    // Quien usa cada esquema, DERIVADO de la tabla de metodos: no se escribe a mano.
    let mut usos: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for mm in &methods {
        let nombre = mm["name"].as_str().expect("name").to_string();
        let mut r = Vec::new();
        referencias_en(mm, &mut r);
        for n in r {
            let e = usos.entry(n).or_default();
            if !e.contains(&nombre) {
                e.push(nombre.clone());
            }
        }
    }
    for (nombre, dto) in esquemas_por_referencia() {
        let cable = match dto {
            Some(t) => format!("En el cable: zk_ssl_wire::{t}."),
            None => "Sin DTO en el cable: la compone el nodo.".to_string(),
        };
        let lo_usan = usos.get(nombre).map(|v| v.join(", ")).unwrap_or_default();
        schemas[nombre] = json!({ "description": format!(
            "Forma normativa: spec/RPC.md, que este documento no repite; la atan los vectores \
             de conformidad. {cable} Lo usan: {lo_usan}.") });
    }
    json!({
        "openrpc": "1.2.6",
        "info": {
            "title": "ZK-SSL JSON-RPC",
            "version": "zkssl/0.4",
            "description": "Especificacion normativa: spec/RPC.md. Principio del API: la clave de gasto no viaja jamas. Desde el asiento 538 (RFC-0009 E3b-2, zkssl/0.4) el probador oculta el testigo y la clave no sale literal en ninguna prueba con fila; entre el asiento 521 y el 538 las pruebas de envio y de cobro la publicaban (winterfell 0.13 no ocultaba el testigo; AUDITORIA.md, asientos 521 y 538)."
        },
        "methods": methods,
        "components": { "schemas": schemas }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn treinta_y_un_metodos_unicos_y_en_orden() {
        // §223: subio a 18 con `zkssl_applyMany`. §242: a 19 con
        // `zkssl_signedEpochHead`. §259: a 20 con
        // `zkssl_inclusionReceipt`. Que este test tenga el numero en el
        // nombre es a proposito: renombrarlo OBLIGA A MIRAR.
        // §275: a 21 con `zkssl_ackPath`.
        // §293 lo sirvio y NO lo publico; §302: a 22 con `zkssl_consistencyProof`.
        // §315: a 24 con `zkssl_submitCosig` y `zkssl_cosigs` — el TRANSPORTE
        // de la cofirma, que hasta hoy no existia: el testigo la escribia en
        // un fichero suyo y nadie mas la veia.
        // §417: a 26 con `zkssl_publishConsumo` y `zkssl_consumoPath` — el
        // consumo por el cable (RFC-0006, E3a).
        // §458: a 28 con `zkssl_frozenPath` y `dev_freeze` -el camino de
        // congelados para el titular (RFC-0007, E3b) y el grifo que congela
        // en el sandbox para poder capturarlo-.
        // §493: a 29 con `zkssl_pendingPath` -lo que el cobrador necesita de la
        // foto del ultimo latido firmado (RFC-0008, D-F)-.
        // §519: a 30 con `zkssl_pledge` -la marca de la prenda con su sobre, que el
        // nodo verifica ANTES de escribirla (RFC-0008, E3)-.
        // §571: a 31 con `zkssl_recepPath` -el camino del recibo de recepcion (RFC-0010, E3)-.
        let nombres = method_names();
        assert_eq!(nombres.len(), 31);
        let mut u = nombres.clone();
        u.sort();
        u.dedup();
        assert_eq!(u.len(), 31, "nombres repetidos");
        let doc = document();
        let met = doc["methods"].as_array().expect("methods");
        assert_eq!(met.len(), 31);
        for (i, mm) in met.iter().enumerate() {
            assert_eq!(mm["name"].as_str().unwrap(), nombres[i]);
        }
    }

    /// ⚠️ **`spec/openrpc.json` es un ARTEFACTO GENERADO, y llevaba
    /// RANCIO desde §242**: 18 metodos frente a los 19 de `document()`.
    /// La cabecera de este modulo dice «una sola fuente» y habia **dos
    /// copias sin nadie que las comparara** — el rito de §217 incumplido
    /// justo donde mas se afirma.
    ///
    /// Regenerar:
    /// `cargo run --release -p zk-ssl-wire --bin gen_openrpc > spec/openrpc.json`
    #[test]
    fn el_json_publicado_es_el_que_genera_esta_tabla() {
        let ruta = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../spec/openrpc.json");
        let publicado: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&ruta).expect("spec/openrpc.json"))
                .expect("json valido");
        assert_eq!(
            publicado, document(),
            "spec/openrpc.json NO es lo que genera esta tabla: regenerar con gen_openrpc"
        );
        // ⚠️ §585, la (d) de la entrada 95: comparar valores PARSEADOS es ciego a la FORMA -el
        // §302 midio que el artefacto llevo desde el §275 una clave fuera de orden sin que nada
        // lo viera-. Ahora tambien los BYTES: lo publicado es exactamente lo que `gen_openrpc`
        // imprime, con su salto de linea final.
        let texto = std::fs::read_to_string(&ruta).expect("spec/openrpc.json");
        let generado = format!(
            "{}\n",
            serde_json::to_string_pretty(&document()).expect("serializable")
        );
        assert!(
            texto == generado,
            "spec/openrpc.json dice lo mismo pero con OTRA FORMA: regenerar con gen_openrpc"
        );
    }

    #[test]
    fn el_documento_declara_version_y_esquemas() {
        let doc = document();
        assert_eq!(doc["openrpc"], "1.2.6");
        assert_eq!(doc["info"]["version"], "zkssl/0.4");
        assert!(doc["components"]["schemas"]["Digest"].is_object());
        // §585, la (a) y la (b) de la entrada 95: lo referenciado y lo declarado, IGUALES.
        let mut r = Vec::new();
        referencias_en(&doc["methods"], &mut r);
        let refs: std::collections::BTreeSet<String> = r.into_iter().collect();
        let decl: std::collections::BTreeSet<String> =
            doc["components"]["schemas"].as_object().expect("schemas").keys().cloned().collect();
        let cuelgan: Vec<_> = refs.difference(&decl).collect();
        assert!(cuelgan.is_empty(), "referencias SIN esquema declarado: {cuelgan:?}");
        let sobran: Vec<_> = decl.difference(&refs).collect();
        assert!(sobran.is_empty(), "esquemas que NADIE referencia: {sobran:?}");
        // Y cada DTO que la tabla nombra es un tipo de este crate: el nombre, contra el tipo.
        let tipos = [
            std::any::type_name::<crate::AccountViewDto>(),
            std::any::type_name::<crate::ClaimMaterialsDto>(),
            std::any::type_name::<crate::ClaimReceiptDto>(),
            std::any::type_name::<crate::ClientStateDto>(),
            std::any::type_name::<crate::CofirmaDto>(),
            std::any::type_name::<crate::EpochHeadDto>(),
            std::any::type_name::<crate::InclusionReceiptDto>(),
            std::any::type_name::<crate::LogEntryDto>(),
            std::any::type_name::<crate::ParamsDto>(),
            std::any::type_name::<crate::PendingNoticeDto>(),
            std::any::type_name::<crate::SendMaterialsDto>(),
            std::any::type_name::<crate::SendReceiptDto>(),
            std::any::type_name::<crate::SignedEpochHeadDto>(),
        ];
        let nombrados: Vec<&str> =
            esquemas_por_referencia().into_iter().filter_map(|(_, d)| d).collect();
        assert_eq!(nombrados.len(), tipos.len(), "un DTO nombrado sin su tipo, o al reves");
        for d in nombrados {
            assert!(
                tipos.iter().any(|t| t.rsplit("::").next() == Some(d)),
                "{d} no es un tipo de zk_ssl_wire"
            );
        }
    }
}
