# RFC-0023 — The clock of expiry: the thirteen-month floor in heartbeats of signed cadence, with the sender's term in entries and a double condition

> **Español:** [`spec/rfc/0023-el-reloj-de-la-caducidad.md`](../0023-el-reloj-de-la-caducidad.md) is
> the original of this page, in Spanish; it prevails wherever the two differ. This translation is of
> commit `7766efb` (audit entry §917) and was produced by the project's assistant, a Claude Code
> session, in audit entry §918 (`GENAI.md`): it is a generated artefact, not text written by the
> author. Line citations of the form `file:N` refer to the Spanish files.

- **Status:** PROPOSED (§827) — the text, its decisions and its stages, drafted from entry 12 of
  `BACKLOG.md`, whose note of §861 postpones to this RFC the floor `Δ_min = 13 meses`
  (`PRINCIPIOS.md` §4.3, with its ❌) and the (e) of T3b, and written on what was measured in the
  tree of the seal. What that note already fixed —the unit, the double condition, `born` in
  heartbeats and attested, the T of the v1 path from its birth, with D5, and the floor for
  everything alive, with reversal over the wire closed until it exists— is written here and not
  reasoned again: decisions 1, 2, 6 and 7. What that note asks of this text —the signed cadence, the
  evidence of speeding it up and the case with no heartbeat—, from which heartbeat the floor counts
  what was born before, the cut it travels in, the form of the meta leaf and where the judge's count
  comes from are taken here by delegation of the author («resuelvelas aplicando los principios»
  [resolve them by applying the principles]), with the criterion of §609 and the precedent of §642;
  REVERSIBLE; they enter with its acceptance: decisions 3, 4, 5, 9, 10, 12 and 13, and D-J for the
  floor. Decision 10 carries a cost that the note of entry 12 did not justify, and it is declared
  (D-H). Two remain open for the author, written as questions: the Δ in heartbeats for new pending
  entries (decision 8), which would amend D1 of RFC-0003, ACCEPTED; and attesting the open `Δ`
  before a third party (decision 11), the part of the (e) of T3b that the mark of §861 in RFC-0003
  leaves to this RFC. No code stage built: E1 is this text. Moving to ACCEPTED requires building
  them (rule 4 of the PROCESO) and the author's acceptance, which is not delegated.
- **Author:** Ángel José Toranzo Portela
- **GenAI assistance:** Claude, in a Claude Code session in the cloud (§827): it drafted the text
  on what the decision of entry 12 and its review measured, and ported it onto the note of §861,
  outside step 4 of `GENAI.md` — see [`GENAI.md`](../../../GENAI.md)
- **Date:** 2026-10-21 (drafted on 2026-10-15, on `d4d6d45`, §814)
- **Protocol version affected:** `zkssl/0.4` -> `zkssl/0.5`, with E3 and E4: it changes what the
  node signs —the composition of `params_digest` and the meta leaf under `pmeta_root`—, which is
  an R1 break of RFC-0021. It travels in the cut of RFC-0018 if it arrives built; if not, in the
  next cut (decision 9). E1 and E2 change nothing that travels.
- **AUDITORIA entry(ies):** §115 and §121 (the one-minute cadence), §119 (the policy: the clock of
  §119.3, the composite power of §119.5, §119.6, without retroactivity, T3b in §119.7 and the
  floor of §119.8), §128 (T3b, split), §174 (the clock that neither the operator nor the sender
  controls), §178 (expiry), §229 (the ceiling of 248 op/s), §268 (the signed heads, with `seq`),
  §285 (whoever signs keeps a journal), §292 (`mmr_t`, since v3), §340 (the clock tied to the
  head), §350 (D-4), §354 (the delta 96 of conformance), §388 (`born` under a root), §451 and
  §452 (`params_digest` and `pmeta_root` in the v5 head), §501 (the «nunca» [never] reduced), §609
  (the criterion), §642 (the precedent), §779 (the floor, in `PRINCIPIOS.md` with ❌), §809 (the
  `superficie` block, up to date), §815 (in PARADA the heartbeat neither composes nor signs), §861
  (the note of entry 12, which fixes decisions 1, 2, 6 and 7, and the mark of RFC-0003 on the `Δ`,
  which leaves the (e) to this RFC) and §827 (this RFC).
- **Backlog:** entry **12** (dead funds), which this RFC does not close: it gives it the floor, and
  its E3 is what entry 12 asks for to consider it done; with entry **117** (the single judge, which
  touches the same calls).

## State of the stages

