<!-- ============================================================
CABECERA DE ESTADO — añadida al committear (AUDITORIA §120, decisión A)
El cuerpo bajo la línea es el texto de sesión VERBATIM (2026-07-31),
reconstruido del registro de sesiones el 2026-08-02. Las citas del código
(`metrics.rs`) y de `AUDITORIA.md` §89 predatan este commit y se refieren
a este texto. Autoridad: PROPUESTA DE SESIÓN, no revisada — escalón
`doc/`, no raíz. Patrón del preprint retirado (README → §31–32).

QUÉ ES RESPECTO A LA MISIÓN (VISION §5 aplicado, 2026-08-02):
  ⚠️ §860: VISION es historia desde el §448 (`doc/historia/VISION.md`); la
  misión vigente se lee en el README y en `doc/USE_CASES.md`, y P4 queda
  como apoyo histórico.
· C2, C3, C6 y el hallazgo §2.2 sirven a la MISIÓN DECLARADA a cualquier
  escala: C3 es la prioridad 1 de VISION (operador que no lee saldos) y
  la pieza B11 de CONFIANZA_RESIDUAL. Entran al flujo normal, pieza a
  pieza, por el criterio de VISION §5.
· El dimensionamiento 5×10⁹ (C7, shards, §1, §5–§7) es ESTUDIO
  CONDICIONAL de una misión no declarada: el propio §6 registra que para
  la declarada (200 instituciones) «3,3 tx/s: el código actual la
  ejecuta hoy, sin cambios». Adoptarlo sería ampliación de alcance —
  P4 — y pasaría por VISION §5 como decisión de misión, no de backlog.
· ⚠️ §860: DECIDIDO por delegación del autor («resuelvelas aplicando los
  principios»), con el criterio del §609 y el precedente del §642;
  REVERSIBLE; entra con su aceptación: el dimensionamiento NO SE ADOPTA
  (la 47 del BACKLOG, cerrada). Se reabre si un caso de USE_CASES declara
  un pico, en operaciones de capa por segundo (un pago son dos), por
  encima del techo del `apply` del nodo único, remedido entonces con el
  protocolo del §89.1 (la última medida con protocolo, la del §217,
  265-320 op/s, es anterior al §538); y al reabrirse entra la restricción
  del orden total de la 23 (§2.3). Fuera del rechazo: la fila de la capa
  de liquidación del §6 y su párrafo de la federación. Lo que sirve a la
  misión se reparte: C3/B3, con B2, B11.1 y B11.3, a la 126; B9, a la 21.
  Y el invariante del §3.2 —ninguna operación toca dos cuentas— queda
  escrito: un RFC que lo rompa lo dice.
· Condiciones de VISION §5 aún abiertas: (4) cada fila de §8 exige test
  discriminante nuevo antes de declararse cerrada; (6) el hallazgo §2.2
  no está en README ni en AUDITORIA — debe registrarse aunque el
  escalado no se adopte.
  ⚠️ §860: la (6), cumplida: la registra `AUDITORIA.md` §123, la da la (iv)
  de `doc/USE_CASES.md` §«Institutional clearing: four tests» y
  `doc/INSTITUCIONAL.md` §«Las limitaciones de escala, cuantificadas»,
  n.º 1, que el README enlaza.

MAPA DE CORRECCIONES (cifra desfasada → dónde está la corrección):
· §5 «250/s/núcleo» y C4 «4 ms/prueba»  → §89.1: 2,35 ms MEDIDOS
  (5 ejecuciones, dispersión 4 %) = 425/s. El doc era conservador ×1,7.
· §6 «64 shards» y la sensibilidad «×4 → 249»  → §89.1: 37 shards;
  la sensibilidad debe rehacerse sobre 13.600 TPS/shard.
  ⚠️ §860: no se rehace: el estudio no se adopta, y su base medida —los 2,35
  ms del §89.1, de los que salen los 13.600 TPS/shard— es anterior al
  §538; el §637 midió 3,98-4,93 ms por prueba, con carga y sin el
  protocolo del §89.1. No se escribe otra cifra.
· C4  → §89.2: verificar es el 3,2 % del `apply`; árbol+disco ~70 ms son
  el 96,8 %. El lote ataca el 96,8 % — argumento MEJOR que el original.
