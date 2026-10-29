<!--
  CADUCIDAD_PENDIENTE — the issuer's refund after T, with a double lock.
  Design document. It closes the only live debt that §177 left noted:
  «la caducidad del pendiente inmovilizado» [the expiry of the immobilised
  pending]. Sibling of ANCLAJE_EXTERNO.md.
  It does not touch the commitment, does not touch claim, does not touch
  the discriminants of §173.
-->

# CADUCIDAD_PENDIENTE — that money in transit has a clock

> **Español:** [`doc/CADUCIDAD_PENDIENTE.md`](./CADUCIDAD_PENDIENTE.md) is the original of this
> page, in Spanish; it prevails wherever the two differ. This translation is of commit `7766efb`
> (audit entry §917) and was produced by the project's assistant, a Claude Code session, in audit
> entry §918 (`GENAI.md`): it is a generated artefact, not text written by the author. Line
> citations of the form `file:N` refer to the Spanish files.

## 0. The problem, with its record

Two-phase immobilises: `send` debits the sender and creates the
commitment `P = H(H(id_receptor, salt), importe)`; only the receiver's
`claim` collects it. If the receiver never claims —lost key, frozen
account (§16.2), or simply never—, the funds stay still **for ever**:
today there is no operation to cancel, expire or return. It is a
**DECLARED cost** (§29-§30, entry 12) and less limbo than the one-step
path it replaced (§7/§36) — but §177 flagged it as **the only front
where the notes model would offer something** (time-locks) that is
missing here. This document designs that something **on the current
model**, as §177 prescribed.

## 1. The attack that kills the naive design — and the double lock

**Refund-by-opening is stealable.** Today, leaking the notice
(`position, salt, amount`) is harmless: `claim` binds the receiver's key
to the `id_receptor` of the commitment (§39.1, `C_PK_CHECK`) — knowing
the notice does not collect. A refund that only demanded *knowing the
opening of P* would break that: the thief-with-notice would refund **to
their own account**. The design shuts the door twice:

1. **The destination is fixed by the records, not by the prover.** In
   `apply_send` the layer already receives `sender_index`; it notes
   `pending_meta[pos] = { sender_index, born_seq }` beside the
   `pending_amounts` that **already exists** (with the same honesty note:
   operator metadata, who already knows the sender — §21/§129; third
   parties do not see it, the discriminants of §173 stay intact). The
   refund credits `meta.sender_index` **whoever proves**: the thief, at
   worst, pays the sender's paperwork.

2. **Only the sender can fabricate the materials.** The credit demands
   the climb of ITS leaf —complete preimage: `public_id`, balance, nonce
   and `leaf_salt`, which derives from ITS spend key (§117)—. The notice
   does not contain it. Double lock: without the key there are no
   materials; with someone else's materials there is no destination.

## 2. The mechanism

**The operation**, one-party (like `claim`):

```text
refund(pos, apertura, subida_de_crédito)
  CAPA verifica:  seq_actual − meta[pos].born_seq ≥ T     ← el reloj
                  hoja[pos] == P(apertura)                 ← sigue viva
  CIRCUITO refund: apertura de P  +  camino P → vacía      ← nuevo, pequeño
  CIRCUITO crédito: mint_climb EXISTENTE, tal cual         ← reuso
                  hoja(saldo) → hoja(saldo + importe)
                  sobre la cuenta meta[pos].sender_index   ← destino fijado
  COMMIT: lote con {hoja emisor, hoja pendiente, meta}     ← la lección §169
```