| stage | what it delivers | does it break the wire?, and which surface | state |
|---|---|---|---|
| E1 — the clock, written | this text: the two clocks with their names, the floor, the signed cadence, what happens with no heartbeat, whom it reaches, the judge's count and the (e) of T3b for the floor | no | **written (§827), with its decisions taken by delegation**, except 8 and 11, the author's |
| E2 — the core, switched off | two new meta leaves, the v2 one and the v1 one —`sender`, `born` in entries and `born` in heartbeats, and in the v1 one also its T—, each with its domain (D-D); `params_digest` with the cadence as eighth parameter, with its encoding and its range (D-C); the year and the floor in seconds as constants with one producer (D-B); their KATs and the second implementation that reproduces them; and the `superficie` block of `spec/RPC.md` §«Versionado y compatibilidad» [versioning and compatibility], up to date, as §809 did. Nobody consumes them yet | no: additive (R2), new paths in `spec/vectors/nucleo/` (S5) | unbuilt |
| E3 — the judge and the node | the layer judges with the double condition in `apply_refund` and `apply_deissue`; it records the new meta at birth; it receives its heartbeat count from a signed head, with the invariants of D-K; the node beats at the ledger's cadence; with the tests of E3, below | **yes**: S3, what is signed (`params_digest` and the meta leaf) | unbuilt |
| E4 — the cut | the heads that sign the new meta and the new `params_digest`; the vectors that change verdict or text, moved to `spec/vectors/0.4/` (R6 b); the notice of R5 | **yes**: that of E3, paid once (R4) | unbuilt; in the `zkssl/0.5` cut if it arrives, or in the next one |
| E5 — the speed-up, measured | a bench that compares the heartbeats counted from a starting head with a date bounded from below against the timestamp of the `0x06` co-signature of a later anchor (RFC-0012, RFC-0013); not two anchors against each other (D-F) | no | unbuilt; of another session, and it waits for a medium with foreign witnesses and for a lower bound of when a head is composed |

## Motivation

§119.8 wrote the line: **«Δ_min = 13 meses.** Ningun pago puede llevar un plazo de reversion
menor» [no payment may carry a shorter reversal term], with the precedent of the SEPA *recall* and
the declared imperfect analogy, which argues for a floor «largo, nunca corto» [long, never short]:
«Un Δ≈0 seria un pago que nunca fue promesa» [a Δ≈0 would be a payment that was never a promise]
(literal, without accents, like the log). §779 took it to
`PRINCIPIOS.md` §4.3 marked ❌, and it stays so: the tree has no floor. The note of §861 in entry
12 says why, measured, in its (3); here goes what this RFC needs from it, re-measured in the tree
of the seal.

1. **The judge counts entries, not time.** `apply_refund` and `apply_deissue` take
   `now = self.log.len()` (`crates/zk-ssl/src/two_phase.rs:676` and `:441`) and compare
   `now - born` with the term: the envelope's Δ in a v2 (`:687-697` and `:452-462`), the global
   `refund_ttl` in a v1 (`:679-683` and `:444-448`). No minimum:
   `un_refund_v2_tras_delta_devuelve_al_emisor` (`two_phase.rs:3774`) refunds with `delta = 1`.
   It is the clock that §119.3 discarded: «`seq` cuenta operaciones, no tiempo. En un sistema
   quieto, un mes son cero entradas» [`seq` counts operations, not time. In a quiet system, a month
   is zero entries].
2. **No number of entries is a calendar.** Thirteen months at 248 op/s, the node's ceiling over
   RPC (§229), are some 8.5·10⁹ entries (DEDUCED), which in a quiet ledger never expire; and
   any smaller figure runs out in hours under load.
3. **The heartbeat is already the unit that §119.3 chose, and the tree already counts it signed.**
   The node emits one head per heartbeat even when there are no operations (`arrancar`,
   `crates/zk-ssl-node/src/latido.rs:413-443`; every 60 s by default, `:72`), and every head signs
   `mmr_t`, how many earlier heads its MMR accumulates (`crates/zk-ssl/src/log.rs:642`), since
   format v3 (§292); the leaf enters when the head is kept (`latido.rs:472-474`). Since §815, in
   PARADA the heartbeat neither composes nor signs (`latido.rs:152-154`). **But the count lives in
   the node, not in the judge**: the layer compares `log.len()`.
4. **The cadence is not signed.** It is an operator option, `--latido`, and `0` switches the
   heartbeat off (`crates/zk-ssl-node/src/main.rs:234-239`). `EpochHead` has no cadence or time
   field (`log.rs:613-680`), and `params_digest` composes the seven parameters at rest without it
   (`crates/zk-ssl-hash/src/lib.rs:972`). Without the cadence, a number of heartbeats does not say
   how much time it is: 569,790 heartbeats are thirteen months at 60 s and some 6.6 days at 1 s
   (DEDUCED). The precedent of §642 (4), «un año de latidos a la cadencia configurada» [a year of
   heartbeats at the configured cadence], is a warning threshold, where speeding up only harms the
   operator itself; here the receiver is protected.
5. **The layer used as a library, without a node, has no heartbeat**, and with `--latido 0` the
   node does not either. And the `mmr_t` that the head signs is not counted by the layer: it is a
   parameter that the caller passes to `epoch_head` (`crates/zk-ssl/src/lib.rs:870-877`), and there
   are callers that pass 0, the sandbox summary (`crates/zk-ssl-cli/src/sandbox.rs:329-333`) and
   conformance (`crates/zk-ssl-cli/src/conformance.rs:139-143`). The node signs the head after
   composing it and outside the layer (`latido.rs:227-229`); without `--clave` it composes it
   unsigned and still puts its leaf into the MMR (`latido.rs:435` and `:472-474`); and without a
   journal its `t` goes back to 0 on restart (`crates/zk-ssl-node/src/main.rs:456-460`), although a
   node with `--clave` does not start without a journal (§285).
