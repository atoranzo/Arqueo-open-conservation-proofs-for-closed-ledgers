# ZK-SSL JSON-RPC — especificación v0.2 (`zkssl/0.2`)

> **Qué cambió de `0.1` a `0.2` (§209, etapa 1 del RFC-0002).** Un solo
> cambio, y no está en los métodos: **`proof_digest` deja de calcularse
> con un hash algebraico**. Con él cambian `chain` y la cabeza de época,
> que son valores de cable. Ningún método, campo ni tipo se añade,
> quita ni renombra.
>
> **Los vectores de `zkssl/0.1` se conservan** en `spec/vectors/` y
> siguen describiendo aquella versión: una implementación de `0.1` sigue
> pudiendo comprobarse contra ellos. `conformance --check` **rechaza**
> validar vectores de una versión distinta a la suya, y eso es correcto.
>
> Motivo, medido en `AUDITORIA.md` §204: ese hash era el **93 %** del
> coste de aplicar una operación —30,99 ms de 33,27— para un resumen que
> **no entra en ningún circuito**.

Especificación normativa del API de nodo. `zk-ssl-node` es la
implementación de referencia; `zk-ssl-wire` define los tipos de cable.
Cualquier implementación que hable esto es un nodo ZK-SSL válido de cara
a las herramientas del ecosistema. (Análogo a `execution-apis` de
Ethereum; una versión OpenRPC generada desde `zk-ssl-wire` está en el
roadmap.)

## Transporte y sobre

- HTTP `POST /`, cuerpo JSON-RPC 2.0. **Un objeto por petición**: no se
  admite el *batch de JSON-RPC* (un array de peticiones en un cuerpo).
  ⚠️ **Esto no tiene nada que ver con `zkssl_applyMany`.** Aquel es un
  **lote de OPERACIONES** dentro de UNA petición; esto es un lote de
  PETICIONES, y no se admite. La misma palabra para dos cosas ha
  causado confusión desde v0.1 y por eso se separa aquí.
- **Tamaño del cuerpo**: el límite por defecto del transporte son
  **2.097.152 bytes** (§218, medido). Una operación con prueba ronda los
  132.728 en hex, así que en un `zkssl_applyMany` entran **15**.
- Respuesta: `{"jsonrpc":"2.0","id":…,"result":…}` o
  `{"jsonrpc":"2.0","id":…,"error":{"code":…,"message":…}}`.

## Codificación (la convención más aceptada, adaptada)

| tipo    | forma                                   | ejemplo |
|---------|-----------------------------------------|---------|
| QUANTITY| u64 en hex con `0x`, sin ceros a la izq.| `"0x3d090"` |
| DATA    | bytes en hex con `0x`, longitud par     | `"0x1f8b…"` |
| Digest  | DATA de 32 bytes, **la misma serialización que persiste la capa** (`store::digest_to_bytes`) | `"0x9a41…"` (64 hex) |

La deserialización valida canonicidad: un digest cuyos elementos no
pertenecen al cuerpo se rechaza con `-32602` antes de tocar la capa.

## Principio que el API preserva

**La clave de gasto no viaja jamás.**

- Abrir cuenta = enviar identificadores derivados en el cliente:
  `publicId`, `viewId`, `leafSalt`.
- Pagar/cobrar = pedir materiales (caminos y raíces: públicos), **probar
  en local**, presentar el recibo.
- Consultar el propio saldo = presentar la **clave de vista** (derivada,
  solo autoriza a leer esa cuenta; 49-A).

## Métodos

### Lectura

| método | params | result |
|---|---|---|
| `zkssl_protocolVersion` | — | `"zkssl/0.2"` |
| `zkssl_params` | — | `{regulatoryLimit, maxSupply, maxAccounts: Q, custodianRoot: Digest}` |
| `zkssl_epochHead` | — | `{seq, accountsRoot, pendingRoot, frozenRoot, chainDigest, acusesRoot, n, mmrRoot, mmrSize, epochDigest}` |
| `zkssl_supply` | — | `{total, pending: Q}` |
| `zkssl_accountCount` | — | `Q` |
| `zkssl_publicId` | `{index: Q}` | `Digest` |
| `zkssl_accountView` | `{index: Q, viewKey: Digest}` | `AccountView` (autenticada; `AccountNotFound` si la clave de vista no corresponde) |
| `zkssl_logEntry` | `{seq: Q}` | `LogEntry` |
| `zkssl_logEntries` | `{fromSeq?: Q, limit?: Q}` | `LogEntry[]` (límite ≤ 1000) |
| `zkssl_verifyChain` | — | `{ok: bool, entries?, error?}` |
| `zkssl_inclusionReceipt` | `{index: Q, viewKey: Digest}` | `{index, leaf, path, leafFormat, head}` |
| `zkssl_ackPath` | `{seq: Q}` | `{available, s?, camino?: {siblings: Digest[], isRight: bool[]}, reason?, beatSeconds?}` |
| `zkssl_consistencyProof` | `{oldSize: Q}` | `{available, mmrSize: Q, camino?: Digest[], reason?}` |
| `zkssl_submitCosig` | `{cosig: Cosig}` | `{accepted: bool, stored: Q, reason?}` |
| `zkssl_cosigs` | `{epochDigest?: Digest}` | `{epochDigest, n: Q, cosigs: Cosig[]}` |

`LogEntry = {seq: Q, kind: string, rootOld, rootNew, proofDigest, chain: Digest}`
con `kind` ∈ {`OpenAccount`,`Mint`,`Transfer`,`Burn`,`Recovery`,
`Governance`,`Freeze`,`Send`,`Claim`,`MintToPending`,`Migration`,`Refund`}
(los `OpKind` de la capa; dicen qué circuito verifica la entrada).

