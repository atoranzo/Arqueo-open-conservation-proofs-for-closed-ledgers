# Comparative Implementation of a Zero-Knowledge Settlement Layer across Five Proof Systems

<p class="sub">Design Findings and Measurements</p>

<div class="meta">
<p><strong>Author:</strong> Angel Jose Toranzo Portela &nbsp;·&nbsp; <strong>Affiliation:</strong> Independent &nbsp;·&nbsp; <strong>DOI:</strong> 10.5281/zenodo.21693706</p>
<p><strong>Repository:</strong> https://github.com/atoranzo/ZK-SSL-ZK-Sovereign-Settlement-Layer-</p>
<p><strong>Companion preprints:</strong> <em>Provable Compliance without Full Ledger Disclosure</em>, 10.5281/zenodo.21693709; <em>From Institutional Trust to Verifiable Properties</em>, 10.5281/zenodo.21693718.</p>
<p><strong>Code artifact:</strong> executable tests and measurement harnesses in-repo</p>
<p><strong>Version:</strong> third revision, July 2026. The second revision added §3.6, §7.5, §7.6 and §8, recorded the retirement of the single-step settlement path, and re-measured the cost figures on the two-phase path. This revision corrects the mutation-coverage figure of §8.5 from twelve circuits to eleven, reports that the nullifier tree described in §7.5 as retained dead weight has since been removed with a verified migration, adds a fifth methodology error class in §8.6, and re-measures the test counts of §9. Changes that reverse an earlier claim are marked in place rather than removed.</p>
</div>

**Keywords:** zero-knowledge proofs, STARKs, Groth16, Halo2, PLONK, Nova, financial settlement, trusted setup, AIR arithmetization, selective disclosure, confidential settlement

## Abstract

We present a comparative implementation of the same financial settlement circuit in five zero-knowledge proof systems — Groth16, Halo2/IPA, STARK/FRI, PLONK/KZG, and Nova/folding — and report the design consequences that appear only when the comparison is performed on a complete stateful application rather than on microbenchmarks.

The application is a minimal settlement layer with double-entry value conservation, spending authority, double-spend prevention, threshold issuance, burns, selective disclosure for supervision, and account freezing. Unlike benchmark suites centered on primitives such as SHA-256, this setting forces persistent state updates, global invariants, and multi-authority rules into every backend.

We document nine implementation findings absent from typical comparative tables. The most consequential for arithmetization is that **AIR lacks native copy constraints**, which creates a silent soundness gap when updating Merkle state unless a lockstep dual-climb pattern is enforced. The most consequential for the application is that **single-step settlement discloses the recipient's balance to the payer**, which forced a two-phase transfer design. We also report identical-condition measurements for setup, proving, verification, and proof size; explain why Groth16 was rejected despite superior proof size; and record methodology errors that were detected and corrected during the work.

The reference implementation is public and test-backed. It is not third-party audited and does not implement distributed consensus. We treat both limitations as part of the result.

## 1. Introduction

### 1.1 Problem

Financial settlement must simultaneously support confidentiality of positions and transfers, and verifiable compliance with monetary and supervisory rules.

Institutional systems resolve the tension by trusting an intermediary that sees the ledger. Zero-knowledge proofs offer another route: demonstrate conservation, authorization, and policy compliance without disclosing full state to every verifier.

The engineering question is not whether ZK can prove isolated statements. It is whether **a full settlement application can be ported across proof paradigms without changing the security meaning of the system**, and what design costs appear only then.

### 1.2 Why microbenchmarks are insufficient

Existing comparative studies of proof systems are valuable, but most measure reference circuits. Primitive-level benchmarks answer questions about raw prover/verifier performance. They do not expose:

- stateful Merkle updates,
- non-membership proofs for freezes and for unused commitment positions,
- multi-tree invariants,
- authority thresholds,
- selective disclosure modes,
- soundness gaps that appear only when the same logical transition is re-arithmetized.

**Hypothesis.** The most important differences among proof systems for settlement are differences of **expressivity and residual trust**, not only differences of milliseconds and bytes. These differences become visible only on a complete application.

### 1.3 Contributions

