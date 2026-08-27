# From Institutional Trust to Verifiable Properties

<p class="sub">A Minimal ZK Settlement Layer and Its Residual Trust Surface</p>

<div class="meta">
<p><strong>Author:</strong> Angel Jose Toranzo Portela</p>
<p><strong>Repository:</strong> https://github.com/atoranzo/ZK-SSL-ZK-Sovereign-Settlement-Layer-</p>
<p><strong>License:</strong> MIT OR Apache-2.0 &nbsp;·&nbsp; <strong>Status:</strong> Technical preprint / systems note &nbsp;·&nbsp; <strong>Affiliation:</strong> Independent</p>
<p><strong>Version:</strong> fourth revision, August 2026. The second revision added §4.4 and §4.5, reporting residual dependencies the first version did not name. The third reported the removal of the nullifier tree behind the collision bound of §4.5 and corrected two figures. This revision adds §4.7 — two conditions in the residual table that an audit found unenforced, fixed and measured, and a confidentiality residual toward an account's tree neighbour, published as open and since closed; adds §4.8, reporting that the evidence machinery a residual-trust schedule requires — signed epoch heads, per-entry acknowledgments under a signed retention promise, closed-epoch inclusion proofs, an independently recomposable head — has moved from proposed to built and measured, along with the four residuals it introduces; adds to the table in §4.1 the operator power an errata found missing from it; and removes the draft metadata appendix.</p>
<p><strong>No institutional affiliation.</strong> This is independent work. It is not affiliated with, endorsed by, or commissioned by the European Central Bank, the Eurosystem, or any other institution, public or private. No claim in this document should be read as anyone's position but its author's.</p>
<p><strong>Companion preprints:</strong> <em>Comparative Implementation of a Zero-Knowledge Settlement Layer across Five Proof Systems</em>, 10.5281/zenodo.21736125; <em>Provable Compliance without Full Ledger Disclosure</em>, 10.5281/zenodo.21736082.</p>
</div>

**Keywords:** financial settlement, zero-knowledge proofs, institutional trust, residual trust, selective disclosure, payment infrastructure, permissioned systems

## Abstract

Financial settlement systems rely heavily on institutional trust: intermediaries maintain ledgers, certify compliance, and prevent unauthorized creation or movement of value. Zero-knowledge (ZK) techniques make it possible to replace part of that trust with verifiable properties. This paper presents a minimal ZK settlement layer designed around a simple principle: prove what must be true, disclose only what must be seen, and declare remaining trust explicitly.

We describe an architecture in which transfers preserve value, spending authority is proven without sharing spending keys with the operator, replay and double-spending are prevented, and supervisors can verify balance bands or thresholds without receiving the full ledger. We also map the residual trust surface: the operator of a single node can still see balances, order transactions, and censor. The contribution is not a claim of full sovereignty or decentralization. It is a precise shift from opaque institutional faith toward a smaller, named set of trust assumptions, with cryptographic checks covering the rest.

We compare this model conceptually with core banking systems and permissioned blockchains, and argue that the main institutional value of ZK settlement is not "trustlessness," but **trust minimization with honest residual boundaries**.

This revision continues to hold the claim to its own standard, now in both directions. An audit pass found two conditions in the residual table that had never been verified and did not hold — three operations were not checking their proofs at all — and a confidentiality residual toward an account's *tree neighbour*; §4.7 reports both, the first fixed and measured, the second published as open and since closed. In the opposite direction, §4.8 reports the machinery that turns lying and censoring from invisible into evidencable — signed epoch heads, per-entry acknowledgments, a retention promise that travels signed, inclusion proofs for closed epochs — now built and measured, together with the four residuals that machinery introduces about itself. A residual-trust schedule behaves like a ledger: entries close, and every closure opens its own.

## 1. Introduction

Modern payment and settlement infrastructures are built on intermediaries. A bank, clearing house, or market infrastructure keeps the authoritative state, applies rules, and reports to supervisors. Users and regulators trust that:

- balances are correct,
- money is not created outside authorized channels,
- only entitled parties can spend,
- historical records are not silently rewritten,
- supervisory access is available when required.

That model works, but it concentrates observational and operational power. Privacy is limited, audit often requires broad data access, and integrity depends on institutional controls that are difficult for outsiders to verify directly.

Cryptographic proof systems change the available design space. A participant can prove a statement about secret data without revealing the data. In settlement terms, this allows questions such as:

- Was value conserved in this transfer?
- Did the spender have authority?
- Was this operation already applied?
- Is this balance within a required band?

without exposing full account histories to every verifier.

The practical risk is overclaiming. A system can be zero-knowledge in its proofs and still depend on a powerful operator. If that residual dependence is hidden, ZK becomes a new form of opacity. This paper therefore focuses on two tasks:

1. identifying which settlement properties can be made verifiable;
2. mapping the residual trust surface that remains when those properties are implemented in a minimal layer.

## 2. Design objective

The objective is a minimal settlement layer with the following properties:

1. Verifiable conservation of value in transfers.
2. Authorized issuance under explicit rules and caps.
3. Spending authority proven by the account holder.
4. Double-spend and replay prevention.
5. Selective disclosure for supervision.
6. No trusted setup ceremony whose compromise could forge value undetectably.
7. Explicit residual trust: any remaining intermediary power is named, not marketed away.

The system is intentionally narrow. It is not a full bank, not a complete CBDC stack, and not a decentralized consensus network. It is an instrument for testing how far verification can replace institutional faith in the settlement core.

## 3. Architecture as a trust-reduction device

### 3.1 Separation of roles

The layer separates three acts:

- **View:** public or semi-public state materials needed to prove a transition.
- **Prove:** performed by the client holding the spending key.
- **Apply:** performed by the layer after verifying a proof against current state.

This separation matters institutionally. In many systems, moving value requires handing authority to the operator. Here, the spending key need not leave the client. The operator can accept or reject a valid proof, but does not need custody of the key to process a transfer.

### 3.2 Verifiable monetary cycle

A minimal monetary cycle includes:

| Operation | Authority | Effect on supply |
|---|---|---|
| Issue / mint | authorized issuers | increases |
| Transfer | account holder | unchanged |
| Burn / destroy | account holder | decreases |
| Audit disclosure | account holder | none |

The important invariant is simple: transfers move value; they do not create it. Creation and destruction are separate, authorized events and can be constrained by public caps and counters.

### 3.3 Selective disclosure

Supervision often needs assurance, not total visibility. The layer supports disclosure modes such as:

- exact balance,
- minimum balance,
- balance band (between X and Y).

A supervisor can verify the proof without receiving the full ledger. This is the institutional core of the design: compliance evidence without routine full-book exposure.

### 3.4 Backend choice as policy

Proof systems differ in performance and trust assumptions. Some require a trusted setup ceremony. If ceremony participants collude and retain trapdoor information, they may be able to forge proofs. In a monetary system, forged proofs can mean undetectable value creation.

For a settlement layer that claims reduced dependence on unaccountable trust, ceremony dependence is not only a cryptographic detail. It is a **governance failure mode**. The reference design therefore prefers a transparent proving path without per-circuit trusted setup, accepting larger proofs in exchange for eliminating that residual ceremony risk.

## 4. The residual trust surface

ZK does not delete intermediaries by default. In a single-node deployment, residual trust remains material.

### 4.1 What the operator can still do

| Residual power | Status in minimal layer |
|---|---|
| Observe balances and state | Yes, if the node maintains plaintext or decryptable state |
| Order transactions | Yes |
| Censor or delay transactions | Yes — but no longer invisibly: every accepted entry receives an acknowledgment leaf under a per-epoch root that travels **signed** in the head, together with the retention promise N; a client can prove inclusion for any closed epoch against a head it already custodies (§4.8) |
| Become a single point of failure | Yes |
| Silently rewrite history | Constrained by chained transition records / integrity checks — and the epoch head is now signed and independently recomposable from its fields (§4.8) |
| Replace the verifier itself | **Unbounded and untraceable today.** Ordering, censoring and observing act *within* the rules; replacing the verifier redefines the rules under which everything else is judged, and the system has no notion of "rules in force", so the change leaves no trace. Designed close: a hash of the active verifier inside the attested epoch head — which first requires giving the system that notion |
| Create value outside rules | Constrained by proof verification and public supply constraints — ⚠️ **the constraint failed once; see §4.7** |
| Spend from an account without key | Constrained if spending proofs require client-side keys — ⚠️ **the condition did not hold; see §4.7** |
| Learn a counterparty's balance | **Constrained now:** the single-step path that exposed it has been retired and the split transition is the only path; see §4.4 — and the *neighbour* reading of §4.7 has since been closed as well |
| Keep an account frozen indefinitely | Not bounded: freezes are counted but never expire |
| Refuse service once a capacity bound is reached | Partly bounded; see §4.5 — one of the three bounds has since been removed |