· §11  → §89.4: el coste de verificación pasa de «incertidumbre» a
  «resuelto», y se escribe (la etiqueta era falsa aunque el valor real
  fuera favorable). §89.5: los 64 núcleos y el margen del 50 % SIGUEN
  siendo supuestos; hash del árbol y latencia de clientes siguen siendo
  estimación y modelo (B9).
· §2.3 / C1 «bloqueante para cualquier escala»  → VISION §4-p7: el techo
  del nullifier solo afecta a `transfer()`; `send`/`claim` no usan
  nullifiers; retirar la vía antigua lo cierra sin tocar circuitos — y
  vuelve con la prioridad 5 (consenso).
· C8 «la política sigue sin resolver» y las citas a §87  → §88
  reestructuró las cuatro piezas; la política está RESUELTA en §119
  (reversión como segundo cobro; Δ por pago sobre suelo de PRINCIPIOS);
  el reloj es de CABEZAS FIRMADAS (§115, §119, §121), no de `seq` crudo.
· C4 «épocas de 1 s; `seq` numera épocas»  → compone con la capa de
  firma (entrada 53, §115): época = lote; cabeza FIRMADA = latido 1/min
  + a demanda, techo adversarial 1/s ya tarifado (16 % de núcleo, 40/8,
  ~18,5 KB/cabeza). Los costes de log de §6 predatan la firma y no
  incluyen ese tamaño. ⚠️ CORREGIDO (§636): el «a demanda» se decidió
  y NO se construyó; sólo firma el latido.
· §3.1 «620 ms × 2 tx/día = 1,24 s/día»  → §130: la generación medida
  con protocolo es 353,2 ms (send) y 237 (claim) — el día del cliente
  recalcula a ~0,59 s. El 620 pasa a histórico.
· C4 «apply por lotes» ≠ «agrupar PRUEBAS por lotes» (RECHAZADO en
  VISION §5.1 por exigir claves de gasto): aquí cada prueba se verifica
  individualmente; el lote es solo de árbol y disco. Homonimia, no
  violación del catálogo de rechazos.
· «Añade B1–B9 al backlog» (Relación) y el orden de §10  → §779: no se
  añadió ninguno. Tienen entrada B8 (la 12) y la réplica de B7 (la 17);
  el resto se evitó o se hizo por otra vía, se descartó o se queda en la
  47. La correspondencia, pieza a pieza, en la nota bajo la tabla de §10.
  ⚠️ §860: la 17, la 23 y la 47 se cerraron en el §860: la réplica no se
  construye, el consenso no se emprende, y lo que quedaba en la 47 va a la
  126 (B3, con B2, B11.1 y B11.3) y a la 21 (B9); el resto no se adopta.
  Dónde queda cada una, en la misma tabla.
· C7 «que ya es un WAL verificable» y la fila B7 de §10 «del WAL
  existente» → ⚠️ §860: no hay WAL (la 19 del BACKLOG); una entrada del
  registro lleva raíces, compromiso y resumen de prueba, no hojas
  (`crates/zk-ssl/src/log.rs`, `LogEntry`), así que enviar el registro no
  reproduce el estado. La réplica es la 17, cerrada en el §860: la alta
  disponibilidad del operador no se construye ni se prescribe; y la
  réplica entre partes independientes es la 23, cerrada sin emprenderse.
============================================================ -->

# ESCALADO — de un nodo a la escala mundial con los cambios mínimos

**Estado**: propuesta evaluada, no implementada. Pendiente de decisión.
**Fecha**: 2026-07-31.
**Origen**: análisis de diseño sobre los números medidos del repositorio
(README, `AUDITORIA.md` §22, §29–§32) y la evaluación de la reversión de
pendientes registrada en §87.
**Relación**: modifica dos filas de la tabla «Qué garantiza el sistema» del
README (ver §8 de este documento). Subsume una pieza de §87. Añade B1–B9 al
backlog.

---

## 0. Qué es y qué no es

**Qué es**: un camino para servir a 5 × 10⁹ usuarios reutilizando el diseño
existente casi intacto. Todos los cambios son de la clase de la entrada 15
—formato y estado, no criptografía nueva—, salvo un circuito que ya estaba
evaluado en §87. El dimensionamiento usa los números medidos del propio
repositorio y distingue en cada punto lo medido de lo estimado y de lo
modelado.

**Qué no es**: descentralización. Cada shard conserva el modelo de confianza
del nodo único —su operador ve sus saldos y ordena sus operaciones—. El
consenso distribuido sigue fuera de alcance, como declara el README.

