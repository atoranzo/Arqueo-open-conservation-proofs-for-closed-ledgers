# RFC-0002 — Transición de hoja, lotes, y el hash del registro

- **Estado:** PROPUESTO
- **Autores:** (mesa)
- **Fecha:** 2026-08-07
- **Versión del protocolo afectada:** `zkssl/0.1` → `zkssl/0.2`
- **Asiento(s) de AUDITORIA:** §204 (las mediciones), §205 (las
  correcciones de `doc/ESCALADO`), §206 (este documento), §207 (etapa 3),
  §209 (etapa 1)

## Estado de las etapas

| etapa | estado | dónde |
|---|---|---|
| **1 · el hash del registro** | ✅ **HECHA** — `apply` de 33,27 a ~3,2 ms; techo del nodo de 30 a **~320 op/s**. Subió a `zkssl/0.2` y se emitieron vectores nuevos conservando los de `0.1` | §209 |
| **2 · transición de hoja + lote** | ⬜ **PENDIENTE.** Es la única que queda, y la que ataca la contención (66 % del trabajo desperdiciado). Rompe el cable otra vez | — |
| **3 · árbol incremental** | ✅ **HECHA** — `send_materials` de e = 1,08 a ~0,1. No tocó cable ni circuitos | §207 |

⚠️ **Corrección a la sección «Compatibilidad» de más abajo**: este RFC
decía que las etapas 1 y 2 debían compartir una sola emisión de
`zkssl/0.2`. **No se hizo así**: la etapa 1 se sella sola y emite `0.2`
ya. La etapa 2 volverá a romper el cable y necesitará **`zkssl/0.3`**.
El motivo del cambio de plan es que la etapa 1 era un cambio de una
función con un ×10 medido, y retenerla meses esperando a la etapa 2
—que introduce circuitos nuevos y su escalera FV— habría sido pagar un
coste real por una elegancia de numeración.

> **Nota de numeración.** El **RFC-0001** queda reservado al
> endurecimiento del KDF del keystore (SHA-256 → Argon2id), señalado en
> `crates/zk-ssl/src/crypto.rs` y en `zk-ssl-sdk/src/keystore.rs`. **No
> está redactado.** Este RFC toma el 0002 porque `AUDITORIA.md` §204 y
> `doc/DIAGNOSTICO_ESCALADO.md` ya lo referencian por ese número.

> **Este documento se reescribió DESPUÉS de medir.** Su primera versión
> proponía cinco etapas conjeturadas; tres de sus premisas cayeron al
> ponerlas a prueba (§204). Lo que sigue es el plan que dictan las
> medidas, con lo descartado escrito al final —no borrado— para que
> nadie vuelva a proponerlo sin leer por qué se cayó.

---

## Motivación

Todo lo de abajo está **medido**, no estimado. Los bancos viven en
`crates/zk-ssl/examples/` y se ejecutan con `--features sandbox`; las
tablas completas, en `AUDITORIA.md` §204.

| | medido |
|---|---|
| `apply` por operación | **37,99 ms** (disco) → techo **26,3 op/s** |
| … de los cuales persistencia | **3 %** |
| … verificación STARK | **7 %** |
| … **`digest_of_proof`** | **93 %** — 30,99 ms, **2.915×** lo que cuesta Blake3 sobre los mismos bytes |
| Rendimiento con 4 hilos | **1,57 pagos/s** · **3,83 regeneraciones por pago** · **0,84×** respecto a 1 hilo |
| Trabajo criptográfico desperdiciado | **66 %** (70 generaciones para 24 operaciones) |
| `send_materials` frente al nº de cuentas | **e = 1,08** (lineal); a 1.000 cuentas, ~248 ms por llamada |

Dos hechos ordenan el plan entero:

1. **El techo del nodo lo fija un hash que ningún circuito usa.**
   `proof_digest` y `chain_digest` no aparecen en `stark-experiment`.
2. **La contención existe y es un livelock**: añadir hilos **empeora** el
   rendimiento. Y con un solo hilo salen 1,85 pagos/s sin una sola
   regeneración — la suma en serie. **El sistema está en serie Y ADEMÁS
   se degrada al paralelizarlo.**

---

## Diseño

### Etapa 1 — `digest_of_proof` deja de usar un hash algebraico

**La más barata y la de mayor efecto. No toca ningún circuito.**

`log::digest_of_proof` recorre la prueba en bloques de 16 bytes y aplica
`native_merge` (Rescue) a cada uno: **4.115 permutaciones** para una
prueba de envío de 65.840 bytes.

**Cambio:** `digest_of_proof` pasa a un hash no algebraico —Blake3, ya
presente vía `winterfell`—.

**Lo que NO cambia:** `chain_digest` (5 merges, ~65 µs) **se queda en
Rescue**. Podría entrar en circuito el día que se implementen las
cabezas atestiguadas (§121); `digest_of_proof` no entrará nunca, porque
resume bytes opacos.

**Efecto, por extrapolación directa de la medida:** `apply` de 33,27 →
**2,29 ms**; techo del nodo de **30 → 436 operaciones/s**.

