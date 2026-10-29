# RFC-0012 — The anchor of heads: the published history where the operator cannot erase it

> **Español:** [`spec/rfc/0012-el-ancla-de-cabezas.md`](../0012-el-ancla-de-cabezas.md) is the
> original of this page, in Spanish; it prevails wherever the two differ. This translation is of
> commit `7766efb` (audit entry §917) and was produced by the project's assistant, a Claude Code
> session, in audit entry §918 (`GENAI.md`): it is a generated artefact, not text written by the
> author. Line citations of the form `file:N` refer to the Spanish files.

- **Status:** ACCEPTED (§607) — **rule 4 of the PROCESO, settled with measurement**, which is what
  this line asked for: «its canon and its merge». (1) The spec: `PAQUETE.md` 2.12, the anchor
  envelope (§592), and in `NUCLEO.md` the REGISTRY with `DOMINIO_ANCLA`, `ancla_digest` and
  `huella_de_clave` (§591). (2) The OpenRPC does not move: not one method nor one field. (3) The
  vectors, new under `zkssl/0.4`: `nucleo/ancla_digest.json` and `nucleo/huella_de_clave.json`
  (§591) and the `ancla/` family, twenty-one envelopes (§593). (4) The suites: the author's
  `--sello` canon, VERDE at every seal since its merge with «ancla 21/21» from the tree and from
  inside the tarball, and `tools/banco_ancla.sh` VERDE in its `--bancos` of 20 on `639b5eb`. The
  four stages were built in session 194 (E1 §590, E2 §591, E3 §592, E4 §593); decisions D-A to D-H
  were DELEGATED and REVERSIBLE, and are accepted with their residue D-H: the anchor still has no
  medium to live in -RFC-0013 proposes it-.
- **Author:** Ángel José Toranzo Portela
- **GenAI assistance:** Claude (session 194, which writes it whole; session 193, the turn of §607) —
  see [`GENAI.md`](../../../GENAI.md)
- **Date:** 2026-09-30
- **Protocol version affected:** `zkssl/0.4` — **it does not move it, and it does not move the head
  either**: not one new method, not one new field, not one new byte under the signature. The family
  enters through its `tipo`, its domain and its vectors, which is the way `NUCLEO.md` §3 reserves to
  the new families that do not need to enter under the signature.
- **AUDITORIA entry(ies):** §174 (the design, measurement 10, `doc/ANCLAJE_EXTERNO.md`), §116 (the
  digest that collides with trailing zeros; here, the key fingerprint with encoded length), §121
  (the ceiling `N_max` and its clock), §248 (the root served by the accused proves nothing; two
  journals prove that it emitted two things for the same index), §291–§292 (the MMR of heads and its
  signed pair), §399 (the embedded index, the only one the signature attests), §567 (the clock is
  the XMSS index, not the `seq`), §580 (the house does not breed code without a caller); and those
  of this series: §590 (E1), §591 (E2), §592 (E3), §593 (E4); and §607 (the acceptance); §712 takes
  the clock of D-C to the extension, to the consumption, to the mode 3 anchor and to the witness,
  with nine new vectors; and §714 adds to D-E mode 5, the fork with the same `mmrSize`, with
  twenty-one new vectors.
- **Backlog:** B10.6 (the anchor format + `M`) and B10.7 (the client-side verifier) of the backlog
  of `doc/CONFIANZA_RESIDUAL.md`; the portable half of B10.2 (the fraud proof); and it composes with
  entries 83 and 86 of the BACKLOG (the anchor bounds the window of the first encounter).

## Status of the stages

