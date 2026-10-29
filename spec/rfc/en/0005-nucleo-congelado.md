# RFC-0005 — The frozen core and the extension rule

> **Español:** [`spec/rfc/0005-nucleo-congelado.md`](../0005-nucleo-congelado.md) is the original of
> this page, in Spanish; it prevails wherever the two differ. This translation is of commit
> `7766efb` (audit entry §917) and was produced by the project's assistant, a Claude Code session,
> in audit entry §918 (`GENAI.md`): it is a generated artefact, not text written by the author. Line
> citations of the form `file:N` refer to the Spanish files.

- **Status:** PROPOSED
- **Author:** Ángel José Toranzo Portela
- **GenAI assistance:** Claude (sessions 96-97) — see [`GENAI.md`](../../../GENAI.md)
- **Date:** 2026-09-05
- **Affected protocol version:** `zkssl/0.3` — **does not go up** (see Compatibility)
- **AUDITORIA entries:** §236, §243, §290, §395, §397, §398, §404 (the first seal of H3), and §405, which seals it; §693 hardens E4; §706 adds the hiding mark to the class «prueba mal formada» [malformed proof] and six negatives with a cause; §711 adds `prueba mal formada: {qué} fuera de rango` to the class, four negatives with a cause and the command's panic as ROJO with exit 1 (`PAQUETE.md` §6); §713 adds `FriVerificationFailed(RemainderCommitmentMismatch)` to the class «consulta que no casa con el compromiso» [query that does not match the commitment] and one negative with a cause; §715 adds `prueba mal formada: usize no canonico` and the tail (`UnconsumedBytes`, `not all bytes were consumed`) to the class «prueba mal formada», `FriVerificationFailed(LayerCommitmentMismatch)` to «consulta que no casa con el compromiso» and twenty-six negatives with a cause

## Status of the stages

| stage | what it delivers | does it break the wire? | status |
|---|---|---|---|
| E1 — the core, written | a normative document under `spec/` that enumerates WHAT does not change and WHY, with every piece tied by citation to the line of the code that produces it; the census is re-derived on sealing | NO | sealed — §407 (`spec/NUCLEO.md`, `tools/check_nucleo.py`) |
| E2 — the extension rule, written and exercised | the rule in the same document, and the tie that exercises it: a verifier accepts an EXPLICIT set of versions and rejects the rest (§404 is the first case) | NO | sealed — §406 |
| E3 — the wire's rejections, with a vector | negative vectors of what a consumer of the wire has to reject, under their own directory, with their gate in `tools/canon.sh`; what the consumer rejects today is MEASURED before a vector is written | NO | sealed — §409 (`spec/vectors/cable/`, `witness --respuesta`, `tools/cable_respuesta.sh`) |
| E4 — the conformance harness | a tool that runs the vector catalogue against ANY binary it is given and says whether it passes and fails the same as the reference | NO | sealed — §408 (`tools/conformidad.sh`, inside the tarball); hardened — §693 (the named cause, `NO SOPORTADO`, `PAQUETE.md` 6.1); §706: the hiding mark, with its text, in the class «prueba mal formada»; six new negatives in age and payment, without breaking the wire or rewriting any vector; §711: `prueba mal formada: {qué} fuera de rango`, with its text, in the same class; four new negatives in age and payment, which before made the command exit with 101; and a panic of the command, ROJO with exit 1; §713: `FriVerificationFailed(RemainderCommitmentMismatch)`, in the class «consulta que no casa con el compromiso»; one new negative in age, captured; §715: `prueba mal formada: usize no canonico` and the tail, in the class «prueba mal formada», and `FriVerificationFailed(LayerCommitmentMismatch)`, in «consulta que no casa con el compromiso»; twenty-six new negatives in age and payment |
| E5 — the criterion | a second implementation, written from the spec without reading the reference code, that passes and fails the same. It is not in the author's hands; E4 is what makes it checkable the day it exists | NO | outside the tree; what was in the author's hands, sealed — §411 (`spec/vectors/nucleo/`, `NUCLEO.md` section 6) |

