# Evaluation: a post-quantum signature of fewer than 100 bytes, stateless

> **Español:** [`doc/firma-corta-evaluacion.md`](./firma-corta-evaluacion.md) is the original of
> this page, in Spanish; it prevails wherever the two differ. This translation is of commit
> `7766efb` (audit entry §917) and was produced by the project's assistant, a Claude Code session,
> in audit entry §918 (`GENAI.md`): it is a generated artefact, not text written by the author. Line
> citations of the form `file:N` refer to the Spanish files.

**The question, as it was asked** (30-09-2026): with what the house handles —the hash-based,
stateful signature of heads, the guardian of its index, the Merkle trees of the acknowledgement and
of the consumption, and the STARK proofs of the payment path—, can a signature resistant to
quantum computers be designed **of fewer than 100 bytes**, «against Dilithium's 2,400», **without
losing security and without state trees**?

**Short answer: no.** The three conditions together collide with an arithmetic this tree already
has written in a test, and everything that today goes below 100 bytes rests on a family of
assumptions the house rejected in `AUDITORIA.md` §106 (with the exception of §705 for the medium of
the anchor, ML-DSA-44, which is lattice-based and not of that family). What does come out of the
house's pieces is something else, smaller and measurable, and it is named in §6.

**Status**: evaluated, no code. Measured on `da6a768` (S588) on the session's machine.

**Provenance**: written in a working session with the assistance of generative AI, on a branch
that was not merged, and integrated in the author's name in §597 of `AUDITORIA.md`. «Integrar no
es aceptar» (integrating is not accepting): the acceptance that [`GENAI.md`](../GENAI.md)
describes remains pending.

**Convention**: ✅ measured in this tree, with the command beside it · 📐 derived with an
arithmetic that can be reproduced with the script below · 📖 read in the primary source, with a
citation · ⚠️ nuance · **«mirror»** = the original was not reachable from this session's network
(the proxy returned 403 for nist.gov, ietf.org, rfc-editor.org, eprint.iacr.org and arxiv.org) and
a copy **identical in two independent mirrors** was read; checking against the original is a
declared pending item (§8).

---

## 1. What the house has, measured in this tree ✅

The 341 JSON vectors of `spec/vectors/` are what the house publishes and what a third party
verifies with the node off. Weighed piece by piece, in **binary bytes** (on the wire they go in
hex, twice as much):

| piece | bytes | where it is measured | occurrences |
|---|---|---|---|
| XMSS signature of the epoch head | **18,519** | `FIRMA_RFC_BYTES` (18,469, the figure of RFC 8391 for `XMSSMT-SHA2_40/8_256`) + 50 of the preamble `ZK-SSL-epoch-head` ‖ version ‖ digest | 314 |
| co-signature of a witness | **18,593** | the same signature + 124 of the co-signature preamble | 11 |
| acknowledgement or receipt path, 32 levels | **1,024** | `RECEP_DEPTH` = 32 in `zk-ssl-verify`; 32 siblings × 32 B | 74 vectors |
| path of the consumption tree, 63 levels | **2,016** | 63 siblings × 32 B | 45 vectors |
| STARK proof of a transition | **43,092 to 78,451** | per vector; the band per payment, two proofs, is `PUBLICADA_PAGO_MIN_B..MAX_B` in `metrics.rs` | 99 |

To reproduce, from the root of the repository:

```python
# Mide, sobre spec/vectors/**/*.json, lo que pesa cada pieza de evidencia que la casa publica.
import json, glob, collections
alturas, firmas, cofirmas, pruebas, n = collections.Counter(), collections.Counter(), collections.Counter(), [], 0
def rec(o, ruta=""):
    if isinstance(o, dict):
        if isinstance(o.get("siblings"), list): alturas[len(o["siblings"])] += 1
        for k, v in o.items():
            if k == "signature" and isinstance(v, str): firmas[(len(v) - 2) // 2] += 1
            if k == "firma" and isinstance(v, str) and "cofirmas" in ruta: cofirmas[(len(v) - 2) // 2] += 1
            if k in ("prueba", "proof") and isinstance(v, str) and len(v) > 1000: pruebas.append((len(v) - 2) // 2)
            rec(v, ruta + "/" + k)
    elif isinstance(o, list):
        for v in o: rec(v, ruta)
for f in sorted(glob.glob("spec/vectors/**/*.json", recursive=True)):
    try: d = json.load(open(f))
    except ValueError: continue
    n += 1; rec(d)
print("vectores JSON leidos:", n)
print("camino: niveles -> vectores:", dict(sorted(alturas.items())))
print("firma de cabeza (bytes -> ocurrencias):", dict(firmas))
print("cofirma de testigo (bytes -> ocurrencias):", dict(cofirmas))
print("pruebas STARK: n =", len(pruebas), "min", min(pruebas), "max", max(pruebas))
```

Output on 30-09-2026: `{2: 1, 3: 1, 31: 3, 32: 74, 62: 1, 63: 45}` · `{18519: 314}` · `{18593: 11}`
· `n = 99 min 43092 max 78451`.

⚠️ **Re-measured on integrating (§597), with this same script on `be8aa31`:** 364 vectors read;
the head signature, 18,519 B in 340 occurrences —the anchor of §593 added 26—; the paths, the
co-signature and the STARK proofs, identical. The sizes do not change; only the count, which is of
its date.

**What the table teaches before looking outside.** The house already has **two** signatures of
the hash family, and both are hundreds of times away from the 100 bytes:

- **Stateful**: XMSS^MT, 18,469 B per head. The «state tree» the question wants to avoid is
  exactly the one the house pays for with the guardian of the index (§234) so that a signature is
  multi-use.
- **Stateless**: the spending authority is not a classical signature, it is **knowledge of a
  preimage proved in a STARK** (`SECURITY.md` §3, `AUDITORIA.md` §117). It is a signature in the
  strict sense, hash only, with no index to spend, and it weighs between 43 and 78 KB per
  transition.

And the formula that joins them is already in a test:
`el_ancho_del_indice_sale_del_conjunto_de_xmss`, in `crates/zk-ssl-verify/src/lib.rs`, binds
`SIG_LEN` to `índice + n + d·(2n+3)·n + h·n` with the formulas of RFC 8391. That `(2n+3)·n` =
67 · 32 = 2,144 B 📐 is a single one-time signature, **before** any tree (the RFC writes
`len = 67`, Table 1; the product is derived).

---

## 2. The arithmetic that closes the hash path 📐

Every hash-based signature reveals chains of preimages. In Winternitz (WOTS+, RFC 8391 §3.1.1),
with elements of `n` bytes, a digest of `ℓ` bits and parameter `w`:

- `len_1 = ⌈ℓ / log2 w⌉` chains for the message;
- `len_2 = ⌊log_w(len_1 · (w − 1))⌋ + 1` chains for the checksum;
- signature = `(len_1 + len_2) · n` bytes; verifying costs, in the worst case,
  `(len_1 + len_2) · (w − 1)` calls to the hash.

With `n = 32` and `ℓ = 256`, in exact integers:

| `log2 w` | `len_1` | `len_2` | `len` | OTS signature | hashes per verification, worst case |
|---|---|---|---|---|---|
| 4 (the RFC) | 64 | 3 | **67** | **2,144 B** | 1,005 |
| 8 | 32 | 2 | 34 | 1,088 B | 8,670 |
| 16 | 16 | 2 | 18 | 576 B | ≈ 1.2 · 10⁶ |
| 32 | 8 | 2 | 10 | 320 B | ≈ 4.3 · 10¹⁰ |
| 64 | 4 | 2 | 6 | 192 B | ≈ 1.1 · 10²⁰ |

The RFC's row gives `len = 67`, which is Table 1 of RFC 8391 and the `2n+3` of the test of §1: the
formula is right.