1. Implementation of one settlement application across five proof paradigms.
2. Same-machine, release-mode measurements for the compared backends.
3. Nine design findings from stateful porting, including the AIR copy-constraint gap and a confidentiality leak intrinsic to single-step settlement.
4. An explicit architectural decision against ceremony-dependent proving for monetary soundness.
5. Public tests and documentation of limitations and corrected methodology errors.

### 1.4 Non-contributions

- No new cryptographic primitive.
- No distributed consensus.
- No third-party security audit.
- No claim that single-run local timings are industry benchmarks.

## 2. Systems under comparison and selection criterion

### 2.1 Backends

| System | Arithmetization | Trusted setup | Notes |
|---|---|---|---|
| Groth16 | R1CS | Per-circuit | Very small proofs |
| Halo2/IPA | Plonkish | None | Transparent polynomial commitment via IPA |
| STARK/FRI | AIR | None | Hash-based; post-quantum oriented |
| PLONK/KZG | Plonkish | Universal SRS | Reusable setup if ceremony exists |
| Nova | Folding / IVC-style | Setup-dependent in measured stack | Constant per-step proving profile |

### 2.2 The decisive criterion

For monetary settlement, a proving system that allows undetectable forgery under setup collusion is not merely slower or faster. **It changes the trust model.**

If a ceremony trapdoor is retained, forged proofs can validate false transitions. In a settlement layer, that can mean value creation without an auditable event.

Therefore the design rule was:

> Prefer transparent proving over better compression when compression reintroduces an unauditable monetary trust dependency.

This rule, not prover latency, determined the production-facing choice.

### 2.3 Decision against Groth16

Groth16 produced the smallest proofs in our measurements and strong proving performance. It was still rejected for the settlement path because per-circuit trusted setup is incompatible with the sovereignty objective of the layer.

This is the only major design decision taken explicitly **against** the performance tables.

## 3. Settlement architecture

### 3.1 Components

**Circuits.** Statements for single-step settlement, two-phase send and claim, mint, mint-to-pending, burn, audit disclosure, threshold authority, recovery, governance, and freeze.

⚠️ The single-step settlement circuit is **retained in the experiment crate and no longer reachable from the layer**. It is kept because it is the artifact this paper's comparison was performed on; the production path is the two-phase pair of §3.6.

**State layer.** Maintains Merkle trees and public scalars, verifies proofs, and applies accepted transitions. It does not hold spending keys.

**Client.** Constructs proofs locally. Spending keys remain on the client.

### 3.2 Prove/apply separation

Every operation is split:

1. **prove** — generates a proof; does not mutate state.
2. **apply** — verifies the proof against current state and, if valid, mutates state.

This enables client-side custody of spending keys, external verification of disclosures without ledger access, and cleaner institutional role separation.

⚠️ **Correction.** An earlier version of this paper stated the above without
qualification. Until 31 July 2026 the layer's `apply_send`, `apply_claim` and
`apply_mint_to_pending` **did not verify the proof at all** before mutating
state, so the payment path did not enforce what §3.1 describes: a spending
key was not needed to move someone else's funds. The defect is fixed and
measured.

### 3.3 State structures

State comprises:

- accounts tree,
- pending-transfers tree,
- frozen-accounts tree,
- public scalars for supply, caps, authority roots, and intervention counters.

Sparse Merkle trees support both membership and non-membership proofs. **Non-membership is used for freeze checks and to prove that a pending position was unused before insertion.**

⚠️ **Earlier versions of this paper listed a fourth tree — the nullifier tree — and described non-membership as the mechanism for double-spend prevention.** Both statements described the single-step path. That path has been retired, and the nullifier tree has since been removed from the layer entirely (§7.5). Double-spend and replay prevention now rest on the chaining of state roots, which is sufficient under a single total order and, as §7.5 states, not sufficient without one.

### 3.4 Monetary invariants

| Operation | Authority | Supply effect |
|---|---|---|
| Mint | authorized issuer set | increases |
| Transfer | account holder | unchanged |
| Burn | account holder | decreases |
| Audit disclosure | account holder | none |

The central invariant is that **transfers conserve value**. Creation and destruction are explicit, authorized events.

### 3.5 Selective disclosure

Audit proofs support:

- exact balance,
- minimum balance,
- balance band.

A supervisor verifies the proof without receiving the full ledger.

*(Editorial note: in the previous version this subsection was printed after §3.6. The order is corrected here; no content changed.)*

