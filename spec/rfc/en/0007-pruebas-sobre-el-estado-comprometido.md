# RFC-0007 — The proofs over the committed state: the rejection, the age and the parameters

> **Español:**
> [`spec/rfc/0007-pruebas-sobre-el-estado-comprometido.md`](../0007-pruebas-sobre-el-estado-comprometido.md)
> is the original of this page, in Spanish; it prevails wherever the two differ. This translation is
> of commit `7766efb` (audit entry §917) and was produced by the project's assistant, a Claude Code
> session, in audit entry §918 (`GENAI.md`): it is a generated artefact, not text written by the
> author. Line citations of the form `file:N` refer to the Spanish files.

- **Status:** ACCEPTED — **rule 4 of the PROCESO, settled with measurement** (§450 the adoption;
  E1 §451–§453, E2 §454, E3 §455–§456 and §458–§460, E4 §461–§467 and §472, E5 §468–§471 and
  §473–§479, the stages with their gates; §480 the spec up to date; §481 this turn). It is
  preserved as a record of what was decided, what was measured and what was discarded.
- **Author:** Ángel José Toranzo Portela
- **GenAI assistance:** Claude (session 119) — see [`GENAI.md`](../../../GENAI.md)
- **Date:** 2026-09-09
- **Affected protocol version:** `zkssl/0.3` — **does not go up** (see Compatibility). The v5
  head and the error's `data` are additive on the wire; the FORMAT version travels in the
  signature (4 → 5), as it went from 2 to 3 (§292) and from 3 to 4 (§415).
- **AUDITORIA entries:** §178, §211, §246, §253, §275, §292, §321, §379, §387, §388,
  §404, §406, §413, §414, §415, §441; §450, which adopts it; §451–§456 and §458–§479, the
  stages; §480, the spec up to date; and §481, this turn to ACCEPTED. §716 writes the
  correction of `AutorizacionConsumida`, a new cause of the layer, after those of §802 and §815.
  §717 corrects the ceiling of the age proof with the hidden trace, `n` = 8192, and the judge
  applies it.
- **Milestone:** H4 of the proposal sent to NLnet Restack (140 h), in its words: *«Rejection
  with proof and the ageing proof. Circuits over the committed state: proof of the cause of a
  refusal; age distribution of what is in flight with its empty-box form and its concentration
  and cap variants; parameter consistency committed in the epoch head. Format and vectors.»*

## Status of the stages

| stage | what it delivers | does it break the wire? | status |
|---|---|---|---|
| E1 — the v5 head | `epoch_digest_v5`: ONE new family under the signature —`params_digest` (the seven parameters), `pmeta_root`, `next_pending`, `next_index`, `total_supply`—; `VersionCabeza` gains V5; the wire serves the five fields and `zkssl_params` the three parameters it does not serve today; the witness recomposes and holds in custody; the command accepts; KAT and row in `NUCLEO.md`; vectors under their version; witnesses: recomposing rejects a v5 without one of the five, and a parameter changed at rest changes the head | NO on the wire (additive keys); YES in the signature (format 4 → 5) | **sealed** — §451 and §451-B (E1a: the core and the command accept v5), §452 and §452-B (E1b: `VERSION_FORMATO` 5, the node signs and serves v5, the wire demands it by version and the witness holds it in custody) and §453 (the v5 positive of the wire catalogue: a real head) |
| E2 — the rejection with a cause, on the wire | the error object gains `data`: the cause by its NAME (the `LayerError` variant), its fields, and the `seq` of the head in whose state it was judged; `message` does not change; the catalogue of the twenty-five causes is published in `RPC.md` and a test ties it to the enum (two lists are two producers) | NO (additive: no vector nor the OpenRPC pins the error object) | **sealed** — §454 (`LayerError::causa` in the layer, `data` in the `-32000` and in the negative of `zkssl_publishConsumo`, the catalogue in `spec/RPC.md` tied by test) |
| E3 — the rejection with proof, by paths | a new form of the evidence package, `tipo: "rechazo"`, that the command verifies without a node: the signed v5 head, the cause, and the material that proves it with paths and public arithmetic over what the head commits; a table cause → material → what it reveals; one positive and one negative per rule, derived by mutation; its manifest and its gate in the canon | NO (the package does not cross the wire) | **sealed** — §455 (E3a-1: `OverRegulatoryLimit`, `AccountLimitReached`, `ConsumoRepetido`, `ConsumoColision`), §456 (E3a-2: `StaleState`, `WrongRegulatoryLimit`, `DuplicateAccountInBatch`, `DuplicatePendingInBatch`, with a real receipt captured by the proxy of a bench), §458-§459 (E3b: `AccountFrozen`, with the frozen path that the wire serves to the holder) and §460 (`SupplyCapExceeded`, with the applicant's request inside the envelope). `AccountNotFound` moves to E5 (CORRECTION of §459); `PendingTreeExhausted` is left declared without a portable proof (CORRECTION of §460) |
| E4 — the age proof, measured first | E4a: the cost as a function of `next_pending`, with the instrument before the circuit, and a gate that decides whether it is built or a ceiling is declared. E4b: the circuit over the RANGE `0..next_pending` of the pending and meta trees, with the empty box, the cap and the concentration per named sender as forms of one same statement; its envelope, its manifest and its vectors | NO | **sealed** — E4a sealed: the instrument (§461, §461-B) and its measurement and verdict (§462): E4b is built, with the ceiling declared (CORRECTION of §462); E4b-1 sealed (§463, §463-B): the real AIR of the per-account forms, with its judge in `crates/zk-ssl-air` without the prover and the gate measured with it: the ceiling, measured at `n` = 16384, and the per-amount forms, declared (CORRECTIONS of §464); E4b-2 sealed (§465): the kit verifies it against a signed v5 head, without the prover; E4b-3 sealed (§466: the layer produces it over the ledger of a real node and REJECTS if the ledger does not reproduce the four figures the v5 head signs; §467: its catalogue, eleven vectors that the command runs without the node). **E4 is whole** |
| E5 — the causes by circuit, and the kit verifies | `InsufficientBalance` with a band proof over the committed leaf (mould `circuit_audit`, without the ownership cycle); the re-verification of the applicant's STARK for `ProofFailed` and `VerificationFailed`; a verifier-only AIR crate that the command consumes, with `winter-verifier` in its closure and the prover out | NO | **sealed** — D-F was settled by E4b (`crates/zk-ssl-air` lives in the kit's closure: 46 packages, zero prover, measured in §468). The THREE causes were split among D-G, D-H and D-I (§468): `AccountNotFound` WAS BUILT —the node produces the rejection envelope (§473), its material and its command (§474, §475) and its catalogue (§476)—; `InsufficientBalance` was MEASURED before writing its AIR —137.3 ms and 51,449 B with the house options and the WHOLE AIR, which is an upper bound (§469, §470, §471)— and WAS BUILT: the band AIR without the ownership cycle (§477), the producer in the layer and the command's tenth arm, with an envelope that does not publish the balance (§478), and its catalogue, whose positive carries a proof of 49,051 B measured in a single capture (§479; the time of the trimmed AIR is not measured); `ProofFailed` and `VerificationFailed` ARE DECLARED, without a portable proof. **E5 is whole** |

