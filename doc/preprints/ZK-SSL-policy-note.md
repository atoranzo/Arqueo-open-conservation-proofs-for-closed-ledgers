# Provable Compliance without Full Ledger Disclosure

<p class="sub">A Zero-Knowledge Settlement Architecture for Supervisory Audit</p>

<div class="meta">
<p><strong>Author:</strong> Angel Jose Toranzo Portela &nbsp;·&nbsp; <strong>DOI:</strong> 10.5281/zenodo.21736082</p>
<p><strong>Repository:</strong> https://github.com/atoranzo/Arqueo-open-conservation-proofs-for-closed-ledgers</p>
<p><strong>Document type:</strong> Technical policy note / preprint &nbsp;·&nbsp; <strong>Affiliation:</strong> Independent</p>
<p><strong>Audience:</strong> supervisors, central banks, fintech risk teams, CBDC researchers, payment architects</p>
<p><strong>Version:</strong> third revision, July 2026. The second revision added §3.4, §3.5, §4.1 and §5.1, extended the residual-trust schedule of §5 with quantified capacity limits, and re-measured the cost figures on the two-phase path. This revision reports that the mechanism behind the collision bound of §5.1 has been removed from the layer with a verified migration, withdraws the conditional caveat in §8 that the two-phase path had not yet become the only path, and re-measures the test count in the artifact list.</p>
<p><strong>No institutional affiliation.</strong> This is independent work. It is not affiliated with, endorsed by, or commissioned by the European Central Bank, the Eurosystem, or any other institution, public or private. References to the digital euro are to publicly published designs and are cited as context for a technical problem. No claim in this document should be read as anyone's position but its author's.</p>
<p><strong>Companion preprint:</strong> the measurement and proof-system comparison behind this note is published separately as <em>Comparative Implementation of a Zero-Knowledge Settlement Layer across Five Proof Systems</em>, 10.5281/zenodo.21736125. Its first version reported 59.1 MB per thousand operations and a 17.5 % apply-to-prove ratio; both figures measured the single-step path, which is no longer the production one. Those figures have since been corrected in that preprint's own revisions, and the values used here are the corrected ones for a complete two-phase payment.</p>
</div>

**Keywords:** supervisory audit, selective disclosure, zero-knowledge proofs, payment settlement, compliance, privacy, residual trust, CBDC infrastructure

## Abstract

Supervisors need reliable assurance over balances, issuance, and transaction integrity. Institutions need to protect sensitive financial data. Traditional audit practice often resolves this tension by granting broad access to ledgers. That approach is effective, but costly in privacy, operational risk, and cross-border data exposure.

This note presents a settlement architecture in which compliance statements can be proven cryptographically without disclosing the full ledger. A supervised entity can demonstrate that a balance equals a value, exceeds a threshold, or lies within a band. Verifiers check the proof without receiving account-level books. Spending keys remain on the client side and do not travel to the operator to authorize a transfer.

A second confidentiality property is reported that is easy to miss in architectural summaries: in a settlement that updates both accounts in a single transition, the payer must know the recipient's balance in order to construct the proof. **Paying someone therefore reveals what they hold.** The architecture addresses this with a two-phase transfer, at a stated cost in finality latency.

The paper is deliberately non-utopian. It specifies which properties become demonstrable and which residual trust remains — especially in a single-node deployment where the operator may still observe state, sequence transactions, or censor. The institutional claim is modest: zero-knowledge settlement can reduce routine full-ledger disclosure while improving the quality of evidence for specific supervisory questions.

## 1. The supervisory problem

Regulated settlement systems must support at least four assurances:

1. **Solvency / balance integrity** — reported positions are consistent with authorized activity.
2. **Issuance control** — value is created only under authorized rules.
3. **Transfer integrity** — only entitled parties spend, and value is conserved.
4. **Auditability** — supervisors can obtain timely, reliable evidence.

In current practice, assurance often depends on access to internal books, reconciliations, attestations, organizational controls, and after-the-fact investigation.

This creates a structural trade-off:

- more supervisory visibility usually means more data concentration;
- more privacy usually means weaker direct inspection.

