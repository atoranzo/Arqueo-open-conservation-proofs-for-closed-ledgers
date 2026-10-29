# Errata of the published preprints

> **Español:** [`doc/preprints/ERRATA.md`](./ERRATA.md) is the original of this page, in Spanish; it
> prevails wherever the two differ. This translation is of commit `7766efb` (audit entry §917) and
> was produced by the project's assistant, a Claude Code session, in audit entry §918 (`GENAI.md`):
> it is a generated artefact, not text written by the author. Line citations of the form `file:N`
> refer to the Spanish files.

**What this file is.** The preprints in this directory are
**deposited with a DOI**: they are published artefacts, and the tree
keeps them **verbatim**. When an error or an omission is detected in one
of them, **the body is not edited**: it is recorded here.

**Why.** The project corrects its living documents and **leaves the
sequences visible** in the published ones. Rewriting a deposited
artefact **falsifies the referent** of the corrections already
published about it (`AUDITORIA.md` §119, §122) — the same criterion by
which `doc/ESCALADO.md` and `doc/CONFIANZA_RESIDUAL.md` were committed
with their body intact and a **map header** on top, instead of fixed.

**What it is NOT.** It is not a revision. A reader who downloads the
deposited PDF **does not see this file**: what is recorded here reaches
them in the next revision, or does not. Recording an erratum **does not
correct it for whoever already read it**, and saying otherwise would be
the kind of claim this project punishes.

---

## Entry 1 — `ZK-SSL-residual-trust.md` §4.1: it omits the greatest power

**Detected**: 2026-08-02 · **Backlog**: entry 54 · `AUDITORIA.md`
§128.2, §135.

The table «What the operator can still do» lists ordering, censoring,
observing the state and keeping custody of the log. **It omits the
greatest of all: replacing the verifier.**

> Ordering, censoring and observing act **within** the rules. Replacing
> the verifier **redefines the rules** under which all the others are
> judged: an operator who changes it can accept as valid what the
> published rules would reject.

And today it **leaves no trace**: the system has no notion of «rules in
force» —`OpKind` says which circuit to use, not which version was
active—, so the change is not checkable after the fact.

⚠️ **It is the fourth blindness of the framing of §4.1**, alongside the
three the document itself already acknowledges —the confidentiality
leak, the capacity limits and the privilege that does not expire—. A
document whose contribution is **naming residual trust** is falsified by
the one it failed to name; its own §24 says so.

**Designed closure**: `hash_verificador_vigente` in the attested epoch
head, which makes every update public
(`doc/CONFIANZA_RESIDUAL.md` §2.2). **It depends on first giving the
system that notion of rules in force**, which does not exist.

**Where the content lives today**: `SECURITY.md` §2, in the list of what
the system does not protect.

