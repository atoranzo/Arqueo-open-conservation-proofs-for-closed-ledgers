# Arqueo for reviewers — thirty minutes

> This page was written by the project's assistant, a Claude Code session, in audit entry §920
> (see [`GENAI.md`](../GENAI.md)): a generated artefact, not text written by the author. It
> summarises and points; wherever it and the page it cites differ, the cited page prevails. Most
> of the pages it cites are in Spanish and have an English twin
> ([`TRANSLATIONS.md`](../TRANSLATIONS.md)).

## 1. The thesis

An operator keeps a closed ledger: the unit of account is born and dies inside it, and the people
who depend on it cannot see it. Today that conflict is settled by a third party who opens the
ledger, once a year, by sample. Arqueo replaces the opening with **evidence the third party can
hold against the operator**: signed epoch heads, receipts and STARK proofs that a verifier checks
offline, without the ledger and without trusting the author. The longer statement, with every
exception, is the opening of [`README_EN.md`](../README_EN.md).

## 2. What is in force

- **Wire protocol:** `zkssl/0.5`, since audit entry §914 (the cut of RFC-0018, accepted in §916).
- **Verifier:** [`arqueo-verify-v0.5.0`](https://github.com/atoranzo/Arqueo-open-conservation-proofs-for-closed-ledgers/releases/tag/arqueo-verify-v0.5.0)
  (§917), the first break of the wire under the compatibility policy of RFC-0021. Its tarball
  fingerprint is in [`doc/KIT_EN.md`](KIT_EN.md), section 0.
- **What a third party checks today, and what only the node checks:** the table «What it
  guarantees and what it does not» of [`README_EN.md`](../README_EN.md). Conservation and
  authorship of each payment are proved in circuit and verified **by the node**, not yet by the
  third party's verifier; making them checkable by a third party is entry 115 of
  [`BACKLOG.md`](../BACKLOG.md).

## 3. The hiding promise

Since §916, for the twenty-three families with `zkssl/0.5`, the view of the honest verifier is
**simulated without the witness**, statistically in the programmable random oracle model, under
named hypotheses. The promise, in its own words, is in RFC-0009, D-A
([twin](../spec/rfc/en/0009-lo-que-revela-una-prueba.md)); the argument, with its simulator and
lemmas, in [`doc/ocultacion-hvzk_EN.md`](ocultacion-hvzk_EN.md), whose section 4 classes the
hypotheses:

- **measured on every canon run:** H1, H4 and H11;
- **proved:** H2, H5, H10 and, since the cut, H6;
- **assumed or of the model, measured by no test:** H3, H7 (the programmable random oracle; the
  fork and the construction are not audited), H8 and H9.

The argument has been reviewed adversarially by the session and by an agent
([`doc/ocultacion-hvzk-revision-agente_EN.md`](ocultacion-hvzk-revision-agente_EN.md)), and by **no
person outside the project**: that review is entry 136 of the backlog, open, and its dossier is
[`doc/ocultacion-hvzk-dosier_EN.md`](ocultacion-hvzk-dosier_EN.md). Not promised: anything against
a quantum adversary, the prover's side channels, anything outside the named hypotheses.

## 4. Check it in five minutes

Download the kit from the release page, check its fingerprint against the one published next to
its commit, and run, with the node and the network off:

```bash
sha256sum arqueo-verify-*.tar.gz && tar xzf arqueo-verify-*.tar.gz && cd arqueo-verify-*/
sha256sum -c SHA256SUMS
./zk-ssl-verify spec/vectors/paquete/posicion-v2.json                          # VERDE, exit 0
./zk-ssl-verify spec/vectors/paquete/rechazo-n-adulterado.json; echo "exit $?"  # names the rule, exit 1
./zk-ssl-verify spec/vectors/completitud/no-resuelta.json; echo "exit $?"       # names the operator, exit 1
```

The five checks, with the expected output of each, are in «Try it in five minutes» of
[`README_EN.md`](../README_EN.md#try-it-in-five-minutes); the whole script, in
[`doc/KIT_EN.md`](KIT_EN.md).

## 5. Reproduce the canon

The canon is the project's single gate: the tests of nineteen crates against their pinned counts,
eleven tools that watch figures, citations, domains, curves and the protocol surface, the
conformance catalogues, a second implementation in Python, and a reproducible build of the kit. With
stable Rust:

```bash
git clone https://github.com/atoranzo/Arqueo-open-conservation-proofs-for-closed-ledgers
cd Arqueo-open-conservation-proofs-for-closed-ledgers && bash tools/canon.sh --sello
```

It ends in `== CANON --sello: VERDE ==` or names what fails. Measured in the session that wrote
this page, on four CPUs: about 11 minutes with the build cache warm, 22 cold. The author runs it
on every change before it enters `main`; a reviewer without Rust uses the kit.

## 6. Limitations, in five lines

- The operator sees every balance; there is no privacy against the operator.
- An operation the node never acknowledged leaves no trace; one it acknowledged ends applied,
  rejected with proof, or with the operator named.
- Across two ledgers, a reused label is detected afterwards, never prevented.
- No third-party audit of the code, the fork of winterfell or the hiding construction; three
  cryptographic dependencies are pinned and not audited ([`SECURITY_EN.md`](../SECURITY_EN.md)).
- Soundness is 127 conjectured bits, classical; against a quantum adversary, not measured.

## 7. Map of the repository

| path | what is there |
|---|---|
| [`README_EN.md`](../README_EN.md) | the full front page: guarantees, status, reading order |
| [`SECURITY_EN.md`](../SECURITY_EN.md) | what is promised and what is not; measured problems; how to report |
| [`spec/README.md`](../spec/README.md) | the reader's guide to the specification |
| `spec/NUCLEO.md`, `spec/PAQUETE.md`, `spec/RPC.md` | the frozen core, the evidence package, the wire protocol |
| `spec/rfc/` (twins in `spec/rfc/en/`) | the decisions, each with its status: accepted or proposed |
| `spec/vectors/` | conformance vectors per version, each file with its fingerprint |
| `crates/zk-ssl/` | the layer: the ledger and its proofs |
| `crates/zk-ssl-verify/` | the independent verifier, the core of the kit |
| `crates/winter-*/` | the fork of winterfell 0.13.1 with the hiding construction |
| `tools/canon.sh`, `tools/*.py` | the canon and its gates |
| `tools/segunda/` | the second implementation, in Python |
| [`AUDITORIA.md`](../AUDITORIA.md) | the record: one entry per verified change; do not read it whole |
| [`BACKLOG.md`](../BACKLOG.md) | open and closed work, numbered |
| [`GENAI.md`](../GENAI.md) | how a generative AI assistant was used, and by what method |

## 8. Where to attack

`AUDITORIA.md` has a section, «16. Donde el autor tiene MENOS confianza» [where the author has
the LEAST confidence], written to be attacked first; it is in Spanish, like the record. The house's
words are in the [glossary](GLOSSARY.md).
