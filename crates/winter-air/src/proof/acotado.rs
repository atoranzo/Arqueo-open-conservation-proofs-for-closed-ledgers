// ARQUEO (§575): fichero NUEVO del fork; ver el README del crate.

//! El lector ACOTADO de una prueba: no reserva lo que sus bytes no traen.
//!
//! `SliceReader` (winter-utils 0.13.1) hereda `read_many`, que reserva `num_elements` ANTES de
//! leer ninguno, y `num_elements` sale de los propios bytes: una prueba malformada que declara una
//! longitud de cientos de GB aborta el proceso, porque una reserva que falla no es un panico y no
//! se recoge. Aqui cada elemento ocupa al menos un byte, asi que mas elementos que bytes restantes
//! es `UnexpectedEOF` ANTES de reservar: el mismo error que upstream da al quedarse sin bytes,
//! cuando la reserva no lo mata antes. Una prueba bien formada se lee igual, byte a byte.
//!
//! `check_eor` compara con lo que queda, sin sumar: el de `SliceReader` suma `pos + n`, y con un
//! `n` gigante esa suma da la vuelta.
//!
//! ARQUEO (§715, la maleabilidad de los bytes de la prueba): el lector es tambien
//! CANONICO en sus `usize`. `read_usize` de winter-utils 0.13.1 acepta cualquier escritura del
//! mismo valor -el `5` en uno, dos o nueve bytes-, y una prueba con una longitud escrita de mas
//! verificaba igual, con otros bytes y otra huella. Aqui solo vale la escritura minima, la que
//! escribe `write_usize`. [`leer_vint_canonico`] es la unica regla de la escritura: la usan este
//! lector, el numero de restricciones del contexto (`context.rs`) y el lote con sal de la casa
//! (`zk-ssl-air/src/sal.rs`), que `winter-fri` lee con un `SliceReader`.
//!
//! ⚠️ La regla no depende de la plataforma. Upstream rechaza en `read_usize` lo que no cabe en el
//! `usize` de la maquina, con un texto sin causa: en `wasm32` (32 bits) eso es todo valor desde
//! `2^32`, y en 64 bits nada, porque alli el valor se lee y el lector se queda sin bytes despues.
//! El kit nativo y el de WebAssembly daban causas distintas para los mismos bytes (medido). Aqui la
//! escritura minima se mide sobre el `u64`, sin la cota de la maquina, y lo que no cabe en el
//! `usize` es fin de bytes en toda plataforma: lo que el nativo da al ir a leerlo.

use alloc::{format, vec::Vec};

use utils::{ByteReader, Deserializable, DeserializationError};

/// Cuantos bytes ocupa `v` en su escritura minima: la formula de `usize_encoded_len` de
/// winter-utils 0.13.1 (`byte_writer.rs`), que no es publica. Siete bits utiles por byte hasta
/// ocho bytes, y nueve -un cero y el `u64`- para lo que no cabe en 56 bits.
///
/// ⚠️ Es una copia, y no `write_usize`, porque `write_usize` toma un `usize`: en 32 bits no
/// escribe un valor desde `2^32`. Un testigo la ata a `write_usize` en 64 bits
/// (`tests_canonica.rs`).
pub(crate) fn largo_minimo(v: u64) -> usize {
    let ceros = v.leading_zeros() as usize;
    9 - core::cmp::min(ceros.saturating_sub(1) / 7, 8)
}

/// Lee un entero vint64 de winter-utils 0.13.1 en su escritura MINIMA, como `u64` y sin la cota de
/// la plataforma; si no es la minima, `InvalidValue` con la causa [`crate::causa::USIZE`].
///
/// El formato: los ceros finales del primer byte dicen cuantos bytes ocupa, y un primer byte a
/// cero son nueve (el cero y el `u64`). Se decodifica como upstream y se exige el largo de
/// [`largo_minimo`].
///
/// La lee directamente quien acota el valor por su cuenta antes de convertirlo, como el numero de
/// restricciones del contexto (`1..=u32::MAX`): asi su causa es la misma en 32 y en 64 bits.
///
/// ⚠️ No llama a `read_usize`: lo sobrescribe [`LectorAcotado`], y seria recursivo.
pub fn leer_vint_canonico<R: ByteReader>(source: &mut R) -> Result<u64, DeserializationError> {
    let primero = source.peek_u8()?;
    let largo = primero.trailing_zeros() as usize + 1;
    let valor = if largo == 9 {
        source.read_u8()?;
        u64::from_le_bytes(source.read_array::<8>()?)
    } else {
        let mut cifrado = [0u8; 8];
        cifrado[..largo].copy_from_slice(source.read_slice(largo)?);
        u64::from_le_bytes(cifrado) >> largo
    };
    let minimo = largo_minimo(valor);
    if minimo != largo {
        return Err(DeserializationError::InvalidValue(format!(
            "{}: {valor} escrito en {largo} bytes, y su escritura minima tiene {minimo}",
            crate::causa::USIZE,
        )));
    }
    Ok(valor)
}

