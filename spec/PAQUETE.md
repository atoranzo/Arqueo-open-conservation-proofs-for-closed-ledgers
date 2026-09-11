# El paquete de evidencia portable — especificación del sobre y del mando

- **Estado:** normativa vigente desde §397 (RFC-0004, etapa E1)
- **Versión del protocolo:** `zkssl/0.3` — este documento no la mueve: el paquete no cruza el cable
- **Origen:** `spec/rfc/0004-paquete-de-evidencia.md`; asiento que lo sella: §397
- **Implementación de referencia:** el binario `zk-ssl-verify` (`crates/zk-ssl-verify/src/main.rs`)

Este documento es el **único productor normativo** del formato del paquete de evidencia y del
contrato del mando que lo verifica. Hasta §397 vivía en la cabecera del propio binario (líneas
1..90, sha de región `293990fedc785833`); `spec/RPC.md` delegaba ahí por escrito, y esa cabecera
ya había caducado una vez sin que nada lo cazara. Aquí se muda **lo que estaba escrito**, se
añade lo que sólo estaba en el código —el orden de comprobación completo, el catálogo de
rechazos y el contrato del mando— y la cabecera del binario pasa a remitir a este fichero.

**Especifica el SOBRE, no los campos.** Los valores del paquete son respuestas del cable **tal
cual** el nodo las sirve —quien reescribe, adultera—, así que la semántica de cada campo vive en
`spec/RPC.md` y aquí sólo se remite por línea. Duplicarla crearía dos productores del mismo
contrato, que es el defecto que este documento repara.

## 1. Qué es

Lo que sostiene la posición del titular ante un tercero **cuando el operador desaparece o
miente**: las respuestas del cable que ya custodia, reunidas en **un** fichero JSON y verificadas
**sin el nodo, sin la capa y sin el probador** (§243) — sólo el binario y lo publicado. Es el
procedimiento de apagado que `spec/RPC.md` declara en su sección «Apagado — el fin de vida»
(`RPC.md:810-855`): al cerrar, el operador no publica nada que no esté ya publicado; el titular se
lleva lo que ya custodia; y con este paquete lo sostiene después.

El paquete **REPORTA, no juzga.** Dice si la cabeza es de quien dice, si el acuse sube hasta la
raíz firmada y cuántas cofirmas acreditan esa cabeza y ese operador. **Qué testigos valen y
cuántos hacen falta lo decide el CLIENTE** con su política (§319, los mandos `--testigos` y `--k`
del testigo), no el paquete: quien lo arma puede ser el operador, y dejarle elegir su propia `k`
le devolvería justo lo que la cofirma le quita.

## 2. Las seis formas

El binario acepta cinco objetos. Los cinco son JSON; los esqueletos van con puntos suspensivos
donde el valor es una respuesta del cable sin reescribir.

### 2.1 El paquete v1 — la posición

```text
{
  "v": 1,
  "cabeza": { …payload de zkssl_signedEpochHead con available:true… },
  "acuse": {                                  // OPCIONAL
    "seq": "0x…",                             // la entrada del titular
    "hashPrueba": "0x…64hex",                 // digest de SU prueba
    "s": "0x…",                               // de zkssl_ackPath
    "camino": { "siblings": […], "isRight": […] }
  }
}
```

- `cabeza` es el `result` de `zkssl_signedEpochHead` (`RPC.md:433-478`), con `available:true`.
- `acuse` es lo que `zkssl_ackPath` devuelve (`RPC.md:564-735`) más el `hashPrueba` de la entrada
  del titular (el `proofDigest` asentado). Si no viaja, la cabeza sola queda demostrada.

### 2.2 El paquete v2 — las cofirmas dentro (§322)

```text
{
  "v": 2,
  "cabeza":   { …igual que en v1… },
  "acuse":    { …igual que en v1, OPCIONAL… },
  "cofirmas": [ …la respuesta de zkssl_cosigs TAL CUAL, OPCIONAL… ]
}
```

- `cofirmas` es el contenido de `zkssl_cosigs` sin reescribir (`RPC.md:737-779`): cada elemento
  acredita que **un** testigo vio **esta** cabeza de **este** operador, y nada más.
- **El binario lee v1 y v2**: lo custodiado no caduca (§290). Lo que la subida de versión compra
  es que un binario viejo se niegue en voz alta ante un v2 en vez de ignorar las cofirmas e
  imprimir VERDE: *un campo que nadie mira es peor que un campo que falta*. Por eso **un v1 que
  traiga `cofirmas` se RECHAZA** — subir la versión es exactamente lo que las hace parte del
  contrato.
- Dos convenciones de versión conviven, y conviene saberlo: la del PAQUETE (`v`) es un número
  JSON desnudo; la de cada COFIRMA (`v`) viaja como `Q`, cadena hex con `0x`, porque llega del
  cable tal cual (`RPC.md:39-48`). No se unifican: reescribir es adulterar.

### 2.3 El paquete de extensión (§293)

```text
{ "v": 1, "tipo": "extension", "vieja": {…}, "nueva": {…}, "camino": […] }
```

