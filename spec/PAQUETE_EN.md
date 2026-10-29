# The portable evidence package — specification of the envelope and of the command

> **Español:** [`spec/PAQUETE.md`](./PAQUETE.md) is the original of this page, in Spanish; it
> prevails wherever the two differ. This translation is of commit `7766efb` (audit entry §917) and
> was produced by the project's assistant, a Claude Code session, in audit entry §918 (`GENAI.md`):
> it is a generated artefact, not text written by the author. Line citations of the form `file:N`
> refer to the Spanish files.

- **Status:** normative, in force since §397 (RFC-0004, stage E1)
- **Protocol version:** `zkssl/0.5` (§914; ⚠️ §914: it said `zkssl/0.4`) — this document does not
  move it: the package does not cross the wire
- **Origin:** `spec/rfc/0004-paquete-de-evidencia.md`; entry that seals it: §397
- **Reference implementation:** the `zk-ssl-verify` binary (`crates/zk-ssl-verify/src/main.rs`)

This document is the **sole normative producer** of the format of the evidence package and of the
contract of the command that verifies it. Until §397 it lived in the header of the binary itself
(lines 1..90, region sha `293990fedc785833`); `spec/RPC.md` delegated there in writing, and that
header had already gone stale once without anything catching it. Here **what was written** moves
in, what was only in the code is added —the complete order of checks, the catalogue of rejections
and the contract of the command— and the binary's header now refers to this file.

**It specifies the ENVELOPE, not the fields.** The values of the package are responses of the wire
**as is**, as the node serves them —whoever rewrites, tampers—, so the semantics of each field
lives in `spec/RPC.md` and here it is only referred to by line. Duplicating it would create two
producers of the same contract, which is the defect this document repairs.

## 1. What it is

What sustains the holder's position before a third party **when the operator disappears or
lies**: the responses of the wire he already keeps, gathered in **one** JSON file and verified
**without the node, without the layer and without the prover** (§243) — only the binary and what
is published. It is the shutdown procedure that `spec/RPC.md` declares in its section «Apagado —
el fin de vida» [shutdown, the end of life] (`RPC.md:814-859`): on closing, the operator publishes
nothing that is not already published; the holder takes what he already keeps; and with this
package he sustains it afterwards.

