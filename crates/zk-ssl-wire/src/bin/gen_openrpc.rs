//! Vuelca el documento OpenRPC del protocolo por stdout.
//! Uso: cargo run --release -p zk-ssl-wire --bin gen_openrpc > spec/openrpc.json

fn main() {
    let doc = zk_ssl_wire::openrpc::document();
    println!("{}", serde_json::to_string_pretty(&doc).expect("serializable"));
}