- `vieja` y `nueva` son dos cabezas **v3** firmadas, cada una el `result` de
  `zkssl_signedEpochHead`; `camino` es la prueba de consistencia entre sus cimas
  (`RPC.md:781-808`). Quien custodia la vieja comprueba que la nueva la **extiende**, con el MMR
  de cabezas (§291) como juez, sin el registro y sin el nodo.
- La forma se elige por `tipo`: si vale `"extension"`, el sobre es este; si no, es el de posición.
  `v` se comprueba antes en los dos casos.

### 2.4 El paquete de consumo (§419)

```text
{ "v": 1, "tipo": "consumo", "vieja": {…}, "nueva": {…}, "camino": […],
  "consumo": "0x…", "ausencia": {siblings, isRight}, "presencia": {siblings, isRight} }
```

- Superconjunto estricto del de extensión: **mismas dos cabezas y misma prueba de
  consistencia**, y por eso el «antes» es de ESTA historia y no de una bifurcación firmada.
  Lo que añade son los dos caminos del árbol de consumos: `presencia` sube el digest del
  consumo hasta el `consRoot` de la **nueva**, `ausencia` sube la **hoja vacía** hasta el de
  la **vieja**. Las dos llevan `consRoot` (**v4 o v5**): una v2 o v3 no lo lleva.
- ⚠️ **La posición no viaja: se DERIVA.** El mando calcula `posicion_de_consumo(consumo)` y
  **cruza** sus bits contra el `isRight` recibido en los dos caminos. Sin ese cruce,
  `ausencia` sólo probaría que *alguna* posición está vacía bajo esa raíz, y quien empaqueta
  elegiría cualquiera de las 2^63 libres para «probar» la ausencia de cualquier cosa. La
  mitad de presencia sí es sólida sin el cruce: no se fabrican hermanos que suban a una
  raíz real. Las reglas viven en `crates/zk-ssl-verify/src/consumos.rs` (`spec/NUCLEO.md`).
### 2.5 El paquete de conflicto (§430)

```text
{ "v": 1, "tipo": "conflicto", "consumo": "0x…",
  "libros": [ { "cabeza": {…}, "presencia": {siblings, isRight} },
              { "cabeza": {…}, "presencia": {siblings, isRight} } ] }
```

- **No es superconjunto del de consumo: es otra afirmación.** Aquel prueba que un consumo se
  publicó ENTRE dos cabezas de UN firmante, y para eso necesita la consistencia del MMR y la
  `ausencia`. Éste prueba que el MISMO consumo está bajo el `consRoot` de **dos cabezas de dos
  firmantes DISTINTOS**, y entre dos operadores no hay historia común que extender: no lleva
  `camino` ni `ausencia`, y las dos cabezas van en una **lista `libros` de exactamente dos**.
- ⚠️ **La lista no las ordena, porque la prueba no las ordena.** `vieja`/`nueva` o `a`/`b`
  inventarían una precedencia que aquí no existe: los dos libros son simétricos y ninguno de
  los dos es «el primero». Un tamaño distinto de dos se rechaza con nombre.
- **La regla de las claves va al revés que en las otras formas.** Donde la extensión y el
  consumo exigen la MISMA `publicKey` —la continuidad es de un firmante—, aquí se exige que
  sean DISTINTAS: dos cabezas del mismo operador no son un conflicto entre libros, y aceptarlas
  haría pasar por conflicto lo que es historia de uno solo.
- ⚠️ **Lo que esto demuestra y lo que no.** Demuestra que dos libros aceptaron el mismo
  consumo: eso es **detección**, y llega después. **No previene nada**, y no dice que la unidad
  consumida sea la misma en los dos: que el identificador signifique lo mismo a los dos lados
  es gobernanza (RFC-0006, D-4), no criptografía. Por eso el `tipo` no se llama «doble-uso».

### 2.6 El paquete de rechazo (§455)

```text
{ "v": 1, "tipo": "rechazo", "data": {"causa": "…", "campos": {…}, "seq": "0x…"},
  "cabeza": {…}, "parametros": {…} }                         (una causa de los parámetros)
{ "v": 1, "tipo": "rechazo", "data": {…}, "cabeza": {…},
  "presencia": {siblings, isRight} }                          (una causa del consumo)
{ "v": 1, "tipo": "rechazo", "data": {…}, "cabeza": {…}, "recibo": {…} }   (StaleState)
{ "v": 1, "tipo": "rechazo", "data": {…}, "cabeza": {…},
  "lote": [ {…}, {…} ] }                                      (un duplicado de lote)
```

- **Prueba la CAUSA de un rechazo, no el rechazo.** `data` es el objeto `data` que el nodo puso
  en su negativa (`spec/RPC.md`, §454), tal cual; `cabeza`, una respuesta de
  `zkssl_signedEpochHead`; `parametros`, la de `zkssl_params`; `presencia`, el `camino` de
  `zkssl_consumoPath`; `recibo`, los `publicInputs` que el titular envió; `lote`, las `ops` de
  `zkssl_applyMany`. Que el nodo rechazó —y cuándo— no lo prueba este sobre. Lo que prueba es
  que la regla que el nodo nombró **se sostiene sobre el estado que una cabeza firmada
  compromete**; si no se sostiene, el ROJO nombra por qué, y el sobre es entonces la prueba de que
  la regla era un disfraz.
