# What breaks when you actually build it

### Zero-knowledge settlement across five proof systems, and the nine questions that found nine bugs

**Angel Toranzo Portela** · MIT / Apache-2.0

---

## Abstract

Most zero-knowledge benchmarks measure the same circuit: SHA-256. It is a
useful reference and it hides everything that matters, because a hash
function has no state.

This work implements the same **financial settlement circuit** — accounts,
balances, spend authority, double-spend prevention, selective disclosure —
across **five proof paradigms**, measures them under identical conditions,
and documents what only appears when you port an application with state.

The most consequential result is not a performance number. It is that
**AIR arithmetization has no copy constraints**, which opens a silent
soundness gap in Merkle tree updates that no honest witness ever reveals.

A second result concerns method. After the system was complete, tested and
documented, **nine transversal questions were applied to it. All nine found
something** — including a systematic bug across five operations that left
the node's in-memory state inconsistent with its disk after any failed
apply.

Everything is reproducible with stable Rust and a single command. The
mistakes are documented alongside the results.

**Repositories:**
- [ZK-SSL — comparative study and sovereign settlement layer](https://github.com/atoranzo/ZK-SSL-ZK-Sovereign-Settlement-Layer-)
- [euro-digital-zk — digital euro privacy requirements](https://github.com/atoranzo/euro-digital-zk)

---

## 1. Why SHA-256 is not enough

Benchmarking a hash function tells you how fast a prover computes hashes.
It tells you nothing about what happens when a circuit has to prove that a
**balance decreased by exactly the amount another balance increased by**,
that the spender **held the key**, that the same money **was not spent
twice**, and that none of this **reveals who paid whom**.

Those constraints interact. And their interaction is where the design
decisions — and the mistakes — live.

So the same settlement circuit was implemented five times.

| | Groth16 | Halo2/IPA | **STARK/FRI** | PLONK/KZG | Nova |
|---|---|---|---|---|---|
| Trusted setup | Per circuit | **None** | **None** | Universal | **None** |
| Proving | 422 ms | 4.86 s | **39 ms** | 6.85 s | — |
| Verification | 5 ms | 91 ms | **1 ms** | 8 ms | — |
| Proof size | **192 B** | 4,096 B | 36.7 KB | 1,008 B | — |
| Post-quantum | No | No | **Yes** | No | No |

*Same machine, release builds, single run. Figures compare orders of
magnitude, not vendors.*

---

## 2. The decision that cost the most

**Groth16 was rejected despite being faster and producing proofs 320 times
smaller.**

Groth16 and PLONK/KZG require a **trusted setup ceremony**: a group of
participants generates parameters from a secret they must destroy. If they
collude and keep it, **they can forge proofs and create money leaving no
detectable trace**. The forged proofs verify correctly. There is no
after-the-fact detection.

That dependency is **permanent** — it does not expire — and
**unauditable**: a video of someone destroying a hard drive proves they
destroyed a hard drive, not that they did not copy the secret first.

For an institution with a sovereignty mandate, 192 bytes is not worth that.

**It is the only decision in this project taken against the performance
numbers**, and it is the one that defines everything else.

---

## 3. The finding that matters most

### AIR has no copy constraints

Updating a Merkle tree requires proving **two climbs from the same
position**: one for the old leaf, one for the new. Nothing in AIR
arithmetization forces both climbs to use **the same sibling nodes**.

A malicious prover could climb with different paths and produce a new root
that does not correspond to modifying that position at all.

**What makes it dangerous is that it is silent.** An honest witness always
uses the same siblings, so no legitimate proof ever reveals the gap. And
adversarial witnesses tend to fail on other constraints first, which masks
it further.

**You do not hit this porting SHA-256.** It needs an application with
state.

The fix is a constraint that forces the sibling of lane A to equal the
sibling of lane B at every level. Four constraints. Easy to write once you
know — and easy to omit forever if you do not.

Plonkish arithmetizations — Halo 2, PLONK — have copy constraints by
construction. **This finding applies to anyone building on STARK.**

### Seven more

| | |
|---|---|
| **Goldilocks is too narrow for identities** | 64 bits means collision in 2³². An attacker could find an identity colliding with an innocent person's |
| **63-bit soundness ceiling** | Without field extension, over Goldilocks. Correcting it costs **1.2× in time and 1.7× in size** — measured |
| **127 conjectured vs 29-63 provable bits** | Coexisting in the same configuration |
| **PLONK/KZG was the slowest prover** | 16-22× slower than Groth16 among curve-based systems. Implementation may account for part of it; the data cannot separate the two |
| **Only two of six libraries** | Prevent unsafe setup in code. The rest let it pass silently |
| **The PLONK/KZG Rust ecosystem is vertically fragmented** | Components that should compose, do not |
| **zkVMs are not comparable** | 3 dependencies against 349 |

---

## 4. The digital euro, measured

The same circuits were applied to two concrete requirements of the European
Union's digital euro regulation.

### Provable holding limits

The regulation contemplates a cap on how many digital euros a person may
hold. **Checking that cap requires knowing balances**, which destroys the
privacy it is meant to coexist with.

The circuit proves the recipient's balance stays under the cap **without
revealing it**, checked on every incoming payment. And when it does not
fit, the excess is routed to a linked account — with a constraint that
prevents abusing the mechanism:

```
(limit − resulting_balance) × diverted = 0
```

Either the digital account ends **exactly at the limit**, or nothing was
diverted. Without it, a payment provider could route funds to the linked
account even when they fit — and if it charges different fees per route, it
has an economic incentive to.

### Offline payments

Without connectivity there is no registry to consult. **Nothing prevents
spending the same money twice**, and no proof fixes that: the information
does not exist at that moment.

So double-spending is not prevented. **It is made to reveal the offender.**

Chaum, Fiat and Naor (1988), adapted to zero-knowledge proofs. Each note
commits to the holder's identity and a random value, both 256 bits.
Spending with an unpredictable challenge reveals one point of each line.

**One point says nothing. Two determine the line, and its intercept is the
identity.**

| Measured, production-grade parameters | |
|---|---|
| Proving on the payer's phone | **32.3 ms** |
| Verification on the merchant's terminal | **1.22 ms** |
| Proof size | 52 KB |

**32 ms on a phone is imperceptible.** That turns *"is cryptographic
privacy feasible in an everyday payment?"* from a position into a number.

### What that number does not settle

The Eurosystem is investigating **embedded secure elements** for offline
payments: a tamper-resistant chip that *refuses* to spend twice.

| | Hardware | Cryptographic |
|---|---|---|
| Offline double-spend | **Prevented** | Detected afterwards |
| Trust required | Chip manufacturer | **None** |
| If the primitive breaks | Unlimited and **undetectable** | Not applicable |
| Cost per payment | Negligible | 32 ms |

**Neither dominates.** Hardware prevents fraud rather than pursuing it,
which is preferable — but concentrates the risk in a physical primitive
that **does break**, and when it does the fraud is unlimited and invisible.

The honest answer to *"why not cryptography, then?"* may well remain
*"because hardware is cheaper and prevents rather than detects"*. The
difference is that it can now be argued with data.

---

## 5. The result about method

The system was complete: 366 tests, ten circuits, a persistence layer, an
audit-preparation document listing where the author had least confidence.

Then nine questions were applied to it. **Not to any function — to the
system.**

| Question | What it found |
|---|---|
| What does **each participant** learn? | **The payer learns the recipient's balance** |
| Against **whom** is this privacy claim true? | The issuing bank can link offline payments to payers |
| What protection does **each artifact** holding the same data have? | Snapshots were written unencrypted while the database was encrypted |
| What is checked **before** authorization? | Freeze status leaked to non-holders |
| What if a receipt is **replayed**? | Two tests missing; one of them would have created money |
| What if an operation **fails halfway**? | **Five operations left the state corrupted in memory** |
| What does the **documentation** claim that the code does not do? | Five stale figures and a limitation that no longer existed |
| What states do **combinations** reach? | A frozen account could still destroy its own money |
| What can the **operator** do that is not declared? | **It can redirect a payment if you do not check the recipient** |

**Nine questions. Nine findings.** In code that compiled, passed every
test, and had been reviewed by its author with the explicit intention of
finding its own faults.

### What they have in common

**None of them is answered by reviewing a function.** Each compares things
that are individually consistent:

- Two artifacts holding the same data with different protection.
- Five operations repeating an incorrect pattern, each coherent with
  itself.
- A privacy claim true against one adversary and false against another.

The systematic bug is the clearest case. Every one of the five operations
mutated state *before* verifying that the resulting root matched what the
proof attested. The error was returned correctly — but the change had
already happened in memory. **The fault was not in any operation. It was in
all five.**

### And the uncomfortable conclusion

If nine questions found nine things, there is no reason to believe the
questions are exhausted. The ones still unasked are **the ones that did not
occur to the author** — which are precisely the ones an outside reviewer
would find.

**That is the strongest argument for external audit this project contains,
and it is not rhetorical. It is the record of what happened when the
questions were asked seriously.**

---

## 6. What this is not

- **Not audited by anyone.** No external review, at any scale.
- **Not a distributed network.** Single node. The operator sees all
  balances and can censor.
- **Not affiliated** with the European Central Bank, the Eurosystem or the
  European Commission. It responds to no call for proposals.
- **Not deployable.** What is missing is not circuits: it is operations,
  disaster recovery, hardware security modules, compliance, support. A
  payment system is 10% cryptography and 90% everything else. This is the
  10%.

### Two privacy failures remain open

**The payer sees the recipient's balance.** Inherent to the account model
with a single prover: settlement updates both leaves. A two-phase design
closes it and is demonstrated with 8 tests; the layer refactor is pending.

**The issuing bank can link offline payments to payers.** Structurally
unsolvable in this design: closing it would require the circuit to verify
post-quantum public-key encryption, which means lattices inside a STARK —
or elliptic curves, which would destroy the reason the system exists.

Both are documented in full, with the reasoning, in the repositories'
audit-preparation files.

---

## 7. What this is for

Not infrastructure. **Evidence.**

If someone asks whether a given cryptographic property is buildable and
what it costs, there is now an answer with numbers instead of a position —
including the cases where the answer is *no*.

And a second thing, less usual: **a record of what a system looks like when
its author sets out to find its own faults and writes down what he finds**,
including the two findings that are still open and the one correction that
turned out not to work.

---

## Reproduce it

```bash
git clone https://github.com/atoranzo/ZK-SSL-ZK-Sovereign-Settlement-Layer-
cd ZK-SSL-ZK-Sovereign-Settlement-Layer-
cargo test -p zk-ssl --release
cargo test -p stark-experiment --release
```

```bash
git clone https://github.com/atoranzo/euro-digital-zk
cd euro-digital-zk
cargo test -p zk-circuits --release
cargo test -p zk-circuits --release metrics -- --nocapture
```

Stable Rust only. No external toolchains, no installers, no ceremony
artifacts to download.

Both repositories include an audit-preparation document with the threat
model, the invariant table, and **an explicit section on where the author
has least confidence**.

---

## Cite

If this is useful, cite the Zenodo record. If you find something wrong,
**open an issue** — that is worth considerably more.

---

*Independent work. MIT or Apache-2.0, at your option.*
