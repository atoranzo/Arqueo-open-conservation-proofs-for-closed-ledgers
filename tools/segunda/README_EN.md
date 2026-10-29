# `tools/segunda` — the second implementation: the core, the heads, the second verifier, the STARK and the wire

> **Español:** [`tools/segunda/README.md`](./README.md) is the original of this page, in Spanish; it
> prevails wherever the two differ. This translation is of commit `7766efb` (audit entry §917) and
> was produced by the project's assistant, a Claude Code session, in audit entry §918 (`GENAI.md`):
> it is a generated artefact, not text written by the author. Line citations of the form `file:N`
> refer to the Spanish files.

The spikes of [`doc/integracion-vertical-evaluacion.md`](../../doc/integracion-vertical-evaluacion_EN.md),
sections 5.2 and 5.3, measured on 01-10-2026 on `e8ac246` (S615).

| file | what it is |
|---|---|
| `nucleo.py` | the **second implementation of the CORE** (`spec/NUCLEO.md`, section 6) in Python, without dependencies: the Goldilocks field, the Rescue-Prime permutation `Rp64_256`, BLAKE3, the serialization, the domains, the preambles and the compositions, in the exact order of the merges; since §800, also the two v7 preambles and the scheme registry of RFC-0020, with its rules, which the judge checks with a falsifier |
| `juez_nucleo.py` | the **judge**: it runs the 35 KATs of `spec/vectors/nucleo/` against `nucleo.py` (⚠️ §914: 34 until the cut) and says `OK` or `ROJO` per vector, plus the `NOTA` lines on what section 6 leaves open. ⚠️ §797: until here it said «it is the only caller of `nucleo.py`», and it no longer was: `juez_cabezas.py:23`, `verificador.py:31` and `stark.py:30` import it too, and since §797 `cable.py` and `juez_cadena.py`. ⚠️ §852: it said `verificador.py:25`, stale since §810, which took the `import` to `:30`; the `import re` of §852 takes it to `:31` |
| `kat_xmss/xmss.py` | the **XMSS^MT verification of RFC 8391** for `XMSSMT-SHA2_40/8_256`, the parameter set the house signs with, written from the text of the RFC and from no crate; verification only |
| `juez_cabezas.py` | the **judge of the signed heads**: for every head and co-signature of the vectors it recomposes the `epochDigest` by version, compares the recovered preamble, verifies the signature with `kat_xmss/xmss.py` and checks the embedded index (`PAQUETE.md`, 2/3 and section 8); with a falsifier that flips a byte |
| `verificador.py` | the **second verifier**: the five forms of the package that do not require a STARK proof —position v1 and v2 with acknowledgement («acuse») and co-signatures, extension, consumption, conflict and anchor—, and since §649 the `actas` that join two keys of one operator in the extension and the consumption (RFC-0015: the reader, the rules of the chain and their XMSS signatures), and since §686 the overlap envelope —since §722, with the handover of the common history: `juzgar_historia` in the extension and the consumption, every head in the handover of its stretch by its two edges, and the same judgement in the overlap—, written from `PAQUETE.md` with the command's contract (one argument, `ROJO: {motivo}` from the catalogue, exit 0/1/2), and judged by `tools/conformidad.sh` with the same manifests as the binary; since §693 it declares what it does not read, in `NO_SOPORTADOS`, and says so with the reserved text `NO SOPORTADO`; and since §810 it reads the **co-signed anchor** envelope (RFC-0013 E4a), which requires not a STARK but the medium: the head, the note with its ML-DSA-44 signature, the co-signatures and the inclusion, with `mldsa44.py` and `medio.py` |
| `mldsa44.py` | the **ML-DSA-44 verification of FIPS 204** (§810), in Python and without dependencies —`hashlib` brings SHAKE128 and SHAKE256—, written from the text of the standard —from the `ml-dsa` 0.1.1 crate it was read, to compare and not to copy, how it builds μ and where it rejects the norm and the hints—: algorithm 3 with the empty context and the pure signature, which is how the house calls `ml-dsa` 0.1.1 (`nota.rs`, `verify_with_context(mensaje, &[], firma)`), over algorithm 8, with pkDecode, sigDecode, HintBitUnpack, SampleInBall, ExpandA, the NTT and UseHint; verification only |
| `medio.py` | the **medium of the anchor** (§810): strict base64, the RFC 9162 inclusion in the SHA-256 tree of anchors and the `checkpoint` note with its `0x06` signature and its co-signatures, with the rules the header of `crates/zk-ssl-medio/src/nota.rs` declares —which is what `PAQUETE.md` 2.13 orders to read— and the texts of its `Display`, so that the cause of a rejection is the binary's |
| `juez_medio.py` | the **judge of the medium** (§810): the 33 notes of §632 with the verdict of their `MANIFIESTO.txt`, a **test signer** of FIPS 204 —KeyGen and Sign, only here— that redoes byte for byte the vkeys and the deterministic lines Go signed, the two boundary signatures of the norm, a hint where the low part is zero with a tailored key, the uniqueness of a signature's encoding, the base64, the 98 inclusion probes of transparency-dev and Rust's `{:?}` with which the names come out in a rejection; and since §859, five rules of `mldsa44.py` and `medio.py` that nothing isolated, each with its case and its control |
| `cable.py` | the **wire in Python**, two pieces in one file, and ⚠️ **neither is core** (§797). ⚠️ §809: the two were written with the same file name —the chain in §797, the consumer in §809— and were joined into one without sharing any name; `juez_cadena.py` imports part I and `main` belongs to II. I · the **chain of the log** as the wire serves it, `chain_v1` and `chain_v2`, and what each class records as `proofDigest`, written from `spec/RPC.md` §«La composición de `chain`, byte a byte» [the composition of `chain`, byte for byte]; it is the REGISTRO class [log] of `NUCLEO.md` (RFC-0005, D-A), and that is why it does not live in `nucleo.py`, from which it only takes the primitives. II · the **second consumer of the wire** (§809): it judges a `zkssl_signedEpochHead` response from a file and without a node, like `zk-ssl-cli witness --respuesta` with fresh memory —the JSON-RPC body, with the limits of the reference's reader, `available`, the wire's typing, what a signed head has to bring, the version, the key, the signature, the preamble, the `index` and the recomposition—, with the command's contract (one argument, `VERDE: nueva` or a line `ROJO: <clase> · <texto>`, exit 0/1/2), and `tools/conformidad.sh` judges it with the wire's manifest, in place of the adapter `tools/cable_respuesta.sh`; the head is split and recomposed with `juez_cabezas.py` and the signature is verified with `kat_xmss/xmss.py`. What `spec/RPC.md` does not write is pinned by the measured behaviour of the witness, as a black box (below, «The second consumer of the wire») |
| `stark.py` | the **STARK verifier** (§626): the machinery of `winter-verifier` 0.13.1 in Python, without dependencies —the `Proof` format, the public coin over Blake3 in the exact Fiat-Shamir order, batched Merkle authentication with the salt of `MerkleConSal`, the out-of-domain check, the DEEP composition, FRI and the `Oculta` wrapper—, read from winterfell's sources and not copied; `INV_MDS` is computed by inverting the MDS and the self-test checks it against the table of `winter-crypto` |
| `airs.py` | the **five AIRs** of the package —Banda, Edad, Prenda, CobroPendiente and PagoEnCurso—, **transcribed** from `crates/zk-ssl-air/src/*.rs` constraint by constraint: no RFC writes their constraints and the `.rs` is the only source, so an under-constrained AIR would pass here just as there. Since §723, Banda and CobroPendiente in their two versions, and the `verificar` that chooses between them |
| `juez_stark.py` | the **judge of the STARK**: it composes, for every vector with a proof, the pair the command judges —the proof and the statement, with the fields and the head `zk-ssl-verify` reads— and compares it with its `MANIFIESTO.txt`; three falsifiers per positive |
| `juez_cadena.py` | the **judge of the chain** (§797): it recomposes from its entries every `chain` of the wire's vectors —in the form of the conformance vector or in that of the `LogEntry[]` list `zkssl_logEntries` serves—, says `OK`, `DECLARADO` or `ROJO <fichero> seq <seq> (<kind>)` per entry, runs its falsifiers, says in its `NOTA` and `SIN EJERCITAR` lines what it does not see and closes with `cadena: N de N cadenas recompuestas` |
| `wasm_runner.py` | the **wrapper** to run `zk-ssl-verify` compiled to `wasm32-wasip1` inside wasmtime with the command's contract (`PAQUETE.md`, section 6), so that `tools/conformidad.sh` judges it like the native binary |
| `kat_xmss/` | the **KAT corpus of XMSS^MT**: keys and signatures produced by the `xmss 0.1.0-pre.0` crate from fixed seeds and verified by `kat_xmss/xmss.py`, which lives inside so that the directory is a repository by itself; with its judge and its generator. It is the module with the profile of `hbs-state`: extractable as is, without a byte of the protocol (`kat_xmss/README.md`) |

## How it is run

