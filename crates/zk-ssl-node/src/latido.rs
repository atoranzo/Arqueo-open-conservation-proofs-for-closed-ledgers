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
//! ⚠️ **CORREGIDO (§636): el «a demanda» NO existe.** Se decidió en §115 y §121 y nunca se
//! construyó: el único que llama a `firmar` es este latido, y `zkssl_signedEpochHead` sirve
//! la última cabeza que el latido conservó, sin firmar otra. Lo vio de pasada el §455, y el
//! RFC-0008 descartó firmar a petición porque quema índices XMSS. Lo que rige es el latido:
//! la ventana es de un latido, sin atajo. La línea de arriba queda corregida aquí en vez de
//! borrarla (§247), y el test `pedir_la_cabeza_firmada_no_firma_otra` lo ata.
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
//! los vectores de conformidad de `zkssl/0.3`. Lo que la clave añade es la
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
use zk_ssl::foto_pendientes::FotoPendientes;
use zk_ssl::log::EpochHead;

/// Cadencia decidida en §121, tras medir el coste de almacenamiento.
pub const LATIDO_POR_DEFECTO_S: u64 = 60;

/// Lo que produce un latido.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Latido {
    /// `seq` del registro en el momento de mirar.
    pub seq: u64,
    /// La cabeza **entera** (§275): los siete campos que el digest
    /// compone. Viaja en el latido para que `zkssl_signedEpochHead`
    /// sirva campos+digest+firma **juntos**, de la misma custodia — sin
    /// carrera entre una llamada que trae la firma y otra los campos.
    pub cabeza: EpochHead,
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
    /// **RFC-0008 D-F (§493): la FOTO de los pendientes**, tomada bajo el
    /// MISMO candado que compone la cabeza: sus dos raices son
    /// `cabeza.pending_root` y `cabeza.pmeta_root`, y `zkssl_pendingPath`
    /// sirve de ella. `Arc` porque `conservar` clona el latido y la foto
    /// pesa; la igualdad es la de `FotoPendientes` (por raices). Tras un
    /// reinicio no hay foto hasta el primer latido, como no hay cabeza.
    pub foto: Arc<FotoPendientes>,
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
    // ── 0 · sin el candado del estado: el limite de la epoca en curso ──
    // `limite_de_epoca` toma `ultima_cabeza` (y lee el diario). Tomarlo
    // AQUI evita solapar los dos candados; el orden establecido
    // estado -> ultima_cabeza no se toca.
    let limite_anterior = limite_de_epoca(app);
    // §570: y el `Q` de la recepcion, por la misma razon y con el mismo orden de fuentes.
    let limite_recepcion = limite_de_recepcion(app);

    // ── 1 · con el candado: leer, componer la pareja, y solo eso ──
    //
    // ⚠️ §275: pares -> pareja -> cabeza, TODO bajo el MISMO candado.
    // Soltarlo entre extraer las entradas y componer la cabeza dejaria
    // que el registro avanzara en medio: la raiz de acuses describiria
    // un arbol que la cabeza ya no cierra. ⚠️ El coste del arbol DENTRO
    // del candado NO esta medido (menos entradas por epoca que las 12k
    // de M.1, pero arbol nuevo): se medira con el metodo de M.1 —dos
    // fases con control—, y hasta entonces queda declarado aqui.
    // §292: la pareja del MMR se lee ANTES y con SU candado — no depende
    // del estado de la capa, y meterla dentro alargaria el candado gordo.
    let (cima_mmr, t_mmr) = pareja_mmr(app)?;
    // ⚠ §318 - EL PRE-FILTRO VA AQUI PORQUE EL SLICE VIVE CON EL
    // CANDADO, y en el caso normal es O(1): `len()` no recorre nada, y
    // `len() < N` implica `pagos < N` porque cada pago escribe al menos una
    // entrada. Solo al cruzar se paga el recorrido, y en cuanto la bandera
    // queda puesta se vuelve a O(1) para siempre. ⚠ Entre `len() = N` y
    // `pagos = N` SI hay una ventana en que se recorre cada vuelta: es una
    // pasada mas sobre un slice que `pares` ya recorre entero, y se declara
    // en vez de fingir que no existe. El aviso se emite FUERA, en el paso 2.
    let (cabeza, epoch_digest, pagos_al_cruzar, foto) = {
        let e = app
            .estado
            .lock()
            .map_err(|_| anyhow::anyhow!("el candado del estado esta envenenado"))?;
        let entradas = e.layer.transition_log().entries();
        let pagos_al_cruzar = if app
            .aviso_acumulacion
            .load(std::sync::atomic::Ordering::Relaxed)
            || entradas.len() < AVISO_ACUMULACION_PAGOS
        {
            None
        } else {
            Some(pagos_registrados(entradas))
        };
        let (acuses_root, n) = crate::vista_acuses::pareja_de_ahora(entradas, limite_anterior);
        // §570 (RFC-0010 E2d): la pareja de recepcion, BAJO el mismo candado: ver
        // `pareja_de_recepcion`, es lo que impide leer un `rx` reservado y sin anotar.
        let (recep_root, recep_count) = pareja_de_recepcion(app, limite_recepcion)?;
        let cabeza =
            e.layer.epoch_head(acuses_root, n, cima_mmr, t_mmr, recep_root, recep_count);
        // §493 (D-F): la foto, AQUI, con la cabeza que la describe. Fuera del
        // candado habria una ventana con cabeza nueva y foto vieja.
        let foto = Arc::new(e.layer.foto_pendientes());
        (
            cabeza,
            zk_ssl_wire::digest_to_wire(&cabeza.digest()).0,
            pagos_al_cruzar,
            foto,
        )
    };

    // §318: el aviso, FUERA del candado. Ver `avisar_acumulacion`.
    if let Some(pagos) = pagos_al_cruzar {
        avisar_acumulacion(app, pagos);
    }

    // ── 2 · sin el candado: firmar, que cuesta 144,5 ms ──
    let firma = match firmante {
        Some(f) => Some(f.firmar(&epoch_digest)?),
        None => None,
    };
    // §569: la era de los recibos sigue a la FIRMA, no a la composicion: se
    // publica el indice en cuanto la firma existe (§567, D-D).
    if let Some(c) = &firma {
        app.indice_firma.store(c.indice, std::sync::atomic::Ordering::Release);
    }

    let emitida_unix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    Ok(Latido { seq: cabeza.seq, cabeza, epoch_digest, firma, emitida_unix, foto })
}