6. **«Latido» [heartbeat] is also used for entry.** `DEFAULT_REFUND_TTL` is documented «en latidos
   de `log.seq`» [in heartbeats of `log.seq`] (`crates/zk-ssl/src/lib.rs:229-231`), and the (i) of
   entry 12 says that «T cuenta latidos de `log.seq`» [T counts heartbeats of `log.seq`]: they are
   entries of the log, not heads of the heartbeat.
7. **Nobody sees the open `Δ`.** The mark of §861 in RFC-0003, under «El invariante de reembolso,
   completo» [the refund invariant, complete], corrected that the second clause, `now − born ≥ Δ`,
   is verified by «cualquiera» [anyone]: the `Δ` is not published, the opening lives only in the
   `RefundReceipt`, which no method of the wire carries, and an operator that composed the send
   knows `(refund_id, delta)`. The author classified it as a limit of the trust model, not as a
   flaw; attesting it is the (e) of T3b, and the mark leaves it to this RFC (D-J).

## Design

### D-A — Two clocks, each with its name

- **Entry**, or epoch of the log: `seq`, `self.log.len()`. The head signs it since §268, and
  §340 checked it and tied it with a test (`la_altura_entra_en_el_digest`, `log.rs:795`). In
  entries are counted the sender's Δ (RFC-0003 §«El compromiso» [the commitment]: «`Δ` = plazo en
  alturas de época» [`Δ` = term in epoch heights], with the sense of its D1) and the T of the v1
  path.
- **Heartbeat**: one head of the heartbeat. It is counted by `mmr_t`, which the head signs since v3.
  In heartbeats the floor is counted.
- Where the tree says «latido» for entry it is corrected with a mark, without erasing, in the two
  that the note of §861 names: the (i) of entry 12 and the comment of `DEFAULT_REFUND_TTL`
  (`crates/zk-ssl/src/lib.rs:229`), both in the seal of this RFC (§827) and on its same line, as
  §861 marked comments of `.rs` files in a seal of documents.

### D-B — The floor, in calendar seconds and counted in heartbeats of signed cadence

- **The floor is thirteen months of a year of 365.25 days**: 13/12 of 31,557,600 s, which are
  34,187,400 s; the 31,557,600 are the year of the node's exhaustion warning (`SEGUNDOS_ANUALES`,
  `main.rs:915`, D-F of RFC-0015). The year and the floor, constants with one producer: E2 puts
  them in the core with their KAT, and E3 makes the node derive from them its `SEGUNDOS_ANUALES`,
  which today is a local constant of `aviso_de_agotamiento`; if not, the core would be a second
  producer of the same year.
- **In heartbeats**, the floor is `⌈34.187.400 / c⌉`, with `c` the ledger's signed cadence, in
  seconds: at 60 s, 569,790 heartbeats (DEDUCED).
- **The double condition**, which the note of entry 12 fixes in its (3): a refund or a de-issuance
  is accepted only if both hold, `now - born >= plazo`, in entries —the envelope's Δ in a v2,
  the T in a v1—, **and** `latidos_now - latidos_born >= suelo`, in heartbeats. The first is
  today's, intact; the second only restricts: below the floor, `expiry = born + Δ` stops being the
  moment at which it reverts, and it reverts later.
- **De-issuance**, by analogy, which the note of entry 12 already declares in its (2) for the frozen
  ones and is declared the same way here: an issuance pending entry that expires ceases to exist
  for the receiver just as a payment one goes back to the sender. It is an extension, not the
  letter of §119.8.
- **The real «nunca» [never]** —that a Δ of the sentinel's class does not expire— is piece (1) of
  the note of entry 12, which goes in the layer without a format; the floor does not touch it.

### D-C — The cadence, signed: the eighth parameter of `params_digest`, fixed for the life of the ledger

- The heartbeat's cadence becomes **a parameter of the ledger**, the eighth that composes
  `params_digest` (`crates/zk-ssl-hash/src/lib.rs:972`), which the head signs since v5 (§451,
  §452). It is the place of the rules: «Dos cabezas del mismo operador con `params_digest`
  distintos dicen que las reglas se movieron» [two heads of the same operator with different
  `params_digest` say that the rules moved] (its documentation, on D-B of RFC-0007).
- **Its encoding**: `c`, a `u64` in seconds. `c >= 1` is a cadence. `c = 0` is the ledger without a
  heartbeat, and it is signed all the same: so a ledger without a heartbeat also has its parameter
  in `params_digest`, and its floor does not expire (D-E). `⌈34.187.400 / c⌉` is only computed with
  `c >= 1`.
- **Fixed for the life of the ledger**: the node beats at the ledger's cadence and does not start
  with a different `--latido`, nor with `--latido 0` in a ledger of `c = 60` nor with `--latido 60`
  in one of `c = 0`. Changing it would reinterpret the heartbeats already counted: those counted at
  60 s would be worth something else at 30 s (DEDUCED).
- **Its range**: a huge `c` leaves the floor at a single heartbeat —with `c >= 34.187.400`,
  `⌈34.187.400 / c⌉ = 1`— and with it the evidence of speeding up in dating a single head
  (DEDUCED). The upper bound of `c` is fixed by E2 with its KAT; this text does not fix the number.

### D-D — `born` in heartbeats, attested