### Apertura

| método | params | result |
|---|---|---|
| `zkssl_openAccount` | `{publicId, viewId, leafSalt: Digest}` | `{index: Q}` |

La cuenta nace con saldo CERO por diseño. El cliente deriva los tres
identificadores con `stark_experiment::native::{derive_public_id_wide,
view_id_of_wide, derive_leaf_salt_wide}`.

### Pago en dos fases

| método | params | result |
|---|---|---|
| `zkssl_sendMaterials` | `{sender: Q, viewKey: Digest, receiverId: Digest, amount: Q, salt: Digest}` | `SendMaterials` |
| `zkssl_applySend` | `{receipt: SendReceipt, sender: Q, senderState: ClientState, amount: Q}` | `Applied` |
| `zkssl_claimMaterials` | `{receiver: Q, viewKey: Digest, notice: PendingNotice}` | `ClaimMaterials` |
| `zkssl_applyClaim` | `{receipt: ClaimReceipt, receiver: Q, receiverState: ClientState, notice: PendingNotice}` | `Applied` |
| `zkssl_applyMany` | `{ops: BatchOp[]}` | `BatchApplied` |

- `Applied = {logSeq: Q, kind, accountsRoot, chain: Digest}` — la
  entrada que quedó en el registro encadenado.
- `SendReceipt.proof` / `ClaimReceipt.proof` son
  `winterfell::Proof::to_bytes` (~54–66 KB según circuito en la configuración actual: el precio, medido, de no
  depender de ninguna ceremonia).
- `PendingNotice = {position: Q, salt: Digest, amount: Q}` y **viaja
  fuera de banda** del pagador al receptor (ISO 20022 no lo transporta;
  AUDITORIA §21). El RPC nunca lo entrega a terceros.
- El pago **no es firme hasta el claim**; sin cobro, el importe queda
  inmovilizado hasta claim o refund (§29/§30).

### `dev_*` — solo con `--dev` (custodios de PRUEBA)

| método | params | result |
|---|---|---|
| `dev_fund` | `{index, amount: Q}` | `Applied` + `custodianNullifiers: Digest[2]` |
| `dev_openSeeded` | `{seed: Q}` | `{index: Q, publicId, viewKey: Digest}` |

`dev_fund` es el grifo del sandbox: emisión delegada REAL con dos
custodios de la suite, incluyendo los nullifiers de umbral que consumen
(`circuit_threshold_single_nullifier`). Un build de producción se
compila sin la feature `dev` y no contiene este espacio.

## Errores

| code | significado |
|---|---|
| `-32601` | método desconocido (o `dev_*` deshabilitado) |
| `-32602` | parámetros inválidos / codificación no canónica |
| `-32000` | rechazo de la capa: `message` = `LayerError` (p. ej. `InsufficientBalance{…}`, `StaleState`, `OverRegulatoryLimit{…}`, `AccountFrozen(…)`) |

`StaleState` es esperable bajo concurrencia: el estado declarado quedó
atrás. El cliente refresca su vista y reintenta.

## Qué afirma el registro de transiciones

Esta sección es **normativa** y describe lo que `zkssl_logEntries` y
`zkssl_verifyChain` garantizan hoy, y lo que dejarán de garantizar si el
nodo aplica operaciones **por lotes** (RFC-0002, etapa 2). Se escribe
**antes** de implementarlo para que nadie construya sobre una garantía
que va a cambiar.

### Lo que el registro afirma SIEMPRE

1. **Los números de secuencia son consecutivos** desde cero.
2. **`rootOld` de una entrada es `rootNew` de la anterior**, y la primera
   arranca en la raíz del génesis. Es lo que impide insertar o borrar
   operaciones del medio.
3. **`chain` es el resumen encadenado** de la entrada y de todo lo
   anterior: alterar cualquier campo de cualquier entrada rompe la cadena
   desde ahí hasta el final.
4. **`proofDigest` ata la entrada a lo que la AUTORIZÓ**, y el dominio
   del sello dice a qué clase pertenece (§278): a los **bytes de la
   prueba** donde la hay (§116); al **compromiso de umbral** en las
   vías delegadas; o a la **ausencia declarada**, con dominio propio,
   donde no hay prueba por diseño. El registro no guarda ninguno de
   los tres; los sellos los distinguen.

Estas cuatro se mantienen con lotes o sin ellos. `zkssl_verifyChain` las
comprueba.

### `zkssl_applyMany` — N operaciones contra UNA raíz de arranque

```
BatchOp = {kind: "send",  receipt: SendReceipt,  sender: Q,
           senderState: ClientState, amount: Q}
        | {kind: "claim", receipt: ClaimReceipt, receiver: Q,
           receiverState: ClientState, notice: PendingNotice}

BatchApplied = {
  batch:   {size, fromSeq, toSeq: Q, rootOld, rootNew, chain: Digest},
  applied: Applied[]
}
```

**Los campos son los MISMOS que los de `zkssl_applySend` y
`zkssl_applyClaim`, más un discriminante.** Quien ya habla el protocolo
no aprende nada nuevo, y por eso el método es **aditivo**: los dos
sueltos siguen existiendo con su respuesta síncrona intacta, y la
versión del protocolo **no cambia**.

Reglas normativas:

1. **Todo o nada.** Se validan las N operaciones contra el estado de
   arranque y solo entonces se aplican. Si una falla, **ninguna** se
   aplica y la respuesta es un error.
2. **Un lote vacío es `InvalidParams`.** No hay `fromSeq` ni `rootOld`
   que devolver.
3. **Una cuenta como máximo una vez por lote**, y **posiciones de
   pendiente distintas**. Dos operaciones que compartan posición
   rechazan el lote entero (`DuplicatePendingInBatch`). Por eso un
   cobro **no puede ir en el mismo lote** que el envío que crea su
   pendiente: van en lotes consecutivos.
