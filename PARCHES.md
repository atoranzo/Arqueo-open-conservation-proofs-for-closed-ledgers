# Parches mínimos para integrar el ecosistema (cli, wire, node, sdk)

Cinco cambios, ninguno toca lógica de producción. El motivo del 2 y el 3:
los ayudantes de escenario (`fund_delegated`, `mint_commitment`,
`custodian_root`…) usan campos **privados** de `SovereignLayer`
(`records`, `accounts`), así que un crate externo no puede replicarlos.
Exponer `tests_support` tras una feature es la vía honesta y mínima:
en compilación normal (`cargo build` sin features) no cambia nada.

## 1. `Cargo.toml` (raíz del workspace)

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

## 2. `crates/zk-ssl/Cargo.toml` — añadir al final

```toml
[features]
# Expone `tests_support` como sandbox para herramientas (zk-ssl-cli).
# Solo cambia visibilidad; no toca el código de producción.
sandbox = []
```

## 3. `crates/zk-ssl/src/lib.rs` — donde hoy pone:

```rust
#[cfg(test)]
mod tests_support;
```

dejar:

```rust
#[cfg(any(test, feature = "sandbox"))]
pub mod tests_support;
```

(`metrics` y `tests` se quedan como están, solo bajo `#[cfg(test)]`.)

## 4. `crates/zk-ssl/src/accounts.rs` — abrir cuenta SIN enviar la clave

Donde hoy pone `fn open_with_id(` dejar `pub fn open_with_id(`.

Motivo: es la puerta que permite a `zkssl_openAccount` recibir SOLO
identificadores derivados (`publicId`, `viewId`, `leafSalt`) en vez de la
clave de gasto. El propio comentario del método lo dice: «la clave no se
almacena en ningún sitio (§93.4)» — con esto, tampoco viaja. Las
derivaciones ya son públicas en `stark_experiment::native`, así que el
cambio no expone nada nuevo: solo evita que la vía cómoda
(`open_account_wide`) sea la única.

## 5. Copiar los crates

Copiar `zk-ssl-cli/`, `zk-ssl-wire/`, `zk-ssl-node/` y `zk-ssl-sdk/` de
este paquete dentro de `crates/`, y `spec/RPC.md` en la raíz del repo.

## Comprobar

```bash
cargo check -p zk-ssl-cli -p zk-ssl-wire -p zk-ssl-node -p zk-ssl-sdk

# sandbox local:
cargo run -p zk-ssl-cli -- simulate --amount 250000

# nodo dev + pago extremo a extremo con el SDK:
cargo run -p zk-ssl-node -- --dev &
#   (desde otro proceso, usando zk-ssl-sdk: Account::open, dev_fund, pay, claim)
```
