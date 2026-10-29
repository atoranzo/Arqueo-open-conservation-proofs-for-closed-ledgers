//! **§906 (RFC-0018 D-G y E3, en LABORATORIO): el registro de las familias de la marca v2.**
//!
//! La marca v2 del corte 0.5 (`winter_air::MarcaV2`) lleva la familia de la prueba en un `u16`, la
//! version de su AIR en otro y el perfil de opciones en un byte, ademas de m y, desde el §899, las
//! columnas aleatorias. Este modulo es el UNICO productor del registro en el codigo, como pide la
//! D-G: una familia por AIR con probador oculto -las 23 con fila en la tabla de D-B del RFC-0009,
//! en su orden-, la version inicial con la que cada una entra en el corte y el perfil de produccion.
//! El registro normativo es la seccion 4 bis de `spec/NUCLEO.md`, que copia esta tabla; un KAT por
//! entrada, los 25 bytes de la marca de cada familia con m = 64, version 1, perfil 1 y dos columnas,
//! vive en los tests de este modulo hasta que el corte lo lleve a `spec/vectors/nucleo/`.
//!
//! Lo que NO hace: ningun probador de produccion escribe la marca v2 hasta el corte, ningun juez la
//! acepta -`Marca::leer` la rechaza como desconocida- y el cero no se asigna en ningun campo.

use winter_air::MarcaV2;

/// El perfil de produccion: el indice 1 de la tabla de opciones de la spec; con un solo
/// `OptionSet` por version del cable es una constante (D-G). El 0 no se asigna y el 255 esta
/// reservado para otro motor.
pub const PERFIL_PRODUCCION: u8 = 1;

/// La version de la AIR con la que cada familia entra en el corte; sube segun la regla de
/// extension de la D-G, y el 0 no se asigna.
pub const VERSION_INICIAL: u16 = 1;

/// Las familias, con su numero: las 23 AIR con probador oculto, en el orden de la tabla de D-B del
/// RFC-0009 (el de `FILAS` en el testigo de lo que revela una prueba). El 0 no se asigna.
pub const FAMILIAS: &[(u16, &str)] = &[
    (1, "SendAir"),
    (2, "SendV2Air"),
    (3, "ClaimAir"),
    (4, "ClaimAirV2"),
    (5, "PrendaAir"),
    (6, "MintPendingClimbAir"),
    (7, "NullifierThresholdAir"),
    (8, "ThresholdAir"),
    (9, "AuditAir"),
    (10, "BurnAir"),
    (11, "SolvencyAir"),
    (12, "DoubleEntryAir"),
    (13, "BandaAir"),
    (14, "EdadAir"),
    (15, "CobroPendienteAir"),
    (16, "PagoEnCursoAir"),
    (17, "MerkleAir"),
    (18, "RefundAir"),
    (19, "RefundAirV2"),
    (20, "CreditClimbAir"),
    (21, "MintClimbAir"),
    (22, "FrozenClimbAir"),
    (23, "RecoveryClimbAir"),
];

/// El numero de la familia de una AIR, por su nombre; `None` si no es de las 23.
pub fn familia_de(air: &str) -> Option<u16> {
    FAMILIAS.iter().find(|(_, n)| *n == air).map(|(f, _)| *f)
}

/// El nombre de la AIR de una familia; `None` si el numero no esta en el registro.
pub fn nombre_de(familia: u16) -> Option<&'static str> {
    FAMILIAS.iter().find(|(f, _)| *f == familia).map(|(_, n)| *n)
}

