# ECST — working material of the verification (complete on 2026-09-30)

> **Español:** [`doc/ecst/borrador/README.md`](./README.md) is the original of this page, in
> Spanish; it prevails wherever the two differ. This translation is of commit `7766efb` (audit entry
> §917) and was produced by the project's assistant, a Claude Code session, in audit entry §918
> (`GENAI.md`): it is a generated artefact, not text written by the author. Line citations of the
> form `file:N` refer to the Spanish files.

> ⚠️ **This is not the report.** It is the intermediate material for writing it: the verification,
> claim by claim, of the drafts on ECST (*Evidence-Carrying State Transitions*), HBS-STATE, the
> Internet-Draft `draft-toranzo-hbs-state` and the email to the CFRG, against this tree, against
> the `hbs-state` repository and against the literature. **It is not cited, not deposited and not
> taken as a source.**
>
> It was produced by a generative AI assistant (Claude, by Anthropic) in two Claude Code sessions,
> on `main` at `d531c80` (2026-09-28) and at `71c5aad` (2026-09-30). Nothing here has yet gone
> through the author's acceptance that `GENAI.md` describes.

## What there is

| file | what it is | status |
|---|---|---|
| `flujo-verificacion.js` | the script of the agent flow: **the catalogue of claims** of the drafts, in seven blocks (H, G, L, C, M, B, P), with the paths each verifier had to open | complete |
| `verificacion-hbs-state.json` | block H: the `hbs-state` crate and the HBS-STATE specification v0.3 | done |
| `verificacion-guardian.json` | block G: Arqueo's own guardian (`zk-ssl-guardian`) and its relation to HBS-STATE | done |
| `verificacion-history.json` | block L: chained log, epoch head, history, package, verification without the node, anchoring | done |
| `verificacion-custody.json` | block C: two-phase payment, custody, what a proof reveals, conservation, single use, recovery, operator powers | done |
| `verificacion-measures.json` | block M: the drafts' figures against the repository's measurements; formal verification; Nova | done |
| `verificacion-biblio.json` | block B: the bibliography, with the corrected citations (text and BibTeX) in `extra_facts` | done |
| `verificacion-ietf-logic.json` | block P: the IETF/IRTF process, the Internet-Draft as it is written, and the formal logic of ECST | done |
| `refutacion-hbs-state.json` | the sceptic of block H: it re-derives every verdict | done |
| `refutacion-guardian.json` | the sceptic of block G | done |
| `refutacion-history.json`, `refutacion-custody.json`, `refutacion-measures.json`, `refutacion-biblio.json`, `refutacion-ietf-logic.json` | the sceptics of L, C, M, B and P, against `main` 71c5aad; `refutacion-biblio.json` also brings the re-derived BibTeX entries (`bibtex_verified`) | done on 2026-09-30 |
| `delta-guardian-hbs.json` | G and H, verified on d531c80, re-checked against 71c5aad (S566–S582) | done on 2026-09-30 |
| `final.json` | the merge: one FINAL verdict per claim (sceptic and delta above the verifier); `../VERIFICACION.md` is generated from here | done on 2026-09-30 |
| `simulate-2026-09-30.log` | the `simulate` re-run on 71c5aad | done |
| `REPRODUCCION.md` | what was re-run on 2026-09-28, with its environment and its outputs | done |
| `simulate-2026-09-28.log` | the full output of one `simulate` (one run, deterministic sandbox keys) | done |
| `sonda-fsync.rs.txt` | the replica of the `fsync` self-check of `hbs-state` used for the table of `REPRODUCCION.md` | done |

Each `verificacion-*.json` has `findings` —`id`, `claim`, `verdict` (CONFIRMADA, PARCIAL, FALSA,
SIN_FUENTE, NO_VERIFICABLE), `severity`, `evidence`, `correct_statement`— and `extra_facts`. Each
`refutacion-*.json` has an adjudication per `id` (`agree`, `final_verdict`, `reason`, `evidence`).

## The final count (2026-09-30)

| block | claims | CONFIRMADA | PARCIAL | FALSA | SIN_FUENTE | NO_VERIFICABLE | the sceptic revoked |
|---|---|---|---|---|---|---|---|
| H | 58 | 24 | 17 | 8 | 8 | 1 | 1 |
| G | 36 | 15 | 12 | 7 | 2 | 0 | 0 |
| L | 39 | 10 | 17 | 9 | 3 | 0 | 1 |
| C | 33 | 10 | 15 | 6 | 2 | 0 | 4 |
| M | 30 | 6 | 13 | 5 | 6 | 0 | 1 |
| B | 58 | 30 | 8 | 17 | 0 | 3 | 2 |
| P | 51 | 9 | 23 | 16 | 2 | 1 | 2 |

All blocks already have their sceptic. The detail, claim by claim, is in `../VERIFICACION.md`.

## What is missing, in order

1. ~~The five missing sceptics~~ — done on 2026-09-30.
2. ~~The readable verification register~~ — `../VERIFICACION.md`, generated from `final.json`.
3. ~~The unified report~~ — `../ECST.md` and its twin `../ECST_EN.md`.
4. ~~Adversarial review of the report~~ — `revision-fidelidad.json` and `revision-rigor.json`; what
   was applied and why, in `revision-aplicada.json`.
5. **Pending, and it is the author's:** his acceptance (`GENAI.md`) and the corresponding entry in
   `AUDITORIA.md`.
