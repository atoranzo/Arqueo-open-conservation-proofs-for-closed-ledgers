# Issue draft — RustCrypto/signatures

**Title**: `xmss: expose signature index on SigningKey (index() / remaining()); BDS traversal state plans?`

---

Thanks for publishing `xmss` 0.1.0-pre.0 — we are evaluating it to sign the
epoch heads of an append-only settlement log (one signature per epoch), where
index reuse means key compromise, so we persist an external monotonic counter
with fsync *before* each signature and reconcile it against the key's own
index after restarts.

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

**2. XMSS^MT verifying keys cannot be parsed back from their own bytes**

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

**Five of the eight SHA2-256 XMSS^MT sets collide** with valid single-tree
OIDs (1..8); their verifying keys are all unrecoverable. Signatures are
unaffected: `Signature::try_from` does not parse an OID.

The existing serde/postcard round-trip tests use `XmssSha2_10_256`, so the
multi-tree case is not covered.

One possible fix: have the caller pass the expected registry, or try the
XMSS^MT branch when the parsed single-tree variant does not match `P::oid()`.
Happy to send a PR.

*(Workaround in use meanwhile: add `XMSSMT_OID_OFFSET` to the first four
bytes before calling `try_from`. Published keys keep the correct RFC OID.)*

**2. Is cached traversal state (BDS) on the roadmap?** Signing currently
appears to rebuild the tree each time — sign ≈ keygen in our measurements
(~0.62 ms/leaf, x86-64, release): XMSS-SHA2_10_256 signs in ~645 ms vs
~634 ms keygen, and XMSSMT-SHA2_40/8_256 signs in ~160 ms (8 layers × 2⁵).
That is O(d·2^(h/d)) per signature; with BDS state it would be O(h). Fully
understandable for a pre-release — mostly asking whether it's planned, since
it decides which parameter sets are usable at ~1 signature/second.

**3. Minor: `Clone` on `SigningKey` is a footgun for a stateful scheme** —
`let k2 = sk.clone()` followed by signing with both silently reuses an index
(we verified both signatures validate). Perhaps worth a doc warning, or
gating `Clone` behind a feature so the misuse is at least deliberate.

Details and measurements available on request. Thanks!
