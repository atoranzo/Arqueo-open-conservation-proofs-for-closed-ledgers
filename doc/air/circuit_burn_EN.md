# AIR specification — `circuit_burn`

> **Español:** [`doc/air/circuit_burn.md`](./circuit_burn.md) is the original of this page, in
> Spanish; it prevails wherever the two differ. This translation is of commit `7766efb` (audit entry
> §917) and was produced by the project's assistant, a Claude Code session, in audit entry §918
> (`GENAI.md`): it is a generated artefact, not text written by the author. Line citations of the
> form `file:N` refer to the Spanish files.

**Status**: first of twenty-seven. Written on 01-08-2026 against the code in
`crates/stark-experiment/src/circuit_burn.rs`.

> It is the contract against which to audit this circuit. `SECURITY.md` §3.1
> calls it the project's highest-priority gap: **a soundness flaw
> is invisible counterfeit money**, and until this document there was nothing
> to check the code against except the code itself.

⚠️ **If this document and the code disagree, the code wins and this document
is wrong.** It is written by reading the code, not the other way round.

---

## 1. What it proves

> The holder of an account, **proving knowledge of its spend key**,
> withdraws `amount` from its balance and **reduces the total supply by the same
> amount**, without being frozen.

The proof convinces the verifier that there exists a witness —key, balance,
nonce, Merkle paths— consistent with the public roots before and
after.

## 2. Layout of the trace

**Width: 42 columns. Length: 512 rows.**

### 2.1 Columns

| range | content |
|---|---|
| `0..12` | **Rescue state, lane A** — the **old** leaf and its path |
| `12..24` | **Rescue state, lane B** (`LANE_B = 12`) — the **new** leaf |
| `24` | `COL_BIT` — direction bit of the accounts path |
| `25..29` | `COL_KEY` — **spend key**, four elements (§90) |
| `29..33` | `COL_ACC_ID` — public identity of the account |
| `33` | `COL_BAL` — balance before |
| `34` | `COL_BAL_NEW` — balance after |
| `35` | `COL_NONCE` |
| `36` | `COL_AMT` — amount burned |
| `37`, `38` | `COL_SUPPLY_OLD`, `COL_SUPPLY_NEW` |
| `39`, `40` | `COL_SBIT`, `COL_SACC` — binary decomposition of the range |
| `41` | `COL_FBIT` — bit of the frozen path |

⚠️ **Columns `0..24` have no named constant.** They are the two lanes
of the Rescue state, and **no sweep that looks for `COL_*` sees them**. §72 —a
soundness flaw— lived exactly there.

### 2.2 Rows

| stretch | what happens |
|---|---|
| `0..7` | Absorption of the leaf: capacity and identity |
| `7` | `ROW_LEAF_LINK` — link of the leaf to the path |
| `15` | `ROW_LEAF_DONE` — leaf computed |
| `15..271` | Climb up the accounts tree, 32 levels × 8 rows |
| `271` | `ROW_ROOT` — root reached; **the key enters** |
| `272..279` | Derivation of the identity from the key |
| `279` | `ROW_PK_DONE` — identity computed; **it is compared** |
| `280..471` | **Non-membership in the frozen tree** |
| `471` | `ROW_FROZEN_ROOT` |

⚠️ The stretch `280..471` **occupied rows that were free**. It was added because
a frozen account **could burn its money**: settlement checked
freezing and the burn did not.

### 2.3 The two lanes are NOT symmetric

**It is the least evident property of the circuit and it is not written anywhere
in the code.** It is deduced by crossing three blocks fifty
lines apart:

| | lane A (`0..12`) | lane B (`12..24`) |
|---|---|---|
| Leaf it absorbs | **old** (`COL_BAL`) | **new** (`COL_BAL_NEW`) |
| Climbs by | the same path, same siblings | idem |
| Produces | **old root** | **new root** |
| At `ROW_ROOT` it receives | the key, from `COL_KEY` | **the same key** |
| At `ROW_PK_DONE` | **it is compared with `COL_ACC_ID`** | ⚠️ **it is not compared** |

⚠️ **That final asymmetry is correct and has to be justified**: in the
identity phase the two lanes receive **the same input** —`C_KEY_INPUT` feeds
both from `COL_KEY`— and **the same capacity** —`C_CAP_A` and `C_CAP_B` set
them to zero at each link—, so **they compute the same digest**. Checking
one is enough because the other is identical **by construction**.

