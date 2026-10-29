//! **§913 — El juez único de verificación** (ARQ-01 de `doc/blueprint-v2.md`, adoptado por el
//! autor en el §705; la 117 de `BACKLOG.md`).
//!
//! Toda verificacion de produccion de una prueba STARK de la casa pasa por [`juzgar`]: los quince
//! sitios de la capa (`crates/zk-ssl`), la pareja umbral (`verify_threshold_pair`, en
//! `stark-experiment`) y los cinco jueces de los sobres del kit (este crate). Lo que el juez fija, y
//! ninguna familia puede cambiar, es la POLITICA DE ACEPTACION: un solo `OptionSet`, el de
//! [`opciones`]; la forma OCULTA de la traza, derivada de la forma en claro que la familia declara y
//! comprobada ANTES de construir ninguna AIR -ancho + 1, el tramo auxiliar tal cual y 2T filas:
//! S529, D-AD del RFC-0009-, con la marca de la casa en el meta (§651, D3-META:
//! [`comprobar_marca`]); y el compromiso con sal, `MerkleConSal<Blake3>`, con la moneda
//! `DefaultRandomCoin<Blake3>`, fijos. Lo que la familia aporta por [`Juez`] es su AIR y su forma
//! en claro: nada mas.
//!
//! Por que un solo sitio: hasta el §913 cada uno de los 21 sitios construia su `OptionSet`,
//! derivaba la forma oculta y llamaba a `verify` con sus tipos, y nada obligaba a que todos
//! aceptaran con la misma politica (la 117). El corte zkssl/0.5 (RFC-0018, D-H) cambia la forma
//! -dos columnas aleatorias, la marca v2- y la juzga en el orden de la D-G: tocara este fichero y
//! no veintiuno.
//!
//! Los TEXTOS del rechazo los pone quien llama, con [`Rechazo`] delante: el kit dice «el enunciado
//! pide» y la capa «el enunciado exige», y los catalogos (`spec/PAQUETE.md` §5) y los manifiestos
//! de `spec/vectors/` citan cada uno el suyo; el §913 no cambia ningun texto ni ningun veredicto.
//!
//! La puerta (ARQ-01): ningun `verify::<` vivo fuera de este fichero en `crates/zk-ssl/src`
//! -tests, `metrics.rs` e `instrumento_*` aparte- ni en `crates/zk-ssl-air/src`, y ninguno en la
//! pareja umbral; la atan `toda_verificacion_viva_de_la_capa_pasa_por_el_juez`
//! (`crates/zk-ssl/src/lib.rs`) y `toda_verificacion_del_kit_pasa_por_el_juez` (`lib.rs` de este
//! crate). Lo que NO hace: no juzga el enunciado -eso sigue en cada familia, `comprobar_enunciado`-
//! ni lee el sobre; y no compila ningun probador (S465).

use winter_air::proof::Proof;
use winter_air::{Air, TraceInfo};
use winter_crypto::DefaultRandomCoin;
use winter_math::fields::f64::BaseElement;
use winter_verifier::{verify_v2, AcceptableOptions, VerifierError};

use crate::sal::MerkleConSal;
use crate::familias::{familia_de as familia_del_registro, PERFIL_PRODUCCION, VERSION_INICIAL};
use crate::{comprobar_marca_v2, opciones, Blake3, COLUMNAS_ALEATORIAS};

/// La forma de la traza EN CLARO de una familia: la que su probador construye antes de ocultar.
/// Las cuatro cuentas son las de `TraceInfo`: el ancho del tramo principal, el del auxiliar, sus
/// elementos aleatorios y la longitud T.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Forma {
    pub ancho: usize,
    pub ancho_aux: usize,
    pub aleatorios: usize,
    pub longitud: usize,
}

impl Forma {
    /// Una familia sin tramo auxiliar y de longitud fija: las trece de la capa, la pareja umbral y
    /// cuatro de las cinco del kit.
    pub const fn fija(ancho: usize, longitud: usize) -> Self {
        Forma { ancho, ancho_aux: 0, aleatorios: 0, longitud }
    }

