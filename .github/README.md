# Arqueo — open conservation proofs for closed ledgers

> This front page is a one-screen summary written by the project's assistant, a Claude Code
> session, in audit entry §920 (see [`GENAI.md`](/GENAI.md)): a generated artefact, not text
> written by the author. Where it and the page it links differ, the linked page prevails.
> **Full README:** [English](/README_EN.md) · [Español](/README.md).

**What it is.** An operator keeps a ledger that the people who depend on it cannot see. Arqueo
replaces opening the ledger with **proofs that it did what its rules say**: a Rust ledger engine
whose payments carry STARK proofs (no trusted setup, no curves), a signed epoch head, witnesses
that co-sign it, and an evidence package that an independent verifier checks **offline, with the
node off**. It proves **conservation, not solvency**.

**What it is not.** Not a blockchain: one node, one writer, no consensus, no token. Not trustless:
the operator sees every balance, and an operation it never acknowledged leaves no trace. Across
ledgers it **detects, it does not prevent**. Not audited by third parties.

```
payer ── STARK proof ──▶ node: the closed ledger, applies or rejects
                           │ signs the epoch head (XMSS), issues receipts
                           ▼
        evidence package ──▶ verifier kit: any third party, offline, node off
                           │
               (optional)  └─▶ anchor medium, co-signed by independent witnesses
```

## Status today

| property | who checks it | where |
|---|---|---|
| Single use of a label; history; inclusion with a receipt | a third party, offline | [README, rows 2–4](/README_EN.md#what-it-guarantees-and-what-it-does-not) |
| Cut-off and completeness of what the node acknowledged | a third party, offline | RFC-0010, [row 6](/README_EN.md#what-it-guarantees-and-what-it-does-not) |
| Conservation and authorship of each payment | proved in circuit, **verified by the node**, not yet by the third party | [rows 1 and 5](/README_EN.md#what-it-guarantees-and-what-it-does-not) |
| Hiding | no literal of the witness in a proof since `zkssl/0.4`; since `zkssl/0.5`, simulatable without the witness for the 23 families, under named hypotheses; outside review still open | [RFC-0009](/spec/rfc/en/0009-lo-que-revela-una-prueba.md) |
| Soundness | 127 conjectured bits, classical; quantum: not measured | [`SECURITY_EN.md`](/SECURITY_EN.md) |
| External anchor | format and verifier built; the medium's publisher and its deployment are not | [RFC-0013](/spec/rfc/en/0013-el-medio-del-ancla.md) |
| Third-party audit | none | [`SECURITY_EN.md`](/SECURITY_EN.md) |

Wire protocol `zkssl/0.5`; current verifier
[`arqueo-verify-v0.5.0`](https://github.com/atoranzo/Arqueo-open-conservation-proofs-for-closed-ledgers/releases/tag/arqueo-verify-v0.5.0).

## Start here

1. **Check it yourself in five minutes**, with no Rust and no repository:
   [the verifier kit](/README_EN.md#try-it-in-five-minutes) and its full script,
   [`doc/KIT_EN.md`](/doc/KIT_EN.md).
2. **What it does not protect:** [`SECURITY_EN.md`](/SECURITY_EN.md), section 2.
3. **Reviewing the project** (NLnet, an auditor, thirty minutes):
   [`doc/FOR_REVIEWERS.md`](/doc/FOR_REVIEWERS.md), with a map of the repository, and the
   [glossary](/doc/GLOSSARY.md) of the house's terms.

**Generative AI.** Arqueo has been developed with a generative AI assistant (Claude) since
29 July 2026, much of its code and documentation in Claude Code sessions, each change recorded in
[`AUDITORIA.md`](/AUDITORIA.md). What was used and how: [`GENAI.md`](/GENAI.md).

**Language.** Most of the repository is written in Spanish; the pages written in Spanish have an
English twin, generated and declared as such: [`TRANSLATIONS.md`](/TRANSLATIONS.md).

**License.** MIT or Apache-2.0, at your option. Author: Angel Toranzo Portela.
