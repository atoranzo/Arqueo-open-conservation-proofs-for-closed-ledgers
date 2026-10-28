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
    /// §899: la marca de laboratorio declara menos de dos columnas aleatorias; §906: la v2, ninguna
    Columnas,
    /// §906: la marca v2 lleva a cero la familia, la version o el perfil, o el perfil reservado 255
    Cero,
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

/// ARQUEO (§906; RFC-0018 D-G y E3, en LABORATORIO): la marca v2 del corte 0.5 con el byte de
/// columnas que le pide la D-AL del RFC-0009: `arqueo:oculta:2`, m (u32), la familia (u16, del
/// registro de `spec/NUCLEO.md`), la version de la AIR (u16), el perfil (u8, el indice de la tabla
/// de opciones) y las columnas aleatorias (u8): 25 bytes fijos, todos little-endian. Hasta el corte
/// ningun probador de produccion la escribe, y `Marca::leer` -lo que lee el verificador de
/// produccion y el kit- la rechaza como desconocida, igual que a la de laboratorio: la escribe el
/// probador que la pide en `Ocultacion::marca_v2` y la lee `verify_v2`. El cero no se asigna en
/// ninguno de los tres campos y el perfil 255 queda reservado (D-G).
pub const PREFIJO_V2: &[u8] = b"arqueo:oculta:2";

/// Lo que ocupa la marca v2: el prefijo, m, la familia, la version, el perfil y las columnas.
pub const LARGO_V2: usize = 15 + 4 + 2 + 2 + 1 + 1;

/// El perfil reservado para una prueba de otro motor (D-G): no se escribe ni se acepta aqui.
pub const PERFIL_RESERVADO: u8 = 255;

/// La marca v2: m, la familia, la version de la AIR, el perfil y las columnas aleatorias.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MarcaV2 {
    pub m: usize,
    pub familia: u16,
    pub version: u16,
    pub perfil: u8,
    pub columnas: u8,
}

impl MarcaV2 {
    /// Los 25 bytes que van al meta. Entra en panico con un campo a cero, el perfil reservado o
    /// una m que no cabe: son errores de quien construye, no de quien lee.
    pub fn escribir(&self) -> Vec<u8> {
        assert!(self.m <= u32::MAX as usize, "marca v2: m no cabe en cuatro bytes");
        assert!(self.familia != 0 && self.version != 0 && self.perfil != 0, "marca v2: un campo a cero");
        assert!(self.perfil != PERFIL_RESERVADO, "marca v2: el perfil 255 esta reservado");
        assert!(self.columnas >= 1, "marca v2: sin columnas aleatorias");
        let mut meta = Vec::with_capacity(LARGO_V2);
        meta.extend_from_slice(PREFIJO_V2);
        meta.extend_from_slice(&(self.m as u32).to_le_bytes());
        meta.extend_from_slice(&self.familia.to_le_bytes());
        meta.extend_from_slice(&self.version.to_le_bytes());
        meta.push(self.perfil);
        meta.push(self.columnas);
        meta
    }

    /// `Ok(None)` con el meta vacio, `Ok(Some(marca))` con el prefijo v2 y sus 25 bytes, y `Err`
    /// con cualquier otro meta: otro prefijo (`Desconocida`, la marca de produccion y la de
    /// laboratorio incluidas), otra longitud (`Largo`), un campo a cero o el perfil reservado
    /// (`Cero`), o ninguna columna (`Columnas`).
    pub fn leer(meta: &[u8]) -> Result<Option<MarcaV2>, MarcaError> {
        if meta.is_empty() {
            return Ok(None);
        }
        if !meta.starts_with(PREFIJO_V2) {
            return Err(MarcaError::Desconocida);
        }
        if meta.len() != LARGO_V2 {
            return Err(MarcaError::Largo);
        }
        let r = &meta[PREFIJO_V2.len()..];
        let m = u32::from_le_bytes([r[0], r[1], r[2], r[3]]) as usize;
        let familia = u16::from_le_bytes([r[4], r[5]]);
        let version = u16::from_le_bytes([r[6], r[7]]);
        let perfil = r[8];
        let columnas = r[9];
        if familia == 0 || version == 0 || perfil == 0 || perfil == PERFIL_RESERVADO {
            return Err(MarcaError::Cero);
        }
        if columnas == 0 {
            return Err(MarcaError::Columnas);
        }
        Ok(Some(MarcaV2 { m, familia, version, perfil, columnas }))
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
                // §906: la marca v2 de laboratorio lleva m igual que las otras
                _ => match MarcaV2::leer(meta) {
                    Ok(Some(marca)) => marca.m,
                    _ => 0,
                },
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
                // §906: la v2 las lleva en su ultimo byte
                _ => match MarcaV2::leer(meta) {
                    Ok(Some(marca)) => marca.columnas as usize,
                    _ => 0,
                },
            },
        }
    }
}
