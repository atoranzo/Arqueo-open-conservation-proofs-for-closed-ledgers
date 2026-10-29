# RFC-0003 — Pending commitment v2: per-payment Δ and refund identity committed

> **Español:** [`spec/rfc/0003-compromiso-v2.md`](../0003-compromiso-v2.md) is the original of this
> page, in Spanish; it prevails wherever the two differ. This translation is of commit `7766efb`
> (audit entry §917) and was produced by the project's assistant, a Claude Code session, in audit
> entry §918 (`GENAI.md`): it is a generated artefact, not text written by the author. Line
> citations of the form `file:N` refer to the Spanish files.

- **Status:** ACCEPTED -- **rule 4 of the PROCESO, settled with measurement**
  (§343 the adoption; §345-§354 the stages E1-E3 with their gates;
  §356 this turn). It is preserved as a record of what was decided, what was
  measured and what was discarded.
- **Author:** Ángel José Toranzo Portela
- **GenAI assistance:** Claude (sessions 56-57) — see [`GENAI.md`](../../../GENAI.md)
- **Date:** 2026-08-22 (r2; r1, of 2026-08-21, `9bc8f30d3a14e0de`/182, is kept for diff)
- **Affected protocol version:** zkssl/0.2 → zkssl/0.3 (format break)
- **AUDITORIA entries:** §178 (design of the expiry), §180 (execution), §340
  (the bound clock), §342 (who/when are metadata, measured; ⚠️ §861: it was
  measured by the session that sealed §342 and brought by its same commit,
  `6a1c1bb`, to entry 12 of `BACKLOG.md`, but the entry §342 deals with the
  header of `mint` and does not record it; §343 says that this RFC was measured
  on that commit) — and **§343** (the adoption of this RFC), and §352–§354 (the
  whole E3c: SendV2Air, the live path, the 0.3 emission), and §356 (this turn to
  ACCEPTED)
- **Number: 0003, FIRM.** Rule 1 of `PROCESO.md` is consecutive numbering
  from 0001, and `0001` is not missing: it is **reserved** for the hardening of
  the keystore KDF (SHA-256 → Argon2id), «no está redactado» [it is not
  drafted] — numbering note of `0002` (`:30-34`) and `SECURITY.md:428-435` («no
  es un descuido, es una deuda con expediente» [it is not an oversight, it is a
  debt with a file]).

## Motivation

Entry 12 of the BACKLOG was left open by divergence (ii): the policy of §119
asks for «Δ=∞ **pago a pago, elección del emisor**» [Δ=∞ payment by payment,
the issuer's choice] and today `T` is global with a knob (`set_refund_ttl`,
persisted). §342 (⚠️ §861: the session that sealed it, on `6a1c1bb`; its entry
does not record it) also measured the strong claim: **today who can refund and
when are the operator's two metadata** —
`pending_meta[pos] = (sender_index, born)` and `refund_ttl`, neither of which
enters any public input (`RefundPublicInputs = {commitment, amount}`,
`CreditClimbPublicInputs = {root_old, root_new, amount}`). That is the declared
design of §178 («pruebe quien pruebe» [whoever proves]), not a hidden defect;
but for a settlement infrastructure with **verifiable compliance**, a third
party must be able to check that a refund respected the deadline the issuer
chose and credited whom the issuer committed to.

The root cause is measured: the production commitment is
`pending_commitment(receiver_id, salt, amount)` (`pending.rs:70-81`) — **three
fields** — and its native `native_refund_commitment` (`circuit_refund.rs:76-91`)
declares itself identical. The Δ does not fit without breaking the format, and
`canon.sh` demands the vectors `zkssl-0.2.json` **identical**. This RFC pays for
that break with the version process.

## Design

### The principle that orders everything

> **The commitment contains exactly the issuer's choices, and nothing more.
> The operator's facts go signed apart.**

- Issuer's domain (inside C₂): `{receiver_id, salt, amount, f, Δ}`
- Operator's domain (outside, attested): `{born, now}` — the `seq` enters
  `epoch_digest_v3`, which the node signs with XMSS (bound by
  `la_altura_entra_en_el_digest`, §340)

### The commitment

