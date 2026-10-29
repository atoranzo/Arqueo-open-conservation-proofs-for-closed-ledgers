# Arqueo — principles (formerly ZK-Sovereign Settlement Layer, ZK-SSL)

> **Español:** [`PRINCIPIOS.md`](./PRINCIPIOS.md) is the original of this page, in Spanish; it
> prevails wherever the two differ. This translation is of commit `7766efb` (audit entry §917) and
> was produced by the project's assistant, a Claude Code session, in audit entry §918 (`GENAI.md`):
> it is a generated artefact, not text written by the author. Line citations of the form `file:N`
> refer to the Spanish files.

**Principles and architecture document — version checked against the
implementation**

---

## Note on this version

This document starts from the project's original statement and
**checks it against what has been built and measured**. The principles
are kept whole; the technical claims have been replaced by verifiable
results.

Each component carries its real state:

| | |
|---|---|
| ✅ | Implemented and verified with tests |
| ⚠️ | Partial — what is missing is stated |
| ❌ | Not implemented |

All figures come from the project's own measurements on one same
machine, in release mode. None is quoted from the literature.

---

## 0. First of all: what this is NOT yet

The project's founding principle is **to remove the dependence on
centralized trusted intermediaries**. One has been removed, and it
matters: the participants of a setup ceremony, who in Groth16 or
PLONK-KZG could collude and forge proofs —creating money without anyone
ever detecting it—.

**But the current implementation is a single node.** Whoever operates
it sees every balance, orders the operations, can censor, and is a
single point of failure.

**That operator is exactly the centralized trusted intermediary the
principle points at.** It is still there.

### What is proven and what is trusted

| Claim | Proven mathematically? |
|---|---|
| This transfer conserves the money | ✅ |
| The balance of this account is X | ✅ |
| No money has been created outside the supply | ✅ |
| Nobody spends without being the holder | ✅ |
| Nobody spends twice | ✅ |
| **This is the current state of the system** | ❌ Trust in the operator |
| **These are all the operations there were** | ❌ Nothing prevents omitting |

**The transitions are proven. The state and the completeness of the
history are not.**

⚠️ **And signing the heads does not change that table yet** (§236, §238).
The signature binds a head to whoever issued it; for that to be of any
use **someone has to keep it and compare it**, and today there is
neither declared custody of the key nor a single witness. What is built
is the first link of five, not the property.

**What this is**: a demonstration that the cryptographic properties of
a sovereign settlement are buildable and measurable. **What it is not**:
a decentralized layer.

This section goes first, and not in a list of limitations, because
burying it would be selling a property while hiding its opposite.

---

## 1. Introduction

In a context of geopolitical fragmentation, saturation of digital
surveillance and erosion of trust in centralized institutions,
infrastructure is needed that reconciles **deep privacy with provable
regulatory compliance**.

That is still the statement. What this project contributes is
**evidence that that reconciliation is technically buildable**, with
the properties verified one by one and the cost of each one measured.

---

## 2. Problem statement

Current systems have structural shortcomings:

- **Centralized**: vulnerable to censorship, single points of failure
  and abuse of power.
- **Classic blockchain**: excessive transparency, limited scalability,
  hard institutional adoption.
- **Traditional financial**: slow settlement, intermediation costs,
  dependence on institutional trust.

**Added after the implementation**: there is a fourth problem that was
not in the original statement and that turned out to be the most
decisive.

> **Most ZK systems demand a trust ceremony.** Groth16 and PLONK-KZG
> need participants who generate parameters and destroy a secret. If
> they collude, they can create money without anyone ever detecting it.
> It is an external, permanent and unauditable dependence —and it is
> incompatible with "sovereignty".

That observation, which only becomes evident when implementing, is the
one that determined the whole architecture.

---

## 3. Protocol objectives

| Objective | State |
|---|---|
| Sovereignty over data and capital | ⚠️ Cryptographic, yes; against the operator, no |
| Settlement with cryptographic finality | ✅ |
| Selective privacy compatible with regulation | ✅ |
| Neutral, minimalist and resilient base layer | ⚠️ See section 0 |
| Orderly transition from traditional infrastructures | ❌ |

---

## 4. Technical architecture

### 4.1 Components

**ZK Core Engine** — ⚠️
The same circuit was implemented in **five paradigms** (Groth16,
Halo2/IPA, STARK/FRI, PLONK/KZG, Nova/folding) and their trade-offs
were measured. Recursive aggregation **does not exist**: only folding
(Nova) was tried as a proof of concept, and it was discarded for the
layer for reintroducing a ceremony and losing quantum resistance.