---

## 1. Objetivo cuantificado

| Magnitud | Valor |
| --- | --- |
| Usuarios | 5 × 10⁹ |
| Operaciones | 2 tx/día/usuario → 1,0 × 10¹⁰ tx/día |
| TPS promedio | 115.741 |
| TPS pico (× 4,3, factor típico de redes de pago) | ~498.000 |

---

## 2. Por qué el diseño actual no llega (sobre lo medido)

**2.1 Techo del `apply` secuencial — ⚠️ CIFRA CORREGIDA (§204).** Esta
entrada decía: «`apply` cuesta el 28,5 % de generar (§22): ~177 ms por
transferencia. Techo de un nodo: **5,7 TPS**». **Ese número era
DERIVADO**, no medido: un porcentaje tomado de §22, de la vía de un paso
que se retiró en §32/§36.

**Medido directamente en §204** (banco `etapa_a_apply`, misma carga sobre
memoria, tmpfs y disco):

| | |
|---|---|
| `apply` por operación | **37,99 ms** en disco · 36,73 en memoria |
| Techo de un nodo | **26,3 operaciones/s** ≈ **13,2 pagos/s** |

Es **4,6× mejor de lo que decía esta entrada**. Y el desglose importa más
que la cifra: persistencia **3 %**, verificación STARK **7 %**, y **93 %
en `digest_of_proof`**, que hashea la prueba entera (65.840 bytes) con
Rescue —2.915× más caro que Blake3 sobre los mismos bytes— para un
resumen que **no entra en ningún circuito**. Corrigiéndolo, el `apply`
bajaría a ~2,3 ms: **436 operaciones/s**.

⚠️ Pero el techo del `apply` **no es el límite efectivo**: ver 2.2.

**2.2 ⚠️ Contención del anclaje de raíz — el límite número uno, ahora
MEDIDO (§204).** La prueba se ata a la raíz previa exacta (es el
anti-replay actual), y generar cuesta ~250 ms el envío y ~237 el cobro.
Bajo concurrencia, casi ninguna prueba llega viva.

Esta entrada estimaba «~1,6 TPS con regeneraciones en cascada». El banco
`etapa_a5_concurrencia` lo puso a prueba con hilos que operan **cada uno
su propio par de cuentas** —así lo único disputable es la raíz—:

| hilos | pagos/s | `StaleState` (envío) | `StaleState` (cobro) | regeneraciones/pago |
|---|---|---|---|---|
| 1 | 1,85 | 0 | 0 | 0,00 |
| 4 | **1,57** | 9 | 37 | **3,83** |

**Confirmado, y peor de lo estimado**: 70 generaciones para 24
operaciones — **el 66 % del trabajo criptográfico se tira** — y el
rendimiento **BAJA** al añadir hilos (0,84×). Eso es un livelock, no un
cuello de botella ordinario.

⚠️ **Dos correcciones a esta entrada:**

1. «La raíz cambia cada ~177 ms» venía del número derivado de 2.1. Lo
   medido son ~33 ms por `apply`, así que bajo carga la raíz **cambia
   aún más rápido** y la contención es **peor**, no mejor.
2. Con **un solo hilo** salen 1,85 pagos/s **sin una sola regeneración**:
   eso es simplemente `250 + 33 + 237 + 33 = 553 ms` en serie. Es decir,
   el sistema **está en serie Y ADEMÁS se degrada al paralelizarlo**. Las
   dos explicaciones eran ciertas a la vez.

**Hallazgo nuevo del mismo banco**: los cobros sufren **4,1× más** que
los envíos (37 frente a 9). `claim` se ata también a `pending_root_old`,
de modo que **cada envío ajeno le mata la prueba**. El acoplamiento de
los tres árboles deja de ser teoría y pasa a ser medida — y es la parte
más barata de atacar.

**2.3 El nullifier — ⚠️ RETIRADO, y esta entrada estaba equivocada.**
Decía que las colisiones a ~65.000 pagos eran «bloqueante para cualquier
escala» y que **ningún cambio de este documento tenía sentido sin
resolverlo primero**. Eso ya no es cierto por una vía que no era la
prevista: **la vía de un paso se retiró con su árbol de nullifiers**
(`AUDITORIA.md` §32 y §36), así que hoy nada los genera y el orden de
trabajo de este documento **no está bloqueado**.

