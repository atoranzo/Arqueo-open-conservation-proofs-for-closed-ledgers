# B13/B14 · Step 2, prerequisite — COMPLETE MAP of the geometry of `circuit_send`

> **Español:** [`doc/mapa-geometria-circuit_send.md`](./mapa-geometria-circuit_send.md) is the
> original of this page, in Spanish; it prevails wherever the two differ. This translation is of
> commit `7766efb` (audit entry §917) and was produced by the project's assistant, a Claude Code
> session, in audit entry §918 (`GENAI.md`): it is a generated artefact, not text written by the
> author. Line citations of the form `file:N` refer to the Spanish files.

It exists because of §139: three reversions showed that the geometry of this
circuit cannot be moved by manual shifting, and the entry orders this map
to be produced BEFORE deciding hack or refactor. Everything that follows
is **read in `circuit_send.rs` (HEAD `1cbfedc`) and verified
mechanically** with `verifica_geometria.py` (attached; candidate for `tools/`): the script
rebuilds the schedule row by row from the exact constants,
cross-checks the three representations against each other, and reproduces the two
failed attempts with their blow-up on the same line. Without `cargo` in this session
(no network): the verification is exact arithmetic, not compilation — every
number in this document comes from the script, not from memory.

The lines cited are those of the current tree. ⚠️ §139 cited «lines
477-489» for the `match`; today that block lives at 464-478 (the entry was
written over an intermediate state). This map pins those of `1cbfedc`.

## 0. The unit of measure: the cycle of 8

The whole circuit beats in cycles of `CYCLE_LENGTH = 8` rows: **7 Rescue
rounds** (`NUM_ROUNDS = 7`, rows with `pos = r % 8 < 7`) plus **one link
row** (`pos == 7`) where the state is emptied and re-seeded for the
next merge. The state has `STATE_WIDTH = 12`: capacity
`0..4`, left half of the rate = outgoing digest `4..8`, right half
`8..12`. Two lanes: A in columns `0..12`, B in `12..24` (`LANE_B`),
OLD and NEW state (spec of the leaf machine, §1).

Convention the whole map uses: **cycle `c` occupies rows
`8c..8c+7`; its link row is `8c+7`; the digest available on that
row is the output of the merge hashed during `c`; what the link
row seeds is the input of cycle `c+1`**. The `match r` of
`build_trace` decides the seeding; the arithmetic of the generic arm uses
`next_cycle = (r+1)/8 = c+1`.

## 1. The complete timeline (verified schedule, section A of the script)

| cycles | rows | what it hashes | link row and event | governed by |
|---|---|---|---|---|
| 0 | 0..7 | `H(id_cuenta, saldo)` ×2 lanes (old balance in A, new in B) | `7 = ROW_LEAF_LINK`: seeds `(digest, nonce)` | explicit arm |
| 1 | 8..15 | `H(interno, nonce)` = `native_leaf` | `15 = ROW_LEAF_DONE`: places the leaf at **level 0** of accounts | explicit arm |
| 2..33 | 16..271 | accounts climb, level `c−2` in cycle `c` | `8c+7` with `nc=c+1 ∈ (2..34)`: places level `nc−2` (effective: **levels 1..31 on rows 23..263**) | generic range |
| 33→34 | 271 | — | `271 = ROW_ROOT`: roots A/B **bound by assertion**; seeds `(SPEND_KEY_DOMAIN, clave)` | explicit arm |
| 34 | 272..279 | derivation of the identity (`272 = ROW_PK_START`, assertion of the domain) | `279 = ROW_PK_DONE`: **ownership** (`C_PK_CHECK`); places ZERO leaf at **level 0** frozen | explicit arm |
| 35..58 | 280..471 | frozen climb, level `c−35` in cycle `c` | `8c+7` with `nc ∈ (36..60)`: places level `nc−35` (effective: **levels 1..23 on rows 287..463**) | generic range |
| 58→59 | 471 | — | `471 = ROW_FROZEN_ROOT`: frozen root **bound by assertion**; seeds `(id_receptor, aleatorio)` | explicit arm |
| 59 | 472..479 | inner commitment `H(id_r, aleatorio)` | `479 = ROW_PEND_INNER`: seeds `(interno, importe)` | explicit arm |
| 60 | 480..487 | the pending one `H(interno, importe)` | `487 = ROW_PENDING_ENTRY`: A places ZERO, B the commitment, **level 0** of pendings | explicit arm |
| 61..92 | 488..743 | pendings climb, level `c−61` in cycle `c` | `8c+7` with `nc ∈ (62..94)`: places level `nc−61` (effective: **levels 1..31 on rows 495..735**) | generic range |
| — | 743 | — | `743 = ROW_PENDING_ROOT`: roots A/B **bound by assertion**. The loop `for r in 0..ROW_PENDING_ROOT` (line 399) **does not process this row**: no seeding | loop bound |
| 93..127 | 744..1023 | **NOTHING** — slack | no `hash_flag`, no ARK, lanes free | (§6) |

