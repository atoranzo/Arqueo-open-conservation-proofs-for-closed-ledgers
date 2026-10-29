> **Document of the ZK-SSL stage (until September 2026).** It is kept as it was, as history:
> its statements in the present tense describe the project of that time, not today's. The current
> position is in [`README.md`](../../README_EN.md) and [`doc/USE_CASES.md`](../USE_CASES.md); what
> changed since then is in [`AUDITORIA.md`](../../AUDITORIA.md) (§448).

# The ceremony and the secret

> **Español:** [`doc/historia/DIVULGACION.md`](./DIVULGACION.md) is the original of this page, in
> Spanish; it prevails wherever the two differ. This translation is of commit `7766efb` (audit entry
> §917) and was produced by the project's assistant, a Claude Code session, in audit entry §918
> (`GENAI.md`): it is a generated artefact, not text written by the author. Line citations of the
> form `file:N` refer to the Spanish files.

**What I learned building the same private payment system five times**

---

## The problem nobody has solved well

Imagine you want to transfer money and have two conditions hold at the
same time.

**First**: nobody else can see how much you have, how much you send or to
whom. Not the neighbour's bank, not a curious employee, not whoever taps
the wire.

**Second**: a supervisor must be able to check that you are not
laundering money, that you do not exceed the legal limits, and that the
money existed before you sent it.

The two seem incompatible. If nobody can see anything, how is it
supervised? And if the supervisor sees everything, where is the privacy?

The solution we use today is simple and old: **trust**. A bank sees
everything and certifies that the rules are met. It works because we have
decided that it works, not because anything guarantees it.

There is another way. It is called a **zero-knowledge proof**, and the
idea is almost counterintuitive: proving that something is true without
revealing why it is.

---

## Proving without telling

The classic example is a ring-shaped cave with a locked door at the back.
Someone claims to know the word that opens the door.

Instead of telling it to you, they go in one side and come out the other.
If they knew the word, they could do it. If not, they would have to come
back the way they went in and would only get it right half the time.
Repeat it twenty times and the probability that they are pretending is
one in a million.

**You have been convinced and you still do not know the word.**

Zero-knowledge proofs do that with mathematics instead of caves. I can
prove to you that my balance is between ten thousand and twenty thousand
euros without telling you what it is. I can prove to you that this
transfer conserves the money —that what leaves one account enters
another— without revealing the accounts or the amount.

It is not theory. Zcash has used it since 2016. What is not solved is
**how to apply it to a complete settlement system**, with issuance of
money, supervision, freezing of accounts and all the rest.

That is what I set out to build.

---

## The ceremony

This is where the story gets strange.

Most zero-knowledge proof systems need some initial parameters. Those
parameters are generated from a secret number. And that number **has to
be destroyed**.

Because whoever keeps it can **forge proofs**. They could prove they have
a billion euros when they have nothing, and the proof would verify
perfectly. Nobody would detect it. Ever.

In the jargon it is called **"toxic waste"**.

To keep a single person from knowing it, a **setup ceremony** is
organized. Several people each contribute their fragment of the secret,
and it is enough for **a single one** to be honest and destroy theirs for
the whole to be secure.

The real ceremonies are extraordinary to read. Zcash's, in 2016, had
participants who bought new computers with cash, booted from discs they
burned themselves, did the computations without ever connecting to the
internet, and then **physically destroyed the hard drive with a hammer**,
recording the process on video.

One of them did it inside a car driving along a motorway, so that nobody
could predict his location.

It is magnificent. And it is also exactly the problem.

---

## Why that is no good for a central bank

Think about what that ceremony means if the system is going to issue a
country's currency.

**You cannot check it.** The hammer video proves they broke a disk, not
that they did not copy the secret first.

**It does not expire.** Thirty years from now, those people —or whoever
has their files— will still have the ability to forge proofs if they kept
something.

**And the failure would be undetectable.** There is no later audit that
would discover it, because the false proofs are mathematically
indistinguishable from the true ones.

A central bank whose mandate is monetary sovereignty would be permanently
delegating it to a group of strangers who held a ceremony decades ago.

**Not all systems need a ceremony.** And that is where the interesting
part begins.

---

## Five times the same thing

There are at least five families of zero-knowledge proof systems, and
everyone has opinions about which is best. The comparisons that exist
measure small things: how long they take to prove that they know the
input of a hash function, for example.

My suspicion was that those comparisons **do not capture what matters**,
because a payment system is not a hash function. It has state that
persists, invariants that must always hold, and several authorities with
different powers.

So I did something tedious: **I implemented the same settlement system
five times**, once in each paradigm, and measured everything under the
same conditions.

Months of work to answer a question that fits in one line: *what really
changes?*

---

## The decision that goes against the numbers

The results were clear, and the choice obvious... until it stopped being
so.

**Groth16** is the fastest and produces proofs of **192 bytes**. It fits
in a text message.

**STARK** produces proofs of **62 kilobytes**. Three hundred times
larger.

Any sensible engineer would choose Groth16. I chose STARK.

**Because Groth16 needs a ceremony and STARK does not.** STARK relies
exclusively on hash functions: there is no secret to destroy, no ceremony
to hold, nobody to trust.

And there is a bonus: since it does not use the mathematics a quantum
computer would break, **it survives quantum computing** without being
redesigned.

It is the only decision in the whole project that I took **against** what
the measurements said. And I still think it is the right one: in monetary
infrastructure, two hundred extra bytes are a cost; an unauditable and
permanent dependency is a design flaw.

---

## The flaw no test could find

Here is what makes this work contribute something, and it is not a
measurement.

