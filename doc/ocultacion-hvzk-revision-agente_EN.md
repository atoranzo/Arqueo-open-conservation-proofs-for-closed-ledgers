# The agent's review of the hiding argument (§907)

> **Español:** [`doc/ocultacion-hvzk-revision-agente.md`](./ocultacion-hvzk-revision-agente.md) is
> the original of this page, in Spanish; it prevails wherever the two differ. This translation is of
> commit `7766efb` (audit entry §917) and was produced by the project's assistant, a Claude Code
> session, in audit entry §918 (`GENAI.md`): it is a generated artefact, not text written by the
> author. Line citations of the form `file:N` refer to the Spanish files.

> **What this file is.** The report, whole and untouched, of the first review of the argument of
> `doc/ocultacion-hvzk.md` made from outside the session: an agent —another instance of Claude Code,
> without the session's chronicle or conclusions— that was given the dossier
> (`doc/ocultacion-hvzk-dosier.md`) and a copy of the tree of §903 (`git archive`), read-only: it
> ran nothing and touched no file. The session asked for it in §903 and §907 of `AUDITORIA.md`
> records it. **It is not the external review of entry 136 of `BACKLOG.md`**, which is by a person
> the author chooses: it is the house's third review, further from the session than the two of §897
> and §898. What it says is recorded here as it is, with what it got right and what it got wrong;
> its closure is in section 11 of `doc/ocultacion-hvzk.md` (the review of §907) and what it asked to
> write and measure, in its section 16. Two things the reader should know when reading it: the
> review predates §905 —it speaks of entry 135 by count, without its measurement, and the
> measurement partly refuted it— and its O4 takes as good a bound «over |E|» that §907 corrects to
> «over |F|».

# External review of the hiding argument (HVZK) of `zkssl/0.4` — entry 136

Reviewer: an agent external to the project, read-only over the copy `scratchpad/revision_copia`
(paths relative to it). Nothing was run; no file was touched. The unforked winterfell sources
(`winter-fri` and `winter-crypto` 0.13.1) were read from the container's cargo registry, because the
argument depends on what FRI opens and on how the coin derives the nonce and the positions, and the
fork does not carry them.

## 1. What I read, in what order, and how much

In the order of section 2 of the dossier:

1. `doc/ocultacion-hvzk-dosier.md`, whole (150 lines).
2. `spec/rfc/0009-lo-que-revela-una-prueba.md`: header and D-A (lines 1–100), D-I with its notes
   §896–§902 (296–378), D-AL and D-AM (863–975). Some 300 lines; I did not read D-B to D-AK nor
   «Seguridad» [Security].
