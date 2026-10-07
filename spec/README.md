# The Arqueo protocol specification — a reader's guide

This folder is the **normative surface** of the protocol: everything that
crosses the wire between a node, a client, a witness and a verifier, plus
the evidence package a verifier accepts with no wire at all. The
files here are the source of truth; this page only tells you what each one
is, in what order to read them, and where the claims that matter live. It
adds no rule of its own. The specification files are written in Spanish;
the line references below point into them so that a reader of either
language lands on the same text.

Verified against `main` at commit `c43890a`. Line numbers are those of that
commit; if a file has moved on, the reference tells you where to look.

## What is in this folder

| file | what it is | read it when |
|---|---|---|
| `RPC.md` | the JSON-RPC specification of the node API, version `zkssl/0.4` — transport, encoding, every method, every error, and what the transition log guarantees | you implement a node, a client or a verifier |
| `openrpc.json` | the same API as a machine-readable OpenRPC 1.2.6 document: `info.version` is `zkssl/0.4` and it lists 32 methods (29 `zkssl_*`, 3 `dev_*`), and every schema it references is declared | you check a node's surface; the shape of each type is in `RPC.md`, not here, so a generated client needs `RPC.md` too (corrected in §585: this cell said «you generate a client») |
| `vectors/zkssl-0.1.json`, `vectors/zkssl-0.2.json`, `vectors/zkssl-0.3.json`, `vectors/zkssl-0.4.json` | the conformance vectors, one file per wire version, never rewritten; four families keep their `0.3` captures under `vectors/0.3/` (`edad`, `pago`, `pendiente`, `rechazo`; RFC-0009 D-AB) | you check that an implementation produces the same values on the wire |
| `PAQUETE.md` | the portable evidence package: the seven forms (v1, v2 with co-signatures inside, extension, consumption, conflict, rejection, age), the envelope keys the verifier reads, the order of checks, the rejection catalogue and the exit contract of `zk-ssl-verify` — it does not cross the wire | you build or verify an evidence package, or write a second verifier |
| `NUCLEO.md` | the frozen core: every public element a verifier reaches in `zk-ssl-verify` and `zk-ssl-hash`, classified (core / reference / ledger / log) with the extension rule; the table is derived from the source and gated by `tools/check_nucleo.py` in every canon | you write a second verifier and need to know exactly what must be reproduced byte for byte |
| `vectors/paquete/` | the vectors of the evidence package — positives per form, one negative per producible rejection rule, and `MANIFIESTO.txt` with the expected exit code and message of each | you check that a verifier accepts and refuses exactly what `PAQUETE.md` says — `tools/conformidad.sh <binary>` runs the whole manifest against any binary and ships inside the artefact |
| `vectors/cable/` | the negative vectors of the WIRE: what the reference consumer (the witness, `witness --respuesta`) refuses, one per measured rejection, plus one positive per head era (v3, v5 and, since §570, v6); `MANIFIESTO.txt` holds exit code and text (`RPC.md`, section "Los rechazos del cable") | you write a second consumer of `zkssl_signedEpochHead` |
| `vectors/nucleo/` | the known-answer vectors of the frozen core: one file per NÚCLEO function of `zk-ssl-hash`/`zk-ssl-verify`, `{fn, entradas, salida}` in hex, emitted by the reference and reproduced on every canon — the bytes a second implementation must match before anything else (`NUCLEO.md`, section 6) | you implement the hash, the compositions or the preambles from `NUCLEO.md` |
| `vectors/consumo/` | the vectors of the CONSUMPTION envelope (`PAQUETE.md`, RFC-0006 E3): one positive and one negative per producible rejection rule, derived by mutation from the captures of the consumption bench, plus `MANIFIESTO.txt` with the expected exit code and message of each | you check that a verifier accepts and refuses exactly what the consumption envelope says, with `tools/conformidad.sh <binary> spec/vectors/consumo/MANIFIESTO.txt` |
| `vectors/conflicto/` | the vectors of the CONFLICT envelope (`PAQUETE.md`, RFC-0006 E4): one positive and one negative per producible rejection rule, derived by mutation from the captures of the two-ledger bench, plus `MANIFIESTO.txt` with the expected exit code and message of each | you check that a verifier accepts and refuses exactly what the conflict envelope says, with `tools/conformidad.sh <binary> spec/vectors/conflicto/MANIFIESTO.txt` |
| `vectors/rechazo/` | the vectors of the REJECTION envelope (`PAQUETE.md`, RFC-0007 E3 and E5): one positive per proven cause and one negative per producible rejection rule, derived by one mutation from the captures of a real node, plus `MANIFIESTO.txt` with the expected exit code and message of each | you check that a verifier accepts and refuses exactly what the rejection envelope says, with `tools/conformidad.sh <binary> spec/vectors/rechazo/MANIFIESTO.txt` |
| `vectors/edad/` | the vectors of the AGE envelope (`PAQUETE.md`, RFC-0007 E4): two positives gathered from the captures of a real node — every live position (`t = 0`) and the empty box — and negatives derived by one mutation each, plus `MANIFIESTO.txt` with the expected exit code and message of each | you check that a verifier accepts and refuses exactly what the age envelope says, with `tools/conformidad.sh <binary> spec/vectors/edad/MANIFIESTO.txt` |
| `vectors/pendiente/` | the vectors of the PENDING-ITEM envelopes (`PAQUETE.md`, RFC-0008 E4, by sides): for the payee's side, two positives gathered from the captures of a real node — the existence (`inferior = 0`) and the tight band (`inferior = importe`) — and seven negatives derived by one mutation each, one per producible rule, plus `MANIFIESTO.txt` with the expected exit code and message of each; the payer's side comes with E2 | you check that a verifier accepts and refuses exactly what the pending-collection envelope says, with `tools/conformidad.sh <binary> spec/vectors/pendiente/MANIFIESTO.txt` |
| `vectors/pago/` | the vectors of the PAYMENT-IN-PROGRESS envelope (`PAQUETE.md` 2.9, RFC-0008 E2): two positives gathered from the captures of a real node — the two forms of `--t`, the head's `seq` and the frontier `nacido + delta` — and one negative per producible rule, each one mutation of the frontier positive; their producer is `tools/banco_pago.sh`, and the manifest pins no size, because two proofs of the SAME pending item under the SAME head weigh differently when only `t` changes (5.A-324) | you check that a verifier accepts and refuses exactly what the payment-in-progress envelope says, with `tools/conformidad.sh <binary> spec/vectors/pago/MANIFIESTO.txt` |
| `vectors/prenda/` | the vectors of the PLEDGE envelope (`PAQUETE.md` 2.10, RFC-0008 E3): two positives CAPTURED from a real node — the same statement under the same head, the second one also published with `zkssl_pledge` under the same heartbeat — and seven negatives derived by one mutation each, one per producible rule, plus `MANIFIESTO.txt` with the expected exit code and message of each; they are captured and cannot be re-derived, because since the prover hides (§538) two proofs of the same statement do not weigh the same, so no size is pinned | you check that a verifier accepts and refuses exactly what the pledge envelope says, with `tools/conformidad.sh <binary> spec/vectors/prenda/MANIFIESTO.txt` |
| `vectors/completitud/` | the vectors of the COMPLETENESS envelope (`PAQUETE.md` 2.11, RFC-0010 E5): three positives CAPTURED from a real node that signs one heartbeat per second — resolved as a rejection with proof, NOT RESOLVED IN THE WINDOW against the first head past it, and the fourth state (exit 3), derived from the real `data` and declared as such — and thirty-two negatives derived by one mutation each, one per producible rule and per site; the applied-transition verdict had no vector until §605, since it needs a real STARK proof of an applied send, and has one since, copied from another producer (`tools/banco_mentiroso_sin_resolver.sh --guardar`) where the sdk's `e2e` example pays with a real STARK proof; the family is copied from one run of `tools/banco_completitud.sh`, because two runs differ in the unsigned `emittedAtUnix`; and since §614 (RFC-0014 E5) 37 more for the batch and pledge resolutions — ten verdicts, among them the applied batch with real STARK proofs and three rejections WITHOUT GROUNDS derived by mutation, and 27 negatives — copied from one run of `tools/banco_recibo_agregado.sh --guardar`; plus `MANIFIESTO.txt` with the expected exit code and message of each | you check that a verifier accepts and refuses exactly what the completeness envelope says, with `tools/conformidad.sh <binary> spec/vectors/completitud/MANIFIESTO.txt` |
| `vectors/ancla/` | the vectors of the ANCHOR envelope (`PAQUETE.md` 2.12, RFC-0012 E4): four positives CAPTURED from a real node, one per mode — the anchor DERIVED from a signed head alone (the binary prints it and the bench parses it: one producer), the exact anchor against its own head, the extended one with the `zkssl_consistencyProof` path, and the SPLIT VIEW, seeded by reproducing the real attack (same seed, fresh index counter, a ledger whose signed `recepCount` moved) — and seventeen negatives derived by one mutation each, one per producible rule and per site; the family is copied from one run of `tools/banco_ancla.sh`, because `emittedAtUnix` is unsigned; plus `MANIFIESTO.txt` with the expected exit code and message of each | you check that a verifier accepts and refuses exactly what the anchor envelope says, with `tools/conformidad.sh <binary> spec/vectors/ancla/MANIFIESTO.txt` |
| `vectors/ancla-cofirmada/` | the vectors of the COSIGNED ANCHOR envelope (`PAQUETE.md` 2.13, RFC-0013 E4): five positives CAPTURED from a real node — three signed heads, their anchors DERIVED by the binary, and the operator's medium publishing them under a `checkpoint` note signed with ML-DSA-44 type `0x06` and cosigned by test witnesses (one, two, a cosignature without its vkey in the envelope, a one-anchor medium, the last anchor) — and twenty-two negatives derived by one mutation each, among them the real attack: another operator's medium, well signed, carrying this head's anchor; the witness threshold has no vector, because the binary reports and does not judge; copied from one run of `tools/banco_ancla_cofirmada.sh`; plus `MANIFIESTO.txt` | you check that a verifier accepts and refuses exactly what the cosigned anchor envelope says, with `tools/conformidad.sh <binary> spec/vectors/ancla-cofirmada/MANIFIESTO.txt` |
| `rfc/PROCESO.md` | the RFC process: states, five rules, what an accepted change must ship with, and the two security clauses — a security fix's RFC is drafted in private and enters with the fix, and, from §691 on, a draft report of another project's flaw stays out of the public tree | you want to change anything above |
| `rfc/0000-plantilla.md` | the RFC template | you write an RFC |
| `rfc/0002-lotes-y-transicion-de-hoja.md` | RFC-0002, ACCEPTED: batches, and the leaf transition that took the wire from `0.1` to `0.2` | you read the log guarantees under batching |
| `rfc/0003-compromiso-v2.md` | RFC-0003, ACCEPTED: the pending commitment v2 (per-payment expiry and refund identity committed), which took the wire from `0.2` to `0.3` | you read why the current commitment has the shape it has |
| `rfc/0004-paquete-de-evidencia.md` | RFC-0004, ACCEPTED: the portable evidence package gets its own normative document (E1, `PAQUETE.md`), its vectors (E2) and the head-index binding (E3) | you read why the package is specified apart from `RPC.md` |
| `rfc/0005-nucleo-congelado.md` | RFC-0005, PROPOSED: the frozen core and the extension rule — the core is the verifier side, not the wire (D-A); what is signed grows only by version and what is not signed does not exist for the core (D-B); nothing in the core identifies anything outside the ledger (D-C). E1 (`NUCLEO.md`) and E2 (`VersionCabeza`) sealed | you read what will never change, and why |
| `rfc/0006-consumo-publicado.md` | RFC-0006, ACCEPTED: the published consumption — a public, precomputable label `H(domain, agreed identifier)` in a sixth root at rest, signed into the head as format v4, wire at `zkssl/0.3` (E2), provable without the node (E3), detectable across ledgers from two signed heads (E4); E1-E4 sealed, §413-§440-B. Uniqueness within a ledger; detection, not prevention, between ledgers; the binding to a payment is out of scope (E5) | you read how a unit is proved consumed once, and what that does not claim |
| `rfc/0007-pruebas-sobre-el-estado-comprometido.md` | RFC-0007, ACCEPTED: proofs over the committed state — the head v5 signs one new family (the parameters' digest, the pending-meta root, the two high-water marks and the supply; E1); a refusal carries its cause as data (E2) and, for the path-provable causes, a portable proof the verifier checks without the node (E3); the ageing proof over what is in flight, measured before it is built (E4); the circuit-proved causes and STARK verification inside the kit (E5). Wire stays `zkssl/0.3`; five stages sealed | you read what a third party will be able to check about a refusal, about what is in flight, and about the rules the operator runs |
| `rfc/0008-pruebas-portables-del-pendiente.md` | RFC-0008, ACCEPTED: the two portable proofs of a pending item — the payee proves, under a signed head v5 with the path inside the circuit, that a pending item exists in their name for at least X (E1); the payer proves, from the same head, an exact amount to a named recipient and no reversal before T (E2); the pledge as a transition with proof, a label of its own domain in the consumption tree, so a second MARK is `ConsumoRepetido`, while the pledge is the pair mark + envelope (E3); catalogue and bench (E4). Wire stays `zkssl/0.3`; E1 and E2 sealed (§484–§495 and §501–§509) and E4 sealed on both sides (§496–§499 for the payee, §508–§509 for the payer); E3, the pledge, sealed (§513–§546), with the stage stopped between §521 and §538 because the proof published the pledger's spend key until RFC-0009 E3b hid the witness; ACCEPTED at §547, once §546 gave the pledge its catalogue: rule 4 of `PROCESO.md` settled with measurement | you read what a third party can check about a pending item without anyone's ledger |
| `rfc/0009-lo-que-revela-una-prueba.md` | RFC-0009, ACCEPTED: what a proof reveals — the house proofs (winterfell 0.13) are sound and, until §538, did not hide their witness: a value came out literally if and only if its column is constant in the trace (§523). The RFC promises soundness and not hiding (D-A), tables what comes out, proof by proof, with the entry that measured it (D-B, §521–§526), keeps the API principle as a rule, unmet until §538 (D-C) and left a hiding prover to an RFC of its own unless one fit without touching any AIR (D-D): one fits, so it enters as E3 (D-F to D-K, §528) — a winterfell 0.13.1 fork that hides inside the core, first switched off with the spikes' falsifiers as tests (E3a, wire unchanged), then switched on once E2 counts zero literals, which moves the wire to `zkssl/0.4` (E3b). Wire moved to `zkssl/0.4` at §538; E1 sealed (§527); E2 sealed (§531: the suite in `crates/zk-ssl/src/instrumento_revela.rs` re-measures the table on every canon run, now 21 rows with the public scalars in their own column, D-L to D-Q); E3a sealed (§532: cut 0, the photo of the pristine prover, three fixed-input proofs the switched-off fork must reproduce byte for byte, D-R; §533: cut 1, the winterfell 0.13.1 fork in the tree under `crates/winter-{air,prover,verifier}`, switched off, entered through `[patch.crates-io]` with the crates.io names, the photo green with it inside; §534: cut 2, m inside the mark with a single reader, the five process-wide statics gone, switching on is `Prover::ocultacion` (off by default, and no house prover turns it on), and the seven falsifiers of D-K as tests in `crates/stark-experiment/src/falsadores_oculta.rs`, D-S to D-W; the salt of D-I is the consumer's `VC`, not the fork's, and goes with E3b); E3b sealed: cut 0 (§535) brings the salted commitment `MerkleConSal` into `zk-ssl-air` as a type with seven witnesses, switching nothing on, D-X to D-AC; cut 1 (§536) gives the photo of D-R its own pristine provers and judges with `MerkleTree`, so cut 2 can switch production on without moving it, D-Y; §537: the hidden prover checks the real trace before hiding it and answers a bad witness with `Err`, never a panic, D-AH; cut 2 (§538) switches the 23 provers with a row on with `MerkleConSal` and m 64, moves the wire to `zkssl/0.4`, keeps the 0.3 vectors under `spec/vectors/0.3/` and births the 0.4 ones from their benches, turns the byte figures into a band and the D-B table to zero (22/22, 100 cells), D-AD to D-AG and D-AI to D-AK | you read what a proof of this system shows to whoever receives it, and what it does not promise |
| `rfc/0010-el-recibo-de-recepcion.md` | RFC-0010, ACCEPTED: the reception receipt — the node signs nothing per operation, but every operation it evaluates becomes a leaf under a reception root in the signed head, with the ceiling N it promises inside (D-A to D-E); a third party then checks, with no node, that a receipt resolved within N heads as an applied transition or as a rejection with proof (D-F); the gap of the causes without a portable proof is named (D-G), and the residue —the operation for which the node never signed a receipt— is declared (D-H) | you read what the operator cannot deny having received |
| `rfc/0011-el-nodo-mentiroso.md` | RFC-0011, ACCEPTED (§604), with its decisions taken in §598 (no crate and no node library: the lies are produced by the unmodified node binary and, for omission, a forwarding proxy, all in `tools/` benches outside the Cargo workspace). Stage E1 done in §599: `tools/banco_mentiroso_vista.sh` has a real node serve a split view, and the witness's `--comparar` and `--auditar` go red naming the index. Stage E2 done in §600: `tools/banco_mentiroso_ausentes.sh` has the operator sign with one journal and show another, and `--ausentes` goes red for the first time, naming exactly the missing indices. Stage E3 done in §601: `tools/banco_mentiroso_censura.sh` puts an operator's proxy in front of an honest node that drops operations before any receipt, and asserts that every defence stays silent — RFC-0010's residue D-H, measured. Stage E4 done in §602: `tools/banco_mentiroso_sin_resolver.sh` has the unmodified node answer with a receipt and then restore its ledger to a copy, so the receipt is signed and never resolved; the completeness envelope is the only defence that sees it, once the window expires. The same bench produces the envelope's verdict 1 live for the first time. As first proposed: the lying node — a separate crate the node does not depend on, a seam where the node signs and records, and a canon gate on the dependency tree, so that every defence against a lying operator is exercised against one that really lies (`BACKLOG.md`, 93) | you read which lies an operator tells with the honest binary, and which defence sees each one |
| `rfc/0012-el-ancla-de-cabezas.md` | RFC-0012, ACCEPTED (§607): the head anchor — a six-field record derivable from a signed head alone (the key's length-encoded fingerprint, the index EMBEDDED in the signature, the digest the signature covers, and the MMR pair the head already signs), whose 32-byte digest is what gets published every `M` heartbeats in an append-only medium the operator does not control (B10.6/B10.7; the medium is not chosen here); the verifier derives the anchor, checks an exact one, checks that a later head EXTENDS an anchored one by MMR consistency, and names the split view — two heads, one embedded index — as portable fraud proof; built and measured in session 194 (§590–§593), accepted with its residue: the medium is still to be deployed (RFC-0013 proposes it) | you read how a head is published where the operator cannot erase it, and what that does not prove |
| `rfc/0013-el-medio-del-ancla.md` | RFC-0013, PROPOSED: the anchor's medium — where the RFC-0012 anchor lives: a SHA-256 tree of anchors of its own, a C2SP `checkpoint` note signed by the publisher with ML-DSA-44 type `0x06` (stateless, no guardian; an Ed25519 bridge declared while witnesses lack `0x06`), and third-party threshold witnesses over `tlog-witness`, which remember the last size they cosigned and so answer a restored directory with their own; XMSS keeps signing the head. Eight delegated, reversible decisions; E1 (the text, §603) and E2 are built — the medium's SHA-256 tree in `crates/zk-ssl-medio` (E2a, D-G decided: copied from mtc-core, §631) and its `checkpoint` note signed with ML-DSA-44 type `0x06` (E2b, contrasted with torchwood over Go's ML-DSA, §632) —, and E4, the `ancla-cofirmada` envelope in the kit, which reports the witnesses and does not judge the threshold (E4a, §633), with its catalog and bench (E4b, §634); E3 pending | you review the design, and the tree, the note and the envelope before the publisher is built on them |
| `rfc/0014-el-recibo-del-lote-y-de-la-prenda.md` | RFC-0014, ACCEPTED (§618): the receipt for what enters through `zkssl_applyMany` and `zkssl_pledge`, which RFC-0010 left out in §576 because it would have no resolution — ONE receipt per batch over the hash of its composition, resolved as applied (one ack per proof), as rejected with the proof of the operation the node names, or by its own form; and one per pledge whose proof is evaluated, resolved by the pledge envelope and its mark under `consRoot`, or named as a rejection WITHOUT GROUNDS when the envelope verifies. All additive under `zkssl/0.4`; its three decisions taken in §609 by delegation, reversible; E2 (the batch hash in the core, with its KAT) built in §610 and E3 (the node issues both receipts, and the batch error names its operation) in §611; E4, the resolutions in the completeness envelope, in §612 (the batch: applied, rejected with proof, by its form, or a rejection WITHOUT GROUNDS when the composition does not sustain it) and §613 (the pledge: accepted as the pair, rejected with proof against the head the node judged, rejected by the layer, or a rejection WITHOUT GROUNDS when the envelope verifies); and E5 in §614 (the bench `tools/banco_recibo_agregado.sh` against three real nodes, the applied batch with real STARK proofs, and 37 vectors in `vectors/completitud/`); accepted with its residue (D-H): the aggregator that does not forward, the stale-`seq` pledge that leaves no receipt, the causes without portable proof, and the operator that does not answer | you read how what enters through an aggregator or a pledge binds the operator like the direct path does |
| `rfc/0015-el-ciclo-de-vida-del-firmante.md` | RFC-0015, PROPOSED (§639): the signer's lifecycle — what the operator's XMSS key does when it rotates, runs out or loses its index (BACKLOG 84, with 92 and 19), written over what §636 and §638 measured: today a key change stops every witness and every multi-head envelope, and nothing tells a rotation from a theft. It proposes PRE-ROTATION as in RFC 8649 (each key enters with an ACT that commits the fingerprint of its successor, kept cold; the successor signs its own act, the outgoing key too when its state is reliable), one index count per operator across keys, a configurable exhaustion warning as in RFC 10033 §3.4, and a written procedure for an indeterminate index; the head does not move and `zkssl/0.4` does not rise. Its five decisions, the recommended ones, taken in §642 by the author's delegation and reversible; E2 built in §643 (the act's digest and domain in the core, its preamble and `verificar_acta` in the kit, three KATs the second implementation reproduces), E3a in §644 (the node signs its genesis and rotation acts at startup, opt-in with `--siguiente`, and does not start with a key nobody committed) E3b-1 in §645 (the outgoing key co-signs its successor's act when the operator hands over its seed, and at the ceiling the signer refuses without reserving an index) E3b-2 in §646 (`zkssl_keyActs` serves the act chain from genesis, written and read by the kit's codec) and E4 in §647 (the witness asks for the chain on a key change, follows a rotation the chain explains, stops on one it does not, names the overlap, and `--auditar` re-judges it from the witness diary, now v4) and E5a in §648 (the kit's envelopes that demand one signer's continuity —extension, consumption and the six heads the completeness envelope compares with its closing head— accept an optional `actas` chain that makes two keys ONE operator, ordered by the embedded index; without it, not a byte changes) and E5b in §649 (`spec/vectors/rotacion/`, copied from a real node that rotates twice, with the two operator behaviours the chain exposes seeded with the real keys, and the second implementation reading `actas`) and E5c in §686 (the OVERLAP envelope: one signed head outside every range of its key in the act chain is a detection with exit 0, like the split view) | you review how a key change would be told apart from a theft before anything is built on it |
| `rfc/0016-un-valor-una-escritura.md` | RFC-0016, PROPOSED (§640), with its five stages built in that seal: canonicity of the core — the field is Goldilocks, `p = 2^64 - 2^32 + 1`, and a `u64` WRITES an element only if it is below `p`. `element_from_bytes` (and so `digest_from_bytes`) rejects the eight bytes of a value `>= p` instead of reducing it, and the independent verifier reads every `u64` of an envelope canonically before composing, with `u64_canonico` as the single producer of the rule and `MODULO` taken from the field. `as_digest` does not change and no KAT moves; the MMR split becomes total; the prover's seven private copies of the embedding become the core's. Four negative vectors under `zkssl/0.4`, in `vectors/paquete/`, `vectors/ancla/` and `vectors/completitud/` | you read what a valid encoding is, and why a signed head now fixes each integer and not its class modulo `p` |
| `rfc/0017-la-firma-acredita-la-aritmetica.md` | RFC-0017, PROPOSED (§641), with its four stages built in that seal: the signature attests the ARITHMETIC, not a class modulo `p`. The layer binds `amount == pi.amount`, the state to the tree leaf and `pi.amount <= limit`, and uses checked arithmetic; the burn derives the supply natively (E1). The range-check segments of the seventeen layer AIRs drop from 63 to 62 bits so a wrapped subtraction no longer fits (`2*2^62 < p`), with `MAX_VALUE` at `2^62 - 1` (E2). Audit binds `COL_ID` to the public `public_id` and `verify_audit` bounds `lower <= upper <= 2^62 - 1` (E3). The client balance is read canonically on the wire (E4). The kit AIRs were already protected by their native precondition. Wire stays `zkssl/0.4`; nothing on the wire changes and no KAT moves | you read why a signed proof now fixes the arithmetic itself, and not an integer's class modulo `p` |
| `rfc/0018-el-tren-0-5.md` | RFC-0018, PROPOSED (§675): the `zkssl/0.5` train — the frozen-account path bound in the AIR for real (the accumulator of §511 reads a different bit from the multiplexer, measured) and four-element custodian and governance keys (today one element, about 64 bits). Both change the proof's bytes; five decisions for the author; nothing built | you decide what goes into the next wire version |
| `rfc/0019-la-completitud-que-no-se-esquiva.md` | RFC-0019, PROPOSED (§675), amending RFC-0010: the completeness window measured with the index the signature attests, not the declared one (a fabricable RED against the operator), and a `StaleState` that no longer resolves a completeness envelope by itself (a fabricable GREEN by the operator), plus the receipt leaf with the public-input digest for `zkssl/0.5`. Changes the verdict of two published vectors; four decisions for the author; nothing built | you decide how the completeness envelope should treat the node's own word |

RFC-0001 is not missing: the number is reserved for the keystore KDF
hardening and is not yet drafted (the «Número» note in the header of `rfc/0003-compromiso-v2.md` §«RFC-0003»).

## Three things that govern everything else

**1. The spend key never travels through the API.** Opening an account
sends identifiers derived on the client; paying and claiming means asking
the node for public materials, proving locally, and presenting a receipt;
reading your own balance means presenting a derived view key that
authorises reading that account only (`RPC.md` §«Principio que el API preserva»). **Until §538 the
proofs broke the principle**: the house prover (winterfell 0.13) did not
hide its witness, so every proof that takes the key —send, claim, pledge,
audit, burn— published it. Since §538 (RFC-0009 E3b-2) the prover hides
it: no proof with a row publishes a literal of its witness (`../SECURITY.md`
§3.bis). Every RFC must declare its effect on this principle, and one that
erodes it is born withdrawn (`rfc/PROCESO.md` §«Reglas», rule 3).

**2. The version number tracks the values on the wire, not the size of
the surface.** `zkssl_protocolVersion` governs compatibility. The version
in force is `zkssl/0.4` (since audit entry §538; `0.3` ruled from §354, `0.2` from §209).
It goes up when values that travel change; adding a method additively
does not raise it, because the conformance vectors do not move
(`RPC.md` §«Notas operativas»). When it does go up, the old vectors stay under their
version and are never rewritten (`rfc/PROCESO.md` §«Reglas», rule 2).

**3. No statement enters the specification without the witness that would
falsify it.** That is why the log-guarantee section of `RPC.md` was
written *before* batching was implemented (`RPC.md` §«Qué afirma el registro de transiciones»), why every
RFC is tied to an audit entry by number in both directions
(`rfc/PROCESO.md` §«Reglas», rule 5), and why each vector file carries the seal it
was issued under (its `sellado` key).

## Reading `RPC.md` in order

- **Transport and envelope** (`RPC.md` §«Transporte y sobre»). HTTP `POST /`, JSON-RPC 2.0,
  one object per request; JSON-RPC batches are not accepted. The default
  body limit is 2,097,152 bytes, measured; one operation with its proof is
  about 159 KB in the body today, so a `zkssl_applyMany` holds 13 (measured
  in §615: 13 weighed 2,064,578 and 2,076,308 bytes in two runs, and 14 get
  a 413; it said 15 until then).
- **Encoding** (`RPC.md` §«Codificación»). `QUANTITY` is a u64 in `0x` hex without
  leading zeros; `DATA` is `0x` hex of even length; a `Digest` is 32 bytes
  in the same serialisation the layer persists. A non-canonical digest is
  rejected with `-32602` before the layer is touched.
- **Methods** (`RPC.md` §«Métodos»). Read methods; `zkssl_openAccount` (an
  account is born with zero balance; the three identifiers are derived on
  the client); the two-phase payment (`sendMaterials` → `applySend` on the
  payer's side, `claimMaterials` → `applyClaim` on the payee's side,
  `applyMany` for a batch of operations against one root); and the `dev_*`
  namespace, which only exists in builds with the `dev` feature and uses
  test custodians.
- **Errors** (`RPC.md` §«Errores»). Three codes: unknown method, invalid or
  non-canonical parameters, and a layer refusal whose `message` is the
  layer's own error. `StaleState` is expected under concurrency: refresh
  the view and retry.
- **What the transition log always guarantees** (`RPC.md` §«Lo que el registro afirma SIEMPRE»).
  Consecutive sequence numbers from zero; `rootOld` of an entry is
  `rootNew` of the previous one, the first starting at genesis; `chain` is
  the running digest of the entry and everything before it; `proofDigest`
  ties the entry to what authorised it. The subsections that follow
  (from `RPC.md` §«`zkssl_applyMany` — N operaciones contra UNA raíz de arranque» through `RPC.md` §«Y una consecuencia para la conformidad») say exactly which of these survive batching and which
  do not — read them before building on the log.
- **The signed epoch head** (`RPC.md` §«`zkssl_signedEpochHead` — la última cabeza firmada», and its formats from `RPC.md` §«Formato v3»). `zkssl_signedEpochHead`
  returns the most recent head the node signed: the nine head fields,
  `publicKey`, `epochDigest`, `formatVersion`, `index` and `signature`,
  together, from the same heartbeat — one custody artefact. Format v3 adds
  `mmrRoot` and `mmrSize`, signed, so that a holder of an older head can
  verify that a newer one *extends* it without downloading the log. Three
  answers are possible and none is a generic error: no heartbeat yet, a
  node started without a key (the head comes unsigned and says so), or a
  signed head.
- **The extension proof** (`RPC.md` §«La prueba de extension»). `zkssl_consistencyProof`
  turns that check into a service: a holder sends its `mmrSize` and gets
  an O(log N) path against the history tree.
- **Inclusion, the acknowledgement path, and what a path exposes**
  (from `RPC.md` §«`zkssl_ackPath` — el camino de acuse» through `RPC.md` §«Qué promete el recibo, antes y después»). What a receipt proves, what an acknowledgement is,
  and the corrections the specification records about itself when an
  earlier statement turned out to be wrong (`RPC.md` §«CORRECCIÓN (§265)» and `RPC.md` §«CORRECCIÓN (§261)»). Those
  corrections are kept in place on purpose: the file narrates its own
  history.
- **Co-signature transport** (`RPC.md` §«El transporte de la cofirma»). `zkssl_submitCosig` and
  `zkssl_cosigs` make the node the *transport* of witness co-signatures,
  not their authority: it checks that a co-signature is for the current
  epoch and that the signature closes, and it does **not** accredit the
  witness — which witnesses count is the client's policy, not the node's.
- **Shutdown** (`RPC.md` §«Apagado»). What the operator publishes on closing
  (nothing that was not already published every heartbeat) and what the
  holder keeps: the last signed head, the proof digest of their entry, and
  the acknowledgement path.
- **Operating notes** (`RPC.md` §«Notas operativas»). One node, one writer; ledger
  parameters are immutable once persisted; distributed consensus is a
  different problem and is not implemented.

## The vectors

Each `vectors/zkssl-X.Y.json` is one object with the same eight keys:
`spec`, `sellado` (the seal it was issued under), `escenario`, `canon`,
`entradas`, `epoch_digest`, `supply`, `pending`. An implementation of a
given wire version is checked against the file of *that* version; the
conformance check refuses to validate vectors of a different version, and
that refusal is correct (the opening note of `RPC.md` §«Arqueo JSON-RPC»). The three files are kept side by
side so that a `0.1` or `0.2` implementation can still be checked against
what it claims to speak.

`vectors/paquete/` holds the vectors of the evidence package (`PAQUETE.md`): positives
per form and one negative per published rejection rule that a real package can produce,
each listed in `vectors/paquete/MANIFIESTO.txt` with its expected exit code and message.
`tools/canon.sh` runs `zk-ssl-verify` on every one of them; a single altered nibble turns
the canon red.

## Changing any of this

A change to what crosses the wire — `RPC.md`, `openrpc.json`, the
vectors — does not enter by direct commit; it enters by RFC
(the opening of `rfc/PROCESO.md` §«Proceso RFC del protocolo»). States: DRAFT → PROPOSED → ACCEPTED → FINAL, or
WITHDRAWN at any point (`rfc/PROCESO.md` §«Estados»). ACCEPTED requires the
specification updated, the OpenRPC document regenerated, the vectors
re-issued or new ones under the new version, and green suites
(`rfc/PROCESO.md` §«Reglas», rule 4). The audit entry that seals the change cites the
RFC by number, and the RFC cites the entry (`rfc/PROCESO.md` §«Reglas», rule 5).
A security fix is the one exception to the order, not to the rules: its RFC, or
the amendment, is drafted in private and enters together with the fix, its stage
already built, so the flaw is not published before its fix
(`rfc/PROCESO.md` §«Fallos de seguridad»).

## Where the rest lives

The audit record (`AUDITORIA.md` at the repository root) holds the
numbered entries cited above by `§`. The declared limits of the system are
in `SECURITY.md`. The evidence package a holder keeps after shutdown — what
`zk-ssl-verify` accepts and refuses — is specified in `PAQUETE.md`, above.
What a second verifier must reproduce byte for byte — the frozen core and its
extension rule — is enumerated, derived and gated in `NUCLEO.md`, above.
This page is maintained by hand: when you change a file
in this folder, update the line references here and the commit in the
header, and run `tools/verificar_citas.py`, which fails if a cited
document does not exist.