```bash
# 1 · la segunda implementación contra los KAT del núcleo        -> nucleo: 34 de 34
python3 tools/segunda/juez_nucleo.py

# 1 bis · las cabezas firmadas de todos los vectores, con XMSS^MT desde RFC 8391  -> ~31 s (§809)
python3 tools/segunda/juez_cabezas.py

# 1 ter · el corpus KAT de XMSS^MT: dos implementaciones, los mismos bytes       -> 8 de 8
python3 tools/segunda/kat_xmss/juez_xmss.py

# 1 quater · el segundo verificador, con el arnes y los manifiestos del binario   -> 250 de 250 (§722)
#   ⚠️ §810: decia 174 de 174 (§780), las cinco familias sin `ancla-cofirmada`, que salia NO SOPORTADO
#   ⚠️ §852: decia 201 de 201 (§810); los cinco negativos `rechazo-json-*` de `paquete` la llevan a 206
#   ⚠️ §712: decia 206 de 206 (§852); los nueve vectores del reloj, cinco de `paquete`, tres de `consumo` y uno de `ancla`, la llevan a 215
#   ⚠️ §714: decia 215 de 215 (§712); los veintiun vectores de la bifurcacion en `ancla` la llevan a 236
#   ⚠️ §718: decia 236 de 236 (§714); el acta con su `mmrSize` escrito como `N + p`, en `rotacion`, la lleva a 237
#   ⚠️ §722: decia 237 de 237 (§718); los trece vectores de la historia comun, en `rotacion`, la llevan a 250
for m in paquete consumo conflicto ancla rotacion ancla-cofirmada; do bash tools/conformidad.sh tools/segunda/verificador.py spec/vectors/$m/MANIFIESTO.txt | tail -1; done

# 1 quinquies · el verificador STARK: las pruebas ocultas y con sal de seis familias      -> 76 de 76 (§723)
#   ⚠️ §711: decia 34 de 34 (§706); los cuatro negativos de la prueba mal formada, dos de la edad y dos del pago, la llevan a 38
#   ⚠️ §713: decia 38 de 38 (§711); el negativo capturado del resto FRI, `edad/neg-prueba-resto-ajeno`, la lleva a 39
#   ⚠️ §715: decia 39 de 39 (§713); los veintiseis negativos de una sola escritura, quince de la edad y once del pago, la llevan a 65
#   ⚠️ §717: decia 65 de 65 (§715); los tres negativos del tope de `m` en `edad/` la llevan a 68
#   ⚠️ §718: decia 68 de 68 (§717); el emisor fuera de las cuentas, en `edad/`, la lleva a 69; los otros cuatro negativos con prueba del §718 caen antes del juez
#   ⚠️ §721: decia 69 de 69 (§718); el negativo que fija el tope de `m` en 13, `edad/neg-m-14-sobre-tope`, la lleva a 70
#   ⚠️ §723: decia 70 de 70 (§721); las seis pruebas de la v2 y de la ventana, cuatro de `rechazo` y dos de `pendiente`, la llevan a 76
python3 tools/segunda/stark.py          # autotest: la extension, INV_MDS, las raices, la marca, el orden de Rust, el resto atado, una sola escritura
python3 tools/segunda/juez_stark.py

# 1 sexies · la cadena del registro del cable (NO es núcleo: spec/RPC.md)       -> cadena: 6 de 6
#   ⚠️ §716: el glob decia «los cuatro: 24 de 24»; `zkssl-0.5.json`, el escenario del §716, lo lleva a cinco y 30 de 30
python3 tools/segunda/juez_cadena.py                          # spec/vectors/zkssl-0.4.json
python3 tools/segunda/juez_cadena.py spec/vectors/zkssl-0.*.json   # los cinco: 30 de 30
python3 tools/segunda/juez_cadena.py respuesta.json   # o el `result` de zkssl_logEntries, desde seq 0
# 1 septies · el segundo consumidor del cable, con el arnes y el manifiesto del testigo     -> 27 de 27 (§850)
#   ⚠️ §850: decia 23 de 23 (§809), de antes de los tres negativos de la escritura y de `no-existe.json`
bash tools/conformidad.sh tools/segunda/cable.py spec/vectors/cable/MANIFIESTO.txt | tail -1
# 1 octies · el medio: ML-DSA-44 de FIPS 204, la nota checkpoint y la inclusion de RFC 9162   -> 92 de 92 (§859), ~2 s
#   ⚠️ §859: decia 87 de 87 (§810); los cinco casos del bloque 7 lo llevan a 92
python3 tools/segunda/juez_medio.py

# 2 · el verificador a WebAssembly, y los diez manifiestos bajo wasmtime
rustup target add wasm32-wasip1
cargo build -p zk-ssl-verify --release --target wasm32-wasip1 --locked
python3 -m pip install wasmtime                                   # el motor wasmtime, con bindings
python3 tools/segunda/wasm_runner.py --precompilar target/wasm32-wasip1/release/zk-ssl-verify.wasm /tmp/zk.cwasm
printf '#!/usr/bin/env bash\nexec python3 %s/tools/segunda/wasm_runner.py /tmp/zk.cwasm "$@"\n' "$PWD" > /tmp/zk-wasm
chmod +x /tmp/zk-wasm
for m in ancla completitud conflicto consumo edad pago paquete pendiente prenda rechazo; do
  bash tools/conformidad.sh /tmp/zk-wasm spec/vectors/$m/MANIFIESTO.txt | tail -1
done
```

## What was measured

**The second implementation.** 35 of 35 vectors reproduced byte for byte (26 until §643, which
adds the three of the act, 29 until §773, which adds `digest_pi` and the v2 leaf of the receipt,
31 until §800, which adds the scheme registry and the two v7 preambles of RFC-0020, and 34 until
§914, which adds the registry of the families of the v2 mark), written from section 6 of
`NUCLEO.md` and from no `.rs` of the house.
The permutation comes from the source of `winter-crypto 0.13.1`, which is the primitive and not the
house, and is validated in the self-test against the vector that crate attributes to the Sage
implementation; BLAKE3 is written from the public specification and validated against 19 outputs of
the `blake3 1.8.5` crate of the `Cargo.lock`.

**The signed heads.** `juez_cabezas.py` walks the 406 files of `spec/vectors/` that carry heads,
including the `0.3/` catalogues and the wire's:

| measure | value |
|---|---|
| signed heads found | 537 |
| that recompose their `epochDigest`, recover their preamble, verify their signature and bind their index | 485 |
| witness co-signatures found / that verify | 15 / 10 |
| files with some head or co-signature that does NOT verify | 57, **the 57 negatives** according to their `MANIFIESTO.txt` |
| positives with some head that does not verify | 0 |
| falsifier: one byte flipped in the signature and another in the preamble | both stop verifying |
| one XMSS^MT verification in Python | ~40 ms |

The 52 head failures are those the negatives fabricate: 29 heads whose fields do not recompose the
digest, 4 signatures that do not verify, 3 declared indices that do not stay above the embedded one
(one below and two equal), 2 without `index`, 5 fields that are not canonical (a `u64` that is not
less than `p`, RFC-0016), 8 versions outside the set or without their fields and one key whose OID is
not XMSS^MT's; and the 5 co-signature ones, 2 signatures that do not verify, 2 preambles that are not
the expected one and a `versionFormato` that does not fit in its byte (§780). The signature travels
as RFC 8391 describes it, 18,469 bytes —`idx` (5) ‖ `r` (32) ‖ 8 × (WOTS+ 67×32 ‖ auth 5×32)—,
followed by the preamble; the key is `OID` (4, `0x00000005`) ‖ `root` ‖ `SEED`, 68 bytes, and a
library that reads the OID as the RFC mandates does not need the workaround of §240.

⚠️ §793: here it said 331 files, 418 heads, 376 that verify, 11 / 7 co-signatures and 46 files, and
«the 42 failures» —26 that do not recompose, 6 signatures, 2 preambles, 2 indices «below the
embedded one», 2 without `index` and the versions—: the figures of S623. The entries recounted them
as the vectors grew (§640, 378 of 424 heads; §706, 472 of 522), and this page did not follow them.
The current ones are the output of `python3 tools/segunda/juez_cabezas.py` on the tree of §789
(`8c02941`); the ~40 ms is from S623 and has not been measured again. The `versionFormato` `0x103`
co-signature that §780 brought counts among the negatives: `juzgar_cofirma` rejects it with the
kit's text, like `verificador.py`.

⚠️ §809: here it said 404 files, 535 heads, 55 files with «the 55 negatives» and «the 50 failures»,
with 2 indices («one below and one equal») and 4 non-canonical fields: the measurement of §793, from
before the two wire negatives §809 adds, `cable/rechazo-index-atrasado.json` (the `index` equal to
the embedded one) and `cable/rechazo-n-no-canonico.json` (`n + p`). The current ones are the output
of `python3 tools/segunda/juez_cabezas.py` on the tree of §809: 485 of 537 heads, 10 of 15
co-signatures, 406 files, 57 with some that does not verify and 0 positives, in 31 to 33 s with the
machine loaded (31.9 s of wall clock and 30.1 of CPU in the last run). And the «1 bis» line of «How
it is run» said «~25 s».

⚠️ §815: kit 0.4.3 brings three completeness vectors, each with its closing, and the output of
`python3 tools/segunda/juez_cabezas.py` on the tree of §815 is 488 of 540 heads, 10 of 15
co-signatures, 409 files, 57 with some that does not verify and 0 positives: three more heads, and
the three verify. The figures above are those of §809.

⚠️ §850: the three wire negatives §850 brings (`cable/rechazo-index-no-minimo.json`,
`cable/rechazo-quantity-con-ceros.json` and `cable/rechazo-hex-mayusculas.json`) the judge gives
`OK`. It reads the QUANTITY with `int(s, 16)` and the hex with `bytes.fromhex`, which admit a leading
zero and upper case; and what it judges —the recomposition, the preamble, the signature and the
index— does not change with the encoding. The output on the tree of §850 is 491 of 543 heads, 10 of
15 co-signatures, 412 files, 57 with some that does not verify and 0 positives, in 29 s, with exit
0: the exit only looks at whether any positive fails, and a file with code 1 in its manifest and all
its heads `OK` does not count it. This seal does not touch `juez_cabezas.py`: the wire's encoding is
judged by `cable.py`, below.

⚠️ §862: **dispatch by scheme** (E3 of RFC-0020). `nucleo.py` gains the two sets and the kit's
implicit one —`ESQUEMAS_ACEPTADOS`, `ESQUEMAS_EMITIDOS` and `ESQUEMA_IMPLICITO`— and
`entrada_aceptada`, which gives the registry entry of an accepted scheme, or the rejection with its
number and the kit's text; `juez_nucleo.py` runs the binding rule on them —the implicit one is
emitted, what is emitted is accepted and what is accepted is in the registry—, with the registry's
KAT and a falsifier, an empty accepted set. `juez_cabezas.py` no longer takes the length of the
signature from the key's OID: every head, every co-signature and the falsifier choose the entry of
the scheme of their signature —the implicit one, in v1 to v6— and then demand a key of its encoding,
the length and the prefix the entry declares; and `cable.py` goes through the same dispatch before
reading the key, with its usual texts, although it still anchors the key by its OID and measures the
signature with it, without checking it against the entry. The figures do not change: 630 of 685
heads, 15 of 20 co-signatures, 512 files, 60 with some that does not verify and 0 positives, with
the dispatch and without it, and the wire, 27 of 27; only the reason of one negative changes,
`conflicto/rechazo-conf-oid-alterno.json`, whose key opens with `00010005` and is now not of the
scheme's encoding. With the empty accepted set, the judge gives 0 of 685 and 73 broken positives,
the wire 17 of 27 and `juez_nucleo.py` ROJO in `registro_de_esquemas`: the three go through it.
`verificador.py` does not go through the dispatch and still chooses by the OID: §862 decided it, and
it can be revisited.

**The second verifier.** `verificador.py` passes, with the same harness and the same manifests as
the reference binary, the six families that do not require STARK (⚠️ §810: it said «the five»;
the sixth, `ancla-cofirmada`, is from §810):