Every measurement in this document was taken on `bb02c71` (§449), in two pure readings that
wrote not one byte to the tree: `PASTE-H4-M` and `PASTE-H4-M2` (see References).

### The boundary with H5b, and what belongs to each

H5b promises completeness: *every committed acknowledgement resolves within a bounded epoch as an
applied transition or a rejection with proof*. This RFC delivers the **rejection with proof**
(E2, E3, E5) and the **empty box as a property of the age distribution** (E4: «nothing older
than T is still in flight»). What it does NOT deliver, and is left for H5b, is the claim about
the acknowledgements: that each one ends up applied or rejected. The empty box speaks of what is
in the pending tree; completeness speaks of what came in through the wire. They are two objects
and two proofs, and mixing them would be the class of «two fronts» the law forbids.

## Motivation

**A rejection is prose today.** The node rejects with `-32000` and a `message` that is the
`Debug` of `LayerError` (`crates/zk-ssl-node/src/main.rs`, the error object is written by hand
and carries no `data`). A rejected client knows WHICH rule the operator invoked because it reads
it; it cannot prove before a third party that that rule really applied to the state the head
commits. Under the law of this house the operator is another adversary, and a rejection that
cannot be verified is a censorship indistinguishable from a rule. The table of properties of
the front page (`README.md`, `README_EN.md`, `doc/USE_CASES.md`) publishes it as *planned* in
its row 7, planned (§444, §445).

**The rules are served without a signature.** `zkssl_params` returns `regulatoryLimit`,
`maxSupply`, `maxAccounts` and `custodianRoot`; at rest live seven parameters (`meta:limit`,
`meta:max_supply`, `meta:max_accounts`, `meta:custodians`, `meta:governance`,
`meta:refund_ttl`, `meta:cust_max`, `crates/zk-ssl/src/persistence.rs`) and none goes under the
signature of the head. `ARQUITECTURA.md` declares the parameters immutable and nothing checks it
from outside; the custodians' cap (`meta:cust_max`) can be raised at rest without anybody
catching it (point 48 of the 5.A queue). Every rejection proof cites a rule —a `T`, a limit, a
cap—, and a rule that is not committed sustains no proof.

**The age of what is in flight exists only in the operator's judge.** Every pending has a
birth (`born`, the height on creation) in the meta tree —leaf `H(sender, born)`, §388— and a
systemic `T` (§178, `DEFAULT_REFUND_TTL`) or a per-payment term (`delta`, RFC-0003).
`RefundTooEarly { born, now, ttl }` is the only judge of age, and it is internal. The meta root
(`root:pmeta`) is stored and checked on opening since §388, but **it is not in the head** (point
43): a third party cannot today affirm anything about how long the money in transit has been
stalled. `doc/USE_CASES.md` already names «the "empty box" proof of what is in flight» as the
piece of the period close; `doc/CADUCIDAD_PENDIENTE.md` designed the clock of the pending. What
is missing is the proof.

**And the three pieces share the same precondition.** None is provable without the head
committing what it does not commit today: the parameters, the meta root and the counters that
bound the universe. That is why the RFC starts with the format (E1) and not with a circuit, and
that is why it goes through an RFC: it touches `spec/RPC.md`, the OpenRPC and the vectors.

## Design

### D-A — The name: proofs over the committed state

That is the name given to every proof whose statement is verified against the fields a signed
head commits, without the node and without the layer: a path, a public arithmetic or a STARK
whose public inputs are roots and counters of the head. The evidence package (`RFC-0004`,
`spec/PAQUETE.md`) is the vehicle, and the `zk-ssl-verify` command the consumer. The five forms
of today (position v1, v2 with co-signatures, extension, consumption, conflict) are proofs over
the committed state; this RFC adds the **rejection** (E3) and the **age** (E4).

### D-B — The v5 head carries ONE family: the state the proofs need

RFC-0006 fixed in its D-3 that a head version carries a single family. Here the family is
«what the rejections and the age need and is not signed today», and they are five pieces, each
with the cause that has no proof without it:

| piece | what it is | without it, this is not proved |
|---|---|---|
| `params_digest` | a single digest of the seven parameters, with its own domain | no cause that cites a rule: `OverRegulatoryLimit`, `WrongRegulatoryLimit`, `RefundTooEarly` (the `T`), `SupplyCapExceeded` (the cap), `AccountLimitReached`, `CustodianSetExhausted` (the quota); and the consistency of parameters between heads |
| `pmeta_root` | the root of the pending meta tree (`root:pmeta`, §388) | the age of anything: `born` lives there |
| `next_pending` | the watermark of the pending tree | the universe of the age proof and `PendingTreeExhausted` |
| `next_index` | the watermark of the accounts tree | `AccountNotFound` (by index) and `AccountLimitReached` |
| `total_supply` | the supply (`meta:supply`) | `SupplyCapExceeded`, and the conservation of money seen from outside (the step point 43 left open) |

**CORRECTION (§247, written by §452).** The `next_index` row calls it «the watermark of the
accounts tree», and it is not: since F3 an account is placed by its identity with linear probing
(`crates/zk-ssl/src/accounts.rs`, `open_with_id`), and `next_index` remains as a CENSUS —the
quota of openings against `max_accounts`—, not as a position. Signing it still pays
`AccountLimitReached`; what it does NOT pay is `AccountNotFound(i)` for `i >= next_index`: the
D-D row that proposes it proves nothing, and E3 will need the ABSENCE of `i` under `accountsRoot`.
The name does not change: it is the layer's, that of `meta:next_index` and that of the KAT of
§451.

The composition follows the mould of v3 and v4 —the wrapping of the previous one, with no domain
tag, because the version byte of the preamble is what separates the compositions (§236)—:

```text
  v5 = merge( epoch_digest_v4(los once),
              merge( merge(params_digest, pmeta_root),
                     merge( as_digest(next_pending),
                            merge(as_digest(next_index), as_digest(total_supply)) ) ) )

  params_digest = merge( as_digest(DOMINIO_PARAMS),
                         merge( merge(as_digest(regulatory_limit), as_digest(max_supply)),
                                merge( merge(as_digest(max_accounts), custodian_set_root),
                                       merge( governance_set_root,
                                              merge(as_digest(refund_ttl),
                                                    as_digest(max_custodian_uses)) ) ) ) )
```

`DOMINIO_PARAMS` is a `u64` of eight ASCII bytes like `DOMINIO_META_PENDIENTE` (`PMETA_V1`) and
its siblings, registered in `zk-ssl-hash` and in its table of domains. The exact form is fixed by
E1 with its KAT: if on measuring a better grouping comes out, E1 declares it and this paragraph
gains its correction; what does not change is the set of the five pieces nor that they go under
the signature.

**Genesis, declared.** The first v5 head composes with the parameters the ledger was opened
with, the root of the empty meta tree, `next_pending = 0`, `next_index` equal to the number of
accounts open when it is emitted and `total_supply` equal to the supply at that moment. There is
no special value: v5 composes what the ledger has at that head, and the empty roots are those of
the empty tree, like `cons_root` in v4.