For domestic systems this is already sensitive. For cross-border platforms, CBDC interoperability, or outsourced infrastructure, full-ledger exposure becomes a strategic and legal problem.

> **Policy question:** Can supervisors obtain strong assurance on specific claims without receiving the entire book?

## 2. What "provable compliance" means here

Provable compliance means that a regulated participant can produce a cryptographic proof of a statement, and a supervisor can verify that proof **without seeing the underlying confidential data**.

Examples of statements:

- "Account A's balance is exactly 1,000,000."
- "Account A's balance is at least 500,000."
- "Account A's balance is between 900,000 and 1,100,000."
- "This transfer conserved value and was authorized by the spender."
- "This issuance did not exceed the configured cap."

The supervisor's result is binary and checkable: the proof verifies or it does not.

This is different from trusting a PDF report, a dashboard, an operator's extract, or the assertion that an internal control operated as described.

It does not replace legal powers, on-site inspection, or forensic investigation. It changes the **default evidence channel** for recurring, well-formed compliance questions.

## 3. Architecture for supervisory use

### 3.1 Three-role flow

The reference architecture separates responsibilities:

| Role | Function |
|---|---|
| Client / account holder | Holds spending authority; generates proofs locally |
| Settlement layer | Maintains state transitions; verifies proofs; applies accepted operations |
| Supervisor / auditor | Verifies selected disclosure proofs without ledger access |

A transfer works as follows:

1. the client obtains public materials needed to prove a state transition;
2. the client generates the proof locally with the spending key;
3. the layer verifies the proof and applies the transition if valid;
4. for audit, the client generates a selective disclosure proof;
5. the supervisor verifies that disclosure proof independently.

### 3.2 Why key separation matters to policy

In many operational designs, intermediaries must handle or control signing capability to process activity. That creates custody and insider-risk concentration.

In this architecture, **the spending key does not need to travel to the operator** for a transfer proof to be created. The operator may still be operationally powerful, but one sensitive trust path — key custody for ordinary spending — is reduced.

⚠️ **Correction.** That is a property of the design, and until 31 July 2026
the implementation did not enforce it: the layer applied payment-path
transitions **without verifying their proofs**, so a spending key was not
needed to move another account's funds. What the design withheld from the
operator, the implementation did not require of anyone. The defect is fixed
and measured; supervisors reading an earlier version of this note received
the claim without that qualification.

For risk officers, this is a concrete control statement:

> Processing does not require surrender of customer spending keys to the settlement operator.

### 3.3 Selective disclosure modes

Supervisory queries differ in precision. The architecture supports at least:

- **Exact balance disclosure** — high precision, higher sensitivity.
- **Minimum balance disclosure** — useful for collateral/reserve-style checks.
- **Band disclosure** — useful for threshold monitoring with less exposure.

This lets supervision scale by necessity: routine monitoring can use bands or minima; escalation can demand tighter disclosure; full investigative access remains a separate legal channel, not the default technical path.

### 3.4 Two-phase transfer and counterparty confidentiality

A settlement that updates both account leaves in one transition requires whoever builds the proof to know both balances. Confidentiality against third parties holds; **confidentiality against the counterparty does not**.

For supervision this matters because the operator is one declared party whose powers can be scheduled and audited, whereas a counterparty can be anyone. A design that protects against the former and not the latter has an uneven privacy claim.

The architecture splits the transition:

| Phase | What happens | Whose balance is needed |
|---|---|---|
| Send | Value leaves the payer into a pending commitment | Payer only |
| Claim | The recipient makes it theirs | Recipient only |

The commitment binds the recipient's public identity, a payer-chosen random value, and the amount. Neither phase reads the other party's balance, and this is enforced by the interface: there is no parameter through which a counterparty balance could be supplied.

✅ **This is now the only path.** The single-step transfer has been retired from the layer entirely, so the property is not one of two options offered to an integrator — it is the only way a payment can be made.

**Costs, stated plainly.** The payment is not final until claimed. If the recipient never claims, the value is immobilised and a return mechanism is not implemented. The payer chose the random value, so they can recompute the commitment and observe **when** it is claimed — a residual timing signal that the design does not remove.

### 3.5 Standards integration

