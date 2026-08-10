//! # El latido: emitir cabezas de época
//!
//! Cierra el eslabón 3 de la cadena de la oponibilidad. §236 construyó el
//! firmante y §240 el testigo; **nadie emitía nada**.
//!
//! ## La cadencia ya estaba decidida, y con víctimas
//!
//! §121: **una vez por minuto, y además a demanda**. Se eligió tras medir
//! que a esa cadencia el almacenamiento **cae 60 veces** frente a firmar
//! por operación, al precio declarado de dar al operador **una ventana de
//! un minuto**. No se reabre aquí.
//!
//! Y de ella cuelgan cosas de §121: *«el plazo se cuenta en cabezas de
//! época firmadas»*, *«llega en ≤1 latido»*, y **«estirar el latido para
//! esquivar N es en sí evidencia oponible»**.
//!
//! ## ⚠️ Sin clave se CALCULA la cabeza, pero NO se firma
//!
//! El nodo **no firma por defecto**. Sin `--clave`, el latido sigue
//! corriendo y anotando la cabeza; lo que falta es la firma.
//!
//! Esto es deliberado y sigue el precedente de `--dev`: **el nodo separa
//! capacidades por bandera explícita**, y los custodios de prueba no se
//! activan solos. Firmar es la misma clase de decisión — algo que el
//! operador habilita **a sabiendas**.
//!
//! ⚠️ Y hay una razón de fondo, no de prudencia: **una firma sin custodia
//! declarada de la clave no tiene valor probatorio** (§236, §238). Un
//! latido que firmara por defecto emitiría **1.440 evidencias sin valor al
//! día**, y el riesgo real no es agotar la clave —2⁴⁰ índices a 1/min son
//! dos millones de años— sino **normalizar la emisión de evidencia sin
//! valor** hasta que alguien lea «el nodo firma cabezas» y concluya lo que
//! no es.
//!
//! Con esta forma, esa frase nace acotada: **el nodo firma cabezas si el
//! operador le entrega una clave, y el arranque lo dice.**
//!
//! ## ⚠️ Calcular y firmar son cosas distintas
//!
//! La cabeza de época **es útil por sí sola**: su `epoch_digest` está en
//! los vectores de conformidad de `zkssl/0.2`. Lo que la clave añade es la
//! firma. Por eso el código los separa: sin clave **hay cabeza**, no hay
//! firma.
//!
//! ## ⚠️ El candado, y lo que NO se ha medido
//!
//! Calcular la cabeza **lee el estado**, así que el latido toma el mismo
//! `Mutex` que `dispatch`. Un latido por minuto contra un nodo que aplica
//! a **248 op/s** (§229) es despreciable **en promedio** — pero
//! **la interacción latido/escrituras NO está medida**, y queda declarado.
//!
//! ⚠️ El `Mutex` es de `std`: **no cruza un `await`**. Se toma, se calcula,
//! se firma y se suelta **dentro de un bloque síncrono**; se duerme fuera.
//! Cruzarlo bloquearía el ejecutor entero.

use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::firma_cabeza::{CabezaFirmada, FirmanteCabeza};
use crate::App;

/// Cadencia decidida en §121, tras medir el coste de almacenamiento.
pub const LATIDO_POR_DEFECTO_S: u64 = 60;

/// Lo que produce un latido.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Latido {
    /// `seq` del registro en el momento de mirar.
    pub seq: u64,
    /// `EpochHead::digest()`, en bytes de cable.
    pub epoch_digest: [u8; 32],
    /// ⚠️ `None` si el nodo arrancó **sin `--clave`**. La cabeza existe
    /// igual; lo que falta es la firma.
    pub firma: Option<CabezaFirmada>,
    /// Segundos Unix del momento de emitirla.
    ///
    /// ⚠️ **Un testigo que pide dos veces y recibe la misma firma necesita
    /// distinguir «no ha habido latido» de «me están engañando».** El
    /// índice XMSS ya lo permite —es monótono— pero conviene que sea
    /// explícito: con esto y la cadencia, el testigo calcula si la cabeza
    /// que recibe es la que tocaba.
    pub emitida_unix: u64,
}

