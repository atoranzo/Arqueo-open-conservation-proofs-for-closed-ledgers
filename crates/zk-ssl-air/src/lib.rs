//! **zk-ssl-air: los AIR que un tercero VERIFICA, sin el probador** (RFC-0007 D-F; E4b-1, §463).
//!
//! Este crate lleva lo que hace falta para JUZGAR una prueba y nada de lo que hace falta para
//! PRODUCIRLA: el AIR (sus restricciones, sus aserciones y sus entradas publicas), el verificador
//! de `winter-verifier` y las raices nativas que el juez recompone. La traza y el `Prover` viven en
//! `stark-experiment` (`circuit_edad`), que depende de este crate: el AIR tiene UN productor, y el
//! probador y el kit lo comparten. Depende de `winter-air`, `winter-crypto`, `winter-math` y
//! `winter-verifier` sueltos y clavados con `=`, de `zk-ssl-hash`, y de `rand_core` con `getrandom`
//! para la sal de [`sal`] (RFC-0009, E3b-0, D-X); no arrastra `winter-prover`. La sal es la
//! excepcion medida al <<nada de lo que hace falta para PRODUCIRLA>>: un solo tipo para las dos
//! orillas, porque `VectorCommitment::new` no admite semilla; el kit lo compila y no lo llama.
//!
//! ## La prueba de edad (RFC-0007 D-E), en una frase
//!
//! Sobre las posiciones `0..next_pending` de los arboles de pendientes y de meta, cuya raiz firma
//! la cabeza v5: el numero de posiciones VIVAS con `seq - nacido >= T` (y, si se nombra, con ese
//! emisor) es a lo sumo `K`. La caja vacia es `K = 0`; el tope por cuenta, `T = 0`; la
//! concentracion por cuenta, un emisor nombrado. Las formas por IMPORTE no se prueban: el operador
//! no guarda la apertura del compromiso (RFC-0003, Seguridad; RFC-0007, correccion del S464) y lo
//! que el circuito no restringe no se afirma (decision D-1 de E4b-1, por la constitucion).
//!
//! ## Como se cablea el arbol: un argumento de multiconjunto (decision D-5)
//!
//! La traza tiene `N = 2^m` ciclos de ocho filas (siete rondas de Rescue y una de enlace). El ciclo
//! `c` calcula el nodo `c` de los dos arboles en orden de monticulo (hijos `2c` y `2c+1`, hojas en
//! `N + p`) y la hoja de meta de la posicion `c`. Quien alimenta a quien no lo ve una restriccion
//! de dos filas: lo ata una suma acumulada en la traza AUXILIAR, `sum 1/(beta - cod(escrito)) - sum
//! 1/(beta - cod(leido)) = 1/(beta - cod(raiz A)) + 1/(beta - cod(raiz B))`, con `cod` una
//! combinacion aleatoria de etiqueta, digest y carril. Cada etiqueta se escribe y se lee una sola
//! vez, y la unica que no se lee es la raiz, que el verificador conoce.
//!
//! ## Lo que NO prueba
//!
//! Nada sobre lo que nunca entro en el arbol (eso es H5b); nada sobre importes ni saldos; nada
//! sobre otra cabeza que la que firma las dos raices. La subida de `m` a 32 niveles la hace el
//! JUEZ, en nativo ([`raiz_desde_subraiz`]), con las constantes de subarbol vacio: el AIR no
//! depende de la profundidad. Y un pendiente vivo cuya meta sea la hoja cero (legado anterior a
//! R-2a) no se puede probar: la prueba falla cerrada (decision D-2).
//!
//! ## Contra que cabeza (E4b-2, S465)
//!
//! [`verificar_contra_cabeza`] es la regla que ENLAZA la prueba a una cabeza v5 firmada, y vive
//! aqui -no en el mando del kit- para que tenga un solo productor en el crate que el tercero
//! compila. De la cabeza toma `seq`, `nextPending`, `pendingRoot` y `pmetaRoot`; `m` lo DERIVA
//! de la marca ([`m_canonico`], la misma regla que el probador), y del sobre solo toma lo que el
//! sobre afirma: la edad, la cota, el emisor o todos, y las dos subraices.

use winter_air::proof::Proof;
use winter_air::{
    Air, AirContext, Assertion, AuxRandElements, BatchingMethod, EvaluationFrame, FieldExtension,
    ProofOptions, TraceInfo, TransitionConstraintDegree,
};
use winter_crypto::hashers::{Blake3_256, Rp64_256};
use winter_math::fields::f64::BaseElement;
use winter_math::{ExtensionOf, FieldElement, ToElements};
use zk_ssl_hash::{native_merge, DOMINIO_META_PENDIENTE};

/// **El AIR de la BANDA del saldo** (RFC-0007, E5): el segundo juez de este crate.
pub mod banda;

/// **El AIR del COBRO PENDIENTE** (RFC-0008, E1): el tercer juez de este crate.
pub mod cobro_pendiente;

/// **El AIR del PAGO EN CURSO** (RFC-0008, E2): el cuarto juez de este crate.
pub mod pago_en_curso;

/// **El AIR de la PRENDA** (RFC-0008, E3): el quinto juez de este crate.
pub mod prenda;

/// **El compromiso con SAL** (RFC-0009, E3b-0, D-X): el `VectorCommitment` que los probadores con
/// fila en D-B y el kit recibiran como `VC` cuando E3b encienda la ocultacion. Hoy nadie lo
/// declara como `VC`: entra como tipo, con sus testigos, y no mueve un byte de ninguna prueba.
pub mod sal;
// §906 (RFC-0018 D-G, en laboratorio): el registro de las familias de la marca v2, un solo productor
pub mod familias;
// §913 (ARQ-01, la 117 de BACKLOG.md): el juez unico de verificacion, por el que pasa toda
// verificacion de produccion -la capa, la pareja umbral y los cinco jueces del kit-.
pub mod juez;

/// El digest de la casa: cuatro elementos (el de `zk-ssl-hash`).
pub type Digest = zk_ssl_hash::Digest;

type Blake3 = Blake3_256<BaseElement>;

// ------------------------------------------------------------------ la forma de la traza

/// Filas por ciclo: siete rondas de Rescue y la fila de enlace.
pub const CICLO: usize = 8;
/// El estado de Rescue (`Rp64_256`).
pub const ESTADO: usize = 12;
const RONDAS: usize = 7;
/// La profundidad de los arboles de pendientes y de meta de la capa.
pub const PROFUNDIDAD: usize = 32;
/// Bits del resto que prueba que una posicion es JOVEN (`T - 1 - edad`).
pub const BITS: usize = 32;
/// Bits de un indice de cuenta: el arbol de cuentas de la capa tiene 32 niveles. El emisor que
/// el enunciado nombra es un indice, y [`comprobar_emisor`] lo acota aqui.
///
/// ⚠️ Tres sitios dicen este numero: aqui, `BITS_CUENTA` de `tools/segunda/airs.py` y la capacidad
/// del arbol de cuentas de la capa. Un test de este crate lee la segunda, y uno de la capa compara
/// la capacidad. Se cambia en los tres a la vez.
pub const BITS_CUENTA: u32 = 32;

/// **El emisor es un indice de cuenta** (§718, la canonicidad): menor que `2^BITS_CUENTA`.
/// El emisor entra al AIR por `BaseElement::new`, que reduce modulo `p`, y `emisor + p` se probaba
/// como `emisor`. Un solo productor de la cota y de su causa: el juez ([`comprobar_enunciado`]), el
/// productor de la capa y la linea de ordenes del nodo la llaman, y la causa es la misma en los
/// tres.
pub fn comprobar_emisor(emisor: u64) -> Result<(), String> {
    if emisor >= 1u64 << BITS_CUENTA {
        return Err(format!(
            "emisor = {emisor} no es un indice de cuenta: no cabe en {BITS_CUENTA} bits"
        ));
    }
    Ok(())
}

/// **El tope de `m`** (§717, el coste del juez de la edad): la prueba cubre el subarbol
/// `[0, 2^m)`, y [`comprobar_enunciado`] rechaza `m > M_MAX` ANTES de leer la prueba y de construir
/// el AIR, con la causa `m = .. fuera de M_MIN..=M_MAX` (⚠️ §914: hasta el §914, `3..=M_MAX`).
///
/// Por que hace falta: la traza oculta mide `2 * 8 * 2^m` filas y el envoltorio de ocultacion
/// declara `1 + 8 * 2^m` exenciones de transicion, asi que el juez paga un coste lineal en `2^m`
/// antes de la primera comprobacion contra la prueba. `m` sale de `nextPending` de la cabeza, y
/// el mando no fija la clave del operador: una cabeza con firma propia y una prueba que solo tiene
/// la forma bastan para elegirla. El fork evalua ya las exenciones en flujo, sin lista (CPU sin
/// memoria); lo que acota la CPU es este tope. El probador lo respeta tambien: no prueba lo que
/// el juez no aceptaria.
///
/// Por que 13: con la traza oculta de produccion, `m = 13` (`n = 8192`) es la mayor `m` cuya
/// prueba cabe en las dos cotas de la puerta del RFC-0007 (D-E4a): probar en un latido, 60 s, y con
/// un pico por debajo de la mitad de la RAM de la maquina de referencia, 6.124.348 kB. `m = 14` no
/// cabe: su pico, medido, pasa de esa cota un 2,15 %, y su tiempo, proyectado a esa maquina, es de
/// 1,27 a 1,46 latidos (§717). El techo del §463, `n = 16384`, se midio sin ocultacion. El juez y
/// el probador llevan el mismo tope: el juez no acepta lo que no se prueba dentro de las cotas, y
/// el probador no prueba lo que el juez no aceptaria. En el tope, rechazar una prueba fabricada
/// cuesta milisegundos (lo mide `tests_coste`).
///
/// ⚠️ Un libro cuyo `nextPending` pase de `2^M_MAX` no tiene prueba de edad, y `nextPending` no
/// baja: es la marca de agua de las posiciones usadas. `allocate_pending` reutiliza el hueco libre
/// mas bajo, y solo `commit_send` y `apply_mint_pending_delegated` la suben
/// (`zk-ssl/src/two_phase.rs`).
///
/// ⚠️ Cambiar el tope no es cambiar solo esta linea. Va con la de la segunda implementacion
/// (`tools/segunda/airs.py`, `M_MAX`), que un test de este crate compara. Y pinan el valor, a
/// proposito (§721), tres tests -`m_13_entra_al_juez_y_se_juzga_la_prueba` y
/// `m_14_cae_por_el_tope_en_milisegundos`, aqui, y `el_probador_rechaza_m_14`, en
/// `stark-experiment`- y dos vectores de `spec/vectors/edad/`: `neg-m-13-fabricada.json` llega a
/// la prueba y `neg-m-14-sobre-tope.json` cae por el tope, asi que juntos fijan el tope en 13. Un
/// tope nuevo pide vectores nuevos; los de hoy no se reescriben.
pub const M_MAX: u32 = 13;

/// §914 (RFC-0018 D-H, T = 128 en las cinco familias de T = 64): el suelo de `m` de la edad sube
/// de 3 a 4, porque la traza en claro mide `CICLO << m` y pide T >= 128 (⚠️ §914: era 3 desde la
/// D-AG del RFC-0009, con T >= 64).
pub const M_MIN: u32 = 4;

/// **§723, la ventana de la v1 de la banda y del cobro pendiente.** En la v1 los tres
/// segmentos de rango de [`banda`] y de [`cobro_pendiente`] eran de 63 bits. El valor que acotan
/// (el saldo, el importe) es PRIVADO, y la cota nativa de `comprobar_enunciado` solo acota los
/// limites publicos. Con `v` en `[0, 2^63)`, `(v - l) mod p` y `(u - v) mod p` en `[0, 2^63)`, un
/// `v > u` pasa si `p - (v - u) < 2^63`, es decir, si `v > u + p - 2^63`. Como `v < 2^63`, eso solo
/// cabe con `u < 2^64 - 1 - p`, que es este numero: `2^32 - 2`. Con `u` desde aqui, la v1 dice lo
/// que parece; por debajo, un `v` de la ventana `[2^62, 2^63)` tambien la pasa (medido: saldo
/// `2^63 - 2^32 + 2` con la banda `[0, 0]`). La v2 baja los segmentos a 62 bits y la cierra.
pub const COTA_V1: u64 = (1u64 << 32) - 2;
const _: () = assert!(COTA_V1 == u64::MAX - <BaseElement as winter_math::StarkField>::MODULUS);

/// El prefijo de la causa con la que [`banda::verificar`] y [`cobro_pendiente::verificar`] rehusan
/// una prueba de la v1 cuyo superior queda bajo [`COTA_V1`]. Quien la llama la distingue por el: el
/// mando del kit la cuenta como <<declarada, no probada>>, no como ROJO, porque una prueba honesta
/// de la v1 tambien la da.
pub const PRUEBA_V1_EN_LA_VENTANA: &str = "prueba de la v1 con segmentos de 63 bits";

/// La v1 vale solo donde su ventana esta vacia: con el superior en [`COTA_V1`] o por encima.
pub fn v1_sin_ventana(superior: u64) -> bool {
    superior >= COTA_V1
}