| stage | what it delivers | does it break the wire? | status |
|---|---|---|---|
| E1 — the promise, written | this text and `doc/ANCLAJE_EXTERNO.md` rebuilt: which object is born (D-A), its fingerprint and the key's (D-B), the embedded clock (D-C), the batch that already existed (D-D), the split view inside (D-E), the cadence `M` without a constant (D-F), what it reveals (D-G) and the residue (D-H) | no | **built (§590)** |
| E2 — the anchor in the core | `DOMINIO_ANCLA` in the REGISTRY, `huella_de_clave` and `ancla_digest` in `zk-ssl-hash`, their KATs in `spec/vectors/nucleo/` and their census rows | no (additive) | **built (§591)** |
| E3 — the envelope, in the command | `tipo: "ancla"`, eleventh form: derive, exact anchor, earlier anchor by MMR consistency, and the split view; each rule with its name | no | **built (§592)** |
| E4 — catalogue and bench | `spec/vectors/ancla/` with its MANIFIESTO, the tenth family in `FAMILIAS`, its canon stanza and `tools/banco_ancla.sh`, which seeds the split view by reproducing the real attack | no | **built (§593)** |

## Motivation

Residue #1 (order and completeness, `SECURITY.md` §2.bis) has all its BOUNDING pieces built: the
signed head, the acknowledgement ceiling, the reception receipt and its envelope (RFC-0010). All
assume that someone external keeps heads since before the conflict. The ELIMINATION way that §174
designed —publishing what the head already signs in an external append-only medium— had been since
then in a document whose content, moreover, the published tree lost (S548: a placeholder). This RFC
executes that design: B10.6, the anchor format; B10.7, its client-side verifier. What is left out,
and is said: choosing the medium and deploying the cadence.

## Design

### D-A — The anchor is a DERIVABLE object, not a method of the wire

A `zkssl_ancla` method would be §248 again: a root served by the accused proves nothing. The anchor
is derived from the signed head alone, and **anyone** can derive it —the operator who publishes it,
the witness who watches him, the holder who distrusts both—. The producer is the command itself
(`zk-ssl-verify`), which with the head alone prints the anchor and its fingerprint: the B10.6
component is the binary that already travels in the kit. The wire is not touched.

```text
ancla = { v: 1,
          clave:       huella_de_clave(publicKey),
          indice:      el índice XMSS EMBEBIDO en la firma,
          epochDigest: el digest que la firma cubre,
          mmrRoot:     la raíz del árbol de cabezas,
          mmrSize:     cuántas contiene }

huella_del_ancla = ancla_digest(clave, indice, epochDigest, mmrRoot, mmrSize)
                 = merge(as_digest(ANCLA_V1),
                     merge(clave,
                       merge(as_digest(indice),
                         merge(epochDigest, merge(mmrRoot, as_digest(mmrSize))))))
```

What is published in the medium is the anchor —six JSON keys— or, where only 32 bytes fit, its
fingerprint. Whoever has the fingerprint demands the anchor from whoever published it and recomposes
it.

### D-B — The key fingerprint, with encoded length and a byte domain of its own

`huella_de_clave(clave) = Blake3(b"ZK-SSL-anchor-key-v1" ‖ len(u64 LE) ‖ clave)`, the exact mould of
`digest_of_proof` with its §116 correction —two keys that differ in trailing zeros do not collide—
and a new byte domain in the REGISTRY. The fingerprint goes through the medium and not the whole
key: a published XMSS-MT key weighs what it weighs, and 32 bytes fit anywhere. Whoever compares
always has the whole key (it travels in every signed head): recomputing the fingerprint is a merge,
not a trust.

### D-C — The anchor's clock is the EMBEDDED index, not the declared one

The signature attests a single index: the one embedded in its first five bytes (§399). The declared
one is only bounded **from below** —two honest heads can declare the same one—, so an anchor over
the declared one would fabricate false split views. The anchor commits the embedded one, the command
extracts it with `indice_de_firma`, and it is the same clock the receipt's era uses since §567: the
key only signs heads, each signature burns an index, and a censor does not freeze it.