⚠️ Con la reserva de siempre: el límite **se evitó, no se resolvió**. Lo
que sustituye al marcador es el encadenamiento de raíces, que exige
**orden total** — el que un nodo único da y un sistema distribuido no.
Cualquier diseño de escalado que reparta el estado **recupera el problema
entero**, y por tanto sigue siendo la primera pregunta de la §5.

**El límite número uno vigente es el 2.2** (contención del anclaje), hoy
medido en **1,5-1,9 TPS**.

---

## 3. Los dos hechos estructurales que salvan el diseño

**3.1 La generación es del cliente.** 620 ms × 2 tx/día = **1,24 s/día por
dispositivo**. La capacidad de prueba escala automáticamente con los
usuarios: el equivalente agregado es ~72.000 días-CPU/día, distribuido y
gratis. El precedente existe: Zcash operó con generaciones de ~40 s en 2016.
El problema de escala es exclusivamente el lado servidor.

**3.2 Ninguna operación toca dos cuentas.** El envío toca la hoja del
pagador; el cobro, la del receptor; el pendiente es el mensaje entre ambos.
El README lo presenta como decisión de privacidad («un envío toca una sola
hoja»), pero su consecuencia mayor es otra: **el particionado por cuenta no
tiene transacciones cross-shard**. No hay 2PC ni atomicidad distribuida: el
aviso enrutado es todo el protocolo inter-shard, y es asíncrono por diseño.

El coste declarado de las dos fases (§29–§30: el pendiente inmovilizado) es
exactamente lo que compra shardabilidad ilimitada. Los sistemas account-based
no tienen esta propiedad y pagan por ella con protocolos de atomicidad
cross-shard.

---

## 4. Cambios propuestos

**C1 — Nullifier (bloqueante, independiente).** Espacio de posiciones a
≥ 128 bits efectivos, o indexado secuencial con no-pertenencia por
compromiso. Sin C1 nada de lo que sigue importa.

**C2 — Nonce en la hoja del pagador.** El anti-replay migra del
encadenamiento de raíces al estado de la propia hoja: aplicar dos veces la
misma operación exigiría que la hoja tuviera el mismo estado dos veces, que
es imposible por construcción. Cambio de formato de hoja, clase entrada 15.
Habilita C3–C5.

**C3 — Entradas públicas = transición local de hoja.** La prueba expone
`(posición i, hash_hoja_viejo, hash_hoja_nuevo, R_E)`. El hash de hoja es un
compromiso *hiding*: no filtra saldo ni nonce. Consecuencias:

- El circuito deja de computar raíces globales. Demuestra autorización,
  conservación y transición local de la hoja.
- El servidor compone la raíz del lote, y el log encadenado por época hace
  esa composición re-verificable por un supervisor.
- La vigencia de una prueba anclada a una raíz vieja se comprueba en O(1):
  `árbol_actual[i] == hash_hoja_viejo`. Si la hoja no cambió desde `R_E`, la
  pertenencia probada sigue siendo cierta hoy; si cambió, el rechazo es
  *correcto* (el nonce/saldo ya es otro y el cliente debe regenerar de todos
  modos). **Sin mapas auxiliares, sin árboles históricos, sin estado
  adicional O(cuentas).**

**C4 — Épocas de 1 s con `apply` por lotes.** Verificación en paralelo
(4 ms/prueba, 250/s/núcleo, trivialmente paralela). Una actualización batch
del árbol, una escritura a disco y una entrada del log encadenado por época;
`seq` pasa a numerar épocas. Conflicto intra-época: dos gastos del mismo
pagador contra el mismo estado de hoja —raro en minorista, y el cliente lo
resuelve encadenando sus operaciones localmente (ver §7.3).

**C5 — Ventana de raíces W = 300 épocas.** El servidor mantiene las últimas
300 raíces (9,4 KB) y acepta anclas de hasta 5 minutos. Dimensionado en §7.

**C6 — Doble check de congelación.** El circuito sigue probando
no-pertenencia al árbol de congelados sobre `R_E` (garantía criptográfica,
con retardo ≤ W como suelo), **y** `apply` comprueba el árbol de congelados
de la época actual (retardo efectivo: 1 época). La raíz de congelados
aplicada queda registrada en el log de época para re-verificación del
supervisor.

