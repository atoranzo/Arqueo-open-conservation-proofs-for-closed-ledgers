> **Document of the ZK-SSL stage (until September 2026).** It is kept as it was, as history:
> its statements in the present tense describe the project of that time, not today's. The current
> position is in [`README.md`](../../README_EN.md) and [`doc/USE_CASES.md`](../USE_CASES.md); what
> changed since then is in [`AUDITORIA.md`](../../AUDITORIA.md) (§448).

# ZK-SSL — Vision, design principles and technical consequences

> **Español:** [`doc/historia/VISION.md`](./VISION.md) is the original of this page, in Spanish; it
> prevails wherever the two differ. This translation is of commit `7766efb` (audit entry §917) and
> was produced by the project's assistant, a Claude Code session, in audit entry §918 (`GENAI.md`):
> it is a generated artefact, not text written by the author. Line citations of the form `file:N`
> refer to the Spanish files.

**Towards a settlement layer as a clean vessel**

*Revised against what was built and measured. The principles have not
changed; what is known of their consequences has.*

---

## 1. Vision

To build a settlement layer in which:

- What is essential can be proven without revealing what is unnecessary.
- The intermediary's power is bounded, visible and minimal.
- The system acts as a **vessel of rules**: it receives operations,
  applies invariants and emits proofs, without appropriating the content
  or hiding its limits.
- Coherence between principle and architecture weighs more than
  performance, premature adoption or narrative.

The system does not aspire to remove all trust at a stroke. It aspires to
**not distort what is real**: where there is verification, it verifies;
where there is still trust, it names it.

---

## 2. Design principles

### P1 — Do not distort what is real

Every property claimed must be buildable, measurable and, preferably,
refutable by test. One's own errors are documented. Limitations are
declared up front, not in appendices.

### P2 — Clean vessel

The system holds state and rules. It must not see more than necessary,
retain more than necessary, nor depend on unauditable ceremony secrets.

### P3 — Minimal and explicit trust

Faith is removed where cryptography can replace it. Faith is declared
where it cannot yet be removed. No trust dependency is sold as
sovereignty.

### P4 — Measure (voluntary limits)

The system defines what it resolves and what it consciously rejects.
Widening the scope is not a good in itself. Preserving purity and
verifiability is.

### P5 — Separation of instrument / operator / key

Whoever operates the node must not need the spending keys. Whoever
generates the proof must not be confused with whoever applies it. The API
design must make hard the uses that reintroduce unnecessary trust.

### P6 — Coherence over brilliance

If an optimization introduces ceremony, opacity or a failure surface that
is hard to audit, it is discarded even if it improves latency or proof
size.

### P7 — Emptying, not accumulation

Data and privileges exist for the time strictly necessary. The system
favours **verifiable discard** over passive retention.

### P8 — Reproducibility without a protagonist

The system's value lies in third parties being able to rebuild it, measure
it and attack it. Legitimacy does not depend on the figure of the author.

---

## 3. Technical consequences

### 3.1 On state and the operator

*Consequence of P2 and P3*

| Requirement | Technical implication | Status |
|---|---|---|
| The operator must not read balances in the clear | Encryption at rest with a key it supplies at startup | ⚠️ **Partial**: it sees them in memory |
| The operator must not rewrite history | Chained log with verification at startup | ✅ |
| The operator must not demand spending keys | `account_view` + materials → proof on the client → `apply` | ✅ |
| Censorship and ordering remain residual power | Declared in the header; consensus as the only closure | ⚠️ **Open** |

**Current accepted state**: single node.
**Target state of coherence**: minimize the readability of the vessel's
contents, not only the forgery of history.

⚠️ **Encryption at rest does not close P2.** It protects against theft of
the disk, not against the operator. Closing it would demand a redesign by
commitments where the layer handles only roots, and that **is not done**.

### 3.2 On the proof backend

*Consequence of P3 and P6*

| Criterion | Decision |
|---|---|
| Setup ceremony | **Forbidden** as a sovereign dependency |
| Post-quantum resistance | Preferred |
| Performance | Secondary to the absence of ceremony |
| Backend chosen | **STARK/FRI** (no setup, post-quantum) |
| Discarded despite better performance | Groth16 (proofs 320× smaller) |

