//! La FOTO DEL PROBADOR PRISTINO (RFC-0009 E3a-0, S532): tres pruebas con entradas fijas,
//! producidas por winterfell 0.13.1 SIN bifurcar y verificadas por su juez, con su tamano y su
//! blake3 como constantes. Cuando el fork entre apagado (E3a-1) cada una tiene que salir byte a
//! byte igual: estas tres constantes son el falsador de <<apagado = winterfell>> (D-R). Tomadas
//! con el fork dentro serian un verde que se certifica a si mismo; por eso nacen antes.
//!
//! Los montajes son los de los tests positivos de `lib.rs`, `circuit_banda.rs` y `circuit_edad.rs`,
//! COPIADOS y no compartidos: la foto no se mueve porque alguien afine un positivo. Solo se mueve
//! con la version clavada de winterfell, y quien la mueva lo dice en su asiento.
//!
//! Medido fuera del arbol por el PASTE-KAT-M (`f7e4b67352204b1a`, salida `8faab897d07a7b05`) sobre
//! `81549de`: dos corridas en release, bytes identicos; los `.bin` viven en Downloads del autor
//! (`KAT-M-20260922-225843/`). Sus sha256, para la shell (`sha256sum`):
//!   work  76eaa1f27e635a61541258391ad110675ce953e1bb0feeb5611a18c123a7af39
//!   banda a75a3ec29a0fed86700ce819d1111c155ad3a001ea8ae3d0627e889635524c9b
//!   edad  1e51809bc37d6068c7a8eab80b9eadf09be00d96c82c39611fc08697af5e34ac
//! Los tres corren solo en release: winterfell valida grados en depuracion.
//!
//! Desde el S536 (RFC-0009 E3b-1, D-Y) la foto lleva sus PROPIOS probadores y sus propios jueces
//! para `banda` y `edad`: los `impl Prover` de `circuit_banda.rs` y `circuit_edad.rs` COPIADOS,
//! con `MerkleTree<Blake3>` como `VC` y la ocultacion apagada (el `None` que el trait provee), y
//! el `verify` de winterfell con `MerkleTree` como juez. Cuando el corte 2 de E3b cambie el `VC`
//! de produccion y encienda la ocultacion, la foto sigue midiendo UNA sola cosa: nucleo apagado
//! con `MerkleTree` = winterfell byte a byte. `WorkProver` no es de produccion: sigue siendo el
//! de la foto y la referencia de `regresion_apagada`.
use winterfell::crypto::{hashers::Blake3_256, Digest as _, Hasher};

/// (nombre, bytes de la prueba, blake3 de esos bytes), de la SALIDA del PASTE-KAT-M.
const FOTO: [(&str, usize, &str); 3] = [
    ("work", 3539, "255eda7cd3f6abea9618248dc064dc3b67914b489bacf101e5c5f1960c87beb6"),
    ("banda", 49418, "3625b2a61aaef9e967fad81420d6d5bf5f075e62724648bebf99296f43749e11"),
    ("edad", 66880, "767ce6e010f7a72221caf11eaaf6a58839bf10d230eae0c0588bb5b041191bc4"),
];

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// Compara los bytes de una prueba con su foto: tamano Y blake3 (el hash que ya viaja con
/// winterfell). Un rojo aqui nombra que prueba se movio y en cual de las dos medidas.
fn asierta(nombre: &str, bytes: &[u8]) {
    let (_, n, b3) = FOTO.iter().find(|(k, _, _)| *k == nombre).expect("foto sin ese nombre");
    let d = Blake3_256::<winterfell::math::fields::f64::BaseElement>::hash(bytes);
    assert_eq!(bytes.len(), *n, "la prueba {nombre} ya no mide lo que la foto");
    assert_eq!(hex(&d.as_bytes()), *b3, "la prueba {nombre} ya no es byte a byte la de la foto");
}

mod work {
    use super::asierta;
    use crate::{build_trace, WorkAir, WorkProver};
    use winterfell::crypto::{hashers::Blake3_256, DefaultRandomCoin, MerkleTree};
    use winterfell::math::fields::f128::BaseElement;
    use winterfell::{
        verify, AcceptableOptions, BatchingMethod, FieldExtension, Proof, ProofOptions, Prover,
    };
    type Blake3 = Blake3_256<BaseElement>;

    fn native_result(start: BaseElement, n: usize) -> BaseElement {
        let mut x = start;
        for _ in 0..(n - 1) {
            x = x * x * x + BaseElement::new(42);
        }
        x
    }