- **The pending entry's meta leaf gains `born` in heartbeats** and, in a v1, its T (D-G). Today it
  is `commit_operation(DOMINIO_META_PENDIENTE, [sender, born])`, with domain `PMETA_V1`
  (`crates/zk-ssl-hash/src/lib.rs:906-912`), and today's one keeps its own. Since
  `commit_operation` assumes a FIXED length per domain, without padding (in the same file, `:1753`,
  and the mark of `:947` repeats it), they are **two new leaves, each with its domain**: the v2 one,
  `[sender, born, born_latidos]`, and the v1 one, `[sender, born, born_latidos, T]` (decision 12).
  Their names and values are fixed by E2 with its KAT. They go under `pmeta_root`, which is stored
  and checked on opening since §388 and which the head signs since v5: the lesson of §388, which
  the note of entry 12 cites.
- **The judge reads a signed number of heartbeats**: with the count of D-K it records `born` in
  heartbeats when the pending entry is born and judges afterwards. Test 4 of E3 ties it as
  `la_altura_entra_en_el_digest` ties `seq` (§340): the number it judges is that of a head whose
  signature verifies.

### D-E — Without a heartbeat, the floor does not expire

- In a ledger of signed cadence `0` (D-C), which the node runs with `--latido 0`; in a node without
  `--clave`, which composes its heads unsigned (fact 5); in the layer used as a library, without a
  node; in PARADA, in which since §815 the heartbeat neither composes nor signs (fact 3); and while
  the layer knows no signed count (D-K), there is no signed calendar: **the floor does not
  expire**, and the rejection says so with its cause. It is the «largo, nunca corto» of §119.8
  applied to whoever gives no clock. It also holds for what was born before (D-H). Its cost is
  declared: there, what the floor reaches does not revert while the clock is missing.
- A clock set by whoever uses the library would be the one §119.3 discarded: one that leaves no
  evidence of its speed-up. That is why the judge's count is not the `mmr_t` passed to
  `epoch_head` (D-K).

### D-F — What speeding up the heartbeat allows, and who sees it

- **Speeding up is beating faster than the signed cadence.** With the cadence and `mmr_t` signed,
  what is missing to see it is a foreign clock: two dated readings by someone who is not the
  operator —one that bounds from below when a head was composed and another that bounds from above
  when a later one existed—, and between them more heartbeats than fit at that cadence. The note
  of entry 12 says it this way: today only a witness's clock, and there is none.
- **The anchor already carries the number, and its date is an upper bound**: the anchor of RFC-0012
  commits `mmrSize`, the same `mmr_t` (its D-A), and the medium of RFC-0013 publishes it with
  `0x06` co-signatures with a timestamp from foreign witnesses (its D-C and its D-D). That mark
  says that the head already existed when they saw it —«esta cabeza estaba publicada, en esta
  posición, cuando `k` testigos ajenos la vieron» [this head was published, in this position, when
  `k` foreign witnesses saw it] (its D-E)—, not when it was composed; and publication is
  asynchronous, one anchor every `M` = 1,440 heartbeats, and «el nodo no espera a nadie» [the node
  waits for nobody] (its D-F).
- **Two anchors compared against each other are not enough.** The rule «dos anclas cofirmadas dan
  `(Δτ, Δmmr_t)`, y más latidos que `Δτ / c` es aceleración» [two co-signed anchors give
  `(Δτ, Δmmr_t)`, and more heartbeats than `Δτ / c` is a speed-up] is not sound: (a) if a witness
  goes down or the publisher falls behind, the earlier anchor of an honest node is co-signed late,
  `Δτ` comes out smaller than the real time and the rule gives «aceleración» [speed-up], a ROJO
  that can be manufactured against an honest operator, the class that RFC-0019 closes; (b) an
  operator that speeds up and withholds the anchors of the sped-up heads, to publish them later at
  its cadence, leaves nothing that can be seen (DEDUCED both).
- **What would be evidence** asks for two things that do not exist today. A lower bound of when a
  starting head was composed: with it, a count accumulated from that head is sound —if between it,
  composed after `τ₀`, and a head co-signed at `τ` there are `N` heartbeats and `N · c > τ − τ₀`,
  the node beat fast, and a medium that falls behind only enlarges `τ`, so it does not manufacture
  ROJO (DEDUCED)—. And someone other than the operator who dates the heads in time, without whom
  case (b) is not seen. What gives that lower bound this RFC does not fix. §174 wanted a clock «que
  ni el operador ni el emisor controlan» [that neither the operator nor the sender controls]: the
  anchor gives the upper bound, dated by others, and the lower one is still to be done.
- **Today nobody sees it, and there is nothing to see it with**: there is not a single witness
  (`PRINCIPIOS.md` §0), nor a deployed medium with foreign witnesses, nor a head that carries a lower
  bound of its composition. Speeding up the heartbeat leaves no evidence today, and it is declared.
  Building the comparison is E5, of another session, on the accumulated count and not on two
  anchors against each other.

### D-G — The T of the v1 path, the one from its birth

- The note of entry 12 fixes it in its (3) and its (4): a v1 born with this RFC carries in its meta
  the T it was born with, and the judge uses that one; the `refund_ttl` in force decides only the T
  of what is born. Today a change of `refund_ttl` also moves the T of what was already sent
  (`set_refund_ttl`, `two_phase.rs:257`, which today only tests call); D5, accepted in that note
  and unbuilt, takes it to `cfg(test)`, and the T is reviewed per release, as §178 wanted it, «se
  declara, se publica y se revisa con datos» [it is declared, published and reviewed with data]
  (`crates/zk-ssl/src/lib.rs:230`). What was born before, with no T in its meta, is judged with the
  one in force, as today.
