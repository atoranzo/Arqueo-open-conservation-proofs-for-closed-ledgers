# Evaluation: «vertical integration» and an «Arqueo cryptographic module» that is the standard of its niche

> **Español:** [`doc/integracion-vertical-evaluacion.md`](./integracion-vertical-evaluacion.md) is
> the original of this page, in Spanish; it prevails wherever the two differ. This translation is of
> commit `7766efb` (audit entry §917) and was produced by the project's assistant, a Claude Code
> session, in audit entry §918 (`GENAI.md`): it is a generated artefact, not text written by the
> author. Line citations of the form `file:N` refer to the Spanish files.

**The question, as it was asked** (01-10-2026): «study vertical integration and develop an Arqueo
cryptographic module, taking whatever is useful from winterfell and from the ecosystem so that it
is the standard of its niche/segment», accompanied by a long draft —four layers, a selection
matrix, a five-crate architecture, Rust code, a DSL, a proof format, a simulation table, security
parameters and a standardisation plan with a consortium— which the author contributed as is.

**Short answer.** The vertical integration the niche rewards **is already done** in the layers
Arqueo controls: the domain, the arithmetisation and, above all, **the verifier side**, which is
what a standard of evidence needs and what `spec/NUCLEO.md` freezes. In the proof-protocol layer it
exists to the exact extent of the winterfell fork: three new files and 350 lines. Going down to the
primitives layer —field, hash, FFT, GPU— **contradicts three measured decisions of the house** and
solves no bottleneck the tree has measured. The draft does not serve as a base: its code does not
compile against the tree's winterfell, the field it chooses aggravates a measured finding, its
figures have no measurement behind them and its «flagship application» is, word for word, what the
README says Arqueo does not prove. What **does** come out of the house's pieces is smaller and
measurable, and is named in §5: naming the module that already exists, the second implementation
that entry 85 has been asking for for months —**done in its first milestone**: 26 of 26 KATs of
the core in Python, from the spec—, the verifier in WebAssembly —which **verifies today** the 316
vectors of the ten manifests under wasmtime, with the same verdict as the native one— and the
layer-2 decision by RFC with the new figures.

**Status**: **ACCEPTED WITH CORRECTIONS** by the author on 07-10-2026, in §705 of
[`AUDITORIA.md`](../AUDITORIA.md). The corrections go in the block below, «Corrections (§705)»,
and prevail where they disagree with the text. When written: evaluated, and the two spikes of §5.2
and §5.3 **run in the same session** with their result inside (`tools/segunda/`). No new code in
`crates/`. Measured on `e8ac246` (S615) on the session's machine, rustc 1.97.0.

**Provenance**: written in a working session with the assistance of generative AI, on a branch
that has not been merged (`claude/nice-planck-ax35zl`), over a draft that also came out of a model
and that the author pasted in to contrast it, like those of `doc/ecst/`. «Integrar no es aceptar»
(integrating is not accepting): the acceptance that [`GENAI.md`](../GENAI.md) describes remained
pending until §705, which grants it with corrections. It does not touch `BACKLOG.md`,
`AUDITORIA.md` or `NOTICE`: that is the author's.

**Convention**: ✅ measured in this tree, with the command beside it · 📐 derived with an
arithmetic that can be reproduced · 📖 read in the primary source, with a citation; when the
source is GitHub or crates.io, with the date of reading (01-10-2026) · ⚠️ nuance · **CONFIRMED /
PARTIAL / FALSE / NO SOURCE / CONTRADICTS WHAT WAS MEASURED**, the verdicts of
`doc/ecst/VERIFICACION.md`, plus one of its own, **ALREADY DONE**, for what the draft proposes as
future and the tree carries sealed.

---

## Corrections (§705)

**Accepted with corrections.** The author accepts it on 07-10-2026, in §705 of
[`AUDITORIA.md`](../AUDITORIA.md), with scope «in this repository and in the NLnet project»: the
tree of this repository and the milestones of proposal 2026-11-009 to NLnet
(`spec/rfc/0005-nucleo-congelado.md:26`). The body is not rewritten: this block prevails where
they disagree. It separates the errors, which already were errors in `e8ac246`, from the snapshot
figures, true then and aged since. Today's were measured on `c990868`, the commit before §705. The
sections and rows are those of this document.

**What is accepted**, with the corrected basis below:

- layers 4 and 3 and the verifier side are already the house's, and the standard lies in what a
  third party reproduces (§4);
- layer 2 is not decided here: before that, a spike and an RFC (§4 and §5.4), still pending;
- layer 1, no (§4 and §6);
- the draft is the base of nothing (§3 and §6);
- the module is named with a map, without a crate (§5.1), and of the pieces only `kat_xmss` comes
  out (§5.5).

The scope covers, by name, three verdicts of §6: «Layer 1: no», «Layer 2: decide by RFC» and «a
new cryptographic module: no; name the one that exists».

### Errors, which already were errors in `e8ac246`

