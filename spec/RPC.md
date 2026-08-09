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
| `zkssl_epochHead` | — | `{seq, accountsRoot, pendingRoot, frozenRoot, chainDigest, epochDigest}` |
| `zkssl_supply` | — | `{total, pending: Q}` |
| `zkssl_accountCount` | — | `Q` |
| `zkssl_publicId` | `{index: Q}` | `Digest` |
| `zkssl_accountView` | `{index: Q, viewKey: Digest}` | `AccountView` (autenticada; `AccountNotFound` si la clave de vista no corresponde) |
| `zkssl_logEntry` | `{seq: Q}` | `LogEntry` |
| `zkssl_logEntries` | `{fromSeq?: Q, limit?: Q}` | `LogEntry[]` (límite ≤ 1000) |
| `zkssl_verifyChain` | — | `{ok: bool, entries?, error?}` |

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
| `zkssl_sendMaterials` | `{sender: Q, receiverId: Digest, amount: Q, salt: Digest}` | `SendMaterials` |
| `zkssl_applySend` | `{receipt: SendReceipt, sender: Q, senderState: ClientState, amount: Q}` | `Applied` |
| `zkssl_claimMaterials` | `{receiver: Q, notice: PendingNotice}` | `ClaimMaterials` |
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
4. **`proofDigest` ata la entrada a una prueba concreta** —a sus bytes—,
   aunque el registro no guarde la prueba.

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

### `zkssl_signedEpochHead` — la última cabeza firmada, para un TESTIGO

Devuelve la cabeza de época **más reciente que el nodo firmó**, con todo lo
que hace falta para verificarla sin él: `publicKey`, `epochDigest`,
`formatVersion`, `index` y `signature`.

⚠️ **Aditivo**: no toca `zkssl_epochHead`, que sigue sirviendo la cabeza
**sin firma** y está en los vectores de conformidad. **La versión no sube**,
por la misma razón que no subió con `zkssl_applyMany`.

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
