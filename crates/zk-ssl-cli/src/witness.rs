//! # El testigo: cuatro clases, y **el primer ancla**
//!
//! Implementación de referencia de lo que **un tercero** correría contra un
//! nodo ZK-SSL. Consulta `zkssl_signedEpochHead` a la cadencia del latido,
//! verifica con `zk-ssl-verify`, y **fija la clave que ve la primera vez**.
//!
//! ## ⚠️ El testigo no informa la decisión del ancla: **ES el ancla**
//!
//! §244 dejó escrito que la clave pública **no tiene ancla**, y que elegir
//! una exigía saber *quién va a usarla*. Estaban tratados como dos
//! problemas, y son **el mismo**:
//!
//! > Un testigo que anota **la clave que vio la primera vez** y **se
//! > detiene si cambia** está haciendo *trust-on-first-use*.
//!
//! Es el modelo de SSH. Débil, con nombre propio, y **es un ancla de
//! verdad**: desde el primer encuentro, el operador **ya no puede cambiar
//! de clave sin que un tercero lo vea**.
//!
//! ## ⚠️⚠️ Lo que TOFU NO da: el primer encuentro
//!
//! **Si el operador ya estaba mintiendo cuando el testigo arrancó, TOFU
//! fija la mentira.** No hay nada en este código que lo detecte, y no lo
//! habrá: es una limitación **del modelo**, no de la implementación.
//!
//! Cerrarla exige un ancla **anterior al primer encuentro** —una huella
//! publicada, una autoridad, una contraparte— y eso sigue sin decidirse.
//! Lo que TOFU aporta es **acotar la ventana a un instante** en vez de
//! dejarla abierta para siempre.
//!
//! ## ⚠️ Un testigo que opera el propio operador NO PRUEBA NADA
//!
//! Guardar tus propias firmas y comprobar que coinciden es circular. Esto
//! **no crea confianza**: **quita la excusa de que no hay cómo**, igual que
//! `spec/RPC.md` hizo con la segunda implementación.
//!
//! ## Cuatro clases, y **dos detienen**
//!
//! | clase | qué es | qué hace |
//! |---|---|---|
//! | **hueco** | falta una cabeza: reinicio, red, o el nodo sin clave | anota y **sigue** |
//! | **fallo de verificación** | la firma no valida | anota y **sigue** — puede ser transitorio, y **el patrón importa** |
//! | ⚠️ **vista dividida** | **mismo índice, digest distinto** | **SE DETIENE** |
//! | ⚠️ **cambio de clave** | la pública no es la fijada | **SE DETIENE** |
//!
//! ⚠️ **La cuarta detiene porque rotar la clave es exactamente cómo un
//! operador escaparía de la tercera**: rota, y presenta otra historia
//! firmada con otra clave. Un testigo que aceptara la nueva en silencio
//! **acepta cualquier cosa a partir de ahí**.
//!
//! ⚠️ Y se detiene ante las dos porque **seguir sería sobrescribir el
//! propio hallazgo con ruido posterior**. Importa preservar la evidencia,
//! no acumular registros.
//!
//! La distinción no es adorno: **§242 declaró que habrá huecos** —una firma
//! en memoria se pierde al reiniciar—. Sin separarlos, **el primer reinicio
//! produce una alarma falsa** y el testigo pierde credibilidad antes de
//! servir para nada.
//!
//! ## El segundo canal: **la historia** (§294)
//!
//! Las cuatro clases de arriba juzgan **la cabeza**. Desde §294 hay un
//! canal aparte que juzga **la historia**: el testigo pide
//! `zkssl_consistencyProof` con el `mmrSize` que custodia y comprueba que
//! la cima nueva EXTIENDE a la suya, con el objeto de §291 como juez.
//!
//! | clase | qué es | qué hace |
//! |---|---|---|
//! | **anclando** | primera cabeza v3: se fija la pareja de partida | anota y **sigue** |
//! | **consistencia-pendiente** | el camino espera a la cabeza que lo firma | anota y **sigue** |
//! | **extiende** | la nueva contiene a la custodiada | anota y **sigue** |
//! | ⚠️ **no-extiende** | **la historia se bifurcó o se recortó** | **SE DETIENE** |
//! | **por-detras** | el acumulador del nodo va detrás: reseteo VISIBLE | anota y **sigue** |
//!
//! ⚠️ **`por-detras` no detiene a propósito**: es el nodo **diciendo la
//! verdad** sobre su reinicio sin diario (§292 lo prometió visible, §293 lo
//! hizo cable). Detener ahí quemaría al testigo en cada reinicio legítimo.
//!
//! ⚠️⚠️ **Y el juicio NO es síncrono.** La pareja firmada es el acumulador
//! ANTES de cada cabeza, así que **el camino de tamaño `t` lo firma la
//! cabeza siguiente en emitirse**: el testigo guarda el camino y espera, a
//! lo sumo un latido. Un testigo que exigiera igualdad instantánea no
//! casaría jamás.
//!
//! ## ⚠️⚠️ DETECTAR NO ES DISTINGUIR
//!
//! **Quien modifique este código tiene que leerlo aquí, no en un asiento.**
//!
//! El testigo ve un índice repetido con dos digests. **No puede saber si
//! fue un reinicio con el estado de travesía mal restaurado o una vista
//! dividida deliberada.** §110.3 lo dijo antes de que existiera este
//! código. Lo mismo con la clave: un cambio puede ser una rotación legítima
//! que nadie anunció, **porque no hay canal para anunciarla**.
//!
//! ## Y sin embargo es OPONIBLE
//!
//! Un tercero con **dos cabezas firmadas del mismo índice y distinto
//! contenido** tiene un artefacto que **el operador no puede negar haber
//! emitido**. Que sea explicable por accidente **no lo hace menos
//! oponible**: obliga al operador a explicarse.
//!
//! **Ese es el modelo Certificate Transparency entero**: no impedir la
//! mentira, sino hacerla imposible de negar.
//!
//! ## ⚠️ Lo que este testigo NO da
//!
//! - **No detecta omisión.** Un operador que no publica una operación
//!   produce cabezas perfectamente consistentes. Eso es el eslabón 5 —el
//!   recibo de admisión—, no éste.
//! - **No guarda el histórico del nodo**: guarda lo que él mismo ve. Si
//!   estuvo apagado, no hay a quién pedírselo (§242).
//! - **No comprueba la custodia**: el nodo la *afirma* (§244).

use std::collections::BTreeMap;
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;

use clap::Args;
use serde::Deserialize;
use serde_json::{json, Value};
// ⚠️ Import EXPLICITO, nunca glob: `zk-ssl-verify` reexporta **su propio**
// `Veredicto` (el de `reverificacion`, §279), homonimo del de este modulo.
use xmss::{KeyPair, SigningKey};
use zk_ssl_guardian::{GuardianError, GuardianIndice, Reconciliacion};
use zk_ssl_verify::{
    indice_de_firma, mmr, preambulo_cofirma, verificar_cabeza, verificar_cofirma,
    CabezaFirmada,
    COFIRMA_V_MAX, COFIRMA_VERSION, Conjunto, VerificaError, VERSION_FORMATO,
};
use zk_ssl_wire::{CofirmaDto, SignedEpochHeadDto};
use zeroize::Zeroize;

/// Lo que el testigo concluye de cada consulta.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Veredicto {
    Nueva { indice: u64, digest: String },
    /// La misma que la anterior: el latido no ha corrido todavía.
    Repetida { indice: u64 },
    /// Falta al menos una. Anota y **sigue**.
    Hueco { desde: u64, hasta: u64 },
    /// El nodo sirve, pero sin firma. Anota y **sigue**.
    SinFirma { motivo: String },
    /// ⚠️ **§314 - NO hubo respuesta usable del nodo.** Transporte caido,
    /// respuesta ilegible o cuerpo sin `result`. El `motivo` lo escribe el
    /// CLIENTE: no confundir con el `reason` de [`Veredicto::SinFirma`], que
    /// lo dice el nodo. Anota y **sigue**.
    SinRespuesta { motivo: String },
    /// La firma no valida. Anota y **sigue**: puede ser transitorio.
    NoVerifica { indice: u64, error: String },
    /// ⚠️ **MISMO ÍNDICE, DIGEST DISTINTO.** Se DETIENE.
    VistaDividida { indice: u64, digest_a: String, digest_b: String },
    /// ⚠️ **La clave pública no es la fijada.** Se DETIENE.
    CambioDeClave { fijada: String, recibida: String },
}

impl Veredicto {
    /// ⚠️ **Dos clases detienen**: la vista dividida y el cambio de clave.
    /// La segunda porque **rotar es cómo se escapa de la primera**.
    pub fn detiene(&self) -> bool {
        matches!(self, Veredicto::VistaDividida { .. } | Veredicto::CambioDeClave { .. })
    }

    /// Nombre **estable** de la clase, para el diario.
    ///
    /// ⚠️ **No se usa `{:?}`.** Hasta §248 el diario guardaba una cadena de
    /// `Debug`, y eso **cambia si alguien toca un `derive`**: un formato de
    /// archivo que depende de un detalle de implementación **no es un
    /// formato**. Estos nombres son parte del formato y no se tocan sin
    /// subir [`DIARIO_VERSION`].
    pub fn clase(&self) -> &'static str {
        match self {
            Veredicto::Nueva { .. } => "nueva",
            Veredicto::Repetida { .. } => "repetida",
            Veredicto::Hueco { .. } => "hueco",
            Veredicto::SinFirma { .. } => "sin-firma",
            Veredicto::SinRespuesta { .. } => "sin-respuesta",
            Veredicto::NoVerifica { .. } => "no-verifica",
            Veredicto::VistaDividida { .. } => "vista-dividida",
            Veredicto::CambioDeClave { .. } => "cambio-de-clave",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
//  §294 · EL SEGUNDO CANAL: la historia (eslabon 3 de la 83, tramo (i))
// ═══════════════════════════════════════════════════════════════════════

/// Lo que el testigo concluye sobre **la historia**, no sobre la cabeza.
///
/// ⚠️ **Canal APARTE de [`Veredicto`], y no es cosmetica.** Son dos
/// preguntas ortogonales: una cabeza `Nueva`, perfectamente firmada, puede
/// venir de una historia RECORTADA. Meter esto en `Veredicto` obligaria a
/// elegir cual de las dos se anota, y dos testigos que comparan diarios
/// necesitan **las dos**.
///
/// ⚠️⚠️ **La comprobacion NO es sincrona, y el diseño lo dice.** La pareja
/// firmada es el acumulador ANTES de cada cabeza —el push va tras el emit
/// (§293)—, asi que **el camino de tamaño `t` lo firma la cabeza SIGUIENTE
/// en emitirse**. Por eso existe [`Consistencia::Pendiente`]: un testigo
/// que exigiera igualdad instantanea no casaria JAMAS.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Consistencia {
    /// Primera cabeza v3: se fija la pareja de partida. No juzga nada.
    Anclando { t: u64 },
    /// Camino pedido y guardado; falta la cabeza que lo firma. Sigue.
    Pendiente { de_t: u64, esperando_t: u64 },
    /// La cima nueva EXTIENDE a la custodiada. Append-only, comprobado.
    Extiende { de_t: u64, a_t: u64, camino: Vec<String> },
    /// ⚠️⚠️ **La nueva NO extiende a la custodiada.** Se DETIENE.
    NoExtiende { de_t: u64, a_t: u64, camino: Vec<String> },
    /// El acumulador del nodo va POR DETRAS de lo custodiado. Anota y sigue.
    PorDetras { t_nodo: u64, pedido: u64 },
    /// El servicio no dio camino, con su razon. Anota y sigue.
    SinCamino { motivo: String },
    /// ⚠️ **§314 - No hubo respuesta usable a la peticion de camino.**
    /// Mismo criterio que [`Veredicto::SinRespuesta`]: lo dice el cliente, no
    /// el servicio. Anota y sigue.
    SinRespuesta { motivo: String },
    /// La cabeza no es v3: no lleva pareja que extender. Anota y sigue.
    NoAplica,
}

impl Consistencia {
    /// ⚠️ **Solo una detiene**, y es la hermana de la vista dividida: una
    /// historia bifurcada o recortada es exactamente aquello para lo que
    /// este testigo existe.
    ///
    /// ⚠️ **`PorDetras` NO detiene**: es el nodo **diciendo la verdad**
    /// sobre su propio reseteo —§292 lo prometio VISIBLE y §293 lo hizo
    /// cable—. Detener ahi quemaria al testigo en cada reinicio legitimo,
    /// que es el argumento de §242 por el que `Hueco` no es una alarma.
    pub fn detiene(&self) -> bool {
        matches!(self, Consistencia::NoExtiende { .. })
    }

    /// Nombre **estable** de la clase, para el diario. Mismo criterio que
    /// [`Veredicto::clase`]: no se tocan sin subir [`DIARIO_VERSION`].
    pub fn clase(&self) -> &'static str {
        match self {
            Consistencia::Anclando { .. } => "anclando",
            Consistencia::Pendiente { .. } => "consistencia-pendiente",
            Consistencia::Extiende { .. } => "extiende",
            Consistencia::NoExtiende { .. } => "no-extiende",
            Consistencia::PorDetras { .. } => "por-detras",
            Consistencia::SinCamino { .. } => "sin-camino",
            Consistencia::SinRespuesta { .. } => "consistencia-sin-respuesta",
            Consistencia::NoAplica => "no-aplica",
        }
    }
}

/// Un camino pedido y **aun no juzgado**.
///
/// ⚠️ Todo en **hex tal cual vino del cable**: el testigo no transcodifica
/// nada (§248). El `Digest` solo vive en variables locales, dentro de
/// [`juzgar`] — asi `zk-ssl-verify` no crece su superficie por un tipo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pendiente {
    pub de_cima: String,
    pub de_t: u64,
    /// El `mmrSize` que la respuesta anuncio: la cabeza que lo firma es
    /// **la siguiente en emitirse**, a lo sumo un latido despues.
    pub esperando_t: u64,
    pub camino: Vec<String>,
}

/// 32 bytes desde el hex del cable. `None` si no son exactamente 32.
fn hex32(s: &str) -> Option<[u8; 32]> {
    let h = s.trim_start_matches("0x");
    if h.len() != 64 {
        return None;
    }
    let mut o = [0u8; 32];
    for i in 0..32 {
        o[i] = u8::from_str_radix(&h[i * 2..i * 2 + 2], 16).ok()?;
    }
    Some(o)
}

/// El juicio: ¿la cima nueva EXTIENDE a la custodiada?
///
/// ⚠️ **Lo ilegible no pasa por bueno**: cualquier hex que no convierta da
/// `false`. Un juez que se saltara lo que no entiende absolveria por
/// ignorancia.
///
/// ⚠️ `mmr::hoja_desde_bytes` **dice «hoja» y aqui se usa sobre cimas y
/// caminos**. Su cuerpo es la conversion PURA (`digest_from_bytes`), sin
/// dominio: vale. Se usa tal cual en vez de pedir un reexport de `Digest`
/// —no se crece la superficie del verificador por cosmetica de nombre—.
fn juzgar(p: &Pendiente, cima_nueva: &str, t_nueva: u64) -> bool {
    let leer = |s: &str| hex32(s).and_then(|b| mmr::hoja_desde_bytes(&b));
    let vieja = match leer(&p.de_cima) {
        Some(d) => d,
        None => return false,
    };
    let nueva = match leer(cima_nueva) {
        Some(d) => d,
        None => return false,
    };
    let mut camino = Vec::with_capacity(p.camino.len());
    for s in &p.camino {
        match leer(s) {
            Some(d) => camino.push(d),
            None => return false,
        }
    }
    mmr::verificar_consistencia(vieja, p.de_t, nueva, t_nueva, &camino)
}

/// **Al llegar una cabeza**: juzga el camino pendiente si esta es la que lo
/// firma, ancla la pareja de partida, o no dice nada.
///
/// `None` = nada que decir del canal de la historia esta vuelta.
pub fn al_llegar_cabeza(m: &mut Memoria, v: &Value) -> Option<Consistencia> {
    if leer_q(&v["formatVersion"]).unwrap_or(0) != 3 {
        return Some(Consistencia::NoAplica);
    }
    let cima = v["mmrRoot"].as_str()?.to_string();
    let t = leer_q(&v["mmrSize"]).ok()?;

    // ⚠️⚠️ §295 · **EL RETROCESO MANDA SOBRE LO PENDIENTE**, y va ANTES
    // que nada. Si el acumulador del nodo va POR DETRAS de lo custodiado,
    // el camino que esperaba **no puede llegar JAMAS**: su `esperando_t`
    // pertenece a una historia que este nodo ya no tiene. Sin esta regla,
    // un reseteo pillaba al testigo con una pendiente viva y lo dejaba
    // **ciego para siempre** — anotando `consistencia-pendiente` vuelta
    // tras vuelta sin decir nunca lo que estaba pasando.
    //
    // ⚠️ Lo cazo el BANCO del §295 **antes de correrlo**, al diseñar su
    // negativo: ningun unitario del §294 podia verlo, porque el caso
    // exige un nodo que se resetea DEBAJO de un cliente vivo. Es la
    // leccion de §293 otra vez — el banco ve el flujo, el unitario la
    // pieza.
    if let Some(pt) = m.pareja.as_ref().map(|(_, t)| *t) {
        if t < pt {
            m.pendiente = None;
            return Some(Consistencia::PorDetras { t_nodo: t, pedido: pt });
        }
    }

    if let Some(p) = m.pendiente.take() {
        if t > p.esperando_t {
            // ⚠⚠ §295 · **LA PENDIENTE CADUCA.** El testigo muestrea; el
            // nodo late. Si consulta mas despacio que el latido, la cabeza
            // que firmaba `esperando_t` **ya paso sin que la viera**, y esa
            // pendiente no casara nunca. Se descarta y se vuelve a pedir en
            // esta misma vuelta: `None` deja pasar al bucle, que tiene red.
            return None;
        }
        if t < p.esperando_t {
            let (de_t, esperando_t) = (p.de_t, p.esperando_t);
            m.pendiente = Some(p);
            return Some(Consistencia::Pendiente { de_t, esperando_t });
        }
        // ⚠️ Esta es la cabeza que FIRMA el camino: se juzga.
        return Some(if juzgar(&p, &cima, t) {
            // El ancla avanza SOLO cuando la historia se demostro.
            m.pareja = Some((cima, t));
            Consistencia::Extiende { de_t: p.de_t, a_t: t, camino: p.camino }
        } else {
            // ⚠️ El ancla NO se mueve ante un hallazgo: preservar la
            // evidencia importa mas que seguir (la razon de §245).
            Consistencia::NoExtiende { de_t: p.de_t, a_t: t, camino: p.camino }
        });
    }

    match m.pareja.as_ref().map(|(_, t)| *t) {
        None => {
            m.pareja = Some((cima, t));
            Some(Consistencia::Anclando { t })
        }
        // ⚠️⚠️ §295 · **UN ANCLA EN t=0 NO ES UN ANCLA.** El genesis
        // declara cima=as_digest(0) y t=0 (§292), y el servicio responde a
        // un `oldSize` de 0 con «no hay historia que extender»: un testigo
        // que arranca contra un nodo RECIEN NACIDO se anclaba ahi y **no
        // podia salir JAMAS** —pedia camino, se lo negaban por diseno, y su
        // ancla no se movia nunca—. Se re-ancla en la primera cabeza con
        // historia de verdad.
        //
        // ⚠⚠ **Lo cazo EL BANCO en su primera corrida**: siete vueltas,
        // siete `sin-camino`. Ningun unitario lo vio porque todos fabricaban
        // cabezas con t>0 — el caso limite estaba en el arranque, no en la
        // matematica.
        Some(0) if t > 0 => {
            m.pareja = Some((cima, t));
            Some(Consistencia::Anclando { t })
        }
        // El retroceso ya no se decide aqui (§295): se resuelve arriba,
        // ANTES de lo pendiente. Sin pendiente y sin retroceso, el camino
        // se pide fuera (ahi hay red).
        Some(_) => None,
    }
}

/// **Al llegar la respuesta de `zkssl_consistencyProof`**: guarda el camino
/// como pendiente, o **nombra** la negativa.
///
/// ⚠️ **La negativa se decide por ESTRUCTURA, no por su texto.** El nodo
/// sirve `mmrSize` en las tres respuestas; si va por detras de lo
/// custodiado, eso se **mide**. Parsear la frase seria atarse a una
/// redaccion.
pub fn al_llegar_camino(m: &mut Memoria, r: &Value) -> Consistencia {
    let pedido = match m.pareja.as_ref() {
        Some((_, t)) => *t,
        None => return Consistencia::SinCamino { motivo: "sin pareja custodiada".into() },
    };
    let t_nodo = leer_q(&r["mmrSize"]).ok();
    if r["available"].as_bool() != Some(true) {
        if let Some(tn) = t_nodo {
            if tn < pedido {
                return Consistencia::PorDetras { t_nodo: tn, pedido };
            }
        }
        return Consistencia::SinCamino {
            motivo: r["reason"].as_str().unwrap_or("sin motivo declarado").into(),
        };
    }
    let esperando_t = match t_nodo {
        Some(t) => t,
        None => return Consistencia::SinCamino { motivo: "respuesta sin mmrSize".into() },
    };
    let camino: Vec<String> = match r["camino"].as_array() {
        Some(a) => a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect(),
        None => return Consistencia::SinCamino { motivo: "respuesta sin camino".into() },
    };
    let de_cima = m.pareja.as_ref().map(|(c, _)| c.clone()).unwrap_or_default();
    m.pendiente = Some(Pendiente { de_cima, de_t: pedido, esperando_t, camino });
    Consistencia::Pendiente { de_t: pedido, esperando_t }
}

/// Versión del formato del diario.
///
/// ⚠️ **Va en CADA LÍNEA, no en una cabecera.** Dos testigos que comparan
/// diarios necesitan saber que hablan el mismo idioma, y las líneas tienen
/// que seguir siendo independientes: se concatenan, se parten y se envían
/// sueltas. Es el argumento de §236 —*un campo vacío miente, una versión
/// dice la verdad*— aplicado al archivo.
/// ⚠️ **1 → 2 en §294**: la linea gana la pareja del MMR
/// (`mmrRoot`/`mmrSize`) y el canal `consistencia`. Sin la pareja, un
/// diario **no permite reauditar la extension sin el nodo**, que es el
/// criterio de §248 con el que se eligieron sus campos.
///
/// ⚠️ Los diarios **v1 siguen valiendo**: `auditar_lineas` acepta toda
/// version `<= DIARIO_VERSION`. Lo custodiado no caduca (§289, §292).
///
/// ⚠️ **NO confundir con `DIARIO_VERSION` del NODO** (`node/src/diario.rs`):
/// homonimas, formatos DISTINTOS, y ninguna gobierna a la otra.
/// ⚠️ **2 -> 3 en §314**: nacen las clases `sin-respuesta` y
/// `consistencia-sin-respuesta`, y con ellas el campo `motivo`. **No es
/// solo un valor mas**: el significado de `sin-firma` se ESTRECHA, porque
/// deja de cubrir <<no hubo respuesta>>. Por eso sube la version.
pub const DIARIO_VERSION: u8 = 3;

/// Una línea del diario: **lo suficiente para que un tercero reverifique
/// sin el nodo**, meses después.
///
/// ⚠️ **Ese es el criterio que decide qué campos entran.** La firma va
/// entera —18.519 bytes, 37 KB en hexadecimal— porque **es lo que impide
/// que un testigo malicioso fabrique evidencia contra el operador**. Sin
/// ella, comparar diarios no probaría nada: cualquiera podría escribir el
/// digest que quisiera.
///
/// ⚠️ Y el hexadecimal se guarda **tal como vino del cable**, sin
/// transcodificar: lo que está en el diario es literalmente lo que el nodo
/// sirvió.
/// ⚠️ **§314 - CORRECCION.** La frase de arriba -- <<lo que esta en el
/// diario es literalmente lo que el nodo sirvio>> -- era FALSA cuando no
/// habia respuesta: el cliente FABRICABA el objeto. Se cita y se corrige
/// (§247), no se borra. Desde §314 esa clase de linea lleva
/// `clase: "sin-respuesta"` y su `motivo`, **nunca un `reason` que el nodo
/// no dijo**.
pub fn linea_de_diario(v: &Veredicto, servido: &Value, visto_unix: u64) -> Value {
    let mut l = json!({
        "v": DIARIO_VERSION,
        "clase": v.clase(),
        "vistoUnix": visto_unix,
    });
    // ⚠️ Solo cuando hay cabeza firmada: es lo unico reverificable.
    if servido["available"].as_bool() == Some(true) {
        // ⚠️ §294: `mmrRoot`/`mmrSize` entran por el criterio de §248 —lo
        // suficiente para reverificar SIN el nodo—: sin la pareja, la
        // extension no se puede reauditar meses despues.
        // ⚠⚠ §295 · **LOS SIETE CAMPOS DE LA CABEZA, TAMBIEN.** §248
        // eligio estos campos para comprobar LA FIRMA; desde §294
        // `verificar` ademas RECOMPONE, y `auditar_lineas` lo reutiliza tal
        // cual — asi que un diario sin `seq`, `n` y las cinco raices **no
        // se puede reauditar**: trece lineas legitimas salieron como
        // `firma-no-verifica` con «QUANTITY no es cadena» en la primera
        // corrida del banco.
        //
        // ⚠️ Es el hueco de §292 por TERCERA vez: **dos listas son dos
        // productores del mismo contrato**, y quien toca una se mide contra
        // la otra. Aqui las ata un test.
        //
        // ⚠️ Coste: siete campos mas por linea — medio KB frente a los
        // ~37 KB que ya ocupa la firma (§248). Despreciable.
        for k in ["index", "epochDigest", "domain", "formatVersion", "signature",
                  "publicKey", "emittedAtUnix", "beatSeconds", "custody",
                  "custodyChecked", "mmrRoot", "mmrSize",
                  "seq", "n", "accountsRoot", "pendingRoot", "frozenRoot",
                  "chainDigest", "acusesRoot"] {
            if !servido[k].is_null() {
                l[k] = servido[k].clone();
            }
        }
    } else if let Some(r) = servido["reason"].as_str() {
        l["reason"] = json!(r);
    }
    // ⚠️ §314 - El motivo del CLIENTE va en `motivo`, nunca en `reason`:
    // `reason` significa <<lo que dijo el nodo>>, y aqui el nodo no dijo nada.
    if let Veredicto::SinRespuesta { motivo } = v {
        l["motivo"] = json!(motivo);
    }
    l
}

/// La linea con **los dos canales** (§294).
///
/// ⚠️ [`linea_de_diario`] se conserva **con su firma intacta**: es la que
/// usan el auditor y sus pruebas, y cambiarla habria movido codigo que no
/// es de este corte.
///
/// ⚠️ **El camino viaja en la linea del veredicto que lo JUZGO**, no en la
/// que lo pidio: con la pareja de la linea previa, la de esta y el camino,
/// un tercero re-verifica la extension **offline**. Un camino no anotado no
/// se puede reauditar.
pub fn linea_de_diario_con(
    v: &Veredicto,
    servido: &Value,
    visto_unix: u64,
    c: Option<&Consistencia>,
) -> Value {
    let mut l = linea_de_diario(v, servido, visto_unix);
    let c = match c {
        Some(c) => c,
        None => return l,
    };
    let q = |x: u64| json!(format!("{x:#x}"));
    let mut o = json!({ "clase": c.clase() });
    match c {
        Consistencia::Anclando { t } => {
            o["t"] = q(*t);
        }
        Consistencia::Pendiente { de_t, esperando_t } => {
            o["deT"] = q(*de_t);
            o["esperandoT"] = q(*esperando_t);
        }
        Consistencia::Extiende { de_t, a_t, camino }
        | Consistencia::NoExtiende { de_t, a_t, camino } => {
            o["deT"] = q(*de_t);
            o["aT"] = q(*a_t);
            o["camino"] = json!(camino);
        }
        Consistencia::PorDetras { t_nodo, pedido } => {
            o["tNodo"] = q(*t_nodo);
            o["pedido"] = q(*pedido);
        }
        Consistencia::SinCamino { motivo } => {
            o["reason"] = json!(motivo);
        }
        Consistencia::SinRespuesta { motivo } => {
            o["motivo"] = json!(motivo);
        }
        Consistencia::NoAplica => {}
    }
    l["consistencia"] = o;
    l
}

