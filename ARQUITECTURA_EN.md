# Arqueo — architecture (formerly ZK-Sovereign Settlement Layer, ZK-SSL)

> **Español:** [`ARQUITECTURA.md`](./ARQUITECTURA.md) is the original of this page, in Spanish; it
> prevails wherever the two differ. This translation is of commit `7766efb` (audit entry §917) and
> was produced by the project's assistant, a Claude Code session, in audit entry §918 (`GENAI.md`):
> it is a generated artefact, not text written by the author. Line citations of the form `file:N`
> refer to the Spanish files.

Settlement layer with cryptographic privacy and demonstrable
compliance, **without any trusted ceremony**.

```rust
let mut layer = SovereignLayer::open(
    "./ledger", custodios, gobernanza, limite, tope, max_cuentas)?;

let alice = layer.open_account(sk_alice);        // saldo CERO
layer.apply_mint_delegated(...)?;                // DOS custodios DISTINTOS

// Un pago son DOS fases, y en NINGUNA la clave de gasto llega a la
// capa: la capa entrega MATERIALES (caminos y raíces, datos públicos)
// y recibe pruebas que verifica.

// FASE 1 - el pagador envía.
let m = layer.send_materials(alice, id_de_bob, 250_000, aleatorio)?;
let envio = client::prove_send(&m, clave_alice, proof_options())?;  // LOCAL
layer.apply_send(&envio, alice, &estado_alice, 250_000)?;

// FASE 2 - el receptor cobra.
let m = layer.claim_materials(bob, &envio.notice)?;
let cobro = client::prove_claim(&m, clave_bob, proof_options())?;   // LOCAL
layer.apply_claim(&cobro, bob, &estado_bob, &envio.notice)?;
```

That generating the proof and applying it are separate is what makes
**no spending key ever reach the layer**. The witness
`a_whole_payment_without_giving_any_key_to_the_layer` shows it.

With **persistence**: the ledger survives a restart, and a corrupt
state is detected before operating on it — the same `open` above
opens or creates the persisted ledger.

With **selective disclosure**: a supervisor can audit without privacy
breaking for anyone.

```rust
let d = layer.audit(sk, cuenta, 900_000, 1_100_000)?;  // "estoy entre X e Y"
verify_audit(&d)?;                                      // el supervisor, SIN la capa
```

With the **complete monetary cycle**: money can be created, moved and
retired, and the global invariant holds at every step.

⚠️ **An earlier version of this front page used the WITHDRAWN API**: the
constructor with `issuer_key`, `layer.mint(issuer_key, …)` and
`layer.apply_mint(…)` —which today only exist in
`crates/settlement-layer`, the PREVIOUS layer—, and `layer.send(sk_alice, …)`
/ `layer.claim(sk_bob, …)` as the payment flow. Issuing requires **two
distinct custodians**, not one key; and the documented flow is the one
of **materials**. `send` and `claim` with a key still exist in
`two_phase.rs` as a **residue**, and that surface is declared as a
finding of our own. Corrected in §361; the error is recorded instead
of erased.

`crates/zk-ssl` — **565 tests** (9 ignored, declared). Material for an
external audit in [`AUDITORIA.md`](./AUDITORIA.md), all in release.
The STARK backend adds **18 circuits** verified separately.

⚠️ **These figures are not remembered: they are run.** They are the pins
of `tools/canon.sh`, and `bash tools/canon.sh --sello` fails if the tree
stops giving them. There is also a gate that checks that this document
does not contradict them (§237): a number written by hand in a `.md`
ages without warning, and this one went **twenty-seven seals** saying
172.

## ⚠️ First of all: the node's operator IS a trusted intermediary

This project was built on the principle of **removing the dependence on
centralized trusted intermediaries**. One was removed —the participants
of the setup ceremony, who in Groth16 or PLONK-KZG could collude and
create money without leaving a trace— and that is a real property.

**But another remains, and it is bigger.** This layer is a **single
node**. Whoever operates it:

- **Sees every balance.** Privacy is against third parties who only see
  roots, not against whoever keeps the state. Against whoever sees a
  proof, since §538 (RFC-0009 E3b-2): the prover hides its witness and
  the proof does not publish literally any constant column value of its
  trace; between §521 and §538 it published them. ⚠️ §916: and since
  `zkssl/0.5` the proof's view is simulated without the witness, under the
  named hypotheses of D-A of RFC-0009 (accepted by the author in §916).
- **Orders the operations.** It decides what enters and in what order.
- **Can censor.** Nothing forces it to process anyone's operation.
- **Is a single point of failure.** Seizing it, shutting it down or
  corrupting it is seizing, shutting down or corrupting the whole system.

It is literally the centralized trusted intermediary the founding
principle points at. It is here in the header, and not in a list of
limitations, because **burying it would be selling a property while
hiding its opposite**.

### What is proved and what is not

| Claim | Mathematically proved? |
|---|---|
| This transfer conserves the money | ✅ |
| The balance of this account is X | ✅ |
| No money has been created outside the supply | ✅ |
| Nobody spends without being the holder | ✅ |
| Nobody spends twice | ✅ |
| **This is the current state of the system** | ❌ **Trust in the operator** |
| **These are all the operations there were** | ❌ **Nothing prevents omitting** |

**The state transitions are proved. The state and the completeness of
the history are not.** Closing that gap requires distributed consensus
— a distributed-systems problem, not a cryptography one, and pending
work.

### What this is then, precisely

**A demonstration that the cryptographic properties of a sovereign
settlement are buildable and measurable**: privacy, demonstrable
compliance, conservation of money, selective audit and absence of a
trusted ceremony, all with tests that try to break it.

**It is not** a decentralized settlement layer. Without distributed
consensus, the principles of neutrality and sovereignty are not yet
met.

---

## Why STARK, and why that was NOT obvious

The project implemented the same circuit in **five paradigms** (Groth16,
Halo2/IPA, STARK/FRI, PLONK/KZG and Nova) and measured their
trade-offs. The choice follows from the principle of sovereignty:

> **Groth16 and PLONK-KZG require a trusted ceremony.** If their
> participants collude, they can forge proofs and **create money out of
> nothing** without anyone ever detecting it.

For a sovereign infrastructure that is a permanent and unauditable
external dependency. Without a ceremony there remain Halo2/IPA and
STARK/FRI; of the two, STARK wins in everything except proof size and
is the only one with quantum resistance.

**Groth16 was discarded despite being the fastest and the one with the
smallest proofs** (192 bytes against the 72.382–84.244 of a send or claim
proof, the band a test ties down since §538). Coherence with the
principle weighed more than performance. ⚠️ **Corrected in §807**: it said
«192 bytes against 62 KB»; the 62 KB, and the 53.164 B and 61.966 B the
table below gave, are from before the hiding of §538
(`crates/zk-ssl/src/metrics.rs`: `BANDA_ENVIO_B` and `BANDA_COBRO_B`).

### The difference, measured

| | Groth16 | **Arqueo (STARK)** |
|---|---|---|
| Start-up of the layer | 2,6 s generating keys | **1,47-1,85 ms** in three runs of `metrics_of_the_layer` (`crates/zk-ssl/src/metrics.rs`), those of the table of «Metrics of the layer, measured» (⚠️ §841: it said «1,12 ms», with no source) |
| Ceremonies needed | One **per circuit** | **None** |
| Issuance proof | 192 B | 66,4 KB of 1.024 B in the run of `doc/README-CLI.md`; it varies with the salt and has no measured band |
| Settlement proof (send or claim) | 192 B | 72.382–84.244 B, the band of §538 |
| Quantum resistance | No | **Yes** |

That the constructor is
`new(custodian_set_root, governance_set_root, regulatory_limit, max_supply, max_accounts)`
and returns `Self`, instead of returning `Result` after generating keys,
**is the property made code**: there is nothing to generate and no
secret to destroy. ⚠️ **Corrected in §807**: it said
`new(issuer_key, limite)`, the signature from when a single key issued,
withdrawn (§«Threshold issuance: no more single key», below;
`SovereignLayer::new` in `crates/zk-ssl/src/lib.rs`).

## Atomic writes: no more manual intervention

**The problem**: a transfer made **four calls with nine writes** —two
accounts, one nullifier and six metadata values—. If the process died in
the middle, some were applied and others not, and the next start-up
detected the inconsistency and **stopped until manual intervention**.

**The fix**: group them into a single atomic batch.

### Why a write-ahead log was NOT written

`sled` already guarantees that a `Batch` is applied whole or not at all.
Building a WAL of our own on top would have been reimplementing —with
more failure surface— something the storage engine already does.

**The problem was never the lack of a WAL**: it was that we were making
nine loose writes where there should have been one operation.
Diagnosing it well saved half the work and all the risk.

### The guarantee, precisely

| Moment of the failure | Resulting state |
|---|---|
| Before applying the batch | The previous one, consistent |
| Between applying and syncing | One of the two, **never halfway** |
| After syncing | The new one, consistent |

In the three cases the ledger stays consistent. What can be lost in the
middle case is **durability** —the operation may not have been saved—
but not **integrity**.

That distinction is the one that matters: **losing an operation is
recoverable** (it is sent again); **a half-applied state is not**.

### ⚠️ What the tests do NOT check

**Killing the process in the middle of a write.** That requires
process-level tests, not unit tests. What is verified is that
consistency holds after each operation, and the integrity check at
start-up would detect any divergence.

