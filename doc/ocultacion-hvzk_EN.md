# The hiding of the fork, with its simulator: the argument of A2 (§897)

> **Español:** [`doc/ocultacion-hvzk.md`](./ocultacion-hvzk.md) is the original of this page, in
> Spanish; it prevails wherever the two differ. This translation is of commit `7766efb` (audit entry
> §917) and was produced by the project's assistant, a Claude Code session, in audit entry §918
> (`GENAI.md`): it is a generated artefact, not text written by the author. Line citations of the
> form `file:N` refer to the Spanish files.

**Status**: working document of entry 130 of `BACKLOG.md` (A2 of the HVZK), written in §897 by the
session, and **reviewable**: every objection with its closure goes in section 11 and in the entry
that carries it. ⚠️ §898 (A3): R3 is measured, and it does not hold in any production geometry;
what that changes is marked with §898 where it applies and gathered in section 13. **It is not a
promise.** What the house promises is still the census —«a proof that verifies demonstrates the
transition it states and publishes literally no constant-column value of its trace», RFC-0009 D-A
and `SECURITY.md` §3.bis— until A4, with A3 sealed, decides what is promised and with what words.
Nothing of what follows changes `SECURITY.md`, RFC-0009 beyond the note of its D-I, or rule 3 of
the PROCESO; no live text says HVZK of anything that exists. ⚠️ §901 (A4): decided in D-AM of
RFC-0009: the promise is still the census; what could be promised for sixteen families with
today's construction —the proposition of section 8, under the hypotheses of section 4 with H12 of
§900— is written there in its words and is the author's; the seven that cannot, by name and with
what they lack. ⚠️ §902: the author accepts that path and decides A5: the salt stays (RFC-0009
D-I); what is written awaits the external review (entry 136 of `BACKLOG.md`). ⚠️ §903: the
dossier for that review —what to review, what to read, how to reproduce every figure and where to
attack— is `doc/ocultacion-hvzk-dosier.md`. ⚠️ §915: with the `zkssl/0.5` cut sealed (§914), the
proposition for the twenty-three families with the production construction —the two paired columns,
T ≥ 128, m = 64, the salted leaves— is written in section 18, with its hypotheses and its count, and
what could be promised with it, in its words, in D-AM of RFC-0009 (its draft of §915). **It is still
not a promise**: it is the author's, and the argument awaits the external review of entry 136, which
from here on is of sections 1 to 18 with the construction of the cut. ⚠️ **§916: accepted by the
author** (29-10-2026, «Aceptar todo» [accept everything]): the proposition of section 18 is, since
§916, **the promise of the house** for the twenty-three families with `zkssl/0.5`, with its
hypotheses (RFC-0009 D-A and D-AM; `SECURITY.md` §3.bis); H7 is kept and the external review of
entry 136 is still pending, and the promise says so. What this document calls «it is not a promise»
from here to section 18 is the history of how it came to be one.

**What it is.** The hiding argument of the hidden prover of the winterfell 0.13.1 fork
(`crates/winter-prover`, `crates/winter-air`, `crates/winter-verifier`, with the salt of
`crates/zk-ssl-air/src/sal.rs`) as it is implemented, with its simulator and its lemmas, each
lemma tied to the test of §896 and §897 that measures its hypothesis
(`crates/zk-ssl/src/instrumento_cota.rs`), the list of hypotheses with their class —measured,
proved, model or assumed—, what it does not cover and where the fork departs from note 2024/1037.

**What it gives, in one sentence.** ⚠️ §898: up to here this paragraph said that for seventeen of
the twenty-three families the honest verifier's view was simulated without the witness «if a
sufficient condition that nobody has measured, R3, holds», and that six stayed out, the five with
T = 64 by L1 and per-account solvency. A3 measured R3 (section 13) and **it does not hold in any
production geometry**: the random column lives in F and, at each value of E that FRI opens, it
moves only one of its two coordinates; the quotient randomizers contribute 80 dimensions in total,
not 40 per chunk; and what FRI opens without determining is 168, 632, 760 and 1016 dimensions
against 126, 378 to 382, 510 and 766 of mask. Today the AIR-independent argument covers no family:
what sustains hiding beyond the census, if anything sustains it, depends on the AIR —on a linear
mask of its own, where its columns enter the constraints only linearly (M_lin, entry 134 of
`BACKLOG.md`, unmeasured), or on the witness (L6)—, and nobody has argued it. The AIR-independent
way out is entry 133: that the DEEP mask live in E, in a future cut. The statement of section 8
cannot be made today for any family; the promise is still the census. ⚠️ **§900 (entry 134):
M_lin, measured (section 14).** In sixteen of the twenty-three families —the six with T = 1024,
the nine with T = 512 and solvency— R3 holds with today's construction once the random rows of the
columns that enter the constraints linearly and with a public coefficient are counted, and in all
of them those rows with the randomizers suffice on their own; MerkleAir and FrozenClimbAir
(T = 256) have no such column and keep the R3 of §898; the five with T = 64 have no M_lin to
measure (entry 131). For the sixteen, the proposition of section 8 holds with today's
construction, under the other hypotheses; the promise is still the census until A4.
⚠️ **§914 (the `zkssl/0.5` cut, RFC-0018 D-H):** the production construction is, since the cut, that
of the two random columns paired as one over E (D-AL bis, L9) in the 23 families, with T = 128 in
the five that had 64: R3 measured green in the five geometries (section 13, `K_ENTERO_PAREADAS`), L1
covering the 23 (`INDEPENDIENTE_DE_LA_AIR`), the bound of A1 green in the 28 proofs. What this
document says of «today» from here to section 17 describes `zkssl/0.4`; the proposition for the 23,
with its hypotheses and its count, is written in the next entry, and the promise is still the census
until the author accepts it.
⚠️ **§915: with the construction of the cut, in one sentence.** For the twenty-three families, the
honest verifier's view is simulated without the witness with the AIR-independent argument: L1 per
geometry (2q + 6 = 90 ≤ 128 ≤ T, in the 23), L2 (q + 2 = 44 ≤ m = 64), L9 (the two paired columns
and the randomizers cover the whole of K, for any positions: R3 is a theorem, not a hypothesis), L4
with the salt and L5 with the seeds, in the model of L0; neither M_lin, nor the fine count, nor L6,
nor L8 are needed, and they remain as guard instruments. The advantage: 2^{−60} (H3) + 2^{−128}
(S5) + Q·2^{−256} (L0) + |E|^{−(N−1)} (L9), with no |F| at all. What is left is what no test
measures —H3, H7, H8 and H9— and the external review; the promise is still the census until the
author accepts it (section 18).

**The note, not read.** ePrint note 2024/1037 —Ulrich Haböck and Al Kindi, «A note on adding
zero-knowledge to STARKs», received on 26 June 2024 and last revised on 22 February 2025, according
to what a search of its page returns— could not be read from this session's container:
`eprint.iacr.org` does not resolve, and to read it from a cloud session that domain has to be
allowed in the session's network. Section 10 compares with what the house knew of it through
RFC-0009 and with its abstract; the section-by-section comparison is pending.

**Convention.** «Proved» is what this document proves; «measured», what a test of the tree
measures in every canon; «model», a standard hypothesis that no test can measure (the random
oracle); «assumed», what is neither proved nor measured, with its failure probability when it is
bounded. The ⚠️ marks what §896 said and this document corrects.

## 1. The protocol, as it is implemented

**Notation.** F is the production field, winterfell's 64-bit one (`f64`, p = 2^64 − 2^32 + 1); E
its quadratic extension, where z, the quotient coefficients and the DEEP polynomial live
(`FieldExtension::Quadratic` in `proof_options()`, `crates/zk-ssl/src/lib.rs`); conj is the
conjugation of E over F. The real trace has T rows, a power of two with T ≥ 64 (RFC-0009 D-AG), W
main columns and W_aux auxiliary ones; the hidden one, L = 2T rows. H = ⟨g⟩ ⊂ F is the domain of
the hidden trace, with g of order L; the real rows are h_0 … h_{T−1} and the random ones
h_T … h_{L−1}. Λ = 7·⟨ω⟩ is the LDE domain, with ω of order 16L (blowup 16) and 7 the generator of
F* (`domain_offset`, `crates/winter-air/src/options.rs`); since 7 has order p − 1, which is not a
power of two, 7 ∉ ⟨ω⟩ and Λ ∩ H = ∅: no opened position falls on a row, and 0 ∉ Λ. The production
options (`proof_options()`): 42 queries, blowup 16, 21-bit grinding, quadratic extension, folding 8
and a remainder of degree ≤ 31 in FRI, linear batching. q is the number of unique positions a
proof opens, from 40 to 42 depending on the run, read from it (`num_unique_queries`).

- **P1. Random rows and column** (`generate_proof_oculto`, `crates/winter-prover/src/lib.rs`). A
  coin `DefaultRandomCoin<Blake3>` is seeded with Blake3(rows_seed), a 32-byte seed from `OsRng`
  per proof (`ocultacion_encendida`, `crates/stark-experiment/src/lib.rs`; `sal::semilla`). From
  that coin come, column by column, T elements of F behind the T real rows of each main column;
  then L elements of F for one more main column —the random column, with no constraint at all—;
  and then, if there is an auxiliary segment, T elements of E behind the real rows of each
  auxiliary column. The trace meta carries the mark with m (D-S); the real trace is checked
  against the inner AIR before hiding it (D-AH).
- **P2. The wrapper** `Oculta<A>` (`crates/winter-air/src/air/oculta.rs`): it presents to the core
  an AIR of L rows with W + 1 main columns and the inner one's transition exemptions plus T —the
  transition constraints are not enforced between a random row and the next, nor between the last
  real row and the first random one— and delegates to the inner one the transitions, the
  assertions (which name real rows, because the inner one sees T), the periodic columns and the
  auxiliary segment. It raises the constraint domain factor if needed; only the prover uses that.
- **P3. The column polynomials.** For each column, the polynomial P ∈ F[x] (E[x] in the auxiliary
  ones) of degree < L that interpolates its L values over H. Its evaluation over Λ is committed
  with `MerkleConSal` (P5), one leaf per point of Λ with the whole row of the segment.
- **P4. The quotient and its chunks** (`segmentar`,
  `crates/winter-prover/src/constraints/composition_poly.rs`; `num_constraint_composition_columns`,
  `crates/winter-air/src/air/context.rs`). With the coefficients α of the constraints, drawn from
  the coin after the trace commitment, the composition polynomial C(x) = Σ_c α_c·C_c(x)/Z_c(x)
  —transitions over the frame (P(x), P(g·x)) of all the columns, and assertions— has degree < N·s,
  with s = L − m and N the number of chunks. It is split as C(x) = Σ_{k<N} x^{k·s}·h_k(x), with deg
  h_k < s, and each chunk is masked: h'_k = h_k + x^s·r_k − r_{k−1}, with r_{−1} = r_{N−1} = 0 and
  r_0 … r_{N−2} ∈ E[x] of degree < m, uniform from a coin seeded with Blake3(quotient_seed), the
  other 32-byte seed. The sum telescopes: Σ_k y^{k·s}·h'_k(y) = C(y) for every y. Each h'_k has
  degree < L and is committed, evaluated over Λ, with `MerkleConSal`. m = 64 in the 28 proofs
  (`M_OCULTACION`).
- **P5. The salted commitment** (`MerkleConSal`, `crates/zk-ssl-air/src/sal.rs`): each leaf is
  `merge(hash_elements(fila), sal)` —the Blake3 of the digest of the row and of the salt— with
  salt = Blake3 of 32 bytes from `OsRng`, fresh per leaf; the opening carries the path and the salt.
  ⚠️ §907 (O5 of the agent's review): it said «Blake3(item ‖ salt)», and the item is the digest of
  the row, not the row; what an adversary would have to guess without the salt is the whole row, not
  a digest they know; L4 does not change. The trace, the chunks and the FRI layers use it
  (winter-fri builds its layers with `V::new`); the FRI remainder does not go in a tree: it is
  committed as the Blake3 of its coefficients and travels whole.
- **P6. The out-of-domain point.** z ∈ E comes from the coin after the quotient commitment. The
  prover sends the OOD frame of the trace —P_i(z) and P_i(z·g) of each column, main (the random
  one included) and auxiliary— and that of the quotient, h'_k(z) and h'_k(z·g) of each chunk
  (`QuotientOodFrame`). The verifier checks Σ_k z^{k·s}·h'_k(z) = C(z), with C(z) computed from
  the trace frame (`crates/winter-verifier/src/lib.rs`, step 3); at z·g it checks nothing: those
  values go into DEEP.
- **P7. DEEP** (`crates/winter-prover/src/composer/mod.rs`,
  `crates/winter-verifier/src/composer.rs`). With a coefficient cc per column and per chunk, drawn
  from the coin after the OOD frame (linear batching): D(x) = Σ_i cc_i·Q[P_i](x) + Σ_k
  cc'_k·Q[h'_k](x), with Q[P](x) = (P(x) − P(z))/(x − z) + (P(x) − P(z·g))/(x − z·g). It is a
  polynomial of degree ≤ L − 2 over E, and the prover asserts that the degree is exactly L − 2.
- **P8. FRI** (winter-fri 0.13.1, unforked). Over D evaluated on Λ: at each layer the values are
  grouped eight by eight —the folding cosets, {x : x^8 = c}— into one leaf, committed with
  `MerkleConSal`, α_i comes from the coin and it is folded. For the house's traces that is one
  layer (T = 64) or two (T ≥ 256), and the remainder, from 8 to 32 coefficients depending on the
  length, travels whole (§896, table).
- **P9. Grinding and positions.** The prover looks for a nonce with 21 leading zeros in the hash
  of the coin; from the coin with the nonce come 42 positions of Λ, with repetitions, and the
  prover opens the unique ones: q ≤ 42. The positions do not travel; the verifier derives them.
- **P10. Openings.** At each position x_j: the whole row of the main trace (the W + 1 columns) and
  of the auxiliary one, with its salt and its path; the value of each chunk h'_k(x_j), with its
  salt; and in FRI, at each layer, the whole leaf of the folding coset of the folded position
  —eight values, one of them the position's— with its salt.
- **P11. The verifier** (`perform_verification`): it rebuilds the coin, checks the grinding, the
  OOD consistency at z and the openings against the roots, computes D(x_j) from the openings and
  the frames with the formula of P7 and hands it to FRI, which checks that the computed value is
  the one the layer-0 leaf carries at its position, folds each coset with α_i and checks the next
  layer, and at the end the remainder.
- **P12. What is public by design.** The public inputs, the context (lengths, widths, options,
  the mark with m) and whatever the envelope, the receipt or the operation carry in the clear
  (RFC-0009 D-B, column «público por diseño» [public by design]). It is not the object of this
  argument.

## 2. The honest verifier's view

