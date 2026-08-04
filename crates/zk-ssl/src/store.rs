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
use winterfell::math::FieldElement;

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

/// **Centinela de `view_id`** para cuentas anteriores a 49-A (asiento de
/// alcance de 49-A): el digest cero, que `view_id_of` no produce nunca
/// (siempre mezcla el dominio VIEW_KEY en el estado Rescue). Una cuenta
/// con este `view_id` es pre-49-A y su vista autenticada devolvera
/// siempre `None`: no-retroactividad declarada, como el salt de §117 con
/// las hojas viejas.
pub const VIEW_ID_LEGACY: Digest = [BaseElement::ZERO; 4];

/// **Centinela de `leaf_salt`** para cuentas anteriores a B13/B14
/// (entrada 50): digest cero. Una cuenta migrada lleva salt-cero
/// (no-retroactividad §126.4: sigue barrible, y lo declara); una abierta
/// después lleva salt real derivado de la clave (§117). `derive_leaf_salt`
/// nunca produce cero (mezcla el dominio), así que el centinela es
/// inalcanzable por una clave real.
pub const LEAF_SALT_LEGACY: Digest = [BaseElement::ZERO; 4];

/// **Formato NUEVO** (80 bytes): identidad (32) + saldo (8) + nonce (8) +
/// view_id (32). Paso 1 de 49-A. Los call-sites migran a esta en pasos
/// posteriores; hasta entonces coexiste con el shim legacy de abajo.
/// **Formato v3** (112 bytes): v2 (80) + leaf_salt (32). Entrada 50 /
/// B13-B14. El salt va AL FINAL para que la longitud siga discriminando
/// versión sin byte de tipo (48/80/112). Los call-sites migran en el
/// paso 1a; hasta entonces coexiste con el shim v2 de abajo.
pub fn record_to_bytes_v3(
    public_id: &Digest,
    balance: u64,
    nonce: BaseElement,
    view_id: &Digest,
    leaf_salt: &Digest,
) -> [u8; 112] {
    let mut out = [0u8; 112];
    out[..32].copy_from_slice(&digest_to_bytes(public_id));
    out[32..40].copy_from_slice(&balance.to_le_bytes());
    out[40..48].copy_from_slice(&element_to_bytes(nonce));
    out[48..80].copy_from_slice(&digest_to_bytes(view_id));
    out[80..112].copy_from_slice(&digest_to_bytes(leaf_salt));
    out
}

pub fn record_to_bytes_v2(
    public_id: &Digest,
    balance: u64,
    nonce: BaseElement,
    view_id: &Digest,
) -> [u8; 80] {
    let mut out = [0u8; 80];
    out[..32].copy_from_slice(&digest_to_bytes(public_id));
    out[32..40].copy_from_slice(&balance.to_le_bytes());
    out[40..48].copy_from_slice(&element_to_bytes(nonce));
    out[48..80].copy_from_slice(&digest_to_bytes(view_id));
    out
}

/// **Shim del formato viejo** (48 bytes, sin view_id). TEMPORAL: existe
/// solo para que los call-sites no migrados sigan compilando durante el
/// despliegue de 49-A. Muere cuando el ultimo call-site pase a `_v2`.
#[deprecated(note = "49-A paso 1: migrar a record_to_bytes_v2; este shim se elimina al cerrar 49-A")]
pub fn record_to_bytes(public_id: &Digest, balance: u64, nonce: BaseElement) -> [u8; 48] {
    let mut out = [0u8; 48];
    out[..32].copy_from_slice(&digest_to_bytes(public_id));
    out[32..40].copy_from_slice(&balance.to_le_bytes());
    out[40..48].copy_from_slice(&element_to_bytes(nonce));
    out
}

/// **Lectura DUAL por longitud** (paso 1 de 49-A). 48 bytes = formato
/// viejo, `view_id` <- centinela legacy; 80 bytes = formato nuevo con
/// view_id real. La longitud fija discrimina sin byte de version —el
/// mismo espiritu que el arbol `legacy_null` de `persistence.rs`—.
/// Cualquier otra longitud se rechaza (un registro truncado no se
/// interpreta mal).
/// **Lectura TRIPLE por longitud** (B13/B14). 48 = pre-49-A (view_id y
/// salt centinela); 80 = post-49-A pre-salt (salt centinela); 112 =
/// con salt real. Cualquier otra longitud se rechaza.
pub fn record_from_bytes_v3(
    b: &[u8],
) -> Result<(Digest, u64, BaseElement, Digest, Digest), StoreError> {
    match b.len() {
        48 => {
            let public_id = digest_from_bytes(&b[..32])?;
            let balance = u64::from_le_bytes(b[32..40].try_into().unwrap());
            let nonce = element_from_bytes(&b[40..48])?;
            Ok((public_id, balance, nonce, VIEW_ID_LEGACY, LEAF_SALT_LEGACY))
        }
        80 => {
            let public_id = digest_from_bytes(&b[..32])?;
            let balance = u64::from_le_bytes(b[32..40].try_into().unwrap());
            let nonce = element_from_bytes(&b[40..48])?;
            let view_id = digest_from_bytes(&b[48..80])?;
            Ok((public_id, balance, nonce, view_id, LEAF_SALT_LEGACY))
        }
        112 => {
            let public_id = digest_from_bytes(&b[..32])?;
            let balance = u64::from_le_bytes(b[32..40].try_into().unwrap());
            let nonce = element_from_bytes(&b[40..48])?;
            let view_id = digest_from_bytes(&b[48..80])?;
            let leaf_salt = digest_from_bytes(&b[80..112])?;
            Ok((public_id, balance, nonce, view_id, leaf_salt))
        }
        n => Err(StoreError::Malformed(format!(
            "registro de {n} bytes, se esperaban 48/80/112"
        ))),
    }
}