/// El límite anterior de la época en curso: el `seq` de la última
/// cabeza emitida. **P sale del DIARIO; la memoria es caché** (§275).
///
/// Orden: `ultima_cabeza` si hay, el diario si no, y 0 en último
/// término. ⚠️ El borde del REINICIO, declarado (la forma de §241): un
/// nodo **sin `--diario`** que reinicia pierde P y su primera época
/// sale gorda — desde 0 hasta el primer latido. Las hojas siguen siendo
/// correctas (la época de cada una es `seq+1`, fijada en el apply); lo
/// que engorda es UN árbol. Con `--diario`, P sobrevive.
pub fn limite_de_epoca(app: &App) -> u64 {
    if let Ok(u) = app.ultima_cabeza.lock() {
        if let Some(l) = u.as_ref() {
            return l.seq;
        }
    }
    app.diario
        .as_ref()
        .and_then(|r| crate::diario::ultimo_seq(r))
        .unwrap_or(0)
}

/// `Q`: el `recep_count` de la ultima cabeza emitida (RFC-0010 D-C; E2d, §570). Mismo orden
/// que [`limite_de_epoca`]: la memoria, el diario si no, y 0 en ultimo termino. ⚠️ El borde del
/// REINICIO, declarado: sin `--diario`, o con un diario anterior al §570 que no lo guardaba, la
/// primera era sale gorda -desde 0-. Las hojas siguen siendo correctas; lo que engorda es UN
/// arbol.
pub fn limite_de_recepcion(app: &App) -> u64 {
    if let Ok(u) = app.ultima_cabeza.lock() {
        if let Some(l) = u.as_ref() {
            return l.cabeza.recep_count;
        }
    }
    app.diario
        .as_ref()
        .and_then(|r| crate::diario::ultimo_recep_count(r))
        .unwrap_or(0)
}