**Design rule**: no paradigm is adopted that allows undetectable value to
be created through collusion in the setup.

⚠️ **Measured and unanticipated consequence**: the choice closes the door
to **recursive aggregation**. Winterfell does not support recursion, and
building it would demand arithmetizing the FRI verifier. See §3.8.

⚠️ **The declared security is conjectured.** Without a field extension
there is a ceiling of 63 bits over Goldilocks. With a degree-2 extension
the level is reasonable, and **it costs 1.2× in time and 1.7× in size** —
measured.

### 3.3 On the API and the transfer flow

*Consequence of P5*

Mandatory flow:

1. The client obtains the public view and the materials.
2. It computes the nullifier and **generates the proof locally**.
3. The layer verifies and applies **without knowing the spending key**.

The proving function is free-standing —not a method of the layer— so that
the API does not suggest handing the key to the node.

**Discriminating test**: `materials_alone_are_not_enough_to_spend`.

⚠️ **P5 has a limit in the threshold authority**: on a single node,
whoever generates the issuance proof needs **both custodian keys at
once**. The real guarantee is *"two compromised keys instead of one"*, not
*"two independent wills"*. Closing it demands verifying signatures in
circuit, and **it is not done**.

### 3.4 On privacy and compliance

*Consequence of P1 and P3*

| Property | Mechanism |
|---|---|
| Privacy against third parties who only see proofs | Zero knowledge about transfers |
| Compliance without a general ledger | Selective disclosure: exact / minimum / band |
| External verification | The supervisor verifies without access to the ledger |
| Not feigning privacy against the operator | Explicit declaration |

✅ **RESOLVED — The payer no longer sees the recipient's balance.**

**The flaw was**: the materials of the one-step path handed the client a
complete view of the recipient, with **its exact balance, its public
identity and its nonce**, because that path updated the two leaves in one
transition and whoever proved needed both.

**How it was closed**: the two-phase path touches **a single leaf** per
operation. The send materials carry **only the recipient's identifier**,
and the type enforces it —there is no field through which the balance
could enter—. The one-step path was withdrawn entirely. See `AUDITORIA.md`
§29 and §32.

Paying someone one euro **reveals how much they have**.

| | The operator sees balances | The payer sees the recipient's |
|---|---|---|
| Who | One entity, declared | **Anyone who pays you** |
| Was it declared? | Yes, in the header | **No** |

**It is inherent to the account model.** Settlement updates the two
leaves, so whoever builds the proof needs the recipient's balance to
compute its new leaf. Zcash does not have it because **it does not update
the recipient's balance**: it creates a new note.

**How it was let through.** The code comment said: *"These are Merkle
paths and account data: state information, not secrets"*. That sentence
takes the leak as given instead of examining it, and nobody wrote the test
that would have given it away.

**Ways out, none cheap:**

| Way | Cost |
|---|---|
| Two-party protocol: each proves its side | Demands interaction and the recipient's availability |
| UTXO-style note model | Redesign of the whole system |
| Let the layer build the proof | **Violates P5**: reintroduces the key into the operator |

**It is more serious than the operator's visibility**, because the
operator is **one and declared**, while a counterparty is **anyone**.

#### The fix: transfers by notes

**The payer does not update the recipient's balance: it creates a note for
them**, and the recipient claims it.

```text
nota = H(H(identidad_receptor, r), [importe, época])
```

| | Today | With notes |
|---|---|---|
| The payer needs | The recipient's balance, identity and nonce | **Only its public identity** |
| What it learns about the recipient | **How much it has** | **Nothing** |

**What each party learns, verified:**

| Party | Learns |
|---|---|
| Payer | The recipient's public identity, which it uses as an address |
| Recipient | Nothing about the payer |
| A third party | Nothing: it only sees commitments |

**And the payer cannot spend it.** Knowing `r` is not enough: claiming it
demands proving the recipient's **spending key**.

**The pieces already exist** in the derived project: `circuit_issue`
creates a note tied to an identity, and `circuit_redeem` claims it by
proving it is yours. They were written for the offline mode and **solve
this problem without changes**.

#### ⚠️ The residue it leaves

**The payer chooses `r`, so it recognizes the note when it is claimed.**
It knows **when** the recipient collects, though not how much they have.

