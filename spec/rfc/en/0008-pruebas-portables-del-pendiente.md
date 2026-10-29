# RFC-0008 — The two portable proofs of the pending item: the claim, the payment in flight and the pledge

> **Español:**
> [`spec/rfc/0008-pruebas-portables-del-pendiente.md`](../0008-pruebas-portables-del-pendiente.md)
> is the original of this page, in Spanish; it prevails wherever the two differ. This translation is
> of commit `7766efb` (audit entry §917) and was produced by the project's assistant, a Claude Code
> session, in audit entry §918 (`GENAI.md`): it is a generated artefact, not text written by the
> author. Line citations of the form `file:N` refer to the Spanish files.

- **Status:** ACCEPTED — **rule 4 of the PROCESO, settled with measurement** (§483 the adoption; E1
  §484–§495, E2 §501–§509 and E4 §496–§499 with §508 and §509, the stages with their gates; E3
  §513–§520 the design and the code, §541–§545 the entry point, the SDK and the bench, and §546 the
  catalogue, the eighth family of the artefact and of the canon; §547 this turn). The spec is up to
  date (`PAQUETE.md` 2.8, 2.9, 2.10 and section 9), the vectors are new under `zkssl/0.4`
  —`pendiente/`, `pago/` and `prenda/`, nine each— and the suites are green: the canon runs them
  through the harness, and also from inside the tarball. The OpenRPC is not regenerated because no
  method has moved since §540, and the wire test ties it to the table since §259. It is kept as a
  record of what was decided, what was measured and what was discarded.
- **Author:** Ángel José Toranzo Portela
- **GenAI assistance:** Claude (sessions 144, 145, 147, 149, 150, 151, 152, 153, 154 and 155; and
  the E3 arc: 158 (§513), 159 (§514 and §515), 160 (§516–§518 with their `-B`), 161 (§519 and
  §519-B), 162 (§520 and §520-B), 179 (§541–§543), 180 (§544 and §545) and 181 (§546 and §547)) —
  see [`GENAI.md`](../../../GENAI.md)
- **Date:** 2026-09-16
- **Protocol version affected:** `zkssl/0.3` — **does not go up** (see Compatibility). The two new
  methods are additive; the v5 head does not change its form; the pledge mark is one more leaf of
  the consumption tree, whose root the head already signs since RFC-0006.
- **AUDITORIA entry(ies):** §121, §178, §211, §275 (the acknowledgement and its root), §343–§345
  (the v2 commitment of RFC-0003), §387 and §388 (the roots at rest), §412–§441 (the published
  consumption), §451–§453 (the v5 head), §458–§460 (the rejection by paths), §463–§467 (the age
  proof), §473–§479 (the band over the committed leaf); §483, which adopts it; §484, which decides
  D-F and D-G and corrects row E1; §485, the instrument of E1; §486, which decides D-H; §489, which
  decides D-I and corrects D-B and Security; §494, which decides D-J, D-K, D-L and D-M and corrects
  row E4; §496, which decides D-N, D-O, D-P and D-Q and splits row E4 by sides; §497 and §497-B, the
  claimant's entry point; §498, which decides D-R..D-W and brings the bench; §499, which decides
  D-X..D-AC and brings the catalogue; §500, which writes those twelve here and puts in its cell what
  was sealed of E1 and of the claim side of E4; and §501, which decides D-AD..D-AJ and opens E2;
  §502 and §503 (the instrument and the AIR of the payment), §504 (the producer), §505 (the payer's
  gate), §506 (the arm of the command and form 2.9), §507 (the `prueba-pago` entry point), §508 (the
  bench and the payer's credential) and §509 (the catalogue); and §510, which writes D-AK..D-AR
  here, seals the E2 cell, closes the E4 one on both sides and notes the bound. And the E3 arc:
  §513, which separates the mark from the pledge; §514 and §515, which decide D-AV..D-AY; §516 and
  §517, the AIR and the prover; §518, the producer; §519, the `zkssl_pledge` wire; §520, the eighth
  arm of the command; §521 and §538, which stop it and unblock it by 5.A-360; §541, which writes
  D-BC..D-BI here; §542 and §543, the SDK and the entry point; §544, the fifth file of the seeding;
  §545, the bench; §546, the catalogue; and §547, this turn.
- **Milestone:** H5 of the proposal sent to NLnet Restack (140 h), in its words: *«The two portable
  proofs of a pending item. Payee side and payer side, derived from the same head; pledge
  transition; format and vectors.»*

## Status of the stages

| stage | what it delivers | does it break the wire? | status |
|---|---|---|---|
| E1 — the pending claim, portable | the claimant's circuit: under the `pendingRoot` of an existing signed v5 head there exists `C2 = M(C1, X)` with `C1 = H(H(receptor, sal), importe)`, in the name of `receptor` (D-G), `importe >= inferior` (band, the mould of `InsufficientBalance`), with the path INSIDE the circuit; its meta `(emisor, nacido)` by path under `pmetaRoot`, with the same bits. `zkssl_pendingPath`, additive, serves the two paths of the last heartbeat's snapshot to whoever presents a notice that recomposes the leaf (D-F). Over `tipo: "cobro_pendiente"` in `PAQUETE.md`, verified by the command without a node | NO | **sealed** — §484 (D-F and D-G, and the corrected row), §485 and §485-B (the instrument), §486 (D-H), §489 (D-I), §490 and §490-B (the AIR of two lanes and one bit), §491 and §491-B (the producer, a free function), §492 and §492-B, §493 and §493-B (the heartbeat snapshot in the layer and in the node, and `zkssl_pendingPath`), §494 (D-J..D-M) and §495 and §495-B (the link with the head, the arm of the command and `PAQUETE.md` 2.8). **E1 is whole** |
| E2 — the payment in flight, portable | the mirror, for the payer: `C2` opens to EXACT `(receptor, importe)`, `nacido` by path, and `nacido + delta >= T` with `delta` and `refund_id` as witness (they are not revealed). Over `tipo: "pago_en_curso"`, verified without a node. Together with E1's, a third party foreign to both verifies a disputed payment without anyone's ledger | NO | **sealed** — §501 (D-AD..D-AJ), §502 and §502-B (the instrument), §503 and §503-B (the AIR of two lanes with the composed envelope inside, and its judge), §504 and §504-B (the producer in the layer, a free function), §505 and §505-B (the payer's gate in `zkssl_pendingPath`, D-AE), §506 and §506-B (the seventh arm of the command and form 2.9), §507 and §507-B (the `prueba-pago` entry point, D-AK..D-AO), §508 (the bench and the payer's credential, D-AP and D-AQ) and §509 (the catalogue, D-AR). **E2 is whole** |
| E3 — the pledge, as a transition with proof | the receiver marks the pending item as pledged: a MARK with its own domain over `C2` in the consumption tree, published by an additive method, `zkssl_pledge`, which REQUIRES the PLEDGE envelope: an AIR of its own (D-AV) that publishes the MARK with `C2` as witness (D-AW), without the band and without the meta, in 42 columns and 512 rows (D-AX, D-AY); a second MARK is `ConsumoRepetido`, which already has a rejection envelope with proof (RFC-0007 E3, `PAQUETE.md` 2.6). The PLEDGE is the pair mark + envelope (D-AS), and it touches neither the claim nor the refund: what binds is contract, and it is declared | NO | **sealed** — §513 (the mark and the pledge are two objects), §514 (D-AV, the AIR of its own) and §515 (D-AW, D-AX and D-AY: the mark published, the key where the band was, no meta), §516 and §516-B (the AIR with its judge), §517 and §517-B (the prover and its first real proof), §518 and §518-B (the producer in the layer), §519 and §519-B (the `zkssl_pledge` wire, which verifies before writing) and §520 and §520-B (the eighth arm of the command); STOPPED between §521 and §538 by 5.A-360 —the proof published the pledgor's spending key 42 times— and reopened when RFC-0009 E3b-2 hid it; §541 (D-BC..D-BI, the entry point), §542 (`Wallet::prueba_de_prenda`), §543 (the `zk-ssl-cli prueba-prenda` entry point), §544 (the receiver's keystore, fifth file of the seeding), §545 (the bench `tools/banco_prenda.sh`) and §546 (the catalogue `spec/vectors/prenda/`, eighth family). **E3 is whole**, and it is HALF a pledge said out loud: the pair with the mark under the `consRoot` is closed by `zkssl_consumoPath` (D-AS) |
| E4 — the catalogue and the bench, by sides | `spec/vectors/pendiente/`: two positives per side, GATHERED from the captures of a real node (mould: `edad/`), and one negative per producible rule; `MANIFIESTO.txt`; the family in `FAMILIAS`; the bench that reproduces it live; section 9 of `PAQUETE.md`. It goes BY SIDES in a single row (D-O): the claim's first —its two forms (D-N), the cli entry point (D-P) and its negatives (D-Q)— and the payment's with E2; this cell names what was sealed on each side. The turn to ACCEPTED requires rule 4 measured letter by letter, like §481 | NO | **sealed on the claim side** — §496 (D-N..D-Q), §497 and §497-B (the claimant's entry point in the cli, `prueba-cobro`, and `simulate --v2`), §498 (D-R..D-W and the bench, `tools/banco_pendiente.sh`) and §499 (D-X..D-AC and the catalogue, `spec/vectors/pendiente/`, sixth family of the artefact and of the canon). **And the PAYMENT side**: §508 (its own bench, `tools/banco_pago.sh`, with its four-file seeding, D-AP and D-AQ) and §509 (the catalogue `spec/vectors/pago/`, seventh family of the artefact and of the canon, D-AR, which reverts D-AC and the letter of D-AJ). **Both sides are in; the stage is whole**, and rule 4, measured letter by letter, was settled by §546 with the eighth family: §547 turns the document |

The measurements of this document were taken on `5ef3b1b` (`TERRENO-H5-144`); those of D-F and D-G,
on `393032e` (`TERRENO-E1-145`); and those of D-H, with the instrument of §485, which ran in a copy
outside the tree, and reading `0424439`; those of D-I, reading `be90eb7` (`TERRENO-AIR-E1-147`);
those of D-J to D-M, reading `7bb3942` (`TERRENO-COBRO-149`); those of D-N to D-Q, reading `9512915`
(`TERRENO-E4-150`); those of D-R to D-W, reading `8d064c6` (`TERRENO-B-150`); those of D-X to D-AC,
reading `f1e0401` (`TERRENO-C-150`); and those of D-AD to D-AJ, reading `477dcab` (`TERRENO-E2-151`,
with its falsifier measured in a copy by the PASTE-E2-PRE). None wrote a byte in the tree (see
References).

**Corrections of §484** (row E1 and D-B, as §483 left them). The row said «`receptor` is the public
identity of whoever proves», and the mould it names does not restrict who proves (D-G). It said
«`importe >= X`» with `X` also as the reversal envelope: one name for two things; the bound is now
called `inferior`, like the `lower` of `banda.rs`, and `X` is left for the envelope. It gave
`zkssl_frozenPath` as the mould of the method, and that mould serves the state of now (D-F). And
this paragraph said that all the measurements were taken on `5ef3b1b`.

**Corrections of §489** (D-B and Security, as §483 and §484 left them). D-B said «the envelope `X`
can be public because it is a commitment»: `X` is a commitment without randomness and hides nothing
from whoever guesses `refund_id` (D-I). Security said, without condition, that the claimant's
envelope does not say when it expires nor who paid; with `X` inside, that only held if `refund_id`
was not guessed. With D-I it holds without condition, because `X` does not travel.

**Correction of §494** (row E4, as §483 left it). It said that E4 delivers `PAQUETE.md` 2.8, and row
E1 already put the claim envelope in that document: the envelope's section is born in E1 with its
judge (D-L), and E4 is left with section 9, the one of the vectors.

**Corrections of §500** (the status cells of E1 and E4, as §483 and §496 left them, and the count of
the Design). The E1 cell said «proposed» with the whole row since §495 —and D-O already said so—; it
now names its seals. The E4 one said «proposed» when its own text asked that it name what was sealed
on each side; it names the claim's and leaves the payment's for E2. The status of the document does
not move: PROPOSED belongs to the process (`PROCESO.md`, rule 4) and that of each stage lives in its
cell, as in RFC-0005; the turn to ACCEPTED requires what row E4 says. And the Design said «seventeen
decisions»: the twelve of session 150 —D-R..D-W of the bench and D-X..D-AC of the catalogue— lived
only in entries 498 and 499, and they enter here with their reversal.

**Correction of §521** (D-B, D-I, the correction of §489 and Security). They said that `X` does not
travel and that the claimant's envelope does not carry it: it does not carry it as a FIELD, but the
proof opens its rows in the clear and publishes it, with the salt and the exact amount; the
payment's publishes the salt, the `delta` and the `refund_id`; and the pledge's, the pledgor's
spending key, 42 times. Measured in sessions 162 and 163: winterfell 0.13 does not hide the witness.
The decisions were taken with the data of their date and do not change here; what changes is what
they can promise, and that is decided by an RFC on what is promised.

**Correction of §523** (D-AW). It said that the house pattern —`prove` in the client, `apply`
verifies without seeing the key— is met to the letter because the pledge envelope carries no key: it
does not carry it as a FIELD, but the proof publishes it (the correction above), and `apply` sees it
when verifying it. The rule that explains it was measured in session 163: every value that goes in a
constant column of the trace comes out literally. As above, the decision does not change here.

**Correction of §538** (to the two previous ones). Since §538 (RFC-0009 E3b-2, `zkssl/0.4`) the
prover hides the witness in what the E2 suite measures: the claim proof does not publish literally
the salt, the `X` nor the exact amount; the payment's, neither the salt, nor the `delta` nor the
`refund_id`; the pledge's, not the pledgor's key. What each envelope carries as a FIELD does not
change, and neither do the decisions; what is promised of the proofs is fixed by RFC-0009 (D-A
reverted), and what is not promised —that nothing be deduced from what is opened— too.

### The frontier with H5b, and what belongs to each

This RFC speaks of a pending item that IS in the tree: it exists, it has an amount, it has an age,
and it can be pledged once. H5b speaks of what came in through the wire and of whether it ended up
applied or rejected. A payment the operator did not apply has no leaf, has no `nacido` and has no
proof here: that is completeness, and it is still H5b. Nor is the pledge completeness: what is
unique is the PROOF —a single PROVEN pledge per leaf (D-AU)—, and the MARK is the label that
publishes it, with the same single use that RFC-0006 gives to any label.

## Motivation

Milestone H5 promises that the pending item stops being the payer's promise and becomes an asset of
the claimant, with proof and with single use. What was measured on `5ef3b1b`, and why each piece
dictates the form of this RFC:

- **The pending item is already committed in an openable form.** `C1 = H(H(receptor, sal), importe)`
  (`crates/zk-ssl/src/pending.rs:70`), the reversal envelope `X = M(refund_id, delta)` (`:88`) and
  the leaf `C2 = M(C1, X)` (`:108`), with the position SALTED (B13/B14,
  `crates/zk-ssl/src/migration.rs`). The circuits that open `C1` and `X` exist:
  `crates/stark-experiment/src/circuit_claim_v2.rs` and `circuit_refund_v2.rs`.
- **The notice to the receiver is opaque on purpose.** `PendingNotice`
  (`crates/zk-ssl/src/two_phase.rs:62`) carries `position`, `salt`, `amount` and
  `x: Option<Digest>`; the receiver reconstructs `C2` without learning `f` nor `delta` (D-1 of
  RFC-0003), and `refund_id` travels committed and is opened only on refund (D-2). The node does not
  keep the opening: of the pending item there persist `pend:`, `pamt:` and `pmeta:`
  (`crates/zk-ssl/src/persistence.rs`, and the §247 correction of §464 in RFC-0007).
- **The claimant's T cannot be produced with today's notice.** `expiry = nacido + delta`; the
  receiver has `nacido` by path under `pmetaRoot` (`meta_pendiente_hoja(emisor, nacido)`,
  `two_phase.rs:238-241`) but `delta` lives inside `X`, opaque to him. Only the payer knows it. It
  is the finding that splits the two halves: the T belongs to the payer.
- **The v5 head already signs what the two proofs need**: `pendingRoot`, `pmetaRoot`, `next_pending`
  and `params_digest` with the refund `ttl` (RFC-0007 E1, §451–§453). No new family under the
  signature: the signature's format stays at 5.
- **The reversal rule already exists and is committed**: `RefundTooEarly` if `ahora - nacido < ttl`
  (v1) or `< delta` (v2) (`two_phase.rs:437-452, 660-674`). «Not reversible before T» is not a new
  rule: it is that one, read from the other side.
- **«At least `inferior`» is circuit, and its mould exists.** `crates/zk-ssl-air/src/banda.rs` opens
  a leaf under a root with the path INSIDE and proves a band over a field; the operator produces it
  without the key. Here the leaf is `C2`, the opening is two compositions, and the band is over
  `importe`. Revealing the amount in the claimant's envelope would contradict the promise.
- **The single-use set that the PDF called «the first measurement» already exists.** There is no set
  of spent items —the claim retires the leaf—, but RFC-0006 left a tree of public labels with a
  signed root (`root:cons`), where repeating is `ConsumoRepetido`
  (`crates/zk-ssl/src/consumo.rs:129-139`), and that cause is already in the catalogue of rejections
  with proof (`spec/PAQUETE.md`, 2.6). Publishing today requires no proof (`spec/RPC.md:940`):
  whoever publishes first blocks, and that is denial, not double use (D-4 of 0006). What anyone can
  put over someone else's pending item is the MARK, and the mark alone proves nothing (D-AS): it is
  not denial, it is one more label. What requires proof is the PLEDGE, and that is why it enters
  through a method of its own that asks for the claimant's envelope with its ownership (D-AV).
- **The portable envelopes have a form and a judge**: `PAQUETE.md` 2.4 (consumption), 2.6
  (rejection) and 2.7 (age); the command verifies them without a node and the artefact carries them
  inside with their catalogue (`tools/artefacto.sh`, `FAMILIAS`). Two new types follow that mould
  without touching the old ones.

## Design

The sixty-one decisions were taken by the assistant by delegation of the author (D-A..D-E in session
144; D-F, D-G and D-H in 145; D-I in 147; D-J..D-M in 149; D-N..D-AC in 150, and of them D-R..D-AC
were written here by §500; D-AD..D-AJ in 151; D-AK..D-AR in 155, and §510 wrote them here;
D-AS..D-AU in 158, which §513 wrote; D-AV..D-AY in 159; D-AZ..D-BB in 161, which §519-B wrote; and
D-BC..D-BI in 162 —D-BC was chosen by the author among three, and D-BF was already applied by §520—,
ratified in 179 and written here by §541), with the decision constitution (purity, clarity,
coherence, faithful image, in that order). All carry their reversal condition, written here.

### D-A — The T belongs to the payer; the claimant says «in my name, at least `inferior`, born at b»

The notice does not carry `delta`, and it must not carry it open: D-1 and D-2 of RFC-0003 exist so
that the receiver learns neither the return choices nor the identity of the return key. Three paths
were measured: (a) that the claimant's half says what its owner knows —it exists, amount
`>= inferior`, `nacido = b`— and the T is carried by the payer's half, which does know everything;
(b) that the notice gains `delta` outside the wire and the receiver opens `X` in circuit, which
requires `refund_id` as witness and breaks D-2; (c) a v3 commitment that separates `delta` from
`refund_id`, which is a format break and a new name. (a) wins: purity (it does not touch 0003),
clarity (each half says what its owner knows and nothing more), faithful image (the spec will say
that expiry is proven by the payer). The milestone's promise is met with the two halves together,
which is exactly how the PDF's text formulates it. **Reversible** if a measured use case requires
the T on the claimant's side: then (c) is opened, never (b).

### D-B — «At least `inferior`» is a band in circuit, with the path inside

The mould is the band of `InsufficientBalance` (RFC-0007 E5): the leaf opened inside the AIR, the
path as witness, two public bounds. Here the bounds are `inferior` and `superior` (the amount cap),
the opening is `C1` and then `C2 = M(C1, X)`, and the envelope `X` is witness (D-I): it hides
nothing from whoever guesses `refund_id`. The position does not leave the circuit: it is what the
salted positions promise. The cost is measured before writing the AIR with the mould of the
instrument of E4a of RFC-0007 (`crates/zk-ssl/src/instrumento_edad.rs`), and if it does not fit
under a heartbeat it is declared with a figure. **Reversible** only towards a narrower band; never
towards revealing the amount.

What is already measured on `393032e` (`TERRENO-E1-145`). The two trees have 32 levels
(`TREE_DEPTH`) and the meta leaf lives at the SAME position as the commitment: the two paths share
the direction bits and NOT the siblings, and their first negative witness is a meta of another
position. The commitment chain is 35 cycles of 8 rows, like the band's; the meta adds 33 (one
permutation with domain and the ascent). In one lane the trace goes from 512 to 1024 rows; in two
lanes with the shared bit it stays at 512. The bounds measured with the house options (the band,
`circuit_audit`, the v1 claim) leave it in milliseconds: what the instrument decides is the
geometry, not whether it fits. The geometry is decided by D-H.

