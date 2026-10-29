# RFC-0022 — The notice in ISO 20022: an element of its own, inside the message or outside it, written by the payer's side

> **Español:** [`spec/rfc/0022-el-aviso-en-iso20022.md`](../0022-el-aviso-en-iso20022.md) is the
> original of this page, in Spanish; it prevails wherever the two differ. This translation is of
> commit `7766efb` (audit entry §917) and was produced by the project's assistant, a Claude Code
> session, in audit entry §918 (`GENAI.md`): it is a generated artefact, not text written by the
> author. Line citations of the form `file:N` refer to the Spanish files.

- **Status:** PROPOSED (§827) — the text and its three stages, written over what
  `doc/AVISO_EN_ISO20022.md` measured (§791, accepted as an analysis in §799) and over the six
  decisions that §861 took, and re-measured in the tree of the seal. §861 took the six by
  delegation of the author («resuelvelas aplicando los principios» [resolve them applying the
  principles]), with the criterion of §609 and the precedent of §642; REVERSIBLE; they enter with
  its acceptance. They are written in `doc/AVISO_EN_ISO20022.md` §8, with their reason, their
  discards and the table of when they are reopened, and in the note of §861 to entry 11 of
  `BACKLOG.md`, with the same names, D-1 to D-6. This RFC writes them as protocol, cites them and
  does not reason them again. It decides a single thing of protocol that they do not say: it
  corrects the gate of the KAT of D-5, which named row E, by delegation of the author, REVERSIBLE
  and with its reason (below, D-5). It leaves two questions for the author, open
  (§«Open questions for the author»). No stage built: the step to ACCEPTED demands building them
  (rule 4 of the PROCESO) and the author's acceptance, which is not delegated.
- **Author:** Ángel José Toranzo Portela
- **GenAI assistance:** Claude, in a Claude Code session in the cloud (§827): it wrote the text over
  the note of §791 and the decisions of §861, «fuera del paso 4» [outside step 4] of `GENAI.md` —
  see [`GENAI.md`](../../../GENAI.md)
- **Date:** 2026-10-21 (drafted on 2026-10-15, on `d4d6d45`, §814)
- **Protocol version affected:** `zkssl/0.4` — **it does not change the wire** (S1 of RFC-0021) nor
  anything the node signs. The element does not travel over the JSON-RPC: it goes inside an
  ISO 20022 message or outside it. E2 adds a family of vectors with new paths (S5, additive: R2 of
  RFC-0021).
- **AUDITORIA entry(ies):** §21 (the ISO codes against the catalogue, which do not speak of the
  notice), §39 (the side channel, «unspecified»), §106 (a single family of assumptions), §268
  (B11.2, with entry 11), §609 (the criterion), §642 (the precedent), §791 (the note, measured and
  without choosing), §799 (accepted as an analysis), §835 (`ring`, which `check_curvas` did not
  see), §838 (the exception of §106 for the TLS transport, and `check_curvas` with R4 and R5), §860
  (D-9 of entry 52, the key per relationship), §861 (the six decisions, the note of entry 11 and the
  sentences corrected where they live) and §827 (this RFC).
- **Backlog:** entry **11** (side channel of ISO 20022), which this RFC does not close: with the RFC
  PROPOSED, it remains open until E1 and, afterwards, only through E3, which is B11.2, as the note
  of §861 says; with entry **122**, closed in §838, and entry **52** (its D-9, the key of the
  encrypted notice).

## Status of the stages

| stage | what it delivers | breaks the wire?, and which surface | status |
|---|---|---|---|
| E1 — the prototype by materials | in `crates/zk-ssl/src/iso.rs`: `Pacs008` gains the notice, a function that writes it in the reference serialisation, another that reads it, and a claim that takes the notice only from the order; with one discriminating test per property (below, «E1 — The prototype, in detail») | no: none of S1 to S6. The Rust API stays outside the promise (RFC-0021, D-A, «Fuera de la promesa» [outside the promise]) | not built |
| E2 — the KAT | the `spec/vectors/aviso/` family, with its `MANIFIESTO.txt`, its line in `HUELLAS.sha256`, its reader in `tools/segunda/` and its mutant; and the `superficie` block of `spec/RPC.md` §«Versionado y compatibilidad» [versioning and compatibility], up to date, as §809 did | S5, additive (R2): new paths | not built; behind its gate (D-5) |
| E3 — the encrypted notice | B11.2: the namespace of the encrypted format, with its KAT; and the withdrawal of the cleartext version inside the message, which becomes deprecated | S5: the encrypted one enters additively; the cleartext vectors move (R6 b), they are not rewritten | not built; it waits for one of the two paths of D-4 |