/// La causa que da el juez cuando una prueba no verifica ni con la v2 ni con la v1: la de la version
/// que llego mas lejos. Una prueba juzgada con la version que no es la suya cae en la coherencia OOD,
/// la primera comprobacion que mira las restricciones; con la suya pasa de ahi y cae por lo que de
/// verdad tiene (un nodo de mas, un resto que no es el suyo). Si una de las dos cae por la OOD y la
/// otra no, la causa es la de la otra; si no, la de la v2. Asi una prueba de la v1 ya emitida y
/// maleada sigue cayendo con su causa, la misma que con el kit de antes.
pub fn la_causa_de_la_que_llego_mas_lejos(
    v2: Result<(), String>,
    v1: Result<(), String>,
) -> Result<(), String> {
    const OOD: &str = "InconsistentOodConstraintEvaluations";
    match (v2, v1) {
        (Err(e2), Err(e1)) if e2 == OOD && e1 != OOD => Err(e1),
        (v2, _) => v2,
    }
}

/// Carril A: el arbol de pendientes (doce columnas del estado).
pub const C_A: usize = 0;
/// Carril B: el arbol de meta.
pub const C_B: usize = 12;
/// Carril M: la hoja de meta, `commit_operation(PMETA_V1, [emisor, nacido])`.
pub const C_M: usize = 24;
/// La hoja del arbol de pendientes de la posicion del ciclo (cuatro columnas).
pub const C_P: usize = 36;
pub const C_EMISOR: usize = 40;
pub const C_NACIDO: usize = 41;
/// 1 si la hoja de pendientes no es cero (una posicion viva).
pub const C_VIVO: usize = 42;
/// 1 si la edad alcanza `T`; 0 exige el resto en `BITS` bits.
pub const C_QA: usize = 43;
/// 1 si el emisor casa con el nombrado; 0 exige su inverso.
pub const C_QS: usize = 44;
pub const C_WINV: usize = 45;
/// `vivo * qa * qs`: lo que la posicion suma a la cuenta.
pub const C_Z: usize = 46;
pub const C_CUENTA: usize = 47;
/// El numero de ciclo `c`: la etiqueta del nodo y de la hoja.
pub const C_CICLO: usize = 48;
/// 0 en el ciclo 0 (no hay nodo 0), 1 despues.
pub const C_ACT: usize = 49;
/// 1 si `c >= n`: por encima de la marca todo es la hoja cero.
pub const C_FUERA: usize = 50;
pub const C_BITS: usize = 51;
/// Ancho de la traza principal.
pub const ANCHO: usize = C_BITS + BITS;

/// Traza auxiliar: cinco inversos y la suma acumulada.
pub const X_H: usize = 0;
pub const X_S: usize = 5;
pub const ANCHO_AUX: usize = 6;
/// Dos aleatorios: `alfa` (la combinacion) y `beta` (el desplazamiento).
pub const ALEATORIOS: usize = 2;

const CARRIL_A: u64 = 1;
const CARRIL_B: u64 = 2;

// Columnas periodicas (longitud CICLO).
const P_HASH: usize = 0;
const P_F0: usize = 1;
const P_F6: usize = 2;
const P_ARK1: usize = 3;
const P_ARK2: usize = P_ARK1 + ESTADO;

/// Restricciones de la traza principal y de la auxiliar, contadas por familia en `new`.
pub const PRINCIPALES: usize = 3 * ESTADO + 8 + ESTADO + 4 + 5 + 1 + 1 + 1 + (4 + BITS) + 2 + 1 + 3;
pub const AUXILIARES: usize = 5 + 3 + 1;

// Las celdas de la traza principal que ninguna restriccion mira, por diseno. El censo de celdas
// (doc/fv/interprete_edad.py) exige cada una aqui. Los testigos de cada posicion solo se leen en
// la fila 0 de su ciclo, que es la siguiente del enlace: alli los atan la hoja del pendiente
// (leida por el multiconjunto), la meta, la edad y el emisor. En las demas filas no los lee nadie.
// La booleanidad de qa, qs y los bits rige en todas las filas, pero no los determina: los
// referencia. Vivo y fuera, en cambio, no se declaran: su constancia dentro del ciclo es la que
// lleva el vivo de la fila 0 a la hoja de meta que se escribe en la fila 6.
// CELDAS_LIBRES: hoja del pendiente, emisor y nacido fuera de la fila 0 del ciclo (clase sin enlace, cols 36..42)
// CELDAS_LIBRES: qa y qs fuera de la fila 0 del ciclo: solo booleanos (clase sin enlace, cols 43..45)
// CELDAS_LIBRES: inverso del emisor y producto z fuera de la fila 0 del ciclo (clase sin enlace, cols 45..47)
// CELDAS_LIBRES: bits del resto fuera de la fila 0 del ciclo: solo booleanos (clase sin enlace, cols 51..83)

// ------------------------------------------------------------------ las opciones

/// §651 (D3-META): la `m` de la ocultacion de la casa, la de `ocultacion_encendida` en todos los
/// probadores con fila (D-Z, D-AE). Un solo productor: el probador la escribe en la marca y las
/// guardas de forma la exigen.
pub const M_OCULTACION: usize = 64;

/// §914 (el corte zkssl/0.5; RFC-0009 D-AL bis, RFC-0018 D-H): las columnas aleatorias de toda
/// prueba de la casa, DOS, pareadas como una sobre E en DEEP. Un solo productor: el probador las
/// escribe en la marca v2 y el juez unico exige la forma oculta con ellas.
pub const COLUMNAS_ALEATORIAS: usize = 2;

/// §912 (RFC-0018 D-G), en produccion desde el corte zkssl/0.5 (§914): el paso 1 del juez de la
/// marca v2, 1b a 1h, con sus textos -los mismos que `verify_v2` del fork y que `tools/segunda`,
/// letra por letra-: la marca antes que nada, sin construir ninguna AIR ni leer el resto de la
/// prueba. Devuelve m y las columnas aleatorias que declara. Lo llaman los cinco jueces del kit
/// ([`marca_del_kit`]), la capa y la pareja umbral por el juez unico ([`juez::comprobar_marca`]).
/// §651 (D3-META): la marca del meta es parte de la FORMA, y se mira ANTES de construir ninguna
/// AIR: con el meta vaciado, una prueba de forma oculta llegaba a `AIR::new` en claro, que entraba
/// en panico por el ancho. Hasta el §914 ese sitio era `comprobar_marca`, el lector de la marca v1
/// de zkssl/0.4, con el texto `marca de la ocultacion {lectura:?}; el enunciado pide m = 64`; una
/// prueba con esa marca cae ahora en 1c, y la juzga el kit 0.4.x.
pub fn comprobar_marca_v2(meta: &[u8], familia: u16) -> Result<(usize, usize), String> {
    use winter_air::{Marca, MarcaError, MarcaV2};
    let marca = match MarcaV2::leer(meta) {
        Ok(Some(marca)) => marca,
        Ok(None) => {
            return Err("marca: meta vacio, prueba anterior a zkssl/0.4; la juzga el kit de su version".to_string())
        },
        Err(MarcaError::Desconocida) if Marca::leer(meta).is_ok() => {
            return Err("marca v1: prueba de zkssl/0.4; la juzga el kit 0.4.x".to_string())
        },
        Err(_) => return Err("marca desconocida".to_string()),
    };
    if marca.familia != familia {
        return Err(format!("marca: la prueba es de la familia {}; se pide la {familia}", marca.familia));
    }
    if marca.version != familias::VERSION_INICIAL {
        return Err(format!(
            "marca: version {} de la AIR de la familia {familia}; se acepta {}",
            marca.version,
            familias::VERSION_INICIAL
        ));
    }
    if marca.perfil != familias::PERFIL_PRODUCCION {
        return Err(format!("marca: perfil {} desconocido", marca.perfil));
    }
    if marca.m != M_OCULTACION {
        return Err(format!("marca: m = {}; la construccion pide m = {M_OCULTACION}", marca.m));
    }
    // §914 (1h bis): las columnas que declara son las de la construccion. Sin esto, una marca con
    // otra cifra pasaba la forma -que exige la constante- y el fork construia la AIR interna con
    // un ancho de menos, que entraba en panico (el barrido byte a byte del kit lo vio).
    if marca.columnas as usize != COLUMNAS_ALEATORIAS {
        return Err(format!("marca: {} columnas; la construccion pide {COLUMNAS_ALEATORIAS}", marca.columnas));
    }
    Ok((marca.m, marca.columnas as usize))
}

/// §913: el texto con que los cinco jueces del kit dicen un [`juez::Rechazo`], los de siempre: la
/// forma con «pide», la marca con el texto de [`comprobar_marca_v2`] y el error del fork por su
/// `Debug`. Los catalogos de `spec/PAQUETE.md` §5 los citan tal cual. Siempre es `Err`: los jueces
/// lo encadenan con `or_else`.
pub fn rechazo_del_kit(r: juez::Rechazo) -> Result<(), String> {
    match r {
        juez::Rechazo::Cabeza => Err("la prueba no se deserializa: sin cabeza".to_string()),
        juez::Rechazo::Forma { real: forma, exige } => {
            Err(format!("forma de traza {forma:?}; el enunciado pide {:?}", exige))
        },
        juez::Rechazo::Marca(m) => Err(m),
        juez::Rechazo::Verificador(e) => Err(format!("{e:?}")),
    }
}

/// §914 (RFC-0018 D-G): el paso 1 del juez del kit, la marca ANTES que nada, leida de la cabeza de
/// la prueba sin deserializar el resto (1a) y juzgada con las causas 1b a 1h; con los textos del
/// kit. Los cinco jueces empiezan por aqui, antes del enunciado.
pub fn marca_del_kit<J: juez::Juez>(prueba: &[u8]) -> Result<(), String> {
    let meta = match juez::meta_de_la_cabeza(prueba) {
        Ok(meta) => meta,
        Err(r) => return rechazo_del_kit(r),
    };
    juez::comprobar_marca::<J>(meta).map(|_| ()).or_else(rechazo_del_kit)
}

/// §711: las causas con nombre de una prueba mal formada, del fork de winterfell, la unica fuente:
/// el kit, la capa y sus tests las nombran desde aqui.
pub use winter_air::causa;

/// §711: el texto con que empieza el error de [`sin_panico`] cuando recoge un panico. Un test que
/// exige que una prueba ajena se rechace SIN panico mira que su error no lo lleve: la red no
/// puede esconder lo que tiene que arreglar el verificador.
pub const PANICO_RECOGIDO: &str = "panico recogido al juzgar una prueba ajena";

/// **§711 · La red del panico de quien juzga una prueba ajena.** Corre `juzgar` y, si entra en
/// panico, devuelve `Err(de_panico(..))` con [`PANICO_RECOGIDO`] y el mensaje del panico, en vez
/// de dejarlo subir.
///
/// ⚠️ Es la defensa de ESTRUCTURA, no el arreglo: el verificador del fork rechaza sin panico lo
/// que se midio (§711), y esta red cubre lo que no se midio. Se pone ANTES de mutar nada: un
/// panico recogido a medio mutar dejaria un estado roto que parece sano. En el nodo, un panico
/// con el candado del estado tomado lo envenenaba y lo llevaba a PARADA (§530): recogido aqui,
/// el candado no llega a verlo.
///
/// Con `panic = "abort"` -WebAssembly sin excepciones- no recoge nada: el proceso aborta.
pub fn sin_panico<T, E>(
    juzgar: impl FnOnce() -> Result<T, E>,
    de_panico: impl FnOnce(String) -> E,
) -> Result<T, E> {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(juzgar)) {
        Ok(r) => r,
        Err(p) => {
            let msg = if let Some(t) = p.downcast_ref::<&str>() {
                (*t).to_string()
            } else if let Some(t) = p.downcast_ref::<String>() {
                t.clone()
            } else {
                "sin mensaje".to_string()
            };
            Err(de_panico(format!("{PANICO_RECOGIDO}: {msg}")))
        }
    }
}

/// **La huella del CONTENIDO de una prueba STARK**: la de sus compromisos -los de la traza, el
/// del cociente y los de FRI, con el del resto-, sin sus aperturas ni su codificacion. `None` si
/// la prueba no se lee. El resumen es [`zk_ssl_hash::huella_de_contenido`], con su dominio de bytes
/// propio en el registro de dominios.
///
/// §715 (la maleabilidad de los bytes de la prueba). La huella de una operacion es la de
/// los bytes de su prueba (`digest_of_proof`), y unos bytes distintos de la misma prueba -una
/// longitud escrita de mas, una fila o un grupo FRI de mas, un nodo de mas en un lote- verificaban
/// con otra huella: el reenvio tras un ciclo A->B->A volvia a debitar (medido en la capa). El
/// lector y el canal del fork rechazan hoy todas las formas medidas; esta huella es la defensa de
/// ESTRUCTURA para las que no se midieron. Los compromisos son lo que el probador manda ANTES de
/// las consultas: dos pruebas con los mismos compromisos son la misma prueba para quien la aplica,
/// y sin el testigo -la traza entera- nadie abre otros compromisos que verifiquen. La sal de cada
/// hoja (`MerkleConSal`) los hace distintos en cada prueba honesta, aunque el enunciado se repita.
///
/// Se lee dentro de [`sin_panico`]: la llama quien todavia no ha juzgado la prueba.
pub fn huella_del_contenido(prueba: &[u8]) -> Option<[u8; 32]> {
    let p = sin_panico(|| Proof::from_bytes(prueba).map_err(|_| ()), |_| ()).ok()?;
    let compromisos = winter_verifier::Serializable::to_bytes(&p.commitments);
    Some(zk_ssl_hash::digest_to_bytes(&zk_ssl_hash::huella_de_contenido(&compromisos)))
}