/// Lee un `usize` de la prueba en su escritura MINIMA ([`leer_vint_canonico`]).
///
/// Lo que no cabe en el `usize` de la plataforma -en `wasm32`, desde `2^32`- es
/// `UnexpectedEOF`. No cabe tampoco en los bytes que quedan: cada `usize` que esto lee es la
/// longitud o la cuenta de elementos de al menos un byte, y ningun slice llega a `usize::MAX`
/// bytes. Es el error que da en 64 bits el lector al ir a leerlos (`check_eor`), asi que el kit de
/// 64 bits y el de `wasm32` dan la misma causa con los mismos bytes. Upstream daba en 32 bits un
/// texto sin causa («Encoded value must be less than ...»). El numero de restricciones del
/// contexto no es una longitud: lo lee [`leer_vint_canonico`] y lo acota el contexto.
///
/// ⚠️ No llama a `read_usize`: lo sobrescribe [`LectorAcotado`], y seria recursivo.
pub fn leer_usize_canonico<R: ByteReader>(source: &mut R) -> Result<usize, DeserializationError> {
    let valor = leer_vint_canonico(source)?;
    usize::try_from(valor).map_err(|_| DeserializationError::UnexpectedEOF)
}

/// Un lector de bytes que falla cerrado ante una cuenta que no cabe en lo que queda.
pub struct LectorAcotado<'a> {
    datos: &'a [u8],
    pos: usize,
}

impl<'a> LectorAcotado<'a> {
    /// El lector, al principio de `datos`.
    pub fn new(datos: &'a [u8]) -> Self {
        Self { datos, pos: 0 }
    }

    fn quedan(&self) -> usize {
        self.datos.len() - self.pos
    }
}

impl ByteReader for LectorAcotado<'_> {
    fn read_u8(&mut self) -> Result<u8, DeserializationError> {
        let b = self.peek_u8()?;
        self.pos += 1;
        Ok(b)
    }

    fn peek_u8(&self) -> Result<u8, DeserializationError> {
        self.datos.get(self.pos).copied().ok_or(DeserializationError::UnexpectedEOF)
    }

    fn read_slice(&mut self, len: usize) -> Result<&[u8], DeserializationError> {
        self.check_eor(len)?;
        let leido = &self.datos[self.pos..self.pos + len];
        self.pos += len;
        Ok(leido)
    }

    fn read_array<const N: usize>(&mut self) -> Result<[u8; N], DeserializationError> {
        let mut leido = [0u8; N];
        leido.copy_from_slice(self.read_slice(N)?);
        Ok(leido)
    }

    fn check_eor(&self, num_bytes: usize) -> Result<(), DeserializationError> {
        if num_bytes > self.quedan() {
            return Err(DeserializationError::UnexpectedEOF);
        }
        Ok(())
    }

    fn has_more_bytes(&self) -> bool {
        self.pos < self.datos.len()
    }

    // ARQUEO (§715): la escritura minima y ninguna otra. Pasan por aqui las longitudes de
    // los valores y de las aperturas de cada `Queries` (via `Vec::<u8>::read_from`). El numero de
    // restricciones no: el contexto lo lee con `leer_vint_canonico` y lo acota el.
    fn read_usize(&mut self) -> Result<usize, DeserializationError> {
        leer_usize_canonico(self)
    }

    fn read_many<D>(&mut self, num_elements: usize) -> Result<Vec<D>, DeserializationError>
    where
        Self: Sized,
        D: Deserializable,
    {
        // cada elemento ocupa al menos un byte: mas elementos que bytes no pueden estar
        self.check_eor(num_elements)?;
        let mut leidos = Vec::with_capacity(num_elements);
        for _ in 0..num_elements {
            leidos.push(D::read_from(self)?);
        }
        Ok(leidos)
    }
}
