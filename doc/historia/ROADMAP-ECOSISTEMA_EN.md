> **Document of the ZK-SSL stage (until September 2026).** It is kept as it was, as history:
> its statements in the present tense describe the project of that time, not today's. The current
> position is in [`README.md`](../../README_EN.md) and [`doc/USE_CASES.md`](../USE_CASES.md); what
> changed since then is in [`AUDITORIA.md`](../../AUDITORIA.md) (§448).

# Ecosystem roadmap — what turns an implementation into a standard

> **Español:** [`doc/historia/ROADMAP-ECOSISTEMA.md`](./ROADMAP-ECOSISTEMA.md) is the original of
> this page, in Spanish; it prevails wherever the two differ. This translation is of commit
> `7766efb` (audit entry §917) and was produced by the project's assistant, a Claude Code session,
> in audit entry §918 (`GENAI.md`): it is a generated artefact, not text written by the author. Line
> citations of the form `file:N` refer to the Spanish files.

Neither SAP nor Ethereum won on the quality of the core alone: they won
because **anyone could build on top without asking permission**. Four
things give that: a stable specification, a reference implementation,
SDKs that preserve the security model, and conformance vectors so that
second implementations can exist. This roadmap adapts the most accepted
tools of other ecosystems to ZK-SSL.

> **Status (06-08-2026): Phases 0 and 1 are EXECUTED and sealed**
> (§197–§199 of `AUDITORIA.md`). This document went from plan to record;
> Phase 2 remains open.

## Map: established tool → ZK-SSL equivalent

| ecosystem | tool | ZK-SSL equivalent | status |
|---|---|---|---|
| Ethereum | `execution-apis` (RPC spec) | `spec/RPC.md` (**`zkssl/0.3`** since §354) | ✅ sealed (§197) |
| Ethereum | QUANTITY/DATA convention | `zk-ssl-wire` (0x hex, canonical digests) | ✅ sealed (§197) |
| Ethereum | geth (node) | `zk-ssl-node` | ✅ sealed (§197) |
| Foundry | anvil (devnet + faucet) | `zk-ssl-node --dev` (`dev_fund` with visible custodian nullifiers) | ✅ sealed (§197) |
| Ethereum | ethers/web3 (SDK) | `zk-ssl-sdk` (LOCAL proof, the key does not travel) | ✅ sealed (§197) |
| Foundry | cast / console | `zk-ssl-cli` (simulate / trace-tx / inspect-state / **conformance**) | ✅ sealed (§197–§198) |
| Ethereum | OpenRPC JSON | generated from `zk-ssl-wire` | ✅ sealed (§198): regenerating reproduces byte for byte |
| Ethereum | EIPs | protocol RFC process (template + numbering) | ✅ sealed (§198): `spec/rfc/` |
| Ethereum | hive / test vectors | **conformance vectors**: canonical scenario → expected roots and digests per version | ✅ sealed (§198): `conformance --check`, permanent gate |
| Ethereum | Etherscan | explorer over `zkssl_logEntries` + `verifyChain` (static HTML first) | Phase 2 |
| geth | encrypted keystore | wallet at rest with chacha20poly1305 + KDF (deps already present in zk-ssl) | ✅ sealed (§199): own domain, domain test |
| — | TS SDK | TypeScript client generated against `spec/RPC.md` (testing in a browser demands compiling the prover to WASM: to evaluate) | later |
| **SAP** | speak the industry's language | **ISO 20022**: the bridge already exists (`iso-bridge`); certifying pacs/camt messages against the layer is the SAP-type move, not the Ethereum-type one | later, priority |

## Phases

**Phase 0 — ✅ SEALED (§197).** Wire + spec + reference node + SDK.
Success criterion MET and recorded: the e2e ended with alice 750000 ·
bob 1250000 · `verifyChain ok`, **without any spending key travelling**
(verified in code: `spend_key` does not even implement `Serialize`).

**Phase 1 — ✅ SEALED (§198–§199).** OpenRPC generated,
versioned conformance vectors (the roots are deterministic per
operation; the proof digests are pinned PER VERSION of the circuit),
RFC process, keystore.

**Phase 2 — adoption.** Explorer, TS SDK, and the institutional route:
end-to-end ISO 20022 conformance with `iso-bridge`, which is where this
system really competes (settlement), not in Ethereum's niche.

## Known debts the RPC inherits (declared, not hidden)

- The notice (`PendingNotice`) travels **out of band** (§21): the RPC
  does not carry it to third parties on purpose, but the payer→recipient
  transport problem remains open.
- Single node: the operator orders and can censor; the RPC does not hide
  it.
- `--dev` uses test custodians; a production build is compiled without
  that feature and today demands adding flags for real roots (marked in
  `zk-ssl-node/src/main.rs`).
