# RFC-0016 — One value, one encoding: the canonicity of the core

> **Español:** [`spec/rfc/0016-un-valor-una-escritura.md`](../0016-un-valor-una-escritura.md) is the
> original of this page, in Spanish; it prevails wherever the two differ. This translation is of
> commit `7766efb` (audit entry §917) and was produced by the project's assistant, a Claude Code
> session, in audit entry §918 (`GENAI.md`): it is a generated artefact, not text written by the
> author. Line citations of the form `file:N` refer to the Spanish files.

- **Status:** ACCEPTED (§825) — **rule 4 of the PROCESO, counted.** (1) The spec: `NUCLEO.md`
  section 6, «Canonicidad» [Canonicity], and `MODULO` and `u64_canonico` in the census of its
  section 4; and `PAQUETE.md`, the text `{campo}: 0x… no es canonico` in the catalogue of its
  section 5 and the value its section 9 was missing (§640). E6 applies in the library the rules of
  the CUENTAS family of `NUCLEO.md`, which were already written (§475), and does not move the spec.
  (2) The OpenRPC does not move: not a method, not a field; `spec/openrpc.json` did not change in
  §640 nor in §782. (3) The vectors, new under `zkssl/0.4`, which does not go up: the four
  negatives of E4 —`paquete/rechazo-n-no-canonico.json`, `paquete/rechazo-mmrSize-no-canonico.json`,
  `ancla/neg-ancla-cero-escrito-como-p.json` and `completitud/neg-n-no-canonico.json`—, with their
  manifest line and their fingerprint in `vectors/HUELLAS.sha256`. E6 has no vector: it changes no
  verdict of the kit, because the command judges the path with the `cuentas` module, and three
  falsifiers of `zk-ssl-verify` tie it. (4) The suites: the `--sello` canon VERDE within §640 and
  within §782, and in full also in §817, in §852, in §858 and in §859 (their entries); from the
  tree of §859 to that of this turn neither the kit's code changes nor anything this RFC touches:
  in `crates/` only the layer changes (`zk-ssl/src/snapshot.rs` and its tests),
  `zk-core/src/merkle.rs` and a comment of `settlement-layer/src/sparse_tree.rs`, and in `tools/`,
  `canon.sh` and `artefacto.sh` (`git diff`). The turn only changes text, and over its tree ran the
  eleven tools of the canon's loop, which read that text, and `check_vectores`, `check_catalogo`,
  `check_tests` and `check_modulos`. And on turning it, over the catalogues of this tree, with the
  four negatives among them: the current release, `arqueo-verify-v0.4.3`, published on 21-10-2026
  (§818), with its published binary (`1b120e597897e005…`), passes `paquete/` 89 of 89, `ancla/` 22
  of 22 and `completitud/` 81 of 81; the kit compiled from this tree, whose sources are those of
  0.4.3 except comments of `zk-ssl-hash` (§849) and of `zk-ssl-medio` and a test of the medium with
  its `[[test]]` and its recording (§848), and two scripts of the medium that record (§855), which
  the kit does not compile, the same; and the second implementation, `paquete/` 89 of 89 and
  `ancla/` 22 of 22, with the three of those two families. Proposed in §640, with the first five
  stages built in the same seal; E6, in §782. The design, D-A to D-E, was applied by the session of
  §640 with its decisions REVERSIBLE (D-1 to D-3 of that entry), and it is accepted as such. The
  decision to accept it was delegated by the author («HACER todo» [DO everything]), with rule 4 as
  the criterion, and pushing this seal is its acceptance, as in §607. **What is accepted WITH its
  residue, declared** in «What it does NOT close»: `as_digest` still reduces (D-C) and it is
  watched by tests, not by a gate; the three path texts still have no vector; and the deserialiser
  of `Q` of `zk-ssl-wire` demands the minimal encoding and leaves the bound to whoever uses it. And
  outside the tree, on accepting it: `arqueo-verify-v0.3.0` is still published and accepts the four
  negatives; the current one, 0.4.3, rejects them and carries E6 (see «Compatibility»). ⚠️ §718: E7
  enters after the acceptance, as an amendment drafted in private with its fix and sealed with it
  (`spec/rfc/PROCESO.md` §«Fallos de seguridad» [security flaws]): six new negative vectors under
  `zkssl/0.4`, which does not go up; their texts in sections 2.7, 2.9, 5 and 6.1 of `PAQUETE.md`;
  not a method nor a field of the OpenRPC; and the suites of its seal. What is counted above is
  from §825 and does not include it: 0.4.3 gives VERDE to the act of
  `rotacion/neg-acta-mmrsize-alias.json` —measured in private, and in §718 with the kit from before
  its seal, whose act reader is that of 0.4.3—, and it is rejected by the kit compiled since §718,
  which comes out with the cut.