/// **La pareja `(recep_root, recep_count)`** que la cabeza va a firmar (RFC-0010 E2d, §570):
/// el arbol de los recibos `(Q, R]` del REGISTRO, con `R` el contador de recepcion.
///
/// ⚠️⚠️ **SE LLAMA CON EL CANDADO DEL ESTADO TOMADO**, y es lo que la hace coherente: el
/// despacho retiene ese candado mientras `recibir` y `anotar`, asi que ningun `rx <= R` puede
/// estar reservado y todavia sin anotar mientras esto lee. Orden de candados: estado ->
/// recepcion y estado -> registro, el mismo del despacho.
///
/// ⚠️ Es el **UNICO productor** de la pareja: la usan el latido, `zkssl_epochHead` y
/// `zkssl_inclusionReceipt`, y el test «la cabeza del latido es la que sirve el RPC» los ata.
///
/// ⚠️ **Falla CERRADA si el contador ha retrocedido** por debajo del `recep_count` de la
/// cabeza anterior: firmar una cuenta que BAJA es firmar que dos eras comparten numeros.
pub fn pareja_de_recepcion(
    app: &App,
    limite_anterior: u64,
) -> anyhow::Result<(zk_ssl_verify::acuses::Digest, u64)> {
    let r = app
        .recepcion
        .lock()
        .map_err(|_| anyhow::anyhow!("el candado del contador de recepcion esta envenenado"))?
        .actual();
    if r < limite_anterior {
        anyhow::bail!(
            "EL CONTADOR DE RECEPCION HA RETROCEDIDO: R = {r} por debajo del recepCount de la \
             cabeza anterior, {limite_anterior}. Componer firmaria una cuenta que baja: no se \
             compone"
        );
    }
    let entradas = app
        .registro
        .lock()
        .map_err(|_| anyhow::anyhow!("el candado del registro de recepcion esta envenenado"))?
        .entradas_posteriores_a(limite_anterior)
        .map_err(|e| anyhow::anyhow!("el registro de recepcion: {e}"))?;
    crate::vista_recibos::pareja_de_ahora(&entradas, limite_anterior, r)
        .map_err(|e| anyhow::anyhow!("la vista de recibos: {e:?}"))
}

/// **El umbral del aviso de acumulacion, en PAGOS** (§318).
///
/// ⚠ **NO es una bandera de la linea de ordenes, y eso es deliberado.**
/// `--max-cofirmas`, `--latido` y `--limit` son recursos del despliegue y el
/// operador los ajusta; esto es **la escala que el proyecto declara y que la
/// nota 22 publica**. Si cada despliegue la moviera, un test solo podria
/// atar el valor por defecto y lo que sonara en produccion podria no
/// corresponder a lo publicado: dos productores del mismo contrato sin
/// atar, que es la figura que el §304 vino a reparar.
///
/// **Por que 100.000 y no otro.** Son dos ordenes de magnitud por encima
/// del unico punto medido -mil pagos = 139,2 a 160,2 MiB, la banda de `metrics.rs`- y
/// con `PUBLICADA_PAGO_MAX_B` salen unos 15,6 GiB. **Es la ultima escala en la que
/// la copia del auditor NO DUELE**: cruzarla no dice que ya duela, dice que
/// **se acabo el margen**. Escrito al reves -"la ultima escala rutinaria"-
/// el aviso quedaria desacreditado el dia que sonara, porque 15,6 GiB se
/// descargan de una sentada.
///
/// ⚠ **Lo que este numero NO sostiene: no hay cruce de curvas.**
/// Agregar cuesta ~65 s de GPU por prueba y ahorra ~67 KB por prueba a
/// CUALQUIER escala (§307), asi que la razon es constante y no existe un
/// punto en que la agregacion se vuelva rentable por tamano. Lo que no es
/// lineal es la capacidad de quien descarga, y por eso el disparador va
/// sobre stock acumulado y no sobre una comparacion de costes.
///
/// **Silenciarlo sin desactivarlo:** el aviso lleva `target` propio, asi que
/// `--log zk_ssl_node::acumulacion=off` lo apaga y deja el resto en pie. El
/// mensaje lo dice dentro, porque `init_tracing` usa `.with_target(false)` y
/// sin eso el nombre del target seria indescubrible.
pub(crate) const AVISO_ACUMULACION_PAGOS: usize = 100_000;