| family | entries | second verifier |
|---|---|---|
| `paquete` (position v1 and v2, extension; ⚠️ §852: it said 84 and 84 of 84, from before the five `rechazo-json-*`; ⚠️ §712: it said 89 and 89 of 89, from before the five of the clock) | 94 | 94 of 94 |
| `consumo` (since §771, also between two keys with `actas`; ⚠️ §712: it said 17 and 17 of 17, from before the three of the clock) | 20 | 20 of 20 |
| `conflicto` | 17 | 17 of 17 |
| `ancla` (⚠️ §712: it said 22 and 22 of 22, from before `neg-ancla-bifurcada-mismo-mmrSize`; ⚠️ §714: it said 23 and 23 of 23, from before the twenty-one of the fork) | 44 | 44 of 44 |
| `rotacion` (the extension between two keys, with `actas`; §649; and the overlap envelope, §686; ⚠️ §718: it said 34 and 34 of 34, from before `neg-acta-mmrsize-alias`; ⚠️ §722: it said 35 and 35 of 35, from before the thirteen of the common history, with the handover) | 48 | 48 of 48 |
| `ancla-cofirmada` (the note of the medium with its ML-DSA-44 signature, the co-signatures and the inclusion; §810) | 27 | 27 of 27 |

And in the positives of the five, its output is **identical line by line** to the binary's, the
anchor's fingerprint and the position derived from the consumption included. What it uncovered:
`NUCLEO.md` §6 does not write `posicion_de_consumo` nor the empty leaf nor the `isRight` convention,
which live in RFC-0006 §413 (the low 63 bits of the first eight bytes; the empty leaf is the zero
digest; `isRight[i]` is bit `i`) and have no KAT; and the key's fingerprint, which is Blake3, enters
`ancla_digest` read as a Digest of four elements, which leaves unsaid what happens if a limb falls
outside the field (2⁻³² per limb): here it was ROJO with a name, and the reference REDUCED it. Since
§640 (RFC-0016, `NUCLEO.md` section 6, «Canonicidad» [canonicity]) the rule is written: a producer
reduces —`limbos_reducidos`, like `resumen_con_dominio`— and a reader rejects what is not less than
`p`, with the same text in both implementations.

⚠️ §793: here it said «the four families», and the table has carried five since §649 added
`rotacion`; `paquete` 72, the count of §640, which §663 took to 73, §693 to 83 and §780 to 84; and
`conflicto` 16, which §664 took to 17; and «How it is run», above, said «173 de 173 (§771)» until
§780, which took it to 174. The five rows
are the output of
`bash tools/conformidad.sh tools/segunda/verificador.py spec/vectors/<familia>/MANIFIESTO.txt`
on the tree of §789 (`8c02941`): 174 of 174, with exit 0 in the five. That the output of the
positives is the binary's line by line is the measurement of S623 and, for `rotacion`, that of E5b
of RFC-0015 (§649): it has not been compared again, because it requires the compiled binary.

⚠️ §810: with `ancla-cofirmada`, six families and **201 of 201**, with exit 0 in the six, on the
tree of §794 (`b6491ef`) and this seal; until here its 27 entries came out `NO SOPORTADO` (0 of 27,
measured with the `verificador.py` of §794). In that family the comparison with the binary HAS been
done, with `zk-ssl-verify` compiled in this seal (`cargo build --release --locked -p zk-ssl-verify`),
on the tree of §794 and again on that of §810, whose crates are those of §808: in the 5 positives
the output is the binary's byte for byte, the anchor's fingerprint, the timestamp and the SHA-256
fingerprint of each witness's key included; in the 22 negatives, the `ROJO` line is the same letter
for letter and the exit code too. What the binary prints before the `ROJO` —the `1/4` to `3/4` lines
it reached— the second does not print: it collects the lines and only gives them with the VERDE, in
the six forms.

Since §810 `juzgar` also rejects a lone surrogate (`\ud800`) in a string, which serde_json does not
read and `json.loads` does: it comes out `ROJO: JSON ilegible: …`, with exit 1, as in the binary.
⚠️ §852: here it went on «It does not reject the rest that `json.loads` accepts and serde_json does
not —`NaN`, `Infinity`, `-Infinity` and a number outside the range of an `f64` (`1e400`)—: in a field
the second does not read they still give VERDE where the kit says “JSON ilegible”, as already
happened in §794 (measured with `"zz": NaN`, `Infinity` and `1e400` in
`ancla/ancla-derivada.json`)». It is no longer true for those four forms: the next paragraph says
so. With the `verificador.py` of §794, `ancla/ancla-derivada.json` with a field `"zz": "\ud800"`
came out VERDE, and
the binary gives `ROJO: JSON ilegible: unexpected end of hex escape …`. Measured with `zk-ssl-verify`
compiled in this seal, on that envelope and two of `ancla-cofirmada`, outside the tree
—`cofirmada-un-testigo.json` with one more witness, `x\ud800`, without a line in the note, and with
a witness `testigo.invalid/\ud800` that co-signs, signed by the test signer of `juez_medio.py`—: in
the three, the binary and the second give `JSON ilegible` with exit 1. The letter after the colon is
different (here, `'utf-8' codec can't encode …`), as in `paquete/rechazo-json-ilegible.json`.

⚠️ §852 · **The numbers serde_json does not read.** Since §852 `juzgar` reads with `parse_constant`,
`parse_float` and `parse_int`: `NaN`, `Infinity` and `-Infinity`, and a number serde_json does not
read because it does not fit in an `f64` —a real such as `1e400`, an integer of 401 digits—, come
out `ROJO: JSON ilegible: …`, with exit 1. The range rule is that of `_fuera_de_rango` in `cable.py`,
copied and not imported (`cable.py` imports `juez_cabezas.py`): it composes the `f64` like
serde_json, without correct rounding, and that is why `1.7976931348623158e308`, which `float()`
reads as finite, is not read. Measured with `zk-ssl-verify` compiled in §852, with `"zz"` at the end
of `paquete/posicion-v2.json`, value by value: in 26 —the three constants, reals and integers on both
sides of the `f64` boundary, `1e-400`, `0e999999999999`, `1e2147483648`, `18446744073709551616`—,
the binary and the second give the same verdict, VERDE or `JSON ilegible`; before §852, the second
gave VERDE in the 17 the binary rejects. Five enter as vectors of the `paquete` manifest:
`rechazo-json-nan.json`, `rechazo-json-infinity.json`, `rechazo-json-menos-infinity.json`,
`rechazo-json-1e400.json` and `rechazo-json-entero-401-cifras.json`. No other value has been
measured, nor a number in a field the second reads.

**The STARK verifier** (§626). The 58 proofs of the six families the kit verifies today go
**hidden** (the mark `arqueo:oculta:1` in the meta of the trace: L = 2T, one more column, the
exemptions of the inner AIR plus T) and **salted** (each leaf is `merge(item, sal)`); `stark.py`
does both paths. The judge composes the pair of each vector and compares it with the manifest:

| measure | value |
|---|---|
| pairs compared with their manifest / that say what they must | 23 / 23 |
| of them, positives that verify | 12, in the six families |
| negatives rejected with the judge's cause | 11: `InconsistentOodConstraintEvaluations` where the statement lies (another account, another bound, another amount, another receiver, another mark), the band's ceiling, the subroots that do not climb to the root, `nacido ≥ seq` |
| `completitud`, by its three verdicts | the accepted pledge verifies against its head; the one rejected with proof does NOT verify against the judged one (`ConstraintQueryDoesNotMatchCommitment`); that of the RECHAZO SIN FUNDAMENTO DOES |
| the same cause the binary prints, pair by pair | 23 of 23, measured by running `zk-ssl-verify` over each vector |
| falsifiers (one byte flipped in three thirds of the proof) that stop verifying | 33 of 33 |
| vectors that fall before the judge (signature, head, fields) | 35, counted and not compared |
| one verification in Python | 0.2 to 0.5 s (Edad, the multi-segment one, the slowest); the whole judge, 12 s |

⚠️ §793: the table is that of §626 and its 23 pairs are not today's. On the tree of §789
(`8c02941`), `python3 tools/segunda/juez_stark.py` gives 34 of 34 pairs that say what they must —11
that verify, in the six families, and 23 that do not, with the judge's cause—, 48 proofs judged, 35
vectors that fall before the judge and 33 of 33 falsifiers silenced, which the paragraph of §706,
below, already says. The row of the binary's cause, pair by pair, has not been measured again: it
needs `zk-ssl-verify` compiled.

That table is the measurement of §626 and has not been redone. The paragraphs that follow say what
each seal changed; today's figure is that of §723: 76 of 76 pairs. ⚠️ §723: it said «that of §721:
70 of 70 pairs».

What the measurement fixed on writing it: that the Fiat-Shamir seed is the context **plus the public
inputs**, so that a badly composed statement does not even reach the Merkle authentication; that the
«last row» assertions of a hidden AIR use the **inner** length T; that the periodic columns of
Banda, Prenda, Cobro and Pago measure the whole inner trace (512) and are evaluated at `z²`; and
that the coin's `check_leading_zeros` counts the **trailing** zeros of the first word.

**The mark, like the kit (§706).** Until §706 `stark.py` read the hiding mark behind the options,
returned whatever m it brought and judged an empty meta as an unhidden proof; the kit looks at the
form, then at the mark, which has to be the house's with m = 64 (`comprobar_marca`, §651), and then
at the options. An honest age proof with m = 32 in the mark was ROJO in the kit and VERDE here, and
the mark negatives fell here for another cause. Since §706 the second demands the same, in the same
order and with the same text, `marca de la ocultacion {lectura}; el enunciado pide m = 64`, and the
self-test checks the house's and the four readings it rejects. Six negatives of the age and the
payment fix it (`neg-marca-m32`, `neg-meta-vacio` and `neg-marca-m32-y-opciones`): with the earlier
`stark.py` the judge gives 28 of 34 pairs; with this one, 34 of 34, 48 proofs judged, 35 vectors
that fall before the judge and 33 of 33 falsifiers silenced. The check was read from the kit, like
the AIRs; its text and its four readings are also given by `spec/PAQUETE.md` §5.
⚠️ §914: since the `zkssl/0.5` cut the second reads the **v2 mark** before anything else
(`comprobar_marca_v2`, with the family each class declares, `FAMILIA`, the registry of
`spec/NUCLEO.md` §4 bis), with the texts of D-G of RFC-0018 letter for letter and the 1h bis of the
columns; it demands the form with the two columns the mark declares and pairs them in DEEP
(`parear_deep`). The v1 mark is only named (`marca v1: prueba de zkssl/0.4; la juzga el kit 0.4.x`),
and the texts above are those of `zkssl/0.4`. Measured in the seal: 89 of 89 pairs (proofs judged:
103; 44 fall before), 36 of 36 falsifiers.

