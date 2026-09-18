//! **RFC-0008 E2 (§502): el INSTRUMENTO del pago en curso portable.**
//!
//! Antes de escribir el AIR del pagador, la E2 mide lo que hereda de E1 y fija con testigos las
//! dos reglas que el RFC ya escribio (D-AD y D-AG), con el molde de `instrumento_cobro.rs`. Este
//! modulo es SOLO de tests (`cfg(test)` en `lib.rs`): no crece la API de la capa ni la del
//! probador.
//!
//! - **La geometria** (D-AF): el carril A es el del cobro -`H(receptor, sal)`, `H(., importe)`,
//!   `M(C1, X)` y la subida: 35 ciclos-; el carril B compone `X = M(refund_id, [delta, 0, 0, 0])`
//!   en el ciclo 0, lo arrastra en el 1, compone la meta en el 2 y sube: 35 ciclos, donde E1 lo
//!   tenia ocioso en los ciclos 0 y 1. Dos carriles con el bit compartido: 280 filas, traza de
//!   512, la misma que E1. La cota temporal `delta - (T - nacido)` en `[0, 2^62)` es UN segmento
//!   de 64 filas donde la banda del importe gastaba tres: el importe es exacto y publico.
//! - **La receta del pagador sube a las dos raices**: la hoja que compone con SU apertura -la
//!   pareja `(refund_id, delta)` incluida, que el cobrador recibe opaca- es la que la capa
//!   escribe, y el par ATA: otro `refund_id` u otro `delta` dan otra hoja.
//! - **Lo que el campo no distingue** (D-AG): `u64::MAX` y `2^32 - 2` dan el mismo sobre y la
//!   misma hoja, porque `refund_envelope` mete `delta` en Goldilocks. Declarado, no reparado: la
//!   letra es del RFC-0003, y su punto va en la cola.
//! - **Los dos testigos medidos** (PASTE-E2b-PRE-r2, SALIDA 20260918-164040): la frontera de
//!   `T` con la regla real de la capa, por los dos lados -en `now = nacido + delta - 1` el
//!   reembolso rebota, en `now = nacido + delta` pasa-; y el <<nunca>> que se abre con el delta
//!   reducido y muere solo en el tiempo. Prueban STARK reales: en depuracion se saltan (nota 41)
//!   y `--release` los corre.
//! - **El instrumento** (`#[ignore]`, se corre a mano en release): la prueba del cobro que el
//!   productor real escribe sobre un libro vivo -la geometria que E2 hereda- y la apertura del
//!   reembolso v2 -lo que E2 pliega en los ciclos ociosos-, en bytes y en tiempo. Un instrumento
//!   mide, no afirma: la puerta la juzga el asiento.

use crate::instrumento_edad::{de_proc, kb};
use crate::pending::{pending_commitment, pending_commitment_v2, refund_envelope};
use crate::prueba_cobro::{prueba_de_cobro_pendiente, CabezaDePendientes, FotoDelCobro};
use crate::sparse_tree::SparseTree;
use crate::tests_support::*;
use crate::two_phase::PendingNotice;
use crate::{AccountIndex, Digest, LayerError, SovereignLayer};
use stark_experiment::circuit_refund_v2 as refund2;
use stark_experiment::merkle::{native_merge, native_root, CYCLE_LENGTH, TREE_DEPTH};
use winterfell::math::{fields::f64::BaseElement, FieldElement};
use winterfell::Prover;