4. **`applied` va en el orden de entrada de `ops`**, que es el mismo en
   que se aplican, y sus `logSeq` son consecutivos desde
   `batch.fromSeq`.
5. **`accountsRoot` de cada `Applied` es el `rootNew` de SU entrada del
   registro**, no la raíz final del lote.
6. **`batch.rootOld` es la raíz de arranque contra la que se validaron
   TODAS las pruebas.** Es lo único que el lote puede devolver a cambio
   de la garantía que quita (ver la sección siguiente): deja al cliente
   comprobar contra qué se validó la suya.

⚠️ **Quien arma el lote no es el nodo.** El nodo no acumula operaciones:
aplica las que le llegan juntas en una petición. Juntarlas es trabajo de
un **agregador** —un banco, un proveedor de pagos—. No necesita claves:
las pruebas vienen hechas.

#### Qué ve exactamente un agregador

⚠️ Hasta §231 aquí se afirmaba que **«ve quién paga a quién»**. Es
**falso**, y era una afirmación no comprobada. Medido campo a campo
sobre los DTO:

| procesa | ve | NO ve |
|---|---|---|
| solo envíos | emisor, importe, `notice.position` | **el receptor** |
| solo cobros | receptor, importe, `notice.position` | **el emisor** |
| **ambos** | **la arista completa** | — |

**El identificador del receptor NO viaja en el recibo de envío.**
`SendReceiptDto` lleva `proof`, `public_inputs`, `commitment` y
`notice`; y `SendPublicInputsDto` son raíces, importe, límite y
suministro. El `receiver_id` solo aparece en `SendMaterialsDto`, que va
**del nodo al titular**, no del titular al nodo.

Lo que une las dos mitades es **`notice.position`**: aparece en
`receipt.notice.position` al enviar y en `notice.position` al cobrar. Un
agregador que vea las dos correlaciona por esa clave y reconstruye
emisor → receptor → importe.

**Consecuencia para quien despliegue**, que la afirmación anterior
ocultaba: **separar envíos y cobros en agregadores distintos es una
mitigación real.** Ninguno de los dos ve el grafo por sí solo.

Y esto no exime al NODO, que ve todo lo anterior por definición.
`SECURITY.md` ya lo decía bien —«qué posiciones cambian y cuándo siguen
siendo observables»—; la afirmación errónea estaba aquí, no allí.

### ⚠️ Un lote por raíz: `applyMany` asume UN agregador

Todas las pruebas de un lote acreditan su transición contra **la raíz de
arranque**. Cuando un lote aplica, la raíz se mueve, y **cualquier otro
lote probado contra la raíz anterior se rechaza entero** con
`StaleState`.

Esto no es un detalle de implementación: es una **restricción del
protocolo**, y quien escriba un cliente concurrente tiene que saberla
antes de descubrirla tirando pruebas.

#### Turnarse, no competir — y un turno no es un consenso

De la restricción se sigue que **varios agregadores tienen que turnarse**
en vez de competir. §230 midió el precio de no hacerlo: cuatro que salen
a la vez aplican uno y los otros tres tiran **el 75 %** de sus pruebas.

⚠️ **Un turno no es un consenso, y conviene no leerlo así.** Un consenso
decide **quién tiene razón** entre partes que discrepan sobre el estado.
Un turno decide **quién va primero** entre partes que ya saben que solo
cabe una y no discrepan de nada. Aquí no hay nada que acordar: la raíz ya
determina el resultado, y turnarse solo evita generar pruebas que van a
morir. Nada de esto abre una grieta en el modelo de nodo único de
`SECURITY.md` §6.

⚠️ **Qué mecanismo de turno usar no lo dice esta especificación, y es
deliberado.** La especificación normativa manda sobre **el cable** —qué
viaja y qué significa—, no sobre **quién opera qué**. El modelo de
despliegue recomendado está en `SECURITY.md` §2.ter, como recomendación
y no como requisito de conformidad.

Medido (§230, banco I.1) con cuatro agregadores enviando a la vez, cada
uno con sus propias cuentas —sin competir por cuenta ni por posición de
pendiente, solo por la raíz—:

| | |
|---|---|
| lotes que aplican por ronda | **1 de 4** |
| rechazados por `StaleState` | **3 de 4** |
| pruebas desperdiciadas | **75 %** |

Dos consecuencias para quien implemente:

1. **El nodo rechaza barato.** Un lote muerto cuesta **3,1 ms** frente a
   los 32 de uno aplicado —el 9 %—, porque la raíz se comprueba antes de
   verificar las pruebas. Un nodo con varios agregadores no se ahoga.
2. **El precio lo paga quien pierde.** Ocho pruebas descartadas son ~2 s
   de CPU del agregador; el nodo tiró 9 ms. Una razón de **655×**.

Y por tanto: **el lote no elimina la contención, le cambia el grano.**
Aplicando de una en una se contiende por operación y se pierde una
prueba; en lote se contiende por lote y se pierden N. Con **un solo
agregador** el desperdicio es cero y el nodo aplica 248 op/s (§229).

§223 declaraba que el agregador ve el grafo de pagos; §231 lo corrigió
—solo lo ve quien procesa **las dos mitades**—. Ésta es la segunda
razón, y técnica, de que haya **uno**: dos no pueden ganar la misma
raíz.

### Lo que vale al aplicar de una en una, y el lote quita

⚠️ **Hoy, cada operación se aplica sola, y por eso las raíces del
registro coinciden con las que la prueba declara.** Un tercero que tenga
una entrada **y** su prueba puede comprobar que la transición que la
prueba acredita es exactamente la que el registro anotó, **sin necesidad
de tener el árbol**.

