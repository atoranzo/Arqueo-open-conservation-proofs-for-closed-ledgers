# From Arqueo to `mtc-core`: the extraction

> **Español:** [`doc/MTC.md`](./MTC.md) is the original of this page, in Spanish; it prevails
> wherever the two differ. This translation is of commit `7766efb` (audit entry §917) and was
> produced by the project's assistant, a Claude Code session, in audit entry §918 (`GENAI.md`): it
> is a generated artefact, not text written by the author. Line citations of the form `file:N` refer
> to the Spanish files.

`mtc-core` is the core of a certification authority for **Merkle Tree
Certificates** (`draft-ietf-plants-merkle-tree-certs`, the IETF's PLANTS group)
that was born inside this repository, in the `mtc/` directory, and was extracted
to a repository of its own: <https://github.com/atoranzo/mtc-core>.

## Why it was extracted

Arqueo proves conservation in closed ledgers, with STARKs and without a
ceremony. A web certificate CA proves no conservation and uses no ZK: it shares
with Arqueo the tree infrastructure and the discipline, not the problem. Having
it here as a separate workspace was a patch so as not to break the canon; having
it in its place gives each project a faithful image of what it is. It is the
same path `hbs-state` followed.

## What it took from Arqueo

| piece of Arqueo | in `mtc-core` |
|---|---|
| `zk-ssl-verify::mmr` (MTH, PATH, SUBPROOF of RFC 6962) | `subtree`, translated to SHA-256 and extended to subtrees |
| `zk-ssl-verify::inclusion` (leaf → root → signed head) | `proof` and `verify` |
| `zk-ssl-node::firma_cabeza` (reserve, sign, self-verify) | `cosign::Cosigner` |
| `zk-ssl-guardian` (already `hbs-state`) | `guard`, as a dependency |
| the idea of `zk-ssl::sparse_tree` (cached nodes) | `log`, over an *append-only* tree |
| the rule of `zk-ssl-hash`: one definition per format | the whole crate |

Nothing of the STARK layer, nor of the winterfell fork, nor of the accounting
state machine travelled with it.

## Where the history is

The extraction kept the commits of the `mtc/` directory with `git subtree
split`: the new repository starts at the history that ends here, with its
messages, its locks and its provenance trailers. The author published that
history on 2026-09-30 as `main` of `mtc-core` (twelve commits, the last one
`a4f9f39`; today `21836fb`, because on 2026-09-30 the author rewrote that history to remove the co-authorship trailer that GitHub showed as co-authorship, and the table of hashes before and after is in entry 16 of mtc-core's audit register), and
at the same moment the `mtc/` directory was deleted from here: this note
remains. The `mtc-core-main` branch of this repository was the split that was
pushed; the author deleted it afterwards, because the whole history already
lives in `mtc-core`.
