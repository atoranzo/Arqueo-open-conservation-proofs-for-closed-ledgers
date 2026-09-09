# Arqueo — 22 Questions

The questions someone asks on meeting this project, answered without ornament. If an answer feels
uncomfortable, it is well written. Every claim in the present tense points to the file or the
[`AUDITORIA.md`](./AUDITORIA.md) entry that carries it; what is verified is verified against
`main` at commit `9c64fd1`. In Spanish: [`PREGUNTAS.md`](./PREGUNTAS.md).

---

## WHAT

### 1. What is this, exactly?

A **closed-ledger engine that publishes open proofs**. An operator keeps a ledger — accounts,
payments, issuances, retirements —; the people who depend on it cannot see it. Arqueo makes the
ledger publish, every epoch, a **signed head** and **evidence packages** with which a third party
checks, without the ledger, offline and without trusting the author, that the ledger did what its
rules say: that money is conserved, that a label was consumed only once, that history was not
rewritten, that an entry is inside, that only the holder moved their account.

Underneath there is a settlement layer in Rust with two-phase payments proved with STARKs, and the
comparative work that grounded its design: the same circuit in five proof systems
([`FIVE_BACKENDS.md`](./FIVE_BACKENDS.md)). It used to be called ZK-SSL; the project's name
changed, its published identifiers did not (`zkssl/0.3`, `zk-ssl-*`).

### 2. What is it NOT?

**Not a chain.** One node, one writer. **Not decentralised**: whoever operates it sees every
balance and can omit an operation without leaving a trace. **Not audited** by anyone external.
**Not in production**, and nobody has used it with real money. **It does not prove solvency**: it
proves that the ledger is consistent with itself, not that its units exist outside it (the oracle
limit, [`SECURITY.md`](./SECURITY.md)). And it is not quantum money: it is the classical
approximation, with a minimal, measured intermediary.

### 3. What does it guarantee?

To a third party who does not see the ledger, today, measured: **conservation** (supply = balances
+ in flight, also on reopening the ledger: `AUDITORIA.md` §379, §387–§394); **single use** of a
label inside a ledger, published in its signed head, and **detection** of the same label in two
ledgers ([`spec/rfc/0006-consumo-publicado.md`](./spec/rfc/0006-consumo-publicado.md));
**unrewritable history** with an extension proof (`zkssl_consistencyProof`); **inclusion with a
receipt** (`zkssl_inclusionReceipt`, `zkssl_ackPath`); **authorship without the key travelling**.
The table, with a source per row, is [`doc/USE_CASES.md`](./doc/USE_CASES.md).

And inside the ledger, in circuit: nobody creates money, nobody spends from someone else's account,
nobody spends twice, a frozen account does not spend, a valid operation is not replayed, and no
operation runs on a corrupt state. Each of those guarantees has a test that tries to break it.

### 4. What does it NOT guarantee?

That an omitted operation is detected: censorship leaves no trace. That the operator does not see
the balances: it does. That two ledgers do not accept the same label: they can; what exists is
detection, afterwards, from the two signed heads. Who is behind a key, or that one person holds one
account. That a payment is final before it is claimed: until the claim it is not, and if nobody
claims, the amount stays locked until the sender refunds it (`AUDITORIA.md` §178–§181). And two
properties the engine **intends** to answer and does not yet: cut-off and completeness, and
rejection with cause ([`doc/USE_CASES.md`](./doc/USE_CASES.md), rows 6 and 7).

### 5. What does it contribute that did not exist?

Two things. First, eight findings absent from the comparative literature because they only appear
when a **complete application** is ported across paradigms, not a reference SHA-256; the main one:
the AIR arithmetisation **lacks copy constraints**, which opens a silent soundness hole when
updating Merkle trees ([`FIVE_BACKENDS.md`](./FIVE_BACKENDS.md)).

Second, a **kit** with which a third party checks on their own machine, offline and without the
repository, a file that adds up, a tampered one that does not — with the broken rule named —, the
same label published in two ledgers, and a swap of ledgers rejected by name
([`doc/KIT_EN.md`](./doc/KIT_EN.md)). It is not a demo: they are real captures from nodes that were
then shut down, and the binary is reproduced from the commit its `VERSION` names.

---

## WHY

### 6. Why STARK and not Groth16, which is faster?

Because Groth16 requires a **trusted ceremony**. If its participants collude and keep the secret,
they can forge proofs — and in a ledger that means **creating money without a trace**. The forged
proofs verify correctly.

The price of avoiding it: proofs of about 62 KB instead of 192 bytes, a factor of 320
([`FIVE_BACKENDS.md`](./FIVE_BACKENDS.md)). It is the only decision in the project taken **against**
the performance numbers.

### 7. Why is there no consensus?

Because it is a distributed-systems problem, not a cryptographic one, and **a badly implemented
consensus is more dangerous than none**: it gives the appearance of a guarantee without giving one.