// ⚠️⚠️ §314 - LA AUSENCIA DE RESPUESTA ES UNA CLASE, NO UN OBJETO
// INVENTADO. Hasta aqui, cuando el transporte fallaba, la respuesta era
// ilegible o el cuerpo no traia `result`, el cliente FABRICABA un
// `{"available": false, "reason": ...}` y se lo pasaba al juez como si lo
// hubiera servido el nodo -- y el diario lo guardaba asi. Un tercero no podia
// distinguir <<el nodo dijo que no>> de <<no hubo nodo>> mas que leyendo prosa.

/// Lo que devuelve pedirle algo al nodo.
pub enum Servido {
    /// El `result` del nodo, **tal cual vino**.
    Respuesta(Value),
    /// No hubo respuesta usable. **Lo dice el CLIENTE**, no el nodo: por eso
    /// su campo se llama `motivo` y no `reason`.
    SinRespuesta { motivo: String },
}

/// La parte PURA: que es un cuerpo JSON-RPC ya parseado.
///
/// ⚠️ Se separa de [`pedir`] para que **sea testeable sin red**. La tercera
/// fabricacion -- la respuesta sin `result` -- era la mas callada de las
/// tres: se convertia en `Value::Null` y acababa clasificada como `sin-firma`
/// con el `reason` **ausente**.
pub fn del_cuerpo(v: Value) -> Servido {
    match v.get("result") {
        Some(r) => Servido::Respuesta(r.clone()),
        None => Servido::SinRespuesta { motivo: "respuesta sin `result`".into() },
    }
}

/// Una peticion al nodo, con la ausencia de respuesta EN EL TIPO.
///
/// ⚠️⚠️ **Un solo sitio.** Hasta §314 este `match` estaba escrito DOS veces
/// en `run`, a veinte lineas de distancia: dos productores del mismo contrato
/// (§292), y por eso el mismo defecto vivia en los dos canales.
///
/// ⚠️ La red no se prueba en un unitario: lo que se testea es
/// [`del_cuerpo`]; el camino con red lo ejercita el banco.
pub fn pedir(agente: &ureq::Agent, url: &str, cuerpo: Value) -> Servido {
    match agente.post(url).send_json(cuerpo) {
        Err(e) => Servido::SinRespuesta { motivo: format!("transporte: {e}") },
        Ok(r) => match r.into_json::<Value>() {
            Err(e) => Servido::SinRespuesta { motivo: format!("respuesta ilegible: {e}") },
            Ok(v) => del_cuerpo(v),
        },
    }
}

/// Lo que el testigo recuerda.
#[derive(Default)]
pub struct Memoria {
    /// ⚠️ Si un índice ya visto reaparece con OTRO digest, eso es la vista
    /// dividida de §110.3.
    vistos: BTreeMap<u64, String>,
    ultimo: Option<u64>,
    /// ⚠️ **EL ANCLA.** La clave pública que se vio la primera vez.
    clave_fijada: Option<String>,
    /// ⚠️ **EL ANCLA DE LA HISTORIA** (§294): la pareja del MMR de la
    /// ultima cabeza v3 cuyo digest se RECOMPUSO. Hex tal cual del cable.
    ///
    /// ⚠️ Solo se fija tras recomponer: anclar una cima que la firma **no
    /// cubre** no ancla nada.
    pareja: Option<(String, u64)>,
    /// El camino pedido y aun no juzgado.
    pendiente: Option<Pendiente>,
}

impl Memoria {
    pub fn nueva() -> Self {
        Self::default()
    }
    pub fn vistos(&self) -> usize {
        self.vistos.len()
    }
    pub fn clave_fijada(&self) -> Option<&str> {
        self.clave_fijada.as_deref()
    }
    /// La pareja del MMR custodiada: `(mmrRoot hex, mmrSize)`.
    pub fn pareja(&self) -> Option<(&str, u64)> {
        self.pareja.as_ref().map(|(c, t)| (c.as_str(), *t))
    }
    /// ⚠️ Se pide camino cuando **hay de donde partir y no hay nada
    /// pendiente**: dos peticiones por latido contra un nodo que mide
    /// 0,255 ms fijos (§217) no cuestan nada, y el caso identidad tiene
    /// respuesta declarada (camino vacio).
    ///
    /// ⚠⚠ **Nunca con t=0**: el servicio niega ese `oldSize` POR DISENO
    /// («no hay historia que extender»), asi que pedirlo solo llena el
    /// diario de negativas que no dicen nada — siete seguidas en la
    /// primera corrida del banco del §295.
    pub fn debe_pedir_camino(&self) -> bool {
        self.pendiente.is_none() && self.pareja.as_ref().map_or(false, |(_, t)| *t > 0)
    }

    /// Fija la clave la primera vez, y **la compara siempre después**.
    ///
    /// ⚠️ Esto es el ancla. No protege el primer encuentro —si el operador
    /// ya mentía, TOFU fija la mentira— pero **acota la ventana a un
    /// instante** en vez de dejarla abierta.
    pub fn anclar(&mut self, clave: &str) -> Option<Veredicto> {
        match &self.clave_fijada {
            None => {
                self.clave_fijada = Some(clave.to_string());
                None
            }
            Some(f) if f == clave => None,
            Some(f) => Some(Veredicto::CambioDeClave {
                fijada: f.clone(),
                recibida: clave.to_string(),
            }),
        }
    }

    /// Clasifica una cabeza recién llegada. **No verifica la firma.**
    ///
    /// ⚠️ **MIRA EL ÍNDICE, NO EL DIGEST**, y eso importa: un operador
    /// **sin tráfico** emite cabezas consecutivas con **el mismo
    /// contenido** —nada cambió en el ledger—, y eso es **legítimo**. Si
    /// esta función mirara el digest, marcaría `VistaDividida` sobre un
    /// operador honesto: **el falso positivo más peligroso del testigo**,
    /// porque desacreditaría a quien no hizo nada malo.
    ///
    /// ⚠️ **Confirmado en L.3 (§251), y NO antes**: L.1 corrió con el
    /// ledger vacío y **un solo digest para nueve índices**, así que esa
    /// distinción **no existía y no pudo comprobarse**. L.3 fabricó dos
    /// pares de índices consecutivos con digest repetido —una ventana
    /// deliberadamente quieta— y los dos salieron `Nueva`.
    pub fn clasificar(&mut self, indice: u64, digest: &str) -> Veredicto {
        if let Some(previo) = self.vistos.get(&indice) {
            if previo == digest {
                return Veredicto::Repetida { indice };
            }
            // ⚠️⚠️ AQUI. Mismo indice de una clave de un solo uso, contenido
            // distinto. DETECTAR NO ES DISTINGUIR (§110.3): no se sabe si fue
            // un reinicio con el estado de travesia mal restaurado o una
            // vista dividida deliberada. Es OPONIBLE de todos modos.
            return Veredicto::VistaDividida {
                indice,
                digest_a: previo.clone(),
                digest_b: digest.to_string(),
            };
        }
        let hueco = match self.ultimo {
            Some(u) if indice > u + 1 => Some((u + 1, indice - 1)),
            _ => None,
        };
        self.vistos.insert(indice, digest.to_string());
        self.ultimo = Some(indice);
        match hueco {
            // ⚠️ §242 DECLARO que habra huecos. Anotar y SEGUIR; si esto
            // gritara, el primer reinicio quemaria la credibilidad.
            Some((desde, hasta)) => Veredicto::Hueco { desde, hasta },
            None => Veredicto::Nueva { indice, digest: digest.to_string() },
        }
    }
}

/// Una vuelta: ancla, verifica y clasifica.
///
/// ⚠️ **El orden importa.** El ancla va **antes** de verificar: si la clave
/// cambió, verificar contra la nueva **no significa nada**.
pub fn una_vuelta(v: &Value, m: &mut Memoria) -> Veredicto {
    if v["available"].as_bool() != Some(true) {
        return Veredicto::SinFirma {
            motivo: v["reason"].as_str().unwrap_or("sin motivo declarado").into(),
        };
    }
    let clave_hex = v["publicKey"].as_str().unwrap_or_default().to_string();
    // ── 1 · EL ANCLA, antes que nada ──
    if let Some(cambio) = m.anclar(&clave_hex) {
        return cambio;
    }
    // ── 2 · AHORA el cable tipa lo servido, y NO ANTES ──
    //
    // ⚠️⚠️ El orden de estas dos etapas es una PROPIEDAD, no una comodidad:
    // un parseo estructural delante del ancla dejaría que el operador
    // ENMASCARE un cambio de clave malformando cualquier otro campo —la
    // vuelta moriría por forma y nunca llegaría a comparar la clave—. Por eso
    // el ancla de arriba lee `publicKey` a pelo, a propósito.
    //
    // ⚠️ §312 · esto es MÁS ESTRICTO que lo anterior, y es cambio de
    // comportamiento DECLARADO: una respuesta a la que le falte `custody` o
    // `beatSeconds` clasificaba como cabeza buena y ahora sale `NoVerifica`
    // con la forma nombrada. El dispatch del nodo sirve esos campos en las
    // TRES formas, así que quien no los sirve está roto.
    let dto = match SignedEpochHeadDto::deserialize(v) {
        Ok(d) => d,
        Err(e) => {
            return Veredicto::NoVerifica { indice: 0, error: format!("forma del cable: {e}") }
        }
    };
    let vista = match dto.firmada() {
        Ok(Some(c)) => c,
        // Inalcanzable por la salida temprana de arriba. No se colapsa con un
        // `unwrap`: el tipo lo contempla, y fingir que no existe es como se
        // entra en los pánicos que un testigo no puede permitirse.
        Ok(None) => {
            return Veredicto::SinFirma {
                motivo: "el cable dice que no hay cabeza firmada".into(),
            }
        }
        Err(e) => return Veredicto::NoVerifica { indice: 0, error: format!("{e}") },
    };
    let indice = vista.index.0;
    // ⚠️ El digest se lee de lo SERVIDO y no se recompone desde la vista:
    // `m.clasificar` compara esta cadena entre vueltas, y devolver los bytes
    // a hex sería REESCRIBIR lo que el nodo sirvió. Quien reescribe, adultera.
    let digest = v["epochDigest"].as_str().unwrap_or_default().to_string();
    // ── 3 · verificar, con el MISMO codigo que usa el firmante ──
    match verificar(v) {
        Err(e) => Veredicto::NoVerifica { indice, error: e },
        Ok(()) => m.clasificar(indice, &digest),
    }
}

/// Verifica con **`zk-ssl-verify`**: el testigo **no reimplementa la
/// verificación**, usa la misma que el firmante.
fn verificar(v: &Value) -> Result<(), String> {
    let firma = leer_hex(&v["signature"])?;
    let clave = leer_hex(&v["publicKey"])?;
    let d = leer_hex(&v["epochDigest"])?;
    if d.len() != 32 {
        return Err(format!("epochDigest de {} bytes", d.len()));
    }
    let mut digest = [0u8; 32];
    digest.copy_from_slice(&d);
    let c = CabezaFirmada {
        version_formato: leer_q(&v["formatVersion"])? as u8,
        indice: 0, // no entra en el preámbulo
        firma,
    };
    verificar_cabeza(&clave, &digest, &c).map_err(|e| format!("{e}"))?;
    // ── §294 · Y EL DIGEST NO SE CREE: SE RECOMPONE ──
    recomponer(v, &digest)
}

/// **Recompone el `epochDigest` de los campos servidos** y lo compara con
/// el que la firma cubre.
///
/// ⚠️⚠️ **EL HUECO QUE ESTO CIERRA.** Hasta §294 el testigo comprobaba que
/// la firma cubria un digest, **no que los campos lo COMPUSIERAN** — y el
/// nodo esta escrito suponiendo lo contrario: *«el testigo custodia
/// campos+digest+firma juntos y recompone sin volver a llamar»* (§275, en
/// el dispatch de `zkssl_signedEpochHead`). Declarado y no hecho: **la
/// misma familia que el hueco DTO-vs-payload de §292**.
///
/// ⚠️ Y no es higiene para el tramo (i): sin recomponer, `mmrRoot` y
/// `mmrSize` son campos **que la firma no cubre**, y anclarlos seria anclar
/// lo que el operador quiera.
///
/// ⚠️ **La version elige recompositor** (§292), como en el mando de §289:
/// v3 con la pareja del MMR, v2 sin ella. Otra version no se recompone
/// aqui — se verifica con la biblioteca, no con el testigo.
fn recomponer(v: &Value, firmado: &[u8; 32]) -> Result<(), String> {
    let version = leer_q(&v["formatVersion"])?;
    if version != 2 && version != 3 {
        return Ok(());
    }
    let b32 = |k: &str| -> Result<[u8; 32], String> {
        let b = leer_hex(&v[k])?;
        b.as_slice()
            .try_into()
            .map_err(|_| format!("{k}: {} bytes, se esperaban 32", b.len()))
    };
    // ⚠️ Macro y no closure: un closure tendria que NOMBRAR `Digest` en su
    // tipo de retorno, y `Digest` no lo reexporta el verificador (D5). La
    // macro deja que la inferencia lo resuelva en el punto de uso.
    macro_rules! dg {
        ($k:expr) => {
            mmr::hoja_desde_bytes(&b32($k)?)
                .ok_or_else(|| format!("{}: no es un digest del campo", $k))?
        };
    }
    let (seq, n) = (leer_q(&v["seq"])?, leer_q(&v["n"])?);
    let (accounts, pending) = (dg!("accountsRoot"), dg!("pendingRoot"));
    let (frozen, chain) = (dg!("frozenRoot"), dg!("chainDigest"));
    let acuses = dg!("acusesRoot");
    let compuesto = if version == 3 {
        zk_ssl_verify::epoch_digest_v3(
            seq,
            accounts,
            pending,
            frozen,
            chain,
            acuses,
            n,
            dg!("mmrRoot"),
            leer_q(&v["mmrSize"])?,
        )
    } else {
        zk_ssl_verify::epoch_digest_v2(seq, accounts, pending, frozen, chain, acuses, n)
    };
    let esperado = mmr::hoja_desde_bytes(firmado)
        .ok_or_else(|| "epochDigest: no es un digest del campo".to_string())?;
    if compuesto != esperado {
        return Err(format!(
            "los campos NO recomponen el epochDigest firmado (v{version}): \
             o el servido esta adulterado o la cabeza nunca fue esa"
        ));
    }
    Ok(())
}

/// ⚠️ El cable usa cantidades **hexadecimales** (`{:#x}`), no decimales.
fn leer_q(v: &Value) -> Result<u64, String> {
    let s = v.as_str().ok_or("QUANTITY no es cadena")?;
    u64::from_str_radix(s.trim_start_matches("0x"), 16).map_err(|e| e.to_string())
}

fn leer_hex(v: &Value) -> Result<Vec<u8>, String> {
    let s = v.as_str().ok_or("no es cadena hex")?;
    let h = s.trim_start_matches("0x");
    if h.len() % 2 != 0 {
        return Err("hex de longitud impar".into());
    }
    (0..h.len() / 2)
        .map(|i| u8::from_str_radix(&h[i * 2..i * 2 + 2], 16).map_err(|e| e.to_string()))
        .collect()
}

// ═══════════════════════════════════════════════════════════════════════
//  §249 · LEER el diario: auditarlo y compararlo
//
//  ⚠️ §248 eligió los campos del diario con este criterio: **lo suficiente
//  para que un tercero reverifique sin el nodo**. Eso es una afirmación
//  sobre un uso futuro, y **nada la ejercitaba**: el diario se escribía y
//  nadie lo leía jamás. Los cuatro tests comprobaban **la forma de una
//  línea en memoria**, que es otra cosa.
//
//  > El formato estaba **declarado, no terminado**.
//
//  ⚠️ **Esto NO crea el segundo testigo.** Hace que su trabajo sea posible,
//  y prueba que el diario sirve para lo que dijo servir. Es la misma forma
//  que §245 y §248: el proyecto no puede construir la confianza, puede
//  **quitar la excusa de que no hay cómo**.
// ═══════════════════════════════════════════════════════════════════════

/// Lo que la auditoría de un diario encuentra.
///
/// ⚠️ **Tres cosas distintas con tres significados distintos**, como en
/// §246: una firma que no verifica, un índice que retrocede y una clave que
/// cambia no son el mismo problema. El vocabulario se mantiene coherente
/// con el de [`Veredicto`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Hallazgo {
    /// La firma guardada **no valida contra su propio digest**.
    FirmaNoVerifica { linea: usize, indice: u64, error: String },
    /// El índice **retrocede**: un diario solo puede avanzar.
    IndiceRetrocede { linea: usize, previo: u64, ahora: u64 },
    /// ⚠️ **La clave cambia a mitad del diario.** Es el caso que §244 dejó
    /// abierto —el anclaje— y **el auditor es quien puede verlo**.
    CambioDeClave { linea: usize, fijada: String, recibida: String },
    /// Una versión de formato que este binario no conoce.
    VersionDesconocida { linea: usize, v: u64 },
    /// La línea no es JSON, o le faltan campos.
    LineaIlegible { linea: usize, error: String },
    /// ⚠️⚠️ **El mismo índice con dos digests, DENTRO DEL MISMO DIARIO.**
    ///
    /// Hasta §250 el auditor **no lo veía**: comprobaba que el índice no
    /// retrocediera, pero no que llevara siempre el mismo contenido. Un
    /// diario que **contiene la prueba de una vista dividida** pasaba
    /// limpio — y en un artefacto cuyo propósito es ser oponible, **un
    /// falso «limpio» es peor que no tener herramienta**, porque alguien
    /// lo enseñaría como prueba de que no pasó nada.
    ///
    /// El testigo la caza **en vivo**, sí — pero **solo si estaba
    /// corriendo**. Un diario que llega de un tercero se comprueba en
    /// frío.
    VistaDividida { linea: usize, indice: u64, digest_a: String, digest_b: String },
}

impl Hallazgo {
    pub fn clase(&self) -> &'static str {
        match self {
            Hallazgo::FirmaNoVerifica { .. } => "firma-no-verifica",
            Hallazgo::IndiceRetrocede { .. } => "indice-retrocede",
            Hallazgo::CambioDeClave { .. } => "cambio-de-clave",
            Hallazgo::VersionDesconocida { .. } => "version-desconocida",
            Hallazgo::LineaIlegible { .. } => "linea-ilegible",
            // ⚠️ El MISMO nombre que en `Veredicto`: que la misma cosa se
            // llame igual en las tres herramientas es lo que permite
            // hablar de ella sin ambiguedad.
            Hallazgo::VistaDividida { .. } => "vista-dividida",
        }
    }
}

/// Un mismo índice con dos contenidos distintos, en dos diarios.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Divergencia {
    pub indice: u64,
    pub digest_a: String,
    pub digest_b: String,
    pub misma_clave: bool,
}

/// Resultado de auditar: cuántas líneas, cuántas **reverificadas**, y qué.
#[derive(Debug, Default)]
pub struct Auditoria {
    pub lineas: usize,
    pub con_firma: usize,
    pub reverificadas: usize,
    pub hallazgos: Vec<Hallazgo>,
}

/// **Relee un diario y lo reverifica SIN EL NODO.**
///
/// ⚠️ Esto es lo que convierte el criterio de §248 en algo **ejecutable**.
/// Un tercero que reciba un diario dentro de seis meses tiene que poder
/// decir *«esto verifica»* con una orden; si para eso hiciera falta escribir
/// un programa, **el formato no estaría terminado**.
///
/// ⚠️ Reutiliza [`verificar`] **tal cual**, y eso no es casualidad: §248
/// guardó los campos **como vinieron del cable**. Si los hubiera
/// transformado, el auditor tendría que reimplementar la verificación — y
/// dos implementaciones pueden discrepar.
pub fn auditar_lineas(lineas: &[String]) -> Auditoria {
    let mut a = Auditoria::default();
    let mut ultimo: Option<u64> = None;
    let mut clave: Option<String> = None;
    let mut vistos: BTreeMap<u64, String> = BTreeMap::new();

    for (i, l) in lineas.iter().enumerate() {
        let n = i + 1;
        if l.trim().is_empty() {
            continue;
        }
        a.lineas += 1;
        let v: Value = match serde_json::from_str(l) {
            Ok(v) => v,
            Err(e) => {
                a.hallazgos.push(Hallazgo::LineaIlegible { linea: n, error: e.to_string() });
                continue;
            }
        };
        match v["v"].as_u64() {
            Some(x) if x <= DIARIO_VERSION as u64 => {}
            Some(x) => {
                a.hallazgos.push(Hallazgo::VersionDesconocida { linea: n, v: x });
                continue;
            }
            None => {
                a.hallazgos.push(Hallazgo::LineaIlegible {
                    linea: n,
                    error: "sin campo `v`: no declara su formato".into(),
                });
                continue;
            }
        }
        // Las lineas sin cabeza firmada son legitimas y no se reverifican.
        if v["signature"].is_null() {
            continue;
        }
        a.con_firma += 1;

        let indice = match leer_q(&v["index"]) {
            Ok(x) => x,
            Err(e) => {
                a.hallazgos.push(Hallazgo::LineaIlegible { linea: n, error: e });
                continue;
            }
        };
        // ⚠️ Un diario SOLO PUEDE AVANZAR: es de solo añadir (§248).
        if let Some(u) = ultimo {
            if indice < u {
                a.hallazgos.push(Hallazgo::IndiceRetrocede { linea: n, previo: u, ahora: indice });
            }
        }
        ultimo = Some(ultimo.map_or(indice, |u| u.max(indice)));

        // ⚠️⚠️ EL MISMO INDICE CON DOS DIGESTS. Un indice REPETIDO es
        // normal —el testigo anota `nueva` y luego `repetida`, y las dos
        // llevan la cabeza—; lo que NO puede pasar es que el contenido
        // cambie. Hasta §250 esto no se miraba.
        let dg = v["epochDigest"].as_str().unwrap_or_default().to_string();
        match vistos.get(&indice) {
            None => {
                vistos.insert(indice, dg);
            }
            Some(previo) if *previo == dg => {}
            Some(previo) => {
                a.hallazgos.push(Hallazgo::VistaDividida {
                    linea: n,
                    indice,
                    digest_a: previo.clone(),
                    digest_b: dg,
                });
            }
        }

        // ⚠️ LA CLAVE FIJADA, no solo las firmas: un diario donde la clave
        // cambia a mitad es el caso que §244 dejo abierto, y NO PUEDE PASAR
        // EN SILENCIO.
        let k = v["publicKey"].as_str().unwrap_or_default().to_string();
        match &clave {
            None => clave = Some(k),
            Some(f) if *f == k => {}
            Some(f) => {
                a.hallazgos.push(Hallazgo::CambioDeClave {
                    linea: n,
                    fijada: f.clone(),
                    recibida: k,
                });
            }
        }

        match verificar(&v) {
            Ok(()) => a.reverificadas += 1,
            Err(e) => a.hallazgos.push(Hallazgo::FirmaNoVerifica { linea: n, indice, error: e }),
        }
    }
    a
}

/// **Compara dos diarios**: el mismo índice con distinto contenido.
///
/// ⚠️ Con un solo operador esto **no prueba nada** —dos diarios míos son un
/// diario con dos ficheros—. Es la pieza que **activa** la propiedad de
/// §248 cuando exista un segundo testigo, y sin ella ese testigo no tendría
/// con qué.
///
/// ⚠️ Y detecta la divergencia, **no dice cuál miente**. No hace falta: lo
/// que queda probado es que **el operador emitió dos cosas distintas para el
/// mismo índice**, y ninguno de los dos testigos pudo fabricar su firma.
/// Resultado de comparar dos diarios.
///
/// ⚠️ **Una vista dividida INTERNA no es una divergencia entre A y B.** Si
/// un diario contiene el mismo índice con dos digests, el problema está
/// **dentro de ese fichero** y se ve con `--auditar`. Presentarlo como
/// «A difiere de B» confundiría al que mira.
#[derive(Debug, Default)]
pub struct Comparacion {
    pub divergencias: Vec<Divergencia>,
    /// El diario A se contradice a sí mismo.
    pub interna_a: bool,
    /// El diario B se contradice a sí mismo.
    pub interna_b: bool,
}

/// **Compara dos diarios**: el mismo índice con distinto contenido.
///
/// ⚠️ Con un solo operador esto **no prueba nada** —dos diarios míos son un
/// diario con dos ficheros—. Es la pieza que **activa** la propiedad de
/// §248 cuando exista un segundo testigo.
///
/// ⚠️ Y detecta la divergencia, **no dice cuál miente**. No hace falta: lo
/// que queda probado es que **el operador emitió dos cosas distintas para el
/// mismo índice**, y ninguno de los dos testigos pudo fabricar su firma.
///
/// ⚠️⚠️ **SE QUEDA CON LA PRIMERA OCURRENCIA DE CADA ÍNDICE.** Hasta §250
/// usaba `insert`, que **sobrescribe**: como el testigo anota `nueva` y
/// luego `repetida` —y las dos llevan la cabeza—, **la última línea tapaba
/// a la primera** y una manipulación de la primera desaparecía del mapa.
/// Lo encontró el banco L.2, no los tests.
pub fn comparar_lineas(a: &[String], b: &[String]) -> Comparacion {
    // Devuelve (mapa con la PRIMERA de cada indice, se_contradice).
    let mapa = |ls: &[String]| -> (BTreeMap<u64, (String, String)>, bool) {
        let mut m: BTreeMap<u64, (String, String)> = BTreeMap::new();
        let mut interna = false;
        for l in ls {
            let v: Value = match serde_json::from_str(l) {
                Ok(v) => v,
                Err(_) => continue,
            };
            if v["signature"].is_null() {
                continue;
            }
            if let Ok(i) = leer_q(&v["index"]) {
                let par = (
                    v["epochDigest"].as_str().unwrap_or_default().to_string(),
                    v["publicKey"].as_str().unwrap_or_default().to_string(),
                );
                match m.get(&i) {
                    None => {
                        m.insert(i, par);
                    }
                    // ⚠️ Repetir un indice es NORMAL; cambiar su digest, no.
                    Some(p) if p.0 == par.0 => {}
                    Some(_) => interna = true,
                }
            }
        }
        (m, interna)
    };
    let ((ma, ia), (mb, ib)) = (mapa(a), mapa(b));
    let mut c = Comparacion { interna_a: ia, interna_b: ib, ..Default::default() };
    for (i, (da, ka)) in &ma {
        if let Some((db, kb)) = mb.get(i) {
            if da != db {
                c.divergencias.push(Divergencia {
                    indice: *i,
                    digest_a: da.clone(),
                    digest_b: db.clone(),
                    misma_clave: ka == kb,
                });
            }
        }
    }
    c
}

/// **La comprobación DIRIGIDA**: índices del testigo ausentes del diario
/// del nodo.
///
/// ⚠️ **Y sólo esa dirección.** El diario del nodo es COMPLETO —uno por
/// latido— y el del testigo es MUESTREADO —sólo lo que pidió—, así que el
/// nodo tendrá siempre líneas que el testigo no tiene y eso **no es un
/// hallazgo, es lo normal**. Contarlo daría rojo en cada corrida y
/// acabaría siendo paisaje.
///
/// La ausencia que sí importa significa una de dos cosas, y las dos son
/// graves: **o el nodo firmó algo que no recuerda, o alguien sirvió una
/// firma que el nodo no emitió**.
///
/// ⚠️ Esto es lo que `comparar_lineas` NO hace: mapea por `index` y sólo
/// recorre los presentes en ambos, así que una ausencia le pasa en
/// silencio.
pub fn ausentes(testigo: &[String], nodo: &[String]) -> Vec<u64> {
    let indices = |ls: &[String]| -> std::collections::BTreeSet<u64> {
        let mut s = std::collections::BTreeSet::new();
        for l in ls {
            let v: Value = match serde_json::from_str(l) {
                Ok(v) => v,
                Err(_) => continue,
            };
            if v["signature"].is_null() {
                continue;
            }
            if let Some(t) = v["index"].as_str() {
                if let Ok(i) = u64::from_str_radix(t.trim_start_matches("0x"), 16) {
                    s.insert(i);
                }
            }
        }
        s
    };
    let (t, n) = (indices(testigo), indices(nodo));
    t.difference(&n).copied().collect()
}

#[derive(Args)]
pub struct WitnessArgs {
    /// Nodo a atestiguar.
    #[arg(long, default_value = "http://127.0.0.1:8545")]
    nodo: String,

    /// Segundos entre consultas. Por defecto, la cadencia del latido.
    ///
    /// ⚠️ 1.440 peticiones al día contra un nodo que mide **0,255 ms fijos
    /// por petición** (§217): **atestiguar no cuesta rendimiento**.
    #[arg(long, default_value_t = 60)]
    cada: u64,

    /// Cuántas consultas hacer. `0` = hasta que algo detenga al testigo.
    #[arg(long, default_value_t = 0)]
    veces: u64,

    /// Diario, una línea JSON por consulta.
    #[arg(long)]
    diario: Option<PathBuf>,

