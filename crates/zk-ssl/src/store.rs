//! **Persistencia del ledger.**
//!
//! Sin esto, reiniciar el nodo pierde todo el estado: cuentas, saldos,
//! nullifiers gastados y suministro emitido. Es la carencia más básica de
//! una capa de liquidación.
//!
//! ## La decisión que importa: verificación de integridad al arrancar
//!
//! Guardar y cargar es lo fácil. Lo que de verdad importa es **detectar
//! un ledger corrupto antes de operar sobre él**.
//!
//! Al abrir, esta capa:
//!
//! 1. Reconstruye los dos árboles desde las hojas almacenadas.
//! 2. **Recalcula sus raíces.**
//! 3. Las compara con las raíces guardadas en el momento del último
//!    cierre.
//!
//! Si no coinciden, el arranque **falla** en vez de continuar. Un fallo
//! de disco, una escritura a medias o una manipulación producirían un
//! estado inconsistente, y un nodo que opere sobre él generaría pruebas
//! válidas de transiciones sobre un ledger que no es el real — el peor
//! fallo posible en un sistema de liquidación, porque sería
//! criptográficamente indetectable desde fuera.
//!
//! ## Formato
//!
//! Todo se guarda en `sled`, con claves prefijadas por tipo:
//!
//! | Clave | Contenido |
//! |---|---|
//! | `meta:*` | suministro, índice siguiente, límite, identidad del emisor |
//! | `root:*` | raíces del último cierre, para la verificación |
//! | `acct:{i}` | identidad (32 B) + saldo (8 B) + nonce (8 B) |
//!
//! Los elementos de Goldilocks caben en 8 bytes; un digest, en 32.
//!
//! ## ⚠️ Lo que NO resuelve
//!
//! - **No hay atomicidad entre operaciones.** Si el proceso muere entre
//!   actualizar el árbol y guardar la raíz, el arranque siguiente
//!   detectará la inconsistencia y se detendrá — que es lo correcto, pero
//!   requiere intervención manual. Un sistema real usaría un log de
//!   escritura anticipada.
//! - **No hay copias ni replicación.** Un disco perdido es un ledger
//!   perdido.
//! - **No hay cifrado en reposo.** Quien acceda al fichero ve todos los
//!   saldos. La privacidad de esta capa es frente a terceros que ven
//!   pruebas, no frente a quien tiene el disco.

use winterfell::math::fields::f64::BaseElement;
use winterfell::math::{FieldElement, StarkField};

use stark_experiment::merkle::Digest;

/// Errores de persistencia.
#[derive(Debug)]
pub enum StoreError {
    Io(String),
    /// **El ledger está corrupto**: las raíces reconstruidas no coinciden
    /// con las guardadas. Operar sobre él produciría pruebas válidas de
    /// transiciones sobre un estado que no es el real.
    IntegrityFailure {
        what: &'static str,
    },
    /// Los parámetros del sistema guardados no coinciden con los que se
    /// pasan al abrir. Cambiar la identidad del emisor o el límite
    /// regulatorio de un ledger existente es un error, no una
    /// configuración.
    ParameterMismatch {
        what: &'static str,
    },
    Malformed(String),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::Io(e) => write!(f, "error de almacenamiento: {e}"),
            StoreError::IntegrityFailure { what } => write!(
                f,
                "LEDGER CORRUPTO: la raiz reconstruida de '{what}' no coincide \
                 con la guardada. El nodo NO debe operar sobre este estado."
            ),
            StoreError::ParameterMismatch { what } => write!(
                f,
                "el parametro '{what}' no coincide con el del ledger existente"
            ),
            StoreError::Malformed(e) => write!(f, "dato mal formado en el ledger: {e}"),
        }
    }
}
impl std::error::Error for StoreError {}

// ---------------------------------------------------------------------
// Serialización
// ---------------------------------------------------------------------

/// Un elemento de Goldilocks cabe en 8 bytes.
pub fn element_to_bytes(e: BaseElement) -> [u8; 8] {
    e.as_int().to_le_bytes()
}

pub fn element_from_bytes(b: &[u8]) -> Result<BaseElement, StoreError> {
    let arr: [u8; 8] = b
        .try_into()
        .map_err(|_| StoreError::Malformed(format!("elemento de {} bytes", b.len())))?;
    Ok(BaseElement::new(u64::from_le_bytes(arr)))
}

pub fn digest_to_bytes(d: &Digest) -> [u8; 32] {
    let mut out = [0u8; 32];
    for (i, e) in d.iter().enumerate() {
        out[i * 8..(i + 1) * 8].copy_from_slice(&element_to_bytes(*e));
    }
    out
}

