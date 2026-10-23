//! **§723, el KAT de la banda v2**, la que cierra la ventana del saldo privado.
//!
//! La foto del probador pristino (`kat_probador.rs`) fija la banda de la v1: se tomo con winterfell
//! 0.13.1 SIN bifurcar, y sigue midiendo eso, con `BandaAirV1`. La v2 cambia una columna periodica
//! (el bit 62 de cada segmento, a cero), y con ella las pruebas: este fichero fija la suya con el
//! MISMO montaje -el probador propio de la foto, `MerkleTree<Blake3>` y la ocultacion apagada, el
//! escenario del positivo y la banda `[0, 4_999_999]`-, medido en release.
//!
//! ⚠️ No es una foto de winterfell sin bifurcar: se tomo con el fork apagado, que la foto de la v1
//! dice igual a winterfell byte a byte. Es el pin de regresion de la v2: si la AIR, el probador o
//! la version clavada se mueven, aqui se ve. Quien lo mueva lo dice en su asiento.

/// (bytes de la prueba, blake3 de esos bytes) de la banda v2 con el montaje de la foto.
const KAT_V2: (usize, &str) =
    (49693, "79f93f1ddc3e3e3c6a6e62352c5fcf057243947b611af4c60c81e6ab3ae2a2ac");

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

mod banda_v2 {
    use super::{hex, KAT_V2};
    use crate::circuit_banda::{opciones, trazar, BandaAir, BandaPublicInputs};
    use crate::kat_probador::banda::{escenario, ProbadorBanda};
    use winterfell::crypto::hashers::Blake3_256;
    use winterfell::crypto::{DefaultRandomCoin, Digest as _, Hasher, MerkleTree};
    use winterfell::math::fields::f64::BaseElement;
    use winterfell::{verify, AcceptableOptions, Proof, Prover};
    type Blake3 = Blake3_256<BaseElement>;

    /// El positivo de la foto, con la v2: saldo 1_000_000 en la banda [0, 4_999_999]. Juzga el
    /// `verify` de winterfell con `MerkleTree` y la v2, y compara tamano y blake3 con [`KAT_V2`].
    /// Con la v1 (la foto), la misma traza da otra prueba: la de `kat_probador.rs`.
    #[test]
    #[cfg_attr(debug_assertions, ignore = "winterfell valida grados en depuracion: juez release")]
    fn kat_banda_v2() {
        let (w, root) = escenario(1_000_000);
        let (lower, upper) = (0u64, 4_999_999u64);
        let bytes = ProbadorBanda::<2> { options: opciones() }
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
        let d = Blake3::hash(&bytes);
        assert_eq!(bytes.len(), KAT_V2.0, "la banda v2 ya no mide lo que su KAT");
        assert_eq!(hex(&d.as_bytes()), KAT_V2.1, "la banda v2 ya no es byte a byte su KAT");
    }
}