## Motivation

Entry 11 says that the notice of the pending item —position, random value and amount— travels
outside the message, «without specifying how», and that this «blocks any pilot» (entry 11 of
`BACKLOG.md`). Without the notice the receiver does not claim even though the money is there
(`crates/zk-ssl/src/two_phase.rs:23-30`). The notice is
`PendingNotice { position, salt, amount, x }` (`two_phase.rs:62-74`), and the presence of `x`, the
reversal envelope, is the dispatch signal between v1 and v2 (`:68-73`).

The note of §791 measured, against the schema and not against memory, which fields of the messages
could carry it, and left six decisions to the author (`doc/AVISO_EN_ISO20022.md` §8). The author
delegated them, and §861 took them. This RFC writes them as protocol, because D-5 decides that it
is.

**What was measured, and where each thing comes from.**

1. **Today's bridge is the middle path.** `settle_pacs008_two_phase`
   (`crates/zk-ssl/src/iso.rs:425`) settles with `layer.send(sender_key, …)` (`:449`): the payer's
   key enters the layer. There the notice is born on settling the incoming order, after it, and the
   bridge returns it apart from the report. `Pacs008` is six fields (`iso.rs:61-75`) and it is built
   in a single place (`:642`); `SplmtryData` does not appear in the code: only in seven comment
   lines, those that §861 put in `iso.rs`, `client.rs` and the SDK (`git grep -c`).
2. **On the materials path the notice exists before the order.** `send_materials` takes the
   position with `allocate_pending` and returns it in `pending_position` before the payer proves
   (`crates/zk-ssl/src/client.rs:252-255`; `SendMaterialsDto.pending_position`,
   `crates/zk-ssl-wire/src/lib.rs:304`), and the salt and the amount are the payer's. It is the path
   on which no spending key reaches the layer
   (`a_whole_payment_without_giving_any_key_to_the_layer`, `client.rs:996`; `PRINCIPIOS.md` §4.2).
3. **The standard has an extension field in the three messages of the path.** Read in the models
   that `xsdata` generates from the XSDs, in the `pyiso20022` 1.6.2 wheel, without installing or
   running it: `CdtTrfTxInf/SplmtryData` in pacs.008.001.08 (`doc/AVISO_EN_ISO20022.md` §2, row A)
   and in pain.001.001.09, and `TxDtls/SplmtryData`, a list, in the `EntryTransaction10` of
   camt.054.001.08; the last two, with their lines in the wheel, in `doc/AVISO_EN_ISO20022.md` §7,
   «el salto al titular» [the hop to the holder], which §861 sealed. Its envelope, `Envlp`, admits
   an element of any namespace. It is a derived copy, not the XSD: the three websites of ISO 20022
   and of the market practices gave 403 to the session that decided entry 11, and §861 counts it as
   what it did not measure.
4. **There are already two containers of the notice, and neither serves here.** `PendingNoticeDto`
   does not carry `x` (`crates/zk-ssl-wire/src/lib.rs:268-272`); `AvisoV2`, the CLI's file, carries
   it as mandatory (`crates/zk-ssl-cli/src/cobro.rs:34-44`). Both are JSON. What does exist is the
   encoding of the values: canonical `Q` and `B32`, which `zk_ssl_hash::cantidad_canonica` and
   `hex_canonico` read (`crates/zk-ssl-hash/src/lib.rs:1150` and `:1135`), the practice of §640 and
   of §662.
