# Arqueo JSON-RPC — specification v0.5 (`zkssl/0.5`)

> **Español:** [`spec/RPC.md`](./RPC.md) is the original of this page, in Spanish; it prevails
> wherever the two differ. This translation is of commit `7766efb` (audit entry §917) and was
> produced by the project's assistant, a Claude Code session, in audit entry §918 (`GENAI.md`): it
> is a generated artefact, not text written by the author. Line citations of the form `file:N` refer
> to the Spanish files.

⚠️ **`zkssl/0.4` does not identify which proofs a node accepts** (§699): the proofs changed without
raising it in §641, §680 and §684, and so it stays until the `zkssl/0.5` cut; and
`vectors/zkssl-0.2.json` was re-issued three times under its version. The detail, in «Versioning»
(§«Operational notes»). ⚠️ §723: and in §723, already under the promise of «Versioning and
compatibility», those of the band and the pending claim, which no node accepts but the kit does in
its envelopes: they move to their v2, which 0.4.3 does not verify, and the same cut pays for it,
like E2 of RFC-0017 (§826).
⚠️ **§914, the cut of the train (RFC-0018 D-G and D-H): `zkssl/0.5`.** Every proof carries the v2
mark with its family, the version of its AIR and its profile, and the judge reads it before anything
else; two paired random columns in DEEP (RFC-0009 D-AL bis) and T ≥ 128 in all the families; kit
0.5.0 rejects by name the proofs of `zkssl/0.4` and earlier (D-E; R6 e of RFC-0021). What the
paragraph above says of `zkssl/0.4` is history: since `zkssl/0.5` the mark identifies the proof.
> **What changed from `0.1` to `0.2` (§209, stage 1 of RFC-0002).** A single
> change, and it is not in the methods: **`proof_digest` is no longer computed
> with an algebraic hash**. With it `chain` and the epoch head change,
> which are wire values. No method, field or type is added,
> removed or renamed.
>
> **The `zkssl/0.1` vectors are kept** in `spec/vectors/` and
> still describe that version: an implementation of `0.1` can still
> be checked against them. `conformance --check` **refuses** to
> validate vectors of a version other than its own, and that is correct.
>
> Reason, measured in `AUDITORIA.md` §204: that hash was **93 %** of the
> cost of applying an operation —30.99 ms of 33.27— for a digest that
> **enters no circuit**.

Normative specification of the node API. `zk-ssl-node` is the
reference implementation; `zk-ssl-wire` defines the wire types.
Any implementation that speaks this is a valid Arqueo node as far
as the ecosystem's tools are concerned. (Analogous to Ethereum's
`execution-apis`; an OpenRPC version generated from `zk-ssl-wire` is on the
roadmap.)

## Transport and envelope

- HTTP `POST /`, JSON-RPC 2.0 body. **One object per request**: the
  *JSON-RPC batch* (an array of requests in one body) is not accepted.
  ⚠️ **This has nothing to do with `zkssl_applyMany`.** That one is a
  **batch of OPERATIONS** inside ONE request; this is a batch of
  REQUESTS, and it is not accepted. The same word for two things has
  caused confusion since v0.1, and that is why it is separated here.
- **Body size**: the transport's default limit is
  **2,097,152 bytes** (§218, measured). An operation with a proof is today around
  **159 KB** in the body, so a `zkssl_applyMany` holds **13**:
  measured in §615, the batch of 13 weighed 2,064,578 and 2,076,308 bytes in two
  runs -up to 99.0 % of the limit, because the size of the proof varies- and the
  one of 14 is rejected with 413. Until §615 this said 15, with the 132,728 of
  §218: the proof grew with the hiding of §538, and nobody measured again.
- Response: `{"jsonrpc":"2.0","id":…,"result":…}` or
  `{"jsonrpc":"2.0","id":…,"error":{"code":…,"message":…}}`; a rejection by the layer (`-32000`)
  also carries `data`, its cause as data (section «Errors», §454).

## Encoding (the most widely accepted convention, adapted)

| type    | form                                    | example |
|---------|-----------------------------------------|---------|
| QUANTITY| u64 in hex with `0x`, no leading zeros. | `"0x3d090"` |
| DATA    | bytes in hex with `0x`, even length     | `"0x1f8b…"` |
| Digest  | 32-byte DATA, **the same serialization the layer persists** (`store::digest_to_bytes`) | `"0x9a41…"` (64 hex) |

Deserialization validates canonicity: a digest whose elements do not
belong to the field is rejected with `-32602` before touching the layer.

## The principle the API preserves

**The spending key never travels.**

⚠️ **Between §521 and §538 the implementation did not honour this principle**:
the key did not cross the API, but the send and claim proofs
published it in the clear —winterfell 0.13 did not hide the witness— and the node
received them. **Since §538 (RFC-0009 E3b-2, `zkssl/0.4`) it is honoured in what
the E2 suite measures**: the prover hides the witness and the key does not come out
literally in any proof with a row; what is promised and what is not is said by
RFC-0009 (D-A, D-C). See `SECURITY.md`.

- Opening an account = sending identifiers derived on the client:
  `publicId`, `viewId`, `leafSalt`.
- Paying/claiming = asking for materials (paths and roots: public), **proving
  locally**, presenting the receipt.
- Reading one's own balance = presenting the **view key** (derived,
  it only authorizes reading that account; 49-A).

## Methods

### Reading

