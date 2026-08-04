# B13/B14 · Paso 2, prerrequisito — MAPA COMPLETO de la geometría de `circuit_send`

Existe por §139: tres reversiones demostraron que la geometría de este
circuito no se puede mover por corrimiento manual, y el asiento manda
producir este mapa ANTES de decidir hack o refactor. Todo lo que sigue
está **leído en `circuit_send.rs` (HEAD `1cbfedc`) y verificado
mecánicamente** con `verifica_geometria.py` (adjunto; candidato a `tools/`): el script
reconstruye el calendario fila a fila desde las constantes exactas,
cruza las tres representaciones entre sí, y reproduce los dos intentos
fallidos con su estallido en la misma línea. Sin `cargo` en esta sesión
(sin red): la verificación es aritmética exacta, no compilación — cada
número de este documento sale del script, no de memoria.

Las líneas citadas son las del árbol actual. ⚠️ §139 citó «líneas
477-489» para el `match`; hoy ese bloque vive en 464-478 (el asiento se
escribió sobre un estado intermedio). Este mapa clava las de `1cbfedc`.

## 0. La unidad de medida: el ciclo de 8

Todo el circuito late en ciclos de `CYCLE_LENGTH = 8` filas: **7 rondas
Rescue** (`NUM_ROUNDS = 7`, filas con `pos = r % 8 < 7`) más **una fila
de enlace** (`pos == 7`) donde el estado se vacía y se re-siembra para
el merge siguiente. El estado tiene `STATE_WIDTH = 12`: capacidad
`0..4`, mitad izquierda del rate = digest saliente `4..8`, mitad derecha
`8..12`. Dos carriles: A en columnas `0..12`, B en `12..24` (`LANE_B`),
estado VIEJO y NUEVO (spec de la máquina de hoja, §1).

Convenio que todo el mapa usa: **el ciclo `c` ocupa las filas
`8c..8c+7`; su fila de enlace es `8c+7`; el digest disponible en esa
fila es la salida del merge hasheado durante `c`; lo que la fila de
enlace siembra es la entrada del ciclo `c+1`**. El `match r` de
`build_trace` decide la siembra; la aritmética del brazo genérico usa
`next_cycle = (r+1)/8 = c+1`.

## 1. La línea temporal completa (calendario verificado, sección A del script)

| ciclos | filas | qué hashea | fila de enlace y evento | gobierna |
|---|---|---|---|---|
| 0 | 0..7 | `H(id_cuenta, saldo)` ×2 carriles (saldo viejo en A, nuevo en B) | `7 = ROW_LEAF_LINK`: siembra `(digest, nonce)` | brazo explícito |
| 1 | 8..15 | `H(interno, nonce)` = `native_leaf` | `15 = ROW_LEAF_DONE`: coloca la hoja en el **nivel 0** de cuentas | brazo explícito |
| 2..33 | 16..271 | subida de cuentas, nivel `c−2` en el ciclo `c` | `8c+7` con `nc=c+1 ∈ (2..34)`: coloca nivel `nc−2` (efectivo: **niveles 1..31 en filas 23..263**) | rango genérico |
| 33→34 | 271 | — | `271 = ROW_ROOT`: raíces A/B **atadas por aserción**; siembra `(SPEND_KEY_DOMAIN, clave)` | brazo explícito |
| 34 | 272..279 | derivación de la identidad (`272 = ROW_PK_START`, aserción del dominio) | `279 = ROW_PK_DONE`: **titularidad** (`C_PK_CHECK`); coloca hoja CERO en **nivel 0** frozen | brazo explícito |
| 35..58 | 280..471 | subida frozen, nivel `c−35` en el ciclo `c` | `8c+7` con `nc ∈ (36..60)`: coloca nivel `nc−35` (efectivo: **niveles 1..23 en filas 287..463**) | rango genérico |
| 58→59 | 471 | — | `471 = ROW_FROZEN_ROOT`: raíz frozen **atada por aserción**; siembra `(id_receptor, aleatorio)` | brazo explícito |
| 59 | 472..479 | compromiso interno `H(id_r, aleatorio)` | `479 = ROW_PEND_INNER`: siembra `(interno, importe)` | brazo explícito |
| 60 | 480..487 | el pendiente `H(interno, importe)` | `487 = ROW_PENDING_ENTRY`: A coloca CERO, B el compromiso, **nivel 0** pendientes | brazo explícito |
| 61..92 | 488..743 | subida de pendientes, nivel `c−61` en el ciclo `c` | `8c+7` con `nc ∈ (62..94)`: coloca nivel `nc−61` (efectivo: **niveles 1..31 en filas 495..735**) | rango genérico |
| — | 743 | — | `743 = ROW_PENDING_ROOT`: raíces A/B **atadas por aserción**. El bucle `for r in 0..ROW_PENDING_ROOT` (línea 399) **no procesa esta fila**: no hay siembra | límite del bucle |
| 93..127 | 744..1023 | **NADA** — holgura | sin `hash_flag`, sin ARK, carriles libres | (§6) |

