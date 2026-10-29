# Five cryptographic backends: a comparison with real metrics

> **Español:** [`FIVE_BACKENDS.md`](./FIVE_BACKENDS.md) is the original of this page, in Spanish; it
> prevails wherever the two differ. This translation is of commit `7766efb` (audit entry §917) and
> was produced by the project's assistant, a Claude Code session, in audit entry §918 (`GENAI.md`):
> it is a generated artefact, not text written by the author. Line citations of the form `file:N`
> refer to the Spanish files.

All the numbers were measured by running the **same compliance
circuit**, on the same machine (WSL2 on a consumer laptop) and **all in
release mode**. There are no figures quoted from the literature.

To reproduce:

```bash
cargo test -p zk-core --release performance -- --nocapture
cargo test -p halo2-experiment --release real_proof -- --nocapture
cargo test -p stark-experiment --release real_proof -- --nocapture
cargo test -p plonk-experiment --release performance -- --nocapture
cargo test -p nova-experiment --release --features test-setup -- --nocapture
```

---

## Methodological note: a corrected error

A previous version compared Groth16 and Halo2 figures measured in
**debug** with STARK figures measured in **release**. On homogenizing,
the first two turned out to be between **11 and 14 times faster** than
they were credited with. The error is documented instead of corrected
in silence.

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
| **Proof size** | **192 bytes** | 4,096 bytes | 39,048 bytes ⚠️ | 1,008 bytes |
| **Quantum resistance** | No | No | **Yes** | No |
| **Tree depth** | 20 | 20 | 32 | 20 |

⚠️ §806: the STARK size cell said «36.7 KB». It is the blowup 16 row
with quadratic extension of section 4, and the same test measures it;
remeasured on 2026-10-14 on `b6491ef`, it gives 39,048 B (the command
and the detail, in section 4: the 36.7 KB were the measurement of its
tree, and §641 moved them). Only the size is corrected, which does not
depend on the machine: the generation and the verification of this
column are those of the table's measurement, and have not been measured
again under comparable conditions.

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

## Nova: a different paradigm, measured apart

The four previous backends produce a **monolithic proof** of the whole
circuit. Nova does something different: it **folds** a sequence of
steps and compresses at the end. It does not compete on the same axis,
so its numbers go apart.

| Phase | Measured cost |
|---|---|
| `PublicParams::setup` (once) | 4.02 s |
| **Per transaction (`prove_step`)** | **~250 ms, CONSTANT** |
| `RecursiveSNARK::verify` | 108 ms |
| `CompressedSNARK::prove` (closing) | 1.84 s |
| `CompressedSNARK::verify` | 50 ms |
| Constraints per step | 10,764 (primary) / 10,538 (secondary) |

**The figure that justifies the backend**: step 9 cost **0.77 times**
step 1. The cost does not grow with the length of the chain — which is
exactly the property that defines folding and that none of the other
four has.

The resulting profile —cheap during the day, expensive at closing—
matches that of a clearing house that settles during the working day
and closes at night.

### ⚠️ Honest nuances

**The overhead of folding is substantial.** 10,764 constraints per step
for a circuit that does **a single Poseidon hash**. Almost all of it is
the "verifier circuit" that Nova inserts in every step. Nova is not
free: it charges fixed machinery and only pays off with many steps.

**The ~250 ms are NOT comparable** with the 1.17 s of the double entry
in Groth16. The measured step is trivial; a step with real double entry
would be around 38,000 constraints. What is shown is that *the mechanism
works and its cost is constant*, not that Nova is faster here.

**Scope**: `nova-experiment` was closed at proof-of-concept level. It
implements neither the compliance circuit nor the double entry, unlike
the other four.

---

## The findings that only appear when measuring

### 1. PLONK-KZG is the slowest generator of the four

It is the most counter-intuitive result of the project. PLONK-KZG is
usually presented as the industry standard, and here it turns out **16
times slower than Groth16** in the compliance circuit and **22 times
slower** in the double-entry one.

