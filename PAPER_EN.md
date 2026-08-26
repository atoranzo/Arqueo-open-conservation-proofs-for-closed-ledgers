# Implementing the Same Settlement Circuit Across Five Zero-Knowledge Proof Systems: Design Findings and Measurements

**Draft note.** This is the English version, intended for arXiv
submission to `cs.CR`. Note that arXiv requires **endorsement** for
first-time submitters in most categories; this must be arranged
separately from preparing the manuscript.

---

## Abstract

We report on implementing the same financial settlement circuit across
five zero-knowledge proof systems — Groth16, Halo2/IPA, STARK/FRI,
PLONK/KZG, and Nova/folding — and on the comparative evaluation that
followed. Unlike existing comparative work, which measures reference
circuits such as SHA-256, our comparison is carried out on a **complete
application**: a settlement layer with double-entry accounting, spend
authority, double-spend prevention, threshold issuance, supply
destruction, selective disclosure for supervision, and account freezing.

This methodological difference proves decisive. We document eight findings
absent from the comparative literature, all of which emerge only when
porting a stateful application between paradigms. The most significant is
that AIR arithmetization **lacks copy constraints**, which opens a silent
soundness gap when implementing Merkle tree updates — invisible to honest
witnesses and absent from Plonkish and R1CS arithmetizations.

We report proving time, verification time, and proof size measured under
identical conditions, and we document a methodological error of our own —
mixing debug and release build figures — that was detected and corrected
during the work.

The reference implementation comprises 319 executable tests — plus one ignored and documented — and is
publicly available. **It has not been audited by third parties and does
not implement distributed consensus**; we discuss the implications of both
limitations in detail.

**Keywords**: zero-knowledge proofs, STARK, financial settlement, AIR
arithmetization, trusted setup, post-quantum security.

---

## 1. Introduction

### 1.1 Motivation

Financial settlement systems face a structural tension between two
seemingly incompatible requirements. On one hand, confidentiality:
participants must not be able to observe third parties' positions or
transactions. On the other, verifiable regulatory compliance: a supervisor
must be able to confirm that limits are respected, that money is not
created outside authorized channels, and that operations are legitimate.

Deployed systems resolve this tension through institutional trust: a
central entity observes everything and certifies compliance.
Zero-knowledge proofs offer in principle a different resolution —
demonstrating compliance without revealing the underlying data — but their
application to financial settlement raises design decisions whose
consequences are not well documented.

This work addresses a specific question: **what actually changes when the
same settlement application is implemented on different proof paradigms?**

### 1.2 Why existing benchmarks do not answer this question

Rigorous comparative work on proof systems exists. zk-Bench evaluates
Groth16, PLONK, halo2, and starky with careful methodology, and reports,
among other results, that custom circuit implementations can distort the
perceived performance of a library.

However, such work measures **reference circuits**: hash functions, modular
exponentiation, signature verification. These are valid comparisons of raw
performance, but they do not capture the design decisions each paradigm
imposes once the application has **persistent state**, **global
invariants**, and **multiple authorities**.

Our working hypothesis — confirmed by the results — is that the most
consequential differences between paradigms are not about performance but
about **expressiveness and soundness risk**, and that they surface only
when implementing a complete application.

### 1.3 Contributions

1. **Implementation of the same settlement circuit across five
   paradigms**, with measurements under identical conditions (§7).
2. **Eight design findings** absent from the comparative literature (§8),
   including AIR's lack of copy constraints and its consequence for state
   updates.
3. **A complete settlement layer** with verifiable monetary properties, an
   authority hierarchy, and supervision by selective disclosure (§4–§6).
4. **Explicit documentation of limitations**, including errors of our own
   detected and corrected (§9), and of the verification methodology
   employed (§10).

### 1.4 What this work does not contribute

Stated for clarity, and because the delimitation is itself part of the
contribution:

- **No new cryptographic primitives.** We use established constructions.
- **No distributed consensus.** The architecture is single-node, with the
  consequences analyzed in §11.
- **This is not a security audit.** The system has not been reviewed by
  third parties.
- **Measurements come from a single run on one machine.** They support
  order-of-magnitude comparison, not benchmarking.

---

