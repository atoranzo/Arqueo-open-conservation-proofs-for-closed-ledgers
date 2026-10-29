> **Document of the ZK-SSL stage (until September 2026).** It is kept as it was, as history:
> its statements in the present tense describe the project of that time, not today's. The current
> position is in [`README.md`](../../README_EN.md) and [`doc/USE_CASES.md`](../USE_CASES.md); what
> changed since then is in [`AUDITORIA.md`](../../AUDITORIA.md) (§448).

# Performance metrics — zk-ssl-real

> **Español:** [`doc/historia/PERFORMANCE.md`](./PERFORMANCE.md) is the original of this page, in
> Spanish; it prevails wherever the two differ. This translation is of commit `7766efb` (audit entry
> §917) and was produced by the project's assistant, a Claude Code session, in audit entry §918
> (`GENAI.md`): it is a generated artefact, not text written by the author. Line citations of the
> form `file:N` refer to the Spanish files.

> ⚠️ **SUPERSEDED DOCUMENT.** The figures in this file were measured in
> **debug** mode and are between 11 and 14 times worse than the real
> ones. The correct measurements, all in release and with the same
> methodology for the three backends, are in
> [`FIVE_BACKENDS.md`](../../FIVE_BACKENDS_EN.md).
>
> The document is kept instead of deleted to leave a record of the
> methodological error and of its correction.



All the numbers in this document come from real runs of `cargo test`,
not from estimates. Every time quoted here has a verifiable test run
behind it (see `test_output.txt` in the various rounds of this
conversation).

## ⚠️ Important methodological warning

These numbers were measured with `cargo test --workspace` in **parallel
mode** (the default behaviour), in a WSL2 environment on Windows with
shared resources. When several Groth16 tests run at once, each one tries
to use all the available CPU cores (via `rayon`), and they step on each
other — this **artificially inflates** the individual times above what
each operation would cost in isolation.

It has not yet been measured with `--test-threads=1` (serial execution)
to obtain a clean baseline without contention. The numbers below are,
therefore, **upper bounds under CPU contention**, not the best case.
Explicitly marked as pending in the final section.

## Observed results

| Operation | Circuit | Time (low contention) | Time (high contention) |
|---|---|---|---|
| Setup (`Groth16::Generator`) | `ComplianceCircuit` (stateless) | ~4.0 – 5.0 s | ~6.6 – 10.8 s |
| Generate proof (`Groth16::Prover`) | `ComplianceCircuit` (stateless) | ~0.78 – 0.94 s | ~3.5 – 4.3 s |
| Setup (`Groth16::Generator`) | `ComplianceCircuitWithState` (with Poseidon, 20-level tree) | ~5.4 – 5.5 s | ~13.2 – 14.3 s |
| Generate proof (`Groth16::Prover`) | `ComplianceCircuitWithState` (with Poseidon, 20-level tree) | ~4.9 – 5.1 s | ~10.7 – 11.7 s |

"Low contention" = measurements taken when that test was the only one or
almost the only one running at that moment within its binary. "High
contention" = measurements taken when 5-6 heavy Groth16 tests were
running at once (the typical case in
`zk-core::circuit_with_state::tests`).

## What is NOT measured yet

- **`verify()` / `verify_with_state()` in isolation.** The `ark-std`
  traces (`print-trace`) wrap `Groth16::Generator` (setup) and
  `Groth16::Prover` (proof generation), but not `SNARK::verify`. By the
  design of Groth16, verification should be fast (a constant number of
  bilinear pairing operations, independent of the circuit size) —
  probably milliseconds — but this is a theoretical expectation based on
  the scheme, not a number measured with a clock in this project yet.
- **Baseline without CPU contention** (`--test-threads=1`). Pending, as
  explained above.
- **Proof size in bytes.** Groth16 produces constant-size proofs (~200
  bytes typically for BLS12-381), but it has not been measured
  explicitly by serializing a real proof from this project.
- **Specific cost of Poseidon in isolation** (how much of the difference
  between `ComplianceCircuit` and `ComplianceCircuitWithState` is pure
  Poseidon, versus the rest of the Merkle tree constraints — the range,
  the nullifier, etc.).

## Honest reading of what we do know

- **Poseidon costs, clearly and measurably.** The jump from ~4-5s to
  ~5.4-5.5s in setup (low contention) and from ~0.8-0.9s to ~4.9-5.1s in
  proof generation confirms that binding the circuit to a 20-level
  Merkle tree with real Poseidon has a non-trivial cost, just as the BIS
  itself warned about the performance limitations of ZKPs in this kind
  of application.
- **The cost is NOT prohibitive** for a settlement use case (not
  high-frequency): a few seconds per transaction is acceptable for
  high-value payment compliance, which does not require thousands of
  operations per second as, for example, an instant retail payment
  system would.
- **The Poseidon parameter cache was the most important performance fix
  of the whole project so far** (see README): without it, each proof
  took several minutes instead of several seconds, from recomputing the
  MDS matrix dozens of times.

## Honest next step

Before quoting these numbers in any external document (preprint,
proposal to a funding programme, etc.), one would have to:
1. Measure with `--test-threads=1` to have a figure without contention.
2. Measure `verify()` explicitly with a real stopwatch, not assume its
   cost from the theory of the scheme.
3. Repeat the measurement several times and report mean/deviation, not a
   single data point per case — the very variability observed here (5.4s
   vs 14.3s for the same operation) shows that a single measurement is
   not enough to characterize the system's real performance.
