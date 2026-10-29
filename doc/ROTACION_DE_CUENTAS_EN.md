<!--
  ROTACION_DE_CUENTAS — a single design for changing an account's keys.
  Design note for entry 52 of the BACKLOG, ACCEPTED as analysis in §799 (⚠️ §799: until here it
  said «in DRAFT»), which joins the two open cases of entry 20, the new leaf of §117, the
  unrotatable view of §127.5 and the B18.2 reclamation of CONFIANZA_RESIDUAL.md. Sister of
  AVISO_EN_ISO20022.md, which adds a case to it. It touches no code, nor the wire, nor a circuit: it
  measures what there is and what each option would cost, and does not choose. ⚠️ §860: the author
  delegated those decisions, and they are taken in §860 (§6).
-->

# ROTACION_DE_CUENTAS — changing key without designing it three times

> **Español:** [`doc/ROTACION_DE_CUENTAS.md`](./ROTACION_DE_CUENTAS.md) is the original of this
> page, in Spanish; it prevails wherever the two differ. This translation is of commit `7766efb`
> (audit entry §917) and was produced by the project's assistant, a Claude Code session, in audit
> entry §918 (`GENAI.md`): it is a generated artefact, not text written by the author. Line
> citations of the form `file:N` refer to the Spanish files.

**Status**: ACCEPTED (§799), as analysis: the author accepts it as it is and leaves its decisions
open, D-1 to D-9 of §6, without choosing any option. ⚠️ §799: until here it said «DRAFT».
Analysis without code and without an RFC number. The options of §4 are left **unchosen**: who
authorizes a rotation, where the new leaf lands and which cases enter is the author's (§6). It
closes neither entry 52 nor entry 20. ⚠️ §812: the author resolved D-4 —K1, apart from the single
design, as E4 of RFC-0001, built in §812—; the other eight remain open.
⚠️ §860: the other eight, taken by delegation from the author («resolve them applying the
principles»), with the criterion of §609 and the precedent of §642; REVERSIBLE; they enter with
its acceptance (§6). A single design: one transition —N1 to N4, §3— and several authorizers. It
closes entry 20, with D-3 at G0; entry 52 stays open until N4. After that seal no decision remains
open in §6. And §4 gains three options it did not have: R3s, G1-M and G3.

**Provenance**: Claude wrote it in a Claude Code session in the cloud, on `8c02941` (§789),
outside step 4 of [`GENAI.md`](../GENAI.md); §791 of `AUDITORIA.md` carries it. Integrating is not
accepting: acceptance remained pending until §799, in which the author accepts it as analysis, with
its decisions open. ⚠️ §799: until here it said «acceptance remains pending». ⚠️ §792: its
citations of `BACKLOG.md` by line became citations by entry, rule v5 of `tools/verificar_citas.py`.
⚠️ §802: the code of §802 moved `lib.rs`, `accounts.rs`, `recovery.rs` and `persistence.rs`, and
its 36 citations of those files were corrected to today's line: each one points to what it pointed
to.

**Convention**: ✅ measured in this tree, with file:line · 📐 derived from reading the code, without
running it · ⚠️ nuance · ❓ not measured.

---

## 0. Why a single design

§134.1 left it written (`AUDITORIA.md:10982-11004`): three decisions deferred their cost to
rotation —§117, «rotating the key implies a new leaf»; §127.5, the view key «is unrotatable
without rotating the spending one»; and B18.2, whose migration reclamation «IS a rotation with a
new leaf»—, and entry 20 has two more open cases. «Either they are unified … in ONE single rotation
design, or rotation will be designed three times, mutually incompatible.» Entry 52 takes it up with
its scope: unifying the two of entry 20, the B18.2 reclamation and the three deferred costs, in a
session of its own (entry 52 of `BACKLOG.md`). The same entry asks to read two things first: how
`recover` couples with the custodians and that the layer has no time. They are §1.2 and §1.4.

## 1. What there is today, measured

### 1.1 The keys, and what hangs from each one

| key | what hangs from it | who changes it today |
|---|---|---|
| **spending** `sk` | everything derived and nothing stored: the identity (`crates/stark-experiment/src/native.rs:25-48`), the nullifier (`native.rs:67-83`), the leaf salt (`native.rs:102-122`) and the view key with its `view_id` (`native.rs:134-185`); opening stores the identity, the salt and the `view_id` already derived (`crates/zk-ssl/src/accounts.rs:170-248`) | two custodians, with `apply_recovery_delegated` (`crates/zk-ssl/src/recovery.rs:52-193`); the holder, no |
| **custodian** | `custodian_set_root` (`crates/zk-ssl/src/lib.rs:964-974`) | governance, with `apply_governance_delegated` (`crates/zk-ssl/src/governance.rs:104`) |
| **governance** | `governance_set_root`, «IMMUTABLE» (`lib.rs:975-983`) | nobody: changing it is creating a new ledger (`governance.rs:18-25`, `:30-34`) |
| **the ledger's at-rest** | `LedgerKey`, from the passphrase the operator gives at start-up (`lib.rs:1056-1062`; `crates/zk-ssl/src/crypto.rs:16-26`) | ~~nobody changes the passphrase~~ (§2, case 5). ⚠️ §812: `SovereignLayer::recifrar` changes it (`crates/zk-ssl/src/persistence.rs:1262-1355`; ⚠️ §816: it said `:1230-1310`, the lines from before §816), in the library; the node stays in the clear |
| **the wallet keystore's** | the file's passphrase (`crates/zk-ssl-sdk/src/keystore.rs:1-27`) | `save` with another passphrase (`keystore.rs:262`) |
| the node signer's | the heads, with XMSS | RFC-0015, with its acts: **outside** this note, which is about account keys |