| method | params | result |
|---|---|---|
| `zkssl_protocolVersion` | — | `"zkssl/0.5"` |
| `zkssl_params` | — | `{regulatoryLimit, maxSupply, maxAccounts: Q, custodianRoot, governanceRoot: Digest, refundTtl, maxCustodianUses: Q}` |
| `zkssl_epochHead` | — | `{seq, accountsRoot, pendingRoot, frozenRoot, chainDigest, acusesRoot, n, mmrRoot, mmrSize, consRoot, consCount, paramsDigest, pmetaRoot, nextPending, nextIndex, totalSupply, recepRoot, recepCount, epochDigest}` |
| `zkssl_supply` | — | `{total, pending: Q}` |
| `zkssl_accountCount` | — | `Q` |
| `zkssl_publicId` | `{index: Q}` | `Digest` |
| `zkssl_accountView` | `{index: Q, viewKey: Digest}` | `AccountView` (authenticated; `AccountNotFound` if the view key does not correspond) |
| `zkssl_logEntry` | `{seq: Q}` | `LogEntry` |
| `zkssl_logEntries` | `{fromSeq?: Q, limit?: Q}` | `LogEntry[]` (limit ≤ 1000) |
| `zkssl_verifyChain` | — | `{ok: bool, entries?, error?}`; at most once every 60 s for everyone, otherwise `-32005` (§678) |
| `zkssl_inclusionReceipt` | `{index: Q, viewKey: Digest}` | `{index, leaf, path, leafFormat, head}` |
| `zkssl_ackPath` | `{seq: Q}` | `{available, s?, camino?: {siblings: Digest[], isRight: bool[]}, reason?, beatSeconds?}` |
| `zkssl_consistencyProof` | `{oldSize: Q}` | `{available, mmrSize: Q, camino?: Digest[], reason?}` |
| `zkssl_submitCosig` | `{cosig: Cosig}` | `{accepted: bool, stored: Q, reason?}` |
| `zkssl_cosigs` | `{epochDigest?: Digest}` | `{epochDigest, n: Q, cosigs: Cosig[]}` |
| `zkssl_publishConsumo` | `{consumo: Digest}` | `{accepted: bool, logSeq?: Q, reason?}` |
| `zkssl_consumoPath` | `{consumo: Digest, seq: Q}` | `{available, s?: Q, camino?: {siblings: Digest[], isRight: bool[]}, reason?}` |
| `zkssl_frozenPath` | `{index: Q, viewKey: Digest}` | `{s: Q, index: Q, leaf: Digest, camino: {siblings: Digest[], isRight: bool[]}}` (holder's credential, §261) |
| `zkssl_pendingPath` | `{index: Q, viewKey: Digest, position: Q, salt: Digest, amount: Q, x: Digest, receiverId?: Digest}` | `{available, s?: Q, caminoPendiente?: {siblings: Digest[], isRight: bool[]}, hermanosMeta?: Digest[], emisor?: Q, nacido?: Q, reason?}` (receiver's credential, §261; RFC-0008 D-F; with `receiverId`, the PAYER's, which names the receiver: RFC-0008 D-AE, §505) |
| `zkssl_recepPath` | `{rx: Q}` | `{available, recepCount?: Q, index?: Q, camino?: {siblings: Digest[], isRight: bool[]}, reason?, beatSeconds?}` (RFC-0010 E3, §571) |
| `zkssl_keyActs` | — | `{actas: ActaFirmada[]}`, from the genesis; empty if the node does not sign acts (RFC-0015 E3b-2, §646) |

`LogEntry = {seq: Q, kind: string, rootOld, rootNew, proofDigest, chain: Digest, compromiso?: Digest}`
with `kind` ∈ {`OpenAccount`,`Mint`,`Transfer`,`Burn`,`Recovery`,
`Governance`,`Freeze`,`Send`,`Claim`,`MintToPending`,`Migration`,`Refund`,`Consumo`}
(the layer's `OpKind`; they say which circuit verifies the entry).
⚠️ §797: until here it said `LogEntry = {seq: Q, kind: string, rootOld, rootNew, proofDigest, chain:
Digest}`, and the list of `kind` ended at `Refund`. It lacked two things the wire serves:
`compromiso`, since §281, which goes in every era-2 entry and is missing from the era-1 ones
(`LogEntryDto`, `crates/zk-ssl-wire/src/lib.rs:545-561`, `skip_serializing_if`) and without which the
chain of an era-2 entry cannot be recomposed; and `Consumo`, since §413 (RFC-0006 E1). The
first was already pointed out by `doc/ecst/ECST.md:761-764` (§5.4). How the other six fields and
the previous entry's `chain` compose `chain`, in «The composition of `chain`, byte by byte».

### Opening

| method | params | result |
|---|---|---|
| `zkssl_openAccount` | `{publicId, viewId, leafSalt: Digest}` | `{index: Q}` |

The account is born with balance ZERO by design. The client derives the three
identifiers with `stark_experiment::native::{derive_public_id_wide,
view_id_of_wide, derive_leaf_salt_wide}`.

⚠️ §802: an identity opens at most one account in the life of the ledger: a `publicId` that already
has an account, or that a recovery retired, is rejected with `-32000` and the cause
`IdentityUnavailable`, with no fields and without touching `nextIndex`, the root or the log. Until
§802 a second account was opened with the same identity (`SECURITY.md` §3.13). ⚠️ §863: a vector
with a new path shows it, `vectors/nodo/abrir-identidad-con-cuenta.json`, as R6 f of RFC-0021
asks —the `code`, the cause with no fields, the `seq` and that no other account is opened;
`nextIndex`, the root and the log, which the wire only gives in a whole `zkssl_epochHead` and in
the entries, it does not pin: the layer ties them—; the canon judges it with a node test that sends
each request through the transport's function (entry 90 of `BACKLOG.md`).

### Two-phase payment

| method | params | result |
|---|---|---|
| `zkssl_sendMaterials` | `{sender: Q, viewKey: Digest, receiverId: Digest, amount: Q, salt: Digest}` | `SendMaterials` |
| `zkssl_applySend` | `{receipt: SendReceipt, sender: Q, senderState: ClientState, amount: Q}` | `Applied` |
| `zkssl_claimMaterials` | `{receiver: Q, viewKey: Digest, notice: PendingNotice}` | `ClaimMaterials` |
| `zkssl_applyClaim` | `{receipt: ClaimReceipt, receiver: Q, receiverState: ClientState, notice: PendingNotice}` | `Applied` |
| `zkssl_applyMany` | `{ops: BatchOp[]}` | `BatchApplied` |

- `Applied = {logSeq: Q, kind, accountsRoot, chain: Digest}` — the
  entry that was left in the chained log.
- `SendReceipt.proof` / `ClaimReceipt.proof` are
  `winterfell::Proof::to_bytes` (~54–66 KB depending on the circuit in the current configuration:
  the measured price of depending on no ceremony).
- `PendingNotice = {position: Q, salt: Digest, amount: Q}` and **travels
  out of band** from the payer to the receiver (ISO 20022 does not carry it;
  AUDITORIA §39; ⚠️ §861: it said «AUDITORIA §21», which deals with the reason codes, and «does not
  carry it» is not exact: there is no field made for the notice, and the extension one,
  `SplmtryData`, admits it —read in a model derived from the XSDs, not checked against the
  original—, while using the remittance one would be forcing it; the channel was decided by
  delegation in entry 11 of `BACKLOG.md` and is still unbuilt, and this document does not change
  because of it). The RPC never hands it to third parties.
- The payment **is not final until the claim**; without a claim, the amount stays
  immobilized until claim or refund (§29/§30).

### `dev_*` — only with `--dev` (TEST custodians)

| method | params | result |
|---|---|---|
| `dev_fund` | `{index, amount: Q}` | `Applied` + `custodianNullifiers: Digest[2]` |
| `dev_openSeeded` | `{seed: Q}` | `{index: Q, publicId, viewKey: Digest}` |
| `dev_freeze` | `{index: Q, frozen: bool}` | `Applied` |

`dev_fund` is the sandbox's faucet: REAL delegated issuance with two
custodians of the suite, including the threshold nullifiers they consume
(`circuit_threshold_single_nullifier`). A production build is
compiled without the `dev` feature and does not contain this namespace.

`dev_freeze` is its twin for freezing (§458): the REAL delegated path —the climb of the frozen
tree and the same two custodians of the suite—, and only over an account that exists. It is
there so that a bank with a real node can provoke `AccountFrozen` and capture its path.

## Errors

| code | meaning |
|---|---|
| `-32601` | unknown method (or `dev_*` disabled) |
| `-32602` | invalid parameters / non-canonical encoding |
| `-32000` | rejection by the layer: `message` = `LayerError` (e.g. `InsufficientBalance{…}`, `StaleState`, `OverRegulatoryLimit{…}`, `AccountFrozen(…)`); since §454, `data` = its cause (below) |
| `-32004` | invalid credential for the requested account (§261, «The credential») |
| `-32005` | rate limit of an expensive read (§678): `data.retryAfterSeconds` (Q) says how long remains. Today only `zkssl_verifyChain`, which walks the whole log with the state lock taken |
| `-32603` | internal node failure: a poisoned lock, a serialization, the reception counter. ⚠️ §815: and the PARADA [halt] (§530, §655), which this row did not name: a `-32603` whose `message` starts with «nodo en PARADA: », with its cause. From it on the node serves no method, and since §815 it does not sign heads either (`SECURITY.md` §3.15) |

`StaleState` is to be expected under concurrency: the declared state fell
behind. The client refreshes its view and retries.

### The cause of a rejection, as data (§454, RFC-0007 E2)

Since §454 a rejection by the layer (`-32000`) carries, besides `message`, a `data` with its
cause. The example is that of a payment without balance:

```text
"error": { "code": -32000,
           "message": "InsufficientBalance { available: 10, requested: 999999 }",
           "data": { "causa": "InsufficientBalance",
                     "campos": { "available": "0xa", "requested": "0xf423f" },
                     "seq": "0x…" } }
```

- **`causa`** is the name of the `LayerError` variant, as is. **`campos`** are its own,
  with this document's encoding: QUANTITY for a number, `Digest` for a digest and
  text as is for `detalle`. The five TUPLE variants have no field name in the
  code: this catalogue gives it (`index` for an account, `detalle` for a text).
- **`seq`** is the height of the log against which it was judged: the same one the head
  publishes (`seq = log.len()`), read under the same lock as the rejection. Whether a SIGNED head
  exists at that height depends on the heartbeat; that is the business of the rejection with
  proof (E3), not of this datum.
- **`message` does not change**: it is still the `Debug` of `LayerError`, with `[receptionSeq=…]`
  if there is one. The other codes carry no `data` and come out with the two keys as always.
- **`zkssl_publishConsumo` rejects INSIDE `result`** (`accepted: false`, `reason`): since
  §454 its refusal carries the same `data`, with `ConsumoRepetido` or `ConsumoColision`. ⚠️ §815: a
  store failure (`Store`) no longer travels this way. Until §815 it came out as one more refusal,
  and the node kept serving and signing with the memory ahead of the disk; since §815 it is the
  PARADA, as in the other methods since §655.
- **It is not opposable evidence**: it is what the node says, like `receptionSeq` (§253). The
  proof of the cause is E3 and E5 of RFC-0007 (`spec/PAQUETE.md`, section 2.6).

The catalogue is produced by the layer —`LayerError::causa`, an exhaustive `match`— and a test
ties it to this table (`tests_causa`, in `crates/zk-ssl/src/lib.rs`): a new variant does not
compile without its cause nor pass without its row. `Store` is an operator failure, not a rule.
Which causes the wire emits today, and through which methods, is measured in entry §454 of
`AUDITORIA.md`, without a gate; ⚠️ §802: §802 adds `IdentityUnavailable`, emitted by
`zkssl_openAccount` and `dev_openSeeded`.

| cause | fields |
|---|---|
| `RefundTooEarly` | `born`, `now`, `ttl` |
| `RefundUnavailable` | — |
| `PendingMismatch` | — |
| `AccountNotFound` | `index` |
| `InsufficientBalance` | `available`, `requested` |
| `OverRegulatoryLimit` | `limit`, `requested` |
| `PendingTreeExhausted` | `capacity` |
| `DuplicateAccountInBatch` | `index` |
| `DuplicatePendingInBatch` | `position` |
| `NotTheIssuer` | — |
| `CustodianSetExhausted` | `uses`, `max` |
| `ProofFailed` | `detalle` |
| `VerificationFailed` | `detalle` |
| `StaleState` | — |
| `WrongRegulatoryLimit` | `expected`, `declared` |
| `Store` | `detalle` |
| `NotTheAccountHolder` | — |
| `BalanceOutsideBand` | `lower`, `upper` |
| `SupplyCapExceeded` | `cap`, `wouldBe` |
| `RecoveryToSameIdentity` | — |
| `AccountLimitReached` | `limit` |
| `AccountFrozen` | `index` |
| `AlreadyInThatFreezeState` | — |
| `ConsumoRepetido` | `consumo` |
| `ConsumoColision` | `consumo`, `ocupante` |
| `IdentityUnavailable` | — |
| `AutorizacionConsumida` | `compromiso` |

## What the transition log asserts

This section is **normative** and describes what `zkssl_logEntries` and
`zkssl_verifyChain` guarantee today, and what they will stop guaranteeing if the
node applies operations **in batches** (RFC-0002, stage 2). It is written
**before** implementing it so that nobody builds on a guarantee
that is going to change.

### What the log asserts ALWAYS

1. **The sequence numbers are consecutive** from zero.
2. **`rootOld` of an entry is `rootNew` of the previous one**, and the first
   starts at the genesis root. This is what prevents inserting or deleting
   operations from the middle.
3. **`chain` is the chained digest** of the entry and of everything
   before it: altering any field of any entry breaks the chain
   from there to the end.
4. **`proofDigest` ties the entry to what AUTHORIZED it**, and the domain
   of the seal says which class it belongs to (§278): to the **bytes of the
   proof** where there is one (§116); to the **threshold commitment** on the
   delegated paths; or to the **declared absence**, with its own domain,
   where there is no proof by design. The log does not keep the bytes of
   the proof; since §281 the era-2 entry does carry the threshold
   commitment, in `compromiso`. The seals tell them apart. (⚠️ §797: until
   here it said «The log keeps none of the three», and since
   §281 the entry carries the commitment; the formula, in «The composition
   of `chain`, byte by byte».)

These four hold with batches or without them. `zkssl_verifyChain`
checks them.

**The issuance counter.** Since §716 (the resending of a delegated
issuance), the threshold commitment of the two issuances, `Mint` and
`MintToPending`, carries behind the cap the issuance counter,
`(n, n + 1)`, as those of freezing, governing and recovering carry theirs.
`n` is the number of `Mint` and `MintToPending` entries of the log
prior to the issuance, of the two classes together. No method serves
it: whoever reads `zkssl_logEntries` from `seq` 0 derives it, and it enters
as `(n, n + 1)` in what the custodians sign. With it, an
authorization issues only once, even if the root and the supply go back
to what they were before; resending it is, in the layer, `AutorizacionConsumida`. The
issuances recorded before this change do not carry it, and the log does not
say from which entry it carries it.

### The composition of `chain`, byte by byte (§797)

⚠️ **This documents the WIRE; it does not widen the core.** `chain` is a value of the
transition log, and the log is the LOG class of
`spec/NUCLEO.md` §«2. Las cuatro clases» [the four classes]: the node and the client compose it
over the wire, and that is why it stays outside the frozen core (RFC-0005, D-A: «the
core is the verifier's side, not the wire»). What follows is for a
**second implementer of the client** who wants to recompose what
`zkssl_logEntries` serves without trusting the node, and it does not enter the package. What stays
outside the core is the **composition**: `chain_digest`, `chain_digest_v2` and
`OpKind::tag` live in `crates/zk-ssl/src/log.rs` (:250, :281, :130), which the
census of `NUCLEO.md` (§«4. El censo» [the census]: `zk-ssl-verify` and `zk-ssl-hash`) does not
walk, and they have no KAT in `spec/vectors/nucleo/`. The **pieces** are there:
`native_merge`, `as_digest`, `embeber`, `digest_to_bytes` and `digest_from_bytes`
are CORE, with a row (`NUCLEO.md:93`, `:84`, `:92`, `:86`, `:85`) and a KAT
(`spec/vectors/nucleo/native_merge.json`, `as_digest.json`, `embeber.json`,
`digest_to_bytes.json`, `digest_from_bytes.json`); `digest_of_proof`, the two
seals and `COMPROMISO_AUSENTE` have a row of the LOG class (`NUCLEO.md:233-235`
and `:226`) and have no KAT. ⚠️ §862: those two citations no longer landed in their place, and the
three rows §862 adds to the FIRMA family of the census shift them three more: today
`digest_of_proof` is at `:249`, the two seals at `:251-252` and `COMPROMISO_AUSENTE` at `:242`.

Until §797 no text in `spec/` gave the formula: rule 3 above
says it in prose. Outside `spec/` it was written with the two eras, but only
the tree of merges, neither the table of tags nor the wire's spelling, in
`doc/ecst/ECST.md` §5.4 (:724-735; of the tags it says only «`tag` es el u64
de `OpKind` (1 a 13)» [`tag` is the u64 of `OpKind` (1 to 13)], :733) and in finding L1a of
`doc/ecst/VERIFICACION.md` (:993, which says the same of `tag_i`, and its sources in :994), and the
era-2 wrapping in the entry of §281 (`AUDITORIA.md:21146`).

`doc/ecst/ECST.md` is a dated report: its base is `main` at 71c5aad
(`ECST.md:8-9`), and it is not marked. It still says that «Nadie recompone la cadena por un
tercero» [nobody recomposes the chain for a third party] (:761-764; in other words, :54, :974 and
:1252-1253) and that this file describes `LogEntry` without `compromiso` (:762-764 and
:1181-1182), and the same in English, `doc/ecst/ECST_EN.md` (:768-771 and :1199-1200; :59,
:985-986 and :1273). Since §797 that is not so: `LogEntry` carries `compromiso` (above, in
«Reading»), and `tools/segunda/juez_cadena.py` recomposes the chain outside the
node, from a file with the entries. What the judge does not change: the proofs
are still not kept, and it does not compare the last `chain` with a signed head
(what was measured, at the end).

⚠️ **And `conformance --check` does not check this.** The conformance scenario
runs the prover again, and since §538 it declares `proof_digest`, `chain` and
`epoch_digest` non-reproducible (RFC-0009, D-AI): with hiding,
the bytes of a proof change on every issuance, and with them everything that
drags them along. What is checked without the prover is the **composition**: with
the entries as the conformance vector brings them (`Entrada`,
`crates/zk-ssl-cli/src/conformance.rs:38-51`, with the keys in snake_case) or
as `zkssl_logEntries` serves them (`LogEntryDto`, in camelCase), with their
`proofDigest`, `chain` is pure arithmetic. The two forms write the same
bytes (`hexd` and `q`, `conformance.rs:65-71`; `B32` and `Q`,
`crates/zk-ssl-wire/src/lib.rs:61-67`, `:84-88` and `:107-111`), and
`tools/segunda/juez_cadena.py` reads both (what was measured, at the end).

**The pieces**, from `crates/zk-ssl-hash/src/lib.rs` except the tag, which is
from `log.rs`, and the same ones `NUCLEO.md` section 6 uses:

- `merge(l, r)` is `native_merge` (`lib.rs:100`): a Rescue-Prime state
  `Rp64_256` of twelve elements at zero, `l` in `[4..8]` and `r` in `[8..12]`, one
  permutation, and the output is `[4..8]`.
- `as_digest(x)` is `[x, 0, 0, 0]` (`embeber`, `lib.rs:126` and `:144`), for a
  `u64` `x < p`.
- A wire `Digest` is its four elements, each in eight bytes
  little-endian (`digest_to_bytes`, `lib.rs:1191`; «Encoding»), and it is read
  with the canonicity of RFC-0016: each element `< p`, and the text in the canonical
  hex, `0x` and `[0-9a-f]` in pairs (`hex_canonico`, `lib.rs:1135`).
- `etiqueta(kind)` is `OpKind::tag` (`crates/zk-ssl/src/log.rs:130`), and the
  text of `kind` is the name of the variant (`LogEntryDto`:
  `format!("{:?}", kind)`, `crates/zk-ssl-wire/src/lib.rs:567`):
  `OpenAccount` 1, `Mint` 2, `Transfer` 3, `Burn` 4, `Recovery` 5,
  `Governance` 6, `Freeze` 7, `Send` 8, `Claim` 9, `MintToPending` 10,
  `Migration` 11, `Refund` 12, `Consumo` 13. A `kind` that is not in the
  list has no tag and the entry is not recomposed. ⚠️ Against a vector
  only four are measured: 1, 2, 8 and 9 (what was measured, at the end).

**Era 1**: an entry without `compromiso`, prior to §281 (`chain_digest`):

```
cabecera = merge(as_digest(seq), as_digest(etiqueta(kind)))
raices   = merge(rootOld, rootNew)
cuerpo   = merge(cabecera, raices)
chain_v1 = merge(merge(cuerpo, proofDigest), anterior)
```

**Era 2**: an entry with `compromiso`, which is every entry since §281
(`chain_digest_v2`), and it is the whole era-1 one, as a subtree, plus the
commitment:

```
chain_v2 = merge(chain_v1, compromiso)
```

- `anterior` is the `chain` of entry `seq - 1`. For `seq = 0` it is the
  zero digest, `[0, 0, 0, 0]` (`TransitionLog::head` of an empty log),
  which is also the `chainDigest` of the head of a node with no entries.
- **The era is decided by the presence of the field, entry by entry**
  (`verify_chain` and `verify`, `match e.compromiso`): a mixed log is
  verified with the formula of each entry. There is no version byte. On disk
  the era is told apart by the length of the entry (`LogEntry::compromiso`);
  on the wire, by whether the `compromiso` key brings a value: `LogEntryDto`
  declares it `Option<B32>` with `default` (`crates/zk-ssl-wire/src/lib.rs:559-560`)
  and reads the absent key the same as `"compromiso": null`, both as era 1.
  The node never writes the `null` (`skip_serializing_if`), and the judge gives it
  red (R7): it is stricter than the DTO.
- ⚠️ **The «v2» of the chain is not the v2 commitment of RFC-0003.** The one of
  RFC-0003 is the pending leaf's (`pending_commitment`, with its Δ per
  payment). The chain's is the **authorizing commitment** of the entry
  (§281): the `commit_operation` against which a delegated path was verified,
  the consumption in a `Consumo`, or the sentinel `COMPROMISO_AUSENTE` in everything
  else. The one of RFC-0003 only reaches the chain through the roots and
  the proof.
- The `chain` of an `Applied` is that of ITS entry; `batch.chain` is that of the
  last entry of the batch (`log_head`); the `chainDigest` of
  `zkssl_epochHead`, that of the last entry of the log.

**What each class records**: what `proofDigest` and `compromiso` carry (according to
the callers of `append` and `append_con_compromiso` in `crates/zk-ssl/src/`),
and whether a client can recompose them without the proof.

| `kind` | `proofDigest` | `compromiso` (era 2) | recomposed without the proof? |
|---|---|---|---|
| `OpenAccount` | `digest_of_proof(sello_sin_prueba())` | `COMPROMISO_AUSENTE` | yes: it is a constant |
| `Mint`, `Freeze`, `Recovery`, `Governance`, `MintToPending` | `digest_of_proof(sello_de_autorizacion(compromiso))` | the `commit_operation` that authorized them | yes, from the `compromiso` of the entry itself |
| `Send`, `Claim`, `Burn`, `Refund` | `digest_of_proof(bytes de la prueba)` | `COMPROMISO_AUSENTE` | **no**: the log does not keep the proof, and since §538 its bytes are not reproduced (D-AI) |
| `Migration` | `digest_of_proof(payload)` (`migration.rs`) | `COMPROMISO_AUSENTE` | **no** from the entry: the payload does not travel |
| `Consumo` | ⚠️ `digest_of_proof([])`, the EMPTY proof (`consumo.rs`, `apply_consumo`) | the published consumption | yes: it is a constant |
| `Transfer` | — | — | it has no caller in the layer; only the tests of `log.rs` record it |

⚠️ **The `proofDigest` column holds for the entries since §278, and the
`compromiso` one, since §281 (era 2).** Before, the `proofDigest` depends on
when it was recorded, and an era-1 entry does not say when: until §209
`digest_of_proof` was an algebraic hash, Rescue by blocks of 16 bytes
(`AUDITORIA.md:14147-14150`), and that is what `zkssl-0.1.json` brings; from §209 to
§278, `OpenAccount` and the five delegated ones recorded the empty proof,
`digest_of_proof([])` (`AUDITORIA.md:20888-20893`). From §278 to §281 the column
already holds, but an era-1 delegated one does not carry in the entry the commitment it
was sealed with, and it is not recomposed from the entry. The exception is `Recovery`: its
commitment ties the two roots of the entry and the recovery counter
(`crates/zk-ssl/src/recovery.rs:141-145`), and the kit recomposes its `proofDigest`
with the whole log from `seq = 0`, counting the previous `Recovery` entries
(`reverificar`, `crates/zk-ssl-verify/src/reverificacion.rs:18`, `:38`, `:119`
and `:179-198`; read and not run). The judge does not do it, and declares it too.
`Consumo` is from §413 and is always era 2. That is why the judge applies the table
only to era 2, and in era 1 declares it.

- `digest_of_proof(b) = Blake3(b"ZK-SSL-proof-digest-v2" ‖ len(b) en u64 LE ‖ b)`,
  and its 32 bytes are read as four `u64` LE **reduced** modulo `p`
  (`resumen_con_dominio`, the mould of §116: a producer reduces).
- `sello_sin_prueba() = b"ZK-SSL-no-proof-by-design-v1"` and
  `sello_de_autorizacion(c) = b"ZK-SSL-authorization-seal-v1" ‖ digest_to_bytes(c)`.
- `COMPROMISO_AUSENTE = [0xC0A92281; 4]`, a declared value and not a hash; on
  the wire, `0x8122a9c000000000` four times.

⚠️ **`Consumo` records the empty proof, not the seal of declared absence.**
§278 separated «there is no proof by design» (`sello_sin_prueba`) from «the proof
was forgotten» (`digest_of_proof([])`), and rule 4 above gives it
separately. `Consumo` (RFC-0006 E1, §413) came later and records the empty one. The
entry is still tied, because the consumption goes in `compromiso` and enters the
chain; but its `proofDigest` is the same as that of a forgotten proof. It is
written here as what the code does, not as a rule. And `reverificar`
(`crates/zk-ssl-verify/src/reverificacion.rs`), **read and not run**, would
reject it with `SelloDiscrepante`: it sees a commitment that is not the sentinel and
expects the authorization seal. No wire vector brings a `Consumo`
entry, so this row is unexercised.

**What was measured (§797).** `python3 tools/segunda/juez_cadena.py` recomposes the
six chains of `spec/vectors/zkssl-0.4.json` from their entries: **six of
six**, all era 2. It also recomposes four of its six `proof_digest`,
the two `OpenAccount` and the two `Mint`. Those of `Send` and `Claim` (seq `0x4` and
`0x5`) are left **declared** as non-recomposable (D-AI). With the four
wire vectors (from `zkssl-0.1` to `zkssl-0.4`) twenty-four of
twenty-four come out, and the six of `zkssl-0.1.json` are era 1: the only ones that
exercise `chain_v1`. The six of `zkssl-0.4.json` rewritten as the wire serves
them, a `LogEntry[]` list in camelCase, also come out six of six. The
judge reads the text with the wire's canonical spelling (`hex_canonico` and
`cantidad_canonica`: an upper-case letter, a `0x١` or a `0x1_0` are red) and gives red,
with the name of the file, to a JSON that cannot be read, to a root that is neither
of the two forms, to a repeated key and to one the chain does not tie. The
`epoch_digest` of the vector is **not** recomposed from the file, because it lacks
`pendingRoot` and `frozenRoot` (`conformance.rs:90`, `recolectar`, composes it
v2 with the whole head). ⚠️ §851: it said «The judge does not enter the
canon: it is run by hand»; since §851 the canon runs it, in «3 duodecies».

⚠️ **What that measurement does not cover**, and the judge prints it in its output:

- **What no vector exercises.** The four bring only `OpenAccount`,
  `Mint`, `Send` and `Claim`. Of the thirteen tags, nine —`Transfer`, `Burn`,
  `Recovery`, `Governance`, `Freeze`, `MintToPending`, `Migration`, `Refund` and
  `Consumo`— are written here and in `tools/segunda/cable.py` without contrast
  against the node: with the `Consumo` one changed to 14 in a copy of `cable.py`
  twenty-four of twenty-four still come out. Of the per-class table,
  only `OpenAccount`, `Mint` (one of the five delegated ones:
  the authorization seal of `Freeze`, `Recovery`, `Governance` and
  `MintToPending` is proved by no entry) and `Send` and `Claim` (only the
  sentinel of their `compromiso`) contrast their row; `Burn`, `Refund`, `Migration`, `Consumo` and
  `Transfer` have no entry.
- **A truncated log, or one rewritten coherently.** Removing the last
  entry of `zkssl-0.4.json` five of five come out: recomposing says that the
  entries are coherent among themselves, not that the last `chain` is the one the node
  signed. Comparing it with the `chainDigest` of a signed head would say that, and
  the judge does not do it.
- **The genesis root.** It travels, as `rootOld` of the entry `seq = 0`, but
  the file brings no other, independent one, to contrast it with (`verify`
  receives it from outside, `log.rs:465`): rule 2 is checked from the second
  entry.
- **That the eras do not go backwards.** Since §281 every record is era 2
  (`append` and `append_con_compromiso`, `log.rs:366-408`: `append` delegates with
  the sentinel, :373, and the only `entries.push` records `chain_digest_v2`,
  :397, and `compromiso: Some`, :403), but neither `verify_chain` (`log.rs:420-447`)
  nor the judge rejects an era-1 entry behind an era-2 one: the judge
  gives it as a note, not as red.
- **A stretch that does not start at `seq = 0`** (`fromSeq`): without the `chain` of the
  previous entry there is nothing to chain the first one to.

### `zkssl_applyMany` — N operations against ONE starting root

```
BatchOp = {kind: "send",  receipt: SendReceipt,  sender: Q,
           senderState: ClientState, amount: Q}
        | {kind: "claim", receipt: ClaimReceipt, receiver: Q,
           receiverState: ClientState, notice: PendingNotice}

BatchApplied = {
  batch:     {size, fromSeq, toSeq: Q, rootOld, rootNew, chain: Digest},
  applied:   Applied[],
  recepcion: {rx, era, n: Q, hashPrueba: Digest}      // §611, RFC-0014 E3
}
```

**The fields are the SAME as those of `zkssl_applySend` and
`zkssl_applyClaim`, plus a discriminant.** Whoever already speaks the protocol
learns nothing new, and that is why the method is **additive**: the two
single ones still exist with their synchronous response intact, and the
protocol version **does not change**.

Normative rules:

1. **All or nothing.** The N operations are validated against the starting
   state and only then applied. If one fails, **none** is
   applied and the response is an error.
2. **An empty batch is `InvalidParams`.** There is no `fromSeq` or `rootOld`
   to return.
3. **An account at most once per batch**, and **distinct pending
   positions**. Two operations that share a position
   reject the whole batch (`DuplicatePendingInBatch`). That is why a
   claim **cannot go in the same batch** as the send that creates its
   pending: they go in consecutive batches.
4. **`applied` goes in the input order of `ops`**, which is the same in
   which they are applied, and their `logSeq` are consecutive from
   `batch.fromSeq`.
5. **`accountsRoot` of each `Applied` is the `rootNew` of ITS log
   entry**, not the final root of the batch.
6. **`batch.rootOld` is the starting root against which ALL the proofs
   were validated.** It is the only thing the batch can give back in exchange for
   the guarantee it removes (see the next section): it lets the client
   check against what its own was validated.
7. **The batch receives ONE reception receipt** (§611, RFC-0014 E3, D-A), in
   the result and in the `error.data` of a rejection by the layer, with the
   form of the direct path's. Its `hashPrueba` is
   `hash_del_lote(composición)`: for each operation, in its order, the
   digest of its proof (§116), its account (`sender` or `receiver`) and the
   position of its pending (`receipt.notice.position` or
   `notice.position`). The composition **does not travel back**: whoever
   assembles the batch has it, and it stays signed because its fingerprint is under the
   `recepRoot`. It is reserved after parsing -an empty or malformed batch
   is noise and consumes no number- and before the layer.
8. **The rejection names its operation**: `error.data.operacion` (Q) is the
   index in `ops` of the one the batch did not admit -by its form, the second
   of the pair that repeats an account or a position; on validation, the first that does not
   validate-. It is the node's word (RFC-0014, D-B).

⚠️ **Whoever assembles the batch is not the node.** The node does not accumulate operations:
it applies those that reach it together in one request. Putting them together is the work of
an **aggregator** —a bank, a payment provider—. It needs no keys:
the proofs come ready-made.

#### What exactly an aggregator sees

⚠️ Until §231 it was asserted here that it **«sees who pays whom»**. It is
**false**, and it was an unchecked assertion. Measured field by field
over the DTOs:

| processes | sees | does NOT see |
|---|---|---|
| sends only | sender, amount, `notice.position` (and, between §523 and §538, **the receiver** inside the proof) | **the receiver** through the proof, since §538 (RFC-0009 E3b-2) |
| claims only | receiver, amount, `notice.position` | **the sender** |
| **both** | **the complete edge** | — |

**The receiver's identifier is not a field of the send receipt.**
`SendReceiptDto` carries `proof`, `public_inputs`, `commitment` and
`notice`; and `SendPublicInputsDto` are roots, amount, limit and
supply. The `receiver_id` appears in `SendMaterialsDto`, which goes
**from the node to the holder**, and as `receiverId` in the parameters of
`zkssl_sendMaterials` (its row), which go from the holder to the node: the node
always knows it. **And between §523 and §538 it travelled inside
`proof`**: the send proof published the receiver's identity,
because the prover did not hide its witness; since §538 (RFC-0009
E3b-2) it hides it and it does not come out literally. The table above is true of the
fields of the receipt; of the proof it was false between §523 and §538, and what
§231 called false —«sees who pays whom»— was true through that way
while it lasted.

What joins the two halves is **`notice.position`**: it appears in
`receipt.notice.position` on sending and in `notice.position` on claiming. An
aggregator that sees both correlates by that key and reconstructs
sender → receiver → amount.

**Consequence for whoever deploys**, which the previous assertion
hid: **separating sends and claims into distinct aggregators is a
real mitigation.** Neither of the two sees the graph on its own.

And this does not exempt the NODE, which sees all of the above by definition.
`SECURITY.md` already said it right —«qué posiciones cambian y cuándo siguen
siendo observables» [which positions change and when they remain observable]—; the erroneous
assertion was here, not there.

### ⚠️ One batch per root: `applyMany` assumes ONE aggregator

All the proofs of a batch attest their transition against **the starting
root**. When a batch applies, the root moves, and **any other
batch proved against the previous root is rejected whole** with
`StaleState`.

This is not an implementation detail: it is a **restriction of the
protocol**, and whoever writes a concurrent client has to know it
before discovering it by throwing proofs away.

#### Taking turns, not competing — and a turn is not a consensus

From the restriction it follows that **several aggregators have to take turns**
instead of competing. §230 measured the price of not doing so: four that go
out at once apply one and the other three throw away **75 %** of their proofs.

⚠️ **A turn is not a consensus, and it is better not to read it that way.** A consensus
decides **who is right** among parties that disagree about the state.
A turn decides **who goes first** among parties that already know that only
one fits and disagree about nothing. There is nothing to agree on here: the root already
determines the outcome, and taking turns only avoids generating proofs that are going to
die. None of this opens a crack in the single-node model of
`SECURITY.md` §6.

⚠️ **Which turn mechanism to use, this specification does not say, and it is
deliberate.** The normative specification rules over **the wire** —what
travels and what it means—, not over **who operates what**. The recommended
deployment model is in `SECURITY.md` §2.ter, as a recommendation
and not as a conformance requirement.

Measured (§230, bench I.1) with four aggregators sending at once, each
with its own accounts —not competing for an account or a pending
position, only for the root—:

| | |
|---|---|
| batches that apply per round | **1 of 4** |
| rejected with `StaleState` | **3 of 4** |
| wasted proofs | **75 %** |

Two consequences for whoever implements:

1. **The node rejects cheaply.** A dead batch costs **3.1 ms** against
   the 32 of an applied one —9 %—, because the root is checked before
   verifying the proofs. A node with several aggregators does not drown.
2. **The price is paid by whoever loses.** Eight discarded proofs are ~2 s
   of the aggregator's CPU; the node threw away 9 ms. A ratio of **655×**.

And therefore: **the batch does not remove contention, it changes its grain.**
Applying one at a time, contention is per operation and one
proof is lost; in a batch, contention is per batch and N are lost. With **a single
aggregator** the waste is zero and the node applies 248 op/s (§229).

§223 declared that the aggregator sees the payment graph; §231 corrected it
—only whoever processes **both halves** sees it—. This is the second
reason, and a technical one, for there being **one**: two cannot win the same
root.

### What holds when applying one at a time, and the batch removes

⚠️ **Today, each operation is applied alone, and that is why the roots of the
log coincide with those the proof declares.** A third party who has
an entry **and** its proof can check that the transition the
proof attests is exactly the one the log noted, **without needing
to have the tree**.

**With `zkssl_applyMany` that stops being true —and since §222 the method
exists, so it is no longer a warning about the future—.** It still
holds for the operations applied one at a time with
`applySend` and `applyClaim`, which have not been touched. In a batch of N
operations, on the other hand, each
proof is generated against the **starting root of the batch** and attests a
**hypothetical** transition —«the root that would come out if my change were the
only one»—. The log, on the other hand, has to note the **real** roots,
because that is what guarantee 2 demands.

Consequently, inside a batch:

- `rootNew` of an entry will **not** be, in general, the root its own
  proof declares;
- tying an entry to the transition its proof attests **will require having
  the tree** —that is, replicating the state—, not only the entry and the
  proof.

This is coherent with the model declared in `SECURITY.md` §6 and in
entry §121: verification by **replica**. But it is a **real
loss** against what can be done today, and that is why it is declared here
instead of being discovered on reading the code.

### And a consequence for conformance

⚠️ **The composition of the batches is observable.** Two implementations
that apply the same sequence of operations **grouping them
differently** will produce entries with distinct `rootNew` and, therefore,
distinct chains. The log stops being a function only of the
sequence of operations: **it also depends on how they were grouped**.

That is why **the conformance vectors declare their grouping
policy**. Those of `zkssl/0.1` and `zkssl/0.2` were generated **without batches**
—one operation per application, N=1— and an implementation must
reproduce them applying one at a time. When batch vectors exist,
they will say so explicitly.

⚠️ **Since §222 `zkssl_applyMany` exists, and the `zkssl/0.2` vectors
are still N=1.** That is deliberate and not an omission: the
protocol surface is additive and the wire values did not
move, so the existing vectors **remain valid as they
are** until §354 —today `conformance --check` of `zkssl/0.2` rejects them as «de OTRA versión»
and they are kept under their version, the regime of `0.1`—. ⚠️ **Corrected in §699**: «as they
are» holds against that method; the file did not stay as it was for other reasons, because
`vectors/zkssl-0.2.json` was re-issued under its same version in §275, §278 and §281
(«Versioning», §«Operational notes»). An
implementation that wants to attest the batch will need vectors of its own;
there are none yet.

---

### `receptionSeq` — the reception counter (§253)

`zkssl_applySend` and `zkssl_applyClaim` return **`receptionSeq`**: in what
order that operation arrived **among those the node set out to evaluate**.

⚠️ **It is not `logSeq`.** `logSeq` is **the order of application** and only exists
if the operation was applied; `receptionSeq` is **the order of arrival**. **Censorship
lives in the gap between the two.**

| fails in | consumes `receptionSeq`? |
|---|---|
| parsing or the wire | **no** — that is noise, and if it counted **anyone could open gaps in someone else's log by sending garbage** |
| the layer (invalid proof, moved root) | **yes** — the node verified a real operation, and **that is where a censor would hide** |

⚠️ **It also travels in the error**, as `[receptionSeq=0x…]` in the message:
the case that matters is precisely the rejection.

### ⚠️ What `receptionSeq` is NOT: opposable evidence

**It is a number the node says and nothing ties.** `chain` authenticates `seq`,
`kind`, the two roots, the proof digest, the previous one and, since §281, the
entry's `compromiso` — **and nothing else**. (⚠️ §797: until here it said «the
proof digest and the previous one — **and nothing else**», without the commitment, which enters
the chain since §281; the formula, in «The composition of `chain`, byte by
byte».)