The reference implementation includes an ISO 20022 bridge. A `pacs.008` credit transfer produces a `pacs.002` status report carrying the proof, and rejections carry standard reason codes from `ExternalStatusReason1Code` rather than proprietary strings.

The two-phase design maps onto the standard's own vocabulary without inventing a contract:

| Phase | Status | Meaning in the standard |
|---|---|---|
| Send accepted | `ACSP` | Accepted, settlement in process |
| Claim applied | `ACSC` | Settlement completed |
| Rejected | `RJCT` | With an ISO reason code |

This matters for adoption: a receiving institution does not need to understand the proof system to route the message, and a rejection reaches the counterparty's existing exception handling.

⚠️ **One piece is missing.** The recipient needs the pending commitment's position, random value, and amount in order to claim, and ISO 20022 has no field for them. The reference implementation returns them alongside the message rather than inside it. **How that side channel is operated is unresolved** and would need to be specified before any pilot.

## 4. Properties that become demonstrable

The following properties can be made subject to cryptographic verification in the settlement layer:

| Property | Demonstrable meaning |
|---|---|
| Value conservation | Transfers do not create money |
| Authorized spending | Only the key holder can produce a valid spend proof |
| No double spend / replay | An accepted operation moves the state root, so a repeated operation presents a stale root and is rejected |
| Issuance constraints | Minting requires authorized policy and respects caps |
| State continuity | Accepted transitions chain from prior valid state roots |
| Selective solvency evidence | Balance claims can be verified without full books |

These are not marketing claims. They are mechanical checks a verifier can run on proofs and public parameters.

⚠️ **A note on the third row, because the mechanism changed.** The single-step path prevented double spending with a public spend marker and a proof that its position was unused. That path has been retired and the marker mechanism removed (§5.1). Double-spend prevention now rests on the chaining of state roots — which is sufficient for a single node that imposes a total order, **and is not sufficient without one**. A supervisor evaluating a distributed successor should treat this row as re-opened.

### 4.1 What a supervisor can read without any proof

Some evidence requires no cryptography at all, because the layer publishes it as plain scalars. A supervisor with read access to the public state sees:

| Public scalar | Supervisory meaning |
|---|---|
| Total supply and cap | Issuance is bounded and the bound is visible |
| Custodian intervention counter and quota | How often privileged parties have acted, against a ceiling |
| Freeze counter | How many accounts have been frozen |
| Recovery counter | How many key recoveries have occurred |
| Governance change counter | How many times the authority set itself changed |
| Value in transit | How much is committed but unclaimed |

These are not balances. They are **the aggregate exercise of privilege**, and they are the answer to a supervisory question that proofs alone do not address: not "was each action valid?" but "how often has authority been used?".

A **chained transition log** records every accepted operation with the state roots before and after, so a supervisor can verify that no operation is missing from the sequence without seeing any operation's contents.

⚠️ **The privilege model is bounded but not time-bounded.** Custodian authority is capped by a usage quota and renewed by rotating the set. Account freezes, by contrast, have no expiry: a freeze lasts until someone lifts it. For supervision this is a governance requirement, not a technical one — but a system that counts interventions and does not expire them places the whole burden on the institution.

## 5. Properties that still depend on trust

Honesty about residual dependence is essential for supervisory credibility. In a single-node deployment, residual trust typically remains around:

| Residual dependence | Practical meaning |
|---|---|
| Operator visibility | Operator may see balances/state if held in readable form |
| Transaction ordering | Operator sequences operations |
| Censorship / availability | Operator can delay or refuse to process |
| Operational resilience | Node failure is system failure unless redundancy exists |
| Legal finality | Cryptographic acceptance is not automatically legal settlement finality |
| Identity / sanctions context | Proofs about balances do not by themselves solve KYC/sanctions case management |
| Emergency governance | Crisis powers remain institutional, not purely cryptographic |
| Capacity | Two quantified availability bounds remain; a third has been removed. See §5.1 |
| Freeze duration | Freezes have no expiry; lifting one is a governance act |
| **Total ordering** | Replay and double-spend prevention now depend on a single total order; a distributed successor would have to restore a marker mechanism or equivalent. See §5.1 |