### 1.2 How recovery couples with the custodians

✅ What `apply_recovery_delegated` does (`recovery.rs:52-193`), read line by line:

- **The authority is two distinct custodians of the current set**: two proofs of
  `circuit_threshold_single_nullifier` with the custodian domain and against `custodian_set_root`
  (`recovery.rs:151-171`). Governance ones do not recover (`governance_keys_cannot_recover`,
  `recovery.rs:458`).
- **It spends custodian quota** after verifying the authority (`recovery.rs:172-174`), and the
  log **derives** it: `usos_custodiados` counts each `Recovery` as one use
  (`crates/zk-ssl/src/log.rs:520-538`).
- **It raises a public counter**, `recovery_count`, which goes in the public inputs of the climb
  (`crates/stark-experiment/src/circuit_recovery_climb.rs:325-332`) and in the commitment the
  custodians sign, together with the two roots (`recovery.rs:145-149`).
- **The leaf stays in its position**: new identity, the same balance and the nonce plus one
  (`recovery.rs:89-96`), with the climb of the two lanes along the same path
  (`circuit_recovery_climb.rs:1-25`). Recovering towards the same identity is rejected before
  verifying (`recovery.rs:81-83`), and resending the same materials, too (`recovery.rs:287`).
- **The API asks for the new identity, not the key** (`recovery.rs:21-25`, `:60`), and that is why
  a narrow account can rotate to a wide key (`a_narrow_account_can_rotate_to_a_256_bit_key`,
  `recovery.rs:223`), which the test itself declares **not sovereign**: «demands TWO CUSTODIANS»
  (`recovery.rs:217-222`).
- **It is neither on the wire nor in the vectors**: `spec/RPC.md` has no recovery method (its
  table, `spec/RPC.md:76-125`), and `grep -ril "recover\|recuper" spec/vectors` comes out empty.
  What is outside the layer is the seal: the re-verifier recomposes the commitment of an era 1
  recovery with those same roots and counters
  (`crates/zk-ssl-verify/src/reverificacion.rs:178-198`).
- **What recovery does not solve**, said in its circuit: voluntary rotation «serves for hygiene,
  but not for compromise: if the attacker has it, they can rotate too, and would win the race»
  (`crates/stark-experiment/src/circuit_recovery.rs:10-17`); and if the attacker spends before it
  is applied, the money goes (`circuit_recovery.rs:58-60`).

### 1.3 What recovery leaves old: the 49-A↔52 seam

✅ **The `view_id` and the salt are copied** from the old key (`recovery.rs:97-110`), and the copy
of the salt is not only the layer's: the climb has **a single** salt column for the two lanes,
«THE COPY» (`circuit_recovery_climb.rs:72-77`; its `build_trace`, `:180-189`, «the rotation belongs
to seam 52, out of scope»). The salt is imposed by the circuit; the `view_id`, which is not in the
leaf (`lib.rs:752-767` against `native_leaf_salted`, `crates/zk-ssl-hash/src/lib.rs:205-212`), is
tied by no proof.

📐 What follows from there, read and not run; recovery is not on the wire, so today no node
reaches it:

- **a. The view.** The node demands the view key for the view, the send materials and the claim
  ones (`exige_credencial`, `crates/zk-ssl-node/src/main.rs:2446-2451`; `:3699-3707`;
  `:3789-3798`), and compares it with the stored `view_id` (`crates/zk-ssl/src/client.rs:123-133`).
  After a recovery that `view_id` is the old key's: the new one does not pass the gate and the old
  one does. `doc/ecst/VERIFICACION.md:1323` already deduced it.
- **b. The salt, and B18.2.** The new key does not re-derive the salt of its leaf; the holder
  depends on the materials giving it (`AccountView.leaf_salt`, `client.rs:66-75`). The migration
  reclamation «requires only the key — exactly property T2b» (`doc/CONFIANZA_RESIDUAL.md:143-146`),
  and in a recovered account that property no longer holds: only the old key re-derives the salt.
- **c. Privacy against the old key.** Whoever has the old key has the salt, and with it the leaf
  becomes dictionary-attackable again from a path, as before the salt (the measurement of entry 50
  of `BACKLOG.md`).
