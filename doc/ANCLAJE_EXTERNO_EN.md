<!--
  ANCLAJE_EXTERNO — the head published where the operator cannot erase it.
  Design document (measure 10, §174). Sibling of CONFIANZA_RESIDUAL.md and
  of CADUCIDAD_PENDIENTE.md. The path of ELIMINATION of residue #1 (order and
  completeness) that the acknowledgement (§121) only bounds. It does not touch the circuit,
  does not touch the wire, does not touch the head: everything on the heads that already exist.
-->

# ANCLAJE_EXTERNO — a clock that neither the operator nor the issuer controls

> **Español:** [`doc/ANCLAJE_EXTERNO.md`](./ANCLAJE_EXTERNO.md) is the original of this page, in
> Spanish; it prevails wherever the two differ. This translation is of commit `7766efb` (audit entry
> §917) and was produced by the project's assistant, a Claude Code session, in audit entry §918
> (`GENAI.md`): it is a generated artefact, not text written by the author. Line citations of the
> form `file:N` refer to the Spanish files.

⚠️ **This text is a RECONSTRUCTION (§590, session 194).** The original document was born in §174 and
the tree published in S548 carried it as a one-line placeholder: the content was lost on publishing.
What follows is reconstructed from the record of §174 —which summarizes what it said— without
contradicting any of its statements, and is declared as a reconstruction, not as the original text.
Note §9 carries the detail.

## 0. The problem, with its record

The operator orders and can omit, and the log that binds it lives in its hands: it is **residue #1
(order and completeness)** of `SECURITY.md` §2.bis, declared from the first day. The pieces built
**bound** it: the signed head with its heartbeat (§115), the ceiling `N_max` of the acknowledgement
(§121), the reception receipt and its completeness envelope (RFC-0010). But they all share an
assumption: that someone EXTERNAL keeps custody of heads. The witness keeps custody of them **from
the moment it sees them** (TOFU, §245), and `CONFIANZA_RESIDUAL.md` §8 left two points open: the
**split view** —the operator signs different heads to different witnesses, each history internally
consistent, and «there is nobody placed to compare»— and the **RPO of the disaster** —what comes
after the last attested head is only recovered if the operator cooperates—. Measure 10 asked for the
interface that attacks them: **publish what the head already signs in an append-only medium
independent of the operator** (§174). This document is that interface.

## 1. The attack that kills the naive design — and the two inheritances

**Signing the anchor blows up the arithmetic.** An anchor with a signature of its own per epoch
would burn XMSS indices at the pace of the heartbeat: it is the same trap that §121.2 killed for the
acknowledgement («signature per acknowledgement, never»). **The anchor inherits the head's
signature: zero new indices.**

**Trusting gossip repeats the error of CT.** Certificate Transparency taught both faces: the pattern
works, and its piece for comparison between witnesses was under-deployed for years
(`CONFIANZA_RESIDUAL.md` §8). A design that requires witnesses that talk to each other inherits that
fragility. **The anchor eliminates it: against a public append-only medium, the split view is
SELF-INCRIMINATING without gossip** —two anchors of the same key for the same index are the
incrimination, and the operator itself publishes them or there is no anchor—. B10.7 turns the
comparison of B10.2 into **reading**.

**Rebuilding a tree of heads would be a second producer.** The head already signs «the root of a
tree of heads»: the MMR pair (`mmrRoot`, `mmrSize`), inside the digest since v3 (§291–§292), with
inclusion and consistency O(log N) in the verifier. **The anchor's batch is that tree, as it is**:
anchoring a head anchors the `mmrSize` earlier ones —the individual anchor of each one is a Merkle
path— and binds to the later ones by consistency. Nothing new is built or signed.

## 2. The mechanism

**The anchor** of a signed head is a six-field object, derivable by ANYONE —operator, witness or
holder— from the head alone:

```text
ancla = { v: 1,
          clave:       huella_de_clave(publicKey)   ← quién firma (32 B)
          indice:      índice XMSS EMBEBIDO         ← el reloj (§399, §567)
          epochDigest: lo que la firma cubre        ← la historia entera
          mmrRoot:     la raíz del árbol de cabezas ← el lote (§292)
          mmrSize:     cuántas contiene }
huella_del_ancla = ancla_digest(los cinco)          ← 32 B, lo que se publica
```

- **The index is the one EMBEDDED in the signature**, not the declared one: it is «the only one the
  signature attests» (§399), and it is the clock that a censor does not freeze (§567): the key only
  signs heads, and each signature burns an index.
- **The fingerprint fits in any medium**: 32 bytes. Whoever has the whole anchor recomposes it;
  whoever has only the fingerprint demands the anchor from whoever published it, and checks it.
- **The cadence is `M`**: one anchor every `M` heartbeats. `M` is a **declared systemic line** —the
  family of `N_max` (§121) and `T` (`CADUCIDAD_PENDIENTE.md`)—: it is chosen, published, measured.
  Proposal until there is a real operator: **`M` = 1,440 heartbeats** —one anchor a day, the horizon
  of `N_max` and the MMD of CT—.