**Important and honest nuance**: part of that difference may be due to
the *implementation* (`dusk-plonk` against arkworks, highly optimized)
and not to the scheme itself. These data do not allow separating both
effects. The defensible claim is: *this implementation of PLONK-KZG is
the slowest of the four measured*, not *PLONK-KZG is slow*.

It also contributes that its hash is more expensive: **997 gates** per
Poseidon hash of arity 2, against ~300 constraints in Groth16.

### 2. But its setup is the only truly reusable one

Groth16 requires a ceremony **for every circuit**. This project already
has two (solvency and double entry), so they would be two ceremonies.

PLONK-KZG uses a **universal** SRS: a single ceremony serves all
present and future circuits. The compilation per circuit (12.8 s /
48.5 s) is deterministic and **without secrets** — it is not a second
ceremony.

And there is a real public ceremony on BLS12-381, coordinated by Dusk,
with a conversion tool to the `PublicParameters` format. It is the only
one of the four options where there is a ceremony **already held** and
not only an implemented mechanism.

### 3. Without field extension, a STARK over Goldilocks has a 63-bit ceiling

The field is 64 bits and soundness cannot exceed it however many
queries are added. The "fast and compact" configuration one would
choose by default **is not comparable** with the ~128 bits of the other
three. The layer does not use it: its options carry quadratic
extension.

### 4. The gap between conjectured and provable security in STARK

| Configuration | Size | Generation | Conjectured | Provable |
|---|---|---|---|---|
| blowup 8, no extension | 27,794 B ⚠️ | 25 ms | 63 bits | 26 / 24 |
| blowup 8, quadratic ext. | 33,125 B ⚠️ | 28 ms | 95 bits | 26 / 47 |
| **blowup 16, quadratic ext.** | **39,048 B** ⚠️ | **39 ms** | **127 bits** | 29 / 63 |
| 120 queries, grinding 20, cubic ext. | 127,274 B ⚠️ | 48 ms | 128 bits | **128 / 128** |

The four rows are of the compliance circuit, with 32 queries except the
last one and without grinding except the last one. There, the 127
conjectured bits the ecosystem usually cites coexist with 29 (UDR) and
63 (LDR) provable. Closing the gap costs 127,274 B instead of 39,048
—⚠️ §806: it said «125.6 KB instead of 36.7»— and even so it is still
the fastest generator.

⚠️ **They are not the production figures.** The layer's options —42
queries, blowup 16, grinding 21 and quadratic extension— give, over its
hidden proofs, 127 conjectured bits —112.8-115.8 depending on the length
with the DEEP term, which the conjectured formula does not discount
(§708)— and, provable, 59 in UDR and 80-86 in LDR depending on the length
of the circuit, less in the age proof the higher its `m`. Tests pin it
since §697, and the table per circuit and per `m` is in
[`SECURITY.md`](./SECURITY_EN.md) §3.11. What it would cost to take those
proofs to 128 provable bits is not measured, nor is their level against
a quantum adversary. ⚠️ **Corrected in §698**: until then the table's
paragraph said «The 127 conjectured bits the ecosystem usually cites
coexist with 29-63 provable», without saying with which options, and
other documents cited it as if it were the layer.

Remeasured on 2026-10-07 (§698) with the same test
(`real_proof_end_to_end_with_metrics`): the same bits in the four rows,
and 27,794, 33,125, 39,048 and 127,274 B.

⚠️ §806: until here the table gave 27.7, 32.7, 36.7 and 125.6 KB, and this
paragraph went on «The sizes moved since the table's measurement —because
of the circuit or because of the winterfell version; which one has not
been looked for—, and the table is kept because it is the one of the
comparison between backends». Remeasured on 2026-10-14 on `b6491ef`, with
the test of `crates/stark-experiment/src/compliance_real_proof.rs`,

```bash
cargo test --release --locked -p stark-experiment --lib real_proof_end_to_end_with_metrics -- --nocapture
```

