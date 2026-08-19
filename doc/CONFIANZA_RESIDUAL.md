<!-- ============================================================
CABECERA DE ESTADO — añadida al committear (AUDITORIA §120, decisión A)
El cuerpo bajo la línea es el texto de sesión VERBATIM (v2, 2026-07-31),
reconstruido del registro de sesiones el 2026-08-02. Las citas del código
(`log.rs` ×3, `lib.rs`) predatan este commit y usan la NUMERACIÓN v1:
«§8.1» y «§8.2» citados en código y en AUDITORIA §121 son §10.1 y §10.2
de esta v2. Autoridad: PROPUESTA DE SESIÓN, no revisada — escalón `doc/`.
Patrón del preprint retirado (README → §31–32).

ENCAJE (VISION §5 aplicado, 2026-08-02): pasa los seis criterios — este
documento ES la misión declarada en forma de plan (VISION §1: «el poder
del intermediario acotado, visible y mínimo»), y B10 corre sobre el nodo
único actual (P4). Única condición, criterio 5: la fila 4 de §7
(transparencia vía `hash_verificador_vigente`) está POR DELANTE del
código — ver mapa.

MAPA — AQUÍ LA REALIDAD ADELANTÓ AL DOCUMENTO (qué está ya construido o
decidido, y dónde):
· B10.1 (cabeza firmada)  → CONSTRUIDO: `EpochHead` en `log.rs` (con
  T1 medido: la cabeza de n ata la historia); firma elegida y medida en
  la entrada 53 / §112–114: XMSS^MT-SHA2_40/8_256, crate `xmss` de
  RustCrypto con guardián de índice obligatorio y declarado; cadencia en
  §115: 1 firma/min + a demanda (no cada época), techo adversarial 1/s
  ya tarifado (16 % de núcleo).
· §2.3 (costes)  → predatan la firma: la cabeza firmada real son
  ~18,5 KB (no ~200 B); a 1/min ≈ 9,7 GB/año/shard — la cifra de §115.
  El «recibo de inclusión ~1 KB» debe re-tarifarse con la cabeza real.
· §2.1 `hash_verificador_vigente`  → el `EpochHead` construido lo
  EXCLUYE deliberadamente, con la razón en el código: no existe noción
  de «reglas vigentes» y un campo vacío sería peor que su ausencia.
  Backlog 54. La fila 4 de §7 NO está comprada aún.
· §2.1 `hash_verificador_vigente` — CORRECCIÓN (§321, 2026-08-19):
  la razón que da la línea de arriba —no existe noción de «reglas
  vigentes»— está corregida desde el §246, y es casi circular: el campo
  ES ese mecanismo. La razón real: **el AIR es CÓDIGO, no datos**; lo
  único hasheable en ejecución son las `ProofOptions`, y un operador
  puede cambiar el AIR dejándolas idénticas, así que el campo sería
  CIEGO, no vacío. Depende de la entrada 55 (el AIR como datos) o de
  compilación reproducible; hashear el fuente al compilar no vale, porque
  no prueba que el binario se construyera de ese fuente. La línea de
  arriba se conserva y se cita en vez de borrarla (§247), y el CUERPO de
  este documento no se toca: sigue siendo texto de sesión verbatim.
· B10.2  → primitiva construida y testada (`first_divergence`, T1);
  falta el componente de testigos.
· B10.3 y §10.2 (el plazo N)  → DECIDIDOS en §121: el «acuse» (nombre
  reservado en §120: ya hay dos `*Receipt` de otra especie) compromete
  su propio N bajo N_max = 1.440 cabezas firmadas (24 h, precedente MMD
  de CT); reloj de CABEZAS FIRMADAS, no épocas crudas; contador de
  recepción monótono como detector de reordenación inmune a congestión.
  Dos correcciones al borrador de §2.1: NADA de firma-por-acuse (la
  aritmética de la 53 lo impone — acuses como hojas bajo una raíz de
  recepción en la cabeza) y `hash_de_la_prueba` con digest con longitud
  codificada (§116: `digest_of_proof` colisiona con ceros finales).
