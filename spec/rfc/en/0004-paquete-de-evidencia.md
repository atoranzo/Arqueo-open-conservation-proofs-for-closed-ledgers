# RFC-0004 — The portable evidence package and its specification

> **Español:** [`spec/rfc/0004-paquete-de-evidencia.md`](../0004-paquete-de-evidencia.md) is the
> original of this page, in Spanish; it prevails wherever the two differ. This translation is of
> commit `7766efb` (audit entry §917) and was produced by the project's assistant, a Claude Code
> session, in audit entry §918 (`GENAI.md`): it is a generated artefact, not text written by the
> author. Line citations of the form `file:N` refer to the Spanish files.

- **Status:** ACCEPTED — **rule 4 of the PROCESO, settled with measurement**
  (§396 the adoption; §397–§399 the stages E1–E3 with their gates;
  §400 this turn). It is preserved as a record of what was decided, what was measured
  and what was discarded.
- **Author:** Ángel José Toranzo Portela
- **GenAI assistance:** Claude (session 92) — see [`GENAI.md`](../../../GENAI.md)
- **Date:** 2026-09-04
- **Affected protocol version:** `zkssl/0.3` — **does not go up** (see Compatibility)
- **AUDITORIA entries:** §243, §289, §290, §293, §322, §395, §396, which seals it, §397
  (E1) and §398 (E2), §399 (E3), and §400 (this turn to ACCEPTED)

## Status of the stages

| stage | what it delivers | does it break the wire? | status |
|---|---|---|---|
| E1 — the move | a normative document of its own under `spec/` as the single producer; the binary's header refers to it; `spec/RPC.md` and `spec/README.md` point to it | NO | sealed — §397 |
| E2 — the vectors and the gate | `spec/vectors/paquete/` and its conformance block in `tools/canon.sh` | NO | sealed — §398 |
| E3 — the head's index | `verificar_cabeza` binds the declared `index` to the leaf index that goes inside the XMSS signature (as §332 did with the co-signature); the command prints the embedded one; the witness reads the served `index`; two negative vectors | NO | sealed — §399 |

Every measurement in this document was taken on `bb5322f` (§395), in two pure readings
that wrote not one byte to the tree: `PASTE-396-M` and `PASTE-396-PRE` (see
References).

## Motivation

The **portable evidence package** exists since §289: the wire responses that the
holder already keeps, gathered in a file and verifiable **without the node, without the
layer and without the prover** (§243). It is what sustains their position when the
operator disappears or lies, and `spec/RPC.md` declares it so in its shutdown section.

Its specification **exists and is good. It lives in the wrong place.**

It is written in the header of `crates/zk-ssl-verify/src/main.rs`, lines 1..90 (region
sha `293990fedc785833`): format v1, v2 with `cofirmas` (§322), the third form
—the extension package (§293)—, the order of checking in three steps and the two
versioning conventions that coexist in the same file. And `spec/RPC.md` 810..855 (region
sha `7963cdea2e82f540`), which is normative about the package, **delegates the format
there in writing**: «formato v1, declarado en la cabecera del binario de `zk-ssl-verify`»
[format v1, declared in the header of the binary].

Three measured problems follow from that.

**1. A third party who receives the binary and wants its specification has to read the
operator's source.** It is the problem of §243 again, from the other side: back then the
only way to *verify* a head was to compile the operator's code; today the only way to
know *what* it verifies is to read its source.

**2. The header already expired once, and it confessed it.** Its own section «La tercera
forma, que esta cabecera NO declaraba (§247)» [the third form, which this header did NOT
declare] says that the extension **was published in `spec/RPC.md` and absent here**, and
draws the conclusion: «el productor rancio era el que más cerca queda del código» [the
stale producer was the one closest to the code]. Two producers of the same contract, and
the one that aged was the one inside. A single normative producer prevents it by
construction.

**3. Nothing binds the document to the binary.** The binary reads **14 distinct keys** of
the JSON and has **39 rejection calls with text**; no gate checks any of them.
`spec/vectors/` contains only the three vectors of the **wire**, and the conformance block
of `tools/canon.sh` (`:278-286`) runs `conformance --check` only over them. There is not
one package vector in the tree, nor a JSON fixture inside the crate.

