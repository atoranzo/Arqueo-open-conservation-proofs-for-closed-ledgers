> **Document of the ZK-SSL stage (until September 2026).** It is kept as it was, as history:
> its statements in the present tense describe the project of that time, not today's. The current
> position is in [`README.md`](../../README_EN.md) and [`doc/USE_CASES.md`](../USE_CASES.md); what
> changed since then is in [`AUDITORIA.md`](../../AUDITORIA.md) (§448).

# ZK-SSL — Positioning in one page

> **Español:** [`doc/historia/POSICIONAMIENTO.md`](./POSICIONAMIENTO.md) is the original of this
> page, in Spanish; it prevails wherever the two differ. This translation is of commit `7766efb`
> (audit entry §917) and was produced by the project's assistant, a Claude Code session, in audit
> entry §918 (`GENAI.md`): it is a generated artefact, not text written by the author. Line
> citations of the form `file:N` refer to the Spanish files.

**What it is.** The verifiable cryptographic core of a sovereign
settlement: monetary properties provable in circuit
(conservation of value, spending authority, issuance cap, no
double-spending, expiry of pendings with a double lock), supervisable
compliance **without revealing the book**
(exact / minimum / band disclosure), and **without a setup ceremony**
(STARK/FRI, post-quantum). Five backends measured on the same real
circuit (`FIVE_BACKENDS.md`); performance and suites in
`PERFORMANCE.md` and in the log (`AUDITORIA.md`).

**What sets it apart.** The combination that resolves the classic
confidentiality↔supervision tension: the supervisor verifies proofs, it
does not read state. And a **radically honest** documentation: every
residue of trust is measured, named and with its attack designed
(`SECURITY.md`, `doc/CONFIANZA_RESIDUAL.md`).

**What it is NOT yet.** A production system: today it is a single node
—the operator sees the state; privacy is against third parties—, without
consensus/replication, without an external audit, and with the
performance of anchoring as explicit scaling work (`doc/ESCALADO.md`).
The old paths pointed out have already been withdrawn; their inventory
remained in (BACKLOG 32/33 ✅, Campaigns A and B).

**Direction.** *Consensus is the last intermediary*: this project
leaves it minimal, measured and with evidence — ceremony eliminated,
custodian keys outside the node, wrapped leaf (entry 50 closed),
acknowledgement designed (§121). Quantum money may remove it entirely;
this is the classical approximation available today (`PRINCIPIOS.md`
§6.bis).

**Proposed conversation.** A bounded and verifiable pilot: e.g.
supervision of limits or reserves by means of **band proofs** on a
low-volume instrument, in a closed environment, with a dual run —
preceded by a formal external audit of the AIR and the circuits.

*Angel Toranzo Portela · MIT / Apache-2.0 · public code and log*