**What it buys.** (1) The *parameter consistency* of the milestone: with two signed heads of
the same operator, equal `params_digest` say that the rules did not move; different ones, that
they moved, and the only legitimate change —the set of custodians by governance,
`apply_governance_delegated`— leaves an `OpKind::Governance` entry in the log. The witness notes
it as a class of its journal; the command checks it with two heads, as the conflict envelope
checks two roots. (2) The closed universe of the age: `allocate_pending` walks `0..next_pending`
looking for a gap and only raises the mark when there is none (`crates/zk-ssl/src/two_phase.rs`,
corrected in §211), so **every live position is less than `next_pending`**; signing it is
signing the universe. (3) Step 2 of point 43: the supply under the signature, next to the roots
that distribute it.

**What it costs, and where it is paid.** The wire's `firmada()` today demands the consumption
pair with a declared `== 4` literal (point 91): v5 demands the five pieces with the same mould,
and the literal grows into a `match` per version. The witness's `linea_de_diario` holds in
custody twenty-one names by a hand-written list (point 89): it gains five, and the test that
ties it to `VistaFirmada` moves with it. `ParamsDto` gains `governanceRoot`, `refundTtl` and
`maxCustodianUses`, additive. Every typed consumer of this house that carries
`deny_unknown_fields` stops deserializing in the same commit that causes it, which is the price
already declared by RFC-0005 (D-B) and paid in v4.

### D-C — The cause travels as data, not as prose

The JSON-RPC error object admits `data`. Since E2 a rejection by the layer carries:

```text
  "error": { "code": -32000,
             "message": "<sin cambio: el Debug de LayerError, con receptionSeq si lo hay>",
             "data": { "causa": "RefundTooEarly",
                       "campos": { "born": "0x..", "now": "0x..", "ttl": "0x.." },
                       "seq": "0x.." } }
```

`causa` is the name of the variant, as is; `campos` its fields, with the encoding of the wire
(QUANTITY in hex, digests as `Digest`); `seq` the height of the head in whose state it was
judged, which is what ties the rejection to a head the client can ask for signed. `message` does
not change: the witness and the clients read it and do not break. The catalogue —the twenty-five
causes with their fields— is published in `spec/RPC.md` and **a test ties it to the enum**: two
lists are two producers of the same contract, and here the compiler ties them (a new variant
without a row goes red). `receptionSeq` stays where it is and still is not opposable evidence
(§253): the cause is only so when E3 puts the proof on it.

**CORRECTION (§247, written by §454).** The measurement of the tree makes three things of this
design precise. The example, `RefundTooEarly`, is a cause the wire does not serve: only
`apply_refund` and `apply_deissue` produce it, and the node dispatches neither (which causes the
wire emits today is measured by entry §454). Five variants are TUPLES and have no field name in
the code: the catalogue gives it (`index`, `detalle`), and three carry text, not QUANTITY nor
`Digest`. And `ConsumoRepetido` and `ConsumoColision` do not travel in the error object but in
the `result` of `zkssl_publishConsumo` (`accepted: false`): E2 gives them the same `data`.

**CORRECTION (§247, written by §802).** The catalogue goes from twenty-five to twenty-six causes:
`IdentityUnavailable`, with no fields —the identity already has an account, or a recovery retired
it (`SECURITY.md` §3.13)—, which `zkssl_openAccount` and `dev_openSeeded` emit. It has no
portable proof, for two reasons: «has an account» would demand opening the leaf (balance, nonce
and salt), and the retired identities do not go under the head. It stays outside the rejection
envelope and completeness: `zkssl_openAccount` reserves no receipt; the command rejects it by its
name, and the list of the four causes without a portable proof does not change. It is additive in
the sense of E2 —no vector nor the OpenRPC pins the error object—: the version does not go up. It
was drafted in private and is published with the fix (`spec/rfc/PROCESO.md` §«Fallos de
seguridad»). The E2 row and the D-D table are not rewritten.
⚠️ §815: «the list of the four causes without a portable proof» was the kit's, and not this
RFC's: D-I also declares `ProofFailed` and `VerificationFailed`, and the list of four left them
out (it came from the D-D rows marked «declared without proof» and from the correction §460 wrote
after the table, which left `PendingTreeExhausted` that way; not from D-I). Since §815, kit
0.4.3, they are six (`spec/PAQUETE.md` 2.11, verdict 4); `IdentityUnavailable` stays out, for what
this paragraph says.

**CORRECTION (§247, written by §716).** The catalogue goes from twenty-six to twenty-seven
causes: `AutorizacionConsumida`, with the field `compromiso` —an already consumed custodians'
authorisation: the materials of a delegated issuance already applied, resent when the root and
the supply have gone back to those of before—, which `apply_mint_delegated` and
`apply_mint_pending_delegated` emit. Today no method of the wire emits it: the node does not
expose the delegated issuance, except `dev_fund`, which only exists compiled with the `dev`
feature. It has no portable proof: it is sustained by the log entry that carries that
commitment, which the head commits only through `chainDigest`, like the quota of
`CustodianSetExhausted`, and it reveals nothing that is not public, because the commitment
already travels in the entry. It does not enter the D-D table nor the kit's six causes without a
portable proof (`spec/PAQUETE.md` 2.11, verdict 4), which came from the rows of that table
(⚠️ §815, above): without a method that emits it, no rejection envelope and no completeness
carries it, and the command rejects it by its name. If a method of the wire comes to emit it, it
enters the table and that list in the same seal. It is additive in the sense of E2, and the
version of the wire does not go up. It is published with the fix, which was drafted in private
(`spec/rfc/PROCESO.md` §«Fallos de seguridad»): the fix added a row to D-D, and on bringing it to
`main` this correction is written in its place, like that of §802. The E2 row and the D-D table
are not rewritten.

### D-D — The rejection envelope, by paths first

E3 adds to the evidence package the form `tipo: "rechazo"`: the signed v5 head (fields, digest,
signature, `index`), the cause as in D-C, and the material that proves it. The command verifies
the signature, recomposes the head, and judges the cause against what the head commits. The
table fixes which causes enter by paths, what material they carry and **what they reveal**,
because revealing is a cost and it is declared:

| cause | material | what it reveals | stage |
|---|---|---|---|
| `AccountFrozen(i)` | path of `i` under `frozenRoot` (presence) | that `i` is frozen | E3 |
| `AlreadyInThatFreezeState` | presence or absence of `i` under `frozenRoot` | idem | E3 |
| `ConsumoRepetido`, `ConsumoColision` | presence under `consRoot` at the position of the consumption, with the occupant | the consumption, which is already public | E3 (reuses the consumption envelope) |
| `RefundTooEarly` | the meta leaf `(sender, born)` under `pmetaRoot` at the position; `now` = `seq`; `ttl` = `refund_ttl` of `params_digest` (v1) or `delta` of the opening (v2) | sender and birth; in v2, the opening `(c1, f, delta)` | E3 by paths; E5 hides it |
| `RefundUnavailable` | absence of meta at the position, or the sentinel | the position | E3 |
| `PendingMismatch` | the leaf under `pendingRoot` at the position, against the commitment of the materials | a digest | E3 |
| `StaleState` | the head: the root declared by the client is not the committed one | nothing | E3 |
| `WrongRegulatoryLimit`, `OverRegulatoryLimit` | public arithmetic with the limit of `params_digest` | the requested amount, which the client itself requested | E3 |
| `SupplyCapExceeded` | `total_supply` of the head plus the amount, against `max_supply` | the amount | E3 |
| `AccountLimitReached` | `next_index` of the head against `max_accounts` | nothing | E3 |
| `AccountNotFound(i)` | `i >= next_index` of the head (accounts are opened densely from zero; E3 measures it before writing the vector) | nothing | E3 |
| `RecoveryToSameIdentity` | the identity of the leaf of `i` under `accountsRoot`, against the requested one | the `public_id`, which is public | E3 |
| `DuplicateAccountInBatch`, `DuplicatePendingInBatch` | the batch itself | nothing of the state | E3 |
| `PendingTreeExhausted` | `next_pending` equal to the capacity, and that there is no gap: the second half is the range proof of E4 | nothing | E3 marks it; E4 the gap |
| `InsufficientBalance` | band proof: balance < requested under `accountsRoot`, without `available` in the envelope | the account and the upper bound (`pedido - 1`), public by design; the balance and the `leaf_salt` came out literal between §521 and §538, when the proof opened its rows in the clear (RFC-0009 D-B; since §538, E3b-2, no longer) | E5 |
| `ProofFailed`, `VerificationFailed` | the STARK and the public inputs the applicant sent, re-verified | what the applicant already sent | E5 |
| `CustodianSetExhausted` | the quota is derived from the log (§393, §394), which the head commits only through `chainDigest` | — | declared without a portable proof |
| `NotTheIssuer`, `NotTheAccountHolder` | the absent authorisation cannot be exhibited | — | declared without proof |
| `BalanceOutsideBand` | it belongs to the holder's audit path, not to an operation | — | out |
| `Store` | a failure of the operator, not a rule | — | declared as a failure, never as a rejection |

**Correction (§476, E5 cut 3b).** The `AccountNotFound(i)` row of the table above says
`nada` in the column of what it reveals and `E3` in that of the stage, and both have stopped
being true. The rule that is proved is not `i >= next_index`: it is **the EMPTY leaf of account
`i` under the `accountsRoot` that the head of the EXACT `seq` signs**, with its 32-level path
travelling INSIDE the rejection envelope itself and with no new method on the wire (D-G). What it
reveals is that that position is free, and only to whoever made the request. The stage is **E5**
since the correction of §459. The row is not rewritten: this corrects it.

**Correction (§479, E5 cut 4c).** The `InsufficientBalance` row promises the band «without
revealing the balance» and says it reveals «nothing more than the inequality». Of the ENVELOPE,
the first is true since §478: the node writes it without `available` and the command rejects
one that carries it. The second, almost: the envelope reveals the bound `[0, requested - 1]`,
which the refusal itself already implies, and also the `publicId` of the account, which the wire
already serves without a credential (`zkssl_publicId`). Of the REJECTION, the first is not true:
the wire's `-32000` carries `available` since §454, to whoever made the request, and this RFC
does not touch that contract. The band protects against whoever READS THE ENVELOPE, not against
whoever received the rejection; and the operator always knows the balance. The row is not
rewritten: this corrects it.

Every cause of E3 carries a positive vector produced by a bench with a real node and a negative
by mutation of the material, with the rejection text taken from the binary in the run (rule of
`spec/PAQUETE.md`, section 9); the manifest pins exit code and text; the canon runs it as it
runs the other catalogues. The head that goes inside is v5: that is why E3 comes after E1.

**CORRECTION (§247, written by §455).** The measurement of the tree, on opening E3, makes four
things of this table precise. Five of its causes the wire does not serve —`RefundTooEarly`,
`RefundUnavailable`, `PendingMismatch`, `RecoveryToSameIdentity` and `AlreadyInThatFreezeState`—
and proving them serves no client: they leave E3. The head is not always that of the exact
`seq`: what only grows —`nextIndex`, the consumptions, the accounts— is proved with an earlier
or later head according to the measured mutability, and only `StaleState`, `AccountFrozen` and
`SupplyCapExceeded` demand the exact one, which exists if nobody wrote until the next heartbeat.
The frozen and the absence-in-accounts paths are served today by NO method: E3b adds two, and
that touches the wire (additive: `zkssl/0.3` does not move). And a path reveals its neighbours:
the absence path of `i` lets its balances be enumerated and the frozen one its state, so «what
it reveals: nothing» does not hold for `AccountNotFound`. E3 is split into E3a-1 (§455: the four
that are provoked without a receipt), E3a-2 (those that demand a proven receipt) and E3b.

