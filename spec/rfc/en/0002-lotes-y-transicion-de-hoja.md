# RFC-0002 — Leaf transition, batches, and the log hash

> **Español:**
> [`spec/rfc/0002-lotes-y-transicion-de-hoja.md`](../0002-lotes-y-transicion-de-hoja.md) is the
> original of this page, in Spanish; it prevails wherever the two differ. This translation is of
> commit `7766efb` (audit entry §917) and was produced by the project's assistant, a Claude Code
> session, in audit entry §918 (`GENAI.md`): it is a generated artefact, not text written by the
> author. Line citations of the form `file:N` refer to the Spanish files.

- **Status:** ACCEPTED — **the three stages are done** (§207, §209,
  §215). It is preserved as a record of what was decided, what was
  measured and what was discarded.
- **Authors:** (mesa) [the table]
- **Date:** 2026-08-07
- **Affected protocol version:** `zkssl/0.1` → `zkssl/0.2`
- **AUDITORIA entries:** §204 (the measurements), §205 (the
  corrections of `doc/ESCALADO`), §206 (this document), §207 (stage 3),
  §209 (stage 1)

## Status of the stages

| stage | status | where |
|---|---|---|
| **1 · the log hash** | ✅ **DONE** — `apply` from 33.27 to ~3.2 ms; node ceiling from 30 to **265-320 op/s** (band measured in §217: `apply` is flat between 2 and 10,000 accounts, and the spread between measurements exceeds the scale effect). It went up to `zkssl/0.2` and new vectors were emitted, keeping those of `0.1` | §209 |
| **2 · batches** | ✅ **CLOSED (§215).** Redefined without circuits in §210; reservation §211; `root_with` §212; the declaration of the log §213; validate/commit §214; `apply_many` §215 | §210-§215 |
| **3 · incremental tree** | ✅ **DONE** — `send_materials` from e = 1.08 to ~0.1. It touched neither the wire nor the circuits | §207 |

⚠️ **Correction to the «Compatibility» section further down**: this RFC
said that stages 1 and 2 had to share a single emission of
`zkssl/0.2`. **That is not how it was done**: stage 1 is sealed alone
and emits `0.2` now. Stage 2 will break the wire again and will need
**`zkssl/0.3`**. The reason for the change of plan is that stage 1 was
a change of one function with a measured ×10, and holding it back for
months waiting for stage 2 —which introduces new circuits and their FV
ladder— would have been paying a real cost for an elegance of
numbering.

> ⚠️ **This correction aged in its turn (§344).** It was written on
> sealing stage 1 alone (§209); its forecast about stage 2 —«it will
> break the wire again and will need `zkssl/0.3`», «it introduces new
> circuits and their FV ladder»— expired in §210, which redefined it
> **without circuits and without breaking the wire** (CLOSED in §215).
> The table above and the «Compatibility» section were updated; the
> previous paragraph was not, and it is preserved untouched: the error
> is recorded, not erased. `zkssl/0.3` was never this RFC's — it is
> emitted by **RFC-0003** (§343).

> **Numbering note.** **RFC-0001** stays reserved for the hardening of
> the keystore KDF (SHA-256 → Argon2id), flagged in
> `crates/zk-ssl/src/crypto.rs` and in `zk-ssl-sdk/src/keystore.rs`. **It
> is not drafted.** This RFC takes 0002 because `AUDITORIA.md` §204 and
> `doc/DIAGNOSTICO_ESCALADO.md` already reference it by that number.

> **This document was rewritten AFTER measuring.** Its first version
> proposed five conjectured stages; three of its premises fell when put
> to the test (§204). What follows is the plan the measurements
> dictate, with what was discarded written at the end —not erased— so
> that nobody proposes it again without reading why it fell.

---

## Motivation

Everything below is **measured**, not estimated. The benches live in
`crates/zk-ssl/examples/` and run with `--features sandbox`; the full
tables, in `AUDITORIA.md` §204.