gives in three runs the same bytes —27,794, 33,125, 39,048 and 127,274 B,
those of §698— and the bits of the table, and the table now carries those
sizes, in bytes. The earlier one was not badly measured: the same command
on the tree of `9a1e2f1` (2026-07-27), the commit that brought the table,
gives in three runs 28,339, 33,535, 37,607 and 128,576 B, which in the KB
of 1,024 B the test prints are its 27.7, 32.7, 36.7 and 125.6 KB. The size
moved afterwards —three rows go down and one goes up, with the same bits—,
and §641 moved it (`2901c54`, RFC-0017), which took the circuit's range to
62 bits: with the same command, the tree of its parent, `7534380`, still
gives the sizes of the table, and its own, today's. Why that change moves
the sizes this way has not been studied. The times are not corrected: they
depend on the machine, and this one, with 4 CPUs and loaded with other
tasks, gave in the same three runs 31-40, 35-40, 47-56 and 74-78 ms.

### 5. Only two of six libraries defend themselves against unsafe use

This project has documented by hand, in every backend, that generating
parameters with a local `setup()` is not valid for production. Groth16,
Halo2, STARK and PLONK-KZG allow it and trust whoever uses it to notice.

**`nova-snark` prevents it in code.** Its error message is literal:

```text
HyperKZG::setup is disabled in production builds. Use
PublicParams::setup_with_ptau_dir ... with ptau files from a trusted
setup ceremony. For tests, enable the 'test-utils' feature.
```

And it offers the correct path already solved: `setup_with_ptau_dir`
consumes ptau files from **Perpetual Powers of Tau**, the existing
public ceremony.

In `crates/nova-experiment` the `test-setup` feature is deliberately
exposed on the command line, and not hidden in `dev-dependencies`, so
that whoever runs the tests sees that they are enabling something that
is not valid for production.

**And there is a second one.** On evaluating `risc0-zkvm` (see finding
7) the same pattern appears by another way: its receipt type includes a
`Fake` variant, *"without cryptographic integrity, used only for
development"*, and it offers the `disable-dev-mode` feature described
as *"disables dev mode so that proving and verifying cannot be faked.
Used to prevent a misplaced `RISC0_DEV_MODE` from breaking security in
production systems"*.

It names **the concrete scenario** —a forgotten environment variable—
which is exactly how these failures happen in practice.

| Library | Faced with proofs without cryptographic guarantee |
|---|---|
| `nova-snark` | **Prevents it** in production builds |
| `risc0-zkvm` | Allows it, but gives a feature to block it |
| The other four | Allow it in silence |

**Two of six actively defend themselves against unsafe use.** It is an
axis no performance table captures, and it distinguishes a library
meant for production from one meant to publish a paper.

### 6. AIR lacks copy constraints, and that forces a redesign

When porting the double entry to STARK a soundness hole appeared that
**does not exist in the other three paradigms**: nothing forces the two
climbs of the tree (old leaf and new leaf) to use the same siblings. A
prover could use different siblings in each one and fabricate a root
that does not correspond to the same position of the tree — and honest
witnesses would never reveal it.

The solution was a **lockstep** design (two lanes advancing level by
level with the sibling forced to be identical), verified in isolation
in `stark-experiment::dual_climb`.

In PLONK and Halo2, both Plonkish, reusing the same `Witness` is
enough: the copy constraints guarantee it. In Groth16, the same.

**Practical conclusion: porting a circuit from Plonkish to AIR is not
mechanical, not even when the logic is identical.**

---

### 7. A zkVM is not comparable on equal terms, and that is already a datum

**RISC Zero** was evaluated as a sixth paradigm. Its approach is the
opposite of the other five's: instead of writing circuits by hand, a
normal Rust program is compiled and the zkVM proves its execution.

The conceptual fit was good: it uses **STARK over Goldilocks**, the same
system and the same field as the chosen backend, so the comparison
would have been almost a controlled experiment —the only variable: how
the logic is written—.

And the path without a ceremony exists: `InnerReceipt::Succinct` proves
*"arbitrarily long zkVM computations with a single STARK"*. The Groth16
wrapper, which does require a ceremony, is only needed to verify
on-chain.

**But it does not meet the project's methodological criterion.**

The five backends are installed with `cargo add` and nothing else. That
criterion discarded `dusk-plonk 0.21` (it required nightly),
`halo2-lib` (unpinned git dependencies) and `plonk-core` (unpublished).