- If a cut stops producing v1 —the note of entry 12 takes it to the RFC that opens reversal over
  the wire or to the format cut, whichever comes first—, this decision is left with nobody to apply
  to going forward, and written.

### D-H — For everything alive, and what was born before, from the head that premieres the cadence

- **The floor reaches everything alive**, as the note of entry 12 fixes in its (3), with its
  reason, which is not repeated here: a v2 with a Δ smaller than the floor —conformance uses 96
  (RFC-0003, the note of §354)— and every v1 with meta revert later than their Δ or their T said.
  What was born without meta does not change: it never reverts («el legado sin meta es inmune para
  siempre» [the legacy without meta is immune forever], entry 12).
- **What was born before has no `born` in heartbeats, and none is invented for it.** Its floor
  counts from the `mmr_t` of the first head that signs the `params_digest` with the cadence (D-C):
  the one that premieres the rules, signed. Anyone with the heads finds it by the head or
  composition version that the cut fixes (Compatibility), not by being the first with a different
  `params_digest`: that digest also changes with `refund_ttl`, one of its seven parameters, which
  D-G reviews per release, and with governance (`crates/zk-ssl-hash/src/lib.rs:953-968`). It never
  gives a shorter floor than counting it from birth: no living pending entry reverts before a whole
  floor has passed since the floor exists (DEDUCED). The judge stores that number once, on
  receiving that signed head (D-K); how it is persisted is fixed by E3. Without a heartbeat there
  is no such head, and the floor does not expire (D-E).
- **What that costs, and the note of entry 12 did not justify.** The note gives the floor for
  everything alive because the thirteen-month one was published policy since §119.8: «No es cambiar
  términos a posteriori» [it is not changing terms after the fact]. Counting from that head also
  delays what that reason does not cover: what expired more than thirteen months after being born
  and less than thirteen months after that head, and what was already reversible when it arrived.
  A v2 born ten months before that head, with a Δ in entries that expires twenty months after
  birth, reverts at twenty counted from its birth and at twenty-three with this decision; one
  already expired and not refunded when the head arrives waits a whole floor from it (DEDUCED). That
  delay, above the floor counted from birth, is not published policy, and it touches §119.6,
  without retroactivity, which the note invokes. It is declared as a cost of decision 10, and its
  reopening names it. Counting from the head that covered each birth would avoid it, but it would
  count heartbeats from before the cadence was signed, which an operator could have given faster
  without anything saying so (fact 4): it would give a shorter floor, which §119.8 does not admit.
- **Reversal over the wire does not open without the floor**, as the note of entry 12 fixes: refund
  over RPC comes afterwards, with its RFC (ARQ-02, `doc/blueprint-v2.md` §«3.4 Arquitectura y API»
  [architecture and API]; there is no refund method in `spec/RPC.md` §«Pago en dos fases» [two-phase
  payment]), and that RFC is not accepted before E3 of this one is built.
- The other way of counting what was born before, from the head that covered its birth, is
  discarded (decision 10).

### D-I — The Δ in entries, with its defect declared

- With the double condition, the sender's Δ stays in entries, and **the defect of §119.3 still
  stands for the Δ**, as the note of entry 12 declares it: in a slow ledger, a Δ in entries may
  never expire, and the sender controls the calendar only through the floor. «Funciona a cualquier
  ritmo» [it works at any pace] holds for the floor, not for the Δ.
- Moving the Δ to heartbeats for new pending entries would amend D1 of RFC-0003, ACCEPTED: it is
  question 8, the author's.

### D-J — The (e) of T3b: the time input, tied to a signed head

- **What it asks for.** The (e) of §119.7: «el input temporal atado a una cabeza firmada» [the time
  input tied to a signed head]. The note of entry 12 leaves it to this RFC (its (6): «(e): espera
  al RFC de (3)» [(e): waits for the RFC of (3)]), and the mark of §861 in RFC-0003
  says that attesting the open `Δ` is the (e) and that this RFC resolves it (fact 7).
- **For the floor, taken.** The two numbers of the floor are heartbeats of signed heads:
  `latidos_now` is the judge's count, which only a signed head moves (D-K), and `latidos_born`,
  the one the meta leaf records under `pmeta_root`, which the head signs since v5; for what was
  born before, the `mmr_t` of the head that premieres the cadence (D-H). Test 4 of E3 ties the
  number it judges to that of a head whose signature verifies, in the node, because the signing
  happens outside the layer (fact 5). It is the (e) in the sense in which §340 fulfilled it for
  `seq`: the judge's number is the signed one.
- **What that does not give**: that a third party proves that a concrete refund was applied before
  the floor or the `Δ`. Of a `Refund`, the log keeps the digest of its proof (the table «Lo que
  asienta cada clase» [what each class records] of `spec/RPC.md`), and the opening and the `born`
  stay in the layer. The proof of the payment in flight of RFC-0008 (E2) counts in entries,
  `nacido + delta >= T`, and says nothing about heartbeats: of the floor it shows nothing.
  Attesting them before a third party is question 11, the author's. This RFC neither builds it nor
  discards it.