| | measured |
|---|---|
| `apply` per operation | **37.99 ms** (disk) → ceiling **26.3 op/s** |
| … of which persistence | **3 %** |
| … STARK verification | **7 %** |
| … **`digest_of_proof`** | **93 %** — 30.99 ms, **2,915×** what Blake3 costs over the same bytes |
| Throughput with 4 threads | **1.57 payments/s** · **3.83 regenerations per payment** · **0.84×** relative to 1 thread |
| Wasted cryptographic work | **66 %** (70 generations for 24 operations) |
| `send_materials` against the number of accounts | **e = 1.08** (linear); at 1,000 accounts, ~248 ms per call |

Two facts order the whole plan:

1. **The node's ceiling is set by a hash that no circuit uses.**
   `proof_digest` and `chain_digest` do not appear in `stark-experiment`.
2. **The contention exists and is a livelock**: adding threads **worsens**
   throughput. And with a single thread it gives 1.85 payments/s without
   a single regeneration — the serial sum. **The system is serial AND ON
   TOP OF THAT degrades when parallelised.**

---

## Design

### Stage 1 — `digest_of_proof` stops using an algebraic hash

**The cheapest and the one with the greatest effect. It touches no
circuit.**

`log::digest_of_proof` walks the proof in blocks of 16 bytes and applies
`native_merge` (Rescue) to each one: **4,115 permutations** for a send
proof of 65,840 bytes.

**Change:** `digest_of_proof` moves to a non-algebraic hash —Blake3,
already present via `winterfell`—.

**What does NOT change:** `chain_digest` (5 merges, ~65 µs) **stays on
Rescue**. It could enter a circuit the day the attested heads are
implemented (§121); `digest_of_proof` never will, because it summarises
opaque bytes.

**Effect, by direct extrapolation of the measurement:** `apply` from
33.27 → **2.29 ms**; node ceiling from **30 → 436 operations/s**.

⚠️ **What this does NOT fix:** the contention. With 66 % of the work
still wasted, the effective throughput rises only **+13 %**
(1.81 → 2.03 payments/s), because on the serialised path generation
rules. **This stage raises the ceiling; it does not make it reachable.**

**Compatibility:** it changes `proof_digest`, and with it the chain and
the epoch head → **it changes the conformance vectors**. It is
`zkssl/0.2`, and the vectors of `0.1` are preserved (rule 2 of the
PROCESO).

### Stage 2 — Batches **without circuits** (redefined in §210)

> **This section was rewritten after measuring.** The previous version
> proposed changing `circuit_send`/`circuit_claim` so that they would
> stop asserting `root_new`, and adding a `circuit_batch_root` that the
> node would prove. **Both pieces fall**: the batch circuit because of
> its measured cost, and the change of circuits because it turned out
> unnecessary. What was discarded is kept written below, with its
> number.

#### 2.1 Why the batch circuit falls: it is measured

Bench `etapa_b0_lote` (§210). Two real geometries, fitted line
`t = 42,3 ms + 3,2 µs por cada 1.000 celdas` [42.3 ms plus 3.2 µs per
1,000 cells]:

| N (leaves) | rows | cells | projected proof | ceiling |
|---|---|---|---|---|
| 10 | 1,280 → 2,048 | 51,200 | 208 ms | 48 op/s |
| 50 | 6,400 → 8,192 | 204,800 | 705 ms | 71 op/s |
| 100 | 12,800 → 16,384 | 409,600 | **1,368 ms** | **73 op/s** |
| 500 | 64,000 → 65,536 | 1,638,400 | 5,344 ms | 94 op/s |

⚠️ It is an **optimistic bound**: the `n·log n` term of the FFTs and the
memory pressure push the real cost above the line.

**The batch circuit would be the new bottleneck**: ~73 op/s against the
**265-320 op/s** that `apply` already reaches since §209 — **~4× worse**,
in exchange for two new circuits, their ESPEC, their census, and the
whole under-constraint surface of class §3.1.

#### 2.2 Why new circuits are not needed either

A client's proof asserts: *«my old leaf is in `root_old`, and applying
my change gives `root_new`»*. That **is still true inside a batch**:
`root_new` is simply «the root if mine were the only change since the
starting root».