- **d. The pending ones in flight.** The commitment ties the receiver's identity
  (`crates/zk-ssl/src/two_phase.rs:925`) and the claim demands that the key derive the account's
  identity (`two_phase.rs:1229-1231`). A pending one sent to the old identity before rotating is
  not claimed by the new key, and the rotated account no longer has that identity; the sender's
  refund is `apply_refund` (`two_phase.rs:657-697`), and what to do with them is D-5. No test
  exercises it.
  ⚠️ §802: ✅ now it does, measured and not deduced.
  `la_clave_nueva_no_cobra_el_pendiente_hacia_la_vieja` (`crates/zk-ssl/src/accounts.rs`, at the
  end): after recovering X→Y, the new key generates the claim proof and applying it gives
  `StaleState`, without moving the balance or the pending one. And what this note did not see: the
  OLD key did claim it, by opening another account with the retired identity X —measured in
  private: it claimed 7.000—. Since §802 opening rejects an identity that already has an account or
  that a recovery retired (`IdentityUnavailable`; `SECURITY.md` §3.13), and the pending one only
  returns to the sender by `apply_refund` on expiry —except a duplicate the old key had opened
  before §802, which is the residue of `SECURITY.md` §3.13—:
  `el_robo_tras_una_recuperacion_no_abre_y_el_pendiente_vuelve_al_emisor`. Neither recovery nor
  refund is on the wire.
- **e. The test the seam cites does not exist.** `recovery.rs:105` refers to
  `recovery_deja_view_id_viejo`, and `AUDITORIA.md:12774-12777` takes it as done; grep does not
  find its definition in any crate. `doc/ecst/VERIFICACION.md:1323-1324` already said so.

⚠️ **And one in passing**: the doc of the public recovery counter (`lib.rs:987-993`) sits above
the `frozen` field, not above `recovery_count` (`lib.rs:1025`). It changes nothing; it is said so
that it is not misread.

### 1.4 The absence of time

✅ The layer rotates the custodians **by use**: `custodian_uses` rises with each intervention and
on reaching the quota the custodians stop acting until governance rotates the set, «this layer has
no notion of time» (`lib.rs:1026-1037`; `governance.rs:69-79`). The quota is protected by no
authorization (`governance.rs:42-56`).

✅ **The only clock of the deadline code is `log.len()`**: `let now = self.log.len()` in
de-issuance and in refund (`apply_deissue`, `two_phase.rs:441`; `apply_refund`, `:676`), against
the `refund_ttl` of v1 and against the `delta` of v2 (`:445` and `:459` in the first, `:680` and
`:694` in the second). §119.3 left it said that «`seq` counts operations, not time» and that the
deadline is counted in signed heads (`AUDITORIA.md:9676-9693`); the code does not read the heads
as time.

📐 Consequence for any option: a rotation **with a window** —that the old key or the custodians can
challenge it during a period— inherits either the operation counter, which in a quiet ledger does
not advance, or the head clock, which is not wired to the layer. Without a window, none of this
applies.

### 1.5 What touching an AIR, an operation or the wire costs

✅ Census taken by the name of the climb, `recovery_climb` / `RecoveryClimb`, across the whole
tree:

- **The AIR**: `crates/stark-experiment/src/circuit_recovery_climb.rs`; its replica in the leaf
  machine (`doc/spec-maquina-de-hoja.md:77` and `:105-106`), its interpreter in
  `doc/fv/interprete_recovery_climb.py` and its census in `tools/check_constraint_layout.py:650-651`
  and `:969`.
- **The layer and its tests**: `recovery.rs`, `crates/zk-ssl/src/tests_support.rs:504-529`, the
  calls of `crates/zk-ssl/src/tests.rs` and the `RecoveryClimbAir` row in the suite of what a
  proof reveals (`crates/zk-ssl/src/instrumento_revela.rs:1029-1036` and `:1066`; ⚠️ §842: it said
  `:1023-1030` and `:1060`, which the byte band of each proof shifted by six lines).
- **The command**: `crates/zk-ssl-cli/src/commands.rs:374`.
- **A new operation code**: a `// REGISTRO:` line in the block «EL REGISTRO DE DOMINIOS (§286)» of
  `crates/zk-ssl-hash/src/lib.rs`, its row of the REGISTRO family in the census of
  `spec/NUCLEO.md` (section 4) and what `tools/check_nucleo.py` watches (⚠️ §890: it said
  `crates/zk-ssl-hash/src/lib.rs:1708-1745` and `spec/NUCLEO.md:228-231`, which in §791 were the
  lines of the `OP_*` in the two files and in `212eca2` no longer are; they are cited by name).
- **A new `OpKind`**: `log.rs:117` and `:136`, the count of `persistence.rs:634`, the derivation
  of the quota (`log.rs:520-538`), the re-verifier's class table (`reverificacion.rs:30-52`) and
  the `kind`s of the wire (`spec/RPC.md:102-105`), which is already protocol.
- **A new method**: `spec/RPC.md`, `spec/openrpc.json`, vectors and RFC (`spec/rfc/PROCESO.md:3-5`
  and its rule 4, `:27-28`).
- **The train**: the v2 mark of RFC-0018 will put in each proof its family and the version of its
  AIR, and touching the AIR raises that version; today D-G is written and not built
  (`spec/rfc/0018-el-tren-0-5.md:164-166` and `:221-234`).

## 2. The cases entry 52 joins, one by one