Zcash closes it by encrypting the note so that the recipient derives `r`.
**Here it is not resolved**, and it is much smaller than revealing the
balance — but it is a linkability leak and it is worth naming.

#### ⚠️ The usability cost

**The transfer becomes two steps**: the payer creates the note, the
recipient claims it. The money stays pending until the recipient acts.

It is exactly Zcash's model, and it is not free: today a payment completes
on its own, and with notes **the recipient has to do something**.

**Status: design analysed, not implemented.** The refactor of the layer
—two-phase transfers, pending notes, claiming— is estimated at 15-25
rounds.

⚠️ **No forced disclosure.** If the holder does not cooperate, there is no
alternative mechanism. It is deliberate —there is no master key to steal—
and in a regulated deployment it would be a policy decision to evaluate.

⚠️ **Side-channel leaks: weak evidence.** Proof size varies by 5.4% and
the correlation with the amount is +0.008 on a logarithmic scale. Time
does not depend on the amount either: a spread of 1.19×. **With 16 and 4
samples that is not a demonstration.**

### 3.5 On limits, closure and moderation

*Consequence of P4 and P7*

| Mechanism | Purpose | Status |
|---|---|---|
| Immutable issuance cap | Hard limit on value creation | ✅ |
| Public intervention counters | Visibility of the use of privileges | ✅ |
| Expiry of privileges | Avoid permanent residual power | ✅ **freezes** |
| Explicit rejection of scope | Document what will not be built | ✅ |
| Deterministic and verifiable snapshots | Copy without opaque formats | ✅ |
| **Verifiable discard over retention** | Reduce the sensitive surface | ✅ **see §3.8** |

### 3.6 On verification and method

*Consequence of P1 and P8*

| Practice | Technical requirement |
|---|---|
| Every security property | **Discriminating test**: a coherent witness that violates only that constraint |
| Measurements | Same machine, release mode, own figures |
| Design or measurement errors | **Documented, not erased** |
| Dependencies | Minimize; avoid hidden toolchains |
| External audit | Necessary; one's own tests **do not replace it** |

**Practice added by experience — the test that validates the test.**

Several properties carry a second test that checks that the first one
**can fail**:

| Test | Its validator |
|---|---|
| Balances are not readable on disk | Without encryption **they are** |
| A frozen account cannot spend | A free one **can** |
| Two spends reveal the identity | One spend **reveals nothing** |
| Double spending across batches is detected | Without the log **it is not detected** |

Without the second of each pair, the first would pass even if the check
were badly built.

**⚠️ Practice that was missing and is now on record: findings must live in
the code.**

Two of the eight documented findings were **reintroduced** when writing
new code without consulting what had been learned: the 64-bit identity and
the soundness ceiling. Documenting does not prevent repeating. A test that
fails does.

### 3.7 On governance and issuance

*Consequence of P3, P5 and P8*

| Element | Design | Status |
|---|---|---|
| Issuance | 2-of-N threshold | ✅ |
| Custodian governance | 2-of-N with a public counter | ✅ |
| Immutability of limits and cap | Yes | ✅ |
| **Advancing time** | **2-of-N, one at a time** | ✅ |
| Replaceability | Avoid roles whose loss freezes the system | ⚠️ **Partial** |
| Legitimacy | Reproducibility > the author's authority | ✅ |

**Time is also a privilege.** The system needed a notion of epoch for
deadlines, and advancing it is a power: whoever controls it **expires
freezes ahead of time**. It is closed like the others —threshold and
counter— and **without jumps**, so that no massive advance goes unnoticed.

⚠️ **The governance set is immutable.** If it is compromised, the only way
out is a new ledger. It is the conscious end of the chain of authority,
and it partly contradicts the replaceability criterion.

### 3.8 On the retention of proofs — **P7 applied**

*Consequence of P7, and the case that justifies it most*

**The problem.** A thousand transfers accumulate 59 MB of proofs. At a
million operations a day that is **59 GB a day**. That makes the system
undeployable.

**The usual answer —aggregate— is not reachable.** Recursion would demand
arithmetizing the FRI verifier. And batching would demand that someone
hold **the N spending keys** at once, which violates P5 directly.

**The question that was missing**: who needs the proof after applying it?

| Party | Does it need it? |
|---|---|
| The operator | **No.** It already verified it |
| The holder | **Yes**, if some day they want a third party to re-verify theirs |

