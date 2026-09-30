# Evidence-Carrying State Transitions (ECST)

*A composition discipline and two instantiations: HBS-STATE and Arqueo*

**Technical report — DRAFT v0.1, not deposited**

- **Author:** Ángel José Toranzo Portela
- **Date:** 2026-09-30
- **Basis:** Arqueo, `main` at 71c5aad (S582); `hbs-state` at a960828 (crate 0.2.0, which implements the
  HBS-STATE v0.3 specification revised on 2026-09-27).
- **Verification register:** `VERIFICACION.md`, in this same directory: each claim of the earlier drafts
  with its verdict, its source and its correct wording (§1.5).

> **Español:** `ECST.md` es este informe en español; en caso de discrepancia manda el español.

> **How to read the marks.** Every figure, every execution result and every state that a repository
> declares about itself carries one of these marks:
>
> - **[MEASURED]** — the author measured it and it is recorded. The source is given (`AUDITORIA.md` §N, or
>   the README or the specification of `hbs-state`) and, where recorded, the machine and the spread.
> - **[REPRODUCED]** — re-run on 2026-09-28 or 2026-09-30 in a Firecracker microVM, not on the author's
>   machine (`doc/ecst/borrador/REPRODUCCION.md`). It is one run in one environment: it does not replace
>   what was measured.
> - **[DERIVED]** — computed from the code or from measured figures; nobody has measured it.
> - **[DECLARED]** — what a repository declares about itself (a threshold, a policy, a state).
> - **[PROPOSAL]** — part of the model that is not implemented.
> - **[NOT MEASURED]** — nobody has measured it; no figure is given.
> - And two more, for precision: **[READING]** — inferred by reading the code, without running it;
>   **[CITED]** — a figure from an external source, measured in neither of the two repositories.
>
> Unmarked descriptions of the code are **[READING]**: they point to a `file:line` path or to an entry of
> the register, the verifier agents checked them against the tree, and nobody has exercised them unless
> stated. «VERIFICACION.md, L3a» refers to the entry for claim L3a in the register. The `file:line` paths
> are of the tree at 71c5aad.
>
> The register's verdict labels are kept in Spanish, as they name its entries: CONFIRMADA (confirmed),
> PARCIAL (partial), FALSA (false), SIN_FUENTE (no source), NO_VERIFICABLE (not verifiable).

---

## Abstract

ECST (*Evidence-Carrying State Transitions*) is a discipline for composing known primitives —per-transition
proofs, hash chaining, signed heads and reconciliation of physical state— so that each transition of a
stateful system leaves a package that can be checked piece by piece: the previous and the new committed
state, the operation, the cryptographic evidence, the authorization and an accumulated digest of the
history. It is not a new primitive, nor incrementally verifiable computation (IVC), nor consensus, nor a
guarantee of physical durability. Its thesis is one of separation: the integrity of the history,
cryptographic validity, authorization, semantic conformance and the uniqueness of the history (consensus)
are distinct properties and, in general, none implies the others. The report states retroactive binding
—the classical property of hash time-stamping— with its exact hypotheses, separated from what a third party
needs in order to benefit from it; a function R(C,K) that classifies into four states the counter–index
pair of a monotonically consumed resource, with a rule for indices of undetermined use; and a tiered audit
protocol, proposed and unimplemented, whose result is a vector with one component per predicate. Two
instantiations by the same author are analysed: HBS-STATE v0.3, a specification with a reference
implementation for deciding after a restart whether the index of a stateful hash-based signature is safe,
which instantiates only reconciliation; and Arqueo, a single-operator closed ledger with one STARK proof per
transition, which instantiates most of the components with declared gaps —it does not keep the proofs, no
third party recomputes the chain, there is no audit tool— and uses the same R(C,K) for two resources. Each
figure carries its mark and its source, and the unmeasured figures of the earlier drafts are withdrawn. A
reading finding, not reproduced, is also reported: with the signing counter file deleted or reset to zero,
the node would sign already-used XMSS leaves again; it is fixed, with tests and the canon green, in §594 of
`AUDITORIA.md` (commit 78d71a4 of the working branch) (§8.1).

---

## 1. Introduction

### 1.1 The problem

**The index of a stateful signature.** In XMSS and LMS [RFC8391; RFC8554], built on Merkle trees
[Merkle89], each signature consumes the index of a one-time key; signing two messages with the same index
allows forgery at a feasible cost [BH17; Fluhrer23], and the key must be considered lost (VERIFICACION.md,
B17d). The state on which security depends lives outside the cryptography: a counter in a file, a process
that dies between reserving and signing, a disk that acknowledges an `fsync` without persisting, a backup
that gets restored. NIST SP 800-208 requires key and signature generation to be validated only within
hardware cryptographic modules [SP800-208]. RFC 10033, informational and published in September 2026 by
the PQUIP group, recommends in its §4 that state management meet the ACID properties («should») and
dedicated hardware to achieve rollback-resistant counters, and analyses in its §5 nine approaches, with no
test vectors [RFC10033].

**The closed ledger audited by third parties.** A single operator keeps a ledger, and the holders, an
auditor or a supervisor want to check what happened without trusting it and, if possible, with it switched
off. The useful question is not whether the ledger is «verifiable», but what can be checked by whom, with
what data and under what hypotheses.

The common shape: transitions over a state, resources that are consumed only once, evidence that has to
survive the process that produced it, and the need for «the process died» to be an explicit, classifiable
state.

### 1.2 What ECST is and what it is not

ECST is a **composition discipline**. It asks that each transition produce a package with six fields, that
four predicates —semantic rules, cryptographic validity, authorization and historical link— be checked and
reported separately, that reconciliation of physical state after a crash be a component with named states,
and that an audit return a vector and not a boolean.

ECST **is not** a new cryptographic primitive: all its pieces are in the literature (§2). **It is not** IVC
or PCD [Valiant08; ChiesaTromer10]: it does not prove inside the proof Π_i that Π_{i−1} was valid; rather,
the accumulated digest H_i commits the digest of each proof (VERIFICACION.md, P17). **It is not** consensus:
it says nothing about the uniqueness of the history if nobody compares what they have seen. **It is not** a
guarantee of physical durability: the level it reaches is that of `fsync`, and resistance to a power cut is
not measured in either of the two repositories.

### 1.3 Contributions

None is a new piece:

1. **An explicit formulation** of the transition package E_i, of four separate predicates and of a
   retroactive-binding proposition whose hypotheses —injective, fixed-version encoding, collision resistance
   of two functions with fixed-width output, binding of the actual bytes of the evidence (R4)— are
   distinguished from the conditions for use by a third party —an authenticated head held in custody
   before the alteration, known genesis and length, available data, the version of the encoding—. For two
   of those hypotheses, counterexamples from Arqueo's history are given: the collision resistance of
   $\mathrm{Hp}$ as a function of the proof bytes, which failed through its internal padding even though its
   permutation was not broken, and R4.
2. **Reconciliation of physical state, R(C,K), as a component of the model**, with a rule for indices of
   undetermined use —treat them as lost, like the interval reservation of [McGrew16] (RFC 10033, §5.9),
   without losing the liveness that [Ariadne16] requires—, and the observation, which is this work's own,
   that the meaning of each state depends on the key model.
3. **An analysis of two instantiations** with code paths: which parts of the model each one covers, what
   is measured, what was reproduced in this revision and what is missing. It includes the second instance
   of R(C,K) within Arqueo, the reception log: a second use of the same classification, by the same
   author, which indicates that it generalises as a classification but is not independent evidence.
4. **A public verification register** of the AI-generated drafts that preceded this report
   (`VERIFICACION.md`), with the figures that are withdrawn (§7.5), and a reading finding about Arqueo's
   index guardian (§8.1).

### 1.4 What this report does not claim

- That ECST is cheaper than IVC: there is no measured comparison at all (§7.4).
- A formal proof: Proposition 1 carries a proof sketch, not a mechanised proof.
- That HBS-STATE is a standard: it is a draft derived from a single implementation (§4.10).
- That Arqueo is in production, handles real money or has been audited by third parties: none of the three
  is the case (§5.1).
- That a third party can today audit the complete history of an Arqueo ledger: it cannot (§5.7).
- Durability against a power cut, in either of the two systems.

### 1.5 Provenance

This report starts from AI-generated drafts —about ECST, HBS-STATE, an Internet-Draft and an email to the
CFRG— that mixed reasonable ideas with invented figures, code facts and citations. Their 305 claims were
verified one by one: 104 CONFIRMADA, 105 PARCIAL, 68 FALSA, 23 SIN_FUENTE and 5 NO_VERIFICABLE
(`VERIFICACION.md`, «El recuento» [The count]). Only what held up is collected here, in its corrected
wording.

---

## 2. Related work

**Time-stamping and tamper-evident logs.** Chaining each record to the digest of the previous one so that
altering the past changes everything after it is the idea of Haber and Stornetta [HS91]. Schneier and
Kelsey applied it to audit logs on untrusted machines [SK99], and Crosby and Wallach gave history-tree
structures with efficient membership and extension proofs [CW09]. Dowling et al. defined four security
properties of logging schemes such as CT and proved that CT meets them [DGHS16]: it is the formal framework
closest to Proposition 1. ECST's retroactive binding (§3.5) is exactly the property of [HS91] and is not
presented as a contribution.

**Transparency and accountability.** Certificate Transparency publishes an append-only log with inclusion
and consistency proofs [RFC6962; RFC9162; Laurie14]: it does not prevent misbehaviour, it makes it hard to
hide if someone compares. Detecting a split view by comparing what each client has seen is the *fork
consistency* introduced by Mazières and Shasha [MS02], which SUNDR applied to an untrusted data repository
[SUNDR04]; the witness of §5.6 is an instance of that pattern with a single observer. PeerReview provides
accountability in distributed systems with tamper-evident logs and witnesses [PeerReview07], and CONIKS
brings transparency to user keys with monitoring by the clients themselves [CONIKS15]. Arqueo adopts the
CT pattern and the history-tree algorithms of RFC 6962 over its own primitives; it does not implement the
CT protocol (VERIFICACION.md, L10a).

**Verifiable ledgers and state machines.** LedgerDB is a centralized ledger database with tamper evidence
and third-party audit [LedgerDB20]; zkLedger allows a ledger to be audited without revealing the contents
of the transactions to the auditor [zkLedger18]; in replicated state machines without replicated execution
[Piperine20] and in verifiable state machines [VSM20], an untrusted prover accompanies each state change
with a succinct proof that the transition is correct; and Al-Bassam et al. combine fraud proofs and
data-availability proofs so that a light client detects invalid blocks [AlBassam21]. ECST shares that
skeleton; what it adds is the table of what each predicate does not detect and the reconciliation of
physical state.

**Proof-carrying code, IVC and PCD.** The term «evidence-carrying» points to Necula's proof-carrying code
[Necula97], where the consumer checks a proof that accompanies the code. Incrementally verifiable
computation [Valiant08], proof-carrying data [ChiesaTromer10], its recursive composition [BCCT13] and
folding schemes [Nova22; HyperNova24] solve a stronger problem than ECST's: that a single proof attest the
validity of the whole chain. There is also accumulation without public-key assumptions: hash-based, in the
random oracle model and, in its first version, with a bounded number of steps [BMNW25]. ECST gives all that
up in exchange for not putting a verifier inside the circuit; the price is that auditing n transitions
requires n verifications and the availability of n proofs (§3.10). Arqueo's proofs are STARKs [BBHR19]
(full version in [STARK18]), in the winterfell implementation.

**State continuity and rollback protection.** Memoir [Memoir11], Ariadne [Ariadne16] and ROTE [ROTE17]
study how a protected module avoids being restored to an earlier state, which is the difficulty that
RFC 10033 attributes to rollback-resistant counters implemented only in software. Ariadne also requires
liveness: an unexpected loss of power must never leave the system in a state from which it cannot resume,
which is the problem of §3.8 for a single-use resource. Nimble offers rollback detection as a cloud
service to applications in trusted execution environments [Nimble23]. ECST does not solve rollback: the
reconciliation of §3.7 classifies what it sees, and what it does not see —a joint restore of counter and
key, for example— stays outside and is declared.

**State management in hash-based signatures.** McGrew et al. analyse the failure modes of state and
propose reserving ranges of indices in advance [McGrew16]; `hbs-state` persists before each signature,
which is the degenerate case of that strategy (interval 1): it pays one synchronised write per signature
and in exchange loses at most one index per crash. ETSI TR 103 692 analyses the challenges and risks of
state management [ETSI21]. RFC 10033 collects nine approaches and recommends dedicated hardware [RFC10033],
and observes in its §4 that a verifier with access to all the signatures detects a repeated index, which
is what a witness can do (§5.6); SP 800-208 requires hardware modules for validation [SP800-208]. The
stateless alternative, SLH-DSA [FIPS205], eliminates the problem, and RFC 10033 (§1.1) recommends it for
most applications. EIP-8310, an Ethereum draft, requires a high watermark persisted before the signature
is handed over, and detection of its regression after restoring a backup [EIP8310]. The cost of reusing an
index was analysed by Groot Bruinderink and Hülsing [BH17]; Fluhrer corrected their Winternitz analysis,
which assumed to be independent probabilities that are not, and kept the conclusion that reuse allows
forgery [Fluhrer23].

**Crash consistency.** Pillai et al. showed how hard it is to write crash-correct update protocols on real
file systems, with a tool, ALICE, that looks for crash vulnerabilities in applications' update protocols
[Pillai14]; and Chidambaram et al., how to separate ordering and durability [Chidambaram13]. POSIX allows
a null implementation of `fsync()` when `_POSIX_SYNCHRONIZED_IO` is not defined [POSIX24], although
Linux's glibc does define it; the practical risk is a different one, that of devices that do not keep
what was acknowledged across a power cut, which Zheng et al. studied in SSDs [Zheng13]. All of this is
consistent with the self-check of §4.8 measuring the cost of `fsync` instead of trusting its return
value; the reason the repository itself gives is the tmpfs measurement of `AUDITORIA.md` §234.

**Position.** What might be new here is not the pieces but two things: the **explicit separation** of the
predicates in a single model, with what each one does not detect (§3.6) —the light-client literature
already separates the validity of a block from the availability of its data [AlBassam21]—, and
**reconciliation as a classified component** of the model (§3.7-§3.8). The **executable semantic
contract** of HBS-STATE —four states, a judge, vectors and a subject protocol— (§4) is prior work of the
author, analysed here together with the **two instantiations**, their code paths and their declared gaps.

---

## 3. The ECST model

### 3.1 Objects

A stateful system advances through transitions i = 1, 2, … For each one, ECST requires a package