/// **Las opciones que el juez ACEPTA, y solo esas** (decision D-4). Son las `proof_options()` de
/// la capa; un test de la capa las ata (dos literales son dos productores si nadie los cruza).
pub fn opciones() -> ProofOptions {
    ProofOptions::new(
        42,
        16,
        21,
        FieldExtension::Quadratic,
        8,
        31,
        BatchingMethod::Linear,
        BatchingMethod::Linear,
    )
}

// ------------------------------------------------------------------ las entradas publicas

/// **Lo que el juez sabe.** Las dos subraices son las del subarbol `[0, 2^m)`; el juez las sube a
/// 32 niveles y las compara con `pendingRoot` y `pmetaRoot` de la cabeza v5 que firma `seq` y
/// `next_pending` (= `n`). `k` es la cota que la prueba demuestra: el probador solo puede contar DE
/// MAS, nunca esconder una posicion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EdadPublicInputs {
    pub subraiz_pend: Digest,
    pub subraiz_meta: Digest,
    pub m: u32,
    pub n: u64,
    pub seq: u64,
    pub t: u64,
    pub emisor: u64,
    pub todos: bool,
    pub k: u64,
}

/// ⚠️ **Cada entero entra por `BaseElement::new`, que reduce modulo `p`**: `x` y `x + p` dan el
/// mismo elemento, y el mismo transcript. Por eso nada llega aqui sin pasar antes por
/// [`comprobar_enunciado`], que acota cada uno muy por debajo de `p`: `n <= 2^m`, `seq` y `t` en
/// 32 bits, `k <= n` y el emisor, un indice de cuenta. §718, la canonicidad: el emisor
/// era el unico sin cota, y la prueba del emisor `X` verificaba tambien con `X + p`.
impl ToElements<BaseElement> for EdadPublicInputs {
    fn to_elements(&self) -> Vec<BaseElement> {
        let mut v = Vec::with_capacity(15);
        v.extend_from_slice(&self.subraiz_pend);
        v.extend_from_slice(&self.subraiz_meta);
        let resto = [
            self.m as u64,
            self.n,
            self.seq,
            self.t,
            self.emisor,
            self.todos as u64,
            self.k,
        ];
        for x in resto {
            v.push(BaseElement::new(x));
        }
        v
    }
}

/// Lo que el enunciado tiene que cumplir ANTES de construir el AIR: fuera de estos rangos la
/// aritmetica del campo daria la vuelta, o el AIR no tendria forma. Nunca entra en panico.
///
/// Desde el §718, cada entero del enunciado queda acotado aqui muy por debajo de `p`, y un
/// enunciado tiene una sola escritura. El emisor es un indice de cuenta, menor que `2^32`, y con
/// todos es 0.
///
/// Desde el §717, `m` va de [`M_MIN`] a [`M_MAX`] (antes, a 24; ⚠️ §914: el suelo sube de 3 a 4).
/// Por encima, el juez pagaria un coste lineal en `2^m` antes de mirar la prueba.
pub fn comprobar_enunciado(pi: &EdadPublicInputs) -> Result<(), String> {
    if pi.m < M_MIN || pi.m > M_MAX {
        return Err(format!("m = {} fuera de {M_MIN}..={M_MAX}", pi.m));
    }
    if pi.n > (1u64 << pi.m) {
        return Err(format!("n = {} no cabe en 2^{}", pi.n, pi.m));
    }
    if pi.t >= (1u64 << BITS) || pi.seq >= (1u64 << BITS) {
        return Err(format!("T = {} o seq = {} no caben en {BITS} bits", pi.t, pi.seq));
    }
    if pi.k > pi.n {
        return Err(format!("k = {} mayor que n = {}", pi.k, pi.n));
    }
    // §718, la canonicidad: el emisor es un indice de cuenta, menor que 2^32 (la
    // capacidad del arbol de cuentas). Sin la cota, `emisor + p` entraba al AIR como `emisor`
    // y el juez devolvia el enunciado con el alias.
    comprobar_emisor(pi.emisor)?;
    // Todos se codifica con emisor 0 (lo hace el probador); otro emisor con todos es otra
    // escritura del mismo enunciado.
    if pi.todos && pi.emisor != 0 {
        return Err(format!("todos con emisor = {}: todos se escribe con emisor 0", pi.emisor));
    }
    Ok(())
}

// ------------------------------------------------------------------ la codificacion

/// `[1, alfa, alfa^2, ..., alfa^5]`.
pub fn potencias<E: FieldElement>(alfa: E) -> [E; 6] {
    let mut p = [E::ONE; 6];
    for i in 1..6 {
        p[i] = p[i - 1] * alfa;
    }
    p
}

/// **El denominador de un termino del multiconjunto**: `beta - (etiqueta + sum alfa^i v_i +
/// alfa^5 carril)`. El probador y el AIR llaman a ESTA funcion: dos codificaciones serian dos
/// productores del mismo hecho.
pub fn codificar<E: FieldElement>(pot: &[E; 6], beta: E, etiqueta: E, v: [E; 4], carril: E) -> E {
    beta - (etiqueta + pot[1] * v[0] + pot[2] * v[1] + pot[3] * v[2] + pot[4] * v[3]
        + pot[5] * carril)
}

/// El carril como elemento, para el probador.
pub fn carril<E: FieldElement<BaseField = BaseElement>>(b: bool) -> E {
    E::from(BaseElement::new(if b { CARRIL_B } else { CARRIL_A }))
}

// ------------------------------------------------------------------ la ronda de Rescue

/// S-box de Rescue, `x^7` (el ALPHA de `winter-crypto`, `rp64_256/mod.rs:52`).
fn sbox<E: FieldElement>(x: E) -> E {
    let x2 = x * x;
    let x4 = x2 * x2;
    x4 * x2 * x
}

/// **La restriccion de una ronda de Rescue**, por encuentro en el medio: `sbox(B) = A` con
/// `A = MDS * sbox(actual) + ark1` y `B = INV_MDS * (siguiente - ark2)`. Es la misma que llevan
/// en linea los circuitos de `stark-experiment` (deuda declarada: veintiocho copias); aqui vive
/// UNA vez y los tres carriles la llaman.
pub fn ronda_rescue<E: FieldElement<BaseField = BaseElement>>(
    actual: &[E],
    siguiente: &[E],
    ark1: &[E],
    ark2: &[E],
    bandera: E,
    salida: &mut [E],
) {
    let mut s = [E::ZERO; ESTADO];
    for j in 0..ESTADO {
        s[j] = sbox(actual[j]);
    }
    for i in 0..ESTADO {
        let mut a = E::ZERO;
        let mut b = E::ZERO;
        for j in 0..ESTADO {
            a += E::from(Rp64_256::MDS[i][j]) * s[j];
            b += E::from(Rp64_256::INV_MDS[i][j]) * (siguiente[j] - ark2[j]);
        }
        salida[i] = bandera * (sbox(b) - (a + ark1[i]));
    }
}

// ------------------------------------------------------------------ el AIR

pub struct EdadAir {
    context: AirContext<BaseElement>,
    pi: EdadPublicInputs,
    hojas: usize,
}

impl EdadAir {
    fn num_aserciones(pi: &EdadPublicInputs, hojas: usize) -> usize {
        4 + usize::from(pi.n >= 1) + usize::from((pi.n as usize) < hojas) + 8
    }
}

impl Air for EdadAir {
    type BaseField = BaseElement;
    type PublicInputs = EdadPublicInputs;

    fn new(trace_info: TraceInfo, pub_inputs: Self::PublicInputs, options: ProofOptions) -> Self {
        let hojas = 1usize << pub_inputs.m;
        let c = |d: usize| TransitionConstraintDegree::with_cycles(d, vec![CICLO]);
        let mut p = Vec::with_capacity(PRINCIPALES);
        // 1. las rondas de los tres carriles
        for _ in 0..3 * ESTADO {
            p.push(c(7));
        }
        // 2. capacidad cero de A y de B en la fila 0 del ciclo
        for _ in 0..8 {
            p.push(c(1));
        }
        // 3. el estado inicial de la hoja de meta en la fila 0
        for _ in 0..ESTADO {
            p.push(c(1));
        }
        // 4. (1 - vivo) * P = 0: toda hoja distinta de cero se cuenta como viva
        for _ in 0..4 {
            p.push(c(2));
        }
        // 5. fuera * P = 0 y fuera * vivo = 0: nada vivo por encima de la marca
        for _ in 0..5 {
            p.push(c(2));
        }
        // 6. la edad: qa = 0 exige T - 1 - (seq - nacido) en BITS bits
        p.push(c(2));
        // 7. el emisor: qs = 0 exige emisor distinto del nombrado
        p.push(if pub_inputs.todos { c(1) } else { c(3) });
        // 8. z = vivo * qa * qs
        p.push(c(3));
        // 9. booleanos: vivo, qa, qs, fuera y los bits
        for _ in 0..(4 + BITS) {
            p.push(TransitionConstraintDegree::new(2));
        }
        // 10. vivo y fuera constantes dentro del ciclo
        for _ in 0..2 {
            p.push(c(1));
        }
        // 11. fuera no baja en el enlace
        p.push(c(2));
        // 12. el contador de ciclo, act y la cuenta
        p.push(TransitionConstraintDegree::new(1));
        p.push(c(1));
        p.push(c(1));
        debug_assert_eq!(p.len(), PRINCIPALES);

        let mut x = Vec::with_capacity(AUXILIARES);
        // cinco lecturas/escritura en la fila 0 y dos escrituras en la 6: h * cod = 1
        for _ in 0..5 {
            x.push(c(2));
        }
        x.push(c(2));
        x.push(c(2));
        // la hoja de meta escrita es vivo * salida de M: un grado mas
        x.push(c(3));
        // la suma acumulada
        x.push(c(2));
        debug_assert_eq!(x.len(), AUXILIARES);

        let aserciones = Self::num_aserciones(&pub_inputs, hojas);
        EdadAir {
            context: AirContext::new_multi_segment(trace_info, p, x, aserciones, 2, options),
            pi: pub_inputs,
            hojas,
        }
    }

    fn context(&self) -> &AirContext<Self::BaseField> {
        &self.context
    }

    /// Selector de hash (filas 0..6), selector de la fila 0, selector de la fila 6, y las
    /// constantes de ronda ARK1 y ARK2 de `Rp64_256`, todas de longitud CICLO.
    fn get_periodic_column_values(&self) -> Vec<Vec<Self::BaseField>> {
        let mut cols = Vec::with_capacity(3 + 2 * ESTADO);
        let fila = |k: usize| {
            let mut v = vec![BaseElement::ZERO; CICLO];
            v[k] = BaseElement::ONE;
            v
        };
        let mut hash = vec![BaseElement::ONE; RONDAS];
        hash.push(BaseElement::ZERO);
        cols.push(hash);
        cols.push(fila(0));
        cols.push(fila(6));
        for i in 0..ESTADO {
            let mut v: Vec<BaseElement> = (0..RONDAS).map(|r| Rp64_256::ARK1[r][i]).collect();
            v.push(BaseElement::ZERO);
            cols.push(v);
        }
        for i in 0..ESTADO {
            let mut v: Vec<BaseElement> = (0..RONDAS).map(|r| Rp64_256::ARK2[r][i]).collect();
            v.push(BaseElement::ZERO);
            cols.push(v);
        }
        cols
    }

