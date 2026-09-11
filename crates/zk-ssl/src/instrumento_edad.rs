//! **RFC-0007 E4a (§461): el INSTRUMENTO de la puerta de la prueba de edad.**
//!
//! La prueba de edad (D-E del RFC) reconstruye, dentro de un circuito, la raiz de los arboles de
//! pendientes y de meta sobre el RANGO `0..next_pending`. Antes de escribir su AIR, la E4a mide
//! lo que cuesta. Este modulo es SOLO de tests (`cfg(test)` en `lib.rs`): no crece la API de la
//! capa ni la del probador, y no toca `merkle.rs`, que usan cuatro circuitos.
//!
//! Tres piezas:
//!
//! - **El recuento** (`hashes_de_rango`): cuantos `native_merge` cuesta la raiz del rango `0..n`
//!   en un arbol de profundidad `p`. Depende SOLO de `n`: los huecos son la hoja cero y se hashean
//!   igual. Lo ata a la capa `raiz_de_rango`, que la recompone con la MISMA convencion de
//!   subarbol vacio que `SparseTree` y se cruza contra su `root()`.
//! - **El testigo negativo** que el RFC promete en Seguridad: una posicion viva por encima de la
//!   marca rompe la reconstruccion del rango.
//! - **El AIR proxy** (`SubidaAir`): la subida Merkle de la casa con la PROFUNDIDAD como
//!   parametro. No copia una sola restriccion: envuelve `MerkleAir` y le delega las transiciones
//!   y las columnas periodicas; solo mueve la asercion de la raiz a la ULTIMA fila de la traza,
//!   que en `MerkleAir` es la 255 fija. Mismo hash, mismas 8 filas por nivel, mismo ancho de 13.
//!   Mide la parte DOMINANTE de la prueba de edad -los hashes de los dos arboles-, no la prueba
//!   entera: las aperturas de cada hoja viva y la comparacion de edad van aparte, en celdas, y
//!   se declaran como proyeccion.
//!
//! El instrumento (`#[ignore]`, se corre a mano en release) prueba y verifica la subida en las
//! tallas de los tres `n` objetivo con `crate::proof_options()`, y reporta tiempos, bytes, el pico
//! de memoria del proceso y la maquina. La puerta la juzga el asiento, no este fichero: un
//! instrumento mide, no afirma.

use crate::pending::pending_commitment;
use crate::sparse_tree::SparseTree;
use crate::Digest;
use stark_experiment::merkle::{
    native_merge, native_root, MerkleAir, MerklePath, MerklePublicInputs, CYCLE_LENGTH,
    TRACE_WIDTH,
};
use stark_experiment::rescue_hash::{NUM_ROUNDS, STATE_WIDTH};
use winterfell::crypto::hashers::{Blake3_256, Rp64_256};
use winterfell::crypto::{DefaultRandomCoin, MerkleTree};
use winterfell::math::{fields::f64::BaseElement, FieldElement};
use winterfell::matrix::ColMatrix;
use winterfell::{
    verify, AcceptableOptions, Air, AirContext, Assertion, AuxRandElements, CompositionPoly,
    CompositionPolyTrace, ConstraintCompositionCoefficients, DefaultConstraintCommitment,
    DefaultConstraintEvaluator, DefaultTraceLde, EvaluationFrame, PartitionOptions, Proof,
    ProofOptions, Prover, StarkDomain, Trace, TraceInfo, TracePolyTable, TraceTable,
};

type Blake3 = Blake3_256<BaseElement>;

/// La columna del bit de direccion en la traza de `merkle.rs` (la 13.a, tras el estado).
const BIT_COL: usize = STATE_WIDTH;

/// Los `n` objetivo de la puerta (D-E4a-4, §461): 1024, 4096 y 16384 posiciones.
const N_OBJETIVO: [u64; 3] = [1024, 4096, 16384];

/// El latido por defecto del nodo (`zk-ssl-node/src/latido.rs`, §121), que es el tiempo de la
/// puerta (D-E4a-1). La capa no depende del nodo, asi que el valor se DECLARA aqui con su fuente;
/// solo lo lee la impresion del instrumento, que no afirma nada.
const LATIDO_S: f64 = 60.0;