**Sovereign Settlement Layer** — ✅
`crates/zk-ssl`. Keeps the state, chains roots, applies operations.
**565 tests** (9 ignored, declared) in `crates/zk-ssl`.

**Protocol surface** — ✅ *(added: §197-§199)*
The layer is no longer alone: `zk-ssl-wire` (wire format),
`zk-ssl-node` (JSON-RPC reference node), `zk-ssl-sdk` (holder's side,
LOCAL proving, wallet encrypted at rest) and `zk-ssl-cli`
(sandbox, tracer and conformance), with a normative `spec/RPC.md`,
`spec/openrpc.json` generated from the code and **conformance vectors**
that a second implementation must reproduce field by field.

**Structural cryptographic authentication** — ⚠️
There are spending keys and cryptographically derived identities
(`pk = H(DOMAIN, sk)`), with domain separation. **There are no DIDs and
no SSI.**

**Execution Environment** — ⚠️
The layer consumes **pacs.008** and returns **pacs.002** with the proof
attached, translating its errors to real ISO reason codes
(`AM04`, `AC01`, `AG01`...). It is not an XML parser nor does it cover
the whole standard, and there is no virtual machine.

**Privacy & Compliance Module** — ✅ the statement, ✅ no literal of the witness since §538
Selective disclosure with three modes: exact balance, minimum, and
**band** ("I am between X and Y"). The holder produces the proof; the
supervisor verifies it **without access to the ledger**. Since §538
(RFC-0009 E3b-2) the proof hides its witness: the audit one —a single
circuit for the three modes— does not publish the exact balance or the
spending key literally; between §523 and §538 it published them. What
is not promised: that nothing can be deduced from what is opened
(RFC-0009 D-A); the construction is not audited (H7). ⚠️ §916: since
§916 it is also promised, for the twenty-three families with
`zkssl/0.5`, that the proof's view is simulated without the witness,
under named hypotheses (RFC-0009 D-A); H7 is kept.

**Governance Layer** — ✅
Two-level hierarchy: the **custodians** issue and recover accounts; the
**governance set** can change the custodians. Both require 2-of-N, both
with a public counter of interventions.

The regulatory limit and the issuance cap are still immutable.

⚠️ **The governance set is immutable**: if it is compromised, the only
way out is to create a new ledger. The circularity is not solved, it is
moved to keys that are used almost never and can be kept offline.

### 4.2 Real operating flow

```rust
let layer = SovereignLayer::open("./ledger", custodios, gobernanza, limite, tope, max_cuentas)?;

// Emitir EXIGE DOS custodios, demostrados en circuito.
// Un pago son DOS fases, y en NINGUNA la clave llega a la capa:
// la capa entrega materiales (caminos y raices, datos publicos) y
// recibe pruebas que verifica.

// FASE 1 — el pagador envia.
let m = layer.send_materials(alice, id_de_bob, 250_000, aleatorio)?;
let envio = client::prove_send(&m, clave_alice, proof_options())?;  // LOCAL
layer.apply_send(&envio, alice, &estado_alice, 250_000)?;

// FASE 2 — el receptor cobra.
let m = layer.claim_materials(bob, &envio.notice)?;
let cobro = client::prove_claim(&m, clave_bob, proof_options())?;   // LOCAL
layer.apply_claim(&cobro, bob, &estado_bob, &envio.notice)?;

let d = layer.audit(clave_alice, alice, &estado_alice, 900_000, 1_100_000)?;
verify_audit(&d)?;                                       // el supervisor, sin la capa
```

Generating the proof and applying it are **separated on purpose**: it
lets whoever produces the proof and whoever accepts it be different
parties — and it is what makes **no spending key ever reach the layer**.
`a_whole_payment_without_giving_any_key_to_the_layer` proves it.

⚠️ **An earlier version of this block used the WITHDRAWN API**
(`layer.send(sk_alice, …)`, `layer.mint(issuer_key, …)`): the spending
key entering the layer, just the opposite of what this document holds.
Corrected in §203; the error is recorded instead of erased.

⚠️ **And this block showed `layer.audit` with FOUR arguments** when the
signature asks for FIVE (`audit.rs`): the holder's state was missing, which
is just what the layer does not keep, and the example did not compile.
Corrected in §363; the error is recorded instead of erased.

