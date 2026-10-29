> **Comparison superseded** by [`FIVE_BACKENDS.md`](../../FIVE_BACKENDS_EN.md), which replaces it with
> the same methodology. It is kept as it was, as history (§448).

# Groth16 vs. Halo2 — final comparison, with real data

> **Español:** [`doc/comparativa/GROTH16_VS_HALO2.md`](./GROTH16_VS_HALO2.md) is the original of
> this page, in Spanish; it prevails wherever the two differ. This translation is of commit
> `7766efb` (audit entry §917) and was produced by the project's assistant, a Claude Code session,
> in audit entry §918 (`GENAI.md`): it is a generated artefact, not text written by the author. Line
> citations of the form `file:N` refer to the Spanish files.

This document compares the two complete and verified cryptographic
engines of this project: `zk-core` (Arkworks/Groth16) and
`halo2-experiment` (Halo2/IPA). Both implement the same logic
(compliance with range checks + 20-level Merkle tree + nullifier + ISO
20022 bridge), verified end to end with real cryptographic proofs, not
only satisfiability.

## Comparison table

| Aspect | Groth16 (`zk-core`) | Halo2 (`halo2-experiment`) |
|---|---|---|
| Trusted setup | **Yes, per circuit.** A single participant in this project — a real and unresolved security limitation (see the README of `zk-core`). | **No.** Deterministic setup via IPA; anyone can regenerate it and check that it matches. |
| Setup time | Seconds (varies with CPU contention, see `PERFORMANCE.md`). | **176 s** (`k=15`) — much more expensive, but paid ONCE for any circuit of that size. |
| `keygen` | Included in the setup. | `keygen_vk`: 20 s. `keygen_pk`: 15 s. |
| Generate a proof | A few seconds (stateful circuit, with CPU contention). | **53.5 s** — noticeably slower. |
| Verify a proof | Fast (not measured separately, but theoretically sub-second). | **1.28 s** — fast in absolute terms, somewhat slower than Groth16. |
| Proof size | ~200 bytes (typical of Groth16). | **4,096 bytes** — ~20x heavier. |
| Maturity of the circuit ecosystem | Arkworks, widely used in academic research. `ark-groth16` officially declares itself an "academic prototype, not ready for production". | Halo2 (zcash/halo2) is in real production (Zcash Orchard) and audited. |
| MPC ceremony tools | Investigated twice (`ark-marlin`, `celo-org/snark-setup`); both turned out abandoned/incompatible with the current generation of the ecosystem — confirmed dead end. | Not applicable — no ceremony needed. |
| Complexity of the circuit code | Medium (R1CS + Arkworks gadgets). | High (Plonkish architecture: custom gates, fixed/advice columns, selectors, regions) — more control, more error surface. |
| Nullifier persistence | ✅ `sled`, verified. | ✅ `sled`, verified (same design, adapted to `ff::PrimeField`). |

## Honest reading

**Neither of the two engines "wins" in absolute terms** — it is the
classic trade-off, already known in the SNARK literature, now confirmed
with our own data:

- **Groth16 is the right option if**: performance (fast and small
  proofs) matters more than removing the trusted setup, and one is
  willing to take on the security risk of a single-participant ceremony
  until it is resolved properly (see the gap documented in the README of
  `zk-core`).
- **Halo2 is the right option if**: removing the trusted setup is a
  non-negotiable requirement (for example, for credibility before a
  regulator who specifically objects to that limitation), and it can be
  tolerated that proofs are ~10 times slower to generate and ~20 times
  heavier.

## What this comparison does NOT change

Neither of the two resolves, on its own, the other gaps already
documented in both projects: distributed coordination of the nullifier
registry between several validators, a complete ISO 20022 parser (both
use the same simplified subset), nor an external audit. This comparison
is exclusively about the cryptographic proof engine, not about the
project's readiness for production in general.

## Governance and institutional life cycle (beyond raw performance)

This section answers a different question from the one above: not
"which is faster today?", but "which is more agile to maintain in a real
institution, over years, when the compliance rules change?".