Verified: every level of every tree is placed **exactly once**
(31+1, 23+1, 31+1) and no link row is left without an arm. The
last cycle of each climb carries no periodic link: its output is the
root, which the assertions bind, not a transition constraint.

## 2. The THREE representations, site by site

### R1 — The `ROW_*` constants (rows). 9 constants, 6 families of consumers

Declared at 143-167: `ROW_LEAF_LINK=7`, `ROW_LEAF_DONE=15`,
`ROW_ROOT=271`, `ROW_PK_START=272`, `ROW_PK_DONE=279`,
`ROW_FROZEN_ROOT=471`, `ROW_PEND_INNER=479`, `ROW_PENDING_ENTRY=487`,
`ROW_PENDING_ROOT=743`. Their consumers — ALL of them, census §137:

1. **Arms of the `match r`** (410-463): seven of the nine
   (`ROW_PK_START` and `ROW_PENDING_ROOT` have no arm).
2. **Loop bound** (399): `for r in 0..ROW_PENDING_ROOT` — this
   constant is at once a landmark and the frontier of the WHOLE schedule.
3. **Point periodics** (661-762): `link_leaf[ROW_LEAF_LINK]`,
   `link_place[ROW_LEAF_DONE]`, selectors `[0, ROW_ROOT, ROW_PK_DONE]`,
   `frozen_entry[ROW_PK_DONE]`, `pend_in[ROW_FROZEN_ROOT]`,
   `pend_val[ROW_PEND_INNER]`, `pend_entry[ROW_PENDING_ENTRY]`; and the
   **extent** of `hash_flag` and the 24 ARK columns:
   `0..=ROW_PENDING_ROOT` (667, 677).
4. **Assertions** (1019-1089): rows `0`, `ROW_ROOT`, `ROW_PK_START`,
   `ROW_FROZEN_ROOT`, `ROW_PENDING_ROOT`.
5. **`get_pub_inputs`** (1116-1153): reads the trace at `ROW_ROOT`,
   `ROW_FROZEN_ROOT`, `ROW_PENDING_ROOT` and row 0.
6. **Landmark test** `trace_landmarks_match_native` (1706-1776): uses the
   constants, not numbers of its own — it moves with R1 alone.

`ROW_PK_START` lives only in assertions; `ROW_PENDING_ROOT` in the loop
bound, the extent of the periodics, assertions, `get_pub_inputs` and the
test — **never in an arm of the match**.

### R2 — The cycle offsets in columns. 3 literals (2 / 35 / 61), 6 loops

| literal | bit loops in `build_trace` | link periodics in the AIR |
|---|---|---|
| `2` (accounts) | `rows[(2+level)*8+p][COL_BIT]`, levels 0..31 (507-516) | `link_merkle[(2+level)*8+7]`, levels 0..30 (691-695) |
| `35` (frozen) | `rows[(35+level)*8+p][COL_FBIT]`, levels 0..23 (496-505) | `frozen_link[(35+level)*8+7]`, levels 0..22 (734-738) |
| `61` (pendings) | `rows[(61+level)*8+p][COL_PBIT]`, levels 0..31 (485-494) | `pend_link[(61+level)*8+7]`, levels 0..30 (756-760) |