| | case | from where | what is missing, measured |
|---|---|---|---|
| 1 | **new leaf** | §117 (`AUDITORIA.md:9569-9625`); «rotating the key will imply a new leaf» (`native.rs:117-118`) | the second salt in the climb (`circuit_recovery_climb.rs:72-77`) |
| 2 | **view** | §127.5 (`AUDITORIA.md:10487-10499`); «only rotates by rotating the key» (`native.rs:143-146`) | bringing the new `view_id` with the rotation (`recovery.rs:97-106`), and deciding what ties it (§3, N3) |
| 3 | **B18.2 reclamation** | `doc/CONFIANZA_RESIDUAL.md:480-486` (⚠️ §895: it said `:471-477`) and `:574` (⚠️ §895: it said `:565`) | everything: the circuit, the data availability of B18.1 (`:457-467`; ⚠️ §895: it said `:448-458`) and the declaration of B18.3 |
| 4 | **immutable governance** | entry 20 | by design, nothing; the cost is that a compromised governance key is compromised forever. It is fixed by ~~`the_governance_set_survives_restart` (`tests.rs:1176`)~~ ⚠️ §860: `changing_the_governance_set_on_an_existing_ledger_fails` (`tests.rs:2917`), which only changes the governance root and on reopening gives `ParameterMismatch`; the earlier one reopens with the SAME root, fixes something else and is skipped in debug. It is D-3, at G0 (§6), and the root goes in the `params_digest` of each head (`lib.rs:890-898`) |
| 5 | **re-encryption** | entry 20 | see below |
| 6 | **the key of the encrypted notice**, if B11.2 comes out | [`AVISO_EN_ISO20022.md`](./AVISO_EN_ISO20022_EN.md) §6.1 | today it does not exist: no account has a key to encrypt to (`lib.rs:752-767`; `spec/RPC.md:111`) |

✅ **Case 5 measured, which is no longer entry 20's.** The grep of entry 20 is still empty, but
since §702 (RFC-0001) **there is a re-sealing of the whole ledger**: `migrar_a_kdf_v2` opens each
value with the v1 key and seals it again with a v2 one of new salt, in a single sled batch
(`crates/zk-ssl/src/persistence.rs:1031-1065`; ⚠️ §812: here it said `:1012-1056`, the lines from
before §812; ⚠️ §816: and `:1019-1052`, those from before §816); and the keystore is migrated on
opening it (`keystore.rs:11-21`). It is with the **same** passphrase: ~~changing the ledger's
passphrase still does not exist~~ (⚠️ §812: it exists since §812, `SovereignLayer::recifrar`,
further down). And there is a measurement entry 20 did not have: **the node does not encrypt at
rest**. It opens the ledger with `SovereignLayer::open` (`crates/zk-ssl-node/src/main.rs:2258-2261`),
which calls `open_encrypted` without a key (`persistence.rs:20-37`); `SECURITY.md:413-417`
(⚠️ §860: its §3.3, whose lines §860 moves; and entry 127 of `BACKLOG.md`) declares it. For the
node there is no passphrase to change.

⚠️ §812: that re-sealing in one batch left in the ledger's files what was sealed with the v1 key,
without salt: sled is a log, and what is overwritten stays in the directory (`SECURITY.md` §3.14).
Since §812 `migrar_a_kdf_v2` goes through a sibling directory and a rename
(`crates/zk-ssl/src/persistence.rs:1031-1065`), and changing the ledger's passphrase exists in the
library: `SovereignLayer::recifrar` (`persistence.rs:1262-1355`), E4 of RFC-0001. The node still
does not encrypt at rest. ⚠️ §816: the two citations said `:1019-1052` and `:1230-1310`, the lines
from before §816, which adds the ledger's lock to opening and to re-encryption (`SECURITY.md`
§3.14).

## 3. The common core: what any option shares

📐 The transition is one: **the leaf `(id, saldo, nonce, sal)` of a position becomes
`(id', saldo, nonce + 1, sal')`, with `view_id'` in the register, without moving balance**. Today's
climb is that transition with `sal' = sal` forced. Any option needs these four pieces:

- **N1. The new salt, in the AIR.** A second salt column for lane B: the trace width goes from 43
  to 47 (`circuit_recovery_climb.rs:76-77`), and the length does not move, because the wrapping of
  the two lanes goes in the same cycle (`CYC_SALT`, `:92-105`; the final row is 279 of 512). With
  its mutation test, which the house demands of every salt constraint
  (`doc/spec-maquina-de-hoja.md:88-95`).
- **N2. The new `view_id` and salt, in what is presented.** Today `apply_recovery_delegated`
  receives only `new_public_id` (`recovery.rs:52-61`). The three derivations would be done by the
  client, as when opening over the wire (`spec/RPC.md:111`; `accounts.rs:205-209`).
- **N3. What ties each one.** The new salt is tied by `root_new`, because it goes in the leaf. The
  new `view_id` goes in no leaf: it would only be tied by putting it in the commitment of the
  authorization, which today is the roots and the counters (`recovery.rs:145-149`). That changes
  the entry's seal: era 1 of the re-verifier recomposes it with today's parameters
  (`reverificacion.rs:178-198`), and era 2, which carries the commitment in the entry
  (`reverificacion.rs:155-167`), is as the layer already writes (`recovery.rs:184`).
- **N4. The client.** The SDK's `Wallet` stores a single spending key
  (`crates/zk-ssl-sdk/src/lib.rs:93-95`) and the keystore checks on loading that that key derives
  the stored identity (`keystore.rs:25-27`): a rotated account is another file, and nothing in the
  SDK leads from one to the other.

What distinguishes the options is **who authorizes** and **where the new leaf lands**.

## 4. The options, unchosen