**Con `zkssl_applyMany` eso deja de ser cierto —y desde §222 el método
existe, así que ya no es una advertencia sobre el futuro—.** Sigue
valiendo para las operaciones que se apliquen de una en una con
`applySend` y `applyClaim`, que no se han tocado. En un lote de N
operaciones, en cambio, cada
prueba se genera contra la **raíz de arranque del lote** y acredita una
transición **hipotética** —«la raíz que saldría si mi cambio fuera el
único»—. El registro, en cambio, tiene que anotar las raíces **reales**,
porque es lo que exige la garantía 2.

En consecuencia, dentro de un lote:

- `rootNew` de una entrada **no** será, en general, la raíz que su propia
  prueba declara;
- atar una entrada a la transición que su prueba acredita **exigirá tener
  el árbol** —es decir, replicar el estado—, no solo la entrada y la
  prueba.

Esto es coherente con el modelo declarado en `SECURITY.md` §6 y en el
asiento §121: la verificación por **réplica**. Pero es una **pérdida
real** frente a lo que hoy se puede hacer, y por eso se declara aquí en
vez de descubrirse al leer el código.

### Y una consecuencia para la conformidad

⚠️ **La composición de los lotes es observable.** Dos implementaciones
que apliquen la misma secuencia de operaciones **agrupándolas de forma
distinta** producirán entradas con `rootNew` distintos y, por tanto,
cadenas distintas. El registro deja de ser una función únicamente de la
secuencia de operaciones: **también depende de cómo se agruparon**.

Por eso, **los vectores de conformidad declaran su política de
agrupación**. Los de `zkssl/0.1` y `zkssl/0.2` se generaron **sin lotes**
—una operación por aplicación, N=1— y una implementación debe
reproducirlos aplicando de una en una. Cuando existan vectores de lote,
lo dirán explícitamente.

⚠️ **Desde §222 `zkssl_applyMany` existe, y los vectores de `zkssl/0.2`
siguen siendo de N=1.** Eso es deliberado y no es una omisión: la
superficie del protocolo es aditiva y los valores de cable no se
movieron, así que los vectores existentes **siguen siendo válidos tal
cual** —`conformance --check` de `zkssl/0.2` sigue dando idéntico—. Una
implementación que quiera acreditar el lote necesitará vectores propios;
no los hay todavía.

---

### `receptionSeq` — el contador de recepción (§253)

`zkssl_applySend` y `zkssl_applyClaim` devuelven **`receptionSeq`**: en qué
orden llegó esa operación **de entre las que el nodo se puso a evaluar**.

⚠️ **No es `logSeq`.** `logSeq` es **el orden de aplicación** y solo existe
si la operación se aplicó; `receptionSeq` es **el orden de llegada**. **La
censura vive en el hueco entre los dos.**

| falla en | ¿consume `receptionSeq`? |
|---|---|
| el parseo o el cable | **no** — eso es ruido, y si contara **cualquiera podría abrir huecos en el registro ajeno mandando basura** |
| la capa (prueba inválida, raíz movida) | **sí** — el nodo verificó una operación de verdad, y **ahí es donde se escondería un censor** |

⚠️ **Viaja también en el error**, como `[receptionSeq=0x…]` en el mensaje:
el caso que importa es justo el rechazo.

### ⚠️ Lo que `receptionSeq` NO es: evidencia oponible

**Es un número que el nodo dice y que nada ata.** `chain` autentica `seq`,
`kind`, las dos raíces, el digest de prueba y el anterior — **y nada más**.

- **Dos titulares que cooperan DETECTAN la reordenación**: A tiene
  recepción 100, B tiene 101, la de B está en el log y la de A no.
- **No pueden PROBARLA**: ninguno tiene nada firmado por el operador que
  diga *«recibí la tuya la 100»*, y **el operador puede negar haberlo
  dicho**.

⚠️ Y el titular **ve un hueco y no sabe si fue censura o su propia prueba
mala**: esto **detecta la reordenación, no la explica**.

Lo que falta para las dos cosas es **el acuse**, como hoja bajo una raíz en
la cabeza — hereda la firma **sin gastar índices XMSS**.

### ⚠️ `receipt` y `acuse` son cosas distintas

| palabra | qué es |
|---|---|
| **`receipt`** | lo que **el titular entrega** al nodo: `SendReceipt` y `ClaimReceipt` llevan `proof`, `public_inputs` y `commitment` |
| **`acuse`** | lo que **el nodo devuelve** al titular |

**No se reutiliza `receipt` para el acuse**: sería un tercer significado en
el mismo cable, y en un proyecto cuyo argumento es la conformidad
verificable un nombre ambiguo cuesta más que en otro sitio.

#### El acuse en la respuesta (§274)

`zkssl_applySend` y `zkssl_applyClaim` —**sólo las vías del titular**—
devuelven, junto al `logSeq` y al `receptionSeq` de siempre:

```json
"acuse": { "epoca": "0x2b", "n": "0x5a0", "hashPrueba": "0x..." }
```

- **`epoca` = `logSeq + 1`**: la **primera cabeza que puede contener** la
  operación — no la que la contendrá, que la elige el operador. Una hoja
  que declara esta época viviendo bajo la cabeza `S` hace legible
  `S − epoca` desde la hoja y la cabeza solas: la magnitud que la promesa
  acota.
- **`n` = 1440**, el techo declarado (§121) y **normativo desde §275**:
  viaja firmado en la cabeza, y un `n` mentido aquí produce una hoja que
  no verifica bajo la raíz firmada.
- La hoja es `acuse_digest(hashPrueba, epoca, n)` (§270) — el titular la
  computa **en el apply** y queda fija para siempre.