### 4.3 If the receiver never claims: the refund

**Floor of the refund: Δ_min = 13 months** — ❌ *(added: §779)*
No payment may carry a shorter refund period: it is the line that
`AUDITORIA.md` §119.8 wrote for this document, and it had not arrived.
After a period Δ, the sender —and only the sender, with their key—
recovers an unclaimed send (§119, §178-§181); on the v2 path (RFC-0003)
the Δ goes inside the commitment and the sender chooses it, Δ=∞ included,
with a measured limit: the field reduces `u64::MAX` to `2^32 - 2`, so the
sender can open their own «never» after `2^32 - 2` entries of the log
(§501). The floor is policy, with precedent in SEPA's *recall* period and
the imperfect analogy declared: the recall requires the receiver's consent
and this is unilateral by abandonment, which argues for a long floor,
never a short one. A Δ≈0 would be a payment that was never a promise.

❌ **Not implemented.** `apply_refund` compares the elapsed time with the Δ
of the envelope and demands no minimum at all
—`un_refund_v2_tras_delta_devuelve_al_emisor` refunds with `delta = 1`—;
the v1 path still has a global period set by the operator
(`set_refund_ttl`); and the clock is `log.len()`, entries of the log and
not months, so «13 months» has no translation in the tree yet. The policy
is also still pending T3b, its acceptance test (§119.7, §128.1; entry 12
of `BACKLOG.md`).

---

## 5. Technological foundations

### The decision that defines the architecture

**Groth16 was discarded despite being the fastest and having proofs
380-440 times smaller** than the send and claim ones (192 B against
72,382-84,244 B — §538).

The reason was not technical but one of coherence: it requires a trust
ceremony. Without a ceremony, Halo2/IPA and STARK/FRI remain; of the
two, STARK wins in everything except size and is the only post-quantum
one.

**It is the only decision in the project taken against the performance
numbers.**

### Measured comparison

| | Groth16 | Halo2/IPA | **STARK/FRI** | PLONK/KZG |
|---|---|---|---|---|
| Ceremony | Per circuit | None | **None** | Universal |
| Setup | 438 ms | 16.3 s | **none** | 26.3 s + 12.8 s |
| Generation | 422 ms | 4.86 s | **39 ms** | 6.85 s |
| Verification | 5 ms | 91 ms | **1 ms** | 8 ms |
| Size | **192 B** | 4,096 B | 39,048 B (⚠️ §806: it said «36.7 KB»; remeasured on 2026-10-14 on `b6491ef`, in `FIVE_BACKENDS.md` §4) | 1,008 B |
| Post-quantum | No | No | **Yes** | No |

**Nova/folding**, measured apart for being of a different nature:
**~250 ms per transaction, constant**; 1.84 s closing, amortizable.
Discarded for the layer: it uses curves and requires a ceremony.

---

## 6. Comparison with existing projects

**Correction with respect to the original statement**, which claimed
alignment with Sovereign SDK and zkSync without checking it.

| Project | Real relation |
|---|---|
| **Zcash** | In production since 2016, a more complete design than this one |
| **Aztec, Aleo, Miden** | Teams of dozens of people, years of work |
| **zk-Bench** (UCL) | **Covers the comparative axis with academic rigor** |
| **ZK-ACE** | Nullifiers on two backends |
| **Drex, mBridge, Project Hamilton** | Institutional consortia with backing |

**What is distinctive here is narrower than the original document
suggested**: not the performance comparison —which zk-Bench already
covers— but the **design findings that emerge when porting a complete
application** between paradigms. That is not discovered by measuring
SHA-256.

---

### 6.bis — The «last intermediary» and quantum money

There is a recent argument this protocol takes seriously: *Bitcoin
removed the banks but not trust — it moved it to consensus, which is
the last intermediary*. Quantum money (no-cloning theorem) promises to
remove it by physics; its memory does not exist at scale yet.

Arqueo is the classical attempt in the same direction, with today's
tools, and it is worth saying precisely what it removes and what it
does not:

**Removed**: the setup ceremony (STARK/FRI — there is no secret
retainable by third parties); the custodian keys in the node for the
operations migrated to the delegated path; and, since entry 50, the
recovery of balances by dictionary over the paths the protocol hands
out (wrapped leaf, §117).

**Reduced but present**: the operator. It sees the state —privacy is
against third parties—, orders, and could omit history. That residue
has a designed name and attack: attested heads and **acknowledgement**
(§121, `doc/CONFIANZA_RESIDUAL.md`), which turn «trust the operator»
into «the operator cannot lie without leaving fail-stop evidence».