    fn evaluate_transition<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        frame: &EvaluationFrame<E>,
        periodic_values: &[E],
        result: &mut [E],
    ) {
        let c = frame.current();
        let s = frame.next();
        let hf = periodic_values[P_HASH];
        let f0 = periodic_values[P_F0];
        let enlace = E::ONE - hf;
        let ark1 = &periodic_values[P_ARK1..P_ARK1 + ESTADO];
        let ark2 = &periodic_values[P_ARK2..P_ARK2 + ESTADO];
        let mut i = 0;

        for base in [C_A, C_B, C_M] {
            let (a, b) = (&c[base..base + ESTADO], &s[base..base + ESTADO]);
            ronda_rescue(a, b, ark1, ark2, hf, &mut result[i..i + ESTADO]);
            i += ESTADO;
        }
        for k in 0..4 {
            result[i] = f0 * c[C_A + k];
            result[i + 4] = f0 * c[C_B + k];
            i += 1;
        }
        i += 4;
        let dominio = E::from(BaseElement::new(DOMINIO_META_PENDIENTE));
        for k in 0..ESTADO {
            let esperado = match k {
                0 => dominio,
                4 => c[C_EMISOR],
                5 => c[C_NACIDO],
                _ => E::ZERO,
            };
            result[i] = f0 * (c[C_M + k] - esperado);
            i += 1;
        }
        let vivo = c[C_VIVO];
        let fuera = c[C_FUERA];
        for k in 0..4 {
            result[i] = f0 * (E::ONE - vivo) * c[C_P + k];
            result[i + 4] = f0 * fuera * c[C_P + k];
            i += 1;
        }
        i += 4;
        result[i] = f0 * fuera * vivo;
        i += 1;

        let dos = E::from(2u32);
        let mut resto = E::ZERO;
        let mut peso = E::ONE;
        for b in 0..BITS {
            resto += c[C_BITS + b] * peso;
            peso *= dos;
        }
        let t = E::from(BaseElement::new(self.pi.t));
        let seq = E::from(BaseElement::new(self.pi.seq));
        result[i] = f0 * (E::ONE - c[C_QA]) * (t - E::ONE - seq + c[C_NACIDO] - resto);
        i += 1;

        let qs = c[C_QS];
        result[i] = if self.pi.todos {
            f0 * (E::ONE - qs)
        } else {
            let nombrado = E::from(BaseElement::new(self.pi.emisor));
            f0 * (E::ONE - qs) * ((c[C_EMISOR] - nombrado) * c[C_WINV] - E::ONE)
        };
        i += 1;
        result[i] = f0 * (c[C_Z] - vivo * c[C_QA] * qs);
        i += 1;

        for col in [C_VIVO, C_QA, C_QS, C_FUERA] {
            result[i] = c[col] * (c[col] - E::ONE);
            i += 1;
        }
        for b in 0..BITS {
            let x = c[C_BITS + b];
            result[i] = x * (x - E::ONE);
            i += 1;
        }
        result[i] = hf * (s[C_VIVO] - vivo);
        result[i + 1] = hf * (s[C_FUERA] - fuera);
        result[i + 2] = enlace * fuera * (E::ONE - s[C_FUERA]);
        result[i + 3] = s[C_CICLO] - c[C_CICLO] - enlace;
        result[i + 4] = s[C_ACT] - c[C_ACT] - enlace * (E::ONE - c[C_ACT]);
        result[i + 5] = s[C_CUENTA] - c[C_CUENTA] - f0 * c[C_Z];
        debug_assert_eq!(i + 6, PRINCIPALES);
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        let ultima = self.trace_length() - 1;
        let n = self.pi.n as usize;
        let mut a = vec![
            Assertion::single(C_CICLO, 0, BaseElement::ZERO),
            Assertion::single(C_ACT, 0, BaseElement::ZERO),
            Assertion::single(C_CUENTA, 0, BaseElement::ZERO),
            Assertion::single(C_CUENTA, ultima, BaseElement::new(self.pi.k)),
        ];
        if n >= 1 {
            a.push(Assertion::single(C_FUERA, CICLO * (n - 1), BaseElement::ZERO));
        }
        if n < self.hojas {
            a.push(Assertion::single(C_FUERA, CICLO * n, BaseElement::ONE));
        }
        // el nodo 1 sale del ciclo 1, en su ultima fila
        let raiz = 2 * CICLO - 1;
        for k in 0..4 {
            a.push(Assertion::single(C_A + 4 + k, raiz, self.pi.subraiz_pend[k]));
            a.push(Assertion::single(C_B + 4 + k, raiz, self.pi.subraiz_meta[k]));
        }
        a
    }

    fn evaluate_aux_transition<F, E>(
        &self,
        main_frame: &EvaluationFrame<F>,
        aux_frame: &EvaluationFrame<E>,
        periodic_values: &[F],
        aux_rand_elements: &AuxRandElements<E>,
        result: &mut [E],
    ) where
        F: FieldElement<BaseField = Self::BaseField>,
        E: FieldElement<BaseField = Self::BaseField> + ExtensionOf<F>,
    {
        let c = main_frame.current();
        let s = main_frame.next();
        let x = aux_frame.current();
        let y = aux_frame.next();
        let f0 = E::from(periodic_values[P_F0]);
        let f6 = E::from(periodic_values[P_F6]);
        let r = aux_rand_elements.rand_elements();
        let pot = potencias(r[0]);
        let beta = r[1];
        let dig = |fila: &[F], col: usize| -> [E; 4] {
            [
                E::from(fila[col]),
                E::from(fila[col + 1]),
                E::from(fila[col + 2]),
                E::from(fila[col + 3]),
            ]
        };
        let hojas = E::from(BaseElement::new(self.hojas as u64));
        let ciclo = E::from(c[C_CICLO]);
        let izq = E::from(2u32) * ciclo;
        let der = izq + E::ONE;
        let (ca, cb) = (carril::<E>(false), carril::<E>(true));
        let act = E::from(c[C_ACT]);
        let vivo = E::from(c[C_VIVO]);

        let lecturas = [
            codificar(&pot, beta, izq, dig(c, C_A + 4), ca),
            codificar(&pot, beta, der, dig(c, C_A + 8), ca),
            codificar(&pot, beta, izq, dig(c, C_B + 4), cb),
            codificar(&pot, beta, der, dig(c, C_B + 8), cb),
            codificar(&pot, beta, hojas + ciclo, dig(c, C_P), ca),
        ];
        let m = dig(s, C_M + 4);
        let hoja_meta = [vivo * m[0], vivo * m[1], vivo * m[2], vivo * m[3]];
        let escrituras = [
            codificar(&pot, beta, ciclo, dig(s, C_A + 4), ca),
            codificar(&pot, beta, ciclo, dig(s, C_B + 4), cb),
            codificar(&pot, beta, hojas + ciclo, hoja_meta, cb),
        ];
        for k in 0..5 {
            result[k] = f0 * (x[X_H + k] * lecturas[k] - E::ONE);
        }
        for k in 0..3 {
            result[5 + k] = f6 * (x[X_H + k] * escrituras[k] - E::ONE);
        }
        let leidas = x[X_H] + x[X_H + 1] + x[X_H + 2] + x[X_H + 3];
        let escritas = act * (x[X_H] + x[X_H + 1]) + x[X_H + 2];
        result[8] = y[X_S] - x[X_S] - f0 * (x[X_H + 4] - act * leidas) - f6 * escritas;
    }

    fn get_aux_assertions<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        aux_rand_elements: &AuxRandElements<E>,
    ) -> Vec<Assertion<E>> {
        let r = aux_rand_elements.rand_elements();
        let pot = potencias(r[0]);
        let e = |d: Digest| [E::from(d[0]), E::from(d[1]), E::from(d[2]), E::from(d[3])];
        let a = codificar(&pot, r[1], E::ONE, e(self.pi.subraiz_pend), carril::<E>(false));
        let b = codificar(&pot, r[1], E::ONE, e(self.pi.subraiz_meta), carril::<E>(true));
        vec![
            Assertion::single(X_S, 0, E::ZERO),
            Assertion::single(X_S, self.trace_length() - 1, a.inv() + b.inv()),
        ]
    }
}

/// §913: la forma en claro de la edad, para el juez unico: la longitud depende de `m`.
impl juez::Juez for EdadAir {
    const NOMBRE: &'static str = "EdadAir";
    fn forma(pi: &EdadPublicInputs) -> juez::Forma {
        juez::Forma { ancho: ANCHO, ancho_aux: ANCHO_AUX, aleatorios: ALEATORIOS, longitud: CICLO << pi.m }
    }
}

// ------------------------------------------------------------------ el juez

/// **Verifica una prueba de edad contra su enunciado.** Comprueba ANTES la forma de la traza que
/// la prueba declara -anchos, aleatorios y longitud `8 * 2^m`- para que un fichero ajeno no llegue
/// a construir el AIR, y acepta SOLO [`opciones`]. Nunca entra en panico por lo que traiga la
/// prueba.
pub fn verificar(prueba: &[u8], pi: &EdadPublicInputs) -> Result<(), String> {
    // §914 (RFC-0018 D-G): la marca v2 antes que nada; despues el enunciado; despues la prueba
    marca_del_kit::<EdadAir>(prueba)?;
    comprobar_enunciado(pi)?;
    let proof =
        Proof::from_bytes(prueba).map_err(|e| format!("la prueba no se deserializa: {e:?}"))?;
    // §913: la forma oculta -ancho + 2, el tramo auxiliar tal cual y 2 * CICLO * 2^m filas-, las
    // opciones y la verificacion con la politica del juez, por el juez unico
    juez::juzgar::<EdadAir>(proof, pi.clone()).or_else(rechazo_del_kit)
}

// ------------------------------------------------------------------ las raices nativas

/// Los subarboles vacios: `vacio[0]` es la hoja cero y `vacio[k] = merge(vacio[k-1], vacio[k-1])`,
/// la misma convencion que `SparseTree` de la capa.
pub fn vacios(profundidad: usize) -> Vec<Digest> {
    let mut v = vec![[BaseElement::ZERO; 4]];
    for k in 1..=profundidad {
        v.push(native_merge(v[k - 1], v[k - 1]));
    }
    v
}

/// La raiz de un subarbol completo de `hojas.len()` hojas (una potencia de dos, al menos 2).
pub fn subraiz(hojas: &[Digest]) -> Digest {
    assert!(hojas.len() >= 2 && hojas.len().is_power_of_two(), "hojas: potencia de dos >= 2");
    let mut nivel = hojas.to_vec();
    while nivel.len() > 1 {
        nivel = nivel.chunks(2).map(|p| native_merge(p[0], p[1])).collect();
    }
    nivel[0]
}

/// **La subida del juez**: de la subraiz de `[0, 2^m)` a la raiz de `profundidad` niveles, con el
/// subarbol vacio de cada nivel como hermano derecho. Una posicion viva en `[2^m, 2^profundidad)`
/// hace que esta raiz no sea la firmada.
pub fn raiz_desde_subraiz(sub: Digest, m: u32, profundidad: usize) -> Digest {
    let v = vacios(profundidad);
    let mut acc = sub;
    for k in (m as usize)..profundidad {
        acc = native_merge(acc, v[k]);
    }
    acc
}

// ------------------------------------------------------------------ contra la cabeza (E4b-2)

/// **Lo que una cabeza v5 firma y la prueba de edad necesita** (RFC-0007 E4b-2, S465). Se lee de
/// una cabeza ya verificada: este crate no verifica firmas, las verifica el kit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CabezaEdad {
    pub seq: u64,
    pub pending_root: Digest,
    pub pmeta_root: Digest,
    pub next_pending: u64,
}

/// **Lo que el sobre afirma**: la edad `t`, la cota `k`, el emisor nombrado (`None` = todos: el
/// indice 0 es una cuenta, asi que no puede ser el centinela) y las dos subraices que la prueba
/// declara. Todos se codifica en el enunciado con `emisor = 0`, como lo hace el probador.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Afirmacion {
    pub t: u64,
    pub k: u64,
    pub emisor: Option<u64>,
    pub subraiz_pend: Digest,
    pub subraiz_meta: Digest,
}

/// **La `m` de una marca**: el menor `m >= M_MIN` con `n <= 2^m`, la que usa el probador
/// (`next_power_of_two`, al menos `1 << M_MIN`). Una marca, un subarbol: un enunciado tiene una
/// sola forma (decision D-2 de E4b-2; el suelo sube de 1 a 3 en D-AG del RFC-0009, con T >= 64, y
/// de 3 a 4 en el §914, con T >= 128). No desborda: para `n > 2^63` da 64, que el enunciado rechaza.
pub fn m_canonico(n: u64) -> u32 {
    if n <= (1u64 << M_MIN) {
        M_MIN
    } else {
        64 - (n - 1).leading_zeros()
    }
}

