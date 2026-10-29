# zk-ssl-cli — Arqueo's sandbox and tracer

> **Español:** [`doc/README-CLI.md`](./README-CLI.md) is the original of this page, in Spanish; it
> prevails wherever the two differ. This translation is of commit `7766efb` (audit entry §917) and
> was produced by the project's assistant, a Claude Code session, in audit entry §918 (`GENAI.md`):
> it is a generated artefact, not text written by the author. Line citations of the form `file:N`
> refer to the Spanish files.

A Rust CLI to exercise the layer's logic, simulate operations, inspect
the state and print detailed traces **without bringing anything up**: it
runs the real `zk_ssl::SovereignLayer`, with real STARK proofs, in memory
or against a persisted `sled` ledger.

## And REVM? No, and the reason matters

REVM is an **EVM executor**: it interprets EVM bytecode and only models
Ethereum's state (accounts, storage slots, opcodes). Its `Inspector`
traces opcodes and CALL frames, nothing more.

This layer has no EVM: it is native Rust logic over STARK/FRI
(winterfell). The custodian-threshold nullifiers
(`circuit_threshold_single_nullifier`), the sparse accounts tree, the
pending tree, the two-phase send/claim flow and the chained
`TransitionLog` **do not exist for REVM**: there is no bytecode to
interpret nor slots to look at them in. Using REVM would demand
rewriting the whole layer as EVM contracts, losing precisely what defines
it (native STARK verification, no ceremony).

The correct equivalent of REVM's `Inspector` here is what this CLI does:
an enum of events (`TraceEvent`) emitted from the layer's real phases,
with interchangeable sinks (trait `Tracer`).