Every measurement in this document was taken on `9ae055c` and `2080e5a` (§404), in pure
readings that wrote not one byte to the tree: `PASTE-H3-M`, `PASTE-H3-M2`, `PASTE-404-M`,
`PASTE-404-PRE-r2` and `PASTE-RFC5-M` (see References).

## Motivation

Milestone H3 of the proposal sent to NLnet (2026-11-009) says, literally: «*The format as a
minimal standard. Frozen core with rationale; extension rule; specification; positive and
negative conformance vectors; rejection gate. Success criterion: a second independent
implementation, written from the spec without reading the reference code, that passes and
fails the same way. Includes closing the one measured place where rejection is not yet
enforced.*» The last sentence was paid in §404. This RFC is the rest.

What the tree has today, measured:

- `spec/PAQUETE.md` is the only normative producer of the evidence package and of the command's
  contract (§397), and `spec/vectors/paquete/` carries **67 entries** —5 positives, 62
  rejections— with their gate in `tools/canon.sh` (§398, §399). It is the whole mould of
  «positive and negative vectors + rejection gate», already in production.
- `spec/RPC.md` is the specification of the wire, with three versions preserved under their own
  and the triple gate that demands `0.3` identical and rejects `0.2` and `0.1` (§354).
- `spec/rfc/PROCESO.md` sets five rules; rule 2 —old vectors are never rewritten— is half of an
  extension rule.
- The pieces a verifier has to reproduce exist and **declare themselves a conformance surface**
  in their comments: `preambulo` (`zk-ssl-verify/src/lib.rs:270-271`, «a second implementation
  has to produce these bytes»), `preambulo_cofirma` (`:284-286`), and the REGISTRY of domains of
  `zk-ssl-hash/src/lib.rs:743-757` that `tools/check_dominios.py` gates in every canon.

And what is missing, also measured:

1. **Nobody says what will NEVER change, nor why.** Each piece of the core lives in its file
   with its comment; there is no document that gathers them, justifies them and says which is
   the only gate through which they may change. A second implementer today has to read the Rust
   to know what is contract and what is a detail of the reference implementation; the milestone
   promises that «*a second implementer needs only the specification*» (v3, line 95).
2. **The extension rule exists as practice, not as text, and carries a tension.** Four
   precedents in `spec/RPC.md` (`:185`, `:441-442`, `:459-463`, `:586-587`, `:611`) and the
   normative text of `:864-867` say that the version goes up when the values that travel change,
   and that an additive field does not raise it. But 16 of the 22 types of `zk-ssl-wire` carry
   `deny_unknown_fields` and `:459-461` calls it design: «additive on the wire, breakage in the
   parser». One same format cannot say at once «adding a field does not break» and «an unknown
   field breaks». One has to choose, and write it down.
3. **The extension rule had a gap measured and sealed this week.** `witness.rs:826`
   returned conformance for every version outside {2, 3} (§404): the half «what is not known
   is rejected» was not written anywhere, and that is why it could be missing in one place.
4. **Negative vectors exist only for the package.** Of the wire there are three positive
   vectors; `conformance --check` re-runs the canonical scenario and compares field by field,
   and the only thing it rejects is «another version» (`conformance.rs:163-168`). Zero vectors of
   what a consumer of the wire must reject.
5. **The success criterion has nothing to be checked with.** The README publishes as «contract
   of the second implementation» (`README.md:60-62`) running `conformance --check`: that demands
   reproducing `proof_digest`, that is, the full STARK prover, that is, the reference code.
   Note 85 of the BACKLOG names it: as long as conformance is only checked against itself it is
   self-conformance. The 67 vectors of the package, on the other hand, any binary without a
   prover can pass — and today only `tools/canon.sh` runs them, on the tree's binary
   (`:296`).

## Design

### D-A — The core is the verifier's side, not the wire

What is frozen is **what a third party needs in order to verify without the node, without the
layer and without the prover** (§243): the evidence package and everything its verification
recomposes. The JSON-RPC wire remains the interoperability contract of `spec/RPC.md`, governed
by this same process, but **outside the frozen core**: it changes by RFC, goes up a version when
its values change, and demands of nobody that they reproduce a prover.

