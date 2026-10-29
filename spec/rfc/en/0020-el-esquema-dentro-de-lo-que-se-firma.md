# RFC-0020 — The scheme inside what is signed: the head and the co-signature say what they were signed with, and a head of scheme A keeps verifying when the node signs with B

> **Español:**
> [`spec/rfc/0020-el-esquema-dentro-de-lo-que-se-firma.md`](../0020-el-esquema-dentro-de-lo-que-se-firma.md)
> is the original of this page, in Spanish; it prevails wherever the two differ. This translation is
> of commit `7766efb` (audit entry §917) and was produced by the project's assistant, a Claude Code
> session, in audit entry §918 (`GENAI.md`): it is a generated artefact, not text written by the
> author. Line citations of the form `file:N` refer to the Spanish files.

- **Status:** PROPOSED (§790) — the text, its stages and its seven decisions, written on what was
  measured in the tree at `8c02941`. The author accepted it in §799, with its seven decisions TAKEN,
  in each one the one the text recommends: see «Decisions (the author's, §799)». E2, the core
  switched off, built in §800, and E3, the dispatch, in §862; the rest, unbuilt: ACCEPTED arrives with them, because moving
  to ACCEPTED requires building its stages (rule 4 of the PROCESO). ⚠️ §799: until here it said «No
  decision is taken and no stage built: adopting it, in whole or in part, is the author's».
  ⚠️ §800: until here it said «No stage built: ACCEPTED arrives with them».
  ⚠️ §862: until here it said «E2, the core switched off, built in §800; the rest, unbuilt».
- **Author:** Ángel José Toranzo Portela
- **GenAI assistance:** Claude, in a Claude Code session in the cloud (§790): it measured the tree
  and wrote the draft, outside step 4 of `GENAI.md` — see [`GENAI.md`](../../../GENAI.md)
- **Date:** 2026-10-14
- **Protocol version affected:** `zkssl/0.4` -> `zkssl/0.5`, with E4: the v7 head changes
  what the node signs and serves, and a verifier of today rejects it. It travels in the cut of
  RFC-0018 (decision 7, taken in §799). The v2 to v6 heads, the acts and the published vectors do
  not change a byte. ⚠️ §799: until here it said «if it is adopted» where it says «with E4», and
  «It proposes to travel in the cut».
- **AUDITORIA entry(ies):** §236 (a single version marker), §240 (the OID workaround), §275 (the
  version byte chooses the recomposer), §297 (the co-signature), §406 (`VersionCabeza`), §643 and
  §644 (the act, with its scheme, and its opt-in), §699 (what a version does not identify), §790
  (this draft), §799 (the author's acceptance, with its seven decisions), §800 (E2: the registry and
  the two v7 preambles, switched off), §826 (the surface E4 breaks, in its table), §852 (the two citations of `spec/vectors/paquete/MANIFIESTO.txt` of the seven lines that move, which its five negatives shifted, with its mark), §862 (E3: the dispatch by scheme, built, with its marks). ⚠️ §826: it ended at §800, and was silent on §852, which touched this file with its mark. ⚠️ §862: it ended at §852.
- **Backlog:** entry **87** (cryptographic agility: the signature scheme is not versioned), which
  this RFC does not close until it is built; with entry **84** (changing scheme is a rotation) and
  entry **108** (which scheme B would be: it is not measured here).

## State of the stages

| stage | what it delivers | does it break the wire? | state |
|---|---|---|---|
| E1 — the rule, written | this text: the identifier, where it enters, the rule of the transition and its cost | no | **written (§790) and decided (§799)** ⚠️ §799: until here it said «proposed (§790)» |
| E2 — the core, switched off | the registry of schemes in `spec/NUCLEO.md` §6, the v7 preamble of the head and that of the co-signature with their KATs, and the second implementation that reproduces them; nobody signs them yet | no (additive: the registry, two functions and three KATs) | **built (§800)**: `Esquema` and `preambulo_v7`/`preambulo_cofirma_v7` in `zk-ssl-verify`, `registro_de_esquemas.json`, `preambulo_v7.json` and `preambulo_cofirma_v7.json` in `spec/vectors/nucleo/`, and `tools/segunda` which reproduces them; neither the node, nor the witness, nor the kit sign them or accept them ⚠️ §800: until here it said «unbuilt» |
| E3 — the dispatch | the verifier chooses the verifier by the scheme, with one accepted set and another issued; a scheme that is not accepted is rejected with its number | no (with a single registered scheme, nothing changes verdict) ⚠️ §862: none of the wire nor of S1 to S6; the library changes, which the D-A of RFC-0021 leaves outside the promise: `verificar_acta` with a previous act of a scheme the registry does not assign and the signature of the outgoing key gives, if it gets to judge it, `FirmaDeLaAnterior(EsquemaNoRegistrado)`, before reading its key, so what gave `Ok` turns red and what already fell —by `IndiceDeLaAnterior` or by another cause of `FirmaDeLaAnterior`— falls by that one, with its text; neither the kit nor the node reach it, because the previous act reaches them already judged | **built (§862)**: `Esquema::ACEPTADOS`, `Esquema::EMITIDOS` and `Esquema::IMPLICITO` in `zk-ssl-verify`, bound in a test; `verificar_cabeza`, `verificar_cofirma` and `verificar_acta` choose the verifier by the scheme —the implicit one for every version byte until E4— and reject with its number the one that is not in the registry and the one that is not accepted, before reading the key (`VerificaError::EsquemaNoRegistrado` and `::EsquemaNoAceptado`); the act judges the signature of the outgoing key with the scheme of the previous act, and keeps its text; `tools/segunda/juez_cabezas.py` and `cable.py` go through the dispatch, `verificador.py` does not; v7 stays switched off ⚠️ §862: until here it said «unbuilt» |
| E4 — the cut | `VersionCabeza::V7` in the set; the node signs and serves v7 with `esquema`; the witness co-signs v7; the kit reads `esquema` in every head of its envelopes; the vectors that change text, moved to `0.4/` | **yes**: S3, the v7 head and co-signature, with their `esquema` under the signature; S1, because the wire serves them; and S4, because the kit reads `esquema` in every head of its envelopes; in S5 it does not break: the seven manifest lines that change text move to `0.4/` with their vectors (R6 b). ⚠️ §826: it said «**yes**», without the surface that R5 of RFC-0021 asks for | unbuilt; in the seal of the `zkssl/0.5` cut, as the author decided in §799 (decision 7) ⚠️ §799: until here it said «if the author decides it so» ⚠️ §913: the author decided on 29-10-2026 that the `zkssl/0.5` cut carries only D-H of RFC-0018; this stage moves to the next break, `zkssl/0.6` (RFC-0018, the note of §913 in D-H) |
| E5 — the transition, measured | a second registered scheme, a rotation A -> B on a real node and a head of A judged by containment under B, in a bench | depending on scheme B | unbuilt; it waits for entry 108 and for the author's choice |

## Motivation

Entry 87 asks for «a scheme identifier **inside what is signed**, and the written rule of how a
head signed with scheme A keeps verifying when the node already issues with B»
(`BACKLOG.md`, entry 87, «Qué lo cierra» [what closes it]). RFC-0015 left it said in its table: «no
signature says with which scheme it was made: the OID goes inside the published key, which nobody
signs» (`spec/rfc/0015-el-ciclo-de-vida-del-firmante.md:61`). What is signed today, measured in
this tree:

| what is signed | its bytes | does it carry the scheme? | where |
|---|---|---|---|
| **the head** | `b"ZK-SSL-epoch-head" ‖ version ‖ epoch_digest`, 50 B | **no** | `spec/NUCLEO.md:329`; `crates/zk-ssl-verify/src/lib.rs:426-432`; the node signs it in `crates/zk-ssl-node/src/firma_cabeza.rs:221` |
| **the co-signature** | `b"ZK-SSL-witness-cosign" ‖ version ‖ epoch_digest ‖ len(u16) ‖ clave_op`, 56 + N B | **the witness's, no**; the operator's, only as the four bytes of the OID its key carries inside | `spec/NUCLEO.md:330`; `lib.rs:463-480`; the witness signs it in `crates/zk-ssl-cli/src/witness.rs:3440` |
| **the act** | `b"ZK-SSL-key-act" ‖ version ‖ acta_digest`, 47 B | **yes**, that of the incoming key: `acta_digest` composes `as_digest(esquema)` in its body | `spec/NUCLEO.md:331`; `crates/zk-ssl-hash/src/lib.rs:861-884` (the scheme, `:871`); `crates/zk-ssl-verify/src/actas.rs:35-49` |
| the fingerprint of a key | `Blake3(b"ZK-SSL-anchor-key-v1" ‖ len ‖ clave)` | no: it covers the bytes of the key | `spec/NUCLEO.md:355`; `crates/zk-ssl-hash/src/lib.rs:1845-1847` |
| the medium's note | the C2SP line, `key_id = SHA-256(nombre ‖ "\n" ‖ 0x06 ‖ clave)[:4]` | yes, the type of `signed-note`, outside the core | `spec/rfc/0013-el-medio-del-ancla.md:129-132` |

Three more facts, from the code:

1. **The verifier fixes the scheme in the code, not in what it reads.** `Conjunto =
   XmssMtSha2_40_8_256` (`crates/zk-ssl-verify/src/lib.rs:145`), and `clave_desde_bytes` only knows
   how to read a key of that set (`lib.rs:621-639`). The second implementation chooses it by the
   OID of the published key, from which it derives the length of the signature to separate it from
   the preamble that goes attached (`tools/segunda/juez_cabezas.py:71-75` and `:91-92`;
   `tools/segunda/kat_xmss/xmss.py:151-163`). With one scheme, the two things give the same; with
   two, whoever verifies has to know which one before verifying, and that is what entry 87 asks to
   bind to the signature. ⚠️ §862: since E3, the verifier chooses by the scheme, and `juez_cabezas.py` and `cable.py` too: they take the registry entry of the scheme of the signature —in v1 to v6, the implicit one— and from it the length with which `partir` splits the signature; `partir` is still at `:71-75` and receives the entry, and `:103-106` passes the key through the dispatch —which requires a key of the length and the prefix of the entry— before reading its OID and splitting the signature, so the OID the judge reads afterwards, for the embedded index, is that of a key already checked. `cable.py` does not check the key against the entry: it keeps anchoring it by the house's OID and measuring with that OID the length of the signature and its index, which with a single scheme gives the same. `tools/segunda/verificador.py` still chooses by the OID.
2. **The OID of the key does not identify the scheme even inside RFC 8391.** Its two registries,
   XMSS and XMSS^MT, both start at 1, and 5 is valid in both (`lib.rs:535-537`, §240). The act
   resolves it with the family in the high 32 bits: `ESQUEMA_XMSSMT_SHA2_40_8_256 = (1 << 32) |
   0x0000_0005` (`actas.rs:35-38`), and `verificar_acta` rejects another with its number before
   looking at anything else (`actas.rs:196-199`). The comment says «the family in the high 32 bits
   (1, the registry of RFC 8391)» and does not say which of the two registries is family 1.
3. **The only signed place that names the scheme is optional and does not cover the witness.** The
   act is opt-in: without `--siguiente` and without acts in the journal the node signs none
   (`spec/rfc/0015-el-ciclo-de-vida-del-firmante.md:154-155`, §644), and a ledger without a genesis
   act has nothing signed that says its scheme. The witnesses' keys have no acts: RFC-0015 «is for
   the operator's key» (`spec/rfc/0015-el-ciclo-de-vida-del-firmante.md:319-320`).

**What this RFC reopens, and why.** RFC-0015 discarded «the key or the scheme inside the head: a v7
of the head and all its vectors, to say in every heartbeat what the act says once per key»
(`spec/rfc/0015-el-ciclo-de-vida-del-firmante.md:381-382`). The argument holds for a node with
acts. It did not weigh facts 1 and 3: that the act is optional, that the co-signature has none, and
that the verifier has to choose the verifier before reading the act of the key. Decision 1, below,
left the author the two branches: v7, or staying with the act and requiring it for every scheme
other than today's; the author chose v7 in §799. ⚠️ §799: until here it said «leaves the author the
two branches», without the choice.

**How much what is signed weighs, measured.** Of the 492 `.json` files of `spec/vectors/`, 405
carry at least one XMSS^MT signature with its attached preamble: 678 occurrences, of which 536 are
of a head (preamble of 50 B), 127 of an act (47 B) and 15 of a co-signature (124 B, with the
operator's key of 68 B). Distinct ones are 60 —49 of a head, 9 of an act and 2 of a co-signature—:
many vectors derive from the same captures. Each signature measures 18.469 B plus its preamble
(`FIRMA_RFC_BYTES`, `lib.rs:287`; the test at `lib.rs:821`). Measured with a Python walk that looks
in each vector for the hex strings of 18.469 B plus a preamble and reads the domain that opens it.

⚠️ §862: the 23 line citations of this document to `crates/zk-ssl-verify/src/lib.rs` and to
`actas.rs`, and the four of the table above to `spec/NUCLEO.md`, belong to the tree its header
declares, `8c02941`, which §800 already took as a dated measurement, and this clone does not have
it: they have not been checked there. At `213b0b0` they no longer fell in their place —`preambulo`,
which the table cites at `lib.rs:426-432`, was at `:540`; the rejection of the scheme in
`verificar_acta`, cited at `actas.rs:196-199`, at `:202-204`; and the preambles of
`spec/NUCLEO.md`, cited at `:329-331`, at `:351-353`—, and E3 shifts further those of `lib.rs` and
those of `spec/NUCLEO.md`, and by one line `actas.rs:35-38` and `:35-49` —`preambulo` moves to
`:610`, and the preambles, to `:354-356`— and leaves the rejection of the act at `:202-204`. They
are not replaced by today's, which the next seal would move again: each piece is found by the name
the text gives next to it. Those of `tools/segunda/juez_cabezas.py` in fact 1 carry their mark.

## Design

### D-A — The identifier: the `esquema` of the act, a `u64`, with a registry

A single identifier for the three signatures: the `esquema` the act already carries, a `u64` with
the family in the high 32 bits and the family's parameter in the low ones (`actas.rs:35-38`). One
encoding and not two, as RFC-0016 asks: the act, the head and the co-signature name a scheme with
the same number.

- **The registry** goes to `spec/NUCLEO.md` §6, next to the domains, with a single producer in the
  code —the mould of `VersionCabeza`, `lib.rs:181-210`—, and its KAT. It is born with one entry:
  `0x1_0000_0005`, XMSS^MT-SHA2_40/8_256, and the registry says which registry of RFC 8391 family 1
  is (fact 2).
- **Each entry declares** the encoding of its public key, that of its signature —for family 1,
  today's: the RFC 8391 signature with the attached preamble behind (`lib.rs:821`)—, whether it has
  state (and with it, a guardian) and the text with which it is named in the rejections.
- **Zero is not assigned**, as in the v2 mark of the D-G of RFC-0018: a signer that does not
  declare its scheme does not write a valid number.
- **No other entry is assigned here.** Which scheme B would be is measured by entry 108; the number
  is given by the registry when it enters.

### D-B — In the head: the v7 preamble

```text
v2 a v6, hoy:  b"ZK-SSL-epoch-head" ‖ version ‖ epoch_digest                (17 + 1 + 32 = 50)
v7:            b"ZK-SSL-epoch-head" ‖ 7 ‖ esquema (u64 BE) ‖ epoch_digest    (17 + 1 + 8 + 32 = 58)
```

- **The mould is that of §275**: the version byte of the preamble is the only disambiguator (§236),
  and the version the signature declares chooses the recomposer, «in the library and in the
  command» (`spec/RPC.md:602-606`). Here it also chooses the form of the preamble: with 7, it
  carries the scheme. The domain still has no version inside (`lib.rs:146-148`).
- **v7 composes like v6.** `epoch_digest` does not change: the scheme belongs to the signature, not
  to the ledger, and it enters nothing that composes over the digest —the MMR of heads, the anchor,
  the acknowledgements, the receipts—. The doc of `VERSION_FORMATO`, which says it rises «when the
  fields of `EpochHead` change» (`lib.rs:165-166`), gains «or the form of the preamble».
- **On the wire**, `zkssl_signedEpochHead` serves `esquema` (QUANTITY) next to `formatVersion`, and
  so does every head of a kit envelope (`cabeza`, `vieja`, `nueva`, `cierre` and the rest). Whoever
  verifies reads it before the key, chooses the verifier with it (D-E) and compares the recovered
  preamble with the one it composes with that scheme: lying about it is `PreambuloDistinto`, like
  lying about `formatVersion` (`lib.rs:669-676`).
- **In a v1 to v6 head the field is not read**: what is not signed does not exist for the core
  (`spec/NUCLEO.md` §«3. La regla de extensión, ejercida» [the extension rule, exercised], second
  half). Its scheme is the implicit one, `0x1_0000_0005`, the only one that ever signed a head. v1
  is not read by the kit, but by the library (`spec/PAQUETE.md:742-743`), and the second one
  recomposes it (`tools/segunda/juez_cabezas.py:66`).

### D-C — In the co-signature: the two schemes

```text
hasta v6:  b"ZK-SSL-witness-cosign" ‖ version ‖ epoch_digest ‖ len(u16 BE) ‖ clave_op
v7:        b"ZK-SSL-witness-cosign" ‖ 7 ‖ esquema_testigo (u64 BE) ‖ esquema_op (u64 BE)
             ‖ epoch_digest ‖ len(u16 BE) ‖ clave_op                                 (72 + N)
```

The version byte of the co-signature is that of the head it co-signs (`lib.rs:722`; §780), so a v7
co-signature is that of a v7 head, and the prefixed field stays last, «by design»
(`lib.rs:461-462`). The witness's scheme is that of its signature, which today nothing names (fact
3). The operator's makes the co-signature say «this operator, with this key **of this scheme**,
published this digest»: the operator's key goes inside so that the co-signature is not transferable
(`lib.rs:440-444`), and the same reason holds for its scheme. The author wanted it in §799
(decision 3); without it there would be eight bytes fewer. ⚠️ §799: until here it said «If the
author does not want it, it is eight bytes fewer».
`Cosig` (`spec/RPC.md:992-994`) gains `esquemaTestigo` and, with decision 3, `esquemaOperador`.

### D-D — In the act: nothing

The act already signs the scheme of the incoming key, and the chain of acts that of every key of
the operator. The signature of the outgoing key, when it carries it, is verified with the scheme of
the previous act. What the act does not bind is the scheme of the **successor**: `siguiente` is the
fingerprint of its key, and the fingerprint covers bytes, not a scheme
(`crates/zk-ssl-hash/src/lib.rs:1845-1847`). RFC-0015 took it as covered inside XMSS —«since the
fingerprint covers the whole key, OID included, the XMSS set is committed by it; the `esquema`
field exists for when the incoming key is of another family, which is entry 87»
(`spec/rfc/0015-el-ciclo-de-vida-del-firmante.md:128-130`)—, and by fact 2 the OID, alone, does not
identify the scheme even inside RFC 8391. If two accepted schemes admitted the same key encoding,
the pre-rotation commitment would not say which. It is decision 4.

### D-E — The transition rule: how a head of A keeps verifying when the node signs with B

1. **The declared scheme chooses the verifier, and the signature binds it.** In a v7 head or
   co-signature, the preamble's; in a v1 to v6 one, the implicit one, `0x1_0000_0005`. A signature
   of another scheme over a v1 to v6 preamble does not exist: it is rejected by name.
2. **Accepting and issuing are two sets.** The node issues with one scheme; the verifier accepts a
   set that contains it. It is the mould of `COFIRMA_VERSION` and `COFIRMA_V_MAX`: «whoever reads
   can be ahead of whoever writes and **never the other way round**» (`lib.rs:308-314`). The
   accepted set has one producer and grows like that of head versions (`spec/NUCLEO.md` §«3. La
   regla de extensión, ejercida», first half); unlike that one, it can lose a scheme, and only by
   rule 5.
3. **Changing scheme is rotating** (entry 84). Key B enters with its act, with `esquema` B, signed
   by B and, if the state of A is reliable, by A; and B is the `siguiente` that A committed
   (RFC-0015, D-B). Without an act there is no change of scheme: a head of a scheme other than that
   of the fixed key is a `cambio-de-clave`, which halts, because a key of another scheme is another
   key.
4. **A head of A keeps verifying with A** after the rotation, within its stretch (rule 4 of the
   D-C of RFC-0015, `spec/rfc/0015-el-ciclo-de-vida-del-firmante.md:141-143`), as long as A is in
   the accepted set. That the node stops being able to sign with A —the dependency dies: today
   `xmss = "=0.1.0-pre.0"`, without an independent audit (`crates/zk-ssl-verify/Cargo.toml:53-57`)—
   does not take it out of the set: verifying does not need the signer, and the verifier of A has
   two implementations, the pinned crate and `tools/segunda/kat_xmss/xmss.py`.
5. **A leaves the accepted set only if it is broken**, and that is a break with its version
   (RFC-0021). From then on a head of A is worth something only **contained** in something signed
   with an accepted scheme: with an extension proof (§293, the MMR of §291) up to a head of B, or
   up to the act of rotation to B, whose `procedencia` signs the last head of A and the MMR the
   incoming key inherits (`spec/rfc/0015-el-ciclo-de-vida-del-firmante.md:121` and `:154-158`;
   `crates/zk-ssl-hash/src/lib.rs:875-881`). A head of A that is not contained —signed after the
   rotation, or fabricated after the break— is not worth anything. That this reaches all the heads
   of A is DERIVED from reading the code; E5 measures it.
6. **A scheme the verifier does not accept is rejected with its number**, before looking at
   anything else, as `verificar_acta` already does (`actas.rs:198-199`), and with the text of the
   registry. It fails closed: a verifier that does not know B does not give VERDE to a head of B,
   nor does it try it with A. ⚠️ §862: so do the head and the co-signature since E3, with the text
   of the registry for the number that is not in it and another, with its name, for the one that is
   and is not accepted; the act keeps its own, «el acta presenta una clave de esquema {e:#x}, que
   este verificador no conoce» [the act presents a key of scheme {e:#x}, which this verifier does
   not know], because a published vector pins it (`spec/vectors/rotacion/MANIFIESTO.txt:43`) and
   changing it would be a break (R1 of RFC-0021; its R6 b would move it).

### D-F — The cost

**In bytes.** Eight per signed head, 50 -> 58 B over a signature of 18.469 B (0,04 %); sixteen per
co-signature with decision 3, 124 -> 140 B with today's operator key, or eight without it; zero in
the act. On the wire, one QUANTITY field per head and one or two per `Cosig`. No extra hash
permutation: the preamble is bytes the scheme digests when signing, and nothing of the core
composes over it.

**In vectors, measured.** None of the 405 files with a signature is rewritten: the v2 to v6 heads
keep their preamble and their verdict. What is born and what moves:

- **Born**, in `spec/vectors/nucleo/`, three KATs: the v7 preamble of the head and that of the
  co-signature, next to `preambulo.json` (version 3) and `preambulo_cofirma.json`, which are not
  touched; and that of the registry. They were born in §800, with E2: `preambulo_v7.json`,
  `preambulo_cofirma_v7.json` and `registro_de_esquemas.json`. In `spec/vectors/cable/`, a v7
  positive captured from a real node, as the rule of «one positive (`nueva`) per head era» asks
  (`spec/RPC.md:1262-1264`), and its negatives: without `esquema`, with a scheme that is not in the
  registry, with zero, with the declared one different from the signed one, and a v7 that does not
  recompose.
- **Seven manifest lines move, with their vectors**, because they pin the whole enumeration of the
  set of accepted versions, which `VersionCabeza::texto()` and its sisters derive: with v7 inside,
  «v2, v3, v4, v5 o v6» comes to say something else. They are
  `spec/vectors/cable/MANIFIESTO.txt:17`, `:18` and `:21`, `spec/vectors/paquete/MANIFIESTO.txt:26`
  and `:30` (⚠️ §852: it said `:21` and `:25`; the five lines §852 inserts after `:10` shift them
  down by five), `spec/vectors/rechazo/MANIFIESTO.txt:50` and
  `spec/vectors/rotacion/MANIFIESTO.txt:72`, found with `grep` over the eighteen manifests; that
  they change is DERIVED from reading who composes their text. Those that pin only the beginning
  —«exige una cabeza v5» [requires a v5 head], «exige cabezas v4» [requires v4 heads]— still hold,
  because the manifest asks that the output *contain* its text (`spec/RPC.md:1272`). Two of the
  seven belong to vectors that bring `formatVersion` 7 as an unknown version
  (`cable/rechazo-formatVersion-7.json` and `paquete/rechazo-formatVersion-7.json`), and with v7
  their cause changes. By rule 2 of the PROCESO they move identical to `spec/vectors/0.4/`, with
  their manifest line, as §682 moved the two of the completeness
  (`spec/rfc/0019-la-completitud-que-no-se-esquiva.md:54-57`), and their role is taken by a vector
  with another name, with `formatVersion` 8. Before §692 such a line was rewritten; since then its
  fingerprint prevents it (`spec/vectors/HUELLAS.sha256:1-8`).
- **At the cut**, the `zkssl/0.5` positives of each family of the kit that the D-G of RFC-0018
  already plans (`spec/rfc/0018-el-tren-0-5.md:398-403`) carry v7 heads without one more file.

### D-G — Its relation to the `zkssl/0.5` cut

A v7 head is a break for whoever verifies today: the witness and kit 0.4.x reject it, with
«formatVersion 7: el testigo recompone cabezas v2, v3, v4, v5 o v6» [formatVersion 7: the witness
recomposes v2, v3, v4, v5 or v6 heads]
(`spec/vectors/cable/MANIFIESTO.txt:21`) and its pair of the package, from the text of the library
(`lib.rs:269-273`): the correct failure, closed and named. The break is paid once for several
reasons together (the corollary of entry 82, `BACKLOG.md`, entry 82), and the cut of RFC-0018
already breaks the wire and issues new vectors. So, as the author adopted it in §799 (decision 7),
**it travels in the same break** (⚠️ §799: until here it said «if the author adopts it»): E4
enters in the seal of the cut, and the D-E of RFC-0018 governs it —kit 0.5.0 comes out when the
node speaks `zkssl/0.5`, not before and without accepting the two versions of the wire
(`spec/rfc/0018-el-tren-0-5.md:160-161`)—. Kit 0.5 accepts v2 to v7 heads: heads are not proofs,
and their set only grows. What changes no byte and no verdict —the registry, the two preamble
functions, their KATs, the replica of the second one and the dispatch with a single scheme— can
enter earlier, switched off (E2 and E3), as the D-G says of its reader of the mark
(`spec/rfc/0018-el-tren-0-5.md:393-396`). `VersionCabeza::V7` cannot: it changes the seven texts
above, and enters at the cut.

The D-G of RFC-0018 decided to put the identity of a proof in the proof and not in the head
(`spec/rfc/0018-el-tren-0-5.md:342-347`). This does not contradict it: each object carries its
identity, and what is not a proof carries its own version (its «Regla de extensión» [extension rule], `:334-340`). The
mark says which proof it is; the v7 preamble, what the head was signed with.

## Compatibility

With E4, the wire rises to `zkssl/0.5` at the cut of RFC-0018 (decision 7, taken in §799). The
custodied v2 to v6 heads keep verifying with the kit of their version and with 0.5, with their
implicit scheme; the `zkssl/0.4` vectors stay with their bytes and their verdict, and the seven
above move to `0.4/`. Kit 0.4.x does not verify a v7 head, and says so. The act does not change:
decision 4 took branch (a). E2 and E3 change nothing that travels. ⚠️ §799: until here it said «at
the cut of RFC-0018 or at one of its own (decision 7)» and «The act does not change, except with
decision 4, branch (b)».

## Decisions (the author's, §799)

⚠️ §799: until here this section was titled «Open decisions (the author's)» and began «None is
taken. The session says what it recommends and why; the author decides».

The author accepted this RFC in §799, and with it took the seven decisions: in each one, the branch
the text recommends. The questions and their branches stay as they were written in §790; after each
one, the taken one and the discarded ones. The RFC stays PROPOSED until its stages are built.

1. **Where does the identifier go?** (a) In the preamble, v7, for the head and the co-signature
   (D-B, D-C); (b) only in the act, as today, requiring it for every scheme other than the implicit
   one —no new bytes or vectors, but nothing for the witness nor for a ledger without a genesis—;
   (c) inside `epoch_digest`, which would put it in the MMR, the anchor and the acknowledgements.
   The session recommends (a): it is the only one that covers the co-signature and the head without
   an act, at the price of D-F, and (b) is the one RFC-0015 chose for a node with acts.
   **Taken (§799): (a)**, the v7 preamble of the head and of the co-signature. Discarded: (b) and
   (c).
2. **How is it written?** The `u64` of the act, eight bytes (recommended: one encoding per scheme),
   or an index of a table in one or two bytes, which would save six and would force translating
   between the act and the head.
   **Taken (§799): the `u64` of the act**, eight bytes. Discarded: the index of a table.
3. **Does the co-signature also sign the operator's scheme?** Eight more bytes per co-signature.
   The session recommends it, for the reason for which it already signs its key.
   **Taken (§799): yes**, `esquema_testigo` and `esquema_op` in the v7 co-signature (D-C).
   Discarded: signing only the witness's.
4. **Does the commitment of the successor bind its scheme?** (a) A rule of the registry: two
   accepted schemes share no key encoding, and the registry checks it when admitting one; (b) a v2
   act that composes `esquema_siguiente` next to `siguiente` (two new KATs, and the new acts with
   another version); (c) a v2 key fingerprint that covers the scheme, which would change the anchor
   of RFC-0012, which composes it. The session recommends (a): it changes nothing signed, and (c)
   touches an ACCEPTED RFC.
   **Taken (§799): (a)**, the rule of the registry. Discarded: (b), the v2 act, and (c), the v2
   fingerprint.
5. **When does a scheme leave the accepted set?** Only if it is broken, with the containment rule
   of D-E (5) (recommended); or also when its library dies, which would leave heads nobody has
   broken unverified.
   **Taken (§799): only if it is broken**, with the containment rule. Discarded: also when its
   library dies.
6. **Where does the registry live?** `spec/NUCLEO.md` §6, with a producer and its KAT
   (recommended, like the one of families of D-G), or a table in the code, without spec.
   **Taken (§799): `spec/NUCLEO.md` §6**, with a producer and its KAT. Discarded: the table in the
   code, without spec.
7. **Does it travel in the `zkssl/0.5` cut?** Yes, in the same seal as the cut, with E2 and E3
   before (recommended); or in a cut of its own afterwards, which would pay another break, another
   kit and another move of vectors; or it is not adopted, and entry 87 stays open with what the act
   gives.
   **Taken (§799): yes**, in the same seal as the cut, with E2 and E3 before. Discarded: a cut of
   its own afterwards, and not adopting it.

## What is discarded

- **The version byte as scheme** (v7 for XMSS, v8 for another): it puts two axes in one marker.
  The house already separated them once: `VERSION_FORMATO` says which fields enter the signature
  and `COFIRMA_VERSION` which format the co-signature has, and «they are two distinct axes and are
  not unified» (`lib.rs:294-300`).
- **Trying the accepted verifiers in order** until one gives VERDE: the result is decided by the
  weakest of the set, and a signature does not say which one accepted it.
- **Reading the scheme from the key**, as the second one does today: the OID is ambiguous inside
  RFC 8391 (fact 2) and is not signed. ⚠️ §862: since E3, `juez_cabezas.py` and `cable.py` take the
  scheme from the dispatch, the implicit one, and not from the key; the judge reads the OID only
  from a key already checked against the entry, for the index, and `cable.py`, without checking it,
  to anchor it and measure the signature; `verificador.py` still chooses by it.

## Security

- **The spending key never travels**: this RFC does not touch it. All its signatures are the
  operator's and the witnesses'.
- **Downgrading the scheme.** With D-E (1) and (2), whoever verifies chooses by what the signature
  declares and binds: a signature of B is not presented as one of A, because the recovered preamble
  would not match, and one of A over a v7 preamble that says B neither.
- **A broken scheme.** Its signatures can be fabricated from the break on. D-E (5) leaves standing
  only the heads of A that a signature of B contains, and a fabricated head is not in the MMR that
  B signed.
- **«If there is no transition signed by the previous key, changing scheme is indistinguishable
  from a key compromise»** (entry 87, which composes with entry 84). With A broken, its signature
  over the act of B no longer proves anything; what sustains the rotation is the `siguiente`
  commitment A signed when it was sound, the pre-rotation one of RFC-0015 (D-B). Without a genesis
  act there is no such commitment, and the rotation to another scheme stays where the D-I of
  RFC-0015 leaves every operator without a genesis. DERIVED from reading the code and RFC-0015; not
  measured.

## What this RFC does not decide

Which scheme B would be, nor with which bench it is chosen (entry 108 and the addendum of entry 87:
the three primitives measured on the real hardware, with the method of §263). The `xmss` dependency
and its pinning. The scheme of the medium, which is C2SP's and already carries its type (RFC-0013,
D-C). The encoding of the signature of a scheme B, which its registry entry gives. And the
compatibility policy that says when a scheme leaves the set: RFC-0021 writes it, which the author
accepted in the same §799. ⚠️ §799: until here it said «RFC-0021 proposes it».

## References

- `BACKLOG.md`, entries 84, 87 (with its addendum on ML-DSA and SLH-DSA) and 108.
- `AUDITORIA.md` §236, §240, §275, §291, §293, §297, §406, §643, §644, §682, §692, §699 and §780.
- RFC-0015 (`spec/rfc/0015-el-ciclo-de-vida-del-firmante.md` §«D-C — El acta» [the act], §«D-I — El
  residuo, declarado» [the residue, declared] and §«Lo que se DESCARTÓ al medir» [what was
  DISCARDED on measuring]); the D-G of RFC-0018 and its D-E; RFC-0013, D-C; RFC-0016.
- `spec/NUCLEO.md` §«3. La regla de extensión, ejercida» and §«6. Los bytes: lo que un KAT fija»
  [the bytes: what a KAT fixes].
- RFC 8391 (XMSS), FIPS 204 (ML-DSA) and FIPS 205 (SLH-DSA).
