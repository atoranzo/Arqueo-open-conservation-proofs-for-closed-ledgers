# RFC-0000 — <short title>

> **Español:** [`spec/rfc/0000-plantilla.md`](../0000-plantilla.md) is the original of this page, in
> Spanish; it prevails wherever the two differ. This translation is of commit `7766efb` (audit entry
> §917) and was produced by the project's assistant, a Claude Code session, in audit entry §918
> (`GENAI.md`): it is a generated artefact, not text written by the author. Line citations of the
> form `file:N` refer to the Spanish files.

- **Status:** DRAFT
- **Author:** <who>
- **GenAI assistance:** <«none», or «Claude, in a Claude Code session in the cloud», with the
  entries and what it did; never the identifier of a model> — see [`GENAI.md`](../../../GENAI.md)
  ⚠️ §856: it said «<model and version, or «none»>». Followed to the letter, it put the
  identifier of a model, which `GENAI.md` does not give and which the ATADO G of
  `tools/check_publicadas.py` rejects in the whole header. This mark does not pass to the new RFC.
- **Date:** <yyyy-mm-dd>
- **Protocol version affected:** zkssl/0.1
- **AUDITORIA entry(ies):** <## NNN. on sealing>

## Motivation

What real problem it solves. With a measurement if there is one.

## Design

The exact change: methods, wire types, errors. Diffs of
`spec/RPC.md` if applicable.

## Compatibility

Does it break the wire? If so: new version, and what happens to the vectors.

## Security

Effect on the API principle (the spending key does not travel) and on
the declared debts (out-of-band notice §21, single node, --dev).

## References

Entries, BACKLOG notes, code.