- **Two holders who cooperate DETECT the reordering**: A has
  reception 100, B has 101, B's is in the log and A's is not.
- **They cannot PROVE it**: neither has anything signed by the operator that
  says *«I received yours as the 100»*, and **the operator can deny having
  said it**.

⚠️ And the holder **sees a gap and does not know whether it was censorship or its own bad
proof**: this **detects the reordering, it does not explain it**.

What is missing for both things is **the acknowledgement**, as a leaf under a root in
the head — it inherits the signature **without spending XMSS indices**.

### ⚠️ `receipt` and `acuse` are different things

| word | what it is |
|---|---|
| **`receipt`** | what **the holder hands** to the node: `SendReceipt` and `ClaimReceipt` carry `proof`, `public_inputs` and `commitment` |
| **`acuse`** | what **the node returns** to the holder |

**`receipt` is not reused for the acknowledgement**: it would be a third meaning on
the same wire, and in a project whose argument is verifiable
conformance an ambiguous name costs more than elsewhere.

#### The acknowledgement in the response (§274)

`zkssl_applySend` and `zkssl_applyClaim` —**only the holder's paths**—
return, next to the `logSeq` and the `receptionSeq` as always:

```json
"acuse": { "epoca": "0x2b", "n": "0x5a0", "hashPrueba": "0x..." }
```