**Central retention was assumed without examining it.** And P7 says the
opposite: *data exist for the time strictly necessary*.

| | Central retention | Distributed retention |
|---|---|---|
| Operator (1M op/day) | 59 GB/day | **131 MB/day** |
| Holder (10 op/month) | — | 620 KB/month |

**463 times less.** The operator keeps the chained log entry —137 bytes—
instead of the proof.

**And the discard is verifiable, not loss**: the log already stores
`H(prueba)`. Whoever keeps a copy proves it is exactly the one the log
says, without the operator holding it.

⚠️ **What remains open**:

- If nobody kept a proof, that transition **cannot be re-verified**. The
  log shows the chain, not the cryptographic validity of that operation.
- The holder **carries the custody**. In a real deployment their provider
  would take it on, with which retention re-concentrates —spread among
  providers, not in the central operator—.
- **Verification in bulk still needs N verifications.** That *would*
  demand recursion, and it is a smaller problem than 59 GB a day.

**Methodological lesson.** An earlier version of this analysis concluded
that the scale limit was *"the main reason why this is not deployable"*.
It was a hasty conclusion, resting on an assumption —central retention—
that **was never examined and that this document already contradicted**.

The principle was written before the problem. Applying it took three
rounds longer than necessary.

### 3.9 On the readability of state — **P2, design demonstrated**

*Consequence of P2. The largest declared incoherence of the system.*

**The problem.** The layer stores `(identidad, saldo, nonce)` for every
account and **the operator sees them in memory**. Encryption at rest
protects against theft of the disk, not against the operator. P2 —*it must
not see more than necessary*— **is not met**.

**The observation.** The layer does not need that content. To maintain the
tree and verify transitions, **the leaf's digest** is enough.

| | Today | By commitments |
|---|---|---|
| The layer stores | id, balance, nonce | **Only `H(H(id,saldo),nonce)`** |
| Computes roots and paths | Yes | **Yes** |
| **Can read a balance** | **Yes** ⚠️ | **No** |

**Why it is still sound.** The client supplies the position and the digest
of the new leaf. The layer verifies the proof, places the leaf, and
**checks that the resulting root is the one the proof attests**. If the
client lied about the leaf or the position, that check fails.

**It does not need to understand the content to detect it.**

**Status**: demonstrated in `commitment.rs` with **7 tests**, including
the two attacks —lying about the leaf, lying about the position— and the
check that the layer retains no balance, with its validator.

**And implementing it changed the approach.** It was a refactor of 101
references; it is **a migration**:

| | |
|---|---|
| The new path (`send`/`claim`) | **Does not read the registry at any point** |
| The old path (`transfer`, issuance…) | Needs it, and is marked |
| When no old path remains | The registry is deleted |

The balance is supplied by the holder and the layer **checks that it
produces the leaf it holds in the tree**. If they lied, it would not
match: proven in `a_holder_lying_about_their_balance_is_caught`, with its
validator.

The advantage over the one-shot refactor is that **at every moment the
system works**. And the acceptance criterion of §5 approves it: it reduces
what the operator sees, declares what remains, and does not break what was
there.

#### Status of the migration

| Module | Balance reads |
|---|---|
| `two_phase` (`send`/`claim`) | ✅ **0** |
| `burn` | ✅ **0** |
| `audit` | ✅ **0** |
| `mint` | ⚠️ 2 |
| `recovery` | ⚠️ 2 |
| `transfer` (old, with the leak) | ⚠️ 4 |
| `client` (materials) | ⚠️ 3 |
| `accounts` (the getters themselves) | ⚠️ 3 |

⚠️ **An earlier version of this table said that `mint`, `recovery` and
`transfer` were at zero. It was false**: the check looked for
`self.records.get` on one line, and those modules write it on several.

It is the same flaw that appears in §8: **taking the result of a search as
evidence without verifying that the search was complete**. Here it
produced an incorrect table in a public document.

**`disclose_exact` was the clearest case.** It read the balance with
`balance_of` —*"ask the layer how much you have and prove it"*— which is
exactly what this model eliminates. Now it is *"prove what you say you
have"*.

#### The two custodian operations do not migrate the same way

