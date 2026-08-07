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
| **2 · lotes** | 🔄 **EN CURSO.** Redefinida en §210; **pieza 1 de 3 hecha en §211** (reserva de posiciones). Sigue pendiente, pero **ya no necesita circuitos nuevos ni romper el cable**: la medición mató al circuito de lote y las tres verificaciones encontraron que el obstáculo era de capa | §210 |
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

### Etapa 2 — Lotes **sin circuitos** (redefinida en §210)

> **Esta sección se reescribió después de medir.** La versión anterior
> proponía cambiar `circuit_send`/`circuit_claim` para que dejaran de
> afirmar `root_new`, y añadir un `circuit_batch_root` que el nodo
> probaría. **Las dos piezas se caen**: el circuito de lote por su coste
> medido, y el cambio de circuitos porque resultó innecesario. Se conserva
> escrito abajo lo descartado, con su número.

#### 2.1 Por qué se cae el circuito de lote: está medido

Banco `etapa_b0_lote` (§210). Dos geometrías reales, recta ajustada
`t = 42,3 ms + 3,2 µs por cada 1.000 celdas`:

| N (hojas) | filas | celdas | prueba proyectada | techo |
|---|---|---|---|---|
| 10 | 1.280 → 2.048 | 51.200 | 208 ms | 48 op/s |
| 50 | 6.400 → 8.192 | 204.800 | 705 ms | 71 op/s |
| 100 | 12.800 → 16.384 | 409.600 | **1.368 ms** | **73 op/s** |
| 500 | 64.000 → 65.536 | 1.638.400 | 5.344 ms | 94 op/s |

⚠️ Es **cota optimista**: el término `n·log n` de las FFT y la presión de
memoria empujan el coste real por encima de la recta.

**El circuito de lote sería el cuello nuevo**: ~73 op/s frente a las
**~320 op/s** que el `apply` ya alcanza desde §209 — **4,4× peor**, a
cambio de dos circuitos nuevos, su ESPEC, su censo, y toda la superficie
de sub-restringimiento de la clase §3.1.

#### 2.2 Por qué tampoco hacen falta circuitos nuevos

La prueba de un cliente afirma: *«mi hoja vieja está en `root_old`, y
aplicando mi cambio sale `root_new`»*. Eso **sigue siendo cierto dentro
de un lote**: `root_new` es simplemente «la raíz si el mío fuera el único
cambio desde la raíz de arranque».

Y el nodo **ya sabe calcular esa raíz hipotética**: es el patrón
`tentativo = árbol.clone(); set_leaf(…)` que ya vive en `commitment.rs` y
`burn.rs`, y que desde §207 cuesta un ascenso —32 hashes—.

Luego `apply_many` puede:

1. Fijar la **raíz de arranque** del lote (instantánea de los tres árboles).
2. Por cada operación: calcular su raíz hipotética **desde el arranque** y
   verificar la prueba contra ella, tal cual la verifica hoy.
3. Aplicar las N hojas al árbol real y recomputar la raíz **una vez**.
4. Rechazar dos operaciones de la **misma cuenta** en el mismo lote — el
   nodo sabe de qué cuenta es cada una, así que **ni siquiera hace falta
   avanzar el `nonce`**.

**Sin tocar circuitos. Sin romper el cable. Sin `zkssl/0.3`.**

#### 2.3 Las tres verificaciones exigidas, y su resultado (§210)

| verificación | resultado |
|---|---|
| **¿Algo depende de la raíz ACTUAL y no de una de arranque?** | ✅ **No.** Todo lo que `apply_send` comprueba tras el cerrojo usa solo `pi`, `self.regulatory_limit` y `self.records`. Las dos comprobaciones de raíz son las únicas, y ambas admiten la instantánea |
| **¿Sobreviven los tres árboles?** | ⚠️ **NO, y aquí está el bloqueante** — ver 2.4 |
| **¿Sobrevive la conservación del suministro?** | ✅ **Sí.** Es estructural y por operación —el saldo baja, la nota sube (`pending_amounts`)—; no hay un total recomputado que pueda descuadrar. Condicionado a que no colisionen las posiciones, que es 2.4 |

#### 2.4 El bloqueante: `allocate_pending` colisiona en lote

```rust
pub(crate) fn allocate_pending(&self) -> Result<u64, LayerError> {
    for p in 0..self.next_pending {
        if !self.pending.is_occupied(p) { return Ok(p); }   // ← estado ACTUAL
    }
    Ok(self.next_pending)
}
```

Es **solo lectura sobre el estado actual**. Dos clientes que pidan
materiales contra la misma raíz de arranque **reciben la MISMA posición**:
sus pruebas afirman insertar en la misma hoja del árbol de pendientes, y
el segundo `apply` pisaría la nota del primero.

En el banco `etapa_a5_concurrencia` (§204) no ocurrió porque el mutex
serializaba materiales-y-aplicación. **En un lote no hay nada que lo
impida.**

**Arreglo: reserva de posiciones.** Un contador que avanza **al entregar
materiales**, no al observar ocupación. Es cambio de **capa**, no de
circuito.

⚠️ **Y un segundo, menor**: una **congelación de gobernanza** a mitad de
lote cambia `frozen_root` y mata todas las pruebas en vuelo. Se declara:
**las congelaciones van en su propio lote**.

#### 2.5 Las tres piezas, en orden

1. ✅ **Reserva de posiciones de pendiente — HECHA (§211).**
   `reserve_pending` / `release_pending` / `reserved_pending_count`;
   `allocate_pending` cuenta una reserva como ocupada. **Las reservas no
   se persisten**: un reinicio las anula, que es lo correcto —si nada se
   aplicó, nada hay que respetar—. Cuatro tests, y el primero demuestra
   el bug antes de arreglarlo: sin reservar, dos llamadas devuelven la
   misma posición.
2. ⬜ **`apply_many`** — instantánea de arranque, verificación de cada
   prueba contra su raíz hipotética, aplicación de las N, recómputo una
   vez. **Es lo siguiente.**
3. ⬜ **Una operación por cuenta y por lote.**

Las tres son de capa. **Cero superficie nueva de sub-restringimiento**:
la clase §3.1 no entra en juego, y por eso esta etapa ya **no** exige
ESPEC ejecutable ni censo — no hay circuito nuevo que censar.

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
| **`circuit_batch_root` (el circuito de lote)** | medido en §210: toparía el nodo en **~73 op/s** (N=100) frente a las ~320 que el `apply` ya alcanza — **4,4× peor**, y como cota optimista. Y resulta innecesario: la raíz nueva es determinista dadas las hojas, y quien replique el árbol la recomputa. La réplica verificable es lo que `SECURITY.md` §6 y §121 ya declaran como camino |
| **Cambiar `circuit_send`/`circuit_claim` para no afirmar `root_new`** | innecesario (§210, 2.2): `root_new` sigue siendo cierta dentro del lote como «la raíz si el mío fuera el único cambio», y el nodo la calcula |
| **Recursión FRI / migrar a Plonky3 o Miden** | `winterfell` no trae verificador recursivo; Miden es un zkVM y contradice el hallazgo 8 propio, además de anular la escalera FV. Y la etapa 2 **no necesita recursión** |

---

## Compatibilidad

| etapa | ¿rompe el cable? | vectores |
|---|---|---|
| 1 · el hash | **sí** → `zkssl/0.2` | los de `0.1` **se conservan**; nuevos bajo `0.2` |
| 2 · lotes (redefinida §210) | **NO** — es cambio de capa | intactos; `conformance --check` debe seguir en IDÉNTICO |
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
