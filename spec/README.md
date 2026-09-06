# The Arqueo protocol specification — a reader's guide

This folder is the **normative surface** of the protocol: everything that
crosses the wire between a node, a client, a witness and a verifier, plus
the evidence package a verifier accepts with no wire at all. The
files here are the source of truth; this page only tells you what each one
is, in what order to read them, and where the claims that matter live. It
adds no rule of its own. The specification files are written in Spanish;
the line references below point into them so that a reader of either
language lands on the same text.

Verified against `main` at commit `c43890a`. Line numbers are those of that
commit; if a file has moved on, the reference tells you where to look.

## What is in this folder

| file | what it is | read it when |
|---|---|---|
| `RPC.md` | the JSON-RPC specification of the node API, version `zkssl/0.3` — transport, encoding, every method, every error, and what the transition log guarantees | you implement a node, a client or a verifier |
| `openrpc.json` | the same API as a machine-readable OpenRPC 1.2.6 document: `info.version` is `zkssl/0.3` and it lists 24 methods (22 `zkssl_*`, 2 `dev_*`) | you generate a client or check a node's surface |
| `vectors/zkssl-0.1.json`, `vectors/zkssl-0.2.json`, `vectors/zkssl-0.3.json` | the conformance vectors, one file per wire version, never rewritten | you check that an implementation produces the same values on the wire |
| `PAQUETE.md` | the portable evidence package: the three forms (v1, v2 with co-signatures inside, extension), the envelope keys the verifier reads, the order of checks, the rejection catalogue and the exit contract of `zk-ssl-verify` — it does not cross the wire | you build or verify an evidence package, or write a second verifier |
| `NUCLEO.md` | the frozen core: every public element a verifier reaches in `zk-ssl-verify` and `zk-ssl-hash`, classified (core / reference / ledger / log) with the extension rule; the table is derived from the source and gated by `tools/check_nucleo.py` in every canon | you write a second verifier and need to know exactly what must be reproduced byte for byte |
| `vectors/paquete/` | the vectors of the evidence package — positives per form, one negative per producible rejection rule, and `MANIFIESTO.txt` with the expected exit code and message of each | you check that a verifier accepts and refuses exactly what `PAQUETE.md` says — `tools/conformidad.sh <binary>` runs the whole manifest against any binary and ships inside the artefact |
| `vectors/cable/` | the negative vectors of the WIRE: what the reference consumer (the witness, `witness --respuesta`) refuses, one per measured rejection, plus one positive; `MANIFIESTO.txt` holds exit code and text (`RPC.md`, section "Los rechazos del cable") | you write a second consumer of `zkssl_signedEpochHead` |
| `vectors/nucleo/` | the known-answer vectors of the frozen core: one file per NÚCLEO function of `zk-ssl-hash`/`zk-ssl-verify`, `{fn, entradas, salida}` in hex, emitted by the reference and reproduced on every canon — the bytes a second implementation must match before anything else (`NUCLEO.md`, section 6) | you implement the hash, the compositions or the preambles from `NUCLEO.md` |
| `rfc/PROCESO.md` | the RFC process: states, five rules, and what an accepted change must ship with | you want to change anything above |
| `rfc/0000-plantilla.md` | the RFC template | you write an RFC |
| `rfc/0002-lotes-y-transicion-de-hoja.md` | RFC-0002: batches, and the leaf transition that took the wire from `0.1` to `0.2` | you read the log guarantees under batching |
| `rfc/0003-compromiso-v2.md` | RFC-0003, ACCEPTED: the pending commitment v2 (per-payment expiry and refund identity committed), which took the wire from `0.2` to `0.3` | you read why the current commitment has the shape it has |
| `rfc/0004-paquete-de-evidencia.md` | RFC-0004, ACCEPTED: the portable evidence package gets its own normative document (E1, `PAQUETE.md`), its vectors (E2) and the head-index binding (E3) | you read why the package is specified apart from `RPC.md` |
| `rfc/0005-nucleo-congelado.md` | RFC-0005, PROPOSED: the frozen core and the extension rule — the core is the verifier side, not the wire (D-A); what is signed grows only by version and what is not signed does not exist for the core (D-B); nothing in the core identifies anything outside the ledger (D-C). E1 (`NUCLEO.md`) and E2 (`VersionCabeza`) sealed | you read what will never change, and why |
| `rfc/0006-consumo-publicado.md` | RFC-0006, PROPOSED: the published consumption — a public, precomputable label `H(domain, agreed identifier)` accumulated in a sixth root at rest, signed into the head as format v4 (`zkssl/0.3` → `0.4` in E2), provable without the node (E3) and detectable across two ledgers by reading two signed heads (E4). Uniqueness within a ledger; detection, not prevention, between ledgers; the binding to a payment is out of scope (E5) | you read how a unit is proved consumed once, and what that does not claim |

