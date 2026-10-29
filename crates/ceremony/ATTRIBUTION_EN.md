# Attribution of the vendored code

> **Español:** [`crates/ceremony/ATTRIBUTION.md`](./ATTRIBUTION.md) is the original of this page, in
> Spanish; it prevails wherever the two differ. This translation is of commit `7766efb` (audit entry
> §917) and was produced by the project's assistant, a Claude Code session, in audit entry §918
> (`GENAI.md`): it is a generated artefact, not text written by the author. Line citations of the
> form `file:N` refer to the Spanish files.

This crate **is not original code of this project**. Its content in
`src/` comes from the crate [`penumbra-sdk-proof-setup`][crate] version
**2.1.1**, the work of **Penumbra Labs** ([penumbra-zone/penumbra][repo]),
published under the dual **MIT / Apache-2.0** license — the same one this
repository uses, which makes the integration legally clean.

Only the `single/` module has been vendored, which implements the
two-phase MPC ceremony (BGM17) for Groth16 on arkworks. `all.rs` and the
whole integration with Penumbra's SDK (`penumbra-sdk-dex`, `-governance`,
`-shielded-pool`, `-stake`), which are specific to its circuits, have
been discarded.

## Files included

| File | Content |
|---|---|
| `phase1.rs` | Powers of Tau (universal phase, reusable across circuits) |
| `phase2.rs` | Specialization to the concrete circuit |
| `dlog.rs` | Discrete-logarithm knowledge proofs (they validate each contribution) |
| `log.rs` | Transcript of contributions and their hashes |
| `group.rs` | Type aliases of the group — **the only point where the curve lives** |
| `parallel_utils.rs` | Parallelization utilities |
| `lib.rs` | Formerly `mod.rs`: public API and the `transition`/`combine` functions |

## Modifications made

The original module structure (`src/single/`) has been kept so that the
diff against upstream is minimal and auditable. The changes are
**exclusively of imports and of the curve alias**; no line of
cryptographic logic has been touched.

### 1. Change of curve (2 lines + mechanical renames)

The original crate is pinned to **BLS12-377** (via `decaf377::Bls12_377`).
Here it has been repointed to **BLS12-381**, the curve of `zk-core`:

| File | Change |
|---|---|
| `single/group.rs` | `use decaf377::Bls12_377;` → `use ark_bls12_381::Bls12_381 as Curve;` |
| `single/mod.rs` | idem |
| all | mechanical rename `Bls12_377` → `Curve` |

The identifier was renamed to `Curve` instead of leaving the name
`Bls12_377` pointing at another curve — that would have been a trap for
anyone reading the code afterwards.

### 2. Imports of traits that `decaf377` provided (4 lines)

On replacing `decaf377` by pure `ark_bls12_381`, several traits that were
in scope indirectly were lost. They had to be declared:

| File | Added | Reason |
|---|---|---|
| `single/group.rs` | `use ark_ff::PrimeField;` | `F::from_le_bytes_mod_order` |
| `single/dlog.rs` | `use ark_ff::UniformRand;` | `F::rand` |
| `single/phase1.rs` | `UniformRand` to the existing import | `F::rand` |
| `single/phase2.rs` | `Field`, `UniformRand` to the existing import | `.inverse()`, `F::rand` |

**Why these changes are safe**: importing a trait cannot alter the
semantics of the code. There is no inherent method competing —that is
why compilation failed—, so the only candidate implementation is the
trait's. Either it compiles with the correct one, or it does not
compile; no silent failure is possible.

### 3. New files, this project's own

- `src/lib.rs`: only the attribution header and the module declarations.
- `Cargo.toml`: dependencies equivalent to the original's, without the
  Penumbra SDK crates.
- This `ATTRIBUTION.md`.

## Verification: what evidence supports this adaptation

The **34 tests of the original code pass on BLS12-381**, and among them
are the negative tests that check the soundness properties of the
ceremony, not only the happy path:

- `test_changing_alpha_makes_crs_invalid`
- `test_changing_beta_makes_crs_invalid`
- `test_changing_delta_makes_crs_invalid`
- `test_setting_zero_elements_makes_crs_invalid`
- `test_not_inverting_delta_makes_crs_invalid`
- `test_contribution_is_not_linked_to_itself`
- `test_bad_statement_makes_proof_fail`
- `test_can_generate_keys_through_ceremony` (complete ceremony → verified
  Groth16 proof)

To reproduce: `cargo test -p ceremony --release`

## ⚠️ Warnings that must be read before using this

1. **The adaptation is not audited.** Penumbra exercised its version on
   BLS12-377 in production. This version on BLS12-381 is an adaptation
   by this project, and the tests of the previous point are the only
   existing verification. That they pass is strong evidence, not an
   audit.

2. **BLS12-377 and BLS12-381 are not equivalent.** The first has a
   highly 2-adic base field and the second does not; only the scalar
   field is in both. The ceremony's FFTs operate on the scalar field,
   and the tests confirm empirically that the change breaks nothing —
   but it is worth knowing that the difference exists.

3. **`combine` builds the verification key with γ = 1**
   (`gamma_g2: G2::generator()`). It is a design decision of the
   original code, valid but not the textbook formulation of Groth16.
   The resulting keys are coherent with themselves; it is worth knowing
   before trying to interoperate with other implementations.

4. **An MPC ceremony is only secure if the participants are real and
   independent.** Running the contributions on the same machine and in
   the same process —as the tests do— shows that the *mechanism* works,
   **not** that there is a security guarantee: if all the randomness is
   generated by the same process, there is no independent honest party.
   A real ceremony requires distinct participants, on distinct machines,
   publishing the transcript so that anyone can verify it.

[crate]: https://crates.io/crates/penumbra-sdk-proof-setup
[repo]: https://github.com/penumbra-zone/penumbra