### 3.6 Two-phase transfer

A single-step transfer updates both account leaves in one proof, so the party constructing that proof must know the recipient's balance in order to compute their new leaf. **Paying someone one euro therefore reveals how much they hold.**

The layer's operator is one declared party; a counterparty can be anyone. The two-phase design removes the second exposure:

| Phase | What happens | Whose state is touched |
|---|---|---|
| `send` | Value leaves the payer and rests in a pending commitment | Payer only |
| `claim` | The recipient makes it theirs | Recipient only |

The pending commitment binds the recipient's public identity, a payer-chosen salt, and the amount. Neither phase reads the other party's balance, and the property is **enforced by the function signature**: there is no parameter into which a counterparty balance could be passed.

⚠️ **Correction.** That signature argument holds for the *balance*, and an
earlier version of this paper let it stand for more than it covers. Until 30
July 2026 the claim circuit **did not bind the commitment to the claimant's
identity**: anyone holding the notice could claim it, and no function
signature prevented that because it was not a signature problem. Fixed and
measured.

**Costs, stated.** The payment is not final until claimed; if the recipient never claims, the value is immobilised, and a return mechanism is not implemented. The payer chose the salt, so they can recompute the commitment and observe **when** it is claimed — not how much the recipient holds, but a residual timing signal that the design does not remove.

**No nullifier.** The two-phase circuits omit nullifiers deliberately: a send changes the payer's balance, hence their leaf, hence the accounts root, so a replay presents a stale root and is rejected.

**The single-step path has been retired entirely** — the layer's `transfer` / `apply` pair, the client-side `prove_transfer`, and their materials type. Nothing generates a nullifier, which closes the capacity bound of §7.5 at the cost of depending on a total order that only a single node provides.

Both phases can be proven client-side. The layer hands out paths and roots; free functions build the proofs with the spending key; the layer verifies. **Neither the payer's key nor the recipient's reaches the operator**, and the materials type for a send carries the recipient's identifier and nothing else — there is no field through which a balance could travel.

## 4. Measurement setup

All comparative numbers were produced from the same settlement logic, on the same machine, in release mode. Figures are single-run order-of-magnitude measurements, **not** multi-sample statistical benchmarks.

Reported dimensions: setup / parameter generation, proving time, verification time, proof size, and qualitative properties (ceremony requirement, post-quantum posture, engineering constraints).

Exact tables should be taken from the repository measurement documents and reproduced by the published artifact commands.

Illustrative comparative profile from the project measurements:

| Backend | Ceremony | Approx. prove | Approx. verify | Proof size | PQ-oriented |
|---|---|---|---|---|---|
| Groth16 | per-circuit | ~0.42 s | ~5 ms | 192 B | no |
| Halo2/IPA | none | ~4.9 s | ~91 ms | ~4 KB | no |
| STARK/FRI | none | ~0.04 s | ~1 ms | ~37 KB | yes |
| PLONK/KZG | universal | ~6.9 s | ~8 ms | ~1 KB | no |
| Nova (step) | setup-dependent stack | ~0.25 s / step | separate folded state | — | no |

These values are orientation numbers from the reference harness and must be reproduced from the artifact for any formal citation.

## 5. Design findings from stateful porting

The following findings emerged while implementing the same application across backends. Several do not appear in primitive-only comparisons.

### Finding 1 — AIR has no native copy constraints

In Plonkish and R1CS, equality across non-adjacent wires/cells is directly enforceable. In AIR, constraints naturally relate consecutive rows of an execution trace.

When a transfer updates Merkle state, a natural implementation climbs the tree twice: once for the old leaf and once for the new leaf. **Without enforced equality of sibling paths, a malicious prover can mix inconsistent siblings and argue a false root transition.** Honest-witness testing does not expose this gap.

*Mitigation:* a lockstep dual-climb pattern, forcing siblings to match level by level in parallel lanes of the trace.

*Consequence:* porting from Plonkish/R1CS to AIR is not mechanical, even when business logic is identical.

### Finding 2 — Field width matters for identities

Using a 64-bit field for identity-bearing values creates collision-scale risk unacceptable for account identity constructions. Identity-grade material required wider digests.