## Snapshots: a lost disk is no longer the lost ledger

```rust
let info = layer.export_snapshot("./copia.bin")?;
let restaurada = SovereignLayer::import_snapshot("./copia.bin")?;
```

### Three decisions that make it useful

**Own binary format, not the engine's.** An archive copy must be
readable in ten years without depending on the version of `sled` that
wrote it.

**Deterministic order.** Two snapshots of the same state are
**byte-for-byte identical**. Without that, comparing them to detect
divergences between nodes would be useless. ⚠️ §821: two in the clear,
yes; two encrypted ones of the same state, no: the sealing carries a new
nonce every time (measured by the (a) of
`una_capa_importada_de_una_cifrada_se_reexporta_cifrada_con_su_cabecera`,
at the end of `crates/zk-ssl/src/tests.rs`; ⚠️ §828: no longer among the
last ones: behind it go those of §828, `SECURITY.md` §3.17).

**The import verifies.** It rebuilds the trees and compares the roots
with the declared ones. Without that, restoring a tampered copy would
make the node generate **valid proofs over a ledger that is not the
real one** — and the proofs would verify perfectly.

### ⚠️ What it is NOT

- **There is no live replication.** It is a point-in-time copy; later
  changes do not propagate. Hot replication requires network and
  coordination.
- **There are no incremental copies.**
- **Snapshots go encrypted** with the same key as the ledger, if there
  is one. **Without a key they go in the clear**, and then whoever has
  the file sees every balance.

## Encryption at rest

```rust
let key = crypto::LedgerKey::from_passphrase("...");
let layer = SovereignLayer::open_encrypted(path, ..., Some(key))?;
```

**XChaCha20-Poly1305** from RustCrypto, wired to every write and read of
the ledger. Writing our own cryptography here would have been a grave
error.

**Random 24-byte nonce per write.** With 12 a counter would have to be
kept, and a badly handled restart would reuse one — which breaks
confidentiality.

**Authenticated**: tampering is detected on decryption. Without that,
whoever had the disk could alter balances **without knowing the
password**.

### The test that really checks it

`balances_are_not_readable_on_disk` searches for the balance in the clear
**byte by byte across all the files**. The other two tests —that the
right password opens and the wrong one does not— would pass even with a
trivial XOR.

And there is a third, `without_encryption_the_balance_is_readable`, which
checks that **without encryption the balance DOES appear**. If it did not
appear, the first one's search would check nothing and would always
pass.

> That discipline avoided a real error: the first version of the test
> failed with a value that exceeded the issuance cap, so the balance was
> never created. It looked like a leak and it was not.

### ⚠️ The scope, and it is narrower than it seems

**Protects against**: theft of the disk, of a copy, or of a snapshot.

**Does NOT protect against**: the node's operator —who sees the balances
in memory—, nor against someone with access to the running process.

Reading it as "balances are private" would be an error. Only
decentralization fixes that.

### And its operational consequence

The key is supplied by the operator at start-up; storing it next to the
data would protect nothing. That means **the node cannot restart on its
own**: someone has to enter the password.

Since §702 (RFC-0001) the key is derived with Argon2id (RFC 9106), with a per-ledger salt and a
cost; until then it was SHA-256, with no salt and no cost, and a ledger from before is migrated on
opening it with its passphrase. The cost makes each brute-force attempt dearer, it does not prevent
them: a weak password is still weak.

Since §812 (RFC-0001, E4) the passphrase is changed: `SovereignLayer::recifrar` seals the whole
ledger again with a new passphrase and a new salt, in a directory alongside which, once verified,
is renamed as the ledger, and the old one is deleted whole. ⚠️ §812: the migration of §702
overwrote in a batch on the same `sled`, and sled, which is a log, left in the files what was
sealed with the version 1 key, which is brute-forced at the speed of a hash (`SECURITY.md` §3.14).
Since §812 the migration goes the same way as the re-encryption; a ledger migrated before is
cleaned by re-encrypting it with a new passphrase. What the file system or the disk keep of what
was deleted is beyond the layer's reach.

## The spending key no longer reaches the node as an argument

**The problem, which was undocumented**: `layer.transfer(sender_key, ...)`
received the spending key. That is, **to transfer you had to hand it to
whoever operates the node**, and with it they can empty the account
whenever they want.

It was not a scaling limitation: it was that the system required
**trusting your money to the operator**, precisely the intermediary the
project says it removes.

### The protocol

```rust
// FASE 1 — el pagador. La capa no recibe su clave; la prueba tampoco la publica (§538).
let m = layer.send_materials(alice, id_de_bob, importe, aleatorio)?;
let envio = client::prove_send(&m, key, proof_options())?;   // LOCAL
layer.apply_send(&envio, alice, &estado_alice, importe)?;

// FASE 2 — el receptor. Tampoco ve la suya.
let m = layer.claim_materials(bob, &envio.notice)?;
let cobro = client::prove_claim(&m, key_bob, proof_options())?;  // LOCAL
layer.apply_claim(&cobro, bob, &estado_bob, &envio.notice)?;
```

⚠️ **This protocol replaces the one-step one, withdrawn.** That path
—`transfer_materials` / `prove_transfer`— handed the payer **a complete
view of the receiver, with its balance**, because it updated both leaves
at once. See `AUDITORIA.md` §29.

**The send materials carry only the receiver's identifier**, and the
type enforces it: there is no field through which the balance could
enter.

⚠️ **Asymmetry of the claim**: `claim_materials` receives the notice, it
does not hand it over. The design wanted **the layer not to know which
pending item is whose**, and it knows: the node receives `receiverId` in
`zkssl_sendMaterials` (`spec/RPC.md`). Between §521 and §538 it also knew
it from the send proof, which carried in the clear the receiver's
identity, the salt and the `X` (§521, §523); since §538 (RFC-0009 E3b-2)
the proof hides them and only the amount comes out, public by design.
That it does not give the notice to the receiver is a decision of the
API, not an impossibility. How the notice reaches them is the piece that
ISO 20022 does not transport. ⚠️ §861: «does not transport» is not exact:
there is no field made for the notice; the extension one, `SplmtryData`,
admits it (read in a model derived from the XSDs, not collated with the
original); using the remittance one would be forcing it. The channel,
decided by delegation in entry 11 of `BACKLOG.md`, is still unbuilt.

**`prove_send` and `prove_claim` are free functions, not methods of the
layer.** It is deliberate: the layer **cannot** call them because it does
not have the key. If it were a method, the API would suggest the
opposite and someone would end up passing it in.

⚠️ **This paragraph described the nullifier, and it no longer applies.**
It said that the nullifier does travel to the layer and reveals nothing
new, because it is public and would appear anyway on applying the
settlement. That path was withdrawn and the nullifier tree **was removed
from the layer**: today nothing generates a nullifier. What is still true
is what is above: the layer does not receive the key as an argument.
Between §521 and §538 the proof did carry it in the clear —winterfell
0.13 did not hide the witness, measured in §521—, so «does not travel»
only held for the API; since §538 (RFC-0009 E3b-2) the prover hides the
witness and the key does not come out literally from the proof.

### The property that makes it secure

`materials_alone_are_not_enough_to_spend`: an attacker with **all the
materials** —Merkle paths, balances, nonces— but without the key
**cannot generate the proof**. That is why the materials can travel over
any channel.

### ⚠️ What this does NOT solve

**The operator still sees the balances.** The layer keeps the state, so
it knows them. This removes its seeing **keys**, not its seeing
**data**.

**Generating the proof costs ~600 ms and memory.** A light client that
wanted to delegate the computation to a third party would need that
third party to be able to prove **without** the key, which requires
verifying a signature inside the circuit (Winternitz, ~8.000 more rows).
That is an **optimization for light clients**, not a security fix:
custody is solved here.

### A correction of analysis

This problem was postponed during the project with the argument that
it "presupposed a decentralized architecture". **That was false**: the
serious part —the key reaching the node— is solved by separating the
API, with no new cryptography. The expensive part is only delegation to
third parties, which is optional.

## Transition log: the operator cannot rewrite the history

### ⚠️ This is NOT decentralization

The operator of a single node has three powers. This log closes one:

| Power | Closed? |
|---|---|
| Sees every balance | **No** |
| Orders the operations and can censor | **No** |
| **Forking or rewriting the history** | **Yes** |

The first two require distributed consensus, which is another
discipline and is not addressed. What is closed here has a name of its
own: **non-repudiation of the history**.

### How

Every operation leaves a chained entry:

```text
resumen = H(numero, tipo, raiz_antigua, raiz_nueva, H(prueba), resumen_anterior)
```

**Publishing the head is enough.** 32 bytes commit to the whole history:
two copies with the same head have the same history, and a later
rewrite separates them.

It is what *Certificate Transparency* does with certificate
authorities: it does not prevent them from misbehaving, **it makes it
so they cannot do it in secret**.

### What it detects

| Attack | |
|---|---|
| Deleting an operation from the middle | ✅ |
| Altering an entry | ✅ |
| Altering the entry **and its digest** | ✅ the chaining propagates it |
| Showing different histories to two parties | ✅ `first_divergence` locates them |
| Erasing the log by restarting | ✅ it persists |
| Tampering with the history in a copy | ✅ `verify_chain` |

### A finding: opening an account is the only transition without a proof

