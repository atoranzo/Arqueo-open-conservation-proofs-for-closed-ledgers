# Evaluation of XMSS candidates — entry 53 (filled in)

> **Español:** [`doc/xmss-evaluacion.md`](./xmss-evaluacion.md) is the original of this page, in
> Spanish; it prevails wherever the two differ. This translation is of commit `7766efb` (audit entry
> §917) and was produced by the project's assistant, a Claude Code session, in audit entry §918
> (`GENAI.md`): it is a generated artefact, not text written by the author. Line citations of the
> form `file:N` refer to the Spanish files.

**Status**: evaluated. Tests run on 01-08-2026 on the project's machine, `--release` (`~/eval-xmss`:
`tests/evaluacion.rs`, `tests/medidas.rs`; the crate's own suite from a clone of `master`).

**Convention intact**: ✅ checked with the test · ❌ fails the test · ⚠️ nuance or pending · **«doc»**
= only the documentation says it, it does not count.

---

## 0. Context

Unchanged from the original: `EpochHead` is signed, one per epoch. At 1 epoch/s that is ~31,5 M
signatures/year — and that number, plus what was measured in §5, decides the height **and** the
parameter set.

**The field changed on 25-06-2026**: the `xmss` crate on crates.io belongs today to
**RustCrypto/signatures** (0.1.0-pre.0) — the same organisation as `sha2` and `chacha20poly1305`,
already accepted by policy in the project's `Cargo.toml`. Pre-release and **without an independent
audit**, declared by the crate itself.

---

## 1. Result per criterion — `xmss` 0.1.0-pre.0