⚠️ **Correction — the fix stated above is necessary but not sufficient.**
Widening the *identity* prevents finding **another** key that collides with
it; it does not prevent finding **the** key. If the secret remains a single
element its space is 2^64 and the identity is public, so exhausting it
offline costs 2^63 — measured at 2.38 million core-years on one CPU without
optimising the attack, which is a **loose upper bound**. The criterion
Finding 3 applies to the soundness ceiling applies equally here, and earlier
versions of this paper did not apply it. The complete fix requires four
elements in the **secret** as well; it is implemented and measured, and
migration is opt-in — keys generated before rotating still carry 64 bits.

### Finding 3 — Conjectured vs provable STARK security can diverge substantially

Fast/compact STARK parameterizations may advertise high conjectured security while delivering much lower provable security unless extension fields / query parameters are strengthened. Closing the gap increases proof size.

### Finding 4 — Proof-system choice is also library-safety choice

Some libraries silently allow insecure local setup paths. Others actively block production use of test setup. For monetary systems, **API hazard is a security property**.

### Finding 5 — Universal setup is not the same as no setup

PLONK/KZG can amortize a universal SRS across circuits, which is operationally better than per-circuit ceremonies. It still preserves a ceremony trust assumption. That is preferable to repeated ceremonies, but not equivalent to transparent proving.

### Finding 6 — Measured PLONK performance can be dominated by implementation stack

In our harness, the measured PLONK/KZG path was markedly slower than Groth16. Part of the gap may be implementation-specific rather than scheme-intrinsic. **Comparative claims must distinguish scheme from stack.**

### Finding 7 — zkVMs change the comparison basis

A zkVM can use similar proving technology but introduces toolchain and dependency surface that breaks equal-footing comparison with circuit-specific libraries installed via ordinary package flow. Generality has a measurable engineering cost.

### Finding 8 — Nova's profile is operationally different, not merely faster or slower

Folding gives an approximately constant per-step proving cost and a separate compression/closing cost. That profile maps to batch settlement better than to one-shot public proof replacement. It is a different systems shape.

### Finding 9 — Single-step settlement discloses the recipient's balance

This finding is not about a proof system. **It appears in all five**, because it is a property of the statement being proven, not of how it is proven.

A settlement that updates two account leaves in one transition requires the prover to know both leaves. Confidentiality against third parties is preserved; confidentiality against the counterparty is not.

Microbenchmarks cannot surface this: a hash circuit has no counterparty. It appears only when the application has account state and two parties.

*Mitigation:* split the transition, as in §3.6. The cost is finality latency and an immobilisation risk, not proving cost.

*Consequence:* for settlement, the confidentiality question is not only "what does a verifier learn?" but **"what does the person paying you learn?"**.

## 6. Why the settlement path chose STARK

Given the application objective — verifiable monetary transitions with minimized unauditable trust — the production-facing path preferred:

1. no per-circuit ceremony trapdoor,
2. competitive proving/verification latency in the measured harness,
3. post-quantum oriented assumptions,
4. acceptance of larger proofs.

Groth16 remains attractive where ceremony governance is acceptable and proof size dominates bandwidth or storage. For a layer whose claim includes reducing hidden monetary trust, ceremony rejection is rational even at a 100×–300× proof-size penalty.

## 7. Residual system limits

### 7.1 Single-node operator residual trust

The current layer does not implement distributed consensus. A single operator may still observe state, order transactions, censor, and act as an availability single point of failure. **Cryptographic transition validity does not erase operational power.**

### 7.2 No third-party audit

Executable tests, including discriminant negative tests, improve engineering confidence. They do not replace external adversarial review.

### 7.3 Measurement limits

Single-machine, single-run timings are useful for order-of-magnitude design decisions. They are not substitutes for multi-environment statistical benchmarks.

### 7.4 Institutional limits

Proof validity is not legal finality. Sanctions, identity, and crisis interventions remain institutional processes.

### 7.5 Capacity limits

Three capacity limits were quantified in a later audit pass. None is a soundness failure; all three are availability bounds. **One has since been removed**, and the sequence is reported rather than deleted.

#### Nullifier position collisions — removed

A nullifier's position in its sparse tree was derived from the nullifier itself — a truncation of its first field element — and the circuit required that position to be free. Two distinct nullifiers landing on the same position are a conflict, so effective capacity followed the birthday bound:

| Payments accumulated | Collision probability |
|---|---|
| 10,000 | 1.2 % |
| 65,536 | 39 % |
| 200,000 | 99 % |

The tree advertised 2^32 positions. **The practical bound was roughly 2^16.** The affected payer could not retry, because their nullifier is deterministic from account state, and the layer originally reported the conflict as a double spend — an accusation that was false.

✅ **This bound is now closed, and the mechanism is gone.** The single-step path has been retired entirely and nothing generates nullifiers. Earlier versions of this paper added that *the tree is kept for on-disk format compatibility and is dead weight* — that is **no longer true**. The tree has been removed from the layer, with a **verified migration** rather than a silent discard: on opening a ledger written before the retirement, the legacy entries are reconstructed and checked against the root stored with them, and only then deleted in a single atomic batch; a mismatch, or legacy entries with no stored root to check against, halts startup. Archived snapshots are handled the same way — the on-disk snapshot format advances one version, and the previous version is still importable, its nullifier records verified against the root it declares before being discarded.

⚠️ **The bound was avoided, not solved.** Root chaining replaces the nullifier and requires a total order, which a single node provides and a distributed system does not. **Anyone distributing this recovers the bound intact.** The correct remedy is to index by the full nullifier rather than a truncation, moving the failure mode to the hash's own collision resistance. Widening the truncation moves the bound without changing its nature. Neither is implemented.

#### Custodian set ceiling

Strict ordering between two authorising custodians is enforced by decomposing their index difference in a 7-bit range segment, which caps the set at **128 members**. Beyond that, authorisations between distant indices would fail intermittently — a failure depending on which two custodians sign. The current tree allows 16, so there is headroom; the coupling between the two constants was undeclared until an audit pass pinned it with a test.

#### Proof accumulation

One thousand complete payments accumulate **120.4 MiB**. This is a storage and bandwidth cost, not a stop.

⚠️ **A payment is two proofs.** Each proof is about 62 KB and that has not changed; the two-phase design of §3.6 requires a send and a claim. An earlier version of this paper reported 59.1 MB, which measured the single-step path after that path had stopped being the production one.

⚠️ **And the expensive half is the recipient's.** A claim proof costs roughly 500 ms to generate against 283 ms for a send, because the claim circuit walks both the pending tree and the accounts tree. **The cost of a payment falls mainly on whoever receives it.**

An earlier draft of this paper described proof accumulation as the system's real limit. **That was incorrect:** the collision bound stopped legitimate payments far earlier and permanently, and it did so while the system was nowhere near saturation.

### 7.6 What the verify/generate asymmetry measures

The asymmetry frequently cited for this design — verification costing a fraction of a percent of proving — is the **audit disclosure's** ratio, not a transfer's.

| Operation | Second-step cost / proving | What the second step does |
|---|---|---|
| Audit disclosure | 0.58 % | Verifies only |
| Complete payment | 28.5 % | Verifies twice, mutates trees, writes to disk |

The 0.58 % figure supports the argument it was cited for — a supervisor verifies without touching state — but had been attached to the wrong operation. The correction narrows what the number describes; it does not weaken the economic claim, because supervision is exactly the case that verifies without applying.

## 8. Methodology errors detected and corrected

The paper commits in §1 to reporting corrected methodology errors. Five are reported, because each is a **class** rather than an incident, and each was found by a different mechanism.

### 8.1 Substitution without contrast

Introducing the two-phase path (§3.6) alongside the single-step one, and then routing the ISO bridge through it, silently dropped two properties the original path had:

1. **The regulatory limit stopped being enforced in the circuit.** The single-step circuit carries it as a public input and proves `amount ≤ limit`; the two-phase circuit did not, so the limit was checked only at proof-generation time by the layer — bypassable by anyone constructing their own trace.
2. **Operations left no trace in the transition log.** The two-phase module was the only one that did not append entries, so payments through what had become the sole institutional path were unrecorded.

Both were found by migrating the original path's tests and asking what each had defended. Neither was found by the new path's own tests, which had been written while looking only at the new path.

> **Substitution is not only writing the replacement. It is contrasting what the original did.**

### 8.2 Properties proven on a model that is not executed

A prototype module carried eight tests demonstrating the two-phase design's properties. Production uses one function from that module and none of its data structures. Contrasting the eight against the executed path found one security property — that claiming an amount other than the committed one is rejected — verified **only on the model**.

