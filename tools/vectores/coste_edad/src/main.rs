//! §717, el coste del juez de la edad. Un paquete de EDAD sin la clave del operador, sin
//! libro y sin probar: una prueba FABRICADA de `m = m_canonico(nextPending)` (solo forma: contexto,
//! compromisos, tablas, marco OOD y FRI con los anchos de `m`, valores cualesquiera) y una cabeza
//! v5 FIRMADA con una clave XMSS propia de semilla fija, con las raices que suben de las subraices
//! que el paquete declara. El mando del kit verifica la firma con la clave que trae la cabeza.
//!
//! Uso: `coste-edad <m> <nextPending> <salida.json>`; `<m>` es la de la prueba fabricada (la de
//! `nextPending` salvo para fabricar una prueba que no casa). Determinista: el mismo fichero byte a
//! byte en cada corrida.

use winter_air::proof::{Commitments, Context, OodFrame, Queries};
use winter_air::{Air, Oculta, TraceInfo};
use winter_crypto::hashers::Blake3_256;
use winter_crypto::{BatchMerkleProof, Hasher};
use winter_math::fields::f64::BaseElement;
use winter_math::fields::QuadExtension;
use winter_math::FieldElement;
use winter_verifier::{ByteWriter, Deserializable, Serializable};
use xmss::KeyPair;
use zk_ssl_air::sal::{MerkleConSal, VariasConSal};
use zk_ssl_air::familias::marca_v2_de;
use zk_ssl_air::{
    m_canonico, opciones, raiz_desde_subraiz, EdadAir, EdadPublicInputs, ALEATORIOS, ANCHO,
    ANCHO_AUX, CICLO, COLUMNAS_ALEATORIAS, PROFUNDIDAD,
};
use zk_ssl_hash::{digest_to_bytes, epoch_digest_v5, Digest};
use zk_ssl_verify::{preambulo, Conjunto};

type E = QuadExtension<BaseElement>;
type B3 = Blake3_256<BaseElement>;

const SP: Digest = [BaseElement::new(11); 4];
const SM: Digest = [BaseElement::new(22); 4];

fn pi_de(m: u32) -> EdadPublicInputs {
    let n = 1u64 << m;
    let (seq, t, emisor, todos, k) = (1000, 70, 0, true, 0);
    EdadPublicInputs { subraiz_pend: SP, subraiz_meta: SM, m, n, seq, t, emisor, todos, k }
}

/// La forma de la casa desde el §914 (zkssl/0.5): las dos columnas aleatorias y la marca v2 de la
/// edad (familia 14, version 1, perfil 1, m = 64, dos columnas), para que el paquete pase el paso 1
/// del juez (la marca, RFC-0018 D-G) y caiga donde el vector dice: en el enunciado o en la prueba.
fn info_de(m: u32) -> TraceInfo {
    let l = 2 * (CICLO << m);
    let marca = marca_v2_de("EdadAir", COLUMNAS_ALEATORIAS as u8).expect("la edad esta en el registro").escribir();
    TraceInfo::new_multi_segment(ANCHO + COLUMNAS_ALEATORIAS, ANCHO_AUX, ALEATORIOS, l, marca)
}

fn digest(k: u8) -> <B3 as Hasher>::Digest {
    B3::hash(&[k, 0x42])
}

fn lote(prof: u32, q: usize) -> Vec<u8> {
    let nodos = vec![vec![digest(7); prof as usize]; q];
    let mut b = BatchMerkleProof::<B3> { nodes: nodos, depth: prof as u8 }.to_bytes();
    b.write_u32(q as u32);
    for i in 0..q {
        b.write(digest(i as u8));
    }
    b
}

fn tabla<F: FieldElement>(prof: u32, q: usize, fila: Vec<F>) -> Queries {
    let l = VariasConSal::<B3>::read_from_bytes(&lote(prof, q)).expect("lote");
    Queries::new::<B3, F, MerkleConSal<B3>>(l, vec![fila; q])
}

