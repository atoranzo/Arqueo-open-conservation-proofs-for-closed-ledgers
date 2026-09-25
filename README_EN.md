# Arqueo — open conservation proofs for closed ledgers

> **Español:** [`README.md`](./README.md) es esta página en español, sección por sección.

**Formerly ZK-SSL.** The project changed its name, not its published identifiers: the wire
protocol was `zkssl/0.3` and is `zkssl/0.4` since audit entry 538 (because of hiding, RFC-0009;
not because of the name), the crates are still `zk-ssl-*`, and the deposits with a DOI keep
the title they were deposited under. What this page says is verified against `main` at commit
`343d3b6`; whatever changes afterwards is recorded in [`AUDITORIA.md`](./AUDITORIA.md), one entry
per change.

An operator keeps a ledger. The people who depend on it — members, holders, beneficiaries,
counterparties — cannot see it, and the two sides do not trust each other. Today that conflict is
settled by a third party who opens the ledger: an auditor, a supervisor, a court; once a year; by
sample. Arqueo replaces the opening of the ledger with a **proof that the ledger did what its rules
say**, which anyone checks **without the ledger, offline, and without trusting the author**. It fits
wherever the unit of account is **born and dies inside the ledger**: issued by the operator, moved
between accounts, retired by the operator. It proves **conservation, not solvency**: the proofs
speak of the ledger, not of the world.

---

## Read this first

- **What it is.** A ledger engine in Rust: accounts; two-phase payments whose STARK proof is
  generated on the payer's machine — no trusted setup, no curves —; a **signed epoch head** with a
  hash-based signature (XMSS) that the node publishes; witnesses that **co-sign** it; and an
  **evidence package** that an independent verifier checks **with the node off**.