- **Qué cabeza sirve depende de la causa, y se exige con el `seq`.** Para lo que sólo crece
  —`nextIndex`, los consumos publicados— sirve una cabeza **anterior** al rechazo, o la misma: lo
  que ya estaba en ella seguía estando al juzgar. Para lo que no tiene setter —el límite
  regulatorio— sirve cualquiera del libro. Y para lo que se juzga sobre un estado **instantáneo**
  —las raíces de un `seq` (`StaleState`), el lote contra ese registro (los duplicados)— la cabeza
  tiene que ser **la misma** del rechazo. La mutabilidad está medida en el código (§455, §456).
- Las causas que este mando prueba (RFC-0007 E3a):

| causa | material | qué comprueba | cabeza |
|---|---|---|---|
| `OverRegulatoryLimit` | `parametros` | los siete recomponen el `paramsDigest`; `limit` es el comprometido y `requested` lo supera | v5, cualquiera del libro |
| `AccountLimitReached` | `parametros` | recomponen; `limit` es `maxAccounts` y `nextIndex` lo alcanza | v5, anterior o la misma |
| `ConsumoRepetido` | `presencia` | la posición se DERIVA del consumo, y el consumo está bajo `consRoot` | v4 o v5, anterior o la misma |
| `ConsumoColision` | `presencia` | el ocupante está en la posición DERIVADA del consumo, y no es él | v4 o v5, anterior o la misma |
| `StaleState` | `recibo` | una de las tres raíces que el recibo declaró (`rootOld`, `pendingRootOld`, `frozenRoot`) no es la de la cabeza | v5, la misma (§456) |
| `WrongRegulatoryLimit` | `parametros` | los siete recomponen; `expected` es el comprometido y `declared` no lo es | v5, cualquiera del libro (§456) |
| `DuplicateAccountInBatch` | `lote` | el primer choque del lote, con la regla de `apply_many`, es una cuenta repetida | v5, la misma (§456) |
| `DuplicatePendingInBatch` | `lote` | el primer choque del lote es una posición repetida | v5, la misma (§456) |

- Cualquier otra causa se rechaza con su nombre. El resto de la tabla D-D del RFC-0007 es E3b
  (`AccountFrozen`, `AccountNotFound`: congelación y ausencia, con dos métodos nuevos del cable),
  E4 y E5. `StaleState`, `WrongRegulatoryLimit` y los duplicados de lote los añadió el §456,
  reuniendo un recibo real por el proxy de un banco (su banco no vive en el árbol, y se declara).

## 3. El sobre — lo que el binario lee

El binario lee **31 nombres** distintos del JSON. Los 14 primeros son el sobre propiamente dicho;
los demás son campos de las respuestas del cable que el binario necesita para recomponer y
verificar, y cuyo significado está en `spec/RPC.md`.

| objeto | claves que el binario lee | dónde está su semántica |
|---|---|---|
| sobre | `v`, `tipo`, `cabeza`, `acuse`, `cofirmas`, `vieja`, `nueva`, `camino` | este documento, sección 2 |
| `cabeza` (y `vieja`/`nueva`) | `available`, `formatVersion`, `seq`, `n`, `accountsRoot`, `pendingRoot`, `frozenRoot`, `chainDigest`, `acusesRoot`, `epochDigest`, `publicKey`, `signature`, `index`; en v3 y v4 `mmrRoot`, `mmrSize`; y en v4 `consRoot`, `consCount` | `zkssl_signedEpochHead`, `RPC.md:433-478` |
| `acuse` | `hashPrueba`, `seq`, `camino` → `siblings`, `isRight` | `zkssl_ackPath`, `RPC.md:564-735` |
| cada cofirma | `v`, `epochDigest`, `clavePublicaOperador`, `clavePublicaTestigo`, `firma`, `versionFormato`, `indice` | `zkssl_cosigs`, `RPC.md:737-779` |
| extensión | `camino` (lista de digests) | `RPC.md:781-808` |
| consumo | `consumo`, y `presencia`/`ausencia` → `siblings`, `isRight` | `zkssl_consumoPath`, `RPC.md` |
| conflicto | `consumo`, y `libros[]` → `cabeza`, `presencia` → `siblings`, `isRight` | este documento, sección 2.5 |
| rechazo | `data` → `causa`, `campos`, `seq`; `parametros` → los siete de `zkssl_params`; `presencia` → `siblings`, `isRight` | este documento, sección 2.6 |

⚠️ **§419 — el «31» de arriba ya no es la cuenta**: el sobre de consumo añade `consumo`,
`presencia` y `ausencia`. **No se sustituye por otro número**, porque el 31 no tiene
productor localizable: un censo de literales del fuente da 30 —es ciego a las claves que se
leen por variable— y la tabla de esta misma sección da 33. Una cifra sin universo se
declara, no se inventa; queda para el corte que le encuentre uno.