The road this project does take is that of *Certificate Transparency*: not preventing the operator
from misbehaving, but making sure it **cannot do so in secret**. For that there are the head signer
(`AUDITORIA.md` §236), the signature-index guardian (§234), the heartbeat (§241), the independent
verifier (§243) and the witnesses that pin the key the first time they see it (§245). What is
missing: an anchor prior to the first encounter, and a **verified** key custody, not just a
declared one (§244).

### 8. Why document your own errors?

Because work without documented errors usually means nobody really looked.

On record are the tests that did not discriminate, the comparison that mixed debug and optimised
builds, the constraints left written as empty placeholders, the performance figure attributed to
the node that measured something else, and the sentences in these documents that went stale while
the tree moved on. The rule is one: **a published figure that is corrected is not erased, it is
marked** (`AUDITORIA.md` §247), and the deposits with a DOI have their errata in
[`doc/preprints/ERRATA.md`](./doc/preprints/ERRATA.md).

### 9. Why does the operator still see the balances?

Because it keeps the state: whoever stores the account tree knows its contents. Removing that means
replicating the state among parties that do not trust each other, that is, consensus.

And a correction this document must carry: for a while it said that privacy was "against third
parties who only see proofs". **That was false, and it is measured** (`AUDITORIA.md` §93): the
Merkle path the protocol hands out carried the neighbour's leaf, and a dictionary recovered a
balance. Since §156 the leaf travels wrapped with a salt and that dictionary no longer hits; since
§157 the indices are neither enumerable nor predictable. What remains open is listed in
[`SECURITY.md`](./SECURITY.md). What Arqueo offers a third party **is not privacy: it is proofs**.

### 10. Why can a frozen account still receive?

Because preventing it would leave funds in limbo and break legitimate payments to an account under
investigation. An honest payer does not know the recipient is frozen; rejecting the payment hurts
the payer, not the investigated party. What a frozen account cannot do is **spend**: non-membership
in the frozen tree is proved in circuit, and that tree has a root at rest since §391.

### 11. Why does selective disclosure depend on the holder?

Because the alternative would be a master supervision key, and that key is a target. **There is no
key to steal here** that grants general access to balances. The trade-off is declared: if the
holder refuses to cooperate, there is no mechanism of forced disclosure.

---

## HOW

### 12. How is it proven that no money is created?

With double entry inside the circuit: what leaves one account enters another, and both Merkle
climbs are bound to the same position. Total supply is **public** and only changes through proven
issuances or destructions, each with its proof.

And since §379 the invariant **supply = balances + pending** is checked when the ledger opens, also
on reopening after a restart (§387–§394), with a test that falsifies it: if one byte of the state
at rest creates money, the ledger does not open.

### 13. How is double-spending prevented?

Inside a ledger, by **root chaining**: every proof binds to the exact root it saw when generated,
and the single node provides the total order that makes it hold. It is anti-replay by construction,
and it is also the limit that bites first (question 21).

Besides, since §413 a ledger **publishes what it consumes**: a label `H(domain, agreed identifier)`
enters its consumption tree only once, and the tree's root goes into the signed head; the
consumption envelope proves, with two heads of the same ledger, absence under the old one and
presence under the new one. Across different ledgers there is no order to impose: two ledgers can
accept the same label, and a third party holding both signed heads **sees it, afterwards**
([`spec/rfc/0006-consumo-publicado.md`](./spec/rfc/0006-consumo-publicado.md)).

The old path, which derived a *nullifier*'s position from the nullifier itself, **was retired
along with its tree** (`AUDITORIA.md` §32 and §36): today nothing generates them.

### 14. How does supervision work?

Without opening the ledger, in two ways. The first, from the **holder**: a proof that their balance
lies in a range, with three modes of the same circuit — exact, minimum, band — that the supervisor
verifies with a free function, without access to the ledger. The second, from the **ledger**: the
signed head and the evidence package, which a supervisor checks with the independent verifier,
node off. What is checked there is question 3; what is not, question 4.

### 15. How do you prevent a compromised custodian from issuing alone?

Issuance requires two distinct custodians. The real risk is not an outsider signing — set membership
closes that — but **the same custodian counting twice**, which would turn a 2-of-N into a covert
1-of-N. It is closed with strictly increasing indices **bound to the Merkle paths** through an
accumulator; without that second part the index would be a declared number unrelated to the proven
position.

### 16. How do I know the operator has not rewritten history?

Through the **chained transition log**: every operation leaves an entry whose digest includes the
previous one, and publishing the head commits the whole history. But a chained log only exposes
rewrites to someone who already saw an earlier head. That is why the node serves the head
**signed** (`zkssl_signedEpochHead`), the signature belongs to the hash-based family (XMSS, §236),
an independent **witness** verifies it, **co-signs** it and pins the key it sees the first time
(§245), and anyone can ask for the proof that today's head **extends** yesterday's
(`zkssl_consistencyProof`).

**The guarantee belongs to whoever watches, not to whoever reads**: with no witness running, the
sentence above protects nobody.