Verificado: cada nivel de cada árbol se coloca **exactamente una vez**
(31+1, 23+1, 31+1) y no queda ninguna fila de enlace sin brazo. El
último ciclo de cada subida no lleva enlace periódico: su salida es la
raíz, que atan las aserciones, no una restricción de transición.

## 2. Las TRES representaciones, sitio por sitio

### R1 — Las constantes `ROW_*` (filas). 9 constantes, 6 familias de consumidores

Declaradas en 143-167: `ROW_LEAF_LINK=7`, `ROW_LEAF_DONE=15`,
`ROW_ROOT=271`, `ROW_PK_START=272`, `ROW_PK_DONE=279`,
`ROW_FROZEN_ROOT=471`, `ROW_PEND_INNER=479`, `ROW_PENDING_ENTRY=487`,
`ROW_PENDING_ROOT=743`. Sus consumidores — TODOS, censo §137:

1. **Brazos del `match r`** (410-463): siete de las nueve
   (`ROW_PK_START` y `ROW_PENDING_ROOT` no tienen brazo).
2. **Límite del bucle** (399): `for r in 0..ROW_PENDING_ROOT` — esta
   constante es a la vez hito y frontera de TODO el calendario.
3. **Periódicas puntuales** (661-762): `link_leaf[ROW_LEAF_LINK]`,
   `link_place[ROW_LEAF_DONE]`, selectores `[0, ROW_ROOT, ROW_PK_DONE]`,
   `frozen_entry[ROW_PK_DONE]`, `pend_in[ROW_FROZEN_ROOT]`,
   `pend_val[ROW_PEND_INNER]`, `pend_entry[ROW_PENDING_ENTRY]`; y la
   **extensión** de `hash_flag` y las 24 columnas ARK:
   `0..=ROW_PENDING_ROOT` (667, 677).
4. **Aserciones** (1019-1089): filas `0`, `ROW_ROOT`, `ROW_PK_START`,
   `ROW_FROZEN_ROOT`, `ROW_PENDING_ROOT`.
5. **`get_pub_inputs`** (1116-1153): lee la traza en `ROW_ROOT`,
   `ROW_FROZEN_ROOT`, `ROW_PENDING_ROOT` y la fila 0.
6. **Test de hitos** `trace_landmarks_match_native` (1706-1776): usa las
   constantes, no números propios — se mueve con R1 solo.

`ROW_PK_START` solo vive en aserciones; `ROW_PENDING_ROOT` en el límite
del bucle, la extensión de periódicas, aserciones, `get_pub_inputs` y el
test — **nunca en un brazo del match**.

### R2 — Los offsets de ciclo en columnas. 3 literales (2 / 35 / 61), 6 bucles

| literal | bucles de bits en `build_trace` | periódicas de enlace en el AIR |
|---|---|---|
| `2` (cuentas) | `rows[(2+level)*8+p][COL_BIT]`, niveles 0..31 (507-516) | `link_merkle[(2+level)*8+7]`, niveles 0..30 (691-695) |
| `35` (frozen) | `rows[(35+level)*8+p][COL_FBIT]`, niveles 0..23 (496-505) | `frozen_link[(35+level)*8+7]`, niveles 0..22 (734-738) |
| `61` (pendientes) | `rows[(61+level)*8+p][COL_PBIT]`, niveles 0..31 (485-494) | `pend_link[(61+level)*8+7]`, niveles 0..30 (756-760) |

Semántica verificada (sección C del script): el bit del nivel `l` vive
durante el ciclo que HASHEA el nivel `l` — y por eso cubre la fila
`next` de la transición que lo COLOCA (`C_PLACE` lee `next[COL_BIT]`).
Los enlaces periódicos genéricos coinciden fila a fila con las
colocaciones genéricas del calendario (31/23/31 unos).