**§451 — la cabeza v5.** Una `cabeza` con `formatVersion` 5 lleva además `paramsDigest`,
`pmetaRoot`, `nextPending`, `nextIndex` y `totalSupply` (RFC-0007 E1, D-B), que el binario lee
con los mismos lectores y exige los cinco; la fila de `cabeza` de arriba se lee con ese añadido.

Cantidades en convención `Q` (`0x` + hex, u64); digests como `0x` + 64 hex; firmas y claves como
`0x` + hex. Un valor que no tenga esa forma se rechaza **antes** de tocar la criptografía (sección 5).

## 4. Lo que se comprueba, en orden

El orden importa: lo barato y lo estructural va antes, y **el digest nunca se cree, se
recompone**. Cada paso que pasa imprime una línea en la salida estándar.

**Paquete de posición (v1 y v2):**

0. el fichero se lee y es JSON; `v` es 1 o 2; un v1 no trae `cofirmas`; el sobre **no lleva
   `tipo`** —un `tipo` presente y distinto de `extension` o `consumo` se **rechaza con su
   nombre**, nunca
   se lee como paquete de posición (§418)—;
1. **`1/3`** — `cabeza` existe y es `available:true`; `formatVersion` es 2, 3, 4 o 5; **la
   versión elige recomponedor**: v2 con la pareja de acuses (§275), v3 además con la del MMR
   (§292), v4 además con la raíz y la cuenta de consumos (RFC-0006 E2a, §414), v5 además con la
   familia del estado comprometido (RFC-0007 E1a, §451); los campos de la cabeza recomponen su
   `epochDigest`;
2. **`2/3`** — la firma XMSS verifica contra `publicKey` **y** el preámbulo recuperado es el
   esperado (verificar sin comparar no prueba nada) **y** el `index` declarado queda por
   encima del índice de hoja que va dentro de la firma (§399; la cota es por abajo, sección 8);
3. **`3/3`** — si hay `acuse`: la hoja `hoja_de_acuse(hashPrueba, seq, n)` sube por `camino`
   hasta `acusesRoot`, y los campos vuelven a componer el digest firmado (v2), el digest y la
   cima (v3), además la raíz y la cuenta de consumos (v4), o además la familia de v5 (v5). Si no
   hay acuse, la cabeza sola queda demostrada;
4. **cofirmas** (sólo v2) — cada una, antes de tocar la criptografía, nombra **esta** cabeza y
   **este** operador; después su firma verifica. Se imprime cuántas verifican; cuántas hacen
   falta no es asunto del paquete.

**Paquete de extensión:** `1/3` las dos cabezas (v3, v4 o v5) recomponen su digest y sus firmas verifican ·
`2/3` misma `publicKey` en las dos: la continuidad es de **un** firmante · `3/3` la cima nueva
extiende a la vieja por `camino`.

**Paquete de consumo:** `1/5` las dos cabezas recomponen su digest y sus firmas verifican ·
`2/5` misma `publicKey` y las dos llevan `consRoot` (**v4 o v5**) a los dos lados · `3/5`
la cima nueva extiende a la vieja · `4/5` los dos caminos son los de la posición **derivada**
del consumo, no de la que el sobre diga · `5/5` el consumo está bajo el `consRoot` de la
nueva y **no estaba** bajo el de la vieja.

**Paquete de rechazo:** `1/3` la cabeza recompone su digest y su firma verifica · `2/3` el
material de la causa: los `parametros` recomponen el `paramsDigest` de una cabeza **v5**, o el
camino es el de la posición **derivada** del consumo bajo una **v4 o v5** · `3/3` la causa se
sostiene sobre lo comprometido, con la cabeza **anterior** al rechazo —o la misma— cuando cita
algo que sólo crece.

Cabezas **v2, v3, v4 y v5** (`formatVersion`): una cabeza v2 custodiada **sigue verificando** — el
apagado de §290 no caduca. Una cabeza v1 se verifica con la biblioteca, no con este mando.

## 5. Catálogo de rechazos

Cualquier rechazo para el binario con **el primer fallo, con nombre**, en la salida de error, y
sale con código 1 (sección 6). No hay pánico por paquete mal formado: cada lectura de campo falla
cerrada. El catálogo es la lista de mensajes que el binario puede emitir, con sus huecos entre
llaves; un lector que vea uno de estos textos sabe qué regla ha caído. **La verdad se mide en el
fuente**: al sellar, el censo de llamadas se re-deriva por llamada (no por línea: seis textos van
partidos en el fuente) y cada texto tiene que estar aquí.

**Lectura del fichero y versión del sobre**

- `no se puede leer {ruta}: {e}`
- `JSON ilegible: {e}`
- `el paquete no declara su version en `v``
- `el paquete declara v:{v_paquete} — este binario lee v1 y v2`
- `un paquete v1 con `cofirmas`: subir la version es lo que las hace parte del contrato — declaralo v2, o quitalas`
- `tipo desconocido: {otro} - se lee un paquete de posicion (sin `tipo`), `tipo: "extension"`, `tipo: "consumo"`, `tipo: "conflicto"` o `tipo: "rechazo"``

**Forma de los valores** (`hex_a_bytes`, `digest_de`, `u64_de`; `{campo}` es la clave que se leía)