⚠️ **Límite declarado**: esta respuesta **no va firmada**. Firmar cada
acuse cuesta 160,5 ms medidos (~×50 de colapso sobre el techo de §217) y
gastaría índices XMSS (§121.2). El acuse hereda la firma **al cerrar la
época**, bajo la raíz — **real desde §275**, vía `zkssl_ackPath`; hasta
entonces es palabra del nodo, y la ventana es ≤1 latido con operador
honesto. Ver los asientos §274 y §275.

### `zkssl_signedEpochHead` — la última cabeza firmada, para un TESTIGO

Devuelve la cabeza de época **más reciente que el nodo firmó**, con todo lo
que hace falta para verificarla sin él: los **nueve campos** de la
cabeza (§275, §292), `publicKey`, `epochDigest`, `formatVersion`, `index` y
`signature` — campos+digest+firma **juntos**, del mismo latido: un solo
artefacto de custodia, sin carrera entre llamadas.

⚠️ **Aditivo**: no toca `zkssl_epochHead`, que sigue sirviendo la cabeza
**sin firma** y está en los vectores de conformidad. **La versión no sube**,
por la misma razón que no subió con `zkssl_applyMany`.

### ⚠️ Formato **v3** (§292): la cabeza prueba que EXTIENDE

La cabeza gana `mmrRoot` y `mmrSize`: la **cima del MMR de cabezas**
sobre los digests de todas las anteriores, y cuántas acumula. Viajan
**firmadas** — `formatVersion: 3` — y con ellas un cliente que custodia
una cabeza vieja puede verificar que la nueva la EXTIENDE con una prueba
de consistencia O(log N), sin descargarse el registro (eslabón 2 de la
cadena de confianza residual).

La composición, por envoltura como v1→v2:
```text
v3 = merge( epoch_digest_v2(los siete), merge(mmrRoot, mmrSize) )
```
**Génesis declarado**: la primera cabeza compone con cima `as_digest(0)`
y `mmrSize: 0`. ⚠️ **Aditivo en el cable, rotura en el parser**:
`zkssl/0.2` no sube (el precedente de §275), y un consumidor con
`deny_unknown_fields` **rompe en voz alta** — el fallo honesto ya
diseñado. Una cabeza **v2 custodiada sigue verificando**: la versión que
la firma declara elige recomponedor, en la biblioteca y en el mando.
Un nodo **sin diario** rearranca con el acumulador vacío y su
`mmrSize` se resetea de forma VISIBLE; quien firma lleva diario
obligado (§285), así que toda cabeza firmada lleva continuidad.

⚠️ **Tres respuestas, y ninguna es un error genérico**:

| `available` | cuándo | qué trae |
|---|---|---|
| `false` | aún no ha habido latido | `reason`, `beatSeconds` |
| `false` | el nodo arrancó **sin `--clave`** | `reason`, la cabeza **sin firma** |
| `true` | hay cabeza firmada | todo lo necesario para verificar |

El segundo caso es la forma de §241 llevada al cable: **la pieza que falta
—la firma— se nota también aquí**, y no como un fallo.

### ⚠️ `custody` y `custodyChecked` — afirmado frente a comprobado

La respuesta lleva **siempre** los dos campos, incluso cuando no hay firma.

| campo | qué es |
|---|---|
| `custody` | **lo que el operador AFIRMA**: `sin-declarar` (por defecto), `fichero`, `hsm`, `kms`, `otro` |
| `custodyChecked` | si **el nodo pudo comprobarlo**. Solo `fichero` es comprobable |

⚠️ **Es una afirmación del operador, no una comprobación del nodo.** El nodo
no puede saber si hay un HSM detrás, y no finge que sí: si el operador
declara `hsm`, la respuesta lleva `custodyChecked: false` y el arranque lo
avisa en voz alta.

⚠️ **El valor de la declaración no está en que sea cierta, sino en que
mentir en ella es oponible.** Un operador que declara `hsm` y opera con un
fichero ha hecho una afirmación falsa que no puede negar — el mismo modelo
que sostiene el resto del aparato.

⚠️ **`sin-declarar` viaja igual que los demás.** Si el campo se omitiera al
no declarar nada, un consumidor no podría distinguir **«no declara»** de
**«versión vieja del nodo»**. Presente y honesto por defecto.

⚠️ Y nada de esto hace la firma oponible: **la clave pública sigue sin
ancla** (`SECURITY.md`). Un tercero verifica contra la clave que el mismo
nodo le dio, y eso es circular.

`emittedAtUnix` y `beatSeconds` existen para que **un testigo que pide dos
veces y recibe la misma firma distinga «no ha habido latido» de «me están
engañando»**. El `index` de XMSS ya lo permite —es monótono— pero conviene
que sea explícito.

⚠️ **Solo la última, y en memoria.** No hay histórico: **se pierde al
reiniciar**, y con un arranque de ~136 s a 10⁶ cuentas un testigo pasará ese
tiempo recibiendo `available: false`.

⚠️ **MEDIDO en L.1 (§247).** Lo que el testigo ve **no es un hueco de
índices**: es una **ventana de `SinFirma`**, y después el índice sigue
**contiguo** —el guardián solo incrementa al firmar, así que morir entre
latidos no gasta ninguno—. Hay hueco de índices **solo si se firmó una
cabeza que nadie llegó a recoger**, y eso depende de la relación entre la
cadencia del latido y la de la consulta, **no del reinicio**.

### ⚠️ El histórico: DECIDIDO en §248 — el operador NO lo sirve

§242 lo aplazó por falta de consumidor. El consumidor llegó (§245) y el dato
también (§247) — y la respuesta **no es la que se esperaba**:

> **Un histórico servido por el operador es el operador diciendo qué dijo
> antes.** Puede reescribirlo. Un tercero que lo consulta no gana ninguna
> propiedad que no tuviera.

