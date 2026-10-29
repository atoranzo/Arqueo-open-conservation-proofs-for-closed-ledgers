# RFC-0009 — What a proof reveals: the promise while the prover does not hide its witness

> **Español:** [`spec/rfc/0009-lo-que-revela-una-prueba.md`](../0009-lo-que-revela-una-prueba.md) is
> the original of this page, in Spanish; it prevails wherever the two differ. This translation is of
> commit `7766efb` (audit entry §917) and was produced by the project's assistant, a Claude Code
> session, in audit entry §918 (`GENAI.md`): it is a generated artefact, not text written by the
> author. Line citations of the form `file:N` refer to the Spanish files.

- **Status:** ACCEPTED — **rule 4 of the PROCESO, settled with measurement** (§527 the adoption and
  E1; §528 E3, to the design; E2 §531, E3a §532–§534 and E3b §535–§538, the stages with their gates;
  §538-B, §538-C and §538-D, the prose, the comments and the spec brought up to date; §539 this
  turn). It is kept as a record of what was decided, what was measured and what was discarded.
  ⚠️ §916: D-A changes what it promises for the first time since §538, through D-AM of §915 accepted
  by the author on 29-10-2026: hiding simulatable without the witness for the twenty-three families
  with `zkssl/0.5`, under named hypotheses.
- **Author:** Ángel José Toranzo Portela
- **GenAI assistance:** Claude (sessions 162, 163, 164, 165, 166, 169, 170, 171, 172, 173, 174,
  176, 177 and 178) — see [`GENAI.md`](../../../GENAI.md)
- **Date:** 2026-09-21
- **Protocol version affected:** `zkssl/0.3` → **`zkssl/0.4` in E3b-2 (§538)** (see
  Compatibility). Until E3b-2 this RFC changed no method, no wire type and no vector: it changed
  what is promised of them; E3b-2 changes the shape of every proof that crosses the wire.