5. **The tree's sentences said something else.** «ISO 20022 no tiene campo donde llevarlos»
   [ISO 20022 has no field in which to carry them] (`iso.rs:484-492` and `:528-530`), and the
   citation of §21 in `spec/RPC.md` §«Pago en dos fases» [two-phase payment], which is §39. §861
   corrected them with a mark where they live, by D-6, and what remains to be corrected is said by
   that decision (below, D-6).

## Design

Each decision says here what the protocol is. Its reason, its discards and its reopening condition
are in `doc/AVISO_EN_ISO20022.md` §8, under the same letter, and they are not copied.

### D-1 — The field: one format and two carriers

- **Inside the message**, the notice goes in `CdtTrfTxInf/SplmtryData` of the pacs.008 (row A of
  `doc/AVISO_EN_ISO20022.md` §2). **From the creditor's bank to the holder**, the same element goes
  in `Ntry/NtryDtls/TxDtls/SplmtryData` of the camt.054; if not, outside the message. **Outside the
  message**, the same element (option E of the note, §7, with the format now specified).
- **Where it is born.** On the materials path, before the order (fact 2). It is written by **the
  payer's side**: the payer puts it in its pain.001 (`CdtTrfTxInf/SplmtryData`) and its bank carries
  it to the pacs.008. **The operator of the layer writes nothing.**
- **The middle bridge** —`settle_pacs008_two_phase`, with the key inside the layer— is not the
  model, and it **does not gain** a pacs.008 emitter. The field of `Pacs008` models the order that
  arrives with a notice born before.
- Discarded: the remittance (`RmtInf`), `InstrForCdtrAgt`, the pacs.002 and the combination of A
  with a fallback `Ustrd`: `doc/AVISO_EN_ISO20022.md` §8, D-1, and §7.

### D-2 — The encoding: an XML element of its own, which conforms by infoset

- An element of its own, with a **versioned namespace**, inside the `Envlp`. Inside, one leaf per
  field of `PendingNotice`: the position and the amount in canonical `Q`, the salt and `x` in
  canonical `B32`, and `x` only in a v2. The encoding of each value is the wire's; **without base64
  and without JSON inside the XML**. Each limb of the salt and of `x` is less than `p` (RFC-0016).
- **Conformance is of the infoset** —the names and the text of the leaves—, not of the bytes. That
  is why the KAT of E2 is an infoset plus **a reference serialisation**.
- **The conforming reader** uses a real XML parser, outside the bridge, and rejects a leaf that is
  not canonical. The prototype of E1 reads only the reference serialisation and declares it:
  «No es un parser XML» [It is not an XML parser] (`iso.rs:34`) is still true, and neither `serde`
  nor an XML crate enters `zk-ssl`.
- **The container is new**: the third, next to `PendingNoticeDto` and `AvisoV2`. What is not new is
  the encoding of the values.
- The names of the element and of its leaves, and the URI of the namespace, are fixed by E2 with its
  KAT: until then there are no bytes to copy. E1 goes before and its test 4 needs some: it uses
  provisional names and a provisional URI, which are not protocol, are not published as vectors and
  are replaced by E2.
- **What the infoset does not fix yet**, and E2 fixes with its KAT, because two readers would
  resolve it differently: what the reader does with two notices in the same transaction
  —`SplmtryData` is 0..n— or with one in the `SplmtryData` of the root of the pacs.008, which also
  exists (row A of `doc/AVISO_EN_ISO20022.md` §2); with a duplicated or unknown leaf, with
  whitespace around the text of a leaf and with an empty `x` leaf; and what the reader rejects in
  the face of an unknown namespace (D-3): the notice or the message, or whether it ignores the
  extension of another market practice in the same `Envlp`.

### D-3 — v1 and v2, with a single signal

- Both. **The presence of the `x` leaf decides** between v1 and v2, like `PendingNotice.x`.
- **The namespace only says the format**: cleartext or, when it exists, encrypted. An element with
  an unknown namespace is rejected; what is rejected is fixed by E2 (D-2).

### D-4 — The order with respect to B11.2: encryption is not waited for

