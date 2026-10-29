//! §911 (la 137 y la 138 de `BACKLOG.md`; RFC-0009 D-AH): el falsador 13 de la ocultacion del
//! nucleo, un testigo valido nunca hace panico al probador. Vive fuera de `falsadores_oculta.rs`
//! porque el barrido de `tools/check_constraint_layout.py` lee un circuito por fichero de `src/`:
//! dos AIR en un fichero le parecen una colision de ranuras. La AIR degenerada de la 137 esta en
//! `falsadores_oculta_degenerada.rs`, por lo mismo.

use std::panic::{catch_unwind, AssertUnwindSafe};

use winter_prover::{Ocultacion, ProverError};
use winterfell::crypto::{DefaultRandomCoin, MerkleTree};
use winterfell::math::fields::f128::BaseElement as Base128;
use winterfell::math::FieldElement;
use winterfell::matrix::ColMatrix;
use winterfell::{
    verify, AcceptableOptions, Air, AirContext, Assertion, AuxRandElements, CompositionPoly,
    CompositionPolyTrace, ConstraintCompositionCoefficients, DefaultConstraintCommitment,
    DefaultConstraintEvaluator, DefaultTraceLde, EvaluationFrame, PartitionOptions, ProofOptions,
    Prover, StarkDomain, TraceInfo, TracePolyTable, TraceTable, TransitionConstraintDegree,
};

use crate::falsadores_oculta::{opciones, Blake3W, OC};
use crate::falsadores_oculta_degenerada::ProbadorDegenerada;

/// §911 (la 138): una AIR de dos columnas, un contador y una constante 7, con una asercion simple
/// en el paso 0 del contador y una PERIODICA cada cuatro pasos sobre la constante; en claro es
/// una AIR como otra cualquiera. (Una sola columna constante dejaba la composicion identicamente
/// nula y el DEEP sin grado: es el `GradoDeDeep` de la 137, medido de paso.)
struct PeriodicaAir {
    context: AirContext<Base128>,
}

impl Air for PeriodicaAir {
    type BaseField = Base128;
    type PublicInputs = Base128;

    fn new(trace_info: TraceInfo, _pub_inputs: Base128, options: ProofOptions) -> Self {
        assert_eq!(2, trace_info.width());
        PeriodicaAir {
            context: AirContext::new(
                trace_info,
                vec![TransitionConstraintDegree::new(1), TransitionConstraintDegree::new(1)],
                2,
                options,
            ),
        }
    }

    fn context(&self) -> &AirContext<Self::BaseField> {
        &self.context
    }

    fn evaluate_transition<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        frame: &EvaluationFrame<E>,
        _periodic_values: &[E],
        result: &mut [E],
    ) {
        result[0] = frame.next()[0] - frame.current()[0] - E::ONE;
        result[1] = frame.next()[1] - frame.current()[1];
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        vec![Assertion::single(0, 0, Base128::ZERO), Assertion::periodic(1, 0, 4, Base128::new(7))]
    }
}

struct ProbadorPeriodica {
    ocultacion: Option<Ocultacion>,
}

impl Prover for ProbadorPeriodica {
    type BaseField = Base128;
    type Air = PeriodicaAir;
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

    fn get_pub_inputs(&self, _trace: &Self::Trace) -> Base128 {
        Base128::new(7)
    }

    fn options(&self) -> &ProofOptions {
        // la ProofOptions del juguete, con el mismo valor en cada llamada
        OPCIONES_PERIODICA.get_or_init(opciones)
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
        DefaultConstraintCommitment::new(composition_poly_trace, num_constraint_composition_columns, domain, partition_options)
    }
}

static OPCIONES_PERIODICA: std::sync::OnceLock<ProofOptions> = std::sync::OnceLock::new();

/// 13. §911 (la 137 y la 138; RFC-0009 D-AH): un testigo valido nunca hace panico al probador
/// oculto. La AIR con una asercion periodica prueba y verifica en claro, y oculta es
/// `AsercionPeriodicaBajoOculta` con su columna, su primer paso y su paso, sin panico y antes de
/// tocar la traza; y el grado del DEEP que no es L − 2 tiene un error con nombre, `GradoDeDeep`,
/// que ningun testigo valido alcanza mas que con probabilidad 2^-128: se mide su texto, no su
/// suceso.
#[test]
fn un_testigo_valido_nunca_hace_panico() {
    let traza = TraceTable::init(vec![(0..64u128).map(Base128::new).collect(), vec![Base128::new(7); 64]]);
    let claro = ProbadorPeriodica { ocultacion: None }.prove(traza.clone()).expect("en claro prueba");
    let acc = AcceptableOptions::OptionSet(vec![opciones()]);
    verify::<PeriodicaAir, Blake3W, DefaultRandomCoin<Blake3W>, MerkleTree<Blake3W>>(claro, Base128::new(7), &acc)
        .expect("y verifica");
    match catch_unwind(AssertUnwindSafe(|| ProbadorPeriodica { ocultacion: Some(OC) }.prove(traza))) {
        Ok(Err(ProverError::AsercionPeriodicaBajoOculta { columna: 1, primer_paso: 0, paso: 4 })) => {},
        Ok(Err(e)) => panic!("oculta, otro error: {e}"),
        Ok(Ok(_)) => panic!("oculta prueba una AIR con asercion periodica"),
        Err(_) => panic!("oculta entra en PANICO con una asercion periodica (la 138)"),
    }
    let e = ProverError::GradoDeDeep { esperado: 510, real: 509 };
    assert_eq!(e.to_string(), "el polinomio DEEP tiene grado 509 y no el esperado 510");
    // y la 137 con un suceso, no solo con su texto: una traza DEGENERADA en claro -la constante
    // sola, sin el contador, con la transicion identicamente nula- deja el DEEP sin grado, y el
    // probador devuelve GradoDeDeep en vez de caer en el aserto de winterfell
    let degenerada = TraceTable::init(vec![vec![Base128::new(7); 64]]);
    match catch_unwind(AssertUnwindSafe(|| ProbadorDegenerada.prove(degenerada))) {
        Ok(Err(ProverError::GradoDeDeep { esperado: 62, real })) => assert!(real < 62, "{real}"),
        Ok(Err(e)) => panic!("la degenerada, otro error: {e}"),
        Ok(Ok(_)) => panic!("la traza degenerada prueba"),
        Err(_) => panic!("la traza degenerada entra en PANICO (la 137)"),
    }
}