    /// El montaje de `valid_computation_produces_verifiable_stark_proof` (lib.rs): start 3,
    /// n 8, opciones 32/8/0/None/8/31.
    #[test]
    fn kat_work() {
        let start = BaseElement::new(3);
        let n = 8;
        let trace = build_trace(start, n);
        let result = native_result(start, n);
        let options = ProofOptions::new(
            32,
            8,
            0,
            FieldExtension::None,
            8,
            31,
            BatchingMethod::Linear,
            BatchingMethod::Linear,
        );
        let prover = WorkProver::new(options);
        let proof: Proof = prover.prove(trace).expect("prove");
        let bytes = proof.to_bytes();
        let min_opts = AcceptableOptions::OptionSet(vec![prover.options().clone()]);
        verify::<WorkAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            proof, result, &min_opts,
        )
        .expect("verify");
        asierta("work", &bytes);
    }
}

mod banda {
    use super::asierta;
    use crate::circuit_banda::{
        opciones, trazar, BandaAir, BandaPublicInputs, BandaWitness, Digest, COL_ID, COL_LOWER,
        COL_UPPER, PROFUNDIDAD, ROW_RAIZ,
    };
    use crate::merkle::{native_merge, MerklePath};
    use crate::native::{native_climb, native_leaf_salted};
    use winterfell::crypto::hashers::Blake3_256;
    use winterfell::crypto::{DefaultRandomCoin, MerkleTree};
    use winterfell::math::{fields::f64::BaseElement, FieldElement};
    use winterfell::matrix::ColMatrix;
    use winterfell::{
        verify, AcceptableOptions, AuxRandElements, CompositionPoly, CompositionPolyTrace,
        ConstraintCompositionCoefficients, DefaultConstraintCommitment, DefaultConstraintEvaluator,
        DefaultTraceLde, PartitionOptions, Proof, ProofOptions, Prover, StarkDomain, TraceInfo,
        TracePolyTable, TraceTable,
    };
    type Blake3 = Blake3_256<BaseElement>;

    /// El probador PROPIO de la foto (D-Y): el `impl Prover for BandaProver` de `circuit_banda.rs`
    /// (S477), copiado y no compartido; `MerkleTree<Blake3>` como `VC` y la ocultacion apagada,
    /// el `None` que el trait provee. Solo lo mueve una version nueva de winterfell, con asiento.
    struct ProbadorBanda {
        options: ProofOptions,
    }

    impl Prover for ProbadorBanda {
        type BaseField = BaseElement;
        type Air = BandaAir;
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

