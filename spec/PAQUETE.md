# El paquete de evidencia portable — especificación del sobre y del mando

- **Estado:** normativa vigente desde §397 (RFC-0004, etapa E1)
- **Versión del protocolo:** `zkssl/0.4` — este documento no la mueve: el paquete no cruza el cable
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
(`RPC.md:814-859`): al cerrar, el operador no publica nada que no esté ya publicado; el titular se
lleva lo que ya custodia; y con este paquete lo sostiene después.

El paquete **REPORTA, no juzga.** Dice si la cabeza es de quien dice, si el acuse sube hasta la
raíz firmada y cuántas cofirmas acreditan esa cabeza y ese operador. **Qué testigos valen y
cuántos hacen falta lo decide el CLIENTE** con su política (§319, los mandos `--testigos` y `--k`
del testigo), no el paquete: quien lo arma puede ser el operador, y dejarle elegir su propia `k`
le devolvería justo lo que la cofirma le quita.

## 2. Las trece formas

El binario acepta trece objetos (la undécima, desde el §573; la duodécima, desde el §592; la
decimotercera, desde el §633). Los trece son JSON; los esqueletos van con puntos suspensivos
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

- `cabeza` es el `result` de `zkssl_signedEpochHead` (`RPC.md:437-482`), con `available:true`.
- `acuse` es lo que `zkssl_ackPath` devuelve (`RPC.md:568-739`) más el `hashPrueba` de la entrada
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

- `cofirmas` es el contenido de `zkssl_cosigs` sin reescribir (`RPC.md:741-783`): cada elemento
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
  (`RPC.md:785-812`). Quien custodia la vieja comprueba que la nueva la **extiende**, con el MMR
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
{ "v": 1, "tipo": "rechazo", "data": {…}, "cabeza": {…},
  "congelados": {index, leaf, camino} }                       (AccountFrozen)
{ "v": 1, "tipo": "rechazo", "data": {…}, "cabeza": {…},
  "cuenta": {index, leaf, camino} }                           (AccountNotFound)
{ "v": 1, "tipo": "rechazo", "data": {…}, "cabeza": {…},
  "banda": {s, publicId, requested, prueba} }                 (InsufficientBalance)
{ "v": 1, "tipo": "rechazo", "data": {…}, "cabeza": {…}, "parametros": {…},
  "peticion": {index, amount} }                               (SupplyCapExceeded)
```

- **Prueba la CAUSA de un rechazo, no el rechazo.** `data` es el objeto `data` que el nodo puso
  en su negativa (`spec/RPC.md`, §454), tal cual; `cabeza`, una respuesta de
  `zkssl_signedEpochHead`; `parametros`, la de `zkssl_params`; `presencia`, el `camino` de
  `zkssl_consumoPath`; `recibo`, los `publicInputs` que el titular envió; `lote`, las `ops` de
  `zkssl_applyMany`; `congelados`, la respuesta de `zkssl_frozenPath` (§458); `peticion`, los
  `params` de la emisión rechazada, tal cual los envió el solicitante (§460); `cuenta`, el
  camino de la hoja vacía que el propio nodo escribe con `zk-ssl-node --prueba-rechazo` (§474),
  y `banda`, la prueba de que el saldo no llega al importe, que el mismo modo escribe sin el
  saldo (§478). Que el nodo rechazó —y cuándo— no lo prueba este sobre. Lo que prueba es
  que la regla que el nodo nombró **se sostiene sobre el estado que una cabeza firmada
  compromete**; si no se sostiene, el ROJO nombra por qué, y el sobre es entonces la prueba de que
  la regla era un disfraz.
- **Qué cabeza sirve depende de la causa, y se exige con el `seq`.** Para lo que sólo crece
  —`nextIndex`, los consumos publicados— sirve una cabeza **anterior** al rechazo, o la misma: lo
  que ya estaba en ella seguía estando al juzgar. Para lo que no tiene setter —el límite
  regulatorio— sirve cualquiera del libro. Y para lo que se juzga sobre un estado **instantáneo**
  —las raíces de un `seq` (`StaleState`), el lote contra ese registro (los duplicados), el árbol
  de congelados, que va y vuelve (`AccountFrozen`), el suministro, que sube con cada emisión y
  baja con cada quema (`SupplyCapExceeded`), y el saldo, que baja con cada envío
  (`InsufficientBalance`)— la cabeza tiene que ser **la misma** del rechazo. La mutabilidad está
  medida en el código (§455, §456, §459, §460, §478).
- Las causas que este mando prueba (RFC-0007 E3 —E3a, E3b y el §460— y las dos causas de E5
  que se construyen: `AccountNotFound` desde el §476 e `InsufficientBalance` desde el §478):

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
| `AccountFrozen` | `congelados` | el camino es el de la cuenta que `data` nombra (cruce con sus bits), mide los 32 niveles que fija el núcleo y sube al `frozenRoot`; la hoja no es la vacía | v3, v4 o v5, la misma (§459) |
| `AccountNotFound` | `cuenta` | el camino es el de la cuenta que `data` nombra (cruce con sus bits), mide los 32 niveles que fija el núcleo y sube al `accountsRoot`; la hoja **es** la vacía | v3, v4 o v5, la misma (§476) |
| `InsufficientBalance` | `banda` | `data` no trae `available`; el `requested` de `data` es el de `banda` y no es cero; la `prueba` verifica, con las opciones de la casa y sin el saldo, que el saldo de la cuenta `publicId` está en `[0, requested - 1]` bajo el `accountsRoot` (`zk_ssl_air::banda`) | v3, v4 o v5, la misma (§478) |
| `SupplyCapExceeded` | `parametros`, `peticion` | los siete recomponen; `cap` es el `maxSupply` comprometido; `wouldBe` es el `totalSupply` de la cabeza más el `amount` de la petición (la suma saturada de la capa) y pasa el tope | v5, la misma (§460) |

- Cualquier otra causa se rechaza con su nombre. `SupplyCapExceeded` la añadió el §460: por el
  cable sólo la produce el grifo del sandbox (`dev_fund`), y el importe viaja en `peticion`
  —palabra del solicitante, no del nodo— para que un `wouldBe` inventado no pase.
  `PendingTreeExhausted` se declara sin prueba portable: las reservas de posición no van bajo
  la firma (RFC-0007, corrección del §460). `AccountFrozen` la añadió el §459, con el camino
  que el cable sirve al titular (§458); `AccountNotFound` pasó a E5 (RFC-0007, corrección del
  §459), y el resto de la tabla D-D es E4 y E5. `StaleState`, `WrongRegulatoryLimit` y los
  duplicados de lote los añadió el §456, reuniendo un recibo real por el proxy de un banco (su
  banco no vive en el árbol, y se declara).
- `InsufficientBalance` la añadió el §478, y es la única causa de este sobre que verifica un STARK.
  El `data` del sobre no trae el saldo —el mando rechaza el que traiga `available`—, y desde el
  §538 (RFC-0009 E3b-2) **la prueba de banda tampoco lo publica**: entre el §521 y el §538 abría sus
  filas en claro con el saldo y el `leaf_salt` dentro (medido en §521; el `saldo-insuficiente.json`
  de la 0.3, conservado bajo `spec/vectors/0.3/rechazo/`, lleva el suyo; el de la 0.4, ninguno). El
  `-32000` del cable también se lo manda a quien hizo la petición desde el §454 (RFC-0007,
  corrección del §479). Un importe por encima del techo del campo (`MAX_VALOR`, 2^62 − 1) produce
  la causa y no su prueba: el productor rehúsa nombrándolo.

### 2.7 El paquete de edad (§465)

```text
{ "v": 1, "tipo": "edad", "cabeza": {…},
  "enunciado": {"t": "0x…", "k": "0x…", "emisor": "0x…"},
  "subraices": {"pendientes": "0x…", "meta": "0x…"}, "prueba": "0x…" }
```

- **Prueba la DISTRIBUCIÓN de edades de lo que está en vuelo** (RFC-0007 D-E): sobre las
  posiciones `0..nextPending` de una cabeza **v5**, las vivas con edad `seq - nacido >= t` —y, si
  el sobre nombra `emisor`, sólo las suyas— son **a lo sumo `k`**. La caja vacía es `k = 0`; el
  tope por cuenta, `t = 0`; la concentración, un `emisor` nombrado. Sin `emisor` cuentan todos:
  lo dice su ausencia, porque el índice 0 es una cuenta y no puede ser el centinela.
- `cabeza` es una respuesta de `zkssl_signedEpochHead` tal cual; `subraices` y `prueba` las
  produce quien prueba, y `prueba` son los bytes de la prueba STARK en `0x` + hex. `seq`,
  `nextPending`, `pendingRoot` y `pmetaRoot` salen **sólo de la cabeza**: el sobre no los
  repite. Y `m` —el subárbol `[0, 2^m)` que la prueba cubre— lo **deriva** el mando de
  `nextPending`, el menor que la cubre y al menos 1: un enunciado tiene una sola forma.
- El juez es `zk_ssl_air::verificar_contra_cabeza` (`crates/zk-ssl-air`), que el kit compila
  **sin el probador**: sube las dos subraíces a 32 niveles, las compara con las raíces firmadas
  y verifica la prueba con las `proof_options()` de la casa y sólo con ellas.
- **Lo que NO prueba:** nada sobre importes —el operador no guarda la apertura del compromiso
  (RFC-0007, corrección del §464)—, nada sobre lo que nunca entró en el árbol (H5b) y nada
  sobre otra cabeza que la que firma las raíces. Su positivo de punta a punta, con una cabeza
  real de un nodo, y su catálogo de vectores son de E4b-3: hoy el mando lleva los negativos que
  caen antes de la firma, y el juez sus testigos con pruebas reales (§465).

### 2.8 El paquete de cobro pendiente (§495)

```text
{ "v": 1, "tipo": "cobro_pendiente", "cabeza": {…},
  "enunciado": {"receptor": "0x…", "nacido": "0x…", "inferior": "0x…"},
  "prueba": "0x…" }