> **Correction (§397).** The «39» above is a census PER LINE, blind to the eleven calls
> whose literal starts on the next line. Per CALL, flattening the continuations, they are
> **50** (49 distinct texts); and the 14 keys are those of the envelope: in all the binary
> reads 31 names. The sentence is kept; the figure that counts is that of
> `spec/PAQUETE.md`, re-derived on sealing.

## Design

### E1 — the move

**A normative document of its own under `spec/`** is born, in Spanish like the rest of
the folder, and it is the **single
normative producer** of the format of the package and of the contract of the command. It
is not drafted from scratch: what is already written in the binary's header is **moved**,
which is the pattern of the move of the guardian (§296).

It covers three objects, which are the three the binary accepts:

1. **the v1 package** — `v`, `cabeza` (the payload of `zkssl_signedEpochHead` with
   `available:true`) and optional `acuse` (`seq`, `hashPrueba`, `s`, `camino` with
   `siblings` and `isRight`);
2. **the v2 package** (§322) — the same object with `cofirmas`, the response of
   `zkssl_cosigs` as it is;
3. **the extension package** (§293) — `tipo: "extension"`, `vieja`, `nueva`, `camino`.

And it adds what today is nowhere as a contract: the **order of checking**, the
**rejection rules** and the **contract of the command** (one argument, and three exit
states: success, failure, and usage).

**The document specifies the ENVELOPE, not the fields.** The values of the package are
wire responses **as they are** —whoever rewrites, tampers—, so the semantics of each
field **refers to `spec/RPC.md` by line**. Duplicating it would create two producers of
the same contract, which is precisely the defect this RFC repairs.

The header of `main.rs` **refers to the document and keeps its history**: it is not
erased, it is cited (§247). The form is already practised in this same crate, in
`crates/zk-ssl-verify/Cargo.toml:25-27`, which says of its own enumeration «la lista no
es su superficie: la superficie se lee de los `pub` de `src/lib.rs`, no de aquí» [the
list is not its surface: the surface is read from the pub items, not from here].

`spec/RPC.md` 810..855 keeps its whole prose and **changes its pointer**: where it
delegates to the binary's header, it comes to delegate to that document.

`spec/README.md` gains its row in the table «What is in this folder» and its mention in
«Where the rest lives».

**Gate of E1**: the document in the tree, with the name E1 fixes; the binary's header
referring and not enumerating; the pointer of `RPC.md` corrected; the README row in place; the nine
tools of `tools/` green and `canon.sh --sello` VERDE. And a content gate:
the document covers **all** the keys and **all** the rejection rules the binary
implements — the census is **re-derived on sealing**, not typed.

### E2 — the vectors and the gate

**`spec/vectors/paquete/`** is born, with one positive vector per form (v1, v2,
extension) and one negative vector per rejection rule the document publishes.

The gate goes in the **conformance block of `tools/canon.sh`** (`:278-286`), where the
wire's conformance already lives with its regime of VERDE and RECHAZADO. Running the
binary against JSON files is immediate and needs no node.

**The gate does not go inside the crate.** An `include_str!` towards `spec/` would bind
the crate to the repository, which is exactly the property that §243 established and that
§395 has just turned into a checkable invariant.

**Which rules are already implemented and which are not, E2 MEASURES before writing a
vector.** This RFC does not assert it. What is indeed published today in two places
—`RPC.md` 835..837 and the binary's header— is the rule «**un v1 que traiga `cofirmas` se
RECHAZA**, porque subir la versión es exactamente lo que las hace parte del contrato» [a
v1 that brings co-signatures is REJECTED, because raising the version is exactly what
makes them part of the contract]; it is the first candidate for a negative vector, and
its real state is measured, not assumed.

**Gate of E2**: each form with its positive vector and each published rule with its
negative; the conformance block running them; and **the gate shown alive**: a vector
sabotaged in a single nibble puts the canon in red. It is the mould that
`tools/banco_apagado.sh` already practises —green with the good package, red with one
tampered nibble— moved to a gate that does not need to bring up a node.

## What was DISCARDED on measuring

1. **Drafting the specification from scratch.** Discarded: it already exists and is
   correct. The defect is not its content, it is its address. This is a move, not a
   drafting.
2. **Putting the specification inside `spec/RPC.md`.** Discarded: `RPC.md` specifies
   what crosses the wire, and the package **does not cross the wire** — it is an artefact
   handed to a third party without a node. `spec/README.md` itself defines itself as the
   normative surface of «everything that crosses the wire». Names that mean one thing only.