- **AUDITORIA entry(ies):** §521 (the witness is published), §522 (the comments and the literal of
  the API), §523 (the model of the constant columns), §524 (PASTE-367-M3, and what was Groth16's),
  §525 (the comments, with the census that says which system each sentence speaks of) and §526
  (PASTE-367-M4); §527, which adopts it; §528, which adds E3 to it; §531, which seals E2 with its
  suite in the tree and re-measures the table; §532, which takes the photo of the pristine prover
  (D-R); §533, which puts the fork into the tree, switched off (E3a-1); and §534, which closes E3a:
  m in the mark, the statics out, the switch-on in the prover's API and the seven falsifiers of
  D-K as tests (E3a-2, D-S to D-W); §535, which opens E3b with the salt as a type, switching
  nothing on (E3b-0, D-X to D-AC); §536, which gives the photo its own provers and judges (E3b-1,
  D-Y); §537, which makes the hidden prover return `Err`, not panic, with a bad witness (D-AH);
  §538, which switches hiding on in the 23 provers with a row and raises the wire to `zkssl/0.4`
  (E3b-2, D-AD to D-AG and D-AI to D-AK); §538-B, §538-C and §538-D, which bring the prose, the
  comments of the `.rs` files and the spec up to date; and §539, this turn to ACCEPTED.

## Status of the stages

| stage | what it delivers | breaks the wire? | status |
|---|---|---|---|
| E1 — the promise, written | this text: what is promised (D-A), what comes out literally in each proof (D-B), the API principle as a rule that today is not met (D-C) and hiding outside this RFC (D-D) | no | sealed in §527 |
| E2 — the table's witness | a suite that produces every type of proof and counts its literal values against the table of D-B, with a control that has to give zero (D-E; its form, D-L to D-Q) | no | sealed in §531 |
| E3a — the prover that hides, inside and switched off | the fork of winterfell 0.13.1 in the tree, with the whole of hiding in the core and without touching an AIR (D-F to D-J); switched off, every proof comes out byte for byte like winterfell's; and the spikes' falsifiers as tests of the tree, with the hidden mode only in the tests (D-K); and before that, the photo of the pristine prover that the switched-off fork has to reproduce (D-R) | no | sealed: cut 0, the photo (D-R), in §532; cut 1, the switched-off fork, in §533; cut 2, m in the mark (D-S, D-T), the statics out and the switch-on in `Prover::ocultacion` (D-U) and the seven falsifiers of D-K as tests (D-V), in §534. The salt of D-I is not the fork's: it is the consumer's `VC` and goes with E3b (D-W) |
| E3b — switching it on | the proofs that cross the wire come out hidden, with the salt as the `VC` of the provers and of the kit (D-W); the table of D-B goes to zero, with the E2 suite as witness (D-K) | yes: `zkssl/0.4` | SEALED: cut 0 (§535, the salt as a type, D-X to D-AC), cut 1 (§536, the photo with its own provers and judges, D-Y), D-AH (§537, `Err` with a bad witness) and cut 2 (§538: the 23 provers with a row switched on with `MerkleConSal` and m 64, D-AD to D-AG; `zkssl/0.4`; the 0.4 vectors from their benches and the 0.3 ones under `spec/vectors/0.3/`; the band of D-AC; the comparator of D-AI; the table of D-B to zero: 22/22 and 100 cells, D-A reverted) |

The measurements that opened this document are those of entries §521, §523, §524 and §526: pure
readings that restored the tree with sha and porcelain, with their instruments outside the tree,
in the author's Downloads, like those of the earlier RFCs. Since §531 the table of D-B is measured
by the tree on every canon run: the E2 suite, `crates/zk-ssl/src/instrumento_revela.rs`.

## Motivation

The house STARK proofs are produced with winterfell 0.13, and winterfell does not hide the
witness: its front page puts perfect zero knowledge among what it expects to add, not among what it
offers, and its issue 9 says so bluntly. Four pure readings measured it in sessions 162 and 163
(§521): every proof opens 42 rows —42 is the number of queries of `ProofOptions`— and what those
rows carry is published. The spend key comes out 42 times in SEND-v1, SEND-v2, CLAIM and PRENDA.

Session 163 found the rule that explains it and subjected it to two falsifiers that could have
knocked it down (PASTE-360-M2, §523): **a value comes out LITERALLY in a proof if and only if its
column is CONSTANT in the trace**. What is opened are not rows of the trace but of its extension,
evaluated on a shifted coset, and there only a constant is worth the same as in the trace. With
that rule, sessions 163 and 164 measured everything that is STARK (§523, §524 and §526). The merkle
path, which carries nothing constant, gave zero (§524).

SOUNDNESS does not fall: every proof still proves what it says. HIDING falls, and with it the
custody of the key against whoever sees a proof —the node receives them all— and privacy against
whoever reads a published envelope. The cuts §521 to §526 stopped asserting the opposite in the
prose and in the comments. What is missing is saying, in a normative place, what is promised: that
is this RFC.

## Design

The five decisions were taken by the assistant by delegation from the author (session 165), with
the decision constitution (purity, clarity, coherence, faithful image, in that order). All carry
their reversal condition, written here.

### D-A — Soundness is promised, not hiding

A proof of Arqueo that verifies demonstrates the transition it states. **It does not hide its
witness.** The rule that is promised is the measured one, in its own words: what goes in a CONSTANT
column of the trace comes out LITERALLY in the proof; what is not constant does not come out
literally, but nothing guarantees that it cannot be deduced from what is opened, because the prover
does not mask. «Does not prove» is not «does not reveal», and «does not come out literally» is not
«hidden».

**Residual trust, in one sentence:** whoever receives a proof —and the node receives them all— sees
what the table of D-B says, and nothing in this document stops them deducing more.

Two paths were weighed: (a) promise only soundness and tabulate what comes out; (b) promise what
does not come out literally as if it were hidden. (a) wins: faithful image (what the prover does
not hide is not hidden) and purity (the rule is the measurement, not a wish). **Reversible** only
when a prover that hides proves it with its own falsifier (D-D); never because a value stops coming
out literally.

**REVERTED in §538 (E3b-2), by the path this very decision left written.** Since §538 the prover
hides: the 23 provers with a row in the table of D-B produce with `Ocultacion { m: 64 }` and seeds
from the system's entropy (D-Z, D-AE), and the own falsifier that D-A and D-C demanded —the E2
suite against a prover that hides— counts ZERO in the 100 cells with k > 0 of the table, in the 22
rows that print, on every canon run (PASTE-538-M, R4 and R9a; the tree since §538). What is
promised now, in its own words: **a proof of Arqueo that verifies demonstrates the transition it
states and publishes literally no value of a constant column of its trace.** What is not promised:
that nothing can be deduced from what is opened. The construction (random rows and blinded
quotient, note 2024/1037, with the bound of D-I: 44 openings per column against T ≥ 64 random
rows) is what sustains hiding beyond the census, and neither the fork nor it are audited (H7).

**Residual trust, in one sentence:** whoever receives a proof —and the node receives them all— sees
the public statement, the roots and what the envelope, the receipt or the operation carry in the
clear by design (the «public by design» column of the table is still true); of the witness they see
no literal, and what they may deduce from the openings is bounded by D-I, not measured by E2.
⚠️ §915: what could be promised with the construction of `zkssl/0.5` for the twenty-three families
—hiding simulatable without the witness, under named hypotheses— is written in D-AM (its draft of
§915) and **is not promised**: this decision does not change until the author accepts it.

**ACCEPTED in §916 by the author, on 29 October 2026 («Aceptar todo» [accept everything]): the
promise of §915, for the twenty-three families with `zkssl/0.5`.** What is promised since §916, in
its own words, in addition to that of §538:

> For the twenty-three families with a row in the table of D-B, with the construction of `zkssl/0.5`
> as it stands —T random rows per column with T ≥ 128, two paired random columns in DEEP as one over
> E, the quotient blinded with m = 64 and the leaves salted—, the view of the honest verifier is
> simulated without the witness: for every real trace the AIR accepts, the view and the output of
> the simulator of section 5 of `doc/ocultacion-hvzk.md` are statistically indistinguishable in the
> programmable random oracle model, with advantage less than 2^{−60} plus 2^{−128} plus Q·2^{−256}
> plus |E|^{−(N−1)} for Q queries to the oracle and N pieces of the quotient (2 in solvency, 8 to 10
> in the others): simulatable hiding, statistical in that model and computational in what H7, Blake3
> as an oracle, has of computational. Under the hypotheses of section 4 of that document with the
> cut (its section 18): those measured on every canon run (H1, H4 and H11, in the twenty-three),
> those proved (H2, H5, H10 and, since the cut, H6 by L9) and those assumed or of the model, which
> no test measures: H3, z outside F and H and without coincidences of conjugates, with failure
> bounded per proof; H7, the programmable random oracle; H8, the fresh salt; H9, no seed of the
> hiding repeated. No family out by its name, and no hypothesis specific to an AIR. And under the
> argument as it is written: reviewed adversarially by the session and by an agent, and by no person
> outside the project.

What is not promised: nothing against a quantum adversary («not measured»); nothing of the prover's
side channels; nothing outside the named hypotheses —H3, H7, H8 and H9, which no test measures— nor
outside the random oracle model; and the outside review of the argument is not done: H7 is kept
—neither the fork nor the construction are audited— and entry 136 of `BACKLOG.md` is still open,
with the promise saying so. Soundness is promised as before. **Reversible** if the outside review
finds a false step that the measurements do not see: then the promise goes back to the census with
an entry by the author, and it is said here.

**Residual trust, in one sentence:** whoever receives a proof —and the node receives them all— sees
the public statement, the roots and what is public by design; of the witness they see no literal,
and what the view shows them is simulated without the witness under those hypotheses: whatever they
may deduce, they would deduce just the same from the simulator's output.

### D-B — The table: what comes out literally, proof by proof

With the rule of D-A and with a control at zero. Since §531 every row is measured, on every canon
run, by its test in `crates/zk-ssl/src/instrumento_revela.rs` (E2), with the unit of D-M: an
element of a constant column comes out k(q+2) times and a digest k·q, with q the unique positions
the proof opens and k the columns or blocks that carry it. The table says «in two columns» or «in
two blocks» where k is 2 and «nothing» where everything is 0. What is public by design —what the
circuit declares in its public inputs or what the envelope, the receipt or the operation carry in
the clear— comes out just as literally and goes in its own column (D-Q); what is small —nonces,
counters, born-at values, lower bounds— is not counted.

| proof | from the witness, comes out literally | public by design, and comes out the same | does not come out | measured in |
|---|---|---|---|---|
| send (SEND-v1 and SEND-v2) | the spend key, the sender's identity, the receiver's identity, the balance before and after, the salt and the `leaf_salt`; in v2, also the envelope `X` | the amount, the regulatory limit and the supply, the latter in two columns because the send does not move it | the view key; in v2, the `refund_id` and the `delta` | §521, §523; §531 |
| claim (CLAIM-v1 and CLAIM-v2) | the spend key, the receiver's identity —in two blocks: the account's and the pending one's, which the circuit forces to be the same—, its balance before and after, the salt and its `leaf_salt`; in v2, also `X` | the amount and the supply, in two columns | the payer's identity; in v2, the `refund_id` and the `delta` | §521, §524; §531 |
| pledge | the spend key, the salt, the amount and `X` | the pledger's identity | — | §521; §531 |
| issuance to pending | the receiver's identity and the salt | the amount, the supply before and after and the supply maximum | — | §524; §531 |
| delegated authorisation by a custodian (issuance, freeze, recovery) | the key of ITS custodian: after a single delegated operation, the node has two and can authorise the next one | the authorised operation | the other custodian's key and that of the one who does not sign | §523; §531 |
| delegated governance | each member's key | the authorised operation | the other member's and that of the one who does not sign | §524; §531 |
| joint threshold (`circuit_threshold`) | both keys | — | that of the custodian who does not sign | §526; §531 |
| audit (the layer's `prove_minimum`, with the `stark-experiment` circuit) | the spend key, the exact balance and the `leaf_salt` | the account's identity, the threshold and the ceiling (`2^62 - 1`) | — | §523, §524; §531 |
| burn | the spend key, the balance before and after, the `leaf_salt` and the account's identity | the amount and the supply before and after | — | §523; §531 |
| solvency (`stark-experiment`) | the balance and the amount | the limit | — | §524; §531 |
| `double_entry` (`stark-experiment`) | the sender's and the receiver's identities, the four balances, the amount and the two nonces | the limit | — | §526; §531 |
| band | the balance and the `leaf_salt` | the account's identity and the upper bound (`pedido - 1`); with the request at the balance plus one, balance and bound are worth the same and the count doubles: it is the declared coincidence of D-Q | — | §521, with its correction in §527; §531 |
| age | the sender, when all the pending ones are from the same one | — | with distinct senders, none; neither the `nacido` nor the leaf of the pending ones | §521, §523; §531 |
| claim envelope (portable) | the salt, `X`, the exact amount and the sender, which is the index of its account | the receiver's identity and the band's ceiling; the lower bound and the `nacido` too, but they are small | — | §521; §531 |
| payment envelope (portable) | the salt, the `delta`, the `refund_id` and the sender, the index of its account | the amount | `X` | §521; §531 |
| merkle path (`stark-experiment`) | nothing: neither the leaf nor the siblings are constant | — | the leaf and the siblings | §524; §531 |
| opening of the refund and of the de-issuance (v1 and v2) | nothing: its trace carries no constant columns | the amount and, in v2, the opening (`refund_id`, `delta`) go in the receipt, not in the proof | the receiver's identity, the salt and the amount; in v2, the `refund_id`, the `delta` and `X` | §531 |
| the refund's credit climb | the identity of the account that recovers the money, its balance before and after and its `leaf_salt` | the amount | — | §531 |
| the delegated issuance climb | the account's identity, its balance before and after and its `leaf_salt` | the amount, the supply before and after and the maximum | — | §531 |
| the delegated freeze climb | nothing | — | the account's index and the frozen flag | §531 |
| the delegated recovery climb | the old identity, the balance and the `leaf_salt` | the new identity, which the operation names | — | §531 |

The rejection catalogue carries TWO real proofs —a SEND and a CLAIM from a bench— from which the
spend key of its two sandbox accounts is derived (§521). They are test keys: what was false was the
published property, not a leak that happened.

The table wins over a general sentence: clarity (a reader can say what is public, proof by proof)
and coherence (every row refers to the entry that measured it). **Reversible** row by row: a new
measurement that contradicts it corrects it, in the entry that measures it, here and in the suite,
which is the table as data (D-O). §531 already did so with three cells that PASTE-E2-M read as «two
columns» and were coincidences with the supply (D-Q).

**Since §538 (E3b-2) the table measures zero.** The 100 cells with k > 0 above —what came out
literally with the prover switched off— count 0 in the 22 rows that print: the E2 suite passes
22/22 with its table at zero as data (`instrumento_revela.rs`, S538), and a cell that counted again
would turn it red. The table is kept as it is, as what 0.3 revealed, measured entry by entry: a
vector is what its version produced, and the row of its truth is the suite, not this text (D-O).

### D-C — The API principle stays as a rule, and today it is not met

Rule 3 of the PROCESO —**the spend key never travels**— is not rewritten. Two paths: (a) keep it
and declare where it is breached; (b) lower it to «does not travel as a field of the wire», which
would make it true without anything changing. (a) wins: (b) embellishes the narrative and dirties
the invariant, and the constitution rejects it. The table of D-B says where it is breached today:
in the send, the claim, the pledge, the audit and the burn the spend key travels; in the delegated
authorisations, governance and the joint threshold, the custodians' keys. The PROCESO says so since
§521; here it is measured proof by proof.

This RFC does not erode the principle —the prover erodes it— and that is why it is not born
WITHDRAWN. **Reversible** towards nowhere: a rule does not expire. It stops being breached when the
E2 suite proves it against a prover that hides.

**It stops being breached in §538 (E3b-2), and in exactly the measure in which the suite measures
it.** The E2 suite counts zero literals of the witness in the 23 proofs with a row (100 cells,
22/22): the spend key does not come out literally in the send, the claim, the pledge, the audit or
the burn, nor the custodians' keys in the delegated authorisations, governance and the joint
threshold. Rule 3 of the PROCESO says so since §538, and what lies outside the measurement is said
by D-A: the residual trust is in the openings and in the construction, not in the census.

### D-D — Hiding, outside this RFC

This RFC says what there is; it does not choose how to change it. Hiding the witness asks for a
prover that masks: winterfell's issue 9 lists what would be needed —random values at the end of the
trace, a random polynomial of low degree combined with the composition one and Merkle commitments
with salt—, and that touches every AIR of the house. It is a new primitive for a new property, and
it goes in an RFC of its own. What other provers declare is read on their front page before
asserting it, and that is the work of that RFC; RISC Zero, besides, is a zkVM, which the
constitution leaves out.

Leaving it out wins: purity (one primitive per property) and the order that entries §521 to §526
followed —first stop asserting what is false, then say what is promised and only then change the
prover—. **Reversible** if a prover that hides turns out to fit without touching the AIRs: then it
fits here as a stage.

**Reverted in E3** (session 166): the prover that hides fits without touching any AIR —measured in
two toys outside the tree, and for the 35 of production by their census and the formulas read; E3a
and E3b confirm it with their tests (D-G, D-K)—, so hiding enters here as a stage and not in an RFC
of its own. The new primitive for the new property is the fork, and D-F declares it: one primitive
per property, as purity asked.

### D-E — The table will have its witness: E2

A promise that nobody re-measures expires in silence. E2 turns the table of D-B into a suite: it
produces every type of proof with keys and values of high entropy, counts their appearances in the
bytes and compares with the table, with a control that cannot appear and has to give zero. A new
circuit that puts a secret in a constant column, or a change that moves a row, turns it red. Until
then, the table is worth what the readings that measured it are worth.

The suite wins over leaving the table as prose: purity (witness before promise) and coherence (the
same rule that every circuit's suite already follows). **Reversible** in its form —a test of the
crate or an instrument of the canon—, not in its existence.

**Sealed in §531**: `crates/zk-ssl/src/instrumento_revela.rs`, twenty-one row tests and the census
of the wire, in the layer's row of the canon. Its form is fixed by D-L to D-Q.

### D-F — The prover that hides is a fork of winterfell 0.13.1

Two paths were weighed (INFORME-PROBADOR-165, corrected in session 166): (b) fork winterfell 0.13.1
and hide inside; (c) migrate to Plonky3, which brings `HidingFriPcs`. (c) reopens the 35 production
AIRs —10,100 lines of implementation, 4,341 of transition— and leaves out `EdadAir`, whose
auxiliary segment p3-uni-stark does not have. (b) measures 488 lines —350 in the fork and 138 of
our own, the salt and the seeded randomness— and none in an AIR, against a reference of 892 to
1,697 lines of Plonky3 for the same thing (SPIKE-B-P4 r2).

(b) wins: purity (the 35 AIRs keep the semantics their suites already prove) and faithful image
(the cost is the measured one, not the expected one). **Reversible** if the fork's code exceeds
twice the reference (1,784 lines), if an audit knocks it down or if winterfell publishes hiding of
its own: then winterfell is followed and the fork is withdrawn.

How it enters (§533): through `[patch.crates-io]` from the root `Cargo.toml`, with the names and
the version of crates.io —it is what a `[patch]` demands, and what the H2 gate measures by name—,
under `crates/winter-air`, `crates/winter-prover` and `crates/winter-verifier` as members of the
workspace with their row in the canon (49, 6 and 0 tests, upstream's), `publish = false`,
winterfell's MIT license in each crate and seven files different from the published ones plus one
new one, listed in their READMEs. It replaces winterfell across the whole workspace, kit included:
`winter-fri`, `-math`, `-crypto` and `-utils` are still those of crates.io.

### D-G — Hiding lives entirely in the core: no AIR changes

Winterfell's issue 9, which its author opened in 2021, asks for three pieces; note 2024/1037 adds a
fourth. All four fit inside:

- **the rows** (the first of the issue): behind the T real rows, T random rows. A wrapper,
  `Oculta<A>`, presents to the core an AIR of 2T rows with one more column and the inner one's
  exemptions plus T, and delegates to the inner one transitions, assertions, auxiliary segment and
  periodic columns. The inner one still sees T, so its last-row assertions do not move. In the
  thread of the issue, its author already took this piece as covered with barely a touch to the
  design of the constraints;
- **the random polynomial before FRI** (the second): the extra column is random throughout; it
  enters DEEP with its coefficient, like any column, and masks what FRI opens;
- **the leaves** (the third): the Merkle commitments carry salt (D-I). The issue asks for it for
  the trace; the salt is put in by the commitment —`MerkleConSal`, the `VC` that prover and
  verifier receive as a type, on the consumer's side and not the fork's (measured in session 172:
  the tree's fork carries no salt)—, and that is why it salts the trace, the constraints and FRI at
  once; it enters in E3b with the change of `VC` (D-W);
- **the quotient** (note 2024/1037, section 4.2, which the issue does not bring): the composition
  polynomial is split with step 2T − m and its pieces are randomised with polynomials of degree
  less than m that cancel in the sum, with m ≥ 44 —the 42 queries, z and z·g— and m = 64.

That no AIR changes is sustained by the census of the 35 (PASTE-P4-M and PASTE-GRADOS-M r2): none
fixes exemptions of its own —all carry the stock one, and 1 + T over 2T is exactly the maximum
winterfell admits (`air/context.rs`:302-307)—; none uses periodic or sequence assertions; none
reads the width of the frame; none writes into the trace's meta; none defines a method of the AIR
that the wrapper does not delegate; and the 35 provers use the stock evaluator.

There remains the second bound of the exemptions (`air/context.rs`:315-327), which depends on the
degree. With the formulas of `air/transition/degree.rs`:90-115, an AIR of 2T rows with T + 1
exemptions fits if d + Σ(1 − 1/c) ≤ ce + ½. 34 of 35 fit with their own ce; `WorkAir`, of degree 3
without cycles, needs 4 where winterfell gives it 2. The wrapper raises the ce to the smallest
power of two that leaves room, capped at the blowup, and computes it with winterfell's functions.
The ce is used only by the prover: measured in a toy with the shape of `WorkAir`, a proof made with
the raise verifies without it, and without the raise no proof that verifies comes out (SPIKE-B-P4
r2).

The core wins over hiding from outside, which is what stage 1 of the spike did: coherence (the
suites and the pins of the 35 AIRs do not change) and purity (one primitive, one place).
**Reversible** AIR by AIR: one that does not fit —exemptions of its own, periodic or sequence
assertions, or a ce above the blowup— says so in its RFC and is hidden from outside.

### D-H — The mark travels in the trace's meta, with m inside

A hidden proof says so in the meta of its `TraceInfo`, which no AIR uses. With the meta empty, the
proof is winterfell's byte for byte: the switched-off fork reproduces its bytes (measured). With
the mark, the meta enters the transcript, and touching one byte of the mark leaves the proof
unverified (measured). The verifier dispatches by the mark without a single call to `verify`
changing. With the mark and a trace that cannot be split in two —length less than 16 or width less
than 2— it returns an error without reaching the wrapper, because the kit has to fail closed
(measured). The m of the quotient travels inside the mark; in the spikes it lives in a static.
Before using them, the verifier checks that it knows the version of the mark and that 0 ≤ m < 2T,
and if not, it rejects. m decides hiding, not soundness: a short m only harms whoever proves.

The meta wins over a new field of `ProofOptions` or of the wire: purity (the unhidden proofs do not
change a single byte) and coherence (the mark goes tied to the same transcript as everything else).
**Reversible** if some AIR comes to need the meta: then the mark moves, and it is said here.

What the tree carries since §533, and was not yet this: the fork entered with the spike's statics
—`COCIENTE_M` and `SUBIR_CE` in `winter-air`; `OCULTAR_FILAS`, `SEMILLA_FILAS` and
`SEMILLA_COCIENTE` in `winter-prover`—, switched off at their birth value, and the verifier read m
from `COCIENTE_M` and dispatched by the mark without version or m inside; with the mark and m = 0,
the kit verified a hidden proof as it was (measured in session 172: PASTE-E3a2-M, and m did not
travel: the proof with m = 64 carried the same 21 header bytes as the one with m = 0, and verified
or not depending on the static). Since §534 it is this: the mark carries m (D-S), a single place
reads it (D-T) and the statics do not exist (D-U).

### D-I — The salt stays until an argument withdraws it

The salt costs 6,567 bytes per proof, 46 % of what grows on hiding; without it, the hidden proof
weighs ×1.12 winterfell's instead of ×1.22 (SPIKE-B-P4 r1). The literal census cannot judge it,
because it does not see the leaves. The argument that withdraws it carries two separate counts: per
column, 44 opened points —the 42 queries, z and z·g— against T random rows; and in DEEP and FRI,
the combinations that open the layers against the random column that masks them.

Keeping it wins: purity (nothing is withdrawn without its argument). **Reversible** when the
argument is written and reviewed: the salt goes and the proof slims down.

> ⚠️ **§896 (A1 of the HVZK): the count above, measured per family, and one more opening.** The
> OOD frame also carries the pieces of the quotient at `z·g` (`QuotientOodFrame`), so the 44
> openings seen are per column AND per piece; and DEEP and FRI let one more direction through —the
> random column enters DEEP as `c·u(x)·(2x − z − z·g)`, with `u` uniform of degree 2T − 3, and
> masks everything except the root of that factor—, which a simulator has to count: 45 to mask.
> Measured over the 28 real proofs of the 23 provers with a row, on every canon run
> (`crates/zk-ssl/src/instrumento_cota.rs`): `q` from 40 to 42 unique positions, 42 to 44 seen and
> 43 to 45 to mask, against T of 64, 256, 512 or 1,024 random rows and m = 64 in all; the Lagrange
> matrices of the T random rows and the Vandermonde ones of degree < m at those points, of full
> rank; and two named mutants, m = 43 and T = 32, in red. The argument this paragraph asks for is
> still unwritten: it is A2, and until A4 this decision and D-A do not change what they promise.
> ⚠️ §915: since the `zkssl/0.5` cut the two counts of this decision are done for the twenty-three:
> per column, 2q + 6 = 90 touched points against T ≥ 128 random rows (L1); in DEEP and FRI, the two
> paired columns with the randomisers cover the whole of K (L9, `K_ENTERO_PAREADAS`). The argument
> is written (`doc/ocultacion-hvzk.md`, section 18) and without outside review; the salt stays where
> the author left it (§902), and this decision does not move until he decides it with the count of
> the leaves in front of him.

> ⚠️ **§897 (A2 of the HVZK): the count of §896, corrected, and the argument, written.** The «one
> more opening» of §896 is not one: the value of DEEP at the root of the linear factor is computed
> from the OOD frame, it is public, and there is nothing to mask in it. The right counts are per
> field: what is opened of a main column is q + 4 dimensions over F —the q positions live in F; z
> and z·g in E—, 46 with q = 42, against the T random rows, which live in F; what is opened of a
> piece, q + 2 points of E, 44, against the m coefficients of E of each randomiser. And what is
> opened is not everything that is touched: the sum of the pieces at each position is worth C(x_j),
> which reads the next row, and that of z·g is worth C(z·g), which reads g²·z, so the random rows
> have to cover 2q + 3 points per column, 2q + 6 = 90 dimensions over F. The 18 families with
> T ≥ 256 meet it and the five with T = 64 —the two thresholds, the two refunds and age with eight
> leaves— do not, measured on every canon run since §897, and for them there is no argument
> independent of the AIR (entry 131 of `BACKLOG.md`). That the random column «masks DEEP in every
> direction but one» is not true in general either: FRI opens from DEEP the whole folding cosets,
> the layers and the remainder, and that the masks that do not depend on the witness —the random
> column and the quotient's randomisers— cover that is a rank condition nobody has measured (R3),
> and which fails solvency, with two pieces, by count (entry 132 of `BACKLOG.md`). The argument,
> with its simulator, its lemmas and the list of what is measured, what is of the model and what is
> assumed, is `doc/ocultacion-hvzk.md`; this decision and D-A still do not change what they promise
> until A4.

> ⚠️ **§898 (A3 of the HVZK): R3, measured, and not met in any production geometry.** What FRI
> opens of the DEEP polynomial without determining it is 168, 632, 760 and 1016 dimensions over F
> with T = 64, 256, 512 and 1024, and the two masks that do not depend on the witness reach 126,
> 378 to 382, 510 and 766 (`crates/zk-ssl/src/instrumento_r3.rs`, on every canon run, with its
> figures pinned). The reason is structural: the random column lives in F and DEEP in E, so in each
> value that FRI opens it moves only one of its two coordinates; and the quotient's randomisers,
> which all enter through the same two gaps, contribute 80 dimensions in total. Today the argument
> independent of the AIR in `doc/ocultacion-hvzk.md` covers no family: whatever sustains hiding
> beyond the census depends on the AIR —on a linear mask of its own, where its columns enter the
> constraints only linearly, which entry 134 of `BACKLOG.md` will measure, or on the witness—, and
> nobody has argued it. The AIR-independent way out is for the DEEP mask to live in E —the random
> column in the auxiliary segment, or a random polynomial of E[x] of the degree of DEEP—, in a
> future cut (entry 133 of `BACKLOG.md`). Three falsifiers in a toy with the production geometry
> measure the rest: with T = 32 a constant column comes out by elimination and with T = 64 it does
> not; with T = 64 what is touched tells the witness between two candidates and with T = 128 it
> does not (entry 131); and the repeated seed returns two constants that each proof alone hides
> (why the seed is fresh, §652). This decision, D-A and the promise do not change: the census.

> ⚠️ **§900 (entry 134 of the HVZK): M_lin, measured AIR by AIR.** The random rows of the real
> columns that enter the constraints only linearly and with a public coefficient —constants,
> periodics or the random elements of the auxiliary segment— are one more mask, independent of the
> witness, which §898 did not count. `crates/zk-ssl/src/instrumento_mlin.rs` probes the 23 AIRs on
> every canon run —which columns enter that way, which with degree ≥ 2 and which linearly but with
> the slope of another column— and measures R3 with it per family, with the same O, the same K and
> the same challenges as §898: in sixteen of twenty-three —the six of T = 1024, the nine of T = 512
> and solvency— R3 is met with today's construction, and the linear ones with the randomisers are
> enough on their own; MerkleAir and FrozenClimbAir (T = 256, the sbox in all their columns) have
> none and stay with the R3 of §898; the five of T = 64 have no M_lin to measure and stay in entry
> 131. It is a lower bound: the instrument conditions each linear column on its 2q + 6 touched
> ones, more than what the verifier sees of it (entry 135 of `BACKLOG.md`). Who needs entry 133 for
> R3: those seven; the other sixteen do not. The argument, the census and the figures, in section
> 14 of `doc/ocultacion-hvzk.md`. This decision, D-A and the promise do not change: the census,
> until A4.

> ⚠️ **§902 (A5 of the HVZK): decided by the author: the salt stays.** The argument is written
> (§897) and measured (§898 and §900), and it recommends it: it is what makes lemma L4 of
> `doc/ocultacion-hvzk.md` unconditional; without it, the leaf of a FRI coset would have entropy
> only through R3, which is green in sixteen families and not in seven. Withdrawing it would break
> the wire and would belong to the cut; it does not enter 0.5. The reversibility of this decision
> still stands: with the outside review of the argument (entry 136 of `BACKLOG.md`) the author will
> be able to weigh it again with the count of the leaves in front of him.

> ⚠️ **§905 (entry 135): the fine count, measured, rescues none of the five families of T = 64.**
> Lemma L1' —conditioning each linear column on what the verifier sees of it, q + 4, and asking that
> what C sees of those columns in S_ab be a surjective map, Λ— is written and measured
> (`doc/ocultacion-hvzk.md`, section 15): Λ is only surjective in the joint threshold, and there R3
> with the fine basis is ten dimensions short; in the other four Λ does not reach, or there are no
> linear columns. What it leaves for the cut: with entry 133 and L1', the joint threshold is covered
> without more rows; the delegated authorisation, age and the two refunds need entry 131.

> ⚠️ **§914 (the `zkssl/0.5` cut, RFC-0018 D-H): the five of T = 64 move to T = 128.** L1 covers
> them by geometry (2q + 6 = 90 ≤ 128), `INDEPENDIENTE_DE_LA_AIR` says «cubre» [covers] in the 23,
> and the hiding bound (A1) is measured green in the 28 proofs of the 23 provers, with the v2 mark
> and the two paired columns (D-AL bis). Entry 131 is closed by T = 128; the fine count of §905 is
> no longer run in any family.

### D-J — The byte figures become bands and the randomness comes from the system

A hidden proof is not deterministic: with another seed it weighs differently (E1, E2 and E4 of
SPIKE-B-P4: 80,936, 81,637 and 80,807 bytes). The published byte figures become bands; the vectors
of `spec/vectors/` need seeded randomness or a watch that does not compare bytes; and in production
the randomness comes from the system's entropy.

What was measured without grinding and against winterfell: ×1.22 in bytes with 42 columns, ×3.2 to
×3.3 when proving and ×1.9 to ×2.0 when verifying; with 5 columns, ≈ ×1.29 in bytes (a single
seed). Against hiding from outside, the same: ×0.98 in bytes and ×1.05 when proving.

The band wins over a figure: faithful image (a single figure would lie). **Reversible** towards
nowhere as long as there is hiding.

### D-K — Two steps: the fork enters switched off, and switching it on raises the wire

E3a puts the fork into the tree switched off: every proof comes out byte for byte as today and the
wire does not change (measured with the fork switched off in the spikes). It brings, as tests of
the tree and with the hidden mode only in the tests, the spikes' falsifiers: zero census, verifies,
rejects public+1, touched mark, invalid dimensions, `WorkAir` with the raise and the regression
with the fork switched off. Without them, E3a would seal code that nothing exercises: negative
witness before the function. Since §534 the seven are tests of the tree,
`crates/stark-experiment/src/falsadores_oculta.rs` (D-V).

E3b switches it on for the proofs that cross the wire. A hidden proof is not verified by an
unforked winterfell (reasoned; illustrated by 2a of the spike, where the verifier without the
quotient step rejects the proof, and in the AIRs that assert their width in `new` it would panic
instead of rejecting). So the version rises to `zkssl/0.4` and the 0.3 vectors are kept under their
version (rule 2 of the PROCESO).

E3b is not sealed without the E2 suite counting zero literal values in every type of proof. That
is the own falsifier that D-A («only when a prover that hides proves it with its own falsifier»)
and D-C («when the E2 suite proves it against a prover that hides») ask for. With it, the table of
D-B goes to zero and D-A is reverted.

Splitting in two wins: coherence (the order entries §521 to §526 followed: first what does not
break, then what does) and clarity (every step with its witness). **Reversible** in its order if
E2 is delayed: E3a can be sealed alone; E3b cannot.

### D-L — The suite's universe is the wire, and a census read from the sources says so

What the suite produces and the table tabulates are the AIRs the node verifies —the thirteen of
the layer, the `verify::<` of `crates/zk-ssl/src` outside the instruments, and that of the
threshold pair, the one `verify_threshold_pair` verifies— and the five of the kit
(`crates/zk-ssl-air`), which an envelope publishes: nineteen. The four experimental ones that were
already measured —`ThresholdAir`, `SolvencyAir`, `DoubleEntryAir` and `MerkleAir`— stay, outside
the wire and said so. The census is read from the sources on every run, not typed: a new AIR on
the wire without a row, a row of the wire that nobody verifies any more or a row that names an AIR
that does not exist turn the suite red.

The census read wins over a list: purity (the universe is the scope of the change, and it is
re-measured) and coherence (the same census with which PASTE-E2-M opened E2). **Reversible** if the
wire stops being readable from the sources: then the list is written and the census crosses it.

### D-M — The unit of the count: k(q+2) and k·q, with q read from each proof

An element of a constant column comes out k(q+2) times —one per opened position and two outside
the domain, at z and at z·g— and a digest of four contiguous columns k·q, because outside the
domain the quadratic extension interleaves it. q are the unique positions the proof opens
(`num_unique_queries`: from 38 to 42 measured, and the 42 of `ProofOptions` are the ceiling) and k
the columns or blocks that carry the value. Measured in PASTE-E2-M over 107 values without
exception (§531). The suite reads q from each proof and demands the exact count: no cell drops to
«at least».

The exact unit wins over a bound: purity (a count that holds without exception is a rule) and
faithful image (a «42 or more» would say less than what is measured). **Reversible** if a prover
changes the shape of the proof: then the unit is re-measured and this decision follows it.

### D-N — One test of the `zk-ssl` crate per row, in release and inside the canon

The suite lives in `crates/zk-ssl/src/instrumento_revela.rs`, tests only, and runs with the
layer's row of the canon (`cargo test -p zk-ssl --release`): twenty-one row tests and the census.
The STARK proofs are skipped in debug (note 41) and `--release` runs them; the census always runs.
A red gathers all the cells that moved and names them.

The crate test wins over an instrument apart from the canon: coherence (the same gate that gates
every circuit) and clarity (one row, one test, one name). **Reversible** if its cost stops fitting
in the row (today it is 6.6 s of tests, §531): then it moves to `--largo`.

### D-O — The table has twenty-one rows, and its source is the suite

The seven AIRs the census found without a row enter like this: `RefundAir` and `RefundAirV2` as
one row —the opening of the refund and of the de-issuance, the same constructor—, `CreditClimbAir`,
`MintClimbAir`, `FrozenClimbAir` and `RecoveryClimbAir` as one row each —the climbs of the refund
and of the delegated operations—, and `ClaimAirV2` as a variant of the claim row, as SEND-v2
already was of the send one. Every row says what comes out of the witness, what is public by
design —and comes out the same— and what does not come out; the unit is that of D-M. The table as
data (D-K): every row of the suite is a list of cells with their k, and E3b sets it to zero by
changing data, not code.

Rewriting it wins over adding rows to the old one: clarity (a reader can say what is public and
what is witness, row by row) and faithful image (the table says what the suite measures, with the
correction of §531). **Reversible** row by row, as before: the entry that measures it corrects it,
here and in the suite.

### D-P — The two proofs of the rejection catalogue stay outside the suite

The rejection catalogue carries a SEND and a CLAIM from a bench (§521). The suite does not count
them: they are vectors, not proofs the suite produces, and their row is already there —send and
claim—. They are still stated in D-B and in Security.

Leaving them out wins: purity (the suite produces what it counts) and coherence (the scope of the
census is the scope of the change). **Reversible** if the catalogue gains proofs of a type without
a row.

### D-Q — Every large scalar of a constant column is a cell, and cells are worth distinct values

What the suite counts are values, not columns: if two distinct values of the trace were worth the
same, the count of one would count the other. That is how PASTE-E2-M read «two columns» in the
burn and in the issuance to pending, where the supply —public, and in a constant column— was worth
what the balance or the amount of a ledger with a single account was; ENSAYO-531 caught it in the
send, with the supply equal to the sender's balance (§531). Hence the rule: every row also counts
the public scalars the circuit carries in a constant column —the supply, the limit, the ceiling,
the maximum—, the suite demands before counting that no cell that comes out is worth what another
is, and every set-up chooses values distinct two by two. What is small —nonces, counters, born-at
values, lower bounds— cannot be counted by bytes and is not counted; the account's nonce goes in a
constant column in the send, the claim, the burn, the climbs with a balance, the audit and the band
—read in the `build_trace` of each circuit, not measured— and says how many operations the account
carries. The band with the request at the balance plus one is the declared coincidence that shows
why: balance and bound are worth the same and the count doubles.

Counting what is public and demanding the distinction wins over counting only the witness: purity
(a count that can count two things measures neither) and clarity (the reader sees in every row
what is public). **Reversible** if the count moves to reading columns instead of bytes: then the
distinction is superfluous.

### D-R — The photo of the prover before the fork

E3a promises that, switched off, «every proof comes out byte for byte like winterfell's». That
sentence needs its falsifier in the tree before the fork enters, because a photo taken with the
fork inside would be a green that certifies itself. Cut 0 of E3a (§532) takes it: three proofs with
fixed inputs —`WorkAir`, winterfell's canonical circuit; `BandaAir` and `EdadAir`, two AIRs of the
wire that the kit verifies without the prover—, produced twice over `81549de` with unforked
winterfell 0.13.1, identical between runs, verified by their judge, and with their size and their
blake3 written as constants in `crates/stark-experiment/src/kat_probador.rs` (PASTE-KAT-M): `work`
3,539 B, `banda` 49,418 B and `edad` 66,880 B. The set-ups are copies of the positives of their
tests, not references: the photo does not move because someone tunes a positive. The prover
without hiding is deterministic; the one that hides is not (D-J).

Taking the photo first wins over comparing against the spike's reference: faithful image (the
falsifier lives in the tree and runs on every canon run) and purity (a KAT with fixed inputs,
without randomness or fork). **Reversible** only with winterfell: if the pinned version changes,
the photo is taken again with its entry, and whoever moves it says why.

### D-S — The mark carries m: the prefix and four bytes, with a single reader

The mark is `arqueo:oculta:1` —the prefix, with its version inside— followed by m in four
little-endian bytes: 19 bytes in the trace's meta. A single place reads it,
`winter-air/src/marca.rs`: `Marca::leer` gives nothing with the meta empty (winterfell as it is),
the mark with the prefix and its four bytes, and an error with any other meta —another version of
the prefix, another prefix or a length that is not its own—. The verifier rejects that error before
building any AIR («meta de traza desconocido»), and with the mark it rejects, also before, a trace
of length less than 16, of width less than 2 or an m that does not fit in it (m ≥ 2T), all with an
error and without panic.

The prefix with the version wins over a separate version byte: clarity (the mark is read in a
dump) and coherence (that of E3a-1 was already `arqueo:oculta:1`). And rejecting every meta that is
not the mark wins over falling back to the usual AIR: fail-closed (measured in session 172: with
the mark touched, the switched-off fork fell back to the usual AIR with a trace of width 2 and
`WorkAir::new` panicked). **Reversible** if some AIR of the house comes to write into the meta
(D-H): then the mark moves and it is said here.
⚠️ §914: since the `zkssl/0.5` cut the production mark is the **v2** (RFC-0018 D-G):
`arqueo:oculta:2`, m, the family, the version of the AIR, the profile and the columns, 25 bytes,
with a single reader in the fork (`MarcaV2::leer`) and a single judge in the kit
(`comprobar_marca_v2`, which also demands the columns of the construction, two); the 19-byte v1 is
recognised only to name it —`marca v1: prueba de zkssl/0.4; la juzga el kit 0.4.x`— and is not
accepted. m still travels inside, and is still 64.

### D-T — A single producer of m: the trace's meta

m is said by the mark and only by the mark. The AIR's context reads it from the meta of its
`TraceInfo` to split the quotient into pieces of step s = L − m (`context.rs`); the prover's domain
reads it from the same meta when it is built and passes it to `segmentar` with the quotient's own
seed, which only the hidden prover sets (`domain.rs`, `composition_poly.rs`); and the verifier
reads it from the AIR's context it already built, for the step of the quotient's evaluation. No
signature the 35 provers implement changes: `CompositionPoly::new` already received the domain.

The meta wins over a process static: purity (what decides verification travels signed in the
transcript, and a proof with m = 64 is no longer accepted by a verifier «at 0» nor rejected by one
«at 64») and coherence (one producer, three readers of the same byte). **Reversible** with D-S.

### D-U — Switching on belongs to the prover: `Prover::ocultacion`, off by default

The `Prover` trait gains a provided method, `ocultacion(&self) -> Option<Ocultacion>`, which
returns `None`; `Ocultacion` carries m and the two seeds —that of the random rows and column and
that of the quotient, separate— and they are set by the caller: in production, from the system's
entropy (D-J). With `Some`, `generate_proof` diverts to the version that hides; with `None`, the
proof is winterfell's byte for byte, and no ARQUEO prover returns `Some` until E3b. The five
statics of the spike die, `SUBIR_CE` included: the wrapper raises the ce whenever it is needed,
and `Oculta::cabe_con` stays as a query for the falsifier.

The provided method wins over a new field of `ProofOptions` or of the wire: purity (the unhidden
proofs do not change a single byte, and the options keep saying what verifies) and coherence (the
35 provers do not change: they inherit the `None`). What is lost, and is said: g2 and g3 of the
spike —that the raised ce is used only by the prover, and that without the raise no proof that
verifies comes out— needed the switch; they stay as measured in the spike (r3, with the debug
asserts), and in the tree `cabe_con(2)` = false leaves it said that the raise is needed.
**Reversible** towards a field of the wire if E3b asks for it.

### D-V — The seven falsifiers, as tests of `stark-experiment`

The seven of D-K live in `crates/stark-experiment/src/falsadores_oculta.rs`, brought from
`spike/src/main.rs` of spike-b-p4r3 (`145e83ecea366126`), where they were stretches of a binary
with a verdict by rc: the zero census (80-83), verifies (84), rejects public+1 (85), the touched
mark (89: here an error is demanded, and the panic the spike took as good is a red with a name),
the invalid dimensions (97, plus m ≥ 2T), WorkAir with the raise (94, 96: ce 2 → 4, it fits, and
the proof verifies) and the switched-off regression (86, 88 and the structural one: switched off
through the API it is byte for byte the house prover; the same hiding, the same bytes; another
quotient seed leaves the trace roots and moves the constraints one). Six run over `WorkAir` with a
prover of the module's own that returns the hiding the test gives it; the zero census, over a toy
with one constant column of the trace and another of the auxiliary segment, copied from the spike
and trimmed to what it measures (128 rows, 3 + 2 columns). `winter-air` and `winter-prover` enter
as development dependencies of the crate, because `Ocultacion`, `Marca` and `Oculta` do not come
out through the `winterfell` umbrella, which is not forked. They run in debug and in release. Since
§537 the module carries two more falsifiers, those of D-AH.

`stark-experiment` wins over `zk-ssl-air` (purity: a judge does not compile the prover) and over
the layer (coherence: the hidden mode only in tests, and the toy belongs to circuits).
**Reversible** if E3b brings the hidden mode to the house provers: then the falsifiers move with
them.

### D-W — The salt belongs to the consumer and enters in E3b

The salt of D-I does not live in the fork: `MerkleConSal` is a `VectorCommitment` that the prover
and the verifier receive as a type (`VC`), on the side of whoever calls them —the «138 lines of our
own» of D-F—, and that is why it salts the trace, the constraints and FRI at once. It changes the
bytes of every proof that uses it, switched off or not, so it does not fit in E3a without breaking
the photo: it enters in E3b with the change of `VC` of the provers with a row in D-B (D-Z; here it
said «the 35») and of the kit, in `zk-ssl-air` (what the kit sees), and with `zkssl/0.4`.

Saying it wins over leaving D-G in the present tense: faithful image (the tree does not salt, and
says so). **Reversible** with D-I: if the argument withdraws the salt, this point goes with it.

### D-X — The salt is born inside the type, from the system's entropy

`VectorCommitment::new(items)` calls `with_options(items, Options::default())` and admits no seed;
the fork's core builds its commitments with `V::new`
(`winter-prover/src/matrix/col_matrix.rs`:285 and `row_matrix.rs`:227) and `winter-fri`, which is
not forked (D-F), its layers too (`prover/mod.rs`:335). So the salt can only be born inside the
type: `MerkleConSal::with_options` takes 32 bytes from `rand_core::OsRng` (`getrandom`) per leaf
and passes them through `H`. The spike took it from a SplitMix64 seeded by a process static
(`spike/src/main.rs`:43-69), the class that D-U withdrew, and declared (:49-50) that in production
it would come from entropy; upstream brings no salted tree (0 occurrences of `salt` in
winter-crypto 0.13.1, a single `impl VectorCommitment`; PASTE-E3b-M, S2).

Consequences, stated: (1) a proof with salt is not reproduced byte for byte, not even in tests: it
is watched by what verifies, what rejects, what the E2 suite counts and what it weighs (D-J already
admitted it); (2) a single type for both shores: the kit compiles `with_options` and never calls
it; `rand_core 0.6` and `getrandom 0.2` were already in the lock through `winter-crypto → sha3 →
digest → crypto-common`, so the lock gains one edge and no package, but the kit's closure DOES gain
them as packages —`cargo tree -p zk-ssl-verify -e normal`: 46 → 48, measured by ENSAYO-535; a lock
does not tell features apart— and the artefact's THIRD-PARTY names them from this seal;
`winter-prover` stays out (the H2 gate, by name); (3) the type lives in
`crates/zk-ssl-air/src/sal.rs` (D-W), brought from the spike (`spike/src/main.rs`:77-209) with
seven witnesses that prove no STARK, and it enters WITHOUT anyone declaring it as `VC` (E3b-0,
§535): no proof changes a byte and the photo of D-R stays green.

It wins over a seed in `Ocultacion` with a channel through the fork: purity (no process state) and
coherence with D-J; the channel would not reach FRI without forking it. **Reversible** with D-I: if
the argument withdraws the salt, the type goes with it.

### D-Y — The photo keeps its own pristine provers

The three provers of the photo (D-R) —`WorkProver`, `BandaProver` and `EdadProver`— declare
`MerkleTree<Blake3>`; two are production ones, and E3b changes their `VC` and switches hiding on
for them: neither with a deterministic salt nor switched off would they give the photo's bytes
again. The photo measures a single thing, «core switched off with `MerkleTree` = winterfell byte
for byte», and keeps measuring it with provers of `kat_probador.rs`'s OWN over the real `BandaAir`
and `EdadAir`, copied as it already copies its set-ups and as the falsifiers of D-V do with theirs.
`WorkProver` is not a production one: it stays as it is and remains the reference of
`regresion_apagada`. It is cut 1 of E3b (§536). The judge of each KAT is also its own: winterfell's
`verify` with `MerkleTree`, as `kat_work` already did, and not the kit's `verificar`, which in cut
2 moves to `MerkleConSal` and would reject the photo (D-AB); and the `edad` prover carries its
auxiliary trace copied, because the production function that builds it is private to
`circuit_edad.rs` and a prover of its own does not lean on it. Cut 2 does not touch
`kat_probador.rs`.

It wins over withdrawing `banda` and `edad` from the photo: purity (the core's falsifier does not
depend on production decisions) and faithful image (the photo says which prover each byte is
from). **Reversible** if the photo stops being the core's falsifier.

### D-Z — The provers that have a row in the table of D-B are switched on

E3b switches on, with `MerkleConSal` as `VC` and `Some(Ocultacion)` seeded from entropy, the
provers that have a row in the table of D-B: those of the wire and the four tabulated experimental
ones, which are the ones the census of D-L (`el_censo_del_cable_da_fila_a_cada_air_verificada`)
names plus the four of the table; it is what «the table goes to zero» demands. Those without a row
do not cross the wire, hide nothing from anyone and stay switched off with `MerkleTree`, and the
suite measures it: a row that is switched on counts zero, one that is not, what the table says. It
corrects the count of D-W: not the 35, those of the table. It is cut 2 of E3b, with D-AA to D-AC.

It wins over switching on the 35: minimalism (every piece with its function) and cost (proving
×2.6 and verifying ×1.9 in every switched-on row, measured in the spike: PASTE-E3b-M, S6); over
switching on only those of the wire: coherence with D-K, which promises the whole table at zero.
**Reversible** towards the 35 if an experiment crosses the wire.

### D-AA — `zkssl/0.4` is born in E3b, and RFC-0006 announced it without consuming it

The header of RFC-0006 (:9) and its E2 row (:225) announced `zkssl/0.4` for head v4; §415 left the
wire at `zkssl/0.3` (additive) and decided to keep that header and correct below it (:247). E3b is
the first stage that really raises the wire: `zkssl/0.4` is born here, and this RFC says so in
Compatibility; the header of 0006 is not touched (past, and already corrected in its place).

Saying it wins: clarity (two RFCs cannot claim the same version without one saying that the other
did not consume it). **Reversible** towards nowhere.

### D-AB — The 0.3 vectors are kept under their version, and kit 0.4 does not verify them

With `MerkleConSal` in the five `verificar` of `zk-ssl-air`, the openings of a 0.3 proof are no
longer read, and a proof without a mark on the 0.4 wire is rejected: fail-closed, without double
dispatch by the mark. The 34 vectors with a proof (age 10, payment 8, pending 8 and rejection 8,
measured) are kept under their version (rule 2 of the PROCESO) and are verified by the kit that
saw them born, `arqueo-verify-v0.2.0` (§442), measured from outside; those of 0.4 are regenerated
from their benches with `--guardar` (§467), `zkssl-0.4.json` is born, and the canon demands «0.4
IDÉNTICO / 0.3 RECHAZADO».

It wins over accepting both openings by the mark: purity (one wire, one shape) and fail-closed;
over rewriting the old vectors, faithful image (a vector is what its version produced).
**Reversible** if a second implementation demands verifying 0.3 with the live kit.

> **Correction (§620, before publishing 0.3.0).** «They are verified by the kit that saw them born,
> `arqueo-verify-v0.2.0` (§442), measured from outside» —and the E3b bullet, which repeats it— was
> never measured, and is false: that release predates these four families (they are born between
> §455 and §509) and its binary, `6356debde7f117b1`, rejects the 38 vectors of `spec/vectors/0.3/`
> with «tipo desconocido». Those that saw them born are the tree's `zk-ssl-verify` binaries, which
> carried the number 0.2.0 without being that release: the last one with the 0.3 wire, compiled at
> `0eda58c` (§537), gives **38 of 38** with the four manifests of `spec/vectors/0.3/`. Measured in
> §620. The sentence is kept; what holds is this, and the eight manifests that repeated it already
> say so.

### D-AC — The published byte figures become a band, with their two ties

`PUBLICADA_PAGO_B` (`crates/zk-ssl/src/metrics.rs`:75, 133,431) and its two ties —the test
`la_cifra_publicada_sigue_siendo_la_medida` and `tools/check_publicadas.py`— are measured today
over deterministic proofs. With hiding a proof weighs according to `q` and according to the salt
(D-J: 80,936, 81,637 and 80,807 bytes in the spike): the constant becomes a band measured over N
proofs, the test ties it to the instrument and `check_publicadas` ties it to the documents, which
cite it as a band; the ten live citations (measured) are rewritten in cut 2, with the `~62 KB` of
`metrics.rs`:513 and :517.

The band wins over a figure: faithful image (D-J). **Reversible** towards nowhere as long as there
is hiding.

**Measured in §538.** Fifteen samples per axis (five through the layer's path and ten through the
client's, PASTE-538-M R12): send 77,444..80,232 B, claim 76,192..79,736 B; with a declared margin
of 5 % on each side, the band the tree ties is send 73,571..84,244, claim 72,382..83,723 and
payment 145,953..167,967 B (139.2 to 160.2 MiB per thousand payments; 146.0 to 168.0 MB in SI).
`PUBLICADA_PAGO_B` is split into `PUBLICADA_PAGO_MIN_B` and `PUBLICADA_PAGO_MAX_B`; the node's
heartbeat and its tie consume the maximum (it brings forward the warning of note 22, which is its
meaning); `check_publicadas.py` ties every live citation to one of the two ends and takes
`BACKLOG.md` out of its universe as a record, as it already was out of ATADO C. The switched-off
figure (133,431 B, 127.2 MiB) stays in the entries and in entry 22.

### D-AD — The hidden shape is the only shape of the 0.4 wire

A hidden proof declares width + 1 (the hiding column) and 2T rows: it is what `Oculta::new` builds
and what the shape guard of S529 —`comprobar_forma` in the layer, the five comparisons of the kit
and `forma_ok` of the threshold pair— rejected with the switched-off shape of the statement
(5.A-393, measured in PASTE-E3b2-M: the layer rejected the switched-on v2 send with `MerkleTree`
and with `MerkleConSal`, and accepted it as soon as the guard demanded the hidden one). Since §538
the shape demanded in `zkssl/0.4` is the HIDDEN one, derived in the function itself from the AIR's
(`ancho + 1`, `2 * longitud`; the auxiliary segment and its random elements do not change), and
the switched-off one is rejected: two falsifiers say so —`guarda_forma` with the hidden shape
(45, 1024) as the exact one and (44, 512) rejected, and the pair mutated to the switched-off shape
gives `WrongTraceWidth`—.
⚠️ §913: the shape guard lives in the single judge (`zk_ssl_air::juez`, ARQ-01, entry 117): the
hidden shape is derived from the clear one each family declares —width + 1, the auxiliary segment as
it is, 2T— and is checked with the mark before building any AIR, in a single place through which the
21 pass; the layer's `comprobar_forma` stays for its tests.
⚠️ §914: the only shape of the `zkssl/0.5` wire is width + 2 —the two random columns of D-AL bis—,
the auxiliary segment as it is and 2T, with T ≥ 128 in the 23 families; that of `zkssl/0.4` (width +
1) falls by its mark before reaching the shape, and a v2 mark with another number of columns also
falls at the mark (the 1h bis of D-G of RFC-0018).

A single shape wins over admitting both by the mark: purity (one wire, one shape; fewer silent
eras) and fail-closed (coherent with D-AB). **Reversible** towards double dispatch by the mark if a
second implementation could not hide.

### D-AE — `Ocultacion` is built in `stark-experiment`, with `winter-prover` as a normal dependency

The 23 provers with a row return `Some(crate::ocultacion_encendida())`: m = 64 and two seeds from
`zk_ssl_air::sal::semilla()`, the same entropy as the salt (D-X). `winter-prover` moves from
dev-dep to a normal dependency of `stark-experiment` (it was already in its closure through
`winterfell`: the lock does not move, `f8c226859ec029af`), and `Ocultacion` does not come out
through the `winterfell` umbrella.

> ⚠️ **Corrected in §652 (CRIPTO-01).** «The same entropy as the salt» was false: `semilla()`
> returned a `u64` of 8 bytes from `OsRng` and the salt takes 32. With 2^64 seeds, the random rows
> of a column could be recovered by brute force (about 2^74 Blake3 compressions, ESTIMATED) and,
> with them, any constant column of the witness: the spend key or a custodian's. Since §652 the two
> seeds of `Ocultacion` are `[u8; 32]` and `semilla()` takes 32 bytes; a falsifier ties the width.
> The sentence above is left as it was.

A single constructor wins over one per circuit: coherence (the same pattern in the 23, and in the
row of D-B one reads where each comes from). **Reversible** towards an m per AIR if D-AG were not
enough.

### D-AF — The type reaches the layer through `stark-experiment`, with no new dependency

`pub use zk_ssl_air::sal::MerkleConSal` in `stark-experiment`; the layer imports it from there (it
does not depend on `zk-ssl-air`: it reaches the AIRs through `stark-experiment`, measured). A
single type for both shores (D-X): the 21 production `verify` —15 of the layer, that of the pair
and the 5 of the kit—, the two of `metrics.rs` and the 69 of the tests of the 23 circuits verify
with `MerkleConSal<Blake3>`; the 16 switched-off provers and the two of the photo stay with
`MerkleTree` (18, the generator's census).

The re-export wins over a new dependency: minimalism. **Reversible** towards nowhere as long as
the layer does not depend on `zk-ssl-air`.

### D-AG — `m` does not fit in the short traces: padding to 64 and `edad` with m ≥ 3

The fork demands m < 2T and since §537 returns it as `Err(OcultacionNoCabe)`; with m = 64,
`RefundAir` (T = 2 × 8 = 16), `RefundAirV2` (4 × 8 = 32) and `EdadAir` fell when `filas = 8 << m`
stays below 64 (5.A-395; the bench sets up ledgers with `nextPending = 2`: m = 1). The T of the 23,
read from their constants: 1024 (6), 512 (9), 256 (3), 64 (2), 32, 16 and 8 × leaves; and D-I asks
besides for T ≥ 44. Since §538 the two refunds carry `CICLOS = 8` (64 rows: the merges and the
padding behind, which nothing asserts; `ROW_P` fixed at the last merge) and `edad` counts at least
8 leaves in `celdas_del_libro` AND the kit raises the floor of `m_canonico` and of
`comprobar_enunciado` from 1 to 3, because it derives m from `nextPending` (D-2 of E4b-2, «one
mark, one subtree»): raising the prover without the kit rejected the mark and raising the kit
without 0.4 rejected the 0.3 `edad` positives (8/11 measured). That is why D-AG is not separable
from 0.4 and goes with E3b-2 (PRECISION 717). Measured: 12/12 in the refunds, 7,401 → 21,014 B and
10,147 → 24,732 B hidden; `banco_edad` VERDE with m = 3 (79,060 and 75,048 B against 43,092 and
44,935 of the 0.3 vectors).

Padding to 64 with m = 64 wins over an m of its own with a floor of 44: purity (a single rule,
without exception per circuit) and clarity. **Reversible** towards an m per AIR (D-AE admits it).

### D-AH — A bad witness is an `Err` of the prover, never a panic

With hiding switched on, a trace that does not meet its constraints tripped two asserts of the
fork: the one for `m < 2T` in `generate_proof_oculto` and the one for the hidden quotient in
`composition_poly.rs` («the polynomial does not fit in N pieces of step 2T − m»). Switched-off
winterfell emitted in release an invalid proof that the verifier rejected with an error; the
switched-on fork crashed (5.A-396, measured in the M3 of session 175 over the three
`t487_una_congelada_no_*_con_camino_ajeno`). A client with a bad witness must not crash.

Since §537 the hidden prover checks the REAL trace against the wrapper's inner AIR
(`Oculta::interno`) before hiding it —the assertions and the transitions, auxiliary segment
included, with `Trace::comprobar`, the twin of `validate` that returns `Err` instead of asserting—
and it does so always, in release too: it costs one evaluation of every constraint per row, with no
FFT. A bad witness is `ProverError::UnsatisfiedTransitionConstraintError(paso)`,
`AsercionNoSatisfecha` or `RestriccionAuxNoSatisfecha`; a trace too short for that m is
`OcultacionNoCabe { m, filas }`; a wrapper whose second bound of the exemptions does not fit even
with the blowup, `EnvoltorioNoCabe`. The quotient's asserts remain as invariants that a checked
trace cannot violate. Switched off, nothing changes: the prover does not check and the proofs are
still byte for byte (the photo of D-R).

Two more falsifiers in `falsadores_oculta.rs` (nine with the seven of D-V): a falsified cell in
row 100 of `WorkAir` returns `Err` at step 99, not a panic; 16 rows with m = 64 return
`OcultacionNoCabe { 64, 32 }`, the case of the refunds before D-AG. Measured in PASTE-537-M
(session 176): with the whole switch-on of cut 2, the three `t487` receive
`prove: UnsatisfiedTransitionConstraintError(544)` from the prover instead of crashing, and since
cut 2 they expect that `Err` where today they expect a proof.

`Err` wins over asserting: fail-closed without panic rules inside the fork too (PRECISION 714).
**Reversible** towards returning the error from `segmentar` itself, which forces threading `Result`
through `CompositionPoly::new`, `DefaultConstraintCommitment::new` and the house's `impl Prover`:
wider, the same promise. D-AD to D-AG —the hidden shape as the only shape of 0.4, the constructor
of `Ocultacion`, the re-export through `stark-experiment` and the short traces— arrive with cut 2.
⚠️ §911: two more panics, which the agent's review saw on reading the fork (§907), become `Err`: the
degree assert of DEEP —`GradoDeDeep { esperado, real }`, entry 137 of `BACKLOG.md`: with a VALID
witness of the 23 AIRs it fires with probability of the order of 2^{−128}, and with a DEGENERATE
trace in the clear —constant columns whose transition is identically zero— it always fires, because
the composition stays at zero and DEEP without degree; it is now a named error in the DEEP
compositor, and so in the hidden path and in winterfell's— and a periodic or sequence assertion of
an inner AIR, which the wrapper would also demand in the T random rows
—`AsercionPeriodicaBajoOculta { columna, primer_paso, paso }`, entry 138: the hidden prover rejects
it before touching the trace, instead of falling into the assert of `segmentar`; none of the house's
23 AIRs uses it—. Falsifier 13 of the core's hiding measures the two events: an AIR with a periodic
assertion proves and verifies in the clear, and hidden it is `Err` without panic, and the degenerate
trace in the clear is `GradoDeDeep` instead of the panic.

### D-AI — The 0.4 comparator crosses what the circuit fixes and declares what is not reproduced

The conformance scenario was deterministic end to end, proof included, and its chain tied it:
`proof_digest = digest_of_proof(proof)` over the bytes of the proof (`log.rs`), `chain` carries it
along and `epoch_digest` carries it inside. With hiding switched on the bytes of a proof are not
reproduced (D-X). Measured in §538 (PASTE-538-M R11): two `--emit` of the same scenario give equal
`spec`, `sellado`, `escenario`, `canon`, `supply`, `pending` and, in the six entries, `seq`,
`kind`, `root_old`, `root_new` and `compromiso`; they differ exactly in `proof_digest` and `chain`
of the two entries with a proof (seq 0x4 and 0x5) and in `epoch_digest`; the 0.3 comparator gave
«conformidad ROTA: 3 diferencia(s)». Since §538 `--check` crosses what the circuit fixes —`seq`,
`kind`, the two roots, the commitment, the supply and the pending— and DECLARES the three digests
not reproducible; the 0.4 vector carries them as what its emission produced (D-AB), and the canon
demands «0.4 → todo IDENTICO (en lo que el circuito fija)» and «0.3 RECHAZADO (otra version)».

Declaring wins over changing what the chain ties: purity (what the circuit does not constrain is
not an invariant: the bytes of a proof are not) and minimalism (the chain of the log does not
change format). **Reversible** towards a chain that ties the statement instead of the bytes of the
proof, which is a change of format of the log and of the head, with its RFC.

### D-AJ — The expensive side stops being asserted: switched on, both sides cost the same

`el_lado_caro_es_el_declarado` asserted the DIRECTION (the receiver pays more, measured paired on
2026-09-19 with 260-290 ms for the send and 160-190 for the claim). With hiding switched on, each
side costs 700-750 ms and the direction fell within the noise: one of three runs gave PAYER (send
741.6 ms and claim 696.9 as minima, ratio 1.064) and the others 0.975 and green (PASTE-538-M R4 and
R12). A contract that falls one time in three asserts nothing: since §538 the assert is withdrawn
and cited (mould S247), `LADO_CARO` is withdrawn with its history, the times are printed and the
test is renamed after what it ties, `los_dos_lados_del_pago_atan_la_banda`: the bytes of both
sides through the client's path, within the band of D-AC (one row of the layer's `--list` changes
its name, none is born or dies). ⚠️ §860: the figures «260-290 ms for the send and 160-190 for the
claim» are not from the paired measurement of 2026-09-19 but from that of 2026-08-14 (§304), when
the send was the expensive one; that of 2026-09-19 gave send 158.9 ms and claim 278.4 (§512), the
receiver more expensive, which is what the sentence asserts.

Withdrawing wins over a temporal band: faithful image (the times depend on the machine and the
direction no longer holds). **Reversible** if the direction were measured stable again.

### D-AK — The 0.4 vectors are produced by the block, and the anchors are born afterwards

With non-deterministic proofs a vector is not embedded by predicted fingerprint: BLOQUE-538 runs
the four benches (`banco_pago`, `banco_pendiente`, `banco_rechazo`, `banco_edad`, with
`--guardar`), renames the rejection captures to the names of the catalogue with the map derived
from the bench itself (the text of each `niega` against the line of the MANIFIESTO), runs the
harness line by line over the fresh ones with kit 0.4 and only with 9/9, 9/9, 84/84 and 11/11 does
it commit; the 0.3 catalogues of payment, pending and age are set aside whole under
`spec/vectors/0.3/<familia>/` (R10: all their files change, the head is another) and of rejection
the nine the bench regenerates, with a MANIFIESTO of nine lines; the remaining 75 of rejection
stay (82/84 with kit 0.4: their texts do not depend on the shape). The 0.3 MANIFIESTOS serve the
0.4 vectors line by line (measured: 9/9, 9/9, 11/11 in SALIDA-538M), so 0.4 is 0.3 with its
header. Everything else in the commit is predicted; the vectors' anchors are measured in the entry
afterwards.

Producing them in the block wins over carrying them in the block: fail-closed (without the harness
green there is no commit) and faithful image (a vector is what its version produced).
**Reversible** towards seeded vectors if D-X were to change.

### D-AL — The DEEP mask has to live in E: two random columns, in the laboratory until the cut

**Status: built in the laboratory, switched off (§899).** §898 measured that the random column of
D-G does not mask what FRI opens of the DEEP polynomial: it lives in F and, in each value of E
that FRI opens, it moves only one of its two coordinates; with the quotient's randomisers, which
contribute 80 dimensions in total, the condition R3 of `doc/ocultacion-hvzk.md` is not met in any
production geometry (entry 133 of `BACKLOG.md`). What meets it, measured with the same instrument
in the five geometries and in the toy: two random columns over F, each with its own DEEP
coefficient in E —together they cover the two coordinates of every point, and with T ≥ 256 they
cover on their own everything FRI opens without determining—, or one random column over E, which
leaves exactly K free by construction. The two-column one is chosen: it does not touch the
auxiliary segment, which winterfell admits only with a constraint and an assertion of its own, and
it is a parameter in `Ocultacion` (`columnas`), a loop in the prover and a subtraction in the
wrapper.

**What is in the tree since §899.** `Ocultacion::columnas`: with 1, the construction of
`zkssl/0.4`, byte for byte —the same draws of the coin, the same mark—; with 2 or more, the
laboratory one, which writes the laboratory mark, `arqueo:oculta:lab`, m and the columns in one
byte, 22 bytes, and which `verify` rejects as an unknown meta, just like another version of the
mark (D-S): only `verify_laboratorio` accepts it, and no production judge calls it. The 23 provers
stay with one column (`ocultacion_encendida`). A falsifier in `falsadores_oculta` measures it: with
two columns, width + 2, the laboratory mark, rejection without panic in `verify` and acceptance in
the laboratory; with one, the usual, and the production one does not enter the laboratory. The R3
bound with two columns, pinned per geometry on every canon run (`R3_MEDIDO_LABORATORIO`).

**What is missing, and belongs to the cut.** Switching the two columns on in the 23 provers
changes the bytes of every proof: it belongs to the 0.5 cut (RFC-0018), never to `zkssl/0.4`; the
v2 mark of its D-G would have to carry the random columns, one more byte, and the laboratory one
goes then. And R3 green is not HVZK: it covers what FRI opens; what is touched per column (L1,
entry 131) and what depends on the AIR (L6) stay as they were, and the promise is still the census.
⚠️ §900: entry 134 says which families need it for R3 with today's construction: MerkleAir and
FrozenClimbAir, with no linear column, and the five of T = 64, which first need entry 131; the
other sixteen meet R3 with the linear mask of their own columns (`doc/ocultacion-hvzk.md`, section
14). It is still the way out independent of each AIR's algebra, and the cut's.

Two columns over F win over one over E: purity (no extra constraint or assertion) and cost (one
more column of 2T elements of F per proof, against a new auxiliary segment where there is none).
**Reversible** towards the column over E, which the instrument measures just the same, if the cut
prefers the auxiliary segment.

⚠️ §906: the laboratory v2 mark (RFC-0018 D-G) already carries the column byte this decision asks
for, and the prover writes it with the columns it is asked for; switched off until the cut.

⚠️ §908 — **D-AL bis, proposal of the session, the author decides: the two columns enter DEEP as ONE
over E.** With two independent DEEP coefficients, cc₁ and cc₂, what the two columns leave free in D
is 2(x − x₀)·V_pos·(x − z̄)(x − z̄·g)·{cc₁·u₀ + cc₂·u₁}, which is E[x] only if cc₂/cc₁ ∉ F: an event
of probability 1 − 1/|F|, and the reason the bounds of the argument on the challenges
carry |F| = 2^64 in the denominator (`doc/ocultacion-hvzk.md`, L8). With cc₂ := cc₁·ν, ν a fixed
element of E outside F, the two columns are one over E, R₀ + ν·R₁, with a single coefficient, and
lemma L9 of section 17 of the document says that then the masks cover the whole of K —not only what
FRI opens— for any positions, α_i and AIR, save for an event of probability ≤ |E|^{−(N−1)}: R3 stops
being a measured hypothesis and becomes a theorem, and with it the argument does not depend on the
AIR. Not a byte of the proof changes with respect to this decision —the same two columns, the same
openings— nor does soundness: the DEEP combination still has one coefficient per polynomial, and
R₀ + ν·R₁ enters no constraint. What changes: one line in the prover's DEEP compositor and another
in the verifier's, in the seal of the cut. Measured in the instrument of R3 (`cubren_k`,
`K_ENTERO_PAREADAS`): in the five geometries and in the toy, with seed 1 and with the adverse
positions of §907, the paired masks give the whole of K; unpaired, too, save for that event; with
one column, no. **Reversible**: it is a parameter of the compositor, and the measurement of D-AL
still holds if it is not paired.
⚠️ §909: **accepted by the author** («Hacer todo» [do everything], 29 October 2026) and **built in
the fork**: `Oculta<A>` draws the DEEP coefficients like winterfell and pairs the random columns
when they are two or more (`parear_deep`, `nu_de_pareado`: the k-th carries cc·ν^k), in the prover
and in the verifier at once, so the proof does not change shape; `verify_laboratorio_sin_parear` is
the falsifier: a laboratory proof with two or three columns verifies pairing and not without
pairing, and the production one, with one column, is byte for byte the one of before. Switching it
on in the 23 provers still belongs to the cut (RFC-0018 D-H).
⚠️ §912: `tools/segunda` pairs the same way, in the laboratory: `parear_deep` over the DEEP
coefficients of the random columns the v2 mark declares, with ν = φ = (0, 1), and its self-test
measures it against the fork's pairing; its production path, with the v1 mark and one column, does
not change.
⚠️ §914: **switched on** in the 23 provers in the seal of the cut (`ocultacion_encendida`,
`COLUMNAS_ALEATORIAS = 2`, the v2 mark with the column byte), and it is the only production path of
the kit, the layer, the pair and `tools/segunda`; R3 remeasured green in the five geometries, the
one of T = 128 included (`K_ENTERO_PAREADAS`).

### D-AM — The promise after A1, A2, A3 and entry 134: it is still the census; what could be promised, for which families and under which hypotheses, is left written in its own words and belongs to the author

**Status: decided in §901 by the session, with the criterion the author dictated for A4 —«it cannot
state HVZK for any family with the argument of A2; what it can say is the census and what is
missing, and leave the promise for what entry 134 measures or for the cut that carries entry 133»—;
REVERSIBLE; it enters with his acceptance.** ⚠️ §902: accepted by the author by path (b): the
promise does not change, and what is left written awaits the outside review of the argument (entry
136 of `BACKLOG.md`). ⚠️ §916: **accepted by the author** on 29 October 2026 («Aceptar todo» [accept
everything]), by path (a) and for the twenty-three with the construction of `zkssl/0.5`: the promise
is, since §916, that of the draft of §915, with H7 inside —the argument has no review by an outside
person, and entry 136 is still open—; D-A, `SECURITY.md` §3.bis and the header of
`doc/ocultacion-hvzk.md` change in §916.

**What lies in front.** A1 (§896) measured the hiding bound per family; A2 (§897) wrote the
argument with its simulator and its lemmas, `doc/ocultacion-hvzk.md`, reviewed twice adversarially
by the session itself (its section 11) and by nobody outside; A3 (§898) measured the condition R3
and found it false in every geometry with the two masks that do not depend on the AIR; entry 133
(§899) met it in the laboratory with two random columns; and entry 134 (§900) measured the linear
mask of each AIR's columns, with which R3 is met in sixteen families with today's construction and
not in seven.

**What is promised: the same.** D-A, as it stood in §538: «a proof of Arqueo that verifies
demonstrates the transition it states and publishes literally no value of a constant column of its
trace». What is not promised: that nothing can be deduced from what is opened. `SECURITY.md` §3.bis
and §3.11, rule 3 of the PROCESO —the spend key never travels—, the residual trust of D-A and the
H7 sentence —neither the fork nor the construction are audited— do not change; «against a quantum
adversary: not measured» does not either. No live text says HVZK of anything that exists.

**What could be promised, in its own words, and is not promised until the author accepts it.**

> For sixteen of the twenty-three families with a row in the table of D-B —SendAir, SendV2Air,
> ClaimAir, ClaimAirV2, BurnAir and DoubleEntryAir (T = 1024); PrendaAir, MintPendingClimbAir,
> AuditAir, BandaAir, CobroPendienteAir, PagoEnCursoAir, CreditClimbAir, MintClimbAir and
> RecoveryClimbAir (T = 512); and SolvencyAir (T = 256)—, with the construction of `zkssl/0.4` as
> it stands —T random rows per column, one random column, the quotient blinded with m = 64 and the
> leaves salted—, the view of the honest verifier is simulated without the witness: for every real
> trace the AIR accepts, the view and the output of the simulator of section 5 of
> `doc/ocultacion-hvzk.md` are statistically indistinguishable in the programmable random oracle
> model, with advantage less than 2^{−60} plus Q·2^{−256} for Q queries to the oracle: simulatable
> hiding, statistical in that model and computational in what H7, Blake3 as an oracle, has of
> computational. Under the hypotheses of section 4 of that document: those measured on every canon
> run (H1, H4 and H11, and H6 with H12, the linear mask, measured per family), those proved (H2, H5
> and H10) and those assumed or of the model, which no test measures: H3, z outside F and H and
> without coincidences of conjugates, with failure bounded per proof; H7, the programmable random
> oracle; H8, the fresh salt; H9, no seed of the hiding repeated. And under the argument as it is
> written: reviewed twice adversarially by the session, and by nobody outside the project.

**What it does not cover, by name.** MerkleAir and FrozenClimbAir (T = 256): none of their columns
enters linearly in their constraints —the sbox touches all of them—, R3 is that of §898 and fails;
they need entry 133. NullifierThresholdAir, ThresholdAir, EdadAir, RefundAir and RefundAirV2
(T = 64): L1 does not cover them —2q + 6 = 90 touched points against 64 random rows— and there is
no linear mask to measure; they need entry 131 (T = 128) or the lemma of entry 135, and afterwards
entry 133 or the measurement of entry 134 again. Five of the seven are of the wire: the delegated
authorisation, age, the two refunds and the delegated freeze climb. For the seven, nothing could be
promised today with any written argument.
⚠️ §905: the fine count (entry 135) is measured and does not change this today; for the cut it does:
with entry 133 and lemma L1', the joint threshold enters what could be promised without more rows,
with MerkleAir and FrozenClimbAir by entry 133; the other four of T = 64 wait for entry 131.
⚠️ §914: with the `zkssl/0.5` cut the seven have what they lacked: the five of T = 64 move to 128
(L1 covers them) and the two paired columns give R3 by L9 to MerkleAir, FrozenClimbAir and the
others, measured (`K_ENTERO_PAREADAS`, `INDEPENDIENTE_DE_LA_AIR`). What can be promised with that,
and with what words, is written by the next entry (the proposition for the 23 in
`doc/ocultacion-hvzk.md`); the outside review and the acceptance belong to the author, and until
then the promise is still the census.

**What the written text lacks to be a promise.** (1) The acceptance of the author, who decides on
what is measured (`doc/METODO.md` §7) and whose every promise is; (2) an outside review of the
argument of A2 with entry 134, which is what A4 asked for in keeping H7 (⚠️ §903: its dossier is
`doc/ocultacion-hvzk-dosier.md`; the reviewer is chosen by the author, entry 136 of `BACKLOG.md`;
⚠️ §907: the first review from outside the session is done and closed, and it is by an agent, not a
person —`doc/ocultacion-hvzk-revision-agente.md`; sections 11 and 16 of the document—: no high
objection; it left written two lemmas that were missing, L7 and L8, three instances of challenges
measured per family with an adverse one, the fingerprint of each composition pinned, and the count
of the advantage redone: ≤ 2^{−60} + 2^{−128} + Q·2^{−256} + ε_R3, with ε_R3 ≤ 7c₀/|F| ≈ 2^{−55,8}
in what L8 proves —layer 0 with T ≥ 512— and measured, not bounded, in layer 1, the coset–touched
coincidences and solvency; the bound on the challenges goes over |F| = 2^64 and not over |E|,
because the random column lives in F, and that is one more reason for entry 133. The review of entry
136 is still to be by a person); (3) for the sixteen, nothing more: neither the fine count of entry
135 nor entry 133 are needed. If the author accepts, D-A changes —its «what is not promised»
becomes, for the sixteen, the list of hypotheses above, and for the seven, today's—, `SECURITY.md`
§3.bis and the header of `doc/ocultacion-hvzk.md`; until then, nothing.

⚠️ **§915 — What could be promised with `zkssl/0.5`, in its own words, for the twenty-three, and is
not promised until the author accepts it.** With the cut sealed (§914) and the proposition of
section 18 of `doc/ocultacion-hvzk.md`:

> For the twenty-three families with a row in the table of D-B, with the construction of `zkssl/0.5`
> as it stands —T random rows per column with T ≥ 128, two paired random columns in DEEP as one over
> E, the quotient blinded with m = 64 and the leaves salted—, the view of the honest verifier is
> simulated without the witness: for every real trace the AIR accepts, the view and the output of
> the simulator of section 5 of `doc/ocultacion-hvzk.md` are statistically indistinguishable in the
> programmable random oracle model, with advantage less than 2^{−60} plus 2^{−128} plus Q·2^{−256}
> plus |E|^{−(N−1)} for Q queries to the oracle and N pieces of the quotient (2 in solvency, 8 to 10
> in the others): simulatable hiding, statistical in that model and computational in what H7, Blake3
> as an oracle, has of computational. Under the hypotheses of section 4 of that document with the
> cut (its section 18): those measured on every canon run (H1, H4 and H11, in the twenty-three),
> those proved (H2, H5, H10 and, since the cut, H6 by L9) and those assumed or of the model, which
> no test measures: H3, z outside F and H and without coincidences of conjugates, with failure
> bounded per proof; H7, the programmable random oracle; H8, the fresh salt; H9, no seed of the
> hiding repeated. No family out by its name, and no hypothesis specific to an AIR. And under the
> argument as it is written: reviewed adversarially by the session and by an agent, and by no person
> outside the project.

**What it lacks to be a promise, since §915.** (1) The acceptance of the author; (2) the outside
review of entry 136, by a person, of sections 1 to 18 with the construction of the cut; nothing more
to measure. If the author accepts, D-A changes —its «what is not promised» becomes, for the
twenty-three, the list of hypotheses above—, `SECURITY.md` §3.bis and the header of
`doc/ocultacion-hvzk.md`; until then, nothing: the promise is still the census, and the path is
still (b).

**Paths weighed.** (a) Promise HVZK for the sixteen now, with the hypotheses in front. (b) Not
change the promise, write what could be promised in its own words and leave it to the author's
acceptance. (c) Write nothing until the cut that carries entry 133 and switches the two columns on,
when R3 holds per geometry and without each AIR's algebra. (b) wins: purity —a security promise
belongs to the author, the house has not said HVZK of anything that exists and the argument has no
outside review— and faithful image —what is measured and what is proved are said by name and with
their hypotheses, and the seven families, with what they lack—; (a) is one acceptance away and (c)
would waste what was measured. **Reversible** towards (a) with the author's acceptance, and
towards (c) if he prefers to wait for the cut.

**A5, the salt.** It is still the author's (D-I): the document recommends keeping it —it is what
makes L4 unconditional, and without it the leaf of a FRI coset only has entropy through R3, which
is now measured green in sixteen families and not in seven—; this decision does not touch it.

## Compatibility

- `zkssl/0.3` **did not rise** until E3b-2; **since §538 the wire is `zkssl/0.4`**: no method,
  type or error of the wire changes, but the SHAPE of every proof that travels changes (width + 1
  and 2T, D-AD) and what it is verified with (`MerkleConSal`, D-AF); no vector is rewritten: those
  of 0.3 are kept under `spec/vectors/0.3/` and those of 0.4 are born from their benches (D-AB,
  D-AK).
- The promise had already changed in the prose and in the comments (§521 to §526): this RFC fixes
  it in one place.
- **E3a** does not raise `zkssl/0.3`: with the fork switched off, the proofs are byte for byte
  winterfell's, and switched off is what `Prover::ocultacion` returns in the 35 house provers
  (D-U).
- **E3b** raises to `zkssl/0.4`: the proofs that cross the wire carry the mark and only the fork
  verifies them. The 0.3 vectors are kept under their version and are verified by the kit that saw
  them born; those of 0.4 are born as captures of their benches, with randomness from entropy, and
  are watched without comparing bytes (D-X, D-AB).
- **E3b-0** (§535) raises nothing: `MerkleConSal` enters as a type and nobody declares it as `VC`;
  no proof changes a byte. `zkssl/0.4` belongs to E3b although RFC-0006 announced it for its E2
  and did not consume it (D-AA).
- **E3b-1** (§536) raises nothing: the photo of D-R gains its own provers and judges with
  `MerkleTree`; no proof changes a byte and the photo stays 3/3 (D-Y).
- **D-AH** (§537) raises nothing: switched off, the prover does not check and no proof changes a
  byte; switched on —today only in the falsifiers—, a bad witness returns `Err` instead of
  panicking.
- **E3b-2** (538) raises to `zkssl/0.4`: the 23 provers with a row switched on (D-Z, D-AE), the
  hidden shape as the only shape (D-AD), `MerkleConSal` on both shores (D-AF), the refunds at 64
  rows and `edad` with m ≥ 3 in the prover and in the kit (D-AG); the `zkssl_protocolVersion`, the
  OpenRPC and the conformance scenario say `zkssl/0.4` with its comparator (D-AI); the 0.3 vectors
  are kept under `spec/vectors/0.3/` and the 0.4 ones are born from their benches (D-AB, D-AK); the
  byte figures are a band (D-AC) and the expensive side stops being asserted (D-AJ); the table of
  D-B measures zero (D-A, D-C).

### Why it enters through an RFC

It does not touch the wire, and the PROCESO does not demand it. It enters because it decides what
the protocol asserts of its own proofs, and that deserves its own file: entries §521 to §526 left
it written as a debt.

> **Note (2026-09-24, §539).** ACCEPTED. Rule 4 of the PROCESO is settled with measurement, and it
> is said with what. First, what this paragraph asserts: **since E3b-2 (§538) this RFC does touch
> the wire** —it raises it to `zkssl/0.4` and changes the shape of every proof that travels—, so
> the PROCESO does demand it and rule 4 applies to it; the paragraph is not rewritten. (a) The
> spec, up to date: `spec/RPC.md` says `zkssl/0.4` (§538) and its principle says what is met and
> since when (§538-B); `spec/PAQUETE.md`, the 0.4 shapes (§538) and its prose (§538-B);
> `spec/README.md`, the principle, the conformance vectors and their citations by line (§538-D);
> RFCs 0007 and 0008, their corrections of §538 (§538-B); and `PROCESO.md`, rule 3 (§538). (b) The
> OpenRPC, regenerated with `gen_openrpc`: the version in §538 and the `description` in §538-B,
> byte for byte what the generator produces. (c) The vectors: `zkssl-0.4.json` and those of the
> families are born under the new version, from their benches (D-AK); those of 0.3 are kept under
> `spec/vectors/0.3/` and none is rewritten (D-AB). (d) The suites, green: the `--sello` canon of
> §538 and that of §538-B, with their harnesses and the reproducible artefact; §538-C, with the
> `--list` identical in the seven crates it opened and the kit identical except for its symbols.
> What this RFC does not promise is still in D-A: that nothing can be deduced from what is opened.
> Double thread of rule 5: this document cites §539 and §539 cites this RFC. Precedents of the
> turn: 0003 (§356), 0004 (§400), 0006 (§441) and 0007 (§481).

## Security

- **The API principle** (rule 3 of the PROCESO): since §538 it is met in what the E2 suite
  measures —no value of a constant column comes out literally in any of the 23 proofs with a row,
  100 cells at zero— and no more: D-C says what is measured and D-A what residual trust remains.
- **The operator is the adversary that receives the proofs.** The node sees everything the table
  of D-B says, and after a single delegated operation it has the keys of two custodians, which are
  enough to authorise the next one. Conservation does not fall: the proof is still sound. What
  falls is who can move the supply.
- **Whoever reads a published envelope** sees the salt, the `X` and the amount of the claim, or the
  salt, the `delta` and the `refund_id` of the payment; and in both, the index of the sender's
  account, which the envelope keeps quiet in its fields (RFC-0008, D-AH) and the proof carries
  (§531). The vectors of `spec/vectors/` are sandbox ones, with test keys.
- **What this RFC names and does not pay for:**
  - the kit's literal «esta causa no publica el saldo: la banda lo prueba sin el» [this cause does
    not publish the balance: the band proves it without it] (`crates/zk-ssl-verify/src/main.rs`)
    and the header of `crates/zk-ssl-air/src/pago_en_curso.rs` («el plazo NO viaja … nunca
    `delta`» [the term does NOT travel … never delta]), which live in the kit's closure and wait
    for the cut that opens it;
  - the same assertion in the header of `crates/zk-ssl/src/prueba_pago.rs`, which is outside that
    closure and no cut touched;
  - that `audit`, `prove_minimum` and `burn` receive the spend key as an argument of a method of
    the layer (`crates/zk-ssl/src/audit.rs`, `crates/zk-ssl/src/burn.rs`);
  - the two real proofs of the rejection catalogue;
  - the promise of `doc/ZENODO.md` that the cap circuit does not reveal the balance, which
    describes a deposit with a DOI and is not edited;
  - and the seed of `zk-core`, which is Groth16 and stays outside the model, like iso-bridge,
    settlement-layer and the PLONK and Halo2 experiments.
- **Rule 3 of the PROCESO:** E3b-2 (§538) is the stage that made it hold in what is measured: the
  E2 suite, which measures it in every type of proof since §531, counts zero since §538 (D-C).
- **What the census does not see:** that nothing comes out literally is not, on its own, hiding
  (D-A, until §538; since then the switched-on construction sustains it). Beyond the census, E3
  rests on the argument of D-I and on the construction of note 2024/1037. Neither the fork nor
  hiding are audited (H7). ⚠️ §901: the argument is written and measured
  (`doc/ocultacion-hvzk.md`, §896 to §900): its sufficient condition is met with today's
  construction in sixteen families and not in seven, by name; what could be promised for the
  sixteen, with its hypotheses, is left written by D-AM and not promised: it belongs to the author,
  and H7 is kept.
- **Failing closed:** with the mark, the verifier rejects with an error a trace that cannot be
  split or an m that does not fit in it (D-H), and since §534 it rejects, before building any AIR,
  a meta that is neither empty nor the mark (D-S). The AIRs that assert their width in `new`
  —`AuditAir`, `circuit_audit.rs`:366, among others— panic at a proof without a mark and with
  another width. It predates this stage and remains as a point of its own in the 5.A queue.
- **Debug:** with winterfell's debug asserts switched on —declared degrees equal to the real ones
  and size of the evaluation domain, `constraints/evaluation_table.rs`: 181-230—, the two toys
  pass with the wrapper and the raise, and proving G without the raise falls into the degree
  assert (`:214`): in debug, forgetting the raise does not go unnoticed (measured, SPIKE-B-P4 r3).
  In the 35, the size they demand coincides with that of the raise (reasoned with the formulas
  read). In the tree they are exercised by the tests of the hidden mode of E3a
  (`falsadores_oculta.rs`, §534), which run in debug and in release; the suite with the fork
  switched off does not touch them.

## References

- Entries §521 (the witness is published: PASTE-521-ROJO, PASTE-ZK-M, PASTE-ZK-M2 and
  PASTE-360-M), §523 (the model: PASTE-360-M2 and its two falsifiers), §524 (PASTE-367-M3) and §526
  (PASTE-367-M4); and §522 and §525, the cuts of the comments. The instruments live outside the
  tree, in the author's Downloads.
- §531 and its instruments, outside the tree: PASTE-E2-M (`3bb03cdbbb44a56d`, output
  `af9b1c5d662ab26b`), ENSAYO-531 (`b95a6f856d493203`, output `2b1888f387367a54`) and
  ENSAYO-531-r2 (`d0b973d5bdc429f8`, output `ed0ebac80ea101fc`, cargo `841b702722246376`).
- §532 and its instrument, outside the tree: PASTE-KAT-M (`f7e4b67352204b1a`, output
  `8faab897d07a7b05`), with the three `.bin` and their lines in `KAT-M-20260922-225843/` of
  Downloads.
- §534 and its instruments, outside the tree: PASTE-E3a2-M (`b97ceff1632aa603`, output
  `b190fdc212bf5271`, cargo `be5669c380e1df63`) with its `meta_m.rs` (`809004b72c1f0587`), which
  measured the meta byte for byte and that m did not travel; and `spike/src/main.rs` of
  spike-b-p4r3 (`145e83ecea366126`), from which the seven of D-V come.
- §535 and its instrument, outside the tree: PASTE-E3b-M (`1f1bb5ba5030c04b`, output
  `ae34451f8cca3a12`, cargo `d934ab3e51f46e2f`, the salt region `34508a89201ca6d3`), which read the
  salt in the spike and the trait in the registry, and ran the spike (VEREDICTO VERDE, the salt
  6,567 bytes per proof).
- §538 and its instruments, outside the tree: PASTE-538-M (`e94784996beeddc7`, output
  `d92b91297a878cc1`, cargo `93d716f89a286f52`), which measured the whole cut over a copy of
  `0eda58c` —the seven suites with zero reds, the four benches, the captures against the
  catalogue, the scenario twice and the band—, and its four generators `gen_m3.py`, `gen537.py`,
  `gen538.py` and `gen538b.py`, which the editor of the cut carries as payloads.
- `spec/rfc/PROCESO.md`, rule 3; `SECURITY.md` §3.bis, where the canonical sentence lives.
- winterfell: the repository's front page, <https://github.com/facebook/winterfell>, and its issue
  9, <https://github.com/facebook/winterfell/issues/9>.
- RFC-0007 (the band, age and the rejection) and RFC-0008 (the two portable envelopes and the
  pledge).
- The instruments of session 166, outside the tree, with the fingerprint of their output:
  PASTE-P4-M (`bc2ab57c2a683bcc`), SPIKE-B-ETAPA2A (`bea7d79ec60919f3`), SPIKE-B-P4 r1
  (`0afe71ce812aa713`), r2 (`5a0d1c5ee86ff89c`) and r3 (`7e4dfda3ced288dc`), and PASTE-GRADOS-M r2
  (`112d6654666f2941`); of session 165, PASTE-PROBADOR-M, PASTE-FORK-M and SPIKE-B-ETAPA1
  (`b51618eaeee76f79`), with INFORME-PROBADOR-165.
- winterfell 0.13.1: `air/context.rs`:290-331 (the exemptions), `air/transition/degree.rs`:90-115
  (the degrees) and `constraints/evaluation_table.rs`:181-230 (the debug asserts); note 2024/1037,
  section 4.2; the thread of issue 9, where its author takes the first piece as covered.
