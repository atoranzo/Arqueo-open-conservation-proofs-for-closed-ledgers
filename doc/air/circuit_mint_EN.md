# AIR specification — `circuit_mint`

> **Español:** [`doc/air/circuit_mint.md`](./circuit_mint.md) is the original of this page, in
> Spanish; it prevails wherever the two differ. This translation is of commit `7766efb` (audit entry
> §917) and was produced by the project's assistant, a Claude Code session, in audit entry §918
> (`GENAI.md`): it is a generated artefact, not text written by the author. Line citations of the
> form `file:N` refer to the Spanish files.

**Status**: second written. 01-08-2026, against
`crates/stark-experiment/src/circuit_mint.rs`.

⚠️ **Written by the circuit's author. It inherits his blind spots and is NOT a
contract.** See `AUDITORIA.md` §105.3: a specification signed by the
same hand as the code checks nothing; an auditor must audit the
code, not this document.

⚠️ **If this document and the code disagree, the code wins.**

---

## 1. What it proves

> **Two distinct custodians** of the authorised set, proving knowledge of
> their keys, **create `amount` in an existing account** and raise the
> total supply by the same amount, **without exceeding the cap**.

It is the most expensive statement of the system: **this is where money is created**.

## 2. Layout

**45 columns, 512 rows.**

| range | content |
|---|---|
| `0..12`, `12..24` | Rescue state, lanes A and B (`LANE_B = 12`) |
| `24`, `25` | `COL_BIT_A/B` — bits of the **custodians'** path |
| `26`, `27` | `COL_KEY_A/B` — keys of the two custodians |
| `28`, `29` | `COL_IDX_A/B` — their declared indices |
| `30`, `31` | `COL_ACC_A/B` — **accumulators** of the index from the path |
| `32..36` | `COL_ACC_ID` — identity of the receiving account |
| `36`, `37` | `COL_BAL`, `COL_BAL_NEW` |
| `38`, `39` | `COL_NONCE`, `COL_AMT` |
| `40`, `41` | `COL_SUPPLY_OLD`, `COL_SUPPLY_NEW` |
| `42` | `COL_MAX_SUPPLY` — **the cap** |
| `43`, `44` | `COL_SBIT`, `COL_SACC` — binary decomposition |

| row | what happens |
|---|---|
| `0..15` | Leaf of the account: lane A the **old** one, lane B the **new** one |
| `15..271` | Climb up the accounts tree |
| `271` | `ROW_ACCT_ROOT` — old and new roots |
| `272..311` | **Derivation of the two custodian identities** and climb up their tree |
| `311` | `ROW_CUST_ROOT` — root of the custodian set |

⚠️ **The two lanes have different roles in each phase**: in the first they are
old leaf / new leaf; in the second they are **custodian A / custodian B**. The
same machinery serving two purposes, and **that is not stated in the
code**.

## 3. The constraints that are the statement

| slots | what it binds |
|---|---|
| `C_INPUT` (10) | Lane A receives `COL_BAL`; lane B receives `COL_BAL_NEW` |
| `C_CUST_INPUT` (2) | At `ROW_ACCT_ROOT` `COL_KEY_A` and `COL_KEY_B` enter |
| **`C_ACC`** (2) | The accumulator adds up the bits of each custodian's path |
| **`C_ACC_FINAL`** (2) | **The accumulated index = the declared index** |
| **`C_BALANCE`** (1) | `BAL_NEW = BAL + AMT` |
| **`C_SUPPLY`** (1) | `SUPPLY_NEW = SUPPLY_OLD + AMT` |
| `C_TRANSPORT` (11) | 11 constant columns, **`COL_MAX_SUPPLY` included** |
| `C_SEG_LINK` (8) | Binary decomposition of **eight** values |

### 3.1 The eight segments, which are half the specification

`expected[]` fixes what must fit in range:

| # | value | what it proves |
|---|---|---|
| 0-3 | `BAL`, `AMT`, `BAL_NEW`, `SUPPLY_NEW` | they fit in the domain's range |
| **4** | `MAX_SUPPLY − SUPPLY_NEW` | ⚠️ **the cap is not exceeded** |
| 5-6 | `IDX_A`, `IDX_B` | indices within the set |
| **7** | `IDX_B − IDX_A − 1` | ⚠️ **`IDX_B > IDX_A`** |

**A single technique serving two different properties** —the monetary cap
and the order of the custodians—, and in neither of the two cases is there a
constraint that says it directly.

---