- **The medium is NOT chosen here** (B10.6): a transparency log that accepts digests, a bulletin, a
  cheap public chain, a newspaper. What is required of the medium is a property, not a brand:
  **append-only, foreign to the operator, with an order readable by third parties**. Entry 86 of the
  BACKLOG carries what has to be measured before choosing: which logs accept arbitrary digests, with
  what cadence, at what cost.

**What a third party checks, without the node** (B10.7, the client-side verifier): that an anchor IS
the signed head it claims to be; that a later head EXTENDS an anchor (consistency against its
`mmrRoot`); that an earlier head IS in the batch of an anchor (inclusion, the Merkle path); and,
with two heads of the same key and the same embedded index with different digests, the **split view
with a name**: the condemning pair that `CONFIANZA_RESIDUAL.md` §2.1 promised, portable.

## 3. The window between anchors, declared

Between one anchor and the next the operator can tell different histories to different parties; the
next anchor forces it to choose ONE and to give itself away before whoever keeps custody of the
other. The exposure is the **tail between anchors**: bounded by `M` —and in amount by N2
(`CONFIANZA_RESIDUAL.md` §5.4)—, **not closed**. A short `M` makes the medium more expensive; a long
`M` lengthens the window. It is the same trade-off as `N_max` and `T`, and it is decided like them:
with data and published.

## 4. The first encounter, bounded

The anchor carries the **fingerprint of the key** inside. The first anchor of a key in an
append-only medium is a fact DATED BY THE MEDIUM and prior to any later encounter: the witness that
arrives late no longer pins the first thing it sees (TOFU, §245), but what the medium says the key
was BEFORE it knew it. It is the composition that entry 86 named: **the external anchor bounds the
window of the first encounter that TOFU leaves open**. What it does not do: the date is the medium's
word —displaced trust, §6—, and if the key was ALREADY lying before its first anchor, the anchor
pins the lie, just like TOFU (§246: what is missing is not the anchor, it is the checked custody of
the key).

## 5. What this design does NOT touch — with witnesses

- **No circuit** (B10.6/B10.7, «none touches a circuit», §174): the anchor is a composition of
  digests, not constraints.
- **The wire**: `zkssl/0.4` stays still. The anchor is derived from what `zkssl_signedEpochHead`
  already serves; no new method is needed.
- **The head and its signature**: nothing new enters the digest or the preamble; `VERSION_FORMATO`
  does not move. Zero extra XMSS indices.
- **The heartbeat** (§115): the cadence of the anchor is a multiple of the heartbeat, not a second
  clock.
- **The acknowledgement and the receipt** (§121, RFC-0010): the anchor does not replace them —it
  makes them enforceable against whoever did not keep custody of heads—.

## 6. What it does NOT resolve

- **The external medium is DISPLACED trust, not eliminated**: its append-only property, its
  availability and its dates are its own. If the anchor becomes a liveness dependency of the system,
  **it goes declared as such** (entry 86); in this design it is not one: without a medium, the
  system operates and the anchor simply does not exist, which is visible.
- **The tail between anchors stays bounded by `M` and N2, not closed** (§3).
- **The anchor proves WHICH root was canonical, not that its content was complete**: completeness
  still belongs to the receipts and to non-inclusion (§121.3, RFC-0010). An operator can punctually
  anchor a history that censors; what it can no longer do is tell another one to whoever checks the
  anchor.

## 7. Cost and first measurable step

A format (the anchor and its fingerprint, with its own domain), a client-side verifier over existing
pieces (the signature, the MMR), and the policy `M` declared. **First step: B10.6** —the format of
the anchor and `M`—, then **B10.7** —the verifier—. Choosing and measuring the medium is left for
when there is a real operator, and `SECURITY.md` §2.bis says so: **pending deployment**. The anchor
makes the last intermediary more expensive; it does not eliminate it (`SECURITY.md` §6).

⚠️ **Executed (§591–§593, session 194)**: B10.6 and B10.7 are **RFC-0012**
(`spec/rfc/0012-el-ancla-de-cabezas.md`): the domain `ANCLA_V1` and the compositions in the core,
the CLI's `tipo: "ancla"` envelope with the split view inside, its bench and its catalogue. The
deployment —choosing a medium, publishing with cadence `M`, measuring the cost— is still pending,
and entry 86 stays open because of it.

## 9. Reconstruction NOTE (§590)

The original of this document was written in §174 (2026-08-05) and did not survive the publication
of the tree: S548 (2026-09-25), the first commit of the public repository, carried it as a one-line
placeholder, and no entry noticed. This reconstruction (session 194) was written FROM the record of
§174 and the live citations of the tree —`SECURITY.md` §2.bis and §6, `CONFIANZA_RESIDUAL.md`
B10.6/B10.7, `CADUCIDAD_PENDIENTE.md` (`M` as a systemic line), entries 70, 83 and 86 of the
BACKLOG, `doc/DIAGNOSTICO_ESCALADO.md` §4— and claims nothing that those sources do not carry. If
the original turns up, the original rules and this note records the divergence.