### D-K — The judge's count: where it comes from, and what it cannot do

- **It is not the parameter of `epoch_head`.** The `mmr_t` that the head signs is passed by
  whoever composes it, and there are callers that pass 0 (fact 5). If the judge counted with that
  parameter, the clock would be set by anyone who uses the library, which is what D-E and decision
  5 discard. The judge's count, `latidos_now`, comes in through another door: the node gives it to
  the layer **after signing** the head, with the signed head. Which function it is and what the
  layer checks of that signature is fixed by E3.
- **Only a signed head moves it, under a signed cadence `c >= 1`.** The unsigned heads of a node
  without `--clave` do not move it, even though their leaf enters the MMR; a ledger of `c = 0`
  does not even have it.
- **The invariants, which are the protocol's**:
  1. the count never goes back: a head with an `mmr_t` smaller than the last known count does not
     move it;
  2. `born` in heartbeats never stays below the last signed count that the layer knows when the
     pending entry is born;
  3. with no known count —after starting and before the first signed head, if it was not seeded;
     in the layer without a node; with `c = 0`— the floor does not expire (D-E);
  4. subtraction saturates: with `latidos_now` smaller than `latidos_born`, what has elapsed is 0.
- **What is born with no known count does not record 0**: that would make it expire right away,
  which is the harm the floor prevents. How it records it —with the last count that the layer keeps
  next to the batch, or with an «unknown» mark that the judge counts from the first signed count
  that arrives, like what was born before (D-H)— is fixed by E3, with a test, within the
  invariants. Whether the count is persisted with the batch or seeded from the journal on starting,
  too.

## The tests of E3

One discriminating test per property (`PRINCIPIOS.md` §9), in `apply_refund` and in `apply_deissue`:

1. **The heartbeat, alone**: with the term in entries already met and no operations, a refund goes
   from `RefundTooEarly` to `Ok` when the floor's heartbeats pass, and not before.
2. **The floor rules over a short Δ**: with entries to spare and heartbeats below the
   floor, `RefundTooEarly`; on reaching the floor, `Ok`.
3. **Without a heartbeat**, in a ledger of `c = 0`, in PARADA, in a node without `--clave` and in
   the layer without a node, the floor does not expire; after starting with no known count, neither,
   and what is born in that window does not record 0 (D-K).
4. **The number it judges is the one signed**, in the node: the count with which the layer judges is
   the `mmr_t` of a head whose signature verifies; an unsigned head does not move it, nor one with a
   smaller `mmr_t`; and a head in which only `mmr_t` is changed changes its digest and its signature
   stops verifying.
5. **The cadence, signed**: changing it changes `params_digest`, and a node with a `--latido`
   different from the ledger's `c` does not start, also `--latido 0` with `c = 60` and `--latido 60`
   with `c = 0`.
6. **The T of the v1, the one from its birth**: a v1 judged after changing the `refund_ttl` in force
   keeps the T it was born with.
7. **For everything alive**: a pending entry born with today's meta does not revert until the
   floor's heartbeats pass, counted from the head that premieres the cadence, even if its Δ or its T
   have already been met; the same one, born with the new meta, counts from its `born` in
   heartbeats.

The first two are the condition with which the note of entry 12 considers the floor fulfilled.

## Compatibility

With E3 and E4, the wire rises to `zkssl/0.5`: what the node signs changes, `params_digest` and the
meta leaf under `pmeta_root` (R1 of RFC-0021). Which head version announces it is fixed by the
cut, together with the v7 of RFC-0020: if they go in the same one, a single new version for both
things (DEDUCED). Heads v2 to v6 and their vectors do not change a byte; what changes verdict or
text is moved to `spec/vectors/0.4/` (R6 b). E1 and E2 change nothing that travels. This RFC
is PROPOSED at least one seal before the cut, as R5 asks and as the note of entry 12 asks.

## Decisions (by delegation of the author, §827), and two questions for him

Decisions 1, 2, 6 and 7 were fixed by the note of §861 in entry 12, and here they are written with
their place in the design, without reasoning them again; 3, 4, 5, 9, 10, 12 and 13 are taken here,
with their reason and what was discarded —3, 4 and 5 are asked of this text by that note, 10 is
asked by 6, and 12 and 13 are asked by the design of D-D and of D-K—; 8 and 11 are questions for
the author, open.

1. **In what unit does the floor count?** **Fixed by the note of entry 12 (§861, its (3)): in
   heartbeats** (D-B), neither withdrawn nor in entries.
2. **One clock or two?** **Fixed by the note of entry 12: the double condition** (D-B), with the
   defect of §119.3 declared for the Δ (D-I). Everything in heartbeats is question 8.
3. **Where is the cadence signed?** In `params_digest`, as eighth parameter and fixed for the life
   of the ledger (recommended: it is the place of the rules, and it is already signed); in a field
   of the head of its own; or unsigned. **Taken: in `params_digest`, fixed**, a `u64` in seconds,
   `0` for the ledger without a heartbeat and with the upper bound that E2 fixes (D-C). Discarded:
   a field of its own, which would pay for another head version for what the parameters already
   group; unsigned, because the floor in heartbeats would not say how much time it is (fact 4); and
   a cadence that changes, which would ask for a clock of declared seconds —the sum of the cadence
   of each heartbeat— and another signed field.