Verified semantics (section C of the script): the bit of level `l` lives
during the cycle that HASHES level `l` — and that is why it covers the
`next` row of the transition that PLACES it (`C_PLACE` reads `next[COL_BIT]`).
The generic periodic links coincide row by row with the generic
placements of the schedule (31/23/31 ones).

⚠️ **The finest off-by-one of the file is here**, and it is the exact
specimen of the «distinct conventions» of §139: the SAME expression
`(2+level)*8` means two things depending on the loop. In the bit loops,
`level` is the level that cycle `2+level` **hashes** (that is why they go
0..31). In the link constructors, the one that is switched on at the end
of cycle `2+level` **places level `level+1`** (that is why they go 0..30: the
outgoing transition of the last cycle is the root row, governed by
R1 — `sel_root`/assertions — and must NOT carry a generic placement).
Same literals, frames shifted by one.

### R3 — The ranges + arithmetic of the `match` (464-478). 1 site, 3 conventions

```rust
let next_cycle = (r + 1) / CYCLE_LENGTH;
if (2..34).contains(&next_cycle)      { place(…,        next_cycle - 2)  }
else if (36..60).contains(&next_cycle){ place_frozen(…, next_cycle - 35) }
else if (62..94).contains(&next_cycle){ place_pending(…, next_cycle - 61)}
// si ningún brazo ni rango aplica: el estado queda A CERO (siembra vacía)
```

The arithmetic is structurally the same in the three stretches (`level =
next_cycle − start_of_stretch`), but the LITERALS of the ranges do not
encode the start uniformly: accounts starts the range at its
real start (2), frozen and pendings start it at `start+1` (36,
62) because their first value is shadowed — see §3. And the empty branch is
silent: an uncovered value does not blow up, it leaves the state at zero and the
proof dies downstream in `C_HASH` with an opaque diagnostic.

## 3. The frontier cycles, explained (the why of the jumps 34→36 and 60→62)

The ranges OVER-COVER and depend on EXTERNAL guards not to step on
illegal values — and the guards **are not of the same kind** at each
frontier. Complete table (section B of the script):

| `next_cycle` | row `8·nc−1` | what would happen in the range | who prevents it |
|---|---|---|---|
| 2 | 15 | accounts level 0 (duplicate) | **shadowed** by `ROW_LEAF_DONE` |
| 34 | 271 | — (34 excluded from the range) | the range itself; besides, arm `ROW_ROOT` |
| 35 | 279 | — (outside every range; empty seeding) | **shadowed** by `ROW_PK_DONE` |
| 59 | 471 | frozen **level 24 over a path of 24 → panic** | **only** the shadowing by `ROW_FROZEN_ROOT` |
| 60 | 479 | — (out of range; empty seeding) | shadowed by `ROW_PEND_INNER` |
| 61 | 487 | — (out of range; empty seeding) | shadowed by `ROW_PENDING_ENTRY` |
| 93 | 743 | pendings **level 32 over a path of 32 → panic** | **only** the loop bound (`0..ROW_PENDING_ROOT`) — `ROW_PENDING_ROOT` has no arm |
| 94 | 751 | — (94 excluded) | the range itself; and the loop bound |

Three distinct mechanisms protect the frontiers: (a) the bound of the
range itself, (b) the shadowing by an explicit arm, (c) the loop
bound. **Only the first travels with the range.** The other two live in
R1: that is why moving R1 without R3 detonates, and detonates precisely in frozen —
the only stretch whose upper illegal value (nc=59) is guarded
solely by the constant that the shift moves.

