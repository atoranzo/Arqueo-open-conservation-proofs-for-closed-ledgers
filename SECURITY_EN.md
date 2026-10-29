# Security policy — Arqueo (formerly ZK-SSL)

> **Español:** [`SECURITY.md`](./SECURITY.md) is the original of this page, in Spanish; it prevails
> wherever the two differ. This translation is of commit `7766efb` (audit entry §917) and was
> produced by the project's assistant, a Claude Code session, in audit entry §918 (`GENAI.md`): it
> is a generated artefact, not text written by the author. Line citations of the form `file:N` refer
> to the Spanish files.

## Read this before the rest

This text describes **design intentions and measured limitations**, not
audited guarantees:

- **Nothing in this project has been audited by third parties.** No amount of
  one's own tests replaces an external audit. There is none.
- The security properties listed are **design goals**, validated
  —where they are— by the author's own tests.
- The problems of §3 are **internal findings**. Most are measured and
  carry a reference to `AUDITORIA.md`; **re-verify them against the current code**
  before trusting them or taking them as resolved.
- The authoritative source of the real behaviour is the code. **If this
  document and the code disagree, the code wins and this document is wrong.**

⚠️ That last point is not rhetoric. This project has recorded **four times**
a guarantee stated in prose whose condition nobody verified
(`AUDITORIA.md` §95.2, §98.4). If something here sounds safer than what you can
verify yourself by reading the repository, treat it as a pending claim.

---

## Project status

- **Research prototype**, not a product. It handles no real money and must
  not handle it in its current state.
- **Single node operated by a single party.** Despite the word «Sovereign» in the former name,
  **it is not decentralised**: the operator sees the state, orders the operations and
  can censor. It is a declared characteristic, not an oversight.
- **Its thesis is not evasion**: privacy against third parties with demonstrable
  supervision —selective disclosure by the holder, issuance limits in
  circuit—.
- **Its measurable contribution** is the empirical comparison of five proof
  systems over the same circuit.

---

## 1. What the design intends to guarantee

- **Expiry of the pending with a double lock** (`AUDITORIA.md` §178-§181):
  once the declared `T` has passed, the sender recovers an unclaimed send — and ONLY
  the sender: the destination is fixed by the layer's records (not the proof) and
  the credit climb requires the salt derived from ITS key (§117). Witnesses:
  `el_ladron_con_aviso_no_puede_reembolsarse`,
  `la_carrera_post_t_el_primero_gana`,
  `las_dos_vias_de_caducidad_no_se_cruzan`. Issuance pendings
  expire by destroying (the supply drops by what it rose).

**Target properties.** Their correctness depends on the circuit constraints
being complete, and that **is neither formally specified nor audited**
(§3.1).

- **No creation of money**: a valid transition conserves the total value.
- **Spending authority**: only whoever controls the key can spend.
- **No silent rewriting of history**: the chained log prevents rewrites
  **detectable by whoever has observed an earlier head**.
  ⚠️ Conditional guarantee: someone other than the operator has to observe
  its heads. Since §245 there is the witness that records them (§2), but one
  operated by the operator itself proves nothing.
- **Privacy of content against third parties**: the leaf commitment hides
  the balance since entry 50 (§3.2); what the proofs hide since
  §538, and what is not promised, is stated in section 3.bis. The operator sees
  everything (§2), and none of this is audited.

⚠️ **Corrected in §698**: until then the third said «today nobody outside the operator
observes heads» and the fourth, «measured that it does not hold», against §2 and §3.2 of this very
document.

None should be cited outside this repository as «guaranteed». The
honest formulation is: *«the design intends X; it is not audited»*.

⚠️ **And two of these properties already failed in implementation** while the
design guaranteed them: until 31-07-2026 the layer did not verify the proofs
of the payment path (`AUDITORIA.md` §73) and the pending's commitment was not
bound to the amount (§74). Both corrected and measured.

## 2. What it does NOT protect

- **No formal proof of the layouts (under-constraining).** The
  suite proves POINTS (valid witnesses verify; the corrupt ones we
  came up with bounce), not the universal «no phantom witness
  satisfies the constraints».

  **What DOES exist today (§195-§196), and what does not yet.** The payment
  core has an **executable SPEC**: a thin interpreter reproduces byte by
  byte the gold-standard output of the Rust circuit —mutants included— and a
  gate counts **every trace cell with a declared owner**
  (`circuit_send`: 23 classes · 1334 cell-classes · **0 without owner**;
  `circuit_claim`: 21 · 1197 · 0). That closes one concrete question —«are there
  cells nobody constrains?»— and **does not close** the one that matters: that the
  existing constraints are *sufficient*. FV-1 (census) is DONE;
  FV-2 (bounded SMT spike) and FV-3 (Lean/K) remain a declared
  horizon. See `doc/VERIFICACION_FORMAL.md` and `doc/fv/`.

  ⚠️ **And this class is not theoretical: it is the one that breaks systems in
  production.** In June 2026 an **under-constraining bug in the elliptic
  multiplication gadget of halo2** was disclosed, affecting the Orchard pool of
  Zcash —the most mature ZK money system that exists— **exposed for four
  years** and detected by an AI-assisted audit, not by its suite. The response
  of that ecosystem was exactly the path declared here: a new
  audited pool and a **formal verification effort** to prove that that
  class cannot repeat. If a project with audits, years of
  production and a dedicated team took four years to see it, **this
  repository —without external audit— cannot call its circuits sound**,
  and it does not.

- **The post-T race belongs to the operator.** After the expiry `T`, claim and
  refund compete and the order within the batch is decided by whoever orders —
  it is the general ordering residue (§121), not a new surface; the
  race is declared and tested in both orders. Pendings PRIOR to
  expiry have no finish line and are immune: claim only,
  forever.
- **Immobilisation, not theft, on the withdrawn path** (`AUDITORIA.md` §177):
  the only demonstrable debt of the inherited nullifier tree is that the
  operator can immobilise; never spend nor redirect.

- **The operator sees the state.** It is the largest privacy limit and it is
  assumed.
- **The operator can censor and order.** Since RFC-0010, censoring AFTER the
  receipt leaves portable evidence; BEFORE the receipt it leaves none, and that is no
  longer merely declared: §601 measures it (`tools/banco_mentiroso_censura.sh`, the D-H residue).
  The after case is measured too: a node that answers with a receipt and undoes the
  operation is named when the window expires (§602,
  `tools/banco_mentiroso_sin_resolver.sh --largo`). ⚠️ §815: and until §815 an HONEST
  node that crashed between recording the receipt and applying the operation was named just
  the same, and so was one to which a third party with no account sent a garbage proof: both,
  in 3.15.
- **Custody of the log is in the operator's hands.** Without external
  observers of its heads, it could present different histories to different
  parties. **Since §241-§243 the node signs, issues and serves its heads, and
  there is an independent verifier** (`zk-ssl-verify`); **since §245 there is a
  witness, with TOFU** — but one operated by the operator itself proves nothing.

- ⚠️ **The anchor of the public key: TOFU since §245, and nothing before.**

  Until §244 this was a blocker with no way out. **The witness half-solved it,
  and by a path that was in plain sight**: a witness that records the
  key it sees **the first time** and **stops if it changes** does
  *trust-on-first-use* — the SSH model.

  **What TOFU does give**: from the first encounter on, the operator **cannot
  change key without a third party seeing it**. And rotating is exactly how
  it would escape a split view, so the witness **stops at both
  things**.

  ⚠️⚠️ **What TOFU does NOT give: the first encounter.** If the operator was already
  lying when the witness started, **TOFU pins the lie**. It is a
  limitation **of the model**, not of the implementation, and it is not closed with
  more code: it demands a **prior** anchor —a published fingerprint, an
  authority, a counterparty—. What TOFU contributes is **bounding the window to
  an instant** instead of leaving it open forever.

  ⚠️ And **a witness operated by the operator itself proves nothing**: it is
  circular. What the project can give is the reference implementation
  — **it removes the excuse that there is no way**, not the distrust.

  ⚠️ **And the prior anchor is not undecided: THERE IS NO KEY TO ANCHOR**
  (§246). The operator **has none**; `--clave` exists *«to
  exercise the mechanism, not as a way to operate»*, and so it is written in
  its own documentation.

  Anchoring today would be **anchoring a test key**. The decision on the anchor
  **comes with the deployment decision**, not before — and that reformulates it:
  it is not «a choice among four options is missing», it is «the key is missing».

  Today a third party verifies that the signature matches **the key the same
  node gave it**. That is circular: an operator can change key between
  two queries and **both answers verify**.

  ⚠️ Even if the private key lived in an HSM with three custodians, **the
  witness still could not state whose the signature is**. The missing link
  is not in the operator: it is in **which anchor the third party uses**.

  Options enumerated, **none chosen**: a fingerprint published outside the
  node · a transparency log · a counterparty that anchors it · a
  certification authority. **It is chosen according to who is going to use the anchor, and
  there is nobody** — deciding it now would be fixing the form before the data, which
  is what §242 avoided with the history.

  It is what Certificate Transparency solves by publishing the log keys
  **outside the log**.

- **Custody of the private key: declared, not checked** (§244). The
  operator states a model with `--custodia`, and the node serves it in
  `zkssl_signedEpochHead`. ⚠️ **Only `fichero` is checked**; the rest are
  its own claims. The value is not in their being true, **but in that
  lying in them is opposable**.
- ⚠️ **The index of the signing key: one process per counter, and what
  that does not cover** (§709). XMSS is stateful: every signature spends a leaf, and two
  messages signed with the same leaf compromise the key. The index
  guardian persists its counter with `fsync` before each signature and, since §709,
  also takes an exclusive lock on the file: a second process on the
  same counter —another node, another co-signer of the witness— **does not start**, and the
  error says which process holds it, if the system shows it. When signing,
  the node's `--indice-firma` and `--contador-recepcion` and the witness's
  `--indice-cofirma` are mandatory and go with their absolute path. Until §709 the node's two
  had a default value relative to the working directory, and it was enough to
  launch two nodes from the same place for them to share a counter; the witness's
  co-signer did not exclude a second one on its own.
  **What the lock does NOT cover**: a network file system, where the
  lock may not cross from one machine to another; another machine, or a copy of the
  counter; **the same seed with another counter**, because two files are two
  locks; and a process that opens the file without asking for the lock, which in unix
  is advisory and not mandatory. Nor does it do anything against a counter that
  goes backwards or is lost: that is declared by `doc/CONFIANZA_RESIDUAL.md`.
  ⚠️ §860: that is why a replica or a handover does not carry the counter or the seed to
  another machine that could sign; with the key's state not reliable, the
  handover is D-G of RFC-0015, PROPOSED; and what the node does not prevent —the
  same key in two places, with two counters— is entry 124 of
  `BACKLOG.md`. The operator's high availability is neither built by the
  project nor prescribed by the standard (entry 17, closed).
- ⚠️ **The operator can change the verifier, and today that is invisible.**
  It is **the greatest power of all** and it was not in this list: whoever can
  update the verifier **changes what a valid transition is** —more
  powerful than any operation, because **it redefines the rules under which
  all the others are judged**—. An operator that replaces it can accept
  as valid what the published rules would reject, **without leaving a trace in
  the state**. There is no notion of «rules in force» today —`OpKind` says which
  circuit to use, not which version was active—, so the change **is neither
  recorded nor checkable afterwards**. The designed closure
  —`hash_verificador_vigente` in the attested head, which makes every update
  public— is in `doc/CONFIANZA_RESIDUAL.md` §2.2 and is
  entry 54; **it first requires giving the system that notion**.
  ⚠️ **CORRECTION (§321, 2026-08-19): the condition of the previous
  line is corrected since §246.** «Giving the system that
  notion» is almost circular —`hash_verificador_vigente` IS that
  mechanism—. The real reason it cannot be filled in today: **the AIR
  is CODE, not data**. The only thing hashable at runtime is the
  `ProofOptions`, and **an operator can change the AIR leaving them
  identical**: the field would not be empty, it would be **blind** — and a blind
  field goes unnoticed lying precisely about what exists to detect.
  It depends on **entry 55** (the AIR as data) or on **reproducible
  compilation**; hashing the source at compile time is no good, because it does not prove that
  the binary was built from that source. The previous line is kept and
  cited instead of deleted (§247).
- **There is no recovery if the node disappears.**
- **Metadata**: which positions change and when remain observable.
  Measured field by field in §231: a send reveals **sender, amount and
  `notice.position`**; a claim reveals **receiver, amount and the same
  position**. Between §523 and §538 the send proof also published
  **the identity of the receiver** (measured in session 163, §523), so
  whoever processed only sends saw the whole edge; since §538 the
  proof hides it (§3.bis). The node always sees it, through the proof or not:
  it receives `receiverId` in `zkssl_sendMaterials`. An **aggregator** (§223)
  that only sees sends no longer reads it in the proof, and what it may deduce
  from a proof is bounded by RFC-0009 (D-A, D-I), not by this document.
- **Soundness of circuits and of the proof system**: not formally verified.
- ⚠️ **Rotating the key requires two custodians.** A holder can spend without
  anyone's permission and **cannot improve its own security without the permission of
  two** (§98.4).
  ⚠️ §860: no longer entirely. The compromised view alone and hygiene without suspicion
  do not require it: the transfer, R4 of `doc/ROTACION_DE_CUENTAS.md`, with no
  code, while there is a slot left in the account cap (entry 8); but it retires neither
  the old key nor the old identity, which keep claiming and opening the view. Theft,
  loss and any suspicion about `sk` still require the two
  custodians —recovery— until the rotation pre-committed in the
  salt is built (D-1 of `doc/ROTACION_DE_CUENTAS.md` §6, entry 52
  of `BACKLOG.md`).
- ⚠️ **Governance does not rotate: its compromise and its loss are forever in its ledger** (§860;
  D-3 of `doc/ROTACION_DE_CUENTAS.md`). The root is fixed when constituting the ledger and is
  compared on reopening it. A leaked key stays in the set, and the margin drops by one key forever.
  With two leaked, the threshold, whoever holds them rotates the custodians to their own; the
  legitimate governance can rotate them back, but what they issue or recover in the meantime is not
  undone. If fewer than two keys remain, the custodians do not rotate again in that ledger and, once
  their quota of interventions is exhausted, they do not issue, nor freeze, nor recover, unless the
  operator raises the quota, which protects no authorisation. The way out is a new ledger, and
  bringing the holders to it requires the claim of B18.2, which does not exist. `PRINCIPIOS.md`
  §4.1.
- ⚠️ **Opening an account is not authorised, and the account cap is exhausted
  forever** (§860; entry 8 of `BACKLOG.md`, closed). `zkssl_openAccount` opens with
  `open_with_id`, without proof and without rate limit
  (`crates/zk-ssl-node/src/main.rs:3783-3796`), and whoever reaches it can
  exhaust the account cap (`max_accounts`, 1,000 by default in the node,
  `main.rs:248`). The cap is immutable and an account is not closed, so
  exhausting it leaves the ledger without openings forever, and also removes the transfer
  —R4 of `doc/ROTACION_DE_CUENTAS.md` §4—, the only rotation without custodians
  that exists today; and each legitimate R4 spends a slot that does not come back, with no
  attacker. It is a liveness attack, and liveness today belongs to the operator, who
  decides who reaches the method: the node's lack of authentication and of rate
  limit, and its `127.0.0.1` by default, are declared by section 3.3; what is new
  is that the cap does not come back. It was decided not to require custodian authorisation
  to open: it would be a second veto over the opening without removing the operator's.
  And the proof of possession of entry 119 does not close it: it proves who can
  occupy an identity, not how many fresh identities are opened.

### 2.bis The two residues that remain, and what eliminates them

After entry 50, the residual trusts of this design are reduced
to two, and it is worth naming them looking forward:

1. **The order and the completeness of history.** The operator decides what
   enters and in what order, and could omit. Designed mitigation: attested
   heads, receipts and **acknowledgement** (§121,
   `doc/CONFIANZA_RESIDUAL.md`) — lying comes to leave fail-stop
   evidence. Elimination: consensus/replication or external anchoring of
   roots (format and verifier built: RFC-0012,
   `doc/ANCLAJE_EXTERNO.md`; pending deployment in a medium, with its
   cadence `M` to be measured).

2. **The operator sees the state.** Privacy is against third parties who
   only see roots and signed heads, not against whoever maintains the
   ledger; against whoever sees a proof, since §538 (§3.bis); the
   holder has an authenticated view (49-A) and the rest is assumed and
   documented. Elimination: blind-operator architecture (B11) or
   federation.

No ZK proof replaces these two; what this project demands is
that they be **written, measured and with their attack designed** instead of
hidden behind the word «decentralised».

### 2.ter The aggregator model — recommendation, not requirement

⚠️ **Neither of the two halves of this exists.** Neither the turn nor the split
is built or measured: the wire allows them as it stands, and that is
all that is known. This section fixes a deployment decision **before**
someone takes it out of convenience; it does not describe something that works.

⚠️ And it is a **desk recommendation with no counterparty to validate it**. If
tomorrow a real participant appears, the first thing they will ask is **who
operates the claims half**, and that question is not answered by this repository.

#### The decision

**Explicit turn, and two alternating aggregators** — one for sends, another for
claims.

The argument is not about performance: it is that **the single aggregator does not need
the graph for its job**. It gathers receipts and sends them in one request. Seeing
who pays whom adds nothing to that function; it is a side effect
of convenience. **Granting observation that is not needed is hard to defend
afterwards.**

And the cost is measured: **zero in the node**. The turn is needed in any
case because the root demands it (§230); the split rides on top.

#### What the property gives, and what it does NOT

- **Each aggregator sees half an edge**: sender + amount, or receiver +
  amount, plus `notice.position` (§231). ⚠️ **That is not zero: it is
  commercial information.** Who pays, how much and how often is enough
  for many analyses.
- ⚠️ **If the same entity operates both halves, the property is lost
  entirely and nobody notices from outside.** It is written as a **condition**, not
  as an assumption: the system does not check it, nor can it.
- **The node sees both halves**, by definition. This is confidentiality
  **between participants**, not against the operator. It was already said in §2.2;
  this section should not be read backwards.
- ⚠️ **Two aggregators are two points of censorship, not half each.** The
  sends one blocks a payment **whole** without touching the claim. That falls under the
  **admission receipt** (§121, `doc/CONFIANZA_RESIDUAL.md`) — and it is the
  **fourth** convergence towards that piece, which is still unbuilt.
  ⚠️ **Since RFC-0010 the reception receipt exists, and for what is
  aggregated, since §611.** Until then it covered `applySend` and
  `applyClaim` (D-E, §576), and the batch an aggregator sends through
  `zkssl_applyMany` did not carry it. RFC-0014 (§608) proposed the batch's
  -and the pledge's-, and since its E3 (§611) the node issues it: ONE per
  batch, over the fingerprint of its composition, which the aggregator forwards to
  each holder with the response, and since its E4 (§612) the completeness
  envelope resolves it: applied, rejected with proof, by its form
  -or RECHAZO SIN FUNDAMENTO, if the composition does not sustain it-, or
  declared. The aggregator that does not forward is left out (D-C, D-H): the
  holder chooses it, and the direct path stays open; the aggregator is a
  recommendation, not a requirement.

#### What remains open

- **The cost of the mixed batch against two separate ones is not measured.** J.1
  measured that the mixed one is admitted; the recommendation assumes that separating costs
  a turn, but **how much that turn costs is not known**. §230 bounds the
  price of **not** taking turns —the loser throws away 75 % of its proofs—,
  not that of taking turns.
- The turn mechanism —external lock, queue, rotating witness— is not
  chosen. It is only stated that **one is needed** and that **it is not a
  consensus**: see `spec/RPC.md`.

## 3. Identified security problems

> Internal findings, **not audited**. Almost all are **measured** with
> date and reference; they are listed because hiding known weaknesses would be
> the opposite of the faithful picture. One of them (§3.3) **changed shape**
> when the network layer appeared in §197: read it whole even if you knew it.

### 3.1 Absence of a formal specification of the AIR — **highest priority**

A circuit without complete constraints can admit a fraudulent witness
that passes verification. **A soundness failure is invisible counterfeit money.**

As long as there is no formal specification of each AIR —what is constrained, of
what degree, and what explicitly is **not** constrained— with negative soundness
tests, the «no creation of money» property rests on an
unchecked assumption.