**`mint`**: the custodians credit someone else's account, so they need its
balance to compute the new leaf. It is **the same problem as the old
transfer**, and it has the same solution: **mint to a pending entry that
the holder claims**.

**Implemented** in `circuit_mint_pending`: **18 tests**. It raises the
supply and creates a pending entry **without touching any account**, so
the custodians do not need anyone's balance.

The property goes in the type: the signature receives the recipient's
identity and a random value. **There is no parameter through which a
balance could enter, nor a column to hold it.**

✅ **A flaw found while building it, and already fixed**: the issuance cap
was carried and declared publicly, but **the range check was missing**. It
was a constraint that existed in name and imposed nothing.

It came up on asking *"is every column I declare really used?"*, which
also uncovered that **seven new columns were never filled**: they were
zero and their constraints held trivially.

**Closed** with a 64-row segment that decomposes `tope −
suministro_nuevo` into 63 bits, in rows that were empty. Two tests pin it:
`minting_beyond_the_cap_is_rejected` and
`minting_exactly_up_to_the_cap_is_allowed`.

⚠️ **The pair is necessary.** The first version only had the negative and
**passed for the wrong reason**: the circuit rejected *any* issuance
because the test reused public inputs with a fixed amount.

It came up on asking *"is every column I declare really used?"*, which
also uncovered that **seven new columns were never filled**: they were
zero and their constraints held trivially.

It was closed with an **independent** 64-row segment —`CAP_LENGTH`—
instead of touching `NUM_SEGMENTS`, which would have shifted the indices
of all the periodic columns.

**Wired to the layer**: `mint_to_pending` and `apply_mint_to_pending`,
with **4 integration tests**. The full cycle works: the custodians mint to
a pending entry and the recipient claims it with `claim`.

⚠️ **Classic `mint()` still exists.** Withdrawing it demands migrating
whatever uses it, and **it is not done**.

✅ **The cap is now imposed by the circuit, not only by the layer.**

For a time only the layer imposed it, and the test was called
`the_supply_cap_is_enforced_by_the_layer` precisely so that it showed. It
was coherent with the declared model —the operator already controls the
layer— but it went against the principle of imposing in the circuit.

⚠️ **And the margin is a single bit.** The segment gives **63 bits, not
64**: if it gave 64, the wrapped value of a negative subtraction would fit
in the range and the cap would stop being imposed **without any test
noticing**. See `AUDITORIA.md` §14.

**`recovery`**: here there is a **structural tension**, not only pending
work.

The custodians change an account's identity while preserving its balance.
To compute the new leaf that balance is needed, and:

| Who could supply it | Problem |
|---|---|
| The operator | It is exactly what this model eliminates |
| The custodians | They do not know it |
| **The holder** | **Lost the key — may have lost their records too** |

If the holder keeps their notes even though they lost the key, they can
supply it and the migration is trivial. **If they lost the whole device,
they do not know it, and recovery becomes impossible.**

> **Recovery by custodians and an operator that does not see balances are
> partly incompatible.** One can have one or the other in its full form,
> not both.

Closing it entirely would demand that the balance be held under threshold
custody —the same design as the openable identity of the offline payment—
and **that gives the custodians a new power that would have to be counted
and bounded**.

**It is not resolved, and it is not obvious that it should be resolved in
favour of privacy**: an impossible recovery is a worse failure than an
operator that sees balances.

⚠️ **The operator still sees the balances** while unmigrated operations
remain. The migration advances one at a time, and **it is not finished**.

⚠️ **The cost, which is real.** The holder carries their own state. If
they lose their local copy **they do not know how much they have**, even
though the money is still there and recovery by custodians works.

Today the layer can answer *"you have X"*. With this design **it cannot**.
It is Zcash's model and a serious usability change. A real deployment
would solve it with the payment provider keeping the client's state — with
which readability re-concentrates, spread out and not in the central
operator.

### 3.10 On the rotation of privileges

*Consequence of P4 and P7: privileges exist for the time strictly
necessary.*

**The problem**: without rotation, a compromised custodian key **serves
forever**. The counters made its use visible; nothing bounded it.

**The design**: rotation is expressed **by use, not by time**, because
this layer has no notion of time.

| | |
|---|---|
| Mint, freeze, recover | Consume one intervention |
| When the quota runs out | The custodians **can no longer act** |
| Rotating the set | **Resets** the counter |