- **`epoca` = `logSeq + 1`**: the **first head that can contain** the
  operation — not the one that will contain it, which the operator chooses. A leaf
  that declares this epoch living under head `S` makes
  `S − epoca` legible from the leaf and the head alone: the magnitude the promise
  bounds.
- **`n` = 1440**, the declared ceiling (§121) and **normative since §275**:
  it travels signed in the head, and an `n` lied about here produces a leaf that
  does not verify under the signed root.
- The leaf is `acuse_digest(hashPrueba, epoca, n)` (§270) — the holder
  computes it **at the apply** and it stays fixed forever.

⚠️ **Declared limit**: this response **is not signed**. Signing each
acknowledgement costs 160.5 ms measured (~×50 of collapse over the ceiling of §217) and
would spend XMSS indices (§121.2). The acknowledgement inherits the signature **on closing the
epoch**, under the root — **real since §275**, through `zkssl_ackPath`; until
then it is the node's word, and the window is ≤1 heartbeat with an honest
operator. See entries §274 and §275.

#### The reception receipt in the response (§571, RFC-0010 E3)

The same two holder's paths return, next to the `acuse`, its sibling:

```json
"recepcion": { "rx": "0x2a", "era": "0x7", "n": "0x5a0", "hashPrueba": "0x..." }
```

and they return it **also in the `error.data` of a rejection by the layer**, beside the cause
(§454): it is the case that matters, because a censor would hide by rejecting. The
acknowledgement says that the operation was APPLIED; the receipt, that the node RECEIVED it and
set out to evaluate it, whether it was applied or not (RFC-0010, D-A and D-E). ⚠️ §815: that it
RECEIVED it and JUDGED it. Since §815 the leaf is noted after the layer's verdict and before
responding (RFC-0010, the amended D-E): an evaluation cut off halfway leaves the number hollow,
with no leaf.

- **`rx`** is the reception number (the `receptionSeq` as always, now as data): inside the
  era `(Q, R]` that closes it, the leaf is the `rx − Q − 1`.
- **`era`** was declared by the node on noting: the XMSS index of the last signed head, plus one
  (D-D, §567). It goes INSIDE the leaf. ⚠️ §815: it fixes it on RECEIVING, before the layer judges,
  and writes it on noting: a signature that finishes while the layer verifies does not move it. It
  is the same reading as before §815, at the same point.
- **`n`** is the ceiling, the same as the acknowledgement's, also inside the leaf.
- **`hashPrueba`** is the digest of the proof that arrived; the holder recomputes it from its own.

The leaf is `recibo_digest(hashPrueba, era, n)` (D-B). ⚠️ Like the acknowledgement, **it is not
signed**: it inherits it on closing the era, with `zkssl_recepPath`. A `-32603` because the node
could not note **carries no receipt**: there is no leaf to promise, and the operation was not
evaluated. ⚠️ The key is `recepcion` and not `recibo`: `recibo` already names on the wire the
`publicInputs` of the rejection envelope.

⚠️ **§815: that `-32603` no longer exists, and «the operation was not evaluated» is no longer
true.** Since §815 the node notes AFTER judging, so a broken log no longer gives a `-32603` of its
own: if it fails after the verdict, the request receives its verdict —the result with its
acknowledgement, or the `-32000` with its cause— WITHOUT `recepcion`, and the node goes into
PARADA; the following requests receive the PARADA's `-32603`. What remains standing is what
matters: without `recepcion` there is no leaf to promise. A store failure on applying leaves no
leaf either: the response is the PARADA (`-32603`, «nodo en PARADA: …»), and the operation may or
may not be in the ledger after restarting. In `applySend` and `applyClaim` the `message` of that
PARADA carries its `[receptionSeq=…]`; in `applyMany` and `zkssl_pledge` it does not, because
those two arms do not put it in the `message` —their number travels in `data.recepcion`, and the
PARADA carries no `data`—, so whoever is left with that response does not know its `rx` to ask
`zkssl_recepPath` whether a gap was left (read in the code, not measured). **Without a response,
or without `recepcion`, before paying again** the holder looks for its `hashPrueba` among the
`proofDigest` of `zkssl_logEntries`: a death between the judgement and the response can leave the
operation applied without anyone having told it, and another payment with another proof would be
another payment (`SECURITY.md` §3.15). It holds for a send and a claim, whose `proofDigest` is the
digest of their proof (table «What each class records»); in a batch one looks for that of each of
its proofs, because its `hashPrueba` is the fingerprint of the composition; and a pledge records a
`Consumo`, whose `proofDigest` is the empty proof: one looks for the entry whose `compromiso` is
its consumption.

⚠️ **Since §611 (RFC-0014 E3) `zkssl_applyMany` and `zkssl_pledge` also carry a receipt**, with
the same rule: one for each thing the node EVALUATES, reserved at the same point as the direct
path. The batch, ONE per batch, over the fingerprint of its composition (rule 7 of its section);
the pledge, when its proof reaches the judge, with the digest of that proof (see «The pledge, with
its envelope»). Until §611 they did not carry it, by the decision of §576 (RFC-0010, D-E), which
RFC-0014 reopened with its resolution. Their resolutions in the completeness envelope are the
`lote` (§612) and `prenda` (§613) forms of `spec/PAQUETE.md` 2.11.

### `zkssl_signedEpochHead` — the last signed head, for a WITNESS

Returns the **most recent** epoch head **the node signed**, with everything
needed to verify it without it: the **eighteen fields** of the
head (§275, §292, §415, §452, §570), `publicKey`, `epochDigest`, `formatVersion`, `index` and
`signature` — fields+digest+signature **together**, from the same heartbeat: a single
custody artefact, with no race between calls.

⚠️ **Additive**: it does not touch `zkssl_epochHead`, which keeps serving the head
**without signature** and is in the conformance vectors. **The version does not go up**,
for the same reason it did not go up with `zkssl_applyMany`.

### ⚠️ Format **v3** (§292): the head proves that it EXTENDS

The head gains `mmrRoot` and `mmrSize`: the **peak of the MMR of heads**
over the digests of all the previous ones, and how many it accumulates. They travel
**signed** — `formatVersion: 3` — and with them a client that custodies
an old head can verify that the new one EXTENDS it with an O(log N)
consistency proof, without downloading the log (link 2 of the
chain of residual trust).

The composition, by wrapping as v1→v2:
```text
v3 = merge( epoch_digest_v2(los siete), merge(mmrRoot, mmrSize) )
```
**Declared genesis**: the first head composes with peak `as_digest(0)`
and `mmrSize: 0`. ⚠️ **Additive on the wire, a break in the parser**:
`zkssl/0.2` does not go up (the precedent of §275), and a consumer with
`deny_unknown_fields` **breaks out loud** — the honest failure already
designed. A custodied **v2** head **still verifies**: the version the
signature declares chooses the recomposer, in the library and in the command.
A node **without a journal** restarts with the accumulator empty and its
`mmrSize` is reset VISIBLY; whoever signs carries a mandatory
journal (§285), so every signed head carries continuity.

### ⚠️ Format **v4** (§415, RFC-0006 E2): the head attests the set of consumptions

The head gains `consRoot` and `consCount`: the **root of the tree of published
consumptions** (RFC-0006 E1, §413) and how many consumptions there are under it. They travel
**signed** — `formatVersion: 4` — and with them a reader with two signed
heads can prove that a consumption is under one and not under the other (E3).

The composition, by wrapping as v2→v3:
```text
v4 = merge( epoch_digest_v3(los nueve), merge(consRoot, consCount) )
```
**Declared genesis**: the first head composes with the root of the empty
consumption tree and `consCount: 0`. ⚠️ **Additive on the wire, a break in the
parser**: `zkssl/0.3` does not go up (the precedents of §275 and §292: what
raises it is that the values the vectors seal change, and
`conformance --check` keeps giving «todo IDENTICO»), and a consumer with
`deny_unknown_fields` **breaks out loud**. A custodied **v2 or v3**
head **still verifies**: the version the signature declares chooses the
recomposer, in the library, in the command and in the witness (§414).

### ⚠️ Format **v5** (§452, RFC-0007 E1): the head commits what the proofs cite

The head gains five keys, **one family** (RFC-0006 D-3: one per version):
`paramsDigest` —the seven parameters of the ledger in a digest with its own domain—,
`pmetaRoot` —the root of the pending-meta tree, where the birth
of each pending lives (§388)—, `nextPending` —the watermark of the pendings:
every live position is smaller (§211)—, `nextIndex` —the count of openings, which is the
**quota** against `maxAccounts` and not a position: since F3 an account is placed
by its identity— and `totalSupply`. They travel **signed** — `formatVersion: 5` —;
with them a rejection can cite a rule that is under the signature, and the age of
what is in flight has a universe and a root (RFC-0007, D-B).

