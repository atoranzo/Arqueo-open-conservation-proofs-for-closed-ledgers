# RFC process of the Arqueo protocol (`zkssl/0.N`)

> **Español:** [`spec/rfc/PROCESO.md`](../PROCESO.md) is the original of this page, in Spanish; it
> prevails wherever the two differ. This translation is of commit `7766efb` (audit entry §917) and
> was produced by the project's assistant, a Claude Code session, in audit entry §918 (`GENAI.md`):
> it is a generated artefact, not text written by the author. Line citations of the form `file:N`
> refer to the Spanish files.

A change to the PROTOCOL (what crosses the wire: `spec/RPC.md`,
`spec/openrpc.json`, the vectors of `spec/vectors/`) does not enter by
direct commit: it enters by RFC. This process is deliberately small —
what matters is that it is left written, numbered and decided.

## States

DRAFT → PROPOSED → ACCEPTED → FINAL · or WITHDRAWN at any point.

## Rules

1. One file per RFC: `spec/rfc/NNNN-titulo-corto.md`, consecutive
   numbering from 0001 (0000 is the template).
2. Every RFC declares COMPATIBILITY: if it breaks the wire, the version goes up
   (`zkssl/0.1` → `zkssl/0.2`) and the old vectors are kept under
   their version — they are never rewritten. Since §692 the canon checks it:
   `tools/check_vectores.py` against `spec/vectors/HUELLAS.sha256`, which
   only grows; a vector moves, with its bytes and its manifest line,
   to `spec/vectors/<versión>/`, and its old path is not reused.
3. Every RFC declares its effect on the API principle: **the spending
   key never travels**. An RFC that erodes it is born WITHDRAWN. Since
   §538 (RFC-0009 E3b-2) the prover hides the witness in what the E2 suite
   measures: zero literals in the 23 proofs with a row; between §521 and
   §538 it was not honoured, and the entries say so.
4. ACCEPTED demands: the spec updated + OpenRPC regenerated + vectors
   re-issued (or new ones under the new version) + green suites.
5. The entry of AUDITORIA.md that seals the change references the RFC by
   number; the RFC references the entry. Double thread, like everything here.

## Security flaws

The openness and the states assume that the RFC is discussed in the tree
before the change. With a security flaw —what `SECURITY.md` §5
asks to report in private— that would publish the flaw before its fix.
Two clauses resolve it. Neither loosens the five rules.

- **One's own fix is drafted in private and published at once.**
  If the fix touches the protocol, its RFC —or the amendment to the RFC the
  flaw touches— does not pass through the tree as DRAFT or as PROPOSED: it is
  drafted outside, with the code, and enters in the same seal as the fix,
  with its stage already built. It takes the free number on entering: a gap in
  the numbering would tell what is kept quiet. The rules hold the same: if it
  breaks the wire, the version goes up and the old vectors are kept under
  theirs (rule 2); a new RFC enters ACCEPTED only if it brings what rule 4
  asks; and the entry that seals it says it was drafted in private.
  If the flaw reaches the published kit, the release that fixes it and the
  security advisory come out with it (`SECURITY.md` §5).
- **The draft of a report to a third party does not enter the public tree.**
  A flaw that belongs to another project is reported to that project through its
  private channel, in coordinated disclosure. The draft, the correspondence and
  any reproducer are kept outside the public tree. What the
  tree can say while the deadline runs is that there is a report in
  progress, without how the flaw is triggered. It governs since §691: what entered
  before is not rewritten by this clause; it stays with its entry, and whatever
  is decided about it will go in a new one.
