# halo2-experiment

> **Español:** [`crates/halo2-experiment/README.md`](./README.md) is the original of this page, in
> Spanish; it prevails wherever the two differ. This translation is of commit `7766efb` (audit entry
> §917) and was produced by the project's assistant, a Claude Code session, in audit entry §918
> (`GENAI.md`): it is a generated artefact, not text written by the author. Line citations of the
> form `file:N` refer to the Spanish files.

An isolated crate, separate from the rest of the workspace (`zk-core`,
`iso-bridge`), to try whether Halo2 is a viable path as a replacement for
Groth16 — without risking any of the already verified code.

## State

**Not compiled yet.** It is the piece of greatest uncertainty in the whole
project — more than `poseidon_hash.rs` was at the time. It expects several
rounds of correction.

## What it does

`SquareCircuit`: proves knowledge of a private value `x` such that
`x * x = y`, with `y` public — the minimal canonical Halo2 example,
without touching Poseidon, Merkle trees or nullifiers yet.

The tests use `MockProver` (a satisfiability check, not generation of
real proofs) — deliberately, to isolate the first possible point of
failure (the definition of the circuit) from the second (the complete
prove/verify pipeline with IPA parameters), which is even more uncertain
and will be tackled in a later round if this compiles.

## How to compile and test

```bash
cargo test -p halo2-experiment -- --nocapture
```

If it fails, paste the full error — the first suspicion is the exact
method names of the `ConstraintSystem`/`Layouter`/`Region` API, which I
have not been able to verify without compiling.
