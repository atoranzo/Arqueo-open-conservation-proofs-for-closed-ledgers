// ARQUEO (§711): fichero NUEVO del fork; ver el README del crate.

//! Los testigos del §711: lo que trae una prueba ajena se rechaza con error y su causa, y no con un
//! panico. Upstream validaba la cabecera con `assert!`: un byte de consultas a cero hacia salir al
//! kit con 101 (medido). Cada testigo recoge el panico con `catch_unwind` y lo convierte en un
//! fallo con nombre; sin los arreglos del fork, entran en panico donde dicen.

extern crate std;

use alloc::{string::String, vec::Vec};
use std::panic::{catch_unwind, AssertUnwindSafe};

use math::fields::f64::BaseElement;
use utils::{ByteWriter, Deserializable, DeserializationError, Serializable};

use super::{Context, OodFrame, Proof, Table};
use crate::{causa, BatchingMethod, FieldExtension, ProofOptions, TraceInfo};

/// Las cinco sustituciones del barrido del §711.
const VALORES: [u8; 5] = [0x00, 0x01, 0x03, 0x40, 0xFF];

fn opciones() -> ProofOptions {
    ProofOptions::new(
        42,
        16,
        21,
        FieldExtension::Quadratic,
        8,
        31,
        BatchingMethod::Linear,
        BatchingMethod::Linear,
    )
}

/// Corre `f` y exige que NO entre en panico; devuelve lo que dio.
fn sin_panico<T>(que: &str, f: impl FnOnce() -> T) -> T {
    catch_unwind(AssertUnwindSafe(f)).unwrap_or_else(|_| panic!("CRITICO: {que} entra en panico"))
}

/// El texto de un `InvalidValue`, o un fallo si el error es otro.
fn texto(r: Result<impl core::fmt::Debug, DeserializationError>) -> String {
    match r {
        Err(DeserializationError::InvalidValue(m)) => m,
        otro => panic!("se esperaba InvalidValue, y dio {otro:?}"),
    }
}

/// Cada uno de los diez bytes de las opciones, con cada uno de sus 256 valores: o se lee y se
/// vuelve a escribir igual, o es error con la causa de las opciones. Ninguno entra en panico.
/// Falsador: sin `validar` en `read_from`, el primer byte a cero entra en panico en `new`.
#[test]
fn cada_byte_de_las_opciones_se_lee_o_es_error_con_causa() {
    let buenas = opciones().to_bytes();
    assert_eq!(buenas.len(), 10);
    let mut errores = 0;
    for i in 0..buenas.len() {
        for v in 0..=255u8 {
            let mut b = buenas.clone();
            b[i] = v;
            let r = sin_panico("leer unas opciones", || ProofOptions::read_from_bytes(&b));
            match r {
                Ok(o) => assert_eq!(o.to_bytes(), b, "byte {i} = {v}"),
                Err(DeserializationError::InvalidValue(m)) => {
                    assert!(m.starts_with(causa::OPCIONES), "byte {i} = {v}: {m}");
                    errores += 1;
                },
                Err(otro) => panic!("byte {i} = {v}: {otro:?}"),
            }
        }
    }
    // los 10 x 256 dan 1.972 errores con causa y 588 opciones que se leen: 1 + 249 + 223 + 253 +
    // 252 + 247 + 253 + 253 + 240 + 1, byte a byte (consultas 1..=255, expansion 2..=128
    // potencia de dos, molienda hasta 32, extension 1..=3, plegado 2..=16 potencia de dos, resto
    // 2^k - 1, dos agrupados 0..=2, particiones 1..=16 y tasa 1..=255)
    assert_eq!(errores, 1_972);
}

/// `new` sigue entrando en panico con el texto de upstream: quien construye las opciones en
/// codigo es quien las escribe.
#[test]
#[should_panic(expected = "number of queries must be greater than 0")]
fn las_opciones_nuevas_con_cero_consultas_siguen_siendo_panico() {
    let _ = ProofOptions::new(
        0,
        16,
        21,
        FieldExtension::Quadratic,
        8,
        31,
        BatchingMethod::Linear,
        BatchingMethod::Linear,
    );
}

/// La traza: los cuatro bytes de cabecera con cada valor. Con un logaritmo de 64 o mas, upstream
/// desbordaba en `2.pow(n)`; con un segmento auxiliar vacio y aleatorios, `new_multi_segment`
/// entraba en panico. Ahora, error con la causa de la traza.
#[test]
fn cada_byte_de_la_traza_se_lee_o_es_error_y_nunca_panico() {
    let buena = TraceInfo::new_multi_segment(84, 6, 2, 128, vec![1, 2, 3]).to_bytes();
    for i in 0..4 {
        for v in 0..=255u8 {
            let mut b = buena.clone();
            b[i] = v;
            let r = sin_panico("leer una traza", || TraceInfo::read_from_bytes(&b));
            if let Ok(t) = r {
                assert_eq!(t.to_bytes(), b, "byte {i} = {v}");
            }
        }
    }
    let con = |i: usize, v: u8| {
        let mut b = buena.clone();
        b[i] = v;
        texto(TraceInfo::read_from_bytes(&b))
    };
    for n in [32u8, 63, 64, 255] {
        let m = con(3, n);
        assert!(m.starts_with(causa::TRAZA), "2^{n}: {m}");
    }
    let mut b = buena.clone();
    b[1] = 0; // sin segmento auxiliar
    let m = texto(TraceInfo::read_from_bytes(&b));
    assert!(m.starts_with(causa::TRAZA) && m.contains("must be zero"), "{m}");
    // la mayor que cabe se sigue leyendo
    let mayor = TraceInfo::read_from_bytes(&TraceInfo::new(1, 1 << 31).to_bytes());
    assert_eq!(mayor.ok().map(|t| t.length()), Some(1 << 31));
}

