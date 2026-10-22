// ARQUEO (§715, la maleabilidad de los bytes de la prueba): fichero NUEVO del fork; ver
// el README del crate.

//! Los testigos de que un `usize` de la prueba tiene UNA escritura, la minima.
//!
//! `read_usize` de winter-utils 0.13.1 lee el mismo valor escrito en uno, dos o nueve bytes. Una
//! prueba con una longitud escrita de mas verificaba igual, con otros bytes y otra huella: la capa
//! indexa por esa huella las pruebas ya aplicadas, y el reenvio de un pago tras un ciclo A->B->A
//! volvia a debitar al pagador (medido en la capa). Sin la regla de la escritura minima en
//! [`leer_vint_canonico`], fallan el primero y el tercero de los testigos de abajo: las escrituras
//! de mas se leen.
//!
//! Y la regla no depende de la plataforma: la escritura minima se mide sobre el `u64`, lo que no
//! cabe en el `usize` es fin de bytes como en 64 bits, y el numero de restricciones lo acota el
//! contexto antes de pasar a `usize`. En `wasm32` (32 bits) el `read_usize` de upstream daba un
//! texto sin causa donde el kit nativo daba la del contexto (medido con el kit en wasmtime).

use alloc::{format, vec::Vec};

use utils::{
    ByteReader, ByteWriter, Deserializable, DeserializationError, Serializable, SliceReader,
};

use super::{leer_usize_canonico, leer_vint_canonico, Context, LectorAcotado, Proof};
use crate::causa;

/// La escritura de `v` en `largo` bytes, como la de winter-utils pero sin exigir que sea la
/// minima: `largo` de 1 a 8 lleva el valor desplazado y el bit de marca; 9 es un cero y el `u64`.
fn escritura(v: u64, largo: usize) -> Vec<u8> {
    if largo == 9 {
        let mut o = alloc::vec![0u8];
        o.extend_from_slice(&v.to_le_bytes());
        return o;
    }
    assert!(v < (1u64 << (7 * largo)), "{v} no cabe en {largo} bytes");
    let x = ((((v as u128) << 1) | 1) << (largo - 1)) as u64;
    x.to_le_bytes()[..largo].to_vec()
}

/// El largo de la escritura minima, sacado de `write_usize` de upstream.
fn largo_minimo(v: u64) -> usize {
    let mut o = Vec::new();
    o.write_usize(v as usize);
    o.len()
}

/// El texto de la causa de un error, o un fallo si no es `InvalidValue`.
fn causa_de<T: core::fmt::Debug>(r: Result<T, DeserializationError>) -> alloc::string::String {
    match r {
        Err(DeserializationError::InvalidValue(m)) => m,
        otro => panic!("se esperaba InvalidValue, y dio {otro:?}"),
    }
}

/// Cada valor en el borde de un largo: su escritura minima se lee y consume justo sus bytes; cada
/// escritura mas larga del mismo valor, hasta nueve bytes, es error con la causa del `usize`.
#[test]
fn un_usize_solo_se_lee_en_su_escritura_minima() {
    let mut valores = alloc::vec![0u64, 1, 5, 100];
    for n in 1..=8u32 {
        valores.push((1u64 << (7 * n)) - 1);
        valores.push(1u64 << (7 * n));
    }
    valores.push(u64::MAX);
    let mut rechazadas = 0;
    for v in valores {
        let minimo = largo_minimo(v);
        let buena = escritura(v, minimo);
        let mut cola = buena.clone();
        cola.push(0xAB);
        let mut r = SliceReader::new(&cola);
        assert_eq!(leer_vint_canonico(&mut r).ok(), Some(v), "{v} en {minimo} bytes");
        assert_eq!(r.read_u8().ok(), Some(0xAB), "{v}: consume justo su escritura");
        for largo in minimo + 1..=9 {
            let b = escritura(v, largo);
            let m = causa_de(leer_vint_canonico(&mut SliceReader::new(&b)));
            assert!(m.starts_with(causa::USIZE), "{v} en {largo} bytes: {m}");
            // y como `usize`: la escritura se mira antes que el rango
            let m = causa_de(leer_usize_canonico(&mut SliceReader::new(&b)));
            assert!(m.starts_with(causa::USIZE), "{v} en {largo} bytes, como usize: {m}");
            rechazadas += 1;
        }
    }
    // 4 valores pequenos (8 escrituras de mas cada uno), y los 17 de los bordes
    assert!(rechazadas > 60, "{rechazadas}");
}