- `sin 0x: {s:.18}` · `hex impar ({} chars)` · `hex: {e}`
- `falta {campo} o no es cadena` · `{campo}: {} bytes, se esperaban 32` · `{campo}: {e:?}`
- `falta {campo} o no es cadena 0x` · `{campo} sin 0x` · `{campo}: {e}`

**La cabeza** (paso 1 y 2)

- `falta cabeza`
- `la cabeza empaquetada no era available:true`
- `formatVersion {version}: el paquete v1 empaqueta cabezas v2, v3, v4 o v5 (la pareja acusesRoot/n viaja firmada desde §275; la del MMR, desde §292)` — el texto dice «v1» aunque el sobre sea v2: regla vigente, prosa que la implementación de referencia debe corregir sin cambiar la regla.
- `los siete campos NO recomponen el epochDigest empaquetado: o el paquete esta adulterado o la cabeza nunca fue esa`
- `falta publicKey` · `falta signature`
- `cabeza: {e}` — la firma no verifica, el preámbulo no es el esperado, o el `index` declarado no queda por encima del que va dentro de la firma (§399); `{e}` es el error de la biblioteca.

**El acuse** (paso 3)

- `acuse sin camino` · `camino sin siblings` · `camino sin isRight`
- `sibling {i} no es cadena` · `sibling {i}: {} bytes` · `sibling {i}: {e:?}`
- `isRight no booleano`
- `acuse: {e:?}` — la hoja no sube hasta la raíz firmada, en v2 o en v3 (dos sitios, un texto).

**Las cofirmas** (paso 4, sólo v2)

- `cofirmas no es una lista`
- `cofirma {n}: falta {campo}`
- `cofirma {n}: version {cv} desconocida, este binario lee hasta la {}` — el tope es `COFIRMA_V_MAX` de la biblioteca.
- `cofirma {n}: acredita OTRA cabeza, no la empaquetada`
- `cofirma {n}: acredita a OTRO operador, no al que firmo la cabeza`
- `cofirma {n}: {e}` — la firma del testigo no verifica.
- Una lista vacía o ausente no es un error: el paquete v2 imprime que la cabeza queda sola.

**La extensión** (`{cual}` es `vieja` o `nueva`)

- `falta vieja` · `falta nueva`
- `{cual}: la cabeza no era available:true`
- `{cual}: formatVersion {version} — la extension exige cabezas v3, v4 o v5: una v2 no lleva la pareja del MMR que extender`
- `{cual}: los campos NO recomponen su epochDigest — adulterada o inventada`
- `{cual}: falta publicKey` · `{cual}: falta signature`
- `{cual}: cabeza: {e}`
- `las cabezas llevan claves DISTINTAS: la continuidad es de UN firmante`
- `falta camino (lista de digests)`
- `camino[{i}] no es cadena` · `camino[{i}]: {} bytes` · `camino[{i}]: {e:?}`
- `la nueva (t={t_n}) NO extiende a la vieja (t={t_v}): historia bifurcada, recortada, o camino que no es el suyo`

**El consumo** (`{cual}` es `presencia` o `ausencia`)

- `el sobre de consumo exige cabezas v4 o v5: una v2 o v3 no lleva consRoot contra el que comprobar`
- `falta {cual} (camino del consumo)` · `{cual}: falta siblings` · `{cual}: falta isRight`
- `{cual}: siblings[{i}] no es cadena` · `{cual}: siblings[{i}]: {} bytes` · `{cual}: siblings[{i}]: {e:?}`
- `{cual}: isRight[{i}] no es booleano`
- `{cual}: el isRight recibido NO es el de la posicion {pos} que el consumo DERIVA - un camino de otra posicion no prueba nada de este consumo`
- `{cual}: el camino no tiene los 63 niveles del arbol de consumos`
- `presencia: el camino NO sube al consRoot de la nueva`
- `ausencia: la hoja vacia NO sube al consRoot de la vieja - el consumo YA estaba`
**El conflicto** (`{i}` es la posición en `libros`; `{cual}` sale `libro[0]` o `libro[1]`)

Estos textos NACEN con esta forma. Todo lo demás que un sobre de conflicto puede emitir sale de
los **mismos productores** que ya están arriba, con su hueco relleno distinto —la cabeza, la
versión v4, los campos del camino y su descuadre—: **un hueco relleno de otra manera no es una
entrada nueva del catálogo**, es el mismo texto y el mismo productor.

- `falta libros` · `libros no es una lista`
- `el sobre de conflicto exige DOS libros: se recibieron {n}`
- `las cabezas llevan la MISMA clave: un conflicto es entre DOS firmantes`
- `libro[{i}]: el camino NO sube al consRoot de su cabeza`

**El rechazo** (`{causa}` es la de `data`; `{que}` sale `limite` o `tope`)

Estos textos NACEN con esta forma. La cabeza, la versión de los consumos (`el sobre de rechazo
exige cabezas v4 o v5: …`), los campos del camino, su descuadre y el cruce de posición salen de
los **mismos productores** de arriba, con su hueco relleno distinto.

