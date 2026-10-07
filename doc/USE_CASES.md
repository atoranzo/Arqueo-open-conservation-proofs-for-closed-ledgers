# Where Arqueo applies — properties, use cases, and where it does not

This page maps what the engine proves to the situations where that proof is
worth having. It adds no claim that the tree does not already make: every
"measured" row below points to the file that carries it, and every domain not
reviewed in the project's documents is marked as a candidate. Verified against
`main` at commit `da6a768` (§589; before it, `dca287b`); rows 1 and 5, and what
hangs on them, corrected in §696 against the tree at `1a9de32`.

## The shape of the problem

An operator keeps a ledger. The people who depend on it — members, holders,
beneficiaries, counterparties — cannot see it, and the two sides do not trust
each other. Today that conflict is settled by a third party who opens the
ledger: an auditor, a supervisor, a court; once a year; by sample. Arqueo
replaces the opening of the ledger with a proof that the ledger did what its
rules say. It fits wherever the unit of account is **born and dies inside the
ledger**: issued by the operator, moved between accounts, retired by the
operator.

It proves **conservation, not solvency**. The proofs speak of the ledger, not
of the world (`SECURITY.md`, the oracle limit).

## Properties, with their status

| # | property | what a third party learns | status |
|---|---|---|---|
| 1 | Conservation | the supply the head signs (since v5) and, for a rejection by cap, that the head's supply plus the amount exceeds the committed cap (`spec/PAQUETE.md`, 2.6); **not** that supply = balances + in flight | measured **in the node**: in circuit, on every send, claim and burn, and the aggregate in the layer on reopening (`AUDITORIA.md` §379; the roots, §387, §388, §391 and §392); **without the node, no**: the checker verifies no transition proof — nor the issuance and refund ones —, and the log does not keep them |
| 2 | No double use | a label is consumed once in a ledger and published in its signed head; the same label in two ledgers is detected from both | measured (RFC-0006; `doc/KIT.md`) |
| 3 | Unrewritable history, with an extension proof | today's signed head extends yesterday's without removal or reordering | measured (`spec/RPC.md:785-812`, `zkssl_consistencyProof`) |
| 4 | Inclusion with a receipt | an entry is in the ledger, provable without the operator | measured (`spec/RPC.md:568-739`, `zkssl_inclusionReceipt`, `zkssl_ackPath`) |
| 5 | Authorship without the key travelling | of the pledge, that whoever holds the key produced it (`spec/PAQUETE.md`, 2.10); **not** that only the holder of a key moves its account | measured **in the node**: in circuit (`C_PK_CHECK`), on every send, claim and burn; **without the node, only the pledge**. That the key does not travel: `spec/RPC.md` §«Principio que el API preserva» |
| 6 | Cut-off and completeness | nothing stays in flight past its time; every operation the node receives ends applied, rejected with proof or declared, or a named red says it did not | measured (RFC-0010; `spec/PAQUETE.md`, 2.11; the empty box, RFC-0007 E4) |
| 7 | Rejection with cause | a refusal carries the rule that produced it | measured (RFC-0007; `spec/PAQUETE.md`, 2.6) |

Row 6, since RFC-0010 (H5b, §556-§577): every operation the node evaluates on
the holder's direct paths gets a receipt under its signed head, and the
completeness envelope says, with no node, that it was resolved in its window
-applied, or rejected with proof- or names the operator that did not
(«NOT RESOLVED IN THE WINDOW»), or counts it apart when its cause has no
portable proof. Its residue is declared: an operator that issues no receipt
at all (RFC-0010, D-H). The batch and the pledge carry one since §611, and the
completeness envelope resolves them since §613 (RFC-0014). Row 7 proves that the rule was applied over what a signed head
commits; not that the rule is fair, and its binding to one received operation
is the node's word in its error data (RFC-0010, D3).