No es que cueste: **26 MB al día no es caro**. Es que **no aporta lo que se
le pedía**. Almacenar 9,5 GB al año para que el operador pueda repetirse a
sí mismo es pagar por nada.

⚠️ **Y el argumento decisivo**: una vista dividida **entre partes distintas**
es indetectable desde un histórico central **por construcción** — el
operador que la produce es el mismo que sirve el histórico. **Solo dos
registros independientes la revelan.**

Por eso el registro lo llevan **los testigos**, en su diario (`--diario`),
que desde §248 guarda **la cabeza firmada entera**: un tercero reverifica
sin el nodo, y dos diarios se comparan campo a campo.

⚠️ **La firma es lo que hace que esto funcione.** Es lo que impide que un
testigo malicioso fabrique evidencia contra el operador: sin ella, comparar
diarios no probaría nada. Y por eso el diario cuesta **más** que el
histórico —el hexadecimal dobla los bytes: ~54 MB al día, ~20 GB al año por
testigo—. **Es la única versión que prueba algo.**

⚠️ **Comparar dos diarios detecta la divergencia, no dice cuál miente** —
*detectar no es distinguir*, otra vez. Pero no hace falta: lo que queda
probado es que **el operador emitió dos cosas distintas para el mismo
índice**, y eso ya es oponible.

⚠️ **Lo que esto NO cubre: un testigo que no existía no tiene diario**, y
ahí no hay nada que hacer. Es la limitación estructural del modelo entero
—Certificate Transparency la tiene igual— y va escrita **junto a la
decisión**, no en una nota aparte.

⚠️ Y **sin custodia declarada de la clave, lo que este método sirve no tiene
valor probatorio** (`SECURITY.md`). §242 hace que el artefacto exista; no
que valga.

### `zkssl_ackPath` — el camino de acuse de una época CERRADA

Devuelve lo que le falta al acuse de la respuesta (§274) para atarse a
una firma: `s` —el `seq` de la cabeza que cierra la época— y `camino`
(`{siblings, isRight}`) hasta la raíz de acuses de esa época.

⚠️ **La cabeza NO viaja** (§248): el camino se verifica contra la que el
titular **ya custodia**. Servirla aquí sería dejar que el operador
fabrique la vara con la que se le mide.

La cadena, entera: `hoja = acuse_digest(hashPrueba, epoca, n)` →
`path_root` → `acusesRoot` → `epoch_digest` **v2/v3** (según `formatVersion`) → la firma custodiada.
El desambiguador v1/v2 es el **byte de versión del preámbulo**
(`formatVersion` 1 → 2), no el dominio (§236).

⚠️ **Tres formas de `available: false`, y ninguna es un error genérico**
(la forma de §241): la época sigue **abierta** (`reason` +
`beatSeconds`); el nodo corre **sin `--diario`** — los límites de época
no se conservan, y tras un reinicio sin diario la primera época sale
gorda: límite declarado —; o esa entrada **no existe** en el registro
dentro de esa época.

⚠️ **Aditivo**: `zkssl/0.2` no sube. La cabeza gana `acusesRoot` y `n`;
`deny_unknown_fields` hace que un parser viejo **rompa en voz alta** —
el fallo honesto ya diseñado, no una lectura a medias.

### `zkssl_inclusionReceipt` — la inclusión, comprobable sin el nodo

Devuelve lo que un tercero necesita para comprobar que **una hoja estaba en
la cabeza firmada**: `index`, `leaf`, `path` y los campos de `head` —
**siete desde §275**; la `formatVersion` de la firma dice cuáles
componen.
La verificación es la de §256: subir el camino hasta `accountsRoot` y
comprobar que esos campos componen —según la versión declarada— el
`epochDigest` **de una cabeza firmada**. Una raíz suelta no prueba nada.

⚠️ **Del árbol `accounts`**, que es el que firma la cabeza. `CommitmentLayer`
solo se instancia en sus propios tests: un camino suyo llevaría a una raíz
**que nadie firma**.

⚠️ **`leafFormat` es una OBSERVACIÓN, no una afirmación.** El nodo compone la
hoja **de las dos formas** —`native_leaf` y `native_leaf_salted`— y declara
la que casó con el árbol. Si mintiera, el titular compondría mal y el recibo
**fallaría**: una forma equivocada **no puede hacer que un recibo falso
verifique**, solo que uno legítimo falle. Está para que el fallo sea
**legible** en vez de un `RaizDistinta` a secas.

⚠️ **Aditivo**: `zkssl/0.2` **no sube**. No cambia ningún valor que viaje.

### ⚠️ Lo que un camino expone: enumeración del saldo

**Esto NO es propio del recibo. Afecta igual a `zkssl_sendMaterials`, que
reparte `senderPath` sin credencial alguna desde antes que este método
exista.** Se dice aquí, bajo las dos puertas, porque decirlo solo bajo una
es cómo se llega a tener dos.

**El camino de acuses (§275) es distinto a propósito**: sus hojas son
**enumerables POR DISEÑO** — digests de acuses, no saldos ni operaciones
(la divergencia deliberada con §157 que documenta `acuses.rs`). Probar
hojas candidatas contra `acusesRoot` no expone nada que el registro
público no diga ya.

Con un camino de cuentas y la raíz, cualquiera puede probar hojas candidatas
**sin volver a hablar con el nodo**: compone `native_leaf(publicId, saldo,
nonce)`, sube el camino y compara. Las tres piezas que hacen el espacio
enumerable **son públicas**:

| pieza | de dónde sale |
|---|---|
| `publicId` | `zkssl_publicId`, **sin credencial** |
| cota del saldo | `regulatoryLimit`, que `zkssl_params` publica |
| nonce | contador que **empieza en cero** |