/// **La prueba de edad contra una cabeza v5** (RFC-0007 E4b-2, decision D-1). Compone el enunciado
/// con lo que la cabeza firma (`seq`, `n` = `nextPending`, `m` derivado) y lo que el sobre afirma;
/// sube las dos subraices a 32 niveles y exige que sean `pendingRoot` y `pmetaRoot`; y solo
/// entonces llama al juez. Devuelve el enunciado verificado. Una prueba de OTRA marca, de otro
/// `seq` o con un subarbol de mas no se enlaza: la prueba es la de SU enunciado.
pub fn verificar_contra_cabeza(
    prueba: &[u8],
    af: &Afirmacion,
    cabeza: &CabezaEdad,
) -> Result<EdadPublicInputs, String> {
    // §914 (RFC-0018 D-G): la marca antes que nada, tambien antes de componer con la cabeza
    marca_del_kit::<EdadAir>(prueba)?;
    let m = m_canonico(cabeza.next_pending);
    let pi = EdadPublicInputs {
        subraiz_pend: af.subraiz_pend,
        subraiz_meta: af.subraiz_meta,
        m,
        n: cabeza.next_pending,
        seq: cabeza.seq,
        t: af.t,
        emisor: af.emisor.unwrap_or(0),
        todos: af.emisor.is_none(),
        k: af.k,
    };
    comprobar_enunciado(&pi)?;
    if raiz_desde_subraiz(af.subraiz_pend, m, PROFUNDIDAD) != cabeza.pending_root {
        let q = "la subraiz de pendientes, subida a 32 niveles, no es el pendingRoot de la cabeza";
        return Err(q.into());
    }
    if raiz_desde_subraiz(af.subraiz_meta, m, PROFUNDIDAD) != cabeza.pmeta_root {
        let q = "la subraiz de meta, subida a 32 niveles, no es el pmetaRoot de la cabeza";
        return Err(q.into());
    }
    verificar(prueba, &pi)?;
    Ok(pi)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(k: u64) -> Digest {
        [
            BaseElement::new(k),
            BaseElement::new(3 * k + 1),
            BaseElement::new(7 * k + 2),
            BaseElement::new(11 * k + 3),
        ]
    }

    fn enunciado() -> EdadPublicInputs {
        EdadPublicInputs {
            subraiz_pend: d(1),
            subraiz_meta: d(2),
            m: 4,
            n: 13,
            seq: 100,
            t: 10,
            emisor: 7,
            todos: false,
            k: 3,
        }
    }

    /// Cada ranura declarada se escribe: con un centinela en `result`, ninguna sobrevive a una
    /// evaluacion sobre un marco cualquiera, y el numero de ranuras es el que el contexto cuenta.
    #[test]
    fn las_restricciones_declaradas_son_las_escritas() {
        let pi = enunciado();
        let filas = CICLO << pi.m;
        let info = TraceInfo::new_multi_segment(ANCHO, ANCHO_AUX, ALEATORIOS, filas, vec![]);
        let air = EdadAir::new(info, pi, opciones());
        assert_eq!(air.context().num_main_transition_constraints(), PRINCIPALES);
        assert_eq!(air.context().num_aux_transition_constraints(), AUXILIARES);
        let centinela = BaseElement::new(0xDEAD_BEEF);
        let fila = |k: u64| -> Vec<BaseElement> {
            (0..ANCHO as u64).map(|i| BaseElement::new(k * 1000 + i + 5)).collect()
        };
        let marco = EvaluationFrame::from_rows(fila(1), fila(2));
        let periodicas: Vec<BaseElement> =
            (0..3 + 2 * ESTADO as u64).map(|i| BaseElement::new(i + 2)).collect();
        let mut r = vec![centinela; PRINCIPALES];
        air.evaluate_transition(&marco, &periodicas, &mut r);
        assert!(r.iter().all(|x| *x != centinela), "una ranura principal sin escribir");
        let aux_fila = |k: u64| -> Vec<BaseElement> {
            (0..ANCHO_AUX as u64).map(|i| BaseElement::new(k * 77 + i + 9)).collect()
        };
        let aux = EvaluationFrame::from_rows(aux_fila(3), aux_fila(4));
        let al = AuxRandElements::new(vec![BaseElement::new(31), BaseElement::new(37)]);
        let mut x = vec![centinela; AUXILIARES];
        air.evaluate_aux_transition(&marco, &aux, &periodicas, &al, &mut x);
        assert!(x.iter().all(|v| *v != centinela), "una ranura auxiliar sin escribir");
    }

    /// La subida del juez da la raiz del arbol entero: un arbol de 16 hojas con las 4 primeras
    /// ocupadas, recompuesto hoja a hoja, contra la subraiz de las 4 subida a 4 niveles.
    #[test]
    fn la_raiz_desde_la_subraiz_es_la_del_arbol_entero() {
        let cuatro = [d(10), d(11), [BaseElement::ZERO; 4], d(13)];
        let mut dieciseis = vec![[BaseElement::ZERO; 4]; 16];
        dieciseis[..4].copy_from_slice(&cuatro);
        assert_eq!(raiz_desde_subraiz(subraiz(&cuatro), 2, 4), subraiz(&dieciseis));
        dieciseis[9] = d(99);
        assert_ne!(
            raiz_desde_subraiz(subraiz(&cuatro), 2, 4),
            subraiz(&dieciseis),
            "una viva fuera del subarbol paso desapercibida"
        );
    }

    /// Un enunciado fuera de rango y una prueba que no es prueba se rechazan con `Err`, sin
    /// construir el AIR ni entrar en panico.
    #[test]
    fn lo_que_no_tiene_forma_se_rechaza_sin_panico() {
        let bien = enunciado();
        for malo in [
            EdadPublicInputs { m: 0, ..bien.clone() },
            EdadPublicInputs { m: 25, ..bien.clone() },
            EdadPublicInputs { n: 17, ..bien.clone() },
            EdadPublicInputs { t: 1 << 32, ..bien.clone() },
            EdadPublicInputs { seq: 1 << 32, ..bien.clone() },
            EdadPublicInputs { k: 14, ..bien.clone() },
        ] {
            assert!(comprobar_enunciado(&malo).is_err(), "aceptado: {malo:?}");
        }
        assert!(comprobar_enunciado(&bien).is_ok());
        assert!(verificar(&[1, 2, 3], &bien).is_err(), "tres bytes verificaron");
    }

    /// §711: la red recoge el panico y lo devuelve como `Err` con [`PANICO_RECOGIDO`] y su
    /// mensaje; lo que no entra en panico pasa tal cual, `Ok` y `Err`. Falsador: sin el
    /// `catch_unwind`, el tercer caso hace entrar en panico al test.
    #[test]
    fn la_red_recoge_el_panico_y_deja_pasar_lo_demas() {
        let de = |m: String| m;
        assert_eq!(sin_panico(|| Ok::<u8, String>(7), de), Ok(7));
        assert_eq!(sin_panico(|| Err::<u8, String>("rojo".into()), de), Err("rojo".into()));
        let r = sin_panico(|| -> Result<u8, String> { panic!("panico inyectado {}", 3) }, de);
        assert_eq!(r, Err(format!("{PANICO_RECOGIDO}: panico inyectado 3")));
        let r = sin_panico(|| -> Result<u8, String> { panic!("literal") }, de);
        assert_eq!(r, Err(format!("{PANICO_RECOGIDO}: literal")));
    }

    /// **D-2 de E4b-2 (S465):** la `m` de una marca es la minima, la del probador
    /// (`next_power_of_two`, al menos 8 desde D-AG), y no desborda en los extremos.
    #[test]
    fn la_m_de_una_marca_es_la_minima() {
        // §914: el suelo sube de 3 a 4 (M_MIN), con T = 128
        let tabla = [
            (0, 4),
            (1, 4),
            (2, 4),
            (3, 4),
            (5, 4),
            (8, 4),
            (9, 4),
            (13, 4),
            (16, 4),
            (17, 5),
            (1 << 24, 24),
            ((1 << 24) + 1, 25),
            (u64::MAX, 64),
        ];
        for (n, m) in tabla {
            assert_eq!(m_canonico(n), m, "n = {n}");
        }
        for n in 0..300u64 {
            let del_probador = (n as usize).next_power_of_two().max(1 << M_MIN) as u64;
            assert_eq!(1u64 << m_canonico(n), del_probador, "n = {n}");
        }
    }

    /// Las quince entradas publicas van al transcripto, cada una en su sitio.
    #[test]
    fn las_entradas_publicas_van_enteras_al_transcripto() {
        let pi = enunciado();
        let v = pi.to_elements();
        assert_eq!(v.len(), 15);
        assert_eq!(&v[0..4], &pi.subraiz_pend);
        assert_eq!(&v[4..8], &pi.subraiz_meta);
        let resto: Vec<u64> = v[8..].iter().map(|x| x.as_int()).collect();
        assert_eq!(resto, vec![4, 13, 100, 10, 7, 0, 3]);
    }
}

/// **§578: toda verificacion del kit lee con SAL** -el compromiso cuyo lote acota `sal.rs` (§575)-:
/// las cinco que el mando alcanza, una por sobre con prueba (edad, banda, prenda, pago en curso y
/// cobro pendiente). Si alguien anade una sin sal, o cambia el compromiso de una, cae.
#[cfg(test)]
mod tests_sal {
    /// Las lineas de CODIGO -no de comentario- que llaman a `verify` con sus tipos. La aguja se
    /// arma por partes para que este mismo fuente no la contenga.
    fn llamadas(fuente: &str) -> Vec<&str> {
        // §914: el juez llama a `verify_v2`, el verificador de la marca v2 del fork
        let agujas = [concat!("verify", "::<"), concat!("verify_v2", "::<")];
        fuente
            .lines()
            .filter(|l| !l.trim_start().starts_with("//") && agujas.iter().any(|a| l.contains(a)))
            .collect()
    }

    /// Cada modulo, con sus llamadas contadas y todas con sal. Devuelve cuantas hubo.
    fn todas_con_sal(modulos: &[(&str, &str, usize)]) -> usize {
        let mut total = 0;
        for (nombre, fuente, esperadas) in modulos {
            let l = llamadas(fuente);
            assert_eq!(l.len(), *esperadas, "{nombre}: {l:?}");
            for x in &l {
                assert!(x.contains("MerkleConSal<Blake3>>("), "{nombre} verifica SIN sal: {x}");
            }
            total += l.len();
        }
        total
    }

    /// §913 (ARQ-01, la 117): ningun `verify` vivo en los cinco jueces del kit; el unico es el del
    /// juez unico, `juez.rs`, y lee con sal. Y cada juez del kit llama al juez unico.
    #[test]
    fn toda_verificacion_del_kit_pasa_por_el_juez() {
        let total = todas_con_sal(&[
            ("lib.rs", include_str!("lib.rs"), 0),
            ("banda.rs", include_str!("banda.rs"), 0),
            ("prenda.rs", include_str!("prenda.rs"), 0),
            ("pago_en_curso.rs", include_str!("pago_en_curso.rs"), 0),
            ("cobro_pendiente.rs", include_str!("cobro_pendiente.rs"), 0),
            ("juez.rs", include_str!("juez.rs"), 1),
        ]);
        assert_eq!(total, 1, "la unica llamada viva a verify es la del juez");
        let juzga = concat!("juzgar", "::<");
        for (nombre, fuente, esperadas) in [
            ("lib.rs", include_str!("lib.rs"), 1),
            ("banda.rs", include_str!("banda.rs"), 2),
            ("prenda.rs", include_str!("prenda.rs"), 1),
            ("pago_en_curso.rs", include_str!("pago_en_curso.rs"), 1),
            ("cobro_pendiente.rs", include_str!("cobro_pendiente.rs"), 2),
        ] {
            let n = fuente.lines().filter(|l| !l.trim_start().starts_with("//") && l.contains(juzga)).count();
            assert_eq!(n, esperadas, "{nombre}: llamadas al juez unico");
        }
    }
}

/// **§697: el piso de la LDR de la edad, para cada `m` que el enunciado admite** (ZK-1, paso (1),
/// de `doc/blueprint-v2.md`). Las familias de longitud fija tienen su test en la capa
/// (`crates/zk-ssl/src/nivel_de_produccion.rs`), sobre pruebas ocultas reales; la edad no puede
/// tenerlo asi, porque su longitud depende de `m`: la traza oculta tiene `2 * CICLO * 2^m` filas, la
/// forma que [`verificar`] exige. El test recorre la funcion real -`Proof::proven_security` y
/// `conjectured_security` sobre el `Context` de esa forma con [`opciones`], como sobre el de una
/// prueba- sin generar ninguna, y exige para cada `m` conjeturada >= 127, UDR = 59 y LDR >= su piso.
///
/// **Los pisos son los valores de hoy, no un objetivo.** La LDR baja dos bits cada vez que la
/// longitud se dobla, y con `m >= 8` queda por debajo de los 80 que la capa exige a las familias de
/// longitud fija. Subirla -un tope de `m`, u otras opciones para la edad- lo decide el autor, con el
/// nivel declarado de produccion (BACKLOG 116). Mientras tanto, el test falla si alguna `m` admitida
/// baja de su piso, si el enunciado admite una `m` sin piso declarado o si un piso nombra una `m`
/// que el enunciado ya no admite.
///
/// **§708**: el segundo test fija, para cada `m`, el tope que el termino DEEP pone a la
/// conjeturada, la cifra que `SECURITY.md` §3.11 publica junto a los 127.
///
/// **§717**: el enunciado admite `m` de 3 a [`M_MAX`], 13. Las filas de 14 a 23 de las dos tablas
/// salen con el, porque cada test exige que lo que declara sea una `m` admitida; sus cifras
/// siguen en `SECURITY.md` §3.11 como medida de la funcion.
#[cfg(test)]
mod tests_nivel {
    use super::*;
    use winter_air::proof::Context;
    use winter_crypto::Hasher;

    const CONJETURADA_MINIMA: u32 = 127;
    /// La UDR de hoy, fijada como valor, la misma que en la capa: solo cambia con un asiento.
    const UDR_ACTUAL: u32 = 59;

    /// El piso declarado de la LDR, en bits, para cada `m` (§697): lo que la funcion da hoy.
    const PISO_LDR: [(u32, u32); 10] = [
        (4, 86),
        (5, 84),
        (6, 82),
        (7, 80),
        (8, 78),
        (9, 76),
        (10, 74),
        (11, 72),
        (12, 70),
        (13, 68),
    ];

    /// Las `m` que el enunciado admite y que no llevan piso. Hasta el §717 era `m = 24`, cuya
    /// prueba no se podia generar; desde el §717 el enunciado admite `m` de 3 a [`M_MAX`], 13, y
    /// ninguna queda sin piso. La lista se queda: el dia que el rango cambie, el test obliga a
    /// tocarla.
    const SIN_PISO: [u32; 0] = [];