Rows 1 and 5 are proved, transition by transition, inside each payment: the
send, claim and burn proofs carry that transition's balance and supply
arithmetic and the key check (`C_PK_CHECK`), and the node verifies them before
applying the transition. The log keeps only their digest
(`crates/zk-ssl-verify/src/reverificacion.rs`), no method of the wire protocol
serves them, and the checker compiles none of those circuits, nor the issuance
and refund ones: it verifies five others —band, age, pending claim, payment in
flight and pledge—, and only the pledge proves an authorization. The
aggregate supply = balances + in flight is checked by the node's layer when it
reopens the ledger, over balances in the clear. A third party receives the
supply and the roots, signed; it does not receive the identity between them
(`doc/ecst/ECST.md` §«6.3 No-garantías», which already said so).

⚠️ **Corrected in §696**: until then row 1 said a third party learns
«supply = balances + in flight; nothing created or lost between epochs»,
«measured, in flight and on reopening»; row 5, «only the holder of a key moves
its account; the operator cannot», «measured»; and the assertions below said
«row 1 counts what is in flight», «rows 1 and 2: the conservation arithmetic
and single use hold under the signed head, recomputed by the checker» and
«row 5: only the key holder moves the account; the operator cannot».
§379 and §387–§394 are checks by the node, not by a third party.

### The classical audit assertions, mapped (§589)

For an auditor who thinks in assertions, the table above reads as follows.
The mapping adds no claim: each cell points at the row that carries it, and
what a row does not cover stays uncovered.

| assertion | where it lands | what stays outside |
|---|---|---|
| Existence / occurrence | row 4: an entry is in the committed record, provable without the operator | that the unit or the event exists **outside** the ledger: the oracle limit (`SECURITY.md`); and the total in flight, which the head does not sign (row 1) |
| Accuracy | row 2: single use holds under the signed head, recomputed by the checker; of row 1, the signed supply against its committed cap | valuation: the proofs carry amounts, not worth; and, without the node, the conservation arithmetic (row 1): the node checks it, the checker does not |
| Cut-off | row 6, first half: the "empty box" proof that nothing in flight outlives its age (RFC-0007 E4) | — |
| Completeness | row 6, second half: every operation the node evaluates on the holder's direct paths resolves in its window or a named red says it did not (RFC-0010) | what the node never receipted (D-H), and the batch and pledge paths (D-E): completeness of the receipted, never of the unreceipted |
| Rights / authorization | row 5, in part: the pledge proves, without the node, that the key holder produced it | who is behind a key, and whether one person holds one account; and, without the node, the authorization of a send, claim or burn: the node checks it, the checker does not |

The classification and presentation assertions have no row: the ledger's
categories are the operator's, and no proof here speaks of them.

## Use cases, by the property that resolves them

Domains marked *reviewed* are the six examined in the project's own documents,
where the technique deployed today was found to protect the record or the
data but not to prove conservation. Everything else shares the shape and has
not been measured.

**1. Conservation** — the unit is a liability the operator issues and retires.
Proved in circuit, transition by transition, on every payment, and checked
by the node, which also checks the aggregate on reopening; a third party,
today, sees the signed supply, not that it equals balances plus in flight
(row 1).
- Deposit-return schemes: the deposit is the unit; the fraud is returning more
  than was sold, or twice. *Reviewed.*
- Guarantees of origin and emission allowances: issued, transferred,
  cancelled. *Reviewed.*
- Federated community currencies and energy communities. *Reviewed.*
- Digital library lending. *Reviewed.*
- Safeguarding of client funds: the operator created no balance without an
  issuance event; the safeguarding account becomes a one-number oracle rather
  than a whole ledger to trust. *Reviewed.*
- Vouchers and public aid: issued = spent + live, without seeing beneficiaries.
- Share and fund-unit registers: outstanding = issued − redeemed.
- Loyalty points and gift cards: a real accounting liability, proved without
  opening the customer base.
- Netting: group treasury between subsidiaries; bilateral balances between
  operators; balances between public bodies.

**2. No double use.**
- Tickets and passes; quotas (fishing, water, municipal emissions); software
  licences and API credits.
- Publicly funded programmes, where the characteristic fraud is double funding:
  the same expense certified under two programmes. Each expense is a label
  consumed once inside the certifying body's ledger, and the consumption is
  published in its signed head. Across bodies it requires an agreed expense
  identifier, which is governance, not cryptography (RFC-0006: `H(domain,
  agreed identifier)`, public and precomputable by whoever knows it). Two bodies
  that compute it the same way detect the same label from their two signed
  heads, with both nodes off — detection, not prevention, after the fact
  (`spec/rfc/0006-consumo-publicado.md`, E4; `doc/KIT_EN.md`, steps 3 and 4).
  Not proved: that the invoice is real or the expense eligible (the oracle
  limit). Between countries, not even detection until a shared label exists.