⚠️ §487: hoy el circuito prueba no-pertenencia de *alguna* posición, no del
titular; el suelo criptográfico de C6 lo pone el arreglo B, y hasta entonces
el check que vale es el de `apply`.

**C7 — Particionado por prefijo de `account_id`.** S shards, cada uno una
instancia casi intacta: su árbol, su log, su verificación de integridad al
arrancar. Los avisos se enrutan por prefijo del `receiver_id`: una cola con
recibos encadenados al log de cada extremo. Réplica pasiva por *log-shipping*
del registro encadenado —que ya es un WAL verificable— y failover
reutilizando la verificación de arranque existente.

**C8 — Reversión de pendientes (§87), ascendida a condición de
operabilidad.** A esta escala deja de ser opcional: 5 × 10⁹ pendientes
nuevos al día no pueden acumularse indefinidamente. Las cuatro piezas siguen
siendo las de §87 —formato del compromiso atando al emisor, altura como
entrada pública, circuito nuevo, decisión de política—, con una
simplificación: con C3/C4, «`seq` como entrada pública» queda subsumido en el
formato de entradas públicas por época. La política sigue sin resolver; las
dos mitigaciones registradas (timeout extensible por el receptor, plazos
largos de abandono con precedente institucional) siguen vigentes.

---

## 5. Dimensionamiento por shard

Sobre lo medido (620 ms, 4 ms, 28,5 %) más dos estimaciones marcadas.

| Etapa | Cálculo | TPS |
| --- | --- | --- |
| Verificación | 64 núcleos × 250/s | 16.000 |
| Árbol | 78 × 10⁶ cuentas → prof. 27; ~54 hashes/tx × 5 µs *(estimado)*; paralelo × 16 subárboles *(estimado)* | ~59.000 |
| **TPS/shard con margen del 50 %** | mín(etapas) × 0,5 | **8.000** |

El coste del hash (5 µs, Rescue/Poseidon en CPU) y el factor de
paralelización de subárboles son estimaciones, no medidas. El margen del
50 % absorbe un error de hasta × 4 sin cambiar la conclusión (§11).

---

## 6. Dimensionamiento global

| Magnitud | Valor |
| --- | --- |
| Shards para el pico mundial | **64** |
| Cuentas por shard | 78,1 × 10⁶ |
| RAM del árbol por shard | ~5 GB |
| Ingesta de pruebas en pico | ~482 MB/s/shard (10–40 GbE); **las pruebas se verifican y se descartan** |
| Log encadenado | 6,3 GB/año/shard (1 entrada/época) + anclas 2 B/tx = 0,31 GB/día/shard |
| Capa de liquidación | 200 instituciones, netting multilateral horario → **3,3 tx/s: el código actual la ejecuta hoy, sin cambios** |
| Sensibilidad (todo × 4 peor) | 249 shards — sigue siendo una sala, no un centro de datos |

La federación entre instituciones (gobernanza, jurisdicciones, prueba de
respaldo con el circuito de banda existente: `emisión_interna ≤
saldo_en_capa_madre`) queda como capa encima del sharding. A 3,3 tx/s, la
capa de liquidación es el único nivel donde el código actual ya basta tal
cual.

---

## 7. La ventana W, dimensionada

### 7.1 Modelo de latencia del cliente *(modelado, no medido — ver B9)*

Latencia total = generación × factor de dispositivo + red + jitter.

- **Dispositivos**: 20 % × 1, 40 % × 3, 30 % × 10, 10 % × 20 sobre los
  620 ms. La generación STARK está limitada por ancho de banda de memoria y
  castiga la gama baja; la mezcla refleja una base de 5 × 10⁹ usuarios
  reales, no una de *early adopters*.
- **Redes**: 70 % buena (0,15 s), 20 % 3G (1 s), 8 % 2G/rural (5 s), 2 %
  pésima (15 s), para ~5 KB de materiales de bajada y 62 KB de prueba de
  subida.
- **Jitter**: lognormal multiplicativo (σ = 0,35).

Percentiles resultantes (2 × 10⁶ muestras):

| p50 | p90 | p95 | p99 | p99,9 | p99,99 |
| --- | --- | --- | --- | --- | --- |
| 3,3 s | 11,9 s | 15,6 s | 24,1 s | 37,5 s | 53,5 s |

### 7.2 Rechazo y regeneraciones en función de W