```

- **Prueba que un pendiente EXISTE a nombre de alguien, por al menos un importe** (RFC-0008 D-A,
  D-B): bajo la `pendingRoot` de una cabeza **v5** hay una hoja v2 que se abre a `receptor` y a
  un importe en `[inferior, MAX_VALOR]`, y en la MISMA posición del árbol de meta, bajo
  `pmetaRoot`, está la meta con ese `nacido`. Es un enunciado de ESTADO (D-G): no dice quién
  produjo la prueba, ni que el pendiente vaya a cobrarse, ni cuándo caduca.
- `cabeza` es una respuesta de `zkssl_signedEpochHead` tal cual; `prueba` son los bytes de la
  prueba STARK en `0x` + hex. `seq`, `pendingRoot` y `pmetaRoot` salen **sólo de la cabeza**: el
  sobre no los repite (D-J). Y la cota **superior no viaja**: la fija el juez en el techo del
  campo, `MAX_VALOR`, porque un enunciado tiene una sola forma.
- El juez es `zk_ssl_air::cobro_pendiente::verificar_contra_cabeza` (`crates/zk-ssl-air`), que el
  kit compila **sin el probador** (D-K): exige `nacido < seq` —una meta nacida después de la
  cabeza que la firma es una cabeza que miente, y se para con su nombre—, compone el enunciado
  con las dos raíces firmadas y el techo, y verifica la prueba con las `proof_options()` de la
  casa y sólo con ellas. Es el MISMO productor con el que la capa verifica lo que produce.
- **Lo que NO prueba:** nada sobre quién pagó ni sobre el sobre `X`, que es testigo (D-I); nada
  sobre el importe exacto por encima de `inferior`; y nada sobre otra cabeza que la que firma las
  dos raíces. **Ser testigo no es estar oculto, y desde el §538 lo está en lo que la suite del
  RFC-0009 mide**: entre el §521 y el §538 la prueba abría sus filas en claro y publicaba la sal, la
  `X` y el importe exacto (medido en §521); desde el §538 (E3b-2) no publica literal ninguno. Su
  catálogo de vectores es `spec/vectors/pendiente/` (§499, RFC-0008 E4 por el lado del cobro) y el
  banco que lo reproduce en vivo, `tools/banco_pendiente.sh` (§498); la boca que escribe el sobre
  es `zk-ssl-cli prueba-cobro` (§497, D-M): el mando lleva los negativos que caen antes de la
  firma, el juez sus testigos con prueba real en `stark-experiment`, el productor de la capa y el
  test del nodo lo enlazan contra un latido real, la boca reúne las cinco entradas del productor
  sin abrir libro, y el manifiesto pina lo que cada sobre dice.

### 2.9 El paquete de pago en curso (§506)

```text
{ "v": 1, "tipo": "pago_en_curso", "cabeza": {…},
  "enunciado": {"receptor": "0x…", "importe": "0x…", "t": "0x…", "nacido": "0x…"},
  "prueba": "0x…" }
```

- **Prueba que alguien pagó un importe EXACTO a un receptor y que no puede revertirlo antes de
  `t`** (RFC-0008 D-AD, D-AF): bajo la `pendingRoot` de una cabeza **v5** hay una hoja v2 que se
  abre a `receptor` y a `importe`, con el sobre `X = M(refund_id, [delta, 0, 0, 0])` compuesto
  DENTRO del circuito, y en la misma posición del árbol de meta, bajo `pmetaRoot`, la meta con
  ese `nacido`; y `t - nacido <= delta`. Es la otra mitad del 2.8: donde el cobrador afirma «me
  deben al menos `inferior`», el pagador afirma «pagué `importe` exacto y estoy atado hasta `t`».
- **El plazo no va en el enunciado, pero sí en la prueba.** Lo que se prueba es
  `delta >= t - nacido`, nunca `delta`; y entre el §521 y el §538 la prueba abría sus filas en claro
  con `delta` dentro (medido en §521): quien leyera los bytes del sobre leía el `delta` que esta
  línea decía que no viajaba. Desde el §538 (RFC-0009 E3b-2) no sale literal.
- `cabeza` es una respuesta de `zkssl_signedEpochHead` tal cual; `seq`, `pendingRoot` y
  `pmetaRoot` salen **sólo de la cabeza** (D-J), y el juez exige `nacido < seq` antes de tocar la
  prueba. El juez es `zk_ssl_air::pago_en_curso::verificar_contra_cabeza` (§503), el MISMO con el
  que la capa re-verifica lo que produce, y el kit lo compila **sin el probador**.
- **Lo que NO prueba:** nada sobre quién lo cobrará, nada sobre la `sal`, el `refund_id` ni el
  `emisor` —son testigo—, y nada sobre otra cabeza que la que firma las dos raíces. **Ser testigo
  no es estar oculto, y desde el §538 lo está en lo que la suite del RFC-0009 mide**: entre el §521
  y el §538 la prueba abría sus filas en claro y publicaba la sal, el `delta` y el `refund_id`
  (medido en §521); desde el §538 (E3b-2) no publica literal ninguno. Quien lo
  produce es `prueba_de_pago_en_curso` en la capa (§504) con la apertura del pagador y la foto
  que el nodo le sirve con su credencial y `receiverId` (§505, D-AE); **la boca es
  `zk-ssl-cli prueba-pago`** (§507, D-AI), que reúne las cinco entradas del productor sin
  abrir libro y rechaza un retorno que no recomponga el `x` de su aviso antes de pedirle
  nada al nodo; **el banco es `tools/banco_pago.sh`** (§508), que lo reproduce en vivo
  contra un nodo real y de cuyas capturas salieron los vectores; **el catálogo es
  `spec/vectors/pago/`** (§509), la séptima familia del artefacto.
- ⚠️ **El nodo tiene que ser del §505 o posterior**: uno anterior ignora `receiverId` —el método
  nunca rechazó campos de más— y sirve la nada en vez de un error, así que el pagador no podría
  reunir la foto. Lo que dice si un nodo sabe de qué habla es su `spec/openrpc.json`, no la
  versión del protocolo, que no sube.

### 2.10 El paquete de prenda (§520)

```text
{ "v": 1, "tipo": "prenda", "cabeza": {…},
  "enunciado": {"receptor": "0x…", "marca": "0x…"},
  "prueba": "0x…" }
