//! # OpenRPC del protocolo — generado DESDE este crate (nota 74, Fase 1).
//!
//! La tabla de metodos vive AQUI, junto a los DTOs que describe: una
//! sola fuente. `gen_openrpc` (src/bin) la vuelca a `spec/openrpc.json`;
//! una herramienta o una SEGUNDA implementacion la consume sin leer el
//! codigo del nodo. v0 deliberadamente conciso: nombre, resumen,
//! parametros y resultado con esquemas por referencia a los tipos de
//! `spec/RPC.md` (Q, DATA, Digest) — el contraste campo a campo lo dan
//! los vectores de conformidad, no este documento.

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
        "dev_fund",
        "dev_openSeeded",
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
        m("dev_fund", "SOLO --dev: emision delegada con custodios de PRUEBA.",
          json!([p("index", "Q"), p("amount", "Q")]), "Applied"),
        m("dev_openSeeded", "SOLO --dev: abre desde una clave determinista de la suite.",
          json!([p("seed", "Q")]), "Opened"),
    ];
    json!({
        "openrpc": "1.2.6",
        "info": {
            "title": "ZK-SSL JSON-RPC",
            "version": "zkssl/0.2",
            "description": "Especificacion normativa: spec/RPC.md. Principio que el API preserva: la clave de gasto no viaja jamas."
        },
        "methods": methods,
        "components": { "schemas": {
            "Q": { "type": "string", "pattern": "^0x[0-9a-f]+$",
                   "description": "u64 en hex, sin ceros a la izquierda" },
            "DATA": { "type": "string", "pattern": "^0x([0-9a-f][0-9a-f])*$" },
            "Digest": { "type": "string", "pattern": "^0x[0-9a-f]{64}$",
                        "description": "32 bytes: la MISMA serializacion que persiste la capa (store::digest_to_bytes)" },
            "ProtocolVersion": { "type": "string", "const": "zkssl/0.2" }
        } }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn veintidos_metodos_unicos_y_en_orden() {
        // §223: subio a 18 con `zkssl_applyMany`. §242: a 19 con
        // `zkssl_signedEpochHead`. §259: a 20 con
        // `zkssl_inclusionReceipt`. Que este test tenga el numero en el
        // nombre es a proposito: renombrarlo OBLIGA A MIRAR.
        // §275: a 21 con `zkssl_ackPath`.
        // §293 lo sirvio y NO lo publico; §302: a 22 con `zkssl_consistencyProof`.
        let nombres = method_names();
        assert_eq!(nombres.len(), 22);
        let mut u = nombres.clone();
        u.sort();
        u.dedup();
        assert_eq!(u.len(), 22, "nombres repetidos");
        let doc = document();
        let met = doc["methods"].as_array().expect("methods");
        assert_eq!(met.len(), 22);
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
    }

    #[test]
    fn el_documento_declara_version_y_esquemas() {
        let doc = document();
        assert_eq!(doc["openrpc"], "1.2.6");
        assert_eq!(doc["info"]["version"], "zkssl/0.2");
        assert!(doc["components"]["schemas"]["Digest"].is_object());
    }
}