**The causes of a malformed proof, like the kit (§711).** Since §711 the winterfell fork validates
what it reads from a proof before building, and rejects it with a named cause
(`crates/winter-air/src/causa.rs`):
`prueba mal formada: opciones / traza / contexto / consultas / marco OOD / FRI fuera de rango`, and
`lote fuera de rango` in `zk-ssl-air/src/sal.rs`. `stark.py` reads with the same conditions and
repeats those causes; it looks at the OOD frame in the order of `OodFrame::parse` and at the FRI
layers in that of `VerifierChannel::new`, and its self-test fixes it with four pairs over
`edad-todos.json` (`pares_fijos_del_orden`). `juez_stark.py` counts `prueba mal formada` among the
judge's messages. Four negatives of the age and the payment fix it (`neg-prueba-opciones-cero` and
`neg-prueba-consultas-cero`): with the earlier `stark.py` the judge gives 34 of 38 pairs
(`UnacceptableProofOptions` and `ProofDeserializationError("queries")`, a cause other than the
kit's); with this one, 38 of 38, 52 proofs judged, 35 vectors that fall before the judge and 33 of
33 falsifiers silenced. Where the kit gives a deserialization error without a name, `stark.py` gives
another text of the same class: the rest of its messages do not carry a named cause yet.

**The clock of the extension, like the kit (§712).** Since §712 the kit judges, after the
consistency, the clock of the extension (`el_reloj_avanza`, RFC-0012 D-C): with the same `mmrSize`,
the same digest; with a greater `mmrSize` and the same key, the embedded index of the new one
strictly greater. `verificador.py` has the same judge, with the same name and the same texts
(`extension: mismo mmrSize y digest distinto` and `extension: el indice embebido no crece`), and
calls it in the same place: the extension, the consumption and mode 3 of the anchor, where the
anchor plays the old one. The nine vectors of §712 in `paquete/`, `consumo/` and `ancla/` fix it,
and that is why the table above counted 94, 20 and 23 (in `ancla`, 44 since §714).

**The fork, like the kit (§714).** Since §714 the anchor envelope has a mode 5: `rama`, another
signed head of the same operator with the same `mmrSize` and another digest, which comes out
DETECTION with exit 0 (`spec/PAQUETE.md` 2.12). `verificador.py` judges it in `bifurcacion`, with
the same texts and in the same order as the kit. The form and the mode are decided by the PRESENCE
of the key, as in the reference: `"rama": null` is a branch, and `"ancla": null`, an anchor. Before,
`doc.get(k) is not None` took a `null` as absent, and in five envelopes the second gave VERDE where
the kit gave ROJO. In the envelopes that read the head with `leer_cabeza_de` (the anchor, the
consumption, the conflict and the overlap), a head that is not an object comes out
`{sujeto}: no es una cabeza (objeto)`, without a traceback. The extension one does not go through
there: with `"vieja": 7` the second still blows up, with exit 1, where the kit gives ROJO with its
text. That of the medium's note (`ancla-cofirmada`, §810) does not go through there either, and does
not blow up: with a head that is not an object it gives `falta formatVersion o no es cadena 0x`, the
kit's text. No vector fixes that common guard. And `verificar_cabeza` now checks the signature
before the preamble, like the kit: a valid signature of another head is
`la firma es VALIDA pero de otro mensaje`, the kit's text, and not «el preambulo recuperado no es el
esperado» [the recovered preamble is not the expected one]. Over the 225 vectors of the six families
it reads, that change of order moves the output of one, `ancla/neg-rama-firma-de-otra-cabeza`. The
rest is fixed by the twenty-one vectors of §714 in `ancla/`, and that is why the table above counts
44 in `ancla`.

**The FRI remainder, tied as in the kit (§713).** The last FRI commitment is the hash of the
remainder, and `winter-fri` 0.13.1 does not check it when verifying. Since §713 the fork's channel
checks it (`read_remainder`), before the degree and the folding, and `stark.py` does the same in the
same place of `verificar_fri`: `hash_e2(resto)` distinct from the last commitment is
`FriVerificationFailed(RemainderCommitmentMismatch)`, the same name as in the kit. Its self-test
fixes it with two pairs (`resto_atado`): the captured negative of the age, `neg-prueba-resto-ajeno`,
and `edad-todos.json` with one bit of the remainder. With the earlier `stark.py` the judge gives 38
of 39 pairs, and the one that fails is that negative, VERDE; with this one, 39 of 39, 53 proofs
judged, 35 vectors that fall before the judge and 33 of 33 falsifiers silenced.

**A single encoding, like the kit (§715).** Since §715 the fork's reader demands the minimal
encoding of each `usize` of the proof, and the channel, one row per queried position and one group
per folded position. `stark.py` does the same: `Lector.usize` demands the minimal encoding, with the
same cause and the same detail (`prueba mal formada: usize no canonico`); `leer_prueba` rejects the
tail of the proof (`UnconsumedBytes`, §653); the opening of each table and the batch of each FRI
layer reject their leftover bytes, in Rust's order; `verificar_con_sal` demands as many items as
indices, and the batch, that each vector be consumed whole (§659, which was missing here).
`juez_stark.py` counts `UnconsumedBytes` and `not all bytes were consumed` among the judge's
messages. The self-test gains `una_sola_escritura`: eight fixed pairs over the new vectors of
`edad/`. The twenty-six negatives of §715 in `edad/` and `pago/` fix it: with the earlier `stark.py`
and `juez_stark.py`, the judge gives 48 of 57 pairs —the six over-long `usize`s and the extra node
in the age and the payment, and the extra row in the age, VERDE—, and the eight tails fall before
the judge, without comparing; with these, 65 of 65, 79 proofs judged, 35 vectors that fall before
the judge and 33 of 33 falsifiers silenced.

**The cap of the age's `m`, like the kit (§717).** Since §717 the age judge rejects `m` above
`zk_ssl_air::M_MAX`, 13, before reading the proof. `airs.py` carries the same cap, `M_MAX` in
`Edad`, with the same cause (`m = {} fuera de 3..={M_MAX}`), and a test of `zk-ssl-air` compares the
two lines. `juez_stark.py` counts `fuera de 3..=`, without the value, among the judge's messages.
`stark.py` already multiplied the exemptions as a stream and does not change. The three negatives of
§717 in `edad/` fix it: with the earlier `airs.py` and `juez_stark.py`, the judge gives 66 of 68
pairs —the two above the cap fall in the OOD coherence and not by their cause, and the whole judge
takes 108 s—; with these, 68 of 68, 82 proofs judged, 35 vectors that fall before the judge and 33
of 33 falsifiers silenced, in 17 s.

**Canonicity, like the kit (§718).** Since §718 the age judge rejects, before reading the proof, an
`emisor` that is not an account index (`emisor = {} no es un indice de cuenta: no cabe en 32 bits`)
and one other than 0 with `todos` (`todos con emisor = {}: todos se escribe con emisor 0`), and
those of the payment and the claim bound their integers over the `u64` as it arrives, before
reducing. `airs.py` carries the same bounds, with `BITS_CUENTA = 32` in `Edad` and the same causes
in the same order, and a test of `zk-ssl-air` compares its `BITS_CUENTA` with the judge's.
`verificador.py` reads the quantities of the act with `u64_canonico`, like the kit's `q_de`. Six
negatives of §718 fix it: in `rotacion/`, the act with its `mmrSize` written as `N + p`, which with
the earlier `verificador.py` gave VERDE with exit 0 (the family, 34 of 35); and in `edad/`, the
issuer `2^32`, which with the earlier `airs.py` and `juez_stark.py` fell in the OOD coherence and
not by its cause (the judge, 68 of 69). The other four, the issuer `p` and the three aliases of the
payment and of the claim, fall in the reader before the judge. With these, 69 of 69, 83 proofs
judged, 39 vectors that fall before the judge and 33 of 33 falsifiers silenced, in 13 s.

**The value of the cap, fixed (§721).** The three negatives of §717 hold with any cap from 13 to 16.
`neg-m-14-sobre-tope` (`nextPending` = `2^13 + 1`) falls by the cap only if the cap is 13. With
`M_MAX = 14` in `airs.py`, the judge gives 69 of 70 pairs, and the one missing is that one: it gives
the OOD coherence and not its cause. With `M_MAX = 12`, also 69 of 70: `neg-m-13-fabricada` falls by
the cap. With 13, 70 of 70, 84 proofs judged, 39 vectors that fall before the judge and 33 of 33
falsifiers silenced, in 12 s.

**The band and the pending claim, in their two versions, like the kit (§723).** Since §723 Banda and
CobroPendiente carry their version: 2, with the three range segments in 62 bits (bit 62 at zero),
and 1, with those of 63, which only judges the proofs already issued. `airs.verificar_v2_o_v1` is
the `verificar` of the two `.rs`: v2 first; if it does not verify, v1, accepted only with the upper
bound at `COTA_V1 = 2^32 - 2` or above, and below it `V1EnLaVentana`, with the same cause prefix as
the judge (a test of `zk-ssl-air` compares the two lines). The exception is `PUBLICADAS_V1`: the
only v1 proof below the bound that was already published (`rechazo/saldo-insuficiente.json`),
recognized by the fingerprint of its content (`huella_del_contenido`, that of `zk-ssl-air`) and its
statement; another test of `zk-ssl-air` compares the list with the judge's. The STARK judge compares
the fourth state of the rejection envelope (exit 3, `DECLARADA, NO PROBADA: banda: …`) with
`V1EnLaVentana`. The eleven vectors of §723 fix it: the six with a proof of `rechazo/` and
`pendiente/` enter the judge, and the five of `completitud/` fall before. With the earlier `airs.py`
and `juez_stark.py`, the judge gives 71 of 75 pairs —the four v2 positives, two of the band and two
of the claim, fall in the OOD coherence— and the v1 envelope in the window falls before the judge,
with the command's fourth state, and its pair gives VERDE; with these, 76 of 76, 90 proofs judged,
44 vectors that fall before the judge and 45 of 45 falsifiers silenced, in 15 s.

**The chain of the log** (§797). ⚠️ This measures the **wire**, not the core: `chain` is of the
REGISTRO class (`spec/NUCLEO.md` §«2. Las cuatro clases» [the four classes]), its composition has no
KAT (its pieces do: `spec/RPC.md` §«La composición de `chain`, byte a byte») and
`conformance --check`, which requires the prover, declares it not reproducible since §538 (RFC-0009,
D-AI). What is checked here is the composition: with the entries as the vector brings them, or as
`zkssl_logEntries` serves them, `proofDigest` included.

| measure | value |
|---|---|
| chains of `zkssl-0.4.json` recomposed | 6 of 6, all of era 2 |
| their `proof_digest` recomposed without the proof | 4: the two `OpenAccount` (constant) and the two `Mint` (from their `compromiso`) |
| `proof_digest` declared | 2: `Send` and `Claim`, proof bytes (D-AI) |
| the four wire vectors, `zkssl-0.1` to `zkssl-0.4` | 24 of 24; the 6 of `0.1` are the only ones of era 1 |
| the six of `zkssl-0.4.json` rewritten as the wire serves them, `LogEntry[]` in camelCase | 6 of 6 |
| ⚠️ labels that some entry of the four vectors cross-checks | **4 of 13**: `OpenAccount` 1, `Mint` 2, `Send` 8 and `Claim` 9. The other nine, without cross-check: with that of `Consumo` changed to 14 in a copy of `cable.py`, they stay at 24 of 24 |
| ⚠️ classes whose `proof_digest` R6 fixes without the proof, cross-checked | **2 of 7**: `OpenAccount` and `Mint`. Without an entry: `Freeze`, `Recovery`, `Governance` and `MintToPending` (the authorization seal) and `Consumo` (the empty proof) |
| falsifiers inside the judge (one field altered each, or the order of the last merge) that stop recomposing | 8 of 8 in era 2, 7 of 7 in era 1 |
| falsifiers outside the judge, over copies in a temporary directory (§797) | 13 cases. With the first draft, 5 + 4 + 1 + 1 + 2: 5 tracebacks (broken JSON, numeric root, an entry that is not an object, the wire's list, a `kind` that is not text but a list), 4 greens that should not have been (upper-case hex, `seq` `0x١`, repeated key, foreign key), 1 false red (`OpenAccount` of era 1 with the empty proof), 1 `ROJO` for `KeyError: 'root_old'` (an entry of the vector with mixed spelling, `rootOld`) and 2 greens without a warning (the truncated one; the eras that go backwards). Now: 9 `ROJO` with a name, 2 greens with their `DECLARADO` or their `NOTA` (the wire's list; era 1) and the 2 it does not reject, with their `NOTA` (the truncated one; the eras that go backwards). And two from the second round (§797): `"compromiso": null`, `ROJO` by R7, where `LogEntryDto` would read era 1; and a file of a single entry (the `OpenAccount` of era 1 above, or the first one of `zkssl-0.4.json` alone), to which the falsifier of the previous one, which set it to zero when it already was, gave a false `ROJO FALSADOR`: now it sets `[1, 0, 0, 0]` and both come out green, and with a copy of `cable.py` that ignores the previous one the falsifier gives it `ROJO` |
| ⚠️ a truncated log (without the last entry) | **5 of 5**: it does not see it, and says so in a `NOTA` per file; nor one rewritten coherently |
| `epoch_digest` of the vector | **not** recomposed from the file: `pendingRoot` and `frozenRoot` are missing |
| the whole judge over `zkssl-0.4.json` / over the four | ~0.5 s / ~0.9 s |

It does not enter the canon: it is run by hand.

**The medium** (§810). `mldsa44.py` verifies ML-DSA-44 from FIPS 204, and `medio.py` the note and
the inclusion; `juez_medio.py` judges them against what other hands wrote, 92 of 92 (⚠️ §859: it
said 87 of 87, the count of §810; the five of the last row take it to 92):

| measure | value |
|---|---|
| the 33 notes of §632 (30 from torchwood over the `crypto/mldsa` of Go 1.27 and 3 from the `ml-dsa` 0.1.1 crate) with the verdict of column 2 of their `MANIFIESTO.txt`, the variant of `ErrorDeNota` included | 33 of 33: 10 accepted, 23 rejected, and among them the four in which the medium is stricter than torchwood (the non-canonical base64, two lines of the publisher, twice, and the foreign `origin`) |
| torchwood's vkeys, redone with the test signer's KeyGen from the seeds of `torchwood/main.go` | 2 of 2 |
| the publisher's and the witness's lines of `pos-01`, `pos-05` and `pos-13`, signed here deterministically with their timestamp | 6 of 6, byte for byte with Go's |
| the BOUNDARY signatures: z with `‖z‖∞` = γ1 − β = 130,994, valid in everything else, and another with 130,993 | the first does NOT verify and the second DOES, here and in the binary |
| a hint where the low part of w̃ is ZERO, which no FIPS 204 signer puts: with a tailored key (t1 = 0, and w̃ = A·z does not depend on c), z with a single coefficient, −83,537, and w̃ = 42·2γ2 exact in one coefficient, with its only hint there | UseHint lowers r1: with w1 = r1 − 1 it verifies, and with r1 + 1 it does not |
| the uniqueness of the encoding: the hints out of order, a repeated hint, a padding other than zero, a cut that passes ω, some that pass ω + k (200, 201, 202 and 255, without reading outside the signature) and the last one below the penultimate, a non-empty context, one of 256 bytes, one bit in each third and one byte short | 12 of 12 do not verify, without an exception |
| the 98 `inclusion/` probes of transparency-dev | 6 positive, 26 that the type rejects and 66 the tree: the figures of `vectores_transparency.rs` |
| every path of every tree from 1 to 33 leaves / with the neighboring index, a sibling touched, one hash short or one too many | 561 verify / 2,225 do not |
| Rust's `{:?}` of the names that come out in a rejection (`debug_rust`): eleven strings with U+200B, a combining accent, Zs, Zl, Zp, Cf, Co, Cn, Other_Grapheme_Extend and the single quote, which is not escaped | 11 of 11 with what the `rustc` 1.94.1 that compiles the kit prints |
| five rules that exist and that nothing isolated (§859, below): a cut that decreases over a polynomial without hints —the signature of the hint over r0 = 0 with the cuts [1, 0, 1, 1]—; size 0 with a root that is not the SHA-256 of nothing, and a timestamp of 2^63, both in notes signed by hand with another tailored key (t1 = 0) over the message the rule forbids; a foreign line of `pos-05` with a `+` in the name; and the publisher's vkey with the `key_id` of another key | 5 of 5 rejected with their cause (the signature with [1, 0, 1, 1] does not verify; `FirmaInvalida` and `FirmaInvalida`; `Malformada`; `Clave`); and their 5 controls —the same input without what the rule forbids— are accepted |
| one ML-DSA-44 verification in Python (`verificar_nota` of `pos-05`, the mean of 20 runs) | from 7 to 18 ms depending on the machine's load, in three measurements of this seal; 8.0 ms (from 6.7 to 12.8) with the load at 4.3 of 4 CPUs. The whole judge, from 1.4 to 2 s |

Outside the canon, because they require compiling, in this seal: the two boundary signatures and one
from the test signer with salt, placed as the co-signature of one more witness in a copy of
`cofirmada-un-testigo.json` (outside the tree), give in `zk-ssl-verify` the same as here —130,994,
`la cofirma de testigo.invalid/frontera no verifica`; 130,993 and the FIPS one, VERDE with 2
witnesses—; and the `medio` example of `zk-ssl-medio`
(`cargo build --release --locked -p zk-ssl-medio --example medio`), with random seeds and trees of
1, 2, 3, 5, 8, 13, 21 and 34 anchors at their first, their central and their last position, signed
21 notes with salt and 42 co-signatures with salt: the 21 notes, the 42 co-signatures and the 21
inclusions verify in Python, and the 42 co-signatures with one bit changed do not.

**The mutants of the verifier itself** (§810), measured one by one over a copy of the tree —changed
by hand, with the judge of 87 and the harness of `ancla-cofirmada` run without bytecode cache,
restored with the same sha256—: of 29, **24 fall**, each with the check that watches it, and **5
live**, below.

| mutant | what kills it | judge / harness |
|---|---|---|
| in `mldsa44.py`: one coefficient of the table of the ζ; the sign of c in SampleInBall; `t1·2^(d−1)`; the domain prefix of μ; ExpandA with r and s swapped; UseHint reversed; the q − 1 edge of Decompose removed | the 10 accepted notes become `FirmaInvalida` | from 20 to 23 `ROJO` / 16 of 27 |
| SampleInBall with `j ≥ i` instead of `j > i` | the 3 notes of the crate, the co-signature of the witness of `pos-05` and its redone line | 82 of 87 / 26 of 27 |
| the norm with `>` instead of `≥`, and the norm unchecked | the boundary signature of 130,994 verifies | 86 of 87 / 27 of 27 |
| one bound less in the norm | that of 130,993 does not verify | 86 of 87 / 27 of 27 |
| `r0 ≥ 0` instead of `r0 > 0` in UseHint | the hint over r0 = 0: UseHint raises r1, and the signature with r1 − 1 does not verify and that with r1 + 1 does | 85 of 87 / 27 of 27 |
| the hints with `>` instead of `≥` | the repeated hint; the out-of-order ones do not kill it, because they go down with `>` too | 86 of 87 / 27 of 27 |
| the padding of the hints unchecked | the padding other than zero | 86 of 87 / 27 of 27 |
| without the bound `corte > ω` | the cuts above ω + k (200, 201, 202 and 255), which read outside the signature (`IndexError`) | 86 of 87 / 27 of 27 |
| in `medio.py`: the leftover bits of the base64 unchecked after a `=` | `neg-firma-no-canonica` and `neg-raiz-no-canonica` | 85 of 87 / 27 of 27 |
| the sibling of the inclusion on the right | the 98 probes, their 12 touched ones and the paths of 1 to 33 leaves | 84 of 87 / 23 of 27 |
| the foreign `origin` unchecked | `neg-origen-ajeno-firmado` and `neg-origen-cambiado` | 85 of 87 / 27 of 27 |
| two lines of the publisher | the two `neg-firma-repetida-*` | 85 of 87 / 27 of 27 |
| the control characters unchecked | `neg-retorno-de-carro` | 86 of 87 / 27 of 27 |
| the leaf without its `0x00` | the root of torchwood of 4 notes | 83 of 87 / 22 of 27 |
| the `debug_rust` that only escapes the control ones | the eleven strings | 86 of 87 / 27 of 27 |
| in `verificador.py`: the publisher of another key | `neg-medio-de-otra-clave` comes out with exit 0: the attack | 87 of 87 / 26 of 27 |
| the co-signature unverified | `neg-cofirma-tocada` comes out with exit 0 | 87 of 87 / 26 of 27 |

`r0 ≥ 0` and the cut above ω are only killed by the signatures the judge fabricates: no FIPS 204
signature carries a hint where the low part is zero —MakeHint does not put one while `‖c·t0‖∞ < γ2`,
which the signer demands—, and none of the notes carries a cut above ω.

⚠️ §859: the table and the 24 that fall are the measurement of §810, with the judge of 87, and have
not been measured again with that of 92. The five below no longer live: since §859 block 7 of the
judge kills them, each one alone with its case, and it gives 91 of 92 (measured over copies of
`tools/segunda`, with `python3 -B`); the unmutated tree gives 92 of 92. The harness of
`ancla-cofirmada` does not change: it stays at 27 of 27 with each one, because none of its entries
isolates them, and §859 adds none to it. What follows is what was written in §810.

The five that live give 87 of 87 and 27 of 27: no entry of the judge or of the harness isolates
them.
- In `mldsa44.py`, HintBitUnpack without `corte < indice`. The decreasing cut of the judge is the
  last one, which leaves hints in the padding, and the padding rule stops it. The rule is needed:
  the signature of the hint over r0 = 0 with the cuts [1, 0, 1, 1] instead of [1, 1, 1, 1] is the
  same h written another way, and the tree rejects it and the mutant accepts it.
- In `medio.py`: the hash of the empty subtree of `mensaje_cofirmado` unchecked (the only note of
  size 0, `pos-00-anclas`, carries the root of the empty tree); the `+` of `nombre_valido` (no
  signature line carries one in the name, and a vkey cannot: it is split by it); `marca > I64_MAX`
  (`neg-marca-2-63` falls anyway, by the signature); and the `key_id` of `leer_vkey` not compared
  (no vkey of the vectors carries that of another key).

**The verifier under wasmtime** (`wasmtime` 49.0.0 from PyPI; `rustc` 1.97.0):

| measure | native | `wasm32-wasip1` |
|---|---|---|
| size of the binary | 2,517,640 B | 1,587,916 B (3,502,520 B precompiled) |
| sha256, first 16 | `e524b7b6cb02e45c` | `0dbd46a1963dd396` |
| `conformidad.sh`, ten manifests | 316 of 316 | 316 of 316, same verdict on each entry |
| the ten manifests, wall clock | 9.28 s | 32.78 s |
| one completeness vector, mean of five | 18.4 ms | 93.3 ms, of which 66.8 ms are starting Python and loading wasmtime |

The ten manifests: `ancla` 21, `completitud` 73, `conflicto` 16, `consumo` 14, `edad` 11, `pago` 9,
`paquete` 70, `pendiente` 9, `prenda` 9 and `rechazo` 84. That of `cable` is the witness's, not the
verifier's, and does not enter.

⚠️ §793: the table and that count are from S623, on `e8ac246`, and have not been measured again: it
takes compiling the `.wasm`, and this seal does not compile. Today those ten manifests have 351
entries —`ancla` 22, `completitud` 78, `conflicto` 17, `consumo` 17, `edad` 15, `pago` 13, `paquete`
84, `pendiente` 10, `prenda` 11 and `rechazo` 84—, and `FAMILIAS` (`tools/artefacto.sh:31`) has
twelve: with `ancla-cofirmada` 27 and `rotacion` 34, they are 412. That today's `.wasm` passes them
is not measured, neither the ten nor the twelve; the loop above is still that of the ten that were
measured.
⚠️ §815: `completitud` has 81 since §815, with the three of kit 0.4.3: 354 entries in the ten and
415 in the twelve. The second verifier does not read the completeness (`NO_SOPORTADOS`), and its
STARK judge only looks at the pledge that goes inside: the three new ones carry no proof, so neither
of the two changes.
⚠️ §852: `paquete` has 89 since §852, with the five `rechazo-json-*`: 359 entries in the ten and 420
in the twelve. What was measured of the `.wasm` does not change: that it passes them is not
measured, neither before nor now.
⚠️ §711: `edad` has 17 and `pago` 15 since §711, with the four negatives of the malformed proof: 363
entries in the ten and 424 in the twelve. What was measured of the `.wasm` does not change: that it
passes them is not measured, neither before nor now; the four, on their own, give under `wasmtime`
the same cause as in native, measured on the tree on which the fix was made and not on this one (the
entry of §711).
⚠️ §712: `paquete` has 94, `consumo` 20 and `ancla` 23 since §712, with the nine vectors of the
clock: 372 entries in the ten and 433 in the twelve. What was measured of the `.wasm` does not
change: that it passes them is not measured, neither before nor now.
⚠️ §713: `edad` has 18 since §713, with the captured negative of the FRI remainder: 373 entries in
the ten and 434 in the twelve. What was measured of the `.wasm` does not change: that it passes them
is not measured, neither before nor now; the vector, on its own, gives under `wasmtime` the same
cause as in native, measured on the tree on which the fix was made and not on this one (the entry of
§713).
⚠️ §714: `ancla` has 44 since §714, with the twenty-one of the fork: 394 entries in the ten and 455
in the twelve. What was measured of the `.wasm` does not change: that it passes them is not
measured, neither before nor now.
⚠️ §715: `edad` has 33 and `pago` 26 since §715, with the twenty-six negatives of a single encoding:
420 entries in the ten and 481 in the twelve. What was measured of the `.wasm` does not change: that
it passes them is not measured, neither before nor now; the kit's manifests, with those twenty-six,
give under `wasmtime` the same lines as in native, measured on the tree on which the fix was made
and not on this one (the entry of §715).
⚠️ §718: `edad` has 38, `pago` 28, `pendiente` 11 and `rotacion` 35 since §718, with its six
negatives of canonicity and the three of the cap of `m` of §717, which its seal did not count here:
428 entries in the ten and 490 in the twelve. What was measured of the `.wasm` does not change: that
it passes them is not measured, neither before nor now.
⚠️ §721: `edad` has 39 since §721, with the negative that fixes the cap of `m` at 13: 429 entries in
the ten and 491 in the twelve. What was measured of the `.wasm` does not change: that it passes them
is not measured, neither before nor now.
⚠️ §722: `rotacion` has 48 since §722, with the thirteen of the common history: the 429 of the ten
do not change, because `rotacion` is not one of them, and they are 504 in the twelve. What was
measured of the `.wasm` does not change: that it passes them is not measured, neither before nor
now.
⚠️ §723: `rechazo` has 88, `completitud` 86 and `pendiente` 13 since §723, with the eleven of the
window: 440 entries in the ten and 515 in the twelve. What was measured of the `.wasm` does not
change: that it passes them is not measured, neither before nor now.

## What the second verifier does not read, declared (§693)

`verificador.py` reads seven forms of envelope —the position one, without `tipo`, and six types:
`extension`, `consumo`, `conflicto`, `ancla`, `ancla-cofirmada` and `solapamiento`, those of
`TIPOS_LEIDOS`— and declares the other six in `NO_SOPORTADOS`: `rechazo`, `edad`, `cobro_pendiente`,
`pago_en_curso`, `prenda` and `completitud`, those that require a STARK proof. ⚠️ §810: it said six
forms and seven declared types, with `ancla-cofirmada` among them, «those that require a STARK proof
or the medium»; it reads the medium since §810. One of them comes out
`ROJO: NO SOPORTADO: tipo {tipo}: …`, with exit 1: it fails closed, and `tools/conformidad.sh` never
counts it: it gives `ROJO <fichero>: NO SOPORTADO - …` (`spec/PAQUETE.md` §«6. El contrato del
mando» [the contract of the command]). Until §693 the text was «no lee este sobre todavia» [it does
not read this envelope yet], and the harness counted it as a negative passed when the manifest
pinned only the subject: `prenda/neg-receptor-mentido`, `prenda/neg-marca-mentida`,
`edad/neg-cota-movida` and the `neg-cota-movida` of `0.3/edad`, measured. Today, over the manifests
of those seven types, of `0.3/` and of `0.4/`, all the entries come out `NO SOPORTADO` and none
`OK`. The cause of a rejection is named with the binary's text (`spec/PAQUETE.md` §«6.1 La causa del
rechazo» [the cause of the rejection]): `la firma no verifica`, `acuse: RaizDistinta`, and the
reading of a `u64` with the minimal encoding of §662 and its texts.
⚠️ §810: measured again over the manifests of the six types that remain, of `0.3/` and of `0.4/`:
254 entries, all `NO SOPORTADO` and none `OK`; with the 201 it reads, the 455 of the seventeen
manifests that are not the wire's.