    /// Las `m` que [`comprobar_enunciado`] admite, con el resto del enunciado en su valor mas
    /// pequeno: el rango sale del juez, no se teclea.
    fn admitidas() -> Vec<u32> {
        (0..=40u32)
            .filter(|&m| {
                let pi = EdadPublicInputs {
                    subraiz_pend: [BaseElement::ZERO; 4],
                    subraiz_meta: [BaseElement::ZERO; 4],
                    m,
                    n: 0,
                    seq: 0,
                    t: 0,
                    emisor: 0,
                    todos: true,
                    k: 0,
                };
                comprobar_enunciado(&pi).is_ok()
            })
            .collect()
    }

    #[test]
    fn el_piso_de_la_edad_para_cada_m() {
        let o = opciones();
        assert_eq!(
            (o.constraint_batching_method(), o.deep_poly_batching_method()),
            (BatchingMethod::Linear, BatchingMethod::Linear),
            "con otro batching el nivel depende del ancho y de las restricciones: el Context de \
             este test tendria que llevar los del AIR"
        );
        let admitidas = admitidas();
        let mut mal = Vec::new();
        for (m, _) in PISO_LDR {
            if !admitidas.contains(&m) {
                mal.push(format!("m = {m} tiene piso y el enunciado ya no la admite"));
            }
        }
        for m in SIN_PISO {
            if !admitidas.contains(&m) {
                mal.push(format!("m = {m} se declara sin piso y el enunciado ya no la admite"));
            }
        }
        let mut medidas = Vec::new();
        for &m in &admitidas {
            if SIN_PISO.contains(&m) {
                continue;
            }
            let Some(&(_, piso)) = PISO_LDR.iter().find(|(x, _)| *x == m) else {
                mal.push(format!("m = {m}: el enunciado la admite y no tiene piso declarado"));
                continue;
            };
            let filas = 2 * (CICLO << m);
            let info = TraceInfo::new_multi_segment(ANCHO + 1, ANCHO_AUX, ALEATORIOS, filas, vec![]);
            // Con batching lineal el numero de restricciones no entra en la cifra; `Context::new`
            // solo pide que sea positivo.
            let mut p = Proof::new_dummy();
            // Desde el §711 `Context::new` devuelve `Result`: este contexto es valido.
            p.context = Context::new::<BaseElement>(info, o.clone(), PRINCIPALES + AUXILIARES)
                .expect("un contexto valido");
            let conjeturada = p.conjectured_security::<Blake3>().bits();
            let demostrable = p.proven_security::<Blake3>();
            let (ldr, udr) = (demostrable.ldr_bits(), demostrable.udr_bits());
            medidas.push(format!("m={m}: {ldr}"));
            if conjeturada < CONJETURADA_MINIMA {
                mal.push(format!("m = {m}: conjeturada {conjeturada} < {CONJETURADA_MINIMA}"));
            }
            if udr != UDR_ACTUAL {
                mal.push(format!("m = {m}: UDR {udr}, y la fijada es {UDR_ACTUAL}"));
            }
            if ldr < piso {
                mal.push(format!("m = {m}: LDR {ldr} por debajo de su piso, {piso}"));
            }
        }
        println!("[§697] LDR de la edad, traza oculta de 2^(m+4) filas: {}", medidas.join(", "));
        assert!(medidas.len() >= PISO_LDR.len(), "solo {} m medidas", medidas.len());
        assert!(mal.is_empty(), "el piso de la edad: {}", mal.join("; "));
    }

    /// La conjeturada con el termino DEEP (§708), en decimas de bit, para cada `m`: el tope que
    /// ese termino pone a la conjeturada, la cifra que `SECURITY.md` §3.11 publica junto a los 127
    /// que devuelve la funcion. Las familias de longitud fija tienen las longitudes de `m` = 3, 5,
    /// 6 y 7: 128, 512, 1024 y 2048 filas de traza oculta.
    const CONJETURADA_CON_DEEP: [(u32, u32); 10] = [
        (4, 1158),
        (5, 1148),
        (6, 1138),
        (7, 1128),
        (8, 1118),
        (9, 1108),
        (10, 1098),
        (11, 1088),
        (12, 1078),
        (13, 1068),
    ];

    /// El termino DEEP de la demostrable en decodificacion unica, en bits: la expresion de
    /// `proven_security_protocol_unique_decoding` (`crates/winter-air/src/proof/security.rs`), que
    /// el fork no exporta. El grado de las restricciones lo acota el blowup, como alli, y las dos
    /// aperturas dan `L + 1`. Es un error de grado sobre el tamano del campo, no de proximidad.
    fn bits_deep(o: &ProofOptions, bits_base: u32, filas: usize) -> f64 {
        let campo = (bits_base * o.field_extension().degree()) as f64;
        let grado = o.blowup_factor() as f64 + 1.0;
        let l = filas as f64;
        -(grado * (l + 2.0 - 1.0) + (l - 1.0)).log2() + campo
    }

    /// El termino de las consultas en la formula de la conjeturada (`ConjecturedSecurity::compute`,
    /// en el mismo fichero): consultas * log2(blowup), y la molienda solo si dan 80 bits o mas.
    fn bits_consultas(o: &ProofOptions) -> f64 {
        let mut consultas = (o.blowup_factor().ilog2() * o.num_queries() as u32) as f64;
        if consultas >= 80.0 {
            consultas += o.grinding_factor() as f64;
        }
        consultas
    }

    /// La formula de la conjeturada con `campo` como su termino de campo:
    /// min(campo, consultas) - 1, con el tope del hash. Con el campo entero es la de `winter-air`.
    fn conjeturada_con(o: &ProofOptions, campo: f64, hash: u32) -> f64 {
        (campo.min(bits_consultas(o)) - 1.0).min(hash as f64)
    }

    /// **§708: la conjeturada no descuenta el termino DEEP; con el, para cada `m`.** La
    /// conjeturada de `winter-air` pone como termino de campo el campo entero, 128 bits con la
    /// extension cuadratica, y ningun termino que crezca con la longitud `L` de la traza oculta. La
    /// demostrable cuenta uno de campo que no es de proximidad, de lo que trata la conjetura, sino
    /// de grado: el del punto fuera del dominio, ((blowup + 1) * (L + 1) + L - 1) / |E| en
    /// decodificacion unica. Con las opciones de produccion es el menor de los terminos de la
    /// formula -las consultas dan 189 bits y el hash pone 128-, y la conjeturada no pasa de el. El
    /// test fija, en decimas y para cada `m` que el enunciado admite con piso, ese tope: el termino
    /// mismo, sin el -1 con que la formula resta un bit a su minimo; y exige que con ese -1 la
    /// formula de un bit menos.
    ///
    /// Antes contrasta las replicas con la funcion real. La formula, con el campo entero, da la
    /// conjeturada de la funcion. El termino DEEP da la UDR de la funcion, que es su parte entera,
    /// con dos juegos de opciones en los que el DEEP la limita: los de produccion con plegado 2,
    /// 255 consultas y molienda 32, y esos mismos con blowup 2. Con blowup 16 el DEEP queda solo
    /// 0,17-0,18 bits por debajo de las capas de FRI, y la parte entera no separa los dos
    /// terminos; con blowup 2 queda un bit por debajo de las capas y del compromiso, y la parte
    /// entera si los separa. Un cambio de decimas en el fork lo ve la lectura de su fuente, no el
    /// contraste. El termino de ALI no se descuenta: con el batching lineal es 1/|E|, sin `L`.
    #[test]
    fn la_conjeturada_con_el_termino_deep_para_cada_m() {
        let o = opciones();
        assert_eq!(
            (o.constraint_batching_method(), o.deep_poly_batching_method()),
            (BatchingMethod::Linear, BatchingMethod::Linear),
            "con otro batching el ALI depende de las restricciones: el test tendria que contarlas"
        );
        // Los dos juegos del contraste: el blowup de produccion, y 2.
        let variantes = [o.blowup_factor(), 2].map(|blowup| {
            ProofOptions::new(
                255,
                blowup,
                32,
                o.field_extension(),
                2,
                31,
                BatchingMethod::Linear,
                BatchingMethod::Linear,
            )
        });
        let hash = <Blake3 as Hasher>::COLLISION_RESISTANCE;
        let admitidas = admitidas();
        let mut mal = Vec::new();
        // La replica de `bits_deep` es la expresion del fichero, letra a letra: el grado acotado
        // por el blowup y las dos aperturas, en los dos regimenes, y el termino de la UDR. El
        // contraste de abajo es entero; esto ve tambien un cambio de decimas en el fork.
        let fuente = include_str!("../../winter-air/src/proof/security.rs");
        for (linea, veces) in [
            ("let max_deg = options.blowup_factor() as f64 + 1.0;", 2),
            ("let num_openings = 2.0;", 2),
            (
                concat!(
                    "-log2(max_deg * (trace_domain_size + num_openings - 1.0) ",
                    "+ (trace_domain_size - 1.0))"
                ),
                1,
            ),
        ] {
            let hay = fuente.matches(linea).count();
            if hay != veces {
                mal.push(format!("security.rs dice {hay} veces, no {veces}, `{linea}`"));
            }
        }
        for (m, _) in CONJETURADA_CON_DEEP {
            if !admitidas.contains(&m) {
                mal.push(format!("m = {m} tiene cifra y el enunciado ya no la admite"));
            }
            if SIN_PISO.contains(&m) {
                mal.push(format!("m = {m} se declara sin piso y tiene cifra"));
            }
        }
        let mut medidas = Vec::new();
        for &m in &admitidas {
            if SIN_PISO.contains(&m) {
                continue;
            }
            let Some(&(_, decimas)) = CONJETURADA_CON_DEEP.iter().find(|(x, _)| *x == m) else {
                mal.push(format!("m = {m}: el enunciado la admite y no tiene cifra declarada"));
                continue;
            };
            let filas = 2 * (CICLO << m);
            let contexto = |opc: &ProofOptions| {
                let info =
                    TraceInfo::new_multi_segment(ANCHO + 1, ANCHO_AUX, ALEATORIOS, filas, vec![]);
                let mut p = Proof::new_dummy();
                let restricciones = PRINCIPALES + AUXILIARES;
                p.context = Context::new::<BaseElement>(info, opc.clone(), restricciones)
                    .expect("un contexto valido");
                p
            };
            let p = contexto(&o);
            let bits_base = p.context.num_modulus_bits();
            let campo = (bits_base * o.field_extension().degree()) as f64;
            let deep = bits_deep(&o, bits_base, filas);
            // La replica de la formula, con el campo entero, es la de la funcion.
            let real = p.conjectured_security::<Blake3>().bits();
            if conjeturada_con(&o, campo, hash) as u32 != real {
                mal.push(format!("m = {m}: la replica de la formula no da la conjeturada, {real}"));
            }
            // La replica del termino DEEP es la de la funcion: con cada variante, el limita la UDR.
            for v in &variantes {
                let udr = contexto(v).proven_security::<Blake3>().udr_bits();
                let deep_v = bits_deep(v, bits_base, filas);
                if deep_v.floor() as u32 != udr {
                    mal.push(format!(
                        "m = {m}, blowup {}: DEEP {deep_v:.2}, y la UDR de la variante es {udr}",
                        v.blowup_factor()
                    ));
                }
            }
            // El tope que el DEEP pone a la conjeturada: el menor de los terminos de la formula
            // con el en lugar del campo, sin el -1. Tiene que ser el mismo, y con el -1, uno menos.
            let tope = deep.min(bits_consultas(&o)).min(hash as f64);
            let con_menos_uno = conjeturada_con(&o, deep, hash);
            if tope != deep || con_menos_uno != tope - 1.0 {
                mal.push(format!(
                    "m = {m}: el tope es {tope:.2} y el DEEP {deep:.2}; con el -1, {:.2}",
                    con_menos_uno
                ));
            }
            medidas.push(format!(
                "m={m} ({filas} filas): DEEP {deep:.2}, con el -1 {con_menos_uno:.1}"
            ));
            if (tope * 10.0).round() as u32 != decimas {
                mal.push(format!(
                    "m = {m}: con el DEEP da {tope:.2}, y la cifra declarada es {}.{}",
                    decimas / 10,
                    decimas % 10
                ));
            }
        }
        println!(
            "[§708] la conjeturada con el termino DEEP, traza oculta de 2^(m+4) filas: {}",
            medidas.join(", ")
        );
        assert!(mal.is_empty(), "la conjeturada con el DEEP: {}", mal.join("; "));
        assert!(medidas.len() >= CONJETURADA_CON_DEEP.len(), "solo {} m medidas", medidas.len());
    }
}

/// **§717, el coste del juez de la edad.** Una prueba FABRICADA -nadie ha probado nada:
/// solo tiene la forma que el lector y la guarda de forma esperan- llega hasta la primera
/// comprobacion contra la prueba (la coherencia OOD) y cae alli. Lo que cuesta llegar es lo que
/// paga el juez por cualquier fichero: con el tope de `m` y las exenciones en flujo del fork, no
/// crece con `2^m` ni en memoria ni, sobre el tope, en CPU.
#[cfg(test)]
mod tests_coste {
    use crate::sal::MerkleConSal;
    use std::time::{Duration, Instant};