3. **A gate inside the crate, with `include_str!` towards `spec/`.** Discarded: it would
   bind the crate to the repository, against §243 and against the gate of §395.
4. **A new tool in `tools/`.** Discarded: the conformance block of `canon.sh` already
   does exactly this, and one more tool would move the census of the nine (§384) without
   gaining anything.
5. **Putting the package vectors among the `zkssl-0.N.json`.** Discarded: those are one
   file per version **of the wire**, never rewritten; mixing vectors of another object
   there would break that semantics. They go in `spec/vectors/paquete/`.
6. **Wiring `tools/banco_apagado.sh` into the canon as a gate.** Discarded *for this
   RFC*: the bench brings up a real node and takes minutes, and putting it in `--sello`
   changes the cost of sealing. That cost has to be measured first, and it is a front of
   its own. The vectors give the gate without paying it.
7. **Fixing here the name of the file E1 will create.** Discarded BY MEASUREMENT, not
   by taste: the tree runs `tools/verificar_citas.py` inside the canon, and that
   gate demands that every Markdown document cited by name EXIST. On assembling
   this RFC with the name written, the gate went from zero ghosts to one ghost
   with six citations, and the seal stopped in red before the commit. And it did so
   rightly: a reader who followed that reference would find nothing. An RFC PROPOSES; it
   cannot cite what does not yet exist. The name is fixed in E1, in the same commit
   that creates the file.
8. **Raising the wire version.** Discarded by the criterion published in
   `spec/RPC.md:868`: the version goes up when **the values that travel** change, not
   when the surface grows. Here no new value travels.

## Compatibility

| stage | does it break the wire? | vectors |
|---|---|---|
| E1 — the move | **NO.** No value that travels changes; `zkssl/0.3` stays in force | the three of the wire, **intact**; `conformance --check` must keep giving `0.3` IDÉNTICO and rejecting `0.2` and `0.1` as of ANOTHER version |
| E2 — the vectors and the gate | **NO.** The package vectors are a universe apart, under `spec/vectors/paquete/` | idem: those of the wire are not touched |
| E3 — the head's index | **NO.** Nothing that travels changes; the verifier becomes stricter about an envelope that no honest producer emits | those of the wire, intact; the package manifest gains two entries |

**Rule 2 of the PROCESO: old vectors are never rewritten.** The three vectors of the
wire —`zkssl-0.1.json`, `zkssl-0.2.json` and `zkssl-0.3.json`— **are preserved** under
their version, without a single modification.

### Why it enters by RFC if it does not break the wire

Both readings are true and this RFC writes both.

**By the letter**, the scope of the process (`spec/rfc/PROCESO.md:3-6`) enumerates
**files**: `spec/RPC.md`, `spec/openrpc.json` and the vectors of `spec/vectors/`. This
change touches `spec/RPC.md` —the pointer of the shutdown section— and adds under
`spec/vectors/`. It falls inside, and it enters by RFC.

**By the spirit**, the package does not cross the wire, so there is no value that travels
that changes. That exempts it from **raising the version**, which is another rule and
another criterion (`RPC.md:868`). It does not exempt it from the process.

**`spec/openrpc.json` is neither touched nor regenerated**, because this RFC adds no
method. Rule 4 demands, for ACCEPTED, «la spec actualizada + OpenRPC regenerado +
vectores re-emitidos (o nuevos bajo la versión nueva) + suites verdes» [the spec updated +
OpenRPC regenerated + vectors re-issued (or new ones under the new version) + green
suites]: the turn to ACCEPTED will be justified with the spec, the new vectors and the
green suites, **and with this declaration that the OpenRPC stays out of scope because
there is no new method**.

> **Note (2026-09-05, §400).** ACCEPTED. Rule 4 of the PROCESO is settled with
> measurement: the spec is `spec/PAQUETE.md` (E1, §397; the bound of the index, §399) and
> `RPC.md:832` refers to it — the OpenRPC **is not regenerated**, by the declaration
> above: there is no new method, and `spec/openrpc.json` stays at the `zkssl/0.3` of §354
> with its three binding tests — the vectors are new under `spec/vectors/paquete/`
> (E2, §398; 67 entries since §399) and the three of the wire stay intact under the
> triple gate of the canon — and the suites green: canon `--sello` VERDE in 178 s (§399).
> Measured in PASTE-400-M (`8c39d8f95e7af349`, output `2c070dfb047359cc`). Double thread:
> the entry §400.