## The second consumer of the wire (§809)

`spec/RPC.md`, in «Los rechazos del cable» [the rejections of the wire], had left it written: *«Una
segunda implementacion del consumidor pone su ejecutable en lugar del adaptador»* [a second
implementation of the consumer puts its executable in place of the adapter]; and the row of
`vectors/cable/` of `spec/README.md` says whom that manifest is for: *«you write a second consumer
of `zkssl_signedEpochHead`»*. `cable.py` is that executable. Until here the only one that judged the
response was the witness of the CLI (`crates/zk-ssl-cli/src/witness.rs`), the reference consumer of
`RPC.md`, which verifies with `zk-ssl-verify`; `cobro`, `pago` and `prenda` also request the method
and read its response to assemble their envelope, but they do not judge the signature. `cable.py`
does not share a line with the witness: it reads the body, types it, and what remains it hands to
`juez_cabezas.py`.

**Against the manifest.**
`bash tools/conformidad.sh tools/segunda/cable.py spec/vectors/cable/MANIFIESTO.txt`: 27 of 27 and
exit 0, in about 2 s; the witness's adapter, `tools/cable_respuesta.sh`, also gives 27 of 27 (⚠️
§850: it said 23 of 23 in both, from before the three negatives of the encoding and
`no-existe.json`; below, «The two negatives it asked for»).

