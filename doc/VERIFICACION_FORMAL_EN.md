# Formal verification of the layouts — triage and design

> **Español:** [`doc/VERIFICACION_FORMAL.md`](./VERIFICACION_FORMAL.md) is the original of this
> page, in Spanish; it prevails wherever the two differ. This translation is of commit `7766efb`
> (audit entry §917) and was produced by the project's assistant, a Claude Code session, in audit
> entry §918 (`GENAI.md`): it is a generated artefact, not text written by the author. Line
> citations of the form `file:N` refer to the Spanish files.

*(AUDITORIA §183; born of an external critique received on 2026-08-05)*

## 0. The verdict, in three layers

**The critique is true at its core.** The suite (297+242 green)
proves POINTS: valid witnesses verify; the corrupt witnesses that
occurred to us bounce. It does not prove the universal: that NO phantom witness
satisfies the constraints for false publics. Under-constrained circuits
are the dominant class of ZK vulnerability in the real world
(the Zcash 2018 counterfeiting bug is the archetype). Our §69
(«Qué NO demuestra este documento» [what this document does NOT show]) demands that this
residue have a name of its own; since §183 it has one, and SECURITY §2 lists it.

**The critique's map was not our territory.** There is no R1CS here, nor
Plonkish selectors: it is AIR/winterfell, Goldilocks field, degree-7 sbox.
The off-the-shelf tools (Circomspect, Ecne, Picus) live in the
circom/R1CS world; for AIR nothing equivalent exists — if it is done, it is done
in-house and in our dialect, like everything else. `circuit_settlement.rs`
no longer exists (museum, §175-§176). And the guardian
(`tools/check_constraint_layout.py`) ALREADY covers the syntactic half of this
class: sound slots, periodic columns read-and-built, seeded mutation
cases — and it was born of real bugs (the three dead slots of the
mint_climb extraction).

**The honest gap**: the guardian audits SLOTS; it does not audit CELLS. That
every cell of the trace (column × row class) has an owner is the exact
syntactic form of under-constraint, and it is computable at our
scale. That is FV-1.

## 1. FV-1 — The cell census (extension of the guardian)

**Property verified** (syntactic): for each circuit, for each
column `c` of its trace and each row class `K`, at least one of the following exists:

1. a transition constraint that **references** `next[c]` (or the
   lane-offset equivalent) under a selector active in `K`;
2. an assertion (`Assertion::single(c, fila, _)`) with `fila ∈ K`;
3. a **free-cell declaration**: witness by design.

**Row classes**, derived from the circuit's own periodic columns
(the guardian already parses them): hash rows (`hash_flag=1`), each
link row (one-hot columns: `acct_link`, `link_leaf`, `link_salt`…),
`first_row`, `first_s`/`cont_s`/`seg_link` per segment, and the rest
(«flat rows», where only the constraints without a selector apply).

**Free-cell declaration** — a convention the guardian parses,
next to the circuit's constants:

```rust
// CELDAS_LIBRES: receptor y salt son testigo (fila 0, cols 4..12) — §178
```

An UNDECLARED free cell = a guardian failure («celda sin dueño» [ownerless cell]).
A declared one that also has an owner = a warning (stale declaration).

**Seeded mutation case** (the tradition of CASO_66): a copy of
`circuit_refund` with one `C_CAP` line deleted must make the
guardian shout «celda sin dueño: col 0..4 en clase enlace» [ownerless cell: col 0..4 in class
enlace]. If the mutation passes in silence, the census is broken.

**What FV-1 does NOT claim, in its own output**: *referenced ≠
determined*. A cell can appear in a constraint and still have
degrees of freedom (algebraic cancellations, multiplicative selectors
at zero). FV-1 catches the empirically dominant class — the cell NOBODY
looks at — not semantic determination. It complements the discriminating tests;
it does not replace them. The output will say so every time it runs.

**Gate**: it is added to the existing ones. Format:
`28 circuitos · N celdas-clase · 0 sin dueño · K libres declaradas`.

## 2. FV-2 — SMT spike, bounded and with permission to fail

*(EXECUTED in §190: q2 unsat · q1 intractable for memory, ON RECORD.)*

**Scope**: ONE circuit, the smallest (`circuit_refund`: 20
constraints, 12 assertions, trace 16×12). An exporter in `tools/` that
emits the system (transitions with selectors evaluated per row +
assertions) in SMT2 over `FiniteField` (cvc5 `--ff`, Goldilocks prime).