### R1. Recovery, completed — custodians, same position

- **Authorizes**: two custodians, as today.
- **Covers**: cases 1 and 2 by the custody route, and closes the seam of §1.3 (a, b and c).
- **Does not cover**: sovereign rotation; each rotation spends quota (`recovery.rs:172-174`) and
  raises the public counter, so a hygiene rotation counts as an intervention; cases 3 to 6; the
  pending ones in flight (§1.3 d) remain out of the new key's reach. ⚠️ §802: and since §802
  recovering towards an identity with an account or a retired one is rejected
  (`IdentityUnavailable`), before verifying: undoing X→Y→X cannot be done either, and an R1 that
  wants to return to an earlier key needs another rule. And whoever knows the new identity before
  it is applied can occupy it by opening an account with it: the recovery then comes out with
  `IdentityUnavailable` (`SECURITY.md` §3.13).
- **Cost**: N1 to N4 on the climb and the layer; the files of §1.5 up to «The command». **Zero
  vectors and zero wire**: recovery is in neither. It breaks the bytes of the climb proof, so it
  goes in a train.
- **Time and race**: no window; the race of `circuit_recovery.rs:58-60` remains.

### R2. Own rotation — the current key, same position

- **Authorizes**: the holder, who proves knowing `sk` as the send and the claim prove it
  (`two_phase.rs:1231`, and in the circuit).
- **Covers**: cases 1 and 2 for hygiene, and for a compromised **view key**: today the holder
  «cannot recover read privacy without changing account» (`AUDITORIA.md:10493-10494`).
- **Does not cover**: the compromise of the spending key, because the attacker rotates first
  (`circuit_recovery.rs:10-17`); cases 3 to 6.
- **Cost**: a new AIR, or the climb with a mode: the climb has no key section
  (`circuit_recovery_climb.rs:22-25`). **It cannot reuse `OpKind::Recovery`**: the log would
  derive a custodian use that did not happen (`log.rs:520-538`) and the public counter would mix
  interventions with rotations (`lib.rs:987-993`). It asks for an `OpKind` and, if it goes on the
  wire, a method, its vectors, its RFC, its row in the suite of RFC-0009 and its family in the
  registry of RFC-0018 (§1.5). And a decision about the frozen account: the climb has no frozen
  phase (`circuit_recovery_climb.rs:42-45`).
- **Time and race**: no window; the resend is stopped by the nonce plus one with the roots, as in
  recovery (`recovery.rs:287`).

### R3. Precommitted rotation — the next key, the pattern of RFC-0015

- **Authorizes**: knowing a next key whose commitment the account fixed before. It is the pattern
  with which RFC-0015 rotates the node's signer, which «does not start with a key nobody
  committed» (`spec/rfc/0015-el-ciclo-de-vida-del-firmante.md:1` and `:38`).
- **Covers**: cases 1 and 2, and **the compromise of the spending key without custodians**, if the
  next one was kept apart: whoever steals `sk` does not have the committed one.
- **Does not cover**: cases 3 to 6; losing the next one sends back to the custodians.
- **Cost**: the commitment has to be where the operator cannot change it. **In the leaf**, it
  changes the preimage in the twelve AIRs that redo it —`circuit_audit`, `circuit_banda`,
  `circuit_burn`, `circuit_claim`, `circuit_claim_v2`, `circuit_credit_climb`, `circuit_mint`,
  `circuit_mint_climb`, `circuit_recovery` (which the layer no longer calls),
  `circuit_recovery_climb`, `circuit_send` and `circuit_send_v2`, according to the grep of
  `leaf_salt` in `crates/stark-experiment/src/`—, the core function
  (`crates/zk-ssl-hash/src/lib.rs:205-212`) with its new KAT next to
  `spec/vectors/nucleo/native_leaf_salted.json`, and a migration of all the leaves: the B13/B14
  class. **Only in the register**, the binding is the operator's word, who keeps the registers
  (`lib.rs:745-750`). Plus everything of R2.
- **Time and race**: it needs no window. RFC-0015 avoids it because whoever judges the acts is the
  witness, not the layer.

### R3s. The next one committed in the salt — ⚠️ §860: added, and adopted with three gates

📐 The variant of R3 that does not touch the leaf: the account that wants it opens with `sal =
merge(derive_leaf_salt_wide(sk), H(DOM_SIGUIENTE, sk_siguiente))`, and the rotation proves knowing
`sk_siguiente`. §860 added it, from the objection of a review of entry 52.

- **Authorizes**: as R3, knowing the next key.
- **Covers**: what R3 covers —cases 1 and 2, and the theft of `sk` without custodians—, and it is
  the hygiene that does retire.
- **Does not cover**: cases 3 to 6; losing the next one sends back to R1, which with N1 writes a
  clean salt.