**Against a real node.** Two release `--dev` nodes (`--latido 1`, one with `--clave-fichero`,
journal, reception counter and signature index, and another without `--clave`), with their home
under `$HOME` and deleted at the end, as in the benches. In it also go the working directory of both
and the reception counter of the node without a key, with `--contador-recepcion`: without that flag,
a node without a key opens it in `recepcion.bin` of its working directory (`zk-ssl-node --help`,
§709), and started from the tree it leaves it at its root, where `git status` does not see it
because `*.bin` is in `.gitignore`. A funded account and a send of zeros rejected (`StaleState`),
which consumes a receipt. Three real responses of `zkssl_signedEpochHead`, kept as they crossed the
wire: the first, before the first heartbeat (`available: false`, «aun no ha habido latido» [there
has been no heartbeat yet]); a signed v6 head, with `nextIndex` 1, `totalSupply` 0x64 and
`recepCount` 1; and that of the node without a key (`available: false` and the head without a
signature). The signed one gives `VERDE: nueva` in both consumers, with the same line; the other
two, `ROJO: sin-firma · <el motivo que da el nodo>`, with the same text from `ROJO:` on (the witness
writes it after `Error: `).

**Against the tampered ones, differentially.** What `RPC.md` does not write was fixed by a
differential: responses tampered from the three real heads of the manifest (the v3, the v5 and the
v6), judged by `zk-ssl-cli witness --respuesta` and by `cable.py` —each key removed, set to `null`,
of another type or extra; each QUANTITY and each digest badly written or outside the field; bytes of
the signature and of the key, the OID, the versions and the `index`; pairs of defects, to fix in
which order they are looked at; broken bodies, and the edges of the reading—. The first version of
`cable.py` did not look at those edges, and in three it gave a class other than the witness's: a
deep nesting, a lone UTF-16 surrogate and a number outside the range of an `f64` (point 6). The
corpora of those differentials are not in the tree and their figures are not given here, because
they are not reproduced from it. What they fixed are the points below, and each case they name is
judged by hand with both consumers; judged so over the tree of §809, with a witness compiled from
it, both give the same exit in all of them, and the same class except in the three at the end of
point 6, in which the witness comes out without a `ROJO:` line (⚠️ §850: until §850; below, at the
end of point 6). With the same class, either the text from `ROJO:` on is the same, or the text of
one is the beginning of the other's —`cable.py` adds after it the name of the field
(`QUANTITY inválida`, and `en` and the key) or the witness the error of its library
(`la firma no verifica: VerificationFailed`)—, or the text is written by a library: `serde_json`
(`missing field`, `invalid type`, `unknown field`, its reading errors) against Python's `json` or
the text of `cable.py`, and the `xmss` crate (`InvalidOid(1)`, `InvalidKeyLength {…}`), which writes
OID 0 and 0x100 with its internal encoding: `InvalidOid(131072)` and `InvalidOid(131328)`, where
`cable.py` says the OID.