⚠️ **Lo que esto NO arregla:** la contención. Con el 66 % del trabajo
todavía desperdiciado, el rendimiento efectivo sube solo un **+13 %**
(1,81 → 2,03 pagos/s), porque en la vía serializada manda la generación.
**Esta etapa sube el techo; no lo hace alcanzable.**

**Compatibilidad:** cambia `proof_digest`, y con él la cadena y la cabeza
de época → **cambia los vectores de conformidad**. Es `zkssl/0.2`, y los
vectores de `0.1` se conservan (regla 2 del PROCESO).

### Etapa 2 — Transición de hoja y prueba de lote

**El cambio arquitectónico, y el único que ataca la contención.**

**2.1 — El cliente deja de afirmar la raíz nueva.** Hoy `circuit_send` y
`circuit_claim` afirman `root_old` **y** `root_new`. Pasan a afirmar solo
pertenencia bajo `root_old` y **cuál es su hoja nueva**.

Y **el `nonce` por fin avanza**: hoy `circuit_send` documenta que «el
nonce NO cambia: destruir no consume el derecho». Con lotes, el circuito
exige `nonce_new = nonce_old + 1`, y eso deduplica dentro del lote: **una
operación por cuenta y por lote**, comprobable sin confiar en nadie.

**2.2 — Circuito nuevo `circuit_batch_root`**, que prueba el nodo: dado
`root_old` y N pares (posición, hoja_nueva), demuestra `root_new`, más la
conservación del suministro del lote. Traza: **N ascensos de Merkle** —
grande, pero de la misma naturaleza que los dos que los circuitos
actuales ya hacen. **No es recursión**: nadie verifica una prueba dentro
de un circuito.

**2.3 — Qué verifica un tercero.** `N pruebas de cliente + 1 prueba de
lote` afirman **exactamente lo mismo** que hoy afirma una prueba por
operación. La propiedad «cada transición de raíz está demostrada» **no se
pierde: se reparte**.

**2.4 — El desacoplamiento de las tres raíces viene incluido, y NO era
una etapa aparte.** Una versión anterior de este RFC —y el traspaso de
§204— decía que desacoplar los tres árboles era «la parte más barata de
atacar». **Es falso, y conviene dejarlo escrito.** El `claim` muere 4,1×
más que el `send` porque se ata a `pending_root_old`, y **no puede dejar
de morir sin cambiar lo que la prueba afirma** — que es exactamente esta
etapa. No hay atajo previo.

*(El único desacoplamiento barato posible sería el de `frozen_root`, que
cambia con muy baja frecuencia y podría fijarse por época. Pero A.5 midió
que el problema es `pending_root`: sería arreglar lo que no duele.)*

**Lo que puede matar esta etapa, y se mide ANTES de escribir código de
producción:**

1. RAM y longitud de traza del circuito de lote para N = 50 / 100 / 500.
2. **Tiempo de prueba del lote**, que pasa a ser el techo nuevo: si
   probar 100 cuesta 10 s, son ~10 op/s y no compensa.
3. Que la conservación del suministro se demuestre a nivel de lote.
4. Qué le pasa al árbol de pendientes y a la cabeza de época cuando dos
   cuentas avanzan sin orden entre sí.

### Etapa 3 — Árbol incremental (implementación, **sin RFC**)

`SparseTree::root()` recomputa el árbol entero en cada llamada, y en cada
nodo decide la ocupación con un **barrido lineal de todas las hojas**.

**A.3 refutó que esto domine el `apply`** (exponente 0,18: plano). Pero
midió que **`send_materials` es lineal (e = 1,08)** y crece: 0,64 ms con
4 cuentas, 11,84 con 60, y **~248 ms extrapolados a 1.000** — corriendo
**en el nodo**, en cada envío.

**Cambio:** cachear la raíz e invalidarla al insertar; guardar nodos
internos y actualizar solo el camino de la hoja modificada.
`O(k²·d)` → `O(d)`.

**No toca el cable ni los circuitos.** Se incluye aquí porque sin él las
otras dos etapas se comen el beneficio en cuanto el ledger tenga tamaño
real. **No requiere RFC**: puede sellarse por su cuenta.

### Opcionales, y por qué no son prioritarias

**Cobro agregado** (N notas pendientes en una operación): divide por dos
el número de operaciones por pago. Real, pero la etapa 2 lo subsume en
buena parte y añade un circuito más —una superficie más de
sub-restringimiento—. Se abre solo si tras la etapa 2 el objetivo sigue
sin alcanzarse.

**Partición por rango de cuentas** (un solo operador; el diseño de dos
fases ya es el protocolo entre particiones). Escalado casi lineal **sin
consenso**. Se abre solo si el pico objetivo lo exige. Ver
`doc/DIAGNOSTICO_ESCALADO.md` §6.4.ter, nivel 3.

---

## Lo que se DESCARTÓ al medir

Se conserva escrito para que no vuelva a proponerse sin leer por qué cayó.

