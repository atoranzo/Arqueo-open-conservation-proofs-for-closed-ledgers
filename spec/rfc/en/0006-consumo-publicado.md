# RFC-0006 — The published consumption: single use within a ledger, detection across ledgers

> **Español:** [`spec/rfc/0006-consumo-publicado.md`](../0006-consumo-publicado.md) is the original
> of this page, in Spanish; it prevails wherever the two differ. This translation is of commit
> `7766efb` (audit entry §917) and was produced by the project's assistant, a Claude Code session,
> in audit entry §918 (`GENAI.md`): it is a generated artefact, not text written by the author. Line
> citations of the form `file:N` refer to the Spanish files.

- **Status:** ACCEPTED — **rule 4 of the PROCESO, settled with measurement**
  (§412 the adoption; E1 §413, E2 §414–§415, E3 §416–§422, E4 §427–§431, §433–§437 and
  §439–§440-B, the stages with their gates; §441 this turn). It is preserved as a record of what
  was decided, what was measured and what was discarded.
- **Author:** Ángel José Toranzo Portela
- **GenAI assistance:** Claude (session 99) — see [`GENAI.md`](../../../GENAI.md)
- **Date:** 2026-09-06
- **Affected protocol version:** `zkssl/0.3` — **goes up to `zkssl/0.4` in E2** (see Compatibility)
- **AUDITORIA entries:** §32, §36, §55, §117, §236, §275, §292, §387, §391, §392, §405, §412, which seals it, §413 (E1), §414–§415 (E2), §416–§422 (E3), §427–§431, §433–§437 and §439–§440-B (E4), and §441 (this turn to ACCEPTED)

## Status of the stages

| stage | what it delivers | does it break the wire? | status |
|---|---|---|---|
| E1 — the tree and the root at rest | the set of consumptions as a sparse tree of the layer, its root `root:cons` stored and checked on opening (the sixth root at rest), the rejection of a repeated consumption, the snapshot that transports it, and the negative witnesses that falsify it | NO | sealed — §413 |
| E2 — the v4 head | the composition `epoch_digest_v4` that puts the root and the count of consumptions under the signature; the accepted set becomes {2, 3, 4}; the wire serves the two fields; new vectors under their version | **YES** (`zkssl/0.3` → `0.4`) | sealed — §414 (E2a) and §415 (E2b; the wire does NOT go up: see the correction) |
| E3 — the portable proof | a new form of the evidence package that proves that a consumption is under the root of a signed head and was not under that of an earlier head; the command verifies it without a node | NO | **sealed** — §416 and §417 (the path, through the wire), §418 and §419 (the envelope and its verification without a node), §420 and §421 (the bench that captures them) and §422 (its separate catalogue). WHOLE since §422 |
| E4 — the two-ledger bench | two nodes, two operators, the same consumption published in both, and a reader that with the two signed heads detects it; and the negative: with a single head there is nothing to detect | NO | **sealed** — E4a (detection) in §427-§431; E4b (blocking at processing) in §433-§437 and §439-§440; E4c (authoritative registry) DISCARDED. The partition and what remains, under the table |
| E5 — the tie in circuit | that the consumption be constrained in the AIR to the operation that consumes it, with `nullifier_tree.rs` as the starting piece. Outside this RFC: today the witness that would falsify it cannot be written without E1 | NO | out of scope |

Every measurement in this document was taken on `2298e61` (§411), in two pure readings that
wrote not one byte to the tree: `PASTE-NULL-M` and `PASTE-412-PRE` (see References).

### The partition of E4, and what is built of each part

The ruling of session 100 split E4 in three, and until §437 that partition did not live in this
document: it was cited by its sentence and not by its name. It is written here.

- **E4a — DETECTION. SEALED.** A third party downloads the tarball and, without the repository
  and without nodes, verifies that the same consumption is under the `consRoot` of two heads
  signed by distinct operators. §427-§429 (the bench that produces the material), §430 (the
  reader, which is the COMMAND) and §431 (its catalogue, run by the same harness in every canon).