**The 100-byte threshold.** With 32-byte elements, **3** fit. With the RFC's formula `len_2 ≥ 2`
as soon as `len_1 ≥ 2`, so `len` **is never 3**: the first `w` with `len ≤ 3` is `log2 w = 256`
(`len_1 = 1`, `len = 2`, 64 B). Without a checksum, a variant the RFC does not have, `len_1 ≤ 3`
demands `log2 w ≥ 86`. In either of the two cases verifying costs **more than 2⁸⁵ calls to the
hash per chain**. It is not a limit of ingenuity: it is that each 32-byte element can only
compress `log2 w` bits of the digest, and `w` is the cost of walking it.

And this is **only the one-time signature**. Making it multi-use adds either the authentication
path and the index (XMSS: `h · n` more, and the state), or the hypertree and FORS (SLH-DSA:
7,856 B in its shortest parameter set, §3).

The ways out that do not hold, and why:

- **A shorter digest or 16-byte elements.** With Grover, 128 bits of post-quantum second-preimage
  resistance demand 256 bits of output; going down to 16 bytes leaves 64. It violates «without
  losing security», and the house has already measured what a 63-bit ceiling costs (`SECURITY.md`
  §3).
- **Compressing the signature with a succinct proof.** It is what the house does with the
  spending authority, and it weighs 43 to 78 KB (§1). The 128-to-192-byte proofs are Groth16 over
  pairing curves (192 B on BLS12-381, Zcash's specification), and the house discarded them for
  that reason (`FIVE_BACKENDS.md`). The post-quantum ones, read at source (conversions of the
  ePrint PDFs, mirror): LaBRADOR, 58 KB for an R1CS of 2²⁰ constraints at 128 bits (lattices);
  Greyhound, 53 KB (abstract); WHIR-CB, 157 KiB, against 306 KiB of FRI in the same configuration
  (hash); Aurora, from 40 to 130 kB (hash). **The smallest of 2025-2026 found**, LUNA+, 4.22 KB
  (by abstract, no primary reading), is designated-verifier and with a CRS of 0.54 GB. None
  implemented goes below 1 KB; the only result of the order of λ bits is theoretical,
  designated-verifier and under a non-standard assumption (evasive LWE). Ishai, Su and Wu
  quantified it in 2021: «a 1000× gap in the proof size between the best pre-quantum
  constructions and the best post-quantum ones».

⚠️ The bound `≥ 2^(256/k)` for `k` elements holds for verifiers that walk hash chains or hash
trees (Winternitz, Lamport, the graph-based ones); it is not a theorem for every conceivable
verification function. It is declared as what it is: the arithmetic of the known constructions.

To reproduce:

```python
n, ell = 32, 256
def fila(lw):
    w = 1 << lw
    len1 = -(-ell // lw)                       # ceil(ell / log2 w)
    x, len2 = len1 * (w - 1), 0
    while x: x //= w; len2 += 1                # floor(log_w(len1 * (w - 1))) + 1
    ln = len1 + len2
    return lw, len1, len2, ln, ln * n, ln * (w - 1)
for lw in (4, 8, 16, 32, 64): print(fila(lw))
```

---

## 3. Where signatures of that size exist today 📖

NIST level 1 figures unless otherwise stated, in bytes, at 30-09-2026. The «source» column says
where each figure was read from; «mirror» as per the convention above.

| scheme · parameter set | NIST category | signature | public key | assumption | status | source |
|---|---|---|---|---|---|---|
| ML-DSA-44 | **2** | 2,420 | 1,312 | lattices | FIPS 204, 13-08-2024 | FIPS 204, Table 2 (mirror) |
| ML-DSA-65 | 3 | 3,309 | 1,952 | lattices | FIPS 204 | idem |
| SLH-DSA-SHA2-128s | 1 | **7,856** | 32 | hash only, stateless | FIPS 205, 13-08-2024 | FIPS 205, Table 2 (mirror) |
| SLH-DSA-SHA2-128f | 1 | 17,088 | 32 | hash only, stateless | FIPS 205 | idem |
| SLH-DSA-SHA2-192s / 256s | 3 / 5 | 16,224 / 29,792 | 48 / 64 | hash only, stateless | FIPS 205 | idem |
| Falcon-512 (FN-DSA-512) | 1 (according to implementers) | 666 (padded; 752 max. without padding) | 897 | NTRU lattices | FIPS 206 **unpublished, not even a draft**, at 30-09-2026 (minutes of the Open Quantum Safe TSC of 01-09-2026, with Pornin) | ⚠️ **no primary source**: PQClean's `api.h` and the README of `rust-fn-dsa`; the Falcon specification was not reachable |
| XMSS-SHA2_10_256 | n = 32 | 2,500 | 68 📐 | hash only, **stateful** | RFC 8391, SP 800-208 | RFC 8391, Table 3 (mirror); the key, `OID ‖ root ‖ SEED`, derived |
| XMSSMT-SHA2_40/8_256 (the house's) | n = 32 | **18,469** | 68 📐 | hash only, **stateful** | RFC 8391, SP 800-208 | RFC 8391, Table 5 (mirror); ✅ measured in §1 |
| LMS_SHA256_M32_H10 / W8 | n = 32 | 1,452 📐 | 56 📐 | hash only, **stateful** | RFC 8554, SP 800-208 | derived from RFC 8554 (§4.1, `4 + n·(p+1)` with p = 34, plus `4 + n·h`); the only literal figure of the RFC is Table 3 of HSS: 1,616 B with h = 15 |
| **uov-Is** (n=160, m=64, GF(16)) | 1 | **96** | 66,576 (412,160 classic) | multivariate | round 3, parameters unchanged | README of `pqov`, round-3 branch |
| uov-Ip (round 3: n=119, m=45) | 1 | 135 | 46,591 (321,300 classic) | multivariate | round 3; in round 2 it was 128 B and the wedge/intersection attacks brought it down to ~128 bits | idem |
| MAYO_2 (round 3) | 1 | 239 | 2,928 | multivariate | round 3; in round 2 it was 186 B | README of `MAYO-C` |
| MAYO_1 (round 3) | 1 | 464 | 1,456 | multivariate | round 3 | idem |
| SNOVA_I_S (round 3) | 1 | 272 | 1,016 | multivariate | round 3; the round-2 parameter sets were replaced | package `SNOVA_Round3` |
| QR-UOV I, L=3 (round 3) | 1 | 200 | 24,256 | multivariate | round 3 | macros of `qruov/round3` |
| SQIsign_p324_3 (round 3, level I) | 1 | **200** | 83 | isogenies | round 3; tag `nist-v3` of 01-09-2026, new primes after Wesolowski's p^(1/3) attack (ePrint 2026/1486) | `api.h` and KAT of the official repository `SQIsign/the-sqisign` |
| SQIsign_lvl1 (round 2) | 1 | 148 | 65 | isogenies | replaced by round 3 | idem, tag `nist-v2` |
| SQIsign, compact format of dimension 4 | 1 | 142 | 84 | isogenies | third-party proposal (ePrint 2026/2221), not the specification's format; verifying costs about ten times more | README of `sqisign-rs` |
| HAWK-512 | 1 | 555 | 1,024 | lattices (lattice isomorphism) | round 3; **withdrawn by its team on 29-07-2026** after a key recovery (ePrint 2026/1593, Straznickas and Weis); round 3 is left without lattices | `hawk.h` of the repository `hawk-sign/dev`; the withdrawal, via the pqc-forum (not read at origin) |
| Rainbow-I (round 3, 2020) | 1 | **66** | 161,600 (60,192 compressed) | multivariate | **broken**: Beullens, ePrint 2022/214, «53 hours (one weekend) of computation time on a standard laptop» for the round-2 SL 1 parameters; NIST did not select it (IR 8413, not read) | sizes from compiling the round-3 reference code, reproduced twice; the attack's abstract, in a third-party copy |
| GeMSS128 (round 3, 2020) | 1 | **33** | 352,188 | multivariate (HFEv-) | **broken**: Tao, Petzoldt and Ding, ePrint 2020/1424 (Crypto 2021); not continued by NIST | sizes from the SUPERCOP headers (not from the official package); the paper, in a third-party conversion |
| VDOO (2023; NGCC v1.1, 2026) | 1 | 96 / **85** | no data / 330,855 | multivariate | academic proposal, today a candidate of the Chinese NGCC competition; **cryptanalysed in September 2026** (ePrint 2026/2178, «all proposed parameters fail to meet their claimed NIST security levels», by abstract; and two «Confirmed» findings on ngcc.dev) | abstract of arXiv 2312.09535 (mirror) and specification v1.1 in the NGCC harness |

Three readings, in order of importance:

**Census of what goes below 100 bytes**, with everything that was found, so that it does not have
to be redone: `uov-Is` 96 B (in force, round 3); VDOO 96 and 85 B (cryptanalysed this month);
TUOV-Is 80 B (NIST round 1, did not advance, 65,552 B key); KAZ-SIGN 74 B (only in
`nist-sigs-zoo`, no primary source), Facto-DSA 40 B and DME-Sign 32 B (broken by forgery);
Rainbow-I 66 B and GeMSS128 33 B (broken by key recovery). **One alive out of nine**, and with a
66 KB public key. The KpqC proposals and HPPC/3WISE were not reviewed in depth.

1. **The real bar is not ML-DSA-44's 2,420 bytes.** Falcon-512 has been at 666 B for years and
   ML-DSA-44 is not even category 1, it is 2. Comparing against Dilithium inflates the merit of
   anything that is proposed.
2. **The only thing alive below 100 bytes is `uov-Is`, at 96 B, with a 66 KB public key.** It is
   multivariate: the family of Rainbow and GeMSS, the two schemes that went below 100 B inside the
   NIST competition and were broken. UOV has been standing since 1999, but in September 2026 three
   of the four multivariate schemes of round 3 changed parameters because of new attacks
   (`uov-Ip`, MAYO, SNOVA), and cryptanalysis dated this very month keeps coming out. Adopting it
   would be revoking the decision of a single family of assumptions (`AUDITORIA.md` §106), and we
   would invent nothing: we would use UOV.
3. ⚠️ **A premise of entry 87 of `BACKLOG.md` was wrong.** Its table said that SLH-DSA signs
   «tens of KB». FIPS 205 says 7,856 B for SHA2-128s and 16,224 B for 192s: **both below the
   18,469 B of the XMSS^MT the house signs with today**, stateless and with the same assumption
   (hash only). Only the category-5 parameter set, 29,792 B, is larger. It decides nothing on its
   own —signing with SLH-DSA-s is slow, and **it has not been measured here**—, but it changes the
   price entry 87 took for granted: stateless no longer costs «tens of KB», it costs less than
   today.

⚠️ **2025-2026 cryptanalysis cited by abstract, not read at source** (eprint blocked from this
session): ePrint 2025/1143 (Ran, wedge on UOV), 2026/298 (Furue-Ikematsu, intersection), 2026/237
(Bros et al., SNOVA), 2026/2154 (Beullens-Hess, SNOVA round 3), 2026/2181 (Merz-Ran,
Schur-Macaulay over the whole of round 2 of UOV, MAYO, SNOVA and QR-UOV), 2026/2247 (Ostuzzi, MAYO
round 3, published on 30-09-2026, which according to its abstract leaves MAYO_2 three bits below
category 1). Of Wesolowski, ePrint 2026/1486 (the supersingular isogeny problem in time and memory
p^(1/3+o(1))), a literal third-party copy was read: it says it does **not** break SQIsign, and the
team changed the primes on 01-09-2026. They are listed so that whoever checks knows where to look,
not as facts verified here.

---

## 4. What Merkle trees buy, and why it is not a signature 📖 ✅

The IETF's Merkle Tree Certificates (`draft-ietf-plants-merkle-tree-certs`, version 06 of
21-09-2026, PLANTS group, Standards Track; read in the copy published by the group itself on
GitHub) replace the certificate's signature with an **inclusion proof** in an issuance log, and
move the signature elsewhere: the CA signs *checkpoints* and subtrees, and independent *cosigners*
co-sign them. Its figures (§6.5 of the draft, SHA-256):

| certificate | batch | proof | and besides |
|---|---|---|---|
| standalone, checkpoint every 2 s | ≈ 3,000 | 12 hashes, **384 B** | the co-signatures of the CA and of the quorum, «a sufficient set of signatures», **not counted** |
| landmark-relative, landmark every hour | ≈ 5,400,000 | 23 hashes, **736 B** | no signature; it demands that the verifier **already has the subtree** |
| bound of the draft | up to 2³² | 32 hashes, 1,024 B | |

And the head does not travel with the certificate: «landmark subtrees are predistributed to
relying parties … separate from the application protocol» (§3); how, «This document does not
prescribe» (§7.4: a browser update service). The verifier keeps of the order of 338 subtrees,
some 10,816 B. The certificate without signatures «only work[s] in a sufficiently up-to-date
relying party», and the draft orders the other one to be deployed beside it (§6.4).

It is exactly the house's construction: **signed head + path + co-signatures**. Measured in §1,
the marginal cost per element is `32 · h` bytes:

| height `h` | elements per head | path | who pays for it today in the house |
|---|---|---|---|
| 32 | 2³² | 1,024 B | the acknowledgement and the receipt (`RECEP_DEPTH`) |
| 63 | 2⁶³ | 2,016 B | the consumption tree |
| 3 | 8 | 96 B | nobody: it would be a batch of eight |

To go below 100 bytes, **h ≤ 3** is needed and the verifier must **already have the head** (the
witness has it: it pins it the first time it sees it, `AUDITORIA.md` §245). And then what it holds
in its hand is not a signature: it is **an inclusion receipt under a head that someone else signed
with 18,469 bytes**, amortised among the eight. MTCs do not shorten a signature: they **amortise**
it, and they reintroduce the tree the question wanted to avoid.

---

## 5. Verdict

| condition | with the hash family | with another family |
|---|---|---|
| < 100 bytes | ✘ arithmetic of §2: ≥ 2⁸⁵ hashes per chain | ✔ only `uov-Is`, 96 B, 66 KB key; SQIsign stays at 200 |
| without losing security | ✘ only by lowering the digest to 64 post-quantum bits | ⚠️ a family with two breaks and parameters changed this month |
| without a state tree | ✔ SLH-DSA, at 7,856 B; STARK, at 43 KB | ✔ |
| all three at once | **✘** | **✘ without revoking §106** |

**It is not reachable with the house's tools nor with its assumptions**, and this document exists
so that the question is not asked again without this number in front.

---

## 6. What is publishable, by name

1. **This document.** A negative result with its reproducible arithmetic and its checked figures
   satisfies «measure before claiming» (`CONTRIBUTING.md` §1) just as a positive one does.
2. **Short receipts under a signed head, as a measurement.** The table of §4 with the house's
   `32 · h` and the amortised signature. It is published as what it is: the amortised cost of an
   XMSS with the head as anchor and the witness as a verifier that already has it. Never as a
   «new signature».
3. **The correction to entry 87** (§3, reading 3): SLH-DSA-SHA2-128s weighs 7,856 B, not «tens of
   KB». It is a premise of the pending decision on cryptographic agility, and it has to be
   re-measured before deciding: the signing time of SLH-DSA-s on the real hardware is the missing
   figure, with the method of §263.

And two paths that are **not** this project's, declared so that they do not come back in
disguise:

- **Deferred disclosure at the pace of the heartbeat** (a 32 B MAC whose key is published in the
  next head). Hash only, no tree, under 100 bytes; but it is another security model: it demands
  synchrony and gives no non-repudiation on its own. Publishing it as a signature would be the
  categorical claim that `SECURITY.md` §5 forbids.
- **Changing family** (UOV, SQIsign). It requires revoking §106 and taking on public keys of tens
  of KB or assumptions with open cryptanalysis.

---

## 7. What this document does NOT claim

- It does not claim that a post-quantum signature of fewer than 100 bytes is impossible in
  general. It claims that it is not possible with the house's family of assumptions and with the
  known constructions, with the figures read at 30-09-2026.
- It measures the time of no third-party scheme. The figures of §3 are sizes read, not measured.
- It does not propose changing §106 nor the XMSS parameter set chosen in `doc/xmss-evaluacion.md`.
- It has not checked the «mirror» copies against the NIST or IETF original: two identical mirrors
  reduce the risk, they do not remove it.

---

## 8. Pending items it leaves

1. Check each «mirror» against the original when the network allows it (SHA-256 of the PDFs of
   FIPS 204 and 205; text of RFC 8391 and 8554 and their errata; NIST IR 8610 for the round-3
   list; the Falcon specification, which today has no primary source in this table).
2. Entry 87: re-measure the price of «stateless» with SLH-DSA-SHA2-128s and 192s on the real
   hardware, signing time included, before the decision.
3. If someone wants the batch receipt (§6, 2): it is a bench, not a design; it starts by measuring
   how much the whole envelope weighs with `h = 3` against the one with `h = 32`.

## Sources

- FIPS 204 (ML-DSA) and FIPS 205 (SLH-DSA), 13-08-2024:
  `https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.204.pdf`,
  `https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.205.pdf` (mirror: identical copies in
  `github.com/Cosmian/standards` and `github.com/hzhang092/nist-pqc-rag-agent`).
- RFC 8391 (XMSS) and RFC 8554 (LMS): `https://www.rfc-editor.org/rfc/rfc8391.txt`,
  `https://www.rfc-editor.org/rfc/rfc8554.txt` (mirror: `mnot/rfc-refs` and
  `mcpherrinm/rfc-mirror`, identical md5). NIST SP 800-208 (October 2020), read in a third-party
  text conversion.
- Falcon / FN-DSA: `https://github.com/PQClean/PQClean`
  (`crypto_sign/falcon-padded-512/clean/api.h`) and `https://github.com/pornin/rust-fn-dsa` (note
  of 22-07-2026 on FIPS 206).
- UOV: `https://github.com/pqov/pqov` (README, round 3). MAYO: `https://github.com/PQCMayo/MAYO-C`
  (README, round 3 and round-2 tag). SNOVA: `https://github.com/PQCLAB-SNOVA/NIST_Submissions`
  (`SNOVA_Round3.zip`). QR-UOV: `https://github.com/qruov/round3`.
- SQIsign: `https://github.com/SQIsign/the-sqisign` (tags `nist-v2` and `nist-v3`,
  `src/nistapi/*/api.h` and `KAT/`); `https://github.com/anchorageoss/sqisign-rs` (compact format,
  third party).
- Merkle Tree Certificates:
  `https://datatracker.ietf.org/doc/draft-ietf-plants-merkle-tree-certs/` (mirror:
  `https://ietf-plants-wg.github.io/merkle-tree-certs/`, tag 06 and the editor's copy of
  30-09-2026).
- Rainbow and GeMSS: round-3 reference code and SUPERCOP headers (compiled sizes); Beullens,
  ePrint 2022/214, and his repository `https://github.com/WardBeullens/BreakingRainbow`; Tao,
  Petzoldt and Ding, ePrint 2020/1424 (third-party conversion).
- HAWK: `https://github.com/hawk-sign/dev` (`src/hawk.h`); withdrawal announced on NIST's
  pqc-forum on 29-07-2026, cited by `https://github.com/PQShield/nist-sigs-zoo` (`history.yaml`).
- VDOO: arXiv 2312.09535 (abstract, mirror) and `sign-33-spec.pdf` in
  `https://github.com/ngcc-dev/ngcc-harness`; ePrint 2026/2178 by abstract. TUOV, KAZ-SIGN,
  DME-Sign and Facto-DSA: `nist-sigs-zoo` and the reports of `ngcc.dev`, not their
  specifications.
- Succinct proofs: Groth16 in `https://github.com/zcash/zips` (`protocol/protocol.tex`); LaBRADOR
  ePrint 2022/1341, Greyhound 2024/1293, WHIR 2024/1586, Aurora 2018/828, LUNA+ 2026/1639, ISW21
  2021/977 and Mathialagan, Peters and Vaikuntanathan 2024/227, read in the conversions of
  `https://github.com/BaDaaS/cryptography.academy` (mirror) or, where «abstract» is said, only in
  their abstract.