⚠️ **El off-by-one más fino del fichero está aquí**, y es el ejemplar
exacto de las «convenciones distintas» de §139: la MISMA expresión
`(2+level)*8` significa dos cosas según el bucle. En los bucles de bits,
`level` es el nivel que el ciclo `2+level` **hashea** (por eso van
0..31). En los constructores de enlace, el uno que se enciende al final
del ciclo `2+level` **coloca el nivel `level+1`** (por eso van 0..30: la
transición de salida del último ciclo es la fila de raíz, gobernada por
R1 — `sel_root`/aserciones — y NO debe llevar colocación genérica).
Mismos literales, marcos desplazados en uno.

### R3 — Los rangos + aritmética del `match` (464-478). 1 sitio, 3 convenciones

```rust
let next_cycle = (r + 1) / CYCLE_LENGTH;
if (2..34).contains(&next_cycle)      { place(…,        next_cycle - 2)  }
else if (36..60).contains(&next_cycle){ place_frozen(…, next_cycle - 35) }
else if (62..94).contains(&next_cycle){ place_pending(…, next_cycle - 61)}
// si ningún brazo ni rango aplica: el estado queda A CERO (siembra vacía)
```

La aritmética es estructuralmente la misma en los tres tramos (`nivel =
next_cycle − arranque_del_tramo`), pero los LITERALES de los rangos no
codifican el arranque de forma uniforme: cuentas arranca el rango en su
arranque real (2), frozen y pendientes lo arrancan en `arranque+1` (36,
62) porque su primer valor está sombreado — ver §3. Y la rama vacía es
silenciosa: un valor no cubierto no estalla, deja el estado a cero y la
prueba muere aguas abajo en `C_HASH` con un diagnóstico opaco.

## 3. Los ciclos-frontera, explicados (el porqué de los saltos 34→36 y 60→62)

Los rangos SOBRE-CUBREN y dependen de guardianes EXTERNOS para no pisar
valores ilegales — y los guardianes **no son del mismo tipo** en cada
frontera. Tabla completa (sección B del script):

| `next_cycle` | fila `8·nc−1` | qué pasaría en el rango | quién lo impide |
|---|---|---|---|
| 2 | 15 | cuentas nivel 0 (duplicado) | **sombreado** por `ROW_LEAF_DONE` |
| 34 | 271 | — (34 excluido del rango) | el propio rango; además brazo `ROW_ROOT` |
| 35 | 279 | — (fuera de todo rango; siembra vacía) | **sombreado** por `ROW_PK_DONE` |
| 59 | 471 | frozen **nivel 24 sobre path de 24 → pánico** | **solo** el sombreado por `ROW_FROZEN_ROOT` |
| 60 | 479 | — (fuera de rango; siembra vacía) | sombreado por `ROW_PEND_INNER` |
| 61 | 487 | — (fuera de rango; siembra vacía) | sombreado por `ROW_PENDING_ENTRY` |
| 93 | 743 | pendientes **nivel 32 sobre path de 32 → pánico** | **solo** el límite del bucle (`0..ROW_PENDING_ROOT`) — `ROW_PENDING_ROOT` no tiene brazo |
| 94 | 751 | — (94 excluido) | el propio rango; y el límite del bucle |

Tres mecanismos distintos protegen las fronteras: (a) el propio límite
del rango, (b) el sombreado por un brazo explícito, (c) el límite del
bucle. **Solo el primero viaja con el rango.** Los otros dos viven en
R1: por eso mover R1 sin R3 detona, y detona precisamente en frozen —
el único tramo cuyo valor ilegal superior (nc=59) está guardado
únicamente por la constante que el corrimiento mueve.

**Reconstrucción de los intentos (sección D del script, coincide con
§139 al detalle).** Intento 1 (`ROW_*` +8, resto viejo) e intento 2
(`ROW_*` +8 y offsets de columna +1, rangos viejos): en ambos, la fila
471 deja de ser `ROW_FROZEN_ROOT` (ahora 479), cae al brazo genérico con
`nc=59 ∈ (36..60)`, y `place_frozen` recibe `nivel 24` sobre un path de
24 → índice fuera de rango, **misma línea las dos veces** (la clausura
de 369-377). El intento 2 movió R2, que no participa en el `match`:
tocar los bucles de bits y las periódicas no cambia dónde estalla.