```

- **Prueba que bajo la `pendingRoot` de una cabeza v5 hay un pendiente que sólo puede cobrar
  quien tiene la clave de `receptor`, y que la marca de esa hoja es `marca`** (RFC-0008 D-AV,
  D-AW): dentro del circuito se recompone la hoja v2 con
  `receptor = derive_public_id_wide(clave)`, se sube por el camino hasta la raíz, y se prueba que
  `marca = H(DOMINIO_PRENDA, C2)` con `C2` de testigo. Publicar `C2` haría enlazables todos los
  sobres de una misma hoja; por eso viaja la marca, que es lo que el árbol de consumos lleva.
- **De AUTORIZACIÓN, no de estado**, y es lo que la separa del 2.8 y del 2.9: aquéllos los
  produce cualquiera que tenga la apertura, y ésta sólo quien tiene la clave (D-AV). La clave no
  viaja ni aparece en el sobre: entra en la traza y se va con ella.
- **No lleva la meta** (D-AY): su cabeza aporta UNA raíz, `pendingRoot`, y el enunciado no dice
  `nacido` ni `importe`. Por eso el juez no compara nada contra el `seq`, y por eso el ancho del
  AIR baja de 44 a 42.
- **La cabeza se exige v5** (D-BF), y el texto del rechazo dice SU razón y no la de sus hermanos:
  no «la única que firma pmetaRoot» —que aquí sería falsa—, sino que es la que el nodo sirve
  desde el §452 y contra la que `zkssl_pledge` juzga. `cabeza_v3_verificada` admitiría v3 y v4,
  pero ningún productor de prenda puede emitir contra ellas.
- ⚠️ **Un VERDE aquí es MEDIA prenda.** El juez NO comprueba que `marca` esté bajo el `consRoot`
  de esa misma cabeza —lo declara en su propia doc, «es la puerta de quien escribe en él»— y este
  binario no tiene árbol que mirar. El PAR es la marca bajo la raíz firmada MÁS este sobre
  (D-AS); la otra mitad se pide con `zkssl_consumoPath`, y quien la escribe EXIGIENDO el sobre es
  `zkssl_pledge` (§519). El mando lo imprime en su veredicto, no sólo aquí.
- **Lo que NO prueba:** nada sobre el importe, la sal, `X` ni `C2` —son testigo—, nada sobre si el
  pendiente sigue vivo, y nada sobre otra cabeza que la que firma esa raíz. **Ser testigo no es
  estar oculto, y desde el §538 lo está en lo que la suite del RFC-0009 mide**: entre el §521 y el
  §538 la prueba abría sus filas en claro y en ellas iba la clave de gasto del prendador, 42 veces
  (medido en §521), y quien leyera el sobre la tenía; desde el §538 (E3b-2) no sale literal. El
  juez es `zk_ssl_air::prenda::verificar_contra_cabeza` (§516), el MISMO con el que la capa
  re-verifica lo que produce (§518), y el kit lo compila **sin el probador**. Quien lo produce es
  `prueba_de_prenda` (§518), con el aviso, la clave y el camino que el nodo sirve del último
  latido; y desde el §543 lo escribe su prendador con `zk-ssl-cli prueba-prenda`
  (D-BC..D-BI): la clave entra por el keystore del SDK y el cli no la ve, y con `--publicar`
  la boca pide además a `zkssl_pledge` que escriba la marca bajo ese mismo latido; **el banco es
  `tools/banco_prenda.sh`** (§545), que lo reproduce en vivo contra un nodo real —dos positivos, un
  rechazo en vivo y siete negativos—, y **el catálogo es `spec/vectors/prenda/`** (§546), la octava
  familia del artefacto, COPIADA de esas capturas y no re-derivada.

### 2.11 El sobre de completitud (§573, RFC-0010 E4)

```text
{ "v": 1, "tipo": "completitud",
  "cierre": {…},                                  la cabeza v6 firmada que cierra la era del recibo
  "recepcion": {"rx", "era", "n", "hashPrueba"},  el recibo del cable, tal cual (§571)
  "limiteAnterior": "0x…",                        Q, DECLARADO (D1)
  "camino": {"siblings": […], "isRight": […]},    el de zkssl_recepPath
  "resolucion": {"tipo": "acuse", "cabeza": {…}, "acuse": {"seq", "hashPrueba", "camino"}}
              | {"tipo": "rechazo", "sobre": {…}}      un sobre de la sección 2.6
              | {"tipo": "declarada", "data": {…}}     el `error.data` del rechazo, tal cual
              | {"tipo": "lote",                       el recibo de un zkssl_applyMany (§612)
                 "composicion": [{"hashPrueba", "cuenta", "posicion"}, …],
                 "acuses": [{"cabeza", "acuse"}, …]  | "sobre": {…}  | "data": {…}}
              | {"tipo": "prenda", "sobre": {…},       el recibo de un zkssl_pledge (§613), y el
                                                       sobre de la sección 2.10
                 "consumo": {"cabeza", "camino"}
                 | "respuesta": {…}, "juzgada": {…}  | "rechazo": {…}},
  "vigente": {…} }                                una cabeza del MISMO operador, para la ventana
```

El mando comprueba, en orden, que el `cierre` es una cabeza v6 que recompone y cuya firma
verifica, que la `n` del recibo es la firmada, que `Q < rx <= recepCount`, y que la hoja
`recibo_digest(hashPrueba, era, n)` sube por un camino de `RECEP_DEPTH` niveles con los lados de
la posición `rx − Q − 1` hasta la `recepRoot` del cierre: el operador **recibió** la operación.
Después, el veredicto (RFC-0010, D-F y D-G). La ventana se MIDE siempre con el índice XMSS de una
cabeza firmada por la misma clave, el que la firma lleva embebido (§399), con `S − era <= n` (D2):

1. **`acuse`**: la cabeza está dentro de la ventana, el acuse es de la MISMA prueba y el par
   cabeza+acuse verifica como el paquete de posición de la sección 2.1. VERDE, «resuelta como
   transición aplicada».
2. **`rechazo`**: el sobre verifica por sus propias reglas, su cabeza está dentro de la ventana, y
   su `data.recepcion.hashPrueba` es el del recibo (D3). ⚠️ Esa atadura es la PALABRA del nodo: el
   `error` del cable no va firmado. VERDE, «resuelta como rechazo con prueba».
3. **Sin resolución**, con una cabeza `vigente` fuera de la ventana: **ROJO NOMBRADO**, «NO
   RESUELTA EN LA VENTANA». Es el producto del hito: la promesa firmada por el acusado, la ventana
   aritmética sobre dos cabezas firmadas, y la carga de exhibir la resolución de quien la tiene.
   No es una prueba criptográfica de ausencia (D-F). Con la `vigente` todavía dentro de la
   ventana, el sobre es prematuro y se rechaza con su nombre.
4. **`declarada`**, con una de las causas que el RFC-0007 dejó sin prueba portable
   (`CustodianSetExhausted`, `PendingTreeExhausted`, `NotTheIssuer`, `NotTheAccountHolder`) y atada
   al recibo por su `recepcion`: el **cuarto estado**, «resolución declarada, no probada», que
   sale con su propio código (D4). Una causa que sí tiene prueba se exhibe, no se declara.
5. **`lote`** (§612, RFC-0014 E4a, D-B): el recibo de un `zkssl_applyMany`, cuyo `hashPrueba` es
   la huella de su composición (§611). El mando recompone `hash_del_lote` con la `composicion`
   -la que el agregador reenvía a cada titular (D-C), en el orden del lote- y, si no es la del
   recibo, no sigue. Después, UNA de tres:
   - **`acuses`**, uno por operación y en su orden, cada uno resuelto como en el veredicto 1 y de
     SU prueba: VERDE, «resuelta como LOTE aplicado». El lote se aplica entero o no se aplica, así
     que faltar uno es no resolver.
   - **`sobre`**, el de la sección 2.6 de la operación que el nodo nombra en `data.operacion`,
     resuelto como en el veredicto 2: VERDE, «resuelta como LOTE rechazado con prueba». Las
     compañeras quedan resueltas por ella: el lote es la unidad. ⚠️ Que el rechazo sea de ESA
     operación es la palabra del nodo (D3), como la atadura al recibo.
   - **`data`**, el `error.data` tal cual. Con `DuplicateAccountInBatch` o
     `DuplicatePendingInBatch` el mando REPITE el juicio con la composición sola -la operación
     nombrada lleva la cuenta o la posición de `campos`, y una anterior también-: si lo sostiene,
     VERDE en el acto y sin cabeza, «resuelta como LOTE rechazado por su FORMA»; si no, **ROJO
     NOMBRADO, «RECHAZO SIN FUNDAMENTO»**: donde el verificador puede repetir el juicio del
     operador, lo dice (RFC-0014, decisión 3). ⚠️ El «no» del nodo es su palabra en su `data`
     (D3): si alguien lo inventara, el operador lo desmiente exhibiendo la resolución verdadera.
     Con una de las causas sin prueba portable, el cuarto estado, como en el 4.
6. **`prenda`** (§613, RFC-0014 E4b, D-E): el recibo de un `zkssl_pledge` cuya prueba llegó al
   juez, con `hashPrueba` el digest de esa prueba (§116). El `sobre` de la sección 2.10 se ata a
   él por ESO -su prueba tiene que tener ese digest- y, si no, no sigue. Después, UNA de tres:
   - **`consumo`**, `{cabeza, camino}`: el sobre verifica y su `marca` está bajo el `consRoot` de
     una cabeza del mismo operador dentro de la ventana, por el camino de `zkssl_consumoPath` y
     con los lados de la posición que la marca deriva. Es el PAR entero (D-AS): VERDE, «resuelta
     como PRENDA aceptada».
   - **`respuesta`**, la negativa del nodo tal cual -`accepted: false`, atada al recibo por su
     `recepcion` y SIN `data`-, y **`juzgada`**, la cabeza contra la que el nodo juzgó: la última
     firmada al recibir, de índice `era − 1` (§567), del mismo operador y v5 o posterior. El mando
     REPITE el juicio con el mismo juez y la `pendingRoot` de esa cabeza: si el sobre no verifica,
     VERDE, «resuelta como PRENDA rechazada con prueba», y cualquiera lo comprueba sin el nodo; si
     verifica, **ROJO NOMBRADO, «RECHAZO SIN FUNDAMENTO»** (RFC-0014, decisión 3): un operador
     honrado no puede producirlo, porque el recibo sólo existe si el `seq` casó y entonces juzgó
     esa cabeza con ese juez. ⚠️ La negativa es su palabra (D3): si alguien la inventara, el
     operador la desmiente con el PAR.
   - **`rechazo`**, el sobre de la sección 2.6 de la causa con que la capa rechazó la marca
     (`apply_consumo`, p. ej. `ConsumoColision`): resuelto como el veredicto 2 -desde el §613 el
     `data` de esa negativa lleva su `recepcion`- y con el consumo rechazado igual a la marca del
     sobre. VERDE, «resuelta como PRENDA rechazada por la capa».

**El banco es `tools/banco_completitud.sh`** (§574), que lo reproduce en vivo contra un nodo real
que firma —con `--largo`, hasta que la ventana EXPIRA—, y **el catálogo es
`spec/vectors/completitud/`** (§574), la novena familia del artefacto, COPIADA de una corrida
suya: sección 9. Las formas 5 y 6 tienen su propio banco, `tools/banco_recibo_agregado.sh`
(§614), y sus vectores en la misma familia.

### 2.12 El sobre del ancla (§592, RFC-0012 E3)

```text
{ "v": 1, "tipo": "ancla",
  "cabeza": {…},                        la cabeza firmada (payload de zkssl_signedEpochHead,
                                        available:true, v3 a v6: la pareja del MMR viaja desde ellas)
  "ancla": { "v": 1, "clave": "0x…",    OPCIONAL: el ancla publicada en el medio, tal cual --
             "indice": "0x…",           la huella de la clave, el indice XMSS EMBEBIDO, el digest
             "epochDigest": "0x…",      firmado y la pareja del MMR (RFC-0012, D-A)
             "mmrRoot": "0x…", "mmrSize": "0x…" },
  "camino": ["0x…", …],                 OPCIONAL: la consistencia MMR del lote anclado a la cabeza
                                        (RFC 6962), la lista plana del sobre de extensión
  "contraria": {…} }                    OPCIONAL: una segunda cabeza firmada -- la vista dividida