**Reconstruction of the attempts (section D of the script, matches
§139 in detail).** Attempt 1 (`ROW_*` +8, rest old) and attempt 2
(`ROW_*` +8 and column offsets +1, old ranges): in both, row
471 stops being `ROW_FROZEN_ROOT` (now 479), falls into the generic arm with
`nc=59 ∈ (36..60)`, and `place_frozen` receives `level 24` over a path of
24 → index out of range, **same line both times** (the closure
at 369-377). Attempt 2 moved R2, which takes no part in the `match`:
touching the bit loops and the periodics does not change where it blows up.

## 4. The extended census (R4): where ELSE the geometry lives

The grep of §139 (`const ROW_`, `* CYCLE_LENGTH`) missed the `match`.
This census looks for the FOUR forms — and what it finds besides:

- **They follow R1 (they move on their own with the constants):** assertions,
  `get_pub_inputs`, extent of `hash_flag`/ARK, loop bound,
  landmark test. None hides a literal of its own. ✔
- **Outside the file: NOTHING.** The `ROW_*` are private; `zk-ssl` only imports
  `build_trace`/`SendProver`/`SendAir`/`SendPublicInputs` (lib.rs:140, two_phase.rs:49 (⚠️ §895: it
  said `:53`), metrics.rs:249 (⚠️ §895: it said `:94`)). `doc/air/` documents burn and mint, not
  send. ✔
- ⚠️ **The comments, the informal representation (the fourth):**
  1. `TRACE_LENGTH` (96-101), **surely OUT OF DATE**: «the phases of the
     pending reach 1007 … 16 rows of margin remain». Reality:
     they reach **743**; the margin is **280 rows** (§6). It describes a
     previous geometry, internally coherent but dead.
  2. `ROW_PEND_INNER` (158-162), **ambiguous frame**: «Insertion of the
     pending: cycles 60..91, rows 480..735» counts the cycles whose
     link rows PLACE (60→row 487 the entry, 61..91→495..735
     levels 1..31); its neighbour at `ROW_PENDING_ROOT` («cycles
     61..92, rows 488..743») counts the cycles that HASH the climb.
     Both true, each in its frame — which is precisely the ambiguity
     this map exists to remove.
- ⚠️ **Minor correction to §139**: «rows 744→751, 281 free» — they are
  **280** (744..1023); 743 is not free, it holds the roots bound by assertion.
- **`FROZEN_DEPTH` is a SHARED coordinate** (the lesson of §137,
  in force here): it lives in `circuit_freeze.rs:61` and is imported by send,
  claim, burn, frozen_climb and settlement; the layer already has
  `FROZEN_DEPTH_POST = 32` in `migration.rs:29` (⚠️ §895: it said `:20`) (1b). Consequence for
  the pilot in §7.

## 5. Which constraint looks at each region (to know what breaks when moving what)

- `C_HASH_A/B` (804): active on every row with `hash_flag=1` — it binds the 12
  state columns; it is the one that drags the loose limbs (§138) and the
  one that dies, with an opaque error, if a seeding is left empty.
- Shared Merkle block (808-827): `tree_link = link_merkle +
  link_place` governs capacity, A/B placement and shared sibling of
  the ACCOUNTS climb — the placement of level 0 and that of levels
  1..31 go through the SAME gate.
- Frozen (928-940) and pendings (950-1001): gates of their own
  (`frozen_entry/frozen_link`, `pend_in/pend_val/pend_entry/pend_link`),
  same structure.
- Bit booleans (829, 940, 1001) and transport (877-909, 1002):
  active on the WHOLE trace, slack included — they are the only ones that
  constrain rows 744..1023 (and hold there trivially:
  broadcast columns constant, bits at zero).
- Assertions: they fix row 0 (capacities and high rate at zero, amount,
  limit, supplies), `ROW_ROOT`, `ROW_PK_START`, `ROW_FROZEN_ROOT`,
  `ROW_PENDING_ROOT`. Declared count: 42 (652).

## 6. The slack at the end

Rows **744..1023: 280 rows = exactly 35 cycles** with no hash pipeline
(`hash_flag` and ARK at zero from 744; lanes 0..24 with no active
constraint; only the globals of §5, which hold on their own). It is the space
the HACK of §139 wanted to use, and the cushion that makes the
refactor's shift viable.

