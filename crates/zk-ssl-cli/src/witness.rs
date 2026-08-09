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
}

pub fn run(a: WitnessArgs) -> anyhow::Result<()> {
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
