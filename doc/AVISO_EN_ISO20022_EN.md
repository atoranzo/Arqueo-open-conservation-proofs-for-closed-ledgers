<!--
  AVISO_EN_ISO20022 — where the notice of the pending would travel inside an ISO 20022 message.
  Design note for entry 11 of the BACKLOG, ACCEPTED as analysis in §799 (⚠️ §799: until here it said
  «in DRAFT»), with its other face: B11.2 of CONFIANZA_RESIDUAL.md, the encrypted notice. Sister of
  CADUCIDAD_PENDIENTE.md and of ROTACION_DE_CUENTAS.md, to which it adds a case. It touches no code,
  nor the wire, nor a circuit: it measures the fields that could carry it and leaves the decisions
  to the author. ⚠️ §861: the six decisions of its §8 were taken by delegation from the author, and
  what §8 says from there on is what was decided; the analysis does not change.
-->

# AVISO_EN_ISO20022 — the notice of the pending, inside or outside the message

> **Español:** [`doc/AVISO_EN_ISO20022.md`](./AVISO_EN_ISO20022.md) is the original of this page, in
> Spanish; it prevails wherever the two differ. This translation is of commit `7766efb` (audit entry
> §917) and was produced by the project's assistant, a Claude Code session, in audit entry §918
> (`GENAI.md`): it is a generated artefact, not text written by the author. Line citations of the
> form `file:N` refer to the Spanish files.

**Status**: ACCEPTED (§799), as analysis: the author accepts it as it is and leaves its decisions
open, D-1 to D-6 of §8, without choosing any option. ⚠️ §799: until here it said «DRAFT». Design
note, without code and without an RFC number. Choosing the field, its encoding and its order
relative to the encryption of the notice is the author's (§8). It closes neither entry 11 nor B11.2.
⚠️ §861: **decisions taken by delegation from the author** («resolve them applying the principles»),
with the criterion of §609 and the precedent of §642; REVERSIBLE; they enter with its acceptance.
They are the six of §8, each one below its question, with the table of when it is reopened; the note
of `BACKLOG.md`, in entry 11. Until here this paragraph said that the six remained open, without
choosing any option, and that choosing was the author's: it was, until §861. It still closes neither
entry 11 nor B11.2.

**Provenance**: Claude wrote it in a Claude Code session in the cloud, on `8c02941` (§789), outside
step 4 of [`GENAI.md`](../GENAI.md); §791 of `AUDITORIA.md` carries it. Integrating is not
accepting: acceptance remained pending until §799, in which the author accepts it as analysis, with
its decisions open. ⚠️ §799: until here it said «acceptance remains pending». ⚠️ §792: its citations
of `BACKLOG.md` by line became citations by entry, rule v5 of `tools/verificar_citas.py`.

**Convention**: ✅ measured in this tree, with file:line · 📐 derived with arithmetic that can be
reproduced · 📖 read outside the tree, with its source and its date · ⚠️ nuance · ❓ not measured.

---

## 0. The problem, with its record

The two-phase path leaves the money in a pending, and the receiver only claims it if the notice
reaches them, `PendingNotice { position, salt, amount, x }` ✅
(`crates/zk-ssl/src/two_phase.rs:62-74`): the position of the commitment in the tree of pendings,
the random value the payer chose, the amount and, in the v2 commitment, the reversal envelope `X`.
Without it the commitment cannot be rebuilt, and losing it means not being able to claim even though
the money is there (`two_phase.rs:23-30`).

The ISO bridge returns it **separately** from the report: `settle_pacs008_two_phase` gives
`(Pacs002, Option<PendingNotice>)` ✅ (`crates/zk-ssl/src/iso.rs:425-432` and `:493`), and
`claim_pacs008` receives it as one more parameter (`iso.rs:531-538`). Two comments of the bridge say
why (`iso.rs:484-491` and `:528-530`), and `doc/historia/VISION.md:679-682` repeats it:

> ISO 20022 has no field in which to carry them, and putting them in the remittance information one
> would be forcing it.

`spec/RPC.md` §«Pago en dos fases» [two-phase payment] (⚠️ §861: until here it cited lines,
`:132-134`) says the same of the wire: the notice «travels out of band from the payer to the
receiver (ISO 20022 does not carry it; AUDITORIA §21)». ⚠️ §861: since §861 the line carries its
mark, which corrects both things: the citation is §39, and the standard has no field made for the
notice, but the extension one admits it (§3). Entry 11 left it as a record of absence (entry 11 of
`BACKLOG.md`), and §268 joined it, in the same entry, with B11.2. This note does what entry 11 left
for when there was a channel to analyse: **measuring which fields of the pacs.008 and of the
pacs.002 can carry the notice**, against the schema and not against memory.