*The complete argument —why consensus is an intermediary of order and
not its absence, and what of this stack would survive quantum money—
is developed in [`SECURITY.md`](./SECURITY_EN.md) §6.*

**Outside the classical domain**: the physical guarantee. A STARK is
computational soundness with a wide, post-quantum margin (XMSS
evaluation in `doc/xmss-evaluacion.md`), not a theorem of nature. When
quantum memory matures, quantum money will be able to retire the last
intermediary entirely; until then, this design seeks to leave it
**minimal, measured and with evidence** — which is the most that honest
classical cryptography can promise.

## 7. Real contributions

Eight findings, none present in the materials that compare
paradigms:

**1. AIR lacks copy constraints.** When porting the state update to
STARK a hole appeared that does not exist in the other paradigms:
nothing forces the two climbs of the tree to use the same siblings. It
is **silent** — honest witnesses would never reveal it. It forced the
design of a lockstep pattern.

**2. Goldilocks is too narrow for identities.** One element is 64 bits:
a single-element identity would admit a collision in 2³² by the birthday
paradox, and that is why the identity is the full digest of four
elements, 256 bits. In BLS12-381 (255 bits) one element is enough. The
narrowness has a second consequence that widening the identity does not
fix —it prevents finding *another* key with the same identity, not
finding *the* key—: the space of a single-element spending key is 2⁶⁴,
and since the identity is public —the payer needs it to address—,
exhausting it offline costs 2⁶³, 2.38 million core-years measured on a
CPU without optimizing the attack, which is a loose upper bound (§82).
`open_account_wide` opens accounts with four-element keys (§97), but
the single-element path, `open_account`, is still alive —marked
`#[deprecated]` (§98.1); adoption is opt-in (§102)— and whoever does not
rotate stays at 64 bits (§90.4); rotating requires the authorization of
two custodians (§98.4). ⚠️ §778: until then it said «One element is 64
bits: collision in 2³². In BLS12-381 (255 bits) the problem does not
exist», as if the collision were still open and were the only
consequence: the identity was already four elements (§82.1), entry 15
of the backlog strikes it out, and what is alive is the exhaustive
search for the key, not a collision.

**3. A 63-bit soundness ceiling** in STARK over Goldilocks without
field extension. That configuration **is not comparable** with the ~128
bits of the others; the layer uses the quadratic extension.

**4. Gap between conjectured and provable security**: in the comparison
circuit, with 32 queries and no grinding, 127 conjectured bits coexist
with 29 (UDR) and 63 (LDR) provable, and closing it costs 127,274 B
instead of 39,048 (⚠️ §806: it said «125.6 KB instead of 36.7»;
remeasured on 2026-10-14 on `b6491ef`, in `FIVE_BACKENDS.md` §4). With
the production options —42 queries, blowup 16,
grinding 21— and over the hidden proofs: 127 conjectured (112.8-115.8
depending on the circuit with the DEEP term, which the formula does not
discount, §708), 59 in UDR and 80-86 in LDR depending on the circuit,
less in the age one with a high `m`; tests pin it (§697) and the table
is in `SECURITY.md` §3.11. Against a quantum adversary, the level is
not measured. ⚠️ **Corrected in §698**: until then it said «127 bits
coexist with 29-63», without saying of which circuit or with which
options; 3, that the configuration without extension was «the default
configuration»; and section 5, that Groth16 had proofs «320 times
smaller» (192 bytes against ~65 KB, §130), with the sizes from before
hiding.

**5. PLONK-KZG turned out to be the slowest generator** of the four,
16-22 times slower than Groth16. *Nuance: part may be due to the
implementation, and these data do not allow separating it.*

**6. Only `nova-snark` prevents single-party setup in code.** The other
four libraries allow it and rely on the documentation.

**7. The PLONK-KZG ecosystem in Rust is closed vertical stacks.** Six
paths investigated, five broken.

**8. A zkVM is not comparable on equal terms.** RISC Zero was evaluated
as a sixth paradigm: it uses STARK over Goldilocks —the same system and
field as the chosen backend— but requires an external toolchain to
compile the guest program, which breaks the criterion that discarded
three other libraries. The concrete measure of what generality costs:
**3 dependencies against 349**. It is documented instead of
implemented.

---

## 8. Roadmap