```

El orden: primero la FORMA —`contraria` y `ancla` no conviven, y un `camino` sin `ancla` no
tiene nada que extender—; después la `cabeza`, ENTERA —recompone su digest, su firma verifica, y
el índice que cuenta es el **EMBEBIDO** en la firma (§399; RFC-0012, D-C)—. Con eso, cuatro
modos:

1. **La cabeza sola**: el mando DERIVA el ancla y su huella
   (`ancla_digest(huella_de_clave(publicKey), indice_embebido, epochDigest, mmrRoot, mmrSize)`,
   `spec/NUCLEO.md` §6) y las imprime. Es el productor de B10.6: lo impreso es lo que se publica
   en el medio. El que comprueba compara la huella con el medio **él mismo**: este binario no
   tiene red y lo dice en su salida. VERDE.
2. **Con `ancla` y sin `camino`**: el ancla ES esta cabeza. Los cinco campos, iguales, cada uno
   con su rechazo nombrado; la `clave` contra la huella de la `publicKey`, el `indice` contra el
   embebido. VERDE.
3. **Con `ancla` y `camino`**: el ancla es ANTERIOR. Misma clave, `indice` estrictamente
   anterior al embebido, y la cima de la cabeza EXTIENDE el lote anclado —`mmrRoot`/`mmrSize`
   del ancla contra los de la cabeza, el juez de consistencia del §291—. El ancla del génesis
   (`mmrSize` 0) no tiene historia que extender y se rechaza con su nombre. VERDE: la historia
   anclada es un prefijo.
4. **Con `contraria`**: la VISTA DIVIDIDA. La contraria se verifica ENTERA como la cabeza;
   misma clave, mismo índice EMBEBIDO, contenidos distintos —dos preámbulos bajo un índice de
   un solo uso, que sólo quien tiene la clave puede producir (§248)—. Sale **DETECCIÓN**, con
   salida 0, el molde del conflicto (2.5): el sobre que la exhibe no falla — delata. Dos
   cabezas con índices embebidos DISTINTOS no dividen ninguna vista y se rechazan con su nombre.

⚠️ Lo que este sobre NO dice: que el ancla estuviera PUBLICADA, ni desde cuándo — eso es del
medio elegido, y la confianza en el medio queda desplazada y declarada (RFC-0012, D-H). El
sobre verifica la criptografía; el orden externo lo da el medio.

**El banco es `tools/banco_ancla.sh`** (§593), que lo reproduce en vivo contra un nodo real que
firma —la vista dividida incluida, reproduciendo el ataque de verdad: la misma semilla con un
contador de índice fresco y otro libro—, y **el catálogo es `spec/vectors/ancla/`** (§593), la
décima familia del artefacto, COPIADA de una corrida suya: sección 9.

### 2.13 El sobre del ancla cofirmada (§633, RFC-0013 E4a)

```text
{ "v": 1, "tipo": "ancla-cofirmada",
  "cabeza": {…},                        la cabeza firmada cuya ancla se publico (como en 2.12)
  "nota": "zkssl/v1/…\n…",              la nota checkpoint del medio, ENTERA, como texto: sus tres
                                        lineas, la linea en blanco y las de firma (RFC-0013 D-A)
  "publicador": "zkssl/v1/…+…+…",       la vkey del publicador del medio (signed-note, tipo 0x06)
  "testigos": ["nombre+…+…", …],        OPCIONAL: las vkeys de los testigos cuyas cofirmas se
                                        reportan
  "posicion": "0x…",                    la posicion del ancla en el arbol del medio
  "inclusion": ["0x…", …] }             el camino RFC 9162 del ancla a la raiz de la nota: hashes
                                        SHA-256 en bytes, NO digests de Goldilocks