/// **Cuantos `native_merge` cuesta la raiz del rango `0..n`** en un arbol de profundidad `p`,
/// con los subarboles vacios como constantes. Nivel a nivel, `ceil(len/2)` mientras quede mas de
/// un nodo; despues, uno por cada nivel que falte hasta la raiz (la espina). `n >= 1`.
pub(crate) fn hashes_de_rango(n: u64, p: usize) -> u64 {
    assert!(n >= 1, "un rango vacio no tiene raiz que reconstruir");
    let (mut len, mut total, mut nivel) = (n, 0u64, 0usize);
    while len > 1 {
        len = (len + 1) / 2;
        total += len;
        nivel += 1;
    }
    total + (p - nivel) as u64
}

/// **La raiz del rango `0..hojas.len()`**, recompuesta a mano con la MISMA convencion que
/// `SparseTree`: la hoja libre es el digest cero y el subarbol vacio de nivel `k` es
/// `merge(vacio[k-1], vacio[k-1])`. Devuelve la raiz y cuantos `native_merge` gasto (sin contar
/// las constantes de vacio, que son publicas).
pub(crate) fn raiz_de_rango(hojas: &[Digest], p: usize) -> (Digest, u64) {
    assert!(!hojas.is_empty(), "un rango vacio no tiene raiz que reconstruir");
    let cero: Digest = [BaseElement::ZERO; 4];
    let mut vacio = vec![cero];
    for k in 1..=p {
        vacio.push(native_merge(vacio[k - 1], vacio[k - 1]));
    }
    let mut nivel_actual: Vec<Digest> = hojas.to_vec();
    let mut nivel = 0usize;
    let mut cuenta = 0u64;
    while nivel_actual.len() > 1 {
        let mut siguiente = Vec::with_capacity((nivel_actual.len() + 1) / 2);
        for par in nivel_actual.chunks(2) {
            let derecho = if par.len() == 2 { par[1] } else { vacio[nivel] };
            siguiente.push(native_merge(par[0], derecho));
            cuenta += 1;
        }
        nivel_actual = siguiente;
        nivel += 1;
    }
    let mut nodo = nivel_actual[0];
    while nivel < p {
        nodo = native_merge(nodo, vacio[nivel]);
        cuenta += 1;
        nivel += 1;
    }
    (nodo, cuenta)
}

/// Un digest cualquiera pero distinto por `k`, para rellenar sin repetir.
fn d(k: u64) -> Digest {
    [
        BaseElement::new(k),
        BaseElement::new(k.wrapping_mul(3) + 1),
        BaseElement::new(k.wrapping_mul(7) + 2),
        BaseElement::new(k.wrapping_mul(11) + 3),
    ]
}

/// Las hojas del rango `0..n` de un arbol de pendientes con HUECOS: la posicion `i` vive si
/// `vive(i)`, y su hoja la produce `pending_commitment` -el productor real de la capa-.
fn rango_de_pendientes(n: u64, vive: impl Fn(u64) -> bool) -> (SparseTree, Vec<Digest>) {
    let mut arbol = SparseTree::new();
    let mut hojas = Vec::with_capacity(n as usize);
    for i in 0..n {
        let hoja = if vive(i) {
            pending_commitment(d(i), d(i + 1000), 10 + i)
        } else {
            [BaseElement::ZERO; 4]
        };
        if vive(i) {
            arbol.set_leaf(i, hoja);
        }
        hojas.push(hoja);
    }
    (arbol, hojas)
}

// ------------------------------------------------------------------ el AIR proxy

/// **La subida de la casa, de profundidad variable.** Envuelve `MerkleAir` sin copiarle una
/// restriccion: el contexto, las columnas periodicas y las transiciones son las suyas. Lo unico
/// propio es la asercion de la raiz, que va a la ULTIMA fila de la traza (`MerkleAir` la fija en
/// la 255, la de sus 32 niveles).
pub(crate) struct SubidaAir {
    interior: MerkleAir,
    raiz: Digest,
}

impl Air for SubidaAir {
    type BaseField = BaseElement;
    type PublicInputs = MerklePublicInputs;

    fn new(trace_info: TraceInfo, pub_inputs: Self::PublicInputs, options: ProofOptions) -> Self {
        let raiz = pub_inputs.root;
        SubidaAir { interior: MerkleAir::new(trace_info, pub_inputs, options), raiz }
    }

    fn context(&self) -> &AirContext<Self::BaseField> {
        self.interior.context()
    }

    fn get_periodic_column_values(&self) -> Vec<Vec<Self::BaseField>> {
        self.interior.get_periodic_column_values()
    }

    fn evaluate_transition<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        frame: &EvaluationFrame<E>,
        periodic_values: &[E],
        result: &mut [E],
    ) {
        self.interior.evaluate_transition(frame, periodic_values, result)
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        let ultima = self.trace_length() - 1;
        let mut a = Vec::with_capacity(8);
        for i in 0..4 {
            a.push(Assertion::single(i, 0, BaseElement::ZERO));
        }
        for i in 0..4 {
            a.push(Assertion::single(4 + i, ultima, self.raiz[i]));
        }
        a
    }
}