### 17. How do I check a file without the node, offline and without trusting the author?

With the kit ([`doc/KIT_EN.md`](./doc/KIT_EN.md)): one download, the release
`arqueo-verify-v0.2.0`, whose fingerprint is published next to its commit. The verifier is a
one-line CLI: `./zk-ssl-verify <file.json>` exits 0 and prints `VERDE: …` if the file holds, 1 and
the **first** failure by name (`ROJO: …`) if not. Four steps: a file that adds up; a tampered one
that does not, with the broken rule named; the same label in two ledgers, detected from the two
heads with both nodes off; and a swap of ledgers, rejected by name. The whole catalogues are checked
with the harness that travels inside the tarball, and the binary is reproduced from the commit its
`VERSION` names.

---

## WHO

### 18. Who can create money?

Two distinct custodians from a set committed in a public root, and only up to an **immutable cap**
of the ledger. Not even the full set can exceed that cap without creating a new ledger, which would
leave an impossible-to-hide trace. And what they issue is counted: supply is public and the
invariant of question 12 watches it.

### 19. Who controls the custodians?

A separate **governance set**, which can change the custodian set; every change is counted in the
record (`governance_change_count`, §393). The circularity does not disappear — whoever controls
governance controls everything — but it moves to keys used almost never, which can be kept offline,
instead of operational keys exposed daily. If governance is compromised, the way out is a new
ledger: it is the conscious end of the chain of authority.

### 20. Who is this useful for?

For whoever keeps a closed ledger on which third parties depend without being able to see it, when
the unit is **born and dies inside the ledger**: deposit-return schemes, guarantees of origin and
emission allowances, community currencies, safeguarding of client funds, public aid where the
fraud is double funding, registers of entitlements and quotas, netting between operators or between
public bodies. The cases, with the property that resolves each and which are reviewed, are in
[`doc/USE_CASES.md`](./doc/USE_CASES.md). It does not serve a central counterparty — its problem is
counterparty risk, not conservation — and it is not a component of a central-bank digital currency.

Honestly: today nobody uses it with real money. What exists is a measured engine, a format with
vectors and a kit that invites you to check it. And the comparative work still serves anyone who
wants data on how zero-knowledge systems choose a paradigm.

---

## HOW MUCH

### 21. How much does it cost, how large is it, how far does it scale?

All measured on one machine, in release; times are given as ranges because two runs of the same
binary differ by ~9 % (`AUDITORIA.md` §131), and **they are not comparable with measurements from
another session**.

- **Verifying versus generating**: verifying an audit proof costs 0.58 % of generating it (§22);
  that asymmetry is what makes the model viable. Applying a transfer is not comparable: it
  verifies, mutates the tree and writes to disk.
- **Node ceiling per RPC**: 248 operations per second (§229). The full cycle of a payment on a
  laptop — generating both parties' proofs and applying them — comes to 1.5-1.9 payments per
  second, and for a while that figure was attributed to the node: **that was false, and by a lot**
  (§229, §238); the node works 4 % of that cycle.
- **Size**: a thousand transfers are ~590 s of proving and 126.2 MiB accumulated (§130). Solving
  that requires recursive aggregation or batched proofs, which are not implemented.
- **The limit that bites first**: root-anchoring contention. Every proof binds to the exact root it
  saw, so two concurrent issuers serialise; with four clients at once, one applies and the other
  three are rejected (§123, §230). With a single issuer per root the waste is zero.
- **Two more**: the pending tree runs out at 2³² simultaneous payments in flight, and the custodian
  set caps at 128 (§13). The *nullifier* collision limit of the old path **was not solved, it was
  avoided**: the path was retired (§32, §36), and the chaining that replaces it requires a total
  order that a single node provides and a distributed system does not.

### 22. How far is it from being usable?

For a real third party to rely on these proofs: an **external audit**, which does not depend on
more code; a **verified key custody**, not just a declared one (§244); and an **anchor prior to the
first encounter** between witness and node. For the engine to answer everything it intends to:
cut-off and completeness, and rejection with cause, which exist as planned rows and not as code.
Distributed consensus is another discipline and not this project's road: the road is provable
accountability, and its pieces are built (question 7).

What is already there: the format as a public contract with vectors that are never rewritten, the
reproducible verifier, and a kit with which anyone can check the above without believing this page.

---

## Next

| | |
|---|---|
| Start | [`README_EN.md`](./README_EN.md) |
| Check it without trusting anyone | [`doc/KIT_EN.md`](./doc/KIT_EN.md) |
| Where it fits and where it does not | [`doc/USE_CASES.md`](./doc/USE_CASES.md) |
| What is still open | [`SECURITY.md`](./SECURITY.md) |
| Break it | [`AUDITORIA.md`](./AUDITORIA.md) |
| The comparison | [`FIVE_BACKENDS.md`](./FIVE_BACKENDS.md) |
| The paper | [`PAPER_EN.md`](./PAPER_EN.md) |
