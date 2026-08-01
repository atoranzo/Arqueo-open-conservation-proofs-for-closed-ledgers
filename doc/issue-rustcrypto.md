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