- **E4b — BLOCKING AT PROCESSING. E4b-1 and E4b-2 sealed.** A node that holds in custody the
  signed heads of other ledgers REFUSES TO PROCESS a consumption that another of them signed as
  having. §433 (the single producer of the consumption root), §434 and §434-B (from the wire to
  the head), §435 (`arbol_de_consumos` published) and §436 and §436-B (the gate in the node).
  **The list of consumptions of another ledger is not believed**: it is rebuilt and its root is
  demanded equal to the `consRoot` that that ledger's signature attests, so whoever emits the
  file does not have to be trustworthy. And **E4b-2**, sealed: the BOUNDARY that gate already
  imposes is written (correction §439) and its two witnesses —the front face and the pair— live
  in `main.rs` since §440.
- **E4c — the AUTHORITATIVE REGISTRY. It is NOT built, and it is a decision, not a debt** (D-4).
  It would always prevent, and in exchange the system would have one more operator, with its
  single point of failure and of censorship. If a body takes it on, it is ITS operator and not
  ours.

⚠️ **Across ledgers it is still DETECTION, never prevention, also with E4b built.** The gate of
§436 blocks PROCESSING here with the evidence this node has: it does not stop the consumption from
being published in the other ledger and **it does not prove double use** —it proves that another
ledger signed as having it—. What E4b adds is not a new guarantee: it is that detection can ACT
where the operator already commands.

## Motivation

`doc/USE_CASES.md:28` publishes, in the table of properties, the row «No double use — a unit is
consumed once (nullifiers) — measured». What is measured is something else: that the two-phase
production path uses no nullifier at all (`SECURITY.md:379`) and that the layer persists none
(RFC-0005, D-C). The row is true for the double spend of an account —the total order of the log
closes it— and false in its parenthesis. The use case the same page describes in `:62-69` —the
double certification of one same expense under two programmes— is covered today by no piece of
the tree: a third party cannot know, reading two signed heads, whether the same unit was consumed
in both.

What the tree has today, measured (`PASTE-NULL-M`):

- **Two live nullifiers and one retired, none of which serves.** The spending one of the
  retired path (§32): `derive_nullifier(sk, nonce) = H(H(DOMINIO, sk), nonce)` in
  `crates/zk-core/src/spend_authority.rs:86`, ported to STARK in
  `crates/stark-experiment/src/nullifier_tree.rs` (793 lines: non-membership and insertion in
  lockstep over `dual_climb`, with the position collision declared as denial of service in its
  header); `settlement-layer` and `circuit_settlement` consume it, and **no file of
  `crates/zk-ssl/src`** except the cleanup of the legacy. The custodians' threshold one
  (`circuit_threshold_single_nullifier.rs:243`, `{identity_domain, custodian_set_root, nullifier,
  operation}`): single use of an authorisation, and that one the layer does use. Both are derived
  from a private key **so that nobody can precompute them** (§117): they are the opposite of a
  label that two bodies have to be able to compute separately.
- **`root:nullifier` and the prefix `null:` are occupied by the migration.** `load`
  (`crates/zk-ssl/src/persistence.rs:456-680`) reads them, verifies them against the declared
  root and **deletes them** (§36); the v3 snapshot transports them and the v4 verifies them on
  import. A new root with that name would not survive the first opening.
- **Five roots at rest, all fail-closed.** `root:state`, `root:pending` (§387), `root:pmeta`
  (§388), `root:froz` (§391) and `root:log` (§392), read with `need()` in `load`
  (`persistence.rs:484-562`) and written in `commit` (`:774-783`). The mould of the sixth is
  written five times.
- **The cost of a signed field is priced and unpaid.** `epoch_digest_v3 = merge(v2,
  merge(cima, as_digest(t)))` (`crates/zk-ssl-hash/src/lib.rs:257`), with no domain, separated
  only by the version byte of the preamble (§236, `zk-ssl-verify/src/lib.rs:329`). The accepted
  set {2, 3} has a single producer since E2 of RFC-0005 (`VersionCabeza`,
  `lib.rs:142`, `TODAS` `:151`) and the test that enumerates it pins `[2, 3]` and the text «v2 o
  v3»: **the falsifier of E2 already exists**. RFC-0005 (D-B, «Familias nuevas» [New families])
  says where a family enters under the signature and what has to be measured first: this document
  is that measurement.
