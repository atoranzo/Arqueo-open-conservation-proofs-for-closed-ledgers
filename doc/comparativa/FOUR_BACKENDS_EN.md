> **Comparison superseded** by [`FIVE_BACKENDS.md`](../../FIVE_BACKENDS_EN.md), which replaces it with
> the same methodology. It is kept as it was, as history (§448).

# Four cryptographic backends: comparison with real metrics

> **Español:** [`doc/comparativa/FOUR_BACKENDS.md`](./FOUR_BACKENDS.md) is the original of this
> page, in Spanish; it prevails wherever the two differ. This translation is of commit `7766efb`
> (audit entry §917) and was produced by the project's assistant, a Claude Code session, in audit
> entry §918 (`GENAI.md`): it is a generated artefact, not text written by the author. Line
> citations of the form `file:N` refer to the Spanish files.

All the numbers were measured by running the **same compliance
circuit**, on the same machine (WSL2 on a consumer laptop) and **all in
release mode**. There are no figures quoted from the literature.

To reproduce:

```bash
cargo test -p zk-core --release performance -- --nocapture
cargo test -p halo2-experiment --release real_proof -- --nocapture
cargo test -p stark-experiment --release real_proof -- --nocapture
cargo test -p plonk-experiment --release performance -- --nocapture
```

---

## Methodological note: a corrected error

An earlier version compared Groth16 and Halo2 figures measured in
**debug** with STARK figures measured in **release**. On homogenizing,
the first two turned out to be between **11 and 14 times faster** than
they were credited with. The error is documented instead of being
corrected in silence.

---

## Comparison table (compliance circuit)

| | Groth16 | Halo2 / IPA | STARK / FRI | PLONK / KZG |
|---|---|---|---|---|
| **Paradigm** | R1CS | Plonkish | AIR | Plonkish |
| **Curve / field** | BLS12-381 | Pallas | Goldilocks | BLS12-381 |
| **Trusted setup** | Per circuit | No | No | **Universal** |
| **Setup / SRS** | 438 ms | 16.3 s | none | 26.3 s (reusable) |
| **Compilation per circuit** | (included above) | — | — | 12.8 s |
| **Generation** | **0.42 s** | 4.86 s | **0.039 s** | 6.85 s |
| **Verification** | 5 ms | 91 ms | **1 ms** | 8 ms |
| **Proof size** | **192 bytes** | 4,096 bytes | 36.7 KB | 1,008 bytes |
| **Quantum resistance** | No | No | **Yes** | No |
| **Tree depth** | 20 | 20 | 32 | 20 |

## Double-entry circuit

| | Groth16 | STARK | PLONK / KZG |
|---|---|---|---|
| Size | 27,562 constraints | 113 constraints × 41 columns × 1024 rows | 84,801 gates |
| Setup / SRS | 1.12 s | none | 54.7 s |
| Compilation | — | — | 48.5 s |
| Generation | **1.17 s** | — | 26.4 s |
| Verification | 5 ms | — | 10 ms |
| Proof size | **192 bytes** | — | 1,008 bytes |

(Halo2 has the double-entry circuit implemented and verified with
`MockProver`, but not measured with real proofs.)

---

## The findings that only appear by measuring

### 1. PLONK-KZG is the slowest prover of the four

It is the most counterintuitive result of the project. PLONK-KZG is
usually presented as the industry standard, and here it turns out **16
times slower than Groth16** on the compliance circuit and **22 times
slower** on the double-entry one.

**Important and honest nuance**: part of that difference may be due to
the *implementation* (`dusk-plonk` versus arkworks, highly optimized)
and not to the scheme itself. These data do not allow separating the two
effects. The defensible claim is: *this implementation of PLONK-KZG is
the slowest of the four measured*, not *PLONK-KZG is slow*.

It also contributes that its hash is more expensive: **997 gates** per
arity-2 Poseidon hash, versus ~300 constraints in Groth16.

### 2. But its setup is the only truly reusable one

Groth16 demands a ceremony **for every circuit**. This project already
has two (solvency and double entry), so that would be two ceremonies.

PLONK-KZG uses a **universal** SRS: a single ceremony serves all present
and future circuits. The compilation per circuit (12.8 s / 48.5 s) is
deterministic and **without secrets** — it is not a second ceremony.