/// Ciclos del carril A: los dos merges de `C1`, el de `C2 = M(C1, X)` y la subida, como en E1.
const CICLOS_CARRIL_A: usize = 3 + TREE_DEPTH;
/// Ciclos del carril B del PAGO: el sobre `X` (ciclo 0), el arrastre (ciclo 1), la meta (ciclo 2)
/// y la subida. En E1 el carril B es la meta y la subida, y no hashea en los ciclos 0 y 1.
const CICLOS_CARRIL_B: usize = 3 + TREE_DEPTH;
/// Ciclos del carril B de E1: la meta y la subida (`instrumento_cobro::CICLOS_META`).
const CICLOS_CARRIL_B_E1: usize = 1 + TREE_DEPTH;
/// Filas de un segmento de bits: el de la banda (`cobro_pendiente::LARGO_SEGMENTO`).
const LARGO_SEGMENTO: usize = 64;
/// Segmentos de la banda del importe en E1: importe, importe - inferior y superior - importe.
const SEGMENTOS_E1: usize = 3;
/// Segmentos del pago: uno, `delta - (T - nacido)` en `[0, 2^62)`. El importe es exacto y publico.
const SEGMENTOS_E2: usize = 1;
/// El importe y la pareja de la conformidad, como los del banco: `delta` = 96.
const IMPORTE: u64 = 250_000;
const DELTA: u64 = 96;

/// El centinela `u64::MAX` reducido al campo Goldilocks, `p = 2^64 - 2^32 + 1`. Se deriva, como en
/// `prueba_edad.rs`; no se teclea.
fn centinela_en_el_campo() -> u64 {
    (1u64 << 32) - 2
}

/// Un digest cualquiera pero distinto por `k`.
fn e(k: u64) -> Digest {
    [
        BaseElement::new(k),
        BaseElement::new(k.wrapping_mul(5) + 1),
        BaseElement::new(k.wrapping_mul(13) + 2),
        BaseElement::new(k.wrapping_mul(17) + 3),
    ]
}

/// La clave de gasto ANCHA de una semilla, como la escribe `foto_pendientes`.
fn clave(sk: u64) -> Digest {
    [BaseElement::new(sk), BaseElement::ZERO, BaseElement::ZERO, BaseElement::ZERO]
}

/// Un libro de pendientes v2 con huecos en `0..n` y su arbol de meta, como los escribe la capa
/// (`instrumento_cobro::libro`): la hoja la produce `pending_commitment_v2` y la de meta
/// `meta_pendiente_hoja`, en la MISMA posicion.
fn libro(n: u64, vive: impl Fn(u64) -> bool) -> (SparseTree, SparseTree) {
    let mut pend = SparseTree::new();
    let mut meta = SparseTree::new();
    for i in 0..n {
        if vive(i) {
            let c2 = pending_commitment_v2(e(i), e(i + 7000), 100 + i, e(i + 9000), 60 + i);
            pend.set_leaf(i, c2);
            meta.set_leaf(i, zk_ssl_hash::meta_pendiente_hoja(i % 5, 1000 + i));
        }
    }
    (pend, meta)
}

/// La hoja que el PAGADOR compone con SU apertura: `C1` con sus dos merges y el sobre `X` que EL
/// compuso de su pareja. El cobrador recibe `X` opaco; el pagador lo produce.
fn hoja_del_pagador(i: u64) -> Digest {
    let c1 = pending_commitment(e(i), e(i + 7000), 100 + i);
    let x = refund_envelope(e(i + 9000), 60 + i);
    native_merge(c1, x)
}

/// Deja un pendiente v2 EN VUELO de alice a bob con la hoja comprometida a `delta`, y devuelve
/// (alice, receptor, f, sal, pos, nacido); el molde es `antes_de_delta_el_reembolso_v2_rebota`.
fn pendiente_en_vuelo(
    layer: &mut SovereignLayer,
    delta: u64,
) -> (AccountIndex, Digest, Digest, Digest, u64, u64) {
    let alice = open_and_fund(layer, SK_ALICE, 1_000_000);
    let bob = open_and_fund(layer, SK_BOB, 0);
    let receptor = layer.public_id_of(bob).expect("bob");
    let f = layer.public_id_of(alice).expect("alice");
    let sal = salt_de(0xE2B);
    let ea = state_of(layer, alice);
    let recibo = layer
        .send(BaseElement::new(SK_ALICE), alice, &ea, receptor, sal, 300_000)
        .expect("send");
    layer.apply_send(&recibo, alice, &ea, 300_000).expect("apply");
    let pos = recibo.notice.position;
    let (_, nacido) = layer.pending_meta_of(pos).expect("meta");
    layer.pending.set_leaf(pos, pending_commitment_v2(receptor, sal, 300_000, f, delta));
    (alice, receptor, f, sal, pos, nacido)
}