RISC Zero needs an **external toolchain** to compile the guest program
to RISC-V: its feature table states that `prove` is available on *"all
targets except rv32im"*, and the README refers to `cargo risczero`, a
separate installer.

**It is not a defect**: it is the inevitable price of compiling
arbitrary programs. But measuring it next to the other five would
falsify the comparison, so **it is documented instead of implemented**.

Figures that are recorded from the evaluation:

| | Own STARK backend | RISC Zero |
|---|---|---|
| Dependencies | **3** | **349** |
| Installation | `cargo add` | `cargo add` + external toolchain |
| Declared security | 127 conjectured bits | 98 conjectured bits |

The difference in dependencies —three against three hundred and
forty-nine— is the concrete measure of what generality costs, and the
reason why a zkVM contradicts this project's principle of minimalism.

## When to choose each one

**Groth16** if proof size rules (192 bytes) and a ceremony per circuit
can be held. It is also the fastest of the three curve-based ones.

**PLONK-KZG** if a reusable universal setup with a public ceremony
already held, small proofs (1 KB) and fast verification are wanted, and
a slow generator can be assumed.

**Halo2/IPA** if every ceremony is to be avoided while keeping small
proofs (4 KB). After correcting the methodology, it turns out to be a
narrower middle point than the previous comparison suggested.

**STARK** if total transparency, quantum resistance and generator
performance matter, and the proof size can be assumed.

**Nova / folding** if the volume is high and the closing is periodic:
the cost per transaction is constant and the cost of closing is
amortized among all of them. It does not replace the others — it
produces a folded state that has to be compressed with a SNARK, so it
needs them.

---

## On proof size in a banking context

The size does not block messaging: financial buses (ISO 20022 over
SWIFT, MQ, Kafka) routinely move payloads of hundreds of KB. The real
friction is **historical accumulation**: at millions of transactions,
storing **72,382-84,244 B** per send or claim proof —the band a test
pins for those two, hidden, since §538; it does not pin the other
proofs; before hiding they were 64.6 and 65.3 KB (§218), and never the
39,048 B of the comparison circuit— instead of 192 bytes changes the
size of the ledger, the synchronization times and cold validation by
two orders of magnitude.

⚠️ §806: it said «never the 36.7 KB of the comparison circuit», the size
of the table of section 4, which is remeasured there. The band still
stands: on 2026-10-14, on `b6491ef`, the test that pins it
(`la_cifra_publicada_sigue_siendo_la_medida`, in `crates/zk-ssl/src/metrics.rs`),
with
`cargo test --release --locked -p zk-ssl --lib la_cifra_publicada_sigue_siendo_la_medida -- --nocapture`,
measured in three runs sends of 79,339, 78,762 and 79,752 B and claims of
78,840, 79,737 and 78,618 B, within their bands; they change from one run
to the next because each run's salt changes the proof. The 64.6 and 65.3
KB are not measured again: they are of proofs without hiding, and today's
send and claim provers always hide.

---

## Honest limitations of this comparison

- **A single run per configuration, on a single machine.** No means
  and no confidence intervals. It serves to compare orders of
  magnitude, not as a rigorous benchmark.
- **The circuits are not identical.** Same logical design, but: the
  STARK tree has 32 levels against 20; the Poseidon parameters differ
  between backends (dusk's Hades, arkworks' Poseidon, halo2_gadgets'
  Poseidon); and PLONK's leaf is a hash of arity 4 while in the others
  they are two nested hashes.
- **The proofs are not interchangeable between backends.** Different
  finite fields and parameters: the Merkle trees and the nullifier
  spaces are incompatible. See
  `crates/settlement-prover/src/lib.rs`.
- **`plonk-experiment` depends on a release candidate**
  (`dusk-poseidon 0.42.0-rc.0`), the only version that compiles on
  stable Rust next to `dusk-plonk 0.22`. And `dusk-plonk` is MPL-2.0,
  not MIT/Apache like the rest of the project.
- **None of this has been audited by third parties.**
