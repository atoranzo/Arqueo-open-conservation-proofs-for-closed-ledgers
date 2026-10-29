# RFC-0011 — The lying node: the defences, exercised against whoever really lies

> **Español:** [`spec/rfc/0011-el-nodo-mentiroso.md`](../0011-el-nodo-mentiroso.md) is the original
> of this page, in Spanish; it prevails wherever the two differ. This translation is of commit
> `7766efb` (audit entry §917) and was produced by the project's assistant, a Claude Code session,
> in audit entry §918 (`GENAI.md`): it is a generated artefact, not text written by the author. Line
> citations of the form `file:N` refer to the Spanish files.

- **Status:** ACCEPTED (§604) — **rule 4 of the PROCESO, settled with measurement**: it does not
  touch the wire, so there is no spec, OpenRPC or vectors to move, and its suites are the four
  benches of the liar, VERDES in the author's `--bancos` on `639b5eb` (20 of 20) with his canon
  `--sello` VERDE. Proposed in §588; decisions TAKEN in §598 -delegated by the author and
  REVERSIBLE, see «Decisions»-; the four stages, from §599 to §602, each one with a bench and
  without a line of code in the workspace: D-B is not reopened. **What is accepted WITH its residue,
  declared:** censorship before the receipt is seen by no defence, and the bench of E3 asserts it;
  the crate `zk-ssl-mentiroso` and the gate D-C were not born, and they are born with the first
  lie that asks for code; and an operator who compiles their own binary lies all the same, as
  «Security» says.
- **Author:** Ángel José Toranzo Portela
- **GenAI assistance:** Claude (session 193, §588, §598 to §602 and §604) — see
  [`GENAI.md`](../../../GENAI.md)
- **Date:** 2026-09-30
- **Affected protocol version:** `zkssl/0.4` — **does not go up**: none of this touches the wire.
- **AUDITORIA entries:** §588 (the proposal); §598 (the decisions); §599 (E1); §600 (E2); §601
  (E3); §602 (E4); §604 (the acceptance).

## Motivation

Entry 93 of the `BACKLOG`: the project has defences against an operator that lies, and none has
ever seen an operator that lies. They are exercised against data FABRICATED in a test or against
an HONEST node; «alcanzable, no ejercitada» [reachable, not exercised] is the most they can say.
Measured in §588:

| the lie | the defence that should see it | how it is exercised today |
|---|---|---|
| two different heads signed with the same index (split view) | the witness, `--comparar` and `--auditar` («el mismo índice con dos digests» [the same index with two digests], `witness.rs`), and two co-signatures of one witness with the same index | journals fabricated in tests; no real node produces it |
| a signed head that its journal does not record | the witness, `--ausentes` (§283) | never ROJO in a bench |
| an issued receipt that is not resolved | the completeness envelope, «NO RESUELTA EN LA VENTANA» (RFC-0010 E4) | the bench assembles the envelope without a resolution against an honest node |
| an operation censored BEFORE issuing a receipt | none, by design: the residue D-H of RFC-0010 | declared, not measured |

⚠️ The first two rows are the snapshot of §588: since §599 the split view is produced by a real
node and the two defences see it in a bench (E1), and since §600 `--ausentes` has given ROJO
against the journal of a real node (E2). The third, since §602: the receipt that is not resolved
is signed by a real node, which no longer has a resolution to show (E4). And the last one is
MEASURED since §601: the bench that censors without a receipt asserts the silence of all the
defences (E3). All four, in «Decisions».

The last row is not a gap that this RFC closes: it is a residue to exercise in order to MEASURE
its silence. A bench that censors without a receipt and checks that all the defences stay VERDES
turns «not detectable» from a claim into a measurement.

⚠️ No self-report of the node protects from anything: «I am honest» is the operator speaking
about itself. What is built here is instrumentation, not a trust model.

## Design

**D-A. The lie lives in a SEPARATE crate, on which the node does not depend.** It is the strongest
of the three guarantees that entry 93 orders: a separate crate; a non-default `feature`, which
lives in the same crate and which `--all-features` compiles; and a run-time flag, the weakest.
The last two are DISCARDED: a production binary that carries the lie inside, switched off, is a
binary that can lie. Measured precedent: `zk-core` depends on `ceremony` only as a
dev-dependency, on purpose, for the acyclic graph. Proposed name: `zk-ssl-mentiroso`
(REVERSIBLE).

**D-B. A seam, not a branch.** The lies of interest -the split view, the signature the journal
does not record- demand the KEY: a proxy in front of the node can only lie by omission. So the
node exposes an extension point -who signs and who records-, the production binary builds only
the honest implementation, and the lying one lives in `zk-ssl-mentiroso` (the pattern
«envoltorio en vez de cambio de firma» [a wrapper instead of a change of signature], §281).
⚠️ **The cost, measured:** `zk-ssl-node` is today ONLY a binary, without `[lib]`; for another
crate to reuse its code, the node first gains a library, and its `main.rs` becomes the thin
wrapper of the honest binary. It is the most expensive cut of the RFC and it goes alone, in its
stage.