pub fn record_from_bytes_v2(
    b: &[u8],
) -> Result<(Digest, u64, BaseElement, Digest), StoreError> {
    match b.len() {
        48 => {
            let public_id = digest_from_bytes(&b[..32])?;
            let balance = u64::from_le_bytes(b[32..40].try_into().unwrap());
            let nonce = element_from_bytes(&b[40..48])?;
            Ok((public_id, balance, nonce, VIEW_ID_LEGACY))
        }
        80 => {
            let public_id = digest_from_bytes(&b[..32])?;
            let balance = u64::from_le_bytes(b[32..40].try_into().unwrap());
            let nonce = element_from_bytes(&b[40..48])?;
            let view_id = digest_from_bytes(&b[48..80])?;
            Ok((public_id, balance, nonce, view_id))
        }
        n => Err(StoreError::Malformed(format!(
            "registro de {n} bytes, se esperaban 48 (viejo) u 80 (nuevo)"
        ))),
    }
}

/// Lectura del formato viejo. Delega en `_v2` y descarta el view_id, para
/// que los call-sites no migrados sigan compilando. TEMPORAL (49-A).
pub fn record_from_bytes(b: &[u8]) -> Result<(Digest, u64, BaseElement), StoreError> {
    let (id, bal, nonce, _view) = record_from_bytes_v2(b)?;
    Ok((id, bal, nonce))
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
    /// Paso 1 de 49-A: formato dual por longitud.
    /// B13/B14: formato triple por longitud (48/80/112).
    #[test]
    fn record_v3_triple_por_longitud() {
        let id: Digest = [BaseElement::new(9); 4];
        let vid: Digest = [BaseElement::new(0x5EE); 4];
        let salt: Digest = [BaseElement::new(0x5A17); 4];

        // 112 -> todo real, roundtrip.
        let b112 = record_to_bytes_v3(&id, 1_000_000, BaseElement::new(7), &vid, &salt);
        assert_eq!(b112.len(), 112);
        let (i2, b2, n2, v2, s2) = record_from_bytes_v3(&b112).unwrap();
        assert_eq!((i2, b2, n2, v2, s2), (id, 1_000_000, BaseElement::new(7), vid, salt));

        // 80 (post-49-A pre-salt) -> salt centinela, view_id real.
        let b80 = record_to_bytes_v2(&id, 500, BaseElement::new(3), &vid);
        let (_, _, _, v3, s3) = record_from_bytes_v3(&b80).unwrap();
        assert_eq!(v3, vid, "el view_id de un v2 debe sobrevivir");
        assert_eq!(s3, LEAF_SALT_LEGACY, "un v2 debe cargar con salt centinela");

        // 48 (pre-49-A) -> ambos centinela.
        #[allow(deprecated)]
        let b48 = record_to_bytes(&id, 42, BaseElement::new(1));
        let (_, _, _, v4, s4) = record_from_bytes_v3(&b48).unwrap();
        assert_eq!((v4, s4), (VIEW_ID_LEGACY, LEAF_SALT_LEGACY),
                   "un v1 debe cargar con AMBOS centinela");

        // El salt real es inalcanzable por el centinela.
        let salt_real = stark_experiment::circuit_settlement::derive_leaf_salt(
            BaseElement::new(0xBEE),
        );
        assert_ne!(salt_real, LEAF_SALT_LEGACY, "derive_leaf_salt no debe dar cero");

        // Frontera: 112 ok, 111 y 113 no.
        assert!(record_from_bytes_v3(&[0u8; 112]).is_ok());
        assert!(record_from_bytes_v3(&[0u8; 111]).is_err());
        assert!(record_from_bytes_v3(&[0u8; 113]).is_err());
    }

    #[test]
    fn record_v2_roundtrip_y_dual() {
        let id: Digest = [BaseElement::new(9); 4];
        let vid: Digest = [BaseElement::new(0x5EE); 4];

        let b80 = record_to_bytes_v2(&id, 1_000_000, BaseElement::new(7), &vid);
        assert_eq!(b80.len(), 80);
        let (id2, bal, nonce, vid2) = record_from_bytes_v2(&b80).unwrap();
        assert_eq!((id2, bal, nonce, vid2), (id, 1_000_000, BaseElement::new(7), vid));

        #[allow(deprecated)]
        let b48 = record_to_bytes(&id, 500, BaseElement::new(3));
        let (id3, bal3, nonce3, vid3) = record_from_bytes_v2(&b48).unwrap();
        assert_eq!((id3, bal3, nonce3), (id, 500, BaseElement::new(3)));
        assert_eq!(vid3, VIEW_ID_LEGACY, "una cuenta vieja debe cargar con centinela");

        let real = stark_experiment::circuit_settlement::view_id_of(
            BaseElement::new(0xA11CE),
        );
        assert_ne!(real, VIEW_ID_LEGACY, "view_id real colisiono con el centinela");

        assert!(record_from_bytes_v2(&[0u8; 40]).is_err());
        assert!(record_from_bytes_v2(&[0u8; 48]).is_ok());
        assert!(record_from_bytes_v2(&[0u8; 80]).is_ok());
    }

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