```

El ancla **no viaja**: se DERIVA de la cabeza, como en el modo 1 de 2.12, y por eso es la
cabeza firmada que dice ser. El orden: primero la FORMA —`ancla`, `camino` y `contraria` son del
sobre del ancla y aquí se rechazan con su nombre; cada campo que falta, también—; después la
`cabeza`, ENTERA, porque el `origin` de la nota se deriva de su clave; después la nota, las
cofirmas y la inclusión:

1. La cabeza recompone su digest y su firma verifica; se derivan el índice EMBEBIDO, la huella
   de la clave y el ancla.
2. La nota es del medio de ESTA clave —el publicador se llama `zkssl/v1/` y la huella de la
   clave XMSS en hexadecimal (RFC-0013 D-A)— y la línea del publicador verifica con su vkey:
   ML-DSA-44, tipo `0x06`, con las reglas de `signed-note` y `tlog-checkpoint` que declara
   `crates/zk-ssl-medio/src/nota.rs`.
3. Las cofirmas: cada línea de la nota con una vkey del sobre del mismo nombre y `key_id` se
   VERIFICA, y si no verifica es ROJO; las líneas sin vkey en el sobre se cuentan y no se juzgan.
4. La hoja `SHA-256(0x00 || huella del ancla)` sube por la `inclusion` hasta la raíz de la nota
   en la `posicion` declarada.

VERDE: la cabeza estaba publicada en ese medio, en esa posición, y la cofirman los testigos que
el mando lista, cada uno con la huella SHA-256 de su clave ENTERA —el `key_id` de cuatro bytes
es un identificador y se fabrica—.

⚠️ **REPORTA, NO JUZGA** (§633), como el paquete v2 con sus cofirmas: qué testigos valen y
cuántos hacen falta lo decide quien verifica con su política (RFC-0013 D-D), no el sobre, que
puede armarlo el operador. Ni el umbral, ni la frescura de las marcas de tiempo, ni que el medio
sea el único del operador los decide este mando.

**El catálogo y el banco** son del §634.

## 3. El sobre — lo que el binario lee

El binario lee **31 nombres** distintos del JSON. Los 14 primeros son el sobre propiamente dicho;
los demás son campos de las respuestas del cable que el binario necesita para recomponer y
verificar, y cuyo significado está en `spec/RPC.md`.

| objeto | claves que el binario lee | dónde está su semántica |
|---|---|---|
| sobre | `v`, `tipo`, `cabeza`, `acuse`, `cofirmas`, `vieja`, `nueva`, `camino` | este documento, sección 2 |
| `cabeza` (y `vieja`/`nueva`) | `available`, `formatVersion`, `seq`, `n`, `accountsRoot`, `pendingRoot`, `frozenRoot`, `chainDigest`, `acusesRoot`, `epochDigest`, `publicKey`, `signature`, `index`; en v3 y v4 `mmrRoot`, `mmrSize`; y en v4 `consRoot`, `consCount` | `zkssl_signedEpochHead`, `RPC.md:437-482` |
| `acuse` | `hashPrueba`, `seq`, `camino` → `siblings`, `isRight` | `zkssl_ackPath`, `RPC.md:568-739` |
| cada cofirma | `v`, `epochDigest`, `clavePublicaOperador`, `clavePublicaTestigo`, `firma`, `versionFormato`, `indice` | `zkssl_cosigs`, `RPC.md:741-783` |
| extensión | `camino` (lista de digests) | `RPC.md:785-812` |
| consumo | `consumo`, y `presencia`/`ausencia` → `siblings`, `isRight` | `zkssl_consumoPath`, `RPC.md` |
| conflicto | `consumo`, y `libros[]` → `cabeza`, `presencia` → `siblings`, `isRight` | este documento, sección 2.5 |
| rechazo | `data` → `causa`, `campos`, `seq`; `parametros` → los siete de `zkssl_params`; `presencia` → `siblings`, `isRight`; `recibo` → `rootOld`, `pendingRootOld`, `frozenRoot`; `lote[]` → `kind`, `sender` o `receiver`, `receipt` → `notice` → `position` o `notice` → `position`; `congelados` → `index`, `leaf`, `camino` → `siblings`, `isRight`; `peticion` → `amount`; `cuenta` → `index`, `leaf`, `camino` → `siblings`, `isRight`; `banda` → `s`, `publicId`, `requested`, `prueba` | este documento, sección 2.6 |
| edad | `enunciado` → `t`, `k`, `emisor`; `subraices` → `pendientes`, `meta`; `prueba` | este documento, sección 2.7 |
| cobro pendiente | `enunciado` → `receptor`, `nacido`, `inferior`; `prueba` | este documento, sección 2.8 |
| pago en curso | `enunciado` → `receptor`, `importe`, `t`, `nacido`; `prueba` | este documento, sección 2.9 |
| ancla cofirmada | `nota`, `publicador`, `testigos`, `posicion`, `inclusion`, y la `cabeza` de 2.12 | este documento, sección 2.13 (§633) |

⚠️ **§419 — el «31» de arriba ya no es la cuenta**: el sobre de consumo añade `consumo`,
`presencia` y `ausencia`. **No se sustituye por otro número**, porque el 31 no tiene
productor localizable: un censo de literales del fuente da 30 —es ciego a las claves que se
leen por variable— y la tabla de esta misma sección da 33. Una cifra sin universo se
declara, no se inventa; queda para el corte que le encuentre uno.

**§451 — la cabeza v5.** Una `cabeza` con `formatVersion` 5 lleva además `paramsDigest`,
`pmetaRoot`, `nextPending`, `nextIndex` y `totalSupply` (RFC-0007 E1, D-B), que el binario lee
con los mismos lectores y exige los cinco; la fila de `cabeza` de arriba se lee con ese añadido.

**§570 — la cabeza v6.** Una `cabeza` con `formatVersion` 6 lleva además `recepRoot` y
`recepCount` (RFC-0010 D-C, E2d), que el binario lee con los mismos lectores y exige los dos,
además de los cinco de la v5; la fila de `cabeza` se lee con los dos añadidos.

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

**Paquete de conflicto:** `1/4` las dos cabezas recomponen su digest y sus firmas verifican ·
`2/4` son de operadores **distintos** y las dos llevan `consRoot` (**v4 o v5**) · `3/4` los
dos caminos son los de la posición **derivada** del consumo · `4/4` el mismo consumo está
bajo el `consRoot` de los **dos** libros.

**Paquete de rechazo:** `1/3` la cabeza recompone su digest y su firma verifica · `2/3` el
material de la causa, el que la sección 2.6 le asigna: los `parametros` recomponen el
`paramsDigest` de una cabeza **v5**; el camino de `presencia` es el de la posición
**derivada** del consumo bajo una **v4 o v5**; el `recibo` declaró sus tres raíces; el `lote`
se lee en orden; el camino de `congelados` o de `cuenta` es el de la cuenta que `data` nombra;
o el enunciado de la `banda` es el de esa cuenta bajo el `accountsRoot` firmado · `3/3` la
causa se sostiene sobre lo comprometido, con la cabeza **anterior** al rechazo —o la misma—
cuando cita algo que sólo crece, y con **la misma** cuando el estado es instantáneo; en la
`banda`, lo que se sostiene es la prueba, que verifica sin el saldo.

**Paquete de edad:** antes de tocar la criptografía, el sobre tiene su forma (`enunciado`,
`subraices`, `prueba`) y la cabeza es **v5** · `1/3` la cabeza recompone su digest y su firma
verifica · `2/3` las dos subraíces, subidas a 32 niveles con la `m` que el mando deriva de
`nextPending`, son `pendingRoot` y `pmetaRoot` · `3/3` la prueba verifica contra el enunciado
que la cabeza fija (`seq`, `nextPending`, `m`) y el sobre afirma (`t`, `k`, `emisor`), con las
opciones de la casa.

**Paquete de cobro pendiente:** antes de tocar la criptografía, el sobre tiene su forma
(`enunciado`, `prueba`) y la cabeza es **v5** · `1/3` la cabeza recompone su digest y su firma
verifica · `2/3` el enunciado lo compone el juez con `pendingRoot`, `pmetaRoot` y el techo del
campo, y el `nacido` que el sobre afirma es anterior al `seq` firmado · `3/3` la prueba verifica
contra ese enunciado, con las opciones de la casa. Las dos últimas las decide el MISMO juez, que
comprueba el `nacido` **antes** de tocar la prueba.

**Sobre del ancla cofirmada** (§633): antes de tocar la criptografía, la forma (sin `ancla`,
`camino` ni `contraria`; con `cabeza`, `nota`, `publicador`, `posicion` e `inclusion`, cada
hash de 32 bytes, y vkeys legibles en `testigos`) · `1/4` la cabeza recompone su digest y su
firma verifica, y su ancla se deriva · `2/4` la nota es del medio de su clave y la firma del
publicador verifica · `3/4` las cofirmas con clave en el sobre verifican, y se listan · `4/4` el
ancla sube hasta la raíz de la nota en su posición.

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
- `tipo desconocido: {otro} - se lee un paquete de posicion (sin `tipo`), `tipo: "extension"`, `tipo: "consumo"`, `tipo: "conflicto"`, `tipo: "rechazo"`, `tipo: "edad"`, `tipo: "cobro_pendiente"`, `tipo: "pago_en_curso"`, `tipo: "prenda"`, `tipo: "completitud"`, `tipo: "ancla"` o `tipo: "ancla-cofirmada"`` — ⚠️ §633: hasta aquí el texto acababa en `prenda`, rancio desde el §573; se pone al día al añadir el brazo, y el test que lo ata busca el último.

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
- `{cual}: falta cabeza`
- `las cabezas llevan la MISMA clave: un conflicto es entre DOS firmantes`
- `libro[{i}]: el camino NO sube al consRoot de su cabeza`

**El rechazo** (`{causa}` es la de `data`; `{que}` sale `limite`, `limite esperado`, `tope` o
`tope de suministro`)

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
- `falta congelados (el camino de zkssl_frozenPath)` · `congelados: falta camino`
- `data: el index que el nodo dice ({dicho}) no es el del camino ({del_camino})`
- `congelados: el isRight recibido NO es el de la cuenta {i} - un camino de otra cuenta no prueba nada de esta`
- `congelados: el camino no tiene los 32 niveles del arbol de congelados`
- `congelados: el camino NO sube al frozenRoot de la cabeza`
- `la causa NO se sostiene: la hoja de la cuenta {i} bajo el frozenRoot es la vacia - no estaba congelada`
- `falta cuenta (el camino del arbol de cuentas)` · `cuenta: falta camino`
- `cuenta: el isRight recibido NO es el de la cuenta {i} - un camino de otra cuenta no prueba nada de esta`
- `cuenta: el camino no tiene los 32 niveles del arbol de cuentas`
- `cuenta: el camino NO sube al accountsRoot de la cabeza`
- `la causa NO se sostiene: la hoja de la cuenta {i} bajo el accountsRoot NO es la vacia - la cuenta existe`
- `falta peticion (los params de la emision rechazada)`
- `data: el wouldBe que el nodo dice ({dice}) no es el suministro de la cabeza mas el importe pedido ({suministro} + {importe} = {seria})`
- `la causa NO se sostiene: el suministro resultante ({seria}) no supera el tope ({tope})`
- `data: esta causa no publica el saldo: la banda lo prueba sin el` · `falta banda (la prueba de la desigualdad)`
- `data: el importe que el nodo dice no es el que la prueba acota` · `la causa NO se sostiene: pedir 0 no puede pasar de ningun saldo`
- `banda: falta prueba o no es cadena 0x` · `banda: {e}`, donde `{e}` es el rojo de `zk_ssl_air::banda::verificar`: `los limites {l} y {u} pasan del techo {MAX_VALOR}`, `banda vacia: inferior {l} sobre superior {u}`, `la prueba no se deserializa: …`, `forma de traza …`, o el que escribe winterfell (§478)

**La edad** (§465; `{e}` sale de `zk_ssl_air::verificar_contra_cabeza`)

Estos textos NACEN con esta forma. La forma de los valores y la cabeza —su recomposición, su
firma y la familia de v5— salen de los **mismos productores** de arriba.

- `falta enunciado` · `falta subraices` · `falta prueba o no es cadena 0x`
- `formatVersion {version}: la prueba de edad exige una cabeza v5, la unica que firma pmetaRoot y nextPending`
- `edad: {e}`, con `{e}` uno de estos:
  - `la subraiz de pendientes, subida a 32 niveles, no es el pendingRoot de la cabeza`
  - `la subraiz de meta, subida a 32 niveles, no es el pmetaRoot de la cabeza`
  - los del enunciado: `m = {} fuera de 1..=24` · `n = {} no cabe en 2^{}` · `T = {} o seq = {} no caben en {BITS} bits` · `k = {} mayor que n = {}`
  - `la prueba no se deserializa: {e:?}` · `forma de traza {forma:?}; el enunciado pide {:?}` · `{e:?}`, el error de `winter-verifier`

**El cobro pendiente** (§495; `{e}` sale de `zk_ssl_air::cobro_pendiente`)

Estos textos NACEN con esta forma. La forma de los valores y la cabeza —su recomposición, su
firma y la familia de v5— salen de los **mismos productores** de arriba, y `falta enunciado`,
`falta prueba o no es cadena 0x` y `falta cabeza` son los de la edad, letra por letra.

- `formatVersion {version}: el cobro pendiente exige una cabeza v5, la unica que firma pmetaRoot`
- `cobro: {e}`, con `{e}` uno de estos:
  - `nacido {} no es anterior a la cabeza de seq {}: una meta nacida despues de la cabeza que la firma`
  - los del enunciado: `las cotas {l} y {u} pasan del techo {MAX_VALOR}` · `banda vacia: inferior {l} sobre superior {u}`
  - `la prueba no se deserializa: {e:?}` · `forma de traza {forma:?}; el enunciado pide {:?}` · `{e:?}`, el error de `winter-verifier`

**La prenda** (§520; `{e}` sale de `zk_ssl_air::prenda`)

- `formatVersion {version}: la prenda exige una cabeza v5 - no por la meta, que no lleva (D-AY), sino porque es la que el nodo sirve y contra la que juzga zkssl_pledge`
- `prenda: {e}`, con `{e}` uno de estos:
  - `la prueba no se deserializa: {e:?}` · `forma de traza {forma:?}; el enunciado pide {:?}` ·
    `{e:?}`, el error de `winter-verifier`

`falta enunciado`, `falta prueba o no es cadena 0x` y `falta cabeza` son los de la edad, letra
por letra. **No hay rechazo por marca no publicada**: este brazo no lo comprueba, y el VERDE lo
dice (D-AS).

**El ancla** (§592, RFC-0012 E3)

- `falta cabeza (la firmada que el ancla compromete)`
- `un sobre con contraria no lleva ancla: la vista dividida se demuestra con las dos cabezas solas`
- `camino sin ancla: no hay nada que extender`
- `cabeza: formatVersion {version}: el ancla lee cabezas v3, v4, v5 o v6: la pareja del MMR
  viaja firmada desde ellas` — el conjunto, derivado de su productor; y el mismo texto con
  `contraria:` como sujeto
- `el ancla no declara v 1: este binario lee ancla v1`
- `el ancla es de OTRA clave: su clave no es la huella de la publicKey de la cabeza`
- `el ancla no ES esta cabeza: su {campo} no casa`, con `{campo}` uno de `indice`,
  `epochDigest`, `mmrRoot` y `mmrSize` — un vector por campo
- `el ancla del genesis (mmrSize 0) no tiene historia que extender: se compara entera, sin camino`
- `el ancla declara un indice ({a}) que no es ANTERIOR al embebido de la cabeza ({b})`
- `la cabeza (t={t}) NO extiende el ancla (t={a}): historia bifurcada, recortada, o camino que
  no es el suyo`
- `los indices embebidos son DISTINTOS ({a}, {b}): dos firmas con su indice propio no dividen
  la vista`
- `las dos cabezas son LA MISMA: no hay vista que dividir`
- `las cabezas llevan claves DISTINTAS: la continuidad es de UN firmante` — el de la extensión,
  letra por letra

Lo que el sobre exige de una cabeza —`available:true`, la recomposición del digest, la firma—
y la lectura del `camino` son los de la extensión, con `cabeza` o `contraria` como sujeto: viven
en su bloque y no se repiten aquí.

**El ancla cofirmada** (§633, RFC-0013 E4a)

- `el sobre del ancla cofirmada no lleva {ajeno}: el ancla se deriva de la cabeza, y la
  extension y la vista dividida son del sobre del ancla`, con `{ajeno}` uno de `ancla`,
  `camino` y `contraria`
- `falta cabeza (la firmada cuya ancla se publico)`
- `falta {campo} o no es cadena ({que})`, con `{campo}` uno de `nota` y `publicador`
- `falta inclusion o no es lista (el camino del ancla a la raiz)` · `inclusion: {e}` ·
  `inclusion: {} bytes, se esperaban 32`
- `testigos no es lista (las vkeys de los testigos)` · `testigos: {e}` · `testigos: {nombre}
  esta dos veces`
- `cabeza: formatVersion {version}: el ancla lee cabezas v3, v4, v5 o v6: la pareja del MMR
  viaja firmada desde ellas` — el del sobre del ancla, letra por letra
- `publicador: {e}` — la vkey no se lee; `{e}` es el error de `zk-ssl-medio`
- `el publicador es {nombre:?} y el medio de esta cabeza es {origen:?}: el origin lleva la
  huella de la clave XMSS (RFC-0013 D-A)`
- `la nota: {e}` — la nota no verifica con su publicador; `{e}` nombra la regla de
  `zk-ssl-medio::nota` que cae (mal formada, checkpoint no canónico, origin ajeno, sin firma
  del publicador, firma repetida o firma inválida)
- `la nota lleva dos cofirmas de {nombre}` · `la cofirma de {nombre} no verifica: {e}`
- `el ancla de esta cabeza no esta en la posicion {p} de las {n} de la nota: {e}`

`posicion` se lee con `u64_de`: sus rechazos son los de la forma de los valores.

## 6. El contrato del mando

- **Invocación:** `zk-ssl-verify <paquete.json>` — **un** argumento, la ruta del fichero. Es la
  única lectura de disco del binario; no hay red, ni reloj, ni telemetría (§395 lo gatea).
- **Salida estándar:** las líneas numeradas de su forma —`1/3` · `2/3` · `3/3` en los paquetes
  de posición (o `3/3 sin acuse en el paquete: la cabeza sola queda demostrada`), de extensión,
  de rechazo, de edad, de cobro pendiente, de pago en curso, de prenda y del ancla; `1/5` a
  `5/5` en el
  de consumo; `1/4` a `4/4` en el de conflicto y en el del ancla cofirmada—, la de cofirmas
  cuando el sobre es v2, y al final el VERDE de su forma, uno de estos quince; los seis del cuarto al noveno siguen en una segunda
  línea, el del cobro llega a una tercera, y los del pago y la prenda a una cuarta:
  - `VERDE: el paquete se sostiene sin el nodo`
  - `VERDE: la extension se sostiene sin el nodo`
  - `VERDE: el consumo se publico entre las dos cabezas, sin el nodo`
  - `VERDE: dos libros aceptaron el mismo consumo. Es DETECCION, no prevencion:`
  - `VERDE: {causa} se sostiene sobre el estado comprometido. Dice que la regla se`
  - `VERDE: bajo la cabeza de seq {seq}, a lo sumo {k} posiciones vivas {quien}`
  - `VERDE: bajo la cabeza de seq {seq} hay un pendiente a nombre del receptor,`
  - `VERDE: bajo la cabeza de seq {seq} hay un pendiente a nombre del receptor por`
  - `VERDE: bajo la cabeza de seq {seq} hay un pendiente que solo puede cobrar quien`
  - `VERDE: el recibo se resolvio dentro de la ventana, y se sostiene sin el nodo` (§573)
  - `VERDE: el ancla se deriva de la cabeza firmada, y se sostiene sin el nodo` (§592; las dos
    líneas anteriores llevan el ancla derivada y su huella, para publicarlas)
  - `VERDE: el ancla ES esta cabeza firmada, y se sostiene sin el nodo` (§592)
  - `VERDE: la cabeza extiende el ancla: la historia anclada es un prefijo, y se sostiene sin el nodo` (§592)
  - `VERDE: VISTA DIVIDIDA - la clave firmo DOS cabezas con el indice embebido {i}. Es
    DETECCION del operador: dos historias, y solo quien tiene la clave pudo producirlas` (§592)
  - `VERDE: la cabeza estaba publicada en el medio {origen}, en la posicion {p}, y la cofirman
    {k} testigo(s) con clave en el sobre. Que testigos valen y cuantos hacen falta lo decide
    quien verifica (RFC-0013 D-D): este mando reporta, no juzga` (§633; una sola línea, y
    antes de ella una por testigo: `testigo {nombre} clave sha256:{hex} marca {t}`)
- **Salida de error:** `ROJO: {motivo}` con un texto del catálogo de la sección 5, y para.
- **Cuatro códigos de salida:** `0` verde · `1` el primer fallo con nombre · `2` uso (ningún
  argumento, o más de uno; imprime el uso en la salida de error) · `3`, desde el §573, el cuarto
  estado del sobre de completitud, «resolución declarada, no probada», que se imprime con su nombre
  y sin `ROJO`, porque no lo es.

## 7. Quién arma el paquete

Dos formas tienen productor en el árbol, y es el **nodo**, en un modo fuera de banda que abre su
libro con el servidor PARADO y copia la cabeza firmada VERBATIM: `zk-ssl-node --prueba-edad`
escribe el sobre de edad (§466), y `zk-ssl-node --prueba-rechazo` el de rechazo de
`AccountFrozen` (§473), `AccountNotFound` (§474) e `InsufficientBalance` (§478); cualquier otra
causa la rehúsa nombrándola. `tools/banco_edad.sh` y `tools/banco_rechazo.sh` los demuestran en
vivo. Del resto, **ningún mando del árbol emite el paquete**: el testigo y el cli sirven y
custodian las respuestas del cable, y quien las reúne en el sobre es el titular — en el árbol,
los bancos `tools/banco_apagado.sh`, `tools/banco_completo.sh`, `tools/banco_evidencia_v2.sh`,
`tools/banco_extension.sh`, `tools/banco_consumo.sh` y `tools/banco_dos_libros.sh`, que
capturan las respuestas de un nodo real y las envuelven sin reescribir un campo; los vectores
de las otras causas del rechazo se reunieron de capturas de un nodo real, y la sección 9 dice
de cuáles. Ese es el contrato: **reunir, no recomponer**, y el modo del nodo lo cumple. Un
mando que arme el paquete es un frente propio y no cambia este documento: cambiaría quién
escribe el sobre, no el sobre —y así fue en el §466 y en el §473—. **El de cobro pendiente lo
produce el COBRADOR** (§497, RFC-0008 D-M y D-P): con su aviso v2 y su credencial, la boca del
cli, `zk-ssl-cli prueba-cobro`, pide la cabeza firmada y la foto a un nodo VIVO, exige que sean
del mismo latido y escribe el sobre con la cabeza VERBATIM; el banco que lo reproduce en vivo y
sus vectores son de E4 (sección 9).

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
**Desde §459 cubre `AccountFrozen`** (RFC-0007, E3b): un positivo reunido de las capturas de un
nodo real que congela por la vía delegada (`dev_freeze`, el PASTE-R7E3b-M) y nueve negativos. El
disfraz no es una mutación: lleva el camino REAL de una cuenta libre, con la hoja vacía bajo la
misma raíz. El ataque del camino truncado con raíz coincidente no se fabrica en JSON: lo falsa el
testigo de `crates/zk-ssl-verify/src/congelados.rs`.
**Desde §460 cubre `SupplyCapExceeded`** (RFC-0007, E3, que queda entera): un positivo reunido
de las capturas de un nodo real con un tope de suministro pequeño (`--max-supply 1000`, el
PASTE-R7SC-M) y siete negativos. Dos no son mutaciones, y se declara: la cabeza anterior es la
FIRMADA REAL de antes de emitir, y el tope no superado es una escena de dos campos —el importe y
el `wouldBe` a la vez—, porque una sola mutación de lo real no llega a esa regla.
**Desde §476 cubre `AccountNotFound`** (RFC-0007, E5, corte 3b, y es la primera causa de E5 con
catálogo): un positivo reunido de las capturas de un nodo real que produce el sobre con
`zk-ssl-node --prueba-rechazo` (`tools/banco_rechazo.sh`, las CAPTURAS-475) y **ocho** negativos,
uno por regla producible. Dos cosas se declaran. El **disfraz** —el nodo dice que la cuenta no
existe y la hoja bajo `accountsRoot` SÍ está ocupada— **no se puede producir**: exigiría el camino
real de una cuenta viva, y la D-G del RFC-0007 cerró la puerta a un método del cable que lo sirva;
su testigo es PURO y vive en los `#[test]` de `crates/zk-ssl-verify/src/cuentas.rs` (§475). Y de
las cuatro mutaciones que el banco falsa en vivo, dos —la hoja y un hermano del camino— caen por
la MISMA regla: el catálogo pina la regla, no la pieza mutada, así que sólo una entra como vector.
**Desde §479 cubre `InsufficientBalance`** (RFC-0007, E5, corte 4c, que cierra E5): un positivo
reunido de las capturas de un nodo real que produce el sobre CON su prueba de banda
(`tools/banco_rechazo.sh`, su tercer brazo, sobre la cuenta libre del banco y un importe
derivado del fondeo; las CAPTURAS-4c) y **ocho** negativos, uno por regla del brazo. Dos son
escenas, y se declara: el pedido cero y el techo tocan los dos `requested` a la vez, porque
tocar uno solo cae antes por el importe distinto. El rojo de la cuenta otra lo escribe
winterfell, y el manifiesto pina el nombre de su variante (0.13.1, fijado en el lock). Sin
vector, y se declara: una prueba corrupta cae por la MISMA regla que la cuenta otra, y una
cabeza con otro `accountsRoot` y el mismo `seq` no se fabrica sin romper la firma.
**Desde §467 cubre el sobre de EDAD** (RFC-0007, E4b-3, que cierra E4): `spec/vectors/edad/`
trae DOS positivos REUNIDOS de las capturas de un nodo real -la capa los produjo con
`zk-ssl-node --prueba-edad` sobre el libro de ese nodo, contra la cabeza v5 de seq 10 que el
nodo firmo- y NUEVE negativos por UNA mutacion cada uno. Los dos positivos son las dos formas
que el enunciado admite hoy: TODOS (`t = 0`) y la CAJA VACIA (`t` por encima de la altura,
luego `k = 0`). Un texto de la familia no se pina entero, y se declara: el del enunciado que
no verifica lo pone WINTERFELL y no la casa, asi que el manifiesto pina solo el prefijo
`edad:` que antepone el mando. La cabeza capturada no viaja como vector: va entera dentro de
los dos sobres, y su huella se declara en la cabecera del manifiesto.
Las demostraciones en vivo con nodo son `tools/banco_apagado.sh`, `tools/banco_consumo.sh`
(RFC-0006, E3), `tools/banco_dos_libros.sh` (E4a), `tools/banco_edad.sh` (E4b-3),
`tools/banco_rechazo.sh` (RFC-0007 E5, cortes 3b y 4c), `tools/banco_pendiente.sh` (RFC-0008 E4,
lado del cobro), `tools/banco_pago.sh` (RFC-0008 E2), `tools/banco_prenda.sh` (RFC-0008 E3) y
`tools/banco_completitud.sh` (RFC-0010 E5): el tercero de ellos levanta DOS nodos con DOS claves
y produce el hecho que E4 existe para detectar, el quinto produce el sobre de rechazo sobre un
libro real con el servidor PARADO, el sexto siembra con el servidor PARADO y pide el sobre del
cobro por la boca con el servidor VIVO, el octavo es el único en el que la boca abre un keystore
y el nodo ACEPTA la marca bajo el mismo latido, y el noveno es el único que ESPERA: firma un
latido por segundo hasta que la ventana de un recibo expira.