/// Calcula la cabeza y, **si hay firmante**, la firma.
///
/// ⚠️ El orden importa y no es casual: se toma el candado, se lee la
/// cabeza y **se suelta antes de firmar**. Firmar cuesta **144,5 ms**
/// medidos (S.3), y retener el candado durante ese tiempo pararía todas
/// las escrituras del nodo — un latido de 144 ms cada 60 s es un 0,24 %
/// del reloj, pero **retenido es un 0,24 % de parada total**, y no hace
/// falta: la cabeza ya está leída.
///
/// ## ⚠️ MEDIDO en M.1 (§252), y hasta entonces solo afirmado
///
/// L.3 lo **ejercitó** —se abrieron cuentas mientras el nodo firmaba y no
/// hubo fallo— pero **ausencia de fallo no es medida de coste** (§251).
///
/// M.1 comparó dos fases con **control**: `--latido 0` (el nodo no firma
/// nunca) frente a `--latido 1` (firma cada segundo). Doce mil escrituras
/// **en serie** por fase, y las dos colas salieron **indistinguibles**:
///
/// ```text
///                p50     p95     p99     max
///   control     0,88    0,99    1,19    7,35   ms
///   firmando    0,88    1,00    1,19    8,04   ms
/// ```
///
/// **Nueve firmas solapadas, CERO escrituras afectadas.** El pico de la
/// fase con firma quedó **+6,9 ms** sobre el p99 del control — muy lejos
/// de los **144,5 ms** que costaría una firma bloqueante.
///
/// ⚠️ **Si el candado se retuviera, se vería en el MÁXIMO, no en la
/// media**: las escrituras van en serie, así que se retrasaría **una por
/// firma** —no una fracción—, y promediar 144 ms entre doce mil da
/// **+0,01 ms**, que lo escondería del todo.
///
/// ⚠️ **Lo que M.1 NO mide**: el camino de pago (`send`/`claim`), que
/// lleva prueba STARK y es mucho más caro que abrir cuenta; la
/// concurrencia real —las escrituras van en serie **a propósito**, porque
/// en paralelo se mide rendimiento y no latencia—; y otra máquina.
pub fn latir(app: &App, firmante: Option<&mut FirmanteCabeza>) -> anyhow::Result<Latido> {
    // ── 1 · con el candado: leer, y solo leer ──
    let (seq, epoch_digest) = {
        let e = app
            .estado
            .lock()
            .map_err(|_| anyhow::anyhow!("el candado del estado esta envenenado"))?;
        let cabeza = e.layer.epoch_head();
        (cabeza.seq, zk_ssl_wire::digest_to_wire(&cabeza.digest()).0)
    };

    // ── 2 · sin el candado: firmar, que cuesta 144,5 ms ──
    let firma = match firmante {
        Some(f) => Some(f.firmar(&epoch_digest)?),
        None => None,
    };

    let emitida_unix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    Ok(Latido { seq, epoch_digest, firma, emitida_unix })
}

/// Lanza el latido en una tarea de fondo.
///
/// ⚠️ `tokio::spawn` y no un hilo: `main` es `async` y el ejecutor ya
/// existe. Pero **el candado es de `std`**, así que [`latir`] es síncrona
/// y se llama entera entre dos `await` — nunca a caballo de uno.
pub fn arrancar(app: Arc<App>, mut firmante: Option<FirmanteCabeza>, cada: Duration) {
    tokio::spawn(async move {
        let mut n: u64 = 0;
        loop {
            tokio::time::sleep(cada).await;
            n += 1;
            let t = Instant::now();
            match latir(&app, firmante.as_mut()) {
                Ok(l) => {
                    let ms = t.elapsed().as_secs_f64() * 1000.0;
                    // ⚠️ HASTA §241 ESTO SE TIRABA: se registraba una línea y
                    // el `Latido` moría al cerrar el `match`. La firma —18.519
                    // bytes— se destruía, y **el único rastro permanente era el
                    // índice consumido**: coste puro. Se corrige en §242.
                    conservar(&app, l.clone());
                    match &l.firma {
                        Some(c) => tracing::info!(
                            latido = n, seq = l.seq, indice = c.indice, ms,
                            "cabeza de epoca FIRMADA"
                        ),
                        None => tracing::info!(
                            latido = n, seq = l.seq, ms,
                            "cabeza de epoca calculada SIN FIRMAR (sin --clave)"
                        ),
                    }
                }
                Err(e) => tracing::error!(latido = n, error = %e, "el latido fallo"),
            }
        }
    });
}