    /// La forma OCULTA que la prueba tiene que declarar (D-AD del RFC-0009; lo que `Oculta::new`
    /// del fork construye): ancho + 1 -la columna aleatoria de `zkssl/0.4`-, el tramo auxiliar tal
    /// cual y 2T filas. Es la regla que `tools/check_publicadas.py` ata (ANCLAS_LONGITUD).
    pub const fn oculta(&self) -> (usize, usize, usize, usize) {
        let exige = (self.ancho + COLUMNAS_ALEATORIAS, self.ancho_aux, self.aleatorios, 2 * self.longitud);
        exige
    }
}

/// Lo que una familia aporta al juez: su AIR y la forma en claro de su traza para unas entradas
/// publicas (la edad la deriva de `m`; las demas son fijas). La politica de aceptacion no es suya:
/// es del juez, y es una.
pub trait Juez: Air<BaseField = BaseElement> {
    /// El nombre de la AIR en el registro de familias (`crate::familias::FAMILIAS`, la seccion 4
    /// bis de `spec/NUCLEO.md`): su numero es la familia que la marca v2 lleva (§914, RFC-0018 D-G).
    const NOMBRE: &'static str;
    fn forma(pi: &Self::PublicInputs) -> Forma;
}

/// La familia de la marca v2 de `J`, del registro; una AIR sin fila es un error de quien construye.
pub fn familia_de<J: Juez>() -> u16 {
    familia_del_registro(J::NOMBRE)
        .unwrap_or_else(|| panic!("{}: sin familia en el registro de la marca v2", J::NOMBRE))
}

/// Por que el juez rechaza. El texto lo pone quien llama (ver la cabecera).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Rechazo {
    /// No hay cabeza: menos de seis bytes, o un meta que no cabe (D-G, 1a).
    Cabeza,
    /// La marca v2 no es la de la familia (D-G, 1b a 1h): el texto de [`comprobar_marca_v2`].
    Marca(String),
    /// La prueba no declara la forma oculta de la familia: `real` es lo que declara y `exige`, la
    /// de [`Forma::oculta`].
    Forma { real: (usize, usize, usize, usize), exige: (usize, usize, usize, usize) },
    /// El verificador del fork la rechaza, con su error.
    Verificador(VerifierError),
}

/// Las opciones que el juez acepta, y solo esas: un `OptionSet` con [`opciones`].
pub fn aceptadas() -> AcceptableOptions {
    AcceptableOptions::OptionSet(vec![opciones()])
}

/// El meta de la traza, leido de la cabeza de la prueba sin deserializar el resto (D-G, 1a): los
/// cuatro bytes de la forma, el largo del meta en dos y el meta.
pub fn meta_de_la_cabeza(prueba: &[u8]) -> Result<&[u8], Rechazo> {
    if prueba.len() < 6 {
        return Err(Rechazo::Cabeza);
    }
    let largo = u16::from_le_bytes([prueba[4], prueba[5]]) as usize;
    prueba.get(6..6 + largo).ok_or(Rechazo::Cabeza)
}

/// El paso 1 del juez (RFC-0018 D-G, 1b a 1h): la marca v2 de la familia de `J`, con m y sus
/// columnas, antes que nada. Devuelve m y las columnas que declara.
pub fn comprobar_marca<J: Juez>(meta: &[u8]) -> Result<(usize, usize), Rechazo> {
    comprobar_marca_v2(meta, familia_de::<J>()).map_err(Rechazo::Marca)
}

/// Las opciones de la prueba son las del juez (D-G, 3b): la validacion del fork, antes de la forma.
pub fn comprobar_opciones(proof: &Proof) -> Result<(), Rechazo> {
    aceptadas().validate::<Blake3>(proof).map_err(Rechazo::Verificador)
}

/// La guarda de forma (S529; D-G, 3c): la forma oculta de `forma`, derivada por el juez, ANTES de
/// construir ninguna AIR. Una `TraceInfo` ajena en `AIR::new` entra en panico (el `assert_eq!`
/// del ancho); aqui es un `Err`. La marca ya no es parte de la forma: va antes, en el paso 1
/// (⚠️ §914: hasta el corte la comprobaba esta guarda, §651).
pub fn comprobar_forma(info: &TraceInfo, forma: &Forma) -> Result<(), Rechazo> {
    let real = (
        info.main_trace_width(),
        info.aux_segment_width(),
        info.get_num_aux_segment_rand_elements(),
        info.length(),
    );
    let exige = forma.oculta();
    if real != exige {
        return Err(Rechazo::Forma { real, exige });
    }
    Ok(())
}

