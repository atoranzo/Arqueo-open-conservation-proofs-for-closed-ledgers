## 9. The map of what is missing — three layers, for entry 71's own session

> **Español:** [`doc/fv/mapa_fv_capas.md`](./mapa_fv_capas.md) is the original of this page, in
> Spanish; it prevails wherever the two differ. This translation is of commit `7766efb` (audit entry
> §917) and was produced by the project's assistant, a Claude Code session, in audit entry §918
> (`GENAI.md`): it is a generated artefact, not text written by the author. Line citations of the
> form `file:N` refer to the Spanish files.

Written on 05-08-2026 after proving the concept on the two simple topologies
(§185/§186). It fixes what remains with precision, read from the tree, so that
whoever resumes entry 71 starts with a plan and not with improvisation. **It corrects an
imprecision of earlier handovers**: the rest is NOT «only coverage». Layer 1
is a third selector pattern that the current interpreter cannot
read — concept work, not volume work.

### Layer 1 — the MULTI-CYCLE interpreter (the concept work that remains)

*(EXECUTED in §188: doc/fv/interprete_multiciclo.py.)*

The interpreter of §185/§186 derives the row class from a **short-cycle**
`hash_flag`: `[1×NUM_ROUNDS, 0]` over `CYCLE_LENGTH=8`, which repeats.
The ACCOUNTS circuits (family `circuit_mint_climb`, `credit_climb`,
`recovery_climb`, `mint`, `send`, `claim`…) build their periodic columns in
another way —**trace-length** columns (`vec![zero; TRACE_LENGTH]`,
512 rows) filled by loops—, and they are **four pattern families**, not
one:

1. **multi-cycle hash_flag** (`mint_climb` :lines of `get_periodic…`):
   `vec![zero; TRACE_LENGTH]`, and `hash_flag[r]=1` only if
   `r % CYCLE_LENGTH < NUM_ROUNDS` **and** `r <= ROW_ACCT_ROOT`. The cycle
   condition is the same, but bounded to the hash region of the trace (the
   accounts climb), with zero outside. The interpreter must evaluate
   `r % CYCLE_LENGTH` over 0..TRACE_LENGTH, not assume a cycle that
   repeats up to the end.

2. **tree-link one-hots** (`acct_link`, `link_leaf`, `link_salt`):
   columns with ONE 1 in a specific row (`ROW_LEAF_LINK`,
   `ROW_SALT_LINK`, `ROW_LEAF_DONE`) or one 1 per level of the tree
   (`acct_link[(CYC_ACC + level)*CYCLE_LENGTH + 7] = 1` for each `level`).
   Each one defines a «point» or «one-per-level» row class. The
   interpreter must read those constructed indices —`ROW_*` resolved and the
   `level` loop— to know which rows each selector activates.

3. **row-0 selector** (`sel[0]=1`): `first_row`. A single-row
   class. Trivial once (2) is in place.

4. **range segments** (`first_s`, `cont_s`, `seg_link`): for each of
   the `NUM_SEGMENTS` segments of `SEGMENT_LENGTH=64` rows, `first_s` marks
   the first row of the segment, `cont_s` the first `SEGMENT_LENGTH-1`, and
   `seg_link` the second to last (`(seg+1)*SEGMENT_LENGTH - 2`). They define the classes
   «first-of-segment», «body-of-segment» and «close-of-segment» — the
   Horner machinery of the binary decomposition. It is the pattern most
   foreign to the current cycle model.

**Declared risk**: the architecture «class = cycle selector pattern»
may need to be rethought as «class = explicit set of rows
computed column by column» to absorb (2) and (4). That can be a
long link with reversions —of the weight of step 2 of B13/B14 (§139, three
`git checkout`)—. **That is why it is its own session with a fresh census**, not the end
of a working day.

**Success criterion, invariant since §185**: on `circuit_mint_climb`,
healthy zero orphans AND one mutant caught (natural candidate: deleting a
`C_SEG_LINK`, which must leave its accumulator column without an owner in the
close-of-segment class). Real subject: `circuit_mint_climb.rs`, its
512-row periodic columns. The two-lane interpreter is read WHOLE before
extending it (the alias trap, §186, is already solved there and must not be
reintroduced).

### Layer 2 — the graft into the guardian (turns demonstration into protection)

*(EXECUTED in §189: the census is a gate of tools/check_constraint_layout.py and the MUERTAS
[dead ones] are GRAVES [serious].)*

Today the three interpreters live in `doc/fv/` as EVIDENCE; none is a
gate. Grafting the generalised interpreter into
`tools/check_constraint_layout.py` —read whole, its 751 lines, before
touching it— makes it an active guardian: one more line in the gate of the rite
(`28 circuitos · N celdas-clase · 0 sin dueño`), with its mutation case
seeded in the style of the CASO_66/CASO_50 that the guardian already carries. Only
then does FV-1 PROTECT instead of demonstrate. It requires Layer 1 to be
done —grafting an interpreter that only covers two topologies would leave the 26
remaining circuits in false green, the sin of §59.2—.

### Layer 3 — FV-2 and FV-3 (intact, behind the graft)

*(FV-2 EXECUTED in §190, with a record; FV-3 is still horizon.)*

**FV-2** (entry 72): the SMT spike on `circuit_refund`, full script
in §2 of this doc; «intractable, with a record» is a valid deliverable. It is done after
the graft, not before: the cell census is the cheap line that always
runs; the SMT is the expensive, one-off probe.

**FV-3**: Lean/K, declared horizon (§4), the treatment of quantum money.

### One-line summary

Missing: the multi-cycle interpreter (concept, its own session, four selector
families broken down above) → the graft into the guardian (protection) →
FV-2 (SMT probe) → FV-3 (horizon). The concept is proven on two
topologies; the third is the one that can still bend the architecture.