· B8/§87 (reversión, familia de B18.3)  → §87 reestructurado en §88;
  política RESUELTA en §119: segundo cobro del `refund_id` del emisor,
  Δ por pago sobre suelo de PRINCIPIOS (13 meses propuestos), sin
  retroactividad estructural (T3a). B18.3 y la ventana de reclamos son
  la misma familia: §119 es el precedente aplicable.
· «La errata del salt» (precondición dura de B18.2)  → ORIGEN RESUELTO
  en §117: salt derivado de la clave (T2a). Y MEJORA B18: el reclamo de
  migración exige solo la clave — exactamente la propiedad T2b.
  Despliegue en hoja: entrada 50 (B13/B14).
· Coherencia §5.3 ↔ §119: la reversión de pendientes NO es el rollback
  que §5.3 prohíbe — es un claim con clave de titular; nunca se muta
  estado sin prueba.
· B12.1  → progresado: BACKLOG 55 (formato de especificación del AIR,
  probado); B12.2 es la entrada 7.
· `ESCALADO.md` (C3, C7, §6, §9, B9)  → existe en `doc/` con su propia
  cabecera: C3 sirve a la misión declarada; el dimensionamiento
  planetario es estudio condicional. La dependencia B11←C3 queda intacta.
============================================================ -->

# CONFIANZA_RESIDUAL — hacer comprobable la honestidad del operador

**Estado**: propuesta evaluada, no implementada. Pendiente de decisión.
**Fecha**: 2026-07-31 (v2: añade §5 modelo de contramedidas y §6/B18 recuperación).
**Origen**: análisis de coherencia entre la tesis del proyecto (*From
Institutional Trust to Verifiable Properties — and Its Residual Trust
Surface*) y su implementación. Complementa `ESCALADO.md`: aquella propuesta
resuelve el rendimiento; esta reduce la superficie de confianza residual que
aquella conserva a propósito (`ESCALADO.md` §9).
**Relación**: modifica los dos ⚠️ del operador en el README y cinco filas de
la tabla «Qué garantiza el sistema». Añade B10–B12 y B18 al backlog. B10 **no
depende** del escalado: es implementable sobre el nodo único actual.

---

## 0. Qué es y qué no es

**Qué es**: tres mejoras que atacan la única incoherencia estructural entre
lo que el proyecto afirma y lo que implementa —la custodia de la evidencia—,
más el cierre del ⚠️ mayor del README («ve todos los saldos») y la
prioridad que los propios hallazgos del proyecto imponen sobre ambas; y, en
esta v2, el modelo que ordena todas las contramedidas del sistema (§5) y el
protocolo que convierte la muerte de un shard en un trámite con evidencia en
vez de una pérdida (§6, B18). Nada de esto es consenso ni criptografía
nueva: B10 es publicación firmada de lo que el log ya contiene; B11
reutiliza maquinaria de rango y cifrado que ya existen en el ecosistema; B12
es especificación y auditoría; B18 es disponibilidad de datos más un
circuito de la clase de `circuit_claim`.

**Qué no es**: descentralización. El operador sigue siendo único, sigue
ordenando y sigue pudiendo censurar. Lo que cambia es que **mentir, censurar
y mirar dejan de ser invisibles** —la primera genera evidencia firmada, la
segunda evidencia portable, la tercera deja de ser posible salvo en
metadatos— y que **morir deja de ser terminal**: los fondos sobreviven a su
operador.

---

## 1. La incoherencia, dicha con precisión

El README afirma: *«no puede reescribir el historial en secreto (registro
encadenado de transiciones)»*. La garantía es condicional y la condición no
está escrita: **un log encadenado solo impide reescrituras detectables por
quien ya observó una cabeza anterior**. Y hoy nadie fuera del operador
observa cabezas: el registro que ata al operador vive en manos del operador.

Consecuencias concretas:

- **Vista dividida (*split-view*)**: el operador puede servir historias
  distintas a partes distintas. Cada una es internamente consistente; la
  contradicción solo es visible comparándolas, y no hay nadie situado para
  comparar.