| W (s) | Llegan tarde | Regeneraciones/día |
| --- | --- | --- |
| 5 | 39,2 % | 6,4 × 10⁹ — colapso |
| 30 | 0,35 % | 35 × 10⁶ |
| **60** | **0,004 %** | **385.000** |
| ≥ 120 | ~0 % | ~0 |

⚠️ **El argumento decisivo es de equidad, no de rendimiento.** El rechazo
con W pequeño no se reparte uniforme: cae casi íntegro sobre el mismo decil
—gama baja en red pobre— que además paga cada regeneración a 6–12 s de
cómputo. Un W pequeño es un impuesto regresivo estructural sobre
exactamente los usuarios que un sistema de 5 × 10⁹ personas debe servir.

### 7.3 El rechazo por «hoja cambiada» no depende de W

`materials` entrega la raíz actual, así que la edad del ancla es ≈ la
latencia del cliente (~5,3 s de media en el modelo), no W. Probabilidad de
que la hoja del pagador cambie antes de que llegue su prueba:

| Perfil | P(hoja cambió) |
| --- | --- |
| Minorista (2 tx/día) | 0,012 % |
| Activo (20 tx/día) | 0,12 % |
| Tesorería (2.000 tx/día) | ~12 % |

Solo el emisor de alta frecuencia lo nota, y se resuelve en cliente con
*pipelining* de nonces: generar la tx *n + 1* sobre el estado posterior a la
tx *n* sin esperar confirmación, cosa que el formato de C2 permite. No
existe todavía (ver §11).

### 7.4 Decisión

**W_hard = 300 épocas (5 min).** Cubre el p99,99 modelado con margen × 5
para relojes desviados y reintentos; cuesta 9,4 KB; el rechazo residual es
ruido de fallos reales, no de diseño. En operación normal el ancla tiene la
edad de la latencia del cliente, no 300 s: **W es un límite, no un
objetivo**. La meseta es ancha y plana —por debajo de ~60 el sistema excluye
usuarios; por encima de ~300 no compra nada—, que es la firma de un
parámetro sin víctimas. El único coste real de W (frescura de la
congelación) lo neutraliza C6, dejándola en 1 época.

---

## 8. Qué cambia en la tabla de garantías del README

⚠️ Esto es un cambio de **qué garantiza qué**, no una optimización. Es donde
un revisor debe mirar primero.

| Vía de ataque | Cerrada hoy por | Cerrada tras C2–C6 por |
| --- | --- | --- |
| Reenviar una operación válida | Encadenamiento de raíces | **Nonce en la hoja del pagador** (C2) |
| Gastar estando congelada | No-pertenencia demostrada en circuito | **Doble check**: circuito sobre `R_E` (suelo criptográfico, retardo ≤ W) + `apply` sobre la época actual (retardo 1 s), registrado en el log (C6) |
| Composición de la raíz global | Implícita en el circuito (raíz vieja → raíz nueva) | **Servidor + log encadenado por época re-verificable** (C3). El circuito garantiza la transición local; el log, la composición. |

⚠️ §487: la fila <<gastar estando congelada>> prueba hoy menos de lo que dice: la
no-pertenencia demostrada en circuito es la de *alguna* posición; la cierra
`apply` (S487) hasta el arreglo B.

El resto de filas de la tabla no cambia: conservación, apertura a cero,
custodios, tope de emisión, doble gasto (sobre C1), autoridad de gasto e
integridad al arrancar siguen demostrados donde estaban.

---

## 9. Lo que esta propuesta no resuelve

- **Confianza por shard = la del nodo único.** El operador de tu shard ve
  tus saldos y ordena tus operaciones. El consenso sigue fuera de alcance.
- **El enrutador de pendientes es un componente nuevo.** Mínimo —una cola
  con recibos encadenados a los logs de origen y destino—, pero nuevo, y su
  diseño de entrega-exactamente-una-vez merece documento propio.
- **Durante la ventana W, la frescura de las políticas depende del
  operador.** Auditada por el log, pero del operador.
- **Los números tienen tres calidades distintas**: medidos (620 ms, 4 ms,
  28,5 %, 65.000 —una sola ejecución, como advierte el README), estimados
  (coste de hash, paralelización de subárboles) y modelados (latencia de
  clientes). Ninguno está medido a escala.
- **Nada de esto está auditado.**

---

## 10. Backlog

