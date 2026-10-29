# RFC-0021 — The compatibility policy: which surface is stable, what breaks it, with how much notice, and what keeps verifying afterwards

> **Español:**
> [`spec/rfc/0021-la-politica-de-compatibilidad.md`](../0021-la-politica-de-compatibilidad.md) is
> the original of this page, in Spanish; it prevails wherever the two differ. This translation is of
> commit `7766efb` (audit entry §917) and was produced by the project's assistant, a Claude Code
> session, in audit entry §918 (`GENAI.md`): it is a generated artefact, not text written by the
> author. Line citations of the form `file:N` refer to the Spanish files.

- **Status:** PROPOSED (§790) — the text, the enumerated surface, the break rule, the amendment it
  proposes to `spec/RPC.md` and its eight decisions, written on what was measured in the tree at
  `8c02941` and in §699. The author accepted it in §799, with its eight decisions TAKEN, in each one
  the one the text recommends: see «Decisions (the author's, §799)». E2 and E3a, built in §801; E3b
  and E4 arrive with the cut, and ACCEPTED with them, because moving to ACCEPTED requires building
  its stages (rule 4 of the PROCESO). ⚠️ §799: until here it said «No decision is taken and no stage
  built: adopting it is the author's». ⚠️ §801: until here it said «No stage built: ACCEPTED arrives
  with them». ⚠️ §914: E4, the `zkssl/0.5` cut, built; E3b moves to `zkssl/0.6` by the author's
  decision (§913), so ACCEPTED is still waiting for it. ⚠️ §916: the author accepted the cut and E4
  with RFC-0018 («Aceptar todo» [accept all], 29-10-2026); this RFC stays PROPOSED by rule 4, until
  E3b.
- **Author:** Ángel José Toranzo Portela
- **GenAI assistance:** Claude, in a Claude Code session in the cloud (§790): it measured the tree
  and wrote the draft, outside step 4 of `GENAI.md` — see [`GENAI.md`](../../../GENAI.md)
- **Date:** 2026-10-14
- **Protocol version affected:** `zkssl/0.4` — **it changes no byte and no verdict**: it says
  how they change. Its first application will be the `zkssl/0.5` cut of RFC-0018. ⚠️ §799: until
  here it said «would be».
- **AUDITORIA entry(ies):** §209 (the `0.2`) and §275, §278 and §281 (its re-issues), §290 (what is
  custodied does not expire), §418 (the unknown `tipo`, by name), §641, §680 and §684 (the proofs
  that changed under `zkssl/0.4`), §662 to §664 (0.4.2), §682 (the first move to `0.4/`), §692 (the
  fingerprints), §699 (what `zkssl/0.4` does not identify, measured), §707 (D-G), §714 (the version
  of the witness's journal, 5 with the `bifurcacion` class, in «Outside the promise», with its
  mark), §716 (the issuance of the vectors, which since then writes the version of the scenario,
  with its mark in fact 5), §725 (the six line citations to `doc/KIT.md` that the notice on 0.4.4
  shifts, with their marks), §726 (the six line citations to `doc/KIT.md` that the paragraph on
  0.4.3 shifts, with their marks), §727 (this list, and the five line citations to `doc/KIT.md` that
  the lines of the notice on 0.4.4 in the paragraphs on 0.4.1, 0.4.0 and 0.3.0 shift, with their
  marks), §790 (this draft), §799 (the author's acceptance, with its eight decisions), §800 (the
  figures of S3 and S5 in D-A, with the E2 of RFC-0020), §801 (E2 and E3a), §807 (the stale
  sentences, with their mark), §810 (the citation of the «3 ter» block of `tools/canon.sh`), §823
  (the machine condition in «reproducible», and the citations of `doc/KIT.md` that §815 and §818 had
  shifted, with their marks), §826 (since when the promise holds, made precise, and which verifier
  judges the positives of E3b), §852 (the citation of `spec/vectors/paquete/MANIFIESTO.txt` in fact
  7, which its five negatives shifted, with its mark), §862 (the citation of the mould of
  `COFIRMA_V_MAX` in R2, which the E3 of RFC-0020 shifts, with its mark), §865 (RFC-0023 in the
  `zkssl/0.5` cut, with its marks in the E4 row of the table of stages and in «Compatibility»).
  ⚠️ §727: the list ended at §862 and was silent on §714, §725, §726 and §865, which touched this
  file with their marks; they and §727 enter by their number, like §716. ⚠️ §862: the list ended at
  §852. ⚠️ §826: the list ended at §799, and was silent on §800, §801, §807, §810, §823 and §852,
  which touched this file: §801 is named by its status line; §800, by its figures of D-A; and §807,
  §810, §823 and §852, by their marks.
- **Backlog:** entry **90** (the compatibility policy, as a promise), which this RFC does not close
  until it is built; with entry **85** (a second implementation needs something to commit against),
  entry **82** (one break for several reasons), entry **118** (how a proof is identified) and entry
  **87** (the signature scheme, which RFC-0020 writes, accepted with this one in §799). ⚠️ §799:
  until here it said «which RFC-0020 proposes».

## State of the stages

| stage | what it delivers | does it break the wire? | state |
|---|---|---|---|
| E1 — the promise, written | this text: the surface, the break rule, the notice and what is guaranteed afterwards | no | **written (§790) and decided (§799)** ⚠️ §799: until here it said «proposed (§790)» |
| E2 — the normative section | the «Versionado y compatibilidad» [versioning and compatibility] section of `spec/RPC.md`, with the text of D-C, and the cross-references from `spec/PAQUETE.md` §6, `spec/NUCLEO.md` §3, `doc/KIT.md` §0 and the «Estado» [status] row of the two READMEs | no (text) | **built (§801)** ⚠️ §801: until here it said «unbuilt» |
| E3a — the gate of the surface | a tool of the canon loop that derives each surface from its producer and compares it with the list of the section (D-D) | no | **built (§801)**: `tools/check_superficie.py`, a provisional name the author confirms ⚠️ §799: until here it said «proposed, not written» ⚠️ §801: until here it said «unbuilt; **decided (§799), not written**» |
| E3b — the gate of the proofs that travel | one positive proof per family that travels, judged in every canon by the verifier of the node and of the kit (D-D) ⚠️ §826: the kit's, only in the kit's five families; those of the layer are judged by the node's, because the kit verifies no send, claim, burn or threshold proofs (`spec/rfc/0018-el-tren-0-5.md:110`), as the «Compatibilidad» of RFC-0017 says | not by itself | unbuilt; with the E3 of the D-G of RFC-0018, at the cut ⚠️ §913: the author decided on 29-10-2026 that the `zkssl/0.5` cut carries only the D-H of RFC-0018; this stage moves to `zkssl/0.6` (RFC-0018, the note of §913 in the D-H) |
| E4 — the cut, under the policy | the `zkssl/0.5` cut with its notice, its moves and kit 0.5.0, as the first break that complies with it | **yes**: that of RFC-0018, and with it those of RFC-0017, RFC-0019 and RFC-0020; which surface each stage breaks is said by the table of its RFC ⚠️ §826: it said «**yes** (that of RFC-0018)» ⚠️ §865: and that of RFC-0023 if it arrives built, as its decision 9 proposes, without agreeing it with whoever drives the train ⚠️ §913: the author left in the cut only the D-H of RFC-0018; those of RFC-0019, RFC-0020 and this one's E3b move to `zkssl/0.6` | **built** — §914: the `zkssl/0.5` cut, with its notice in `spec/RPC.md`, the moves to `vectors/0.4/` (R6 b, 78 vectors) and kit `0.5.0` (R3, R6 e): the first break under the policy; ⚠️ §916: the author approved it with the train («Aceptar todo» [accept all], 29-10-2026) |

## Motivation

Entry 90 says it so: «a protocol is its promise, not its history: which surface is stable, what
can break, with how much notice, and what is guaranteed to keep verifying after a
break» (`BACKLOG.md`, entry 90). It asks for a normative section with the surface enumerated and
the break rule written, and a gate that makes it true («Qué lo cierra» [what closes it]). Its last
line, from §699: «the promise, and how a proof is identified, stay open».

**What is already written, and where.** There are pieces of the promise, loose:

| piece | where | who enforces it |
|---|---|---|
| if a break changes the wire, the version rises and the old vectors are preserved under their own | rule 2, `spec/rfc/PROCESO.md:16-21` | `tools/check_vectores.py` against `spec/vectors/HUELLAS.sha256` since §692 (`HUELLAS.sha256:1-8`); the issuer respects it since §781 |
| the wire version rises when «the values that travel» change, not the size of the surface | `spec/RPC.md:1290-1295` («Versionado», in «Notas operativas»), corrected in §699 (`:1296-1298`) | nothing: the test `el_aviso_de_compatibilidad_va_con_la_version` binds the notice to the version (`crates/zk-ssl-wire/src/openrpc.rs:400`), not the rule |
| what is signed grows only by version; the set of head versions has one producer | `spec/NUCLEO.md` §«3. La regla de extensión, ejercida» [the extension rule, exercised] | the compiler, over `VersionCabeza` (`crates/zk-ssl-verify/src/lib.rs:181-210`) |
| what is custodied does not expire: the kit reads the v1 package and the v2 | `spec/PAQUETE.md:73` (§290) | the kit's catalogues in the canon |
| an unknown `tipo` is rejected by name | `crates/zk-ssl-verify/src/main.rs:183-221` (§418) | its tests and its catalogue |
| no kit accepts two versions of the wire | the D-E of RFC-0018, `spec/rfc/0018-el-tren-0-5.md:160-161` | nothing yet: it belongs to the cut |
| a new version of AIR, of profile or of mark is a new version of the wire | the «Regla de extensión» [extension rule] of D-G, `spec/rfc/0018-el-tren-0-5.md:334-340` | nothing yet: it belongs to the cut |
| an old kit stays published and rebuildable from its commit (⚠️ §823: it said «reproducible»: with the same fingerprint it is only shown within one same machine, and 0.4.2 rebuilt from its commit on another comes out with 13 of its 35 sections different) | `doc/KIT.md:131-187` (⚠️ §823: it said `:19-63`; §815 put in between the notice on 0.4.3, and §818, which makes it current, the paragraph on 0.4.2. ⚠️ §725: it said `:76-123`; §711 and §715 lengthened the paragraph on 0.3.0 by two lines, and §725 puts in the notice on 0.4.4 and a mark in that on 0.2.0. ⚠️ §726: it said `:125-177`; §726, which makes 0.4.4 current, puts the paragraph on 0.4.3 in front and lengthens that on 0.4.2 by one line. ⚠️ §727: it said `:131-184`; §727 gives one more line to the paragraphs on 0.4.1, 0.4.0 and 0.3.0, with the notice that affects them): 0.2.0, 0.3.0, 0.4.0, 0.4.1 and 0.4.2, all published | nothing requires it; `tools/artefacto.sh --check` checks in every canon that the tree reproduces its artefact on the machine that runs it and from its same path, in two `target`s (the «3 ter» block of `tools/canon.sh`; ⚠️ §810: until here it cited its lines, `:598-603`, which §798, §800 and §804 had already shifted to `:690-695`, and the fourteen that §810 puts into «3 duodecies» take them to `:704-709`: it is cited by its block), not that the published ones are still there; ⚠️ §823: it said only «that the tree reproduces its artefact», without saying on which machine nor from which path |

**What §699 measured, and is the starting point.** The practice was not always the promise:

1. **`zkssl/0.4` does not identify which proofs a node accepts.** The AIR of the proofs that
   travel changed three times without raising the version —in §641, §680 and §684—, and measured
   between nodes and clients of six points of the history that all answer `zkssl/0.4`, when §641
   or §680 lies between the two the send does not verify in either direction, with
   `VerificationFailed` («envio: InconsistentOodConstraintEvaluations») (`AUDITORIA.md` §699;
   `spec/RPC.md:1299-1316`). The version stayed because the conformance of `0.4` does not compare
   proof bytes (RFC-0009, D-AI).
2. **`spec/vectors/zkssl-0.2.json` was re-issued three times under its version**, in §275, §278
   and §281 (`spec/RPC.md:1319-1325`). Since §692 the fingerprint prevents it.

**And five more facts, measured in this tree, that a promise has to say:**

3. **The node changed head version without raising the wire**, four times: v3, «additive on the
   wire, a break in the parser: `zkssl/0.2` does not rise» (`spec/RPC.md:602-603`); v4, v5 and v6,
   with «`zkssl/0.3` does not rise» (`spec/RPC.md:623-624`, `:656` and `:678`); and the doc of
   `VERSION_FORMATO` says the same of v5 and v6 (`crates/zk-ssl-verify/src/lib.rs:165-170`). An
   earlier verifier rejects the new head, closed and by name; but what the node issues changed
   without the version saying so, which is the class of §699 applied to the heads. Of v6, besides,
   the two texts give the wrong version: v6 is from §570, and the wire was at `zkssl/0.4` since
   §538 (`spec/RPC.md:1290-1292`). They are not corrected here: the seal the author chooses will.
   ⚠️ §807: corrected in §807, with its mark, within the task list the author commissioned.
4. **Within `0.4.x` two fixes changed verdicts.** 0.4.2 rejects what 0.4.1 accepted: a `u64` with
   `+` or with leading zeros (§662), the same co-signature counted several times (§663) and the key
   with the alternate OID, which gave VERDE to a conflict with a single ledger (§664)
   (`doc/KIT.md:143-149`; ⚠️ §823: it said `:30-35`, which §815, with its notice on 0.4.3, and §818
   shifted. ⚠️ §725: it said `:87-93`, which the notice on 0.4.4 shifts. ⚠️ §726: it said
   `:136-142`, which the paragraph on 0.4.3 shifts); and 0.4.1 already rejected the proof with extra
   nodes inside a batch that 0.4.0 verified (§659, `doc/KIT.md:151-158`; ⚠️ §823: it said `:36-41`.
   ⚠️ §725: it said `:94-100`. ⚠️ §726: it said `:143-149`. ⚠️ §727: it said `:150-156`; the
   paragraph on 0.4.1 gains one line, with the notice that affects it, and this one another). It is
   the right thing, and no rule says so. The line of the kit has not followed that of the wire
   either: 0.3.0 was cut in §619, with the wire at `zkssl/0.4` since §538, and 0.4.0 (§657) rejects
   what 0.3.0 accepted (`doc/KIT.md:160-167`; ⚠️ §823: it said `:43-49`. ⚠️ §725: it said
   `:102-108`. ⚠️ §726: it said `:151-157`. ⚠️ §727: it said `:158-164`; the paragraphs on 0.4.1 and
   0.4.0 gain one line each, with the notice that affects them, and this one another) without the
   wire rising.
5. **The wire version is written in four places of the production code, and nothing binds them**:
   the node's answer (`crates/zk-ssl-node/src/main.rs:2822`), the OpenRPC twice
   (`crates/zk-ssl-wire/src/openrpc.rs:239` and `:269`) and the issuance of the vectors
   (`crates/zk-ssl-cli/src/conformance.rs:106`). The tests that check it write it once more
   as a literal (`crates/zk-ssl-node/src/main.rs:5129` and `:5162`; `openrpc.rs:355` and `:403`).
   Measured with `grep` of the literal in `crates/` and `tools/`. ⚠️ §716: the issuance of the
   vectors stopped being a writing of the wire: since §716 it writes the version of the conformance
   scenario, `zkssl/0.5`, which rises before the wire because the commitment of the two issuances
   carries the issuance counter (RFC-0018, «Cómo se construye» [how it is built]). The list of
   `spec/RPC.md` fixes it apart, in `escenario`, and `tools/check_superficie.py` compares it in its
   R1; the writings of the wire are left at three.
6. **The contract of the kit has four exit codes, and one of its documents counts three.**
   `codigo_de_salida` gives 0, 1 or 3, and incorrect usage 2
   (`crates/zk-ssl-verify/src/main.rs:1789-1795` and `:2937`); `spec/PAQUETE.md:1187-1190` says the
   four; `doc/KIT.md:201-203` (⚠️ §823: it said `:78-80`. ⚠️ §725: it said `:137-139`, which §711
   and §715 had already shifted by two lines, and the notice on 0.4.4 shifts further. ⚠️ §726: it
   said `:191-193`, which the paragraph on 0.4.3 shifts. ⚠️ §727: it said `:198-200`, which the
   three lines the paragraphs on 0.4.1, 0.4.0 and 0.3.0 gain with the notice that affects them
   shift), «exits 0 … 1 … 2».
7. **A v1 head is not verified by the kit, it is verified by the library.** The witness and the
   package reject `formatVersion` 1 (`spec/vectors/cable/MANIFIESTO.txt:17`,
   `spec/vectors/paquete/MANIFIESTO.txt:26`; ⚠️ §852: it said `:21`, and the five lines §852
   inserts after `:10` shift it down by five), and «a v1 head is verified with the library, not with
   this command» (`spec/PAQUETE.md:742-743`); the second implementation also recomposes it
   (`tools/segunda/juez_cabezas.py:51-68`). What entry 90 calls «a v1 head keeps verifying under a
   v2 binary» is true of the library and of the second one, not of the kit's binary.

## Design

### D-A — The stable surface, enumerated

Six surfaces. Each with what forms it, its identifier and its producer, measured today:

| | surface | what forms it | its identifier | its producer, today |
|---|---|---|---|---|
| S1 | **the wire** | the 29 `zkssl_*` methods, their parameters, their results, their errors with the cause in `data`, and the encoding (`spec/RPC.md`) | `zkssl/0.N`, the answer of `zkssl_protocolVersion` | four unbound writings (fact 5; ⚠️ §716: three, with the scenario's apart); `spec/openrpc.json`: `info.version` `zkssl/0.4`, 32 methods, 29 `zkssl_*` and 3 `dev_*` |
| S2 | **the proofs that travel** | the bytes the node and the kit judge | none under `zkssl/0.4` (§699); the v2 mark from `zkssl/0.5`, if D-G is built | the 23 provers with a row |
| S3 | **what is signed** | the head (v2 to v6), the co-signature and the act: their preambles and their compositions, with their KATs (31 in `spec/vectors/nucleo/`; 34 since §800, with the three of the E2 of RFC-0020, switched off) | `formatVersion`; the co-signature version; `ACTA_VERSION`; the scheme, if RFC-0020 is built | `VersionCabeza` (`lib.rs:181-210`), `spec/NUCLEO.md` §6 |
| S4 | **the envelopes of the kit** | the fourteen forms of `spec/PAQUETE.md` §«2. Las catorce formas» [the fourteen forms], with their catalogue (§5) and their causes (§6.1) | `v` (1 or 2) and `tipo` (twelve) | `crates/zk-ssl-verify/src/main.rs:171` and `:190-221` |
| S5 | **the vectors** | every file of `spec/vectors/`: 492 `.json` and 982 lines of 18 manifests, each with its fingerprint (495 `.json` since §800, with the three KATs of the E2 of RFC-0020) | its path, its manifest line and its fingerprint | `spec/vectors/HUELLAS.sha256`; `tools/check_vectores.py` |
| S6 | **the kit as a binary** | `arqueo-verify-vX.Y.Z`: its tarball, its binary and the contract of `spec/PAQUETE.md` §«6. El contrato del mando» [the contract of the command] —one argument, four exit codes, the VERDE lines and the ROJO with a cause from §6.1— | the tag, the commit and the two fingerprints, together (`doc/KIT.md:189-191`; ⚠️ §823: it said `:66-68`, which §815, with its notice on 0.4.3, and §818 shifted. ⚠️ §725: it said `:125-127`, which §711 and §715 had already shifted by two lines, and the notice on 0.4.4 shifts further. ⚠️ §726: it said `:179-181`, which the paragraph on 0.4.3 shifts. ⚠️ §727: it said `:186-188`, which the three lines the paragraphs on 0.4.1, 0.4.0 and 0.3.0 gain with the notice that affects them shift) | `tools/artefacto.sh`; its `FAMILIAS`, `tools/artefacto.sh:31` |

**Outside the promise**, and said so that nobody takes it as promised: the `dev_*`, which only
exist with `--dev`; the Rust API of the crates —the SDK, and `zk-ssl-verify` as a library—; the
command-line options of `zk-ssl-cli` and of the witness; the journals of the node (`DIARIO_VERSION`
1, `crates/zk-ssl-node/src/diario.rs:67`) and of the witness (5,
`crates/zk-ssl-cli/src/witness.rs:610`; ⚠️ §714: it said 4, at `:478`; the `bifurcacion` class of
§714 raises it); the ledger on disk and its snapshots; the operation limits, such as that of the
body of a request (`spec/RPC.md`, «Transporte y sobre» [transport and envelope]); times and sizes;
and `doc/`. The author took decision 2 in §799: these six surfaces, and outside the promise what is
enumerated here, the witness's journal and `zk-ssl-verify` as a library included. ⚠️ §799: until
here it said «Decision 2 is the author's».

### D-B — The break rule

**R1 — What a break is.** A change by which an implementation conformant with the earlier version
produces or accepts something else on a surface of D-A:

- a value that travels changes, **also the bytes of a proof or its AIR** —what §699 measured—, or
  what the node signs, also a new head version that the node **issues** (fact 3);
- a method, a field or a `tipo` is removed or renamed, or its type changes;
- the verdict or the pinned text of a published vector changes;
- a version leaves an accepted set: of head, of package, of mark or of scheme;
- an exit code or a VERDE line of the kit's contract changes.

**R2 — What is not one.** The additive that no published producer issues yet: a new method, an
optional field, a new `tipo`, a KAT or a vector with a new path, and a new version in an
**accepted** set that nobody issues —whoever reads can be ahead of whoever writes and never the
other way round (the mould of `COFIRMA_V_MAX`, `crates/zk-ssl-verify/src/lib.rs:308-314`; ⚠️ §862:
the citation is of the tree of the header, and at `213b0b0` the mould was already at `:422-428`;
since §862 it is at `:465-471`, and `Esquema::ACEPTADOS` and `Esquema::EMITIDOS`, the set of schemes
that is accepted and the one that is issued, follow the same mould)—. A pinned text that derives
from the accepted set does change on widening it, and that is R1: the vector moves (R6 b).

**R3 — What rises.** A break of S1, S2 or S3 raises `zkssl/0.N`, which comes to have **one**
producer from which the other writings derive (R7). One of S4 raises the `v` of the envelope or
enters as a new `tipo`. In S5 nothing breaks: it moves (R6 b). In S6, the kit raises its minor
version with the wire, `0.N.0`, and only then —the D-E of RFC-0018 says it of `0.5.0`; this
extends it, and changes the practice of 0.3.0 and 0.4.0 (fact 4)—; a `0.N.x` does not break S1 to
S4, except by R6 f.

**R4 — A break is paid once.** Those that are ready are joined in a cut, the corollary of entry 82
(«it breaks once for SEVERAL reasons together», `BACKLOG.md`, entry 82), like the train of
RFC-0018.

**R5 — The notice.** The RFC that breaks lives in the tree, PROPOSED, at least one seal before the
cut, with its table of stages saying which surface each one breaks; and the seal of the cut writes
the notice where the wire is read —the line under the title of `spec/RPC.md` (`:2`), the
description of the OpenRPC, `doc/KIT.md` §0 and the «Estado» [status] row of the READMEs—, as §699
did with what it had already broken. A security fix follows `spec/rfc/PROCESO.md` §«Fallos de
seguridad» [security flaws]: it is drafted in private and enters at once, with its notice of
`SECURITY.md` §5. How much notice is decision 4.

**R6 — What keeps verifying after a break.** It is what is promised:

- **a. No published kit is withdrawn**, and each one is still rebuildable from its commit
  (`doc/KIT.md` §«Reproducir el kit, para quien no se fíe de la descarga» [reproducing the kit, for
  whoever does not trust the download]); with the same fingerprint, only shown within one same
  machine. What it judged, it keeps judging the same. ⚠️ §823: it said «reproducible from its
  commit», without the machine condition: 0.4.2 rebuilt from its commit on another comes out with
  another fingerprint, and judges its eleven catalogues the same.
- **b. No vector is rewritten.** One whose verdict or text changes moves identical, with its
  manifest line, to `spec/vectors/<versión>/`, keeps there the verdict of the published kit of its
  version, and its old path is not reused (rule 2; `spec/vectors/HUELLAS.sha256:4-7`). It is what
  §682 did with two vectors of the completeness
  (`spec/rfc/0019-la-completitud-que-no-se-esquiva.md:54-57`).
- **c. A signed head that a kit accepted is accepted by every later kit**: the set of head
  versions only grows. v1, with the library and the second one (fact 7). The exception, if
  RFC-0020 is built: a head signed with a scheme that gets broken is worth something only contained
  in something signed with an accepted scheme (its D-E, rule 5).
- **d. An envelope that a kit gave as VERDE** is still given so by the kit of its version; a later
  one verifies it or rejects it by name in the class «fuera del contrato» [outside the contract] of
  `spec/PAQUETE.md` §«6.1 La causa del rechazo» [the cause of the rejection], never with another
  cause, except by R6 f.
- **e. A proof of `zkssl/0.N` is judged by kit `0.N.x`**: no kit accepts two versions of the
  wire, because accepting both opens a way to downgrade (the D-E of RFC-0018). A later kit rejects
  it by name, as D-G proposes (steps 1b and 1c of its table).
- **f. The exception: a security fix** can change a verdict within `0.N.x`. It comes out with its
  notice and the release says which verdicts it changes and why, like 0.4.2 (fact 4); the vectors
  that show it enter with new paths.

**R7 — Each surface, one identifier and one producer.** Two markers that can disagree are worth less
than one (§236). Today three are missing: the wire version has four writings (fact 5; ⚠️ §716:
three, since the issuance of the vectors writes that of the scenario);
a proof of `zkssl/0.4` has no identifier (the D-G of RFC-0018, entry 118); and a signature does not
say its scheme (RFC-0020, entry 87). While they are missing, the section declares them.

### D-C — The amendment to `spec/RPC.md`: the «Versionado y compatibilidad» section

`spec/RPC.md` already has a «Versionado» [versioning]: an entry of «Notas operativas» [operational
notes] (`spec/RPC.md:1290-1298`), with the two of §699 behind it (`:1299-1325`), and the line of the
notice, under the title, refers to it (`spec/RPC.md:2`). So this RFC does not write a section in
`spec/RPC.md`: it proposes its amendment, and E2 applies it; the author adopted it in §799 (⚠️ §799:
until here it said «E2 applies it if the author adopts it»). The new section goes **at the end of
the document**, after «Notas operativas», so as not to move any line that other documents cite —§699
left `:9`, `:88`, `:101` and `:164-165` where they were—. The «Versionado» entry is not deleted: it
gains on its same line a note «⚠️ §M:» that refers to the new section, and it keeps telling what
happened before. The text that is proposed:

> ## Versioning and compatibility
>
> This section is the protocol's promise to whoever implements it and to whoever custodies what it
> produces. It governs since §799, the seal in which the author accepted RFC-0021 with its decisions
> —the RFC remains PROPOSED until its stages are built (rule 4 of `rfc/PROCESO.md`)—, and it is
> written here since §801; what came before is told by «Versionado», in «Notas operativas», which is
> not rewritten. ⚠️ §826: it said «It governs since the seal that accepts RFC-0021». §799 accepted
> its decisions and left it PROPOSED; read as the seal that moves it to ACCEPTED, which arrives with
> its E4, the promise would govern from the cut that premieres it. But the author's decision 4 in
> §799 fixes the notice in a seal with the RFC PROPOSED before the cut (R5), and that only binds if
> the promise governs while the RFC is PROPOSED. §801 left it to the author to say which of the two
> seals it is; §826 makes it precise, by his delegation («HACER todo» [DO everything]) and with that
> criterion, and it is reversible. The same precision goes in this section of `spec/RPC.md` and in
> the D-C of RFC-0021, which proposes it.
>
> **The stable surface** is six surfaces, each with its identifier: (1) the wire, `zkssl/0.N`: the
> `zkssl_*` methods of `openrpc.json`, their parameters, their results, their errors with their
> cause in `data`, and the encoding; (2) the proofs that travel, by the mark of their trace; (3)
> what is signed —the head, the co-signature and the act—, by its version, with its preambles and
> its compositions (`NUCLEO.md` §6); (4) the kit's envelopes, by their `v` and their `tipo`
> (`PAQUETE.md` §2), with their catalogue and their causes; (5) the vectors of `vectors/`, by their
> path, their manifest line and their fingerprint; (6) the kit, `arqueo-verify-vX.Y.Z`, by its tag,
> its commit and its fingerprints, with the contract of `PAQUETE.md` §6. The rest —the `dev_*`, the
> crates' API, the command-line options, the journals, the ledger on disk and the operation limits—
> is not promised.
>
> **Breaks** whatever makes an implementation conformant with the previous version produce or accept
> something else in one of the six: a value that travels, also the bytes of a proof or its AIR, and
> what the node signs, also a new head version it issues; removing or renaming a method, a field or
> a `tipo`; the verdict or the text of a published vector; taking a version out of an accepted set;
> an exit code of the kit. **Does not break** what is additive and nobody issues yet.
>
> **Raises** `zkssl/0.N` with every break of (1), (2) or (3); the `v` of the envelope, or a new
> `tipo`, with one of (4); the kit moves to `0.N.0` with the wire and not before, and a `0.N.x`
> breaks nothing except by a security fix. Breaks are joined in a cut, and its RFC is in the tree
> before it.
>
> **After a break it keeps verifying**: every published kit, which is not withdrawn and is rebuilt
> from its commit (⚠️ §823: it said «and is reproduced from its commit»; with the same fingerprint,
> only shown within one same machine, as `spec/RPC.md` already says); every vector, which is not
> rewritten and, if its verdict changes, moves to `vectors/<versión>/` with that of its kit; every
> head that a kit accepted, in every later kit, because the set of head versions only grows, except
> that of a broken signature scheme, which is valid contained in something signed with an accepted
> one; every envelope, with the kit of its version, and with a later one it either verifies or is
> rejected as «outside the contract»; and every proof, with the kit of its wire version, because no
> kit accepts two. A security fix can change a verdict within `0.N.x`, with its notice and the list
> of what changes.
>
> **What the version does not identify yet** is declared here, with its entry, until it
> identifies it.

The figures of the section —how many methods, how many surfaces— do not go in the normative text,
but in the list that the gate of E3a compares, so that the prose does not have to chase them.

### D-D — The gate, proposed as a stage and not written

Entry 90 asks for «a gate that makes it true: what is declared stable does not change without the
canon saying so». This RFC does not write it: it was not accepted when it was written (§790), and
what it would watch is the section it proposes; accepted in §799, the gate arrives with its stage.
It proposes it in two stages. ⚠️ §799: until here it said «it is not accepted», nothing more.
⚠️ §801: E3a is built: `tools/check_superficie.py`, a provisional name, with the seven rules below
in its header, and the list it compares at the end of the «Versionado y compatibilidad» section of
`spec/RPC.md`.

**E3a — The surface, by its producer.** A tool of the canon loop, `tools/check_superficie.py` or the
name the author chooses, that reads each surface from its producer and compares it with the list
fixed for the version in force:

1. **the wire version**: one producer, and each production writing equal to it (four today, fact 5);
   and the section, the OpenRPC and the node's answer say the same one;
2. **the methods** of `spec/openrpc.json`, with the schema of the parameters and of the result of
   each one: one that is missing, or changes name or type with the same `info.version`, is ROJO; a
   new one is counted and is not ROJO (R2);
3. **the head versions**: `VersionCabeza::TODAS` only grows against the list, and `VERSION_FORMATO`
   is a member;
4. **the envelopes**: the `tipo` values of the kit's dispatch and the `v` values it reads only grow
   against `spec/PAQUETE.md` §2;
5. **the kit's contract**: the four codes of `codigo_de_salida` and of the usage, against
   `spec/PAQUETE.md` §6 and against `doc/KIT.md` §0 (⚠️ §801: it said «today they disagree, fact 6»;
   §801 corrected `doc/KIT.md` and `doc/KIT_EN.md`, and they no longer disagree);
6. **the vectors**: it does not repeat `tools/check_vectores.py`; it requires it in the same canon;
7. **proof of life in both directions**, like that of §787: a mutation of each list turns it red,
   and so does an empty list.

It is Python over the tree, without compiling, and its cost is that of the loop's tools.

**E3b — The proofs that travel.** No text gate sees that an AIR changed: that was §699. What sees it
is one positive proof per family that travels —the five the kit judges and the fourteen the layer
judges—, kept as a vector under its version, with the v2 mark that identifies it, and judged in
every canon by the node's verifier and by the kit's: if an AIR changes without its version rising
(the «Regla de extensión» [extension rule] of the D-G), the old proof stops verifying and the canon
says so. The kit's five already have it through their catalogues; the layer's fourteen do not, and
the D-G covers them with tests of the layer (`spec/rfc/0018-el-tren-0-5.md:425-426`). It needs
compiling, and it enters with the E3 of the D-G, at the cut: a proof of `zkssl/0.4` does not say
which AIR it is from. ⚠️ §826: «and by the kit's» holds for the kit's five. The layer's fourteen are
judged by the node's verifier: the kit verifies no send, claim, burn or threshold proofs
(`spec/rfc/0018-el-tren-0-5.md:110`). That a kit judges them is not decided: it would be opened by
the strong version of `ProofFailed` and `VerificationFailed` of the E3 of RFC-0019, in the same cut
(the «Compatibilidad» of RFC-0017).

## Compatibility

None. E1 is this text; E2, text in `spec/RPC.md` and cross-references; E3, gates that change nothing
of what travels. No vector moves because of this RFC. Its first application will be the `zkssl/0.5`
cut (E4): RFC-0018 is already in the tree, PROPOSED since §675, with its stages and its status in
the header, and that would satisfy R5 for what it already has built. ⚠️ §799: until here it said
«would be». ⚠️ §826: it did not satisfy it whole. R5 also asks for the table of stages with the
surface each one breaks, and neither RFC-0018 nor RFC-0019 had a table; they have it since §826; and
since then that of RFC-0017 names the surface of its E2, which said «NO», and that of RFC-0020, that
of its E4. ⚠️ §865: RFC-0023, if it travels in it, has it since §827.

## Decisions (the author's, §799)

⚠️ §799: until here this section was titled «Open decisions (the author's)» and began «None is
taken. The session says what it recommends; the author decides».

The author accepted this RFC in §799, and with it took the eight decisions: in each one, the branch
the text recommends; in decision 2, where the session did not recommend on two doubts, what D-A
already said. The questions and their branches stay as they were written in §790; after each one,
the one taken and the discarded ones. The RFC stays PROPOSED until its stages are built.

1. **Where does the promise live?** In `spec/RPC.md`, as the section of D-C (recommended: it is the
   normative document that whoever implements the wire reads first, and entry 90 puts it «next to»
   its own), or in a document of its own in `spec/`, with cross-references from `spec/RPC.md`,
   `spec/PAQUETE.md` and `spec/NUCLEO.md`.
   **Taken (§799): in `spec/RPC.md`**, the section of D-C. Discarded: a document of its own.
2. **What enters the surface?** The six of D-A and what it says is outside. Two doubts the session
   does not resolve: the witness's journal, which `--auditar` rereads and which an auditor
   custodies; and `zk-ssl-verify` as a library, which with the second one is the only thing that
   verifies a v1 head today.
   **Taken (§799): the six of D-A**, and «Outside the promise» what is enumerated there, the
   witness's journal and `zk-ssl-verify` as a library included. Discarded: putting the witness's
   journal or the library in the surface.
3. **Does issuing a new head version break the wire?** The session recommends yes (R1): it closes
   the practice from v3 to v6 (fact 3), and it is what RFC-0020 proposes for its v7.
   **Taken (§799): yes**, it is R1. Discarded: that it does not break.
4. **How much notice?** One seal with the RFC PROPOSED before the cut (R5, recommended, because it
   is what the house can check); a period in days; or a release of the kit that accepts the new
   before the node issues it, which R2 allows for head versions and the D-E of RFC-0018 forbids for
   the wire.
   **Taken (§799): one seal with the RFC PROPOSED before the cut** (R5). Discarded: the period in
   days and the release of the kit ahead of the node.
5. **What does a `0.N` line promise?** That within `0.N.x` nothing of the surface breaks, except by
   R6 f, and that between `0.N` and `0.N+1` everything can break with R5 and R6 (recommended, while
   the protocol is below 1.0); or a stronger promise from now on.
   **Taken (§799): the first**, while the protocol is below 1.0. Discarded: a stronger promise from
   now on.
6. **Can a security fix change verdicts within `0.N.x`?** The session recommends yes, with its
   notice (R6 f), like 0.4.2; the other branch forces a new line for each fix.
   **Taken (§799): yes**, with its notice (R6 f). Discarded: a new line for each fix.
7. **How long does a kit stay published?** Always, which is the practice (recommended, R6 a); or
   until a date the release declares.
   **Taken (§799): always** (R6 a). Discarded: until a date the release declares.
8. **Does E3b enter with the cut or before?** With the cut (recommended): before, a proof of
   `zkssl/0.4` does not say which AIR it is from, and the gate could only compare it with the
   commit.
   **Taken (§799): with the cut.** Discarded: before.

## Security

- **The spending key never travels**: this RFC does not touch it.
- **Downgrading.** R6 e and the D-E of RFC-0018: no kit accepts two versions of the wire, so an old
  proof does not get in through a new kit.
- **Promising stability does not oblige one to accept what is broken.** R6 f allows fixing a verdict
  with notice, and R6 c, with RFC-0020, stops accepting the signatures of a broken scheme without
  orphaning the heads that another signature contains.
- **What is promised and not checked.** Until E3, the promise is text; R7 and the section say what
  the version does not identify, as §699 does today with the proofs.

## What this RFC does not decide

How a proof is identified (the D-G of RFC-0018, entry 118) nor a signature (RFC-0020): it only
requires that each surface have its identifier. Who accepts an RFC the day the author is not there
(entry 88). The parameters of the cut (the D-F of RFC-0018, entry 116). And the single judge (entry
117).

## References

- `BACKLOG.md`, entries 82, 85, 87, 88, 90, 116, 117 and 118.
- `AUDITORIA.md` §209, §236, §275, §278, §281, §290, §418, §641, §662, §663, §664, §680, §682, §684,
  §692, §699, §707, §781 and §787.
- `spec/rfc/PROCESO.md` §«Reglas» [rules] and §«Fallos de seguridad»; `spec/RPC.md` §«Notas
  operativas»; `spec/PAQUETE.md` §«6. El contrato del mando»; `spec/NUCLEO.md` §«3. La regla de
  extensión, ejercida»; `doc/KIT.md` §«0. Descargar, y comprobar la descarga antes de creerla»
  [download, and check the download before believing it]; `SECURITY.md` §«5. Cómo reportar una
  vulnerabilidad» [how to report a vulnerability].
- The D-E and the D-G of RFC-0018; the E2 of RFC-0019; RFC-0020.