And the node **already knows how to compute that hypothetical root**: it
is the pattern `tentativo = árbol.clone(); set_leaf(…)` that already
lives in `commitment.rs` and `burn.rs`, and that since §207 costs one
climb —32 hashes—.

So `apply_many` can:

1. Fix the batch's **starting root** (snapshot of the three trees).
2. For each operation: compute its hypothetical root **from the start**
   and verify the proof against it, just as it verifies it today.
3. Apply the N leaves to the real tree and recompute the root **once**.
4. Reject two operations of the **same account** in the same batch — the
   node knows which account each one belongs to, so **there is not even
   a need to advance the `nonce`**.

**Without touching circuits. Without breaking the wire. Without
`zkssl/0.3`.**

#### 2.3 The three required checks, and their result (§210)

| check | result |
|---|---|
| **Does anything depend on the CURRENT root and not on a starting one?** | ✅ **No.** Everything `apply_send` checks after the lock uses only `pi`, `self.regulatory_limit` and `self.records`. The two root checks are the only ones, and both admit the snapshot |
| **Do the three trees survive?** | ⚠️ **NO, and here is the blocker** — see 2.4 |
| **Does supply conservation survive?** | ✅ **Yes.** It is structural and per operation —the balance goes down, the note goes up (`pending_amounts`)—; there is no recomputed total that could go out of balance. Conditional on the positions not colliding, which is 2.4 |

#### 2.4 The blocker: `allocate_pending` collides in a batch

```rust
pub(crate) fn allocate_pending(&self) -> Result<u64, LayerError> {
    for p in 0..self.next_pending {
        if !self.pending.is_occupied(p) { return Ok(p); }   // ← estado ACTUAL
    }
    Ok(self.next_pending)
}
```

It is **read-only over the current state**. Two clients that ask for
materials against the same starting root **receive the SAME position**:
their proofs assert inserting into the same leaf of the pending tree,
and the second `apply` would overwrite the first one's note.

In the bench `etapa_a5_concurrencia` (§204) it did not happen because
the mutex serialised materials-and-application. **In a batch there is
nothing to prevent it.**

**Fix: reservation of positions.** A counter that advances **when
materials are handed out**, not when occupancy is observed. It is a
**layer** change, not a circuit one.

⚠️ **And a second one, minor**: a **governance freeze** in the middle of
a batch changes `frozen_root` and kills every proof in flight. It is
declared: **freezes go in their own batch**.

#### 2.4.bis The chained log — finding of §213

Before writing `apply_many`, `log::verify_chain` was read and a
condition appeared that the plan of §210 had not taken into account:

```rust
if i > 0 && e.root_old != self.entries[i - 1].root_new {
    return Err(LogError::BrokenChain { at: e.seq });
}
```

And its twin `verify(genesis_root)` follows an expected root from
genesis. That is: **the log demands carrying the REAL chain of roots**
— the guarantee the README publishes as «reescribir el historial →
registro encadenado de transiciones» [rewriting history → chained log
of transitions].

**The problem.** In a batch, all the proofs declare the same
`root_old` and **hypothetical** new roots that do not chain with each
other. Writing into the log what the proof declares would produce a log
that **fails `verify_chain`**.

**Three ways out, and the decision.**

| | what it does | price |
|---|---|---|
| (a) the batch is **a single entry** | it chains | an entry stops corresponding to a concrete proof; the log format changes |
| **(b) N entries with the REAL roots** | **it chains, and with N=1 nothing changes** —the real root *is* the hypothetical one, proven by the test of `root_with` in §212—, so the vectors stay intact | an entry's `rootNew` stops being the one its proof declares; tying them demands replicating the tree |
| (c) not grouping | nothing changes | the contention stays |

**(b) is chosen**, and not by preference: `verify` **demands** the real
chain, so it is the only one compatible with what is already published.

**What was measured to decide it** (§213): that **nothing in the tree
verifies the link between the log's roots and the ones the proof
declares**. The only link is the construction —`append(OpKind::Send,
pi.root_old, pi.root_new, …)`—, which is a choice of what to write, not
a check. Changing what is passed to `append` breaks no existing
verification.