- **The word is worn out.** `nullifier` appears 200 times in 67 documents, `nullificador` 39
  and `nulificador` 43; with three spellings and three meanings (spending, threshold, legacy).
  Point 25 of the queue of session 80 left it open and this reading bounds it: the pattern «tiene
  / hay un árbol de nulificadores» [has / there is a nullifier tree] gives zero outside
  `AUDITORIA.md`, and the rows in the present tense that remain facing the public
  —`INSTITUCIONAL.md:271`, `INSTITUTIONAL.md:282`, `VISION.md:127`, `doc/USE_CASES.md:28`— are
  cited here and are corrected when a seal touches each file.

And what is missing, also measured: there is no object in the tree that two operators can compute
separately from an agreed datum, nor a root that accumulates it, nor a head that signs it, nor a
reader that compares two heads. The four pieces are this RFC.

⚠️ §890: the line citations of the bullets on the five roots and on the cost of a signed field
are from `2298e61`, the tree this document declares (above, «Every measurement…»), and there
they fall in their place: §890 checked them. In `212eca2` they no longer do:
`zk-ssl-verify/src/lib.rs:329` is a field of a `struct`, not the version byte that `preambulo`
writes, and `TODAS` carries five versions, not two. They are kept as the measurement of their
date. The other line citations of this section were not checked.

## Design

### D-1 — The name: published consumption

The object is called **consumption**; its prefix at rest is `cons:` and its root `root:cons`.
Adversary: whoever certifies the same unit twice. Proof: the consumption is under the root of
the signed head of epoch t and was not under that of an earlier head. A consumption is
`H(DOMINIO_CONSUMO, identificador)` over an identifier **agreed and public**; whoever knows it
computes it. That is deliberate and it is the opposite of the spending nullifier: here
precomputability is the property, not the defect.

