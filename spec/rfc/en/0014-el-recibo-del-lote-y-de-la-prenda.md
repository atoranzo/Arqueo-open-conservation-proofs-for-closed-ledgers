# RFC-0014 — The receipt of the batch and of the pledge: the aggregated, tied like the direct

> **Español:**
> [`spec/rfc/0014-el-recibo-del-lote-y-de-la-prenda.md`](../0014-el-recibo-del-lote-y-de-la-prenda.md)
> is the original of this page, in Spanish; it prevails wherever the two differ. This translation is
> of commit `7766efb` (audit entry §917) and was produced by the project's assistant, a Claude Code
> session, in audit entry §918 (`GENAI.md`): it is a generated artefact, not text written by the
> author. Line citations of the form `file:N` refer to the Spanish files.

- **Status:** ACCEPTED (§618) — **rule 4 of the PROCESO, settled with measurement.** (1) The spec:
  `PAQUETE.md` 2.11 gains its forms 5 and 6, the resolution of the batch (§612) and that of the
  pledge (§613); `RPC.md` says that the batch and the evaluated pledge carry `recepcion`, that the
  batch error names its `operacion` (§611) and that the `data` of the pledge rejection carries its
  receipt (§613); and in `NUCLEO.md`, the REGISTRY with `DOMINIO_LOTE` and `hash_del_lote` (§610).
  (2) The OpenRPC does not move: no new method, and the fields travel in results and errors whose
  normative form is `RPC.md`, to which the OpenRPC refers. (3) The vectors, new under `zkssl/0.4`:
  the KAT `nucleo/hash_del_lote.json` (§610) and thirty-seven envelopes in `completitud/` (§614).
  (4) The suites: the author's `--sello` canon, VERDE at every seal from S610 to S617 -with
  «completitud 73/73» from the tree and from inside the tarball since S614-, and its `--bancos`
  VERDE, 21 of 21, on `8b8e3f4`, with `tools/banco_recibo_agregado.sh` among them. Proposed in §608,
  with its decisions TAKEN in §609 -delegated by the author with his written criterion, and
  REVERSIBLE, see «Decisions»-; the stages, built from §610 to §614. It is accepted WITH its
  residue, D-H, intact.
- **Author:** Ángel José Toranzo Portela
- **GenAI assistance:** Claude (session 193, §608 to §614 and the turn of §618) — see
  [`GENAI.md`](../../../GENAI.md)
- **Date:** 2026-09-30
- **Protocol version affected:** `zkssl/0.4` — **does not go up**: everything is additive, like the
  receipt of §571 -two responses and one error gain `recepcion`, the batch error gains `operacion`,
  and the completeness envelope gains two resolutions-. The head does not move.
- **AUDITORIA entry(ies):** §608 (the proposal); §609 (the decisions); §610 (E2); §611 (E3); §612
  (E4a); §613 (E4b); §614 (E5); §618 (the acceptance).
- **Backlog:** the second part of D-H of RFC-0010 -«a stage to be designed, not an oversight»-; and
  `SECURITY.md` 2.ter, the deployment with aggregators.

## Motivation

RFC-0010 ties the operator with a receipt for every operation it EVALUATES through the holder's two
direct ways. In §576 it left out, by decision and with its measured reason, what comes in through
`zkssl_applyMany` and through `zkssl_pledge`: a receipt of theirs would have no resolution today,
and the completeness envelope would accuse an honest operator. It is declared residue -the second
part of its D-H- and a witness of the node ties it. And in the deployment with aggregators
(`SECURITY.md` 2.ter) that is EVERYTHING aggregated: whoever pays through an aggregator has today no
object to oppose.

What was MEASURED in §608, reading the code of the node and of the layer:

| way | what it evaluates | what it returns on accepting | what it returns on rejecting | resolution today |
|---|---|---|---|---|
| `zkssl_applyMany` | validates ALL the operations against the same snapshot and applies only if all pass: the first that fails aborts the whole batch (`apply_many`, `two_phase.rs`) | per entry, `logSeq`, `kind`, `accountsRoot` and `chain`; no `acuse`, but `zkssl_ackPath` serves each entry's by its `logSeq` | an error of the layer with `{causa, campos, seq}` that **does not say which operation failed**; and two causes belong to the batch as such, `DuplicateAccountInBatch` and `DuplicatePendingInBatch` | applied, **yes**, operation by operation; rejected, **no**: neither for the companions nor for the one that failed, because it is not named |
| `zkssl_pledge` | verifies the pledge proof against the `pendingRoot` of the LAST signed head -its `seq` has to match- and records the mark with `apply_consumo` | `accepted: true`, `yaEstaba`, `logSeq`, `s` | FIVE soft responses with `accepted: false`: no heartbeat, unsigned head, old `seq`, proof that does not verify and `apply_consumo` error | accepted: the mark under `consRoot` (`zkssl_consumoPath`) and the pledge envelope (`PAQUETE.md` 2.10), which re-verifies the proof -both exist, but nothing ties them to a receipt-; rejected: no envelope |