/// La traza de la subida de `hermanos.len()` niveles: la de `merkle::build_trace` con la
/// profundidad como parametro. La longitud, `8 * niveles`, tiene que ser potencia de dos.
pub(crate) fn traza_de_subida(hoja: Digest, camino: &MerklePath) -> TraceTable<BaseElement> {
    let niveles = camino.siblings.len();
    assert_eq!(niveles, camino.is_right.len());
    let largo = niveles * CYCLE_LENGTH;
    assert!(largo.is_power_of_two(), "la traza tiene que medir una potencia de dos: {largo}");
    let hermanos = camino.siblings.clone();
    let derecha = camino.is_right.clone();
    let mut traza = TraceTable::new(TRACE_WIDTH, largo);
    traza.fill(
        |s| {
            for x in s.iter_mut() {
                *x = BaseElement::ZERO;
            }
            if derecha[0] {
                s[4..8].copy_from_slice(&hermanos[0]);
                s[8..12].copy_from_slice(&hoja);
            } else {
                s[4..8].copy_from_slice(&hoja);
                s[8..12].copy_from_slice(&hermanos[0]);
            }
            s[BIT_COL] = if derecha[0] { BaseElement::ONE } else { BaseElement::ZERO };
        },
        |paso, s| {
            let fase = paso % CYCLE_LENGTH;
            if fase < NUM_ROUNDS {
                let mut e: [BaseElement; STATE_WIDTH] = s[..STATE_WIDTH].try_into().unwrap();
                Rp64_256::apply_round(&mut e, fase);
                s[..STATE_WIDTH].copy_from_slice(&e);
            } else {
                let digest: Digest = [s[4], s[5], s[6], s[7]];
                let sig = (paso + 1) / CYCLE_LENGTH;
                for x in s.iter_mut() {
                    *x = BaseElement::ZERO;
                }
                if sig < niveles {
                    if derecha[sig] {
                        s[4..8].copy_from_slice(&hermanos[sig]);
                        s[8..12].copy_from_slice(&digest);
                    } else {
                        s[4..8].copy_from_slice(&digest);
                        s[8..12].copy_from_slice(&hermanos[sig]);
                    }
                    s[BIT_COL] = if derecha[sig] { BaseElement::ONE } else { BaseElement::ZERO };
                }
            }
        },
    );
    traza
}

pub(crate) struct SubidaProver {
    options: ProofOptions,
}

impl Prover for SubidaProver {
    type BaseField = BaseElement;
    type Air = SubidaAir;
    type Trace = TraceTable<BaseElement>;
    type HashFn = Blake3;
    type VC = MerkleTree<Blake3>;
    type RandomCoin = DefaultRandomCoin<Blake3>;
    type TraceLde<E: FieldElement<BaseField = Self::BaseField>> =
        DefaultTraceLde<E, Self::HashFn, Self::VC>;
    type ConstraintEvaluator<'a, E: FieldElement<BaseField = Self::BaseField>> =
        DefaultConstraintEvaluator<'a, Self::Air, E>;
    type ConstraintCommitment<E: FieldElement<BaseField = Self::BaseField>> =
        DefaultConstraintCommitment<E, Self::HashFn, Self::VC>;

    fn get_pub_inputs(&self, trace: &Self::Trace) -> MerklePublicInputs {
        let ultima = trace.length() - 1;
        MerklePublicInputs {
            root: [
                trace.get(4, ultima),
                trace.get(5, ultima),
                trace.get(6, ultima),
                trace.get(7, ultima),
            ],
        }
    }

    fn options(&self) -> &ProofOptions {
        &self.options
    }

    fn new_trace_lde<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        trace_info: &TraceInfo,
        main_trace: &ColMatrix<Self::BaseField>,
        domain: &StarkDomain<Self::BaseField>,
        partition_option: PartitionOptions,
    ) -> (Self::TraceLde<E>, TracePolyTable<E>) {
        DefaultTraceLde::new(trace_info, main_trace, domain, partition_option)
    }

    fn new_evaluator<'a, E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        air: &'a Self::Air,
        aux_rand_elements: Option<AuxRandElements<E>>,
        composition_coefficients: ConstraintCompositionCoefficients<E>,
    ) -> Self::ConstraintEvaluator<'a, E> {
        DefaultConstraintEvaluator::new(air, aux_rand_elements, composition_coefficients)
    }

    fn build_constraint_commitment<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        composition_poly_trace: CompositionPolyTrace<E>,
        num_constraint_composition_columns: usize,
        domain: &StarkDomain<Self::BaseField>,
        partition_options: PartitionOptions,
    ) -> (Self::ConstraintCommitment<E>, CompositionPoly<E>) {
        DefaultConstraintCommitment::new(
            composition_poly_trace,
            num_constraint_composition_columns,
            domain,
            partition_options,
        )
    }
}