- El supervisor que «verifica sin acceso al ledger» verifica pruebas contra
  raíces que le entrega… el operador.
- La censura es indistinguible de la no-recepción: un cliente cuya operación
  no aparece no puede demostrar que la envió.

Para un proyecto cuya tesis es sustituir confianza institucional por
propiedades verificables, que la evidencia contra la institución esté en
poder de la institución es el punto que un revisor serio señalará primero.

---

## 2. B10 — Cabezas atestiguadas y recibos

El patrón es el de Certificate Transparency (RFC 6962), que resolvió
exactamente este problema para las autoridades de certificación: operadores
únicos hechos responsables sin sustituirlos.

### 2.1 Mecanismo

Cada época, el operador **firma y publica una cabeza mínima** a *k* testigos
independientes:

```
cabeza = firma_operador(seq, raíz_estado, raíz_pendientes,
                        raíz_congelados, chain_digest,
                        hash_verificador_vigente)
```

- **Testigos**: entidades sin papel en la operación —otras instancias de la
  federación (que se atestiguan mutuamente), el supervisor, espejos
  públicos—. Solo almacenan y comparan cabezas; no ordenan, no validan, no
  votan. **Esto no es consenso**: es el punto intermedio entre «nodo único»
  y «consenso distribuido» que el README declaró fuera de alcance sin
  explorar lo que hay en medio.
- **Prueba de fraude portable**: dos cabezas firmadas con el mismo `seq` y
  distinto `chain_digest` son evidencia criptográfica autocontenida de vista
  dividida, presentable ante cualquiera —incluido un juez— sin acceso al
  ledger.
- **Recibo de inclusión**: el cliente ya recibe el camino Merkle de su
  operación; conservarlo junto a la cabeza firmada de su época (~1 KB por
  operación) le permite demostrar su saldo contra la última cabeza
  atestiguada **aunque el operador desaparezca**. Es derecho de salida:
  soberanía del ciudadano, no solo del emisor.
- **Recibo de recepción firmado** (B10.3): el operador firma
  `(hash_de_la_prueba, época_de_recepción)` al aceptar una operación en
  cola. Recibo firmado + no inclusión en N épocas = **evidencia portable de
  censura**. Hoy la censura no está cerrada (README); con esto sigue siendo
  posible, pero deja de ser gratuita.

### 2.2 Lo que compra, principio a principio

- **Honestidad**: deja de ser una promesa y pasa a ser exigible — mentir
  genera evidencia firmada por el propio mentiroso.
- **Transparencia**: al incluir `hash_verificador_vigente` en la cabeza,
  **cambiar las reglas se vuelve un acto público**. Hoy, quien puede
  actualizar el verificador es la raíz de confianza real del sistema y nadie
  lo ve. Con la raíz de congelados en la cabeza, la política de congelación
  también queda bajo observación externa.
- **Imagen fiel**: un ledger cuya fidelidad atestiguan partes distintas de
  su custodio.
- **Sinergia con la federación** (`ESCALADO.md` §6): la prueba de respaldo
  (`emisión_interna ≤ saldo_en_capa_madre`) puede exigirse referida a una
  cabeza atestiguada — atando solvencia e historia en el mismo acto.

### 2.3 Costes

| Concepto | Valor |
| --- | --- |
| Cabeza | ~200 B/época → 17 MB/día/shard |
| Testigo que cubre los 64 shards | ~11 KB/s; ~340 GB/año en disco |
| Recibo retenido por el cliente | ~1 KB/tx → ~730 KB/año a 2 tx/día |
| Clase de cambio | Entrada 15: el log ya tiene `seq` y `chain_digest`; publicar es aditivo |

**B10 funciona sobre el nodo único actual, hoy, sin esperar al escalado.**

---

## 3. B11 — El operador ciego

El ⚠️ mayor del README —*«Ve todos los saldos»*— es hoy un coste declarado.
Tras C3 (`ESCALADO.md`) deja de ser necesario: las entradas públicas son
compromisos *hiding* y el operador puede componer el árbol sin ver nada.
Quedan exactamente dos fugas:

**3.1 El importe llega en claro a `apply_send`** — visiblemente, para el
límite por operación. La comprobación puede moverse al circuito con la
maquinaria que **ya existe**: la descomposición de Horner en segmentos de 64
(`SEGMENT_LENGTH`, `NUM_SEGMENTS`), la misma que impone
`max_supply − supply_new`. Coste: un segmento de rango más en el circuito de
envío — la misma clase de coste estimada en §87 para el circuito de
reversión.

**3.2 El aviso viaja legible.** Cifrado del aviso a la clave del receptor
(AEAD + clave efímera, ~100 B de sobrecoste). Es exactamente lo que Sapling
resolvió con *note encryption*; el precedente de ingeniería móvil es el
mismo que B9 ya iba a estudiar.

### 3.3 Coherencia

El modelo de cumplimiento del proyecto **nunca necesitó que el operador
viera**: la revelación selectiva siempre fue del titular al supervisor, y no
cambia. La emisión sigue siendo pública por diseño (suministro atado en
circuito). B11 no resta ninguna capacidad de supervisión; convierte
«privacidad frente a terceros» en «privacidad frente a todos salvo a quien
tú reveles». Es el único ⚠️ del README que puede pasar de coste asumido a
problema cerrado.

### 3.4 Lo que B11 no cierra

**Metadatos.** El operador sigue viendo que la posición *i* tuvo actividad
en la época *E*, desde qué conexión, y con qué frecuencia. El análisis de
tráfico sobre el grafo de actividad queda abierto; cerrarlo exigiría
conjuntos de anonimato en las actualizaciones y es otra disciplina. El ⚠️
del README no se borra: se sustituye por «ve posiciones y momentos de
actividad, no saldos ni importes».

---

## 4. B12 — Especificación formal del AIR y auditoría externa

La prioridad la imponen los propios hallazgos del proyecto. El hallazgo
n.º 1 —**AIR carece de restricciones de copia, y eso abre un agujero
silencioso**— y el precedente de Zcash 2019 —un fallo de solidez que
permitía crear dinero sin rastro sobrevivió años en código auditado y
desplegado— dicen la misma cosa: **un bug de solidez es dinero falso
invisible**, y un ledger que miente sobre cuánto dinero existe es la
violación suprema de la imagen fiel. Ni B10 ni B11 valen nada sobre
circuitos no especificados: las cabezas atestiguarían fielmente un estado
computado con reglas rotas.

Contenido: especificación formal de cada AIR (qué restricciones existen, qué
grados declaran, qué *no* restringen), tests de solidez negativos
sistemáticos (trazas inválidas que deben fallar), y auditoría externa con la
especificación como contrato — que es además la única forma de que una
auditoría de circuitos sea algo más que una lectura.

---

## 5. Modelo de contramedidas: fail-stop + evidencia + diversidad

Una capa de liquidación no se protege manteniendo el flujo: **se protege
negándose a fluir sin prueba**. Este modelo ordena todas las contramedidas
del sistema —implementadas y propuestas— bajo un principio: las dos capas
del sistema tienen físicas opuestas, y confundirlas es el error clásico de
los diseños «resilientes».

### 5.1 Transporte: agresividad licenciada por la idempotencia

Avisos, pendientes en tránsito y réplicas se reenrutan, reintentan y
duplican **agresivamente**: multipath, colas redundantes, failover por
*log-shipping* (C7). Es seguro serlo porque la capa de estado es idempotente
por construcción: un aviso duplicado no puede cobrarse dos veces (unicidad
del pendiente) y una prueba re-entregada no puede aplicarse dos veces (nonce
en hoja, C2). **La idempotencia del estado es lo que licencia la
agresividad del transporte.** La «reconfiguración instantánea de rutas»
vive aquí — y solo aquí.

### 5.2 Estado: la jerarquía de contramedidas