pub fn digest_from_bytes(b: &[u8]) -> Result<Digest, StoreError> {
    if b.len() != 32 {
        return Err(StoreError::Malformed(format!(
            "digest de {} bytes, se esperaban 32",
            b.len()
        )));
    }
    let mut d = [BaseElement::ZERO; 4];
    for i in 0..4 {
        d[i] = element_from_bytes(&b[i * 8..(i + 1) * 8])?;
    }
    Ok(d)
}

/// Registro de cuenta serializado: identidad (32) + saldo (8) + nonce (8).
pub fn record_to_bytes(public_id: &Digest, balance: u64, nonce: BaseElement) -> [u8; 48] {
    let mut out = [0u8; 48];
    out[..32].copy_from_slice(&digest_to_bytes(public_id));
    out[32..40].copy_from_slice(&balance.to_le_bytes());
    out[40..48].copy_from_slice(&element_to_bytes(nonce));
    out
}

pub fn record_from_bytes(b: &[u8]) -> Result<(Digest, u64, BaseElement), StoreError> {
    if b.len() != 48 {
        return Err(StoreError::Malformed(format!(
            "registro de {} bytes, se esperaban 48",
            b.len()
        )));
    }
    let public_id = digest_from_bytes(&b[..32])?;
    let balance = u64::from_le_bytes(b[32..40].try_into().unwrap());
    let nonce = element_from_bytes(&b[40..48])?;
    Ok((public_id, balance, nonce))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ida y vuelta de un elemento.
    #[test]
    fn element_roundtrip() {
        for v in [0u64, 1, 42, u64::MAX >> 1] {
            let e = BaseElement::new(v);
            assert_eq!(element_from_bytes(&element_to_bytes(e)).unwrap(), e);
        }
    }

    /// Ida y vuelta de un digest.
    #[test]
    fn digest_roundtrip() {
        let d: Digest = [
            BaseElement::new(1),
            BaseElement::new(2),
            BaseElement::new(3),
            BaseElement::new(4),
        ];
        assert_eq!(digest_from_bytes(&digest_to_bytes(&d)).unwrap(), d);
    }

    /// Ida y vuelta de un registro completo.
    #[test]
    fn record_roundtrip() {
        let id: Digest = [BaseElement::new(9); 4];
        let bytes = record_to_bytes(&id, 1_000_000, BaseElement::new(7));
        let (id2, bal, nonce) = record_from_bytes(&bytes).unwrap();
        assert_eq!(id2, id);
        assert_eq!(bal, 1_000_000);
        assert_eq!(nonce, BaseElement::new(7));
    }

    /// Un dato de tamaño incorrecto se rechaza en vez de interpretarse
    /// mal. Sin esto, un ledger truncado produciría valores plausibles
    /// pero falsos.
    #[test]
    fn malformed_data_is_rejected() {
        assert!(digest_from_bytes(&[0u8; 16]).is_err());
        assert!(record_from_bytes(&[0u8; 40]).is_err());
        assert!(element_from_bytes(&[0u8; 4]).is_err());
    }
}

/// Serializa una entrada del registro de transiciones.
///
/// Formato explícito de 137 bytes: `seq(8) | kind(1) | root_old(32) |
/// root_new(32) | proof_digest(32) | chain(32)`.
pub fn log_entry_to_bytes(e: &crate::log::LogEntry) -> Vec<u8> {
    let mut out = Vec::with_capacity(137);
    out.extend_from_slice(&e.seq.to_le_bytes());
    out.push(e.kind.tag_byte());
    out.extend_from_slice(&digest_to_bytes(&e.root_old));
    out.extend_from_slice(&digest_to_bytes(&e.root_new));
    out.extend_from_slice(&digest_to_bytes(&e.proof_digest));
    out.extend_from_slice(&digest_to_bytes(&e.chain));
    out
}

/// Lee una entrada del registro.
///
/// Un tipo de operación desconocido se **rechaza** en vez de ignorarse:
/// una entrada que no se puede interpretar no debe darse por buena.
pub fn log_entry_from_bytes(b: &[u8]) -> Result<crate::log::LogEntry, StoreError> {
    if b.len() != 137 {
        return Err(StoreError::Malformed(format!(
            "entrada del registro de {} bytes, se esperaban 137",
            b.len()
        )));
    }
    let seq = u64::from_le_bytes(b[0..8].try_into().unwrap());
    let kind = crate::log::OpKind::from_tag_byte(b[8])
        .ok_or_else(|| StoreError::Malformed(format!("tipo de operacion {} desconocido", b[8])))?;
    Ok(crate::log::LogEntry {
        seq,
        kind,
        root_old: digest_from_bytes(&b[9..41])?,
        root_new: digest_from_bytes(&b[41..73])?,
        proof_digest: digest_from_bytes(&b[73..105])?,
        chain: digest_from_bytes(&b[105..137])?,
    })
}