/// Avanza el registro UNA entrada cada vez (abrir una cuenta asienta), hasta `objetivo`.
fn avanza_hasta(layer: &mut SovereignLayer, objetivo: u64, semilla: &mut u64) {
    while (layer.log.len() as u64) < objetivo {
        let antes = layer.log.len();
        *semilla += 1;
        let _ = open_and_fund(layer, 0x5000 + *semilla, 0);
        assert!(layer.log.len() > antes, "abrir una cuenta tiene que asentar");
    }
}

/// **La geometria de D-AF**, atada a la profundidad y al ciclo: el carril B del pago llena los dos
/// ciclos que E1 le deja ociosos y no pasa de los 35 del carril A; la traza sigue en 512. Y la
/// cota temporal cabe en un segmento donde la banda del importe gastaba tres.
#[test]
fn la_geometria_del_pagador_es_la_de_e1_con_el_hueco_lleno() {
    assert_eq!((TREE_DEPTH, CYCLE_LENGTH), (32, 8), "el RFC mide con 32 niveles y ciclos de 8");
    assert_eq!((CICLOS_CARRIL_A, CICLOS_CARRIL_B), (35, 35), "los dos carriles del pago");
    let ociosos_en_e1 = CICLOS_CARRIL_A - CICLOS_CARRIL_B_E1;
    assert_eq!(ociosos_en_e1, 2, "los ciclos 0 y 1 del carril B de E1, donde va el sobre");
    let dos_carriles = CICLOS_CARRIL_A.max(CICLOS_CARRIL_B) * CYCLE_LENGTH;
    assert_eq!((dos_carriles, dos_carriles.next_power_of_two()), (280, 512), "la traza de E1");
    assert!(SEGMENTOS_E2 < SEGMENTOS_E1, "un segmento donde habia tres");
    assert!(SEGMENTOS_E2 * LARGO_SEGMENTO < dos_carriles, "el segmento cabe antes de la raiz");
}

/// **La receta del pagador sube a las dos raices, y el par ata**: la hoja que compone con su
/// apertura es la que la capa escribe; la meta de la MISMA posicion sube a la de meta con los
/// MISMOS bits; y otro `refund_id` u otro `delta` dan otra hoja.
#[test]
fn la_receta_del_pagador_sube_a_las_dos_raices_y_el_par_ata() {
    let (pend, meta) = libro(37, |i| i % 5 != 2);
    for p in [0u64, 11, 36] {
        let hoja = hoja_del_pagador(p);
        assert_eq!(hoja, pend.leaf(p), "la receta no es la hoja que la capa escribe");
        assert_eq!(
            hoja,
            pending_commitment_v2(e(p), e(p + 7000), 100 + p, e(p + 9000), 60 + p),
            "la receta no es el compositor v2"
        );
        let cam_p = pend.path_for(p);
        let cam_m = meta.path_for(p);
        assert_eq!(cam_p.is_right, cam_m.is_right, "la misma posicion lleva otros bits");
        assert_eq!(native_root(hoja, &cam_p), pend.root(), "la hoja no sube a pendientes");
        let hoja_m = zk_ssl_hash::meta_pendiente_hoja(p % 5, 1000 + p);
        assert_eq!(native_root(hoja_m, &cam_m), meta.root(), "la meta no sube a su raiz");
        let c1 = pending_commitment(e(p), e(p + 7000), 100 + p);
        assert_ne!(native_merge(c1, refund_envelope(e(p + 9001), 60 + p)), hoja, "otro refund_id");
        assert_ne!(native_merge(c1, refund_envelope(e(p + 9000), 61 + p)), hoja, "otro delta");
    }
}