/// EL juez, en el orden de la D-G: la marca v2 (1b a 1h), las opciones (3b), la forma oculta (3c)
/// y la verificacion del fork (3d), `verify_v2`, que vuelve a leer la marca y la pide con la
/// familia, la version inicial y el perfil de produccion. La UNICA llamada viva a `verify` de la
/// casa.
pub fn juzgar<J: Juez>(proof: Proof, pi: J::PublicInputs) -> Result<(), Rechazo> {
    let familia = familia_de::<J>();
    comprobar_marca::<J>(proof.trace_info().meta())?;
    comprobar_opciones(&proof)?;
    comprobar_forma(proof.trace_info(), &J::forma(&pi))?;
    verify_v2::<J, Blake3, DefaultRandomCoin<Blake3>, MerkleConSal<Blake3>>(
        proof,
        pi,
        &aceptadas(),
        familia,
        VERSION_INICIAL,
        PERFIL_PRODUCCION,
    )
    .map_err(Rechazo::Verificador)
}

/// Solo para tests: una cabeza de prueba con la marca v2 de `J` y nada detras, lo que deja pasar el
/// paso 1 del juez (la marca) y cae despues, en el enunciado o al leer la prueba entera.
#[cfg(test)]
pub(crate) fn solo_cabeza<J: Juez>() -> Vec<u8> {
    let meta = crate::familias::marca_v2_de(J::NOMBRE, COLUMNAS_ALEATORIAS as u8).expect("del registro").escribir();
    let mut b = vec![0u8, 0, 0, 4];
    b.extend_from_slice(&(meta.len() as u16).to_le_bytes());
    b.extend_from_slice(&meta);
    b
}

#[cfg(test)]
mod tests {
    use super::*;
    use winter_math::FieldElement;

    fn marca_v2(familia: u16) -> Vec<u8> {
        winter_air::MarcaV2 { m: crate::M_OCULTACION, familia, version: 1, perfil: 1, columnas: 2 }.escribir()
    }

    /// **§913, §914: la forma oculta se deriva de la clara -ancho + 2, el auxiliar tal cual, 2T-,
    /// y cualquier otra es un `Err` con su causa, antes de construir ninguna AIR.** Los casos del
    /// test de `comprobar_forma` de la capa (§651), que desde el §913 llama aqui; desde el §914 la
    /// marca va aparte y antes.
    #[test]
    fn la_forma_oculta_se_deriva_de_la_clara() {
        let f = Forma::fija(44, 512);
        assert_eq!(f.oculta(), (46, 0, 0, 1024));
        assert_eq!(comprobar_forma(&TraceInfo::new(46, 1024), &f), Ok(()));
        // la forma APAGADA, la del AIR, y la de zkssl/0.4, con una columna: en 0.5 solo hay una forma
        assert_eq!(
            comprobar_forma(&TraceInfo::new(44, 512), &f),
            Err(Rechazo::Forma { real: (44, 0, 0, 512), exige: (46, 0, 0, 1024) })
        );
        assert!(matches!(comprobar_forma(&TraceInfo::new(45, 1024), &f), Err(Rechazo::Forma { .. })));
        assert!(matches!(comprobar_forma(&TraceInfo::new(99, 1024), &f), Err(Rechazo::Forma { .. })));
        assert!(matches!(comprobar_forma(&TraceInfo::new(46, 128), &f), Err(Rechazo::Forma { .. })));
        // con tramo auxiliar, la de la edad con m = 4: el auxiliar y sus aleatorios no se ocultan
        let e = Forma { ancho: 83, ancho_aux: 6, aleatorios: 2, longitud: 128 };
        assert_eq!(e.oculta(), (85, 6, 2, 256));
    }