The composition, by wrapping as v3→v4 (the exact form, with the counters
inside `as_digest`, in `spec/NUCLEO.md`, section 6):
```text
v5 = merge( epoch_digest_v4(los once),
            merge( merge(paramsDigest, pmetaRoot),
                   merge(nextPending, merge(nextIndex, totalSupply)) ) )
```
`paramsDigest` composes `regulatoryLimit`, `maxSupply`, `maxAccounts`,
`custodianRoot`, `governanceRoot`, `refundTtl` and `maxCustodianUses`, in that
order, and `zkssl_params` serves the seven since §452. **Declared genesis**: the
first v5 head composes with what the ledger has at rest when it issues it —the
opening parameters, the root of the empty meta tree, `nextPending: 0`—; there is
no new key in the store, so an earlier ledger opens and composes v5.
As in v4: additive on the wire (`zkssl/0.3` does not go up), a break out loud for
a consumer with `deny_unknown_fields`, and a custodied v2, v3 or v4 head
**still verifies** with its recomposer.

### ⚠️ Format **v6** (§570, RFC-0010 E2d): the head signs what the node RECEIVED

The head gains two keys, **a pair** (RFC-0010 D-C): `recepRoot` —the root of the tree of
receipts of the era `(Q, R]`, with one leaf for each operation the node set out to EVALUATE,
whether it was applied or not— and `recepCount` —`R`, the last reception number reserved on
composing—. The lower bound `Q` **does not travel**: it is the `recepCount` of the previous head.
They travel **signed** — `formatVersion: 6` — and with them a rejection the operator would like to
silence leaves a leaf under the signature: the acknowledgement tree says what was APPLIED; this
one, what was RECEIVED (RFC-0010, D-A and D-E). ⚠️ §815: one leaf for each operation the node
JUDGED. Since §815 the leaf is noted after the layer's verdict, and an evaluation cut off halfway
leaves its number hollow, with no leaf (above, «The reception receipt in the response»; RFC-0010,
the amended D-E).

The composition, by wrapping as v4→v5 (the exact form in `spec/NUCLEO.md`, section 6):
```text
v6 = merge( epoch_digest_v5(los dieciséis), merge(recepRoot, as_digest(recepCount)) )
```
The leaf, the dense position `rx - Q - 1` and the era the receipt declares —the XMSS index of the
last signed head, plus one— are the rules of `zk_ssl_verify::recibos` (§562, §567), the same
ones the node's builder uses. **Declared genesis**: the first head composes with the root of the
empty tree and `recepCount: 0`; after a restart `Q` comes from the node's journal, which since
§570 notes `recepCount` on every line, and without it the first era comes out fat —from 0—, like
the first epoch of acknowledgements. As in v5: additive on the wire (`zkssl/0.4` does not go up),
a break out loud for a consumer with `deny_unknown_fields`, and a custodied v2 to v5 head **still
verifies** with its recomposer. The reception receipt on the wire is E3; until then the holder
does not have its own, and what this version fixes is the tree under the signature.
⚠️ **Corrected in §807**: it said «`zkssl/0.3` does not go up»; v6 is from §570, and the wire had
been at `zkssl/0.4` since §538 (§«Operational notes», «Versioning»): what did not go up was `0.4`.

⚠️ **Three responses, and none is a generic error**:

| `available` | when | what it brings |
|---|---|---|
| `false` | there has been no heartbeat yet | `reason`, `beatSeconds` |
| `false` | the node started **without `--clave`** | `reason`, the head **without signature** |
| `true` | there is a signed head | everything needed to verify |

The second case is the form of §241 brought to the wire: **the missing piece
—the signature— is noticed here too**, and not as a failure.

### ⚠️ `custody` and `custodyChecked` — asserted versus checked

The response **always** carries both fields, even when there is no signature.

| field | what it is |
|---|---|
| `custody` | **what the operator ASSERTS**: `sin-declarar` (by default), `fichero`, `hsm`, `kms`, `otro` |
| `custodyChecked` | whether **the node could check it**. Only `fichero` is checkable |

⚠️ **It is an assertion of the operator, not a check by the node.** The node
cannot know whether there is an HSM behind, and does not pretend it can: if the operator
declares `hsm`, the response carries `custodyChecked: false` and the start-up
warns out loud.

⚠️ **The value of the declaration is not in its being true, but in that
lying in it is opposable.** An operator who declares `hsm` and operates with a
file has made a false assertion that it cannot deny — the same model
that holds up the rest of the apparatus.

⚠️ **`sin-declarar` travels like the others.** If the field were omitted when
nothing is declared, a consumer could not tell **«does not declare»** from
**«old version of the node»**. Present and honest by default.

⚠️ And none of this makes the signature opposable: **the public key is still without
an anchor** (`SECURITY.md`). A third party verifies against the key the same
node gave it, and that is circular.

`emittedAtUnix` and `beatSeconds` exist so that **a witness that asks twice
and receives the same signature can tell «there has been no heartbeat» from «I am being
deceived»**. The XMSS `index` already allows it —it is monotonic— but it is better
that it be explicit.

⚠️ **Only the last one, and in memory.** There is no history: **it is lost on
restarting**, and with a start-up of ~136 s at 10⁶ accounts a witness will spend that
time receiving `available: false`.

⚠️ **MEASURED in L.1 (§247).** What the witness sees **is not a gap of
indices**: it is a **window of `SinFirma`**, and afterwards the index continues
**contiguous** —the guardian only increments on signing, so dying between
heartbeats spends none—. There is a gap of indices **only if a
head was signed that nobody managed to collect**, and that depends on the relation between the
cadence of the heartbeat and that of the query, **not on the restart**.

### ⚠️ The history: DECIDED in §248 — the operator does NOT serve it

§242 postponed it for lack of a consumer. The consumer arrived (§245) and so did the
datum (§247) — and the answer **is not the one expected**:

> **A history served by the operator is the operator saying what it said
> before.** It can rewrite it. A third party who consults it gains no
> property it did not have.

It is not that it costs: **26 MB a day is not expensive**. It is that **it does not provide what
was asked of it**. Storing 9.5 GB a year so that the operator can repeat
itself is paying for nothing.

⚠️ **And the decisive argument**: a split view **between distinct parties**
is undetectable from a central history **by construction** — the
operator that produces it is the same one that serves the history. **Only two
independent records reveal it.**

That is why the record is kept by **the witnesses**, in their journal (`--diario`),
which since §248 keeps **the whole signed head**: a third party re-verifies
without the node, and two journals are compared field by field.

⚠️ **The signature is what makes this work.** It is what prevents a
malicious witness from fabricating evidence against the operator: without it, comparing
journals would prove nothing. And that is why the journal costs **more** than the
history —the hexadecimal doubles the bytes: ~54 MB a day, ~20 GB a year per
witness—. **It is the only version that proves something.**

⚠️ **Comparing two journals detects the divergence, it does not say which one lies** —
*detecting is not distinguishing*, again. But it is not needed: what stays
proven is that **the operator issued two different things for the same
index**, and that is already opposable.

⚠️ **What this does NOT cover: a witness that did not exist has no journal**, and
there is nothing to be done there. It is the structural limitation of the whole model
—Certificate Transparency has it too— and it goes written **next to the
decision**, not in a separate note.

⚠️ And **without declared custody of the key, what this method serves has no
probative value** (`SECURITY.md`). §242 makes the artefact exist; not
that it be worth something.

### `zkssl_ackPath` — the acknowledgement path of a CLOSED epoch

Returns what the acknowledgement of the response (§274) lacks to tie itself to
a signature: `s` —the `seq` of the head that closes the epoch— and `camino`
(`{siblings, isRight}`) up to the acknowledgement root of that epoch.

⚠️ **The head does NOT travel** (§248): the path is verified against the one the
holder **already custodies**. Serving it here would be letting the operator
fabricate the yardstick it is measured with.

The chain, whole: `hoja = acuse_digest(hashPrueba, epoca, n)` →
`path_root` → `acusesRoot` → `epoch_digest` **v2/v3** (according to `formatVersion`) → the
custodied signature. The v1/v2 disambiguator is the **version byte of the preamble**
(`formatVersion` 1 → 2), not the domain (§236).

⚠️ **Three forms of `available: false`, and none is a generic error**
(the form of §241): the epoch is still **open** (`reason` +
`beatSeconds`); the node runs **without `--diario`** — the epoch bounds
are not kept, and after a restart without a journal the first epoch comes out
fat: declared limit —; or that entry **does not exist** in the log
within that epoch.

⚠️ **Additive**: `zkssl/0.2` does not go up. The head gains `acusesRoot` and `n`;
`deny_unknown_fields` makes an old parser **break out loud** —
the honest failure already designed, not a half reading.

### `zkssl_recepPath` — the path of a reception receipt, when its era CLOSES (§571)

`{rx}` → the path of the receipt's leaf in the tree of the era `(Q, R]` that contains it,
**without the head and without the root** (§248): the holder climbs ITS leaf —that of its
receipt— up to the `recepRoot` of the head it custodies. The era comes from the series of
`recepCount` in the node's journal (§570): `R` is the first `>= rx` and `Q` that of the previous
head. The response identifies the closing head by its `recepCount` and, if it is signed, its
`index`.

It says what is missing, in the form of §241: without `--diario`, `available: false`; with the
era still OPEN, `available: false` and `beatSeconds`; with an `rx` reserved and not noted —a
declared GAP—, `available: false`. The `rx` 0 does not exist (the counter starts at 1) and is
`-32602`. ⚠️ Additive: the surface goes from 30 to 31 methods and `zkssl/0.4` does not go up.

### `zkssl_keyActs` — the chain of key acts of the operator (§646)

No parameters → `{actas}`: the whole chain from the genesis, in the order in which the node
noted them (RFC-0015, D-D). Each `ActaFirmada` is
`{acta: {clave: DATA, esquema: Q, desde: Q, siguiente: Digest, procedencia: {anterior, epochDigest,
mmrRoot: Digest, mmrSize: Q} | null}, firma: DATA, firmaAnterior: DATA | null}`: `procedencia` is
`null` at the genesis, and `firmaAnterior` when the key that left did not sign —the act declares it
burnt—. Both keys go ALWAYS, with their `null`. It is the same object the journal line
carries, without the journal's `v`, `tipo` and `index`: the kit writes and reads it
(`zk_ssl_verify::actas::acta_a_json` and `acta_de_json`), and an element of a digest worth `p` or
more is not read.

The node judged it on STARTING —a broken chain does not start— and assembles it once: the acts are
only born on starting. Whoever asks for it judges it with `verificar_acta`, link by link: the
genesis alone, and each rotation against the previous one. Empty if the node does not sign acts
(never without `--siguiente`, D-I): then a change of key has no object that explains it, and the
witness stops as today. ⚠️ Additive: the surface goes from 31 to 32 methods and `zkssl/0.4` does
not go up.

### `zkssl_inclusionReceipt` — inclusion, checkable without the node

Returns what a third party needs to check that **a leaf was in
the signed head**: `index`, `leaf`, `path` and the fields of `head` —
**eighteen since §570** (seven in §275, nine in §292, eleven in §415, sixteen in §452); the
`formatVersion` of the signature says which ones compose.
The verification is that of §256: climb the path up to `accountsRoot` and
check that those fields compose —according to the declared version— the
`epochDigest` **of a signed head**. A loose root proves nothing.

⚠️ **From the `accounts` tree**, which is the one the head signs. `CommitmentLayer`
is only instantiated in its own tests: a path of its own would lead to a root
**that nobody signs**.

⚠️ **`leafFormat` is an OBSERVATION, not an assertion.** The node composes the
leaf **both ways** —`native_leaf` and `native_leaf_salted`— and declares
the one that matched the tree. If it lied, the holder would compose wrongly and the receipt
**would fail**: a wrong form **cannot make a false receipt
verify**, only make a legitimate one fail. It is there so that the failure is
**legible** instead of a bare `RaizDistinta`.

⚠️ **Additive**: `zkssl/0.2` **does not go up**. It changes no value that travels.

### ⚠️ What a path exposes: enumeration of the balance

**This is NOT specific to the receipt. It affects `zkssl_sendMaterials` just the same, which
hands out `senderPath` without any credential since before this method
existed.** It is said here, under both doors, because saying it only under one
is how one ends up having two.

**The acknowledgement path (§275) is different on purpose**: its leaves are
**enumerable BY DESIGN** — digests of acknowledgements, not balances or operations
(the deliberate divergence from §157 that `acuses.rs` documents). Trying
candidate leaves against `acusesRoot` exposes nothing the public log
does not already say.

With an account path and the root, anyone can try candidate leaves
**without speaking to the node again**: compose `native_leaf(publicId, saldo,
nonce)`, climb the path and compare. The three pieces that make the space
enumerable **are public**:

| piece | where it comes from |
|---|---|
| `publicId` | `zkssl_publicId`, **without credential** |
| bound of the balance | `regulatoryLimit`, which `zkssl_params` publishes |
| nonce | a counter that **starts at zero** |

A reader can do the arithmetic: the product of those two bounds is the number
of leaves to try, and **each try is a few hashes**. With the
`regulatoryLimit` of the reference node —500,000— and a low nonce, that is
enumerable on any machine.

⚠️ **The salt of §117 is the only thing that prevents it**, and its own documentation
says exactly this: it protects «de terceros que ven caminos y pruebas» [from third parties who see
paths and proofs]. It is derived from the spending key, so a third party **does not have it**, and
it multiplies the space by the size of the key.

> ⚠️ **That is why the migration of §117 is not an improvement: it is the condition for
> operating in the open.** An unmigrated ledger that exposes `sendMaterials` or
> this method **publishes the balances** to whoever wants to compute them.

### ⚠️ CORRECTION (§860): the store prior to D4 does not open with this code, and the door that remains is the snapshot ≤ v6

The table below said that a legacy store never migrated, prior to D4,
**remains exposed**, and that its tool is `migrate_to_salted_positions`. With
this code **it does not open**: `load` demands with `need` the roots `root:pending`
—since §387—, `root:pmeta`, `root:froz`, `root:cons` and `root:log`, and
none existed before the D4 flip, so the store comes out, at the latest, with
`Malformed("falta la clave root:pending")`, before serving a path or
opening an account. The whole class, «prior to §387», is exercised by
`a_ledger_without_a_pending_root_does_not_open`. What is below stopped being
true in §387 (`53d339e`), and that seal did not touch this table. With the code
that wrote it, that store **remains exposed**, and what is above still stands
for it: reasoned over what was read in `git`, not opened against a real store.

The legacy door that remains alive is another:

- **the v3..v6 snapshot is read**, by the promise of the format: as an old
  world, without salt, with the frozen ones at 24, in memory and without `sled`. No
  path of the node imports it;
- **`migrate_to_salted_positions` is its tool, of geometry and not of
  privacy**: it removes the mixed tree, but its accounts with zero salt —those of
  v3 to v5 and, in v6, those opened before §117— are still enumerable,
  because zero salt is public (§126.4, §132.3).

It is prose, not wire: it changes no method, field or vector, like the
correction below. The measurement and the reasons, in entry 76 of `BACKLOG.md`,
closed in §860.

### ⚠️ CORRECTION (§265): the statement still stands, the subject expired

What is above **was not false and is not withdrawn**: an unsalted ledger that hands out
paths publishes the balances. What expired is **whom it speaks of**.

| ledger | state today |
|---|---|
| created or persisted by this code | **salted**. `commit()` writes `meta:geometry_v7` on every persistence (`persistence.rs:578`; ⚠️ §860: today `:794`), and live every new account comes out salted |
| legacy store never migrated, prior to D4 | **remains exposed**. Its tool is `migrate_to_salted_positions` (§157). ⚠️ §860: with this code it does not open, and that tool is the snapshot's ≤ v6 (the correction above) |

**When it was fulfilled, with the commit in front**: the D4 flip in **§156**
(`2ac37c5`) —the layer speaks salted end to end and the legacy ones die— and the
placement in **§157** (`e3116c6` + `f926de6`), where `open_account` adopts
`public_id mod capacidad` and the two privacy contracts
—`account_indices_are_not_predictable` and `_not_enumerable`— take effect. Label
`entrada-50`.

⚠️ **What this correction does NOT say**: that the condition is fulfilled *for
every ledger*. A store prior to D4 opened in the open still publishes
balances, and besides **does not open again** if an account is created in it —§258-A—.
Both things and their tool live together in **note 76 of the BACKLOG**,
which is where they can be ticked. ⚠️ §860: entry 76 was closed: the store does not open with this
code, and the tool is the snapshot's ≤ v6 (the correction above).

> ⚠️ **Why this sentence lasted five seals.** It was born in §259 accompanied by
> «it is not a pending backlog entry; it is a deployment condition».
> That took it out of the only place in the project where things have a state. A
> BACKLOG note is ticked; a deployment condition written in an
> entry **has no box**. It was promoted in category and with that it became
> uncloseable.

### ⚠️ CORRECTION (§261): the above was false

This specification asserted, word for word:

> ⚠️ **`claimMaterials` is not in the same case**: it demands the payer's
> `PendingNotice`. It is not a credential the node checks, but **one has to
> have it**. The door wide open is one, not two.

**Measured in §261: `claim_materials` (`client.rs:138`) uses of the notice
only `notice.position`, and the dispatch does not validate it.** Anyone could
call with an **invented** notice and obtain the receiver's path. **Neither a
secret nor a prior payment was needed: they were two doors wide
open, not one.**

### The credential (§261)

`sendMaterials`, `claimMaterials` and `inclusionReceipt` now demand the
`viewKey` of the account **whose path is returned** —the sender, the
receiver and the requested index—, checked against **that** index. Error
`-32004` if it does not match.

⚠️ **It is not a new control: it is one that existed on the wrong side.** The SDK
already checked «los materiales de cobro no corresponden a esta wallet»,
**after** receiving the paths. A client that removes that line
obtains them all the same. **A check on the client's side does not protect the
system: it protects the client that runs it.**

### ⚠️ What the receipt promises, before and after

| | before | after |
|---|---|---|
| **verifying** a receipt | anyone | **anyone** — the same |
| **obtaining** someone else's receipt | anyone | only the holder |

**«Checkable by third parties» is not lost: «obtainable by
third parties» is lost, which was the hole.** §256 asserted that a third party can check
an inclusion without the node, and **that does not change**: the receipt is self-contained and
`verificar_inclusion` still does not need the operator. What changes is
**who can ask for it** — and the holder can forward it to whomever it wants, so
checking by third parties becomes **by delegation** instead of
open.

⚠️ The latter **was never a designed property**: it was **the absence of a
control**, and §259 measured what it cost.

⚠️ **Real loss, said as a conjecture:** a witness can no longer check
inclusions of other people's accounts. Today it does not do so either —it watches heads, not
accounts—, and in Certificate Transparency the monitors check
consistency of the log, not individual inclusions. **That it therefore does not matter
in practice is a conjecture, not a measured argument.**

Closing the exposure —demanding a credential for the paths— **is a separate
seal**: it changes a surface public today and can break clients. Putting it
here would chain «serving the receipt» with «closing `sendMaterials`», and if something
went wrong one would not know which of the two it was.

## The transport of the co-signature (§315)

`zkssl_submitCosig` and `zkssl_cosigs` are the path that link 3
needed and did not have: until today a witness co-signed a head and
wrote it **in a file of its own**, where nobody else saw it. The node becomes
the TRANSPORT — which is what note 83 decided — without becoming the
authority.

```text
Cosig = {v: Q, epochDigest: Digest, clavePublicaOperador: DATA,
         clavePublicaTestigo: DATA, versionFormato: Q, indice: Q,
         firma: DATA, vistoUnix: Q}
```

They are the eight fields the witness already writes in its JSONL file, **in
wire convention**: there `v` and `vistoUnix` go as raw numbers and
here as QUANTITY. They are two artefacts with two conventions, and it is said
instead of being disguised.

⚠️ **What the node checks and what it does NOT.** It checks that the co-signature is of
the CURRENT epoch and that **the signature closes**: rejecting what is malformed is
cheap and the reason goes named. **It does not accredit the witness**, and it cannot: the
witness's key travels in the object itself and nothing backs it, so
anyone can sign with their own and this will pass. **Who decides which
witnesses count is the client, with its policy** — and it has to be so,
because the node is the operator and an operator that names its watchers is
not watched.

⚠️ **Bounds of resource, not of trust.** The store keeps **a single
live co-signature per witness key** —the series is per witness (§310), so
resending replaces instead of accumulating— and **only the current epoch**: each
co-signature carries its `epochDigest` and the old ones are discarded on the next
submission. On top goes a hard cap, `--max-cofirmas`, because each co-signature
is ~37 KB.

⚠️ **What the store DOES guarantee (§317).** The co-signatures of an epoch
**remain served by their `epochDigest`** until a submission of
another arrives. It is not a history —ONE epoch is kept, the last with submissions—
but it is what makes the method usable: a client asks for the head at T and
the co-signatures afterwards, and **comparing the digests only works if the node has
that of the epoch it custodies**. Asking for an epoch without co-signatures returns
`n: 0`, and that **is not an error**. This section published the opposite
since §315; the bench of §316 uncovered it.

## The extension proof (§293)

`zkssl_consistencyProof` turns link 2 into a SERVICE: whoever
custodies a v3 head asks for the path with its `mmrSize` as
`oldSize`, captures the head whose `mmrSize` EQUALS that of the
response — ⚠️ the signed pair is the accumulator BEFORE each
head, so that one is **the next to be issued**, at most one
heartbeat later (§248: the head does not travel in the path) — and verifies **without the log**
that the new peak EXTENDS its own — O(log N) consistency of the history tree
(§291).

Three responses without a path, each with its reason: `oldSize: 0`
(there is no history to extend), `oldSize` equal to the current
`mmrSize` (identity: empty path), and `oldSize` GREATER than the current one —
**the accumulator is behind**: a node that restarted without a journal says so
on the wire, which is exactly the VISIBLE reset the v3 format
promises.

**The extension package** of the third-party verifier:
```text
{ "v": 1, "tipo": "extension", "vieja": {…}, "nueva": {…}, "camino": […] }
```
The two heads go AS IS as `signedEpochHead` served them (both
**v3**: a v2 carries no pair to extend) and the command also demands the
SAME `publicKey`: the continuity is of one signer. The
demonstration: `tools/banco_extension.sh` — live node, package
captured, verification without the node, the tampered path in red, and the
restart without a journal answering behind.

## The published consumption (§417, RFC-0006 E3a)

`zkssl_publishConsumo` and `zkssl_consumoPath` put on the wire the object RFC-0006
introduced: a **public and precomputable** label that the ledger accumulates in a tree of its
own, whose root travels signed in the v4 head since §415.

⚠️ **Publishing demands neither proof nor authorization**, and it is declared: whoever has access
to the node publishes, and whoever publishes first blocks. It is denial of service, not double
use (D-4 of RFC-0006). What the node guarantees is the other thing: **an already published
consumption is rejected by name**, and a position collision too, with its own. Since §454 that
refusal (`accepted: false`) also carries, next to `reason`, the same `data` as a `-32000`
(section «Errors»).

⚠️ **The head does NOT travel, and NEITHER does the root** (§248, one step further than in
`zkssl_ackPath`): whoever verifies takes the root from the SIGNED head it already custodies and
climbs the path itself. Serving them would be letting the operator fabricate the yardstick it is
measured with.

⚠️ **Whoever verifies chooses the LEAF.** The same path proves both things: with the digest of
the consumption, that it is under the root of that head; with the ZERO digest, that it was not.
That is why there is one method and not two, and why the response does not say which of the two
it proves.

Two reasons for `available: false`, each with its text: the `seq` is ahead of the log of
this node —another chain, or a head that has not arrived yet—, or a consumption entry of the
stretch does not carry its commitment, and then the stretch is not reconstructed and it is SAID.

⚠️ **Additive**: the surface goes from 24 to 26 methods and `zkssl/0.3` does NOT go up — the
version is moved by a change in the values that travel, not by the size of the surface
(Operational notes; §222, §242, §275). The three conformance vectors stay intact.

## The frozen path (§458, RFC-0007 E3b)

`zkssl_frozenPath {index, viewKey}` returns the leaf of the account `index` in the frozen
tree and its path: `{s, index, leaf, camino: {siblings, isRight}}`. It is the material with
which the rejection `AccountFrozen(i)` —whose cause travels as data since §454— is proved without
the node: the leaf of `i` under the `frozenRoot` of a signed head **is not the empty one**.

- **Demands the holder's credential** (the `viewKey` of §261); without it, `-32004`. The frozen
  state is the holder's —the layer checks it after the authority so as not to leak it
  (`two_phase.rs`)— and whoever receives `AccountFrozen` is always the holder, who can forward the
  path to whomever it wants. Serving it without a credential would publish which accounts are
  frozen, which is a custody action.
- **Serves the leaf whether it is empty or not.** With the mark, the holder proves the rejection;
  with the empty one, it proves that the rejection was false.
- **It is of the state of NOW**, and `s` says of which `seq`. The tree is not reconstructed at a
  past `seq`: the log entry of a freeze carries its commitment, not the index. As that tree only
  changes with another freeze, the path climbs to the `frozenRoot` of every signed head since the
  last one.
- **What it reveals besides the leaf**: the level-0 sibling is the neighbouring leaf, and its
  frozen state can be read (the mark is constant); the siblings higher up say which subtrees have
  no frozen account. It is the same as what the `frozenPath` that `zkssl_sendMaterials` hands to
  the same holder already reveals.

**The depth is fixed by whoever verifies, not by the path.** The tree is climbed with
`native_merge`, which does not separate leaf from node: a 31-level path whose «leaf» were the
level-1 node would climb to the same root, and that node is not the zero. A verifier that
accepted the depth the path brings would take any account as frozen. The core publishes the
depth (`FROZEN_DEPTH`, 32, `spec/NUCLEO.md`) and the verifier demands that one and the crossing
with the bits of the index (`zk-ssl-verify`, module `congelados`). An unmigrated ledger
reconstructs the tree at 24 levels: its path does not pass, and it fails closed.

⚠️ **Additive**: the surface goes from 26 to 28 methods (`zkssl_frozenPath` and `dev_freeze`) and
`zkssl/0.3` does NOT go up: it changes no value that already travelled.

## What the claimant needs from the snapshot (§493, RFC-0008 D-F)

`zkssl_pendingPath {index, viewKey, position, salt, amount, x}` returns, for the receiver
`index`, what the producer of the pending claim (RFC-0008 E1, §491) needs: the path of its
pending in the pending tree, the siblings of the same position in the meta tree and the
meta `(emisor, nacido)`: `{available: true, s, caminoPendiente: {siblings, isRight},
hermanosMeta, emisor, nacido}`. With that and a signed head of `seq` `s` the claimant proves,
without the node, that its pending exists under `pendingRoot` and `pmetaRoot` of that head.

- **It is not of the state of NOW: it is of the SNAPSHOT of the last heartbeat.** The pending
  tree moves with every payment, so a path of the state of now stops climbing to the signed head
  as soon as a payment lands. The heartbeat takes the snapshot —the two trees and the meta— under
  the same lock with which it composes the head, and `s` is the `seq` of that heartbeat: the path
  climbs to the `pendingRoot` of the head that `zkssl_signedEpochHead` serves with that `seq`, and
  to no other.
- **Demands the receiver's credential** (the `viewKey` of §261); without it, `-32004`. And the
  layer only serves if the notice recomposes the leaf of that position
  (`C2 = M(H(H(receptor, salt), amount), x)`): someone else's notice, one with another amount or
  another salt, and a free position receive the same response, `{available: false, reason}`,
  without saying what there is. `x` is mandatory: the v1 pending has no envelope and there is no
  `C2` to recompose.
- **The PAYER asks with `receiverId`** (RFC-0008 D-AE, §505): the credential it brings is ITS OWN
  (`index`, `viewKey`), it names the receiver with the public identity it gave to
  `zkssl_sendMaterials`, and composes the whole notice with its opening —position, salt, amount
  and the `x` of its envelope—. The node serves it exactly what it serves the receiver (the same
  paths, the same meta, the same `s`), but only if the meta of that position names it
  (`emisor == index`): a third party with a valid credential, a wrong receiver and an already
  refunded pending receive the same nothing as someone else's notice, without saying what there
  is. With `receiverId` whoever asks is ALWAYS the payer, even if it names its own identity. And a
  warning measured before §505: an earlier node IGNORES `receiverId` —the method never rejected
  extra fields— and answers as if it had not come, `{available: false, reason}` and not `-32602`;
  what says whether a node knows what it is talking about is its `spec/openrpc.json` (it carries
  `receiverId` since §505), not the version, which does not go up.