- **What a third party can check today, measured.** Conservation (supply = balances + in flight,
  also on reopening); single use of a label inside a ledger and **detection** of the same label in
  two ledgers; unrewritable history with an extension proof; inclusion with a receipt; authorship
  without the key travelling. The table in [What it guarantees and what it does
  not](#what-it-guarantees-and-what-it-does-not) gives the source of every row.
- **What it is not.** Not a chain: one node, one writer, no distributed consensus, no token. **The
  operator sees every balance** and can omit an operation without leaving a trace. Across ledgers
  it **detects, it does not prevent**.
- **Not audited by third parties.** No amount of the author's own tests replaces that. One
  cryptographic dependency (`xmss`, a pre-release) is pinned with `=` and declared. All of it in
  [`SECURITY.md`](./SECURITY.md).
- **The six deposits with a DOI predate corrections in the tree.** What was corrected is marked,
  not erased: [`doc/preprints/ERRATA.md`](./doc/preprints/ERRATA.md).

---

## Try it in five minutes

### Without Rust and without the repository: the verifier kit

One download, and four checks on the checker's own machine, offline. The current release is
`arqueo-verify-v0.2.0`, produced on commit `1528943fdfb9399f56fd836f75ffbe655d004d78`; the
tarball's fingerprint is published next to its commit, on the release page and in the
`AUDITORIA.md` entry that records it. The full script, with the expected output of every step, is
[`doc/KIT_EN.md`](./doc/KIT_EN.md).

```bash
sha256sum arqueo-verify-*.tar.gz          # must be the fingerprint published next to the commit
tar xzf arqueo-verify-*.tar.gz && cd arqueo-verify-*/
sha256sum -c SHA256SUMS                   # every file inside, against its fingerprint
cat VERSION                               # commit, describe, toolchain, glibc_max
```

```bash
# 1 · a file that adds up                                   -> VERDE, exit 0
./zk-ssl-verify spec/vectors/paquete/posicion-v2.json
./zk-ssl-verify spec/vectors/consumo/consumo.json

# 2 · a tampered one does not, and the program names the broken rule   -> exit 1
./zk-ssl-verify spec/vectors/paquete/rechazo-n-adulterado.json; echo "exit $?"
./zk-ssl-verify spec/vectors/consumo/rechazo-cons-ausencia-ya-estaba.json; echo "exit $?"

# 3 · the same label in two ledgers, with both nodes off      -> DETECTION, not prevention
./zk-ssl-verify spec/vectors/conflicto/conflicto.json

# 4 · a swap of ledgers is rejected by name                    -> exit 1 on both
./zk-ssl-verify spec/vectors/conflicto/rechazo-conf-camino-no-sube.json; echo "exit $?"
./zk-ssl-verify spec/vectors/conflicto/rechazo-conf-misma-clave.json; echo "exit $?"
```

And the whole catalogues, with the harness that travels inside the tarball: `bash conformidad.sh
./zk-ssl-verify` checks, entry by entry, that every vector says what its manifest declares.

### With stable Rust

`--release` is mandatory for the layer, not an optimisation (the circuits validate in debug mode
with degrees that depend on the witness).

```bash
git clone https://github.com/atoranzo/Arqueo-open-conservation-proofs-for-closed-ledgers
cd Arqueo-open-conservation-proofs-for-closed-ledgers

# a full payment (fund x2 -> send -> claim) with real STARK proofs and a per-phase trace
cargo run --release -p zk-ssl-cli -- simulate --amount 250000

# the contract of a second implementation: re-run the canonical scenario, compare field by field
cargo run --release -p zk-ssl-cli -- conformance --check spec/vectors/zkssl-0.3.json

# the canon: the table of tests per crate (--lista only shows it; --sello runs it whole)
bash tools/canon.sh --lista
```

You should see `cadena de transiciones íntegra` and `CONFORMIDAD: … todo IDENTICO`. The whole
CLI — `simulate`, `trace-tx`, `inspect-state`, `conformance` — is in
[`doc/README-CLI.md`](./doc/README-CLI.md) (Spanish).

---

## How it works, in one paragraph

A payment is two transitions: the payer **sends** (one leaf) and the payee **claims** (another),
each with a proof generated on their own machine; the layer hands out paths and roots — public
data — and verifies proofs. **The spend key does not travel through the API and, since §538, does
not come out literally in the proof either**: the house prover hides its witness (RFC-0009 E3b-2,
`zkssl/0.4`); between §521 and §538 the proof published it (see [`SECURITY.md`](./SECURITY.md)
§3.bis, in Spanish). Every epoch the node signs a **head** that binds the state, the chained
transition log and the tree of published consumptions; independent witnesses co-sign it and **pin
the key the first time they see it**. An **evidence package** carries a signed head, an
acknowledgement with its path and the co-signatures: a verifier that does not know the node
recomposes it and accepts or rejects it **naming the rule**. The core is in
[`spec/NUCLEO.md`](./spec/NUCLEO.md), the package in [`spec/PAQUETE.md`](./spec/PAQUETE.md) and the
published consumption in [`spec/rfc/0006-consumo-publicado.md`](./spec/rfc/0006-consumo-publicado.md)
(specification files are in Spanish; [`spec/README.md`](./spec/README.md) is the reader's guide in
English).

---

## What it guarantees and what it does not

The same table, with the use cases, is [`doc/USE_CASES.md`](./doc/USE_CASES.md).

| # | property | what a third party checks | status |
|---|---|---|---|
| 1 | Conservation | supply = balances + in flight; nothing created or lost between epochs | measured, in flight and on reopening (`AUDITORIA.md` §387–§394) |
| 2 | No double use | a label is consumed once in a ledger and published in its signed head; the same label in two ledgers is detected from both | measured (RFC-0006; `doc/KIT_EN.md`) |
| 3 | Unrewritable history, with an extension proof | today's signed head extends yesterday's without removal or reordering | measured (`spec/RPC.md`, `zkssl_consistencyProof`) |
| 4 | Inclusion with a receipt | an entry is in the ledger, provable without the operator | measured (`spec/RPC.md`, `zkssl_inclusionReceipt`, `zkssl_ackPath`) |
| 5 | Authorship without the key travelling | only the holder of a key moves its account; the operator cannot | measured (`spec/RPC.md`, the API principle) |
| 6 | Cut-off and completeness | nothing stays in flight past its time; every acknowledgement ends applied or rejected, with a trace | partly: the empty box — nothing in flight older than a given age — is proven without the node (RFC-0007, E4); completeness is planned |
| 7 | Rejection with cause | a refusal carries the rule that produced it | measured (RFC-0007; which causes are proven without the node: `spec/PAQUETE.md`, 2.6) |

Row 6 **exists only in part**: the empty box is proven, and that every acknowledgement ends
applied or rejected is not yet; it is listed whole so that a reader knows which question the
engine intends to answer.

**What none of this claims:**

- Privacy against the operator: the operator sees everything.
- That the ledger's units exist outside the ledger.
- That an omitted operation would be detected: censorship leaves no trace.
- Prevention across ledgers: two ledgers can accept the same label; a third party holding both
  signed heads sees it afterwards, never before.
- Who is behind a key, or that one person holds one account.
- Row 6 whole: the completeness of acknowledgements does not exist yet.
- Of a rejection with cause, that the node refused, or when, or that the rule is fair: the
  proof says the rule was applied over what a signed head commits.

**What is missing, in order of importance:** distributed consensus (without it the operator sees
the balances and can censor; the alternative this project does pursue — provable accountability,
in the manner of Certificate Transparency — has the signer, the index guardian, the heartbeat, the
independent verifier and the witnesses, and lacks an anchor prior to the first encounter and a
**verified** key custody, not just a declared one) · an external audit · the admission receipt
(`AUDITORIA.md` §121) · delegating the proof to third parties (verifying the signature in circuit) ·
an expiry policy for freezes. Everything else is listed in [`AUDITORIA.md`](./AUDITORIA.md),
section 4.

---

## Reading order

| If you are… | Start with |
|---|---|
| **An evaluator or an external auditor** | [`doc/KIT_EN.md`](./doc/KIT_EN.md) · [`doc/USE_CASES.md`](./doc/USE_CASES.md) · [`SECURITY.md`](./SECURITY.md) · [`AUDITORIA.md`](./AUDITORIA.md) |
| **Implementing the protocol** | [`spec/README.md`](./spec/README.md) · [`spec/RPC.md`](./spec/RPC.md) + [`spec/openrpc.json`](./spec/openrpc.json) · [`spec/vectors/`](./spec/vectors/) · [`spec/PAQUETE.md`](./spec/PAQUETE.md) · [`spec/rfc/`](./spec/rfc/) |
| **A cryptographer** | [`spec/NUCLEO.md`](./spec/NUCLEO.md) · [`ARQUITECTURA.md`](./ARQUITECTURA.md) · [`doc/VERIFICACION_FORMAL.md`](./doc/VERIFICACION_FORMAL.md) + [`doc/fv/mapa_fv_capas.md`](./doc/fv/mapa_fv_capas.md) · [`FIVE_BACKENDS.md`](./FIVE_BACKENDS.md) |
| After the reasoning behind it | [`PRINCIPIOS.md`](./PRINCIPIOS.md) · [`doc/IDEA_CENTRAL.md`](./doc/IDEA_CENTRAL.md) · [`doc/APORTACION.md`](./doc/APORTACION.md) · [`doc/CONSECUENCIAS.md`](./doc/CONSECUENCIAS.md) |
| With questions | [`QUESTIONS.md`](./QUESTIONS.md) · [`PREGUNTAS.md`](./PREGUNTAS.md) |
| Five minutes and not technical | [`RESUMEN_BILINGUE.md`](./RESUMEN_BILINGUE.md) · [`RESUMEN_EJECUTIVO.md`](./RESUMEN_EJECUTIVO.md) |
| Institutions, and the limits of scale | [`doc/INSTITUTIONAL.md`](./doc/INSTITUTIONAL.md) · [`doc/INSTITUCIONAL.md`](./doc/INSTITUCIONAL.md) |
| Coming from Zenodo | [`doc/ZENODO.md`](./doc/ZENODO.md) |
| Contributing, or reporting a vulnerability | [`CONTRIBUTING.md`](./CONTRIBUTING.md) · [`SECURITY.md`](./SECURITY.md) |

Most of the repository is written in Spanish. The pages in English are this one,
`spec/README.md`, `doc/KIT_EN.md`, `doc/USE_CASES.md`, `QUESTIONS.md`, `doc/INSTITUTIONAL.md`,
`PAPER_EN.md` and the preprints under `doc/preprints/`. `AUDITORIA.md` includes a section with
**the points where the author has the least confidence**; if you intend to break the code, start
there.

---

## Status

| piece | where it is measured |
|---|---|
| **21 crates** in one workspace —18 of our own and the three of the winterfell 0.13.1 fork (§533)—; the canon (`tools/canon.sh --sello`) runs every crate's tests, in release, and the eight tools under `tools/` that watch figures, citations, domains and geometry | the table in [`tools/canon.sh`](./tools/canon.sh) carries the passing tests per crate; every seal updates it |
| **Protocol `zkssl/0.4`**: 30 JSON-RPC methods (27 `zkssl_*`, 3 `dev_*`), OpenRPC generated from the code, vectors per version that are never rewritten | [`spec/RPC.md`](./spec/RPC.md) · [`spec/openrpc.json`](./spec/openrpc.json) · [`spec/vectors/`](./spec/vectors/) (315 files: cable, núcleo, paquete, consumo, conflicto, rechazo, edad, pendiente, pago, prenda, the 0.3 catalogues under `0.3/` and the four `zkssl-0.N.json`) |
| **RFCs**: 0002, 0003, 0004, 0006, 0007 (proofs over the committed state), 0008 (the portable proofs of a pending item) and 0009 (what a proof reveals) accepted; 0005 (the frozen core) and 0010 (the reception receipt) proposed | [`spec/rfc/`](./spec/rfc/) |
| **Independent verifier** `zk-ssl-verify` 0.2.0, release `arqueo-verify-v0.2.0`, reproducible from the commit its `VERSION` names | [`doc/KIT_EN.md`](./doc/KIT_EN.md) · [`tools/artefacto.sh`](./tools/artefacto.sh) |
| **Record**: one entry per verified change, with its commit; what is corrected is marked, not erased | [`AUDITORIA.md`](./AUDITORIA.md) · [`BACKLOG.md`](./BACKLOG.md) |

The design was chosen by measuring **the same circuit in five proof systems**
([`FIVE_BACKENDS.md`](./FIVE_BACKENDS.md)); STARK/FRI without a ceremony was the only decision
taken against the numbers, because a ceremony whose participants collude can create money without
a trace. Measured times and sizes, with their dispersion, are in `AUDITORIA.md` (§130 and §131 the
dispersion between runs, §229 the node's ceiling per RPC); they are not repeated here so that they
do not go stale.

---

## Papers

Preprints, with their current DOI on Zenodo. Earlier versions remain accessible and are cited in
the Spanish page: **a published figure that is corrected is not erased, it is marked**.

- **Comparative Implementation of a Zero-Knowledge Settlement Layer across Five Proof Systems:
  Design Findings and Measurements** — [10.5281/zenodo.21736125](https://doi.org/10.5281/zenodo.21736125)
- **Provable Compliance without Full Ledger Disclosure — A Zero-Knowledge Settlement Architecture
  for Supervisory Audit** — [10.5281/zenodo.21736082](https://doi.org/10.5281/zenodo.21736082)
- **From Institutional Trust to Verifiable Properties — A Minimal ZK Settlement Layer and Its
  Residual Trust Surface** — [10.5281/zenodo.21905595](https://doi.org/10.5281/zenodo.21905595)
- **Reputation, Residual Power, and Verification Investment in Digital Settlement** —
  [10.5281/zenodo.22078086](https://doi.org/10.5281/zenodo.22078086)
- **Residual Surfaces in Retail CBDC Incidents and the Verification–Residual Partition: A Coding
  Study with Contrast to an Explicit Residual-Trust Settlement Design** —
  [10.5281/zenodo.22077991](https://doi.org/10.5281/zenodo.22077991)
- **Residual Trust After Verification: A Microeconomic Account of What Proofs Cannot Eliminate** —
  [10.5281/zenodo.22076721](https://doi.org/10.5281/zenodo.22076721)

Further notes: [`doc/ZENODO.md`](./doc/ZENODO.md).

## Authorship and license

**Angel Toranzo Portela**, 2026. How to cite: [`CITATION.cff`](./CITATION.cff).

Dual licensed: MIT **or** Apache-2.0, at your option. See [`LICENSE-MIT`](./LICENSE-MIT) and
[`LICENSE-APACHE`](./LICENSE-APACHE). Both licenses require keeping the copyright notice and the
license text in any copy or derivative; Apache-2.0 also requires honouring [`NOTICE`](./NOTICE).

### Third-party code

`crates/ceremony/` **is not original code of this project**: it comes from
`penumbra-sdk-proof-setup` (Penumbra Labs), under the same dual license. See
[`crates/ceremony/ATTRIBUTION.md`](./crates/ceremony/ATTRIBUTION.md).

### ⚠️ No institutional affiliation

This project is **independent** work. **It is not affiliated with, endorsed by or commissioned by
the European Central Bank, the Eurosystem, or any other institution**, public or private.
References to the digital euro are to the public design published by those institutions and are
cited as the context of a technical problem. **No statement in this repository should be read as
the position of anyone but its author.**
