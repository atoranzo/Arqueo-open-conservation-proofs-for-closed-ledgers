# How work is done here

> **Español:** [`doc/METODO.md`](./METODO.md) is the original of this page, in Spanish; it prevails
> wherever the two differ. This translation is of commit `7766efb` (audit entry §917) and was
> produced by the project's assistant, a Claude Code session, in audit entry §918 (`GENAI.md`): it
> is a generated artefact, not text written by the author. Line citations of the form `file:N` refer
> to the Spanish files.

What a change has to carry to enter Arqueo, told outside the session handovers: the seal and its
entry, how a live document is corrected, the conventions of `BACKLOG.md`, the RFCs and their stages,
the levels of the canon and the benches, and step 4 of `GENAI.md`. It is the third piece of entry 88
of `BACKLOG.md`, which asked to «write down the "how work is done here" outside the session
handovers, which today are the only place where the method lives».

**This document establishes nothing.** It tells what other files establish and cites, in each
statement, the one that does. Where something is always done the same way and no file mandates it,
it says so with the word *custom*. If this document and the file it cites stop saying the same
thing, the file rules, and this one is corrected with its mark (section 3).

**What it does NOT decide.** Who maintains the project and by what rule an RFC is accepted. Entry 88
puts it first among what closes it —«declare the maintenance and the acceptance criterion of an RFC
—the process already exists in `spec/rfc/`, what it lacks is who decides and by what rule—»— and it
is the author's. What is written today about who accepts, and is not that rule, goes in sections 5
and 7.

---

## 1. The method: four steps, and outside step 4

`GENAI.md` §«Cómo se usa, exactamente» [How it is used, exactly] gives the method in four steps: one
first **measures**, on the tree, and a remembered summary is not a source; one discusses those
numbers and **the author decides**; the assistant proposes a **block**, a script that applies the
change with its own gates and its red path; and **the author runs it on his machine**, runs the
canon `--sello` and commits only what comes out green. From there come its two claims: nothing
enters without passing the gates its entry declares, and no claim of the repository rests on the
model's knowledge.

Measuring before claiming is also the first rule of `CONTRIBUTING.md` (`CONTRIBUTING.md:10-13`), and
its section 1 adds another, learned in §59.2: every tool declares its coverage and the declaration
is checked (`CONTRIBUTING.md:24-27`). A pin of the canon is updated the same way: «It is MEASURED
first and the table is edited afterwards. Never the other way round» (`tools/canon.sh`, its section
«Como se actualiza un pin» [How a pin is updated]; ⚠️ §854: it said `:77-79`, and the header grew
above it).

**Outside step 4.** A Claude Code session does not propose the block: it runs it itself. `GENAI.md`
§«Fuera del paso 4: la sesión de Claude Code» [Outside step 4: the Claude Code session] says what
changes, and it is this:

- **who applies, checks and commits**: the session, in its container and its branch, with the
  author's identity and without lines that name it, as `CLAUDE.md` §«Commits: el asistente no firma»
  [Commits: the assistant does not sign] mandates;
- **its gates**: those of the canon, which the session runs and nobody else sees run; the entry says
  whether `--sello` came out VERDE on the tree of the change and, if it was not run in full or did
  not come out green, what was run and what it gave. No gate checks that what is declared was run;
- **who accepts**: the author, and his acceptance is the change's entry into `main`.

The entry of a change made this way says so in its opening, almost always with the words «fuera del
paso 4 de `GENAI.md`» [outside step 4] (the same section). And what an assistant drafted and entered
with acceptance pending is stated by the document itself, with the words «integrar no es aceptar»
[integrating is not accepting], and does not count as a decision of the author until an entry of his
accepts it (`GENAI.md` §«Alcance» [Scope]).

## 2. The seal and its entry

The per-change record is `AUDITORIA.md`: «one entry per verified change, with its commit, its
counters, what the change does **not** close and the lessons it left» (`GENAI.md`
§«Dónde está el registro» [Where the record is]). In the entries, *sello* [seal] is the commit that
carries one —«Un solo sello» [A single seal] (§780), «Un sello corto de documentos» [A short seal of
documents] (§710)—, and *the canon `--sello`* is the level of gates that is run on its tree before
committing it (section 6).

