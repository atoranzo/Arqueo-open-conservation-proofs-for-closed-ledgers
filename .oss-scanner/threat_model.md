<!--
Modelo de amenazas para el OSS Scanner de Anthropic (`anthropics/oss-scanner`,
`projects/arqueo`). El escáner lo lee antes de empezar; va en inglés porque sus
informes lo están. Lo que dice sale de `SECURITY.md`, y si discrepan manda
`SECURITY.md`, y sobre los dos, el código. Lo trajo el §800 de `AUDITORIA.md`.
-->
# Threat model — Arqueo

Arqueo is a **research prototype**, not a product: it handles no real money,
nothing in it has been audited by a third party, and its docs are mostly in
Spanish (English mirrors: `README_EN.md`, `PAPER_EN.md`, `doc/KIT_EN.md`). The
security policy is [`SECURITY.md`](../SECURITY.md); when this file and that one
disagree, `SECURITY.md` wins, and the code wins over both. `§N` references point
to entries of [`AUDITORIA.md`](../AUDITORIA.md), the project's chronological log.

## What this project does and where untrusted input enters

A closed ledger run by **one operator on one node**. Every state transition
(send, claim, refund, issuance, freeze, governance…) carries a STARK proof built
on a fork of winterfell 0.13.1, so that the node accepts only transitions that
conserve money and are authorised by the spending key. The node signs its epoch
heads with XMSS; third parties check portable evidence (signed heads,
cosignatures, receipts, anchor notes) **without the node** using an independent
verifier kit.

Untrusted input enters here, most exposed first:

1. **The node's JSON-RPC port** (`crates/zk-ssl-node`, axum; listener in
   `src/main.rs`, default `127.0.0.1:8545`). Every request body is
   attacker-controlled: operations with their proofs (`zkssl_applySend`,
   `zkssl_applyClaim`, batches through `zkssl_applyMany`, …) and public queries.
   Methods: `spec/RPC.md` and `spec/openrpc.json`. The `dev_*` methods exist only
   with a compile-time feature **and** the `--dev` flag.
2. **The verifier kit** (`crates/zk-ssl-verify`, binary `zk-ssl-verify`,
   released as `arqueo-verify-v*`). It reads one JSON evidence package
   (format, check order, rejection catalogue and exit codes: `spec/PAQUETE.md`)
   that **anyone may have assembled, including a lying operator**. Its job is to
   reject everything that is not proven. It depends on `zk-ssl-air` (the AIRs
   it verifies), `zk-ssl-hash` (native hash, Merkle roots, digests),
   `zk-ssl-medio` (RFC 6962 SHA-256 tree and ML-DSA-44 checkpoint notes) and
   `xmss`; it deliberately does **not** link the prover, the node or the layer.
3. **Proof bytes**, deserialised and verified by the forked
   `crates/winter-air`, `crates/winter-prover` and `crates/winter-verifier`
   (they replace the crates.io ones through `[patch.crates-io]` in the root
   `Cargo.toml`; the fork adds hiding: random rows and a blinded quotient).
4. **The wire format** (`crates/zk-ssl-wire`): DTOs with `deny_unknown_fields`,
   canonical hex and fixed-width digests, shared by the node and the SDK.
5. **On-disk state** the layer reads back (`crates/zk-ssl`): the ledger (sled),
   encrypted snapshots, and keystores (Argon2id-derived keys, §702). Several
   invariants are re-checked on open (conservation, stored roots; §379-§394).
6. **The XMSS index guardian** (`crates/zk-ssl-guardian`): persists the
   one-time-signature counter with `fsync` and an exclusive lock before each
   signature (§709). Two messages signed with the same leaf compromise the key.
7. **The CLI and the SDK** (`crates/zk-ssl-cli`, `crates/zk-ssl-sdk`): files
   and responses on the client side.

## Components that matter most / least

- **Matter most (soundness and the verifier):** `zk-ssl-air`,
  `zk-ssl-verify`, the `winter-*` fork, `zk-ssl-hash`, `zk-ssl-medio`;
  `stark-experiment`, which despite its name holds the provers and traces that
  `zk-ssl`, the node, the CLI and the SDK use, and with it `settlement-prover`;
  and `zk-ssl` (the layer: conservation, replay protection by proof digest
  §654/§659, expiry of pending sends §178-§181).
- **Matter next:** `zk-ssl-node` (RPC handling and its parsing),
  `zk-ssl-guardian`, `zk-ssl-wire`.
- **Matter less:** `zk-ssl-cli`, `zk-ssl-sdk` (client tools, local input).
- **Research benchmarks, out of scope:** `zk-core` (arkworks Groth16),
  `ceremony`, `halo2-experiment`, `plonk-experiment`, `nova-experiment`,
  `settlement-layer`, `iso-bridge`. None of them is a dependency of the node,
  the layer, the verifier kit or the CLI; they exist to compare proof systems.