§712: the same clock, outside the anchor. With the same `mmrSize`, the consistency of §291 is the
identity, and it did not see two distinct heads at the same place in the history: a signed fork
passed as extension, as consumption and as mode 3 anchor. After consistency, the judge
`el_reloj_avanza` requires the same `epochDigest` with the same `mmrSize` and, with the same key, an
embedded index of the new one strictly greater than the old one's (`PAQUETE.md` 2.3; in mode 3 the
index was already required to be earlier, 2.12). With distinct keys it does not compare: the
crossing is judged by the stretches of the chain. The witness counts by the embedded one, one count
per operator, and applies the same rule to every head that arrives. Nine new vectors in `paquete/`,
`consumo/` and `ancla/` (`PAQUETE.md` section 9); none is rewritten, and the wire does not move. The
fix and this paragraph were drafted in private (`SECURITY.md` §5, `spec/rfc/PROCESO.md` §«Fallos de
seguridad»).

### D-D — The batch is the tree the head already signs

§174 asked for «the root of a tree of heads every `M` heartbeats». That tree exists since §292: the
pair `(mmrRoot, mmrSize)` travels inside the signed digest, with inclusion and consistency O(log N)
in the verifier (§291, RFC 6962). Anchoring a head anchors its `mmrSize` previous ones —the
individual anchor of each one is its Merkle path— and ties the later ones by consistency. **Nothing
new is built, persisted or signed**: zero extra XMSS indices, the correction that saved the
acknowledgement (§121.2), inherited whole.

### D-E — The split view enters this family

Two heads of the same key with the same embedded index and different contents are the condemning
pair that `CONFIANZA_RESIDUAL.md` §2.1 promised: only whoever has the key can produce it, because
producing it is reusing a one-time signature. §248 left it measured —two journals «prove that the
operator emitted two signed things for the same index»— and the portable object was missing. It
enters here and not in a family of its own because it is the other face of the anchor: the anchor
forces the operator to choose ONE history; this envelope is what awaits him if he chose two. It is
the portable half of B10.2, and the envelope prints it as DETECTION with exit 0, the mould of the
ledger conflict (§430): the envelope that exhibits it does not fail — it denounces.

§714: the sibling of the split view. Two heads of the same operator with the same `mmrSize` and
different digests are two heads at the same place in its history: with different leaves they reuse
no index, and the split view does not see them. The anchor envelope gains a mode 5, with `rama`: the
other head, whole, of the same operator —the same key, or keys that the `actas` join, with the
continuity judge—; the same `mmrSize`, with any peak and also at genesis; and different digests. It
comes out DETECTION with exit 0, like mode 4: «bifurcación o reinicio sin diario» [fork or restart
without a journal]. It does not distinguish the two, and there is no need: an honest node with its
journal does not repeat an `mmrSize`. It has two declared gaps in the honest node: a power cut
before the line reaches the disk, and an `anotar` of the journal that fails (`PAQUETE.md` 2.12). The
witness stops it with its class, `bifurcacion`, and `--auditar` and `--comparar` assemble the
envelope. Twenty-one new vectors in `ancla/` (`PAQUETE.md` section 9); none is rewritten, and the
wire does not move. The fix was made in private (`SECURITY.md` §5, `spec/rfc/PROCESO.md` §«Fallos de
seguridad»), and this paragraph was also drafted in private, when porting it.

### D-F — The cadence `M` is declared in prose; the constant waits for its caller

`M` is a systemic line —family of `N_max` (§121) and `T` (`doc/CADUCIDAD_PENDIENTE.md`)—: it is
chosen, published, measured. The proposal, until there is a real operator: **`M` = 1.440
heartbeats**, one anchor a day, the horizon of `N_max` and CT's MMD. **No constant is born in the
code**: its only caller would be the deployment component —the one that publishes in the chosen
medium—, which this RFC leaves out on purpose, and the house does not breed code without a caller
(§580). When that component exists, `M` enters with it and with its watchman, as `N_MAX_CABEZAS` has
its own.

### D-G — What the anchor reveals