> **A security property proven on a model is not proven on what runs.**

### 8.3 Restart tests that compare values rather than attempting the attack

Of twelve tests covering node restart, eleven compared a value before and after; one attempted the forbidden operation afterwards.

Converting one of the eleven found that the custodian quota's **maximum** was not persisted while its counter was. Restarting the node renewed an exhausted quota, lifting any restriction placed on a custodian set under suspicion.

> **Comparing a restored value is an indicator. Attempting the operation it is supposed to block is the property.**

### 8.4 Tests declared versus tests executed

A test written into the wrong lexical scope compiled, did not register with the harness, and executed nothing. It was detected only by comparing test attributes **declared** against tests **executed**.

> **A test that does not appear in the run list is invisible: it does not fail and does not warn.**

A second instance of the same class was found later, and it is worth reporting because of how it would have been read. A pre-registered experiment — designed to decide, against criteria written before the result was known, why a set of constraint degrees fail outside release mode — was nested inside another test. It compiled, did not register, and produced an empty output where a failure would have appeared. **The pre-registered decision table would have read that empty output as the experiment's most favourable branch.** Pre-registering a criterion protects against reinterpreting a result; it does not protect against reading the absence of a result as a result. The table needed a fourth row: *no result means no conclusion*.

### 8.5 A note on where these were found

None of the above came from the tools built for the audit — a vacuous-constraint detector by mutation and an unfilled-column checker. All came from asking what a given check defends, and then attempting the thing it is supposed to prevent.

⚠️ **A correction to how those tools' coverage was reported.** Earlier versions of this paper stated that between them the tools found no defect across **twelve production circuits**. Two things were wrong with that sentence, and the audit documents both.

The mutation sweep covered **eleven** circuits, not twelve: its report for the twelfth was generated against an **invalid reference trace**, so the report did not mean what it appeared to mean. The self-check that would have caught this is a debug-mode assertion, and it never executed, because all documentation for this project specifies release mode.

And "production" was inexact: of the twelve, two belong to the design path since retired, so ten were production circuits. Across the eleven actually covered, neither tool found a defect.

> **A verification tool whose self-check no one has run does not say what it appears to say.**

We report this because the tools were the more expensive investment, and because the discrepancy between the published figure and the real one is itself a datum of the kind this paper aims to contribute.

### 8.6 A format change contrasted only against what it renamed

Removing the nullifier tree (§7.5) changed both the on-disk layout and the archived snapshot layout. The change was prepared by sweeping the codebase for every reference to the retired mechanism **by name**, and the sweep was clean.

A test then failed that the sweep could not have found. It verified that a tampered snapshot is rejected, and it located the byte to tamper with by means of a constant encoding the **geometry** of the header — so many roots, so many counters. The constant does not mention nullifiers; it encodes their size. With one root and one counter fewer, the byte it altered fell outside the account record it claimed to alter and landed in a region whose corruption raises a different error than the one the test asserts. The test passed for years, then stopped testing what it said it tested.

The same test had already broken once in the opposite direction: an earlier version located its byte by counting from the end of the file, and appending a transition log turned that byte into log data.

> **A format change has to be contrasted against every test that encodes offsets, not only against the tests that name what changed.** A sweep by name cannot find a constant that encodes a size.

A companion instance, found in the same pass: a circuit's documentation listed among the statements it proves *"the nullifier has not already been spent"* — twelve lines above a section titled *"Why this circuit carries no nullifier."* The list item was a fragment copied from the retired circuit. A comment block that contradicts itself will not be caught by any test, and it is the part of a system that a reader trusts most directly.

## 9. Reproducibility

The repository provides:

- settlement layer implementation,
- backend experiments,
- test suites — **375 executable tests** across the two production crates, plus one documented skipped test,
- measurement commands,
- architecture and principles documentation, including a standing audit document that records open defects and their cost.

The two production crates run in roughly **30 seconds**. The full workspace, including the comparative backends, is about 550 tests and 22 minutes; **89 % of that time is PLONK and Halo2**, which is itself a restatement of §4.

⚠️ **`--release` is required, not merely recommended.** Winterfell verifies in debug builds that each declared constraint degree is realised by the specific trace being proven. A boolean constraint over a column that happens to be constant in a given witness has degree zero, so **65 of the layer's 174 tests fail in debug mode for a reason unrelated to correctness**.

