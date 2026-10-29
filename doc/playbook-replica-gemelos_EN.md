# B13/B14 · Step 3 — PLAYBOOK of the replication (the nine twins)

> **Español:** [`doc/playbook-replica-gemelos.md`](./playbook-replica-gemelos.md) is the original of
> this page, in Spanish; it prevails wherever the two differ. This translation is of commit
> `7766efb` (audit entry §917) and was produced by the project's assistant, a Claude Code session,
> in audit entry §918 (`GENAI.md`): it is a generated artefact, not text written by the author. Line
> citations of the form `file:N` refer to the Spanish files.

Distilled from the send pilot (§140-§143, commits `0451462`→`ac6cc30`).
One session per circuit, full rite. **Recommended order**: claim
(three stretches, sibling of two_phase: maximum transfer of the
pattern) → burn (accounts+frozen) → mint, audit, mint_climb, recovery,
recovery_climb (one stretch) → freeze, frozen_climb (depth only,
no salt: minimal twin).

## Method optimization (learned in send, for the nine)

**Twin-first; SB0 INSIDE the twin.** Send did SB0 on the legacy
because it preceded decision C (§142). The nine do not: the copy is
born, and the unification of representations is done in it already.
The legacy keeps its literals — irrelevant: it is deleted in the flip.

## R0 — The prior census (half an hour that buys the rest)

1. Read the WHOLE leaf region of the circuit (spec §5: the variants
   exist — mint has no subtraction lane, audit does not mutate state,
   recovery COPIES the salt, seam 52).
2. Grep for the FOUR forms (§139/§140): `const ROW_`, the offsets
   `(N + level)`, the ranges of the `match` (§141 already censused
   which ones each has), and the COMMENTS with numbers.
3. Internal callers of `build_trace` (in send: one nexus `scenario()`
   + N sites in two moulds — look for the equivalent).
4. Frozen couplings: does it use shared `frozen_climb`/`frozen_leaf`?
   ⚠️ `frozen_climb` PINS 24 inside (§143.1) — the twin with
   frozen-32 needs its own local `frozen_climb_32` (withdrawal clause).
5. Name collisions: send already had `COL_SALT` (the pending's random
   value) — the new witness is ALWAYS `COL_LEAF_SALT`.
6. Accidental doctests? (``` fences in //! must be `text`).

## R1 — Birth (= SB1.a of send)

Compiling copy + scaffold header with a **withdrawal clause** +
`pub mod <circuito>_salted;` in `lib.rs` (alphabetical order). The
slot guardian absorbs it on its own (`listdir`). Verify: stark suite
+N tests of the twin, layer intact.

## R2 — Internal SB0 (= SB0.1-0.4, in the twin)

`CYC_*` block derived from `(CYCLE_LENGTH, TREE_DEPTH[,
FROZEN_DEPTH])`; `ROW_*` derived with transitory pins; loop literals
to names; `match` ranges to the single convention
`(CYC_arranque..CYC_fin)` + `debug_assert!` in the `place_*`; pins
out and **budget guard** `ROW_FINAL < TRACE_LENGTH`;
comments re-anchored to names. Each sub-step: green suite.

## R3 — The wrapped world (= SB1.b)

`CYC_SALT` in the chain (everything shifts on its own); `COL_LEAF_SALT`
(+4, width +4); arm of the third merge (digest carried along, FOUR
limbs of the salt into the rate); parameter `leaf_salt: Digest` +
propagation; the test nexus to the salted native —
`derive_leaf_salt_wide(key)` + `native_leaf_salted` (home:
`circuit_settlement.rs`), NEVER a toy literal. ⚠️ recovery: the salt is
COPIED (§93.4/seam 52) — its scenario reflects a copy, not a fresh
derivation from the new state.

## R4 — The six (= SB1.c)

`C_SALT_{CAP,DIG,IN}_{A,B}` (24 slots) BETWEEN the last block and the
constant the house left at the end; `P_LINK_SALT` next to
`P_LINK_LEAF` (same order in `get_periodic_column_values`); degrees:
24 × `with_cycles(1, full)` at the index position; evaluate with the
mould of the leaf links. Strong verification for free: the slot
guardian (static) + `no_constraint_is_vacuous` (dynamic) cover
the 24.

## R5 — Mutations + mirror (= SB1.d, non-negotiable, spec §4)

(a) altered witness salt limb (poison = **honest + 1**) →
`C_SALT_IN` rejects; (b) unwrapped leaf into the placement transition
(BOTH halves of the next state, bit-agnostic) →
`C_PLACE` rejects; + native↔circuit limb by limb (unwrapped in
`ROW_SALT_LINK`, wrapped in `ROW_LEAF_DONE`, both lanes). The
harness: catch_unwind + prove + verify, `!verifica`.

## R6 — Local frozen-32 (= SB1.e; only claim, burn, freeze, frozen_climb)

Remove the shared import; local `const FROZEN_DEPTH: usize = 32;`
with the flip-in-one-line note; `frozen_climb_32` in tests if the
circuit climbed with the shared one. In freeze/frozen_climb (no salt):
R3-R5 do not apply; their twin is R1+R2+R6 + their own depth
mutations (a path of 24 declared as 32 must reject).

## R7 — Seal per circuit

Row of table §3 of the spec (today / +salt / +frozen-32 / slack /
verdict — **it is not assumed that 1024 is enough**); entry in AUDITORIA
with the chain of md5 and hashes; BACKLOG 50 up to date; guardians 34·28+.

## Living rules that have already bitten

- Comment numbers are reviewed IN the step that moves the
  geometry (§143.2 — it bit the assistant in SB1.b).
- Canonical suite ALWAYS `--release` (§140, instrument note).
- Atomic multi-file guards: EVERYTHING verified before writing
  ANYTHING; anchors with an explicit expected count when they are global.
- Blocks with the `=== fin ===` sentinel; heredocs close on their own.

## Preview of the flip (D4, for when the ten exist)

ONE release: the layer switches imports to the twins; legacies,
`_32` helpers and local consts are deleted; shared `FROZEN_DEPTH` to 32
(or `frozen_climb` moves to iterating the path — that step's decision,
§143.1); a legacy fixture tests the migration; snapshot v7.

## Epilogue — the campaign, closed (post-flip)

The playbook delivered: TEN twins (§143-§152), T2b-circuit (§154),
paired measurement (§155), FLIP D4 as a single release (§156) and F3
(§157) — **zero reds** in both suites for the first time. This
document remains as METHOD for future replications of this size.

Rules won under fire, added to those of the campaign: **the canon is
the pilot's tree** (the mirror's md5s are informative; what judges is
the assertions of each transformation, the guardians and the suites);
**entries are born in the python** (paragraphs joined by `\n\n`,
immune to the clipboard); **a commit message does not assert a future
green** (it is written after the green or it is written conditionally);
**«done» is detected by structure**, not by byte-equality; and **tails
and counts are censused from the real thing, to the end** — burn's
second call waited for the flip to collect its noted doubt.