4. **What evidence does speeding it up leave?** That of the anchor co-signed with a timestamp, with
   a count accumulated from a starting head with a date bounded from below (recommended: the anchor
   already carries the number, the date is set by others, and the accumulated count does not give
   ROJO when the medium falls behind); two anchors compared against each other; or a time field in
   the head, which the operator dates. **Taken: that of the anchor, with the accumulated count**
   (D-F), declared today without evidence: the lower bound and someone who dates the heads in time
   are missing. Discarded: two anchors against each other, which give a ROJO that can be
   manufactured against the honest operator when a witness goes down or the publisher falls behind;
   and the operator's time, which is not a foreign clock.
5. **And without a heartbeat?** The floor does not expire (recommended: long, never short); or a
   clock from whoever uses the library. **Taken: it does not expire** (D-E), also in PARADA, without
   `--clave` and with no known count. Discarded: the unsigned clock.
6. **Is the floor retroactive?** **Fixed by the note of entry 12: for everything alive** (D-H),
   with reversal over the wire closed until the floor exists. From which heartbeat what was born
   before counts is decision 10.
7. **The T of the v1 path?** **Fixed by the note of entry 12: the one from its birth** (D-G), with
   its D5.
8. **The Δ in heartbeats for new pending entries?** It would close the defect of §119.3 for the Δ
   (D-I), and it would amend D1 of RFC-0003, ACCEPTED, which counts the Δ «en alturas de época»
   [in epoch heights]. The note of entry 12 leaves it written for the author. **Open, the
   author's**: applying is not amending, and the delegation does not take it.
9. **Does it travel in the `zkssl/0.5` cut?** Yes, if E2 and E3 arrive built (recommended: a
   break is paid once, R4); if not, in the next cut. **Taken: so**, as far as this RFC is
   concerned. Whether it travels in the train is agreed by whoever carries it (RFC-0018), and it
   remains to be coordinated: the cut row of RFC-0018 does not name it yet, nor the Compatibility
   of RFC-0021, which counts which RFCs of the cut already meet R5. Discarded: a cut of its own.
   ⚠️ §865: they name it without agreeing.
10. **From which heartbeat does the floor count what was born before?** From the `mmr_t` of the
    first head that signs the cadence (recommended: it is signed, it is a single one for the whole
    ledger and it never gives a shorter floor than counting it from birth); or from the head that
    covered the birth of each one. **Taken: from the first head that signs the cadence** (D-H),
    with its cost declared: it also delays what expired above the floor counted from birth, and
    what was already reversible when that head arrived, up to a whole floor from it, which the
    reason of the note of entry 12 does not cover and which touches §119.6. Discarded: from the head
    of each birth, which asks the judge for an old head per pending entry, counts heartbeats from
    before the cadence was signed, which could have gone faster without anything saying so, and
    what was born before the v3 heads has none that signs `mmr_t` (§292).
11. **Is the opening of a refund, its `Δ` and its `born`, attested before a third party?** It is
    the part of the (e) of T3b that D-J does not take (fact 7). The branches, recommending none:
    - **no**: the limit of the trust model that the author classified in §861 remains, declared;
      the floor restricts the honest judge, and what it leaves enforceable is the signed cadence
      and the heartbeats, with which speeding up the clock would be seen once there is the
      accumulated count and someone who dates the heads (D-F), not a judge that skips the term;
    - **the opening in the log entry of the refund**, in the clear or in its commitment: with two
      heads, anyone checks both conditions. It changes what the log commits (S3, at the cut) and
      what the row of the refund opening of RFC-0009 D-B leaves in the receipt, outside the proof;
      and in the clear it would reveal `f`, against D2 of RFC-0003, which made `X` opaque for its
      privacy;
    - **`born` and `Δ` in the public inputs of `RefundAirV2`**: the proof's verifier would check
      it, and it departs from the decision of RFC-0003 to leave `born` and `now` outside the
      circuit, attested; it belongs to the zone of the AIRs. And alone it gives a third party
      nothing to check: the log keeps the digest of the proof and not the proof, no method of the
      wire serves it and the kit does not compile the refund circuit (entry 115 of `BACKLOG.md`),
      so it also asks for one of the two ways out of entry 115.

    **Open, the author's**: the second and the third move what two ACCEPTED RFCs decide, 0009
    and 0003, and the first takes as final a limit that §861 left to this RFC; choosing between
    them is not applying the principles, it is fixing them.
12. **How does the meta leaf carry the T of the v1?** Two leaves, each with its domain (recommended:
    `commit_operation` assumes a fixed length per domain, without padding); or a single one, with a
    sentinel of T in the v2. **Taken: two leaves, two domains** (D-D). Discarded: the sentinel, a
    value with two meanings, which is what piece (1) of the note of entry 12 fixes in the Δ.
13. **Where does the judge's count come from?** From a signed head, which the node gives to the
    layer after signing it, with the invariants of D-K (recommended: the judge's number is the
    signed one, and without a count the floor does not expire); or from the `mmr_t` passed to
    `epoch_head`. **Taken: from the signed head** (D-K). Discarded: the parameter of
    `epoch_head`, which whoever composes sets, and which the sandbox and conformance already pass
    as 0.