It had not been seen in the whole project, and it came out because the
log forces **every** root change to leave an entry. `open_account`
generates no proof —it creates no money, the account is born at zero—
but **it does move the state root**.

Now it leaves an entry with a proof digest of **zero**, and that is
visible to whoever verifies: they know that transition is *recorded*
but not *proved*.

### A design error corrected: the chain cannot cross trees

The first version chained `custodian_root` for governance and
`frozen_root` for freezing, mixing them with the account ones. **It does
not work**: the custodian root of one entry need not be the account
root of the next.

The chain always goes **over the account root**; the operations that do
not touch it have `root_old == root_new`, and what they did change is
tied by the digest of their proof.

### ⚠️ What remains unsolved

- **Nobody is obliged to look.** The log allows a fork to be detected;
  that someone detects it depends on there being observers comparing
  copies.
- **The operator might not publish the log.** That it exists does not
  oblige handing it over.
- **It does not prevent censorship.** An operation that is never
  processed leaves no entry, and its absence is indistinguishable from
  its never having been requested.

## Freezing accounts: enforced by the circuit, not by the operator

A supervisor can block an account under investigation.

```rust
let f = layer.set_frozen(&auth, cuenta, true)?;   // dos custodios
layer.apply_freeze(&f, cuenta)?;
```

### Why in circuit and not in the layer

Freezing from the layer would be the operator refusing to process. And
**the operator can already censor any operation**: that would add no
guarantee, it would only give it a name.

In circuit it is different: **the settlement proof attests that the
sender is not frozen** in that state root. Anyone who verifies it checks
it, without trusting the operator.

### The design: a separate tree, not a field in the leaf

Adding a flag to the account leaf would have forced redoing the six
circuits. Instead there is a **frozen tree** and the settlement proves
**non-membership** — the same machinery the pending tree uses.

⚠️ §487: that non-membership is that of *some* position, not the
holder's: the bit of that climb is not tied to that of the account
path. The one that holds is the layer's native check on applying
(S487); fix B will return it to the AIR.

**Depth 24**, because its climb fits in the **192 free rows** of the
settlement circuit. With depth 32 the trace would have had to be
doubled, and with it the cost of generating each transfer.

### What it prevents and what it does not

| | |
|---|---|
| Spending while frozen | ❌ Prevented |
| **Receiving while frozen** | ✅ **Allowed, on purpose** |
| Freezing without two custodians | ❌ Prevented |
| Unfreezing more easily than freezing | ❌ Symmetric |
| Lifting a freeze by restarting | ❌ It persists |

That it can **receive** is deliberate: preventing it would leave funds in
limbo and break legitimate payments to an account under investigation.

### ⚠️ What the circuit cannot know

**Nothing justifies the freeze.** It proves that two custodians
authorized it, not that they were right. And **there is no expiry**: it
lasts until someone lifts it.

### Two failures the tests found

**Persistence saved the counter but not the tree.** On restart, the
number said "there is one freeze" and no account was frozen — worse
than saving nothing, because the state was left contradictory.

**Snapshots did not include them either.** Restoring a copy lifted every
freeze. It was fixed by changing the format to `ZKSSL2`, so that an old
copy **is rejected** instead of loading halfway.

## Governance: custodians stop being eternal

**The problem, which recovery made worse**: custodians can issue money
and —since recovery— **reassign any account**. With an immutable set, a
compromised custodian keeps that power forever and the only way out is
to create a new ledger.

**The circularity**: if the custodians themselves authorize changing the
set, two compromised ones can expel the honest ones and perpetuate
themselves.

### The solution: separate authorities by level

| Set | Can | Mutability |
|---|---|---|
| **Custodians** | Issue, recover accounts | **Changeable** by governance |
| **Governance** | Change the custodians | **Immutable** |

```rust
let g = layer.update_custodians(&gov_auth, nuevo_conjunto)?;
layer.apply_governance(&g)?;
```

**The circularity does not disappear: it moves.** If the governance set
is compromised, there is no way out except creating a new ledger. ⚠️ §860: the example above does
not compile: `update_custodians` and `apply_governance` are not methods of the layer —the first
only exists as a helper of the tests, `update_custodians_delegated`, and the second as a name in a
comment—; the path is `apply_governance_delegated` (`crates/zk-ssl/src/governance.rs:104`), the
same family of withdrawn API as entry 106 of `BACKLOG.md`. And governance remains immutable by
decision: D-3 of `doc/ROTACION_DE_CUENTAS.md`, at G0 (§860; entry 20 of `BACKLOG.md`).

But it moves to keys that are used **almost never** and can be kept
offline, as against operational keys exposed daily. It is a real
improvement, and it is not the same as solving the problem.

### Options discarded, and why

**Higher threshold (3-of-N)**: reduces the risk, does not remove it, and
each signer needs its own lane in the trace — the circuit blows up.

**Delay with veto**: in a single node **it protects nothing**, because
the operator controls the order of the operations and therefore the
clock. ⚠️ §860: they were discarded for the custodians' circularity; for governance they apply by
analogy, and D-3 of `doc/ROTACION_DE_CUENTAS.md` collects them with G1, that it be rotated with
its own threshold, and G1-M, with a larger quorum: none is chosen now.

### The change of semantics in persistence

On opening a ledger, the **governance** set **is verified** (it is
immutable) and the **custodian** set **is read** (it is mutable).
Comparing the second would force whoever opens to know beforehand which
one is current — which is exactly what a governance change modifies.

Verified in `the_current_custodian_set_survives_restart`: if on
reopening the original were restored, **a revoked custodian would
recover its power by restarting the node**.

## Account recovery: no more irreversible loss

**The problem**: if a spending key was compromised, the money in that
account was lost **forever**. The system's other shortcomings degrade
the service; this one destroyed value.

**Why voluntary rotation is not enough**: changing the key using the
current key serves for hygiene, but not for compromise — if the
attacker has it, they can rotate too, and would win the race.

**The solution**: recovery assisted by the custodians that already
exist.

```rust
let nueva_id = derive_public_id(nueva_clave);   // el titular genera su clave
let r = layer.recover(&auth, cuenta, nueva_id)?; // dos custodios autorizan
layer.apply_recovery(&r, cuenta)?;
```

The API asks for **the new identity, not the key**. The new owner never
hands it to whoever operates the node — the opposite would be
contradictory in an operation that exists precisely because a key was
compromised.

### ⚠️ The trade-off, said plainly

*"Irreversible loss if your key is stolen"* is exchanged for **"two
custodians can reassign any account"**.

It is the right trade-off for a banking system —a bank can reassign
under a court order— **but only if it is visible**.

### The recovery counter

That is why there is a **public counter**, tied in the circuit, that
increments on every recovery. Without it, the custodians could reassign
accounts in silence: from outside, a recovery is indistinguishable from
any other state transition.

**It does not prevent abuse —nothing in a circuit can— but it makes it
countable**, which is the condition for accountability to exist. And it
persists across restarts, or an operator could restart the node to erase
the trace.

### What the circuit guarantees

| | |
|---|---|
| Two **distinct** custodians from the set | ✅ |
| **The balance does NOT change** | ✅ |
| The counter increments by exactly one | ✅ |
| The compromised key stops working | ✅ |

That second one is critical: without it, two custodians could **empty
any account under the guise of a recovery**.

### What the circuit CANNOT verify

**That the new holder is the legitimate one.** That is checked by the
custodians offline. Cryptography does not know whose an account is.

And **nothing prevents a race**: if the attacker spends before the
recovery is applied, that money is gone. Recovery protects the remaining
balance, not what was already spent.

## Metrics of the layer, measured

All in release and on the same machine, with the test `metrics_of_the_layer`
(`crates/zk-ssl/src/metrics.rs`) run three times; each time is the range of
the three runs. ⚠️ §841: it said «All in release, same machine, same
execution.». Re-measured on 2026-10-20 on top of §840, with
`cargo test -p zk-ssl --release --locked --lib metrics_of_the_layer -- --nocapture`,
on a 4-CPU machine with a load of 2,1 to 2,3.

| Operation | Generate | Apply / verify | Proof |
|---|---|---|---|
| **Start-up** | **1,47-1,85 ms** | — | — |
| Issuance (2-of-N) | 656-689 ms | 3,3-3,4 ms | 66.756-68.965 B measured; no band |
| Transfer (send + claim) | 1.446-1.678 ms | 9,4-10,3 ms | 145.953–167.967 B, the band of §538 |
| ├─ send | 649-858 ms | 4,7-6,1 ms | 73.571–84.244 B, the band of §538 |
| └─ claim | 758-820 ms | 4,3-4,7 ms | 72.382–83.723 B, the band of §538 |
| Burn | 625-911 ms | 3,6-4,8 ms | 74.532-75.850 B measured; no band |
| Audit (band) | 220-396 ms | 1,8-2,3 ms | 63.310-64.566 B measured; no band |

⚠️ §841: the table said, from a single run and from before the hiding of
§538: start-up 0,67 ms; issuance ~105 ms, ~2 ms and 57.342 B; transfer ~620 ms,
~4 ms and 61.966 B; burn ~110 ms, ~2 ms and 54.924 B; audit ~250 ms, ~1,5 ms
and 48.782 B. The transfer was one proof then; today a payment is two, the
send and the claim, and the row adds up the two. With the hiding the size
varies with the salt, and that is why the payment proofs are given by their
band, the one that `BANDA_ENVIO_B` and `BANDA_COBRO_B` set in `metrics.rs` and
that `tools/check_publicadas.py` ties to this table. The three runs fall
inside it: send 78.632-79.820 B, claim 78.488-80.219 B and payment
158.147-159.202 B. Issuance, burn and the audit have no measured band, and
their row gives the range of the three runs.