> **That justification did not exist before this document.** It was deduced
> by crossing lines 565-566, 606-608 and 615. It is exactly the kind of
> reasoning that §72 did not do, and §72 was a soundness flaw.

## 3. The 28 constraints

`NUM_CONSTRAINTS = C_FBIT_BOOL + 1`.

### 3.1 Hash and Merkle path

| slots | what it binds | on which rows |
|---|---|---|
| `C_HASH_A`, `C_HASH_B` (12+12) | The Rescue round in each lane | where `P_HASH_FLAG` = 1 |
| `C_CAP_A`, `C_CAP_B` (4+4) | Capacity at **zero** when linking | `P_LINK_MERKLE` |
| `C_PLACE_A`, `C_PLACE_B` (4+4) | Placement according to the direction bit | `P_LINK_PLACE` |
| `C_SIBLING` (4) | The sibling enters where it belongs | `P_LINK_PLACE` |
| `C_BIT_BOOL` (1) | `COL_BIT` ∈ {0,1} | all |
| `C_LEAF_CAP_A/B` (4+4) | Capacity of the leaf at zero | `P_LINK_LEAF` |
| `C_LEAF_DIG_A/B` (4+4) | Intermediate digest of the leaf | `P_LINK_LEAF` |

### 3.2 Input of the witness

| slots | what it binds |
|---|---|
| `C_NONCE` (2) | The nonce enters both leaves |
| `C_INPUT` (10) | **Lane A receives `COL_BAL`; lane B receives `COL_BAL_NEW`** — here the roles split |
| `C_KEY_INPUT` (8) | The **four** elements of the key, in **both** lanes |

### 3.3 The three that are the statement

| slots | what it binds |
|---|---|
| **`C_PK_CHECK`** (4) | **OWNERSHIP**: the derived identity = `COL_ACC_ID` |
| **`C_BALANCE`** (1) | `COL_BAL_NEW = COL_BAL − COL_AMT` |
| **`C_SUPPLY`** (1) | `COL_SUPPLY_NEW = COL_SUPPLY_OLD − COL_AMT` |

### 3.4 Constancy and range

| slots | what it binds |
|---|---|
| `C_TRANSPORT` (10) | 10 columns constant between rows: key (4), balances, nonce, amount, supplies |
| `C_ID_CONST` (4) | `COL_ACC_ID` constant |
| `C_SBIT_BOOL` (2), `C_FIRST_S` (2), `C_HORNER` (1), `C_SEG_LINK` | Binary decomposition of the amount |
| `C_FROZEN_*`, `C_FBIT_BOOL` | Non-membership in the frozen set |

---

## 4. ⚠️ What is NOT constrained

**This section is the reason this document exists.** Each `const C_X` of the
code says what it binds; **none says what it leaves free**, and what stays free is
where a soundness flaw lives.

### 4.1 Nothing binds `COL_AMT` to be positive

`C_BALANCE` requires `BAL_NEW = BAL − AMT` and `C_SUPPLY` the same for the
supply. **None requires `AMT > 0`.**

An `AMT = 0` produces an empty burn: valid, useless and **with no effect
on the state except the nonce**. It does not create money.

⚠️ **And a negative `AMT` is not representable**: the field is Goldilocks, there is no
sign. What is representable is a **huge** `AMT` that through modular
arithmetic «reduces» the balance by wrapping around — **and that is prevented by the
binary decomposition** (`C_SBIT_BOOL`, `C_HORNER`, `C_SEG_LINK`), which binds
`AMT` to fit in the declared range.

**Verified**: without that decomposition, the modular subtraction would allow creating
balance. With it, no.

### 4.2 Nothing binds lane B at `ROW_PK_DONE`

Already explained in §2.3: it is correct **because the two lanes compute the same**,
not because it is checked.

⚠️ **It is a conditional guarantee**: it depends on `C_KEY_INPUT` and `C_CAP_B`
continuing to feed lane B **exactly the same way** as lane A. If someone changes
one of the two and not the other, `C_PK_CHECK` **will not detect it**.