## 7. Budget of the complete pilot (section E of the script)

Step 2 is not only the salt: it is **salt + frozen-32** (Addendum 9; spec
§2-§3). In rows:

| front | cost | `ROW_PENDING_ROOT` |
|---|---|---|
| today | — | 743 |
| + salt (one cycle after the nonce) | +8 | 751 |
| + frozen-32 (24→32 levels) | +64 | **815** |

Remaining margin: **208 rows** — `TRACE_LENGTH = 1024` is enough for
send; the per-circuit budget table of the spec (§3) receives here
its first row with data.

⚠️ **Scope question the census uncovers** (the author's decision, not
this map's): `FROZEN_DEPTH` is global to five circuits + settlement.
The pilot cannot raise it to 32 for send alone by touching the constant —
it would break the other four at once. Two ways to stage it: (i) the
pilot parametrizes send's frozen depth locally (e.g.
`SEND_FROZEN_DEPTH`, today `= FROZEN_DEPTH`, and the global flip arrives with
the atomic step 3); (ii) the pilot executes the complete salt + the MECHANICS of
frozen-32 prepared but without the flip, and frozen-32 is switched on in step 3
for the five at once (the deployment is already atomic by design of the
migration). Option (i) keeps the promise «complete pilot in one
circuit»; (ii) keeps «one constant, one truth» (§125). The map
leaves both on the table with the census in front.

## 8. The choice: HACK against REFACTOR, with the map in front

### The hack, truly costed

The idea of §139: the salt cycle in the slack (e.g. rows 744-751),
logical link backwards, zero shift. The map reveals what it
costs to execute, because the needed links **are not local** (the
constraints see a 2-row frame) and they have to be transported in
columns:

- Transport variant: capture the unsalted leaf from row 15 (A and B),
  seed it at 743→744 with the salt, bind the output of 751, and make
  the placement of row 15→16 place the transported value. **+20
  columns** (52→72: salt 4, unsalted leaf A/B 8, salted leaf A/B 8),
  ~40 new constraint slots.
- Recomputation variant: the tail re-derives the leaf from the broadcast
  columns (id, balance, nonce: 3 cycles), transports only the salted one.
  **+12 columns** (52→64), but leaves cycles 0-1 as machinery
  whose product nobody consumes any more — against §125.
- Both require **splitting `tree_link`** (the placement of level 0 stops
  being the generic one): touching the shared Merkle block, the most
  delicate point of the AIR; extending `hash_flag`/ARK and the loop to the tail; growing
  and reordering the tail of the degree vector; new landmarks and mutations.
- The reading cost — the trace stops being temporal — is paid in every
  future audit, **multiplied by the ten AIRs** of step 3.
- **And the argument that decides**: the hack dodges the shift of the salt
  (8 rows) but **not that of frozen-32 (64 rows)**, which moves
  `ROW_FROZEN_ROOT` — row of assertion and of seeding of the pending — and
  everything after. Putting frozen-32 in the tail as well would require another
  transport (+4 columns), moving the root constant all the same, and
  an already baroque topology. **The shift has to be done in any
  case; the hack only defers 8 of its 72 rows and charges columns and
  legibility for the deferral.**

### The refactor, with the derived form already shown

Section F of the script shows that the NINE row constants collapse
into **eight derived cycle starts** from `(CYCLE_LENGTH, TREE_DEPTH,
FROZEN_DEPTH)`:

```rust
const CYC_NONCE: usize      = 1;
const CYC_ACC: usize        = 2;                          // → 3 con salt
const CYC_PK: usize         = CYC_ACC + TREE_DEPTH;       // 34
const CYC_FROZEN: usize     = CYC_PK + 1;                 // 35
const CYC_PEND_IN: usize    = CYC_FROZEN + FROZEN_DEPTH;  // 59
const CYC_PEND_VAL: usize   = CYC_PEND_IN + 1;            // 60
const CYC_PEND_CLIMB: usize = CYC_PEND_VAL + 1;           // 61
const CYC_FIN: usize        = CYC_PEND_CLIMB + TREE_DEPTH;// 93
// ROW_X = CYC_X * CYCLE_LENGTH − 1   (ROW_PK_START = CYC_PK * 8)
```