## 4. El censo ampliado (R4): dónde MÁS vive la geometría

El grep de §139 (`const ROW_`, `* CYCLE_LENGTH`) se perdió el `match`.
Este censo busca las CUATRO formas — y lo que encuentra de más:

- **Siguen a R1 (se mueven solas con las constantes):** aserciones,
  `get_pub_inputs`, extensión de `hash_flag`/ARK, límite del bucle,
  test de hitos. Ninguna esconde un literal propio. ✔
- **Fuera del fichero: NADA.** Las `ROW_*` son privadas; `zk-ssl` solo
  importa `build_trace`/`SendProver`/`SendAir`/`SendPublicInputs`
  (lib.rs:140, two_phase.rs:53, metrics.rs:94). `doc/air/` documenta
  burn y mint, no send. ✔
- ⚠️ **Los comentarios, la representación informal (la cuarta):**
  1. `TRACE_LENGTH` (96-101), **DESFASADO seguro**: «las fases del
     pendiente llegan a la 1007 … quedan 16 filas de margen». Realidad:
     llegan a la **743**; el margen es **280 filas** (§6). Describe una
     geometría anterior, internamente coherente pero muerta.
  2. `ROW_PEND_INNER` (158-162), **marco ambiguo**: «Inserción del
     pendiente: ciclos 60..91, filas 480..735» cuenta los ciclos cuyas
     filas de enlace COLOCAN (60→fila 487 la entrada, 61..91→495..735
     los niveles 1..31); su vecino en `ROW_PENDING_ROOT` («ciclos
     61..92, filas 488..743») cuenta los ciclos que HASHEAN la subida.
     Ambos ciertos, cada uno en su marco — que es justo la ambigüedad
     que este mapa existe para eliminar.
- ⚠️ **Corrección menor a §139**: «filas 744→751, 281 libres» — son
  **280** (744..1023); la 743 no es libre, sostiene las raíces atadas por aserción.
- **`FROZEN_DEPTH` es coordenada COMPARTIDA** (la lección de §137,
  vigente aquí): vive en `circuit_freeze.rs:61` y la importan send,
  claim, burn, frozen_climb y settlement; la capa ya tiene
  `FROZEN_DEPTH_POST = 32` en `migration.rs:20` (1b). Consecuencia para
  el piloto en §7.

## 5. Qué restricción mira cada región (para saber qué se rompe al mover qué)

- `C_HASH_A/B` (804): activa en toda fila con `hash_flag=1` — ata las 12
  columnas del estado; es la que arrastra los limbos sueltos (§138) y la
  que muere, con error opaco, si una siembra queda vacía.
- Bloque Merkle compartido (808-827): `tree_link = link_merkle +
  link_place` gobierna capacidad, colocación A/B y hermano compartido de
  la subida de CUENTAS — la colocación del nivel 0 y la de los niveles
  1..31 pasan por el MISMO gate.
- Frozen (928-940) y pendientes (950-1001): gates propios
  (`frozen_entry/frozen_link`, `pend_in/pend_val/pend_entry/pend_link`),
  misma estructura.
- Booleanos de bits (829, 940, 1001) y transporte (877-909, 1002):
  activos en TODA la traza, holgura incluida — son los únicos que
  restringen las filas 744..1023 (y se cumplen allí trivialmente:
  columnas difundidas constantes, bits a cero).
- Aserciones: fijan fila 0 (capacidades y rate alto a cero, importe,
  límite, suministros), `ROW_ROOT`, `ROW_PK_START`, `ROW_FROZEN_ROOT`,
  `ROW_PENDING_ROOT`. Cuenta declarada: 42 (652).

## 6. La holgura del final

Filas **744..1023: 280 filas = 35 ciclos exactos** sin tubería de hash
(`hash_flag` y ARK a cero desde 744; carriles 0..24 sin restricción
activa; solo los globales de §5, que se cumplen solos). Es el espacio
que el HACK de §139 quería usar, y el colchón que hace viable el
corrimiento del refactor.

## 7. Presupuesto del piloto completo (sección E del script)

El paso 2 no es solo el salt: es **salt + frozen-32** (Adenda 9; spec
§2-§3). En filas:

| frente | coste | `ROW_PENDING_ROOT` |
|---|---|---|
| hoy | — | 743 |
| + salt (un ciclo tras el nonce) | +8 | 751 |
| + frozen-32 (24→32 niveles) | +64 | **815** |