| # | where | it says | correction | evidence |
|---|---|---|---|---|
| E1 | §1, row «1 · Primitives»; §2, row 1 | 68 files use `fields::f64` | **89**, and none of the tried variants of the grep —with the fork, over the whole tree, by lines or only in `src/`— gives 68; today, 90. `fields::f128`, 3 | `git grep -l 'fields::f64' <rev> -- crates/ ':!crates/winter-*' \| wc -l`: 89 in `e8ac246`, 90 in `c990868` |
| E2 | short answer; §1, row «2 · Protocol»; §6 | the fork is «350 lines», «touched in the prover and the verifier» | the 350 is the **estimate** of the spike of RFC-0009 D-F («488 lines —350 in the fork and 138 of its own», `spec/rfc/0009-lo-que-revela-una-prueba.md:212-213`), not a measurement. Measured: +813/−6 in `e8ac246`; today **+829/−6** in 14 files, 11 touched and 3 new, and **617** without blanks or comments. The prover and the verifier alone, +500 gross and 399 net. Still below the threshold of 1,784 at which D-F is reverted (`:217-219`) | `git diff --no-index --numstat` of `src/` against the 0.13.1 `.crate` files from crates.io, with the sha256 of line 3 of `crates/winter-*/Cargo.toml`: `winter-air`, 7 files, +329/−3; `winter-prover`, 6, +427/−2; `winter-verifier`, 1, +73/−1. `NOTICE:39-40` |
| E3 | §2, row 9; Sources | «BACKLOG 92» for the suspended GPU | it is **entry 22**; 92 is the custody of the key | entry 22 of `BACKLOG.md`, SUSPENDED, with the RTX 5090 and the 12.4 GiB in its body; and entry 92. ⚠️ §792: until here it cited lines of `BACKLOG.md`: `:2019`, `:2065`, `:2092` and `:956` until §705, and four others in each seal from §706 to §709; those of §709, `:2055`, `:2101`, `:2128` and `:961`. In `8c02941` none of those of §709 lands in its place any more: entry 22 starts at `:2205` and entry 92 at `:968`. The entry's number does not move |
| E4 | short answer; §2, row 2; §4, «Layer 1»; §7 | the 31-bit field «aggravates a measured finding»: the collision of identities at 2³² | that finding **was already struck through** as corrected: the identity is a 256-bit digest. What is live is the key space, 2⁶⁴ (§82) | entry 15 of `BACKLOG.md`, its struck-through paragraph «Goldilocks es estrecho para identidades: 64 bits son colision en 2³²» [Goldilocks is narrow for identities: 64 bits are a collision at 2³²] and the note that follows it. ⚠️ §792: until here it cited its lines: `:1401-1408` until §705, and a new range in each seal from §706 to §709, the last one `:1437-1444`. In `8c02941` that range lands in another paragraph of the entry, and the struck-through text is at `:1527` |
| E5 | §2, row 4; §4, «Layer 1» | the only layer-1 decision with a number is `digest_of_proof` with Rescue (30.99 ms, the node's ceiling) | it is **done since §209** (`zkssl/0.2`): `digest_of_proof` is Blake3. The 30.99 ms are from §204, earlier | `digest_of_proof` in `crates/zk-ssl-hash/src/lib.rs`, with its `Blake3_256`. ⚠️ §891: until here it cited its lines, `:1716`, `:1724` and `:1737-1739`, which landed in their place in §705; in `8c02941`, the tree of §792, they were already three lines of the REGISTRO, and in `212eca2` the function is at `:1836` |
| E6 | §1, after the table; §4, «Layer 1» | «a rule written in layer 1»: «writing our own cryptography here would be a grave error» | it is a **local comment** that justifies a dependency, `chacha20poly1305`, the one of encryption at rest; it is not a project rule. The general written criterion is that of `CONTRIBUTING.md` §6, «Qué encaja y qué no» [what fits and what does not]: do not pile up surface without a present, measured problem. ⚠️ §891: until here it cited it by its lines, `:149-157` and, since §793, `:159-167`; in `212eca2` the section starts at `:191`, and each note of section 4 pushes it down | `crates/zk-ssl/Cargo.toml:35-37` |
| E7 | §2, row 11 | the house declares 127 conjectured, and 128 provable would cost «125.6 KB instead of 36.7» | those figures, like the «29-63» provable, are from the comparison circuit, with 32 queries and no grinding, not from production; the house attributed them to production until §698. Production —42 queries, blowup 16, grinding 21, quadratic extension— gives over the hidden proofs 127 conjectured, UDR 59 and LDR 80-86 depending on the circuit, less in age with a high `m`; since §708, the conjectured one with the DEEP term, which its formula does not discount: 112.8-115.8 depending on the circuit | `crates/stark-experiment/src/compliance_real_proof.rs:211-220`; the tests of §697; `SECURITY.md` §3.11 |
| E8 | §2, row 17 | «the size [of the WASM] has not been measured» | it contradicts §5.3, which measures it: 1,587,916 B | §5.3, the command block and the table |
| E9 | §2, rows 16 and 19; §4, «Layer 1»; §5.4 | `VISION.md` (P4, P6, P8 and §5) as a criterion in force | `VISION.md` declares itself **historical**: the position in force is in `README.md` and `doc/USE_CASES.md`. It is cited as a precedent, not as a rule | `doc/historia/VISION.md:1-4` |
| E10 | §5.1; §6 | «§580: the house does not breed code without a caller» | §580 does the opposite: it declares `podar` **without a caller** and keeps it by the author's decision. The phrase is from `spec/rfc/0012-el-ancla-de-cabezas.md:25` and `:121`, and holds as a criterion of that RFC, not as a rule of §580 | `AUDITORIA.md` §580 |

Two nuances, without an error range:

- **«35 AIRs»** (§1, §2 row 14, §4 and §5.4) is the census inherited from RFC-0009 D-F (`:211`,
  `:216`). The tree has no single census: outside the fork there are **38** `impl Air for`, 32 in
  `stark-experiment`, with falsifiers and experiments inside, 5 in `zk-ssl-air`, which are the
  kit's, and 1 in `crates/zk-ssl/src/instrumento_edad.rs`. It is cited as «the census of
  RFC-0009».
- **«Verifying costs 1.5 to 4 ms»** (§2, row 8) predates the hiding of §538. The v2.0 blueprint
  measured 3.98-4.93 ms (`doc/blueprint-v2.md` §1); here it has not been measured again.

### Snapshot figures: true in `e8ac246`, aged since

| where | in `e8ac246` | today | evidence |
|---|---|---|---|
| short answer; §5.2; §6 | 26 of 26 KATs of the core | **29 of 29**; **31 of 31** since §773 (`digest_pi` and the v2 leaf of the receipt); **34 of 34** since §800 (the scheme registry and the two v7 preambles of RFC-0020) | `git ls-tree` of `spec/vectors/nucleo`; `python3 -B tools/segunda/juez_nucleo.py` |
| §1 | the kit `arqueo-verify-v0.2.0`; the closure of `zk-ssl-verify`, 48 packages, and that of `zk-ssl`, 65 | **v0.4.2** (⚠️ §818: the current one is already **v0.4.3**; the two closures beside it have not been counted again. ⚠️ §726: and since §726, **v0.4.4**; the two closures, counted again on `95bc457` with the method of §1, remain at 61 and 68, and the `THIRD-PARTY.txt` of the published 0.4.4 lists the 61. ⚠️ §917: and since §917, **v0.5.0**, `zkssl/0.5`, whose published `THIRD-PARTY.txt` also lists 61; the two closures have not been counted again here); **61** and **68**: the three extra ones in the layer are `argon2`, `base64ct` and `blake2`, which only enter through the `argon2` of §702. The other three crates, 17, 25 and 52, the same | `doc/KIT.md:19`; the method of §1, `cargo tree -p <crate> -e normal --prefix none --locked`; `cargo tree -i` |
| short answer; §5.3; §7 | 316 vectors of ten manifests | **384 vectors in 12 manifests**: `ancla-cofirmada` (27) and `rotacion` (34) come in, and `paquete` (73), `conflicto` (17), `ancla` (22) and `completitud` (75) grow. They are 399 lines: the 15 that §693 adds name the cause of vectors that were already there. That, until §705; since §706, 390 vectors and 405 lines: age and payment gain three negatives each; and since §771, 396 vectors and 411 lines: consumption and completeness gain three each, from a node that rotates. Whether the WASM passes them today has not been measured | the `MANIFIESTO.txt` files of `FAMILIAS` (`tools/artefacto.sh:31`) |
| §5.2, third milestone | `paquete` 70, `consumo` 14, `conflicto` 16 and `ancla` 21 | the second verifier passes today `paquete` in its 83 lines, `consumo` 17 (14 until §771), `conflicto` 17 and `ancla` 22, and also `rotacion` 34. ⚠️ §810: `paquete` has 84 lines since §780, and since §810 it also passes `ancla-cofirmada`, 27, with ML-DSA-44 in Python: 201 in six families. ⚠️ §852: `paquete` has 89 since §852 (the five `rechazo-json-*`): 206 in six families. ⚠️ §712: `paquete` has 94, `consumo` 20 and `ancla` 23 since §712 (the nine of the clock): 215 in six families. ⚠️ §714: `ancla` has 44 since §714 (the twenty-one of the fork): 236 in six families | `tools/conformidad.sh tools/segunda/verificador.py`, as the «3 duodecies» block of the canon |
| §2, row 2; §3 | 63-bit range | **62 bits** since §641 (RFC-0017) | `crates/stark-experiment/src/range_check.rs:8-15`, `:76` |
| §5.3 | the `.wasm`, 1,587,916 B | it has not been compiled again | — |
| provenance | «a branch that has not been merged» | merged: `71de3d3` and `808e9b9` are in `origin/main` | `git merge-base --is-ancestor` |
| §1 | 14 RFCs; `zk-ssl-verify`, 7,443 lines; `zk-ssl`, 25,970 | they were 13 RFCs plus the template `0000`; today, 19 (0001-0019) plus the template, and 21 (0001-0021) since §790, and 23 (0001-0023) since §827; and 10,054 and 28,040 lines | `git ls-tree spec/rfc`; `wc -l` of the `.rs` files of `src/` |

### «Layer 1: no», with the corrected basis

The verdict holds, but not for the reasons of §4 —«by written decision and by three measurements»—
nor for the «three measured decisions» of the short answer. The written decision is a comment (E6)
and `VISION.md` is historical (E9). Of the three findings of `PRINCIPIOS.md` §7, the one on
identities was struck through (E4), the 63-bit ceiling belongs to the configuration without
extension, which production does not use (`crates/zk-ssl/src/lib.rs:221`), and the provable
figures were from another configuration (E7). The basis, all of it in force:

1. **The frozen core.** Goldilocks, `Rp64_256` and the XMSS signature do not change without a new
   version of the preamble, new vectors and an RFC (`spec/NUCLEO.md` §2 and §5; RFC-0005).
2. **The criterion of `CONTRIBUTING.md` §6, «Qué encaja y qué no»** (⚠️ §891: it said `:149-157`
   and, since §793, `:159-167`). No surface is piled up without a present, measured problem, and
   «there is no audit».
3. **No layer-1 bottleneck measured against the target.** The one of `digest_of_proof` was closed
   in §209 (E5). The ceiling the tree estimates for a chain of Rescue roots, ≈1,300 op/s
   (`doc/blueprint-v2.md` §1, ESTIMATED), stays above the RTGS target of 21-105 op/s. And changing
   Rescue in the trees is core (`doc/blueprint-v2.md` §6).

### What §5 and §8 left, at `c990868`

| point | status | evidence |
|---|---|---|
| §5.2 and §8.1, the second implementation | done, in four milestones | entry 85 of `BACKLOG.md`; §623 and §626. ⚠️ §792: until here it cited it also by its line, `:720`; in `8c02941` entry 85 starts at `:727` |
| §5.5, `kat_xmss` | done | §623 |
| §5.6, the report of §575 | sent | §625 |
| §5.6, `NOTICE`: from eight and two to eleven and three | corrected | `NOTICE:39-40` |
| §5.6, the entry saying that D-F did not weigh PR 293 | pending | no entry up to §704, nor `BACKLOG.md` nor RFC-0009, names it |
| §5.1 and §8.5, the one-page map of the module | pending | only this document names it |
| §5.4 and §8.3, the spike and the layer-2 RFC | pending | none of RFCs 0001 to 0019 weighs the D-F of RFC-0009 again |
| §5.3 and §8.4, `getrandom` and WASM in the issue to RustCrypto | in the draft since §793, not sent; until then, pending | `doc/issue-rustcrypto.md`, part 5, with `cargo tree -i getrandom@0.4.3 --locked --offline -p zk-ssl-verify -e normal`; until §793 it named neither of the two. ⚠️ §793: the verifier carries another `getrandom`, 0.2.17, through `rand_core` 0.6.4 of `zk-ssl-air` (the salt, §535): §5.3 only names the one of `xmss`, and the issue does not fix the house's |
| §8.2, the acceptance of `tools/segunda/` | pending: §705 does not cover it | `tools/segunda/README.md` |
| §8.6, «eight tools» | corrected in §700 | `GENAI.md` §«Cómo se usa, exactamente» |

---

## 1. What the house has, layer by layer ✅

The draft proposes four layers. They are walked through with what the tree has in each one,
because the question «what to integrate vertically?» is only answered by knowing what is already
owned.

| layer of the draft | what there is in the tree | where it is measured |
|---|---|---|
| **4 · Domain** | the settlement layer (`zk-ssl`, 25,970 lines), 14 RFCs, the evidence package with twelve forms, 380 vectors that are never rewritten | `spec/PAQUETE.md`, `spec/rfc/`, `spec/vectors/`, `tools/canon.sh` |
| **3 · Arithmetisation** | 35 AIRs of its own in `stark-experiment`, with their guardians: vacuity by mutation (`mutation.rs`), slots and periodics (`tools/check_constraint_layout.py`), the FV-1 cell census, the FV-2 SMT spike, the suite that counts what a proof reveals (`instrumento_revela.rs`) and the nine falsifiers of hiding | `doc/VERIFICACION_FORMAL.md`, RFC-0009 E2 and D-V |
| **2 · Protocol** | winterfell 0.13.1 and the fork: `Oculta<A>` (140 lines), `Marca` (69), `LectorAcotado` (84), 350 lines touched in the prover and the verifier; `MerkleConSal` (458) in `zk-ssl-air`; `winter-fri` unforked. Parameters: 42 queries, blowup 16, grinding 21 bits, quadratic extension | `crates/winter-*/README.md`, `crates/zk-ssl/src/lib.rs:216-227`, RFC-0009 D-F |
| **1 · Primitives** | Goldilocks `f64` in production (68 files; `f128` in 3, experiments), Rescue Prime `Rp64_256` from `winter-crypto`, Blake3 in the transcript, `xmss 0.1.0-pre.0` pinned with `=`, `chacha20poly1305` and `sha2` from RustCrypto | `range_check.rs:1-5`, `zk-ssl-hash`, `SECURITY.md` |
| **the verifier side** (the layer the draft lacks and the niche demands) | `zk-ssl-verify` (7,443 lines) without the prover in its closure, with a gate; the frozen core with its KATs; the kit `arqueo-verify-v0.2.0` with `conformidad.sh` | `spec/NUCLEO.md`, `doc/KIT.md`, `tools/canon.sh` (gate H2) |

And a rule written in layer 1, which the draft inverts: *«writing our own cryptography here would
be a grave error»* (`crates/zk-ssl/Cargo.toml`, on encryption at rest). The house does not write
primitives: it chooses them, pins them with `=`, declares them and watches them.

**Dependency closure, measured today** ✅ (`cargo tree -p <crate> -e normal --prefix none
--locked | sed 's/ (.*//' | sort -u | wc -l`; it counts unique packages, the crate itself included,
with the fork inside through the `[patch]`):

| crate | unique packages |
|---|---|
| `zk-ssl-hash` | 17 |
| `zk-ssl-air` | 25 |
| `zk-ssl-verify` | 48 |
| `stark-experiment` | 52 |
| `zk-ssl` | 65 |

---

## 2. The draft, claim by claim

| # | claim of the draft | verdict | what was measured or read |
|---|---|---|---|
| 1 | «Winterfell primarily used `f128`» and the example code imports `fields::f128` | **FALSE** for this tree ✅ | 68 files use `fields::f64` and 3 `f128`; `Rp64_256` fixes Goldilocks and `range_check.rs` tells of the migration and its finding |
| 2 | 31-bit fields (BabyBear, M31) «multiply speed by 10× to 50×» | **NO SOURCE** and **CONTRADICTS WHAT WAS MEASURED** 📖 | `PRINCIPIOS.md` §7.2: *«Goldilocks is too narrow for identities»*, collision at 2³² with 64 bits. With 31 bits the collision drops to 2¹⁵ and the 63-bit range check (`range_check.rs`) would be left at 30. The multiplier brings no measurement |
| 3 | «Circle STARKs reduce memory overhead by up to 80 %» | **NO SOURCE** | no measurement, no citation |
| 4 | Poseidon2 inside the circuit, Blake3 outside | **PARTIAL** 📖 | Blake3 is already there. Rescue Prime is CORE (`spec/NUCLEO.md` §2): changing it is a new version of the preamble, 380 vectors and an RFC, not a module. ⚠️ What is measured is narrower: `digest_of_proof` with Rescue costs 30.99 ms and is the node's ceiling, 30 → 436 op/s with a non-algebraic hash (`doc/DIAGNOSTICO_ESCALADO.md` §0.bis). It is **one** place, with its number |
| 5 | «Arqueo must add blinding rows at the end of the table so as not to reveal private inputs» | **ALREADY DONE** 📖 | RFC-0009 D-G, §538: T random rows, a random column, salt in the leaves and a randomised quotient; the E2 suite counts zero literals in 100 cells since §538 |
| 6 | Architecture `arqueo-math / crypto / air / prover / verifier`, «eliminating uncontrolled external dependencies» | **CONTRADICTS ITS OWN TITLE** ✅ | it is the winterfell workspace renamed, and the draft's `Cargo.toml` depends on `winter-air`, `winter-crypto`, `winter-math`, `winter-prover` and `winter-verifier` 0.13. It does not integrate: it wraps |
| 7 | «Winterfell FRI supports multi-stage trace commitments and randomness at proving time» | **PARTIAL** 📖 | the auxiliary segment exists (`EdadAir` uses it); the randomness for hiding is not upstream: its issue 9 has been open since 2021 and its README lists it as «Planned» (read 01-10-2026) |
| 8 | The figures of the «simulation»: 1 M accounts in 5.40 s, verifier in 4.8 ms, 136 KB proof | **NO MEASUREMENT** ✅ | the draft's «simulator» hashes fictitious data and applies a formula; it neither generates nor verifies a proof. `CONTRIBUTING.md` §1: *«Figures without a measurement behind them do not enter»*. What the house measures: a hidden payment proof weighs between 145,953 and 167,967 B (`metrics.rs:82-83`) and verifying costs 1.5 to 4 ms (`ARQUITECTURA.md`, «Métricas de la capa, medidas» [metrics of the layer, measured]) |
| 9 | GPU, CUDA, clusters: «10 million accounts in seconds» | **ALREADY MEASURED AND SUSPENDED** 📖 | BACKLOG 92, §305 to §318: on an RTX 5090 a proof inside the zkVM costs 40.9 s plus 23.8 s of compression; *«there is no crossing of curves»*; suspended with a trigger wired in the node at 12.4 GiB |
| 10 | Recursion, a «Master Proof of less than 100 KB» | **ALREADY MEASURED** 📖 | §307: the succinct receipt measures exactly 223,234 B, constant, 3.3 times a loose proof; aggregating pays off from about four proofs; `VISION.md` §3.2: winterfell does not support recursion |
| 11 | «128 post-quantum bits» with `F_{p^4}` ≈ 2¹²⁴, and in the next row `F_{p^5}` | **CONTRADICTS ITSELF** 📐 and **CONTRADICTS WHAT WAS MEASURED** 📖 | 2¹²⁴ < 2¹²⁸ in the same table. The house declares **127 conjectured bits** and that 128 **provable** would cost 125.6 KB per proof instead of 36.7 (`lib.rs:211-215`, `FIVE_BACKENDS.md`, `PRINCIPIOS.md` §7.4). The draft does not distinguish conjectured from provable, which is the distinction the house measured |
| 12 | The soundness formulas, with the «Jon-Sion proximity theorem» | **NO SOURCE** | no result with that name can be located; none of the bounds is computed or checked against a source |
| 13 | «Verification < 10 ms on CPU and < 300,000 gas on EVM» | **NO SOURCE** | no EVM verifier exists in the tree nor is measured; the only succinct wrapper measured cost 23.8 s of GPU (§306) |
| 14 | The `#[arqueo_circuit]` DSL «reduces the learning curve from 3 weeks to 15 minutes» | **FALSE** as written ✅ | the draft's macro ignores the annotated structure and always emits the same balance AIR: it compiles nothing, it copies it. What the house has in its place: 35 hand-written AIRs with guardians born of real failures (§39, §50, §66.2) and an executable spec that reproduces the circuit byte for byte (`doc/fv/`) |
| 15 | «APF v1.0 format», binary, self-documented, with Borsh or Protobuf | **ALREADY EXISTS** under another name 📖 | `spec/PAQUETE.md` (twelve forms, order of checking, catalogue of rejections, output contract), `spec/NUCLEO.md` (what a second implementation reproduces byte for byte), `spec/vectors/nucleo/` (KATs). A second format for the same thing is «two definitions», which §254 and §255 forbid |
| 16 | «Proof of Reserves 2.0»: assets ≥ liabilities, Basel III, DeFi, L2 | **CONTRADICTS THE DECLARED SCOPE** 📖 | `README.md`: *«It proves conservation, not solvency: the proofs speak of the ledger, not of the world»*. `VISION.md` P4: *«Widening the scope is not a good in itself»*. `doc/ESCALADO.md`, header: widening the scope is a mission decision under VISION §5, not a backlog one. `ROADMAP-ECOSISTEMA.md`: the house competes in settlement and ISO 20022, *«not in the Ethereum niche»* |
| 17 | C-FFI, Java, Python bindings; WASM «< 1 MB» | **PARTIAL** ✅ | none of that exists. But the verifier **compiles today** to `wasm32-wasip1` (§5.3); the size has not been measured |
| 18 | «100 % formal verification of the verifier core and 0 vulnerabilities in AIRs» | **CONTRADICTS WHAT WAS MEASURED** 📖 | `doc/VERIFICACION_FORMAL.md` §0: the suite proves points, not the universal; FV-1 is a syntactic census; FV-2 an SMT spike over `circuit_refund`; FV-3 «the horizon, named without promising it» |
| 19 | Donation to a foundation, an «Arqueo Consortium», a technical committee, the Big Four accepting APF as legal evidence | **NO SOURCE** and outside the principles 📖 | `PRINCIPIOS.md` §8 withdrew a more modest roadmap than this one because *«saying it would breach the principle of transparency»*; §10: *«What is missing is decentralisation, external audit and adoption. None of the three is a cryptographic problem»*. `VISION.md` P8: legitimacy does not depend on the author or on a seal |
| 20 | A grinding module of its own (`arqueo-pow`) with Blake3 | **DUPLICATES** what is already used ✅ | `ProofOptions` carries `grinding_factor` (`winter-air/src/options.rs:98`) and the house sets it at 21 bits (`lib.rs:220`). A grinding outside the prover's transcript binds nothing: it has to live inside the proof |
| 21 | An `F_{p^4}` of its own over BabyBear, with Frobenius and norm | **UNNECESSARY** 📖 | winterfell brings quadratic and cubic extensions and the house uses the quadratic one over Goldilocks (`lib.rs:221`); the draft's code carries neither tests nor vectors and uses `%` over `u64`, which is not constant-time |

What the draft **does** get right, said without haggling: that the verifier has to live outside
the prover, that the specification has to be decoupled from the code, that second implementations
and vectors are needed. The four things have been in `ROADMAP-ECOSISTEMA.md` since August, the
first three sealed (§197 to §199) and the fourth is entry 85 of the BACKLOG.

---

## 3. The draft's code, read against the tree ✅

It is not compiled: it is read against the vendored winterfell, which is the one the tree uses.

- `use winter_prover::{DefaultTraceLwe, ...}`: **zero occurrences** in `crates/winter-prover/src`.
  The type is `DefaultTraceLde` (`trace/mod.rs:13`).
- `trace.fill(|execution_trace| { execution_trace[0][0] = ... }, |buf| { ... })`: the real
  signature is `fill<I, U>(init: I, update: U)` with `I: FnOnce(&mut [B])` and
  `U: FnMut(usize, &mut [B])` (`trace/trace_table.rs:164-168`). The first closure receives **one
  row**, not a matrix; the second receives **two** arguments. Neither of the draft's two closures
  type-checks. And `buf[0] = buf[0]` is a no-op that leaves the balance column with the previous
  row's value only by accident.
- The draft's AIR imposes `S_{t+1} = S_t + Δ_t` and nothing else. Without a range check on `Δ_t`,
  a «negative» `Δ` is a large field element and the balance wraps around: it is the class of
  failure that `ARQUITECTURA.md` closed under «Ninguna vía para crear dinero» [no way to create
  money] and that `range_check.rs` and `double_entry.rs` protect with 63 bits and double entry.
  The draft leaves it for a «Phase 2».
- The `#[arqueo_circuit]` macro does not read `#[trace_col]`, `#[public_input]`,
  `#[transition_logic]` or `#[boundary_constraints]`: it emits the same AIR whatever the
  structure. A DSL that does not compile its input is not a DSL.
- The «simulator» builds no proof: it chains hashes of zeros and multiplies integers. Its
  milliseconds measure that loop.

Verdict of this section: **no line of the draft is the base of anything**. What a house module
needs in code is already written, with tests, in the crates of §1.

---

## 4. Vertical integration: what it means for Arqueo, layer by layer

Integrating vertically is controlling what, if it changes, changes what valid money is or what a
verifier accepts. By that criterion:

**Layers 4 and 3, and the verifier side: they are already the house's, and that is where the
standard is.** A standard of evidence is not a prover: it is **what a third party reproduces
without trusting the author**. That is `NUCLEO.md` plus `PAQUETE.md` plus the vectors plus the
kit. HBS-STATE and `mtc-core` came out of here with that shape —specification, reference
implementation, vectors, a judge— and it is the shape Arqueo's niche admits. No other format or
other name is needed.

**Layer 2: the house's to the exact extent of the fork, and it is an open decision.** What was
measured in session on 01-10-2026 📖: winterfell has had no commit on `main` since 19-07-2025; its
zero-knowledge PR 293 has been open since July 2024, with failures reported in January 2026 and no
answer; its author, from Miden, merged on 02-01-2026 the migration of the Miden virtual machine to
Plonky3; on crates.io seven crates depend on `winter-prover`, none of them large. The
reversibility clause of RFC-0009 D-F —«if winterfell publishes hiding of its own, winterfell is
followed and the fork is withdrawn»— has today an unlikely trigger. And what the house discarded
in D-F for its cost, Plonky3, was measured today ✅ with a test crate outside the tree
(`p3-uni-stark`, `p3-fri`, `p3-merkle-tree`, `p3-commit`, `p3-challenger`, `p3-dft`, `p3-field`,
`p3-matrix`, `p3-goldilocks`, `p3-symmetric`, `p3-blake3`, `p3-air`, all 0.8.0):

| measurement | value |
|---|---|
| dependency closure (same method as §1) | **54** packages, against 52 for `stark-experiment` |
| hiding in `p3-fri` 0.8.0 | `HidingFriPcs`, `HidingFriPcsProverData` |
| hiding in `p3-merkle-tree` 0.8.0 | `MerkleTreeHidingMmcs` |
| hiding in `p3-uni-stark` 0.8.0 | `is_zk` in 44 places, `randomized` in 9 |
| lookups in `p3-uni-stark` 0.8.0 | `lookup` in 24 places |

⚠️ What this measurement does **not** say: whether Plonky3's univariate STARK covers the auxiliary
segment that `EdadAir` needs (D-F said no in the version of the time; in 0.8 there are lookups,
and whether they suffice has not been measured), nor how much reopening 35 AIRs costs (D-F: 10,100
lines of implementation and 4,341 of transition), nor that Plonky3 fails closed with a malformed
proof: its README warns that *«the verifier may panic on certain malformed proofs»* and asks for
`catch_unwind`, which is exactly what §575 fixed in the fork with a bounded reader. Migrating does
not remove that debt: it moves it.

**Layer 1: no, by written decision and by three measurements.** The rule of the layer's
`Cargo.toml`; P6 of `VISION.md`, *«coherence over brilliance»*: an optimisation that adds a
failure surface that is hard to audit is discarded even if it improves latency or size; and the
three findings of `PRINCIPIOS.md` §7 on the field (identities, 63-bit ceiling, conjectured versus
provable), which a 31-bit field worsens in all three. The only layer-1 decision with a number
behind it is that of the hash on **one** hot path of the node (§4 of the table), and it is a change
of the LEDGER, not of the core.

---

## 5. What does come out of the house's pieces, with its measurement

### 5.1 Naming the module that already exists, without creating a crate

The «Arqueo cryptographic module» is the set that today has no name but does have a contract:
`zk-ssl-hash` (the format primitives, one definition), `zk-ssl-air` (the AIRs a third party
judges and `MerkleConSal`), `zk-ssl-verify` (the verifier without the prover), the three new files
of the fork (`oculta.rs`, `marca.rs`, `acotado.rs`), the snapshot of the pristine prover
(`kat_probador.rs`), the falsifiers and `mutation.rs`. Its contract is `spec/NUCLEO.md`. A new
crate without a caller goes against §580 (*«the house does not breed code without a caller»*);
what is missing is a one-page map that names it, and it enters through an entry.

### 5.2 The second implementation: entry 85, done in its first milestone ✅

BACKLOG 85 has said it since before this draft: *«a spec that only one code implements is not a
spec, it is documentation»*, and sets the bar: **a verifier in another language that passes the
vectors**. The first measurable milestone is the frozen core, and it is done:
`tools/segunda/nucleo.py`, Python with no dependencies, written from `NUCLEO.md` §6 and from no
`.rs` of the house, with `juez_nucleo.py` as its only caller.

```text
python3 tools/segunda/juez_nucleo.py
  autotest de las primitivas (BLAKE3 contra blake3 1.8.5; Rescue contra el vector Sage de winter-crypto): VERDE
  OK   acuse_digest … OK   recibo_digest            (26 líneas)
  nucleo: 26 de 26 vectores reproducidos byte a byte
```

What it uncovered, which is what the entry exists for (detail in `tools/segunda/README.md`):

| gap | where | measured |
|---|---|---|
| `hoja_de_acuse(hashPrueba, seq, n)` was not in §6; `RPC.md` gives it with `epoca` in «`zkssl_ackPath`» and says `epoca = logSeq + 1` in «El acuse en la respuesta (§274)» [the acknowledgement in the response], but nobody says that the leaf's `seq` is that `logSeq` | `NUCLEO.md` §6, `PAQUETE.md` §4 step 3/3 | with `seq` it does not reproduce; with `seq + 1`, it does |
| `hash_del_lote`: the encoded length is that of the bytes, `48·k`, not `k` | `NUCLEO.md` §6, RFC-0014 E2 | with `k` it does not reproduce; with `48·k`, it does |
| `cima`: it does not say whether each leaf goes through `mmr_hoja` inside | `NUCLEO.md` §6 | it does, RFC 6962 reading |
| the KATs encode the u64 in two ways, eight LE bytes or a hex number, depending on the field | `spec/vectors/nucleo/` | a judge knows it from outside |

None changes a byte of the format: they change the **prose** of the spec, which is what a second
implementation measures, and the three clarifications are already in `NUCLEO.md` §6 with the
error recorded.

**Second milestone, done in the same session** ✅: `tools/segunda/kat_xmss/xmss.py`, the XMSS^MT
verification of RFC 8391 written from the RFC, and `juez_cabezas.py`, which walks the 331 files
with heads of `spec/vectors/`:

```text
python3 tools/segunda/juez_cabezas.py
  cabezas: 376 de 418 verifican; cofirmas: 7 de 11; ficheros con cabezas: 323;
  con alguna que NO verifica: 46, de ellos positivos segun su MANIFIESTO: 0
  FALSADOR: un byte volteado en la firma y otro en el preambulo NO verifican: VERDE
```

The 42 that do not verify are the ones the negatives fabricate, and each one falls by the rule the
manifest assigns to it. With this the second implementation covers steps 1/3 and 2/3 of the
package —the head recomposes, the signature verifies, the preamble is the expected one, the
embedded index stays below the declared one— and the witness co-signatures.

**Third milestone, done** ✅: `tools/segunda/verificador.py`, the second verifier, with the
command's contract and the five forms without STARK, judged by `tools/conformidad.sh` with the
binary's manifests: `paquete` 70 of 70, `consumo` 14 of 14, `conflicto` 16 of 16, `ancla` 21 of
21, and output identical line by line to the binary's on every positive. With this the conformance
of those four families stops being self-conformance: two codes, one in Rust and another in Python
written from the spec, say the same thing input by input. The judges enter the canon as a block of
their own (`tools/canon.sh`, «3 duodecies»), at pin zero: what does not run does not protect. The
whole `--sello`, run with the block inside: **VERDE**, 18 crates on their pins, 1,054 s of tests,
the nine tools, the ten manifests and the reproducible artefact.

### 5.3 The verifier in WebAssembly: it verifies the 316 vectors under wasmtime ✅

```text
cargo build -p zk-ssl-verify --release --target wasm32-wasip1 --locked   -> 1.587.916 B
tools/conformidad.sh con tools/segunda/wasm_runner.py (wasmtime 49.0.0), diez manifiestos:
  ancla 21/21 · completitud 73/73 · conflicto 16/16 · consumo 14/14 · edad 11/11 · pago 9/9 ·
  paquete 70/70 · pendiente 9/9 · prenda 9/9 · rechazo 84/84          -> 316 de 316, como el nativo
```

| measurement | native | `wasm32-wasip1` |
|---|---|---|
| size | 2,517,640 B | 1,587,916 B |
| the ten manifests, wall clock | 9.28 s | 32.78 s |
| one vector, average of five | 18.4 ms | 93.3 ms, 66.8 of them starting Python and wasmtime |

Without touching a line of the verifier. On `wasm32-unknown-unknown`, the browser target, it is
stopped by the random number generator that `xmss` drags in through `rand 0.10` to generate keys,
which the verifier does not use: one more line for the issue to RustCrypto of entry 77. ⚠️ What
has **not** been measured: the `.wasm` optimised for size, nor `wasm-opt`; and the draft promised
«< 1 MB» without measuring, and it measures 1.5 MiB as it comes out. The historical roadmap marked
WASM as «evaluate», but for the prover (`ROADMAP-ECOSISTEMA.md:34`); the verifier is the one that
makes sense, because it is what a third party runs.

### 5.4 Layer 2 by RFC, with the new figures

D-F weighed two paths and did not see the third (PR 293), and the premise of its reversibility has
changed. The honest thing is to weigh again with the same method, not to decide here: a spike like
SPIKE-B-P4 that measures (a) whether `p3-uni-stark` 0.8 covers `EdadAir`, (b) real reopening lines
of the 35 AIRs, (c) the snapshot of D-R against Plonky3 switched off and the E2 suite switched on,
(d) a bounded reader or `catch_unwind` in the verifier, and (e) whether upstream winterfell remains
silent. With that, an RFC decides between taking on the fork under its own name or migrating, and
sets a date. Both outcomes satisfy VISION §5 if they are declared; the one that does not is
carrying on with an exit clause nobody watches.

### 5.5 Modularising like `hbs-state` and `mtc-core`: decided, piece by piece

The criterion is that of `doc/MTC.md`: what comes out is what shares with Arqueo the
infrastructure and the discipline, **not the problem**, and has consumers outside.

| piece | does it come out? | why |
|---|---|---|
| the winterfell fork | **not yet** | its exit depends on the layer-2 decision (§5.4): extracting it before measuring the spike is deciding without measuring. If the RFC says «take on the fork», it comes out under its own name and `NOTICE` changes first |
| `spec/` and the vectors | **no** | they are the problem, not the infrastructure; and the canon's double thread (`check_nucleo`, the generated OpenRPC, the vectors of the benches) lives off having them beside the code. What makes a spec a standard is not a separate repository: it is two implementations and a harness, and that is what this branch builds. The distributable form already exists: the kit |
| the AIR hygiene tools | **later** | they are good for any winterfell AIR, but the Python part is tied to the house's conventions, and the winterfell ecosystem is running out of consumers (§4) |
| **`tools/segunda/kat_xmss/` with `xmss.py`** | **yes, and it is ready** ✅ | verifying XMSS^MT of RFC 8391 is not Arqueo's problem and anyone who signs with RFC 8391 needs it, `hbs-state` included. Zero dependencies, zero bytes of the protocol. It closes the «missing KAT» of entry 77: a corpus produced by the pinned crate and verified by another implementation, 8 of 8 and three falsifiers silenced. Split with `git subtree split` and pushed as branch `xmss-kat-main` (§623); it serves as a repository on its own |

The rest of `tools/segunda/` —`nucleo.py`, the judges of the core and of the heads— does **not**
come out: it is the second implementation of the protocol and its place is beside the spec and the
vectors it judges, in the canon.

### 5.6 The debt of §575 and the record

Outside this document but before everything above: the private report to winterfell of the abort
by memory reservation (§575, *«letting them know is a debt»*), through the channel its policy
asks for; and the entry that says that D-F did not weigh PR 293 and that `NOTICE` counts eight
files touched and two new when they are eleven and three since §537 and §575.

---

## 6. Verdict

| question | answer |
|---|---|
| Integrate vertically? | Already done, in the layers that matter for the niche. Layer 2: decide by RFC. Layer 1: no |
| A new cryptographic module? | No: name the one that exists. A crate without a caller goes against §580 |
| Take what is useful from winterfell? | Already taken: 350 lines and three files, with the snapshot that proves that switched off it is winterfell byte for byte |
| And from the ecosystem? | Plonky3 0.8 carries hiding and lookups with a closure of 54 packages, measured; whether it covers `EdadAir` is the first thing to measure |
| Be the standard of the niche? | With entry 85: a second implementation that passes the vectors. Four milestones done and in the canon: 26 of 26 KATs of the core, the signed heads of every vector with XMSS^MT from RFC 8391, the second verifier with 121 of 121 entries of the four families without STARK, and the STARK verifier in Python over the hidden and salted proofs of the six families that carry them, 23 of 23 pairs with the binary's verdict and cause (§626). What remains a promise: the five AIRs, transcribed from the only place where they exist, and the whole command of those forms in the second code |
| The draft? | No line serves as a base; four of its ideas are already sealed and one contradicts the declared scope |
| Modularise like `hbs-state` and `mtc-core`? | One piece, today: `kat_xmss` with `xmss.py`, ready for `git subtree split`. The fork, when the layer-2 RFC decides; the spec, never by that criterion |

---

## 7. What this document does NOT claim

- That Plonky3 is better than the fork: only that its cost is no longer the one D-F weighed and
  that it is measured before deciding.
- That the verifier works in a **browser**: it works under wasmtime with WASI, 316 of 316. The
  target without a system is stopped by `getrandom` via `xmss`, and nothing beyond that has been
  measured.
- That a 31-bit field is slower or faster: that it aggravates three measured findings on
  identities and range, and that the draft's multiplier brings no measurement.
- Nothing about solvency, reserves or Basel III: the README says the proofs speak of the ledger,
  not of the world, and this document does not move that.
- That the draft's figures are false: that they **are not measured**, which is the only thing the
  house needs to know in order not to copy them.
- That the second implementation is a second implementation of the **protocol**: it is one of the
  frozen core, of the signed head with its co-signatures, of the five forms without STARK and,
  since §626, of the STARK verification of the six families with a proof. It is not one of its
  five AIRs, transcribed from the `.rs` because no RFC writes their constraints, nor of the
  command of those forms, which in the second code is not yet composed with the head and the
  signature.

---

## 8. Pending items it leaves

1. ✅ **Done in §626.** Entry 85, fourth milestone: a winterfell STARK verifier in Python for the
   forms with a proof (FRI, DEEP and the AIRs as judges). ⚠️ **Correction (§626):** here it said
   «the 23 AIRs», and that is inaccurate: the package verifies **five** —Banda, Edad, Prenda,
   CobroPendiente and PagoEnCurso—; the 23 of `stark-experiment` are the prover's and none travels
   in a vector. **The language decided, with a measurement**: Python, not a compiled language.
   What was measured on this machine over the primitives already written —Rescue permutation
   1.28 ms, one Blake3 merge 78 µs, one multiplication in the quadratic extension 0.65 µs—
   projects ~0.73 s per proof and ~59 s for the whole catalogue (81 STARK proofs), of the order of
   what suites of the tree already take. A second compiled verifier would go 10× to 50× faster and
   it is not needed: the second implementation is not measured by its speed but by saying the same
   bytes, and putting it in another language would add a compilation box for a conformance that
   does not need it. It stays in Python, with the permutation and the Blake3 it already has. The
   milestone is writing FRI, the DEEP sampling and the AIRs as constraints; the cost is one of
   concept, not of performance. Measured on doing it: 0.2 to 0.5 s per proof, below the projected
   0.73; the whole judge, 12 s.
2. The author's entry that accepts or reverts what this branch leaves: `tools/segunda/` with
   `kat_xmss/`, the «3 duodecies» block of the canon and the prose clarifications of `NUCLEO.md` §6
   and §8; and, if he wants it outside, the `git subtree split` of `kat_xmss/`.
3. The layer-2 spike (§5.4), before any RFC.
4. `wasm32-unknown-unknown`: one more line for the issue to RustCrypto (entry 77), `xmss` drags
   `rand` into the verifier; and measuring the `.wasm` optimised for size.
5. The one-page map of the module (§5.1), as an entry, without a crate.
6. The figure «eight tools in `tools/`» of `README.md`, `README_EN.md`, `GENAI.md` and
   `RESUMEN_EJECUTIVO.md` no longer squares with the canon's loop, which runs nine; it is not
   touched here because the new block is not one of them, and it is left said.

---

## Sources

- This tree at `e8ac246`: `README.md`, `PRINCIPIOS.md` §6 to §10, `CONTRIBUTING.md` §1,
  `ARQUITECTURA.md` («Métricas de la capa, medidas», «Ninguna vía para crear dinero»),
  `SECURITY.md` §3.7, `spec/NUCLEO.md` §1 and §2, `spec/PAQUETE.md`, `spec/rfc/0009-*.md` (D-F,
  D-G, D-I, D-R, D-V), `doc/VERIFICACION_FORMAL.md` §0, `doc/DIAGNOSTICO_ESCALADO.md` §0.bis,
  `doc/ESCALADO.md` (header), `doc/historia/VISION.md` (P4, P6, P8, §3.2, §5),
  `doc/historia/ROADMAP-ECOSISTEMA.md`, `BACKLOG.md` 77, 85, 92, `AUDITORIA.md` §533, §575,
  `crates/zk-ssl/src/lib.rs:211-227`, `crates/zk-ssl/src/metrics.rs:82-83`,
  `crates/stark-experiment/src/range_check.rs:1-25`, `crates/winter-prover/src/trace/mod.rs:13`,
  `crates/winter-prover/src/trace/trace_table.rs:164-168`, `crates/winter-air/src/options.rs:98`,
  `crates/zk-ssl/Cargo.toml` (the comment on cryptography of our own).
- GitHub and crates.io, read on 01-10-2026: `facebook/winterfell` (commits on `main`, issue 9,
  PR 293, PR 383, security policy), `0xMiden/miden-vm` (README, `Cargo.toml`, PR 2472),
  `Plonky3/Plonky3` (README, CHANGELOG), `crates.io/api/v1/crates/winter-prover` and its reverse
  dependencies.
- Sources of Plonky3 0.8.0 downloaded by `cargo fetch` in the test crate and read with `grep`:
  `p3-fri`, `p3-merkle-tree`, `p3-uni-stark`.
- `tools/segunda/`: `nucleo.py`, `juez_nucleo.py`, `wasm_runner.py` and its `README.md`, with the
  outputs of the two spikes; `winter-crypto 0.13.1`, `src/hash/rescue/rp64_256/{mod,tests}.rs`
  (the permutation and its Sage vector); `blake3 1.8.5` from the `Cargo.lock` (the 19 reference
  outputs); `wasmtime` 49.0.0 from PyPI.
- The draft contributed by the author, in its entirety, as the object of the evaluation and not as
  a source.