### 5.1 Quantified capacity limits

Residual trust is usually discussed as *who can do what*. A supervisory schedule should also state **when the system stops working**, because an availability bound reached in production is an operational incident, not a theoretical note.

Three were quantified against the reference implementation. **One has since been removed**, and the sequence is reported rather than deleted, because how a bound disappears is supervisory information in its own right.

#### Double-spend marker collisions — removed

Each spend in the single-step path produced a public marker whose position in a tree was derived from the marker itself. Two distinct payments could therefore land on the same position, and the collision followed the birthday bound rather than the tree's advertised size:

| Payments accumulated | Collision probability |
|---|---|
| 10,000 | 1.2 % |
| 65,536 | 39 % |
| 200,000 | 99 % |

The tree advertised roughly four billion positions. **The practical bound was around sixty-five thousand payments.**

What happened to the affected customer is the part that mattered for supervision: their marker was deterministic, so they could not retry, their payment was permanently blocked, and the system originally reported the event as a **double-spend attempt — an accusation that was false**. Any deployment of a mechanism of this shape must specify what a customer is told when it occurs.

✅ **The bound no longer applies.** The two-phase path of §3.4 does not use these markers; the single-step path has been retired; and the tree that held the markers has now been **removed from the layer with a verified migration** — an existing ledger or archived snapshot is checked against its own stored root before the legacy records are discarded, and a mismatch halts startup rather than passing in silence.

⚠️ **But it was avoided, not solved,** and the distinction is a supervisory one. What replaces the marker is the chaining of state roots, which **requires a total order**: a single node provides one, a distributed system does not. Anyone distributing this architecture recovers the bound intact. The correct remedy — indexing by the full marker rather than a truncation of it, which moves the failure mode to the hash's own collision resistance — remains unimplemented.

#### Authority set size

The ordering rule between two authorising custodians is enforced in a way that **caps the set at 128 members**. Beyond that, authorisations between distant members would fail intermittently — a failure that depends on which two custodians sign, and would surface in production rather than in testing.

#### Evidence accumulation

One thousand complete payments accumulate **120.4 MB** of proofs. This is a storage and bandwidth cost, not a stop.

⚠️ **A payment is two proofs, not one.** Each proof is about 62 KB and that has not changed; what changed is that the two-phase design of §3.4 requires a send and a claim. An earlier version of this note reported 59.1 MB, which was the figure for the single-step path that is no longer the production one.

⚠️ **And the expensive half is the recipient's.** Generating a claim proof costs roughly 500 ms against 283 ms for a send, because the claim circuit walks both the pending tree and the accounts tree. For a merchant settling hundreds of payments a day, the cost falls on **whoever receives** — the opposite of the usual assumption, and a fact worth knowing before pricing anything.

An earlier version of this note treated evidence accumulation as the system's binding limit. **That was wrong:** the collision bound blocked legitimate payments far earlier and permanently, and it did so while the system was nowhere near saturation.

#### Policy formulation

- ZK settlement can strengthen evidence for invariants.
- It does not automatically eliminate operator risk, governance risk, or legal risk.
- Therefore, supervisory adoption should require a **published residual trust schedule**: a short inventory of powers and assumptions that remain outside the proof system — including the bounds at which the system stops, and the bounds that were removed by replacing a mechanism with one that carries a different assumption.

## 6. Institutional comparison

### 6.1 Against conventional ledger audit

| Conventional audit path | Provable disclosure path |
|---|---|
| Extract or access books | Verify proof of a claim |
| Broad data exposure | Minimal disclosure by design |
| Assurance tied to process quality | Assurance tied to mathematical verification of stated properties |
| High cost for repeated checks | Repeatable verification with less data movement |
| Strong for investigation | Strong for recurring control questions |

**Best use:** recurring compliance checks and continuous assurance. **Not** a substitute for investigative powers when misconduct is suspected.

### 6.2 Against permissioned blockchain supervision

Permissioned chains often relocate trust to a consortium and still provide broad observer nodes or privileged audit channels.

A ZK selective-disclosure design changes the default:

- observers need not see transaction detail to accept integrity proofs;
- supervisors can verify specific claims without becoming full data co-owners;
- residual operator/consortium powers are treated as explicit risk items.