⚠️ **Two stale items in passing, which this note does not correct.** (⚠️ §861: D-6 corrects them,
with a mark where they live: `spec/RPC.md` and the doc of `settle_pacs008_two_phase`.) The citation
in `spec/RPC.md` §«Pago en dos fases» (⚠️ §861: until here it cited lines, `:134`) does not support
what it cites: §21 (`AUDITORIA.md:1844`) is the one about the reason codes against the catalogue and
does not speak of the notice; the one that says it is §39, which calls the channel «side» and
«unspecified» (`AUDITORIA.md:3566-3568`). And the doc of `settle_pacs008_two_phase` says in «Lo que
falta» [what is missing] that the second phase «is not implemented» (`iso.rs:421-424`), when
`claim_pacs008` implements it (`iso.rs:531`) and `the_full_two_phase_iso_cycle` exercises it
(`iso.rs:975`).

## 1. What the tree knows of the standard, and what it does not

✅ **Of the schema, nothing.** The bridge «is not an XML parser» (`iso.rs:34-37`): `Pacs008` is six
fields —`msg_id`, `end_to_end_id`, the two IBANs, `amount_minor` and `currency`— (`iso.rs:61-75`),
and `Pacs002`, eight (`iso.rs:117-129`). No file of the tree names `SplmtryData`, `RmtInf` or
`Ustrd`: `git grep` of the three names on `8c02941` comes out empty; outside this note, only the
notes that link to it name them. What the code checks of the standard is the `Max35Text` of `MsgId`
and `EndToEndId` (`iso.rs:311-339`) and the status and reason codes (`iso.rs:77-107` and
`:131-223`).

📖 **What was read outside, and how.** The ISO 20022 site was not reachable from the session: the
proxy returned 403. On 14-10-2026, the models that `xsdata` generates from the published XSDs were
read, in the `pyiso20022` 1.6.2 package from PyPI (wheel `pyiso20022-1.6.2-py3-none-any.whl`, sha256
`98693c96b245541141224c4f1a52e913485bc4cf3d2376101d4de60f9b1c3a50`, downloaded without installing or
running it): `pyiso20022/pacs/pacs_008_001_08/pacs_008_001_08.py` and
`pyiso20022/pacs/pacs_002_001_10/pacs_002_001_10.py`; the lines cited below are from those two
files. It is **a derived copy**, not the XSD: the cardinality is read in the convention of `xsdata`
—a list without `max_occurs` is `maxOccurs="unbounded"`, a field with `default=None` is optional—,
and checking it against the original schema is a declared pending item (§9). `pacs.008.001.08` and
`pacs.002.001.10` were chosen because they go together in one same release of the catalogue; the
tree does not pin a version, and which version a pilot would speak is left unmeasured ❓.

❓ **What was not read**: the usage rules of each infrastructure (SEPA, CBPR+, TARGET2…), which
restrict what the schema allows —how many occurrences of `Ustrd`, which character set, whether they
admit `SplmtryData`—. Nothing of what «fits» below holds for them yet.

## 2. The candidate fields, measured

📐 **What the notice takes up.** The house wire writes it `{position: Q, salt: B32, amount: Q}`
(`crates/zk-ssl-wire/src/lib.rs:266-272`), with `Q` in hexadecimal `0x…` (`lib.rs:82-88`) and `B32`
as `0x` and 64 digits (`lib.rs:61-68` and `:105-111`). In compact JSON, a v1 notice takes up **109
to 139 characters** (position and amount from one hexadecimal digit to sixteen), and the v2 with
`x`, like the CLI's file (`crates/zk-ssl-cli/src/cobro.rs:34-44`), **182 to 212**. In canonical
bytes —8 of position, 32 of salt, 8 of amount and 32 of envelope— they are 48 and 80, which in
base64 give **64 and 108 characters**. It is reproduced with `json.dumps(…, separators=(",", ":"))`
and `base64.b64encode` over the extremes.

