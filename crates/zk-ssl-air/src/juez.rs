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
use winter_verifier::{verify, AcceptableOptions, VerifierError};

use crate::sal::MerkleConSal;
use crate::{comprobar_marca, opciones, Blake3};

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
        let exige = (self.ancho + 1, self.ancho_aux, self.aleatorios, 2 * self.longitud);
        exige
    }
}

/// Lo que una familia aporta al juez: su AIR y la forma en claro de su traza para unas entradas
/// publicas (la edad la deriva de `m`; las demas son fijas). La politica de aceptacion no es suya:
/// es del juez, y es una.
pub trait Juez: Air<BaseField = BaseElement> {
    fn forma(pi: &Self::PublicInputs) -> Forma;
}

/// Por que el juez rechaza. El texto lo pone quien llama (ver la cabecera).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Rechazo {
    /// La prueba no declara la forma oculta de la familia: `real` es lo que declara y `exige`, la
    /// de [`Forma::oculta`].
    Forma { real: (usize, usize, usize, usize), exige: (usize, usize, usize, usize) },
    /// El meta de la traza no es la marca de la casa con su m (§651): el texto de
    /// [`comprobar_marca`].
    Marca(String),
    /// El verificador del fork la rechaza, con su error.
    Verificador(VerifierError),
}

/// Las opciones que el juez acepta, y solo esas: un `OptionSet` con [`opciones`].
pub fn aceptadas() -> AcceptableOptions {
    AcceptableOptions::OptionSet(vec![opciones()])
}

/// La guarda de forma (S529, §651): la forma oculta de `forma`, derivada por el juez, y la marca
/// de la casa, ANTES de construir ninguna AIR. Una `TraceInfo` ajena en `AIR::new` entra en panico
/// (el `assert_eq!` del ancho); aqui es un `Err`.
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
    comprobar_marca(info.meta()).map_err(Rechazo::Marca)
}

/// EL juez: la forma oculta y la marca, y la verificacion del fork con la politica del juez. La
/// UNICA llamada viva a `verify` de la casa.
pub fn juzgar<J: Juez>(proof: Proof, pi: J::PublicInputs) -> Result<(), Rechazo> {
    comprobar_forma(proof.trace_info(), &J::forma(&pi))?;
    verify::<J, Blake3, DefaultRandomCoin<Blake3>, MerkleConSal<Blake3>>(proof, pi, &aceptadas())
        .map_err(Rechazo::Verificador)
}

#[cfg(test)]
mod tests {
    use super::*;
    use winter_math::FieldElement;
    use crate::M_OCULTACION;

    fn marca(m: usize) -> Vec<u8> {
        winter_air::Marca { m }.escribir()
    }

    /// **§913: la forma oculta se deriva de la clara -ancho + 1, el auxiliar tal cual, 2T- y la
    /// marca de la casa es parte de ella; cualquier otra es un `Err` con su causa, antes de
    /// construir ninguna AIR.** Los casos del test de `comprobar_forma` de la capa (§651), que
    /// desde el §913 llama aqui.
    #[test]
    fn la_forma_oculta_se_deriva_de_la_clara_y_la_marca_es_parte() {
        let f = Forma::fija(44, 512);
        assert_eq!(f.oculta(), (45, 0, 0, 1024));
        assert_eq!(comprobar_forma(&TraceInfo::with_meta(45, 1024, marca(M_OCULTACION)), &f), Ok(()));
        // la forma APAGADA, la del AIR: en zkssl/0.4 solo hay una forma
        assert_eq!(
            comprobar_forma(&TraceInfo::new(44, 512), &f),
            Err(Rechazo::Forma { real: (44, 0, 0, 512), exige: (45, 0, 0, 1024) })
        );
        // ancho ajeno, longitud ajena
        assert!(matches!(comprobar_forma(&TraceInfo::new(99, 1024), &f), Err(Rechazo::Forma { .. })));
        assert!(matches!(comprobar_forma(&TraceInfo::new(45, 128), &f), Err(Rechazo::Forma { .. })));
        // §651: la forma oculta con el meta VACIO llegaba a `AIR::new` y entraba en panico
        match comprobar_forma(&TraceInfo::new(45, 1024), &f) {
            Err(Rechazo::Marca(m)) => assert!(m.contains("marca"), "{m}"),
            otro => panic!("sin marca: {otro:?}"),
        }
        // y otra m tampoco es la de la casa
        assert!(matches!(comprobar_forma(&TraceInfo::with_meta(45, 1024, marca(4)), &f), Err(Rechazo::Marca(_))));
        // con tramo auxiliar, la de la edad con m = 3: el auxiliar y sus aleatorios no se ocultan
        let e = Forma { ancho: 83, ancho_aux: 6, aleatorios: 2, longitud: 64 };
        assert_eq!(e.oculta(), (84, 6, 2, 128));
    }

    /// **§913: las cinco familias del kit declaran su forma en claro al juez, con sus constantes;
    /// y la politica de aceptacion es una, la de `opciones`.**
    #[test]
    fn las_cinco_familias_del_kit_declaran_su_forma_y_la_politica_es_una() {
        use crate::{banda, cobro_pendiente, pago_en_curso, prenda};
        let d = [BaseElement::ZERO; 4];
        let pi = crate::EdadPublicInputs { subraiz_pend: d, subraiz_meta: d, m: 3, n: 8, seq: 1, t: 0, emisor: 0, todos: true, k: 0 };
        assert_eq!(
            <crate::EdadAir as Juez>::forma(&pi),
            Forma { ancho: crate::ANCHO, ancho_aux: crate::ANCHO_AUX, aleatorios: crate::ALEATORIOS, longitud: crate::CICLO << 3 }
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