## 2. Technology: proof systems and their selection

### 2.1 The criterion that proved decisive

The five paradigms evaluated differ along several dimensions — proof size,
proving time, verification time, cryptographic assumptions — but one
proved decisive for this application: **whether they require a trusted
setup ceremony**.

Groth16 requires a per-circuit ceremony; PLONK/KZG a reusable universal
one. In both cases, a set of participants generates parameters from a
secret value that must then be destroyed. **If all participants collude
and retain that secret, they can forge proofs.**

In a settlement application, this capability translates into something
concrete: **creating money without leaving a detectable trace**. Forged
proofs verify correctly. No subsequent detection mechanism exists.

For infrastructure whose stated purpose includes independence from third
parties, this dependency is structurally incompatible: it is permanent —
it does not expire — and unauditable — one cannot verify that the secret
was destroyed.

### 2.2 A decision taken against the performance figures

Of the five paradigms, two dispense with a ceremony: Halo2/IPA and
STARK/FRI. Of these two, STARK/FRI is superior in both proving and
verification, and adds a further property: **post-quantum security**,
depending only on hash functions.

The cost is substantial. The STARK proofs of the comparison circuit
occupy 36.7 KB against Groth16's 192 bytes — a factor of **320×**. The
layer's production circuits are larger —**53.6 to 65.3 KB**, measured
in §218— so the real factor is around 300-350×. In proving time for
the complete circuit the difference favors STARK, but proof size
constrains any scenario where proofs must be transmitted or stored at
volume.

This is, in our work, the only design decision taken explicitly **against**
the performance results.

### 2.3 Arithmetization: R1CS, Plonkish, and AIR

The three arithmetization models differ in how they express constraints,
and this difference has consequences we analyze in §8.

**R1CS** (Groth16) expresses computation as rank-one constraints. Each
variable exists once; equality between variables is direct.

**Plonkish** (Halo2, PLONK) organizes computation into a table with gate
constraints and **copy constraints** that enforce equality between
arbitrary cells.

**AIR** (STARK) expresses computation as constraints over transitions
between consecutive rows of an execution trace. **It has no copy
constraints**: the only directly expressible relation is between one row
and the next.

This absence, unremarkable in stateless circuits, turned out to be the
most significant finding of this work (§8.1).

---

## 3. Architecture

### 3.1 Overview

The system is structured in three levels:

**Circuits** (AIR arithmetization over the Goldilocks field). Eight
circuits proving the properties of each operation type: settlement,
issuance, destruction, audit, threshold, recovery, governance, and
freezing.

**State layer.** Maintains the Merkle trees, chains roots between
operations, verifies proofs, and applies transitions. It holds no private
keys.

**Client.** Generates proofs. **The spend key never leaves the holder's
machine**: the layer supplies authentication paths and the client builds
the proof locally.

> ⚠️ **Correction note (fourth revision).** Until 31 July 2026 this was a
> property **of the design**, not of the system: the layer **did not verify
> the proofs** of the payment path before applying the transition, so the
> key was not needed to move someone else's funds. Earlier revisions stated
> the property without that caveat. The defect is fixed and measured; full
> record in `AUDITORIA.md` §73.

### 3.2 Separating proving from applying

All operations are split into two acts:

```
prove → produces a proof, does NOT modify state
apply → verifies the proof and, if valid and consistent with
        the current state, applies it
```

The separation is deliberate and has a relevant consequence: it allows
**the party producing a proof and the party accepting it to be
different**. This is the condition for an external supervisor to verify a
settlement without ledger access, and for proof generation to occur
client-side.

### 3.3 State

State consists of three sparse Merkle trees and several public scalars:

| Structure | Depth | Contents |
|---|---|---|
| Account tree | 32 | Leaf = `H(H(id, balance), nonce)` |
| Pending tree | 32 | Sent commitments not yet claimed |
| Frozen tree | 24 | Blocked accounts |

Public scalars: total supply, regulatory limit, issuance cap, custodian
set root, governance set root, and three intervention counters
(recoveries, governance changes, freezes).

The frozen tree's depth of 24 is not arbitrary: it is the maximum whose
Merkle climb fits within the unused rows of the settlement circuit without
doubling the trace length, which would have doubled the proving cost of
every transfer.