**D-C. What makes it CHECKABLE and not promised: a gate of the canon.** The tree of NORMAL
dependencies of `zk-ssl-node` does not contain `zk-ssl-mentiroso`; it can only appear as a
dev-dependency or not appear. The gate is rehearsed with its falsifier -the crate added as a
normal dependency puts the canon ROJO- before a single lie exists.

**D-D. Each lie is exercised in a bench that expects ROJO from the defence.** The success of a
liar's bench is that the defence FIRES, naming what it saw; a liar's bench that comes out VERDE
without the defence speaking is a failure. The only exception is that of the residue D-H, whose
bench asserts the SILENCE of all the defences and says so.

**D-E. Test keys, always.** The liar signs with deterministic seeds of the suite, like
`dev_openSeeded` and the TEST custodians; it never reads `--clave-fichero` or a keystore.

## Stages

- **E1 — the gate and the empty crate.** `zk-ssl-mentiroso` is born without code, with its row in
  the canon at pin zero; the gate D-C is born, with its falsifier rehearsed.
- **E2 — the proxy: lying by omission.** Without touching the node. Censoring before the receipt
  (the residue D-H, whose bench asserts the silence) and swallowing a response.
- **E3 — the node's library.** The cut of D-B, without changing behaviour: the node's suite and
  the benches, the same before and after.
- **E4 — the split view.** Two heads with the same index: `--comparar` and `--auditar` ROJOS in a
  bench, naming the index.
- **E5 — the signature not recorded.** `--ausentes` ROJO in a bench for the first time.

⚠️ **This list is REPLACED by that of §598**, in «Decisions»: without a crate or a library,
stages E1 and E3 above disappear, and the order changes. It is kept as what was proposed.

## Compatibility

None on the wire or on the vectors. The workspace gains a member, with its row in the canon,
and in E3 the node gains a library: the binary `zk-ssl-node` and its behaviour do not change,
and its suite and the benches prove it.

## Security

The API principle -the spending key does not travel- is not touched. The risk proper to this RFC
is that the lie reaches production, and D-A and D-C are the answer: there is no `feature` or flag
that turns it on in the node's binary, and the canon checks the dependency tree at every seal.
What it does NOT cover: an operator who compiles their own binary can lie all the same; this
neither prevents it nor claims to, it only makes the defences be tested against what that
operator would do.

## Decisions

The three questions that §588 left to the author, as they were put:

- **D-A**, the name of the crate: `zk-ssl-mentiroso` (REVERSIBLE).
- **D-B**, giving the node a library in E3, or limiting the RFC to the lies by omission (E1 and
  E2) and leaving the split view and the signature not recorded at «reachable, not exercised».
- **The order** E1 to E5: the gate before any lie, and omission before the key.

**TAKEN in §598**, by the author's delegation («HACER: tu decisión» [DO: your decision]) and with
the principles of the house -measure before building, the cheapest thing that answers, do not
touch the production binary without need-. All REVERSIBLE.

**What was measured before deciding.** The anchor bench (`tools/banco_ancla.sh`, §593) already
produces a REAL split view with the node's binary UNTOUCHED: its run B brings up the same seed
with fresh directories, the index counter starts again, and the node signs two heads with the
same index and different digests. The lies «that demand the key» (D-B) demand the KEY, not
different code: any operator with the seed produces them with the honest binary. That is,
besides, just what the defences have to see.

- **D-B: the node is NOT given a library.** It is not needed for any of the known lies with a
  key, and splitting `main.rs` would be the most expensive cut of the RFC with nothing asking for
  it. It is reopened if a lie appears that only modified code produces; the candidate is the last
  stage.
- **D-A: there is no crate either, for now.** The lies live in `tools/banco_mentiroso_*.sh` -bash
  and python, like the other benches-, outside the Cargo workspace: nothing can link them, which
  is stronger than the separate crate D-A proposed. The name `zk-ssl-mentiroso` stays reserved in
  case a stage needs a crate. Consequence: the gate D-C has no tree to watch and is not born; it
  would be born with the crate. The benches are watched by `--bancos` (§582), like all of them.