/// El contexto: `Context::new` devuelve error donde upstream entraba en panico, y `read_from`
/// rechaza con su causa un dominio de extension que no cabe en 32 bits.
#[test]
fn el_contexto_fuera_de_rango_es_error_y_no_panico() {
    let grande = TraceInfo::new(1, 1 << 31);
    assert_eq!(
        Context::new::<BaseElement>(grande.clone(), opciones(), 1),
        Err("LDE domain size too big")
    );
    assert_eq!(
        Context::new::<BaseElement>(TraceInfo::new(1, 8), opciones(), 0),
        Err("number of constraints should be greater than zero")
    );
    let bueno = Context::new::<BaseElement>(TraceInfo::new(1, 1 << 27), opciones(), 1)
        .expect("2^27 x 16 cabe en 32 bits");
    let mut b = bueno.to_bytes();
    // el logaritmo de la longitud es el cuarto byte: 2^31 x 16 ya no cabe
    b[3] = 31;
    let m = sin_panico("leer un contexto", || texto(Context::read_from_bytes(&b)));
    assert!(m.starts_with(causa::CONTEXTO) && m.contains("LDE"), "{m}");
}

/// La prueba de relleno, byte a byte con las cinco sustituciones: se lee o es error, y nunca
/// panico. Y sus dos causas nuevas: cero consultas unicas, y una particion FRI de mas.
#[test]
fn la_prueba_de_relleno_byte_a_byte_nunca_entra_en_panico() {
    let buena = Proof::new_dummy().to_bytes();
    for i in 0..buena.len() {
        for v in VALORES {
            let mut b = buena.clone();
            b[i] = v;
            let _ = sin_panico("leer una prueba", || Proof::from_bytes(&b));
        }
    }
    let p = Proof::new_dummy();
    let mut cero = p.clone();
    cero.num_unique_queries = 0;
    let m = texto(Proof::from_bytes(&cero.to_bytes()));
    assert!(m.starts_with(causa::CONSULTAS), "{m}");
    // la particion FRI es el ultimo byte antes del nonce
    let mut b = buena.clone();
    let i = b.len() - 9;
    b[i] = 64;
    let m = sin_panico("leer una prueba", || texto(Proof::from_bytes(&b)));
    assert!(m.starts_with(causa::FRI), "{m}");
}

/// `Queries::parse` y `Table::from_bytes` devuelven error, con la causa de las consultas, donde
/// upstream tenia `assert!`: cero consultas -el byte 46 de las pruebas de edad y de pago, que
/// entraba en panico dentro de `verify`-, cero valores, un dominio que no es potencia de dos y
/// 255 filas.
#[test]
fn las_consultas_sin_forma_son_error_y_no_panico() {
    type H = crypto::hashers::Blake3_192<BaseElement>;
    type V = crypto::MerkleTree<H>;
    let q = Proof::new_dummy().constraint_queries;
    for (dominio, consultas, valores) in [(8, 0, 1), (8, 1, 0), (7, 1, 1)] {
        let r = sin_panico("leer unas consultas", || {
            q.clone().parse::<BaseElement, H, V>(dominio, consultas, valores).map(|_| ())
        });
        let m = texto(r);
        assert!(m.starts_with(causa::CONSULTAS), "{m}");
    }
    let bytes: Vec<u8> = vec![0; 8 * 255];
    let m = sin_panico("leer una tabla", || texto(Table::<BaseElement>::from_bytes(&bytes, 255, 1)));
    assert!(m.starts_with(causa::CONSULTAS), "{m}");
}

/// El marco fuera del dominio: su primer byte dice cuantas filas trae, y upstream exigia 2 con
/// `assert_eq!`. Ahora, otro valor es error con su causa.
#[test]
fn un_marco_ood_que_no_tiene_dos_filas_es_error_y_no_panico() {
    let mut b = Vec::new();
    for estados in [[3u8, 0], [2, 0]] {
        // el marco de la traza con 3 filas; el del cociente, bien
        b.clear();
        b.write_u16(estados.len() as u16);
        b.write_bytes(&estados);
        b.write_u16(2);
        b.write_bytes(&[2, 0]);
        let marco = OodFrame::read_from_bytes(&b).expect("los bytes del marco se leen");
        let r = sin_panico("leer un marco", || marco.parse::<BaseElement>(1, 0, 1).map(|_| ()));
        if estados[0] == 3 {
            let m = texto(r);
            assert!(m.starts_with(causa::MARCO_OOD), "{m}");
        }
    }
}