⚠️ That figure was re-measured for this revision, because it had been stated inconsistently: earlier versions of this paper reported 56, while the repository's audit document reported 65 in one place and 56 in another. The measured value is 65 of 174 (`cargo test -p zk-ssl` without `--release`), and the repository has been corrected to match. **The discrepancy is reported rather than quietly fixed, because a figure that appears with two values in the same project is evidence about the project's own bookkeeping, not a typo.**

A later audit pass identified the cause more precisely: the constant-column problem is concentrated in the pending-transfer family of constraints, whose position is allocated by a counter that begins at zero, and is therefore not corrected by placing test accounts at higher tree indices. **The remedy is a reformulation of those constraint degrees, and it is not implemented.**

A publication package should pin the git commit hash, the Rust toolchain version, a machine description, and the exact commands used for the tables.

Recommended artifact practice:

1. publish the paper text on Zenodo with DOI,
2. link the exact commit,
3. include raw measurement outputs,
4. include a short "How to reproduce in 30 minutes" appendix.

## 10. Related work positioning

Primitive-oriented comparative suites answer performance questions about proof systems. This work answers a different question:

> What design failures and residual trust shifts appear when a **complete settlement application** is forced through multiple arithmetizations?

The distinguishing method is application-level portability under fixed business invariants.

## 11. Conclusion

A zero-knowledge settlement layer can make core monetary properties — value conservation, spending authority, double-spend resistance, and selective supervisory disclosure — mechanically checkable. When the same application is implemented across five proof systems, the decisive differences are not only speed and size. They include expressivity gaps, setup trust, library hazards, and operational residual power.

The central engineering result is that **stateful settlement circuits reveal soundness and sovereignty issues that microbenchmarks miss**. The central design result is that **rejecting ceremony-dependent proving can be the correct monetary choice even when it loses on compression**.

The public artifact is offered as a measurable, limited, and inspectable reference for that trade-off.

What the audit passes reported here suggest is narrower and, we think, more useful than the measurements: on an application of this shape, the defects that survive a green test suite are found by **asking what each check defends, and then attempting the thing it is meant to prevent**. Every finding in §8 came from that question. None came from the tooling built to find them.

To which this revision adds a corollary, drawn from §8.5 and §8.6: **a published figure is a claim like any other, and it decays.** Three of the corrections in this paper are not discoveries about the system but discoveries about its own description — a coverage count that was wrong, a structure described as retained after it was removed, and a constant that encoded a layout it had outlived. A test suite does not check prose.

## References

1. Project repository and measurement documents: https://github.com/atoranzo/ZK-SSL-ZK-Sovereign-Settlement-Layer-
2. Related comparative ZK engineering literature (zk-Bench and system papers to be completed in final bibliography).
3. Backend library documentation used by each experiment crate.

## Appendix A — Suggested Zenodo metadata

- **Title:** Comparative Implementation of a Zero-Knowledge Settlement Layer across Five Proof Systems: Design Findings and Measurements
- **Resource type:** Preprint
- **Subjects:** Computer security; cryptography; financial infrastructure
- **Related identifier:** GitHub commit URL
- **Author:** Angel Jose Toranzo Portela — ORCID if available
- **Affiliation:** Independent
- **Disclaimer:** Not affiliated with, endorsed by, or commissioned by the European Central Bank, the Eurosystem, or any other institution. References to the digital euro are to publicly published designs and are cited as context for a technical problem.
- **License:** CC BY 4.0 (text) + code under repository licenses

## Appendix B — One-paragraph promotion blurb

This paper reports what happens when one complete ZK settlement application — not a hash benchmark — is implemented across five proof systems. The comparison surfaces design findings invisible to microbenchmarks: an AIR copy-constraint gap for stateful Merkle updates, a confidentiality leak intrinsic to single-step settlement that discloses the recipient's balance to the payer, and a justification for rejecting ceremony-dependent proving for monetary soundness despite much smaller proofs. It also reports the methodology errors found in later audit passes, including two properties that were silently lost when one settlement path replaced another, a verification tool whose clean report covered one circuit fewer than claimed, and a format change that broke a test which encoded the old layout's geometry.