## Design

**D-A. The batch receives ONE receipt, not one per operation.** The node evaluates it as a unit -one
snapshot, one verdict, all or nothing-, and the rule of D-E of RFC-0010 counts what the node got to
evaluate. The leaf is the usual one, `recibo_digest(hashPrueba, era, n)`, with
`hashPrueba := hash_del_lote`, and `hash_del_lote` is the fingerprint, with its own domain
(`DOMINIO_LOTE`) and encoded length (§116), of the COMPOSITION: `k` and, for each operation in its
order, `(hashPrueba_i, cuenta_i, posicion_i)`. The composition is known to whoever assembles the
batch; the node does not return it, and it is signed because its fingerprint is under the
`recepRoot`. Nothing new under the head's signature.

**D-B. The resolution of the batch**, in the completeness envelope, `resolucion.tipo = "lote"`,
which carries the composition and the command recomposes `hash_del_lote` before anything else:
- **applied**: one acknowledgement for each `hashPrueba` of the composition, each under the
  `acusesRoot` of a head within the window, like verdict 1;
- **rejected with proof**: the RFC-0007 rejection envelope of the operation `j` the node names -the
  batch error gains `operacion: j`, additive-, tied to the batch receipt by its `data.recepcion` and
  its `data.operacion`. That tie is the node's WORD, as in D3 of RFC-0010: the hash of a proof does
  not let its public inputs be seen, and the command cannot recompose it. The companions are
  resolved by it: the batch is the unit;
- **rejected by its form**: `DuplicateAccountInBatch` and `DuplicatePendingInBatch` are proven with
  the composition alone -two entries with the same account, or with the same position-. And if the
  composition does NOT sustain the cause the node gave, the command names it, «RECHAZO SIN
  FUNDAMENTO» [rejection without grounds] (§612): it is the rule of decision 3 -where the verifier
  can repeat the operator's judgement, it says so- in the other place where it can;
- **the rest**, the causes RFC-0007 left without a portable proof: the fourth state, «declarada, no
  probada» [declared, not proven], as in D-G of RFC-0010.

**D-C. The holder inside an aggregator.** The receipt is received by whoever calls: the aggregator.
Each holder in the batch needs the response and the composition to assemble his envelope; the
aggregator forwards them to him, and the holder checks that his `hashPrueba` is in it. The operator
cannot deny having received that batch: its fingerprint is under his signature. The aggregator that
does not forward is left out: the holder chooses it, and the direct way stays open.

**D-D. The pledge receives a receipt when its proof is EVALUATED**, with `hashPrueba` the digest of
the proof that arrived, the same the house uses for the others (§116). It is not consumed by the
three responses that do not get to evaluate it -no heartbeat, unsigned head, old `seq`-: they are
like the wire noise of D-E of RFC-0010, and the holder proves again under the new head.

**D-E. The resolution of the pledge**, `resolucion.tipo = "prenda"`, which carries the pledge
envelope (2.10) against the head of the `seq` it was tied to, and the command requires that its
proof have the receipt's `hashPrueba`:
- **accepted**: the envelope VERIFIES and the `marca` of its statement is under the `consRoot` of a
  head within the window -the path of `zkssl_consumoPath`-. It is the WHOLE pledge, the one 2.10
  calls «the PAIR»;
- **rejected with proof**: the envelope does NOT verify against that head, and anyone checks it
  without the node. «That head», made precise in §613, is the one the node JUDGED: the last one
  signed on receiving, of index `era − 1` (§567), which the completeness envelope carries as
  `juzgada` -the node composes the statement with the root of its head, not with the envelope's-;
- ⚠️ **rejected without grounds**: the envelope VERIFIES and the node said no. The command names it
  -NAMED ROJO, «RECHAZO SIN FUNDAMENTO»-, which is stronger evidence than non-resolution: there is
  no need to wait for the window;