Running out **does not block the system: it forces renewal**. And the old
set becomes inert by another route — its root is no longer the current
one.

**5 tests**, including the one that checks that the quota **survives a
restart**: otherwise, restarting would be enough to keep using an
exhausted set.

**Two decisions that the compiler and the order gave away:**

Consumption happens **on applying**, not on generating the proof. Placed
in generation, discarded proofs would have exhausted the quota. The
compiler pointed it out by rejecting a mutation on `&self`.

And it happens **after** verifying the authority: before, anyone could
exhaust the custodians' quota without being one.

⚠️ **What it does NOT cover.** `set_max_custodian_uses` is not protected
by any authorization: **an operator can raise the quota and nullify the
rotation**.

It is coherent with the declared model —the operator already controls the
layer— but rotation is **a policy the operator applies, not a guarantee
that binds it**. Imposing it would demand carrying the quota into the
issuance, freeze and recovery circuits. **It is not done.**

---

### 3.11 The ISO bridge cannot use the leak-free path

**The leak of the balance to the payer is closed in the direct API and
open in the interface through which a bank would enter.**

`iso.rs` calls `transfer()`, the old path. Not by oversight: **the
two-phase model is incompatible with the semantics of ISO 20022.**

| | |
|---|---|
| `pacs.002` with `ACSC` | *"settled and **final**"* |
| The two-phase model | **Not final until the recipient claims** |

A `send` leaves the money in a pending entry. If the bridge answered
`ACSC`, it would be asserting something false: the recipient does not have
the money yet, and might never claim it.

#### The decision: A, and it does not change the contract

| Option | Why it is discarded or taken |
|---|---|
| **B.** Accept the leak on the ISO path | **Discarded.** The project's thesis is that privacy and compliance are compatible. Leaving the leak in the interface through which a bank enters contradicts it at the exact point where it matters. |
| **C.** Declare ISO incompatible with private transfers | **Discarded.** It is honest, but it gives up a result the project did demonstrate: that a `pacs.008` can be settled with a proof attached. |
| **A.** Two states: accepted and final | **Taken.** |

**The argument that decides: A does not invent a new contract.**

ISO 20022 **already distinguishes acceptance from finality**:

| Code | Means |
|---|---|
| `ACSP` | Accepted, settlement **in process** |
| `ACSC` | Settlement **completed** |
| `RJCT` | Rejected |

And that **is** the two-phase model:

| | |
|---|---|
| `send()` — the money leaves the payer and waits in a pending entry | **`ACSP`** |
| `claim()` — the recipient collects it, the operation is final | **`ACSC`** |

The bridge modelled only `ACSC` and `RJCT`: **it collapsed two states into
one**. The objection "it changes the contract" was mine and it was false —
settlement in two steps is the norm in payments, and the standard has the
vocabulary.

#### What was done

✅ **`TxStatus::InProcess` → `ACSP`** exists in the bridge.

✅ **The three status codes, verified** against the real catalogue.

#### The change, made additively

✅ **`settle_pacs008_two_phase` exists and works**, with three tests:

| Test | What it demonstrates |
|---|---|
| `..._reports_acsp_not_acsc` | Returns `ACSP`: accepted, not final |
| `..._does_not_touch_the_recipient` | The recipient's balance **does not change** |
| `the_classic_bridge_does_touch_the_recipient` | And the classic path **does** credit it |

⚠️ **The third one is necessary.** Without it, the second could be
checking that the operation failed instead of that it does not touch the
recipient.

**It was added next to the classic one, without touching its signature.**
No existing test changed: the two paths remain in view to compare.

⚠️ **Classic `settle_pacs008` still exists**, and still leaks. It is the
path through which a bank would enter today if it does not choose the
other.

✅ **And the collection phase too**: `claim_pacs008` returns the second
`pacs.002` with `ACSC`. The standard's `RCVD → ACCP → ACSP → ACSC` cycle
is materialized in its last two stages.

| Test | What it demonstrates |
|---|---|
| `the_full_two_phase_iso_cycle` | `ACSP` → the recipient has nothing → collects → `ACSC` |
| `a_third_party_cannot_claim_the_pending` | With the full notice in hand, without the key nothing is collected |