⚠️ **And it is not hypothetical in this project**: §72 records a constraint
written **on the wrong lane** —well formed, of the right degree, and
binding what it should not have—. No tool detected it.

**Status: open.** Backlog 48 (B12.1).

### 3.2 The leaf commitment **IS ALREADY hiding** — MEASURED and RESOLVED (entry 50)

**Old world**: `native_leaf(identity, balance, nonce)` = `H(H(id, saldo), nonce)`, **without salt**.
And `path_for` hands the client the level-0 sibling, which **is the leaf of the neighbouring
account**.

**Measured on 31-07-2026: the neighbour's balance was recovered in 10.84 s.**

The cost is a **curve** over the balance range the attacker assumes:

| assumed range | cost, one core |
|---|---|
| 0–10,000 € | **2.4 min** |
| 0–1 M € | 4.1 h |
| 64 uniform bits | 8.3 × 10⁷ core-years — **which they never are in money** |

Scope: **one account** per path. And the indices were sequential, so
**the neighbour was chosen**. Both things, dead — see below and §157.

The solution §99.3 discarded —deriving on each write— **was not
needed**: the salt is fixed ONCE on opening (`derive_leaf_salt(sk)`, §117)
and is STORED in the record; whoever writes without the secret **reads**
`r.leaf_salt`, and recovery preserves THE COPY (§93.4).

**Status: RESOLVED.** Entry 50 CLOSED and tagged (`entrada-50`):
leaf wrapped in tree and circuits (flip D4), placement `public_id mod
capacity`, and the sweep turned into a CONTRACT (`hallado.is_none()`).
`AUDITORIA.md` §117, §156-§158.

### 3.3 The account-reading contract does not require authorisation — ⚠️ MEASURED

`account_view(index)`, `balance_of`, `nonce_of` and `public_id_of` **take an
index and ask for no credential** — the OPERATOR's by design (§129); the
holder has `account_view_authenticated` (49-A). And the indices are **no
longer** sequential: placement `public_id mod capacity` (F3, §157)
— enumerating them requires guessing, and the contract-test watches it.

⚠️ **ATTENTION: the exemption of this finding EXPIRED.** Until §197 this
section said «there is no network layer in this repository». **Today there is**:
`zk-ssl-node` (JSON-RPC 2.0, axum). What the node really does, read
from the code and not assumed:

| method | credential | verdict |
|---|---|---|
| `zkssl_accountView` | **requires the VIEW key** (49-A) | ✅ the RPC was born with access control in place |
| `zkssl_publicId`, `zkssl_logEntries`, `zkssl_epochHead`, `zkssl_supply`, `zkssl_accountCount` | none | **public by design** — the log and the aggregates are auditable on purpose. ⚠️ **CORRECTED (§656)**: the `pending` of `zkssl_supply`, queried after each entry of `zkssl_logEntries`, gives by differences the amount of each send and claim; the proxy must not expose it to third parties (section 3.9) |
| `dev_*` | double lock: compile feature **and** `--dev` | a production build does not have them |

**What remains open, said plainly**: the node **has no
authentication, no TLS and no rate limit**, and the only thing that today separates
«leak towards-the-operator» from «leak towards-third-parties» is that it listens on
`127.0.0.1:8545` by default. **Publishing it on `0.0.0.0` without a proxy in
front is exactly the scenario this section warns about.** ⚠️ **§838: the stretch up to
the proxy.** It is sent to the node by the SDK (`crates/zk-ssl-sdk/src/lib.rs:192`, `:227` and
`:267`) and the cli (`crates/zk-ssl-cli/src/pago.rs:150`, `cobro.rs:260` and `prenda.rs:163`),
both through `ureq`; with an `https` URL, it is encrypted by `ureq`'s TLS, which is `rustls` over
`ring`, and its confidentiality in transit rests on a classical ECDHE (X25519, P-256 or P-384):
exposed to «store now, decrypt later». It is the §106 exception for transport, declared in §838
of `AUDITORIA.md`; with an `http` URL, it travels in the clear.

⚠️ **And a second finding, new**: the node opens the ledger **without encryption
at rest**. The layer has had `open_encrypted` and `zk_ssl::crypto` for a long
time, and the SDK's wallet sleeps encrypted since §199 — **but the node
binary wires no key**. Whoever steals a node's disk reads
the balances. It is not a failure of the primitive: it is that it is not connected.

**Status: partially resolved (authenticated view), and open in
deployment** (no auth/TLS/rate limit; node ledger in the clear).
Backlog 49. `AUDITORIA.md` §93.1, §129, §157.
⚠️ §860: 49-A is closed, and that pointer covered two things. The node's
ledger in the clear is entry 127 of `BACKLOG.md`. The lack of authentication,
of TLS and of rate limit is a deployment residue with no entry: the proxy
in front, above; and for `zkssl_openAccount`, entry 8, closed, with its bullet in
section 2.

### 3.4 Space of the anti-double-spend identifiers — **does not apply**

The production path is the **two-phase** one, and **it uses no nullifiers**: a
send changes the payer's balance, then its leaf, then the root, so that
a resend presents a stale root and is rejected. And the root alone was not enough:
the global state repeats after a refund or an A→B→A cycle, and a receipt already
applied was valid again. Since §654 the layer also keeps the fingerprint of each
holder's proof already applied and rejects the one that comes back, and since §659 also
the one that only changes in inner nodes of a Merkle batch (§3.9).

⚠️ **Nuance (§715): the fingerprint of the bytes was not enough.** The §654 guard recognises a proof
already applied by the fingerprint of its bytes, and §659 removed one way of dodging it. That
fingerprint only identifies the operation if each proof has a single encoding, and it did not.
Without the witness or the key, from the bytes of a proof came others that verified the same: a
length written with more bytes than necessary, an extra row or group in an opening, an extra FRI
layer, or the number of constraints with a multiple of `2^32` added. With a length written long or
with an extra group in a FRI layer, a proof already applied was valid again under the same
conditions of §654: the roots that recur after a refund or an A→B→A cycle. What §659 took as «now
already unique» was not.

**Closed, in two places.** The fork's reader and the verifier's channel require a single encoding:
the minimal length, one row per queried position and one group per folded position (§715); the
number of constraints and that of FRI layers, within their range (§711). A proof that does not
comply is rejected with its cause. And the layer also keeps, of each holder's proof applied, the
fingerprint of its content: its commitments, which do not depend on how the openings are written.
`ya_aplicada` looks at both fingerprints.

**What it does not close.**

- **The content fingerprint does not persist.** After restarting the node, the guard looks only at
  the bytes, and the defence is the fork's reader and channel.
- **Receipts and acknowledgements** keep identifying the proof by its bytes. With the canonical
  reader, those bytes are unique in the measured forms. In receipts it has not been measured.
- **The underlying closure** is still the one §659 states: the nonce in every debtor transition,
  which the `zkssl/0.5` train asks for.
- **The kits published up to 0.4.3** accept those encodings. The kit compiled since §715 rejects
  them. ⚠️ §726: and 0.4.4, published. ⚠️ §917: and 0.5.0, published (`zkssl/0.5`).

⚠️ **The problem existed and was withdrawn with its path.** The one-step path
derived the marker's position from the marker itself, with a birthday
collision around **sixty-five thousand payments** against the four
billion the tree announced.

⚠️ **And it was avoided, not solved**: what replaces the marker is the
chaining of roots, which **demands a total order** — the one a single node
gives and a distributed system does not. Whoever distributes this recovers the limit
intact.

**Status: closed in the single node, open for any distribution.**

### 3.5 Duplicate definitions of the leaf — ⚠️ MEASURED

There are **three** `native_leaf` in the tree: in `circuit_settlement`,
`compliance_circuit` and `double_entry`.

**Same structure, different domain width**: the production one takes the
identity as a `Digest` —256 bits—; the other two as a `BaseElement` —64—.

⚠️ **Sharing the name makes it worse**: it invites assuming they are the same function.
Whoever verifies one and takes the other two as good is mistaken, and nothing in the
code warns them.

**Status: open.** Backlog 51. `AUDITORIA.md` §94.

### 3.6 The proof does not bind the holder's position in frozen — ⚠️ MEASURED, closed in the layer

In the five circuits with a frozen phase (`burn`, `claim`, `claim_v2`, `send`,
`send_v2`) the column of the bit of that climb (`COL_FBIT`) is not bound to that of the
account path (`COL_BIT`). What the AIR proves is that **some** position of the
tree has a zero leaf, not the holder's. Measured on 16-sep-2026 (`AUDITORIA.md`
§487): a frozen account that proved on its own behalf with the path of the neighbouring
position verified in all five and the applier accepted it in `apply_send`,
`apply_claim` and `apply_burn`. It requires the holder's own spending key and does not break
conservation: it is a fail-open of a compliance function, not of the money.

**Closed in the layer** the same day (S487): `validate_send`, `validate_claim` and
`apply_burn` reject with `AccountFrozen` before looking at the proof; three witnesses
falsify it. **Open in the AIR**: whoever only sees the proof still cannot know whether the
holder is frozen; binding the position in the five circuits is fix B,
a format break, dated behind the AIR of E1 of RFC-0008.

There is no deployed ledger and no third parties to notify; that is why the notice is this section
and entries §487 and §488, not a GHSA. `AUDITORIA.md` §487, §488.

⚠️ **§670: fix B is in the tree and does NOT bind.** §511 gave the five circuits
two accumulators of the position —one per climb— and an equality between them, and took the
binding as done. Measured in the re-triage of the second swarm on `ed0f96b`: the
accumulator of the frozen climb does not add the same bit the multiplexer of that
climb reads, so the equality of accumulators does not force the path walked to be
the holder's. A trace of a frozen account that walks another position **verifies**
(`prove` and `verify` give `Ok`), and the layer stops it with `AccountFrozen`. The status of this
section does not change —closed in the layer, open in the AIR—, but the AIR fix
**is not done**, although §511 says so: it has to accumulate the same bit the
multiplexer reads, in all five, and it changes the proof format (`zkssl/0.5` train).

⚠️ **§679: nor does the account accumulator bind.** Measured in the session that decided
RFC-0018: `COL_IACC` adds `COL_BIT` on row +7 of each cycle of the account climb and the
multiplexer of that climb reads it on row +0 of the next, the same offset. A trace that
climbs the accounts through the frozen holder and puts on row +7 the bits of the free neighbour
also verifies. Same class and same defence: the layer stops it with `AccountFrozen`. The fix
(E1 of RFC-0018, variant A measured) binds both accumulators.

**§680: bound in the tree's AIR.** E1 of RFC-0018 is built in the five
circuits: the step of each accumulator goes on the rows of the multiplexer of its climb and
adds the bit the multiplexer reads. The two measured attacks stop verifying in all
five (ten falsifiers). The wire still says `zkssl/0.4` until the cut of the
`zkssl/0.5` train, which raises the version and emits its vectors; until then what binds is the
tree's code, not a published version.

### 3.7 A malformed proof aborted the process — ⚠️ MEASURED, closed on the salted paths

When reading a proof, `winter-utils` 0.13.1 (`read_many`) and `winter-crypto` 0.13.1
(`BatchMerkleProof::read_from`) allocate memory with a count taken from the bytes
themselves BEFORE checking that those bytes exist. A malformed proof that declares a
gigantic length makes the process ask for hundreds of GB, and a failed allocation **is
not a panic**: the process aborts, and the node's panic net (§530) does not see it.
Measured on 28-sep-2026 (`AUDITORIA.md` §575): the §519 witnesses that flip a
byte of a pledge proof aborted the node asking for 862,916,669,440 and
544,766,607,872 bytes, once in thirty runs and once in the LIVE run of a block;
proofs carry salt since §538, their bytes change on every run, and the flipped
byte sometimes lands on a length. Anyone who sent a proof to the node could trigger it,
and a malformed envelope made the command abort instead of exiting ROJO.

**Closed** in §575 in the two places that produced it: `Proof::from_bytes` reads with a bounded
reader (fork of `winter-air`, `src/proof/acotado.rs`), and the batch of the salted openings
(`zk-ssl-air/src/sal.rs`) is read with its counts bounded, also in the FRI layers. Four
deterministic witnesses falsify it. **Residue, MEASURED in §578: none on the live paths.** Every
verification that reaches a foreign proof reads with salt: the fifteen `verify` of the layer, the
custodians' threshold pair and the five of the kit, and two witnesses bind it. ⚠️ §913: the 21 sites
go through the single judge (`zk_ssl_air::juez::juzgar`), whose only call to `verify` reads with
salt; the two witnesses are now the gate of the single judge, in the layer and in the kit. The
UNsalted verifiers remain in code that no live path reaches —`range_check` and the `ComplianceAir`
of `settlement-prover`, experiments, and `instrumento_edad`, tests only—; if a path reached them,
they would read their batch with upstream's `BatchMerkleProof`, unbounded. ⚠️ **CORRECTION (§578):**
§575 wrote here «the two UNsalted circuits (`range_check`, `governance`)» and «what foreign input
reaches them is unmeasured». The list came from a truncated search and was wrong on both sides:
governance is verified with salt (`verify_threshold_pair`), and unsalted there are more, none
reachable. Upstream `winterfell` has the same defect. **Reported on 01-10-2026** by private email to
its maintainer —GitHub's private advisory is not enabled on `facebook/winterfell` and its policy
inherits Meta's, which requires a Facebook account—, with the fix offered as a PR and without a
reproducer; awaiting reply. `AUDITORIA.md` §575, §578, §625.

### 3.8 A signature attested an integer modulo `p` — ⚠️ MEASURED, closed in the core and in the command

The core composes over `F_p`, `p = 2^64 - 2^32 + 1`, and two readings reduced silently: `as_digest`
embeds a `u64` reduced modulo `p`, and `element_from_bytes` read eight bytes with
`BaseElement::new`, which also reduces. So a signed head fixed each integer only **modulo `p`**, and
the command reasons about the integer. Measured on 01-10-2026 (`AUDITORIA.md` §640) on real vectors
mutated in a single field: the same signature attested a head with `n` and another with `n + p`; the
completeness envelope that names the operator —«NO RESUELTA EN LA VENTANA»— turned into «ventana
ABIERTA» with the same signature, so that **the accused chose the verdict** by writing `n + p`; an
extension with `mmrSize + p` **hung the command** (the MMR partition overflowed for sizes greater
than `2^63`); and zero written as `p` passed in the reference and was rejected in the second
implementation.

**Closed** in §640 (RFC-0016): `u64_canonico` is the only producer of the rule «a `u64` writes an
element only if it is less than `p`»; `element_from_bytes` applies it to each element and the
command to each `u64` of the envelope, before recomposing and before the signature; the MMR
partition is total. Four negative vectors bind it, and the second implementation rejects them with
the same text. No KAT nor any custodied head moves. **Residue**: the wire's `QUANTITY` reader and
the witness's recomposer keep reading without the rule —what they sign and compare is the digest,
not the integer—, and the published kit `arqueo-verify-v0.3.0` predates it: it accepts the four
negatives until a new release. The `arqueo-verify-v0.4.0` (§658) rejects them, measured with the
downloaded kit. ⚠️ §825: the residue above is no longer today's. The witness's recomposer reads with
the rule since §775, and so does the SDK, in the `logSeq` of the record; the `QUANTITY` reader
requires the minimal encoding since §662 and leaves the bound to whoever uses it
(`zk_ssl_wire::cantidad_canonica`). When RFC-0016 was accepted, the current release was
`arqueo-verify-v0.4.3`, published with its advisory (§818), which rejects the four just the same,
measured with its published binary (§825); 0.4.2 (§671), published and no longer current, too. And
RFC-0016 is ACCEPTED since §825, with its residue declared in its header.

### 3.9 Eight findings from an analysis with agents — ⚠️ MEASURED, seven closed and one bounded

In §637 a Claude Code session analysed the tree with a swarm of agents
([`doc/blueprint-v2.md`](doc/blueprint-v2.md)) and handed the author, in private, eight findings
open in `main`. They were fixed before being described here, as section 5 asks. Three were
reproduced with tests before the fix; each fix carries falsifiers that fall with it disabled.

| finding | what it broke | priority | closed in |
|---|---|---|---|
| The wire's and the kit's hexadecimal was sliced by bytes of a `&str` | a single request without credential with a multibyte character halted the node (PARADA); the kit exited with 101 instead of ROJO; `+` and upper case were accepted | P0 | §650; ⚠️ in the `u64` (QUANTITY), §662; in the witness and the SDK, §775 |
| The shape guard did not look at the meta's mark | a hidden proof with the meta emptied made the layer panic (with the node's lock taken) and the kit too | P0 | §651 |
| The hiding seeds were 64 bits | whoever sees a proof cleared any constant column of the witness —the spending key, a custodian's— with about 2^74 Blake3 compressions (ESTIMATED) | P0 | §652 |
| `applySend` and `applyClaim` stored the amount and the balance the client sent | a holder broke conservation in the `u64` accounting and the ledger did not reopen | P0 | §641 (RFC-0017) |
| `Proof::from_bytes` accepted trailing bytes | the fingerprint of a proof did not identify an operation | P1 | §653 |
| Resending was stopped only by the equality of roots | after a refund or an A→B→A cycle, a receipt already applied was valid again | P1 | §654 |
| `zkssl_supply` publishes without credential the total in transit | by differences it gives the amount of each send and claim | P1 | **bounded**: see below |
| A store failure did not halt the node | memory could sign roots the disk does not have | P2 | §655 (c); ⚠️ §815: §655 halted the dispatch and not the heartbeat, which kept signing in PARADA; and `zkssl_publishConsumo` returned the failure as a refusal. Closed whole in §815 (3.15) |

⚠️ **CORRECTION (§815), cited and not deleted: «seven closed» were not seven.** The store failure
was closed by §655 in the dispatch, not in the heartbeat. With the node in PARADA, `latido::latir`
did not look at the PARADA and kept signing heads from memory, the one that runs ahead of the disk:
measured in private on `4d8017d`, heads of `seq` 9 with the disk at `seq` 8, which is exactly what
the row says was closed. And `zkssl_publishConsumo` returned the store failure inside a result,
without PARADA. Until §815 there were six closed, one half closed and one bounded; since §815,
seven. The comment of `despachar` («the node does not keep serving») and that of the §655 test carry
their mark. It is told by 3.15.

⚠️ **What should be done if the system was used with own keys.** The hidden proofs issued
between §538 and §652 protect the spending key and the custodians' only to about 2^74
classical operations (ESTIMATED; with Grover, about 2^32 iterations). The attack is offline and
an issued proof cannot be hidden again: **whoever has published proofs with own keys
must rotate them**. The known exposure is development and test deployments, with keys already
public: the node without the `dev` feature does not start (§637).

⚠️ **`zkssl_supply`, bounded and not closed.** The `pending` field is normative (`spec/RPC.md`,
`spec/openrpc.json`): requiring the operator's credential or removing it changes the wire, and it
will be done with the next version that is already needed, not raising it only for this.
**Meanwhile, the proxy that publishes the node must not expose `zkssl_supply` to third parties**:
the row of the table of section 3.3 that called it «public by design» stands corrected. Whoever sees
a send or claim proof already sees its amount (they are public inputs, RFC-0009); what the method
added was giving it to anyone.

What remains open of these eight, and is the author's decision: parts (a) and (b) of the store
failure (recording before verifying, and caching the reception pair), the wire change of
`zkssl_supply`. ⚠️ §815: (a) changed shape halfway. Since §815 the leaf is recorded AFTER judging
(3.15); but the reception counter still reserves the `rx` with its `fsync` BEFORE verifying
(`recibir`; RFC-0010, D-E), so paying an `fsync` for a request without credential before verifying
is still true. Each request the layer judges now pays two, the counter's before and the leaf's
after, also that of a third party with no account, because a rejection leaves a leaf (deduced from
reading the order; latency is not measured). And it was that leaf that gave a third party a ROJO
against the honest operator, until kit 0.4.3 (3.15). The new version of the kit is published:
`arqueo-verify-v0.4.0` (§658); `v0.3.0` is still affected by the first two and by the trailing
bytes. ⚠️ **And a variant of the trailing bytes, on the inside (§659)**: inner nodes within a Merkle
batch did not move the root, so the proof verified with other bytes and the index of applied proofs
of §654 was dodged; the layer and the kit reject them since §659, and the current one was
`arqueo-verify-v0.4.1` (§661), and then `arqueo-verify-v0.4.2` (§671), which also fixes §662 to
§664; then `arqueo-verify-v0.4.3` (§818), which also fixes F3 of 3.15 (§815); today it is
`arqueo-verify-v0.4.4` (§726), which also fixes the eight of 3.18. ⚠️ §917: today it is
`arqueo-verify-v0.5.0` (§917), the first break of the wire (`zkssl/0.5`), with the same fixes; 0.4.4
is still published for the envelopes of `zkssl/0.4`. ⚠️ §818: it said «today it is
`arqueo-verify-v0.4.2` (§671)». ⚠️ §726: it said «today it is `arqueo-verify-v0.4.3` (§818)».
`AUDITORIA.md` §637, §641, §650 to §656.