/// **Lo que el campo no distingue** (D-AG): dos `delta` que difieren en `p` dan el mismo sobre y
/// la misma hoja. Con su prueba de vida: `u64::MAX - 1` sigue siendo otro sobre.
#[test]
fn dos_deltas_que_difieren_en_p_dan_la_misma_hoja() {
    let (r, s, f) = (e(1), e(2), e(3));
    let reducido = centinela_en_el_campo();
    assert_eq!(refund_envelope(f, u64::MAX), refund_envelope(f, reducido), "el sobre");
    assert_eq!(
        pending_commitment_v2(r, s, 7, f, u64::MAX),
        pending_commitment_v2(r, s, 7, f, reducido),
        "la hoja"
    );
    assert_ne!(refund_envelope(f, u64::MAX), refund_envelope(f, u64::MAX - 1), "la vida");
}

/// **D-AD, medida con la regla real de la capa**: en `now = nacido + delta - 1` el reembolso
/// REBOTA (`RefundTooEarly`); en `now = nacido + delta`, PASA y el dinero vuelve. Luego <<no
/// reversible antes de T>> es exactamente `T <= nacido + delta`, en epocas del registro.
#[test]
#[cfg_attr(debug_assertions, ignore = "winterfell valida grados en depuracion: juez release")]
fn la_frontera_de_t_es_nacido_mas_delta() {
    let mut layer = new_layer();
    let (alice, receptor, f, sal, pos, nacido) = pendiente_en_vuelo(&mut layer, 1);
    let mut semilla = 0u64;
    avanza_hasta(&mut layer, nacido + 3, &mut semilla);
    let now1 = layer.log.len() as u64;
    let delta1 = now1 - nacido + 1;
    layer.pending.set_leaf(pos, pending_commitment_v2(receptor, sal, 300_000, f, delta1));
    let ea = state_of(&layer, alice);
    let m = layer
        .refund_v2(
            BaseElement::new(SK_ALICE), alice, &ea, pos, receptor, sal, 300_000, f, delta1,
        )
        .expect("materiales v2");
    assert!(
        matches!(layer.apply_refund(&m), Err(LayerError::RefundTooEarly { .. })),
        "now - nacido = delta - 1: tiene que rebotar"
    );
    avanza_hasta(&mut layer, now1 + 1, &mut semilla);
    let now2 = layer.log.len() as u64;
    let delta2 = now2 - nacido;
    layer.pending.set_leaf(pos, pending_commitment_v2(receptor, sal, 300_000, f, delta2));
    let ea2 = state_of(&layer, alice);
    let m2 = layer
        .refund_v2(
            BaseElement::new(SK_ALICE), alice, &ea2, pos, receptor, sal, 300_000, f, delta2,
        )
        .expect("materiales v2");
    layer.apply_refund(&m2).expect("en now - nacido == delta el reembolso PASA");
    assert_eq!(state_of(&layer, alice).balance, 1_000_000, "el dinero vuelve");
}

/// **D-AG, medida con la regla real**: la hoja comprometida a `u64::MAX` (<<nunca>>) se abre con
/// `2^32 - 2`: la recomposicion pasa y lo unico que para el reembolso es el tiempo.
#[test]
#[cfg_attr(debug_assertions, ignore = "winterfell valida grados en depuracion: juez release")]
fn el_nunca_se_abre_con_el_delta_reducido_y_muere_solo_en_el_tiempo() {
    let mut layer = new_layer();
    let (alice, receptor, f, sal, pos, _nacido) = pendiente_en_vuelo(&mut layer, u64::MAX);
    let reducido = centinela_en_el_campo();
    let ea = state_of(&layer, alice);
    let m = layer
        .refund_v2(
            BaseElement::new(SK_ALICE), alice, &ea, pos, receptor, sal, 300_000, f, reducido,
        )
        .expect("los materiales con el delta reducido se producen");
    let r = layer.apply_refund(&m);
    assert!(
        matches!(r, Err(LayerError::RefundTooEarly { .. })),
        "con el delta reducido la recomposicion pasa y solo el tiempo lo para: {r:?}"
    );
}