**3. Unrewritable history.**
- Membership rolls, internal electoral censuses, minute books.
- Custody of case files: not the content, but that no entry vanished or moved
  after signing.
- Batch traceability (food, pharma, aerospace parts).

**4. Inclusion with a receipt.**
- Public registries of submissions: the citizen keeps a verifiable receipt and
  no longer depends on the administration acknowledging it.
- Legal deadlines: a notice entered the register before a date, against the
  signed head of that epoch.
- Marketplaces: a seller proves an order or refund was recorded, even after
  the platform closes.

**5. Authorship.**
- Systems where the operator is the suspect: local currencies, time banks,
  community savings. The operator sees everything; spending from an account
  takes a proof made with its key, and the node checks that proof before
  applying it.
  A third party cannot check it without the node today (row 5): the log keeps
  the proof's digest, not the proof. Companion limitation, published: the operator
  *can* fail to include a legitimate operation; since RFC-0010 a received one
  that it neither applies nor rejects in its window leaves a signed trace, and
  only an operator that issues no receipt leaves none (row 6).

**6. Cut-off and completeness** (measured).
- Two-phase settlement with expiry between firms; clearing between operators;
  period close, where the "empty box" proof of what is in flight is the
  cut-off that is reconciled by hand today.

**7. Rejection with cause** (measured).
- Appeals: aid denied, claims refused, admissions to regulated programmes.

## Institutional clearing: four tests

(i) Is the unit born and dies inside the ledger? (ii) Is there a third party
who cannot see it? (iii) Is the question conservation, or counterparty risk?
(iv) Scale: measured throughput is 1.5–1.9 transactions per second, given as a
band because two runs on the same machine differed by 22 % (`AUDITORIA.md`
§123); 2^32 simultaneous payments in flight; one node, one writer, no
distributed consensus (`spec/RPC.md:860-871`).

- Fits: registers of entitlements (agricultural payment rights, irrigation and
  fishing quotas, planting rights); netting between operators; netting between
  public administrations. Thousands of operations a year; real third parties.
- Fits in part: securities registers at low volume (unlisted shares,
  crowdfunding platforms, unit-holder registers): non-dilution and an
  unrewritable history; not pricing, not payment.
- Does not fit: central counterparties. Their job is counterparty risk (iii),
  their volume is orders of magnitude above (iv), and their supervisors
  already have full access (ii).

## Central-bank money: an argument, not a use case

A retail central-bank digital currency is a closed ledger in which the unit is
born by issuance and dies by redemption, so the solvency objection does not
arise. But where the central bank operates a centralised ledger and verifies
all settlements and holdings itself, there is no third party the design
intends to serve with a proof; and an offline bearer token is a different data
model from an account-based two-phase protocol. Arqueo is not a component of
such a system. It is a demonstration of the proofs such a ledger could publish
to third parties — the question raised in the project's deposit on residual
surfaces in retail CBDC incidents (doi:10.5281/zenodo.22077991).

## What none of this claims

- That a third party checks, without the node, the conservation or the
  authorship of a payment: the node verifies those proofs and the log keeps
  only their digest (rows 1 and 5).
- Privacy against the operator: the operator sees everything (`SECURITY.md`).
- That the ledger's units exist outside the ledger.
- That an operation the node never acknowledged would be detected: an
  operator that issues no receipt leaves no trace (RFC-0010, D-H), and the
  batch and the pledge carry none (D-E).
- Prevention across ledgers: two ledgers can accept the same label; a third
  party holding both signed heads sees it afterwards, never before.
- Who is behind a key, or that one person holds one account.
- Any domain beyond the six reviewed as measured.

⚠️ **Corrected in §581**: until then this list also said «censorship leaves no
trace» and «Rows 6–7 as existing», and row 6 read «in part». Row 7 exists
since RFC-0007 and row 6 since RFC-0010; the trace now exists for every
operation the node receives on the holder's direct paths.
