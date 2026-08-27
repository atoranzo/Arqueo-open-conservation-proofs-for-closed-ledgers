# ZK-SSL — a settlement layer that proves compliance without revealing the ledger

Rust. STARK proofs, no trusted setup. Payments are proved **on the payer's
machine**; the layer verifies and applies.

---

## What it is

A minimal ZK settlement layer with **two-phase payments**: the payer builds the
materials, generates the proof locally, and the layer applies it after
verifying. No trusted setup. Post-quantum-oriented: hash-based STARKs, and
hash-based signatures for the epoch heads.

## What it is not

Not a rollup. Not a retail CBDC. Not multi-node consensus. Not production
audited. **Not "no operator".**

## Residual trust — read this before anything else

Verification does not extinguish residual dependence; it makes it **measurable
and declarable**. What remains, stated plainly:

- **A single node, run by an operator.** The operator orders the operations and
  **can censor**. There is no distributed consensus, and this project does not
  claim one.
- **The operator sees balances.** Account views are the operator's by design;
  the holder has an authenticated view. Account indices are not enumerable;
  other third-party surfaces remain open (see Spanish README, entries 49–50).
- **A residual API surface.** The `send`/`claim` path still accepts a spend key.
  It carries no `#[deprecated]` attribute; the absence is documented in the
  source, at `crates/zk-ssl/src/two_phase.rs`.
- **One cryptographic dependency is unaudited**, and it is named in the repo.
- **No institutional affiliation**, and no external audit.

What the operator *cannot* do: change signing keys without a third party seeing
it. The node serves a **signed** epoch head; an independent witness verifies it
and pins the first key it sees. **The guarantee belongs to whoever watches,
not to whoever reads**: with no witness running, the sentence above protects
nobody.

Longer treatment: [`doc/preprints/ZK-SSL-residual-trust.md`](./doc/preprints/ZK-SSL-residual-trust.md),
and in Spanish the "Léelo antes que nada" section of [`README.md`](./README.md).

---

## Run it

This is not a fifteen-minute path — **it is about a second.** The fifteen
minutes are `git clone` and one release build.

```bash
git clone https://github.com/atoranzo/ZK-SSL-ZK-Sovereign-Settlement-Layer-
cd ZK-SSL-ZK-Sovereign-Settlement-Layer-
cargo build --release            # rustc stable; this is the slow part
```

**1 · A full payment, end to end, with real STARK proofs.**

```bash
./target/release/zk-ssl-cli simulate
```

No flags needed. It opens and funds two accounts by delegated issuance
(**two custodians required**), sends, and claims. Each phase prints the proof
digest, the ledger root before and after, and the transition chain. Every proof
is generated locally, from the payer's side.

**2 · The conformance contract — re-run the canonical scenario and compare it
field by field against the published vector.**

```bash
./target/release/zk-ssl-cli conformance --check spec/vectors/zkssl-0.3.json
```

You should see it finish with:

```
CONFORMIDAD: 6 entradas + cabeza + suministro, todo IDENTICO
```

**What you just verified:** the same scenario, re-executed from scratch on your
machine, reproduces the values that the published vector fixes — every log
entry, the epoch head and the supply. That is the contract a second
implementation would have to meet.

`zkssl/0.3` is the current wire version. Vectors for `0.1` and `0.2` are kept in
the tree under their own version and are **rejected on purpose** by
`conformance --check`, which exits non-zero and names both versions.

---

## Papers

Preprints, with their current DOI on Zenodo:

- **Comparative Implementation of a Zero-Knowledge Settlement Layer across Five
  Proof Systems: Design Findings and Measurements**
  [10.5281/zenodo.21736125](https://doi.org/10.5281/zenodo.21736125)
- **Provable Compliance without Full Ledger Disclosure**
  [10.5281/zenodo.21736082](https://doi.org/10.5281/zenodo.21736082)
- **From Institutional Trust to Verifiable Properties**
  [10.5281/zenodo.21905595](https://doi.org/10.5281/zenodo.21905595)

Further notes: [`doc/ZENODO.md`](./doc/ZENODO.md)

## License

Dual licensed: MIT **or** Apache-2.0, at your option.
See [`LICENSE-MIT`](./LICENSE-MIT) and [`LICENSE-APACHE`](./LICENSE-APACHE).

## Where to go next

- [`README.md`](./README.md) — the full Spanish README
- [`spec/RPC.md`](./spec/RPC.md) — the normative JSON-RPC specification
- [`ARQUITECTURA.md`](./ARQUITECTURA.md) — architecture, and how to verify it
- [`doc/CONFIANZA_RESIDUAL.md`](./doc/CONFIANZA_RESIDUAL.md) — residual trust, in depth
