# What Arqueo contributes

> **Español:** [`doc/APORTACION.md`](./APORTACION.md) is the original of this page, in Spanish; it
> prevails wherever the two differ. This translation is of commit `7766efb` (audit entry §917) and
> was produced by the project's assistant, a Claude Code session, in audit entry §918 (`GENAI.md`):
> it is a generated artefact, not text written by the author. Line citations of the form `file:N`
> refer to the Spanish files.

Compared with the current public/sovereign banking system and with
private blockchains, its contribution is not *replacing everything*, but
changing the basis of trust at one specific point: **settlement with
privacy and provable compliance**.

> ⚠️ **Correction with respect to the initial version of this document.**
> It said that privacy is *"against third parties who only see proofs"*.
> That is true but incomplete: **a counterparty is not a third party**.
> See §5.

---

## 1. With respect to the current public / sovereign banking system

| Dimension | Current banking | Arqueo |
|---|---|---|
| **Basis of trust** | Institutions, regulators, audits and legal processes | Verifiable cryptographic proofs + **declared** residual trust in the operator |
| **Privacy** | Opaque to the public; visible to the bank and, on demand, to the State | Against third parties who only see proofs. ⚠️ The operator sees the state, and **a counterparty sees your balance** (§5) |
| **Compliance** | Reports, inspections, access to the ledger | **Without handing over the ledger**: the holder proves band/minimum/balance; the supervisor verifies without a ledger |
| **Creation of money** | Monetary and institutional policy | Explicit rules in circuit: immutable cap, issuance with a threshold. **Value cannot be created without breaking the proof** |
| **Integrity of the history** | Internal controls and supervision | The operator **cannot rewrite it quietly** (chained log). It can censor and order |
| **Finality** | Legal and operational (T+N) | Cryptographic, per transition |
| **Transparency about limits** | Implicit or diluted in institutional complexity | Limits and residual powers **in the foreground** |

### Central contribution

It introduces **mathematical verifiability** into properties that today
rest on institutional faith and retrospective audit: conservation of
value, spending authorisation, no double spending and selective
compliance without total exposure of the ledger.

**It does not eliminate the State or the regulator**: it changes the kind
of evidence they can demand and verify.

---

## 2. With respect to private blockchains

| Dimension | Typical private blockchain | Arqueo |
|---|---|---|
| **Who validates** | A consortium of known nodes | Today: **single node**, declared |
| **Privacy** | Partial (channels, restricted access) or non-existent between members | Against third parties who only see roots and, since §538, against whoever sees a proof: no literal of the witness (§538) |
| **Compliance** | Usually requires privileged access to data | Selective disclosure **without access to the ledger** |
| **Setup ceremony** | Depends on the stack | **Forbidden** as a sovereign dependency |
| **Integrity** | Consortium consensus | Proven transitions; history not rewritable in secret |
| **Trust model** | In the set of validators | **Minimal and explicit**: what depends on the operator is named |
| **Minimalism** | Contracts, complex governance, permissions | Narrow layer + invariants + selective audit |

### Central contribution

Most permissioned chains **shift trust from the bank to the consortium**.
This shifts it towards verifiable proofs, and **refuses to hide the
operator's residual power**.

It also rejects backends that reintroduce unauditable ceremonies, **even
if they are faster**: Groth16 produces proofs 380-440 times smaller than
the layer's send and claim ones, hidden since §538, and it was discarded.
⚠️ **Corrected in §698**: it said «320 times», which does not come out of
today's sizes.

It is not *"another private blockchain with ZK"*. It is a settlement
layer with provable properties and a design ethic: **not selling
sovereignty where there is still an intermediary**.

---

## 3. Specific contribution

**Compliance without a general ledger.** The supervisor verifies band,
minimum or exact balance without seeing the full state. Rare both in
core banking and in private networks. Between §523 and §538 the proof
did not hide its witness —the audit one published the exact balance and
the spend key, and the supervisor learnt more than the mode asked for—;
since §538 (RFC-0009 E3b-2) it hides it and publishes no literal of
either. ⚠️ §916: and since `zkssl/0.5` the proof's view is simulated
without the witness, under named hypotheses (RFC-0009 D-A).

**Absence of ceremony as a sovereignty decision.** It is not a technical
preference: if the setup allows creating invisible money, it is
discarded.

**Structural honesty.** It declares that the operator sees balances and
can censor. Neither banking nor most private chains state their power
that way.

**Key / node separation.** The spend key does not travel to the operator
through the API; the proof is generated on the client. Between §521 and
§538 the proof published it —winterfell 0.13 did not hide the witness,
and the node receives every proof—; since §538 the prover hides the
witness and the key does not come out literally (RFC-0009 E3b-2; see
`SECURITY.md`).

**Empirical comparison on a real application.** Not SHA-256 benchmarks:
the same settlement circuit on five backends, with findings that only
appear when porting state.

---

## 4. What it does NOT contribute

- It does not replace monetary policy, macro supervision or the legal
  framework.
- **It is not today a decentralised network.**
- It does not eliminate the operational intermediary.
- **It is not more sovereign than a central bank for using ZK**: it is
  more verifiable in a subset of properties.
- **It is not audited.** Neither partially nor at any scale.

---

## 5. ⚠️ The privacy limits this document omitted

Found by applying the question *"what does each participant learn?"* to
code that already worked. Detail in [`AUDITORIA.md`](../AUDITORIA.md).

**The payer sees the receiver's balance.** Settlement updates both
leaves, so whoever builds the proof needs the receiver's balance.
**Paying someone one euro reveals how much they have.** Correction
analysed —transfers by notes— not implemented.

**The bank links offline payments.** The same commitment appears in the
issuance and in the spend: the bank knows who paid whom. **It is not
privacy like cash.**

Neither was declared when the first version of this document was
written. **A counterparty is not a third party**, and the distinction was
not a nuance.

---

## Summary

Against current sovereign banking, it contributes **cryptographic
evidence where today there is institutional faith**.

Against private blockchains, it contributes **privacy with provable
compliance and an explicit rejection of hidden trust** —ceremonies,
undeclared powers— instead of merely changing intermediary.

And its limit, said without softening: **there is still an operator who
sees everything, counterparties who learn more than they should, and no
external audit.**