| Nivel | Contramedida | Mecanismo | Estado |
| --- | --- | --- | --- |
| 1 | Prevención por construcción | Invariantes en circuito: los estados inválidos no son demostrables, luego no ocurren (conservación, tope, custodios, autoridad de gasto) | Implementado |
| 2 | Detención demostrada | Verificación de integridad al arrancar: el nodo se niega a operar sobre estado corrupto | Implementado |
| 2b | Detención demostrada | Divergencia entre verificadores N-versión detiene la época | B17 |
| 3 | Evidencia portable | Vista dividida, recibo de recepción, recibo de salida | B10 |
| 4 | Diversidad | Dos códigos independientes del mismo AIR: el fallo exige el mismo bug dos veces | B17 |
| 5 | Recuperación | Los fondos sobreviven a la muerte del shard | **B18** |

### 5.3 Las prohibiciones (lo que una contramedida automática nunca hace)

- **Nunca muta estado sin prueba.** No hay «auto-reparación» del ledger:
  reparar es un acto institucional ejecutado *con* evidencia a través de
  B18, no un reflejo del software.
- **Nunca revierte automáticamente.** Un *rollback* es, por definición, una
  vista dividida — y sería detectado por los propios testigos. La maquinaria
  antifraude hace imposible el rollback silencioso **incluso para el
  software del propio operador**: el sistema no puede hacer a escondidas ni
  lo que haría «por tu bien».
- **La reconfiguración automática vive en transporte y réplica, jamás en
  semántica.** Un sistema de liquidación que «mantiene el flujo» a través de
  un ataque convierte un fallo en una catástrofe contable.

### 5.4 Finalidad por niveles (la consecuencia operativa)

| Nivel | Condición | Latencia | Sobrevive a |
| --- | --- | --- | --- |
| N1 — incluida | En la época del shard | 1–2 s | Operación normal |
| N2 — atestiguada | Cabeza de su época firmada por ≥ *q* de *k* testigos | + segundos | **La muerte del shard** (es el punto de recuperación de B18) |
| N3 — liquidada | Netting en capa madre | Ventana de netting | Disputas entre instituciones |

La política del cliente elige nivel por importe. **La cola no atestiguada es
la ventana máxima de pérdida en un siniestro**; N2 la acota para lo que
importa.

### 5.5 Testigos caídos: decisión de diseño, con su porqué

Si los testigos son inalcanzables, el sistema **no se detiene** —detenerse
les daría el poder de hacer DoS al sistema entero—. Sigue firmando cabezas
localmente encadenadas, y la antigüedad de la última cabeza *atestiguada* es
pública: los clientes de N2 dejan de considerar firme, lo que presiona
económicamente a restaurar la atestiguación. Operar sin atestiguar **no
habilita mentir sin evidencia** (las cabezas siguen firmadas y encadenadas:
dos versiones acabarán siendo prueba de fraude); habilita **perder la cola
en un siniestro**. Por eso la frescura de atestiguación es señal de riesgo
de pérdida, no de fraude — y debe ser visible en cliente (B10.5).

---

## 6. B18 — Protocolo de recuperación desde cabezas atestiguadas

Propósito: que la muerte permanente de un shard —hardware más réplicas, u
operador desaparecido o terminalmente malicioso— no mate los fondos. El
dinero de nadie muere con un servidor.

### 6.1 El hallazgo previo: las cabezas solas detectan, no recuperan

Con solo la última cabeza atestiguada H\*, un reclamante con recibo viejo
(cuyo saldo ya gastó) es indistinguible de uno honesto: la sobre-reclamación
se detecta en agregado (conservación) **pero no se atribuye**. La
recuperación sólida exige **disponibilidad de datos (DA)** en los testigos:
deltas por época (posición + hash de cada hoja tocada) y *snapshots*
periódicos del árbol. Los hashes de hoja son *hiding*: la DA no ve saldos ni
importes —compatible con B11— aunque sí metadatos de actividad (la misma
fuga de §3.4; se declara, no se esconde).

### 6.2 Deberes de testigo ampliados

| Concepto | Coste |
| --- | --- |
| Delta por época | ~40 B/hoja tocada → 72 KB/s medio/shard → 6,3 GB/día → 2,3 TB/año/shard |
| Peor caso adversarial (shard saturado a 8.000 TPS) | 320 KB/s → 27,6 GB/día → 10,1 TB/año/shard |
| Snapshot mensual del árbol | ~5 GB/shard |
| Retención rodante (snapshots + 3 meses de deltas) | ~580 GB/shard |
| Testigo pleno (64 shards, rodante) | ~37 TB — un servidor de almacenamiento |
| Reparto | El deber de DA es fragmentable entre testigos con replicación r ≥ 3 |