- **Without a heartbeat, and without `--clave`, there is nothing to serve**, and `reason` says
  which of the two: the claim's envelope is tied to a signed head.
- **What it reveals besides the path**: the level-0 siblings are the neighbouring leaf of the two
  trees (an opaque commitment and the hash of a meta); those higher up say which subtrees have no
  pendings. It is what the `pendingPath` of `zkssl_sendMaterials` already reveals.

⚠️ **Additive**: the surface goes from 28 to 29 methods (`zkssl_pendingPath`) and `zkssl/0.3`
does NOT go up: it changes no value that already travelled. And §505 (RFC-0008 D-AE) adds an
OPTIONAL `receiverId` to the same method: additive too, the surface stays at 29 and `zkssl/0.3`
still does not go up.

## The pledge, with its envelope (§519, RFC-0008 E3)

`zkssl_pledge {prueba, receptor, marca, seq}` publishes the MARK of a pledge DEMANDING the
envelope that holds it up: `{accepted: true, yaEstaba, logSeq, s}` if the proof verifies, and
`{accepted: false, reason, data}` if not. The pledge is the PAIR —the mark under the signed root
plus the envelope (RFC-0008 D-AS)—, and this method is the only place where the two halves come
together: the judge of `zk-ssl-air` verifies the envelope against a statement, and the node
writes the consumption tree. What comes out of the pledger is `SobrePrenda {prueba, receptor,
marca, seq, pendingRoot}` (§518); from it come the four parameters, and `pendingRoot` **does not
travel**. ⚠️ §815: «`{accepted: false, reason, data}` if not» has one exception, the store failure
on recording the consumption (`Store`), which no longer comes out as `accepted: false` inside the
result: it is the PARADA, `-32603`, without `data` and without a receipt, because its outcome is
not known (below, «Carries a receipt»).

- **The root is set by the node, never by the caller.** The statement is composed with the
  `pendingRoot` of the signed head the node custodies, not with the one the request brings:
  serving that yardstick would be letting the one who asks fabricate the one it is measured with
  (§248). That is why `seq` is a parameter and `pendingRoot` is not.
- **Verifies BEFORE writing, and if it does not verify it writes nothing.** The judge
  (`zk_ssl_air::prenda::verificar_contra_cabeza`) declares in its own doc that it does not check
  that the mark is under `consRoot` because «es la puerta de quien escribe en él» [it is the door
  of whoever writes into it]: whoever writes is this node, and here the pair is closed.
- **It is not a door of the consumption tree** (RFC-0008 D-AT). The mark alone still enters
  through `zkssl_publishConsumo`, which asks for neither proof nor authorization, and that does
  not change: this is an entry point WITH proof beside a free entry point. **And it demands no
  credential**: the authorization is the proof, not the possession of a view key. It is the only
  entry point of the wire that verifies a STARK.
- **A pledge is valid within its epoch.** The node custodies ONE signed head, so `seq` has to be
  that of the last one; if it is not, `{accepted: false}` and `reason` **names** the one there is.
  The opposite of the mute refusal of `zkssl_pendingPath`: here there is nothing to hide, because
  the mark is public and precomputable by anyone who has the notice. The order is
  `zkssl_pendingPath`, produce, `zkssl_pledge`, under the same heartbeat.
- **A repeat whose envelope verifies is NOT a failure** (RFC-0008 D-BB). If the leaf was already
  there and it is the same, the response is `accepted: true` with `yaEstaba: true` and the layer
  is not touched: the pair exists as soon as the envelope verifies, whoever wrote it —the mark the
  payer writes is the SAME leaf (D-AT)—. `yaEstaba` is the only thing in the response the caller
  cannot compute on its own. A `ConsumoColision` is a rejection, and it travels with its cause as
  data, in the form of §454.
- **What it reveals**: nothing that was not public. The mark is `H(DOMINIO_PRENDA, C2)` and
  whoever has the notice precomputes it; the envelope carries neither the amount, nor the salt,
  nor `X`, nor `C2`, nor the key (RFC-0008 D-AW).
- **Carries a receipt when its proof is EVALUATED** (§611, RFC-0014 E3, D-D): `recepcion`, with
  the form of the direct path's and `hashPrueba` the digest of the proof that arrived (§116), in
  ALL the responses that reach the judge -accepted, `yaEstaba`, does not verify, or rejected by the
  layer-. The three that do not reach it -no heartbeat, unsigned head, old `seq`- do not carry it
  nor consume a number: they are noise, and whoever pledges proves again under the new head. Since
  §613, the one the layer rejects also carries it INSIDE its `data`, like the rejection of the
  batch and that of the direct path: its rejection envelope is assembled with that `data` as is.
  ⚠️ §815: «in ALL» has since §815 two exceptions, those of the direct path («The reception receipt
  in the response»): a store failure on recording the consumption leaves no leaf and answers the
  PARADA, without a receipt; and if the log fails after the verdict, the response comes out with
  its verdict and without `recepcion`, and the node halts.

⚠️ **Additive**: the surface goes from 29 to 30 methods (`zkssl_pledge`) and `zkssl/0.3` does NOT
go up: it changes no value that already travelled.

## Shutdown — the end of life, declared (note 91)

Every settlement layer ends — by closure, by migration or by
abandonment. What this protocol declares for that day:

**What the operator publishes on closing**: nothing that is not already published.
The two responses that hold up a position — `zkssl_signedEpochHead`
and `zkssl_ackPath` — travelled on every heartbeat; an orderly closure only
demands serving them until the last one.

**What the holder takes away**: what it already custodies. The last signed
head (the eighteen fields with `publicKey`, `epochDigest`,
`formatVersion`, `index` and `signature`, together, from the same heartbeat), the
`hashPrueba` of its entry (the recorded `proofDigest`, served in the
acknowledgement of the response and in `zkssl_logEntry`), and the acknowledgement path
(`s`, `siblings`, `isRight`).

**With what it holds up its position afterwards**: the portable evidence
package — format declared in `spec/PAQUETE.md`, verified by the binary
`zk-ssl-verify` — the wire's responses AS IS, gathered in a
file and verified without the node, without the layer and without the prover.

**UPDATE (§322): the package has format v2.** v2 adds a
`cofirmas` key with the response of `zkssl_cosigs` AS IS, without
rewriting, and the command checks that each one accredits THAT head and THAT
operator. The binary reads **v1 and v2**: what is custodied does not expire. A v1 that
brings `cofirmas` is REJECTED, because raising the version is exactly what
makes them part of the contract.

⚠️ **The package REPORTS, it does not judge.** It says how many co-signatures verify
against the packaged head. **Which witnesses count and how many are needed
is decided by the CLIENT**, with its policy, not by the package: whoever assembles it
may be the operator. The policy's controls live in the witness.

The line above is kept and cited instead of being deleted (§247).

⚠️ It is the opposite of the characteristic failure of validium: here there is no
committee to ask for the data, because the holder already has them. **Nobody
stays inside.**

**The demonstration, not the promise**: `tools/banco_apagado.sh` brings up a
real node, funds a position, captures the package over the wire, kills
the process with `kill -9` and verifies in VERDE — and in ROJO the same
package with a single tampered nibble. **A system that says how it dies
is demonstrated by dying**, not by promising.

## The wire's rejections, with a vector (§409, RFC-0005 E3)

The reference's consumer of the wire is the witness (`zk-ssl-cli witness`), and since §409 it
judges a response of `zkssl_signedEpochHead` **from a file and without a node** with
`--respuesta <cuerpo.json>`: the JSON-RPC body as it crosses the wire goes through the same path
as a live round (`result`, wire typing, key anchor, signature, recomposition) and comes out with
the class the witness would note. Contract: exit `0` only if the class is `nueva`; any other,
`ROJO: <clase> · <texto>` and exit `1`; usage, `2`. It does not judge the second channel
(consistency): with fresh memory there is no pair.

The vectors live in `spec/vectors/cable/` —never among the `zkssl-0.N.json`, which are the
scenario's—: one positive (`nueva`) per head era and one negative for each rejection MEASURED
before writing it (`PASTE-409-S`, `PASTE-453-M`). Those of the v3 era derive by mutation from the
real head of `spec/vectors/paquete/posicion-v2.json`; the two `rechazo-v5-*` of §452, from the
real v4 head `nueva` of `spec/vectors/consumo/consumo.json`; and the v5 positive is a REAL v5 head
that `tools/banco_consumo.sh --guardar` captured (§453), from which its recomposition negative
derives by mutation. The v6 positive (§570) is another REAL head from the same bench, against the
node that already signs v6, with the genesis pair —the consumption bench receives no sends:
`recepCount` 0 and the root of the empty tree—; from it derive by mutation its two negatives,
without `recepRoot` and with `recepCount` + 1. From the same v6 positive derive the two of §809:
`rechazo-index-atrasado`, with the declared `index` equal to the one embedded in the signature,
and `rechazo-n-no-canonico`, with `n + p`, which recomposes the same digest if the reader reduces
instead of rejecting (RFC-0016). The witness already rejected them, and a consumer without those
two rules passed the 21 entries of before. From the same v6 positive derive the three of §850, a
single mutation of the spelling each and none of the value: the `index` with a leading zero
(`0x06`), the QUANTITY `mmrSize` with a leading zero (`0x05`) and the `epochDigest` in upper
case. The QUANTITY goes without leading zeros (the section «Encoding») and the hex, in the
canonical one, `0x` and `[0-9a-f]` in pairs (`hex_canonico`, in «The composition of `chain`», and
the DATA, Digest and Q schemas of «The list of `zkssl/0.5`»; ⚠️ §914: it said 0.4); the witness
rejects them on typing, as `forma del cable`. And an entry without a file, `no-existe.json`: a
response that cannot be read -the file does not exist, is a directory or is not UTF-8- also comes
out with `ROJO: respuesta ilegible · …` and exit `1`; until §850 the witness gave the I/O error
there, with exit `1` and no `ROJO:` line.
`MANIFIESTO.txt` pins, per file, the exit code and the text the output has to contain;
`tools/conformidad.sh tools/cable_respuesta.sh spec/vectors/cable/MANIFIESTO.txt` runs them —the
same harness as the package, without doubling its contract— and `tools/canon.sh` does it on every
canon. A second implementation of the consumer puts its executable in place of the adapter. Since
§809 there is one: `tools/segunda/cable.py`, in Python, without a line of Rust or code of the
reference, gives 27 of 27 (⚠️ §850: it said 23 of 23, from before the three spelling negatives and
`no-existe.json`); what this section does not write —which keys are demanded, in which order they
are looked at, with what text what the manifest does not pin is rejected, and the limits of the
reference's reader: the nesting, the range of the numbers and the lone UTF-16 surrogates— was
fixed by the measured behaviour of the witness
(`tools/segunda/README.md` §«El segundo consumidor del cable» [the second consumer of the wire]).

**Residual trust, measured:** the class does not distinguish the cause. A `formatVersion` outside
the set `VersionCabeza` accepts comes out `no-verifica` live and `firma-no-verifica` in
`--auditar`, and `version-desconocida` is the version of the JOURNAL (`v`), not the head's. The
text does distinguish it, and that is why the manifest pins class and text. Opening a class would
raise `DIARIO_VERSION`: a cut of its own, not E3's.

## Operational notes

- One node, one writer: writes serialize in the node (the order
  of operations is the operator's by design; distributed consensus is
  another problem and is not implemented).
- Parameters of a persisted ledger: immutable
  (`ParameterMismatch` on reopening with other values).
- Versioning: ⚠️ §801: the rule is written by «Versioning and compatibility», at the end of this
  document (§801), which governs since §799, the seal in which the author accepted RFC-0021 with
  its decisions (⚠️ §826: it said «since the seal that accepts RFC-0021»; §799 left it PROPOSED
  until its stages); this entry tells what came before, which is not rewritten. Under that one,
  compatibility is governed by six surfaces, each with its identifier —what they lack today is
  declared there—, and the wire version goes up with every break of the wire, of the proofs that
  travel or of what is signed, also with a new head version the node issues, which from v3 to v6
  did not raise it (RFC-0021, fact 3). `zkssl_protocolVersion` governs compatibility. **The
  current version is `zkssl/0.5`** since §914, the cut of the train (RFC-0018; `0.4` governed
  from §538, `0.3` from §354, `0.2` from §209), and
  what raises it is that
  the **values that travel** change, not the size of the surface:
  adding a method additively —like `zkssl_applyMany` in §222— does not
  raise it, because the conformance vectors do not move.
  ⚠️ **Corrected in §699**: the compatibility that governs is that of the values the conformance
  vectors check, and these do not compare proof bytes (RFC-0009, D-AI). The proofs
  can change without the version going up, and they have changed.
- **What `zkssl/0.4` does not identify (§699).** Which proofs a node accepts. Within `0.4` the AIR
  of the proofs that travel changed three times without raising it:
  - in §641 (RFC-0017 E2, the range to 62 bits), the periodic selector of the seventeen
    circuits of the layer;
  - in §680 (RFC-0018 E1), the accumulators of the position in the five circuits of send,
    claim and burn;
  - in §684 (RFC-0018 E2), the threshold circuit —issuance, freezing, recovery, payment to
    pending and governance—: the trace goes from 18 to 21 columns and the nullifier changes value.

  Measured in §699 with the send, between nodes and clients built at six points of the
  history that all say `zkssl/0.4` —§622, §640, §641, §679, §680 and the tree of
  §699—: when §641 or §680 lies between the two, the send of one side does not verify on the
  other, in both directions, with `VerificationFailed` («envio: InconsistentOodConstraintEvaluations»),
  and each cut was measured against its parent commit; between §622 and §640, between §641 and
  §679, and between §680 and §699, the send and the claim pass in both directions. The threshold,
  by its form and unmeasured. From §538 to §622 it was not measured: the history the measurement
  reached starts at §622. Until the cut of the `zkssl/0.5` train (RFC-0018), which raises the
  version and issues its vectors, two implementations that say `zkssl/0.4` may not accept each
  other's proofs: what identifies them today is the commit, not the version. The kit does not
  verify these proofs, and what it accepts does not change. There is no deployed ledger
  (`SECURITY.md` 3.6).
