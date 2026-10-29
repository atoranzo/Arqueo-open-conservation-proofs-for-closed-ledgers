# ECST — state transitions with built-in evidence

> **Español:** [`doc/ecst/README.md`](./README.md) is the original of this page, in Spanish; it
> prevails wherever the two differ. This translation is of commit `7766efb` (audit entry §917) and
> was produced by the project's assistant, a Claude Code session, in audit entry §918 (`GENAI.md`):
> it is a generated artefact, not text written by the author. Line citations of the form `file:N`
> refer to the Spanish files.

> **English:** `ECST_EN.md` is the report in English. The verification register and the working
> material are in Spanish.

A technical report, **in draft and not deposited**, on *Evidence-Carrying State Transitions*: a
discipline for composing per-transition proofs, hash chaining, signed heads and reconciliation of
the physical state, with two instantiations by the same author —HBS-STATE (repository `hbs-state`)
and Arqueo (this one)—.

⚠️ **Provenance.** The report comes out of several AI-generated drafts that the author contributed
and that mixed reasonable ideas with invented figures, code facts and citations. Before writing a
line, their 305 claims were verified against the two trees and the literature, and the report only
collects what withstood it. Everything in this directory was produced by a generative AI assistant
(Claude, by Anthropic) and is **pending the author's acceptance** that `GENAI.md` describes.

⚠️ **Integration (§594).** The directory enters `main` in §594, with the fix of its §8.1, by the
author's decision. It comes from the working branch `claude/awesome-pasteur-u08nq9`, which was NOT
merged: its commits carried the assistant as author, and here they were redone in the author's
name, with the same content. The hashes the material cites (`a7c72b9`, `6a868ed`, `78d71a4`…) are
from that branch; their content is the one here. «Integrar no es aceptar» (integrating is not
accepting): the acceptance of `GENAI.md` is still pending.

## What there is

| file | what it is |
|---|---|
| `ECST.md` | the report, in Spanish |
| `ECST_EN.md` | the same report, in English |
| `VERIFICACION.md` | the register: every claim of the drafts with its final verdict (CONFIRMADA, PARCIAL, FALSA, SIN_FUENTE, NO_VERIFICABLE), what can be claimed and its sources. It is generated whole from `borrador/final.json`. ⚠️ **Corrected in §847**: it said it is generated whole; the «Lo que se puede afirmar» [what can be claimed] line of M11 carries the `zk-ssl` pin by hand (see its header) |
| `referencias.bib` | the verified bibliography: only entries re-derived from consulted metadata |
| `borrador/` | the working material: the catalogue of claims, the results of each verifier and of each sceptic, the merge, and the reproduction (`borrador/REPRODUCCION.md`) |

## Base

Arqueo `main` at 71c5aad (S582); `hbs-state` at a960828 (crate 0.2.0, HBS-STATE specification v0.3).
Whatever changes in the trees after those commits is not collected: the `fichero:línea` paths age
with them.

## How the PDF is regenerated

Like the preprints of `doc/preprints/`, with `pandoc`; the bibliography is resolved by hand in the
«Referencias» section of the report itself, so `citeproc` is not needed.