Reasons, all measured: the project's law asks for «a system that a supervisor can verify
without seeing the ledger»; line 95 of the proposal promises a second implementer who needs only
the specification, and the wire cannot fulfil it (point 5); the package already has a spec,
vectors and a gate (§397-§399) and its verifier closes in two crates and four direct
dependencies, with no network and no clock (§395); and the use cases reviewed in
`doc/USE_CASES.md` —the auditor, the supervisor, the holder after the shutdown— are all on the
verifier's side.

**The census of the core is derived, not typed.** E1 produces it with a pure reading over the
`pub` items of `zk-ssl-verify/src/lib.rs` and of the `zk-ssl-hash/src/lib.rs` that crate uses.
What is measured today, as a CANDIDATE and in the order in which a verifier consumes it:

| piece | where it lives today | why it cannot change |
|---|---|---|
| the canonical hex and the encoding of the digest (`0x`, `digest_to_bytes`) | `zk-ssl-hash/src/lib.rs:444-452`, `spec/RPC.md` «Codificación» | it is the boundary between the JSON and the bytes: two implementations that differ here agree on nothing |
| the hash permutation and the merge (`native_merge`, `as_digest`, `path_root`) | `zk-ssl-hash/src/lib.rs:100-181` | every root, every path and every custodied head is recomposed with them |
| the head compositions `epoch_digest` v1/v2/v3 | `zk-ssl-hash/src/lib.rs:200-272` | what is in custody does not expire (§290): an old composition must always be recomposable |
| the domains: the REGISTRY of `u64` and of `ZK-SSL-…` strings | `zk-ssl-hash/src/lib.rs:743-757`, `zk-ssl-verify/src/lib.rs:113,125` | they separate objects of the same hash; changing one would make them collide or would stop verifying what is signed |
| the preambles that are signed (`preambulo`, `preambulo_cofirma`) | `zk-ssl-verify/src/lib.rs:276-331` | they are the exact bytes under the signature; a second implementation «has to produce these bytes» (its own doc) |
| the version byte of the preamble and the accepted set {2, 3} | `zk-ssl-verify/src/lib.rs:132`, `main.rs:128`, `witness.rs:828` | it is the ONLY gate through which the core can grow (§236) |
| the signature scheme: `XmssMtSha2_40_8_256` and the embedded index (`ANCHO_INDICE`) | `zk-ssl-verify/src/lib.rs:109,348-370` | every head and every custodied co-signature is signed with it |
| the package envelope and its catalogue of rejections | `spec/PAQUETE.md` sections 3-5 | it is already normative (§397) |

What is **not** of the core even though it lives next to it, and the E1 document will say so
with the same clarity: `aplicar_apano_del_oid` and `OFFSET_MT_UPSTREAM` (`lib.rs:399-415`)
correct a bug of `xmss 0.1.0-pre.0` (§240) and belong to the reference implementation, not to
the format — a second implementation with a correct XMSS library must not replicate them; the
`canon` field of the wire vector (`conformance.rs:121`) is a snapshot of test figures, not a
value of the protocol; and everything the verifier does not recompose.

⚠️ §890: the line citations of this section are from `2080e5a`, the tree this document
declares (above, «Every measurement…»), and there they fall in their place: §890 checked them one
by one. In `212eca2` they no longer do: `preambulo` and `preambulo_cofirma` are at
`zk-ssl-verify/src/lib.rs:537` and `:574`, not at `:276-331`, and the same happens to the others.
They are not changed for today's, which the next seal would move again: each piece is found by
the name the first column gives, and the norm of the bytes is `spec/NUCLEO.md` (E1, §407), not
this table.

### D-B — The extension rule, in two halves

