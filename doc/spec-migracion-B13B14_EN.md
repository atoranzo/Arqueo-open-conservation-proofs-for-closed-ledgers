# B13/B14 · Step 1 — Spec of the MIGRATION EVENT (four sub-fronts, deterministic replay)

> **Español:** [`doc/spec-migracion-B13B14.md`](./spec-migracion-B13B14.md) is the original of this
> page, in Spanish; it prevails wherever the two differ. This translation is of commit `7766efb`
> (audit entry §917) and was produced by the project's assistant, a Claude Code session, in audit
> entry §918 (`GENAI.md`): it is a generated artefact, not text written by the author. Line
> citations of the form `file:N` refer to the Spanish files.

> **STATUS: EXECUTED** — flip D4 (`2ac37c5`) + F3 (§156-§157); tag
> `entrada-50`. The spec remains as the reference design of the event.

Deliverable of step 1 of the campaign. Full census executed (rule §137:
ALL the structures, not the sample). Corrections to the statement: **TEN
AIRs, not eight** (FROZEN_DEPTH lives in burn, claim, freeze,
frozen_climb, send + settlement-helpers; union with the 8 leaf ones =
10); and a **fourth sub-front**: sled's `acct:{índice}` keyspace is
rewritten. The IBAN→index map is NOT persisted (memory): the operator's
responsibility to re-register, outside the state.

## Design decisions, with verdict and precedent

**D1 — The leaf salt IS STORED IN THE RECORD.** The layer recomputes
leaves on every apply (25 call-sites of `native_leaf` in 11 files);
without the salt it cannot, and it does not have the key (§93.4). Storing
it is consistent with the threat model: entry 50 protects from THIRD
PARTIES (dictionary over siblings), not from the operator, who already
sees balances. The client derives it from its key (deterministic) to
prove; the layer reads it from the record to apply.
→ `AccountRecord` gains `leaf_salt: Digest`. Playbook 49-A P2 re-run:
field + 13 constructors (operating preserves; opening populates; loading
from the format) + `record_to_bytes_v3` (112 B) + **snapshot v6**
(MAGIC_V6; v5/v4/v3 importable, zero-salt sentinel). Regression
analogous to `operar_preserva_el_view_id`.

**D2 — The migration is recorded WITHOUT a ZK proof, WITH deterministic
replay.** Exact precedent: `open_account`, «the only transition without
a proof», zero digest visible in the log. A circuit that proves N
re-hashes would be a work in itself; the system's honesty already admits
recorded-not-proven transitions when they are deterministic and
auditable. The entry commits to: (old_accounts_root, new,
old_frozen_root, new, n_accounts, procedure_version). **Verification =
replay**: whoever has the pre-migration records recomputes both new
roots and compares.

**D3 — `OpKind::Migration` at the END of the enum** (append-safe for the
persisted log; census 4).

**D4 — ATOMIC deployment with a legacy fixture for the test.** The new
AIRs verify the new world (wrapped leaf, frozen-32) and not the old one:
circuits + layer + migration go in ONE release. Fresh test layers are
born post-world (real salt from opening, frozen-32, derived position).
The migration test builds the old world via a TEST-ONLY legacy
constructor (sequential, no salt, frozen-24) — the pattern of the
hand-made v3 snapshots that already exists.

## The procedure, deterministic and numbered (the replay IS the spec)

Input: pre-migration records {old_index → (public_id, balance, nonce,
view_id)}, pre frozen marks {old_index}, next_index quota.
1. Sort the records by **ascending old index** (the order fixes the
   probing: exact replay).
2. For each record, in that order: `pos = public_id[0] mod 2^32`; linear
   probing `pos = (pos+1) mod 2^32` while occupied. Record
   `índice_viejo → pos`.
3. New leaf at `pos`: `native_leaf_salted(id, balance, nonce,
   r.leaf_salt)` — **the salt OF THE RECORD** [AMENDMENT 1b]: zero for
   what was loaded from legacy formats, REAL for what was opened
   post-1a. The original text said zero-salt-for-everything because it
   was written before 1a; 1a made the record the source of truth. Pinned
   in a test (`cada_cuenta_sobrevive_con_su_hoja_envuelta`, commit
   dcc2af0).
4. New frozen at **depth 32**: for each old mark at `índice_viejo`, FROZ
   mark at `mapa[índice_viejo]`.
5. Sled: for each account, delete `acct:{viejo}`, insert `acct:{nuevo}`
   with `record_to_bytes_v3`. (`pend:` and `meta:` intact.)
6. Log: `OpKind::Migration`, zero proof digest, the commitments of D2.
7. `next_index` is kept as the census of openings (quota); it stops being
   a position for ever.
Post: new openings = derived position + probing + real salt derived at
opening (the narrow one derives it from the sk; `open_with_id` receives
it as a PARAMETER, like view_id — same wall §90/§93.4, same solution).

## Implementation sequence (each step compiles and passes before the next)

1a. **Formats** (playbook 49-A): `leaf_salt` in the record + 13
    constructors + `_v3`/112 B + snapshot v6 + `derive_leaf_salt_wide`
    already exists (§117). Tests: roundtrip, dual-load v5→sentinel,
    operating preserves.
1b. **The event**: `migrate_to_salted_positions()` per the procedure
    + legacy fixture + replay test (two runs = same roots)
    + test that frozen survives the remap.
1.5 **Spec of the leaf machine** (item 7 brought forward, §134-real): the
    C_LEAF_* constraints of the two lanes WRITTEN before touching them
    — §72 showed that a spec catches the lane bug by being read.
2.  **Pilot `circuit_send`**: wrapped leaf (one more merge, ROW_SALT_*)
    + frozen-32 (its path grows 8 levels) + MUTATION TEST (sabotage the
    salt → the AIR rejects, or the constraint is decorative).
3.  Replicate to the NINE remaining, one by one, each with its mutation.
4.  T2b-circuit (§126): end-to-end acceptance.
5.  Measure with the instrument of §130 (paired if it compares) and
    correct the cost label of entry 50.

## What this spec honestly leaves open
The trace cost of frozen-32 in each circuit (do +64 rows fit in 1024? it
is verified PER CIRCUIT in steps 2-3, not assumed); and whether the
atomic release demands a «pre/post» start-up flag or the deployment order
on a single node suffices (the operator's decision, to be documented in
the runbook).

## Implementation amendments (step 1b, commits dcc2af0/f1c08f2)
E1: step 3 of the procedure — the RECORD's salt, not forced zero (see
above). E2: the sled sub-front gained the `meta:migrated` marker (not
sealed: its semantics is presence and the stored roots arbitrate) and
the geometry-aware LOAD (wrapped leaf + frozen-32 with the marker;
legacy without it). E3: the idempotence guardian has two legs (log in
memory OR marker on disk): without the second, reopening and
re-migrating would re-wrap the leaves. E4: open question partially
resolved — a snapshot exported post-migration does NOT re-import until
the flip (v7).
