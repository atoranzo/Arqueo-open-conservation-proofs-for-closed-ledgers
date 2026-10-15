# Issue draft — RustCrypto/signatures

> **Internal note — strip before sending** (§710 of `AUDITORIA.md`, 2026-10-08). Still a draft:
> there is no record of it having been sent. Its part 2, the OID defect, is already reported upstream
> in [RustCrypto/signatures#1442](https://github.com/RustCrypto/signatures/issues/1442). As read on
> 2026-09-28 (`doc/ecst/VERIFICACION.md`, G10c), #1442 was opened on 2026-09-10 by another account
> and does not mention parts 1, 3 and 4; this project's author did not open it, as he said on
> 2026-10-07. If this draft is sent, part 2 goes as a comment on #1442, with the count corrected
> below, and not as a new issue. What is sent starts at **Title**, without this note or the ones
> marked ⚠️ in parts 2 and 5. (⚠️ §793: until here this said «the one marked ⚠️ in part 2»; part 5
> and its note are from §793.)
>
> ⚠️ **Internal note — strip before sending** (§861 of `AUDITORIA.md`, 2026-10-15). **Decided by
> delegation of the author** (entry 77 of `BACKLOG.md`): what is sent is a NEW issue with parts 1,
> 3, 4 and 5, which keep their numbers, and, in the place of part 2, only the line marked «sent in
> place of part 2» below; part 2 itself, from its heading to the workaround paragraph, is not sent,
> nor are the notes marked ⚠️. Not as another comment on #1442 —it is about the OID only, and the PR
> that says it fixes it will close it when merged, burying the rest—, nor one issue per part: this
> draft already asks first and offers the PR. As read on 2026-10-15 with WebFetch —a summary, not
> the raw page: GitHub's API and HTML gave the session 403—, #1442 is still open, with a PR that
> says it closes it, [#1468](https://github.com/RustCrypto/signatures/pull/1468) («xmss: parse raw
> OIDs by the expected key kind», opened on 2026-10-15, «Fixes #1442»), and it carries one comment
> by the account `atoranzo`, the owner of this repository's `origin`, dated 2026-09-26, with the
> `SigningKey` point and the 21 of 56: «if sent, part 2 goes as a comment on #1442» is already
> done. G10c, which read it with no comments on 2026-09-28, does not record that comment; confirming
> it is his is the author's. RustCrypto's `master`, read raw on the same day, is at 0.1.0-pre.1,
> unpublished, and still has no `index()` or `remaining()`, still derives `Clone` on `SigningKey`,
> still declares `rand = "0.10"` without `default-features = false`, and still has the `or_else` of
> `xmss.rs:21`: parts 1, 4 and 5 stand. Part 3 was not checked. crates.io has only 0.1.0-pre.0.
> Check `master` again on the day it is sent; sending it is the author's.

**Title**: `xmss: expose signature index on SigningKey (index() / remaining()); BDS traversal state plans?`

---

Thanks for publishing `xmss` 0.1.0-pre.0 — we use it to sign the
epoch heads of an append-only settlement log (one signature per epoch), where
index reuse means key compromise, so we persist an external monotonic counter
with fsync *before* each signature and reconcile it against the key's own
index after restarts.

⚠️ **Changed in §861** (internal; strip before sending): until here the opening
said «we are evaluating it to sign»; the project has signed its epoch heads
with it since §236 of `AUDITORIA.md`.

**1. Could `SigningKey` expose the current index?** Something like:

```rust
impl<P: XmssParameter> SigningKey<P> {
    pub fn index(&self) -> u64;
    pub fn remaining(&self) -> u64;  // for exhaustion monitoring
}
```

Today the only way to read it is parsing `as_ref()` at the reference-format
offset (verified: OID ‖ idx ‖ 4×n), which is fragile and parameter-set
dependent — the index width is ⌈h/8⌉ for XMSS^MT. For state-critical
deployments a typed accessor removes a whole class of layout bugs. Happy to
send a PR if you'd take one.

Prior art, if useful: `oxicrypt-xmss` 0.22.0 exposes `leaf_index(&self) -> u32`
and `is_exhausted(&self) -> bool` on its private key, and documents an internal
state counter that refuses to sign past exhaustion. Different scope (single
parameter set, XMSS-SHA2_10_256) but it shows the accessor is practical.

---

*(Sent in place of part 2.)* **2. The OID defect** is already
RustCrypto/signatures#1442, with PR #1468 open to fix it; nothing to add
there.

⚠️ **Added in §861** (internal; strip before sending): the line above is what
goes in the new issue for part 2. From the heading below to the workaround
paragraph, part 2 stays here as the record of what was found and of the
comment already on #1442; it is not sent.

**2. Keys of 21 of the 56 XMSS^MT sets cannot be parsed back from their own
bytes** (verifying **and signing**)

```rust
let kp = KeyPair::<XmssMtSha2_40_8_256>::from_seed(&seed)?;
let bytes = kp.verifying_key().as_ref().to_vec();
VerifyingKey::<XmssMtSha2_40_8_256>::try_from(bytes.as_slice())
// => Err(InvalidOid(5))
```

The same round-trip works for single-tree sets (`XmssSha2_10_256`, OID 1).

RFC 8391 keeps **two separate OID registries** — XMSS and XMSS^MT — and both
start at 1. `XMSSMT-SHA2_40/8_256` is wire OID `0x00000005`, which the key
correctly carries. But in `parse_oid_and_params`:

```rust
let oid = XmssOid::try_from(raw).or_else(|_| XmssOid::from_xmssmt_raw_oid(raw))?;
```

`try_from(5)` **succeeds** — 5 is a valid single-tree OID
(`XmssSha2_16_512`) — so the `or_else` branch never runs, the wrong variant
comes back, and the equality check against `P::oid()` fails.

**21 of the 56 XMSS^MT sets collide** with valid single-tree OIDs: every
set whose raw OID is 1..21, that is, all eight SHA2-256 sets (1..8), all
eight SHA2-512 sets (9..16) and the first five `XMSSMT-SHAKE_*_256` sets
(SHAKE128, n = 32; 17..21). For each of the 21, both the verifying key and
the signing key are rejected with `InvalidOid` when read back from their own
bytes; the other 35 sets round-trip. The failure is closed: the read is
rejected, it never yields a key of another set. Signatures are unaffected:
`Signature::try_from` does not parse an OID.

⚠️ **Corrected in §710** (internal; strip before sending): until then this
paragraph said «Five of the eight SHA2-256 XMSS^MT sets collide with valid
single-tree OIDs (1..8); their verifying keys are all unrecoverable», and the
heading of this part said «XMSS^MT keys cannot be parsed back from their own
bytes». All eight collide, since 1..8 are all valid single-tree OIDs, and 21
of the 56 in all, signing keys included; the other 35 parse back. Measured on
`xmss` 0.1.0-pre.0.

The existing serde/postcard round-trip tests use `XmssSha2_10_256`, so the
multi-tree case is not covered.

One possible fix: have the caller pass the expected registry, or try the
XMSS^MT branch when the parsed single-tree variant does not match `P::oid()`.
Happy to send a PR.

**The same happens to `SigningKey`**, and there it is worse than an
inconvenience:

```rust
let kp = KeyPair::<XmssMtSha2_40_8_256>::from_seed(&seed)?;
let sk = kp.signing_key().as_ref().to_vec();          // untouched bytes
SigningKey::<XmssMtSha2_40_8_256>::try_from(sk.as_slice())
// => Err(InvalidOid(5))
```

Because `KeyPair::new` is `pub(crate)` and gated behind the `pkcs8` feature,
`generate` and `from_seed` are the only public constructors — so a signing key
**cannot be restored at a given index at all**. For a stateful scheme that is
the difference between a node that survives a restart and one that does not:
persisting the key is pointless if it cannot be loaded back, and rebuilding it
at the counter's index is impossible through the public API.

*(Workaround in use meanwhile: add `XMSSMT_OID_OFFSET` to the first four
bytes before calling `try_from`, for both key kinds. Published keys keep the
correct RFC OID.)*

**3. Is cached traversal state (BDS) on the roadmap?** Signing currently
appears to rebuild the tree each time — sign ≈ keygen in our measurements
(~0.62 ms/leaf, x86-64, release): XMSS-SHA2_10_256 signs in ~645 ms vs
~634 ms keygen, and XMSSMT-SHA2_40/8_256 signs in ~160 ms (8 layers × 2⁵).
That is O(d·2^(h/d)) per signature; with BDS state it would be O(h). Fully
understandable for a pre-release — mostly asking whether it's planned, since
it decides which parameter sets are usable at ~1 signature/second.

**4. Minor: `Clone` on `SigningKey` is a footgun for a stateful scheme** —
`let k2 = sk.clone()` followed by signing with both silently reuses an index
(we verified both signatures validate). Perhaps worth a doc warning, or
gating `Clone` behind a feature so the misuse is at least deliberate.

**5. Minor: `rand` with its default features puts `getrandom` in every
consumer, verify-only ones included**, which keeps them off
`wasm32-unknown-unknown`. `xmss` declares `rand = "0.10"` without
`default-features = false`, and rand's defaults (`std`, `std_rng`, `sys_rng`,
`thread_rng`) enable `getrandom` 0.4 with `sys_rng`. Our verifier never
generates a key and still gets it:

```text
$ cargo tree -i getrandom@0.4.3 -e normal -p zk-ssl-verify
getrandom v0.4.3
└── rand v0.10.2
    └── xmss v0.1.0-pre.0
        └── zk-ssl-verify v0.4.2
```

On `wasm32-unknown-unknown`, the browser target, `getrandom` stops the build
with a `compile_error!` unless its `wasm_js` feature is enabled, so a
verify-only consumer would have to opt into a JavaScript entropy backend it
never calls. As far as we can read 0.1.0-pre.0, the library itself only names
the `rand::CryptoRng` trait (`KeyPair::generate`, `xmssmt_core_keypair`),
which rand re-exports with no features enabled, and `rand::rng()` appears
only under `#[cfg(test)]`. So `rand = { version = "0.10", default-features =
false }`, with the defaults kept in `[dev-dependencies]`, might be all it
takes. We have not built it that way; happy to send a PR.

⚠️ **Added in §793** (internal; strip before sending). Measured with `cargo tree
-i getrandom@0.4.3 --locked --offline -p zk-ssl-verify -e normal` on the tree
of §789 (`8c02941`); the output above omits the local path of the crate. With
`getrandom` alone the command refuses as ambiguous, because the verifier's
closure carries two: the other, `getrandom` 0.2.17, comes through `rand_core`
0.6.4 from our own `zk-ssl-air`, the salt of `MerkleConSal` (§535, D-X), not
from `xmss`. So this part alone would not take the verifier to the browser:
our salt blocks it too, and fixing that is ours, not RustCrypto's. Nothing was
built for `wasm32-unknown-unknown`: that both stop the build is read in their
sources from the cargo registry —`getrandom` 0.4.3 `src/backends.rs:170-182`
(`wasm_js`) and 0.2.17 `src/lib.rs:344-349` (`js`)—, as are `xmss` 0.1.0-pre.0
(`Cargo.toml`, `[dependencies.rand]`; `src/xmss_core.rs:135`,
`src/xmss.rs:591`, `src/lib.rs:104`) and `rand` 0.10.2 (`Cargo.toml`,
`[features]`; `src/lib.rs:59`), the pinned versions, not RustCrypto's
`master`. This is the line that §5.3 and §8.4 of
`doc/integracion-vertical-evaluacion.md` asked for, pending since §705.

Details and measurements available on request. Thanks!