Un lector puede hacer la cuenta: el producto de esas dos cotas es el número
de hojas a probar, y **cada prueba son unos pocos hashes**. Con el
`regulatoryLimit` del nodo de referencia —500.000— y un nonce bajo, eso es
enumerable en una máquina cualquiera.

⚠️ **El salt de §117 es lo único que lo impide**, y su propia documentación
dice exactamente esto: protege «de terceros que ven caminos y pruebas». Se
deriva de la clave de gasto, así que un tercero **no lo tiene**, y multiplica
el espacio por el tamaño de la clave.

> ⚠️ **Por eso la migración de §117 no es una mejora: es la condición para
> operar en abierto.** Un ledger sin migrar que exponga `sendMaterials` o
> este método **publica los saldos** a quien quiera calcularlos.

### ⚠️ CORRECCIÓN (§265): el enunciado sigue en pie, el sujeto caducó

Lo de arriba **no era falso y no se retira**: un ledger sin sal que reparta
caminos publica los saldos. Lo que caducó es **de quién habla**.

| ledger | estado hoy |
|---|---|
| creado o persistido por este código | **salado**. `commit()` escribe `meta:geometry_v7` en cada persistencia (`persistence.rs:578`), y en caliente toda cuenta nueva sale salada |
| store legacy nunca migrado, anterior a D4 | **sigue expuesto**. Su herramienta es `migrate_to_salted_positions` (§157) |

**Cuándo se cumplió, con el commit delante**: el flip D4 en **§156**
(`2ac37c5`) —la capa habla salted de punta a punta y los legacy mueren— y la
colocación en **§157** (`e3116c6` + `f926de6`), donde `open_account` adopta
`public_id mod capacidad` y los dos contratos de privacidad
—`account_indices_are_not_predictable` y `_not_enumerable`— cobran. Etiqueta
`entrada-50`.

⚠️ **Lo que NO dice esta corrección**: que la condición esté cumplida *para
todo ledger*. Un store anterior a D4 abierto en abierto sigue publicando
saldos, y además **no vuelve a abrir** si se le crea una cuenta —§258-A—.
Las dos cosas y su herramienta viven juntas en **la nota 76 del BACKLOG**,
que es donde pueden marcarse.

> ⚠️ **Por qué esta frase duró cinco sellos.** Nació en §259 acompañada de
> «no es una entrada de backlog pendiente; es una condición de despliegue».
> Eso la sacó del único sitio del proyecto donde las cosas tienen estado. Una
> nota del BACKLOG se marca; una condición de despliegue escrita en un
> asiento **no tiene casilla**. Se ascendió de categoría y con eso se volvió
> incerrable.

### ⚠️ CORRECCIÓN (§261): lo anterior era falso

Esta especificación afirmaba, palabra por palabra:

> ⚠️ **`claimMaterials` no está en el mismo caso**: exige el `PendingNotice`
> del pagador. No es una credencial que el nodo compruebe, pero **hay que
> tenerlo**. La puerta abierta de par en par es una, no dos.

**Medido en §261: `claim_materials` (`client.rs:138`) usa del aviso
únicamente `notice.position`, y el despacho no lo valida.** Cualquiera podía
llamar con un aviso **inventado** y obtener el camino del receptor. **No
hacía falta ni un secreto ni un pago previo: eran dos puertas abiertas de
par en par, no una.**

### La credencial (§261)

`sendMaterials`, `claimMaterials` e `inclusionReceipt` exigen ahora la
`viewKey` de la cuenta **cuyo camino se devuelve** —el remitente, el
receptor y el índice pedido—, comprobada contra **ese** índice. Error
`-32004` si no cuadra.

⚠️ **No es un control nuevo: es uno que existía del lado equivocado.** El SDK
ya comprobaba «los materiales de cobro no corresponden a esta wallet»,
**después** de recibir los caminos. Un cliente que quite esa línea los
obtiene igual. **Una comprobación del lado del cliente no protege al
sistema: protege al cliente que la ejecuta.**

### ⚠️ Qué promete el recibo, antes y después

| | antes | después |
|---|---|---|
| **verificar** un recibo | cualquiera | **cualquiera** — igual |
| **obtener** el recibo de otro | cualquiera | solo el titular |

**No se pierde «comprobable por terceros»: se pierde «obtenible por
terceros», que era el agujero.** §256 afirmó que un tercero puede comprobar
una inclusión sin el nodo, y **eso no cambia**: el recibo es autocontenido y
`verificar_inclusion` sigue sin necesitar al operador. Lo que cambia es
**quién puede pedirlo** — y el titular puede reenviarlo a quien quiera, así
que la comprobación por terceros pasa a ser **por delegación** en vez de
abierta.

⚠️ Lo segundo **nunca fue una propiedad diseñada**: era **la ausencia de un
control**, y §259 midió lo que costaba.

⚠️ **Pérdida real, dicha como conjetura:** un testigo ya no puede comprobar
inclusiones de cuentas ajenas. Hoy tampoco lo hace —vigila cabezas, no
cuentas—, y en Certificate Transparency los monitores comprueban
consistencia del log, no inclusiones individuales. **Que por eso no importe
en la práctica es una conjetura, no un argumento medido.**

Cerrar la exposición —exigir credencial para los caminos— **es un sello
aparte**: cambia superficie hoy pública y puede romper clientes. Meterlo
aquí encadenaría «servir el recibo» con «cerrar `sendMaterials`», y si algo
se torciera no se sabría cuál de los dos fue.

## El transporte de la cofirma (§315)

`zkssl_submitCosig` y `zkssl_cosigs` son el camino que el eslabon 3
necesitaba y no tenia: hasta hoy un testigo cofirmaba una cabeza y la
escribia **en un fichero suyo**, donde nadie mas la veia. El nodo pasa a
ser el TRANSPORTE — que es lo que decidio la nota 83 — sin pasar a ser la
autoridad.

