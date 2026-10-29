# RFC-0013 — The anchor's medium: the heads as C2SP checkpoints, co-signed by third-party witnesses

> **Español:** [`spec/rfc/0013-el-medio-del-ancla.md`](../0013-el-medio-del-ancla.md) is the
> original of this page, in Spanish; it prevails wherever the two differ. This translation is of
> commit `7766efb` (audit entry §917) and was produced by the project's assistant, a Claude Code
> session, in audit entry §918 (`GENAI.md`): it is a generated artefact, not text written by the
> author. Line citations of the form `file:N` refer to the Spanish files.

- **Status:** PROPOSED — the text and the decisions, written over what was measured in mtc-core
  and in the C2SP specifications; E2 built —the medium's tree (E2a, §631) and its signed note
  (E2b, §632)— and E4 —the envelope in the kit (E4a, §633) and its catalogue and its bench (E4b,
  §634)—; of E3, only E3a, and ONLY LOCALLY (§808). It was born DRAFT and enters PROPOSED on being
  integrated (§603): DRAFT has no published face in the `check_publicadas` lock, and RFC-0011 was
  born the same way, proposed with its delegated decisions. Decisions D-A to D-H are DELEGATED and
  REVERSIBLE: the acceptance is the author's.
- **Author:** Ángel José Toranzo Portela
- **GenAI assistance:** Claude (the mtc-core session, which writes the whole draft over the map of
  the pieces it measured before; integrated in the author's name in §603; the same session brings
  the motivation up to date in §703) — see
  [`GENAI.md`](../../../GENAI.md)
- **Date:** 2026-09-30
- **Protocol version affected:** `zkssl/0.4` — **it does not move it, and does not move the head
  either**: what is published is the anchor of RFC-0012, which is already derived from the signed
  head; the medium and its note go outside the wire, as RFC-0012 D-A demands the anchor go.