- **Cost**: ✅ in the twelve AIRs with `leaf_salt`, every derivation of the salt is inside the test
  module —the first line of `derive_leaf_salt` always comes after the `#[cfg(test)]`, and
  `circuit_banda` does not have it—; the salt is a witness the AIRs only read
  (`crates/stark-experiment/src/circuit_claim.rs:246`: «the witness salt is only READ in
  link_salt»; in the constraints of `circuit_send.rs`, `COL_LEAF_SALT` appears only in the salt
  link, `:1067` and `:1069`), and «the protocol does not impose the origin»
  (`crates/stark-experiment/src/native.rs:117-119`); `zkssl_openAccount` receives the salt from the
  client (`accounts.rs:205-210`). 📐 So R3s touches neither the preimage, nor the leaf's KAT, nor
  migrates leaves, nor the accounts that do not ask for it: it adds an authorizer. It asks for N1
  with a key section and the check of the composite salt in the rotation AIR, its `OpKind`, the
  `DOM_SIGUIENTE` domain with its KAT and the cold key in the SDK.
- **The three gates** (D-1 of §6): (i) N1 exists; (ii) entry 119 of `BACKLOG.md` ties the salt as
  a public input of its opening proof and does not impose its origin; (iii) measured that an
  account with a composite salt opens, receives, sends, claims and audits without touching any
  AIR, and T2b written with the two keys, against §117.1 and §93.4. If one falls, the hygiene that
  retires passes to R2.
- **Why (ii)**: ✅ the client proves with the salt the materials give it, not with the one it
  re-derives (`client.rs:366`, `:400`, `:492` and `:517`), and the operator writes the leaf on
  opening without a proof (entry 119). 📐 If R3s existed before entry 119, the operator could write
  on opening a salt with ITS OWN committed next one; the holder would not notice, because their
  proofs use the materials' salt, and the operator would rotate the account to itself with a valid
  proof, against «Nobody spends without being the holder» (`PRINCIPIOS.md` §0). Today it is not a
  failure, because R3s does not exist: it is a design rule, and it holds the same for R3 in the
  leaf.
- **Time and race**: no window, as R3.

### R4. Transfer — a new account and a payment, without code

- **Authorizes**: the holder, with a send and a claim.
- **Covers**: cases 1 and 2, because the new account opens with the salt and the view of the new
  key (`accounts.rs:170-178`). It is **the only one** in which the pending ones in flight are still
  claimed: the old account keeps its identity, and the old key claims them in it
  (`two_phase.rs:1229-1231`) to transfer them again (in the others, §1.3 d).
- **Does not cover**: compromise, because the attacker transfers first and to their own account;
  cases 3 to 6. The old account stays alive with the old key and keeps receiving; the bridge's IBAN
  register, which is local (`crates/zk-ssl/src/iso.rs:297-299`) and is not persisted
  (`crates/zk-ssl/src/migration.rs:46-47`), has to be re-pointed by hand.
- **Cost**: **no file**. The operational: two proofs per rotation, those of a payment; a slot of
  the account cap (`lib.rs:1043-1054`), which opening does not authorize (entry 8 of `BACKLOG.md`;
  ⚠️ §860: closed, with no custodian authorization to open; and with the cap exhausted, there is no
  R4); the old → new link, visible to whoever sees the two halves (`spec/RPC.md:344-347`) and to
  the operator. A frozen account cannot: the send checks it (`two_phase.rs:859`).
- **Time and race**: no window.

### R5. The reclamation as rotation — B18.2 generalized

- **Authorizes**: the holder's key, opening its leaf under a signed root —the `H*` of the disaster
  or a live head— (`doc/CONFIANZA_RESIDUAL.md:480-486` (⚠️ §895: it said `:471-477`)); the
  custodians, if wanted, as backup.
- **Covers**: case 3 by construction, and 1 and 2 if the new leaf is written with new identity,
  salt and view. And 4 **sideways**: «changing it requires creating a new ledger»
  (`lib.rs:975-983`) stops being fatal for the holders if each account can pass by reclamation to a
  successor with another governance root (G2, below).
- **Does not cover**: case 5. It is the most expensive.
- **Cost**: a new circuit of the `circuit_claim` class; a register of positions already reclaimed
  in the successor —the precedent is the `claimed` map of the pending tree
  (`crates/zk-ssl/src/pending.rs:204`)—; B18.1 (`doc/CONFIANZA_RESIDUAL.md:457-467` (⚠️ §895: it
  said `:448-458`)), B18.3 and B12.1; method, vectors, RFC and second implementation. And its
  precondition, that the holder re-derive the salt with their key, **fails in recovered accounts**
  as long as N1 does not exist (§1.3 b).
- **Time**: the reclamation window is «policy, not parameter»
  (`doc/CONFIANZA_RESIDUAL.md:496-499` (⚠️ §895: it said `:487-490`)): it needs a clock, and
  inherits §1.4.

### Governance and re-encryption, which are not account keys

- **G0. Governance, immutable** as today: zero cost, and the one declared by entry 20.
- **G1. That governance rotate with its own threshold**: a route like `apply_governance_delegated`
  (`governance.rs:104-169`) with a new operation code (§1.5), the root mutable and persisted, and
  `params_digest` changing between heads (`lib.rs:890-898`). ❓ What the witness does when
  `params_digest` changes between two heads is not measured here; rotating the custodian set
  already moves it today. The circularity moves again: two compromised governance keys would
  rotate governance to their own.
  ⚠️ §860: the ❓, resolved by reading: rotating the custodians already moves `params_digest`
  between heads (it composes it with `custodian_set_root`, `lib.rs:890-898`), and the witness reads
  it only as one more field of the head it recomposes (`crates/zk-ssl-cli/src/witness.rs:569`,
  `:1093` and `:1113`): G1 asks nothing new of the witness. And «without witnesses, immutability
  binds governance and not the operator, who can constitute another ledger» is a limit common to
  all the governance options, not a reason for any.
