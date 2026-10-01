//! Vectores de respuesta conocida (KAT) para XMSS^MT, producidos por `xmss 0.1.0-pre.0`.
//!
//! Para cada conjunto: una semilla fija de 3n = 96 bytes, la clave publica (OID || root || SEED) y
//! cuatro firmas consecutivas (indices 0 a 3) sobre mensajes de 0, 3, 50 y 1000 bytes, cada una en
//! las dos formas del crate: `sign` (firma || mensaje) y `sign_detached` (solo la firma). Las firmas
//! se autoverifican con el propio crate antes de imprimirse. Salida: JSON por stdout, en hex.
use xmss::{KeyPair, XmssMtSha2_20_2_256, XmssMtSha2_40_8_256, XmssParameter};

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn mensajes() -> Vec<Vec<u8>> {
    vec![
        vec![],
        b"abc".to_vec(),
        (0..50u32).map(|i| (i * 7 + 3) as u8).collect(),
        (0..1000u32).map(|i| (i % 251) as u8).collect(),
    ]
}

fn conjunto<P: XmssParameter>(nombre: &str, sal: u8) -> String {
    let semilla: Vec<u8> = (0..96u32).map(|i| ((i * 13 + sal as u32) % 256) as u8).collect();
    let mut kp = KeyPair::<P>::from_seed(&semilla).expect("clave desde semilla");
    let pk = kp.verifying_key().as_ref().to_vec();
    let mut firmas = Vec::new();
    for (i, m) in mensajes().into_iter().enumerate() {
        // Dos claves gemelas desde la misma semilla: una firma adjunta y otra separada, al MISMO indice.
        let mut kp_d = KeyPair::<P>::from_seed(&semilla).expect("clave gemela");
        for _ in 0..i {
            kp_d.signing_key().sign_detached(b"avance").expect("avance");
        }
        let firmado = kp.signing_key().sign(&m).expect("sign");
        let separada = kp_d.signing_key().sign_detached(&m).expect("sign_detached");
        let recuperado = kp.verifying_key().verify(&firmado).expect("el crate verifica lo adjunto");
        assert_eq!(recuperado, m, "el mensaje recuperado es el firmado");
        kp.verifying_key().verify_detached(&separada, &m).expect("el crate verifica lo separado");
        firmas.push(format!(
            "    {{\"indice\": {i}, \"mensaje\": \"{}\", \"firmado\": \"{}\", \"firma\": \"{}\"}}",
            hex(&m), hex(firmado.as_ref()), hex(separada.as_ref())
        ));
    }
    format!(
        "  {{\"conjunto\": \"{nombre}\", \"oid\": \"{}\", \"semilla\": \"{}\", \"clave_publica\": \"{}\",\n   \"firmas\": [\n{}\n   ]}}",
        hex(&pk[..4]), hex(&semilla), hex(&pk), firmas.join(",\n")
    )
}

fn main() {
    let bloques = vec![
        conjunto::<XmssMtSha2_40_8_256>("XMSSMT-SHA2_40/8_256", 3),
        conjunto::<XmssMtSha2_20_2_256>("XMSSMT-SHA2_20/2_256", 5),
    ];
    println!("{{\"generador\": \"xmss 0.1.0-pre.0 (RustCrypto/signatures), tools/segunda/kat_xmss/generador\",\n \"conjuntos\": [\n{}\n ]}}", bloques.join(",\n"));
}