- **Author:** Ángel José Toranzo Portela
- **GenAI assistance:** Claude, in a Claude Code session in the cloud (§640): it measured, proposed,
  applied, ran the canon and committed, «fuera del paso 4» [outside step 4] of `GENAI.md`; E6
  (§782) and this turn (§825), also in Claude Code sessions in the cloud and outside step 4; E7
  (§718) was drafted in private by Claude, in another Claude Code session in the cloud, with its
  fix, and another one brought it to this tree, also outside step 4 — see
  [`GENAI.md`](../../../GENAI.md)
- **Date:** 2026-10-01
- **Protocol version affected:** `zkssl/0.4` — **it does not go up**: nothing of what the core
  produces moves, not a KAT nor a signed head. What changes is what is ACCEPTED on reading: less.
- **AUDITORIA entry(ies):** §640 (the proposal and its first five stages); §641 (RFC-0017, which
  qualifies a sentence of «What it does NOT close»); §662 (the minimal encoding of `Q` on the wire,
  which «What it does NOT close» cites); §775 (the witness and the SDK read with the rule); §782
  (E6); §825 (the acceptance); §718 (E7, the «Second pass», with its row in the table of stages and
  two marks, in the «Status» and in «What it does NOT close», drafted in private with its fix).
  ⚠️ §825: it said «§640.»: it left out E6, the other two entries that «What it does NOT close»
  cites —§662 and §775— and §641. ⚠️ §865: and it left out §258, which D-E cites.

## Status of the stages

| stage | what it delivers | breaks the wire? | status |
|---|---|---|---|
| E1 — the rule, in the core | `MODULO` and `u64_canonico` in `zk-ssl-hash`; `element_from_bytes`, and with it `digest_from_bytes`, rejects what is not less than `p`; `FormatoError::NoCanonico` | NO | sealed — §640 |
| E2 — the command applies it | `u64_de` of the independent verifier reads canonical; the three digest readers state the rule through `{e}`; the MMR partition stops hanging with sizes greater than `2^63` | NO | sealed — §640 |
| E3 — the second implementation, with the same rule | `tools/segunda/nucleo.py` (one producer, `u64_canonico`, and the same text), `verificador.py` and `juez_cabezas.py` | NO | sealed — §640 |
| E4 — the vectors | four negatives, one per measured defect, in `paquete/`, `ancla/` and `completitud/`, with their manifest entry; the reference and the second reject them with the same text where both read | NO | sealed — §640 |
| E5 — one definition in the prover | the seven private copies of the embedding in `stark-experiment` become `zk_ssl_hash::embeber`, and `SPEND_KEY_DOMAIN` is re-exported from the core | NO | sealed — §640 |
| E6 — the account's path, whole | the six `verificar_inclusion*` demand that the path measure `ACCOUNTS_DEPTH` and that its orientations be those of the receipt's `indice`, with the rules of the `cuentas` module; two new errors, `ProfundidadDistinta` and `IndiceCruzado` | NO | sealed — §782 |
| E7 — the second pass | the rule at the frontiers that were missing: the judge of the age, of the payment in flight and of the pending claim (`zk-ssl-air`) and the second implementation; the store's load, the snapshot, the parameters of `paramsDigest` and the placement that skips the sentinel's index in the layer; the producer of the age and the node's start-up; the kit's acts; six negative vectors (see «Second pass») | NO | sealed — §718 |

## Motivation

The core composes over the Goldilocks field, `F_p` with `p = 2^64 - 2^32 + 1`. A `u64` has `2^64`
values and the field `p`: `BaseElement::new` reduces, and with it two readings of the core.

- `as_digest(x) = [x mod p, 0, 0, 0]` is injective on `[0, p)` and nowhere else:
  `as_digest(x + p) = as_digest(x)` for `x < 2^32 - 1`.
- `element_from_bytes` read eight bytes with `BaseElement::new`: the element `x < 2^32 - 1` had two
  encodings, `x` and `x + p`, and with it every digest that contained it.