⚠️ **The wire's notice does not carry `x`.** `PendingNoticeDto` is three fields and sets it to
`None` when reading, «0.2 does not carry the envelope» (`zk-ssl-wire/src/lib.rs:266-272` and
`:284-294`), and the ISO bridge only produces v1 notices, with `x: None` (`two_phase.rs:931-936`).
The envelope does cross the wire, but loose and towards the node: `zkssl_pendingPath` asks for the
four pieces, `x` included (`spec/RPC.md:98`). As a notice from the payer to the receiver, a v2
inside an ISO message would be the first message that carries it whole.

| | field (path) | 📖 type and cardinality | v1 in the clear | v2 in the clear | goes towards | what its name says |
|---|---|---|---|---|---|---|
| A | pacs.008 `CdtTrfTxInf/SplmtryData` | `SupplementaryData1`, 0..n: `PlcAndNm` 0..1 `Max350Text` and `Envlp` 1..1 with any element, `##any` (`pacs_008_001_08.py:780-787`, `:1496-1513`, `:3515-3522`); also at the root (`:3548-3554`) | yes | yes | the creditor: it is the order | supplementary data, the extension mechanism of the schema |
| B | pacs.008 `CdtTrfTxInf/RmtInf/Ustrd` | `Max140Text`, 0..n, inside `RmtInf` 0..1 (`:3123-3141`, `:3507-3514`) | yes, in one occurrence | in JSON, two occurrences; in base64, one | the creditor | remittance information, unstructured |
| C | pacs.008 `CdtTrfTxInf/RmtInf/Strd/AddtlRmtInf` | `Max140Text`, 0..3 (`:3110-3118`) | yes | in JSON, two of the three; in base64, one | the creditor | additional remittance information, structured |
| D | pacs.008 `CdtTrfTxInf/InstrForCdtrAgt/InstrInf` | `Max140Text` in `InstructionForCreditorAgent1`, 0..n (`:1305-1322`, `:3458-3464`) | yes | in base64, yes | the creditor's agent | instruction for the creditor's agent |
| E | pacs.002 `TxInfAndSts/SplmtryData`, and the one at the root | `SupplementaryData1`, 0..n (`pacs_002_001_10.py:748-755`, `:1433-1450`, `:3524-3530`, `:3563-3569`) | yes | yes | **back** to whoever sent the order | supplementary data, in the status report |
| F | pacs.002 `TxInfAndSts/StsRsnInf/AddtlInf` | `Max105Text`, 0..n (`:2811-2836`, `:3448-3454`) | in JSON, two occurrences (109 > 105); in base64, one | in base64, two occurrences (108 > 105); in JSON, up to three (212 > 210) | back | additional information on the reason for the status |
| G | pacs.002 `TxInfAndSts/OrgnlTxRef/RmtInf` | the remittance of the original order, copied (`:3200`, `:3280-3287`) | like B | like B | back | the original remittance, returned |

The «goes towards» column is not from the schema: it is from the tree. The pacs.002 is the answer to
the pacs.008 that carries its `MsgId` and its `EndToEndId` (`iso.rs:8-19` and `:117-121`); it goes
back to whoever sent the order.

## 3. The tree's two sentences, against the measurement

1. **«ISO 20022 has no field in which to carry them»** (`iso.rs:487` and `:528-529`;
   `doc/historia/VISION.md:680-681`; `spec/RPC.md` §«Pago en dos fases»; ⚠️ §861: until here it
   cited lines, `:133`). 📖 Read against the derived model, **it is not exact**: both messages have
   `SplmtryData`, whose envelope admits an element of any namespace, in the transaction and at the
   root (rows A and E). What is true is narrower: there is no field **made for** a notice of a
   pending, and the extension one is worth what the infrastructure lets it be worth (❓, §1).
   Correcting the sentence in the code, in `spec/RPC.md` or in the history is the author's (D-6):
   this note edits neither the code nor the spec. ⚠️ §861: D-6 was taken by delegation: the sentence
   above is corrected, with a mark, in the code and in the spec, and in the live documents that
   repeated it; the history and the entries, not.
2. **«putting them in the remittance information one would be forcing it»**. 📐 **It is not a size
   problem**: the v1 fits in a single `Ustrd`, and in base64 the v2 does too (§2). It is a judgement
   about **what the field is** —its name says remittance, what the payment settles— and about **who
   reads it**. The judgement is not measured; it is left with its reason so that the author ratifies
   or withdraws it. ⚠️ §861: ratified by delegation, in D-1: the remittance is discarded for what
   the field means, not for its size.

## 4. Who produces the notice and who needs it