- **The cleartext notice inside the message is the payer's choice**, not what the standard brings
  by default. **The path outside the message, with the same element, grants no observation**
  (`SECURITY.md` §2.ter).
- **The leak is stated**: inside the message and in cleartext, the position and the salt, and in a
  v2 also `x`, reach the whole chain of agents, without the power to claim (Security, below). ⚠️ D-4
  of `doc/AVISO_EN_ISO20022.md` §8 says only «position and salt», and it carries its mark (§827):
  with `x`, whoever reads the message learns which account paid and its Δ, the leak for which
  RFC-0008 took `X` out of its envelope (its D-I). Whether a v2 can go in cleartext inside the
  message is question 2, the author's.
- **The cleartext version enters with its withdrawal written from the beginning.** When the
  encrypted one exists, the cleartext one inside the message becomes deprecated, and its vectors
  move by R6 b of RFC-0021.
- **Encryption needs one of two things**:
  - **(i) an amendment of §106**, which only the author can make, like that of §705 for ML-DSA-44:
    a lattice KEM adds a family of assumptions (§106.3), and a Diffie-Hellman over a curve is
    prohibited by §106.1. It is question 1, below;
  - **(ii) the symmetric path**: XChaCha20-Poly1305, the house AEAD
    (`crates/zk-ssl/src/crypto.rs:39-50`; `chacha20poly1305` is already in
    `crates/zk-ssl/Cargo.toml:39` and in `crates/zk-ssl-sdk/Cargo.toml:40`), with a key that the
    receiver hands over together with its address, **one per relationship**, derived from `sk` with
    its own domain: D-9 of entry 52, which §860 resolved (`doc/ROTACION_DE_CUENTAS.md` §6). It adds
    neither a family nor a primitive. A single key per account, handed to each payer, would let
    each one decrypt the notices of the others (DEDUCED).
- **Choosing between (i) and (ii) belongs to B11.2**; the principles —§106.3 and §3 of
  `PRINCIPIOS.md`, minimalist— lean towards (ii). (i) only exists if the author amends §106.
- **With a curve, no**, even though `ring` is already in the SDK through the TLS of `ureq`: §106
  prohibits it, and since §838 the gate sees it too, because R5 of `tools/check_curvas.py` gives
  ROJO to a `ring::`, or to `rustls` or `webpki` as a whole word, in the house sources. The
  exception of §106 that §838 gave to the TLS transport of the cli and of the SDK is for the
  transport, and it does not reach the notice.

### D-5 — It is protocol, and it enters through an RFC

- **Yes, through an RFC**: this one, with the number free on entering. `spec/rfc/PROCESO.md` calls
  protocol what crosses the wire, and a field of an ISO message is not in that list; it enters all
  the same, because a format that a second implementation cannot reproduce does not set a standard
  (§609).
- **The KAT goes in a family of its own**, `spec/vectors/aviso/`, with its `MANIFIESTO.txt`, its
  line in `HUELLAS.sha256` and its reader in `tools/segunda/`. **It does not go in `nucleo/`**: the
  core is the verifier's side, what a third party recomposes without the node (RFC-0005, D-A), and
  `nucleo_kat.rs` gives one KAT for each NUCLEO `fn` (`crates/zk-ssl-cli/src/nucleo_kat.rs:3-4`).
  The notice is not that.
- **The KAT enters behind a gate**: the check against the original XSD of the three messages of D-1
  —`CdtTrfTxInf/SplmtryData` of the pain.001 and of the pacs.008, and `TxDtls/SplmtryData` of the
  camt.054— and the public usage rules of at least one target market practice. Both are readings,
  not acts before third parties; contacting a bank stays out. Today both readings give 403 (fact
  3). ⚠️ **It is the only new protocol decision of this RFC**, a correction taken here by delegation
  of the author, with the same criterion, REVERSIBLE: D-5 of `doc/AVISO_EN_ISO20022.md` §8 says
  «rows A and E and the camt.054». The reason: row E of its §2 is the pacs.002, which D-1 discards,
  and option E of its §7 goes outside the message, with no XSD to check against; and the pain.001,
  where the payer writes the notice (D-1), was not in it. The gate checks the messages that carry
  the notice, and they are those three. What was sealed carries its mark (§827), and it is reopened
  with D-5 or if the author prefers the gate of §861.
