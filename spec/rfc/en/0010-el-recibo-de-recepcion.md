# RFC-0010 — The reception receipt: what the operator cannot deny having received

> **Español:** [`spec/rfc/0010-el-recibo-de-recepcion.md`](../0010-el-recibo-de-recepcion.md) is the
> original of this page, in Spanish; it prevails wherever the two differ. This translation is of
> commit `7766efb` (audit entry §917) and was produced by the project's assistant, a Claude Code
> session, in audit entry §918 (`GENAI.md`): it is a generated artefact, not text written by the
> author. Line citations of the form `file:N` refer to the Spanish files.

- **Status:** ACCEPTED — **rule 4 of the PROCESO, settled with measurement** (§556 the adoption; E2
  §557–§570 —the v6 composition in the core, the rules of the receipt, the register and the
  node's view of receipts, and the signed pair on the wire—, E3 §571 —the receipt on the wire and
  `zkssl_recepPath`—, the binding of the ceiling `N` §572, E4 §573 —the completeness envelope— and
  E5 §574 —the bench and the catalogue, the ninth family of the artefact and of the canon—, §576 the
  scope of the receipt, decided and bound; §577 this turn). The spec is up to date (`NUCLEO.md`
  with the v6 composition and the leaf of the receipt, `RPC.md` with `recepcion` and
  `zkssl_recepPath`, `PAQUETE.md` 2.11 and section 9); the OpenRPC was regenerated in §571 with
  method 31, and the wire test binds it to the table; the vectors are new under `zkssl/0.4`
  —`nucleo/epoch_digest_v6.json` and `nucleo/recibo_digest.json` (§557), the out-of-set
  `rechazo-formatVersion-7` of the wire and of the package (§558-B), the v6 positive and two wire
  rejections (§570) and `completitud/`, thirty-five envelopes and their manifest (§574), thirty-six
  since §605 and seventy-three since §614, with those of RFC-0014—; and the suites are green: the
  canon runs them through the harness, and also from inside the tarball. **What is accepted WITH
  its residue, declared:** verdict 1 has no vector —it asks for the real STARK proof of an applied
  send; its verification is that of the position package. Since §602 it is produced LIVE, in
  `tools/banco_mentiroso_sin_resolver.sh`, and since §605 it has its vector in `completitud/`
  (BACKLOG 110, closed)—;
  the operator can issue no receipt (D-H); and what comes in through `applyMany` or through
  `zkssl_pledge` did not carry one, by decision (D-E, §576) and bound by a witness, until RFC-0014
  gave it a stage: it carries one since §611 (its E3), and the completeness envelope resolves it
  since §612 and §613 (its E4). It is preserved as a record of what was decided, what was measured
  and what was discarded.
- **Author:** Ángel José Toranzo Portela
- **GenAI assistance:** Claude (session 184, which writes it; and 193, from §566 to §577; the
  stages in between are dated by their entries) — see [`GENAI.md`](../../../GENAI.md)
- **Date:** 2026-09-25
- **Affected protocol version:** `zkssl/0.4` — **does not go up** (see Compatibility). The head
  moves to **v6** with an additive pair, exactly like the v4 of RFC-0006 (§414) and the v5 of
  RFC-0007 (§451), which did not move the wire either; the vectors of the new era are born as
  those of v5 were born in §453.
- **AUDITORIA entries:** §115 (the heartbeat and its cadence), §116 (`digest_of_proof` collides
  with trailing zeros), §120 (the name «acuse» [acknowledgement], reserved), §121 (the policy of
  the acknowledgement: the ceiling `N`, the clock of signed heads, and the two corrections — no
  signature per acknowledgement, and the hash of the proof with encoded length), §234 (the
  guardian with `fsync` that refuses on `tmpfs`), §242 (the index gaps on restart, declared
  benign), §248 (the RPC does not serve the root: the holder recomposes it), §253 (the reception
  counter), §270–§275 (the acknowledgement tree: the leaf, the delegated ones, the limits of the
  journal, the node's view and the signed `n`), §414 and §415 (the consumption pair, the mould of
  a new pair in the head), §451–§453 (the v5 head and the vectors of its era), §555 (the operator
  of ATADO D [binding D], without which this RFC goes in blind); and §556, which adopts it. The
  stages, from §557 to §574 (see «Status of the stages»); §576, which decides the scope of the
  receipt; and §577, which accepts it.