## 4. ⚠️ What is NOT constrained

### 4.1 Nothing compares the custodian indices directly — ✅ **checked**

**There is no constraint that says «`IDX_A ≠ IDX_B`».** The property is
given by **three separate things**:

1. `C_ACC` accumulates the bits of each custodian's Merkle path.
2. `C_ACC_FINAL` requires that accumulated value to **equal the declared index** — the
   path determines the index, one cannot lie.
3. **Segment 7** requires that `IDX_B − IDX_A − 1` fit in range. With
   equal indices it is `−1`, which is `p−1` in Goldilocks, **and it does not fit**.

✅ **MEASURED** — `one_custodian_cannot_sign_twice`:

```
Err("verificacion fallo: InconsistentOodConstraintEvaluations")
```

The proof **was generated, was verified, and the verifier rejected it**. The defence
is in a constraint, **not in the trace builder** — which is the
distinction that it took §73 to learn.

> ⚠️ **None of the three gives the property alone.** It is the project's **third guarantee
> by consequence**, after the two of `circuit_burn`. And all three
> share a fragility: **they depend on the other pieces not changing, and nothing
> warns if they change.**

### 4.2 Nothing binds a custodian's key to *its* path — **and it does not need to**

`C_CUST_INPUT` puts `COL_KEY_A` into lane A and `COL_KEY_B` into lane B. Nothing says
that `path_a` is the path **of** `key_a`.

✅ **The root closes it**: the identity derived from the key climbs the given
path, and the result must equal `custodian_set_root`, which is a public
input. **A key with someone else's path does not reach the root.**

Same form as the frozen path of `circuit_burn` §4.5.

### 4.3 Nothing prevents issuing to a nonexistent account — **closed by `root_old`**

If the old leaf were invented, an account with a balance out of nothing would be created.
It is prevented by that leaf having to **climb up to `root_old`**, which is a public input: an
account that is not in the tree does not rebuild the published root.

### 4.4 ⚠️ Nothing binds `AMT > 0`

As in `circuit_burn` §4.1: an issuance of zero is valid and useless.
**It does not create money.**

### 4.5 ⚠️ The cap is checked on `SUPPLY_NEW`, and that is correct **because
`MAX_SUPPLY` is constant**

Segment 4 bounds `MAX_SUPPLY − SUPPLY_NEW`. If `COL_MAX_SUPPLY` could
vary between rows, a witness would raise it in the row where it is checked.

✅ **It is in `C_TRANSPORT` (11 columns) and in the assertions**, bound to the
public input. **It cannot be forged.**

⚠️ It was verified explicitly because **the transport list has 11
slots and at first sight only 10 columns were counted**. The eleventh was
precisely `COL_MAX_SUPPLY`. A mismatch there would have been a flaw of the
class of §74 —money created beyond the cap—.

### 4.6 Nothing prevents the two custodians from being the **same human**

The circuit proves that **two distinct indices of the set** authorised.
It cannot prove that there are two people behind them.

**It is a limitation of the model, not of the circuit**, and it belongs to the governance
of the custodian set. It is stated because a specification that does not
distinguish «two keys» from «two parties» invites reading more than there is.

---

## 5. Negative witnesses

| test | attack |
|---|---|
| **`one_custodian_cannot_sign_twice`** | ⚠️ **one custodian signing twice** — new, §107 |
| `no_constraint_is_vacuous` | mutation: every constraint fires |
| *(the others of the module)* | keys outside the set, unbalanced supply, cap exceeded |

### What is missing

| attack | state |
|---|---|
| A custodian's key with another's path (§4.2) | ❌ no test; **closed by analysis** —the root— |
| `AMT = 0` | ❌ no test; harmless |
| `MAX_SUPPLY` varying between rows (§4.5) | ❌ no test; **closed by analysis** — it is in the transport |

---

## 6. What forced us to ask

- **§4.1**: §80 stated the risk of the 2-of-N and **nobody had tested it**.
  Now it has a test, and the defence turned out to be in the binary decomposition,
  not where it was being looked for.
- **§4.5**: counting the slots of `C_TRANSPORT` and seeing that they did not match
  the columns in sight.
- **§4.6**: the distinction between «two keys» and «two parties», which the statement
  of the circuit cannot cover.

⚠️ **And two bad measurements along the way** (`AUDITORIA.md` §107.2, §107.3):
a test **vacuous by construction** —`prove()` does not fail in release— that would have
announced a nonexistent flaw, and a suspicion **right for the wrong
reason**.