```text
C1 = M( M(receiver_id, salt), [amount,0,0,0] )        // v1, INTACTO como prefijo
X  = M( f, d(Δ) )                                     // el sobre de reversión
C2 = M( C1, X )                                       // v2 = cuatro merges de Rescue
```

where `M` = Rescue merge, `f = derive_public_id(refund_key)` (the issuer's
return key, it may be dedicated) and `Δ` = deadline in epoch heights, chosen by
the issuer. It is the exact form of the prototype `t3a`/`t3b` of `pending.rs`
(«v2 = merge(v1, merge(refund_id, ·)): v1 como prefijo» [v1 as prefix], `:365`),
**with the semantics of the fourth field decided** (see D1).

### D1 — RELATIVE Δ, not absolute expiry

`expiry = born + Δ` is computed by the layer; the issuer commits **only Δ**.
Reasons, by the project's yardstick:

1. **Purity**: `born` is not the issuer's — the layer fixes it on applying. A
   committed absolute would mix a choice with a bet on someone else's datum.
2. **The power of §119.5**: the operator orders the inclusion race. With an
   absolute, delaying the `send` eats the receiver's window; with Δ, the
   window is invariant under delays.
3. **Δ=∞**: the sentinel `Δ = u64::MAX` makes `now ≥ born+Δ` unreachable
   (saturating arithmetic) — the «nadie nunca» [nobody ever] of §119 is a
   particular case, not a branch. ⚠️ Declared: `u64::MAX` is already the
   sentinel of ANOTHER field (`REFUND_SENDER_NONE`, «sin emisor» [no issuer] ⇒
   de-issuance, §180); they do not collide because they live in different
   fields, and this RFC leaves it written.
4. Neither of the two prototypes is exactly this: `t3a` commits the birth
   `seq` and `t3b` an absolute `expiry` (`ahora >= expiry`). The FORM of the
   hash is adopted and the semantics is adapted — measured, not copied.

### D2 — f INSIDE: the destination, committed and verifiable without being revealed

The v2 refund circuit proves: *I know the opening of C₂ **and** the credit goes
to the leaf of f* — with `f` as a **private witness**. The mould exists, **it is
in production and it is paid for**: it is the THIRD MERGE of §117
(`circuit_claim.rs:883-896`) — «digest arrastrado y los CUATRO limbos del rate
:= salt **testigo**» [digest carried along and the FOUR limbs of the rate :=
salt witness], with its column (`COL_LEAF_SALT`) carried constant.
`C2 = M(C1, X)` is exactly that operation with another witness: D2 does not propose a
pattern, it copies a constrained one. (And in `RefundAir`, besides,
`get_assertions` leaves columns 4..12 of row 0 free as a witness,
`circuit_refund.rs:253-254`.) The climb of the destination account already
travels inside the root change that `CreditClimbAir` binds. A third party
verifies that the credited one was the committed one **without knowing who it
is**.

**Relation to §178, cited and superseded**: §178 chose «Ni el compromiso ni
`claim` ni los discriminantes de §173 se tocan» [neither the commitment nor
the claim nor those discriminants are touched] and building «entero sobre
piezas EXISTENTES» [entirely on EXISTING pieces] — that is, not paying for the
format break. This RFC **is** the break, paid for with process: the reason not
to bind expires with it. The attack that §178 does name (the leaked notice,
stealable in the naive design) stays closed by the SAME two locks, now with the
first one verifiable: (1) the destination is no longer fixed by the records but
by the issuer's commitment, and (2) the materials are made only by the issuer
(salt of its leaf, §117) — the prototype exercises it («no eres el retorno»
[you are not the return], «un tercero pasó como retorno» [a third party passed
as the return]).