What it receives, in order: (V1) the roots of the main trace and, if there is one, of the
auxiliary; (V2) the OOD frame of the trace, 2(W + 1 + W_aux) values of E; (V3) the root of the
quotient; (V4) the OOD frame of the quotient, 2N values of E; (V5) the roots of the FRI layers, the
commitment of the remainder —the Blake3 of its coefficients, the last of `commitments`, which the
fork's `read_remainder` binds to the remainder— and the whole remainder; (V6) the nonce; (V7) at
each unique position x_j, the main row (W + 1 values of F), the auxiliary one (W_aux of E), the N
chunks (E), and their salts and paths; (V8) at each FRI layer and each folded position, the eight
values of the coset, with salt and path. Plus what it derives itself: the challenges (the elements
of the auxiliary segment, α, z, cc, α_i, the nonce and the positions) and what it computes from the
above: C(z), D(x_j) and —although it does not check it— C(x_j) = Σ_k x_j^{k·s}·h'_k(x_j), C(z·g) =
Σ_k (z·g)^{k·s}·h'_k(z·g) and D(x₀), with x₀ = (z + z·g)/2 (L3).
⚠️ §907 (O5 of the agent's review, section 11): the `Proof` also carries `num_unique_queries`, a
`u8` that is a function of V6 —the unique positions that `draw_integers` derives from the nonce—,
and the commitment of the remainder, which V5 did not name and is a function of the remainder; with
both stated, the view and the `Proof` coincide field by field (`context`, `num_unique_queries`,
`commitments`, `trace_queries`, `constraint_queries`, `ood_frame`, `fri_proof`, `pow_nonce`).

The point sets, per column:

- **Opened**, S_ab = {x_1 … x_q} ∪ {z, z·g}: q + 2 points. The positions live in F and the two
  outside ones in E, so what is opened of a main column measures **q + 4 dimensions over F** (46
  with q = 42); of an auxiliary one, q + 2 over E; of a chunk, **q + 2 over E** (44).
- **Touched**, S_toc = S_ab ∪ {g·x_1 … g·x_q} ∪ {g²·z}: 2q + 3 points, because C(x_j) reads the
  frame (P(x_j), P(g·x_j)) and C(z·g) reads (P(z·g), P(g²·z)). It measures **2q + 6 dimensions
  over F** per main column (90 with q = 42) and 2q + 3 over E per auxiliary one. A g·x_j that
  coincides with another opened position counts once.
- **x₀** = (z + z·g)/2 ∈ E, where the linear factor with which each object enters DEEP vanishes.

## 3. The lemmas

Each lemma says what it proves, under which hypotheses, and which test measures that hypothesis
over the 28 real proofs of the 23 provers with a row
(`la_cota_de_ocultacion_de_los_23_probadores_con_fila`, in every canon) or over the toy, `WorkAir`
with 128 real rows, m = 64 and the production queries and blowup.

### L0 — The transcript and the oracle

Fiat–Shamir with the coin `DefaultRandomCoin<Blake3>`: each challenge is the Blake3 of what has
been committed so far. The argument is made over the interactive protocol with an honest verifier
—the simulator chooses the challenges, uniform, before building anything— and passes to the
non-interactive one in the programmable random oracle model: the simulator fixes the outputs of
Blake3 on the prefixes of its transcript so that the coin gives its challenges, and fails only if
the adversary had already queried one of those prefixes, which carry fresh roots (L4) and have
entropy to spare. Grinding does not depend on the witness: the search for the nonce —the first
whose hash with the coin carries 21 zeros— runs over oracle outputs that the simulator does NOT
fix, so that the nonce comes out with its distribution; what it fixes are the outputs after it,
the ones `draw_integers` derives from the nonce for the positions. ⚠️ §907 (O6 of the agent's
review): «the first» is the sequential search, `(1..u64::MAX).find`, the one the house compiles: no
`Cargo.toml` of the workspace switches on the `concurrent` feature of `winter-prover`; with it the
search is `find_any` and the nonce is the first one a thread finds, a distribution that does not
depend on the witness either and that a simulator with the same parallel search reproduces. L0 stays
tied to the feature switched off; whoever switches it on runs the simulator with the same search.
**Class: model.** No test measures it; it is the standard hypothesis of every Fiat–Shamir proof, and
the house already uses it for soundness.

### L1 — The random rows cover a set of points if its count over F fits in T

**Statement.** Let P = P_real + Σ_{r=T}^{L−1} ρ_r·L_r be the polynomial of a main column, with ρ
uniform in F^T and L_r the Lagrange basis over H. Let S ⊂ E \ H be finite, S_F = S ∩ F,
S_E = S \ F, and S̄ = S_F ∪ S_E ∪ conj(S_E). If the points of S̄ are pairwise distinct
and |S_F| + 2|S_E| ≤ T, then (P(y))_{y∈S} is uniform in F^{S_F} × E^{S_E}, for any P_real. For
an auxiliary column, with ρ uniform in E^T: uniform in E^S if |S| ≤ T and the points of S are
distinct. For the random column, with its L values uniform in F: the same with L in place of T.

**Proof.** A polynomial of degree < L that is 0 on the T real rows is Z_real·R, with
Z_real = Π_{r<T}(x − h_r) and R ∈ F[x] of degree < T, and the correspondence ρ ↦ Σ ρ_r·L_r ↦ R is
bijective: Z_real does not vanish on the random rows, and evaluating R at T distinct points,
scaled by non-zero constants, is bijective. Thus P(y) = P_real(y) + Z_real(y)·R(y) with
Z_real(y) ≠ 0 outside H, and the uniformity of (P(y))_S is equivalent to the evaluation
ev_S: F[x]_{<T} → F^{S_F} × E^{S_E} being surjective. For R ∈ F[x], R(conj y) = conj R(y): the
value at a point of E carries that of its conjugate. Given a target (v_y)_{S_F}, (w_y)_{S_E}, it
is extended to S̄ with conj(w_y) at conj(y); the Vandermonde of E[x]_{<T} over S̄ is surjective if
the points are distinct and |S̄| ≤ T, and gives a solution R' ∈ E[x]; conj(R') solves the same
system, and R = (R' + conj R')/2 ∈ F[x]_{<T} does too (2 is invertible in F). Conversely,
if |S_F| + 2|S_E| > T the dimension does not reach, and if y and conj(y) are both in S_E, the
target with w_{conj y} ≠ conj(w_y) is not reached. In the auxiliary ones it is the Vandermonde of
E[x]_{<T} over S, nothing more. ∎

**What the instrument measures.** `rango_f(matriz_de_lagrange(S, T))`, the rank over F of the
matrix (L_r(y))_{y∈S, T≤r<L}, is the rank of ev_S —the Lagrange matrix is that of ev_S composed
with the bijection above—, and the test requires it equal to |S_F| + 2|S_E|:

- over S_ab, `rango_pleno`: 46 of 46 in the 28 proofs (45 of 45 in the one that opened 41 unique
  positions; q + 4 ≤ T, with T ≥ 64);
- over S_toc, `rango_de_los_tocados`: 90 of 90 in the 19 proofs of the 18 families with T ≥ 256,
  and T of 2q + 6 —64 of 90, or 64 of 88 in the one that opened 41— in the 9 proofs of the 5
  families with T = 64 (NullifierThresholdAir, ThresholdAir, EdadAir with eight leaves, RefundAir,
  RefundAirV2), which the table `INDEPENDIENTE_DE_LA_AIR` declares and the test checks in both
  directions;
- the mutant T = 32 (`el_mutante_t_32_deja_las_columnas_sin_cota`): 46 against 32, rank 32;
- the proof of life (`el_rango_es_pleno_con_puntos_distintos_y_cae_con_uno_repetido`): with a
  repeated point the rank drops by one; 87 distinct touched points give 90 dimensions with
  T = 128 and 64 with T = 64.

**Hypotheses.** (H1, measured) the count, q + 4 ≤ T over S_ab and 2q + 6 ≤ T over S_toc, per
family, with the q read from each proof; the full rank the instrument computes is over positions
the test chooses and a fixed z, a proof of life of the lemma, not a measurement over the real
positions. (H2, proved) Λ ∩ H = ∅ and the positions are distinct (the prover opens unique ones).
(H3, assumed, with bounded failure probability) z ∉ F ∪ H and no conjugate of z, z·g, g²·z
coincides with another of the three: each coincidence is a non-trivial equation in z, of the form
z^{p−1} = g^e, with at most p − 1 solutions in E, so the total probability is less than 2^{−60}
per proof; the instrument uses a fixed z that satisfies them, and the real positions and z do not
travel (§896, «lo que NO hace» [what it does NOT do]). A coincidence between g·x_j and another
opened position reduces S_toc and does not get in the way: the simulator samples per distinct
point.

### L2 — The chunks open, at each point, a uniform vector of their hyperplane

