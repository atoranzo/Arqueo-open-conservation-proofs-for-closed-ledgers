//! §713: el resto FRI, ATADO a su compromiso. Los falsadores del arreglo del canal del fork
//! (`crates/winter-verifier/src/channel.rs`, `read_remainder`).
//!
//! El probador se compromete con el resto: el ultimo compromiso FRI es `hash_elements(resto)`,
//! con los coeficientes en el orden en que viajan (`winter-fri` 0.13.1, `set_remainder`). Pero su
//! verificador no lo comprueba: ese compromiso solo siembra la moneda. Una prueba cuyo ultimo
//! compromiso es el hash de OTRO resto verificaba.
//!
//! ⚠️ Aqui se fabrica esa prueba SIN tocar el probador. [`Atajo`] es Blake3 byte a byte salvo
//! en UNA llamada a `hash_elements`, la del compromiso del resto, donde devuelve el hash de otro
//! resto: el mismo con su primer coeficiente mas uno. Lo demas es una corrida coherente con ese
//! compromiso ajeno -la moneda, las consultas, las aperturas y el resto de verdad, que viaja-, y
//! los bytes son los de una prueba de Blake3: la juzga el verificador de siempre, con `Blake3_256`.
//!
//! Cual es la llamada del resto se MIDE, no se supone. Una primera corrida honesta cuenta las
//! llamadas a `hash_elements` de su hilo y comprueba que la ultima es el ultimo compromiso FRI y
//! el hash del resto que viaja. La segunda cambia esa misma. Sin `concurrent` el probador es de un
//! hilo, y la cuenta tiene que salir igual en las dos corridas: se comprueba.
//!
//! Dos montajes:
//! 1. **WorkAir**, con `MerkleTree` y sin ocultacion: la demo del fallo, en un test. Corre
//!    en depuracion y en release.
//! 2. **La edad de la casa**, oculta y con sal, juzgada por `zk_ssl_air::verificar`, el juez del
//!    kit. Solo en release, como sus hermanos de `circuit_edad.rs`.

use std::cell::Cell;
use std::marker::PhantomData;

use winterfell::crypto::{
    hashers::Blake3_256, DefaultRandomCoin, Digest as _, ElementHasher, Hasher, MerkleTree,
};
use winterfell::math::fields::f128::BaseElement as Base128;
use winterfell::math::fields::f64::BaseElement as Base64;
use winterfell::math::fields::QuadExtension;
use winterfell::math::{FieldElement, StarkField};
use winterfell::matrix::ColMatrix;
use winterfell::{
    verify, AcceptableOptions, AuxRandElements, BatchingMethod, CompositionPoly,
    CompositionPolyTrace, ConstraintCompositionCoefficients, DefaultConstraintCommitment,
    DefaultConstraintEvaluator, DefaultTraceLde, FieldExtension, PartitionOptions, Proof,
    ProofOptions, Prover, StarkDomain, Trace, TraceInfo, TracePolyTable, TraceTable,
};

use crate::circuit_edad::{
    construir, construir_auxiliar, opciones as opciones_casa, verificar as verificar_edad, Digest,
    EdadAir, EdadPublicInputs, Enunciado, TrazaEdad,
};
use crate::{build_trace, MerkleConSal, WorkAir};

/// Lo que el kit imprime cuando el resto no es el comprometido: la variante que `winter-fri`
/// 0.13.1 define y no usa.
const CAUSA: &str = "FriVerificationFailed(RemainderCommitmentMismatch)";

// EL HASHER QUE CAMBIA UNA LLAMADA
// ================================================================================================

thread_local! {
    /// Las llamadas a `hash_elements` de este hilo desde el ultimo `rearmar`.
    static LLAMADAS: Cell<usize> = const { Cell::new(0) };
    /// La llamada que devuelve el hash de OTRO resto, si hay una.
    static CAMBIAR: Cell<Option<usize>> = const { Cell::new(None) };
    /// El digest de la ultima llamada.
    static ULTIMO: Cell<Option<[u8; 32]>> = const { Cell::new(None) };
}

fn rearmar(cambiar: Option<usize>) {
    LLAMADAS.set(0);
    CAMBIAR.set(cambiar);
    ULTIMO.set(None);
}

/// `Blake3_256<B>` byte a byte, salvo la llamada `CAMBIAR` a `hash_elements`: esa devuelve el hash
/// de los mismos elementos con el primero mas uno. Cada test corre en su hilo, y la cuenta es de
/// su hilo.
#[derive(Debug, PartialEq, Eq)]
struct Atajo<B: StarkField>(PhantomData<B>);

impl<B: StarkField> Hasher for Atajo<B> {
    type Digest = <Blake3_256<B> as Hasher>::Digest;

    const COLLISION_RESISTANCE: u32 = <Blake3_256<B> as Hasher>::COLLISION_RESISTANCE;

    fn hash(bytes: &[u8]) -> Self::Digest {
        Blake3_256::<B>::hash(bytes)
    }