**The principle «el emisor jamás en el árbol» [the issuer never in the tree] is
PRESERVED, and with a new piece**: the notice to the receiver carries the
**opaque envelope X**, not its opening — the receiver rebuilds `C2 = M(C1, X)`
for its `claim` without learning either `f` or `Δ`. ✅ **MEASURED** (session
57, `PASTE-E1-M`), **and the expectation of r1 came out FALSE and is left
written**: the claim's reconstruction does NOT take `X` as it is — `C_PEND_IN`
binds identity (`COL_ACC_ID`, not `COL_R_ID`: §39) and randomness (`COL_SALT`)
with SEPARATE constraints (`circuit_claim.rs:996-1032`). The v2 claim adds ONE
merge phase for the envelope: one cycle (8 rows, with 208 free after
`ROW_PENDING_ROOT`=815; 26 fit), one periodic flag, one block of constraints
and 4 witness columns (`TRACE_WIDTH` 55→59). The `CYC_*`/`ROW_*` are derived by
summing from the previous one and **propagate by themselves** (`:51-130`). The
cost is derived from the constants, not assumed.

### The refund invariant, complete

```text
abre(C2) ∧ (now − born ≥ Δ) ∧ crédito → f
```

- the 1st and the 3rd are proven by the circuit;
- the 2nd is verified by **anyone** with two signed heads (that of `born` and
  that of `now`) and the `Δ` that the opening of the refund makes public —
  detectable with opposable evidence, consistent with the watch of §119.5.
  ⚠️ §861: **today nobody verifies it.** The `Δ` is published nowhere: the
  log keeps only the digest of the proof, the public inputs of
  `RefundAirV2` are `{commitment, amount}` (`RefundPublicInputs`,
  `crates/stark-experiment/src/circuit_refund.rs`) and the opening lives only in
  the `RefundReceipt` (`crates/zk-ssl/src/two_phase.rs`), which no method of the
  wire carries. An operator that composed the send through `send_materials_v2`
  (`crates/zk-ssl/src/client.rs`) knows `(refund_id, delta)` and can apply a
  v2 refund before `Δ` without anyone being able to prove it. The money goes back
  to the issuer: it is the «acelerar devoluciones, nunca desviarlas» [speeding up
  refunds, never diverting them] that `doc/CADUCIDAD_PENDIENTE.md` already
  declares, not a leak. What this line oversold is the opposability. Attesting
  the opened `Δ` is clause (e) of T3b, and it is resolved by the RFC of the
  clock of entry 12 (⚠️ §827: RFC-0023, PROPOSED, takes it up for the floor, in
  its D-J, and leaves as an open question for the author, its 11, attesting the
  opened `Δ` before a third party). The author classified it as a limit of the
  trust model, not as a failure (§861). Read, not measured.

### What is touched, measured

| piece | change |
|---|---|
| `pending_commitment` + 5 production sites (`two_phase.rs:343,433,640,1208`, `client.rs:347`) | they come to compose/carry C₂ (or coexist, see Compatibility) |
| `native_refund_commitment` (`circuit_refund.rs:76-91`) | v2 twin with `X` |
| `RefundAir` | `evaluate_transition` **does NOT change** (one Rescue path with flags, agnostic to the number of merges — measured `:205-247`); `build_trace`, `ROW_*` and `get_assertions` change (trace ~×2: from 2 to 4 merges) |
| `ClaimAir` (`circuit_claim.rs`, `905495e70251de86`/2208) | **it IS touched, measured (session 57)** — asymmetry with `RefundAir`: here the stitching is PER PHASE, so the transition DOES gain a block (phase of the envelope X), plus a cycle, a periodic flag and `COL_X` (55..59). Mould: the third merge of §117 (`:883-896`) |
| `apply_send` / `apply_refund` / `apply_deissue` | the time gate moves from the global `refund_ttl` to the `Δ` of the opening; `refund_ttl` stays as a systemic CEILING or is retired (decision of the cut, declared) |
| `spec/vectors/` | new `zkssl-0.3.json`; `0.2` moves to the regime of `0.1` (rejected) |
| wire (`zk-ssl-wire`) | only if `Δ` is exposed over RPC — today the wire has ZERO lines of expiry (measured §341-M) |

## Compatibility

**It breaks the format of the commitment ⇒ it goes up to `zkssl/0.3`.** And
`0.3` is **FREE**: the status table of RFC-0002 (`:13-19`) gives its THREE
stages CLOSED, with stage 2 **redefined without breaking the wire** (§210) — its
line `:24`, which announced «necesitará `zkssl/0.3`» [it will need
it], expired in that same §210 and was not updated (a defect of 0002,
declared and with a separate correction).

