//! Los nueve falsadores de la ocultacion del nucleo (RFC-0009 D-K; E3a-2, S534; 8 y 9, D-AH), traidos del
//! spike que los midio (`~/spike-b-p4r3/spike/src/main.rs`, `145e83ecea366126`: un binario con
//! veredicto por rc, no tests) y con el modo oculto SOLO aqui: ningun probador de la casa devuelve
//! `Some` en `Prover::ocultacion`, y los dos de este modulo lo devuelven solo cuando el test se lo
//! da. Corren en depuracion y en release: el RFC (Seguridad) promete que los asertos de grados de
//! winterfell los ven.
//!
//! 1. `censo_cero`: apagada, una columna constante de la traza y otra del tramo auxiliar salen
//!    literales en la prueba y un control nunca (rc 80 y 81 del spike); oculta, ninguna (82, 83).
//! 2. `verifica`: la prueba oculta la acepta `verify`, y dice de si misma 2T, ancho + 1 y la
//!    marca con m (84).
//! 3. `rechaza_publico_mas_uno` (85).
//! 4. `marca_tocada`: otra version, otro prefijo o un largo que no es el de la marca se RECHAZAN
//!    con error, sin panico (89; el spike daba el panico por bueno, y la 172 midio que el fork
//!    apagado panicaba en `WorkAir::new` con la marca tocada).
//! 5. `dimensiones_invalidas`: con la marca, largo 8, ancho 1 y m = 2T se rechazan sin panico (97).
//! 6. `workair_con_la_subida`: WorkAir (grado 3 sin ciclos) no cabe con su ce de serie, 2; el
//!    envoltorio sube a 4, cabe, y la prueba hecha con esa subida verifica (94, 96).
//! 7. `regresion_apagada`: apagado por el API, byte a byte lo del probador de la casa (la foto de
//!    D-R); la misma ocultacion, los mismos bytes; otra semilla de filas u otra del cociente, otros
//!    bytes, y la del cociente deja las raices de la traza y mueve la de restricciones (86, 88 y
//!    la estructural del spike).
//! 8. `traza_no_satisfecha_da_err` (D-AH, 5.A-396): un testigo malo con la ocultacion encendida
//!    es un `Err` del probador con su paso, nunca un panico: la traza real se comprueba antes
//!    de que el cociente oculto aserte, tambien en release.
//! 9. `m_que_no_cabe_da_err` (D-AG, D-AH, 5.A-395): m >= 2T es un `Err` nombrado, no un panico;
//!    16 filas con m = 64 era el caso de los reembolsos antes del acolchado.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Mutex;

use winter_air::{Marca, MarcaError, Oculta};
use winter_prover::{Ocultacion, ProverError};
use winterfell::crypto::{hashers::Blake3_256, DefaultRandomCoin, Hasher, MerkleTree};
use winterfell::math::fields::f128::BaseElement as Base128;
use winterfell::math::fields::f64::BaseElement as Base;
use winterfell::math::{ExtensionOf, FieldElement, ToElements};
use winterfell::matrix::ColMatrix;
use winterfell::{
    verify, AcceptableOptions, Air, AirContext, Assertion, AuxRandElements, BatchingMethod,
    CompositionPoly, CompositionPolyTrace, ConstraintCompositionCoefficients,
    DefaultConstraintCommitment, DefaultConstraintEvaluator, DefaultTraceLde, EvaluationFrame,
    FieldExtension, PartitionOptions, Proof, ProofOptions, Prover, StarkDomain, Trace, TraceInfo,
    TracePolyTable, TraceTable, TransitionConstraintDegree,
};

use crate::{build_trace, WorkAir, WorkProver};

/// La ocultacion de los falsadores: m 64 (D-G) y las semillas del spike (F1 y Q1).
const OC: Ocultacion = Ocultacion {
    m: 64,
    semilla_filas: 0xf11a_0001,
    semilla_cociente: 0xc0c1_0001,
};

/// Las opciones de la casa para los circuitos y la foto: 32/8/0/None/8/31.
fn opciones() -> ProofOptions {
    ProofOptions::new(
        32,
        8,
        0,
        FieldExtension::None,
        8,
        31,
        BatchingMethod::Linear,
        BatchingMethod::Linear,
    )
}