    fn merge(values: &[Self::Digest; 2]) -> Self::Digest {
        Blake3_256::<B>::merge(values)
    }

    fn merge_many(values: &[Self::Digest]) -> Self::Digest {
        Blake3_256::<B>::merge_many(values)
    }

    fn merge_with_int(seed: Self::Digest, value: u64) -> Self::Digest {
        Blake3_256::<B>::merge_with_int(seed, value)
    }
}

impl<B: StarkField> ElementHasher for Atajo<B> {
    type BaseField = B;

    fn hash_elements<E: FieldElement<BaseField = B>>(elements: &[E]) -> Self::Digest {
        let n = LLAMADAS.get();
        LLAMADAS.set(n + 1);
        let digest = if CAMBIAR.get() == Some(n) {
            let mut otro = elements.to_vec();
            otro[0] += E::ONE;
            Blake3_256::<B>::hash_elements(&otro)
        } else {
            Blake3_256::<B>::hash_elements(elements)
        };
        ULTIMO.set(Some(digest.as_bytes()));
        digest
    }
}

// LAS DOS CORRIDAS
// ================================================================================================

/// De unos bytes de prueba, con Blake3: el ultimo compromiso FRI, el hash del resto que viaja y
/// el del OTRO resto (el mismo con su primer coeficiente mas uno).
fn lo_del_resto<B, E>(bytes: &[u8]) -> ([u8; 32], [u8; 32], [u8; 32])
where
    B: StarkField,
    E: FieldElement<BaseField = B>,
{
    let proof = Proof::from_bytes(bytes).expect("leer la prueba");
    let capas = proof.options().to_fri_options().num_fri_layers(proof.lde_domain_size());
    let segmentos = proof.trace_info().num_segments();
    let (_, _, fri) = proof
        .commitments
        .clone()
        .parse::<Blake3_256<B>>(segmentos, capas)
        .expect("los compromisos");
    let resto: Vec<E> = proof.fri_proof.parse_remainder().expect("el resto");
    let mut otro = resto.clone();
    otro[0] += E::ONE;
    (
        fri.last().expect("el compromiso del resto").as_bytes(),
        Blake3_256::<B>::hash_elements(&resto).as_bytes(),
        Blake3_256::<B>::hash_elements(&otro).as_bytes(),
    )
}

/// Prueba dos veces con [`Atajo`]: la primera, honesta, mide que llamada es la del resto; la
/// segunda cambia esa. Devuelve (honesta, ajena) y comprueba en el camino lo que se afirma de las
/// dos: la honesta se compromete con su resto, y la ajena con OTRO.
fn dos_corridas<B, E>(probar: impl Fn() -> Vec<u8>) -> (Vec<u8>, Vec<u8>)
where
    B: StarkField,
    E: FieldElement<BaseField = B>,
{
    rearmar(None);
    let honesta = probar();
    let llamadas = LLAMADAS.get();
    let ultimo = ULTIMO.get().expect("el probador no llamo a hash_elements");
    let (compromiso, del_resto, _) = lo_del_resto::<B, E>(&honesta);
    assert_eq!(compromiso, del_resto, "honesta: el ultimo compromiso FRI no es el hash del resto");
    assert_eq!(ultimo, compromiso, "la ultima llamada a hash_elements no es la del resto");

    rearmar(Some(llamadas - 1));
    let ajena = probar();
    assert_eq!(LLAMADAS.get(), llamadas, "la segunda corrida no hizo las mismas llamadas");
    let (compromiso, del_resto, del_otro) = lo_del_resto::<B, E>(&ajena);
    assert_ne!(compromiso, del_resto, "ajena: el compromiso sigue siendo el del resto que viaja");
    assert_eq!(compromiso, del_otro, "ajena: el compromiso no es el del OTRO resto");
    rearmar(None);
    (honesta, ajena)
}

// 1. WORKAIR: LA DEMO
// ================================================================================================