- **G2. By R5**: a new ledger with another root and the reclamations of all the accounts.
- **G1-M. That governance rotate with a larger quorum** — ⚠️ §860, added: the self-rotation of G1
  with a threshold larger than that of its ordinary use. It would cure the compromise of a
  minority and would only let a majority entrench itself. 📐 It asks for a new constitution
  parameter, N or the quorum: the layer only knows the root (`params_digest` composes seven values,
  and none is N or a quorum, `lib.rs:890-898`). Uncosted: it is the best candidate if D-3 is
  reopened.
- **G3. Precommitted governance** — ⚠️ §860, added: the R3s pattern for governance, as an option
  when constituting the ledger. Uncosted: it is designed with R3s (D-3).
- **K0. Re-encryption, as today**: only the library, and the node in the clear.
- **K1. Changing the passphrase**: generalizing `migrar_a_kdf_v2` to two passphrases
  (`persistence.rs:1050-1065`; ⚠️ §816: it said `:1038-1052`); in the node, first, wiring the
  encryption (`SECURITY.md:413-417`; ⚠️ §860: its §3.3, and entry 127 of `BACKLOG.md`); in the
  keystore, `load` and `save` already suffice (`keystore.rs:262` and `:280`). No circuit and
  nothing on the wire, like RFC-0001, which did not move `zkssl/0.4`
  (`spec/rfc/0001-el-kdf-de-reposo.md:21`; ⚠️ §816: it said `:16`, before §816 added five lines to
  its header). ✅ **It shares no piece with the account options** except `persistence.rs`, which
  persists the registers: if it enters the single design it is by calendar, not by mechanism.
  ⚠️ §812: done, as E4 of RFC-0001 and in the library: `SovereignLayer::recifrar`
  (`persistence.rs:1262-1355`). Generalizing `migrar_a_kdf_v2` as it was would have inherited its
  residue: the in-place batch left in the files what was sealed with the old key (`SECURITY.md`
  §3.14). Both now go through a sibling directory (`persistence.rs:1357-1480`). The two citations
  above said, until §812, `persistence.rs:1022-1056` and `0001-el-kdf-de-reposo.md:12`: the second
  was already shifted one line since §805, and §812 moved both. The node, unwired: it does not
  enter what the author asked for, the library's E4. ⚠️ §816: the two of this paragraph said
  `persistence.rs:1230-1310` and `:1312-1425`, the lines from before §816.

### V. Only the view, with a counter

📐 A variant of case 2 that does not rotate the spending key: a **public** counter in the
derivation of the view. It is not a new secret —§127.3 ruled out the independent secret for being
the fatal loss mode that §117 rejected (`AUDITORIA.md:10455-10469`)—, but it changes that decision,
and fixing the new `view_id` still asks that `sk` authorize it with a proof: it is R2's class of
cost without touching the leaf. ⚠️ §860: ruled out (D-1): it changes the decision of §127.3, costs
like R2, and R4 already covers that case.

## 5. The matrix

| | 1 leaf | 2 view | 3 B18.2 | 4 governance | 5 re-encryption | 6 notice | theft of `sk` | AIR | wire |
|---|---|---|---|---|---|---|---|---|---|
| R1 | yes | yes | — | — | — | — | yes, with custodians | the climb | no |
| R2 | yes | yes | — | — | — | — | no | new, or the climb with a mode | yes |
| R3 | yes | yes | — | — | — | — | yes, without custodians | the twelve of the leaf | yes |
| R3s ⚠️ §860 | yes | yes | — | — | — | — | yes, without custodians | none of the leaf's; the climb (N1) with a key section | yes |
| R4 | yes | yes | — | — | — | — | no | none | no |
| R5 | yes | yes | yes | by G2 | — | — | depending on who authorizes | new | yes |
| G1 | — | — | — | yes | — | — | — | none | ❓ |
| G1-M ⚠️ §860 | — | — | — | yes | — | — | — | none; a constitution parameter | ❓ |
| K1 | — | — | — | — | yes | — | — | none | no |
| V | — | yes | — | — | — | — | no | none of the leaf's; a proof of `sk` | yes |

Case 6 is covered by none because there is no key yet: if B11.2 creates it, it will rotate with
the option chosen here, or it will be derived from `sk` and rotate with it (D-9).

## 6. Open decisions for the author

⚠️ §860: none is open any more: the author resolved D-4 in §812, and the other eight are taken by
delegation (above, in the «Status»), each with its condition for reopening.

- **D-1. Who authorizes** an account rotation: the custodians, the current key, a committed next
  one, or a combination —for example, the current one for hygiene and the custodians for theft—.
  ✅ **Resolved by delegation (§860), REVERSIBLE**: a single design, the N1-N4 transition of §3 with
  several authorizers. Theft, loss and any suspicion about `sk`: the custodians (R1, recovery
  completed, which retires the identity). The compromised view alone and hygiene without
  suspicion: the transfer (R4), without code, with its cost declared: it retires neither the old
  key nor the old identity —they keep claiming and opening the view—, it spends a slot of the cap,
  it leaves the link between the two accounts visible and it does not serve a frozen account; and
  with the account cap exhausted (entry 8 of `BACKLOG.md`), there is no R4. R3s is ADOPTED,
  optional for each account on opening it, and is built after its three gates (§4); if one falls,
  the hygiene that retires passes to R2, and when it enters, R1 stays as backup. R2 is not built;
  R3 in the leaf is ruled out for its cost, which R3s avoids, and R3 only in the register, because
  it would tie the authorization to the operator's word; R5 is a future authorizer of the same
  transition. V is ruled out (§4).