Nothing the signed head does not already publish: the five fields are the public key (by its
fingerprint, derivable by anyone who has the key), the index the signature carries in the clear, and
three values that travel in every `zkssl_signedEpochHead`. Publishing the anchor in an external
medium reveals **that that operator exists and at what pace it signs** —that is exactly the
property— and its approximate volume of epochs, which the `mmrSize` already published to whoever
asked.

### D-H — The residue, declared

The three of §174, intact: the medium is trust **displaced, not eliminated**; the tail between
anchors is bounded by `M` and N2, **not closed**; and the anchor proves **which history was
canonical, not that it was complete** — completeness still belongs to the receipts and to
non-inclusion (§121.3, RFC-0010). And two of its own: the genesis anchor (`mmrSize` 0) has no
history to extend —it is compared whole or waits for the next one—; and as long as nobody publishes
in a real medium, the anchor is a correct object with nowhere to live: **pending deployment**,
exactly what `SECURITY.md` §2.bis says.

## What was DISCARDED on measuring

- **A method of the wire that serves the anchor.** §248, and it would moreover move the published
  cardinals without gaining anything: the signed head already travels and the anchor is derived from
  it.
- **Anchoring the whole key instead of its fingerprint.** It does not fit in the 32–80-byte media
  and adds nothing: the fingerprint with encoded length ties the key byte by byte.
- **The declared index as the anchor's clock.** Bounded only from below (§399): two honest heads can
  share it, and a false split view would kill the whole family.
- **A tree of anchors of its own.** A second producer of the tree of heads that already exists
  (§292); two implementations of the same problem can disagree.
- **A constant `M` in the code, today.** Without a publisher there is no caller (§580, D-F).

## Compatibility

The wire **does not move**: `zkssl/0.4`, thirty-one methods, OpenRPC intact. The head **does not
move**: `VERSION_FORMATO` stays at 6 and nothing new enters under the signature. The family enters
with its `tipo` («ancla»), its domain (`ANCLA_V1` and `ZK-SSL-anchor-key-v1`, both in the REGISTRY),
its compositions in the core with their KATs, and its vectors under `spec/vectors/ancla/` — the way
of `NUCLEO.md` §3 for the families that do not need to enter under the signature. The old vectors
are not touched. The envelope reads heads v3 to v6 —those that carry the MMR pair—, and what is kept
does not expire (§290).

### Why it enters through an RFC

It adds a family of vectors, an object to the evidence package and three elements to the frozen
core. It is exactly what the PROCESO reserves for an RFC, and what RFC-0010 did.

## Security

- **The API principle is preserved**: the spending key does not travel. The anchor carries the
  fingerprint of a public SIGNING key, two numbers and three digests; no private key of anything
  comes near this.
- **The anchor does not attest the content**: that the anchored history was the canonical one does
  not say it was complete or fair (D-H). A verifier that reads VERDE in an anchor envelope knows
  which history the operator signed; what was inside is said by the other ten forms of the package.
- **The split view requires BOTH signed heads.** Whoever only has one proves nothing, and the
  envelope says so: the operator's silence with one of the parties is still residue D-H of RFC-0010.
  What this object changes: emitting both is no longer free.
- **The anchor's date is the medium's word.** The envelope verifies the cryptography; the external
  order —that the anchor was published BEFORE— is given by the chosen medium, and that trust is
  displaced and declared (D-H). This RFC does not choose the medium (B10.6: «it does not choose a
  medium»).

## References

`doc/ANCLAJE_EXTERNO.md` (the design, rebuilt in §590), `doc/CONFIANZA_RESIDUAL.md` §2.1, §8 and its
backlog B10.*, `SECURITY.md` §2.bis and §6, `spec/NUCLEO.md` (the MMR and the domain registry),
`spec/PAQUETE.md` 2.12 (the envelope's form), entries 70, 83 and 86 of the BACKLOG, RFC-0010 (the
mould of a family that enters without moving the wire) and the entries of the header.
