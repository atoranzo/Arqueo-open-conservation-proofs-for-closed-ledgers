# The dossier for the external review of the hiding argument (§903)

> **Español:** [`doc/ocultacion-hvzk-dosier.md`](./ocultacion-hvzk-dosier.md) is the original of
> this page, in Spanish; it prevails wherever the two differ. This translation is of commit
> `7766efb` (audit entry §917) and was produced by the project's assistant, a Claude Code session,
> in audit entry §918 (`GENAI.md`): it is a generated artefact, not text written by the author. Line
> citations of the form `file:N` refer to the Spanish files.

**Status**: entry 136 of `BACKLOG.md`. The session wrote it in §903 for a reviewer who is not of
the house, and it is not a promise: what the house promises of its proofs is still the census
—«a proof that verifies demonstrates the transition it states and publishes literally no
constant-column value of its trace», RFC-0009 D-A and `SECURITY.md` §3.bis—, and what could be
promised is written in D-AM of RFC-0009 and awaits this review. The reviewer does not decide what
is promised: they say whether the argument holds, where it does not, and what the measurements do
not measure. The author decides, afterwards, with an entry.
⚠️ §916: the author accepted the promise for the twenty-three with `zkssl/0.5` (RFC-0009 D-A and
D-AM, §916) before this review, with H7 inside: the review is still requested, and what it says
enters through section 11 of the document and, if it touches the promise, through an entry of the
author (D-A of §916 is reversible for that reason).

## 1. What is asked to be reviewed, in one sentence

That, for sixteen of the twenty-three families of proofs with a row in the D-B table of RFC-0009
—the six with T = 1024, the nine with T = 512 and solvency—, with the construction of `zkssl/0.4`
as it is —T random rows per column, one random column, the quotient blinded with m = 64 and the
salted leaves—, the honest verifier's view is simulated without the witness: for every real trace
the AIR accepts, the view and the output of the simulator of section 5 of
`doc/ocultacion-hvzk.md` are statistically indistinguishable in the programmable random oracle
model, under the hypotheses of its section 4; and that for the other seven —MerkleAir,
FrozenClimbAir and the five with T = 64— it is not established, for what section 14 says of each.

⚠️ **§915:** since the `zkssl/0.5` cut (§914) what is asked to be reviewed is the proposition of
section 18 of `doc/ocultacion-hvzk.md`, for the twenty-three families with the production
construction —T ≥ 128, two random columns paired in DEEP as one over E, m = 64 and the salted
leaves—, with L9 (section 17) in the place of the measured R3 and without M_lin or the fine count;
what is above describes `zkssl/0.4`, which is still what it was for the vectors kept under
`spec/vectors/0.4/`. The reading order of section 2 holds, with sections 17 and 18 at the end; the
figures of section 4 are reproduced the same way, with `K_ENTERO_PAREADAS` and its row for T = 128.

## 2. What to read, and in what order

1. **What is decided and what is not**: `spec/rfc/0009-lo-que-revela-una-prueba.md`, D-A (the
   promise, reverted in §538), D-I (the salt; with its notes §896 to §902), D-AL (the DEEP mask in
   E, entry 133) and D-AM (what could be promised and what it lacks). Half an hour.
2. **The argument**: `doc/ocultacion-hvzk.md`, whole and in its order: the protocol as it is
   implemented (1), the verifier's view (2), the lemmas L0 to L6 (3), the hypotheses with their
   class (4), the simulator step by step (5), the condition R3 as an instrument (6), what it does
   not cover (7), the statement (8), what was measured in A1 and what was corrected (9), where the
   fork departs from note 2024/1037 (10), the objections of the session's two reviews with their
   closure (11), what it asks of A3, A4 and A5 (12), A3 measured (13) and entry 134 measured (14);
   ⚠️ §905 and §907: entry 135 measured with its lemma L1' (15) and the agent's review closed, with
   L7, L8 and the measured instances (16). The ⚠️ §n marks say what each passage said before and why
   it changed. Half a day.