/// Guarda la última cabeza, **con su propio candado**.
///
/// ⚠️ Candado aparte del estado a propósito: guardar no debe volver a
/// competir con las escrituras cuando el latido ya soltó el otro.
pub fn conservar(app: &App, l: Latido) {
    // ⚠️ **§272: ANOTAR ANTES DE PISAR.** La copia en memoria dura hasta
    // el latido siguiente; el diario es lo que sobrevive al reinicio. Si
    // anotar fallara, el latido no se pierde —la copia en memoria se
    // guarda igual—: se pierde la LINEA, no la cabeza.
    //
    // ⚠️ Y NO se propaga el error ni se aborta el latido. Perder el
    // diario no compromete la clave —eso es el guardian, con su
    // `PersistenciaFalsa`—; parar de firmar porque el disco no admite una
    // linea seria cambiar un problema pequeno por uno grande.
    if let Some(r) = app.diario.as_ref() {
        if let Err(e) = crate::diario::anotar(r, &l, &app.clave_publica_firma) {
            tracing::warn!(error = %e, "no se pudo anotar el latido en el diario");
        }
    }
    if let Ok(mut u) = app.ultima_cabeza.lock() {
        *u = Some(l);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::firma_indice::Reconciliacion;
    use serde_json::json;

    fn en_disco(nombre: &str) -> std::path::PathBuf {
        let d = std::path::Path::new("target").join(format!("latido_{nombre}"));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).expect("crear");
        d.join("indice.bin")
    }

    fn semilla() -> [u8; 96] {
        let mut s = [0u8; 96];
        for (i, b) in s.iter_mut().enumerate() {
            *b = (i as u8).wrapping_mul(11).wrapping_add(5);
        }
        s
    }

    #[test]
    fn sin_clave_hay_cabeza_pero_no_firma() {
        // ⚠️ EL TEST QUE DEFINE LA FORMA. La cabeza de epoca es util por si
        // sola —su digest esta en los vectores de 0.2—; lo que la clave
        // añade es la FIRMA. Sin clave: hay cabeza, no hay firma.
        let app = crate::tests::nodo(30);
        let l = latir(&app, None).expect("latir");
        assert!(l.firma.is_none(), "sin clave NO debe haber firma");
        assert_ne!(l.epoch_digest, [0u8; 32], "pero la cabeza SI se calcula");
    }

    #[test]
    fn con_clave_la_cabeza_va_firmada_y_verifica() {
        let app = crate::tests::nodo(30);
        let mut f = FirmanteCabeza::desde_semilla(&semilla(), en_disco("firma")).expect("abrir");
        let pk = f.clave_publica();
        let l = latir(&app, Some(&mut f)).expect("latir");
        let c = l.firma.as_ref().expect("con clave debe haber firma");
        crate::firma_cabeza::verificar_cabeza(&pk, &l.epoch_digest, c)
            .expect("un testigo debe poder verificar la cabeza del latido");
    }

    #[test]
    fn el_latido_gasta_un_indice_por_cabeza() {
        // ⚠️ Cada latido QUEMA UN INDICE. A 1/min son 1.440 al dia, y con
        // 2^40 eso son dos millones de años — pero conviene que el numero
        // sea visible y este probado, no supuesto.
        let app = crate::tests::nodo(30);
        let mut f = FirmanteCabeza::desde_semilla(&semilla(), en_disco("indices")).expect("abrir");
        for esperado in 1..=3u64 {
            let l = latir(&app, Some(&mut f)).expect("latir");
            assert_eq!(l.firma.expect("firma").indice, esperado);
        }
        assert_eq!(
            f.reconciliar().expect("reconciliar"),
            Reconciliacion::Coincide { indice: 3 },
            "el guardian y la clave deben ir juntos tras cada latido"
        );
    }

    #[test]
    fn la_cabeza_del_latido_es_la_que_sirve_el_rpc() {
        // ⚠️ Si el latido firmara OTRA cosa que la que `zkssl_epochHead`
        // publica, un testigo compararia peras con manzanas.
        let app = crate::tests::nodo(30);
        let l = latir(&app, None).expect("latir");
        let v = crate::dispatch(&app, "zkssl_epochHead", json!({})).expect("epochHead");
        let del_rpc = v["epochDigest"].as_str().expect("epochDigest");
        let del_latido = format!("0x{}", hex_de(&l.epoch_digest));
        assert_eq!(del_rpc, del_latido, "el latido y el RPC deben dar la MISMA cabeza");
    }

    fn hex_de(b: &[u8; 32]) -> String {
        b.iter().map(|x| format!("{x:02x}")).collect()
    }

    #[test]
    fn una_cabeza_que_cambia_da_un_digest_distinto() {
        // Si el estado se mueve, la cabeza tambien: de otro modo firmar
        // cada minuto no acreditaria nada nuevo.
        let app = crate::tests::nodo(30);
        let antes = latir(&app, None).expect("latir").epoch_digest;
        crate::tests::cuenta(&app, 900, 1_000);
        let despues = latir(&app, None).expect("latir").epoch_digest;
        assert_ne!(antes, despues, "abrir una cuenta debe mover la cabeza");
    }

    #[test]
    fn la_cadencia_por_defecto_es_la_decidida() {
        // §121: una vez por minuto, decidido tras medir el almacenamiento.
        assert_eq!(LATIDO_POR_DEFECTO_S, 60);
    }
}