## Security

**Effect on the API principle —the spending key never travels: NONE.** This
RFC adds or changes no method, no type of the wire and no material. The package
is wire responses already published, gathered in a file.

**Effect on the declared debts:**

- **Out-of-band notice (§21): none.** The package carries no notices: its fourteen
  keys are `v`, `tipo`, `cabeza`, `available`, `publicKey`, `signature`,
  `epochDigest`, `acuse`, `camino`, `siblings`, `isRight`, `cofirmas`, `vieja` and
  `nueva`. None is a notice or its envelope.
- **Single node: none.** This RFC **introduces no consensus** at any stage,
  deliberately (`SECURITY.md` §6). On the contrary: it documents the procedure by which
  a position stays provable **without** the node.
- **`--dev`: none.** The verifier's manifest declares no `[features]` section
  and the crate has no test custodians.

**Publishing the schema publishes nothing that was not already published.** Every field
of the package is a wire response that `spec/RPC.md` already specifies; what this
document adds is the envelope that gathers them and the rules by which it is rejected.

**The package REPORTS, it does not judge.** It says how many co-signatures verify against
that head and that operator; **which witnesses count and how many are needed is decided
by the CLIENT** with its policy (§319), because whoever assembles the package may be the
operator. This RFC **does not move that frontier**, and the new document repeats it in its
place.

## References

Measured on `bb5322f` (§395), with the fingerprints in `sha256 | cut -c1-16` and the
lines in `wc -l`.

- `spec/rfc/PROCESO.md` `0eba90e3fe5d93f8` / 24 — the scope (`:3-6`), the states
  (`:10`) and the five rules.
- `spec/RPC.md` `3eaa7a433d53f579` / 867 — the section «Apagado — el fin de vida,
  declarado (nota 91)» [Shutdown — the end of life, declared (note 91)], 810..855, region
  sha `7963cdea2e82f540`; the versioning criterion at `:864`.
- `crates/zk-ssl-verify/src/main.rs` `2a99ef5e366d434c` / 582 — the header 1..90, region
  sha `293990fedc785833`.
- `crates/zk-ssl-verify/Cargo.toml` `a4f92b9047a12c97` / 51 — the precedent of the pointer
  that does not enumerate (`:25-27`), and the absence of `[features]`.
- `spec/README.md` `6dacd2d90986184d` / 142 — regions to edit 14..28
  (`8a36873fd7f99272`) and 135..142 (`f148d32d90d1de39`).
- `spec/vectors/zkssl-0.1.json` `3aa7b0623cfe1abf` / 64 · `zkssl-0.2.json`
  `d9c1b153a8d2311c` / 70 · `zkssl-0.3.json` `287f6a5a538ca7f7` / 70.
- `tools/canon.sh` `3ecad706a43c940e` / 302 — the conformance block `:278-286`.
- `spec/rfc/0002-lotes-y-transicion-de-hoja.md` `91bbdafa42d5ee51` / 389 and
  `spec/rfc/0003-compromiso-v2.md` `034c13a89173b825` / 276 — the moulds of this
  document.
- `PASTE-396-M` `db7e5bec4a04dce0` / 657, output `1423200c6f57acce` / 687 — the terrain of
  the milestone: the surface, the untyped schema, the state of `spec/`.
- `PASTE-396-PRE` `8f1509d1a95cb6bc` / 440, output `fc096dc5f441d600` / 535 — the shutdown
  section, the schema field by field, the 39 rejection calls and the regions of the
  README.

**Numbering.** **RFC-0001 stays reserved** for the hardening of the keystore KDF
(SHA-256 → Argon2id), undrafted, according to `spec/rfc/0002-lotes-y-transicion-de-hoja.md`
`:30-34`. This RFC takes **0004** consecutively, after 0002 and 0003.

**Provenance of two carried-over citations.** Everything above was measured in session 92
except two references, which are declared: `spec/RPC.md:868` —the criterion of when the
version goes up— was measured in session 66 and **is still in force because the file keeps
its fingerprint** (`3eaa7a433d53f579` / 867, re-measured today); and the reference to
`SECURITY.md` §6 for the single node is **copied from the Security section of RFC-0003**,
which is the ACCEPTED precedent.