### 3.10 The custodian and governance keys: of four elements on the wire, of one in the `dev` ledger — ⚠️ BUILT (§683, §684, §701) and DECLARED

Until §684, the identity of a custodian (`derive_custodian_id`, `circuit_threshold.rs`) and
that of a governance member (`derive_governor_id`, `circuit_governance.rs`) were
derived from a key of a **single** Goldilocks element: about 64 bits. The identity is
public, so exhausting a key's space is an offline attack of about 2^63
evaluations (ESTIMATED; with Grover, about 2^32 iterations). Issuance asks for two custodians
(2-of-N), so two keys are needed. §670 declared it, and RFC-0018 E2 closes it in
the format, in three pieces:

- **The wide derivations** (§683): `derive_custodian_id_wide` and `derive_governor_id_wide`,
  the mould of `derive_public_id_wide` of the spending key (entry 15, §82): the domain and
  a four-element key in the same `native_merge`.
- **The wire's threshold** (§684): `circuit_threshold_single_nullifier`, the only authority
  circuit the layer verifies —for the custodians and for governance—, carries the key in
  four columns and binds the four elements to the identity.
- **The layer** (§701): a ledger constituted with wide keys issues, issues to a pending,
  recovers, freezes and rotates its custodians; and a single-element key —the only thing
  the 2^63 search reaches; here, even with the true first element of the wide
  key and the path of its leaf— does not authorise, because its identity is another
  (`una_clave_de_un_solo_elemento_no_autoriza_sobre_un_conjunto_ancho`). Forcing a wide
  key costs about 2^255 evaluations (ESTIMATED, by the size of the `Digest`, like
  RFC-0018; with Grover, about 2^128 iterations, by the same count as the 2^32 above).

What is still worth one element, and is said here:

- **A narrow key is still valid, padded with zeros**: it gives the same identity (§683).
  A ledger constituted with single-element keys keeps its roots and its 64 bits. Its
  custodians can rotate to wide keys with the signature of their governance
  (`un_libro_de_claves_estrechas_rota_sus_custodios_a_claves_anchas`); its governance cannot,
  because it is immutable: a wide governance is given only by a new ledger (RFC-0018, D-B and D-C).
- **The node constitutes a ledger with the roots it is given** (§784, ARQ-02): `--custodian-root`
  and `--governance-root`, those of the ceremony, in the canonical wire encoding and different from
  the zero digest, stated at startup. In a ledger that already exists, the governance one is
  compared and, if it is not its own, the ledger does not open. Without the two flags, with the
  `dev` feature —that of the default build— it opens the layer with the suite's roots, of
  single-element keys written in `crates/zk-ssl/src/tests_support.rs`, and says so; without `dev` it
  does not start. The canon compiles the node without `dev` on every seal. ⚠️ §784: until here it
  said «The node does not constitute ledgers» and that the two flags were still pending.
- **The wire still says `zkssl/0.4`** until the cut of the train (RFC-0018; `spec/RPC.md`,
  §699). The rejection vectors carry the suite's custodian root, and the published ones
  are not rewritten.
- **The other seven circuits that carry an authority key** —`ThresholdAir`, `SingleThresholdAir`,
  `GovernanceAir` and those of issuance, issuance to pending, freeze and recovery of the old path—
  remain with one element. They do not go over the wire: neither the layer nor the kit verifies them
  (the RFC-0009 census), and they do not prove membership of a wide set.

### 3.11 The level of the proofs, and the system's randomness — ⚠️ MEASURED and DECLARED (§697, §698, §708)

**What is measured is classical.** The production options, those of the layer and of the kit, are 42
queries, blowup 16, 21 bits of grinding and quadratic extension (`proof_options()` in
`crates/zk-ssl/src/lib.rs`; the age uses the same, and a test requires it). On the hidden form
of each proof, the `winter-air` function (`proof/security.rs`, upstream's) gives **127 conjectured
bits** —from 112.8 to 115.8 depending on the length with the DEEP term, which its formula does not
discount (below)— and, provable, **59 in UDR** (unique decoding) in all, and in
LDR (list decoding) whatever the trace length gives:

| hidden trace | LDR | conjectured, with the DEEP | proofs |
|---|---|---|---|
| 256 rows | **86** | 115.8 | the two openings of the refund, the threshold and the delegated authorisation (T = 128) |
| 512 rows | **84** | 114.8 | the frozen climb (T = 256) |
| 1,024 rows | **82** | 113.8 | the credit, the climbs of issuance, of issuance to pending and of recovery, the audit and the four fixed-length ones of the kit (T = 512) |
| 2,048 rows | **80** | 112.8 | send, claim and burn (T = 1,024) |

⚠️ **§914** (the zkssl/0.5 cut, RFC-0018 D-H): the first row was the 128-row one, with T = 64, and
its figures were the ones the age table then gave for `m = 3`; the five families that
occupied it move to T = 128, and the age starts at `m = 4`.

**The age drops with `m`.** Its hidden trace has 2^(m+4) rows, with `m` the logarithm of the number
of pendings it covers, and it loses two bits of LDR each time the trace doubles (⚠️ §914: the
statement admits `m` from 4, `zk_ssl_air::M_MIN`, with T = 128; the column of `m = 3` was that of
zkssl/0.4):

| `m` | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 |
|---|---|---|---|---|---|---|---|---|---|---|
| LDR | 86 | 84 | 82 | 80 | 78 | 76 | 74 | 72 | 70 | 68 |
| with the DEEP | 115.8 | 114.8 | 113.8 | 112.8 | 111.8 | 110.8 | 109.8 | 108.8 | 107.8 | 106.8 |

| `m` | 14 | 15 | 16 | 17 | 18 | 19 | 20 | 21 | 22 | 23 | 24 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| LDR | 66 | 64 | 62 | 60 | 58 | 56 | 54 | 52 | 50 | 48 | — |
| with the DEEP | 105.8 | 104.8 | 103.8 | 102.8 | 101.8 | 100.8 | 99.8 | 98.8 | 97.8 | 96.8 | — |

Conjectured 127 and UDR 59 in all; the «with the DEEP» row is the conjectured one counting the DEEP
term (below). With `m = 24`, which the statement admits, the proof cannot be generated today.
⚠️ **§717:** since §717 the statement admits `m` from 3 to 13 (`zk_ssl_air::M_MAX`; ⚠️ §914: from 4
to 13, `M_MIN`), and the judge rejects the rest before reading the proof. The second table, from 14
to 24, remains as what the function gives for lengths the judge no longer accepts, and the `m = 24`
sentence stopped being true. The cap comes from the judge's cost and from the bounds of the RFC-0007
gate, not from the level: with `m = 13` the LDR is 68, and the LDR floor and the UDR threshold
remain undecided. The figures of the second table are no longer fixed by a test: they are bound to
the replica of the function by BINDING F of `tools/check_publicadas.py`.

Tests fix them since §697: in the fixed-length families, conjectured ≥ 127, LDR ≥ 80 and UDR = 59 on
real hidden proofs; in the age, the floor of each `m`. Since §839, each row of the first table has
its proof in that test —the 512-row one was derived, not measured—, and a census requires that every
AIR of the wire, except the age, have one of those lengths. **The level that is declared —the LDR
floor, the UDR threshold and the range of `m`— is decided by the author** (`BACKLOG.md`, entry 116);
until then, what holds are these tables. The conjectured one rests on conjectures about proximity to
Reed-Solomon codes that are not proven; the provable one does not. Both are what the upstream
function returns on the form of the proof, and nobody outside the project has contrasted them with
the fork's hidden construction (H7).

**The conjectured one does not discount the DEEP term (§708).** Its formula, `winter-air`'s, is
min(bits of the field, queries · log2(blowup) + grinding) − 1, with the cap of 128 the hash sets: it
puts as the field term the whole field, 128 bits with the quadratic extension, and no term that
grows with the length L of the hidden trace. The provable one does count one, and it is not about
proximity —what the conjectures deal with— but about degree: that of the out-of-domain point (DEEP),
an error of ((blowup + 1) · (L + 1) + L − 1)/|E|, with |E| the size of the extension, which the
function counts as 2^128, the degree of the constraints bounded by the blowup, and in unique
decoding, without the list size of the LDR. It is worth 116.8 bits with 128 rows and almost one less
each time L doubles —112.8 with 2,048, and 106.8 in the age with `m = 13`, the largest the judge
accepts since §717 (96.8 with `m = 23`, before the cap)—, and, put in place of the field, it is the
smallest of the terms of the formula: the queries give 189 bits and the hash sets 128. With it, the
conjectured one does not exceed that term, and that is the «with the DEEP» figure of the tables: the
term itself, without the −1 with which the formula subtracts a bit from its minimum (its 127 are 128
− 1); with that −1, it would be one bit less. The other field term, ALI's, does not change with L:
with the linear batch of production it is 1/|E|. Those figures are a derivation, not what the
function returns; a test fixes them (`tests_nivel` in `crates/zk-ssl-air/src/lib.rs`), which
contrasts its replica of the formula and of the term with the real function and with its source. The
127 are what the function returns, and what the tests of §697 keep requiring.

The «127 conjectured against 29-63 provable» and the «36.7 KB against 125.6 KB» that several
documents published until §698 are from the comparison circuit of `FIVE_BACKENDS.md` §4, with 32
queries and without grinding or hiding, not from these proofs. What it would cost to bring the
production ones to 128 provable bits is not measured. ⚠️ §806: and those two sizes are not what that
circuit gives today either: re-measured on 2026-10-14 on `b6491ef`, with the test of its table, it
gives 39,048 and 127,274 B.

**Against a quantum adversary: not measured.** That the proof system only uses hashes
means it has no curve assumptions. It does not mean it has a known quantum
level: no figure in this section is one, no parameter was chosen with one in mind, and
that level has not been measured.

**The system's randomness is a trust assumption.** What has to be unpredictable in the
layer, the SDK and the medium comes from the operating system's random generator, in eight calls and
without a common abstraction (seven until §702, which adds the salt of the at-rest KDF):

- the salts of each Merkle leaf and the two hiding seeds of each proof
  (`OsRng`, `crates/zk-ssl-air/src/sal.rs`);
- the spending key of `Wallet::random` and the pending's salt of `random_salt`
  (`rand::thread_rng`, `crates/zk-ssl-sdk/src/lib.rs`);
- the nonces of the at-rest encryption of the ledger and of the keystore (`OsRng`,
  `crates/zk-ssl/src/crypto.rs` and `crates/zk-ssl-sdk/src/keystore.rs`), and the
  salt of the at-rest KDF of both (`OsRng`, `Kdf::nuevo` in `crates/zk-ssl/src/crypto.rs`);
- the randomness of the medium's ML-DSA signature in its salted mode (`getrandom`,
  `crates/zk-ssl-medio/src/nota.rs`).

If that generator is predictable or repeats, the hiding does not protect the witness, the spending
key can be guessed, the nonces can repeat and two encrypted files can share salt. No test checks it,
nor can it. Fiat-Shamir uses no randomness: the coin comes from what the proof commits. ⚠️ §901: the
fresh salt and the non-repeated hiding seeds are two of the named hypotheses of what D-AM of
RFC-0009 leaves written and does not promise (H8 and H9 of `doc/ocultacion-hvzk.md`); measured in
the §898 toy, two proofs with the same row seed return what each one alone hides. ⚠️ §916: since
§916 they are hypotheses of the promise (RFC-0009 D-A), not only of what is written.

### 3.12 The index guardian sometimes started on tmpfs — ⚠️ MEASURED, closed on Linux (§740)

The XMSS index guardian refuses to start where `fsync` does not persist, because a counter that
is lost on shutdown reopens at 0, and with it the leaves already spent look free again. Until
§740 it decided only by timings, with the mean of twenty writes, and it failed open: a single
slow write was enough. Measured in a session's container (`AUDITORIA.md` §740): from 2 to 11
openings of every 2000 started on `/dev/shm`, at rest as under load, and the author's canon
came out ROJO on 08-10-2026 because the witness's co-signer started there once. The only test that
should have exercised the refusal in the guardian always skipped itself with GNU's `df`.

**Closed on Linux** in §740: the guardian reads the file system type of the descriptor in
`/proc/self/mountinfo` and refuses on `tmpfs`, `ramfs`, `devtmpfs` and `rootfs` without measuring
anything, on the folder and on the counter file; and the `fsync` measurement remains as a second
net, with the median. Measured: 8000 of 8000 openings on `/dev/shm` refused by the type, and none of
538,000 started with the median alone. **Residue**: outside Linux there is still only the
measurement; on Linux, a system that is not named like those on the list and does not persist is
only caught by the measurement if `fsync` costs the same as not doing it, and the measurement looks
at the folder, not at a counter that is a mount of its own. And none of this protects from a disk
that confirms writes that are still in a volatile cache, nor from a power cut, which has not been
measured. The guardian is not in the published kit: it carries no advisory nor release.

### 3.13 Opening an account did not check that the identity was new — ⚠️ MEASURED, closed in the layer (§802)

`open_with_id`, through which all the paths that open pass —`zkssl_openAccount`, `dev_openSeeded`,
the narrow and wide openings of the layer—, did not look at whether the identity was new: it probed
the position `public_id[0] mod capacity` and opened in the first gap. Measured on 14-10-2026, in
private and with four tests outside the tree (`AUDITORIA.md` §802): a third party **without the
key** opened a second account with Bob's identity and with its own view and its own salt (`Ok`,
index 700774416 against Bob's 700774415); after a recovery to a new key, whoever kept the **old**
key —the thief, if the recovery was for theft— opened another account with the retired identity and
claimed there a pending of 7,000 that was in flight towards it; the new key does not claim it in the
recovered account (`StaleState`); and the same pending is not claimed twice between duplicates (the
second `apply_claim` gives `StaleState`, and the supply of 500,000 adds up). It creates no money:
what it broke is whom it reaches.

**Scope.** Over the wire, a third party without the key occupied another's identity and a slot of
the account cap, without being able to claim anything in it. The theft of a pending in flight
required a recovery, and recovery is only in the library and in its tests: no node reached that
case.

**Closed** in §802, in the layer, with two rules: an identity —the whole `public_id`, its four
elements— has at most one account in the life of the ledger, and the one a recovery retires never
has one again. Opening rejects both with `IdentityUnavailable` (`-32000` on the wire, AM05 in
ISO 20022) before mutating anything; recovering to an identity with an account or retired —also
undoing X→Y→X— is rejected before verifying, without spending quota, counter or nonce. The
retirements are written in the same batch as the recovery (`retir:`, with its root `root:retir`),
sealed if the ledger is encrypted, and are required on reopening: a ledger does not open if any is
missing or extra, and an encrypted one neither if they are replaced; the snapshot moves to v9 and
carries them; the KDF migration seals them again, and that of B13/B14 does not touch them. Of the
twenty-two new tests, twenty-one falsify it, each against its mutant, and one fixes the measurement
of D-5 of `doc/ROTACION_DE_CUENTAS.md`. Until §802, a repeated seed in `dev_openSeeded` or in the
CLI's sandbox opened, without error, another account with the same identity; both move to the path
that returns the rejection, because with the `expect` of `open_account_wide` the new gate would be a
panic —in the node, with the lock taken, and the PARADA, which already gave `AccountLimitReached`
with the ledger full—.

**Residue**:

- the duplicate accounts already in a ledger are still there: the ledger opens and the index
  collapses them. And the worst of them: if the duplicate B(X) was opened by whoever had X's key and
  afterwards the legitimate account X→Y is recovered, B keeps X and claims whatever is in flight
  towards X —the theft from before §802, with a duplicate from before the fix—. Measured in private:
  the recovery retires X, and even so the claim in B gives `Ok` and its balance stays at 7,000. Only
  a ledger with duplicates and recoveries reaches it, and recovery is not on the wire;
- the integrity of the retirements at rest is the ledger's: in an encrypted one, a retirement
  deleted, added or replaced does not open; in one in the clear, whoever writes on the disk redoes
  `root:retir` —the chain is `native_merge`, public— and a replaced retirement opens (measured: the
  retired identity opens an account again). It is what they can already do with the rest of a ledger
  in the clear, and the retirements do not go under the head, so no witness sees it;
- with an earlier copy of the ledger and access to the disk, the later rotated leaf pasted over the
  earlier log opens, encrypted or in the clear: `load` requires the retirements against the log, but
  does not cross `root:state` with the last entry that moves the account tree. Measured in private:
  the ledger reopens with zero recoveries and the retired identity opens again. It is the same as a
  whole rewind, which undoes the whole recovery and returns the account to the old key; the
  cross-check remains apart;
- a v9 snapshot in the clear carries the retirements without any root the receiver can check:
  `SnapshotInfo` does not bring it, and a changed tail imports (measured: the retired identity opens
  again in the imported layer). Only the encrypted one protects them (⚠️ §817: and, until §817, not
  even it: with its passphrase, the import also accepted one in the clear, without looking at the
  key; §3.16);
- a ledger with recoveries written before §802 does not open: without its retirements it looks the
  same as one whose retirements were deleted at rest, and opening it empty would bring back the
  theft. Only the library makes it, because recovery is not on the wire; and likewise a v3..v8
  snapshot with recoveries. It is what the author chose over opening them empty (D-1 of §802);
- early occupation gets worse: whoever knows an identity before its owner opens the account occupies
  it with their view and their salt, and the owner has to change key. And it blocks a recovery:
  whoever knows the new identity before it is applied occupies it, and the legitimate recovery comes
  out with `IdentityUnavailable`; today without consequences, because recovery is not on the wire.
  Closing it requires proving possession of the key on opening, a circuit and a wire change: the
  author left it as a declared residue and takes it to the `zkssl/0.5` cut, in entry 119 of
  `BACKLOG.md` (D-2 of §802);
- the cause has no portable proof: «has an account» would require opening the leaf, and the
  retirements do not go under the head;
- a single cause does not say which of the two rules fired, although walking the probe chain with
  `zkssl_publicId` lets it be inferred in the common case (not for the new identity of a recovered
  account, which stays in the probe chain of the old one);
- a pending towards a retired identity only returns to the sender by refund, on expiry, and the
  refund is not on the wire either (D-5 of `doc/ROTACION_DE_CUENTAS.md`). ⚠️ §860: D-5, decided by
  delegation (entry 52 of `BACKLOG.md`): they are stranded until that refund. And DEDUCED, without
  test: what an account sent through v2 before it was recovered no longer returns to it by refund
  —`apply_refund` requires the sender's account to keep the identity of the commitment, and gives
  `PendingMismatch` (`crates/zk-ssl/src/two_phase.rs:710-714`)—: it only gets out if the receiver
  claims it. It fails closed: it neither creates nor moves money.

It was drafted in private and is published at once with the code, as section 5 asks. There is no
deployed ledger and no third parties to notify: the notice is this section and entry §802, not a
GHSA. The published kit does not change: it does not depend on the layer, and a rejection envelope
with this cause is rejected by its name. `AUDITORIA.md` §802. ⚠️ §863: and a vector with a new path
shows it, as R6 f of RFC-0021 asks of a fix that changes a verdict within `0.N.x`:
`spec/vectors/nodo/abrir-identidad-con-cuenta.json`, opening over the wire an identity that already
has an account, which the canon judges with a node test (entry 90 of `BACKLOG.md`).

### 3.14 The encrypted ledger migrated from version 1 kept in its files what was sealed with the v1 key — ⚠️ MEASURED, closed in the library (§812); its race, measured and closed in §816