**First half — what is signed grows only by version.** Every composition that goes under a
signature carries a version byte; a new composition is a new version of the preamble, with its
vectors under its version, and the vectors of the earlier ones are never rewritten (rule 2 of
the PROCESO). A verifier of the core **accepts an explicit set of versions and rejects
everything else with text**: v1, v4, 0x103 are rejections, not conformance. §404 is the first
place where this half was enforced; E2 writes it and gives the accepted set **a single
producer** —today there are two, the command (`verify/main.rs:128`) and the witness
(`witness.rs:828`), tied by copy and not by test— and ties it with a test that enumerates it
there; the rejections consume it, they do not repeat it. A set with two producers is the class of
defect the Motivation takes a census of.

**Second half — what is not signed does not exist for the core.** An envelope may gain keys the
core does not know without raising its version, because the verifier **believes nothing it does
not recompose**: a key that enters no composition and no preamble cannot alter what is checked.
The verifier of the core ignores them — and the reference already does: the `zk-ssl-verify`
command carries not a single `serde` attribute and reads its 14 keys with `.get()` (measured in
session 92 on `main.rs`); an extra key does not change its verdict. This half is not introduced
by the RFC: it is what the binary published in `arqueo-verify-v0.1.0` does, declared. That the
reference implementation also rejects unknown keys in its consumer of the wire
(`deny_unknown_fields` in 16 types of `zk-ssl-wire`) is **a choice of that implementation,
declared as such** in `spec/RPC.md`, not a rule of the format.

The tension is declared, not hidden: the project's law also says «when in doubt, stop». Here
the two halves split the doubt: **on the version, fail-closed**; **on the unsigned keys,
indifference**, because there is no doubt to resolve — nothing of what is verified depends on
them. If the author prefers fail-closed on the keys too, the consequence is that every additive
field raises the version, `spec/RPC.md:868` stops being true and the published binary changes
its behaviour: it is a reversible decision and is written as such.

**New families.** A new object (another portable proof, another envelope) enters with its own
`tipo` or its own domain, its own vectors under its own directory and its own RFC — as the
extension package did (`tipo: "extension"`) and as the vectors of the package did with respect to
those of the wire (discard 5 of RFC-0004). The core is not touched to receive a family — unless
the family needs to enter **under the signature of the head**: then it does not enter this way
but through the first half, as a new version of the preamble with its vectors under its
version, and its cost (a new `epoch_digest` composition, the additive wire, the verifier reading
up to that version, the vectors) is measured before it is written. It is the path foreseen for
the uniqueness across ledgers of D-C.
⚠️ §914: the proofs that travel carry their identifier inside since `zkssl/0.5`, the v2 mark of
RFC-0018 (D-G): the family, the version of its AIR and the profile, with the registry of families in
`spec/NUCLEO.md` §4 bis; a new family or a new AIR version enters through that registry, and what is
not in it the judge rejects by name before reading the proof. It is the same extension rule —it is
signed and grows only by version— applied to the proof.

### D-C — The core identifies nothing outside the ledger

No frozen piece names, derives or transports an identifier from outside the ledger: not an
invoice, not a counter, not a label shared between operators. It is measured that the layer
persists no nullifier at all, that the 27 types of public inputs of the circuits take no
external identifiers and that the nullifier that exists is derived from the private spending
key precisely so that nobody precomputes other people's (§395, `PASTE-H2-M`). Uniqueness
**across** ledgers —the case of double certification between bodies that `doc/USE_CASES.md:65-69`
already declares as governance and not cryptography— would demand shared key material that
nobody holds whole, that is, a new trust assumption: it is a **new family** in the sense of
D-B, with its RFC, and the core is frozen so that it can exist without breaking it. This RFC does
not build it: today the witness that would falsify it cannot be written.

### E3 — The wire's rejections, with a vector

The reference's consumer of the wire is the witness (`zk-ssl-cli`), and it publishes its
rejection classes as stable text (`witness.rs`, `clase()`: `no-verifica`, `version-desconocida`,
`indice-discordante`, `campo-torcido`…, measured in `PASTE-404-PRE-r2`). E3 first measures which
responses of the wire it rejects today and with which class, and only then writes one vector per
rejection, under a directory of its own in `spec/vectors/` —never among the `zkssl-0.N.json`— and
its block in `tools/canon.sh` modelled on the «3 bis» of the package. The first candidate is the
one §404 has just closed: a head served with `formatVersion` outside {2, 3}.