/// Un camino de `niveles` con hermanos distintos y direcciones alternas (las dos ramas).
fn camino_de(niveles: usize) -> (Digest, MerklePath) {
    let hermanos: Vec<Digest> = (0..niveles).map(|i| d(5000 + i as u64)).collect();
    let derecha: Vec<bool> = (0..niveles).map(|i| i % 3 == 0).collect();
    (d(42), MerklePath { siblings: hermanos, is_right: derecha })
}

fn verifica(proof: Proof, raiz: Digest, opciones: &ProofOptions) -> bool {
    let aceptadas = AcceptableOptions::OptionSet(vec![opciones.clone()]);
    verify::<SubidaAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
        proof,
        MerklePublicInputs { root: raiz },
        &aceptadas,
    )
    .is_ok()
}

// ------------------------------------------------------------------ los testigos

/// El recuento y la raiz del rango son los del arbol real: un arbol de pendientes de 37
/// posiciones con huecos, hojas de `pending_commitment`, profundidad 32.
#[test]
fn la_raiz_del_rango_es_la_del_arbol_y_su_recuento_el_de_la_formula() {
    let n = 37u64;
    let (arbol, hojas) = rango_de_pendientes(n, |i| i % 5 != 2);
    let (raiz, cuenta) = raiz_de_rango(&hojas, 32);
    assert_eq!(raiz, arbol.root(), "la raiz del rango no es la del SparseTree");
    assert_eq!(cuenta, hashes_de_rango(n, 32), "el recuento no es el de la formula");
}

/// El recuento depende SOLO de `n`: dos rellenos distintos del mismo rango -otros huecos, otras
/// hojas, las de meta (`meta_pendiente_hoja`)- gastan los mismos hashes, y los dos son los del
/// arbol real.
#[test]
fn el_recuento_depende_solo_de_n() {
    let n = 100u64;
    let (a1, h1) = rango_de_pendientes(n, |i| i % 3 == 0);
    let mut a2 = SparseTree::new();
    let mut h2 = Vec::new();
    for i in 0..n {
        let hoja = if i % 7 == 1 {
            [BaseElement::ZERO; 4]
        } else {
            zk_ssl_hash::meta_pendiente_hoja(i, 3 * i)
        };
        if i % 7 != 1 {
            a2.set_leaf(i, hoja);
        }
        h2.push(hoja);
    }
    let (r1, c1) = raiz_de_rango(&h1, 32);
    let (r2, c2) = raiz_de_rango(&h2, 32);
    assert_eq!((r1, r2), (a1.root(), a2.root()), "las raices del rango no son las del arbol");
    assert_eq!(c1, c2, "dos rellenos del mismo n gastan hashes distintos");
    assert_eq!(c1, hashes_de_rango(n, 32));
    assert_ne!(r1, r2, "los dos rellenos tienen que ser DISTINTOS o el test no discrimina");
}

/// **El testigo negativo del RFC (Seguridad):** una posicion viva por ENCIMA de la marca rompe la
/// reconstruccion del rango: la raiz de `0..n` ya no es la del arbol.
#[test]
fn una_posicion_viva_por_encima_de_la_marca_rompe_el_rango() {
    let n = 37u64;
    let (mut arbol, hojas) = rango_de_pendientes(n, |i| i % 5 != 2);
    assert_eq!(raiz_de_rango(&hojas, 32).0, arbol.root(), "la base tiene que cuadrar");
    arbol.set_leaf(n + 5, pending_commitment(d(9), d(10), 11));
    let tras = raiz_de_rango(&hojas, 32).0;
    assert_ne!(tras, arbol.root(), "una viva fuera del rango paso desapercibida");
}