```text
Cosig = {v: Q, epochDigest: Digest, clavePublicaOperador: DATA,
         clavePublicaTestigo: DATA, versionFormato: Q, indice: Q,
         firma: DATA, vistoUnix: Q}
```

Son los ocho campos que el testigo ya escribe en su fichero JSONL, **en
convencion de cable**: alli `v` y `vistoUnix` van como numeros crudos y
aqui como QUANTITY. Son dos artefactos con dos convenciones, y se dice en
vez de disimularse.

⚠️ **Lo que el nodo comprueba y lo que NO.** Comprueba que la cofirma es de
la epoca EN CURSO y que **la firma cierra**: rechazar lo malformado es
barato y el motivo va nombrado. **No acredita al testigo**, y no puede: la
clave del testigo viaja en el propio objeto y nada la respalda, asi que
cualquiera puede firmar con la suya y esto pasara. **Quien decide que
testigos cuentan es el cliente, con su politica** — y tiene que ser asi,
porque el nodo es el operador y un operador que nombra a sus vigilantes no
esta vigilado.

⚠️ **Cotas de recurso, no de confianza.** El almacen guarda **una sola
cofirma viva por clave de testigo** —la serie es por testigo (§310), asi que
reenviar sustituye en vez de acumular— y **solo la epoca en curso**: cada
cofirma lleva su `epochDigest` y las viejas se descartan en la siguiente
submision. Encima va un tope duro, `--max-cofirmas`, porque cada cofirma
son ~37 KB. Pedir una epoca que no es la actual devuelve `n: 0`, y eso **no
es un error**: el nodo no guarda historico, igual que con la cabeza.

## La prueba de extension (§293)

`zkssl_consistencyProof` convierte el eslabon 2 en SERVICIO: quien
custodia una cabeza v3 pide el camino con su `mmrSize` como
`oldSize`, captura la cabeza cuyo `mmrSize` IGUALA al de la
respuesta — ⚠️ la pareja firmada es el acumulador ANTES de cada
cabeza, asi que esa es **la siguiente en emitirse**, a lo sumo un
latido despues (§248: la cabeza no viaja en el camino) — y verifica **sin el registro** que la cima
nueva EXTIENDE a la suya — consistencia O(log N) del arbol de historia
(§291).

Tres respuestas sin camino, cada una con su razon: `oldSize: 0`
(no hay historia que extender), `oldSize` igual al `mmrSize`
actual (identidad: camino vacio), y `oldSize` MAYOR que el actual —
**el acumulador va por detras**: un nodo que rearranco sin diario lo
dice en el cable, que es exactamente el reseteo VISIBLE que el formato
v3 promete.

**El paquete de extension** del verificador de terceros:
```text
{ "v": 1, "tipo": "extension", "vieja": {…}, "nueva": {…}, "camino": […] }
```
Las dos cabezas van TAL CUAL las sirvio `signedEpochHead` (ambas
**v3**: una v2 no lleva pareja que extender) y el mando exige ademas la
MISMA `publicKey`: la continuidad es de un firmante. La
demostracion: `tools/banco_extension.sh` — nodo vivo, paquete
capturado, verificacion sin el nodo, el camino adulterado en rojo, y el
rearranque sin diario respondiendo por detras.

## Apagado — el fin de vida, declarado (nota 91)

Toda capa de liquidacion termina — por cierre, por migracion o por
abandono. Lo que este protocolo declara para ese dia:

**Que publica el operador al cerrar**: nada que no este ya publicado.
Las dos respuestas que sostienen una posicion — `zkssl_signedEpochHead`
y `zkssl_ackPath` — viajaron en cada latido; un cierre ordenado solo
exige servirlas hasta el ultimo.

**Que se lleva el titular**: lo que ya custodia. La ultima cabeza
firmada (los siete campos con `publicKey`, `epochDigest`,
`formatVersion`, `index` y `signature`, juntos, del mismo latido), el
`hashPrueba` de su entrada (el `proofDigest` asentado, servido en el
acuse de la respuesta y en `zkssl_logEntry`), y el camino de acuse
(`s`, `siblings`, `isRight`).

**Con que sostiene su posicion despues**: el paquete de evidencia
portable — formato v1, declarado en la cabecera del binario de
`zk-ssl-verify` — las respuestas del cable TAL CUAL, reunidas en un
fichero y verificadas sin el nodo, sin la capa y sin el probador.

⚠️ Es lo contrario del fallo caracteristico de validium: aqui no hay
comite al que pedir los datos, porque el titular ya los tiene. **Nadie
queda dentro.**

**La demostracion, no la promesa**: `tools/banco_apagado.sh` levanta un
nodo real, fondea una posicion, captura el paquete por el cable, mata
el proceso con `kill -9` y verifica en VERDE — y en ROJO el mismo
paquete con un solo nibble adulterado. **Un sistema que dice como muere
se demuestra muriendo**, no prometiendo.

## Notas operativas

- Un nodo, un escritor: las escrituras serializan en el nodo (el orden
  de operaciones es del operador por diseño; el consenso distribuido es
  otro problema y no está implementado).
- Parámetros de un ledger persistido: inmutables
  (`ParameterMismatch` al reabrir con otros valores).
- Versionado: `zkssl_protocolVersion` gobierna compatibilidad. **La
  versión vigente es `zkssl/0.2`** desde §209, y lo que la sube es que
  cambien los **valores que viajan**, no el tamaño de la superficie:
  añadir un método de forma aditiva —como `zkssl_applyMany` en §222— no
  la sube, porque los vectores de conformidad no se mueven.