---

## 4. Merkle structures and their constraints

### 4.1 Sparse trees

All three trees are sparse: only occupied leaves are materialized, and
empty subtrees are represented by their precomputed canonical root. At
depth 32 the address space is 2³² positions, but memory is proportional to
the number of leaves actually occupied.

This representation allows generating authentication paths for both
occupied and free positions, and the latter capability is the basis of
**non-membership** proofs.

### 4.2 Non-membership as a primitive

Two system properties are proven via non-membership, using the same
technique:

| Property | Tree | What is proven |
|---|---|---|
| Account not frozen | Frozen | its leaf is zero |
| Pending position was free | Pending | its leaf was zero before insertion |

⚠️ **Double-spend prevention no longer works this way.** In the one-step
path it was a third non-membership proof —the nullifier's position being
free—; today it rests on **root chaining**, with the total-order dependency
that implies (§4.1 and `AUDITORIA.md` §36).

⚠️ **Note on earlier versions.** The one-step path derived a nullifier's
position from the nullifier itself, which capped practical capacity at
roughly 65,000 payments by the birthday paradox —an availability limit, not
a soundness one. **Earlier English versions of this text did not state that
limit, while the Spanish version did.** The path has since been retired and
the tree removed from the layer, so the limit no longer applies; it is
recorded here because it applied to what those versions described.

The technique consists of climbing from a **zero** leaf to the declared
root. If the position were occupied, its leaf would not be zero and the
climb would not reach that root.

### 4.3 State update: the lockstep pattern

The system's central operation — a transfer — requires proving a state
transition: that starting from a known root and modifying two leaves, one
arrives at another root.

The natural implementation uses two Merkle climbs: one with the old leaf,
one with the new. In arithmetizations with copy constraints, it suffices
to force both climbs to use the same siblings.

In AIR this is not directly expressible, and its absence opens a gap
analyzed in §8.1. The adopted solution — two parallel lanes in the trace,
with a constraint enforcing sibling equality at each link row — is what we
call the **lockstep pattern**.

---

## 5. Verifiable monetary properties

### 5.1 The complete cycle

| Operation | Required authority | Effect on supply |
|---|---|---|
| Issuance | Two distinct custodians, within cap | Increases |
| Transfer | Account holder | **Unchanged** |
| Destruction | Account holder | Decreases |

The asymmetry between transfer and the other two is the central verifiable
property: **money moves without being created**, and only appears or
disappears through operations that record it in a public figure.

### 5.2 Money-creation vectors and their closure

We enumerate exhaustively the vectors by which an adversary could create
money, and the constraint closing each:

| Vector | Closed by |
|---|---|
| Transferring more than debited | Conservation (double entry) |
| Opening an account with a balance | Accounts always open at zero |
| Issuing without authorization | Two custodians proven in-circuit |
| Issuance without supply update | Public supply bound in-circuit |
| Exceeding the issuance cap | Range check on `cap − supply` |
| Double spending | Root chaining (total order of the single node) |
| Spending without being the holder | Proven spend authority |
| Replaying a valid operation | Root chaining |

### 5.3 Threshold authority: the guarantee and its limit

Issuance requires two distinct custodians from a set committed to a public
root. The non-trivial risk is not that an outsider signs — membership
proofs close that — but that **the same custodian counts twice**, which
would turn a 2-of-N scheme into a covert 1-of-N.

This is closed by two interdependent constraints:

1. **Strictly increasing indices**: a range check proves
   `index_b − index_a − 1 ≥ 0`.
2. **Indices bound to paths**: an accumulator reconstructs each index from
   the direction bits of the proven Merkle path.

The second is indispensable: without it, the index would be a declared
value unrelated to the position actually proven, and the first would
guarantee nothing.

**Limit of the guarantee.** In a single-node architecture, whoever
generates the proof needs both keys simultaneously. The guarantee obtained
is therefore **"two keys compromised instead of one," not "two independent
wills."** Genuinely separated authorization — each custodian signing from
their own hardware security module — requires verifying signatures
in-circuit, which is not implemented.

---

## 6. Supervision: selective disclosure

### 6.1 The mechanism