- **Not attack surface:** `tools/` (gates, benches, and `tools/segunda/`, a
  second implementation in Python used as an independent judge), `doc/`,
  `spec/` (but `spec/vectors/` is a good source of valid and rejected inputs).

## How to exercise it

The image already holds a release build of the whole workspace and every test
binary, with the same setting (`CARGO_PROFILE_RELEASE_DEBUG=true`, release with
debug info) in its environment. Running one crate with `-p` still recompiles a
few dependencies, because features unify differently than in the workspace
(about a minute for `zk-ssl-wire` on four cores):

- `cargo test --release --locked -p <crate>` for one crate. The fast ones:
  `zk-ssl-verify`, `zk-ssl-wire`, `zk-ssl-hash`, `zk-ssl-air`, `zk-ssl-medio`,
  `zk-ssl-guardian`, `zk-ssl-cli`. `zk-ssl`, `stark-experiment` and
  `zk-ssl-node` prove many STARKs and take minutes on two cores. Avoid
  `zk-core`, `halo2-experiment` and `plonk-experiment` (tens of minutes).
- `bash tools/canon.sh --sello`: the project's own gate (all fast crates plus
  the checks in `tools/`). Never `--largo` or `--completo`.
- Conformance vectors:
  `cargo run --release --locked -p zk-ssl-cli -- conformance --check spec/vectors/zkssl-0.4.json`.
- The verifier kit on a package: `target/release/zk-ssl-verify <package.json>`
  (green: exit 0; first named failure `ROJO: …`: exit 1; usage: exit 2).
- A node: `target/release/zk-ssl-node --help`; it serves JSON-RPC on
  `127.0.0.1:8545`. `crates/zk-ssl-sdk/examples/e2e.rs` drives one end to end,
  and the `tools/banco_*.sh` scripts start nodes and play a lying operator.

## How you rate severity

Critical is what `SECURITY.md` §5 calls a soundness failure («fallo de
solidez»): create balance, spend without authorisation, or make the verifier
accept an invalid transition. The grades below it were proposed with this file
(§800) and are the author's to revise:

- **Critical — soundness.** Anything that makes the node, the layer or the
  verifier kit **accept what the published rules reject**: creating balance or
  supply, spending or moving funds without the spending key, applying the same
  proof or receipt twice, refunding someone else's pending send, or making
  `zk-ssl-verify` print green for a package, head, cosignature, receipt or
  anchor note that does not hold. A concrete under-constrained witness that a
  verifier accepts is critical even though the AIRs are known to lack a formal
  specification. Also critical: a proof that publishes a literal witness value
  (spending key, balance, salt) under `zkssl/0.4`, and any path that signs two
  messages with the same XMSS leaf.
- **High.** A single request, package or proof that crashes, aborts or hangs
  the node or the verifier kit (`SECURITY.md` §3.7 records this class);
  loading tampered on-disk state without the check-on-open noticing; secret
  material written in clear where the code means to encrypt it or wipe it.
- **Medium.** Information reaching third parties beyond what `SECURITY.md` §2
  and §3.bis already declare; resource exhaustion that needs one client but
  is not a single request.
- **Low.** Issues that need a malicious local user of the CLI or SDK, and
  anything in the research crates listed above.

## Anything to leave alone

These are declared limitations, written down in `SECURITY.md` (§2, §2.bis,
§3.3, §4) with their measurements; please do not report them as findings:

- The operator sees the whole state, orders and can censor operations (before
  a receipt it leaves no evidence); there is one node, no consensus, and no
  recovery if the node disappears.
- The operator's signing key is trusted on first use by the witness; there is
  no anchored key yet. The operator can swap the verifier binary unseen.
- The AIRs have no formal specification (`SECURITY.md` §3.1). A general remark
  that constraints are not formally verified is not a finding; a concrete
  witness that verifies and should not, is (see Critical).
- The node's RPC has no authentication, no TLS and no rate limit, listens on
  `127.0.0.1:8545` by default, and stores its ledger unencrypted at rest
  (`SECURITY.md` §3.3). Volume-based denial of service against it is known.
- Metadata: a send reveals sender, amount and position, a claim reveals
  receiver, amount and position (`SECURITY.md` §2); the `pending` of
  `zkssl_supply` reveals amounts by difference (§656).
- The vectors before 0.4 (`spec/vectors/zkssl-0.1.json` to `zkssl-0.3.json`)
  are kept as history: the 0.3 ones carry witness literals, and two sandbox
  spending keys derive from its rejection catalogue, on purpose. Keys in
  tests, vectors, `--clave` and the `sandbox` feature are fixtures, not leaked
  secrets.
- The `winter-*` fork and the `xmss` crate are unaudited. Bugs in them are in
  scope; their being unaudited is not.

Reports and patches: a minimal patch with a failing Rust test that the patch
turns green is most useful; English or Spanish are both fine.