✅ **The producer depends on the path.** In the bridge, the notice is born **inside** the settlement:
the position is assigned by the layer when sending (`two_phase.rs:875`), the salt is brought by the
debtor's bank as a parameter (`iso.rs:418-419` and `:431`) and the amount comes from the message, so
the notice exists after processing the incoming pacs.008, not before (`iso.rs:449-493`). Over the
wire, on the other hand, the payer has it **before** sending: `zkssl_sendMaterials` gives the payer
the reserved position (`pending_position` in `zk-ssl-wire/src/lib.rs:299-310`; `spec/RPC.md` §«Pago
en dos fases»; ⚠️ §861: until here it cited lines, `:121`), and the salt and the amount are the
payer's own.

✅ **The consumer is the creditor's side.** `claim_pacs008` receives the receiver's key and state
(`iso.rs:531-538`), which in the bridge's model come from the creditor's bank (`iso.rs:523-526`),
just as the payer's key comes from a store separate from the debtor's bank (`iso.rs:42-45` and
`:410-416`).

📐 **Hence the direction.** The notice has to go towards the creditor, which is the direction of the
pacs.008, and the pacs.002 goes back. Therefore:

- in the bridge as it stands, the notice cannot go in the **incoming** pacs.008, because it does not
  exist yet, and the bridge does not **emit** any pacs.008: `Pacs008` is only read (`iso.rs:61-75`
  and `:425-432`). Carrying it inside requires the bridge to forward the order to the creditor's
  side with the notice in it, or the notice to be born earlier, as on the wire;
- putting it in the pacs.002 (rows E to G) is the smallest change to the shape of the bridge —the
  pair of `iso.rs:432` becomes a field—, but it leaves it **where it already was**: on the debtor's
  side, which knows it whole because it chose the salt (`two_phase.rs:32-36`) and which would have
  to get it to the creditor just as today.

## 5. What the notice exposes inside the message, and to whom

✅ **Knowing the notice does not allow claiming.** With the complete notice and without the
receiver's key, the claim is rejected: `a_third_party_cannot_claim_the_pending`
(`iso.rs:1029-1067`). It is the fix of §39 (`AUDITORIA.md:3600`): the claim binds the key to the
identity of the commitment (`two_phase.rs:1231`).

📐 **What it gives is the link.** In the pacs.008 the parties and the amount already travel in the
clear —`Dbtr`, `Cdtr` and `IntrBkSttlmAmt` are mandatory in the transaction (📖
`pacs_008_001_08.py:3164`, `:3388`, `:3436`)—, so the notice adds two things for whoever reads the
message (⚠️ §827: three in a v2: also `x`, the envelope without randomness from which, by
enumerating the identities, one gets which account paid and its Δ, D-I of RFC-0008; the Security
section of RFC-0022 says so):

- **the position**, which is the key that joins a send with its claim: `spec/RPC.md` §«Qué ve
  exactamente un agregador» [what exactly an aggregator sees] (⚠️ §861: until here it cited lines,
  `:344-347`) measures it for the aggregator that sees both halves, and with the notice in the
  message any agent that reads it has it;
- **the salt**, which together with the receiver's identity and the amount recomposes the commitment
  (`two_phase.rs:925`) and lets one see **when** it disappears from the tree: the residue that
  `two_phase.rs:32-40` declares for the payer becomes the whole chain of agents'.

❓ How much an agent that already sees the order learns from that is not measured: it is the metadata
leak of `CONFIANZA_RESIDUAL.md` §3.4, with more observers.

## 6. The relation with B11.2, the encrypted notice

B11.2 asks for the «encryption of the notice to the receiver's key (AEAD + ephemeral key, ~100 B of
overhead)», with the Sapling pattern (`doc/CONFIANZA_RESIDUAL.md:324-327` (⚠️ §895: it said
`:315-318`)); it declares it independent and implementable today (`:541-543` and `:562`), and §268
left it with this entry (`AUDITORIA.md:20214-20222`). Measured against today's tree, it has four
conditions that touch the channel:

1. ✅ **No account has a key to encrypt to.** `AccountRecord` keeps identity, balance, nonce,
   `view_id` and the salt of the leaf (`crates/zk-ssl/src/lib.rs:752-767`; ⚠️ §861: it said
   `:740-755`), and opening over the wire sends `{publicId, viewId, leafSalt}` (`spec/RPC.md`
   §«Apertura» [opening]; ⚠️ §861: until here it cited lines, `:111`): three digests and no public
   key. Encrypting «to the receiver's key» requires **a new key per account**, registered at
   opening, and with it one more case for rotation: the sixth of
   [`ROTACION_DE_CUENTAS.md`](./ROTACION_DE_CUENTAS_EN.md) §2.
