# RFC-0017 — The signature attests the arithmetic: the native binding and the range at 62 bits

> **Español:**
> [`spec/rfc/0017-la-firma-acredita-la-aritmetica.md`](../0017-la-firma-acredita-la-aritmetica.md)
> is the original of this page, in Spanish; it prevails wherever the two differ. This translation is
> of commit `7766efb` (audit entry §917) and was produced by the project's assistant, a Claude Code
> session, in audit entry §918 (`GENAI.md`): it is a generated artefact, not text written by the
> author. Line citations of the form `file:N` refer to the Spanish files.

- **Status:** PROPOSED (§641) — with its four stages built in the same seal and the `--sello` canon
  VERDE within it. The step to ACCEPTED is the author's: rule 4 of the PROCESO asks for the spec,
  the OpenRPC (it does not move: no method and no field changes), the vectors (none new under
  `spec/vectors/`: what enters are negative tests in the crates) and the suites; all that is in
  §641, and the decision is not. ⚠️ §826: not all of it. Its E2 broke the proofs that travel (§699),
  and for a break of the wire rule 4 asks for vectors re-issued or new under the new version, which
  do not exist yet: if they are born, they are born with the `zkssl/0.5` cut, which pays for the
  break, and whether they are born for the layer's proofs is not decided (see «Compatibility»).
- **Author:** Ángel José Toranzo Portela
- **GenAI assistance:** Claude, in a Claude Code session in the cloud (§641): it measured, proposed,
  applied, ran the canon and committed, «fuera del paso 4» [outside step 4] of `GENAI.md` — see
  [`GENAI.md`](../../../GENAI.md)
- **Date:** 2026-10-01
- **Protocol version affected:** `zkssl/0.4` — **it does not go up**: not a method nor a field of
  the wire changes. What changes is what is ACCEPTED: an amount, a balance or a supply in the high
  63-bit window (`[2^62, 2^63)`) stop passing; the field's sound ceiling drops from `2^63 - 1` to
  `2^62 - 1`.
  ⚠️ **Corrected in §699**: the honest proofs from before do not pass either. The periodic selector
  of the seventeen AIRs changes, and a send proof of §640 does not verify with the AIR of §641, nor
  the other way round (measured with the send); `zkssl/0.4` does not say so.
  ⚠️ §826: with the policy of RFC-0021, that is breaking surface S2, the proofs that travel, without
  raising the version; it is paid for by the `zkssl/0.5` cut of RFC-0018 (see «Compatibility»).
- **AUDITORIA entry(ies):** §641 (the proposal and its four stages); §699 (what E2 changed in the
  proofs that travel, measured, and the correction of the header); §719 (it completes E2: the cap
  segment of the issuance to pending and `solvency`, at 62 bits); §723 (the correction of D-3: the
  kit's band and pending claim, at 62 bits with their v2); §778 (the citations inherited from
  RFC-0015 and from §631, corrected); §826 (its compatibility, written, and its security section).
  ⚠️ §826: it said «§641.».

## Status of the stages

