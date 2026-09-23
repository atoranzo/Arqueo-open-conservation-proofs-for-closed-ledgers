// ARQUEO (RFC-0009 E3a-2, AUDITORIA 534): la marca que lleva el meta de la traza de una prueba
// oculta, con m dentro. Un solo lector para los tres que lo necesitan: el contexto del AIR (los
// trozos del cociente), el dominio del probador y el verificador (el despacho y el paso). Con el
// meta vacio la prueba es la de winterfell tal cual; con la marca, el envoltorio `Oculta<A>`; con
// cualquier otro meta, error: ningun AIR de la casa escribe en el meta (D-G), asi que un meta que
// no es la marca no es de nadie.

use alloc::vec::Vec;

/// El prefijo, y en el su version: `arqueo:oculta:1`. Otra version es otra marca, desconocida.
pub const PREFIJO: &[u8] = b"arqueo:oculta:1";

/// Lo que ocupa una marca: el prefijo y los cuatro bytes de m, little-endian.
pub const LARGO: usize = 15 + 4;

/// Lo que dice el meta de una prueba oculta: m, el grado de los aleatorizadores del cociente (los
/// trozos avanzan de s = L - m en s, con L la longitud de la traza oculta, 2T). m decide la
/// ocultacion, no la solidez: el verificador solo exige que quepa en la traza (D-H).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Marca {
    pub m: usize,
}

/// Por que un meta no es una marca.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarcaError {
    /// no esta vacio y no empieza por el prefijo: otra version, u otro meta
    Desconocida,
    /// el prefijo esta y detras no van exactamente los cuatro bytes de m
    Largo,
}

impl Marca {
    /// Los bytes que van al meta de la traza: el prefijo y m en cuatro bytes little-endian.
    pub fn escribir(&self) -> Vec<u8> {
        assert!(self.m <= u32::MAX as usize, "marca: m no cabe en cuatro bytes");
        let mut meta = Vec::with_capacity(LARGO);
        meta.extend_from_slice(PREFIJO);
        meta.extend_from_slice(&(self.m as u32).to_le_bytes());
        meta
    }

    /// `Ok(None)` con el meta vacio (winterfell tal cual), `Ok(Some(marca))` con el prefijo y sus
    /// cuatro bytes, y `Err` con cualquier otro meta.
    pub fn leer(meta: &[u8]) -> Result<Option<Marca>, MarcaError> {
        if meta.is_empty() {
            return Ok(None);
        }
        if !meta.starts_with(PREFIJO) {
            return Err(MarcaError::Desconocida);
        }
        if meta.len() != LARGO {
            return Err(MarcaError::Largo);
        }
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(&meta[PREFIJO.len()..]);
        Ok(Some(Marca { m: u32::from_le_bytes(bytes) as usize }))
    }

    /// m cuando el meta es una marca, y 0 en cualquier otro caso: sin marca, los trozos del
    /// cociente son los de winterfell. Quien tenga que rechazar un meta que no es marca -el
    /// verificador- usa `leer`, no esto.
    pub fn m_de(meta: &[u8]) -> usize {
        match Marca::leer(meta) {
            Ok(Some(marca)) => marca.m,
            _ => 0,
        }
    }
}