### D-C — The pledge lives in the consumption tree, with its own domain and with proof

Two places were measured: (a) a label `H(dominio_de_prenda, C2)` in the consumption tree, published
by a new method that requires the proof of opening of the claim before writing it; (b) a `prendado`
field in the meta leaf, which changes `meta_pendiente_hoja`, with it `pmetaRoot`, and with it the
FORMAT version of the signature and the age vectors that open that leaf. (a) wins: one primitive per
property —single use is consumption—, the rejection with proof of the second attempt already exists
(`ConsumoRepetido`), the head does not change and no live vector expires. The domain is registered
in `zk-ssl-hash` like the others and `check_dominios` takes its census. What changes with respect to
0006 is not the tree but who can write THAT class of label: the free publication of 0006 continues
for the consumption labels; the pledge's requires proof. **Reversible** towards (b) if the
measurement of E3 showed that the consumption tree cannot distinguish classes without breaking its
rule. ⚠️ **The measurement was made and the tree does NOT distinguish them**: (a) holds, but the
name splits into MARK and PLEDGE. See D-AS, which amends this decision without replacing it, and
D-AV, which decides which is that «proof of opening of the claim» this paragraph names: not the E1
envelope, but one with the key cycle.

### D-D — The receiver pledges, and the pledge binds only the pledge

Authorization is knowledge of a preimage or proof: the payer also has the opening of `C1`, so
«knowing the opening» does not distinguish. What only the receiver has is what `circuit_claim_v2`
already requires in order to claim: its proven identity. The pledge is that authorization without
the credit. And the pledge does NOT touch the claim nor the refund: a pledged pending item is
claimed the same and refunded the same after T. What the pledge guarantees is that there is no
second PROVEN pledge (D-AU), with proof of the rejection; what binds the claimant with his financier
is contract, and the system produces the condemning pair, it does not adjudicate (§121.6). A rule
that blocked the refund would be a new rule of the layer over the payer's rights: fewer exceptions.
**Reversible** towards «the pledge blocks the refund until T» only with its negative witness written
beforehand and with the rule committed in the head, because a rule that is not committed does not
sustain a proof.

### D-E — Two envelope types, one per side

`cobro_pendiente` and `pago_en_curso`, as `consumo` and `conflicto` are two: the command dispatches
by `tipo`, not by an inner field, and each type carries its list of rejections and its vectors. Both
carry the signed v5 head whole, like the age envelope. **Reversible** towards one type with `lado`
only if the catalogue showed that the two rejection lists are identical, which today they are not
(the payer's has the T and the `refund_id`; the claimant's has the band).

### D-F — The path and the head, from the same state: the heartbeat snapshot

Measured on `393032e`: `zkssl_signedEpochHead` serves the head of the LAST HEARTBEAT
(`crates/zk-ssl-node/src/main.rs:1505`), and the path the layer knows how to give is that of the
state of NOW (`claim_materials`, `crates/zk-ssl/src/client.rs:163`). The pending tree is written by
four production functions (`apply_deissue`, `apply_refund`, `commit_send` and `commit_claim`, in
`two_phase.rs`), so any payment between two heartbeats moves `pendingRoot`, and a path of now stops
climbing to a signed head. The mould that row E1 named, `zkssl_frozenPath`, holds because its tree
only changes with another freeze (`spec/RPC.md`, «El camino de congelados»); copied here, it does
not hold.

Four paths were measured: (i) the heartbeat keeps, in the same critical section in which it composes
the head (`crates/zk-ssl-node/src/latido.rs`, «TODO bajo el MISMO candado»), a snapshot («foto») of
the two trees, and the method serves the path of that snapshot with the `seq` of the signed head;
(ii) the path of now with its `s`, and the client waits for a head of that `seq`; (iii) rebuilding
the tree at the head's `seq` from the log; (iv) signing a head on request. (i) wins: coherence (the
same notion of head for the path and for the signature) and fail-closed (without a heartbeat,
without `--clave` or with a pending item born after the last heartbeat, the method says so, in the
form of §241). (ii) fails closed, but on a node with traffic it may never converge. (iii) is not
possible: a log entry carries neither the position nor the leaf (`crates/zk-ssl/src/log.rs`,
`LogEntry`). (iv) burns XMSS indices on demand. The snapshot is a cache in memory: after a restart
there is no path until the first heartbeat. **Declared price**: cloning two trees inside the lock,
in time and in memory; the E1 instrument measures it before the method, with the mould of
measurement M.1 of §252 (writes in series with and without heartbeat, looking at the maximum).
**Reversible** towards (ii) if that price does not fit.

