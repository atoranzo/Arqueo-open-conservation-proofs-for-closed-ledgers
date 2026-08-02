<!-- ============================================================
CABECERA DE ESTADO — añadida al committear (AUDITORIA §120, decisión A)
El cuerpo bajo la línea es el texto de sesión VERBATIM (2026-07-31),
reconstruido del registro de sesiones el 2026-08-02. Las citas del código
(`metrics.rs`) y de `AUDITORIA.md` §89 predatan este commit y se refieren
a este texto. Autoridad: PROPUESTA DE SESIÓN, no revisada — escalón
`doc/`, no raíz. Patrón del preprint retirado (README → §31–32).

QUÉ ES RESPECTO A LA MISIÓN (VISION §5 aplicado, 2026-08-02):
· C2, C3, C6 y el hallazgo §2.2 sirven a la MISIÓN DECLARADA a cualquier
  escala: C3 es la prioridad 1 de VISION (operador que no lee saldos) y
  la pieza B11 de CONFIANZA_RESIDUAL. Entran al flujo normal, pieza a
  pieza, por el criterio de VISION §5.
· El dimensionamiento 5×10⁹ (C7, shards, §1, §5–§7) es ESTUDIO
  CONDICIONAL de una misión no declarada: el propio §6 registra que para
  la declarada (200 instituciones) «3,3 tx/s: el código actual la
  ejecuta hoy, sin cambios». Adoptarlo sería ampliación de alcance —
  P4 — y pasaría por VISION §5 como decisión de misión, no de backlog.
· Condiciones de VISION §5 aún abiertas: (4) cada fila de §8 exige test
  discriminante nuevo antes de declararse cerrada; (6) el hallazgo §2.2
  no está en README ni en AUDITORIA — debe registrarse aunque el
  escalado no se adopte.

MAPA DE CORRECCIONES (cifra desfasada → dónde está la corrección):
· §5 «250/s/núcleo» y C4 «4 ms/prueba»  → §89.1: 2,35 ms MEDIDOS
  (5 ejecuciones, dispersión 4 %) = 425/s. El doc era conservador ×1,7.
· §6 «64 shards» y la sensibilidad «×4 → 249»  → §89.1: 37 shards;
  la sensibilidad debe rehacerse sobre 13.600 TPS/shard.
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
  incluyen ese tamaño.
· §3.1 «620 ms × 2 tx/día = 1,24 s/día»  → §130: la generación medida
  con protocolo es 353,2 ms (send) y 237 (claim) — el día del cliente
  recalcula a ~0,59 s. El 620 pasa a histórico.
· C4 «apply por lotes» ≠ «agrupar PRUEBAS por lotes» (RECHAZADO en
  VISION §5.1 por exigir claves de gasto): aquí cada prueba se verifica
  individualmente; el lote es solo de árbol y disco. Homonimia, no
  violación del catálogo de rechazos.
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

**2.1 Techo del `apply` secuencial.** `apply` cuesta el 28,5 % de generar
(§22): ~177 ms por transferencia. Techo de un nodo: **5,7 TPS**. Para el pico
mundial harían falta ~88.000 instancias. Eso no es un camino.

**2.2 ⚠️ Contención del anclaje de raíz — el límite número uno.** La prueba
se ata a la raíz previa exacta (es el anti-replay actual). La raíz cambia
cada ~177 ms y generar cuesta 620 ms: bajo concurrencia, casi ninguna prueba
llega viva. El throughput efectivo colapsa a **~1,6 TPS** con regeneraciones
en cascada. Este límite muerde antes que cualquier otro y no aparece en la
lista de límites cuantificados del README.

**2.3 El nullifier.** Colisiones de posición probables a ~65.000 pagos (ya
registrado). Bloqueante para cualquier escala; ninguno de los cambios de este
documento tiene sentido sin resolverlo primero.

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