- **The seal of E2 brings the `superficie` block up to date** in `spec/RPC.md` §«Versionado y
  compatibilidad» —the new manifest, `vectores.json` and `vectores.lineas`—, as D-5 of the note
  asks and as §809 did.

### D-6 — The sentences, corrected where they live

§861 fulfilled it: with a mark, in the comments of `iso.rs`, of `client.rs` and of the SDK, in
`spec/RPC.md`, `ARQUITECTURA.md`, the two `PAPER`s and `doc/DIAGNOSTICO_ESCALADO.md`. What remains
for another seal —the «§21» of the RFC template and of RFCs 0002 to 0007, that of entry 74, closed,
that of `doc/CADUCIDAD_PENDIENTE.md` and that of the `println!` of
`crates/zk-ssl-sdk/examples/e2e.rs`, which is code— is listed by D-6 of `doc/AVISO_EN_ISO20022.md`
§8. This RFC corrects none of them.

### When it is reopened

The table of `doc/AVISO_EN_ISO20022.md` §8, «Cuándo se reabre» [when it is reopened], which is that
of these decisions: D-1, if the XSD or the usage rules remove or restrict the carriers; D-2, if a
market practice vetoes foreign namespaces or bounds the `Envlp`; D-4, when the encrypted version
exists or if a measurement of what an agent learns gives more than the declared link; D-5, if the
author prefers documentation of the bridge or changes D-A of RFC-0021; and all of them, if the
author revokes any. This RFC adds to it only the condition of its correction of the KAT's gate
(D-5): it is reopened if the author prefers that of §861.

## Open questions for the author

1. **Is §106 amended for the encrypted notice?** It is path (i) of D-4: a lattice KEM, which adds a
   family of assumptions (§106.3), as the amendment of §705 added ML-DSA-44. Without it, E3 can only
   go by (ii), the symmetric one, which does not need it. The delegation does not take it: amending
   §106 belongs to the author, and §861 left it so («the amendment of §106 that a KEM for the
   encrypted notice would ask for»). **Open.**
2. **Can a v2 go in cleartext inside the message?** D-4 leaves it to the payer's choice, with the
   leak of the position and the salt; with `x`, whoever reads the message also learns which account
   paid and its Δ, and links the notices of the same issuer (Security). The branches: **yes**, with
   that leak declared, as D-4 says today; **no**: a v2 travels inside the message only encrypted,
   and in cleartext only outside it, with the precedent of D-I of RFC-0008 («privacy is a property
   of the protocol, not a policy of the issuer»); or **yes, when the envelope gains randomness**,
   which is a format break of the commitment (D-I leaves it as its reversal). The delegation does
   not take it: the second revises a decision that §861 took, and the third, the commitment of
   RFC-0003, ACCEPTED. **Open.**

The acceptance of this RFC, also the author's.

## E1 — The prototype, in detail

In `crates/zk-ssl/src/iso.rs`, next to `the_full_two_phase_iso_cycle` (`iso.rs:975`):

- **The field.** `Pacs008` gains `aviso: Option<String>`, the content of the `Envlp` of
  `CdtTrfTxInf/SplmtryData`. It is built in a single place (`iso.rs:642`).
- **The writer.** `aviso_a_envlp(&PendingNotice) -> String` gives the reference serialisation.
- **The reader.** `aviso_de_envlp(&str) -> Result<PendingNotice, …>` reads only that serialisation.
  The leaves go through `cantidad_canonica` and `hex_canonico`; each limb of the salt and of `x` has
  to be less than `p`; an unknown namespace is rejected.
- **The claim.** `claim_pacs008_materiales` takes the notice only from the order and applies it
  with `apply_claim`, with a receipt that the receiver proved locally. It gives an ACSC with the
  identifiers of that order, or an RJCT, with a code from the catalogue that the bridge already
  uses, if the amount of the notice is not that of the order. No spending key reaches the layer.