/// Cuantos **PAGOS** registra el log de transiciones.
///
/// **Un pago es UNA entrada, y de que clase depende de la ERA.** La via de
/// dos fases escribe `Send` y `Claim` por pago, asi que los `Send` cuentan
/// los pagos; la via de un paso, retirada, escribia UNA entrada `Transfer`.
/// Se suman las dos: contar solo `Send` infravalora los pagos viejos.
///
/// ⚠ **Y el byte por pago SOBREVALORA la era 1, a proposito.**
/// `PUBLICADA_PAGO_MAX_B` se midio con la via de dos fases; aplicarlo tambien a
/// los `Transfer` -que costaban unos 59.100 B- adelanta el aviso. Es la
/// direccion correcta para un despertador, y va declarado en vez de
/// escondido.
pub(crate) fn pagos_registrados(entradas: &[zk_ssl::log::LogEntry]) -> usize {
    entradas
        .iter()
        .filter(|e| {
            matches!(
                e.kind,
                zk_ssl::log::OpKind::Send | zk_ssl::log::OpKind::Transfer
            )
        })
        .count()
}

/// Avisa **UNA SOLA VEZ** de que la acumulacion cruzo la escala declarada.
///
/// Lo garantiza el `swap`: quien pone la bandera a `true` es quien avisa, y
/// el que llegue despues sale por la puerta de arriba. Sin eso, con
/// `--latido 1` serian 86.400 avisos al dia.
///
/// El mensaje lleva **los bytes dentro y no solo el recuento**, que es lo
/// que pide el §254: la magnitud que le importa a quien reverifica es
/// cuanto tiene que descargarse, no cuantos pagos hubo.
pub(crate) fn avisar_acumulacion(app: &App, pagos: usize) {
    if pagos < AVISO_ACUMULACION_PAGOS {
        return;
    }
    if app
        .aviso_acumulacion
        .swap(true, std::sync::atomic::Ordering::Relaxed)
    {
        return;
    }
    let bytes = pagos as u128 * zk_ssl::PUBLICADA_PAGO_MAX_B as u128;
    let gib = format!("{:.1}", bytes as f64 / (1024.0 * 1024.0 * 1024.0));
    tracing::warn!(
        target: "zk_ssl_node::acumulacion",
        pagos = pagos as u64,
        gib = gib.as_str(),
        "la acumulacion cruzo la escala declarada en la nota 22: esto es lo \
         que tendria que descargarse quien reverifique sin fiarse del \
         operador. Silenciar sin desactivar: --log zk_ssl_node::acumulacion=off"
    );
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
    // §292: la hoja del MMR entra AQUI, en el mismo sitio que anota el
    // diario — cache y diario avanzan juntos, y tras un reinicio la
    // siembra del diario reconstruye exactamente esta serie (P-doctrina:
    // el diario manda; la memoria es cache).
    let hoja = l.cabeza.digest();
    if let Ok(mut u) = app.ultima_cabeza.lock() {
        *u = Some(l);
    }
    if let Ok(mut h) = app.hojas_mmr.lock() {
        h.push(hoja);
    }
}

/// La pareja `(cima, t)` que la cabeza de este latido va a firmar
/// (§292): la cima del MMR sobre las cabezas YA emitidas, y cuantas son.
///
/// ⚠️ Genesis declarado: sin hojas, `(as_digest(0), 0)` — el valor que
/// la composicion v3 fija. ⚠️ La cima se RECOMPONE de las hojas en cada
/// latido: O(hojas) por latido, despreciable a 1/min durante meses
/// (§292 lo declara y deja los picos incrementales anotados en el
/// BACKLOG — no se optimiza sin medir).
///
/// ⚠️ §667: con el candado envenenado devolvia el genesis, `(as_digest(0), 0)`,
/// y el latido FIRMABA una cabeza que decia «ninguna cabeza emitida»: una
/// historia vacia bajo la firma del operador. Ahora es un error, y el latido
/// no compone ni firma, como con el candado del estado.
///
/// ⚠️ §673: la cima ya no se recompone de todas las hojas: la da el
/// [`ArbolMmr`] (la frontera del §673, con todos sus niveles desde el §676),
/// que se pone al dia con las hojas nuevas. Medido por el
/// segundo enjambre, recomponerla costaba 0,95 s a 43.830 hojas (un mes) y
/// 11,6 s a 525.960 (un año), y se hacia en cada latido y en cada
/// `zkssl_epochHead`.
pub fn pareja_mmr(app: &App) -> anyhow::Result<(zk_ssl_verify::acuses::Digest, u64)> {
    let h = app
        .hojas_mmr
        .lock()
        .map_err(|_| anyhow::anyhow!("el candado de las hojas del MMR esta envenenado"))?;
    let mut f = app
        .arbol_mmr
        .lock()
        .map_err(|_| anyhow::anyhow!("el candado del arbol del MMR esta envenenado"))?;
    f.al_dia(&h);
    let cima = f.cima().unwrap_or_else(|| zk_ssl_verify::acuses::as_digest(0));
    Ok((cima, h.len() as u64))
}