/// La subida de profundidad variable prueba y verifica con las opciones de la casa, y su traza
/// acaba en la raiz nativa del camino.
#[test]
fn la_subida_variable_prueba_y_verifica() {
    let (hoja, camino) = camino_de(16);
    let traza = traza_de_subida(hoja, &camino);
    let raiz = native_root(hoja, &camino);
    let ultima = traza.length() - 1;
    for i in 0..4 {
        assert_eq!(traza.get(4 + i, ultima), raiz[i], "la traza no acaba en la raiz nativa");
    }
    let opciones = crate::proof_options();
    let prueba = SubidaProver { options: opciones.clone() }.prove(traza).expect("probar");
    assert!(verifica(prueba, raiz, &opciones), "una subida valida no verifica");
}

/// El falsador de la subida: la misma prueba contra OTRA raiz no verifica.
#[test]
fn la_subida_variable_con_otra_raiz_no_verifica() {
    let (hoja, camino) = camino_de(16);
    let traza = traza_de_subida(hoja, &camino);
    let opciones = crate::proof_options();
    let prueba = SubidaProver { options: opciones.clone() }.prove(traza).expect("probar");
    assert!(!verifica(prueba, d(999_999), &opciones), "una raiz ajena verifico");
}

// ------------------------------------------------------------------ el instrumento

fn de_proc(fichero: &str, clave: &str) -> String {
    std::fs::read_to_string(fichero)
        .ok()
        .and_then(|t| t.lines().find(|l| l.starts_with(clave)).map(|l| l.to_string()))
        .unwrap_or_else(|| format!("{clave} (no disponible)"))
}

fn kb(linea: &str) -> u64 {
    linea.split_whitespace().nth(1).and_then(|x| x.parse().ok()).unwrap_or(0)
}

/// **INSTRUMENTO, no comprobacion** (RFC-0007 E4a, §461). Correr en release, a mano:
///
/// ```text
/// cargo test --release -p zk-ssl instrumento_de_la_puerta_de_edad -- --ignored --nocapture
/// ```
///
/// Por cada `n` objetivo: los hashes del rango de UN arbol (`hashes_de_rango`, profundidad 32),
/// los de los DOS (pendientes y meta), los niveles de la subida -la potencia de dos que los
/// cubre- y sus filas; y la subida probada y verificada con `crate::proof_options()`: tiempos,
/// bytes de la prueba y el pico de memoria del proceso (`VmHWM`, que es ACUMULADO: las tallas
/// van de menor a mayor). Arriba, la maquina: CPU, nucleos y `MemTotal`.
#[test]
#[ignore = "instrumento de medida, no comprobacion: correr a mano, en release"]
fn instrumento_de_la_puerta_de_edad() {
    use std::time::Instant;
    let cpu = de_proc("/proc/cpuinfo", "model name");
    let nucleos = std::thread::available_parallelism().map(|x| x.get()).unwrap_or(0);
    let total = kb(&de_proc("/proc/meminfo", "MemTotal"));
    println!("E4a| maquina: {cpu} . nucleos {nucleos} . MemTotal {total} kB");
    println!(
        "E4a| puerta (D-E4a-1, D-E4a-2): probar <= {LATIDO_S} s y pico <= MemTotal/2 = {} kB",
        total / 2
    );
    let opciones = crate::proof_options();
    for n in N_OBJETIVO {
        let uno = hashes_de_rango(n, 32);
        let dos = 2 * uno;
        let niveles = (dos as usize).next_power_of_two();
        let filas = niveles * CYCLE_LENGTH;
        let (hoja, camino) = camino_de(niveles);
        let t = Instant::now();
        let traza = traza_de_subida(hoja, &camino);
        let raiz = native_root(hoja, &camino);
        let traza_ms = t.elapsed().as_secs_f64() * 1000.0;
        let t = Instant::now();
        let prueba = SubidaProver { options: opciones.clone() }.prove(traza).expect("probar");
        let probar_s = t.elapsed().as_secs_f64();
        let bytes = prueba.to_bytes().len();
        let t = Instant::now();
        let ok = verifica(prueba, raiz, &opciones);
        let verificar_ms = t.elapsed().as_secs_f64() * 1000.0;
        let pico = kb(&de_proc("/proc/self/status", "VmHWM"));
        println!(
            "E4a| n {n} . hashes {uno} por arbol, {dos} los dos . subida {niveles} niveles = \
             {filas} filas x {TRACE_WIDTH} . traza {traza_ms:.0} ms . probar {probar_s:.2} s . \
             verificar {verificar_ms:.1} ms . prueba {bytes} B . verifica {ok} . pico {pico} kB . \
             cabe en un latido: {} . cabe en media RAM: {}",
            if probar_s <= LATIDO_S { "si" } else { "no" },
            if total > 0 && pico <= total / 2 { "si" } else { "no" }
        );
    }
}