### 4.2 Why naming residual trust is the contribution

Many systems advertise "trustless" or "sovereign" properties while retaining operational chokepoints. The approach here is the opposite:

- verify what can be verified;
- bound what can be bounded;
- publish what remains trusted.

This produces a smaller trust surface and a clearer accountability map. For financial infrastructures, that clarity is more useful than an absolute claim.

### 4.3 What residual trust implies for institutions

Supervisors, banks, and market infrastructures should read ZK settlement as:

- stronger evidence for specific invariants,
- weaker justification for indiscriminate data harvesting,
- continued need for operational resilience, governance, and legal accountability around residual powers.

In other words, cryptography can shrink the domain of faith. It does not remove the need for institutional design around the domain that remains.

### 4.4 A residual dependency that is not the operator's

The table in §4.1 asks what the operator can do. That framing has a blind spot, and an audit pass found it.

In a settlement that updates both account leaves in a single transition, whoever constructs the proof must know both balances. Confidentiality holds against third parties and **fails against the counterparty**: paying someone reveals what they hold.

This is not an implementation defect. It is a property of the statement being proven, and it appears in every proof system, because a proof about two accounts requires knowledge of two accounts.

It matters more than its size suggests, for a reason this paper's own thesis supplies: the operator is one declared party whose powers can be scheduled, counted and audited. A counterparty can be anyone. **A design that bounds the first and ignores the second has named the wrong residual.**

The mitigation is to split the transition: value leaves the payer into a commitment bound to the recipient's public identity, and the recipient later makes it theirs. Neither step reads the other party's balance.

✅ **The single-step path has since been retired entirely**, so the split transition is not an alternative offered alongside it — it is the only way to pay. The residual named in §4.1 is therefore closed, and the limitation in §7 that made it conditional no longer applies.

The residue that remains, stated rather than resolved: the payment is not final until claimed; if the recipient never claims, the value is immobilised and no return path exists; and the payer, having chosen the commitment's random value, can observe **when** it is claimed — not the balance, but a timing signal.

### 4.5 Bounds are residual trust too

A residual-trust schedule usually answers *who can do what*. It should also answer *when the system stops*, because a bound reached in production is indistinguishable, to the customer, from a refusal.

Three were quantified. **One of the three has since been removed**, and the sequence is left visible below because how a bound disappears is itself residual-trust information.

**Marker collisions — removed.** A double-spend marker's position was derived from the marker itself, so two distinct payments could collide, and the collision followed the birthday bound rather than the tree's advertised size. The tree advertised roughly four billion positions; the practical bound was around **sixty-five thousand payments**. The affected customer could not retry — the marker is deterministic — so their payment was permanently blocked, and the system originally reported it as a double-spend attempt, which was false.

✅ The split transition of §4.4 does not use these markers. With the single-step path retired, nothing generated markers any longer, and the tree that held them has now been **removed from the layer with a verified migration**: an existing ledger or archived snapshot is checked against its own stored root before the legacy data is discarded, and a mismatch halts startup rather than passing silently. The bound cannot be reached because the mechanism no longer exists.

⚠️ **But it was avoided, not solved.** What replaces the marker is the chaining of state roots, and that **requires a total order** — which a single node provides and a distributed system does not. Anyone distributing this system recovers the bound intact, and the correct remedy remains unimplemented: index by the full marker rather than a truncation of it, which moves the failure mode to the hash's own collision resistance. Widening the truncation moves the bound without changing its nature.

**The authorising set is capped at 128 members** by the way ordering between two custodians is enforced. Beyond that, authorisations between distant members fail intermittently, depending on which two sign.

**Evidence accumulates at 120.4 MiB per thousand complete payments.** This is a cost, not a stop, and an earlier framing of this work treated it as the binding limit. It was not: the collision bound blocked legitimate payments far earlier, permanently, and while the system was nowhere near saturation.

⚠️ That figure has now been corrected three times, and every correction belongs in a paper about naming residual trust. **The third is the unit**: the number was always binary and was reported as "MB"; in SI units it is 129.0 MB. The first version treated accumulation as the binding limit, which it is not. The second reported 59.1 MB, which measured the single-step path after that path had stopped being the production one — a payment through the split transition of §4.4 is **two proofs, not one**.