**Desde §499 cubre el sobre de COBRO PENDIENTE** (RFC-0008, E4 por el lado del cobro):
`spec/vectors/pendiente/` trae DOS positivos REUNIDOS de las capturas de un nodo real —la boca del
cobrador (`zk-ssl-cli prueba-cobro`, §497) los escribió con el nodo VIVO contra la cabeza v5 de seq
5 que ese nodo firmó, sobre un pendiente v2 nacido en 4— y SIETE negativos por UNA mutación cada
uno, uno por regla producible (D-Q). Los dos positivos son las dos formas de D-N: la existencia
(`inferior = 0`) y la banda ajustada (`inferior = importe`). Un texto de la familia no se pina
entero, y se declara: el de la prueba que no verifica lo pone WINTERFELL y no la casa, así que el
manifiesto pina sólo el prefijo `cobro:` que antepone el mando. La cabeza no viaja como vector: va
entera dentro de los dos sobres; el aviso y la credencial del escenario tampoco —el mando no los
lee— y el manifiesto los declara por su huella. Su productor es `tools/banco_pendiente.sh` (§498),
que siembra con el nodo PARADO y pide con el nodo VIVO. **El lado del pago llegó en el §509** y su
catálogo es `spec/vectors/pago/`, que tiene aquí su párrafo propio desde el §548.

