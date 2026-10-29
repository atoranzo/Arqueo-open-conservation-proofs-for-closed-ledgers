# Contribution guide — Arqueo (formerly ZK-SSL)

> **Español:** [`CONTRIBUTING.md`](./CONTRIBUTING.md) is the original of this page, in Spanish; it
> prevails wherever the two differ. This translation is of commit `7766efb` (audit entry §917) and
> was produced by the project's assistant, a Claude Code session, in audit entry §918 (`GENAI.md`):
> it is a generated artefact, not text written by the author. Line citations of the form `file:N`
> refer to the Spanish files.

Read [`SECURITY.md`](./SECURITY_EN.md) first: the state of the project and its
known limits condition what kind of contribution makes sense here.

---

## 1. Philosophy

- **Measure before asserting.** The distinctive contribution of this
  repository is the empirical comparison of proof systems. Every claim of
  performance or of security must bring the number, the method or the
  proof that backs it. **Figures with no measurement behind them do not
  enter.**
- **Distinguish the measured from the estimated from the assumed.** A
  «~X» with no origin is a debt, not a datum.
- **Soundness rules over speed.** A change in a circuit or in the
  transition function is a change in **what is valid money**. It is
  reviewed with more care than anything else.
- **Honesty about the state.** If a component is scaffolding, say so in
  the code and in the PR. Do not present a skeleton as an implementation.
- **No exaggeration.** This project does not promise a new world order: it
  proposes and measures concrete cryptographic properties.

⚠️ **And a rule this project learned at its own expense** (`AUDITORIA.md`
§59.2): **every tool, helper or sweep declares its coverage, and the
declaration is checked.** A check applied to part of the code without
saying which part is worse than none, because it looks like coverage.

## 2. Development environment

- **Language**: Rust, current stable edition.
- **Compilation**: the code must compile **with no `rustc` warnings**, and
  today it does. ⚠️ **§864: not entirely.** The canon pins the warnings
  that are left in view on purpose —the nine of `ceremony` (counted in
  §864), pinned debt in its row of `tools/canon.sh`, whose pin, 11, also
  counts cargo's two summary lines, and the `podar` one in the node's
  binary, with `dev` and without it (§580)—, and no new one passes in
  what the canon compiles, because every pin is exact. What no level
  compiles —the code under a feature that no row and no gate turns on—
  is declared by «Ceguera declarada» [declared blindness], at the foot of
  `tools/canon.sh`, with its instances.