**The number.** It is the next free one, and a session that works in parallel with others looks for
it in all their branches (§710: «none of the six branches of `origin` that the clone knows […]
carries a §710»), or works in a range of its own («the next one in this session's range (§770 to
§799)», §778). Up to §274 the headings are «## 274.»; from §275, «## §275 —».

**The opening.** Custom of the recent entries —from §778 to §789, for example—: on which commit it
goes; which files it touches and which not; what is the author's and what the session proposed; who
writes it, checks it and commits it, with «fuera del paso 4» if it is a session; the zones it does
not touch; and whether the canon `--sello` came out VERDE on that tree with the entry in place. Two
of those things are asked for by `GENAI.md` §«Fuera del paso 4: la sesión de Claude Code»: that the
opening say that the change came from a session, and whether `--sello` came out VERDE or, if not,
which gates were run and what they gave; `CLAUDE.md` §«Commits: el asistente no firma» also asks for
the first. No file mandates the rest.

**The sections.** Also custom, in bold: **De dónde sale.** [Where it comes from], **Quién lo
decide.** [Who decides it], **Lo que hace.** [What it does], **Medido.** [Measured], **Contadores.**
[Counters] and **Lo que NO hace.** [What it does NOT do] (§778, §779, §780); sometimes **El
defecto.** [The defect] or **Por qué no lleva aviso de seguridad.** [Why it carries no security
advisory] (§780). What `GENAI.md` requires is less: the commit, the counters, what it does not close
and the lessons.

**The counters.** What the seal moves, from where to where: the test pins of the canon's table, each
with its chronicle in its row («§780: 147 -> 148, …», `tools/canon.sh`, the row of `zk-ssl-cli`),
the totals that `PRINCIPIOS.md`, `PAPER.md` and `PAPER_EN.md` repeat, the vectors and the manifest
lines with their fingerprint, and the boxes of `BACKLOG.md` (§780, «Contadores»). That the figures
of the live documents keep being those of the canon is watched by `tools/check_cifras.py` and
`tools/check_figures.py`, in the canon's loop (section 6).

**An entry is not rewritten.** §237 says so: «the entries describe **a moment** and are not
rewritten»; and §247: «the records are not rewritten, they are corrected on top»: what an entry said
wrong is corrected by another, later one. That is why the figure tools leave out `AUDITORIA.md` and
`BACKLOG.md` (`tools/check_cifras.py:65-71`), and `tools/check_figures.py` checks the premise that
makes that exclusion honest in `BACKLOG.md` (section 4).

**The double thread with the RFCs.** The entry that seals a protocol change cites the RFC by its
number, and the RFC cites the entry (`spec/rfc/PROCESO.md` §«Reglas» [Rules], rule 5).

## 3. How a live document is corrected

In place, and with a note that says what it said and why it changes: «in a live document, the text
is corrected in place and a note «⚠️ §778» says what it said and why it changes; in the table of
this document, which strikes through, it is struck through and the correct one is put beside it»
(§778, «Quién lo decide»). No RFC strikes through: it is corrected without striking and with a note
at the end of the section (§778, point 4). It is the rule that `BACKLOG.md` takes as its own: «this
project marks the corrections instead of deleting them» (the header of `BACKLOG.md`).

What is not rewritten is not corrected in place:

- the entries (§237, §247), which are corrected with another one;
- the deposited preprints, which are kept as they are (§237: «deposited with a DOI, kept verbatim»);
- the published vectors, which are never rewritten: if the wire breaks, the old ones are kept under
  their version (`spec/rfc/PROCESO.md` §«Reglas», rule 2), and since §692 the canon pins the
  fingerprint of each one: a vector can be added, or moved with the same bytes to
  `spec/vectors/<versión>/`, and modifying or deleting it is ROJO (`tools/check_vectores.py`; the
  «2 quater» block of `tools/canon.sh`; ⚠️ §854: it said `:343-352`, and the block moved down);
- and the author's reports with acceptance pending, which «are neither rewritten nor marked without
  him» (§710, «Lo que NO hace», about `doc/ecst/`).

When a document and the code stop matching, «the code wins» (the footer of `CONTRIBUTING.md`;
⚠️ §855: it said `:171-172`, its lines in §793; §814 and those that followed it added lines above,
and §855 another seventeen).