Verified: the nine derived ones coincide with today's nine literals.
With them, R2 is the same three names (`CYC_ACC`, `CYC_FROZEN`,
`CYC_PEND_CLIMB`) and R3 **normalizes to ONE convention**: range =
`(CYC_START..CYC_END_OF_STRETCH)`, level = `nc − CYC_START`, with the
entry shadowed by its arm and the exit excluded by the range:

- accounts `(CYC_ACC..CYC_PK)` = (2..34) — identical to today;
- frozen `(CYC_FROZEN..CYC_PEND_IN)` = (35..59) — today (36..60); the
  values that change, 35 and 59, are **unreachable** today (shadowed,
  §3): byte-identical trace;
- pendings `(CYC_PEND_CLIMB..CYC_FIN)` = (61..93) — today (62..94); 61 and
  93 unreachable (shadowing / loop bound): identical.

After this, inserting the salt is **adding `CYC_SALT = 2` and shifting
`CYC_ACC` to `CYC_SALT + 1`**: everything else — rows, ranges, loops,
periodics, assertions, landmarks — re-derives itself. And frozen-32 is the
change of the depth (with the staging of §7). The
frontier cycles stop having three distinct guards: the range
guards itself.

**Plan in compilable steps (rule §129 — nothing in one go), each
with a green suite and an atomic md5 guard:**

1. **SB0.1** — introduce the `CYC_*` block and redefine the `ROW_*` as
   derived, with `const _: () = assert!(ROW_ROOT == 271);` (and the eight
   sisters) as transitional pins. Compiles, suite 286/0, landmarks.
2. **SB0.2** — replace the literals 2/35/61 of the six loops (three
   of bits, three periodics) with `CYC_ACC`/`CYC_FROZEN`/`CYC_PEND_CLIMB`.
   Green suite.
3. **SB0.3** — normalize the three ranges of the `match` to the single
   convention and add `debug_assert!(level < profundidad)` in the three
   `place_*`, so that the next mismatch speaks clearly instead of
   blowing up by index. Equivalence: only unreachable values change
   (§3). Green suite + landmarks + `no_constraint_is_vacuous`.
4. **SB0.4** — correct the dead comment on the margin, unify the
   frame of the two comments on the pending (§4), remove the transitional
   pins, and leave ONE sentence of convention next to the block:
   «every start of a stretch is a `CYC_*`; no cycle literal outside
   this block». Green suite. Entry in AUDITORIA.

Afterwards, **SB1 (salt)** is again what the spec designed — the new
cycle, `link_salt`, the six `C_SALT_*` of the spec (24 slots, four limbs
bound by §138), +4 witness columns — but its shifting mechanics
is one line. SB2..SB5 of the pilot's plan remain valid.

### Recommendation

**REFACTOR (SB0), and then the salt.** It is not the pretty option against the
quick one: it is that the quick one does not exist — frozen-32 forces the shift
the hack meant to avoid, so the hack would pay its 12-20 columns, its
split gate and its illegible trace **on top of** the shift, not instead
of it. The refactor is pure re-expression (byte-identical, shown
arithmetically and verifiable with the suite at each step), leaves ONE
source of truth where there were three-with-guards-of-three-kinds, and is the
pattern the other nine AIRs are going to need just the same in step 3. The
fourth reversion §139 feared is avoided not by choosing better between two
ways of moving three representations, but by ceasing to have three.

## 9. What this map does NOT cover

The geometry of the other nine AIRs (each one is censused when migrated —
burn shares landmarks 7/15/271/279/471 according to `doc/air/circuit_burn.md`,
but «it is not assumed that what holds for send holds for burn without proving it there»,
§138); the staging decision on `FROZEN_DEPTH` (§7, the author's);
and the execution itself of SB0, which belongs to the pilot's session, on the
real tree, with its full rite.