/// La marca v2 de una AIR con `columnas` aleatorias, con la m de la casa, la version inicial y el
/// perfil de produccion; `None` si la AIR no es de las 23.
pub fn marca_v2_de(air: &str, columnas: u8) -> Option<MarcaV2> {
    familia_de(air).map(|familia| MarcaV2 {
        m: crate::M_OCULTACION,
        familia,
        version: VERSION_INICIAL,
        perfil: PERFIL_PRODUCCION,
        columnas,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use winter_air::{Marca, MarcaError, MarcaLaboratorio};

    /// Los 25 bytes de la marca v2 de cada familia, en hex y en su orden: m = 64, version 1,
    /// perfil 1 y dos columnas, las del corte (la 133). Una foto de la referencia: cambiarla es
    /// cambiar el corte.
    const KAT: &[&str] = &[
        "61727175656f3a6f63756c74613a3240000000010001000102",
        "61727175656f3a6f63756c74613a3240000000020001000102",
        "61727175656f3a6f63756c74613a3240000000030001000102",
        "61727175656f3a6f63756c74613a3240000000040001000102",
        "61727175656f3a6f63756c74613a3240000000050001000102",
        "61727175656f3a6f63756c74613a3240000000060001000102",
        "61727175656f3a6f63756c74613a3240000000070001000102",
        "61727175656f3a6f63756c74613a3240000000080001000102",
        "61727175656f3a6f63756c74613a3240000000090001000102",
        "61727175656f3a6f63756c74613a32400000000a0001000102",
        "61727175656f3a6f63756c74613a32400000000b0001000102",
        "61727175656f3a6f63756c74613a32400000000c0001000102",
        "61727175656f3a6f63756c74613a32400000000d0001000102",
        "61727175656f3a6f63756c74613a32400000000e0001000102",
        "61727175656f3a6f63756c74613a32400000000f0001000102",
        "61727175656f3a6f63756c74613a3240000000100001000102",
        "61727175656f3a6f63756c74613a3240000000110001000102",
        "61727175656f3a6f63756c74613a3240000000120001000102",
        "61727175656f3a6f63756c74613a3240000000130001000102",
        "61727175656f3a6f63756c74613a3240000000140001000102",
        "61727175656f3a6f63756c74613a3240000000150001000102",
        "61727175656f3a6f63756c74613a3240000000160001000102",
        "61727175656f3a6f63756c74613a3240000000170001000102",
    ];

    /// **El registro tiene las 23 familias, numeradas de 1 a 23 en su orden, y cada marca v2 da
    /// sus bytes del KAT**; la de produccion y la de laboratorio la rechazan como desconocida.
    /// **§912 (RFC-0018 D-G, en laboratorio): el juez de la marca v2 rechaza en el orden de la
    /// D-G con sus textos, letra por letra los de `verify_v2` y de `tools/segunda`:** el meta vacio
    /// (1b), la marca v1 (1c), otro prefijo, la de laboratorio, otra longitud, un campo a cero o
    /// el perfil 255 (1d), otra familia (1e), otra version (1f), otro perfil (1g) y otra m (1h); y
    /// acepta la de la casa con sus columnas.
    #[test]
    fn el_juez_de_la_marca_v2_rechaza_en_el_orden_de_la_d_g() {
        use winter_air::{Marca, MarcaLaboratorio, MarcaV2};
        let v2 = |m: usize, familia: u16, version: u16, perfil: u8, columnas: u8| {
            MarcaV2 { m, familia, version, perfil, columnas }.escribir()
        };
        assert_eq!(crate::comprobar_marca_v2(&v2(64, 7, 1, 1, 2), 7), Ok((64, 2)));
        assert_eq!(crate::comprobar_marca_v2(&v2(64, 23, 1, 1, 1), 23), Ok((64, 1)));
        let casos: Vec<(Vec<u8>, u16, &str)> = vec![
            (Vec::new(), 7, "marca: meta vacio, prueba anterior a zkssl/0.4; la juzga el kit de su version"),
            (Marca { m: 64 }.escribir(), 7, "marca v1: prueba de zkssl/0.4; la juzga el kit 0.4.x"),
            (MarcaLaboratorio { m: 64, columnas: 2 }.escribir(), 7, "marca desconocida"),
            (b"arqueo:oculta:3\x40\x00\x00\x00\x07\x00\x01\x00\x01\x02".to_vec(), 7, "marca desconocida"),
            (b"arqueo:oculta:2\x40\x00\x00\x00\x07\x00\x01\x00\x01".to_vec(), 7, "marca desconocida"),
            (b"arqueo:oculta:2\x40\x00\x00\x00\x00\x00\x01\x00\x01\x02".to_vec(), 7, "marca desconocida"),
            (b"arqueo:oculta:2\x40\x00\x00\x00\x07\x00\x01\x00\xff\x02".to_vec(), 7, "marca desconocida"),
            (b"arqueo:oculta:2\x40\x00\x00\x00\x07\x00\x01\x00\x01\x00".to_vec(), 7, "marca desconocida"),
            (v2(64, 8, 1, 1, 2), 7, "marca: la prueba es de la familia 8; se pide la 7"),
            (v2(64, 7, 2, 1, 2), 7, "marca: version 2 de la AIR de la familia 7; se acepta 1"),
            (v2(64, 7, 1, 2, 2), 7, "marca: perfil 2 desconocido"),
            (v2(32, 7, 1, 1, 2), 7, "marca: m = 32; la construccion pide m = 64"),
        ];
        for (meta, familia, texto) in casos {
            assert_eq!(crate::comprobar_marca_v2(&meta, familia), Err(texto.to_string()), "{meta:?}");
        }
        // el orden: una marca con la familia, la version, el perfil y la m mal cae por la familia
        assert_eq!(
            crate::comprobar_marca_v2(&v2(32, 8, 2, 2, 2), 7),
            Err("marca: la prueba es de la familia 8; se pide la 7".to_string())
        );
    }

    #[test]
    fn el_registro_de_familias_y_su_kat() {
        assert_eq!(FAMILIAS.len(), 23);
        assert_eq!(KAT.len(), 23);
        for (i, (f, n)) in FAMILIAS.iter().enumerate() {
            assert_eq!(*f as usize, i + 1, "{n}: numerada en su orden, sin el cero");
            assert_eq!(familia_de(n), Some(*f));
            assert_eq!(nombre_de(*f), Some(*n));
            assert!(FAMILIAS.iter().filter(|(_, m)| m == n).count() == 1, "{n}: una sola vez");
            let marca = marca_v2_de(n, 2).expect("de las 23");
            assert_eq!((marca.m, marca.version, marca.perfil, marca.columnas), (64, 1, 1, 2));
            let bytes = marca.escribir();
            assert_eq!(bytes.len(), winter_air::marca::LARGO_V2);
            let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
            assert_eq!(hex, KAT[i], "{n}: los bytes de su marca");
            assert_eq!(MarcaV2::leer(&bytes), Ok(Some(marca)), "{n}: ida y vuelta");
            assert_eq!(Marca::leer(&bytes), Err(MarcaError::Desconocida), "{n}: la produccion no la lee");
            assert_eq!(MarcaLaboratorio::leer(&bytes), Err(MarcaError::Desconocida), "{n}: ni el laboratorio de la 133");
        }
        assert_eq!(familia_de("WorkAir"), None, "el juguete no es una familia");
        assert_eq!(nombre_de(0), None);
        assert_eq!(nombre_de(24), None);
        assert_ne!(PERFIL_PRODUCCION, 0);
        assert_ne!(PERFIL_PRODUCCION, winter_air::PERFIL_RESERVADO);
        assert_ne!(VERSION_INICIAL, 0);
    }
}
