//! **RFC-0008 E1 (§485): el INSTRUMENTO del cobro pendiente portable.**
//!
//! Antes de escribir el AIR del cobrador, la E1 mide lo que cuesta y fija con testigos las dos
//! reglas que el RFC ya escribio: los bits compartidos (D-B) y la foto del latido (D-F). Este
//! modulo es SOLO de tests (`cfg(test)` en `lib.rs`), con el molde de `instrumento_edad.rs`, del
//! que toma la subida proxy y la lectura de la maquina: no crece la API de la capa ni la del
//! probador.
//!
//! - **La geometria** (D-B): la cadena del compromiso -`H(receptor, sal)`, `H(., importe)`,
//!   `M(C1, X)` y la subida- y la de la meta -una permutacion con dominio y la subida-, en ciclos
//!   de `CYCLE_LENGTH` filas. En un carril piden una traza de 1024 filas; en dos carriles que
//!   comparten el bit, 512.
//! - **La receta sube a las dos raices**: la hoja que el cobrador recompone con el aviso es la
//!   que el productor de la capa escribe, y los caminos de la MISMA posicion en los dos arboles
//!   llevan los mismos bits.
//! - **Primer testigo negativo** (D-B): con los bits del pendiente, la meta de OTRA posicion no
//!   sube a la raiz de meta.
//! - **Segundo testigo negativo** (D-F): el camino que se sirve de la FOTO sube a la raiz que la
//!   cabeza firmo, y no a la del arbol vivo despues de un pago o de un cobro.
//! - **El instrumento** (`#[ignore]`, se corre a mano en release): la subida proxy de la casa en
//!   512 y en 1024 filas con `crate::proof_options()`, y el precio de la foto -clonar los dos
//!   arboles- en las tallas de la E4a. Un instrumento mide, no afirma: la puerta la juzga el
//!   asiento.

use crate::instrumento_edad::{
    camino_de, de_proc, kb, traza_de_subida, verifica, SubidaProver, LATIDO_S,
};
use crate::pending::{pending_commitment, pending_commitment_v2, refund_envelope};
use crate::sparse_tree::SparseTree;
use crate::Digest;
use stark_experiment::merkle::{
    native_merge, native_root, MerklePath, CYCLE_LENGTH, TREE_DEPTH,
};
use winterfell::math::{fields::f64::BaseElement, FieldElement};
use winterfell::Prover;

/// Ciclos de la cadena del compromiso: los dos merges de `C1`, el de `C2 = M(C1, X)` y la subida
/// por el arbol de pendientes.
const CICLOS_COMPROMISO: usize = 3 + TREE_DEPTH;
/// Ciclos de la meta: la hoja (`meta_pendiente_hoja`, una permutacion) y su subida.
const CICLOS_META: usize = 1 + TREE_DEPTH;
/// Las tallas del precio de la foto: las de la puerta de la E4a (D-E4a-4, §461).
const N_FOTO: [u64; 3] = [1024, 4096, 16384];
/// Repeticiones de cada clon; se reporta la mediana.
const REPETICIONES: usize = 5;

/// Un digest cualquiera pero distinto por `k`.
fn e(k: u64) -> Digest {
    [
        BaseElement::new(k),
        BaseElement::new(k.wrapping_mul(5) + 1),
        BaseElement::new(k.wrapping_mul(13) + 2),
        BaseElement::new(k.wrapping_mul(17) + 3),
    ]
}

/// Un libro de pendientes v2 con huecos en `0..n` y su arbol de meta, como los escribe la capa:
/// la hoja de pendientes la produce `pending_commitment_v2` y la de meta `meta_pendiente_hoja`,
/// en la MISMA posicion (`two_phase.rs`, `meta_set`). Las posiciones son las de `0..n`, como las
/// asigna `allocate_pending`.
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

/// La hoja que el COBRADOR recompone con lo que el aviso le da: `C1` con sus dos merges y
/// `C2 = M(C1, X)`. El sobre `X` lo trae el aviso, opaco; aqui se compone solo para tenerlo.
fn hoja_del_cobrador(i: u64) -> Digest {
    let c1 = pending_commitment(e(i), e(i + 7000), 100 + i);
    let x = refund_envelope(e(i + 9000), 60 + i);
    native_merge(c1, x)
}

/// El camino que el metodo de D-F sirve: el de la FOTO que el latido tomo con la cabeza, nunca el
/// del arbol vivo (que se recibe solo para que el testigo pueda discriminar).
fn camino_servido(foto: &SparseTree, _vivo: &SparseTree, p: u64) -> MerklePath {
    foto.path_for(p)
}