#### ⚠️ The piece ISO 20022 does not carry

`settle_pacs008_two_phase` returns `(Pacs002, Option<PendingNotice>)`. The
notice —position, random value, amount— **goes outside the message**,
because the standard has no field for it and using the remittance
information one would be forcing it.

**It shows in the type on purpose**: whoever uses this bridge has to work
out how that notice reaches the recipient, and the signature reminds them.

⚠️ It was discovered **while writing the full-cycle test**: the first
version returned only the `Pacs002`, and with that **the recipient could
not collect**. The path was impossible to use and nothing pointed it out.

#### ✅ Classic `settle_pacs008`, withdrawn

**No ISO path that leaks the recipient's balance remains.**

It was withdrawn instead of being marked deprecated, by the principle that
§3.11 itself established:

> A leak that is present and reachable **is a leak**, even if a better
> alternative exists next to it.

**What changed on migrating the tests:**

| | Before | Now |
|---|---|---|
| Status of a valid payment | `ACSC` | **`ACSP`** |
| Recipient's balance after the send | +250,000 | **unchanged** |
| Test name | `..._settles_...` | **`..._is_accepted_...`** |

⚠️ **The second row is the property, not a detail.** The recipient does
not have the money until they collect, and **that is why the payer does
not need to know their balance**.

And the name matters: `ACSP` **is not final settlement**, so a test called
*«settles»* would lie about what it checks.

#### What remains of the old path

`transfer()` is still in the layer. **It no longer has production
consumers**: its 35 remaining uses are all in tests.

⚠️ **Withdrawing it demands migrating those 35**, and some check
properties —supply invariant, restart, transition log— that would have to
be rethought over `send`/`claim`. **It is not done.**

**The apparent obstacle and why it is not one.** `send()` needs
`sender_state` —the balance and the nonce **declared by the holder**— and
the ISO message does not carry it. If the bridge asked the layer for it,
**it would be reading the registry**, which is exactly what the new path
avoids.

The way out is already in the docstring of `settle_pacs008` itself, said
for the key:

> *`sender_key` comes from a key store, not from the message: ISO 20022
> does not carry cryptographic keys.*

**The same argument holds for the state.** The debtor's bank knows its
client's balance: that is ordinary banking, not a concession.

**The new signature:**

```rust
pub fn settle_pacs008(
    layer: &mut SovereignLayer,
    registry: &IbanRegistry,
    msg: &Pacs008,
    sender_key: BaseElement,
    sender_state: &ClientState,   // del mismo almacen que la clave
    salt: Digest,                 // lo elige el pagador
) -> Pacs002                      // devuelve ACSP, no ACSC
```

**And a new function** for the second phase, which returns `ACSC` when
the recipient collects.

#### ⚠️ What it costs, said with numbers

| | |
|---|---|
| Calls to update | **11**, all in `iso.rs` |
| Tests that change semantics | Those asserting `Settled` move to `InProcess` |
| New function | The collection phase |

**It is not done here because changing the public API and the semantics
of eleven tests at once is exactly the kind of change this audit has seen
go wrong.** Leaving it designed and verifiable is preferred to leaving it
half done.

✅ **Verified against the real catalogue.** `ACSP` is
*AcceptedSettlementInProcess* and `ACSC` is *AcceptedSettlementCompleted*;
the standard cycle is `RCVD → ACCP → ACSP → ACSC`. The correspondence with
`send` and `claim` is the correct one.

⚠️ **With a declared nuance**: `ACWP` (*AcceptedWithoutPosting*) exists,
semantically even closer, discarded because its definition confines it to
holds for scrutiny. See `AUDITORIA.md` §21.

#### Why this was wrongly declared

The priority table marked priority 0 as **✅ RESOLVED**. It was false: a
path that does not leak exists, but **the one that is exposed does**.

> A leak that is present and reachable **is a leak**, even if a better
> alternative exists next to it.

---

## 4. Order of priority for greater coherence

*Updated with what was done.*