RFC-0001 is not missing: the number is reserved for the keystore KDF
hardening and is not yet drafted (`rfc/0003-compromiso-v2.md:11-15`).

## Three things that govern everything else

**1. The spend key never travels.** Opening an account sends identifiers
derived on the client; paying and claiming means asking the node for
public materials, proving locally, and presenting a receipt; reading your
own balance means presenting a derived view key that authorises reading
that account only (`RPC.md:50-59`). Every RFC must declare its effect on
this principle, and one that erodes it is born withdrawn
(`rfc/PROCESO.md:19-20`).

**2. The version number tracks the values on the wire, not the size of
the surface.** `zkssl_protocolVersion` governs compatibility. The version
in force is `zkssl/0.3` (since audit entry §354; `0.2` ruled from §209).
It goes up when values that travel change; adding a method additively
does not raise it, because the conformance vectors do not move
(`RPC.md:863-867`). When it does go up, the old vectors stay under their
version and are never rewritten (`rfc/PROCESO.md:16-18`).

**3. No statement enters the specification without the witness that would
falsify it.** That is why the log-guarantee section of `RPC.md` was
written *before* batching was implemented (`RPC.md:142-148`), why every
RFC is tied to an audit entry by number in both directions
(`rfc/PROCESO.md:23-24`), and why each vector file carries the seal it
was issued under (its `sellado` key).

## Reading `RPC.md` in order

- **Transport and envelope** (`RPC.md:25-37`). HTTP `POST /`, JSON-RPC 2.0,
  one object per request; JSON-RPC batches are not accepted. The default
  body limit is 2,097,152 bytes, measured; one operation with its proof is
  about 132,728 bytes in hex, so a `zkssl_applyMany` holds 15.
- **Encoding** (`RPC.md:39-48`). `QUANTITY` is a u64 in `0x` hex without
  leading zeros; `DATA` is `0x` hex of even length; a `Digest` is 32 bytes
  in the same serialisation the layer persists. A non-canonical digest is
  rejected with `-32602` before the layer is touched.
- **Methods** (`RPC.md:61-129`). Read methods; `zkssl_openAccount` (an
  account is born with zero balance; the three identifiers are derived on
  the client); the two-phase payment (`sendMaterials` → `applySend` on the
  payer's side, `claimMaterials` → `applyClaim` on the payee's side,
  `applyMany` for a batch of operations against one root); and the `dev_*`
  namespace, which only exists in builds with the `dev` feature and uses
  test custodians.
- **Errors** (`RPC.md:131-140`). Three codes: unknown method, invalid or
  non-canonical parameters, and a layer refusal whose `message` is the
  layer's own error. `StaleState` is expected under concurrency: refresh
  the view and retry.
- **What the transition log always guarantees** (`RPC.md:150-168`).
  Consecutive sequence numbers from zero; `rootOld` of an entry is
  `rootNew` of the previous one, the first starting at genesis; `chain` is
  the running digest of the entry and everything before it; `proofDigest`
  ties the entry to what authorised it. The subsections that follow
  (`RPC.md:169-360`) say exactly which of these survive batching and which
  do not — read them before building on the log.
