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

- HTTP `POST /`, cuerpo JSON-RPC 2.0. Un objeto por petición (los lotes
  no están soportados en v0.1).
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

## Notas operativas

- Un nodo, un escritor: las escrituras serializan en el nodo (el orden
  de operaciones es del operador por diseño; el consenso distribuido es
  otro problema y no está implementado).
- Parámetros de un ledger persistido: inmutables
  (`ParameterMismatch` al reabrir con otros valores).
- Versionado: `zkssl_protocolVersion` gobierna compatibilidad; cambios
  incompatibles suben a `zkssl/0.2`.