- `falta data (el objeto del rechazo)` · `data: falta causa` · `data: falta campos`
- `data: la causa {otra} no la prueba este mando (spec/PAQUETE.md, seccion 2.6)`
- `la causa {causa} exige una cabeza v5: sus parametros viajan en paramsDigest`
- `falta parametros (zkssl_params)`
- `parametros: NO recomponen el paramsDigest de la cabeza - no son los de este libro`
- `data: el {que} que el nodo dice ({dicho}) no es el comprometido ({comprometido})`
- `la causa NO se sostiene: el importe pedido ({pedido}) no supera el limite ({limite})`
- `la causa NO se sostiene: nextIndex ({n}) no alcanza el tope de cuentas ({tope})`
- `la causa NO se sostiene: ocupante y consumo son el MISMO - eso seria ConsumoRepetido`
- `la causa NO se sostiene: el ocupante vive en la posicion {po}, no en la {pos} del consumo`
- `presencia: el camino NO sube al consRoot de la cabeza`
- `la cabeza (seq {s_cabeza}) es POSTERIOR al rechazo (seq {s_rechazo}): {porque}, y solo una cabeza anterior lo prueba`
- `falta recibo (los publicInputs del rechazado)` · `StaleState no lleva campos: su material es el recibo`
- `la causa NO se sostiene: las tres raices del recibo son las de la cabeza - ese estado NO estaba atras`
- `la causa {causa} exige una cabeza v5: sus parametros viajan en paramsDigest` (también `WrongRegulatoryLimit`)
- `la causa NO se sostiene: el limite declarado ({declarado}) ES el comprometido ({limite})`
- `falta lote (las ops de applyMany)` · `lote[{i}]: falta kind` · `lote[{i}]: kind desconocido: {otro}`
- `lote[{i}]: falta receipt.notice` · `lote[{i}]: falta notice`
- `la causa NO se sostiene: el lote no tiene cuentas ni posiciones repetidas`
- `la causa NO se sostiene: el primer choque del lote es {n}, no {causa}`
- `data: el {clave} que el nodo dice ({dicho}) no es el del primer choque del lote ({v})`
- `la cabeza (seq {s_cabeza}) no es la del rechazo (seq {s_rechazo}): esta causa se juzga sobre un estado instantaneo, no sobre lo que crece`

## 6. El contrato del mando

- **Invocación:** `zk-ssl-verify <paquete.json>` — **un** argumento, la ruta del fichero. Es la
  única lectura de disco del binario; no hay red, ni reloj, ni telemetría (§395 lo gatea).
- **Salida estándar:** las líneas `1/3` · `2/3` · `3/3` (o `3/3 sin acuse en el paquete: la cabeza
  sola queda demostrada`), la de cofirmas cuando el sobre es v2, y al final
  `VERDE: el paquete se sostiene sin el nodo` o `VERDE: la extension se sostiene sin el nodo`.
- **Salida de error:** `ROJO: {motivo}` con un texto del catálogo de la sección 5, y para.
- **Tres códigos de salida:** `0` verde · `1` el primer fallo con nombre · `2` uso (ningún
  argumento, o más de uno; imprime el uso en la salida de error).

## 7. Quién arma el paquete

Hoy **ningún mando del árbol emite el paquete**: el testigo y el cli sirven y custodian las
respuestas del cable, y quien las reúne en el sobre es el titular — en el árbol, los bancos
`tools/banco_apagado.sh`, `tools/banco_completo.sh`, `tools/banco_evidencia_v2.sh` y
`tools/banco_extension.sh`, que capturan las respuestas de un nodo real y las envuelven sin
reescribir un campo. Ese es el contrato: **reunir, no recomponer**. Un mando que arme el paquete
es un frente propio y no cambia este documento: cambiaría quién escribe el sobre, no el sobre.

## 8. Lo que este documento NO afirma

- Que exista un verificador independiente **no hace las firmas oponibles**: sigue faltando la
  custodia declarada de la clave del operador (`SECURITY.md`). Esto hace posible verificar; no
  hace válido lo verificado.
- Que las cofirmas verifiquen no dice que basten: la política es del cliente (sección 1).
- Nada sobre el contenido de la posición: el paquete demuestra **que** la entrada está acusada
  bajo esa cabeza firmada, no **qué** dice.
- El `index` de la cabeza sólo está acotado **por abajo**. La firma acredita el índice de hoja
  que lleva dentro y el binario exige que el declarado sea mayor (§399); un sobre que declare
  más de lo que firmó no se rechaza, y lo que el mando imprime como «indice de firma» es el
  embebido, no el declarado.

- La ventana en la que el consumo queda demostrado es la que **el propio operador ordenó con
  su MMR**: no es tiempo de reloj, y él elige qué cabezas firma y cuándo. Dentro de un libro
  el uso único es un invariante comprobable; **entre libros este paquete no dice nada** —eso
  es detección, no prevención, y vive en el RFC-0006 E4—.

## 9. Vectores y puerta

