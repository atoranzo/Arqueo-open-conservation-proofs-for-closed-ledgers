# B13/B14 · Step 1.5 — Spec of the LEAF MACHINE (read before touching)

> **Español:** [`doc/spec-maquina-de-hoja.md`](./spec-maquina-de-hoja.md) is the original of this
> page, in Spanish; it prevails wherever the two differ. This translation is of commit `7766efb`
> (audit entry §917) and was produced by the project's assistant, a Claude Code session, in audit
> entry §918 (`GENAI.md`): it is a generated artefact, not text written by the author. Line
> citations of the form `file:N` refer to the Spanish files.

It exists because of §72: a written spec catches the lane bug by reading it, and the ten AIRs are
ten opportunities for that bug. Reference: `circuit_send` (the pilot). Anatomy verified in code, not
from memory.

## 1. The machine AS IT IS (send)

**Geometry**: `TRACE_LENGTH = 1024`, `TRACE_WIDTH = 52`. Two Rescue lanes of `STATE_WIDTH = 12`
(capacity 0..4, digest 4..8, rate 8..12): lane A in columns `0..12`, lane B in `LANE_B = 12..24`.

**The lanes are NOT redundancy: they are old and new state.** A computes the OLD leaf and proves its
membership; B computes the NEW one and builds the subsequent path. `C_SIBLING` binds both to the
SAME path (same siblings, same bit); `C_PLACE_B` places the new digest in B's climb.

**The chain mirrors `native_leaf` exactly** — two merges:
`merge(merge(public_id, [bal,0,0,0]), [nonce,0,0,0])`.
- `first_row`: `C_INPUT` injects into the digest of each lane the `COL_ACC_ID` (the same identity in
  A and B) and into the rate `[8]` the balance — **`COL_BAL` in A, `COL_BAL_NEW` in B**: here is
  born the difference between lanes that `C_BALANCE` audits.
- `link_leaf` (periodic column, 1 at `ROW_LEAF_LINK`): stitches the second merge.
  `C_LEAF_CAP_{A,B}`: capacity of `next` to zero. `C_LEAF_DIG_{A,B}`: the digest is carried over
  (`next[4+i] == current[4+i]` — the output of the first merge is the left input of the second).
  `C_NONCE` (×2): the rate `[8]` of `next` := `COL_NONCE`, **in both lanes** (§92.2: binding only
  one lane leaves the other free).
- `C_KEY_INPUT` with `sel_root`: the FOUR limbs of the key, both lanes (§92.2 again, already caught
  once).

**⚠️ Question to verify in step 2** (honesty of this spec): the limbs `9..11` of the rate in the
nonce merge (`as_digest(nonce)` = `[nonce,0,0,0]`) — locate WHICH constraint binds them to zero. If
none, it is a finding prior to the salt and it is closed first.

## 2. SALT delta (the wrapping: one more merge)

`native_leaf_salted = merge(native_leaf, leaf_salt)` → **a third merge cycle per lane**, after the
nonce one:
- **+4 witness columns** `COL_SALT..+3` (52 → 56). ONE single salt shared by both lanes: the salt
  does NOT change in send/claim/burn/mint/transfer (it is preserved, 1a). In recovery the record
  COPIES the old salt (seam 52) → also a single salt there. No circuit of today needs two salts.
  (The day entry 52 rotates the salt, that circuit will need `COL_SALT_NEW` — out of scope, noted.)
- **+1 periodic selector** `link_salt`, 1 at `ROW_SALT_LINK =
  ROW_LEAF_LINK + CYCLE`. All the later `ROW_*` are shifted `+CYCLE` (8 rows). Cost: +8 rows per
  leaf circuit.
- **New constraints** (×2 lanes, all six): `C_SALT_CAP_{A,B}`: capacity to zero. `C_SALT_DIG_{A,B}`:
  digest carried over. `C_SALT_IN_{A,B}`: **the FOUR limbs** of the rate := `COL_SALT..+3` — the
  salt is a full digest, not a scalar; binding only `[8]` would be the bug of §92.2 in its new form.
- **Uniformity** (D-arquitectura [the architecture decision]): legacy accounts → witness salt =
  zeros, SAME machine. No conditional branch in the AIR.
- The input of the Merkle path becomes the output of the THIRD merge. The leaf→path link is shifted
  with the `ROW_*`.

## 3. FROZEN-32 delta (§137)

The freeze paths grow from 24 to 32 levels: **+8 cycles = +64 rows** in each circuit that verifies a
frozen path (send, claim, burn, freeze, frozen_climb). Budget per circuit: TABLE TO BE FILLED IN at
steps 2-3 by measuring the rows used today — **it is not assumed that 1024 is enough**.

| circuit | today (final row) | + salt | + frozen-32 | slack in 1024 | verdict |
|---|---|---|---|---|---|
| send (PILOT, §143) | 743 | 751 | **815** | **208** (26 cycles) | **FITS** |
| claim (§144) | 743 | 751 | **815** | **208** (26 cycles) | **FITS** |
| burn (§145) | 471 | 479 | **543** | **480** (60 cycles, in ITS 1024) | **DOES NOT FIT in 512 → own TRACE 1024** |
| mint (§146) | 311 | 319 | — (no frozen) | **192** (24 cycles, in its 512) | **FITS** |
| audit (§147) | 279 | 287 | — (no frozen) | **224** (28 cycles, in its 512) | **FITS** |
| mint_climb (§148) | 271 | 279 | — (no frozen) | **232** (29 cycles, in its 512) | **FITS** |
| recovery (§149) | 311 | 319 | — (no frozen) | **192** (24 cycles, in its 512) | **FITS** |
| recovery_climb (§150) | 271 | 279 | — (no frozen) | **232** (29 cycles, in its 512) | **FITS** |
| freeze (§151) | — (no salt) | — | **295** (at 32) | **216** (27 cycles, in its 512) | **FITS** |
| frozen_climb (§152) | — (no salt) | — | **255** (at 32, EXACT) | **0** (perfect fit, in its 256) | **FITS — exact** |

(First row: measured by the pilot in the twin; the rest is filled in circuit by circuit in step 3.)

If a circuit overflows: its own `TRACE_LENGTH` at 2048 (cost ~2× in its proof, measured in step 5)
or compaction of phases — decision PER circuit, with the data in front.

## 4. The MUTATION obligation (per circuit, non-negotiable)

Each migrated AIR brings at least TWO sabotages that must be REJECTED:
(a) a limb of the witness salt altered → `C_SALT_IN` fires;
(b) the third merge omitted (unwrapped leaf enters the path) → the
    link shift fires.
If a sabotage does NOT break the proof, the constraint is decorative (§72).

## 5. Checklist per circuit (steps 2-3)

| circuit         | leaf+salt | frozen-32 | note |
|-----------------|-----------|-----------|------|
| send (PILOT)    | yes       | yes       | reference of this spec |
| claim           | yes       | yes       | |
| burn            | yes       | yes       | |
| mint            | yes       | —         | |
| audit           | yes       | —         | |
| recovery        | yes       | —         | single salt (copy, seam 52) |
| recovery_climb  | yes       | —         | |
| mint_climb      | yes       | —         | |
| freeze          | —         | yes       | depth only |
| frozen_climb    | —         | yes       | depth only |

Order: complete send pilot (salt + frozen-32 + mutations + its native↔circuit test), then replicate
one by one. Each circuit: read its leaf region BEFORE (the variants exist: mint has no subtraction
lane, audit does not mutate state), apply, mutate, whole suite.