A single circuit proves that `lower ≤ balance ≤ upper`. Varying the
parameters yields three disclosure modes:

| Mode | Configuration | What is revealed |
|---|---|---|
| Exact | `lower = upper = balance` | The balance |
| Minimum | `lower = X`, `upper = MAX` | That it exceeds X |
| Band | `lower = X`, `upper = Y` | That it lies between X and Y |

Band mode satisfies a supervisory requirement — confirming that a position
lies within a range — without revealing the figure.

### 6.2 Structural property: no custodial keys

The proof is generated by **the holder** using their key. The supervisor
verifies it through a free function, without ledger access and without any
master key.

The consequence is that **there is no key to steal** in order to obtain
general access to balances: there is no supervisory backdoor. The
trade-off is that the supervisor depends on the holder's cooperation; if
the holder refuses, the system offers no forced-disclosure mechanism.

This trade-off is deliberate and worth stating: it makes supervision a
cooperative rather than coercive process.

### 6.3 Account freezing

A supervisor may block an account through authorization by two custodians.
The relevant property is **where the restriction is enforced**.

If enforced only by the state layer, it would be equivalent to the
operator refusing to process the operation — a capability it already has —
and would add no third-party-verifiable guarantee.

In our implementation, **the settlement proof attests that the sender does
not belong to the frozen tree** at that state root. Any verifier confirms
this without trusting the operator.

A frozen account retains the ability to **receive**. Preventing this would
strand funds and break legitimate payments toward an account under
investigation.

### 6.4 Intervention counters

The three operations granting discretionary power to custodians — account
recovery, custodian set change, and freezing — increment a **public
counter bound in-circuit**.

The counters do not prevent abuse: no circuit constraint can. They make it
**accountable**, which is the necessary condition for accountability to
exist at all.

---

## 7. Comparative evaluation

### 7.1 Methodology

- The same compliance circuit implemented across all five paradigms.
- All measurements taken from optimized builds.
- Same machine, same execution session.
- No figure is quoted from the literature.

**Explicit limitation**: a single run per measurement, without variance
control or hardware characterization. Observed times for the same
operation ranged from 180 to 620 ms depending on cache state. The figures
support order-of-magnitude comparison between paradigms, not benchmarking.

### 7.2 Results

| | Groth16 | Halo2/IPA | STARK/FRI | PLONK/KZG |
|---|---|---|---|---|
| Arithmetization | R1CS | Plonkish | AIR | Plonkish |
| Ceremony | Per circuit | None | **None** | Universal |
| Setup | 438 ms | 16.3 s | **none** | 26.3 s + 12.8 s |
| Proving | 422 ms | 4.86 s | **39 ms** | 6.85 s |
| Verification | 5 ms | 91 ms | **1 ms** | 8 ms |
| Proof size | **192 B** | 4,096 B | 36.7 KB | 1,008 B |
| Post-quantum | No | No | **Yes** | No |

**Nova/folding** is evaluated separately given its different nature: it
does not produce a final deliverable proof but a folded state requiring
subsequent compression. Marginal cost per transaction: **~250 ms,
constant** (step 9 cost 0.77× step 1). Closing: 1.84 s, amortizable.

### 7.3 Full-layer measurements

| Operation | Prove | Verify | Proof |
|---|---|---|---|
| Startup | **0.67 ms** | — | — |
| Issuance (2-of-N) | ~105 ms | ~2 ms | 57,342 B |
| Transfer | ~620 ms | ~4 ms | 61,966 B |
| Destruction | ~110 ms | ~2 ms | 54,924 B |
| Audit (band) | ~250 ms | ~1.5 ms | 48,782 B |

**Verify/prove asymmetry: 0.5–0.8%.** This is the economic property that
makes the model viable: cost falls on the party producing the proof, not
on the party accepting it.

> ⚠️ **That ratio is the AUDIT DISCLOSURE's, not the transfer's.**
>
> `verify_audit` **only verifies**: 1.6 ms against 274 ms to generate — a
> **0.58 %**. It is the right figure for the argument it supports —a
> supervisor checks without touching state— but **it had been attributed to
> the transfer**.
>
> Applying a transfer costs **28.5 %** of generating it, because `apply`
> **verifies, mutates the tree and writes to disk**. The two are not
> comparable. See `AUDITORIA.md` §22.