2. ✅ **An ephemeral key over a curve does not enter the SDK.** The house encrypts with
   XChaCha20-Poly1305 and a 24-byte nonce (`crates/zk-ssl/src/crypto.rs:39-50`), and in the tree
   there is no key agreement. B11.2 speaks of an «ephemeral key» without saying over which group,
   and the tree does not say it of Sapling either (📖 not read here). What was measured is this:
   since §788, `tools/check_curvas.py` rejects the curve libraries —`x25519-dalek` and
   `curve25519-dalek` among them— in the closure of the node, the witness, the SDK and the kit
   (`tools/check_curvas.py:14-21` and `:44-50`), so a Diffie-Hellman over a curve would not pass
   that gate in the SDK. ⚠️ §835: it said `:10-14` and `:26-30`; the mark of §835 in that header
   moved two lines. And it said that a Diffie-Hellman over a curve does not pass the gate in the
   SDK. `ring`, which is already in the SDK through the TLS of `ureq`, brings X25519 and ECDH
   P-256/P-384, and is not in the pattern (entry 122 of `BACKLOG.md`): a direct dependency on `ring`
   in the SDK comes out VERDE, measured in §835 on a copy. And it is not even needed: `ureq`
   re-exports `rustls`, and the SDK can do an X25519 today with `ureq::rustls::crypto::ring`,
   without touching `Cargo.toml` or `Cargo.lock`, measured in §835 with a separate crate. No gate
   that looks at crate names closes it while `ureq` carries `tls`. The title of this point holds for
   the libraries of the pattern, not for `ring`. And «no key agreement» holds for the house's code:
   the TLS of `rustls` does one on every `https` connection. ⚠️ §838: it said `:12-16` and `:28-32`;
   the header grew with two marks, R4 and R5. And what the mark of §835 says of `ring` no longer
   holds: `ring`, `rustls`, `rustls-webpki` and `aws-lc-*` are in the pattern; R4 admits `ring` only
   under `rustls` and `ureq`, and only in the cli and the SDK; and R5 gives ROJO to `ureq::rustls`,
   `rustls::`, `ring::` or `webpki::` in the house's sources (`:26-31`). A direct dependency on
   `ring` in the SDK, or a `use ureq::rustls::crypto::ring` in the cli, come out ROJO, measured in
   §838 on copies of the tree, and the title of this point again holds for the house's code with the
   libraries of the pattern (OpenSSL, another TLS stack, is not in it). The key agreement of the TLS
   remains: it is the exception of §106 for transport (§838). A post-quantum one, a KEM, is not in
   the tree —the grep for «ML-KEM» and for «Kyber» comes out empty— and **its size is not measured
   here**: the ~100 B of B11.2 cannot be taken as good for it.
3. 📐 **Encrypted, it only fits whole in the extension.** Without counting the KEM, the house's AEAD
   adds 40 bytes, 24 of nonce and 16 of tag: 88 and 120 bytes for v1 and v2, which in base64 are 120
   and 160 characters. The v2 no longer fits in a field of 140 (rows B to D), and with the KEM's
   encapsulation neither of the two fits; only `SplmtryData` (rows A and E) carries it in one
   element. In `Ustrd`, which the schema lets repeat without a cap, it would have to be split into
   pieces, and how many occurrences each infrastructure admits is not measured (❓, §1).
4. ✅ **Encrypting does not hide the notice from the payer**, who chose the salt (`iso.rs:418-419`).
   The residue of `two_phase.rs:32-40` is closed, according to that same comment, the way Zcash
   closes it: by having the receiver derive the random value. That is a change of the commitment,
   not of the channel.

📐 **And the order matters.** If the notice travels first in the clear and later encrypted, the same
field changes content: whatever is chosen has to be able to state its version —`PlcAndNm`, or a tag
inside the envelope— or the encryption will be a second break of the format.

## 7. The options, unchosen

⚠️ §861: «unchosen» held until here: §8 chooses them, by delegation from the author (D-1).