| # | test run | result |
|---|---|---|
| **1** | `cargo tree` + grep for curves/lattices | ✅ **0 matches**; the whole tree is hashes (`sha2`, `sha3`/`keccak`) and support, ~24 crates. ⚠️ nuance recorded: `rand` pulls in `chacha20` (a stream cipher, for keygen's RNG) — it is not a new family of assumptions and it is already in the project. |
| **2** | sign → serialise → deserialise → index | The state **persists** (identity round-trip) but **the API does not expose the index**: only `sign`/`sign_detached` + bytes. By the letter («it is discarded»): ❌. Nuance verified: SK = 136 B = **OID(4, =0x00000001) + index(4, BE) + 4×32** — the exact reference format; after one signature only byte 7 changes.
⚠️ **Those figures are for the SINGLE-TREE set** (§236, probe S.3). For
the CHOSEN set, `XMSSMT-SHA2_40/8_256`, it was measured: **SK = 137 B =
OID(4) + index(5, BE) + 4×32**, with the index in bytes [4, 9) —the
width is ⌈h/8⌉ = 5 for h=40, as this same row warned—. On signing
once, **byte 8** changes, the least significant one. Derivable, fragile (in MT the index measures ⌈h/8⌉: the offset depends on the set). **It is downgraded to ⚠️ conditional**: an upstream issue asking for `index()`, and a guardian with its own layout test in the meantime. |
| **3** | sign at n → restore the previous state → sign | ❌ **it signs**: two valid signatures at the same index. A second route without disk: `SigningKey` implements `Clone` — cloning and signing with both reproduces the reuse. **Prediction recorded and fulfilled** — see E1: no candidate can pass this test as it was written. |
| **3b** | exhaust 2^h signatures (h=10) | ✅ `Err(KeyExhausted)` at signature 1025 **exactly**, a named variant, no wrap. |
| **4** | two sets, one simple and one multi-tree | ✅ h=10 and MT 20/2 instantiate, sign and verify. Heights = **a discrete menu** (E4). |
| **5** | vectors in its suite; history | ⚠️ `master` suite: **10/10 in 4,1 s**, but what is visible is functional (round-trips), with no vector file in the published package nor a `tests/` directory. In its favour, **format conformance tested byte for byte**: signatures of 18.469 and 27.688 B (the RFC's figures) and the SK in reference format with the correct OID. Pending: the grep of the 10 names and pinning the commit of the tag (master already differs: `sha3`→`shake`). RustCrypto excludes KATs from the published packages by precedent (ml-dsa), so their absence in cargo tells nothing. |

### Amendments to the protocol (what the tests taught about the tests)

- **E1 · Criterion 3 was unsatisfiable together with 2.** A state serialised to the caller's bytes
  (what test 2 demands) and restored is indistinguishable from a legitimate one: returning an error
  would require information **external** to the blob — which is exactly the counter of §3. SP
  800-208 concedes it by requiring hardware modules for stateful HBS. Reformulation: **3a** signing
  via `&mut` that advances before returning, with no sign-with-index API (here ✅) and no silent
  `Clone` (here ✘, recorded); **3b** hard failure on exhaustion (here ✅); protection against
  rollback is **always** the project's.
- **E2 · The grep of criterion 1 presumes that curves arrive as dependencies.** A monolith defeats
  it (see `purecrypto`): for monoliths the test is features + what gets compiled, not the tree.
- **E3 · «RFC 8391 vectors» do not exist under that name**: the RFC publishes no vectors. The
  canonical ones are those of `xmss-reference` (the RFC's repo) and NIST's ACVP for SP 800-208. The
  criterion stands corrected.
- **E4 · Heights are a menu, not a continuum**: 10/16/20 (XMSS) and 20/40/60 (MT). Rows 25 and 30 of
  the original table do not exist as standardised sets.
- **E5 · Strict «hash only» has a recorded exception**: `chacha20` via `rand`, for key generation.
  No new assumption.

---

## 2. Evaluation table

| candidate | 1 · hash only | 2 · index | 3 · reuse | 3b · exhaust | 4 · height | 5 · reviewed | verdict |
|---|---|---|---|---|---|---|---|
| `xmss` (RustCrypto, 0.1.0-pre.0) | ✅ | ⚠️ cond. | ❌ (E1) | ✅ | ✅ | ⚠️ | **viable, the only usable one of FOUR** (§235) — with a declared mandatory guardian + upstream issue |
| `purecrypto` (KarpelesLab) | ⚠️ proxy does not apply (monolith, E2) | — | — | — | — | ❌ «doc»: *«do not use it for anything real yet»*, from its own author | discarded on 5 without spending the tests |
| QRL's XMSS | — | — | — | — | — | «doc» | discarded from the frame: C++ (qrllib) via FFI — outside the pure-Rust surface and test 1 |
| `oxicrypt-xmss` (oxiforge, **0.22.0**) | ✅ 0 curves, 11 crates | ✅ **`leaf_index()`** | — | ✅ `is_exhausted()` | ❌ **height 10** | ✅ `tests/nist_kat.rs` | **discarded on HEIGHT** (§235): it implements **a single set**, XMSS-SHA2_10_256 → **1.024 signatures in total**. At the heartbeat of 1/min it is exhausted in **17 h**. It falls short by a factor of **30.762×** against the ~31,5 M signatures/year. **It is not a worse crate: it is for another use** —its description says «CNSA 2.0 firmware signing», where 1.024 is more than enough— |

⚠️ **The table had three rows and there was a fourth** (§235). The verdict «only» was correct in the
conclusion and **false in the premise**: it was asserted over an incomplete set of candidates.
`oxicrypt-xmss` is a real candidate —not a monolith, not C++ via FFI, with no warnings from its
author— and **beats the chosen one on three criteria**; height discards it, not quality.

⚠️ **And it reinforces pending item 3**: `leaf_index()` exists and is published. The RustCrypto
issue could be read as a request for convenience; with an implementation in production that exposes
it, it becomes **an omission of `xmss`**. The same with its internal state counter: the guardian
solves a problem that others also solved.

---

## 3. The guardian is no longer a contingency: it is the mechanism

The original §3 stated it as a possible way out; E1 and the tests make it **the only one**: an own
counter, `fsync`, **sign-after-persisting**, with the two original cautions (`fsync` can lie; the
order inverts the natural flow) intact. Two measured requirements are added:

- **Layout test**: the guardian reads the SK's index at the offset of the reference format; it must
  carry a test that signs with a known key and checks that the expected byte moves — so that a
  serialisation change between versions **fails in CI, not in production**.
- **Reconciliation after restart**: if own counter > SK index (a crash between `fsync` and the
  signature), the orphan index is burned by signing a discard or the divergence is recorded; the
  counter is never rolled back.

And as the original §6 demands, **it is declared**: the security of the scheme comes to depend on
this own, unaudited code.

---

## 4. Index and `seq`

Unchanged: **independent counter** (§111.1). Reinforced by E1 — not even the library itself tells an
honest restart from a malicious reuse.

---

## 5. Height: from a theoretical table to measured numbers

**Finding**: in this implementation **signing ≈ keygen** (645 vs 634 ms at h=10, constant over 1024
signatures; MT 20/2 signs in 1,27 s ≈ two layers). With 136 B of SK there is no traversal state:
**it rebuilds the tree on every signature**, O(d·2^(h/d)), not O(h). It is a property of the
*implementation* (BDS would remove it), not of the scheme — the third victim of §5, which the
original table did not have: **latency**.

Validated model (~0,62 ms/leaf on this machine; error < 1 % in two predictions):

| set | signature (measured/derived) | verif | signature size | horizon at 1/s | verdict at 1 s epochs |
|---|---|---|---|---|---|
| XMSS 20 | ≈ 650 s *(der.)* | — | 2.820 B | 12 days | ✘✘ |
| MT 40/2 | ≈ 22 min *(der.)* | — | 5.605 B | ~35.000 years | ✘ |
| MT 40/4 | ≈ 2,5 s *(der.)* | — | 9.893 B | ~35.000 years | ✘ |
| **MT 40/8** | **160,5 ms** *(measured)* | 2,7 ms | **18.469 B** | ~35.000 years | ✔ |
| MT 60/6 | ≈ 3,8 s *(der.)* | — | 14.824 B | 2⁶⁰ | ✘ |
| MT 60/12 | **241,2 ms** *(measured)* | 3,8 ms | 27.688 B | 2⁶⁰ | ✔ |

First-order consequences at 31,5 M signatures/year:

- **Storage**: 40/8 ≈ **0,58 TB/year**; 60/12 ≈ 0,87 TB/year — only in head signatures. It dominates
  the storage of the whole system (compare with 120 MB/1000 transfers). It forces a **retention
  policy** that does not exist: the witness's dispute needs *two* contradictory heads, not the
  history — but deciding it is policy (`PRINCIPIOS.md`).
- **Recurring CPU**: 16 % of one core, permanently (24 % with 60/12). It would disappear with
  upstream BDS — a point of the issue.
- **Keygen** stopped being a cost (18,9 ms at 40/8: only the top layer, 2⁵).

**Proposal**: `XMSSMT-SHA2_40/8_256`. It dominates in signing, size and CPU; 60/12 only buys
horizon, and that of 40/8 (~35.000 years) only bites if the cadence dropped below ~10 ms/epoch. It
is still a decision of `PRINCIPIOS.md`; this table leaves it without unknowns.

⚠️ The 0,62 ms/leaf are from **this machine**: if the target node is ARM (B9), repeat
`tests/medidas.rs` there before pinning anything.
⚠️ §861: decided by delegation from the author (entry 77 of `BACKLOG.md`). The harness
`tests/medidas.rs` lives in `~/eval-xmss`, outside the tree (the header of this document): bringing
it in is only needed if it is measured. The ARM measurement stays in entry 77, and does not go with
B9, which is the client's proof generation: the node and the witness sign, and the witness also
carries its XMSS key. It is measured when the node or the witness has a declared ARM target, or when
B9 is run on ARM —since entry 47 was closed, B9 is in entry 21—, and then the same batch measures
the signature. A figure on ARM is from another machine, and it is not compared with the one from
here without saying so. At the decided cadence, one heartbeat per minute, the 144,5 ms of signing
that §236 re-measured leave a margin of about 415 times: **deduced, not measured**; and, also
deduced, the verdict «at 1 s epochs» of the table does not condition any open decision.

---

## 6. Record and pending items

Entries drafted separately (`AUDITORIA.md §112-114`), including the one the original §6 required:
*none meets criterion 3, and none can*.

Pending items, in order:

1. ✅ **DONE** (§235, probe S.1). **17 tests**, and **there is a KAT in the published package**:
   `test_kat_xmss_sha2_10_256_verify`. The note of §1 said «no vector file», and it is true —there
   is no file— but the KAT is inside the source. Better than recorded.
   ⚠️ **`Clone` is still derived on `SigningKey`**: the footgun of E1 is intact, verified on the
   downloaded source.
   ⚠️ And **there is no `index()` nor `remaining()`**: pending item 3 is still alive.
2. ✅ **DONE** (§235, probe S.1): it was pinned with `cargo add xmss@=0.1.0-pre.0` and the registry's
   source was read, not `master`. **The chosen set exists**: `XmssMtSha2_40_8_256`, among the 56
   multi-tree variants. Criterion 1 re-verified: **0 curves, 0 lattices**; `chacha20` via `rand` as
   the only exception, the one already declared in E5.
3. **Issue at RustCrypto/signatures** (`issue-rustcrypto.md`): `index()`, `remaining()`, the BDS
   plan, and the warning about `Clone`.
4. Guardian: implementation + layout test (§3).
5. ARM measurements if applicable (together with B9). ⚠️ §861: not together with B9: they stay in
   entry 77, with their condition (above, after the table).