Los vectores del paquete viven en `spec/vectors/paquete/` (etapa E2 del RFC-0004, sellada en §398; E3 en §399):
un positivo por forma —v1, v2, v2 sin acuse, v2 con cero cofirmas, extensión— y **un negativo por
cada regla de la sección 5 que se puede producir a partir de un paquete real**, derivados por
mutación de dos capturas de los bancos. Los dos negativos del índice (`rechazo-index-atrasado`,
`rechazo-index-cero`) entraron con su regla en §399.
El fuera-del-conjunto pasó de `rechazo-formatVersion-5` a `rechazo-formatVersion-6` en §451
(RFC-0007 E1a, con la v5 dentro del conjunto): el `-5` sigue listado y cae por otra causa, con el
texto medido.
`MANIFIESTO.txt` dice, por cada fichero, el código de
salida y el texto que el binario tiene que emitir. **El arnés `tools/conformidad.sh <binario>`**
(RFC-0005, E4, §408) corre el manifiesto entero contra cualquier binario que cumpla el contrato
de la sección 6 —el de la referencia o el de una segunda implementación— y dice, entrada a
entrada, si el código de salida y el texto son los del manifiesto; es el único productor de ese
bucle. `tools/canon.sh` lo corre en cada canon sobre el binario de referencia y se pone en rojo
si un solo vector no dice lo que el manifiesto dice, o si aparece un vector sin entrada. Un
nibble adulterado en cualquiera pone el canon en rojo.

Cuatro textos del catálogo **no tienen vector**, y se declaran: `{campo}: {e:?}`,
`sibling {i}: {e:?}`, `camino[{i}]: {e:?}` y `{cual}: siblings[{i}]: {e:?}` exigen 32 bytes que
`digest_from_bytes` rechace, y no se conoce un valor que lo haga. Siguen siendo reglas: lo que
no tienen es testigo en el árbol. **Desde §431 «las cabezas llevan claves DISTINTAS» sí lo
tiene**: el banco de dos libros produce ese sobre con su defecto AISLADO —las dos cabezas son v4,
las dos recomponen su digest y las dos firmas verifican—, y su vector vive en la familia del
consumo, que es la del sobre que lo lleva.
**Y desde §422 los rechazos del sobre de consumo SÍ lo tienen, y desde §431 los del sobre de
conflicto**: `spec/vectors/conflicto/` trae el positivo y un negativo por cada regla producible
de su familia, derivados por mutación de las capturas del banco de dos libros; y
`spec/vectors/consumo/` trae el
positivo y un negativo por cada regla producible de su familia, derivados por mutación de las
capturas del banco del sobre de consumo (RFC-0006, E3), cada uno con su entrada en
`MANIFIESTO.txt`; el mismo arnés los corre en cada canon con otro manifiesto. Los `#[test]` de
`crates/zk-ssl-verify/src/consumos.rs` siguen falsando las reglas puras —la hoja vacía, la
convención y el cruce— sin necesitar firmas.
**Y desde §455 los del sobre de rechazo** (RFC-0007, E3a-1): `spec/vectors/rechazo/` trae un
positivo por causa probada —cuatro— y un negativo por cada regla producible de su familia,
derivados por UNA mutación de las capturas de un nodo real (el PASTE-455-M; su banco no vive
todavía en el árbol, y se declara). Un texto de la familia no tiene vector: «nextIndex no alcanza
el tope» exige unos parámetros que recompongan con un tope por encima de `nextIndex`, y una sola
mutación de lo real no llega a él. **Desde §456 el catálogo cubre también `StaleState`,
`WrongRegulatoryLimit` y los dos duplicados de lote**, con un recibo real capturado por el proxy de
un banco (el ejemplo `e2e` del SDK), un positivo por causa y un negativo por regla producible por
una sola mutación.
Las demostraciones en vivo con nodo son `tools/banco_apagado.sh`, `tools/banco_consumo.sh`
(RFC-0006, E3) y `tools/banco_dos_libros.sh` (E4a): el último levanta DOS nodos con DOS claves
y produce el hecho que E4 existe para detectar.

## 10. Historia

- §289: nace el paquete (formato v1) y su binario; §290: el apagado declarado; §293: el paquete de
  extensión; §322: el v2 con las cofirmas dentro.
- §399 — el `index` declarado se ata al que va dentro de la firma (E3); el mando imprime el embebido; el testigo lee el `index` servido.
- §400 — el RFC-0004 pasa a ACEPTADO: la regla 4 del PROCESO, saldada con medida.
- §401 — el artefacto: `tools/artefacto.sh`, el tarball reproducible y el 3 ter del canon (sección 11).
- §408 — el arnés de conformidad (RFC-0005, E4): `tools/conformidad.sh`, el único productor del bucle
  del manifiesto, consumido por el canon y por `artefacto.sh`, y dentro del tarball (sección 9).
- §419 — el paquete de consumo (RFC-0006, E3b-2, D-15/D-17): dos cabezas v4 con la
  consistencia dentro, y la posición **derivada y cruzada** contra el `isRight` recibido.
- §418 — el `tipo` desconocido se rechaza con su nombre (RFC-0006, E3b, D-12):
  `rechazo-tipo-desconocido.json` deja de caer por `falta cabeza`, que era la regla de otro.
- §423 — el catálogo del consumo viaja DENTRO del artefacto: `montar()` copia `spec/vectors/consumo/`
  y el `--check` corre los DOS manifiestos con el mismo binario (RFC-0006, E3c; §422).