Since §702 (RFC-0001, E2) an encrypted ledger of version 1 —key `SHA-256(dominio_v1 ‖ frase)`,
without salt or cost— is migrated to 2 on opening it with its passphrase. The migration overwrote
each value in a batch on the same `sled` and did `flush`: what is read changed whole or not at all,
but sled is a log, and the overwritten values remain in the files of the directory until sled reuses
their segment, which may never happen. Measured in §812 on sled 0.34.7, the one of the `Cargo.lock`,
searching for the bytes of each sealed value in the ledger's files: with a ledger of two accounts,
its 25 values sealed with the v1 key remained, all 25, in the files after the batch, after 200 more
batches and after three reopenings (on the code from before §802; since it, the same ledger carries
26, with `root:retir`, and the batch in place leaves the 26); with one of 702 accounts, 1425 of 1425
after the batch, 1356 after 200 more batches, and none after three reopenings, when sled redid its
segments (whether it overwrote them or returned them as free blocks to the file system was not
measured). With the one of two accounts the 25 remained after three reopenings: one cannot count on
reopening cleaning up. How long the rest lasts depends on what the ledger writes afterwards: in a
small or quiet one, it does not go away. The §702 test looked at what sled returns, not at the
files, and no document said so.

**The attack.** Whoever takes the disk —or a copy of the directory— of a migrated ledger tries
passphrases against that residue at the speed of a SHA-256, about 159 ns per attempt (§702), instead
of an Argon2id of 64 MiB; the XChaCha20-Poly1305 tag confirms the right one, and that passphrase
also opens version 2, whose salt is in the clear. The hardening of RFC-0001 did not protect those
disks. **The window**: the ledgers the code migrated from `2ac6db4` (§702, 07-10-2026) until §812.

**What closes it** (§812, RFC-0001 E4, drafted in private with this fix): the migration no longer
writes in place. It copies what is live, sealed with the new key, to a sibling directory
(`<libro>.recifrando`), verifies it in the clear by reading it from the disk, renames it as the
ledger and deletes the old one whole. A death in the middle of the migration is resolved by the next
opening, with the same passphrase, leaving nothing of the v1 in the ledger's directory
(`una_muerte_en_la_migracion_v1_no_deja_en_el_directorio_lo_sellado_con_v1`, with a death at the end
of each step). The same piece is `SovereignLayer::recifrar`, which changes the passphrase and the
salt of the whole ledger; if a re-encryption dies before its commit, the opening rolls back and the
ledger stays whole with the old passphrase, with what was sealed with it: the re-encryption has to
be repeated; and if it fails with an error past the commit, the `.retirado`, if it is still on the
disk, keeps what was sealed with the old one until the next opening, which deletes it (⚠️ §816:
until §816, only if nobody opened the ledger at the same time; below, «The race of E4»). The retired
identities of §3.13 (`retir:{k}` and `root:retir`) go sealed, and are copied and verified like the
rest (`recifrar_conserva_las_identidades_retiradas`). Measured with mutants: if the migration
becomes again a batch in place, `la_migracion_v1_no_deja_en_los_ficheros_lo_sellado_con_v1` falls
with 26 of 26 v1 values in the files (25 of 25 before §802); with the new piece none remains in the
files of the ledger or of its siblings, also in the one of 702 accounts.

**The remedy for a ledger already migrated**: `SovereignLayer::recifrar` **with a new passphrase**.
The library does not distinguish a migrated ledger —that migration only wrote `meta:kdf`—, so it is
by hand. It leaves the directory clean of what was sealed with the v1 and with the old passphrase
(`un_libro_migrado_por_el_702_se_limpia_recifrandolo`). With the same passphrase the directory also
ends up clean, but the copies made since the migration and the free blocks keep giving the v1
passphrase at the speed of a hash, and that passphrase opens the ledger.

**⚠️ §816: the race of E4, and what closes it.** Measured in private on `b6994c3` —§812, already in
`main`— by a reviewer, with two threads and pauses injected into the library: the opening looked at
the siblings —each one's sled lock, taken and released on the spot; without siblings, none— and then
acted and called `sled::open`, which creates the ledger if it does not exist. Nothing held a lock
between looking and acting. (a) Without siblings, an opening that looked and paused, and a
re-encryption that arrived in between at its commit —`rename(P, V)`—, left an EMPTY ledger in the
ledger's place, which the opening opened `Ok` without balance; step 6 of the re-encryption failed
with «the next opening finishes it», and the next one refused: the ledger, whole in its
`.recifrando` and in its `.retirado`, did not open again without a person. (b) With the ledger and
its `.retirado` from a death after step 6, the opening deleted the NEW `.retirado` of the
re-encryption and created an empty ledger, and the next opening with the passphrase the error named
deleted the `.recifrando`, the last copy. (c) With the ledger and its `.recifrando` from a death
before the commit —measured by a reviewer of the fix, also on `b6994c3`—, the opening deleted the
NEW `.recifrando` and created an empty ledger; step 6 failed saying that the ledger was left with
the NEW passphrase, and it was not; and the next opening, with any passphrase, deleted the
`.retirado` —the only copy— and with the old one opened `Ok` an empty ledger. In all three, the
opening that paused opened `Ok` an empty ledger, **without any error**. In (b) and in (c) **the
ledger, with its balances, was lost whole**, and both said it equally badly: the error of the
re-encryption named the new passphrase, with which the ledger no longer opened, and with the
previous one the opening opened `Ok` an empty ledger, without any error. It creates no balance nor
authorises any spend. **Scope**: whoever uses the library with an encrypted ledger and two openings
at once, one of them a re-encryption or a migration from version 1 —opening a v1 is migrating it—;
the node and the cli open in the clear and never re-encrypt, and the kit does not depend on the
layer. **The window**: since §812. **What closes it** (§816): a lock on something that is not
renamed, `<libro>.cerrojo`, a file with an exclusive `flock` that the layer creates —empty; on Unix,
with 0600— and never deletes. Every opening holds it from before looking at the siblings until it
returns with the ledger open —and, if it was v1, migrated—, and `recifrar`, from before looking at
the siblings until the end —after the Argon2id of the new passphrase, before touching the disk—;
whoever arrives waits for it as for sled's, ten times every 50 ms, and if it cannot take it gives
`Io` with its name, without touching anything. Nothing is created through a link: if it is not
there, it is created with `create_new`, which does not follow a link in the last name; one that
appears between looking and opening is followed, without creating or writing anything at its
destination although it takes the `flock` for an instant, and with the `flock` taken it is checked
that the name is still that file, by its device and its inode, and it refuses if it is not. Whoever
holds it looks at it again before what is not undone —the opening's `sled::open` and the
re-encryption's commit—, and if someone deleted or changed it, refuses with `Malformed` without
touching anything. The opening creates the ledger's directory first if it is not there, as
`sled::open` created it; `recifrar`, not, nor the `.cerrojo` where there is no ledger nor siblings.
Through a path without a name —one ending in `..`, like `<libro>/blobs/..`, which can name the
ledger— nothing is opened or re-encrypted: without a name there is no lock. The lock is not `Send`:
it does not cross threads. With the lock taken, the opening does not create a ledger where there is
a `.retirado` or a `.recifrando` without a ledger. Measured with the same pauses, in tests of the
suite: the three races —the opening opens the whole ledger, the re-encryption gives the lock's `Io`
without touching anything, and afterwards no opening creates an empty ledger or deletes the only
copy, and the balance remains—, two re-encryptions at once, an opening that arrives while another
re-encrypts or migrates, the lock, which no step renames or deletes, re-entry in the thread, which
does not remain after returning, a new ledger in a directory that does not exist, a lock that
changes while being taken, one that is deleted or changed while held, a path without a name, a
re-encryption without a ledger, which does not leave the lock, and the lock, which does not cross
threads; each piece falls with its mutant, and those of the race fall on `b6994c3`. With two real
processes a reviewer measured it on the first fix of §816, outside the suite, and the session with
its same tests on the fix of each round, also the last one: they exclude each other in both
directions, and one killed with SIGKILL with the lock taken does not leave the ledger locked. Why a
sibling file and not a `flock` on the parent directory is stated by D-J of RFC-0001. **Residue**:

- a process with the code from before §816 on the same ledger at the same time does not take the
  lock, and with it the race continues;
- deleting or replacing the `.cerrojo` by hand —restoring a copy, an `rsync`— while someone holds it
  lets another in, who creates or takes a new one: on the first fix of §816, a reviewer measured
  that with an opening paused before `sled::open` it was race (a) again. Since the fix, whoever held
  it notices before `sled::open` and before the commit, and refuses (there is a test); what remains
  is the window between that check and the call that follows, and outside Unix it is only checked
  that it is still a file. With a re-encryption already past its commit, the sled lock of its
  `.retirado` keeps excluding (measured by the reviewer);
- whoever opens through a link to the ledger's directory takes the lock next to the link;
  re-encrypting and migrating through a link are refused, and in the gap of a re-encryption, between
  steps 5 and 6, `sled::open` through the broken link gives «File exists» and creates nothing
  (measured by a reviewer of the fix);
- a ledger whose parent directory cannot be written and without its `.cerrojo` no longer opens,
  until it is created by hand, empty and for whoever opens it; if the parent is on a read-only file
  system —a ledger that is a mount point inside a read-only one—, the `.cerrojo` that is already
  there is opened read-only, because there nobody can change it, and if it is not there it has to be
  created first, where it can be written (measured outside the suite, with the cli and a read-only
  mount);
- a `.cerrojo` that whoever opens cannot open for writing —of another user and with 0600, like the
  one `root` leaves if it opens with the cli, before the node, the node's ledger that did not yet
  have its own— leaves the ledger unopened: `Io` with its name, which says the remedy: with the
  ledger closed by everyone, whoever can gives it to them (`chown`) or deletes it, and the next
  opening creates it. Until §816 it opened. Opening it read-only would not fix it: with 0600,
  another user cannot read it either (measured outside the suite, without `CAP_DAC_OVERRIDE` nor
  `CAP_DAC_READ_SEARCH`);
- through a path without a name —`.`, `..`, or one ending in `..`— no ledger is opened any more
  (`Malformed`); until §816 whatever it named was opened;
- it is not tested on Windows nor on NFS, where the client emulates `flock` with `fcntl`.

**What the library does not control, and is not promised.** Deleting is deleting, not overwriting.
There is no overwriting before deleting: the author decided it (§812), on the session's
recommendation —it cannot be measured here, it is no good on an SSD, on a copy-on-write system or
with the journal, and it would invite promising—. It is deleted, and what may still keep copies is
stated:

- **the blocks the file system frees** on deleting the old directory; and the same blocks of other
  remains: the inode of the v1 keystore that E3 rewrites with a `rename` (deduced, without
  measuring: on the session's machine the device could not be read), the segments sled truncates,
  the previous snapshot, which `export_snapshot` replaces whole with a rename (⚠️ §820: it said
  «which `export_snapshot` truncates in place (entry 120 of `BACKLOG.md`)»; entry 120 is closed by
  §820, and the blocks of the previous one are freed just the same. A dead export leaves next to the
  destination `{instantánea}.exportando`, whole or half done, with what the destination would carry
  —sealed with the key the layer had, or in the clear if it had no key; a whole one from before a
  `recifrar` opens with the old passphrase (deduced, without measuring), and a half-done one does
  not import—; it is not swept: the next export to that destination refuses naming it, and it is
  deleted by hand. The new one takes from the previous one, with privileges, its owner, and its
  group and its mode bits, but not its POSIX ACLs nor its other extended attributes: with an
  extended ACL, the group bits of the mode are its mask, and the new one gives them to the owning
  group, which thus reads a snapshot —in the clear, if the layer has no key— that before it did not
  read), and the `{instantánea}.tmp-descifrada` that the import wrote IN THE CLEAR and deleted,
  between `5d2833e` (28-07-2026) and `c87d7d6` (§774, 08-10-2026);
- **a whole `.tmp-descifrada`, in the clear**, if a process died in the middle of an import in that
  window: §774 removed the writer and did not sweep. Since §812, by the author's decision, the
  import of that same snapshot sweeps it: if next to it there is a file with that name, it deletes
  it (`la_importacion_barre_el_claro_que_dejaba_una_importacion_muerta`) and says so on the error
  output. It is the session's, reversible, that if it cannot delete it the import fails with an
  error that names it, without importing anything, and that it only deletes a regular file: a
  directory or a link with that name are not touched. ⚠️ §819: it said «Neither the refusal nor the
  warning has a test». Both have one since §819
  (`si_no_puede_barrer_el_claro_la_importacion_falla_sin_importar_nada`): the refusal, with the
  deletion itself made to fail, and the warning, read in what the thread keeps in the tests; that it
  goes to the error output the test does not see: outside the tests it is a one-line `eprintln!`,
  without test, and it is declared; leaving it so is the session's, reversible. The one of a
  snapshot that is not imported again is looked for next to it and deleted by hand; and deleting it
  does not overwrite either;
- **the imported layer** of an encrypted snapshot is born without a key and, if exported again,
  comes out in the clear (entry 121 of `BACKLOG.md`; ⚠️ §821: no longer: since §821, by the author's
  decision, the layer `import_snapshot_with_key` returns keeps the snapshot's key, fixed to its
  header, and is re-exported encrypted, with the same passphrase; and by the session's decision,
  reversible, that of a v1, in version 2, with a new salt; entry 121 is closed by §821, and 3.16
  says so. What a layer without a key exports, like that of `import_snapshot`, is still in the
  clear. And the key it keeps is the snapshot's, not the ledger's: a copy from before a `recifrar`,
  imported with the old passphrase and re-exported, comes out encrypted with the old passphrase and
  with the header of that copy, not with the re-encrypted ledger's, and does not open with the new
  one (measured in the session, with a test that does not stay in the tree: the §821 entry). Each
  re-export of a copy restored with the old passphrase is a NEW copy under that passphrase, with
  whatever the imported layer carries then, and whoever changed the passphrase because it leaked has
  to count it among the copies that keep the old one);
- **the hardware and the file system**: an SSD (its FTL, wear levelling, TRIM), the copy-on-write or
  log-structured systems (btrfs, ZFS, APFS, F2FS) and their snapshots, those of LVM, of a virtual
  machine or of the cloud, and the journal with `data=journal`;
- **the copies**: of the directory, the snapshots exported before and a hard link made outside
  keep opening with the old passphrase; those of a migrated ledger also carry the v1 residue;
- **the keystore**: changing its passphrase (`load` and `save`) does not change the spending key,
  and an old copy opened with its old passphrase gives the same key;
- **memory**: swap, hibernation and dumps.

**The two names are reserved**: there is no mark that distinguishes a sibling left by the
re-encryption from a directory someone named that way, and the opening DELETES a sled
`<libro>.recifrando` or `<libro>.retirado` it finds next to the ledger and that nobody has open,
also a backup copy with that name. What is not sled's it does not touch. ⚠️ §816: and a third,
`<libro>.cerrojo`, the ledger's lock, which the layer never deletes; one that is not a file —a
directory, a link— makes the opening refuse.

The node opens its ledger in the clear (§3.3): it has no passphrase to change nor v1 residue to
clean, and wiring its encryption stays out. But its opening is the layer's, and it also resolves the
two reserved names next to its ledger, with the same deletion and the same refusal: a `.recifrando`
or a sled `.retirado` next to the node's ledger are resolved at startup like any opening —it deletes
them, or, if the ledger is missing and its `.retirado` is there, puts the `.recifrando` in its
place—, and a state the re-encryption does not leave prevents it from starting. ⚠️ §816: and it
takes the ledger's lock like any opening: at startup, the node creates `<libro>.cerrojo` next to its
ledger if it is not there —and, on its first startup on a new path, the ledger's directory, as
before §816—.

It was drafted in private and is published at once with the code, as section 5 asks. The notice is
this section and entry §812, without a GHSA, by the author's decision: the published kit is not
affected —it does not depend on the layer—, so it carries no advisory nor release. A marker to
distinguish the ledgers already migrated is not added; that is the session's, and the author has not
decided it: it would be a new format and would suggest that that is enough; it is reversible.
`AUDITORIA.md` §812. ⚠️ §816: the fix of the race was drafted in private, as section 5 asks, and is
published like §812, by the author's decision: at once with the code, and the notice is this section
and entry §816, without GHSA or release, because the published kit does not depend on the layer. The
chosen lock is the session's, and reversible while RFC-0001 remains PROPOSED. `AUDITORIA.md` §816.

### 3.15 The reception receipt named an honest operator in ROJO, and in PARADA the node kept signing — ⚠️ MEASURED, closed in the node and in kit 0.4.3 (§815)

Three failures of the same family: the reception receipt of RFC-0010, which exists so that an
operator that censors is named, also named the one that does not censor, and the PARADA of §655,
which exists so that memory does not sign what the disk does not have, did not halt the one that
signs. The first was deduced, reading the order in which the node records and applies, by whoever
was preparing the decision on entry 19 of `BACKLOG.md` (no write-ahead log); the other two came out
while designing and judging its fix, and the third was already noted by reading in the draft
`doc/ecst/ECST.md` §8.2. The three were measured in private, in a clone without a remote, on
`4d8017d` (§813), with real nodes that sign, injecting the death and the full disk with `strace`.
The code of `crates/` does not change between `4d8017d` and `ff9d4fc` (§838; `git diff --stat`), so
what was measured is today's. The logs of those measurements stay outside the tree, in the private
clones, and are not published; entry §815 says what each one measured and with what.

**F1 — a crash between recording and applying left a leaf without verdict.** Since §569 the node
reserved the `rx`, RECORDED the leaf with `fsync` and only afterwards the layer judged. Measured:
with the process dead between recording and applying —an abort injected after recording, and a
SIGKILL injected by `strace`—, the leaf stays on the disk; on restart, the log and the counter are
«in step», without warning; the first signed head attests it (`recepCount` 1) and `zkssl_recepPath`
serves it; the operation is not in the ledger. The holder received a 502 and no receipt, and even so
recovered it with `zkssl_recepPath` and the kit (33 calls, 1 hit). If they do not resend, once the
window is over —1,440 heads; 2,875 s in the measurement— the completeness envelope comes out ROJO,
«NO RESUELTA EN LA VENTANA», against an operator that did not lie; if they resend when the state has
already moved, `StaleState`. Seen from outside it is the same signed state the liar of §602
fabricates by restoring the ledger. A panic of the layer or a store failure after recording left the
same (deduced from reading the order; in the measurement of F2, below, the `rx` of the send that
halted the node remained attested, `recepCount` 3, and its operation was not in the ledger on
restart). **Window**: in `applySend` and `applyClaim` since §569, §570 and §573 (28-09-2026: the
leaf, the signed leaf and the kit's verdict); in `applyMany` and `zkssl_pledge`, since §611
(30-09-2026); until §815.

**F2 — in PARADA, the heartbeat kept signing.** Since §655 a store failure puts the node in PARADA,
and the dispatch stops serving; but `latido::latir` did not look at the PARADA. Measured with the
disk full (`ENOSPC` injected into the ledger) during a send: the response is the PARADA, the node
rejects every request, and the heartbeat signs heads from memory, which carries the send the disk
does not have: heads of `seq` 9 with the disk at `seq` 8, ten in 20 s in one run and five in 10 s in
another. After restarting, the ledger reopens at `seq` 8, and in the second run the node signs again
a `seq` 9, that of another operation: two signed heads at the same height with different histories
against an honest operator (deduced from the operation being another; the measurement did not
compare their `chainDigest`). The witness, `--auditar` and `--ausentes` gave no finding. And
`zkssl_publishConsumo` returned the store failure inside a result (`accepted: false`, cause
`Store`), without PARADA: measured with the binary from before the fix, the node kept serving and
signed 6 heads in 10 s. **Window**: the node signs its heads since §241-§243 (09-08-2026); until
§655 (01-10-2026) a store failure did not even halt the dispatch, and since it 3.9 took it as
closed; `zkssl_publishConsumo`, since §417 (06-09-2026). Until §815.

**F3 — a rejection for an invalid proof had no resolution.** A third party **with no account**, with
the roots of `zkssl_epochHead` and the limit of `zkssl_params` —public reads— and 32 bytes `0xab`
per proof, sends a send. The node rejects it with `VerificationFailed` («prueba mal formada»), gives
it its receipt in `error.data.recepcion` and leaves the leaf under the signed `recepRoot`; the
claim, the same. The honest operator had no resolution the kit accepted: the rejection envelope
comes out ROJO, «la causa VerificationFailed no la prueba este mando», and the declared one too, «no
es de las que el RFC-0007 dejo sin prueba portable», although D-I of RFC-0007 declares them so and
D-G of RFC-0010 sends them to the fourth state. Once the window is over —a current head of index
1444, in 48 minutes—, the envelope without resolution comes out «NO RESUELTA EN LA VENTANA»: the
operator is named by the garbage of a third party. Only by lying about the cause —declaring
`PendingTreeExhausted`— did it drop to 3. Measured end to end. An honest client from another point
of history whose proof does not verify (RFC-0021, fact 1) left the same (deduced). `ProofFailed`
goes with it because D-I declares it the same; by reading, the layer only emits it when proving, not
when judging what arrives. **Window**: in the tree since §573 (28-09-2026), and in the batch since
§612; in the kit releases that read the completeness envelope, 0.3.0, 0.4.0, 0.4.1 and 0.4.2, which
carry the same list of four causes (read in their commits; measured with 0.4.2); 0.2.0 does not read
it.

**Scope.** None of the three creates balance, spends without authorisation or makes the kit accept
an invalid transition: they are not soundness failures in the sense of section 5. What they broke is
the evidence against the operator. F1 and F3 gave a signed ROJO, «NO RESUELTA EN LA VENTANA»,
against an operator that did not censor: F1 with a crash at just the right instant, and F3 at the
will of anyone, with no account and with public reads, for each garbage send or claim they send. F2
signed heads of a state the disk does not have, and after restarting the node signed another at the
same height with another operation: two incompatible signed heads for the same `seq`, from an honest
operator (deduced: the measurement did not compare their `chainDigest`). F2 reaches the node that
signs its heads; the ROJO of F1 and F3 is given by every kit that reads the completeness envelope of
§573.

**What closes it** (§815), by the author's decision:

- **F1, in the node: judge first and record afterwards** (RFC-0010, D-E amended). In the four arms
  with a receipt, the node reserves the `rx` and fixes the era on receiving, the layer judges, and
  the leaf is recorded with `fsync` after the verdict and before responding. A cut-off evaluation
  leaves a gap, which promises nothing; with `--ledger`, the leaf of an applied operation is born
  after the `flush` of its entry (without it, no: below, the residue). A store failure leaves no
  leaf and halts the node. If the log fails after the verdict, the request receives its verdict
  without a receipt and the node halts: decision (a) of §569 is reverted, with its reason in the
  RFC. An in-flight failure flag, written with the state lock taken, makes the request that was
  waiting for that lock not be evaluated (the race, measured with real threads by one of the §815
  reviews, with its test in the seal). And a trace line with the method, the `rx` and the
  `hashPrueba` —without the parameters— comes out before the layer judges.
- **F2, in the node**: `latir` neither composes nor signs with the PARADA set, nor with the
  in-flight failure flag, and spends no XMSS index; `zkssl_publishConsumo` returns the store failure
  as an error, and the node halts.
- **F3, in kit 0.4.3**: `ProofFailed` and `VerificationFailed` are declared —exit 3, «DECLARADA, NO
  PROBADA»— on the direct path and in the batch. Their rejection envelope remains unproven. Three
  new vectors in `spec/vectors/completitud/` fix it, and no published vector moves. `doc/KIT.md` §0
  says which verdicts change against the published 0.4.2 (RFC-0021, R6 f).

**Measured afterwards**, with the node and the kit of §815 and, as falsifiers, those from before:

- with a SIGKILL after the judgement and before recording, the `rx` is left as a gap, the startup
  warns of it, `zkssl_recepPath` reports it as a gap, and the payment is in the ledger, with its
  acknowledgement;
- with a SIGKILL after the leaf's `fsync`, the leaf is there and so is the payment, and the envelope
  resolved with its acknowledgement comes out VERDE; with the node from before and the same
  injection, the leaf is there, the payment is not, and the same envelope comes out ROJO;
- in PARADA from a send with the disk full, the journal gains no head in 10 s and the signing index
  does not move, and after restarting no height has two different signed heads; in PARADA from a
  consumption, no head in 10 s and the index still, against the 6 heads of the node from before
  (without restarting: the two heads at the same height were only measured with the send);
- the envelopes of the F3 measurement, with kit 0.4.3: the declared one with `VerificationFailed`
  comes out 3, and the rejection envelope, 1, as with 0.4.2: still unproven. On the 500 vectors of
  `spec/vectors/`, against the tree's kit without the fix, only the three new ones change.

**Residue**:

- **`ProofFailed` and `VerificationFailed` remain unproven.** That the proof did not verify is the
  node's word until the kit can re-verify it against what the leaf signs: with the `hashPrueba` and
  the `digest_pi` under the signature, whoever has the proof presents it and the rejection is proven
  or refuted. It belongs to the `zkssl/0.5` train (RFC-0019 E3), and so the author decided.
- **Nothing binds the declared one.** Measured in private on `4d8017d`: the kit gives 3 to a
  declared one with a cause from the list that is beside the point —`NotTheAccountHolder` on a send—
  and even without a current head; the only thing that binds it to the receipt is the `hashPrueba`,
  which the operator knows. That is why adding two causes opens no door: any ROJO already dropped to
  3 by declaring one of the four. Exit 3 is the operator's word, and nothing more (RFC-0010, D-G).
  `ProofFailed`, moreover, is not produced today by any arm with a receipt —the layer only gives it
  when proving; read in the code—, so a declared one with it is always that word. And in the batch
  the kit does not cross the declared cause with the composition: `lote-declarada` (published since
  §614) declares `NotTheIssuer` on a composition that repeats the account, which would refute it on
  its own. Entry 129 of `BACKLOG.md`.
- **The rejection nobody keeps** (deduced, without measuring). The node does not keep the `data` of
  a rejection. If it dies between the `fsync` of a rejection's leaf and the response, if the
  response is lost or if the holder keeps quiet about it, the honest operator does not have the
  envelope to show, and a cause with proof is not declared: once the window is over, ROJO. It
  predates §815, and §815 neither opens nor closes it. Entry 129.
- **Without `--ledger`, the leaf without its operation** (deduced from reading the code, without
  measuring; A8 of two of the reviews that judged the order). A node that signs only needs
  `--diario` together with `--clave` (§285); without `--ledger` the ledger lives in memory and its
  `commit` writes nothing, while the counter and the reception log always go to disk. After a
  restart, the leaves of the applied operations are still under a signed `recepRoot` and their
  operations are not in the ledger: the state of F1, and the same ROJO when the window expires. It
  predates §815, which does not make it worse; requiring `--ledger` of whoever signs stays outside
  this seal.
- **More gaps.** Every cut-off evaluation now leaves a gap, which the startup declares. The holder
  without a response does not know whether their operation was applied: **they look for their
  `hashPrueba` among the `proofDigest` of `zkssl_logEntries` before paying again** (`spec/RPC.md`).
  In `applyMany` and `zkssl_pledge` the PARADA of a store failure does not carry its `rx` in the
  message, unlike in `applySend` and `applyClaim` (read in the code).
- **The trace costs bytes.** The line «recepcion: la capa juzga» comes out for each reception
  judged, also for the garbage of a third party with no account; with the leaf and the dispatch's
  error line, it adds log per request and brings a full disk closer, which leads to the PARADA
  (deduced, without measuring; the cost per request already existed, §253, and the log's destination
  belongs to the deployment).
- **Unmeasured**: the latency of `applySend` before and after (the same two writes, in another
  order: deduced); the death between the `fsync` of a rejection's leaf and the response; the witness
  against a node in PARADA; and the window of 1,440 heads end to end with the §815 node.

**What should be done.** A ROJO «NO RESUELTA EN LA VENTANA» obtained with 0.3.0, 0.4.0, 0.4.1 or
0.4.2 on a receipt the node rejected for `VerificationFailed` or `ProofFailed` does not accuse
anyone: it is judged again with 0.4.3 and the `declarada` resolution, which comes out 3. A ROJO of
that class on a receipt from the F1 window —in `applySend` and `applyClaim` since §569, in
`applyMany` and `zkssl_pledge` since §611, until §815— may be an honest crash between recording and
applying, which from outside is indistinguishable from the liar of §602: on its own it does not
prove censorship. Whoever signs heads should start the node with `--ledger`, which persists the
ledger.

It was drafted in private and is published at once with its fix, as section 5 asks and like 3.14.
Unlike 3.13 and 3.14, the published kit IS affected (F3): with this seal come out, as section 5 and
`spec/rfc/PROCESO.md` §«Fallos de seguridad» ask, the release `arqueo-verify-v0.4.3` and its
security advisory, which names the affected ones —0.3.0, 0.4.0, 0.4.1 and 0.4.2— and the first that
fixes it, 0.4.3. The author publishes them: ~~until the entry that registers the release, the
current one remains 0.4.2~~ (`doc/KIT.md` §0). ⚠️ §818: he published them after the seal: the
release `arqueo-verify-v0.4.3`, on `5fd5212`, and the advisory `GHSA-6p4m-qg2w-8r6j`, of medium
severity (CVSS 4.3)
(<https://github.com/atoranzo/Arqueo-open-conservation-proofs-for-closed-ledgers/security/advisories/GHSA-6p4m-qg2w-8r6j>),
with those affected and that one fixed; §818 registers the release with its fingerprint, and the
current one is 0.4.3. ⚠️ §726: it was until §726; the current one is 0.4.4, which fixes the eight of
3.18. ⚠️ §917: it was until §917; the current one is 0.5.0 (`zkssl/0.5`), and 0.4.4 is still
published for the envelopes of `zkssl/0.4`. F1 and F2 are the node's, which has no release: their
notice is this section and the entry. `AUDITORIA.md` §815.

### 3.16 The import with a key accepted a snapshot in the clear — ⚠️ MEASURED, closed in the library (§817)

`SovereignLayer::import_snapshot_with_key`, which imports a snapshot with the ledger's passphrase,
also accepted one with the mark `0x00`, in the clear: it imported it without error or warning,
without looking at the key. It was noted, by reading the code, by a reviewer of the snapshot seals
the session was preparing, and it was measured in private, in a clone without a remote, on §816: an
encrypted layer exports its snapshot; in its place one in the clear is put, made with the library,
with another balance; and with the right passphrase the import gives Ok, with the balance of whoever
put it there. The test of §817 measures it, and with the code from before it fails on that very
point.

**Scope.** Whoever can write where the encrypted copy is kept —a backup disk, a shared store— and
whoever restores it with their passphrase: they receive Ok and a layer with the accounts, the
balances, the frozen and the retired ones that whoever put it there chose, and also with their
custodians and their governance —the roots of the two sets, which decide who issues, freezes,
recovers and changes the custodians— and their caps —the regulatory limit, the maximum supply and
the maximum number of accounts—, and nothing says so. The import takes those sets and those caps
from the file without comparing them with anything (`crates/zk-ssl/src/snapshot.rs:391-392` and
`:413-415`). The authenticated encryption of the snapshot (§702) did not protect what is restored,
because whether it is authenticated or not was decided by the mark, which the file sets. It does not
reach the node, the cli or the sdk, which do not import snapshots with a key (the function is only
called by the layer's tests; `git grep`), nor the kit, which does not depend on the layer. The
window runs from `5d2833e` (28-07-2026), where the function is born, until §817. Two comments in the
header of `snapshot.rs` left it false —the encrypted snapshot, and the tail of retired ones that
only the sealing binds—, and so did 3.13 («Only the encrypted one protects them»); they carry a
mark.

**The fix.** With a key, the mark `0x00` is refused, with a `Malformed` whose text says that the
snapshot is in the clear and that, if an encrypted one was expected, whoever writes where it is kept
may have put it there: `import_snapshot`, which receives no key, would import it without
authenticating anything. It fails closed. It is bound by
`con_clave_una_instantanea_en_claro_no_se_importa`, at the end of `crates/zk-ssl/src/tests.rs`
(⚠️ §819: it is no longer the last: behind it goes that of §819, in 3.14; ⚠️ §820: and behind that
one, the three of §820, also in 3.14; ⚠️ §821: and behind those, the two of §821, below, in «What it
does not do»; ⚠️ §828: and behind those, the twelve of §828, in 3.17): with the passphrase, with
another passphrase and with a key already fixed to its KDF, one in the clear is not imported; the
same body after a mark that is not an encryption one, neither; nor a v8 in the clear; and without a
key, it is.

**What it does not do.** `import_snapshot` still imports one in the clear: it is the path of a layer
without a key, and whoever calls it has to know where it comes from. The import with a key does not
bind the snapshot to a ledger or to a moment: an encrypted one with the same passphrase is imported
even if it is from another ledger (measured outside the tree, in the review of §817: the one from
another ledger, put in the place of the right one, gives Ok with its balances) or from before
(deduced: what the encryption authenticates is the key the passphrase derives with the salt of its
header, not which of the snapshots sealed with it it is). And entry 121 of `BACKLOG.md`, the
imported layer that is born without a key and is re-exported in the clear, remains open; because of
it, a layer imported with a key and exported again comes out in the clear, and that copy, which
cannot be told apart from one put there by someone else, is no longer imported with a key: it gives
the new error. It fails closed, but whoever chains import, export and import will see an error they
did not see before. ⚠️ §821: entry 121 is closed by §821, by the author's decision: the layer
imported with a key keeps the snapshot's key, fixed to its header, and is re-exported encrypted,
with the same passphrase; the re-exported one is imported again with the passphrase, and whoever
chains import, export and import no longer sees that error
(`una_capa_importada_de_una_cifrada_se_reexporta_cifrada_con_su_cabecera`
and `una_capa_importada_de_una_v1_se_reexporta_en_la_version_2`, at the end of
`crates/zk-ssl/src/tests.rs`; ⚠️ §828: they are no longer the last: behind them go the twelve of
§828, in 3.17).
And two decisions of the session, reversible: that of a v1 is re-exported in version 2, with a new
salt; and a v1 is no longer imported with a key already fixed (`LedgerKey::derivar`), which does not
carry the passphrase, because the new salt requires deriving it with the passphrase: whoever imports
it that way will see an error they did not see before, which says so and names
`LedgerKey::from_passphrase`. It fails closed. What a layer without a key exports, like that of
`import_snapshot`, is still in the clear, and that copy is still not imported with a key.

It was drafted in private and is published at once with its fix, as section 5 asks, and like §812
and §816, by the author's decision: the notice is this section and entry §817, without a GHSA or
release, because the published kit does not depend on the layer. `AUDITORIA.md` §817.

### 3.17 The re-encryption and the v1→v2 migration left the ledger owned by the user and the group of whoever ran them — ⚠️ MEASURED, closed in the library (§828)

The re-encryption of E4 (`SovereignLayer::recifrar`, §812, `crates/zk-ssl/src/persistence.rs:1318`)
and the migration of the at-rest encryption from version 1 to 2 —since §812, the same piece, which
runs on every opening with a key of a v1 ledger (`migrar_a_kdf_v2`, :1050)— call
`recifrar_en_hermano` (at :1353 and at :1064), which writes the ledger into a sibling directory, R
(`<libro>.recifrando`), and puts it in its place.
R was born with the user and the group of the process —the directory, in `nace.create(&r)` (:1421);
`conf`, `db`, the `snap.*` and `blobs/`, on opening sled over R (:1501)—, or with the parent's group
if the parent has setgid. Step 4 gave it only the ledger's modes (`copiar_modos`, :2143, and the
directory's, :1598-1599), and R took the ledger's place in step 6 (:1456) or, after a death between
steps 5 and 6, in the next opening (`resolver_restos`, :1702). The session saw it by reading the
code, and it was measured in private, in a clone without a remote, on §821, with `setpriv`:

- **root** re-encrypting or migrating a ledger of 12345:12346 leaves it as 0:0, with the same modes,
  and its owner no longer opens it: «IO error: Permission denied (os error 13)» in 5 of 5 ledgers,
  and of those five it reads 0 of their 25 entries, all of which it read before. After a death in
  step 5 it does not finish the re-encryption either («….recifrando: Permission denied»): the ledger
  stays in its `.retirado` and its `.recifrando` until root acts. The content does not change: root
  opens it, with its balance;
- **the unprivileged owner with another primary group** —12345, of primary group 12345 and with
  12346 as supplementary— leaves a ledger of 12345:12346 as 12345:12345, with the same modes: with
  0750 and 0640, the ledger's group loses read access (12347:12346, 0 of 10 entries) and the owner's
  primary group gains it (12348:12345, 10 of 10). Being in group 12346 does not prevent it. With a
  setgid parent, the group is kept.

Whoever gains read access reads what is sealed and what goes in the clear —the KDF header, with its
salt and its cost—, and with that can try passphrases offline against Argon2id, the threat RFC-0001
already declares; the key does not change hands, and root already read everything. **Scope**:
whoever integrates the library and re-encrypts, or opens a v1 ledger with a key, as another user
—root or `sudo`, in maintenance— or with another primary group —a service—. No binary of the
repository calls `recifrar` or opens with a key: the node and the cli open without a key
(`SovereignLayer::open`, `crates/zk-ssl-node/src/main.rs:2293` and
`crates/zk-ssl-cli/src/sandbox.rs:57`), and that opening only finds an R if a re-encryption by the
library died before. The kit does not depend on the layer. **The window**: since `b6994c3` (§812,
14-10-2026), for the re-encryption and for the migration; before, the migration was a batch on the
same sled and did not move the files (deduced from the code of `b6994c3^`, without compiling it).
The header of `copiar_modos`, steps 1 and 4 of E4 in RFC-0001 and the row of R in its table of
reserved names left it false, and they carry a mark.

**The fix** (§828). What the author decided is a choice (21-10-2026, in the chat): to a
multiple-choice question from the session, he chose the option «Rehusar si no es el dueño
(Recomendado)» [refuse if it is not the owner (recommended)], and not the other one, «Copiar dueño y
grupo como root» [copy owner and group as root]. The description of the option he chose, written by
the session, said: «re-encrypting and the v1→v2 migration fail closed if whoever runs them is not
the owner of the ledger (for example, root on a service's ledger) and the error says to do it as its
owner. If it is the owner, it keeps the ledger's group if it is one of theirs; if not, it removes
the group and other bits, like 120. It never does chown as root. Less code and no new surface.»
Everything else is the session's, and reversible: where and how, and how it reads that description.
Two sentences of the description it reads as follows. «It never does chown as root» is not exact
about the code: the user of nothing is ever changed, nor the group or the modes of what whoever
re-encrypts did not create; but root, when it is the owner of the ledger, does change the group
—`fchown(fd, None, Some(gid))`— of what it itself created inside an R of its own with 0700, and of R
(the second point below). And «if it is one of theirs», as the group the `fchown` lets it set: for
root, any, also one that is not among its groups, like the 12346 of the second test; and if the
`fchown` fails, for whatever reason, it removes the bits. That `recifrar` also compares before
taking the ledger's lock and opening it is likewise the session's, and reversible: it was done after
measuring what the opening that precedes the guard of step 1 left behind (in «What it does not do»).
This is how it does it:

- **the ledger of another owner is neither re-encrypted nor migrated.** On Unix the user of whoever
  re-encrypts or migrates is compared with that of the ledger's directory and with that of its `db`,
  its `conf` and its `blobs/`: those sled opens for writing, and those step 6 would leave owned by
  whoever re-encrypts. `recifrar` compares it before taking the ledger's lock and opening it
  (`quien_recifra_es_el_dueno`, at the end of `persistence.rs`); it learns the user of whoever
  re-encrypts without `libc`, from the owner of a probe it creates and deletes on the spot next to
  the ledger —an empty file `<libro>.sonda.<pid>.<n>`, with `create_new`, which does not follow a
  link nor open whatever already bears that name, and 0600, looked at through its descriptor—, which
  is the user with which the process creates in that directory, the one R would have (why there, in
  «What it does not do»); next to what is not sled's it does not create it —it looks first, without
  writing—, and if it cannot delete it, it refuses naming it (both things, with a test: the
  eleventh); and if it cannot know —the probe cannot be created, or the ledger is not there, is not
  sled's or cannot be looked at—, it does not refuse for that, and carries on (with a test, with a
  probe that does not fit in its name and with its name already taken —a symbolic link to an outside
  file that is not there, one to one that is, and a hard one—, which is not followed: the twelfth;
  with what is not sled's, the eleventh; with the ledger that is not there or cannot be looked at,
  deduced). And as soon as R is born —step 1— the same is compared with its owner, the user of the
  process, which has just created it: in the migration it is the only comparison, because that the
  ledger is v1 is only known on opening it; in the re-encryption, the backstop. If any is not the
  same, it is refused with `Malformed` —and in step 1, R, empty, is deleted—: «el libro es del
  usuario 12345, y lo intenta recifrar el usuario 0: asi quedaria del usuario 0, y su dueno no lo
  abriria; recifralo como su dueno. No se copia, no se sella ni se renombra nada», and in the
  migration, «lo intenta migrar a la version 2 -abrirlo con su frase lo migra-» and «abrelo con su
  frase como su dueno, que lo migra». A ledger that does not belong to a single owner —the directory
  of an `install -d -o root -g svc -m 2770` with the service's files— is neither re-encrypted nor
  migrated by either of the two, and the error names what belongs to another: «…/db: es del usuario
  12345, y el directorio del libro del usuario 0: el libro no es de un solo dueno, y lo intenta
  recifrar el usuario 0, que lo dejaria todo suyo; que sea todo de su dueno, y recifralo como su
  dueno». An earlier draft of this fix compared only the directory, and with that ledger root
  re-encrypted and migrated, and `conf`, `db` and `blobs/` were left owned by root: a review
  measured it, and the service no longer opened it («Permission denied (os error 13)»). The `snap.*`
  are not compared: root's opening leaves one of its own in another's ledger (below, in «What it
  does not do»), and refusing because of it would be excessive (deduced). The refusal of the
  re-encryption comes before creating, opening or writing anything of the ledger or of its lock —if
  the earlier comparison knows; if it could not create its probe, in step 1, after taking the lock,
  and creating it if it was not there, and opening the ledger—, and a v1 ledger being re-encrypted
  never gets migrated; that of the migration, before copying, sealing or renaming anything —what the
  sled opening leaves before it, in the first point of «What it does not do»—. Neither leaves a
  `.recifrando` or a `.retirado`, nor changes the owner, the group or the mode of what was there.
  The opening with a key of another's v1 ledger fails whole: it is not opened without migrating, and
  it fails closed. R also has to be what was born: a directory, empty; its device and its inode are
  kept, and step 4 requires them;
- **if it is the owner, R and each file of R take the group of its namesake in the ledger** —the one
  that has none, a `snap.*` or a blob, which carry their number, that of `db`; the directory, the
  ledger's—, with the rule of the export of entry 120: `fchown` with the owner as `None`, which the
  owner of a file can set if it is one of its groups, and root, any, through a descriptor opened
  without following a link —the type and the inode, before and after opening, and on Linux x86 and
  arm, also `O_NOFOLLOW`—; if the group cannot be set, the entry is left without the group bits or
  the other bits (`& 0o700`); and afterwards, the namesake's modes: also the setuid, which the
  `chown` of a file always removes, and the setgid, which it removes if the file has the group x bit
  (measured in the session, on Linux 6.18: a `fchown` by root leaves 2640 at 2640 and takes 2650 to
  0650 and 4640 to 0640). The directory, last, after syncing the whole of R. The user of nothing is
  ever changed: what step 6 puts in the ledger's place belongs to whoever re-encrypts, and the guard
  requires that to be the owner of the ledger's directory and of its `db`, its `conf` and its
  `blobs/`. But root, when it is the owner, does change the group, like any owner: with
  `fchown(fd, None, Some(gid))`, that of what it itself created inside the R that was born, its own
  and with 0700, and that of R; to what it did not create it gives neither group nor modes: it
  refuses, and the re-encryption that refuses deletes R with whatever it holds, as since §812 (with
  what that guard does not reach, in «What it does not do»). It is the session's reading of «It
  never does chown as root», above. It is the session's, and reversible, to refuse before the commit
  —R is deleted and the ledger stays as it was— if in R there is a link or something that is neither
  a file nor a directory, a file with another name (a hard link), an entry of another owner or a
  directory inside `blobs/`, or if R is no longer the directory that was born —another device or
  another inode—, owned by the ledger's owner and with 0700;
- **the opening neither finishes nor deletes a `.recifrando` of another owner** —the one the code
  from before left if root re-encrypted or migrated and died halfway, or one someone put there—: if
  its owner is not the ledger's —that of the directory and that of its `db`, its `conf` and its
  `blobs/`, or those of its `.retirado` if the ledger is missing—, or the `db`, the `conf` or the
  `blobs/` of the `.recifrando` do not belong to its owner, it refuses with `Malformed` without
  touching anything, and says whose each one is and what to do. The ledger is not lost: with the
  ledger beside it, it is whole with the previous passphrase; with its `.retirado`, there are the
  `.retirado`, with the previous one, and the `.recifrando`, with the new one. Refusing also with
  the ledger beside it, and not deleting that `.recifrando`, is the session's, and reversible: it
  fails closed. The owner's `.recifrando` that the opening finishes after a death between steps 5
  and 6 already took in step 4, which comes before 5, its group and its modes, and keeps them; and
  one of the same owner and of another group —the one the code from before left, re-encrypted by its
  owner with another primary group— it finishes as before, with its group;
- outside Unix, as before: with no owners to compare nor groups to set.

Twelve tests bind it, at the end of `crates/zk-ssl/src/tests.rs`. Six require root and, without it,
are skipped whole: the first, the second, the fifth, the sixth, the seventh and the tenth of those
below; the fourth runs without root except its (c); the eleventh, except its (a) with another owner
and its (d); the twelfth, except its (b) and its (d); and the third, the eighth and the ninth do not
require it. The (a) and the (b) of the twelfth are also skipped, with or without root, if the
process's pid has fewer than three digits: then no ledger name fits its `.recifrando` without the
probe always fitting too (below); its (c) and its (d) do not depend on the pid. What is skipped says
so on the error output, which libtest only shows with `--nocapture` or `--show-output`: in the usual
output they come out «ok» (measured as 12345:12345 with `setpriv`, in the private clone, with the
binary of the last draft: twelve «ok» without any «sin medir» line, and eleven with `--nocapture`).
The twelve:
`el_libro_de_otro_dueno_no_se_recifra_ni_se_migra_y_no_se_toca` —a v2 ledger and a v1 one of
12345:12346, settled beforehand (opened and closed after their last write, so that root's opening
does not write a `snap.*` of its own in them) and with their lock: the re-encryption of both refuses
before the E4 piece, which starts only once, that of the migration; the opening with a key of the v1
refuses in its step 0 or 1, with an error injected at the end of step 1 that the guard has to
pre-empt; all three, with the text that names the two users and without reaching the step that gives
the group, and the ledger stays owner by owner as it was—;
`el_libro_recifrado_o_migrado_por_su_dueno_conserva_su_grupo_y_sus_modos` —a ledger of 0:12346, with
modes set by hand (the directory 2750 and `conf` 4640): at the end of step 4, each entry of the copy
is of group 12346 and carries the mode of its namesake —at least one, a `snap.*`, without a
namesake—, and keeps it in the ledger—;
`sin_poder_poner_el_grupo_la_copia_se_queda_sin_bits_de_grupo_ni_de_otros` —with the `FALLAR_DUENO`
hook of entry 120, `& 0o700`—;
`un_enlace_o_lo_ajeno_en_la_copia_no_recibe_el_grupo_del_libro` —a link, a hard link, an entry of
another owner and the copy opened to 0755, put in R at the start of step 4: it refuses, and the
link's target does not change—;
`la_apertura_no_termina_ni_borra_un_recifrando_de_otro_dueno`;
`el_libro_que_no_es_de_un_solo_dueno_no_se_recifra_ni_se_migra` —the directory as 0:12346 with 2770,
and `db`, `conf` or `blobs/` of 12345: the re-encryption refuses —that it is before the lock this
test does not see, because step 1 gives the same text: the tenth sees it—, and the migration of each
one in step 1, with the injected error, naming it, and the ledger stays as it was; and the opening
does not delete a `.recifrando` of root's next to a ledger with its `db` belonging to another, nor
one with its own `db` belonging to another—;
`el_recifrando_de_su_dueno_que_termina_la_apertura_conserva_su_grupo_y_sus_modos` —after a death in
step 5, and with another group on the `.recifrando`—;
`lo_que_lleva_blobs_en_la_copia_toma_lo_de_db_y_un_directorio_hace_rehusar` —the ledger has no
blobs, and into the copy's `blobs/` are put a file, which takes the group and the mode of `db`, and
a directory, which makes it refuse—;
`la_copia_cambiada_por_otro_directorio_no_recibe_el_grupo_del_libro` —R swapped for another
directory of the process, with 0700 and a file inside, between creating and comparing it and at the
start of step 4: it refuses, and the file keeps its mode—;
`el_libro_sin_asentar_de_otro_dueno_rehusa_antes_del_cerrojo_y_no_recibe_nada` —in a directory of
12345:12346 with 0700, four unsettled ledgers without a lock, one of 12345:12346 and three of
0:12346 with their `db`, their `conf` or their `blobs/` of 12345: the test runs itself in another
process, with umask 0077, and root re-encrypts them: they refuse with their text, and the directory
and the four ledgers stay entry by entry as they were, also the bytes of `db` —not a `snap.*`, nor a
lock, nor the probe, nor a sibling—; and its owner, with `setpriv`, afterwards opens the one that is
theirs, with its supply—;
`lo_que_no_es_un_libro_no_recibe_la_sonda_y_la_que_no_se_borra_hace_rehusar` —re-encrypting a
directory that is not sled's refuses as before, without creating the probe and without changing
anything of the directory that holds it, not even its modification time, also as root with the
directory of 12345:12346; with the probe swapped for a directory, which is not deleted, it refuses
only once, naming it, without taking the lock or touching the ledger, and with that directory
removed it re-encrypts and the probe does not stay; and as root, with the parent `chattr +a` and the
ledger of 12345:12346, it refuses naming it, and the only new thing in the parent is that probe,
empty, of 0:0 and with 0600—;
and `la_sonda_que_no_se_puede_crear_no_hace_rehusar_y_el_paso_1_queda_de_respaldo` —the probe that
is not created, in two ways. With the longest name in which the ledger's `.recifrando` fits,
measured in the test's directory, in which the probe does not fit if its suffix, `.sonda.<pid>.<n>`,
is longer than `.recifrando`: with a pid of three digits or more; with one of one or two there is no
name in which `.recifrando` fits without the probe always fitting too —the shortest,
`.sonda.<pid>.0`, has ten or eleven characters, and `.recifrando`, eleven—, and that part is skipped
and says so (measured with `unshare --pid`, with pid 1). And with its name already taken, which does
not depend on the pid: the test runs itself in another process, in which only it runs and the count
of probes starts at 0, and puts in the names of its first three probes, next to three ledgers, one
per name, a symbolic link to an outside file that is not there, one to one that is, and a hard one
to that one, and re-encrypts each ledger once; none is followed: the one that was not there is not
created, the one that is does not change —not its bytes, nor its owner, nor its group, nor its mode,
nor its modification time— and the three names stay as they were. In both, the re-encryption by its
owner carries on, goes through step 1 and is re-encrypted; and, as root, that of a settled ledger of
12345:12346 —with the name already taken, with the owner's symbolic links, in its directory, of
12345:12346 with 0700— refuses in step 1, with its text, without siblings and with the ledger owner
by owner as it was, and leaves the ledger's lock, of 0:0 with 0600—.
Measured with mutants, each applied in the private clone, compiled —with a binary different from the
good one—, run with the twelve tests and restored afterwards, with sha256, on the code of the last
draft, on §821 —in `crates/`, the same diff as this seal—: thirty-nine of the forty fall, all in the
first run, and one survives, which is declared (below). The code from before (the `persistence.rs`
and `snapshot.rs` of §821, with only `O_NOFOLLOW`, `ELOOP` and `abrir_sin_enlace` added), in all
twelve; without the prior comparison, in the first —the re-encryption of the v1 reaches the
migration and gives its text—, in the tenth, which sees root's `snap.*` and lock, of 0:0 with 0600,
and in the eleventh, which re-encrypts without going through the probe; the comparison after taking
the lock, in the tenth, because of root's locks, and in the eleventh, which takes it; the probe that
stays on the disk, in the tenth, because of root's next to each ledger, and in the eleventh, which
re-encrypts; the prior comparison that only looks at the ledger's directory, and the one that leaves
out `db`, `conf` or `blobs/`, in the tenth, because of root's lock and `snap.*` in the `mixto_*` it
does not look at; creating the probe without first looking that it is sled's, in the eleventh, which
sees it next to what is not a ledger; not refusing if it cannot be deleted, in the eleventh, which
re-encrypts; refusing then with `Io`, which `recifrar_e4` retries, in the eleventh, which goes
through the probe ten times, with another each time; refusing if the probe is not created, in the
twelfth, whose owner does not re-encrypt, in its (a); the guard of step 1 only in the migration, in
the twelfth, in which root, with another's ledger, refuses later, in step 4 —the `.recifrando` does
not belong to the ledger's owner—, with another text, in its (b); creating the probe with `create`
instead of `create_new`, in the twelfth, in its (c), where it follows the link to what is not there,
creates it and, on deleting the probe, removes the link; without the guard of step 1, in the first,
in the sixth and, like the migration-only one, in the twelfth; and with it after step 1 —just before
giving the group, or at the end of step 2—, in the first and in the sixth, which in the migration
give, as without it, the injected error («fallo inyectado al acabar el paso 1») and not the one that
names the two users; in step 1, comparing only the directory, or leaving `db`, `conf` or `blobs/`
out of the comparison, in the sixth, through its migration; without the group, in the second, the
third and the seventh; without the directory, in the second and in the seventh; following the links,
and removing the guard of the hard link, that of the owner of each entry or that of the copy at
0700, in the fourth; without the `& 0o700` fallback, in the third; setting the modes before the
group, in the second and in the seventh, because the `chown` removes the setuid of `conf` (4640 →
0640): only that assertion sees it, the one in `comprobar_la_copia`, which the two share; taking
`db`'s instead of the namesake's, also in the second and in the seventh, because of `conf` (0640
instead of 4640); not requiring in step 4 the inode of step 1, or R empty in 1, in the ninth; not
descending into `blobs/`, or not refusing a directory inside it, in the eighth; giving the group
only to the entries with a namesake, in the second, the seventh and the eighth; the opening not
looking at whose the `.recifrando` is, in the fifth and in the sixth; looking only at the
directories, or not looking inside the `.recifrando`, in the sixth; and also comparing the group, in
the seventh. With pid 1 —with `unshare --pid --fork`, with which the (a) and the (b) of the twelfth
are skipped—, there also fall, in the twelfth and in the first run, those of refusing if the probe
is not created and of creating the probe with `create`, in its (c), and those of the guard of step 1
only in the migration and without it, in its (d). There survives, 3 of 3 times, the one that makes
the comparison accept an empty list of owners, like an earlier draft: it is not reached
—`de_quien_es` always gives the ledger's directory, or an error—, and that it now refuses remains
deduced, without a test. The other thirty-nine were also run on the previous draft, with its twelve
tests, and thirty-eight fell and the same one survived; thirty, on the one before, with the ten
tests of then, and all fell; and twenty-five, on the one before that, without the prior comparison,
with the nine of then, and so did they (the entry). Of the nine the previous draft added, four —the
prior comparison without `conf` or without `blobs/`, refusing if the probe is not created and the
guard of step 1 only in the migration— passed the ten tests of the one before (its review measured
it); and that of `create`, which the last one adds, passed the twelve of the previous one, 3 of 3
times (its verification measured it).
In the private clone, the whole layer: 511 pass, 0 fail and 7 ignored, without warnings on
compiling, with the binary of the last draft (146.2 s of tests); and the twelve, 10 of 10 times with
that binary (9.1-11.0 s), and 3 of 3 with pid 1 (8.8-9.3 s); and the twelfth alone, 20 of 20
(5.6-7.1 s). With the previous draft, 511 pass and the twelve, 3 of 3; with the one before, 509 and
the ten, 3 of 3. Before the prior comparison, the layer gave 508 pass, 0 fail and 7 ignored, five
times, and the nine of then passed 29 of 29 times with the binaries of the drafts of then (the
entry). On this tree, §827 with this seal, and as root: the whole layer, 515 pass, 0 fail and 7
ignored, without warnings on compiling (260.2 s of tests), and the twelve, 3 of 3 times (13.3-17.5
s), and 1 of 1 with `--nocapture`, without any «sin medir» line.

Measured also outside the suite, with `setpriv` and the binary of an earlier draft, from before the
prior comparison, on ledgers like those of the starting measurement: root refuses in 12 of 12 —seven
re-encryptions of a v2 and five openings with a key of a v1—, with the text above. With that binary
the seven re-encryptions refused in step 1, after taking the lock and opening the ledger; since the
prior comparison they refuse earlier, without creating the lock (deduced from the code; with the
ledger unsettled and without a lock, the tenth test measures it). Afterwards, the owner reads the 50
entries of the ten settled ledgers; opens the two v2 that had their lock, with their balance —the
one that did not have it, no, because root created it as its own, with that binary—; the two v1 in a
directory of root's the owner cannot migrate, because they cannot create their `.recifrando` in it;
and the five in a directory of their own the owner re-encrypts or migrates themselves, `Ok`, and
opens afterwards, with their balance. With primary group 12345 and 12346 as supplementary, the
ledger stays as 12345:12346: 12347:12346 reads 15 of 15 entries and 12348:12345, 0 of 15; before the
fix, on the same three ledgers, 5 of 15 and 10 of 15 —the 5 were those of the ledger of the setgid
parent, which already kept its group—. Without group 12346, the ledger is left as 12345:12345 with
0700 and 0600 —the fallback—, and 12347:12346 reads 0 of 10. In this measurement all the ledgers
belong to a single owner; the one that does not is measured by the sixth test.

**What it does not do:**

- **in the migration, the sled opening that precedes the refusal, and the lock that whoever opens
  creates before it: it is what follows from the migration not being able to compare before
  opening** —the option the author chose names no step; putting the guard in step 1, and also in
  `recifrar` before taking the lock, is the session's, and reversible—. Before step 1 the opening
  creates the ledger's lock if it is not there (`Cerrojo::tomar`) and opens the ledger with sled,
  because opening it is what triggers the migration; and it only migrates a v1 ledger, and that it
  is v1 is said by what sled keeps —data and no KDF header, `meta:kdf`—, which is read on opening
  it: without knowing beforehand that it is v1 there is no way to refuse beforehand only the opening
  that would migrate it, and refusing beforehand every opening with a key by another user would also
  refuse that of a v2, which does not migrate, and the chosen option does not say that (deduced). In
  the re-encryption the same happens only if the earlier comparison could not know who re-encrypts
  —the probe could not be created— (with a test: the twelfth, which measures the lock of root's that
  it leaves). A draft of the twelfth re-encrypted as root, by that path, three times in a row the
  same settled ledger of 12345:12346, and in one of its runs of the mutant bench —with one that does
  not touch that path— the ledger had, after the three refusals, a `snap.*` of 0:0 with 0644 in the
  place of the owner's (measured once; it was looked at after the three, and it is not known after
  which one; how often, was not measured; the twelfth now re-encrypts each ledger only once). Why,
  deduced and without measuring: a refusal can write in `db` (below, 3 of 140) and leave the ledger
  unsettled, and an opening of an unsettled ledger leaves a `snap.*` (below, 40 of 40). That opening
  by root does what sled does on any opening; what follows was measured with the code from before
  the prior comparison, when the re-encryption too opened the ledger before refusing. With the
  ledger settled —opened and closed after its last write— neither the name, the inode, the owner,
  the group nor the mode of anything changes, nor the bytes of `conf` and of the `snap.*`, in 420 of
  420 refusals, and it wrote in `db` in 3 of 140 refusals to re-encrypt a v2 —about 1.7 KB of
  differing bytes— and in 0 of 280 to open or re-encrypt a v1 (with four threads at once; the same
  in two measurements, with the binary of an earlier draft and with that of before a touch-up of
  comments and texts, the same code as that of the draft before the prior comparison; what `db`
  keeps, its keys and its values, does not change: the test binds it). With a ledger that is not
  settled it leaves a new `snap.*`, of 0:0 and with the umask —0644 with 0022—, in 40 of 40
  refusals, also in the two measurements: in the 20 to re-encrypt a v2, next to those that were
  there, and rewriting `db` (between 506257 and 509254 differing bytes, in the second); in the 20 to
  open a v1 with a key, in the place of the owner's `snap.*`, which is no longer there; and the 20
  to re-encrypt that v1 afterwards, already settled by that refusal, change nothing. That the owner
  reads that 0644 is deduced from the mode; it was not measured. With umask 0077 —or any umask that
  removes read access from others, like 0027 for whoever is not in group 0 (deduced)— the new
  `snap.*` is of 0:0 and the owner does not read it —with 0077, 0600—, and no longer opens the
  ledger («IO error: Permission denied (os error 13)», in 2 of 2 in the two measurements: a
  re-encryption, which rewrote `db` —from 17561 to 524287 B in the second—, and a migration, which
  refused; since the prior comparison, the re-encryption refuses earlier and leaves nothing: the
  tenth test measures it) until root gives it to them —the review of an earlier fix, discarded,
  measured it with a minimal sled opener: «Ok» after the `chown` of that `snap.*`— or removes it
  (deduced, without measuring). Any opening by root on that ledger would do the same, also the cli's
  one without a key (deduced: it is the same sled opening; without measuring). And if there was no
  lock, the one root creates leaves the owner unable to open (below). **The alternative, comparing
  in `recifrar` before taking the lock and opening the ledger, is done** (above): it avoids it in
  the re-encryption, and not in the migration. Its cost was a second comparison —that of step 1 is
  still needed for the migration, and as a backstop—, the probe, and changing the first test, which
  counted `recifrar:0` three times and now counts it once. It did not require `libc`: the user of
  whoever re-encrypts is given by the owner of the probe, as in step 1 it is given by the owner of
  R. An earlier draft of this text said that it required it, and that that was new surface; it was
  not (its verification saw it). Doing it is the session's, and reversible; the author chose his
  option before this was measured;
- **the probe**: it is created next to the ledger —in its parent directory, where step 1 would
  create R— because its owner is the user with which the process creates in that directory, the one
  R would have, also with an NFS with `root_squash` or with a mount that changes the owners, and
  because a permissions failure on creating it is the same as that of step 1 (deduced). Not
  `/proc/self`: it gives the effective user, and not that of the files (`setfsuid`); it only exists
  on Linux with `/proc` mounted; and it knows nothing of the ledger's file system (deduced; it does
  give the effective user after a `setuid` in the same process, which leaves it without a dump:
  measured in the session, on Linux 6.18, with its `status` already root's). Nor the temporary
  directory, which may be another file system, and a probe left there would not be seen next to the
  ledger (deduced). Creating and deleting it changes the parent's modification time (measured in the
  session, on Linux 6.18 and ext4, with a harness outside the tree: root's re-encryption that
  refuses another's ledger, v2 or v1, changes it without leaving anything in the parent; and the one
  that refuses a directory that is not sled's, does not), as R changes it (deduced). Next to what is
  not sled's it is not created: it is looked at first, without writing, and it refuses as before
  §828 (with a test, the eleventh; an earlier draft created it before looking, and its review saw
  it). If it cannot be deleted —a parent with `chattr +a`, where it is created and not deleted—, it
  refuses with `Malformed`, naming it, without taking the lock or opening the ledger, and it stays
  until it is deleted by hand (with a test, the eleventh; an earlier draft left it without saying
  so, and its review measured it). `Malformed`, and not `Io`, because whoever retries an `Io` would
  create another probe on each attempt (measured: with an intermediate state of this fix, which
  refused with `Io`, the `recifrar_e4` of the tests, which retries it up to ten times, left ten with
  the harness; and with the mutant that refuses with `Io`, the eleventh test goes through the probe
  ten times). If the process dies between creating and deleting it, it also stays —empty, owned by
  whoever was re-encrypting, with 0600— and nobody looks at it or deletes it: it is deleted by hand
  (deduced, without a test). If the probe cannot be created —whoever re-encrypts does not write in
  the parent, the name is already there, or it does not fit—, the earlier comparison does not
  refuse, and there remains that of step 1, with what that opening leaves (with a test, with a name
  in which it does not fit and with the name already taken, by links: the twelfth). With the name
  already taken —ten probes of the ledger's owner, or ten links, with the process's pid—, root thus
  refuses in step 1, leaves in the ledger, unsettled, a `snap.*` of its own —with umask 0077, 0600—
  and, next to the ledger, its lock, and the owner no longer opens it (measured with the harness). A
  link is not followed: neither a symbolic one, to a file that is not there or to one that is, nor a
  hard one; the one that was not there is not created, and the one that is does not change (with a
  test: the twelfth, with the process's ledger and, as root, with another owner's and that owner's
  links; before, the two symbolic ones, measured with the harness). If whoever re-encrypts cannot
  write in the parent, step 1 will not be able to create R either (deduced);
- **the `snap.*` that sled writes on reopening the ledger in step 8**, and on any opening by the
  owner in day-to-day use: they are of the owner's primary group —or of the parent's group, if it
  has setgid— and with the owner's umask, not the ledger's (measured: a `snap.*` of 12345:12346 with
  0644 in a 2750 ledger, and one of 12345:12345 with 0644 in a 0700 one, after their owner
  re-encrypted or migrated them). With a 0750 or 0700 directory they reach nobody beyond those who
  already went through it (deduced);
- **the lock of §816** that was not there: whoever opens creates it as their own, and whoever
  re-encrypts too, if they are the ledger's owner or if the earlier comparison could not know who
  they are (the twelfth test measures it); one of root's leaves the owner unable to open (measured
  again, with the code from before the prior comparison: «….cerrojo: Permission denied»); 3.14
  already says so, in its residue;
- **what the code from before already re-encrypted or migrated** stays owned by whoever did it. A
  `chown -R` to its owner gives it its owner back, and removes the setuid of a file that has it,
  which has to be set again (measured in the session: after root's `chown -R`, a 4640 `conf` is left
  at 0640, and the 2750 directory stays; that with that the owner opens it, deduced). A
  `.recifrando` of another's left halfway makes the opening refuse, with the text that says what to
  do; and a `.retirado` next to a ledger of another owner —a death of root after its step 6— the
  opening deletes as before, and the ledger stays owned by whoever re-encrypted (deduced, without a
  test);
- **what the owner does not control of the group**: if they are not in the ledger's group, the
  re-encrypted or migrated ledger stays in the group the copy was born with —their primary group, or
  the parent's with setgid—, without group or other bits, and the ledger's group no longer reads it;
  recovering it requires a `chgrp` by whoever can. It does not take the POSIX ACLs nor the other
  extended attributes of the ledger —R takes the parent's default ACL— (deduced, without measuring);
- **what the guard of the links does not reach**: `std` has neither `openat` nor `fchownat`, and
  each look resolves the path again; a change undone between two looks is not seen, and inside R
  only its owner, or root, can make it. Whoever can write in the parent directory can swap R for
  another directory —for that, the parent does not have the sticky bit, and the directory they put
  is already in that parent, because moving a directory to another parent requires writing in it
  (deduced from `rename(2)`)—. Up to step 4 the guard sees it, with a test: the R that is not empty
  when compared, in step 1, and the one with another device or another inode, in 4; and the
  re-encryption that refuses deletes whatever is in R's place, as since §812. An empty one of the
  ledger's owner, put there before comparing it, receives the copy and the group like the real one
  (deduced). And there remains the gap between step 4's check that R is the one that was born and
  the pass over its entries, which reads R by its path (`read_dir`) and opens each entry by its
  path: if in that gap someone puts another directory in R's place, the entries of that other one
  that belong to R's owner —the ledger's, who is whoever re-encrypts— and, if they are files, with a
  single name, take the group and the modes of their namesake in the ledger, or those of `db`, and
  whatever a directory of theirs inside holds, likewise; the rest make it refuse. On finishing the
  entries the guard sees the change and refuses, but after giving them; to the directory put there
  it gives neither the group nor the mode of the ledger, and the re-encryption that refuses deletes
  whatever is in R's place, if whoever put it there did not remove it before. Since this seal it
  also gives them the ledger's group; until it, `copiar_modos` gave by path only the modes, to
  whatever was there, whoever's it was, from step 4 to 6. It can be done by whoever writes in the
  parent, with a directory already in that parent, as above; and what they gain is nothing that was
  not already the ledger's owner's —what belongs to whoever makes the change, if they are not the
  owner, makes it refuse—: that whatever of the owner's there was in that directory takes the group
  and the modes of the ledger, and that the ledger's group reads it if it can enter it, because that
  directory's mode is not changed (all of it, deduced from the code, without a test). Afterwards,
  from the end of step 4 until 6, changing it is the gap §812 already had (deduced, without a test);
- **what is refused in excess**: the ledger that does not belong to a single owner is neither
  re-encrypted nor migrated by either of the two —with a test, that of the directory; that of the
  files, deduced—, although for the owner of the files it would suffice (deduced): making it all one
  owner's fixes it. And, deduced and without measuring: on a file system that does not keep Unix
  modes, R is not left at 0700 and the re-encryption refuses; with NFS and `root_squash`, R is born
  owned by another user and root refuses —in the re-encryption, already in the prior comparison,
  because the probe is born the same way—;
- outside Unix nothing changes, and it is not tested on Windows nor on NFS.

It was drafted in private, as section 5 asks. How it is published was chosen by the author
(21-10-2026, in the chat), in answer to another question of the session: the option «Como el §812,
sin GHSA (Recomendado)» [as with that entry, without a GHSA (recommended)], whose description,
written by the session, said: «At once with the code. The notice is the new section of SECURITY.md
(3.17) and the entry, without a GHSA or release, like §812, §816 and §817.» That section 5 asks for
no advisory or release, because the published kit does not depend on the layer, is the session's
reading. `AUDITORIA.md` §828.

### 3.18 Eight defects of the kit's verifier, from 0.1.0 to 0.4.3, and one of the layer: VERDE to envelopes that do not hold, 101 on a malformed proof, a cost chosen by whoever sends and a delegated issuance that was resent — ⚠️ MEASURED, closed in kit 0.4.4 and in the layer (§711 to §723)

The kit's verifier, `zk-ssl-verify`, had eight defects in the published releases. Each one reaches
its own releases, stated in its point. The layer had, besides, one of its own, that of the delegated
issuance, which comes after the eight and without a number: it is neither the kit's nor the
advisory's. They came out of an internal review by the author with the help of Claude Code
(`GENAI.md`). They were measured before publishing anything, with the kit compiled from each tree
and, where stated, with the published binary. Each entry says what it measured and with what. The
points carry the number of the security advisory of 0.4.4.

1. **A malformed proof made the kit exit with 101** (§711). From 0.3.0 to 0.4.3. Upstream validates
   the header of a proof with `assert!`, and the bytes are set by whoever sends it. Measured before
   the fix: of the mutations of the header, 47 of 464 panicked; of those of the shape, 85 of 521;
   and one FRI layer fewer, too. In the node, a panic with the lock taken took it to PARADA. It is a
   different path from that of 3.7, which aborted for memory. **Closed**: the fork validates what it
   reads before constructing, and the kit gives ROJO with its cause, `prueba mal formada: …`.
2. **The same `mmrSize` with a different `epochDigest` gave VERDE** (§712, §714). In 0.1.0, the
   extension envelope; since 0.2.0, also the consumption one; since 0.3.0, also the anchor in
   mode 3. With the same size, the consistency of the MMR is the identity: two heads signed at the
   same place in history, with different contents, passed. And with the same key and a larger
   `mmrSize`, nothing looked at the embedded index. **Closed**:
   `extension: mismo mmrSize y digest distinto` and `extension: el indice embebido no crece`. The
   anchor gains mode 5, which exposes those two heads: `VERDE: BIFURCACION` (§714). The witness
   groups by the index the signature attests and stops with the class `bifurcacion`.
3. **The FRI remainder was not compared with its commitment** (§713). From 0.3.0 to 0.4.3.
   `winter-fri` 0.13.1 puts the commitment of the remainder into the coin, and its verifier does not
   check it: whoever proves could choose the remainder after seeing the queries. The reach in
   soundness was not quantified. Measured: with the house options, the folding checks the remainder
   against 35 to 40 positions. **Closed** in the fork's channel and in the second implementation:
   `FriVerificationFailed(RemainderCommitmentMismatch)`.
4. **A proof admitted more than one encoding** (§711, §715). From 0.3.0 to 0.4.3. Other bytes of the
   same proof verified the same: a length with extra bytes, an extra row or group, an extra FRI
   layer, or the number of constraints with a multiple of `2^32` added. The verdict on the statement
   did not change, but the fingerprint of the bytes did not identify the proof, and the §654 guard
   in the layer was dodged (the nuance of 3.4). **Closed**: a single encoding, in the fork's reader
   and in the verifier's channel; and the layer also keeps the fingerprint of the content.
5. **The age judge worked in proportion to `2^m` before looking at the proof** (§717, §721). From
   0.3.0 to 0.4.3. `m` comes from the `nextPending` of a head that the command verifies with the key
   the head itself carries, so whoever sent the package chose it. Measured before the fix: 63.2 s
   and 531 MB with `m = 23`. **Closed**: `M_MAX` is 13, the highest `m` that fits within the two
   bounds of RFC-0007 with the production prover. A larger `m` is ROJO before reading the proof,
   `edad: m = N fuera de 3..=13`, in milliseconds. The prover applies the same cap.
6. **A rotation act with a non-canonical `mmrSize`** (§718). Only 0.4.3, the first that reads acts.
   Low severity. Its reader did not apply the bound `< p`, and the signature only binds the reduced
   element: an act signed with `mmrSize = N` gave VERDE presented with `N + p`, measured with the
   0.4.3 binary. It does not break conservation nor impersonate an operator. **Closed**:
   `actas[i]: mmrSize: … no es canonico`. The same pass takes the rule of 3.8 to the boundaries that
   were missing it: the judge, the store, the snapshot and the witness.
7. **The band and the pending claim bounded the balance to 63 bits, not 62** (§723). From 0.3.0 to
   0.4.3, read in their commits. Low severity. RFC-0017 lowered the live AIRs to 62 bits (§641,
   §719); the kit's two stayed at 63. With a leaf with a balance in `[2^62, 2^63)`, the band proof
   verified a range the leaf does not meet. Measured with the kit from before the fix: an
   `InsufficientBalance` rejection envelope with its signed head gave VERDE. The layer does not sign
   that leaf: its load and its snapshot reject it, measured. Only whoever signs a root outside the
   layer reaches it. **Closed**: the v2 of the two AIRs forces bit 62 of each segment to zero. A v1
   proof is valid with the upper bound from `2^32 - 2`, or if it is the one already published, which
   the kit recognises by its fingerprint; below, exit 3. The v2 changes the proofs that travel
   without raising `zkssl/0.4`: like the break of RFC-0017 (§826), the `zkssl/0.5` cut pays for it.
   It carries the erratum to D-3 of RFC-0017, drafted with the fix.
8. **Two colluding operators refuted a genuine conflict without anything giving it away** (§722).
   Only 0.4.3. Medium severity. Section 2.5 of `spec/PAQUETE.md` said that they could not sign each
   other a common history, and it was false. Measured before the fix, with two real nodes: a new
   ledger copies the journal of another, declares itself its heir with an act, and its head refutes
   the conflict. **Closed** in what can be seen: the `procedencia` of the acts pins the handover in
   the MMR. The eight places that require continuity —the extension, the consumption and the six
   heads of completeness— require each head to fall within the handover of its stretch, at both
   edges: `RELEVO: …`. The overlap envelope gives VERDE to a head signed outside the handover of its
   stretch. The node gains `--relevo-minimo`.

**And the layer's: a delegated issuance was resent when the state went back** (§716). The layer
verified the delegated issuance and the issuance to pending against a commitment that binds only the
state —the roots before and after, the amount, the supply and the cap—, with no counter nor record
of what was already applied. A burn, or a de-issuance, returns the root and the supply to those of
before issuing, and then the same authorisation of the custodians issued again. Measured before the
fix: issuing 250,000, burning and resending gave `Ok`, with two `Mint` entries and a single
authorisation; in a loop, three resends accepted, 750,000 redeemed and 250,000 in balance with one
authorisation. Where it was noted, the aggregate added up: it is an authorisation failure, not a
conservation one. The layer has no release, and the node does not expose the delegated issuance,
except `dev_fund` with `--dev`. **Closed**: what the custodians sign carries the layer's issuance
counter, and a resend is `AutorizacionConsumida`. An authorisation signed with the format from
before, without a counter, no longer issues: `StaleState`.

**Scope.** None of the kit's eight creates balance or spends without authorisation. Points 2 and 8
broke detection: the kit gave VERDE to a history that does not hold, or did not expose the
fabricated one. Point 3 removed a check of the STARK verifier, without measured reach. Point 7 gave
a false VERDE about what the committed leaf says. Points 1 and 5 were about availability: a 101
instead of ROJO, and a cost chosen by whoever sends. Points 4 and 6 let the bytes, or the integer,
say something other than what was signed. The layer's did issue without a new authorisation: one
from the custodians was good for more than one issuance.

**Residue**:

- **The fabricated refutation is still VERDE** (8). It proves a history of heads, not of state.
  Detection requires a head of the departed key signed after its handover, and someone who keeps it.
  Alternating the two ledgers, padding the MMR with the skipped counter or signing the handover
  again leave no evidence an envelope judges. The witness does not apply the handover. And the
  handover exposes an honest operator that rotates from a restored journal without following D-G of
  RFC-0015.
- **The reach in soundness of the FRI remainder** (3) was not quantified.
- **The CPU of the age judge is still linear in `2^m`** up to the cap (5). A ledger that goes beyond
  `2^13` pending positions has no age proof.
- **The v1 of the band is accepted from the bound** (7). The bound is derived and its boundary
  tested on integers, not searched against the circuit with a prover. No vector exercises the
  pending claim through the command with a small upper bound.
- **The content fingerprint does not persist** (4). After restarting the node, the defence is the
  fork's reader and channel. The underlying closure is the nonce of the `zkssl/0.5` train.
- **On `wasm32-wasip1` the panic net catches nothing** (1): there a panic aborts.
- **Rolling back the layer's whole store**, log included, returns the counter, and the resend is
  valid again (§716). It is the class of the bifurcation (2): the layer cannot know that its disk is
  old. Freezing, governing and recovering, which already carried their counter, do not gain the
  named cause: their resend falls with `StaleState`.
- **Upstream.** Points 1, 3 and 4 are also in winterfell 0.13.1, from which the fork comes. Its
  maintainer received the notice on 22-10-2026, with the three patches and with no agreed deadline;
  upstream they are not yet fixed.

**What should be done.**

- Update to 0.4.4.
- A VERDE from 0.4.3 or earlier on an extension, a consumption or an anchor with the same `mmrSize`,
  on an act, on an `InsufficientBalance` rejection or on an extension, a consumption or a
  completeness between two keys is judged again with 0.4.4.
- An exit 101 from an earlier release is a rejection, never a VERDE.
- Whoever rotates a node's key first rebuilds the journal (D-G of RFC-0015) and starts with
  `--relevo-minimo`.
- Whoever uses the layer's delegated issuance asks the custodians again for the authorisations
  signed with the format from before: they no longer issue.

It was prepared without publishing and is published at once with its fix, as section 5 asks, with
the amendment of §724. With this seal come out, as section 5 and `spec/rfc/PROCESO.md` §«Fallos de
seguridad» ask, the release `arqueo-verify-v0.4.4` and its security advisory, which names the
affected ones —from 0.1.0 to 0.4.3, each point with its own— and the first that fixes it, 0.4.4. The
author publishes them: ~~until the entry that registers the release (§726), the current one remains
0.4.3~~ (`doc/KIT.md` §0). ⚠️ §726: he published them after the seal, on 28-10-2026: the release
`arqueo-verify-v0.4.4`, on `95bc457`, and the advisory `GHSA-gq4j-w7rw-p96f`, of high severity and
without a CVSS vector
(<https://github.com/atoranzo/Arqueo-open-conservation-proofs-for-closed-ledgers/security/advisories/GHSA-gq4j-w7rw-p96f>),
which names as affected the releases up to 0.4.3, each point with its own, and as fixed 0.4.4; §726
registers the release with its fingerprint, and the current one is 0.4.4 (⚠️ §917: it was until
§917, which registers 0.5.0, `zkssl/0.5`; 0.4.4 is still published for the envelopes of
`zkssl/0.4`). The same act also fixes the layer, its circuits, the node and the witness, which have
no release: their notice is this section and their entries, from §711 to §723. `AUDITORIA.md` §711
to §725.

## 3.bis The protocol surface (§197-§201): what it adds and what it defends

Since August 2026 this is not only a layer: there is a wire, a node, an SDK and a public contract. A
new surface is, by definition, new risk — and it also brings two defences that did not exist before.

**Risk it adds** (above, §3.3): an open port without auth or TLS.

**What the design protects, verified in code:**

- **The keys do not leave through the API and, since §538, they do not leave as literals in the
  proof either** (§521, §523; §538). The payment goes in two phases and the proof is generated
  **locally** (`prove_send`/`prove_claim`); `Wallet::spend_key` is private and **does not even
  implement `Serialize`**: there is no possible accident through serialisation. That remains true.
  Measured in sessions 162 and 163 (20 and 21 September 2026): the house STARK proofs with
  winterfell 0.13 **did not hide their witness**. Each proof opened 42 rows of its trace in the
  clear, and what went in those rows was published: the spending key in the send, the claim and the
  pledge —and in the v2 send, also the balance, the amount, the salt, the `leaf_salt` and the `X`—;
  the balance and the `leaf_salt` in the band; the issuer in the age, when all the pendings are from
  the same one; and the salt in the two portable envelopes, with the `X` and the exact amount in the
  claim one and the `delta` and the `refund_id` in the payment one. The rule that explained it was
  measured with two falsifiers (§523): **every value that goes in a constant column of the trace
  came out as a literal**. That is why the audit —one circuit for the three modes— published the
  spending key and the exact balance; the burn, the key; the send, the receiver's identity; and
  **each delegated authorisation, the key of its custodian**: after a single delegated issuance,
  freeze or recovery, the node had the keys of two custodians and could authorise the next one.
  Governance used the same circuit: deduced, not measured. Conservation did not fall; who could move
  the supply did. Soundness did not fall either: each proof kept proving what it said. What fell was
  the hiding, and with it the custody of the keys —the spending one and the custodian ones— against
  whoever saw a proof. **Since §538 (RFC-0009 E3b-2, `zkssl/0.4`) the prover hides the witness**:
  the 23 provers with a row in the table of RFC-0009 produce with random rows and a blinded quotient
  on a fork of winterfell 0.13.1 (`crates/winter-*`), and what is promised is what the E2 suite
  measures in each canon: **a proof that verifies demonstrates the transition it states and
  publishes as a literal no value of a constant column of its trace** (zero literals in 100 cells
  and 22 rows). What is not promised: that nothing is deduced from what is opened; the bound lives
  in RFC-0009 (D-I: 44 openings per column against T ≥ 64 random rows) and neither the fork nor the
  construction is audited (H7). ⚠️ §916: since §916 more is promised, under named hypotheses (the
  note of §916, below); T ≥ 128 since §914. Whoever sees a proof is still the node, which receives
  them, and whoever reads a published vector or package: those of `spec/vectors/` of 0.4 carry no
  literal of the witness; those of 0.3, preserved under `spec/vectors/0.3/`, keep carrying what they
  carried, and from the 0.3 rejection catalogue two sandbox spending keys can still be derived. The
  API rule —the key does not travel— holds since §538 in what the suite measures; between §521 and
  §538 it did not hold, and the entries say so. ⚠️ **§901 (A4 of the HVZK; RFC-0009 D-AM):** what
  lies behind «it is not promised that nothing is deduced», measured and written: the hiding
  argument of the fork with its simulator (`doc/ocultacion-hvzk.md`, §897), the condition it lacked,
  measured by geometry (§898), and the mask of the columns of each AIR, measured per family (§900).
  With the construction of `zkssl/0.4` that sufficient condition holds in sixteen of the
  twenty-three families —the six of T = 1024, the nine of T = 512 and solvency— and not in seven:
  MerkleAir and FrozenClimbAir, which need a DEEP mask in E (entry 133 of `BACKLOG.md`, of a future
  cut), and the five with 64 real rows —the two thresholds, the age and the two refunds—, for which
  their random rows do not cover what the proof touches (entry 131; ⚠️ §905: the fine count of entry
  135, measured, does not rescue them today, and with the DEEP mask in E it would cover the joint
  threshold without more rows). What could be promised for the sixteen —simulatable hiding without
  the witness, statistical in the programmable random oracle model, under named hypotheses, among
  them the fresh salt and the non-repeated seeds of §3.11— is written in D-AM with its words and
  **is not promised**: it is the author's, and the argument has no outside review (H7). The promise
  of this section does not change. ⚠️ §907: the first review from outside the session —by an agent,
  not a person— found no false step; it left two lemmas written and the count of the argument's
  advantage redone, with what depends on the real positions proven in part and measured in the rest,
  and with the bound on the challenges over |F| = 2^64, not over |E|. Still nothing more is
  promised.
  ⚠️ **§915 (the `zkssl/0.5` cut, §914):** the production construction is, since the cut, that of
  the two random columns paired in DEEP and T ≥ 128 in the twenty-three families: the sufficient
  condition that was missing is a theorem (L9) and the random rows cover what the proof touches in
  all of them, measured in each canon; what could be promised for the twenty-three is written in
  D-AM of RFC-0009 (its draft of §915) and **is not promised**: it is the author's, and the argument
  awaits outside review (entry 136). The promise of this section does not change.
  ⚠️ **§916 (accepted by the author on 29-10-2026, «Aceptar todo» [accept all]):** the promise of
  this section changes for the first time since §538. What is promised since §916, besides the
  census: **for the twenty-three families with the construction of `zkssl/0.5`, the view of the
  honest verifier is simulated without the witness** —simulatable hiding, statistical in the
  programmable random oracle model—, with advantage less than 2^{−60} + 2^{−128} +
  Q·2^{−256} + |E|^{−(N−1)}, under the named hypotheses of D-AM of RFC-0009 (its D-A, §916;
  `doc/ocultacion-hvzk.md`, section 18): H3, z outside F and H; H7, Blake3 as a programmable random
  oracle; H8, the fresh salt, and H9, no repeated seed (§3.11); and under an argument reviewed
  adversarially by the session and by an agent, not by an outside person: entry 136 remains open and
  H7 is kept; neither the fork nor the construction is audited. What is still not promised: nothing
  against a quantum adversary, nothing about side channels, nothing outside those hypotheses.
- **The wire rejects what it does not understand**: DTOs with `deny_unknown_fields`, canonical hex,
  fixed-width digests.
- **The conformance vectors are a defence, not just documentation.** A second implementation that
  diverged —in the leaf, in the order, in the chaining— **would not pass `conformance --check`**:
  the divergence becomes detectable field by field instead of silent.
- **The wallet at rest** (§199) uses the same construction as the ledger with **its own domain**,
  and a test requires that the ledger's key does **not** open the keystore. Since §702 (RFC-0001)
  the key of both is derived with **Argon2id** (RFC 9106: 64 MiB, three passes, four lanes) and a
  salt of 16 bytes per file, kept in the clear next to what is encrypted; a `zkssl-keystore/1`
  keystore or an encrypted ledger from before, opened with their passphrase, are migrated to
  version 2. ⚠️ **Corrected in §702**: until then this paragraph said «Its KDF is SHA-256, which is
  **not** a password derivation function: a weak password can be forced», and it was so: without
  salt or cost, each offline attempt cost one SHA-256. What still stands: a weak password is still
  weak —the cost makes each attempt more expensive, it does not reduce the attempts—, and whoever
  has the process running has the derived key in memory. ⚠️ **Corrected in §812**: until here this
  paragraph did not say what remained on the disk, and a migrated ledger kept in its files what was
  sealed with the v1 key, which could be forced at the speed of a hash (§3.14). Since §812 the
  migration does not leave it, and `SovereignLayer::recifrar` changes the passphrase and the salt of
  the whole ledger.

---

## 3.ter Against the systems that already exist — an honest comparison

> This table exists because a serious reader is going to make it mentally anyway. It is made here,
> with both directions in place: **there are three axes where this design is ahead and four where
> the others crush it**. The outside data carry a date and a source at the foot; our own, a
> reference to `AUDITORIA.md`.

### Where this design is ahead

**1. Post-quantum spending authority — today, not on a roadmap.**

| system | what authorises a spend | status against Shor |
|---|---|---|
| Bitcoin | ECDSA / Schnorr (secp256k1) | vulnerable. BIP-360 (P2MR) was merged into the BIPs repository on 11-02-2026 — a documentation milestone, **not activation**; BIP-361 (retirement of legacy signatures) is from 14-04-2026 and controversial. ~6-7 M BTC (25-33 % of the supply) have their public key already exposed |
| Ethereum | ECDSA in accounts, BLS in validators | vulnerable. The Foundation created a PQ team in January 2026; `leanXMSS` (hash signatures) + `leanVM` in development, with a declared horizon towards 2029 |
| Zcash | signatures over curves (Pallas/Vesta) | vulnerable |
| Solana | Ed25519 | vulnerable |
| **Arqueo** | **knowledge of a preimage**: identity, leaf salt and authority derive from the key **by hash** (§117) | **there is no classical signature on the payment path**, and STARK/FRI only uses hashes: there is no curve to break |

⚠️ **The reservation that has to be made**: «post-quantum» here means *without curve assumptions*,
not *invulnerable*, and the level against a quantum adversary **is not measured** (§3.11). What is
measured is classical: 127 conjectured bits —112.8-115.8 depending on the circuit with the DEEP
term, which the formula does not discount, and less in the age— and, provable, 59 in UDR and 80-86
in LDR depending on the circuit, less in the age with high `m` (§3.11). The ceiling of **63 bits of
soundness** that this project measured and published (finding 3) is that of a configuration without
field extension; the production one uses the quadratic. A system with thousands of validators and
years of production is still, today, **safer in practice** than an unaudited one. ⚠️ **Corrected in
§698**: until then this paragraph said that «its default configuration has a ceiling of 63 bits of
soundness without field extension», and did not say that the quantum level is not measured.
⚠️ **§838**: «without curve assumptions» holds for the payment path and for what is signed, not for
the transport. The TLS of the cli and of the SDK negotiates a classical ECDHE and verifies ECDSA
P-256/P-384 and Ed25519 signatures of certificates and of the handshake, and the view key travels
under it (section 3.3): it is the §106 exception for transport, declared in §838 of `AUDITORIA.md`.

**2. No trusted ceremony — and never having had one.** Zcash removed it with Halo 2 in Orchard (May
2022), but Sprout and Sapling were born from ceremonies and those legacy pools exist. Here the
startup **generates no keys**: 1.47-1.85 ms measured (⚠️ §841: it said «0.67 ms»; re-measured in
three runs of `metrics_of_the_layer` on §840, in the table of `ARQUITECTURA.md` «Métricas de la
capa, medidas» [metrics of the layer, measured]), there is no secret to destroy nor participant to
trust. *Honest nuance: this does not differentiate against Bitcoin or Solana, which do not use
SNARKs at all — it differentiates against the Groth16/PLONK-KZG family, which was the one
discarded.*

**3. Demonstrable supervision with BOUNDED disclosure.** Zcash's viewing key gives **read access**
to whoever holds it: it is all-or-nothing over what it covers. Here the holder produces a **band**
proof —«I am between X and Y»— that the supervisor verifies **without access to the ledger** and
without learning the balance; and the issuance cap is **bound in the circuit**, not in a client's
policy. Bitcoin, Ethereum L1 and Solana offer no content privacy at all, so the comparison on this
axis only applies against Zcash.

### Where the others crush this project

| axis | they | Arqueo |
|---|---|---|
| **Decentralisation** | thousands of independent validators/miners | **ONE node, one operator**. It sees the state, orders, can censor |
| **Throughput** | Solana in the thousands of TPS; Bitcoin and Ethereum an order far above this one | **1.5-1.9 TPS** measured (§123) |
| **Maturity** | years in production, repeated audits, bug bounties | **zero external audits**, research prototype |
| **Proof size** | Groth16: 192 B | **72,382-84,244 B** per send or claim proof, hidden, the band a test binds since §538 (a payment is two: 145,953-167,967 B); the size of the other proofs is bound by no band — the price of depending on nobody. Before hiding, the send and the claim measured 64.6 and 65.3 KB (§218). The 39,048 B of the comparative tables (⚠️ §806: it said «The 36.7 KB»; re-measured on 2026-10-14 on `b6491ef`, in `FIVE_BACKENDS.md` §4) are from the comparison circuit, with other options and without hiding |

**And a lesson this project borrows, not lends**: Orchard's under-constraining bug (June 2026)
occurred in the class that here appears as **§3.1, highest priority**. It is not cited to point at
anyone: it is cited because **it validates the order of priorities** of this repository and because
its remedy —formal verification— is the ladder that here is half climbed. When an audited and mature
system takes four years to see one of these, the right conclusion is not «it will not happen to us»:
it is **«our circuits are not proven either, and we say so»**.

---

## 4. Scope

**In scope**: soundness of circuits and of the transition function; spending authorisation;
correctness of the verifier; declared privacy properties; integrity of the chained log.

**Out of scope today**, by design or immaturity: distributed consensus; resistance to a malicious
operator without external observers; recovery from loss of the node; metadata analysis; economic
security of a token —the project **requires none**—.

## 5. How to report a vulnerability

**Soundness failures** —creating balance, spending without authorisation, or making the verifier
accept an invalid transition— report them **in private** before disclosing them.

Use **«Report a vulnerability»** in the **Security** tab of this repository. It opens a private
advisory that only the maintainer sees; it exposes no email address.

**Non-sensitive problems** —documentation, limits already listed, improvements—: a normal *issue*.

**What to include**: description of the failure, the logic that produces it and, if possible, a
minimal case that demonstrates it —for example, an invalid trace that the verifier accepts—.

**What to expect**: a research project without a dedicated team. There are no response commitments
or bounties. Responsible disclosure is appreciated and credit is given to whoever wishes it.

**What comes out when it is fixed**: each failure fixed in the published kit —the `arqueo-verify-v*`
releases— comes out with its **security advisory**, a *GitHub Security Advisory* of this repository
that names the affected versions of the kit and the first that fixes it, with the credit of whoever
reported it if they wish.

**How it is fixed**: in private —in the temporary private fork created from the draft of the
advisory or in a private repository of the maintainer (also a clone without a remote), never in a
public branch, PR or *issue*— and it is published at once: the code, the vectors, the RFC or the
amendment with its stage already built, the `AUDITORIA.md` entry, the release and the advisory. What
is prepared in a private repository enters `main` by fast-forward, one commit per fix with its
entry, just before the release and the advisory, if it carries them. Old vectors are not rewritten:
if the fix changes the wire, the version goes up and the old ones are kept under theirs (rule 2 of
`spec/rfc/PROCESO.md`). It is written down in `spec/rfc/PROCESO.md` §«Fallos de seguridad», which
also says why the RFC of a security fix does not go through the tree first, and that the draft of a
report to another project does not enter it.

**No** categorical statements about the security of the system will be made in response to a report:
it will be fixed, or the limit will be documented.

---

## 6. Consensus is the last intermediary: the case for quantum money

It is worth ending a security policy by answering the question that orders all of it: **whom do we
still have to trust, and why?**

**Each of these systems is a machine for eliminating intermediaries.** Bitcoin removed the issuing
bank. Zcash removed the observer. Ethereum removed the host of the application. This project removed
the participants of a setup ceremony —those who, colluding, could create money without leaving a
trace—. All of them removed someone. **None removed the one who orders.**

Because that is what consensus is, said without liturgy: **an intermediary of order, made plural and
made expensive.** Miners and validators are not the absence of a third party; they are a third party
spread among many, which keeps deciding what enters, in what position and what stays out. From there
come censorship, reordering and the extraction of value by ordering (MEV): they are not pathologies
of consensus, they are **its surface**. A system with a thousand validators makes corrupting the
order a thousand times more expensive, and has exactly the same kind of power concentrated in the
function.

This repository is unusually honest about this because it cannot disguise it: **it has a single
orderer of operations, and it is written in the first line of every document.** A single node is not
a worse consensus; it is the same intermediary, without the make-up of the number. And having it in
plain sight allows asking the right question, which is not *«is this decentralised?»* but **«what
would it take for this to need nobody?»**.

**Why does the one who orders exist, to begin with?** Because of a physical property of classical
information: **a bit can be copied.** Double spending is a corollary of clonability, and consensus
is the patch humanity found for a limitation of physics, not of money. The whole edifice —blocks,
quorums, finality, penalties— exists to decide which of two identical copies counts.

**Quantum money attacks the root instead of the symptom.** A token held in a quantum state cannot be
duplicated: not because it is forbidden, but because the **no-cloning theorem** prevents it. If the
object is not copyable, **there is no double spend to order**, and the intermediary of order stops
having a function. It is the only known proposal that does not spread the last intermediary among
more hands, but **suppresses it**.

**And what it lacks has to be said, or this would be propaganda.** There is no deployable
construction today: it requires quantum memory with long coherence; Wiesner's original scheme forces
going back to the issuer to verify —swapping the orderer for a central verifier, which is not
obviously better—; and several public-key quantum money proposals have been broken. **It is a
horizon, not a construction plan.** This project points to it in `PRINCIPIOS.md` §6.bis and nowhere
claims to be quantum money: the README separates it explicitly.

**So where, exactly, does this project stand?**

Not as quantum money, but as **the classical approximation from which everything else has already
been removed**. And that has a concrete technical consequence, which is the thesis of this document:

> **If quantum money arrived tomorrow, almost nothing of this stack would have to be redone.** The
> spending authority is not a signature that Shor breaks: it is knowledge of a preimage. Soundness
> does not rest on curves or pairings: only on hash functions. The supply is not guarded by a
> client's policy: it is bound in the circuit and attested in a chained log. Compliance does not
> require opening the ledger: it is bounded disclosure that the supervisor verifies without access.
> **The only thing that would have to be replaced is the one who orders — and that is exactly what
> physics would take away.**

That is the position, said in one sentence: **this system is not decentralised, and its residual
trust, measured and written, comes down to two residues that are the same shadow** —the order and
the completeness of history, and that the operator sees the state (§2.bis)—. Both are the orderer of
operations. A system whose remaining trust has **a name, a measure and a designed attack** is a
system that knows what it lacks in order to need nobody; one that calls itself decentralised and
spreads the orderer among thousands knows less about itself.

As long as quantum money does not exist, there remain the classical closures, all written and none
promised as done: attested heads and acknowledgement (§121), external anchoring of roots (RFC-0012,
`doc/ANCLAJE_EXTERNO.md`; built, pending deployment), consensus or replication. Each one **makes the
last intermediary more expensive**. None eliminates it.

**Eliminating it is not a problem of cryptography. It is a problem of physics** — and that is why it
is worth there being, already, a whole stack whose only remaining obstacle is that one.

---

### External sources cited in this document

Outside data verified on 06-08-2026; **re-verify them**, this field moves fast:

- Under-constraining bug in `halo2_gadgets` / Zcash's Orchard pool, disclosed in June 2026
  (emergency soft fork on 2 June; NU6.2 on 3 June, block 3,364,600) and the formal verification
  effort of the new pool.
- Halo 2 and the removal of the ceremony in Orchard (NU5, May 2022); earlier ceremonies of Sprout
  (2016) and Sapling (2018) — Electric Coin Co.
- Bitcoin: BIP-360 (Pay-to-Merkle-Root) merged on 11-02-2026; BIP-361 («Post Quantum Migration and
  Legacy Signature Sunset»), 14-04-2026; estimates of 6-7 M BTC with an exposed public key.
- Ethereum: the Foundation's post-quantum security team (January 2026), `leanXMSS`/`leanVM`, roadmap
  published in February 2026 and a declared horizon towards 2029 — `pq.ethereum.org`.
- Google Quantum AI (March 2026): estimate of ~1,200 logical qubits to break 256-bit curves, ~20×
  fewer than previous estimates.

---

*It reflects the state known at the time of writing, without external audit. When this document and
the code stop matching, correct the document.*