**Desde §509 cubre el sobre de PAGO EN CURSO** (RFC-0008, E2): `spec/vectors/pago/` trae DOS
positivos REUNIDOS de las capturas de un nodo real —la boca del pagador (`zk-ssl-cli prueba-pago`,
§507) los escribió con el nodo VIVO, con SU credencial y con `receiverId` (D-AE, §505), contra la
cabeza v5 de seq 5 que ese nodo firmó, sobre un pendiente v2 nacido en 4 por 250000 y con delta
96— y SIETE negativos por UNA mutación cada uno, uno por regla producible, todos derivados del
positivo de la frontera. Los dos positivos son las dos formas de `--t`, que es ABSOLUTO (D-AI-4):
el `seq` de la cabeza y la FRONTERA `nacido + delta` = 100, que es la última época que el pago
sostiene. Un texto de la familia no se pina entero, y se declara: el de `neg-importe-mentido` lo
pone WINTERFELL —el importe es entrada PÚBLICA del AIR, así que mentirlo deja la prueba sin
verificar su enunciado—, así que el manifiesto pina sólo el prefijo `pago:` que antepone el
mando. Y aquí no se pina ningún PESO: los dos positivos son del MISMO pendiente bajo la MISMA
cabeza y pesan distinto —55.317 B y 54.888 B—, porque lo único que cambia entre ellos, `t`,
cambia el transcript y con él las consultas de FRI (5.A-324). La cabeza no viaja como vector: va
entera dentro de los dos sobres (D-J); los CUATRO ficheros del escenario tampoco —el mando no los
lee— y el manifiesto los declara por su huella. Su productor es `tools/banco_pago.sh` (§508).

**Desde §546 cubre el sobre de PRENDA** (RFC-0008, E3): `spec/vectors/prenda/` trae DOS positivos
CAPTURADOS de un nodo real —la boca del prendador (`zk-ssl-cli prueba-prenda`, §543) los escribió
abriendo el keystore del receptor (§544) contra la cabeza v5 de seq 5 que ese nodo firmó, y el
segundo además publicó la marca con `zkssl_pledge` bajo el mismo latido— y SIETE negativos por UNA
mutación cada uno, uno por regla producible. Los dos positivos afirman LO MISMO bajo la MISMA
cabeza, así que el veredicto del mando no los distingue y el manifiesto los pina con el mismo
texto: lo único que los separa son los bytes de la prueba. Y por eso esta familia se CAPTURA y no
se re-deriva —desde el §538 el probador oculta, y dos pruebas del mismo enunciado no pesan lo
mismo: cuatro medidas, 68.068 y 66.333 B, 67.145 y 66.398 B—, así que el manifiesto no pina ningún
peso. Un texto de la familia no se pina entero, y se declara: el de la prueba que no verifica lo
pone WINTERFELL y es el MISMO para los dos campos del enunciado, así que el manifiesto pina sólo
el prefijo `prenda:` que antepone el mando. La cabeza no viaja como vector: va entera dentro de
los dos sobres; los cinco ficheros de la siembra tampoco —el mando no los lee—, y el manifiesto
declara por huella los cuatro deterministas, no el `keystore.json`, que lleva nonce. Su productor
es `tools/banco_prenda.sh` (§545). Un VERDE aquí es MEDIA prenda (D-AS): la otra mitad es la marca
bajo el `consRoot` de esa misma cabeza, y se pide con `zkssl_consumoPath`.

