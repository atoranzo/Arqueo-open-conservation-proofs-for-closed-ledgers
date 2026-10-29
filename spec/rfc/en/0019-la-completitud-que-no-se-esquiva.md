# RFC-0019 — Completeness that cannot be dodged: the window with the attested index, and the `StaleState` that does not resolve on its own

> **Español:**
> [`spec/rfc/0019-la-completitud-que-no-se-esquiva.md`](../0019-la-completitud-que-no-se-esquiva.md)
> is the original of this page, in Spanish; it prevails wherever the two differ. This translation is
> of commit `7766efb` (audit entry §917) and was produced by the project's assistant, a Claude Code
> session, in audit entry §918 (`GENAI.md`): it is a generated artefact, not text written by the
> author. Line citations of the form `file:N` refer to the Spanish files.

- **Status:** PROPOSED (§675), with its four decisions taken (§679, §681), E1 and E2 built
  (§681, §682) and the first piece of E3, the core, built (§773).
  It amends RFC-0010 (ACCEPTED); the step to ACCEPTED demands building its stages (rule 4
  of the PROCESO). ⚠️ §826: it said «demands deciding them and building its stages»: the four
  decisions are taken (§679, §681). Its stages, with the surface each one breaks, in «Status of the
  stages», before «Security» (§826).
- **Author:** Ángel José Toranzo Portela
- **GenAI assistance:** Claude, in a Claude Code session in the cloud (§675): it wrote the draft
  over what the re-triage of the second swarm of the v2.0 plan measured, «fuera del paso 4»
  [outside step 4] of `GENAI.md` — see [`GENAI.md`](../../../GENAI.md)
- **Date:** 2026-10-02
- **Protocol version affected:** `zkssl/0.4`. E1 does not change the wire; it changes a verdict of
  the kit. E2 changes the verdict of two published vectors (⚠️ §826: it said «of one published
  vector»). E3, the receipt's leaf, does change the wire (its core enters in §773 without changing
  it).
- **AUDITORIA entry(ies):** §675 (this draft), §679 (the decisions), §681 (D-A and E1), §682
  (E2), §707 (what `digest_pi` binds, through D-G of RFC-0018), §726 (the current one, 0.4.4, with
  its marks in the paragraph of E1 and E2 and in the row «E3, the rest»), §727 (this list, with its
  mark), §773 (E3, first piece: the core), §815 (the strong version of `ProofFailed` and
  `VerificationFailed`, which E3 also carries), §826 (the table of stages and the header,
  corrected). ⚠️ §727: it left out §726, which touched this file with two marks; it and §727
  enter by their number, before §773. ⚠️ §826: it ended at §773, and it left out §815, which
  touched this file with its paragraph of E3.

## Motivation

The completeness envelope (RFC-0010) says, without the node, whether an operation that the node
received under its signature was resolved within its window. The re-triage of the second swarm
measured two ways for the kit to say something false, in opposite directions.

1. **The window is measured with the DECLARED index.** The kit decides «NO RESUELTA EN LA VENTANA»
   [NOT RESOLVED IN THE WINDOW] by comparing the receipt's era with the declared `index` of the
   current head. The XMSS signature attests its embedded index, and the only rule between the two
   is `embebido < declarado` (§332): the declared one has no upper bound. A head signed with the
   `index` inflated turns an era still open into a portable ROJO against the operator (measured:
   index 1444 in a head signed with 1).