| piece | does it break the wire? | vectors |
|---|---|---|
| v2 commitment (C₂) | **yes** → `zkssl/0.3` | new `zkssl-0.3.json`; **those of `0.2` and `0.1` ARE PRESERVED under their version** — the file stays, and `conformance --check` rejects them as «de OTRA versión» [of ANOTHER version]: the regime already practised with 0.1 (`RPC.md:9`) |
| circuits (`RefundAirV2` · phase X of the claim) | they do not travel | no effect |
| wire (`zk-ssl-wire`) | **no** in E1-E3 — today it has ZERO lines of expiry (measured, §341-M) | no effect until E4, which will bring its own section |

**Rule 2 of the PROCESO: old vectors are never rewritten.**

The legacy is immune **by domain, not by mark**: a C₁ is not a valid opening
of the form C₂ (`t3b_v1_no_es_reversible` proves it against 3 keys × 3
expiry) and the v1 pending ones without meta are already unrecoverable through
the sentinel (§180). Coexistence leaf by leaf: each pending carries its version
implicitly in its form; v1 `claim` keeps working over v1 leaves.

## Security

**Effect on the API principle —the spending key never travels:
NONE.** The refund demands making materials, not showing keys; `f`, `Δ` and
the openings are generated and stay on the client.

- **§173 intact**: C₂ is an opaque `Digest`; `f` and `Δ` are only opened on
  refunding, and before the receiver not even then (opaque X).
- **§119.5 is REDUCED**: the operator can no longer move the deadline (it lives
  in the commitment) nor the destination (idem); it keeps ordering the race and
  speeding up the clock, both with opposable evidence (signed heads).
  ⚠️ §861: speeding up the clock, no: without the published `Δ` there is no
  opposable evidence that a refund was made before the deadline (see «The refund
  invariant, complete»).
- **B18.3 no change**: the issuer with a lost return key does not recover —
  the same declared exposure as today; `f` can be derived from a dedicated key
  to isolate the risk.

**Effect on the declared debts:**

- **Out-of-band notice (§21): it is the debt this RFC TOUCHES, and it is
  declared.** The envelope `X` travels IN the notice, which is still out of
  band. A leaked notice stays where §178 left it, with the first lock improved:
  `X` is opaque (the receiver learns neither `f` nor `Δ`) and the materials are
  made only by the issuer (salt §117). The debt is neither aggravated nor
  settled: **its load changes** — it now also carries the reversal.
- **Single node: no change.** This RFC introduces no consensus at any stage,
  deliberately — `SECURITY.md` §6. The 2nd prong of the invariant is verified
  with **signed** heads, not with more nodes.
- **`--dev`: no change.**

## What was DISCARDED on measuring

- **Absolute expiry**: a window the operator can eat by delaying the
  inclusion (§119.5).