**Applying a payment costs 0,6-0,7 % of generating it, and verifying an
audit, 0,6-0,8 %.** That asymmetry is the economic property that makes the
system useful: whoever receives a settlement spends two orders of magnitude
less than whoever produces it. ⚠️ §841: it said «Verifying costs 0,5-0,8% of
generating.».

> ⚠️ **That ratio is the AUDIT's, not the transfer's.**
>
> `verify_audit` **only verifies**: 1,6 ms against 274 of generation, a
> **0,58 %**. It is the right figure for the argument it supports —a
> supervisor checks without touching the state— but **it was attributed to
> the transfer**.
>
> Applying a transfer costs **28,5 %** of generating it, because `apply`
> **verifies, mutates the tree and writes to disk**. It is not comparable.
>
> It was detected by running `cargo test -p zk-ssl --release metrics --
> --nocapture` and comparing with what was published. See `AUDITORIA.md` §22.
>
> ⚠️ §841: those two ratios, the 0,58 % and the 28,5 %, are from before §538.
> In the three runs of the table, `verify_audit` costs 1,8-2,3 ms against
> 220-396 ms of generation, a 0,6-0,8 %; and applying the payment, send and
> claim, 9,4-10,3 ms against 1.446-1.678 ms, a 0,6-0,7 %. This measurement
> does not write to disk: the test's layer, `new_layer()`, does not open `sled`.

**Start-up generates no keys.** In Groth16 it was 2,6 s of setup per
circuit. Here there is nothing to generate and no secret to destroy, and
that shows in the constructor itself.

### The quantified limit

**A thousand payments: 1.446 to 1.678 s of proof and 139,2 to 160,2 MiB
accumulated.** That is the numerical argument in favor of batch proofs — not
an intuition. ⚠️ §841: it said «A thousand transfers: ~620 s of proof», a
thousand times the one-proof transfer of the old table; the seconds are
today a thousand times the payment of the table above.

⚠️ **The limit this document published here no longer applies.** It said
that the position of the nullifier was derived from the nullifier itself and
that collisions were likely at **~65.000 payments**, a permanent stop for
whoever was affected. That path was withdrawn and the nullifier tree
**was removed from the layer**: today double spending is closed by
**root chaining**, with the dependence on total order that this
implies. See `PAPER.md` §4.2 and `AUDITORIA.md` §13.

### The cost per operation does not grow

Five chained transfers: the last one cost 1,18 times the first.
⚠️ This figure was justified with the nullifier tree, which
**was removed from the layer**; the measurement remains valid on its own.
⚠️ §841: and it is of the one-proof transfer, from before the hiding of
§538: it arrived with `9a1e2f1` (2026-07-27), in `README.md`, and in this
document with `70b5808`, the same day; it has not been re-measured.

### ⚠️ Proof size is NOT constant, and that deserved investigating

It was assumed to be identical for any amount. **It is not**: it varies by
**5,4%** (3.334 B over 61.517).

⚠️ §841: those figures, and those of the correlation table below, are of the
one-proof transfer, from before the hiding of §538: they arrived with
`9a1e2f1` (2026-07-27), in `README.md`, and in this document with `70b5808`,
the same day. Since §538 the hiding salt comes from entropy, and the size
also varies between runs of the same inputs; that is why the table above
gives a band. They have not been re-measured here.

The probable cause is the deduplication of nodes in the Merkle paths of the
queried positions, which come from Fiat-Shamir and change from one proof to
another.

**What matters is not the magnitude but whether it correlates with the
secret.** If the size grew with the amount, an observer who only saw the
proofs could infer magnitudes.

| Scale | Correlation (16 samples) |
|---|---|
| Absolute amount | −0,196 |
| **log₂(amount)** | **+0,008** |

The logarithmic one is the relevant one: a plausible leak would depend on
the number of bits of the amount. There the correlation is essentially zero.

**Weak evidence, not a proof.** With 16 samples a gross leak is ruled out;
ruling out the leak would require hundreds of samples and a statistical
analysis that has not been done.

### ⚠️ What these figures are NOT

Three runs on one machine. The times of the same operation vary from one
run to another: the generation of the burn went from 625 to 911 ms. They
serve to compare orders of magnitude between operations, **not as a
rigorous benchmark**. ⚠️ §841: it said «A single run on one machine.
Transfer times oscillated between 180 and 620 ms depending on the cache
context.».

## No way to create money

| Path | Closed by |
|---|---|
| Transferring more than was debited | Conservation (double entry) |
| Opening an account with a balance | Opening always at zero |
| Issuing without authorization | Two custodians proven in circuit (also against the operator since §538: §523, §538); one authorization issues once |
| Covert issuance | Public supply tied in the circuit |
| Spending twice | Root chaining (total order of the single node) |
| Spending without being the holder | Spend authority |
| Replaying a valid operation | Root chaining; the fingerprint of the holder's proof already applied (§654); on the delegated paths, the counter of the commitment the custodians sign |

⚠️ §716 (the replay of a delegated issuance): the last row said only «Root
chaining in the layer», and it was not enough. A burn or a de-issuance return
the root and the supply to those from before issuing, and the same materials of
a delegated issuance issued again. Both issuances now carry the issuance
counter, `(n, n + 1)`, in what the custodians sign, like freezing, governing and
recovering. That is why one authorization issues only once.

Each one with its discriminating test — an internally coherent witness
that only violates the specific constraint. Three times during the project
a negative test turned out not to discriminate (it failed because of
another constraint) and had to be redone.

## Persistence with integrity verification

The ledger is stored in `sled` and recovered at start-up. But saving and
loading is the easy part; what matters is **detecting a corrupt state
before operating on it**.

On opening, the layer rebuilds the two trees from the stored leaves,
**recomputes their roots** and compares them with those of the last
close. If they do not match, **start-up fails**.

> Without that check, the node would generate **perfectly valid proofs
> of transitions over a ledger that is not the real one**. From outside
> nobody would notice: the proofs would verify. It is the gravest possible
> failure in a settlement system, and the only defense is to refuse to start.

Verified in `corrupted_ledger_is_detected_at_startup`.

⚠️ **Here it was said that spent nullifiers also persist**, and that without
that, restarting the node would allow re-spending everything before because
the tree would be empty again. The nullifier tree **was removed from the
layer**: today double spending and replay are closed by **root chaining**,
with the dependence on total order that this implies.

**The system's parameters are immutable.** Opening an existing ledger
with another issuer identity or another regulatory limit fails. Silencing
it would allow **replacing the central bank without leaving a trace**.

## What the audit review uncovered, and what was done with it

Both things appeared while writing the threat model of
[`AUDITORIA.md`](./AUDITORIA.md), not during development. It is an
argument for writing it even if there is no auditor: **it forces you to
look at the system from outside**.

### `open_account` without authorization → account cap ✅

Anyone could create accounts until the node's memory was exhausted. Now
there is an **immutable cap**, like the issuance one.

⚠️ **It bounds the damage, it does not prevent the abuse.** An attacker can
exhaust the quota and leave legitimate users without room. Really closing
it would require custodian authorization to open, and that requires a new
circuit: **opening today generates no proof**. ⚠️ §860: it was decided not to
do it. Opening will not require the custodians' authorization, which would be
a second veto over the sign-up without removing the operator's; the
exhaustion of the cap stays declared in `SECURITY.md` §2 (entry 8 of
`BACKLOG.md`, closed).

### "The receiver does not authorize" → relabeled, it was not the problem

It was noted as a flaw and on examining it **it was not one**: in cash and
in banking anyone can give you money, and the universal mitigation is
**returning it**, not preventing it. Building prevention would break the
model — you could not pay someone who is not connected to authorize.

**What is missing does have another name: freezing accounts.** A supervisor
needs to block an account under investigation. That requires a flag in the
leaf, which changes the structure of the six circuits. It is not
implemented.

## ⚠️ What this layer is NOT

- **There is no network or consensus.** Single node.
- **There are no copies or replication**, nor encryption at rest: whoever
  has the disk sees every balance.
- **None of this has been audited by third parties.**

---

# The comparative work that grounded the choice

# Arqueo (formerly ZK-Sovereign Settlement Layer)

A real proof of concept, verified end to end, of the settlement of bank
payments with regulatory compliance demonstrated through zero-knowledge
proofs — without revealing balances or amounts, bound to the real state of
the ledger, with protection against double spending.

**Everything in this repository has really been compiled and
executed.** Every claim in this README has a real test run behind it,
not a promise. Where something is not solved, it is said explicitly —
see the limitations section below.

## What this project demonstrates

A compliance circuit that proves, without revealing the values:
- `amount <= balance` (the sender has sufficient funds)
- `amount <= regulatory_limit` (the transaction does not exceed the regulatory limit)
- The balance corresponds to a real account inside a Merkle tree of the
  ledger state (not an invented value)
- The proof cannot be reused as if it were a new transaction
  (nullifier with domain separation)

All of it integrated with an **ISO 20022** messaging translator (pacs.008,
simplified subset — see limitations).