| # | Ítem | Clase | Depende de | Nota |
| --- | --- | --- | --- | --- |
| B1 | Fix del espacio de posiciones del nullifier | bloqueante | — | Sin B1, nada de lo demás importa |
| B2 | Nonce en la hoja del pagador | formato | B1 | Migra el anti-replay; clase entrada 15 |
| B3 | Entradas públicas = transición local de hoja | formato + circuito | B2 | Redefine el reparto circuito/servidor (§8) |
| B4 | Épocas de 1 s + `apply` por lotes | estado | B3 | `seq` numera épocas |
| B5 | Ventana W = 300 + registro de anclas en el log | estado | B4 | 2 B/tx en el log de época |
| B6 | Doble check de congelación | estado + circuito | B4 | Retardo de política: 1 época |
| B7 | Particionado por prefijo + enrutador de pendientes | componente | B4 | Réplica por log-shipping del WAL existente |
| B8 | Reversión de pendientes (§87) | formato + circuito + **política** | B3 | Condición de operabilidad a escala; política sin resolver |
| B9 | Benchmark de generación en dispositivos ARM de gama baja | medición | — | Valida el modelo de §7; primero en orden de información |

Orden sugerido: B1 → B9 en paralelo con B2 → B3 → B4 → {B5, B6, B7} → B8.
B9 es barato y es el que más incertidumbre elimina por unidad de esfuerzo.

⚠️ §779: **dónde vive cada B en `BACKLOG.md`.** La línea «Relación» del
encabezado decía que este documento «añade B1–B9 al backlog», y no se añadió
ninguno: la entrada 47 pidió reconciliarlos antes (`AUDITORIA.md` §128.3),
porque anexarlos sin mirar duplicaba entradas. Medidos contra el árbol, solo
uno tiene entrada propia entera (B8, la 12) y otro a medias (B7: su réplica
es la 17); el límite de B1 lo evitó otra vía; B4 y B6 se hicieron en otra
forma y sin pasar por B3; B2 solo tiene objeto dentro de B3; y B3, B5, B9 y
el resto de B7 no tienen más sitio que la 47. **Esta tabla no adopta nada**:
dice dónde está cada pieza, no que vaya a hacerse. ⚠️ §860: la 47 se cerró, y la tabla dice
dónde queda cada pieza ahora.