/// Las opciones de la foto y de los falsadores de la ocultacion: 32/8/0/None/8/31.
fn opciones_work() -> ProofOptions {
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

struct ProbadorWork {
    opciones: ProofOptions,
}

impl Prover for ProbadorWork {
    type BaseField = Base128;
    type Air = WorkAir;
    type Trace = TraceTable<Base128>;
    type HashFn = Atajo<Base128>;
    type VC = MerkleTree<Atajo<Base128>>;
    type RandomCoin = DefaultRandomCoin<Atajo<Base128>>;
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

/// Filas de la traza de WorkAir: 8192 en el dominio extendido, dos capas FRI y 16 coeficientes de
/// resto con las opciones de arriba.
const FILAS_WORK: usize = 1024;

/// El juez de siempre, con `Blake3_256`: el que ve los bytes que salen de [`Atajo`].
fn verificar_work(bytes: &[u8], publico: Base128) -> Result<(), String> {
    let proof = Proof::from_bytes(bytes).map_err(|e| format!("leer: {e:?}"))?;
    let acc = AcceptableOptions::OptionSet(vec![opciones_work()]);
    type B3 = Blake3_256<Base128>;
    verify::<WorkAir, B3, DefaultRandomCoin<B3>, MerkleTree<B3>>(proof, publico, &acc)
        .map_err(|e| format!("{e:?}"))
}

/// **La demo, en un test.** La prueba honesta verifica; la que se compromete con OTRO resto y
/// lleva el de verdad cae, y por el resto.
#[test]
fn work_un_resto_que_no_es_el_comprometido_no_verifica() {
    let probador = ProbadorWork { opciones: opciones_work() };
    let traza = || build_trace(Base128::new(3), FILAS_WORK);
    let publico = probador.get_pub_inputs(&traza());
    let (honesta, ajena) = dos_corridas::<Base128, Base128>(|| {
        probador.prove(traza()).expect("probar WorkAir").to_bytes()
    });
    assert_eq!(verificar_work(&honesta, publico), Ok(()), "la honesta no verifico");
    assert_eq!(verificar_work(&ajena, publico), Err(CAUSA.to_string()));
}

// 2. LA EDAD DE LA CASA: OCULTA, CON SAL, Y SU JUEZ
// ================================================================================================

/// El probador de `circuit_edad.rs` con [`Atajo`] en vez de Blake3: la misma traza, la misma
/// auxiliar, la misma ocultacion encendida y la misma sal.
struct ProbadorEdad {
    opciones: ProofOptions,
}

impl Prover for ProbadorEdad {
    type BaseField = Base64;
    type Air = EdadAir;
    type Trace = TrazaEdad;
    type HashFn = Atajo<Base64>;
    type VC = MerkleConSal<Atajo<Base64>>;
    type RandomCoin = DefaultRandomCoin<Atajo<Base64>>;
    type TraceLde<E: FieldElement<BaseField = Self::BaseField>> =
        DefaultTraceLde<E, Self::HashFn, Self::VC>;
    type ConstraintEvaluator<'a, E: FieldElement<BaseField = Self::BaseField>> =
        DefaultConstraintEvaluator<'a, Self::Air, E>;
    type ConstraintCommitment<E: FieldElement<BaseField = Self::BaseField>> =
        DefaultConstraintCommitment<E, Self::HashFn, Self::VC>;

    fn get_pub_inputs(&self, trace: &Self::Trace) -> EdadPublicInputs {
        trace.enunciado().clone()
    }

    fn options(&self) -> &ProofOptions {
        &self.opciones
    }

    fn ocultacion(&self) -> Option<winter_prover::Ocultacion> {
        Some(crate::ocultacion_encendida())
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
        construir_auxiliar(main_trace.main_segment(), aux_rand_elements.rand_elements())
    }
}

/// Un digest distinto de cero por `k`: la hoja de un pendiente vivo (la de `circuit_edad.rs`).
fn hoja(k: u64) -> Digest {
    [Base64::new(k + 1), Base64::new(3 * k + 2), Base64::new(7 * k + 3), Base64::new(11 * k + 4)]
}

/// Siete posiciones, huecos en la 0 y la 4, emisor `p % 3`, nacido `10 p`; con `seq = 130` las
/// cinco vivas tienen 120, 110, 100, 80 y 70 de edad, y con `T = 100` hay tres viejas.
fn traza_edad() -> TrazaEdad {
    let (mut hojas, mut meta) = (Vec::new(), Vec::new());
    for p in 0..7u64 {
        if p % 4 == 0 {
            hojas.push([Base64::ZERO; 4]);
            meta.push(None);
        } else {
            hojas.push(hoja(p));
            meta.push(Some((p % 3, 10 * p)));
        }
    }
    let en = Enunciado { seq: 130, t: 100, emisor: 0, todos: true };
    construir(&hojas, &meta, &en).expect("construir la traza de edad")
}

/// **La prueba de edad de la casa**, con la ocultacion encendida y la sal, y juzgada por
/// `zk_ssl_air::verificar`, el juez del kit: la honesta verifica, la que se compromete con OTRO
/// resto cae con la causa del resto. Es lo que el vector `edad/neg-prueba-resto-ajeno.json` fija
/// para el kit y para la segunda implementacion.
#[test]
#[cfg_attr(debug_assertions, ignore = "winterfell valida grados en depuracion: juez release")]
fn edad_un_resto_que_no_es_el_comprometido_no_verifica() {
    let probador = ProbadorEdad { opciones: opciones_casa() };
    let pi: EdadPublicInputs = traza_edad().enunciado().clone();
    assert_eq!(pi.k, 3, "la cuenta de la traza no es la del libro");
    let (honesta, ajena) = dos_corridas::<Base64, QuadExtension<Base64>>(|| {
        probador.prove(traza_edad()).expect("probar edad").to_bytes()
    });
    assert_eq!(verificar_edad(&honesta, &pi), Ok(()), "la honesta no verifico");
    assert_eq!(verificar_edad(&ajena, &pi), Err(CAUSA.to_string()));
}