    use winter_air::proof::{Commitments, Context, OodFrame, Queries};
    use winter_air::{Air, Oculta};
    use winter_crypto::{BatchMerkleProof, Hasher};
    use winter_math::fields::QuadExtension;
    use winter_verifier::{ByteWriter, Deserializable, Serializable};

    use super::*;
    use crate::sal::VariasConSal;

    type E = QuadExtension<BaseElement>;

    const SP: Digest = [BaseElement::new(11); 4];
    const SM: Digest = [BaseElement::new(22); 4];

    fn pi_de(m: u32) -> EdadPublicInputs {
        let n = 1u64 << m;
        let (seq, t, emisor, todos, k) = (1000, 70, 0, true, 0);
        EdadPublicInputs { subraiz_pend: SP, subraiz_meta: SM, m, n, seq, t, emisor, todos, k }
    }

    fn info_de(m: u32) -> TraceInfo {
        let l = 2 * (CICLO << m);
        // §914: la marca v2 de la edad y las dos columnas aleatorias
        let marca = familias::marca_v2_de("EdadAir", COLUMNAS_ALEATORIAS as u8).expect("registro").escribir();
        TraceInfo::new_multi_segment(ANCHO + COLUMNAS_ALEATORIAS, ANCHO_AUX, ALEATORIOS, l, marca)
    }

    fn digest(k: u8) -> <Blake3 as Hasher>::Digest {
        Blake3::hash(&[k, 0x42])
    }

    /// Un lote con sal de la forma que el lector espera: `q` hojas sobre un arbol de `2^prof`.
    fn lote(prof: u32, q: usize) -> Vec<u8> {
        let nodos = vec![vec![digest(7); prof as usize]; q];
        let mut b = BatchMerkleProof::<Blake3> { nodes: nodos, depth: prof as u8 }.to_bytes();
        b.write_u32(q as u32);
        for i in 0..q {
            b.write(digest(i as u8));
        }
        b
    }

    fn tabla<F: FieldElement>(prof: u32, q: usize, fila: Vec<F>) -> Queries {
        let l = VariasConSal::<Blake3>::read_from_bytes(&lote(prof, q)).expect("lote");
        Queries::new::<Blake3, F, MerkleConSal<Blake3>>(l, vec![fila; q])
    }

    /// La prueba fabricada de `m`: contexto, compromisos, tres tablas, marco OOD y FRI con los
    /// anchos de `m`, y valores cualesquiera. Sin probador: unos milisegundos.
    fn fabricar(m: u32) -> Vec<u8> {
        let opts = opciones();
        let air = Oculta::<EdadAir>::new(info_de(m), pi_de(m), opts.clone());
        let lde = air.lde_domain_size();
        let fri = opts.to_fri_options();
        let capas = fri.num_fri_layers(lde);
        let ncomp = air.context().num_constraint_composition_columns();
        let nc = air.context().num_transition_constraints() + 16;
        let q = 42usize;
        let ld = lde.ilog2();
        let ctx = Context::new::<BaseElement>(info_de(m), opts, nc).expect("contexto");
        let mut b = ctx.to_bytes();
        b.write_u8(q as u8);
        let fri_raices = (0..=capas).map(|i| digest(10 + i as u8)).collect();
        b.write(Commitments::new::<Blake3>(vec![digest(1), digest(2)], digest(3), fri_raices));
        b.write(tabla(ld, q, vec![BaseElement::new(5); ANCHO + COLUMNAS_ALEATORIAS]));
        b.write(tabla(ld, q, vec![E::from(BaseElement::new(6)); ANCHO_AUX]));
        b.write(tabla(ld, q, vec![E::from(BaseElement::new(8)); ncomp]));
        let mut ood: Vec<u8> = Vec::new();
        for (ancho, base) in [(ANCHO + COLUMNAS_ALEATORIAS + ANCHO_AUX, 1u64), (ncomp, 3)] {
            let mut marco: Vec<u8> = vec![2];
            for i in 0..2 * ancho {
                marco.write(E::from(BaseElement::new(i as u64 + base)));
            }
            ood.write_u16(marco.len() as u16);
            ood.write_bytes(&marco);
        }
        b.write(OodFrame::read_from_bytes(&ood).expect("ood"));
        // FRI: las capas que el dominio pide, cada una con su lote, y un resto de 32
        b.write_u8(capas as u8);
        let mut dom = ld;
        for _ in 0..capas {
            dom -= fri.folding_factor().ilog2();
            let mut vals: Vec<u8> = Vec::new();
            for _ in 0..q * fri.folding_factor() {
                vals.write(E::from(BaseElement::new(9)));
            }
            let caminos = lote(dom, q);
            b.write_u32(vals.len() as u32);
            b.write_bytes(&vals);
            b.write_u32(caminos.len() as u32);
            b.write_bytes(&caminos);
        }
        let mut resto: Vec<u8> = Vec::new();
        for _ in 0..32 {
            resto.write(E::from(BaseElement::new(4)));
        }
        b.write_u16(resto.len() as u16);
        b.write_bytes(&resto);
        b.write_u8(0);
        b.write_u64(0);
        b
    }

    /// La cabeza que inventa quien envia la prueba: sus subraices y las raices que de ellas salen.
    fn cabeza(next_pending: u64) -> CabezaEdad {
        let m = m_canonico(next_pending).min(PROFUNDIDAD as u32);
        CabezaEdad {
            seq: 1000,
            pending_root: raiz_desde_subraiz(SP, m, PROFUNDIDAD),
            pmeta_root: raiz_desde_subraiz(SM, m, PROFUNDIDAD),
            next_pending,
        }
    }

    const AF: Afirmacion =
        Afirmacion { t: 70, k: 0, emisor: None, subraiz_pend: SP, subraiz_meta: SM };

    /// El menor de `veces` rechazos (el menor resiste la carga de la maquina) y el ultimo error.
    fn rechazo(prueba: &[u8], c: &CabezaEdad, veces: usize) -> (Duration, String) {
        let mut mejor = Duration::MAX;
        let mut err = String::new();
        for _ in 0..veces {
            let t0 = Instant::now();
            err = verificar_contra_cabeza(prueba, &AF, c).expect_err("la fabricada verifico");
            mejor = mejor.min(t0.elapsed());
        }
        (mejor, err)
    }

    /// Sobre el tope, el enunciado cae ANTES de leer la prueba y de construir el AIR, con su causa
    /// (no `no se deserializa`): con una prueba vacia y `nextPending = 2^M_MAX + 1`, y con la
    /// prueba fabricada de `m = 23`, la mayor que el lector admite (el dominio de extension cabe
    /// en 32 bits). En el tope, el enunciado pasa y es la prueba lo que se juzga. Falsador: con el
    /// tope de antes (24), la de `m = 23` llega al juez, paga un producto por fila de la traza y
    /// cae en la coherencia OOD.
    #[test]
    fn m_sobre_el_tope_cae_antes_de_leer_la_prueba() {
        let sobre = (1u64 << M_MAX) + 1;
        let e = verificar_contra_cabeza(&juez::solo_cabeza::<EdadAir>(), &AF, &cabeza(sobre)).expect_err("vacia");
        assert_eq!(e, format!("m = {} fuera de {M_MIN}..={M_MAX}", M_MAX + 1));
        let (t, e) = rechazo(&fabricar(23), &cabeza(1 << 23), 3);
        assert_eq!(e, format!("m = 23 fuera de {M_MIN}..={M_MAX}"));
        assert!(t < Duration::from_millis(50), "m = 23 sobre el tope tardo {t:?}");
        let e = verificar_contra_cabeza(&[], &AF, &cabeza(1 << M_MAX)).expect_err("vacia");
        assert!(e.starts_with("la prueba no se deserializa"), "en el tope: {e}");
    }

    /// **§721: el tope es 13, y `m = 13` entra al juez.** 13 es el `m` mas alto que cumple las dos
    /// cotas de la puerta del RFC-0007 con la traza oculta (la regla y la medida, en [`M_MAX`]). En
    /// 13 el enunciado pasa: con una prueba vacia, la causa es la del lector; con la fabricada, la
    /// de la coherencia OOD. Es la prueba lo que se juzga, no el tope. Al final, el valor: el tope
    /// es 13 y no otro. Falsador: con el tope en 12, el enunciado de `m = 13` cae con
    /// `m = 13 fuera de 3..=12`; con el tope en 14, falla el valor.
    #[test]
    fn m_13_entra_al_juez_y_se_juzga_la_prueba() {
        assert_eq!(comprobar_enunciado(&pi_de(13)), Ok(()), "el enunciado de m = 13");
        let e = verificar_contra_cabeza(&[], &AF, &cabeza(1 << 13)).expect_err("vacia");
        assert!(e.starts_with("la prueba no se deserializa"), "m = 13: {e}");
        let (_, e) = rechazo(&fabricar(13), &cabeza(1 << 13), 1);
        assert_eq!(e, "InconsistentOodConstraintEvaluations", "m = 13 llega a la prueba");
        assert_eq!(M_MAX, 13, "el tope de la regla");
    }

    /// **§721: `m = 14` cae por el tope, con su causa y en milisegundos.** La prueba fabricada de
    /// `m = 14` no se lee: el enunciado cae antes, con `m = 14 fuera de 3..=13`. Con un tope de 14,
    /// esa misma prueba llegaria a la coherencia OOD y caeria alli. Falsador: con el tope en 14, la
    /// causa es `InconsistentOodConstraintEvaluations`.
    #[test]
    fn m_14_cae_por_el_tope_en_milisegundos() {
        let causa = format!("m = 14 fuera de {M_MIN}..=13");
        let (t, e) = rechazo(&fabricar(14), &cabeza(1 << 14), 3);
        assert_eq!(e, causa, "la fabricada de m = 14");
        eprintln!("rechazo: m = 14 sobre el tope en {t:?}");
        assert!(t < Duration::from_millis(50), "m = 14 sobre el tope tardo {t:?}");
        assert_eq!(comprobar_enunciado(&pi_de(14)), Err(causa.clone()), "el enunciado");
    }