| # | Entrada del BACKLOG | Lo que hay en el árbol |
| --- | --- | --- |
| B1 | ninguna; su residuo es la **23** ⚠️ §860: la **23**, cerrada: no se emprende | Evitado por otra vía, no resuelto (§2.3 de este documento): la vía de un paso se retiró de la capa con su árbol de nullifiers (`AUDITORIA.md` §32 y §36). Vuelve si el estado se reparte sin orden total, y eso es la 23 |
| B2 | ninguna ⚠️ §860: la **126**, como pieza de B3 | Descartado como palanca propia: el RFC-0002 descartó el anti-replay por `nonce` de cuenta —el obstáculo es el camino contra `root_old`, no el `nonce` (`DIAGNOSTICO_ESCALADO.md` §5)— y «una operación por cuenta y por lote» la impone `apply_many` sin `nonce`. Solo tiene objeto como pieza de B3. ⚠️ §860: la primera pregunta del RFC de la 126 es ese descarte |
| B3 | la **47**, que además recibió el B11 de la 48 al cerrarse esta (§268) ⚠️ §860: la **126**, con B2, B11.1 y B11.3; la 47, cerrada | Sin construir. El RFC-0002 mostró que los lotes no lo necesitan (su 2.2), así que dejó de ser condición del lote; queda el motivo de privacidad —el operador ciego: B11.1 de `CONFIANZA_RESIDUAL.md` depende de él, y B11.3 de B11.1— y la prioridad 1 de VISION (`AUDITORIA.md` §122.2). Y de él sigue colgando B5 |
| B4 | ninguna; lo tomó la etapa 2 del RFC-0002 | Hecho en otra forma: `apply_many` (§215; el §216 midió cero pruebas tiradas), **sin circuitos y sin B3**. No por épocas de 1 s: el registro lleva una entrada por operación con las raíces reales (RFC-0002, 2.4.bis), `seq` sigue contando operaciones (§340) y la cabeza firmada es el latido de 1/min (cabecera) |
| B5 | la **47**, sin entrada propia ⚠️ §860: la 47, cerrada: el dimensionado no se adopta; si el RFC de la **126** necesita anclas más viejas, lo plantea allí | Sin construir. Una prueba se ata hoy a la raíz vigente o a la instantánea del lote; no hay ventana ni registro de anclas. W descansa en la comprobación O(1) de C3, que es la que este documento da para aceptar un ancla vieja «sin árboles históricos»; y su dimensionado (§7) es del estudio condicional |
| B6 | ninguna | Hecho para el nodo único, contra la raíz vigente y sin W: `apply` rechaza la cuenta congelada (§487) y el AIR ata la posición del titular en el árbol de congelados desde el §680 (E1 del RFC-0018; en el árbol, sin versión de cable hasta `zkssl/0.5`: `SECURITY.md` §3.6). La raíz de congelados va en la cabeza firmada. La nota §487 bajo C6 vale para el cable publicado (`zkssl/0.4`), y el arreglo B que nombra no es el del §511, que no ataba (§670), sino el del §680. Falta solo el «retardo ≤ W», que es B5. ⚠️ §860: y B5 no se adopta |
| B7 | la **17**, la réplica; el particionado y el enrutador, la **47** ⚠️ §860: la réplica, la **17**, cerrada; el particionado y el enrutador, la **47**, cerrada sin adoptarse | La réplica por *log-shipping* y el failover son la 17, que va con la 23. El particionado y el enrutador no tienen entrada propia, y no deben tenerla mientras el escalado no se adopte: son estudio condicional (cabecera; `AUDITORIA.md` §122.2). El RFC-0002 deja la partición como opcional, «solo si el pico objetivo lo exige». ⚠️ §860: la 17 y la 23 se cerraron: la alta disponibilidad del operador no se construye ni se prescribe, y la réplica entre partes independientes no se emprende; el «WAL existente» de la fila B7 de la primera tabla de este §10 no existe (cabecera) |
| B8 | la **12** | Política decidida (§119) y mecanismo hecho sin B3 (§178-§181); el Δ por pago lo tomó el RFC-0003, que se aceptó en el §356 |
| B9 | la **47**, adonde lo manda la **77** ⚠️ §860: la **21**; la firma XMSS en ARM, que es del nodo y del testigo, en la **77** | Sin medir. Es una medida y no depende de adoptar nada; la 77 le junta la de XMSS en ARM. Toda comparación de tiempos entre sesiones arrastra la deriva de la **68** |

El orden sugerido de arriba ya no ordena: B1 se evitó por otra vía, y B4, B6
y B8 avanzaron sin pasar por B3. Lo que sigue colgando de B3 es B2, que solo
tiene objeto dentro de él, B5 y, fuera de esta tabla, el B11 de
`CONFIANZA_RESIDUAL.md`. ⚠️ §860: B3, B2, B11.1 y B11.3 van juntos a la 126 —B11.2 sigue en la 11—; B5 no se adopta.

---

## 11. Los puntos donde esta propuesta tiene menos confianza

Si vas a atacarla, empieza aquí.

1. **El coste del hash en el árbol es estimado.** 5 µs por permutación
   Rescue/Poseidon en CPU. El margen del 50 % absorbe hasta × 4; más allá,
   el cuello de botella del shard cambia de etapa y el número de shards
   crece (la sensibilidad de §6 acota el daño: 249).
2. **La distribución de latencia de clientes es un modelo.** El p99,9 real
   de una flota mundial de dispositivos viejos puede ser peor. W = 300 da
   margen × 8 sobre el p99,9 modelado, pero es margen sobre un modelo. B9
   convierte el modelo en medida.
3. **El pipelining de nonces en cliente no existe.** Sin él, los emisores de
   alta frecuencia ven ~12 % de regeneraciones. Es código de cliente, no de
   protocolo, pero alguien tiene que escribirlo.
4. **La re-verificación del supervisor está descrita, no diseñada.** Qué
   firma y qué encadena exactamente cada entrada de época —anclas, raíz de
   congelados aplicada, transiciones locales del lote— merece su propia
   especificación antes de tocar código.
5. **El conflicto intra-época se declara raro sin medirlo.** Con 2 tx/día
   por usuario lo es; con patrones de comercio (muchos cobros, pocos envíos)
   también, porque el cobro toca la hoja del receptor y no compite con los
   envíos. Pero la afirmación es de distribución de carga, y las
   distribuciones se miden.