| option | what changes in the code | v1 / v2 / encrypted | goes towards | depends on |
|---|---|---|---|---|
| **A.** `SplmtryData` of the pacs.008, with a namespace of its own | `Pacs008` gains an optional field (`iso.rs:61-75`); the bridge accepts the notice in the incoming order (the wire's path, §4) or emits the order towards the creditor, which today it does not emit; the schema of the element, with its version | yes / yes / yes | the creditor | ❓ that the infrastructure admits the extension |
| **B.** `RmtInf` (`Ustrd` or `Strd/AddtlRmtInf`), with a compact encoding | what A needs, plus the encoding: base64 of the canonical bytes and a prefix that tells it apart from a real remittance | yes / in base64 / no | the creditor | ❓ character set and occurrences of the usage rules; the judgement of §3.2 |
| **C.** `InstrForCdtrAgt/InstrInf` | like B | yes / in base64 / no | the creditor's agent | ❓ like B |
| **D.** `SplmtryData` of the pacs.002 | `Pacs002` gains the notice and the pair of `iso.rs:432` becomes a field | yes / yes / yes | **back to the debtor**: it does not solve the delivery | — |
| **E.** Outside the message, specified | nothing in `iso.rs`; a document of the channel: who sends it, by which route and in which format | whatever the channel chooses | whatever the channel chooses | ❓ the channel: `iso.rs:47` says there is no `camt.05x`, and a notice to the creditor by that side has not been looked at (⚠️ §861: it has been looked at now, below: the camt.054 carries `SplmtryData` in the transaction) |
| **A + B.** ⚠️ §861: the combination, added and discarded: A, with a compact `Ustrd` as a fallback | what A needs and what B needs at once | yes / in base64 / no | the creditor | discarded: it forces the remittance field and adds a second writing of the same notice; the fallback, if `SplmtryData` is pruned, is E with the same element |

What they have in common: in all five (⚠️ §861: and in the combination A + B, added to the table)
**the notice stops being only a Rust value**, and that changes what the comment of `iso.rs:490-492`
wanted —that the signature remind whoever uses the bridge that the delivery is their business—. From
A to D the message solves it; in E, a document.

⚠️ §861: **the jump to the holder.** Row A reaches the creditor's agent, not the holder. From the
creditor's bank to its customer, the same element fits in `Ntry/NtryDtls/TxDtls/SplmtryData` of the
camt.054: 📖 `EntryTransaction10` carries `SplmtryData`, a list
(`pyiso20022/camt/camt_054_001_08/camt_054_001_08.py:5997` and `:6201-6208`, from the same wheel,
with the same checksum), and it also carries `RmtInf` and an `AddtlTxInf` of 500 characters; read in
the derived model, not in the XSD. If not, it travels outside the message. On the payer's side, the
pain.001 carries the same `CdtTrfTxInf/SplmtryData` (📖
`pyiso20022/pain/pain_001_001_09/pain_001_001_09.py:3139` and `:3344`). The tree has no camt.05x
(`iso.rs:47`), and none of this is built.

## 8. Decisions (by delegation from the author, §861, REVERSIBLE)

⚠️ §861: until here this title said «Open decisions for the author». The six questions stay as they
were, and each one carries below it what was decided, by delegation from the author («resolve them
applying the principles»), with the criterion of §609 and the precedent of §642; REVERSIBLE; they
enter with its acceptance. One format and two carriers. The table of when it is reopened, at the
end.

- **D-1. The field**: A, B, C, D or E, or a combination (in the clear in one and encrypted in
  another).
  **Decided (§861)**: inside the message, A, `CdtTrfTxInf/SplmtryData` of the pacs.008; from the
  creditor's bank to the holder, the same element in `TxDtls/SplmtryData` of the camt.054 (§7);
  outside the message, the same element (E). It is born on the materials path, before the order
  (`zkssl_sendMaterials` gives the position; the salt and the amount are the payer's), and the
  payer's side writes it: the payer in their pain.001, and their bank carries it to the pacs.008.
  The operator writes nothing, and the bridge in the middle, the one that puts the key into the
  layer, is not the model and does not gain a pacs.008 emitter. Discarded: B, for what the field
  means (§3.2), and because encrypted it does not fit in 140 characters; C, because its semantics is
  an instruction to the agent, and encrypted it does not fit; D, because it goes back; and the
  combination of A with B, by §7.
- **D-2. The encoding**: the wire's JSON (`zk-ssl-wire/src/lib.rs:266-272`), the canonical bytes in
  base64, or an XML element of its own with its namespace.
  **Decided (§861)**: an XML element of its own, with a versioned namespace, in the `Envlp`. Its
  leaves use the wire's writing: canonical `Q` for the position and the amount, canonical `B32` for
  the salt and `x`; no base64 and no JSON inside the XML. Conformance is of the infoset —names and
  text of the leaves—, not of the bytes, because an agent can reserialise the XML: the KAT is an
  infoset plus a reference serialisation, and the conforming reader uses a real XML parser, outside
  the bridge, and rejects a leaf that is not canonical. The container is new —the third, next to
  `PendingNoticeDto` and `AvisoV2`—, and the RFC says so; the writing of the values is not.
- **D-3. v1 or v2**: whether the envelope `x` travels. The bridge produces v1
  (`two_phase.rs:931-936`), the wire's notice does not carry it (`zk-ssl-wire/src/lib.rs:266-272`
  and `:291`), `zkssl_pendingPath` does ask for it (`spec/RPC.md:98`) and the CLI's file already
  carries the v2 (`cobro.rs:34-44`).
  **Decided (§861)**: both. The presence of the `x` leaf decides between v1 and v2, like
  `PendingNotice.x`; the namespace only states the format —in the clear or, when it exists,
  encrypted—, and an unknown one is rejected. A single signal for each fact.
- **D-4. The order relative to B11.2**: in the clear now, with the version in the format, or wait
  for the encryption and for the per-account key it requires (§6.1), which is a design matter of
  rotation.
  **Decided (§861)**: the encryption is not waited for. In the clear **inside** the message is the
  payer's choice, not what the standard gives by default: the path outside the message, with the
  same element, grants no observation («Granting observation that is not needed is hard to defend
  afterwards», `SECURITY.md` §2.ter). The leak is stated: position and salt for the whole chain of
  agents (⚠️ §827: and in a v2 also `x`, from which one learns which account paid and its Δ, D-I of
  RFC-0008; whether a v2 can go in the clear inside the message is a question of RFC-0022 for the
  author), without the power to claim (§5). The version in the clear enters with its withdrawal
  written from the start: when the encrypted one exists, the one in the clear inside the message
  becomes deprecated, and its vectors move (R3 and R6 b of RFC-0021). The encryption needs one of
  two things: (i) an amendment of §106, which only the author can give —a lattice KEM adds a family
  of assumptions, and a Diffie-Hellman over a curve is prohibited by §106—, or (ii) the symmetric
  path, with the house's XChaCha20-Poly1305 and a key that the receiver hands over with their
  address, one per relationship, derived from `sk` with its own domain (D-9 of entry 52 of
  `BACKLOG.md`): it adds no family and no primitive, and does not need the per-account public key of
  §6.1. Choosing belongs to B11.2; the principles lean towards (ii). With a curve, no, even though
  `ring` is already in the SDK (entry 122 of `BACKLOG.md`): §106 prohibits it, not the gate. ⚠️
  §827: since §838 the gate sees it too: R5 of `tools/check_curvas.py` gives ROJO to a `ring::` in
  the house's sources, as the mark of §838 in §6 says; the sentence came from a decision written
  before §838.
- **D-5. Whether it is protocol.** `spec/rfc/PROCESO.md:3-5` calls protocol what crosses the wire
  —`spec/RPC.md`, `spec/openrpc.json` and the vectors of `spec/vectors/`—, and a field of an ISO
  message is not on that list. Whether a banking interoperability format enters by RFC, with
  vectors, or as documentation of the bridge, the process does not say.
  **Decided (§861)**: yes, by RFC, with the number that is free when it enters, PROPOSED. It
  declares its compatibility —additive, R2; the S1 wire does not move— and its effect on «the
  spending key never travels» (`spec/rfc/PROCESO.md`, rule 3): none, because the element carries no
  key and the path it specifies is the materials one. Its KAT goes in a family of its own,
  `spec/vectors/aviso/`, with its `MANIFIESTO.txt`, its line in `HUELLAS.sha256` and its reader in
  `tools/segunda/`, not in `nucleo/`, and it enters only after checking rows A and E and the
  camt.054 (⚠️ §827: row E of §2 is the pacs.002, which D-1 discards, and option E of §7 goes
  outside the message, with no XSD to check against; what is checked are the three messages of D-1,
  `CdtTrfTxInf/SplmtryData` of the pain.001 and of the pacs.008 and `TxDtls/SplmtryData` of the
  camt.054: a correction that D-5 of RFC-0022 takes by delegation, REVERSIBLE) against the original
  XSD and against the public usage rules of at least one market practice. Both are readings, not
  acts before third parties; today both give 403. The seal that brings it in also updates the
  `superficie` block of `spec/RPC.md` §«Versionado y compatibilidad» [versioning and compatibility]
  —the new `manifiesto`, `vectores.json` and `vectores.lineas`—, as §809 did.
- **D-6. The sentences of §3**: whether they are corrected with their note in `iso.rs:484-491` and
  `:528-530`, in `spec/RPC.md` §«Pago en dos fases» (with the citation of §21 noted in §0) and in
  the history, or stay as they are with this note alongside. ⚠️ §861: until here it cited lines,
  `:132-134`.
  **Decided (§861)**: they are corrected, with a mark, where they live. The new sentence: there is
  no field made for the notice; the extension one, `SplmtryData`, admits it (read in a derived
  model, without checking against the XSD); using the remittance one would be forcing it. The
  citation of §21, which deals with the reason codes, moves to §39. The seal of this decision does
  it in the comments of `iso.rs`, of `client.rs` and of the SDK —also in the example of its header,
  which only cites §21—, in `spec/RPC.md`, `ARQUITECTURA.md`, the two `PAPER` and
  `doc/DIAGNOSTICO_ESCALADO.md`. Left for another seal: the «§21» with which the RFC template and
  RFCs 0002 to 0007 name the «aviso fuera de banda» [out-of-band notice] debt, and entry 74 of
  `BACKLOG.md`, closed; the one in `doc/CADUCIDAD_PENDIENTE.md:35`; and the `println!` of
  `crates/zk-ssl-sdk/examples/e2e.rs:25`, which is code. `doc/historia/`, the deposited preprints
  and the entries of `AUDITORIA.md` are not touched.

**When it is reopened** (⚠️ §861):

| decision | it is reopened if |
|---|---|
| D-1 | the original XSD, or the public usage rules of a market practice, remove or restrict `CdtTrfTxInf/SplmtryData` of the pacs.008 or `TxDtls/SplmtryData` of the camt.054; then the choice is between E alone and the combination, with the rules measured |
| D-2 | a market practice prohibits foreign namespaces in the `Envlp`, or bounds it in size or in character set |
| D-4 | the encrypted version exists —and the one in the clear inside the message is deprecated, by its written withdrawal—, or a measurement of what an agent learns gives more than the declared link: then the one in the clear inside the message is withdrawn before the encryption |
| D-5 | the author prefers documentation of the bridge, outside the promise, or changes D-A of RFC-0021 |
| all | the author revokes any of them. Nothing of what is built is irreversible: a PROPOSED RFC, an optional field, two functions and one of the bridge, with their tests |

## 9. The next step, which needs compiling

**The prototype is not made here**: this session does not compile. What would be written, in
`iso.rs` and next to `the_full_two_phase_iso_cycle` (`iso.rs:975-1027`):

1. a test that settles a pacs.008, **writes the notice in the chosen field** —a string with the
   shape of `SplmtryData` or of `Ustrd`, according to D-1 and D-2—, **reads it back only from that
   string**, claims with what was read and reaches `ACSC`;
2. its negative: the same field with one byte of the salt changed does not claim, with the rejection
   the layer gives, measured and not assumed;
3. before pinning the format, the check of the fields of §2 against the published XSD, which this
   note read in a derived copy.

⚠️ §861: with the decisions of §8, it is E1 of the RFC of D-5, and it goes by the materials path:
the notice is written in the order and the creditor's side reads it only from there, without any
spending key reaching the layer. Its negative changes one byte of the salt while keeping the hex
canonical, so that what falls is the opening of the commitment and not the reader; the reader's
canonicity and the binding between the `ACSC` and the order are tested separately. The check of
point 3 is the KAT's gate (D-5), not the prototype's, which reads only the reference serialisation:
the bridge is still not an XML parser (§1).

## 10. What this note does not do

It touches no code, nor the wire, nor a circuit; it spends no RFC number; it does not choose a
field; it closes neither entry 11 nor B11.2; it does not measure the usage rules of any
infrastructure, does not validate against the original XSD and does not measure the size of any KEM.
⚠️ §861: «it does not choose a field» was true until here: the field was chosen by the delegation
(§8, D-1). The rest still stands: the note touches no code, nor the wire, nor a circuit, nor spends
an RFC number, and the RFC of §8, D-5, will spend one when it enters.
