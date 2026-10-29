# RFC-0015 — The signer's lifecycle: rotating, exhausting and losing the index, with the next key committed

> **Español:**
> [`spec/rfc/0015-el-ciclo-de-vida-del-firmante.md`](../0015-el-ciclo-de-vida-del-firmante.md) is
> the original of this page, in Spanish; it prevails wherever the two differ. This translation is of
> commit `7766efb` (audit entry §917) and was produced by the project's assistant, a Claude Code
> session, in audit entry §918 (`GENAI.md`): it is a generated artefact, not text written by the
> author. Line citations of the form `file:N` refer to the Spanish files.

- **Status:** PROPOSED (§639) — the text and its decisions, written over what was measured in the
  tree (§636, §638) and in the IETF's RFC 10033. The five decisions, TAKEN in §642 by delegation
  of the author, with the criterion of §609, and REVERSIBLE: see «Decisions». Built: E2, the core
  of the act (§643), E3a, the node that signs its acts (§644), E3b-1, the signature of the
  outgoing key and the ceiling that does not reserve (§645), E3b-2, `zkssl_keyActs` (§646), and
  E4, the witness that rotates with the acts (§647), E5a, the kit's envelopes that read `actas`
  (§648), E5b, its catalogue with its bench and the second implementation (§649), and E5c, the
  overlap envelope (§686); E6, not built.
- **Author:** Ángel José Toranzo Portela
- **GenAI assistance:** Claude (session 193, §639, which writes it whole over the measurement of
  §636 and §638) — see [`GENAI.md`](../../../GENAI.md)
- **Date:** 2026-10-01
- **Protocol version affected:** `zkssl/0.4` — **it does not go up, and the head does not move**: a
  family is born with its domain and its vectors (the act), a read method and an optional field in
  the envelopes that compare heads. What changes in semantics is the witness: today it stops at
  any change of key, and with E4 it would carry on at one that brings its act.
- **AUDITORIA entry(ies):** §110.2 (reusing the index leaks the key), §112 (upstream's
  `KeyExhausted`, measured), §115 (the cadence), §245 (the witness's TOFU), §246 (there is no key
  to anchor), §288 (the 19-84-92 family), §298 and §328–§337 (the guardian and the reconciliation
  at start-up), §399 (the embedded index), §567 (the receipt's clock is the index), §591 (the
  key's fingerprint), §594 (the journal gate in every state), §636 and §638 (what happens today
  on changing key, measured), §639 (this proposal), §642 (the decisions) and
  §643 (E2),
  §644 (E3a), §645 (E3b-1), §646 (E3b-2), §647 (E4), §648 (E5a), §649 (E5b), §686 (E5c), §687
  (the witness that assembles its envelope) and §690 (the jump of `desde`).
- **Backlog:** entry **84** (exhaustion, rotation and loss of the index), with entry **92**
  (custody and survival of the index) and entry **19** in its family line, which §288 asked to be
  cut together; entry **87** (cryptographic agility: the act carries the scheme of the key it
  presents); and entry **103** (the journal gate is switched off by removing a file), which this
  proposal does not close.

## Status of the stages