**What the measurement fixed, and `RPC.md` does not write.** `RPC.md` names «los dieciocho campos de
la cabeza, `publicKey`, `epochDigest`, `formatVersion`, `index` y `signature`» [the eighteen fields
of the head, `publicKey`, `epochDigest`, `formatVersion`, `index` and `signature`], says that
`custody` and `custodyChecked` always travel, and gives the order of the path: `result`, the typing,
the anchoring of the key, the signature, the recomposition. The rest was fixed by the differential,
looking at the witness as a black box:

1. **Thirty keys, and only four of the form.** `available`, `beatSeconds`, `custody` and
   `custodyChecked` are those the form of the wire does not let be missing (`SignedEpochHeadDto`,
   `crates/zk-ssl-wire/src/lib.rs`:791-798), and the absence of the last three is `forma del cable`;
   that of `available`, like its `null`, is `sin-firma`, because the witness looks at it before the
   form (`crates/zk-ssl-cli/src/witness.rs`:867; point 3). The other twenty-six are optional in the
   form, and a signed head demands those its version composes and the common ones —twenty-five in a
   v6, all but `reason`—, with `cabeza firmada sin X: el productor la sirvio incompleta`: also
   `domain` and `emittedAtUnix`, which are not signed. An optional one at `null` counts as absent,
   and `beatSeconds`, `custody` or `custodyChecked` at `null` is `forma del cable`. A repeated key
   is worth what the last one says, if what it overwrites is readable (point 6).
2. **An extra key breaks** (`forma del cable`): it is the reference's `deny_unknown_fields`, what
   `RPC.md` calls «aditivo en el cable, rotura en el parser» [additive on the wire, breakage in the
   parser]. RFC-0005 (D-B) says that this is a choice of the reference implementation and not a rule
   of the format, and leaves to the author whether the format makes it its own. `cable.py` chooses
   the same as the reference, which complies with either of the two outcomes of that decision; and
   that is why no vector pins it: it would turn it into a rule.
3. **The order.** First, that the whole body is read (point 6). Then `available`, before the form:
   what is not JSON `true` —`"true"`, `1`, `null`, or a `result` that is not an object— is
   `sin-firma`, with the node's `reason` if it is a string and «sin motivo declarado» [no reason
   declared] if not. The form is looked at key by key in order of NAME, not of arrival, and what is
   obligatory and missing, at the end. Then, if the written version is v4, v5 or v6, what its
   version adds; then what is common, in this order: `seq`, `epochDigest`, `emittedAtUnix`,
   `domain`, `formatVersion`, `mmrRoot`, `mmrSize`, `index`, the four roots of v1, `acusesRoot`,
   `n`, `signature` and `publicKey` —a v1 or v2 head has to bring the MMR pair too—. Then the
   version, read canonical and from the set; the `index`, canonical; the key; the signature; the
   preamble; the `index` against the embedded one; and the recomposition, which reads each value
   canonical in the order `seq`, `n`, the roots, `acusesRoot`, and from there each pair in the order
   of its version.
4. **The form looks at the encoding, not at the value.** A `u64` or a digest outside the field
   (RFC-0016) passes the form and falls on being read, with
   `QUANTITY "<valor>": <valor> no es canonico…` or `<campo>: no es un digest del campo`;
   `beatSeconds` and `emittedAtUnix` are never read as elements, and outside the field they break
   nothing. `domain` is not compared: another domain passes, because the preamble carries its own.
5. **The key is of XMSSMT-SHA2_40/8_256, OID 5, or it is not read.** Another OID of RFC 8391 —1, 4,
   6— is `clave publica ilegible`, not a signature that does not verify.
6. **The body that is not read** is of the class `respuesta ilegible`, the only one with a space
   instead of a hyphen: empty, cut, with a BOM, with `NaN`, with a tail; and what exceeds the three
   limits that the reference's reader —`serde_json` 1.0.151, that of the `Cargo.lock`, without
   `float_roundtrip`— puts on the WHOLE body, not only on `result`, and Python's `json` does not:
   - **more than 127 levels** of nested `[` and `{`, counted from the outer brace: 127 are read and
     128 are not —`{"result":[…]}` with 126 brackets is `sin-firma` and with 127 is unreadable, and
     nesting in `id`, in `error` or in an extra key is enough—;
   - **a lone UTF-16 surrogate** (`\ud800` without its pair, `\udc80` alone) in any string, key or
     value, signed or not: in `custody`, in `jsonrpc`, in the `reason` of a response without a
     signature, or in the value of a repeated key that the reading overwrites;
   - **a number whose `f64` comes out infinite** as that reader composes it —the mantissa in a
     `u64`, which drops the digits that do not fit, and a single power of 10 from its table—,
     wherever it is: `1e309`, an integer of 400 digits, and also `1.7976931348623158e308`, which
     correct rounding leaves at the largest finite `f64`; `1.7976931348623157e308` and `1e308` are
     read.

   RFC 8259 lets the reader set limits on the depth and on the range of numbers (section 9) and does
   not say what to do with a lone surrogate (section 8.2): they are not rules of the format, but of
   the reference's reader, like the extra key of point 2. `cable.py` imitates them to give the same
   class, and no vector pins them, which would turn them into rules. The edge of the `f64` was read
   from the source of `serde_json` in the cargo registry (`f64_from_parts`, without compiling it)
   and measured against the witness.

   And in three cases the witness does not meet the command's contract: it exits with 1 and
   **without a `ROJO:` line**, with Rust's I/O error —a file that does not exist
   (`Error: No such file or directory (os error 2)`), a directory
   (`Error: Is a directory (os error 21)`) and some bytes that are not UTF-8
   (`Error: stream did not contain valid UTF-8`)—, where `cable.py` gives
   `ROJO: respuesta ilegible · no se lee el fichero: …` or the decoding error. The exit is the same,
   and no entry of the wire's manifest asks for it.

   ⚠️ §850: that was until §850. Since then the witness gives in the three a line
   `ROJO: respuesta ilegible · …` and exit 1:
   `no se lee el fichero: No such file or directory (os error 2)`,
   `no se lee el fichero: Is a directory (os error 21)` and
   `invalid utf-8 sequence of 1 bytes from index 0`, measured with some bytes that start with
   `0xff`. The entry `no-existe.json` of the manifest asks for it, with the text
   `ROJO: respuesta ilegible · no se lee el fichero`, which both consumers give.

**The two negatives it asked for.** With the 21 entries from before, a consumer without the `index`
rule (§399) or that reduces a `u64` instead of rejecting it (RFC-0016) gave 21 of 21: measured by
breaking, in a copy, `cable.py` (`if False and declarado <= embebido`) and `nucleo.u64_canonico`
(returning `u % P` before the check of the bound; in place of the final `return u` it mutates
nothing). Two negatives enter, derived by one mutation of the v6 positive (`aaffc6ceee72f2fb`) and
registered in `HUELLAS.sha256` with `tools/check_vectores.py --registrar`:
`rechazo-index-atrasado.json`, with the `index` 5, equal to the embedded one, and
`rechazo-n-no-canonico.json`, with `n` = 0x5a0 + p, which recomposes the same digest if the reader
reduces. With them each mutant falls in its own —22 of 23, «exit 0, el manifiesto espera 1» [exit 0,
the manifest expects 1]—, and the two consumers of the tree of §809 gave 23 of 23 (⚠️ §850: it said
«of the tree give 23 of 23»; since §850 they give 27 of 27). Eleven other steps were already watched
by the 21, measured also by breaking each one over a copy: without verifying the signature
`rechazo-firma` falls; without comparing the preamble, `rechazo-v5-con-firma-de-v4`; without
comparing the recomposition, the three of recomposition; with v7 inside the set,
`rechazo-formatVersion-7`, and without looking at the set, the three of a version outside (1, 0x103
and 7); without what each version demands, the five `sin …`; without the common part,
`rechazo-sin-index`; without the obligatory ones of the form, `rechazo-forma-del-cable`; without the
length of the digest, `rechazo-epochDigest-corto`; without `result`, `rechazo-sin-result`; and
without the class `sin-firma`, `rechazo-available-false`. What no vector pins the manifest does not
watch, and breaking each one in a copy `cable.py` still gives 27 of 27 (⚠️ §850: it said 23 of 23):
the limits of the reader, on purpose (point 6: the depth at 100,000, without looking at the nesting
and the surrogates, or without the range of the `f64`); the extra key (point 2); the OID (point 5);
and the `index` read canonical (point 3). With the last one broken, `cable.py` gives `VERDE` where
the witness gives `ROJO`, and only the cases of that point measure it, judged by hand with both.