/// §673: la frontera de un arbol de Merkle de RFC 6962 (la `mth` de
/// `zk_ssl_verify::mmr`), para que la cima cueste O(log t).
///
/// §676: ya no guarda solo la frontera sino TODOS los subarboles perfectos
/// alineados: `niveles[h][j]` es la `mth` de las hojas `[j·2^h, (j+1)·2^h)`.
/// Con ellos la `mth` de cualquier tramo que la particion de RFC 6962 pide
/// cuesta O(log t), y la prueba de consistencia O(log² t) en vez de O(t):
/// medido por el segundo enjambre, `zkssl_consistencyProof` recomponia 11,5 s
/// a 525.960 hojas (un año al latido), con el candado de las hojas tomado y
/// el latido esperando. Cuesta unos 2·t digests de memoria (unos 34 MB al
/// año). Añadir una hoja sigue siendo O(1) amortizado. `zk_ssl_verify::mmr`
/// es su oraculo en los tests: la misma cima y los mismos caminos.
#[derive(Default)]
pub struct ArbolMmr {
    niveles: Vec<Vec<zk_ssl_verify::acuses::Digest>>,
    hojas: usize,
}

impl ArbolMmr {
    fn push(&mut self, hoja: zk_ssl_verify::acuses::Digest) {
        if self.niveles.is_empty() {
            self.niveles.push(Vec::new());
        }
        self.niveles[0].push(zk_ssl_hash::mmr_hoja(hoja));
        let mut h = 0;
        while self.niveles[h].len() % 2 == 0 {
            let n = self.niveles[h].len();
            let nodo = zk_ssl_hash::mmr_nodo(self.niveles[h][n - 2], self.niveles[h][n - 1]);
            if self.niveles.len() == h + 1 {
                self.niveles.push(Vec::new());
            }
            self.niveles[h + 1].push(nodo);
            h += 1;
        }
        self.hojas += 1;
    }

    /// Pone el arbol al dia con `hojas`: añade las que falten. Si `hojas`
    /// fuera mas corta que lo ya visto -no pasa: solo se añaden-, empieza de
    /// cero en vez de servir una cima de otra serie.
    pub(crate) fn al_dia(&mut self, hojas: &[zk_ssl_verify::acuses::Digest]) {
        if hojas.len() < self.hojas {
            *self = ArbolMmr::default();
        }
        for h in &hojas[self.hojas..] {
            self.push(*h);
        }
    }

    /// La `mth` de las hojas `[a, b)`, con `a < b <= hojas`. La particion de
    /// RFC 6962 solo pide tramos cuyo `a` esta alineado a la mitad que corta,
    /// asi que todo bloque perfecto que aparece esta en `niveles`.
    fn mth(&self, a: usize, b: usize) -> zk_ssl_verify::acuses::Digest {
        let n = b - a;
        if n.is_power_of_two() && a % n == 0 {
            let h = n.trailing_zeros() as usize;
            return self.niveles[h][a >> h];
        }
        let k = mitad(n);
        zk_ssl_hash::mmr_nodo(self.mth(a, a + k), self.mth(a + k, b))
    }

    fn cima(&self) -> Option<zk_ssl_verify::acuses::Digest> {
        (self.hojas > 0).then(|| self.mth(0, self.hojas))
    }

    /// El camino de consistencia de `viejo` hojas a todas: el `SUBPROOF` de
    /// `zk_ssl_verify::mmr::prueba_de_consistencia`, con la misma recursion,
    /// y la `mth` de cada tramo de los `niveles`.
    pub fn prueba_de_consistencia(&self, viejo: u64) -> Option<Vec<zk_ssl_verify::acuses::Digest>> {
        if viejo == 0 || viejo > self.hojas as u64 {
            return None;
        }
        let mut camino = Vec::new();
        self.subprueba(0, self.hojas, viejo as usize, true, &mut camino);
        Some(camino)
    }