        /// **Las entradas publicas DERIVADAS de la traza.** La identidad sale de `COL_ID` en la
        /// fila 0, igual que en el hermano; lo que impide que un probador declare una identidad
        /// que no es la suya son las CUATRO aserciones del AIR (D-2b), que el verificador comprueba
        /// contra el `public_id` que EL declara.
        fn get_pub_inputs(&self, trace: &Self::Trace) -> BandaPublicInputs {
            BandaPublicInputs {
                root: [
                    trace.get(4, ROW_RAIZ),
                    trace.get(5, ROW_RAIZ),
                    trace.get(6, ROW_RAIZ),
                    trace.get(7, ROW_RAIZ),
                ],
                public_id: [
                    trace.get(COL_ID, 0),
                    trace.get(COL_ID + 1, 0),
                    trace.get(COL_ID + 2, 0),
                    trace.get(COL_ID + 3, 0),
                ],
                lower: trace.get(COL_LOWER, 0),
                upper: trace.get(COL_UPPER, 0),
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

    /// `escenario(balance)` de los tests de circuit_banda.rs, copiado: el operador sin clave.
    fn escenario(balance: u64) -> (BandaWitness, Digest) {
        let mut vacios = vec![[BaseElement::ZERO; 4]];
        for k in 1..=PROFUNDIDAD {
            let previo = vacios[k - 1];
            vacios.push(native_merge(previo, previo));
        }
        let public_id = [
            BaseElement::new(0xA11CE),
            BaseElement::new(0xA0D17),
            BaseElement::new(0x0DDBA11),
            BaseElement::new(0x5EA51DE),
        ];
        let leaf_salt = [
            BaseElement::new(0x5A17_0001),
            BaseElement::new(0x5A17_0002),
            BaseElement::new(0x5A17_0003),
            BaseElement::new(0x5A17_0004),
        ];
        let nonce = BaseElement::ZERO;
        let mut siblings = Vec::with_capacity(PROFUNDIDAD);
        let mut is_right = Vec::with_capacity(PROFUNDIDAD);
        for nivel in 0..PROFUNDIDAD {
            siblings.push(vacios[nivel]);
            is_right.push(nivel % 3 == 0);
        }
        let path = MerklePath { siblings, is_right };
        let root = native_climb(
            native_leaf_salted(public_id, BaseElement::new(balance), nonce, leaf_salt),
            &path,
        );
        (BandaWitness { public_id, balance, nonce, leaf_salt, path }, root)
    }

    /// El positivo de `el_positivo_verifica`: saldo 1_000_000 en la banda [0, 4_999_999]. Prueba
    /// el probador PROPIO y juzga el `verify` de winterfell con `MerkleTree` (D-Y), no el kit.
    #[test]
    #[cfg_attr(debug_assertions, ignore = "winterfell valida grados en depuracion: juez release")]
    fn kat_banda() {
        let (w, root) = escenario(1_000_000);
        let (lower, upper) = (0u64, 4_999_999u64);
        let bytes = ProbadorBanda { options: opciones() }
            .prove(trazar(&w, lower, upper))
            .expect("prove")
            .to_bytes();
        let declarado = BandaPublicInputs {
            root,
            public_id: w.public_id,
            lower: BaseElement::new(lower),
            upper: BaseElement::new(upper),
        };
        let aceptadas = AcceptableOptions::OptionSet(vec![opciones()]);
        verify::<BandaAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            Proof::from_bytes(&bytes).expect("from_bytes"),
            declarado,
            &aceptadas,
        )
        .expect("verify");
        asierta("banda", &bytes);
    }
}

mod edad {
    use super::asierta;
    use crate::circuit_edad::{construir, Digest, Enunciado, TrazaEdad};
    use winterfell::crypto::hashers::Blake3_256;
    use winterfell::crypto::{DefaultRandomCoin, MerkleTree};
    use winterfell::math::{batch_inversion, fields::f64::BaseElement, FieldElement};
    use winterfell::matrix::ColMatrix;
    use winterfell::{
        verify, AcceptableOptions, AuxRandElements, CompositionPoly, CompositionPolyTrace,
        ConstraintCompositionCoefficients, DefaultConstraintCommitment, DefaultConstraintEvaluator,
        DefaultTraceLde, PartitionOptions, Proof, ProofOptions, Prover, StarkDomain, Trace,
        TraceInfo, TracePolyTable,
    };
    use zk_ssl_air::{
        carril, codificar, opciones, potencias, EdadAir, EdadPublicInputs, CICLO, C_A, C_ACT, C_B,
        C_CICLO, C_M, C_P, C_VIVO,
    };
    type Blake3 = Blake3_256<BaseElement>;

    const CERO: Digest = [BaseElement::ZERO; 4];

    /// `d`, `libro` y `en` de los tests de circuit_edad.rs, copiados: trece posiciones, huecos
    /// en los multiplos de 4, emisor `p % 3`, nacido `10 p`, `seq = 130`.
    fn d(k: u64) -> Digest {
        [
            BaseElement::new(k + 1),
            BaseElement::new(3 * k + 2),
            BaseElement::new(7 * k + 3),
            BaseElement::new(11 * k + 4),
        ]
    }

    fn libro() -> (Vec<Digest>, Vec<Option<(u64, u64)>>) {
        let (mut hojas, mut meta) = (Vec::new(), Vec::new());
        for p in 0..13u64 {
            if p % 4 == 0 {
                hojas.push(CERO);
                meta.push(None);
            } else {
                hojas.push(d(p));
                meta.push(Some((p % 3, 10 * p)));
            }
        }
        (hojas, meta)
    }

    fn en(t: u64, emisor: u64, todos: bool) -> Enunciado {
        Enunciado { seq: 130, t, emisor, todos }
    }

