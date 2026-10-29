# Arqueo — Executive summary

> **Español:** [`RESUMEN_EJECUTIVO.md`](./RESUMEN_EJECUTIVO.md) is the original of this page, in
> Spanish; it prevails wherever the two differ. This translation is of commit `7766efb` (audit entry
> §917) and was produced by the project's assistant, a Claude Code session, in audit entry §918
> (`GENAI.md`): it is a generated artefact, not text written by the author. Line citations of the
> form `file:N` refer to the Spanish files.

For whoever has five minutes and is not technical. In two languages and shorter:
[`RESUMEN_BILINGUE.md`](./RESUMEN_BILINGUE.md). The project was formerly called ZK-SSL; it changed
its name, not its published identifiers (`zkssl/0.5` since entry 914, `zkssl/0.4` since 538,
`zk-ssl-*`). What this page says is verified against commit `8c02941` (§789), in §794. ⚠️ **§794**:
until here it said «against `main` at commit `3294986`», the one of §448; what the re-reading of
§794 corrected carries its mark where it is.

## What it is, in three sentences

An operator keeps a closed ledger —accounts, payments, issuances, retirements— and those who depend
on it cannot see it: members, holders, beneficiaries, counterparties. Today that conflict is settled
by a third party who opens the ledger: an auditor, a supervisor, a court; once a year; by sample.
Arqueo replaces the opening of the ledger with **proofs that the ledger did what its rules say**,
and part of it anyone checks **without the ledger, offline and without trusting the author**; the
conservation of the money and the authorship of each payment, today, only the node.

## What a third party can check today, measured

- **That a unit was used only once** inside the ledger, and that if two distinct ledgers accepted
  the same unit, it is **detected** with the two signed heads, without any node taking part.
- **That the history was not rewritten**: today's head extends yesterday's without erasing or
  reordering.
- **That an entry is inside**, with a receipt that does not depend on the operator.
- **That a refusal has a cause**: the rule travels with the rejection, and for the causes the
  rejection envelope covers, a third party checks without the node that it holds over what the
  head signs.
- **That nothing in flight is older than a given age**: the empty box, proved without the node.
- **That what the node received was resolved in time**: every operation the node evaluates through
  the holder's direct paths leaves a receipt under its signed head, and a portable envelope says,
  without the node, that it was resolved within its window —applied or rejected with a cause— or
  names the operator that did not resolve it. Its residues, declared: an operator that issues no
  receipt leaves no trace (RFC-0010); the batch and the pledge carry it since §611, and of them
  there remain the aggregator that does not forward the receipt and the pledge presented over an
  old state (RFC-0014). ⚠️ **Corrected in §794**: it said «and the batch and the pledge do not
  carry it (RFC-0010)». ⚠️ **§815**: the receipt is left by every operation the node JUDGES; an
  evaluation cut short —a crash, a panic, a failure of the store— leaves its number hollow and no
  receipt, and the resolution can also be the one declared by the operator without proof, the
  fourth state (`SECURITY.md` §3.15).

⚠️ **Corrected in §589**: until then this page said here «the completeness of a period […] is
listed as planned, not as done», and in «What is missing» asked for «the completeness of the
acknowledgements». Row 6 was measured in RFC-0010 (§556–§577) and the large documents were
corrected in §581; this summary had fallen behind.

## What the node checks, and a third party does not yet

- **That the money is conserved**: what was issued equals what is in the accounts plus what is in
  flight. Each payment proves in circuit that its balance changes by exactly the amount and that
  the supply does not change, and the node checks that proof before applying it; the aggregate
  —issued = accounts + in flight— is checked by the layer on reopening the ledger. What reaches a
  third party is the total issued that the head signs, not the account that squares it.
- **That only the holder moved their account**: it also goes inside the proof of each payment, and
  the node also checks it. The key never travels. Without the node, a third party only checks the
  pledge's.

The log keeps the digest of each of those proofs, not the proof, and the verifier kit does not
verify them again, nor those of issuance and refund.

⚠️ **Corrected in §696**: until then these two were in the list above, the one of what a third
party can check today —«That the money is conserved […] nor on reopening the ledger» and «That only
the holder moved their account: the operator cannot, and the key never travels»—, and «What it is»
said «a proof that the ledger did what its rules say, which anyone checks without the ledger,
offline and without trusting the author», with no exception.

## What it is NOT

It is not a chain: one node, one writer, no distributed consensus and no token. **The operator sees
every balance** and can omit an operation: without leaving a trace if it issues no receipt; if it
issued one, the completeness envelope names it (RFC-0010). Across ledgers it detects, it does not
prevent. It is not audited by third parties. Nobody uses it with real money. And it does not prove
solvency: it proves that the ledger is consistent with itself, not that its units exist outside it.

## How it is checked, without knowing how to program

With **the verifier kit**: one download, a one-line program and five checks on the machine of
whoever checks, offline. An evidence package that adds up. A tampered one that does not add up, and
the program says which rule was broken. The same unit published in two ledgers, detected with both
nodes off. A swap of ledgers, rejected by name. And what the node received under its signature,
resolved in time or with the operator that did not resolve it named. The program's fingerprints are
published next to the commit it comes from, and whoever does not trust the download can compile it
from that commit; that its binary comes out with the published fingerprint is only shown within one
same machine. Step-by-step script: [`doc/KIT.md`](./doc/KIT_EN.md).
⚠️ **Corrected in §823**: it said «and the program is reproduced from that commit: there is no need
to trust the download», without the machine condition; rebuilt from its commit on another one,
0.4.2 comes out with another fingerprint (`doc/KIT.md` §«Reproducir el kit, para quien no se fíe de
la descarga» [reproducing the kit, for whoever does not trust the download]).
⚠️ **Corrected in §794**: it said «four checks»; since 0.3.0 the kit has a fifth
(`doc/KIT.md`:5-12).