**Quantified scaling limit**: one thousand transfers accumulate 126.2 MiB of
proofs. This is the dominant practical constraint of choosing STARK, and
the quantitative argument for recursive aggregation or batched proofs.

---

## 8. Findings

### 8.1 AIR lacks copy constraints

**The principal finding.** Porting state update to AIR reveals a soundness
gap absent from R1CS and Plonkish.

The update requires two Merkle climbs — old leaf and new — from the same
position. Nothing in AIR arithmetization forces both climbs to use the
same sibling nodes. A malicious prover could use different paths and
produce a new root that does not correspond to modifying that position of
the original tree.

**The gap is silent**: an honest witness always uses the same siblings, so
no legitimate proof reveals it. It surfaces only by analyzing which
constraints exist and which do not.

The adopted solution — the lockstep pattern of §4.3 — carries a concrete
cost: it doubles trace width during the climb phase.

**General implication**: porting a circuit from Plonkish to AIR is not a
mechanical operation, even when the logic is identical. Properties obtained
in Plonkish via copy constraints must be redesigned.

### 8.2 The Goldilocks field is too narrow for identities

A Goldilocks field element is 64 bits. An account identity represented by
a single element admits collision in **2³² operations** by the birthday
bound: computationally trivial.

On BLS12-381 (255 bits per element) the issue does not arise, which
explains why it does not appear when designing over that curve. The fix is
to use full four-element digests (256 bits).

⚠️ **That fix is necessary but not sufficient, and earlier revisions
presented it as complete.** Widening the **identity** prevents finding
*another* key colliding with it; **it does not prevent finding *the* key**.
If the secret is still a single element its space is 2⁶⁴ and the identity is
public, so exhausting it offline costs 2⁶³ — measured at 2.38 million
core-years on a CPU without optimising the attack, which is a **loose upper
bound**.

The criterion §8.3 applies to the soundness ceiling — insufficient against
the ~128 bits of the other paradigms — **applies equally to the key space**,
and earlier revisions did not apply it. The complete fix requires **four
elements in the secret too**; it is implemented and measured
(`AUDITORIA.md` §82, §90, §97), and **migration is opt-in**: keys generated
before rotating still have 64 bits.

### 8.3 Soundness ceiling for STARK over Goldilocks

Without field extension, the configuration an implementer would choose by
default — fast and compact — has a ceiling of **63 bits of soundness**,
insufficient and not comparable to the ~128 bits of the other paradigms.

### 8.4 The gap between conjectured and provable security

In the configurations evaluated, 127 bits of **conjectured** security
coexist with 29–63 bits of **provable** security. Closing the gap raises
proof size from 36.7 KB to 125.6 KB.

The distinction is rarely made explicit in cross-paradigm comparisons, and
is directly relevant to anyone selecting parameters under regulatory
criteria.

### 8.5 PLONK/KZG proved the slowest prover

Counterintuitive for a system frequently presented as an industry
standard: 16–22× slower than Groth16 in proving.

**Methodological caveat**: part of the difference may be attributable to
implementation (`dusk-plonk` versus `arkworks`), and our data do not allow
separating the two effects. This is precisely the phenomenon zk-Bench
identifies.

### 8.6 Only two of six libraries defend against unsafe use

Of the six libraries evaluated, only `nova-snark` **prevents in code** a
single-party setup in production builds. `risc0-zkvm` permits receipts
without cryptographic integrity but offers an explicit flag to block them,
naming the concrete scenario — a forgotten environment variable — that
causes such failures in practice. The remaining four permit it silently and
rely on documentation.

This is an axis no performance table captures, and it distinguishes a
library designed for production from one designed for publishing results.

### 8.7 The Rust PLONK/KZG ecosystem is vertically fragmented

Six implementation routes investigated, five unviable: unpublished
packages, hash functions without a specification for the required curve,
unpinned git dependencies, and dependency chains requiring non-stable
compilers.

### 8.8 A zkVM is not comparable on equal terms

RISC Zero was evaluated as a sixth paradigm. Its conceptual fit was
favorable: it uses STARK over Goldilocks — the same system and field as
our chosen backend — which would have isolated a single variable: how the
logic is expressed.

