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

use alloc::vec::Vec;

use utils::{ByteReader, Deserializable, DeserializationError};

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
