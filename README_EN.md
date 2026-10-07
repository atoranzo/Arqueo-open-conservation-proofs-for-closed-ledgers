# Arqueo — open conservation proofs for closed ledgers

> **Español:** [`README.md`](./README.md) es esta página en español, sección por sección.

**Formerly ZK-SSL.** The project changed its name, not its published identifiers: the wire
protocol was `zkssl/0.3` and is `zkssl/0.4` since audit entry 538 (because of hiding, RFC-0009;
not because of the name; and that label does not say which proofs a node accepts: «Status»), the crates are still `zk-ssl-*`, and the deposits with a DOI keep
the title they were deposited under. What this page says is verified against `main` at commit
`343d3b6`; whatever changes afterwards is recorded in [`AUDITORIA.md`](./AUDITORIA.md), one entry
per change.

An operator keeps a ledger. The people who depend on it — members, holders, beneficiaries,
counterparties — cannot see it, and the two sides do not trust each other. Today that conflict is
settled by a third party who opens the ledger: an auditor, a supervisor, a court; once a year; by
sample. Arqueo replaces the opening of the ledger with **proofs that the ledger did what its rules
say**. Part of it anyone checks **without the ledger, offline, and without trusting the author**:
history, inclusion with a receipt, single use of a label, cut-off and completeness and, for the
causes the rejection envelope covers, the cause of a refusal. Part of it, today, only the node
checks: the **conservation** and the **authorship** of each payment travel inside its proof, and
that proof is verified by the node before it applies the payment, not by the third party's
verifier (rows 1 and 5 of [What it guarantees and what it does
not](#what-it-guarantees-and-what-it-does-not)). It fits wherever the unit of account is **born and
dies inside the ledger**: issued by the operator, moved between accounts, retired by the operator.
It proves **conservation, not solvency**: the proofs speak of the ledger, not of the world.

---

## Read this first

- **What it is.** A ledger engine in Rust: accounts; two-phase payments whose STARK proof is
  generated on the payer's machine — no trusted setup, no curves —; a **signed epoch head** with a
  hash-based signature (XMSS) that the node publishes; witnesses that **co-sign** it; and an
  **evidence package** that an independent verifier checks **with the node off**.
- **What a third party can check today, measured.** Single use of a label inside a ledger and
  **detection** of the same label in two ledgers; unrewritable history with an extension proof;
  inclusion with a receipt. The table in [What it guarantees and what it does
  not](#what-it-guarantees-and-what-it-does-not) gives the source of every row.
- **What the node checks, and a third party does not yet.** The conservation of each transition
  — on a send and a claim, the balance changes by exactly the amount and the supply does not
  change; on a burn, both drop by the amount — and authorship — only the holder of a key moves its
  account — are enforced in circuit by the send, claim and burn proofs; the aggregate (supply =
  balances + in flight) is checked by the layer on reopening the ledger. The node verifies each of
  those proofs before it applies the transition, and the log keeps only their digest: the third
  party's verifier does not verify them again, nor the issuance and refund ones. Of the supply, a
  third party sees the one the head signs; of authorship, the pledge's.
- **What it is not.** Not a chain: one node, one writer, no distributed consensus, no token. **The
  operator sees every balance** and can omit an operation: without a trace if it issues no
  receipt; if it issued one, the completeness envelope names it (RFC-0010). Across ledgers it
  **detects, it does not prevent**.
- **Not audited by third parties.** No amount of the author's own tests replaces that. Two
  cryptographic dependencies without an independent audit —`xmss`, a pre-release, and `ml-dsa`,
  in the kit since §633— are pinned with `=` and declared, and since §694 the canon checks it,
  together with winterfell's pins and the kit's closure as a closed list. All of it in
  [`SECURITY.md`](./SECURITY.md).
- **The six deposits with a DOI predate corrections in the tree.** What was corrected is marked,
  not erased: [`doc/preprints/ERRATA.md`](./doc/preprints/ERRATA.md).

---

## Try it in five minutes

### Without Rust and without the repository: the verifier kit

One download, and five checks on the checker's own machine, offline. The current release is
`arqueo-verify-v0.4.2`, produced on commit `a4e888d8b98ca6254904486f6ada8710c2a5fe08`; the
tarball's fingerprint is published next to its commit, on the release page and in the
`AUDITORIA.md` entry that records it. The full script, with the expected output of every step, is
[`doc/KIT_EN.md`](./doc/KIT_EN.md). It recomposes heads up to v6, the one the node signs today, and
reads the ten envelope families; release 0.2.0, which stops at v4, is still published and rejects
what came later — it fails closed, never a false GREEN. Measured and declared in `doc/KIT_EN.md`,
section 0.

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

# 5 · what the node RECEIVED under its signature: resolved in its window, or the operator named
./zk-ssl-verify spec/vectors/completitud/resuelta-por-acuse.json; echo "exit $?"   # -> 0
./zk-ssl-verify spec/vectors/completitud/no-resuelta.json; echo "exit $?"          # -> 1
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
| 1 | Conservation | the supply the head signs (since v5) and, for a rejection by cap, that the head's supply plus the amount exceeds the committed cap (`spec/PAQUETE.md`, 2.6); **not** that supply = balances + in flight | measured **in the node**: in circuit, on every send, claim and burn, and the aggregate in the layer on reopening (`AUDITORIA.md` §379; the roots, §387, §388, §391 and §392); **without the node, no**: the kit verifies no transition proof — nor the issuance and refund ones —, and the log does not keep them |
| 2 | No double use | a label is consumed once in a ledger and published in its signed head; the same label in two ledgers is detected from both | measured (RFC-0006; `doc/KIT_EN.md`) |
| 3 | Unrewritable history, with an extension proof | today's signed head extends yesterday's without removal or reordering | measured (`spec/RPC.md`, `zkssl_consistencyProof`) |
| 4 | Inclusion with a receipt | an entry is in the ledger, provable without the operator | measured (`spec/RPC.md`, `zkssl_inclusionReceipt`, `zkssl_ackPath`) |
| 5 | Authorship without the key travelling | of the pledge, that whoever holds the key produced it (`spec/PAQUETE.md`, 2.10); **not** that only the holder of a key moves its account | measured **in the node**: in circuit (`C_PK_CHECK`), on every send, claim and burn; **without the node, only the pledge**. That the key does not travel: `spec/RPC.md`, the API principle |
| 6 | Cut-off and completeness | nothing stays in flight past its time; every operation the node receives ends applied, rejected with proof or declared, or a named red says it did not | measured (RFC-0010; `spec/PAQUETE.md`, 2.11; the empty box, RFC-0007 E4) |
| 7 | Rejection with cause | a refusal carries the rule that produced it | measured (RFC-0007; which causes are proven without the node: `spec/PAQUETE.md`, 2.6) |

Row 6, since RFC-0010 (H5b, §556–§577): every operation the node evaluates on the holder's
direct paths gets a receipt under its signed head, and the completeness envelope says, with no
node, that it was resolved in its window —applied, or rejected with proof— or names the operator
that did not («NOT RESOLVED IN THE WINDOW»), or counts it apart when its cause has no portable
proof. Its residue is declared: an operator that issues no receipt leaves no trace (D-H). The
batch and the pledge carry one since §611, and the completeness envelope resolves them since §613
(RFC-0014).

**What none of this claims:**

- That a third party checks, without the node, the conservation or the authorship of a payment:
  the send, claim and burn proofs are verified by the node, the log keeps only their digest, and
  no method of the wire protocol serves them (rows 1 and 5).
- Privacy against the operator: the operator sees everything.
- That the ledger's units exist outside the ledger.
- That an operation the node never acknowledged would be detected: an operator that issues no
  receipt leaves no trace (RFC-0010, D-H), and the batch and the pledge carry none (D-E).
- Prevention across ledgers: two ledgers can accept the same label; a third party holding both
  signed heads sees it afterwards, never before.
- Who is behind a key, or that one person holds one account.
- Of a rejection with cause, that the node refused, or when, or that the rule is fair: the
  proof says the rule was applied over what a signed head commits.

⚠️ **Corrected in §581**: until then row 6 read «partly» and this list also said «censorship
leaves no trace» and «Row 6 whole: the completeness of acknowledgements does not exist yet».

⚠️ **Corrected in §696**: until then row 1 said that a third party checks «supply = balances + in
flight; nothing created or lost between epochs», «measured, in flight and on reopening»; row 5,
«only the holder of a key moves its account; the operator cannot», «measured»; the opening
paragraph, «a proof that the ledger did what its rules say, which anyone checks without the
ledger, offline, and without trusting the author», with no exception; and «Read this first»
counted conservation and authorship among what a third party can check today. §379 and §387–§394
are checks by the node, not by a third party. The title does not change in that entry:
whether it is reworded, or the path that would make conservation checkable by a third party is
built, is the author's decision (`BACKLOG.md`, entry 115).

**What is missing, in order of importance:** distributed consensus (without it the operator sees
the balances and can censor; the alternative this project does pursue — provable accountability,
in the manner of Certificate Transparency — has the signer, the index guardian, the heartbeat, the
independent verifier and the witnesses, and lacks the deployment of the external anchor — the
format and its verifier exist since RFC-0012; the medium, and with it the anchor prior to the
first encounter, is still missing — and a
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
| **22 crates** in one workspace —19 of our own and the three of the winterfell 0.13.1 fork (§533)—; the canon (`tools/canon.sh --sello`) runs every crate's tests, in release, and the nine tools under `tools/` that watch figures, citations, domains, the core census and geometry | the table in [`tools/canon.sh`](./tools/canon.sh) carries the passing tests per crate; every seal updates it |
| **Protocol `zkssl/0.4`**: 32 JSON-RPC methods (29 `zkssl_*`, 3 `dev_*`), OpenRPC generated from the code, vectors per version, each one's hash fixed in `spec/vectors/HUELLAS.sha256` and checked by every canon run (§692), which since then lets none be rewritten. ⚠️ **The version does not identify which proofs a node accepts** (§699): the proofs changed without raising it in §641 (the layer's seventeen circuits), §680 (send, claim and burn) and §684 (threshold), because conformance does not compare proof bytes; measured, a node rejects a client's send when both say `zkssl/0.4` and §641 or §680 lies between them, whichever is older. This holds until the `zkssl/0.5` cut. And `zkssl-0.2.json` was re-issued three times under its own version before the hashes (§275, §278 and §281). ⚠️ **Corrected in §699**: it said «vectors per version that are never rewritten» | [`spec/RPC.md`](./spec/RPC.md) · [`spec/openrpc.json`](./spec/openrpc.json) · [`spec/vectors/`](./spec/vectors/) (496 files: cable, núcleo, paquete, consumo, conflicto, rechazo, edad, pendiente, pago, prenda, completitud, ancla, ancla cofirmada, rotación, the 0.3 catalogues under `0.3/`, the two completitud vectors of kit 0.4 under `0.4/`, the four `zkssl-0.N.json` and `HUELLAS.sha256`) |
| **RFCs**: 0002, 0003, 0004, 0006, 0007 (proofs over the committed state), 0008 (the portable proofs of a pending item), 0009 (what a proof reveals), 0010 (the reception receipt), 0011 (the lying node), 0012 (the head anchor) and 0014 (the batch and pledge receipt) accepted; 0001 (the KDF at rest: Argon2id with salt and cost), 0005 (the frozen core), 0013 (the anchor's medium), 0015 (the signer's lifecycle), 0016 (one value, one encoding), 0017 (the signature attests the arithmetic), 0018 (the `zkssl/0.5` train) and 0019 (completeness that cannot be dodged) proposed | [`spec/rfc/`](./spec/rfc/) |
| **Independent verifier** `zk-ssl-verify` 0.4.2, published as `arqueo-verify-v0.4.2` and reproducible from the commit its `VERSION` names | [`doc/KIT_EN.md`](./doc/KIT_EN.md) · [`tools/artefacto.sh`](./tools/artefacto.sh) |
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

### Use of generative AI

This project is developed with the assistance of a generative AI model
(Anthropic's Claude): the assistant proposes, **the author runs, measures
and accepts**, and only what the canon lets through gets into `main`. The
method, its scope and where the per-change record lives:
[`GENAI.md`](./GENAI.md).

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