## The complete monetary cycle

| Operation | Authority | Supply |
|---|---|---|
| `mint` | **Two distinct custodians**, within the cap | Rises |
| `transfer` | **Holder** | **Does not change** |
| `burn` | **Holder** | Falls |

### Threshold issuance: no more single key

Before, **one** key was enough to create money. Stealing it allowed issuing
up to the cap. No central bank operates like that.

Now issuance requires **two distinct custodians** from a set committed in
a public root:

```rust
let (raiz, caminos) = build_custodian_set(&claves);
let mut layer = SovereignLayer::open(
    "./ledger", raiz, gobernanza, limite, tope, max_cuentas)?;
layer.apply_mint_delegated(...)?;      // dos custodios DISTINTOS del conjunto
```

**The layer no longer receives any issuance key**, only the root of the
set. The authorization proofs carried it in the clear between §523
and §538: each custodian published theirs, and after a single delegated
issuance the node had two (§523). Since §538 (RFC-0009 E3b-2) the
proof hides the custodian's key.

#### The risk that had to be closed: the same custodian counting twice

The hard part is not preventing someone from outside from signing —membership
in the set solves that—. The hard part is that **a 2-of-N in which a
custodian can duplicate itself is a disguised 1-of-N**.

It is closed in two steps that depend on each other:

1. **Strictly increasing indices**, checked with a range over
   `indice_b − indice_a − 1`.
2. **The indices tied to the paths** through an accumulator that
   rebuilds them from the direction bits of the Merkle. Without this the index
   would be an invented number and step 1 would be worth nothing.

Verified in isolation in `circuit_threshold` (14 tests) and in the
full issuance circuit (10 tests).

#### ⚠️ What guarantee it gives exactly, and which it does not

**It gives**: stealing one key is no longer enough. Two are needed.

**It does not give**: that two independent people have authorized *this*
issuance. In a single-node architecture, **whoever generates the proof
needs both keys at once**, and if they have them they can issue whatever
they want.

Truly separate authorization —each custodian signing from their own
HSM— requires delegation of the proof, which in turn requires the
decentralization this project does not have.

**The guarantee is "two compromised keys instead of one", not "two
independent wills".**

### The issuance cap: not even the issuer can inflate

`max_supply` is an **immutable parameter of the ledger**, not of the
operation. If the issuer could declare it on each issuance, it would set
whichever suited it and the constraint would be vacuous.

The circuit proves `suministro_nuevo <= tope`, with the cap as a
public input. The issuing authority has the key, but **it cannot exceed
the cap without creating a new ledger** — and that would leave a trace
impossible to hide.

Two properties that go beyond the obvious case:

- **The cap applies to the accumulated total**, not to each issuance. Without
  that, a thousand small issuances would exceed any limit
  (`the_cap_applies_to_the_accumulated_supply`).
- **Burning frees up issuance capacity**
  (`burning_frees_up_minting_capacity`). That defines what the cap *is*: a
  limit on **circulation**, not a historical counter. A historical
  counter would make the system unusable in the long run — it would run out
  even if nothing were left circulating.

That asymmetry is what a central bank needs to see: money moves without
being created, and it only appears or disappears with an explicit operation
that records it in a public figure.

### Why burning does NOT require the issuer's key

**Burning cannot create money.** It reduces a balance and the supply by the
same amount, so the global invariant is preserved without the issuer
authorizing anything.

Requiring its signature would be **monetary policy, not a cryptographic
guarantee** — and it would have an effect hard to justify: that the
holder could not get rid of their own balance without permission. If a
deployment wanted to control the withdrawal of circulating money, it would
be a policy layer on top, not a constraint of the circuit.

**The identity of whoever burns is not public.** From outside it is known
that someone with authority burned that amount and that the supply fell
accordingly, but not whose it was. Issuance does identify the issuer,
because there the identity of the authority is precisely what has to be
checked.

Verified in `full_monetary_cycle`: issue → transfer → burn, checking the
invariant at every step.

## ISO 20022: the layer speaks the language of banking

Without this the system is an engine connected to nothing. ISO 20022 is the
standard on which SWIFT, SEPA and TARGET2 operate.

```text
pacs.008 (orden de transferencia)  →  pacs.002 (informe de estado)
                                        ACSC + prueba adjunta
                                        RJCT + código de motivo ISO
```

### Errors are translated to real ISO codes

| Situation | Code |
|---|---|
| Insufficient balance | `AM04` |
| Over the regulatory limit | `AM02` |
| Currency not supported | `AM03` |
| Unknown IBAN | `AC01` |
| Wrong spending key | `AG01` |
| Stale state | `DS0G` |

A receiving system understands `AM04` without knowing anything about this
implementation. That is interoperating.

### What the layer adds to the standard

The `pacs.002` carries **the proof and the roots attached**. ISO 20022 on
its own forces the receiver to trust whoever sends it the report; here it
can verify cryptographically that the transition happened.

### `settle_pacs008` never returns `Err`

A rejection is a valid response of the protocol, not an error of the
program. A messaging system **always** expects a status report;
treating the rejection as an exception would lead to lost messages and
operations in limbo.

### What building the bridge uncovered

`transfer` did not check that whoever signs was the holder, while `burn`
and `audit` did. The circuit enforced it anyway —there was no hole— but
the layer spent ~1 s generating a proof it then did not verify, and
returned a technical error instead of saying that authorization was missing.

**It only appeared when building the integration**, because a bridge forces
every error to mean something precise.

### ⚠️ Limitations

- **It is not an XML parser**: it is a struct with a subset of fields.
- **The IBAN → account resolution is outside the proof.** The circuit
  proves that the transfer between two positions is valid; that they
  correspond to those IBANs is guaranteed by the operator.
- **The spending key does not travel in the message** (ISO does not transport
  keys): it comes from a separate store.
- **A single currency.** There is no `pacs.004` nor `camt.05x`.

## Privacy and compliance: selective disclosure for the supervisor

The system proved compliance with the regulatory limit inside the
circuit, but **a supervisor could not audit anything**: there was no way to
verify the balance of a specific account without the entity showing
its whole state.

That is the real blocker for institutional adoption. A perfectly private
system in which the regulator cannot check anything is not adoptable,
however much mathematics it has behind it.

`crates/zk-core/src/circuit_audit.rs` solves it with two forms of
disclosure:

| | What it proves | Constraints |
|---|---|---|
| **Exact disclosure** | "my balance in state R is exactly X" | 6.403 |
| **Range disclosure** | "my balance exceeds X", **without saying by how much** | 7.225 |

The second is the interesting one: a bank can attest minimum reserves or
capital requirements **without exposing its position**. The relevant
regulatory fact is proved and not one bit more.

**Auditing costs a seventh of transferring.** A supervisor can
verify hundreds of accounts at the cost of a single operation — the profile
a real supervisory regime needs.

### The design decision: voluntary disclosure, not key escrow

There were two paths:

**A)** A viewing key held by the supervisor (the model of
"view keys" with escrow).

**B)** Voluntary disclosure: the holder produces the proof addressed to
whoever asks for it.

**B was implemented**, because of systemic risk: an escrowed key is a
single point of failure. Whoever steals it —an attacker, an employee, a
hostile state— sees the activity of the whole system, retroactively and
without leaving a trace. With voluntary disclosure **there is nothing to
steal**: each disclosure is a deliberate, one-off and traceable act.

**Honest trade-off**: the supervisor depends on the holder's cooperation.
If an entity refuses, the coercion comes from outside (legal requirement,
sanction, license suspension), the same as today with banking secrecy.
This does not replace legal authority; it gives it a cryptographic tool to
verify what it is handed.

### The three tests that make it useful

- `lying_about_the_balance_is_rejected` — without this, the entity would
  declare to the supervisor whatever suited it.
- `third_party_cannot_disclose_someone_elses_balance` — without the
  ownership constraint, anyone could fabricate false disclosures about
  third parties.
- `balance_below_threshold_is_rejected` — solvency cannot be faked.

9 tests. `cargo test -p zk-core --release circuit_audit`

## The regulatory limit: half corrected, and said clearly

**The hole there was**: `regulatory_limit` was an argument of
`transfer()` that traveled inside the `Settlement`, and `apply` verified the
proof against that same declared value. **The prover chose its own
limit**; with `u64::MAX` the constraint became vacuous.

**The fix applied**: the limit becomes a **system
parameter**, fixed when the layer starts. `apply` rejects any
settlement that declares another. Verified in
`settlement_with_foreign_limit_is_rejected`, which tampers the limit to
`u64::MAX` and checks the rejection.

**What remains unsolved**: now the limit is set by *whoever starts
the layer*. It is better than leaving it to the regulated party, but **there is
no regulatory authority cryptographically distinguishable from the node's
operator**.

The complete fix is the "Governance Layer" of the architecture: a
regulator commits the parameters in a public root, and each
settlement proves it used the committed ones. It would be the fifth
circuit.

The partial fix is documented as such because it is the kind of nuance
a supervisor would detect in the first review.

## Issuance: the last door through which money could be created

The system devoted 15.522 constraints to proving that money is
conserved in each transfer. But `open_account` created accounts **with a
balance, without any proof**: the node's operator could open one with a
billion. All the conservation was irrelevant while that door existed.

`crates/zk-core/src/circuit_mint.rs` closes it by separating **opening** from
**issuance**:

```rust
layer.open_account(sk)                    // saldo CERO. No crea dinero, no necesita prueba.
layer.mint(issuer_key, cuenta, importe)   // EXIGE clave del emisor y prueba.
```

The issuance circuit proves:

1. **Authority**: whoever signs knows the issuer's key
   (`issuer_id = H(DOMAIN_ISSUER, issuer_key)`, with a domain separate from
   that of the accounts).
2. **The account exists** with the declared balance.
3. **The balance increases by exactly `amount`.**
4. **The public supply increases by exactly `amount`.**

**15.016 constraints** — a third of a transfer's. Issuing is
cheap; transferring is expensive, because it must prove conservation between two
accounts and non-membership of the nullifier.

### Conservation goes from local to global

With `total_supply` as a public value that only grows through proven
issuances, **anyone can audit that the sum of all balances
equals what was issued, without seeing a single balance**.

Verified in `total_balances_always_equal_total_supply`, which also
checks that transferring does not alter the supply.

### There is no longer any way to create money

| Path | Closed by |
|---|---|
| Transferring more than was debited | Conservation (double entry) |
| Opening an account with a balance | Opening always at zero |
| Issuing without authorization | Issuer's key proven in circuit |
| Covert issuance | Public supply, tied in the circuit |
| Spending twice | Demonstrable non-membership |
| Spending without being the holder | Spend authority |
| Replaying a valid operation | Root chaining in the layer |

⚠️ The last row is the one from then and it did not hold: as soon as the root returns to the
earlier one, chaining does not stop the replay. §654 measured it for the holder's proofs and,
§716 (the replay of a delegated issuance), for the delegated issuance of today's layer. The
live table is in «No way to create money», above.

17 tests in `settlement-layer`, 9 in `circuit_settlement`, 8 in
`circuit_mint`.

### What this circuit does NOT solve

- **There is no burning of circulating money (burn).** A real system needs to
  withdraw money; it would be the symmetric circuit.
  Imposing rules (ceilings, schedules) would be another layer.
- **The issuer's key is unique.** A real central bank would use a threshold
  (m-of-n), not a single key.

## The PREVIOUS layer: `crates/settlement-layer`

> ⚠️ **This crate is superseded by `crates/zk-ssl`, and no other one depends
> on it.** `zk-ssl` cites it as its predecessor:
>
> ```rust
> //! Equivalente del `settlement-layer::sparse_tree`, adaptado al backend
> ```
>
> It is kept because it documents how the current design was reached —including
> **two errors of our own** that are told below— but **it is not the system's
> layer**. The layer is `zk-ssl`: 36 modules and **565 tests**, against
> the 2 modules and 17 of this one.
>
> ⚠️ §778: here it said **33 modules**, the count of §531, and they are **34** since
> §697, which added `nivel_de_produccion.rs`; §697 itself left it said
> that the figure stayed at 33 and did not recount it. It is counted as §531 did: the
> `.rs` of `src/` minus `lib.rs`, which are the file `mod`s that
> `crates/zk-ssl/src/lib.rs` declares (its three inline modules do not count). The 2
> of this crate is by another yardstick: with this one they would be 35 against 1 (its only
> file `mod` is `sparse_tree`), and with all the `.rs`, 37 against 2.
> ⚠️ §839: it said 35. The 36 is `src/nivel_de_produccion/censo.rs`, a
> submodule of `nivel_de_produccion` and not a `mod` of `lib.rs`: the 34 does not change.
> ⚠️ §840: it said 34 modules, 34 against 1 and 36 `.rs`. The 35 and the 37 are
> `src/censo_de_hojas.rs`, a file `mod` of `lib.rs`, tests only.
> ⚠️ §718: it said 35 modules, 35 against 1 and 37 `.rs`. The 36 and the 38 are
> `src/tests_canonicidad.rs`, another file `mod` of `lib.rs`, tests only.
>
> An earlier version of this document titled it *"The layer"* with nothing more,
> which **contradicted `AUDITORIA.md` §15** in the same repository.

The other crates implement **circuits**: they prove that *one*
transfer is valid. This one keeps the **state** and applies operations
one after another. It was what turned cryptographic primitives into a settlement
layer **for the first time** in this project.

```rust
let mut layer = SettlementLayer::new(seed)?;
let alice = layer.open_account(sk_alice, 1_000_000);
let bob   = layer.open_account(sk_bob,      50_000);

let settlement = layer.transfer(sk_alice, alice, bob, 250_000, limite)?;
layer.apply(&settlement, alice, bob, 250_000)?;
```

**`transfer` generates the proof without touching the state; `apply` verifies it and
only then applies.** The separation is deliberate: it allows whoever
produces the proof and whoever accepts it to be different parties, which is the
real case between banks.

### What the layer contributes and the circuit cannot

**Rejection of replays.** Resending a valid settlement would duplicate
the money. The circuit does not prevent it —the proof is still valid—; the
layer blocks it by checking that `root_old` is the current state.
Verified in `replaying_a_settlement_is_rejected`.

**State chaining.** Each operation starts from the root the
previous one left, with a different nonce and nullifier. Verified in
`consecutive_transfers_chain_correctly`.

**A viable sparse tree.** The one in `zk-core::merkle` rebuilds 2^20
leaves on every change: it serves for tests, it is unviable for a ledger. ⚠️ §822:
since §822 it no longer rebuilds 2^20 leaves: it does not store the padding, only the nodes
that cover some given leaf, and any other one is the hash of an empty subtree; with eight
leaves, a `build` is 44 hashes, and the root and the paths are those of the dense tree (entry 75
of `BACKLOG.md`). The underlying point still stands: it is not mutable, and changing a balance
means building it again from the whole list of leaves.
`SparseMerkleTree<DEPTH>` stores only the occupied ones and updates in `DEPTH`
hashes. The depth goes in the type (20 for accounts, 32 for
nullifiers) because mixing them already caused a failure, and this way the
compiler prevents it.

**17 tests.** Run: `cargo test -p settlement-layer --release`

⚠️ An earlier version of this document said **12**, and another point said
**16**. Neither of the two was true.

### Two errors of our own that the tests uncovered

**`apply` did not insert the nullifier.** The tree never grew, so the
non-membership —the most expensive guarantee of the system, 15.522 constraints—
would have been **vacuous in practice**. It was detected by
`full_transfer_cycle_updates_state` when checking that the root changes.

**Making the nullifier private was a design error.** It was presented as
a privacy improvement; it was not one. Without knowing it, whoever applies the
settlement cannot maintain their tree. In Zcash nullifiers are
public exactly for that reason: privacy comes from their being
**indistinguishable** (derived from `sk`, with no link to any account),
not from hiding them. Reverted.

### ⚠️ What this layer is NOT

- **There is no network or consensus.** It is a single in-memory node. A real
  federation needs agreement on the order of the operations — a problem
  of distributed systems, not of cryptography.
- **There is no persistence of the account tree.**
- **`open_account` is not proven.** It modifies the tree directly:
  it is administration, not a user operation. Without an issuance circuit,
  **the operator can create money**.
- **There is no delegation of the proof.** Whoever generates it needs the spending
  key.
- **Only on Groth16.**

## Spend authority: from proof of solvency to settlement layer

`crates/zk-core/src/circuit_settlement.rs` is **the circuit that should be
used**. The earlier ones (`circuit_with_state`, `circuit_double_entry`)
are kept as a reference of the evolution of the design, but they have a
grave hole:

> **They proved that whoever runs the circuit KNOWS the account's data,
> not that they are AUTHORIZED to spend.** Anyone with access to the
> balance, the nonce and the Merkle path —from a leak, a backup, an
> employee— could move the money.

The corrected design follows the Zcash Sapling model:

```text
sk        = clave de gasto (privada, nunca sale del titular)
pk        = H(DOMAIN_PK,   sk)
leaf      = H(H(pk, balance), nonce)
nullifier = H(H(DOMAIN_NULL, sk), nonce)
```

**Two new properties:**

**Authorization.** The circuit requires knowing `sk` such that `pk = H(sk)`. The
identity of the account becomes a cryptographic commitment to the key.

**Unobservability of the nullifier.** Before, it was derived from a public
identifier, so anyone could **precompute the nullifier of someone else's
account and watch the registry of spent ones to know when that
account moves money**. In an interbank network that reveals the pattern of
operations of every participant to any observer.
Deriving it from `sk`, only the holder can compute it.

**Measured cost: 475 constraints, 1,7% of the circuit.**

| | Constraints | Setup | Proof |
|---|---|---|---|
| Double entry without authority | 27.562 | 1,12 s | 1,17 s |
| **Settlement circuit** | **28.037** | **2,59 s** | **1,68 s** |

Verified with 8 tests, among them
`attacker_without_spend_key_cannot_transfer`: an attacker who knows everything
about the account except the key cannot transfer.

### The sender/receiver asymmetry is deliberate

Only the sender proves authority; the receiver appears with its identity
public, without a key. **Receiving money requires no permission** — requiring the
receiver's signature would make it impossible to transfer to whoever is not present
to authorize. The same asymmetry as in Zcash and in any real payment
system.

## Demonstrable non-membership: double spending stops depending on a database

`persistent_nullifier_registry` is a single-node `sled` database, and the
circuit **checked nothing about it**. Whoever controlled
that node could accept a repeated nullifier: **the proof would still be
cryptographically valid**. It was the only point of the system where a
party could create money without breaking any mathematics.