    /// **Relee un diario y lo reverifica SIN EL NODO** (§249).
    ///
    /// ⚠️ Es lo que convierte el criterio de §248 —*lo suficiente para
    /// que un tercero reverifique sin el nodo*— en algo **ejecutable**.
    #[arg(long, value_name = "DIARIO", conflicts_with = "comparar")]
    auditar: Option<PathBuf>,

    /// **Compara dos diarios**: el mismo índice con distinto contenido.
    ///
    /// ⚠️ Con un solo operador **no prueba nada**. Es la pieza que activa
    /// la propiedad de §248 cuando exista un segundo testigo.
    #[arg(long, value_name = "DIARIO", num_args = 2)]
    comparar: Option<Vec<PathBuf>>,

    /// **Indices que el TESTIGO tiene y el NODO no** (nota 80): la
    /// comprobacion dirigida sobre dos diarios.
    ///
    /// ⚠️ DIRECCIONAL: el diario del testigo primero, el del nodo despues.
    /// Al reves no es un hallazgo —el diario del nodo es completo y el del
    /// testigo, muestreado—.
    ///
    /// ⚠️ Un testigo que opera el propio operador NO prueba nada: este
    /// mando vale cuando el diario del testigo lo custodia un tercero.
    #[arg(long, num_args = 2, value_names = ["TESTIGO", "NODO"],
          conflicts_with_all = ["auditar", "comparar"])]
    ausentes: Option<Vec<PathBuf>>,

    /// **Cofirma cada cabeza que EXTIENDE** (§300). Ruta a la semilla.
    ///
    /// ⚠️⚠️ **La semilla es material de clave.** De donde sale y quien la
    /// guarda es **decision de despliegue, y no esta tomada** (nota 92).
    ///
    /// ⚠️ Activar esto convierte al testigo en **parte interesada**: hereda
    /// las mismas obligaciones que audita —guardian del indice, custodia
    /// declarada, transicion firmada al rotar (nota 84)— y **duplica la
    /// superficie de las notas 84, 92 y 19**.
    /// ⚠️ **Formato: 96 bytes en BINARIO crudo.** El nodo lee su semilla en
    /// HEX (`--clave-fichero`); este mando NO. Los dos formatos son del proyecto
    /// (§301) y desde el §330 el error dice cual has confundido.
    #[arg(long, value_name = "SEMILLA", requires_all = ["indice_cofirma", "cofirmas"])]
    cofirmar: Option<PathBuf>,

    /// El contador del guardian del testigo, **su propio fichero**.
    ///
    /// ⚠️ NO se comparte con el del nodo: son dos claves distintas y dos
    /// series distintas. Compartirlo seria reusar indices entre firmantes.
    #[arg(long, value_name = "CONTADOR")]
    indice_cofirma: Option<PathBuf>,

    /// Donde se anotan las cofirmas, **una linea JSON por cofirma**.
    ///
    /// ⚠️ **Fichero PROPIO, no el diario** (§300): una firma XMSS son ~18 KB
    /// en hex sobre una linea de diario que ya pesa ~37 KB, y sobre todo
    /// **son artefactos con destinatarios distintos** — el diario es del
    /// testigo para si mismo; la cofirma es **para terceros**. En el diario
    /// queda solo una marca que las ata.
    #[arg(long, value_name = "COFIRMAS")]
    cofirmas: Option<PathBuf>,

    /// **Envia cada cofirma al nodo** (§316), con `zkssl_submitCosig`.
    ///
    /// ⚠️⚠️ **SEGUNDA escalada, y por eso lleva flag propio y nace
    /// APAGADO.** `--cofirmar` ya convierte al testigo en parte interesada;
    /// esto ademas **publica su clave publica ante el operador al que
    /// vigila**. Un testigo que enviara por defecto perderia la posibilidad
    /// de ser observador ANONIMO que publica su evidencia por otro canal —
    /// que es justo lo que `--verificar-cofirmas` le permite a un tercero
    /// sin tocar el nodo.
    ///
    /// ⚠️ Un envio que falla **NO detiene al testigo**: se imprime y se
    /// sigue. Vigilar es lo primero, avalar lo segundo y publicar el aval lo
    /// tercero. **No se anota en el diario**: eso subiria `DIARIO_VERSION`,
    /// y eso es corte propio.
    #[arg(long, requires_all = ["cofirmar"])]
    enviar_cofirmas: bool,

    /// **Relee un fichero de cofirmas y las verifica** (§301).
    ///
    /// ⚠️ Es lo que convierte en EJECUTABLE la autosuficiencia que el §300
    /// declaro: sin esto, «la linea basta por si sola para un tercero» era
    /// una propiedad que **nadie podia ejercer**. Mismo papel que
    /// `--auditar` (§249) para el diario.
    ///
    /// ⚠️ No necesita el nodo, ni el diario, ni al testigo: **solo el
    /// fichero**. Cada linea trae la clave del testigo, la del operador, el
    /// digest y la firma.
    #[arg(long, value_name = "COFIRMAS",
          conflicts_with_all = ["auditar", "comparar", "ausentes", "cofirmar"])]
    verificar_cofirmas: Option<PathBuf>,

    /// **La politica que NOMBRA que testigos valen** (S319).
    ///
    /// Un fichero de texto: una clave publica de testigo en hexadecimal
    /// por linea. Se ignoran las lineas en blanco y las que empiezan por
    /// almohadilla.
    ///
    /// ⚠️⚠️ **Sin esto, `--verificar-cofirmas` comprueba COHERENCIA
    /// INTERNA y no autenticidad**: las dos claves viajan DENTRO de la
    /// cofirma, asi que cualquiera fabrica una que verifica con SU clave.
    /// Lo que convierte "verifica" en "vale" es que el cliente haya
    /// NOMBRADO antes que claves acepta.
    ///
    /// ⚠️ **Entra por FUERA a proposito**: no puede venir del nodo -que
    /// es el operador- ni del paquete de evidencia -que lo entrega el
    /// operador-. Quien nombra es el cliente, o no es politica. El
    /// contrato publicado ya lo decia y hasta aqui no se cumplia.
    #[arg(long, value_name = "TESTIGOS", requires_all = ["verificar_cofirmas"])]
    testigos: Option<PathBuf>,

    /// **Cuantas cofirmas de testigos NOMBRADOS hacen falta** (S319).
    ///
    /// ⚠️ El valor por defecto es **1**, y esta DECLARADO, no medido: la
    /// k que de verdad vale depende de a cuantos testigos independientes
    /// tenga acceso el cliente, y eso no lo sabe el proyecto. Lo que si es
    /// del proyecto es la forma: **por debajo de k la cabeza NO se acepta**
    /// y el mando sale con error. Falla cerrada.
    #[arg(long, value_name = "K", requires_all = ["testigos"])]
    k: Option<usize>,

    /// **RECOLECTA del nodo las cofirmas que otros le enviaron** (§320) y
    /// las añade al fichero, en el formato que lee `--verificar-cofirmas`.
    ///
    /// ⚠️ Lo que se añade **NO está verificado**: `zkssl_cosigs` en el nodo
    /// *«no verifica, sólo filtra»*. Quien comprueba es `--verificar-cofirmas`,
    /// y quien decide si valen es la política de `--testigos` (§319).
    ///
    /// ⚠️ El nodo guarda **una sola época** (§317), así que esto trae una
    /// época por vez: **acumular es trabajo de este mando**, corriéndolo más
    /// de una vez. Declarado y no hecho: pedir una época **por su nombre**,
    /// que el nodo ya sabe servir.
    #[arg(long, value_name = "COFIRMAS")]
    recolectar_cofirmas: Option<PathBuf>,

}

/// La linea del fichero de cofirmas. **AUTOSUFICIENTE**: lleva todo lo que
/// [`zk_ssl_verify::verificar_cofirma`] consume y **nada mas**.
///
/// ```text
/// verificar_cofirma(clave_del_testigo, epoch_digest, clave_del_operador, c)
///                          |                 |               |          |
///     clavePublicaTestigo <-+   epochDigest <-+               |          |
///                     clavePublicaOperador <------------------+          |
///          CabezaFirmada{ versionFormato, indice, firma } <--------------+
/// ```
///
/// ⚠️⚠️ **El DOMINIO no se escribe aqui, y es la decision del corte.** Ya
/// viaja **dentro del preambulo firmado**, puesto por `preambulo_cofirma`:
/// ponerlo tambien en el JSON serian **dos marcadores que pueden discrepar**
/// (§236, el mismo argumento que mantiene la version fuera del nombre del
/// dominio). Como efecto, la ceguera (e) de la nota 94 —`LIT_ZK` no ve los
/// dominios escritos como cadena JSON— **NO gana un caso vivo aqui**, y no
/// por suerte sino porque escribirlo habria sido un error aparte.
///
/// ⚠️ Todo en hex tal cual, sin transcodificar (§248).
pub fn linea_de_cofirma(
    epoch_digest: &[u8; 32],
    clave_del_operador: &[u8],
    clave_del_testigo: &[u8],
    c: &CabezaFirmada,
    visto_unix: u64,
) -> Value {
    let hx = |b: &[u8]| format!("0x{}", b.iter().map(|x| format!("{x:02x}")).collect::<String>());
    json!({
        "v": COFIRMA_VERSION,
        "epochDigest": hx(epoch_digest),
        "clavePublicaOperador": hx(clave_del_operador),
        "clavePublicaTestigo": hx(clave_del_testigo),
        "versionFormato": format!("{:#x}", c.version_formato),
        "indice": format!("{:#x}", c.indice),
        "firma": hx(&c.firma),
        "vistoUnix": visto_unix,
    })
}

/// **La MISMA cofirma, en convencion de CABLE** (§316).
///
/// ⚠️⚠️ Toma **los mismos cinco argumentos** que `linea_de_cofirma` y se
/// alimenta **del mismo punto de llamada**: son dos productores del mismo
/// contrato. El doc de `CofirmaDto` dejo escrito que lo que los ata es un
/// test sobre el **CONJUNTO de claves**, no sobre la representacion
/// —unificarlos exigiria subir `COFIRMA_VERSION`, que es corte propio— y
/// que ese test **vive donde esten los dos productores**. Es aqui, y esta
/// abajo.
///
/// ⚠️ El fichero JSONL del testigo **no cambia**: `linea_de_cofirma`
/// sigue emitiendo lo suyo, con su hex y sus cantidades como cadena.
fn cofirma_dto(
    epoch_digest: &[u8; 32],
    clave_del_operador: &[u8],
    clave_del_testigo: &[u8],
    c: &CabezaFirmada,
    visto_unix: u64,
) -> CofirmaDto {
    // ⚠️ Alcance minimo A PROPOSITO: `Q`, `B32` y `Blob` son nombres de
    //    una letra o tres y este fichero tiene casi tres mil lineas.
    use zk_ssl_wire::{Blob, B32, Q};
    CofirmaDto {
        v: Q(COFIRMA_VERSION),
        epoch_digest: B32(*epoch_digest),
        clave_publica_operador: Blob(clave_del_operador.to_vec()),
        clave_publica_testigo: Blob(clave_del_testigo.to_vec()),
        version_formato: Q(c.version_formato as u64),
        indice: Q(c.indice),
        firma: Blob(c.firma.clone()),
        visto_unix: Q(visto_unix),
    }
}

/// Lo que un tercero puede encontrar mal en un fichero de cofirmas.
#[derive(Debug, PartialEq, Eq)]
pub enum HallazgoCofirma {
    Ilegible { linea: usize, error: String },
    VersionDesconocida { linea: usize, v: u64 },
    CampoAusente { linea: usize, campo: String },
    /// La firma no verifica, o verifica sobre OTRA cosa.
    NoVerifica { linea: usize, indice: u64, error: String },
    /// ⚠️⚠️ **DOS COFIRMAS DEL MISMO TESTIGO CON EL MISMO INDICE.** Es lo
    /// que el guardian existe para impedir: un indice XMSS reutilizado
    /// **filtra la clave** (curva QRL: a la cuarta repeticion, ~2^18 hashes).
    /// Un tercero que recibe el fichero tiene que poder verlo **sin
    /// preguntarle a nadie**.
    ///
    /// ⚠️ **La serie es POR TESTIGO** (§310): cada cofirmante lleva su propia
    /// clave y su propio contador —lo declara `--indice-cofirma`: «son dos
    /// claves distintas y dos series distintas»—, asi que en un fichero que
    /// junte cofirmas de VARIOS el mismo indice aparece por diseno. Por eso
    /// `testigo` viaja dentro del hallazgo: sin el, «indice repetido» no dice
    /// DE QUIEN (§254).
    ///
    /// ⚠️⚠️ **EL NUMERO QUE MIRA ES EL EMBEBIDO EN LA FIRMA** (§333), no el
    /// ordinal declarado en la linea. El declarado es metadato que la firma
    /// NO acredita —lo dice el doc de `CabezaFirmada` y lo ato el §332—, asi
    /// que indexar por el dejaba pasar el REINICIO: al perderse el SK la
    /// clave vuelve a cero y el firmante emite ordinales NUEVOS con indices
    /// de hoja ya gastados. El embebido no se falsea sin romper la firma.
    ///
    /// ⚠️⚠️ **Y EL MENSAJE TIENE QUE SER DISTINTO** (§334). Repetir el
    /// indice con el MISMO preambulo -version, epochDigest y clave del
    /// operador- es la misma firma sobre el mismo mensaje: no revela nada.
    /// Levantarlo como hallazgo dejaba desacreditar a un cofirmante honesto
    /// duplicandole una linea. Lo que filtra la clave son DOS mensajes.
    IndiceRepetido { linea: usize, indice: u64, antes: usize, testigo: String },
    /// ⚠️⚠️ **EL ORDINAL DECLARADO NO CUADRA CON EL QUE VA DENTRO DE LA
    /// FIRMA** (§333). El §332 ato los dos numeros al final de
    /// `verificar_cofirma`; esto solo le pone NOMBRE, para que un tercero no
    /// tenga que leerse una cadena de error para distinguirlo.
    ///
    /// ⚠️ Dice que los dos numeros no cuadran y **no acusa a nadie de
    /// mentir**: puede ser un ordinal reescrito o una clave adelantada. Por
    /// eso **no quema al testigo** —eso lo hace `IndiceRepetido`, que es el
    /// que prueba el reuso—, aunque su linea queda descartada como cualquier
    /// otra con hallazgo.
    IndiceDiscordante { linea: usize, declarado: u64, embebido: u64 },
}

impl HallazgoCofirma {
    /// Clases estables: se leen desde fuera y no cambian sin subir version.
    ///
    /// ⚠️⚠️ **DOS EMPIEZAN POR «indice» Y MIRAN NUMEROS DISTINTOS** (§333):
    /// `indice-repetido` mira **el indice EMBEBIDO** —dos cofirmas del mismo
    /// testigo con el mismo indice de hoja, que es lo que filtra la clave— y
    /// `indice-discordante` **compara el ordinal declarado con el embebido**.
    /// Quien lea una clase desde fuera tiene que saber cual de los dos
    /// numeros la produjo.
    pub fn clase(&self) -> &'static str {
        match self {
            HallazgoCofirma::Ilegible { .. } => "ilegible",
            HallazgoCofirma::VersionDesconocida { .. } => "version-desconocida",
            HallazgoCofirma::CampoAusente { .. } => "campo-ausente",
            HallazgoCofirma::NoVerifica { .. } => "no-verifica",
            HallazgoCofirma::IndiceRepetido { .. } => "indice-repetido",
            HallazgoCofirma::IndiceDiscordante { .. } => "indice-discordante",
        }
    }

    /// La linea del fichero a la que apunta el hallazgo. S319
    ///
    /// ⚠️ Las SEIS variantes la llevan, y de ahi sale la propiedad que
    /// hace barato el S319: la acreditacion puede DESCARTAR lineas sin
    /// volver a verificar nada. Quien decide que una linea esta mal sigue
    /// siendo `verificar_cofirmas`; esto solo LEE su veredicto.
    pub fn linea(&self) -> usize {
        match self {
            HallazgoCofirma::Ilegible { linea, .. } => *linea,
            HallazgoCofirma::VersionDesconocida { linea, .. } => *linea,
            HallazgoCofirma::CampoAusente { linea, .. } => *linea,
            HallazgoCofirma::NoVerifica { linea, .. } => *linea,
            HallazgoCofirma::IndiceRepetido { linea, .. } => *linea,
            HallazgoCofirma::IndiceDiscordante { linea, .. } => *linea,
        }
    }
}

/// **Verifica un fichero de cofirmas SIN el nodo, sin el diario y sin el
/// testigo.** Solo con lo publicado.
///
/// ⚠️ Esto convierte en EJECUTABLE lo que el §300 declaro: si para
/// comprobar una cofirma hiciera falta escribir un programa, **el formato
/// no estaria terminado**. Es el papel que `--auditar` cumple para el
/// diario (§249).
///
/// ⚠️ Y ademas **audita la SERIE**: dos cofirmas **del mismo testigo** con el
/// mismo indice son un hallazgo por si solas, verifiquen o no. El guardian
/// impide reutilizar un indice **en el testigo**; esto lo comprueba **desde
/// fuera**.
///
/// ⚠️ **CEGUERA DECLARADA (§310)**: NO mira si un mismo testigo cofirma DOS
/// digests distintos para la misma epoca. Eso es vista dividida FIRMADA, y
/// pide comparar epocas, no series: queda declarado y sin reparar aqui.
pub fn verificar_cofirmas(lineas: &[String]) -> Vec<HallazgoCofirma> {
    let mut h = Vec::new();
    // ⚠️ La clave es (CLAVE DEL TESTIGO, indice), no el indice a secas: cada
    //    cofirmante lleva su propia serie (§310).
    // ⚠️⚠️ El VALOR lleva la linea previa **y el PREAMBULO** (§334): repetir
    //    un indice solo prueba algo si los dos MENSAJES son distintos.
    let mut vistos: BTreeMap<(Vec<u8>, u64), (usize, Vec<u8>)> = BTreeMap::new();
    for (i, l) in lineas.iter().enumerate() {
        let n = i + 1;
        let v: Value = match serde_json::from_str(l) {
            Ok(v) => v,
            Err(e) => {
                h.push(HallazgoCofirma::Ilegible { linea: n, error: e.to_string() });
                continue;
            }
        };
        match v["v"].as_u64() {
            Some(x) if x <= COFIRMA_V_MAX => {}
            Some(x) => {
                h.push(HallazgoCofirma::VersionDesconocida { linea: n, v: x });
                continue;
            }
            None => {
                h.push(HallazgoCofirma::CampoAusente { linea: n, campo: "v".into() });
                continue;
            }
        }
        let mut falta = None;
        for k in ["epochDigest", "clavePublicaOperador", "clavePublicaTestigo",
                  "versionFormato", "indice", "firma"] {
            if v[k].is_null() {
                falta = Some(k);
                break;
            }
        }
        if let Some(k) = falta {
            h.push(HallazgoCofirma::CampoAusente { linea: n, campo: k.into() });
            continue;
        }
        let hx = |k: &str| leer_hex(&v[k]);
        let (dig, op, testigo, firma) = match (hx("epochDigest"), hx("clavePublicaOperador"),
                                               hx("clavePublicaTestigo"), hx("firma")) {
            (Ok(a), Ok(b), Ok(c), Ok(d)) => (a, b, c, d),
            _ => {
                h.push(HallazgoCofirma::Ilegible { linea: n, error: "hex torcido".into() });
                continue;
            }
        };
        let d32: [u8; 32] = match dig.try_into() {
            Ok(x) => x,
            Err(_) => {
                h.push(HallazgoCofirma::Ilegible { linea: n, error: "epochDigest no mide 32".into() });
                continue;
            }
        };
        let (vf, idx) = match (leer_q(&v["versionFormato"]), leer_q(&v["indice"])) {
            (Ok(a), Ok(b)) => (a, b),
            _ => {
                h.push(HallazgoCofirma::Ilegible { linea: n, error: "cantidad torcida".into() });
                continue;
            }
        };
        // ⚠️ La SERIE, antes que la firma: un indice repetido es un hallazgo
        //    aunque las dos firmas verifiquen. Precisamente por eso.
        // ⚠️ La clave del mapa lleva la clave ENTERA del testigo; lo que se
        //    IMPRIME es un prefijo, con el mismo corte que el resumen de
        //    cabezas de arriba. Truncar la CLAVE seria invitar a la colision.
        // ⚠️⚠️ LA CLAVE DE LA SERIE ES EL INDICE EMBEBIDO (§333). El ordinal
        //    declarado no entra en el preambulo y nadie lo acredita, asi que
        //    indexar por el dejaba pasar el REINICIO: contador 6 y clave 0
        //    dan ordinales nuevos con indices de hoja ya quemados. El que va
        //    DENTRO de la firma no se puede falsear sin romperla.
        // ⚠️ Una firma que no llega al ancho NO entra en la serie: no puede
        //    repetir un indice que no tiene, y su clase la pone la firma unas
        //    lineas mas abajo. Antes, dos lineas de basura con el mismo
        //    ordinal contaban como reuso y quemaban a un testigo honesto.
        // ⚠️⚠️⚠️ EL MENSAJE TIENE QUE SER DISTINTO (§334). Lo que filtra
        //    una clave de un solo uso son DOS mensajes bajo el mismo indice.
        //    Dos lineas IDENTICAS son la misma firma sobre el mismo mensaje:
        //    no revelan nada, y levantarlo como hallazgo dejaba que cualquiera
        //    desacreditara a un cofirmante honesto **duplicandole una linea**.
        //    El mensaje es el PREAMBULO entero -version, epochDigest y clave
        //    del operador-, no solo el digest: con el mismo digest y otra
        //    clave de operador ya son dos mensajes. Es la regla que
        //    `cribar_repetidas` declara para la recoleccion -"acusacion de
        //    doble firma fabricada por la herramienta"- traida al verificador
        //    del tercero, que se habia quedado sin ella.
        // ⚠️ Si el preambulo no se puede construir, la linea NO entra en la
        //    serie: morira unas lineas mas abajo con su clase de siempre.
        let emb = indice_de_firma(&firma).ok();
        let pre = preambulo_cofirma(vf as u8, &d32, &op).ok();
        if let (Some(e), Some(p)) = (emb, pre) {
            let clave = (testigo.clone(), e);
            match vistos.get(&clave).cloned() {
                None => {
                    vistos.insert(clave, (n, p));
                }
                // ⚠️ Mismo mensaje: NO se toca la entrada, para que `antes`
                //    siga senalando la PRIMERA aparicion y no la ultima copia.
                Some((antes, previo)) => {
                    if previo != p {
                        let tg = v["clavePublicaTestigo"].as_str().unwrap_or("");
                        h.push(HallazgoCofirma::IndiceRepetido {
                            linea: n,
                            indice: e,
                            antes,
                            testigo: tg[..18.min(tg.len())].to_string(),
                        });
                    }
                }
            }
        }
        let c = CabezaFirmada { version_formato: vf as u8, indice: idx, firma };
        if let Err(err) = verificar_cofirma(&testigo, &d32, &op, &c) {
            // ⚠️ `if let` y NO un `match` exhaustivo: `VerificaError` no es
            //    `#[non_exhaustive]`, asi que cerrarlo obligaria a tocar el
            //    testigo cada vez que el verificador gane una variante. La
            //    REGLA vive en `verificar_cofirma` (§332) y aqui solo se
            //    TRADUCE a la clase que el tercero lee.
            if let VerificaError::IndiceDiscordante { declarado, embebido } = &err {
                h.push(HallazgoCofirma::IndiceDiscordante {
                    linea: n,
                    declarado: *declarado,
                    embebido: *embebido,
                });
            } else {
                h.push(HallazgoCofirma::NoVerifica {
                    linea: n,
                    indice: idx,
                    error: format!("{err}"),
                });
            }
        }
    }
    h
}

// ---------------------------------------------------------------------
//  §320 · LA RECOLECCIÓN · rehacer la línea del TESTIGO desde el CABLE
// ---------------------------------------------------------------------

/// **Lo que impide recomponer una cofirma llegada del cable.**
///
/// ⚠️ No es un hallazgo sobre el testigo que la firmó: es que **esta**
/// recolección no sabe rehacerla. Se cuenta y se dice; no se calla.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecogidaRechazada {
    /// El cable declara una versión de cofirma que este cliente no conoce.
    VersionDesconocida { v: u64 },
    /// Un campo del cable no se deja leer con la convención declarada.
    CampoTorcido { campo: &'static str },
}

impl RecogidaRechazada {
    /// Nombre **estable** de la clase. Mismo criterio que §248: nunca `{:?}`.
    pub fn clase(&self) -> &'static str {
        match self {
            RecogidaRechazada::VersionDesconocida { .. } => "version-desconocida",
            RecogidaRechazada::CampoTorcido { .. } => "campo-torcido",
        }
    }
}

/// **Rehace la línea del TESTIGO a partir de la cofirma del CABLE.**
///
/// `CofirmaDto` lleva los **mismos ocho campos** que escribe
/// [`linea_de_cofirma`], pero **no la misma convención**: en el cable `v` y
/// `vistoUnix` viajan como QUANTITY (hex con `0x`) y en el fichero del
/// testigo van como números crudos. Sin cruzar esa frontera,
/// [`verificar_cofirmas`] mata la línea en el primer campo —`v["v"]`
/// `.as_u64()` sobre `"0x1"` da `None`— y el resto ni se mira.
///
/// ⚠️⚠️ **EL GATE DE VERSIÓN NO ES ADORNO.** [`linea_de_cofirma`] estampa
/// `COFIRMA_VERSION` **sin preguntar**. Recomponer sin comprobar antes lo
/// que el cable declara convertiría una cofirma de versión DESCONOCIDA en
/// una línea que dice ser de la nuestra, y [`verificar_cofirmas`] la
/// aceptaría: **blanqueo de versión**. Por eso `v` se mira **primero** y se
/// RECHAZA, en vez de reescribirlo.
///
/// ⚠️ La frontera se cruza **por la serialización declarada**, no leyendo
/// el DTO por dentro: se serializa con su propio `serde` y se leen los
/// valores con `leer_hex`/`leer_q`, **los mismos** que usa
/// [`verificar_cofirmas`]. Si el cable cambia de representación, esto la
/// sigue; leyéndolo por dentro, no.
///
/// ⚠️ La línea que sale **no es literalmente el payload del nodo**: se
/// cruza una frontera de convención, y se cruza a la vista.
pub fn linea_desde_dto(dto: &CofirmaDto) -> Result<Value, RecogidaRechazada> {
    let j = serde_json::to_value(dto)
        .map_err(|_| RecogidaRechazada::CampoTorcido { campo: "cofirma" })?;

    // ⚠️ PRIMERO la versión, antes de tocar ningún otro campo.
    let v = leer_q(&j["v"]).map_err(|_| RecogidaRechazada::CampoTorcido { campo: "v" })?;
    if v > COFIRMA_V_MAX {
        return Err(RecogidaRechazada::VersionDesconocida { v });
    }

    let hx = |k: &'static str| {
        leer_hex(&j[k]).map_err(|_| RecogidaRechazada::CampoTorcido { campo: k })
    };
    let dig = hx("epochDigest")?;
    let op = hx("clavePublicaOperador")?;
    let testigo = hx("clavePublicaTestigo")?;
    let firma = hx("firma")?;

    let d32: [u8; 32] = dig
        .try_into()
        .map_err(|_| RecogidaRechazada::CampoTorcido { campo: "epochDigest" })?;

    let q = |k: &'static str| {
        leer_q(&j[k]).map_err(|_| RecogidaRechazada::CampoTorcido { campo: k })
    };
    let vf = q("versionFormato")?;
    let idx = q("indice")?;
    let visto = q("vistoUnix")?;

    let c = CabezaFirmada { version_formato: vf as u8, indice: idx, firma };
    Ok(linea_de_cofirma(&d32, &op, &testigo, &c, visto))
}

/// **Deja pasar sólo las líneas que no estaban ya, comparadas ENTERAS.**
///
/// ⚠️⚠️ **Se criba por la LÍNEA COMPLETA, nunca por `(testigo, índice)`.**
/// Cribar por la clave borraría exactamente la evidencia que
/// [`verificar_cofirmas`] existe para cazar: dos cofirmas **distintas** con
/// el mismo índice del mismo testigo son un hallazgo del §310, no un
/// duplicado. Aquí sólo se descarta lo que es **byte a byte lo mismo**.
///
/// ⚠️ Sin esto, recolectar dos veces volvería a escribir las mismas
/// cofirmas y el lector las denunciaría como `IndiceRepetido`: **una
/// acusación de doble firma fabricada por la herramienta**.
pub fn cribar_repetidas(
    ya: &std::collections::BTreeSet<String>,
    nuevas: &[String],
) -> Vec<String> {
    let mut visto = ya.clone();
    let mut fuera = Vec::new();
    for l in nuevas {
        if visto.insert(l.clone()) {
            fuera.push(l.clone());
        }
    }
    fuera
}