/// Las del juguete del censo: las mismas, con la extension cuadratica del spike para el tramo
/// auxiliar.
fn opciones_censo() -> ProofOptions {
    ProofOptions::new(
        32,
        8,
        0,
        FieldExtension::Quadratic,
        8,
        31,
        BatchingMethod::Linear,
        BatchingMethod::Linear,
    )
}

/// Cuantas veces salen los ocho bytes little-endian de `v` en `bytes` (main.rs:698 del spike).
fn cuenta(bytes: &[u8], v: u64) -> usize {
    let p = v.to_le_bytes();
    bytes.windows(8).filter(|w| w[..] == p[..]).count()
}

// WORKAIR, EL CIRCUITO CANONICO, CON UN PROBADOR QUE EL TEST ENCIENDE
// ================================================================================================

type Blake3W = Blake3_256<Base128>;

struct ProbadorWork {
    opciones: ProofOptions,
    ocultacion: Option<Ocultacion>,
}

impl Prover for ProbadorWork {
    type BaseField = Base128;
    type Air = WorkAir;
    type Trace = TraceTable<Base128>;
    type HashFn = Blake3W;
    type VC = MerkleTree<Blake3W>;
    type RandomCoin = DefaultRandomCoin<Blake3W>;
    type TraceLde<E: FieldElement<BaseField = Self::BaseField>> =
        DefaultTraceLde<E, Self::HashFn, Self::VC>;
    type ConstraintEvaluator<'a, E: FieldElement<BaseField = Self::BaseField>> =
        DefaultConstraintEvaluator<'a, Self::Air, E>;
    type ConstraintCommitment<E: FieldElement<BaseField = Self::BaseField>> =
        DefaultConstraintCommitment<E, Self::HashFn, Self::VC>;

    fn get_pub_inputs(&self, trace: &Self::Trace) -> Base128 {
        trace.get(0, trace.length() - 1)
    }

    fn options(&self) -> &ProofOptions {
        &self.opciones
    }