```text
E_i = ( C_{i-1}, C_i, O_i, Π_i, A_i, H_i )
```

- $C_{i-1}$, $C_i$: the **commitment** to the previous and to the new state, $C_i = \mathsf{Com}(S_i)$ with
  $\mathsf{Com}$ binding. The drafts wrote $S_{i-1}$ and $S_i$; what travels and is chained is a
  commitment, not the state.
- $O_i$: the **operation**, with its class and its public parameters.
- $\Pi_i$: the **cryptographic evidence**: a proof, a seal or, explicitly, nothing; absence is encoded
  with a constant of its own domain, distinct from $\mathrm{Hp}$ of the empty string (R3).
- $A_i$: the **authorization**. It may go inside $\Pi_i$ —the authorship of a payment in Arqueo—, in a
  separate object —the pair of proofs from two custodians— or not exist.
- $H_i$: the **accumulated digest of the history**, with

```text
H_i = Hc( dom_c ‖ H_{i-1} ‖ Enc_v(i, O_i, C_{i-1}, C_i, a_i) ‖ Hp(Π_i) ),     H_0 = g
```

where $a_i$ is the part of $A_i$ that is chained (it may be empty), $\mathrm{Hp}$ is the digest of the
evidence and $g$ a constant, public genesis; $\mathsf{dom}_c$, $H_{i-1}$ and $\mathrm{Hp}(\Pi_i)$ have
fixed width. This formula is the **abstract model**; it is not Arqueo's, which is given in §5.4
(VERIFICACION.md, L1b). Neither $E_i$ nor $H_i$ commits $vk_i$ or the rules in force: unless a digest of
them is chained —for example, inside $O_i$—, the rules of each epoch enter level B of §3.10 as
unauthenticated data, which is what allows the verifier to be changed without a trace (§6.1). Arqueo does
not chain it; its `paramsDigest`, in the head since v5, commits system parameters, not the AIR.

### 3.2 Encoding requirements

- **R1 · Injectivity and canonicity.** Distinct tuples give distinct encodings, and each tuple has a single
  accepted encoding: fixed-width fields, or fields with length and domain.
- **R2 · Version.** The encoding carries its version, or a mark that discriminates it, and a verifier
  rejects a version it does not know.
- **R3 · Domain separation.** $\mathsf{dom}_c$ does not coincide with any other use of $\mathrm{Hc}$;
  $\mathrm{Hp}$ carries its own domain and the length of the input.
- **R4 · Actual bytes.** $\mathrm{Hp}$ is computed over the bytes of $\Pi_i$ as they were verified,
  not over a constant or over a reserialisation.

Arqueo has violated R4, and has had an $\mathrm{Hp}$ that was not collision-resistant without any
permutation being broken: the internal padding of `digest_of_proof` appended zeros without encoding the
length (`AUDITORIA.md` §116, the length clause of R3) and reduced 8-byte blocks modulo $p$ (§124), so that
two distinct proofs gave the same digest. It was not a failure of $\mathrm{Enc}_v$, but of $\mathrm{Hp}$ as
a function of the proof bytes; that is why Proposition 1 requires the collision resistance of that
function, with its internal encoding, and not only that of its permutation. R4 was violated in six paths
that chained the constant digest of the empty proof until §278 (VERIFICACION.md, P9b); consumption still
chains `digest_of_proof(&[])`, with the digest of the consumption in the commitment (§5.4). R3 is met only
structurally (§5.4).

### 3.3 Four predicates

- $\mathsf{StateOK}(S_{i-1}, O_i, S_i)$: the transition respects the semantic rules of the system. It is
  evaluated over openings of $(C_{i-1}, C_i)$; without them —a third party in Arqueo— only what
  $\Pi_i$ proves is checkable.
- $\mathsf{Verify}(vk_i, x_i, \Pi_i) = 1$: the evidence is cryptographically valid for the public inputs
  $x_i$, derived from $(C_{i-1}, C_i, O_i)$, with the verification key $vk_i$ in force at $i$.
- $\mathsf{Authorize}(A_i, O_i)$: whoever had to authorize $O_i$ did so.
- $\mathsf{Link}(H_{i-1}, H_i, E_i)$: $H_i$ is the recomputation of the formula of §3.1 over the fields of
  $E_i$.

Over a segment, $\mathsf{HistoryOK}_n$ —$\mathsf{Link}$ at each step and $H_n$ equal to an authenticated
head $\hat H_n$— and $\mathsf{CryptoValid}_n$ —$\mathsf{Verify}$ at each step— are defined.

### 3.4 The separation thesis

> **Commitment ≠ Validity ≠ Semantic conformance ≠ Consensus.**

The lemma is one of non-implication **in general**. In a concrete system one property may imply another by
construction —in Arqueo's send, $\mathsf{Verify}$ implies $\mathsf{Authorize}$ because authorship goes
inside $\Pi_i$—, but that is a claim about that system that has to be proved.

- **HistoryOK ⇏ CryptoValid.** The chain links digests; it does not say that the digested proofs verified
  (VERIFICACION.md, P10a). Real case: until 31-07-2026 Arqueo's layer accepted sends and claims without
  verifying their proof, and nothing in the chain would have given it away (`AUDITORIA.md` §73).
- **CryptoValid ⇏ StateOK.** Soundness is relative to the relation that the circuit constrains, and that
  this relation captures all the rules is another claim. Real case: in §487 it was measured that a frozen
  account produced proofs that verified in five circuits; it was closed first by the layer, which rejects
  with `AccountFrozen`, and §511 added the position binding to the AIR of those circuits (`AUDITORIA.md`
  §487, §511).
- **None implies Consensus.** An operator can show two coherent histories to two observers; only whoever
  compares signed heads of the same index detects it. Nor do they imply **completeness**: an operation that
  never entered the chain alters no $H_i$.

### 3.5 Proposition 1: retroactive binding

**Proposition 1.** Let $\mathrm{Hc}$ and $\mathrm{Hp}$ be collision-resistant and with fixed-width output
—$\mathrm{Hp}$ as a function of the proof bytes, with its internal encoding—; $\mathrm{Enc}_v$ injective
(R1) for a fixed and known version $v$ (R2); and $\mathrm{Hp}$ computed over the bytes of $\Pi_i$ as they
were verified (R4). Let there be two histories $(E_1,\dots,E_n)$ and $(E'_1,\dots,E'_n)$ with the same
genesis $H_0$ and the same length $n$, which satisfy $\mathsf{Link}$ at each step and which differ in some
chained field —$O_k$, $C_{k-1}$, $C_k$, $a_k$ or $\Pi_k$— of some $E_k$ with $k \le n$. Then, if
$H_n = H'_n$, from the two histories one exhibits, with at most $2n$ evaluations of $\mathrm{Hc}$ and $2n$
of $\mathrm{Hp}$, an explicit collision of one of the two. Since $\mathrm{Hc}$ and $\mathrm{Hp}$ carry no
key, it reads as a constructive reduction [Rogaway06]: whoever produces such pairs produces a collision,
with at least the same success and an additional cost linear in $n$. In the abstract model R4 holds by
definition; it is named because it is the condition for an implementation to instantiate the formula.