`crates/zk-core/src/nullifier_tree.rs` closes it with a sparse tree of
32 levels. On spending, the circuit proves **two things at once**:

1. **Non-membership**: the position of the nullifier was empty in
   `nullifier_root_old`.
2. **Insertion**: `nullifier_root_new` is that tree with the nullifier
   inserted.

Both roots are public, so the chain is auditable. **Double
spending goes from "detectable by a database" to "mathematically
impossible".**

**Favorable side effect**: the nullifier stops being a public input
— it stays committed inside the tree. Less information exposed with the
same guarantee.

### The cost of each guarantee, measured

| | Constraints | Increase |
|---|---|---|
| Double entry alone | 27.562 | — |
| + spend authority | 28.037 | +475 (**1,7%**) |
| **+ demonstrable non-membership** | **43.559** | **+15.522 (55%)** |

The strongest guarantee of the system is also, by far, the most
expensive: closing double spending cryptographically costs **33 times more**
than closing authorization.

Verified with 9 tests, among them
`double_spend_is_rejected_by_the_circuit`.

### ⚠️ Documented limitation: position collisions

The position is derived from the 32 low bits of the nullifier (a tree over
the whole field would have 254 levels). Two nullifiers can collide: the
second one **could not be spent**. It is **denial of service, not double
spending** — soundness holds, completeness does not.

With 10.000 nullifiers the probability is around 1 in 10⁵. Acceptable for
a demonstration; at scale an indexed tree would be needed (like the one of
Aztec). There is an executable test
(`position_collisions_are_possible_and_documented`) that demonstrates it instead
of leaving it in a comment.

### What is still missing for production

⚠️ **This list is the one of `zk-core`/Groth16, the COMPARISON backend.**
The production path is STARK (§106) and its gaps are in
`README.md` §«Qué garantiza y qué no» [what it guarantees and what it does not] and in
`SECURITY.md`. It is said here because reading this list as «what the system
lacks» would be confusing the backend that is kept **as evidence of why STARK was
chosen** with the one that is on the production path.

- **There is no delegation of the proof.** Generating a proof costs ~1,7 s and
  requires `sk`. In a bank, the key would live in an HSM and the computation
  would be done by another service — but that service would need the key, and with
  it could send the money wherever it wanted. The solution (signing the
  details with `sk` and verifying the signature INSIDE the circuit) requires
  elliptic-curve arithmetic in circuit, a much bigger piece.
- **There is no selective disclosure** for the supervisor's audit.
- **It is only implemented in Groth16.** The other four backends still
  have the design without authority or non-membership, so the comparison
  presents as equivalent things that no longer are.

## The accounting core: double entry

The main circuit (`ComplianceCircuitWithState`) proves that the
sender has sufficient balance and respects the regulatory limit. But **it does
not prove what happens to the money**: nothing ties the debit of the origin to the
credit of the destination.

`crates/zk-core/src/circuit_double_entry.rs` closes that gap. It proves
the complete state transition:

```text
saldo_emisor_nuevo   = saldo_emisor   - importe   (ADEUDO)
saldo_receptor_nuevo = saldo_receptor + importe   (ABONO)
```

Conservation is not a tautology: **both balances are committed
in the Merkle tree and both roots (before and after) are public**, so
the same amount that is subtracted from a committed balance is added to another
committed balance, and the result is reflected in a public root.

Verified with tests that deliberately build scenarios of
**creation** and **destruction** of money (the receiver receives 10.000 more or
less than was debited) and check that the circuit rejects them — cases that
the previous circuit could not detect.

| | Circuit with state | Double-entry circuit |
|---|---|---|
| Constraints | 9.934 | 27.562 |
| Setup (release) | 438 ms | 1.123 ms |
| Proof (release) | 422 ms | 1.170 ms |
| Proves | solvency of the sender | **conservation of money** |

Reproduce: `cargo test -p zk-core --release circuit_double_entry -- --nocapture`

### Implemented in the FOUR backends

| | Groth16 | Halo2/IPA | STARK | PLONK-KZG |
|---|---|---|---|---|
| Module | `zk-core::circuit_double_entry` | `halo2-experiment::circuit_double_entry` | `stark-experiment::double_entry` | `plonk-experiment::circuit_double_entry` |
| Size | 27.562 constraints | k = 17 | 113 constraints, 41 columns, 1024 rows | 84.801 gates |
| Tests | 8/8 | 6/6 | 8/8 | 7/7 |

**The port to STARK revealed a structural problem that does not exist in the
other two paradigms**: AIR lacks copy constraints, so
nothing forces the two climbs of the tree (old leaf and new leaf) to
use the same siblings. A sequential design would have allowed using
different siblings in each climb and fabricating a root that does not correspond
to the same position of the tree — a **silent** hole, because the
honest witnesses do use the same siblings.

The solution is a **lockstep** design: the two lanes advance level
by level at the same time and a constraint forces the sibling to be identical in
both. Verified in isolation in `stark-experiment::dual_climb`,
with a test that builds two internally coherent lanes along
different paths — exactly the attack the sequential design
would allow.

Practical conclusion: **porting a circuit from Plonkish to AIR is not
mechanical, not even when the logic is identical.**

**Limitations**: it does not handle the creation of currency by a central bank
(which would break conservation deliberately and would need a public
issuance input); and it does not prevent a self-transfer with a
dedicated constraint, although it rejects it by construction.

## Five cryptographic engines, verified separately

This project implements the same logic **four times**, with paradigms
and commitment schemes that differ (R1CS, Plonkish with IPA, AIR with FRI,
and Plonkish with KZG), plus a **fifth backend of a different paradigm**
(Nova / folding), to compare their trade-offs with measured data instead
of with what the papers say:

| | `zk-core` + `iso-bridge` | `halo2-experiment` | `stark-experiment` |
|---|---|---|---|
| Scheme | Groth16 (Arkworks, BLS12-381) | Halo2 with IPA (Zcash, Pallas) | STARK/FRI (Winterfell, Goldilocks) |
| Paradigm | R1CS | Plonkish | AIR |
| Trusted setup | Yes, per circuit — MPC ceremony implemented (see below) | No | No |
| Setup / parameters | 438 ms | 16,3 s | none |
| Generation | 422 ms | 4,86 s | 38 ms |
| Verification | 5 ms | 91 ms | 1 ms |
| Proof size | 192 bytes | 4.096 bytes | 39.048 B / 127.274 B (⚠️ §806: it said «36,7 KB / 125,6 KB») |
| Quantum resistance | No | No | Yes (hashes only) |
| Merkle tree | 20 levels | 20 levels | 32 levels |
| `SettlementProver` | Implemented | Implemented | Implemented |
| Status | Verified end-to-end | Verified end-to-end | Verified end-to-end |

The two size figures of the STARK backend correspond to two different
security levels, and the difference matters: **39.048 B with 127
_conjectured_ bits** (the level usually cited in the STARK
ecosystem) against **127.274 B with 128 _provable_ bits** (soundness with a
proof, without relying on the Reed-Solomon decoding conjecture). The ISO 20022 bridge of the
STARK backend uses the second by default.

⚠️ §806: the table and this paragraph said «36,7 KB» and «125,6 KB». They are the
blowup-16 rows of the test `real_proof_end_to_end_with_metrics` of
`stark-experiment`, and re-measured on 2026-10-14 on top of `b6491ef`, with
`cargo test --release --locked -p stark-experiment --lib real_proof_end_to_end_with_metrics -- --nocapture`,
they give 39.048 and 127.274 B, as on 2026-10-07 (§698); `FIVE_BACKENDS.md` §4
tells it. The size of the proof of the ISO bridge has not been measured
again here.

**The complete comparison of the three, with methodology and measured
numbers, is in [`FIVE_BACKENDS.md`](./FIVE_BACKENDS_EN.md).** The
earlier two-way comparison, with the analysis of institutional
governance, is still available in
[`GROTH16_VS_HALO2.md`](./doc/comparativa/GROTH16_VS_HALO2_EN.md). The detailed metrics
of the Groth16 engine are in [`PERFORMANCE.md`](./doc/historia/PERFORMANCE_EN.md).

### The fifth backend: Nova, a different paradigm

`crates/nova-experiment` does not compete on the same axis as the other four.
They produce a **monolithic proof** of the whole circuit; Nova
**folds** a sequence of steps and compresses at the end.

The question it answers and the others do not: **how much does transaction
number N+1 cost?** Measured: **~250 ms, constant** — step 9 cost 0,77 times
step 1. The closing (compressing to a deliverable proof) costs 1,84 s,
amortizable across all the transactions of the day.

**Honest scope**: it was closed at proof-of-concept level. It does not
implement the compliance circuit nor double entry. And its
fixed overhead is substantial — 10.764 constraints per step for a
circuit that does a single hash.

**A side finding that is worth it**: `nova-snark` is the only one of
the five libraries that **prevents in code** generating parameters with a
single party, and offers `setup_with_ptau_dir` to consume files of
Perpetual Powers of Tau. The other four allow it and trust the
documentation.

### Note on the most relevant finding of the STARK backend

Without a field extension, soundness has a ceiling of **63 bits** — the
size of the Goldilocks field itself — however many queries are added. The
"fast and compact" configuration one would choose by default **is not
comparable** with the ~128 bits of Groth16 or Halo2. It is the kind of detail
that disappears from promotional materials and that here is measured and
documented.