§409 sealed it as it was written: it was measured first (`PASTE-409-S`: the real head re-verifies
and each mutation gives rc and text), the mouth is `witness --respuesta` and the harness is E4's
without touching a line of it, with `tools/cable_respuesta.sh` as adapter to the command's contract.

### E4 — The conformance harness for any binary

The block «3 bis · conformidad del PAQUETE» of `tools/canon.sh` already runs the tree's binary
against each entry of the package's MANIFIESTO (⚠️ §854: it said `:291-296`, its lines at §405;
since E4, the block calls the harness, `tools/conformidad.sh`). E4 takes that loop out into a tool
that receives **the path of any binary** and the MANIFIESTO, and says per entry whether the rc and
the text match what is expected. It is what makes E5 checkable: the day a second implementation
exists, the publishable result is the output of that harness on its binary, not a claim. Until
then, the harness on the reference binary is a publishable result in itself: the full catalogue,
run outside the canon, by anyone.

**Since §693** the harness demands, besides the code and the text, the cause: «fails the same» is
failing for the same one, and a text that only names the subject —`edad:`— was also passed by
whoever verifies nothing. An output that says the binary does not read the envelope
(`NO SOPORTADO`) never counts; the text of a negative has to be on the only line with `ROJO:`; a
vector whose entries only pin the subject comes out `SIN CAUSA`; and the binary runs with empty
standard input. The table of the causes lives in `PAQUETE.md` §«6.1 La causa del rechazo». It
does not break the wire nor rewrite a vector: eighteen new lines in eight manifests, three of
them of `0.3/`, measured with their kit.
§706: the hiding mark, with its text, in the class «prueba mal formada»; six new negatives in
age and payment, without breaking the wire or rewriting any vector.
§711: `prueba mal formada: {qué} fuera de rango`, with its text, in the same class; four new
negatives in age and payment, of ONE byte of the proof, which before made the command exit with
101, a code that is none of the four of `PAQUETE.md` §6; and a panic of the command comes out as
ROJO with exit 1 (§6). Without breaking the wire or rewriting any vector: sixteen new lines at
the end of two manifests.
§713: `FriVerificationFailed(RemainderCommitmentMismatch)`, in the class «consulta que no casa con
el compromiso»; one new negative in age, captured from a real node and not derived, which before
gave VERDE. Without breaking the wire or rewriting any vector: twelve new lines at the end of one
manifest.
§715: `prueba mal formada: usize no canonico` and the tail of the proof, of an opening or of the
batch of a FRI layer (`UnconsumedBytes`, `not all bytes were consumed`), in the class «prueba mal
formada», and
`FriVerificationFailed(LayerCommitmentMismatch)`, in «consulta que no casa con el compromiso»;
twenty-six new negatives in age and payment, of ONE mutation of the proof each, nine of them
VERDE before §715. Without breaking the wire or rewriting any vector: fifty-nine new lines at the
end of two manifests.

### E5 — The criterion, and what it lacked

E5 is not in the author's hands and this RFC does not seal it. What was in his hands was
measured and closed in §411: the sentence sent to NLnet —«a second implementer needs only the
specification»— was false as of §410, because the bytes of the core lived in doc-comments of
the code and in a dependency (`winter-crypto`) that `spec/` did not name. Since §411 the KATs of
`spec/vectors/nucleo/` are the norm of the bytes, `NUCLEO.md` section 6 names the permutation, the
serialization, the domains and the preambles, and a test of `zk-ssl-cli` reproduces them in every
canon. A second implementer checks its hash against the KATs without reading the reference; E4
then says whether it passes and fails the same. What no KAT gives is the signature: RFC 8391.

## What was DISCARDED on measuring

1. **Freezing the wire.** Discarded: reproducing it demands the prover (`proof_digest`), hence
   demands reading the reference code; it contradicts line 95 of the proposal and note 85.
2. **Rejecting unknown keys as a rule of the format.** Discarded (reversible): it would make
   every additive field a breakage and would leave the normative text of `spec/RPC.md:868` and
   its four precedents false. It remains as a declared choice of the reference implementation.