*Sketch.* The two chains are recomputed. Let $u_i$ be the complete input of $\mathrm{Hc}$ at step $i$. One
walks backwards from $i = n$, knowing that $H_i = H'_i$. If $u_i \neq u'_i$, the pair is a collision of
$\mathrm{Hc}$. If $u_i = u'_i$, then, since $\mathsf{dom}_c$, $H_{i-1}$ and $\mathrm{Hp}(\Pi_i)$ have
fixed width, $u_i$ decomposes in only one way: $H_{i-1} = H'_{i-1}$, the encoded fields coincide (by R1)
and $\mathrm{Hp}(\Pi_i) = \mathrm{Hp}(\Pi'_i)$; if moreover $\Pi_i \neq \Pi'_i$, it is a collision of
$\mathrm{Hp}$. Since the histories differ in a chained field of $E_k$, the walk cannot get below $k$
without finding a collision. ∎

**What the proposition does not say.** (i) Nothing about the fields that are **not** chained: if an
operation class chains a seal instead of its proof, altering that proof does not change $H_n$ (§5.4).
(ii) The restriction to the same genesis and the same length is for convenience: since $i$ enters
$\mathrm{Enc}_v$ and $H_0$ enters $u_1$, the same walk gives a collision if $H_n = H'_m$ with $n \neq m$ or
with $g \neq g'$. What it does not give is that the third party knows which genesis and which $n$ the head
corresponds to: that is D2. (iii) Nothing about the validity of what is chained: it is
$\mathsf{HistoryOK}$, not $\mathsf{CryptoValid}$. (iv) It is the property of Haber and Stornetta [HS91],
the basis of [CW09] and of CT [RFC6962]; ECST uses it, it does not contribute it (VERIFICACION.md, P9c).

**Conditions for a third party to benefit from it.** They belong to the deployment, not to the theorem,
and are separated on purpose (VERIFICACION.md, P9b):

- **D1 · Authenticated head, prior to the alteration.** $\hat H_n$ has to reach them signed, with the key
  anchored before the first encounter, and held in custody by them or by a witness **before** the
  alteration to be detected: against the signer, who can sign again, a head signed after rewriting the
  history authenticates the rewritten history. In Arqueo the anchor is TOFU and there is no real operator
  key (SECURITY.md, «NO HAY CLAVE QUE ANCLAR» [THERE IS NO KEY TO ANCHOR]; §5.6).
- **D2 · Known genesis and length**: without them, a correct head of a prefix passes for that of the whole
  history.
- **D3 · Availability.** The entries and the bytes of each $\Pi_i$, to recompute $\mathrm{Hp}$. Arqueo
  does not keep the proofs (§5.4).
- **D4 · The version of the encoding**, and in Arqueo the era of each entry, to know which fields enter
  and how.

**In Arqueo** (§5.4), $\mathrm{Hc}$ is `native_merge` (Rescue-Prime Rp64_256 over Goldilocks) and
$\mathrm{Hp}$, Blake3-256 with domain and length, reduced to the field; that reduction is not injective
—values greater than or equal to $p$ collide with small values—, a marginal loss **[DERIVED]**: each of
the four limbs falls in $[p, 2^{64})$ with probability $\approx 2^{-32}$, and a collision due to the
reduction additionally requires the other three limbs to coincide. The collision resistance of both
functions is an assumption. The proposition carries over to the nested composition of §5.4 merge by merge,
because each input of `native_merge` has fixed width; but, without a domain tag, two histories that differ
in the era of an entry only give a collision if it is also hard to find an output of `native_merge` with a
prescribed form —`[x,0,0,0]` or the constant `COMPROMISO_AUSENTE`—, a preimage-type property that
collision resistance does not imply; that is why D4 includes the era. The module `t1_chain_retroactivo`
(`crates/zk-ssl/src/log.rs:1189-1295`) exercises the proposition in **one** case —12 entries, alteration
at the 5th, synthetic 8-byte proofs, no STARK—: recomputation from the raw fields reproduces the head, the
divergence is located at k = 5 —with the two histories at hand— and contaminates every j ≥ 5, and
`verify_chain` detects the substitution
**[REPRODUCED: `3 passed`, 2026-09-28 and 2026-09-30]**. It checks the chaining; it does not measure
collision resistance (`AUDITORIA.md` §116; VERIFICACION.md, M14a).

### 3.6 What each layer detects and what it does not

| layer | detects | does not detect |
|---|---|---|
| **Link / HistoryOK** | with an authenticated head **held in custody before** the alteration and known $n$, the alteration, insertion, deletion or reordering of a chained field (Proposition 1 and its note (ii)) | that a transition is invalid; what never entered the chain; a split view if nobody compares; the unchained fields |
| **Verify / CryptoValid** | a transition that does not satisfy the circuit's relation, if the proof system is sound | that the relation captures the rules; that the proof is in the history; that the rules in force are the published ones, if the operator can change the verifier without a trace |
| **Authorize** | an operation without the authorization its class requires | that the authorization corresponds to an independent will: two compromised keys authorize just the same |
| **StateOK** | the semantic rule that is checked outside the circuit | a rule that nobody encoded |
| **Reconciliation R(C,K)** | after a restart, the violation of the invariant visible in the pair (KeyAhead); classifies the rest into three non-fatal states, with their lost or undetermined range | the joint restore of counter and key; a counter deleted or reset to zero when the key starts at zero (§8.1); durability across a power cut |
| **Signed head and witnesses** | a split view among those who compare; a key change after the first encounter | the first encounter; witnesses that collude or that do not exist |
| **Completeness** (RFC-0010 in Arqueo) | that an operation **with a receipt** was not resolved in its window | the operation that was never given a receipt |

### 3.7 Semantic reconciliation of physical state

Let there be a resource of **monotonic consumption** —a one-time signature index, a reception number—
with two durable data: $C$, a counter that is reserved and persisted **before** using the resource, and
$K$, what the resource itself says it has reached —the index within the key, the largest number recorded
in a log—. The invariant of a correct execution is

> **Every use has a leaf index, counted from 0, lower than the persisted counter; equivalently,
> $K \le C$.**

With the declared index of §5.5, which is the leaf plus one, the same rule is written «no index greater
than the counter», which is the wording of HBS-STATE and of `AUDITORIA.md` §234 (§4.2); EIP-8310 fixes the
convention with «signing is permitted only strictly above» the watermark [EIP8310]. Reconciliation is a
pure function $R : \mathbb{N} \times \mathbb{N} \to$ {InSync, KeyAtZero, CounterAhead, KeyAhead}, with one
derived field per state (§4.4). Its role in ECST is to turn «the process died» into an explicit,
classifiable state, with a **judge** that marks a single state as evidence of a violation of the invariant
(KeyAhead) and a **policy** that belongs to the owner of the resource.

Two caveats. **The meaning of each state depends on the key model** (§4.3): with a seed-derived key, every
real restart gives KeyAtZero; with a persisted key, CounterAhead is the normal case after a crash. And
**R sees only the pair**: a rollback that leaves it coherent —restoring counter and log together, or
deleting the counter or resetting it to zero when the key starts at zero— is invisible to R and requires a
datum outside the pair.

### 3.8 The rule for undetermined indices

When $R$ returns a state with a range whose use cannot be decided with the durable data —$[0, C)$ in
KeyAtZero, where with counter 1 and key 0 dying before signing and dying after leave the same state on disk
(HBS-STATE v0.3, §2); $[K, C)$ in CounterAhead with a persisted key, where a signature may have gone out
before the key was persisted—, the rule is:

1. never reuse an index lower than $C$;
2. continue from $C$, the first index that was never reserved, and treat the undetermined ones as lost:
   «un índice perdido es mejor que uno indeterminado» [a lost index is better than an undetermined one]
   (note 92, an entry of `BACKLOG.md`, as quoted by `crates/zk-ssl-guardian/src/lib.rs:146-147`);
3. stop only if an **independent** durable record contradicts $C$, or if $R$ returns the fatal state.

Treating as lost what was reserved and not confirmed is the semantics of the interval reservation of
McGrew et al. [McGrew16], which RFC 10033 (§5.9) summarises thus: the reserved indices are lost if the
device loses power or restarts. And not staying stuck forever is the liveness requirement of [Ariadne16].
It is not the rule «KeyAtZero ⇒ do not sign», which would make a seed-derived signer never sign again after
its first restart (VERIFICACION.md, P15a, G1f), nor is it normative text of HBS-STATE, which leaves the
policy to the owner and requires the orphans to be burnt or recorded (§4.5). ECST adopts as a general rule
the policy that Arqueo already applies to its signing index in ClaveEnCero since §335 and §337 (§5.13); its
application to CounterAhead and, given what §8.1 shows, the consultation of the independent record of
point 3 **in all states** and not only in KeyAtZero are **[PROPOSAL]**.

### 3.9 Single use: by counter and by set

The abstract property: each element of a set of resources is consumed **at most once** and, if the
consumption has an external effect, it is recorded before producing it: it is the usual answer to the
*output commit* problem of rollback recovery, that no output to the outside world should have to be
revoked [Elnozahy02]. It appears with two mechanisms. By monotonic counter —the leaf of an XMSS tree,
reserved by the guardian before signing, and the reception number `rx` of RFC-0010—, to which R(C,K)
applies. And by set —the consumption label of RFC-0006, unique within a ledger, and the leaf of a pending
item, which the claim circuit leaves empty—, which has no pair to reconcile. With a single writer, the
property is **enforced**; between independent writers, at best it is **detected** afterwards (§5.9).

### 3.10 Tiered audit protocol

**[PROPOSAL]** as a tool; some pieces exist in Arqueo (§5.14).

- **Input:** an authenticated head $\hat H_n$ (signature and anchor), held in custody before the alteration
  to be detected (D1); the entries of a segment; and the rules —the $vk_i$ and the version of the
  encoding— in force at each step.
- **Level A · history:** recompute $H_i$ from the genesis or from a head held in custody. Valid if
  $H_n = \hat H_n$; Invalid if not; Undetermined if entries or the specification of the encoding are
  missing. With a single head, Invalid applies to the whole segment: the divergent index is only bounded if
  there are intermediate authenticated values $\hat H_j$ —signed heads of earlier epochs—, and only to the
  stretch between the last one that matches and the first one that does not.
- **Level B · evidence:** for each $i$ with level A Valid, check that $\mathrm{Hp}(\Pi_i)$ matches the
  digest chained in entry $i$, and then $\mathsf{Verify}$ over $\Pi_i$ with the $x_i$ derived from that
  entry. Undetermined if $\Pi_i$ is not available or the rules of its epoch are not known.
- **Level C · authorization and semantics:** $\mathsf{Authorize}$ and $\mathsf{StateOK}$ where they can be
  checked with public data; Undetermined where not.

The result is a **vector** $(r_{\mathsf{Link}}, r_{\mathsf{Verify}}, r_{\mathsf{Auth}}, r_{\mathsf{State}})$
per transition, with one component per predicate and each one in {Valid, Invalid, Undetermined, Not
applicable}; Not applicable is for the classes that declare $\Pi_i$ empty. Over a segment, each component
is aggregated with Invalid above Undetermined and the latter above Valid, and the coverage is reported. It
is never collapsed to a boolean, and an Undetermined is not read as Valid. Without recursion, the cost is
linear in n; with IVC, the verifier does not depend on n, in exchange for verifying inside each step
(VERIFICACION.md, P11b). Which one is preferable for a concrete workload **is not measured** (§7.4).

---

## 4. Instantiation 1: HBS-STATE

### 4.1 What it is and where it comes from

HBS-STATE v0.3 specifies **one** property: how to decide, after a restart, whether the state of a stateful
hash-based signature is still safe to use, with four states, a judge and numeric vectors. It presupposes
[SP800-208] and [RFC8391] and is not an implementation of XMSS (HBS-STATE v0.3, spec/ of `hbs-state`).
Its reference implementation, the crate `hbs-state` 0.2.0, with no dependencies and under MIT OR
Apache-2.0 (the vectors, CC0-1.0), is an almost literal translation of `zk-ssl-guardian`, Arqueo's index
guardian, with the reconciliation extracted as a pure function, a subject binary and `tests/vectors.rs`
(VERIFICACION.md, H1a, H13c, G1a).

### 4.2 The invariant

> **No signature may exist with an index greater than the persisted counter.**

`IndexGuard::reserve` persists `current + 1` —eight bytes *little-endian* in place and `sync_all`— before
returning it (`hbs-state/src/lib.rs:482-489`, `:506-519`). There is no `fsync` of the directory, no
temporary file with rename, no lock, and `open` creates the counter at 0 if it does not exist, without
warning: a deleted counter is indistinguishable from a new one (VERIFICACION.md, H2a, H2b). Against the
death of the process **[MEASURED: `AUDITORIA.md` §234, bench K.1, a single machine, on Arqueo's original
guardian when it still had no consumer —before the head signer (§236) and before `hbs-state` existed— and
without `xmss` as a dependency]**, a child that persists and signs, killed with `kill -9` at a random
instant, left no signature ahead of the counter in 25 out of 25 kills; what the child «signed» is not
recorded. It applies to `IndexGuard` because it is an almost literal translation of that guardian
(VERIFICACION.md, H18a, G7a, G1a). The bench code is not in the trees; with 0 failures in 25, the 95%
upper bound is ≈ 11% **[DERIVED: 1 − 0.05^(1/25)]**, a bound on the rate per random kill, not on the
correctness of the ordering, because the fraction of the cycle in which an inverted ordering would fail is
not known. And it measures that the write of the counter precedes the signature as the page cache sees
it —a `kill -9` does not distinguish a write with `sync_all` from one without it—, not persistence or a
power cut. What would bound the ordering is injecting the crash at each system call, as ALICE does
[Pillai14].

### 4.3 The two key models

Before classifying one has to answer «the zero question»: after a restart, where does the index carried by
the private key come from? (HBS-STATE v0.3, §1)

| | **persisted key** | **seed-derived key** |
|---|---|---|
| what is on disk | the key material with its index | only the seed |
| on restart | the index survives | `from_seed` returns it at index 0 |
| states after a restart | InSync, CounterAhead, KeyAhead, and KeyAtZero if the subject deletes the key when it is exhausted | **always KeyAtZero** (or InSync if the counter is 0) |
| CounterAhead | the normal case after a crash | only within a live process |

The counterexample that corrected that table is `hbs-lms` 0.2.0-alpha.1 (Fraunhofer AISEC, commit
7063cc8): with a persisted key, it deletes the key when the tree is exhausted and reaches KeyAtZero, which
does not distinguish a deleted, terminal key from a restart; the discriminant is the parameter set, which
the index does not carry (HBS-STATE v0.3, §0-§1; VERIFICACION.md, H4). The «13 of 25 kills leave the
counter ahead» **[MEASURED: §234]** is about kills within a process, and the inference «the normal case
after a crash is CounterAhead» is retracted in the specification itself as a general rule and for the
seed-derived key; with a persisted key it still holds, as the table says (VERIFICACION.md, H18b, G7b).

### 4.4 The four states

Let `counter` be what is reserved and persisted and `key` the index carried by the key.

| state | condition | derived field | fatal |
|---|---|---|---|
| InSync | `key == counter` | `index = counter` | no |
| KeyAtZero | `counter > key` and `key == 0` | `indeterminate = counter` | no |
| CounterAhead | `counter > key` and `key != 0` | `orphans = counter − key` | no |
| KeyAhead | `key > counter` | `unrecorded = key − counter` | **yes** |

The four conditions form a partition of $\mathbb{N}^2$, so no precedence among them is needed; the code
realises it with a `match` on `counter.cmp(&key)` —the three values of `Ordering`— with a single guard,
`Greater if key == 0`, as an arm before the general `Greater`, and the specification calls that guard
«precedence»: (0,0) is InSync, (0,1) is KeyAhead, `indeterminate` is a single term and not a subtraction,
and there is no exhaustion variant (`hbs-state/src/lib.rs:412-431`; HBS-STATE v0.3, §2 and §8;
VERIFICACION.md, H3).

### 4.5 The judge and the policy

The judge is `fatal(state) = (state is KeyAhead)`; `is_fatal` is a pure predicate and stopping is the
caller's duty. Arqueo's production code does not call its equivalent, `no_admite_matiz`: each policy does
its own `match`; in the witness, a test requires that its policy not start where `no_admite_matiz` marks
fatal, and the node's signing policy has no such binding: its tests enumerate the cases by hand
(`crates/zk-ssl-cli/src/witness.rs:3050-3075`; VERIFICACION.md, H6a). **The policy belongs to the owner**:
the specification only requires that the counter not go back and that the orphans be burnt or recorded
(§2, §9); it does not require intervention on KeyAtZero, nor does it classify it as a violation in any model
(VERIFICACION.md, H6b, H6c). Its «KeyAtZero cannot be resolved, only failed closed» means «the undetermined
indices are not reused», not «do not operate»: N3 requires refusing to operate **only** on KeyAhead
(VERIFICACION.md, G1f).

### 4.6 Conformance: N0 to N3 and the subject protocol

The levels are cumulative (HBS-STATE v0.3, §4):

- **N0 · declares** its model, `persisted` or `seed_derived`. Since v0.3 the verifier requires it: whoever
  does not declare it is left without a level even if it classifies correctly.
- **N1 · classifies** the four states. Merging CounterAhead and KeyAhead into a «state error» does not meet
  N1.
- **N2 · derives** `index`, `orphans`, `indeterminate` and `unrecorded`.
- **N3 · judges**: marks exactly KeyAhead as fatal.

A **subject** is an executable that responds to `--model`, `<counter> <key>` and `--sk <hex>`; the verifier
reads only the last line of the standard output, which must be JSON, and every failure —an exit code other
than zero, empty standard output or a last line that is not JSON— counts as a failed row, with 60 s per
invocation at most. It checks the boolean `fatal` without exercising a real refusal, and the reference
subject declares `seed_derived` without touching disk or keys: its N3 certifies the pure function, not the
durability of `IndexGuard` (VERIFICACION.md, H9a, H9c).
**[REPRODUCED, 2026-09-28]**: family A N1, N2 and N3 12/12, family B 6/6, N0 `seed_derived`: level N3.

### 4.7 Vector families

- **Family A · reconciliation**: A1 to A12 (`spec/state-vectors-v0.3.json`, schema `hbs-state/0.3`),
  including the extremes `2^40−1` (A11, A12).
- **Family B · reading the index** in the key format of `XMSSMT-SHA2_40/8_256` (OID `0x00000005`,
  h = 40, d = 8): 137 bytes = OID(4) ‖ index(5, *big-endian*) ‖ 4×32; in a single tree the index takes 4
  bytes and the key 136. B1 to B6 read; B7 writes `2^40` and must fail with `IndexOutOfField` **without
  touching a byte**. The Python verifier exercises 18 rows (12 of A and 6 of reading); B7 is bound only by
  `tests/vectors.rs` (VERIFICACION.md, H10a, H9c).
- **Declared without vectors**: B8 (with h not a multiple of 8, writing `2^h` must fail even if it fits in
  the field; the originating implementation does not cover it) and family C, seven errors about the
  on-disk state, among them `FakePersistence` (VERIFICACION.md, H19d).

That key format **is not standardised**: RFC 8391 (§4.1.7, §4.2.2), RFC 8554 (§4.2, §5.2) and SP 800-208
decline to define it. It is that of the reference implementation in C and of its translation `xmss`
0.1.0-pre.0; the revision of 2026-09-27 withdrew the attribution to RFC 8391, and making family B
conditional on a declared format is left for the next one (HBS-STATE v0.3, §6; VERIFICACION.md, H11b,
H11c).

### 4.8 The `fsync` self-check, as implemented

`IndexGuard::open` writes a probe, `.hbs-state-selfcheck`, in the parent directory of the counter:
20 writes of 8 bytes without `sync_all` and then 20 with `sync_all`, each batch timed with a single
`Instant` and divided by 20. It computes the ratio of the means with/without and returns
`FakePersistence{with_fsync_us, without_fsync_us, ratio}` **if and only if** the ratio is less than 10
**and** the mean cost with `fsync` is less than 20 µs (`hbs-state/src/lib.rs:523-556`). It is neither a
statistical distribution nor a difference of latencies (VERIFICACION.md, H7b, H7d). `zk-ssl-guardian`
applies the same algorithm (VERIFICACION.md, G1d).

The thresholds are **[DECLARED]**: 20 samples, 10× and 20 µs, set from a single machine. The measurement
that motivates them is **[MEASURED: `AUDITORIA.md` §234, WSL2 on i5-1135G7, with neither n nor spread
published]**:

| file system | cost of `fsync` | versus writing without persisting |
|---|---|---|
| ext4 (`$HOME` in WSL2, a virtual disk) | 0.907 ms | 382× |
| tmpfs (`/tmp`) | 0.002 ms | 1× |

The 382× is the ratio of `fsync` versus not persisting **on the same ext4**, not the ext4/tmpfs quotient
(VERIFICACION.md, M5b). A literal replica of the self-check (`doc/ecst/borrador/sonda-fsync.rs.txt`), 30
times per path in two batches, gave **[REPRODUCED, 2026-09-28, Firecracker microVM, 4 vCPU, ext4 root over
virtio]**:

| path | with `fsync`, µs, p50 [min–max] | without `fsync`, µs, p50 | ratio p50 [min–max] | rejections |
|---|---|---|---|---|
| ext4, batch 1 | 128.9 [74.7–292.2] | 0.876 | 161.4 [92.6–292.9] | 0/30 |
| ext4, batch 2 | 173.1 [103.6–795.2] | 0.879 | 189.4 [116.9–986.5] | 0/30 |
| tmpfs, batch 1 | 0.537 [0.531–1.582] | 0.421 | 1.3 [0.7–3.8] | 30/30 |
| tmpfs, batch 2 | 0.536 [0.533–0.732] | 0.421 | 1.3 [0.7–1.8] | 30/30 |

What this teaches is a limit. Inside a microVM, the guest cannot know whether the host carries the flush
through to the physical medium: the self-check only shows that `fsync` **costs something**, which is what
it declares it measures. It detects the «there is no disk» case (tmpfs); it does not detect a disk or a
virtual layer that acknowledges writes still in volatile cache, nor a fake `fsync` that costs 20 µs or
more, or that costs at least 10 times a write without persisting (VERIFICACION.md, G9a). The roughly
100 µs of an NVMe cited by the comments are an assumption, not a measurement (VERIFICACION.md, H7f).

### 4.9 What it does not cover

Per its §9: how the counter is persisted —`fsync`, ordering, locks, copies—; the policy; several signers
over the same public key; LMS and HSS, «the table ought to hold; it has not been tested»; family C and B8.
A third-party defect also prevents exercising end to end the persisted-key branch with the available Rust
library: `xmss` resolves the OID by trying single tree first, and 21 of the 56 XMSS^MT sets load
wrongly; `pq-xmss` no longer has the defect (HBS-STATE v0.3, §9; VERIFICACION.md, G10c).

### 4.10 Status

**[DECLARED]** HBS-STATE v0.3 is a **DRAFT**, revised on 2026-09-27, derived from **a single**
implementation; until a second independent subject passes the vectors, «it describes a programme» (§10).
The MEASURED label of its vectors means «taken from the code and the tests of Arqueo's guardian»: it is
self-conformance (VERIFICACION.md, H19a). `hbs-lms` was measured as a second **implementer**, with a
separate, unpublished harness, without going through the subject protocol and with no level attributed;
`pq-xmss` shares author and code with `xmss` and is not a second implementer (VERIFICACION.md, H19b, H19c).
**[REPRODUCED, 2026-09-28]**: `cargo test --release` in `hbs-state` comes out green (27 + 8 `passed`,
0 `failed`).

### 4.11 The cost of reusing an index

HBS-STATE v0.3 §2 gives, with the MEASURED label, «at the second reuse of an index, forging a signature
costs on the order of 2^34 hashes». The figure comes from a QRL blog article, «Statefulness and security»,
consulted on 2026-08-12 (`AUDITORIA.md` §288), which has no entry in `referencias.bib` because its metadata
could not be verified from the environment of this report: ~2^34 hashes with **two** signatures on the same
index, ~2^23 with three, ~2^18 with four. It is therefore **[CITED]**, not measured; 2^34 is the **most
expensive** point of that curve and not a minimum, so the drafts' formula, «as low as 2^34», inverts the
meaning; and «second reuse» must be read as two signatures, that is, a single reuse (VERIFICACION.md, H15,
B17a, B17b, B17c). The primary sources that RFC 10033 cites are [BH17] and [Fluhrer23], and the second
corrects the first: its Winternitz analysis assumed to be independent probabilities that are not. The QRL
figure has not been checked against either of the two, so it must not be cited as a figure; the conclusion
common to both suffices: a single reuse allows forgery at a feasible cost.

### 4.12 Correspondence with ECST

HBS-STATE instantiates **only** the reconciliation R(C,K) with its judge and the monotonic consumption of
OTS indices: it has no $\Pi_i$, no chain, no signed head, no authorization. What it contributes, and no
source of §2 provides yet —EIP-8310 announces regression test cases for the watermark and for crashes, but
does not publish them yet—, is an **executable semantic contract** against which another implementation
can be measured.

---

## 5. Instantiation 2: Arqueo

### 5.1 Trust model

**[DECLARED]** Arqueo is a research prototype, not a product; there is no production and no real operator,
and it does not handle real money (SECURITY.md, «Estado del proyecto» [Project status];
`doc/INSTITUCIONAL.md:91-92`; RFC-0010, «Seguridad» [Security]). One node, one writer, no distributed
consensus and no token. The operator sees all the balances, orders the operations and can censor. Nothing
in the project has been audited by third parties (README.md; SECURITY.md). The guardian was born in the
first half of August 2026 (`AUDITORIA.md` §234), some seven weeks before `hbs-state` was published; the
sentences «in production for a year» (README.md:170) and «a production system where it has been signing
for a year» (README.md:33-34; `src/lib.rs:73-74`) of `hbs-state` are false (VERIFICACION.md, G2a, H12b).
The only node that starts today is the one compiled with the `dev` feature, enabled by default, whose
custodian and governance roots come from test keys written in the source
(`crates/zk-ssl-node/src/main.rs:1336-1348`; `crates/zk-ssl/src/tests_support.rs`).

### 5.2 Two-phase payment and key custody

A payment is two transitions: the payer **sends** and the payee **claims**, each with a STARK proof
generated on their own machine through the SDK (`Account::pay`, which calls `client::prove_send`). The
layer delivers paths and roots and verifies (README.md, «Cómo funciona» [How it works]). Three
clarifications:

- **By design, not by types.** The spend key does not travel over the JSON-RPC API: no production method
  receives it, and in the SDK `Wallet::spend_key` is private and `Wallet` does not implement `Serialize`.
  But the layer keeps public methods that receive the key —`send`, `claim`, `burn`, `audit`, among
  others—, which the tests and the metrics use and the node does not call; there is no type barrier
  (VERIFICACION.md, C2a, C1b).
- **What the proof published.** Until §538 the proof the node received carried the spend key
  **literally**, and the custodian's in each delegated authorization, because winterfell 0.13 does not hide
  its witness; §521 is when it was measured, not when it began (README.md summarises it as «entre el §521
  y el §538» [between §521 and §538]). Since §538 (RFC-0009 E3b-2, `zkssl/0.4`) a fork in the tree of
  winterfell 0.13.1 hides it, and the guarantee is what the E2 suite measures —zero literals—, not a
  proof; neither the fork nor the construction is audited (README.md; SECURITY.md §3.bis; VERIFICACION.md,
  C3, C1c).
- **The payment is not final until the claim.** The amount stays locked until it is claimed or until the
  refund after expiry T: in v1 pending items, by default 64 log entries (`log.seq`, not 60 s heartbeats;
  `DEFAULT_REFUND_TTL`, `crates/zk-ssl/src/lib.rs:226`); in v2 ones, the deadline `delta` committed in the
  pending item itself (`crates/zk-ssl/src/two_phase.rs:656-676`).

### 5.3 What a proof reveals

Against a third party who sees only a proof, since §538 no balance comes out literally; before, the send
and the claim published the balance before and after. **The amount is a public input** of the send and
claim proofs; the metadata show the sender or the recipient and the position of the pending item. Against
the operator there is no privacy at all (VERIFICACION.md, C4b, P10b). Soundness rests on ~127
**conjectured** bits, not proven ones (`crates/zk-ssl/src/lib.rs:207-221`; VERIFICACION.md, M8a), and
«post-quantum» security means without curve assumptions, not a quantified quantum security.

### 5.4 The chained log

The actual composition (`crates/zk-ssl/src/log.rs:250-294`), with $M$ = `native_merge` and
`emb(x) = [x,0,0,0]`:

```text
c1_i = M( M( M( M(emb(seq_i), emb(tag_i)), M(Racc_{i-1}, Racc_i) ), pd_i ), c_{i-1} )    era 1
c_i  = M( c1_i, compromiso_i )                                                          era 2
```

`chain_digest` gives `c1_i` and `chain_digest_v2` the era-2 wrapper, which governs every new entry since
§281. `tag` is the u64 of `OpKind` (1 to 13), the `Racc` roots are those of the **accounts** tree and
`c_{-1} = [0;4]`. On disk, an entry takes 137 bytes in era 1 and 169 in era 2, and the length
discriminates the era (`crates/zk-ssl/src/store.rs:411-455`).

**It does include a proof digest.** `pd_i` is `digest_of_proof`: Blake3-256 of
`b"ZK-SSL-proof-digest-v2"` ‖ length (u64 LE) ‖ proof bytes, split into four u64 and reduced to the
field (`crates/zk-ssl-hash/src/lib.rs:1290-1320`). But it depends on the class (VERIFICACION.md, L1e, L1f):

- **Send, Claim, Burn and Refund** chain the digest of their STARK proof.
- **Mint, MintToPending, Recovery, Freeze and Governance** chain the digest of `sello_de_autorizacion`
  over the authorizing commitment, not the bytes of any proof.
- **OpenAccount** chains the constant seal of declared absence; **Migration**, the digest of a 64-byte
  block with the old and new frozen roots; **Consumo**, `digest_of_proof(&[])` with the digest of the
  consumption as commitment.

That is why Proposition 1 covers the send, claim, burn and refund proofs, and not the custodians' ones. The
two runs of `simulate` illustrate it **[REPRODUCED]**: with deterministic keys, the Mint entries (log#1,
log#3) have the same chained digest in both even though their proof changed, and the Send (log#4), with
the same roots, has different digests (`54dd7634…` and `e1db21f3…`), because with hiding the bytes of the
proof depend on the salt (`doc/ecst/borrador/simulate-2026-09-28.log`, `simulate-2026-09-30.log`). It is an
observation about two trees with the same log formula, not a designed experiment.

Other limits (VERIFICACION.md, L1d, L5c, L5d, L12):

- `chain_digest` and `chain_digest_v2` **carry no domain tag**: the era is separated by the additional
  merge and by the length on disk (R3 is met only structurally).
- **The proofs are not kept.** The node stores only the digest, no RPC method serves proofs, and
  re-verifying them depends on the producer keeping theirs, with no retention policy (D3 fails).
- **Nobody recomputes the chain for a third party.** It is recomputed by the node over itself
  (`zkssl_verifyChain`), by the CLI over its local layer and by the layer on reopening. `spec/RPC.md:97`
  describes `LogEntry` without the `compromiso` field, so with the specification in hand `chain_digest_v2`
  cannot be recomputed (the wire DTO does carry it).

### 5.5 The v6 epoch head

Since §570 the node composes, signs and serves the **v6** head (`VERSION_FORMATO = 6`,
`crates/zk-ssl-verify/src/lib.rs:166`), which commits 18 fields: `seq`, the roots of accounts, pending
items and frozen, `chainDigest`, the acknowledgement root and its ceiling `n` (1,440 heads), the top and
the size of the history tree of heads, the consumption root and count, the parameters digest, the
pending-meta root, the two high-water marks (`nextPending`, `nextIndex`), the supply, and the
`recepRoot`/`recepCount` pair of the reception log (`crates/zk-ssl/src/log.rs:586-649`;
`crates/zk-ssl-hash/src/lib.rs:469-510`). Before it, v5 (§452), v4 (§415), v3 (§292) and v2 (§275) were in
force. Only 50 bytes are signed, `b"ZK-SSL-epoch-head"` (17) ‖ version (1) ‖ `epoch_digest` (32), with
XMSS^MT-SHA2_40/8_256; the fields are covered because they recompute `epoch_digest`. The head binds the
last digest of the chain, not the entries (VERIFICACION.md, L2a, L2b, L2c).

The signer (`crates/zk-ssl-node/src/firma_cabeza.rs:179-203`) **reserves the index with `fsync` before
signing** in the guardian and verifies its own signature before returning it; the declared index is the
value of the counter and the spent leaf, embedded in the signature, is counter − 1 (VERIFICACION.md, G4c).
The heartbeat signs every 60 s by default and only if the operator started with key and journal; without a
key, the head is computed and served unsigned (VERIFICACION.md, G4d). The dependency `xmss` 0.1.0-pre.0
from RustCrypto is a *pre-release* without independent audit, pinned with `=` in the node, the verifier
and the CLI (VERIFICACION.md, G4b, M10b).

### 5.6 Witnesses

The reference witness (`crates/zk-ssl-cli/src/witness.rs`) pins by TOFU the key it sees first, stops if it
changes or if it sees two digests with the same index and, with `--cofirmar`, co-signs with its own XMSS
key and its own guardian. There are no independent witnesses: only that implementation, which the author
runs (VERIFICACION.md, G5a, L7b). The co-signatures are optional and how many suffice is decided by the
client (G5b); the node carries them and checks their signature, but accredits no witness (G5c). The
operator serves only the last signed head; the history is kept by the witnesses' journals (L4d).

### 5.7 Evidence packages and the independent verifier

`zk-ssl-verify` is a binary that depends neither on the layer, nor on the node, nor on the wire, and
accepts eleven package forms: position v1 and v2, extension, consumption, conflict, rejection, age,
pending claim, payment in progress, pledge and, since §573, completeness
(`crates/zk-ssl-verify/src/main.rs:158-180`). It exits with 0 (VERDE [green]), 1 (ROJO [red], with the
named rule), 2 (usage) or 3 (the fourth state of the completeness envelope).

**What it checks.** In the position package: it recomputes the `epochDigest`, verifies the XMSS signature
against the key that the package itself carries, climbs the acknowledgement up to the acknowledgement root
and counts the co-signatures. It verifies STARK proofs only in the envelopes that carry them —age, pending
claim, payment in progress, pledge, certain rejections—, which are **new** proofs over the state committed
under a v5 or v6 head, not the $\Pi_i$ of the log (VERIFICACION.md, L5b).

**What it does not check.** It does not re-verify the transition proofs, which are not kept; it neither
walks nor recomputes the log; it verifies no aggregate of balances: the invariant supply = balances +
pending is checked by the operator's layer on reopening, over cleartext balances (VERIFICACION.md, L9a,
C4c, C4d). The published release `arqueo-verify-v0.2.0` predates the v5 and v6 heads and rejects them; the
verifier in the tree does check them, but it has not been published as a release (VERIFICACION.md, L9b).
**[REPRODUCED, 2026-09-30]**: nine catalogues, 257/257 entries say what they should (§7.2).

### 5.8 Consistency and inclusion

What the code calls «MMR de cabezas» [heads MMR] is a history tree in the style of RFC 6962 (MTH, PATH,
SUBPROOF) over its own primitives, with domains `MMRHOJA1`/`MMRNODO1`; its leaves are earlier heads, not
entries (`crates/zk-ssl-verify/src/mmr.rs`; VERIFICACION.md, L4a). `zkssl_consistencyProof` gives the
extension path, and the extension package is verified without the node: two heads v3 to v6 that recompute
and verify, the same public key and the new top extending the old one (VERIFICACION.md, L4b, L4c). The
structure gives **tamper evidence, not immutability**: only whoever holds an earlier head of the same key
sees it; the consistency proof does not check that the new `chainDigest` continues the old one, and the
extension package does not check that the head held in custody is itself a leaf of the new tree
**[READING]** (VERIFICACION.md, L4d).
The inclusion of an applied operation that the kit proves is that of its **acknowledgement** under the
acknowledgement root, which only `applySend` and `applyClaim` emit; it also proves that of a consumption
under `consRoot` and, since §573, that of a receipt under `recepRoot` (VERIFICACION.md, L5a).

### 5.9 Single use within a ledger, detection across ledgers

The consumption tree of RFC-0006 gives uniqueness of an agreed public label **within** a ledger: the
layer rejects the repeated consumption, and the consumption root and count are signed since the v4 head.
Across ledgers, a third party with the two signed heads detects the same label afterwards, without nodes:
**detection, not prevention**. The consumption is not bound to any payment, neither in circuit nor by
authentication of the publisher (VERIFICACION.md, C5a, C5b). The double spending of balances is prevented
by something else: the chaining of roots under the total order of a single node, «evitado, no resuelto»
[avoided, not solved] (SECURITY.md §3.4).

### 5.10 Recovery

Recovery replaces the identity of **the same leaf**, with the same balance and the nonce + 1: the circuit
that the layer uses, `circuit_recovery_climb`, builds the old and the new leaf with the same balance
column, so it moves no money (VERIFICACION.md, C6a, C6b). It increments by one a **global** recovery
counter of the ledger, which enters the commitment that two custodians sign and acts as a nonce against
replay; **it is not in the signed head** nor in the RPC, and the proof that asserts it is not stored
(VERIFICACION.md, C6c). The double authorization is in the layer, not in the circuit: two proofs from two
distinct custodians, with the threshold fixed at 2; nothing proves that the new holder is legitimate.
Recovery is not exposed in the JSON-RPC and is exercised only by the tests (VERIFICACION.md, C6d, C6f).

### 5.11 Operator powers, and what a third party can re-verify

With two custodians one can issue (with the cap in circuit), issue to pending, freeze or unfreeze any
account with no justification in circuit and no expiry, and reassign any account. With two governance
members the custodian set is changed. The operator, alone, can see all the balances, order, open accounts
at zero, publish consumptions, set the quota and the expiry —visible in `paramsDigest` since the v5
head—, declare a custody of its key that is not checked, and change the verifier without a trace
(VERIFICACION.md, C7c, C7d; SECURITY.md). Of the delegated operations the log keeps only the seal of the
authorization: a third party can recompute the seal, but can re-verify neither conservation in circuit nor
that two distinct custodians authorized (VERIFICACION.md, C8). `reverificar()`, which checks the seals,
exists in `crates/zk-ssl-verify/src/reverificacion.rs` and has no callers other than its own tests
(VERIFICACION.md, L5d).

### 5.12 Censorship after RFC-0010

RFC-0010 has been ACCEPTED since §577. Every operation that the node **evaluates** through `applySend` or
`applyClaim`, also if the layer rejects it, reserves a reception number with `fsync`, records
(`rx`, era, proof digest) before evaluating it and, when its era closes, remains as a leaf under the
`recepRoot` of a signed v6 head; the era is the XMSS index of the last signed head plus one. With the
receipt and its path, which the node serves through `zkssl_recepPath` when the era closes, the holder forms
a **completeness envelope** that is verified without the node, with four outcomes: resolved as applied;
resolved as a rejection with proof; «declarada, no probada» [declared, not proven], for the causes without
a portable proof; or «NO RESUELTA EN LA VENTANA» [NOT RESOLVED IN THE WINDOW] of N = 1,440 signed heads, a
named ROJO. It is **opposable evidence signed by the operator itself, not a cryptographic proof of
absence** (RFC-0010 D-F to D-H; spec/PAQUETE.md, 2.11; VERIFICACION.md, C12a, C12b).

Still without an opposable trace (RFC-0010 D-E and D-H; README.md, «Qué garantiza y qué no» [What it
guarantees and what it does not]): the operation to which the node never gave a receipt —it does not
answer, answers without a receipt or fails to record—; everything that enters through `zkssl_applyMany` or
`zkssl_pledge`, left out by decision of §576; and the methods that do not reserve a receipt, such as
`zkssl_openAccount` or `zkssl_publishConsumo`. Nor does a receipt leave one when the node does not serve
its path and the holder did not request it in time. The receipt log grows without pruning: `podar` exists
and has no caller by decision of §580, because pruning when the window expires would leave that holder
without an envelope (RFC-0010, «Seguridad» [Security]). **[REPRODUCED, 2026-09-30]**: the completeness
catalogue gives 35/35.

### 5.13 The signing index and the reception log: two instances of R(C,K)

Arqueo uses the **same** `zk_ssl_guardian::Reconciliacion` (Coincide, ContadorAdelantado, ClaveEnCero,
ClaveAdelantada, equivalent to the four states of §4.4) for two resources, with two policies
(`crates/zk-ssl-node/src/main.rs:592-691`; VERIFICACION.md, G1g, G6b).

| state | **signing index** (seed-derived key) | **reception log** (persisted-key model) |
|---|---|---|
| C and K | the guardian's counter; index read from the key | counter `rx`; largest `rx` recorded in the log (0 if empty) |
| Coincide | starts | starts |
| ContadorAdelantado | starts with a warning (orphans) | starts with a warning: declared gaps |
| ClaveEnCero | resyncs the key to the counter and abandons 0..C−1, **unless** the journal shows a larger index, in which case it does not start | starts with a warning, without resyncing: empty or lost log |
| ClaveAdelantada | does not start: key compromised | does not start: the counter was restored without the log |

Three observations. (1) The classification is the same and the policy is not: on ClaveEnCero one instance
resyncs and the other moves nothing, because «no hay clave que mover» [there is no key to move]. It is a
second use of the same classification, with the same type and by the same author: it indicates that R(C,K)
**generalises as a classification** over two monotonic counters, one of which must stay behind the other,
but it is not independent evidence, nor does it support it as a guarantee (VERIFICACION.md, G6d). (2) The
meaning depends on the model: in the log, with a persisted key, ContadorAdelantado is the normal case after
a crash and ClaveEnCero arises from a lost log, not from a restart. (3) The pair is not enough: restoring
**only** the log to an old state gives ContadorAdelantado, just like a gap from a crash. For a joint
restore, Arqueo added a datum outside the pair: the heartbeat does not compose a head if the reception
counter is below the `recepCount` of the previous one, provided that that head or the journal survive
(`crates/zk-ssl-node/src/latido.rs:270-287`). For the signing index, that datum is the node's journal,
with the limits declared by `doc/CONFIANZA_RESIDUAL.md` —restore of the entire directory, deleted journal—
and the one that §8.1 adds.

`tools/banco_reutilizacion.sh` measures since §579 that the witness resyncs on restart, by the log line
«clave resincronizada» [key resynchronised]; its assertion that no co-signature repeats an index compares
the declared ordinal, not the embedded WOTS index **[READING]** (VERIFICACION.md, G1f). The failure is not
hypothetical: before the ClaveEnCero variant, the witness run twice with the same seed and the same counter
reused WOTS indices 0 to 4, and a detector that looked at the ordinal did not see it (`AUDITORIA.md` §331).

### 5.14 ECST component ↔ Arqueo

The paths of each row are in Appendix A.

| ECST component | in Arqueo | status |
|---|---|---|
| $C_{i-1}$, $C_i$ | root of the accounts tree in the entry; the other roots, in the head | exists |
| $O_i$ | `seq` and `tag` of `OpKind` | exists |
| $\Pi_i$ | STARK per transition, generated by the holder | exists; **not kept** |
| $\mathrm{Hp}(\Pi_i)$ | `digest_of_proof` | only Send, Claim, Burn, Refund |
| $A_i$ / $a_i$ | authorship inside the STARK; custodian pair → `compromiso` and seal | the seal is chained, not the proofs |
| $H_i$ | `chain_digest_v2` | exists; no domain tag |
| Link | `TransitionLog::verify_chain` and `verify`; `zkssl_verifyChain` | only the node and the CLI |
| Verify | native verification in `apply_*`; envelopes of the verifier | does not re-verify historical $\Pi_i$ |
| authenticated head | v6 head signed with XMSS^MT | exists; TOFU anchor |
| witnesses | reference witness with TOFU and co-signature | exists; none independent |
| R(C,K) | `GuardianIndice` + `Reconciliacion`, two policies | exists; two instances |
| single use | XMSS index and `rx` (by counter); consumption tree and pending leaf (by set) | exists |
| completeness | completeness envelope | only `applySend`/`applyClaim` |
| result vector | verifier codes 0 / 1 / 3 | partial: per package, not per history; 3 is «declarada, no probada» [declared, not proven], not missing data |
| tiered audit A/B/C | — | **does not exist** as a tool |
| versioned canonical encoding | domain registry; frozen core with KAT | partial |

---

## 6. Threat model, guarantees and non-guarantees

### 6.1 Adversaries

- **Operator or network.** Orders, delays or censors operations; can show different views to different
  observers; sees all the balances; can change the verifier without leaving a trace, because the AIR is
  code and not data (SECURITY.md; `doc/CONFIANZA_RESIDUAL.md`).
- **Execution and storage.** Kills the process at any instant; restores backups; deletes files; places the
  counter on tmpfs; a disk acknowledges writes it did not persist; the power is cut.
- **Cryptographic.** Searches for collisions of $\mathrm{Hc}$ or $\mathrm{Hp}$; attacks the soundness of
  the STARK (~127 conjectured bits); exploits a reused XMSS leaf; induces faults during signing.
- **Client side.** Steals the spend key on the payer's machine, where the prover and the key live;
  exploits side channels; cracks a weak password of the SDK keystore, whose derivation is SHA-256 and not
  a password KDF (`crates/zk-ssl-sdk/src/keystore.rs:5-13`).

### 6.2 Guarantees, with their hypotheses and what backs them

| # | guarantee | hypotheses | backing |
|---|---|---|---|
| G1 | altering a chained field of $E_k$ changes $H_n$ | Proposition 1: R1 and R2 (known version and era), R4, collision resistance of Rescue-Prime and of reduced Blake3 as a function of the bytes; across eras, the preimage-type property of §3.5 | `t1_chain_retroactivo` **[REPRODUCED]**; `AUDITORIA.md` §115 |
| G2 | a signed head comes from whoever holds the key | security of XMSS^MT; no reused leaf; custody of the key (declared, only the file modality is checked); anchor of the key (TOFU) | tests of `firma_cabeza.rs`; catalogue `paquete` **[REPRODUCED]** |
| G3 | whoever holds an earlier head detects a rewritten history of heads | the earlier head held in custody; same key. It does not cover that the head held in custody is itself a leaf of the new tree, which the extension package does not check **[READING]** (VERIFICACION.md, L4d), nor the continuity of the `chainDigest` (§5.8) | `zkssl_consistencyProof`; extension package in the catalogue `paquete` |
| G4 | a label is consumed once in a ledger; across ledgers it is detected afterwards | total order of one node; two signed heads | RFC-0006; catalogues `consumo` and `conflicto` **[REPRODUCED]** |
| G5 | an operation with a receipt that was not resolved in N heads leaves an opposable ROJO | that the node issued a receipt and later serves its path (`zkssl_recepPath`), or that the holder requested it in time; signature of the head that closes the era; and, while §8.2 is not resolved, a ROJO for a proof rejected as invalid does not distinguish the operator who censors from the holder who sent an invalid proof | RFC-0010; catalogue `completitud` **[REPRODUCED]**; `tools/banco_completitud.sh` |
| G6 | no head signature has an index greater than the persisted counter | persist-before-sign ordering; `fsync` honoured; no restore, deletion or reset to zero of the counter | bench K.1, 25/25, on the isolated guardian, before the head signer existed **[MEASURED: `AUDITORIA.md` §234]**; the reserve-sign-verify ordering of `FirmanteCabeza::firmar`, by reading and by tests (VERIFICACION.md, G4c); the guardian's tests **[REPRODUCED: `26 passed`]** |
| G7 | after a restart, the signing key does not reuse the undetermined indices | the policy of §3.8; the journal present and not restored; **and, until §594, the counter neither deleted nor reset to zero (§8.1)** | for the node, `politica_de_reconciliacion` and its unit tests **[READING]**, with no bench that restarts the node; for the co-signing witness, `politica_del_cofirmante` and `tools/banco_reutilizacion.sh` (§579), which measures the resync, compares the declared ordinal and not the WOTS leaf, and whose negative restores the counter to 1, not to 0 (§5.13) |
| G8 | the spend key neither travels over the API nor comes out literally in the proof | deployment with the SDK; hiding of the winterfell fork | no RPC method receives it; E2 suite of RFC-0009 (§538): measured, not proven |

### 6.3 Non-guarantees

- That a third party can validate the complete history: the proofs are not kept and nobody recomputes the
  chain for them (§5.4, §5.7).
- That a third party verifies conservation: they receive the supply and the roots, signed; the aggregate
  is checked by the operator (§5.7).
- Privacy against the operator, who sees everything.
- Completeness of what never received a receipt, or entered by batch or pledge (§5.12).
- Prevention across ledgers: only detection (§5.9).
- Identity behind a key, or independence of witnesses and custodians: two compromised keys authorize just
  as two wills do.
- Durability across a power cut; resistance to a rollback of the node's entire directory, and —before
  §594, or without a journal— to the deletion or reset to zero of the counter (§8.1); resistance to faults induced in the XMSS^MT signature,
  which has not been evaluated.

---

## 7. Evaluation

### 7.1 Measured

Everything in this subsection is **[MEASURED]** by the author on a laptop under WSL2 except where
indicated, with the sources confirmed by the sceptic of block M. The model, an Intel Core i5-1135G7, is
recorded in §229 and §234; the earlier series (§89, §130–§131, §204–§217) do not document the hardware, and
§181 mentions a «Ryzen del piloto» [pilot's Ryzen] (VERIFICACION.md, M1). The time figures depend on the
machine, and the repository itself does not bind them with gates (`AUDITORIA.md` §304).

| what | figure | source and conditions |
|---|---|---|
| guardian's `fsync` | ext4 0.907 ms (382× versus not persisting); tmpfs 0.002 ms (1×) | §234, bench K.1, with neither n nor spread published |
| process death | 25 of 25 with no signature ahead; 13 of 25 with the counter ahead, within the process | §234, on the guardian still without a consumer, before the head signer; the bench code is not in the tree |
| size of a send proof | 77,444–80,232 B | §538, 15 samples per axis, with hiding |
| size of a claim proof | 76,192–79,736 B | §538, idem |
| one payment (two proofs), measured | 155,337–159,329 B | §538 |
| one payment, published band **[DECLARED]** | 145,953–167,967 B | §538: what was measured with a declared margin of 5%; `crates/zk-ssl/src/metrics.rs:82-83` |
| verifying a send or claim proof | 2.32–2.41 ms (mean 2.35 ms, five runs); 2.43–2.49 ms | §89.1 and §204; **prior to §538** |
| proof digest | Blake3 0.011 ms versus 30.99 ms with Rescue, on the same proof | §204, §209 |
| generating a proof | send 322–353 ms, claim 218–243 ms; intra-batch σ ≈ 0.5%, drift between batches ≈ 9% | §130–§131, two batches of five; **prior to §538** |
| implicit cost per Rescue merge (≈ one permutation), of `set_leaf` and of startup | 7.44–8.91 µs | §217 (and 7.53 in §204): times of tree operations divided by their merges, with the overhead included |
| node ceiling over RPC | 248 op/s (line 0.225 + 4.035·n ms) | §229: `zkssl_applyMany` batches of 1, 4, 8 and 15 sends, three repetitions; in-memory node, one serial client, prior to §538 and without reception receipt |
| XMSS^MT signature of the head | sign 144.5–160.5 ms; verify 2.4–2.7 ms; 18,469 B | `doc/xmss-evaluacion.md` and §236, two dates |
| wrapping in a zkVM | 47.5 M cycles; succinct receipt of 223,234 B | §305–§307, pod with an RTX 5090 whose host is not identified; one proof prior to §538 |

The generation times after hiding (send 697.9–741.6 ms and claim 696.9–715.6 ms, as minimums) are recorded
only in a comment of `metrics.rs` and in D-AJ of RFC-0009, not in `AUDITORIA.md`; there is no re-measurement
of verification after §538 (VERIFICACION.md, M3a, M3b). The figures of 66,739 and 66,692 B that a comment
of `metrics.rs` cites are from §512, prior to hiding, and are not the current figure (VERIFICACION.md,
M2b).

### 7.2 Reproduced in this revision

Environment: Firecracker microVM, 4 vCPU «Intel(R) Xeon(R) Processor @ 2.10GHz», 15 GiB of RAM, ext4 root
over virtio and `/dev/shm` on tmpfs; rustc/cargo 1.94.1. On 2026-09-28 on Arqueo d531c80 and `hbs-state`
a960828 (kernel 6.18.44-fc-v37); on 2026-09-30 on Arqueo 71c5aad plus the draft's commit (kernel
6.18.44-fc-v50). One run per day: it is not a measurement with spread (`REPRODUCCION.md`).

- **Arqueo, 2026-09-30:** the layer's `t1_` filter gives `3 passed, 0 failed`; `zk-ssl-guardian` gives
  `26 passed, 0 failed`; `tools/conformidad.sh` with the tree's binary (fea39a053efd7089) gives paquete
  70/70, consumo 14/14, conflicto 16/16, rechazo 84/84, edad 11/11, pendiente 9/9, pago 9/9, prenda 9/9 and
  completitud 35/35: **257/257** entries.
- **`simulate --amount 250000`, one run each day**, sandbox with deterministic keys (seed
  0xa11ce), real STARK proofs. «KB» in the CLI means bytes/1024.

| date, tree | issuances | send | claim | apply |
|---|---|---|---|---|
| 2026-09-28, d531c80 | 64.3 KiB [417 ms], 65.3 KiB [345 ms] | 77.6 KiB [970 ms] | 76.7 KiB [1,198 ms] | 4–6 ms |
| 2026-09-30, 71c5aad | 66.8 KiB [328 ms], 66.5 KiB [364 ms] | 78.9 KiB [1,333 ms] | 77.8 KiB [879 ms] | 5–6 ms |

Send plus claim add up to about 158,000 and 160,460 bytes, **within** the published band; the send of
2026-09-30, about 80,800 bytes, is above the maximum **measured** in §538 and within the band. With hiding,
the size changes from one run to another according to the salt, and the expensive side flips between runs:
absolute times do not transfer between machines or between runs. The issuance proofs have no published
band.

- **`hbs-state`, 2026-09-28:** `cargo test --release` green; `verify-state.py` reaches N3 (§4.6); the table
  of the `fsync` self-check of §4.8.

### 7.3 Derived

- **Log growth:** 169 B of value per era-2 entry, plus 12 B of key in `sled` (`"log:"` and the `seq` in
  eight bytes): **181 B** unencrypted; with the ledger encrypted (XChaCha20-Poly1305, 24 B of nonce and
  16 of tag), **221 B**. Without the internal overhead of `sled`, which has not been measured
  (`crates/zk-ssl/src/persistence.rs:839-841`; `crates/zk-ssl/src/crypto.rs`). To that are added 40 B for
  each `applySend` or `applyClaim` evaluated in the reception log
  (`crates/zk-ssl-node/src/registro_recepcion.rs:110`).
- **Cost of the chaining:** five Rescue merges per entry in era 1 and six in era 2, at the implicit cost
  per merge of 7.44–8.91 µs (§217), give about 37–54 µs per entry; against an `apply_send` of 3.11–3.33 ms
  dominated by STARK verification, it is of the order of 1–2%. All prior to §538 (VERIFICACION.md, M4a).
- **Storing the entry instead of the proof:** 169 B versus 76,192–80,232 B per proof is a saving of
  99.78–99.79%. It does not «reduce» anything deployed, because the node never stored proofs
  (VERIFICACION.md, M4d).
- **Chain recomputation rate:** about 19,000–27,000 entries per second and core, estimated from the cost
  per merge; not measured (VERIFICACION.md, M4b).
- **Bound of bench K.1:** ≈ 11% at 95% for 0 failures in 25 (§4.2).

### 7.4 Not measured

Everything in this subsection is **[NOT MEASURED]**:

- The cost of ECST versus IVC on the same workload, machine and circuit. The only thing measured with
  folding is a proof of concept with Nova whose step is a single hash, with curves and a commitment that
  requires a ceremony; it is not comparable (FIVE_BACKENDS.md; VERIFICACION.md, M7b, M7c). Nor does it
  represent curve-free, hash-based accumulation [BMNW25], which is the alternative that would have to be
  measured against ECST. The possible advantage of ECST over IVC is presented here only as a qualitative
  argument, unmeasured.
- The performance of a complete historical audit by a third party: the tool does not exist (§5.7).
- Durability across a power cut, in the two systems.
- The cost of the two additional `fsync` per operation that the reception receipt adds —counter and log,
  plus the directory one when each era opens— and the growth of the log on disk at scale.
- The node's concurrency with several clients.
- LMS and HSS against HBS-STATE.
- The scenario of §8.1, with the counter deleted or reset to zero: it has not been reproduced.
- Whether `xmss` 0.1.0-pre.0 caches the signatures of the intermediate layers of XMSS^MT, which is the
  countermeasure that VERIFICACION.md records (P12a) against grafting-type faults, first described against
  the SPHINCS framework [Grafting18].

### 7.5 Figures from the drafts that this report withdraws

| figure from the drafts | entry | why it is withdrawn | what there is |
|---|---|---|---|
| «entorno estandarizado» [standardised environment] AMD EPYC 7763, 64 cores, 128 GB, NVMe | M1 | no measurement in the repository declares it; the 64 cores are a sizing assumption | a laptop under WSL2 (i5-1135G7 where recorded; §7.1) |
| proof $\Pi_i$ of ~62.4 KB | M2a | no source; «~62 KB» is prose prior to hiding | send 77,444–80,232 B; claim 76,192–79,736 B (§538) |
| generating in 412 ms | M3a | no source | §130–§131 and the minimums after §538 (§7.1) |
| verifying in 8.1 ms | M3b | false; the 8 ms of the tables is the PLONK/KZG verification of the comparison circuit | 2.32–2.41 ms (§89.1) and 2.43–2.49 ms (§204), prior to §538 |
| chaining in less than 0.002 ms | M4a, L6d | false and without source; 0.002 ms is the `fsync` on tmpfs | estimate of 37–54 µs (§7.3) |
| audit at ~125,000 blocks/s | M4b, L6c | no source; Arqueo has no blocks | estimate of 19,000–27,000 entries/s and core, not measured |
| ~256 B per transition | M4c, L6a | false | 137 or 169 B per entry (code) |
| 99.6% reduction | M4d, L6b | quotient of two unsourced figures | 99.78–99.79%, derived |
| `fsync` ext4 1.82 ms / tmpfs 0.03 ms | M5a, H7c2, G3d | no source | 0.907 ms / 0.002 ms (§234) |
| the test T1 with N = 100 epochs, alteration at k = 10 of one byte of $\Pi_k$ | L3a, L3b, L3c | false | N = 12 entries, K = 5, synthetic proofs, no STARK |

Also withdrawn, without being measurement figures, are the numerical self-assessments, the threshold that
«eleva exponencialmente» [raises exponentially] the complexity of the attack, and the claim that the
guardian has been «en producción desde hace un año» [in production for a year] (VERIFICACION.md, P18b,
P13, G2a).

---

## 8. Limitations and open findings

### 8.1 READING FINDING, NOT REPRODUCED: a signing counter deleted or reset to zero falls into `Coincide`

> **Status (2026-09-30): FIXED in §594 (commit 78d71a4 of the working branch)**, after the report's base. Both gates —the
> journal gate in the node (`<`) and the co-signature gate in the witness (`<=`)— are now consulted in
> `Coincide` and `ContadorAdelantado`, not only in `ClaveEnCero`, with their usual operators and texts.
> **[MEASURED in this session, microVM]**: a test in `zk-ssl-guardian` measures the premise —a deleted
> counter reopens at 0 and reconciles `Coincide { indice: 0 }` with the key at 0—; the node gains four tests
> and the witness three, positives first; with the new gate disabled exactly the four new reds fail, and
> through the wrong branch; and `tools/canon.sh --sello` comes out GREEN with the pins moved. **What is still
> not done**: reproducing it with a real node or in a bench that deletes the counter between two starts; the
> same change in `hbs-state`, whose `open` has the same premise; and the entry in `AUDITORIA.md`, which is
> the author's. What was already declared stays as it was —without a journal, or with the whole directory
> restored, it is not seen—, and there is a new consequence, declared in the code: a new key with the old
> key's journal no longer starts. The text below describes the tree at 71c5aad and is kept as it was; its
> line numbers are those of that commit.

**[READING]** In `crates/zk-ssl-node/src/main.rs`, `politica_de_reconciliacion` (lines 592-652) consults
the journal —the second record, the one that detects a counter restored backwards— **only** in the
`ClaveEnCero` branch:

```rust
// abridged excerpt from crates/zk-ssl-node/src/main.rs, 592-652
zk_ssl_guardian::Reconciliacion::Coincide { indice } => {
    DecisionDeArranque::Arranca(format!("guardian y clave a la par en el indice {indice}"))
}
// ...
zk_ssl_guardian::Reconciliacion::ClaveEnCero { contador, indeterminados } => {
    match tope_diario {
        Some(d) if *contador < d => DecisionDeArranque::NoArranca(/* ... */),
```

If **only** the signing counter file is deleted, `GuardianIndice::abrir`
(`crates/zk-ssl-guardian/src/lib.rs:419-444`) recreates it at 0 without warning; the same happens if the
file is restored with the value 0, for example from a copy taken after the first start —`abrir` persists
the 0 when creating it— and before the first signature. The key, derived from the seed, is also at 0; the
reconciliation gives `Coincide { indice: 0 }`, the policy starts without looking at the journal and the
signer would sign again with declared index 1, that is, spending leaf 0 and the following ones again:
**reuse of XMSS leaves**, even though the journal has a larger index recorded. The co-signing witness has
the same shape: `politica_del_cofirmante` (`crates/zk-ssl-cli/src/witness.rs:2696-2771`) consults the
co-signatures only in `ClaveEnCero`.

It is not among the limits declared by `doc/CONFIANZA_RESIDUAL.md` —restore of the entire directory,
deleted journal, journal without `fsync`, unreadable lines— nor in entry 103 of `BACKLOG.md`, and no test
or bench covers it: the `Coincide` test passes `None` as the journal ceiling, and the negative of
`tools/banco_reutilizacion.sh`, which exercises the witness and not the node, restores the counter to 1,
not to 0. Since the signing index is also the clock of the receipts' eras
(`crates/zk-ssl-node/src/main.rs:1061-1063`), the deletion or the reset to zero would also move that clock
backwards **[READING]**. **It was found by reading, it has not been reproduced**, and it is reported to
the author. Suggested fix: apply the journal check —in the witness, the co-signatures one— **whatever**
the state of the reconciliation, not only in `ClaveEnCero`, and add a bench that deletes the counter, or
resets it to zero, between two starts of the node. It is the lesson of §3.7: R sees only the pair, and the
datum outside the pair has to be consulted always (VERIFICACION.md, P15b).

### 8.2 Possible gap in the completeness envelope

**[READING]** The verifier admits as a «declarada, no probada» [declared, not proven] resolution only four
causes: `CustodianSetExhausted`, `PendingTreeExhausted`, `NotTheIssuer` and `NotTheAccountHolder`
(`crates/zk-ssl-verify/src/main.rs:1622-1628`). It does not admit `ProofFailed` or `VerificationFailed`,
which RFC-0007 declared without a portable proof and for which there is no rejection envelope. If this
reading is correct, a send or claim rejected for an invalid proof cannot be resolved as VERDE or in the
fourth state, and its envelope ends in ROJO, in tension with D-G of RFC-0010; and a holder could provoke a
ROJO against an honest operator by deliberately sending an invalid proof, which does consume a receipt. It
is an inference by the sceptic of block C, collected in the working material
(`doc/ecst/borrador/refutacion-custody.json`) and **with no vector that exercises it**.

### 8.3 Living documents that do not reflect RFC-0010

§581 updated row 6 of the README and its lists of what is not claimed, but left without a caveat, in the
header summaries, that the operator «puede omitir una operación sin dejar rastro» [can omit an operation
without leaving a trace]: README.md:34, README_EN.md:35, QUESTIONS.md:30, PREGUNTAS.md:30,
RESUMEN_EJECUTIVO.md:35 and RESUMEN_BILINGUE.md:28 and 81 (checked at 71c5aad). They contradict the
wording of the README itself, «Qué garantiza y qué no» [What it guarantees and what it does not]: the
censorship of what the node never acknowledges leaves no trace; that of what it acknowledges does. Other
verified drifts: the docstring of `politica_de_reconciliacion` (`crates/zk-ssl-node/src/main.rs:584-591`)
still says that ContadorAdelantado is «el caso NORMAL tras una caída» [the NORMAL case after a crash],
which with a seed-derived key does not happen after a restart (§4.3), and that ClaveEnCero «SÍ para»
[DOES stop], the opposite of the code, which resyncs; and `spec/RPC.md:97`
describes `LogEntry` without `compromiso` (§5.4).

### 8.4 A rustdoc comment attached to the wrong variant

In `crates/zk-ssl-guardian/src/lib.rs:337-351`, the comment of `ClaveAdelantada` («LO QUE NUNCA DEBE
PASAR … La clave debe considerarse comprometida» [WHAT MUST NEVER HAPPEN … The key must be considered
compromised]) is written above that of `ClaveEnCero`, so rustdoc attributes both to `ClaveEnCero` and
leaves `ClaveAdelantada` undocumented. Whoever reads the generated documentation may conclude precisely the
false rule «KeyAtZero ⇒ do not sign». `hbs-state` has the correct order (VERIFICACION.md, P15a).

### 8.5 Verification status and other limitations

- **Formal verification.** There is no mechanised proof of the soundness of the AIR, of FRI, of Rescue or
  of the layer. FV-1, a syntactic census of cells, is a gate of the canon in six circuits and does not
  cover the v2 variants; FV-2 was an SMT probe over `circuit_refund` with Rescue abstracted; FV-3 (Lean,
  Coq or K) is a declared horizon (`doc/VERIFICACION_FORMAL.md`; VERIFICACION.md, M6b). **There is no
  formal specification of the AIR**, which SECURITY.md §3.1 declares the highest-priority gap
  (VERIFICACION.md, M6c).
- **Already said above:** no external audit, neither of Arqueo nor of its fork of winterfell 0.13.1
  (§5.1-§5.2; VERIFICACION.md, M10a); `xmss` 0.1.0-pre.0, a *pre-release* pinned with `=` and without KAT in
  the project (§5.5; G10b); LMS/HSS untested against HBS-STATE and no second independent subject
  (§4.9-§4.10).
- **No external anchor:** `doc/ANCLAJE_EXTERNO.md` is an empty placeholder; the design is only described
  in `AUDITORIA.md` §174 and in rows B10.6/B10.7 of `doc/CONFIANZA_RESIDUAL.md` (VERIFICACION.md, L7c).
- **The Zenodo DOIs** could not be checked (§References), and **the reproduction** is from a virtual
  environment, one run per day, not done by the author on their machine (§7.2).

---

## 9. Future work

- **ECST canonical encoding** **[PROPOSAL]**, with a domain in the chaining, an explicit version and the
  `compromiso` field in the wire specification. Arqueo already has a basis: the domain registry, the
  `REGISTRO` table of `crates/zk-ssl-hash/src/lib.rs` that `tools/check_dominios.py` watches since §286, and
  the frozen core of spec/NUCLEO.md with its KATs (RFC-0005, PROPOSED).
- **Several anchors** for the head key, beyond TOFU, and witnesses that are not the author.
- **Availability of the evidence:** decide which proofs are retained, by whom and for how long, so that
  level B of the protocol stops being Undetermined.
- **Mechanised verification of R(C,K)** and of the persist-before-sign protocol, in TLA+ or with an SMT
  solver, including the deletion and the reset to zero of the counter of §8.1. Neither of the two
  repositories has it (VERIFICACION.md, M6a).
- **A second independent subject** for HBS-STATE, and **LMS vectors**.
- **ECST-R agenda** **[PROPOSAL, not implemented in any repository]** (VERIFICACION.md, C10c), with the
  corrections from the verification: the zero knowledge of a proof does not imply confidentiality of the
  memory of whoever generates it, and a side channel on the payer's machine breaks authorship with valid
  proofs (P12c); a k-of-n threshold raises from 1 to k the keys that have to be compromised, and only with
  independent compromises does the probability fall as p^k, with nothing «exponential» (P13); a trusted
  execution environment moves trust to the manufacturer and to attestation, and its own state suffers the
  problem of [Memoir11; ROTE17]; an induced fault that makes an invalid transition be accepted is detected
  only afterwards, and only if $\Pi_i$ is published and a third party re-verifies it, and in the XMSS^MT
  signature a fault in an intermediate layer can give a signature that verifies and expose a second
  one-time signature (P12a; grafting-type faults were described against the SPHINCS framework in
  [Grafting18]); a state machine for key compromise (healthy, suspected, quarantined, revoked, migrated)
  does not exist, and key rotation remains open. **Pedersen commitments are excluded**: they are not
  post-quantum and contradict CONTRIBUTING.md (P14, C10a).
- **Standardisation.** If HBS-STATE is taken to the IETF, its natural destination is the PQUIP working
  group, which produced RFC 10033; not CFRG or LAMPS. There is no record that the Internet-Draft has been
  submitted. What is said about the process —PQUIP's charter covers operational guidance without new
  cryptographic mechanisms; in the IRTF stream, a draft adopted by the CFRG would be called
  `draft-irtf-cfrg-*`, it would be approved by the IRSG and the IESG would only do the conflict review—
  rests on the CFRG guide, copied from the IETF wiki, and on fragments of search results about the PQUIP
  charter, RFC 5742 and RFC 5743, not on reading those texts (VERIFICACION.md, P2, P3, P5c, P5d). The other
  corrections to the draft of the I-D are in block P of `VERIFICACION.md` (P2 to P8d).

---

## 10. Conclusion

No piece of ECST is new, and this report does not pretend otherwise. What it offers is a way of not
confusing them: hash chaining commits the history but does not validate it; a valid proof does not show
that the rules are complete; none of that establishes a unique history; and the death of a process has to
end in a state with a name. The two instantiations show the two faces. HBS-STATE contributes an executable
contract for reconciliation and nothing more. Arqueo instantiates most of the components and declares what
it lacks: proofs that are not kept, an anchor that is TOFU, no third party that recomputes the chain and no
audit tool. And its reception log shows that the same R(C,K) serves a second resource with another policy.
Contrasting the model with the code also turned up, by reading and not yet reproduced, a case that Arqueo's
policy does not seem to see (§8.1); that R cannot see it follows from its definition.

> **Commitment ≠ validity ≠ semantic conformance ≠ consensus: each one is checked separately, and what
> has not been measured is said.**

---

## Generative AI use statement

This report was drafted with the assistance of a generative AI assistant (Claude, by Anthropic, in a
Claude Code session), starting from drafts, also generated with AI, that the author provided. The
verification was done by agents of the same assistant —one verifier and one sceptic per block—; two
adversarial reviews of the first draft, also by agents, were applied afterwards
(`doc/ecst/borrador/revision-fidelidad.json` and `revision-rigor.json`); and the reproduction of §7.2 was
run in an ephemeral container, not on the author's machine. In accordance with GENAI.md, the project's
method is that measuring comes first, the author decides, and only what the author runs and accepts gets
in. **This draft has not yet gone through that acceptance.** The author is Ángel José Toranzo Portela, and
he is the only one; the assistant does not appear as author or as co-author.

What was checked against sources and what was not. Every figure and every claim about the code or the
state of the two repositories points to a file of the trees or to an entry of `VERIFICACION.md`. From the
literature, the metadata of each entry of `referencias.bib` were verified, with the source noted in the
file itself; the text was checked in RFC 10033, SP 800-208, EIP-8310 and the HBS-STATE v0.3
specification, of which there was a copy. The characterisations of the other works of §2 rest on their
titles and on abstracts or fragments of search results, not on reading the full text. What is said about
the IETF and IRTF process (§9) rests on the CFRG guide and on search fragments. The sketch of
Proposition 1 and the arguments of §3 are the text's own reasoning, not citations, and nobody has reviewed
them outside this process.

## Availability

- **This report:** `doc/ecst/ECST.md`; its first draft, unrevised, is in commit a7c72b9, and the
  two adversarial reviews that were applied to it, in 6a868ed.
- **Verification register:** `doc/ecst/VERIFICACION.md`; bibliography in BibTeX, with the metadata
  verified and the source of each entry: `doc/ecst/referencias.bib`.
- **Intermediate material:** `doc/ecst/borrador/` —the verdicts per block (`verificacion-*.json`,
  `refutacion-*.json`, `final.json`), `REPRODUCCION.md`, the two `simulate` logs, the replica of the
  `fsync` probe (`sonda-fsync.rs.txt`) and the two adversarial reviews (`revision-*.json`)—. It is
  working material: this report cites it only to locate the reproduction, the provenance of an
  inference of §8.2 and the reviews that were applied to it.
- **Trees:** Arqueo, `main` at 71c5aad (S582), with the verification material in commits 71710b2 to
  70f6370, the first draft of this report in a7c72b9 and its revisions in 6a868ed, which only add
  files under `doc/ecst/`
  (https://github.com/atoranzo/Arqueo-open-conservation-proofs-for-closed-ledgers);
  `hbs-state` at a960828, tag v0.2.0 (https://github.com/atoranzo/hbs-state).

---

## References

Only entries of `doc/ecst/referencias.bib`, whose metadata were verified on 2026-09-30 against the sources
that each one notes, plus the Zenodo DOIs of the projects themselves, which are marked. Each label in the
text points to the BibTeX key given at the end of its entry. The QRL blog article of §4.11 has no entry:
it is cited through `AUDITORIA.md` §288, because its metadata could not be verified.

- **[AlBassam21]** M. Al-Bassam, A. Sonnino, V. Buterin, I. Khoffi. «Fraud and Data Availability Proofs:
  Detecting Invalid Blocks in Light Clients». FC 2021, Part II, LNCS 12675, Springer, 2021, pp. 279–298. DOI
  10.1007/978-3-662-64331-0_15. BibTeX: `FC:ASBK21`.
- **[Ariadne16]** R. Strackx, F. Piessens. «Ariadne: A Minimal Approach to State Continuity». 25th USENIX
  Security Symposium, 2016, pp. 875–892. BibTeX: `USENIX:StrPie16`.
- **[BBHR19]** E. Ben-Sasson, I. Bentov, Y. Horesh, M. Riabzev. «Scalable Zero Knowledge with No Trusted
  Setup». CRYPTO 2019, Part III, LNCS 11694, Springer, 2019, pp. 701–732. DOI 10.1007/978-3-030-26954-8_23.
  BibTeX: `C:BBHR19`.
- **[BCCT13]** N. Bitansky, R. Canetti, A. Chiesa, E. Tromer. «Recursive Composition and Bootstrapping for
  SNARKs and Proof-Carrying Data». STOC 2013, ACM, pp. 111–120. DOI 10.1145/2488608.2488623. BibTeX:
  `STOC:BCCT13`.
- **[BH17]** L. Groot Bruinderink, A. Hülsing. «“Oops, I Did It Again” – Security of One-Time Signatures Under
  Two-Message Attacks». SAC 2017, LNCS 10719, Springer, 2018, pp. 299–322. DOI 10.1007/978-3-319-72565-9_15.
  ePrint 2016/1042. BibTeX: `SAC:BruHul17`.
- **[BMNW25]** B. Bünz, P. Mishra, W. Nguyen, W. Wang. «Accumulation Without Homomorphism». ITCS 2025, LIPIcs
  325, 2025, pp. 23:1–23:25. DOI 10.4230/LIPIcs.ITCS.2025.23. ePrint 2024/474. BibTeX: `ITCS:BMNW25`.
- **[Chidambaram13]** V. Chidambaram, T. S. Pillai, A. C. Arpaci-Dusseau, R. H. Arpaci-Dusseau. «Optimistic
  Crash Consistency». SOSP ’13, ACM, 2013, pp. 228–243. DOI 10.1145/2517349.2522726. BibTeX:
  `ChidambaramPAA13`.
- **[ChiesaTromer10]** A. Chiesa, E. Tromer. «Proof-Carrying Data and Hearsay Arguments from Signature Cards».
  Innovations in Computer Science (ICS 2010), Tsinghua University Press, 2010, pp. 310–331. BibTeX:
  `ITCS:ChiTro10`.
- **[CONIKS15]** M. S. Melara, A. Blankstein, J. Bonneau, E. W. Felten, M. J. Freedman. «CONIKS: Bringing Key
  Transparency to End Users». 24th USENIX Security Symposium, 2015, pp. 383–398. BibTeX: `USENIX:MBBFF15`.
- **[CW09]** S. A. Crosby, D. S. Wallach. «Efficient Data Structures for Tamper-Evident Logging». 18th USENIX
  Security Symposium, 2009, pp. 317–334. BibTeX: `USENIX:CroWal09`.
- **[DGHS16]** B. Dowling, F. Günther, U. Herath, D. Stebila. «Secure Logging Schemes and Certificate
  Transparency». ESORICS 2016, Part II, LNCS 9879, Springer, 2016, pp. 140–158. DOI
  10.1007/978-3-319-45741-3_8. ePrint 2016/452. BibTeX: `ESORICS:DGHS16`.
- **[EIP8310]** A. Shukla, B. Wagner, G. Singh, G. Ballet, J. Drake, K. Moroz Liebl, P. Ramanujam, S. Naiyer,
  T. Coratger, U. Leepaisalsuwanna. «EIP-8310: Post-Quantum Keystore for Stateful Keys». Ethereum Improvement
  Proposals, draft, created on 2026-06-19. BibTeX: `eip8310`.
- **[Elnozahy02]** E. N. Elnozahy, L. Alvisi, Y.-M. Wang, D. B. Johnson. «A Survey of Rollback-Recovery
  Protocols in Message-Passing Systems». ACM Computing Surveys 34(3), 2002, pp. 375–408. DOI
  10.1145/568522.568525. BibTeX: `ElnozahyAWJ02`.
- **[ETSI21]** ETSI. «CYBER; State management for stateful authentication mechanisms». ETSI TR 103 692 V1.1.1,
  November 2021. BibTeX: `etsi-tr-103-692`.
- **[FIPS205]** NIST. «Stateless Hash-Based Digital Signature Standard». FIPS 205, August 2024. DOI
  10.6028/NIST.FIPS.205. BibTeX: `fips205`.
- **[Fluhrer23]** S. Fluhrer. «Oops, I did it again revisited: another look at reusing one-time signatures».
  Cryptology ePrint Archive, Paper 2023/1905, 2023. BibTeX: `eprint-2023-1905`.
- **[Grafting18]** L. Castelnovi, A. Martinelli, T. Prest. «Grafting Trees: A Fault Attack Against the SPHINCS
  Framework». PQCrypto 2018, LNCS 10786, Springer, 2018, pp. 165–184. DOI 10.1007/978-3-319-79063-3_8. BibTeX:
  `PQCRYPTO:CasMarPre18`.
- **[HS91]** S. Haber, W. S. Stornetta. «How to Time-Stamp a Digital Document». Journal of Cryptology 3(2),
  1991, pp. 99–111. DOI 10.1007/BF00196791. BibTeX: `JC:HabSto91`.
- **[HyperNova24]** A. Kothapalli, S. T. V. Setty. «HyperNova: Recursive Arguments for Customizable Constraint
  Systems». CRYPTO 2024, Part X, LNCS 14929, Springer, 2024, pp. 345–379. DOI 10.1007/978-3-031-68403-6_11.
  BibTeX: `C:KotSet24`.
- **[Laurie14]** B. Laurie. «Certificate Transparency». Communications of the ACM 57(10), 2014, pp. 40–46. DOI
  10.1145/2659897. BibTeX: `Laurie14`.
- **[LedgerDB20]** X. Yang, Y. Zhang, S. Wang, B. Yu, F. Li, Y. Li, W. Yan. «LedgerDB: A Centralized Ledger
  Database for Universal Audit and Verification». PVLDB 13(12), 2020, pp. 3138–3151. DOI
  10.14778/3415478.3415540. BibTeX: `YangZWYLLY20`.
- **[McGrew16]** D. McGrew, P. Kampanakis, S. Fluhrer, S.-L. Gazdag, D. Butin, J. Buchmann. «State Management
  for Hash-Based Signatures». SSR 2016, LNCS 10074, Springer, 2016, pp. 244–260. DOI
  10.1007/978-3-319-49100-4_11. BibTeX: `McGrewKFGBB16`.
- **[Memoir11]** B. Parno, J. R. Lorch, J. R. Douceur, J. W. Mickens, J. M. McCune. «Memoir: Practical State
  Continuity for Protected Modules». IEEE S&P 2011, pp. 379–394. DOI 10.1109/SP.2011.38. BibTeX: `SP:PLDMM11`.
- **[Merkle89]** R. C. Merkle. «A Certified Digital Signature». CRYPTO ’89, LNCS 435, Springer, 1990, pp.
  218–238. DOI 10.1007/0-387-34805-0_21. BibTeX: `C:Merkle89a`.
- **[MS02]** D. Mazières, D. Shasha. «Building Secure File Systems out of Byzantine Storage». PODC 2002, ACM,
  pp. 108–117. DOI 10.1145/571825.571840. BibTeX: `MazieresS02`.
- **[Necula97]** G. C. Necula. «Proof-Carrying Code». POPL ’97, ACM, 1997, pp. 106–119. DOI
  10.1145/263699.263712. BibTeX: `Necula97`.
- **[Nimble23]** S. Angel, A. Basu, W. Cui, T. Jaeger, S. Lau, S. Setty, S. Singanamalla. «Nimble: Rollback
  Protection for Confidential Cloud Services». OSDI 23, USENIX, 2023, pp. 193–208. ePrint 2023/761. BibTeX:
  `AngelBCJLSS23`.
- **[Nova22]** A. Kothapalli, S. Setty, I. Tzialla. «Nova: Recursive Zero-Knowledge Arguments from Folding
  Schemes». CRYPTO 2022, Part IV, LNCS 13510, Springer, 2022, pp. 359–388. DOI 10.1007/978-3-031-15985-5_13.
  BibTeX: `C:KotSetTzi22`.
- **[PeerReview07]** A. Haeberlen, P. Kouznetsov, P. Druschel. «PeerReview: Practical Accountability for
  Distributed Systems». SOSP ’07, ACM, 2007, pp. 175–188. DOI 10.1145/1294261.1294279. BibTeX:
  `HaeberlenKD07`.
- **[Pillai14]** T. S. Pillai, V. Chidambaram, R. Alagappan, S. Al-Kiswany, A. C. Arpaci-Dusseau, R. H.
  Arpaci-Dusseau. «All File Systems Are Not Created Equal: On the Complexity of Crafting Crash-Consistent
  Applications». OSDI ’14, USENIX, 2014, pp. 433–448. BibTeX: `PillaiCAAAA14`.
- **[Piperine20]** J. Lee, K. Nikitin, S. T. V. Setty. «Replicated state machines without replicated
  execution». IEEE S&P 2020. DOI 10.1109/SP40000.2020.00068. ePrint 2020/195. BibTeX: `SP:LeeNikSet20`.
- **[POSIX24]** IEEE and The Open Group. IEEE Std 1003.1-2024 (POSIX.1-2024, Issue 8), System Interfaces,
  `fsync()`. June 2024. BibTeX: `posix2024`.
- **[RFC6962]** B. Laurie, A. Langley, E. Kasper. «Certificate Transparency». RFC 6962, June 2013. DOI
  10.17487/RFC6962. Experimental; obsoleted by RFC 9162. BibTeX: `rfc6962`.
- **[RFC8391]** A. Huelsing, D. Butin, S. Gazdag, J. Rijneveld, A. Mohaisen. «XMSS: eXtended Merkle Signature
  Scheme». RFC 8391, May 2018. DOI 10.17487/RFC8391. IRTF (CFRG), Informational. BibTeX: `rfc8391`.
- **[RFC8554]** D. McGrew, M. Curcio, S. Fluhrer. «Leighton-Micali Hash-Based Signatures». RFC 8554, April
  2019. DOI 10.17487/RFC8554. IRTF (CFRG), Informational. BibTeX: `rfc8554`.
- **[RFC9162]** B. Laurie, E. Messeri, R. Stradling. «Certificate Transparency Version 2.0». RFC 9162,
  December 2021. DOI 10.17487/RFC9162. Experimental. BibTeX: `rfc9162`.
- **[RFC10033]** T. Wiggers, K. Bashiri, S. Kölbl, J. Goodman, S. Kousidis. «Hash-Based Signatures: State and
  Backup Management». RFC 10033, September 2026. DOI 10.17487/RFC10033. IETF (PQUIP), Informational.
  BibTeX: `rfc10033`.
- **[Rogaway06]** P. Rogaway. «Formalizing Human Ignorance». VIETCRYPT 2006, LNCS 4341, Springer, 2006. DOI
  10.1007/11958239_14. ePrint 2006/281, «… Collision-Resistant Hashing without the Keys». BibTeX:
  `VIETCRYPT:Rogaway06`.
- **[ROTE17]** S. Matetic, M. Ahmed, K. Kostiainen, A. Dhar, D. Sommer, A. Gervais, A. Juels, S. Capkun.
  «ROTE: Rollback Protection for Trusted Execution». 26th USENIX Security Symposium, 2017, pp. 1289–1306.
  BibTeX: `USENIX:MAKDSG17`.
- **[SK99]** B. Schneier, J. Kelsey. «Secure Audit Logs to Support Computer Forensics». ACM Transactions on
  Information and System Security 2(2), 1999, pp. 159–176. DOI 10.1145/317087.317089. BibTeX: `SchneierK99`.
- **[SP800-208]** D. A. Cooper, D. C. Apon, Q. H. Dang, M. S. Davidson, M. J. Dworkin, C. A. Miller.
  «Recommendation for Stateful Hash-Based Signature Schemes». NIST SP 800-208, October 2020. DOI
  10.6028/NIST.SP.800-208. BibTeX: `nist-sp800-208`.
- **[STARK18]** E. Ben-Sasson, I. Bentov, Y. Horesh, M. Riabzev. «Scalable, transparent, and post-quantum
  secure computational integrity». Cryptology ePrint Archive, Paper 2018/046, 2018. BibTeX: `eprint-2018-046`.
- **[SUNDR04]** J. Li, M. N. Krohn, D. Mazières, D. Shasha. «Secure Untrusted Data Repository (SUNDR)». OSDI
  2004, USENIX, pp. 121–136. BibTeX: `LiKMS04`.
- **[Valiant08]** P. Valiant. «Incrementally Verifiable Computation or Proofs of Knowledge Imply Time/Space
  Efficiency». TCC 2008, LNCS 4948, Springer, 2008, pp. 1–18. DOI 10.1007/978-3-540-78524-8_1. BibTeX:
  `TCC:Valiant08`.
- **[VSM20]** S. Setty, S. Angel, J. Lee. «Verifiable state machines: Proofs that untrusted services operate
  correctly». Cryptology ePrint Archive, Report 2020/758, 2020. BibTeX: `EPRINT:SetAngLee20`.
- **[Zheng13]** M. Zheng, J. Tucek, F. Qin, M. Lillibridge. «Understanding the Robustness of SSDs under Power
  Fault». FAST 13, USENIX, 2013, pp. 271–284. BibTeX: `ZhengTQL13`.
- **[zkLedger18]** N. Narula, W. Vasquez, M. Virza. «zkLedger: Privacy-Preserving Auditing for Distributed
  Ledgers». NSDI 18, USENIX, 2018, pp. 65–80. ePrint 2018/241. BibTeX: `NarulaVV18`.

**Deposits of the projects themselves** (DOIs declared by the author; **not checked from the environment in
which this report was written**, because Zenodo was not reachable):

- **Arqueo:** preferred citation, 10.5281/zenodo.21736125 (CITATION.cff); the six deposits are listed in
  README.md, «Publicación» [Publication], and predate corrections to the tree.
- **`hbs-state`:** 10.5281/zenodo.22980547, which `CITATION.cff` declares and the repository calls the
  concept DOI. The 10.5281/zenodo.22993572 that the drafts cited could be the DOI of version 0.2.0; it has
  not been verified and it is not cited (VERIFICACION.md, H13a, B18a, B18b).

---

## Appendix A — ECST ↔ code correspondence

| model element | Arqueo | `hbs-state` |
|---|---|---|
| $C_{i-1}$, $C_i$, $O_i$ | roots and `seq`/`tag` of the entry, `crates/zk-ssl/src/log.rs` | — |
| $\Pi_i$ | STARK per transition, `crates/zk-ssl/src/client.rs`; circuits in `crates/stark-experiment/` | — |
| $A_i$ / $a_i$ | `crates/zk-ssl/src/mint.rs`, `recovery.rs`, `freeze.rs`, `governance.rs` | — |
| chaining $\mathrm{Hc}$ | `native_merge` (Rescue-Prime Rp64_256, Goldilocks), `crates/zk-ssl-hash/src/lib.rs` | — |
| evidence digest $\mathrm{Hp}$ | `digest_of_proof`, `crates/zk-ssl-hash/src/lib.rs:1290-1320` | — |
| $H_i$ | `chain_digest`, `chain_digest_v2`, `crates/zk-ssl/src/log.rs:250-294` | — |
| serialisation of the entry | `log_entry_to_bytes`, 137/169 B, `crates/zk-ssl/src/store.rs:411-455` | — |
| Link | `TransitionLog::verify_chain` and `verify`, `crates/zk-ssl/src/log.rs:393`, `:438`; `zkssl_verifyChain` | — |
| Verify | native verification in `apply_*`, `crates/zk-ssl/src/two_phase.rs`; envelopes of `crates/zk-ssl-verify/` | — |
| test of the retroactive binding | `t1_chain_retroactivo`, `crates/zk-ssl/src/log.rs:1189-1295` | — |
| authenticated head | `EpochHead`, `crates/zk-ssl/src/log.rs:586-649`; `epoch_digest_v6`, `crates/zk-ssl-hash/src/lib.rs:469`; `FirmanteCabeza::firmar`, `crates/zk-ssl-node/src/firma_cabeza.rs:179-203` | — |
| witnesses | reference witness, `crates/zk-ssl-cli/src/witness.rs` | — |
| history tree and extension | `crates/zk-ssl-verify/src/mmr.rs` | — |
| verifier without the node | `crates/zk-ssl-verify/src/main.rs` (codes 0 / 1 / 2 / 3) | `spec/verify-state.py` (conformance, not evidence) |
| guardian: reserve before use | `GuardianIndice::reservar`, `crates/zk-ssl-guardian/src/lib.rs` | `IndexGuard::reserve`, `src/lib.rs:482-489` |
| opening the counter | `GuardianIndice::abrir`, `crates/zk-ssl-guardian/src/lib.rs:419-444` | `IndexGuard::open`, `src/lib.rs:448-473` |
| `fsync` self-check | `comprobar_persistencia`, `crates/zk-ssl-guardian/src/lib.rs:510-543` | `check_persistence`, `src/lib.rs:523-556` |
| R(C,K) | `Reconciliacion`, `crates/zk-ssl-guardian/src/lib.rs:322-353` | `reconcile_values`, `src/lib.rs:412-431` |
| judge | `no_admite_matiz`, `crates/zk-ssl-guardian/src/lib.rs:369-376` | `is_fatal`, `src/lib.rs:370-377` |
| policy, signing index | `politica_de_reconciliacion`, `crates/zk-ssl-node/src/main.rs:592-652`; witness: `politica_del_cofirmante`, `crates/zk-ssl-cli/src/witness.rs:2696` | the owner's (spec §9) |
| policy, reception log | `politica_del_registro`, `crates/zk-ssl-node/src/main.rs:662-691`; `recepcion.rs`, `registro_recepcion.rs` | — |
| second record outside the pair | the node's journal, `crates/zk-ssl-node/src/diario.rs`; the witness's co-signatures | — |
| single use by set | `crates/zk-ssl/src/consumo.rs` (RFC-0006); pending leaf | — |
| single use by counter | XMSS index (guardian); `rx`, `crates/zk-ssl-node/src/registro_recepcion.rs` | OTS index (`IndexGuard`) |
| completeness | completeness envelope, `crates/zk-ssl-verify/src/main.rs`; `spec/rfc/0010-el-recibo-de-recepcion.md` | — |
| canonical encoding | `REGISTRO` table of `crates/zk-ssl-hash/src/lib.rs`; `tools/check_dominios.py`; spec/NUCLEO.md | — |
| vectors | catalogues of `spec/vectors/` and `tools/conformidad.sh` | `spec/state-vectors-v0.3.json` |

## Appendix B — Reproduction

The commands, as recorded by `doc/ecst/borrador/REPRODUCCION.md` (at the root of each tree):

```sh
# Arqueo, at 71c5aad
cargo test --release -p zk-ssl --lib t1_
cargo test --release -p zk-ssl-guardian
cargo build --release -p zk-ssl-verify
for c in paquete consumo conflicto rechazo edad pendiente pago prenda completitud; do
  bash tools/conformidad.sh target/release/zk-ssl-verify spec/vectors/$c/MANIFIESTO.txt
done
cargo run --release -p zk-ssl-cli -- simulate --amount 250000

# hbs-state, at a960828
cargo test --release
cargo build --release
python3 spec/verify-state.py --vectors spec/state-vectors-v0.3.json \
    --subject ./target/release/hbs-state-subject
```

`REPRODUCCION.md` writes the manifest as `[MANIFIESTO]`; here it is replaced by that of each catalogue,
`spec/vectors/<catálogo>/MANIFIESTO.txt`, which is how `tools/canon.sh` invokes it. The table of the
`fsync` self-check was obtained with the replica `doc/ecst/borrador/sonda-fsync.rs.txt`, compiled
separately and run 30 times per path and batch on the ext4 root and on `/dev/shm`.
