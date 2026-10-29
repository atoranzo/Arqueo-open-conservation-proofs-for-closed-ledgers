# Sources of the preprints

> **Español:** [`doc/preprints/README.md`](./README.md) is the original of this page, in Spanish; it
> prevails wherever the two differ. This translation is of commit `7766efb` (audit entry §917) and
> was produced by the project's assistant, a Claude Code session, in audit entry §918 (`GENAI.md`):
> it is a generated artefact, not text written by the author. Line citations of the form `file:N`
> refer to the Spanish files.

The text published on Zenodo, in markdown, with the stylesheet and the
script that turn it into the deposited PDFs.

It is here because the three papers say the work is **reproducible from
the artefact**, and until this commit the published text was not in the
artefact. A claim of reproducibility that does not include what was
published is an unchecked claim.

## The three

| File | Title | DOI |
|---|---|---|
| `ZK-SSL-preprint.md` | Comparative Implementation of a Zero-Knowledge Settlement Layer across Five Proof Systems | [10.5281/zenodo.21736125](https://doi.org/10.5281/zenodo.21736125) |
| `ZK-SSL-policy-note.md` | Provable Compliance without Full Ledger Disclosure | [10.5281/zenodo.21736082](https://doi.org/10.5281/zenodo.21736082) |
| `ZK-SSL-residual-trust.md` | From Institutional Trust to Verifiable Properties | [10.5281/zenodo.21905595](https://doi.org/10.5281/zenodo.21905595) |

## How a PDF is regenerated

```bash
./generar.sh ZK-SSL-preprint
```

It needs `pandoc` and `wkhtmltopdf`. The deposited PDFs were made with
**wkhtmltopdf 0.12.6**; another version may paginate differently without
changing the content.

## Provenance, which matters for reading this properly

⚠️ **These sources are a reconstruction, not the original.** The first
two revisions were typeset from an HTML that was not kept, and when the
third revision came the text had to be **reconstructed from the text
layer of the published PDFs**, the corrections applied and the whole
typeset again. Hence the PDFs of the third revision paginate differently
from those of the second.

Consequences a reader should know:

- The content of the third revision **did** come out of these files:
  they are the real source of what was deposited, not a later
  transcription.
- Against the second revision some italic, footnote or formatting nuance
  that the extraction did not recover may have been lost. The text was
  reviewed, but the check was human, not mechanical.
- From here on the problem disappears: the fourth revision will be a
  patch on these files.

## Cross-references: corrected

~~The cross-references point to earlier versions.~~ **Corrected on
01-08-2026** (entry **16**): the seven citations —five cross-references
and **two in which a preprint cited itself with its first-revision
DOI**— now point to the third ones.

⚠️ Entry 16 spoke only of the cross-references; the two self-citations
were found while doing it.

## Status of the deposits

The files in this directory carry the corrections of the fourth revision
(entry **28**): notes on the two properties the design guaranteed and the
implementation did not enforce, the incomplete fix of the identity
width, the **MiB** unit, and —in `ZK-SSL-residual-trust.md`— section
§4.7.

✅ **`ZK-SSL-residual-trust.md`: fourth revision deposited on
2026-08-12** — [10.5281/zenodo.21905595](https://doi.org/10.5281/zenodo.21905595) —
as a new version of the same record. It adds on top of what was already
written: the closure of §4.7 (salt derived from the key, with the
sequence visible), the new section §4.8 (the evidence layer, measured,
with its four residuals), the row of §4.1 that entry 1 of `ERRATA.md`
demanded, figures re-measured on `7ad62a9`, and the withdrawal of the
metadata appendix. ~~This file of the tree IS the exact source of what
was deposited.~~

~~⚠️ **`ZK-SSL-preprint.md` and `ZK-SSL-policy-note.md` are still written
and not deposited**: their DOIs above point to the third revisions,
which are the ones a reader receives today. And their citations of
*residual-trust* still point to the third — same class as entry 16, for
their own revision.~~

✅ **Corrected on 2026-08-27.** Both WERE deposited since 2026-08-01,
checked by opening the records: `ZK-SSL-preprint.md` at
[10.5281/zenodo.21736125](https://doi.org/10.5281/zenodo.21736125) and
`ZK-SSL-policy-note.md` at
[10.5281/zenodo.21736082](https://doi.org/10.5281/zenodo.21736082).

⚠️ **The record of `ZK-SSL-policy-note.md` has its files RESTRICTED**:
the record page is public and the PDF cannot be downloaded without an
account.

⚠️ **The DOIs of the three files in this directory were updated on that
same date, lifting for that cut the suspension of entry 28** and the
rule of `tools/check_cifras.py` that kept them verbatim. **Only
identifiers were touched**: not one figure, not one claim, not one
paragraph of the texts. **Declared consequence: since that commit the
three files are no longer byte for byte the deposited PDFs** — they
differ in the DOI lines —, and that is why the struck-through sentence
above no longer holds. The next deposit synchronises them again.

⚠️ **On 2026-09-05 (`AUDITORIA.md` §403) the repository URL was also
moved in the three files**, by the same rule as the DOIs: only
identifiers, not one figure nor one paragraph. Since that commit they
differ from the deposited PDFs in the DOI lines **and in the URL
lines**; the rest stays the same. What the reader of the PDF receives is
stated in `ERRATA.md`, entry 4. The next deposit synchronises them
again.