The policy advantage is not "no intermediary." It is **less mandatory data replication per unit of assurance**.

## 7. Use cases with near-term relevance

1. **Reserve / holding threshold checks** — prove that holdings remain within regulatory bands.
2. **Intraday solvency-style attestations** — provide frequent assurance with less data transfer.
3. **Cross-border supervisory cooperation** — share verifiable claims where sharing full ledgers is legally or politically hard.
4. **CBDC or regulated token settlement pilots** — support privacy-preserving transfers with explicit compliance evidence channels.
5. **Outsourced processing** — allow technical processing while reducing key custody and full-book exposure to processors.

## 8. Implementation implications for risk and regulation

### What regulators can ask for

- independent verifiability of disclosure proofs;
- documentation of proving assumptions and residual trust;
- separation of client key custody from operator processing;
- operational metrics on failed proofs, emergency actions, and operator interventions;
- recovery and contingency plans for node failure and key loss.

### What institutions must operationalize

- key management and customer recovery journeys;
- performance budgets for proving latency;
- incident response when proofs fail or state integrity checks trip;
- legal mapping between proof verification and existing reporting obligations;
- controls over residual operator privileges.

### What should not be claimed

- "fully trustless compliance,"
- "no supervisory need for data access,"
- "decentralization complete,"
- "legal finality achieved by proof alone,"
- "unlimited throughput" — the bounds of §5.1 are specific and must be published with any deployment claim,
- "privacy from all parties" — the two-phase path of §3.4 closes the counterparty exposure, but the payer retains a timing signal and an unclaimed payment has no return path,
- "double-spend prevention independent of deployment shape" — it now depends on a single total order (§5.1).

⚠️ Earlier versions of this note listed the sixth item **conditionally**: *until the two-phase path of §3.4 is the only path, a payer may still learn a recipient's balance*. That condition has been met — the single-step path is retired — so the caveat is withdrawn and replaced by the residues named above.

## 9. A practical supervisory control model

A conservative adoption path:

1. Pilot on non-critical attestations (band checks, minima).
2. Maintain conventional audit rights for escalation.
3. Require residual trust disclosure as part of system authorization.
4. Measure **false operational confidence** (where teams over-trust proofs and under-manage operator risk).
5. Only then expand to broader settlement functions.

This staged approach treats ZK as an **evidence upgrade**, not as an institutional replacement.

## 10. Conclusion

Supervisory assurance does not require universal visibility. It requires reliable answers to specific questions.

A zero-knowledge settlement architecture can provide those answers for balance constraints, transfer integrity, and issuance rules without full ledger disclosure, while keeping spending keys off the operator path. Its institutional value depends on **equal clarity about residual trust**: who can still see, stop, reorder, or fail the system.

For regulators and risk officers, the relevant evaluation standard is simple:

> Does this system increase the quality of compliance evidence while reducing unnecessary data exposure, and does it expose remaining powers with the same rigor it advertises its proofs?

If the answer is yes, the architecture is policy-relevant even before full decentralization is achieved.

One caution earned in this revision belongs here. Two of the changes above are not new properties but **corrections to a previously published schedule**: a bound that was described as live had been removed, and a caveat that was described as conditional had had its condition met. A residual-trust schedule is only as good as the discipline of re-reading it against what the system now does, and that discipline is itself part of what a supervisor should be assessing.

## Artifacts

Reference implementation and documentation: https://github.com/atoranzo/Arqueo-open-conservation-proofs-for-closed-ledgers

- Architecture, principles, and **375 executable tests** across the two production crates
- A standing audit document that records open defects, their cost, and the methodology errors found while looking for them

The audit document is offered as **part of the artifact** rather than as an appendix to it. For a supervisory reader, a system's record of what it found wrong in itself is evidence of a different kind than its test count.

### Suggested Zenodo framing

- **Type:** Preprint / Technical note
- **Fields:** Finance, payment systems, supervisory technology, applied cryptography
- **One-line pitch:** "A settlement architecture for proving compliance claims without handing over the full ledger — with a published schedule of the trust that remains and the capacity limits at which it stops."