However, it requires an external toolchain to compile the guest program,
violating the methodological criterion applied to the other five
(installation via package manager only). This is not a defect: it is the
price of compiling arbitrary programs.

The figure that quantifies it: **3 dependencies versus 349**.

---

## 8.bis The two-phase path and its fit with ISO 20022

### The payer needed the recipient's balance

A settlement that updates **both leaves** in one transition requires whoever
builds the proof to know **both balances**. Confidentiality against third
parties holds; against the **counterparty** it does not: **paying someone
reveals what they hold**.

This is not an implementation defect. It is a property of the statement being
proven, and it appears in all five proof systems, because a proof about two
accounts requires knowledge of two accounts.

It matters more than its size suggests: **the operator is one declared party**
whose powers are counted and audited; **a counterparty can be anyone**.

### The design that closes it

| Phase | What happens | Whose balance is needed |
|---|---|---|
| `send` | Value leaves the payer into a **pending commitment** | Payer's only |
| `claim` | The recipient makes it theirs | Recipient's only |

> ⚠️ **Correction note (fourth revision).** Until 30 July 2026 the claim
> circuit **did not bind the commitment to the claimant's identity**:
> anyone holding the notice could claim it. Earlier revisions described
> claiming as a proof of ownership — what the design intended and the
> implementation did not enforce. Fixed and measured; see `AUDITORIA.md`
> §27 and §39.1.

The commitment binds the recipient's public identity, a payer-chosen random
value, and the amount. **Neither phase reads the other party's balance**, and
it is enforced by the signature: there is no parameter through which one
could be passed.

⚠️ **Costs, stated.** The payment is not final until claimed; if the recipient
never claims, the value is immobilised and **no return path is implemented**;
and the payer, having chosen the random value, can recompute the commitment
and observe **when** it is claimed — not how much the recipient holds, but a
timing signal the design does not remove.

### No nullifier, and why

The two-phase circuits **omit the nullifier deliberately**: a send changes the
balance, hence the leaf, hence the accounts root, so a resend starts from a
stale root and is rejected.

⚠️ **Distributed consensus would change this**: root chaining requires a total
order, and the nullifier detects a repeated spend without needing one.

### The fit with ISO 20022

A `pacs.008` produces a `pacs.002` carrying the proof, and rejections carry
codes from `ExternalStatusReason1Code` rather than proprietary strings.

The two-phase model **uses the standard's own vocabulary**:

| Phase | Status | Meaning in the standard |
|---|---|---|
| Send accepted | `ACSP` | Accepted, settlement in process |
| Claim applied | `ACSC` | Settlement completed |
| Rejection | `RJCT` | With its reason code |

⚠️ **One piece is missing.** The recipient needs the pending position, the
random value and the amount in order to claim, and **ISO 20022 has no field
for them**. The implementation returns them alongside the message, not inside
it. **How that side channel is operated is unresolved.**

---

## 9. Errors of our own, detected and corrected

Documenting these is part of the methodological contribution: work without
documented errors typically indicates that verification was superficial.

**Comparing across build profiles.** A preliminary version of the
measurements compared debug-build figures against optimized ones, making
STARK appear 130× faster than Groth16 when the true ratio is ~11×.

**Private nullifier.** An initial version kept the nullifier private,
which prevented the state layer from maintaining its tree.

**Nullifier not inserted.** The apply operation did not insert the
nullifier into the tree, which would have rendered vacuous the system's
most expensive guarantee.

**Empty constraints.** While implementing the freezing circuit, two
constraints were written as identically-zero placeholders. An
identically-zero constraint **is always satisfied and fails no negative
test**; it was caught by manual review, not automated checking.

**Non-discriminating tests.** On three occasions a negative test failed
due to a constraint other than the one it purported to verify. These were
corrected by constructing internally consistent witnesses that violate only
the constraint under test.

---

### Substitution without contrast

Introducing the two-phase path and routing the ISO bridge through it
**silently dropped two properties** the original path had:

1. **The regulatory limit stopped being enforced in the circuit.** The old
   circuit carries it as a public input and proves `amount ≤ limit`; the new
   one did not, so the limit was checked only at generation time — bypassable
   by anyone constructing their own trace.