The decisions taken, all REVERSIBLE. **When they are reopened**: 3, if an operator needs to change
the cadence of a living ledger; 4, if the medium does not come to have foreign witnesses, if its
co-signatures lose the timestamp or if no lower bound of when a head is composed appears; 5, if
the layer without a node or a ledger of cadence 0 come to have a signed count that is not set by
whoever uses them; 9, if the cut closes without E3, or if whoever carries the train does not admit
it; 10, if the judge comes to read the head of each birth and the author prefers to count from it,
or if the author judges the delay that D-H declares a change of terms after the fact (§119.6);
12, if `commit_operation` gains a padding rule (`crates/zk-ssl-hash/src/lib.rs:1753`); 13,
if the layer comes to sign its own heads; D-J, if the answer to 11 puts the opening in
the log or in the circuit; 1, 2, 6 and 7, with the note of entry 12; and all of them, if the author
revokes any.

## What was DISCARDED on measuring

What was discarded by the note of entry 12 —withdrawing the floor, the floor in entries and the
floor only for what is born afterwards— and by D-4 of §350 —`refund_ttl` as ceiling of the v2 Δ,
which the note cites in its (4)— is not repeated. Here:

- **The floor as an SDK policy**: an honest payer would not compose a short Δ, but whoever
  composes their own envelope skips it, and RFC-0003 puts the judge in the layer.
- **Counting what was born before from its birth** (decision 10).
- **Comparing two anchors against each other** (decision 4): a medium that falls behind gives ROJO
  to an honest operator (D-F).
- **Judging with the `mmr_t` passed to `epoch_head`** (decision 13): the clock would be set by
  whoever composes (D-K).

## Security

- **The spend key never travels**: this RFC does not touch it. It changes when a term expires, not
  who signs.
- **The floor only restricts**: no refund that is rejected today becomes accepted. The destination
  does not change: it is still the one the envelope commits in a v2 and the sender of the meta in
  a v1.
- **The cadence, signed and fixed**, takes from the operator the power to shorten the floor by
  changing `--latido` without a head saying so. Speeding up the heartbeat beyond the signed cadence
  leaves no evidence today: it would be given by a count accumulated from a head with a date
  bounded from below against the mark of a co-signature, and against whoever withholds their
  anchors, only by someone who dates the heads in time (D-F); there is neither of the two. Two
  anchors compared against each other do not give it, and they would manufacture a ROJO against the
  honest operator.
- **The judge's count** is moved only by signed heads, does not go back, and without it the floor
  does not expire (D-K): whoever uses the library, or a node without `--clave`, does not bring the
  floor forward by passing another `mmr_t`.
- **What was born before** stays under the floor, counted from the first head that signs the
  cadence (D-H): no living term expires before a whole floor since the floor exists. And some
  expire later than the floor counted from their birth would ask, up to a whole floor from that
  head, which is the declared cost of decision 10. Reversal over the wire does not open without the
  floor.
- **What the floor does not give**: that a third party proves that a judge skipped it. The operator
  that runs the judge can bring a refund forward without enforceable evidence, as today with the
  `Δ` (the mark of §861 in RFC-0003): the money goes back to the sender, which is the «acelerar
  devoluciones, nunca desviarlas» [speed up returns, never divert them] of
  `doc/CADUCIDAD_PENDIENTE.md`. It is question 11.

## What this RFC does not do

- It does not touch code: the core is E2, the judge and the node are E3. Its seal only marks, on its
  same line, the comment of `DEFAULT_REFUND_TTL` (D-A).
- It does not put the clock in a circuit: RFC-0003 leaves `born` and `now` outside, attested, which
  is the «condicionada, no caída» [conditioned, not dropped] branch of §119.7. Clause (c) of T3b in
  circuit belongs to the session of the AIRs.
- It does not build the single judge: E3 touches the calls to `verify::<…>` of `apply_refund` and
  `apply_deissue` (entry 117), and is ordered with it.
- It does not decide the real «nunca», nor the frozen receiver, nor stopping producing v1: they are
  pieces (1), (2) and (4) of the note of entry 12.
- It does not decide 8 or 11.
- It does not build the medium or its witnesses (RFC-0013), nor the comparison of E5, nor the lower
  bound of when a head is composed.
- It does not agree with whoever carries the train that it travels in the `zkssl/0.5` cut (decision
  9).

## References

- `BACKLOG.md`, entries 12, 115 and 117.
- `AUDITORIA.md` §115, §119, §121, §128, §174, §178, §229, §268, §285, §292, §340, §350, §354,
  §388, §451, §452, §501, §609, §642, §779, §809, §815 and §861.
- `PRINCIPIOS.md` §0, §4.3 and §9; `doc/CADUCIDAD_PENDIENTE.md` §2 and §6; `doc/blueprint-v2.md`
  §«3.4 Arquitectura y API» (ARQ-01, ARQ-02).
- RFC-0003 (§«El compromiso», §«El invariante de reembolso, completo», with the mark of §861, D1,
  D2), RFC-0007 (D-B), RFC-0008 (the proof of the payment in flight), RFC-0009 (D-B), RFC-0012
  (D-A), RFC-0013 (D-C, D-D, D-E, D-F), RFC-0015 (D-F), RFC-0018 (the cut), RFC-0019, RFC-0020 (the
  v7) and RFC-0021 (D-B and its Compatibility).