- **Milestone:** H5b of the proposal sent to NLnet Restack (120 h), in its words: *«The proof of
  completeness. Every committed acknowledgement resolves within a bounded epoch as an applied
  transition or a rejection with proof; the residue declared.»*

## Status of the stages

| stage | what it delivers | does it break the wire? | status |
|---|---|---|---|
| E1 — the promise, written | this text: what object is born and why it is not the acknowledgement (D-A), the leaf and its domain (D-B), the pair in the v6 head (D-C), the era the receipt declares (D-D), what counts and what does not (D-E), the completeness envelope and its three verdicts (D-F), the crack of the causes without a portable proof (D-G) and the residue (D-H) | no | **sealed (§556)** |
| E2 — the reception root in the head | **E2a** the v6 composition in the core with its known vector, V6 in the set and the consumers on the predicate — sealed (§557, §558, §559 the binary, §563 the cli and the seven benches, §566 the v6 acknowledgement of the command) · **E2b** the shared rules in `zk-ssl-verify`, mould `acuses.rs` — sealed (§562; §567 the edge `(Q, R]` and the era in XMSS index) · **E2c** the node's view, mould `vista_acuses.rs` — E2c-1 the reception register, sealed (§565); E2c-2 the view of receipts (§568) and the connection of the register to the node (§569) — sealed · **E2d** the signed pair on the wire (5.A-422) — sealed (§570): the node composes, signs and serves v6, `Q` comes from the previous head or from the journal | no (additive, v6) | **sealed** (§557–§570) |
| E3 — the receipt, on the wire | `{rx, era, n}` as DATA in the result and in `error.data` of the holder's ways, and the method to read the path when the era closes — `recepcion: {rx, era, n, hashPrueba}` and `zkssl_recepPath` | no (additive methods) | **sealed (§571)** |
| E4 — the portable completeness envelope | `tipo: "completitud"`, which a third party verifies with the kit and without a node, with its three verdicts and its rejection rules — sealed (§573) with the four decisions of the author in session 193: `Q` declared, the window measured with a current signed head, the rejection bound by `data.recepcion` (the node's word) and the fourth state with exit 3 | no | **sealed (§573)** |
| E5 — catalogue and bench | `spec/vectors/completitud/`, its MANIFIESTO, the family in `FAMILIAS` and its stanza of the canon, and the bench that seeds one resolved reception and one unresolved — sealed (§574): `tools/banco_completitud.sh` against a real node that signs, THREE positives captured —the one resolved by rejection, the NOT RESOLVED one with the window EXPIRED and the fourth state— and THIRTY-TWO negatives, one per producible rule and per site; verdict 1 is not seeded —it asks for a real STARK proof of an applied send— and it is declared | no | **sealed (§574)** |

### The frontier with H4 and with H5, and what belongs to each

RFC-0007 wrote that the completeness of the acknowledgements **is not its own**, and RFC-0008
repeated the frontier. This document looks at it from the other side and closes it:

- **RFC-0007 owns** *what* the node rejects and *how it proves it*: the cause as data (E2) and
  the envelope `tipo: "rechazo"` that the command verifies without a node (E3). This RFC **does
  not touch a cause or a vector of that family**: it consumes them.
- **RFC-0008 owns** what a third party can check of a pending one. Nothing of this depends on
  this document nor the other way round.
- **§121 owns** *what the operator promises on receiving*: the ADMISSION receipt, whose policy
  this document inherits whole and keeps. What is missing from it is the extension of the head
  that §253 already has on file, and **this document does not build it**: here the
  acknowledgement is the object whose resolution is proven, and the statement of H5b puts the
  acknowledgement not issued on the side of what is DECLARED. A decision of the assistant,
  DELEGATED with the law and REVERSIBLE: if the two receipts have to go in one expediente
  [evidence package], this point is withdrawn and the table gains its row.
- **This RFC owns** *when*: that every operation the node got to evaluate is resolved within a
  bounded window, and that the holder has the means to prove that it was not resolved.

## Motivation

The node orders, and it can stay silent. It is declared in four live documents and it is not an
oversight: it is the model. What is not there is the operational consequence, and the measurement
shows it without ambiguity.

The path of an operation has four stages —`parse` → `try_into` → `apply` → `append`— and `seq`
is only born in the last one. **Censorship lives in the gap between receiving and applying, and
today that stretch leaves no signed trace.** An operator that censors does not increment `seq`,
and nothing gives it away.

Half the machine is built and measured. There is a monotonic reception counter, persisted with
`fsync` before returning the number, which refuses to operate on a medium that does not persist
and which survives restart without reuse (§234, §253). It travels to the holder as
`receptionSeq`. With it, two holders who cooperate **detect** the reordering: A has reception
100, B has 101, B's is in the log and A's is not. And there it ends: **they cannot prove it**,
because nobody has anything signed by the operator that says *«I received yours as number 100»*.
The operator can deny it.

What is missing is named in the code itself since §253 —*«el acuse como hoja bajo una raíz en la
cabeza»* [the acknowledgement as a leaf under a root in the head]— and filed as one of the two
pending extensions of the head. And the policy was already decided in §121, with its ceiling:
**non-inclusion in `N` epochs is censorship**, with `N = 1.440` signed heads, twenty-four hours
at the heartbeat of §115, with the precedent of the MMD of Certificate Transparency. That constant
lives in the tree today, travels signed in the head since §275 and enters every leaf of the
acknowledgement, so that a false `n` does not verify.
Since §572 the prose that cites it is BOUND to that constant: `tools/check_techo.py`, in the
canon, reads `N_MAX_CABEZAS` from the code and demands that every site of the published perimeter
—this RFC, `spec/RPC.md` and `doc/CONFIANZA_RESIDUAL.md`— say that number, located by pattern and
not by line.

The missing piece is an object, not an idea: **a reception root in the signed head**. With it,
the equation that `doc/CONFIANZA_RESIDUAL.md` already writes becomes checkable by a third party:
a signed receipt plus no resolution within `N` heads is portable evidence of censorship.
Censorship is still possible; it stops being free.

## Design

### D-A — The new object is the RECEPTION RECEIPT, and it is not the acknowledgement

The «acuse» [acknowledgement] is taken. Today it is the leaf of an **applied transition**:
`vista_acuses` builds the epoch's tree from `transition_log().entries()`, so an acknowledgement
**only exists for what was already applied**. That is why the milestone's sentence —«every
committed acknowledgement resolves… or as a rejection with proof»— is today true vacuously: the
only branch that can occur is the first.

A sibling and separate object is born, the **reception receipt**: the leaf of an operation that
the node **got to evaluate**, whether it was applied or not. Two trees, two roots, two pairs in
the head. The acknowledgement tree is not reused: mixing them would make the number of leaves
stop meaning one thing only, and the house does not recycle a name when what it commits changes.

**Reversal:** a single tree with a class bit per leaf. It is discarded in «What was DISCARDED».

### D-B — The leaf, with its own domain and the mould of the acknowledgement

`recibo_digest(hash_de_la_prueba, era, n) = merge(as_digest(RECEP_V1), merge(hash_de_la_prueba,
merge(as_digest(era), as_digest(n))))`, traced from `acuse_digest` with a seventh domain,
`RECEP_V1`, in the REGISTRY of `zk-ssl-hash`. Same form, same class of object, another domain:
a receipt can never pass for an acknowledgement nor the other way round.

`hash_de_la_prueba` goes **with the length encoded**, not as plain `digest_of_proof`: §116
measured that this digest collides with trailing zeros, and §121 already left it decided. It is
kept.

`n` goes **inside the leaf**, as in the acknowledgement (§270): changing the ceiling changes the
whole tree, and that is why a false `n` in a response produces a leaf that does not verify
against the root recomposed with the `n` of the **signed** head.

### D-C — The v6 head signs the pair `(recepRoot, recepCount)`

`v6 = merge(v5, merge(recep_root, as_digest(recep_count)))`, with the exact mould of the
consumption pair that v4 introduced (§414). Genesis: the root of the empty tree and
`recep_count = 0`.

`recep_count` is the reception counter at the moment of composing the head. The **lower** bound
of the era is not signed: it is the `recep_count` of the previous head, and the holder who keeps
two consecutive heads has it. It is the same split that `limites_para` already does with the
`seq` of the journal, and the same edge convention: the era is `[Q, R)`, with `Q` **inclusive**,
because with `Q` exclusive reception number one would belong to no era.

The index of the leaf inside the tree of its era is **dense from zero**: `rx - Q`. Dense on
purpose, as in the acknowledgement: the era is small and anyone rebuilds positions from two
signed heads without extra data.

⚠️ **CORRECTION (§567), cited and not erased.** The two paragraphs above reason with a
reception number zero that the counter **does not give**: `reservar` returns `actual + 1`, so the
first reception is `rx = 1`. With `[Q, R)` and `R = recep_count`, the LAST reception of each era
was left out of its own tree. What rules, by the author's decision in session 193: the era is
**`(Q, R]`**, with `R = recep_count` —the receptions evaluated when composing the head, which is
the last `rx` reserved— and `Q` that of the previous head; `Q < rx <= R` belongs; the dense index
is **`rx - Q - 1`**. The genesis `recep_count = 0` is still the empty tree, and the convention
of the receipt is exactly that of the acknowledgement applied to `rx - 1`.

### D-D — The era the receipt declares is the first head that can contain it

`era = seq_de_la_última_cabeza_firmada + 1`, computed **at reception**, not at closing. It is
the mirror of `epoca_de_acuse(seq) = seq + 1` and it is chosen for the same reason as there:
binding the era to the real head that ends up containing it **would put the value of the
evidence in the hands of the accused** —the holder could not fix their receipt until the
operator decided—.

And it gives the magnitude the promise bounds: if the receipt declares `era = e` and the holder
shows a signed head with `seq = S`, then `S - e` is the delay **in heads**, readable from the
receipt and the head alone. The promise is `S - e <= N`.

An era set ahead is of no use to the operator: the holder checks `e` against the head they
already keep at the moment of receiving, and an `e` ahead of what the node has signed is visible
at once. And since the era goes **inside** the leaf, lying about it afterwards breaks the path.

⚠️ **CORRECTION (§567), cited and not erased.** This section fixes the era on the `seq` of the
last signed head and says that `S - e` is the delay **in heads**. The `seq` of a head is
`log.len()` —the clock of §340— and the heartbeat emits a head even if there are no transitions,
so two heads in a row carry the same `seq`: `S - e` counted **applied entries**, and a censor that
stopped applying **froze** the clock, with the window open forever. What rules, by the author's
decision in session 193: the era and `S` are the **XMSS index** of the head's signature. The
node's key only signs heads, so the index counts **signed heads** —plus the orphan indices, burnt
without a signature, which only shorten the window and count against whoever burnt them—, and
`N = 1.440` is again what it says: 1,440 signed heads.

⚠️ **AMENDMENT (§681, RFC-0019 E1, the author's decision), cited and not erased.** The correction
of §567 had two holes in the kit, which read `S` from the **declared** `index` of the head and `e`
from the leaf. The declared one does not enter what is signed (§399: it is only bound from below,
`embebido < declarado`): anyone could inflate it in an envelope and fabricate a «NO RESUELTA EN LA
VENTANA» against an honest operator (H10 of the re-triage of the second swarm). And the `e` of the
leaf is counted by the node with its counter (`anotar`: the declared one of the last head plus
one), in which an index burnt without signing —free for the operator— would move every window.
What rules: the window is measured in the unit of the SIGNATURE at both its ends. It starts at the
**embedded** index of the closing —the credited one of the last head signed before it, the era in
that unit— and `S` is the **credited** one (`embebido + 1`) of the head that measures it; the
promise is `S - inicio <= N`. The orphans stop counting against whoever burnt them: it is the only
thing that is lost. The `e` of the leaf stays inside the leaf and still binds the receipt to its
path; it no longer measures the window. In an honest flow without orphans the start is the era,
or the era minus one when the receipt arrived while signing: in the operator's favour, at most one
head. The 74 published vectors of the completeness envelope say the same with the new rule.
⚠️ §815: the `e` of the leaf is no longer counted by `anotar`. Since §815 it is FIXED on receiving
(`era_al_recibir`, in `crates/zk-ssl-node/src/main.rs`), before the layer judges, and `anotar`
only writes it: it is the same reading, at the same point (D-E, amended below).

### D-E — It counts what the node got to EVALUATE; neither the noise nor what was accepted

The rule is already measured and written in `recepcion.rs`, and this RFC raises it to a norm
without changing it:

| fails at | does it consume a receipt? | why |
|---|---|---|
| the parsing or the wire | **no** | it is noise, not an operation; if it counted, anyone could open gaps in someone else's register by sending garbage |
| the layer (invalid proof, moved root) | **yes** | the node verified a real operation and decided not to apply it, **and that is where a censor would hide**: rejecting on the claim of an invalid proof |

The counter is the one that already exists (`ContadorRecepcion`), with its `fsync` before
returning and its refusal to start on a medium that does not persist. **No second counter is
born**: two implementations of the same problem can disagree.

**The register records at the SAME point (§569)**: in `applySend` and `applyClaim`, after
reserving the number and before the layer, with the era of the index of the last signature
(D-D). If recording fails, the operation **is not evaluated** and the number stays a **gap**
—reserved without a leaf—, which the start-up declares when reconciling the register with the
counter.

⚠️ **AMENDMENT (§815), cited and not erased: the node JUDGES first and RECORDS afterwards.** The
author's decision, drafted in private with its fix (`spec/rfc/PROCESO.md` §«Fallos de seguridad»
[security flaws]; `SECURITY.md` §3.15). The paragraph above fixes the order reserve, record and
evaluate, and with it a death of the process between recording and applying left a leaf WITHOUT
a verdict: the receipt entered the signed `recepRoot`, `zkssl_recepPath` served it, the
operation was not in the ledger and, if the holder did not resend, once the window was over the
completeness envelope gave ROJO, «NO RESUELTA EN LA VENTANA», against an honest operator.
Measured in private on `4d8017d` (§813), with an abort injected after recording and with a
SIGKILL injected by `strace`: on restarting, the register and the counter «a la par» [in step],
without a warning, and the leaf under the first signed head. What rules since §815, in the four
arms with a receipt —`applySend`, `applyClaim`, `applyMany` and `zkssl_pledge`—: the node
reserves the `rx` and fixes the era ON RECEIVING, as D-D asks («computed at reception»); the
layer JUDGES; and the leaf is recorded with `fsync` AFTER the verdict and BEFORE responding
(`juzgar_y_anotar`, in `crates/zk-ssl-node/src/main.rs`). The table above does not change: it
counts what the layer judged, and a rejection leaves a leaf. What changes is what an interrupted
evaluation leaves —a death, a panic of the layer, a failure of the store—: a GAP, which promises
nothing, and never a leaf that nobody can resolve. With `--ledger`, the leaf of an applied
operation is born after the `flush` of its entry; without it, not, because the ledger lives in
memory and the reception register on disk (D-H declares it). A failure of the store (`Store`)
leaves no leaf: the operation may or may not be in the ledger after restarting, and the node
stops.

**And decision (a) of §569 is reversed**, the assistant's and REVERSIBLE: «if recording fails,
the operation is NOT evaluated». With the new order, when recording fails the operation has
already been judged: the request receives its true verdict WITHOUT `recepcion` —the first part
of D-H: the holder knows it in their response— and the node goes to STOPPED. The reason: (a)
avoided evaluating without a leaf, which the holder sees in their own response, at the price of
leaving leaves without a verdict, which nobody sees until they accuse an honest one. A mark of a
failure in flight, written with the lock of the state taken, makes the request that was waiting
for that lock not be evaluated: at most ONE evaluation is left without a leaf per failure. The
race was measured with real threads by a review of §815, and its test goes into the seal; that
the mark is set and read with the lock taken is checked by the tests, in which setting it or
reading it without the lock panics. And before the layer judges, a trace line comes out with the
method, the `rx` and the `hashPrueba`, without the parameters: the request that brings the node
down no longer leaves its leaf, and its trail stays in the log, without evidential value. What
this order does not close goes in D-H.

⚠️ **GAP MEASURED (session 193) —DECIDED in §576, below—:** `applyMany` and `zkssl_pledge`
evaluate proofs and **do not reserve a number**. By this same rule they should; until it is
decided, what comes in through them leaves no reception leaf, and a censor that rejected there
would not be in view.

**DECIDED (§576): the receipt covers the holder's TWO direct ways, `applySend` and
`applyClaim`** —the same cut as the acknowledgement, which only they return (§274)—. A decision
delegated to the assistant (session 193) with the instruction to apply the principles and the
manifesto of the project, and REVERSIBLE. `applyMany` and `zkssl_pledge` stay OUTSIDE the
promise, and it is declared residue (D-H), because a receipt of theirs **would have no resolution
today** and the completeness envelope would accuse an honest operator: (a) a batch is applied or
rejected WHOLE, and when rejected its cause is that of ONE operation —the others have no
rejection envelope of their own—; (b) the accepted pledge records its `Consumo` with an EMPTY
proof (`consumo.rs`), so no acknowledgement carries the hash of its proof, and the pledge that
does not verify has no rejection envelope in RFC-0007. Its response does not bring `recepcion`:
whoever uses them knows it at once, as in D-H, and the direct way stays open for whoever wants the
operator bound. **A witness of the node binds it** (§576): the batch and the pledge EVALUATE
without consuming a receipt, and the direct way does; if either of the two starts to reserve, it
falls, and this is decided again with its resolution. The strong property —the receipt of the
batch, with the binding of its companions, and that of the pledge, with its resolution— is a
stage to be designed, not an oversight. **RFC-0014 proposes it (§608)**: one receipt per batch
over the fingerprint of its composition, and one per evaluated pledge, each with its resolution.
**And the witness fell, on purpose, in §611** (its E3): the batch and the evaluated pledge
reserve and record their receipt at the same point as the direct way, and the witness was
rewritten the other way round. Their resolutions are E4 of RFC-0014 (§612, §613).

### D-F — The completeness envelope, and its three verdicts

`tipo: "completitud"`, ninth family, verified by the command **without a node**. It carries the
receipt with its path under the `recepRoot` of a signed head, the closing head with its `seq`,
and —if it exists— the resolution. The verifier says one of three things:

1. **Resolved as an applied transition**: the resolution is the path of the acknowledgement's
   leaf under the `acusesRoot` of a head with `seq <= e + N`, with the same
   `hash_de_la_prueba`. VERDE.
2. **Resolved as a rejection with proof**: the resolution is an envelope `tipo: "rechazo"` of
   RFC-0007, verified by its own rules, over a head inside the window. VERDE.
3. **Not resolved in the window**: the receipt verifies, the window has expired —`S - e > N` with
   `S` the XMSS index of a signed head (§567; see the correction of D-D)— and neither of the two
   is presented. **NAMED ROJO**, and that is the
   product: a portable object that says, with the operator's own signature inside, that it
   committed to resolve and did not.

⚠️ The third verdict is a statement about what the operator **does not show**, and that is said
here in so many words: it is not a cryptographic proof of absence, because proving that something
is in none of `N` epochs would demand the `N` whole epochs. What it is —and it is enough for the
milestone— is **opposable** evidence: the promise is signed by the accused, the window is
arithmetic over two signed heads, and the burden of showing the resolution is on whoever has it.

### D-G — The crack: the rejection causes without a portable proof

The second branch of the disjunction **is not total, and it is not going to be sold as if it
were**. RFC-0007 declared causes that produce no portable proof. A receipt resolved by one of
them cannot show verdict 2, and the envelope says so by its name instead of keeping quiet: the
verifier answers «resolución declarada, no probada» [resolution declared, not proven], which is a
fourth state and is counted apart. Closing that crack is giving a portable proof to those causes,
and that belongs to RFC-0007, not here.

⚠️ **§815 (kit 0.4.3): `ProofFailed` and `VerificationFailed`, in the fourth state.** D-I of
RFC-0007 DECLARES them without a portable proof, and the kit, up to 0.4.2, only admitted four
causes as declared, without them: a receipt that the node rejected for a proof that did not
verify had no resolution —the rejection envelope does not prove it and the declared one came out
ROJO—, and once the window was over it named the honest operator. Measured in private: a third
party with no account sending garbage bytes with the public roots is enough (`SECURITY.md`
§3.15). Since 0.4.3 they come out with exit 3, and they are still unproven: proving them is the
kit re-verifying the proof against what the leaf signs, and that belongs to the `zkssl/0.5`
train (RFC-0019 E3). And what this section did not say is said here: the declared one is the
operator's word, and nothing more. The kit gives 3 to a declared one with any cause of the list,
relevant or not, and without a current head; the only thing that binds it to the receipt is the
`hashPrueba`, which the operator knows (measured in private on `4d8017d`, with a
`NotTheAccountHolder` declared over a send). That is why adding the two opens no door: any ROJO
already dropped to 3 by declaring one of the four. It stays open in entry 129 of `BACKLOG.md`.

### D-H — The residue, declared

What this RFC **does not close**: the operation for which the node **never signed a receipt**.
If it does not answer, or answers without a receipt, there is no object to oppose. The holder
knows it at once —their request brings no receipt— and can retry and publish the absence, but
**cannot prove it**.

⚠️ **CORRECTION (§815), cited and not erased.** «If it does not answer […] there is no object to
oppose» was not true until §815. Measured in private on `4d8017d`: the node that died between
recording and applying did not answer —the holder received a 502 from the measurement's proxy—
and its leaf entered all the same the `recepRoot` of the next signed head. The holder had,
without knowing it, a receipt that the oracle of `zkssl_recepPath` and the kit returned to them
(33 calls, 1 hit), and that once the window was over gave ROJO against an honest operator. Since
§815 (D-E, amended) the leaf is born after the verdict. If the node dies before recording, the
`rx` stays a gap and there is no object to oppose, as this section says. If it dies after
recording and before responding, the leaf carries its verdict: with `--ledger`, an applied
operation is in the ledger and its acknowledgement resolves it (VERDE, measured with the node of
§815); a rejection leaves the leaf of a rejection that nobody has. That is residue, prior to
§815 and that §815 does not close, of the same class as the holder who keeps quiet about a live
rejection: the node does not keep the `data` of a rejection, and without it the honest operator
has nothing to show for a cause with a proof (DEDUCED; entry 129 of `BACKLOG.md`). «Answers
without a receipt» includes, since §815, the verdict that comes out when the register fails after
judging: there is no leaf, and the node stops. Whoever is left without a response looks for their
`hashPrueba` among the `proofDigest` of `zkssl_logEntries` before paying again. And there remains
a residue that §815 neither closes nor worsens, named by two of the reviews that judged the order
(A8): **a node that signs without `--ledger`**. `--diario` together with `--clave` (§285) is
enough, and then the ledger lives in memory and its `commit` writes nothing, while the counter
and the reception register always go to disk. After restarting there remain the leaves of the
applied operations under a signed `recepRoot`, without their operation in the ledger: the state
of F1, and the same ROJO when the window expires. It is DEDUCED from reading the code, without
measuring, and prior to §815; whoever signs receipts should persist the ledger, and demanding it
at start-up stays outside §815 (`SECURITY.md` §3.15).

**Measured in §601** (RFC-0011, E3: `tools/banco_mentiroso_censura.sh`): a proxy of the operator,
in front of an honest node, that does not answer or answers without a receipt. The holder knows it
at once; the signed log does not move, because the node did not see it; and all the defences
stay silent -two witnesses, one through the proxy and another direct, `--auditar`, `--comparar`
and `--ausentes`-. With the proxy forwarding they say exactly the same: for them, the censored
operation is indistinguishable from the one that was never sent.

That is the residue of the milestone, and it is irreducible in this model: an operator that does
not respond is indistinguishable from a network that is down. What the receipt changes is the
ground: whoever answers is bound, and total silence is a visible conduct sustained in time, not a
silent discard among thousands of operations served.

**And the residue has a second part, which IS a matter of DESIGN and not of the model (D-E,
§576):** what comes in through `applyMany` or through `zkssl_pledge` carries no receipt by
construction. Whoever sends it knows it just as soon —the response does not bring `recepcion`—,
and in the deployment with aggregators (`SECURITY.md` 2.ter) that is EVERYTHING aggregated:
whoever wants the operator bound sends through the direct way. Unlike the first part, this one
can be closed, with a stage that designs how a receipt of the batch and one of the pledge are
resolved. RFC-0014 designed it; since §611 both carry a receipt, and since §613 the completeness
envelope resolves both.

## What was DISCARDED on measuring

- **A single tree with a class bit.** It saves a root and a pair, and in exchange the number of
  leaves of an era stops meaning one thing only. Clarity rules over the byte.
- **Signing one receipt per operation.** Decided against in §121 by arithmetic, and the
  measurement confirms it: each signature burns an XMSS index, and the heartbeat already burns
  1,440 a day. One receipt per operation would make the signature the bottleneck of the system.
  The receipt inherits the head's signature by being under its root: zero new indices.
- **The RPC serving the reception root.** It is §248 again: the holder **recomposes** it and
  compares it with the head they keep. A root served by the accused proves nothing.
- **Binding the era to the head that ends up containing the receipt.** It records the
  publication, which is already derivable, and leaves the holder waiting for the operator to
  decide. It is the reasoning of §274, applied to the new object.
- **Counting what reaches the port.** It would open gaps in someone else's register by sending
  garbage.

## Compatibility

The wire **does not go up**: it stays at `zkssl/0.4`. The head moves to **v6** with an additive
pair, and there is precedent measured twice —the v4 of RFC-0006 (§414) and the v5 of RFC-0007
(§451), both with the wire still—. What is signed grows only by version, and what is not signed
does not exist for the core: v6 enters `NUCLEO.md` with its exact composition and its known
vector in `spec/vectors/nucleo/`, and the vectors of the new era are born in `spec/vectors/cable/`
as those of v5 were born in §453. The old vectors **are not rewritten**.

The method to read the path is **additive**, as `zkssl_applyMany` was in §222: adding a method
does not raise the version because the conformance vectors do not move. It does move the
published counts of methods, and that is paid by the stage that brings it, not this one.

**The API principle is preserved**: the spending key does not travel. The receipt carries the
hash of a proof, two numbers and nothing more; neither the node nor the envelope sees it at any
moment.

### Why it enters by RFC

It touches the signed head, which is what a verifier recomposes, and it adds a family of vectors
and an object to the evidence package. It is exactly what the PROCESO reserves for an RFC.

## Security

- **What the reception root reveals.** The number of leaves of an era says **how many operations
  the node evaluated**, including those it rejected. Today the `acusesRoot` already publishes how
  many it applied; this adds the volume of what was rejected. It is a leak of volume, not of
  content, and it is declared.
- **What the receipt reveals.** The `hash_de_la_prueba` and an order number. The order is just
  what makes the reordering detectable: it is the property, not a side effect.
- **What is still not closed.** The operator can issue no receipt (D-H). It can order. And
  an unsigned head binds nobody: this whole promise hangs on the node signing its heads, which
  is the same root of trust that the rest of the system declares.
- **The retention of the register of receipts, undecided (§580).** `podar` exists, tested, and
  has NO caller by the author's decision: pruning when the window expires would leave without an
  envelope the holder who did not ask for their path in time, because the NOT RESOLVED arrives
  after the window and needs that path. It is policy with victims, like the deadline `N`; until
  there is a real operator, the register grows without a cap, 40 bytes per receipt.
- **The restart.** §242 declared index gaps benign on restart; the reception counter, on the
  other hand, **does not restart** and its file has its own guardian. That it was benign in one
  place and catastrophic in the other is measured and written since §253: two limits that are
  declared separately and together would break the property.

## References

`spec/PAQUETE.md` (the forms of the envelope), `spec/NUCLEO.md` (the compositions of the head
and the registry of domains), `spec/RPC.md` (the reception counter and what it is not), RFC-0004
(the evidence package), RFC-0006 (the mould of a new pair in the head), RFC-0007 (the rejection
with proof), `doc/CONFIANZA_RESIDUAL.md` §2.1 (the signed reception receipt as portable evidence
of censorship) and the entries of the header.