### Formatting and linting

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features
```

⚠️ **`clippy -D warnings` is NOT required, and it is worth knowing why.**
Measured on 01-08-2026:

| | warnings |
|---|---|
| Whole workspace | **189** |
| `zk-ssl` + `stark-experiment` —the system— | **155** |
| Of those, «the loop variable is used to index» | **122** |
| Dead code in the system | **0** |

The 122 are a **false positive of the domain**: in a circuit the index
**is the datum** —position in the trace, constraint slot, column—, and
rewriting
`for i in 0..4 { result[C_KEY + i] = next[8 + i] - current[COL_KEY + i] }`
with iterators would lose exactly what has to remain readable.

**They are not silenced with a global `allow`**: they are declared here.
A global `allow` would make the remaining 33 invisible, and among them
there are warnings this project has seen point at real things twice.

⚠️ **The dead code clippy finds is all in `crates/ceremony`** —the setup
machinery of the backends that **were not chosen**—. It is material of
the comparison, not of the production path.

**For a contribution**: do not worsen the number in the files you touch.

## 3. Workflow

1. *Fork* and local clone.
2. Branch with a descriptive name: `feat/`, `fix/`, `crypto/` —circuits
   or primitives—, `docs/`.
3. Small commits with clear messages.

⚠️ **The message of a commit describes what that commit contains**, not
what you meant to do. This project has on record **three messages that
claimed what their commit did not do** (`AUDITORIA.md` §91), and the
history is as much a document of the project as the README.

⚠️ **If you use a generative AI assistant, say so in the PR**: which
model and for what. This repository declares its own use in
[`GENAI.md`](./GENAI.md), and the same rule of the previous paragraph
rules here: the message describes what the commit contains. What is
generated without measuring, without running and without someone
answering for it does not enter.

⚠️ **What a PR does not see: the seals and their entries** (§793). Here a
change enters as a *sello* (a seal): a commit that carries its entry, its
*asiento*, in [`AUDITORIA.md`](AUDITORIA.md), with its commit, its
counters, what the change does **not** close and the lessons it left
(`GENAI.md` §«Dónde está el registro» [where the record is]), and an
entry is not rewritten afterwards: it is corrected with another. How one
is written, how a living document is corrected, the conventions of
`BACKLOG.md`, the RFCs with their stages and the levels of the canon and
the benches are told by [`doc/METODO.md`](./doc/METODO_EN.md), with the
citation to the file that establishes each thing.

## 4. Tests and validation

```bash
cargo test --workspace --all-features
```

⚠️ **That command runs in DEBUG, and the layer is NOT measured there.**
Measured on 26-08-2026: `cargo test -p zk-ssl` in debug gives 188 pass,
93 fail and 9 ignored; in release, 307 pass, 0 fail and 3 ignored. Those
failures **are not a soundness defect**: they are the degree limit of
entry 41 and of §78, declared and decided -winterfell checks in debug
that the declared degree is realized, and there are constraints whose
degree depends on the witness-. But you will see red if you run the
command above as is. **The layer is measured with
`cargo test -p zk-ssl --release`**, which is what `tools/canon.sh` runs.

⚠️ **§814: re-measured on 15-10-2026, and the layer no longer gives red
in debug.** `cargo test --locked -p zk-ssl` in debug gives 443 pass, 0
fail and 45 ignored, on `0315b14` (§813), in 3.253,94 s with other work
on the machine (4 CPU, load from 9,2 to 18,6). In release, the layer's
row in `tools/canon.sh` pins today 565 pass and 9 ignored (⚠️ §913: it
said 564; ⚠️ §908: it said 563 and 8; ⚠️ §907: it said 562; ⚠️ §905: it
said 561; ⚠️ §900: it said 558 and 7; ⚠️ §899: it said 557; ⚠️ §898: it
said 552; ⚠️ §896: it said 547; ⚠️ §839: it said 481; the 482nd is the
census of the lengths, which also runs in debug, where they would be 444
pass: the 443 are of `0315b14`, and the whole layer has not been run
again there. ⚠️ §840: it said 482; the two more, the census of the
`native_leaf`, also run there:
they would be 446. ⚠️ §842: it said 484; the one more, the census of the
byte bands, too: they would be 447. ⚠️ §816: it said 485; the eleven
more, those of the ledger's lock, have not been run in debug. ⚠️ §817:
it said 496; the one more, the one of the snapshot in the clear,
neither. ⚠️ §819: it said 497; the one more, the one of the sweep's
refusal, neither. ⚠️ §820: it said 498; the three of the export,
neither. ⚠️ §821: it said 501; the two of the layer imported with a key,
neither. ⚠️ §828: it said 503; the twelve of the re-encryption and the
migration of another owner's ledger, neither. ⚠️ §711: it said 515; the
three more, those of the layer's network, neither. ⚠️ §715: it said 518;
the four more, those of the replay with the malleated proof, neither.
⚠️ §716: it said 522; the eight more, those of the replay of a delegated
issuance, neither. ⚠️ §717: it said 530; the one more, the one of the
ledger over the cap on `m`, neither. ⚠️ §718: it said 531; the fifteen
more, those of the canonicity of the store, the snapshot and the age
producer, neither, and one carries
`cfg_attr(debug_assertions, ignore = …)`. ⚠️ §723: it said 546; the one
more, the one of the window's leaf, neither: the 45 and the 38 that
follow are those of `0315b14`). Of the 45, 7 are measuring instruments,
ignored in every profile, and 38 carry
`cfg_attr(debug_assertions, ignore = …)`: in debug they do not run, and
run there with `--ignored` all 38 pass. There are no longer failures to
attribute to the degree limit, and with this measurement entry 41 of
`BACKLOG.md` is closed. Still standing: those 38 are run only by release,
and the layer is measured with `--release`. Whether the command above,
the whole workspace with `--all-features`, gives red today is not known:
it has not been re-measured. And the «307» of release was not of
26-08-2026: that day they were 287 (§366), and the seals raised it with
the pin up to §394 without touching the date. ⚠️ **§864: and the five
crates a third party compiles without the prover** —the kit,
`zk-ssl-verify`; its closure, `zk-ssl-hash`, `zk-ssl-air` and
`zk-ssl-medio`; and the witness's guardian, `zk-ssl-guardian`— are also
checked with the debug assertions on:
`cargo test -p <crate> --profile aserciones --locked`, the profile of the
root `Cargo.toml`, which inherits from release; the canon runs it in
`--sello` with the release pins of each one. The rest of the workspace is
not measured in debug, and «Ceguera declarada», at the foot of
`tools/canon.sh`, declares it.

⚠️ **§855: the canon, in a freshly made clone.** It is run from the root
with `bash tools/canon.sh --sello`; the levels and what each one covers,
in [`doc/METODO.md`](./doc/METODO_EN.md). Before, once and with network:

```bash
cargo fetch --locked
```

`tools/check_curvas.py`, one of the canon's tools, reads the closure of
the production binaries with `cargo tree --offline --target all`, which
asks for the source of every crate on all targets, and a `cargo build`
only downloads those of your machine (§843). Without that step, or with
cargo's cache empty, the canon warns at the start, with the remedy, and
`check_curvas` comes out ROJO in its block. It downloads nothing. It
looks for the binaries where cargo leaves them: `tools/target_dir.sh`
says so, which reads it from `cargo metadata` and follows
`CARGO_TARGET_DIR`, `CARGO_BUILD_TARGET_DIR` and the `build.target-dir`
of a `.cargo/config.toml` (⚠️ §824: with `build.target-dir`, or with any
other key of `[build]`, in the `.cargo/config.toml` or the `.cargo/config`
of the root or of a parent directory, or in the `config.toml` or the
`config` of `CARGO_HOME`, the «3 ter» of the canon comes out ROJO:
`tools/artefacto.sh` compiles in its own `target` and rejects the table
before compiling, because other keys of its change what is compiled;
`doc/KIT.md` §«Reproducir el kit, para quien no se fíe de la descarga»
[reproducing the kit, for whoever does not trust the download]).

And for the circuits:

```bash
python3 tools/check_constraint_layout.py
python3 tools/verificar_citas.py
```

It cross-checks the indices of `result[...]` **and** of `periodic[...]`
in the 27 circuits: collisions, overflows and dead slots. It has its own
self-test —`--autotest`— over cases that really failed.

### If you touch a circuit

- **Run in release and in debug.** They do not give the same: in debug
  winterfell validates on generation and **names the constraint and the
  step** (`AUDITORIA.md` §77.1).
- **Add negative soundness tests**: cases that **must** fail. A change in
  a circuit without a test that shows what stops being acceptable is
  incomplete.
- ⚠️ **The positive goes first and alone** (§66.2). With the positive
  broken, **all the negatives pass** — and they pass for the wrong
  reason.
- ⚠️ **A negative test that passes may be passing for the wrong
  reason.** Check that it fails for what you think: §77 records one that
  failed for the right error in one mode and for a different one in the
  other.

**No change is merged without passing the suite.**

## 5. Pull Requests

- **Description**: what problem it solves, the logic implemented and
  **how you validated it** —with numbers or tests if applicable—.
- **Changes in circuits, in the state transition or in the verifier**:
  require explicit review by the maintainer.
  ⚠️ **There is no formal specification of the AIR to check against**
  (`SECURITY.md` §3.1). While there is none, the PR must **state against
  which rules it claims to comply**, because the work is being done over
  the code and not over a contract — and that has to be said.
- **Claims of security or performance**: with their measurement, or
  labeled as conjecture.
- **Scope**: one PR, one conceptual change. Format or state changes that
  affect privacy or soundness go **separate and flagged**.

## 6. What fits and what does not

**Fits**: measurable improvements; **formal specification and soundness
tests of the circuits** —shortcoming no. 1—; closing the limits of
`SECURITY.md` through verifiable mechanisms; more rigor, not more
surface.

**Does not fit without prior discussion with numbers**: proof systems
that reintroduce trusted ceremonies or non-post-quantum assumptions
against the thesis of the core; a token, *staking* or validator markets
that presuppose a decentralization the project **does not have**;
stacking components —zkVM, identity, modular availability— that increase
the surface without solving a present and measured problem.

**Minimalism here is not aesthetics**: every piece added is surface that
someone will have to audit, and **there is no audit**.

---

*Keep this document consistent with the code. When they stop matching,
the code wins.*