## 4. The BACKLOG

What its own header says:

- it is what is missing, **numbered** and by priority, so that it can be cited by its number between
  sessions;
- **numbers are neither reused nor renumbered**;
- what is resolved is marked `[x]`, annotated with which commit and **stays in the list**;
- what is new goes at the end with the next number, and is placed in its group without changing
  number.

**The Estado line** [Status] counts the boxes, `- [ ]` open and `- [x]` resolved, plus the suspended
ones, and says in which entries it was recounted: «contada, no recordada» [counted, not remembered].
It counts boxes, not entries: the two that were closed with the `[x]` in front and the original body
below, with its own box, entries 85 and 77, count that box as open. Custom: a seal that opens or
closes an entry recounts it and says so in its Contadores [Counters] (§779).

**Counting is not enough.** The «rancia de OTRA clase» [stale of ANOTHER kind] (the header of
`BACKLOG.md`, §583): the count added up and the list lied, because three open entries were closed in
the tree. An entry is checked against the tree, not only counted.

**The notes of an entry.** Custom: a note that opens with the number of its seal —«⚠️ **(§639) …**»,
«⚠️ **§710: …**»— and says what changes (for example, those of entry 84); and in an open entry that
advances, «✅ a medias (§643): E2, the core of the act […]. REMAINING: the node (E3), […]», which
almost always ends in «sigue abierta» [still open] (entry 84, the one of RFC-0015). No file defines
«a medias» [half done]; the entry stays open until it is really closed, and when closing it, «the
closure goes after the body» (§779, about entry 42, «like 69, 102 and 106»).

**No test figure in an open entry.** The figure tools do not look at `BACKLOG.md`, and that is only
honest while its test figures live in closed notes: `premisa_de_la_exclusion` checks it in every
seal (`tools/check_figures.py:55`).

**Who closes.** Custom: the seal that closes an entry proposes the closure with the measurement in
front, and says so in «Quién lo decide» (§779: «Closing 42 and 104 is proposed by the session with
the measurement in front»).

## 5. The RFCs

**What goes by RFC.** A change to the protocol —what crosses the wire: `spec/RPC.md`,
`spec/openrpc.json` and the vectors of `spec/vectors/`— does not enter by direct commit
(`spec/rfc/PROCESO.md:3-6`).

**The states.** DRAFT → PROPOSED → ACCEPTED → FINAL, or WITHDRAWN at any point
(`spec/rfc/PROCESO.md` §«Estados» [States]).

**The five rules** (`spec/rfc/PROCESO.md` §«Reglas»): one file per RFC, numbered; every RFC declares
its compatibility, and if it breaks the wire the version goes up and the old vectors are kept under
theirs; every RFC declares its effect on the principle of the API, that the spending key never
travels, and the one that erodes it is born WITHDRAWN; ACCEPTED requires the spec up to date, the
OpenRPC regenerated, the vectors re-emitted or new and the suites green; and the double thread with
the entry (section 2). A security flaw has its two clauses: the project's own fix is drafted in
private and enters in one go, and the draft of the report to a third party does not enter the public
tree (`spec/rfc/PROCESO.md` §«Fallos de seguridad» [Security flaws]).

**The template.** `spec/rfc/0000-plantilla.md`: the header with the status, the author, the GenAI
assistance, the date, the affected version and the entries; and the sections Motivación, Diseño,
Compatibilidad, Seguridad and Referencias [Motivation, Design, Compatibility, Security, References].

**The stages and the decisions**, which are custom and not rule: neither the template nor the
PROCESO asks for them.

- The RFC splits the work into stages, E1, E2…, and each one is built in its seal, which the RFC
  names. Most carry them in a table, «stage | what it delivers | breaks the wire? | status» —0001,
  from 0004 to 0010 and from 0012 to 0023 (⚠️ §827: it said «from 0012 to 0021»; 0022 and 0023 carry
  it since §827, with the third column of 0018 and 0019); for example,
  `spec/rfc/0015-el-ciclo-de-vida-del-firmante.md` §«Estado de las etapas» [Status of the stages]—;
  0002, in a table of another shape; and 0003 and 0011, in a list. In 0018 and 0019, since §826, the
  third column is «breaks the wire?, and which surface», and the table goes before «Seguridad»
  [Security], so as not to move its cited lines; E2 of 0017 and E4 of 0020 also name their surface:
  R5 of RFC-0021 asks for it, one seal before the cut. ⚠️ §826: it said «from 0012 to 0017» and «and
  0018 and 0019, in their header and their design»: 0020 and 0021 carry it since §790, and 0018 and
  0019, since §826.