    fn ocultacion(&self) -> Option<Ocultacion> {
        self.ocultacion
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

/// El resultado publico de WorkAir con start 3 y n filas.
fn resultado(n: usize) -> Base128 {
    let mut x = Base128::new(3);
    for _ in 0..(n - 1) {
        x = x * x * x + Base128::new(42);
    }
    x
}

/// Los bytes de una prueba de WorkAir (start 3, n filas) con la ocultacion que se le de.
fn prueba_work(n: usize, ocultacion: Option<Ocultacion>) -> Vec<u8> {
    let probador = ProbadorWork { opciones: opciones(), ocultacion };
    probador.prove(build_trace(Base128::new(3), n)).expect("probar WorkAir").to_bytes()
}

/// Verifica unos bytes como prueba de WorkAir contra el resultado publico que se le de.
fn verificar_work(bytes: &[u8], publico: Base128) -> Result<(), String> {
    let proof = Proof::from_bytes(bytes).map_err(|e| format!("leer: {e:?}"))?;
    let acc = AcceptableOptions::OptionSet(vec![opciones()]);
    verify::<WorkAir, Blake3W, DefaultRandomCoin<Blake3W>, MerkleTree<Blake3W>>(
        proof, publico, &acc,
    )
    .map_err(|e| format!("{e:?}"))
}

/// El listado de los falsadores 4 y 5: el verificador tiene que RECHAZAR, con error; aceptar es
/// rojo, y entrar en panico es otro rojo con su nombre (el kit tiene que fallar cerrado).
fn rechaza_sin_panico(bytes: &[u8], publico: Base128, que: &str) {
    match catch_unwind(AssertUnwindSafe(|| verificar_work(bytes, publico))) {
        Ok(Err(_)) => {},
        Ok(Ok(())) => panic!("{que}: el verificador ACEPTA"),
        Err(_) => panic!("{que}: el verificador entra en PANICO en vez de rechazar"),
    }
}

/// Las raices de la traza y la de restricciones de una prueba, leidas de sus compromisos.
fn raices(bytes: &[u8]) -> (Vec<<Blake3W as Hasher>::Digest>, <Blake3W as Hasher>::Digest) {
    let proof = Proof::from_bytes(bytes).expect("leer");
    let lde = proof.trace_info().length() * proof.options().blowup_factor();
    let capas = proof.options().to_fri_options().num_fri_layers(lde);
    let (traza, restricciones, _fri) =
        proof.commitments.parse::<Blake3W>(1, capas).expect("leer las raices");
    (traza, restricciones)
}

#[test]
fn verifica() {
    let bytes = prueba_work(256, Some(OC));
    assert_eq!(verificar_work(&bytes, resultado(256)), Ok(()));
    // lo que la prueba dice de si misma: 2T filas, una columna mas y la marca con m
    let proof = Proof::from_bytes(&bytes).expect("leer");
    assert_eq!(proof.trace_info().length(), 512);
    assert_eq!(proof.trace_info().main_trace_width(), 2);
    assert_eq!(Marca::leer(proof.trace_info().meta()), Ok(Some(Marca { m: 64 })));
}

#[test]
fn rechaza_publico_mas_uno() {
    let bytes = prueba_work(256, Some(OC));
    assert!(verificar_work(&bytes, resultado(256) + Base128::ONE).is_err());
}

#[test]
fn marca_tocada() {
    let bytes = prueba_work(256, Some(OC));
    // la marca empieza en el byte 6: ancho, aux, rands, log2T y el largo del meta (u16) delante
    assert_eq!(&bytes[6..6 + 15], winter_air::marca::PREFIJO);
    let mut version = bytes.clone();
    version[6 + 14] = b'2'; // arqueo:oculta:2
    rechaza_sin_panico(&version, resultado(256), "otra version de la marca");
    let mut ajena = bytes.clone();
    ajena[6] ^= 0x01; // ya no empieza por el prefijo
    rechaza_sin_panico(&ajena, resultado(256), "un meta que no es la marca");
    // y el lector solo, sin pasar por winterfell
    assert_eq!(Marca::leer(b""), Ok(None));
    assert_eq!(Marca::leer(&Marca { m: 64 }.escribir()), Ok(Some(Marca { m: 64 })));
    assert_eq!(Marca::leer(b"arqueo:oculta:2\x40\x00\x00\x00"), Err(MarcaError::Desconocida));
    assert_eq!(Marca::leer(b"arqueo:oculta:1"), Err(MarcaError::Largo));
    assert_eq!(Marca::leer(b"arqueo:oculta:1\x40\x00\x00\x00\x00"), Err(MarcaError::Largo));
}

#[test]
fn dimensiones_invalidas() {
    // n 8 con m 4: 2T = 16 y ancho 2, justo en la frontera; pasa
    let oc = Ocultacion { m: 4, ..OC };
    let bytes = prueba_work(8, Some(oc));
    assert_eq!(verificar_work(&bytes, resultado(8)), Ok(()));
    let mut corta = bytes.clone();
    corta[3] = 3; // log2T 4 -> 3: la prueba dice largo 8
    rechaza_sin_panico(&corta, resultado(8), "largo 8 con la marca");
    let mut estrecha = bytes.clone();
    estrecha[0] = 1; // ancho 2 -> 1
    rechaza_sin_panico(&estrecha, resultado(8), "ancho 1 con la marca");
    let mut grande = bytes.clone();
    grande[6 + 15..6 + 19].copy_from_slice(&16u32.to_le_bytes()); // m = 2T
    rechaza_sin_panico(&grande, resultado(8), "m igual a la longitud de la traza oculta");
}

#[test]
fn workair_con_la_subida() {
    let n = 256;
    let publico = resultado(n);
    let plano = WorkAir::new(TraceInfo::new(1, n), publico, opciones());
    assert_eq!(plano.context().ce_domain_size() / n, 2, "el ce de serie de WorkAir es 2");
    let info = TraceInfo::with_meta(2, 2 * n, Marca { m: 64 }.escribir());
    let oculto = Oculta::<WorkAir>::new(info, publico, opciones());
    assert!(!oculto.cabe_con(2), "con el ce de serie no cabe");
    assert!(oculto.cabe_con(4), "con 4 cabe");
    assert!(oculto.cabe());
    assert_eq!(oculto.context().ce_domain_size() / (2 * n), 4, "el envoltorio subio el ce a 4");
    assert_eq!(oculto.context().num_transition_exemptions(), n + 1);
    // y la prueba hecha con esa subida verifica
    assert_eq!(verificar_work(&prueba_work(n, Some(OC)), publico), Ok(()));
}

#[test]
fn regresion_apagada() {
    // apagada por el API, la prueba es la del probador de la casa, byte a byte (la foto de D-R)
    let casa = WorkProver::new(opciones())
        .prove(build_trace(Base128::new(3), 8))
        .expect("probar con el probador de la casa")
        .to_bytes();
    assert_eq!(prueba_work(8, None), casa);
    // la misma ocultacion, los mismos bytes
    let a = prueba_work(256, Some(OC));
    assert_eq!(a, prueba_work(256, Some(OC)));
    // otra semilla de filas: otros bytes, y otra raiz de la traza
    let b = prueba_work(256, Some(Ocultacion { semilla_filas: 0xf11a_0002, ..OC }));
    assert_ne!(a, b);
    // otra semilla del cociente: otros bytes, las raices de la traza quedan y la de restricciones
    // se mueve
    let c = prueba_work(256, Some(Ocultacion { semilla_cociente: 0xc0c1_0002, ..OC }));
    assert_ne!(a, c);
    let (traza_a, restricciones_a) = raices(&a);
    let (traza_b, _) = raices(&b);
    let (traza_c, restricciones_c) = raices(&c);
    assert_ne!(traza_a, traza_b);
    assert_eq!(traza_a, traza_c);
    assert_ne!(restricciones_a, restricciones_c);
}

#[test]
fn traza_no_satisfecha_da_err() {
    // una celda falseada en medio: la transicion 99 -> 100 deja de cumplirse
    let mut traza = build_trace(Base128::new(3), 256);
    let fila = 100;
    let v = traza.get(0, fila);
    traza.set(0, fila, v + Base128::ONE);
    let probador = ProbadorWork { opciones: opciones(), ocultacion: Some(OC) };
    match catch_unwind(AssertUnwindSafe(|| probador.prove(traza))) {
        Ok(Err(ProverError::UnsatisfiedTransitionConstraintError(paso))) => {
            assert_eq!(paso, fila - 1, "el paso que no se cumple");
        },
        Ok(Err(e)) => panic!("otro error: {e:?}"),
        Ok(Ok(_)) => panic!("probo una traza no satisfecha"),
        Err(_) => panic!("el probador entra en PANICO en vez de devolver Err (5.A-396)"),
    }
}

#[test]
fn m_que_no_cabe_da_err() {
    // 16 filas y m = 64: 2T = 32 < m, la traza corta de los reembolsos antes de D-AG
    let probador = ProbadorWork { opciones: opciones(), ocultacion: Some(OC) };
    match catch_unwind(AssertUnwindSafe(|| probador.prove(build_trace(Base128::new(3), 16)))) {
        Ok(Err(ProverError::OcultacionNoCabe { m: 64, filas: 32 })) => {},
        Ok(Err(e)) => panic!("otro error: {e:?}"),
        Ok(Ok(_)) => panic!("probo con m = 64 en una traza oculta de 32 filas"),
        Err(_) => panic!("el probador entra en PANICO en vez de devolver Err (5.A-395)"),
    }
}

// EL JUGUETE DEL CENSO: UNA COLUMNA CONSTANTE Y OTRA EN EL TRAMO AUXILIAR
// ================================================================================================
// El `AirJuguete` del spike, recortado a lo que mide: la clave en una columna constante (lo que
// sale literal, 5.A-360), un contador, una columna de grado 7 para que el cociente tenga trozos,
// y un tramo auxiliar con clave + alfa, constante, y alfa. 128 filas y 3 + 2 columnas en vez de
// las 512 y 42 + 2 del spike, que median el coste; el censo es el mismo.

type Blake3J = Blake3_256<Base>;
const CLAVE: u64 = 0x7b24_ad75_3939_240c;
const CONTROL: u64 = 0x09b6_88e0_7e73_d8ee;
const FILAS: usize = 128;
const ANCHO: usize = 3;
const GRADO: usize = 7;

#[derive(Clone)]
struct Publico {
    ultimo: Base,
}

impl ToElements<Base> for Publico {
    fn to_elements(&self) -> Vec<Base> {
        vec![self.ultimo]
    }
}

struct AirConstante {
    ctx: AirContext<Base>,
    ultimo: Base,
}

impl Air for AirConstante {
    type BaseField = Base;
    type PublicInputs = Publico;

    fn new(info: TraceInfo, p: Publico, opciones: ProofOptions) -> Self {
        let grados = vec![
            TransitionConstraintDegree::new(1),
            TransitionConstraintDegree::new(1),
            TransitionConstraintDegree::new(GRADO),
        ];
        let aux = vec![TransitionConstraintDegree::new(1), TransitionConstraintDegree::new(1)];
        let ctx = AirContext::new_multi_segment(info, grados, aux, 2, 1, opciones);
        Self { ctx, ultimo: p.ultimo }
    }

    fn context(&self) -> &AirContext<Self::BaseField> {
        &self.ctx
    }

    fn evaluate_transition<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        frame: &EvaluationFrame<E>,
        _periodic_values: &[E],
        result: &mut [E],
    ) {
        let a = frame.current();
        let s = frame.next();
        result[0] = s[0] - a[0]; // la clave, constante
        result[1] = s[1] - a[1] - E::ONE; // el contador
        let y = a[2];
        let y2 = y * y;
        let y4 = y2 * y2;
        result[2] = s[2] - (y4 * y2 * y + a[1]); // grado 7: el cociente tiene trozos
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        vec![
            Assertion::single(1, 0, Base::ZERO),
            Assertion::single(2, self.trace_length() - 1, self.ultimo),
        ]
    }

    fn evaluate_aux_transition<F, E>(
        &self,
        main_frame: &EvaluationFrame<F>,
        aux_frame: &EvaluationFrame<E>,
        _periodic_values: &[F],
        aux_rand_elements: &AuxRandElements<E>,
        result: &mut [E],
    ) where
        F: FieldElement<BaseField = Self::BaseField>,
        E: FieldElement<BaseField = Self::BaseField> + ExtensionOf<F>,
    {
        let alfa = aux_rand_elements.rand_elements()[0];
        let m = main_frame.current();
        let a = aux_frame.current();
        let s = aux_frame.next();
        result[0] = a[0] - E::from(m[0]) - alfa; // clave + alfa, constante
        result[1] = s[1] - a[1]; // alfa, constante
    }

    fn get_aux_assertions<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        aux_rand_elements: &AuxRandElements<E>,
    ) -> Vec<Assertion<E>> {
        vec![Assertion::single(1, 0, aux_rand_elements.rand_elements()[0])]
    }
}