Discarded, with their measurement: «nulificador» [nullifier] (two live and one dead, three
spellings; law 2, a name means one thing only), «marca» [mark] (the accreditation mark of note 99
and the end-of-file mark), «uso» [use] (`meta:cust_uses`, the custodians' quota, §394). The judge
of the name (`PASTE-412-PRE`): `cons:`, `root:cons` and `consumo` give **zero** in the 158 `.rs`;
in the 67 `.md` the word appears five times, four as «CPU de consumo» [CPU consumption] and one in
`VISION.md:574` («el consumo va al aplicar» [the consumption happens on apply], the consumption of
a pending). That neighbourhood is declared: the consumption of this RFC is a published object, not
the phase of a payment.

### D-2 — Where it lives: the minimum, and the circuit as a declared extension

A sparse tree of the layer, modelled on that of the frozen ones (§391): leaf = consumption,
position = the full digest and not its low bits (the position collision of `nullifier_tree.rs` is
not inherited, as note 3045 of `BACKLOG.md` asks). Its root `root:cons` is stored in `commit` and
checked in `load` with `need()`: a ledger without `root:cons` **does not open** (fail-closed, like
the five before it; migrating is a separate operation, §392). The snapshot goes up from version 4
to 5 and transports the set; a v4 is still imported with the empty set, declared. Publishing a
consumption is an operation of the layer that enters the transition log as an entry of its own,
and **a consumption already present is rejected** by name. Three negative witnesses, shown red
before the fix: the repeated one is rejected; without `root:cons` it does not open; with a false
`root:cons` it does not open.

> ⚠️ **Correction (§413, on sealing E1).** Two sentences of this paragraph and discard 8 were
> written on a skimmed reading and were falsified on measuring the whole fifth root
> (`PASTE-E1-M`, `PASTE-E1-M2`, `PASTE-E1-M3`): (1) **the snapshot was not at version 4 but
> at 7** (`MAGIC_V7`, `snapshot.rs`; the «Versión 4 del formato» [Version 4 of the format] of its
> doc-comment was history, v4 was the one that retired the nullifier): E1 raises it from **v7 to
> v8** (`ZKSSL8`), and v3..v7 are still imported with the empty set (⚠️ §802: since §802, only if
> their log has no `Recovery`, by the D-1 the author took: `SECURITY.md` §3.13); (2) **the
> position cannot be the full digest**: `SparseTree` is indexed by `u64`, so the position is the
> **low 63 bits of the first eight bytes** of the consumption (`CONS_DEPTH = 63`), and the leaf is
> the whole digest, so that a prefix collision (2⁻⁶³ per pair) is detected and rejected with its
> own name (`ConsumoColision`), not as a repeat. Discard 8 is thereby corrected: what is
> discarded is the ~32-bit position of a circuit digest; the 63 bits are what the layer's tree
> admits, and they are declared. The text above is kept as it was written.

What the minimum **proves** is exactly that and nothing more: that a consumption is under the
root and was not there before. **It does not prove that the consumption corresponds to a
payment** nor that the payment is the one it says: what the circuit does not constrain does not
exist, and here the circuit constrains nothing. Tying the consumption to the operation in the AIR
is E5, with `nullifier_tree.rs` (non-membership and insertion in lockstep, already proven in its
crate) as the starting piece, and it stays outside this RFC by the short rule: without E1 there
is no ledger with consumptions, so the witness that would falsify the tie cannot be written today.

### D-3 — The head: v4 carries a single family

`epoch_digest_v4 = merge(epoch_digest_v3(los nueve), merge(root_cons, as_digest(k)))`, with `k`
the count of consumptions under the root — the mould of §275 and §292, again, with no domain, with
the version byte of the preamble going from 3 to 4. Genesis, declared: the first head composes
with the root of the empty tree and `k = 0`. `VersionCabeza` gains `V4` and the accepted set
becomes {2, 3, 4}: the test that today demands `[2, 3]` goes red and is corrected with the seal,
not before. The wire serves the two new fields next to the nine of the head; the signature covers
them.

Point 43 of the queue —the head that signs `supply` and `meta_root`— **does not ride in this v4**:
it has neither an RFC nor a written falsifier, and a version that puts in two families without
measuring the second is the class of silent era the law forbids. It is declared: v4 is taken by
this RFC; 43, when it writes its witness, is v5. Reversible in §412.

### D-4 — The limit, written

- **The agreed identifier is governance, not cryptography.** `doc/USE_CASES.md:66-69` already
  says so; this RFC does not change it. Two bodies that do not agree on the identifier detect
  nothing.
- **The proof does not say that the unit is real** (the invoice, the eligible expense): it is the
  limit of the oracle of `SECURITY.md`, and the consumption inherits it whole.
- **Within a ledger it is an invariant; across ledgers it is detection, not prevention.** A
  single node orders; nobody stops two ledgers from accepting the same consumption. What this RFC
  gives a third party is that, with the two signed heads in hand, they see it. E4 falsifies the
  detection; it promises no more.
- **Whoever publishes first blocks.** A consumption is precomputable by design, so a third party
  with access to the node can publish it before the legitimate holder. It is denial of service,
  not double use, and it is declared with the same weight with which `nullifier_tree.rs` declares
  its position collision. Mitigating it (tying the consumption to an authorisation) is E5 or a
  later RFC.
- **Unobservability: none.** Whoever knows the identifier sees when it was consumed. It is the
  property the use case asks for, and the law forbids selling absolute privacy: here none is
  sold.
- **The operator is still another adversary.** It can omit a consumption or delay it; it cannot
  forge the root it signed nor withdraw one already published without the head ceasing to extend
  the previous one (§292).
- **The window, and the authoritative registry.** The bullets above say what this RFC does NOT
  promise. This one says what could be promised and at what price, and it is written on sealing
  E4a because it is the stage that makes the limit concrete. **Within a ledger, single use is a
  checkable invariant. Across ledgers, without an authoritative registry, what can be prevented
  is double use OUTSIDE a declared and measured time window; inside that window there is only
  detection. With an authoritative registry it is always prevented, and then the system has one
  more operator, with its single point of failure and of censorship, named.** This RFC specifies
  the detection and promises no more. The other two stay outside and it is said why: blocking at
  processing demands the window as a **measured figure** in the two-node bench —not as prose—,
  and the authoritative registry is not an engineering decision: if a body takes it on, it is ITS
  operator, not ours.

## What was DISCARDED on measuring

1. **Calling it a nullifier.** Discarded: three meanings, live or dead, in the same tree (D-1).
2. **Reusing `root:nullifier` and `null:`.** Discarded: `load` deletes them (§36,
   `persistence.rs:656-668`).
3. **Starting with the circuit.** Discarded (reversible): the negative witness of the minimum
   —repeated, no root, false root— is written today; that of the tie in the AIR demands E1.
   Witness before feature.
4. **Putting `supply` and `meta_root` (point 43) into the same v4.** Discarded (reversible): with
   neither an RFC nor a falsifier of its own. 43 would be v5.
5. **The label with shared key material (OPRF), as D-C of RFC-0005 pointed to.** Discarded for
   now (reversible): it demands a new trust assumption and material that nobody holds whole; the
   minimum publishes the label in the clear and declares what that costs (D-4). If the use case
   demands that a third party not see when it was consumed, it is a new family with its RFC.
6. **Calling the detection across ledgers «prevention».** Discarded: a single node orders;
   without a reader of two heads there is nothing, and with it there is detection (D-4).
7. **Fixing here the name of the wire method, of the vector file or of the form of the package.**
   Discarded by the same measurement as in RFCs 0004 and 0005 (discard 7):
   `tools/verificar_citas.py` demands that every cited document exist, and this RFC cannot cite
   what does not yet exist. They are fixed in E1, E2 and E3, in the commit that creates them.
8. **Deriving the position from the low bits of the consumption.** Discarded: it is the declared
   collision of `nullifier_tree.rs`; the layer's tree indexes by the full digest, like `froz:`.

## Compatibility

| stage | does it break the wire? | vectors |
|---|---|---|
| E1 — the tree and the root | **NO.** No method or type of the wire changes; the log entry is additive | the three of the wire, those of the package, those of the negative wire and the KATs, intact |
| E2 — the v4 head | **YES.** `formatVersion` becomes 4 and two new signed fields travel: `zkssl/0.3` → `zkssl/0.4` | `zkssl-0.1.json`, `0.2` and `0.3` intact; `zkssl-0.4.json` is born; the wire rejections of `0.3` stay under their version (a `0.3` consumer keeps rejecting `formatVersion: 4`, and that is correct); the KAT of `epoch_digest_v4` and of the v4 preamble is born |
| E3 — the portable proof | **NO.** New form of the package, with its `tipo` and its vectors under its directory | the 67 of the package, intact; a separate catalogue is born |
| E4 — the bench | **NO.** Tool, not format | intact |

**Rule 2 of the PROCESO: old vectors are never rewritten.** `zkssl-0.1.json`,
`zkssl-0.2.json`, `zkssl-0.3.json`, the 66 files of `spec/vectors/paquete/`, the 11 of
`spec/vectors/cable/` and the 18 of `spec/vectors/nucleo/` are preserved without one modification.

> **Correction (§414, E2a).** Two sentences of this document were measured before sealing E2a
> and came up short; they are corrected here, without rewriting them (rule of §247). **(1)** The
> `k` of D-3 is called `cons_count` in the tree (`consumo.rs`, next to `cons_root`), and on the
> wire `consCount` next to `consRoot`, by the mould of `acusesRoot`/`n` (v2) and
> `mmrRoot`/`mmrSize` (v3): `k` already names the witness's co-signature policy, and a name means
> one thing only. **(2)** «A `0.3` consumer keeps rejecting `formatVersion: 4`» is falsified by
> nobody: the harness runs the wire rejections against the HEAD binary, and a `.json` with no
> entry in the manifest is red (`tools/conformidad.sh`). The two `rechazo-formatVersion-4.json`
> are not rewritten and stay listed: since E2a they are rejected **for another cause**, with the
> text measured live —the command, for the missing fields (`falta seq`); the witness, for the
> signature, whose preamble carries byte 3 and not the 4 the head declares—; the out-of-set case
> lives in `rechazo-formatVersion-5.json`, derived as `TODAS.max + 1`. E2a is additive:
> `VERSION_FORMATO` stays at 3, the node does not emit v4 and `zkssl/0.3` does not move; that is
> E2b.

> **Correction (§415, E2b).** This document said three times that E2 would raise the wire to
> `zkssl/0.4` (the header, the row of stages and Compatibility). It was measured before sealing
> E2b and it is false: the vector the canon seals (`zkssl-0.N.json`) carries the log entries, an
> `epoch_digest` composed as **explicit v2** (§292: a frozen artefact), the supply and the
> pending, and E2b moves none of them —`conformance --check` keeps giving «todo IDENTICO»—. The
> versioning rule (`spec/RPC.md`, Notas operativas [operational notes]) raises the version when
> the **values that travel** in the vectors change, and its two precedents of this same class
> (§275, the acknowledgement pair; §292, the MMR pair) did not raise it. E2b follows the
> precedent: **`zkssl/0.3` does not go up**, the FORMAT version travels in the signature
> (`formatVersion: 4`) and an old typed parser breaks out loud by `deny_unknown_fields`,
> declared. No `zkssl-0.4.json` is born. Nor is a «KAT of the v4 preamble» born:
> `spec/NUCLEO.md` fixes one file per `fn` of the core, and the version byte is an INPUT of the
> `preambulo` KAT, not a new function. Reversible in §415.

> **Correction (§417, E3a).** The E3 row says «NO [it does not break the wire]. New form of the
> package, with its `tipo` and its vectors under its directory». The first is still true and the
> second falls short: **E3 DOES touch the wire**. The package is specified as «responses of the
> wire AS THEY ARE, gathered» (`spec/PAQUETE.md`, sections 1 and 7), and the path of a consumption
> was served by no method, so either it was born on the wire or the envelope stopped being what
> it says it is. `zkssl_publishConsumo` and `zkssl_consumoPath` are born (§417): **additive, the
> surface goes from 24 to 26 and `zkssl/0.3` does not go up**, by the rule of the Notas operativas
> and its precedents §222, §242 and §275. The three vectors of the wire stay intact and the
> triple gate falsifies it in every run.
> **Correction (§430, E4a).** The E4 row says «**NO.** Tool, not format». The first is still
> true —not a method or a type of the wire changes, and `zkssl/0.3` does not move— and the second
> no longer is. **The reader of the conflict is the COMMAND, not a script of the bench**: a
> detection that only runs inside an in-house bench cannot be exercised by the third party who
> downloads the tarball, and the H3 criterion —«passes and fails the same»— is measured with
> deliverable material behind it. ⇒ E4a adds the **fifth form of the envelope**
> (`tipo: "conflicto"`), with its group in the catalogue and its vector directory, exactly as E3
> added the fourth. It is still true that the format of the WIRE is not touched; what is touched
> is that of the PACKAGE.
> **Correction (§436, E4b).** The bullet «The window, and the authoritative registry» says that
> blocking at processing «demands the window as a **measured figure** in the two-node bench —not
> as prose—». It was written before building it and came out **stricter than the tree needed**:
> §436 blocks with the **SIGNED evidence alone** —the other ledger's head and the list of
> consumptions its own signature attests— and **with no window at all**. What the measurement
> changed is the subject: the window **cannot be a measured figure at all**, because there is no
> signed clock that crosses ledgers. `emitida_unix` is computed AFTER the signature and its
> `unwrap_or(0)` turns a broken clock into 1970; `seq` and the XMSS index are PER LEDGER. ⇒ the
> window is a **DECLARED operational parameter and never a guarantee** (D-D, §433-B), measuring
> it is E4b-2, and the blocking did not depend on it. The bullet is not rewritten: it told the
> truth of its date.

> **Correction (§439, E4b-2).** The correction above left said what the window **cannot
> be**. This one says what **is affirmed**, because on measuring something signed appeared
> underneath that can be falsified. **It is not a time window: it is a BOUNDARY in the signed
> sequence of the other ledger.** This node rejects every consumption that that ledger had under
> the `consRoot` of the head `seq = N` it was handed, and **affirms nothing about what that
> ledger may have signed after N**.
>
> **It is ONE limit with TWO faces, not two loose limits.** Nothing in the signed material tells
> this node whether what it holds in hand is the latest of the other ledger. Forward, it does not
> block what comes after `N`; backward, it does not know whether `N` was already passed. A single
> property, two consequences.
>
> **Why `N` cannot be forged, and it is measured.** `seq` is not a counter that someone
> increments: it is the **height of the log** (`entries.len()`), so skipping it or going back on
> it demands losing entries. And **nobody reads it on loading and even so it is attested**: it
> travels under the signature as the first field of `epoch_digest_v4`, the digest is
> **recomputed** from the recomposed head, and the signature is verified against the computed
> one — a lied `seq` breaks the digest before anyone looks at it. The pair `consRoot`/`consCount`
> fixes the **set**; the `epochDigest` **identifies** it; the signed `seq` **orders** it, and it
> is the order that makes the two faces above writable.
>
> **The two continuations are named and not built, and they attack the same root.** That this
> node's head declare which boundary applied —it would make it auditable by a third party, and it
> costs a new field in the head, with its rise of format version and its own RFC—; and that the
> node refuse a boundary older than another it already had —which demands that the node persist
> the boundary it already applied, and today it loads it once on start-up and stores nothing of
> it—.

### Why it goes through an RFC

By the letter: E2 changes values that travel (`spec/RPC.md`, `spec/openrpc.json`, the vectors)
and raises the version of the wire; E3 adds under `spec/vectors/`. By the spirit: it is the
first family that enters **under the signature of the head** since RFC-0005 wrote where one
enters; if it does not go through an RFC, the extension rule debuts unfulfilled.

> **Note (2026-09-09, §441).** ACCEPTED. Rule 4 of the PROCESO is settled with measurement, and
> it is said with what. (a) The spec, updated: `spec/RPC.md` gains the two consumption methods and
> their section (§417); `spec/NUCLEO.md`, the position of the consumption and the CONSUMO family
> (§416, §419); `spec/PAQUETE.md`, the fourth and the fifth form of the envelope (§419, §430).
> (b) The OpenRPC, regenerated with `gen_openrpc` in §417: 24 → 26 methods, and `zkssl/0.3` does
> not go up. The paragraph above is the fourth time this document took for granted that E2 raised
> the wire: correction §415 refuted the other three and this note refutes this one. (c) The
> vectors, new and none rewritten: `spec/vectors/consumo/` (14, §422 and §431),
> `spec/vectors/conflicto/` (16, §431), the KAT of `epoch_digest_v4` and the negatives
> `rechazo-formatVersion-5` of the wire and of the package (§414–§415); the three of the wire,
> intact under the triple gate. (d) The suites, green: the `--sello` canon of §440 (178 s) runs
> the three catalogues also from inside the tarball —package 68/68, consumption 14/14, conflict
> 16/16—. E5 stays out of scope, as the table declares. Double thread of rule 5: this document
> cites §441 and §441 cites this RFC. Precedents of the turn: 0003 (§356) and 0004 (§400).

## Security

**Effect on the principle of the API —the spending key never travels: NONE.** The consumption is
derived from a public identifier; no key intervenes, neither spending, nor viewing, nor
custodian. Publishing it demands proving nothing to the node, and that is a declared limitation
(D-4, «whoever publishes first blocks»), not an erosion of the principle.

**Effect on the declared debts:**

- **Out-of-band notice (§21): none.** The consumption transports no notices.
- **Single node: none.** E4 runs two nodes because they are two ledgers of two operators, not
  two replicas of one; this RFC introduces consensus at no stage (`SECURITY.md`, «un solo
  nodo» [a single node]), and detection across ledgers works precisely because it does not need
  it.
- **`--dev`: none.** There are no test custodians on the path of the consumption.

**What this RFC newly publishes:** one more root under the signature and a set of public and
precomputable labels. What it does **not** affirm: that a consumption corresponds to a payment
(E5), that the identifier is correct (governance), that the unit exists (oracle), that two
ledgers cannot accept the same consumption (detection, not prevention), nor that nobody sees when
it was consumed.

## References

Measured on `2298e61`; fingerprints in `sha256 | cut -c1-16`, lines in `wc -l`.

- `spec/rfc/PROCESO.md` `0eba90e3fe5d93f8` / 24 ·
  `spec/rfc/0000-plantilla.md` `488e7055cbfcad09` / 29 ·
  `spec/rfc/0005-nucleo-congelado.md` `3489769d5af99460` / 315 — the mould; D-B «Familias
  nuevas» (`:146-154`), D-C (`:156-167`), discard 6 (`:219-221`).
- `crates/zk-ssl/src/persistence.rs` `3264d97198d68236` / 878 — the legacy
  `null:`/`root:nullifier` (`:456-680`), the five `need("root:…")` of `load` (`:484-562`), the
  five writes of `commit` (`:774-783`).
- `crates/zk-ssl-hash/src/lib.rs` `2f05cfd9fcf7e690` / 896 — `epoch_digest` `:200` / v2 `:224` /
  v3 `:257`.
- `crates/zk-ssl-verify/src/lib.rs` `b4a2cefe564aae03` / 1154 — `VERSION_FORMATO` `:132`,
  `VersionCabeza` `:142`, `TODAS` `:151`, `texto` `:157`, `preambulo` `:329`.
- `crates/zk-ssl-wire/src/lib.rs` `9669d5d02050ad70` / 1354 — `SignedEpochHeadDto` `:695`.
- `crates/zk-ssl-cli/src/witness.rs` `819df9e2d790dde8` / 4738 — `recomponer` `:843`, the `match`
  of versions `:871`. `crates/zk-ssl-verify/src/main.rs` `6dc17e15b8b2c97e` / 570 — `:129`,
  `:148`.
- `crates/zk-core/src/spend_authority.rs` `2afc3874aea442aa` / 239 — `derive_nullifier` `:86`.
  `crates/stark-experiment/src/nullifier_tree.rs` `7dd2d2b4d0b2578c` / 793 — header `:1-43`,
  `NullifierPublicInputs` `:221`.
  `crates/stark-experiment/src/circuit_threshold_single_nullifier.rs` `02bd2cdc63108d89` / 1119
  — `:243`.
- `spec/RPC.md` `272ff83140600046` / 889 — `zkssl_epochHead` `:69`, `zkssl_signedEpochHead`
  `:433-478`, the residual trust `:873`.
- `doc/USE_CASES.md` `22fcbb837ae7df72` / 137 — the row `:28`, the case `:62-69`.
  `SECURITY.md` `e98ebd8462b0a3c5` / 651 — `:376`. `BACKLOG.md` `c2db6c5e38a442a2` / 3235 — `:3045`.
- Pure readings of session 99: `PASTE-NULL-M` `de16e15a4efa9bed` (output `af3599b28a30b0d5` /
  2228) · `PASTE-412-PRE` `fb58926ddaf09346` (`9fe4664b793ad743` / 835).

**Numbering.** **RFC-0001 remains reserved** for the hardening of the keystore's KDF. This RFC
takes **0006** by sequence, after 0005 (derived in `PASTE-412-PRE`: 0, 2, 3, 4, 5 taken).

**Delegated decisions (session 99).** The author delegated to the assistant the four decisions
D-1 to D-4 with the instruction to apply the project's decision constitution. All are REVERSIBLE
and §412, which seals this RFC, records them one by one.