    /// Copiada de `circuit_edad.rs` (S463) para el probador propio de la foto (D-Y):
    /// **La traza auxiliar**: por ciclo, los inversos de los cinco terminos de la fila 0
    /// (cuatro lecturas y la hoja de pendientes) y de los tres de la fila 6 (los dos nodos y la
    /// hoja de meta, con los valores de la fila 7), y la suma acumulada que el AIR comprueba fila
    /// a fila.
    fn construir_auxiliar<E: FieldElement<BaseField = BaseElement>>(
        m: &ColMatrix<BaseElement>,
        r: &[E],
    ) -> ColMatrix<E> {
        let filas = m.num_rows();
        let hojas = filas / CICLO;
        let pot = potencias(r[0]);
        let beta = r[1];
        let v = |col: usize, fila: usize| E::from(m.get(col, fila));
        let dig = |col: usize, fila: usize| {
            [v(col, fila), v(col + 1, fila), v(col + 2, fila), v(col + 3, fila)]
        };
        let (ca, cb) = (carril::<E>(false), carril::<E>(true));
        let n_hojas = E::from(BaseElement::new(hojas as u64));
        let dos = E::from(2u32);

        let mut den = Vec::with_capacity(8 * hojas);
        for c in 0..hojas {
            let (f0, f7) = (c * CICLO, c * CICLO + 7);
            let ciclo = v(C_CICLO, f0);
            let vivo = v(C_VIVO, f0);
            den.push(codificar(&pot, beta, dos * ciclo, dig(C_A + 4, f0), ca));
            den.push(codificar(&pot, beta, dos * ciclo + E::ONE, dig(C_A + 8, f0), ca));
            den.push(codificar(&pot, beta, dos * ciclo, dig(C_B + 4, f0), cb));
            den.push(codificar(&pot, beta, dos * ciclo + E::ONE, dig(C_B + 8, f0), cb));
            den.push(codificar(&pot, beta, n_hojas + ciclo, dig(C_P, f0), ca));
            let meta = dig(C_M + 4, f7);
            let hoja_meta = [vivo * meta[0], vivo * meta[1], vivo * meta[2], vivo * meta[3]];
            den.push(codificar(&pot, beta, ciclo, dig(C_A + 4, f7), ca));
            den.push(codificar(&pot, beta, ciclo, dig(C_B + 4, f7), cb));
            den.push(codificar(&pot, beta, n_hojas + ciclo, hoja_meta, cb));
        }
        let inv = batch_inversion(&den);

        let mut h = vec![vec![E::ZERO; filas]; 5];
        for c in 0..hojas {
            for k in 0..5 {
                h[k][c * CICLO] = inv[8 * c + k];
            }
            for k in 0..3 {
                h[k][c * CICLO + 6] = inv[8 * c + 5 + k];
            }
        }
        let mut s = vec![E::ZERO; filas];
        for fila in 0..filas - 1 {
            let act = v(C_ACT, fila);
            let delta = match fila % CICLO {
                0 => h[4][fila] - act * (h[0][fila] + h[1][fila] + h[2][fila] + h[3][fila]),
                6 => act * (h[0][fila] + h[1][fila]) + h[2][fila],
                _ => E::ZERO,
            };
            s[fila + 1] = s[fila] + delta;
        }
        h.push(s);
        ColMatrix::new(h)
    }

    /// El probador PROPIO de la foto (D-Y): el `impl Prover for EdadProver` de `circuit_edad.rs`
    /// (S463), copiado y no compartido con su traza auxiliar; `MerkleTree<Blake3>` como `VC` y la
    /// ocultacion apagada, el `None` que el trait provee. Solo lo mueve una version nueva de
    /// winterfell, con asiento.
    struct ProbadorEdad {
        options: ProofOptions,
    }

    impl Prover for ProbadorEdad {
        type BaseField = BaseElement;
        type Air = EdadAir;
        type Trace = TrazaEdad;
        type HashFn = Blake3;
        type VC = MerkleTree<Blake3>;
        type RandomCoin = DefaultRandomCoin<Blake3>;
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

        fn build_aux_trace<E: FieldElement<BaseField = Self::BaseField>>(
            &self,
            main_trace: &Self::Trace,
            aux_rand_elements: &AuxRandElements<E>,
        ) -> ColMatrix<E> {
            construir_auxiliar(main_trace.main_segment(), aux_rand_elements.rand_elements())
        }
    }

    /// El positivo de `la_cuenta_de_viejos_es_la_del_libro`: T = 70, todos, cinco viejas. Prueba
    /// el probador PROPIO y juzga el `verify` de winterfell con `MerkleTree` (D-Y), no el kit.
    #[test]
    #[cfg_attr(debug_assertions, ignore = "winterfell valida grados en depuracion: juez release")]
    fn kat_edad() {
        let (hojas, meta) = libro();
        let e = en(70, 0, true);
        let traza = construir(&hojas, &meta, &e).expect("construir");
        let prover = ProbadorEdad { options: opciones() };
        let pi = prover.get_pub_inputs(&traza);
        let bytes = prover.prove(traza).expect("prove").to_bytes();
        assert_eq!(pi.k, 5, "la cuenta de viejas no es la del libro");
        let aceptadas = AcceptableOptions::OptionSet(vec![opciones()]);
        verify::<EdadAir, Blake3, DefaultRandomCoin<Blake3>, MerkleTree<Blake3>>(
            Proof::from_bytes(&bytes).expect("from_bytes"),
            pi,
            &aceptadas,
        )
        .expect("verify");
        asierta("edad", &bytes);
    }
}