- §425 — los dos catálogos se corren DESDE DENTRO del tarball desempaquetado y sin repo: el `--check`
  lo abre en `target/artefacto/desde-dentro/` y exige el MISMO veredicto que desde el árbol (punto 110).
- §431 — el catálogo del sobre de conflicto (RFC-0006, E4a): `spec/vectors/conflicto/` con el positivo
  y quince negativos, uno por regla producible, y la cuarta estrofa del canon. Los catálogos son TRES.
- §451 — la cabeza v5 en el mando (RFC-0007, E1a): la versión elige el recomponedor v5 con su
  familia; los rechazos por versión citan «v2, v3, v4 o v5»; el fuera-del-conjunto es
  `rechazo-formatVersion-6`; el `-5` sigue listado y cae por otra causa.
- §455 — el sobre de rechazo (RFC-0007, E3a-1): el mando prueba la CAUSA de cuatro rechazos sobre
  el estado comprometido, con una cabeza anterior al rechazo cuando la causa cita algo que sólo
  crece; `spec/vectors/rechazo/` y la quinta estrofa del canon. Los catálogos son CUATRO.
- §456 — el sobre de rechazo prueba también las cuatro causas que exigen un recibo (RFC-0007 E3a-2):
  `StaleState` (una raíz declarada que no es la comprometida), `WrongRegulatoryLimit` (como
  `OverRegulatoryLimit`) y los dos duplicados de lote (la regla de `apply_many`, el primer choque),
  con la cabeza del `seq` exacto donde el estado es instantáneo; el catálogo, desde un recibo real.
- Hasta §397 este contrato vivía en la cabecera de `crates/zk-ssl-verify/src/main.rs` (1..90,
  `293990fedc785833`), que ya confesó una vez (§247) haber declarado su superficie como completa
  sin serlo. §397 lo muda aquí y deja la cabecera remitiendo, sin enumerar.
- Cambiar este documento es cambiar el contrato: entra por RFC (`spec/rfc/PROCESO.md`).

## 11. El artefacto

Lo que un tercero descarga es `arqueo-verify-<versión>-<host>.tar.gz` (§401), y dentro:
`zk-ssl-verify` (el binario), `conformidad.sh` (el arnés de la sección 9, §408), `spec/PAQUETE.md`
(este documento), `spec/vectors/paquete/`, `spec/vectors/consumo/`, `spec/vectors/conflicto/`
y `spec/vectors/rechazo/` (los cuatro manifiestos y sus vectores), `LICENSE-APACHE`, `LICENSE-MIT`, `NOTICE`, `THIRD-PARTY.txt` (las
licencias de todo lo enlazado), `VERSION` (el commit, el toolchain y los flags con que se compiló)
y `SHA256SUMS` (la huella de cada fichero de dentro). Se comprueba con `sha256sum -c SHA256SUMS`,
y el binario contra los cuatro catálogos con `bash conformidad.sh ./zk-ssl-verify`,
`bash conformidad.sh ./zk-ssl-verify spec/vectors/consumo/MANIFIESTO.txt`,
`bash conformidad.sh ./zk-ssl-verify spec/vectors/conflicto/MANIFIESTO.txt` y
`bash conformidad.sh ./zk-ssl-verify spec/vectors/rechazo/MANIFIESTO.txt`: cada entrada dice el
código de salida y el texto.

La huella del binario **no depende de la máquina ni del usuario** —se compila con
`--remap-path-prefix`—, pero sí del toolchain y de `Cargo.lock`: con el `rustc` que `VERSION`
nombra, `bash tools/artefacto.sh` sobre el commit que `VERSION` nombra vuelve a producir el mismo
binario y el mismo tarball, y `tools/canon.sh` comprueba esa propiedad en cada sello (dos
compilaciones en dos rutas, misma huella; dos tarballs, misma huella; los cuatro manifiestos desde el
árbol y, desde §425, otra vez **desde dentro del tarball desempaquetado y sin repo**, con el mismo
veredicto). Lo que el binario exige: x86_64 Linux y una glibc igual o mayor que la que `VERSION`
declara (`glibc_max`); no es estático, y se dice.

Lo que el artefacto NO es: no es una publicación en crates.io (el crate no lleva la spec ni los
vectores) ni prueba nada sobre la clave del operador (sección 8). No lleva los vectores del cable
—su consumidor es el testigo del cli, no este binario, y su propio adaptador está escrito para que
una segunda implementación ponga ahí el suyo— ni los KAT de `spec/vectors/nucleo/`, que no tienen
manifiesto y los ejercita `nucleo_kat.rs`: entregar vectores que el binario entregado no puede
correr sería afirmar más de lo que se demuestra. La release es un fichero con huella, y la huella
vive en el asiento que lo selló, nunca aquí: **el tarball no puede publicar su propia huella**,
porque lleva dentro `commit` y `describe` y sellar los mueve —y poner un tag mueve el `describe`
otra vez—. Por eso una release se hace en este orden: `tag`, producir, subir; y lo que este
documento gatea es la PROPIEDAD, no un número.