Margen restante: **208 filas** — `TRACE_LENGTH = 1024` alcanza para
send; la tabla de presupuesto por circuito de la spec (§3) recibe aquí
su primera fila con dato.

⚠️ **Cuestión de alcance que el censo destapa** (decisión del autor, no
de este mapa): `FROZEN_DEPTH` es global a cinco circuitos + settlement.
El piloto no puede subirla a 32 solo para send tocando la constante —
rompería los otros cuatro de golpe. Dos formas de estadificar: (i) el
piloto parametriza la profundidad frozen de send localmente (p. ej.
`SEND_FROZEN_DEPTH`, hoy `= FROZEN_DEPTH`, y el flip global llega con el
paso 3 atómico); (ii) el piloto ejecuta salt completo + la MECÁNICA de
frozen-32 preparada pero sin flip, y frozen-32 se enciende en el paso 3
para los cinco a la vez (el despliegue ya es atómico por diseño de la
migración). La opción (i) mantiene la promesa «piloto completo en un
circuito»; la (ii) mantiene «una constante, una verdad» (§125). El mapa
deja las dos sobre la mesa con el censo delante.

## 8. La elección: HACK contra REFACTOR, con el mapa delante

### El hack, costeado de verdad

La idea de §139: el ciclo del salt en la holgura (p. ej. filas 744-751),
enlace lógico hacia atrás, cero corrimiento. El mapa revela lo que
cuesta ejecutarla, porque los enlaces necesarios **no son locales** (las
restricciones ven un marco de 2 filas) y hay que transportarlos en
columnas:

- Variante transporte: capturar la hoja sin sal de la fila 15 (A y B),
  sembrarla en 743→744 con el salt, atar la salida de 751, y hacer que
  la colocación de la fila 15→16 coloque el valor transportado. **+20
  columnas** (52→72: sal 4, hoja sin sal A/B 8, hoja salada A/B 8),
  ~40 ranuras de restricción nuevas.
- Variante recomputación: la cola re-deriva la hoja desde las columnas
  difundidas (id, saldo, nonce: 3 ciclos), solo transporta la salada.
  **+12 columnas** (52→64), pero deja los ciclos 0-1 como maquinaria
  cuyo producto ya no consume nadie — contra §125.
- Ambas exigen **partir `tree_link`** (la colocación del nivel 0 deja de
  ser la genérica): tocar el bloque Merkle compartido, el punto más
  delicado del AIR; extender `hash_flag`/ARK y el bucle a la cola; crecer
  y reordenar la cola del vector de grados; nuevos hitos y mutaciones.
- El coste de lectura — el trace deja de ser temporal — se paga en cada
  auditoría futura, **multiplicado por los diez AIR** del paso 3.
- **Y el argumento que decide**: el hack esquiva el corrimiento del salt
  (8 filas) pero **no el de frozen-32 (64 filas)**, que mueve
  `ROW_FROZEN_ROOT` — fila de aserción y de siembra del pendiente — y
  todo lo posterior. Meter también frozen-32 en la cola exigiría otro
  transporte (+4 columnas), mover igualmente la constante de la raíz, y
  una topología ya barroca. **El corrimiento hay que hacerlo de todos
  modos; el hack solo aplaza 8 de sus 72 filas y cobra columnas y
  legibilidad por el aplazamiento.**

### El refactor, con la forma derivada ya demostrada

La sección F del script prueba que las NUEVE constantes de fila colapsan
en **ocho arranques de ciclo derivados** de `(CYCLE_LENGTH, TREE_DEPTH,
FROZEN_DEPTH)`:

```rust
const CYC_NONCE: usize      = 1;
const CYC_ACC: usize        = 2;                          // → 3 con salt
const CYC_PK: usize         = CYC_ACC + TREE_DEPTH;       // 34
const CYC_FROZEN: usize     = CYC_PK + 1;                 // 35
const CYC_PEND_IN: usize    = CYC_FROZEN + FROZEN_DEPTH;  // 59
const CYC_PEND_VAL: usize   = CYC_PEND_IN + 1;            // 60
const CYC_PEND_CLIMB: usize = CYC_PEND_VAL + 1;           // 61
const CYC_FIN: usize        = CYC_PEND_CLIMB + TREE_DEPTH;// 93
// ROW_X = CYC_X * CYCLE_LENGTH − 1   (ROW_PK_START = CYC_PK * 8)
```