// -- S319 . LA POLITICA DEL CLIENTE Y LA k QUE FALLA CERRADA ---------
//
// ⚠️⚠️ POR QUE EXISTE, y no es comodidad: las DOS claves -la del
//    testigo y la del operador- viajan DENTRO de la propia cofirma, asi
//    que `verificar_cofirmas` comprueba COHERENCIA INTERNA, no
//    autenticidad: cualquiera fabrica una cofirma que verifica con SU
//    clave. Lo que convierte "verifica" en "vale" es que el CLIENTE haya
//    NOMBRADO antes que claves acepta. El nodo no puede hacerlo porque el
//    nodo ES el operador -- y el contrato publicado ya lo dice, en el
//    summary de `zkssl_submitCosig`: "NO acredita al testigo: eso es
//    politica del cliente". Esto es esa promesa, cumplida.
//
// ⚠️ TODO lo de aqui lleva `allow(dead_code)` CON CADUCIDAD DECLARADA:
//    hasta el S319-2 -los dos flags y el mando- nadie lo llama fuera de
//    los tests, y `cargo test` compila CON cfg(test), donde si se usa. El
//    unico que lo veria muerto es un build SIN tests, que es justo el que
//    la VIVA de este bloque corre.

/// Lo que la politica dice de un fichero de cofirmas, POR EPOCA. S319
///
/// ⚠️ Se agrupa por `epochDigest` porque una k sobre cofirmas de
/// epocas DISTINTAS no significa nada: cada testigo cofirma una vez por
/// epoca, asi que juntarlas sumaria testigos que nunca avalaron lo mismo.
pub struct Acreditacion {
    /// `(epochDigest tal cual viene, testigos NOMBRADOS distintos)`, en el
    /// orden en que las epocas aparecen en el fichero.
    pub por_epoca: Vec<(String, usize)>,
    /// La epoca de la ULTIMA linea: la mas reciente que el testigo anoto,
    /// y la que decide el desenlace del mando.
    pub ultima: Option<String>,
    /// Cofirmas descartadas porque su testigo NO esta nombrado.
    pub no_nombrados: usize,
    /// Cofirmas descartadas porque `verificar_cofirmas` les puso hallazgo.
    pub con_hallazgo: usize,
    /// Testigos QUEMADOS: reusaron indice SOBRE MENSAJES DISTINTOS, asi que
    /// su clave se da por filtrada y no cuenta NINGUNA de sus cofirmas.
    ///
    /// ⚠️ Solo queman las lineas SIN ningun otro hallazgo (§334): una linea
    /// forjada produce el hallazgo pero no prueba reuso de nada.
    pub quemados: usize,
}

impl Acreditacion {
    /// Testigos nombrados distintos en una epoca concreta.
    pub fn en(&self, epoca: &str) -> usize {
        self.por_epoca
            .iter()
            .find(|(e, _)| e == epoca)
            .map(|(_, n)| *n)
            .unwrap_or(0)
    }

    /// ⚠️⚠️ FALLA CERRADA. Sin epoca ultima -fichero vacio, o
    /// ninguna linea con `epochDigest`- NO acredita. La ausencia de datos
    /// no es una acreditacion.
    pub fn acreditada(&self, k: usize) -> bool {
        match &self.ultima {
            None => false,
            Some(e) => self.en(e) >= k,
        }
    }
}

/// Las claves de testigo que el cliente acepta, leidas de un fichero. S319
///
/// Una clave en hexadecimal por linea; se ignoran las lineas en blanco y
/// las que empiezan por `#`.
///
/// ⚠️ El hex lo parsea `leer_hex`, EL MISMO de las cofirmas, envolviendo
/// la linea en un `Value`. Si naciera aqui un segundo parser, el fichero y
/// las cofirmas podrian aceptar formas distintas de la misma clave: dos
/// productores del mismo contrato, que es lo que esta casa repara.
fn leer_politica(lineas: &[String]) -> Result<std::collections::BTreeSet<Vec<u8>>, String> {
    let mut p = std::collections::BTreeSet::new();
    for (i, l) in lineas.iter().enumerate() {
        let t = l.trim();
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        match leer_hex(&Value::String(t.to_string())) {
            Ok(b) => {
                p.insert(b);
            }
            Err(e) => return Err(format!("linea {}: {e}", i + 1)),
        }
    }
    Ok(p)
}

/// La clave del testigo de una linea, si se deja leer. S319
fn testigo_de(lineas: &[String], linea: usize) -> Option<Vec<u8>> {
    let l = lineas.get(linea.checked_sub(1)?)?;
    let v: Value = serde_json::from_str(l).ok()?;
    leer_hex(&v["clavePublicaTestigo"]).ok()
}

/// El nucleo del S319: testigos NOMBRADOS distintos por epoca.
///
/// ⚠️⚠️ PRIVADA, Y CON `hallazgos` COMO PARAMETRO A PROPOSITO. Asi se
/// prueba sin firmar un solo XMSS -un test le pasa lineas hechas a mano y
/// `&[]`-, y a la vez no recorre la criptografia dos veces: quien la corre
/// es el mando, UNA vez, y le pasa aqui lo que salio. El invariante -que
/// `hallazgos` sea el de ESTAS lineas- vive en su unico llamante.
///
/// ⚠️ Un INDICE REPETIDO QUEMA AL TESTIGO. El doc de `IndiceRepetido`
/// lo dice: reusar un indice XMSS FILTRA la clave. Una clave filtrada no
/// avala nada, asi que no cuenta ninguna de sus cofirmas, ni las que
/// verifican. Es la lectura estricta, y es la que falla cerrada.
fn contar_acreditacion(
    lineas: &[String],
    hallazgos: &[HallazgoCofirma],
    politica: &std::collections::BTreeSet<Vec<u8>>,
) -> Acreditacion {
    let malas: std::collections::BTreeSet<usize> =
        hallazgos.iter().map(|h| h.linea()).collect();

    // ⚠️⚠️⚠️ LA MARCA SOLO SE DISPARA SOBRE MATERIAL QUE VERIFICA (§334).
    //    La SERIE se comprueba ANTES que la firma a proposito, asi que una
    //    linea FORJADA -copiar una cofirma real y reescribirle el
    //    epochDigest- colisiona con la buena y produce `indice-repetido`
    //    aunque nadie la haya firmado. Quemar por eso seria dejar que un
    //    tercero desacredite a un cofirmante honesto con una linea inventada.
    //    El hallazgo se DICE igual -la linea es mala y queda descartada-,
    //    pero la MARCA exige que la linea no traiga ningun OTRO hallazgo.
    //    Es la regla que el §332 escribio para el atado -solo dispara sobre
    //    lo que verifica- un piso mas arriba.
    let otros: std::collections::BTreeSet<usize> = hallazgos
        .iter()
        .filter(|h| h.clase() != "indice-repetido")
        .map(|h| h.linea())
        .collect();
    let mut quemados: std::collections::BTreeSet<Vec<u8>> =
        std::collections::BTreeSet::new();
    for h in hallazgos {
        if h.clase() == "indice-repetido" && !otros.contains(&h.linea()) {
            if let Some(t) = testigo_de(lineas, h.linea()) {
                quemados.insert(t);
            }
        }
    }

    let mut orden: Vec<String> = Vec::new();
    let mut por: std::collections::BTreeMap<String, std::collections::BTreeSet<Vec<u8>>> =
        std::collections::BTreeMap::new();
    let mut no_nombrados = 0usize;
    let mut con_hallazgo = 0usize;
    let mut ultima: Option<String> = None;

    for (i, l) in lineas.iter().enumerate() {
        let n = i + 1;
        // Lo ilegible ya lo dijo `verificar_cofirmas` con su clase propia.
        let v: Value = match serde_json::from_str(l) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let ep = match v["epochDigest"].as_str() {
            Some(s) => s.to_string(),
            None => continue,
        };
        ultima = Some(ep.clone());
        if !orden.contains(&ep) {
            orden.push(ep.clone());
        }
        if malas.contains(&n) {
            con_hallazgo += 1;
            continue;
        }
        let t = match leer_hex(&v["clavePublicaTestigo"]) {
            Ok(t) => t,
            Err(_) => continue,
        };
        if quemados.contains(&t) {
            continue;
        }
        if !politica.contains(&t) {
            no_nombrados += 1;
            continue;
        }
        por.entry(ep).or_default().insert(t);
    }

    Acreditacion {
        por_epoca: orden
            .iter()
            .map(|e| (e.clone(), por.get(e).map(|s| s.len()).unwrap_or(0)))
            .collect(),
        ultima,
        no_nombrados,
        con_hallazgo,
        quemados: quemados.len(),
    }
}

fn leer(p: &std::path::Path) -> anyhow::Result<Vec<String>> {
    Ok(std::fs::read_to_string(p)?.lines().map(str::to_string).collect())
}

pub fn run(a: WitnessArgs) -> anyhow::Result<()> {
    // ── §249/§283 · los tres modos que LEEN, antes del que observa ──
    if let Some(p) = &a.auditar {
        let r = auditar_lineas(&leer(p)?);
        println!("{}: {} lineas · {} con firma · {} REVERIFICADAS sin el nodo",
                 p.display(), r.lineas, r.con_firma, r.reverificadas);
        for h in &r.hallazgos {
            println!("  ⚠️ {} · {h:?}", h.clase());
        }
        if r.hallazgos.is_empty() {
            println!("sin hallazgos");
            return Ok(());
        }
        anyhow::bail!("{} hallazgo(s) en el diario", r.hallazgos.len());
    }
    if let Some(p) = &a.verificar_cofirmas {
        let ls = leer(p)?;
        let h = verificar_cofirmas(&ls);
        println!("{}: {} cofirma(s) leidas SIN el nodo, sin el diario y sin el testigo",
                 p.display(), ls.len());
        for x in &h {
            println!("  ⚠️ {} · {x:?}", x.clase());
        }
        // -- S319 . LA POLITICA DEL CLIENTE, si se dio ----------------
        // ⚠️ Aqui es donde "verifica" se convierte en "vale". Los
        //    hallazgos ya estan calculados arriba y se le PASAN: la
        //    criptografia no se recorre dos veces, y quien decide que una
        //    linea esta mal sigue siendo `verificar_cofirmas`.
        if let Some(pt) = &a.testigos {
            let politica = leer_politica(&leer(pt)?)
                .map_err(|e| anyhow::anyhow!("{}: {e}", pt.display()))?;
            let k = a.k.unwrap_or(1);
            let ac = contar_acreditacion(&ls, &h, &politica);
            let corto = |e: &str| e[..18.min(e.len())].to_string();
            println!("politica: {} testigo(s) NOMBRADO(s) · k = {k}", politica.len());
            for (e, n) in &ac.por_epoca {
                println!("  epoca {} · {} testigo(s) nombrado(s)", corto(e), n);
            }
            println!("  descartadas: {} con hallazgo · {} de testigo NO nombrado · {} testigo(s) QUEMADO(s)",
                     ac.con_hallazgo, ac.no_nombrados, ac.quemados);
            match &ac.ultima {
                // ⚠️⚠️ FALLA CERRADA: un fichero sin epocas no acredita
                //    nada. La ausencia de datos no es una acreditacion.
                None => anyhow::bail!("ninguna linea trae epoca: no hay nada que acreditar"),
                Some(e) if ac.acreditada(k) => {
                    println!("ACREDITADA la epoca {}: {} cofirma(s) nombradas, k = {k}",
                             corto(e), ac.en(e));
                }
                Some(e) => anyhow::bail!(
                    "la epoca {} tiene {} cofirma(s) de testigos NOMBRADOS y k = {k}: NO acredita",
                    corto(e), ac.en(e)),
            }
        }
        if h.is_empty() {
            println!("sin hallazgos: todas verifican y ningun indice DE FIRMA se repite sobre mensajes distintos");
            return Ok(());
        }
        anyhow::bail!("{} hallazgo(s) en las cofirmas", h.len());
    }
    if let Some(ps) = &a.comparar {
        let (x, y) = (leer(&ps[0])?, leer(&ps[1])?);
        let c = comparar_lineas(&x, &y);
        println!("{} ({} lineas) vs {} ({} lineas)",
                 ps[0].display(), x.len(), ps[1].display(), y.len());
        // ⚠️ Lo INTERNO va aparte: no es una divergencia entre A y B.
        for (n, mal) in [(&ps[0], c.interna_a), (&ps[1], c.interna_b)] {
            if mal {
                println!("  ⚠️⚠️ {} CONTIENE UNA VISTA DIVIDIDA INTERNA: el mismo",
                         n.display());
                println!("     indice con dos digests. Mirar con --auditar.");
            }
        }
        if c.divergencias.is_empty() {
            if c.interna_a || c.interna_b {
                anyhow::bail!("hay una vista dividida INTERNA: usar --auditar");
            }
            println!("sin divergencias");
            return Ok(());
        }
        for v in &c.divergencias {
            println!("  ⚠️⚠️ VISTA DIVIDIDA en el indice {}: {} != {} (misma clave: {})",
                     v.indice, v.digest_a, v.digest_b, v.misma_clave);
        }
        eprintln!();
        eprintln!("   DETECTAR NO ES DISTINGUIR: esto no dice cual miente.");
        eprintln!("   Pero el operador EMITIO LAS DOS, y ninguno de los dos");
        eprintln!("   testigos pudo fabricar su firma.");
        anyhow::bail!("{} divergencia(s) entre los diarios", c.divergencias.len());
    }

    if let Some(ps) = &a.ausentes {
        let (te, no) = (leer(&ps[0])?, leer(&ps[1])?);
        let faltan = ausentes(&te, &no);
        println!("{} ({} lineas) vs {} ({} lineas)",
                 ps[0].display(), te.len(), ps[1].display(), no.len());
        if faltan.is_empty() {
            println!("sin ausentes: el nodo recuerda todo lo que el testigo vio");
            return Ok(());
        }
        for i in &faltan {
            println!("  ⚠️⚠️ AUSENTE en el diario del nodo: indice {i}");
        }
        eprintln!();
        eprintln!("   O el nodo firmo algo que no recuerda, o alguien sirvio una");
        eprintln!("   firma que el nodo no emitio. En ambos casos responde el operador.");
        anyhow::bail!("{} indice(s) del testigo ausentes en el diario del nodo", faltan.len());
    }

    // ── §320 · la RECOLECCIÓN: pedir al nodo lo que OTROS firmaron ──
    //
    // ⚠️ Va detrás de los modos que sólo LEEN y delante del que observa:
    //    habla con el nodo, pero no se queda mirando.
    if let Some(p) = &a.recolectar_cofirmas {
        let agente = ureq::AgentBuilder::new().timeout(Duration::from_secs(10)).build();
        let peticion = json!({"jsonrpc":"2.0","id":1,
            "method":"zkssl_cosigs","params":{}});
        // ⚠️⚠️ LEE `error`. Es la regla que el §316 dejó escrita para el
        //    código que NACE HOY: sin ella, un nodo sin el método contesta
        //    -32601, `result` viene ausente, y esto diría «cero cofirmas»
        //    en vez de decir que el nodo no sabe servirlas.
        let v: Value = match agente.post(&a.nodo).send_json(peticion) {
            Err(e) => anyhow::bail!("no se pudieron pedir las cofirmas · transporte: {e}"),
            Ok(r) => match r.into_json::<Value>() {
                Err(e) => anyhow::bail!("respuesta ilegible: {e}"),
                Ok(v) => v,
            },
        };
        if let Some(err) = v.get("error") {
            anyhow::bail!("el nodo dio ERROR a zkssl_cosigs: {err}");
        }
        let res = v.get("result").cloned().unwrap_or(Value::Null);
        let epoca = res
            .get("epochDigest")
            .and_then(|x| x.as_str())
            .unwrap_or("(el nodo no dijo cual)")
            .to_string();
        // ⚠️ El sobre se abre TIPADO: `deny_unknown_fields` en `CofirmaDto`
        //    hace que un campo que el cable no conoce sea un error aquí, y
        //    no una línea silenciosamente incompleta más abajo.
        let brutas: Vec<CofirmaDto> = match res.get("cosigs").cloned() {
            None => anyhow::bail!("el nodo no sirvio el campo cosigs"),
            Some(c) => serde_json::from_value(c).map_err(|e| {
                anyhow::anyhow!("el sobre trae cofirmas que el cable no reconoce: {e}")
            })?,
        };

        let mut rehechas = Vec::new();
        let mut rechazadas = 0usize;
        for dto in &brutas {
            match linea_desde_dto(dto) {
                Ok(l) => rehechas.push(l.to_string()),
                Err(r) => {
                    rechazadas += 1;
                    eprintln!("  ⚠️ cofirma RECHAZADA · {} · {r:?}", r.clase());
                }
            }
        }

        let ya: std::collections::BTreeSet<String> = if p.exists() {
            leer(p)?.into_iter().filter(|l| !l.trim().is_empty()).collect()
        } else {
            std::collections::BTreeSet::new()
        };
        let ponen = cribar_repetidas(&ya, &rehechas);
        let repetidas = rehechas.len() - ponen.len();

        let mut f = std::fs::OpenOptions::new().create(true).append(true).open(p)?;
        for l in &ponen {
            writeln!(f, "{l}")?;
        }
        f.flush()?;

        println!("{}: {} cofirma(s) del nodo · {} anadida(s), {} ya estaba(n), {} rechazada(s)",
                 p.display(), brutas.len(), ponen.len(), repetidas, rechazadas);
        println!("epoca servida: {epoca}");
        println!("⚠️ lo anadido NO esta verificado: el nodo no verifica, solo filtra.");
        println!("   Compruebalo con --verificar-cofirmas, y con --testigos y --k");
        println!("   si quieres saber si ademas VALE.");
        return Ok(());
    }


    let mut m = Memoria::nueva();
    let agente = ureq::AgentBuilder::new().timeout(Duration::from_secs(10)).build();
    let mut diario = a
        .diario
        .as_ref()
        .map(|p| std::fs::OpenOptions::new().create(true).append(true).open(p))
        .transpose()?;

    // ── §300 · el cofirmante, si se pidio ──
    // ⚠️ Se abre AQUI, antes del bucle: `desde_semilla` comprueba el layout
    //    del SK y el guardian su propio `fsync`. **Mejor no arrancar que
    //    arrancar y descubrirlo en la primera firma.**
    let mut cofirmante = match &a.cofirmar {
        None => None,
        Some(p) => {
            let semilla = zk_ssl_guardian::semilla::leer_cruda(p)
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
            let ruta = a.indice_cofirma.as_ref().expect("clap lo exige");
            let mut c = Cofirmante::desde_semilla(&semilla, ruta)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("⚠️ COFIRMANDO: el testigo pasa a ser PARTE INTERESADA y hereda");
            println!("   las obligaciones que audita (notas 84, 92 y 19).");
            // ⚠️⚠️ §300 · **RECONCILIAR AL ARRANCAR, y decir que salio.** El
            //    contador adelantado es el caso NORMAL tras una caida —13 de
            //    25 en K.1—, no la excepcion: hay indices QUEMADOS SIN FIRMA.
            //    Un testigo que empieza a firmar sin mirar esto esta
            //    ignorando su propio invariante.
            //    ⚠⚠ K.1 midio DENTRO de un proceso, no tras un reinicio: al
            //    reiniciar la clave vuelve a cero y sale `ClaveEnCero`, que NO arranca.
            // ⚠️⚠️ §336 - EN DOS PASOS A PROPOSITO, como el nodo en el §335:
            //    si `c.reconciliar()` va dentro del escrutinio, su temporal
            //    mantiene `c` prestado durante TODO el match, y el dia que un
            //    brazo pida `&mut c` -resincronizar- deja de compilar. Se
            //    prepaga hoy, que cuesta una linea.
            let reconciliacion = c.reconciliar().map_err(|e| anyhow::anyhow!("{e}"))?;
            // ⚠️⚠️ S337 - el TOPE se lee AQUI, antes del escrutinio: la
            //    politica es PURA y no toca disco, igual que la del nodo, que
            //    recibe el suyo del diario ya leido.
            let tope = a.cofirmas.as_deref().and_then(tope_de_cofirmas);
            match politica_del_cofirmante(&reconciliacion, tope) {
                DecisionDelCofirmante::Arranca(m) => println!("{m}"),
                DecisionDelCofirmante::ArrancaAvisando(m) => println!("{m}"),
                DecisionDelCofirmante::ArrancaResincronizando { hasta, aviso } => {
                    eprintln!();
                    eprintln!("{aviso}");
                    c.resincronizar_a(hasta).map_err(|e| anyhow::anyhow!("{e}"))?;
                    println!("   clave resincronizada en el indice {hasta}.");
                }
                DecisionDelCofirmante::NoArranca { aviso, razon } => {
                    eprintln!();
                    eprintln!("{aviso}");
                    anyhow::bail!("{razon}");
                }
            }
            Some(c)
        }
    };
    let mut cofirmas = a
        .cofirmas
        .as_ref()
        .map(|p| std::fs::OpenOptions::new().create(true).append(true).open(p))
        .transpose()?;

    println!("testigo: {} cada {} s", a.nodo, a.cada);
    println!("⚠️ un testigo que opera el propio operador NO prueba nada: esto es la");
    println!("   implementacion de referencia de lo que correria un TERCERO.");

    let mut n = 0u64;
    loop {
        n += 1;
        let cuerpo = json!({"jsonrpc":"2.0","id":n,"method":"zkssl_signedEpochHead","params":{}});
        // ⚠️ `servido` se conserva: el diario guarda LO QUE EL NODO SIRVIO,
        // no una interpretacion.
        // ⚠️ §314 - CORRECCION del comentario de arriba: era FALSO en tres de
        // los cuatro caminos. `servido` NO era siempre lo que el nodo sirvio -- si
        // el transporte fallaba, si la respuesta era ilegible o si no traia
        // `result`, lo fabricaba el cliente. Se cita y se corrige (§247).
        //
        // ⚠️⚠️ La ausencia de respuesta se decide AQUI, donde se sabe, y no
        // dentro del juez: `una_vuelta` solo veria un objeto indistinguible de
        // una negativa legitima del nodo. Su firma queda INTACTA.
        let (veredicto, servido) = match pedir(&agente, &a.nodo, cuerpo) {
            Servido::SinRespuesta { motivo } => (Veredicto::SinRespuesta { motivo }, Value::Null),
            Servido::Respuesta(v) => {
                let ver = una_vuelta(&v, &mut m);
                (ver, v)
            }
        };

        // ── §294 · el SEGUNDO canal: la historia ──
        // ⚠️ Solo si la cabeza VERIFICO. Juzgar la historia de una cabeza
        // que no verifica seria dar valor a lo que acaba de fallar.
        let mut cons = match &veredicto {
            Veredicto::Nueva { .. } | Veredicto::Repetida { .. } | Veredicto::Hueco { .. } => {
                al_llegar_cabeza(&mut m, &servido)
            }
            _ => None,
        };
        if cons.is_none() && m.debe_pedir_camino() {
            let viejo = m.pareja().map(|(_, t)| t).unwrap_or(0);
            let peticion = json!({"jsonrpc":"2.0","id":n,"method":"zkssl_consistencyProof",
                                  "params":{"oldSize": format!("{viejo:#x}")}});
            // ⚠️ §314 - El mismo `pedir`: aqui vivia la copia del bloque de
            // arriba, que es por lo que el defecto estaba en los DOS canales.
            cons = Some(match pedir(&agente, &a.nodo, peticion) {
                Servido::SinRespuesta { motivo } => Consistencia::SinRespuesta { motivo },
                Servido::Respuesta(r) => al_llegar_camino(&mut m, &r),
            });
        }
        let servido = &servido;

        match &cons {
            Some(c) => println!("[{n}] {veredicto:?} · {c:?}"),
            None => println!("[{n}] {veredicto:?}"),
        }
        // ── §300 · COFIRMA ⇔ `Nueva` ∧ `Extiende`. NADA MAS. ──
        //
        // ⚠️⚠️ Un aval sobre algo que hizo saltar al testigo no es un aval, y
        // la regla se aplica ENTERA:
        //   · `Anclando` FUERA: anclar no es juzgar. Es el TOFU del MMR —la
        //     primera vez que se ve—, y la consistencia **no se ha juzgado
        //     porque no habia con que**. Cofirmar ahi seria avalar sin ese
        //     juicio, y **el tercero no puede distinguir una cofirma-tras-
        //     consistencia de una cofirma-en-anclaje sin mirar el diario**,
        //     que es justo la carga que este diseno no le reparte. La
        //     primera cofirma llega en la primera `Extiende`, un muestreo
        //     despues: el arranque se resuelve solo.
        //   · `Pendiente` FUERA por lo mismo: «aun no pude verificar la
        //     extension» es tan anomalia-para-avalar como «va por detras».
        //   · `Repetida` y `NoAplica` FUERA por **PRESUPUESTO**: cada
        //     cofirma **quema un indice de la serie XMSS** —la 83 lo declaro
        //     como EL precio de este eslabon— y re-avalar una cabeza ya
        //     avalada gasta serie sin informacion nueva. **Una cofirma por
        //     cabeza, en la vuelta que la juzgo.**
        let cofirmable = matches!(veredicto, Veredicto::Nueva { .. })
            && matches!(cons, Some(Consistencia::Extiende { .. }));
        let mut marca: Option<Value> = None;
        if let (Some(cf), true) = (cofirmante.as_mut(), cofirmable) {
            let d = servido.get("epochDigest").and_then(|v| v.as_str()).and_then(hex32);
            match d {
                None => eprintln!("[{n}] no se cofirma: el epochDigest no es hex de 32"),
                Some(d) => match cf.cofirmar_lo_anclado(&m, &d) {
                    // ⚠️ Un fallo al cofirmar NO detiene al testigo: observar
                    //    es lo primero, avalar es lo segundo. Pero se DICE.
                    Err(e) => eprintln!("[{n}] ⚠️ no se pudo cofirmar: {e}"),
                    Ok(c) => {
                        let pk = cf.clave_publica();
                        let op = m.clave_fijada().unwrap_or_default();
                        let opb = leer_hex(&json!(op)).unwrap_or_default();
                        let t = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|x| x.as_secs())
                            .unwrap_or(0);
                        if let Some(f) = cofirmas.as_mut() {
                            writeln!(f, "{}", linea_de_cofirma(&d, &opb, &pk, &c, t))?;
                            f.flush()?;
                        }
                        // ── §316 · y AHORA se publica, si se pidio ──
                        //
                        // ⚠️ DESPUES del fichero A PROPOSITO: la evidencia
                        //    propia del testigo sobrevive aunque la red
                        //    falle. El diario es del testigo para si mismo;
                        //    la cofirma es para terceros, y publicarla es lo
                        //    ultimo que se hace.
                        if a.enviar_cofirmas {
                            let dto = cofirma_dto(&d, &opb, &pk, &c, t);
                            let envio = json!({"jsonrpc":"2.0","id":n,
                                "method":"zkssl_submitCosig",
                                "params":{"cosig": dto}});
                            // ⚠️⚠️ Esta peticion LEE `error` y las dos de
                            //    arriba NO. No es repararlas —eso es corte
                            //    propio—: es no heredar el defecto en codigo
                            //    que nace hoy. Sin esto, un nodo sin el
                            //    metodo contesta -32601, `result` viene
                            //    ausente, y el testigo imprimiria un fallo
                            //    MUDO en cada vuelta.
                            match agente.post(&a.nodo).send_json(envio) {
                                Err(e) => eprintln!("[{n}] no se pudo enviar la cofirma · transporte: {e}"),
                                Ok(r) => match r.into_json::<Value>() {
                                    Err(e) => eprintln!("[{n}] no se pudo enviar la cofirma · respuesta ilegible: {e}"),
                                    Ok(v) => match v.get("error") {
                                        Some(err) => eprintln!("[{n}] el nodo dio ERROR a la cofirma: {err}"),
                                        None => {
                                            let res = v.get("result").cloned().unwrap_or(Value::Null);
                                            if res.get("accepted").and_then(|x| x.as_bool()).unwrap_or(false) {
                                                let g = res.get("stored").cloned().unwrap_or(Value::Null);
                                                println!("[{n}] cofirma enviada · guardadas {g}");
                                            } else {
                                                let razon = res
                                                    .get("reason")
                                                    .and_then(|x| x.as_str())
                                                    .unwrap_or("el nodo no dijo por que");
                                                eprintln!("[{n}] el nodo NO acepto la cofirma: {razon}");
                                            }
                                        }
                                    },
                                },
                            }
                        }
                        // ⚠️ La MARCA y la linea de cofirmas son **dos listas
                        //    del mismo contrato**: el indice las ata, y hay un
                        //    test que lo exige (§292→§293, §294→§295, §297).
                        marca = Some(json!({ "indice": format!("{:#x}", c.indice) }));
                        println!("[{n}] cofirmada · indice {:#x}", c.indice);
                    }
                },
            }
        }

        if let Some(f) = diario.as_mut() {
            // ⚠️ SOLO AÑADIR. El fichero se abre en modo `append`: un diario
            // que se puede reescribir tiene el mismo problema que un
            // historico servido por el operador, solo que con otro dueño.
            let t = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            // ⚠️ `linea_de_diario_con` se conserva CON SU FIRMA INTACTA
            //    (§294): la marca se anade fuera, no cambiando la funcion
            //    que el auditor y sus pruebas usan.
            let mut l = linea_de_diario_con(&veredicto, servido, t, cons.as_ref());
            if let Some(mm) = &marca {
                l["cofirmada"] = mm.clone();
            }
            writeln!(f, "{l}")?;
            f.flush()?;
        }

        let para_la_historia = cons.as_ref().map_or(false, |c| c.detiene());
        if veredicto.detiene() || para_la_historia {
            // ⚠️ SE DETIENE. Seguir seria sobrescribir el hallazgo con ruido.
            eprintln!();
            if para_la_historia {
                eprintln!("⚠️⚠️ EL TESTIGO SE DETIENE: {:?}", cons.as_ref().expect("comprobado"));
                eprintln!("   La cima nueva NO extiende a la custodiada: la historia se");
                eprintln!("   bifurco o se recorto POR DEBAJO de una cabeza ya firmada.");
            } else {
                eprintln!("⚠️⚠️ EL TESTIGO SE DETIENE: {veredicto:?}");
            }
            eprintln!("   Preservar la evidencia importa mas que acumular registros.");
            eprintln!("   DETECTAR NO ES DISTINGUIR: puede ser un accidente. Pero es");
            eprintln!("   OPONIBLE: el operador no puede negar haber emitido lo que firmo.");
            anyhow::bail!("hallazgo oponible tras {n} consultas");
        }
        if a.veces > 0 && n >= a.veces {
            let c = m.clave_fijada().unwrap_or("ninguna").to_string();
            println!("{} cabezas distintas · clave fijada: {}", m.vistos(), &c[..18.min(c.len())]);
            return Ok(());
        }
        std::thread::sleep(Duration::from_secs(a.cada));
    }
}