/// Sin bytes, o con menos de los que el primero anuncia: el `UnexpectedEOF` de upstream, no la
/// causa del `usize`.
#[test]
fn un_usize_cortado_es_fin_de_bytes() {
    for b in [Vec::new(), alloc::vec![0u8], alloc::vec![0b10u8]] {
        let r = leer_vint_canonico(&mut SliceReader::new(&b));
        assert!(matches!(r, Err(DeserializationError::UnexpectedEOF)), "{b:?}: {r:?}");
        let r = leer_usize_canonico(&mut SliceReader::new(&b));
        assert!(matches!(r, Err(DeserializationError::UnexpectedEOF)), "{b:?}: {r:?}");
    }
}

/// La prueba de relleno: el numero de restricciones del contexto y la longitud de los valores de
/// sus primeras consultas, escritos con un byte de mas y con nueve, son error con la causa del
/// `usize`; la prueba sin tocar se lee y vuelve a dar sus bytes.
#[test]
fn la_prueba_con_un_usize_escrito_de_mas_no_se_lee() {
    let p = Proof::new_dummy();
    let buena = p.to_bytes();
    assert_eq!(Proof::from_bytes(&buena).map(|q| q.to_bytes()).ok(), Some(buena.clone()));
    let contexto = p.context.to_bytes();
    // el numero de restricciones es lo ultimo del contexto
    let nc = p.context.num_constraints() as u64;
    let en_nc = contexto.len() - largo_minimo(nc);
    // la longitud de los valores de las primeras consultas va tras las consultas unicas y los
    // compromisos
    let en_valores = contexto.len() + 1 + p.commitments.to_bytes().len();
    let (valores, largo_valores) = {
        let mut r = SliceReader::new(&buena[en_valores..]);
        let v = r.read_usize().expect("la longitud de los valores");
        (v as u64, largo_minimo(v as u64))
    };
    for (que, en, v, largo) in [
        ("restricciones", en_nc, nc, largo_minimo(nc)),
        ("valores", en_valores, valores, largo_valores),
    ] {
        for nuevo in [largo + 1, 9] {
            let mut b = buena[..en].to_vec();
            b.extend(escritura(v, nuevo));
            b.extend_from_slice(&buena[en + largo..]);
            let m = causa_de(Proof::from_bytes(&b));
            assert!(m.starts_with(causa::USIZE), "{que} en {nuevo} bytes: {m}");
        }
    }
}

/// La formula del largo minimo es la de `write_usize`: en cada borde de largo, a cada lado, y en
/// 200.000 valores repartidos por todos los anchos. Solo en 64 bits, donde `write_usize` escribe
/// cualquier `u64`. Falsador: con `9 - min(ceros / 7, 8)` (sin el `- 1`) falla en `128`.
#[cfg(target_pointer_width = "64")]
#[test]
fn el_largo_minimo_es_el_de_write_usize() {
    let mut valores = alloc::vec![0u64, 1, u64::MAX, u64::MAX - 1];
    for n in 1..64u32 {
        let b = 1u64 << n;
        valores.extend([b - 1, b, b + 1]);
    }
    // un generador lineal congruente: cada valor con un ancho de 1 a 64 bits
    let mut x = 0x9E37_79B9_7F4A_7C15u64;
    for i in 0..200_000u64 {
        x = x.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
        valores.push(x >> (i % 64));
    }
    for v in valores {
        assert_eq!(super::acotado::largo_minimo(v), largo_minimo(v), "{v}");
    }
}