## Where it fits

Where the unit of account **is born and dies inside the ledger** and there is a third party who
cannot see it: deposit-return systems, guarantees of origin and emission allowances, community
currencies, custody of client funds, public subsidies where the fraud is double funding, registers
of rights and quotas, clearing between operators or between administrations. It does not serve a
central counterparty clearing house, whose problem is counterparty risk, nor is it a component of a
central bank digital currency. The cases, with which property each one resolves and which are
reviewed: [`doc/USE_CASES.md`](./doc/USE_CASES.md).

## What exists, measured

| piece | status |
|---|---|
| The engine | 22 crates in Rust —19 of our own and the three of the winterfell fork—; every change says in its entry which gates it passed, and the largest is the canon: the tests of the crates —19 in the seal, all 22 with `--completo`— and eleven tools that watch figures, citations, domains, the census of the core, the geometry, that no curve enters the production binaries and that what the protocol promises is not broken without its list saying so. ⚠️ **Corrected in §794**: it said «18 crates» (`Cargo.toml`:2-51 lists 22) and «every change goes through the canon (the tests of all the crates …)»; the seal leaves three crates for `--largo` and `--completo` (the `halo2-experiment`, `plonk-experiment` and `zk-core` rows of the table of `tools/canon.sh`, by their `nivel` column; ⚠️ **Corrected in §847**: it said `:128-130`, and the table moved them), and `GENAI.md` §«Fuera del paso 4» [outside step 4] says that a session declares the gates it ran, not always the whole canon. ⚠️ **Corrected in §835**: it said they watch «that no curve enters the production binaries»; the curves one does not see the TLS of `ureq`, which brings curves into the cli and the SDK for transport (entry 122 of `BACKLOG.md`). ⚠️ **Corrected in §838**: §835 said it does not see it; since §838 it does, and admits it only as the exception of §106 for the transport of the cli and the SDK, with its cost: the view key travels under a classical ECDHE |
| The protocol | `zkssl/0.5` (§914; ⚠️ §914: it said `zkssl/0.4`): 32 JSON-RPC methods, conformance vectors per version that since §692 are not rewritten, and a version that does not say which proofs a node accepts (§699); RFC 0002, 0003, 0004, 0006, 0007, 0008, 0009, 0010, 0011, 0012, 0014, 0016 and 0018 accepted (⚠️ §916: the train, until here in the other list); 0001, 0005, 0013, 0015, 0017, 0019, 0020, 0021, 0022 and 0023 proposed |
| The verifier | `zk-ssl-verify` 0.5.0, published as `arqueo-verify-v0.5.0` (§917; `zkssl/0.5`, the first break of the wire under the policy); ⚠️ §917: it said 0.4.4, published as `arqueo-verify-v0.4.4` (§726), which is still published for the envelopes of `zkssl/0.4`, a security fix with its advisory (`SECURITY.md` §3.18), rebuildable from the commit its `VERSION` names; with the same fingerprint, only shown within one same machine. ⚠️ §818: it said «cut (§815), a security fix; the published one is `arqueo-verify-v0.4.2`»; 0.4.2 is still published, no longer current. ⚠️ **Corrected in §823**: it said «reproducible from the commit its `VERSION` names», without that condition. ⚠️ §725: it said «0.4.3, published as `arqueo-verify-v0.4.3` (§818), a security fix with its advisory». ⚠️ §726: it said «0.4.4, cut (§725), a security fix (`SECURITY.md` §3.18); the published one is `arqueo-verify-v0.4.3` (§818), a security fix with its advisory»; 0.4.3 is still published, no longer current |
| The record | [`AUDITORIA.md`](./AUDITORIA.md): one entry per verified change, with its commit; what is corrected is marked, not erased |

## The underlying decision

The design was chosen by measuring the same circuit in five zero-knowledge proof systems
([`FIVE_BACKENDS.md`](./FIVE_BACKENDS_EN.md)). The fastest one and the one that produces the smallest
proofs, Groth16, was discarded because it demands a trusted ceremony whose participants, if they
collude, can **create money without leaving a trace**. STARK without a ceremony —only hashes, no
curves— was the only decision of the project taken against the numbers. Only hashes means without
curve assumptions, not a known level against a quantum adversary: that level is not measured
(`SECURITY.md` §«3.11 El nivel de las pruebas» [the level of the proofs]). Out of that work came
eight findings that were not in the comparative literature and six deposits with a DOI; what was
corrected afterwards is marked in its errata. ⚠️ **Corrected in §794**: it said «resistant to a
quantum adversary in its soundness», which that section contradicts since §698.

## What is missing for a real third party to rely on this

An external audit, which does not depend on more code. A custody of the signing key that is
**checked**, not only declared. An anchor prior to the first encounter between the witness and the
node. Distributed consensus is another discipline and is not the path of this project: the path is
demonstrable accountability, in the manner of *Certificate Transparency*, and its pieces are built.

## To continue

| | |
|---|---|
| Start | [`README.md`](./README_EN.md) |
| Check it without trusting anyone | [`doc/KIT.md`](./doc/KIT_EN.md) |
| Where it fits and where it does not | [`doc/USE_CASES.md`](./doc/USE_CASES.md) |
| Twenty-two questions | [`PREGUNTAS.md`](./QUESTIONS.md) |
| What is still open | [`SECURITY.md`](./SECURITY_EN.md) |
| Break it | [`AUDITORIA.md`](./AUDITORIA.md) |
| The paper | [`PAPER.md`](./PAPER_EN.md) |