### 6.3 El protocolo

1. **Declaración de siniestro.** Acto de gobernanza en la capa madre,
   irreversible, registrado y atestiguado, que fija H\* = la cabeza de mayor
   `seq` firmada por ≥ *q* testigos. Todo lo posterior a H\* es nulo por
   declaración — lo que previene el doble cobro si el shard «resucita».
   Quien declara **no puede mover fondos**: solo los titulares tienen claves
   para reclamar. Una declaración falsa es DoS, no robo: **el protocolo es a
   prueba de robo, no a prueba de DoS de gobernanza** (y se dice así).
2. **Reconstrucción.** El sucesor reconstruye el árbol en H\* desde el
   último snapshot más los deltas (~580 GB → ≈30 min de proceso). El RTO lo
   dominan la gobernanza y la ventana de reclamos, no el cómputo.
3. **Reclamo individual.** Circuito nuevo, clase `circuit_claim`: el titular
   demuestra en cero conocimiento conocer `(sk, salt, nonce, saldo)` tales
   que su hoja en la posición *i* del árbol de H\* abre a esos valores y
   `derive_public_id(sk) = id`, y recibe un compromiso nuevo en el shard
   sucesor. ⚠️ **Requiere el salt de hoja: la errata es precondición dura de
   B18.** (Versión v1 sin ZK —apertura directa al sucesor— solo es
   aceptable pre-B11; con B11, la versión ZK es obligatoria por coherencia.)
4. **Pendientes.** El aviso es autosuficiente como reclamo: el receptor
   cobra contra el árbol de pendientes reconstruido en H\* con el circuito
   de cobro; los no cobrados heredan la reversión de §87/B8. El diseño de
   dos fases vuelve a pagar: **los pendientes son reclamos
   autodescriptivos.**
5. **Conservación de la migración.** El sucesor demuestra, con la maquinaria
   de auditoría existente, que Σ acreditado ≤ suministro en H\*: la propia
   recuperación queda bajo prueba de no-creación de dinero. La maquinaria de
   banda audita su propia migración.
6. **Ventana y remanente.** El plazo de reclamos es **política, no
   parámetro** —decisión con víctimas, la misma familia que §87: plazos
   largos, precedente institucional de cuentas inactivas—. El remanente
   queda custodiado en la capa madre bajo esa política.

### 6.4 Métricas

**RPO** = antigüedad de la última cabeza atestiguada: segundos en operación
sana, acotado para importes altos por la finalidad N2 (§5.4). **RTO** =
declaración (gobernanza) + ≈30 min de reconstrucción + ventana de reclamos.

### 6.5 Recursión

¿Y si muere la capa madre? Es una instancia más: sus cabezas las atestiguan
las hijas (atestiguación mutua de la federación) y su recuperación sigue
este mismo protocolo un nivel arriba. El punto único de fallo remanente es
**el acuerdo institucional** — que es donde vive en cualquier sistema de
liquidación, y donde debe vivir.

---

## 7. Qué cambia en el README

| Vía de ataque | Cerrada hoy por | Cerrada tras B10–B18 por |
| --- | --- | --- |
| Reescribir el historial | Registro encadenado **custodiado por el operador** | Registro encadenado + **cabezas atestiguadas externamente**: la vista dividida produce prueba de fraude portable (B10) |
| Censurar | ⚠️ No cerrada | No cerrada, pero **con evidencia**: recibo de recepción firmado + no inclusión demostrable (B10.3) |
| Ver los saldos | ⚠️ Coste declarado del nodo único | **Cerrada salvo metadatos**: compromisos *hiding* + límite en circuito + aviso cifrado (B11, requiere C3) |
| Cambiar las reglas en silencio | — (no listada) | `hash_verificador` en la cabeza atestiguada: **toda actualización del verificador es un acto público** (B10) |
| Muerte del shard | — (no listada: los fondos mueren con el servidor) | **Recuperación desde H\***: reclamo individual en ZK + conservación demostrada de la migración (B18) |

