# Consequences of deploying an Arqueo-type system

> **Español:** [`doc/CONSECUENCIAS.md`](./CONSECUENCIAS.md) is the original of this page, in
> Spanish; it prevails wherever the two differ. This translation is of commit `7766efb` (audit entry
> §917) and was produced by the project's assistant, a Claude Code session, in audit entry §918
> (`GENAI.md`): it is a generated artefact, not text written by the author. Line citations of the
> form `file:N` refer to the Spanish files.

*Settlement with ZK privacy, provable compliance, no setup ceremony,
explicit limits.*

> Prospective analysis. It describes what **would happen** if something
> like this were deployed, not what this code does today. For that there
> is [`AUDITORIA.md`](../AUDITORIA.md).

---

## 1. By dimension

### Financial

**Positive**
- Less friction in audit: selective proofs instead of mass handover of
  ledgers.
- Lower intermediation and reconciliation costs.
- Less risk of opaque creation of value if the rules are in circuit.
- Traceability of the integrity of the history.

**Negative**
- Computational cost of generating proofs. **Measured**: 620 ms for a
  transfer, 32 ms for an offline payment.
- New operational complexity: keys, clients that prove, recovery.
- Risk of concentration in whoever operates nodes.
- Expensive transition from legacy banking cores.

### Economic

**Positive**
- It can make compliance cheaper and increase competition in settlement.
- It enables programmable money with bounded privacy.
- It reduces information asymmetries in audits.

**Negative**
- It does not solve credit, monetary policy or macro stability.
- Duality of systems for years.
- Benefits captured first by actors with technical capacity.

### Systemic

**Positive**
- Fewer mass replicas of sensitive data. **With distributed retention,
  the operator keeps 137 bytes per operation instead of ~65 KB**: ~472
  times less.
- A minimalist design more efficient than bloated networks.
- No ceremonies, fewer fragile dependencies on a global setup.

**Negative**
- Proof generation consumes energy and hardware.
- Risk of standardising a critical layer with few implementers.
- Real effects depend on scale; today it is demonstrative.

### Current crisis

**Positive**
- It answers the erosion of trust: verification instead of opaque faith.
- Compliance without total surveillance.
- It fits where not everyone trusts the same intermediary.

**Negative**
- In an acute crisis control may prevail over privacy.
- A single node is still a point of failure and censorship.
- **It can be used to harden selective control with a better technical
  story.**

### Technological

**Positive**
- It pushes setup-free and post-quantum standards.
- It separates proving (client) from application (layer).
- It forces discriminating tests and honesty about limits.

**Negative**
- Uneven maturity of libraries. **Four out of six silently allow an
  insecure setup.**
- Error surface in circuits, nullifiers, trees and APIs. **This project
  found six bugs in code that already passed its tests.**
- A false sense of security if the operator's residual power is ignored.

### Human

**Positive**
- Informational dignity: proving without undressing.
- A culture of explicit limits.
- Less forced custody of keys at the intermediary.

**Negative**
- It demands cryptographic literacy; risk of a new technical elite.
- **Losing the key is losing access.** Recovery through custodians
  mitigates it at the price of giving them power.
- It can increase anxiety: *"if it is mathematical, who answers for it?"*

### Geopolitical

**Positive**
- Operational sovereignty for whoever does not want to depend on
  foreign stacks.
- Verifiable compliance without handing over complete databases.
- Less dependence on ceremonies or global providers.

**Negative**
- Fragmentation of standards by blocs.
- States can demand exceptions that break the model.
- Tension between citizens' privacy and supervisory sovereignty.

### Paradigm shift

**Positive**
- From *"trust the institution"* to *"verify the property"*.
- From total transparency or surveillance to **provable minimal
  disclosure**.
- From absolute promises to systems with declared limits.

**Negative**
- **Risk of techno-solutionism**: believing that the circuit solves
  politics and power.
- Resistance to the change in the basis of legitimacy.

### Architectural

**Positive**
- A narrow layer, clear invariants, an API that does not demand handing
  over keys.
- Chained log, verifiable and encrypted snapshots.

**Negative**
- **Single node**: an architecture incomplete with respect to the
  principle.
- Complexity of state not readable by the operator versus usability.
- Integration with ISO 20022 and legacy is not trivial.

### Regulatory

**Positive**
- Selective audit that can be aligned with risk-based anti-money
  laundering.
- Better evidence of controls without mass access to data.
- Traceability of interventions through public counters.

**Negative**
- Current frameworks assume access to ledgers and centralised
  responsible parties.
- Legal ambiguity about ZK proofs as evidence.
- Risk of hostile regulation or of exceptions that break privacy.

---

## 2. SWOT

| Strengths | Weaknesses |
|---|---|
| Provable properties | **Single node** |
| No setup ceremony | Proof cost and operational maturity |
| Structural honesty | **Audited by nobody** |
| Key / node separation | Usability and key recovery |
| | **Counterparties see more than they should** (§5 of APORTACION) |

| Opportunities | Threats |
|---|---|
| Wholesale money with bounded privacy | Demands for back doors |
| Regulatory reporting by proofs | Capture by concentrated operators |
| Jurisdictions with technological sovereignty | More complete institutional stacks |
| Early post-quantum standards | Cryptographic or implementation errors |

---

## 3. PESTEL

| Factor | Reading |
|---|---|
| **Political** | Sovereignty versus control; fragmentation helps and complicates at once |
| **Economic** | Savings in compliance in the medium term; high transition cost |
| **Social** | Demand for privacy **and** for control of crime; unresolved tension |
| **Technological** | ZK matures fast; interoperability and talent are bottlenecks |
| **Ecological** | Energy impact of proofs; possible offset if it reduces replicas |
| **Legal** | Uncertain fit with money laundering, privacy, liability and evidentiary value |

---

## 4. Four quadrants

**Individual-interior.** It fosters responsibility over keys and limits.
It can generate lucid trust or **magical faith in the mathematical**.

**Individual-exterior.** New practices: key custody, verification of
proofs, operation of nodes. Harsher human errors.

**Collective-interior.** A culture of verification and of not selling
what has not been built. A clash with cultures of institutional opacity
or of techno-utopian storytelling.

**Collective-exterior.** A new layer in the payments stack. Redesign of
supervision and reporting. Risk of a dual architecture and of technical
concentration.

---

## 5. Synthesis

### What it brings if deployed coherently

A **partial** shift from institutional faith towards cryptographic
verification, with privacy against third parties who only see roots —the
proofs published their witness between §521 and §538, and since §538 they
hide it (RFC-0009 E3b-2)— and compliance without total nakedness of the
ledger — **provided the operator's residual powers are declared and
bounded**.

### What it does not bring

Neither automatic sovereignty, nor macroeconomic justice, nor political
neutrality, nor the elimination of power.

Without distributed consensus, external audit and a legal framework, **it
remains a powerful but incomplete vessel**.

### Condition for a positive impact

Keeping the founding honesty: **not selling as sovereign or trustless
what still depends on an operator, on human keys and on institutions that
authorise issuance and resolve conflicts.**