A nuance about "the nullifiers": in the STARK layer, double spending of
payments is closed with nonce + leaf + pending tree ("provable
non-membership"), not with note nullifiers; the nullifiers that DO exist
here are those of the custodian/governance authorisations, and
`simulate` prints them on every issuance. ⚠️ They are not an anti-replay:
they say that the two custodians of a pair are distinct, and the layer
does not store them. What prevents resending an authorisation is the
counter that goes in its commitment. The classic note nullifiers
(`nullifier.rs`, `persistent_nullifier_registry.rs`) live in the
experimental Halo2 backend, not in the layer.

## Structure

```
crates/zk-ssl-cli/
├── Cargo.toml
└── src/
    ├── main.rs      # clap (derive) + init de tracing-subscriber (stderr)
    ├── commands.rs  # simulate / trace-tx / inspect-state
    ├── conformance.rs # --emit/--check: los vectores del protocolo (§198)
    ├── sandbox.rs   # motor: fases reales de la capa, instrumentadas
    ├── trace.rs     # enum TraceEvent + trait Tracer (consola / JSONL)
    └── fmt.rs       # hex de Digest vía zk_ssl::store (misma serialización)
```

There is no `mychain-core` crate: the state engine already exists and it
is `zk-ssl`. The CLI depends on it with the `sandbox` feature (see
`PARCHES.md`).

## Usage

```bash
# Envío + cobro en dos fases, en memoria, con traza por fases:
cargo run -p zk-ssl-cli -- simulate --amount 250000

# Dejar el pendiente en tránsito (para verlo en inspect-state):
cargo run -p zk-ssl-cli -- simulate --no-claim

# Contra un ledger persistido (se crea si no existe):
cargo run -p zk-ssl-cli -- simulate --ledger ./ledger --amount 100000

# Paso a paso del registro encadenado (demo en memoria si no hay ledger):
cargo run -p zk-ssl-cli -- trace-tx --ledger ./ledger --last 20
cargo run -p zk-ssl-cli -- trace-tx --ledger ./ledger --seq 3

# Estado: raíces, suministro, en tránsito, cabeza del registro, cuentas:
cargo run -p zk-ssl-cli -- inspect-state --ledger ./ledger --accounts

# Conformidad del protocolo: re-ejecuta el escenario canónico y compara
# campo a campo contra los vectores fijados — la 2ª implementación empieza aquí:
cargo run --release -p zk-ssl-cli -- conformance --check spec/vectors/zkssl-0.5.json

# Eventos como JSON Lines (datos por stdout, diagnóstico por stderr):
cargo run -p zk-ssl-cli -- --json simulate | jq .

# Spans y tiempos internos:
cargo run -p zk-ssl-cli -- --log debug simulate
```

⚠️ **Corrected in §807**: the conformance command pointed to `spec/vectors/zkssl-0.3.json`, and the
wire is `zkssl/0.5` since §914 (⚠️ §914: it said `zkssl/0.4` since §538): with that file the
binary exits with 1 and `los vectores son de
OTRA version: zkssl/0.3 …`, which is what the canon demands of it (`tools/canon.sh`, «3 ·
conformidad»). With `zkssl-0.4.json` it exits with 0 and `CONFORMIDAD: 6 entradas + suministro +
pendiente, todo IDENTICO en lo que el circuito fija`: `proof_digest`, `chain` and `epoch_digest`
are not reproduced with hiding (RFC-0009, D-AI), so «field by field» is not every field.

⚠️ **Corrected in §716**: the command pointed to `spec/vectors/zkssl-0.4.json`. Since §716 the
commitment of the two issuances carries the issuance counter, and the scenario moves up to
`zkssl/0.5` with the wire at `zkssl/0.4` until the cut (RFC-0018; ⚠️ §914: the cut is done and the
wire says `zkssl/0.5`, the scenario's): with `zkssl-0.4.json` the binary exits with 1 and
`los vectores son de OTRA version: zkssl/0.4 …`, as with 0.3, and what the paragraph above says of
0.4 is given since §716 by `zkssl-0.5.json`.

Typical output of `simulate` (abridged; a release run of 2026-10-07):

```
━━ FASE FUND ━━ emisión delegada de 1000000 a #296740276: exige DOS custodios
  ✔ prueba STARK generada: 66.4 KB — digest cd3d146d…1b38f2a7 [559 ms]
  · custodio #1 autoriza — nullifier consumido: 81bf2d5f…  op: d441d94f…4c50121b
  · custodio #3 autoriza — nullifier consumido: 50956ab9…  op: d441d94f…4c50121b
  ✔ aplicado → log#1 Mint  raíz 6590b0bf…0ebb7b1a → f3fca118…3ebc0ad6  cadena 7dc073fa…f0c149c4 [3 ms]

━━ FASE SEND ━━ #296740276 → #4052836150, importe 250000: materiales → prueba LOCAL → apply_send
  · materiales listos (pendiente@0) [0 ms]
  ✔ prueba STARK generada: 76.5 KB — digest 20d965d5…c320be5f [930 ms]
  ✔ aplicado → log#4 Send  raíz 75418f6b…a592ea0b → d44542e0…6088e69e  cadena 50a3ace2…a1500ce1 [11 ms]
  el dinero está EN TRÁNSITO: no es del receptor hasta que cobre (§29)

━━ FASE CLAIM ━━ ...
✔ cadena de transiciones íntegra (6 entradas)
── estado del libro mayor ── ...
```

The CLI prints KB of 1,024 B. The proofs hide their witness since §538, and their size changes from
one run to another: the send's and the claim's are bounded by the band of
`crates/zk-ssl/src/metrics.rs`. ⚠️ **Corrected in §698**: the earlier output gave 54.3 KB for the
issuance and 64.6 KB for the send, which are the 66,164 B of the send of §218, from before hiding.

## Design decisions

- **`trace-tx` reads the layer's `TransitionLog`**, not a parallel log:
  the "transaction id" is the `seq` of the chained log, with its
  `proof_digest` and the chain digest. It is the audit mechanism the
  project already has; the CLI only makes it visible and runs
  `verify_chain()`.
- **`send`/`claim` go through the client path** (`send_materials` →
  `client::prove_send` → `apply_send`): no key reaches the layer, which
  is the front-page flow of the project's README.
- **No `tokio`.** The layer is synchronous and "there is no network nor
  consensus" by design. When a node with JSON-RPC exists, the clean thing
  is an `rpc` feature (tokio + jsonrpsee) that implements
  `Tracer`/queries against the node, not dragging in an async runtime
  today. That node **already exists** since §197 (`zk-ssl-node`, axum):
  this CLI stays synchronous on purpose, against the layer directly; the
  path with a network is SDK↔node.
- **`Result` at the edges, enums at the centre**: the layer's
  `LayerError`s go up as they are; every rejection emits
  `TraceEvent::Rejected` before propagating.

## Warnings

- Sandbox keys are **deterministic** (derived from `--key-seed`): only
  for testing. With `--ledger`, reuse the same seed it was created with,
  or `prove_send` will fail with `NotTheAccountHolder`. ⚠️ §802: but if
  the run has to open accounts, repeating the `--key-seed` on a ledger
  with `--ledger` opens the same identities again, and since §802 that
  is rejected (`IdentityUnavailable`): an identity has at most one
  account in the life of the ledger.
- The parameters (`--limit`, `--max-supply`, `--max-accounts`) of a
  persisted ledger must match those of its creation
  (`ParameterMismatch` otherwise: it is immutability, not a failure).
- The proofs are real: each phase takes of the order of seconds depending
  on the machine (the figures of PERFORMANCE.md apply).
- Balances listed with `--accounts`: the **operator's** view by design
  (§129 of AUDITORIA.md).
- Written against the current signatures of `zk-ssl` (verified in the
  source code); it passes `cargo check -p zk-ssl-cli` after applying
  `PARCHES.md` — this environment has no network to compile dependencies.
