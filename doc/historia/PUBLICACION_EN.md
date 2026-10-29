> **Document of the ZK-SSL stage (until September 2026).** It is kept as it was, as history:
> its statements in the present tense describe the project of that time, not today's. The current
> position is in [`README.md`](../../README_EN.md) and [`doc/USE_CASES.md`](../USE_CASES.md); what
> changed since then is in [`AUDITORIA.md`](../../AUDITORIA.md) (§448).

# Texts to accompany the publication

> **Español:** [`doc/historia/PUBLICACION.md`](./PUBLICACION.md) is the original of this page, in
> Spanish; it prevails wherever the two differ. This translation is of commit `7766efb` (audit entry
> §917) and was produced by the project's assistant, a Claude Code session, in audit entry §918
> (`GENAI.md`): it is a generated artefact, not text written by the author. Line citations of the
> form `file:N` refer to the Spanish files.

Four versions of the same message, depending on where it is published.
**Adapt them**: they are written to sound like you, not like an
announcement.

One rule that runs through all four: **what is distinctive about this
work is not the layer, it is the measurements and the findings**. Selling
the layer as infrastructure invites comparison with Zcash or Aztec, and
losing. Presenting it as the artefact on which the comparison was made is
exact and competes with nobody.

---

## 1. Technical forum or cryptography community

> I implemented the same settlement circuit in five proof systems
> —Groth16, Halo2/IPA, STARK/FRI, PLONK/KZG and Nova— to see what really
> changes when porting a complete application between paradigms, and not
> only a reference SHA-256.
>
> Some results I did not expect:
>
> - **AIR has no copy constraints.** On porting the state update to STARK
>   a hole appeared that does not exist in Plonkish: nothing forces the
>   two climbs of the tree to use the same siblings. It is silent —an
>   honest witness never reveals it— and it forces a lockstep pattern.
> - **PLONK-KZG turned out to be the slowest prover** of the four
>   curve-based ones, 16-22× more than Groth16. Part may be down to the
>   implementation and the data do not allow separating it.
> - **Goldilocks is too narrow for identities**: 64 bits are a collision
>   at 2³².
> - Only **two of six libraries prevent insecure use in code**.
>
> All measured on the same machine and in release, with the code to
> reproduce it. The artefact it was measured on is a complete settlement
> layer —privacy, selective disclosure, threshold issuance, freezing—
> with its limits declared in the header: it is a single node, and the
> operator sees all the balances.
>
> [link]

**Why it works here**: it opens with the method, gives concrete findings
someone can discuss, and declares the limitation without them having to
find it.

---

## 2. Professional network

> I have published the work of several months: a settlement layer with
> zero-knowledge proofs where transfers are private but regulatory
> compliance is provable, built without any trust ceremony.
>
> What I am most interested in sharing is not the layer but **the
> measurements**. I implemented the same circuit in five different proof
> paradigms to be able to compare with data instead of with literature.
> Verifying a settlement costs 0.5% of generating it; startup needs no
> key generation; and a thousand transfers accumulate 126.2 MiB of
> proofs.
>
> And there is a limit that bites first: the position of a nullifier is
> derived from the nullifier itself, so by the birthday paradox, at
> ~65,000 payments collisions are already likely, and the one affected
> cannot retry.
>
> What it does **not** do is documented too: it is a single node, the
> operator sees the balances, and nobody has audited it. That part takes
> up as much space as the achievements, on purpose.
>
> [link]

**Why it works here**: no unnecessary jargon, one memorable figure, and
honesty as a trait of the work rather than as an apology.

---

## 3. Direct email to a specific person

> Hi [name],
>
> I am writing to you because [concrete reason: you work on X / you wrote
> about Y].
>
> I have published a comparative work that may be useful to you: I
> implemented the same settlement circuit in five proof systems and
> measured the differences under identical conditions. The part I think
> contributes something is that the comparison was made on a **complete
> application**, not on reference circuits — and several of the findings
> only appear that way. For example, that AIR lacks copy constraints is
> not discovered by implementing SHA-256.
>
> I am not after anything in particular; if it is useful to you, good,
> and if you see something wrong, I would like to know. The audit
> document includes a section with the points where I have the least
> confidence.
>
> [link]

**Rules for this one**: personalize the reason for real —a generic email
shows—, ask for nothing, and be brief. If it does not fit on a phone
screen, it is too long.

---

## 4. One line, for when someone asks

> The same settlement circuit implemented in five proof systems,
> measured, with the findings and the limits documented.

---

## What NOT to say

**"Sovereign infrastructure for a new economic order"** — a practitioner
in the field stops reading at the first paragraph. It is the kind of
sentence that makes good work look unserious.

**"It solves the privacy problem in CBDC"** — it does not solve it. It
demonstrates that certain properties are buildable and measures what they
cost. The difference matters and it shows.

**Omitting that it is a single node** — it is discovered in five minutes,
and whoever discovers it on their own will also distrust what is true.

**Numbers without context** — "verify in 4 ms" invites the question
"compared to what, on which machine, how many runs?". Giving it with its
nuance from the start avoids that conversation.

---

## Before publishing

- [ ] Decide the repository's name.
- [ ] Check that `README.md` is the first thing seen and that the
      internal links work.
- [ ] Check that no keys, local paths or personal data remain in the
      code and the test files.
- [ ] Run the full suite from a clean clone.
- [ ] Date the publication: if it is later sent to a call, having the
      authorship dated helps.