- **`apply_consumo` error** other than `ConsumoRepetido`: its cause in `data`, as in the batch;
  since §613 that `data` carries its `recepcion`, and it is resolved with its rejection envelope
  (2.6).

**D-F. The witness of §576 falls ON PURPOSE**, in the node's stage, as its own text announces: «if
one of the two starts reserving, this witness falls and D-E is decided again, with its resolution».
It is rewritten the other way round: the batch and the pledge consume a receipt when they evaluate,
and the pledge's three prior responses do not.

**D-G. What is added, all additive under `zkssl/0.4`:** `recepcion` in the response and in the error
of the batch, and in the pledge's response when it evaluates; `operacion` in the batch error; in the
REGISTRY of `NUCLEO.md`, `DOMINIO_LOTE` and `hash_del_lote`, with its KAT; and two resolutions in
the completeness envelope (2.11), with their vectors and their bench.

**D-H. The residue, declared.** The aggregator that does not forward; the pledge with an old `seq`,
which leaves no receipt and the holder retries -and a FALSE «old seq» is visible conduct, not
portable evidence-; the causes without a portable proof, in the fourth state; and the usual one, the
operator that does not answer (D-H of RFC-0010), which here too leaves no trace.

## Decisions

The three this text leaves to the author, with the one it recommends first:

1. **One receipt per batch (D-A)**, or one per operation with the batch fingerprint inside each
   leaf. Per operation would give each holder his own `rx`, but it changes the receipt leaf -a third
   field under the signature, a new `recibo_digest` and its vectors- to gain what the composition
   already gives.
2. **Which pledge responses consume a receipt (D-D)**: those that evaluate the proof, or all.
   Counting the prior ones would open gaps in the log for whoever sends pledges against an old head.
3. **The pledge rejection whose envelope verifies (D-E)**: a verdict with a name, «RECHAZO SIN
   FUNDAMENTO», or letting it fall into «not resolved in the window». The name costs nothing the
   verification does not already do, and turns twenty-four hours of waiting into immediate evidence.

**TAKEN in §609**, by delegation of the author and with the criterion he dictated: the project's
principles and its front page -a base layer «neutral, minimalist and resilient» (`PRINCIPIOS.md`
§3), the evidence that «an independent verifier checks with the node off» and the «demonstrable
accountability, in the manner of Certificate Transparency» (`README.md`)-, applicable **to the
greatest number of use cases without significant modifications**, because the goal is to set the
standard. All REVERSIBLE.

- **1, the receipt goes per BATCH (D-A).** It is the rule there already is -one receipt for each
  thing the node EVALUATES (RFC-0010, D-E)- applied to what the node evaluates as a unit: one
  snapshot, one verdict, all or nothing. It touches nothing existing: neither the leaf
  `recibo_digest(hash_prueba, era, n)`, nor `DOMINIO_RECEP`, nor the v6 head, nor a vector; only the
  fingerprint of the composition is born, with its domain and its KAT. It holds for any aggregator,
  with any batch size and mix, and also for the batch of one; the holder does not change his proof,
  and the only thing he needs -the response and the composition- the aggregator already has. One per
  operation would have put a third field under the signature -another `recibo_digest`, another KAT,
  other vectors, 2.11 changed for all the ways- and promised resolution to operations the node did
  NOT evaluate, because the batch aborts at the first that fails. One rule, one leaf, all the ways:
  that is what a standard can ask of another implementation.
- **2, only the responses that EVALUATE the proof consume a receipt (D-D).** It is the same frontier
  of D-E -«not even the noise»- and the same point of §569: it is reserved after checking the `seq`
  and before verifying, as in the direct way. Counting the three prior ones would open gaps in
  someone else's log for whoever sends pledges against an old head, at no cost. The holder sees them
  at once -the response says why- and proves again under the new head. Residue, declared in D-H: a
  false «old seq» is visible conduct, not portable evidence, the same class as the operator that
  does not answer.
- **3, the rejection of a pledge whose envelope verifies has a name: «RECHAZO SIN FUNDAMENTO»
  (D-E).** It costs nothing new: the pledge judge is already in the kit, without the prover, and it
  is THE SAME the node judges with (`verificar_contra_cabeza`, `PAQUETE.md` 2.10); the completeness
  envelope already delegates to other judges, and here it delegates to one more. An honest operator
  cannot produce it: the receipt only exists if the `seq` matched, hence the node judged against
  that head with that judge. And it is the only way where today it is possible: in the direct one,
  the proof of the rejection depends on the state (crack D-G of RFC-0010). It turns a wait of `n`
  heads into immediate evidence, and sets the rule of the standard: where the verifier can repeat
  the operator's judgement, it says so.

