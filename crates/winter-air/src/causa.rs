// ARQUEO (§711): fichero NUEVO del fork; ver el README del crate.

//! Las CAUSAS con nombre de una prueba mal formada.
//!
//! Upstream valida la cabecera de una prueba con `assert!`: sus bytes los pone quien la envia, y
//! un byte a cero hacia entrar en panico al verificador (medido: el kit salia con 101). El fork
//! devuelve `DeserializationError::InvalidValue` y empieza su texto por una de estas causas; detras
//! va el detalle, que es el texto de upstream y no se pina.
//!
//! Son la UNICA fuente de estos textos. Los manifiestos de `spec/vectors/` los pinan y la segunda
//! implementacion (`tools/segunda/stark.py`) los repite: si uno cambia aqui, cambia alli.
//!
//! ⚠️ ASCII a proposito: un ancla con tilde en un manifiesto no falla, se queda muda.

/// Un byte de las opciones de la prueba fuera de lo que `ProofOptions::new` admite: consultas,
/// expansion, molienda, extension, plegado, grado del resto, agrupado o particiones.
pub const OPCIONES: &str = "prueba mal formada: opciones fuera de rango";

/// La traza que declara la prueba no la admite `TraceInfo::new_multi_segment`: anchos,
/// aleatorios o una longitud que no cabe en 32 bits.
pub const TRAZA: &str = "prueba mal formada: traza fuera de rango";

/// El contexto no lo admite `Context::new`: un dominio de extension que no cabe en 32 bits o un
/// numero de restricciones fuera de `1..=u32::MAX`.
pub const CONTEXTO: &str = "prueba mal formada: contexto fuera de rango";

/// El numero de consultas unicas es cero o mayor que el de las opciones, o una tabla de consultas
/// no tiene forma.
pub const CONSULTAS: &str = "prueba mal formada: consultas fuera de rango";

/// El marco fuera del dominio no tiene dos filas, o le falta ancho.
pub const MARCO_OOD: &str = "prueba mal formada: marco OOD fuera de rango";

/// La prueba FRI no tiene las capas que el dominio exige, o declara particiones que `winter-fri`
/// 0.13.1 no escribe nunca.
pub const FRI: &str = "prueba mal formada: FRI fuera de rango";