3. **The code the argument describes**, with the document beside it: in the fork,
   `crates/winter-prover/src/lib.rs` (`generate_proof_oculto`: the rows, the column, the two coins
   and their seeds), `crates/winter-prover/src/constraints/composition_poly.rs` (`segmentar`: the
   chunks and the randomizers), `crates/winter-air/src/air/oculta.rs` (the wrapper: 2T rows, the
   exemptions), `crates/winter-air/src/marca.rs` (the mark), `crates/winter-verifier/src/lib.rs`
   (`verificar_oculta`: what the verifier rebuilds) and `crates/zk-ssl-air/src/sal.rs`
   (`MerkleConSal`, the salted leaves); and the instruments of `crates/zk-ssl/src/`:
   `instrumento_cota.rs` (A1: the bound and L1 per family), `instrumento_r3.rs` (A3: O, K, the
   masks and R3 per geometry), `instrumento_falsadores_hvzk.rs` (the three falsifiers of the toy,
   with the coin rebuilt from a real proof) and `instrumento_mlin.rs` (entry 134: the probe, M_lin
   and R3 per family). One day.
4. **The chronicle**: entries §896 to §902 of `AUDITORIA.md`: what each one measured, what it
   corrected of the previous one and the objections each review of the session raised. One hour.
   ⚠️ §907: and §905 (entry 135) and §907 (the agent's review), plus
   `doc/ocultacion-hvzk-revision-agente.md`, the full report of that review: what it read, its seven
   objections and what it took as correct with what. One more hour.

## 3. What is claimed, with its class and where it is measured

| claim | class | where it is measured or sustained |
|---|---|---|
| L0: the non-interactive transcript is simulated by programming the oracle | model (H7) | — |
| L1: the T random rows cover the 2q + 6 touched points per column, over F, in 18 families | measured in every canon | `instrumento_cota`: `INDEPENDIENTE_DE_LA_AIR`, `rango_de_los_tocados`, `rango_pleno`; the mutant T = 32 |
| L2: the m coefficients of each randomizer cover the q + 2 opened points per chunk, over E | measured in every canon | `instrumento_cota`: `rango_pleno` (Vandermonde); the mutant m = 43 |
| L3: what FRI opens of DEEP is simulable if the witness-independent masks cover O|_K (R3) | proved as a sufficient condition; R3 measured | `instrumento_r3`: `R3_MEDIDO` (false with the column and the randomizers), `R3_MEDIDO_LABORATORIO` (true with two columns); `instrumento_mlin`: `MLIN_MEDIDO` (true with M_lin in sixteen families) |
| H12: the random rows of a column that enters linearly with a public coefficient are one more mask | measured in every canon (the census) and per family; ⚠️ §907: in every canon only the census of the 23 and the rank of solvency; the rank of the other fifteen was measured in the long test and is guarded by the fingerprint of each composition, `HUELLA_C` (O3 of the agent's review); and proved as a lemma, L7 | `instrumento_mlin`: `CENSO_134`, the probe; the toy reproduces what is pinned without linear ones and adds exactly what is free with them |
| H14 (§907): R3 with M_lin holds on the real challenges | proved on layer 0 with T ≥ 512 outside the coset–touched coincidences (L8, failure ≤ 7c₀/\|F\|); measured in the rest on three instances per family, one adversarial | `instrumento_mlin`: `adversa`, `MLIN_INSTANCIAS`; solvency in every canon, the fifteen in the long test |
| L4: the salted leaves hide what is not opened | proved with the fresh salt (H8, assumed) | `crates/zk-ssl-air/src/sal.rs`, tests of size and effect; freshness is not measured |
| L5: the masks come from coins of their own, seeded apart from the public one | proved by the code; H9 (no repeated seed), assumed | `generate_proof_oculto`, `segmentar`; the falsifier of the repeated seed |
| L6: what depends on the AIR with T = 64 | stated, not proved | the distinguisher of the toy with T = 64 |
| H3: z outside F and H, no coincidences of conjugates | assumed, with bounded failure | not measured |

⚠️ §915: with the construction of the cut, L1 is measured in the 23 (not in 18), L3 stays as a
condition proved by L9 for any positions (`K_ENTERO_PAREADAS`, the five production geometries), H12
and H14 are guard instruments the proposition does not need, and L6 is not used; the classes of L0,
L2, L4, L5 and H3 do not change.

## 4. How to reproduce each figure

Everything in release and with `--locked`; in debug winterfell requires the declared degrees to be
the real ones and the test of the families does not run. The times are those of the session's
container, with four CPUs.

```bash
cargo test -p zk-ssl --release --locked instrumento_cota -- --nocapture
#   ~150 s: las 28 pruebas reales de los 23 probadores; la cota por columna y por trozo; L1 por
#   familia contra INDEPENDIENTE_DE_LA_AIR; R3 por geometría contra R3_MEDIDO y
#   R3_MEDIDO_LABORATORIO; el censo de la 134 contra CENSO_134; la solvencia contra MLIN_MEDIDO.
cargo test -p zk-ssl --release --locked instrumento_r3 -- --nocapture
#   4 s: el juguete (T = 128): 424 sin determinar, 254 con una columna, 424 con dos.
cargo test -p zk-ssl --release --locked instrumento_mlin -- --nocapture
#   20 s: la sonda en el juguete de las tres entradas; sin lineales 424/254/80, con las dos
#   lineales 156 y 330; la tabla del censo, coherente con lo clavado.
cargo test -p zk-ssl --release --locked instrumento_falsadores_hvzk -- --nocapture
#   los tres falsadores del juguete, con la moneda rehecha de los bytes de cada prueba:
#   T = 32 recupera la constante y T = 64 no; T = 64 distingue el testigo y T = 128 no; la semilla
#   repetida devuelve las dos constantes y la fresca no.
cargo test -p zk-ssl --release --locked mide_m_lin_en_las_23_familias -- --ignored --nocapture
#   ~45 min: la sonda en las 23 con dos semillas y M_lin en las dieciséis, contra MLIN_MEDIDO;
#   MLIN_SOLO=AuditAir,BandaAir mide sólo esas.
cargo test -p stark-experiment --release --locked falsadores_oculta -- --nocapture
#   los diez falsadores de la ocultación del núcleo; el décimo, el laboratorio de dos columnas.
bash tools/canon.sh --sello
#   ~10 min: todo lo anterior salvo el test largo, con sus pines, y el resto de la casa.
```

Every pinned figure is in a constant with its name, and changing it is red: `R3_MEDIDO`,
`R3_MEDIDO_LABORATORIO` (`instrumento_r3.rs`), `CENSO_134`, `MLIN_MEDIDO` (`instrumento_mlin.rs`),
`INDEPENDIENTE_DE_LA_AIR` (`instrumento_cota.rs`). The challenges of the instruments are chosen by
the test from a coin seeded with a number (seed 1 in what is pinned; seed 2 and other queries in
the seal's runs for T = 64); the falsifiers of the toy rebuild the real coin.

## 5. Where to attack

What the session could not check by itself, or where the edge is finest:

1. **The count of L1.** The argument conditions each column on its 2q + 6 touched points,
   although the verifier sees only q + 4 of it and, of the others, sums over all the columns
   (C(x_j) and C(z·g)). It is sufficient. Is it necessary? If it is not, the fine count (entry 135
   of `BACKLOG.md`) could cover the five families with T = 64 without more rows, and a new lemma
   is needed.
2. **K and what is public of D.** L3 takes as public D at the q positions and at
   x₀ = (z + z·g)/2 and nothing else. Is there anything more of D that the verifier determines
   from what it sees?
3. **The joint kernel of entry 134.** M_lin enters the quotient chunks, and the instrument
   discounts what the verifier sees of each chunk —the q positions, z and z·g— with the joint
   kernel with the randomizers. Is that exactly what `verificar_oculta` sees of the chunks, no
   more and no less?
4. **The replica of the composition.** `Sonda::composicion` computes C as the prover's evaluator
   does; it was validated against real proofs in the toy without periodic columns (§898, `vista`)
   and by structure in entry 134 (C(P_ρ) fits in the N chunks). A validation against the OOD frame
   of a real proof of an AIR with periodic columns would be stronger, and it is not done.
5. **The probe.** «Public coefficient» is constant, periodic or a random element of the auxiliary
   segment. The AIRs are built with empty public inputs, except age; is there any constraint
   whose form depends on the public inputs in a way the probe does not see? (Age changes the
   degree of a constraint with `todos`; it was probed only with `todos`, the statement of its two
   witness proofs.)
6. **The test's challenges against the proof's.** The ranks are measured with positions, z and
   coefficients chosen by the test; the argument is that the rank is generic (Schwartz-Zippel
   over E). Are there real challenges with a smaller rank, and with what probability? ⚠️ §907: the
   agent's review attacked here (its O1): the real positions are not generic —coset collisions in
   more than half of the proofs of solvency, coset–touched coincidences with frequency Θ(1)—; and
   «over E» was an error: the map is F-linear and the bound goes over |F|. What there is now: L8 for
   layer 0 with T ≥ 512, three measured instances per family —one adversarial— and the count of the
   advantage redone (section 16 of the document). What is still open: a bound for layer 1, the
   coincidences and T = 256, which today are only measured.
7. **The salt (L4).** With R3 green, is the salt needed? What would fail without it, leaf by leaf:
   the trace row at a point of Λ (L1), the vector of the chunks (L2) and the eight values of a FRI
   coset (only R3). The author decided to keep it (A5, §902).
8. **Note 2024/1037.** Section 10 compares with what the house knew of it through RFC-0009 and
   its abstract; the note could not be read from the session. A section-by-section comparison is
   pending.
9. **The fork.** This dossier is about the argument; the other half of H7 —neither the fork nor
   the construction is audited— is the reading of the fork's code against winterfell 0.13.1,
   which the house has not commissioned.

Outside the dossier, and said: the random rows of the auxiliary segment (only age has one, with
T = 64); the five families with T = 64, which need entry 131 or 135; MerkleAir and FrozenClimbAir,
which need entry 133; the level against a quantum adversary, which is not measured
(`SECURITY.md` §3.11).

## 6. How what they say is recorded

Each objection goes to section 11 of `doc/ocultacion-hvzk.md` with its closure —or without it, if
it has none—, like those of the session's two reviews; an entry of `AUDITORIA.md` says who
reviewed, what they objected and what changed; and afterwards the author decides on D-AM, with
his entry. The house corrects in place and marks what changes: a reviewer who finds an error will
not see the error erased, but corrected with a note that says what it said. ⚠️ §907: that is how the
first review from outside the session was recorded, that of an agent (section 8 of this dossier):
its whole report in `doc/ocultacion-hvzk-revision-agente.md`, its seven objections with their
closure in section 11 of the document, what it asked to write and measure in section 16, and entry
§907.

## 7. What this dossier is not

It is not a promise nor does it change any. It is not an audit of the fork. It measures nothing:
it says where each measurement is and how to do it again. And it does not replace the reviewer:
the house does not choose them here.

## 8. The agent's review (§907), and what changed with it

Before the author chooses the reviewer, the session asked an agent external to it for a review
—another instance, without the chronicle or the conclusions, with this dossier and a copy of the
tree of §903 in front of it, read-only—. It does not count as the external review of entry 136, and
it serves so that the human reviewer does not spend their time on what has already been seen: the
full report is `doc/ocultacion-hvzk-revision-agente.md`. It found no false step nor a measurement
that measures something else; it raised seven objections, none high: one instance of challenges per
family and non-generic positions (O1), the lemma of the joint kernel unwritten (O2), the rank of
fifteen families without a guard in the canon (O3), the count of the advantage incomplete (O4), two
imprecisions of the view (O5), the grinding of the `concurrent` feature (O6) and the panic of the
DEEP degree assert (O7). §907 closed them —L7 and L8 written, three instances measured per family
with an adversarial one, the fingerprint of each composition pinned, the count redone, the texts
corrected— except what belongs to the fork or is still without a bound, and it says so: layer 1, the
coset–touched coincidences and T = 256 are measured and not bounded; the panic is entry 137 of
`BACKLOG.md`; and a correction the review did not see: the bound over the challenges goes over |F| =
2^64, not over |E|, because the random column lives in F. What the human reviewer will find,
therefore, is an argument with a third review on top of it and with its gaps named; section 5 still
holds, with item 6 rewritten.

## 9. The letter to the reviewer, for the author

The choice of the reviewer is the author's (entry 136). A draft letter, in English, for when he
makes it; he puts in the names. The natural choice is the authors of ePrint note 2024/1037, whose
construction the fork's is; second, whoever maintains winterfell, for the other half of H7; third,
any cryptographer with STARKs and FRI who accepts a public review with their objections recorded.

> Subject: Request for an external review of a zero-knowledge argument for a winterfell 0.13.1
> fork built on ePrint 2024/1037
>
> I maintain Arqueo, an open-source settlement layer whose proofs are STARKs produced by a fork of
> winterfell 0.13.1. Since its zkssl/0.4 cut the fork hides the witness with the construction of
> the note "A note on adding zero-knowledge to STARKs" (ePrint 2024/1037): T random rows behind each
> trace column, one random column, a blinded quotient with randomizers of degree < m, and salted
> Merkle leaves. The repository, its reasoning and every measurement are public.
>
> I am writing to ask whether you would review the hiding argument written for that
> implementation. It is a simulator-based argument (honest-verifier, statistical in the
> programmable random-oracle model), with every hypothesis classified as measured, proven,
> modelled or assumed, and each measured hypothesis tied to a test that re-measures it on every
> build. Two things found while measuring may interest you as readers of the construction:
>
> 1. With the random column living in the base field F while the DEEP polynomial lives in the
>    quadratic extension E, the column masks only one of the two E-coordinates of each value FRI
>    opens; together with the quotient randomizers it leaves what FRI opens of DEEP uncovered in
>    every production geometry (a rank condition we call R3). Two random columns, each with its
>    own DEEP coefficient, cover it in all geometries; so does one random column over E.
> 2. For AIRs whose columns enter the constraints linearly with public coefficients, the random
>    rows of those columns are an additional witness-independent linear mask that, in 16 of our
>    23 proof families, covers the gap with the current construction. Two families (hash-only
>    AIRs) have no such column, and five families with T = 64 rows are not covered by the base
>    lemma at all. A layer-0 coverage lemma holds for any query positions when T ≥ 512; what
>    depends on the positions beyond it is measured on several instances, not bounded.
>
> The review package is a single document, `doc/ocultacion-hvzk-dosier.md`: what to read and in
> which order (about two days), each claim with its class and where it is measured, the commands
> that reproduce every figure, the places where we think the argument is thinnest, and a first
> review by an automated reader whose objections are recorded with their resolution. We would
> record your objections verbatim, with their resolution or without one, in the document and in
> the project's audit log, under your name if you consent, and the project's author decides
> afterwards what, if anything, is promised to users. Nothing is promised today: the only claim
> the project makes about hiding is a measured census of literal values.
>
> Please let me know whether this is something you would be willing to do, under what conditions,
> and in what time frame. I would also welcome a pointer to someone you consider better placed.