/// **La geometria de D-B**, atada a la profundidad y al ciclo: 35 ciclos el compromiso y 33 la
/// meta; un carril pide 1024 filas, y dos carriles con el bit compartido, 512.
#[test]
fn la_geometria_del_cobrador_es_la_del_rfc() {
    assert_eq!((TREE_DEPTH, CYCLE_LENGTH), (32, 8), "el RFC mide con 32 niveles y ciclos de 8");
    assert_eq!((CICLOS_COMPROMISO, CICLOS_META), (35, 33), "los ciclos no son los del RFC");
    let un_carril = (CICLOS_COMPROMISO + CICLOS_META) * CYCLE_LENGTH;
    let dos_carriles = CICLOS_COMPROMISO.max(CICLOS_META) * CYCLE_LENGTH;
    assert_eq!((un_carril, un_carril.next_power_of_two()), (544, 1024), "un carril");
    assert_eq!((dos_carriles, dos_carriles.next_power_of_two()), (280, 512), "dos carriles");
}

/// **La receta del cobrador sube a las dos raices**: la hoja que recompone es la que el productor
/// de la capa escribe y sube a la raiz de pendientes; la meta de la MISMA posicion sube a la de
/// meta, y los dos caminos llevan los MISMOS bits.
#[test]
fn la_receta_del_cobrador_sube_a_las_dos_raices() {
    let (pend, meta) = libro(37, |i| i % 5 != 2);
    for p in [0u64, 11, 36] {
        let c2 = hoja_del_cobrador(p);
        assert_eq!(c2, pend.leaf(p), "la receta no es la hoja que la capa escribe");
        let cam_p = pend.path_for(p);
        let cam_m = meta.path_for(p);
        assert_eq!(cam_p.is_right, cam_m.is_right, "la misma posicion lleva otros bits");
        assert_eq!(native_root(c2, &cam_p), pend.root(), "la hoja no sube a pendientes");
        let hoja_m = zk_ssl_hash::meta_pendiente_hoja(p % 5, 1000 + p);
        assert_eq!(native_root(hoja_m, &cam_m), meta.root(), "la meta no sube a su raiz");
    }
}

/// **Primer testigo negativo (D-B)**: con los bits del pendiente `p`, la meta de OTRA posicion
/// `q` no sube a la raiz de meta, ni con sus propios hermanos ni con los de `p`. Con sus propios
/// bits si sube: lo que la descarta son los bits compartidos.
#[test]
fn una_meta_de_otra_posicion_no_sube_con_los_bits_del_pendiente() {
    let (pend, meta) = libro(37, |i| i % 5 != 2);
    let (p, q) = (11u64, 13u64);
    let bits_p = pend.path_for(p).is_right;
    let cam_q = meta.path_for(q);
    let hoja_q = zk_ssl_hash::meta_pendiente_hoja(q % 5, 1000 + q);
    assert_eq!(native_root(hoja_q, &cam_q), meta.root(), "la base tiene que subir");
    assert_ne!(bits_p, cam_q.is_right, "p y q tienen que diferir en algun bit");
    let con_hermanos_de_q =
        MerklePath { siblings: cam_q.siblings.clone(), is_right: bits_p.clone() };
    assert_ne!(
        native_root(hoja_q, &con_hermanos_de_q),
        meta.root(),
        "la meta de q subio con los bits de p"
    );
    let con_hermanos_de_p = MerklePath { siblings: meta.path_for(p).siblings, is_right: bits_p };
    assert_ne!(
        native_root(hoja_q, &con_hermanos_de_p),
        meta.root(),
        "la meta de q subio por el camino de p"
    );
}

/// **Segundo testigo negativo (D-F)**: la foto del latido. Tras un pago (una hoja nueva) o un
/// cobro (una hoja que se retira) en el arbol vivo, el camino SERVIDO sigue subiendo a la raiz
/// que la cabeza firmo y no sube a la de ahora; el camino de ahora sube a la de ahora, y no a la
/// cabeza.
#[test]
fn un_camino_de_antes_de_un_pago_no_sube_a_la_raiz_de_despues() {
    let (vivo, _) = libro(37, |i| i % 5 != 2);
    let p = 11u64;
    let hoja = hoja_del_cobrador(p);
    for mueve in 0..2 {
        let foto = vivo.clone();
        let raiz_cabeza = foto.root();
        let mut ahora = vivo.clone();
        if mueve == 0 {
            ahora.set_leaf(37, pending_commitment_v2(e(1), e(2), 3, e(4), 5));
        } else {
            ahora.set_leaf(36, [BaseElement::ZERO; 4]);
        }
        assert_ne!(ahora.root(), raiz_cabeza, "el arbol vivo tenia que moverse");
        let servido = camino_servido(&foto, &ahora, p);
        assert_eq!(
            native_root(hoja, &servido),
            raiz_cabeza,
            "el camino servido no sube a la cabeza firmada"
        );
        assert_ne!(
            native_root(hoja, &servido),
            ahora.root(),
            "un camino de antes subio a la raiz de despues"
        );
        let de_ahora = ahora.path_for(p);
        assert_eq!(native_root(hoja, &de_ahora), ahora.root(), "el camino de ahora no sube");
        assert_ne!(native_root(hoja, &de_ahora), raiz_cabeza, "el de ahora subio a la cabeza");
    }
}