| stage | what it delivers | breaks the wire? | status |
|---|---|---|---|
| E1 — the lifecycle, written | this text | no | **proposed (§639), decided (§642)** |
| E2 — the core | `DOMINIO_ACTA`, `acta_digest` and its KAT, and `verificar_acta` in the kit, with the second implementation | no (additive: a new family) | **built (§643)**: `DOMINIO_ACTA` (`ACTAS_V1`) and `acta_digest` in `zk-ssl-hash`; the kit's `actas` module with the preamble (`ZK-SSL-key-act`), today's scheme and `verificar_acta`, with the rules before the signatures; three KATs —genesis, rotation and preamble— that the second implementation reproduces, 29 of 29; twelve rows of the ACTA family in `NUCLEO.md` |
| E3a — the node signs its acts | the genesis act, the rotation to the committed successor and the exhaustion notice, at start-up | no | **built (§644)**: opt-in with `--siguiente` (without it and without acts, the node signs as it has until today); `--huella-de-clave-fichero` prints the fingerprint of the cold key; at start-up, the node judges the chain of acts in the journal, signs the genesis or the rotation through the heads' path and records it with `fsync`, and does not start with a key nobody committed; the threshold of one year of heartbeats demands `--reconozco-agotamiento`. The act's line is seen by no reader of heads, and it is seen by the counter's gate |
| E3b-1 — the old one's signature and the ceiling | the signature of the outgoing key (decision 5), and the heartbeat that stops burning the counter at the ceiling | no | **built (§645)**: `--clave-anterior-fichero` gives the seed of the outgoing key, only in a rotation; the old one signs the same preamble at the leaf the counter gives and the new one starts at the next, and a seed that is not that of the act in force spends nothing. At the ceiling the signer returns `Agotada` **without reserving**: the counter states the ceiling, because the SK does not represent it (measured) |
| E3b-2 — the wire | `zkssl_keyActs`, which serves the chain of acts from the genesis | no (additive: a method) | **built (§646)**: without parameters, `{actas}` from the genesis, assembled once at start-up after judging it; the act in JSON is written and read by the kit's `acta_a_json` and `acta_de_json`, which are also the journal line —byte for byte the one from before—; an element of a digest worth `p` or more is not read. The surface goes from 31 to 32 methods and `zkssl/0.4` does not go up |
| E4 — the witness | at a change of key it asks for the act, judges it against the key it pinned and carries on or stops; `--auditar` judges it in the journal | no | **built (§647)**: at a change of key the witness asks for `zkssl_keyActs` and judges it with the kit's `juzgar_rotacion`; if the chain leads from the pinned key to the received one, it records `rotada` with the chain inside, pins the new one with its stretch and carries on; if not, it stops as before, with the reason. `solapamiento` is born, which stops (rules 3 and 4). `--auditar` re-judges the rotation from the line, without the node. The witness's journal goes to v4 |
| E5a — the kit | the `actas` field in the envelopes that compare heads | no (additive: an optional field) | **built (§648)**: `juzgar_continuidad` in the kit, one judge for the eight places that demand the continuity of a signer —the extension, the consumption and the six heads that completeness compares with its closing—; the EMBEDDED index orders the two heads, the chain leads from the earlier to the later (rules 1 to 3) and the later falls within its stretch (rule 4, which is born with a name: `FueraDeTramo`). Without `actas`, the text of `claves_distintas` byte for byte. The conflict and the split view do not read it, and `spec/PAQUETE.md` says why. ⚠️ §722: in those eight places the judge is now `juzgar_historia`, which contains it and adds the handover to it (`spec/PAQUETE.md` 2.3); `juzgar_continuidad` alone remains for the anchor's fork |
| E5b — the catalogue and the bench | the vectors with `actas`, `tools/banco_rotacion.sh` against a real node and the second implementation that reads them | no | **built (§649)**: `spec/vectors/rotacion/`, the artefact's twelfth family, copied from a real node that rotates twice —A, B with A's signature, C without B's—: three positives, the two operator behaviours that the chain gives away seeded with their keys —the old one that signs afterwards, the new one that signs before— and eighteen negatives by one mutation; the second implementation reads `actas` and passes the 23 with the same harness, and its output is the binary's line by line on the positives; since §771, also in consumption and in completeness, from the same node |
| E5c — the overlap envelope | `tipo: "solapamiento"`: one head and the chain; the portable detection of rules 3 and 4 | no (additive: a new envelope) | **built (§686)**: `Tramo` and `tramos_de` in the core —one stretch for each entry of the key in the chain, and the genesis without judging what came before opting in—; a head outside all the stretches of its key is DETECTION with exit 0, the mould of the split view; eleven envelopes derived from the `rotacion/` family without another run, which the second implementation passes; and the refutation of the conflict between two keys of one operator, written: the extension envelope with the chain, not the chain alone. Since §687 the witness assembles it from its journal, `--auditar --sobres` |
| E6 — the medium | the act as a leaf of the medium of the outgoing key and of the incoming one, with E3 of RFC-0013 | no | pending, after E3 of RFC-0013 |

## Motivation

Entry 84 says it plainly: «a settlement system that has no answer to "today the key has to be
changed" is not finished». What the tree does today, MEASURED by reading the code and with the
tests of §636 and §638:

| case | what the tree does today | where |
|---|---|---|
| **the budget** | XMSS^MT-SHA2_40/8_256: 2^40 = 1,099,511,627,776 indices. The heartbeat spends one per head, also without transitions, and nothing else signs with that key: at 60 s that is about 2.09 million years; at 1 s, about 34,800 | `zk_ssl_verify::Conjunto`, `latido::latir` |
| **exhaustion** | there is no threshold and no notice. Read in the code and exercised by no test: `firmar` reserves the index before signing, so at the ceiling the heartbeat fails, records it and keeps burning the counter; on restarting, `IndiceFueraDeCampo` and the node does not start | `FirmanteCabeza::firmar`, `poner_indice_en_sk` |
| **rotation, in the node** | there is no signed transition. The node does not tell a rotation from a restart: a new key with the old one's counter gives `ClaveEnCero`, resynchronises and continues the count; with a new counter and the old journal it does not start | `politica_de_reconciliacion`; tests of §636 and of §638 |
| **rotation, outside** | the witness pins the first key it sees and stops at any other (`CambioDeClave`), and the pinned key lives in memory: a restarted witness pins again. The kit's envelopes that compare heads demand a single key. The anchor carries the key's fingerprint, and the medium carries it in its `origin`: a new key is another medium | `Memoria::anclar`, `claves_distintas`, RFC-0012 D-B, RFC-0013 D-A |
| **the lost index** | the SK is not persisted: the seed gives the key at zero and the counter is the whole state. Counter erased and journal with signatures: it does not start (§594). Counter and journal lost at once: the node signs again from leaf 0, and with two signatures of the same leaf the key is extracted with about 2^34 hashes (QRL's curve, which entry 84 cites) | `GuardianIndice::abrir`, `diario::maximo_indice`; entry 103 |
| **the emergency** | there is no written procedure for «I think I have reused an index». The witness detects it (`indice-repetido`, split view) but there is nothing to do afterwards | entry 92 (c) |
| **the scheme** | no signature says which scheme it was made with: the OID goes inside the published key, which nobody signs | entry 87 |

Two facts from outside set the frame. RFC 10033 (PQUIP, «Hash-Based Signatures: State and Backup
Management», September 2026) proposes a configurable exhaustion notice that demands explicit
acknowledgement (§3.4); it acknowledges that a verifier with all the signatures detects a repeated
index, which is what the witness does (§4); and it says of rotation that «it is not infallible,
because it demands that there be at least one available key that vouches for the new one» (§5.5).
And RFC 8649 —the one NIST SP 800-208 suggests for updating the stored key— solves exactly that in
the PKI: the current root certificate publishes the **hash of the next key**, a commitment that
lets the successor be recognised without ambiguity when it arrives.

And a fact from inside decides when: **there is no key to anchor** (§246). The node signs with test
keys, and the first real key will be born with the deployment. The genesis is the only moment at
which committing the successor costs nothing.

## Design

### D-A — The count of indices belongs to the operator, not to the key

The new key continues the old one's count: it starts at an index `desde` later than everything the
old one signed, and not at zero. It is what the node already does with the old one's counter
(§638), and it turns into a rule what today is an accident:

- the **receipt's era** (`índice + 1`, §567) and its window `S − e ≤ N` stay monotonic from one key
  to the other;
- the **anchor's clock**, the embedded index (RFC-0012 D-C), too;
- the **journal gate** (§594) does not get in the way: the old one's maximum stays below;
- the **witness**, which classifies by index, does not see a repeated index between two keys.

The price is the new key's leaves below `desde`, which are lost: with 2^40, nothing. A jump forward
counts against the operator, as `recibos::dentro_de_ventana` already counts the orphan indices: it
shortens or expires the open windows, and that is paid by whoever jumps, not by the holder.

### D-B — Pre-rotation: each key commits the fingerprint of the next one

Each key enters with an **act** (D-C) that carries the fingerprint of the key that will succeed it.
A rotation holds if the incoming key is exactly the committed one and signs its own act. The
outgoing key also signs the act when its state is reliable. It is RFC 8649 carried over to a
stateful signature, and it solves the three cases in which an act signed only by the old one fails:

1. **The old one compromised.** Whoever stole it can sign an act, but not with the next key, which
   they do not have: they cannot rotate to one of their own.
2. **The old one's index undetermined** (entry 92). Signing the act with the old one risks reusing
   a leaf, which is what one wants to avoid. With pre-rotation the old one need not sign anything
   else.
3. **The old one lost or exhausted.** It cannot sign, and the committed successor rotates all the
   same.

The next key lives **cold**: the node never receives its seed, only its fingerprint. Whatever the
node loses or leaks does not reach it.

### D-C — The act

```text
acta = { v: 1,
         anterior:  huella_de_clave(la clave que se va)          ← ausente en el acta génesis
         clave:     la clave pública que entra, entera
         esquema:   el conjunto de la clave que entra, p. ej. "xmssmt-sha2_40/8_256"
         desde:     el primer índice que firma la clave que entra
         siguiente: huella_de_clave(la clave que sucederá a esta)
         ultima:    { epochDigest, mmrRoot, mmrSize } de la última cabeza de la anterior
                    ← ausente en el acta génesis }

firma de la que entra:  preámbulo(DOMINIO_ACTA, v, acta_digest), con su hoja `desde`
firma de la que se va:  el mismo preámbulo, con su hoja siguiente, si su estado es fiable
```

`huella_de_clave` is that of RFC-0012 D-B, with its length encoded: no other is born. As the
fingerprint covers the whole key, OID included, the XMSS set is committed by it; the `esquema`
field exists for when the incoming key is of another family, which is entry 87. The domain
separates the act from the head and from the co-signature: a head signature is not read as an act.
The first leaf of the incoming key signs its act, and the heads start at the next one.

**The verifier's rules**, the same in the kit, in the witness and in the second implementation:
1. The genesis act has neither `anterior` nor `ultima`, and it is signed by the key it presents.
2. A rotation act holds if its `anterior` is the fingerprint of the key of the previous act, if the
   fingerprint of its `clave` is the `siguiente` of the previous act, if its signature verifies
   with its `clave` and, when it brings the previous key's, if that one verifies too.
3. Its `desde` exceeds every embedded index of the previous key that the verifier has in front of
   it.
4. A head of the key `K` holds if it verifies with `K` and its embedded index is above the `desde`
   of `K`'s act —that leaf belongs to the act— and below that of the next one, if there is one. A
   head of the old one above the `desde` of the new one is **overlap**: opposable evidence with a
   name, like the split view.

**Fixed in E2 (§643).** The exact composition of `acta_digest` is in `NUCLEO.md` §6 and its bytes
in three KATs: a label behind the domain separates the genesis (0) from the rotation (1), and the
provenance enters in front of the body. Today's scheme is `0x1_0000_0005`: the family of RFC 8391
in the high 32 bits and its OID in the low ones; another one is rejected with its number. `desde`
is the EMBEDDED index, and the first signature of the incoming key carries it inside. And
`verificar_acta` judges the rules BEFORE the signatures: an uncommitted key is rejected even if the
new one and the old one sign it, because no signature rescues it.

**Fixed in E3a (§644).** The act enters through start-up and is **opt-in**: without `--siguiente`
and without acts in the journal, the node signs none, and the residue of D-I remains. The
`procedencia` of a rotation is the last head the outgoing key signed and **the accumulator of heads
that the incoming one inherits**: the peak and the size of the journal's MMR, which is the pair its
first head will sign. Without a head of the previous key, its digest is the declared zero of the
MMR's genesis. And in E3a the outgoing key **does not sign**: every rotation act the node signs
declares it burned, until E3b lets it sign when its state is reliable.

**Fixed in E3b-1 (§645).** The outgoing key signs **only if the operator gives it**
(`--clave-anterior-fichero`): giving it is asserting that its state is reliable, and the node cannot
know that on its own. It signs the same preamble as the new one, at the leaf the operator's counter
gives —the count is one (decision 2)—, and the new one starts at the next: two leaves reserved by a
rotation with both signatures, one by the one that declares the old one burned. The seed is checked
against the key of the act in force BEFORE reserving anything. And the ceiling is stated by the
**counter**, not by the SK: after signing with the last leaf, the SK's index still reads
2^40 − 1, because its five-byte field does not represent 2^40, and a second signature does not give
`KeyExhausted` but a signature that does not verify; the signer returns `Agotada` without touching
the counter, and the heartbeat at the ceiling fails without burning one index per heartbeat.

**Fixed in E3b-2 (§646).** The act has **one** JSON and one producer, in the kit: what the journal
line carries, without its `v`, its `tipo` and its `index`, is what the wire serves and what the
envelopes of E5 will carry. `procedencia` and `firmaAnterior` always go, with `null` when there is
none. The node assembles the chain at start-up and does not re-read the journal per request: acts
are only born at start-up, and the journal grows one signature per heartbeat.

**Fixed in E4 (§647).** The judgement of a rotation has one producer, `juzgar_rotacion` in the kit,
which the live witness and `--auditar` use, and the whole chain has one too, `verificar_cadena`,
which the node also uses at start-up. It can cross several links: a witness that was off sees
today's key and not those in between. Rule 3 is applied to what the witness SAW: the highest
embedded index of the pinned key, among the heads that verified, stays below the `desde` of its
successor. Rule 4, to each head of a key that arrived by rotation —with TOFU there is no stretch,
because nobody said it—. Outside its stretch, `solapamiento`, which stops. The rotation is recorded
in its own line, with the chain the node served, and the same head is judged afterwards with the new
key; the leaves the acts spent come out as a `hueco`, which does not stop. The meaning of
`cambio-de-clave` narrows to the change the chain does not explain, and that is why the witness's
journal goes up to v4.

### D-D — Where the act lives

In three places, and in all of them as it is:
1. in the node's **journal**, as a line with its type, recorded by whoever signs (§285);
2. on the **wire**, with `zkssl_keyActs`, which serves the whole chain from the genesis: an
   operator's chain fits in a file that the holder keeps with their heads;
3. in the **medium** of RFC-0013, as a leaf, with its E3: an act published where the operator
   cannot erase it, and two distinct acts for the same key are a split view of acts that the
   medium's witnesses see.

### D-E — The witness and the kit

**The witness** (E4). At a `CambioDeClave` it asks for `zkssl_keyActs`, judges the chain from the
key it pinned with the rules of D-C, and pins the new one if it holds: a new verdict, «rotada»
[rotated], which it records and carries on. If there is no act, or it does not hold, it stops as
today. `--auditar` does the same with the journal. §245 stays the same in what matters: the
operator cannot change key without a third party seeing it; what changes is that now it can
**explain it with an object**, not with an excuse.

**The kit** (E5). The envelopes that compare heads accept an optional field `actas` with the chain
that joins their keys. Without it, today's text, byte for byte: «las cabezas llevan claves
DISTINTAS: la continuidad es de UN firmante» [the heads carry DISTINCT keys: continuity is of ONE
signer], and no existing vector moves. With it, continuity is of ONE operator, with the rules of
D-C.

**Fixed in E5a (§648).** In an envelope there is no witness that saw the rotation go by: there are
two signed heads and a chain. The judge is `juzgar_continuidad`, in the kit, and what orders the
two heads is their EMBEDDED index, not the name of the field that brings them: the count is one
(D-A), so the one with the lower index is the earlier one, be it `vieja`, `cierre` or `juzgada`.
Rule 3 is applied to the earlier one —what the verifier saw the outgoing key sign is that head— and
rule 4 to the later one; its RED is born with a name, `FueraDeTramo`, which is also OVERLAP. With
the same key `actas` is not read: there is no change to explain. What E5a does NOT cover, said in
`spec/PAQUETE.md`: the **conflict** demands distinct keys and is assembled by the denouncer, who
does not bring the chain, so a rotation can be presented to it as two ledgers —whoever judges it
asks both for `zkssl_keyActs`—; the **split view** is of one key, and the overlap between two
deserves its own envelope, with a single head of the old one and the chain; and the **anchor** of
the key that left, extended by a head of the incoming one, is the medium's seam, E6.

**Fixed in E5b (§649).** The catalogue comes from ONE node that rotates twice over the same journal,
and the behaviours the chain gives away are seeded with the real keys, like the anchor's split view:
the old key, already rotated, signing again with its counter restored —its leaf stays above the
`desde` of its successor, rule 3—, and the new one signing with a fresh counter below the leaf of
its act —rule 4—. Only whoever has the keys produces either of the two pairs. Two rules of D-C have
no vector, because a mutation does not produce them without signing again —the leaf of the incoming
key's signature different from its `desde`, that of the outgoing key's outside its stretch—: they
are judged by the core's tests. The second implementation, written from this D-C and section 2.3 of
`spec/PAQUETE.md`, says the same as the reference on every entry.

**And since §771, in consumption and in completeness**, what E5b left: the same bench, against the
same node, publishes a consumption with the new key in force and leaves a receipt with the old one,
and from there come three consumption envelopes and three completeness ones, with `actas`, without
them and with the chain cut. No completeness verdict 0 crosses a rotation: it would ask for a STARK
proof that the bench does not have.

**Fixed in E5c (§686).** The overlap has its envelope, because ONE signature against the chain
proves it: the other head is not needed. Each key has a stretch for every time it enters the chain
—above the `desde` of its act and below that of the next one—, and the genesis admits what its key
signed before opting in, except the leaf of its act: that is the residue of D-I, and judging it
would give away an honest operator. A head outside ALL the stretches of its key is detection, with
exit 0. Whoever denounces cannot fabricate a stretch —the acts are signed by the operator's keys—
and a prefix of the chain only gives away less. And the conflict between two keys of one operator
is refuted with the extension envelope between its two heads, with the chain: the chain alone is
not enough, because two operators in agreement can sign themselves one. ⚠️ §722: and also the
common history. Here it was said they could not, without measuring it: the second one starts its
node over a copy of the first one's journal, whose leaves are public digests, and its MMR contains
the first one's (measured with two real nodes, `tools/banco_historia_comun.sh`). Since the fix the
eight places that demand continuity demand the HANDOVER —the act's `procedencia` ties the MMR of
the two heads, each in the handover of its stretch by its two edges— and the overlap envelope gives
away, with the same judgement, a head of a key outside the handover of its stretch. The refutation
is still possible. A head of the key that left, signed from its handover, gives it away, with any
chain the operator signed —the one with the lowest handover gives away the most—. Nothing gives it
away if the two ledgers ALTERNATE instead of shutting one down, if the second one pads its MMR and
jumps the counter at the same time, or if the operator signs another act of the same transition
with another handover: two valid acts of the same key after the same previous one are an
equivocation of the chain, and they have no envelope yet (`spec/PAQUETE.md` 2.3, 2.5, 2.14 and 8).
Since §687 the witness assembles the envelope from its journal: by rule 4, with the head of the line
and the chain of the rotation that fixed its stretch; by rule 3, with the head of the old key with
the highest leaf —the one the memory uses as what was seen— and the chain of the line that gives it
away.

### D-F — Exhaustion: a notice with a threshold, and the successor already committed

RFC 10033 §3.4, as it is: a configurable threshold `M` of remaining indices. Below it, the node
says so in every heartbeat, and does not start without an explicit acknowledgement from the
operator. At the heartbeat's cadence the ceiling is two million years away and the notice will not
fire; it is put in because it is cheap, because the cadence is configurable, and because with D-B
exhausting the key **is** rotating to the successor, which is already committed. When `firmar`
fails at the ceiling, the heartbeat stops burning the counter.

⚠️ **Made precise in §645.** With D-A, «exhausting the key **is** rotating» does not hold AT the
ceiling: the successor starts at a `desde` above everything the old one signed, with the same
five-byte field for its index, so at 2^40 what is exhausted is the OPERATOR's count, and rotating
gives back no leaves. The ceiling belongs to the operator, not to the key, and the notice of E3a,
which reads the operator's counter, already treats it that way. At the heartbeat's cadence it is
two million years away and it changes nothing of what is built; if it ever came to matter, what is
revised is decision 2, which is reversible, not the signer.

### D-G — The lost index and the emergency, written

An undetermined index is an index that may have been signed (entry 92: «a lost index is better than
an undetermined one»). The rule, written as an operator's procedure and not as code:
1. **If the key's state is not reliable, the key is burned.** It signs nothing more: neither a
   head, nor the act.
2. **One rotates to the committed successor**, which signs its act alone (D-B, case 2).
3. **The successor's `desde` jumps above every known index**: the journal, the witnesses' journals
   and the medium. The jump is paid by the operator (D-A).
4. **«I think I have reused an index»** is the same case. The witness already detects it from
   outside (`indice-repetido`, the split view), and the medium will see it with its E3; the act is
   the operator's answer, and its earlier heads remain evidence against it.

What the node can do by construction (E3): offer the rotation with the `desde` computed from what it
sees, and refuse to start with the burned key if it is told that it is.

**Fixed in §690.** Step 3 has a flag: `--desde-minimo HOJA`, only in a rotation, is the highest leaf
the outgoing key may have signed according to what the operator knows from outside —the witnesses'
journals, the medium, a backup—. The counter jumps above it with `fsync`, the leaves in between stay
burned without a signature, and the successor enters above: with a counter restored behind what the
old one signed, rotating from it would have left its real heads inside the new one's stretch, an
overlap against an operator that only lost its state. And «refuse to start with the burned key» is
already done by the chain: with the journal rotated, the key that left is neither that of the act in
force nor the committed one, and the node neither starts nor spends (measured in §690).

**Since §770 the witness gives that leaf.** `witness --auditar DIARIO` prints, per key, the highest
embedded leaf among the journal's heads that re-verify without the node, also those of another key
and those that fall outside their stretch, which are leaves that key signed. That figure, as it is,
is the one for `--desde-minimo`: the node enters at the next. The medium and the backup are still
the operator's business, and a signature that does not verify gives no leaf.

⚠️ **§722 — step 3, in the other count.** Since the fix, the rotation act also fixes the HANDOVER:
the `mmrSize` and the peak the successor inherits, which the node reads from its journal —the
handover is its number of heads—. `--desde-minimo` jumps by index, not by `mmrSize`: with a snapshot
shorter than what the old one signed and served, the old one's real heads stay above the handover,
and the overlap envelope gives them away with a single one. That is why step 3 is completed:
**before rotating, the journal is rebuilt** with the heads the old one signed —those of the
witnesses, the medium or the clients; for the line, its `epochDigest` is enough—, and the node
demands it with `--relevo-minimo N`, only in a rotation: `N` is the largest `mmrSize` the old one
signed, plus one, and with fewer heads in the journal the node neither signs the act nor spends a
leaf. The figure the witness has given since §770 is the leaf, the one for `--desde-minimo`; this
`N` it does not give yet.

### D-H — Custody, declared as today

The next key is cold by design (D-B). Where the current one lives remains an assertion of the
operator (`--custodia`, §244), checked only for `fichero`. NIST SP 800-208 demands hardware modules
and RFC 10033 §4 recommends them; this proposal does not demand them, and says so.

### D-I — The residue, declared

- **The genesis is TOFU.** Whoever sees it first pins it; if the operator was already lying, it
  pins it all the same (§245). The anchor and the medium narrow the window; they do not close it.
- **Stealing the current key and the next one at once** breaks pre-rotation. It is the same model
  as RFC 8649: the next one is kept apart so that that is harder, not impossible.
- **An operator without a genesis act** stays where it is today: the witness stops at the change,
  and rotating cannot be told apart from a theft. A late genesis holds from when it is published,
  not before.
- **The witness pins in memory**, and a restarted witness pins again. E4 does not change that; its
  journal does see it (`--auditar`).
- **Entry 103 remains open**: losing the counter and the journal at once still switches the gate
  off. D-G says what to do when it is known; when it is not known, no procedure is any use.
- ⚠️ **§722 — what came before opting in, by index yes and by `mmrSize` no.** The genesis stretch
  does not judge what its key signed before opting in; the handover of the first
  rotation, on the other hand, does: a genesis signed over a new journal leaves its old heads above
  that handover, and the overlap envelope gives them away. The same goes for a rotation from a
  journal restored without the heads the old one signed (D-G, step 3). One opts in over the usual
  journal, and rotates with it complete.
- **The witness's co-signer** has the same lifecycle and the same guardian (§298). This proposal is
  for the operator's key; the witness's would follow the same rules in a stage of its own.

## Decisions

The five that this text leaves to the author, with the one it recommends first:

1. **Who signs the act (D-B).** Pre-rotation —the committed successor signs, and the old one too
   when its state is reliable— versus an act signed only by the old one. The second is simpler and
   fails exactly in the three cases in which it is needed.
2. **The count (D-A).** The new key continues the count, versus each one starting at zero. Starting
   at zero forces rewriting the receipt's era, the anchor's clock and the journal gate as pairs
   `(clave, índice)`, and a standard with two clocks per operator is harder to copy.
3. **The medium at a rotation (D-D).** One medium per key —the `origin` of RFC-0013 is still the
   key's fingerprint, and the act is the last leaf of the outgoing medium and the first of the
   incoming one— versus one medium per operator, with the `origin` at the fingerprint of the
   genesis act. The first touches nothing built of RFC-0013 and leaves the medium's witnesses with
   the C2SP rule, one `origin` per key; the second saves reconfiguring witnesses at every rotation.
4. **The notice threshold (D-F).** By default, one year of heartbeats at the configured cadence,
   with explicit acknowledgement to start below it, versus only a notice.
5. **The old one's signature (D-B).** Mandatory when its state is reliable and forbidden when it is
   not, versus always optional. The first gives the verifier one more signature where it can, and a
   rule that the node checks.

**TAKEN in §642**, by delegation of the author and with the criterion he dictated for §609
(RFC-0014, «Decisiones» [Decisions]): the principles of the project and its front page, applicable
to the largest number of use cases without significant modifications, because the goal is to set
the standard. The five, the recommended ones, and all REVERSIBLE.

- **1, pre-rotation (D-B).** It is the only one of the two that answers in the three cases in which
  a rotation is needed —the key stolen, with the index undetermined, or lost—, and the one that has
  a precedent with a name: RFC 8649, which NIST SP 800-208 suggests. For the verifier it is two more
  rules (the committed fingerprint and the successor's signature), and for the operator, a cold key
  that the node does not see: it holds for any operator, with any custody.
- **2, the count continues (D-A).** One clock per operator, not one per key: the receipt's era, the
  anchor's clock, the journal gate and the witness's classification do not change a line, and it is
  what the node already does with the old one's counter (§638). Starting at zero would have put
  pairs `(clave, índice)` in four rules, and another implementation would have to copy the four.
- **3, one medium per key (D-D).** It touches nothing built of RFC-0013 —neither
  `origen_del_medio`, nor its 27 envelopes, nor the 33 notes contrasted with torchwood— and leaves
  the medium's witnesses with the C2SP rule, one `origin` per key, which is what a generic witness
  knows how to do. The act is the seam: the last leaf of the outgoing medium and the first of the
  incoming one. The price, reconfiguring the medium's witnesses at every rotation, is that of a rare
  operation.
- **4, the default threshold is one year of heartbeats, with explicit acknowledgement (D-F).** It is
  RFC 10033 §3.4 as it is —a notice that is not skipped by accident—, measured in the unit the
  operator configures: one year at the cadence of its heartbeat, whatever it is. Only a notice would
  have been one more message in a log.
- **5, the old one's signature, mandatory if its state is reliable and forbidden if not (D-B).** A
  rule that the node checks instead of an option. The verifier accepts the act with the old one's
  signature or without it, and if it brings it, demands it valid; an act without it is the
  operator's declaration that the old one is burned (D-G), and it stays written in the act that the
  operator signs.

## What was DISCARDED on measuring

- **Variable-length signature chains** —the last leaf of a tree signs the root of the next one—:
  RFC 10033 §5.6 discards them because of the signature's variable length and because one same tree
  would sign messages and trees with very different authorisations. The act is another object, with
  its domain, and the head does not change length.
- **Several valid keys at once** (SP 800-208, in RFC 10033 §5.1): it breaks «continuity is of ONE
  signer» and leaves invisible a split view spread across keys. Pre-rotation is sequential: one
  current key, one committed.
- **The key or the scheme inside the head**: a v7 of the head and all its vectors, to say in every
  heartbeat what the act says once per key.
- **Persisting the SK** instead of the seed and the counter: RFC 10033 §4 asks that the state and
  the key cannot diverge or be cloned, and today's design (§328–§337) already decides that with the
  guardian. It is not touched.

## Compatibility

`zkssl/0.4` does not go up. The v6 head does not move a single byte. A family is born, the act,
with its domain in the REGISTRY of `NUCLEO.md`, its KAT and its vectors; a read method,
`zkssl_keyActs`; and an optional field, `actas`, in the envelopes that compare heads. No existing
vector moves, and neither does the text of `claves_distintas`. A node without a genesis act keeps
working as today. What changes in semantics is the witness with E4: it stopped at every change of
key, and it would carry on at one with a valid act.

## Security

- **The spending key never travels**: this proposal does not touch it. All the keys of this text
  are the operator's.
- **The act is not a head**: its own domain, like the co-signature of §297; a signature cannot be
  reused from one family in another.
- **A rotation erases nothing**: the old key's heads keep verifying with it, and they remain
  evidence against the operator. Rotating does not escape a split view: it documents it.
- **The jump of `desde`** can expire windows of open receipts. It counts against the operator
  (D-A), who is the one who decides to jump.
- **Availability**: rotating needs the next key. If the current one and the next one are lost, the
  operator starts another identity, and its holders keep what they hold in custody
  (`spec/RPC.md`, «Apagado» [shutdown]).

## References

- RFC 10033, Wiggers, Bashiri, Kölbl, Goodman and Kousidis, «Hash-Based Signatures: State and
  Backup Management», IETF PQUIP, September 2026: §3.4 (notice threshold), §4 (requirements of the
  state; the verifier that detects a repeated index), §5.1 (several keys), §5.5 (rotation), §5.6
  (variable-length chains).
- RFC 8649, Housley, «Hash Of Root Key Certificate Extension», August 2019: the hash of the next
  key in the current certificate.
- NIST SP 800-208, Cooper et al., «Recommendation for Stateful Hash-Based Signature Schemes»,
  October 2020.
- RFC 8391 (XMSS), and FIPS 205 (SLH-DSA, stateless), for entry 87.
- In this tree: entries 84, 92, 19, 87 and 103 of `BACKLOG.md`; RFC-0010 (the receipt and its era),
  0012 (the anchor and the key's fingerprint) and 0013 (the medium); `doc/ecst/ECST.md`, which
  leaves written that «key rotation remains open».