| | Priority | Status |
|---|---|---|
| **0** | Close the leak of the recipient's balance to the payer | ✅ **Closed on the ISO path**: classic `settle_pacs008` **withdrawn**. No ISO path that leaks remains. ⚠️ `transfer()` is still in the layer, without production consumers (§3.11) |
| 1 | Reduce the readability of state by the operator | ⚙️ **In migration**: the new path (`send`/`claim`) **does not read the registry**. The old one does, and is marked (§3.9) |
| 2 | Maintain and harden the key / layer separation | ✅ Except for the threshold authority (§3.3) |
| 3 | Formalize the catalogue of rejections | ✅ Roadmap with what is not reachable and what is unnecessary |
| 4 | Privileges with measure: rotation, counters, expiry | ⚙️ **Partial**: public counters and rotation by use (§3.10). ⚠️ **Expiry of freezes does NOT exist**: `freeze.rs` says «dura hasta que alguien la levante» [it lasts until someone lifts it] |
| 5 | Distributed consensus | ⬜ **Open**. The only real closure of censorship |
| 6 | External audit | ⬜ **Condition, not capability** |
| 7 | ⚠️ **Capacity of the nullifier tree** | ⬜ **Only affects `transfer()`**: `send`/`claim` do not use nullifiers. Withdrawing the old path closes it without touching circuits — but **it comes back with priority 5** (§13) |
| 8 | **Exhaustion of the pending tree** | ✅ **Closed**: positions are recycled on collection. The limit becomes one of **simultaneous** pendings |

**Priority 0 appeared on starting the refactor of priority 1**, and
displaces it. Closing the operator's visibility while any counterparty
sees the balances would be solving the smaller problem first.

Priority 1 is no longer an unknown: the design is demonstrated (§3.9) and
what remains is mechanical work.

---

## 5. Acceptance criterion for future changes

Every change of architecture, backend or API must answer yes to:

1. Does it preserve the absence of unauditable ceremony?
2. Does it reduce or declare the residual trust?
3. Does it prevent the operator from **seeing or retaining** more than
   necessary?
4. Does it keep discriminating tests for the affected properties?
5. Can it be explained without selling as resolved what is still open?

**Criterion added by experience:**

6. Does the finding that motivates the change stay **in the code**, not
   only in a document?

If any answer is negative, the change is not coherent with the vision.

### 5.1 Recorded application of the criterion

| Change evaluated | Answer | Result |
|---|---|---|
| Batching proofs | Fails 3: it would demand the spending keys | **Rejected** |
| Proof delegation | Unnecessary: 620 ms and 14 MB fit on a phone | **Not built** |
| Distributed retention | Meets all six | **Adopted** |

Rejecting by criterion and **documenting the rejection** is what P4 calls
a *catalogue of rejections*.

---

## 6. Derived project

The ZK-SSL circuits are reused in
**[euro-digital-zk](https://github.com/atoranzo/euro-digital-zk)**, which
implements concrete requirements of the digital euro regulation: a
provable holding limit, overflow to a linked account, offline payment with
identity disclosure on double spending, and return of unspent notes.

It serves as **a test of the principles**: applying them to external,
published requirements shows which ones hold. All eight held; two findings
were reintroduced for not consulting them (§3.6).

---

## 6.bis Relation to the «último intermediario» [last intermediary]

The underlying direction of this document coincides with the critique that
*consensus is the last intermediary*: every decision here —STARK without
ceremony, delegated path, wrapped leaf, acknowledgement— reduces what has
to be entrusted to someone. The full argument, with what was removed, what
is residual and what only physics will be able to take away, is in
`PRINCIPIOS.md` §6.bis.

## 7. Closing

This document fixes the vision of ZK-SSL as a **settlement vessel with
provable rules**: minimal faith, maximum clarity of limits, separation
between instrument and residual power, and a preference for verifiable
purity over brilliance.

The architecture is not judged only by what it allows one to do. It is
judged by **what it prevents from being falsified** and by **what it dares
to declare it does not yet resolve**.

And there are two proofs that the principles work, neither foreseen.

**P7 solved a problem that seemed to require a technology this project
does not have.** The principle preceded the problem, and applying it was
enough.

**P2 uncovered a worse flaw than the one it was going to fix.** On
starting the refactor so that the operator would not see the balances, it
appeared that **any counterparty already sees them** (§3.4). It had been
there from the start, with a comment that justified it instead of
examining it.

That is the argument in favour of this document: **a principle applied in
earnest finds what the implementation took for granted**.

---

*Angel Toranzo Portela · MIT / Apache-2.0*