## Repository structure

```
crates/
  zk-core/                circuito Groth16 (ComplianceCircuit, ComplianceCircuitWithState)
  iso-bridge/              traductor ISO 20022 -> testigos del circuito Groth16
  halo2-experiment/        mismo circuito, reimplementado en Halo2/IPA
    src/range_check.rs           range check de 64 bits
    src/poseidon_hash.rs         Poseidon real via halo2_gadgets
    src/merkle.rs                 arbol de Merkle de 20 niveles
    src/nullifier.rs              nullifier con separacion de dominio
    src/compliance_circuit.rs     los cuatro anteriores, unificados
    src/compliance_real_proof.rs  pipeline de prueba real con IPA + metricas
    src/iso_bridge.rs             traductor ISO 20022 -> Halo2
    src/persistent_nullifier_registry.rs  persistencia en sled
  stark-experiment/        mismo circuito, reimplementado en STARK/AIR
    src/range_check.rs            range check de 62 bits (RFC-0017: sólido para restas)
    src/rescue_hash.rs            permutacion Rescue Prime como restricciones AIR
    src/merkle.rs                 arbol de Merkle de 32 niveles
    src/nullifier.rs              nullifier con separacion de dominio
    src/solvency.rs               solvencia con valores privados (Horner)
    src/compliance_circuit.rs     todo unificado en una sola traza de 512 filas
    src/compliance_real_proof.rs  pipeline real + metricas de 4 configuraciones
    src/iso_bridge.rs             traductor ISO 20022 -> STARK (sin claves)
    src/persistent_nullifier_registry.rs  persistencia en sled
```

## How to build and verify

```bash
# Motor Groth16 (zk-core + iso-bridge)
cargo test --workspace --no-fail-fast -- --nocapture

# Motor Halo2 (aislado, dependencias distintas)
cargo test -p halo2-experiment -- --nocapture

# Motor STARK (aislado, ecosistema winterfell)
cargo test -p stark-experiment -- --nocapture

# Metricas STARK reales (requiere release para que los tiempos sean citables)
cargo test -p stark-experiment real_proof --release -- --nocapture

# Los tests que la comprobacion de grados de winterfell no puede validar en
# depuracion -limite conocido de la herramienta, no fallo de solidez- se
# saltan SOLO en depuracion y corren con --release junto al resto: no hace
# falta ninguna orden aparte. Lo que la orden de abajo corre son los
# instrumentos de MEDIDA, marcados #[ignore] sin condicion porque miden y no
# comprueban; su razon vive en el docstring de cada uno.
cargo test -p stark-experiment --release -- --ignored
```

**Wall-clock notice**: the tests that generate complete cryptographic proofs
(not just a satisfiability check) take from several
seconds to several minutes, depending on the piece and the CPU contention of the
environment. This is expected, documented, and indicates no problem. See
`PERFORMANCE.md` and `GROTH16_VS_HALO2.md` for concrete measured times.

**Profile notice**: the first command above runs in **debug**, and
the layer is not measured there. Measured on 26-08-2026: `cargo test -p zk-ssl` in
debug gives 188 pass, 93 fail and 9 ignored; in release, 307 pass, 0
fail and 3 ignored. The failures are the degree limit of entry 41
and of §78 -declared and decided, not a soundness defect-, but you will see
red if you run that command as it is. **The layer is measured with
`cargo test -p zk-ssl --release`**, which is what `tools/canon.sh` runs.

⚠️ **§814: re-measured on 15-10-2026, and the layer no longer gives red in debug.**
`cargo test --locked -p zk-ssl` in debug gives 443 pass, 0 fail and 45
ignored, on `0315b14` (§813), in 3.253,94 s with other work on the
machine (4 CPU, load from 9,2 to 18,6). In release, the layer's row in
`tools/canon.sh` pins today 562 pass and 8 ignored (⚠️ §905: it said 561; ⚠️ §900: it said 558
and 7; ⚠️ §899: it said 557; ⚠️ §898: it said 552; ⚠️ §896: it said 547; ⚠️ §839: it said 481;
the 482nd is the census of the lengths, which also runs in debug, where it would be
444 pass: the 443 are from `0315b14`, and the whole layer has not been run there
again. ⚠️ §840: it said 482; the two extra ones, the census of the `native_leaf`s, also run
there: it would be 446. ⚠️ §842: it said 484; the extra one, the census of the byte
bands, too: it would be 447. ⚠️ §816: it said 485; the eleven extra ones, those of the ledger's
lock, have not been run in debug. ⚠️ §817: it said 496; the extra one, that of the snapshot in
the clear, neither. ⚠️ §819: it said 497; the extra one, that of the refusal of the sweep,
neither. ⚠️ §820: it said 498; the three of the export, neither. ⚠️ §821: it said 501; the two of
the layer imported with a key, neither. ⚠️ §828: it said 503; the twelve of the re-encryption
and of the migration of a ledger of another owner, neither. ⚠️ §711: it said 515; the three extra
ones, those of the layer's network, neither. ⚠️ §715: it said 518; the four extra ones, those of
the replay with the malleated proof, neither. ⚠️ §716: it said 522; the eight extra ones, those
of the replay of a delegated issuance, neither. ⚠️ §717: it said 530; the extra one, that of the
ledger on the cap of `m`, neither. ⚠️ §718: it said 531; the fifteen extra ones, those of the
canonicity of the store, the snapshot and the producer of the age, neither, and one carries
`cfg_attr(debug_assertions, ignore = …)`. ⚠️ §723: it said 546; the extra one, that of the leaf
of the window, neither: the 45 and the 38 that follow are those of `0315b14`). Of the 45, 7 are
measuring instruments, ignored in every profile, and 38 carry
`cfg_attr(debug_assertions, ignore = …)`: in debug they do not run, and run there with
`--ignored` the 38 pass. There are no longer failures to attribute to the degree limit, and with
this measurement entry 41 of `BACKLOG.md` is closed. It still stands: those 38 are only run by
release, and the layer is measured with `--release`. Whether the first command above, the one
for the whole workspace, gives red today is not known: it has not been re-measured. And the
«307» of release was not from 26-08-2026: that day they were 287 (§366), and the seals raised it
with the pin until §394 without touching the date.

## Honest limitations — read before considering this for production

- **MPC ceremony: mechanism solved, ceremony not held.** The
  Groth16 engine needs a trusted setup per circuit. The crate
  `crates/ceremony` implements the two-phase MPC ceremony (BGM17) and
  is verified over the real circuit: `cargo test -p zk-core
  --release --test ceremony_integration`. **But running the
  contributions on a single machine shows that the mechanism
  works, NOT that security exists**: the MPC guarantee ("it is enough for
  one participant to be honest and destroy their randomness") requires
  real and independent participants publishing the transcript. That
  step is pending. See `crates/ceremony/ATTRIBUTION.md`.

  Historical note: this project documented for a long time that there was
  no ceremony tooling for Arkworks 0.4.x, after investigating
  `ark-marlin` and `celo-org/snark-setup`. That conclusion was **wrong or
  became obsolete**: Penumbra published `penumbra-sdk-proof-setup`, from which
  the code of `crates/ceremony` comes.
- **Nullifier registry persistent but of a single node**, in both
  engines. It solves persistence across restarts of one validator, not the
  coordination between several validators of a consortium — that requires
  the registry to be part of the state replicated by consensus.
- **`iso-bridge` is a simplified subset** of pacs.008.001.08, not
  an XML parser conforming to the real XSD, and it does not handle currency-specific
  decimals (JPY, KWD, etc. differ from EUR/USD).
- **The three backends are NOT interchangeable with one another.** Each one operates
  over a different finite field, so their Merkle trees and their
  nullifier spaces are incompatible: an account registered in the
  Groth16 tree does not exist in the STARK one. The trait
  `SettlementProver` unifies the FORM of the call, not the data — see
  the extended note in `crates/settlement-prover/src/lib.rs`.
- **The STARK backend covers 62 bits of range, not 64** (RFC-0017, §641), because
  of the size of the Goldilocks field (2^64 - 2^32 + 1, smaller than 2^64): the
  63-bit range was not sound for a subtraction, whose wrapped result could
  fit in it; with 62 bits it cannot (`2*2^62 < p`). Enough for
  any real monetary amount, but it is a difference from the other
  two engines, not an equivalence.
- **The post-quantum resistance of the STARK backend is not free.**
  It rests on the hash function resisting Grover's algorithm, which
  in practice requires doubling the output size with respect to the
  classical case. It is a much simpler adjustment than patching elliptic curves
  with lattices, but it is not "without patches".
- **None of this code has been audited externally.**
- **`ark-groth16` is officially declared "an academic prototype, not ready
  for production"** by its own maintainers.

## License

This project is published under a dual MIT / Apache-2.0 license, at the choice
of whoever uses it — the same convention that Arkworks and Halo2 use, the two
ecosystems on which it is built. See `LICENSE-MIT` and
`LICENSE-APACHE`.

## Contributing / reporting problems

This is an open research and engineering project. If you find
an error, a vulnerability, or want to propose an improvement (in
particular: the pending MPC ceremony, the distributed coordination of the
nullifier registry, or the complete ISO 20022 parser),
contributions are welcome.