struct TrazaConstante {
    info: TraceInfo,
    principal: ColMatrix<Base>,
    publico: Publico,
}

impl Trace for TrazaConstante {
    type BaseField = Base;

    fn info(&self) -> &TraceInfo {
        &self.info
    }

    fn main_segment(&self) -> &ColMatrix<Base> {
        &self.principal
    }

    fn read_main_frame(&self, row_idx: usize, frame: &mut EvaluationFrame<Base>) {
        let siguiente = (row_idx + 1) % self.info.length();
        self.principal.read_row_into(row_idx, frame.current_mut());
        self.principal.read_row_into(siguiente, frame.next_mut());
    }
}

fn traza_constante() -> TrazaConstante {
    let clave = Base::new(CLAVE);
    let mut c: Vec<Vec<Base>> = vec![vec![Base::ZERO; FILAS]; ANCHO];
    c[2][0] = clave + Base::new(2);
    for i in 0..FILAS {
        c[0][i] = clave;
    }
    for i in 1..FILAS {
        c[1][i] = c[1][i - 1] + Base::ONE;
        let y = c[2][i - 1];
        let y2 = y * y;
        let y4 = y2 * y2;
        c[2][i] = y4 * y2 * y + c[1][i - 1];
    }
    let publico = Publico { ultimo: c[2][FILAS - 1] };
    let info = TraceInfo::new_multi_segment(ANCHO, 2, 1, FILAS, vec![]);
    TrazaConstante { info, principal: ColMatrix::new(c), publico }
}