3. **Putting the core into `spec/RPC.md`.** Discarded: `spec/RPC.md` specifies what crosses the
   wire, and the core is what is verified without a wire (discard 2 of RFC-0004, same reason).
4. **Including the `xmss` OID workaround in the core.** Discarded: it is a patch over a bug of
   a pre-release library (§240); freezing it would force every second implementation to
   reproduce someone else's defect.
5. **Taking the `canon` field of the wire vector as part of the protocol.** Discarded: it is a
   snapshot of test figures (§284); a second implementation could only copy it. It stays on
   file for the wire front.
6. **Building the identity across ledgers inside this RFC.** Discarded by the short rule of the
   law: the witness that would falsify it cannot be written today (there are no two ledgers nor
   a shared label). It is a new family, with its own RFC.
7. **Fixing here the name of the E1 document.** Discarded by the same measurement as in
   RFC-0004 (discard 7): `tools/verificar_citas.py` demands that every cited `.md` exist, and an
   RFC cannot cite what does not yet exist. The name is fixed in E1, in the commit that creates it.
8. **Calling any of this a «standard».** Discarded until E5 is met: the project decided in
   session 83 that that word is not written until the second implementation that passes and
   fails the same exists. This RFC says «core» and «contract».

## Compatibility

| stage | does it break the wire? | vectors |
|---|---|---|
| E1 — the core, written | **NO.** It documents what already is; no value that travels changes | the three of the wire, intact; the 67 of the package, intact |
| E2 — the rule, written and exercised | **NO.** The accepted set {2, 3} is already the one the tree applies since §404 | intact |
| E3 — the wire's rejections | **NO.** New vectors under a directory of their own | those of the wire and those of the package, intact; a separate catalogue is born |
| E4 — the harness | **NO.** Tool, not format | intact |

**Rule 2 of the PROCESO: old vectors are never rewritten.** `zkssl-0.1.json`,
`zkssl-0.2.json`, `zkssl-0.3.json` and the 66 files of `spec/vectors/paquete/` are preserved
without one modification.

### Why it goes through an RFC if it does not break the wire