> **That is the flaw of §72 in potential, in this circuit, today.** It is not
> present; **it is one change away** and nothing warns.

### 4.3 Nothing binds `COL_NONCE` to increase

`C_NONCE` puts it into both leaves and `C_TRANSPORT` keeps it constant between
rows. **The old leaf and the new one use the same nonce.**

⚠️ **Consequence**: this circuit **does not protect against replay by
itself**. The chaining of roots does —a burn changes the root,
so repeating the proof presents a stale root—, and **that requires a total
order**, which a single node gives and a distributed system does not.

It is recorded in `SECURITY.md` §3.4 for the payment path; **it applies equally
here**.

### 4.4 Nothing binds columns `0..24` outside the rows with a selector

Between `ROW_PK_DONE` (279) and the frozen stretch, and after
`ROW_FROZEN_ROOT` (471) up to the end of the trace (512), **the Rescue
state is not constrained**: `P_HASH_FLAG` is zero and the links do not
fire.

**It is correct** —those rows do not take part in the statement— but it means that
a witness can put **anything** there. It does not affect validity because
nothing reads it afterwards.

⚠️ **If some future constraint read those rows, it would be immediately
exploitable.** It is written down so that whoever adds it knows.

### 4.5 The frozen path is not bound to the index — **and it does not need to be**

`COL_BIT` and `COL_FBIT` are **independent** direction bits, and no
constraint binds them to represent the same position. The layer passes the
correct path —`self.frozen.path_for(account_index)`— but **that is the layer**, and
§73 records what happens when a property is enforced by the layer and not by the
circuit.

✅ **Even so the circuit is sound, by an IMPLICIT guarantee**: the recomputed frozen
root is **asserted against the public input**
(`frozen_root`). A path from another position **does not rebuild the same root**,
so the proof does not verify.

⚠️ **But it is a guarantee by consequence, not by constraint** — the same
form as §4.2 — and it has a condition: **it depends on the siblings of the
two positions differing**.

In a **completely empty** frozen tree all paths are
identical, and presenting the one of another index works. **It does not matter: if nobody is
frozen, the check protects nothing.** The case that matters —a
frozen account presenting the path of a free neighbour— fails, because its leaf is
different and the root does not match.

> **The correctness of the statement matters.** The first version of this section
> said «nada lo verifica» [nothing verifies it] and left it open. The correct one is: **nothing
> verifies it explicitly, and the root verifies it implicitly.** It is
> `AUDITORIA.md` §99.5 applied to this document: a correct fact about the
> wrong object.

---

## 5. Negative witnesses

The ones that exist, and which attack each one covers:

| test | attack |
|---|---|
| `third_party_cannot_burn_someone_elses_money` | burn without the key |
| `burning_more_than_the_balance_is_rejected` | negative balance |
| `burning_without_updating_supply_is_rejected` | burn without reducing the supply |
| `deflating_supply_beyond_amount_is_rejected` | reduce the supply by too much |
| `a_frozen_account_cannot_burn` | burn while frozen |
| `a_forged_frozen_root_is_rejected` | forged frozen root |
| `wrong_new_root_is_rejected` | new root that does not correspond |
| `no_constraint_is_vacuous` | mutation: every constraint fires |

### ⚠️ The missing ones

| attack | state |
|---|---|
| **Frozen path of another index** (§4.5) | ⚠️ **no test, and closed by analysis**: the root prevents it |
| Lane B diverging from lane A in the identity phase (§4.2) | ❌ **no test** — not reachable today, but nothing pins it |
| `AMT = 0` | ❌ no test; **harmless**, but not declared |

---

## 6. What this document forced us to ask

- **The asymmetry of the lanes** (§2.3): it was in the code, spread over
  three blocks, and **it was not stated in any of them**.
- **That `C_PK_CHECK` is conditional** (§4.2): it protects **because** the lanes
  compute the same, not because it is checked. **One change away from
  §72.**
- **That the frozen path is not bound to the index** (§4.5): it looked like
  a leak and **it is an implicit guarantee** — the root closes it. Writing the
  question forced us to find why the circuit is sound, which was not
  stated anywhere either.

> None of the three is seen by reading the code from top to bottom. All three
> came out of asking, constraint by constraint: **«if this binds X, what
> stays free?»**