⚠️ **But a capability is lost**, and it is declared in `spec/RPC.md`
(«Qué afirma el registro de transiciones» [what the transition log
asserts]): today, with an entry and its proof, a third party can tie
them **without having the tree**; with batches it will need to
replicate it.

⚠️ **And the composition of the batches becomes observable**: two
implementations that group differently produce different chains for the
same sequence. That is why the vectors declare their grouping policy,
and those of `0.1` and `0.2` were generated **without batches (N=1)**.

#### 2.5 The three pieces, in order

1. ✅ **Reservation of pending positions — DONE (§211).**
   `reserve_pending` / `release_pending` / `reserved_pending_count`;
   `allocate_pending` counts a reservation as occupied. **Reservations
   are not persisted**: a restart cancels them, which is correct —if
   nothing was applied, there is nothing to respect—. Four tests, and the
   first one shows the bug before fixing it: without reserving, two calls
   return the same position.
2. ✅ **`apply_many` — DONE (§215).** Starting snapshot, the N validated
   against it **without mutating**, application afterwards, and `append`
   with the REAL roots. Four tests, and the first one shows what the
   stage existed to achieve: **two proofs generated against the same root
   are both applied**.
3. ✅ **One operation per account and per batch — DONE (§215)**, inside
   `apply_many`. It turned out **not to need the `nonce`**: the node
   knows which account each operation belongs to, so the circuit is not
   touched.

All three are of the layer. **Zero new under-constraint surface**: class
§3.1 does not come into play, and that is why this stage **no** longer
demands an executable ESPEC or a census — there is no new circuit to
take a census of.

### Stage 3 — Incremental tree (implementation, **without an RFC**)

`SparseTree::root()` recomputes the whole tree on every call, and at
every node it decides occupancy with a **linear sweep of all the
leaves**.

**A.3 refuted that this dominates `apply`** (exponent 0.18: flat). But
it measured that **`send_materials` is linear (e = 1.08)** and grows:
0.64 ms with 4 accounts, 11.84 with 60, and **~248 ms extrapolated to
1,000** — running **on the node**, on every send.

**Change:** cache the root and invalidate it on insertion; store
internal nodes and update only the path of the modified leaf.
`O(k²·d)` → `O(d)`.

**It touches neither the wire nor the circuits.** It is included here
because without it the other two stages eat up the benefit as soon as
the ledger has a real size. **It does not require an RFC**: it can be
sealed on its own.

### Optional ones, and why they are not a priority

**Aggregated claim** (N pending notes in one operation): it halves the
number of operations per payment. Real, but stage 2 largely subsumes it
and it adds one more circuit —one more under-constraint surface—. It is
opened only if after stage 2 the target is still not reached.

**Partition by account range** (a single operator; the two-phase design
is already the protocol between partitions). Almost linear scaling
**without consensus**. It is opened only if the target peak demands it.
See `doc/DIAGNOSTICO_ESCALADO.md` §6.4.ter, level 3.

---

## What was DISCARDED on measuring

It is kept written so that it is not proposed again without reading why
it fell.

| proposal | why it is discarded |
|---|---|
| **Group commit / journal** (it was stage B) | persistence is **3 %** of `apply` (bench A). It would optimise 1.26 ms out of 38 |
| **Anti-replay by account `nonce`** | the obstacle is not the `nonce` but the Merkle path against `root_old` and the assertion of `root_new`. Removing the root binding reintroduces the double-spend marker retired in §32/§36 |
| **Fixing the tree to speed up `apply`** | exponent **0.18**: flat (A.3). The tree does have to be fixed, but because of `send_materials` — stage 3 |
| **Parallel verification as the main lever** | verification is **7 %** (A.2) |
| **Delegating generation to GPUs** | the **client** proves (`client.rs`); the witness carries material of the spending key. It would be a security regression, not an optimisation |
| **`circuit_batch_root` (the batch circuit)** | measured in §210: it would cap the node at **~73 op/s** (N=100) against the 265-320 that `apply` already reaches (band, §217) — **~4× worse**, and as an optimistic bound. And it turns out unnecessary: the new root is deterministic given the leaves, and whoever replicates the tree recomputes it. The verifiable replica is what `SECURITY.md` §6 and §121 already declare as the path |
| **Changing `circuit_send`/`circuit_claim` so as not to assert `root_new`** | unnecessary (§210, 2.2): `root_new` is still true inside the batch as «the root if mine were the only change», and the node computes it |
| **FRI recursion / migrating to Plonky3 or Miden** | `winterfell` brings no recursive verifier; Miden is a zkVM and contradicts our own finding 8, besides voiding the FV ladder. And stage 2 **does not need recursion** |