By the letter, the scope of the process (`spec/rfc/PROCESO.md:3-6`) enumerates `spec/RPC.md`,
`spec/openrpc.json` and the vectors of `spec/vectors/`: E1 touches `spec/` (a new document and
the declaration in `spec/RPC.md` of the reference's choice on unknown keys) and E3 adds under
`spec/vectors/`. By the spirit, it is the first time the project writes what will not change:
that, if it does not go through an RFC, goes through nothing. `spec/openrpc.json` is neither
touched nor regenerated: there is no new method; the turn to ACCEPTED will declare it as RFC-0004
did.

## Security

**Effect on the principle of the API —the spending key never travels: NONE.** This RFC neither
adds nor changes any method, any type of the wire or any material that travels.

**Effect on the declared debts:**

- **Out-of-band notice (§21): none.** Nothing in the core transports notices.
- **Single node: none.** This RFC introduces consensus at no stage, deliberately
  (`SECURITY.md` §6). Freezing what a third party verifies without the node is the opposite of
  depending on it.
- **`--dev`: none.** The verifier has no features and no test custodians.

**Freezing publishes nothing new.** Every piece of the core is already public, already in the
tree and already exercised by a gate. What this RFC adds is the written promise that it will not
change, and the only gate through which it can grow. **The operator is still another
adversary**: the core prevents it from forging what is in custody; it does not prevent it from
omitting or ordering (`doc/USE_CASES.md`, «What none of this claims»).

## References

Measured on `9ae055c` and, after §404, on `2080e5a`; fingerprints in `sha256 | cut -c1-16`,
lines in `wc -l`.

- `NLNET-form-answers-EN-v3.txt` `26dcde32091e857d` / 160 — the text of H3 (line 42) and the
  promise of the second implementer (line 95). Outside the tree, in the expediente [the dossier].
- `spec/rfc/PROCESO.md` `0eba90e3fe5d93f8` / 24 ·
  `spec/rfc/0000-plantilla.md` `488e7055cbfcad09` / 29 ·
  `spec/rfc/0004-paquete-de-evidencia.md` `53ea2afb28ba92d8` / 270 — the mould.
- `spec/PAQUETE.md` `1278699dc460aa44` / 283 — sections 3-5 (`ea39d85164e9b6c9`,
  `8b603a4ef60d139e`, `6dbda8c65a1b2bf7`) and 8 («lo que este documento NO afirma» [what this
  document does NOT claim]).
- `spec/vectors/paquete/MANIFIESTO.txt` `9dafd1f061b52dc8` / 71 — 5 positives, 62 rejections.
- `spec/RPC.md` `cb40263b3b91237c` / 867 — the parser rule 455..466 (`88a03af26f9e5016`) and
  the normative text on the version 856..867 (`cfd8fc6b0e642463`).
- `crates/zk-ssl-verify/src/lib.rs` `4327e89dd517dc58` / 1068 — `preambulo` 270..282,
  `preambulo_cofirma` 284..331, `Conjunto` :109, `DOMINIO` :113, `DOMINIO_COFIRMA` :125,
  `VERSION_FORMATO` :132, `ANCHO_INDICE` :370, the OID workaround 399..415, `verificar_cabeza`
  494..530.
- `crates/zk-ssl-hash/src/lib.rs` `2f05cfd9fcf7e690` / 896 — `native_merge` :100, `as_digest`
  :130, `path_root` :181, `epoch_digest` :200 / v2 :224 / v3 :257, `acuse_digest` :333,
  `mmr_hoja` :374, `mmr_nodo` :380, `digest_to_bytes` :444, the REGISTRY 743..757.
- `crates/zk-ssl-wire/src/lib.rs` `9669d5d02050ad70` / 1354 — 22 types, 16 with
  `deny_unknown_fields`.
- `crates/zk-ssl-cli/src/witness.rs` `7d1da2575eb0dc06` / 4647 — `recomponer` with the guard of
  §404; the classes of `clase()`.
- `crates/zk-ssl-cli/src/conformance.rs` `c308b7576c074781` / 209 — what `--check` compares
  (:163-205).
- `doc/USE_CASES.md` `22fcbb837ae7df72` / 137 · `ROADMAP-ECOSISTEMA.md` `6008684d123173c3` / 56 ·
  `README.md` `a6d412397da863a6` / 569 (:60-62, :74-89) · `BACKLOG.md` `c2db6c5e38a442a2` / 3235
  (note 85, :642-658).
- Pure readings of session 96: `PASTE-H3-M` `0fd433f8c96ff378` (output `5d0caab078ab9dd2` / 383) ·
  `PASTE-H3-M2` `b510877df2c85375` (`f5f6c9aca96bd222` / 797) · `PASTE-404-M` `c8e5c40a20598301`
  (`da0aa8ae2b2d24d0` / 199) · `PASTE-404-PRE-r2` `6c4385ff7069f4d3` (`db513d279f11f8e5` / 208) ·
  `PASTE-RFC5-M` `7967b37e9e29b8f2` (`f4abd1572c2c8567` / 1192).

**Numbering.** **RFC-0001 remains reserved** for the hardening of the keystore's KDF, unwritten
(`spec/rfc/0002-lotes-y-transicion-de-hoja.md:39`). This RFC takes **0005** by sequence, after
0004.

**Provenance of a carried-over citation.** The measurement of single use (no nullifier
persisted, 27 types of public inputs without an external identifier, derivation from the private
key) was taken in session 91 on `bb5322f` (`PASTE-H2-M`); it is cited because §395 sealed it and
none of the later seals touched those producers.

**Delegated decisions (r2, session 97).** The author delegated to the assistant the scope
decisions D-A, D-B and D-C and the three precisions of r2 (the accepted set with a single
producer; the second half of D-B as a measured fact of the binary; the entry of a family under
the signature through the first half), with the instruction to apply the project's principles
and manifesto. All are REVERSIBLE and §405, which seals this RFC, records them.