    fn subprueba(&self, a: usize, b: usize, m: usize, borde: bool, out: &mut Vec<zk_ssl_verify::acuses::Digest>) {
        let n = b - a;
        if m == n {
            if !borde {
                out.push(self.mth(a, b));
            }
            return;
        }
        let k = mitad(n);
        if m <= k {
            self.subprueba(a, a + k, m, borde, out);
            out.push(self.mth(a + k, b));
        } else {
            self.subprueba(a + k, b, m - k, false, out);
            out.push(self.mth(a, a + k));
        }
    }
}

/// La mayor potencia de dos ESTRICTAMENTE menor que `n >= 2`: la particion de
/// RFC 6962, la de `zk_ssl_verify::mmr`.
fn mitad(n: usize) -> usize {
    debug_assert!(n >= 2);
    1usize << (usize::BITS - 1 - (n - 1).leading_zeros())
}

#[cfg(test)]
mod tests {
    use super::*;
    use zk_ssl_guardian::Reconciliacion;
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

    /// §667: con el candado de las hojas envenenado, la pareja es un error y el
    /// latido no firma. Antes salia `(as_digest(0), 0)` -el genesis- y el latido
    /// componia una cabeza con la historia vacia. Falsador: con la version de
    /// antes, `pareja_mmr` da `Ok` y el `is_err` cae.
    /// §673: la frontera da la MISMA cima que recomponerla de todas las hojas
    /// (`zk_ssl_verify::mmr::cima`, el oraculo) para cada t de 1 a 300, añadida
    /// hoja a hoja y puesta al dia a saltos. Falsador: con el pliegue al reves
    /// (por la izquierda), las cimas difieren desde t = 3.
    #[test]
    fn la_frontera_da_la_cima_de_todas_las_hojas() {
        let hojas: Vec<_> = (0..300u64).map(zk_ssl_verify::acuses::as_digest).collect();
        let mut f = ArbolMmr::default();
        assert_eq!(f.cima(), None);
        for t in 1..=hojas.len() {
            f.al_dia(&hojas[..t]);
            assert_eq!(f.cima(), zk_ssl_verify::mmr::cima(&hojas[..t]), "t = {t}");
        }
        let mut g = ArbolMmr::default();
        for t in [1usize, 2, 7, 64, 65, 200, 300] {
            g.al_dia(&hojas[..t]);
            assert_eq!(g.cima(), zk_ssl_verify::mmr::cima(&hojas[..t]), "a saltos, t = {t}");
        }
    }

    /// §676: el arbol da el MISMO camino de consistencia que recomponerlo de
    /// todas las hojas (`zk_ssl_verify::mmr::prueba_de_consistencia`, el
    /// oraculo) para cada t de 1 a 40 y a saltos hasta 129, y cada `viejo` de 0
    /// a t + 1, y el camino verifica entre las dos cimas. Falsadores, ensayados:
    /// con el bloque perfecto leido del vecino de la izquierda, o con el
    /// hermano de la derecha empujado en lugar del de la izquierda, los caminos
    /// difieren desde t = 3. Quitar la condicion de alineado NO cambia nada:
    /// la particion solo pide bloques alineados; la condicion esta para que el
    /// arbol no dependa de eso.
    #[test]
    fn el_arbol_da_el_camino_de_consistencia_de_todas_las_hojas() {
        let hojas: Vec<_> = (0..129u64).map(|i| zk_ssl_verify::acuses::as_digest(0x9000 + i)).collect();
        let mut f = ArbolMmr::default();
        for t in (1..=40).chain([63, 64, 65, 100, 129]) {
            f.al_dia(&hojas[..t]);
            let nueva = zk_ssl_verify::mmr::cima(&hojas[..t]).expect("cima");
            for viejo in 0..=(t as u64 + 1) {
                let camino = f.prueba_de_consistencia(viejo);
                assert_eq!(camino, zk_ssl_verify::mmr::prueba_de_consistencia(&hojas[..t], viejo), "t = {t}, viejo = {viejo}");
                if let Some(c) = camino {
                    let vieja = zk_ssl_verify::mmr::cima(&hojas[..viejo as usize]).expect("cima vieja");
                    assert!(zk_ssl_verify::mmr::verificar_consistencia(vieja, viejo, nueva, t as u64, &c), "t = {t}, viejo = {viejo}");
                }
            }
        }
    }