The package **REPORTS, it does not judge.** It says whether the head is from whom it says, whether
the acknowledgement climbs to the signed root and how many co-signatures attest that head and that
operator. **Which witnesses count and how many are needed is decided by the CLIENT** with its
policy (§319, the witness's `--testigos` and `--k` switches), not by the package: whoever assembles
it may be the operator, and letting him choose his own `k` would give him back exactly what the
co-signature takes away from him.

## 2. The fourteen forms

The binary accepts fourteen objects (the eleventh, since §573; the twelfth, since §592; the
thirteenth, since §633; the fourteenth, since §686). All fourteen are JSON; the skeletons carry
ellipses where the value is a response of the wire, unrewritten.

### 2.1 The v1 package — the position

```text
{
  "v": 1,
  "cabeza": { …payload de zkssl_signedEpochHead con available:true… },
  "acuse": {                                  // OPCIONAL
    "seq": "0x…",                             // la entrada del titular
    "hashPrueba": "0x…64hex",                 // digest de SU prueba
    "s": "0x…",                               // de zkssl_ackPath
    "camino": { "siblings": […], "isRight": […] }
  }
}
```

- `cabeza` is the `result` of `zkssl_signedEpochHead` (`RPC.md:437-482`), with `available:true`.
- `acuse` is what `zkssl_ackPath` returns (`RPC.md:568-739`) plus the `hashPrueba` of the holder's
  entry (the `proofDigest` recorded). If it does not travel, the head alone is proven.

### 2.2 The v2 package — the co-signatures inside (§322)

```text
{
  "v": 2,
  "cabeza":   { …igual que en v1… },
  "acuse":    { …igual que en v1, OPCIONAL… },
  "cofirmas": [ …la respuesta de zkssl_cosigs TAL CUAL, OPCIONAL… ]
}
```

- `cofirmas` is the content of `zkssl_cosigs` unrewritten (`RPC.md:741-783`): each element
  attests that **one** witness saw **this** head of **this** operator, and nothing more.
- **The binary reads v1 and v2**: what is kept does not expire (§290). What the version bump buys
  is that an old binary refuses out loud before a v2 instead of ignoring the co-signatures and
  printing VERDE: *a field nobody looks at is worse than a field that is missing*. That is why **a
  v1 that carries `cofirmas` is REJECTED** — raising the version is exactly what makes them part of
  the contract.
- Two version conventions coexist, and it is worth knowing: the PACKAGE's (`v`) is a bare JSON
  number; each CO-SIGNATURE's (`v`) travels as `Q`, a hex string with `0x`, because it arrives from
  the wire as is (`RPC.md:39-48`). They are not unified: rewriting is tampering.

### 2.3 The extension package (§293)

```text
{ "v": 1, "tipo": "extension", "vieja": {…}, "nueva": {…}, "camino": […] }
```

- `vieja` and `nueva` are two signed **v3, v4, v5 or v6** heads, each the `result` of
  `zkssl_signedEpochHead`; `camino` is the consistency proof between their peaks
  (`RPC.md:785-812`). Whoever keeps the old one checks that the new one **extends** it, with the
  MMR of heads (§291) as judge, without the log and without the node.
- ⚠️ **§712 · RFC-0012 D-C — the clock is the EMBEDDED index.** Consistency proves that the new
  peak contains the old one; with the same `mmrSize` it is the identity, and it does not see two
  DISTINCT heads at the same place in the history: a signed fork passed as an extension, and the
  two directions of the anchor's split view came out VERDE. So, **after** consistency —a red of
  consistency is still its own—, two rules: with the **same `mmrSize`**, the **same
  `epochDigest`** (the same head on both sides is still VERDE), and if not, ROJO
  `extension: mismo mmrSize y digest distinto`; with a **greater** `mmrSize` and the **same
  `publicKey`**, the embedded index of the new one **strictly greater** than the old one's, and if
  not, ROJO `extension: el indice embebido no crece`. An orphan index —reserved and not signed—
  only advances the leaf: it gives no red. ⚠️ With distinct keys the index is not compared here:
  the count is one per operator (2.14), and the crossing between its keys is judged by the
  stretches of the chain (below, rules 3 and 4), not by this clock. The catalogue's rotations
  cross keys from leaf 3 to 13, from 8 to 12 and from 3 to 8, and remain VERDE. The judge is one,
  `el_reloj_avanza`, the same in the consumption (2.4), in the refutation of the conflict (2.5)
  and in the anchor of mode 3 (2.12).
- The form is chosen by `tipo`: without `tipo`, the envelope is the position one; with
  `"extension"`, this one; each of the other eleven leads to its form (2.4 to 2.14), and an
  unknown one is rejected by name (§418). `v` is checked first in every case. ⚠️ §776: until here
  it said «if not, it is the position one», stale since §418, like step 0 of section 4.
- ⚠️ **§648 (RFC-0015 E5a) — `actas`, optional: the continuity of ONE operator.** With the same
  `publicKey` in both heads it is not read. With distinct keys and without `actas`, the usual
  rejection, letter for letter. With `actas` —the list `zkssl_keyActs` serves, each act in the
  JSON of `acta_a_json` (`spec/RPC.md`)—, the two heads belong to the same operator if the whole
  chain holds (rules 1 and 2 of D-C of RFC-0015), it leads from the key of the head with the lower
  EMBEDDED index to the other one's —the index count is one per operator (D-A), and that is why
  the index orders, not the field's name—, the earlier one signed below the `desde` of its
  successor (rule 3) and the later one inside its stretch, above its `desde` and below the next
  one's if the chain carries it (rule 4). If not, ROJO with the rule that falls: outside the chain
  or backwards, an act that does not hold, or the **OVERLAP**. The judge is one,
  `zk_ssl_verify::actas::juzgar_continuidad`, and it is the same in the eight places that demand
  continuity: here, in the consumption (2.4) and in the six heads the completeness envelope
  compares with its closing (2.11). Its catalogue, since §649, is `spec/vectors/rotacion/`, from a
  real node that rotates (section 9); and since §771, three envelopes in `consumo/` and three in
  `completitud/`, from the same node. ⚠️ §722: in those eight places the judge is now
  `juzgar_historia`, which contains it and adds the handover (below). `juzgar_continuidad` alone
  remains for the anchor's fork (2.12), for the reason section 8 gives.
- ⚠️ **§722 — the HANDOVER: the chain ties the MMR.** The continuity above only looks at keys and
  indices. Two operators in agreement signed themselves one: the genesis of one commits the key
  of the other, and the other starts its node over a COPY of the first one's journal —its lines
  are head digests, public—. The node rotates for real, its MMR contains the first one's, and the
  extension between two heads of two distinct ledgers came out VERDE (measured with
  `tools/banco_historia_comun.sh`, two real nodes untouched). So, in the eight places above
  —here, in the consumption and in the six heads of the completeness—, with distinct keys and
  `actas`, the judge is `juzgar_historia`: the continuity and, after it, the **handover**. The
  `procedencia` of a rotation act carries the `mmrSize` and the peak of the accumulator of heads
  that the incoming key inherits —the node reads them from its journal on rotating, and its first
  head signs them—. Each stretch of a key thus has its handover (2.14): from the `mmrSize` its act
  makes it inherit —and at it, with the peak it inherits; the genesis has no such edge— and below
  the handover of the next act, if the chain carries it. **Each of the two heads falls in the
  handover of its stretch, by its two edges**: it is the judgement the overlap envelope makes with
  a single head, and with one chain there is a single verdict. The rules, in this order, which is
  the order of precedence of the texts (section 5) in the kit and in the second implementation:
  1. the earlier one, the one with the lower embedded index, falls in a stretch of its key up to
     the step: the one its index says, which with a key that returns to the chain is not always
     the last. If it falls in none, OVERLAP, rule 4 for it too;
  2. the handovers of the acts crossed, from that stretch to the later one's, **do not go
     backwards**;
  3. the earlier one, in the handover of its stretch: if not, ROJO below the handover of its act,
     at it with another peak, or at the next one's or above —in this order—;
  4. the later one, in the handover of its own, with the same three edges and the same order.

  If not, ROJO with the rule that falls —`SOLAPAMIENTO: …` in rule 1, `RELEVO: …` in the others—,
  in step `2/3`: before consistency. With that, the head with the greater `mmrSize` is the later
  one: the clock crosses the keys through the chain. The three rotations of the catalogue remain
  VERDE. ⚠️ The later one is judged against the LAST entry of its key, as in the continuity above:
  with a key that returns to the chain, a later one from an earlier stretch of its key falls by
  index (`FueraDeTramo`). It is a false negative of the continuity, prior to the handover, and it
  is declared. ⚠️ The handover **does not prevent** the common history: two operators in agreement
  assemble it with the handover well placed, and the envelope sees what it would see of a rotation
  (2.5). What it does is **tie it down**: the act says where the outgoing key ends, and a head of
  that key above it is evidence (2.14).

### 2.4 The consumption package (§419)

```text
{ "v": 1, "tipo": "consumo", "vieja": {…}, "nueva": {…}, "camino": […],
  "consumo": "0x…", "ausencia": {siblings, isRight}, "presencia": {siblings, isRight} }
```

- A strict superset of the extension one: **the same two heads and the same consistency
  proof**, and that is why the «before» is of THIS history and not of a signed fork. And the same
  optional `actas` (§648), with the same judge. ⚠️ And since §712 the same clock (2.3): without
  it, an old head from ANOTHER branch —the same `mmrSize` and the same peak, without the
  consumption— «proved» that the consumption was published after it, and so did a new one signed
  below the old one. All three cases gave VERDE (measured).
  What it adds are the two paths of the tree of consumptions: `presencia` climbs the digest of
  the consumption up to the `consRoot` of the **new** one, `ausencia` climbs the **empty leaf** up
  to the **old** one's. Both carry `consRoot` (**v4, v5 or v6**): a v2 or v3 does not carry it.
- ⚠️ **The position does not travel: it is DERIVED.** The command computes
  `posicion_de_consumo(consumo)` and **cross-checks** its bits against the `isRight` received in
  both paths. Without that cross-check, `ausencia` would only prove that *some* position is empty
  under that root, and whoever packages would choose any of the 2^63 free ones to «prove» the
  absence of anything. The presence half is solid without the cross-check: siblings that climb
  to a real root are not fabricated. The rules live in `crates/zk-ssl-verify/src/consumos.rs`
  (`spec/NUCLEO.md`).
### 2.5 The conflict package (§430)

```text
{ "v": 1, "tipo": "conflicto", "consumo": "0x…",
  "libros": [ { "cabeza": {…}, "presencia": {siblings, isRight} },
              { "cabeza": {…}, "presencia": {siblings, isRight} } ] }
```

- **It is not a superset of the consumption one: it is another claim.** That one proves that a
  consumption was published BETWEEN two heads of ONE signer, and for that it needs the MMR
  consistency and the `ausencia`. This one proves that the SAME consumption is under the
  `consRoot` of **two heads of two DISTINCT signers**, and between two operators there is no
  common history to extend: it carries neither `camino` nor `ausencia`, and the two heads go in a
  **`libros` list of exactly two**.
- ⚠️ **The list does not order them, because the proof does not order them.** `vieja`/`nueva` or
  `a`/`b` would invent a precedence that does not exist here: the two ledgers are symmetric and
  neither of the two is «the first». A size other than two is rejected by name.
- **The rule on keys goes the other way round from the other forms.** Where the extension and
  the consumption demand the SAME `publicKey` —continuity is of one signer—, here they are
  required to be DISTINCT: two heads of the same operator are not a conflict between ledgers, and
  accepting them would pass off as a conflict what is the history of a single one.
- ⚠️ **§648 — the conflict does not read `actas`, and a rotation fools it.** Two keys of ONE
  operator that rotated (RFC-0015) are two distinct `publicKey`s, and a consumption under a head
  of each is exhibited here as a conflict between two ledgers that is the history of one. The
  envelope cannot demand the chain —whoever assembles it is the denouncer, and does not bring
  it—, so whoever judges it asks the two operators for `zkssl_keyActs`: if a chain joins the two
  keys, there are not two ledgers. **The refutation has an envelope** (§686): the extension one
  between the two heads, with the chain (2.3). If it verifies, the two are ONE history and the
  consumption under both is the same one, published once. ⚠️ The chain alone is NOT enough: two
  operators in agreement can sign themselves a chain that joins their keys. ⚠️ **§722: and they
  can also sign themselves the common history.** Here it was said they could not —the MMR
  consistency between the two heads—, without measuring it, and it is false. The leaves of the
  MMR are head digests, public: the second one starts its node over a copy of the first one's
  journal, whose genesis commits its key, and its MMR contains the first one's. Measured with
  `tools/banco_historia_comun.sh`: two real nodes untouched, two ledgers —two states, two trees of
  consumptions— that accept the same consumption; the conflict comes out VERDE, and so does the
  refutation, before and after the fix. The envelope sees what it would see of a rotation, and the
  history of heads does not tie the STATE: nothing in it distinguishes a real handover from a new
  ledger that declares itself heir.
  **The refutation, therefore, does not close the conflict: it moves it.** Since the fix the
  extension demands the handover (2.3), and the act of the refutation fixes where the key that
  left ends. If the first ledger kept signing with it —and in a genuine conflict it does: every
  heartbeat is a head—, any head of its with an `mmrSize` from the handover on, with the chain of
  the refutation, is an OVERLAP (2.14), DETECTION with exit 0. Before the fix that head was inside
  its stretch and gave nothing away; only the fork of the anchor's mode 5 (2.12) gave it away, and
  it asked for the two heads at the same `mmrSize`, one from each node. Whoever judges the
  refutation asks, then, for the most recent head of the key that left. And whoever denounces
  chooses the chain: any chain the operator signed serves him, and the one with the lowest
  handover gives away the most.

  ⚠️ **What remains, with no evidence that any envelope judges:**
  1. **Alternating the ledgers, not shutting them down.** The node accepts as a rotation any key
     whose fingerprint is the next one of the act in force, including one that returns. With
     untouched nodes, A, B, A, B…, each over a copy of the other's journal and with its own
     ledger, they sign a linear history of heads and a valid chain. Each head falls in its stretch
     and in its handover, and both ledgers stay alive (measured in the core: a chain of four acts
     that holds, and one head per turn that falls in its stretch and in its handover, so the
     overlap envelope gives none of them away). There is no need to shut down the first ledger:
     it is enough that each key signs only in its turns.
  2. **Padding.** The second one seeds its MMR with subtrees of equal leaves, whose peak costs one
     hash per level, and places the handover above everything the first one will sign. That the
     `mmrSize` ends up far above the leaves its key spent does not give it away if the counter
     jumps too: in the bench itself B jumps with `--desde-minimo` to leaf 1000001 and signs with
     `mmrSize` 7, so the padding can raise the handover up to near its index without a rule
     «index ≥ `mmrSize`» seeing it.
  3. **Signing the handover again.** One same transition admits more than one act: the incoming
     key can sign another, with another `procedencia`, at a leaf its jump burned. The generator of
     the vectors signs four distinct acts of the same transition of B, at leaves 500001 to 500004,
     and all four hold in `verificar_cadena`. «The act fixes where the key ends» holds per chain:
     two valid acts of the same key after the same previous one are an equivocation of the chain,
     and today no envelope gives it away (pending: an envelope for that equivocation).
  ⚠️ **§712:** the refutation is an extension, and inherits its clock. With the same `mmrSize`,
  consistency is the identity and sharing the peak was enough: two operators that seed their MMR
  with the same journal share it at that size, and their two heads are not one history, they are
  two branches of the same past. Since §712, two distinct heads with the same `mmrSize` refute
  nothing: `extension: mismo mmrSize y digest distinto`. With distinct keys the embedded index is
  not compared (2.3).
- ⚠️ **What this proves and what it does not.** It proves that two ledgers accepted the same
  consumption: that is **detection**, and it comes afterwards. **It prevents nothing**, and it
  does not say that the unit consumed is the same in both: that the identifier means the same on
  both sides is governance (RFC-0006, D-4), not cryptography. That is why the `tipo` is not called
  «double-spend».

### 2.6 The rejection package (§455)

```text
{ "v": 1, "tipo": "rechazo", "data": {"causa": "…", "campos": {…}, "seq": "0x…"},
  "cabeza": {…}, "parametros": {…} }                         (una causa de los parámetros)
{ "v": 1, "tipo": "rechazo", "data": {…}, "cabeza": {…},
  "presencia": {siblings, isRight} }                          (una causa del consumo)
{ "v": 1, "tipo": "rechazo", "data": {…}, "cabeza": {…}, "recibo": {…} }   (StaleState)
{ "v": 1, "tipo": "rechazo", "data": {…}, "cabeza": {…},
  "lote": [ {…}, {…} ] }                                      (un duplicado de lote)
{ "v": 1, "tipo": "rechazo", "data": {…}, "cabeza": {…},
  "congelados": {index, leaf, camino} }                       (AccountFrozen)
{ "v": 1, "tipo": "rechazo", "data": {…}, "cabeza": {…},
  "cuenta": {index, leaf, camino} }                           (AccountNotFound)
{ "v": 1, "tipo": "rechazo", "data": {…}, "cabeza": {…},
  "banda": {s, publicId, requested, prueba} }                 (InsufficientBalance)
{ "v": 1, "tipo": "rechazo", "data": {…}, "cabeza": {…}, "parametros": {…},
  "peticion": {index, amount} }                               (SupplyCapExceeded)
```

- **It proves the CAUSE of a rejection, not the rejection.** `data` is the `data` object the node
  put in its refusal (`spec/RPC.md`, §454), as is; `cabeza`, a response of
  `zkssl_signedEpochHead`; `parametros`, the one of `zkssl_params`; `presencia`, the `camino` of
  `zkssl_consumoPath`; `recibo`, the `publicInputs` the holder sent; `lote`, the `ops` of
  `zkssl_applyMany`; `congelados`, the response of `zkssl_frozenPath` (§458); `peticion`, the
  `params` of the rejected issuance, as the applicant sent them (§460); `cuenta`, the path of the
  empty leaf that the node itself writes with `zk-ssl-node --prueba-rechazo` (§474), and `banda`,
  the proof that the balance does not reach the amount, which the same mode writes without the
  balance (§478). That the node rejected —and when— this envelope does not prove. What it proves
  is that the rule the node named **holds over the state that a signed head commits**; if it does
  not hold, the ROJO names why, and the envelope is then the proof that the rule was a disguise.
- **Which head serves depends on the cause, and it is demanded with the `seq`.** For what only
  grows —`nextIndex`, the published consumptions— a head **prior** to the rejection serves, or
  the same one: what was already in it was still there when judging. For what has no setter —the
  regulatory limit— any head of the ledger serves. And for what is judged over an **instantaneous**
  state —the roots of a `seq` (`StaleState`), the batch against that log (the duplicates), the
  frozen tree, which comes and goes (`AccountFrozen`), the supply, which rises with every issuance
  and falls with every burn (`SupplyCapExceeded`), and the balance, which falls with every send
  (`InsufficientBalance`)— the head has to be **the same one** as the rejection's. Mutability is
  measured in the code (§455, §456, §459, §460, §478).
- The causes this command proves (RFC-0007 E3 —E3a, E3b and §460— and the two causes of E5 that
  are built: `AccountNotFound` since §476 and `InsufficientBalance` since §478):

| cause | material | what it checks | head |
|---|---|---|---|
| `OverRegulatoryLimit` | `parametros` | the seven recompose the `paramsDigest`; `limit` is the committed one and `requested` exceeds it | v5 or v6, any of the ledger |
| `AccountLimitReached` | `parametros` | they recompose; `limit` is `maxAccounts` and `nextIndex` reaches it | v5 or v6, prior or the same |
| `ConsumoRepetido` | `presencia` | the position is DERIVED from the consumption, and the consumption is under `consRoot` | v4, v5 or v6, prior or the same |
| `ConsumoColision` | `presencia` | the occupant is at the position DERIVED from the consumption, and it is not it | v4, v5 or v6, prior or the same |
| `StaleState` | `recibo` | one of the three roots the receipt declared (`rootOld`, `pendingRootOld`, `frozenRoot`) is not the head's | v3, v4, v5 or v6, the same (§456) |
| `WrongRegulatoryLimit` | `parametros` | the seven recompose; `expected` is the committed one and `declared` is not | v5 or v6, any of the ledger (§456) |
| `DuplicateAccountInBatch` | `lote` | the first clash of the batch, with the rule of `apply_many`, is a repeated account | v3, v4, v5 or v6, the same (§456) |
| `DuplicatePendingInBatch` | `lote` | the first clash of the batch is a repeated position | v3, v4, v5 or v6, the same (§456) |
| `AccountFrozen` | `congelados` | the path is the one of the account `data` names (cross-check against its bits), it measures the 32 levels the core fixes and climbs to the `frozenRoot`; the leaf is not the empty one | v3, v4, v5 or v6, the same (§459) |
| `AccountNotFound` | `cuenta` | the path is the one of the account `data` names (cross-check against its bits), it measures the 32 levels the core fixes and climbs to the `accountsRoot`; the leaf **is** the empty one | v3, v4, v5 or v6, the same (§476) |
| `InsufficientBalance` | `banda` | `data` does not carry `available`; the `requested` of `data` is the one of `banda` and is not zero; the `prueba` verifies, with the house options and without the balance, that the balance of account `publicId` is in `[0, requested - 1]` under the `accountsRoot` (`zk_ssl_air::banda`). Since §723, the proof is of the **v2** band (the three range segments in 62 bits); a v1 one is valid only with `requested - 1 >= 2^32 - 2`, or if it is the already published one the judge recognizes by its fingerprint and its statement (`zk_ssl_air::banda::PUBLICADAS_V1`), and otherwise exit 3 | v3, v4, v5 or v6, the same (§478) |
| `SupplyCapExceeded` | `parametros`, `peticion` | the seven recompose; `cap` is the committed `maxSupply`; `wouldBe` is the `totalSupply` of the head plus the `amount` of the request (the layer's saturating sum) and it passes the cap | v5 or v6, the same (§460) |

- ⚠️ §776: `StaleState` and the two batch duplicates said «v5» since §456, and the code never
  demanded it: their arms do not ask the version and accept what `cabeza_v3_verificada` does (v3
  to v6). It is not the §558 staleness of the other rows.
- Any other cause is rejected by name. `SupplyCapExceeded` was added by §460: over the wire only
  the sandbox faucet (`dev_fund`) produces it, and the amount travels in `peticion` —the
  applicant's word, not the node's— so that an invented `wouldBe` does not pass.
  `PendingTreeExhausted` is declared without portable proof: position reservations do not go
  under the signature (RFC-0007, correction of §460). `AccountFrozen` was added by §459, with the
  path the wire serves to the holder (§458); `AccountNotFound` moved to E5 (RFC-0007, correction
  of §459), and the rest of table D-D is E4 and E5. `StaleState`, `WrongRegulatoryLimit` and the
  batch duplicates were added by §456, gathering a real receipt through the proxy of a bench (its
  bench does not live in the tree, and this is declared).
  ⚠️ §815: `ProofFailed` and `VerificationFailed` have no arm here either, and are still rejected
  by name: D-I of RFC-0007 declares them without portable proof —that the proof did not verify is
  the node's word until the command re-verifies it (RFC-0019 E3, the 0.5 train)—, and since kit
  0.4.3 their receipt is resolved as `declarada` (2.11, verdict 4), in the fourth state. Until
  §815 neither this nor that accepted them, and a receipt rejected for them had no resolution to
  exhibit. The ROJO of this envelope with those causes, «la causa … no la prueba este mando» [the
  cause … this command does not prove], says that the envelope does not resolve the receipt, and
  it is not an accusation against the operator: the resolution is the `declarada` one.
- `InsufficientBalance` was added by §478, and it is the only cause of this envelope that
  verifies a STARK. The `data` of the envelope does not carry the balance —the command rejects one
  that carries `available`—, and since §538 (RFC-0009 E3b-2) **the band proof does not publish it
  either**: between §521 and §538 it opened its rows in the clear with the balance and the
  `leaf_salt` inside (measured in §521; the `saldo-insuficiente.json` of 0.3, preserved under
  `spec/vectors/0.3/rechazo/`, carries its own; the one of 0.4, none). The `-32000` of the wire
  is also sent to whoever made the request since §454 (RFC-0007, correction of §479). An amount
  above the field's ceiling (`MAX_VALOR`, 2^62 − 1) produces the cause and not its proof: the
  producer refuses, naming it.

### 2.7 The age package (§465)

```text
{ "v": 1, "tipo": "edad", "cabeza": {…},
  "enunciado": {"t": "0x…", "k": "0x…", "emisor": "0x…"},
  "subraices": {"pendientes": "0x…", "meta": "0x…"}, "prueba": "0x…" }
```

- **It proves the DISTRIBUTION of ages of what is in flight** (RFC-0007 D-E): over the positions
  `0..nextPending` of a **v5 or v6** head, the live ones with age `seq - nacido >= t` —and, if the
  envelope names `emisor`, only its own— are **at most `k`**. The empty box is `k = 0`; the
  per-account cap, `t = 0`; concentration, a named `emisor`. Without `emisor` all count: its
  absence says so, because index 0 is an account and cannot be the sentinel. The `emisor` is an
  account index, below `2^32` (§718, canonicity): the judge rejects it before reading the proof,
  because the field does not distinguish `emisor` from `emisor + p`.
- `cabeza` is a response of `zkssl_signedEpochHead` as is; `subraices` and `prueba` are produced
  by whoever proves, and `prueba` is the bytes of the STARK proof in `0x` + hex. `seq`,
  `nextPending`, `pendingRoot` and `pmetaRoot` come **only from the head**: the envelope does not
  repeat them. And `m` —the subtree `[0, 2^m)` the proof covers— is **derived** by the command
  from `nextPending`, the smallest that covers it and at least 3: a statement has a single form.
- **`m` has a cap** (§717): the judge rejects `m` above `zk_ssl_air::M_MAX`, 13, **before** reading
  the proof, with `m = {} fuera de {M_MIN}..={M_MAX}` (⚠️ §914: `3..=` until the cut; the floor is
  `M_MIN`, 4, with T = 128). The judge pays a cost linear in `2^m` before the first check against
  the proof, and the command does not fix the operator's key: without the cap, a package with a head
  signed with its own key and a proof that only has the form chose that cost. 13 is the largest `m`
  whose proof, with the hidden production trace, fits in the two bounds of the gate of RFC-0007 (one
  heartbeat and half the RAM of the reference machine): `m` = 14 does not fit (§717). The house
  prover and the layer's producer apply the same cap, the latter before reading the ledger. Two
  vectors of `spec/vectors/edad/` fix the value (§721): `neg-m-13-fabricada.json` reaches the proof
  and `neg-m-14-sobre-tope.json` falls by the cap.
- The judge is `zk_ssl_air::verificar_contra_cabeza` (`crates/zk-ssl-air`), which the kit
  compiles **without the prover**: it climbs the two subroots to 32 levels, compares them with the
  signed roots and verifies the proof with the house's `proof_options()` and only with them.
- **What it does NOT prove:** nothing about amounts —the operator does not keep the opening of
  the commitment (RFC-0007, correction of §464)—, nothing about what never entered the tree (H5b)
  and nothing about any head other than the one that signs the roots. Its end-to-end positive,
  with a real head from a node, and its catalogue of vectors belong to E4b-3: today the command
  carries the negatives that fall before the signature, and the judge its witnesses with real
  proofs (§465). And, since §717, a ledger whose `nextPending` passes `2^M_MAX` has no age proof,
  nor will it have one. `nextPending` is the high-water mark of the positions used and does not
  fall: `allocate_pending` reuses the lowest free slot, and only `commit_send` and
  `apply_mint_pending_delegated` raise it (`crates/zk-ssl/src/two_phase.rs`).

### 2.8 The pending claim package (§495)

```text
{ "v": 1, "tipo": "cobro_pendiente", "cabeza": {…},
  "enunciado": {"receptor": "0x…", "nacido": "0x…", "inferior": "0x…"},
  "prueba": "0x…" }
```

- **It proves that a pending item EXISTS in someone's name, for at least an amount** (RFC-0008
  D-A, D-B): under the `pendingRoot` of a **v5 or v6** head there is a v2 leaf that opens to
  `receptor` and to an amount in `[inferior, MAX_VALOR]`, and at the SAME position of the meta
  tree, under `pmetaRoot`, is the meta with that `nacido`. It is a statement of STATE (D-G): it
  does not say who produced the proof, nor that the pending item will be claimed, nor when it
  expires.
- `cabeza` is a response of `zkssl_signedEpochHead` as is; `prueba` is the bytes of the STARK
  proof in `0x` + hex. `seq`, `pendingRoot` and `pmetaRoot` come **only from the head**: the
  envelope does not repeat them (D-J). And the **upper bound does not travel**: the judge fixes it
  at the field's ceiling, `MAX_VALOR`, because a statement has a single form. Since §723, v2 of
  the claim's AIR (the three range segments in 62 bits, like the band): it is the one the front
  end produces since then. A v1 proof already issued still verifies, because with the upper bound
  `MAX_VALOR` v1 lets no amount outside the band through.
- The judge is `zk_ssl_air::cobro_pendiente::verificar_contra_cabeza` (`crates/zk-ssl-air`),
  which the kit compiles **without the prover** (D-K): it demands `nacido < seq` —a meta born
  after the head that signs it is a head that lies, and it stops with its name—, composes the
  statement with the two signed roots and the ceiling, and verifies the proof with the house's
  `proof_options()` and only with them. It is the SAME producer with which the layer verifies
  what it produces.
- **What it does NOT prove:** nothing about who paid nor about the envelope `X`, which is witness
  (D-I); nothing about the exact amount above `inferior`; and nothing about any head other than
  the one that signs the two roots. **Being witness is not being hidden, and since §538 it is
  hidden in what the RFC-0009 suite measures**: between §521 and §538 the proof opened its rows in
  the clear and published the salt, the `X` and the exact amount (measured in §521); since §538
  (E3b-2) it publishes no literal. Its catalogue of vectors is `spec/vectors/pendiente/` (§499,
  RFC-0008 E4 on the claim side) and the bench that reproduces it live, `tools/banco_pendiente.sh`
  (§498); the front end that writes the envelope is `zk-ssl-cli prueba-cobro` (§497, D-M): the
  command carries the negatives that fall before the signature, the judge its witnesses with a
  real proof in `stark-experiment`, the layer's producer and the node's test link it against a
  real heartbeat, the front end gathers the producer's five inputs without opening a ledger, and
  the manifest pins what each envelope says.

### 2.9 The payment-in-flight package (§506)

```text
{ "v": 1, "tipo": "pago_en_curso", "cabeza": {…},
  "enunciado": {"receptor": "0x…", "importe": "0x…", "t": "0x…", "nacido": "0x…"},
  "prueba": "0x…" }
```

- **It proves that someone paid an EXACT amount to a receiver and cannot revert it before `t`**
  (RFC-0008 D-AD, D-AF): under the `pendingRoot` of a **v5 or v6** head there is a v2 leaf that
  opens to `receptor` and to `importe`, with the envelope `X = M(refund_id, [delta, 0, 0, 0])`
  composed INSIDE the circuit, and at the same position of the meta tree, under `pmetaRoot`, the
  meta with that `nacido`; and `t - nacido <= delta`. It is the other half of 2.8: where the
  claimant states «I am owed at least `inferior`», the payer states «I paid exactly `importe` and
  I am bound until `t`».
- **The term does not go in the statement, but it does go in the proof.** What is proven is
  `delta >= t - nacido`, never `delta`; and between §521 and §538 the proof opened its rows in the
  clear with `delta` inside (measured in §521): whoever read the bytes of the envelope read the
  `delta` that this line said did not travel. Since §538 (RFC-0009 E3b-2) it does not come out
  literally.
- `cabeza` is a response of `zkssl_signedEpochHead` as is; `seq`, `pendingRoot` and `pmetaRoot`
  come **only from the head** (D-J), and the judge demands `nacido < seq` before touching the
  proof, and that `importe`, `t` and `nacido` do not pass the ceiling as they arrive, without
  reducing (§718, canonicity: `el importe {} pasa del techo {MAX_VALOR}`, `T {} o nacido {} pasan
  del techo {MAX_VALOR}`). The judge is `zk_ssl_air::pago_en_curso::verificar_contra_cabeza`
  (§503), the SAME with which the layer re-verifies what it produces, and the kit compiles it
  **without the prover**.
- **What it does NOT prove:** nothing about who will claim it, nothing about the `sal`, the
  `refund_id` nor the `emisor` —they are witness—, and nothing about any head other than the one
  that signs the two roots. **Being witness is not being hidden, and since §538 it is hidden in
  what the RFC-0009 suite measures**: between §521 and §538 the proof opened its rows in the clear
  and published the salt, the `delta` and the `refund_id` (measured in §521); since §538 (E3b-2)
  it publishes no literal. Its producer is `prueba_de_pago_en_curso` in the layer (§504) with the
  payer's opening and the snapshot the node serves him with his credential and `receiverId`
  (§505, D-AE); **the front end is `zk-ssl-cli prueba-pago`** (§507, D-AI), which gathers the
  producer's five inputs without opening a ledger and rejects a refund that does not recompose
  the `x` of its notice before asking the node for anything; **the bench is
  `tools/banco_pago.sh`** (§508), which reproduces it live against a real node and from whose
  captures the vectors came; **the catalogue is `spec/vectors/pago/`** (§509), the seventh family
  of the artefact.
- ⚠️ **The node has to be from §505 or later**: an earlier one ignores `receiverId` —the method
  never rejected extra fields— and serves nothing instead of an error, so the payer could not
  gather the snapshot. What says whether a node knows what it is talking about is its
  `spec/openrpc.json`, not the protocol version, which does not rise.

### 2.10 The pledge package (§520)

```text
{ "v": 1, "tipo": "prenda", "cabeza": {…},
  "enunciado": {"receptor": "0x…", "marca": "0x…"},
  "prueba": "0x…" }
```

- **It proves that under the `pendingRoot` of a v5 or v6 head there is a pending item that only
  whoever holds the key of `receptor` can claim, and that the mark of that leaf is `marca`**
  (RFC-0008 D-AV, D-AW): inside the circuit the v2 leaf is recomposed with
  `receptor = derive_public_id_wide(clave)`, climbed along the path up to the root, and it is
  proven that `marca = H(DOMINIO_PRENDA, C2)` with `C2` as witness. Publishing `C2` would make
  every envelope of one same leaf linkable; that is why the mark travels, which is what the tree
  of consumptions carries.
- **Of AUTHORIZATION, not of state**, and that is what separates it from 2.8 and 2.9: those are
  produced by anyone who has the opening, and this one only by whoever has the key (D-AV). The
  key does not travel nor appear in the envelope: it enters the trace and leaves with it.
- **It does not carry the meta** (D-AY): its head contributes ONE root, `pendingRoot`, and the
  statement says neither `nacido` nor `importe`. That is why the judge compares nothing against
  the `seq`, and why the width of the AIR drops from 44 to 42.
- **The head is required to be v5 or v6** (D-BF fixed v5; v6 entered the set in §558), and the
  rejection text says ITS reason and not its siblings': not «the ones that sign pmetaRoot» —which
  here would be false—, but that it is the one the node serves (v5 since §452, v6 since §570) and
  against which `zkssl_pledge` judges. `cabeza_v3_verificada` would admit v3 and v4, but no pledge
  producer can issue against them.
- ⚠️ **A VERDE here is HALF a pledge.** The judge does NOT check that `marca` is under the
  `consRoot` of that same head —it declares it in its own doc, «it is the gate of whoever writes
  in it»— and this binary has no tree to look at. The PAIR is the mark under the signed root PLUS
  this envelope (D-AS); the other half is requested with `zkssl_consumoPath`, and whoever writes
  it DEMANDING the envelope is `zkssl_pledge` (§519). The command prints it in its verdict, not
  only here.
- **What it does NOT prove:** nothing about the amount, the salt, `X` nor `C2` —they are
  witness—, nothing about whether the pending item is still alive, and nothing about any head
  other than the one that signs that root. **Being witness is not being hidden, and since §538 it
  is hidden in what the RFC-0009 suite measures**: between §521 and §538 the proof opened its rows
  in the clear and in them went the pledgor's spending key, 42 times (measured in §521), and
  whoever read the envelope had it; since §538 (E3b-2) it does not come out literally. The judge
  is `zk_ssl_air::prenda::verificar_contra_cabeza` (§516), the SAME with which the layer
  re-verifies what it produces (§518), and the kit compiles it **without the prover**. Its
  producer is `prueba_de_prenda` (§518), with the notice, the key and the path the node serves
  from the last heartbeat; and since §543 its pledgor writes it with `zk-ssl-cli prueba-prenda`
  (D-BC..D-BI): the key enters through the SDK's keystore and the cli does not see it, and with
  `--publicar` the front end also asks `zkssl_pledge` to write the mark under that same heartbeat;
  **the bench is `tools/banco_prenda.sh`** (§545), which reproduces it live against a real node
  —two positives, a live rejection and seven negatives—, and **the catalogue is
  `spec/vectors/prenda/`** (§546), the eighth family of the artefact, COPIED from those captures
  and not re-derived.

### 2.11 The completeness envelope (§573, RFC-0010 E4)

```text
{ "v": 1, "tipo": "completitud",
  "cierre": {…},                                  la cabeza v6 firmada que cierra la era del recibo
  "recepcion": {"rx", "era", "n", "hashPrueba"},  el recibo del cable, tal cual (§571)
  "limiteAnterior": "0x…",                        Q, DECLARADO (D1)
  "camino": {"siblings": […], "isRight": […]},    el de zkssl_recepPath
  "resolucion": {"tipo": "acuse", "cabeza": {…}, "acuse": {"seq", "hashPrueba", "camino"}}
              | {"tipo": "rechazo", "sobre": {…}}      un sobre de la sección 2.6
              | {"tipo": "declarada", "data": {…}}     el `error.data` del rechazo, tal cual
              | {"tipo": "lote",                       el recibo de un zkssl_applyMany (§612)
                 "composicion": [{"hashPrueba", "cuenta", "posicion"}, …],
                 "acuses": [{"cabeza", "acuse"}, …]  | "sobre": {…}  | "data": {…}}
              | {"tipo": "prenda", "sobre": {…},       el recibo de un zkssl_pledge (§613), y el
                                                       sobre de la sección 2.10
                 "consumo": {"cabeza", "camino"}
                 | "respuesta": {…}, "juzgada": {…}  | "rechazo": {…}},
  "vigente": {…} }                                una cabeza del MISMO operador, para la ventana
```

The command checks, in order, that the `cierre` is a v6 head that recomposes and whose signature
verifies, that the `n` of the receipt is the signed one, that `Q < rx <= recepCount`, and that
the leaf `recibo_digest(hashPrueba, era, n)` climbs along a path of `RECEP_DEPTH` levels with the
sides of position `rx − Q − 1` up to the `recepRoot` of the closing: the operator **received** the
operation. Then, the verdict (RFC-0010, D-F and D-G). The window is ALWAYS MEASURED in the unit of
the signature (D2; since §681, RFC-0019 E1): it starts at the **embedded** XMSS index of the
closing, and `S` is the **attested** index —the embedded one plus one— of a head signed by the
same key, with `S − inicio <= n`. Since §648 that head may be of another key of the same
operator, if the envelope carries `actas` that join it to the closing (2.3): the index count is
one (RFC-0015 D-A), and the window is measured the same way. ⚠️ §722: the six heads this envelope
compares with its closing are judged with `juzgar_historia`, with the handover of 2.3, like the
extension: a resolution with a head of another key does not resolve with more freedom than the
refutation of the conflict. Since §771 two completeness vectors carry `actas`,
`rotada-ventana-abierta.json` and `neg-rotada-vigente-cadena-corta.json`, and neither moves: both
say, line by line, what they said without the handover (measured, 81 of 81).
The declared `index` is not part of what is signed (§399) and no longer measures anything; the
`era` of the leaf ties it to its path but does not measure the window:

1. **`acuse`**: the head is inside the window, the acknowledgement is of the SAME proof and the
   head+acknowledgement pair verifies as the position package of section 2.1. VERDE, «resolved as
   an applied transition».
2. **`rechazo`**: the envelope verifies by its own rules, its head is inside the window, and its
   `data.recepcion.hashPrueba` is the receipt's (D3). ⚠️ That binding is the node's WORD: the
   `error` of the wire is not signed. VERDE, «resolved as a rejection with proof». ⚠️ **Since §682
   (RFC-0019 E2), a `StaleState` rejection does not resolve on its own**, neither on the direct
   path nor in the batch: its cause holds, but the roots its `recibo` declares and its `data` are
   not signed, and that they are those of THIS operation is the word of the accused. It comes out
   as the fourth state, «DECLARADA, NO PROBADA» (exit 3). The two vectors that resolved it as
   VERDE are preserved in `spec/vectors/0.4/completitud/` with their manifest of then. ⚠️ §723: an
   `InsufficientBalance` rejection with the v1 band below the bound (section 2.6) does not resolve
   either: it is the fourth state, with the place, and on the direct path, in the batch and in the
   pledge it is said AFTER the checks of the form, so an operation the batch does not have, or a
   mark that is not the envelope's, are still ROJO.
3. **Without a resolution**, with a `vigente` head outside the window: **NAMED ROJO** [a named
   red], «NO RESUELTA EN LA VENTANA». It is the milestone's product: the promise signed by the
   accused, the arithmetic window over two signed heads, and the burden of exhibiting the
   resolution on whoever has it. It is not a cryptographic proof of absence (D-F). With the
   `vigente` still inside the window, the envelope is premature and is rejected by name.
4. **`declarada`**, with one of the causes RFC-0007 left without portable proof
   (`CustodianSetExhausted`, `PendingTreeExhausted`, `NotTheIssuer`, `NotTheAccountHolder`) and
   tied to the receipt by its `recepcion`: the **fourth state**, «resolution declared, not
   proven», which comes out with its own code (D4). A cause that does have proof is exhibited, not
   declared.
   ⚠️ §815 (kit 0.4.3): the causes are SIX: the four plus `ProofFailed` and `VerificationFailed`,
   which D-I of RFC-0007 declares without portable proof and D-G of RFC-0010 sends here. The list
   of four came from the rows of table D-D of RFC-0007 marked «declared without proof» and from
   the correction §460 wrote after it (`PendingTreeExhausted`), not from its D-I; and with it a
   receipt the node rejected for a proof that did not verify —that of a third party without an
   account who sends garbage bytes with the public roots, for example— had no resolution: the
   rejection envelope does not prove it (2.6) and the declared one came out ROJO, so once the
   window had passed the envelope named the honest operator. The vectors
   `completitud/declarada-verification-failed.json`, `declarada-proof-failed.json` and
   `lote-declarada-verification-failed.json` fix it. Two things exit 3 does not say.
   **`ProofFailed` is produced today by no arm with a receipt**: the layer only gives it when
   PROVING, and `applySend`, `applyClaim`, `applyMany` and `zkssl_pledge` only verify (read in the
   code, not measured). A declared one with `ProofFailed` is, therefore, always the operator's
   word, with no honest case behind it: it is in the list because D-I declares it, and bounding
   the declared one to the causes that fit each path is entry 129 (b) of `BACKLOG.md`. And **a
   rejection envelope with `VerificationFailed` or `ProofFailed` is still ROJO**, «la causa … no la
   prueba este mando» (2.6): that ROJO says that THAT resolution does not resolve, and accuses no
   one —anyone fabricates such an envelope with nothing to sustain it—. The resolution the honest
   operator exhibits for those causes is the `declarada` one.
5. **`lote`** (§612, RFC-0014 E4a, D-B): the receipt of a `zkssl_applyMany`, whose `hashPrueba`
   is the fingerprint of its composition (§611). The command recomposes `hash_del_lote` with the
   `composicion` -the one the aggregator forwards to each holder (D-C), in the batch's order- and,
   if it is not the receipt's, does not go on. Then, ONE of three:
   - **`acuses`**, one per operation and in their order, each resolved as in verdict 1 and of ITS
     proof: VERDE, «resolved as an applied BATCH». The batch is applied whole or not applied, so
     one missing is not resolving.
   - **`sobre`**, the one of section 2.6 for the operation the node names in `data.operacion`,
     resolved as in verdict 2: VERDE, «resolved as a BATCH rejected with proof». Its companions are
     resolved by it: the batch is the unit. ⚠️ That the rejection is of THAT operation is the
     node's word (D3), like the binding to the receipt.
   - **`data`**, the `error.data` as is. With `DuplicateAccountInBatch` or
     `DuplicatePendingInBatch` the command REPEATS the judgement with the composition alone -the
     named operation carries the account or the position of `campos`, and an earlier one does
     too-: if it sustains it, VERDE on the spot and without a head, «resolved as a BATCH rejected
     by its FORM»; if not, **NAMED ROJO, «RECHAZO SIN FUNDAMENTO»** [rejection without grounds]:
     where the verifier can repeat the operator's judgement, it says so (RFC-0014, decision 3).
     ⚠️ The node's «no» is its word in its `data` (D3): if someone invented it, the operator
     refutes it by exhibiting the true resolution. With one of the causes without portable proof,
     the fourth state, as in 4.
6. **`prenda`** (§613, RFC-0014 E4b, D-E): the receipt of a `zkssl_pledge` whose proof reached
   the judge, with `hashPrueba` the digest of that proof (§116). The `sobre` of section 2.10 is
   tied to it by THAT -its proof has to have that digest- and, if not, it does not go on. Then,
   ONE of three:
   - **`consumo`**, `{cabeza, camino}`: the envelope verifies and its `marca` is under the
     `consRoot` of a head of the same operator inside the window, along the path of
     `zkssl_consumoPath` and with the sides of the position the mark derives. It is the whole PAIR
     (D-AS): VERDE, «resolved as an accepted PLEDGE».
   - **`respuesta`**, the node's refusal as is -`accepted: false`, tied to the receipt by its
     `recepcion` and WITHOUT `data`-, and **`juzgada`**, the head against which the node judged:
     the last one signed on receiving, of index `era − 1` (§567), of the same operator and v5 or
     later. The command REPEATS the judgement with the same judge and the `pendingRoot` of that
     head: if the envelope does not verify, VERDE, «resolved as a PLEDGE rejected with proof», and
     anyone checks it without the node; if it verifies, **NAMED ROJO, «RECHAZO SIN FUNDAMENTO»**
     (RFC-0014, decision 3): an honest operator cannot produce it, because the receipt only exists
     if the `seq` matched and then it judged that head with that judge. ⚠️ The refusal is its word
     (D3): if someone invented it, the operator refutes it with the PAIR.
   - **`rechazo`**, the envelope of section 2.6 for the cause with which the layer rejected the
     mark (`apply_consumo`, e.g. `ConsumoColision`): resolved as verdict 2 -since §613 the `data`
     of that refusal carries its `recepcion`- and with the rejected consumption equal to the
     envelope's mark. VERDE, «resolved as a PLEDGE rejected by the layer».

**The bench is `tools/banco_completitud.sh`** (§574), which reproduces it live against a real
node that signs —with `--largo`, until the window EXPIRES—, and **the catalogue is
`spec/vectors/completitud/`** (§574), the ninth family of the artefact, COPIED from one of its
runs: section 9. Forms 5 and 6 have their own bench, `tools/banco_recibo_agregado.sh` (§614), and
their vectors in the same family.

### 2.12 The anchor envelope (§592, RFC-0012 E3)

```text
{ "v": 1, "tipo": "ancla",
  "cabeza": {…},                        la cabeza firmada (payload de zkssl_signedEpochHead,
                                        available:true, v3 a v6: la pareja del MMR viaja desde ellas)
  "ancla": { "v": 1, "clave": "0x…",    OPCIONAL: el ancla publicada en el medio, tal cual --
             "indice": "0x…",           la huella de la clave, el indice XMSS EMBEBIDO, el digest
             "epochDigest": "0x…",      firmado y la pareja del MMR (RFC-0012, D-A)
             "mmrRoot": "0x…", "mmrSize": "0x…" },
  "camino": ["0x…", …],                 OPCIONAL: la consistencia MMR del lote anclado a la cabeza
                                        (RFC 6962), la lista plana del sobre de extensión
  "contraria": {…},                     OPCIONAL: una segunda cabeza firmada -- la vista dividida
  "rama": {…},                          OPCIONAL (§714): otra cabeza firmada del mismo operador con
                                        el mismo mmrSize -- la bifurcacion
  "actas": [ … ] }                      OPCIONAL, solo con rama: la cadena de zkssl_keyActs (2.3)
```

The order: first the FORM —`contraria` and `ancla` do not coexist, and a `camino` without
`ancla` has nothing to extend—; ⚠️ it is decided by the PRESENCE of the key, also with `null`: an
`"ancla": null` is an anchor (§714: the second implementation took it as absent and gave VERDE
where the kit gave ROJO). Then the `cabeza`, WHOLE —it recomposes its digest, its signature
verifies, and the index that counts is the one **EMBEDDED** in the signature (§399; RFC-0012,
D-C)—. With that, four modes, and since §714, five:

1. **The head alone**: the command DERIVES the anchor and its fingerprint
   (`ancla_digest(huella_de_clave(publicKey), indice_embebido, epochDigest, mmrRoot, mmrSize)`,
   `spec/NUCLEO.md` §6) and prints them. It is the producer of B10.6: what is printed is what is
   published in the medium. The checker compares the fingerprint with the medium **himself**:
   this binary has no network and says so in its output. VERDE.
2. **With `ancla` and without `camino`**: the anchor IS this head. The five fields, equal, each
   with its named rejection; the `clave` against the fingerprint of the `publicKey`, the `indice`
   against the embedded one. VERDE.
3. **With `ancla` and `camino`**: the anchor is EARLIER. Same key, `indice` strictly earlier
   than the embedded one, and the head's peak EXTENDS the anchored batch —`mmrRoot`/`mmrSize` of
   the anchor against the head's, the consistency judge of §291—. The genesis anchor (`mmrSize`
   0) has no history to extend and is rejected by name. ⚠️ After consistency, the clock of the
   extension (2.3), with the anchor as the old one: with the **same `mmrSize`** consistency is
   the identity, and the anchor of one branch against the head of the other —the same peak,
   another digest— came out VERDE (measured in the kit and in the second implementation from
   before §712). Since §712 it is ROJO `extension: mismo mmrSize y digest distinto`, the cause of
   the extension, and the clock's line is printed under the `2/3`. VERDE: the anchored history is
   a prefix.