⚠️ §850: here it also said «and the minimal encoding of a QUANTITY and of a hex (point 4: `0x05`, or
a hex in upper case)», «with one of the last three broken» and «there is no vector of a
non-canonical `index`, nor of a QUANTITY with leading zeros, nor of a hex in upper case». Since §850
there are vectors of an `index` with a leading zero, of a QUANTITY with leading zeros and of a hex
in upper case; of the non-canonical `index` (the bound of point 3) there is still none. The three
derive by one mutation of the encoding of the same v6 positive, registered with `--registrar`:
`rechazo-index-no-minimo.json` (`index` `0x06`), `rechazo-quantity-con-ceros.json` (`mmrSize`
`0x05`) and `rechazo-hex-mayusculas.json` (the `epochDigest` in upper case). Both consumers reject
them on typing, with `forma del cable`. Measured in a copy, each mutant of `cable.py` falls in its
own: without the leading-zero rule in `tipar`, 25 of 27 (those of the `index` and of the QUANTITY
fall); without it only for the `index`, 26 of 27; only for `mmrSize`, 26 of 27; and with a `_hex`
that admits upper case, 26 of 27 (that of the hex falls). The `index` read without the bound of
`u64_canonico` (`int(cab["index"], 16)` instead of `q(cab, "index")`) stays at 27 of 27: none of the
three looks at it. The other four of the list above, broken the same way in a copy, also give 27 of
27: the depth at 100,000 without looking at the nesting or the surrogates, without the range of the
`f64`, the extra key and the OID. With the 23 entries from before, the first three gave 23 of 23,
measured.

## What the second implementation uncovered in the spec

It is what entry 85 of the BACKLOG exists for: *«una spec que sólo implementa un código no es una
spec»* [a spec that only one code implements is not a spec]. Five things that whoever writes from
the spec cannot know (⚠️ §810: it said «four things» and «from section 6»; the fifth is the
medium's, from §810):

1. **`hoja_de_acuse(hashPrueba, seq, n)`** was not in section 6. `RPC.md`, in «`zkssl_ackPath`»,
   gives the leaf as `acuse_digest(hashPrueba, epoca, n)`, and in «El acuse en la respuesta (§274)»
   [the acknowledgement in the response] says `epoca = logSeq + 1`, but no page says that the `seq`
   of the leaf is that `logSeq`. With `epoca = seq` the KAT does not reproduce; with `seq + 1`, it
   does. Measured.
2. **`hash_del_lote`**: the encoded length is that of the **bytes** of the composition, `48·k`, as
   in the mould of §116; it is not `k`. The sentence «`k` va en la longitud» [`k` goes in the
   length] of section 6 and of RFC-0014 E2 has to be read as «`k` is left implicit in the length».
   Measured.
3. **`cima`**: section 6 does not say whether each leaf goes through `mmr_hoja` inside the peak. It
   does, which is the reading of RFC 6962; the KAT supports it.
4. **The format of the KATs** encodes the same type in two ways: `embeber.x` and the `balance` and
   `nonce` of `native_leaf*` go as u64 in **eight little-endian bytes**; `seq`, `n`, `t`, the
   counters and `as_digest.x` as the u64 in **hex**. A judge has to know it from outside.
5. **The texts of the medium's note** (§810). `PAQUETE.md` §5 gives `la nota: {e}` with `{e}` «la
   regla de `zk-ssl-medio::nota` que cae» [the rule of `zk-ssl-medio::nota` that falls], and
   `el ancla de esta cabeza no esta en la posicion {p} de las {n} de la nota: {e}` without saying
   `{e}`; the `MANIFIESTO.txt` of `ancla-cofirmada` pins three of those texts
   (`checkpoint mal formado`, `ninguna firma del publicador`,
   `la firma del publicador no verifica`), which are only in the `Display` of `ErrorDeNota`, and
   those of the inclusion, in English, in that of `SubtreeError`. `medio.py` copies them from the
   `.rs`, as `airs.py` does the AIRs. And one of them misnames who signs: a WITNESS co-signature
   that does not verify comes out
   `la cofirma de {testigo} no verifica: la firma del publicador no verifica`, in the binary and
   here.

## In the canon

The three judges and the second verifier run in every seal (`tools/canon.sh`, block «3 duodecies»),
at pin zero: a new KAT or a new head version that the second implementation does not reproduce turns
the seal red with its name; the judge of `kat_xmss/` runs in the same block, so that a change of
bytes of the `xmss` crate is seen in the seal; the second verifier runs against the six manifests
(⚠️ §810: it said «the five»; the sixth, `ancla-cofirmada`, is from §810) —⚠️ §793: here it said
«the four», and the loop of the canon (the `for F in …` of block «3 duodecies» of `tools/canon.sh`;
⚠️ §810: until here it cited it by its line, `:578`, which §798, §800 and §804 had already shifted
to 670, and the fourteen that §810 puts earlier in the block take it to 684: it is cited by its
block, which does not move) has carried five since §649—, so that a new vector has to pass through
both codes; and since §626 the STARK judge runs in the same block, so that a new proof of those
families has to verify in Python too. Since §810 `juez_medio.py` also runs and, in the loop of the
second verifier, the sixth family, `ancla-cofirmada`: some 6 s more. `wasm_runner.py` does not run
in the canon: it needs `wasmtime` from PyPI. Since §851 the two pieces of `cable.py` also run, in
the same block and at pin zero: the chain of the log, with `juez_cadena.py` over the vectors
`spec/vectors/zkssl-0.*.json` (a glob: a new catalogue enters by itself), and the second consumer,
with `tools/conformidad.sh` and the wire's manifest, the same one that block «3 bis cable» runs with
the witness's adapter: some 3 s more. The `SIN EJERCITAR` lines of the chain judge do not turn the
seal red; they stay in its output. ⚠️ §851: here it said «`cable.py` neither (§809): neither
`RPC.md` nor `spec/README.md`, which ask for it, say that the canon runs it, and block «3 bis cable»
still runs the witness's adapter. Putting it next to the second verifier is one more row of «3
duodecies», at pin zero, and it remains to be done». And in «What was measured», under the table of
the chain of the log, «It does not enter the canon: it is run by hand» holds until §851.
The whole `--sello` was run with the block inside on 01-10-2026: VERDE, 1,054 s of tests.

## What it does NOT cover

- The whole medium: `medio.py` does not check the consistency between two sizes of the tree (a
  witness checks it before co-signing; the envelope does not ask for it), nor does it judge the
  witnesses, nor the freshness of the timestamps: the same as the kit, which reports and does not
  judge (RFC-0013 D-D). And the test signer of `juez_medio.py` belongs to the judge and not to the
  verifier: it signs deterministically to redo Go's lines, and outside FIPS 204 for the two boundary
  ones; with the tailored key of the hint over r0 = 0 (t1 = 0) anyone signs, and it is good for
  nothing else. Since §859, block 7 signs by hand, with another tailored key (t1 = 0, z = 0 and
  without hints), two messages that `mensaje_cofirmado` refuses to form —that of size 0 with a root
  that is not that of the empty tree and that of a timestamp of 2^63—: they are written byte by byte
  by `_mensaje_sin_reglas`, the judge's, without any rule. None is a vector: the kit has not judged
  them in this seal, nor does the harness judge them.
- Rust's `{:?}` of a name with a character assigned after Unicode 14.0, or that has changed category
  since then. `debug_rust` takes the categories from `unicodedata`, the Unicode of the Python that
  runs (14.0 in 3.11), and the Other_Grapheme_Extend table of 16.0 (that of `regex-syntax` 0.8.11);
  the kit, those of the `rustc` 1.94.1 that compiles it, 17.0 (`char::UNICODE_VERSION`). Measured in
  this seal scalar by scalar, the 1,112,064, against what that `rustc` prints: 1,097,084 come out
  the same and 14,980 do not, all escaped here and in the clear in the kit; 14,979 are Cn in 14.0
  —assigned later— and one, U+1171E, is Mn in 14.0 and Mc in 16.0. None comes out in the clear here
  and escaped in the kit, and none is changed by the extension table of 16.0 against that of 17.0.
  So a name with U+1FAE9, which is among the 14,979, comes out escaped here and in the clear in the
  kit. It changes the letter of the cause, not the verdict: `debug_rust` only writes the text of a
  rejection already decided (`tools/segunda/medio.py:451-452` and
  `tools/segunda/verificador.py:956-957`). An exact `{:?}` would require the kit's Unicode tables
  inside the second. ⚠️ §852: it said `verificador.py:955-956`; the `import re` of §852 moves those
  two down one line.
- The AIRs as a second opinion: `airs.py` transcribes them from the `.rs`, the only source that
  exists. What the STARK judge measures is the machinery of the verifier, not whether an AIR
  constrains what it says it constrains.
- The whole command of the forms with STARK in the second code: `juez_stark.py` judges the pair
  (proof and statement) that the binary composes, but does not walk the head, the signature and the
  fields of those envelopes as `verificador.py` walks those of the five forms without STARK.
- The 34 preserved vectors of `spec/vectors/0.3/`: unhidden and unsalted, with the AIRs of
  `0eda58c`; today's kit does not verify them either (it rejects their form, D-AD).
- That the `.wasm` runs in a browser: `wasm32-unknown-unknown` is stopped by `getrandom`, which
  `xmss` drags in through `rand 0.10` to generate keys the verifier does not use (measured with
  `cargo tree -i getrandom@0.4.3`). It is one more line for the issue to RustCrypto of entry 77. ⚠️
  §793: the line has been in the draft since §793, `doc/issue-rustcrypto.md`, part 5, not sent. And
  it is not the verifier's only `getrandom`:
  `cargo tree -i getrandom --locked --offline -p zk-ssl-verify -e normal` refuses as ambiguous,
  because there are two, and the other one, `getrandom 0.2.17`, arrives through `rand_core 0.6.4` of
  `zk-ssl-air`, the salt of `MerkleConSal` (§535, D-X): it is the house's, and the issue does not
  fix it. That both stop the compilation is what their source says, read in the cargo registry and
  not compiled: a `compile_error!` for `wasm*-unknown-unknown` without the `wasm_js` feature
  (`getrandom` 0.4.3, `src/backends.rs:170-182`) or without `js` (0.2.17, `src/lib.rs:344-349`).
- The whole witness in the second code: `cable.py` judges ONE response with fresh memory, like
  `witness --respuesta`. The key pinned from one round to the next, the second channel (the
  consistency, with `zkssl_consistencyProof`), the co-signatures and the journal have no second
  implementation.
- Performance: the `.wasm` is not optimized for size nor has it been passed through `wasm-opt`.

## Provenance

Written in a session with generative AI assistance, on the branch `claude/nice-planck-ax35zl`, and
integrable in the author's name like the ECST block. «Integrar no es aceptar» [integrating is not
accepting]: the acceptance that [`GENAI.md`](../../GENAI.md) describes remains pending.