/// Un `usize` desde `2^32` en su escritura minima: el `u64` se lee entero con
/// [`leer_vint_canonico`], y por el lector de la prueba una longitud asi es `UnexpectedEOF` en
/// TODA plataforma: en 64 bits al ir a leer sus bytes (`check_eor`), en 32 porque no cabe en el
/// `usize`. Aqui corre la rama de 64 bits; la de 32 se mide con el kit compilado a
/// `wasm32-wasip1` en wasmtime, con una longitud y una cuenta del lote desde `2^32`: la misma
/// salida que el nativo.
#[test]
fn una_longitud_desde_dos_a_la_32_es_fin_de_bytes_en_toda_plataforma() {
    let tope = u64::from(u32::MAX);
    for v in [tope + 1, 1u64 << 40, (1u64 << 56) - 1, 1u64 << 56, u64::MAX] {
        let b = escritura(v, largo_minimo(v));
        assert_eq!(leer_vint_canonico(&mut SliceReader::new(&b)).ok(), Some(v), "{v}");
        let directo = leer_usize_canonico(&mut SliceReader::new(&b));
        #[cfg(target_pointer_width = "32")]
        assert!(matches!(directo, Err(DeserializationError::UnexpectedEOF)), "{v}: {directo:?}");
        #[cfg(target_pointer_width = "64")]
        assert_eq!(directo.ok(), Some(v as usize), "{v}");
        // una longitud de `Vec<u8>`, con cuatro bytes detras: por el lector de la prueba
        let mut b = b;
        b.extend([1u8, 2, 3, 4]);
        let r = Vec::<u8>::read_from(&mut LectorAcotado::new(&b));
        assert!(matches!(r, Err(DeserializationError::UnexpectedEOF)), "{v}: {r:?}");
    }
}

/// El `read_usize` de upstream con un `usize` de 32 bits: el mismo rechazo y el mismo texto que
/// en `wasm32`. Simula esa plataforma en la que corren los tests.
struct Lector32<'a>(SliceReader<'a>);

impl ByteReader for Lector32<'_> {
    fn read_u8(&mut self) -> Result<u8, DeserializationError> {
        self.0.read_u8()
    }

    fn peek_u8(&self) -> Result<u8, DeserializationError> {
        self.0.peek_u8()
    }

    fn read_slice(&mut self, len: usize) -> Result<&[u8], DeserializationError> {
        self.0.read_slice(len)
    }

    fn read_array<const N: usize>(&mut self) -> Result<[u8; N], DeserializationError> {
        self.0.read_array()
    }

    fn check_eor(&self, num_bytes: usize) -> Result<(), DeserializationError> {
        self.0.check_eor(num_bytes)
    }

    fn has_more_bytes(&self) -> bool {
        self.0.has_more_bytes()
    }

    fn read_usize(&mut self) -> Result<usize, DeserializationError> {
        let v = self.0.read_usize()?;
        if v as u64 > u64::from(u32::MAX) {
            return Err(DeserializationError::InvalidValue(format!(
                "Encoded value must be less than {}, but {} was provided",
                u32::MAX,
                v
            )));
        }
        Ok(v)
    }
}

/// El numero de restricciones mas `2^32`, en su escritura minima (cinco bytes): la causa del
/// contexto, por la prueba entera y por un lector de 32 bits. Asi la da tambien el kit de
/// `wasm32`, donde `read_usize` rechazaba antes, sin causa: medido con el kit en wasmtime, los
/// vectores `neg-prueba-restricciones-alias.json` de `edad/` y de `pago/` daban «Encoded value
/// must be less than 4294967295». Falsador: con `source.read_usize()` en `Context::read_from`,
/// el lector de 32 bits da ese texto.
#[test]
fn las_restricciones_mas_dos_a_la_32_son_del_contexto_tambien_en_32_bits() {
    let p = Proof::new_dummy();
    let buena = p.to_bytes();
    let contexto = p.context.to_bytes();
    let nc = p.context.num_constraints() as u64;
    let en_nc = contexto.len() - largo_minimo(nc);
    for v in [nc + (1u64 << 32), u64::MAX] {
        let mut ctx = contexto[..en_nc].to_vec();
        ctx.extend(escritura(v, largo_minimo(v)));
        let m = causa_de(Context::read_from(&mut Lector32(SliceReader::new(&ctx))));
        assert!(m.starts_with(causa::CONTEXTO), "{v}, con un lector de 32 bits: {m}");
        let mut b = buena[..en_nc].to_vec();
        b.extend(escritura(v, largo_minimo(v)));
        b.extend_from_slice(&buena[contexto.len()..]);
        let m = causa_de(Proof::from_bytes(&b));
        assert!(m.starts_with(causa::CONTEXTO), "{v}, la prueba: {m}");
    }
    // el contexto sin tocar se lee igual con el lector de 32 bits
    let leido = Context::read_from(&mut Lector32(SliceReader::new(&contexto)));
    assert_eq!(leido.map(|c| c.to_bytes()).ok(), Some(contexto));
}