- **Δ in `pending_meta`**: just as movable as today — it would not be «elección
  del emisor» [the issuer's choice], which is what entry 12 demands.
- **f only in records**: not verifiable by a third party — it is exactly the
  current state that §342 left measured (⚠️ §861: the session that sealed it, in
  `6a1c1bb`; its entry deals with the header of `mint`).

## Stages, with their gates

1. **E1 — vectors and natives**: production `commitment_v2` + 0.3 vectors +
   conformance. Gate: the conformance accepts 0.3 and rejects 0.2 and 0.1.
2. **E2 — the circuits**: `RefundAirV2` (trace, ROW_*, assertions,
   credit→f) **and the phase X of the claim** (mould §117) — a session of its
   own, as §178 commands («trabajo que estrena restricciones» [work that
   introduces new constraints]). Gate: the layout guardian goes up by one per
   circuit that introduces new constraints, and the thief-with-notice and the
   third-party-as-return in red.
3. **E3 — the layer**: v2 `apply_send`/`apply_refund`/`apply_deissue`, the
   notice with X, persistence. Gate: full canon + the battery of §178
   (persistence after restart included).
4. **E4 (optional) — the wire**: exposing Δ over RPC, with its wire version.

> ⚠️ **The gate of E1, re-staged (§345).** As it was written
> above, «the conformance accepts 0.3» demands that the `run_send`/`run_claim` of
> the canonical scenario PRODUCE v2 commitments — the notice with X and the claim
> that recomposes C2 are matter for E3, and the scenario runs **real proofs**,
> so it also steps on E2. Measured in session 57 on `conformance.rs`
> (`--emit`/`--check`, `fn escenario`). **E1 stays in the production
> composers and their tests** (`pending_commitment_v2`, `refund_envelope`, the
> native twin); the 0.3 emission and the triple gate of the conformance **move
> to E3**. Meanwhile `zkssl-0.2` stays IDENTICO in every canon: the
> coexistence, proven by the existing gate instead of promised.

> ⚠️ **E2 had a gap, measured and paid for (§352, 2026-08-23).**
> The stages named `RefundAirV2` and the phase X of the claim, but the
> conformance scenario also PRODUCES sends: the commitment that
> the claim collects is deposited by `circuit_send`, and no stage
> contemplated the v2 of the SEND. §352 introduces it as **E3c-1a** --
> `SendV2Air` (`circuit_send_v2.rs`): the fourth merge `C2 = M(C1, X)`
> with the envelope READ in `COL_X` (the pattern of the leaf salt, §117),
> the public inputs of v1, and the envelope never published. The layer
> adopts it in E3c-1b (the materials gain the envelope, dispatch in
> `prove_send`); the 0.3 emission and the turn of the canon stay in E3c-2.
>
> **Note (2026-08-23, §353).** E3c-1b PAID: the live path learns the envelope
> (D-6) and the layer gains the width guard (D-7) -- the rejection of a foreign
> trace is an `Err` of the layer, not a panic of the Air. E3c-2 remains (the
> scenario, the 0.3 emission and the turn of the canon).
>
> **Note (2026-08-23, §354).** E3c-2 PAID and with it **the whole
> E3**: the scenario collects through the v2 path (f = the issuer's id, delta 96
> deliberately different from the global TTL), `zkssl-0.3.json` emitted with the
> snapshot of DECLARED semantics, the triple gate runs LIVE in the canon and
> the live producers say 0.3 (openrpc REGENERATED by the rite).
> Finding of the run: no root moves -- the pending lives outside
> the `state_root` (see §354). 0.2 ruled §209-§354.
>
> **Note (2026-08-23, §356).** ACCEPTED. Rule 4 of the PROCESO is
> settled with measurement: the spec at v0.3 (RPC.md, the title and :864) -- the
> OpenRPC regenerated by the rite (§354) with its three binding tests --
> the re-emitted vectors (zkssl-0.3.json) with 0.2 and 0.1 PRESERVED and
> rejected by the triple gate of the canon -- and the suites green
> (tests_verificacion 25/0/0 and the --completo of §354). E4 stays
> optional, as it was written (:212): if it is paid, it will raise the wire
> version through its own channel. Double thread: the entry §356.

**The first measurable step is already taken** (session 57, `PASTE-E1-M`, output
`b44b24915a67c268`/1175): `circuit_claim` opened and measured. The reconstruction
does NOT admit X as it is; the v2 claim gains a merge phase with a DERIVED cost
(8 rows of 208 free) and a mould of its own (the third merge of §117). **This
design no longer has pieces reasoned without measuring.**

## References

AUDITORIA §178, §180, §340, §342 (⚠️ §861: its measurement, not its entry: it
was measured by the session that sealed it, in `6a1c1bb`) · BACKLOG entry 12
(the (ii)) and note 106 ·
`doc/CADUCIDAD_PENDIENTE.md` (`dbd7f95593d36180`/150) ·
`pending.rs` `t3a_reversion_como_segundo_cobro` (:353-452) and
`t3b_reversion_temporal_nativa` (:456-503) · `circuit_refund.rs`
(`5f4f04c51ee31a26`/469) · `circuit_claim.rs` (`905495e70251de86`/2208) ·
`PASTE-RFC-M` (output `58af1bc0b1cbfe9f`/483) · `PASTE-E1-M` (output
`b44b24915a67c268`/1175) · `PASTE-RFC-N` (output `845c692ce353265b`/384) —
all three on `6a1c1bb`.