**Questions, in order of merit**:
1. **Determination**: with the publics `(P, amount)` and the free
   cells of row 0 fixed, is the rest of the trace uniquely determined?
2. **Consistency of the chain**: does an assignment exist that satisfies the
   constraints without following the Rescue rounds? (If the answer is yes,
   there is a phantom; if the solver does not finish, there is a record.)

**Honest expectation, declared in advance**: degree 7 over a 64-bit
prime may be intractable for the solver even at this scale. A
«TIMEOUT with these parameters» result is a valid deliverable and is
recorded as such — the spike buys KNOWLEDGE of the cost, it does not promise a
green. If it turns out tractable on the small one, the question of scaling to
`circuit_send` (51 columns) is opened with data; if not, it is closed on record.

## 3. FV-3 — The horizon, named without promising it

Lean4/Coq (formalising the specification and proving the soundness of the AIR) or
K-Framework (executable semantics of the layer's transitions) are
multi-year programmes that also require formalising Rescue, FRI and the
semantics of winterfell so that the theorem speaks of the real system and not of
a model. They are declared as a direction — the same treatment as quantum
money in README — not as debt. Whoever arrives with hands and years, here
is the map.

## 4. Specification precision (so as not to verify the wrong theorem)

The property «∀ w₁,w₂ valid ⇒ same nullifier» is FALSE as a universal:
distinct users legitimately produce distinct nullifiers. The
correct property is **functional determination**: the nullifier is a function
of (key, domain) with no extra degrees of freedom in the witness; likewise the
commitment with respect to (receiver, salt, amount). FV-1 and FV-2 aim at
that form. Formally verifying the wrong specification is the other
way FV fails, and it is documented here so that it does not happen.

## 6. FV-1: the prototype, EXECUTED — and the frontier, MEASURED

`doc/fv/censo_celdas_prototipo.py` implements the approach on
`circuit_refund` and **was executed**. The result comes in two halves, both
valuable:

**The approach works**: it resolves the constants, parses the transitions
(`next[...]`) and the assertions (`Assertion::single`), and crosses both against
the free-cell convention. On `circuit_refund` it reports the 12
columns covered, with no orphan cells.

**The frontier is MEASURED, not assumed**: the mutation case —deleting
`C_CAP`— **is NOT told apart** in the prototype's simplification of classes.
The reason is exactly the one foreseen: without an interpreter that computes «row
class K = the rows where selector S equals 1», the aggregate class
«enlace» [link] does not isolate the rows where `C_CAP` was the only owner; the column
is still referenced by `C_HASH` in the «hash» class, and the aggregate census does not
see the gap.

**What this decides for entry 71**: real FV-1 is NOT a graft onto the
guardian. It is, at its core, **the periodic-selector interpreter** —
reading `get_periodic_column_values` row by row and deriving, per selector, the
exact set of rows it activates. With that, «cell × class» becomes
computable and the mutant is caught. The prototype turns the estimate of
§183 into a proven requirement: **the work is the interpreter, not the crossing.**

**Why it is NOT in `tools/` as a guardian**: a tool that does not
tell apart its own mutant watches nothing, and the guardian's header
says so —a sweep that approves what it does not understand is worse than none
(§42.5)—. It lives in `doc/fv/` as executable evidence of the frontier.

## 7. FV-1: the frontier, CROSSED — the interpreter catches the mutant

`doc/fv/interprete_selectores.py` implements the selector interpreter
that §6 measured as missing, and **catches the mutation case** the prototype did not
tell apart. On `circuit_refund`: sound circuit, zero orphan cells;
mutant with `C_CAP` deleted, **four ownerless cells** —the capacities
`(0,1,2,3)` in the «enlace» row class— exactly the ones that had to be
reborn at zero in the link row and that nothing now governs.

**How it crosses the frontier**: it derives the row classes from the REAL selector, not
from a label —it reads `hash_flag = [1×NUM_ROUNDS, 0]` over the cycle and
obtains class «hash» = rows 0..6, class «enlace» = row 7— and places each
constraint by its selector (`hash_flag` → hash, `1−hash_flag` → enlace).

**The finding that unblocked it** (§185): an assertion lives in ONE row,
hence in ONE class. The prototype treated `Assertion::single(c, 0, _)` as
coverage of column `c` in EVERY class, and that was the leak: the
capacities are asserted to zero in row 0 (class «hash»), but
`C_CAP` governs them in row 7 (class «enlace») — distinct rows,
distinct classes. Covering «every class» with an assertion of row 0 erases
precisely the distinction that catches under-constraint. Corrected —the
assertion covers `(columna, clase-de-su-fila)`— the mutant jumps out.

**What remains of entry 71**: generalising the interpreter to the 28
circuits —row classes of multi-cycle selectors (`with_cycles`,
one-hot tree links, segments), dual lanes (`offset`/
`LANE_B`), and assertions on non-zero rows— and then grafting it onto the
guardian as a gate. The hard core —deriving the row class from the
selector and catching the gap— is **proven on the simple circuit**. The
rest is coverage, not concept.

## 8. FV-1: two lanes — and the alias false positive

`doc/fv/interprete_dos_carriles.py` takes the interpreter to the first two-lane
circuit, `circuit_frozen_climb` (45 constraints, Merkle
climb). Four new pieces on top of the one-lane one: a lane loop
`for (lane, offset) in [(0,0),(1,LANE_B)]`, raw indices (`result[24+i]`,
`result[44]`), a boolean full-row selector (`current[COL_BIT]*...`,
class «todas» [all]), and **alias tracking**. The criterion of §185 is met:
sound circuit, zero orphans; mutant (`result[24+i]` deleted) caught —
four capacities of lane A ownerless in «enlace».

**The finding, the most important of the series**: in the first version, the
SOUND circuit reported `(24, enlace)` as orphaned —`COL_BIT`, the direction
bit—. **It was not under-constraint: it was the interpreter's blindness.**
`COL_BIT` is read once as `let bit = next[COL_BIT];` and that `bit` enters
the placement constraints under `link_flag`; the parser looked for
`next[COL_BIT]` INSIDE each `result[...]` and did not follow the alias. A
syntactic verifier that does not follow intermediate variables **shouts
"vulnerability" over sound code** —the worst failure of a security
tool, because it destroys its credibility (§137: census all the
representations; §42.5: do not condemn what is not understood)—. Fixed by
harvesting `let X = next[...]` and propagating it; the sound one is left clean and the
mutant still jumps out.

**State of entry 71**: the interpreter covers one lane and two lanes, with the
four pieces and alias tracking. What remains: multi-cycle selectors
(`with_cycles` with one-hot tree periodic columns and range segments — the
account circuits), and the final graft onto the guardian as a gate.
The concept is proven on the two topologies; the rest is still
coverage, now with a trap —the alias— already known and solved.

## 9. The map of what is missing — three layers, for entry 71's own session

Written on 05-08-2026 after proving the concept on the two simple topologies
(§185/§186). It pins down what remains precisely, read from the tree, so that
whoever takes up 71 again starts with a plan and not with improvisation. **It corrects an
imprecision of earlier handovers**: the rest is NOT «only coverage».
Layer 1 is a third selector pattern that the current interpreter cannot
read — concept work, not volume work.

### Layer 1 — the MULTI-CYCLE interpreter (the concept work that remains)

*(EXECUTED in §188: doc/fv/interprete_multiciclo.py.)*

The interpreter of §185/§186 derives the row class from a `hash_flag` of
**short cycle**: `[1×NUM_ROUNDS, 0]` over `CYCLE_LENGTH=8`, which repeats.
The ACCOUNT circuits (family `circuit_mint_climb`, `credit_climb`,
`recovery_climb`, `mint`, `send`, `claim`…) build their periodic columns in
another way —columns of **trace length** (`vec![zero; TRACE_LENGTH]`,
512 rows) filled by loops—, and they are **four pattern families**, not
one:

1. **multi-cycle hash_flag** (`mint_climb` :lines of `get_periodic…`):
   `vec![zero; TRACE_LENGTH]`, and `hash_flag[r]=1` only if
   `r % CYCLE_LENGTH < NUM_ROUNDS` **and** `r <= ROW_ACCT_ROOT`. The
   cycle condition is the same, but bounded to the hash region of the trace (the
   accounts climb), with zero outside it. The interpreter must evaluate
   `r % CYCLE_LENGTH` over 0..TRACE_LENGTH, not assume a cycle that
   repeats to the end.

2. **tree link one-hots** (`acct_link`, `link_leaf`, `link_salt`):
   columns with ONE 1 in a specific row (`ROW_LEAF_LINK`,
   `ROW_SALT_LINK`, `ROW_LEAF_DONE`) or one 1 per tree level
   (`acct_link[(CYC_ACC + level)*CYCLE_LENGTH + 7] = 1` for each `level`).
   Each one defines a «point» or «one-per-level» row class. The
   interpreter must read those built indices —the `ROW_*` resolved and the
   `level` loop— to know which rows each selector activates.

3. **row-0 selector** (`sel[0]=1`): `first_row`. A class of a single
   row. Trivial once (2) is in place.

4. **range segments** (`first_s`, `cont_s`, `seg_link`): for each
   of `NUM_SEGMENTS` segments of `SEGMENT_LENGTH=64` rows, `first_s` marks
   the first row of the segment, `cont_s` the first `SEGMENT_LENGTH-1`, and
   `seg_link` the second-to-last (`(seg+1)*SEGMENT_LENGTH - 2`). They define the classes
   «first-of-segment», «segment-body» and «segment-close» — the
   Horner machinery of the binary decomposition. It is the pattern most
   foreign to the current cycle model.

**Declared risk**: the «class = cycle selector pattern» architecture
may need rethinking into «class = explicit set of rows
computed column by column» to absorb (2) and (4). That can be a
long link of the chain, with reversions —of the weight of step 2 of B13/B14 (§139, three
`git checkout`)—. **That is why it is a session of its own with a fresh census**, not an end of the
working day.

**Success criterion, invariant since §185**: on `circuit_mint_climb`,
sound zero orphans AND a mutant caught (natural candidate: deleting a
`C_SEG_LINK`, which must leave its accumulator column ownerless in the
segment-close class). Real subject: `circuit_mint_climb.rs`, its
512-row periodic columns. The two-lane interpreter is read IN FULL before
extending it (the alias trap, §186, is already solved there and must not be
reintroduced).

### Layer 2 — the graft onto the guardian (turns demonstration into protection)

*(EXECUTED in §189: the census is a gate of tools/check_constraint_layout.py and the MUERTA
[dead] slots are GRAVE [severe].)*

Today the three interpreters live in `doc/fv/` as EVIDENCE; none is a
gate. Grafting the generalised interpreter onto
`tools/check_constraint_layout.py` —read in full, its 751 lines, before
touching it— makes it an active guardian: one more line in the gate of the
rite (`28 circuitos · N celdas-clase · 0 sin dueño`), with its mutation case
seeded in the style of the CASO_66/CASO_50 the guardian already carries. Only
then does FV-1 PROTECT instead of demonstrating. It requires Layer 1 to be
done —grafting an interpreter that only covers two topologies would leave the 26
remaining circuits in false green, the sin of §59.2—.

### Layer 3 — FV-2 and FV-3 (intact, behind the graft)

*(FV-2 EXECUTED in §190, on record; FV-3 is still horizon.)*

**FV-2** (entry 72): the SMT spike on `circuit_refund`, full script
in §2 of this doc; «intractable, on record» is a valid deliverable. It is done after
the graft, not before: the cell census is the cheap line that always
runs; the SMT is the expensive, one-off probe.

**FV-3**: Lean/K, declared horizon (§4), the quantum-money treatment.

### One-line summary

Missing: the multi-cycle interpreter (concept, a session of its own, four families
of selector broken down above) → the graft onto the guardian (protection) →
FV-2 (SMT probe) → FV-3 (horizon). The concept is proven on two
topologies; the third is the one that can still bend the architecture.

## 5. Status

FV-0 executed (§183 + SECURITY §2). **FV-1: concept SOLVED on the two
topologies** — prototype with a measured frontier (§184/§6), one-lane interpreter
catching its mutant (§185/§7), two-lane interpreter with
alias tracking (§186/§8); three executable pieces of evidence in
`doc/fv/`, two capital findings recorded (the assertion has a class;
the alias false positive). **What remains of entry 71**: multi-cycle selectors
(the trace-length periodic columns with tree one-hots and segments —
the account circuits) and the graft onto the guardian as a gate —
a session of its own, coverage over a proven concept. FV-2: after the graft.
FV-3: horizon.

## 10. FV-1 in the kit (`zk-ssl-air`)

The kit's five AIRs —band, pending claim, payment in flight, pledge and
age— have a census:
`doc/fv/interprete_{banda,cobro_pendiente,pago_en_curso,prenda,edad}.py`,
on the same core (`censo_nucleo.py`), and the guardian
(`tools/check_constraint_layout.py`) runs them as a gate together with the
six of `stark-experiment`. Before that, neither the slot sweep nor the
census entered the crate: they are the AIRs that judge the proofs of the milestones
and the ones the second implementation transcribes from the `.rs`, so a cell
without an owner in the `.rs` would have passed in both.

In the two roots, a gap of the sweep (a write it does not know how to
expand) is GRAVE [severe], not a warning: both are at 0 gaps, and a write in
a form the sweep does not expand —a reversed loop, an alias of a
slice of `result`, `result.iter_mut()`— came out as a gap with exit 0. And the
sweep's self-test runs before each sweep.

**What the core learns for the kit** (`ESPEC["KIT"]`; without that key the
core does what it always did, and the parity of the six earlier censuses was
measured byte for byte):

- the round through `ronda_rescue(actual, siguiente, ark1, ark2, bandera, ..)`:
  its owners are the twelve cells of the lane's next row, under its
  flag;
- the **inline** selectors (`periodic[P_HASH_B]` inside the statement).
  One that has no mask is an error, not «no selector»: taking it as having no
  selector would claim the cell in every class;
- the loops over lists and tuples (the two lanes) and the tables
  `const TRANSPORTE: [usize; N]`;
- the harvest **per summand**: in `s[C_CICLO] - c[C_CICLO] - enlace` the
  `enlace` is subtracted, it does not select, and the cell is left bound in every
  row.

**The age AIR extends the doctrine of §1, and says so.** It is cyclic (each cycle
of 8 rows is a node) and it has an auxiliary segment. Row 0 of each cycle, which
is the one after the link, is bound by constraints of the CURRENT row (`f0 *
c[..]`) and by the multiset reads. That is why, only in that row:

4. a reference to the current row in transition `r+1`, **under a
   selector** active in all the successor classes of that of `r` (the classes
   of the rows `r+1`), is an owner of the cell of row `r+1`;
5. the owners of the auxiliary segment (reads in `f0`, writes in `f6`) are
   declared in the ESPEC and anchored in the lines of
   `evaluate_aux_transition`.

Both conditions of rule 4 were measured. **Without a selector there is no
owner**: the booleanity `c[x] * (c[x] - 1)` references the cell in every
row, but does not determine it (the «referenced ≠ determined» of §1).
Counting it masked the constancy of vivo [alive] within the cycle: changing `hf` to
`enlace` in `hf * (s[C_VIVO] - vivo)` left the sweep and the five censuses
green (measured), and vacuity does not see it either, because the constraint still
reacts at the link. With the strict rule, that mutant leaves col
42 without an owner, and it is the third mutant of the age census («mutantes
3/3» [mutants 3/3]). Qa, qs
and the bits outside row 0 of the cycle are declared free: only the
booleanity binds them. What that mutant opens is knocked down by a discriminating test,
`un_vivo_que_se_apaga_en_la_fila_6_no_verifica`. **In all the
successors, not in one**: the `hash` class of the age AIR (rows 1..5) has as
successors `hash` and `f6+hash`, and a selector active only in f6 approved the
cell of rows 2..5, which it does not look at (bias to green, §59.2; with the
right rule the result does not change).

The constraints of the auxiliary segment are looked at by the mutation test of
`stark-experiment` (`buscar_vacias_aux`). Their cells (`X_H`, `X_S`) are looked at
by no net.

**What it found.** No ownerless cell that is not a witness by design.
The declared ones (`// CELDAS_LIBRES:` in each AIR) are the dead lanes
after the root, lane B idle before its cycle, the path bit outside
the climb, the rests of the range accumulator, the salt of the leaf of the
band and, in the age AIR, the witnesses of each position outside row 0 of its
cycle (qa, qs and the bits, only boolean). And one already known, which is not by
design: the three high limbs of the rate in the link of the band's nonce,
the slack of §489, open, not exploitable against a signed root, and which
the census sees again.

**What it does not close.** Neither the sweep, nor the census, nor vacuity look at the
second implementation (`tools/segunda/airs.py`). A transcription
error in it —an empty constraint in Python— would let the
second one accept what the kit rejects, and no honest vector would see it. The
fundamental fix is a differential KAT: evaluate the kit's `evaluate_transition` and
`evaluate_aux_transition` on pseudo-random frames with a fixed
seed, and have the second compare `transicion` and `transicion_aux` element by
element (agreeing at random points carries the guards of the `.rs` over to the
`.py`). It remains pending.