- **D-2. Where the new leaf lands**: in its position, in another of the same ledger or in another
  ledger.
  ✅ **Resolved by delegation (§860), REVERSIBLE**: in its position. To another position of the same
  ledger, no: it would break «no operation touches two accounts», which entry 47 of `BACKLOG.md`,
  closed in §860, leaves written, and its non-atomic version is already R4. To another ledger, only
  with the B18.2 reclamation, when there is B18.
- **D-3. Whether governance enters**: G0, G1 or G2.
  ✅ **Resolved by delegation (§860), REVERSIBLE**: G0, the same as for entry 20, which is closed. G1
  is not chosen now; G1-M and G3 are added uncosted. It is reopened (a) before constituting the
  first deployed ledger, costing G1-M against the concrete use case; (b) if a use case asks to renew
  its governors without a new ledger; or (c) when R3s is built, and then G3 is designed with it, as
  an option when constituting the ledger. G2 arrives with the B18.2 reclamation and reopens
  nothing. A ledger constituted in G0 stays in G0.
- **D-4. Whether re-encryption enters or goes apart**: K0 or K1, and whether the encryption is
  wired in the node first.
  ✅ **Resolved by the author (§812)**: K1, apart from the single design, as E4 of RFC-0001, on
  asking for it; wiring the node's encryption does not enter what he asked for, the library's E4.
- **D-5. The pending ones in flight** towards the old identity (§1.3 d): whether they are accepted
  stranded until the refund, whether the rotation waits for them or whether the old identity keeps
  claiming. ⚠️ §802: the third can no longer be a new account with the old identity —a retired
  identity does not open again—; only a design in which the rotated account accepts its earlier
  identity. And every rotation that retires an identity has to add it to the same set of retired
  ones, in the same batch.
  ✅ **Resolved by delegation (§860), REVERSIBLE**: stranded until the sender's refund, and it is
  declared (`SECURITY.md` §3.13). That the rotation wait for them (D-5b), only as advice to the
  client. That the rotated account accept its earlier identity (D-5c) is ruled out for its cost
  —touching `circuit_claim` and `circuit_claim_v2`—, and is reopened if those AIRs are reopened for
  another reason. 📐 And what an account sent by v2 before rotating no longer returns to it by
  refund: `apply_refund` demands that the sender's account keep the identity of the commitment,
  and gives `PendingMismatch` (`crates/zk-ssl/src/two_phase.rs:710-714`); it only gets out if the
  receiver claims it. It fails closed; no test yet.
- **D-6. The counter**: whether an own rotation counts in `recovery_count` or has its own.
  ✅ **Resolved by delegation (§860), REVERSIBLE**: `recovery_count` counts only custodian
  interventions; R4 has no counter; R3s, when it enters, carries its `OpKind`, without quota and
  without `recovery_count`.
- **D-7. The frozen account**: whether it can rotate, and whether rotating can take it out of its
  position.
  ✅ **Resolved by delegation (§860), REVERSIBLE**: no rotation takes an account out of its
  position, so a frozen one stays frozen. Today only the custodians rotate it, because R4 rejects
  it; R3s will be able to rotate it in place, and it will stay frozen.
- **D-8. The train**: the 0.5 of RFC-0018 or another.
  ✅ **Resolved by delegation (§860), REVERSIBLE**: N1 goes in the `zkssl/0.5` train, before the
  cut; if the cut comes first, in the next one, with whatever AIR version is due.
- **D-9. The key of the encrypted notice**, if B11.2 comes out: derived from `sk`, and rotating
  with it, or independent, which is the new secret that §117 and §127.3 did not want.
  ✅ **Resolved by delegation (§860), REVERSIBLE**: derived from `sk`, with its own domain, and
  rotating with it; in the symmetric route of B11.2 (entry 11 of `BACKLOG.md`), one per
  relationship. The independent one is ruled out by the decision of §117 and of §127.3.

## 7. What this note does not do, and the next step

It touches no code, nor the wire, nor a circuit; it spends no RFC number; it does not choose; it
closes neither entry 52 nor entry 20. ⚠️ §860: the author delegated the choice, and it was taken
in §860 (§6): it closes entry 20; entry 52 continues until N4, and on closing it, R3s and B18.2
pass to an entry of their own if they have not been built. The steps that follow need to compile,
and this session does not compile:

1. the test `recovery.rs:105` cites and does not exist: a recovery leaves the old `view_id`, and
   the new key does not pass `account_view_authenticated`;
2. the pending one in flight after a recovery (§1.3 d), measured with a test and not deduced;
   ⚠️ §802: done, with the theft it uncovered closed (§1.3 d);
3. of the option chosen, its mutation test on the new salt (N1).