Each paradigm expresses computation differently. On porting the system
from one to another, I found a security hole **that only exists in one of
them and that is invisible**.

Without technicalities: to update an account's balance you have to prove
two things —how it was before and how it ends up after— and both proofs
must refer **to the same account**.

In some paradigms you can directly demand "these two computations use
the same data". In the paradigm I chose, **that cannot be expressed**. And
if you do not notice, an attacker could prove they modified one account
when in reality they modified another.

The unsettling thing is not the flaw. It is that **no test finds it**.

An honest implementation always uses the same data in both computations,
so all the legitimate proofs work and all the malicious proofs you can
think of also fail for other reasons. The hole only appears if you sit
down and ask yourself: *what am I demanding here, and what am I not?*

That finding would not have come out of measuring hash functions. It came
out of building something complex enough to have state to update.

---

## What privacy costs, in numbers

Part of the debate on central bank digital currencies discusses whether
cryptographic privacy is viable. Many things are said and few figures are
given. These are mine, measured:

| | |
|---|---|
| Start the system | **0.67 milliseconds** |
| Check that a transfer is valid | **4 milliseconds** |
| Generate the proof of that transfer | **620 milliseconds** |
| Space for a thousand transfers | **59 megabytes** |

Look at the ratio between the two in the middle: **checking costs half a
percent of proving**.

That is what makes the whole model viable. The heavy work falls on
whoever wants to prove something; whoever verifies it spends almost
nothing. A supervisor could check millions of operations a day on an
ordinary computer.

And the last number is a problem: **59 megabytes for every thousand
transfers**. A national system would do millions a day.

But **it is not the first one that would appear**, and an earlier version
of this text said it was.

⚠️ **And here the correction needs correcting.** For a time this text
explained that the limit that appeared first was another: each payment
generated a mark stored in a slot **computed from the mark itself**, and
by the same effect that makes two people in a classroom of thirty share a
birthday, at **sixty-five thousand payments** the coincidence was already
likely — leaving someone's payment blocked forever.

**That limit no longer exists: the path that produced it was withdrawn
entirely** (`AUDITORIA.md` §32 and §36), and today nothing generates those
marks. The payment is now protected by chaining the state: resending an
old operation presents an expired snapshot and is rejected.

**But it was not solved, it was avoided** — and it is worth saying it that
way: the new mechanism needs **someone to put the operations in an
order**, and that is exactly what a distributed system does not get for
free. Whoever spreads this across several nodes recovers the whole
problem.

And what, then, is the limit that bites first today? **It is not the
size**: it is that each proof is tied to the exact snapshot of the system
it saw when generated, so two people paying at the same time get in each
other's way. Measured: **between 1.5 and 1.9 operations per second**.
That is the real ceiling, and it is written down with its number.

---

## What it does not do

This is where many projects go quiet. I am going to do the opposite,
because I believe it is what makes the work useful.

**This is not decentralized.** It runs on a single computer. Whoever
operates it sees all the balances and can refuse to process operations.
Fixing it requires distributed consensus, which is an entire field of
computer science this project does not address.

**Nobody has audited it.** I wrote more than two hundred tests that try
to break it, and some succeeded. But I wrote the code and I wrote the
tests, and that is exactly the blindness an external audit exists to
correct.

**It has never moved real money.** It is not in production, nor close.

**It is not a blockchain.** There is no chain of blocks, no mining, no
tokens.

---

## Why I publish my mistakes

In the project's documentation there is a section with the mistakes I
made. Not the elegant summaries: the mistakes.

I compared speeds between two different versions of the program, and for
a time I believed one system was a hundred and thirty times faster than
another when the real difference was eleven.

I wrote two security rules as empty placeholders, intending to complete
them later. An empty rule **always holds**: the system seemed to check
that only two authorities can freeze an account, and it checked nothing.

Three times I wrote tests that failed for the wrong reason, giving me
unjustified confidence.

They are all published. For two reasons.

The first is practical: a project without documented errors usually means
that **nobody really looked**. If someone is going to trust this work,
they deserve to know how it was verified and where it failed.

The second is that mistakes are the most instructive thing there is. That
an empty constraint fails no test is a lesson you do not forget once you
have suffered it.

---

## What remains

For it to be usable infrastructure demands two things that do not depend
on writing more code.

**Distributed consensus**, so that there stops being one operator who
sees everything. It is a separate project, in another discipline, and
doing it badly is worse than not doing it: a defective consensus gives
the appearance of a guarantee without giving one.

**An external audit**, which is the only real condition. Without someone
from outside trying to break it with knowledge and ill intent, none of
this should touch real money.

---

## What does stand demonstrated

That the properties can be built at the same time.

Privacy against third parties. Verifiable compliance without master keys.
Conservation of money, impossible to circumvent. Supervision that works
without seeing the balances. And all of it **without depending on any
ceremony or any secret that someone had to destroy**.

It was not obvious before building it. Now it is measured, and anyone can
reproduce it on their computer with a single command.

That, and eight technical findings that were not documented anywhere, is
what this work contributes. Not infrastructure ready to use: **data on
what it costs to build it, and on what goes wrong while you try**.

---

## To look inside

**`https://github.com/atoranzo/Arqueo-open-conservation-proofs-for-closed-ledgers`**

Everything is published under a free license: the code, the measurements,
the technical documentation, the mistakes and a document meant for whoever
wants to break it, which includes a section with the points where I have
the least confidence.

It is reproduced with stable Rust and one command. Nothing I have told
here requires trusting me.

---

*Angel Toranzo Portela · 2026*
*License: MIT / Apache-2.0*