    /// **§914 (RFC-0018 D-G): la marca antes que nada, de la cabeza y sin deserializar el resto, con
    /// la familia del registro; y las cinco familias del kit declaran su forma en claro al juez, con
    /// sus constantes, y la politica de aceptacion es una, la de `opciones`.**
    #[test]
    fn la_marca_va_antes_que_nada_y_las_cinco_familias_del_kit_declaran_su_forma() {
        use crate::{banda, cobro_pendiente, pago_en_curso, prenda};
        // la cabeza: ancho 46, aux 0, aleatorios 0, log2 de 1024, el largo del meta y el meta
        let mut cabeza = vec![46u8, 0, 0, 10];
        let meta = marca_v2(familia_de::<prenda::PrendaAir>());
        cabeza.extend_from_slice(&(meta.len() as u16).to_le_bytes());
        cabeza.extend_from_slice(&meta);
        assert_eq!(meta_de_la_cabeza(&cabeza), Ok(&meta[..]));
        assert_eq!(comprobar_marca::<prenda::PrendaAir>(&meta), Ok((crate::M_OCULTACION, 2)));
        assert_eq!(meta_de_la_cabeza(&cabeza[..5]), Err(Rechazo::Cabeza));
        assert_eq!(meta_de_la_cabeza(&cabeza[..cabeza.len() - 1]), Err(Rechazo::Cabeza));
        // otra familia, la v1 y el meta vacio: las causas de la D-G, por su texto
        assert!(matches!(comprobar_marca::<prenda::PrendaAir>(&marca_v2(14)), Err(Rechazo::Marca(m)) if m.starts_with("marca: la prueba es de la familia 14; se pide la 5")));
        assert!(matches!(comprobar_marca::<prenda::PrendaAir>(&winter_air::Marca { m: 64 }.escribir()), Err(Rechazo::Marca(m)) if m.starts_with("marca v1")));
        assert!(matches!(comprobar_marca::<prenda::PrendaAir>(&[]), Err(Rechazo::Marca(m)) if m.starts_with("marca: meta vacio")));
        // las familias del registro, por el nombre de cada AIR
        assert_eq!(familia_de::<crate::EdadAir>(), 14);
        assert_eq!(familia_de::<banda::BandaAir>(), 13);
        assert_eq!(familia_de::<banda::BandaAirV1>(), 13);
        assert_eq!(familia_de::<cobro_pendiente::CobroPendienteAir>(), 15);
        assert_eq!(familia_de::<pago_en_curso::PagoEnCursoAir>(), 16);
        assert_eq!(familia_de::<prenda::PrendaAir>(), 5);
        let d = [BaseElement::ZERO; 4];
        let pi = crate::EdadPublicInputs { subraiz_pend: d, subraiz_meta: d, m: 4, n: 16, seq: 1, t: 0, emisor: 0, todos: true, k: 0 };
        assert_eq!(
            <crate::EdadAir as Juez>::forma(&pi),
            Forma { ancho: crate::ANCHO, ancho_aux: crate::ANCHO_AUX, aleatorios: crate::ALEATORIOS, longitud: crate::CICLO << 4 }
        );
        let b = banda::BandaPublicInputs { root: d, public_id: d, lower: BaseElement::ZERO, upper: BaseElement::ZERO };
        let f = <banda::BandaAir as Juez>::forma(&b);
        assert_eq!(f, Forma::fija(banda::ANCHO, banda::TRAZA));
        assert_eq!(<banda::BandaAirV1 as Juez>::forma(&b), f);
        let c = cobro_pendiente::CobroPendientePublicInputs {
            pending_root: d, pmeta_root: d, receptor: d, nacido: BaseElement::ZERO, inferior: BaseElement::ZERO, superior: BaseElement::ZERO,
        };
        let f = <cobro_pendiente::CobroPendienteAir as Juez>::forma(&c);
        assert_eq!(f, Forma::fija(cobro_pendiente::ANCHO, cobro_pendiente::TRAZA));
        assert_eq!(<cobro_pendiente::CobroPendienteAirV1 as Juez>::forma(&c), f);
        let p = pago_en_curso::PagoEnCursoPublicInputs {
            pending_root: d, pmeta_root: d, receptor: d, importe: BaseElement::ZERO, t: BaseElement::ZERO, nacido: BaseElement::ZERO,
        };
        assert_eq!(<pago_en_curso::PagoEnCursoAir as Juez>::forma(&p), Forma::fija(pago_en_curso::ANCHO, pago_en_curso::TRAZA));
        let r = prenda::PrendaPublicInputs { pending_root: d, receptor: d, marca: d };
        assert_eq!(<prenda::PrendaAir as Juez>::forma(&r), Forma::fija(prenda::ANCHO, prenda::TRAZA));
        match aceptadas() {
            AcceptableOptions::OptionSet(v) => assert_eq!(v, vec![opciones()]),
            _ => panic!("el juez acepta por nivel, no por conjunto"),
        }
    }
}
