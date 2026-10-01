//! # base64, el de mtc-core, sin PEM
//!
//! RFC 4648 base64 estándar, con relleno, sin dependencias. Es el
//! `base64_encode`/`base64_decode` de `src/pem.rs` de mtc-core
//! (`github.com/atoranzo/mtc-core`, commit
//! `d3b0ca614e51f177a0c30f1d21abd87b91208f59`), copiado como el árbol en el
//! §631: byte a byte salvo lo marcado `ADAPTADO (§632)`. Lo que no se copia
//! es lo de PEM (`encode`, `decode_all`, `PemBlock`): la nota del medio no
//! lleva bloques. Y un cambio de fondo, marcado: aquí **no se salta ningún
//! blanco**.
//!
//! El decodificador es estricto: relleno obligatorio, bits sobrantes a cero
//! y nada fuera del alfabeto. Es lo que pide la nota: la raíz y cada firma
//! tienen una sola escritura, y una variante que decodificara a lo mismo
//! sería una nota distinta con el mismo contenido.

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// ADAPTADO (§632): de las tres variantes de mtc-core queda la única que
/// no es de PEM.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PemError {
    /// A character outside the alphabet, a wrong length, a wrong padding
    /// or non-zero bits after the last symbol.
    BadBase64,
}

impl core::fmt::Display for PemError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            PemError::BadBase64 => write!(f, "invalid base64"),
        }
    }
}

impl std::error::Error for PemError {}

/// Standard base64 with padding, no line breaks.
pub fn base64_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        out.push(ALPHABET[(n >> 18) as usize & 63] as char);
        out.push(ALPHABET[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 {
            ALPHABET[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            ALPHABET[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}

fn value(c: u8) -> Option<u32> {
    match c {
        b'A'..=b'Z' => Some(u32::from(c - b'A')),
        b'a'..=b'z' => Some(u32::from(c - b'a') + 26),
        b'0'..=b'9' => Some(u32::from(c - b'0') + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

/// Strict standard base64: padding required, canonical trailing bits.
///
/// ADAPTADO (§632): en mtc-core este decodificador saltaba los blancos
/// ASCII, porque PEM parte las líneas a 64 columnas. En una nota no hay
/// nada que saltar: un blanco dentro de una línea de la nota es un error, y
/// se rechaza como cualquier otro carácter fuera del alfabeto.
pub fn base64_decode(s: &str) -> Result<Vec<u8>, PemError> {
    let symbols: Vec<u8> = s.bytes().collect();
    if !symbols.len().is_multiple_of(4) {
        return Err(PemError::BadBase64);
    }
    let mut out = Vec::with_capacity(symbols.len() / 4 * 3);
    for (i, quad) in symbols.chunks(4).enumerate() {
        let last = i == symbols.len() / 4 - 1;
        let pad = quad.iter().rev().take_while(|c| **c == b'=').count();
        if pad > 2 || (pad > 0 && !last) {
            return Err(PemError::BadBase64);
        }
        let mut n = 0u32;
        for c in &quad[..4 - pad] {
            n = (n << 6) | value(*c).ok_or(PemError::BadBase64)?;
        }
        n <<= 6 * pad as u32;
        let bytes = n.to_be_bytes();
        match pad {
            0 => out.extend_from_slice(&bytes[1..4]),
            1 => {
                if n & 0xff != 0 {
                    return Err(PemError::BadBase64);
                }
                out.extend_from_slice(&bytes[1..3]);
            }
            _ => {
                if n & 0xffff != 0 {
                    return Err(PemError::BadBase64);
                }
                out.push(bytes[1]);
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_round_trips_every_length_modulo_three() {
        for len in 0..40usize {
            let data: Vec<u8> = (0..len as u8).map(|i| i.wrapping_mul(37)).collect();
            let enc = base64_encode(&data);
            assert_eq!(enc.len(), len.div_ceil(3) * 4);
            assert_eq!(base64_decode(&enc).unwrap(), data);
        }
        // RFC 4648 §10 test vectors.
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foob"), "Zm9vYg==");
        assert_eq!(base64_encode(b"fooba"), "Zm9vYmE=");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn base64_decoder_fails_closed() {
        assert_eq!(base64_decode("Zg"), Err(PemError::BadBase64)); // no padding
        assert_eq!(base64_decode("Zh=="), Err(PemError::BadBase64)); // trailing bits
        assert_eq!(base64_decode("Zm9v!A=="), Err(PemError::BadBase64)); // alphabet
        assert_eq!(base64_decode("Zg==Zg=="), Err(PemError::BadBase64)); // padding inside
        assert_eq!(base64_decode("Z==="), Err(PemError::BadBase64)); // too much padding

        // ADAPTADO (§632): en mtc-core, «whitespace ok»; aquí es un error.
        assert_eq!(base64_decode("Zm9v\nYmFy\n"), Err(PemError::BadBase64));
        assert_eq!(base64_decode("Zm9v YmFy"), Err(PemError::BadBase64));
    }
}
