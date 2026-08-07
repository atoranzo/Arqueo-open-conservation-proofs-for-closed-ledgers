# Diagnóstico de escalado: lo que NO es el cuello de botella

> **Documento defensivo.** `doc/ESCALADO.md` dice cuál es el límite número
> uno. Este dice **cuáles NO lo son, y por qué**, con la medida al lado.
> Existe porque ese hueco tiene un coste real: sin él, cualquier
> diagnóstico importado de otro ecosistema —competente, bien escrito, y
> equivocado para este árbol— parece razonable.
>
> Todas las comprobaciones de aquí son **reproducibles con un `grep`**.
> Si alguna deja de serlo, este documento está mal y hay que corregirlo.

---

## 0.bis LO MEDIDO (§204) — léelo antes que el resto

Este documento se escribió **antes** de medir. Después se midieron cinco
cosas, y **tres de sus hipótesis cayeron**. Se conservan escritas más
abajo con su refutación al lado —es la regla de la casa— pero el estado
vigente es este:

| banco | pregunta | resultado |
|---|---|---|
| `etapa_a_apply` | ¿dónde se van los ms del `apply`? | persistencia **3 %** · `apply` = **37,99 ms** → **26,3 op/s** (no 5,7 TPS) |
| `etapa_a2_verify` | ¿verificación o árboles? | verificación **7 %** · resto **93 %** |
| `etapa_a3_escala` | ¿crece con las cuentas? | `apply` **e = 0,18** (plano) · `send_materials` **e = 1,08** |
| `etapa_a4_hash` | ¿qué es ese 93 %? | **`digest_of_proof`: 30,99 ms, 2.915× Blake3** |
| `etapa_a5_concurrencia` | ¿la contención existe? | **3,83 regeneraciones/pago**, 66 % del trabajo tirado, 0,84× al paralelizar |

**Las tres hipótesis refutadas:** el `flush` por operación (§6.4.ter
nivel 0), el `nonce` como anti-replay (§5), y el árbol cuadrático.

**Las dos confirmadas, y lo que arregla cada una:**

| arreglo | arregla | NO arregla |
|---|---|---|
| `digest_of_proof` Rescue → hash no algebraico | techo del nodo: **30 → 436 op/s** | la contención (sigue el 66 % de desperdicio) |
| desacoplar las tres raíces | que los cobros mueran por envíos ajenos (**4,1×** peor que los envíos) | los envíos entre sí |
| lotes | la contención entera | el techo, si el hash sigue |
| árbol incremental | el desplome de `send_materials` a escala | nada de lo anterior |

**Ninguna sirve sola.** Los bancos están en `crates/zk-ssl/examples/` y
las cinco tablas completas, en `AUDITORIA.md` §204.

⚠️ Lo que este documento afirmaba y **ya no vale**: que el group commit
fuera «el múltiplo barato» (la persistencia es el 3 %), y que hubiera que
elegir entre «contención real» y «falta de paralelismo» (son **las dos**
a la vez).

---

## 0. El caso que lo motivó

En agosto de 2026 se contrastó contra este repositorio un diagnóstico de
escalado externo. Recomendaba, en resumen: delegar la generación de
pruebas a un clúster de GPUs por la saturación de **MSM y NTT**;
particionar el **registro de nullifiers**; aislar el crate
`settlement-prover` como microservicio; y agregar mediante **Nova o
Halo2**.

Es consejo estándar y correcto **para un rollup sobre curvas
elípticas**. Contra este árbol, cuatro de las cinco recomendaciones
apuntan a cosas que aquí no existen, y una destruiría la propiedad
central del sistema.

Se registra el episodio, y no el sonrojo: **la lección es que un
documento que solo dice dónde está el límite invita a inventar los
demás**.

---

## 1. El cuello de botella real, medido

| | |
|---|---|
| **Contención del anclaje de raíz** | **1,5-1,9 TPS** — `AUDITORIA.md` §123, entrada 65 del backlog |
| Techo del `apply` secuencial | ~5,7 TPS (`doc/ESCALADO.md` §2.1) |
| Acumulación de pruebas | ~124 MiB por mil transferencias |