The signature attests the digest, and the digest attests each integer **modulo `p`**. The command,
on the other hand, reasons over the integer: the window of the promise (`S - e <= n`), the size of
the MMR, the era of a receipt. What was measured, over four real vectors mutated in a single field,
with the reference binary and with the second implementation (`tools/segunda/`):

| vector | mutation | reference, before | second, before |
|---|---|---|---|
| `paquete/posicion-v2.json` | `cabeza.n` becomes `n + p` | VERDE, exit 0 | VERDE, exit 0 |
| `completitud/no-resuelta.json` | the `n` of the closing and of the receipt become `n + p` | «ventana ABIERTA… el sobre es prematuro» [window OPEN… the envelope is premature], exit 1 | does not read the family |
| `paquete/extension.json` | `nueva.mmrSize` becomes `t + p` | **hangs**: exit 124 at 20 s | ROJO «NO extiende» [does NOT extend], exit 1 |
| `ancla/ancla-exacta.json` | the zero of the `chainDigest` written as `p` | VERDE, exit 0 | ROJO «elemento fuera del campo» [element outside the field], exit 1 |

Four distinct defects from a single cause:

1. **One signature, two heads.** The same XMSS signature and the same co-signatures attest a head
   with `n` and another with `n + p`.
2. **The operator gets out of its own red.** The completeness envelope that names it —«NO RESUELTA
   EN LA VENTANA» [NOT RESOLVED IN THE WINDOW]— becomes «ventana ABIERTA», with the same signature,
   if the `n` the operator served is `n + p`. The accused chooses the verdict.
3. **The command hangs.** The MMR partition doubled `k` while `k * 2 < n`; for `n > 2^63` the double
   of `2^63` overflows, in release it wraps to zero and the loop does not end. With `t + p` the
   signature attested it; an operator that signs a size greater than `2^63` provokes it without
   mutating anything.
4. **The two implementations diverged.** The second, written from the spec, already rejected the
   non-canonical element; the reference reduced it. `NUCLEO.md` section 6 did not say which of the
   two was the rule, and section 9 of `PAQUETE.md` declared that four rejection texts had no vector
   because «no value is known» that `digest_from_bytes` rejects: there was none.

And a hint that the rule was already in the intention: the wire (`zk-ssl-wire`) declares
`WireError::NotCanonical`, «bytes no canónicos para el cuerpo» [non-canonical bytes for the body],
over `digest_from_bytes` and `element_from_bytes`, and could never produce it.

## Design

**D-A — an element is read canonical.** `element_from_bytes` passes the `u64` through
`u64_canonico` before making the element, and rejects with `FormatoError::NoCanonico(x)`, which is
written `0x… no es canonico: no es menor que p = 2^64 - 2^32 + 1` with the value as it arrived.
`digest_from_bytes` inherits it for each of its four elements. With this `element_to_bytes` and
`element_from_bytes` are exact inverses: a bijection of `F_p` onto `[0, p)`, and a digest has one
32-byte encoding.

**D-B — a `u64` that composes is read canonical.** `u64_canonico` is the only producer of the rule.
The command applies it in `u64_de`, to all the `u64`s of the envelope: all of them enter a
composition or reason next to one, and no producer of the house writes one that does not fit. The
rejection names the field, `n: 0xffffffff000005a1 no es canonico: …`, and comes BEFORE recomposing
and before the signature.

**D-C — `as_digest` does not change.** It is frozen core: its KATs and all the heads in custody
depend on it. The alternative of making it injective on every `u64` —splitting the integer into two
32-bit limbs— would move the bytes of every value greater than `2^32` and would ask for a new
version of the preamble for a range that no honest producer uses. The frontier goes in the reading.

**D-D — the MMR partition is total.** `mitad(n)` becomes the highest bit of `n - 1`,
`1 << (63 - lz(n - 1))`: the same partition for every `n` for which the loop ended, tied in the
tests against the old loop, and `2^63` where that one did not end. A huge size no longer hangs: the
path is not that of that history and the verification comes out `false`.

**D-E — one definition of the embedding.** The seven private copies of `as_digest(x: BaseElement)`
in `stark-experiment` were `embeber` letter for letter, and the header of `embeber` had listed them
in its census since §258. They are imported from the core with their usual name; the prover's
`SPEND_KEY_DOMAIN` becomes the core's, which rule R2 of `check_dominios` kept equal by hand. Not a
byte moves: the 403 tests of the prover, its fixed-input KATs and the `zkssl/0.4` conformance tie
it.