And a real public ceremony exists over BLS12-381, coordinated by Dusk,
with a conversion tool to the `PublicParameters` format. It is the only
one of the four options where there is a ceremony **already held** and
not only a mechanism implemented.

### 3. Without a field extension, a STARK over Goldilocks has a 63-bit ceiling

The field is 64 bits wide and soundness cannot exceed it however many
queries are added. The "fast and compact" configuration one would choose
by default **is not comparable** with the ~128 bits of the other three.

### 4. The gap between conjectured and provable security in STARK

| Configuration | Size | Generation | Conjectured | Provable |
|---|---|---|---|---|
| blowup 8, no extension | 27.7 KB | 25 ms | 63 bits | 26 / 24 |
| blowup 8, quadratic ext. | 32.7 KB | 28 ms | 95 bits | 26 / 47 |
| **blowup 16, quadratic ext.** | **36.7 KB** | **39 ms** | **127 bits** | 29 / 63 |
| 120 queries, grinding 20, cubic ext. | 125.6 KB | 48 ms | 128 bits | **128 / 128** |

The 127 conjectured bits the ecosystem usually quotes coexist with 29-63
provable. Closing the gap costs 125.6 KB instead of 36.7 — and even so it
is still the fastest prover.

### 5. AIR lacks copy constraints, and that forces a redesign

On porting the double entry to STARK a soundness hole appeared that
**does not exist in the other three paradigms**: nothing forces the two
climbs of the tree (old leaf and new leaf) to use the same siblings. A
prover could use different siblings in each and fabricate a root that
does not correspond to the same position of the tree — and honest
witnesses would never reveal it.

The solution was a **lockstep** design (two lanes advancing level by
level with the sibling forced to be identical), verified in isolation in
`stark-experiment::dual_climb`.

In PLONK and Halo2, both Plonkish, it is enough to reuse the same
`Witness`: the copy constraints guarantee it. In Groth16, the same.

**Practical conclusion: porting a circuit from Plonkish to AIR is not
mechanical, not even when the logic is identical.**

---

## When to choose each

**Groth16** if proof size rules (192 bytes) and a ceremony per circuit
can be held. It is also the fastest of the three curve-based ones.

**PLONK-KZG** if one wants a reusable universal setup with a public
ceremony already held, small proofs (1 KB) and fast verification, and a
slow prover can be accepted.

**Halo2/IPA** if one wants to avoid any ceremony while keeping proofs
small (4 KB). After correcting the methodology, it turns out to be a
narrower middle ground than the earlier comparison suggested.

**STARK** if total transparency, quantum resistance and prover
performance matter, and the proof size can be accepted.

---

## On proof size in a banking context

Size does not block messaging: financial buses (ISO 20022 over SWIFT,
MQ, Kafka) routinely move payloads of hundreds of KB. The real friction
is **historical accumulation**: at millions of transactions, storing
**53.6-65.3 KB** per proof —what the circuits of this layer measure
(§218), not the 36.7 KB of the comparison circuit— instead of 192 bytes
changes the size of the ledger, the synchronization times and cold
validation by two orders of magnitude.

---

## Honest limitations of this comparison

- **A single run per configuration, on a single machine.** No means or
  confidence intervals. It serves to compare orders of magnitude, not as
  a rigorous benchmark.
- **The circuits are not identical.** Same logical design, but: the
  STARK tree has 32 levels versus 20; the Poseidon parameters differ
  between backends (dusk's Hades, arkworks' Poseidon, halo2_gadgets'
  Poseidon); and PLONK's leaf is an arity-4 hash while in the others it
  is two nested hashes.
- **Proofs are not interchangeable between backends.** Different finite
  fields and parameters: the Merkle trees and the nullifier spaces are
  incompatible. See
  `crates/settlement-prover/src/lib.rs`.
- **`plonk-experiment` depends on a release candidate**
  (`dusk-poseidon 0.42.0-rc.0`), the only version that compiles on
  stable Rust together with `dusk-plonk 0.22`. And `dusk-plonk` is
  MPL-2.0, not MIT/Apache like the rest of the project.
- **None of this has been audited by third parties.**