Cada prueba se ata a la **raíz exacta** que vio al generarse: es el
anti-replay vigente. La raíz cambia cada ~177 ms y generar cuesta
cientos de ms, así que bajo concurrencia casi ninguna prueba llega viva
y el rendimiento efectivo colapsa con regeneraciones en cascada.

**Ese es el límite que muerde primero, y no es de cómputo: es de
estructura del anti-replay.**

### 1.1 Tres limitaciones acopladas al mismo cuello, no escritas hasta ahora

**1.1.a — La contención no es de UNA raíz: son TRES, y están acopladas.**
`apply_send` exige que coincidan `root_old`, `pending_root_old` **y**
`frozen_root` (`two_phase.rs`). Consecuencia: **una congelación de
gobernanza —que no tiene ninguna relación con los pagos en vuelo— los
invalida todos.** La contención no la generan solo los pagos.

**1.1.b — Un pago son DOS operaciones, así que el objetivo de TPS se
duplica.** `Send` y `Claim` son entradas de registro separadas, cada una
con su prueba y su transición de raíz (`spec/vectors/`: `… Send →
Claim`). Cualquier objetivo expresado en «pagos por segundo» debe
**doblarse** para expresarlo en operaciones de capa. Ver §6.2.

**1.1.c — La finalidad depende del receptor.** El dinero queda en el
árbol de pendientes hasta que alguien **cobra**. Para un sistema de
liquidación esto es un desajuste semántico serio: en un RTGS la firmeza
es inmediata al acto del ordenante. Aquí depende de que el receptor
actúe **y** de que le llegue el aviso fuera de banda (§3.2). La
caducidad y el reembolso (#27/#28) evitan que el dinero se pierda, pero
**no convierten el pago en firme**: lo devuelven.

---

## 2. Lo que NO es el cuello de botella

### 2.1 No hay MSM ni curvas elípticas en la vía de producción

El backend es **STARK/FRI sobre Goldilocks** (`winterfell`). El coste de
probar es **hash + FFT**, no multiplicación multi-escalar.

```bash
grep -rn "msm\|multi_scalar\|G1Projective\|pairing" crates/stark-experiment/src/
# → 0 coincidencias
```

Las curvas viven **solo** en los crates de experimento del estudio
comparativo (`zk-core` con BLS12-381, `halo2-experiment` con Pallas,
`plonk-experiment`, `nova-experiment`), aislados a propósito y **fuera
de la vía de pago**. Optimizar MSM aquí es afinar un motor que no está
montado.

### 2.2 El nodo no genera pruebas — las genera el cliente

```bash
grep -rn "pub fn prove_send\|pub fn prove_claim" crates/zk-ssl/src/client.rs
```

`prove_send` y `prove_claim` corren **en local, en el titular**. El nodo
entrega materiales —caminos y raíces, datos públicos— y **verifica**.

⚠️ Por eso «delegar el proving a un clúster externo» **no es una
optimización, es una regresión de seguridad**: el testigo contiene
material derivado de la clave de gasto, así que un clúster que recibe
testigos recibe las claves. Es exactamente la propiedad que el sistema
sostiene (`Wallet::spend_key` ni siquiera implementa `Serialize`).

Y además **no hace falta**: `doc/ESCALADO.md` §3.1 ya mide que la
generación es del cliente y por tanto **escala sola con los usuarios**
—~1,24 s/día por dispositivo a dos pagos diarios—. La capacidad de
prueba agregada crece con la adopción; es la única parte del sistema que
no necesita plan de escalado.

### 2.3 El registro de nullifiers no existe

La vía de un paso se **retiró con su árbol de nullifiers**
(`AUDITORIA.md` §32 y §36). La vía de producción es la de dos fases, con
árbol de **pendientes** y anti-replay por encadenamiento de raíces.

```bash
grep -c "nullifier" crates/zk-ssl-wire/src/lib.rs   # → 0
grep -c "pending"   crates/zk-ssl-wire/src/lib.rs   # → 23
```

Particionarlo, cachearlo o *shardearlo* es trabajo sobre una estructura
retirada. *(Ese mismo error sobrevivió dentro del propio repositorio
hasta §203: cuatro documentos seguían presentando su límite como vigente.
No es un error exótico — es fácil de cometer.)*

### 2.4 `settlement-prover` no es un servicio de prueba

Existe, y es un **trait de 91 líneas deliberadamente sin dependencias**:
la interfaz que permite comparar backends de prueba de cumplimiento sin
que Arkworks y Halo2 colisionen. **No lo usa `zk-ssl`, ni el nodo, ni el
SDK.** No hay un prover que aislar en GPUs; hay una abstracción del
estudio comparativo.

### 2.5 El ledger no crece con las pruebas

`LogEntry` guarda **el digest de la prueba, no la prueba** (`log.rs`).
Una entrada son ~137 bytes.

```bash
grep -n "no la prueba entera" crates/zk-ssl/src/log.rs
```

Cualquier plan de escalado que empiece por «hay que comprimir el
histórico de pruebas del nodo» está resolviendo un problema que el nodo
no tiene. Lo que sí falta es **política de retención para terceros**
(§6.3).

---

## 3. Los dos esquives, declarados como lo que son

Ninguno de estos problemas está **resuelto**. Están **evitados**, y lo
que los evita es tener **un nodo único**. Quien distribuya el estado los
recupera enteros.

**3.1 Orden total.** El anti-replay por encadenamiento de raíces exige
que alguien ponga las operaciones en un orden. Un nodo único lo da
gratis; un sistema distribuido, no. Sustituyó al marcador de nullifier
—y con él a su colisión a ~65.000 pagos— pero **cambió un límite por un
requisito**.

**3.2 Sincronización del cliente.** En sistemas con notas privadas el
receptor no puede saber que le han pagado sin escanear todo el tráfico
—el problema de usabilidad clásico de las monedas privadas—. Aquí no
ocurre porque el `PendingNotice` **viaja fuera de banda** del pagador al
receptor (`spec/RPC.md`, deuda §21). Coste de sincronización: cero.

⚠️ **Pero el coste no desapareció: se movió de la criptografía al
producto**, y sus consecuencias no están medidas:

- el canal fuera de banda es una **dependencia no auditada**, y si filtra
  quién habla con quién, filtra el grafo de pagos que las pruebas
  protegen;
- **ISO 20022 no lo transporta** (`spec/RPC.md`), así que el puente
  institucional tiene un hueco por donde debe pasar el dato que hace
  cobrable el pago;
- **no existen los pagos no solicitados**: no se puede pagar a quien no
  se tiene canal. En un contexto regulado es casi una virtud; como
  dinero, es una limitación real.

Si el aviso se pierde, el dinero **no queda atrapado**: la caducidad y el
reembolso lo devuelven (circuitos #27 y #28, `AUDITORIA.md` §178-§180).

---

## 4. Lo que está descartado por principio, no por dificultad

**Consenso propio.** `SECURITY.md` §6 sostiene que el consenso no elimina
al intermediario: lo hace plural y caro. Implementarlo y llamar a eso
«descentralizado» contradiría al propio repositorio. Y un BFT escrito por
una sola persona y sin auditar cambiaría **una debilidad honesta y
declarada por una fortaleza falsa**: sus fallos —liveness bajo partición,
equivocación, cambio de vista— no se prueban con testigos discriminantes,
que es el único método de verificación que esta casa tiene.

**Agregación con Nova o Halo2.** Se midieron y se descartaron por
reintroducir ceremonia y perder resistencia cuántica. Es la única
decisión del proyecto tomada **contra** los números de rendimiento;
recuperarlos por la puerta de atrás la anularía. Si algún día hace falta
agregar, el camino coherente es **recursión FRI**, sin curvas.

**Proving delegado.** Ver §2.2. No es cuestión de coste.

**El escalón que sí es coherente**, y ya está diseñado sin construir:
cabezas atestiguadas y **acuse** (§121, `doc/CONFIANZA_RESIDUAL.md`),
anclaje externo de raíces (`doc/ANCLAJE_EXTERNO.md`) y replicación
verificable. No reparten el orden entre pares: hacen que **el operador no
pueda mentir sin dejar evidencia**. Se puede probar con un test
discriminante —el operador omite, el acuse lo detecta—, cabe en una
persona, y no exige ninguna afirmación nueva.

---

## 5. La línea del `nonce`: HIPÓTESIS REFUTADA, y lo que sí sobrevive

> Una versión anterior de este documento proponía colgar el anti-replay
> del `nonce` de cuenta como «la única línea con ventaja». **Se midió y
> no funciona.** Se deja escrita la refutación, no se borra la idea: es
> lo que ahorra el intento a quien venga después.

### 5.1 Lo que dice el código

Dos hechos, comprobables:

```bash
sed -n '554,568p' crates/zk-ssl/src/two_phase.rs   # apply_send
grep -n "El nonce NO cambia" crates/stark-experiment/src/circuit_send.rs
```

1. `apply_send` ata **tres raíces globales a la vez** —`root_old`,
   `pending_root_old` y `frozen_root`—: cualquier cambio en cualquiera
   de los tres devuelve `StaleState` y mata **todas** las pruebas en
   vuelo.
2. **El `nonce` ni siquiera avanza con el pago**: `circuit_send` lo dice
   en su propio comentario —*«el nonce NO cambia: destruir no consume el
   derecho»*—. Hoy no es un contador de operaciones.

### 5.2 Por qué la idea falla, y no por el `nonce`

El obstáculo está **por debajo** del anti-replay: la prueba lleva un
**camino de Merkle contra `root_old`** y además **afirma `root_new`**.
Aunque el anti-replay colgara del `nonce`, el camino sigue siendo contra
una raíz que cualquier otra cuenta puede cambiar.

Y quitar la atadura de raíz obliga a reintroducir **un marcador de doble
gasto** — es decir, el árbol de nullifiers que se retiró en §32/§36, con
su colisión a ~65.000 pagos. **La idea se muerde la cola.**

### 5.3 La reformulación que SÍ sobrevive, con su precio

Que el cliente pruebe **solo la transición de su hoja** —más pertenencia
contra `root_old`— y **NO afirme `root_new`**; que el nodo aplique un
**lote** de transiciones de hoja y recalcule la raíz una vez. Entonces N
pruebas contra la misma raíz **componen**, y la contención pasa de ser
por operación a ser por lote.

Y aquí el `nonce` **sí gana un trabajo real**: deduplicar dentro del
lote —una operación por cuenta y por lote—, que es justo lo que el
encadenamiento global hacía gratis.

⚠️ **El precio.** Hoy **cada transición de raíz está demostrada**. Con
lotes, la actualización del árbol la haría el nodo **sin prueba**.

### 5.3.bis Cómo pagar ese precio SIN recursión

> Una versión anterior de este apartado daba el precio por inevitable y
> remitía a la recursión —que la §6.4 declara inviable—. **Eso era
> pesimismo mal razonado**: la propiedad no se pierde, se reparte.

La verificación se divide en dos afirmaciones **independientes**, que no
requieren verificar ninguna prueba *dentro* de un circuito:

| quién | qué demuestra |
|---|---|
| **cada cliente** | conoce su clave, su hoja vieja pertenece a `root_old`, y esta es su hoja nueva. **No afirma `root_new`** |
| **el nodo, una vez por lote** | aplicando estas N hojas nuevas en estas N posiciones sobre `root_old`, sale `root_new` — más la conservación del suministro del lote |

Un verificador comprueba **N pruebas de cliente + 1 prueba de lote**, y
juntas afirman **exactamente lo mismo que hoy afirma una sola**.

**Esto no es recursión**: nadie verifica una prueba dentro de un
circuito; se **componen afirmaciones independientes**. El circuito de
lote es N ascensos de Merkle en una traza — grande, pero de la misma
naturaleza que los dos ascensos que los circuitos actuales ya hacen. Y
el `nonce` cumple aquí su papel: una operación por cuenta y por lote.

**Lo que hay que medir antes de creérselo** (y esta sección ya ha tenido
que corregirse una vez, así que la reserva va en serio):

1. Longitud de traza y **RAM** para N = 50 / 100 / 500. Puede reventar
   antes de lo que la intuición sugiere.
2. Que la **conservación del suministro** se demuestre a nivel de lote,
   no de operación.
3. Que los **tres** árboles (cuentas, pendientes, congelados) entren en
   el mismo lote sin volver a serializar (§1.1.a).
4. El **tiempo de prueba del lote**, que pasa a ser el techo nuevo: si
   probar 100 cuesta 10 s, son ~10 TPS y no se ha ganado tanto.

### 5.4 Lo que sigue sin medirse

- Si la **conservación del suministro** sobrevive a un orden parcial.
- Qué le pasa al árbol de pendientes y a la cabeza de época cuando dos
  cuentas avanzan sin orden entre sí.

Hasta que eso se mida, la §5.3 es **una dirección, no un plan**.

---

## 6. El objetivo RTGS: qué falta de verdad, y qué no es criptografía

Esta sección existe porque la pregunta «¿esto podría servirle a la FED o
al BCE?» se responde casi siempre con la intuición equivocada. Los
números de aquí son ajenos y **llevan fuente y fecha al pie**.

### 6.1 La corrección de encuadre: un RTGS no tiene consenso

**TARGET2 y Fedwire no son sistemas descentralizados.** Los opera un
banco central; el operador de confianza **es** la institución. Por lo
tanto:

> El nodo único de este proyecto **no es descalificante para este caso de
> uso**. Su modelo de confianza residual —el operador ordena y ve el
> estado— **coincide** con el modelo RTGS real en vez de contradecirlo.

Lo descalificante es otra cosa, y no es criptográfica: **alta
disponibilidad, recuperación ante desastres, continuidad operativa
auditada, certificación y firmeza jurídica de la liquidación**.

Corolario práctico, coherente con `SECURITY.md` §6: **perseguir consenso
alejaría del objetivo institucional en vez de acercarlo.** Lo que acerca
es que el operador **no pueda mentir sin dejar evidencia**.

### 6.2 La aritmética, que es mejor de lo que la intuición sugiere

| | |
|---|---|
| Fedwire Funds, volumen medio diario (2024) | **836.322 operaciones** |
| Horario | 22 h por día hábil |
| **Media implícita** | **~10,6 TPS** |
| Pico estimado (concentración de fin de día, 3-5× la media) | **~30-50 TPS** |
| ZK-SSL hoy, bajo concurrencia | **1,5-1,9 TPS** (§1) |

⚠️ **Corrección: un pago son DOS operaciones de capa** (§1.1.b). El
objetivo real no es 10,6 TPS sino **~21 TPS de media** (~1,67 M
operaciones/día), y el pico estimado sube a **~60-105 TPS**.

**La brecha es de ~11× en media y ~30-55× en pico.** Sigue sin ser tres
órdenes de magnitud —es distancia de ingeniería, no de física— pero es
**el doble de lo que decía una versión anterior de este apartado**.

### 6.3 El almacenamiento NO es el descalificador — y una versión anterior de este documento dijo que sí

> **Corrección medida.** Este apartado afirmaba que a volumen Fedwire el
> sistema acumularía ~36 TiB/año de pruebas, y lo llamaba «el verdadero
> descalificador técnico». **Es falso**, y basta leer el código para
> verlo. Se corrige y se deja constancia del error, que es la regla de
> la casa.

**La capa persiste digests, no pruebas.** `crates/zk-ssl/src/log.rs`, en
la definición de `LogEntry`, lo dice literalmente: *«Resumen de la
prueba, no la prueba entera. Guardar las pruebas completas serían ~62 KB
por operación… El resumen basta para atar la entrada a una prueba
concreta, y quien quiera verificarla puede pedirla.»*

Una entrada del registro son `seq + kind + root_old + root_new +
proof_digest + chain` ≈ **137 bytes**. A volumen Fedwire:

| | |
|---|---|
| Crecimiento del registro | **~114 MB/día** · **~42 GB/año** |

*(836.322 × 137 B. Estimación desde la estructura, no medida en disco.)*

Los ~124 MiB por mil pagos son **bytes de prueba generados y en
tránsito**, no crecimiento del ledger. El nodo no los guarda.

⚠️ **Lo que sí queda abierto, y no estaba dicho en ninguna parte**: si
nadie conserva las pruebas, la **re-verificación por un tercero** depende
de que el productor guarde la suya. Es una laguna real —de política de
retención, no de terabytes— y hoy no tiene ni política ni sección propia.
Quien quiera que el registro sea re-verificable por terceros a perpetuidad
tiene que decidir **quién archiva, cuánto tiempo y con qué garantía**.

### 6.4 Las tres palancas, con veredicto de viabilidad

> Una versión anterior las listaba como si las tres estuvieran
> disponibles. **Medidas, una está hecha, una es inviable hoy y una solo
> sobrevive reformulada y con un precio.** Esta tabla es el resultado.

| Palanca | ¿Se puede? | Por qué |
|---|---|---|
| **1. Anti-replay por `nonce`** | ❌ tal como se formuló · ⚠️ **sí reformulada** | El obstáculo no es el `nonce` sino el camino de Merkle contra `root_old` y la afirmación de `root_new` (§5.2). Reformulada —prueba de transición de hoja + lote en el nodo— es viable y cae en el dominio del proyecto, **a costa de que la actualización del árbol deje de estar demostrada** (§5.3) |
| **2. Recursión FRI / lotes** | ❌ **no con estos recursos** | `winterfell` **no trae verificador recursivo** (`grep -c recursi` sobre el crate: 0). Escribir un verificador STARK como AIR es trabajo de nivel investigación —Miden lo construyó sobre Winterfell con un equipo y años—. Adoptar Plonky3 o Miden rompería el criterio de dependencias auditables y el mismo que descartó a RISC Zero por exigir toolchain externa. **Y su motivación declarada era falsa** (§6.3) |
| **3. Generación del cliente** | ✅ **ya conseguido** | `client.rs`; ~1,24 s/día por dispositivo (`doc/ESCALADO.md` §3.1). ⚠️ Con reserva: bajo contención, las **regeneraciones en cascada multiplican el trabajo del cliente**, así que escala en agregado y **se degrada por usuario bajo carga**. Está acoplada a la palanca 1 |

**Lectura honesta de la tabla**: de las tres palancas que este documento
presentó como camino, **queda una dirección** (1 reformulada), **una
puerta cerrada** (2) y **una casilla ya marcada** (3). El techo de
1,5-1,9 TPS **no tiene hoy un camino barato**, y decirlo es más útil que
mantener tres flechas en un diagrama.

### 6.5 La escalera de ingeniería, en orden

*(Ninguno de estos pasos depende de las palancas 1 y 2: son elegibilidad,
no rendimiento. Se pueden hacer hoy.)*

1. **Cifrar el ledger del nodo.** La primitiva existe (`zk_ssl::crypto`,
   `open_encrypted`); falta el cable. Un sello. `SECURITY.md` §3.3.
2. **Autenticación, TLS y límite de tasa en el RPC.** Hoy solo lo protege
   el bind a `127.0.0.1`. `SECURITY.md` §3.3.
3. **El `PendingNotice` y el estándar.** Hoy viaja fuera de banda y
   `spec/RPC.md` declara que ISO 20022 no lo transporta — mientras
   Fedwire completó su migración a ISO 20022 en julio de 2025. Es un
   problema de **especificación**, con cauce: RFC en `spec/rfc/`.
4. **Réplica verificable y acuse** (§121, `doc/CONFIANZA_RESIDUAL.md`):
   que un participante verifique la historia sin tener que creer al
   operador. **Esto es lo que una institución pide de verdad**, y no es
   consenso.
5. **Recuperación ante desastres real**: réplica en caliente, punto de
   recuperación medido, ejercicio de restauración documentado.

### 6.6 Lo que no es ingeniería, y por eso ningún sello lo resuelve

Firmeza jurídica de la liquidación (Directiva de firmeza en la UE,
Regulation J en EE.UU.), certificación operativa, gobernanza de
operadores y continuidad auditada. **Eso solo existe dentro de una
institución.** No se puede construir desde un repositorio; solo se puede
ser **elegible**.

### 6.7 La estrategia que se deduce de todo lo anterior

El camino **no** es escalar hasta parecerse a Fedwire. Es llegar a un
**piloto estrecho** —un supervisor, un banco pequeño, un consorcio de
tres— donde ~1,9 TPS y un operador honesto sean *suficientes*, porque el
argumento de este proyecto no es el rendimiento:

> privacidad **con** supervisión demostrable, **sin ceremonia de
> confianza** y **post-cuántica hoy** — justo cuando la migración
> post-cuántica está abierta como problema de agenda en el BIS y en ambos
> bancos centrales (`SECURITY.md` §3.ter).

El TPS se resuelve después, con recursos, y la aritmética de §6.2 dice
que es alcanzable. Lo de §6.6 no se resuelve sin la institución. **Por
eso el orden correcto es: elegibilidad primero, rendimiento después.**

---

### 6.4.ter Cómo se cierra la brecha: cuatro niveles y una medición previa

La brecha de ~11× **no es un problema, son dos**: ×3 de desperdicio por
contención (1,9 → 5,7) y ×4 de techo del `apply` (5,7 → 21). No se
atacan igual.

**Nivel 0 — ✅ HECHO (§204), y refutó su propia premisa.** Este apartado
decía: «si el coste dominante es I/O, ahí está el múltiplo más barato».
Se midió: **la persistencia es el 3 %**. El `flush` por operación existe,
pero cuesta 0,55 ms de 38. **La premisa del nivel 1 era falsa.**

Lo que sí resultó ser el 93 %: `digest_of_proof`, un hash algebraico
sobre la prueba entera para un resumen que ningún circuito usa (§0.bis).

| nivel | qué | toca circuitos | ganancia esperada |
|---|---|---|---|
| **1 · capa** | ⚠️ **el group commit queda DESCARTADO** (persistencia 3 %). En su lugar: **cambiar `digest_of_proof`** (×14 del techo) y **desacoplar los tres árboles** (§1.1.a, medido en §204: los cobros mueren 4,1× más). La verificación en paralelo rinde poco: es el 7 % | no (el hash sí toca el **cable**: RFC) | **×14** el techo del nodo |
| **2 · circuitos** | **lotes** (§5.3.bis) · **cobro agregado**: N notas pendientes en UNA operación | sí | ×3 (contención) y ×2 (un pago deja de costar dos operaciones) |
| **3 · arquitectura** | **partición por rango de cuentas, con un solo operador**. El diseño de dos fases **ya es el protocolo entre particiones**: `send` debita y emite la nota en la partición de la pagadora, `claim` la consume en la del receptor. La nota es el compromiso atómico | sí | casi lineal en número de particiones, **sin consenso** |
| **4 · dominio** | **netting y ahorro de liquidez**: lo que TARGET2 y Fedwire usan de verdad — compensar ciclos en vez de liquidar bruto en pico | no | reduce el volumen a liquidar desde el otro lado |

⚠️ **Los multiplicadores NO se componen limpiamente**: cada nivel destapa
un techo nuevo. La lectura razonable es que **la media de ~21 TPS parece
alcanzable con los niveles 1 y 2**, que el **pico de 63-106 necesita el
nivel 3**, y que el nivel 4 baja el listón desde el otro extremo.

**El plan de obra detallado, en cinco etapas con sus medidas y sus
compuertas, está redactado en
[`spec/rfc/0002-lotes-y-transicion-de-hoja.md`](../spec/rfc/0002-lotes-y-transicion-de-hoja.md)**
(estado BORRADOR).

Y lo que no cambia: **nada de esto arregla la elegibilidad** (§6.5,
§6.6). El TPS es el problema fácil de los dos.

### 6.4.bis Si algún día hiciera falta recursión: Plonky3 vs Miden

Se deja el análisis hecho para no repetirlo, con la conclusión por
delante: **hoy no hace falta ninguno de los dos**, porque la ruta de
lotes (§5.3.bis) no necesita recursión.

**La comparación está mal planteada de origen**: Miden VM **usa Plonky3
como sistema de prueba**. No son alternativas — Miden corre sobre
Plonky3. La elección real es **biblioteca AIR** frente a **máquina
virtual**.

**Miden VM — ❌ descartado, y lo decide un hallazgo propio.** Es un zkVM:
exactamente la categoría que este proyecto descartó al evaluar RISC Zero
(hallazgo 8: toolchain externa, 3 dependencias frente a 349). Adoptarlo
contradiría un hallazgo publicado. Y el coste mayor no es ese: los
circuitos pasarían a ser **programas**, de modo que la **ESPEC
ejecutable, el censo de celdas y el guardián de 28 circuitos —toda la
escalera FV (§195-§196)— dejarían de aplicar**. Además su recursión
sigue anunciada como próxima, no disponible.

**Plonky3 — ⚠️ el único técnicamente viable, con tres costes.** Mismo
paradigma AIR que Winterfell, así que la migración es AIR→AIR; el propio
equipo de Miden documentó públicamente esa migración desde Winterfell, lo
que prueba que el camino es transitable. Es un toolkit auditado, y
`Plonky3-recursion` ofrece verificación recursiva nativa. Los costes:

1. **Rompe un diferenciador medido de este proyecto**: «3 dependencias
   frente a 349» es el hallazgo 8. Plonky3 son muchos crates.
2. **Colisión fina con el hallazgo 4**: la biblioteca de recursión
   advierte que **las extensiones de campo aún no son totalmente
   parametrizables** — y la extensión de campo es justo lo que hace falta
   aquí para subir de 63 a ~128 bits de solidez. El remedio choca con la
   necesidad.
3. **Migrar 28 circuitos**, el guardián y las ESPECs. Meses.

**Regla para el futuro**: no migrar por rendimiento ni por elegancia.
Migrar solo si aparece una necesidad que la §5.3.bis no cubra — y
entonces reabrir este apartado, no improvisar.

---

## 7. Lo que este proyecto necesita, y no está en el repositorio

Se dice aquí porque es la conclusión honesta del diagnóstico, y porque
ningún sello la resuelve:

- **Auditoría externa**, aunque sea parcial y de un solo circuito. El
  precedente está en `SECURITY.md` §3.1: un sub-restringimiento en
  producción, cuatro años sin verse, encontrado por auditoría y no por la
  suite del propio proyecto.
- **Un segundo par de manos.** Todo el rigor de este repositorio procede
  de un único evaluador. La disciplina de uno no sustituye al desacuerdo
  de dos.
- **Un caso de uso estrecho y real** donde ~1,9 TPS y un operador honesto
  sean *suficientes*. Los sistemas se adoptan por el caso pequeño que
  encajan, no por la visión grande.
- **Una segunda implementación**, aunque sea parcial y de solo lectura.
  Es la única prueba de que el contrato del protocolo
  (`spec/openrpc.json`, `spec/vectors/`) es real y no autorreferencial —
  y encontraría errores que las compuertas propias no ven, porque las
  escribió quien escribió el código.

---

## 8. Cómo usar este documento

- Antes de aceptar una recomendación de escalado —de una persona, de un
  artículo o de un asistente— **contrastarla contra la §2**. Si el
  problema que dice resolver no aparece medido en la §1, no es el
  problema de este sistema.
- Antes de proponer descentralizar, leer la §3: se está proponiendo
  **recuperar dos problemas evitados**, y eso hay que decirlo en la misma
  frase.
- Antes de responder «¿serviría para un banco central?», leer la §6: la
  respuesta corta es que **el nodo único no es el obstáculo**, y que el
  obstáculo real —§6.3 y §6.6— es mitad almacenamiento y mitad
  institucional.
- Si alguna comprobación de la §2 deja de dar el resultado escrito, este
  documento está desactualizado: **corregirlo es parte del cambio que lo
  invalidó.**

---

### Fuentes externas de la §6

Datos ajenos verificados el 07-08-2026; **re-verifícalos**:

- Fedwire Funds Service, PFMI Disclosure 2025: volumen medio diario de
  **836.322 operaciones** en 2024, valor medio diario ~4,51 billones de
  dólares, horario de **22 h por día hábil**
  (`frbservices.org` — «Fedwire Funds PFMI Disclosure»).
- Migración de Fedwire Funds a **ISO 20022 en julio de 2025**
  (Federal Reserve Board, comunicado de octubre de 2025).
- El pico estimado de 3-5× la media es **una estimación de este
  documento**, no un dato publicado: los RTGS concentran volumen al
  final del día. Si alguien mide el pico real, esta cifra se sustituye.

---

*Medido y escrito el 07-08-2026. Todas las cifras propias llevan
referencia a `AUDITORIA.md`; todas las comprobaciones de código son
reproducibles con `grep` sobre este mismo árbol. Sin auditoría externa.*