fn mediana(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).expect("un tiempo que no es un numero"));
    v[v.len() / 2]
}

/// **INSTRUMENTO, no comprobacion** (RFC-0008 E1, §485). Correr en release, a mano:
///
/// ```text
/// cargo test --release -p zk-ssl instrumento_del_cobro -- --ignored --nocapture
/// ```
///
/// Arriba, la maquina y la geometria derivada. **La geometria**: la subida proxy de la casa
/// (`SubidaAir`, 13 columnas) de 64 niveles (512 filas) y de 128 (1024), probada y verificada
/// con `crate::proof_options()`: tiempos y bytes. Mide el factor de las FILAS; el del ancho lo
/// acotan pruebas ya medidas (la banda, 27 columnas; el cobro v1, 55). **El precio de la foto**
/// (D-F): por cada talla, un libro con una posicion de cada cuatro vacia; clonar los dos arboles
/// (la mediana de cinco), los nodos en cache de cada uno, el `VmRSS` que la foto retiene (una
/// aproximacion del proceso, no del mapa) y un camino servido de la foto.
#[test]
#[ignore = "instrumento de medida, no comprobacion: correr a mano, en release"]
fn instrumento_del_cobro() {
    use std::time::Instant;
    let cpu = de_proc("/proc/cpuinfo", "model name");
    let nucleos = std::thread::available_parallelism().map(|x| x.get()).unwrap_or(0);
    let total = kb(&de_proc("/proc/meminfo", "MemTotal"));
    println!("E1| maquina: {cpu} . nucleos {nucleos} . MemTotal {total} kB . latido {LATIDO_S} s");
    let un_carril = (CICLOS_COMPROMISO + CICLOS_META) * CYCLE_LENGTH;
    let dos_carriles = CICLOS_COMPROMISO.max(CICLOS_META) * CYCLE_LENGTH;
    println!(
        "E1| geometria: compromiso {CICLOS_COMPROMISO} ciclos, meta {CICLOS_META} . un carril \
         {un_carril} filas -> {} . dos carriles {dos_carriles} filas -> {}",
        un_carril.next_power_of_two(),
        dos_carriles.next_power_of_two()
    );
    let opciones = crate::proof_options();
    for niveles in [64usize, 128] {
        let (hoja, camino) = camino_de(niveles);
        let traza = traza_de_subida(hoja, &camino);
        let raiz = native_root(hoja, &camino);
        let t = Instant::now();
        let prueba = SubidaProver { options: opciones.clone() }.prove(traza).expect("probar");
        let probar_s = t.elapsed().as_secs_f64();
        let bytes = prueba.to_bytes().len();
        let t = Instant::now();
        let ok = verifica(prueba, raiz, &opciones);
        let verificar_ms = t.elapsed().as_secs_f64() * 1000.0;
        println!(
            "E1| subida proxy {niveles} niveles = {} filas x 13 . probar {probar_s:.2} s . \
             verificar {verificar_ms:.1} ms . prueba {bytes} B . verifica {ok}",
            niveles * CYCLE_LENGTH
        );
    }
    for n in N_FOTO {
        let t = Instant::now();
        let (pend, meta) = libro(n, |i| i % 4 != 3);
        let libro_s = t.elapsed().as_secs_f64();
        let antes = kb(&de_proc("/proc/self/status", "VmRSS"));
        let mut tiempos = Vec::with_capacity(REPETICIONES);
        let mut foto = None;
        for _ in 0..REPETICIONES {
            let t = Instant::now();
            let f = (pend.clone(), meta.clone());
            tiempos.push(t.elapsed().as_secs_f64() * 1000.0);
            foto = Some(f);
        }
        let despues = kb(&de_proc("/proc/self/status", "VmRSS"));
        let (fp, fm) = foto.expect("una foto");
        let p = n / 2;
        let t = Instant::now();
        let servido = camino_servido(&fp, &pend, p);
        let camino_us = t.elapsed().as_secs_f64() * 1e6;
        let sube = native_root(fp.leaf(p), &servido) == pend.root();
        println!(
            "E1| foto n {n} . libro {libro_s:.1} s . nodos {} + {} . clonar los dos {:.3} ms \
             (mediana de {REPETICIONES}) . VmRSS +{} kB . camino {camino_us:.0} us . sube {sube}",
            pend.cached_nodes(),
            meta.cached_nodes(),
            mediana(tiempos),
            despues.saturating_sub(antes)
        );
        drop((fp, fm));
    }
}