struct ProbadorConstante {
    opciones: ProofOptions,
    ocultacion: Option<Ocultacion>,
    /// clave + alfa del tramo auxiliar, como entero, para que el censo lo busque: alfa lo da la
    /// moneda al probar, y solo `build_aux_trace` lo ve
    aux_esperado: Mutex<u64>,
}

impl Prover for ProbadorConstante {
    type BaseField = Base;
    type Air = AirConstante;
    type Trace = TrazaConstante;
    type HashFn = Blake3J;
    type VC = MerkleTree<Blake3J>;
    type RandomCoin = DefaultRandomCoin<Blake3J>;
    type TraceLde<E: FieldElement<BaseField = Self::BaseField>> =
        DefaultTraceLde<E, Self::HashFn, Self::VC>;
    type ConstraintEvaluator<'a, E: FieldElement<BaseField = Self::BaseField>> =
        DefaultConstraintEvaluator<'a, Self::Air, E>;
    type ConstraintCommitment<E: FieldElement<BaseField = Self::BaseField>> =
        DefaultConstraintCommitment<E, Self::HashFn, Self::VC>;

    fn get_pub_inputs(&self, trace: &Self::Trace) -> Publico {
        trace.publico.clone()
    }

    fn options(&self) -> &ProofOptions {
        &self.opciones
    }

