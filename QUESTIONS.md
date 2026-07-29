# ZK-SSL — 22 Questions

The questions someone asks when they come across this project, answered
without decoration. If an answer feels uncomfortable, it is well written.

---

## WHAT

### 1. What is this, exactly?

Two things.

A **financial settlement layer** where transfers are private and
regulatory compliance is cryptographically provable, built without any
trusted setup ceremony.

And the **comparative work** that informed its design: the same circuit
implemented across five zero-knowledge proof systems and measured under
identical conditions.

Of the two, the second is what contributes something that did not exist.

### 2. What is it NOT?

**Not a blockchain.** It is a single node.

**Not decentralized.** Whoever operates it sees every balance and can
censor operations.

**Not audited.** No external party has reviewed it.

**Not in production**, and no one has used it with real money.

### 3. What does it guarantee?

Without revealing identities, balances, or amounts: that no one can create
money, spend from someone else's account, double-spend, spend while
frozen, replay a valid operation, or operate on corrupted state.

Each of those guarantees has a test that tries to break it.

### 4. What does it NOT guarantee?

That the state the operator shows is the real one, and that the recorded
operations are all the ones that happened.

**State transitions are proven. State itself, and history completeness,
are not.** That requires distributed consensus.

### 5. What does it contribute that did not exist?

Eight findings absent from the comparative literature, because they only
surface when porting a **complete application** between paradigms, not a
reference SHA-256.

The principal one: AIR arithmetization **lacks copy constraints**, which
opens a silent soundness gap when updating Merkle trees. Invisible to
honest witnesses.

---

## WHY

### 6. Why STARK and not Groth16, which is faster?

Because Groth16 requires a **trusted setup ceremony**. If its participants
collude and retain the secret, they can forge proofs — and in a settlement
layer that means **creating money without leaving a trace**. Forged proofs
verify correctly.

The price of avoiding this: 62 KB proofs instead of 192 bytes. A factor of
320.

It is the only decision in the project taken **against** the performance
figures.

### 7. Why is there no consensus?

Because it is a distributed-systems problem, not a cryptographic one, and
would require twenty to forty rounds of work in a field where nothing
learned here helps.

And there is a stronger reason: **badly implemented consensus is more
dangerous than none**, because it gives the appearance of a guarantee
without providing it. Byzantine failures are subtle and are not caught by
tests.

### 8. Why document your own errors?

Because work without documented errors usually means nobody really looked.

On record: the three tests that did not discriminate, the comparison that
mixed debug and optimized build figures, and the two constraints left as
empty placeholders — which are always satisfied and fail no negative test.

### 9. Why does the operator still see the balances?

Because it maintains the state. Whoever holds the account tree knows its
contents.

This system's privacy is **against third parties who only see proofs**,
not against whoever maintains the ledger. Removing that requires
replicating state across mutually distrusting parties — that is,
consensus.

### 10. Why can a frozen account still receive?

Because preventing it would strand funds and break legitimate payments
toward an account under investigation.

An honest payer does not know the recipient is frozen. Rejecting the
payment harms the payer, not the party under investigation.

### 11. Why does selective disclosure depend on the holder?

Because the alternative would be a supervisory master key, and such a key
is a target. **Here there is no key to steal** in order to obtain general
access to balances.

The trade-off is stated: if the holder refuses to cooperate, there is no
forced-disclosure mechanism.

---

## HOW

### 12. How is it proven that no money is created?

Through double-entry accounting inside the circuit: what leaves one
account enters another, and both Merkle climbs are bound to the same
position.

Total supply is **public** and changes only through proven issuance or
destruction, each with its own proof.

### 13. How is double-spending prevented?

Every operation generates a **nullifier** derived from the spend key and
the nonce. The circuit proves that its position in the tree was **free**
before the operation, and the layer inserts it when applying.

Only the holder can compute their nullifier, which prevents an observer
from precomputing them to monitor when someone else's account spends.

### 14. How does supervision work?