⚠️ **And the expensive half falls on the recipient.** Generating a claim proof costs roughly 500 ms against 283 ms for a send. For a residual-trust schedule this is not a footnote: the party with the least choice about participating carries the larger cost.

A bound that is not published is a residual dependency on the operator's willingness to explain the failure.

### 4.6 What the audit pass says about the method

None of §4.4 or §4.5 was found by the tooling built for the audit — a vacuous-constraint detector and an unfilled-column checker.

⚠️ **A correction to how that tooling's result was reported.** Earlier versions of this paper stated that between them the tools found no defect across **twelve production circuits**. The coverage was **eleven**: the report for the twelfth was generated against an invalid reference trace, and the self-check that would have caught it is a debug-mode assertion that never executed, because all documentation specified release mode. Of those twelve, two belonged to the retired path, so "production" meant **ten**. Across the eleven actually covered, neither tool found a defect.

That correction is not incidental to this paper's thesis. **A verification tool whose self-check no one has run does not say what it appears to say** — and reporting its clean result as coverage was itself an unnamed dependency on the author's assumption.

All of §4.4 and §4.5 came from a different question: *what does this check defend, and what happens if I attempt the thing it is supposed to prevent?* Applied to tests inherited from a superseded implementation path, that question found a regulatory limit no longer enforced in-circuit and a set of operations leaving no audit trail. Applied to restart tests, it found a privilege quota that a node restart silently renewed.

We report this because it bears on the paper's thesis. Naming residual trust is not a one-time act of documentation. It is a recurring adversarial exercise against one's own claims, and its yield came from asking what was defended — not from building instruments to detect what was not.

### 4.7 Three residuals this paper had not named

§4.4 states that a design which bounds the operator and ignores the
counterparty **has named the wrong residual**. An audit pass on 31 July 2026
found three more, and the same sentence applies to them.

**Two conditions in §4.1 that did not hold.** The table said value creation
was *constrained by proof verification* and that spending without a key was
*constrained if spending proofs require client-side keys*. Until 31 July
2026 the layer's `apply_send`, `apply_claim` and `apply_mint_to_pending`
**did not verify their proofs at all**, so neither constraint operated: any
party could empty any account, and value could be minted outside the supply
cap. Both are fixed and measured.

⚠️ **The table's own wording anticipated this and no one checked it.** A row
reading *«constrained **if** X»* states a condition; verifying that X holds
is a separate act, and it was never performed.

**And a residual that is still open.** §4.4 closed the leak toward the
*payer* — paying someone no longer reveals what they hold. It did not close
the leak toward a **neighbour who has not paid anything**:

The layer hands each client the Merkle path for its own account. The
sibling at level 0 **is the neighbouring account's leaf**, and the leaf is
`H(H(identity, balance), nonce)` with **no salt**. With the target's public
identity — which travels as the payment address — the balance is recovered
by dictionary. **Measured: 10.84 s** for a retail balance.

The cost is not a number but a curve over the balance range an attacker
assumes: **2.4 minutes** for 0–10 000 €, 4.1 hours for 0–1 M, and 8.3 × 10⁷
core-years if balances were uniform over 64 bits — **which they never are in
a money system**. Scope is bounded: **one account** per path, since only the
level-0 sibling is a leaf preimage. And account indices are sequential, so
**the neighbour is chosen, not drawn**.

⚠️ **This one does not end in a checkmark.** Deriving a salt from the
spending key is ruled out: `open_account`, `mint`, `freeze` and `recover`
all write a holder's leaf **without knowing that holder's secret**. No
solution is currently known, and stating that is the point of this section.

✅ **Closed since this section was first written** (August 2026). The asymmetry that ruled the derivation out has been resolved: the leaf salt is now **derived from the spending key**, a certified test shows that a holder who keeps nothing but the key re-derives identity and salt and reconstructs their exact leaf, and the migration queue for pre-salt leaves has been emptied. The dictionary attack above no longer has an unsalted preimage to walk. The paragraph declaring that no solution was known is left standing, because the sequence — named while open, closed with the reasoning visible — is this paper's method applied to its hardest entry.

### 4.8 From proposed to built: the evidence layer, measured

Earlier revisions of this paper could only say that the operator orders and censors, and that a chained log constrains rewriting for whoever already holds an earlier head. Since the third revision, the layer has built the machinery that a residual-trust schedule actually requires — the machinery that makes lying and censoring produce evidence, in the Certificate Transparency tradition of holding a single operator accountable rather than replacing it.