- The open decisions go numbered —D-A, D-B…, or 1, 2…— and they are the author's. When he delegates
  them, the assistant takes them («The five decisions were taken by the assistant by delegation of
  the author», `spec/rfc/0009-lo-que-revela-una-prueba.md:65`), and the RFC says with what criterion
  and that they are reversible: «TAKEN in §609, by delegation of the author and with the criterion
  he dictated» (`spec/rfc/0014-el-recibo-del-lote-y-de-la-prenda.md:128`); «TAKEN in §642 by
  delegation of the author, with the criterion of §609, and REVERSIBLE» (the header of RFC-0015).
- An RFC is usually born PROPOSED with its decisions not taken and no stage built —of RFC-0015, in
  §639: «Five decisions of the author, none taken, and no stage built» (entry 84)—, and its header
  says at each moment what is decided and what is built (that of RFC-0019, for example).

**Who accepts.** What the record says when accepting RFC-0014 is that «the acceptance is the
author's, and rule 4 of the PROCESO asks for its canon and its merge» (§618); the text of rule 4
asks for the spec, the OpenRPC, the vectors and the suites green, and does not name the merge. The
PROCESO does not say who decides nor by what rule: it is what entry 88 asks to declare, and this
document does not decide it.

## 6. The canon and the benches

**One source.** `tools/canon.sh` is the only source of the gates since §224; before, each block
rewrote them by hand (the header of `tools/canon.sh`, «La fuente unica» [The single source]).

**The levels** (the list of commands in the header of `tools/canon.sh`):

| command | what it runs |
|---|---|
| `bash tools/canon.sh --sello` | every block: the tests of each crate in release —⚠️ §864: and those of the five crates that a third party compiles without the prover, again, with debug assertions (the «1 bis»)— and the rest of the gates; it is the one of every seal |
| `bash tools/canon.sh --largo` | everything in `--sello`, and also halo2 and plonk, and the node and the witness compiled from two roots (`tools/artefacto.sh --binarios`, «3 ter binarios»); ⚠️ §824: it said «everything in `--sello`, and also halo2 and plonk» |
| `bash tools/canon.sh --completo` | everything in `--largo`, and also `zk-core` |
| `bash tools/canon.sh --lista` | only shows the table; runs nothing |
| `bash tools/canon.sh --bancos` | the benches, all of them (`tools/bancos.sh`); ⚠️ §804: except the one of the cuts, which is run apart |

They are three levels because putting `zk-core` in every seal «would guarantee that someone ends up
skipping the gate, and a gate that is skipped does not protect» (§225; `tools/canon.sh`, «Los TRES
niveles, y por que tres» [The THREE levels, and why three]). Nobody forces `--completo`: it leaves a
record in `.canon/ultimo-completo`, and every invocation says when it was and how many seals have
passed (the same section); which paths make it fall due, the block «lo que cubre el nivel
--completo, y nadie mas» [what the --completo level covers, and nothing else] of `tools/canon.sh`
(§284). ⚠️ §854: the five citations of this section up to here went by line: `:2-6`, `:8-20`,
`:49-60`, `:62-67` and `:134-146`, those of §793. The first two were still in place; the other three
were not: the header grew above them, and `:134-146` fell in the table. They are cited by their
section or their block.

**What makes it robust** (`tools/canon.sh`, «Lo que lo hace ROBUSTO, y no es la tabla» [What makes
it ROBUST, and is not the table]): the members of the workspace are read from `Cargo.toml`, and a
crate without a row turns the gate red; the pins are exact, not minimums; it does not stop at the
first failure; each crate has its timeout; no `.rs` of `src/` is left undeclared; and every `cargo`
of the levels runs with `--locked` (§694). ⚠️ §854: it said `:22-47`, the one of §793, and the
section moved down five lines.