**Statement.** Let S be a set of distinct points of E \ {0} with |S| ≤ m. With r_0 … r_{N−2}
uniform and independent in E[x]_{<m}, the vectors (h'_k(y))_{k<N}, y ∈ S, are independent of each
other, and each one is uniform in the affine hyperplane {v ∈ E^N : Σ_k y^{k·s}·v_k = C(y)},
whatever the h_k are.

**Proof.** For fixed y, the map (r_0(y), …, r_{N−2}(y)) ↦ (y^s·r_0(y),
y^s·r_1(y) − r_0(y), …, y^s·r_{N−2}(y) − r_{N−3}(y), −r_{N−2}(y)) is linear and injective
—triangular from the last coordinate— and its image lies in the hyperplane Σ_k y^{k·s}·u_k = 0,
because the sum telescopes; the hyperplane has dimension N − 1, that of the domain: it is the
whole hyperplane. And (r_k(y))_{y∈S} is uniform in E^S for each k, independent of the other k,
because the Vandermonde of E[x]_{<m} over S has rank |S| when |S| ≤ m and the points are
distinct. Adding h_k(y), fixed, translates the hyperplane to C(y). ∎

**What the instrument measures.** `rango(matriz_de_vandermonde(S_ab, m))` over E: 44 of 44 in
the 28 proofs (q + 2 ≤ m = 64); the mutant m = 43 of the toy
(`el_mutante_m_43_deja_los_trozos_sin_cota_y_sigue_verificando`): 44 against 43, rank 43, and the
proof still verifies (m decides hiding, not soundness, RFC-0009 D-H); and the tree's mutant,
`M_OCULTACION` 64 → 43, run in §897 with the corrected count: in red the 23 proofs that opened 42
unique positions —«chunks: 44 opened points, 44 dimensions over E, against m = 43» and rank 43—
and in green the four that opened 41 and the one that opened 40, as the inequality says. In a
first run the same mutant fell earlier at another gate, the byte band of D-AC: with m = 43 the
chunks are fewer and a RefundAirV2 proof with q = 41 slimmed below its band (38.595 B against
39.019), and `asierta` stopped the test before judging the bound. ⚠️ §896 counted
q + 2 + 1 ≤ m, put the 28 of its run in red and said that m = 43 fell with q ≥ 41; with the right
count, q + 2 ≤ m, m = 43 falls only with q = 42, m = 42 with q ≥ 41, and m ≤ 41 with every
production proof (q from 40 to 42). The instrument says it that way since §897.

**Hypotheses.** (H4, measured) q + 2 ≤ m and the full rank. (H5, proved) s = L − m ≥ 1 and N ≥ 2
(`segmentar` asserts it), y ≠ 0 (0 ∉ Λ; z = 0 has probability p^{−2}). What the lemma does NOT
say: nothing about the h_k or about C(y). What the verifier learns from the chunks at y is exactly
C(y), and L1 and L6 say what C(y) is.

### L3 — DEEP and FRI: what is determined, what is opened in excess and the condition R3

**What is determined.** D(x_j), at the q positions, is computed by the verifier from V2, V4 and V7:
it is not new information. And at x₀ = (z + z·g)/2: for any P, Q[P](x) = [P(x)·(2x − z − z·g) −
P(z)·(x − z·g) − P(z·g)·(x − z)] / ((x − z)(x − z·g)), and at x₀ the first term vanishes, so
Q[P](x₀) = 2·(P(z·g) − P(z))/(z·g − z): the slope between the two values of the frame. Hence D(x₀) =
2·[Σ_i cc_i·(P_i(z·g) − P_i(z)) + Σ_k cc'_k·(h'_k(z·g) − h'_k(z))] / (z·g − z) is public. ⚠️ §896
counted that direction as «one more opening that a simulator has to mask» (`DIRECCION_RESIDUAL`, the
note of D-I): it is not. The mask of the random column vanishes at x₀, true; but the value of D at
x₀ does not depend on the witness except through the OOD frame, which is already part of the view:
neither a leak nor an opening. The instrument no longer counts it.

**What is opened in excess.** FRI opens much more of D than D(x_j): at layer 0, the other seven
values of the folding coset of each position; at each following layer, the eight values of the
coset of the folded position, which are linear functionals of D —layer i is the fold of layer
i − 1 with α_i (`apply_drp`), a polynomial of degree < L/8^i—; and the whole remainder. Call O the
linear map that takes D ∈ E[x]_{≤L−2} to everything FRI opens of it, the D(x_j) included. Its
rank is not the number of opened values: with c₀ distinct cosets at layer 0 and c₁ at layer 1
(c₀ and c₁ ≤ q), layer 0 contributes 8c₀ dimensions over E; layer 1, at most L/8 —the dimension
of the space of its polynomial—, of which c₀ were already there (the fold of each opened coset);
and the remainder contributes nothing when the previous layer already determines it (8c₁ ≥ L/8,
always in the house). With q = 42 and distinct cosets: T = 64, 127 over E and 254 over F, all of D
—FRI hands over D whole—; T = 256, 358 and 716; T = 512, 422 and 844; T = 1024, 550 and 1100, of
the 1022, 2046 and 4094 dimensions that D measures. And what is left undetermined, rank_F(O|_K)
with K the kernel of the q + 1 conditions below, is the rank of O minus 2q and minus whatever
D(x₀) adds (0 or 2): about 168, 630, 758 and 1014. ⚠️ The first version of this document, in §897
and before its review, counted rows instead of ranks —704, 1276, 1292 and 1324— and said that
with T = 256 FRI handed over D whole; it is not so: 716 < 1022.

**The masks, and which ones count.** Of the three randomizers, only two enter D linearly and
independently of the witness: the random column, which is in no constraint and enters only
through cc_R·Q[R], and the quotient randomizers r_k, which enter through
Σ_k cc'_k·Q[x^s·r_k − r_{k−1}]. The random rows of the real columns do NOT: they enter D through
Q[P_i], linear, but also through the chunks h_k, which are those of
C(x) = Σ_c α_c·C_c(P(x), P(g·x))/Z_c(x), and C depends on the whole column polynomials —the
exemptions remove factors from the divisor, not terms from the numerator— with the degree of each
constraint: at the opened points it is not noticed, because C(y) reads only the touched frame and
L1 and L2 absorb it, but FRI opens D at the x_j·μ with μ ≠ 1 and at the layers, where the
h_k(x_j·μ) enter, which depend on the random rows polynomially and, in their linear part, on the
witness. The same for the auxiliary ones. So, conditioning the view on what is opened, what is
touched and the values of the r_k on S_ab, D = D_resto + M_R(ρ_R) + M_r(r_res): D_resto depends on
the witness, on the random rows of the real columns and on what is conditioned, in any way; ρ_R is
what is free of the random column, L − q − 4 dimensions over F (its L values minus the q + 4
opened ones); r_res, what is free of the randomizers, 2(m − q − 2) for each of the N − 1 (their m
coefficients of E minus their values at the q + 2 opened points). ⚠️ §898: that count of the
randomizers is that of their degrees of freedom, not of what reaches D: the N − 1 enter D through
two slots, Σ_k cc'_k·(x^s·r_k − r_{k−1}) = x^s·A − B with A and B in E[x]_{<m}, so their image in
D measures at most 2(m − q − 2) over E after what is opened, 40, that is 80 over F, with any
N ≥ 3 (40 with N = 2); measured (section 13). And the random column, at each point x of Λ,
contributes cc_R·Q[R](x) = R(x)·c(x) with R(x) ∈ F and c(x) ∈ E fixed: a single dimension over F
of the two that each value FRI opens has, however large L − q − 4 may be. M_R and M_r are linear
over F and injective (the kernel of P ↦ Q[P] is the constants, which the residual part does not
contain), independent of D_resto, and are 0 at x_j and at x₀, because the residual part does not
move the values on S_ab and Q[·](x₀) depends only on them: im(M_R), im(M_r) ⊆ K = {D : D(x_j) = 0
∀j, D(x₀) = 0}, and D_resto is in the affine space of D.

**The condition R3.** The real view of FRI is O(D_resto) + O(M_R ρ_R + M_r r_res); the
simulator's (section 5, S5) is uniform in O(D_0) + O(K), with D_0 any polynomial with the
determined values. If **rank_F(O∘[M_R | M_r]) = rank_F(O|_K)**, then O(M_R ρ_R + M_r r_res) is
uniform in O(K) and independent of D_resto, and the real view is uniform in O(D_resto) + O(K) =
O(D_0) + O(K): the same. R3 is **sufficient**, not necessary: if it fails, whatever is left
uncovered is moved by the random rows of the real columns in a way that depends on the AIR and on
the witness, as in L6, and no linear instrument measures it. ⚠️ The first version of this
document stated it as «if and only if», with the random rows of the real columns inside M; the
skeptical review of §897 corrected it (section 11, objections 1 and 2). It is a rank condition
over matrices built from the geometry (q, T, m, N, layers, remainder) and from the challenges
(positions, z, cc, α_i), like those of L1 and L2, and **nobody has measured it**: neither §896 nor
§897. The dimension counts, necessary and not sufficient, with q = 42: the masks add up to at
most (L − q − 4) + 40·(N − 1); with T = 1024 (L = 2048, N = 8), 2002 + 280 against about 1014, and
the random column alone reaches by count; with T = 512 (N = 8), 978 + 280 against about 758, and
the column alone as well; with T = 256, 466 + 320 = 786 against about 630 in MerkleAir and
FrozenClimbAir (N = 9), to be measured, and **466 + 40 = 506 against about 630 in SolvencyAir
(N = 2): R3 fails by count, and with q = 40 and repeated cosets it still fails**; with T = 64,
82 + 560 against about 168, but those five were already out by L1. ⚠️ §898: those counts were
upper bounds and the structure sinks them: measured, the masks reach 126, 378, 382, 510 and 766
against 168, 632, 632, 760 and 1016 (section 13), and R3 fails in the five geometries, not only
in solvency. ⚠️ What the note of D-I of §896 took for granted —«the random column … masks DEEP in
every direction but one»— is not true in general: the random column enters DEEP as cc_R·Q[R] with
R ∈ F[x]_{<L}, and Q[R] ranges over an image of dimension L − q − 4 over F inside a space of
2(L − 1); with T = 64 and T = 256 it cannot mask alone what FRI opens, and in all of them the
uniformity depends on the joint rank.

**Hypotheses.** (H6, **not measured**) R3, sufficient. It is the first thing A3 has to measure,
and section 6 says how; the matrices are small and it fits in the seal. If R3 fails in a family,
the argument of this document does NOT hold for it: in solvency it fails by count, and the way out
is that of entry 132, a DEEP mask of its own or more rows (section 10). ⚠️ §898: **measured, and
false in the five production geometries** (section 13): the argument of this document does not
hold today for any family, and the way out is entry 133.

### L4 — The salted leaves hide what is not opened and let what is opened be equivocated

**Statement** (programmable random oracle model, Blake3). With a fresh 256-bit salt per leaf:
(a) the digest Blake3(item ‖ salt) of an unopened leaf is indistinguishable from uniform for
whoever does not know the salt, unless they query the oracle at that (item ‖ salt), probability
2^{−256} per query; (b) for an opened leaf, a simulator that committed a uniform digest can
afterwards choose the item and a fresh salt and program Blake3(item ‖ salt) := that digest,
consistent except for that same probability. Hence the roots and paths of the trace, the chunks
and the FRI layers say nothing about the unopened leaves. The simulator of section 5 uses (a) and
not (b): it chooses the challenges beforehand and commits what it is going to open; (b) is what a
simulator that could not would use, and it is what A5 weighs when deciding on the salt. The FRI
remainder carries no salt: it travels whole, and its commitment Blake3(remainder) hides nothing
and does not have to.

**What the tree measures.** `crates/zk-ssl-air/src/sal.rs`:
`la_sal_pesa_treinta_y_dos_bytes_por_hoja_abierta`,
`la_sal_mueve_la_raiz_y_dos_compromisos_de_lo_mismo_difieren`,
`una_sal_ajena_o_un_item_ajeno_rechazan` and `una_sal_de_menos_falla_cerrada`; that the `VC` of the
23 provers and of the kit is `MerkleConSal` is required by the census of D-Z. **Hypotheses.** (H7,
model) Blake3 as a programmable random oracle. (H8, assumed, `SECURITY.md` §3.11) `OsRng` gives
fresh bytes: a salt repeated between two leaves of the same tree breaks (a) only for those two; no
test checks it nor can.

### L5 — The seeds and the coin: the randomness of the masks

The random rows and column and the randomizers are not uniform: they are outputs of
`DefaultRandomCoin<Blake3>` seeded with Blake3(seed), 32 bytes from `OsRng` per proof and per use
(rows and quotient, two seeds). In the model of L4, the outputs of the coin with a seed the
adversary has not queried are uniform and independent; a distinguisher's advantage is that of
guessing 256 bits, per query. The two uses have distinct seeds, so the rows and the randomizers
are independent of each other and of the salt. **What the tree measures.**
`crates/stark-experiment/src/falsadores_oculta.rs`: `la_semilla_de_produccion_son_32_bytes_frescos`
(§652) and `el_ultimo_byte_de_cada_semilla_mueve_la_prueba`, which changes byte 31 of each seed
and requires the proof to change. **Hypotheses.** (H7, model) as in L4. (H9, assumed) no seed is
repeated between two proofs: with the same rows seed, two proofs of two witnesses open
P(x_j) − P'(x_j) = P_real(x_j) − P'_real(x_j) at each common position, and the difference of the
witnesses comes out in the clear; it is the falsifier of A3. (H10, proved) the coin of the masks
shares no state with the public one: it is seeded apart, in the prover, and the public one does
not see it.

### L6 — What depends on the AIR, stated and not proved

For a family with 2q + 6 > T —the five with T = 64— L1 does not cover S_toc: the touched values of
a main column live in an affine variety of dimension T inside 2q + 6 over F, and C(x_j) and C(z·g)
depend on the witness through P(g·x_j) and P(g²·z) in a way the AIR decides. An argument for those
families would have to prove, AIR by AIR, that the distribution of (C(x_j))_j and C(z·g) given
what is opened does not depend on the witness —for example, because the map from the residual
part of (P(g·x_j))_j to (C(x_j))_j is surjective onto E^q for every real trace—, and that depends
on the algebra of its transitions. It is not attempted here, and the leak is not a scruple: in
the toy, whose transition is x_{i+1} = x_i³ + 42, C(x_j) gives P(g·x_j) by solving, and with
T = 64 the view depends on the witness —87 evaluations of a column of degree < 128 that 64 random
rows do not cover—; it is not a recovery by linear algebra (23 linear equations over 64 real
values are left, and the relation left over is cubic), it is a distribution that changes with the
witness, and the falsifier of A3 is a distinguisher (section 12). The ways out: L6 per AIR, or
T ≥ 2q + 6 = 90, that is T = 128 for the five —double the rows and the cost for them, in a future
cut, never in `zkssl/0.4`— (entry 131 of `BACKLOG.md`); age with 16 leaves or more (T = 8·2^m
with m ≥ 4) already meets the count, measured only in the bound: the instrument measures it with
eight leaves.
⚠️ §905: the other way out, the fine count for the linear columns (L1', section 15), is measured: it
rescues none of the five today; it would cover the joint threshold with entry 133.

## 4. The hypotheses, with their class

| hypothesis | class | lemma | what measures or sustains it |
|---|---|---|---|
| H1: q + 4 ≤ T over S_ab and 2q + 6 ≤ T over S_toc | measured, in every canon; the rank, a proof of life over the test's points | L1 | `rango_pleno`, `rango_de_los_tocados`, `INDEPENDIENTE_DE_LA_AIR`; the mutant T = 32; the proof of life |
| H2: Λ ∩ H = ∅, unique positions | proved | L1 | the order of 7; the verifier's `dedup` |
| H3: z outside F and H, no coincidences of conjugates | assumed, failure < 2^{−60} per proof | L1 | not measured: the positions and z do not travel |
| H4: q + 2 ≤ m, Vandermonde of full rank over E | measured, in every canon | L2 | `rango_pleno`; the mutant m = 43; the tree's mutant |
| H5: s ≥ 1, N ≥ 2, y ≠ 0 | proved | L2 | the asserts of `segmentar` |
| H6: R3, sufficient: the two witness-independent masks cover what FRI opens | **measured in §898: with M = {column, randomizers}, false in the five geometries**; ⚠️ §900: with M_lin (H12), true in sixteen families and false in MerkleAir and FrozenClimbAir (section 14) | L3 | `instrumento_r3`, in every canon, with its pinned figures (section 13); `instrumento_mlin`, the census in every canon and the measurement per family (section 14) ⚠️ §915: with `zkssl/0.5`, proved (L9, section 17) and also measured in the five production geometries, that of T = 128 included (`K_ENTERO_PAREADAS`, section 18) |
| H7: Blake3 as a programmable random oracle | model | L0, L4, L5 | — |
| H8: the salt is fresh | assumed | L4 | `SECURITY.md` §3.11; the tests of `sal.rs` measure size and effect, not freshness |
| H9: no seed is repeated | assumed; what it costs, measured in the toy (§898) | L5 | `instrumento_falsadores_hvzk`: two proofs with the same seed give their two constants |
| H10: the coin of the masks shares no state with the public one | proved | L5 | `generate_proof_oculto`, `segmentar` |
| H11: T ≥ 2q + 6 | measured: 18 families yes, 5 no; and what it costs, in the toy (§898) | L1, L6 | `INDEPENDIENTE_DE_LA_AIR`; with T = 64 what is touched tells the witness, with T = 128 it does not ⚠️ §915: measured, 23 families yes: T ≥ 128 ≥ 90 in all of them (`INDEPENDIENTE_DE_LA_AIR`) |
| H12 (§900): the random rows of a column that enters C linearly and with a public coefficient are one more mask, M_lin: uniform in a kernel that does not depend on the witness once conditioned on S_toc, with a linear and public image in D | measured: which columns are such, in every canon, AIR by AIR; and R3 with M_lin, per family; ⚠️ §907: proved as a lemma, with the joint kernel that discounts what is opened of the chunks (L7) | L3, L7 | `instrumento_mlin`: `CENSO_134` and `MLIN_MEDIDO` (section 14) ⚠️ §915: guard instrument; the proposition of section 18 does not need it |
| H13 (§905): Λ, what C sees on S_ab of the linear columns conditioned on what is seen, is surjective: C(S_ab) uniform and independent of the witness without going through S_toc | measured, per family, in those that do not fit with the coarse count: only in the joint threshold | L1' | `instrumento_mlin`: `sobreyectiva`, `FINA_MEDIDO` (section 15) ⚠️ §915: guard instrument; the proposition of section 18 does not need it |
| H14 (§907): R3 with M_lin holds on the real challenges, not only on the measured instance | proved on layer 0 with T ≥ 512 outside the coset–touched coincidences, for any positions, with failure ≤ 7c₀/\|F\| (L8); measured in the rest, on three instances per family —seed 2, seed 3 in solvency, and an adversarial one with a coset collision, a coincidence and a position that is the next of another—, with the fingerprint of each composition pinned | L7, L8 | `instrumento_mlin`: `adversa`, `MLIN_INSTANCIAS`, `HUELLA_C` (section 16) ⚠️ §915: guard instrument; the proposition of section 18 does not need it |

## 5. The simulator

S receives the AIR, the public inputs, the context (T, W, W_aux, m, N, options) and nothing of the
witness.

- **S1. The challenges.** It samples, uniform: the random elements of the auxiliary segment if
  there is one, the α of the constraints, z ∈ E (it samples again if it violates H3), the cc of
  DEEP, the α_i of FRI, and 42 positions of Λ with repetition, of which it takes the unique ones;
  the nonce, with the same search as the prover. In the non-interactive protocol, it programs the
  oracle so that the coin gives them (L0).
- **S2. The trace at the touched points.** For each real main column and each auxiliary one, it
  samples uniform its values on S_toc (F or E); for the random column, on S_ab. A point that
  appears twice is sampled once. Valid by L1 if 2q + 6 ≤ T (H11).
- **S3. The quotient at the opened points.** It computes C(y) for y ∈ S_ab with the verifier's
  public formulas (`evaluate_constraints`) over the frames of S2 —(y, g·y) for y = x_j, (z, z·g)
  and (z·g, g²·z)— and samples (h'_k(y))_k uniform in the hyperplane Σ_k y^{k·s}·v_k = C(y).
  Valid by L2 (H4).
- **S4. What is determined of DEEP.** d_j = D(x_j) with the formula of P7 over S2, S3 and the cc;
  d₀ = D(x₀) with the formula of L3.
- **S5. DEEP and FRI.** It samples D̃ uniform in {D ∈ E[x]_{≤L−2} : D(x_j) = d_j, D(x₀) = d₀},
  conditioned on degree exactly L − 2 as the prover asserts (difference 2^{−128}), and runs honest
  FRI over D̃ on Λ with the α_i of S1: layers, remainder, cosets. It coincides with the real view
  if R3 (H6), which is sufficient. ⚠️ §907 (O7 of the agent's review): the assert is
  `assert_eq!(trace_length - 2, deep_composition_poly.degree())` in `generate_proof_oculto`
  (`crates/winter-prover/src/lib.rs`), inherited from winterfell's `generate_proof`: with
  probability ≈ 2^{−128} a VALID witness makes the prover panic, against the letter of D-AH of
  RFC-0009 —it is not even a bad witness—; it does not touch hiding, and it is entry 137 of
  `BACKLOG.md`.
  ⚠️ §911: closed: it is `ProverError::GradoDeDeep`, an `Err` with a name, on both paths; and the
  event exists without waiting for 2^{−128}: a degenerate trace in the clear, of constant columns,
  always gives it, and a hidden one never, because the random rows are not constant.
- **S6. The commitments.** For the trace, the quotient and each FRI layer: a tree whose opened
  leaves are those of S2, S3 and S5 with fresh salt, and whose unopened leaves are uniform digests
  (L4). The FRI remainder, whole, is that of S5.
- **S7. The transcript.** V1 … V8 with the above. The verifier accepts it: the OOD consistency by
  S3, D(x_j) by S4, FRI by S5, the openings by S6, the grinding by S1.

**Why the distribution is the real one** (under H1 … H11): the challenges are uniform on both
sides; the touched values, uniform and independent per column (L1), and the real verifier sees
less than the simulator —the touched points that are not opened—, so matching the extended view
matches the view; the chunks, uniform in their hyperplane given C(y) (L2), with C(y) the same
function of the touched values on both sides; what FRI opens, uniform in the same affine space if
R3 (L3); the unopened leaves, indistinguishable (L4); and the real masks are outputs of the coin,
indistinguishable from uniform (L5). Everything but L1 and L2 rests on the model of L0 and L4:
the statement is one of **statistical hiding in the programmable random oracle model**, with
advantage bounded by the 2^{−60} of H3 plus the oracle queries times 2^{−256}, and
**computational outside it**, with Blake3 in the place of the oracle. ⚠️ §907 (O1 and O4 of the
agent's review): the whole count is in section 16: the 2^{−128} of S5 is added, the drop in rank
over the challenges is bounded with |F| in the denominator and not with |E|, and what depends on the
positions has a lemma (L8) on layer 0 with T ≥ 512 and a measurement, not a bound, in the rest.

## 6. The condition R3, as an instrument

**O**, rows: for each unique position x_j, the eight points of its folding coset in Λ,
{x_j·μ : μ^8 = 1} —D evaluated there: 8c₀ rows over E, of which those of the positions themselves
are what is determined—; for each layer i ≥ 1, the eight values of the coset of the folded
position, each a linear functional of D: winter-fri's fold with α_i applied to the cosets of the
previous layer (`apply_drp`); and the remainder, its coefficients, linear functionals of D (the
interpolation of the last layer). The rank of O is that of L3, not its number of rows. **M**,
columns, only the two sources that enter D linearly and independently of the witness: the random
column, cc_R·Q[L_r] for each of its L rows, restricted to the kernel of the evaluation on S_ab
(L − q − 4 columns over F after the change of basis); and each randomizer r_k,
(cc'_k·Q[x^{s+d}] − cc'_{k+1}·Q[x^d])_{d<m}, restricted to the kernel of the evaluation on S_ab
(2(m − q − 2) columns over F). The random rows of the real and auxiliary columns do not go in M
(L3). **The comparison**: rank_F(O∘M) against rank_F(O|_K), with a basis of K built from that of
E[x]_{≤L−2} with the q + 1 linear conditions. Equal: R3 holds for those challenges. A sufficient
shortcut: if rank_F(O∘M_R) = rank_F(O|_K) with M_R the random column alone, R3 holds by it alone;
with T ≥ 512 the dimension count allows it (978 against about 758; 2002 against about 1014), with
T = 256 it does not (466 against about 630). ⚠️ §898: the count allowed it and the structure did
not: measured, rank_F(O∘M_R) is 294 + 2·(L/8 − 42) —338, 466 and 722 with T = 256, 512 and
1024—, one dimension for each of the 7q points of layer 0 and two for each new functional of
layer 1, because R(x) ∈ F at each point of Λ.

As in L1 and L2, the rank for challenges the test chooses is the measured instance. That it holds
for the real ones: over z, cc and α_i, a drop in rank is a polynomial condition (a vanishing
minor) and its probability is bounded by the degree over |E|; over the positions it is not
polynomial, and the instrument has to try several choices and say so. ⚠️ §907: «over |E|» was an
error, and the agent's review took it as good (O4 of section 11): the map whose rank is measured is
F-linear —u and w live in F[x], because the random column and the real columns live in F— and its
matrix over F carries the two coordinates of each entry of E, so a minor is a polynomial in the
coordinates over F of the challenges and Schwartz–Zippel bounds its vanishing by degree/|F|, with
|F| = 2^64. For fixed positions at which the measured instance gives full rank, the whole minor
—size ≤ 1016, degree bounded by about ten per entry— gives something of the order of 2^{−51}; L8
(section 16) does it point by point and brings it down to 7c₀/|F| ≈ 2^{−55,8} on layer 0 with T ≥
512, for any positions. The denominator belongs to the construction, not to the argument: with the
DEEP mask in E (entry 133) everything would be E-linear and the bound would be over |E|. And over
the positions: measured on more instances, one adversarial, section 16. The cost is small: M has at
most L − q − 4 + 40·(N − 1) columns —2282 with T = 1024 and N = 8— and O at most 2·(8c₀ + 8c₁ +
remainder) rows over F, 1408; building O∘M and eliminating fits in the seal. What the instrument of
A3 has to print, per family: rank_F(O), rank_F(O|_K), rank_F(O∘M) and rank_F(O∘M_R), and the
verdict. ⚠️ §898: done, `crates/zk-ssl/src/instrumento_r3.rs`, with two challenge seeds that give
the same ranks, and in the seal per production geometry (section 13). ⚠️ §907: the second seed of T
≥ 256 was an exploration run that was not in the tree (O1); since §907 it is, with M_lin, in
`MLIN_INSTANCIAS` (section 16).

## 7. What the argument does not cover

- The five families with T = 64 (L6, H11): NullifierThresholdAir and ThresholdAir, which belong to
  the wire, RefundAir, RefundAirV2 and EdadAir with eight leaves. ⚠️ §914: there are no longer
  families with T = 64: the five go to 128 in the cut, and L1 covers them (H11 holds in the 23).
- Solvency (SolvencyAir, T = 256 with two chunks): R3 fails it by dimension count (L3), and it has
  no argument.
- R3 (H6), in the other seventeen, until A3 measures it. ⚠️ §898: measured, it fails in all of
  them: today the argument covers no family.
- The dependence of the quotient chunks on the random rows of the real columns outside the opened
  points (L3): it is neither linear nor independent of the witness, and this document does not
  use it as a mask; where R3 holds it does not get in the way, and where it fails it is what is
  left.
- Soundness: this document does not touch it; the hidden prover still proves what it said
  (RFC-0009 D-H, and the snapshot of D-R).
- What is public by design (P12) and what the node sees by being the node (RFC-0009,
  «Seguridad» [Security]).
- The repeated seed (H9), the repeated salt (H8), a predictable `OsRng`: outside the model,
  `SECURITY.md` §3.11.
- Blake3 outside the random oracle model, and a quantum adversary: «nivel cuántico: no medido»
  [quantum level: not measured], unchanged.
- The prover's side channels (time, memory) and whatever the layer, the SDK or the kit reveal
  outside the bytes of the proof.
- The 0.3 proofs (`spec/vectors/0.3/`), which do not hide, and the two of the rejection catalogue
  (RFC-0009 D-P).
- The real positions and z of each proof (H3 is assumed, not measured), and the real challenges
  of R3 (section 6). ⚠️ §907: for R3, L8 covers layer 0 with T ≥ 512 outside the coset–touched
  coincidences for any positions; layer 1, those coincidences and T = 256 remain in the measurement
  (section 16).
- The external audit of the fork and of the construction (H7 of `SECURITY.md`): this document asks
  for it, it does not replace it.

## 8. The statement

**Proposition (conditional).** Under H1 … H11 —in particular R3 (H6), sufficient and not measured,
and T ≥ 2q + 6 (H11)— there exists a simulator S, that of section 5, such that, for every real
trace the inner AIR accepts, the honest verifier's view of the hidden protocol and the output of S
are statistically indistinguishable in the programmable random oracle model, with advantage less
than 2^{−60} plus Q·2^{−256} for Q oracle queries. It can be stated, pending the measurement of
R3, for seventeen families. **For the five families with T = 64 it is not stated**: they lack L6
or T = 128. **Nor for solvency**: R3 fails it by count (L3, entry 132). ⚠️ §898: **with R3
measured and false in the five geometries, today it is not stated for any family**; it holds as a
statement of what the construction would have if the DEEP mask lived in E (entry 133), not of
what it has. ⚠️ §899: with the laboratory construction of entry 133 —two random columns—, R3
holds in the five geometries (section 13), and the proposition holds for it, under the other
hypotheses, for the eighteen families with T ≥ 2q + 6, solvency included; not for the five with
T = 64 (L1). It is what A4 will be able to state when the cut switches it on, and not before:
today no production proof carries it. ⚠️ §900: with R3 measured WITH M_lin (H12, section 14), the
proposition holds for today's construction, under the other hypotheses, for **sixteen families**:
the six with T = 1024, the nine with T = 512 and solvency; **not** for MerkleAir or FrozenClimbAir
(T = 256, with no linear column: their R3 is that of §898) nor for the five with T = 64 (L1). It is
what A4 can state, with these words and with the seven outside by name. **None of this is a
promise of the house**: A4 will decide, with A3 sealed, what is promised and for which families,
and until then the promise is the census. ⚠️ §907: the agent's review (section 11, the third;
`doc/ocultacion-hvzk-revision-agente.md`) found no false step and left the gap of its O1: what was
measured was one instance of challenges per family, and the real positions are not generic. With
§907 the proposition is stated with the count of section 16: advantage ≤ 2^{−60} (H3) + 2^{−128}
(S5) + Q·2^{−256} (L0) + ε_R3, with ε_R3 ≤ 7c₀/|F| ≈ 2^{−55,8} for what L8 proves —layer 0 with T ≥
512, outside the coset–touched coincidences— and, for the rest —layer 1, the coincidences and the
whole of solvency—, what the measured instances say, which is not a bound. It is still not a
promise.
⚠️ §915: the proposition for the twenty-three with the construction of `zkssl/0.5` —all of them,
solvency and the five with T = 128 included, with no exception by name— is that of section 18, with
the count of L9; what is above describes `zkssl/0.4`. It is still not a promise.

## 9. What A1 measured and what A2 corrects

| what §896 said | what it is | where |
|---|---|---|
| 45 to mask per column and per chunk: 44 seen plus one «residual direction» | per column, q + 4 = 46 dimensions over F; per chunk, q + 2 = 44 over E; the residual one is public, not an opening | L3; the instrument, `Aberturas`; the note of D-I |
| the random column masks DEEP in every direction but one | it masks an image of dimension L − q − 4 over F of a space of 2(L − 1); the uniformity of what FRI opens is R3, unmeasured | L3 |
| the tightest bound is T = 64, 45 against 64, nineteen of slack | the per-column count is that of what is touched, 2q + 6 = 90, and T = 64 does not meet it: five families with no AIR-independent argument | L1, L6, entry 131 |
| the rank of the Lagrange, over E | over F, because the main random rows live in F: 46 ≤ T, not 44 | the instrument, `rango_f` |
| m = 43 falls with q ≥ 41 | with q + 2 ≤ m, it falls only with q = 42, measured in §897 (23 red with q = 42, five green with q ≤ 41); m ≤ 41 falls with every production proof | L2 |

What §897 measured, with the corrected instrument (`cargo test -p zk-ssl --release --locked
instrumento_cota`): 28 proofs of 23 AIRs, 23 provers that switch it on; in all of them, m = 64 and
44 opened points per object with q = 42 (45 and 43 in the one that opened 41), 46 dimensions over
F per column against T from 64 to 1024, 44 over E per chunk against m = 64, full rank in both;
touched 90 against T: it covers, with rank 90 of 90, in the 19 proofs with T ≥ 256; it does NOT
cover, with rank 64 of 90, in the 9 with T = 64. Widths, with the random column: from 6
(SolvencyAir) to 84 + 6 auxiliary (EdadAir); chunks, 2 (SolvencyAir), 8 (T ≥ 512), 9 (MerkleAir,
FrozenClimbAir) and 15 (T = 64). The toy, T = 128: 46 against 128, 44 against 64, 90 touched
covered with rank 90 of 90, 4 chunks, one FRI layer and a remainder of 32, and it verifies; its
ranks are computed by the instrument in the production field, with its extension, over the toy's
geometry, which lives in f128 without extension —there z ∈ F and what is opened per column would
be q + 2—: it is a measurement of the geometry, not of the toy.

## 10. Where the fork departs from note 2024/1037

What the house knew of the note through RFC-0009 (D-G, D-I): the quotient is split with step
2T − m and its chunks are randomized with polynomials of degree < m that cancel in the sum (its
section 4.2, the only one the RFC cites), with m ≥ 44 because of «the 42 queries, z and z·g»,
which is the count of L2 over E; the random rows and the random polynomial before FRI come from
winterfell's issue 9, and so does the salt of the leaves. What a search of its page returns: that
it deals with the hiding of univariate arguments with FRI as commitment over small fields; that it
dwells on two techniques, «randomization with polynomials over the base field» and «splitting the
quotient into polynomials of lower degree»; that it revisits the 2019 construction it cites and
underlines its masking step; and that the decomposition of the quotient is a known source of
errors, in the literature and in code. With that, what can be said, and what cannot:

- **The quotient** (L2): the fork's construction is the one the RFC describes and the count
  q + 2 ≤ m is the one the note asks for according to the RFC. Pending: checking against the text
  whether the note also counts the chunks at z·g (`QuotientOodFrame`, which is winterfell 0.13's
  and not the issue's).
- **The DEEP mask** (L3): the fork masks with a random column of the trace, R ∈ F[x]_{<L}, which
  enters DEEP through the quotient, cc_R·Q[R]: L − q − 4 degrees of freedom over F. A random
  polynomial of the degree of D added to D —what issue 9 calls «the random polynomial before FRI»
  and what the note's abstract calls the masking step of the 2019 construction— would have
  2(L − 1) and would make R3 automatic. If the note builds the mask as a column of the base field,
  the count of R3 is its own too; if it adds it to D, the fork departs from it here, and R3 is the
  price, which in solvency is not paid: with two chunks, R3 fails it by count (L3). **Pending
  reading it.** ⚠️ §898: the price is not paid in any family: the mask in F against a DEEP in E
  leaves one coordinate of every two uncovered in everything FRI opens. That the note deals,
  according to its abstract, with «randomization with polynomials over the base field» makes
  reading it more urgent: it is exactly this.
- **What is touched** (L1): the count of D-I, 44 per column against T, did not count g·x_j or
  g²·z; if the note counts them, D-AG was decided with a count smaller than its own. Pending.
- **The salt** (L4): the issue asks for it on the trace; the fork puts it on the trace, the chunks
  and FRI. What the note asks for, pending.
- **The extension**: the fork draws z and the randomizers from E and the main random rows from F;
  the count over F of L1 (two dimensions per point of E) is a consequence of that, and whether
  the note does it, pending.

## 11. Objections and closures

The skeptical review of §897 —an agent of the session with the document, the fork, winter-fri and
the instrument in front of it, without the conclusions— raised eight objections, two high. Each
one, and how it ended:

1. **(high) The random rows of the real columns are not a linear mask of D.** They enter the
   quotient chunks through C, with the degree of each constraint, and their linear part depends
   on the witness; at the opened points L1 and L2 absorb it, at the x_j·μ and at the layers they
   do not. The first version counted them in M and stated R3 as «if and only if». **Closed**: M is
   the random column and the randomizers, R3 is sufficient and not necessary, and if it fails what
   is left depends on the AIR and on the witness (L3, section 6, section 7).
2. **(high) The rank of O is not its number of rows.** The layers fold: layer 1 contributes at
   most L/8 dimensions over E and the remainder none. With T = 256, 716 over F and not 1276, and
   FRI does not hand over D whole. With the right counts, solvency —T = 256 with two chunks— has
   506 dimensions of mask against about 630 undetermined: R3 fails it by count. **Closed**: ranks
   instead of rows in L3 and in section 6, the «against» redone, solvency out of the statement of
   section 8 and in entry 132.
3. **(medium) The header and the statement oversized.** «A condition» that was «if and only if»
   and «eighteen families». **Closed**: seventeen families, R3 sufficient, solvency apart, and the
   non-linearity of the chunks in section 7.
4. **(medium) The «recovery by linear algebra» of the toy with T = 64 was not justified.** Solving
   for P(g·x_j) from C(x_j) is valid, but 87 evaluations of a column of degree < 128 are 87
   equations over 128 unknowns, and the relation left over is cubic. **Closed**: the leak with
   T = 64 is distributional and the falsifier of A3 is a distinguisher; the linear recovery stays
   for T = 32 with a constant column (L6, section 12).
5. **(low) Grinding.** The search for the nonce runs over oracle outputs that the simulator does
   not fix; what is programmable are the ones after, from which the positions come. **Closed** in
   L0.
6. **(low) The ranks of the toy** are computed by the instrument in the production field, with its
   extension, and the toy lives in f128 without it. **Closed**: said in section 9.
7. **(low) «Measured» in H1** could be read as measured over the real positions. **Closed**: what
   is measured is the count; the rank is a proof of life over the test's points (L1, table).
8. **(low) A figure and a citation.** «46 of 46 in the 28 with q = 42» did not count the proof
   that opened 41, and section 11 referred to an entry that did not yet carry the objections.
   **Closed**: the figure, in L1; the objections, here and in the entry.

**The review of §898**, over the R3 instrument, section 13 and the falsifiers, with the code of
winter-fri and of the fork in front of it; it took as correct O (the exact map of the FRI prover),
the two masks, K, the comparison of ranks, the structural claim of one dimension per point and
the three falsifiers, and raised six objections:

1. **(high) M leaves out a linear mask that exists in solvency.** Objection 1 of §897 holds for
   columns that enter some constraint with degree ≥ 2 or with coefficients from the witness; in
   `SolvencyAir` four columns enter all of them linearly, with constant or periodic coefficients,
   and their random rows are a witness-independent linear mask that the instrument does not count
   and that can cover what is missing. **Closed**: the verdict is stated as «R3 with M = {column,
   randomizers} does not hold», the header of the instrument and this section say so, and
   measuring M_lin per AIR is entry 134. ⚠️ §900: measured (section 14): the mask exists in
   sixteen families and covers everything that is missing; in solvency, 632 of 632.
2. **(medium) Section 13 claimed more than what was measured**: «H6 is false, the simulator does
   not reproduce … in any family», and «exactly 250 are missing, the second coordinate», which is
   an inference. **Closed**: rewritten as above, and the inference said as an inference.
3. **(medium) What was sealed was a single instance of challenges**, seed 1 and q = 42, while the
   text said «two seeds» and «q = 40 and 41». **Closed**: in the seal, the geometry of T = 64 also
   runs with q = 40 and 41 and with seed 2; the rest is said as an exploration run, outside the
   tree. And «the positions and z do not travel» was inexact: they are not rebuilt in the bound
   instrument, which does not have the AIR; in the falsifiers they are.
4. **(low) No inclusion self-check** in `mide`. **Closed**: three asserts, each mask inside the
   two together and the two inside K.
5. **(low) The header of the falsifiers said 92 equations** and they are 91. **Closed**.
6. **(low) What validates the rebuilt coin** is not the assert of unique positions but the
   consistency of the first attack, and `siguiente_de` divides by a slope that would be 0 with a
   secret constant 0. **Closed**: said in the header and in section 13, and an assert on the
   slope.

**The review of §907**, by an agent external to the session —another instance, without the chronicle
or the conclusions, with the dossier, a copy of the tree of §903 (`git archive`) and the sources of
winter-fri and winter-crypto 0.13.1 from the cargo registry in front of it, read-only, executing
nothing; the whole report, with what it read and in what order, what it took as correct and with
what, and what the dossier did not let it answer, is `doc/ocultacion-hvzk-revision-agente.md`—. It
is not the external review of entry 136, which is by a person the author chooses; it is the third of
the house, further from the session than those of §897 and §898. It found no false step nor a
measurement that measures something other than what it says, and raised seven objections, none high:

1. **(medium) R3 with M_lin was measured on ONE instance of challenges per family, and the real
   positions are not generic in the sense the argument needs.** The 42 positions fall in |Λ|/8
   folding cosets: two share a coset at layer 0 in 57 % of the proofs of solvency (34 % with T =
   512, 19 % with T = 1024); with frequency Θ(1) per proof a point of the coset of one position
   coincides with a touched point of another, x_j·μ = g·x_k, and there the direct path of every
   linear column is zero; the pinned instance had 42 distinct cosets, and the bound of the
   proposition had no term for the positions. **Half closed** (section 16): solvency is measured in
   every canon with seeds 2 and 3 whole and with an adversarial instance that forces a collision, a
   coincidence and a position that is the next of another; the other fifteen, with seed 2 and the
   adversarial one, in the long test; L8 proves the coverage of layer 0 point by point for any
   positions with T ≥ 512, outside the coincidences; layer 1, the coincidences and T = 256 remain
   measured and not bounded, and the proposition says so (section 8).
2. **(medium) H12 was applied as the instrument computes it, but the lemma that justifies it was not
   written**: the random rows of a linear column move the openings of the chunks, and «one more mask
   of L3» did not say why conditioning on those openings does not break independence; a reader who
   checks M_lin against the definition of L3 rejects it by objection 1 of §897. **Closed**: L7, the
   joint kernel, section 16.
3. **(medium) What the canon measured of entry 134 was the census and solvency; the rank of the
   other fifteen families was a snapshot**: a change in the constraints of an AIR that kept the
   counts of the census moved no guard. **Closed**: the fingerprint of the composition of each
   family, pinned (`HUELLA_C`) and compared in every canon; a change forces remeasuring entry 134
   (section 16).
4. **(low) The count of the advantage was incomplete**: the 2^{−128} of S5 was not added, the degree
   of the minor was not stated and the positions had no term. **Closed** (section 16), and with a
   correction the review did not see: the bound over the challenges goes over |F|, not over |E|
   (section 6).
5. **(low) Two imprecisions of the view**: the salted leaf is `merge(hash_elements(fila), sal)`, not
   Blake3(row ‖ salt); and the commitment of the remainder and `num_unique_queries` were not in
   V1–V8. **Closed**, in P5 and in section 2.
6. **(low) L0 described the grinding with the `concurrent` feature switched off, without saying
   so.** **Closed**, in L0.
7. **(low) The degree of DEEP is asserted with a panic, in production**: with probability ≈ 2^{−128}
   a valid witness makes the prover panic, against D-AH. **Half closed**: said in S5, and entry 137
   of `BACKLOG.md` for the `Err`, in a separate seal, because it touches the fork's production path.

And two things it saw while reading, outside the dossier: that an `Assertion::periodic` of an inner
AIR would also be enforced on the random rows —today none of the 23 uses one; the hidden prover
would fail at the assert of `segmentar`, without leaking anything—, which is entry 138; and that the
exploration run of §898 with seed 2 for T ≥ 256 was not in the tree: since §907 the second seed is,
with M_lin, in `MLIN_INSTANCIAS`. On entry 135, by count and without measuring, it said that at most
it would rescue one of the five, and pointed to delegated authorization; §905 measured that it
rescues none, and that the only one with a surjective Λ is the joint threshold, which the agent's
count ruled out: the count is not enough, the rank of Λ decides.

What the review of §907 took as correct, with the code in front of it: P1, P2, P4, P5/L4, P6/P11,
P7, P8/P10/V8 and P9/L0 against the sources of the fork and of winter-fri; the whole view against
the `Proof`; L3, with a closed argument that D(x₀) is the only public thing of D outside the
positions —the residue of a single chunk ranges over the whole of K—; the rank of O;
`instrumento_r3` against section 6, with the formulas of the two masks deduced separately and
coinciding; of `instrumento_mlin`, that the composition is line by line that of the verifier and not
a replica of the prover, the probe, the construction of M_lin, the joint kernel and the inclusion;
that in the sixteen AIRs the form of C does not depend on the public inputs; the falsifiers and the
rebuilt coin; L5; the count of what is touched; and the order of the simulator against Fiat–Shamir,
prefix by prefix.

What the review of §897 took as correct, with the code in front of it: the order of the
commitments and of the challenges, the two coins and their seeds (P1, P4, P6, P7); `segmentar` and
the number of chunks; the whole proof of L1, with the exemptions on the last rows; L2; Q[P](x₀) and
that D(x₀) is public; that K is the correct space and that the two legitimate masks fall in it;
what FRI opens and how; the salt and the tests cited; the simulator, step by step, and that
nothing of the `Proof` is left unsimulated except what objection 1 covers; and that the document
claims HVZK nowhere.

## 12. What it asks of A3, A4 and A5

- **A3** (⚠️ §898: done, section 13; what follows is what it asked), in this order: (1) the R3
  instrument of section 6, per family, in the seal or in `--largo` with its pin; (2) the falsifier
  of the recovery by linear algebra, in the toy: with T = 32 and a constant column, the q + 4
  equations of L1 over T + 1 unknowns return the constant; and for entry 131, with T = 64 in the
  toy, a distinguisher and not a recovery: from C(x_j) one solves for P(g·x_j), as L6 says, but
  the 2q + 3 evaluations of a column of degree < 2T are 87 linear equations over 128 unknowns, and
  the relation left over, x_{i+1} = x_i³ + 42, is cubic; what is measured is that the view changes
  with the witness, not that the witness comes out; (3) the seed repeated in two proofs, in the
  seal (H9); (4) the statistical distinguisher only if it fits in `--largo` with its pin, and if
  not, declared out.
- **A4**: the new promise only with R3 measured green and with the five families of entry 131
  said apart; the words, those of section 8 and no more; H7 kept until an external review;
  «nivel cuántico: no medido» [quantum level: not measured], intact. ⚠️ §898: R3 is measured and
  red in all of them: A4 cannot state HVZK for any family with the argument of this document. What
  A4 can do is say that with the words of this document —the census goes on, and what is missing
  and why— and leave the HVZK promise for what entry 134 measures AIR by AIR or for the cut that
  carries entry 133. ⚠️ §899: entry 133 is measured and built in the laboratory (section 13): A4
  can say what the construction of the cut will have, with R3 green, and what today's has,
  without it. ⚠️ §900: entry 134 is measured (section 14): with today's construction, R3 holds
  with M_lin in sixteen families and not in seven, by name; A4 can state for the sixteen what this
  document proves, with its hypotheses, and say of the seven what they lack: entry 133 for
  MerkleAir and FrozenClimbAir, and entry 131 —or 135— for the five with T = 64.
  ⚠️ §915: the cut is sealed (§914): A4 can state for the twenty-three what section 18 proves with
  the construction of `zkssl/0.5`, with its hypotheses and with no family outside by name; the words
  are in D-AM of RFC-0009 (the draft of §915), and the promise is still the census until the author
  accepts it.
- **A5**: the salt is what makes L4 unconditional. Without it, (a) needs every unopened leaf to
  have entropy against the oracle —the whole row of the trace at a point of Λ has it by L1, the
  vector of the chunks by L2, and the eight values of a FRI coset only by R3— and (b) disappears.
  This document's recommendation to the author is to keep it until R3 is measured, and to decide
  afterwards with the leaf count in front of him. ⚠️ §902: decided by the author: it is kept.

## 13. A3, measured (§898)

**The R3 instrument** (`crates/zk-ssl/src/instrumento_r3.rs`) builds, for a geometry
(q, T, m, N, layers, remainder) and challenges from a Blake3 coin seeded with a number, what
section 6 asks for: O with winter-fri's fold (`apply_drp`, `fold_positions`, the same offset), K,
M_R and M_r with their formulas of L3, and the ranks over F by elimination, over random
combinations with more rows than O (Schwartz–Zippel over 2^64), and with the inclusion asserts
the construction promises (each mask inside the two together, and the two inside K). It runs in
the seal, inside `la_cota_de_ocultacion_de_los_23_probadores_con_fila`, once per production
geometry with q = 42 and seed 1, against its pinned figures (`R3_MEDIDO`), and in the geometry of
T = 64, in addition, with q = 40 and 41 and with seed 2, with the same verdict. The other seeds
and queries of the other geometries —the same ranks with seed 2— and the rank of the whole O,
which informs and does not decide, were measured in the exploration run of §898 and are not in
the tree.

| geometry (T, chunks, layers, remainder) | families | rank of O (E) | undetermined, O over K (F) | column | randomizers | together | R3 |
|---|---|---|---|---|---|---|---|
| 64, 15, 1, 16 | the five with T = 64 | 127 (D whole) | 168 | 82 | 80 | 126 | no |
| 256, 2, 2, 8 | SolvencyAir | 358 | 632 | 338 | 40 | 378 | no |
| 256, 9, 2, 8 | MerkleAir, FrozenClimbAir | 358 | 632 | 338 | 80 | 382 | no |
| 512, 8, 2, 16 | the nine with T = 512 | 422 | 760 | 466 | 80 | 510 | no |
| 1024, 8, 2, 32 | the six with T = 1024 | 550 | 1016 | 722 | 80 | 766 | no |
| 128, 4, 1, 32 (the toy) | — | 255 | 424 | 210 | 80 | 254 | no |

Everything over F except the column of the rank of O. With q = 41 (T = 64): 170 against 129. And
the proof of life: with four queries in the toy's geometry, 112 against 112, R3 holds; the
instrument knows how to say yes.

**Why it fails, read from the figures.** The rank of O is that of L3 corrected by the review
(objection 2): 8c₀ + L/8 − c₀ over E. What the random column covers is 294 + 2·(L/8 − 42):
**one** dimension for each of the 7q = 294 points of layer 0 that are not positions, and two for
each new functional of layer 1. It is structural: the random column is a polynomial of F[x], and
at a point x of Λ ⊂ F its contribution to D is R(x)·c(x), a multiple in F of the fixed number
c(x) = cc_R·ℓ(x)/((x − z)(x − z·g)) ∈ E; the value D(x) that FRI opens has two coordinates over F,
and the column moves only one. The quotient randomizers cover at most 80: they all enter through
the two slots x^s·A − B. With T ≥ 256, 250 = 294 − 44 dimensions are missing in the three
geometries; that they are the second coordinate of the 294 points of layer 0 minus what the
randomizers reach is an inference consistent with the figures (632 = 2·294 + 44,
column = 294 + 44), not a measurement: the instrument measures ranks, not the subspace that is
missing. With T = 64, FRI hands over D whole and 42 are missing. ⚠️ Where this document said, in
L3 and in section 6, that «with T ≥ 512 the random column alone reaches by count», it counted
dimensions and not structure: the count was an upper bound.

**What it means**, with the precision the review of §898 put in (section 11): what is measured is
that **R3 with M = {random column, randomizers} does not hold** in any production geometry, and
therefore the argument of this document does not establish the simulability of what FRI opens for
any family. It does not establish the opposite: the condition is sufficient, and what is left
uncovered is moved by the random rows of the real columns. In an AIR whose constraints touch a
column with degree ≥ 2 or with coefficients that depend on the witness, that contribution is not
linear (objection 1 of §897); but in an AIR that touches some column only linearly and with
public coefficients —solvency, where four of its columns enter all its constraints that way—, the
random rows of that column are one more linear mask, independent of the witness, M_lin, which
this instrument does not count and which can cover what is missing: measuring it, AIR by AIR, is
entry 134. What can be bounded by count: the direct DEEP part of all the columns over F, modulo
the direction the random column already covers, fits in a single polynomial of Z_real·F[x]_{<T}
and gives at most T − 46 dimensions —210 with T = 256, fewer than the 250 that are missing—, so
with T = 256 only the way through the chunks can close the gap, and with T ≥ 512 not even that is
excluded by count. Solvency (entry 132) and the five with T = 64 (entry 131, which also fail by
L1) remain as they were. The AIR-independent way out the figures point to is a DEEP mask that
lives in E: the random column in the auxiliary segment, which winterfell carries over E, or a
random polynomial of E[x] of the degree of D added to D, the «random polynomial before FRI» of
issue 9; it is entry 133 of `BACKLOG.md`, of a future cut and never of `zkssl/0.4`, and it is the
author's; entry 134 says first which families need it.

**The falsifiers of the toy** (`crates/zk-ssl/src/instrumento_falsadores_hvzk.rs`): `ConstanteAir`,
a secret constant column with a transition of degree 3 and a public column with two assertions,
in the production field with its extension and with the queries, the blowup and the folding of
`proof_options()`, m = 64 —half when T = 32, which admits no more—, without grinding or salt. The
adversary is the test: it rebuilds the public coin from the bytes of the proof exactly as the
verifier does —the context and the public inputs, the roots, the constraint coefficients, z, the
OOD frame, the DEEP coefficients, one α per FRI commitment, the nonce and the positions— and
reads what is opened. What is measured, in the seal:

1. **The trace bound has teeth** (L1). With T = 32: 45 equations over F (41 unique positions, z
   and z·g) over 33 unknowns —the constant and the 32 coefficients of R, with the column written
   as c·I_real + Z_real·R, I_real the indicator of the real rows—, and the constant comes out by
   elimination. With T = 64: 46 over 65, and it does not come out.
2. **What is touched tells the witness** (entry 131). With T = 64 and two candidates, the 85 to 90
   touched values —the positions, their next rows solved from C(x_j), z, z·g and g²·z solved from
   C(z·g)— are consistent with `Z_real·F[x]_{<64}` for the right candidate and not for the wrong
   one: the view tells the witness, in both proofs. With T = 128, both candidates are consistent:
   it tells nothing. It is not a recovery: it is that the distribution changes with the witness,
   as the review said.
3. **The repeated seed** (H9). Two proofs of two constants with the same rows seed share R: 91
   equations over 66 unknowns return the two constants with T = 64, where each proof alone (46
   over 65) returns none. With distinct seeds, 91 over 130: none.

**The statistical distinguisher** is declared out: the one above is algebraic and decides with
certainty, and for what FRI opens the question is not statistical but one of rank, and it is
measured. What validates the rebuilt coin of the falsifiers is not the assert of unique
positions, which is weak, but the first attack: 45 equations over 33 unknowns are consistent only
if z and the positions are those of the proof.

**What A3 does not do.** It attacks no proof of the house nor any AIR of the wire: the toy is the
toy, and what it says of the real families is what their geometries say. It does not touch the
promise. It does not decide entry 131, 132 or 133. It does not read the note.

**§899: entry 133, measured and built in the laboratory.** The R3 instrument also measures other
masks (`Mascaras`): k random columns over F, each with its DEEP coefficient in E, and one random
column over E, whose residual image is exactly K by construction
(V·(x − z)(x − z·g)·E[x]_{<L−q−2} ↦ cc·V·ℓ·v). What is measured, over F, with seed 1:

| geometry (T, chunks, layers, remainder) | undetermined | two columns over F: columns / together | three: columns / together | one over E: column |
|---|---|---|---|---|
| 64, 15, 1, 16 | 168 | 164 / **168** | 164 / 168 | 168 |
| 256, 2, 2, 8 | 632 | 632 / **632** | 632 / 632 | 632 |
| 256, 9, 2, 8 | 632 | 632 / **632** | 632 / 632 | 632 |
| 512, 8, 2, 16 | 760 | 760 / **760** | 760 / 760 | 760 |
| 1024, 8, 2, 32 | 1016 | 1016 / **1016** | 1016 / 1016 | 1016 |
| 128, 4, 1, 32 (the toy) | 424 | 420 / **424** | 420 / 424 | 424 |

Two columns over F suffice in all of them: with T ≥ 256 they cover alone what FRI opens
undetermined, and with T = 64 and in the toy they lack four dimensions that the randomizers
supply. Three add nothing. The column over E covers it alone, as its construction says. The
choice is the two columns over F (RFC-0009 D-AL): it does not touch the auxiliary segment, which
winterfell admits only with a constraint and an assertion of its own, and it is a parameter,
`Ocultacion::columnas`. What is in the tree: with `columnas: 1`, the construction of `zkssl/0.4`
byte for byte; with 2, the laboratory one, with the mark `arqueo:oculta:lab` that `verify`
rejects and only `verify_laboratorio` accepts; a falsifier in `falsadores_oculta` measures it, and
the bound of the two columns is pinned per geometry in every canon (`R3_MEDIDO_LABORATORIO`),
next to that of one. What does not change: L1 and entry 131 (the five with T = 64 are still out),
L6 and entry 134, and the promise. What entry 133 leaves to the 0.5 cut: switching on the two
columns in the 23 provers with the columns in the v2 mark (RFC-0018 E5).

## 14. Entry 134, measured (§900)

**What it measures.** §898 measured R3 with the two masks that do not depend on the AIR and §897
left an objection open (section 11, the 1 of §898): the random rows of a real column whose
constraints touch it only linearly and with public coefficients —constant, periodic, or the
random elements of the auxiliary segment, which are the verifier's— are one more linear mask,
M_lin, independent of the witness. `crates/zk-ssl/src/instrumento_mlin.rs` measures it, over each
hidden AIR without knowing its type: `Sonda`, which `Oculta<A>` implements for every `A`, gives
the widths, the chunks of the context, the periodic columns and the composition C at a point with
a frame, computed as the prover computes it —the combination of the transitions, main and
auxiliary, over their divisor, plus each group of assertions over its own—. Two steps:

1. **The probe.** For each real column, at three random points x of E, with two random frames, f
   and f', and a displacement δ, for the current row and for the next: C is affine in the input
   if C(f + 2δe) − C(f) = 2·(C(f + δe) − C(f)), and the slope is public if at f' it is the same as
   at f. Three verdicts per column: linear with a public coefficient; non-linear, degree ≥ 2 in
   it; linear with a slope from the witness, another column multiplies it. Schwartz-Zippel over
   E: that a non-linear one passes as linear at the three points is an event of the order of
   2^{−120}. In a toy of four columns —degree 3; a transport; linear with the slope of the first;
   linear with a periodic slope— the probe gives exactly that, with two seeds.
2. **M_lin, and R3 with it.** Of each linear column what is left free, after conditioning it on
   the 2q + 6 touched points of L1, is P_ρ = Z_real·V_toc·w —Z_real vanishing on the T real rows,
   V_toc on the touched points and their conjugates, w ∈ F[x] of degree < T − 2q − 6—, and it
   enters D through two paths: the direct one, cc_i·Q[P_ρ], and the quotient one, C(P_ρ) − C(0),
   linear in P_ρ —the slope of each column at each point of the constraint domain,
   precomputed—, chunked like `segmentar` with step s = L − m and each chunk with its DEEP
   coefficient. What the verifier sees of the chunks —each one at the q positions, at z and at
   z·g— is discounted by the JOINT kernel with the randomizers:
   rank(O∘M) = rank([opened | O∘D]) − rank(opened), over random combinations of the whole
   parameter space, more than its rank can be; with the same O, the same K and the same
   challenges as the R3 instrument. Two checks inside the measurement: that C(P_ρ) fits in the N
   chunks of step s —which is how one notices that the numerator vanishes where the divisor does;
   if not, the interpolant would not be a polynomial—, and that the masks do not leave K, by
   ranks: that of the masks with K, minus that of the opened ones, is that of K.

**How it is validated.** Without linear columns the measurement reproduces exactly what §898
pinned —in the toy, 424 undetermined, 254 of mask and 80 of randomizers alone; in solvency, in
every canon, 632 and 378—: the same O, the same K and the same subspace, with another sampling
and the joint kernel instead of the residue per randomizer. With the two linear ones of the toy,
38 free each (T − 2q − 6 with T = 128), the mask grows by exactly its 76 dimensions, inside K:
156 = 80 + 76 with the randomizers, 330 = 254 + 76 all three.

**The census**, measured in every canon over the 23 families (`CENSO_134`), with the trace and
the options of a real proof of each —with m = 64—, the probe with two seeds and empty public
inputs, which no AIR looks at when it is built except age, which goes with those of eight leaves:

| AIR | T | real | linear, public coefficient | linear, slope from the witness | non-linear | free per linear (T − 2q − 6) | verdict |
|---|---|---|---|---|---|---|---|
| SendAir | 1024 | 58 | 30 | 0 | 28 | 934 | holds |
| SendV2Air | 1024 | 62 | 34 | 0 | 28 | 934 | holds |
| ClaimAir | 1024 | 57 | 29 | 0 | 28 | 934 | holds |
| ClaimAirV2 | 1024 | 61 | 33 | 0 | 28 | 934 | holds |
| BurnAir | 1024 | 48 | 21 | 0 | 27 | 934 | holds |
| DoubleEntryAir | 1024 | 41 | 15 | 0 | 26 | 934 | holds |
| PrendaAir | 512 | 42 | 17 | 0 | 25 | 422 | holds |
| MintPendingClimbAir | 512 | 39 | 13 | 0 | 26 | 422 | holds |
| AuditAir | 512 | 31 | 17 | 0 | 14 | 422 | holds |
| BandaAir | 512 | 27 | 13 | 0 | 14 | 422 | holds |
| CobroPendienteAir | 512 | 44 | 18 | 0 | 26 | 422 | holds |
| PagoEnCursoAir | 512 | 44 | 18 | 0 | 26 | 422 | holds |
| CreditClimbAir | 512 | 39 | 13 | 0 | 26 | 422 | holds |
| MintClimbAir | 512 | 42 | 16 | 0 | 26 | 422 | holds |
| RecoveryClimbAir | 512 | 43 | 17 | 0 | 26 | 422 | holds |
| SolvencyAir | 256 | 5 | 4 | 0 | 1 | 166 | holds |
| MerkleAir | 256 | 13 | 0 | 0 | 13 | 166 | no linear ones: needs entry 133 |
| FrozenClimbAir | 256 | 25 | 0 | 0 | 25 | 166 | no linear ones: needs entry 133 |
| NullifierThresholdAir | 64 | 21 | 8 | 0 | 13 | 0 | does not fit: entry 131 first |
| ThresholdAir | 64 | 34 | 7 | 0 | 27 | 0 | does not fit: entry 131 first |
| EdadAir | 64 | 83 + 6 aux | 4 | 7 | 72 | 0 | does not fit: entry 131 first |
| RefundAir | 64 | 12 | 0 | 0 | 12 | 0 | does not fit: entry 131 first |
| RefundAirV2 | 64 | 16 | 4 | 0 | 12 | 0 | does not fit: entry 131 first |

In no AIR without an auxiliary segment is there a linear column with a slope from the witness:
the columns either enter with degree ≥ 2 —those of the hashes, the bits of the ranges— or enter
linearly with constant or periodic coefficients —the transports of roots, amounts and counters—.
In age, seven main columns enter linearly with the slope of another, because of the reads of the
auxiliary segment.

**What is measured** (`MLIN_MEDIDO`; solvency in every canon, the others in the long test
`mide_m_lin_en_las_23_familias`, once, with seed 1; ⚠️ §907: and with seed 2 and an adversarial
instance, section 16, with the fingerprint of the composition of each family pinned, which forces
remeasuring if an AIR changes), over F:

| AIR | T, chunks | undetermined (O over K) | §898: column and randomizers | linear ones and randomizers | all three | R3 with M_lin |
|---|---|---|---|---|---|---|
| SendAir, SendV2Air, ClaimAir, ClaimAirV2, BurnAir, DoubleEntryAir | 1024, 8 | 1016 | 766 | **1016** | 1016 | holds |
| PrendaAir, MintPendingClimbAir, AuditAir, BandaAir, CobroPendienteAir, PagoEnCursoAir, CreditClimbAir, MintClimbAir, RecoveryClimbAir | 512, 8 | 760 | 510 | **760** | 760 | holds |
| SolvencyAir | 256, 2 | 632 | 378 | **632** | 632 | holds |
| MerkleAir, FrozenClimbAir | 256, 9 | 632 | 382 | — (no linear ones) | 382 | does not hold (§898) |

The times, in the session's container: from 15 s (solvency) to 314 s (SendV2Air) per family,
with the eliminations over F as the dominant cost.

**What it says.** Sixteen of the twenty-three families meet R3 with today's construction once
M_lin is counted: all those with T ≥ 512 and solvency. In all of them, the linear ones with the
randomizers reach K without the random column, and the reason is the one §898 pointed out the
other way round: each linear column enters D with its own DEEP coefficient in E through the
direct path and with those of the chunks through the quotient one —distinct directions of E at
each point FRI opens—, while the random column alone, with a single coefficient, moved one
coordinate of two. MerkleAir and FrozenClimbAir have no linear column —Rescue's sbox of degree 7
touches all of them, as entry 134 foresaw—: their R3 is that of §898, 382 of 632, and **they need
entry 133**. The five with T = 64 have no M_lin to measure, T − 2q − 6 < 0, and stay in entry
131; they have 8, 7, 4, 0 and 4 linear columns, which the fine count of entry 135 could exploit
without more rows, with a lemma nobody has written. **Which families need entry 133 for R3**:
seven, MerkleAir, FrozenClimbAir and the five with T = 64 (these, entry 131 first); the other
sixteen, no. Of the seven, five belong to the wire —delegated authorization, age, the two refunds
and the delegated freeze climb—; the joint threshold and the Merkle path do not.

**What holds.** L3 does not change: K, O, what is public and the condition are the same; M grows
by one more mask that meets what L3 asks of a mask —the random rows of a linear column,
conditioned on S_toc, are uniform in the kernel of the evaluation on S_toc, which does not depend
on the witness, and their image in D is linear and public— (⚠️ §907: and the conditioning on the
openings of the chunks, which M_lin moves, does not break the independence: L7, section 16), and
step S5 of the simulator of section 5 —D̃ uniform in the class of K— is justified by R3 with it, as
L3 says. With L1 (the
eighteen with T ≥ 256) and the other hypotheses, the proposition of section 8 holds for the
sixteen with today's construction. R3 with M_lin is sufficient, not necessary: the instrument
conditions each linear column on its 2q + 6 touched points, more than what the verifier sees of
it (entry 135), so a «does not hold» is not a counterexample; for MerkleAir and FrozenClimbAir
the fine count is not needed: they have no column to count.
⚠️ §905: the fine count is measured (section 15): L1' holds as a lemma and does not change the
census today.

**What it does not do.** It does not count the random rows of the auxiliary segment (only age has
one, with T = 64). It does not measure the challenges of any real proof: the geometry and the
options are those of one proof of each family, and the challenges are chosen by the test, as in §898
(⚠️ §907: three instances per family, one adversarial, and L8 for layer 0 with T ≥ 512, section 16).
It does not touch a byte of production: the instrument is tests only. It is not a promise:
what A4 states, A4 states.

## 15. Entry 135, measured (§905): the fine count rescues none of the five

### L1' — The fine count for the linear columns: what is seen, not what is touched (§905)

**What L1 counts in excess.** L1 asks that the T random rows of a column cover the 2q + 3 points the
proof touches of it —the q positions, their q next ones, z, z·g and g²·z, 2q + 6 dimensions over F—,
because C(x_j) reads P(g·x_j) and C(z·g) reads P(g²·z). But the verifier does not see P(g·x_j) or
P(g²·z): it sees P on S_ab —q + 4 dimensions over F per column— and it sees C on S_ab, a sum of all
the columns with q + 2 values of E in total, 2q + 2 dimensions over F that enter the OOD frame and
the openings of the chunks (and C(z), which reads P(z) and P(z·g), is a function of what is seen).
For a column that enters C with degree ≥ 2 that does not help: its part in C(x_j) depends on the
witness in a way the AIR decides (L6). For the linear columns with a public coefficient, it does.

**Statement.** Let Lin be the set of main columns that enter C linearly with a public coefficient
(H12). For each i ∈ Lin, what is left free of its random rows once conditioned on what is SEEN of it
is P_ρ,i = Z_real·V_ab·w_i, with V_ab vanishing on S_ab and its conjugates (degree q + 4) and w_i ∈
F[x] of degree < T − q − 4; and C on S_ab is written as C(y) = A(y) + Σ_{i∈Lin}
[σ_i^cur(y)·P_ρ,i(y) + σ_i^next(y)·P_ρ,i(g·y)] with σ public and A a function of the witness, of the
non-linear columns and of what is seen. Since P_ρ,i(y) = 0 on S_ab, what is left is Λ(w) := (Σ_i
σ_i^next(x_j)·P_ρ,i(g·x_j))_j, Σ_i σ_i^next(z·g)·P_ρ,i(g²·z)), an F-linear map from ⊕_i F^{T−q−4} to
E^{q+1} (2q + 2 dimensions over F). **If Λ is surjective**, the distribution of (P(S_ab) of all the
columns, C(S_ab)) given the real trace is the uniform one over its support, independent of the
witness: that of P(S_ab) by L1 with q + 4 ≤ T, and that of C(S_ab) because A + Λ(w) with w uniform
and Λ onto is uniform whatever A is. And what is residual of w after conditioning on C(S_ab) —the
kernel of Λ— is what is left to mask D: it is what the instrument measures with the fine basis, with
the joint kernel of the randomizers, which already discounts the openings of the chunks.

**Proof.** The random rows of each column are uniform in F^T and independent of the witness and
between columns (L5). For i ∈ Lin, conditioning P_i on its q + 4 values on S_ab is conditioning on a
linear map of rank q + 4 (the Lagrange matrix over S_ab, of full rank by H1), so what is residual is
uniform in the kernel, Z_real·V_ab·F[x]_{<T−q−4}, and does not depend on the witness. C is affine in
the columns of Lin with public slopes (H12), so C(y) − A(y) = Σ_i [σ_i^cur(y)·P_ρ,i(y) +
σ_i^next(y)·P_ρ,i(g·y)], and the first summand is zero on S_ab. If Λ is surjective, Λ(w) is uniform
in E^{q+1} and, since w is independent of A, C(S_ab) is uniform and independent of the witness; so
is the joint distribution with P(S_ab) of the other columns, because those rows are independent of
w. □

**What it asks, and it is measured.** (1) q + 4 ≤ T with the Lagrange of full rank over S_ab:
already measured for all the families (`rango_pleno`). (2) Λ surjective: its rank over F is 2q + 2;
`instrumento_mlin::sobreyectiva` measures it, per family. (3) R3 with the fine basis: the masks —the
random column, the randomizers and the kernel of Λ— cover O|_K, with the usual joint kernel. With
the three, the simulator of section 5 changes only in S2: it samples P(S_ab) and C(S_ab) instead of
P(S_toc); the rest, the same. What it does not ask: nothing of the non-linear columns beyond q + 4 ≤
T.

**What the count gives, with T = 64.** Each linear column leaves T − q − 4 = 18 dimensions, and Λ
needs 2q + 2 = 86: at least five linear columns are needed. The two thresholds have 8 and 7; age, 4
(plus seven with a slope from the witness, which are of no use); refund v2, 4; v1, 0. By count, L1'
can only hold for the two thresholds, and the measurement says whether it holds.

**What is measured** (`instrumento_mlin`: `Base::Vistos`, `sobreyectiva`, `FINA_MEDIDO`; in every
canon, with seed 1, over the families that do not fit with the coarse count and have linear columns,
and over the toy as a control):

| AIR | T | linear | rank of Λ (target 86) | R3 with the fine basis: linear ones and randomizers / all three (K) | verdict |
|---|---|---|---|---|---|
| NullifierThresholdAir | 64 | 8 | 36 | not measured | no: Λ is not surjective |
| ThresholdAir | 64 | 7 | **86** | 120 / **158** (168) | no: L1' holds and R3 lacks 10 |
| EdadAir | 64 | 4 (+ 7 from the witness) | 18 | not measured | no |
| RefundAirV2 | 64 | 4 | 72 | not measured | no: 4 · 18 = 72 < 86, by count |
| RefundAir | 64 | 0 | — | — | no: no linear ones |
| the toy | 128 | 2 | 86 | 158 / 332 (424) | control: the fine one does not fall below the coarse one (156 / 330) |

That Λ does not reach it with eight linear columns (36 = 2 · 18) says that almost all of them enter
C only through the current row, or with dependent slopes in the next row; it is an inference
consistent with the figure, not a measurement. And the fine basis adds little to R3: in the toy, of
the 2 · 82 free, 86 go to making C(S_ab) uniform —the joint kernel fixes each chunk on S_ab, and
with it C— and 78 are left, two more than the 76 of the coarse one.

**What it says.** L1' is a lemma that holds, and today it rescues none of the five: in four, Λ is
not surjective, and in the joint threshold it is, but R3 with the fine basis falls ten dimensions
short of K. What it does leave written for the cut: with entry 133 —two random columns, with which
R3 holds by geometry also with T = 64 (section 13)— and L1' measured, the joint threshold is covered
without more rows; the other four —delegated authorization, age and the two refunds— need entry 131,
T = 128, and with it the coarse count (90 ≤ 128) and entry 133 cover them by geometry. After the
cut, then: the sixteen of today, plus MerkleAir and FrozenClimbAir by entry 133, plus the joint
threshold by entry 133 and L1', nineteen; and four that wait for entry 131.

**What it does not do.** It does not change the coarse count of L1 nor what §900 pinned: the fine
basis is another measurement, with its own constants. It rescues nothing today. It does not touch
the promise.

## 16. The agent's review, closed (§907)

**Where it comes from.** Entry 136 asks for an external review; the one there has been since §907 is
by an agent external to the session, not by a person: another instance, without the chronicle or the
conclusions, with the dossier, a copy of the tree of §903 and the sources of winterfell in front of
it, read-only (`doc/ocultacion-hvzk-revision-agente.md`, the whole report). It is not the review of
entry 136 —the author chooses the reviewer— and it counts as the third review of the house. Its
seven objections are in section 11 with their closure; this section writes the two lemmas it asked
for, redoes the count of the advantage and measures what it asked to measure.

### L7 — The joint kernel: why M_lin is a mask although it moves what is opened of the chunks (§907)

**What was missing in L3.** L3 asks two things of a mask: a linear and public image in D, and
uniformity independent of the rest of D once the view is conditioned on. The random rows of a linear
column meet the first, and the second against what is opened of the trace —they vanish on S_toc— and
against C(x_j) and C(z·g) —which do not see them—; but NOT against the openings of the chunks: P_ρ
moves each h'_k(y) at y ∈ S_ab, and only the sum Σ_k y^{k·s}·h'_k(y) = C(y) stays fixed. The
instrument solves it with the joint kernel (section 14); the lemma that justifies it is this one.

**Lemma L7.** Let W = Π_i F[x]_{<T−2q−6} be what is free of the linear columns after L1 (P_ρ,i =
Z_real·V_toc·w_i), R = E[x]_{<m}^{N−1} the randomizers, and o = (h'_k(y))_{k<N, y∈S_ab} the openings
of the chunks. Then o = n + A(w) + B(r), with A and B linear and public —the quotient path of w,
chunked like `segmentar`, and x^s·r_k − r_{k−1}, evaluated on S_ab— and n the chunks of C without
the part of (w, r): a function of the witness and of the random rows of the non-linear columns.
Conditioned on everything else in the view and on n, the pair (w, r) is uniform in a translate t + J
of the joint kernel J = ker[A | B]: t depends on the witness, J does not. And D = D' + M(j), with j
uniform in J and independent of D', where M is the linear image of (w, r) in D —cc_i·Q[P_ρ,i]
through the direct path, Σ_k cc'_k·Q[·] through the chunks— and D' is determined by the rest of the
view.

**Proof.** (a) The prior of (w, r) is uniform in W × R: each w_i by L1 —the residue of a uniform
column conditioned on its values on S_toc is uniform in the kernel of the evaluation on S_toc— and
each r_k by construction (L5), independent of each other and of the witness. (b) The condition «the
openings equal o» is affine in (w, r): [A | B](w, r) = o − n. A uniform distribution on a vector
space, conditioned on an affine equation, is uniform on the fiber —a translate of the kernel— if the
fiber is not empty. (c) The fiber is not empty for any w: at each y ∈ S_ab, the vector (y^s·r_k(y) −
r_{k−1}(y))_{k<N} ranges, as (r_k(y))_{k<N−1} ranges over E^{N−1}, over the hyperplane {v : Σ_k
y^{k·s}·v_k = 0} (the triangularity of L2), and (r_k(y))_{y∈S_ab} ranges over E^{q+2} as r_k ranges
over E[x]_{<m}, because q + 2 ≤ m (H4, Vandermonde). And o − n − A(w) is in the product of those
hyperplanes for every w: Σ_k y^{k·s}·o_{k,y} = C(y) and Σ_k y^{k·s}·n_{k,y} = C(y), because the
quotient path of w does not move C on S_ab —C(P_ρ)(y) − C(0)(y) = σ(y)·P_ρ(y) + σ'(y)·P_ρ(g·y) = 0,
both points are touched— and the telescopic sum does not see r. (d) Hence (w, r), conditioned on the
view without D and on n, is uniform in t + J, with t any solution and J the kernel, which depends
only on A and B, public. D is affine in (w, r) with linear part M, so D = D' + M(t) + M(j) with j
uniform in J; M(t) goes with D'. ∎

**What it gives.** rank_F(O∘M|_J) = rank[opened | O∘M] − rank[opened], which is what
`instrumento_mlin` computes over random combinations of the parameter space (rank and nullity: the
rank of the stack is that of the first plus the dimension of the image of its kernel); and R3 with
M_lin is O(M(J)) + O(M_R(U)) = O(K), with U what is free of the random column, which enters no chunk
and is independent of (w, r). With that S5 is justified: the class of D modulo K does not depend on
the witness (L3) and, inside K, the mask is uniform over O(K). H12 goes from measured to proved as a
lemma, with the measurement of its rank.

### L8 — The coverage of layer 0 point by point, for any positions (§907)

**What can be proved without measuring.** Let P₀ be the set of points of Λ that FRI opens at layer 0
outside the positions: the seven coset companions of each position, 7c₀ distinct points with c₀
distinct cosets; and P₀' ⊆ P₀ those that are not touched, x ≠ g·x_k for every position x_k (the
others are the coincidences of objection 1). A coset collision only removes points —c₀ < q makes P₀
smaller—, and the lemma is monotone in that.

**Lemma L8.** (i) For any distinct positions, if 7c₀ ≤ L − q − 4, the random column alone covers at
each x ∈ P₀ the line F·a_x ⊂ E, with a_x = cc_R·(2x − z − z·g)·V_pos(x)·(x − z̄)(x − z̄·g) ≠ 0, and
the 7c₀ lines at once: what is free of the column is Φ·u with u ∈ F[x]_{<L−q−4}, and u ↦
(u(x))_{x∈P₀} is surjective onto F^{P₀} (Vandermonde, with distinct nodes of F and no more nodes
than coefficients). It is exact, without probability: 294 ≤ 466, 978 and 2002 with T = 256, 512
and 1024. (ii) Let i be a linear column. Its direct path covers at each x ∈ P₀' the line F·b_x, with
b_x = cc_i·(2x − z − z·g)·Z_real(x)·V_toc(x)/((x − z)(x − z·g)) ≠ 0, and b_x/a_x = (cc_i/cc_R)·f_x
with f_x = Z_real(x)·(x − g²z)(x − g²z̄)·Π(x − g·x_k) ∈ F, non-zero (the product, over the g·x_k
that are not positions): if cc_i/cc_R ∉ F, (a_x, b_x) is an F-basis of E at each x ∈ P₀'. If
moreover |P₀'| ≤ T − 2q − 6, the map (u, w_i) ↦ (D(x))_{x∈P₀'} —with the direct path AND the
quotient path of w_i, Γ_x(w_i), which is not pointwise— is surjective onto E^{P₀'} except with
probability ≤ |P₀'|/|F| over cc_i. (iii) With n linear columns, «some one with cc_i/cc_R ∉ F» fails
with probability
|F|^{−n}: with n ≥ 4, less than 2^{−256}.

**Proof of (ii).** The matrix over F of (u, w) ↦ (D(x))_{x∈P₀'} ∈ E^{P₀'} ≅ F^{2|P₀'|} has the
columns of u fixed —a_x·u(x), without cc_i— and those of w affine in the two coordinates (c₀, c₁) of
cc_i: cc_i·ψ(x)·w(x) + Γ_x(w), with Γ the quotient path, which carries the cc'_k and the
coefficients of the constraints and not cc_i. Fix a cc_i* with cc_i*/cc_R ∉ F. Without Γ, the map is
surjective at cc_i*: at each x, (a_x, b_x) is a basis, and u and w each range over F^{P₀'}
(Vandermonde: |P₀'| ≤ T − 2q − 6 ≤ L − q − 4); hence some minor of size 2|P₀'| of the matrix without
Γ, with |P₀'| columns of u and |P₀'| of w, is non-zero at cc_i*. Take those columns in the matrix
with Γ: their minor Q is a polynomial in (c₀, c₁) of degree ≤ |P₀'| —each column of w is affine in
them— whose homogeneous part of degree |P₀'| is the minor without Γ, non-zero; so Q is not the zero
polynomial, and Schwartz–Zippel over F² gives Pr[Q(cc_i) = 0] ≤ |P₀'|/|F|. Where Q ≠ 0, the map is
surjective. ∎

**What it says and what it does not.** With T = 512 and 1024, |P₀'| ≤ 294 ≤ 422 and 934: layer 0 is
covered coordinate by coordinate for any positions, outside the coincidence points, with failure ≤
294/2^64 ≈ 2^{−55,8} over the DEEP coefficient of a single linear column —a bound of the technique,
not of the event: the only bad event the proof names, cc_i ∈ F·cc_R, has probability 1/|F| per
column and |F|^{−n} with the n there are, and even in it the quotient path can cover—. With T = 256,
166 < 294: solvency does not get in with one column, and with its four together (664 ≥ 294)
surjectivity is a joint-rank condition, not a Vandermonde one, which the measurement decides
instance by instance. The lemma does not cover the second coordinate at the coincidence points
—b_x = 0, and only the quotient path and the randomizers can supply it— nor the L/8 − c₀ new
functionals of layer 1 (22 over E with T = 256 and c₀ = 42): that stays in the measurement. The
denominator |F| instead of |E| belongs to the construction: u and w live in F[x] because the random
column and the real columns live in F, the map is F-linear and its minors are polynomials in the
coordinates over F of the challenges; with the DEEP mask in E (entry 133) the bound would be
over |E|.

**The count of the advantage, redone (O4).** What the simulator of section 5 leaves: 2^{−60} from H3
(z without coincidences of conjugates, per proof); 2^{−128} from S5 (the exact degree L − 2, which
the prover asserts); Q·2^{−256} from L0 (Q oracle queries on prefixes with fresh roots); and ε_R3,
that R3 with M_lin fails on the real challenges. Of ε_R3: on layer 0 with T ≥ 512, outside the
coincidences, ≤ 7c₀/|F| ≈ 2^{−55,8} by L8; for fixed positions at which a measured instance gives
full rank, the drop in rank over (z, cc, cc', α_i) is the vanishing of a non-zero minor —size ≤
1016, degree bounded by about ten per entry in the coordinates over F of the challenges—, ≤
10·1016/|F| ≈ 2^{−50,7}, of the order of 2^{−51} (⚠️ section 6 said «over |E|»; corrected); and for
the rest —layer 1, the coincidences and the whole of solvency— there is no bound: there are the
measured instances below. The proposition of section 8 is stated with this count, and it is still
not a promise.

**What is measured (O1).** Solvency, the family with the thinnest margin —four linear ones with 166
free, a single randomizer—, in every canon, with four instances of challenges: seed 1 (that of
`MLIN_MEDIDO`), seeds 2 and 3 whole, and the adversarial one —the positions of seed 1 with three
forced pairs: p₁ = p₀ + |Λ|/8 (a coset collision at layer 0), p₃ = p₂ + 3|Λ|/8 − blowup (a
coset–touched coincidence: g·x₃ = x₂·μ³, the next of x₃ in the coset of x₂) and p₅ = p₄ − blowup
(g·x₅ = x₄, a position that is the next of another), with the z, the coefficients and the α of seed
1—. What each instance brings and what it measures, over F (`MLIN_INSTANCIAS`):

| instance | collisions · coincidences · next ones | free per linear | undetermined (O over K) | linear ones and randomizers | all three | linear ones alone | R3 with M_lin |
|---|---|---|---|---|---|---|---|
| seed 1 | 0 · 3 · 0 | 166 | 632 | 632 | 632 | 587 | holds |
| seed 2 | 0 · 1 · 0 | 166 | 632 | 632 | 632 | — | holds |
| seed 3 | 2 · 1 · 0 | 166 | 604 | 604 | 604 | — | holds |
| adversarial 1 | 1 · 4 · 1 | 167 | 618 | 618 | 618 | — | holds |

Each collision removes 14 from what FRI opens undetermined —seven points over E—, and the mask drops
by exactly the same: R3 holds in the four, with the linear ones and the randomizers just enough in
all of them, and with coincidences in the four —three already in seed 1, which the review could not
see, and four in the adversarial one: points of layer 0 where the direct path of the four linear
ones is zero and the second coordinate is supplied by the quotient path with the randomizer—. The
positions depend only on |Λ| and q, so what each seed brings is the same for all the families of a
given T: with T = 512 and 1024, seeds 1 and 2 bring no pair, seed 3 brings a collision and the
adversarial one one of each (the toy's test prints it). The linear ones alone, without the
randomizer, reach 587 of 632 in seed 1: the randomizer supplies 45, what the review asked to know
(its section 4.1). What this says is what it says: four instances, not a bound.

**The other fifteen**, in the long test, with seed 1 (that of `MLIN_MEDIDO`, now with the linear
ones alone), seed 2 whole and the adversarial one; the positions of each seed are the same for all
the families of a given T, and the ranks came out equal within each T:

| families | instance | collisions · coincidences · next ones | free per linear | undetermined (O over K) | linear ones and randomizers | all three | linear ones alone | R3 with M_lin |
|---|---|---|---|---|---|---|---|---|
| the six with T = 1024 (SendAir, SendV2Air, ClaimAir, ClaimAirV2, BurnAir, DoubleEntryAir) | seed 1 | 0 · 0 · 0 | 934 | 1016 | 1016 | 1016 | **1016** | holds |
| | seed 2 | 0 · 0 · 0 | 934 | 1016 | 1016 | 1016 | — | holds |
| | adversarial 1 | 1 · 1 · 1 | 935 | 1002 | 1002 | 1002 | — | holds |
| the nine with T = 512 (PrendaAir, MintPendingClimbAir, AuditAir, BandaAir, CobroPendienteAir, PagoEnCursoAir, CreditClimbAir, MintClimbAir, RecoveryClimbAir) | seed 1 | 0 · 0 · 0 | 422 | 760 | 760 | 760 | **760** | holds |
| | seed 2 | 0 · 0 · 0 | 422 | 760 | 760 | 760 | — | holds |
| | adversarial 1 | 1 · 1 · 1 | 423 | 746 | 746 | 746 | — | holds |

R3 with M_lin holds in the 48 instances of the sixteen families, the adversarial one included —the
collision removes 14 from K and the same from the mask, in all of them—. And what the column of the
linear ones alone says, which no earlier measurement said: with T ≥ 512, the random rows of the
linear columns cover BY THEMSELVES everything FRI opens undetermined, without the random column and
without the quotient randomizers; only solvency, with four linear ones and 166 free, needs the
randomizer, which supplies 45. The times: from 279 to 425 s per family for seed 1 with T = 1024
(four eliminations over F), from 118 to 133 s with T = 512 and 18 s for solvency; seed 2 and the
adversarial one, two eliminations each, from 189 to 296 s and from 77 to 90 s.

**The fingerprint of each composition (O3).** `huella`: the Blake3 of the counts of the AIR —L, real
and auxiliary columns, auxiliary elements, domain factor, chunks, transitions and assertions— and of
C at four points of E with four random frames from a fixed coin, with fixed coefficients and fixed
auxiliary elements. It changes with any change in the constraints —another slope, another assertion,
another column order, another degree, another periodic one— and not with a comment or with the
witness. `HUELLA_C` pins the 23 and the test of the 23 families compares it in every canon: a change
is red and forces remeasuring entry 134 in the long test and pinning again `MLIN_MEDIDO`,
`MLIN_INSTANCIAS` and `HUELLA_C`. It is what the census did not guard: an AIR that changed its
constraints while keeping how many columns enter linearly moved the rank without moving any guard.

**What is left.** A bound for what remains measured and not bounded: layer 1 (L/8 − c₀ new
functionals over E), the second coordinate at the coset–touched coincidences, and the whole of
solvency, where 166 < 294 and the four columns together are a joint-rank condition; L8 says where
the edge is and why the denominator is |F|. The panic of the degree assert (entry 137) and the guard
of the periodic assertions (entry 138), of the fork (⚠️ §911: both closed). And the review of entry
136, which is by a person the author chooses: this one does not replace it, and saves it what has
already been seen.

## 17. What the cut would make of the argument: R3 as a theorem with the paired columns (§908)

**The observation.** Everything section 16 leaves measured and not bounded —layer 1, the
coset–touched coincidences, solvency, and the |F| in the denominator of L8— comes from a single
thing: the DEEP mask lives in F. With the two random columns of D-AL (§899) and two independent DEEP
coefficients, cc₁ and cc₂, what the two leave free in D is 2(x − x₀)·V_pos·(x − z̄)(x −
z̄·g)·{cc₁·u₀ + cc₂·u₁} with u_i ∈ F[x]_{<L−q−4}, which is a space over E —the whole of
E[x]_{<L−q−4}— if and only if cc₂/cc₁ ∉ F. With cc₂ := cc₁·ν, ν ∈ E \ F fixed, it always is: the two
columns are one column over E, R₀ + ν·R₁, with a single coefficient, and the whole argument becomes
E-linear. It is D-AL bis of RFC-0009, proposed; it does not change a byte of the proof, and it is
one line in the prover's DEEP composer and another in the verifier's.

### L9 — With the paired columns, the masks cover the whole of K, for any positions (§908)

**Statement.** Let M_par = 2(x − x₀)·V_pos·(x − z̄)(x − z̄·g)·E[x]_{<L−q−4} be what the two paired
columns leave free in D, and M_r = Σ_k (cc'_k·x^s − cc'_{k+1})·V_pos·(2x − z − z·g)·E[x]_{<m−q−2}
what the quotient randomizers leave (L2, L7). Then M_par + M_r = K for any positions, α_i and AIR,
unless cc'_{k+1} = cc'_k·z̄^s for every k < N − 1, an event of probability ≤ |E|^{−(N−1)} over the
DEEP coefficients of the chunks; under H3 (z ∉ F).

**Proof.** K = (x − x₀)·V_pos·E[x]_{<L−q−2}. Let ev: K → E², k ↦ (k(z̄), k(z̄·g)). Since (x −
x₀)·V_pos does not vanish at z̄ or at z̄·g (z̄ ∉ F ⊇ Λ, and z̄ = x₀ would require z̄/z = (1 + g)/2 ∈
F of norm 1, that is g = 1 or g = −3), ev is surjective —E[x]_{<L−q−2} interpolates two values at
two points— and ker(ev) = {k ∈ K : (x − z̄)(x − z̄·g) | k} = M_par. Hence K = M_par + M_r if and
only if ev(M_r) = E². Take a k and v ∈ E[x]_{<m−q−2}: ev of its term is (A·v(z̄), B·v(z̄·g)) with
A = (cc'_k·z̄^s − cc'_{k+1})·V_pos(z̄)·(2z̄ − z − z·g) and B the same at z̄·g. V_pos(z̄) ≠ 0 (z̄ ∉
F); 2z̄ − z − z·g = 0 is the case g ∈ {1, −3} above, and 2z̄·g − z − z·g = 0 would require (1 +
g)/(2g) of norm 1, g ∈ {1, −1/3}: none is a generator of order L ≥ 128. What is left is cc'_{k+1} =
cc'_k·z̄^s, probability 1/|E| per k, independent. If A ≠ 0 and B ≠ 0, since m − q − 2 = 20 ≥ 2,
(v(z̄), v(z̄·g)) ranges over E² as v ranges over E[x]_{<m−q−2}, and ev(M_r) = E². ∎

**What it gives.** R3 (H6) stops being a measured hypothesis: the witness-independent masks cover
the WHOLE of K —not only what FRI opens of it— for any positions, α_i and AIR; neither M_lin (H12),
nor the fine count (H13), nor L6, nor L8 (H14) are needed for the statement, and they remain as
guard instruments. The simulator of section 5 does not change: S5 samples D̃ uniform in the class of
K, and now the real distribution is exactly that one. The count of the advantage stays at 2^{−60}
(H3) + 2^{−128} (S5) + Q·2^{−256} (L0) + |E|^{−(N−1)} (L9), without ε_R3 and without any |F|. What
the proposition still asks for: L1 per geometry (T ≥ 2q + 6, entry 131 for the five with T = 64), L2
(q + 2 ≤ m), H3, and the model of L0, L4 and L5.

**Measured** (`instrumento_r3::cubren_k`, `K_ENTERO_PAREADAS`), in the coefficient space and without
going through O, over F, with seed 1 and with the adversarial positions of §907:

| geometry | instances | 2·dim_E K | one column (today): column · randomizers · together | two unpaired (D-AL): columns · randomizers · together | two paired (D-AL bis): columns · randomizers · together | ev(M_r), of 4 |
|---|---|---|---|---|---|---|
| toy, T = 128, 4 chunks | seed 1 and adversarial | 424 | 210 · 80 · 254 | 420 · 80 · 424 | 420 · 80 · **424** | 4 |
| T = 64, 15 chunks | same | 168 | 82 · 80 · 126 | 164 · 80 · 168 | 164 · 80 · **168** | 4 |
| T = 256, 2 chunks | same | 936 | 466 · 40 · 506 | 932 · 40 · 936 | 932 · 40 · **936** | 4 |
| T = 256, 9 chunks | same | 936 | 466 · 80 · 510 | 932 · 80 · 936 | 932 · 80 · **936** | 4 |
| T = 512, 8 chunks | same | 1960 | 978 · 80 · 1022 | 1956 · 80 · 1960 | 1956 · 80 · **1960** | 4 |
| T = 1024, 8 chunks | same | 4008 | 2002 · 80 · 2046 | 4004 · 80 · 4008 | 4004 · 80 · **4008** | 4 |

In the twelve measurements —six geometries, seed 1 and the adversarial one of §907— the paired
columns alone give 2(L − q − 4), four dimensions over F fewer than K (those of z̄ and z̄·g), and
with the randomizers the whole of K: what L9 says, figure by figure; the two unpaired ones give the
same on these seeds, where cc₂/cc₁ ∉ F; one column, half plus the randomizers. The criterion of the
lemma —ev(M_r), the rank over F of the randomizers evaluated at (z̄, z̄·g)— is 4 of 4 in the twelve.
What measuring it whole costs: 1.755 s for the six geometries with the three constructions, almost
all of it in T = 1024, with ranks of 4008 × 4096 over F; that is why the seal measures whole the
toy, T = 64 and the two with T = 256 (55 s) and the criterion in the six, and the long test of
`instrumento_r3` (`k_entero_en_las_geometrias_grandes`, ignored and declared) measures whole T = 512
and 1024, with its figures pinned in `K_ENTERO_PAREADAS`.


**What it does not do.** It switches nothing on: D-AL bis is a proposal and the cut is the author's
(RFC-0018 D-H). ⚠️ §909: D-AL bis is accepted and built in the fork —`Oculta<A>` pairs the DEEP
coefficients of the random columns when there are two or more, in the prover and in the verifier—,
and the falsifier `laboratorio_columnas_pareadas` measures that it is in force: the laboratory proof
verifies with pairing and not with `verify_laboratorio_sin_parear`; the production one, with one
column, does not change. What stays switched on is what always was; the cut takes it to the 23. It
does not change today's statement (section 8 with section 16): the construction of `zkssl/0.4` is
still what it is, with one column. It does not touch H3, whose 2^{−60} is now the largest term, nor
the 2^{−128} of S5 (entry 137).

## 18. The proposition for the 23 with the construction of `zkssl/0.5` (§915)

**What changed with the cut** (§914: RFC-0018 D-H; RFC-0009 D-AL bis). Since `zkssl/0.5` every
production proof of the twenty-three families carries two random columns paired in DEEP as one over
E (L9), T random rows per column with T ≥ 128 in all of them —the five that had 64 go to 128—, the
blinded quotient with m = 64 and the salted leaves. The v2 mark that identifies it (25 bytes: m, the
family, the version of the AIR, the profile and the columns) is constant per family and does not
depend on the witness: it enters the view as what was already public by design. Nothing else in the
argument changes: the simulator of section 5 is the same, with S5 sampling D̃ uniform in the class
of K, which with L9 is exactly the real distribution.

**The hypotheses, with the cut.** From the table of section 4:

| hypothesis | with `zkssl/0.5` | what measures or sustains it |
|---|---|---|
| H1 (q + 4 ≤ T over S_ab, 2q + 6 ≤ T over S_toc) and H11 (T ≥ 2q + 6) | measured, in the 23: 2q + 6 = 90 ≤ 128 ≤ T | `INDEPENDIENTE_DE_LA_AIR`, «cubre» [covers] in the 23; `rango_de_los_tocados` and `rango_pleno`, in every canon |
| H2 (Λ ∩ H = ∅, unique positions) | proved | L1 |
| H3 (z outside F and H, no coincidences of conjugates) | assumed, failure < 2^{−60} per proof | not measured |
| H4 (q + 2 ≤ m) | measured: 44 ≤ 64 | `rango_pleno`; the mutant m = 43 |
| H5, H10 | proved | L2, L5 |
| H6 (R3) | **proved**: L9, for any positions, α_i and AIR; and measured in the five production geometries, the whole of K | `K_ENTERO_PAREADAS`: in every canon, the geometries with T ≤ 256 whole and the criterion ev(M_r) in all of them; the long test, T = 512 and 1024 |
| H7 (Blake3 as a programmable random oracle) | model | — |
| H8 (the salt is fresh), H9 (no seed is repeated) | assumed | `SECURITY.md` §3.11; the falsifier of the repeated seed |
| H12, H13, H14 (M_lin, the fine count, the real challenges of R3 with M_lin) | **not needed**: L9 covers the whole of K without them; they remain as guard instruments | `instrumento_mlin`, in every canon and in its long test |

**Proposition (conditional, for the twenty-three).** Under H1, H2, H3, H4, H5, H7, H8, H9, H10 and
H11 —H6 is L9—, for each of the twenty-three families with a row in the D-B table of RFC-0009 with
the construction of `zkssl/0.5`, there exists a simulator S, that of section 5, such that, for every
real trace the inner AIR accepts, the honest verifier's view of the hidden protocol and the output
of S are statistically indistinguishable in the programmable random oracle model, with advantage
less than 2^{−60} (H3) + 2^{−128} (S5) + Q·2^{−256} (L0) + |E|^{−(N−1)} (L9) for Q oracle queries,
where N is the number of chunks of the quotient of the geometry: 2 in solvency (2^{−128}) and 8, 9
or 10 in the others. No family stays out by name, and none needs a hypothesis of its own AIR (L6 is
not used). What the proposition does not say: nothing of soundness, nothing against a quantum
adversary, nothing of side channels and nothing of what is public by design (section 7, what still
stands).

**What is measured on the construction of the cut** (§914 and §915):

| what | where | result |
|---|---|---|
| the bound of A1 (H1, H4, H11) over the production proofs | `instrumento_cota`, in every canon | 28 proofs of the 23 provers: q from 40 to 42, 90 touched against T ≥ 128, full rank; «cubre» [covers] in the 23 |
| R3 with the paired columns: the whole of K | `instrumento_r3`, `K_ENTERO_PAREADAS`, with seed 1 and the adversarial positions of §907 | the five production geometries: T = 128 with 10 chunks, 424 = 420 + 80 (with one column, 254 of 424); T = 256 with 2 and with 9 chunks, 936; T = 512, 1960; T = 1024, 4008; ev(M_r) 4 of 4 in all of them |
| M_lin in the four with T = 128 with linear columns (guard) | `instrumento_mlin`, the long test (§914) | the joint threshold meets R3 with its linear mask alone (424 of 424); delegated authorization, age and refund v2 do not (406 of 424); refund v1 has no linear ones: for those four R3 is given by L9 |
| the census of entry 134 and the fingerprint of each composition | `CENSO_134`, `HUELLA_C` | the five with T = 128 remeasured (§914); the remaining eighteen change neither trace nor AIR |
| the byte bands | `instrumento_revela`, `BANDAS` | the five with T = 128 fall within their bands; no band byte moved |
| the whole of K in the large geometries, the long test (§915) | `instrumento_r3`, `k_entero_en_las_geometrias_grandes`, in release over the tree of the cut, 1.695 s | T = 512: 1956 + 80 = 1960 of 1960; T = 1024: 4004 + 80 = 4008 of 4008, with seed 1 and with the adversarial positions; with one column, 1022 and 2046: the figures of `K_ENTERO_PAREADAS` |
| M_lin in the 23 families, the long test (§915) | `instrumento_mlin`, `mide_m_lin_en_las_23_familias`, in release over the tree of the cut, 8.499 s | the 23, each one with its three instances (seed 1, seed 2 and the adversarial one of §907), reproduce the census of entry 134 (`CENSO_134`), `MLIN_MEDIDO`, `MLIN_INSTANCIAS` and the fingerprint of each composition (`HUELLA_C`) as they are pinned: no discrepancy; it is a guard, not a hypothesis |

**What it does not cover**, unchanged from section 7 except what is resolved: soundness; H7 —neither
the fork nor the construction is audited— and the external review, which this document asks for and
does not replace; Blake3 outside the model and the quantum adversary; the prover's side channels;
the 0.3 proofs and the 0.4 ones kept under their version, which are what they were; H3, H8 and H9,
assumed; and what the node sees by being the node and what is public by design.

**What it lacks to be a promise.** (1) The acceptance of the author, to whom every promise belongs
(`doc/METODO.md` §7): what could be promised with these words is in D-AM of RFC-0009, its draft of
§915, and with it D-A, `SECURITY.md` §3.bis and the header of this document would change; (2) the
external review of entry 136, by a person, which from here on is of sections 1 to 18 with the
construction of the cut, as the dossier says; (3) nothing more to measure: what the proposition asks
for is measured in every canon or is proved. Until then, the promise is the census.
⚠️ §916: (1) done: the author accepted on 29-10-2026 («Aceptar todo» [accept everything]), and the
proposition is the promise (RFC-0009 D-A); (2) still pending, and the promise says so: H7 is kept
until the review of entry 136.