**CORRECTION (§247, written by §459).** The one of §455 says two things that the measurement of
session 125 corrects. The frozen and the absence paths are NOT served with two methods: §458
serves one, `zkssl_frozenPath`, with the holder's credential (§261) —the freeze state is theirs:
`two_phase.rs` checks it after the authority so as not to leak it—, and a sandbox tap,
`dev_freeze`; the surface goes from 26 to 28 methods and `zkssl/0.3` does not move. And an
absence path of `i` does NOT let balances be enumerated: the v7 account leaves carry salt (§117).
What it reveals is OCCUPANCY —each sibling equal to the empty one of its level says that that
subtree has no accounts—, and with `zkssl_publicId`, which asks for no credential, that would
reopen the enumeration of identities that F3 closed. That is why `AccountNotFound` leaves E3 and
goes to E5, as a zero-knowledge proof of an empty leaf (the author's decision, session 125).
`AccountFrozen` is proved since §459 with the leaf under the `frozenRoot` of the head of the
exact `seq`; the depth is fixed by the core (`FROZEN_DEPTH`, `spec/NUCLEO.md`), because
`native_merge` does not separate leaf from node.

**CORRECTION (§247, written by §460).** The measurement of session 126 closes the two rows the
table left in E3. `SupplyCapExceeded` is produced by two paths of the layer
(`apply_mint_delegated` and `apply_mint_pending_delegated`), both before verifying any proof at
all, and through the wire only the sandbox tap, `dev_fund`, reaches it: there is no production
issuance method. It is proved since §460 with the head of the exact `seq` and a material the
table did not name: the `peticion`, the `params` of the rejected issuance exactly as the
applicant sent them. Without it the `wouldBe` would be the node's word, and a node that rejected
an amount that fits would obtain a green proof; with it, `wouldBe` has to be the head's
`totalSupply` plus the requested amount (the author's decision, session 126).
`PendingTreeExhausted` is NOT proved: `allocate_pending` counts the RESERVED positions as
occupied, and the reservations are neither persisted nor go under the signature —their duration
is node policy—, so `next_pending` equal to the capacity is neither a necessary nor a sufficient
condition; only the committed tree being full would be, 2^32 leaves that no bench produces. It is
left declared without a portable proof, like `CustodianSetExhausted` (the author's decision,
session 126). With this E3 is sealed whole.

### D-E — The age proof is over a RANGE, not over n paths

The statement: over the positions `0..next_pending` of the pending and meta trees —whose roots and
whose watermark the v5 head signs—, the age distribution of the live positions, with
`edad = seq - born`. The form of the circuit comes from the structure, measured:

- **Closed universe**: every live position is less than `next_pending` (D-B, §211). The empty
  positions carry the zero leaf in both trees.
- **Rebuilding the root of the range costs what a complete tree over `n` leaves costs**: on the
  order of `2·n` hashes per tree, plus the spine up to the root with the empty-subtree siblings as
  public constants; not `n` paths of thirty-two. For each live position, opening the commitment of
  the pending (`pending_commitment` and, in v2, the reversal envelope) and the meta leaf are a few
  more hashes. With the mould of `circuit_audit` —eight rows per hash, a climb of thirty-two levels
  in 256 rows—, `n = 4096` gives on the order of 2^17 to 2^18 rows. **It is a projection, not a
  measurement**, and that is why E4a exists.
- **One statement, three forms**, as `circuit_audit` covers three audits with two public bounds: the
  **empty box** (`count(edad >= T) == 0`, with a public `T`: nothing older than `T` is still in
  flight), the **cap** (`suma(importe) <= cap` or `count <= cap`, with the public bound) and the
  **concentration** (for a named sender `s`, `suma(importe con sender == s) <= X`). The global
  concentration —the maximum over all senders— requires grouping inside the circuit and is the form
  with the least confidence: E4a measures its cost and E4b builds it only if it fits; if not, it is
  declared and the per-named-sender form is served.
- **Public inputs**: `pendingRoot`, `pmetaRoot`, `next_pending` and `seq` of the v5 head, plus the
  bounds of the statement. The operator's **witness**: for each live position, the opening of the
  commitment (recipient, salt, amount; and `f`, `delta` in v2) and `(sender, born)`; the operator
  has them all (it receives them in `sendMaterials` and keeps them in `pamt:` and `pmeta:`).
- **What it does NOT prove**: nothing about what never entered the tree (a payment the operator did
  not apply has no age: that is H5b), nothing about balances, and nothing about pendings of an epoch
  other than that of the head that signs the roots.

**CORRECTION (§247, written by §464).** The bullet on the public inputs says the operator has the
opening of each live pending because «it receives them in `sendMaterials` and keeps them in `pamt:`
and `pmeta:`», and the tree of `b91109c` refutes it for the opening, not for `(sender, born)`.
`zkssl_sendMaterials` does receive the recipient, the salt and the amount (`spec/RPC.md`, its row),
but the storage at rest (`crates/zk-ssl/src/persistence.rs`) keeps only three things of the pending:
the leaf —the commitment, opaque— in `pend:`, the amount in `pamt:` and `(sender, born)` in
`pmeta:`. No key keeps the recipient or the salt, and in v2 `f` and `Δ` do not reach the node: the
0.x wire does not carry the envelope, and the openings «se generan y se quedan en el cliente» [are
generated and stay on the client] (`spec/rfc/0003-compromiso-v2.md`, Seguridad [Security]). The
operator sees them in transit, so not keeping them belongs to today's code, not a property of the
protocol. Without the opening, an amount is not tied inside the circuit to the committed leaf, and
what the circuit does not constrain is not asserted. That is why E4b-1 (§463) builds the
**per-account** forms —the empty box (`K = 0`), the per-account cap (`T = 0`) and the concentration
of a named sender—, which only need the leaf and `(sender, born)`, and **declares** the per-amount
ones of the bullet «One statement, three forms»: `suma(importe) <= cap` and
`suma(importe con sender == s) <= X` are not proved without the opening. The global concentration
was not built and still has no measured cost.

**E4a, the gate.** Before writing the AIR: the number of hashes as a function of `n` measured over
the real leaves of a ledger of the bench, the projection of rows with the mould, and a proving time
measured on the reference machine for three increasing `n` with the configuration of
`proof_options`. The gate decides: if the proof for the `n` of the reference ledger fits in a
declared time, E4b builds it; if not, this RFC declares the ceiling of `n` for which the proof is
practicable, and the milestone says so with that figure. A ceiling declared with measurement is
valid; a proof that cannot be produced is not.

**CORRECTION (§247, written by §462).** E4a was measured in session 126 and makes four things of
this section precise. **One**: the projection above («`n = 4096` gives on the order of 2^17 to 2^18
rows») counted the hashes of the trees and omitted the age comparison —a bit decomposition per live
position— and the padding to a power of two that the prover demands. **Two**: the «real leaves of a
ledger of the bench» were not needed: the number of hashes of the range depends only on `n`, not on
the leaves nor on the gaps, and the instrument proves it with leaves from the real producers
(`pending_commitment`, `meta_pendiente_hoja`) crossed against the layer's `SparseTree`. **Three**:
the gate now has its figures, fixed by the project's principles before measuring (a decision
delegated by the author, entry §461): the time is a heartbeat, 60 s (`LATIDO_POR_DEFECTO_S`, §121),
and the memory, half the physical RAM, measured in the run.

**Four, what was measured.** The instrument of §461 —the house's Merkle climb with the depth as a
parameter, which wraps `MerkleAir` without copying a constraint from it— proves and verifies the
hashes of both trees with `proof_options`, on the reference machine (i5-1135G7, 8 cores, 12,248,696
kB of RAM, WSL2) and on commit `808a093`:

| `n` | rows | prove | verify | proof | peak |
|---|---|---|---|---|---|
| 1024 | 32,768 | 2.43 s | 1.1 ms | 93,153 B | 250,344 kB |
| 4096 | 131,072 | 10.09 s | 1.3 ms | 111,660 B | 983,532 kB |
| 16384 | 524,288 | 44.94 s | 1.6 ms | 134,975 B | 3,915,228 kB |

The cost per row is almost constant (74, 77 and 86 µs; about 7.5 kB). The verdict, with the rule
above and the reference `n` that this same section wrote before measuring, 4096: the trees cost 10
s, and the projected complete proof —the openings of each live leaf and the comparison, at the same
scale per row— pads to 2^18 rows, on the order of 22 s. **It fits: E4b is built.** The ceiling is
declared now, in two figures that are not mixed: **measured**, the trees fit in a heartbeat and in
half the RAM up to `n` = 32,750 (the size of 2^19 rows); **projected**, the complete proof, up to
`n` ≈ 8192. E4b measures its real AIR and replaces the projection with the measurement; if the
complete proof for 4096 did not fit in a heartbeat, the ceiling is declared then, with that figure.

**CORRECTION (§247, written by §464).** The one of §462 declared the complete proof PROJECTED up to
`n` ≈ 8192 and left it said that E4b would replace it with the measurement: the measurement exists.
The real AIR of §463 —`EdadAir` and its judge in `crates/zk-ssl-air`, the prover in
`crates/stark-experiment/src/circuit_edad.rs`— proves the whole statement, the two trees of the
range wired by a multiset argument in the auxiliary trace and the age comparison, with
`proof_options`, on the same reference machine and on commit `393be18` (entry §463):

| `n` | rows | prove | proof | peak |
|---|---|---|---|---|
| 1024 | 8,192 | 1.53 s | 119,369 B | 198,828 kB |
| 4096 | 32,768 | 7.03 s | 139,153 B | 778,096 kB |
| 16384 | 131,072 | 29.86 s | 162,305 B | 3,107,712 kB |

Verifying, from 1.1 to 1.6 ms; the three proofs verify and climb to the root of the ledger, and the
three fit in a heartbeat and in half the RAM (6,124,348 kB on that machine). **The ceiling becomes
measured: `n` = 16384**, the complete proof in 29.86 s. The projection above (`n` ≈ 8192) is
superseded and is not used; the ceiling of the trees alone (`n` = 32,750) is still that of the
instrument of §461, not of the proof. What follows is projection and is said as such: from 4096 to
16384 the time grew 4.25 times and the peak 3.99 —about 2.06 and 2.00 times per doubling—, so `n` =
32768 would cost on the order of 61 s and 6.2 million kB, just outside both bounds. No ceiling above
16384 is declared without measuring it.

**CORRECTION (§247, written by §717).** The ceiling above, `n` = 16384, was measured without hiding:
§463 predates §538, which switched on the hidden trace in production. The hidden trace has twice the
rows and one more column, and with it the ceiling is another. It was measured in private, with the
production prover and the fix of §717, on a 4-vCPU machine that is not the reference one:

| `n` | `m` | prove (CPU) | peak | against 6,124,348 kB |
|---|---|---|---|---|
| 8192 | 13 | 47.5–48.4 s | 3,130,836 kB | 51.1 % |
| 16384 | 14 | 100.2–101.9 s | 6,255,956 kB | 102.15 % |

The peak of `n` = 16384 goes over the memory bound on any machine with that RAM, without projecting
anything. The time, projected to the reference one with the factor the prover without hiding gives
there (from 0.76 to 0.874, measured in two sessions), is from 36 to 42 s for `n` = 8192 and from 76
to 88 s for `n` = 16384: from 1.27 to 1.46 heartbeats. **With both bounds, the ceiling of the hidden
proof is `n` = 8192.** Since §717 the judge rejects `m` > 13 (`zk_ssl_air::M_MAX`) before reading
the proof, and the prover and the layer's producer do not prove it. The table of §463 is kept: it
measures the proof without hiding.

### D-F — The kit verifies proofs: the AIR enters the closure, the prover does not

Today `zk-ssl-verify` depends on `serde_json`, `xmss`, `zk-ssl-hash` and, for the tests,
`winter-math`; no `winter-air` nor `winter-verifier` lives in the tree outside the `winterfell`
umbrella, which drags in the prover (`crates/zk-ssl-verify/Cargo.toml`, `Cargo.toml`). A rejection
or age proof that only the operator could verify would not be a proof: the final authority is the
proof, and whoever judges it cannot be whoever issues it. That is why E5 —and E4b if its gate opens—
deliver a **verifier-only** AIR crate: the AIRs without `build_trace` or `Prover`, over the four
winter sub-crates loose and pinned, as was already measured outside the tree in the spike of PLAN-22
(`air-solo`: the send verifier runs in ~2.4 ms without the facade).
`zk-ssl-verify` consumes it; `winter-verifier` enters the kit's closure and the prover stays out.
The sentence of `crates/zk-ssl-verify/Cargo.toml` that says «ninguno con prover ni verifier de
winterfell» [none with winterfell's prover or verifier] then gains its §247 correction, it is not
rewritten. What H2 promised —without the repository, without the author, without a network, without
telemetry— still stands: the kit grows, it does not change its nature.

### D-G — `AccountNotFound`: the path of the EMPTY leaf, inside the rejection envelope

The cause is produced over an **INDEX**, not over an identity: `records.get(&index)` returns
`AccountNotFound(index)` and the index is the one the applicant sent. Proving it is, therefore, **a
single empty leaf under the `accountsRoot` that the v5 head signs** — the same form §458 gave to
`AccountFrozen` with the frozen tree—, and not the reconstruction of a probing chain. The linear
probing of F3 (`accounts.rs`: `public_id[0] % cap`, and `+1` while the position is occupied) would
only get in the way of the assertion <<this IDENTITY has no account>>, which is not what the cause
says.

**The expensive part is not proving it: it is serving it.** An absence path over any index and
without a credential is an occupancy oracle, and repeated it undoes what F3 closed. That is why NO
method of the wire is born: **the path travels INSIDE the rejection**, in the same `data` that §454
opened. Only whoever made the request the node rejected receives it, which is exactly whoever
already knew the index.

And what the measurement made visible: **that oracle ALREADY exists** and this RFC does not widen
it. The cause `AccountNotFound(index)` is published on the wire (`spec/RPC.md`, table of causes)
since §454, and saying <<account 7 does not exist>> already reveals the occupancy of 7. What F3
closed was not that question but its SWEEP: with the position derived from the identity over a tree
of depth 32, enumerating stops being walking `0..censo`. Adding the path does not change what can be
asked; it changes whether the answer can be CHECKED.

**Residual trust, in one sentence:** the third party verifies that the position the node names is
empty under the signed head; it does not verify that that position is the one that belonged to its
identity, and that last part depends on the placement, which the operator executes.

### D-H — `InsufficientBalance`: it is MEASURED before writing its AIR

The mould exists and the layer already uses it: `SovereignLayer::audit` proves the band with
`AuditProver::new(self.options)`, that is, with the **house options** (`42, 16, 21, Quadratic`), not
with the circuit's (`32, 8, 0, None`). **The figure is already MEASURED**, and it is measured by an
instrument that lives in the tree: `medicion_470_audit_casa`, sealed in §470. Run PAIRED with
`medicion_130_audit` -the two alternated within each round, three rounds, on `12dd192`- it gives
**137.3 ms and 51,449 B** with the house options, against 28.7 ms and 29,525 B with the circuit's:
**x4.79 in time and x1.74 in proof size**. They were paired on purpose: comparing today's house
against a run of five weeks ago would be comparing two different machines, and this RFC already paid
once for publishing a figure without a source (§469). The bytes came out constant in the three
rounds of both series, which is the check that the proof is deterministic.

And there is a fundamental difference that decides the form: `audit` **requires the spending key**
—only the holder reveals—, whereas the cause is produced by the OPERATOR, who has the leaf and does
not have the key. That is why E5 needs the band **without the ownership cycle**: measured on
`crates/stark-experiment/src/circuit_audit.rs`, that is ONE cycle of 8 rows (`CYC_PK`, with
`CYC_FIN = CYC_PK + 1`) and the FOUR columns of `COL_KEY` (13..17), over a trace of 31 columns whose
pipeline ends at row **287** of 512 —224 rows of slack, 28 cycles, DERIVED from the chain of
constants of the file itself—. It is not half a reform: it is a small trim on a circuit that already
fits.

**The verdict, with the number up front: IT GOES IN, and the AIR is written.** What decides it is
not an expectation but what the layer ALREADY pays for each proof in production, and those are
constants of the code: `metrics.rs` fixes `PUBLICADA_ENVIO_B = 66_998` and
`PUBLICADA_COBRO_B = 65_313`. The band of `InsufficientBalance`, with the production options and
WITHOUT trimming anything, weighs 51,449 B: **23.2 % less than the send** and **21.2 % less than the
claim**. The upper bound is already cheaper than the cheapest proof the layer emits in each payment,
and the trim of the ownership cycle can only

> **Note of §512 (S247).** The two constants this verdict cites are those of its date.
> Fix B (5.A-272) moved them to **66,739 B** the send and **66,692 B** the claim, so
> the percentages above become 22.9 % and 22.9 %. The verdict does not change: the band is still
> cheaper than the cheapest proof of the layer.
take away from there. The TIMES are not compared here: `metrics.rs` publishes no constant in
milliseconds, so a production ms cited would be carried over and not measured.

**And what the measurement uncovered, which the row did not foresee: the rejection envelope HAS NO
PRODUCER.** Censused in the tree by the key `tipo`, separating whoever WRITES it from whoever READS
it: `extension`, `consumo`, `conflicto` and `edad` have a producer in code; `rechazo` has 84
vectors, its manifest and its verifier in the command, and **zero** code that builds one. Its own
manifest declares it: the bench that captured those responses does not live in the tree.
**Correction (§476).** The last two sentences are the measurement of entry 132 and still tell what
there was then; since §473 the rejection envelope DOES have a producer —`--prueba-rechazo`, a mode
of the node's binary— and its bench DOES live in the tree: `tools/banco_rechazo.sh`. The count of
vectors is today's, and §479 takes it to 84. Writing the AIR therefore drags along a piece this
decision did not name, and its form is already proven: `--prueba-edad` is a MODE of the node's
binary that writes the envelope and **exits before starting the server**
(`crates/zk-ssl-node/src/main.rs:713`), driven by a versioned bench. Out of band and by explicit
invocation, those 137.3 ms are paid by no request: nobody can buy them with a cheap rejection.
**REVERSIBLE decision**: the producer of `InsufficientBalance` follows that form, not a path of the
wire.

### D-I — `ProofFailed` and `VerificationFailed`: they are DECLARED, and this withdraws what the row
promised

The E5 row promised <<the re-verification of the applicant's STARK>>. **Measured, there is no
material to re-verify.** The log keeps the `proof_digest` and NOT the proof, with its reason written
in `crates/zk-ssl/src/log.rs`: it would be ~62 KB per operation, and whoever wants one can ask for
it. And `OpKind` only has the eleven APPLIED classes —OpenAccount, Mint, Transfer, Burn, Recovery,
Governance, Freeze, Send, Claim, MintToPending, Migration—: **a REJECTED transition leaves no
entry**, hence there is no committed digest to tie anything to either.

One way out remains and it is DISCARDED: putting the applicant's proof inside the envelope, as §460
put the request for `SupplyCapExceeded`. There it worked because the request is checked against
parameters the head signs. Here it does not: the command would verify some bytes that **the node
itself chose to put there**, and a node that wanted to disguise a rejection would put in any broken
proof. That is not a proof against the operator, and yardstick 4 forbids selling it as such.

**Residual trust, in one sentence:** that a proof failed to verify is today the node's WORD, without
a portable witness; the third party can only check that the transition is not in the log, which is
an absence and not a cause.

⚠️ §815: the kit did not comply with this D-I until 0.4.3. Its list of declarable causes was of
four, without these two, and a reception receipt (RFC-0010) rejected by `VerificationFailed` had no
resolution: the rejection envelope does not prove it and the declared one came out ROJO, «no es de
las que el RFC-0007 dejo sin prueba portable» [is not one of those RFC-0007 left without a portable
proof]. Measured in private before the fix: a third party without an account sends a garbage proof
with the public roots, the node rejects it, gives it its receipt, and once the window is over the
completeness envelope names the honest operator in ROJO. Since §815 they are declared, in the fourth
state of D-G of RFC-0010, and the residual trust above still stands: they are still not proved. What
would close it is the `digest_pi` of RFC-0019 E3, in the 0.5 train.

## What was DISCARDED on measuring

1. **A hash of the verifier in force in the head** (entry 83 of `BACKLOG.md`, F4). The AIR is code,
   not data: the only thing hashable at run time is the `ProofOptions`, and an operator can change
   the AIR leaving them identical (§246, §321, `crates/zk-ssl/src/log.rs`). This RFC signs the
   **data** half of that chain —the parameters— and leaves the AIR half blocked with its reason,
   which depends on entry 55 or on reproducible compilation (H6).
2. **`n` paths for the age.** Proving each live position by its path costs `n·32` hashes and does
   not show that there are no others: the reconstruction of the range costs `2·n` and does show it.
3. **A histogram maintained per transition.** Having each operation update a signed accumulator of
   ages would require touching all the transition circuits and their vectors; one proof per epoch
   over the committed state touches none.
4. **Verifying only in the reference.** Discarded in D-F: a proof that only the operator verifies
   does not change the residual trust.
5. **Changing `message`.** The witness and the clients read it; the cause goes into `data`,
   additive, and `message` keeps saying the same.
6. **One head version per piece.** Five versions for five pieces that are needed at the same time
   would be five silent eras; the law asks for fewer exceptions. One version, one family, and
   RFC-0006 D-3 is respected in its sense: one family per version.
7. **`receptionSeq` as proof of the rejection.** It is not opposable evidence (§253) and does not
   change: the proof is the material of E3, not the counter.

## Compatibility

- **The wire does NOT go up.** The five v5 keys in `zkssl_epochHead` and `zkssl_signedEpochHead`,
  the three new ones of `zkssl_params` and the error's `data` are additive; `zkssl/0.3` continues,
  by the same rule by which it did not go up with `acusesRoot`, `mmrRoot` or `consRoot`. The FORMAT
  version of the signature goes from 4 to 5 and **the recomposer chooses it**: `VersionCabeza` gains
  `V5`, and a consumer that does not know it rejects with its text (§404, §406).
- **The vectors are not rewritten.** The three scenarios `zkssl-0.N.json` and the eleven vectors of
  the wire are still v3 heads; those of the package carry v2 and v3 heads, and those of consumption
  and of conflict, v4: all of them stay. E1 emits its v5 positive and its negatives under its
  version; E3 and E4 are born with their own catalogue.
- **`deny_unknown_fields`, the price.** The typed consumers of this house that read
  `SignedEpochHeadDto`, `EpochHeadDto` or `ParamsDto` stop deserializing the new fields until the
  commit that adds them, which is the same one: the breakage happens inside the seal, as in v4.
- **The error's `data`.** No vector, nor the OpenRPC, nor the witness pins the error object beyond
  `code` and `message`: adding `data` breaks no one that exists today. The OpenRPC is regenerated
  with its generator and the block gates that the diff is exactly the expected one.
- **A ledger from before E1 opens**: v5 is composed of what is already at rest; there is no new key
  in the store, so the migration debt of point 52 does not grow.

### Why it goes through an RFC

`spec/rfc/PROCESO.md` bounds the RFC to what crosses the wire, and this change crosses the wire
three times: the new keys of the head and of the parameters in `spec/RPC.md`, the error's `data`,
and the new vectors. And by rule 4: ACCEPTED requires the spec up to date, the OpenRPC regenerated,
the vectors under their version and the suites green, stage by stage.

> **Note (2026-09-16, §481).** ACCEPTED. Rule 4 of the PROCESO is settled with measurement, and it
> is said with what. (a) The spec, updated: `spec/RPC.md` carries the v5 head (§452), the error's
> `data` with its catalogue tied by test (§454) and `zkssl_frozenPath` (§458); `spec/NUCLEO.md`,
> `epoch_digest_v5` and `params_digest` (§451) and the modules `congelados` (§458) and `cuentas`
> (§475); `spec/PAQUETE.md`, the sixth and the seventh form of the envelope (§455, §465), and each
> cause and each text of the command since §480. (b) The OpenRPC, regenerated with `gen_openrpc` in
> §458: 26 → 28 methods, and that of §454 came out identical; `zkssl/0.3` does not go up. (c) The
> vectors, new: `spec/vectors/rechazo/` (84: §455, §456, §459, §460, §476 and §479),
> `spec/vectors/edad/` (11, §467), the KATs of `epoch_digest_v5` and `params_digest` and the v5 ones
> of the wire and of the package (§451–§453); the three version files of the wire, intact under the
> triple gate. Two things are declared: the wildcard negative of the rejection was rewritten once
> (§459: its cause went from `AccountFrozen` to `InsufficientBalance`) and renamed another time with
> its same bytes (`banda-ausente`, entry §478); and `rechazo-formatVersion-5` is still listed with
> another text since §451. (d) The suites, green: the canon `--sello` of §480 (181 s) runs the five
> families also from inside the tarball —package 69/69, consumption 14/14, conflict 16/16, rejection
> 84/84, age 11/11— and the wire 17/17. What this RFC does not deliver stays in its table:
> `ProofFailed`, `VerificationFailed` and `PendingTreeExhausted`, declared without a portable proof,
> and completeness, which is H5b. The `Store` row of the D-D table goes back to its place: the
> correction of §476 had left it loose, and the row is moved, not its text. Double thread of rule 5:
> this document cites §481 and §481 cites this RFC. Precedents of the turn: 0003 (§356), 0004 (§400)
> and 0006 (§441).

## Security

- **No stage of this RFC touches the spending key**: the proofs of E3 are produced by the operator
  with what it already has, and those of E5 with what it already received. Between §521 and §538 the
  key did travel outside it: the send and claim proofs that E5 re-verifies carried it in the clear,
  because winterfell 0.13 did not hide the witness (§521); since §538 (RFC-0009 E3b-2) they hide it,
  and the key does not come out literal from any of them.
- **A rejection with proof shows that the rule was applied, not that the rule is fair.** What it
  buys is that censorship has to disguise itself as a rule and that that disguise is verifiable.
- **The committed parameters make a change visible; they do not prevent it.** Whoever controls the
  node can still change `meta:limit` at rest; what changes is that the next head says so, and that
  two heads with different `params_digest` and without a governance entry are evidence.
- **The empty box does not speak of what never entered.** A payment the operator did not apply is
  not in the tree and has no age: the age proof does not see it. That is completeness, and it is
  H5b.
- **The completeness of the universe rests on the watermark.** That every live position is less than
  `next_pending` is today a property of the code (`allocate_pending`), not a proven invariant; E1
  signs it and E4a gives it a negative witness: a ledger with a live position above the mark has to
  break the proof.
- **A `PendingTreeExhausted` is not refutable.** The position reservations do not go under the
  signature: with the tree not full, the operator can claim it was exhausted and nobody can prove
  otherwise. With 2^32 positions the claim is implausible, not impossible, and it is declared
  (correction of §460).
- **Each proof of E3 reveals what its row says.** `RefundTooEarly` by paths reveals sender, birth
  and, in v2, the opening; whoever does not want to reveal it waits for E5. Nothing reveals a
  balance.
- **A balance, and from whom it is hidden** (correction of §479 to the previous sentence). No
  REJECTION envelope reveals a balance: that of `InsufficientBalance` carries the band proof in its
  place, and the command rejects the envelope that brings `available`. The WIRE does reveal it: the
  `-32000` of that cause sends it since §454 to whoever made the request. The band protects against
  whoever reads the envelope, not against whoever received the rejection; and the operator knows all
  the balances, which is the model of this era.
- **Correction of §521 to the previous sentence.** The band proof DOES reveal the balance: it opens
  its rows in the clear with the balance and the `leaf_salt` inside, and the repo's vector
  `saldo-insuficiente.json` carries its own, `1000`, in its 42 openings. What is still true is that
  the envelope's `data` does not bring `available` and that the command rejects the one that brings
  it; what was false was that that was enough not to reveal it.
- **Correction of §538 to the previous two.** Since §538 (RFC-0009 E3b-2) the band proof does not
  reveal the balance nor the `leaf_salt`: the prover hides the witness and the E2 suite of RFC-0009
  counts zero literals in its row. The `saldo-insuficiente.json` of 0.4 carries none; that of 0.3,
  kept under `spec/vectors/0.3/rechazo/`, still carries its own. What is still true is the above:
  the envelope's `data` does not bring `available` and the command rejects the one that brings it;
  and what is not promised is said by RFC-0009 (D-A): that nothing is deduced from what is opened.
- **An amount above the ceiling has no band proof.** `MAX_VALOR` of the AIR is 2^62 − 1 and no guard
  bounds the amount before `InsufficientBalance` comes out: with a larger request the cause can be
  produced through the wire and its proof cannot. The producer refuses naming the ceiling (§478) and
  the bench falsifies it live (§479); bounding the amount earlier would move the order of the guards
  of E5, on which the cuts already sealed depend.
- **Declared debts that remain**: out-of-band notice (§21), single node, `--dev`; and the global
  concentration, which may not fit.

## References

- The milestone, verbatim, in line 44 of the form sent (`NLNET-form-answers-EN-v3.txt`, fingerprint
  `26dcde32091e857d`; the block 44..45, `cc71d28bfbf28de3`).
- The two pure readings: `PASTE-H4-M` (`b97fa3671a204fd8`; output `874081d54f78eab9`, 2191 lines)
  and `PASTE-H4-M2` (`ab3c84deb15111db`; output `3609f88610ec6480`, 1444 lines), on `bb02c71`.
- The measured code: `crates/zk-ssl/src/lib.rs` (the `LayerError` enum), `two_phase.rs`
  (`allocate_pending`, `RefundTooEarly`), `persistence.rs` (the seven parameters and the roots at
  rest), `sparse_tree.rs` (depth 32), `crates/zk-ssl-hash/src/lib.rs` (`epoch_digest_v4`,
  `meta_pendiente_hoja`, the domains), `crates/zk-ssl-wire/src/lib.rs` (`ParamsDto`, `VistaFirmada`,
  `firmada`), `crates/zk-ssl-cli/src/witness.rs` (`linea_de_diario`, `recomponer`),
  `crates/zk-ssl-node/src/main.rs` (the error object, the `zkssl_params` and `zkssl_supply` arms),
  `crates/stark-experiment/src/circuit_audit.rs` (the mould), `crates/zk-ssl-verify/Cargo.toml` (the
  closure).
- The entries: §178 (the `T`), §211 (`allocate_pending`), §236 (the version byte), §246 and §321
  (why there is no hash of the verifier), §253 (`receptionSeq`), §275, §292, §414 and §415 (the
  three previous compositions), §379 (conservation), §387 and §388 (the roots of the pending and of
  its meta), §393 and §394 (the quota), §404 and §406 (`VersionCabeza`), §413 (consumption), §441
  (RFC-0006, accepted).
- The documents: `spec/rfc/0004-paquete-de-evidencia.md`, `spec/rfc/0005-nucleo-congelado.md` (D-B:
  what is signed grows only by version), `spec/rfc/0006-consumo-publicado.md` (D-3: one family per
  version; the conflict envelope), `spec/PAQUETE.md`, `spec/NUCLEO.md`,
  `doc/CADUCIDAD_PENDIENTE.md`, `doc/USE_CASES.md`, `SECURITY.md`; in `BACKLOG.md`, entries 54, 55
  and 83; in the 5.A queue, points 43, 48, 89 and 91.
