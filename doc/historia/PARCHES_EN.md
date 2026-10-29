> **Document of the ZK-SSL stage (until September 2026).** It is kept as it was, as history:
> its statements in the present tense describe the project of that time, not today's. The current
> position is in [`README.md`](../../README_EN.md) and [`doc/USE_CASES.md`](../USE_CASES.md); what
> changed since then is in [`AUDITORIA.md`](../../AUDITORIA.md) (§448).

# Minimal patches to integrate the ecosystem (cli, wire, node, sdk)

> **Español:** [`doc/historia/PARCHES.md`](./PARCHES.md) is the original of this page, in Spanish;
> it prevails wherever the two differ. This translation is of commit `7766efb` (audit entry §917)
> and was produced by the project's assistant, a Claude Code session, in audit entry §918
> (`GENAI.md`): it is a generated artefact, not text written by the author. Line citations of the
> form `file:N` refer to the Spanish files.

Five changes, none touches production logic. The reason for 2 and 3:
the scenario helpers (`fund_delegated`, `mint_commitment`,
`custodian_root`…) use **private** fields of `SovereignLayer`
(`records`, `accounts`), so an external crate cannot replicate them.
Exposing `tests_support` behind a feature is the honest and minimal way:
in a normal build (`cargo build` without features) nothing changes.

## 1. `Cargo.toml` (workspace root)

```toml
[workspace]
members = [
    "crates/zk-core",
    "crates/iso-bridge",
    "crates/halo2-experiment",
    "crates/settlement-prover",
    "crates/stark-experiment",
    "crates/ceremony",
    "crates/plonk-experiment",
    "crates/nova-experiment",
    "crates/settlement-layer",
    "crates/zk-ssl",
    "crates/zk-ssl-cli",        # ← NUEVO
    "crates/zk-ssl-wire",       # ← NUEVO (formato de cable)
    "crates/zk-ssl-node",       # ← NUEVO (nodo JSON-RPC)
    "crates/zk-ssl-sdk",        # ← NUEVO (SDK de cliente)
]
```

## 2. `crates/zk-ssl/Cargo.toml` — add at the end

```toml
[features]
# Expone `tests_support` como sandbox para herramientas (zk-ssl-cli).
# Solo cambia visibilidad; no toca el código de producción.
sandbox = []
```

## 3. `crates/zk-ssl/src/lib.rs` — where today it says:

```rust
#[cfg(test)]
mod tests_support;
```

leave:

```rust
#[cfg(any(test, feature = "sandbox"))]
pub mod tests_support;
```

(`metrics` and `tests` stay as they are, only under `#[cfg(test)]`.)

## 4. `crates/zk-ssl/src/accounts.rs` — open an account WITHOUT sending the key

Where today it says `fn open_with_id(` leave `pub fn open_with_id(`.

Reason: it is the door that lets `zkssl_openAccount` receive ONLY
derived identifiers (`publicId`, `viewId`, `leafSalt`) instead of the
spending key. The method's own comment says so: «la clave no se
almacena en ningún sitio (§93.4)» [the key is not stored anywhere] — with
this, it does not travel either. The derivations are already public in
`stark_experiment::native`, so the change exposes nothing new: it only
avoids the convenient path (`open_account_wide`) being the only one.

## 5. Copy the crates

Copy `zk-ssl-cli/`, `zk-ssl-wire/`, `zk-ssl-node/` and `zk-ssl-sdk/` from
this package into `crates/`, and `spec/RPC.md` into the root of the repo.

## Check

```bash
cargo check -p zk-ssl-cli -p zk-ssl-wire -p zk-ssl-node -p zk-ssl-sdk

# sandbox local:
cargo run -p zk-ssl-cli -- simulate --amount 250000

# nodo dev + pago extremo a extremo con el SDK:
cargo run -p zk-ssl-node -- --dev &
#   (desde otro proceso, usando zk-ssl-sdk: Account::open, dev_fund, pay, claim)
```