- **`circuit_refund`** is of the light-claim class: it proves the opening
  of the commitment (the same `C_PEND_IN` reconstruction that claim
  already does) and the emptying of the pending path. **No PK-check**
  (the destination is already fixed), **no nullifier** (the empty leaf
  is its own anti-replay: the second attempt does not find P), **no
  frozen-check** (returning is not collecting). The layout guardian will
  go from 26 to **27 circuits**. ⚠️ **Overtaken on execution**: they
  were TWO circuits (#27 opening + #28 credit) and the guardian says
  **28** — see the closing note of this document and `AUDITORIA.md`
  §178-§180.
- **The credit reuses `mint_climb`** without touching a comma: it is
  exactly «leaf(b) → leaf(b+amount), same pid/nonce/salt».
- **The clock**: `born_seq = epoch_head().seq` on applying the send. `T`
  is a **declared systemic line** —the family of `N_max` (§121) and `M`
  (§174)—: it is chosen, published, measured. When signed heads exist
  (§115), `T` inherits their clock without changing the design;
  stretching the heartbeat to dodge `T` is already opposable evidence
  (§121.4).

## 3. The race, declared

After `T`, **claim and refund compete and the first to be applied wins**:
both empty the position, the second collides with an empty leaf and
bounces. It is not a defect but the chosen semantics: once the deadline
has passed, the receiver's right stops being exclusive. The diligent
receiver claims before `T`; the sender does not recover before `T`. **A
single parameter splits time into two rights.**

## 4. ISSUANCE pendings are un-issued

`mint_to_pending` creates pendings **without a sender account** (the
money is born there). Their expiry does not refund: it **un-issues** —the
leaf is emptied and `total_supply` drops by the amount—, with the clean
symmetry that *uncollected issuance ceases to exist* (the custodians can
re-issue if appropriate, through the delegated path). Same gate `T`, no
credit climb, no extra circuit: the opening of the commitment suffices
and the destination is accounting nothingness.

## 5. What this design does NOT touch — with witnesses

- **The commitment**: identical. `el_commitment_no_revela_al_emisor`
  (§173) stays green — the sender enters the operator's RECORDS (who
  already knew it), never the tree.
- **`claim`**: not one constant. The pre-`T` receiver claims exactly as
  today.
- **The notice**: same `PendingNotice`, same harmlessness when leaked —
  the double lock guarantees it.
- **Unlinkability**: the salt is still the declared carrier (§173);
  `refund` adds no correlation visible to third parties.

## 6. What it does NOT solve

- **The sender with a lost key** does not recover either (it cannot
  fabricate the climb). Its case —and that of operations orphaned of
  every key— is **phase 2**: a sweep to a remainder with a claims window,
  which is a political decision of the **B18.3** family (§87/§88) and is
  discussed with that family, not here.
- **The operator can fabricate refund materials** (it has the records).
  It is benign maintenance —the destination stays fixed to the sender—
  and it is declared: an operator can only *accelerate* returns, never
  divert them.
- **`T` is a judgement**, not a theorem: short punishes slow receivers,
  long lengthens the immobilisation. It is declared and reviewed with
  data, like `N_max`.

## 7. Cost and first measurable step

One structure (`pending_meta`, beside `pending_amounts` and persisted in
the batch), one layer gate, **one small circuit** (claim-without-identity
class; the guardian will say 27), the reuse of `mint_climb`, and the
tests: the legitimate refund after `T`, the pre-`T` rejection, the race,
the thief-with-notice (bounced by destination and by materials), the
replay (empty leaf), the un-issuance of the pending mint, and the
persistence of the meta across a restart — the lesson of the orphan
commit (§169) written as a test from day one. **First step**:
`circuit_refund` with its layout in the guardian and its discriminants,
before touching the layer. It is circuit work: **a session of its own**,
as the house demands for everything that introduces new constraints.

## 9. Execution ERRATUM (§180) — the reuse of the climber, falsified

This document designed the credit as a reuse of `mint_climb` «without
touching a comma» (§2) and promised ONE circuit (§7). **The tree
falsified it on execution**: `mint_climb` binds
`supply_new − supply_old == importe` in circuit (its `C_SUPPLY`;
`supply_delta` exists so that covert issuance is REJECTED, not to dodge
it). A refund does not issue, and lying to a circuit is not of this
house. The answer: **`circuit_credit_climb`** (#28), a twin with three
marked cuts and the discriminant of lock #2 (someone else's salt does
not verify). Expiry uses TWO circuits —opening #27 + credit #28— and the
guardian says **28**. Full record: AUDITORIA **§178-§180**. The rest of
this design (double lock, T, race, un-issuance, legacy) remains in force
as is.