The snapshot forces a rule of the method. Serving `(emisor, nacido)` of any position to any account
with a credential would publish who paid and when. The node recomposes the leaf with what the notice
brings and the credential's `public_id`, `C2 = M(H(H(public_id, salt), amount), x)`, and compares it
with the leaf of that position in the snapshot; if it does not match, it refuses without saying what
there is. It is the lesson of §261 (a notice that does not authenticate) applied before writing the
method. E1 belongs to the v2 commitment: a notice without `x` has no `C2`, and the method refuses
it.

### D-G — E1 is a statement of STATE, without ownership

Measured on `393032e`: the mould D-B names, `crates/zk-ssl-air/src/banda.rs`, does not carry the key
cycle; it ties the identity with four assertions against the public input declared by whoever
verifies (its D-2b). With that mould, row E1 asserted something the circuit does not restrict:
whoever knows the opening —the payer knows it— produces the same proof. Two paths: (a) the statement
is one of state, «under this head there exists a pending item in the name of `receptor`», and the
RFC says so; (b) the key cycle of `circuit_claim_v2` (`CYC_PK`: one cycle and four more columns),
and only the receiver produces it. (a) wins: purity (one primitive per property; authorization
belongs to E3, where the pledge requires it by D-D), fewer columns, and faithful image (what the
circuit does not restrict is not asserted). And (b) does not buy what it seems: a proof is forwarded
the same with a key or without it, and tying it to whoever PRESENTS it asks for a challenge inside
the statement, which neither of the two carries. **Reversible** towards (b) only with that challenge
and with a measured use case that asks for it.

### D-H — The claimant's geometry: two lanes and a single bit

Measured with the E1 instrument (§485, `crates/zk-ssl/src/instrumento_cobro.rs`): the commitment
chain is 35 cycles of 8 rows and the meta's, 33; in one lane they add up to 544 rows and ask for a
trace of 1024, and in two lanes they fit in 512. The house proxy ascent (13 columns, with
`crate::proof_options()`) proved 512 rows in 0,16 s and 43.700 B, and 1024 rows in 0,15 s and 49.631
B: at this size, doubling the rows raises the bytes by 13,6 % and does not move the time (a single
run, without pairing). A second lane adds one hash state, 12 columns; the difference measured
between `circuit_audit` (31 columns, 51.449 B) and the band (27 columns, 49.051 B) gives about 600 B
per column, so the width costs the same order (reasoned). The cost does not decide.

Two paths: (a) two lanes placed with ONE bit column, the mould of the v2 claim
(`crates/stark-experiment/src/circuit_claim_v2.rs`, its two lanes with the same `bit`) without its
equality of siblings, because here they are two distinct trees; (b) one lane, with the position
accumulated in the first ascent, recomposed in the second and an assertion that equates them. (a)
wins: purity (that the commitment and its meta are at the SAME position is given by the structure,
not by an equality one has to remember to write), coherence (the mould exists) and clarity (one
position, one column). The house already climbs two trees by the same position in one lane -the
ascent of frozen ones, after that of accounts- and the bit column of the second is not tied to that
of the first: read in the five circuits that do it and MEASURED in §487 (the path of another
position verifies in all five; the layer closes it when applying and fix B will return it to the
AIR). The lesson of §487 reinforces (a): the position is tied by the SHARED BIT in a single column,
which is just what `COL_FBIT` lacks.

The form: lane A composes `C1` in cycles 0 and 1 and `C2 = M(C1, X)` in 2, and climbs from 3 to 34;
lane B leaves cycles 0 and 1 free, composes the meta leaf in 2 and climbs from 3 to 34 with the same
bit. The first negative witness of the AIR is the instrument's: a meta of another position does not
verify. **Reversible** towards (b) if the E1 AIR, measured at its seal with the house options,
exceeds 65.313 B, which is the v1 claim (1024 rows by 55 columns) and the upper bound reasoned with
here.

> **Note of §512 (S247: it is cited and not erased).** Fix B (5.A-272) moved that bound: the v1
> claim goes to **66.692 B** and its circuit, to **57 columns**. The decision was taken with the
> data of its date and **does not change**; what changes is the number it is measured against.

### D-I — The envelope `X` is the claimant's witness, not a public input

Measured on `be90eb7` (`TERRENO-AIR-E1-147`): `X = M(refund_id, delta)`
(`crates/zk-ssl/src/pending.rs:88`) is a commitment WITHOUT randomness, and `refund_id` is chosen by
the sender (`crates/zk-ssl/src/client.rs:199`). The three producers of v2 sends in the tree use the
`public_id` of the paying account: the conformance scenario, with `delta = 96`
(`crates/zk-ssl-cli/src/conformance.rs:78-80`), and two tests of the layer
(`crates/zk-ssl/src/two_phase.rs:3429` and `:3473`). With `X` in the envelope, whoever has a list of
candidate identities tries `M(pid, d)` for the plausible `d` and learns who paid and when it
expires, the two things Security says the envelope keeps quiet; and two envelopes of the same sender
with the same `delta` carry the same `X`, linkable without enumerating anything. It is deterministic
and it is reasoned; its falsifier (two sends with the same pair give the same `x`) goes with the
AIR.

Two paths: (a) `X` is witness: the AIR reads it from four columns in the link of cycle 2, the
statement says «there exists an envelope `X` such that `M(C1, X)` is under `pendingRoot`», and the
claimant's envelope does not carry it; (b) `X` public, and the RFC declares that its secrecy depends
on the entropy of `refund_id`. (a) wins: privacy is a property of the protocol, not a policy of the
sender; what is not hidden is not asserted (faithful image); and it costs four columns, 44 instead
of 40 (reasoned), without moving the bound of D-H. E1 is still of the v2 commitment (D-F): a v1 leaf
is not `M(C1, X)` of any `C1` that can be opened.

What (a) leaves to E2: without `X` in the envelope, nothing public links the claimant's half with
the payer's; the link, if needed, is decided there (the pledge label, `H(dominio, C2)`, carries the
salt and is the candidate). And the same limit holds against the RECEIVER, who receives `X` in the
notice: the opacity of D-1 of RFC-0003 depends on `refund_id` not being guessed. That belongs to
0003 and this decision does not touch it. **Reversible** towards (b) only if the envelope `X` gains
randomness (a salt of the sender, a format break of the commitment) and E2 measures that the link
needs it.

### D-J — The claim envelope: the age mould, and what is signed only in the head

Measured on `7bb3942` (`TERRENO-COBRO-149`): the producer of §491 delivers, besides the proof and
the statement, the `seq` and the two roots the head already signs
(`crates/zk-ssl/src/prueba_cobro.rs`, `SobreCobro`), and fixes the upper bound at the field's
ceiling (`MAX_VALOR`), which the AIR declares public. Two paths: (a) the mould of the age envelope
(`spec/PAQUETE.md` 2.7),
`{v: 1, tipo: "cobro_pendiente", cabeza, enunciado: {receptor, nacido, inferior}, prueba}`, with the
head VERBATIM, `seq`, `pendingRoot` and `pmetaRoot` only from it, and without `superior`, which the
judge fixes; (b) the envelope repeats what the producer delivers. (a) wins: coherence (it is the
sibling D-E names), clarity (one datum, one source: what the envelope does not repeat cannot
disagree with what is signed) and purity (one statement, one form; with `superior` in the envelope,
the same claim would have as many forms as ceilings). It is born as `PAQUETE.md` 2.8, with the arm
of the command, in E1 (D-L). **Reversible** towards (b) only if a measured consumer needs the
statement without opening the head; and towards a `superior` in the envelope, only by the way D-B
leaves open (a narrower band) and with its witness.

### D-K — The judge that links lives in the kit crate, and the born index comes before the head

Measured on `7bb3942`: the AIR's judge, `zk_ssl_air::cobro_pendiente::verificar`, takes the two
roots as public input and nothing ties them to a head; the age proof has that rule in the same crate
(`zk_ssl_air::verificar_contra_cabeza`), which the kit compiles without the prover; and the producer
of §491 verifies what it produces with the judge without the link. Reasoned from the code, and not
yet measured: the head signs `seq = log.len()` (`crates/zk-ssl/src/lib.rs`) and a pending item is
born with `nacido = log.len()` before its entry is recorded (`crates/zk-ssl/src/two_phase.rs`), so
under an honest head every live meta satisfies `nacido < seq`; the AIR does not restrict it. Two
paths: (a) the rule lives in `cobro_pendiente` —a single producer, in the crate the third party
compiles—, composes the statement with the two roots of the head and `superior = MAX_VALOR`, and
requires `nacido < seq` natively BEFORE verifying the proof; the layer's producer now verifies with
that same rule (the mould of §466); (b) the same, without the `nacido` rule. (a) wins: fail-closed
(a meta born after the head that signs it is a head that lies, and it is stopped with its name),
clarity (Security says «born at `b`», and with the rule `b` is earlier than the head that asserts
it) and coherence (the age link). The rule is measured before being written, in the PRE of the cut,
on a real pending item with its head. **Reversible** towards (b) if that measurement refutes it —an
honest pending item with `nacido >= seq`—: then the rule was false and it is not written.

### D-L — The perimeter of E1: the envelope and its judge; the catalogue, in E4

Measured on `7bb3942`: row E1 closes with the envelope «verified by the command without a node», and
row E4 carries the catalogue and the bench; the work plan, outside the tree, also put the vectors
into E1. Two paths: (a) E1 delivers the envelope (`PAQUETE.md` 2.8), the judge of D-K, the arm of
the command with the negatives that fall before the signature and the LINKED positive, with a real
proof, in the tests; the catalogue, the entry point that writes the envelope (D-M) and the bench
belong to E4; (b) E1 also carries its vectors. (a) wins: faithful image (the text adopted in §483
rules, and the plan is corrected) and coherence (the mould of RFC-0007: §465 gave the envelope and
its judge; §466 and §467, the entry point and the catalogue). For the same reason row E4 is
corrected: section 2.8 is born in E1, and E4 is left with section 9. **Reversible** towards (b) if,
on closing E1, E4 is split by sides and the claim half is brought forward before E2.

### D-M — The claimant's entry point is the cli, with the v2 notice in a file of its own

Measured on `7bb3942`, although it is executed in E4. There is no serialized form of the v2 notice:
`PendingNoticeDto` does not carry `x`, and its conversion sets it to `None`
(`crates/zk-ssl-wire/src/lib.rs`). No tool leaves a v2 pending item in a persisted ledger:
`simulate` sends by the v1 way, and `run_send_v2` is only called by conformance, in memory
(`crates/zk-ssl-cli/src/`). And the age bench is not copied: it produces with the node STOPPED, and
`zkssl_pendingPath` asks for the node LIVE, with heartbeat and `--clave`. Three paths: (a) the cli:
`simulate` learns the v2 way and writes the notice in a file of its own, and a subcommand of the
claimant reads that notice, the response of `zkssl_pendingPath` and the head, and writes the
envelope; (b) a mode of the node, like `--prueba-edad`; (c) the SDK. (a) wins: coherence of actors
(the witness of the claim belongs to the claimant, not to the operator, and its producer reads no
ledger) and clarity (the v2 notice has a client format while the wire does not transport it, and
`PendingNoticeDto` is not recycled). (b) changes actor and asks to stop the node right after a
heartbeat without traffic; (c) pays by the v1 way of the wire, and its pending item is not E1's
(D-F). **Reversible** towards a v2 DTO of the wire when the v2 claim travels through it, which is E4
of RFC-0003 and is decided there.

### D-N — The two forms of the claim positive: existence and the narrowest band

Measured on `9512915` (`TERRENO-E4-150`): row E4 asks for two positives per side, gathered from the
captures of a real node with the mould of `edad/`, whose two positives are the two forms its
statement admits (all, `t = 0`, and the empty box). The claim statement is
`{receptor, nacido, inferior}`; the producer takes `inferior` as an argument and refuses if it
exceeds the amount of the notice (`crates/zk-ssl/src/prueba_cobro.rs`), and the judge fixes the
upper bound at `MAX_VALOR` (D-J). Two paths: (a) two forms of the SAME statement over ONE pending
item: `inferior = 0` —the pending item exists in my name, without saying how much— and
`inferior = importe` —the narrowest band that holds, «at least what there is»—; (b) two pending
items from two runs, with the same form. (a) wins: purity (one statement, two forms, as in the age),
clarity (each positive asserts a different thing, and between the two they show the range the
statement admits) and faithful image (the row asks for two positives, not two copies).
**Reversible** towards a third form —a strict band between 0 and the amount— only if a measured
consumer needs it, and then it enters with its vector without retiring these two.

