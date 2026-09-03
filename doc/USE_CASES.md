# Where Arqueo applies — properties, use cases, and where it does not

This page maps what the engine proves to the situations where that proof is
worth having. It adds no claim that the tree does not already make: every
"measured" row below points to the file that carries it, and every domain not
reviewed in the project's documents is marked as a candidate. Verified against
`main` at commit `b91896b`.

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
| 1 | Conservation | supply = balances + in flight; nothing created or lost between epochs | measured in flight; on reopening, not yet (front page, "What it does not do") |
| 2 | No double use | a unit is consumed once (nullifiers) | measured |
| 3 | Unrewritable history, with an extension proof | today's signed head extends yesterday's without removal or reordering | measured (`spec/RPC.md:781-808`, `zkssl_consistencyProof`) |
| 4 | Inclusion with a receipt | an entry is in the ledger, provable without the operator | measured (`spec/RPC.md:564-735`, `zkssl_inclusionReceipt`, `zkssl_ackPath`) |
| 5 | Authorship without the key travelling | only the holder of a key moves its account; the operator cannot | measured (`spec/RPC.md:50-59`, the API principle) |
| 6 | Cut-off and completeness | nothing stays in flight past its time; every acknowledgement ends applied or rejected, with a trace | planned (front page, "What comes next") |
| 7 | Rejection with cause | a refusal carries the rule that produced it | planned (front page, "What comes next") |

Rows 6 and 7 do not exist in the tree. They are listed so that a reader knows
which questions the engine intends to answer and does not yet.

## Use cases, by the property that resolves them

Domains marked *reviewed* are the six examined in the project's own documents,
where the technique deployed today was found to protect the record or the
data but not to prove conservation. Everything else shares the shape and has
not been measured.

**1. Conservation** — the unit is a liability the operator issues and retires.
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
  the same expense certified under two programmes. That is a nullifier — each
  expense consumed once. Within one managing body this is direct; across bodies
  it requires an agreed expense identifier, which is governance, not
  cryptography. Not proved: that the invoice is real or the expense eligible.

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
  community savings. The operator sees everything and still cannot move an
  account it does not control. Companion limitation, published: the operator
  *can* fail to include a legitimate operation, and that leaves no trace.

**6. Cut-off and completeness** (planned).
- Two-phase settlement with expiry between firms; clearing between operators;
  period close, where the "empty box" proof of what is in flight is the
  cut-off that is reconciled by hand today.

**7. Rejection with cause** (planned).
- Appeals: aid denied, claims refused, admissions to regulated programmes.

## Institutional clearing: four tests

(i) Is the unit born and dies inside the ledger? (ii) Is there a third party
who cannot see it? (iii) Is the question conservation, or counterparty risk?
(iv) Scale: measured throughput is 1.5–1.9 transactions per second, given as a
band because two runs on the same machine differed by 22 % (`AUDITORIA.md`
§123); 2^32 simultaneous payments in flight; one node, one writer, no
distributed consensus (`spec/RPC.md:856-867`).

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

- Privacy against the operator: the operator sees everything (`SECURITY.md`).
- That the ledger's units exist outside the ledger.
- That an omitted operation would be detected: censorship leaves no trace.
- Who is behind a key, or that one person holds one account.
- Rows 6–7 as existing.
- Any domain beyond the six reviewed as measured.