4. **With `contraria`**: the SPLIT VIEW. The contrary one is verified WHOLE like the head; same
   key, same EMBEDDED index, distinct contents —two preambles under a single-use index, which
   only whoever holds the key can produce (§248)—. It comes out **DETECTION**, with exit 0, the
   mould of the conflict (2.5): the envelope that exhibits it does not fail — it gives it away.
   Two heads with DISTINCT embedded indices split no view and are rejected by name.
5. ⚠️ **§714 · With `rama`: the FORK.** Two signed heads of the SAME operator with the SAME
   `mmrSize` and distinct digests: two heads at the same place in its history. An honest node
   with its journal does not repeat an `mmrSize` —the MMR leaf enters where the journal is
   written, and on starting the journal seeds the MMR—, so the pair only comes from a fork or
   from a restart without the journal, or with one that lost lines: a power cut without `fsync`,
   or an `anotar` of the journal that failed —the node warns of it and keeps serving (§272), and
   the head is left without its line—. Those two are declared gaps of the honest node, not
   closed. DETECTING IS NOT DISTINGUISHING: the envelope does not say which of the two, and there
   is no need. It comes out **DETECTION**, with exit 0, like mode 4. The order: the `rama` does
   not coexist with `ancla` nor with `contraria`, and that rule goes after the two form rules
   above; the `rama` is an object, it carries the MMR pair (v3 to v6) —its `formatVersion` is read
   before its `available`, like the head's— and it is verified WHOLE, like the head —two real
   signatures or there is no denunciation—; the operator is the same —the same `publicKey`, or
   distinct keys that the `actas` join, with the judge of 2.3, `juzgar_continuidad`: two
   operators at the same size fork nothing; since §722, without the handover, for the reason of
   section 8—; the `mmrSize` is the same, with any peak —with the same one, the two branches
   diverge at that head; with another, earlier—, and the genesis (`mmrSize` 0) counts too; and
   the digests differ —the same digest is the same head, even if two leaves sign it—. Until §714
   no envelope GAVE AWAY the fork: the extension one rejects it with its cause (§712), and a
   rejection is not a denunciation; mode 4 only saw it if the leaf was repeated as well. With the
   same leaf the pair is both things, and the `3/3` says so.

⚠️ What this envelope does NOT say: that the anchor was PUBLISHED, nor since when — that belongs
to the chosen medium, and the trust in the medium is displaced and declared (RFC-0012, D-H). The
envelope verifies the cryptography; the external order is given by the medium.

**The bench is `tools/banco_ancla.sh`** (§593), which reproduces it live against a real node
that signs —the split view included, reproducing the real attack: the same seed with a fresh
index counter and another ledger—, and **the catalogue is `spec/vectors/ancla/`** (§593), the
tenth family of the artefact, COPIED from one of its runs: section 9. **The split view one is
also assembled by the WITNESS** (§688): `zk-ssl-cli witness --auditar DIARIO --sobres DIR` writes
the mode 4 envelope with the two heads its journal kept, when they are of the same key and the
same embedded index —since §712 the witness and its auditor count by that index, and not by the
declared `index`: with the declared one, two signatures of leaf 1 declared 0x2 and 0x3 passed
clean—; `tools/banco_mentiroso_vista.sh` demonstrates it with a live witness before a node that
comes back with its counter restored. ⚠️ And the witness applies the clock of 2.3 to every head
that arrives (§712): an `mmrSize` already seen with another digest
—with the same peak always; with another, if the node is not behind the kept pair— and a head
later than the kept one, of the same key, that signs at its leaf or below, are `no-extiende`,
which stops and is not co-signed. ⚠️ §714: the `mmrSize` already seen with another digest stops
with **any** peak, whether or not the node is behind, and at the genesis too, with its own class
in the journal —`bifurcacion`, «fork or restart without journal»; `DIARIO_VERSION` 5—, among the
heads of the same operator: the fixed key and those a judged rotation joins to it. What is
between the dashes was the rule of §712, which let through as `por-detras` the node that restarts
without journal and signs something else where it had already signed. The visible reset of §295
that signs the SAME digests again still does not stop. The index count is one per operator on
the three routes —the live loop, `--auditar`, which cold gives the clock as `indice-retrocede`,
and `--ausentes`—, and «behind» is on the first two the kept pair, which `--auditar` rebuilds
from the `anclando` and `extiende` lines of the journal. `--comparar` counts as a fork the same
`mmrSize` with the same peak and another digest; with ANOTHER peak it says so and does not count
it: it is the visible reset of §295 or a fork, and between two journals one cannot tell who was
behind. ⚠️ §714: that was §712. Since §714 `--auditar` no longer rebuilds the pair —the fork does
not depend on who is behind— and `--comparar` counts the same `mmrSize` with another digest with
any peak, of the same operator —the same key, or two that a chain of acts from the journals
joins—; two operators at the same sizes fork nothing. ⚠️ And it only counts the heads the witness
VERIFIED —the `nueva`, `repetida`, `hueco` and `vista-dividida` lines, and those of the node's
journal, which carry no class—: a head injected on the way that the witness wrote down as
`no-verifica` no longer accuses the operator (§714). And since §689,
`--comparar DIARIO_A DIARIO_B --sobres DIR` assembles
one for every index that TWO witnesses saw split —each one, alone, sees nothing—, with the head
of each journal. Since §714, `--auditar --sobres` and `--comparar --sobres` also assemble mode 5
for every fork, with the chain of acts when the two keys differ.

### 2.13 The co-signed anchor envelope (§633, RFC-0013 E4a)

```text
{ "v": 1, "tipo": "ancla-cofirmada",
  "cabeza": {…},                        la cabeza firmada cuya ancla se publico (como en 2.12)
  "nota": "zkssl/v1/…\n…",              la nota checkpoint del medio, ENTERA, como texto: sus tres
                                        lineas, la linea en blanco y las de firma (RFC-0013 D-A)
  "publicador": "zkssl/v1/…+…+…",       la vkey del publicador del medio (signed-note, tipo 0x06)
  "testigos": ["nombre+…+…", …],        OPCIONAL: las vkeys de los testigos cuyas cofirmas se
                                        reportan
  "posicion": "0x…",                    la posicion del ancla en el arbol del medio
  "inclusion": ["0x…", …] }             el camino RFC 9162 del ancla a la raiz de la nota: hashes
                                        SHA-256 en bytes, NO digests de Goldilocks
```

The anchor **does not travel**: it is DERIVED from the head, as in mode 1 of 2.12, and that is
why it is the signed head it claims to be. The order: first the FORM —`ancla`, `camino` and
`contraria` belong to the anchor envelope and here are rejected by name; every missing field,
too—; then the `cabeza`, WHOLE, because the `origin` of the note is derived from its key; then
the note, the co-signatures and the inclusion:

1. The head recomposes its digest and its signature verifies; the EMBEDDED index, the key's
   fingerprint and the anchor are derived.
2. The note is from the medium of THIS key —the publisher is called `zkssl/v1/` plus the
   fingerprint of the XMSS key in hexadecimal (RFC-0013 D-A)— and the publisher's line verifies
   with its vkey: ML-DSA-44, type `0x06`, with the `signed-note` and `tlog-checkpoint` rules that
   `crates/zk-ssl-medio/src/nota.rs` declares.
3. The co-signatures: every line of the note with a vkey of the envelope of the same name and
   `key_id` is VERIFIED, and if it does not verify it is ROJO; the lines without a vkey in the
   envelope are counted and not judged.