**What is built and measured:**

- **The epoch head is signed.** XMSS^MT (SHA-2, 40/8): 160.5 ms to sign — re-measured at 144.5 ms — 2.7 ms to verify, 18,469-byte signatures. The scheme is stateful and post-quantum, and index reuse leaks the key, so a mandatory index guardian sits in front of every signature.
- **Every accepted entry receives an acknowledgment leaf.** The per-epoch Merkle root of those leaves and the retention promise **N travel signed inside the head**. N = 1,440 signed heads — roughly 24 hours, the same order as Certificate Transparency's maximum merge delay. The promise is no longer policy prose: it is a signed field.
- **Closed epochs serve inclusion proofs.** A dedicated RPC method returns the acknowledgment path for any closed epoch. The head does not travel with it: the client verifies against the head it already custodies, and the independent verifier **recomposes** the head from its seven fields rather than trusting a served digest. The head format is versioned — a single version byte governs recomposition, and version-1 heads keep verifying under the version-2 binary — so the format was broken once, for every field that needed to enter the signature.
- **The node keeps an append-only journal** of every head it emits, and a witness client transcribes and compares against it.

**The priced residual.** Signing every acknowledgment individually would cost one XMSS signature per entry; measured, that collapses throughput from roughly 320 to roughly 6 operations per second — a factor of fifty. The acknowledgment therefore inherits the epoch signature, and the price is a window of at most one heartbeat during which the evidence exists only if the operator cooperates — which is precisely the case that does not matter — and the window is declared where the promise is made.

**Four residuals this machinery names about itself.** A schedule that taxes everyone must tax its own additions:

1. Epoch boundaries are read from the node's **own** journal. Self-reported, declared as such, not yet externally verifiable.
2. The witness compares in **one direction only** (witness against node) and by sampling. The grave case — a served signature the node does not remember emitting — is reachable and not yet exercised.
3. Five delegated operations — mint, freeze, recovery, governance, mint-to-pending — still append an **empty proof** to the transition log. Their *authorization* is cryptographically bound to the exact transition; the **record** does not carry the authorizing proof's digest. The account-holder paths do. As it stands, the registry proves more about the holder's operations than about the operator's.
4. There is no re-verification pass for a replicated log: a party that receives the log still trusts the writer's digests.

This section is the paper's thesis executed at the smallest unit it permits: each property converted, and each conversion's own residue named in the same act.

## 5. Comparison with existing models

### 5.1 Core banking / sovereign payment systems

| Dimension | Core institutional model | Minimal ZK settlement layer |
|---|---|---|
| Source of assurance | Process, audit, regulation | Proofs for selected invariants + residual operational trust |
| Privacy | Strong outward secrecy; high inward visibility | Privacy toward third parties; operator visibility must be explicitly managed |
| Compliance access | Often broad ledger access | Selective proofs without full ledger disclosure |
| Unauthorized value creation | Controlled by institutional process | Constrained by circuit rules and verification |
| History integrity | Organizational controls | Cryptographic chaining helps detect silent rewrite |
| Failure modes | Governance, fraud, operational risk | Same residual operational risks if node is centralized |

**Contribution relative to core banking:** better evidence and narrower disclosure for specific controls; not a replacement for monetary authority, legal finality, or crisis governance.

### 5.2 Permissioned blockchains

| Dimension | Permissioned chain | Minimal ZK settlement layer |
|---|---|---|
| Trust relocation | From one bank to a validator consortium | From institutional faith toward verifiable properties |
| Privacy among participants | Often limited | Stronger transaction privacy via ZK |
| Compliance | Frequently privileged observation | Supervisory verification via selective disclosure |
| Transparency about power | Variable | Residual operator powers are first-class design content |
| Setup assumptions | Depends on stack | Explicit rejection of ceremony-dependent value security |

**Contribution relative to permissioned chains:** less "new intermediary set," more "smaller claim set with proofs." The differentiator is not permissions, but the combination of private verification and honest residual-trust accounting.

## 6. Economic and infrastructural implications

### 6.1 Possible benefits

- Lower routine data exposure in supervisory workflows.
- Clearer separation between customer custody and operator processing.
- Stronger technical assurance on conservation and authorization invariants.
- A path for privacy-preserving compliance in regulated payment environments.

### 6.2 Costs and constraints