**The tool loop** (the «2 ter» block of `tools/canon.sh`): those in the list of its `for`
—`check_cifras`, `check_figures`, `check_columns`, `check_constraint_layout`, `verificar_citas`,
`check_dominios`, `check_publicadas`, `check_nucleo`, `check_techo`, `check_curvas` and
`check_superficie`—, at pin zero. And being in the loop is not enough, «it has to look at something»
(§695, in the comment of the same block): `check_columns` ran there without looking at any `.rs` and
said green, and since §695 its universe and its proof of life live in the tool. Outside the loop,
each in its block, go other gates; among them, the vectors (`tools/check_vectores.py`, «2 quater»,
which leaves the loop «so that each ROJO comes in with its name»), the catalogue of rejections
(`tools/check_catalogo.py`, «2 quinquies»), the conformance of the kit in its families
(«3 bis familias»), the second implementation (`tools/segunda/`, «3 duodecies») and the artefact
(«3 ter»; ⚠️ §824: from here on in two roots, and from `--largo` also the node and the witness,
«3 ter binarios»). The verdict is VERDE, or ROJO with the list of failures (the «veredicto»
[verdict] block of `tools/canon.sh`). ⚠️ §801: the three lines of `tools/canon.sh` of this paragraph
said `:312-341`, `:324-330` and `:608-621`, those of §793; the file grew ahead of the loop and in
§799 they were already where they say now, with the same text. ⚠️ §810: the third, the verdict's,
said `:697-710`; §804 had already shifted it two lines, and the fourteen that §810 puts into
«3 duodecies» shift it another fourteen: it is cited by its block. ⚠️ §854: the other two too: they
said `:401-430` and `:413-419`, and the block had already moved down two lines.

**The benches.** `tools/banco_*.sh` start the real node, witness and CLI, and that is why they live
outside the levels (§582). `tools/bancos.sh` runs them all: the list is read from the directory,
each bench has its timeout, the tree has to be clean before and after each one, and two long modes
are left out, declared (`tools/bancos.sh:7-30`). Nobody forces them either: every green pass leaves
a record in `.canon/ultimo-bancos`, and every invocation of the canon says when it was and whether
what they exercise changed since then (`tools/canon.sh`, «Los bancos, tampoco» [Nor the benches]).
Since §786 they compile with `--locked`, and a gate of the canon requires it per call (the block
«§786 · los bancos compilan con --locked» [the benches compile with --locked] of `tools/canon.sh`;
⚠️ §854: the two citations said `:69-75` and `:413-416`, and neither fell in its place any more).
⚠️ §804: it runs them all except `tools/banco_cortes.sh`, which point 5 of `tools/bancos.sh` leaves
out by its name: it repeats the measurement of §699 on six fixed commits of the history, it does not
measure the seal's tree, and it compiles six times. It is run apart, and every pass of the others
names it.

## 7. Who decides what, in what is written today

- **The author decides** on what is measured (`GENAI.md` §«Cómo se usa, exactamente», step 2) and
  **accepts**: in step 4, with his commit; outside it, with the change's entry into `main`
  (`GENAI.md` §«Fuera del paso 4: la sesión de Claude Code»).
- **The commits** go in the author's name, also those a session makes, and none carries
  `Co-Authored-By` or `Claude-Session:` (`CLAUDE.md` §«Commits: el asistente no firma»).
- **What a session proposes** is stated by the entry in «Quién lo decide» (custom: §778 to §789).
- **The delegated decisions** are taken by the assistant with the criterion the author dictated, and
  the RFC says that they are reversible (section 5).
- **What an assistant drafted** and entered with acceptance pending does not count as a decision of
  the author until an entry of his accepts it (`GENAI.md` §«Alcance»).
- **What is not written**: who maintains the project, the acceptance criterion of an RFC and the
  second pair of hands; entry 88 asks for them, and they are not supplied here.

## Provenance

Written in a Claude Code session (§793), outside step 4 of `GENAI.md`, on the tree of §789
(`8c02941`): each citation was read in that tree. «Integrar no es aceptar» [integrating is not
accepting]: the acceptance that `GENAI.md` §«Alcance» describes remains pending with the author.