/// La prueba fabricada de `m` (la misma que `tests_coste` de `zk-ssl-air`).
fn fabricar(m: u32) -> Vec<u8> {
    let opts = opciones();
    let air = Oculta::<EdadAir>::new(info_de(m), pi_de(m), opts.clone());
    let lde = air.lde_domain_size();
    let fri = opts.to_fri_options();
    let capas = fri.num_fri_layers(lde);
    let ncomp = air.context().num_constraint_composition_columns();
    let nc = air.context().num_transition_constraints() + 16;
    let q = 42usize;
    let ld = lde.ilog2();
    let ctx = Context::new::<BaseElement>(info_de(m), opts, nc).expect("contexto");
    let mut b = ctx.to_bytes();
    b.write_u8(q as u8);
    let fri_raices = (0..=capas).map(|i| digest(10 + i as u8)).collect();
    b.write(Commitments::new::<B3>(vec![digest(1), digest(2)], digest(3), fri_raices));
    b.write(tabla(ld, q, vec![BaseElement::new(5); ANCHO + COLUMNAS_ALEATORIAS]));
    b.write(tabla(ld, q, vec![E::from(BaseElement::new(6)); ANCHO_AUX]));
    b.write(tabla(ld, q, vec![E::from(BaseElement::new(8)); ncomp]));
    let mut ood: Vec<u8> = Vec::new();
    for (ancho, base) in [(ANCHO + COLUMNAS_ALEATORIAS + ANCHO_AUX, 1u64), (ncomp, 3)] {
        let mut marco: Vec<u8> = vec![2];
        for i in 0..2 * ancho {
            marco.write(E::from(BaseElement::new(i as u64 + base)));
        }
        ood.write_u16(marco.len() as u16);
        ood.write_bytes(&marco);
    }
    b.write(OodFrame::read_from_bytes(&ood).expect("ood"));
    b.write_u8(capas as u8);
    let mut dom = ld;
    for _ in 0..capas {
        dom -= fri.folding_factor().ilog2();
        let mut vals: Vec<u8> = Vec::new();
        for _ in 0..q * fri.folding_factor() {
            vals.write(E::from(BaseElement::new(9)));
        }
        let caminos = lote(dom, q);
        b.write_u32(vals.len() as u32);
        b.write_bytes(&vals);
        b.write_u32(caminos.len() as u32);
        b.write_bytes(&caminos);
    }
    let mut resto: Vec<u8> = Vec::new();
    for _ in 0..32 {
        resto.write(E::from(BaseElement::new(4)));
    }
    b.write_u16(resto.len() as u16);
    b.write_bytes(&resto);
    b.write_u8(0);
    b.write_u64(0);
    b
}

fn hex(b: &[u8]) -> String {
    let mut s = String::from("0x");
    for x in b {
        s.push_str(&format!("{x:02x}"));
    }
    s
}

fn hd(d: &Digest) -> String {
    hex(&digest_to_bytes(d))
}

fn q(x: u64) -> String {
    format!("{x:#x}")
}

fn dg(k: u64) -> Digest {
    [BaseElement::new(k), BaseElement::new(k + 1), BaseElement::new(k + 2), BaseElement::new(k + 3)]
}

fn main() {
    let mut a = std::env::args().skip(1);
    let m: u32 = a.next().expect("m").parse().expect("m");
    let next_pending: u64 = a.next().expect("nextPending").parse().expect("nextPending");
    let salida = a.next().expect("salida.json");
    let prueba = fabricar(m);

    let seq = 1000u64;
    let sube = m_canonico(next_pending).min(PROFUNDIDAD as u32);
    let pending_root = raiz_desde_subraiz(SP, sube, PROFUNDIDAD);
    let pmeta_root = raiz_desde_subraiz(SM, sube, PROFUNDIDAD);
    let (accounts, frozen, chain, acuses, mmr, cons, params) =
        (dg(100), dg(200), dg(300), dg(400), dg(500), dg(600), dg(700));
    let (n, mmr_size, cons_count, next_index, supply) = (3u64, 5u64, 0u64, 3u64, 1_000_000u64);
    let ed = epoch_digest_v5(
        seq, accounts, pending_root, frozen, chain, acuses, n, mmr, mmr_size, cons, cons_count,
        params, pmeta_root, next_pending, next_index, supply,
    );
    let ed_b = digest_to_bytes(&ed);

    // Una clave XMSS^MT de test (el conjunto de la casa), de semilla fija, y la firma de la cabeza.
    let mut semilla = [0u8; 96];
    for (i, b) in semilla.iter_mut().enumerate() {
        *b = (i as u8).wrapping_mul(29).wrapping_add(7);
    }
    let mut kp = KeyPair::<Conjunto>::from_seed(&semilla).expect("clave");
    let pk = kp.verifying_key().as_ref().to_vec();
    let firma = kp.signing_key().sign(&preambulo(5, &ed_b)).expect("firmar");

    let cabeza = serde_json::json!({
        "available": true,
        "formatVersion": q(5),
        "seq": q(seq),
        "accountsRoot": hd(&accounts),
        "pendingRoot": hd(&pending_root),
        "frozenRoot": hd(&frozen),
        "chainDigest": hd(&chain),
        "acusesRoot": hd(&acuses),
        "n": q(n),
        "mmrRoot": hd(&mmr),
        "mmrSize": q(mmr_size),
        "consRoot": hd(&cons),
        "consCount": q(cons_count),
        "paramsDigest": hd(&params),
        "pmetaRoot": hd(&pmeta_root),
        "nextPending": q(next_pending),
        "nextIndex": q(next_index),
        "totalSupply": q(supply),
        "epochDigest": hex(&ed_b),
        "publicKey": hex(&pk),
        "signature": hex(firma.as_ref()),
        // el declarado va por encima del embebido (0): lo que el productor de la casa hace
        "index": q(1),
    });
    let paquete = serde_json::json!({
        "v": 1,
        "tipo": "edad",
        "cabeza": cabeza,
        "enunciado": {"t": q(70), "k": q(0)},
        "subraices": {"pendientes": hd(&SP), "meta": hd(&SM)},
        "prueba": hex(&prueba),
    });
    let texto = serde_json::to_string(&paquete).expect("json");
    std::fs::write(&salida, &texto).expect("escribir");
    let (bp, bt) = (prueba.len(), texto.len());
    println!("PAQUETE\tm\t{m}\tnextPending\t{next_pending}\tprueba\t{bp}\tpaquete\t{bt}");
}
