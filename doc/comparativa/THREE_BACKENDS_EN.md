> **Comparison superseded** by [`FIVE_BACKENDS.md`](../../FIVE_BACKENDS_EN.md), which replaces it with
> the same methodology. It is kept as it was, as history (§448).

# Groth16 vs Halo2 vs STARK: comparison with real metrics

> **Español:** [`doc/comparativa/THREE_BACKENDS.md`](./THREE_BACKENDS.md) is the original of this
> page, in Spanish; it prevails wherever the two differ. This translation is of commit `7766efb`
> (audit entry §917) and was produced by the project's assistant, a Claude Code session, in audit
> entry §918 (`GENAI.md`): it is a generated artefact, not text written by the author. Line
> citations of the form `file:N` refer to the Spanish files.

All the numbers in this document were measured by running the **same
compliance circuit**, on the same machine (WSL2 on a consumer laptop)
and **all in release mode**. There are no figures quoted from the
literature nor estimates.

To reproduce:

```bash
cargo test -p zk-core --release performance -- --nocapture
cargo test -p halo2-experiment --release real_proof -- --nocapture
cargo test -p stark-experiment --release real_proof -- --nocapture
```

---

## Methodological note: why this version corrects the previous one

An earlier version of this document compared Groth16 and Halo2 figures
measured in **debug** with STARK figures measured in **release**. On
homogenizing the methodology, the first two backends turned out to be
between **11 and 14 times faster** than they were credited with:

| | Earlier figure (debug) | Real figure (release) |
|---|---|---|
| Groth16, generation | ~5.2 s | **0.42 s** |
| Halo2, generation | ~53.5 s | **4.86 s** |
| Halo2, verification | ~1.28 s | **0.09 s** |
| Halo2, setup | ~176 s | **16.3 s** |

STARK's advantage in generation is still real, but it is of **one order
of magnitude**, not two. The error is documented instead of being
corrected in silence.

---

## Comparison table

| | Groth16 (BLS12-381) | Halo2 / IPA (Pallas) | STARK / FRI (Goldilocks) |
|---|---|---|---|
| **Paradigm** | R1CS | Plonkish | AIR |
| **Trusted setup** | Yes, per circuit | No | No |
| **Setup** | 438 ms | 16.3 s | **none** |
| **Generation** | 422 ms | 4.86 s | **38 ms** |
| **Verification** | 5 ms | 91 ms | **1 ms** |
| **Proof size** | **192 bytes** | 4,096 bytes | 36.7 KB |
| **Quantum resistance** | No | No | **Yes** |
| **Tree depth** | 20 levels | 20 levels | 32 levels |

The STARK figures correspond to the configuration of **127 conjectured
bits** (blowup 16, quadratic extension), which is the one comparable with
the ~128 bits of Groth16 and Halo2.

### The three axes, in proportions

- **Size**: Groth16 wins by far. A STARK proof takes ~196 times more
  than a Groth16 one, and ~9 times more than a Halo2 one.
- **Generation speed**: STARK wins. It is ~11 times faster than Groth16
  and ~128 times faster than Halo2.
- **Startup cost**: STARK has none. Halo2 pays 16 seconds of parameters;
  Groth16 pays less time but demands a ceremony (see
  `crates/ceremony`).

---

## The nuance that changes the comparison: conjectured vs provable

Winterfell distinguishes two security estimates, and its documentation
warns that reaching the same level provably demands 2 to 3 times more
queries:

- **Conjectured**: the one usually quoted in the STARK ecosystem. It
  rests on a conjecture about Reed-Solomon decoding that is considered
  reasonable, but which is a conjecture.
- **Provable**: soundness with a proof, without that conjecture.

Measured on our circuit:

| Configuration | Size | Generation | Conjectured | Provable (udr/ldr) |
|---|---|---|---|---|
| blowup 8, no extension | 27.7 KB | 25 ms | 63 bits | 26 / 24 |
| blowup 8, quadratic ext. | 32.7 KB | 28 ms | 95 bits | 26 / 47 |
| **blowup 16, quadratic ext.** | **36.7 KB** | **38 ms** | **127 bits** | 29 / 63 |
| 120 queries, blowup 16, grinding 20, cubic ext. | 125.6 KB | 45 ms | 128 bits | **128 / 128** |

Three observations that do not appear in the usual promotional
materials:

1. **Without a field extension, the ceiling is 63 bits.** The Goldilocks
   field is 64 bits wide and soundness cannot exceed it however many
   queries are added. The "fast and compact" configuration one would
   choose by default is not comparable with Groth16 or Halo2.
2. **The gap between conjectured and provable is enormous** in the
   comfortable configurations: 127 conjectured bits coexist with 29-63
   provable.
3. **But closing that gap is affordable**: 128 provable bits cost
   125.6 KB and 45 ms — still ~9 times faster in generation than
   Groth16, without a ceremony and without conjectures.

---

## The cost of the accounting guarantee

The double-entry circuit (`circuit_double_entry`) proves the
conservation of money, not only the issuer's solvency. Measured in
Groth16:

| | Solvency | Double entry | Factor |
|---|---|---|---|
| Constraints | 9,934 | 27,562 | 2.8× |
| Setup | 438 ms | 1,123 ms | 2.6× |
| Generation | 422 ms | 1,170 ms | 2.8× |
| Verification | 5 ms | 5 ms | **1×** |
| Size | 192 bytes | 192 bytes | **1×** |

The cost grows almost proportionally to the constraints on the prover's
side, and **is nil on the verifier's side**: a Groth16 proof is always
the same size and verifies in the same time, whatever the circuit. In a
settlement network with many more verifiers than provers, that asymmetry
matters.

---

## When to choose each

**Groth16** if proof size rules above all (192 bytes, two orders of
magnitude less than any alternative) and a real MPC ceremony can be
held. The ceremony mechanism exists and is verified in
`crates/ceremony`; holding it with independent participants is still
pending.

**Halo2** if one wants to avoid the ceremony while keeping proofs small
(4 KB). It is the middle ground, although after the methodological
correction it turns out to be the slowest of the three both in setup and
in generation.

**STARK** if total transparency, quantum resistance and prover
performance matter, and the proof size can be accepted. The opposite
profile to Groth16's: large proofs, everything else fast, zero setup.

---

## On proof size in a banking context

Size does not block messaging: financial buses (ISO 20022 over SWIFT,
MQ, Kafka) routinely move payloads of hundreds of KB. The real friction
is **historical accumulation**: at millions of transactions, storing
**53.6-65.3 KB** per proof —what the circuits of this layer measure
(§218), not the 36.7 KB of the comparison circuit— instead of 192 bytes
changes the size of the persistent ledger, the synchronization times
between nodes and cold validation by two orders of magnitude.

That is the real axis of the decision, and it depends on the volume and
the retention policy of each deployment.

---

## Honest limitations of this comparison

- **A single run per configuration, on a single machine.** There are no
  means of repetitions or confidence intervals. These numbers serve to
  compare orders of magnitude, not as a rigorous benchmark.
- **The circuits are not identical**: same logical design, but the STARK
  tree has 32 levels versus 20 and uses Rescue instead of Poseidon. The
  STARK backend therefore does *more* hashing work, not less — its
  advantage in generation is measured at a disadvantage.
- **The double-entry circuit only exists in Groth16.** The Halo2 and
  STARK figures correspond to the solvency circuit.
- **Proofs are not interchangeable between backends**: each one operates
  over a different finite field, so the Merkle trees and the nullifier
  spaces are incompatible. See
  `crates/settlement-prover/src/lib.rs`.
- **None of this has been audited by third parties.**