Verificado: las nueve derivadas coinciden con los nueve literales de
hoy. Con ellas, R2 son los mismos tres nombres (`CYC_ACC`, `CYC_FROZEN`,
`CYC_PEND_CLIMB`) y R3 se **normaliza a UNA convención**: rango =
`(CYC_ARRANQUE..CYC_FIN_DE_TRAMO)`, nivel = `nc − CYC_ARRANQUE`, con la
entrada sombreada por su brazo y la salida excluida por el rango:

- cuentas `(CYC_ACC..CYC_PK)` = (2..34) — idéntico a hoy;
- frozen `(CYC_FROZEN..CYC_PEND_IN)` = (35..59) — hoy (36..60); los
  valores que cambian, 35 y 59, son **inalcanzables** hoy (sombreados,
  §3): traza byte-idéntica;
- pendientes `(CYC_PEND_CLIMB..CYC_FIN)` = (61..93) — hoy (62..94); 61 y
  93 inalcanzables (sombreado / límite del bucle): idéntica.

Tras esto, insertar el salt es **añadir `CYC_SALT = 2` y correr
`CYC_ACC` a `CYC_SALT + 1`**: todo lo demás — filas, rangos, bucles,
periódicas, aserciones, hitos — se re-deriva solo. Y frozen-32 es el
cambio de la profundidad (con la estadificación de §7). Los
ciclos-frontera dejan de tener tres guardianes distintos: el rango se
guarda a sí mismo.

**Plan en pasos compilables (regla §129 — nada de una tacada), cada uno
con suite verde y guarda md5 atómica:**

1. **SB0.1** — introducir el bloque `CYC_*` y redefinir las `ROW_*` como
   derivadas, con `const _: () = assert!(ROW_ROOT == 271);` (y las ocho
   hermanas) como clavos transitorios. Compila, suite 286/0, hitos.
2. **SB0.2** — sustituir los literales 2/35/61 de los seis bucles (tres
   de bits, tres periódicas) por `CYC_ACC`/`CYC_FROZEN`/`CYC_PEND_CLIMB`.
   Suite verde.
3. **SB0.3** — normalizar los tres rangos del `match` a la convención
   única y añadir `debug_assert!(level < profundidad)` en los tres
   `place_*`, para que el próximo desajuste hable claro en vez de
   estallar por índice. Equivalencia: solo cambian valores inalcanzables
   (§3). Suite verde + hitos + `no_constraint_is_vacuous`.
4. **SB0.4** — corregir el comentario muerto del margen, unificar el
   marco de los dos comentarios del pendiente (§4), retirar los clavos
   transitorios, y dejar UNA frase de convención junto al bloque:
   «todo arranque de tramo es un `CYC_*`; ningún literal de ciclo fuera
   de este bloque». Suite verde. Asiento en AUDITORIA.

Después, **SB1 (salt)** vuelve a ser lo que la spec diseñó — el ciclo
nuevo, `link_salt`, las seis `C_SALT_*` de la spec (24 ranuras, cuatro limbos
atados por §138), +4 columnas testigo — pero su mecánica de corrimiento
es una línea. SB2..SB5 del plan del piloto siguen válidos.

### Recomendación

**REFACTOR (SB0), y luego el salt.** No es la opción bonita frente a la
rápida: es que la rápida no existe — frozen-32 obliga al corrimiento que
el hack pretendía evitar, así que el hack pagaría sus 12-20 columnas, su
gate partido y su traza ilegible **además** del corrimiento, no en vez
de él. El refactor es re-expresión pura (byte-idéntica, demostrada
aritméticamente y verificable con la suite en cada paso), deja UNA
fuente de verdad donde había tres-con-guardianes-de-tres-tipos, y es el
patrón que los otros nueve AIR van a necesitar igual en el paso 3. La
cuarta reversión que §139 temía se evita no eligiendo mejor entre dos
maneras de mover tres representaciones, sino dejando de tener tres.

## 9. Qué NO cubre este mapa

La geometría de los otros nueve AIR (cada uno se censa al migrarlo —
burn comparte hitos 7/15/271/279/471 según `doc/air/circuit_burn.md`,
pero «no se asume que lo de send vale para burn sin probarlo allí»,
§138); la decisión de estadificación de `FROZEN_DEPTH` (§7, del autor);
y la ejecución misma de SB0, que es de la sesión del piloto, sobre el
árbol real, con su rito completo.