    /// La segunda implementacion tiene el mismo tope: su `M_MAX` (`tools/segunda/airs.py`) es una
    /// linea `M_MAX = <valor>` y el valor es este, y su enunciado lo usa con la misma causa.
    #[test]
    fn el_tope_de_la_segunda_es_el_del_juez() {
        let ruta = format!("{}/../../tools/segunda/airs.py", env!("CARGO_MANIFEST_DIR"));
        let fuente = std::fs::read_to_string(ruta).expect("airs.py");
        let lineas: Vec<&str> =
            fuente.lines().map(str::trim).filter(|l| l.starts_with("M_MAX = ")).collect();
        assert_eq!(lineas, vec![format!("M_MAX = {M_MAX}")], "el tope de la segunda");
        assert!(fuente.contains("self.m > self.M_MAX"), "la segunda no usa su tope");
        assert!(fuente.contains(r#"f"m = {self.m} fuera de {self.M_MIN}..={self.M_MAX}""#), "otra causa");
        let suelos: Vec<&str> =
            fuente.lines().map(str::trim).filter(|l| l.starts_with("M_MIN = ")).collect();
        assert_eq!(suelos, vec![format!("M_MIN = {M_MIN}")], "el suelo de la segunda (§914)");
    }

    /// En el tope, la prueba fabricada llega a la coherencia OOD y cae alli, y rechazarla no cuesta
    /// mucho mas que en `m = 3`: el divisor de las transiciones ya no recorre `1 + 8 * 2^m`
    /// exponenciaciones (el bloque) ni guarda la lista. Queda un producto en flujo por exencion, el
    /// unico termino lineal. Con el tope en 13, el cociente es de 1,3 a 1,7 (medido, seis corridas
    /// en release). Falsador: con la lista de upstream en `from_transition`, de 31,6 a 60,2 en
    /// `m = 13` (medido, tres corridas: unos 70 ms frente a 1,2-2,2 ms en `m = 3`; 57,9 al portar
    /// el §717). ⚠️ El umbral es 10, no 25 (§721): con 25, una corrida del falsador quedo a 1,26
    /// veces de el. Con 10, el honesto tiene unas 6 veces de holgura y el falsador cae con 3 o mas.
    #[test]
    fn el_coste_de_rechazar_no_crece_con_m() {
        let ood = "InconsistentOodConstraintEvaluations";
        let (t3, e3) = rechazo(&fabricar(M_MIN), &cabeza(1 << M_MIN), 5);
        assert_eq!(e3, ood, "m = {M_MIN}");
        let (tt, et) = rechazo(&fabricar(M_MAX), &cabeza(1 << M_MAX), 5);
        assert_eq!(et, ood, "m = {M_MAX}");
        let cociente = tt.as_secs_f64() / t3.as_secs_f64();
        eprintln!("rechazo: m = {M_MIN} {t3:?}, m = {M_MAX} {tt:?}, cociente {cociente:.1}");
        assert!(cociente < 10.0, "m = {M_MAX}: {tt:?} frente a {t3:?} en m = {M_MIN} ({cociente:.1})");
    }
}

/// **§718, la canonicidad u64 -> campo.** Un entero que el llamante da y que el juez
/// reduce modulo `p` tiene dos escrituras, `x` y `x + p`, y la misma prueba las acreditaba a las
/// dos. Aqui cada entero de fuera se acota sobre el `u64` tal como llega, ANTES de leer la
/// prueba: con una prueba vacia, la causa es la del enunciado y no `no se deserializa`. Falsador:
/// sin las cotas, cada alias de abajo llega al lector de la prueba (los tests con la prueba real
/// viven en el kit y en la capa, que tienen sobres firmados y un libro).
#[cfg(test)]
mod tests_canonicidad {
    use super::*;
    use crate::{cobro_pendiente as cobro, pago_en_curso as pago};
    use winter_math::StarkField;

    const P: u64 = BaseElement::MODULUS;
    const SP: Digest = [BaseElement::new(11); 4];
    const SM: Digest = [BaseElement::new(22); 4];
    const NO_LEE: &str = "la prueba no se deserializa";

    fn cabeza_edad() -> CabezaEdad {
        CabezaEdad {
            seq: 1000,
            pending_root: raiz_desde_subraiz(SP, M_MIN, PROFUNDIDAD),
            pmeta_root: raiz_desde_subraiz(SM, M_MIN, PROFUNDIDAD),
            next_pending: 5,
        }
    }

    fn af(emisor: Option<u64>) -> Afirmacion {
        Afirmacion { t: 70, k: 0, emisor, subraiz_pend: SP, subraiz_meta: SM }
    }

    /// El emisor es un indice de cuenta: `X + p`, `2^32` y el centinela `u64::MAX` caen en el
    /// enunciado con su causa; `2^32 - 1`, `2^32 - 2` (la reduccion del centinela, declarada en
    /// la capa) y 0 pasan y llegan a la prueba.
    #[test]
    fn edad_alias_del_emisor_es_err() {
        let c = cabeza_edad();
        for x in [7 + P, P, 1 << 32, u64::MAX] {
            let e = verificar_contra_cabeza(&juez::solo_cabeza::<EdadAir>(), &af(Some(x)), &c)
                .expect_err("alias aceptado");
            assert_eq!(e, format!("emisor = {x} no es un indice de cuenta: no cabe en 32 bits"));
        }
        for x in [0, 7, (1 << 32) - 2, (1 << 32) - 1] {
            let e = verificar_contra_cabeza(&[], &af(Some(x)), &c).expect_err("prueba vacia");
            assert!(e.starts_with(NO_LEE), "emisor {x}: {e}");
        }
    }

    /// §718, la canonicidad: el ancho del indice de cuenta de la segunda es el del juez. Su
    /// `BITS_CUENTA` es una linea `BITS_CUENTA = <valor>` con este valor, y su enunciado lo usa con
    /// la misma causa que [`comprobar_emisor`]. Falsador: con `BITS_CUENTA = 33` en `airs.py`, cae
    /// la primera comparacion.
    #[test]
    fn el_indice_de_cuenta_de_la_segunda_es_el_del_juez() {
        let ruta = format!("{}/../../tools/segunda/airs.py", env!("CARGO_MANIFEST_DIR"));
        let fuente = std::fs::read_to_string(ruta).expect("airs.py");
        let lineas: Vec<&str> =
            fuente.lines().map(str::trim).filter(|l| l.starts_with("BITS_CUENTA = ")).collect();
        assert_eq!(lineas, vec![format!("BITS_CUENTA = {BITS_CUENTA}")], "el ancho de la segunda");
        assert!(fuente.contains("self.emisor >= (1 << self.BITS_CUENTA)"), "la segunda no lo usa");
        assert!(
            fuente.contains(r#"f"emisor = {self.emisor} no es un indice de cuenta: no cabe en ""#),
            "otra causa"
        );
        assert_eq!(
            comprobar_emisor(1 << BITS_CUENTA).unwrap_err(),
            format!("emisor = {} no es un indice de cuenta: no cabe en 32 bits", 1u64 << 32)
        );
        assert!(comprobar_emisor((1 << BITS_CUENTA) - 1).is_ok());
    }

    /// Todos se escribe con emisor 0: un enunciado con `todos` y otro emisor es otra escritura del
    /// mismo y cae, tambien por `verificar` directo (por donde pasaba `emisor = p`).
    #[test]
    fn edad_todos_con_emisor_es_err() {
        let pi = verificar_contra_cabeza(&juez::solo_cabeza::<EdadAir>(), &af(None), &cabeza_edad()).expect_err("vacia");
        assert!(pi.starts_with(NO_LEE), "{pi}");
        let bien = EdadPublicInputs {
            subraiz_pend: SP,
            subraiz_meta: SM,
            m: M_MIN,
            n: 5,
            seq: 1000,
            t: 70,
            emisor: 0,
            todos: true,
            k: 0,
        };
        assert!(comprobar_enunciado(&bien).is_ok());
        // `p` es el alias de 0: con todos, `verificar` lo aceptaba con la prueba de todos.
        let causas = [
            (1, "todos con emisor = 1: todos se escribe con emisor 0".to_string()),
            (P, format!("emisor = {P} no es un indice de cuenta: no cabe en 32 bits")),
        ];
        for (x, causa) in causas {
            let malo = EdadPublicInputs { emisor: x, ..bien.clone() };
            assert_eq!(verificar(&juez::solo_cabeza::<EdadAir>(), &malo).expect_err("todos con emisor"), causa);
        }
    }

    fn cabeza_pago() -> pago::CabezaPago {
        pago::CabezaPago { seq: 1000, pending_root: SP, pmeta_root: SM }
    }

    fn af_pago(importe: u64, t: u64) -> pago::AfirmacionPago {
        pago::AfirmacionPago { receptor: SP, importe, t, nacido: 4 }
    }

    /// `importe + p` y `t + p` caen con la causa del techo, sobre el entero crudo.
    #[test]
    fn pago_importe_y_t_alias_son_err() {
        let c = cabeza_pago();
        let m = pago::MAX_VALOR;
        let cab = juez::solo_cabeza::<pago::PagoEnCursoAir>();
        let e = pago::verificar_contra_cabeza(&cab, &af_pago(250_000 + P, 100), &c).unwrap_err();
        assert_eq!(e, format!("el importe {} pasa del techo {m}", 250_000 + P));
        let e = pago::verificar_contra_cabeza(&cab, &af_pago(250_000, 100 + P), &c).unwrap_err();
        assert_eq!(e, format!("T {} o nacido 4 pasan del techo {m}", 100 + P));
        let e = pago::verificar_contra_cabeza(&cab, &af_pago(m + 1, 100), &c).unwrap_err();
        assert_eq!(e, format!("el importe {} pasa del techo {m}", m + 1));
        let e = pago::verificar_contra_cabeza(&[], &af_pago(m, m), &c).unwrap_err();
        assert!(e.starts_with(NO_LEE), "en el techo: {e}");
    }

    /// `inferior + p` cae con la causa del techo, sobre el entero crudo.
    #[test]
    fn cobro_inferior_alias_es_err() {
        let c = cobro::CabezaCobro { seq: 1000, pending_root: SP, pmeta_root: SM };
        let af = |inferior| cobro::AfirmacionCobro { receptor: SP, nacido: 4, inferior };
        let m = cobro::MAX_VALOR;
        for x in [1 + P, P, m + 1] {
            let e = cobro::verificar_contra_cabeza(&juez::solo_cabeza::<cobro::CobroPendienteAir>(), &af(x), &c).unwrap_err();
            assert_eq!(e, format!("las cotas {x} y {m} pasan del techo {m}"));
        }
        for x in [0, 1, m] {
            let e = cobro::verificar_contra_cabeza(&[], &af(x), &c).unwrap_err();
            assert!(e.starts_with(NO_LEE), "inferior {x}: {e}");
        }
    }
}

/// **§723, la ventana de la v1.** Lo que el circuito de la v1 comprueba de un valor
/// privado `v` con la banda `[l, u]`, escrito en enteros: `v < 2^63`, `(v - l) mod p < 2^63` y
/// `(u - v) mod p < 2^63`. Con el de la v2, `2^62` en los tres.
#[cfg(test)]
mod tests_ventana_v1 {
    use super::*;
    use winter_math::StarkField;

    const P: u128 = BaseElement::MODULUS as u128;

    fn pasa(v: u128, l: u128, u: u128, bits: u32) -> bool {
        let r = |x: u128| x % P;
        v < (1 << bits) && r(v + P - l) < (1 << bits) && r(u + P - v) < (1 << bits)
    }

    /// El mayor `v` fuera de la banda que la v1 deja pasar con el superior `u`: el mas grande de
    /// los `v < 2^63` con `v > u + p - 2^63`, si lo hay.
    fn fuera_que_pasa(u: u64) -> Option<u128> {
        let v = (1u128 << 63) - 1;
        (v > u as u128 && pasa(v, 0, u as u128, 63)).then_some(v)
    }

    /// La ventana empieza EXACTAMENTE en [`COTA_V1`]: con el superior una unidad por debajo hay un
    /// saldo fuera de la banda que la v1 deja pasar, y desde la cota no. Los dos del falsador
    /// medido (superior 0 y `4_999_999`) estan dentro, y el techo [`banda::MAX_VALOR`] fuera. Con la
    /// v2, ningun superior deja pasar nada de la ventana. Falsador: con `COTA_V1` una unidad mas
    /// alta o mas baja, cae la primera o la segunda comparacion.
    #[test]
    fn la_ventana_de_la_v1_acaba_en_la_cota() {
        assert!(fuera_que_pasa(COTA_V1 - 1).is_some(), "bajo la cota hay ventana");
        assert!(fuera_que_pasa(COTA_V1).is_none(), "en la cota no la hay");
        assert!(!v1_sin_ventana(COTA_V1 - 1) && v1_sin_ventana(COTA_V1));
        let w = (1u128 << 63) - (1u128 << 32) + 2;
        assert!(pasa(w, 0, 0, 63), "el falsador medido, [0, 0]");
        assert!(pasa(w + 4_999_999, 0, 4_999_999, 63), "el falsador medido, [0, 4_999_999]");
        assert!(fuera_que_pasa(banda::MAX_VALOR).is_none(), "con el techo, la v1 dice lo que parece");
        for u in [0u128, 1000, COTA_V1 as u128 - 1, banda::MAX_VALOR as u128] {
            for v in [1u128 << 62, w, w + u, (1u128 << 63) - 1] {
                assert!(!pasa(v, 0, u, 62), "la v2 deja pasar {v} con el superior {u}");
            }
        }
    }

    /// La segunda implementacion tiene la misma cota y la misma causa: su `COTA_V1`
    /// (`tools/segunda/airs.py`) es una linea `COTA_V1 = <valor>` con este valor, y su causa lleva
    /// el mismo prefijo. Falsador: con `COTA_V1 = 4294967293` en `airs.py`, cae la primera.
    #[test]
    fn la_cota_de_la_v1_de_la_segunda_es_la_del_juez() {
        let ruta = format!("{}/../../tools/segunda/airs.py", env!("CARGO_MANIFEST_DIR"));
        let fuente = std::fs::read_to_string(ruta).expect("airs.py");
        let lineas: Vec<&str> =
            fuente.lines().map(str::trim).filter(|l| l.starts_with("COTA_V1 = ")).collect();
        assert_eq!(lineas, vec![format!("COTA_V1 = {COTA_V1}")], "la cota de la segunda");
        let prefijo: Vec<&str> = fuente
            .lines()
            .map(str::trim)
            .filter(|l| l.starts_with("PRUEBA_V1_EN_LA_VENTANA = "))
            .collect();
        assert_eq!(
            prefijo,
            vec![format!("PRUEBA_V1_EN_LA_VENTANA = \"{PRUEBA_V1_EN_LA_VENTANA}\"")],
            "el prefijo de la causa"
        );
    }
}

/// §723, la ventana de la v1: la causa cuando ninguna version verifica.
#[cfg(test)]
mod tests_causa_v1 {
    use super::la_causa_de_la_que_llego_mas_lejos as causa;

    /// La version que no es la de la prueba cae en la OOD; la otra, por lo suyo, y esa es la causa.
    /// Si las dos caen por lo mismo, o por otra cosa las dos, la de la v2. Falsador: devolviendo
    /// siempre la de la v2, una prueba de cobro de la v1 maleada en el resto dejaba de caer por
    /// `RemainderCommitmentMismatch` (medido en `zk-ssl-verify`).
    #[test]
    fn la_causa_es_la_de_la_version_que_llego_mas_lejos() {
        let ood = || Err("InconsistentOodConstraintEvaluations".to_string());
        let resto = || Err("FriVerificationFailed(RemainderCommitmentMismatch)".to_string());
        let nodo = || Err("TraceQueryDoesNotMatchCommitment".to_string());
        assert_eq!(causa(ood(), resto()), resto(), "una de la v1 maleada");
        assert_eq!(causa(resto(), ood()), resto(), "una de la v2 maleada");
        assert_eq!(causa(ood(), ood()), ood(), "ninguna llega");
        assert_eq!(causa(nodo(), resto()), nodo(), "las dos pasan de la OOD: la de la v2");
    }
}