The holder generates a proof that their balance lies within a range. Three
modes from the same circuit:

| Mode | Reveals |
|---|---|
| Exact | The balance |
| Minimum | That it exceeds X |
| Band | That it lies between X and Y |

The supervisor verifies it with a free function, **without ledger access**.

### 15. How do you prevent a compromised custodian from issuing alone?

Issuance requires two distinct custodians. The real risk is not an
outsider signing — set membership closes that — but **the same custodian
counting twice**, which would turn a 2-of-N scheme into a covert 1-of-N.

This is closed by strictly increasing indices **bound to the Merkle paths**
via an accumulator. Without that second part, the index would be a declared
number unrelated to the position actually proven.

### 16. How do I know the operator has not rewritten history?

Through the **chained transition log**. Each operation leaves an entry
whose digest includes the previous one, so altering the past invalidates
everything after it.

**Publishing the head — 32 bytes — commits to the entire history**: two
copies with the same head share the same history.

This is what *Certificate Transparency* does with certificate authorities:
it does not prevent misbehavior, it makes misbehavior impossible to hide.

---

## WHO

### 17. Who can create money?

Two distinct custodians from a set committed to a public root, and only up
to an **immutable cap** of the ledger.

Not even the full set can exceed that cap without creating a new ledger,
which would leave a trace impossible to conceal.

### 18. Who controls the custodians?

A separate **governance set**, which can change the custodian set.

The circularity does not disappear — whoever controls governance controls
everything — but it moves to keys used almost never, which can be kept
offline, rather than operational keys exposed daily.

**The governance set is immutable.** If it is compromised, the only way
out is creating a new ledger. It is the deliberate end of the authority
chain.

### 19. Who is this useful for?

Honestly: **anyone who wants data on how zero-knowledge systems choose a
paradigm**, and anyone evaluating whether privacy with provable compliance
is viable and at what cost.

Not for anyone looking for deployable infrastructure. Zcash and Aztec
exist for that, with years and teams of head start.

---

## HOW MUCH

### 20. How much does it cost to operate?

| | |
|---|---|
| Layer startup | **0.67 ms** |
| Verifying a transfer | ~4 ms |
| Proving it | ~620 ms |
| **Verify / prove** | **0.5 %** |

That asymmetry is what makes the model viable: cost falls on the party
producing the proof, not on the party accepting it.

⚠️ A single run on one machine. Useful for order-of-magnitude comparison,
not as a benchmark.

### 21. How large is it? Does it scale?

**62 KB per transfer. One thousand transfers accumulate 59.1 MB.**

Resolving it requires recursive aggregation or batched proofs, neither of
which is implemented.

⚠️ **But it is not the limit that bites first**, and an earlier version of
this answer said it was.

A nullifier's position **is derived from the nullifier itself**, and the
circuit requires it to be free. By the birthday paradox, at around
**65,000 payments** the probability that two land on the same position is
already 39 %.

And the affected user **cannot retry**: their nullifier is deterministic,
so the payment is permanently blocked.

**The 59.1 MB are a cost. The collision is a stop**, and it hits a specific
user while the system is nowhere near saturation.

Two further limits exist —the pending tree exhausts at 2³² total payments,
and the custodian set caps at 128— and all four are in `AUDITORIA.md` §13.

### 22. How far is it from being usable?

For real use: **distributed consensus** and **external audit**. The first
is a separate project; the second does not depend on more code.

And something worth saying: **reaching production was never the goal of
this work.** The goal was to find out what changes when the same thing is
implemented across five paradigms, and to build something complete enough
that the answer would mean something.

That is done, and measured.

---

## Next

| | |
|---|---|
| Start here | [`README.md`](./README.md) |
| Break it | [`AUDITORIA.md`](./AUDITORIA.md) |
| The comparison | [`FIVE_BACKENDS.md`](./FIVE_BACKENDS.md) |
| The paper | [`PAPER_EN.md`](./PAPER_EN.md) |