    fn ocultacion(&self) -> Option<Ocultacion> {
        self.ocultacion
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

    fn build_aux_trace<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        main_trace: &Self::Trace,
        aux_rand_elements: &AuxRandElements<E>,
    ) -> ColMatrix<E> {
        let t = main_trace.info.length();
        let alfa = aux_rand_elements.rand_elements()[0];
        let valor = E::from(main_trace.principal.get(0, 0)) + alfa;
        let base = E::slice_as_base_elements(core::slice::from_ref(&valor));
        *self.aux_esperado.lock().expect("aux_esperado") = base[0].as_int();
        ColMatrix::new(vec![vec![valor; t], vec![alfa; t]])
    }
}

/// Los bytes de una prueba del juguete, con clave + alfa del tramo auxiliar y lo publico.
fn prueba_constante(ocultacion: Option<Ocultacion>) -> (Vec<u8>, u64, Publico) {
    let probador = ProbadorConstante {
        opciones: opciones_censo(),
        ocultacion,
        aux_esperado: Mutex::new(0),
    };
    let traza = traza_constante();
    let publico = traza.publico.clone();
    let bytes = probador.prove(traza).expect("probar el juguete").to_bytes();
    let aux = *probador.aux_esperado.lock().expect("aux_esperado");
    (bytes, aux, publico)
}

fn verificar_constante(bytes: &[u8], publico: Publico) -> Result<(), String> {
    let proof = Proof::from_bytes(bytes).map_err(|e| format!("leer: {e:?}"))?;
    let acc = AcceptableOptions::OptionSet(vec![opciones_censo()]);
    verify::<AirConstante, Blake3J, DefaultRandomCoin<Blake3J>, MerkleTree<Blake3J>>(
        proof, publico, &acc,
    )
    .map_err(|e| format!("{e:?}"))
}

#[test]
fn censo_cero() {
    // apagado: la clave de la columna constante y clave + alfa del tramo auxiliar salen literales,
    // y el control nunca: el censo ve, y no ve fantasmas
    let (a, aux_a, publico_a) = prueba_constante(None);
    assert_eq!(verificar_constante(&a, publico_a), Ok(()));
    assert!(cuenta(&a, CLAVE) > 0, "apagado, el censo no ve la clave: el falsador esta ciego");
    assert!(cuenta(&a, aux_a) > 0, "apagado, el censo no ve clave + alfa del tramo auxiliar");
    assert_eq!(cuenta(&a, CONTROL), 0, "el control aparece: el censo da falsos positivos");
    // oculto: ninguna de las dos, y la prueba verifica y rechaza publico + 1
    let (o, aux_o, publico_o) = prueba_constante(Some(OC));
    assert_eq!(verificar_constante(&o, publico_o.clone()), Ok(()));
    assert_eq!(cuenta(&o, CLAVE), 0, "una prueba oculta ensena la clave");
    assert_eq!(cuenta(&o, aux_o), 0, "una prueba oculta ensena clave + alfa del tramo auxiliar");
    assert_eq!(cuenta(&o, CONTROL), 0);
    let falso = Publico { ultimo: publico_o.ultimo + Base::ONE };
    assert!(verificar_constante(&o, falso).is_err());
}
