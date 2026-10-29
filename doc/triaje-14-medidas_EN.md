# Triage of the 14 measures (external documents) against the log

> **Español:** [`doc/triaje-14-medidas.md`](./triaje-14-medidas.md) is the original of this page, in
> Spanish; it prevails wherever the two differ. This translation is of commit `7766efb` (audit entry
> §917) and was produced by the project's assistant, a Claude Code session, in audit entry §918
> (`GENAI.md`): it is a generated artefact, not text written by the author. Line citations of the
> form `file:N` refer to the Spanish files.

**Origin.** The 14 measures are anchored to a press article
(«consensus is the last intermediary»), not to `AUDITORIA.md`. They
describe the project's direction but reflect a state of weeks ago.
Collated 2026-08-03. The house rule: nothing that reverts an entry
without reverting the entry explicitly. Citable summary in BACKLOG entry 70.

## GROUP 1 — ALREADY DONE or IN PROGRESS (implementing the measure = undoing work)
- **3a** «accessors demand a credential or are removed» → §129 decided the
  OPPOSITE: they are the operator's (97 legitimate uses); an
  `account_view_authenticated` is ADDED (49-A step 4), they are not crippled. Do NOT implement.
- **3b** «salt in native_leaf» → entry 50 / B13-B14, designed (§132).
- **1/2/11** «custodians via B, measure» → entries 32/33, design closed
  (§47/§51). A nuance the list does not see: migrating without resolving the
  strict order IDX_B-IDX_A-1 opens custodian double-use (§51).
- **5b** linkability of the pending → residue already recorded.

## GROUP 2 — CONTRADICT an entry
- **4a** «remove transfer()» → it is not even deprecated; its withdrawal IS entry 32.
- **4b/14** «update preprints» → §135 FORBIDS editing published ones; they go via errata.

## GROUP 3 — VALID and aligned
- **8** reorder the backlog by trust → it is already the de facto order.
- **9** tests of «what each participant learns» (operator/counterparty/third party),
  extending §16 → NEW, the best one: mechanism, not narrative. IMPLEMENT.
- **6** notes/UTXO → major architecture, its own session.

## GROUP 4 — NEW documentary ones (they do not touch soundness)
- **7** a «quantum money» section in the living docs (preprints via errata).
- **10** external anchoring interface → connects with the residual intermediary
  = order+completeness, which the acknowledgement (§119/§121) attacks.
- **12** update SECURITY.md/threat model.
- **13** public positioning document.

## Verdict
Of 14: ~4 already done or in progress with more rigour (3a/3b/1/2/11), 2 revert
entries (4b/14→§135, 3a→§129), and the new ones without a clash are 9 (the best)
and the documentary ones 7/10/12/13. Implementing them as a block would undo work.
