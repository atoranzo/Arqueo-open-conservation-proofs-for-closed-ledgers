# The most important idea, explained so that anyone can understand it

> **Español:** [`doc/IDEA_CENTRAL.md`](./IDEA_CENTRAL.md) is the original of this page, in Spanish;
> it prevails wherever the two differ. This translation is of commit `7766efb` (audit entry §917)
> and was produced by the project's assistant, a Claude Code session, in audit entry §918
> (`GENAI.md`): it is a generated artefact, not text written by the author. Line citations of the
> form `file:N` refer to the Spanish files.

## The underlying problem

Today, when you trust a bank or a payment system, you actually trust
**people and institutions**:

- You trust that they do not tamper with the books.
- You trust that they do not create money quietly.
- You trust that when the supervisor asks for information, the system
  answers properly.
- You trust that nobody rewrites the past on the sly.

That is not necessarily bad. But it is **faith in intermediaries**.

---

## What a system like this changes

Instead of only saying *"trust us"*, it allows saying:

> **"You do not have to believe me: check it."**

That is achieved with cryptographic proofs.

### That no money is invented

The system can **prove** that every movement conserves value. It is not
a report from the bank: it is a proof that anyone verifies.

### That nobody spends what is not theirs

It can be proved that whoever moved the money had the right to do so,
**without showing their whole financial life**.

### That the supervisor does its job without seeing everything

Instead of handing over the whole ledger, a person can prove:

- *"I have at least X"*
- *"I am between X and Y"*
- *"my balance is exactly Z"*

The supervisor verifies the proof **without seeing the rest**.

### That the past is not rewritten quietly

The history is chained in such a way that tampering with it on the sly
**is detected**.

---

## What does NOT change, and it has to be said clearly

This system **does not eliminate all power**.

There is still someone who operates it and who can:

- See the balances.
- Decide the order of operations.
- Block or delay something.
- Become a point of failure if it goes down.

And there is something else worth saying, because it was discovered
**after** writing the first version of this text:

> ⚠️ **When you pay someone, you learn how much they have.**
>
> It is not a failure of the operator: it is a consequence of the design.
> It is documented and has a known correction, but **today it is so**.

That is why the central idea is not *"there is no longer anyone to
trust"*.

The central idea is:

> **"Much less is trusted, and what still requires trust is said
> openly."**

That honesty is part of the design, not a minor detail.

---

## Why it matters in real life

Today we are usually trapped between two extremes:

| | |
|---|---|
| **All opaque** | The bank sees everything and we believe |
| **All transparent** | Anyone can see too much |

This approach seeks a third way:

- **Privacy** against the curious and third parties.
- **Verification** for whoever has a legitimate need to control.
- Less blind faith, more checking.

In one sentence:

> **You can prove what is necessary without undressing, and you can
> verify what matters without having to believe by default.**

---

## An analogy

Imagine a shared safe.

**Current model.** A trusted person keeps the key and the book of who
put money in or took it out. Everyone trusts that person.

**This model.** Every time someone takes money out or puts it in, they
leave a **mathematical proof** that they did it right. Anyone can check
it. Nobody needs to see the entire contents of every box.

But **there is still someone who maintains the building**. That someone
should not be able to forge the book in secret — and if they can still
see or block things, it has to be said.

---

## What is worth keeping in mind

This does not promise a world without power or without institutions. It
promises something more modest and more realistic:

| |
|---|
| Less *"trust me"* |
| More *"check it"* |
| Privacy when there is no need to expose |
| Control when there is a need to prove |
| **Clear limits on what is not yet solved** |

It is not magic. It is a change of basis: **from opaque faith to
verification with honest limits**.

---

## And a final warning

None of this is audited by anyone external.

While this project was being written, **six bugs were found in code that
already worked and passed all its tests** — including one that left the
system's state corrupt in memory after an error.

If six well-posed questions found six things, **there is no reason to
think they have run out**. That does not invalidate what was built: it
places it.

A system that says *"check it"* has to be the first to accept being
checked.