### D-O — E4 is split by sides, in a single row

Measured on `9512915`: D-L left written the reversal «if, on closing E1, E4 is split by sides and
the claim half is brought forward before E2», and E1 is whole (§485-§495); 5.A-272 puts fix B ahead
of E2, so E2 drags a format break that the claim half does not drag; and the table of RFC-0007 never
split a stage into two rows: E3 and E4 name their cuts (E3a, E3b, E4a, E4b-1..3) in the status cell.
Two paths: (a) row E4 remains one, its text says that it goes by sides and its status cell names
what was sealed on each side; (b) two rows are born, E4a (claim) and E4b (payment). (a) wins:
coherence (the mould of 0007), purity (fewer rows, fewer exceptions) and faithful image (the cell
says what there is: until both sides are in, the stage is not). The payment side is done with E2,
because its bench and its entry point need the proof E2 brings. **Reversible** towards (b) if the
payment side requires deliveries that do not fit in a cell.

### D-P — The claimant's credential is his own: the entry point receives it, it does not derive it

Measured on `9512915`: `zkssl_pendingPath` requires the receiver's credential (`viewKey`, §261) and
the layer crosses it with the stored `view_id`; the only entry point of the tree that hands over a
view key is `dev_openSeeded`, which opens a NEW account; `derive_view_key_wide` lives in
`stark_experiment::native`, which the cli does not declare although it already compiles it by way of
`zk-ssl`; and the position of an account is `public_id[0] % cap` with linear probing, derivable but
not pure. Three paths: (a) the entry point receives `--index` and `--view-key` by argument, as a
real claimant would have them, and the sandbox —which in the demonstration IS the claimant's
machine— writes that credential in a file of its OWN, apart from the notice, derived from its
deterministic key, declaring `stark-experiment` in the cli (zero new nodes in the graph); (b) the
entry point derives index and credential from `--key-seed` and `--to`, and only works in the
sandbox; (c) a method of the wire that serves the view key. (a) wins: coherence of actors (the
notice belongs to the payer and the credential to the claimant: two files, two owners, and no
recycled name), clarity (an entry point a real claimant can use) and the design law ((c) would make
the node hand over what it only authorizes to read, §261). **Reversible** towards (b) if the
credential file turns out to be a piece nobody really keeps and the sandbox trace already says it
all.

### D-Q — One negative per producible rule with a mutation of a real capture

Measured on `9512915`: the arm of the command falls before the signature with house texts
(`falta enunciado`, `falta prueba o no es cadena 0x`, `falta cabeza`, `falta signature`,
`exige una cabeza v5`); the judge that links falls with a house text on `nacido >= seq` and, if the
proof does not verify, with the prefix `cobro:` in front of what winterfell says (the mould of
`neg-cota-movida` in `edad/`). Two paths: (a) one vector per rule: the five from before the
signature, `nacido = seq` pinned whole, and ONE for the judge's rule —receiver moved, `inferior`
above the amount and corrupt proof fall by the same rule, so one enters—: seven, and no scene; (b)
one vector per mutated piece. (a) wins: coherence (the catalogue pins the rule, not the piece, as in
`rechazo/` and in `edad/`) and purity (fewer vectors that say nothing new). It is confirmed with the
bench's captures and not before: a mutation that falls by a rule other than its own does not enter.
**Reversible** towards more vectors only if a capture shows a rule this list does not name.

### D-R — A single bench for the two sides of E4, with the name the queue already used

Measured on `8d064c6` (`TERRENO-B-150`): row E4 goes by sides in a single row (D-O) and the
catalogue will be one (D-AC); the age bench produces with the node STOPPED, because the layer opens
the ledger and proves alone, and the claim cannot copy that order: the snapshot lives in the
heartbeat, in memory, and `zkssl_pendingPath` only serves it with the node LIVE (D-F), while `sled`
has the ledger exclusively. Two paths: (a) one bench, `tools/banco_pendiente.sh`, which seeds with
the node stopped and claims with the node live, runs today the claim side and will gain the
payment's with E2; (b) one bench per side. (a) wins: coherence (one row, one catalogue, one bench)
and clarity (the name the queue already used, and that the script names the pending item and not a
side). **Reversible** towards (b) if the payment side requires an order that does not fit in the
same script.

### D-S — ONE seeding: the two positives are two bounds over the same pending item

Measured on `8d064c6`: the two positives of D-N are two forms of the SAME statement, and a single
run avoids the trap of the logical positions of the sparse tree. Two paths: (a) one seeding and two
claims, `inferior = 0` and `inferior = importe`, over the same leaf and under the same head; (b) one
seeding per positive. (a) wins: purity (D-N: one pending item, two forms), faithful image (the two
envelopes assert different things about the same fact) and fewer scenes. **Reversible** towards (b)
only if D-N is reverted towards two pending items.

### D-T — The heartbeat window is paid with retry and with a short heartbeat

Measured on `8d064c6`: the entry point requires `s == cabeza.seq` and does not retry (§497); the
snapshot lives in the heartbeat and the node beats at its own pace, so between asking for the head
and asking for the snapshot an epoch may close. Three paths: (a) the bench retries —five times, ONLY
on the text of that race, and saying how many— with a short heartbeat; (b) a long heartbeat, so that
the window does not close; (c) that the entry point retries. (a) wins: the entry point still does
not retry, hence a race is distinguished from a real failure by its text and by its count, and the
bench does not wait minutes. **Reversible** towards (b) if five retries did not converge on a node
with traffic.

### D-U — The envelopes, the notice and the credential are kept; the head, not apart

Measured on `8d064c6`: the envelope carries the head VERBATIM (D-J), and a separate copy would be
two sources of the same datum. Two paths: (a) the bench keeps the two positives, the seven negative
bodies, the notice and the credential, and not the head; (b) also the head, in its own file. (a)
wins: clarity (one datum, one source: the lesson of D-J) and faithful image (what the catalogue
copies is what the command reads). **Reversible** towards (b) only by the way D-J leaves open: a
measured consumer that needs the head without opening the envelope.

### D-V — The negative of the judge's rule is the bound above the amount

Measured on `8d064c6`: `inferior` is a public input of the AIR by a boundary assertion
(`COL_INFERIOR`), and the layer's producer refuses a bound above the amount by its name, «la banda
NO se sostiene» [the band does NOT hold]. Two paths: (a) the judge's negative is
`inferior = importe + 1`: the envelope of the band positive with that bound, which the judge rejects
with `cobro:` in front of what winterfell says, and which the entry point moreover rejects LIVE
without writing anything; (b) a corrupt proof or a moved receiver, which fall by the same rule
(D-Q). (a) wins: the same fact has two witnesses —the producer that stops and the judge that does
not verify— and a mutation that is read in the statement. **Reversible** towards (b) if a capture
showed that this one falls by another rule.

### D-W — Outside the canon, like the age one; run by the block that seals

Measured on `8d064c6`: a bench with a live node, a heartbeat and a race that is retried is not a
deterministic gate, and the canon already costs between 189 and 325 s (5.A-223). Two paths: (a)
outside the canon, like `banco_edad.sh`, and run by the block that seals what the bench produces;
(b) inside the canon. (a) wins: coherence (the age one) and that what the canon watches is the
catalogue, which is deterministic (D-X). **Reversible** towards (b) only if a bench could run
without a node.

### D-X — The source of the catalogue is the captures of the SEAL

Measured on `f1e0401` (`TERRENO-C-150`) and in PASTE-499-PRE: there are two sets of captures, that
of the loose run and that of the run of §498, and the nine envelopes are the SAME bytes in both
except `emittedAtUnix`. Two paths: (a) the source is the run of the seal, the one entry 498 names by
its OUTPUT; (b) the loose one. (a) wins: faithful image (a source the entry does not name is not a
source, and the block re-measures it by fingerprint before copying). **Reversible** towards another
run if the bench changes and the seal is redone.

### D-Y — The nine envelopes enter; the notice and the credential, declared by fingerprint

Measured on `f1e0401`: `tools/conformidad.sh` requires one manifest entry for every `.json` in the
directory, and the command reads neither the notice nor the credential: they are not envelopes. Two
paths: (a) the nine envelopes enter, and the manifest declares by their fingerprint the notice and
the credential they came from; (b) the eleven files enter. (a) wins: the harness dictates the form,
and purity (the catalogue is one of envelopes, like the other five). **Reversible** towards (b) if a
consumer of the catalogue needed to reproduce the entry point from it.

### D-Z — The names, as the bench kept them

Measured on `f1e0401`: a vector is copied byte for byte from its capture and is not renamed; the
name is the one the OUTPUT of the seal prints, and by it it is re-measured. Two paths: (a) the
bench's names; (b) renaming in the style of another family. (a) wins: faithful image (what the
catalogue says is a capture is found by its name in the capture). **Reversible** towards (b) only if
the payment side brought names that clashed with these.

### D-AA — What the manifest pins, and of the judge only the prefix

Measured on `f1e0401`: the positives give a three-line VERDE whose second line says the statement
(«por al menos 0, nacido en 4» and «por al menos 250000, nacido en 4»); the negatives fall with six
house texts and one with a prefix, `cobro:`, followed by what winterfell says. Two paths: (a) pin
the second line of the positives, the whole house texts and, of the judge, ONLY the prefix; (b) pin
the whole text of the judge. (a) wins: faithful image (the catalogue pins what the house asserts;
what follows the prefix is winterfell's and changes with it) and coherence (the mould of
`neg-cota-movida` in `edad/`). **Reversible** towards (b) if the manifest pinned winterfell by
version.

### D-AB — The catalogue with all its prose in one cut (C-1); the RFC, apart (C-2)

Measured on `f1e0401`: `FAMILIAS` of `tools/artefacto.sh` is the only producer of the artefact and a
sixth family moves the tarball, so the catalogue runs the whole canon; and the prose a catalogue
leaves stale —`PAQUETE.md` 9, 11 and 2.8, the two READMEs and the row of `spec/README.md`— is paid
in the cut that refutes it, taking the census of the file's SENTENCES and not of the files. Two
paths: (a) two cuts, the catalogue with all that prose and the canon inside, and the RFC apart and
without canon; (b) one. (a) wins: a canon cut and a prose cut do not mix, and the RFC does not wait
for the canon. **Reversible** towards (b) only if a catalogue did not move the artefact.

### D-AC — The family and the stanza are called `pendiente`