- **The tests**, one discriminating test per property (`PRINCIPIOS.md` §9). Tests 1, 2, 3 and 5 are
  the four properties that `doc/AVISO_EN_ISO20022.md` §9 names since §861 —the positive, the
  negative, the canonicity of the reader and the binding between the `ACSC` and the order—; test 4,
  which proves D-3, and the RJCT by amount of test 5, which proves the claim above, come from the
  decision of entry 11, and §861 did not copy them into §9:
  1. **the positive**: materials, local proof and `apply_send`; the order is written with the
     notice and the creditor's side reads it only from the order; claim by materials, ACSC with the
     order's `EndToEndId`, and the balances;
  2. **the negative**: one byte of the salt changed that leaves the hex canonical; the reader gives
     `Ok`, and the rejection is the layer's, at the step that is measured, not assumed;
  3. **canonicity**: upper case, a leading zero in `Q` and a limb `>= p` are rejected before
     calling the layer, and their canonical twin passes;
  4. **a single signal**: with the `x` leaf the notice is v2, without it v1, and an unknown
     namespace is rejected;
  5. **the binding**: with two orders, the ACSC names the order that brought the notice; a notice
     with another amount gives RJCT. It binds the `ACSC` to the order, not the pending item to the
     order (Security, «The limit of `claim_pacs008`»).
- **What it moves**: the pin of `zk-ssl` in the canon's table and the figures that repeat it. Size
  DEDUCED: from 100 to 150 lines of code and some 200 lines of tests.

## Compatibility

**The wire does not move** (S1), nor anything signed (S3), nor the kit's envelopes (S4), nor the
kit (S6). E1 is Rust API, outside the promise. E2 adds paths in `spec/vectors/` (S5, R2). E3 adds
the encrypted format with its namespace (R2) and deprecates the cleartext one inside the message:
withdrawing it is not an R1 break, because its reader is not in S1 to S4, and in S5 its vectors
move, they are not broken (RFC-0021, D-A and R3 of its D-B). This last point is DEDUCED, from
reading RFC-0021.

**The format is not a surface**, and it is said expressly: although D-5 calls it protocol, it is
none of S1 to S6 of RFC-0021, and what the promise covers of it are its vectors, in S5. A bank's
reader has no other compatibility promise than those vectors and the version of its namespace
(D-3). Making it a surface would require changing D-A of RFC-0021, which is one of the conditions
under which D-5 is reopened; this RFC does not do it.

## Security

- **The spending key never travels** (rule 3 of the PROCESO): **no effect**. The element carries no
  key, and the path this RFC specifies is that of materials. The middle path, which puts the
  payer's key into the layer, is not the model and gains no emitter.
- **Knowing the notice does not allow claiming**: with the complete notice and without the
  receiver's key, the claim is rejected (`a_third_party_cannot_claim_the_pending`, `iso.rs:1033`).
- **What the cleartext notice gives away: the link and, in a v2, who pays and its Δ.** The position
  joins a send with its claim; the salt, with the receiver's identity and the amount, recomposes the
  commitment and lets one see when it disappears from the tree (`doc/AVISO_EN_ISO20022.md` §5). And
  in a v2, `x`: `X =
  M(refund_id, [delta, 0, 0, 0])` is a commitment without randomness
  (`crates/zk-ssl/src/pending.rs:88-98`), the layer demands that `f` be the `public_id` of the
  account that pays (`two_phase.rs:710-714`; item (7) of entry 12) and the identities are
  enumerated without a credential (`zkssl_accountCount` and `zkssl_publicId`, `spec/RPC.md`
  §«Lectura» [reading]). Whoever reads the pacs.008 or the camt.054 tries `M(pid, d)` for the
  plausible `d` and learns which account paid and its Δ; and two notices of the same issuer with
  the same Δ carry the same `X`, linkable without enumerating anything. It is D-I of RFC-0008,
  which for that reason took `X` out of the portable envelope —«privacy is a property of the
  protocol, not a policy of the issuer»—, applied to the whole chain of agents (DEDUCED, from
  reading it; not measured here). Inside the message, all that passes from the payer to the whole
  chain of agents. How much more an agent that already sees the order learns is not measured.
  Outside the message there are no new observers. What `x` gives is more than the link that D-4
  declared: when it is measured, it is its reopening condition; whether a v2 can go in cleartext
  inside the message is question 2.