| stage | what it delivers | breaks the wire? | status |
|---|---|---|---|
| E1 — the layer does not trust | `validate_send` binds `amount == pi.amount`, the sender's state to the leaf of the tree and `pi.amount <= límite`, and subtracts with `checked_sub`; the claim binds `notice.amount == pi.amount` and adds with `checked_add`; the burn derives the native supply (`checked_sub`) instead of copying `pi.supply_new`, and binds its state to the leaf | NO | sealed — §641 |
| E2 — the range at 62 bits in the layer's AIRs | the seventeen circuits of `stark-experiment` force to zero **two** high bits (63 and 62) of each range segment, not one: the proven range drops to `[0, 2^62)` and a subtraction that wraps around in Goldilocks no longer fits. `MAX_VALUE` of `compliance_circuit`, `double_entry` and `range_check` at `2^62 - 1` | **yes**: S2, the proofs that travel; a send proof of §640 does not verify with the AIR of §641, nor the other way round (§699). ⚠️ §826: it said «NO» | sealed — §641; completed — §719. ⚠️ **Completed in §719**: the cap segment of `MintPendingAir` and of its climb, `MintPendingClimbAir`, has its own selector, `cap_first`, and still had a single row: 63 bits. `solvency`, outside the seventeen, too. §719 lowers them to 62 bits. Like the rest of E2, a proof of those AIRs made before does not verify afterwards; but none of the three travels, with the criterion of E3 (see «Compatibility»): no method of the OpenRPC carries them, and `MintPendingClimbAir`, the only one the layer judges, is verified only by `apply_mint_pending_delegated`, API of the layer, outside the promise. It does not break S2 |
| E3 — the audit binds the identity and bounds the band | `circuit_audit::get_assertions` binds `COL_ID` (the witness's account) to the public `public_id`, and `verify_audit` demands `lower <= upper <= 2^62 - 1` before touching the proof | NO | sealed — §641 |
| E4 — the client's balance, canonical on the wire | `zk-ssl-wire` reads `balance` with the rule of RFC-0016 (`u64` canonical, `< p`) in the two state DTOs, before the layer adds or subtracts it | NO | sealed — §641 |

## Motivation

The core proves `saldo - importe >= 0` (and the bands of the type `inferior <= saldo <= superior`)
by decomposing the **difference** into bits and checking that it fits in a range. Over the
Goldilocks field, `p = 2^64 - 2^32 + 1`, a subtraction `a - b` with `b > a` does not give a negative
number: it gives `p - (b - a)`, an element of the field. If that element fits in the range the
circuit checks, the proof attests a subtraction that **never happened**.

The range was of **63 bits** (`< 2^63`). A wrapped subtraction is worth `p - d`, with `d = b - a`
the deficit. For `p - d < 2^63` it is enough that `d > p - 2^63 = 2^63 - 2^32 + 1`. And as the
operands are also bounded to 63 bits, `d` can reach almost `2^63`: there exists a window of some
`2^32` deficits —those greater than `2^63 - 2^32 + 1`— whose wrapped subtraction **does** pass the
63-bit range. Measured in the session: a send with balance `0` and an amount in that window was
proven and the layer wrote a huge new balance, minting value out of nothing; the claim repeated the
pattern by adding to the balance the client sends; and the burn took the supply from the proof.

The arithmetic that closes it: if the operands fit in **62 bits** (`< 2^62`), a real deficit
satisfies `d < 2^62`, so a wrapped subtraction is worth `p - d > p - 2^62`, which is greater than
`2^62` and **does not fit** in a 62-bit range. `2 * 2^62 < p`, so there is no overlap. The 62-bit
range is sound; the 63-bit one was not.

## What was measured, before touching anything

With the real prover and verifier (release profile), over the layer:

| case | before | after |
|---|---|---|
| send: balance 0, limit 500,000, amount in the high 63-bit window | `Ok(())` — value minted | rejected |
| send: control amount 500,001 | rejected | rejected |
| burn: balance `1e6`, supply `1e8`, amount `2^63 - 1` | `Ok` | rejected |
| audit: band `[2^63-1, 2^63-1]` over balance 0 | `Ok` | rejected (band bound) |
| audit: someone else's `public_id` with a prover of one's own | `Ok` | rejected (identity binding) |

## What it does

1. **The layer does not trust** (E1). `validate_send`: the `amount` parameter has to be the PROVEN
   `pi.amount`; the sender's state, the leaf the tree keeps; `pi.amount <= límite`; and the new
   balance comes out of `checked_sub`, which rejects instead of wrapping around. The claim binds
   `notice.amount` to `pi.amount` and uses `checked_add`. The burn derives the new supply with
   `checked_sub` from the current supply and demands that the proof attest EXACTLY that value,
   instead of copying `pi.supply_new`; and it binds its state to the leaf of the tree, like the
   claim.
2. **The range at 62 bits** (E2). In the seventeen circuits of `stark-experiment`, the periodic
   selector of the first row of each segment (`first_s`) now covers rows 0 **and 1**: the
   constraint `first_s * sbit` forces bit 63 and bit 62 to zero. It changes neither the number of
   constraints, nor the degrees, nor the count of assertions, nor the columns. `MAX_VALUE` of
   `compliance_circuit`, `double_entry` and `range_check` drops to `2^62 - 1` (in `circuit_audit` it
   already was), and `range_check` also fixes bit 62 to zero with an assertion (5 -> 6).
   ⚠️ **Completed in §719**: two places of `stark-experiment` did not have the second row. The cap
   segment of `MintPendingAir` and of its climb, `MintPendingClimbAir`, decomposes
   `tope - suministro_nuevo` with its own selector, `cap_first`; and the range of `solvency` had
   `first[0]` and not `first[1]`. They were still at 63 bits: in the climb, with a cap of `1e8`, a
   «negative» new supply (`tope - margen + p`, with a margin of `2^62 + 5`) verified. §719 puts the
   second row in the three AIRs (`cap_first[CAP_START + 1]` and `first[1]`), lowers
   `EFFECTIVE_BITS` to 62 and `MAX_VALUE` to `2^62 - 1` in `solvency`, and gives each range
   segment, the cap's included, a test that looks at its two high bits.
3. **The audit** (E3). `circuit_audit::get_assertions` binds the four slots of `COL_ID` in row 0
   to the `public_id` of the public inputs (17 -> 21 assertions): the prover attested ITS account
   and declared the `public_id` of another, and the verifier attributed the band to the victim.
   `verify_audit` demands `lower <= upper <= 2^62 - 1` before parsing the proof.
4. **The wire** (E4). `zk-ssl-wire` reads the `balance` of `ClientStateDto` and of `AccountViewDto`
   with the canonical rule of RFC-0016 (reusing `element_from_bytes`): a balance written as `b + p`
   is rejected on the wire, before the layer adds or subtracts it.

## Decisions (REVERSIBLE)

- **D-1: two defences, not one.** The 62-bit range closes the gap in the circuit; the native binding
  closes it in the layer, and it also covers the path of calling `apply` with a trace of one's own
  without going through the house prover. Neither alone is enough: the circuit protects the third
  party that only has the proof; the layer protects the node.
- **D-2: 62 and no less.** `2 * 2^62 < p` is the exact bound that eliminates the overlap; 62 bits is
  the maximum sound range over Goldilocks for a subtraction. `2^62 - 1` is still a ceiling of some
  `4,6 x 10^18` minimum units, far above any real amount.
- **D-3: the kit was ALREADY there.** The kit's AIRs (`zk-ssl-air`: `banda`, `cobro_pendiente`)
  demand `comprobar_enunciado` in the verifier —`lower, upper <= MAX_VALOR = 2^62 - 1` and `lower <=
  upper`— before verifying the STARK, so the third party that verifies with the kit was already safe
  from the wraparound through a NATIVE bound. E3 brings `circuit_audit` and `verify_audit` to the
  same level the kit already had. No AIR of the kit is touched, and therefore no portable vector.

  > ⚠️ **Correction of §723 (the window of the private balance).** For the band and the pending
  > claim, D-3 is false. `comprobar_enunciado` bounds the PUBLIC limits; the band's balance and the
  > claim's amount are PRIVATE, and they were bounded only by segment 0 of their range, which in the
  > kit was still at 63 bits (`primera_s[seg * LARGO_SEGMENTO] = uno`). With an upper bound `u`
  > below `2^64 - 1 - p = 2^32 - 2`, a value of the window `[2^62, 2^63)` passed the band: measured
  > with the kit's judge, a leaf with balance `2^63 - 2^32 + 2` proved «balance in `[0, 0]`» and
  > verified, and with `4_999_999` more, `[0, 4_999_999]`; the controls with `2^62 + 1` did not
  > verify. An honest transition does not leave that leaf, and the layer does not sign it (its cap
  > rejects the balance on loading the store and on importing the snapshot), but an operator that
  > signs a root by hand obtained a false VERDE of `InsufficientBalance` before whoever only checks
  > the head and the band. The pending claim had the same shape in `verificar` with a small upper
  > bound; `verificar_contra_cabeza` fixes the upper bound at `2^62 - 1`, and there the window is
  > empty. The payment in flight is sound. It is closed by a **v2** of `BandaAir` and of
  > `CobroPendienteAir`, with the three segments at 62 bits like the live AIRs of E2
  > (`primera_s[seg * LARGO_SEGMENTO + 1] = uno`), in the kit and in the second implementation. The
  > v1 proofs already issued are judged by their upper bound: from `2^32 - 2` they say what they
  > seem and keep verifying; below it, the command gives the fourth state, «DECLARADA, NO PROBADA»
  > [DECLARED, NOT PROVEN] (exit 3). The only v1 proof under the bound that was already published,
  > that of `rechazo/saldo-insuficiente.json` (captured from a house node), stays VERDE: the judge
  > recognises it by the fingerprint of its content and its statement, in a closed list that does
  > not grow, and no vector changes verdict. The text above is kept as it was written.
- **D-4: the RFC stays PROPOSED**; accepting it is the author's.

## Compatibility

⚠️ §826: this section did not exist, and rule 2 of the PROCESO asks for it. What the RFC said of its
compatibility was in its header («it does not go up») and in «What it does NOT do» («It does not
raise the wire»), and §699 corrected the header with what it measured.

**It breaks the wire, and the version did not go up.** E2 changed the periodic selector `first_s`
of the seventeen AIRs of the layer, and with it the bytes of the proofs that travel, surface S2 of
D-A of RFC-0021: a send proof of §640 does not verify with the AIR of §641, nor the other way round.
§699 measured it with the send, between nodes and clients from six points of the history that all
say `zkssl/0.4` (`spec/RPC.md`, «Versionado» [versioning], in «Notas operativas» [operational
notes]). By rule 2 of the PROCESO that raises the version, and by R1 and R3 of RFC-0021 too; the
wire stayed at `zkssl/0.4` because the conformance of `0.4` does not compare proof bytes (RFC-0009,
D-AI). What §699 did was write the notice of what was already broken: the line under the title of
`spec/RPC.md`, the description of the OpenRPC and the «Estado» [Status] row of the READMEs.

**The cut pays for it.** The break is paid for in the `zkssl/0.5` cut of RFC-0018, together with
those of E1 (§680) and E2 (§684) of that RFC, which changed other proofs of the wire under the same
version: one break for several reasons (R4 of RFC-0021). The vectors that rule 4 asks for from a
break of the wire —re-issued or new under the new version— are not decided. They would be born in
the cut if E3b of RFC-0021 keeps as a vector the positive proof of each family of the layer, as its
D-D proposes, judged in each canon by the node's verifier; the kit's does not judge it, because
today the kit does not verify send, claim, burn or threshold proofs
(`spec/rfc/0018-el-tren-0-5.md:110`) and this RFC touched no AIR of the kit (D-3). That changes if
the strong version of `ProofFailed` and `VerificationFailed`, which E3 of RFC-0019 brings to the
same cut, gives the kit a verifier of the layer's proofs (the table of stages of that RFC says so).
Today D-G of RFC-0018 foresees positives only of the five families of the kit
(`spec/rfc/0018-el-tren-0-5.md:399`), and in the families that only the node judges their cases go
as tests of the layer, not as vectors (`:424-426`); E3b refers to it for those families. Until the
cut none of `spec/vectors/` changes, and this RFC stays PROPOSED: rule 4 asks for vectors to accept
it, and which ones cover its break is decided by the cut.

**The rest stops accepting, without changing what is written.** E1 and E4 change neither a method,
nor a field, nor the encoding of a value: they stop accepting what minted value or threw off
conservation —an amount, a balance or a supply in the high 63-bit window, which no real amount
reaches (D-2: the sound ceiling drops to `2^62 - 1`); a state that is not the leaf of the tree; a
balance written as `b + p`, which no producer of the house writes—. E3 changes the audit's AIR,
which does not travel: no method of the OpenRPC carries it, and the API of the layer is outside the
promise (D-A of RFC-0021). Not a method, nor a field, nor a KAT, nor a signed head moves, and the
kit's AIRs are not touched (D-3), so neither are its portable vectors.
⚠️ §719: what §719 completes of E2 —the cap segment of `MintPendingAir` and of its climb,
`MintPendingClimbAir`, and the range of `solvency`, at 62 bits— changes three AIRs that do not
travel either, for the same reason as the audit's: no method of the OpenRPC carries them and the
node verifies none of them. `MintPendingClimbAir` is verified only by
`apply_mint_pending_delegated`, API of the layer, and the `MintToPending` entry of the log carries
the digest of the seal of its authorisation, not that of that proof (`spec/RPC.md`, the table of
`proofDigest`); `MintPendingAir` and `solvency` are verified by no production judge. The census of
`crates/zk-ssl/src/instrumento_revela.rs` marks the climb, like the audit, because the layer
verifies it, and D-G of RFC-0018, in proposal, counts it among the fourteen families that the layer
judges, which the cut would version with its v2 mark; neither the one nor the other makes them
travel under `zkssl/0.4`. It does not break S1 or S2, and it does not add to the breaks of
`zkssl/0.4` that `spec/RPC.md` and the READMEs declare.
⚠️ §723: «this RFC touched no AIR of the kit (D-3)», above, is true of the RFC, and the D-3 that
justified it was false for the band and the pending claim (see its correction): the value they bound
is private, and in the kit its range was still at 63 bits. §723 lowers them to 62 bits with a v2 of
the two AIRs, in the kit's judge, in the provers and in the second implementation. Those do travel:
the band, in the `InsufficientBalance` rejection envelope that the node writes (`--prueba-rechazo`),
and the claim, in the pending-claim envelope that the client assembles; both are judged by the kit.
The v2 changes the selector of a periodic column, and with it the bytes of those proofs: 0.4.3 and
earlier do not verify a v2 proof. It is a break of S2 without raising the version, already under the
promise, which governs since §799, and the same `zkssl/0.5` cut pays for it, like that of E2. The
kit keeps reading the v1 already issued, by its upper bound or, the only one already published under
the bound, by its fingerprint in a closed list, and below it gives the fourth state, exit 3: a
security fix that changes a verdict within `0.N.x`, with its advisory. No published vector changes
verdict or moves; eleven new ones enter, with their lines at the end of the manifests of `rechazo/`,
`completitud/` and `pendiente/`. It adds to the breaks of `zkssl/0.4` that `spec/RPC.md`,
`spec/README.md` and the READMEs declare; the description of the OpenRPC speaks of the proofs that a
node accepts over the wire, and these no node accepts, so it does not touch it.

## Security

⚠️ §826: this section did not exist, and rule 3 of the PROCESO asks for it. The spending key does
not travel, and this RFC does not touch it: it changes checks of the layer, the range of the layer's
AIRs, the binding of the audit's public identity and the reading of the balance on the wire. What it
closes is said by «Motivation» and «What was measured»: a wrapped subtraction that minted value in
the send, the claim and the burn, and an audit that attributed its band to another account.

## What it does NOT do

- It does not raise the wire nor touch the OpenRPC. It does not move a KAT nor a signed head.
  ⚠️ §826: it did not raise the wire, and its E2 broke it: see «Compatibility».
- It neither adds nor regenerates a vector of `spec/vectors/`: what enters are three negative tests
  in the crates (someone else's `public_id` in `stark-experiment`, the band bound in `zk-ssl`, the
  non-canonical balance in `zk-ssl-wire`).
- It does not touch the kit's AIRs (`zk-ssl-air`), already protected by `comprobar_enunciado`
  (D-3), nor their portable vectors (`edad`, `pendiente`, `pago`, `prenda`), which are captures not
  regenerable in `--sello`.
  ⚠️ §723: «already protected» is false for the band and the pending claim: see the correction of
  D-3.
- It does not run `--bancos` nor `--completo`: only `--sello`.

## What it corrects of RFC-0016

§640 (RFC-0016) closed with «no third party's verdict depends» on the wire's `u64` reader without
the canonical rule. E4 closes it for the client's balance: the state the holder supplies is read
canonical before the layer uses it, so that a balance written as `b + p` no longer throws off the
conservation of the supply on reopening. It stays named in §641.
⚠️ §778: this RFC said «RFC-0015» in row E4, in point 4 of «What it does», in the title of this
section and here, and «§631» as the entry of the quoted sentence: they were the numbers that
canonicity carried in its branch before being renamed, which §641 gives as «today §631» and
«already renamed from RFC-0015 to RFC-0016». In the tree, the rule of the canonical `u64` is E1 of
RFC-0016, whose entry is §640, and the quoted sentence is in the «Lo que NO cierra» [what it does
NOT close] of that entry; RFC-0015 is the signer's lifecycle, and §631, E2a of RFC-0013. §641
carries the same two attributions and, being an entry, is not rewritten.
