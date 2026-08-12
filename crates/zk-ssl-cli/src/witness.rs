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
use zk_ssl_verify::{verificar_cabeza, CabezaFirmada};

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

/// Versión del formato del diario.
///
/// ⚠️ **Va en CADA LÍNEA, no en una cabecera.** Dos testigos que comparan
/// diarios necesitan saber que hablan el mismo idioma, y las líneas tienen
/// que seguir siendo independientes: se concatenan, se parten y se envían
/// sueltas. Es el argumento de §236 —*un campo vacío miente, una versión
/// dice la verdad*— aplicado al archivo.
pub const DIARIO_VERSION: u8 = 1;

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
        for k in ["index", "epochDigest", "domain", "formatVersion", "signature",
                  "publicKey", "emittedAtUnix", "beatSeconds", "custody",
                  "custodyChecked"] {
            if !servido[k].is_null() {
                l[k] = servido[k].clone();
            }
        }
    } else if let Some(r) = servido["reason"].as_str() {
        l["reason"] = json!(r);
    }
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
    verificar_cabeza(&clave, &digest, &c).map_err(|e| format!("{e}"))
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
        let servido = &servido;

        println!("[{n}] {veredicto:?}");
        if let Some(f) = diario.as_mut() {
            // ⚠️ SOLO AÑADIR. El fichero se abre en modo `append`: un diario
            // que se puede reescribir tiene el mismo problema que un
            // historico servido por el operador, solo que con otro dueño.
            let t = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            writeln!(f, "{}", linea_de_diario(&veredicto, servido, t))?;
            f.flush()?;
        }

        if veredicto.detiene() {
            // ⚠️ SE DETIENE. Seguir seria sobrescribir el hallazgo con ruido.
            eprintln!();
            eprintln!("⚠️⚠️ EL TESTIGO SE DETIENE: {veredicto:?}");
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