3. `doc/ocultacion-hvzk.md`, whole (947 lines), sections 1 to 14, with their ⚠️ marks.
4. The fork, with the document alongside:
   - `crates/winter-prover/src/lib.rs` whole (`generate_proof` and `generate_proof_oculto`),
     `constraints/composition_poly.rs` whole (`segmentar`), `domain.rs`, `composer/mod.rs` (the
     prover's DEEP), `constraints/evaluator/default.rs`, `boundary.rs`, `periodic_table.rs`,
     `evaluation_table.rs` (`combine`), `channel.rs` (positions and grinding);
   - `crates/winter-air/src/air/oculta.rs` and `marca.rs` whole; `divisor.rs`; `transition/mod.rs`
     (`combine_evaluations`); `context.rs` (`num_constraint_composition_columns`); `air/mod.rs`
     (coefficients, periodic ones, boundaries); `proof/mod.rs` (the `Proof` structure) and
     `proof/ood_frame.rs` whole;
   - `crates/winter-verifier/src/lib.rs`, `channel.rs`, `composer.rs` and `evaluator.rs` whole;
   - `crates/zk-ssl-air/src/sal.rs`, the implementation (lines 1–330; not its tests);
   - `crates/stark-experiment/src/lib.rs` (`ocultacion_encendida`), `crates/zk-ssl/src/lib.rs`
     (`proof_options`), and the `Cargo.toml` files (the `concurrent` feature).
5. Unforked upstream: `winter-fri-0.13.1` (`prover/mod.rs`, `prover/channel.rs`, `verifier/mod.rs`,
   `folding/mod.rs`, `utils.rs`, `options.rs`) and `winter-crypto-0.13.1` (`random/default.rs`).
6. The instruments, whole: `crates/zk-ssl/src/instrumento_cota.rs` (936), `instrumento_r3.rs` (558),
   `instrumento_mlin.rs` (1262) and `instrumento_falsadores_hvzk.rs` (594).
7. The AIRs: `crates/stark-experiment/src/solvency.rs` (the whole AIR, as it is the tightest
   family), and over the sixteen families of D-AM a sweep of `new`, `evaluate_transition` and
   `get_assertions` (branches on public inputs, periodic assertions, auxiliary segment).
8. `BACKLOG.md`: entries 131 and 135.

I did not read: the chronicle of `AUDITORIA.md` §896–§902 (section 11 of the document summarizes the
session's two reviews and that was enough for me), note 2024/1037 (it is not in the tree), the full
code of the other fifteen AIRs, `SECURITY.md`, the tests of `sal.rs`.

## 2. Objections

No objection of high severity: I did not find a step of the argument that is false nor a measurement
that measures something other than what it says. What follows are gaps between what is measured and
what is stated, and a lemma that the instrument applies and the document does not write.

### O1 (medium). R3 with M_lin is measured on ONE instance of challenges per family, and the real positions are not «generic» in the sense the document needs

Passages. `doc/ocultacion-hvzk.md`, section 6 (line 489): «over z, cc and α_i, a drop in rank is a
polynomial condition (a vanishing minor) and its probability is bounded by the degree over |E|; over
the positions it is not polynomial, and the instrument has to try several choices and say so».
Section 14, «Lo que no hace» [What it does not do] (line 945): «It does not measure the challenges
of any real proof: the geometry and the options are those of one proof of each family, and the
challenges are chosen by the test, as in §898». Section 8 / D-AM: «statistically indistinguishable
in the programmable random oracle model, with advantage less than 2^{−60} plus Q·2^{−256}».

What the code does. The measurement of entry 134 runs with seed 1 and nothing else: in the seal,
`instrumento_cota.rs:846` (`instrumento_mlin::mide(&*sonda, &g, &censo.lineales(), 1, …)`, only
solvency) and in the long test `instrumento_mlin.rs:1175`
(`mide(&*s, &g, &censo.lineales(), 1, LARGO)`, `#[ignore]`, line 1136). The runs with seed 2 and
with q = 40 and 41 (`instrumento_cota.rs:768`) are only of the geometry T = 64 and only with the two
masks of §898, not with M_lin. Section 6 (and 13) says of the R3 of §898 «with two challenge seeds
that give the same ranks» (line 494), but the second seed of T ≥ 256 «is not in the tree».

What the document already declares, it declares; what I add:

1. The «generic» positions are not the typical case. The 42 positions fall in |Λ|/8 folding cosets:
   1024 with T = 256, 2048 with T = 512, 4096 with T = 1024. The probability that two of the 42
   share a coset in layer 0 is ≈ 1 − exp(−861/1024) ≈ 57% for solvency, ≈ 34% with T = 512 and ≈ 19%
   with T = 1024; in layer 1 (128, 256 and 512 cosets) the coincidences are almost certain. The
   pinned instance has 42 distinct cosets in layer 0 (the «rank of O» of the table of section 13,
   7·42 + L/8, says so). That is: most real proofs of solvency have an O different from the measured
   one.
2. There is an event that depends on the positions, of probability Θ(1) per proof, in which the
   structure of M_lin changes: a point of the folding coset of one position that coincides with a
   TOUCHED point of another, x_j·μ = g·x_k (μ^8 = 1). Since g = ω^16 and μ ∈ ⟨ω^{2L}⟩, it happens
   when p_j − p_k ≡ 16 − 2L·t (mod 16L); the expected number of pairs is 8·42·41/(16L): ≈ 1.7 with
   T = 256, ≈ 0.8 with T = 512, ≈ 0.4 with T = 1024. At that point the DIRECT path of every linear
   column is zero (P_ρ = Z_real·V_toc·w vanishes on the touched points), and the second coordinate
   of D(x) can only be covered by the quotient path or the randomizer. In solvency the margin is the
   finest of the sixteen (4 linear ones × 166 free and a single randomizer of 40 dimensions, against
   632; «linear ones and randomizers» gives exactly 632), so that is where a different instance can
   give another rank.
3. The advantage bound of the proposition has no term for this. Over (z, cc, α_i) the
   Schwartz–Zippel argument holds (although the degree of the minor is not stated; it is of the
   order of thousands, which gives ~2^{−115}, more than enough); over the positions there is no
   bound and the proposition states it as if there were one.
4. What is independent of the positions, and the document could say: the coverage of ONE dimension
   over F per point of layer 0 by the random column is a Vandermonde fact, not a measurement. Its
   residue is V_pos·(x−z)(x−z·g)(x−z̄)(x−z̄·g)·u with u ∈ F[x] of degree < L − q − 4, and at the 7c₀
   points of the cosets (distinct from one another, outside the positions and outside H) the fixed
   factor does not vanish, so u ↦ (u(x))_x is surjective onto F^{7c₀} whenever 7c₀ ≤ L − q − 4
   (294 ≤ 466 with T = 256). The same for the direct path of each linear column, with T − 2q − 6
   instead of L − q − 4, at the points that are not touched. What depends on the positions is
   reduced to the second coordinate at the coincidence points of point 2 and to the new functionals
   of layer 1.

What closes it, cheaply: in the seal, solvency with two or three more seeds (15 s each) and with an
instance of positions that forces a coincidence x_j·μ = g·x_k and a collision of cosets; in the long
test, the sixteen with two seeds. Or a lemma that makes the coverage independent of the positions,
by way of point 4. Until then, the proposition of D-AM holds for the measured instance and for what
the dimension count suggests, not for «every proof» with a numerical bound.

### O2 (medium). H12 is applied as the instrument computes it, but the lemma that justifies it is not written

Passage. Section 14, «Lo que vale» [What holds]: «M grows by one more mask that meets what L3 asks
of a mask —the random rows of a linear column, conditioned on S_toc, are uniform in the kernel of
the evaluation on S_toc, which does not depend on the witness, and their image in D is linear and
public—». And step 2 (line 853): «What the verifier sees of the chunks —each one at the q positions,
at z and at z·g— is discounted by the JOINT kernel with the randomizers: rank(O∘M) = rank([opened |
O∘D]) − rank(opened)».

Why it is a gap. L3 asks two things of a mask: a linear image in D and a uniformity INDEPENDENT of
D_resto once the view is conditioned on. The random rows of a linear column meet the second against
what is opened of the trace and against C(x_j) and C(z·g) (everything vanishes on S_toc), but NOT
against the openings of the chunks: P_ρ moves each h'_k(y) at y ∈ S_ab (only the sum Σ_k
y^{ks}·h'_k(y) = C(y) stays fixed). The instrument resolves it correctly —it restricts the parameter
space (w_i, r_k) to the joint kernel of the openings of the chunks, with the rank formula—, but the
quoted sentence presents M_lin as «one more mask» of L3 without saying why conditioning on the
openings of the chunks does not break the independence. The missing lemma is this one, and it is
short:

> Let A_L and A_r be the linear maps from (ρ_lin residual, r) to the openings of the chunks
> (h'_k(y))_{k, y∈S_ab}, and A_h(N) the contribution, non-linear, of the witness and of the random
> rows of the non-linear columns. Conditioned on the openings o and on N, the pair (ρ_lin, r) is
> uniform in a translate of the joint kernel J = ker[A_L | A_r]: the prior is uniform on a product
> of vector spaces and the condition is affine. The translate depends on the witness (through N and
> o), the free part j ∈ J does not. The fiber is non-empty for every ρ_lin because A_r is surjective
> onto the hyperplane of each point (the triangularity of L2). Hence D = D' + M(j) + M_R(u) with D'
> in D₀ + K determined by the rest of the view and j, u uniform and independent of everything else;
> and O(D) is uniform in O(D₀) + O(K) if and only if O(M(J) ⊕ M_R) = O(K), which is exactly
> rank([opened | O∘D]) − rank(opened) = rank(O|_K).

Without this paragraph, a reader who checks M_lin against the definition of mask of L3 rejects it on
objection 1 of §897 (it moves what is opened) and does not understand why the instrument subtracts
ranks. It is worth writing it as a lemma (L7, or H12 with a proof) and citing L2 from it.

### O3 (medium). What the canon measures of entry 134 is the census and solvency; the rank of the other fifteen families is a snapshot

Passage. Dossier, section 3, row H12: «measured in every canon (the census) and per family». Section
14: «`MLIN_MEDIDO`; solvency in every canon, the others in the long test
`mide_m_lin_en_las_23_familias`, once, with seed 1».

What I add: in the seal, for fifteen of the sixteen families, the only thing checked against
`MLIN_MEDIDO` is the COHERENCE of the declared verdict with the pinned row (`instrumento_cota.rs`,
the «§900» block: `esperado = … if m.5 == m.2`), not the rank. A change in the constraints of
SendAir that keeps the census counts (30 linear, 0, 28) —another slope, another assertion, another
order of columns— moves no guard of the canon even though it changes the rank of M_lin; only someone
who reruns the long test by hand would see it. The census is a guard of STRUCTURE, not of rank. For
a security promise it is advisable that at least the families with the finest margin (those with
T = 512 have 422 free per linear one and ≥ 13 linear ones; solvency is in the seal) or one per
geometry be measured in every canon, or that the canon require the date/sha of the last run of the
long test against the sha of the AIR files.

### O4 (low). The accounting of the proposition's advantage is incomplete

Passage. Section 5, end, and section 8: «advantage bounded by the 2^{−60} of H3 plus the oracle
queries times 2^{−256}». The 2^{−128} of the conditioning on the exact degree L − 2 (S5, line 445)
is not added; the drop in rank in (z, cc, α_i) is said to be boundable «by the degree over |E|»
without giving the degree; and the positions (O1) have no term. Neither of the first two changes
anything (they are 2^{−115} or smaller); the third does. I redid the count of H3 and it is correct:
each coincidence z̄ = z·g^e is z^{p−1} = g^e, p − 1 solutions in E, ≈ 2^{−64} per equation.

### O5 (low). Two imprecisions of the verifier's view

- P5: «each leaf is Blake3(item ‖ salt) with salt = Blake3(32 bytes from `OsRng`)». What `sal.rs`
  does (lines 194 and 254) is leaf = `merge(hash_elements(fila), sal)` with salt =
  `H::hash(32 bytes)`: the item is the digest of the row, not the row, and the salt is a digest. It
  does not change L4, but «item» should say «digest of the row» so that (a) reads correctly: what
  the adversary would have to guess without the salt is the whole row, not a digest it knows.
- Section 2: (V5) «the roots of the FRI layers and the whole remainder» does not name the commitment
  of the remainder (`hash_elements(resto)`, the last one of `commitments`, which the fork's
  `read_remainder` binds); and `num_unique_queries` (a `u8` of the `Proof`) is not in V1–V8. Both
  are functions of what is listed; it is worth saying so, so that the model of the view and the
  `Proof` match field by field.

### O6 (low). L0 describes the grinding with the `concurrent` feature off, and does not say so

Passage. L0 (line 177): «the search for the nonce —the first whose hash with the coin carries 21
zeros— runs over oracle outputs that the simulator does NOT fix». The code is
`(1..u64::MAX).find(…)` without `concurrent` (`winter-prover/src/channel.rs:179`) and `find_any`
with it (line 185). I checked that no `Cargo.toml` of the workspace enables `concurrent` (only
`winter-prover` defines it), so today L0 is exact. But with `find_any` the real nonce is not the
first —it depends on the split between threads— and a simulator «with the same search» would not
reproduce its distribution; it does not depend on the witness, it leaks nothing, and it is a formal
deviation. The document should tie L0 to the feature being off, or say that the simulator runs the
same parallel search.

### O7 (low). The degree of DEEP is asserted with a panic, in production

Passage. S5: «conditioned on degree exactly L − 2 as the prover asserts». The assertion is
`assert_eq!(trace_length - 2, deep_composition_poly.degree())` in `generate_proof_oculto`
(`winter-prover/src/lib.rs:729`) and another identical one inside `add_trace_polys`
(`composer/mod.rs:165`): with probability ≈ 2^{−128} a VALID witness panics, against the letter of
D-AH («a bad witness is an `Err`, never a panic» —here it is not even bad—). It does not touch
hiding (the simulator conditions the same way and the difference is the one the document states),
but the document presents it as a condition of the protocol and it is a panic inherited from
winterfell.

## 3. What I took as correct, and with what

Each line: the document's sentence against the code that supports it.

- **P1** («T elements of F behind the T real rows of each main column; then L elements of F for …
  the random column; and then, if there is an auxiliary segment, T elements of E») against
  `generate_proof_oculto`: the loop `for c in 0..principal.num_cols()` with `t` draws,
  `(0..2 * t).map(…)` for the random columns, and `for _ in 0..t { col.push(sacar::<E>) }` in the
  auxiliary segment; the coin `DefaultRandomCoin::new(&[])` reseeded with `Blake3::hash(&semilla)`
  and never with anything from the public one (H10).
- **P4** and **L2** («h'_k = h_k + x^s·r_k − r_{k−1}, with r_{−1} = r_{N−1} = 0 and r ∈ E[x] of
  degree < m … s = L − m») against `segmentar`: `s = trace_len - m`; `t[k] -= r_ant[k]` for k < m;
  `t[s + k] += r[k]`; `r` is drawn with `moneda.draw::<E>()`; the last chunk receives no r; the
  assertion that the coefficients beyond s·N are zero; `m < trace_len && num_cols >= 2` (H5).
- **P2** against `Oculta::new`: `n = base.num_transition_exemptions() + t`, the inner AIR with
  `t = l / 2` and `main_trace_width − columnas`, the `ce` that rises until `cabe_con`; the
  transitions, assertions, periodic ones and auxiliary segment delegated untouched. With n = n_base
  + T the exempt rows are T − n_base … 2T − 1: the last real one → first random one and all the
  random ones, as the document says. Λ ∩ H = ∅: the offset is `GENERATOR` = 7, of order p − 1, which
  does not divide 16L.
- **P5/L4** against `sal.rs`: `with_options` draws one salt per leaf (`sal::<H>()`, `OsRng`),
  `con_sal` does `merge(item, sal)`, `open_many` returns the salts of the opened leaves and
  `verify_many` mixes them in again; the three commitments go through `V::new` (trace and quotient
  in `matrix/col_matrix.rs:285` and `row_matrix.rs:227`; FRI in `build_layer_commitment` of
  `winter-fri`), so the three classes of leaf are salted.
- **P6/P11** against `perform_verification`: the OOD consistency only at z, with `paso = L − m`
  (`winter-verifier/src/lib.rs:363`) and `current_row()`; at z·g nothing is checked; DEEP is redone
  by `DeepComposer::compose_columns` with the formula of P7 (one same cc per column for the two
  quotients, `self.cc.trace[i]` and `self.cc.constraints[i]`) and FRI checks it against the leaf of
  layer 0 (`evaluations != query_values` in `verify_generic`).
- **P7** against `composer/mod.rs` (prover): `acc_trace_poly` subtracts the OOD value, divides by (x
  − z) and (x − z·g) and adds them; one coefficient per polynomial.
- **P8/P10/V8** against `winter-fri` 0.13.1: `transpose_slice` groups eight by eight (row i is the
  values i, i + n/8, …: the coset {x·μ}), `apply_drp` folds with α, `set_remainder` interpolates and
  sends `len / blowup` whole coefficients; the verifier reads the whole leaf of the coset
  (`read_layer_queries`), interpolates the row and evaluates at α; the remainder stays bound to its
  commitment in the fork's channel (`read_remainder`). The numbers of layers and remainders per T
  (1/16, 2/8, 2/16, 2/32) come out of `num_fri_layers` with `max_remainder_size = 32·16`.
- **P9/L0** against `DefaultRandomCoin`: `check_leading_zeros` looks at `merge_with_int(seed,
  nonce)`; `draw_integers` reseeds with the nonce and draws the positions from
  `merge_with_int(seed', counter)` for counter = 1..42, with repetition; the prover
  (`get_query_positions`) and the verifier do `sort_unstable(); dedup()`. They are oracle outputs
  distinct from those of the grinding: what L0 needs to fix some without fixing the others. The
  claim «it does not depend on the witness» holds.
- **Section 2** against `Proof { context, num_unique_queries, commitments, trace_queries,
  constraint_queries, ood_frame, fri_proof, pow_nonce }`: nothing of the proof is left outside
  V1–V8 except what O5 says (functions of what is listed). The `OodFrame` carries the two rows of
  the trace (the random one included) and the two of the chunks, and `merge_ood_evaluations` puts
  all four into the coin.
- **L3, Q[P](x₀)**: redone by hand; 2·(P(z·g) − P(z))/(z·g − z). That D(x₀) is the ONLY public thing
  of D outside D(x_j) I took as correct with an argument that the document does not write and that
  is worth writing: the residue of a single chunk h'_k (degree < L, zero on S_ab) is
  (x−z)(x−z·g)·V_pos·v with deg v ≤ L − q − 3, and its image Q[·] is ℓ·V_pos·v, which ranges over
  ALL of K = ℓ·V_pos·E[x]_{≤L−q−3}; hence no functional of D outside the one generated by {ev_{x_j},
  ev_{x₀}} is a function of what is opened. K is the correct space as a support; the distribution
  over it is R3.
- **L3, rank of O**: 8c₀ + (L/8 − c₀) over E when 8c₁ ≥ L/8, coherent with the table of section 13
  (550, 422, 358; 127 = the whole D with T = 64) and with «undetermined» = 2·rank − 2q (− 2 only
  with T = 64, where x₀ falls in the row space). The «one dimension per point» of the random column
  is true and, as I say in O1.4, provable.
- **`instrumento_r3`** against section 6: `Mirilla::observa` reproduces exactly what `FriProver`
  opens (same transposition, `fold_positions`, `apply_drp`, same offset, the remainder as
  coefficients); K = V_pos·(x − x₀)·u; M_R = cc_R·ℓ·V_pos·(x−z̄)(x−z̄g)·u with u ∈ F[x] —I deduced
  the formula of the residue of R and it matches—; M_r = (cc'_k·x^s − cc'_{k+1})·V_pos·ℓ·x^i
  —deduced the same way—; the ranks over F by `sobre_f` with the pair (row, u·row) for the spaces
  over E; the inclusion assertions. The pinned figures are coherent with the structure (632 = 2·294
  + 44; column = 294 + 44).
- **`instrumento_mlin`**, the heart of entry 134:
  - `Sonda::composicion` is NOT a replica of the prover's evaluator: it is, line by line, the
    verifier's `evaluate_constraints` (`get_transition_constraints` → `combine_evaluations` with the
    divisor at x; `get_boundary_constraints` → `evaluate_at`; the periodic ones as `p(x^{L/c})`, the
    same as `periodic_table.rs` with `num_cycles = L / len`). Since C is a polynomial and the
    rational formula coincides with it at every x where the divisors do not vanish —and the
    constraint domain 7·⟨ω_ce⟩ touches neither H nor the exemptions—, the interpolation of `muestra`
    over that domain returns the linear part of the prover's C exactly. That this formula is the
    prover's is guaranteed by the 28 real proofs passing the OOD check at z. The validation «against
    the OOD frame of an AIR with periodic ones» that the dossier finds missing (attack 4) would add
    nothing that the verifier does not already give in every proof.
  - The probe: zero second difference along e (affine) and equal slope in two random frames
    (public). It catches the products with another column and the bilinear P_i(x)·P_i(g·x) (the
    slope carries the other entry of the frame, which changes between f and f'), and a false
    «linear» requires a non-zero polynomial to vanish at three random points of E. «Linear» implies
    affine in (P_i(x), P_i(g·x)) jointly, which is what `pendientes` uses afterwards.
  - M_lin: basis Z_real·V_toc (V_toc with the conjugates, asserted in F[x]), free = T − 2q − 6, the
    slope at each point of the constraint domain for the current row and the next
    (`kg = (k + ce) % ce_tam` is the index of g·x_k), the chunking with step s like `segmentar`, the
    randomizers added and subtracted as there, the openings of the chunks on S_ab as the «opened»
    block, D = Q[direct + Σ cc'_k·chunk_k] + Φ·u. The formula rank([A | B]) − rank(A) = dim B(ker A)
    is correct, and the inclusion check (rank with K minus rank(opened) = rank(K)) proves B(ker A) ⊆
    O(K), as I deduced. With M_lin empty it reproduces `R3_MEDIDO`, which validates O, K and the two
    masks but NOT the quotient path; that one is validated by the toy (76 = 2·38 exact inside K).
  - The empty public inputs (attack 5 of the dossier): in the sixteen AIRs the public inputs are
    only VALUES of `Assertion::single` (e.g. `banda.rs:150–159`, `cobro_pendiente.rs:164–175`,
    `pago_en_curso.rs:171–182`) or are stored; no `if`/`match` on them in `new`,
    `evaluate_transition` or `get_assertions`; the number of assertions is a constant of `new`
    (solvency: `AirContext::new(trace_info, degrees, 1, options)`). None of the sixteen uses
    `Assertion::periodic`/`sequence` or an auxiliary segment (only `circuit_edad.rs` and
    `zk-ssl-air/src/lib.rs`, age). The shape of C does not depend on the public inputs in the
    sixteen.
  - The census of solvency (4 linear, 0, 1 non-linear) against its AIR: `COL_BIT` enters in bit·(bit
    − 1) (degree 2); `COL_ACC` and the three transport columns enter linearly with constant or
    periodic slopes (`first`, `cont`, `link_*`, fixed matrices). Coherent.
- **`instrumento_cota`**: the Lagrange `Z_H(y)·h_r/(2T·(y − h_r))` is the correct basis over H of
  size 2T; `rango_f` splits each row of E into its two coordinates; S_toc are the opened points, the
  g·x_j and g²·z without repeats. I add that the «full rank» of H1 is not only measured: the very
  proof of L1 gives it for any distinct points with |S̄| ≤ T (Vandermonde with distinct nodes and
  the trick (R' + conj R')/2); what is assumed is only the distinctness (H3). The document says it
  as a proof of life; it could say it as a theorem.
- **The falsifiers**: `vista` redoes the coin exactly in the order of `perform_verification`
  (context and inputs, root of the trace, α, root of the quotient, z, the digest of the OOD frame,
  cc, one α per FRI commitment, the nonce, the positions with `dedup`), and the three attacks are
  the ones the document describes; «45 equations over 33 unknowns consistent only with the real
  challenges» is a strong validation of the redone coin, as section 13 says.
- **L5**: `ocultacion_encendida` calls `sal::semilla()` twice (32 bytes from `OsRng` each); the two
  coins are seeded apart from the public one and the public one does not see them.
- **Section 2, the count of what is touched**: the transitions of winterfell 0.13 read only (row,
  next row) (`EvaluationFrame`), the boundaries only the current row, the periodic ones are not
  trace columns: S_toc = S_ab ∪ g·{x_j} ∪ {g²·z}, 2q + 3 points.
- **The simulator's order against Fiat–Shamir** (S1–S7 with L0): I checked that the programming
  chain is consistent: the positions are chosen before building any tree; the prefixes that are
  programmed (α after the root of the trace, z after that of the quotient, cc after the OOD digest,
  α_i after each FRI root, the positions after the nonce) carry roots with fresh digests; the
  commitment of the remainder and the grinding are honest outputs of the oracle.

## 4. What the dossier does not let me answer, and what would be needed

1. **The real challenges (O1).** Without running anything, I do not know what rank R3 with M_lin
   gives in solvency with another seed, with a collision of cosets or with a coincidence x_j·μ =
   g·x_k. It would take running `instrumento_mlin::mide` with several seeds and with forced
   positions (a `Desafios` built by hand is enough), and also printing `rango_lineales` WITHOUT
   randomizers, which the instrument does not report: it would say how much of the margin of
   solvency rests on the 40 of the randomizer.
2. **The exploration run of §898** with seed 2 for T ≥ 256 is not in the tree; section 13 cites it.
   A file with its output, or its inclusion in the long test, would make it reviewable.
3. **A coverage lemma independent of the positions.** The path of O1.4 (Vandermonde at the points of
   the cosets) covers layer 0 coordinate by coordinate if, besides the random column, at least one
   linear column with T − 2q − 6 ≥ 7q + (coincidences) free contributes the direction cc_i·c(x),
   F-independent of cc_R·c(x) when cc_i/cc_R ∉ F. With T ≥ 512 (422 and 934 free against 294) it is
   immediate; with T = 256 (166 < 294) the four columns are needed together and the argument is one
   of joint rank, not of Vandermonde. What that path does not cover are the L/8 − c₀ functionals of
   layer 1 (22 over E with T = 256). Writing it would turn «measured in one instance» into «proved
   except for layer 1», which is what a promise needs.
4. **Entry 135, with numbers.** The dossier (attack 1) asks whether the fine count would rescue the
   five with T = 64. By count, and only by count: each linear column conditioned on S_ab keeps T − q
   − 4 = 18 free; C(x_j) and C(z·g) are 2q + 2 = 86 dimensions over F IN TOTAL (one sum for all the
   columns, not per column); and FRI delivers the whole D, 168 undetermined, with 126 covered by the
   two masks of §898. NullifierThresholdAir (8 linear: 144 − 86 = 58; 126 + 58 = 184 ≥ 168) is the
   only one that passes the count; ThresholdAir (7: 126 − 86 = 40; 166 < 168) does not, and
   RefundAir (0), RefundAirV2 (4: 72 < 86) and EdadAir (4 + auxiliary segment) do not even reach C.
   Hence entry 135 can, at most, rescue one of the five; the other four need entry 131 or entry 133
   by count, not by structure. It would take the lemma L1' that entry 135 asks for and the
   measurement with V_ab.
5. **The salt (attack 7).** With R3 green, the question «what would fail without it, leaf by leaf»
   has an answer in the model: an unopened leaf of the trace at x ∉ S_toc carries R(x) uniform (82
   to 2002 free against the 46 opened) and, with T ≥ 256, each P_i(x) uniform (2q + 7 ≤ T), so its
   preimage has ≥ 64·(W + 1) bits of entropy; a leaf of the chunks, ≥ 128·(N − 1) (q + 3 ≤ m); a FRI
   leaf of an unopened coset, ≥ 8·64 from the single coordinate that the random column moves at each
   point (7c₀ + 8 ≤ L − q − 4). Nothing would break with T ≥ 256; with T = 64 the FRI leaves would
   have zero entropy, but D is already public in full. What the salt buys is that the SIMULATOR does
   not have to argue entropy leaf by leaf (L4(a)); without it the simulator changes (L4(b) or a
   lemma per class of leaf). The recommendation to keep it is correct and cheap to sustain; removing
   it would require rewriting S6 and L4.
6. **Note 2024/1037.** It is not in the copy and I did not read it outside; section 10 is still
   pending, just as it says.
7. **Two things of the fork that do not belong to this dossier but that I saw while reading**, for
   the audit of H7: (a) `ConstraintDivisor::from_assertion(assertion, L)` takes `num_steps` over L,
   so an `Assertion::periodic` of an inner AIR would also be enforced on the random rows (today none
   of the 23 uses it; if one did, the hidden prover would fail at the assertion of `segmentar`, it
   would not leak anything); (b) the exemptions of `Oculta` «are written without the assertion» of
   `AirContext` and the second bound is decided in `cabe_con`, which only the prover looks at.

## 5. Verdict

The proposition of D-AM for the sixteen families holds in its structure: the honest verifier's view
is the one section 2 describes (I checked it against the `Proof` and against
`perform_verification`), K is the correct support of what FRI opens without determining (and I gave
a closed argument that D(x₀) is the only public thing of D outside the positions), L1 and L2 are
theorems with honest hypotheses, R3 is sufficient by the reasoning of L3, and H12 is correct —with
the joint-kernel lemma that the instrument applies and the document does not write (O2)—; the probe
and the two rank instruments do what the document says they do, with the verifier's composition and
not a replica, and the sixteen AIRs do not change shape with their public inputs. What it lacks to
be a promise with the numerical bound it states is what O1 describes: R3 with M_lin is measured on
ONE instance of challenges per family, the real positions are not generic in the sense of the
argument (coset collisions in more than half of the proofs of solvency, coset–touched coincidences
with frequency Θ(1)), the bound 2^{−60} + Q·2^{−256} has no term for them, and the family where that
matters —solvency, with the finest margin— is also the only one the canon re-measures. With two or
three more seeds and an adverse instance of positions in the seal, or with the per-point coverage
lemma of O1.4, the proposition would go from «holds for the measured instance and for what the count
suggests» to what D-AM means to say; until then, the author can accept it knowing that that is the
gap, and that it is not a gap of structure but of coverage of the measurement.