4. The leaf `SHA-256(0x00 || huella del ancla)` [the anchor's fingerprint] climbs along the
   `inclusion` up to the root of the note at the declared `posicion`.

VERDE: the head was published in that medium, at that position, and it is co-signed by the
witnesses the command lists, each with the SHA-256 fingerprint of its WHOLE key —the four-byte
`key_id` is an identifier and can be fabricated—.

⚠️ **IT REPORTS, IT DOES NOT JUDGE** (§633), like the v2 package with its co-signatures: which
witnesses count and how many are needed is decided by whoever verifies, with his policy (RFC-0013
D-D), not by the envelope, which the operator may assemble. Neither the threshold, nor the
freshness of the timestamps, nor that the medium is the operator's only one are decided by this
command. The threshold is judged, since §635, by
`zk-ssl-cli witness --ancla-cofirmada SOBRE --testigos-medio VKEYS [--k-medio K]`, with the
client's policy and over the same file.

**The bench is `tools/banco_ancla_cofirmada.sh`** (§634), which reproduces it live against a
real node that signs —three heads, their anchors derived by the command, and the operator's
medium signed by the `medio` example of `zk-ssl-medio`, which is not the publisher (E3)—, and
**the catalogue is `spec/vectors/ancla-cofirmada/`** (§634), the eleventh family of the artefact,
COPIED from one of its runs: section 9.

### 2.14 The overlap envelope (§686, RFC-0015 E5c)

```text
{ "v": 1, "tipo": "solapamiento", "cabeza": {…}, "actas": [ … ] }
```

- `cabeza` is a signed head (v3 to v6) as `zkssl_signedEpochHead` served it; `actas`, the
  operator's chain as `zkssl_keyActs` serves it (the one of 2.3). Without the other head: the
  overlap is proven by a single signature against the chain.
- **The rule** (RFC-0015, D-C, rules 3 and 4, and D-A): the index count is one per operator, and
  each key signs heads only in its **stretches** —above the `desde` of its act, which is the
  act's leaf, and below the next one's if the chain carries it—. A head whose EMBEDDED index
  falls outside ALL the stretches of its key is a leaf the operator was already giving to another
  key of his or to an act: **DETECTION with exit 0**, the mould of the split view (2.12, mode 4)
  and of the conflict: the envelope that exhibits it does not fail, it gives it away. Inside a
  stretch, ROJO: there is no overlap.
- ⚠️ **The genesis does not judge what its key signed before opting in** (the residue of D-I of
  RFC-0015): its stretch admits any leaf except that of its act. A key that returns to the chain
  has one stretch per entry, and falling in one is enough.
- ⚠️ **§722 — and the HANDOVER of the stretch**, the other count: that of the history of heads
  (2.3). Inside its stretch, the head has to carry an `mmrSize` equal to or greater than the
  handover of its act —and if equal, the peak it inherits— and below the handover of the next
  one, if the chain carries it. The genesis has no lower edge; the upper one, on the other hand,
  also judges what its key signed **before opting in**: D-I exempts it by index, not by
  `mmrSize`. Outside, it is a head at a place in the history that the chain gives to another key
  of the operator: **DETECTION with exit 0**, `VERDE: SOLAPAMIENTO - la clave firmo una cabeza
  con el mmrSize {t}, …`. It is the same judgement the extension makes on each of its two heads
  (2.3): with one chain, one verdict. Before the fix that head gave ROJO, «inside its stretch»
  (measured). A prefix of the chain removes upper handovers here too.
- ⚠️ **§722 — the VERDE does not say why.** It is produced by the common history of two operators
  in agreement (2.5) —the key that left kept signing above its handover—, and also by an honest
  operator that rotates from a **restored** journal: the emergency procedure of D-G of RFC-0015
  restores a snapshot, shorter than what the old key signed and served, and rotates with
  `--desde-minimo`. That jump avoids the overlap by index, not by `mmrSize`: the act's handover
  is the number of heads in the journal, and the old key's real heads are left above it. The same
  for a genesis signed over a new journal, with heads from before opting in over the old one.
  That is why D-G, step 3, rebuilds the journal BEFORE rotating with the heads the old key signed
  —from the witnesses, from the medium, from the clients; a line needs only its `epochDigest`—,
  and the node fails closed with `--relevo-minimo N`: it does not sign the rotation act if the
  journal carries fewer than `N` heads (`N`, the greatest `mmrSize` the old key signed, plus one).
  The envelope does not distinguish one case from the other, and does not pretend to: it is
  detection of what the key signed.
- ⚠️ **What whoever denounces cannot fabricate.** The acts are signed by the operator's keys
  —each one, the incoming one, and committed beforehand by the previous one—: whoever assembles
  the envelope cannot add a stretch nor move it. He can present a prefix of the chain, and that
  only removes upper stretches: it gives away less, never more. ⚠️ §722: the operator, on the
  other hand, can sign more than one chain —two acts of the same transition, with another
  `procedencia` (2.5)—, and whoever denounces uses any of them: the one with the lowest handover
  gives away the most.
- **The bench is `tools/banco_rotacion.sh`** (§649; the envelope, since §686) and **the catalogue
  lives in `spec/vectors/rotacion/`**, with the envelopes of the extension between two keys:
  section 9. The handover's is `tools/banco_historia_comun.sh` (§722).
- **The witness assembles it** (§687): `zk-ssl-cli witness --auditar DIARIO --sobres DIR` writes
  one envelope for every overlap its journal gives away, with the head that crosses —the line's,
  or by rule 3 the old key's with the highest leaf— and the chain of the rotation that fixed the
  stretch. The witness's detection becomes evidence that anyone verifies without it and without
  the node. ⚠️ §722: the witness does not apply the handover yet —neither in the live loop, nor in
  `--auditar`, nor in `--comparar`—, and the envelope of a head outside the handover of its
  stretch is assembled by hand (section 8).

## 3. The envelope — what the binary reads

The binary reads **31 distinct names** from the JSON. The first 14 are the envelope proper; the
rest are fields of the wire's responses that the binary needs to recompose and verify, and whose
meaning is in `spec/RPC.md`.

| object | keys the binary reads | where its semantics is |
|---|---|---|
| envelope | `v`, `tipo`, `cabeza`, `acuse`, `cofirmas`, `vieja`, `nueva`, `camino`; and `actas` (§648) | this document, section 2 |
| each act of `actas` (§648) | `acta` → `clave`, `esquema`, `desde`, `siguiente`, `procedencia` → `anterior`, `epochDigest`, `mmrRoot`, `mmrSize`; `firma`, `firmaAnterior` | `zkssl_keyActs`, `RPC.md`; RFC-0015 D-C |
| `cabeza` (and `vieja`/`nueva`) | `available`, `formatVersion`, `seq`, `n`, `accountsRoot`, `pendingRoot`, `frozenRoot`, `chainDigest`, `acusesRoot`, `epochDigest`, `publicKey`, `signature`, `index`; from v3 to v6 `mmrRoot`, `mmrSize`; and from v4 to v6 `consRoot`, `consCount` | `zkssl_signedEpochHead`, `RPC.md:437-482` |
| `acuse` | `hashPrueba`, `seq`, `camino` → `siblings`, `isRight` | `zkssl_ackPath`, `RPC.md:568-739` |
| each co-signature | `v`, `epochDigest`, `clavePublicaOperador`, `clavePublicaTestigo`, `firma`, `versionFormato`, `indice` | `zkssl_cosigs`, `RPC.md:741-783` |
| extension | `camino` (list of digests) | `RPC.md:785-812` |
| consumption | `consumo`, and `presencia`/`ausencia` → `siblings`, `isRight` | `zkssl_consumoPath`, `RPC.md` |
| conflict | `consumo`, and `libros[]` → `cabeza`, `presencia` → `siblings`, `isRight` | this document, section 2.5 |
| rejection | `data` → `causa`, `campos`, `seq`; `parametros` → the seven of `zkssl_params`; `presencia` → `siblings`, `isRight`; `recibo` → `rootOld`, `pendingRootOld`, `frozenRoot`; `lote[]` → `kind`, `sender` or `receiver`, `receipt` → `notice` → `position` or `notice` → `position`; `congelados` → `index`, `leaf`, `camino` → `siblings`, `isRight`; `peticion` → `amount`; `cuenta` → `index`, `leaf`, `camino` → `siblings`, `isRight`; `banda` → `s`, `publicId`, `requested`, `prueba` | this document, section 2.6 |
| age | `enunciado` → `t`, `k`, `emisor`; `subraices` → `pendientes`, `meta`; `prueba` | this document, section 2.7 |
| pending claim | `enunciado` → `receptor`, `nacido`, `inferior`; `prueba` | this document, section 2.8 |
| payment in flight | `enunciado` → `receptor`, `importe`, `t`, `nacido`; `prueba` | this document, section 2.9 |
| anchor (§592) | `ancla` → `v`, `clave`, `indice`, `epochDigest`, `mmrRoot`, `mmrSize`; `camino`; `contraria`; and since §714 `rama` and, with it, `actas` | this document, section 2.12 |
| co-signed anchor | `nota`, `publicador`, `testigos`, `posicion`, `inclusion`, and the `cabeza` of 2.12 | this document, section 2.13 (§633) |
| overlap | `cabeza` and `actas`, those of 2.3 | this document, section 2.14 (§686) |

⚠️ **§419 — the «31» above is no longer the count**: the consumption envelope adds `consumo`,
`presencia` and `ausencia`. **It is not replaced by another number**, because the 31 has no
locatable producer: a census of literals in the source gives 30 —it is blind to the keys read
through a variable— and the table of this very section gives 33. A figure without a universe is
declared, not invented; it is left for the cut that finds it one.

**§451 — the v5 head.** A `cabeza` with `formatVersion` 5 also carries `paramsDigest`,
`pmetaRoot`, `nextPending`, `nextIndex` and `totalSupply` (RFC-0007 E1, D-B), which the binary
reads with the same readers and demands all five; the `cabeza` row above is read with that
addition.

**§570 — the v6 head.** A `cabeza` with `formatVersion` 6 also carries `recepRoot` and
`recepCount` (RFC-0010 D-C, E2d), which the binary reads with the same readers and demands both,
in addition to the five of v5; the `cabeza` row is read with the two additions.

Quantities in the `Q` convention (`0x` + hex, u64); digests as `0x` + 64 hex; signatures and keys
as `0x` + hex. A value that does not have that form is rejected **before** touching the
cryptography (section 5).

## 4. What is checked, in order

The order matters: the cheap and the structural go first, and **the digest is never believed, it
is recomposed**. Every step that passes prints a line on standard output.

**Position package (v1 and v2):**

0. the file is read and is JSON; `v` is 1 or 2; a v1 does not carry `cofirmas`; the envelope
   **carries no `tipo`** —a present `tipo` leads to its form, and twelve are read, those of
   sections 2.3 to 2.14; one other than the twelve is **rejected by name**, never read as a
   position package (§418)—;
1. **`1/3`** — `cabeza` exists and is `available:true`; `formatVersion` is 2, 3, 4, 5 or 6; **the
   version chooses the recomposer**: v2 with the acknowledgement pair (§275), v3 also with the
   MMR's (§292), v4 also with the root and the count of consumptions (RFC-0006 E2a, §414), v5
   also with the family of the committed state (RFC-0007 E1a, §451), v6 also with the reception
   pair (RFC-0010 E2, §558); the fields of the head recompose its `epochDigest`;
2. **`2/3`** — the XMSS signature verifies against `publicKey` **and** the recovered preamble is
   the expected one (verifying without comparing proves nothing) **and** the declared `index`
   stays above the leaf index that goes inside the signature (§399; the bound is from below,
   section 8);
3. **`3/3`** — if there is an `acuse`: the leaf `hoja_de_acuse(hashPrueba, seq, n)` climbs along
   `camino` up to `acusesRoot`, and the fields compose again the signed digest (v2), the digest
   and the peak (v3), plus the root and the count of consumptions (v4), plus the family of v5
   (v5), or plus the reception pair (v6). If there is no acknowledgement, the head alone is
   proven;
4. **co-signatures** (v2 only) — each one, before touching the cryptography, names **this** head
   and **this** operator; then its signature verifies. How many verify is printed; how many are
   needed is not the package's business.

**Extension package:** `1/3` the two heads (v3, v4, v5 or v6) recompose their digest and their
signatures verify · `2/3` the same `publicKey` in both: continuity is of **one** signer —or, with
`actas` (§648), distinct keys that the chain joins: continuity is of **one** operator, and since
§722 the handover of 2.3: each head in its stretch's, by its two edges— · `3/3` the new peak
extends the old one along `camino`, and **afterwards**, since §712, the clock of 2.3, on an
indented line below: the same head with the same `mmrSize`, or the leaf that advances with the
same key, or «distinct keys: the order of their leaves is judged by the stretches».

**Consumption package:** `1/5` the two heads recompose their digest and their signatures verify
· `2/5` the same `publicKey` and both carry `consRoot` (**v4, v5 or v6**) on both sides · `3/5`
the new peak extends the old one, and then the clock of 2.3 (§712) · `4/5` the two paths are
those of the position **derived** from the consumption, not of the one the envelope says · `5/5`
the consumption is under the `consRoot` of the new one and **was not** under the old one's.

**Conflict package:** `1/4` the two heads recompose their digest and their signatures verify ·
`2/4` they are of **distinct** operators and both carry `consRoot` (**v4, v5 or v6**) · `3/4` the
two paths are those of the position **derived** from the consumption · `4/4` the same consumption
is under the `consRoot` of **both** ledgers.

**Rejection package:** `1/3` the head recomposes its digest and its signature verifies · `2/3`
the material of the cause, the one section 2.6 assigns it: the `parametros` recompose the
`paramsDigest` of a **v5 or v6** head; the path of `presencia` is that of the position
**derived** from the consumption under a **v4, v5 or v6**; the `recibo` declared its three roots;
the `lote` is read in order; the path of `congelados` or of `cuenta` is that of the account
`data` names; or the statement of the `banda` is that of that account under the signed
`accountsRoot` · `3/3` the cause holds over what is committed, with the head **prior** to the
rejection —or the same one— when it cites something that only grows, and with **the same one**
when the state is instantaneous; in the `banda`, what holds is the proof, which verifies without
the balance.

**Age package:** before touching the cryptography, the envelope has its form (`enunciado`,
`subraices`, `prueba`) and the head is **v5 or v6** · `1/3` the head recomposes its digest and
its signature verifies · `2/3` the two subroots, climbed to 32 levels with the `m` the command
derives from `nextPending`, are `pendingRoot` and `pmetaRoot` · `3/3` the proof verifies against
the statement the head fixes (`seq`, `nextPending`, `m`) and the envelope asserts (`t`, `k`,
`emisor`), with the house options. The statement is checked in step `2/3`, before climbing the
subroots: an `m` over its cap falls there (§717), without reading the proof.

**Pending claim package:** before touching the cryptography, the envelope has its form
(`enunciado`, `prueba`) and the head is **v5 or v6** · `1/3` the head recomposes its digest and
its signature verifies · `2/3` the statement is composed by the judge with `pendingRoot`,
`pmetaRoot` and the field's ceiling, and the `nacido` the envelope asserts is earlier than the
signed `seq` · `3/3` the proof verifies against that statement, with the house options. The last
two are decided by the SAME judge, which checks the `nacido` **before** touching the proof.

**Co-signed anchor envelope** (§633): before touching the cryptography, the form (without
`ancla`, `camino` nor `contraria`; with `cabeza`, `nota`, `publicador`, `posicion` and
`inclusion`, each hash of 32 bytes, and readable vkeys in `testigos`) · `1/4` the head recomposes
its digest and its signature verifies, and its anchor is derived · `2/4` the note is from the
medium of its key and the publisher's signature verifies · `3/4` the co-signatures with a key in
the envelope verify, and are listed · `4/4` the anchor climbs up to the root of the note at its
position.

**Overlap envelope** (§686): before touching the cryptography, the form (`cabeza`, and `actas`
read one by one) and the version of the head · `1/3` the head recomposes its digest and its
signature verifies, and its embedded index is read · `2/3` the chain holds whole, link by link,
and the head's key enters it · `3/3` the index falls outside all the stretches of its key: VERDE,
SOLAPAMIENTO, the detection. ⚠️ §722: if it falls in one, the `3/3` looks at its `mmrSize`
against the handover of that stretch (2.14), and outside it, it is also VERDE, SOLAPAMIENTO.

**Anchor envelope, mode 5: the fork** (§714): before touching the cryptography, the form (the
`rama` without `ancla` nor `contraria`, after the two form rules of the anchor) · `1/3` the head
recomposes its digest and its signature verifies · `2/3` the `rama` carries the MMR pair,
recomposes its digest and its signature verifies, and both are of the same operator —the same
`publicKey`, or keys the `actas` join— · `3/3` the same `mmrSize`, with any peak, and distinct
digests: VERDE, BIFURCACIÓN, the detection.

Heads **v2, v3, v4, v5 and v6** (`formatVersion`): a kept v2 head **still verifies** — the
shutdown of §290 does not expire. A v1 head is verified with the library, not with this command.

## 5. Catalogue of rejections

Any rejection stops the binary with **the first failure, by name**, on the error output, and it
exits with code 1 (section 6); the three texts of the fourth state of the completeness envelope
exit with 3 and without `ROJO` (completeness block), and since §723 so does that of the
rejection envelope whose band is of v1 below the bound, alone or inside a completeness
resolution (rejection block, `banda: {e}`). There is no panic on a malformed package: every
field read fails closed. The catalogue is the list of messages the binary can emit, with their
placeholders in braces; a reader who sees one of these texts knows which rule has fallen. **The
truth is measured in the source**: on sealing, `tools/check_catalogo.py` re-derives the census by
call, not by line, inside the canon (§777) —at §776, 254 rejection calls and 225 distinct texts,
and 63 of those calls are split with `\` in the source—, and every text has to be here, including
the one that is not reached today: the census is by call, and an unreachable text enters with
the note that says why.

**Reading the file and the envelope's version**

- `no se puede leer {ruta}: {e}`
- `JSON ilegible: {e}`
- `el verificador entro en panico: {m}` — §711: the command's net, around everything else; `{m}`
  is `panico recogido al juzgar una prueba ajena: {mensaje}`, with the panic's message (section
  6, «A panic is not an exit code»). No vector pins it: what was measured is rejected earlier by
  the fork, with its cause (section 6.1), and the net covers what was not measured.
- `el paquete no declara su version en `v``
- `el paquete declara v:{v_paquete} — este binario lee v1 y v2`
- `un paquete v1 con `cofirmas`: subir la version es lo que las hace parte del contrato — declaralo v2, o quitalas`
- `tipo desconocido: {otro} - se lee un paquete de posicion (sin `tipo`), `tipo: "extension"`, `tipo: "consumo"`, `tipo: "conflicto"`, `tipo: "rechazo"`, `tipo: "edad"`, `tipo: "cobro_pendiente"`, `tipo: "pago_en_curso"`, `tipo: "prenda"`, `tipo: "completitud"`, `tipo: "ancla"`, `tipo: "ancla-cofirmada"` o `tipo: "solapamiento"`` — ⚠️ §633: until here the text ended at `prenda`, stale since §573; it is brought up to date on adding the arm, and the test that ties it looks for the last one (since §686, `solapamiento`).

**Form of the values** (`hex_a_bytes`, `digest_de`, `u64_de`; `{campo}` is the key being read)

- `sin 0x: {s:.18}` · `hex impar ({} chars)` · `hex: cifra no admitida en la posicion {i}` —
  ⚠️ §776: until here it said `hex: {e}`, stale since §650, when the reader moved to
  `zk_ssl_hash::hex_canonico` and each `HexError` to its text; `{i}` is the position, counted in
  bytes, of the first non-admitted digit in the string without the `0x`.
- `hex no minimo: {s:.18}` — unreachable today: it is the arm of `HexError::NoMinima`, which only
  `cantidad_canonica` produces (§662), and `hex_a_bytes` reads with `hex_canonico`, which does
  not return it. It enters because the census is by call and the `match` is exhaustive.
- `falta {campo} o no es cadena` · `{campo}: {} bytes, se esperaban 32` · `{campo}: {e}`
- `falta {campo} o no es cadena 0x` · `{campo} sin 0x` · `{campo}: {e}`
- `{campo}: 0x… no es canonico: no es menor que p = 2^64 - 2^32 + 1` — since §640 (RFC-0016,
  `NUCLEO.md` section 6, «Canonicidad»): a `u64` of the envelope, or one of the four elements of
  a digest, that is not below the field's modulus. It is rejected on READING, before recomposing
  and before the signature, with the value as it arrived. The same text in the paths:
  `sibling {i}: …`, `camino[{i}]: …`, `{mote}: siblings[{i}]: …`. Until §640 the digest was read
  reducing and the text at this point was `{campo}: {e:?}`, unreachable.

**The head** (steps 1 and 2)

- `falta cabeza`
- `la cabeza empaquetada no era available:true`
- `formatVersion {version}: el paquete v1 empaqueta cabezas v2, v3, v4, v5 o v6 (la pareja acusesRoot/n viaja firmada desde §275; la del MMR, desde §292)` — the text says «v1» even when the envelope is v2: the rule in force, prose the reference implementation must correct without changing the rule. The set is derived by `VersionCabeza::texto()`. ⚠️ §776: until here it said `v2, v3, v4 o v5`, stale since §558, when v6 entered the set.
- `los siete campos NO recomponen el epochDigest empaquetado: o el paquete esta adulterado o la cabeza nunca fue esa`
- `falta publicKey` · `falta signature`
- `cabeza: {e}` — the signature does not verify, the preamble is not the expected one, or the declared `index` does not stay above the one that goes inside the signature (§399); `{e}` is the library's error.

**The acknowledgement** (step 3)

- `acuse sin camino` · `camino sin siblings` · `camino sin isRight`
- `sibling {i} no es cadena` · `sibling {i}: {} bytes` · `sibling {i}: {e}`
- `isRight no booleano`
- `acuse: {e:?}` — the leaf does not climb to the signed root, or the fields do not compose the
  digest of the signed head with the recomposer of its version; `{e:?}` is the `Debug` of
  `zk_ssl_verify::inclusion::InclusionError` (`RaizDistinta`, `CabezaDistinta` or
  `CaminoDescuadrado { … }`). One place, `acuse_contra_cabeza`, for the five versions, from v2 to
  v6, and one text.

**The co-signatures** (step 4, v2 only)

- `cofirmas no es una lista`
- `cofirma {n}: falta {campo}`
- `cofirma {n}: version {cv} desconocida, este binario lee hasta la {}` — the cap is the library's `COFIRMA_V_MAX`.
- `cofirma {n}: versionFormato {vf:#x} no cabe en el byte que el preambulo le reserva` — since
  §780: the co-signature's preamble carries the head's version in one byte, and until then the
  kit truncated it (`as u8`): `0x103` verified as `0x3`, a value with two encodings.
- `cofirma {n}: acredita OTRA cabeza, no la empaquetada`
- `cofirma {n}: acredita a OTRO operador, no al que firmo la cabeza`
- `cofirma {n}: repite el testigo de la cofirma {m}` — since §663: one witness, one co-signature;
  the same one copied several times no longer counts as several witnesses. `{m}` is the first one
  that carries it, counted from 1 like `{n}`.
- `cofirma {n}: {e}` — the witness's signature does not verify.
- An empty or absent list is not an error: the v2 package prints that the head stands alone.

**The extension** (`{cual}` is `vieja` or `nueva`)

- `falta vieja` · `falta nueva`
- `{cual}: la cabeza no era available:true`
- `{cual}: formatVersion {version} — la extension exige cabezas v3, v4, v5 o v6: una v2 no lleva la pareja del MMR que extender` — the set is derived by `VersionCabeza::texto_con_mmr()`. ⚠️ §776: until here it said `v3, v4 o v5`, stale since §558.
- `{cual}: los campos NO recomponen su epochDigest — adulterada o inventada`
- `{cual}: falta publicKey` · `{cual}: falta signature`
- `{cual}: cabeza: {e}`
- `las cabezas llevan claves DISTINTAS: la continuidad es de UN firmante` — without `actas`;
  with them (§648), one of these, the same ones in the consumption and in the places of the
  completeness closing:
  - `actas no es lista (la cadena de zkssl_keyActs)`
  - `actas[{k}]: {e}`, with `{e}` the error of `acta_de_json`, which names the field
  - `las cabezas llevan claves DISTINTAS y las actas no las unen: {e}`, with `{e}` one of: `el
    acta {eslabon} de la cadena no vale: {regla}` · `la clave que llega no esta en la cadena:
    nadie la comprometio` · `la clave que se tenia no esta en la cadena antes de la que llega` ·
    `SOLAPAMIENTO: la clave que se va firmo en la hoja {indice}, y su sucesora empieza en la
    {desde}` · `SOLAPAMIENTO: la clave que llega firmo en la hoja {indice}, fuera de su tramo:
    por encima de la {desde}[ y por debajo de la {hasta}]`
  - `las cabezas llevan claves DISTINTAS y las actas no las unen: {e}`, the same prefix: ⚠️
    §722, in the eight places of 2.3 —the extension, the consumption and the six heads of the
    completeness—, after those above and in the order of the rules of 2.3, with `{e}` one of:
    1. `SOLAPAMIENTO: la clave que se va firmo en la hoja {indice}, fuera de todos sus tramos: la
       cuenta del operador la daba a otra clave suya o a un acta`;
    2. `RELEVO: el acta {k} releva en el mmrSize {s}, por debajo del {s'} de la previa: la
       historia de cabezas no retrocede`;
    3. the earlier one: `RELEVO: la cabeza de la clave que se va lleva el mmrSize {t}, y el acta {k}
       la hace entrar en el {s}: la clave que se va firma desde ahi` · `RELEVO: la cabeza de la
       clave que se va firma el mmrSize {s} de su relevo con otra cima que la que el acta {k} le
       hace heredar` · `RELEVO: la cabeza de la clave que se va lleva el mmrSize {t}, y el acta
       {k} la releva en el {s}: la clave que se va firma por debajo`;
    4. the later one: `RELEVO: la cabeza de la clave que llega lleva el mmrSize {t}, y el acta {k}
       la hace entrar en el {s}: la clave que llega firma desde ahi` · `RELEVO: la cabeza de la
       clave que llega firma el mmrSize {s} de su relevo con otra cima que la que el acta {k} le
       hace heredar` · `RELEVO: la cabeza de la clave que llega lleva el mmrSize {t}, y el acta
       {k} la releva en el {s}: la clave que llega firma por debajo`.

    An envelope that breaks two rules says the first one in that order, in the kit and in the
    second implementation. The anchor's fork does not apply it (section 8).
- `falta camino (lista de digests)`
- `camino[{i}] no es cadena` · `camino[{i}]: {} bytes` · `camino[{i}]: {e}` — ⚠️ §776: until
  here it said `{e:?}`, stale since §640.
- `la nueva (t={t_n}) NO extiende a la vieja (t={t_v}): historia bifurcada, recortada, o camino que no es el suyo`
- ⚠️ §712, after consistency, and the same ones in the consumption and in the anchor of mode 3,
  with `{vieja}` and `{nueva}` the names of the sides: `la vieja` and `la nueva` in the extension
  and the consumption, `el ancla` and `la cabeza` in the anchor:
  - `extension: mismo mmrSize y digest distinto - {vieja} y {nueva} firman la misma historia de {t} cabeza(s) y no son la misma cabeza: una bifurcacion firmada, no una extension`
  - `extension: el indice embebido no crece - {nueva} (t={t_n}) firma en la hoja {e_n} y {vieja} (t={t_v}) en la {e_v}: con la misma clave, una cabeza posterior firma por encima (RFC-0012 D-C)`

**The consumption** (`{cual}` is `presencia` or `ausencia`)

- `el sobre de consumo exige cabezas v4, v5 o v6: una v2 o v3 no lleva consRoot contra el que comprobar` — the set is derived by `VersionCabeza::texto_con_consumos()`, and the producer (`exige_consumos`) is one, with `consumo`, `conflicto`, `rechazo` or `prenda` as subject. ⚠️ §776: until here it said `v4 o v5`, stale since §558.
- `falta {cual} (camino del consumo)` · `{cual}: falta siblings` · `{cual}: falta isRight`
- `{cual}: siblings[{i}] no es cadena` · `{cual}: siblings[{i}]: {} bytes` · `{cual}: siblings[{i}]: {e}` — ⚠️ §776: until here it said `{e:?}`, stale since §640.
- `{cual}: isRight[{i}] no es booleano`
- `{cual}: el isRight recibido NO es el de la posicion {pos} que el consumo DERIVA - un camino de otra posicion no prueba nada de este consumo`
- `{cual}: el camino no tiene los 63 niveles del arbol de consumos`
- `presencia: el camino NO sube al consRoot de la nueva`
- `ausencia: la hoja vacia NO sube al consRoot de la vieja - el consumo YA estaba`

**The conflict** (`{i}` is the position in `libros`; `{cual}` comes out `libro[0]` or `libro[1]`)

These texts are BORN with this form. Everything else a conflict envelope can emit comes from the
**same producers** that are already above, with their placeholder filled differently —the head,
the v4, v5 or v6 version, the fields of the path and its mismatch—: **a placeholder filled
another way is not a new entry of the catalogue**, it is the same text and the same producer.

- `falta libros` · `libros no es una lista`
- `el sobre de conflicto exige DOS libros: se recibieron {n}`
- `{cual}: falta cabeza`
- `las cabezas llevan la MISMA clave: un conflicto es entre DOS firmantes`
- `libro[{i}]: el camino NO sube al consRoot de su cabeza`

**The rejection** (`{causa}` is the one of `data`; `{que}` comes out `limite`, `limite esperado`,
`tope` or `tope de suministro`)

These texts are BORN with this form. The head, the version of the consumptions (`el sobre de
rechazo exige cabezas v4, v5 o v6: …`), the fields of the path, its mismatch and the position
cross-check come from the **same producers** above, with their placeholder filled differently.

- `falta data (el objeto del rechazo)` · `data: falta causa` · `data: falta campos`
- `data: la causa {otra} no la prueba este mando (spec/PAQUETE.md, seccion 2.6)`
- `la causa {causa} exige una cabeza v5 o v6: sus parametros viajan en paramsDigest` — one producer (`parametros_comprometidos`) for the four parameter causes: `OverRegulatoryLimit`, `AccountLimitReached`, `WrongRegulatoryLimit` and `SupplyCapExceeded`; the set is derived by `VersionCabeza::texto_con_parametros()`. ⚠️ §776: until here it said `una cabeza v5`, stale since §558, and the entry appeared twice, the second one «also `WrongRegulatoryLimit`».
- `falta parametros (zkssl_params)`
- `parametros: NO recomponen el paramsDigest de la cabeza - no son los de este libro`
- `data: el {que} que el nodo dice ({dicho}) no es el comprometido ({comprometido})`
- `la causa NO se sostiene: el importe pedido ({pedido}) no supera el limite ({limite})`
- `la causa NO se sostiene: nextIndex ({n}) no alcanza el tope de cuentas ({tope})`
- `la causa NO se sostiene: ocupante y consumo son el MISMO - eso seria ConsumoRepetido`
- `la causa NO se sostiene: el ocupante vive en la posicion {po}, no en la {pos} del consumo`
- `presencia: el camino NO sube al consRoot de la cabeza`
- `la cabeza (seq {s_cabeza}) es POSTERIOR al rechazo (seq {s_rechazo}): {porque}, y solo una cabeza anterior lo prueba`
- `falta recibo (los publicInputs del rechazado)` · `StaleState no lleva campos: su material es el recibo`
- `la causa NO se sostiene: las tres raices del recibo son las de la cabeza - ese estado NO estaba atras`
- `la causa NO se sostiene: el limite declarado ({declarado}) ES el comprometido ({limite})`
- `falta lote (las ops de applyMany)` · `lote[{i}]: falta kind` · `lote[{i}]: kind desconocido: {otro}`
- `lote[{i}]: falta receipt.notice` · `lote[{i}]: falta notice`
- `la causa NO se sostiene: el lote no tiene cuentas ni posiciones repetidas`
- `la causa NO se sostiene: el primer choque del lote es {n}, no {causa}`
- `data: el {clave} que el nodo dice ({dicho}) no es el del primer choque del lote ({v})`
- `la cabeza (seq {s_cabeza}) no es la del rechazo (seq {s_rechazo}): esta causa se juzga sobre un estado instantaneo, no sobre lo que crece`
- `falta congelados (el camino de zkssl_frozenPath)` · `congelados: falta camino`
- `data: el index que el nodo dice ({dicho}) no es el del camino ({del_camino})`
- `congelados: el isRight recibido NO es el de la cuenta {i} - un camino de otra cuenta no prueba nada de esta`
- `congelados: el camino no tiene los 32 niveles del arbol de congelados`
- `congelados: el camino NO sube al frozenRoot de la cabeza`
- `la causa NO se sostiene: la hoja de la cuenta {i} bajo el frozenRoot es la vacia - no estaba congelada`
- `falta cuenta (el camino del arbol de cuentas)` · `cuenta: falta camino`
- `cuenta: el isRight recibido NO es el de la cuenta {i} - un camino de otra cuenta no prueba nada de esta`
- `cuenta: el camino no tiene los 32 niveles del arbol de cuentas`
- `cuenta: el camino NO sube al accountsRoot de la cabeza`
- `la causa NO se sostiene: la hoja de la cuenta {i} bajo el accountsRoot NO es la vacia - la cuenta existe`
- `falta peticion (los params de la emision rechazada)`
- `data: el wouldBe que el nodo dice ({dice}) no es el suministro de la cabeza mas el importe pedido ({suministro} + {importe} = {seria})`
- `la causa NO se sostiene: el suministro resultante ({seria}) no supera el tope ({tope})`
- `data: esta causa no publica el saldo: la banda lo prueba sin el` · `falta banda (la prueba de la desigualdad)`
- `data: el importe que el nodo dice no es el que la prueba acota` · `la causa NO se sostiene: pedir 0 no puede pasar de ningun saldo`
- `banda: falta prueba o no es cadena 0x` · `banda: {e}`, where `{e}` is the red of `zk_ssl_air::banda::verificar`: `los limites {l} y {u} pasan del techo {MAX_VALOR}`, `banda vacia: inferior {l} sobre superior {u}`, `la prueba no se deserializa: …`, `forma de traza …`, those of the v2 mark (`marca: …`, `marca v1: …`, `marca desconocida` and `la prueba no se deserializa: sin cabeza`: the age's eight, before anything else, §914), `prueba de la v1 con segmentos de 63 bits: con el superior {u}, bajo {COTA_V1}, no acota el saldo privado: un saldo de la ventana [2^62, 2^63) tambien la pasa` (§723: the judge gives it, and the command prints it not as ROJO but in the fourth state, below), or the one winterfell writes (§478)
- `DECLARADA, NO PROBADA: {m}` — since §723, the fourth state of the rejection envelope, with exit 3 and without `ROJO`. The v2 band (the range segments at 62 bits; the correction of D-3 of RFC-0017): a v1 band verifies only with the upper bound at `2^32 - 2` or above, or if it is the already published one of `PUBLICADAS_V1`. Below, a balance in the window `[2^62, 2^63)` passes it too, and the envelope does not come out ROJO but with the fourth state: `{m}` is `banda: {e}. El operador declara InsufficientBalance con una prueba de la v1, que no lo prueba: una prueba nueva es de la v2`, with `{e}` the window text of the list above, so the line starts with `DECLARADA, NO PROBADA: banda: prueba de la v1 con segmentos de 63 bits: con el superior {u}, bajo 4294967294, no acota el saldo privado: …`; inside a completeness envelope, with the place in front: `DECLARADA, NO PROBADA: resolucion.sobre: banda: …` or `resolucion.rechazo: banda: …`

**The age** (§465; `{e}` comes from `zk_ssl_air::verificar_contra_cabeza`)

These texts are BORN with this form. The form of the values and the head —its recomposition,
its signature and the v5 family— come from the **same producers** above.

- `falta enunciado` · `falta subraices` · `falta prueba o no es cadena 0x`
- `formatVersion {version}: la prueba de edad exige una cabeza v5 o v6, las que firman pmetaRoot y nextPending` — the set is derived by `VersionCabeza::texto_con_parametros()`. ⚠️ §776: until here it said `una cabeza v5, la unica que firma pmetaRoot y nextPending`, stale since §558, when the text moved to the predicate and v6 entered the set.
- `edad: {e}`, with `{e}` one of these:
  - `la subraiz de pendientes, subida a 32 niveles, no es el pendingRoot de la cabeza`
  - `la subraiz de meta, subida a 32 niveles, no es el pmetaRoot de la cabeza`
  - those of the statement: `m = {} fuera de {M_MIN}..={M_MAX}` · `n = {} no cabe en 2^{}` · `T = {} o seq = {} no caben en {BITS} bits` · `k = {} mayor que n = {}` · `emisor = {} no es un indice de cuenta: no cabe en 32 bits` · `todos con emisor = {}: todos se escribe con emisor 0` (§718) — ⚠️ §706: until §706 it said `1..=24`, stale since §538, when D-AG of RFC-0009 raised the floor of m to 3. ⚠️ §717: until §717 it said `3..=24`; since §717 the cap is `M_MAX`, 13 (§2.7). ⚠️ §914: until §914 it said `3..=`; since the cut the floor is `M_MIN`, 4, with T = 128.
  - `la prueba no se deserializa: {e:?}` · `forma de traza {forma:?}; el enunciado pide {:?}` · `{e:?}`, the error of `winter-verifier`
  - those of the v2 mark, BEFORE anything else (§914, the `zkssl/0.5` cut; RFC-0018 D-G): the judge reads them from the header of the proof —the four bytes, the length of the meta and the meta— without deserializing the rest, before the statement and the whole proof, with `zk_ssl_air::comprobar_marca_v2`, letter for letter those of the fork's `verify_v2` and of the second implementation: `la prueba no se deserializa: sin cabeza` (1a: the proof does not reach the header) · `marca: meta vacio, prueba anterior a zkssl/0.4; la juzga el kit de su version` (1b) · `marca v1: prueba de zkssl/0.4; la juzga el kit 0.4.x` (1c: the 19-byte mark of `zkssl/0.4`) · `marca desconocida` (1d: without the prefix `arqueo:oculta:2`, with a length other than 25, a field at zero, profile 255 or zero columns) · `marca: la prueba es de la familia {}; se pide la {}` (1e; the families, in `spec/NUCLEO.md` §4 bis) · `marca: version {} de la AIR de la familia {}; se acepta {}` (1f) · `marca: perfil {} desconocido` (1g) · `marca: m = {}; la construccion pide m = {}` (1h; m = 64) · `marca: {} columnas; la construccion pide {}` (1h bis, §914: the columns the mark declares are those of the construction, two; without this a mark with another figure passed the form and the fork panicked when building the AIR). ⚠️ §914: until §914 the text was `marca de la ocultacion {lectura:?}; el enunciado pide m = 64`, that of the reader of the v1 mark (§651, §706), which the judge looked at after the trace form and before the options, with `{lectura}` what `Marca::leer` read; since the cut a proof with that mark falls in 1c, and is judged by kit 0.4.x.

**The pending claim** (§495; `{e}` comes from `zk_ssl_air::cobro_pendiente`)

These texts are BORN with this form. The form of the values and the head —its recomposition,
its signature and the v5 family— come from the **same producers** above, and `falta enunciado`,
`falta prueba o no es cadena 0x` and `falta cabeza` are the age's, letter for letter.

- `formatVersion {version}: el cobro pendiente exige una cabeza v5 o v6, las que firman pmetaRoot` — ⚠️ §776: until here it said `una cabeza v5, la unica que firma pmetaRoot`, stale since §558.
- `cobro: {e}`, with `{e}` one of these:
  - `nacido {} no es anterior a la cabeza de seq {}: una meta nacida despues de la cabeza que la firma`
  - those of the statement: `las cotas {l} y {u} pasan del techo {MAX_VALOR}` · `banda vacia: inferior {l} sobre superior {u}` · `nacido {} pasa del techo {MAX_VALOR}` (§718); and, over the `u64` as it arrives and before reducing (§718, canonicity), `las cotas {l} y {u} pasan del techo {MAX_VALOR}` with `{l}` the envelope's lower bound, and the `nacido` above
  - `la prueba no se deserializa: {e:?}` · `forma de traza {forma:?}; el enunciado pide {:?}` · `{e:?}`, the error of `winter-verifier`
  - those of the v2 mark, before anything else (§914): the age's eight
  - `prueba de la v1 con segmentos de 63 bits: con el superior {u}, bajo {COTA_V1}, no acota el importe privado: un importe de la ventana [2^62, 2^63) tambien la pasa` — §723: the v1 one with an upper bound below the bound; against the head the upper bound is `MAX_VALOR` and it does not come out

**The payment in flight** (§506; `{e}` comes from `zk_ssl_air::pago_en_curso`)

These texts are BORN with this form. The form of the values and the head —its recomposition,
its signature and the v5 family— come from the **same producers** above, and `falta enunciado`,
`falta prueba o no es cadena 0x` and `falta cabeza` are the age's, letter for letter.

- `formatVersion {version}: el pago en curso exige una cabeza v5 o v6, las que firman pmetaRoot` —
  the set is derived by `VersionCabeza::texto_con_parametros()`; until §776 this block was not
  in the catalogue.
- `pago: {e}`, with `{e}` one of these:
  - `nacido {} no es anterior a la cabeza de seq {}: una meta nacida despues de la cabeza que la firma` — the claim's, letter for letter, and as there before touching the proof
  - those of the statement: `el importe {i} pasa del techo {MAX_VALOR}` · `T {t} o nacido {n} pasan del techo {MAX_VALOR}` · `nacido {n} posterior a T {t}: el plazo iria hacia atras`
  - `la prueba no se deserializa: {e:?}` · `forma de traza {forma:?}; el enunciado pide {:?}` · `{e:?}`, the error of `winter-verifier`
  - those of the v2 mark, before anything else (§914): the age's eight

**The pledge** (§520; `{e}` comes from `zk_ssl_air::prenda`)

- `formatVersion {version}: la prenda exige una cabeza v5 o v6 - no por la meta, que no lleva (D-AY), sino porque es la que el nodo sirve y contra la que juzga zkssl_pledge` — ⚠️ §776: until here it said `una cabeza v5`, stale since §558.
- `prenda: {e}`, with `{e}` one of these:
  - `la prueba no se deserializa: {e:?}` · `forma de traza {forma:?}; el enunciado pide {:?}` ·
    `{e:?}`, the error of `winter-verifier`
  - those of the v2 mark, before anything else (§914): the age's eight

`falta enunciado`, `falta prueba o no es cadena 0x` and `falta cabeza` are the age's, letter for
letter. **There is no rejection for an unpublished mark**: this arm does not check it, and the
VERDE says so (D-AS).

**The completeness** (§573, RFC-0010 E4; RFC-0014 E4a and E4b; RFC-0019 E1 and E2; section 2.11)

These texts are BORN with this form; until §776 this block was not in the catalogue. What the
envelope RE-VERIFIES comes from the **same producers** above, with its subject as placeholder:
every head —the `cierre`, the `vigente`, that of each resolution (`resolucion.cabeza`,
`resolucion.acuses[{i}].cabeza`, `resolucion.sobre.cabeza`, `resolucion.rechazo.cabeza`,
`resolucion.consumo.cabeza`) and the `resolucion.juzgada`— by those of the extension, with
`{cual}` that name, and by those of the `actas` when its key is not the closing's (§648); the
paths, by those of the consumption, with `camino` or `resolucion.consumo.camino` as `{cual}`; the
pair of the resolution by acknowledgement, by those of the position package
(`verificar_paquete`); the envelope of a resolution by rejection, by those of the rejection; and
the envelope of the accepted pledge, by those of the pledge. The form of the values is the one
above. **A placeholder filled another way is not a new entry.**

*The closing and the receipt* (steps 1 and 2)

- `falta cierre (la cabeza v6 que cierra la era del recibo)`
- `cierre: formatVersion {version}: el sobre de completitud lee cabezas v6` — a version
  `VersionCabeza` does not know; the set is derived by `VersionCabeza::texto_con_recepcion()`
- `cierre: el sobre de completitud exige una cabeza v6: la pareja de recepcion viaja firmada desde ella` — a known version that does not carry the pair, from v2 to v5
- `{cual}: {e}` — that of `indice_embebido`, which reads the XMSS index from inside the signature
  (§681): that of the `cierre` —the start of the window—, that of each head that measures it
  (`{que}.cabeza`, `resolucion.consumo.cabeza`, `vigente`) and, with `cabeza` fixed as `{cual}`,
  that of the two heads `misma_continuidad` compares with `actas` (§648: extension, consumption
  and the six of the completeness) and that of the overlap's head (§686); `{e}` would be the
  library's `FirmaSinIndice`. Unreachable today: every call comes after `cabeza_v3_verificada`
  over the same head, and if its `verificar_cabeza` passed, it already read that index from the
  same bytes without error. It enters because the census is by call.
- `falta recepcion (el recibo del cable, §571)`
- `recepcion: n {n} no es la que el cierre firma ({n_firmada}): una n mentida produce una hoja que no verifica`
- `recepcion: el rx {rx} no pertenece a la era ({q}, {r}] que el cierre cierra` —
  `Q < rx <= recepCount`, with `Q` the DECLARED `limiteAnterior` (D1)
- `falta camino (el de zkssl_recepPath)` — its fields are read afterwards with the consumption's
  texts and `camino` as subject
- `camino: no mide 32 niveles o sus lados no son los de la posicion {pos} (rx {rx} - Q {q} - 1)` —
  32 is `RECEP_DEPTH`
- `el recibo NO sube a la recepRoot que el cierre firma`

*The verdict* (step 3)

- `resolucion: tipo {otro:?} desconocido: se lee acuse, rechazo o declarada, y lote o prenda para los recibos del RFC-0014` — `{otro:?}` is the `Debug` of what was read in `resolucion.tipo`: `Some("…")`, or `None` if it is missing or is not a string
- `sin resolucion y sin cabeza vigente: no hay ventana que medir`
- `ventana ABIERTA: la cabeza vigente tiene indice acreditado {s} y la ventana empieza en {inicio}; con n {n} la promesa sigue viva y el sobre es prematuro`
- `NO RESUELTA EN LA VENTANA: el operador recibio la operacion bajo su firma (rx {rx}, era {era}) y su cabeza vigente de indice acreditado {s} -{d} cabezas firmadas despues del inicio {inicio}, n {n}- llega sin resolucion: se comprometio a resolver y no lo hizo` — verdict 3, **NAMED ROJO** and the milestone's product; `{d}` is `S − inicio`

*The resolution by acknowledgement* (verdict 1; `{que}` is `resolucion`, and `resolucion.acuses[{i}]` in the batch)

- `{que}: falta cabeza` · `{que}: falta acuse`
- `{que}: el acuse es de OTRA prueba: no resuelve este recibo`
- `{que}: llega FUERA de la ventana (indice acreditado {s}, inicio {inicio}, n {n})`

*The resolution by rejection* (verdict 2; `{que}` is `resolucion.sobre`, also in the batch, and `resolucion.rechazo` in the pledge)

- `resolucion: falta sobre (el de rechazo)` — only on the direct path
- `resolucion: el sobre no es de tipo rechazo` — with `resolucion` fixed, whatever `{que}` is
- `{que}: falta data` · `{que}: falta cabeza`
- `{que}: su data no lleva recepcion: no se ata a este recibo` · `{que}: es de OTRA operacion: su hashPrueba no es el del recibo` — the binding to the receipt by the `recepcion.hashPrueba` of the receiving object (D3), which is the node's WORD: the `data` of the rejection envelope; the same producer (`exige_mismo_recibo`) reads it in `resolucion.data` in the batch and in the declared one, and in `resolucion.respuesta` itself in the pledge, which carries no `data` although the text says «su data»
- `resolucion: llega FUERA de la ventana (indice acreditado {s}, inicio {inicio}, n {n})` — with `resolucion` fixed; the same text in the accepted pledge

*The batch* (verdict 5, §612; `{que}` is `resolucion.sobre.data` or `resolucion.data`)

- `resolucion: falta composicion (la lista del lote, D-C)` · `resolucion: composicion VACIA: el nodo no evalua un lote vacio`
- `resolucion.composicion[{i}]: {e}` — `{e}` is a rejection of the form of the values of its
  `hashPrueba`, its `cuenta` or its `posicion`
- `resolucion: la composicion NO es la del recibo: su huella no es su hashPrueba`
- `resolucion del lote: lleva UNA de tres, acuses, sobre o data`
- `resolucion.acuses: no es una lista` · `resolucion: {a} acuse(s) para un lote de {k}: el lote se aplica ENTERO o no se aplica` — afterwards, each acknowledgement by those of the resolution by acknowledgement
- `{que}: {e}: el rechazo de un lote nombra su operacion (§611)` · `{que}: la operacion {j} no esta en un lote de {k}` — `data.operacion`, read with `u64_de`: `{e}` is its rejection of the form of the values
- `resolucion.data: falta causa` · `resolucion.data: falta campos` · `resolucion.data.campos: {e}` — `{e}`, the rejection of `u64_de` of the field that names what is repeated (`index` or `position`); `resolucion.data: falta causa` is also the declared one's
- `resolucion del lote con la causa {causa} y sin sobre: no es de forma ni de las que el RFC-0007 dejo sin prueba portable: se exhibe su sobre de rechazo`
- `RECHAZO SIN FUNDAMENTO: el nodo dijo {causa} en la operacion {j} del lote, y la composicion que su recibo firma no lo sostiene: {por_que}. El juicio se repite sin el nodo; su negativa es su palabra, en su data (D3)` — **NAMED ROJO** (RFC-0014, decision 3), with `{por_que}` one of these:
  - `la operacion {j} lleva la {que} {v} y la causa nombra la {valor}` · `ninguna operacion anterior a la {j} lleva la {que} {valor}` — here `{que}` is `cuenta` or `posicion`
  - `{causa} no es una causa de la forma del lote` · `no hay operacion {j} en un lote de {k}` — unreachable today: `juzgar_forma` is only called with a cause `campo_de_forma` already recognized and with a `j` that `operacion_nombrada` already bounded below `k`. They enter because the census is by call.

*The pledge* (verdict 6, §613)

- `resolucion: falta sobre (el de la prenda, 2.10)` · `resolucion: el sobre no es de tipo prenda`
- `resolucion.sobre: {e}` — `{e}` is a rejection of the reading of the statement and the proof
  (`enunciado_de_prenda`): `falta enunciado`, `falta prueba o no es cadena 0x` or one of the form
  of the values
- `resolucion: la prenda es de OTRA prueba: su digest no es el hashPrueba del recibo`
- `resolucion de la prenda: lleva UNA de tres, consumo, respuesta o rechazo`
- `resolucion.sobre: falta cabeza` · `resolucion.consumo: falta cabeza` — the accepted one; its
  window is measured with the consumption's head
- `resolucion.consumo: la marca NO esta bajo el consRoot de su cabeza`
- `resolucion.respuesta: no es una negativa (accepted false): una prenda aceptada se resuelve con su consumo`
- `resolucion.respuesta: la negativa lleva causa en su data: se exhibe su sobre de rechazo (resolucion.rechazo)`
- `resolucion: falta juzgada (la cabeza que el nodo juzgo: la ultima firmada al recibir, de indice era - 1)`
- `resolucion.juzgada: formatVersion {version}: la prenda se juzga contra una cabeza v5 o v6` —
  the set is derived by `VersionCabeza::texto_con_parametros()`
- `resolucion.juzgada: su indice {sj} no es el de la cabeza que el nodo juzgo: la ultima firmada al recibir es la de la era menos uno ({e1})` — `{e1}` is `era − 1`
- `RECHAZO SIN FUNDAMENTO: el sobre de la prenda VERIFICA contra la cabeza que el nodo juzgo (indice {sj}), con el mismo juez y sin el nodo, y el nodo dijo que no: {dijo}. Su negativa es su palabra, en su respuesta (D3); si alguien la inventara, el operador la desmiente con el PAR` — **NAMED ROJO** (RFC-0014, decision 3); `{dijo}` is the `reason` of the response, or `(sin razon)`
- `resolucion.rechazo: el consumo que la capa rechazo no es la marca de este sobre`

*The declared one* (verdict 4)

- `resolucion: falta data (la del rechazo declarado)` · `resolucion.data: falta causa`
- `resolucion declarada con la causa {causa}, que no es de las que el RFC-0007 dejo sin prueba portable: se exhibe su sobre de rechazo, no se declara`

*The fourth state* (RFC-0010 D-G; since §682 also RFC-0019 E2)

These three are not a rejection: they are the **fourth state**, «resolution declared, not
proven». They carry the prefix `DECLARADA, NO PROBADA: ` —the constant `DECLARADA_NO_PROBADA`—,
and the command prints them **without** `ROJO: ` in front and exits with `3` (`codigo_de_salida`
and `main`; section 6): neither VERDE nor ROJO.

- `DECLARADA, NO PROBADA: el operador declara {causa}, una causa que el RFC-0007 dejo sin prueba portable: se cuenta aparte (RFC-0010, D-G)` — the declared one, with `{causa}` one of `CustodianSetExhausted`, `PendingTreeExhausted`, `NotTheIssuer` and `NotTheAccountHolder`, tied to the receipt by its `data.recepcion`. ⚠️ §815: and `ProofFailed` and `VerificationFailed`, since kit 0.4.3: six
- `DECLARADA, NO PROBADA: el operador declara {causa} en la operacion {j} del lote, una causa que el RFC-0007 dejo sin prueba portable: se cuenta aparte (RFC-0010, D-G)` — the batch resolved by `data`, with the same four. ⚠️ §815: the same six, since kit 0.4.3
- `DECLARADA, NO PROBADA: {que}: el operador resuelve por StaleState. La causa se sostiene sobre el estado comprometido (cabeza de indice acreditado {s}), pero su atadura a ESTA operacion -los publicInputs del recibo del rechazo y su data- no va firmada: es la palabra del nodo (RFC-0019 E2)` — since §682 a `StaleState` does not resolve on its own, neither on the direct path nor in the batch (`{que}` is `resolucion.sobre`); it comes out after the rejection envelope verifies by its rules, so that a malformed one is still ROJO for its own reason

**The anchor** (§592, RFC-0012 E3)

- `falta cabeza (la firmada que el ancla compromete)`
- `un sobre con contraria no lleva ancla: la vista dividida se demuestra con las dos cabezas solas`
- `camino sin ancla: no hay nada que extender`
- `cabeza: formatVersion {version}: el ancla lee cabezas v3, v4, v5 o v6: la pareja del MMR
  viaja firmada desde ellas` · `contraria: formatVersion {version_o}: el ancla lee cabezas v3, v4,
  v5 o v6: la pareja del MMR viaja firmada desde ellas` — the set, derived from its producer
  (`VersionCabeza::texto_con_mmr()`); the second one, with the `contraria` of the split view as
  subject, is another call, and since §776 its own entry
- `el ancla no declara v 1: este binario lee ancla v1`
- `el ancla es de OTRA clave: su clave no es la huella de la publicKey de la cabeza`
- `el ancla no ES esta cabeza: su {campo} no casa`, with `{campo}` one of `indice`,
  `epochDigest`, `mmrRoot` and `mmrSize` — one vector per field
- `el ancla del genesis (mmrSize 0) no tiene historia que extender: se compara entera, sin camino`
- `el ancla declara un indice ({a}) que no es ANTERIOR al embebido de la cabeza ({b})`
- `la cabeza (t={t}) NO extiende el ancla (t={a}): historia bifurcada, recortada, o camino que
  no es el suyo`
- ⚠️ §712, after the consistency of mode 3, the clock of the extension with its cause:
  `extension: mismo mmrSize y digest distinto - el ancla y la cabeza firman
  la misma historia de {t} cabeza(s) y no son la misma cabeza: una bifurcacion firmada, no una
  extension`. The one of `el indice embebido no crece` cannot come out here: the anchor's
  `indice` is already required to be EARLIER, before consistency.
- `los indices embebidos son DISTINTOS ({a}, {b}): dos firmas con su indice propio no dividen
  la vista`
- `contraria: {e}` — the embedded index of the contrary one, read with `indice_de_firma`; `{e}`
  would be the library's `FirmaSinIndice`. Unreachable today: it comes after
  `cabeza_v3_verificada` over the contrary one, and if its `verificar_cabeza` passed, it already
  read that index from the same bytes without error. It enters because the census is by call.
- `las dos cabezas son LA MISMA: no hay vista que dividir`
- `las cabezas llevan claves DISTINTAS: la continuidad es de UN firmante` — the extension's,
  letter for letter. ⚠️ §648: here **`actas` is not read**: the split view is of ONE key. Two
  keys of an operator with the same embedded index are an overlap, and its envelope, which does
  not need the second head —one of the old key above the `desde` of its successor is enough, with
  the chain—, is that of section 2.14 (§686). Nor is mode 3 with an anchor of the key that left:
  that seam is the medium's (RFC-0015, E6)
- ⚠️ §714, mode 5, the fork:
  - `un sobre con rama no lleva ancla ni contraria: la bifurcacion se demuestra con las dos
    cabezas solas`
  - `rama: no es una cabeza (objeto)` — a `"rama": null` or a string
  - `rama: {e}` — `{e}`, the rejection of `u64_de` of the `formatVersion` of the branch, before
    `available`: `rama: falta formatVersion o no es cadena 0x`, if it is missing or is not a string
  - `rama: formatVersion {version}: el ancla lee cabezas v3, v4, v5 o v6: la pareja del MMR viaja
    firmada desde ellas` — the head's, with `rama:` as subject, and before `available`; and what
    is demanded of a head, also with `rama` as subject: `rama: la cabeza no era
    available:true`, `rama: cabeza: …`
  - `las cabezas llevan claves DISTINTAS: la continuidad es de UN firmante`, and with `actas`, `las
    cabezas llevan claves DISTINTAS y las actas no las unen: {regla}` — the extension's, letter
    for letter: here the `actas` **are** read, because the fork is the operator's and not a key's
  - `bifurcacion: mmrSize distinto ({t}, {t_r}) - dos cabezas en sitios distintos de la historia no
    la bifurcan: el orden entre ellas lo juzga la extension`
  - `bifurcacion: las dos cabezas son LA MISMA (mmrSize {t} y el mismo digest): no hay bifurcacion`

What the envelope demands of a head —`available:true`, the recomposition of the digest, the
signature— and the reading of the `camino` are the extension's, with `cabeza` or `contraria` as
subject —and since §714, `rama`—: they live in its block and are not repeated here.

**The co-signed anchor** (§633, RFC-0013 E4a)

- `el sobre del ancla cofirmada no lleva {ajeno}: el ancla se deriva de la cabeza, y la
  extension y la vista dividida son del sobre del ancla`, with `{ajeno}` one of `ancla`,
  `camino` and `contraria`
- `falta cabeza (la firmada cuya ancla se publico)`
- `falta {campo} o no es cadena ({que})`, with `{campo}` one of `nota` and `publicador`
- `falta inclusion o no es lista (el camino del ancla a la raiz)` · `inclusion: {e}` ·
  `inclusion: {} bytes, se esperaban 32`
- `testigos no es lista (las vkeys de los testigos)` · `testigos: {e}` · `testigos: {nombre}
  esta dos veces`
- `cabeza: formatVersion {version}: el ancla lee cabezas v3, v4, v5 o v6: la pareja del MMR
  viaja firmada desde ellas` — the anchor envelope's, letter for letter
- `publicador: {e}` — the vkey cannot be read; `{e}` is the error of `zk-ssl-medio`
- `el publicador es {nombre:?} y el medio de esta cabeza es {origen:?}: el origin lleva la
  huella de la clave XMSS (RFC-0013 D-A)`
- `la nota: {e}` — the note does not verify with its publisher; `{e}` names the rule of
  `zk-ssl-medio::nota` that falls (malformed, non-canonical checkpoint, foreign origin, no
  publisher signature, repeated signature or invalid signature)
- `la nota lleva dos cofirmas de {nombre}` · `la cofirma de {nombre} no verifica: {e}`
- `el ancla de esta cabeza no esta en la posicion {p} de las {n} de la nota: {e}`

`posicion` is read with `u64_de`: its rejections are those of the form of the values.

**The overlap** (§686, RFC-0015 E5c)

- `falta cabeza (la firmada que se juzga contra su tramo)`
- `falta actas (la cadena de zkssl_keyActs: sin ella no hay tramo que cruzar)` · `actas no es
  lista (la cadena de zkssl_keyActs)` · `actas[{k}]: {e}` — those of 2.3, letter for letter
- `cabeza: formatVersion {version}: el sobre del solapamiento lee cabezas v3, v4, v5 o v6: las
  que firma un nodo con actas`
- `el acta {i} de la cadena no vale: {regla}`, with `{regla}` one of those of `verificar_acta`
- `la clave de la cabeza no esta en la cadena: nadie la comprometio`
- `la cabeza firma en la hoja {e}, dentro de su tramo {tramo}: no hay solapamiento`, with
  `{tramo}` `(acta 0, la genesis: cualquier hoja salvo la {d} de su acta[, por debajo de la {h}])`
  or `(acta {i}: por encima de la hoja {d}[, por debajo de la {h}])`. ⚠️ §722: only if its
  `mmrSize` also falls in the handover of that stretch (2.14). If not, the VERDE of the
  detection: `VERDE: SOLAPAMIENTO - la clave firmo una cabeza con el mmrSize {t}, fuera del relevo
  de su tramo: …`, with the `3/3` line that names the stretch and the handover.

What the envelope demands of a head —`available:true`, the recomposition, the signature— is the
extension's, with `cabeza` as subject.

## 6. The contract of the command

- **Invocation:** `zk-ssl-verify <paquete.json>` — **one** argument, the file's path. It is the
  binary's only disk read; there is no network, no clock, no telemetry (§395 gates it).
- **Standard output:** the numbered lines of its form —`1/3` · `2/3` · `3/3` in the position
  packages (or `3/3 sin acuse en el paquete: la cabeza sola queda demostrada`), the extension,
  rejection, age, pending claim, payment in flight, pledge and anchor ones; `1/5` to `5/5` in
  the
  consumption one; `1/4` to `4/4` in the conflict one and in the co-signed anchor one—, the
  co-signatures line when the envelope is v2, and at the end the VERDE of its form, one of these
  sixteen —fifteen until §714—; the six from the fourth to the ninth continue on a second
  line, the claim's reaches a third, and the payment's and the pledge's a fourth:
  - `VERDE: el paquete se sostiene sin el nodo`
  - `VERDE: la extension se sostiene sin el nodo`
  - `VERDE: el consumo se publico entre las dos cabezas, sin el nodo`
  - `VERDE: dos libros aceptaron el mismo consumo. Es DETECCION, no prevencion:`
  - `VERDE: {causa} se sostiene sobre el estado comprometido. Dice que la regla se`
  - `VERDE: bajo la cabeza de seq {seq}, a lo sumo {k} posiciones vivas {quien}`
  - `VERDE: bajo la cabeza de seq {seq} hay un pendiente a nombre del receptor,`
  - `VERDE: bajo la cabeza de seq {seq} hay un pendiente a nombre del receptor por`
  - `VERDE: bajo la cabeza de seq {seq} hay un pendiente que solo puede cobrar quien`
  - `VERDE: el recibo se resolvio dentro de la ventana, y se sostiene sin el nodo` (§573)
  - `VERDE: el ancla se deriva de la cabeza firmada, y se sostiene sin el nodo` (§592; the two
    previous lines carry the derived anchor and its fingerprint, to publish them)
  - `VERDE: el ancla ES esta cabeza firmada, y se sostiene sin el nodo` (§592)
  - `VERDE: la cabeza extiende el ancla: la historia anclada es un prefijo, y se sostiene sin el nodo` (§592)
  - `VERDE: VISTA DIVIDIDA - la clave firmo DOS cabezas con el indice embebido {i}. Es
    DETECCION del operador: dos historias, y solo quien tiene la clave pudo producirlas` (§592)
  - `VERDE: BIFURCACION - el operador firmo DOS cabezas distintas con el mmrSize {t}: bifurcacion
    o reinicio sin diario. Es DETECCION del operador: un nodo con su diario no repite un mmrSize,
    y solo quien tiene la clave pudo firmarlas` (§714; a single line)
  - `VERDE: la cabeza estaba publicada en el medio {origen}, en la posicion {p}, y la cofirman
    {k} testigo(s) con clave en el sobre. Que testigos valen y cuantos hacen falta lo decide
    quien verifica (RFC-0013 D-D): este mando reporta, no juzga` (§633; a single line, and
    before it one per witness: `testigo {nombre} clave sha256:{hex} marca {t}`)
- **Error output:** `ROJO: {motivo}` with a text from the catalogue of section 5, and it stops.
  Since §693 the reason names a cause of section 6.1, and goes on a single line.
- **Four exit codes:** `0` green · `1` the first failure by name · `2` usage (no argument, or
  more than one; it prints the usage on the error output) · `3`, the fourth state, «declared,
  not proven», which is printed with its name and without `ROJO`, because it is not one: since
  §573, that of the completeness envelope; and since §723, also that of the `InsufficientBalance`
  rejection envelope whose band is of v1 with the upper bound below `2^32 - 2` (section 2.6).
  A completeness envelope that resolves with that rejection says so with its place, and after its
  own checks: if it names another operation of the batch or another mark of the pledge, it is `1`.
- **A panic is not an exit code** (§711). If the command panics while verifying, it does not exit
  with `101`: it prints `ROJO: el verificador entro en panico: panico recogido al juzgar una prueba
  ajena: {mensaje}` and exits with `1`. It is a structural net: what the winterfell fork validates
  comes out earlier, with its cause (section 6.1), and the net covers what was not validated. On
  `wasm32-wasip1`, which does not unwind a panic, the net catches nothing, and only the fork
  protects.
- **What an implementation does not read** (§693). An implementation that does not read one of
  the forms says so with the reserved text `NO SOPORTADO`, on its error line —`ROJO: NO SOPORTADO:
  {qué}`—, and exits with `1`: it fails closed, never `0`. It is not a verdict on the envelope,
  but on itself, and the harness never counts it, neither as a hit nor as a negative passed
  (section 6.1). What it does not read it declares separately, in a list, as
  `tools/segunda/verificador.py` does. The reference reads the fourteen forms and never prints
  it.
- **What is promised** (§801). This contract is surface (6) of the protocol's promise, and the
  forms of section 2 are (4): `RPC.md` §«Versionado y compatibilidad» [versioning and
  compatibility] says what breaks them —an exit code that changes, a `tipo` that is removed or
  renamed, a `v` that is no longer read—, what pays for it —a new `v`, a new `tipo`, or a `0.N.0`
  kit with the wire— and what keeps verifying afterwards: an envelope a kit gave VERDE is still
  given VERDE by the kit of its version, and a later one either verifies it or rejects it in the
  «outside the contract» class of section 6.1. `tools/check_superficie.py` compares in every
  canon run the codes of `codigo_de_salida` and of the usage with those of this section, and the
  `tipo`s and the `v`s the binary reads with those of section 2.

### 6.1 The cause of the rejection (§693)

The `{motivo}` of the ROJO names **one cause**. The table groups those of the catalogue of
section 5 into thirteen classes, read from the innermost cause to the outermost —in `nueva:
cabeza: la firma no verifica` the cause is the signature, not the new head—, so that two
implementations fail **for the same one**: it is the criterion of H3, «that it passes and fails
the same way» (RFC-0005, E5). A cause is named with the text the house governs. What follows it
and is put there by a library —the `VerificationFailed` of `xmss`, the JSON reader's message— is
detail, and is not pinned. In a STARK proof the name is put by `winter-verifier` 0.13.1, the
version of the `Cargo.lock`: that of the variant of its `VerifierError`.

| cause | what names it on the ROJO line |
|---|---|
| **unreadable** — the file cannot be read, or is not JSON | `no se puede leer` · `JSON ilegible` |
| **outside the contract** — a version or a type the command does not read | `el paquete no declara su version` · `el paquete declara v:` · `tipo desconocido` · `version {cv} desconocida` · `kind desconocido` · `el ancla no declara v 1` · `marca v1: prueba de zkssl/0.4; la juzga el kit 0.4.x` and `marca: meta vacio, prueba anterior a zkssl/0.4; la juzga el kit de su version` (§914: a proof of another version of the wire, which this kit does not judge; R6 e of RFC-0021 and D-E of RFC-0018) |
| **field missing, extra or badly written** | `falta {campo}` · `no es cadena` · `no es lista` · `no booleano` · `sin 0x` · `hex impar` · `hex: cifra no admitida` · `cantidad hex no minima` · `bytes, se esperaban 32` · `no es canonico` · `no lleva {ajeno}` · `no es una cabeza (objeto)` (§714: the anchor's `rama` that is not an object) |
| **head outside its family** | `no era available:true` · `formatVersion {v}:` · `exige una cabeza v5` · `exige cabezas v4` |
| **recomposition** — the fields do not give the signed digest | `NO recomponen` |
| **invalid signature** | `la firma no verifica` · `la firma es VALIDA pero de otro mensaje` · `el indice declarado ({d}) no cuadra con el que va dentro de la firma` |
| **path that does not climb** | `RaizDistinta` · `el camino NO sube` · `NO extiende` · `el isRight recibido NO es el de la posicion` · `no tiene los {n} niveles` |
| **does not match what is committed** — another head, another operator, another key, another datum | `OTRA cabeza` · `OTRO operador` · `claves DISTINTAS` · `la MISMA clave` · `no es el comprometido` · `no es el pendingRoot de la cabeza` · `no esta en un lote de` and `no es la marca de este sobre` (§723: the operation the batch does not have and the consumption that is not the pledge's mark, in a resolution by rejection, 2.11; with a v1 band below the bound, they also come out before the fourth state) |
| **the rule does not hold** — the cause, the window, the stretch, the order, the statement | `la causa NO se sostiene` · `NO RESUELTA EN LA VENTANA` · `SOLAPAMIENTO` · `no es anterior a la cabeza de seq` · `pasan del techo` · `mismo mmrSize y digest distinto` and `el indice embebido no crece` (§712: the clock of the extension, the consumption and the anchor of mode 3, 2.3) · `bifurcacion: mmrSize distinto` and `bifurcacion: las dos cabezas son LA MISMA` (§714: mode 5 of the anchor, 2.12) · `fuera de 3..=` (§717: an `m` of the age over its cap, `m = {} fuera de 3..={M_MAX}`, 2.7; pinned with its `m` and without the cap) · `no es un indice de cuenta` and `todos se escribe con emisor 0` (§718: the age's issuer, 2.7) · `pasa del techo` (an integer of the payment or the claim over its ceiling; since §718, also as it arrives, before reducing, 2.9 and section 5) · `RELEVO` and `no hay solapamiento` (§722: a head outside the handover of its stretch in the eight places of 2.3, and one inside its stretch and its handover in the overlap envelope, 2.14) |
| **malformed proof** | `la prueba no se deserializa` · `forma de traza` · `marca desconocida`, `marca: la prueba es de la familia …`, `marca: version …`, `marca: perfil …`, `marca: m = …` and `marca: … columnas` (§914, RFC-0018 D-G: the v2 mark, before anything else; those of another version of the wire go above, in «outside the contract»; until §914, `marca de la ocultacion`, §706, the v1 mark, part of the form since §651) · `prueba mal formada: {qué} fuera de rango`, with `{qué}` one of `opciones`, `traza`, `contexto`, `consultas`, `marco OOD`, `FRI` and `lote` (§711; put there by the winterfell fork, `crates/winter-air/src/causa.rs`, and the `lote` one by `zk-ssl-air/src/sal.rs`, inside `la prueba no se deserializa`) · `prueba mal formada: usize no canonico` (§715: a `usize` of the proof that is not in its minimal encoding; put there by the fork's reader, `crates/winter-air/src/proof/acotado.rs`, with the text of `causa.rs`) · `UnconsumedBytes` and `not all bytes were consumed` (§715: leftover bytes after the proof, or inside an opening or the batch of an FRI layer; they are the same winter-utils error, `DeserializationError::UnconsumedBytes`, by its `Debug` and by its `Display`) |
| **options not accepted** | `UnacceptableProofOptions` · `InsufficientConjecturedSecurity` · `InsufficientProvenSecurity` |
| **inconsistent OOD** — the out-of-domain evaluations do not match the constraints: the proof does not prove that statement, which in the vectors is a lied one or, since §717, a fabricated proof that only has the form | `InconsistentOodConstraintEvaluations` |
| **query that does not match the commitment** | `TraceQueryDoesNotMatchCommitment` · `ConstraintQueryDoesNotMatchCommitment` · `FriVerificationFailed(RemainderCommitmentMismatch)` (§713: the FRI remainder that is not the committed one; put there by the fork's channel, `crates/winter-verifier/src/channel.rs`) · `FriVerificationFailed(LayerCommitmentMismatch)` (§715: an extra group in an FRI layer; put there by the fork's channel, which demands as many groups as folded positions, and since §715 an extra row of the trace or of the quotient also gives the table's variant) |

Any other variant of `VerifierError` names its cause by its name, like these. `NO SOPORTADO` is
none of them: it says something about the implementation, not about the envelope. If
`winter-verifier` renames a variant, what the command prints changes, and the manifests see it:
the line that pins it moves to its version, with its vector, and is not rewritten (rule 2 of the
PROCESO).

**What a manifest pins.** A negative pins a text that names the cause. The subject alone
—`edad:`, `cabeza: `, `nueva: cabeza: `— does not name it, and whoever verifies nothing passes it
too: `tools/segunda/verificador.py`, which does not read the pledge envelope, passed the two
negatives of the lied statement with its ROJO of an unread envelope. The lines that pinned the
subject alone are not rewritten; since §693 each has next to it another, new one, that pins the
cause (section 9).

**What the harness demands** (`tools/conformidad.sh`, since §693):

- an output that says the binary does not read the envelope comes out
  `ROJO <fichero>: NO SOPORTADO - <la línea que lo dice>` [the line that says so] and never counts,
  whatever the manifest says. The reserved text is `NO SOPORTADO`; «no lee este sobre», «no
  implementado» and «not implemented» are recognized as a courtesy, and they are not all the ways of
  saying it: another —«sin implementar»— comes out as one more ROJO, and its vector does not pass
  because it does not name the cause;
- in an entry with code `1`, the output has **one** line with `ROJO:`, and the manifest's text is
  in it;
- a vector with entries of code `1` has at least one that names the cause: an empty text, or one
  that ends in `:`, is the subject alone, and the vector comes out `SIN CAUSA`;
- the binary runs with standard input empty: one that read it swallowed the manifest.

## 7. Who assembles the package

Two forms have a producer in the tree, and it is the **node**, in an out-of-band mode that opens its
ledger with the server STOPPED and copies the signed head VERBATIM: `zk-ssl-node --prueba-edad`
writes the age envelope (§466), and `zk-ssl-node --prueba-rechazo` the rejection one for
`AccountFrozen` (§473), `AccountNotFound` (§474) and `InsufficientBalance` (§478); any other cause
it refuses, naming it. `tools/banco_edad.sh` and `tools/banco_rechazo.sh` demonstrate them live. Of
the rest, **no command in the tree emits the package**: the witness and the cli serve and keep the
responses of the wire, and whoever gathers them in the envelope is the holder — in the tree, the
benches `tools/banco_apagado.sh`, `tools/banco_completo.sh`, `tools/banco_evidencia_v2.sh`,
`tools/banco_extension.sh`, `tools/banco_consumo.sh` and `tools/banco_dos_libros.sh`, which capture
the responses of a real node and wrap them without rewriting a field; the vectors of the other
causes of the rejection were gathered from captures of a real node, and section 9 says which. That
is the contract: **gather, not recompose**, and the node's mode fulfills it. A command that
assembled the package is a front of its own and does not change this document: it would change who
writes the envelope, not the envelope —and so it was in §466 and in §473—. **The pending claim one
is produced by the CLAIMANT** (§497, RFC-0008 D-M and D-P): with his v2 notice and his credential,
the cli's front end, `zk-ssl-cli prueba-cobro`, asks a LIVE node for the signed head and the
snapshot, demands that they are of the same heartbeat and writes the envelope with the head
VERBATIM; the bench that reproduces it live and its vectors belong to E4 (section 9). **The overlap
one is assembled by the WITNESS** (§687), from its journal and without the node:
`zk-ssl-cli witness --auditar DIARIO --sobres DIR` (2.14), and with the same command, since §688,
the split view one (2.12, mode 4).

## 8. What this document does NOT claim

- That an independent verifier exists **does not make the signatures enforceable against third
  parties**: the declared custody of the operator's key is still missing (`SECURITY.md`). This makes
  verifying possible; it does not make what is verified valid.
- That the co-signatures verify does not say that they are enough: the policy is the client's
  (section 1).
- Nothing about the content of the position: the package proves **that** the entry is acknowledged
  under that signed head, not **what** it says.
- The `index` of the head is only bounded **from below**. The signature attests the leaf index it
  carries inside and the binary demands that the declared one be greater (§399); an envelope that
  declares more than it signed is not rejected, and what the command prints as «indice de firma»
  [signature index] is the embedded one, not the declared one.

- The window in which the consumption is proven is the one **the operator himself ordered with his
  MMR**: it is not clock time, and he chooses which heads he signs and when. Inside a ledger single
  use is a checkable invariant; **between ledgers this package says nothing** —that is detection,
  not prevention, and it lives in RFC-0006 E4—.
- ⚠️ §722. The extension between two keys with `actas` **does not prove a history of STATE**: it
  proves that the two heads fit in a history of heads with that chain and that handover. Two
  operators in agreement fabricate it (2.5), and it is given away by a head of the key that left
  above the handover (2.14), if it exists and someone keeps it. **Nothing gives it away** if they
  alternate the ledgers instead of shutting one down —each key signs only in its turns, with
  untouched nodes—, if they pad the MMR and jump the counter at the same time, or if the operator
  signs another act of the same transition with another handover (2.5). That last one is an
  equivocation of the chain —two valid acts of the same key after the same previous one— for which
  there is no envelope yet.
- ⚠️ §722. The handover is judged in the eight places of 2.3 —the extension, the consumption and the
  six heads of the completeness closing (2.11)— and in the overlap. **Not** in the anchor's fork
  (2.12), where two heads at the same `mmrSize` on the two sides of a handover are the fork itself,
  and have to give it away. The witness does not apply it yet either: neither on seeing a rotation,
  nor in `--auditar`, nor in `--comparar`. And with a key that returns to the chain, the later one
  is judged against the last stretch of its key (2.3).
- ⚠️ §722. The handover **judges what the genesis signed before opting in** —D-I of RFC-0015 exempts
  it by index, not by `mmrSize`— and **the rotations from a restored journal** behind what the old
  one signed. Both give away an operator that did not lie: D-G asks to rebuild the journal before
  rotating, and the node demands it with `--relevo-minimo` (2.14).

## 9. Vectors and gate

The package's vectors live in `spec/vectors/paquete/` (stage E2 of RFC-0004, sealed in §398; E3 in
§399): one positive per form —v1, v2, v2 without acknowledgement, v2 with zero co-signatures,
extension— and **one negative for each rule of section 5 that can be produced from a real package**,
derived by mutation of two captures of the benches. The two negatives of the index
(`rechazo-index-atrasado`, `rechazo-index-cero`) came in with their rule in §399.
The out-of-the-set one moved from `rechazo-formatVersion-5` to `rechazo-formatVersion-6` in §451
(RFC-0007 E1a, with v5 inside the set): the `-5` is still listed and falls for another cause, with
the measured text.
`MANIFIESTO.txt` says, for each file, the exit code and the text the binary has to emit. **The
harness `tools/conformidad.sh <binario>`** (RFC-0005, E4, §408) runs the whole manifest against any
binary that meets the contract of section 6 —the reference one or that of a second implementation—
and says, entry by entry, whether the exit code and the text are the manifest's; it is the only
producer of that loop. `tools/canon.sh` runs it in every canon over the reference binary and goes
red if a single vector does not say what the manifest says, or if a vector without an entry appears.
A tampered nibble in any of them turns the canon red.

Four texts of the catalogue **had no vector**, and they were declared: `{campo}: {e:?}`,
`sibling {i}: {e:?}`, `camino[{i}]: {e:?}` and `{cual}: siblings[{i}]: {e:?}` demand 32 bytes that
`digest_from_bytes` rejects, and no value that did so was known. ⚠️ **There was none**: the reading
reduced modulo `p`, and the only way to fail was the length, which the reader had already judged
before. **Since §640 (RFC-0016) the value exists**: any element written as `x + p`, and the text
becomes `{e}`, the rule with its name. `{campo}: …` has a vector, the zero of the `chainDigest`
written as `p` in `ancla/neg-ancla-cero-escrito-como-p.json`, and two of `u64` live here:
`rechazo-n-no-canonico` and `rechazo-mmrSize-no-canonico`. The three of the path still have no
vector: it takes a sibling with an element below `2^32 - 1` to write it as `x + p`, and the captured
paths do not bring one; the rule has a witness in the tests of the command and of the core. **Since
§431 «las cabezas llevan claves DISTINTAS» does have one**: the two-ledger bench produces that
envelope with its defect ISOLATED —the two heads are v4, both recompose their digest and both
signatures verify—, and its vector lives in the consumption family, which is that of the envelope
that carries it.
**And since §422 the rejections of the consumption envelope DO have them, and since §431 those of
the conflict envelope**: `spec/vectors/conflicto/` brings the positive and one negative for each
producible rule of its family, derived by mutation of the captures of the two-ledger bench; and
`spec/vectors/consumo/` brings the positive and one negative for each producible rule of its family,
derived by mutation of the captures of the consumption envelope's bench (RFC-0006, E3), each with
its entry in `MANIFIESTO.txt`; the same harness runs them in every canon with another manifest. The
`#[test]`s of `crates/zk-ssl-verify/src/consumos.rs` still falsify the pure rules —the empty leaf,
the convention and the cross-check— without needing signatures.
**And since §455 those of the rejection envelope** (RFC-0007, E3a-1): `spec/vectors/rechazo/` brings
one positive per proven cause —four— and one negative for each producible rule of its family,
derived by ONE mutation of the captures of a real node (the PASTE-455-M; its bench does not live in
the tree yet, and this is declared). One text of the family has no vector: «nextIndex no alcanza el
tope» demands parameters that recompose with a cap above `nextIndex`, and a single mutation of the
real does not reach it. **Since §456 the catalogue also covers `StaleState`, `WrongRegulatoryLimit`
and the two batch duplicates**, with a real receipt captured by the proxy of a bench (the SDK's
`e2e` example), one positive per cause and one negative per rule producible by a single mutation.
**Since §459 it covers `AccountFrozen`** (RFC-0007, E3b): a positive gathered from the captures of a
real node that freezes by the delegated route (`dev_freeze`, the PASTE-R7E3b-M) and nine negatives.
The disguise is not a mutation: it carries the REAL path of a free account, with the empty leaf
under the same root. The attack of the truncated path with a matching root is not fabricated in
JSON: it is falsified by the witness of `crates/zk-ssl-verify/src/congelados.rs`.
**Since §460 it covers `SupplyCapExceeded`** (RFC-0007, E3, which is left whole): a positive
gathered from the captures of a real node with a small supply cap (`--max-supply 1000`, the
PASTE-R7SC-M) and seven negatives. Two are not mutations, and this is declared: the previous head is
the REAL SIGNED one from before issuing, and the cap not exceeded is a scene of two fields —the
amount and the `wouldBe` at once—, because a single mutation of the real does not reach that rule.
**Since §476 it covers `AccountNotFound`** (RFC-0007, E5, cut 3b, and it is the first cause of E5
with a catalogue): a positive gathered from the captures of a real node that produces the envelope
with `zk-ssl-node --prueba-rechazo` (`tools/banco_rechazo.sh`, the CAPTURAS-475) and **eight**
negatives, one per producible rule. Two things are declared. The **disguise** —the node says the
account does not exist and the leaf under `accountsRoot` IS occupied— **cannot be produced**: it
would demand the real path of a live account, and D-G of RFC-0007 closed the door to a method of the
wire that serves it; its witness is PURE and lives in the `#[test]`s of
`crates/zk-ssl-verify/src/cuentas.rs` (§475). And of the four mutations the bench falsifies live,
two —the leaf and a sibling of the path— fall by the SAME rule: the catalogue pins the rule, not the
mutated piece, so only one enters as a vector.
**Since §479 it covers `InsufficientBalance`** (RFC-0007, E5, cut 4c, which closes E5): a positive
gathered from the captures of a real node that produces the envelope WITH its band proof
(`tools/banco_rechazo.sh`, its third arm, over the bench's free account and an amount derived from
the funding; the CAPTURAS-4c) and **eight** negatives, one per rule of the arm. Two are scenes, and
this is declared: the zero request and the ceiling touch both `requested` at once, because touching
only one falls earlier by the different amount. The red of the other account is written by
winterfell, and the manifest pins the name of its variant (0.13.1, fixed in the lock). Without a
vector, and declared: a corrupt proof falls by the SAME rule as the other account, and a head with
another `accountsRoot` and the same `seq` is not fabricated without breaking the signature. **Since
§723, the v2 band** adds FOUR vectors at the end of the manifest, with their cause: the bench's
capture with v2 (positive); the envelope of an operator that signs by hand, with its own test key,
an accounts root of one leaf with the balance in the window `[2^62, 2^63)` and the v1 proof (exit 3,
and VERDE with the earlier kit); the same leaf with v2 and a malicious prover (ROJO by the OOD
coherence); and a control of the same form with an honest leaf (positive). The positive of §479 is
of v1 with the upper bound 1000 and stays VERDE with its line: the judge recognizes that proof by
the fingerprint of its content and its statement, in a closed list (`PUBLICADAS_V1`), and no other.
The three of the operator are not reproduced byte for byte.
**Since §467 it covers the AGE envelope** (RFC-0007, E4b-3, which closes E4): `spec/vectors/edad/`
brings TWO positives GATHERED from the captures of a real node -the layer produced them with
`zk-ssl-node --prueba-edad` over that node's ledger, against the v5 head of seq 10 that the node
signed- and NINE negatives by ONE mutation each. The two positives are the two forms the statement
admits today: ALL (`t = 0`) and the EMPTY BOX (`t` above the height, hence `k = 0`). One text of the
family is not pinned whole, and this is declared: that of the statement that does not verify is put
by WINTERFELL and not by the house, so the manifest pins only the prefix `edad:` that the command
prepends. The captured head does not travel as a vector: it goes whole inside the two envelopes, and
its fingerprint is declared in the header of the manifest.
The live demonstrations with a node are `tools/banco_apagado.sh`, `tools/banco_consumo.sh`
(RFC-0006, E3), `tools/banco_dos_libros.sh` (E4a), `tools/banco_edad.sh` (E4b-3),
`tools/banco_rechazo.sh` (RFC-0007 E5, cuts 3b and 4c), `tools/banco_pendiente.sh` (RFC-0008 E4,
claim side), `tools/banco_pago.sh` (RFC-0008 E2), `tools/banco_prenda.sh` (RFC-0008 E3) and
`tools/banco_completitud.sh` (RFC-0010 E5): the third of them brings up TWO nodes with TWO keys and
produces the fact that E4 exists to detect, the fifth produces the rejection envelope over a real
ledger with the server STOPPED, the sixth seeds with the server STOPPED and asks for the claim
envelope through the front end with the server LIVE, the eighth is the only one in which the front
end opens a keystore and the node ACCEPTS the mark under the same heartbeat, and the ninth is the
only one that WAITS: it signs one heartbeat per second until the window of a receipt expires.

**Since §499 it covers the PENDING CLAIM envelope** (RFC-0008, E4 on the claim side):
`spec/vectors/pendiente/` brings TWO positives GATHERED from the captures of a real node —the
claimant's front end (`zk-ssl-cli prueba-cobro`, §497) wrote them with the node LIVE against the v5
head of seq 5 that node signed, over a v2 pending item born at 4— and SEVEN negatives by ONE
mutation each, one per producible rule (D-Q). The two positives are the two forms of D-N: existence
(`inferior = 0`) and the tight band (`inferior = importe`). One text of the family is not pinned
whole, and this is declared: that of the proof that does not verify is put by WINTERFELL and not by
the house, so the manifest pins only the prefix `cobro:` that the command prepends. The head does
not travel as a vector: it goes whole inside the two envelopes; nor do the notice and the credential
of the scenario —the command does not read them— and the manifest declares them by their
fingerprint. Its producer is `tools/banco_pendiente.sh` (§498), which seeds with the node STOPPED
and asks with the node LIVE. **Since §723, v2 of the claim**: the two positives above are of v1,
with the upper bound `MAX_VALOR`, and stay VERDE; TWO more positives, at the end of the manifest,
are the two forms of D-N with v2, COPIED from a run of `tools/banco_pendiente.sh --guardar` with
today's node, so that a kit that only implements v1 does not pass the family. **The payment side
arrived in §509** and its catalogue is `spec/vectors/pago/`, which has its own paragraph here since
§548.

**Since §509 it covers the PAYMENT IN FLIGHT envelope** (RFC-0008, E2): `spec/vectors/pago/` brings
TWO positives GATHERED from the captures of a real node —the payer's front end
(`zk-ssl-cli prueba-pago`, §507) wrote them with the node LIVE, with HIS credential and with
`receiverId` (D-AE, §505), against the v5 head of seq 5 that node signed, over a v2 pending item
born at 4 for 250000 and with delta 96— and SEVEN negatives by ONE mutation each, one per producible
rule, all derived from the frontier positive. The two positives are the two forms of `--t`, which is
ABSOLUTE (D-AI-4): the `seq` of the head and the FRONTIER `nacido + delta` = 100, which is the last
epoch the payment sustains. One text of the family is not pinned whole, and this is declared: that
of `neg-importe-mentido` is put by WINTERFELL —the amount is a PUBLIC input of the AIR, so lying
about it leaves the proof without verifying its statement—, so the manifest pins only the prefix
`pago:` that the command prepends. And here no WEIGHT is pinned: the two positives are of the SAME
pending item under the SAME head and weigh differently —55,317 B and 54,888 B—, because the only
thing that changes between them, `t`, changes the transcript and with it the FRI queries (5.A-324).
The head does not travel as a vector: it goes whole inside the two envelopes (D-J); nor do the FOUR
files of the scenario —the command does not read them— and the manifest declares them by their
fingerprint. Its producer is `tools/banco_pago.sh` (§508).

**Since §546 it covers the PLEDGE envelope** (RFC-0008, E3): `spec/vectors/prenda/` brings TWO
positives CAPTURED from a real node —the pledgor's front end (`zk-ssl-cli prueba-prenda`, §543)
wrote them opening the receiver's keystore (§544) against the v5 head of seq 5 that node signed, and
the second one also published the mark with `zkssl_pledge` under the same heartbeat— and SEVEN
negatives by ONE mutation each, one per producible rule. The two positives assert THE SAME under the
SAME head, so the command's verdict does not distinguish them and the manifest pins them with the
same text: the only thing that separates them is the bytes of the proof. And that is why this family
is CAPTURED and not re-derived —since §538 the prover hides, and two proofs of the same statement do
not weigh the same: four measurements, 68,068 and 66,333 B, 67,145 and 66,398 B—, so the manifest
pins no weight. One text of the family is not pinned whole, and this is declared: that of the proof
that does not verify is put by WINTERFELL and is the SAME for the two fields of the statement, so
the manifest pins only the prefix `prenda:` that the command prepends. The head does not travel as a
vector: it goes whole inside the two envelopes; nor do the five files of the seeding —the command
does not read them—, and the manifest declares by fingerprint the four deterministic ones, not the
`keystore.json`, which carries a nonce. Its producer is `tools/banco_prenda.sh` (§545). A VERDE here
is HALF a pledge (D-AS): the other half is the mark under the `consRoot` of that same head, and it
is requested with `zkssl_consumoPath`.

**Since §574 it covers the COMPLETENESS envelope** (RFC-0010, E5): `spec/vectors/completitud/`
brings THREE positives CAPTURED from a real node that signs one heartbeat per second —a send with a
proof of zeros that the layer rejects leaves its receipt in the `error.data` of the wire, and the v6
head of index 2 closes its era—, one per verdict that is seeded: RESOLVED as a rejection with proof
(exit 0), NO RESUELTA EN LA VENTANA with the current head of index 1443, the first one outside it
(exit 1, the NAMED ROJO), and the fourth state (exit 3), DERIVED from the real `data` with a cause
without portable proof, because the node does not produce those causes by this route today, and this
is declared. And THIRTY-TWO negatives by ONE mutation each, one per producible rule and per PLACE:
those of the distinct keys carry the head of a second node, from another seed. Verdict 1 —resolved
as an APPLIED transition— had no vector until §605: it asks for the real STARK proof of an applied
send, and its head + acknowledgement pair is verified as the position package, with the vectors of
`paquete/`; the rules of that branch's own do have them. **Since §605 it has one**:
`resuelta-por-acuse`, a FOURTH positive copied from ANOTHER producer,
`tools/banco_mentiroso_sin_resolver.sh --guardar` (RFC-0011 E4): a `--dev` node against which the
sdk's `e2e` example pays with a real STARK proof, and the acknowledgement of the entry under the
head that signs it. Those the envelope shares with other families —what is demanded of a head, the
reading of the path, what the resolution re-verifies— live in them, and this is declared. Two runs
of the bench give the same closing except for `emittedAtUnix`, which is not signed: the family is
COPIED from the seal's run. Its producer is `tools/banco_completitud.sh` (§574), with `--largo`
—some 24 minutes— for the expired window. **Since §614 it also covers the `lote` and `prenda`
forms** (RFC-0014, E5): THIRTY-SEVEN more vectors, COPIED from a run of
`tools/banco_recibo_agregado.sh --guardar` against three real nodes —the batch one with the `--dev`
faucet, against which the sdk's `d2_lote_rpc` example pays a batch of two sends with real STARK
proofs; the two of the pledge with their seeding and a real pledge—: ten verdicts —the BATCH
applied, rejected with proof, by its FORM, declared and two RECHAZO SIN FUNDAMENTO; the PLEDGE
accepted, rejected with proof, rejected by the layer and one RECHAZO SIN FUNDAMENTO— and
twenty-seven negatives, one per producible rule and per place. The RECHAZO SIN FUNDAMENTO verdicts
and the fourth state of the batch are DERIVED by mutation of what was captured, because an honest
node does not produce them, and this is declared; the distinct keys and the resolution outside the
window are not produced there. **Since §723, the v2 band** adds FIVE vectors, assembled by an
operator with its own test key -the rejection's head, the v6 closing and a receipt leaf- and a
resolution by an `InsufficientBalance` rejection with the honest v1 band below the bound: on the
direct path, in the batch and in the pledge, the fourth state with the place (exit 3); with the
operation outside the batch, or with another mark in the pledge, ROJO for its own reason. The two of
the pledge copy the pledge envelope of a captured vector: that branch does not judge its proof, only
its digest.

**Since §593 it covers the ANCHOR envelope** (RFC-0012, E4): `spec/vectors/ancla/` brings FOUR
positives CAPTURED from a real node that signs one heartbeat per second, one per mode: the anchor
DERIVED from the head alone —the command prints it and the bench parses it, without recomputing
anything: a single producer—, the EXACT one against its own head, the EXTENDED one —the anchor of
the old head, the path of `zkssl_consistencyProof` and the head that signs the size of that path—
and the SPLIT VIEW, seeded reproducing the real attack: the same seed with a fresh index counter and
a ledger in which an evaluated-and-rejected send moved the signed `recepCount` —two heads with the
same embedded index and distinct digests, which only whoever holds the key can produce—. And
SEVENTEEN negatives by ONE mutation each, one per producible rule and per place: those of the
distinct keys carry the head of a second node, from another seed. The rules the envelope shares with
other families —what is demanded of a head, the reading of the flat path— live in them, and this is
declared. Two runs of the bench do not give the same bytes (`emittedAtUnix` is not signed): the
family is COPIED from the seal's run. Its producer is `tools/banco_ancla.sh` (§593).

**Since §634 it covers the CO-SIGNED ANCHOR envelope** (RFC-0013, E4b):
`spec/vectors/ancla-cofirmada/` brings FIVE positives CAPTURED from a real node that signs one
heartbeat per second —the seed of the operator of `ancla/`—: three heads, their anchors DERIVED by
the command as in the anchor envelope, and that operator's medium, which publishes all three in its
tree and signs its note with ML-DSA-44 type `0x06`, WITH SALT, co-signed by test witnesses. One
witness, two, the co-signature without its vkey in the envelope —it is counted and not judged—, a
medium of a single anchor and the last of the three. And TWENTY-TWO negatives by ONE mutation each,
one per producible rule and per place; that of the real ATTACK —the medium of ANOTHER operator, well
signed, with this head's anchor inside— carries the key of a second node, from another seed. The
threshold of witnesses has no vector: the command does not judge it (§633). The family is COPIED
from the seal's run: the heads, the timestamps and the salt change in every run. Its producer is
`tools/banco_ancla_cofirmada.sh` (§634).

**Since §712 it covers the CLOCK of the extension** (RFC-0012 D-C, 2.3): five new envelopes in
`spec/vectors/paquete/`, three in `spec/vectors/consumo/` and one in `spec/vectors/ancla/`, with
their lines at the end of each `MANIFIESTO.txt`. Six are CAPTURED from a run of
`tools/banco_bifurcacion.sh`, against a real node with the binary untouched: the operator trims his
journal and keeps his counter —the fork: the same `mmrSize`, another leaf—, trims it and restores
the counter to the leaf of then —the split view: the same `mmrSize` and the same leaf—, or restores
both from before some orphan indices —the clock that goes backwards: the history is still consistent
and the new one signs below—. The other two are the two directions of the anchor's split view
(`ancla/vista-dividida.json`) as an extension, derived without another run. The seven negatives gave
VERDE in the kit and in the second implementation from before §712 (measured); the positive, an
extension above six orphan leaves, stays VERDE. The bench also watches the live witness (§712): it
does not stop before the orphans, the restart nor the recovery of §690, and it stops with
`no-extiende` before the fork and before the clock that goes backwards over a consistent history, at
the first head later than the kept one. The one of `ancla/`,
`neg-ancla-bifurcada-mismo-mmrSize.json`, is the anchor the command derives from the old head of the
fork against the new one, with the empty path, DERIVED without another run; it also gave VERDE in
the kit and in the second one from before (measured).

**Since §714 it covers the FORK as detection** (2.12, mode 5): twelve new envelopes in
`spec/vectors/ancla/`, with their lines at the end of its `MANIFIESTO.txt`, DERIVED without another
run from heads already captured in the catalogue. Three positives, exit 0: the two heads of the fork
of §712 (`mmrSize` 5, the same peak); A's at 5 against that of the operator restored from before the
orphans (`mmrSize` 5, ANOTHER peak: a restart with a journal that was not the last one); and two
heads of the genesis of the suite's key, from two runs of `tools/banco_completitud.sh` —each run is
a new node with the same key and without the previous one's journal: the restart without journal,
literally—. And nine negatives, one per rule of mode 5 and per place, that of two keys that the acts
DO join with another `mmrSize` included —the operator is accepted and the size falls—. No earlier
vector changes verdict: `rama` is a key that no earlier envelope carried. And nine more negatives,
derived the same way: five fix that the form is decided by the PRESENCE of the key, also with `null`
—the branch with `"ancla": null` or `"contraria": null`, the split view with `"ancla": null`, the
derived anchor and the exact one with `"camino": null`—, on which the second implementation gave
VERDE (three, already before §714); and four fix the `rama` as the subject of its reds —`null`, a
string, without `formatVersion`, and a v2 not available: the version before `available`—. The kit
from before §714 passes 26 of the 44 entries of the family: the 23 from before and the three
envelopes of the contrary one and of the path. It ignores the `rama`: it gives VERDE on the three
positives, without its text, and on twelve of the fifteen negatives of the branch; the other three
fall with another text.

**Since §649 it covers the ROTATION of the operator's key** (RFC-0015, E5b):
`spec/vectors/rotacion/` brings THREE positives CAPTURED from ONE real node that rotates twice over
the same journal —A, its genesis committing B; B, with A's signature; C, without B's— and extension
envelopes with the chain that `zkssl_keyActs` served: one link with both signatures, one with the
incoming one's alone, and two links of one jump. The TWO behaviours of the operator that the chain
gives away, seeded with their real keys like the anchor's split view: the old key signing again with
its counter restored —OVERLAP by rule 3— and the new one signing with a fresh counter below its act
—outside its stretch, rule 4—. And EIGHTEEN negatives by ONE mutation each: one per rule of the
reader of the act and of the chain that a mutation produces, and one that says the acts do not order
—with the roles swapped the continuity holds and the consistency falls—. The two rules that a
mutation does not produce without signing again —the leaf of the incoming one's signature distinct
from its `desde`, that of the outgoing one outside its stretch— are judged by the tests of the core,
and this is declared. The second implementation passes the whole family with the same harness
(§649). The family is COPIED from the seal's run. Its producer is `tools/banco_rotacion.sh` (§649).
**Since §686 it also brings the OVERLAP envelope** (2.14): eleven envelopes derived from those above
without another run —the head and the chain as they are—: the two seeded behaviours, now as
detection with exit 0, and nine negatives, the three keys inside their stretch among them.

**Since §771 the rotation is also in the consumption and in the completeness**, with envelopes from
the same node that rotates and from the same producer. In `consumo/`, the consumption is published
with the new key in use, between a head of the old one and one of the new one: with the chain it
comes out VERDE and says that the acts join the two keys; without it, or cut at the genesis, the
keys are DISTINCT. In `completitud/`, a send of zeros with the old one in use leaves its receipt and
a head of the old one closes the era; the current one is a head of the new one, and the window is
measured in the operator's single count of indices: with the new one inside, the envelope is
premature (`ventana ABIERTA`); without the chain, or cut, the keys are DISTINCT. No completeness
verdict 0 crosses a rotation: it asks for a real STARK proof —acknowledgement, batch or pledge— that
this producer does not have, and this is declared. The second implementation passes the three of
consumption; it does not read the completeness (section 9, `tools/segunda/`).

**Since §693 each negative names its cause** (section 6.1). Fifteen vectors of five families had the
subject as their only oracle: the `edad:`, the `pago:`, the `cobro:` and the two `prenda:` above,
and ten of the package —`seq: `, `sin 0x: `, `hex: `, two `cabeza: `, two `acuse: `, two
`cofirma 1: ` and `nueva: cabeza: `—, and three more in the `0.3/` catalogue, those of age, payment
and pending. The four paragraphs above that say that only the prefix is pinned tell how it was until
then. No line is rewritten: each vector gains a NEW line in its manifest, with its comment, that
pins the cause —the variant of `winter-verifier` in the eight of the lied statement;
`la firma no verifica`, `RaizDistinta`, `hex: cifra no admitida en la posicion 0`, and the value
after `sin 0x: `, in those of the package—, and the old one remains and keeps passing. The three of
`0.3/` pin what their kit prints, the tree's `zk-ssl-verify` at `0eda58c` compiled from that commit,
measured in §693: with it and the current harness, the four manifests of `0.3/` give 41 of 41 and
exit with 0, like their 38 of 38 of §620 with the harness of then.

**Since §706 the hiding mark has a vector.** The age and the payment gain three negatives each, from
the `prueba` key of the positive from which the others of their family come —`edad-todos.json` and
`pago-t-frontera.json`—: `neg-marca-m32`, with the m of the mark at 32; `neg-meta-vacio`, without a
mark; and `neg-marca-m32-y-opciones`, a SCENE, and this is declared: the m at 32 and other options
at once, because with a single defect one does not see in which order the judge looks. The manifest
pins the whole cause, `marca de la ocultacion …` (section 5, the age), which the house governs. The
six fix that the second implementation judges the mark like the kit, with the same m, the same cause
and in the same order: until §706 the second one rejected the six for another cause, and gave VERDE
to an honest proof with m = 32. The paragraphs of the age and of the payment above, with their nine
and seven negatives, tell how it was until then.

**Since §711 a malformed proof is rejected with its cause, not with a panic.** The age and the
payment gain two negatives each, from the `prueba` key of `edad-todos.json` and
`pago-t-frontera.json`, with ONE byte changed: `neg-prueba-opciones-cero`, byte 34 —the queries of
the options— to zero, and `neg-prueba-consultas-cero`, byte 46 —the unique queries— to zero. Before
§711 both made the command exit with 101, a panic inside the winterfell fork, which is none of the
four codes of section 6. Now they are ROJO with exit 1, and the manifest pins the cause,
`prueba mal formada: opciones fuera de rango` and `prueba mal formada: consultas fuera de rango`
(section 6.1), which the fork puts and the second implementation repeats.

**Since §713 the FRI remainder is tied to its commitment, and has a vector.** The age gains a
negative, `neg-prueba-resto-ajeno`, CAPTURED and not derived: changing the commitment moves the
queries, and a mutation does not produce it. It comes from the same run as `tools/banco_edad.sh`,
with a node whose `winter-fri` commits to the hash of ANOTHER remainder and carries the real one in
the proof; it is produced by `tools/vectores/resto_ajeno.sh`. Before §713 it gave VERDE in the kit
and in the second implementation. Now it is ROJO with exit 1, and the manifest pins the cause,
`FriVerificationFailed(RemainderCommitmentMismatch)` (section 6.1).

**Since §715 a proof has a single encoding.** Until §715, from the bytes of a proof others came out,
without the witness or the key, that verified the same and had another fingerprint: a `usize`
written with more bytes than necessary, an extra row in the tables or an extra group in an FRI
layer. The age gains fifteen negatives and the payment eleven, from the `prueba` key of
`edad-todos.json` and `pago-t-frontera.json`, each with ONE mutation; they are produced by
`tools/vectores/maleabilidad.py`. With the kit from before §715, the over-long `usize` (in three
places), the extra FRI group and the extra row gave VERDE with exit 0; in the payment the extra row
does not fit. The others fix forms of the same class that the kit already rejected: the number of
constraints plus `2^32`, an extra FRI layer, an extra node in a batch (§659), a tail byte (§653),
three tails inside an opening or a batch, and in the age three bytes that before §711 made the
reader panic. The manifests pin the cause: `prueba mal formada: usize no canonico`, the variant of
`winter-verifier` of an opening that does not match its commitment, and the tail with its two names
(section 6.1).

**Since §717 the age's `m` has a cap, and three attacker vectors.** They do not come from
`edad-todos.json`: mutating the `nextPending` of a real head falls earlier, in the `epochDigest`.
They are packages that anyone assembles without the operator's key, without a ledger and without
proving: a v5 head signed with a fixed-seed XMSS test key —the command verifies the signature with
the key the head carries— and a fabricated proof that only has the form. They are produced by
`tools/vectores/coste_edad.py`, with the binary of `tools/vectores/coste_edad/`. `neg-m-sobre-tope`
(`m` = 17) and `neg-m-mayor-legible` (`m` = 23, the largest the reader of the proof admits) fall by
the cap (2.7), and the manifest pins their cause with their `m` and without the cap;
`neg-m-13-fabricada` (`m` = 13, at the cap) reaches the proof and falls in the OOD coherence, and
the whole cause is pinned. With the kit from before §717 all three gave ROJO in the OOD coherence,
after 0.07 s of CPU at `m` = 13, 1.1 s at `m` = 17 and more than a minute and 531 MB at `m` = 23
(measured in private): the cost was chosen by whoever sent the package.

**Since §721 a fourth vector fixes the value of the cap.** The three of §717 say the same with any
cap from 13 to 16. `neg-m-14-sobre-tope` comes from the same script, with the same test key:
`nextPending` = `2^13 + 1`, the smallest high-water mark above 13, and a fabricated proof of `m`
= 14. It falls by the cap before reading the proof, and the manifest pins its cause with its `m` and
without the cap. With a cap of 14 it would reach the proof and fall in the OOD coherence. Together
with `neg-m-13-fabricada`, the cap stays at 13 and at no other value: with 12, that one falls by the
cap; with 14 or more, this one reaches the proof.

**Since §722, the COMMON HISTORY** (2.3, 2.5 and 2.14): thirteen more envelopes in
`spec/vectors/rotacion/`, with their lines at the end of the `MANIFIESTO.txt`. Seven are CAPTURED
from a run of `tools/banco_historia_comun.sh`, two real nodes untouched: the genuine conflict; the
refutation, VERDE —the residue that is declared—; the head of the key that left, signed after its
handover, as OVERLAP; the fork of the anchor's mode 5 with the chain; and three negatives —that head
extended to a later one, which falls by the handover, and the two heads of the conflict inside their
stretch and their handover—. Four more, one per rule of the handover that a mutation does not
produce, are assembled by `tools/vectores/historia_comun/` with an act signed again by the same keys
of the bench, over the captured refutation. Before the fix, in the kit and in the second
implementation, six of the eleven did not say their own: the overlap gave ROJO, the extension of the
late head fell by the consistency and the four handovers signed again gave VERDE (measured). Two
more, from the review of the handover, with ONE chain of three —that of the refutation and an act of
C, the key B commits, whose handover is at the `mmrSize` of B's head—: the extension of the
refutation falls by the upper edge of the later one, and the overlap of that same head gives it
away. With the handover of a single edge the same chain gave two contrary VERDE, in the kit and in
the second one (measured). None rewritten.

**Since §914, the `zkssl/0.5` cut** (RFC-0018 D-G and D-H; RFC-0009 D-AL bis): every proof carries
the v2 mark —25 bytes: the family, the version of its AIR, the profile, m and the two random
columns, paired in DEEP— and the judge reads it BEFORE anything else; the five families of T = 64
move to 128 and the age starts at `m = 4`. Kit 0.5 rejects by the mark every proof of `zkssl/0.4`,
so **seventy-eight vectors of six families change verdict or text and MOVE whole**, with their
lines, to `spec/vectors/0.4/<familia>/` (rule 2 of the PROCESO, R6 b of RFC-0021): 31 from `edad/`,
20 from `pago/`, 6 from `pendiente/`, 4 from `prenda/`, 7 from `rechazo/` and 10 from
`completitud/`; they are judged by the tree's `zk-ssl-verify` at `cbe95d70` (§913, the last seal of
the 0.4 wire), which `tools/conservados.sh` builds from its commit —by stretches since this seal:
`0.4/completitud` also carries the two of §682 with its own—. The negatives that fall before the
proof stay with their bytes. What is reborn is reborn with the version of the wire in the name
(`-0.5`, the author's decision 4 in §913): the positives and the negatives of their rule, from one
run of each bench (`banco_edad`, `banco_pago`, `banco_pendiente`, `banco_prenda`, `banco_rechazo`,
`banco_recibo_agregado`, all of them VERDE; in the pending one and the rejection the node proves
with v2, so `cobro-v2-*` and `saldo-insuficiente-banda-v2` are not reborn); those of the
malleability, of the cost, of the canonicity, of the window and of the foreign remainder, from their
scripts in `tools/vectores/`; and the negatives of the v2 MARK that D-G foresaw, `neg-v2-*`, from
`tools/vectores/marca_v2.py`, new: fifteen in the age —the fourteen of the table and
`neg-v2-columnas`, the 1h bis that the kit's byte-by-byte sweep led to writing— and three in each of
the other four families (the 0.4 positive with its bytes, the empty meta and another family).
Ninety-six new vectors; each one with its line and its cause (section 6.1, the «outside the
contract» class for v1 and the empty meta). The second implementation judges the 89 pairs with a
proof like the kit, letter for letter (`tools/segunda/juez_stark.py`). None rewritten.

## 10. History

- §289: the package is born (format v1) and its binary; §290: the declared shutdown; §293: the
  extension package; §322: the v2 with the co-signatures inside.
- §399 — the declared `index` is tied to the one inside the signature (E3); the command prints the
  embedded one; the witness reads the served `index`.
- §400 — RFC-0004 becomes ACCEPTED: rule 4 of the PROCESO, settled with a measurement.
- §401 — the artefact: `tools/artefacto.sh`, the reproducible tarball and 3 ter of the canon
  (section 11).
- §408 — the conformance harness (RFC-0005, E4): `tools/conformidad.sh`, the only producer of the
  manifest loop, consumed by the canon and by `artefacto.sh`, and inside the tarball (section 9).
- §419 — the consumption package (RFC-0006, E3b-2, D-15/D-17): two v4 heads with the consistency
  inside, and the position **derived and cross-checked** against the `isRight` received.
- §418 — the unknown `tipo` is rejected with its name (RFC-0006, E3b, D-12):
  `rechazo-tipo-desconocido.json` stops falling by `falta cabeza`, which was another one's rule.
- §423 — the consumption catalogue travels INSIDE the artefact: `montar()` copies
  `spec/vectors/consumo/` and the `--check` runs the TWO manifests with the same binary (RFC-0006,
  E3c; §422).
- §425 — the two catalogues are run FROM INSIDE the unpacked tarball and without a repo: the
  `--check` opens it in `target/artefacto/desde-dentro/` and demands the SAME verdict as from the
  tree (point 110).
- §431 — the catalogue of the conflict envelope (RFC-0006, E4a): `spec/vectors/conflicto/` with the
  positive and fifteen negatives, one per producible rule, and the fourth stanza of the canon. The
  catalogues are THREE.
- §451 — the v5 head in the command (RFC-0007, E1a): the version chooses the v5 recomposer with its
  family; the rejections by version cite «v2, v3, v4 o v5»; the out-of-the-set one is
  `rechazo-formatVersion-6`; the `-5` is still listed and falls for another cause.
- §455 — the rejection envelope (RFC-0007, E3a-1): the command proves the CAUSE of four rejections
  over the committed state, with a head prior to the rejection when the cause cites something that
  only grows; `spec/vectors/rechazo/` and the fifth stanza of the canon. The catalogues are FOUR.
- §456 — the rejection envelope also proves the four causes that demand a receipt (RFC-0007 E3a-2):
  `StaleState` (a declared root that is not the committed one), `WrongRegulatoryLimit` (like
  `OverRegulatoryLimit`) and the two batch duplicates (the rule of `apply_many`, the first clash),
  with the head of the exact `seq` where the state is instantaneous; the catalogue, from a real
  receipt.
- §459 — the rejection envelope proves `AccountFrozen` (RFC-0007 E3b): the leaf of the account under
  the `frozenRoot` of the head of the exact `seq`, with the path of `zkssl_frozenPath` as is, the
  depth the core fixes, the cross-check with the index and the non-empty leaf.
- §460 — the rejection envelope proves `SupplyCapExceeded` (RFC-0007 E3, which is left whole): the
  parameters give the committed `maxSupply`, the head is that of the exact `seq` and the node's
  `wouldBe` has to be the `totalSupply` of the head plus the amount the applicant sent (`peticion`);
  `PendingTreeExhausted` is declared without portable proof.
- §465 — the age package (RFC-0007, E4b-2), the seventh form: the command demands a v5 head and
  verifies the `zk-ssl-air` proof against what it signs, without the prover.
- §466 — the node writes the age envelope with `--prueba-edad`, over its ledger and with the server
  stopped (RFC-0007, E4b-3): the first form with a producer in the tree (section 7).
- §467 — the catalogue of the age envelope (RFC-0007, E4b-3, which closes E4): `spec/vectors/edad/`,
  two gathered positives and nine negatives. The catalogues are FIVE.
- §473 — the node writes the rejection envelope with `--prueba-rechazo` (RFC-0007, E5): the cause is
  DEMANDED, not deduced from the order of the guards; the first is `AccountFrozen`.
- §475 — the rejection envelope proves `AccountNotFound` (RFC-0007, E5): the EMPTY leaf of the
  account under the `accountsRoot` of the head of the exact `seq`, with the material inside the
  rejection itself and without a method of the wire (the node writes it since §474).
- §476 — its catalogue: a gathered positive and eight negatives; the disguise cannot be produced.
- §478 — the rejection envelope proves `InsufficientBalance` (RFC-0007, E5): the only cause that
  verifies a STARK; the envelope does not publish the balance, and the node writes the `banda`.
- §479 — its catalogue: a gathered positive and eight negatives. E5 is left whole, and with it the
  five stages of RFC-0007.
- §480 — this document is brought up to date with the whole of RFC-0007: the texts of
  `AccountNotFound` in section 5, the heads its arm accepts, the material of each cause and the
  conflict package in section 4, the lines and the VERDE of each form, who assembles the package and
  the fifth catalogue of the artefact.
- Until §397 this contract lived in the header of `crates/zk-ssl-verify/src/main.rs` (1..90,
  `293990fedc785833`), which had already confessed once (§247) to having declared its surface
  complete without it being so. §397 moves it here and leaves the header referring to it, without
  enumerating.
- §640 — canonicity (RFC-0016): a `u64` of the envelope or an element of a digest that is not below
  `p` is rejected on reading, with its value; the three digest readers say the rule by `{e}` and not
  by `{e:?}`; four negative vectors -two here, one in `ancla/` and one in `completitud/`-, which the
  reference and the second implementation reject with the same text. No earlier verdict moves: the
  ten manifests say the same as before in every entry they already had.
- §648 — `actas`, optional (RFC-0015 E5a): in the envelopes that demand the continuity of a signer
  —the extension, the consumption and the heads the completeness compares with its closing—, a chain
  of acts makes two keys ONE operator, with the kit's judge (`juzgar_continuidad`). Without `actas`,
  not a byte changes: no vector moves. The conflict and the split view do not read it, and it is
  said why. Without a vector yet: those of E5b will be drawn by `tools/banco_rotacion.sh` from a
  node that rotates, and the second implementation will learn to read it with them.
- §649 — the rotation catalogue (RFC-0015 E5b): `spec/vectors/rotacion/`, the twelfth family of the
  artefact, copied from `tools/banco_rotacion.sh` against a real node that rotates twice; the second
  implementation reads `actas` and passes it whole. The `1/3` line of the extension stops saying
  «v3» —the extension reads v3 to v6—, in both implementations: no manifest pinned it.
- §686 — the OVERLAP envelope (RFC-0015 E5c), the fourteenth form: a signed head outside all the
  stretches of its key in the chain of acts is DETECTION, with exit 0; eleven envelopes in
  `spec/vectors/rotacion/`, derived from those of the rotation without another run, which the second
  implementation passes with the same harness. And the refutation of the conflict between two keys
  of an operator, written: it is the extension envelope with the chain, not the chain alone.
- §687 — the overlap envelope has a producer: the witness assembles it from its journal with
  `--auditar --sobres`. `tools/banco_rotacion.sh` demonstrates it with a real witness that sees a
  rotation, goes on, and stops before the new key signing below its act.
- §688 — the split view the witness sees has an assembled envelope: `--auditar --sobres` writes the
  anchor one, mode 4, with the two heads of its journal, and the command judges it without the node
  or the witness.
- §689 — `--comparar --sobres` assembles the anchor envelope, mode 4, for every index that two
  witnesses saw split, one per journal: the scene of the two witnesses and the two views, portable.
- §693 — the cause of the rejection, with a name (section 6.1, RFC-0005 E4): thirteen classes, the
  reserved text `NO SOPORTADO` for what an implementation does not read, and the harness that
  demands the cause on the ROJO line; eighteen new lines in eight manifests, three of them from
  `0.3/`, none rewritten (section 9).
- §706 — the hiding mark, with a name (RFC-0005 E4): its text enters the catalogue of the age, the
  claim, the pledge and the band, and the «malformed proof» class of 6.1; six new negatives in the
  age and the payment (section 9), and the age's range of m, up to date.
- §771 — the rotation in the consumption and in the completeness (RFC-0015, what E5b left): three
  envelopes in each family, from the same node that rotates, with `actas` and without them (section
  9).
- §711 — a malformed proof, with its cause (RFC-0005 E4): `prueba mal formada: {qué} fuera de rango`
  enters the «malformed proof» class of 6.1, and four new negatives in the age and the payment
  (section 9) that before made the command exit with a panic.
- §712 — the clock is the embedded index (RFC-0012 D-C): the extension, the consumption and the
  anchor of mode 3 demand, after the consistency, the same digest with the same `mmrSize` and a leaf
  that grows with the same key (2.3); the witness counts by the embedded index, one count per
  operator on its three routes, and applies the whole rule to every head that arrives: the `mmrSize`
  already seen with another digest and the later head that signs below the kept one (2.12). The two
  new texts enter the «the rule does not hold» class of 6.1; nine new vectors in three families,
  none rewritten (section 9).
- §713 — the FRI remainder, tied to its commitment: a STARK proof whose last FRI commitment is not
  the hash of the remainder it carries is ROJO `FriVerificationFailed(RemainderCommitmentMismatch)`,
  the `winter-verifier` error of the catalogue of every family with a proof (section 5), in the kit
  and in the second implementation. Until here, a proof that committed to ANOTHER remainder
  verified. The text enters the «query that does not match the commitment» class of 6.1; a new
  vector in `edad/`, captured from a real node with a prover that commits to another remainder; none
  rewritten (section 9).
- §714 — the fork with the same `mmrSize` is DETECTION: mode 5 of the anchor envelope, with exit 0,
  for two heads of the same operator with the same `mmrSize` and distinct digests, in the kit and in
  the second implementation (2.12). And the witness stops it with any peak, whether or not the node
  is behind, with its class in the journal —`bifurcacion`, «fork or restart without journal»,
  `DIARIO_VERSION` 5—, in the live loop, in `--auditar` and in `--comparar`, which assemble its
  envelope. Its rejection texts enter 6.1; twenty-one new vectors in `ancla/`, none rewritten
  (section 9).
- §715 — a proof has a single encoding: the fork's reader demands the minimal encoding of each
  `usize`, and the channel, one row per queried position and one group per folded position, in the
  kit and in the second implementation. Until here, an over-long `usize`, an extra row or an extra
  FRI group verified with other bytes. `prueba mal formada: usize no canonico` and the tail enter
  the «malformed proof» class of 6.1, and `FriVerificationFailed(LayerCommitmentMismatch)` «query
  that does not match the commitment»; twenty-six new vectors in `edad/` and `pago/`, none rewritten
  (section 9).
- §717 — the age's `m` has a cap, 13: the judge rejects it before reading the proof, and the prover
  and the layer's producer do not prove a ledger that passes `2^13` positions. The fork evaluates
  the exemptions of the hiding as a stream. Until here, a package with a head with its own signature
  and a fabricated proof chose the judge's cost, linear in `2^m`. `fuera de 3..=` enters «the rule
  does not hold» of 6.1; three new vectors in `edad/`, none rewritten (section 9).
- §721 — the value of the cap of `m`, fixed: a new vector in `edad/`, `neg-m-14-sobre-tope`, which
  with `neg-m-13-fabricada` leaves the cap at 13 and at no other value; none rewritten (2.7 and
  section 9).
- §722 — the common history: two operators in agreement signed themselves a common history that
  refuted a genuine conflict, and 2.5 said they could not (measured with two real nodes). The eight
  places that demand continuity —the extension, the consumption and the six heads of the
  completeness— demand from here on the HANDOVER (2.3): the `procedencia` of the acts ties the MMR
  of the two heads, each one in the handover of its stretch by its two edges. The overlap envelope
  gives away the head of a key outside the handover of its stretch (2.14), with the same judgement.
  The node gains `--relevo-minimo` for the emergency of D-G. The refutation is still possible, and
  it is declared with what remains (2.5 and 8). Thirteen new vectors in `rotacion/`, none rewritten
  (section 9).
- §914 — the `zkssl/0.5` cut: the v2 mark before anything else, with its eight causes in the
  catalogue (section 5, the age; v1 and the empty meta in «outside the contract», 6.1), the floor of
  the age at 4, and seventy-eight vectors of six families moved to `0.4/` with ninety-six new ones
  under `zkssl/0.5` (section 9); none rewritten.
- Changing this document is changing the contract: it comes in by RFC (`spec/rfc/PROCESO.md`).

## 11. The artefact

What a third party downloads is `arqueo-verify-<versión>-<host>.tar.gz` (§401), and inside:
`zk-ssl-verify` (the binary), `conformidad.sh` (the harness of section 9, §408), `spec/PAQUETE.md`
(this document), `spec/vectors/<familia>/` for each of the TWELVE families of `FAMILIAS` —paquete,
consumo, conflicto, rechazo, edad, pendiente, pago, prenda, completitud, ancla, ancla cofirmada and
rotación— (the twelve manifests and their vectors), `LICENSE-APACHE`, `LICENSE-MIT`, `NOTICE`,
`THIRD-PARTY.txt` (the licenses of everything linked), `VERSION` (the commit, the toolchain and the
flags it was compiled with; ⚠️ §824: and the host's `cc`, `as`, glibc and startup objects, and the
variables cargo sees) and `SHA256SUMS` (the fingerprint of each file inside). It is checked with
`sha256sum -c SHA256SUMS`, and the binary against the twelve catalogues with
`bash conformidad.sh ./zk-ssl-verify` —the package one, by default— and
`bash conformidad.sh ./zk-ssl-verify spec/vectors/<familia>/MANIFIESTO.txt` for each of the other
eleven: each entry says the exit code and the text. This section said SIX until §574: the payment
(§509) and the pledge (§546) entered `FAMILIAS`, and the tarball, without it saying so; §574
corrects it on adding the ninth, §593 adds the tenth, the anchor, and §634 the eleventh, the
co-signed anchor, and §649 the twelfth, the rotation.

The binary's fingerprint **does not depend on the path of the tree nor on `HOME` within one same
machine, with `CARGO_HOME` and `RUSTUP_HOME` under `HOME`** —it is compiled with
`--remap-path-prefix`; outside `HOME`, their paths would enter whole (deduced, not measured; ⚠️
§824: measured afterwards in the kit, with the tool of `b6994c3`, and they do enter: `doc/KIT.md`
§«Reproducir el kit, para quien no se fíe de la descarga» [reproducing the kit, for whoever does not
trust the download]), those of `RUSTUP_HOME` if `rust-src` is installed—, and it does depend on the
toolchain, on `Cargo.lock` and on the machine: with the `rustc` that `VERSION` names,
`bash tools/artefacto.sh` over the commit that `VERSION` names produces again the same binary and
the same tarball on the same machine, and `tools/canon.sh` checks in every seal what it can check
there (at `b6994c3`, two compilations of the same tree in two `target`s, same fingerprint; two
tarballs, same fingerprint; the twelve manifests from the tree and, since §425, again **from inside
the unpacked tarball and without a repo**, with the same verdict). ⚠️ §823: it said «**it does not
depend on the machine nor on the user** —it is compiled with `--remap-path-prefix`—, but it does on
the toolchain and on `Cargo.lock`», and that the canon checked «that property» with «two
compilations in two paths». Measured on 15-10-2026: 0.4.2 rebuilt from its commit with its `rustc`
on another machine comes out with 13 of its 35 sections different; the fingerprint also depends on
where `rust-src` is, if it is installed —its paths enter the binary with the name of its toolchain,
because no rule of `b6994c3` takes them to `/rustc/<commit-hash>`— and on the native code compiled
by the host's `cc` and `as` (blake3's assembler). The two paths of the canon were two `target`s of
the same tree. The detail, in `doc/KIT.md` §«Reproducir el kit, para quien no se fíe de la
descarga». What the binary demands: x86_64 Linux and a glibc equal to or greater than the one
`VERSION` declares (`glibc_max`); it is not static, and this is said.

Since §824, `tools/artefacto.sh` draws the remap rules from a single producer, derived from
`CARGO_HOME` and from the sysroot that `rustc` reports and ordered so that the most specific one
wins, one of them for `rust-src`, and with them the fingerprint no longer depends on where
`CARGO_HOME` or `RUSTUP_HOME` are nor on whether `rust-src` is installed; `cargo`, `rustc` and the
`cc` run with `env -i` and only the variables the tool lets through, and a cargo configuration file
that changes what is compiled is ROJO; each `target` carries the stamp of the host it was compiled
with, and is emptied if it is not the current one; `VERSION` names the `cc`, the `as`, the host's
glibc, the fingerprint of the startup objects the `cc` links and the variables that pass; and the
`--check` compiles the kit in two roots —the tree and a copy of it under `target/artefacto/raiz2`—
and searches the binary, whole, for the paths of the two roots, of `HOME`, of `CARGO_HOME`, of
`RUSTUP_HOME` and of `<sysroot>/lib/rustlib`, and if it cannot search for them it is ROJO. What that
changes in the kit's fingerprint, and what remains unproven between machines, in `doc/KIT.md`
§«Reproducir el kit, para quien no se fíe de la descarga».

What the artefact is NOT: it is not a publication on crates.io (the crate does not carry the spec
nor the vectors) nor does it prove anything about the operator's key (section 8). It does not carry
the vectors of the wire —their consumer is the cli's witness, not this binary, and its own adapter
is written so that a second implementation puts its own there— nor the KATs of
`spec/vectors/nucleo/`, which have no manifest and are exercised by `nucleo_kat.rs`: delivering
vectors that the delivered binary cannot run would be claiming more than is proven. The release is a
file with a fingerprint, and the fingerprint lives in the entry that sealed it, never here: **the
tarball cannot publish its own fingerprint**, because it carries `commit` and `describe` inside and
sealing moves them —and putting a tag moves the `describe` again—. That is why a release is made in
this order: `tag`, produce, upload; and what this document gates is the PROPERTY, not a number.