- **The signed epoch head** (`RPC.md:433-478`). `zkssl_signedEpochHead`
  returns the most recent head the node signed: the nine head fields,
  `publicKey`, `epochDigest`, `formatVersion`, `index` and `signature`,
  together, from the same heartbeat — one custody artefact. Format v3 adds
  `mmrRoot` and `mmrSize`, signed, so that a holder of an older head can
  verify that a newer one *extends* it without downloading the log. Three
  answers are possible and none is a generic error: no heartbeat yet, a
  node started without a key (the head comes unsigned and says so), or a
  signed head.
- **The extension proof** (`RPC.md:781-808`). `zkssl_consistencyProof`
  turns that check into a service: a holder sends its `mmrSize` and gets
  an O(log N) path against the history tree.
- **Inclusion, the acknowledgement path, and what a path exposes**
  (`RPC.md:564-735`). What a receipt proves, what an acknowledgement is,
  and the corrections the specification records about itself when an
  earlier statement turned out to be wrong (`RPC.md:651-694`). Those
  corrections are kept in place on purpose: the file narrates its own
  history.
- **Co-signature transport** (`RPC.md:737-779`). `zkssl_submitCosig` and
  `zkssl_cosigs` make the node the *transport* of witness co-signatures,
  not their authority: it checks that a co-signature is for the current
  epoch and that the signature closes, and it does **not** accredit the
  witness — which witnesses count is the client's policy, not the node's.
- **Shutdown** (`RPC.md:810-855`). What the operator publishes on closing
  (nothing that was not already published every heartbeat) and what the
  holder keeps: the last signed head, the proof digest of their entry, and
  the acknowledgement path.
- **Operating notes** (`RPC.md:856-867`). One node, one writer; ledger
  parameters are immutable once persisted; distributed consensus is a
  different problem and is not implemented.

## The vectors

Each `vectors/zkssl-X.Y.json` is one object with the same eight keys:
`spec`, `sellado` (the seal it was issued under), `escenario`, `canon`,
`entradas`, `epoch_digest`, `supply`, `pending`. An implementation of a
given wire version is checked against the file of *that* version; the
conformance check refuses to validate vectors of a different version, and
that refusal is correct (`RPC.md:11-13`). The three files are kept side by
side so that a `0.1` or `0.2` implementation can still be checked against
what it claims to speak.

`vectors/paquete/` holds the vectors of the evidence package (`PAQUETE.md`): positives
per form and one negative per published rejection rule that a real package can produce,
each listed in `vectors/paquete/MANIFIESTO.txt` with its expected exit code and message.
`tools/canon.sh` runs `zk-ssl-verify` on every one of them; a single altered nibble turns
the canon red.

## Changing any of this

A change to what crosses the wire — `RPC.md`, `openrpc.json`, the
vectors — does not enter by direct commit; it enters by RFC
(`rfc/PROCESO.md:3-7`). States: DRAFT → PROPOSED → ACCEPTED → FINAL, or
WITHDRAWN at any point (`rfc/PROCESO.md:8-10`). ACCEPTED requires the
specification updated, the OpenRPC document regenerated, the vectors
re-issued or new ones under the new version, and green suites
(`rfc/PROCESO.md:21-22`). The audit entry that seals the change cites the
RFC by number, and the RFC cites the entry (`rfc/PROCESO.md:23-24`).

## Where the rest lives

The audit record (`AUDITORIA.md` at the repository root) holds the
numbered entries cited above by `§`. The declared limits of the system are
in `SECURITY.md`. The evidence package a holder keeps after shutdown — what
`zk-ssl-verify` accepts and refuses — is specified in `PAQUETE.md`, above.
What a second verifier must reproduce byte for byte — the frozen core and its
extension rule — is enumerated, derived and gated in `NUCLEO.md`, above.
This page is maintained by hand: when you change a file
in this folder, update the line references here and the commit in the
header, and run `tools/verificar_citas.py`, which fails if a cited
document does not exist.