**Desde §574 cubre el sobre de COMPLETITUD** (RFC-0010, E5): `spec/vectors/completitud/` trae TRES
positivos CAPTURADOS de un nodo real que firma un latido por segundo —un envío de prueba de ceros
que la capa rechaza deja su recibo en el `error.data` del cable, y la cabeza v6 de índice 2 cierra
su era—, uno por veredicto que se siembra: RESUELTA como rechazo con prueba (salida 0), NO RESUELTA
EN LA VENTANA con la cabeza vigente de índice 1443, la primera fuera de ella (salida 1, el ROJO
NOMBRADO), y el cuarto estado (salida 3), DERIVADO del `data` real con una causa sin prueba
portable, porque el nodo no produce hoy esas causas por esta vía, y se declara. Y TREINTA Y DOS
negativos por UNA mutación cada uno, uno por regla producible y por SITIO: los de las claves
distintas llevan la cabeza de un segundo nodo, de otra semilla. El veredicto 1 —resuelta como
transición APLICADA— no tuvo vector hasta el §605: pide la prueba STARK real de un envío aplicado,
y su par cabeza + acuse se verifica como el paquete de posición, con los vectores de `paquete/`; las
reglas propias de esa rama sí lo tienen. **Desde el §605 lo tiene**: `resuelta-por-acuse`, un CUARTO
positivo copiado de OTRO productor, `tools/banco_mentiroso_sin_resolver.sh --guardar` (RFC-0011
E4): un nodo `--dev` contra el que el ejemplo `e2e` del sdk paga con una prueba STARK real, y el
acuse de la entrada bajo la cabeza que la firma. Las que el sobre comparte con otras familias —lo que se exige de
una cabeza, la lectura del camino, lo que la resolución re-verifica— viven en ellas, y se declara.
Dos corridas del banco dan el mismo cierre salvo `emittedAtUnix`, que no va firmado: la familia se
COPIA de la corrida del sello. Su productor es `tools/banco_completitud.sh` (§574), con `--largo`
—unos 24 minutos— para la ventana expirada. **Desde el §614 cubre también las formas `lote` y
`prenda`** (RFC-0014, E5): TREINTA Y SIETE vectores más, COPIADOS de una corrida de
`tools/banco_recibo_agregado.sh --guardar` contra tres nodos reales —el del lote con el grifo
`--dev`, contra el que el ejemplo `d2_lote_rpc` del sdk paga un lote de dos envíos con pruebas STARK
reales; los dos de la prenda con su siembra y una prenda de verdad—: diez veredictos —el LOTE
aplicado, rechazado con prueba, por su FORMA, declarado y dos RECHAZOS SIN FUNDAMENTO; la PRENDA
aceptada, rechazada con prueba, rechazada por la capa y un RECHAZO SIN FUNDAMENTO— y veintisiete
negativos, uno por regla producible y por sitio. Los RECHAZOS SIN FUNDAMENTO y el cuarto estado del
lote se DERIVAN por mutación de lo capturado, porque un nodo honrado no los produce, y se declara;
las claves distintas y la resolución fuera de la ventana no se producen ahí.

**Desde §593 cubre el sobre del ANCLA** (RFC-0012, E4): `spec/vectors/ancla/` trae CUATRO
positivos CAPTURADOS de un nodo real que firma un latido por segundo, uno por modo: el ancla
DERIVADA de la cabeza sola —el mando la imprime y el banco la parsea, sin recomputar nada: un
solo productor—, la EXACTA contra su propia cabeza, la EXTENDIDA —el ancla de la cabeza vieja,
el camino de `zkssl_consistencyProof` y la cabeza que firma el tamaño de ese camino— y la
VISTA DIVIDIDA, sembrada reproduciendo el ataque de verdad: la misma semilla con un contador de
índice fresco y un libro en el que un envío evaluado-y-rechazado movió el `recepCount` firmado
—dos cabezas con el mismo índice embebido y digests distintos, que sólo quien tiene la clave
puede producir—. Y DIECISIETE negativos por UNA mutación cada uno, uno por regla producible y
por sitio: los de las claves distintas llevan la cabeza de un segundo nodo, de otra semilla.
Las reglas que el sobre comparte con otras familias —lo que se exige de una cabeza, la lectura
del camino plano— viven en ellas, y se declara. Dos corridas del banco no dan los mismos bytes
(`emittedAtUnix` no va firmado): la familia se COPIA de la corrida del sello. Su productor es
`tools/banco_ancla.sh` (§593).

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
- §459 — el sobre de rechazo prueba `AccountFrozen` (RFC-0007 E3b): la hoja de la cuenta bajo el
  `frozenRoot` de la cabeza del `seq` exacto, con el camino de `zkssl_frozenPath` tal cual, la
  profundidad que fija el núcleo, el cruce con el índice y la hoja no vacía.
- §460 — el sobre de rechazo prueba `SupplyCapExceeded` (RFC-0007 E3, que queda entera): los
  parámetros dan el `maxSupply` comprometido, la cabeza es la del `seq` exacto y el `wouldBe` del
  nodo tiene que ser el `totalSupply` de la cabeza más el importe que el solicitante envió
  (`peticion`); `PendingTreeExhausted` se declara sin prueba portable.
- §465 — el paquete de edad (RFC-0007, E4b-2), la séptima forma: el mando exige una cabeza
  v5 y verifica la prueba de `zk-ssl-air` contra lo que ella firma, sin el probador.
- §466 — el nodo escribe el sobre de edad con `--prueba-edad`, sobre su libro y con el
  servidor parado (RFC-0007, E4b-3): la primera forma con productor en el árbol (sección 7).
- §467 — el catálogo del sobre de edad (RFC-0007, E4b-3, que cierra E4): `spec/vectors/edad/`,
  dos positivos reunidos y nueve negativos. Los catálogos son CINCO.
- §473 — el nodo escribe el sobre de rechazo con `--prueba-rechazo` (RFC-0007, E5): la causa
  se EXIGE, no se deduce del orden de las guardas; la primera es `AccountFrozen`.
- §475 — el sobre de rechazo prueba `AccountNotFound` (RFC-0007, E5): la hoja VACÍA de la
  cuenta bajo el `accountsRoot` de la cabeza del `seq` exacto, con el material dentro del
  propio rechazo y sin método del cable (el nodo lo escribe desde el §474).
- §476 — su catálogo: un positivo reunido y ocho negativos; el disfraz no se puede producir.
- §478 — el sobre de rechazo prueba `InsufficientBalance` (RFC-0007, E5): la única causa que
  verifica un STARK; el sobre no publica el saldo, y el nodo escribe la `banda`.
- §479 — su catálogo: un positivo reunido y ocho negativos. E5 queda entera, y con ella las
  cinco etapas del RFC-0007.
- §480 — este documento se pone al día con el RFC-0007 entero: los textos de `AccountNotFound`
  en la sección 5, las cabezas que su brazo acepta, el material de cada causa y el paquete de
  conflicto en la sección 4, las líneas y los VERDE de cada forma, quién arma el paquete y el
  quinto catálogo del artefacto.
- Hasta §397 este contrato vivía en la cabecera de `crates/zk-ssl-verify/src/main.rs` (1..90,
  `293990fedc785833`), que ya confesó una vez (§247) haber declarado su superficie como completa
  sin serlo. §397 lo muda aquí y deja la cabecera remitiendo, sin enumerar.
- Cambiar este documento es cambiar el contrato: entra por RFC (`spec/rfc/PROCESO.md`).

## 11. El artefacto

Lo que un tercero descarga es `arqueo-verify-<versión>-<host>.tar.gz` (§401), y dentro:
`zk-ssl-verify` (el binario), `conformidad.sh` (el arnés de la sección 9, §408), `spec/PAQUETE.md`
(este documento), `spec/vectors/<familia>/` por cada una de las DIEZ familias de `FAMILIAS`
—paquete, consumo, conflicto, rechazo, edad, pendiente, pago, prenda, completitud y ancla— (los
diez
manifiestos y sus vectores), `LICENSE-APACHE`, `LICENSE-MIT`, `NOTICE`, `THIRD-PARTY.txt` (las
licencias de todo lo enlazado), `VERSION` (el commit, el toolchain y los flags con que se compiló)
y `SHA256SUMS` (la huella de cada fichero de dentro). Se comprueba con `sha256sum -c SHA256SUMS`, y
el binario contra los diez catálogos con `bash conformidad.sh ./zk-ssl-verify` —el del paquete,
por defecto— y `bash conformidad.sh ./zk-ssl-verify spec/vectors/<familia>/MANIFIESTO.txt` para
cada una de las otras nueve: cada entrada dice el código de salida y el texto. Esta sección decía
SEIS hasta el §574: el pago (§509) y la prenda (§546) entraron en `FAMILIAS`, y en el tarball, sin
que ella lo dijera; el §574 lo corrige al sumar la novena, y el §593 suma la décima, el ancla.

La huella del binario **no depende de la máquina ni del usuario** —se compila con
`--remap-path-prefix`—, pero sí del toolchain y de `Cargo.lock`: con el `rustc` que `VERSION`
nombra, `bash tools/artefacto.sh` sobre el commit que `VERSION` nombra vuelve a producir el mismo
binario y el mismo tarball, y `tools/canon.sh` comprueba esa propiedad en cada sello (dos
compilaciones en dos rutas, misma huella; dos tarballs, misma huella; los diez manifiestos desde el
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