## Stages

| stage | what it delivers | does it break the wire? | status |
|---|---|---|---|
| E1 — the promise, written | this text | no | **proposed (§608), decided (§609)** |
| E2 — the core | `DOMINIO_LOTE` and `hash_del_lote` in `zk-ssl-hash`, their KAT in `spec/vectors/nucleo/` and their REGISTRY row | no (additive) | **built (§610)**: `DOMINIO_LOTE` in the bytes family of the domain REGISTRY, `hash_del_lote` with its row in the census of `NUCLEO.md` (RECIBOS), three witnesses, and the KAT `hash_del_lote.json`, the twenty-sixth, with the previous twenty-five byte for byte equal; `k` goes in the encoded length |
| E3 — the node | `zkssl_applyMany` and `zkssl_pledge` reserve and record their receipt at the same point as the direct way (§569); `operacion` in the batch error; the witness of §576, rewritten | no (additive) | **built (§611)**: the layer gains `apply_many_con_operacion` -the same judgement, and the error with the index- and `apply_many` is that one without the index; the batch reserves after parsing and before the layer, over the `hash_del_lote` of its composition, re-exported by the kit (`zk_ssl_verify::recibos`); the pledge, after checking the `seq` and before the judge; the witness of §576 falls and is rewritten the other way round, with the three prior ones asserted without a receipt. ⚠️ Until E4 the receipt exists and its resolution does not: the command does not yet know how to resolve that of an applied batch nor that of a pledge, and none is brought to it. ⚠️ §815: the point moved in the four ways: the two still reserve where this cell says, but since §815 they record AFTER the judgement —the layer's in the batch; the judge's and the layer's in the pledge— and before answering (RFC-0010, the amended D-E; `SECURITY.md` §3.15) |
| E4a — the command, the batch | `resolucion.tipo` «lote» in the completeness envelope: the composition recomposed, and applied, rejected with proof, by its form -or without grounds- and declared | no | **built (§612)**: the fingerprint recomposed with the kit's function before anything else; `acuses` one per operation resolved like verdict 1, `sobre` like 2 with its `operacion` inside the batch, and `data` with the form judged again or the fourth state; the two verdicts of the direct way go out to functions the batch reuses, with their texts intact and the 36 vectors equal |
| E4b — the command, the pledge | `resolucion.tipo` «prenda», with the «RECHAZO SIN FUNDAMENTO» | no (additive: the `data` of the layer's rejection gains `recepcion`) | **built (§613)**: the 2.10 envelope tied to the receipt by the digest of its proof; `consumo` -the PAIR, the mark under the `consRoot` in the window-, `respuesta` with the `juzgada` -the judgement repeated against the head of index `era − 1`: rejected with proof, or RECHAZO SIN FUNDAMENTO- or `rechazo` -the 2.6 of the layer's cause, with the rejected consumption equal to the mark-; live smoke with real pledges, 17 of 17 |
| E5 — catalogue and bench | its vectors in `completitud/`, and a bench that seeds against a real node the two resolutions of the batch, the three of the pledge and the rejection without grounds | no | **built (§614)**: `tools/banco_recibo_agregado.sh` against three real nodes -the APPLIED batch with real STARK proofs from the `d2_lote_rpc` example, which now forwards the composition and the response like an aggregator-, 37 of 37 envelopes; its 37 vectors in `completitud/`, COPIED from that run, and the harness gives 73 of 73 |

## Compatibility

No break. Whoever does not read `recepcion` in the batch or in the pledge carries on the same; the
wire vectors are not rewritten; the head does not change. The witness of §576 changes its meaning in
E3, and that is announced in it.

## Security

The spending key does not travel: none of this touches the proofs, only what the node promises about
them. The batch response only gains its `recepcion`: the composition is held by whoever assembles
it. What it does not cover is in D-H.

## References

RFC-0010 (D-E and its decision of §576, D-F, D-G, D-H); RFC-0007 (the rejection envelope);
`PAQUETE.md` 2.10 and 2.11; `SECURITY.md` 2.ter; §116, §569, §571, §576; in
`crates/zk-ssl-node/src/main.rs`, `zkssl_applyMany`, `zkssl_pledge` and the witness
`el_lote_y_la_prenda_evaluan_sin_consumir_recibo_y_la_via_directa_si`; and `apply_many` in
`crates/zk-ssl/src/two_phase.rs`.