- **The `zkssl/0.2` vectors were not kept as they were issued (§699).**
  `vectors/zkssl-0.2.json`, issued in §209, was re-issued three times under its same version: in
  §275 (the v2 head: `epoch_digest` and `sellado` change), in §278 (four `proof_digest`, all the
  chains and the head) and in §281 (the six chains, the head and a new field,
  `compromiso`, in every entry). What is published is the issue of §281: an implementation of `0.2`
  prior to that entry does not reproduce it, and the label does not tell them apart. It was against
  rule 2 of `rfc/PROCESO.md`; since §692 the fingerprint of each vector (`vectors/HUELLAS.sha256`)
  prevents it.

## Versioning and compatibility

This section is the protocol's promise to whoever implements it and to whoever custodies what it
produces. It governs since §799, the seal in which the author accepted RFC-0021 with its decisions
—the RFC remains PROPOSED until its stages are built (rule 4 of `rfc/PROCESO.md`)—, and it is
written here since §801; what came before is told by «Versioning», in «Operational
notes», which is not rewritten. ⚠️ §826: it said «It governs since the seal that accepts
RFC-0021». §799 accepted its decisions and left it PROPOSED; read as the seal that moves it to
ACCEPTED, which arrives with its E4, the promise would govern from the cut that premieres it. But
the author's decision 4 in §799 fixes the notice in a seal with the RFC PROPOSED before the cut
(R5), and that only binds if the promise governs while the RFC is PROPOSED. §801 left it to the
author to say which of the two seals it is; §826 settles it, by its delegation («HACER todo») and
with that criterion, and it is reversible. The same clarification goes in this section of
`spec/RPC.md` and in D-C of RFC-0021, which proposes it.

**The stable surface** is six surfaces, each with its identifier: (1) the wire,
`zkssl/0.N`: the `zkssl_*` methods of `openrpc.json`, their parameters, their results, their errors
with their cause in `data`, and the encoding; (2) the proofs that travel, by the mark of their
trace; (3) what is signed —the head, the co-signature and the act—, by its version, with its
preambles and its compositions (`NUCLEO.md` §6); (4) the kit's envelopes, by their `v` and their
`tipo` (`PAQUETE.md` §2), with their catalogue and their causes; (5) the vectors of `vectors/`, by
their path, their manifest line and their fingerprint; (6) the kit, `arqueo-verify-vX.Y.Z`, by its
tag, its commit and its fingerprints, with the contract of `PAQUETE.md` §6. The rest —the `dev_*`,
the crates' API, the command-line options, the journals, the ledger on disk and the operating
limits— is not promised.

**Breaks** whatever makes an implementation conformant with the previous version produce or accept
something else in one of the six: a value that travels, also the bytes of a proof or its AIR, and
what the node signs, also a new head version it issues; removing or renaming a method, a
field or a `tipo`; the verdict or the text of a published vector; removing a version from an
accepted set; an exit code of the kit. **Does not break** what is additive and nobody issues yet.

**Raises** `zkssl/0.N` with every break of (1), (2) or (3); the `v` of the envelope, or a new
`tipo`, with one of (4); the kit moves to `0.N.0` with the wire and not before, and a `0.N.x`
breaks nothing except by a security fix. Breaks are gathered in a cut, and its RFC is in the tree
before it.

**After a break it still verifies**: every published kit, which is not withdrawn and is rebuilt
from its commit —with the same fingerprint, only demonstrated within one same machine: `doc/KIT.md`
§«Reproducir el kit, para quien no se fíe de la descarga» [reproducing the kit, for whoever does
not trust the download]—; every vector, which is not rewritten and, if its verdict changes, moves
to `vectors/<versión>/` with that of its kit; every head a kit accepted, in every later kit,
because the set of head versions only grows, except that of a broken signature scheme, which is
valid contained in something signed with an accepted one; every envelope, with the kit of its
version, and with a later one it either verifies or is rejected as «outside the contract»; and
every proof, with the kit of its wire version, because no kit accepts two. A security fix can
change a verdict within `0.N.x`, with its notice and the list of what changes. ⚠️ §823: it said
«which is not withdrawn and is reproduced from its commit», without the machine condition: redone
from its commit on another one, 0.4.2 comes out with 13 of its 35 sections different, and judges
its eleven catalogues the same.

**What the version does not identify yet** is declared here, with its entry, until it
identifies it.

### What `zkssl/0.4` does not identify yet

⚠️ §914: the title stays, because it is cited by its name; what follows was written under
`zkssl/0.4`, and since the cut `zkssl/0.5` the second of the three, the one of the proofs, is
identified by the v2 mark (the notes below say so where it belongs).

Each surface must have an identifier, and a producer from which the other
writings derive (R7 of RFC-0021). Today three are missing (⚠️ §914: two; the second, the one of the
proofs, has it since the cut):

- **The wire version has no producer.** It is written, with nothing tying them, by the node's
  response to `zkssl_protocolVersion`, the OpenRPC, twice, and the issue of the conformance
  vectors; the tests that check it write it once more (RFC-0021, fact 5, measured in
  §790). The list below names each production writing, and the gate demands that all
  say the same. When the version goes up, it comes to have one (R3 of RFC-0021). ⚠️ §716: the issue
  of the vectors no longer writes the wire's, but the conformance scenario's, `zkssl/0.5`, which
  goes up before the wire because the commitment of the two issuances carries since §716 the
  issuance counter (RFC-0018, «Cómo se construye» [how it is built]). The list fixes it apart, in
  `escenario`, and the wire's writings are three.
- **A `zkssl/0.4` proof does not say which AIR it is from** (§699): those that travel changed in
  §641, §680 and §684 without raising the version, and what identifies them today is the commit;
  the detail, in «Versioning», in «Operational notes». The mark that will identify them is D-G of
  RFC-0018 (entry 118 of `BACKLOG.md`), and the gate that will see a changed AIR, E3b of RFC-0021,
  enters with the cut. ⚠️
  §723: and in §723 two more changed, already under this promise: the kit's band and pending
  claim, which travel in the `InsufficientBalance` rejection envelope and in the claim's, move to
  their v2, with the three range segments at 62 bits (the correction of D-3 of RFC-0017). It is a
  break of (2) that does not raise the version, and the `zkssl/0.5` cut pays for it, like that of
  E2 of that RFC (§826, its «Compatibilidad» [compatibility]). Until the cut, the kit judges the v2
  and, if it does not verify, the v1, and 0.4.3 and the earlier ones do not verify the v2. ⚠️ §914:
  **paid for and resolved**: since `zkssl/0.5` every proof carries the v2 mark (RFC-0018 D-G) with
  its family, the version of its AIR and its profile, the judge reads it before anything else, and a
  `zkssl/0.4` proof falls by name (`marca v1: prueba de zkssl/0.4; la juzga el kit 0.4.x`); the
  registry of families and versions is `NUCLEO.md` §4 bis, and the extension rule of D-G raises the
  version of the AIR with whatever changes.
- **The head and the co-signature do not say with which signature scheme they were signed**: what
  is signed does not name it (the act names, since §643, that of the key that comes in). Putting
  it inside is RFC-0020 (entry 87), accepted with RFC-0021 in §799.

### The list of `zkssl/0.5`

What each surface that has an entry is today, read from its producer and fixed in §801 for
`zkssl/0.4` and in §914 for `zkssl/0.5`: the current version, and with another version, another
list. `tools/check_superficie.py`, in the canon's loop (E3a of RFC-0021), reads it again from each
producer and compares it with this list: an entry its producer no longer gives, or gives changed, is
ROJO; what is additive and nobody issues yet —a method, an optional parameter, a `tipo`, a vector,
an accepted version nobody issues— is counted and is not ROJO. Its rules, what it reads and what it
does not see are in its header; `--lista` prints what the producers give today, in this same form,
and writes nothing.

| key | surface | its producer |
|---|---|---|
| `cable`, `cable.escrituras` | (1) | the production writings of the version, in `crates/` (today without a producer) |
| `escenario` | (5) | the `spec:` of `crates/zk-ssl-cli/src/conformance.rs`: the version of the conformance scenario, which since §716 runs ahead of the wire until the cut (RFC-0018) |
| `metodo`, `esquema` | (1) | `openrpc.json`, which `zk-ssl-wire` generates; the `dev_*` do not enter |
| `cabeza.acepta`, `cabeza.emite`, `cofirma.emite`, `cofirma.lee-hasta`, `acta.emite` | (3) | `VersionCabeza::TODAS`, `VERSION_FORMATO`, `COFIRMA_VERSION`, `COFIRMA_V_MAX` and `ACTA_VERSION`, in `zk-ssl-verify` |
| `sobre.v`, `sobre.tipo` | (4) | the kit's dispatch, `verificar_paquete`, against `PAQUETE.md` §2 |
| `vectores.json`, `vectores.lineas`, `manifiesto` | (5) | `vectors/`; their fingerprints are watched by `tools/check_vectores.py`, which the canon runs |
| `kit.salida`, `familia` | (6) | `codigo_de_salida` and the usage, in the kit, against `PAQUETE.md` §6 and section 0 of `doc/KIT.md`; `FAMILIAS`, in `tools/artefacto.sh` |

Surface (2) has no entry: it has no identifier (above). ⚠️ §914: it has one, the v2 mark (family,
version of the AIR and profile, `NUCLEO.md` §4 bis); its entry in the list, with
`tools/check_superficie.py` reading it from the registry, is left for a later seal.

```superficie
cable zkssl/0.5
cable.escrituras zk-ssl-node/src/main.rs zk-ssl-wire/src/openrpc.rs×2
escenario zkssl/0.5
metodo zkssl_protocolVersion() -> ProtocolVersion
metodo zkssl_params() -> Params
metodo zkssl_epochHead() -> EpochHead
metodo zkssl_supply() -> Supply
metodo zkssl_accountCount() -> Q
metodo zkssl_publicId(index: Q) -> Digest
metodo zkssl_accountView(index: Q, viewKey: Digest) -> AccountView
metodo zkssl_logEntry(seq: Q) -> LogEntry
metodo zkssl_logEntries(fromSeq?: Q, limit?: Q) -> LogEntries
metodo zkssl_verifyChain() -> VerifyChain
metodo zkssl_openAccount(publicId: Digest, viewId: Digest, leafSalt: Digest) -> Opened
metodo zkssl_sendMaterials(sender: Q, viewKey: Digest, receiverId: Digest, amount: Q, salt: Digest) -> SendMaterials
metodo zkssl_applySend(receipt: SendReceipt, sender: Q, senderState: ClientState, amount: Q) -> Applied
metodo zkssl_claimMaterials(receiver: Q, viewKey: Digest, notice: PendingNotice) -> ClaimMaterials
metodo zkssl_applyClaim(receipt: ClaimReceipt, receiver: Q, receiverState: ClientState, notice: PendingNotice) -> Applied
metodo zkssl_applyMany(ops: BatchOp) -> BatchApplied
metodo zkssl_signedEpochHead() -> SignedEpochHead
metodo zkssl_inclusionReceipt(index: Q, viewKey: Digest) -> InclusionReceipt
metodo zkssl_ackPath(seq: Q) -> AckPath
metodo zkssl_consistencyProof(oldSize: Q) -> ConsistencyProof
metodo zkssl_submitCosig(cosig: Cosig) -> CosigAccepted
metodo zkssl_cosigs(epochDigest?: Digest) -> Cosigs
metodo zkssl_publishConsumo(consumo: Digest) -> ConsumoPublicado
metodo zkssl_consumoPath(consumo: Digest, seq: Q) -> ConsumoPath
metodo zkssl_frozenPath(index: Q, viewKey: Digest) -> FrozenPath
metodo zkssl_pendingPath(index: Q, viewKey: Digest, position: Q, salt: Digest, amount: Q, x: Digest, receiverId?: Digest) -> PendingPath
metodo zkssl_pledge(prueba: DATA, receptor: Digest, marca: Digest, seq: Q) -> PrendaPublicada
metodo zkssl_recepPath(rx: Q) -> RecepPath
metodo zkssl_keyActs() -> KeyActs
esquema Bool boolean
esquema DATA string pattern ^0x([0-9a-f][0-9a-f])*$
esquema Digest string pattern ^0x[0-9a-f]{64}$
esquema ProtocolVersion string const zkssl/0.5
esquema Q string pattern ^0x[0-9a-f]+$
cabeza.acepta v2 v3 v4 v5 v6
cabeza.emite v6
cofirma.emite 1
cofirma.lee-hasta 1
acta.emite 1
sobre.v 1 2
sobre.tipo extension consumo conflicto rechazo edad cobro_pendiente pago_en_curso prenda completitud ancla ancla-cofirmada solapamiento
kit.salida 0 1 2 3
vectores.json 702
vectores.lineas 1740
manifiesto 0.3/edad/MANIFIESTO.txt
manifiesto 0.3/pago/MANIFIESTO.txt
manifiesto 0.3/pendiente/MANIFIESTO.txt
manifiesto 0.3/rechazo/MANIFIESTO.txt
manifiesto 0.4/completitud/MANIFIESTO.txt
manifiesto 0.4/edad/MANIFIESTO.txt
manifiesto 0.4/pago/MANIFIESTO.txt
manifiesto 0.4/pendiente/MANIFIESTO.txt
manifiesto 0.4/prenda/MANIFIESTO.txt
manifiesto 0.4/rechazo/MANIFIESTO.txt
manifiesto ancla-cofirmada/MANIFIESTO.txt
manifiesto ancla/MANIFIESTO.txt
manifiesto cable/MANIFIESTO.txt
manifiesto completitud/MANIFIESTO.txt
manifiesto conflicto/MANIFIESTO.txt
manifiesto consumo/MANIFIESTO.txt
manifiesto edad/MANIFIESTO.txt
manifiesto nodo/MANIFIESTO.txt
manifiesto pago/MANIFIESTO.txt
manifiesto paquete/MANIFIESTO.txt
manifiesto pendiente/MANIFIESTO.txt
manifiesto prenda/MANIFIESTO.txt
manifiesto rechazo/MANIFIESTO.txt
manifiesto rotacion/MANIFIESTO.txt
familia paquete consumo conflicto rechazo edad pendiente pago prenda completitud ancla ancla-cofirmada rotacion
```