// ═══════════════════════════════════════════════════════════════════════
//  §299 · EL COFIRMANTE: el testigo tambien firma
//
//  ⚠️ Hasta aqui el testigo solo MIRABA. Cofirmar lo convierte en parte
//  interesada: hereda **las mismas obligaciones que audita** —guardian del
//  indice (§234), custodia declarada (nota 92), transicion firmada al
//  rotar (nota 84)— y **duplica la superficie de esas notas**. Eso esta
//  declarado en el asiento, no de perfil.
//
//  ⚠️ El objeto que se firma lo definio el §297 y **un tercero ya podia
//  verificarlo antes de que existiera ninguno**: `preambulo_cofirma` y
//  `verificar_cofirma` viven en `zk-ssl-verify`, no aqui.
// ═══════════════════════════════════════════════════════════════════════

// ⚠️ §300 · el `allow(dead_code)` que el §299 puso **con fecha de
// caducidad escrita** se retira aqui: el bucle ya llama al cofirmante, asi
// que la pieza tiene llamante de verdad. Un allow que se cumple es un
// allow que se quita.
#[derive(Debug)]
pub enum CofirmaError {
    Guardian(GuardianError),
    /// El crate `xmss` rechazo la operacion. Incluye `KeyExhausted`.
    Xmss(String),
    /// La cofirma recien hecha no verifica, o verifica contra otra cosa.
    Verifica(VerificaError),
    /// ⚠️ **No hay clave anclada, asi que no hay a quien atestiguar.**
    SinAncla,
    /// El hex de la clave anclada no se pudo leer.
    ClaveIlegible(String),
}

impl From<GuardianError> for CofirmaError {
    fn from(e: GuardianError) -> Self {
        CofirmaError::Guardian(e)
    }
}

impl From<VerificaError> for CofirmaError {
    fn from(e: VerificaError) -> Self {
        CofirmaError::Verifica(e)
    }
}

impl core::fmt::Display for CofirmaError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CofirmaError::Guardian(e) => write!(f, "cofirmante: {e}"),
            CofirmaError::Xmss(e) => write!(f, "cofirmante: xmss rechazo: {e}"),
            CofirmaError::Verifica(e) => write!(f, "cofirmante: {e}"),
            CofirmaError::SinAncla => write!(
                f,
                "cofirmante: el testigo no ha anclado ninguna clave todavia, \
                 asi que no hay operador a quien atestiguar. Una cofirma sin \
                 ancla no diria de QUIEN es la cabeza."
            ),
            CofirmaError::ClaveIlegible(e) => write!(f, "cofirmante: clave anclada ilegible: {e}"),
        }
    }
}

impl std::error::Error for CofirmaError {}

/// Que hace el ARRANQUE DEL COFIRMANTE con lo que el guardian encuentra al
/// reconciliar.
///
/// ⚠️ Es la POLITICA DEL COFIRMANTE, hermana de `DecisionDeArranque` del
/// nodo: cada dueno tiene la suya A PROPOSITO -es politica, no invariante de
/// la pieza, la forma del §319- y el guardian solo aporta el INVARIANTE.
/// **Cual es el caso que no admite matiz lo dice
/// `zk_ssl_guardian::no_admite_matiz`**, y hay un test que exige que esta
/// politica lo honre.
///
/// ⚠️⚠️ `NoArranca` lleva DOS cadenas porque el arranque hace dos cosas
/// distintas con ellas: `aviso` sale ENTERO por stderr y `razon` es lo que
/// viaja en el error. Fundirlas cambiaria los bytes que el banco lee.
#[derive(Debug, PartialEq, Eq)]
enum DecisionDelCofirmante {
    /// Todo cuadra: se dice por stdout y se sigue.
    Arranca(String),
    /// Anomalia ESPERADA: se dice por stdout y se sigue.
    ArrancaAvisando(String),
    /// ⚠️⚠️ S337 - La clave estaba en cero y NADA prueba que el
    /// contador haya retrocedido: se resincroniza hasta `hasta` y se sigue.
    /// Hermana de `DecisionDeArranque::ArrancaResincronizando` del nodo.
    ArrancaResincronizando { hasta: u64, aviso: String },
    /// No se arranca: `aviso` por stderr, `razon` al error.
    NoArranca { aviso: String, razon: String },
}

/// El TOPE del fichero de cofirmas: el mayor indice de hoja que el testigo
/// puede PROBAR que ya gasto.
///
/// ⚠️⚠️ **El indice sale de DENTRO de la firma** (§332, §333), no del campo
/// `indice` declarado: el ordinal declarado no entra en el preambulo y nadie
/// lo acredita, asi que tras un reinicio da ordinales NUEVOS sobre hojas ya
/// quemadas. El que va dentro no se puede falsear sin romper la firma.
///
/// ⚠️⚠️ **MAXIMO y no ULTIMO**, como `diario::maximo_indice` del nodo: el
/// caso del que esto defiende -un contador restaurado hacia atras- hace
/// escribir indices MENORES detras de mayores, asi que agregar por el ultimo
/// seria medir con un instrumento que el propio caso desarma.
///
/// ⚠️ Falla hacia el lado PERMISIVO: una linea ilegible se SALTA. Este
/// tope puede quedar POR DEBAJO del real y nunca por encima, asi que solo
/// puede dejar arrancar de mas, jamas dar un rojo falso.
fn tope_de_cofirmas(p: &std::path::Path) -> Option<u64> {
    let lineas = leer(p).ok()?;
    let mut max: Option<u64> = None;
    for l in lineas {
        let v: Value = match serde_json::from_str(&l) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let firma = match leer_hex(&v["firma"]) {
            Ok(f) => f,
            Err(_) => continue,
        };
        if let Ok(e) = indice_de_firma(&firma) {
            if max.map_or(true, |m| e > m) {
                max = Some(e);
            }
        }
    }
    max
}

/// La politica del arranque del cofirmante ante cada caso del guardian.
///
/// ⚠️ PURA: no imprime, no toca disco y no mira el reloj. Vivia dentro de
/// `run` y por eso no tenia un solo test unitario; el §336 la saca al molde
/// del §328 **sin cambiar un byte de lo que se imprime**, y por eso los
/// textos van tal cual estaban -incluida la sangria de tres espacios-.
fn politica_del_cofirmante(
    r: &Reconciliacion,
    tope_cofirmas: Option<u64>,
) -> DecisionDelCofirmante {
    match r {
        Reconciliacion::Coincide { indice } => DecisionDelCofirmante::Arranca(format!(
            "   guardian y clave a la par en el indice {indice}."
        )),
        Reconciliacion::ContadorAdelantado { contador, clave, huerfanos } => {
            DecisionDelCofirmante::ArrancaAvisando(
                [
                    format!("   contador {contador} por delante de la clave {clave}:"),
                    format!("   {huerfanos} indice(s) quemados sin firma. Es el caso"),
                    "   NORMAL tras una caida —el precio del orden—, no un fallo.".to_string(),
                ]
                .join("\n"),
            )
        }
        // ⚠️⚠️ S337 - El contador y el FICHERO DE COFIRMAS son dos
        //    productores del mismo hecho -quien firma, anota (§285)-: se atan
        //    aqui. La hoja `contador` NUNCA se reservo -`reservar` persiste
        //    ANTES de firmar-, asi que ponerse ahi es CONSERVADOR y las de
        //    abajo quedan PERDIDAS, que es lo que la nota 92 pide.
        //
        // ⚠️⚠️ EL OPERADOR ES `<=` Y ESTA DERIVADO, no copiado del nodo.
        //    El invariante que `verificar_cofirma` exige es `embebido <
        //    declarado` (§332), y aqui el tope es el EMBEBIDO: en estado
        //    limpio va por DEBAJO del contador. El nodo compara contra el
        //    indice DECLARADO que anota su diario y por eso alli el operador
        //    es `<`. Copiarlo habria dejado pasar `contador == tope`, que es
        //    exactamente reutilizar una hoja PROBADA.
        //
        // ⚠️ LIMITE DECLARADO: `--cofirmar` exige `--cofirmas`
        //    (`requires_all`), pero el fichero se abre en `append` y borrarlo
        //    lo recrea VACIO. Esto DETECTA un subconjunto, no PRUEBA nada.
        //    Va a CONFIANZA_RESIDUAL.
        Reconciliacion::ClaveEnCero { contador, indeterminados } => match tope_cofirmas {
            Some(t) if *contador <= t => DecisionDelCofirmante::NoArranca {
                aviso: [
                    format!("⚠️⚠️ EL CONTADOR HA RETROCEDIDO: dice {contador} y las"),
                    format!("   cofirmas prueban el indice {t}, que sale de DENTRO de una"),
                    "   firma y no se puede falsear. Resincronizar aqui reutilizaria".to_string(),
                    "   una hoja ya gastada, y eso filtra la clave. Se falla cerrada.".to_string(),
                ]
                .join("\n"),
                razon: format!("contador retrocedido: dice {contador} y las cofirmas prueban {t}"),
            },
            _ => {
                let ultima = contador.saturating_sub(1);
                DecisionDelCofirmante::ArrancaResincronizando {
                    hasta: *contador,
                    aviso: [
                        format!("⚠️⚠️ LA CLAVE ESTABA EN CERO Y EL CONTADOR EN {contador}:"),
                        format!("   la clave se resincroniza hasta {contador}. SE ABANDONAN"),
                        format!("   {indeterminados} indice(s), de 0 a {ultima}, que quedan"),
                        "   PERDIDOS y NO REUTILIZABLES. Un indice perdido es mejor que".to_string(),
                        "   uno indeterminado (nota 92).".to_string(),
                    ]
                    .join("\n"),
                }
            }
        },
        Reconciliacion::ClaveAdelantada { contador, clave, sin_registrar } => {
            DecisionDelCofirmante::NoArranca {
                aviso: [
                    format!("⚠️⚠️ LA CLAVE VA POR DELANTE DEL CONTADOR: {clave} frente a {contador}."),
                    format!("   {sin_registrar} firma(s) sin registrar: o el orden se invirtio,"),
                    "   o `fsync` no hizo lo que dijo. **LA CLAVE DEBE CONSIDERARSE".to_string(),
                    "   COMPROMETIDA**, y el testigo no firma nada mas con ella.".to_string(),
                ]
                .join("\n"),
                razon: format!("clave adelantada: {sin_registrar} firma(s) sin registrar"),
            }
        }
    }
}

/// **El cofirmante del testigo.** Clave XMSS propia y guardian del indice.
///
/// ⚠️ El guardian es **el mismo crate que usa el nodo** (§296, §298), no
/// una copia: dos implementaciones del mismo invariante pueden discrepar, y
/// aqui discrepar significa **filtrar una clave**.
///
/// ⚠️⚠️ **NO implementa `Debug`, y es a proposito.** Dentro vive un
/// `KeyPair`: **material de clave**. Un `Debug` derivado por comodidad
/// acaba en un log el dia que alguien depure con `{:?}`, y entonces la
/// clave privada del testigo esta en un fichero de texto. `FirmanteCabeza`
/// tampoco lo deriva. Si un test necesita formatear un fallo, que formatee
/// **el error**, que si lo implementa.
pub struct Cofirmante {
    par: KeyPair<Conjunto>,
    guardian: GuardianIndice,
}

impl Cofirmante {
    /// ⚠️ **La semilla es material de clave.** De donde sale y quien la
    /// guarda es **decision de despliegue**, y no esta tomada (nota 92).
    ///
    /// ⚠️ El layout del SK se comprueba **AL ABRIR**, no al firmar: si
    /// upstream cambio la serializacion, es mejor no arrancar que firmar y
    /// anotar mal. Es el molde de `firma_cabeza` (§234), con la lectura ya
    /// compartida desde el §298.
    pub fn desde_semilla(
        semilla: &[u8],
        ruta_contador: impl AsRef<std::path::Path>,
    ) -> Result<Self, CofirmaError> {
        let guardian = GuardianIndice::abrir(ruta_contador)?;
        let par = KeyPair::<Conjunto>::from_seed(semilla)
            .map_err(|e| CofirmaError::Xmss(format!("{e}")))?;
        let mut c = Cofirmante { par, guardian };
        let _ = c.indice_de_la_clave()?;
        Ok(c)
    }

    /// La clave publica del TESTIGO, tal como la publicaria.
    pub fn clave_publica(&mut self) -> Vec<u8> {
        self.par.verifying_key().as_ref().to_vec()
    }

    /// El indice que la clave dice tener, leido de su SK.
    ///
    /// ⚠️ `&mut` porque `signing_key()` de `xmss` lo exige, como en el molde
    /// del nodo. No es capricho de esta funcion.
    pub fn indice_de_la_clave(&mut self) -> Result<u64, CofirmaError> {
        Ok(zk_ssl_guardian::indice_de_sk(self.par.signing_key().as_ref())?)
    }

    /// Compara el contador con el indice real de la clave.
    ///
    /// ⚠️ [`Reconciliacion::ContadorAdelantado`] es **el caso normal tras
    /// una caida**, no la excepcion: K.1 lo midio en 13 de 25.
    pub fn reconciliar(&mut self) -> Result<Reconciliacion, CofirmaError> {
        // ⚠️ En dos pasos a proposito: `self.guardian.reconciliar(...)` con
        //    `self.indice_de_la_clave()` dentro toma `self` prestado DOS
        //    veces a la vez y no compila.
        let i = self.indice_de_la_clave()?;
        Ok(self.guardian.reconciliar(i))
    }

    /// **Reserva el indice y luego firma. Ese orden es la pieza.**
    ///
    /// ⚠️ Y despues **se verifica a si misma** con el mismo verificador que
    /// usara el tercero: 2,4 ms sobre 144,5 es el 1,7 %, y una firma que no
    /// verifica no debe salir de aqui creyendose buena.
    pub fn cofirmar(
        &mut self,
        epoch_digest: &[u8; 32],
        clave_del_operador: &[u8],
    ) -> Result<CabezaFirmada, CofirmaError> {
        // ── 1 · persistir con fsync ANTES de firmar ──
        let indice = self.guardian.reservar()?;
        // ── 2 · y solo entonces gastar el indice de la clave ──
        let pre = preambulo_cofirma(VERSION_FORMATO, epoch_digest, clave_del_operador)?;
        let sig = self
            .par
            .signing_key()
            .sign(&pre)
            .map_err(|e| CofirmaError::Xmss(format!("{e}")))?;
        let c = CabezaFirmada {
            version_formato: VERSION_FORMATO,
            indice,
            firma: sig.as_ref().to_vec(),
        };
        // ── 3 · y NO se devuelve sin comprobarla ──
        let pk = self.clave_publica();
        verificar_cofirma(&pk, epoch_digest, clave_del_operador, &c)?;
        Ok(c)
    }

    /// **Cofirma bajo la clave que el testigo ANCLO**, no bajo la que venga
    /// en el mensaje.
    ///
    /// ⚠️⚠️ **Esta funcion es la decision, escrita en el tipo.** Misma
    /// doctrina que §294 —el testigo dejo de creerse el `epochDigest` y paso
    /// a recomponerlo—: **no firmes lo que te dan, firma lo que anclaste**.
    ///
    /// ⚠️ Y **falla CERRADA**: sin ancla no hay nada que escribir en el
    /// preambulo, asi que la cofirma es IMPOSIBLE, no aproximada. Hoy la
    /// clave anclada y la recibida coinciden en valor —`anclar()` corre
    /// antes y `CambioDeClave` detiene—, pero **el orden es una convencion y
    /// el tipo es una garantia**: si algun dia alguien reordena las
    /// llamadas, esto sigue negandose y la otra version firmaria lo que le
    /// pusieran delante.
    pub fn cofirmar_lo_anclado(
        &mut self,
        m: &Memoria,
        epoch_digest: &[u8; 32],
    ) -> Result<CabezaFirmada, CofirmaError> {
        let hex = m.clave_fijada().ok_or(CofirmaError::SinAncla)?;
        // ⚠️ Se reusa el lector del cable en vez de escribir otro: un
        //    segundo lector de hex es una segunda forma de equivocarse.
        let bytes = leer_hex(&json!(hex)).map_err(CofirmaError::ClaveIlegible)?;
        self.cofirmar(epoch_digest, &bytes)
    }