---

## Compatibility

| stage | does it break the wire? | vectors |
|---|---|---|
| 1 · the hash | **yes** → `zkssl/0.2` | those of `0.1` **are preserved**; new ones under `0.2` |
| 2 · batches (redefined §210) | **NO** — it is a layer change | intact; `conformance --check` must stay at IDÉNTICO |
| 3 · incremental tree | **no** | intact; `conformance --check` must stay at IDÉNTICO |
| aggregated claim (optional) | yes | idem |
| partition (optional) | yes (addressing of notes) | to be decided when it is opened |

Rule 2 of the PROCESO: **old vectors are never rewritten.**

Stages 1 and 2 break the wire for the same reason (the chain and the
public inputs), so **it is advisable to emit `zkssl/0.2` only once**,
when both are done, instead of two versions in a row.

---

## Security

**Effect on the API principle —the spending key never travels:
NONE.** In all three stages the witness is still the holder's and the
proof is generated locally. Stage 2 **reduces** what the client
asserts; it does not widen what it exposes.

**Effect on the declared debts:**

- **Out-of-band notice (§21):** no change.
- **Single node:** no change. This RFC **introduces no consensus** at
  any stage, deliberately — `SECURITY.md` §6.
- **`--dev`:** no change.

**Risk of stage 1.** Changing the log hash changes the chain and the
epoch head. A verifier that compares old heads with new ones **will see
legitimate divergence**: that is why the version goes up and the old
vectors are preserved. Collision resistance comes to rest on Blake3
instead of Rescue for `proof_digest`; for the use it has —tying an
entry to a concrete proof— it is equivalent or better.

**Risk of stage 2.** The node comes to produce a proof of its own. If it
were missing or incorrect, the update of the tree would be left
unproven: `circuit_batch_root` is **mandatory**, and its absence must be
a verification error, not a warning.

⚠️ **Risk of class §3.1 (`SECURITY.md`).** Stage 2 introduces **new
circuits**, and therefore new under-constraint surfaces — the class
this project declares as the highest priority, and the one that cost
Zcash's Orchard pool four years. **Stage 2 is not sealed without an
executable ESPEC and a census of cells with 0 without an owner**, just
like `circuit_send` and `circuit_claim` in §195-§196. Without that, the
throughput gain is paid in the most expensive currency of this house.

---

## References

- `AUDITORIA.md` §204 (the five measurements, with their tables), §205
  (corrections of `doc/ESCALADO`), §123 (contention), §32 and §36
  (retirement of the nullifier), §195-§196 (executable ESPEC), §198 (RFC
  process), §121 (attested heads)
- `doc/DIAGNOSTICO_ESCALADO.md` — §0.bis (what was measured), §2 (what
  is NOT the bottleneck), §6 (the RTGS target)
- `doc/ESCALADO.md` §2.1 and §2.2 (corrected with what was measured)
- `SECURITY.md` §3.1 (under-constraint), §6 (consensus as the last
  intermediary)
- Benches: `crates/zk-ssl/examples/etapa_a_apply.rs`, `etapa_a2_verify.rs`,
  `etapa_a3_escala.rs`, `etapa_a4_hash.rs`, `etapa_a5_concurrencia.rs`
- Code: `crates/zk-ssl/src/log.rs` (`digest_of_proof`),
  `crates/zk-ssl/src/two_phase.rs` (the three-root lock),
  `crates/zk-ssl/src/sparse_tree.rs` (the linear sweep),
  `crates/stark-experiment/src/circuit_send.rs` (the `nonce` that does not advance)
