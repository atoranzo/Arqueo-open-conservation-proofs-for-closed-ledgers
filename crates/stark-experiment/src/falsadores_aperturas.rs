//! §715, la maleabilidad de los bytes de la prueba: las aperturas de mas, con el compromiso
//! de Merkle de upstream. Los testigos de las comprobaciones del canal del fork
//! (`crates/winter-verifier/src/channel.rs`): una fila por posicion consultada en la traza y en el
//! cociente, y un grupo por posicion plegada en cada capa FRI.
//!
//! `get_root` de `winter-crypto` 0.13.1 recorre los indices y no mira si sobran hojas: una fila de
//! mas -con una consulta unica mas- o un grupo de mas en una capa FRI no se leen, y la prueba
//! verificaba con otros bytes. Los vectores de la casa usan `MerkleConSal`, cuyo `verify_many`
//! exige ya tantos items como indices y da la MISMA variante: alli, sin el canal, nada falla.
//! Aqui el compromiso es `MerkleTree<Blake3_256>`, sin sal, y el canal es la unica defensa.
//!
//! Montaje: `WorkAir` (`lib.rs`) con 16 pasos, expansion 8 y resto de grado 7: un dominio de 128,
//! una capa FRI, y menos consultas unicas que las 32 de las opciones, para que quepa una fila mas.

use winterfell::crypto::{hashers::Blake3_256, DefaultRandomCoin, MerkleTree};
use winterfell::math::fields::f128::BaseElement;
use winterfell::math::FieldElement;
use winterfell::{
    verify, AcceptableOptions, BatchingMethod, FieldExtension, Proof, ProofOptions, Prover,
};
use winter_prover::{proof::Queries, Deserializable, Serializable, SliceReader};

use crate::{build_trace, WorkAir, WorkProver};

type Blake3 = Blake3_256<BaseElement>;

/// El plegado FRI de las opciones de abajo: un grupo son 8 elementos.
const PLEGADO: usize = 8;

fn opciones() -> ProofOptions {
    ProofOptions::new(
        32,
        8,
        0,
        FieldExtension::None,
        PLEGADO,
        7,
        BatchingMethod::Linear,
        BatchingMethod::Linear,
    )
}

/// Una prueba honesta de `WorkAir` y su resultado publico.
fn honesta() -> (Proof, BaseElement) {
    let trace = build_trace(BaseElement::new(3), 16);
    let prover = WorkProver::new(opciones());
    let resultado = prover.get_pub_inputs(&trace);
    (prover.prove(trace).expect("la prueba honesta"), resultado)
}

/// El veredicto del verificador del fork, con el compromiso de upstream: `Ok` o el `Debug` del
/// error.
fn juzgar(p: Proof, resultado: BaseElement) -> Result<(), String> {
    let aceptadas = AcceptableOptions::OptionSet(vec![opciones()]);
    verify::<WorkAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
        p,
        resultado,
        &aceptadas,
    )
    .map_err(|e| format!("{e:?}"))
}

/// `q` con una copia de su primera fila al final: los valores crecen una fila, la apertura no
/// cambia (sin sal no hay nada por fila en ella).
fn con_una_fila_de_mas(q: &Queries, filas: usize) -> Queries {
    let b = q.to_bytes();
    let mut r = SliceReader::new(&b);
    let mut valores = Vec::<u8>::read_from(&mut r).expect("los valores");
    let apertura = Vec::<u8>::read_from(&mut r).expect("la apertura");
    assert_eq!(valores.len() % filas, 0, "una tabla de {filas} filas");
    let fila = valores.len() / filas;
    valores.extend_from_within(..fila);
    let mut o = Vec::new();
    valores.write_into(&mut o);
    apertura.write_into(&mut o);
    Queries::read_from_bytes(&o).expect("la tabla con una fila de mas")
}

/// La prueba con una fila de mas en cada tabla -la traza y el cociente- y una consulta unica mas.
fn fila_de_mas(mut p: Proof) -> Proof {
    let unicas = p.num_unique_queries as usize;
    assert!(unicas < p.options().num_queries(), "no cabe una consulta unica mas: {unicas}");
    p.trace_queries = p.trace_queries.iter().map(|q| con_una_fila_de_mas(q, unicas)).collect();
    p.constraint_queries = con_una_fila_de_mas(&p.constraint_queries, unicas);
    p.num_unique_queries += 1;
    p
}

/// La prueba con una copia del primer grupo de la capa FRI 0 al final de sus valores. Los bytes
/// de `FriProof`: el numero de capas (`u8`) y, por capa, los valores y la apertura, cada uno con
/// su longitud `u32`; detras, el resto y las particiones.
fn grupo_de_mas(mut p: Proof) -> Proof {
    let b = p.fri_proof.to_bytes();
    assert!(b[0] >= 1, "sin capas FRI");
    let largo = u32::from_le_bytes(b[1..5].try_into().unwrap()) as usize;
    let grupo = PLEGADO * BaseElement::ELEMENT_BYTES;
    assert_eq!(largo % grupo, 0, "la capa 0 son grupos enteros");
    let mut nb = b[..1].to_vec();
    nb.extend(((largo + grupo) as u32).to_le_bytes());
    nb.extend_from_slice(&b[5..5 + largo]);
    nb.extend_from_slice(&b[5..5 + grupo]);
    nb.extend_from_slice(&b[5 + largo..]);
    p.fri_proof = Deserializable::read_from_bytes(&nb).expect("la FRI con un grupo de mas");
    p
}

/// Una fila de mas en la traza y en el cociente, con una consulta unica mas, es
/// `TraceQueryDoesNotMatchCommitment`: el canal la mira antes que la apertura. La honesta
/// verifica. Falsador: sin las comprobaciones de filas del canal, la prueba con la fila de mas
/// VERIFICA (medido).
#[test]
fn una_fila_de_mas_sin_sal_la_para_el_canal() {
    let (p, resultado) = honesta();
    assert_eq!(juzgar(p.clone(), resultado), Ok(()), "la honesta verifica");
    let m = fila_de_mas(p.clone());
    assert_ne!(m.to_bytes(), p.to_bytes(), "otros bytes");
    assert_eq!(juzgar(m, resultado), Err("TraceQueryDoesNotMatchCommitment".to_string()));
}

/// Un grupo de mas en la capa FRI 0 es `FriVerificationFailed(LayerCommitmentMismatch)`.
/// Falsador: sin la comprobacion de grupos del canal (`read_layer_queries`), VERIFICA (medido).
#[test]
fn un_grupo_fri_de_mas_sin_sal_lo_para_el_canal() {
    let (p, resultado) = honesta();
    let m = grupo_de_mas(p.clone());
    assert_ne!(m.to_bytes(), p.to_bytes(), "otros bytes");
    assert_eq!(
        juzgar(m, resultado),
        Err("FriVerificationFailed(LayerCommitmentMismatch)".to_string())
    );
}