## Compatibility

It breaks neither the wire nor the format. What was measured:

- **The 26 KATs of the core**, equal, by the reference and by the second implementation.
- **The ten manifests**, entry by entry, say the same as before in the 316 entries they already
  had; the four new ones are negatives.
- **The signed heads of all the vectors**, which the judge of the second implementation recomposes
  and verifies: none of those that verified stops doing so.
- **The wire**: `digest_from_wire` and `elem_from_wire` inherit D-A, and at last produce the
  `NotCanonical` they declared. The node and the SDK write elements with `as_int`, canonical:
  nothing of what is persisted or published changes in how it is read.

The kit `arqueo-verify-v0.3.0` is earlier and reads by reducing: it accepts the four new negatives.
It is still published, and it is no longer the current one. `arqueo-verify-v0.4.0` (§657) reads
canonical, and measured from inside its downloaded tarball it passes the three catalogues that carry
the four —`paquete/` 72 of 72, `ancla/` 22 of 22 and `completitud/` 74 of 74 (§658)—; 0.4.1, also
from inside its own, the same: 72 of 72, 22 of 22 and 74 of 74 (§661); and 0.4.2, measured from
outside in §671, too: 73 of 73, 22 of 22 and 74 of 74. On accepting this RFC the current one is
`arqueo-verify-v0.4.3` (`doc/KIT.md` section 0), published with its advisory on 21-10-2026 and
produced on `5fd5212`, the commit of §817: measured from inside its downloaded tarball, 84 of 84,
22 of 22 and 81 of 81 (§818); over the catalogues of this tree, point (4) of the header. E6 (§782)
is later than 0.4.2 and 0.4.3 carries it: its commit, `2638f92`, is in
`git log a4e888d..5fd5212 -- crates/zk-ssl-verify/src`, from 0.4.2 to 0.4.3. E6 changes no verdict
of the kit, because the command does not call those six functions.
⚠️ §825: it said «The published kit, `arqueo-verify-v0.3.0`, is earlier and reads by reducing: it
accepts the four new negatives. A new release corrects it, which is the author's decision; until
then, the tree and the published kit disagree in exactly these four vectors, and in no other». The
new release arrived in §657, and since §658 the current kit and the tree read these four the same.

## Security

- **The principle of the API** —the spending key does not travel— is not touched.
- **What it closes**: a signed head fixes each integer, not its class modulo `p`; the operator no
  longer chooses the verdict of the completeness envelope by writing `n + p`; no signed envelope
  hangs the command through its MMR size; and the reference and the second implementation read the
  same byte the same way.
- **After the change, the trick does not serve the operator.** A holder with an `n + p` in hand sees
  a red with a name, and the reduction modulo `p` gives back the `n` that the signature attests:
  with it, the envelope names the operator again.

## What it does NOT close

- **The wire and the witness read `u64` without the rule.** The `QUANTITY` reader of `zk-ssl-wire`
  and the witness's recomposer accept `n + p` and recompose the same digest. What they sign and
  compare is the digest, not the integer, so no third party's verdict changes because of them; it
  stays named, without a stage. ⚠️ §775: the witness already reads with the rule —the minimal
  encoding and the bound `< p`—, and so does the SDK, in the `logSeq` of the record. What remains
  is the deserialiser of `Q` of `zk-ssl-wire`, which since §662 demands the minimal encoding and
  leaves the bound to whoever uses it (`zk_ssl_wire::cantidad_canonica`). ⚠️ §825: RFC-0017
  qualifies the «no third party's verdict changes because of them» in its section «Lo que corrige
  de RFC-0016» [what it corrects of RFC-0016] (§641): read on the wire without the rule, a
  `balance` of the state the holder supplies, written as `b + p`, threw off the conservation of the
  supply on reopening; its E4 reads it canonical in `ClientStateDto` and in `AccountViewDto`, before
  the layer adds or subtracts it. ⚠️ §718: for the witness it was not exact before §775 either:
  with `mmrSize + p` and the signature valid, the history channel anchored at `t = p`, and from
  there every honest head looked `por-detras` to it, which does not stop (measured in private, in
  its loop and with its reader from before §775; the tests of E7 fix it). Of the deserialiser of
  `Q`, what the node reads are indices, positions and quantities that the layer bounds, and no other
  verdict that changes was found (read, without measuring each method).