2. **Operations left no trace in the transition log.** The two-phase module
   was **the only one that appended nothing**, and it had become the sole
   institutional path.

Both surfaced by migrating the old path's tests and asking what each had
defended. Neither was found by the new path's own tests, written while
looking only at the new path.

> **Substitution is not only writing the replacement. It is contrasting what
> the original did.**

### Properties proven on a model that is not executed

A prototype module carried eight tests demonstrating the design's properties.
Production uses **one function** from it and none of its data structures.
Contrasting the eight against the executed path found one security property
—that claiming an amount other than the committed one is rejected— verified
**only on the model**.

> **A security property proven on a model is not proven on what runs.**

### Restart tests that compare rather than attack

Of twelve restart tests, **eleven compared a value** before and after; one
attempted the forbidden operation.

Converting one of the eleven found that the custodian quota's **maximum** was
not persisted while its **counter** was: restarting the node renewed an
exhausted quota, lifting any restriction placed on a set under suspicion.

> **Comparing a restored value is an indicator. Attempting the operation it is
> supposed to block is the property.**

### Tests declared versus tests executed

A test written into the wrong scope compiled, did not register, and **executed
nothing**. It was detected only by comparing declared test attributes against
tests executed.

> **A test that does not appear in the run list is invisible: it does not fail
> and does not warn.**

### Where these were found

⚠️ None came from the tools built for the audit —a vacuous-constraint detector
by mutation and an unfilled-column checker—. The claim in earlier versions
of this text —that between them they found no defect across twelve
production circuits— was wrong on two counts, both documented in the
audit: the mutation sweep covered **eleven** circuits, not twelve —its
report on `circuit_audit` ran against an invalid reference trace, and the
self-check that would have caught it is a `debug_assert` that never
executed because all documentation specified `--release`—, and two of the
twelve belong to a design path since retired, so "production" meant ten.
Across the eleven actually covered, neither tool found a defect.

That the published figure was 12 and the real one 11 is itself a finding
of the kind this work aims to contribute: **a verification tool whose
self-check no one has run does not say what it appears to say.**

All came from asking **what each check defends**, and then attempting the
thing it is supposed to prevent.

---

## 10. Verification methodology

Each security property is verified by a **discriminating test**: an
internally consistent witness that violates only the specific constraint
under test. An indiscriminately corrupted witness breaks several
constraints at once, and the test passes even if the constraint of
interest enforces nothing.

Additionally, several tests include **verification of the test itself**:
checking that the test can fail. For instance, the test verifying that
balances are not readable on disk is accompanied by one verifying that
**without encryption they are**; without the second, the first would pass
even if the search were malformed.

This discipline caught a concrete case: an encryption test was failing, and
the cause was not a leak but a test value exceeding the issuance cap, so
that the state was never created.

---

## 11. Absence of consensus: analysis

This section analyzes the principal limitation of the work. We include it
as a section rather than a footnote because it delimits the scope of all
preceding guarantees.

### 11.1 The operator and its three capabilities

The implemented architecture is **single-node**. Its operator holds three
distinct capabilities:

| Capability | Mitigated? |
|---|---|
| Observing all balances | **No** |
| Ordering operations and censoring | **No** |
| Rewriting history | **Yes** (§11.3) |

The first two are inherent to the architecture: whoever maintains state
knows it, and whoever processes operations controls their order. **No
cryptographic construction eliminates them without replicating state
across mutually distrusting parties** — that is, without consensus.

### 11.2 What is proven and what is not

The distinction is precise and worth stating explicitly:

| Claim | Proven? |
|---|---|
| This transfer conserves money | Yes |
| This account's balance is X | Yes |
| No one spends without being the holder | Yes |
| No one double-spends | Yes |
| **This is the system's current state** | **No** |
| **These are all the operations that occurred** | **No** |

**State transitions are proven. State itself, and history completeness,
are not.**

### 11.3 Partial mitigation: chained transition log

Without addressing consensus, the third capability can be closed by a
chained log.

Each applied operation generates an entry:

```
digest_n = H(n, kind, root_old, root_new, H(proof), digest_{n-1})
```