### Why verification time is NOT "milliseconds in both"

It is a common simplification in the general SNARK literature, but **our
own data contradict it for Halo2/IPA**: we measured `verify_proof` at
**1.28 seconds**, not milliseconds — more than a thousand times slower
than the typical expectation.

The reason is technical, not a flaw of our implementation: verification
in **IPA** (the scheme `zcash/halo2` uses, chosen precisely so as not to
need a trusted setup) scales **approximately linearly** with the circuit
size. Verification in **KZG** (the scheme Groth16 uses, and which a
Halo2/PLONK variant could also use) is almost constant. It is the same
"no ceremony ↔ slower" trade-off we already saw in proof generation, but
showing up in verification too — a nuance that is lost if one assumes
"both verify in milliseconds" without further ado. If in the future the
PSE fork with KZG were chosen instead of IPA, verification would probably
be faster, but that would reintroduce the need for a universal setup —
there is no combination that gives the best of both worlds for free.

### "Circuit changes are instantaneous" — true, but not entirely

Without an MPC ceremony in between, a circuit change in Halo2 is much
faster than in Groth16 — but **not instantaneous**. Our own data:
`keygen_vk` (20 s) + `keygen_pk` (15 s) ≈ **35 seconds** after any
circuit change. It is still an enormous difference against the weeks or
months of coordinating a ceremony — "minutes, not months" is the honest
comparison, not "instantaneous".

### What removing the ceremony does NOT remove: governance friction

This is the most important point to add to the raw performance analysis.
Removing the cryptographic ceremony **does not remove the need for
review and audit** of a new circuit before trusting it in production. If
a bank changes a Basel III/IV rule, that new circuit needs to go through
change control, security review and probably regulatory approval —
whether or not there is an MPC ceremony in between. For a real
institution, it is reasonable to expect **that governance process to be
the dominant bottleneck**, not the cryptography itself. Halo2's
"infinite temporal agility" is real at the level of technical
infrastructure, but it does not by itself resolve the organizational
friction that surrounds any change of compliance logic in a regulated
environment.

### Which concrete implementation of Groth16 is being compared

"Groth16 has mature tooling" is true for the **circom/snarkjs**
ecosystem (the one Tornado Cash and Zcash Sprout/Sapling use, with real
public ceremonies such as Perpetual Powers of Tau). It is more debatable
for **Arkworks**, which is what this project uses: the `ark-groth16`
repository itself declares itself an *"academic prototype, not ready for
production"*, and the two MPC ceremony tools we investigated for this
specific ecosystem (`ark-marlin`, `celo-org/snark-setup`) turned out
abandoned and incompatible with the current generation of Arkworks (see
the investigation documented earlier in this conversation). The maturity
of "Groth16" depends a lot on which concrete implementation is evaluated
— it is not a monolithic block.

## Practical recommendation

If the goal is to continue with **a single** engine from here on (so as
not to maintain two code bases in parallel), the reasonable choice
depends on the audience:

- For a **technical demonstration or academic preprint** aimed at
  research communities: Halo2, because it removes the easiest objection
  to raise ("and the trusted setup?").
- For a **compliance-as-a-service prototype** where response latency
  matters (recalling the conversation about monetization): Groth16,
  because generating proofs 10 times faster is a real product advantage,
  and the trusted setup can be resolved later with a serious MPC ceremony
  when there are resources for it.

There is no single correct answer — it is a product decision, not a
technical limitation left to resolve.

## Methodology of this comparison (so that it is verifiable, not only asserted)

All the Halo2 numbers come from a single real run measured with
`Instant::now()` in `compliance_real_proof.rs` (`k=15`, with no relevant
CPU contention as it ran alone). The Groth16 numbers come from the
`cargo test` runs documented in `PERFORMANCE.md`, with the warning noted
there about CPU contention in parallel. **Before quoting this comparison
in any external document**, it would be necessary to repeat both
measurements under the same controlled conditions (same hardware, no
contention, several repetitions with mean/deviation) — this is an honest
order-of-magnitude comparison, not a rigorous benchmark.