- ~~**`verificar_inclusion` and its sisters v2 to v6** do not fix the depth of the accounts tree
  nor cross the path with the `indice` that the receipt carries, which today is not used. No
  consumer of the tree calls them; the rule that is applied in the command lives in the `cuentas`
  module. It stays named.~~
  ⚠️ §782: closed in E6. The six apply them before climbing up from the leaf, with the rules of the
  `cuentas` module; a short path or that of another account, with its root and its head
  recomposed, no longer verifies. They still have no consumer in the tree: the command judges with
  `cuentas`, as before.
- **`as_digest` still reduces**, by D-C: a new producer that composes a `u64` without going through
  `u64_canonico` opens the gap again. It is watched by the tests that prick the non-injectivity, not
  by a gate.
- **The three path texts** —`sibling {i}`, `camino[{i}]`, `{mote}: siblings[{i}]`— still have no
  vector: a sibling with an element less than `2^32 - 1` is needed to write it as `x + p`, and the
  captured paths do not bring one.

## Second pass (§718, canonicity)

The rule, at the frontiers that were missing. A `u64` from outside that is composed into an element
is bounded over the integer as it arrives, before reducing:

- **The API of `zk-ssl-air`.** The issuer of the age is an account index, less than `2^32`, and with
  «todos» [all] it is 0 (`comprobar_enunciado`); the amount and the term of the payment in flight,
  and the lower bound and the birth of the claim, do not exceed the ceiling over the raw `u64`
  (`verificar_contra_cabeza`). The second implementation carries the same bounds, with the same
  causes.
- **The layer's producer and the node**: `prueba_de_edad` and `--edad-emisor` reject the issuer that
  is not an index, before reading the ledger.
- **The store's load and the snapshot**: the meta of the pending items demands an issuer that is the
  sentinel or an index of the accounts tree, and a canonical birth; the balances, the pending
  amounts, the supply and the issuance cap, below the layer's cap (§668). The layer assigns to no
  account the index `2^32 - 2`, which in the field is the issuance sentinel, and the snapshot does
  not import an account at it.
- **The parameters of `paramsDigest` and the head's counters**: the refund term and the custodian
  cap, which no root covers, are read canonical from the store; the limit and the accounts cap, on
  opening, at start-up and in the snapshot. The layer's two settings do not accept a value of `p`
  or more. `nextPending` and `nextIndex` are read within the capacity of their tree. With `x + p`
  the head signed the same `paramsDigest` and the layer applied another rule: the term `+ p` blocked
  forever the refund and the un-issuance of the v1 path, and the cap `+ p` left the custodian quota
  without a cap.
- **The witness**: its quantity reader demands the minimal encoding and `u64_canonico`, like the
  kit, since §775; E7 adds the tests to it, with the signed heads of the vectors through its loop,
  and the measurement of the anchor at `t = p`.
- **The acts**: the kit's reader and that of the second implementation demand `u64_canonico` in
  their quantities.

Except the act, none of those aliases let a false statement through the kit or the second
implementation, nor broke conservation, nor allowed impersonation. The act signed with `mmrSize`
equal to `N` gave VERDE with `N + p` in the kit and in the second: they accepted an act that
declared a size nobody signed (`spec/vectors/rotacion/neg-acta-mmrsize-alias.json` fixes it). The
most serious was in the store: a rewrite in the clear could change to whom a pending item returns,
or move a signed rule, without the integrity gate, the signed root or the `paramsDigest` seeing it.
The pending item's case was latent because the node does not expose the refund.

What it does NOT close: an earlier ledger with an account already at index `2^32 - 2` opens, and in
it the meta leaf of a payment of its own is still that of an issuance; its snapshot, on the other
hand, is not imported. It was decided so, by delegation of the author, and it is not migrated to a
sentinel outside the range: migrating would change `PMETA` and `root:pmeta`, and reaching that index
takes some 4,300 million accounts. The sentinel of the delay of the reversal envelope, D-AG of
RFC-0008, remains declared there.

## References

`spec/NUCLEO.md` section 6, «Canonicidad»; `spec/PAQUETE.md` sections 5 and 9; `zk-ssl-hash`
(`MODULO`, `u64_canonico`, `element_from_bytes`); `zk-ssl-verify` (`u64_de`, `mmr::mitad`, and since
E6 `inclusion::exigir_camino_de_la_cuenta`); `tools/segunda/nucleo.py`; `AUDITORIA.md` §640,
§782 and §825.