⚠️ Las filas 2, 4 y 5 añaden garantías que hoy ni siquiera figuran en la
tabla. La honestidad del documento exige decir también la inversa: la fila 2
no cierra la censura, la hace cara; la fila 3 deja los metadatos abiertos
(§3.4); y la fila 5 no recupera la cola no atestiguada (§6.4).

---

## 8. Lo que esta propuesta no resuelve

- **El operador sigue ordenando.** Puede retrasar y reordenar dentro de los
  márgenes que los recibos y sus plazos permitan demostrar. Consenso sigue
  fuera de alcance.
- **La atestiguación exige testigos que comparen.** Certificate Transparency
  enseñó las dos caras: el patrón funciona, y su pieza de *gossip* estuvo
  años infradesplegada. Un testigo que archiva sin comparar no detecta la
  vista dividida (ver §10.1).
- **La cola no atestiguada se pierde en un siniestro.** El RPO de B18 es la
  frescura de atestiguación; las operaciones posteriores a H\* solo son
  recuperables si el operador o sus réplicas cooperan. N2 (§5.4) acota la
  exposición por importe; no la elimina.
- **B18 degrada sin federación.** En el nodo único actual, sin capa madre ni
  gobernanza, B18 se reduce a «recuperación ante un tribunal con evidencia»
  —que ya es más que hoy, pero no es el protocolo.
- **B11 depende de C3.** Sin el rediseño de entradas públicas del escalado,
  el operador necesita ver estado para componer. B11.2 (aviso cifrado) es la
  excepción: independiente e implementable hoy.
- **Los metadatos quedan abiertos** (§3.4), y la DA de B18 los extiende a
  los testigos: ven qué posiciones cambian y cuándo, no saldos.
- **Nada de esto sustituye a B12**; lo presupone.

---

## 9. Backlog

| # | Ítem | Clase | Depende de | Nota |
| --- | --- | --- | --- | --- |
| B12.1 | Especificación formal del AIR + tests de solidez negativos | especificación | — | **Primero.** Contrato de la auditoría |
| B12.2 | Auditoría externa de circuitos y protocolo | auditoría | B12.1 | Ya listada como falta en el README; aquí, con contrato |
| B10.1 | Cabeza firmada por época, publicada a *k* testigos | formato + componente | — | Sobre el nodo actual, hoy |
| B10.2 | Comparación entre testigos y prueba de fraude portable | componente | B10.1 | La pieza que CT infradesplegó |
| B10.3 | Recibo de recepción firmado; no-inclusión demostrable | formato | B10.1 | La censura deja de ser gratuita |
| B10.4 | Recibo de salida del cliente (camino + cabeza retenidos) | cliente | B10.1 | Derecho de salida |
| B10.5 | Frescura de atestiguación visible + finalidad por niveles en cliente | cliente | B10.1 | N2 como señal de riesgo por importe (§5.4–5.5) |
| B11.1 | Límite por operación en circuito (segmento de rango) | circuito | C3 | Maquinaria de Horner ya existente |
| B11.2 | Cifrado del aviso al receptor | formato | — | Independiente; patrón Sapling |
| B11.3 | Retirar el importe en claro de la API de `apply` | formato | B11.1 | Cierra la fuga 3.1 |
| B18.1 | DA en testigos: deltas por época + snapshots + retención | componente | B10.1 | Sin DA, las cabezas detectan pero no recuperan (§6.1) |
| B18.2 | Circuito de reclamo de migración (apertura ZK en H\*) | circuito | errata salt, B12.1 | Clase `circuit_claim` |
| B18.3 | Declaración de siniestro, ventana de reclamos y remanente | **política** + gobernanza | B10.1 | Decisión con víctimas; familia §87 |
| B18.4 | Recuperación de pendientes desde el árbol reconstruido | circuito | B18.1 | Hereda B8/§87; el aviso es el reclamo |
| B18.5 | Prueba de conservación de la migración | circuito existente | B18.2 | La maquinaria de banda audita su propia migración |
| B10.6 | Ancla externa de la raíz de cabezas (interfaz + parámetro *M*) | formato + componente | B10.1, §121.2 | Importa disponibilidad ajena; no elige medio (`doc/ANCLAJE_EXTERNO.md`) |
| B10.7 | Verificador de ancla en cliente/testigo (camino Merkle → raíz anclada) | cliente | B10.6 | Convierte la comparación B10.2 en lectura sin gossip |