Measured on `f1e0401`: the payment side will go to the SAME catalogue (D-O), and each family of the
canon carries a stanza derived from its sister's by substitution. Two paths: (a) `pendiente`, the
object, with two sides inside; (b) `cobro`, the side. (a) wins: coherence with D-O and clarity
(names that mean a single thing: the catalogue is the pending item's). **Reversible** towards two
families if the payment side required a manifest of its own, the same condition as D-O's.

⚠️ **REVERTED by D-AR (§509/§510)**: the payment side required a manifest of its own, which is the
condition written above. They are two families, `pendiente` and `pago`.

### D-AD — The payment statement: `{receptor, importe, T}` exact, and `nacido`, as in the claim

Measured on `477dcab` (`TERRENO-E2-151`): the payer has the whole opening —receiver, salt, amount,
`refund_id`, `delta` and position—, because `send_materials_v2` takes it from him and
`SendMaterials` keeps it (`crates/zk-ssl/src/client.rs:201`, `:301`); the layer refunds when `now
- born >= delta`, with `now = log.len()` (`crates/zk-ssl/src/two_phase.rs:674`), so «not
reversible before `T`» is exactly `T <= nacido + delta`, in epochs of the log; and the claim
envelope already publishes `nacido` (`crates/zk-ssl/src/prueba_cobro.rs:54`). Two paths: (a) the
statement is `{receptor, importe, T, nacido}` —the three exact and the `nacido` of the meta, as in
E1—, and what the circuit proves is `delta >= T - nacido`: a lower bound of `delta`, never `delta`;
(b) `nacido` witness, and the circuit proves `nacido + delta >= T` with both hidden. (a) wins:
coherence (the same mould as E1 and the same link `nacido < seq`), clarity (one datum, one source:
`nacido` comes from the meta the head signs) and purity (one comparison instead of two ranges).
**Reversible** towards (b) only if a measured consumer needs to hide `nacido`, paying one more range
in the circuit.

### D-AE — The path for the payer: the same method, his credential and a gate in the meta

Measured on `477dcab`: `zkssl_pendingPath` requires the credential of `index` and derives the
receiver from it (`crates/zk-ssl-node/src/main.rs:1826`); the layer does not discriminate:
`FotoPendientes::cobro` recomposes the leaf with the notice and returns the path, the siblings of
the meta, the `emisor` and the `nacido` (`crates/zk-ssl/src/foto_pendientes.rs:62`, `:69`); a payer
has the notice and knows the receiver, but does not have the claimant's view key: today he cannot
ask for the path of the pending item he paid. Three paths: (a) the SAME method, with the PAYER's
credential, `receptor` as a parameter instead of derived, and a new gate in the node: the meta has
to name whoever asks, `foto.emisor == index`; (b) a new method, `zkssl_pendingPathPagador`; (c) that
the receiver passes him the snapshot. (a) wins: the same notion of snapshot and of credential
(§261), zero new methods, and fail-closed: «the meta does not name you» receives the same nothing as
someone else's notice, without saying what there is. (c) would make the payer depend on his
counterparty to prove against it. **Reversible** towards (b) if the double gate dirtied the
claimant's method.

### D-AF — The geometry of E1, with the gap filled

Measured on `477dcab`: `circuit_refund_v2` already opens the whole commitment
—`(receptor, aleatorio, f, delta)` with `C2` and the amount public— in four merges, composing the
envelope FIRST (`crates/stark-experiment/src/circuit_refund_v2.rs:5`, `:15`); in the E1 AIR lane B
does not hash in cycles 0 and 1 (`crates/zk-ssl-air/src/cobro_pendiente.rs:20`), the link of `X` is
at row 15 (`:91`) and the amount band spends three segments of 64 rows (`:62`). Two paths: (a) the
geometry of E1: lane B composes `X = M(refund_id, [delta, 0, 0, 0])` in its cycles 0-1 and the link
of cycle 2 reads it from lane B instead of four witness columns; the amount no longer has a band —it
is public and the link of cycle 1 already ties it— and its segments prove `delta - (T - nacido)` in
`[0, 2^62)`; (b) a third lane, or a trace of 1024. (a) wins: purity (same trace, same columns plus
`delta` and `T`) and coherence (the claim mould, with `X` tied by structure and not by witness).
Cost REASONED, not measured: it is measured with the E1 instrument (§485) before writing the AIR, as
D-B dictates. **Reversible** towards (b) if the measured cost did not fit.

### D-AG — `delta` and `T` live in the field; the bound of «never», declared

Measured on `477dcab` and in PASTE-E2-PRE (SALIDA 20260918-155647): `refund_envelope` puts `delta`
through `BaseElement::new` (`crates/zk-ssl/src/pending.rs:92`), Goldilocks reduces `u64::MAX` to
`2^32 - 2` (`crates/stark-experiment/src/range_check.rs:10`), and a test in a copy of the tree
measured it with proof of life: `refund_envelope(f, u64::MAX) == refund_envelope(f, 2^32 - 2)`, and
`C2` likewise. Two paths: (a) `T` and `nacido` in epochs of the log, `T - nacido` and `delta`
bounded to 62 bits like `MAX_VALOR`, and the RFC DECLARES that a committed «never» envelope proves
at most `T <= nacido + 2^32
- 2`; (b) that E2 closes the envelope to the field. (a) wins: faithful image (the circuit proves
what the commitment ties, which is modulo `p`) and perimeter (the letter of 0003, «the sender's
choices bind», belongs to 0003: the point goes to its queue, and what is decided there —declaring or
closing the envelope to the field, which is a format break— is not this RFC's). **Reversible**
towards (b) only if 0003 closes the envelope to the field: then the bound disappears and this
paragraph becomes history.

### D-AH — The link between the two halves: nothing new, and the strong one for the pledge

Measured on `477dcab`: `SobreCobro` publishes `receptor`, `nacido`, `inferior`, `seq` and the two
roots, and keeps quiet the amount, the salt, `X` and the sender
(`crates/zk-ssl/src/prueba_cobro.rs:52`); D-I left written that without `X` in the envelope nothing
public links the two halves and that the pledge label, `H(dominio, C2)`, is the candidate. Two
paths: (a) nothing new: a third party crosses `receptor`, `nacido` and `seq`, which both envelopes
publish, and the strong link is left for the pledge (E3); (b) publishing `C2` in both envelopes. (a)
wins: `C2` in the envelope would make all the envelopes of one same leaf linkable and would break
what the salted positions promise (D-H), and a dispute is opened with the pledge, which is the one
that names it. **Reversible** towards (b) only with a measured use case that the pledge does not
cover.

⚠️ **What the pledge delivers, measured in 159**: the strong link is for whoever has the NOTICE, not
for a stranger with the two envelopes. See D-AY, which measures it and declares it without replacing
this decision: path (a) from here is still the one that was taken.

### D-AI — The return credential belongs to the payer: third file, `--retorno`

Measured on `477dcab`: `simulate --v2` fixes the conformance pair —`f` = the sender's identity,
`delta` = 96— (`crates/zk-ssl-cli/src/commands.rs:68`) and writes the notice and the receiver's
credential in two files (D-P), but persists nothing of the payer: `(refund_id, delta)` are derived
from the seed and leave no file. Two paths: (a) `simulate --v2 --retorno` writes `{refundId, delta}`
in a file of the payer's OWN, and the `prueba-pago` entry point reads the notice, the return and the
payer's own credential: three files, two owners; (b) that the entry point derives them from the
seed. (a) wins: coherence of actors (D-P: a real claimant has his credential, a real payer keeps his
return), clarity (an entry point a real payer can use) and names that mean a single thing (the
notice is not recycled to carry the return). **Reversible** towards (b) only if the sandbox trace
already said it all, which is the reversal of D-P.

### D-AJ — The 2.9 envelope and its vectors, in `pendiente/`

Measured on `477dcab`: form 2.8 is `{v, tipo: "cobro_pendiente", cabeza, enunciado, prueba}` with
the head VERBATIM and the roots only from it (`spec/PAQUETE.md:240`); the command dispatches by
`tipo` (`crates/zk-ssl-verify/src/main.rs:167`); the first line of the `pendiente/` manifest says
«Vectores del sobre de COBRO PENDIENTE» (`spec/vectors/pendiente/MANIFIESTO.txt:1`); and
`pago_en_curso` does not exist in any `.rs`, `.sh`, `.json` or `.txt`. Two paths: (a) form 2.9,
`tipo: "pago_en_curso"`, with `enunciado: {receptor, importe, t, nacido}` and its own list of
rejections (D-E); the `pago-*` vectors in the same `pendiente/` (D-AC), gathered from the SAME
seeding as the claim's (D-S): two positives per form of the statement —the exact `T`,
`T = nacido + delta`, and a smaller `T`, because «not before T» admits every `T <= nacido + delta`
as `inferior` admits every value below the amount— and one negative per producible rule (D-Q); the
first line of the manifest is paid in that cut; (b) a type with `lado`. (a) wins: D-E already
decided it and the rejection lists are different (the payment's carries the `T`), and the manifest's
sentence would be left stale and is paid where it is refuted. **Reversible** towards (b) only by the
way D-E leaves open: two identical rejection lists, which today they are not.

⚠️ **Form 2.9 stays; the DIRECTORY does not** (D-AR, §509/§510): the `pago-*` vectors do not live in
`pendiente/` but in `spec/vectors/pago/`. What fell was the premise: this decision gave them as
<<gathered from the SAME seeding as the claim's (D-S)>>, and they are not. The text above is CITED,
not erased (§247).

### D-AK — The payment producer is a free function, with the whole opening and without a ledger

Measured on `48699ab`: the claim producer (§491) is a free `fn` that opens no ledger, and the payer
has the same plus the pair `(refund_id, delta)` that the receiver receives opaque. Two paths: (a) a
file of its own, `crates/zk-ssl/src/prueba_pago.rs`, next to the mould, with `AperturaDelPago` as
input and reusing `FotoDelCobro` and `CabezaDePendientes` as they are; (b) a method of the ledger.
(a) wins: it reads no ledger, hence it runs in a client that does not have one, and reusing the two
types avoids two producers of the same datum. The producer **re-verifies what it produces** with the
same judge the third party runs, and checks `nacido < seq` BEFORE spending a proof, because an rc
that says «it does not link» hides the cause. **Reversible** if the pending item stopped fitting in
a free `fn` —today it does not: the snapshot is served to it by the node—.

### D-AL — The node prior to §505 ignores `receiverId`: it is DECLARED, not fixed

Measured on the ground of E2e: the `P` of `zkssl_pendingPath` never carried `deny_unknown_fields`,
so an old node takes the extra field, drops it and answers as if it had not come: it serves NOTHING
instead of an error, and the payer does not know why. Two paths: (a) declare it where whoever
suffers it reads it —`spec/RPC.md` and `PAQUETE.md` 2.9—, saying that what tells whether a node
knows what it is talking about is its `spec/openrpc.json` and not the protocol version, which does
not go up; (b) raise the version. (a) wins: raising it for an additive field breaks the wire for
everyone for a case only suffered by whoever talks to an old node. **Reversible** if a third party
appeared that cannot read the OpenRPC.

### D-AM — The payer's entry point is a file of its own, and three private ones go up to the crate

Measured on `6ecb063`: the claim entry point (§497) has the reusable public part —`leer_cabeza`,
`leer_foto`, `escribir`, `notice_de`, `credencial_de` and its two DTOs— and FOUR private functions,
of which the payment's uses three (`q_de`, `b32_de`, `respuesta`). Two paths: (a) its own `pago.rs`
and those three to `pub(crate)` where they are; (b) the payment inside `cobro.rs`. (a) wins: the
header of `cobro.rs` says «the claimant's ENTRY POINT», and putting the payment inside would make it
lie —yardstick 2: one name, one thing—. `cuerpo` stays private, since only `respuesta` uses it:
opening what is not used is cosmetic. **Reversible** towards a common module if a third entry point
were born.

### D-AN — The return travels in its own file, and a cheap gate falsifies it against the notice

Measured in PASTE-E2g-M: `simulate --v2` persisted nothing of the payer —the pair was derived from
the seed— and `refund_envelope` is the ONLY producer of `X`. Two paths: (a) a third file
`{refundId, delta}` written by `simulate --v2 --retorno`, with the precedent of D-P —one file per
owner—, and the entry point checking `refund_envelope(retorno) == aviso.x` BEFORE asking the node
for anything; (b) loose flags. (a) wins: the gate costs zero, fails closed and saves a proof, and
the file is passed as the notice is passed. What the gate does NOT prove is written in the
function's doc: `delta` enters Goldilocks and two congruent deltas give the same envelope (D-AG); it
discards the obvious, it does not fix the delta. **Reversible** towards (b) in a client without
files.

### D-AO — `--t` is ABSOLUTE

Measured: the producer takes `T` absolute and the statement publishes it (`PAQUETE.md` 2.9). Two
paths: (a) absolute; (b) relative to `nacido`. (a) wins: relative would force reading the snapshot
before composing the order, and the SAME order would give different envelopes depending on when it
is run. **Reversible** if a high-level entry point wanted to offer both, with the absolute one as
the one that travels.

### D-AP — The PAYER's credential also goes in a file, and its witness is the bench

Measured when writing the bench: `simulate --v2 --credencial` writes the RECEIVER's, and
`zkssl_pendingPath` requires the PAYER's when the one asking is the payer (D-AE). That file did not
exist, and the entry point gets by with a flag only because a human can type it; a bench cannot. Two
paths: (a) `--credencial-pagador`, the SAME triple and the SAME `credencial_de` with `a.from`; (b)
putting it inside the return. (a) wins: they are two distinct objects with two distinct types, and
joining the triple with the pair would give a file that is neither one thing nor the other. Its
witness is NOT a unit test: if it wrote the receiver's, the node would accept it —it is valid— and
the gate `f.emisor == p.index.0` would serve nothing, so the falsifier is the bench. **Reversible**
towards a single payer file if one day the return stopped being opaque.

### D-AQ — The payment bench is its own, with its four-file seeding

Measured in §508: the claim's sibling (§498) seeds two files of two owners; the payment needs four,
and its cheap rejection does not even get to ask. Two paths: (a) its own `tools/banco_pago.sh`, with
the sibling's mould and the same order the heartbeat snapshot imposes; (b) a `--lado` parameter in
the claim's. (a) wins: the two seedings are no longer the same —the payment's writes the return and
the payer's credential— and a bench with two modes is a bench that one day tests the one it was not.
**Reversible** towards (b) if the two seedings coincided again.

### D-AR — The payment catalogue goes in `spec/vectors/pago/`, seventh family

Measured in §509, and this REVERTS D-AC and the letter of D-AJ through the door D-AC left written
—«if the payment side required a manifest of its own»—, with three facts: the payment's seeding is
ANOTHER (D-AQ), the rejection list is different (D-E already said so) and `tools/conformidad.sh`
requires that every `.json` in the directory have an entry, so a single directory would be a
manifest of eighteen entries mixing two envelopes and two benches, with ONE count in the canon where
there are two. Two paths: (a) `pago/` with its manifest, its stanza and its family; (b) `pendiente/`
with the eighteen. (a) wins, and as a bonus the pending item's manifest is not left stale. ⚠️ **It
was in fact taken in §509 BEFORE being written here**, following the line of the TRASPASO [handover]
that cited D-AJ and said «the seventh family»: a summary that had drifted from the decision it
cites. The lesson goes to the entry. **Reversible** towards (b) if the two rejection lists became
equal.

### D-AS — The MARK and the PLEDGE are two objects; D-C holds in (a), amended

Measured on the ground of E3 on `0bfe066`: the leaf of the consumption tree IS the whole digest
(`consumo.rs:143`) and the position is its 63 low bits (`zk-ssl-hash:243`), so the layer receives an
opaque and cannot distinguish classes without the preimage; and `apply_consumo` has seventeen lines
of body with zero domain, zero proof and zero authorization, which is what its own doc-comment
already declared by D-4 of 0006. The condition D-C left written for itself **is met**: the tree does
not distinguish classes. But what falls is not (a): it is the NAME, which meant two things. It
splits in two. The MARK is `H(DOMINIO_PRENDA, C2)`, one more leaf of the consumption tree, public
and precomputable by whoever has the notice —measured: the notice with the opaque envelope
recomposes `C2` exactly, `two_phase.rs:3364`—, and **asserts nothing by itself**. The PLEDGE is the
PAIR: the mark under the signed root plus the envelope with the proof of opening of the claim; a
third party requires the pair. Thus the guarantee is sustained not by who wrote the label —the payer
can write it, who also has the notice— but by who could produce the envelope, which is only the
receiver: the ownership that `circuit_claim_v2` already restricts in `COL_KEY` 25..29 with
`SPEND_KEY_DOMAIN`. It beats (b) by the whole yardstick: one primitive per property, not one new
column, no format era and no live vector expires. **Reversible** towards (b) only with a measured
use case that requires exclusivity of the POSITION and not only of the proof; then era 5 → 6 is paid
with the age vectors behind it.

⚠️ **The envelope of the pair is NOT the claim's**: E1's does not restrict ownership (D-G), measured
again in 159 —zero spending key in `cobro_pendiente.rs` and in `circuit_cobro_pendiente.rs`—. The
pair carries the PLEDGE envelope, which does carry it. See D-AV, which decides its proof without
replacing this decision.

### D-AT — The limit of the mark, and it is written where whoever suffers it reads it

Anyone with the notice can publish the mark before the receiver, through the free entry point
`zkssl_publishConsumo`, which asks for neither proof nor authorization; and whoever only looks at
the tree does not know whether there was a pledge. It is neither double use nor denial: it is that
the mark alone proves nothing (D-AS), and it is the same class that D-4 of 0006 already publishes
—whoever publishes first blocks—. Nor does getting ahead harm the receiver: the mark the payer would
write is **the same leaf** the receiver needed, and the envelope is still his. The real denial would
be a TARGETED `ConsumoColision` —another digest in the same 63 low bits—, which the layer already
rejects with its name (`consumo.rs:222`) and which costs on the order of 2^63 attempts: **reasoned,
not measured**. This is said in the Security of this RFC and, when E3 is assembled, in
`spec/PAQUETE.md`, which is where whoever is going to pledge reads it. **Reversible** only if a
measurement lowered that bound, and then the pledge asks for a tree of its own.

### D-AU — D-D narrows by one word: second PROVEN pledge

Where D-D says «there is no second pledge» it now says «there is no second PROVEN pledge», and where
row E3 says «a second pledge is `ConsumoRepetido`» it now says «a second MARK». The rejection with
proof is the same and stays where it was (RFC-0007 E3, `PAQUETE.md` 2.6): what changes is which
object it speaks of. The rest of D-D is ratified without touching a comma: the pledge touches
neither the claim nor the refund, a pledged pending item is claimed and refunded the same after `T`,
and what binds the claimant with his financier is contract —the system produces the condemning pair,
it does not adjudicate—. **Reversible** towards «the pledge blocks the refund until `T`» only with
its negative witness written beforehand and with the rule committed in the head, because a rule that
is not committed does not sustain a proof.

### D-AV — The pledge proof is an AIR of its own: the claim mould with the key cycle

Measured on `4aee725`: `crates/zk-ssl-air/src/cobro_pendiente.rs` and
`crates/stark-experiment/src/circuit_cobro_pendiente.rs` do not name the spending key even once, and
the public inputs of the claim judge are
`{pending_root, pmeta_root, receptor, nacido, inferior, superior}`: the statement of STATE that D-G
chose on purpose. The payment's does not name it either. Only `circuit_claim_v2` carries it —eight
times, with `SPEND_KEY_DOMAIN` in `COL_KEY` 25..29—, and it is welded to the credit: its public
inputs are the whole transition (`ClaimPublicInputs`, with the two account roots, the amount and the
supply) and its constraints attest the balance (`C_BALANCE` and `C_SUPPLY`,
`circuit_claim_v2.rs:855` and `:857`). Three paths. (a) The pair carries the claim envelope, which
already exists: it falls, because that envelope does not restrict ownership and the payer produces
it the same (D-G), so the pair would condemn nobody. (b) `circuit_claim_v2` «without the credit», as
if it were a mode of its own: it falls, because it is not a mode but another circuit —removing the
credit changes the public inputs and the constraints—, and leaving it would publish the amount and
the account roots, which the pledge does not need. (c) An AIR of its own: the mould of
`cobro_pendiente` plus the key cycle that D-G left named for here (`CYC_PK`: one cycle and four more
columns). (c) wins by yardsticks 1 and 4: what the circuit does not restrict does not exist, and one
primitive per property —the pledge statement is «under this signed head there exists this `C2` in my
name, and I am the one who could claim it», without transition and without credit—. The objection
D-G put to its (b) does not bite here: there one wanted to tie the proof to whoever PRESENTS it, and
that asks for a challenge inside the statement; the pledge only needs that nobody else HAS BEEN ABLE
to produce it, which is just what the key cycle gives. The cost is declared: a pin in `zk-ssl-air`
and in `stark-experiment`, and the ~40 s per seal that a witness which produces and verifies a real
STARK costs (points 303 and 310). This decides the PROOF, not the assembly: the method, the sixth
domain and the eighth arm of the command remain unwritten, and E3 remains proposed. **Reversible**
towards (a) only if a measured use case showed that the mark with a state envelope is enough; then
the pledge stops being authorization and is a label, and D-D falls with it.

### D-AW — The pledge envelope publishes the MARK; `C2` stays as witness

A third party has to be able to cross what the envelope says with what the consumption tree carries.
Two paths: (a) the envelope publishes `C2` and whoever judges computes the mark outside the circuit;
(b) the envelope publishes the MARK and the circuit proves inside that
`marca = H(DOMINIO_PRENDA, C2)`, with `C2` as witness. (a) falls: publishing `C2` makes all the
envelopes of one same leaf linkable and breaks what the salted positions promise, which is exactly
what D-AH discarded on its side. (b) wins: the mark is what the tree carries, so whoever judges
compares what he sees with what the envelope says and computes nothing; and the mould is already
written in the house —lane B of E1 starts its cycle by putting its domain in the capacity,
`b[0] = DOMINIO_META_PENDIENTE` (`circuit_cobro_pendiente.rs:157`)—, so the sixth domain enters the
circuit as CAPACITY and not only as a row of the REGISTRY of `zk-ssl-hash`. It costs one hash cycle,
and it fits without a new row (D-AX). **Reversible** towards (a) only if the consumption tree
stopped being public, which today it is by RFC-0006.

### D-AX — The geometry: not one new column nor one more row

Measured side by side on `3806f1e` (`PASTE-E3GEO-M`, pure reading, rc 0): the two judges that exist
share `ANCHO` 44, `TRAZA` 512 and `ROW_RAIZ` 279, and differ in countable pieces —E2 carries ten
more constraints and two fewer periodic ones than E1—. The band the pledge does not need costs,
counted from its own file, FOUR columns (`COL_INFERIOR`, `COL_SUPERIOR`, `COL_SBIT`, `COL_SACC`),
EIGHT constraints (from `C_SBIT_BOOL` to `NUM_RESTRICCIONES`), FIVE periodic ones and 192 rows. The
key cycle, priced where it lives today, is ONE cycle and FOUR columns with twelve constraints
(`C_KEY_INPUT` eight and `C_PK_CHECK` four). And under the root there remain 232 free rows
—twenty-nine cycles— when two are needed. Hence the key occupies the columns the band leaves and the
mark is hashed in a free cycle: `TRAZA` stays at 512 and `ANCHO` does not exceed 44 in either of the
two forms —44 exactly with the meta, 42 without it (D-AY)—. The constraint count stays on the order
of E2's 120, and the positive, on the order of the 213-218 ms E2 takes in the same harness, against
E1's 127-137. That last thing is a PREDICTION and not a measurement: the instrument cannot run a
circuit that does not exist, and its gate is the written AIR. **Reversible** towards a greater width
only if, on writing it, a constraint did not fit; then the column and its cause are declared.

### D-AY — The pledge does not carry the meta, and the link with the claim is made by the NOTICE

The pledge statement is «under this signed head there exists this `C2`, whose mark is the one
published, and the key that would claim it is mine». It does not need `nacido`, which is what the
meta leaf contributes. Two paths: (a) carrying the meta lane, like E1 and E2, and publishing
`nacido`; (b) not carrying it. (b) wins by purity —what the statement does not need does not enter—
and because it is really cheaper: measured, lane B does not hash in cycles 0 and 1 and climbs the
meta from 2 to the root (`circuit_cobro_pendiente.rs:131` and `:140`), so without the meta it is
left entirely free for the key and the mark, without touching a row; and `COL_EMISOR` and
`COL_NACIDO` go too, with which the width drops from 44 to 42.

⚠️ **And what this does to D-AH has to be said**, which left written that «the strong link is left
for the pledge». Measured: it is not left for a stranger. The claim envelope does not publish `C2`,
so whoever only has the two envelopes still crosses `receptor` and `seq`, which is the WEAK link
D-AH itself describes. Strong is what whoever has the NOTICE has: with it he recomposes `C2`,
computes the mark and checks that the two halves speak of the same leaf. That is enough for the
pledgee, who receives the notice on pledging, and it is faithful image: the system produces the
condemning pair for whoever is a party, not a proof before everyone. **Reversible** towards (a), and
towards putting the band INSIDE the pledge so that it says the amount by itself; the latter would
cost 46 columns, two more than E1, and asks for a measured use case that requires it.

### D-AZ — `zkssl_pledge` verifies the envelope and only then writes, and asks for no credential

The additive method row E3 announces receives `{prueba, receptor, marca, seq}`, composes the
statement with the `pendingRoot` of the signed head the NODE keeps —never with one that comes from
outside—, calls `zk_ssl_air::prenda::verificar_contra_cabeza` and only with green calls
`apply_consumo`. Four paths: (a) requiring the envelope without looking at it, which falls by
faithful image —a method that says it requires something it does not check stores bytes nobody
verified—; (b) verifying and writing; (c) verifying and not writing, leaving the mark to the free
entry point; (d) that the method not exist. (b) wins by coherence and by what the judge itself
declares: it does not check that the mark is under `consRoot` because «it is the gate of whoever
writes in it», and whoever writes is the node. The house pattern —`prove` in the client, `apply`
VERIFIES without seeing the key— is met to the letter, because the envelope carries no key (D-AW).

⚠️ **It is not a gate of the consumption tree, and this closes 5.A-353.** The free entry point
`zkssl_publishConsumo` stays open and asks for nothing (D-AT): this is an entry point WITH proof
next to a free entry point. And for that very reason **it requires no credential**: the
authorization is the proof, not the possession of a view key on the server.

⚠️ **A pledge is valid within its epoch**, and it is a MEASURED restriction that was not written:
the node keeps ONE signed head and loses it on restart, so the `seq` the caller declares has to be
that of the last one. If it is not, it is rejected NAMING the one there is —the opposite of the
silent refusal of D-AE, because the mark is public and precomputable by whoever has the notice—. The
order is `zkssl_pendingPath`, produce, `zkssl_pledge`, under the same heartbeat. **Reversible**
towards (c) if it is ever decided that the node should not compile a STARK verifier; today
`zk-ssl-air` was already in its graph by way of `zk-ssl-verify` and declaring it added zero nodes,
measured.

### D-BA — What it returns: the mould of the additive sibling, plus one field

In green, `{accepted: true, yaEstaba, logSeq, s}`; in red, `{accepted: false, reason, data}` with
the form of §454. `logSeq` is the exact mould of `zkssl_publishConsumo`; `s` is the `seq` of the
head it was judged against, which is what a third party needs in order to ask for the path
afterwards; and `yaEstaba` enters because it is the only thing in the response the caller cannot
compute on his own (D-BB). It does not return the mark, which he sent, nor the path, which is
`zkssl_consumoPath`.

⚠️ **And it does not accept `pendingRoot` as a parameter.** Receiving it would be letting the one
who asks make the yardstick he is measured with, which is the same reason why the head does not
travel in §248. **Reversible**: adding fields to a response is additive and does not raise
`zkssl/0.3`; removing them is not. That is why it enters with the minimum.

### D-BB — A repeat whose envelope verifies is NOT a failure of the pledge

`zkssl_pledge` ALWAYS verifies first. If the proof does not verify, rejection, and it does not
matter what is in the tree. If it verifies and the leaf is already there and is THE SAME, the
response is `accepted: true` with `yaEstaba: true` and the layer is not touched. A `ConsumoColision`
is a rejection, with its cause as data. It comes from crossing two decisions already taken: the pair
is the mark under the root PLUS the envelope (D-AS), and the mark the payer would write is the SAME
leaf (D-AT). If both are true, the pair exists as soon as the envelope verifies, whoever wrote it,
and returning `ConsumoRepetido` would be calling a success a failure.

⚠️ **The deduction that sustains it is declared**: two distinct pledges of the same pending item
cannot exist, because the mark is a function of the leaf (`marca_prenda`, a single producer) and the
leaf carries the receiver inside; with another key the leaf does not climb to the root and the
producer fails closed. Its falsifier —two envelopes of the same notice with two distinct keys, and
the second does not verify— is not the node's: it lives where the producer lives, and it remains
pending. **Reversible** if it is decided that the record must distinguish WHO wrote the leaf; then
the pledge asks for a tree of its own, which is the exit D-AT itself leaves written.

### D-BC — The entry point's spending key enters through the SDK keystore: the cli never sees a `Digest`

Measured in 162 on `afd72d6` and re-measured in 179 on `cf65876`. The pledge entry point is the
first of the cli that needs the spending key —the claim is of STATE (D-G) and the payment goes with
its return (D-AI)—, and in `crates/zk-ssl-cli/src/` there is not a single `spend_key`. The SDK
already keeps it: `Wallet` is «the holder's wide key and its derivations. It never leaves here», and
its accessor `spend_key()` is `pub(crate)`. And it already has a format at rest, `zkssl-keystore/1`
(`crates/zk-ssl-sdk/src/keystore.rs`: XChaCha20-Poly1305, the `public_id` in the clear and the
spending material encrypted), whose `load` requires that the decrypted key derive the declared
`public_id`. Section 3.bis of `SECURITY.md` publishes, as verified in code, that `Wallet::spend_key`
is private and does not even implement `Serialize`. Three paths: (a) the cli depends on
`zk-ssl-sdk`, `keystore::load(fichero, frase)` gives it the `Wallet`, and the whole operation is
done by the SDK (D-BH); (b) a file in the clear with the key, which is what was proposed first,
deducing its format from the witness's `--clave-fichero` —which is the XMSS seed, SIGNING material,
and not the spending key—; (c) the entry point inside the SDK. (a) wins, and the author chose it,
because it touches a published property and is not taken with the yardstick: no other format of key
material is born (yardstick 1), the sentence of `SECURITY.md` remains true letter by letter
(yardstick 4) and the entry point stays in the cli with its two sisters (yardstick 3). (b) brings in
a third format, in the clear, next to an encrypted keystore, and forces rewriting that sentence; (c)
touches nothing, but it stops being `zk-ssl-cli prueba-prenda` and breaks the symmetry with
`prueba-cobro` and `prueba-pago`. The cost, measured in the `Cargo.lock` of `cf65876` by name and
version: the cli's closure gains three packages —`zk-ssl-sdk`, `rand 0.8.7` and
`rand_chacha 0.3.1`—, because `chacha20poly1305`, `sha2` and `ureq` were already in it; none is born
in the lock, where the SDK already resolves as a member of the workspace. The cli has no closure
gate, and this is the dependency 5.A-359 announced. The residual trust, in the keystore's own words:
SHA-256 is not a password derivation function, so the file keeps the key as much as its passphrase
keeps it; hardening it is matter for another RFC. ⚠️ §702: that RFC is RFC-0001, and the keystore
moves to `zkssl-keystore/2`, with Argon2id, salt and cost; a `zkssl-keystore/1` is migrated on
opening it with its passphrase. **Reversible** only if the SDK stopped keeping the key: then the
entry point would need a format of its own, and that would be another decision.

### D-BD — The entry point carries no `--receptor`: the receiver is derived from the key

The producer does not accept it as data: `prueba_de_prenda` derives it from the key with
`derive_public_id_wide`, and if the key is not the pending item's, the recomposed leaf does not
climb to the root and it refuses before spending a proof (§518). Two paths: (a) the entry point does
not ask for it; (b) a `--receptor` flag the producer would ignore. (a) wins by clarity: a flag that
governs nothing can lie, and whoever read the order would believe one can pledge in someone else's
name. The receiver comes out in the envelope, derived. **Reversible** only if the statement stopped
deriving it from the key, and that would be another AIR (D-AV).

### D-BE — `--publicar`, optional: the entry point always writes the envelope, and with the flag publishes it

`prueba-pago` asks a live node for the head and the snapshot, composes its envelope and writes it
(§507); it publishes nothing, because its own is not a transition. The pledge is one, and D-AZ
measured that it is valid WITHIN ITS EPOCH: the node keeps a single signed head and `zkssl_pledge`
rejects the envelope whose `seq` is not that one's, so the order is `zkssl_pendingPath`, produce,
`zkssl_pledge`, under the same heartbeat. Three paths: (a) only writing, like `prueba-pago`; (b)
writing and always publishing; (c) always writing and publishing only with `--publicar`, printing
what `zkssl_pledge` returns, `yaEstaba` included (D-BA). (a) falls: between writing and publishing
by hand, the epoch window opens by itself. (b) falls: publishing is writing in the ledger —the mark
enters the consumption tree—, and neither of its two sisters writes in it; the envelope, moreover,
is already by itself the half the command judges without a node (§520). (c) wins. **Reversible**
towards (a) if `zkssl_pledge` stopped tying itself to the last signed head (D-AZ).

### D-BF — The pledge requires a v5 head, and with ITS OWN reason

Measured in 162 on `afd72d6` and re-measured in 179 on `cf65876`: `cabeza_v3_verificada` admits v3,
v4 and v5, and `pendingRoot` enters the digest of all three, so the pledge COULD accept them. The
other three arms of the family —the age, the claim and the payment— require v5 saying «the only one
that signs pmetaRoot», and the pledge does not carry the meta (D-AY): copying that reason would
publish something false. Three paths: (a) admitting the three eras, which is what the statement
strictly needs; (b) requiring v5 with the siblings' text; (c) requiring v5 with a reason of its own.
(a) falls because the node serves v5 since §452 and `zkssl_pledge` judges against the head it keeps:
v3 and v4 are eras no pledge producer can emit, and what has no witness to falsify it is not built.
(b) falls by faithful image. (c) wins, and a witness of the command gates that the text does NOT say
`pmetaRoot`. §520 applied it in the eighth arm of the command and in form 2.10 of `PAQUETE.md`, and
here it is written; the entry point inherits it (D-BI). **Reversible** the day a producer emits
against another era.

### D-BG — The passphrase enters through `--frase-fichero`

The node already decided this for its seed and publishes the reason in its own rejection: `--clave`
on the command line leaves the seed in the shell HISTORY and in `ps`, and for operating there is
`--clave-fichero` (`crates/zk-ssl-node/src/main.rs`). The reason transfers whole to the keystore's
passphrase (D-BC). Three paths: (a) `--frase-fichero`; (b) the passphrase as the value of a flag;
(c) asking for it on the terminal. (b) falls by the reason the node publishes; (c) falls because a
bench does not automate it without carrying the passphrase in its script. (a) wins: the same pattern
the node uses for its secret material (yardstick 3), and the only thing a bench automates without
putting a secret in its script. **Reversible** without touching the keystore: the passphrase is an
input of the entry point and not a format, and another input can be added if an interactive use asks
for it.

### D-BH — The operation lives in `Wallet`, not in `Account`

Measured in 162 and re-measured in 179: `Account` is built with an `Rpc` —`Account::open` and
`Account::attach` receive it— and talks to the node; `Wallet` only carries the key and its
derivations (`public_id`, `view_key`, `view_id`, `leaf_salt`). And the two sister entry points do
the network themselves —they ask for `zkssl_signedEpochHead` and `zkssl_pendingPath`— and call a
producer that reads no ledger (§491, §504). Two paths: (a) `Wallet::prueba_de_prenda`, with the
head, the notice and the path the entry point passes it, which wraps the layer's producer
(`zk_ssl::prueba_prenda::prueba_de_prenda`, §518) with the key that does not leave; (b) a method of
`Account`, which would moreover ask for the head and the path. (a) wins: the key leaves neither the
crate nor the process, and `SECURITY.md` remains true letter by letter (yardstick 4); the network
stays in the entry point, which is where its sisters put it (yardstick 3); and the producer is still
a function that reads no ledger. **Reversible** towards (b) only if the SDK acted as entry point,
which is the (c) D-BC discards.

### D-BI — The pledge entry point reads the head and the snapshot with readers of its own

Measured in 162 and re-measured in 179 on `cf65876`: `leer_cabeza` of `cobro.rs`, which the payment
entry point reuses, requires v5 with the claim's reason —«the only one that signs pmetaRoot», false
for the pledge (D-BF)— and requires `pmetaRoot`; `leer_foto` requires `hermanosMeta`, `emisor` and
`nacido`, which the pledge statement does not use (D-AY). Two paths: (a) short readers of its own in
the pledge entry point; (b) generalizing the claim's, widening the cut to `cobro.rs`. (a) wins: the
perimeter of the cut stays at what the pledge needs, and the rejection says ITS reason (yardstick 4,
D-BF). **Reversible** towards (b) if a fourth entry point were born: then they are unified.

## What was DISCARDED on measuring

1. Opening `X` on the claimant's side to prove the T: it breaks D-2 of RFC-0003 (the receiver would
   learn `refund_id`).
2. Revealing the amount in the claimant's envelope instead of the band: it contradicts the
   milestone's text and the mould already exists.
3. The pledge as a field of the meta: format break (5 → 6) and expiry of the age vectors, for a
   property the consumption tree already gives.
4. A «set of spent or pledged items» of its own, as the PDF named it: the first measurement said
   that the single-use set with a signed root already exists, and that what is missing is the
   authorization, not the tree.
5. That the payer pledges, or anyone with access to the node: it is the denial of service of 0006
   applied to the claimant's asset.
6. That the pledge blocks the refund: a new rule over the payer's rights without a written witness
   and without a commitment in the head.
7. Serving the path of the pending item with the mould of `zkssl_frozenPath`, from the state of now:
   as soon as a payment falls between heartbeats, it climbs to no signed head (D-F).
8. Rebuilding the tree at the head's `seq` from the log: its entries carry neither the position nor
   the leaf (D-F).
9. Signing a head on request of whoever asks for the path: it burns XMSS indices (D-F).
10. Ownership in E1 without a challenge in the statement: it does not tie the proof to whoever
    presents it (D-G).
11. The claimant in one lane, with the position accumulated and an equality that ties it: a tie the
    house has never written and which depends on remembering to write it (D-H).
12. The envelope `X` as a public input of the claimant: with `refund_id` guessable, it says who paid
    and when it expires, and it links the envelopes of the same sender (D-I).
13. That the claim envelope repeat the `seq` and the two roots: two sources of the same signed datum
    (D-J).
14. The claim vectors inside E1: it contradicts the adopted row E4 (D-L).
15. The claimant's entry point in a mode of the node, which changes actor, or in the SDK, which pays
    by a way E1 refuses (D-M).
16. Two pending items from two runs as the two claim positives: the same form twice does not show
    the range the statement admits (D-N).
17. E4 in two rows, E4a and E4b: the table of RFC-0007 names its cuts in the status cell and never
    split a stage (D-O).
18. That the claimant's entry point derive its credential from the sandbox seed, or that a method of
    the wire serve it: the first only works in the sandbox and the second would make the node hand
    over what it only authorizes to read (D-P).
19. One vector per mutated piece where the rule is the same: the catalogue pins the rule (D-Q).
20. One bench per side of E4: one row, one catalogue and two benches (D-R).
21. One seeding per positive: two pending items where D-N asks for two forms of the same (D-S).
22. That the entry point retry, or a long heartbeat that covers the window: the entry point still
    does not retry and the bench says how many times it did (D-T).
23. Keeping the head apart from the envelope that already carries it: two sources of the same datum
    (D-U).
24. The corrupt proof or the moved receiver as the judge's negative: they fall by the same rule as
    the bound, and the bound is moreover rejected by the entry point live (D-V).
25. The bench inside the canon: a live node with a heartbeat is not a deterministic gate (D-W).
26. The captures of the loose run as the source of the catalogue: the entry does not name them
    (D-X).
27. The notice and the credential inside the catalogue: they are not envelopes, and the harness
    would require an entry for them (D-Y).
28. Renaming the vectors: a vector is copied, not renamed (D-Z).
29. Pinning the whole text of the judge: what follows `cobro:` is winterfell's (D-AA).
30. The catalogue and the RFC in a single cut: a canon cut and a prose cut (D-AB).
31. Calling the family `cobro`: the payment side goes to the same catalogue (D-AC).
32. `nacido` as witness in the payment envelope: two ranges where one comparison suffices, and a
    mould other than E1's (D-AD).
33. A new method for the payer's path, or that the receiver pass him the snapshot: two notions of
    snapshot, or depending on the counterparty (D-AE).
34. A third lane or a trace of 1024 for the payment: lane B has cycles 0 and 1 free (D-AF).
35. That E2 close the envelope to the field: it is the letter of 0003 and a format break; here the
    bound is declared (D-AG).
36. Publishing `C2` in both envelopes to link them: it links all the envelopes of a leaf (D-AH).
37. That the payer's entry point derive its return from the seed: it only works in the sandbox
    (D-AI).
38. An envelope type with `lado`: the rejection lists of the claim and of the payment are not
    identical (D-AJ).
39. The claim envelope (E1) as the pledge envelope: it does not restrict ownership —zero spending
    key in its judge and in its prover, measured in 159—, and the payer produces it the same (D-AV).
40. `circuit_claim_v2` «without the credit» as a mode of the same circuit: removing the credit moves
    its public inputs and its constraints, hence it is another circuit and not a mode (D-AV).
41. Publishing `C2` in the pledge envelope so that the link is public: it links all the envelopes of
    that leaf, which is what D-AH discarded from the other side (D-AW).
42. The pledge with the band inside, so that it says the amount by itself: 46 columns, two more than
    E1, and the pledgee already receives the claim envelope (D-AY).
43. The pledge with the meta lane: it publishes a `nacido` its statement does not use and costs two
    columns and the whole ascent of lane B (D-AY).
44. The entry point's spending key in a file in the clear: a third format of key material next to
    the SDK's encrypted keystore, and a sentence of `SECURITY.md` that would have to be rewritten
    (D-BC).
45. The pledge entry point inside the SDK: it stops being a subcommand of the cli and breaks the
    symmetry with its two sisters (D-BC).
46. A `--receptor` flag: the producer derives it from the key, and a flag that governs nothing can
    lie (D-BD).
47. That the entry point only write the envelope, like `prueba-pago`: between writing and publishing
    by hand, the epoch window opens by itself (D-BE).
48. That the entry point always publish: it would write in the ledger without being asked, and
    neither of its two sisters writes in it (D-BE).
49. That the pledge admit v3 and v4: no pledge producer emits against them, and what has no witness
    to falsify it is not built (D-BF).
50. Requiring v5 with the siblings' reason: it would give as reason a root, `pmetaRoot`, that the
    pledge does not carry (D-BF).
51. The passphrase as the value of a flag, or asked for on the terminal: the first stays in the
    history and in `ps`, and the second is not automated by a bench without carrying it in its
    script (D-BG).
52. The operation as a method of `Account`: `Account` carries an `Rpc`, and the network belongs to
    the entry point (D-BH).
53. Generalizing the claim's readers for the pledge: it widens the cut to `cobro.rs` for a fourth
    entry point that does not exist (D-BI).

## Compatibility

- `zkssl/0.3` **does not go up**. `zkssl_pendingPath` and `zkssl_pledge` are new methods; no
  existing method or object changes its form. The FORMAT version of the signature stays at 5: the
  MARK is a leaf of the tree `consRoot` already signs, and the pledge is the pair (D-AS), which does
  not live in the head.
- The vectors of `cable/`, `nucleo/`, `paquete/`, `consumo/`, `conflicto/`, `rechazo/` and `edad/`
  are not touched. `pendiente/` is born under its own manifest.
- The `PendingNotice` notice does not change: the claimant's half is produced with what it already
  receives.

### Why it enters through an RFC

It touches `spec/RPC.md` and `spec/openrpc.json` (two methods) and vectors are born: the three
things `PROCESO.md` reserves to an RFC. And it decides a new function of the ledger, the pledge,
which deserves its file even though it breaks nothing.

## Security

- **The operator sees.** Neither of the two proofs hides anything from the node: privacy is against
  the external verifier, as in the whole system.
- **What the claimant's envelope says**: that under that signed head there exists a pending item in
  the name of `receptor`, for at least `inferior`, born at `b`. What it does NOT say: when it
  expires, nor that it is going to be claimed, nor who paid it, nor who produced the proof (D-G).
  The envelope `X` does not travel (D-I): with it, whoever guessed `refund_id` would know who paid
  and when it expires.
- **What the path method reveals**: to whoever presents a notice that recomposes the leaf, the two
  paths and the meta of THAT position, in the state of the last heartbeat; to whoever does not,
  nothing (D-F).
- **What the payer's envelope says**: that under that head the payment is committed, for the exact
  `importe`, to `receptor`, and that it cannot be reversed before `T`. What it does NOT say: that it
  is done; it is consummated on claiming.
- **What the pledge says**: that the claimant authorized the pledge of that leaf, and it is said by
  the PAIR —the mark under the signed root plus the PLEDGE envelope, which proves the opening AND
  the ownership (D-AV)—, never by the mark alone, and never by the claim envelope, which anyone with
  the opening produces. The envelope publishes the mark and keeps `C2` quiet (D-AW), and does not
  publish `nacido` (D-AY). A second PROVEN attempt has its rejection proof (D-AU). What it does NOT
  say: that the claim is going to go to the pledgee, nor —for whoever does not have the notice— that
  this leaf is the one of the claim envelope. Both things are contract.
- **What the mark does NOT say**: nothing. It is a public leaf, precomputable by anyone who has the
  notice —the payer has it, because he built it—, and publishing it through the free entry point of
  0006 does not prevent the pledge: it is the same leaf the receiver needed. The targeted denial
  would require a position collision on the order of 2^63 (D-AT).
- **Trust in the head** is the usual one: the kept signature and the co-signatures of the witnesses
  under threshold sustain it, without guaranteeing that a financier accepts it as a basis (challenge
  4 of the proposal, already written).
- **What the head does NOT sign, measured**: `emittedAtUnix`. Between the loose run and the bench's
  seal run, the nine envelopes of the catalogue are the same bytes except that field of the head:
  the STARK proof and the XMSS signature are deterministic, and the time enters neither the digest
  nor the signature (PASTE-499-PRE, SALIDA 20260918-112533). It is faithful image of the v5 head of
  RFC-0007, not a rule of this RFC; if it is debt —declaring it in `RPC.md`, or covering it—, it is
  decided apart (5.A-296).
- **What an envelope can «never» prove, measured**: the reversal envelope puts `delta` in the field,
  and Goldilocks reduces `u64::MAX` to `2^32 - 2`: two `delta` that differ by `p` give the same `X`
  and the same `C2` (PASTE-E2-PRE, SALIDA 20260918-155647, measured in a copy of the tree with proof
  of life). Hence, in the payment circuit, a committed pending item «never» proves at most
  `T <= nacido + 2^32 - 2`, in epochs of the log (D-AG). It is faithful image of the v2 commitment
  of RFC-0003; what 0003 does with its letter («the sender's choices bind», today modulo `p`) is
  decided there, and goes to its queue.
- **How far the temporal bound reaches, measured**: the 64-row segment of the payment lane does NOT
  prove <<fits in 64 bits>> —in this field that would say nothing, because `p < 2^64`—: it proves
  **`v < 2^63`**, because its first row requires bit and accumulator at ZERO and the accumulator
  doubles in the following 63. Hence the AIR sustains the statement as long as
  **`delta - (T - nacido) < 2^63`**, and `comprobar_enunciado` bounds `importe`, `T` and `nacido` to
  `MAX_VALOR = 2^62 - 1` and requires `nacido <= T`. The <<never>> of the point above, which the
  field reduces to `2^32 - 2`, falls comfortably inside. Whoever reads an envelope above those
  bounds reads no envelope: there is none.
- **Rule 3 of the PROCESO —the spending key never travels— is met here since §538** in what the E2
  suite of RFC-0009 measures: the pledge proof is produced in the client, like the claim, and since
  §538 (E3b-2) it does not publish the pledgor's key literally; between §521 and §538 it carried it
  in the clear, 42 times (§521).

## References

- The pure reading of session 144 on `5ef3b1b`, `TERRENO-H5-144` (text, `19c76ad04ad1d84b`/124; it
  lives outside the tree, in the author's Downloads, like the PASTEs of the previous RFCs).
- The pure reading of session 145 on `393032e`, `TERRENO-E1-145` (text, `953f0aeca3a1175c`/122; in
  the author's Downloads, like the previous one).
- The pure reading of session 147 on `be90eb7`, `TERRENO-AIR-E1-147` (text, `70c808caa9918072`/126;
  in the author's Downloads, like the previous ones).
- The pure reading of session 149 on `7bb3942`, `TERRENO-COBRO-149` (text, `e14211dd032dd4c0`/213,
  with 54 citations that `verifica_terreno149.py` re-asserts; in the author's Downloads, like the
  previous ones).
- The pure reading of session 150 on `9512915`, `TERRENO-E4-150` (text, `705a74e9a9a26487`/294, with
  55 citations that `verifica_terreno150.py` re-asserts, 55 of 55 in the author's tree with
  porcelain 0 before and after; in the author's Downloads, like the previous ones).
- The pure reading of session 150 on `8d064c6`, `TERRENO-B-150` (text, `2ed966d6bb84daf2`/225, with
  54 citations that `verifica_terrenoB.py` re-asserts, 60 of 60 in the author's tree with porcelain
  0 before and after; in the author's Downloads, like the previous ones).
- The pure reading of session 150 on `f1e0401`, `TERRENO-C-150` (text, `9d510f4b488375d0`/216, with
  44 citations and the eleven captures of the seal by fingerprint, which `verifica_terrenoC.py`
  re-asserts, 60 of 60; in the author's Downloads, like the previous ones).
- The pure reading of cut C-1, PASTE-499-PRE (`ffefe25c314f6e53`/206; SALIDA 20260918-112533), which
  measured that between two runs of the bench only `emittedAtUnix` moves.
- The pure reading of session 151 on `477dcab`, `TERRENO-E2-151` (text, `274acf0873d38344`/287, with
  54 citations and three absences that `verifica_terrenoE2.py` (`e95466ce8a5b1f70`/173) re-asserts,
  79 of 79 in the author's tree; in the author's Downloads, like the previous ones).
- The falsifier of section 4 of that ground reading, measured in a copy of the tree by PASTE-E2-PRE
  (`386cbcc36c4da6f3`/206; SALIDA 20260918-155647: `cargo test` green, with proof of life).
- The E1 instrument and its run: `crates/zk-ssl/src/instrumento_cobro.rs` (§485) and the output of
  the PASTE that rehearsed it outside the tree (`96cf0169b0c09589`/44, in the author's Downloads).
- The milestone, verbatim, on line 46 of the submitted form (`NLNET-form-answers-EN-v3.txt`,
  `26dcde32091e857d`/160) and in section 3.6 of the attached proposal (the PDF `a6d5b620bf4e2283`, 9
  pages).
- RFC-0003 (the v2 commitment: D-1 the opaque envelope, D-2 `refund_id` committed), RFC-0006 (the
  published consumption and its D-4), RFC-0007 (the v5 head, the rejection by paths, the band).