| propuesta | por qué se descarta |
|---|---|
| **Group commit / journal** (era la etapa B) | la persistencia es el **3 %** del `apply` (banco A). Optimizaría 1,26 ms de 38 |
| **Anti-replay por `nonce` de cuenta** | el obstáculo no es el `nonce` sino el camino de Merkle contra `root_old` y la afirmación de `root_new`. Quitar la atadura de raíz reintroduce el marcador de doble gasto retirado en §32/§36 |
| **Arreglar el árbol para acelerar el `apply`** | exponente **0,18**: plano (A.3). El árbol sí hay que arreglarlo, pero por `send_materials` — etapa 3 |
| **Verificación en paralelo como palanca principal** | la verificación es el **7 %** (A.2) |
| **Delegar la generación a GPUs** | prueba el **cliente** (`client.rs`); el testigo lleva material de la clave de gasto. Sería una regresión de seguridad, no una optimización |
| **Recursión FRI / migrar a Plonky3 o Miden** | `winterfell` no trae verificador recursivo; Miden es un zkVM y contradice el hallazgo 8 propio, además de anular la escalera FV. Y la etapa 2 **no necesita recursión** |

---

## Compatibilidad

| etapa | ¿rompe el cable? | vectores |
|---|---|---|
| 1 · el hash | **sí** → `zkssl/0.2` | los de `0.1` **se conservan**; nuevos bajo `0.2` |
| 2 · transición de hoja + lote | **sí** → `zkssl/0.2` | ídem |
| 3 · árbol incremental | **no** | intactos; `conformance --check` debe seguir en IDÉNTICO |
| cobro agregado (opcional) | sí | ídem |
| partición (opcional) | sí (direccionamiento de notas) | a decidir al abrirla |

Regla 2 del PROCESO: **los vectores viejos jamás se reescriben.**

Las etapas 1 y 2 rompen el cable por la misma razón (la cadena y las
entradas públicas), así que **conviene emitir `zkssl/0.2` una sola vez**,
cuando las dos estén hechas, en vez de dos versiones seguidas.

---

## Seguridad

**Efecto sobre el principio del API —la clave de gasto no viaja jamás:
NINGUNO.** En las tres etapas el testigo sigue siendo del titular y la
prueba se genera en local. La etapa 2 **reduce** lo que el cliente
afirma; no amplía lo que expone.

**Efecto sobre las deudas declaradas:**

- **Aviso fuera de banda (§21):** sin cambio.
- **Nodo único:** sin cambio. Este RFC **no introduce consenso** en
  ninguna etapa, deliberadamente — `SECURITY.md` §6.
- **`--dev`:** sin cambio.

**Riesgo de la etapa 1.** Cambiar el hash del registro cambia la cadena y
la cabeza de época. Un verificador que compare cabezas antiguas con
nuevas **verá divergencia legítima**: por eso sube la versión y los
vectores viejos se conservan. La resistencia a colisión pasa a descansar
en Blake3 en lugar de Rescue para `proof_digest`; para el uso que tiene
—atar una entrada a una prueba concreta— es equivalente o mejor.

**Riesgo de la etapa 2.** El nodo pasa a producir una prueba propia. Si
faltara o fuera incorrecta, la actualización del árbol quedaría sin
demostrar: `circuit_batch_root` es **obligatorio**, y su ausencia debe
ser un error de verificación, no una advertencia.

⚠️ **Riesgo de la clase §3.1 (`SECURITY.md`).** La etapa 2 introduce
**circuitos nuevos**, y por tanto superficies nuevas de
sub-restringimiento — la clase que este proyecto declara como prioridad
más alta, y la que costó cuatro años al pool Orchard de Zcash. **La etapa
2 no se sella sin ESPEC ejecutable y censo de celdas con 0 sin dueño**,
igual que `circuit_send` y `circuit_claim` en §195-§196. Sin eso, la
ganancia de rendimiento se paga en la moneda más cara de esta casa.

---

## Referencias

- `AUDITORIA.md` §204 (las cinco mediciones, con sus tablas), §205
  (correcciones de `doc/ESCALADO`), §123 (contención), §32 y §36
  (retirada del nullifier), §195-§196 (ESPEC ejecutable), §198 (proceso
  RFC), §121 (cabezas atestiguadas)
- `doc/DIAGNOSTICO_ESCALADO.md` — §0.bis (lo medido), §2 (lo que NO es el
  cuello de botella), §6 (el objetivo RTGS)
- `doc/ESCALADO.md` §2.1 y §2.2 (corregidos con lo medido)
- `SECURITY.md` §3.1 (sub-restringimiento), §6 (el consenso como último
  intermediario)
- Bancos: `crates/zk-ssl/examples/etapa_a_apply.rs`, `etapa_a2_verify.rs`,
  `etapa_a3_escala.rs`, `etapa_a4_hash.rs`, `etapa_a5_concurrencia.rs`
- Código: `crates/zk-ssl/src/log.rs` (`digest_of_proof`),
  `crates/zk-ssl/src/two_phase.rs` (el cerrojo de tres raíces),
  `crates/zk-ssl/src/sparse_tree.rs` (el barrido lineal),
  `crates/stark-experiment/src/circuit_send.rs` (el `nonce` que no avanza)
