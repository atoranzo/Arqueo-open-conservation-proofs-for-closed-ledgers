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
//! La cabeza de época **es útil por sí sola** (⚠️ §849: decía que su `epoch_digest` está en los
//! vectores de conformidad de una versión ya rechazada; `zk-ssl-cli/src/conformance.rs` no lo
//! cruza: D-AI). Lo que la clave añade es la firma. Por eso el código los separa: sin clave
//! **hay cabeza**, no hay firma.
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
//! Cruzarlo bloquearía el ejecutor entero. ⚠️ §866: y sin cruzarlo también: `latir` es síncrona, firma incluida, y corría en un hilo del ejecutor de tokio, que mientras tanto no atendía la escucha (medido en la 123 del BACKLOG); desde el §866 corre en su propio hilo, el de [`arrancar`].

use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::firma_cabeza::{CabezaFirmada, FirmaError, FirmanteCabeza}; // ⚠️ §866: `FirmaError`, para el trait [`Firmar`] del final del fichero
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
/// falta: la cabeza ya está leída. ⚠️ §892: el 0,24 % es de la firma sola; `latir` entera dura unos 804 ms medidos sin carga, y el RPC no contesta mientras dura (BACKLOG 123, §829). ⚠️ §866: medido, no era el candado, que se suelta antes de firmar, sino el hilo: `latir` corría en un hilo del ejecutor y nadie aceptaba conexiones; desde el §866 corre en el suyo, y el RPC contesta mientras firma (`el_rpc_contesta_por_tcp_mientras_el_latido_firma`).
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
/// de los **144,5 ms** que costaría una firma bloqueante. ⚠️ §892: M.1 midió escrituras en serie; las sondas de lectura del RPC sí se paran mientras dura `latir` (BACKLOG 123, §829). ⚠️ §866: y las escrituras tampoco se libraban, aquí: con un chorro de `dev_openSeeded` en serie contra el binario de `213b0b0`, 16 de 2.569 pasaron de 100 ms (292 ms la peor), por el ejecutor y no por el candado (la 123 del BACKLOG).
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
pub fn latir(app: &App, firmante: Option<&mut dyn Firmar>) -> anyhow::Result<Latido> { // ⚠️ §866: era `Option<&mut FirmanteCabeza>`; `dyn Firmar`, para que los tests de la 123 retengan la firma (el trait, al final del fichero)
    // ⚠️⚠️ §815 · EN PARADA NO SE COMPONE NI SE FIRMA (F2). La PARADA (§530, §655) dice que la
    // memoria puede ir por delante del disco -un candado envenenado a medio mutar, o el almacen
    // que fallo despues de que la capa mutara-, y una cabeza compuesta desde esa memoria acredita
    // raices que el disco no tiene. MEDIDO en privado sobre 4d8017d (m-parada, ENOSPC inyectado
    // en el libro): con el nodo en PARADA y el disco en el seq 8, este latido firmo cinco cabezas
    // del seq 9 en diez segundos, y tras reiniciar el nodo firmo OTRA del seq 9. Antes del §815
    // `latir` no miraba la PARADA: el «no sigue sirviendo» del §655 era del despacho, no de esto.
    // Aqui se mira ANTES de todo, y el error lleva la causa; no se gasta indice XMSS.
    if let Some(causa) = app.parada.get() {
        anyhow::bail!("nodo en PARADA: el latido no compone ni firma: {causa}");
    }
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
        // ⚠️ §815 · Y LA MARCA DE FALLO EN VUELO, con el candado YA tomado: `dispatch` la pone
        // antes de soltarlo y `despachar` pone la PARADA despues. Mirar solo la PARADA dejaria el
        // instante de en medio, en que este candado ya esta libre y la memoria ya esta rota.
        // (`marca_en_vuelo_con_el_candado`: en los tests, mirarla sin el candado entra en panico.)
        if let Some(fallo) = crate::marca_en_vuelo_con_el_candado(app) {
            anyhow::bail!("fallo en vuelo: el latido no compone ni firma: {fallo}");
        }
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

/// Lanza el latido en una tarea de fondo. ⚠️ §866: en su propio hilo del sistema, «latido», y ya no en una tarea de tokio (la CORRECCIÓN, abajo).
///
/// ⚠️ `tokio::spawn` y no un hilo: `main` es `async` y el ejecutor ya
/// existe. Pero **el candado es de `std`**, así que [`latir`] es síncrona
/// y se llama entera entre dos `await` — nunca a caballo de uno. ⚠️ CORRECCIÓN (§866), corregida y no borrada (§247): un hilo, y no `tokio::spawn`. Llamada entera entre dos `await`, `latir` ocupaba un hilo del ejecutor mientras firmaba, y en un nodo ocioso nadie más atendía la escucha: el RPC no contestaba (medido en la 123 del BACKLOG). Ahora corre en su hilo del sistema, «latido», con el mismo bucle; lanzarlo puede fallar, y el arranque no sigue.
pub fn arrancar<F: Firmar + 'static>(app: Arc<App>, mut firmante: Option<F>, cada: Duration) -> std::io::Result<()> {
    std::thread::Builder::new().name("latido".into()).spawn(move || {
        let mut n: u64 = 0;
        loop {
            std::thread::sleep(cada);
            n += 1;
            let t = Instant::now();
            match latir(&app, firmante.as_mut().map(|f| f as &mut dyn Firmar)) {
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
    }).map(drop)
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
    // ⚠️ CORRECCIÓN (§714): «exactamente» no, si `diario::anotar` falla
    // arriba: la hoja entra igual y la linea no. Tras un reinicio la siembra
    // trae una hoja de menos, y el nodo vuelve a firmar un tamaño servido
    // (`si_anotar_falla_el_nodo_repite_un_mmr_size`). Declarado, no
    // cerrado: cerrarlo (fallar cerrado si no puede anotar) va al trabajo
    // del nodo de fallar cerrado, y choca con la doctrina de arriba.
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
    fn en_parada_el_latido_no_compone_ni_firma_y_el_indice_no_se_mueve() {
        // ⚠️⚠️ §815 · F2, MEDIDO en privado sobre 4d8017d: con el nodo en PARADA por un fallo del
        // almacen, este latido siguio firmando cabezas desde la memoria (seq 9 con el disco en
        // seq 8). Ahora, con la PARADA puesta, `latir` da error con la causa, la era de los
        // recibos no se mueve y el guardian sigue en el mismo indice: no se gasta indice XMSS.
        // Falsador: quitar la guarda de la PARADA al principio de `latir`: firma el 2 y cae.
        let app = crate::tests::nodo(30);
        let mut f = FirmanteCabeza::desde_semilla(&semilla(), en_disco("parada")).expect("abrir");
        latir(&app, Some(&mut f)).expect("sano, firma el indice 1");
        let _ = app.parada.set("un fallo del almacen en zkssl_applySend: (inyectado por el test)".into());
        let e = latir(&app, Some(&mut f)).expect_err("en PARADA no se compone ni se firma");
        assert!(format!("{e}").contains("PARADA"), "el error lleva la causa: {e}");
        assert_eq!(
            app.indice_firma.load(std::sync::atomic::Ordering::Acquire),
            1,
            "la era de los recibos no se mueve"
        );
        assert_eq!(
            f.reconciliar().expect("reconciliar"),
            Reconciliacion::Coincide { indice: 1 },
            "y el guardian sigue en el 1: ningun indice XMSS gastado en PARADA"
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

    // ── ⚠️ §866 · LA 123: el latido en su hilo, y el RPC que contesta mientras se firma ──
    //
    // Medido sobre `213b0b0`: con `latir` dentro de un `tokio::spawn`, la firma ocupaba un hilo del
    // ejecutor y, en un nodo ocioso, nadie atendia la escucha; el nucleo completaba la conexion y el
    // RPC no contestaba hasta que el latido volvia a su `sleep`. Estos tests retienen la firma a
    // medio latido con un firmante REAL (`FirmanteRetenido`) y miran el nodo mientras tanto. El
    // orden lo dan canales; los tiempos son solo topes para que un fallo caiga en vez de colgarse.

    /// ⚠️ §866 · Lo que un `FirmanteRetenido` cuenta de cada firma que le piden: el hilo desde el que
    /// se la piden y si ese hilo es del ejecutor de tokio.
    struct Llamada {
        hilo: Option<String>,
        en_el_ejecutor: bool,
    }

    /// ⚠️ §866 · Un firmante REAL al que se le RETIENE la firma. Sus primeras `guion` firmas avisan
    /// por `aviso` y esperan a `suelta` antes de firmar de verdad; si el test cae antes de soltarla,
    /// `suelta` se cierra y la firma sale con error, sin firmar. Las que pasan del guion se quedan
    /// paradas sin gastar CPU: un latido en su hilo no sigue firmando cuando el test ya ha acabado.
    struct FirmanteRetenido {
        real: FirmanteCabeza,
        guion: usize,
        llamadas: usize,
        aviso: std::sync::mpsc::Sender<Llamada>,
        suelta: std::sync::mpsc::Receiver<()>,
    }

    impl Firmar for FirmanteRetenido {
        fn firmar(&mut self, epoch_digest: &[u8; 32]) -> Result<CabezaFirmada, FirmaError> {
            self.llamadas += 1;
            if self.llamadas > self.guion {
                loop {
                    std::thread::park();
                }
            }
            let _ = self.aviso.send(Llamada {
                hilo: std::thread::current().name().map(str::to_owned),
                en_el_ejecutor: tokio::runtime::Handle::try_current().is_ok(),
            });
            if self.suelta.recv().is_err() {
                return Err(FirmaError::Xmss("el test cayo sin soltar la firma: no se firma".into()));
            }
            self.real.firmar(epoch_digest)
        }
    }

    /// La otra punta de un `FirmanteRetenido`: sus avisos, su suelta y la clave publica del real.
    struct Retencion {
        avisos: std::sync::mpsc::Receiver<Llamada>,
        suelta: std::sync::mpsc::Sender<()>,
        clave_publica: Vec<u8>,
    }

    fn retenido(nombre: &str, guion: usize) -> (FirmanteRetenido, Retencion) {
        let real = FirmanteCabeza::desde_semilla(&semilla(), en_disco(nombre)).expect("abrir");
        let clave_publica = real.clave_publica();
        let (aviso, avisos) = std::sync::mpsc::channel();
        let (soltar, suelta) = std::sync::mpsc::channel();
        (
            FirmanteRetenido { real, guion, llamadas: 0, aviso, suelta },
            Retencion { avisos, suelta: soltar, clave_publica },
        )
    }

    /// El tope de las esperas de estos tests, que solo existe para que un fallo caiga: una firma
    /// cuesta aqui unos 0,25 s en release y unos 13 s en depuracion (§643); abrir y fondear una
    /// cuenta, 0,6 s en release, y de 10 a 23 s con ocho `yes` y el test a `nice` 10.
    const TOPE: Duration = Duration::from_secs(120);

    /// Un runtime de tokio de dos hilos, el sabor de `#[tokio::main]`, que no se suelta nunca: con un
    /// hilo del ejecutor retenido (los falsadores), soltarlo esperaria a ese hilo para siempre.
    fn runtime() -> &'static tokio::runtime::Runtime {
        Box::leak(Box::new(
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .expect("runtime"),
        ))
    }

    /// Una peticion JSON-RPC por TCP, a mano: HTTP/1.1 con `Connection: close`, leida hasta el
    /// final, con 30 s de tope de lectura.
    fn por_tcp(dir: std::net::SocketAddr, metodo: &str) -> std::io::Result<String> {
        use std::io::{Read, Write};
        let cuerpo = format!(r#"{{"jsonrpc":"2.0","id":1,"method":"{metodo}","params":{{}}}}"#);
        let mut s = std::net::TcpStream::connect_timeout(&dir, Duration::from_secs(5))?;
        s.set_read_timeout(Some(Duration::from_secs(30)))?;
        write!(
            s,
            "POST / HTTP/1.1\r\nHost: {dir}\r\nContent-Type: application/json\r\n\
             Content-Length: {}\r\nConnection: close\r\n\r\n{cuerpo}",
            cuerpo.len()
        )?;
        let mut r = String::new();
        s.read_to_string(&mut r)?;
        Ok(r)
    }

    fn contesta(r: &str) -> bool {
        r.starts_with("HTTP/1.1 200") && r.contains("\"result\"")
    }

    /// ⚠️ §866 · LA 123, EL HILO. El latido corre en su propio hilo del sistema, «latido», y no en un
    /// hilo del ejecutor de tokio, aunque se lance desde dentro de un runtime, como en `main`.
    /// Falsador, corrido: con el `tokio::spawn` de antes en `arrancar`, la firma se pide desde un
    /// «tokio-rt-worker», dentro del ejecutor, y cae.
    #[test]
    fn el_latido_corre_en_su_hilo_y_no_en_un_hilo_del_ejecutor() {
        let rt = runtime();
        let _dentro = rt.enter();
        let app = Arc::new(crate::tests::nodo(30));
        let (f, r) = retenido("en_su_hilo", 1);
        arrancar(app, Some(f), Duration::ZERO).expect("lanzar el latido");
        let ll = r.avisos.recv_timeout(TOPE).expect("el latido llega a la firma");
        assert_eq!(ll.hilo.as_deref(), Some("latido"), "la firma se pide desde el hilo del latido");
        assert!(!ll.en_el_ejecutor, "y ese hilo no es del ejecutor de tokio");
    }

    /// ⚠️ §866 · LA 123, EL SINTOMA. Con la firma del latido retenida, el nodo contesta por TCP: la
    /// escucha de axum, en un runtime de dos hilos, acepta y el despacho responde; y al soltarla la
    /// cabeza sale firmada con el indice 1, se sirve por TCP y verifica. Es lo que, medido sobre
    /// `213b0b0`, no pasaba en un nodo ocioso. El latido duerme 200 ms, y no cero: con cero, el
    /// `tokio::time::sleep` de antes podia vencer dentro de su propio `poll` y la tarea volvia a la
    /// cola como un `yield`, con aviso a otro hilo, que atendia la escucha (tokio 1.53.1,
    /// `runtime/time/mod.rs:436-438` y `runtime/task/harness.rs:156-161`), y el falsador no caia
    /// siempre. Falsador, corrido 20 veces: con el `tokio::spawn` de antes en `arrancar`, la lectura
    /// de la sonda caduca a los 30 s y cae.
    #[test]
    fn el_rpc_contesta_por_tcp_mientras_el_latido_firma() {
        let rt = runtime();
        let _dentro = rt.enter();
        let app = Arc::new(crate::tests::nodo(30));
        let escucha = rt.block_on(tokio::net::TcpListener::bind("127.0.0.1:0")).expect("escucha");
        let dir = escucha.local_addr().expect("direccion");
        let enrutador = crate::enrutador(app.clone());
        let _servidor = rt.spawn(async move { axum::serve(escucha, enrutador).await });
        let antes = por_tcp(dir, "zkssl_supply").expect("premisa: el nodo contesta por TCP");
        assert!(contesta(&antes), "premisa: el nodo contesta antes del latido: {antes}");
        let (f, r) = retenido("rpc_por_tcp", 1);
        arrancar(app.clone(), Some(f), Duration::from_millis(200)).expect("lanzar el latido");
        r.avisos.recv_timeout(TOPE).expect("el latido llega a la firma");
        assert!(
            app.ultima_cabeza.lock().expect("cabeza").is_none(),
            "premisa: la firma esta retenida y no hay cabeza"
        );
        let durante = por_tcp(dir, "zkssl_supply").expect("con la firma retenida, el nodo contesta por TCP");
        assert!(contesta(&durante), "con la firma retenida, el nodo contesta: {durante}");
        r.suelta.send(()).expect("soltar la firma");
        let t = Instant::now();
        while app.ultima_cabeza.lock().expect("cabeza").is_none() {
            assert!(t.elapsed() < TOPE, "soltada la firma, la cabeza no sale");
            std::thread::sleep(Duration::from_millis(10));
        }
        let servida = por_tcp(dir, "zkssl_signedEpochHead").expect("la cabeza firmada, por TCP");
        assert!(contesta(&servida) && servida.contains("\"available\":true"), "{servida}");
        let u = app.ultima_cabeza.lock().expect("cabeza");
        let l = u.as_ref().expect("cabeza");
        let c = l.firma.as_ref().expect("la cabeza va firmada");
        assert_eq!(c.indice, 1, "la primera cabeza lleva el indice 1");
        crate::firma_cabeza::verificar_cabeza(&r.clave_publica, &l.epoch_digest, c)
            .expect("y verifica");
    }

    /// ⚠️ §866 · LA 123, LA GUARDA. Mientras se firma, ningun candado del nodo esta tomado: el del
    /// estado, los de recepcion y registro, los de la cabeza, las hojas y el arbol del MMR, el del
    /// indice del diario y el de las cofirmas. El candado se suelta antes de firmar desde el §241, y
    /// esto lo ata: lo que dejaba al RPC sin contestar era el hilo, no un candado. Falsador, corrido:
    /// tomar el candado del estado alrededor de la firma; cae en el primero.
    #[test]
    fn mientras_se_firma_ningun_candado_del_nodo_esta_tomado() {
        let app = Arc::new(crate::tests::nodo(30));
        let (mut f, r) = retenido("candados", 1);
        let a = app.clone();
        let h = std::thread::spawn(move || {
            latir(&a, Some(&mut f)).map(|l| l.firma.map(|c| c.indice))
        });
        r.avisos.recv_timeout(TOPE).expect("el latido llega a la firma");
        assert!(app.estado.try_lock().is_ok(), "el candado del estado esta libre mientras se firma");
        assert!(app.recepcion.try_lock().is_ok(), "el del contador de recepcion");
        assert!(app.registro.try_lock().is_ok(), "el del registro de recepcion");
        assert!(app.ultima_cabeza.try_lock().is_ok(), "el de la ultima cabeza");
        assert!(app.hojas_mmr.try_lock().is_ok(), "el de las hojas del MMR");
        assert!(app.arbol_mmr.try_lock().is_ok(), "el del arbol del MMR");
        assert!(app.indice_diario.try_lock().is_ok(), "el del indice del diario");
        assert!(app.cofirmas.try_lock().is_ok(), "el de las cofirmas");
        r.suelta.send(()).expect("soltar la firma");
        let indice = h.join().expect("el hilo del latido").expect("latir");
        assert_eq!(indice, Some(1), "y la cabeza sale firmada con el indice 1");
    }

    /// ⚠️ §866 · LA 123, LO QUE CAMBIA. Con la firma retenida, el despacho lee y escribe; la cabeza
    /// que sale es la compuesta antes de la escritura, y la escritura entra en la siguiente, que
    /// lleva el indice 2 con el guardian a la par. Mientras se firma la cabeza 1, la era publicada
    /// sigue en 0, y la que fija un recibo que llega en esa ventana (`era_al_recibir`) es la 1. Que
    /// caiga en la cabeza 2 no lo ata este test, que no emite ningun recibo: se deduce de la D-D
    /// del RFC-0010, que lo contempla («cuando el recibo llego mientras se firmaba», §681), y en un
    /// nodo ocioso no se daba porque el despacho esperaba. Falsadores, corridos: tomar el candado
    /// del estado alrededor de la firma, y el despacho no contesta en el tope y cae; y fijar al
    /// recibir la era de una cabeza mas, y cae en la era.
    #[test]
    fn con_la_firma_retenida_el_despacho_lee_y_escribe_y_la_cabeza_es_la_compuesta_antes() {
        let app = Arc::new(crate::tests::nodo(30));
        let seq_antes = latir(&app, None).expect("componer sin firmar").seq;
        let (f, r) = retenido("despacho", 1);
        let a = app.clone();
        let h = std::thread::spawn(move || {
            let mut f = f;
            let l = latir(&a, Some(&mut f));
            (l, f)
        });
        r.avisos.recv_timeout(TOPE).expect("el latido llega a la firma");
        let (hecho, leido) = std::sync::mpsc::channel();
        let a = app.clone();
        std::thread::spawn(move || {
            let lee = crate::dispatch(&a, "zkssl_supply", json!({})).is_ok();
            crate::tests::cuenta(&a, 902, 1_000);
            let _ = hecho.send(lee);
        });
        let lee = leido
            .recv_timeout(TOPE)
            .expect("con la firma retenida, el despacho lee y escribe");
        assert!(lee, "y la lectura sale bien");
        assert_eq!(
            app.indice_firma.load(std::sync::atomic::Ordering::Acquire),
            0,
            "mientras se firma la cabeza 1, la era publicada sigue en 0"
        );
        assert_eq!(crate::era_al_recibir(&app), 1, "y un recibo de esa ventana fija la era 1");
        r.suelta.send(()).expect("soltar la firma");
        let (l1, mut f) = h.join().expect("el hilo del latido");
        let l1 = l1.expect("latir");
        assert_eq!(l1.firma.as_ref().expect("firmada").indice, 1);
        assert_eq!(l1.seq, seq_antes, "la cabeza es la compuesta antes de la escritura");
        let l2 = latir(&app, Some(&mut f.real)).expect("latir 2");
        assert!(l2.seq > seq_antes, "la escritura entra en la cabeza siguiente");
        assert_eq!(l2.firma.as_ref().expect("firmada").indice, 2);
        assert_eq!(app.indice_firma.load(std::sync::atomic::Ordering::Acquire), 2);
        assert_eq!(
            f.real.reconciliar().expect("reconciliar"),
            Reconciliacion::Coincide { indice: 2 },
            "el guardian y la clave van juntos"
        );
    }
}

/// ⚠️ §866 · **Lo que el latido pide a su firmante: firmar un digest.** [`latir`] lo toma como
/// `dyn Firmar` para que los tests de la 123 puedan RETENER la firma a medio latido y mirar, mientras
/// tanto, en qué hilo corre, qué candados hay tomados y si el RPC contesta. En el binario no abre
/// ningún camino: lo implementa [`FirmanteCabeza`] con su `firmar` de siempre —reserva el índice con
/// su fsync, firma y se autoverifica—, y nada más. `Send` porque el firmante viaja, una vez, al hilo
/// del latido. Va al final del fichero, detrás de los tests, para no mover ninguna línea citada.
pub trait Firmar: Send {
    /// Firma `epoch_digest`, como el `firmar` de [`FirmanteCabeza`].
    fn firmar(&mut self, epoch_digest: &[u8; 32]) -> Result<CabezaFirmada, FirmaError>;
}

impl Firmar for FirmanteCabeza {
    fn firmar(&mut self, epoch_digest: &[u8; 32]) -> Result<CabezaFirmada, FirmaError> {
        FirmanteCabeza::firmar(self, epoch_digest)
    }
}
