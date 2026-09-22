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
        probar, trazar, verificar, BandaPublicInputs, BandaWitness, Digest, PROFUNDIDAD,
    };
    use crate::merkle::{native_merge, MerklePath};
    use crate::native::{native_climb, native_leaf_salted};
    use winterfell::math::{fields::f64::BaseElement, FieldElement};

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

    /// El positivo de `el_positivo_verifica`: saldo 1_000_000 en la banda [0, 4_999_999].
    #[test]
    #[cfg_attr(debug_assertions, ignore = "winterfell valida grados en depuracion: juez release")]
    fn kat_banda() {
        let (w, root) = escenario(1_000_000);
        let (lower, upper) = (0u64, 4_999_999u64);
        let (bytes, _pi) = probar(trazar(&w, lower, upper)).expect("probar");
        let declarado = BandaPublicInputs {
            root,
            public_id: w.public_id,
            lower: BaseElement::new(lower),
            upper: BaseElement::new(upper),
        };
        verificar(&bytes, &declarado).expect("verificar");
        asierta("banda", &bytes);
    }
}

mod edad {
    use super::asierta;
    use crate::circuit_edad::{construir, probar, verificar, Digest, Enunciado};
    use winterfell::math::{fields::f64::BaseElement, FieldElement};

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

    /// El positivo de `la_cuenta_de_viejos_es_la_del_libro`: T = 70, todos, cinco viejas.
    #[test]
    #[cfg_attr(debug_assertions, ignore = "winterfell valida grados en depuracion: juez release")]
    fn kat_edad() {
        let (hojas, meta) = libro();
        let e = en(70, 0, true);
        let (bytes, pi) = probar(construir(&hojas, &meta, &e).expect("construir")).expect("probar");
        assert_eq!(pi.k, 5, "la cuenta de viejas no es la del libro");
        verificar(&bytes, &pi).expect("verificar");
        asierta("edad", &bytes);
    }
}