/// **INSTRUMENTO, no comprobacion** (RFC-0008 E2, S502). Correr en release, a mano:
///
/// ```text
/// cargo test --release -p zk-ssl instrumento_del_pago -- --ignored --nocapture
/// ```
///
/// Arriba, la maquina y la geometria derivada. Luego **la prueba del cobro** que el productor real
/// escribe sobre un libro vivo -la traza de 512 filas que E2 hereda- en bytes y en tiempo, y
/// **la apertura del reembolso v2** -los cuatro merges de `circuit_refund_v2`, lo que E2 pliega en
/// los ciclos ociosos del carril B-. Mide el techo y el sumando; el AIR del pago se juzga contra
/// los dos.
#[test]
#[ignore = "instrumento de medida, no comprobacion: correr a mano, en release"]
fn instrumento_del_pago() {
    use std::time::Instant;
    let cpu = de_proc("/proc/cpuinfo", "model name");
    let nucleos = std::thread::available_parallelism().map(|x| x.get()).unwrap_or(0);
    let total = kb(&de_proc("/proc/meminfo", "MemTotal"));
    println!("E2| maquina: {cpu} . nucleos {nucleos} . MemTotal {total} kB");
    let dos_carriles = CICLOS_CARRIL_A.max(CICLOS_CARRIL_B) * CYCLE_LENGTH;
    println!(
        "E2| geometria: carril A {CICLOS_CARRIL_A} ciclos, carril B {CICLOS_CARRIL_B} . dos \
         carriles {dos_carriles} filas -> {} . segmentos {SEGMENTOS_E2} x {LARGO_SEGMENTO} filas",
        dos_carriles.next_power_of_two()
    );
    let opciones = crate::proof_options();

    let mut l = new_layer();
    let alice = open_and_fund(&mut l, SK_ALICE, 1_000_000);
    let bob = open_and_fund(&mut l, SK_BOB, 0);
    let id_bob = l.public_id_of(bob).expect("bob");
    let f = l.public_id_of(alice).expect("alice");
    let sal = salt_de(0xE2);
    let m = l
        .send_materials_v2(alice, id_bob, IMPORTE, sal, f, DELTA)
        .expect("materiales v2");
    let t = Instant::now();
    let recibo = crate::client::prove_send(&m, clave(SK_ALICE), opciones.clone())
        .expect("probar el envio");
    let envio_s = t.elapsed().as_secs_f64();
    let ea = state_of(&l, alice);
    l.apply_send(&recibo, alice, &ea, IMPORTE).expect("aplicar el envio");
    let aviso: PendingNotice = recibo.notice;
    let pos = aviso.position;
    let (emisor, nacido) = l.pending_meta_of(pos).expect("el pendiente tiene meta");
    let cab = CabezaDePendientes {
        seq: l.log.len() as u64,
        pending_root: l.pending.root(),
        pmeta_root: l.pending_meta_tree.root(),
    };
    let foto = FotoDelCobro {
        camino_pendiente: l.pending.path_for(pos),
        hermanos_meta: l.pending_meta_tree.path_for(pos).siblings,
        emisor,
        nacido,
    };
    let t = Instant::now();
    let sobre = prueba_de_cobro_pendiente(&cab, id_bob, &aviso, &foto, 0).expect("el cobro");
    let cobro_s = t.elapsed().as_secs_f64();
    println!(
        "E2| cobro heredado (512 filas): prueba {} B . probar {cobro_s:.2} s . envio v2 \
         {envio_s:.2} s . seq {} nacido {}",
        sobre.prueba.len(),
        sobre.seq,
        sobre.nacido
    );

    let traza = refund2::build_trace(id_bob, sal, IMPORTE, f, DELTA);
    let t = Instant::now();
    let prueba = refund2::RefundV2Prover::new(opciones).prove(traza).expect("probar la apertura");
    let abrir_s = t.elapsed().as_secs_f64();
    println!(
        "E2| apertura v2 ({} filas x {} columnas): prueba {} B . probar {abrir_s:.3} s",
        refund2::TRACE_LENGTH,
        refund2::TRACE_WIDTH,
        prueba.to_bytes().len()
    );
}
