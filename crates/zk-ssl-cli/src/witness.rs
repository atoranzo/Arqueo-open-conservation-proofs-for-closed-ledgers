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
use serde_json::{json, Value};
// ⚠️ Import EXPLICITO, nunca glob: `zk-ssl-verify` reexporta **su propio**
// `Veredicto` (el de `reverificacion`, §279), homonimo del de este modulo.
use xmss::KeyPair;
use zk_ssl_guardian::{GuardianError, GuardianIndice, Reconciliacion};
use zk_ssl_verify::{
    mmr, preambulo_cofirma, verificar_cabeza, verificar_cofirma, CabezaFirmada, Conjunto,
    VerificaError, VERSION_FORMATO,
};

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
pub const DIARIO_VERSION: u8 = 2;

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
        Consistencia::NoAplica => {}
    }
    l["consistencia"] = o;
    l
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
    let indice = match leer_q(&v["index"]) {
        Ok(i) => i,
        Err(e) => return Veredicto::NoVerifica { indice: 0, error: e },
    };
    let digest = v["epochDigest"].as_str().unwrap_or_default().to_string();
    // ── 2 · verificar, con el MISMO codigo que usa el firmante ──
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

    let mut m = Memoria::nueva();
    let agente = ureq::AgentBuilder::new().timeout(Duration::from_secs(10)).build();
    let mut diario = a
        .diario
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
        let servido = match agente.post(&a.nodo).send_json(cuerpo) {
            Err(e) => json!({"available": false, "reason": format!("transporte: {e}")}),
            Ok(r) => match r.into_json::<Value>() {
                Err(e) => json!({"available": false, "reason": format!("respuesta ilegible: {e}")}),
                Ok(v) => v.get("result").cloned().unwrap_or(Value::Null),
            },
        };
        let veredicto = una_vuelta(&servido, &mut m);

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
            let r = match agente.post(&a.nodo).send_json(peticion) {
                Err(e) => json!({"available": false, "reason": format!("transporte: {e}")}),
                Ok(r) => match r.into_json::<Value>() {
                    Err(e) => json!({"available": false, "reason": format!("respuesta ilegible: {e}")}),
                    Ok(v) => v.get("result").cloned().unwrap_or(Value::Null),
                },
            };
            cons = Some(al_llegar_camino(&mut m, &r));
        }
        let servido = &servido;

        match &cons {
            Some(c) => println!("[{n}] {veredicto:?} · {c:?}"),
            None => println!("[{n}] {veredicto:?}"),
        }
        if let Some(f) = diario.as_mut() {
            // ⚠️ SOLO AÑADIR. El fichero se abre en modo `append`: un diario
            // que se puede reescribir tiene el mismo problema que un
            // historico servido por el operador, solo que con otro dueño.
            let t = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            writeln!(f, "{}", linea_de_diario_con(&veredicto, servido, t, cons.as_ref()))?;
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

// ⚠️⚠️ **MUERTO A PROPOSITO HASTA EL §300.** El bucle todavia no cofirma
// —el mando va aparte para que este corte sea auditable—, asi que
// `cargo build` ve la pieza sin llamantes. Los TESTS si la usan, por eso
// `cargo test` casi no se queja: es la ceguera (b) de la nota 94 vista
// desde el otro lado, y aqui la estructura del corte la provoca.
//
// ⚠️ **Este `allow` LO QUITA EL §300**, y su asiento lo declara. Un `allow`
// sin fecha de caducidad se queda para siempre — y entonces tapa el
// dead_code de verdad que venga despues.
#[allow(dead_code)]
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
#[allow(dead_code)]
pub struct Cofirmante {
    par: KeyPair<Conjunto>,
    guardian: GuardianIndice,
}

#[allow(dead_code)]
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
            .map_err(|e| CofirmaError::Xmss(format!("{e:?}")))?;
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
            .map_err(|e| CofirmaError::Xmss(format!("{e:?}")))?;
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(l["v"], json!(2), "la version del diario SUBE");
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