    /// Pone la clave EN el indice que el guardian tiene registrado.
    ///
    /// ⚠️⚠️ **Es CONSERVADOR**: `reservar` persiste ANTES de firmar, asi
    /// que la hoja `indice` NUNCA se reservo y no puede estar quemada. Las de
    /// abajo quedan PERDIDAS -no reutilizables-, que es lo que la nota 92
    /// pide frente a dejarlas indeterminadas.
    ///
    /// ⚠️⚠️ **Es la HERMANA de `FirmanteCabeza::resincronizar_a`, y no una
    /// llamada compartida A PROPOSITO**: son dos duenos del mismo invariante y
    /// la casa los mantiene con DOS AFIRMACIONES PARALELAS mas su test, no con
    /// una funcion comun (§319). Lo que si tiene un solo dueno es el APANO
    /// DEL OID, que vive en `zk-ssl-verify` y se llama desde los dos.
    ///
    /// ⚠️ El SK viejo se zeroiza solo al asignar (`SigningKey` tiene
    /// `Drop`). El buffer temporal se borra a mano y es **BEST-EFFORT**: un
    /// `Vec` pudo reubicarse mientras se construia. Y `from_seed` de upstream
    /// ya deja una copia sin borrar en cada arranque: eso es del crate ajeno y
    /// va DECLARADO, no arreglado aqui.
    pub fn resincronizar_a(&mut self, indice: u64) -> Result<(), CofirmaError> {
        let mut sk = self.par.signing_key().as_ref().to_vec();
        zk_ssl_guardian::poner_indice_en_sk(&mut sk, indice)?;
        zk_ssl_verify::aplicar_apano_del_oid(&mut sk)?;
        let nueva = SigningKey::<Conjunto>::try_from(sk.as_slice())
            .map_err(|e| CofirmaError::Xmss(format!("{e}")))?;
        *self.par.signing_key() = nueva;
        sk.zeroize();
        // ⚠️ Se AUTOCOMPRUEBA antes de devolver, como el firmar del §299:
        //    no se afirma que la clave esta donde se pidio sin releerlo.
        let leido = self.indice_de_la_clave()?;
        if leido != indice {
            return Err(CofirmaError::Xmss(format!(
                "resincronizar pidio el indice {indice} y la clave dice {leido}"
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── §336 · LA POLITICA DEL COFIRMANTE, que hasta hoy no tenia tests ──

    /// Los cuatro casos, con su texto EXACTO: si alguien reescribe una frase,
    /// el banco se entera tarde y este test se entera ya.
    #[test]
    fn a_la_par_el_cofirmante_arranca_y_lo_dice() {
        match politica_del_cofirmante(&Reconciliacion::Coincide { indice: 7 }, None) {
            DecisionDelCofirmante::Arranca(m) => {
                assert_eq!(m, "   guardian y clave a la par en el indice 7.");
            }
            otra => panic!("a la par se arranca: {otra:?}"),
        }
    }

    #[test]
    fn el_contador_adelantado_arranca_avisando_porque_es_el_caso_normal() {
        match politica_del_cofirmante(&Reconciliacion::ContadorAdelantado {
            contador: 9,
            clave: 7,
            huerfanos: 2,
        }, None) {
            DecisionDelCofirmante::ArrancaAvisando(m) => {
                assert!(m.contains("contador 9 por delante de la clave 7:"), "{m}");
                assert!(m.contains("2 indice(s) quemados sin firma"), "{m}");
                assert_eq!(m.lines().count(), 3, "el aviso son TRES lineas: {m}");
            }
            otra => panic!("el caso normal no para: {otra:?}"),
        }
    }

    /// ⚠️⚠️ **ESTE TEST CAMBIO DE TESIS EN EL S337, y la vieja se CITA**
    /// (§247): hasta el §336 afirmaba que la clave en cero NO arranca
    /// nunca. Era correcto mientras nadie supiera resincronizar; ahora la hoja
    /// `contador` es demostrablemente virgen y negarse dejaba al testigo
    /// inservible tras su primera cofirma.
    #[test]
    fn la_clave_en_cero_sin_prueba_en_contra_se_resincroniza_hasta_el_contador() {
        match politica_del_cofirmante(
            &Reconciliacion::ClaveEnCero { contador: 5, indeterminados: 5 },
            None,
        ) {
            DecisionDelCofirmante::ArrancaResincronizando { hasta, aviso } => {
                assert_eq!(hasta, 5, "se va al contador, no a otro sitio");
                assert!(aviso.contains("de 0 a 4"), "nombra lo que abandona: {aviso}");
                assert_eq!(aviso.lines().count(), 5, "el aviso son CINCO lineas: {aviso}");
            }
            otra => panic!("sin prueba en contra se resincroniza: {otra:?}"),
        }
    }

    /// El estado LIMPIO es `tope < contador`: `reservar` devuelve `actual + 1`
    /// y la clave firma con el SUYO, asi que el embebido va por debajo. Con
    /// cuatro cofirmadas y el contador en 5 se resincroniza.
    #[test]
    fn con_el_tope_por_debajo_del_contador_se_resincroniza_igual() {
        match politica_del_cofirmante(
            &Reconciliacion::ClaveEnCero { contador: 5, indeterminados: 5 },
            Some(4),
        ) {
            DecisionDelCofirmante::ArrancaResincronizando { hasta: 5, .. } => {}
            otra => panic!("el estado limpio tiene que arrancar: {otra:?}"),
        }
    }

    /// ⚠️⚠️ **EL OPERADOR ES `<=` Y AQUI SE EJERCITA EL BORDE.** Con el
    /// tope IGUAL al contador, la hoja `contador` esta PROBADAMENTE gastada:
    /// con el `<` del nodo esto habria arrancado y reutilizado una hoja.
    #[test]
    fn con_el_tope_a_la_par_del_contador_no_arranca() {
        match politica_del_cofirmante(
            &Reconciliacion::ClaveEnCero { contador: 5, indeterminados: 5 },
            Some(5),
        ) {
            DecisionDelCofirmante::NoArranca { razon, .. } => {
                assert!(razon.contains("retrocedido"), "{razon}");
            }
            otra => panic!("la hoja 5 esta probada: no se puede reutilizar: {otra:?}"),
        }
    }

    #[test]
    fn un_contador_por_debajo_de_lo_cofirmado_no_arranca_y_lo_dice() {
        match politica_del_cofirmante(
            &Reconciliacion::ClaveEnCero { contador: 4, indeterminados: 4 },
            Some(9),
        ) {
            DecisionDelCofirmante::NoArranca { aviso, razon } => {
                assert!(aviso.contains("EL CONTADOR HA RETROCEDIDO"), "{aviso}");
                assert!(razon.contains("dice 4") && razon.contains("prueban 9"), "{razon}");
            }
            otra => panic!("un contador que retrocede filtra la clave: {otra:?}"),
        }
    }

    #[test]
    fn la_clave_adelantada_no_arranca_y_la_da_por_comprometida() {
        match politica_del_cofirmante(&Reconciliacion::ClaveAdelantada {
            contador: 3,
            clave: 9,
            sin_registrar: 6,
        }, None) {
            DecisionDelCofirmante::NoArranca { aviso, razon } => {
                assert!(aviso.contains("LA CLAVE VA POR DELANTE DEL CONTADOR: 9 frente a 3."), "{aviso}");
                assert!(aviso.contains("COMPROMETIDA**"), "{aviso}");
                assert_eq!(razon, "clave adelantada: 6 firma(s) sin registrar");
            }
            otra => panic!("la clave adelantada NUNCA arranca: {otra:?}"),
        }
    }

    /// ⚠️⚠️ **EL INVARIANTE ES DEL GUARDIAN, LA POLITICA ES DE CADA DUENO.**
    /// Este test no ata las dos politicas entre si -no lo llames atado-: son
    /// **dos afirmaciones paralelas** del mismo invariante, una en cada crate.
    /// Lo que las mantiene juntas es que el PREDICADO tiene un solo dueno y
    /// que el `match` de abajo es EXHAUSTIVO: el dia que nazca una quinta
    /// variante de `Reconciliacion`, este fichero deja de compilar hasta que
    /// alguien decida que hace la politica con ella.
    #[test]
    fn la_politica_honra_el_invariante_del_guardian_en_todas_las_variantes() {
        let casos = [
            Reconciliacion::Coincide { indice: 7 },
            Reconciliacion::ContadorAdelantado { contador: 9, clave: 7, huerfanos: 2 },
            Reconciliacion::ClaveEnCero { contador: 5, indeterminados: 5 },
            Reconciliacion::ClaveAdelantada { contador: 3, clave: 9, sin_registrar: 6 },
        ];
        for r in &casos {
            // el match SIN comodin es el atado: una variante nueva rompe aqui
            match r {
                Reconciliacion::Coincide { .. }
                | Reconciliacion::ContadorAdelantado { .. }
                | Reconciliacion::ClaveEnCero { .. }
                | Reconciliacion::ClaveAdelantada { .. } => {}
            }
            if zk_ssl_guardian::no_admite_matiz(r) {
                assert!(
                    matches!(
                        politica_del_cofirmante(r, None),
                        DecisionDelCofirmante::NoArranca { .. }
                    ),
                    "el guardian dice que {r:?} no admite matiz y la politica arranca"
                );
            }
        }
    }

    // ── §294 · EL SEGUNDO CANAL: la historia ──────────────────────────

    /// Una cabeza v3 servida, con la pareja del MMR. **No verifica firma**:
    /// estas pruebas son de la MAQUINA DE ESTADOS del canal, no de la
    /// criptografia (que ya tiene las suyas en `zk-ssl-verify`).
    fn cabeza_v3(cima: &str, t: u64) -> Value {
        json!({
            "available": true,
            "formatVersion": "0x3",
            "mmrRoot": cima,
            "mmrSize": format!("{t:#x}"),
        })
    }

    fn cima(b: u8) -> String {
        format!("0x{}", hex::encode_32(b))
    }

    #[test]
    fn la_primera_cabeza_v3_ancla_la_pareja_y_no_juzga_nada() {
        let mut m = Memoria::nueva();
        let c = al_llegar_cabeza(&mut m, &cabeza_v3(&cima(1), 4));
        assert_eq!(c, Some(Consistencia::Anclando { t: 4 }));
        let c1 = cima(1);
        assert_eq!(m.pareja(), Some((c1.as_str(), 4)));
    }

    #[test]
    fn una_cabeza_que_no_es_v3_no_tiene_historia_que_extender() {
        let mut m = Memoria::nueva();
        let v2 = json!({"available": true, "formatVersion": "0x2"});
        assert_eq!(al_llegar_cabeza(&mut m, &v2), Some(Consistencia::NoAplica));
        assert!(m.pareja().is_none(), "una v2 NO puede anclar pareja");
    }

    #[test]
    fn con_pareja_y_sin_pendiente_el_canal_calla_y_el_bucle_pide_camino() {
        let mut m = Memoria::nueva();
        al_llegar_cabeza(&mut m, &cabeza_v3(&cima(1), 4));
        assert_eq!(al_llegar_cabeza(&mut m, &cabeza_v3(&cima(1), 4)), None);
        assert!(m.debe_pedir_camino(), "hay de donde partir y nada pendiente");
    }

    #[test]
    fn el_camino_se_guarda_y_espera_a_la_cabeza_que_lo_firma() {
        // ⚠️⚠️ LA PROPIEDAD DE §293: el camino de tamaño t lo firma la
        // cabeza SIGUIENTE. Un testigo que exigiera igualdad instantanea no
        // casaria jamas.
        let mut m = Memoria::nueva();
        al_llegar_cabeza(&mut m, &cabeza_v3(&cima(1), 4));
        let r = json!({"available": true, "mmrSize": "0x6", "camino": [cima(9)]});
        assert_eq!(
            al_llegar_camino(&mut m, &r),
            Consistencia::Pendiente { de_t: 4, esperando_t: 6 }
        );
        // llega una cabeza que NO es la que firma el camino: sigue esperando
        let c = al_llegar_cabeza(&mut m, &cabeza_v3(&cima(2), 5));
        assert_eq!(c, Some(Consistencia::Pendiente { de_t: 4, esperando_t: 6 }));
        assert!(m.pendiente.is_some(), "la pendiente NO se pierde por el camino");
    }

    #[test]
    fn la_cabeza_que_firma_el_camino_lo_juzga_y_el_ancla_avanza() {
        // Caso IDENTIDAD (viejo == nuevo, camino vacio): la consistencia
        // exige cima igual, y eso es comprobable sin fabricar un MMR.
        let mut m = Memoria::nueva();
        al_llegar_cabeza(&mut m, &cabeza_v3(&cima(1), 4));
        let r = json!({"available": true, "mmrSize": "0x4", "camino": []});
        al_llegar_camino(&mut m, &r);
        let c = al_llegar_cabeza(&mut m, &cabeza_v3(&cima(1), 4));
        assert_eq!(
            c,
            Some(Consistencia::Extiende { de_t: 4, a_t: 4, camino: vec![] })
        );
        assert!(m.pendiente.is_none(), "juzgada, la pendiente se consume");
        let c1 = cima(1);
        assert_eq!(m.pareja(), Some((c1.as_str(), 4)));
    }

    #[test]
    fn una_cima_que_no_extiende_detiene_al_testigo_y_no_mueve_el_ancla() {
        // ⚠️⚠️ EL TEST QUE HACE QUE ESTO SEA UN ESLABON. Sin el, el canal
        // seria decorativo.
        let mut m = Memoria::nueva();
        al_llegar_cabeza(&mut m, &cabeza_v3(&cima(1), 4));
        let r = json!({"available": true, "mmrSize": "0x4", "camino": []});
        al_llegar_camino(&mut m, &r);
        // misma t, OTRA cima: la identidad exige cima igual
        let c = al_llegar_cabeza(&mut m, &cabeza_v3(&cima(7), 4)).expect("hay veredicto");
        assert!(matches!(c, Consistencia::NoExtiende { .. }), "{c:?}");
        assert!(c.detiene(), "una historia que no extiende DEBE detener");
        let c1 = cima(1);
        assert_eq!(
            m.pareja(),
            Some((c1.as_str(), 4)),
            "el ancla NO se mueve ante un hallazgo: preservar la evidencia"
        );
    }

    #[test]
    fn un_camino_ilegible_no_pasa_por_bueno() {
        // Un juez que se saltara lo que no entiende absolveria por ignorancia.
        let p = Pendiente {
            de_cima: cima(1),
            de_t: 4,
            esperando_t: 4,
            camino: vec!["0xnoesunhex".into()],
        };
        assert!(!juzgar(&p, &cima(1), 4), "el hex ilegible no puede dar VERDE");
        let mala = Pendiente { de_cima: "0x00".into(), ..p.clone() };
        assert!(!juzgar(&mala, &cima(1), 4), "una cima corta tampoco");
    }

    #[test]
    fn el_acumulador_que_va_por_detras_se_anota_pero_no_detiene() {
        // ⚠️ Es el nodo DICIENDO la verdad sobre su reseteo (§292/§293).
        // Detener aqui quemaria al testigo en cada reinicio legitimo.
        let mut m = Memoria::nueva();
        al_llegar_cabeza(&mut m, &cabeza_v3(&cima(1), 9));
        let c = al_llegar_cabeza(&mut m, &cabeza_v3(&cima(1), 3)).expect("hay veredicto");
        assert_eq!(c, Consistencia::PorDetras { t_nodo: 3, pedido: 9 });
        assert!(!c.detiene(), "un reseteo VISIBLE no es un hallazgo oponible");
    }

    #[test]
    fn un_reseteo_descarta_la_pendiente_en_vez_de_dejar_al_testigo_ciego() {
        // ⚠️⚠️ LA FE DE ERRATAS DEL §295. Antes: con una pendiente viva,
        // un nodo reseteado no alcanzaba JAMAS su `esperando_t`, asi que
        // el canal decia `consistencia-pendiente` para siempre y el
        // reseteo no se anotaba nunca. Ahora el retroceso manda.
        let mut m = Memoria::nueva();
        al_llegar_cabeza(&mut m, &cabeza_v3(&cima(1), 9));
        let r = json!({"available": true, "mmrSize": "0xc", "camino": [cima(5)]});
        al_llegar_camino(&mut m, &r);
        assert!(m.pendiente.is_some(), "hay un camino esperando a t=12");
        // el nodo rearranca sin diario: vuelve a t=2, que NO es 12
        let c = al_llegar_cabeza(&mut m, &cabeza_v3(&cima(2), 2)).expect("hay veredicto");
        assert_eq!(c, Consistencia::PorDetras { t_nodo: 2, pedido: 9 });
        assert!(m.pendiente.is_none(), "la pendiente de una historia muerta se DESCARTA");
        assert!(!c.detiene(), "un reseteo VISIBLE sigue sin ser un hallazgo oponible");
    }

    #[test]
    fn el_diario_guarda_todo_lo_que_la_recomposicion_lee() {
        // ⚠⚠ EL CONTRATO QUE ROMPIO EL BANCO. Dos listas —la del diario
        // y la que `recomponer` lee— son DOS PRODUCTORES DEL MISMO
        // CONTRATO (§292, tercera vez). Este test las ata: si alguien anade
        // un campo a la recomposicion y no al diario, `--auditar` deja de
        // funcionar sobre diarios legitimos y **nadie se entera**.
        let servido = json!({
            "available": true, "index": "0x7", "epochDigest": "0xaa",
            "signature": "0xbb", "publicKey": "0xcc", "formatVersion": "0x3",
            "seq": "0x1", "n": "0x2", "accountsRoot": "0xd1", "pendingRoot": "0xd2",
            "frozenRoot": "0xd3", "chainDigest": "0xd4", "acusesRoot": "0xd5",
            "mmrRoot": "0xd6", "mmrSize": "0x3",
        });
        let l = linea_de_diario(&Veredicto::Nueva { indice: 7, digest: "0xaa".into() },
                                &servido, 1000);
        for k in ["seq", "n", "accountsRoot", "pendingRoot", "frozenRoot",
                  "chainDigest", "acusesRoot", "mmrRoot", "mmrSize",
                  "epochDigest", "formatVersion", "signature", "publicKey"] {
            assert!(!l[k].is_null(),
                    "el diario NO guarda {k}, y la recomposicion lo LEE: --auditar moriria");
        }
    }

    #[test]
    fn un_ancla_en_cero_no_es_un_ancla_y_se_vuelve_a_anclar() {
        // ⚠⚠ LO CAZO EL BANCO: contra un nodo RECIEN NACIDO la primera
        // cabeza trae mmrSize=0, y el servicio niega ese oldSize por
        // diseno. Anclarse ahi dejaba al testigo atascado PARA SIEMPRE.
        let mut m = Memoria::nueva();
        let c = al_llegar_cabeza(&mut m, &cabeza_v3(&cima(0), 0));
        assert_eq!(c, Some(Consistencia::Anclando { t: 0 }));
        assert!(!m.debe_pedir_camino(), "con t=0 no se pide lo que el servicio niega");
        let c = al_llegar_cabeza(&mut m, &cabeza_v3(&cima(3), 2));
        assert_eq!(c, Some(Consistencia::Anclando { t: 2 }), "se RE-ancla con historia de verdad");
        let c3 = cima(3);
        assert_eq!(m.pareja(), Some((c3.as_str(), 2)));
        assert!(m.debe_pedir_camino());
    }

    #[test]
    fn una_pendiente_que_el_testigo_se_perdio_caduca() {
        // El testigo MUESTREA y el nodo LATE: si consulta mas despacio, la
        // cabeza que firmaba `esperando_t` pasa sin que la vea. Sin
        // caducidad, esa pendiente no casaria jamas.
        let mut m = Memoria::nueva();
        al_llegar_cabeza(&mut m, &cabeza_v3(&cima(1), 4));
        al_llegar_camino(&mut m, &json!({"available": true, "mmrSize": "0x6", "camino": [cima(9)]}));
        assert!(m.pendiente.is_some());
        // llega una cabeza YA PASADA de largo: t=9 > esperando 6
        assert_eq!(al_llegar_cabeza(&mut m, &cabeza_v3(&cima(2), 9)), None);
        assert!(m.pendiente.is_none(), "la pendiente perdida se DESCARTA");
        assert!(m.debe_pedir_camino(), "y se vuelve a pedir en esta misma vuelta");
    }

    #[test]
    fn la_negativa_del_servicio_se_mide_por_estructura_no_por_su_frase() {
        let mut m = Memoria::nueva();
        al_llegar_cabeza(&mut m, &cabeza_v3(&cima(1), 9));
        let r = json!({"available": false, "mmrSize": "0x3", "reason": "lo que sea"});
        assert_eq!(
            al_llegar_camino(&mut m, &r),
            Consistencia::PorDetras { t_nodo: 3, pedido: 9 },
            "el mmrSize servido decide, no el texto"
        );
        let r2 = json!({"available": false, "mmrSize": "0x9", "reason": "oldSize 0"});
        assert!(matches!(
            al_llegar_camino(&mut m, &r2),
            Consistencia::SinCamino { .. }
        ));
    }

    #[test]
    fn la_linea_lleva_los_dos_canales_y_el_camino_del_que_juzgo() {
        // ⚠️ D3: sin el camino anotado, la extension no se reaudita.
        let v = Veredicto::Nueva { indice: 7, digest: "0xaa".into() };
        let servido = json!({
            "available": true, "index": "0x7", "epochDigest": "0xaa",
            "signature": "0xbb", "publicKey": "0xcc",
            "mmrRoot": cima(1), "mmrSize": "0x4",
        });
        let c = Consistencia::Extiende { de_t: 3, a_t: 4, camino: vec![cima(9)] };
        let l = linea_de_diario_con(&v, &servido, 1000, Some(&c));
        assert_eq!(l["v"], json!(3), "la version del diario SUBE");
        assert_eq!(l["clase"], json!("nueva"), "la clase de la CABEZA no se pierde");
        assert_eq!(l["mmrRoot"], json!(cima(1)));
        assert_eq!(l["mmrSize"], json!("0x4"));
        assert_eq!(l["consistencia"]["clase"], json!("extiende"));
        assert_eq!(l["consistencia"]["deT"], json!("0x3"));
        assert_eq!(l["consistencia"]["camino"], json!([cima(9)]));
    }

    #[test]
    fn sin_canal_la_linea_es_exactamente_la_de_antes() {
        // ⚠️ Compatibilidad: `linea_de_diario` no cambio de forma.
        let v = Veredicto::Repetida { indice: 3 };
        let s = json!({"available": false, "reason": "sin latido"});
        assert_eq!(
            linea_de_diario_con(&v, &s, 7, None),
            linea_de_diario(&v, &s, 7)
        );
    }

    /// Hex de 32 bytes con un byte distintivo: no hace falta que sea un
    /// digest valido para las pruebas de MAQUINA DE ESTADOS.
    mod hex {
        pub fn encode_32(b: u8) -> String {
            let mut s = String::with_capacity(64);
            for i in 0..32u8 {
                s.push_str(&format!("{:02x}", if i == 31 { b } else { 0 }));
            }
            s
        }
    }

    // ── §250: la vista dividida EN FRIO ──

    #[test]
    fn un_indice_repetido_con_el_mismo_digest_no_es_hallazgo() {
        // ⚠️ ASI ESCRIBE EL TESTIGO DE VERDAD: anota `nueva` y luego
        // `repetida`, y LAS DOS LLEVAN LA CABEZA. Los tests de §249
        // fabricaban un indice por linea —una IDEA del diario— y por eso no
        // vieron nada.
        let d = vec![
            dia("0x1", "0xaa", "0xdead"),
            dia("0x1", "0xaa", "0xdead"),
            dia("0x2", "0xbb", "0xdead"),
        ];
        let r = auditar_lineas(&d);
        let c: Vec<_> = r.hallazgos.iter().map(|h| h.clase()).collect();
        assert!(!c.contains(&"vista-dividida"), "repetir un indice es NORMAL: {c:?}");
        assert!(!c.contains(&"indice-retrocede"), "{c:?}");
    }

    #[test]
    fn auditar_ve_una_vista_dividida_dentro_del_mismo_diario() {
        // ⚠️⚠️ EL FALLO QUE ENCONTRO EL BANCO L.2. Un diario que CONTIENE la
        // prueba de una vista dividida pasaba limpio — y en un artefacto
        // oponible, UN FALSO "LIMPIO" ES PEOR QUE NO TENER HERRAMIENTA.
        let d = vec![
            dia("0x9", "0xaaaa", "0xdead"),
            dia("0x9", "0xbbbb", "0xdead"),
        ];
        let r = auditar_lineas(&d);
        let h = r.hallazgos.iter().find(|h| h.clase() == "vista-dividida")
            .expect("el auditor DEBE verla en frio");
        match h {
            Hallazgo::VistaDividida { indice, digest_a, digest_b, .. } => {
                assert_eq!(*indice, 9);
                assert_eq!(digest_a, "0xaaaa");
                assert_eq!(digest_b, "0xbbbb");
            }
            otro => panic!("{otro:?}"),
        }
    }

    #[test]
    fn comparar_no_deja_que_la_ultima_linea_tape_a_la_primera() {
        // ⚠️⚠️ LA CAUSA MECANICA DEL FALLO: `m.insert` SOBRESCRIBIA. Con el
        // indice repetido, manipular la PRIMERA linea desaparecia del mapa.
        let a = vec![dia("0x1", "0xaa", "0xdead"), dia("0x1", "0xaa", "0xdead")];
        let b = vec![dia("0x1", "0xde", "0xdead"), dia("0x1", "0xaa", "0xdead")];
        let c = comparar_lineas(&a, &b);
        assert_eq!(c.divergencias.len(), 1, "la manipulacion de la 1a NO puede taparse: {c:?}");
        assert_eq!(c.divergencias[0].digest_b, "0xde");
        assert!(c.interna_b, "y B se contradice a si mismo");
    }

    #[test]
    fn comparar_separa_la_vista_dividida_interna_de_la_divergencia() {
        // ⚠️ Una vista dividida INTERNA no es "A difiere de B": el problema
        // esta DENTRO de un fichero, y se mira con --auditar. Presentarlo
        // como divergencia confundiria al que mira.
        let a = vec![dia("0x1", "0xaa", "0xdead"), dia("0x1", "0xbb", "0xdead")];
        let b = vec![dia("0x1", "0xaa", "0xdead")];
        let c = comparar_lineas(&a, &b);
        assert!(c.interna_a, "A se contradice a si mismo");
        assert!(!c.interna_b);
        assert!(c.divergencias.is_empty(), "la PRIMERA de A coincide con B: {c:?}");
    }

    // ── §283 / nota 80: la comprobacion DIRECCIONAL ──

    #[test]
    fn ausentes_detecta_el_indice_que_el_testigo_tiene_y_el_nodo_no() {
        // ⚠️ El caso grave: o el nodo firmo algo que no recuerda, o alguien
        // sirvio una firma que el nodo no emitio. dia() lleva signature:
        // sin firma la linea se salta y el test pasaria VACIO (§250).
        let testigo = vec![dia("0x1", "0xaa", "0xdead"), dia("0x5", "0xbb", "0xdead")];
        let nodo = vec![dia("0x1", "0xaa", "0xdead")];
        assert_eq!(ausentes(&testigo, &nodo), vec![5]);
    }

    #[test]
    fn ausentes_es_direccional_lo_que_sobra_en_el_nodo_es_paisaje() {
        // El diario del nodo es completo y el del testigo muestreado: un
        // indice que el nodo tiene y el testigo no NO es un hallazgo.
        let testigo = vec![dia("0x1", "0xaa", "0xdead")];
        let nodo = vec![dia("0x1", "0xaa", "0xdead"), dia("0x9", "0xcc", "0xdead")];
        assert!(ausentes(&testigo, &nodo).is_empty());
    }

    // ── §249: LEER el diario ──

    /// Una linea de diario bien formada, para las pruebas de estructura.
    fn dia(indice: &str, digest: &str, clave: &str) -> String {
        let mut v = json!({});
        v["v"] = json!(DIARIO_VERSION);
        v["clase"] = json!("nueva");
        v["vistoUnix"] = json!(1000);
        v["index"] = json!(indice);
        v["epochDigest"] = json!(digest);
        v["formatVersion"] = json!("0x1");
        v["signature"] = json!("0xabcd");
        v["publicKey"] = json!(clave);
        v.to_string()
    }

    #[test]
    fn una_linea_sana_solo_falla_en_la_firma() {
        // ⚠️ El crate del testigo NO PUEDE FIRMAR —no depende de `xmss`—,
        // asi que una firma valida no se puede fabricar aqui. Lo que SI se
        // prueba: que una linea estructuralmente sana produce UN SOLO
        // hallazgo, el de la firma. Si hubiera mas, fallaria la estructura.
        //
        // ⚠⚠ **ESTE TEST CUADRABA Y MENTIA** (§266, §295): desde §294
        // `verificar` RECOMPONE antes de mirar la firma, asi que sobre esta
        // linea sintetica —que no lleva los campos de la cabeza— el
        // hallazgo sigue siendo UNO y sigue siendo de clase
        // `firma-no-verifica`, pero por OTRA razon: muere recomponiendo, no
        // verificando. El contador cuadraba; el significado, no. Queda
        // dicho aqui en vez de arreglado a la fuerza: el fixture `dia()` es
        // de ESTRUCTURA, y el contrato de campos lo ata el test de arriba.
        let r = auditar_lineas(&[dia("0x1", "0xaa", "0xdead")]);
        assert_eq!(r.lineas, 1);
        assert_eq!(r.con_firma, 1);
        assert_eq!(r.reverificadas, 0, "la firma de prueba no puede verificar");
        assert_eq!(r.hallazgos.len(), 1, "solo la firma: {:?}", r.hallazgos);
        assert_eq!(r.hallazgos[0].clase(), "firma-no-verifica");
    }

    #[test]
    fn auditar_ve_un_indice_que_retrocede() {
        // Un diario es de SOLO AÑADIR: solo puede avanzar.
        let r = auditar_lineas(&[
            dia("0x9", "0xaa", "0xdead"),
            dia("0x3", "0xbb", "0xdead"),
        ]);
        let c: Vec<_> = r.hallazgos.iter().map(|h| h.clase()).collect();
        assert!(c.contains(&"indice-retrocede"), "{c:?}");
    }

    #[test]
    fn auditar_ve_la_clave_que_cambia_a_mitad() {
        // ⚠️ Es el caso que §244 dejo abierto —el anclaje— y el auditor es
        // quien puede verlo. NO PUEDE PASAR EN SILENCIO.
        let r = auditar_lineas(&[
            dia("0x1", "0xaa", "0xdead"),
            dia("0x2", "0xbb", "0xbeef"),
        ]);
        let c: Vec<_> = r.hallazgos.iter().map(|h| h.clase()).collect();
        assert!(c.contains(&"cambio-de-clave"), "{c:?}");
    }

    #[test]
    fn auditar_rechaza_lo_ilegible_y_lo_que_no_declara_formato() {
        let r = auditar_lineas(&[
            "esto no es json".into(),
            json!({"clase": "nueva"}).to_string(),
            json!({"v": 99, "clase": "nueva"}).to_string(),
        ]);
        let c: Vec<_> = r.hallazgos.iter().map(|h| h.clase()).collect();
        assert_eq!(c, vec!["linea-ilegible", "linea-ilegible", "version-desconocida"], "{c:?}");
    }

    #[test]
    fn las_lineas_sin_firma_son_legitimas_y_no_se_reverifican() {
        let mut v = json!({});
        v["v"] = json!(DIARIO_VERSION);
        v["clase"] = json!("sin-firma");
        v["reason"] = json!("aun no ha habido latido");
        let r = auditar_lineas(&[v.to_string()]);
        assert_eq!(r.lineas, 1);
        assert_eq!(r.con_firma, 0);
        assert!(r.hallazgos.is_empty(), "{:?}", r.hallazgos);
    }

    #[test]
    fn comparar_un_diario_consigo_mismo_da_cero() {
        // ⚠️ Comprobable HOY, sin contraparte.
        let d = vec![dia("0x1", "0xaa", "0xdead"), dia("0x2", "0xbb", "0xdead")];
        assert!(comparar_lineas(&d, &d).divergencias.is_empty());
    }

    #[test]
    fn comparar_encuentra_el_mismo_indice_con_otro_digest() {
        // ⚠️⚠️ LA PROPIEDAD que §248 hizo posible: una vista dividida entre
        // partes distintas, indetectable desde un historico central.
        let a = vec![dia("0x1", "0xaa", "0xdead"), dia("0x9", "0xaaaa", "0xdead")];
        let b = vec![dia("0x1", "0xaa", "0xdead"), dia("0x9", "0xbbbb", "0xdead")];
        let c = comparar_lineas(&a, &b);
        assert_eq!(c.divergencias.len(), 1, "{c:?}");
        let d = &c.divergencias[0];
        assert_eq!(d.indice, 9);
        assert_eq!(d.digest_a, "0xaaaa");
        assert_eq!(d.digest_b, "0xbbbb");
        assert!(d.misma_clave, "firmadas con la misma clave: el operador emitio las dos");
        assert!(!c.interna_a && !c.interna_b, "ninguno se contradice a si mismo");
    }

    // ── §248: el diario, verificable y comparable ──

    fn servido(indice: &str, digest: &str, clave: &str) -> Value {
        let mut v = json!({});
        v["available"] = json!(true);
        v["index"] = json!(indice);
        v["epochDigest"] = json!(digest);
        v["domain"] = json!("ZK-SSL-epoch-head");
        v["formatVersion"] = json!("0x1");
        v["signature"] = json!("0xabcd");
        v["publicKey"] = json!(clave);
        v["emittedAtUnix"] = json!("0x64");
        v["beatSeconds"] = json!("0x3c");
        v["custody"] = json!("fichero");
        v["custodyChecked"] = json!(true);
        v
    }

    #[test]
    fn el_diario_guarda_lo_suficiente_para_reverificar_sin_el_nodo() {
        // ⚠️ ESE es el criterio que decide que campos entran.
        let v = Veredicto::Nueva { indice: 7, digest: "0xaa".into() };
        let l = linea_de_diario(&v, &servido("0x7", "0xaa", "0xdead"), 1000);
        for k in ["index", "epochDigest", "formatVersion", "signature", "publicKey"] {
            assert!(!l[k].is_null(), "sin {k} no se puede reverificar");
        }
        assert_eq!(l["v"], json!(DIARIO_VERSION), "la version va en CADA linea");
        assert_eq!(l["clase"], json!("nueva"));
        assert_eq!(l["vistoUnix"], json!(1000));
    }

    #[test]
    fn una_respuesta_sin_result_no_es_una_negativa_del_nodo() {
        // ⚠️ §314 - La fabricacion mas callada de las tres: hasta hoy esto
        // acababa en `Value::Null` y se clasificaba como `sin-firma` con el
        // `reason` AUSENTE, indistinguible de un nodo que dice que no.
        match del_cuerpo(json!({"jsonrpc": "2.0", "id": 1, "error": {"code": -32601}})) {
            Servido::SinRespuesta { motivo } => {
                assert!(motivo.contains("result"), "el motivo debe decirlo: {motivo}");
            }
            Servido::Respuesta(_) => panic!("sin `result` no hay respuesta usable"),
        }
    }

    #[test]
    fn un_result_presente_llega_tal_cual_al_juez() {
        let cuerpo = json!({"jsonrpc": "2.0", "id": 1,
                            "result": {"available": false, "reason": "sin clave"}});
        match del_cuerpo(cuerpo) {
            Servido::Respuesta(v) => {
                assert_eq!(v["reason"], json!("sin clave"), "el result no se toca");
            }
            Servido::SinRespuesta { motivo } => panic!("habia result y dio: {motivo}"),
        }
    }

    #[test]
    fn sin_respuesta_y_sin_firma_son_clases_distintas_en_el_diario() {
        // ⚠️⚠️ LA PROPIEDAD DEL §314: el diario distingue <<el nodo dijo que
        // no>> de <<no hubo nodo>>, y por ESTRUCTURA, no por prosa.
        let negativa = Veredicto::SinFirma { motivo: "sin clave".into() };
        let ausencia = Veredicto::SinRespuesta { motivo: "transporte: rechazado".into() };
        assert_ne!(negativa.clase(), ausencia.clase(), "dos clases, no una");

        let l = linea_de_diario(&negativa, &json!({"available": false, "reason": "sin clave"}), 7);
        assert_eq!(l["clase"], json!("sin-firma"));
        assert_eq!(l["reason"], json!("sin clave"), "lo que dijo el nodo va en `reason`");
        assert!(l["motivo"].is_null(), "el nodo respondio: no hay motivo del cliente");

        let a = linea_de_diario(&ausencia, &Value::Null, 7);
        assert_eq!(a["clase"], json!("sin-respuesta"));
        assert_eq!(a["motivo"], json!("transporte: rechazado"),
                   "lo que vio el cliente va en `motivo`");
        assert!(a["reason"].is_null(),
                "el nodo no dijo nada: no se le atribuye un `reason`");
        assert_eq!(a["v"], json!(3), "la clase nueva vive en el formato v3");
    }

    #[test]
    fn el_canal_de_consistencia_tambien_distingue_la_ausencia_de_respuesta() {
        let v = Veredicto::Nueva { indice: 7, digest: "0xaa".into() };
        let s = json!({"available": true, "index": "0x7", "epochDigest": "0xaa"});
        let c = Consistencia::SinRespuesta { motivo: "respuesta ilegible: x".into() };
        let l = linea_de_diario_con(&v, &s, 1000, Some(&c));
        assert_eq!(l["consistencia"]["clase"], json!("consistencia-sin-respuesta"));
        assert_eq!(l["consistencia"]["motivo"], json!("respuesta ilegible: x"));
        assert!(l["consistencia"]["reason"].is_null(),
                "el canal tampoco atribuye al nodo lo que no dijo");
    }

    #[test]
    fn el_diario_no_usa_debug_y_las_clases_son_estables() {
        // ⚠️ Una cadena de `Debug` CAMBIA si alguien toca un `derive`: un
        // formato que depende de un detalle de implementacion NO es un formato.
        let esperadas = [
            (Veredicto::Nueva { indice: 1, digest: "a".into() }, "nueva"),
            (Veredicto::Repetida { indice: 1 }, "repetida"),
            (Veredicto::Hueco { desde: 2, hasta: 3 }, "hueco"),
            (Veredicto::SinFirma { motivo: "x".into() }, "sin-firma"),
            (Veredicto::NoVerifica { indice: 1, error: "x".into() }, "no-verifica"),
            (Veredicto::VistaDividida { indice: 1, digest_a: "a".into(), digest_b: "b".into() },
             "vista-dividida"),
            (Veredicto::CambioDeClave { fijada: "a".into(), recibida: "b".into() },
             "cambio-de-clave"),
        ];
        for (v, nombre) in esperadas {
            assert_eq!(v.clase(), nombre);
            assert!(!nombre.contains("Veredicto"), "una clase no puede parecerse a un Debug");
        }
    }

    #[test]
    fn sin_cabeza_firmada_el_diario_guarda_el_motivo_y_nada_mas() {
        let v = Veredicto::SinFirma { motivo: "sin clave".into() };
        let l = linea_de_diario(&v, &json!({"available": false, "reason": "sin clave"}), 5);
        assert_eq!(l["clase"], json!("sin-firma"));
        assert_eq!(l["reason"], json!("sin clave"));
        assert!(l["signature"].is_null(), "no hay firma que guardar");
    }

    #[test]
    fn dos_diarios_revelan_la_vista_dividida_que_un_historico_central_no_puede() {
        // ⚠️⚠️ EL ARGUMENTO DECISIVO de §248. El operador sirve a A un digest
        // y a B otro para EL MISMO indice. Ninguno de los dos lo ve solo;
        // comparar las dos lineas lo revela.
        let a = linea_de_diario(&Veredicto::Nueva { indice: 9, digest: "0xaaaa".into() },
                                &servido("0x9", "0xaaaa", "0xdead"), 100);
        let b = linea_de_diario(&Veredicto::Nueva { indice: 9, digest: "0xbbbb".into() },
                                &servido("0x9", "0xbbbb", "0xdead"), 101);
        assert_eq!(a["index"], b["index"], "el mismo indice");
        assert_ne!(a["epochDigest"], b["epochDigest"], "y distinto contenido");
        assert_eq!(a["publicKey"], b["publicKey"], "firmadas con la misma clave");
        // ⚠️ Y la FIRMA es lo que impide que un testigo lo fabrique: sin ella,
        // comparar diarios no probaria nada.
        assert!(!a["signature"].is_null() && !b["signature"].is_null(),
                "sin firma, la comparacion no prueba nada");
    }

    #[test]
    fn una_cabeza_nueva_es_nueva() {
        let mut m = Memoria::nueva();
        assert!(matches!(m.clasificar(1, "0xaa"), Veredicto::Nueva { .. }));
        assert_eq!(m.vistos(), 1);
    }

    #[test]
    fn la_misma_cabeza_dos_veces_es_repetida_y_no_alarma() {
        // El testigo consulta cada minuto; si el latido no ha corrido, ve la
        // misma. Eso NO es un hallazgo.
        let mut m = Memoria::nueva();
        m.clasificar(1, "0xaa");
        let v = m.clasificar(1, "0xaa");
        assert_eq!(v, Veredicto::Repetida { indice: 1 });
        assert!(!v.detiene());
    }

    #[test]
    fn un_hueco_se_anota_y_no_detiene() {
        // ⚠️ §242 declaro que habra huecos: la firma vive en memoria y se
        // pierde al reiniciar. Si esto gritara, el PRIMER reinicio quemaria
        // la credibilidad del testigo.
        let mut m = Memoria::nueva();
        m.clasificar(1, "0xaa");
        let v = m.clasificar(5, "0xbb");
        assert_eq!(v, Veredicto::Hueco { desde: 2, hasta: 4 });
        assert!(!v.detiene(), "un hueco NO puede detener al testigo");
    }

    #[test]
    fn mismo_indice_con_otro_digest_es_vista_dividida_y_detiene() {
        // ⚠️⚠️ EL TEST QUE HACE QUE ESTO SEA UN TESTIGO. Sin el, guardar
        // firmas y comprobar que verifican es un cliente de archivo.
        let mut m = Memoria::nueva();
        m.clasificar(7, "0xaaaa");
        let v = m.clasificar(7, "0xbbbb");
        match &v {
            Veredicto::VistaDividida { indice, digest_a, digest_b } => {
                assert_eq!(*indice, 7);
                assert_eq!(digest_a, "0xaaaa");
                assert_eq!(digest_b, "0xbbbb");
            }
            otro => panic!("debia ser vista dividida y dio: {otro:?}"),
        }
        assert!(v.detiene(), "la vista dividida DEBE detener al testigo");
    }

    #[test]
    fn la_primera_clave_se_fija_y_las_iguales_pasan() {
        // ⚠️ EL ANCLA. Trust-on-first-use, como SSH.
        let mut m = Memoria::nueva();
        assert!(m.anclar("0xdead").is_none(), "la primera se fija sin alarma");
        assert_eq!(m.clave_fijada(), Some("0xdead"));
        assert!(m.anclar("0xdead").is_none(), "la misma no alarma");
    }

    #[test]
    fn un_cambio_de_clave_detiene() {
        // ⚠️⚠️ ROTAR ES COMO SE ESCAPA DE UNA VISTA DIVIDIDA: se rota, y se
        // presenta otra historia firmada con otra clave. Un testigo que
        // aceptara la nueva en silencio acepta CUALQUIER COSA desde ahi.
        let mut m = Memoria::nueva();
        m.anclar("0xdead");
        let v = m.anclar("0xbeef").expect("un cambio de clave debe dar veredicto");
        match &v {
            Veredicto::CambioDeClave { fijada, recibida } => {
                assert_eq!(fijada, "0xdead");
                assert_eq!(recibida, "0xbeef");
            }
            otro => panic!("debia ser cambio de clave y dio: {otro:?}"),
        }
        assert!(v.detiene(), "un cambio de clave DEBE detener al testigo");
    }

    #[test]
    fn solo_dos_clases_detienen() {
        for v in [
            Veredicto::Nueva { indice: 1, digest: "0xaa".into() },
            Veredicto::Repetida { indice: 1 },
            Veredicto::Hueco { desde: 2, hasta: 3 },
            Veredicto::SinFirma { motivo: "sin clave".into() },
            Veredicto::NoVerifica { indice: 4, error: "x".into() },
            Veredicto::SinRespuesta { motivo: "transporte: x".into() },
        ] {
            assert!(!v.detiene(), "no debia detener: {v:?}");
        }
        assert!(Veredicto::VistaDividida {
            indice: 1, digest_a: "a".into(), digest_b: "b".into()
        }
        .detiene());
        assert!(Veredicto::CambioDeClave { fijada: "a".into(), recibida: "b".into() }.detiene());
    }

    #[test]
    fn el_ancla_va_antes_que_la_verificacion() {
        // ⚠️ Si la clave cambio, verificar contra la NUEVA no significa nada:
        // el veredicto tiene que ser el cambio, no un fallo de firma.
        //
        // ⚠️⚠️ §312 · Y AHORA ADEMÁS ES EL GUARDIÁN DEL ORDEN. Esta fixture
        // es INVÁLIDA para `SignedEpochHeadDto` —le faltan `beatSeconds`,
        // `custody` y `custodyChecked`, y su `epochDigest` es de un byte—:
        // si el tipado se colase delante del ancla, este test moriría. Que
        // siga en verde es la prueba EJECUTABLE de que no se ha colado.
        let mut m = Memoria::nueva();
        m.anclar("0xaaaa");
        let v = una_vuelta(
            &json!({"available": true, "publicKey": "0xbbbb", "index": "0x1",
                    "epochDigest": "0x00", "signature": "0x00", "formatVersion": "0x1"}),
            &mut m,
        );
        assert!(matches!(v, Veredicto::CambioDeClave { .. }), "dio: {v:?}");
    }

    #[test]
    fn sin_firma_no_ancla_ni_alarma() {
        // El nodo sin clave responde available:false. Eso NO fija clave.
        let mut m = Memoria::nueva();
        let v = una_vuelta(
            &json!({"available": false, "reason": "el nodo arranco SIN --clave"}),
            &mut m,
        );
        assert!(matches!(v, Veredicto::SinFirma { .. }));
        assert!(!v.detiene());
        assert_eq!(m.clave_fijada(), None, "sin firma no hay clave que fijar");
    }

    /// La forma firmada COMPLETA, con las veinte claves que sirve el dispatch
    /// del nodo (`main.rs:663-717`).
    ///
    /// ⚠️ **Fuente única de los dos tests de abajo**: el que necesita una
    /// respuesta rota se la quita a ESTA, no escribe una segunda copia.
    fn cabeza_firmada_completa() -> Value {
        json!({
            "available": true,
            "beatSeconds": "0x1e",
            "custody": "fichero",
            "custodyChecked": true,
            "seq": "0x5",
            "epochDigest": "0x1111111111111111111111111111111111111111111111111111111111111111",
            "emittedAtUnix": "0x64",
            "domain": "ZK-SSL-EPOCH-HEAD",
            "formatVersion": "0x3",
            "mmrRoot": "0x2222222222222222222222222222222222222222222222222222222222222222",
            "mmrSize": "0x9",
            "index": "0x7",
            "accountsRoot": "0x3333333333333333333333333333333333333333333333333333333333333333",
            "pendingRoot": "0x4444444444444444444444444444444444444444444444444444444444444444",
            "frozenRoot": "0x5555555555555555555555555555555555555555555555555555555555555555",
            "chainDigest": "0x6666666666666666666666666666666666666666666666666666666666666666",
            "acusesRoot": "0x7777777777777777777777777777777777777777777777777777777777777777",
            "n": "0x3",
            "signature": "0xaabb",
            "publicKey": "0xccdd"
        })
    }

    #[test]
    fn la_respuesta_del_nodo_deserializa_en_el_dto_y_la_vista_trae_el_indice() {
        // ⚠️ §312 · el primer consumidor TIPADO del cable, y vive aquí: el
        // verificador no puede tener `zk-ssl-wire` sin arrastrar la capa.
        //
        // ⚠️ Este test **no pasa por `verificar`** a propósito: una firma de
        // mentira contra `verificar_cabeza` mide la librería de abajo, no la
        // migración. Lo que se prueba es el tramo nuevo, y nada más.
        let v = cabeza_firmada_completa();
        let dto = SignedEpochHeadDto::deserialize(&v).expect("la respuesta del nodo es del cable");
        let vista = dto.firmada().expect("bien formada").expect("hay cabeza firmada");
        assert_eq!(vista.index.0, 7, "el indice sale tipado, sin leer_q");
        assert_eq!(vista.custody, "fichero");
    }

    #[test]
    fn una_cabeza_sin_custody_ya_no_clasifica_y_lo_dice_por_su_nombre() {
        // ⚠️⚠️ §312 · éste es el que prueba DÓNDE vive el tipado: dentro de
        // `una_vuelta` y **después** del ancla. Con la clave sin fijar el
        // ancla deja pasar, así que lo único que puede matar la vuelta es el
        // cable — y antes de este corte esta misma respuesta clasificaba.
        let mut v = cabeza_firmada_completa();
        v.as_object_mut().expect("objeto").remove("custody");
        let mut m = Memoria::nueva();
        match una_vuelta(&v, &mut m) {
            Veredicto::NoVerifica { indice, error } => {
                assert_eq!(indice, 0, "sin forma no hay indice que declarar");
                assert!(error.starts_with("forma del cable"), "dio: {error}");
            }
            otro => panic!("una cabeza sin custody no puede clasificar: {otro:?}"),
        }
    }

    #[test]
    fn las_cantidades_se_leen_en_hexadecimal() {
        // El cable usa `{:#x}`. Leerlas en decimal fue un fallo real (§242).
        assert_eq!(leer_q(&json!("0x1")).expect("q"), 1);
        assert_eq!(leer_q(&json!("0x2a")).expect("q"), 42);
        assert!(leer_q(&json!(42)).is_err(), "un numero crudo no es QUANTITY");
    }

    #[test]
    fn el_hex_torcido_se_rechaza_sin_reventar() {
        // Un testigo recibe lo que le manden.
        assert!(leer_hex(&json!("0xabc")).is_err(), "longitud impar");
        assert!(leer_hex(&json!("0xzz")).is_err(), "no es hex");
        assert!(leer_hex(&json!(7)).is_err(), "no es cadena");
        assert_eq!(leer_hex(&json!("0x0a1b")).expect("hex"), vec![0x0a, 0x1b]);
    }

    // ── §299 · el COFIRMANTE ──

    /// ⚠️ En `target/`, no en `temp_dir()`: el guardian **se niega a operar
    /// en tmpfs** (§234, K.1) y `/tmp` suele serlo.
    fn en_disco(nombre: &str) -> std::path::PathBuf {
        let d = std::path::PathBuf::from("target").join("cofirmante_tests");
        std::fs::create_dir_all(&d).expect("crear");
        let p = d.join(nombre);
        let _ = std::fs::remove_file(&p);
        p
    }

    fn semilla_testigo() -> [u8; 96] {
        let mut s = [0u8; 96];
        for (i, b) in s.iter_mut().enumerate() {
            *b = (i as u8).wrapping_mul(31).wrapping_add(9);
        }
        s
    }

    /// Una clave de operador cualquiera, con la forma de una de verdad.
    fn clave_operador() -> Vec<u8> {
        let mut s = [0u8; 96];
        for (i, b) in s.iter_mut().enumerate() {
            *b = (i as u8).wrapping_mul(17).wrapping_add(4);
        }
        // ⚠️ Sin `mut`: aqui solo se LEE la clave publica. Con `mut` sale
        //    `unused_mut` y el cli deja de estar en cero warnings.
        let kp = KeyPair::<Conjunto>::from_seed(&s).expect("keygen");
        kp.verifying_key().as_ref().to_vec()
    }

    /// ⚠️⚠️ El tope NO sale del campo `indice` declarado sino de DENTRO
    /// de la firma, y **el numero esperado se DERIVA**, no se teclea: se lee
    /// con el mismo `indice_de_firma` que usa el tercero.
    #[test]
    fn el_tope_sale_de_dentro_de_la_firma_y_es_el_maximo() {
        let p = en_disco("tope_contador");
        let mut c = Cofirmante::desde_semilla(&semilla_testigo(), &p).expect("abrir");
        let op = clave_operador();
        let pk = c.clave_publica();
        let mut lineas: Vec<String> = Vec::new();
        let mut esperado = 0u64;
        for k in 0..3u8 {
            let d = [k; 32];
            let f = c.cofirmar(&d, &op).expect("cofirmar");
            let e = indice_de_firma(&f.firma).expect("indice embebido");
            if e > esperado {
                esperado = e;
            }
            lineas.push(linea_de_cofirma(&d, &op, &pk, &f, 0).to_string());
        }
        let fichero = en_disco("tope_fichero");
        std::fs::write(&fichero, lineas.join("\n") + "\n").expect("escribir");
        assert_eq!(tope_de_cofirmas(&fichero), Some(esperado), "el maximo embebido");
        // ⚠️ y una linea de basura se SALTA, no tumba la lectura
        std::fs::write(&fichero, lineas.join("\n") + "\nno soy json\n").expect("escribir");
        assert_eq!(tope_de_cofirmas(&fichero), Some(esperado), "la basura se salta");
    }

    /// ⚠️⚠️ **LA MITAD DEL TESTIGO DE LA NOTA 100, EJERCITADA DE
    /// PUNTA A PUNTA**: se cofirma, se REINICIA -la clave se rederiva de la
    /// semilla y vuelve a cero mientras el contador sobrevive-, se resincroniza
    /// y el testigo vuelve a firmar algo que un TERCERO verifica.
    ///
    /// ⚠️⚠️ El contador se mueve con la clave, y por eso el invariante
    /// `embebido < declarado` sigue en pie: `reservar` devuelve `contador + 1`
    /// y la clave firma con `contador`. Resincronizar la clave SIN mover el
    /// contador romperia ese invariante, y este test lo recorre por el camino
    /// bueno para que se vea cual es.
    #[test]
    fn tras_cofirmar_y_reiniciar_la_clave_se_resincroniza_y_el_testigo_sigue() {
        let p = en_disco("reinicio_testigo");
        let op = clave_operador();
        {
            let mut c = Cofirmante::desde_semilla(&semilla_testigo(), &p).expect("abrir");
            for k in 0..2u8 {
                c.cofirmar(&[k; 32], &op).expect("cofirmar");
            }
        }
        // EL REINICIO.
        let mut c = Cofirmante::desde_semilla(&semilla_testigo(), &p).expect("reabrir");
        assert_eq!(c.indice_de_la_clave().expect("indice"), 0, "el SK no se persiste");
        // ⚠️ en dos pasos, como en `run`: el temporal mantendria `c`
        //    prestado durante todo el match.
        let rec = c.reconciliar().expect("reconciliar");
        let hasta = match politica_del_cofirmante(&rec, None) {
            DecisionDelCofirmante::ArrancaResincronizando { hasta, .. } => hasta,
            otra => panic!("tras reiniciar toca resincronizar: {otra:?}"),
        };
        c.resincronizar_a(hasta).expect("resincronizar");
        assert_eq!(c.indice_de_la_clave().expect("indice"), hasta, "la clave se movio");
        let d = [0x11u8; 32];
        let f = c.cofirmar(&d, &op).expect("cofirmar tras resincronizar");
        let pk = c.clave_publica();
        verificar_cofirma(&pk, &d, &op, &f).expect("un tercero debe poder");
    }

    #[test]
    fn una_cofirma_recien_hecha_verifica_con_lo_publicado() {
        let p = en_disco("verifica");
        let mut c = Cofirmante::desde_semilla(&semilla_testigo(), &p).expect("abrir");
        let op = clave_operador();
        let d = [0x5Au8; 32];
        let firmada = c.cofirmar(&d, &op).expect("cofirmar");
        // Y un TERCERO la comprueba con lo publicado, sin el testigo.
        let pk = c.clave_publica();
        verificar_cofirma(&pk, &d, &op, &firmada).expect("un tercero debe poder");
    }

    #[test]
    fn el_indice_se_reserva_antes_de_firmar_y_los_dos_avanzan() {
        // ⚠️ EL INVARIANTE: ninguna firma puede existir con un indice mayor
        //    que el contador persistido. Por eso se reserva PRIMERO.
        let p = en_disco("orden");
        let mut c = Cofirmante::desde_semilla(&semilla_testigo(), &p).expect("abrir");
        assert_eq!(c.indice_de_la_clave().expect("indice"), 0, "clave nueva en 0");
        let op = clave_operador();
        let a = c.cofirmar(&[1u8; 32], &op).expect("una");
        let b = c.cofirmar(&[2u8; 32], &op).expect("dos");
        assert_ne!(a.indice, b.indice, "dos cofirmas NO repiten indice");
        assert_eq!(c.indice_de_la_clave().expect("indice"), 2, "la clave gasto dos");
    }

    #[test]
    fn sin_ancla_no_se_puede_cofirmar() {
        // ⚠️⚠️ LA DECISION, EN EL TIPO. Una memoria recien nacida no tiene
        //    clave anclada, asi que no hay operador a quien atestiguar y la
        //    cofirma es IMPOSIBLE — no aproximada.
        let p = en_disco("sin_ancla");
        let mut c = Cofirmante::desde_semilla(&semilla_testigo(), &p).expect("abrir");
        let m = Memoria::nueva();
        assert!(m.clave_fijada().is_none());
        match c.cofirmar_lo_anclado(&m, &[3u8; 32]) {
            Err(CofirmaError::SinAncla) => {}
            otro => panic!("sin ancla debe negarse, y dio: {otro:?}"),
        }
    }

    #[test]
    fn la_cofirma_sale_bajo_la_clave_anclada_no_bajo_otra() {
        // ⚠️⚠️ El testigo firma LO QUE ANCLO. Se ancla una clave, se cofirma
        //    por la via de la memoria, y la cofirma resultante verifica
        //    contra ESA y **no** contra otra distinta.
        let p = en_disco("anclada");
        let mut c = Cofirmante::desde_semilla(&semilla_testigo(), &p).expect("abrir");
        let op = clave_operador();
        let hex: String = op.iter().map(|b| format!("{b:02x}")).collect();
        let mut m = Memoria::nueva();
        assert!(m.anclar(&format!("0x{hex}")).is_none(), "el primer anclaje no alarma");
        let d = [0x7Eu8; 32];
        let firmada = c.cofirmar_lo_anclado(&m, &d).expect("cofirmar");
        let pk = c.clave_publica();
        verificar_cofirma(&pk, &d, &op, &firmada).expect("verifica bajo la anclada");
        let mut otra = op.clone();
        otra[10] ^= 0x01;
        assert!(
            verificar_cofirma(&pk, &d, &otra, &firmada).is_err(),
            "no puede valer para otro operador"
        );
    }

    #[test]
    fn el_cofirmante_hereda_la_negativa_en_tmpfs() {
        // ⚠️ El guardian no arranca donde `fsync` no persiste, y el testigo
        //    hereda esa negativa ENTERA: es la misma implementacion (§296).
        let p = std::path::PathBuf::from("/dev/shm").join("cofirma_tmpfs.bin");
        if !std::path::Path::new("/dev/shm").is_dir() {
            return; // sin tmpfs a mano, no hay nada que probar
        }
        let _ = std::fs::remove_file(&p);
        match Cofirmante::desde_semilla(&semilla_testigo(), &p) {
            Err(CofirmaError::Guardian(GuardianError::PersistenciaFalsa { .. })) => {}
            // ⚠️ Los dos modos de fallo, separados. Y el `Ok` NO se formatea:
            //    `Cofirmante` no implementa `Debug` A PROPOSITO (ver su doc).
            Err(otro) => panic!("en tmpfs debe negarse por PersistenciaFalsa, y dio: {otro:?}"),
            Ok(_) => panic!("en tmpfs NO debe arrancar: fsync no persiste nada ahi"),
        }
    }

    /// Un fichero de cofirmas de verdad, con `n` lineas.
    fn fichero_de_cofirmas(n: usize, nombre: &str) -> (Vec<String>, Vec<u8>) {
        let p = en_disco(nombre);
        let mut cf = Cofirmante::desde_semilla(&semilla_testigo(), &p).expect("abrir");
        let op = clave_operador();
        let mut ls = Vec::new();
        for i in 0..n {
            let d = [i as u8 + 1; 32];
            let c = cf.cofirmar(&d, &op).expect("cofirmar");
            let pk = cf.clave_publica();
            ls.push(linea_de_cofirma(&d, &op, &pk, &c, 1000 + i as u64).to_string());
        }
        (ls, op)
    }

    #[test]
    fn un_fichero_de_cofirmas_bien_hecho_verifica_sin_nada_mas() {
        // ⚠️ EL CRITERIO, EJECUTABLE. Ni nodo, ni diario, ni testigo: solo
        //    las lineas. Si esto necesitara mas, el formato no estaria hecho.
        let (ls, _) = fichero_de_cofirmas(3, "fichero_ok");
        assert!(verificar_cofirmas(&ls).is_empty(), "un fichero sano no da hallazgos");
    }

    #[test]
    fn una_cofirma_adulterada_no_pasa() {
        let (mut ls, _) = fichero_de_cofirmas(2, "fichero_adulterado");
        // Un solo nibble de la firma, y ya no verifica.
        let v: Value = serde_json::from_str(&ls[1]).expect("json");
        let f = v["firma"].as_str().expect("hex").to_string();
        let mut b: Vec<char> = f.chars().collect();
        b[20] = if b[20] == 'a' { 'b' } else { 'a' };
        let mut w = v.clone();
        w["firma"] = json!(b.into_iter().collect::<String>());
        ls[1] = w.to_string();
        let h = verificar_cofirmas(&ls);
        assert_eq!(h.len(), 1, "un solo hallazgo: {h:?}");
        assert_eq!(h[0].clase(), "no-verifica");
    }

    #[test]
    fn el_mismo_indice_sobre_mensajes_distintos_es_hallazgo() {
        // ⚠️⚠️ EL QUE MAS IMPORTA. Reusar un indice XMSS **filtra la clave**;
        //    el guardian lo impide DENTRO del testigo, y esto lo detecta
        //    DESDE FUERA — que es lo que un tercero necesita poder hacer.
        // ⚠️⚠️⚠️ §334: antes este test CLONABA la linea, o sea el MISMO
        //    mensaje, que es justo el caso que NO revela nada. Se reescribe el
        //    epochDigest: mismo indice embebido, mensaje distinto, que es lo
        //    unico que de verdad quema una clave de un solo uso.
        let (ls, _) = fichero_de_cofirmas(1, "fichero_repe");
        let mut w: Value = serde_json::from_str(&ls[0]).expect("json");
        w["epochDigest"] =
            json!("0x9999999999999999999999999999999999999999999999999999999999999999");
        let dos = vec![ls[0].clone(), w.to_string()];
        let h = verificar_cofirmas(&dos);
        assert!(h.iter().any(|x| x.clase() == "indice-repetido"), "{h:?}");
        match h.iter().find(|x| x.clase() == "indice-repetido") {
            Some(HallazgoCofirma::IndiceRepetido { linea, antes, .. }) => {
                assert_eq!((*linea, *antes), (2, 1), "dice DONDE estaba antes");
            }
            otro => panic!("{otro:?}"),
        }
    }

    #[test]
    fn duplicar_una_linea_no_es_hallazgo_ni_quema_al_testigo() {
        // ⚠️⚠️⚠️ LA NOTA 99. Antes del §334 cualquiera desacreditaba a un
        //    cofirmante honesto duplicandole una linea: el indice se repetia,
        //    saltaba `indice-repetido` y la politica lo quemaba entero. Pero la
        //    misma linea es la misma firma sobre el MISMO mensaje y no revela
        //    nada. Lo que filtra la clave son DOS mensajes.
        let (ls, _) = fichero_de_cofirmas(1, "fichero_duplicado");
        let dos = vec![ls[0].clone(), ls[0].clone()];
        let h = verificar_cofirmas(&dos);
        assert!(h.is_empty(), "duplicar una linea no acusa a nadie: {h:?}");

        let v: Value = serde_json::from_str(&ls[0]).expect("json");
        let pk = leer_hex(&v["clavePublicaTestigo"]).expect("hex");
        let mut pol = std::collections::BTreeSet::new();
        pol.insert(pk);
        let a = contar_acreditacion(&dos, &h, &pol);
        assert_eq!(a.quemados, 0, "el testigo honesto sigue limpio");
        assert!(a.acreditada(1), "y su cofirma sigue acreditando");
    }

    #[test]
    fn una_linea_forjada_no_puede_quemar_a_un_testigo_honesto() {
        // ⚠️⚠️⚠️ §334, LA OTRA MITAD. La SERIE se comprueba ANTES que la
        //    firma a proposito, asi que una copia con el epochDigest reescrito
        //    colisiona con la buena y produce `indice-repetido` aunque sea una
        //    FORJA que nadie firmo. El hallazgo se DICE -la linea es mala-,
        //    pero la MARCA no se dispara: solo queman las lineas sin ningun
        //    otro hallazgo.
        // ⚠️⚠️ Lo que se reescribe es la CLAVE DEL OPERADOR y no el
        //    epochDigest, por dos razones: demuestra que el mensaje es el
        //    PREAMBULO entero y no solo el digest, y deja la epoca intacta
        //    -si la forja trajera epoca nueva se convertiria en la ULTIMA y
        //    la acreditacion caeria por otro motivo, tapando lo que se mide.
        let (ls, _) = fichero_de_cofirmas(1, "fichero_forjado");
        let mut w: Value = serde_json::from_str(&ls[0]).expect("json");
        w["clavePublicaOperador"] = json!("0xdeadbeef");
        let dos = vec![ls[0].clone(), w.to_string()];
        let h = verificar_cofirmas(&dos);
        assert!(h.iter().any(|x| x.clase() == "indice-repetido"), "se DICE: {h:?}");
        assert!(h.iter().any(|x| x.clase() == "no-verifica"), "la forja no verifica: {h:?}");

        let v: Value = serde_json::from_str(&ls[0]).expect("json");
        let pk = leer_hex(&v["clavePublicaTestigo"]).expect("hex");
        let mut pol = std::collections::BTreeSet::new();
        pol.insert(pk);
        let a = contar_acreditacion(&dos, &h, &pol);
        assert_eq!(a.quemados, 0, "una forja no prueba reuso de nada");
        assert!(a.acreditada(1), "la cofirma buena sigue en pie");
    }

    /// ⚠️⚠️ **DOS TESTIGOS DISTINTOS PUEDEN LLEVAR EL MISMO INDICE, y eso NO
    /// es hallazgo.** Cada cofirmante tiene clave y contador propios —lo
    /// declara `--indice-cofirma`—, asi que un fichero que junte cofirmas de
    /// VARIOS repite indices por diseno. El control va DENTRO del test: el
    /// mismo testigo repitiendo indice tiene que seguir siendo hallazgo.
    ///
    /// Las lineas se construyen a mano A PROPOSITO: lo que se mide aqui es la
    /// CLAVE DEL MAPA, no la criptografia. Las firmas son invencion, asi que
    /// cada linea da ademas su `no-verifica`, y el test lo EXIGE en vez de
    /// callarlo — un test que no dice lo que espera tapa lo que cambie.
    ///
    /// ⚠️ **DESDE EL §333 LA CLAVE DEL MAPA ES EL INDICE EMBEBIDO**, asi que
    /// la firma inventada lleva un prefijo de cinco bytes con un indice de
    /// hoja (3) MENOR que el ordinal declarado (7), como en el material
    /// legitimo. Sigue sin verificar —y el test lo exige—, pero su indice ya
    /// es legible: con un solo byte estas lineas se quedaban FUERA de la
    /// serie y el test dejaba de medir lo que dice medir.
    #[test]
    fn dos_testigos_distintos_con_el_mismo_indice_no_son_hallazgo() {
        let linea = |testigo: &str, ep: &str| {
            json!({
                "v": 1,
                "epochDigest": ep,
                "clavePublicaOperador": "0xaabb",
                "clavePublicaTestigo": testigo,
                "versionFormato": "0x3",
                "indice": "0x7",
                "firma": "0x00000000030d"
            })
            .to_string()
        };
        let e1 = "0x1111111111111111111111111111111111111111111111111111111111111111";
        let e2 = "0x2222222222222222222222222222222222222222222222222222222222222222";

        // ⚠️ §334: los MENSAJES tienen que ser distintos. Con las dos lineas
        //    identicas el hallazgo no diria nada, y era el caso que quemaba a
        //    un testigo honesto.
        let mismo = verificar_cofirmas(&[linea("0xcc", e1), linea("0xcc", e2)]);
        assert!(
            mismo.iter().any(|x| x.clase() == "indice-repetido"),
            "el MISMO testigo repitiendo indice sigue siendo hallazgo: {mismo:?}"
        );

        let distintos = verificar_cofirmas(&[linea("0xcc", e1), linea("0xdd", e2)]);
        assert!(
            !distintos.iter().any(|x| x.clase() == "indice-repetido"),
            "dos testigos DISTINTOS con el mismo indice NO son hallazgo: {distintos:?}"
        );
        assert_eq!(
            distintos.iter().filter(|x| x.clase() == "no-verifica").count(),
            2,
            "las dos firmas son invencion: las dos dan no-verifica: {distintos:?}"
        );
    }

    /// ⚠️⚠️⚠️ **EL REINICIO, VISTO DESDE FUERA** (§333). Al perderse el SK la
    /// clave vuelve a cero y el firmante honesto emite ordinales NUEVOS con
    /// indices de hoja ya gastados: dos digests distintos firmados con el
    /// mismo indice, que es exactamente lo que filtra la clave. Con la clave
    /// del mapa en el ordinal declarado esto pasaba por bueno.
    #[test]
    fn el_reinicio_se_ve_aunque_el_ordinal_declarado_avance() {
        let linea = |ordinal: &str, ep: &str| {
            json!({
                "v": 1,
                "epochDigest": ep,
                "clavePublicaOperador": "0xaabb",
                "clavePublicaTestigo": "0xcc",
                "versionFormato": "0x3",
                "indice": ordinal,
                "firma": "0x00000000030d"
            })
            .to_string()
        };
        let e1 = "0x1111111111111111111111111111111111111111111111111111111111111111";
        let e2 = "0x2222222222222222222222222222222222222222222222222222222222222222";
        let h = verificar_cofirmas(&[linea("0x7", e1), linea("0x8", e2)]);
        match h.iter().find(|x| x.clase() == "indice-repetido") {
            Some(HallazgoCofirma::IndiceRepetido { linea, indice, antes, .. }) => {
                assert_eq!(
                    (*linea, *indice, *antes),
                    (2, 3, 1),
                    "el numero que se repite es el EMBEBIDO, no el declarado: {h:?}"
                );
            }
            otro => panic!("el reinicio tiene que verse: {otro:?} · {h:?}"),
        }
    }

    /// ⚠️ **UNA FIRMA QUE NO LLEGA AL ANCHO NO ENTRA EN LA SERIE** (§333): no
    /// puede repetir un indice que no tiene. Antes, dos lineas de pura
    /// invencion con el mismo ordinal declarado se contaban como reuso, y el
    /// reuso QUEMA al testigo: se le podia quemar con basura.
    #[test]
    fn una_firma_sin_indice_no_entra_en_la_serie() {
        let linea = || {
            json!({
                "v": 1,
                "epochDigest":
                    "0x1111111111111111111111111111111111111111111111111111111111111111",
                "clavePublicaOperador": "0xaabb",
                "clavePublicaTestigo": "0xcc",
                "versionFormato": "0x3",
                "indice": "0x7",
                "firma": "0xdd"
            })
            .to_string()
        };
        let h = verificar_cofirmas(&[linea(), linea()]);
        assert!(
            !h.iter().any(|x| x.clase() == "indice-repetido"),
            "una firma de un byte no da indice: no hay serie que comprobar: {h:?}"
        );
        assert_eq!(
            h.iter().filter(|x| x.clase() == "no-verifica").count(),
            2,
            "y las dos siguen muriendo por la firma: {h:?}"
        );
    }

    /// ⚠️⚠️ **LA SEXTA CLASE, Y SOLO SOBRE MATERIAL QUE VERIFICA** (§333). El
    /// atado del §332 vive al FINAL de `verificar_cofirma`, asi que solo se
    /// llega a el cuando la firma es buena y el preambulo cuadra. Se reescribe
    /// el ordinal —que la firma no acredita— y el hallazgo lo dice con clase
    /// propia en vez de esconderlo dentro de un `no-verifica`.
    #[test]
    fn el_ordinal_reescrito_tiene_su_propia_clase() {
        let (ls, _) = fichero_de_cofirmas(1, "fichero_discordante");
        let v: Value = serde_json::from_str(&ls[0]).expect("json");
        let mut w = v.clone();
        w["indice"] = json!("0x0");
        let h = verificar_cofirmas(&[w.to_string()]);
        assert_eq!(h.len(), 1, "un solo hallazgo: {h:?}");
        assert_eq!(h[0].clase(), "indice-discordante");
        match &h[0] {
            HallazgoCofirma::IndiceDiscordante { declarado, embebido, .. } => {
                assert_eq!(*declarado, 0, "es el ordinal que acabamos de reescribir");
                assert!(
                    *embebido >= *declarado,
                    "y por eso es discordante, no por un numero tecleado: {h:?}"
                );
            }
            otro => panic!("{otro:?}"),
        }
    }

    /// ⚠️⚠️ **EL REPERTORIO, ATADO** (§326). El doc de `clase` promete que
    /// estos nombres se leen desde fuera y no cambian sin subir version. Si
    /// nace una septima o alguien renombra una, esto lo dice POR SU NOMBRE y
    /// no por un numero.
    #[test]
    fn las_seis_clases_del_tercero_son_estas() {
        let todas = vec![
            HallazgoCofirma::Ilegible { linea: 1, error: "x".into() },
            HallazgoCofirma::VersionDesconocida { linea: 1, v: 9 },
            HallazgoCofirma::CampoAusente { linea: 1, campo: "v".into() },
            HallazgoCofirma::NoVerifica { linea: 1, indice: 1, error: "x".into() },
            HallazgoCofirma::IndiceRepetido {
                linea: 2,
                indice: 1,
                antes: 1,
                testigo: "0xcc".into(),
            },
            HallazgoCofirma::IndiceDiscordante { linea: 1, declarado: 0, embebido: 0 },
        ];
        let clases: Vec<&str> = todas.iter().map(|h| h.clase()).collect();
        assert_eq!(
            clases,
            vec![
                "ilegible",
                "version-desconocida",
                "campo-ausente",
                "no-verifica",
                "indice-repetido",
                "indice-discordante"
            ]
        );
        assert!(todas.iter().all(|h| h.linea() >= 1), "las SEIS llevan linea");
    }

    #[test]
    fn una_linea_incompleta_dice_que_campo_falta() {
        let (ls, _) = fichero_de_cofirmas(1, "fichero_incompleto");
        let mut v: Value = serde_json::from_str(&ls[0]).expect("json");
        v["clavePublicaTestigo"] = Value::Null;
        let h = verificar_cofirmas(&[v.to_string()]);
        assert_eq!(h.len(), 1);
        match &h[0] {
            HallazgoCofirma::CampoAusente { campo, .. } => assert_eq!(campo, "clavePublicaTestigo"),
            otro => panic!("debe decir QUE campo falta: {otro:?}"),
        }
        // Y una linea ilegible tampoco revienta.
        // ⚠⚠ Cadena SIN llaves A PROPOSITO. Un `{` dentro de un literal
        //    desplaza el ambito de `check_tests.py`, que cuenta llaves **sin
        //    excluir cadenas**, y marca como ANIDADOS los tests que vengan
        //    detras — seis, en la primera corrida del §301. Una cadena
        //    cualquiera prueba lo mismo: que lo ilegible no revienta.
        assert_eq!(verificar_cofirmas(&["no es json".into()])[0].clase(), "ilegible");
    }

    /// La politica del cliente: un fichero de texto, y nada mas.
    #[test]
    fn la_politica_se_lee_ignorando_blancos_y_comentarios() {
        let ls: Vec<String> = vec![
            "# los tres testigos que este cliente acepta".into(),
            "0xaabb".into(),
            "".into(),
            "   0xccdd   ".into(),
            "0xaabb".into(),
        ];
        let p = leer_politica(&ls).expect("se lee");
        assert_eq!(p.len(), 2, "la repetida no cuenta dos veces: {p:?}");
        assert!(p.contains(&vec![0xaa, 0xbb]));
        assert!(p.contains(&vec![0xcc, 0xdd]));
        assert!(leer_politica(&["0xzz".into()]).is_err(), "el hex torcido se dice");
    }

    /// ⚠️⚠️ EL TEST QUE MIDE LA UNIDAD DE LA k: testigos DISTINTOS,
    /// y por EPOCA. Las lineas van a mano a proposito -lo que se mide es
    /// el conteo, no la criptografia-, y por eso se le pasa `&[]` como
    /// hallazgos: es justo la razon por la que `contar_acreditacion` los
    /// recibe en vez de calcularlos.
    #[test]
    fn la_k_cuenta_testigos_distintos_y_por_epoca() {
        let linea = |ep: &str, testigo: &str| {
            json!({
                "v": 1,
                "epochDigest": ep,
                "clavePublicaOperador": "0xaabb",
                "clavePublicaTestigo": testigo,
                "versionFormato": "0x3",
                "indice": "0x7",
                "firma": "0xdd"
            })
            .to_string()
        };
        let e1 = "0x1111111111111111111111111111111111111111111111111111111111111111";
        let e2 = "0x2222222222222222222222222222222222222222222222222222222222222222";
        let ls = vec![
            linea(e1, "0xc1"),
            linea(e1, "0xc1"),
            linea(e1, "0xc2"),
            linea(e2, "0xc1"),
        ];
        let mut pol = std::collections::BTreeSet::new();
        pol.insert(vec![0xc1u8]);
        pol.insert(vec![0xc2u8]);
        let a = contar_acreditacion(&ls, &[], &pol);
        assert_eq!(a.en(e1), 2, "el mismo testigo dos veces cuenta UNO");
        assert_eq!(a.en(e2), 1, "la otra epoca va por su cuenta");
        assert_eq!(a.ultima.as_deref(), Some(e2), "la ultima linea manda");
        assert!(!a.acreditada(2), "la epoca ultima solo tiene uno");
        assert!(a.acreditada(1));
    }

    /// ⚠️⚠️ UN INDICE REPETIDO QUEMA AL TESTIGO, y sus cofirmas
    /// BUENAS tampoco cuentan: reusar el indice FILTRA la clave, y una
    /// clave filtrada no avala. La linea 1 aqui esta limpia -no lleva
    /// hallazgo- y aun asi no suma.
    #[test]
    fn un_indice_repetido_quema_al_testigo_entero() {
        let linea = |testigo: &str, idx: &str, ep: &str| {
            json!({
                "v": 1,
                "epochDigest": ep,
                "clavePublicaOperador": "0xaabb",
                "clavePublicaTestigo": testigo,
                "versionFormato": "0x3",
                "indice": idx,
                "firma": "0xdd"
            })
            .to_string()
        };
        // ⚠️ §334: las dos primeras llevan MENSAJES distintos. Eran identicas,
        //    o sea el caso inocuo, asi que este test afirmaba como deseable
        //    justo lo que la nota 99 denuncia. El hallazgo se le sigue pasando
        //    a mano: lo que se prueba aqui es la POLITICA, no la deteccion.
        let e1 = "0x1111111111111111111111111111111111111111111111111111111111111111";
        let e2 = "0x2222222222222222222222222222222222222222222222222222222222222222";
        let ls = vec![
            linea("0xc1", "0x7", e1),
            linea("0xc1", "0x7", e2),
            linea("0xc2", "0x9", e1),
        ];
        let h = vec![HallazgoCofirma::IndiceRepetido {
            linea: 2,
            indice: 7,
            antes: 1,
            testigo: "0xc1".into(),
        }];
        let mut pol = std::collections::BTreeSet::new();
        pol.insert(vec![0xc1u8]);
        pol.insert(vec![0xc2u8]);
        let a = contar_acreditacion(&ls, &h, &pol);
        assert_eq!(a.quemados, 1, "el testigo que repitio esta quemado");
        assert_eq!(
            a.por_epoca[0].1, 1,
            "solo cuenta el testigo limpio, no el quemado: {:?}",
            a.por_epoca
        );
        assert!(!a.acreditada(2));
    }

    /// ⚠️⚠️ EL TEST QUE DEMUESTRA LA PROPIEDAD ENTERA, con una
    /// cofirma XMSS DE VERDAD: la misma linea que VERIFICA no acredita si
    /// su testigo no esta NOMBRADO. Verificar no es acreditar, y hasta
    /// este sello el mando no sabia decir la diferencia.
    #[test]
    fn una_cofirma_que_verifica_no_acredita_si_el_testigo_no_esta_nombrado() {
        let (ls, _) = fichero_de_cofirmas(1, "acreditacion_real");
        assert!(verificar_cofirmas(&ls).is_empty(), "la cofirma es buena");
        let v: Value = serde_json::from_str(&ls[0]).expect("json");
        let pk = leer_hex(&v["clavePublicaTestigo"]).expect("hex");
        let h = verificar_cofirmas(&ls);

        let ajena = leer_politica(&["0xdeadbeef".into()]).expect("politica");
        let a = contar_acreditacion(&ls, &h, &ajena);
        assert_eq!(a.no_nombrados, 1, "verifica, pero nadie la nombro");
        assert!(!a.acreditada(1), "sin nombrar no hay acreditacion");

        let mut propia = std::collections::BTreeSet::new();
        propia.insert(pk);
        let b = contar_acreditacion(&ls, &h, &propia);
        assert!(b.acreditada(1), "nombrada y verificando: acredita");
        assert!(!b.acreditada(2), "una sola cofirma no llega a dos");
    }

    #[test]
    fn la_linea_de_cofirma_basta_por_si_sola_para_un_tercero() {
        // ⚠️⚠️ EL TEST QUE ATA LAS DOS LISTAS. La linea de cofirmas y lo que
        //    `verificar_cofirma` consume son **dos productores del mismo
        //    contrato**: si alguien anade un campo a la verificacion y no a
        //    la linea, el tercero se queda sin poder comprobar nada. La casa
        //    ya lo pago tres veces (§292→§293, §294→§295, §297); esta nace
        //    atada.
        let p = en_disco("autosuficiente");
        let mut cf = Cofirmante::desde_semilla(&semilla_testigo(), &p).expect("abrir");
        let op = clave_operador();
        let d = [0x3Cu8; 32];
        let c = cf.cofirmar(&d, &op).expect("cofirmar");
        let pk = cf.clave_publica();
        let l = linea_de_cofirma(&d, &op, &pk, &c, 1000);
        for k in ["v", "epochDigest", "clavePublicaOperador", "clavePublicaTestigo",
                  "versionFormato", "indice", "firma", "vistoUnix"] {
            assert!(!l[k].is_null(), "la linea no lleva {k}, y el tercero lo necesita");
        }
        // Y con SOLO la linea, sin el diario y sin el testigo, se verifica.
        let leer = |k: &str| leer_hex(&l[k]).expect("hex");
        let rec = CabezaFirmada {
            version_formato: leer_q(&l["versionFormato"]).expect("q") as u8,
            indice: leer_q(&l["indice"]).expect("q"),
            firma: leer("firma"),
        };
        let dd: [u8; 32] = leer("epochDigest").try_into().expect("32");
        verificar_cofirma(&leer("clavePublicaTestigo"), &dd,
                          &leer("clavePublicaOperador"), &rec)
            .expect("la linea SOLA basta para verificar");
    }

    /// ⚠️⚠️ **EL ATADO QUE EL §315 DEJO ESCRITO, ejecutado aqui porque
    ///    aqui viven LOS DOS PRODUCTORES.** El doc de `CofirmaDto` lo dice
    ///    con todas las letras: la linea del fichero y el DTO del cable son
    ///    dos artefactos con dos convenciones, y lo que los ata es un test
    ///    sobre el **CONJUNTO DE CLAVES**, no sobre la representacion. El
    ///    fichero manda las cantidades como cadena hex y el cable como `Q`;
    ///    unificarlos exigiria subir `COFIRMA_VERSION`, que es corte propio.
    ///
    /// ⚠️ La casa ya pago tres veces por no atar dos listas (§292→§293,
    ///    §294→§295, §297). Si alguien anade un campo a uno y no al otro,
    ///    esto lo dice y dice CUAL.
    #[test]
    fn la_linea_y_el_dto_llevan_el_mismo_conjunto_de_claves() {
        let d = [0x5Au8; 32];
        let op = vec![0xAAu8, 0xBB];
        let tg = vec![0xCCu8, 0xDD];
        let c = CabezaFirmada { version_formato: 3, indice: 7, firma: vec![0xEE] };
        let t = 1_700_000_000u64;

        let linea = linea_de_cofirma(&d, &op, &tg, &c, t);
        let dto = serde_json::to_value(cofirma_dto(&d, &op, &tg, &c, t))
            .expect("el dto serializa");

        let claves = |v: &Value| -> Vec<String> {
            let mut k: Vec<String> =
                v.as_object().expect("objeto").keys().cloned().collect();
            k.sort();
            k
        };
        let kl = claves(&linea);
        let kd = claves(&dto);
        assert_eq!(
            kl, kd,
            "la linea del fichero y el DTO del cable publican conjuntos DISTINTOS"
        );
        assert_eq!(kl.len(), 8, "son ocho claves; si cambian, se decide: {kl:?}");
    }


    // -- §320 · la recolección: del cable a la línea del testigo --------

    /// Material de cofirma **de mentira** para las pruebas de FORMA. La
    /// firma no verifica y no tiene que hacerlo: aquí se mide la
    /// representación, no la criptografía.
    fn material_320() -> ([u8; 32], Vec<u8>, Vec<u8>, CabezaFirmada, u64) {
        (
            [7u8; 32],
            vec![0xaa, 0xbb, 0xcc, 0xdd],
            vec![0x11, 0x22, 0x33, 0x44],
            CabezaFirmada { version_formato: 1, indice: 3, firma: vec![0xce; 8] },
            1_723_000_000,
        )
    }

    /// ⚠️⚠️ **§327: CORRECCION. Decía «EL TEST QUE EL §315 RESERVÓ Y NO
    /// TENÍA CASA», y la tenía** (§247: se cita, no se borra). El atado del
    /// conjunto de claves ya estaba más arriba en este mismo módulo, en
    /// `la_linea_y_el_dto_llevan_el_mismo_conjunto_de_claves`, que además
    /// fija que son OCHO.
    ///
    /// ⚠️ **Este no aserta nada que aquél no aserte**: es el MISMO
    /// invariante con otro material (`material_320`, con firma de ocho
    /// bytes y `versionFormato` 1, frente al de uno y 3). Se conserva por
    /// eso —dos materiales sobre un invariante no sobran— y no por lo que
    /// su cabecera decía. Retirarlo costaría el pin del cli, las tres sumas
    /// en los diez sitios y **resucitar el homónimo del 1024** que el §324
    /// había matado: no compensa.
    #[test]
    fn el_cable_y_la_linea_del_testigo_llevan_el_mismo_conjunto_de_claves() {
        let (d, op, tg, c, t) = material_320();
        let linea = linea_de_cofirma(&d, &op, &tg, &c, t);
        let cable = serde_json::to_value(cofirma_dto(&d, &op, &tg, &c, t))
            .expect("el DTO serializa");
        let mut ka: Vec<String> =
            linea.as_object().expect("la linea es objeto").keys().cloned().collect();
        let mut kb: Vec<String> =
            cable.as_object().expect("el cable es objeto").keys().cloned().collect();
        ka.sort();
        kb.sort();
        assert_eq!(
            ka, kb,
            "los dos productores dejaron de llevar las MISMAS claves"
        );
    }

    /// La vuelta completa: lo que el cable trae, recompuesto, es **byte a
    /// byte** lo que habría escrito un testigo local.
    #[test]
    fn recomponer_desde_el_cable_da_la_misma_linea_que_escribe_el_testigo() {
        let (d, op, tg, c, t) = material_320();
        let esperada = linea_de_cofirma(&d, &op, &tg, &c, t);
        let dto = cofirma_dto(&d, &op, &tg, &c, t);
        assert_eq!(linea_desde_dto(&dto).expect("recompone"), esperada);
    }

    /// ⚠️⚠️ **EL TEST QUE JUSTIFICA EL CORTE ENTERO.** Volcar el payload
    /// del cable tal cual mata la línea en `v` y el resto ni se mira;
    /// recompuesta, se lee entera. Se comprueba por la VARIANTE del
    /// hallazgo: la firma de este material es de mentira, así que
    /// `NoVerifica` es **lo esperado** y no un fallo.
    #[test]
    fn el_payload_del_cable_a_pelo_muere_en_v_y_recompuesto_se_lee() {
        let (d, op, tg, c, t) = material_320();
        let dto = cofirma_dto(&d, &op, &tg, &c, t);

        let crudo = serde_json::to_value(&dto).expect("serializa").to_string();
        let h = verificar_cofirmas(&[crudo]);
        assert!(
            h.iter().any(|x| matches!(x, HallazgoCofirma::CampoAusente { campo, .. }
                                      if campo.as_str() == "v")),
            "el payload del cable a pelo deberia morir por el campo v: {h:?}"
        );

        let buena = linea_desde_dto(&dto).expect("recompone").to_string();
        let r = verificar_cofirmas(&[buena]);
        assert!(
            r.iter().all(|x| matches!(x, HallazgoCofirma::NoVerifica { .. })),
            "recompuesta solo deberia fallar la FIRMA: {r:?}"
        );
    }

    /// ⚠️⚠️ **NO SE BLANQUEA UNA VERSIÓN DESCONOCIDA.** Sin este gate,
    /// `linea_de_cofirma` estamparía `COFIRMA_VERSION` sobre una cofirma
    /// que declara otra cosa, y el lector la aceptaría creyéndola nuestra.
    #[test]
    fn una_cofirma_de_version_desconocida_no_se_blanquea() {
        let (d, op, tg, c, t) = material_320();
        let mut j = serde_json::to_value(cofirma_dto(&d, &op, &tg, &c, t))
            .expect("serializa");
        j["v"] = json!(format!("{:#x}", COFIRMA_VERSION + 1));
        let futura: CofirmaDto =
            serde_json::from_value(j).expect("el cable acepta la version futura");
        assert_eq!(
            linea_desde_dto(&futura),
            Err(RecogidaRechazada::VersionDesconocida { v: COFIRMA_VERSION + 1 }),
            "una version desconocida NO puede salir como linea nuestra"
        );
    }


    /// ⚠️⚠️ **EL TEST QUE IMPIDE UNA ACUSACIÓN FABRICADA.** Recolectar dos
    /// veces no puede convertir la misma cofirma en un `IndiceRepetido`;
    /// pero una cofirma **distinta** con el mismo índice del mismo testigo
    /// tiene que seguir pasando, porque **ése sí es el hallazgo del §310**.
    #[test]
    fn recolectar_dos_veces_no_fabrica_un_indice_repetido() {
        let (d, op, tg, c, t) = material_320();
        let l = linea_de_cofirma(&d, &op, &tg, &c, t).to_string();
        let ya: std::collections::BTreeSet<String> =
            [l.clone()].into_iter().collect();
        assert!(
            cribar_repetidas(&ya, &[l.clone()]).is_empty(),
            "la MISMA cofirma no puede anadirse dos veces"
        );

        let otra = CabezaFirmada { version_formato: 1, indice: 3, firma: vec![0x99; 8] };
        let l2 = linea_de_cofirma(&d, &op, &tg, &otra, t).to_string();
        assert_ne!(l, l2, "el material de prueba deberia dar lineas distintas");
        assert_eq!(
            cribar_repetidas(&ya, &[l2.clone()]),
            vec![l2],
            "una cofirma DISTINTA con el mismo indice TIENE que pasar: es el §310"
        );
    }

    #[test]
    fn el_dominio_no_se_escribe_en_la_linea_de_cofirma() {
        // ⚠️ Ya viaja DENTRO del preambulo firmado: escribirlo tambien en el
        //    JSON serian dos marcadores que pueden discrepar (§236). Y de
        //    paso, la ceguera (e) de la nota 94 no gana un caso vivo.
        let p = en_disco("sin_dominio");
        let mut cf = Cofirmante::desde_semilla(&semilla_testigo(), &p).expect("abrir");
        let op = clave_operador();
        let c = cf.cofirmar(&[1u8; 32], &op).expect("cofirmar");
        let pk = cf.clave_publica();
        let s = linea_de_cofirma(&[1u8; 32], &op, &pk, &c, 0).to_string();
        assert!(!s.contains("ZK-SSL"), "el dominio NO va en el JSON: {s:.80}");
    }

    #[test]
    fn la_marca_del_diario_y_la_cofirma_llevan_el_mismo_indice() {
        // ⚠️ La otra mitad del contrato: la marca ligera del diario ata con
        //    la linea del fichero de cofirmas POR EL INDICE. Si divergen, un
        //    tercero no puede emparejarlas.
        let p = en_disco("misma_marca");
        let mut cf = Cofirmante::desde_semilla(&semilla_testigo(), &p).expect("abrir");
        let op = clave_operador();
        let d = [9u8; 32];
        let c = cf.cofirmar(&d, &op).expect("cofirmar");
        let pk = cf.clave_publica();
        let l = linea_de_cofirma(&d, &op, &pk, &c, 0);
        let marca = json!({ "indice": format!("{:#x}", c.indice) });
        assert_eq!(marca["indice"], l["indice"], "la marca y la cofirma deben atar");
    }

    #[test]
    fn reconciliar_ve_el_contador_y_la_clave_a_la_par() {
        // ⚠️ Tras una caida, el contador puede ir POR DELANTE de la clave:
        //    K.1 lo midio en 13 de 25. Aqui, sin caida, van a la par.
        let p = en_disco("reconciliar");
        let mut c = Cofirmante::desde_semilla(&semilla_testigo(), &p).expect("abrir");
        assert!(
            matches!(c.reconciliar().expect("reconciliar"), Reconciliacion::Coincide { indice: _ }),
            "una clave nueva y un contador nuevo COINCIDEN"
        );
        c.cofirmar(&[4u8; 32], &clave_operador()).expect("cofirmar");
        assert!(
            matches!(c.reconciliar().expect("reconciliar"), Reconciliacion::Coincide { indice: _ }),
            "tras una cofirma limpia siguen a la par"
        );
    }

    #[test]
    fn el_error_dice_que_paso_y_se_puede_leer() {
        // Debug, Display y Error desde que nace (§228, §234, §241).
        let e = CofirmaError::SinAncla;
        assert!(format!("{e}").contains("no ha anclado"));
        assert!(!format!("{e:?}").is_empty());
        let _: &dyn std::error::Error = &e;
    }

    #[test]
    fn una_cofirma_no_es_una_firma_de_cabeza() {
        // ⚠️ Dominios distintos: la cofirma del testigo NO puede presentarse
        //    como cabeza del operador, aunque el digest sea el mismo.
        let p = en_disco("dominios");
        let mut c = Cofirmante::desde_semilla(&semilla_testigo(), &p).expect("abrir");
        let op = clave_operador();
        let d = [0x11u8; 32];
        let firmada = c.cofirmar(&d, &op).expect("cofirmar");
        let pk = c.clave_publica();
        assert!(
            verificar_cabeza(&pk, &d, &firmada).is_err(),
            "una cofirma no vale como cabeza: el dominio es otro"
        );
    }
}