- **The order, redone**, from what the house already knows how to produce to what is yet to be
  measured:
  - **E1 — the split view against the WITNESS.** The method of the anchor bench: the same seed
    with a fresh counter. Success: `--comparar` and `--auditar` ROJOS, naming the index.
    **DONE in §599**: `tools/banco_mentiroso_vista.sh`, in two scenes. (1) Two nodes of the same
    seed AT THE SAME TIME, one witness on each: each witness, alone, sees nothing, and each
    journal passes `--auditar` CLEAN -the silence is asserted: one view, alone, is coherent-;
    `--comparar` of the two comes out ROJO and names EACH split index, all of the same key. (2)
    The counter restored under a live witness: the node goes down and comes back with the same
    seed on the same port; the witness STOPS with `vista-dividida` at the index, and `--auditar`
    sees it without the node. The expected indices come from the raw journals, not from the text
    of the tool. With the three checks of the witness switched off (the falsifier, outside the
    tree) the bench comes out ROJO in the four that look at them, and the live witness stops ALL
    THE SAME through the other channel: the history does not extend.
  - **E2 — the signature the journal does not record.** The operator signs with one `--diario`
    and shows another. Success: `--ausentes` ROJO in a bench for the first time. Whether it can be
    done without code is measured in it.
    **DONE in §600**, without code: `tools/banco_mentiroso_ausentes.sh`, in two scenes. (1) The
    journal swapped: the node signs with a witness watching, the operator restarts it with the
    same counter and the same ledger and ANOTHER `--diario`, and shows only the new one. The node
    starts without reusing any index -nothing in it demands that the journal continue; it only
    warns about it in its log- and the live witness records `por-detras` and goes on, the visible
    reset of §292. `--ausentes` against the journal shown: ROJO, with each index of the first
    period that the witness saw; against the two journals together, no absentees. (2) The deleted
    line: against the whole journal of an honest node, VERDE even though the node records indices
    that the witness -which samples one in two- did not ask for; with a signed line that the
    witness saw deleted by hand, `--ausentes` names THAT index and no other. Two falsifiers
    outside the tree: with the check switched off, ROJO in the two that expect ROJO; counting
    also the opposite direction (BACKLOG 80), ROJO the control of the honest node.
  - **E3 — censorship before the receipt.** A proxy that forwards and discards, written inside
    the bench. Success: the SILENCE of all the defences, asserted: the residue D-H of RFC-0010,
    measured.
    **DONE in §601**: `tools/banco_mentiroso_censura.sh`. In front of an honest node, a proxy of
    the operator that forwards everything except what it decides to keep quiet. Three sends from
    the same holder: A with the proxy forwarding -the receipt arrives, the signed head goes to
    recepCount 1-; B without a response; C with an error without a receipt. The holder knows it
    at once in B and in C, and the signed register stays at recepCount 1: the node did not see
    them. Two witnesses, one through the proxy and another direct: they end without a finding,
    and `--auditar`, `--comparar` and `--ausentes` stay silent. The falsifier -the proxy that
    forwards everything- turns ROJAS the four checks that say that the censorship happened, and
    leaves VERDES those of the silence: the defences say exactly the same with censorship and
    without it.
  - **E4 — the receipt that is not resolved, by a real node.** How to produce it without code
    -stopping the node between the receipt and the resolution- is yet to be measured. If it can
    only be done with code, D-B is reopened here.
    **DONE in §602, without code: D-B is not reopened.** `tools/banco_mentiroso_sin_resolver.sh`.
    The operator applies the operation, responds with the receipt and, before the next head,
    stops the node and restores the ledger to a copy, keeping the counter and the reception
    register. The receipt stays signed; the signed `seq` never reaches the entry, and
    `zkssl_ackPath` answers «vuelve tras el próximo latido» [come back after the next heartbeat]
    head after head; the witness stays silent. The envelope says «ventana ABIERTA» [window OPEN]
    and, with `--largo`, the window expired, «NO RESUELTA EN LA VENTANA»: NAMED ROJO, and this
    time rightly. Its control, against the same honest node, seeds verdict 1 live for the first
    time. The falsifier -the operator who does not restore- turns ROJAS the two checks that say
    that there is no resolution to show.
    ⚠️ **§815: stopping the node was enough, and without lying.** Until §815 the node recorded
    the receipt BEFORE judging, and an honest crash between recording and applying left the same
    signed state that this bench fabricates by restoring the ledger -the receipt under the
    `recepRoot`, the operation outside the ledger- and the same ROJO when the window expired:
    measured in private on `4d8017d` (`SECURITY.md` §3.15). The «and this time rightly» was true
    of this bench, which undoes the ledger, but not of every ROJO of that class: from outside, the
    lie and the crash could not be told apart. Since §815 the leaf is born after the verdict
    (RFC-0010, D-E amended), and stopping the node no longer produces it with an APPLIED
    operation: it leaves a gap, or the leaf with its operation already in the ledger. What
    produces it is undoing the ledger, which is what this bench does. With two caveats, which D-H
    of RFC-0010 declares: a death between the `fsync` of the leaf of a REJECTION and the response
    leaves a leaf whose envelope nobody has, and with a cause that has a proof (`StaleState`) the
    same ROJO when the window expires (DEDUCED; entry 129 of `BACKLOG.md`); and a node that signs
    without `--ledger` loses the ledger on restarting and keeps the leaves, which is again this
    state without lying (DEDUCED; `SECURITY.md` §3.15).

## References

`BACKLOG.md`, entry 93 (the starting design) and entry 94 (what the canon does not see); §281 (the
wrapper); §283 (`--ausentes`); §285 («quien firma, anota» [whoever signs, records]); RFC-0010, D-H
and E4; `crates/zk-ssl-cli/src/witness.rs` (`--comparar`, `--auditar`, `--ausentes`).