Chaining ensures that altering an old entry invalidates all subsequent
digests. **Publishing the head digest — 32 bytes — commits to the entire
history**: two copies with the same head share the same history, and any
subsequent rewrite separates them.

This is the construction Certificate Transparency employs against
certificate authorities: it does not prevent misbehavior, it makes it
**detectable after the fact**.

**Limitations of the mitigation**: no one is obliged to observe; the
operator could decline to publish the log; and censorship leaves no trace,
because an operation never processed generates no entry, and its absence
is indistinguishable from its never having been requested.

### 11.4 An asymmetry revealed by the log

Constructing the log revealed a property not previously noticed: **opening
an account is the only state transition that generates no proof**. It
creates no money — the account opens at zero balance — but it does modify
the state root.

In the current implementation the operation generates a log entry with a
null proof digest, making it explicitly distinguishable to a verifier:
that transition is **recorded but not proven**.

---

## 12. Related work

**Deployed systems.** Zcash has implemented private transactions since
2016 on a more complete design than the one presented here. Aztec, Aleo,
and Miden develop zero-knowledge infrastructure with substantially larger
teams and time horizons.

**Comparative evaluation.** zk-Bench provides rigorous evaluation of
Groth16, PLONK, halo2, and starky. Our work differs in the object measured
— a complete application rather than reference circuits — and in the type
of result: design findings rather than performance metrics.

**Zero-knowledge authorization.** Recent work addresses nullifier-based
authorization evaluated across multiple backends.

**Institutional initiatives.** Central bank digital currency programs —
Drex, mBridge, the Eurosystem pilots — address similar requirements with
incomparably greater resources. This work does not compete with them; it
contributes measurements on a specific question those programs must also
resolve.

---

## 13. Conclusions

Implementing the same settlement application across five proof paradigms
reveals differences that comparisons over reference circuits do not
capture. The most significant is that AIR arithmetization, lacking copy
constraints, requires redesigning state updates with a specific pattern,
and that omitting it produces a soundness gap invisible to honest
witnesses.

The paradigm decision was determined by a criterion absent from performance
tables: **whether a trusted setup ceremony is required**. For a settlement
application, such a ceremony constitutes a permanent and unauditable
dependency whose compromise permits creating money without detectable
trace.

Measurements show that verification costs 0.5–0.8% of proving — an
asymmetry that makes the model viable — and quantify its principal limit:
**126.2 MiB of accumulated proofs per thousand transfers** — unit
corrected in the fourth revision: the figure was always binary and was
labelled "MB"; in SI units it is 132.3 MB.

⚠️ That asymmetry is measured on **audit disclosures**, which verify without
mutating state — precisely the supervisory case. A transfer's apply step
costs 28.5 % of proving, because it also writes.

The results also delimit what has not been demonstrated. The single-node
architecture means that state transitions are proven while the current
state and history completeness are not, and closing that gap requires
distributed consensus, which belongs to a different discipline.

### Future work

- **Distributed consensus**, prerequisite for the guarantees of §11.
- **External security audit**, not performed.
- **Recursive aggregation or batched proofs**, for the limit of §7.3.
- **In-circuit signature verification**, for genuinely separated
  authorization (§5.3).

---

## 14. Availability and reproducibility

The complete implementation is available at:

**`https://github.com/USER/REPOSITORY`**

It requires only the stable Rust compiler; no external toolchains or
non-stable compilers are used.

```bash
# Or all at once, with the canon's pinned counts checked:
bash tools/canon.sh --sello

cargo test -p zk-ssl --release              # layer: 285 tests (3 ignored)
cargo test -p stark-experiment --release    # circuits: 297 (10 ignored)
cargo test -p zk-ssl-node --release         # node: 31
cargo test -p zk-ssl --release metrics -- --nocapture
```

The repository includes an audit-preparation document with the threat
model, a table of invariants and their enforcement points, and an explicit
section listing the aspects in which the authors have least confidence.

---

## Acknowledgments

[To be completed]

## References

[To be completed with citations for: Groth16, PLONK, Halo2, STARK/FRI,
Nova, Rescue-Prime, zk-Bench, Certificate Transparency, Zcash, and the
documentation of the libraries used.]