**Corrected in the fourth revision** (2026-08-12, deposited version:
[10.5281/zenodo.21905595](https://doi.org/10.5281/zenodo.21905595)): the
table of §4.1 incorporates the row «Replace the verifier itself» — the
power, its lack of a trace today, and the designed closure
(`hash_verificador_vigente` in the attested head) with its condition.

---

## Entry 2 — Four correction notes were inserted IN the body

**Detected**: 2026-08-02 · `AUDITORIA.md` §100, §135.

The fourth revision added **four correction notes inside the body** of
`ZK-SSL-preprint.md` and `ZK-SSL-policy-note.md` —on the spend key, the
claim, the incomplete correction of §8.2 and the MiB unit—.

**Their content is correct and remains in force.** What is not: **they
used the vehicle this file declares wrong**, and they did so before this
file existed.

⚠️ **It is recorded instead of undone**, for three reasons:

1. Those notes **are already deposited** in the fourth revision:
   withdrawing them now would leave the published PDF without them and
   the tree too.
2. Withdrawing them would be **rewriting the artefact once more** — the
   same error, in the opposite direction.
3. **An errata file that declares a rule without admitting that the
   directory breaks it in four places would be born false.**

**From here on**: every correction to a published preprint enters
through this file. The four notes remain as **the documented
exception**, not as the pattern.

---

## Entry 3 — `ZK-SSL-residual-trust.md` §4.1: the condition of the closure

**Detected**: 2026-08-19 · **Backlog**: entries 54 and 83 · `AUDITORIA.md`
§246, §321.

The **fourth revision** (2026-08-12,
[10.5281/zenodo.21905595](https://doi.org/10.5281/zenodo.21905595))
incorporated into the table of §4.1 the row «Replace the verifier
itself» **with its condition**, as entry 1 of this file recorded it: the
closure —`hash_verificador_vigente` in the attested head— **would depend
on first giving the system a notion of «rules in force»**.

**That condition is incorrect, and the tree had corrected it in §246**
—before the revision that published it—. It is **almost circular**:
`hash_verificador_vigente` **is** the mechanism for having that notion,
so the condition asks for itself.

> **The real reason: the AIR is code, not data.** What the field must
> give away is what a valid transition is, and that is defined by the
> AIR. The only thing hashable at run time is the `ProofOptions`, and
> **an operator can change the AIR leaving them identical**. A
> `verifier_hash` like that would not be an empty field: it would be a
> **blind** field — and a blind field goes unnoticed **lying precisely
> about what it exists to detect**.

⚠️ **And both ways out are closed for reasons outside the protocol**:

- **Hashing the source at compile time** does not prove that the binary
  was built from that source. Without a **reproducible build**, the
  operator reports the recorded hash and runs something else: it lies in
  the case that matters.
- **The AIR as data** —entry 55— would indeed be hashable, and it is
  halted for a reason that is not effort: *a specification written by
  whoever wrote the circuit inherits its blind spots*, and it must be
  written **with the audit, not before it**.

**Consequence for the reader of the preprint**: the designed closure is
still the right one, but **it is not an implementation task pending**:
it is an architecture decision with two preconditions that live outside
the protocol. The published condition underestimates the cost.

**Entry 1 is not edited** — this file grows by addition: it stays as it
was written, and this entry corrects it on top. The tree's living text
is corrected in `BACKLOG.md` (entry 83; entry 54 already said so),
`SECURITY.md` §2, the map header of `doc/CONFIANZA_RESIDUAL.md` and
`crates/zk-ssl/src/log.rs`, inside `EpochHead` itself.

---

## Entry 4 — The three preprints: the repository URL they cite is no longer the repository's

**Detected**: 2026-09-04 · **Corrected in the tree**: 2026-09-05 (`AUDITORIA.md`
§403).

The three deposited preprints give as repository
`https://github.com/atoranzo/ZK-SSL-ZK-Sovereign-Settlement-Layer-` (in
the header of each one and in its list of artefacts or references). The
repository was renamed to
`https://github.com/atoranzo/Arqueo-open-conservation-proofs-for-closed-ledgers`,
and the old URL only resolves while GitHub redirects it: **it expires by
someone else's decision** the day someone creates a repository with that
name.

**What was done in the tree, and what was not.** The six lines were
moved in the three files of this directory, as on 27-08-2026 their DOIs
were moved (`README.md` of this directory, «sólo se tocaron
identificadores» [only identifiers were touched]): the URL is an
identifier of the same class, and two kinds of treatment for one same
class of object would be what this project punishes. **Not one figure,
not one claim, not one paragraph was touched.** Since that commit the
three files differ from the deposited PDFs in the DOI lines and in the
URL lines, and nothing else; a gate is also born (`tools/check_publicadas.py`,
ATADO C) that turns the canon red if the old URL returns to a living
document.

**For the reader of the deposited PDF**: the link works today by
redirection, and this entry does not fix it for them. The URL in force
is the one above; the next deposit will carry it inside.

---

## Entry 5 — The three preprints: «the expensive half is the receiver's» is not supported by what was measured

**Detected**: 2026-08-14 · **Backlog**: entry 96 (closed in §860, merged
into entry 28) · `AUDITORIA.md` §304, §512, §538 ·
`spec/rfc/0009-lo-que-revela-una-prueba.md` D-AJ.

The three deposited ones say that generating the claim proof costs
«roughly 500 ms against 283 ms for a send» (`ZK-SSL-policy-note.md:235`,
`ZK-SSL-residual-trust.md:182`; `ZK-SSL-preprint.md:329`, with «to
generate» after «500 ms»). Two of them give the cause: «because the claim
circuit walks both the pending tree and the accounts tree». And they draw
a conclusion from it: the cost of a payment falls on whoever receives. In
the policy note it is advice («a fact worth knowing before pricing
anything»), and in the residual-trust one a normative argument («the
party with the least choice about participating carries the larger
cost»). The 500 against 283 amount to a claim/send ratio of 1.77.

**What was measured after depositing them, in order**:

- **2026-08-14** (§304): the other way round. Four runs: send 260.7-286.8
  ms, claim 159.4-189.6 ms, and the bands do not overlap. The claim cost
  less than the send. It was measured through the layer's path, which
  joins the client's work with the layer's, and that is why §362 took the
  measurement to the client's path. Through that path the direction
  stayed the same until §511: measured paired on `3db491f`, send 240.8 ms
  and claim 160.2.
- **2026-09-19** (§512): fix B (5.A-272) inverts the order again.
  Measured paired: the claim/send ratio goes from 0.67 to 1.75, close to
  the 1.77 that corresponds to what was published. The direction is the
  published one again, but not for the published cause: §512 attributes
  it to fix B's accumulator, and the why of that difference is not
  explained.
- **2026-09-24** (§538, D-AJ): with the hidden witness, each side costs
  700-750 ms and both measure the same within the noise. Of three runs,
  two gave receiver and one payer (send/claim ratio of 0.975 and of
  1.064). The canon stopped asserting the direction.
- **2026-09-28 and 2026-09-30** (`doc/ecst/ECST.md` §7.2): another
  machine, a Firecracker microVM with 4 vCPUs, with
  `simulate --amount 250000` from the CLI and one run per day —the
  layer's path, the one §362 discarded as the object of this relation—:
  send 970 ms and claim 1,198 ms the first day; send 1,333 ms and claim
  879 ms the second.

**What is not measured**: the relation on today's tree. Since §538 the
circuits of the payment path changed (§641, §680) and so did the prover
(§652), and the canon prints the times without asserting or storing
them. The seal that writes this entry did not measure them: **NOT
MEASURED**.

**Consequence for the reader of the preprint**: what was published —the
claim, 77 % dearer than the send— was contradicted in §538, where both
sides measured the same within the noise, and today it is not measured
which side of the payment is the expensive one. And the published cause
was already wrong when deposited: in `6384907` (2026-07-30, the commit
that brought the published text into the tree) and in `69fa2d4`
(2026-08-01), `circuit_send.rs` and `circuit_claim.rs` have the same
geometry —`TRACE_LENGTH` 1024, the accounts root at row 271 and the
pending one at row 743—, and the send carries one more segment (5
against 4) and one more column (49 against 48 in the first, 52 against
51 in the second). In `3db491f`, the parent of fix B, the send and the
two claim circuits climb through the same cycles of the two trees
(`CYC_ACC..CYC_PK` and `CYC_PEND_CLIMB..CYC_FIN`), and through the
client's path the claim cost 160.2 ms and the send 240.8 (§512). What is
read in the code is MEASURED; that «walking the two trees» does not tell
the cost of the two sides apart is DEDUCED. The times depend on the
machine, and this project does not publish absolute times as a property
(§304). Whoever is going to set prices must not rely on that sentence.

**And the bytes of the same paragraph** —«Each proof is about 62 KB and
that has not changed» and the «120.4 MiB» per thousand payments, «MB» in
the policy note— also moved: since §538 a send proof weighs
73,571-84,244 B and a claim one 72,382-83,723 B, and a thousand
payments, 139.2-160.2 MiB (the D-AC band of RFC-0009, in
`crates/zk-ssl/src/metrics.rs`). They are not the object of this entry;
the next revision picks them up with it.

**Correction**: pending the next revision (entry 28 of `BACKLOG.md`).
The sentence is either withdrawn or measured again on that revision's
tree, and is not copied from here. The condition for claiming a
direction again is D-AJ's: that it measures stable. It does not reach
the reader of the deposited PDF («What it is NOT», above). §512 gave it
as pending and called it «Entrada 3» [Entry 3], which was already another
one; it is this one.

---

*This file grows by addition. Entries are neither edited nor deleted: if
an erratum is corrected in a later revision, it is noted below with its
date and its version.*