**Correction of the original**, which foresaw testnet in 2026-2027 and
mainnet in 2027-2028. That is not reachable with the current resources
and saying it would fail the principle of transparency.

### What is done

- Five paradigms implemented and measured.
- Settlement layer with a complete monetary cycle, persistence, audit
  and integrity verification.
- **2104 tests in the seal gate** —565 of the layer, 492 of circuits,
  **244 of the node**, 34 of the ceremony, 17 of settlement, **225 of
  the independent verifier**, **175 of the witness**, and those of the
  SDK, the wire, the ISO bridge, the middle tree and the winterfell
  fork—, **2244 counting the pins of `--largo` and `--completo`**, and
  **1364 declared** (⚠️ §865: from §497; today `tools/check_tests.py`
  counts them, with another rule). 0 failures and 21 warnings **pinned**
  (they do not grow). Every security property with a discriminating
  test.

  ⚠️ Here it said **539** with the breakdown «297 + 242», and neither
  figure was true any more. **They are not remembered: they are run** —
  `bash tools/canon.sh --sello` fails if the tree stops giving them.

  ⚠️ §795: and it said **24** warnings, the sum of the pins of the table
  (11 + 3 + 10). The 3 of `plonk-experiment` were **a single warning**
  —an unused `use`, removed— and cargo's two summary lines, because the
  pin counts the lines that start with `warning`, not the warnings. Its
  pin goes down to 0; the 11 of the ceremony row and the 10 of the
  `zk-core` row remain.
- **From implementation to protocol** (§197-§199): specification,
  generated OpenRPC, conformance vectors as a permanent gate, RFC
  process, reference node and SDK with keystore.
- **Executable SPEC of the payment core** (§195-§196): an interpreter
  that reproduces the circuit's output byte for byte, mutants included,
  and a cell census with **0 without owner**.

- **First link of opposability** (§234-§236): signature index guardian
  —a counter with `fsync` before signing, which **refuses to start if
  its `fsync` does not persist** and, since §709, if another process
  holds its counter; what that lock does not cover, in `SECURITY.md`
  §2— and epoch head signer with XMSS, its own domain and explicit
  format version.

  ⚠️ **And with it, the tree's first cryptographic dependency**:
  `xmss 0.1.0-pre.0`, a pre-release **without an independent audit** by
  the crate's own declaration. Chosen among four candidates with five
  criteria (`doc/xmss-evaluacion.md`).

  ⚠️ **Signing is not having opposable evidence.** Missing are the
  declared custody of the key —without it *a signature has no
  probative value*—, the heartbeat that emits heads, re-reading a
  signature from its bytes, and **any witness**: today nobody outside
  the operator sees a head.

### What is reachable

| Phase | Content | Corrects a principle |
|---|---|---|
| **1** | Refactor + proof delegation + issuance with threshold | **Yes** |
| **2** | ISO 20022 bridge + layer metrics | No |
| **3** | Publication | — |

### What would require other resources

**Decentralization of the state** (20-40 rounds, distributed systems).
It is the only piece that would correct the principles of neutrality
and sovereignty, and it is outside the current scope. **Without it,
those principles are not met.**

**External audit.** No amount of the project's own tests replaces it.

---

## 9. Method

What distinguishes this work is not the code, but how it was verified.

**Every property has a discriminating test**: an *internally coherent*
witness that violates only the specific constraint. A witness corrupted
by brute force breaks several at once, and the test passes even if the
one that matters does nothing.

**Three times a negative test turned out not to discriminate** and had
to be redone. In the three cases the code was correct; the test did not
prove what it said.

**Own errors documented, not erased:**
- The comparison mixed debug figures with release ones, making STARK
  look 130 times faster than Groth16 when the real figure is ~11.
- Making the nullifier private broke the layer's ability to maintain
  its tree.
- `apply` did not insert the nullifier: it would have made the system's
  most expensive guarantee vacuous.

---

## 10. Conclusion

The original statement described an infrastructure for "a new economic
order founded on mathematical truth".

**What is built is more modest and more verifiable**: a demonstration
that the cryptographic properties of a sovereign settlement — privacy,
provable compliance, conservation of money, selective audit, absence of
a ceremony— are buildable, measurable and compatible with each other.

What is missing is what would turn that into infrastructure:
decentralization, external audit, and adoption. **None of the three is
a cryptographic problem.**

If the criterion is transparency, coherence and a faithful picture of
reality, this is the exact description of what there is.