Orden sugerido: B12.1 → {B10.1, B11.2} en paralelo → B10.3/B10.4/B10.5 →
B18.1 (los testigos ya reciben cabezas; añadir deltas es incremental) →
B12.2 → (con C3 del escalado y la errata del salt resuelta) B11.1 → B11.3 →
B18.2/B18.4/B18.5 → B10.2 en cuanto existan dos testigos. B18.3 es política:
puede y debe discutirse en paralelo con la de B8, porque son la misma
familia de decisión.

---

## 10. Los puntos donde esta propuesta tiene menos confianza

Si vas a atacarla, empieza aquí.

1. **La independencia de los testigos es un supuesto social, no
   criptográfico.** *k* testigos coludidos con el operador devuelven el
   sistema al punto de partida. La federación mitiga (instancias con
   intereses enfrentados se vigilan mejor que voluntarios), pero «quién
   atestigua a los que atestiguan» no tiene respuesta técnica, y este
   documento no finge tenerla.
2. **El plazo del recibo de recepción es otra decisión con víctimas.** «No
   inclusión en N épocas = evidencia de censura» exige elegir N, y un N
   corto convierte congestión legítima en falsa evidencia. Misma clase de
   decisión que el timeout de §87; mismo tratamiento: es política, no
   parámetro.
3. **B11 declara cerrado «ver saldos» pero no lo mide.** La fuga por
   metadatos (§3.4) podría permitir reconstrucción estadística de patrones
   con esfuerzo suficiente. La afirmación honesta es «no ve saldos ni
   importes», no «no aprende nada»; cuánto aprende de los metadatos es una
   pregunta empírica sin responder.
4. **El coste del segmento de rango en el circuito de envío está estimado
   por analogía** con §87, no medido. La transferencia ya es la prueba más
   cara (~620 ms); si el segmento la encarece más de lo estimado, el
   presupuesto de latencia de cliente de `ESCALADO.md` §7 absorbe el golpe,
   pero conviene medirlo junto a B9.
5. **La cabeza atestiguada fija el formato más público del sistema.** Todo
   lo que se le añada después (más raíces, más metadatos) es una migración
   observada por terceros. Elegir mal el contenido mínimo ahora es el error
   más caro de corregir de toda esta propuesta.
6. **El parámetro *q* de *k* tiene contenido y no tiene óptimo evidente.**
   Un *q* alto hace H\* más fiable pero empeora el RPO cuando caen testigos;
   un *q* bajo abarata la colusión. Está atado al supuesto social del punto
   1, y elegirlo es otra decisión, no un número.
7. **El coste de DA está calculado en media y con un peor caso simple.** Un
   adversario con cuentas y capacidad de prueba puede sostener el shard a
   saturación e inflar los deltas a ~10 TB/año/shard; no hay tarifas en el
   diseño que lo encarezcan. La cota real es la capacidad del shard, y si
   eso basta como freno es una pregunta de economía del sistema, no de
   criptografía.
8. **B18 asume una gobernanza capaz de declarar.** La declaración de
   siniestro es el único paso no criptográfico del protocolo, y es
   exactamente el paso que un operador con captura institucional podría
   retrasar. El protocolo convierte el robo en imposible y la parálisis en
   visible; no convierte la parálisis en imposible.
9. **El reclamo de migración vincula.** Reclamar revela la posición *i* y el
   momento — enlazable con el historial de actividad de esa posición en la
   DA. La recuperación es el momento de menor privacidad del ciclo de vida
   de una cuenta, y debe decirse en la documentación de cara al titular.
