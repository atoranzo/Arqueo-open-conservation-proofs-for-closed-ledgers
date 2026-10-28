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
    /// §899: la marca de laboratorio declara menos de dos columnas aleatorias
    Columnas,
}

/// ARQUEO (§899, la 133 de `BACKLOG.md`): la marca de LABORATORIO, `arqueo:oculta:lab`, m y las
/// columnas aleatorias en un byte. La produccion no la escribe ni la lee: `Marca::leer`, que es lo
/// que lee el verificador de produccion y el kit, la rechaza como desconocida; la escribe el
/// probador que enciende mas de una columna aleatoria y la lee `verify_laboratorio`. Cuando el
/// corte 0.5 fije su marca v2 (RFC-0018 D-G), las columnas iran en ella y esta se va.
pub const PREFIJO_LABORATORIO: &[u8] = b"arqueo:oculta:lab";

/// Lo que ocupa la marca de laboratorio: el prefijo, los cuatro bytes de m y el de las columnas.
pub const LARGO_LABORATORIO: usize = 17 + 4 + 1;

/// La marca de laboratorio: m y cuantas columnas aleatorias lleva la traza principal, dos o mas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MarcaLaboratorio {
    pub m: usize,
    pub columnas: usize,
}

impl MarcaLaboratorio {
    /// Los bytes que van al meta: el prefijo, m en cuatro bytes little-endian y las columnas.
    pub fn escribir(&self) -> Vec<u8> {
        assert!(self.m <= u32::MAX as usize, "marca de laboratorio: m no cabe en cuatro bytes");
        assert!((2..=255).contains(&self.columnas), "marca de laboratorio: columnas {}", self.columnas);
        let mut meta = Vec::with_capacity(LARGO_LABORATORIO);
        meta.extend_from_slice(PREFIJO_LABORATORIO);
        meta.extend_from_slice(&(self.m as u32).to_le_bytes());
        meta.push(self.columnas as u8);
        meta
    }

    /// `Ok(None)` con el meta vacio, `Ok(Some(marca))` con el prefijo de laboratorio y sus cinco
    /// bytes, y `Err` con cualquier otro meta, la marca de produccion incluida.
    pub fn leer(meta: &[u8]) -> Result<Option<MarcaLaboratorio>, MarcaError> {
        if meta.is_empty() {
            return Ok(None);
        }
        if !meta.starts_with(PREFIJO_LABORATORIO) {
            return Err(MarcaError::Desconocida);
        }
        if meta.len() != LARGO_LABORATORIO {
            return Err(MarcaError::Largo);
        }
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(&meta[PREFIJO_LABORATORIO.len()..PREFIJO_LABORATORIO.len() + 4]);
        let columnas = meta[LARGO_LABORATORIO - 1] as usize;
        if columnas < 2 {
            return Err(MarcaError::Columnas);
        }
        Ok(Some(MarcaLaboratorio { m: u32::from_le_bytes(bytes) as usize, columnas }))
    }
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

    /// m cuando el meta es una marca -de produccion o, desde el §899, de laboratorio-, y 0 en
    /// cualquier otro caso: sin marca, los trozos del cociente son los de winterfell. Quien tenga
    /// que rechazar un meta que no es marca -el verificador- usa `leer`, no esto.
    pub fn m_de(meta: &[u8]) -> usize {
        match Marca::leer(meta) {
            Ok(Some(marca)) => marca.m,
            _ => match MarcaLaboratorio::leer(meta) {
                Ok(Some(marca)) => marca.m,
                _ => 0,
            },
        }
    }

    /// §899: las columnas aleatorias que declara el meta: una con la marca de produccion, las
    /// suyas con la de laboratorio, y 0 sin marca.
    pub fn columnas_de(meta: &[u8]) -> usize {
        match Marca::leer(meta) {
            Ok(Some(_)) => 1,
            _ => match MarcaLaboratorio::leer(meta) {
                Ok(Some(marca)) => marca.columnas,
                _ => 0,
            },
        }
    }
}