- **In cleartext, the notice has no integrity.** An agent can remove the `x` leaf —its presence
  decides (D-3)— and leave a v2 as a v1, or change a leaf: the claim comes out RJCT, the money does
  not move and, if nobody resends the notice outside, the pending item reverts to the issuer when it
  expires (DEDUCED). The path outside the message does not go through the agents; inside, it would
  only be given by the notice encrypted with the AEAD and a key that the agent does not have, path
  (ii) of D-4 (DEDUCED).
- **The payer's residue remains** (`two_phase.rs:32-40`): it chose the salt, and encrypting the
  notice does not hide it from it. Closing it is changing the commitment, not the channel.
- **The limit of `claim_pacs008`.** It copies the identifiers of `previous` without checking that
  the notice is of that payment (`iso.rs:539-542`): an ACSC can name the `EndToEndId` of another
  payment. It creates no balance nor spends without a key; it is a limit of the bridge, which its
  documentation declares since §861 (`iso.rs:530`). The new path bounds it, it does not close it:
  test 5 of E1 binds the `ACSC` to the order that brings the notice and to its amount, but nothing
  binds the pending item to the order —the commitment is `M(M(receiver_id, salt), amount)`, without
  the `EndToEndId` (`pending.rs:70`; `two_phase.rs:925`)— and the cleartext notice is not
  authenticated. If the payer or an agent puts the notice of pending item A in order B, with the
  same amount and the same creditor, the `ACSC` names B for A's money (DEDUCED). ⚠️ §861 wrote in
  `iso.rs:530` that the materials path «lo cerrará» [will close it], and it carries its mark
  (§827).
- **§106**: nothing in this RFC adds a family of assumptions. Encryption stays conditional on the
  amendment that only the author gives, or on the path that does not need it.

## What this RFC does not do

It touches neither code, nor the wire, nor a circuit: the prototype is E1, and its seal only marks,
on its same line, the comment of `iso.rs:530`. It does not choose between the two encryption paths,
which is B11.2, nor amend §106, nor decide whether a v2 goes in cleartext inside the message
(question 2). It does not decide the key of the encrypted notice: D-9 of entry 52 decided it
(`doc/ROTACION_DE_CUENTAS.md` §6), in §860. It does not check against the original XSD, does not
read the usage rules of any infrastructure and does not measure the size of any KEM. It does not
build the camt.054: the tree has no camt.05x (`iso.rs:47`). It does not touch `nucleo/` nor
`doc/historia/VISION.md`. It does not close entry 11, and it does not move to ✅ «Transición
ordenada desde infraestructuras tradicionales» [orderly transition from traditional
infrastructures] (`PRINCIPIOS.md` §3), which stays at ❌ until the prototype exists: this decision
gives it a path, it does not close it.

## References

- `BACKLOG.md`, entries 11, 52 and 122.
- `AUDITORIA.md` §21, §39, §106, §268, §609, §640, §642, §662, §705, §791, §799, §809, §835, §838,
  §860 and §861.
- `doc/AVISO_EN_ISO20022.md` §2, §5, §6, §7, §8 and §9; `doc/ROTACION_DE_CUENTAS.md` §6;
  `doc/CONFIANZA_RESIDUAL.md` (B11.2).
- `spec/rfc/PROCESO.md` §«Reglas» [rules]; RFC-0003, D2; RFC-0005, D-A; RFC-0008, D-I; RFC-0016;
  RFC-0021, D-A and D-B.
- `PRINCIPIOS.md` §3, §4.2 and §9; `SECURITY.md` §2.ter; `tools/check_curvas.py` (R4 and R5, §838);
  `spec/RPC.md` §«Lectura».
- ISO 20022: pain.001.001.09, pacs.008.001.08 and camt.054.001.08, read in the models derived from
  `pyiso20022` 1.6.2, not in the XSD.