    #[test]
    fn con_las_hojas_envenenadas_no_hay_pareja_ni_latido() {
        let app = std::sync::Arc::new(crate::tests::nodo(30));
        app.hojas_mmr.lock().expect("hojas").push(zk_ssl_verify::acuses::as_digest(7));
        assert!(pareja_mmr(&app).is_ok(), "sano, hay pareja");
        let a = app.clone();
        let _ = std::thread::spawn(move || {
            let _g = a.hojas_mmr.lock().expect("hojas");
            panic!("envenenar el candado de las hojas");
        })
        .join();
        assert!(app.hojas_mmr.is_poisoned(), "premisa: el candado esta envenenado");
        assert!(pareja_mmr(&app).is_err(), "envenenado, no hay pareja que firmar");
        assert!(latir(&app, None).is_err(), "y el latido no compone");
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
    fn la_era_de_los_recibos_sigue_al_indice_de_la_firma() {
        // §569: sin clave el indice no se mueve; con clave, cada latido lo
        // lleva al de su firma.
        let app = crate::tests::nodo(30);
        latir(&app, None).expect("latir sin clave");
        assert_eq!(app.indice_firma.load(std::sync::atomic::Ordering::Acquire), 0);
        let mut f =
            FirmanteCabeza::desde_semilla(&semilla(), en_disco("indices_era")).expect("abrir");
        for esperado in 1..=2u64 {
            latir(&app, Some(&mut f)).expect("latir");
            assert_eq!(app.indice_firma.load(std::sync::atomic::Ordering::Acquire), esperado);
        }
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
    fn la_cabeza_viaja_entera_y_su_digest_es_el_del_latido() {
        // §275: campos+digest JUNTOS en el latido — un solo artefacto de
        // custodia. Y el n que viaja es el techo declarado, firmable.
        let app = crate::tests::nodo(30);
        let l = latir(&app, None).expect("latir");
        let recompuesto = zk_ssl_wire::digest_to_wire(&l.cabeza.digest()).0;
        assert_eq!(recompuesto, l.epoch_digest, "la cabeza y su digest divergen");
        assert_eq!(l.cabeza.n, crate::vista_acuses::N_MAX_CABEZAS);
        assert_eq!(l.cabeza.seq, l.seq);
    }

    #[test]
    fn la_foto_del_latido_es_la_de_su_cabeza_y_sirve_el_cobro() {
        // §493 (D-F): la foto se toma bajo el candado que compone la cabeza,
        // asi que sus raices son las de ESA cabeza; y de ella sale lo que el
        // cobrador necesita, que sube a esas mismas raices.
        let app = crate::tests::nodo(30);
        let pv = crate::tests::pendiente_v2(&app, 0xF493, 0xB493, 0x5493);
        let l = latir(&app, None).expect("latir");
        assert_eq!(l.foto.raiz_pendientes(), l.cabeza.pending_root);
        assert_eq!(l.foto.raiz_meta(), l.cabeza.pmeta_root);
        let f = l.foto.cobro(pv.id_bob, &pv.aviso).expect("la foto sirve el cobro de Bob");
        let hoja = stark_experiment::merkle::native_merge(
            zk_ssl::pending::pending_commitment(pv.id_bob, pv.aviso.salt, pv.aviso.amount),
            pv.aviso.x.expect("v2"),
        );
        assert_eq!(
            stark_experiment::merkle::native_root(hoja, &f.camino_pendiente),
            l.cabeza.pending_root,
            "el camino servido sube a la cabeza servida"
        );
        crate::tests::cuenta(&app, 901, 1_000);
        let l2 = latir(&app, None).expect("latir 2");
        assert_ne!(l2.cabeza.accounts_root, l.cabeza.accounts_root);
        assert_eq!(l2.foto, l.foto, "sin pago en medio, la foto es la misma");
    }

    #[test]
    fn la_cadencia_por_defecto_es_la_decidida() {
        // §121: una vez por minuto, decidido tras medir el almacenamiento.
        assert_eq!(LATIDO_POR_DEFECTO_S, 60);
    }
}