- Proving cost and operational complexity.
- Key management risk for users or institutions.
- Integration burden with legacy payment messaging and legal finality regimes.
- Residual centralization risk until consensus and operational distribution are solved.

### 6.3 Institutional reading

The near-term value is not full disintermediation. It is **better evidence per unit of disclosure**, and a design culture that refuses to hide remaining intermediaries behind cryptographic branding.

## 7. Limitations

This work does not claim:

- decentralized consensus,
- third-party production audit,
- end-to-end replacement of RTGS/core banking,
- disappearance of operator power in single-node deployments,
- benchmark completeness beyond controlled local measurements,
- unbounded capacity: the limits of §4.5 are specific and must accompany any deployment claim — including the one that was removed, because its replacement depends on a total order that a distributed deployment would not have,
- freedom from the residues of the split transition: the payment is not final until claimed, an unclaimed payment is immobilised with no return path, and the payer retains a timing signal,
- unwitnessed evidence inside the open epoch: the unsigned window of §4.8 is at most one heartbeat and exists only under an honest operator,
- record-binding for delegated operations: they are authorization-bound but not yet record-bound (§4.8),
- re-verification of a replicated transition log (§4.8).

⚠️ Earlier versions listed here a **conditional** limitation on confidentiality — *until the split transition is the only path, a payer may still learn a recipient's balance*. The condition has been met: the single-step path is retired and the split transition is the only path. The limitation is withdrawn, and the residues that remain in its place are the three named above.

These limitations are part of the trust map. Omitting them would recreate the institutional opacity the design seeks to reduce.

## 8. Conclusion

A minimal ZK settlement layer can move financial infrastructure from broad institutional faith toward specific verifiable properties: conservation of value, spending authority, double-spend resistance, and selective supervisory disclosure. That shift is meaningful only if residual trust is treated as design material rather than marketing residue.

The central claim of this paper is therefore modest:

> Zero-knowledge settlement is valuable when it **reduces the amount of trust required** and **makes the remaining trust visible**.

For banks, supervisors, and payment architects, the useful question is not whether a system is "trustless." The useful question is:

> Which properties are proven, which powers remain, and can both be inspected without ceremony or myth?

The reference implementation associated with this paper is offered as a concrete artifact for that inspection.

**Three corrections belong in the conclusion rather than in a footnote.**

The first version of this paper argued that naming residual trust is the contribution, and then failed to name a confidentiality leak toward counterparties, three capacity bounds, and a privilege that never expires. They are named in §4.4 and §4.5 now.

The second is smaller and more embarrassing: the cost figure this paper cited measured an operation the system had already stopped using. **A number that describes nothing executable is a residual dependency on the reader's trust in the author** — which is precisely the kind of dependency this paper claims to be about.

The third is of the same family and was found while preparing this revision. Figures published across this project's own documents **did not agree with each other**: the count of executable tests, the number of circuits covered by the mutation tool, and the number of tests that fail outside release mode each appeared with more than one value across the repository and these preprints. A residual-trust schedule that cannot keep its own arithmetic consistent is asking the reader for exactly the trust it claims to remove. **All three have now been re-measured against the implementation** — 375 executable tests, eleven circuits covered of twelve, and 65 debug-mode failures of 174 — and the repository has been corrected where it disagreed. (Re-measured once more for this revision, because the figures have kept moving in the direction this paper argues for: the 16-member workspace now declares **926 tests, 775 of them in the sealing gate** — and the growth since the previous count is, in large part, the evidence layer of §4.8.)

We leave the sequence visible because it is the paper's own argument applied to itself: **a residual-trust schedule is not a document you write once. It is a claim that has to be attacked periodically, including by its author.**

## References / artifacts

- Source repository: https://github.com/atoranzo/ZK-SSL-ZK-Sovereign-Settlement-Layer-
- Architecture and principles documents in the repository
- Comparative backend measurements; **926 declared tests across the 16-member workspace, 775 of them in the sealing gate**
- Versioned conformance vectors: the current vector is regenerated by tooling, byte-identical regeneration is a permanent gate, and the previous format version is rejected on purpose
- A machine-generated OpenRPC document produced from the wire table, with byte-exact regeneration enforced
- A standing audit document (`AUDITORIA.md`, more than 20,000 lines) recording open defects, their cost, and the methodology errors found while looking for them

That last artifact is deliberate. For a paper arguing that residual trust should be design material rather than marketing residue, **a public record of what the system found wrong in itself is the claim's only real evidence.**