- **AUDITORIA entry(ies):** §174 (the design of the external anchoring), §248 (a root served by
  the accused proves nothing), §291–§292 (the MMR of heads and its signed partner), §590–§593
  (RFC-0012, the anchor and its envelope), §594 (the diary gate in every state), §599 (the split
  view served by a real node: each witness alone does not see it), §603 (the integration), §631
  (E2a: the medium's tree, and D-G decided), §632 (E2b: the note, contrasted with torchwood), §633
  (E4a: the envelope in the kit, and D-D corrected), §634 (E4b: the catalogue and the bench), §635
  (the threshold judge in the cli), §703 (the motivation up to date: #341, closed by #355), §705
  (§106, amended: the signature of D-C is its declared exception, and the Ed25519 bridge is not
  built), §808 (E3a, locally) and §836 (D-H, decided); and, in mtc-core, its entries §14 and §15
  (the interoperability measured against the IETF's reference implementation, in both directions,
  and that tool's checkpoint rebuilt from its tiles) and §26 and §27 (#355 merged, and the
  reference tool measured afterwards: its checkpoint, now a `tlog-cosignature` co-signature).
- **Backlog:** entry 86 (choose and measure the medium), which this RFC closes in design and leaves
  open in deployment; the open question of entry 83 —«the reference design uses Ed25519 and ZK-SSL
  signs with stateful XMSS (…) it has to be measured whether each witness signs with its own XMSS
  key or whether another primitive is better there. This is measured before designing anything»—,
  which is answered here with measurement; B10.6 in its deployment half; and entry 103 (two
  witnesses on the same disk), which this medium narrows.

## Status of the stages

| stage | what it delivers | breaks the wire? | status |
|---|---|---|---|
| E1 — the medium, decided | this text: what is published (D-A), in which tree (D-B), with which signature (D-C), to which witnesses and how (D-D), who checks it (D-E), at what cadence (D-F), with which pieces (D-G) and what it does not solve (D-H) | no | **this text, proposed (§603)** |
| E2a — the medium's tree | the SHA-256 tree of anchors (`crates/zk-ssl-medio`) and the SHA-256 consistency vectors: the accumulated ones and the large ones of the IETF draft and the 685 probes of `transparency-dev/merkle` | no (additive, outside the wire) | **built (§631)** |
| E2b — the medium's note | the `checkpoint` note with its ML-DSA-44 signature of type `0x06` (`zk_ssl_medio::nota`), and 33 positive and negative notes with the verdict of `filippo.io/torchwood` | no (additive, outside the wire) | **built (§632)** |
| E3 — the publisher | the `add-checkpoint` client of the tlog-witness protocol, the retention of the witnesses' co-signatures and their publication as a co-signed note | no | ⚠️ §808: **E3a built (§808), locally** —the client, the retention and the co-signed note (`zk_ssl_medio::testigos`), measured against the litewitness engine with ML-DSA-44 and against litewitness, on `127.0.0.1`—; pending what asks for real witnesses: choosing them, reaching them with TLS, the cadence and serving the note (it said «pending») |
| E4a — the envelope in the kit | `tipo: "ancla-cofirmada"`: the note, the witnesses' co-signatures —reported, not judged: D-D—, the inclusion of the anchor in the medium's tree and, underneath, the anchor derived from the signed head | no (additive: a new shape of the kit) | **built (§633)** |
| E4b — the catalogue and the bench | `spec/vectors/ancla-cofirmada/` (5 positives and 22 negatives, the well-signed foreign medium among them), its block of the canon, the artefact's eleventh family and `tools/banco_ancla_cofirmada.sh` | no | **built (§634)** |

## Motivation

RFC-0012 left the anchor «correct and with nowhere to live» (D-H): the medium was not chosen, and
`doc/ANCLAJE_EXTERNO.md` §2 says what is demanded of it, «append-only, foreign to the operator,
with an order readable by third parties», and what has to be measured before choosing it (entry
86). And entry 83 left a question that blocked the design: the reference model of witnesses
—Sigsum— co-signs with Ed25519, and ZK-SSL signs with stateful XMSS; does each witness sign with
its own XMSS key, or is another primitive better? «This is measured before designing anything.»

It has been measured, outside this tree. mtc-core —the crate that was born from the `mtc/` here—
implements the co-signature format that the IETF's PLANTS group aligned with the C2SP witness
ecosystem: `tlog-cosignature` defines since 2026 a co-signature type **ML-DSA-44 with timestamp,
type `0x06`**, stateless, whose signed message is exactly the `CosignedMessage` that mtc-core
implements and that it verified byte for byte against the reference implementation in Go: the same
keys from the same seed, the same verdicts in 26 of 26 and 9 of 9 certificates, and that tool's
checkpoint rebuilt from its 2,122 entries up to the same root (mtc-core, AUDIT §14 and §15). Along
the way it turned out that the reference tool itself did not yet write its checkpoint as a
`tlog-cosignature` line, and its main author confirmed it (`ietf-plants-wg/merkle-tree-certs`
#341). The fix was sent by the author of Arqueo (#355), and the main author approved and merged it
on 2026-10-07 (`26db9f2`, the day `draft-07` was published, shortly after its tag), closing #341:
since then the reference tool writes the CA's signature in its checkpoint as a `tlog-cosignature`
ML-DSA-44 co-signature of type `0x06`, with timestamp —the co-signature type this RFC chooses
(D-C)—, and that is how mtc-core reads it against the `demo/` of that commit, and so does
`torchwood` (mtc-core, AUDIT §26; mtc-core's measurement, repeated by the author on his machine in
§27). The ecosystem converges on that format already in its reference tool, and its tooling is
still young.

The answer to entry 83, with that measurement: **in the medium XMSS does not sign, neither the
operator's nor one per witness**. Stateful keys are exactly what that ecosystem avoids in its
witnesses, which are public infrastructure with many logs each; the primitive the ecosystem has
already standardised for the post-quantum case is ML-DSA-44, stateless, and it exists implemented
and contrasted. XMSS stays where it is, signing the head: it is the opposable signature. The medium
is another channel, with another key, and neither of the two inherits the other's state.

And the price of not doing it was just written by §594: the guardian and the diary live on the
same disk, and «two witnesses on the same disk are a single witness against a restoration» (103).
A third-party witness that remembers the last size it co-signed is the second witness that disk
cannot give.

## Design

### D-A — What is published is the anchor of RFC-0012, and the medium is a log of C2SP checkpoints

No new object is born. Every `M` heartbeats (D-F) the publisher takes the signed head, derives its
anchor as RFC-0012 D-A does —anyone can derive it—, and **appends it as a leaf** to the medium's
tree (D-B). The medium is a transparency log in the C2SP sense: a `checkpoint` note
(`tlog-checkpoint`) with three mandatory lines, `origin`, `size` and the root in base64, signed by
the publisher (D-C) and co-signed by third-party witnesses (D-D).

```text
zkssl/v1/<huella_de_clave en hex>          ← origin: la identidad es la clave XMSS del operador
<size>                                      ← cuántas anclas lleva el medio
<raíz SHA-256 del árbol de anclas, base64>
— zkssl/v1/<huella> base64(key_id || timestamp || firma ML-DSA-44)   ← el publicador (D-C)
— <nombre del testigo> base64(…)                                     ← cada testigo (D-D)
```

The `origin` carries the fingerprint of the key of RFC-0012 D-B, not a host name: the medium
belongs to the ledger, not to the machine, and the identity a witness remembers is the same one a
holder verifies in every head.

### D-B — The medium's tree is SHA-256 per RFC 6962, and belongs to the medium, not to the ledger

Generic witnesses check the consistency between two checkpoints with the MTH of RFC 6962 over
SHA-256; none recomposes Rescue Prime, and they are not going to be asked to. The tree of heads
the head already signs (§292, `mmr_hoja`/`mmr_nodo` over Goldilocks) is still **the canonical
history** and is not touched. The medium carries its own tree: leaf `i` is the fingerprint of
anchor `i` (32 bytes, RFC-0012 D-A), the nodes are SHA-256 with the `0x00`/`0x01` prefixes of RFC
6962, and the consistency proofs between old and new `size` are those `tlog-witness` demands.

This answers the RFC-0012 discard «an anchor tree of its own: a second producer of the tree of
heads». **It is not a second producer of the history**: every leaf of the medium carries inside,
signed by XMSS through the head, the pair `(mmrRoot, mmrSize)` of the canonical tree. The medium's
tree only asserts that the **sequence of published anchors grew and was not rewritten**, which is
exactly the property `ANCLAJE_EXTERNO` §2 demands of the medium; and if the two sequences were to
disagree, anyone with the anchors sees it by recomputing both, and the disagreement is evidence
against the publisher, not noise.

### D-C — The note is signed by the publisher with ML-DSA-44, type `0x06`, and that key has no state

`signed-note` has no type for XMSS and witnesses ignore signatures of types they do not know. The
note is signed with the construction `tlog-cosignature` reserves for type `0x06`: the `subtree/v1`
message with the `origin`, the subtree `[0, size)`, the root and a timestamp, and in the line
`key_id || timestamp || firma`, with `key_id = SHA-256(nombre || "\n" || 0x06 || clave)[:4]`. It is
what the IETF's Merkle certificate logs sign in their checkpoints (`mtc-tlog` profile), and what
mtc-core already produces and verifies. The publisher's ML-DSA-44 key **has no index and no
guardian**: it is the first signed channel of this house that a deleted counter cannot compromise.

**Bridge, declared**: as long as today's public witnesses only accept an `origin` with Ed25519
(type `0x01`), the note may also carry an Ed25519 line of the same publisher. Witnesses ignore
what they do not understand, so the two lines coexist. What that bridge is worth and what it costs
goes in Security.

> ⚠️ **Decided by the author in §705: the bridge is not built while §106 holds** (adopted in the
> block acceptance; reversible, like D-A to D-H). The ML-DSA-44 signature of this note is the
> exception §705 declares to §106: it amends, for the anchor's medium, its half «a single family of
> assumptions», without revoking it. An Ed25519 line would clash with the other half, «the whole
> production path is post-quantum», which still has no exception. The paragraph above is left as
> it was and is corrected here; the same holds for (1) of D-H and for the bridge in Security.

> ⚠️ §836: D-H, decided by the author: the bridge is still not built, and there is no `0x01` log
> key.

### D-D — The witnesses are third-party, generic and of threshold; the protocol is `tlog-witness`

The publisher sends each witness `add-checkpoint`: `old <size anterior>`, up to 63 lines of
consistency proof, and the signed note. The witness verifies the `origin`'s signature with the key
it has registered, verifies consistency with **the last size it itself co-signed**, and returns its
co-signature. If the old size is not its own, it answers `409` with the size it remembers: **that
is the detection of entry 103**, the restored directory presents a smaller size and the witness
says so. Which witnesses count and how many is decided by **the client**, not the operator: the
kit already has that policy for the XMSS co-signatures (`--testigos`, `--k`, S319) and reuses it.
ZK-SSL's own witness keeps existing and co-signing heads with XMSS; this channel does not replace
it, it surrounds it with third parties that are not of the house.

> ⚠️ **Corrected (§633).** «The kit already has that policy (`--testigos`, `--k`, S319)» was
> false: the policy lives in `zk-ssl-cli witness --verificar-cofirmas`, and the kit accepts a
> single argument and, with the XMSS co-signatures of the v2 package, **reports and does not
> judge** (`spec/PAQUETE.md` §1). The sentence is left as it was and is corrected here.

**Decided in §633, by the author: the kit reports, it does not judge.** The `ancla-cofirmada`
envelope lists the witnesses whose co-signature verifies with the vkeys the envelope brings, each
with the SHA-256 fingerprint of its WHOLE key —the four-byte `key_id` is an identifier and can be
fabricated—, and the threshold is applied by the client with its policy, as with the XMSS
co-signatures. So the kit keeps a single argument and the conformance harness does not change. The
other two options —`--testigos`/`--k` in the kit, or both things— changed the kit's published
contract and are discarded for now. Judging the threshold of the medium's co-signatures in
`zk-ssl-cli`, as S319 judges the XMSS ones, is left unbuilt.

**Built in §635: the client's judge.** `zk-ssl-cli witness --ancla-cofirmada SOBRE
--testigos-medio VKEYS [--k-medio K]` reads the client's policy —the vkeys of the medium's
witnesses it trusts, from outside the envelope, whose vkeys it ignores—, re-verifies the note with
its publisher and counts the NAMED and distinct witnesses whose co-signature verifies over its
checkpoint; below `k` it does not accredit, and `k = 0` is rejected. It accredits the CHECKPOINT:
the head, its anchor and the tie to the medium are the kit's, over the same file.

**Built in §808, ONLY LOCALLY: the publisher's side (E3a).** `zk_ssl_medio::testigos`, with the
`firmar` feature, over `tlog-witness` v1.1.0 (C2SP `b2dd989`): it assembles `add-checkpoint`
(`old N`, up to 63 proof lines, the publisher's note ALONE); it reads the `200` by appending it to
the note, as the specification allows, and ignores the key lines it does not know; it retries ONCE
from the size of a `409`, and if the witness remembers MORE anchors than the note, it stops and
says so: it is the detection of entry 103 above, and it is not dodged. It retains the `0x06`
co-signature of each witness, one per witness and verified again over its checkpoint, and writes
the co-signed note —the publisher's and those lines behind it— in the `checkpoint` file of a
directory, by atomic rename and without going backwards. Its transport only speaks HTTP to loopback
addresses (`127.0.0.0/8` and `::1`). Measured against two implementations of another hand,
locally: the `add-checkpoint` of litewitness (`filippo.io/torchwood` v0.10.0, `internal/witness`)
with an ML-DSA-44 key, and litewitness v0.10.0 untouched; the note co-signed by two of those
witnesses enters the catalogue's envelope, the kit gives it VERDE and the judge of §635 accredits it
with `k = 2`. What asks for real witnesses —which ones, with TLS, at what cadence and where the
note is served— is left unbuilt.

> ⚠️ §836: in `k` only the `0x06` co-signatures count; the `0x04` ones are neither verified nor
> counted (D-H, decided by the author).

### D-E — What a third party checks, without the node

The `tipo: "ancla-cofirmada"` envelope (E4) verifies, in this order: the note (three lines, the
publisher's signature of type `0x06`); the witnesses' co-signatures against the threshold policy;
the inclusion of the anchor's fingerprint in the note's root (SHA-256 path); and, underneath, the
RFC-0012 anchor envelope as it is: that the anchor IS the signed head it claims to be. A VERDE
says: «this head was published, at this position, when `k` third-party witnesses saw it». What it
does not say is said by D-H.

### D-F — The cadence is the `M` of RFC-0012, and the node waits for nobody

One leaf per anchor, one anchor every `M` heartbeats, `M` = 1,440 as RFC-0012 D-F proposes until
there is a real operator. Publication is **asynchronous**: the epoch closes and the head is signed
without waiting for co-signatures; what a fallen witness costs is liveness of the medium, not of
the ledger, which is price (b) of entry 83 and is accepted as it is accepted there.

### D-G — The pieces exist, measured, and the decision is where they live

- The SHA-256 tree of RFC 6962 with inclusion and consistency: `mtc-core::{subtree, log}`, which
  passes the 65,058 cases of the accumulated vectors of the IETF draft, its large vectors up to
  `2^64-1` leaves, and rebuilt from tiles the 2,122-entry log of the reference implementation up
  to its same root.
- The `subtree/v1` message, the `key_id` and the ML-DSA-44 signature: `mtc-core::cosign`,
  contrasted byte for byte with the implementation in Go (the same keys from the same seed).
- Strict PEM and base64 without dependencies: `mtc-core::pem`.
- The note verifier: mtc-core's `interop checkpoint` already reads a note, rebuilds the root and
  checks the publisher's line in the two forms that exist today.

**Delegated decision**: depend on mtc-core by pinned commit, as `hbs-state` is depended on, or
copy the three modules with their provenance. The first avoids two implementations of the same
problem; the second avoids a dependency on an unaudited crate. Either of the two is valid for E2;
neither changes the wire.

> ⚠️ **Corrected (§631).** «As `hbs-state` is depended on» was false: no `Cargo.toml` nor the
> `Cargo.lock` of Arqueo name `hbs-state`; the one that depends on it is mtc-core. There was no
> precedent of a dependency by git to follow. The sentence is left as it was and is corrected
> here.

**Decided in §631, by delegation: copy.** `src/hash.rs` and `src/subtree.rs` of mtc-core
`d3b0ca6` live in `crates/zk-ssl-medio`, byte for byte except for the lines marked
`ADAPTADO (§631)`, with their provenance in the crate's header and in `NOTICE`. For three measured
reasons: mtc-core depends without `optional` on `hbs-state` by git and, by default, on `ml-dsa`,
and brings the whole X.509/MTC format, when the tree needs two files and `sha2`, which was already
in the `Cargo.lock`; the E4 kit must be rebuildable from this repository and crates.io; and the
premise of the precedent was false. `log.rs` is not copied: it is a log of MTC entries, and the
medium uses the literal recursion, O(n) per query, which E3 will measure if it falls short. The
price, declared: two copies that may diverge, tied by the same IETF vectors, which pass in both
repositories. `cosign.rs` and `pem.rs` are decided in E2b, with the same rule.

**Decided in §632.** Of `pem.rs` the base64 is copied (`src/base64.rs`), without the PEM part and
without skipping whitespace, which in a note is an error. `cosign.rs` is **not** copied: it
assembles its message with the MTC OID identifiers (`TrustAnchorId`) and drags in `hbs-state`, and
the medium signs with `signed-note` names. `zk_ssl_medio::nota` was written here from the
specifications, and what ties it is not mtc-core but an implementation of another hand:
`filippo.io/torchwood` v0.10.0, by the author of the C2SP specifications, over the `crypto/mldsa`
of Go 1.27. The publisher's line that torchwood signs comes out here byte for byte, and of 33
notes there is none that torchwood rejects and the medium accepts; the 4 that torchwood accepts
and the medium does not are three stricter rules, declared in `src/nota.rs`: the `origin` is the
publisher's name (D-A), a single publisher line, and canonical base64 in the signatures.

### D-H — The residue, declared

Those of RFC-0012 D-H remain: the medium is trust **displaced, not eliminated** —now to a
threshold of third-party witnesses, which is better than to a single medium and worse than to
nothing—; the tail between anchors is bounded by `M`, not closed; and the anchor proves which
history was canonical, not that it was complete. And its own: (1) there is today no public witness
that accepts an `origin` of type `0x06`; until there is, E3 is measured with the Ed25519 bridge
against a real public witness and with `0x06` against mtc-core's verifier, and it is said which of
the two was measured; (2) `ml-dsa` is not audited, its own crate says so —since §632 it enters
pinned with `=`, and what bounds it is the byte-for-byte contrast with Go's `crypto/mldsa`, not an
audit—; (3) the publisher's ML-DSA-44 key is one more key to keep in custody, stateless but with
custody.

> ⚠️ **Corrected (§705).** Without the bridge of D-C, E3 is not measured against a public witness
> while none accepts `0x06`: it is measured with `0x06` against mtc-core's verifier, and against
> public witnesses when there are some.

> ⚠️ §808: **what was measured locally in E3a makes (1) more precise, and does not close it.** The
> reference witness software already accepts this `origin`: litewitness v0.10.0 reads the key of a
> log of type `0x06` (`NewLogVerifier`, and the notes of its version: «Logs can now sign
> checkpoints with ML-DSA-44 keys») and co-signed the medium's notes on `127.0.0.1`. But **it
> co-signs with Ed25519**: its key comes from an SSH agent and only accepts Ed25519, although
> `tlog-witness` v1.1.0 says witnesses SHOULD use ML-DSA-44. Such a co-signature is not verified by
> the publisher —the house does not link curves: §106, §705 and `tools/check_curvas.py`—, which
> ignores it as the line of a key it does not know, and in the kit's envelope it would be an
> unjudged line. (⚠️ §835: it said the house does not link curves; the cli and the SDK link `ring`
> through `rustls`, for transport, and entry 122 of `BACKLOG.md` opens it; no co-signature, note
> or head is verified with curves. ⚠️ §838: it said entry 122 opens it; the author decided it: §106
> is amended for that transport, as a declared exception in the manner of §705, and
> `tools/check_curvas.py` bounds it by edges and looks for its use in the house's sources. The
> exception covers no co-signature, note or head.) What the house knows how to retain and the kit
> knows how to verify is the `0x06` co-signature, that of its same engine (`internal/witness`)
> with an ML-DSA-44 key, which the litewitness binary does not offer. Which public witness runs
> that version, and with which key, has not been measured: it asks for the network. And E3a was
> not measured against mtc-core but against those two witnesses, which are witnesses and not note
> verifiers.

> ⚠️ §836: **decided by the author on 2026-10-15: only `0x06`, at both ends** (reversible, like
> D-A to D-H). The publisher signs the note only with ML-DSA-44. The house only retains, verifies
> and counts ML-DSA-44 co-signatures (`0x06`). The `0x01` bridge of D-C is still not built (§705,
> point 5). Ed25519 `0x04` co-signatures are neither verified nor counted in `k`. Why: an Ed25519
> co-signature that counted in the threshold would put on a curve the whole property of the
> medium, not only its admission; and §106 and §705, point 4, have no exception for classical
> signatures. ⚠️ §838: since §838, §106 has one, that of the TLS transport of the cli and of the
> SDK, which verifies ECDSA and Ed25519 signatures of certificates and of the handshake; it covers
> no co-signatures, notes or heads, and this decision does not change.
>
> Enrollment in any witness network is not requested yet. Three things are needed first: the whole
> publisher (the loop, the cadence, serving the note and the transport, which waits for entry 122
> of `BACKLOG.md`. ⚠️ §838: it said it waits for entry 122; entry 122 decided the transport of the
> cli and of the SDK, not the publisher's, which is still unbuilt: `zk-ssl-medio` does not link
> TLS, and if it needs it, the by-edges rule of `tools/check_curvas.py` is widened in a seal of its
> own); witnesses that accept a `0x06` log, through a network's list or configured by hand; and at
> least two witnesses with an ML-DSA-44 key. They are necessary, not sufficient: the enrollment is
> the author's to give. Until the enrollment, E3 is measured locally: against litewitness, as in
> §808, and against `transparency-dev/witness` (`omniwitness` with an ML-DSA-44 key), which is
> read and not run. ⚠️ §848: it said «which is read and not run». Run on 2026-10-21, on
> `127.0.0.1`: `omniwitness` of `5397155`, untouched, with the medium's `0x06` log configured by
> hand (`--additional_logs`), one with an ML-DSA-44 key and another with an ML-DSA-44 and an
> Ed25519 of the same name. The client of §808, unchanged, says in ten steps what `tlog-witness`
> asks for (`b2dd989`; its text and that of `tlog-cosignature` did not change up to C2SP's `main`
> of 2026-10-21, `e5746a0`): the `404`, the `409` with its retry, the `409` with a size larger than
> the note's, the `422`s and the `403`. It is recorded and stated by
> `crates/zk-ssl-medio/tests/vectores/testigos/README.md`. It is software of another hand, locally:
> it does not say which public witness runs it or with which key, and it does not bring the
> enrollment forward.
>
> Another route is left written, undecided: witnesses of a network that configure the `0x06` log
> by hand and co-sign with `0x04`. The house would neither verify nor count those lines. Serving
> them unjudged in the note, so that a third party checks them with its own tools, would be
> decided by the author later on.
>
> Read on 2026-10-15 outside this tree, without running it:
>
> - `https://github.com/transparency-dev/witness-network`, at `a79e654`: its `participate` page
>   (in `site/content/`) asks the log for a `0x01` key (lines 36-38) and the witness for a `0x04`
>   one (lines 151-153), and says there is no production list (line 73). Its list format,
>   `log-list-format`, admits ML-DSA-44 (lines 46-49). But its validator (`cmd/validate`) reads the
>   keys with `golang.org/x/mod/sumdb/note` v0.28.0, which only knows Ed25519 (`note.go:259-264`):
>   by the code, it would reject a `0x06` key.
> - `https://github.com/transparency-dev/witness`, at `5397155`: it accepts `0x06` logs
>   (`config/public_witness_network.go:99`, over `transparency-dev/formats` `340a9e5`; and by hand,
>   `omniwitness/configs.go:61`), and its
>   `omniwitness` co-signs with an Ed25519 or ML-DSA-44 key, or with several
>   (`cmd/omniwitness/monolith.go:46` and `:194-224`; in `formats` `340a9e5`,
>   `note/note_cosigv1.go:218-232`). That the kit verifies those co-signatures has not been
>   measured.
>   ⚠️ §848: it said it has not been measured. Measured locally on 2026-10-21: it verifies them.
>   The note co-signed by two `omniwitness` with ML-DSA-44 gives VERDE in the kit, the cli's judge
>   ACCREDITS it with `k = 2` and not with `k = 3`, and the Ed25519 line of the one with two keys is
>   ignored. And the ML-DSA-44 `vkey` that Go's ML-DSA used by `formats` (`filippo.io/mldsa`
>   v1.0.0) derives from a seed is the one `ml-dsa` derives from the same seed.
> - `https://github.com/FiloSottile/torchwood`, at `1cadf59`: litewitness only co-signs with
>   Ed25519 (`cmd/litewitness/litewitness.go:294`).
>
> (1) above is left as it was.

## What was DISCARDED on measuring

- **A note type for XMSS.** `signed-note` does not define it, `0x03` is reserved, and a
  third-party witness ignores what it does not know: the note would be valid only for witnesses
  of the house, which is what entry 103 says is not enough.
- **One XMSS key per witness.** The question of entry 83, answered: it is the state the witness
  ecosystem avoids, and it already has a standardised stateless primitive for the post-quantum
  case.
- **Publishing the MMR of heads as the medium's tree.** Its nodes are Rescue Prime; no generic
  witness recomposes them, and asking for it is asking for a witness of the house.
- **Waiting for a post-quantum log signature type other than `0x06`.** None is proposed, and
  `0x06` is already used by the Merkle certificate logs to sign their own checkpoints.
- **A wire method for the co-signed note.** §248 again for the publisher's note; the witnesses'
  co-signatures, on the other hand, do prove something when served, because the accused did not
  make them. They are served as a checkpoint file in the `tlog-checkpoint` convention, outside the
  wire, and the kit reads them from there.

## Compatibility

It does not move `zkssl/0.4`: no methods, no fields, no bytes under the signature. Everything new
lives outside the wire —the medium's tree, the note, the publisher— and in the kit, as a new family
with its `tipo`, its vectors and its bench, which is the route `NUCLEO.md` §3 reserves and RFC-0012
already used. The new vectors are of three classes: positive and negative notes (bad signature,
different `origin`, root that does not add up, co-signature of an unnamed witness), SHA-256
consistency (those of the IETF draft serve as they are, and are already copied in mtc-core with
their license; since §631, also in `crates/zk-ssl-medio`, with the 685 probes of
`transparency-dev/merkle`), and `0x06` co-signatures (mtc-core's corpus, contrasted with Go; since
§632, the 33 notes of `crates/zk-ssl-medio/tests/vectores/notas/`, signed and judged by torchwood).

### Why it enters through an RFC

Because it chooses the medium, which `ANCLAJE_EXTERNO` §2 left unchosen on purpose; because it
introduces a second signature primitive into the house, and that is a design decision and not a
code one; and because it answers a question the BACKLOG had marked as «before designing anything».

## Security

- **The API principle is preserved**: the spend key does not travel. The note carries a root, two
  numbers and signatures of public keys; no private key of anything comes near.
- **§106, amended in §705.** The publisher's ML-DSA-44 signature is the declared exception to the
  «single family of assumptions» of §106: the kit's verdict on the `ancla-cofirmada` envelope
  rests on the weaker of two families, the hash's resistance and lattices, and besides on
  `ml-dsa =0.1.1`, unaudited (D-H). Conservation, the heads and their XMSS signature do not depend
  on it.
- **The opposable signature is still XMSS.** The ML-DSA-44 key only signs notes of the medium. A
  stolen note key allows making witnesses co-sign a false sequence of anchors; **it does not allow
  fabricating heads**, and an anchor that does not correspond to a head signed by XMSS is
  detectable by anyone who has both. The damage is confusion and denial, not forgery.
- **The Ed25519 bridge is not post-quantum**, and it is said: while it is there, admission into
  today's witnesses rests on a classical key. What a quantum adversary gains with it is that of
  the previous point, no more. It is withdrawn when there are witnesses with `0x06`.
  ⚠️ **Decided (§705):** it is not built while §106 holds (D-C), so this point describes a bridge
  that does not exist.
- **The property is of threshold.** An adversary that controls more than `k` witnesses breaks it;
  `k` and the names are set by the client, and entry 83 has it declared.
- **What this medium closes of entry 103**: a restored directory presents a `size` smaller than
  the one the witnesses remember, and the witnesses answer with theirs. What it does not close: an
  operator who besides controls all the witnesses, and the absent diary without a published
  anchor.

## References

- `spec/rfc/0012-el-ancla-de-cabezas.md` (D-A, D-B, D-D, D-F, D-H and its discard of the tree of
  its own); `doc/ANCLAJE_EXTERNO.md` §2 and §7; `doc/CONFIANZA_RESIDUAL.md`, B10.6 and B10.7.
- `BACKLOG.md`: entry 83 (the chain of residual trust and its open question), entry 86 (choose and
  measure the medium), entry 103 (two witnesses on the same disk), entries 87 and 108 (the
  versioned scheme and the short signature; ML-DSA as a candidate).
- C2SP: `signed-note` (types `0x01`, `0x04`, `0x06`), `tlog-checkpoint`, `tlog-cosignature` (the
  `subtree/v1` message and the ML-DSA-44 `key_id`), `tlog-witness` (`add-checkpoint`, `409` with
  the remembered size), `mtc-tlog` (the checkpoint signed by the CA's cosigner as a note).
- mtc-core (`github.com/atoranzo/mtc-core`): `src/subtree.rs`, `src/log.rs`, `src/cosign.rs`,
  `src/pem.rs`, `examples/interop.rs`; its audit log, §9 (start-up consults the diary in every
  state, read in this node), §14 and §15 (the interoperability, measured twice).
- `transparency-dev/merkle` (`fbbcd74`): `testdata/`, the 685 probes E2a copies, and
  `proof.VerifyConsistency`, with which the witness of `transparency-dev/witness` checks
  consistency (`witness/witness.go`, `b4c9458`).
- `filippo.io/torchwood` v0.10.0 (`cosignature.go`, `checkpoint.go`) and `golang.org/x/mod/sumdb`
  v0.40.0 (`note`, `tlog`), over the `crypto/mldsa` of Go 1.27: the contrast of E2b. And its
  `internal/witness`, `cmd/litewitness` and `cmd/witnessctl` (git `2fa53be`), the local witnesses
  of E3a (§808), with `tlog-witness` v1.1.0 (C2SP `b2dd989`).
- `ietf-plants-wg/merkle-tree-certs`, issue #341: the reference tool's checkpoint was not yet a
  `tlog-cosignature` line; confirmed by its author and closed by pull request #355 of the author
  of Arqueo (`bfb754f`), merged as `26db9f2` on 2026-10-07.