2. **A `StaleState` resolves without being bound to the operation.** A resolution by rejection is
   bound to the receipt by the `hashPrueba` of the rejection's `data`, which is not signed (the kit
   calls it «la palabra del nodo» [the node's word]), and the cause `StaleState` is sustained with a
   `recibo` that is not signed either. The accused operator can answer with an invented
   `StaleState` and the kit says VERDE. RFC-0010 declares it in part (E4, the node's word); the
   verdict is still VERDE.

## Design

**E1 — The window with the attested index.** In the window of the completeness envelope, in its
resolutions and in the pledge, the kit uses `embebido + 1`, the index the signature attests, instead
of the declared one. A single head reader returns `(declarado, embebido, acreditado)`. It changes
neither the wire nor any signed byte.

**E1, built (§681).** `indice_embebido` and `indice_acreditado` in the kit's command; the window
starts at the embedded index of the closing head and `S` is the attested index of the current head
or of the head of the resolution (acknowledgement, rejection, batch and pledge). The 74 published
vectors of the completeness envelope say the same; a new one, `neg-vigente-index-inflado.json`, is
probe H10: the current head with the `index` inflated to 1444 gave NO RESUELTA and gives ventana
ABIERTA [window OPEN].

**E2 — `StaleState` does not resolve on its own.** A resolution by rejection whose cause is
`StaleState` comes out as the fourth state of RFC-0010 D-G, «DECLARADA, NO PROBADA» [DECLARED, NOT
PROVEN] (exit 3), and not as VERDE: the cause is sustained over the committed state, but its binding
to THIS operation is the node's word.

**E2, built (§682).** `stale_no_resuelve`, asked in the arm of the direct rejection and in that of
the batch, after their own checks: a malformed envelope is still ROJO on its own account. Exactly
the two vectors that this RFC named change; they move whole to `spec/vectors/0.4/completitud/` with
their manifest of the time, which the published kit `0.4.2` satisfies, and the same bytes live in
`completitud/` as `rechazo-stale-declarada.json` and `lote-rechazado-stale-declarada.json`, with the
fourth state.

**E3 — The receipt's leaf carries the digest of the public inputs.** The reception receipt gains
`digest_pi`, the digest of the public inputs of the received proof, under the signature. A
`StaleState` rejection is then bound by the signature and not by the `data`, and E2 can give VERDE
again when the binding is verifiable. The receipt's leaf changes, so it goes to the `zkssl/0.5`
train (RFC-0018). (§707: `digest_pi` also binds the family and the AIR version of the received
proof, so that a receipt says which proof it is of; D-G of RFC-0018 brings it, in proposal status,
and this stage fixes the exact form.)

**E3, first piece (§773): the core.** `digest_pi(familia, version_air, entradas)` is
Blake3(`ZK-SSL-public-inputs-v1` ‖ length ‖ family u16 LE ‖ AIR version u16 LE ‖ the elements of
the public inputs, in the order of `to_elements()` and eight bytes LE each), reduced limb by limb,
the mould of the batch; and `recibo_digest_v2(hash_prueba, digest_pi, era, n)` is the v1 leaf with
`digest_pi` behind `hash_prueba`, under its own domain, `RECEP_V2`. Both have KATs, and the second
implementation reproduces them (`spec/NUCLEO.md`, section 6). Nothing consumes them yet: the node
recording the v2, the wire, the kit in 2.11 and the second in its verifier go with the cut, and the
family and version numbers are given by the registry of D-G of RFC-0018. Changing this form, once
published, is another version of the leaf, not a rewrite of it.

⚠️ **§815: E3 also carries the strong version of `ProofFailed` and `VerificationFailed`**, by
decision of the author. Since kit 0.4.3 both are declared —the fourth state, exit 3—, as D-I of
RFC-0007 asks, and they are still not proven (`SECURITY.md` §3.15). With `hash_prueba` and
`digest_pi` under the signature, whoever has the proof presents it, the kit re-verifies it with
those inputs and the rejection is proven or refuted: the objection of D-I —the command would verify
some bytes the node chose— stops holding, because the leaf the node signed binds them. Until the
cut, they are declared.

## Compatibility

E1 does not change the wire. It changes the kit's verdict only when the declared indices depart from
the attested ones. ⚠️ The sentence of this draft said «which an honest node does not sign»: it is
not true, an honest node with orphan indices signs them (§332), and the declared index of any head
can be changed by whoever assembles the envelope, because it is not signed. Measured (§681): none of
the 74 published vectors changes verdict.

E2 changes the verdict of published vectors: `completitud/resuelta-por-rechazo.json` and
`completitud/lote-rechazado-con-prueba.json` are resolved by `StaleState` and their manifests demand
VERDE. The vectors are not rewritten: the manifests become per kit version, and those of `0.4` are
kept with their verdict of the time.

E3 breaks the wire: `zkssl/0.5`.

## Decisions (the author's, §679)

The author decided them in the session of §676 to §678, «as recommended» by the session; D-A, when
the session told him that what was recommended did not hold (§681).

- **D-A — the attested index, and D-D of RFC-0010 amended (§681).** The session first recommended
  demanding `declarado == embebido + 1` and rejecting the head that does not satisfy it. It did not
  hold: §332 and §399 discarded it in writing (`crates/zk-ssl-verify/src/lib.rs`,
  `verificar_cabeza` and `verificar_cofirma`): `GuardianIndice::reservar` persists the index BEFORE
  signing, so a process that dies between the reservation and the signature leaves an orphan index,
  and demanding the +1 would give ROJO on legitimate heads. The author chose the other branch:
  measure with the attested index and amend D-D. On building it, the session saw that measuring with
  the attested index only the head that measures left another hole: the `era` of the leaf is counted
  by the node with its counter (`anotar`: the declared index plus one), and in that unit an index
  burned without signing —free for the operator— would delay all the windows. What was built
  measures both ends in the unit of the signature: the start is the embedded index of the closing
  head and `S` the attested index of the head that measures. The amendment is in D-D of RFC-0010.
- **D-B — the fourth state.** E2 comes out as «DECLARADA, NO PROBADA» (exit 3), not as VERDE with a
  warning, which would end up being read as VERDE.
- **D-C — new vectors, with a new name.** The two published vectors keep their manifest and their
  verdict of `0.4`; those of E2 are born with another name. No vector is rewritten.
- **D-D — E3 within the train of RFC-0018** (its D-D).

## Status of the stages

⚠️ §826: the RFC did not have this table, and R5 of RFC-0021 asks for it one seal before the cut,
saying which surface of its D-A each stage breaks: S1, the wire; S2, the proofs that travel; S3,
what is signed; S4, the kit's envelopes; S5, the vectors; S6, the kit as a binary. It goes here, and
not after the header, so as not to move the lines of this RFC that other documents cite.

| stage | what it delivers | breaks the wire?, and which surface | status |
|---|---|---|---|
| E1 — the window with the attested index | `indice_embebido` and `indice_acreditado` in the kit's command, and a new vector, `completitud/neg-vigente-index-inflado.json` | it does not break the wire; in S4 it changes the verdict of a completeness envelope whose declared indices depart from the attested ones —probe H10 gave NO RESUELTA and gives ventana ABIERTA—, and that of none of the 74 published vectors of the family (§681); in S6 it does not break, because it changes the text and not the cause: of the vectors that travel in the tarball of 0.4.2, four give with 0.4.3 the same 1 with another text, because the window is named with the index the signature attests and its start (measured in §815, `doc/KIT.md` §0), and the line of their manifest, which pins the cause (`spec/PAQUETE.md` §6.1), is the same at `a4e888d` as in the tree | built — §681 |
| E2 — `StaleState` does not resolve on its own | `stale_no_resuelve`, in the arm of the direct rejection and in that of the batch: the fourth state of RFC-0010, exit 3 | it does not break the wire; in S4, two published envelopes go from VERDE to «DECLARADA, NO PROBADA»; in S5 it does not break: their two vectors moved with their bytes to `spec/vectors/0.4/completitud/` (R6 b), and the same bytes live in `completitud/` under another name | built — §682 |
| E3, first piece — the core | `digest_pi` and `recibo_digest_v2`, under the domain `RECEP_V2`, with their KATs and their replica in `tools/segunda`; nothing consumes them | no: two functions and their KATs, which nobody issues yet (R2 of RFC-0021) | built — §773 |
| E3, the rest — the v2 leaf, in use | the node records the v2 leaf, the wire serves it, the kit reads it in form 2.11 and the second in its verifier, with the family and version numbers of the registry of D-G of RFC-0018; and, by decision of the author in §815, the strong version of `ProofFailed` and `VerificationFailed`: the kit re-verifies the proof presented to it with `hash_prueba` and `digest_pi` under the signature. For that it has to verify proofs of the layer, like those of send and claim, to which the node gives a receipt (`SECURITY.md` §3.15), and today it does not verify send, claim, burn or threshold proofs (`spec/rfc/0018-el-tren-0-5.md:110`): the «Compatibilidad» [Compatibility] of RFC-0017 takes for granted that the kit does not judge those of the layer, and that changes with this piece, in the same cut | **yes**: S3, the receipt's leaf, which composes under the `recepRoot` that the head signs; S1, the receipt on the wire; and S4, form 2.11 of the envelope, and the rejection by `ProofFailed` or `VerificationFailed`, which since 0.4.3, the current one (§818), is declared —the `declarada` one comes out 3, and its rejection envelope, ROJO (`doc/KIT.md` §0)— and comes to be proven or refuted ⚠️ §726: 0.4.3 was the current one until §726; the current one is 0.4.4, which also declares the two causes: its three vectors of `completitud/` give 3 with the published binary | not built; in the seal of the `zkssl/0.5` cut (D-D) ⚠️ §913: the author decided on 29-10-2026 that the `zkssl/0.5` cut carries only D-H of RFC-0018; this piece moves to the next break, `zkssl/0.6` (RFC-0018, the note of §913 in D-H) |

E1 and E2 change verdicts of the kit (S4) and were built before the policy of RFC-0021 governed.
`arqueo-verify-v0.4.2` does not carry them: it was built from `a4e888d`, §670, earlier than both,
and §671 registered it. 0.4.3 carries them, which §815 cut as a security fix within `0.4.x` (R6 f),
with the list of what changes, E1 and E2 among it, in section 0 of `doc/KIT.md`. The author
published it on 21-10-2026 with its advisory, `GHSA-6p4m-qg2w-8r6j`, produced on `5fd5212`, and
since §818, which registers it with its fingerprint, it is the current one (`doc/KIT.md` §0).
⚠️ §726: it was the current one until §726, which registers 0.4.4, the current one since then;
0.4.4 also carries E1 and E2. ⚠️ §917: and that one was until §917, which registers 0.5.0
(`zkssl/0.5`), with E1 and E2 inside and the rest of E3 in `zkssl/0.6` (§913).
Their commits, `41f765d` (§681) and `b0b6770` (§682), come out in
`git log a4e888d..5fd5212 -- crates/zk-ssl-verify/src`, from 0.4.2 to the commit of 0.4.3.

## Security

The spending key does not travel and this RFC does not touch it. E1 removes a ROJO that could be
fabricated against the operator; E2 removes a VERDE that could be fabricated by the operator; E3
closes the residue that E2 names.

## References

RFC-0010 (D-D and its correction of §567, D-F, D-G, E4); `AUDITORIA.md` §332, §567, §571, §675;
`crates/zk-ssl-verify/src/main.rs` (`verificar_completitud`, `resolver_por_rechazo`,
`exige_mismo_recibo`).
