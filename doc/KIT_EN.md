# The verifier kit — checking offline, without the repository, without trusting anyone

This is the script for a single download, `arqueo-verify-<version>-<host>.tar.gz`, with which a
third party — an inspector, an auditor, an applicant — checks by themselves, on their own machine
and offline, four things in this order: that a file adds up · that a tampered file does **not**
add up and the program **names the broken rule** · that the same label published in **two distinct
ledgers** is detected from the two signed heads with both nodes off · and that a swap of ledgers is
rejected by name. Since 0.3.0, a fifth: that what the node **received** under its signature was
resolved within its window, or the program names the operator that did not resolve it. What travels
in the tarball, and why, is in `spec/PAQUETE.md`, section 11, which
travels inside. None of the five checks needs the repository, the author, a node or a connection.

It is a CLI and not a web page on purpose: the answer to "how do I know that program does what it
says?" is "download it and run it yourself", and that only holds if the download exists and can be
reproduced.

## 0. Download, and check the download before believing it

The current release is `arqueo-verify-v0.4.2`, published at
<https://github.com/atoranzo/Arqueo-open-conservation-proofs-for-closed-ledgers/releases/tag/arqueo-verify-v0.4.2>,
produced on commit `a4e888d8b98ca6254904486f6ada8710c2a5fe08`:
tarball `arqueo-verify-0.4.2-x86_64-unknown-linux-gnu.tar.gz`, sha256
`33acdc9a8fe38f0355b4e49e11338a9a94b6f933d86eb5bde23ead315ae1831d` (16,677,910 bytes, 368 files);
binary `7734376463d120b0bbd47d3bde852da628557caf28b530cbe5076fce17a68ee2`. Both hashes are
measured from the outside — `curl` download, the asset's `digest` in the API, the downloaded kit
running its eleven catalogues from inside the tarball — in the `AUDITORIA.md` entry that registers
this release (§671). It recomposes heads up to **v6**, the one the node signs today, and reads
eleven families: the ten of section 11 of `spec/PAQUETE.md` and the co-signed anchor (§633, §634).

**0.4.1 is still published and is no longer current** (`arqueo-verify-v0.4.1`, commit `ff1f06f`,
tarball `505de994f243bc76…`). It has three defects that 0.4.2 fixes: it reads a `u64` of the
envelope with a `+` or leading zeros (§662); it counts the same co-signature repeated as several
witnesses in a v2 package (§663); and it reads an XMSS key with the alternate OID `0x00010005` as the
published one, so it gives GREEN to a "conflict between two ledgers" with a single ledger written
twice (§664). Use 0.4.2.

**0.4.0 is still published and is no longer current** (`arqueo-verify-v0.4.0`, commit `65d1e89`,
tarball `90184e41299e737a…`). It has one defect that 0.4.1 fixes: a proof with extra nodes inside a
Merkle batch verifies anyway, with different bytes (§659). The verdict on the statement does not
change — the altered proof proves the same as the original — but its bytes no longer identify the
operation. Use 0.4.2. Both read the same eleven families and give the same verdicts to every
published vector.

**Release 0.3.0 is still published and is no longer the current one** (`arqueo-verify-v0.3.0`,
commit `65cabfb`, tarball `06648502e0171fea…`). It has four defects that 0.4.0 fixes, described in
section 3.9 of `SECURITY.md`: a hex field with a multibyte character, or a hidden proof with its meta
emptied, make it exit with **101** (panic) instead of RED (§650, §651); a proof with trailing bytes
is read as if they were not there (§653); and it accepts the four negatives of the canonical reading
(RFC-0016, §640). It fails with the wrong code, never with a false GREEN for those cases; still,
use 0.4.2.

⚠️ **In 0.3.0, an outdated sentence travels in its `NOTICE`, declared; 0.4.0 carries the corrected one.** That tarball's `NOTICE` says the
witness hiding in the winterfell fork is switched on only by `Prover::ocultacion`, "which no ARQUEO
prover returns", and that the fork touches "eight files and two new ones". That was the description
of §534; since §538 every ARQUEO prover switches it on, and since §575 the fork touches eleven files
and three new ones. It does not change what the verifier checks — it verifies proofs with hiding
on, and its catalogues measure it — but it is a false description of the fork. A published tarball
is not corrected: the tree's `NOTICE` was corrected in §623, and the next release carries it.

**Release 0.2.0 is still published and is no longer the current one** (`arqueo-verify-v0.2.0`,
commit `1528943`, tarball `2fe9030a310a1e0b…`). It recomposes heads up to v4: with the vectors in
its own tarball it works, and the envelopes born later — `edad`, `prenda`, `cobro_pendiente`,
`completitud` and the `ancla`, with v5 and v6 heads — it rejects with exit 1 ("tipo desconocido",
unknown type). It fails closed: it never gives a false GREEN (measured on 2026-09-30; `BACKLOG.md`,
entry 107).

Every release carries a tag and is produced on the commit its `VERSION` file names; the tarball's
hash is published **next to its commit**, on the release page and in the `AUDITORIA.md` entry that
sealed it, never as a bare number. With the tarball in hand:

```bash
sha256sum arqueo-verify-*.tar.gz          # must be the hash published next to the commit
tar xzf arqueo-verify-*.tar.gz && cd arqueo-verify-*/
sha256sum -c SHA256SUMS                   # every file inside, against its hash
cat VERSION                               # commit, describe, toolchain, glibc_max
```

Requirements: Linux x86_64 and a glibc equal to or newer than the one `VERSION` declares in
`glibc_max`. The binary is not static, and it says so. The command contract is one line:
`./zk-ssl-verify <file.json>`; exit 0 and `VERDE: …` if the file holds, exit 1 and the **first**
failure by name (`ROJO: …`) if not, exit 2 on wrong usage. The texts are in Spanish; the rule
names are what the conformance manifests check.

## 1. A file that adds up

```bash
./zk-ssl-verify spec/vectors/paquete/posicion-v2.json
```
Expected: `VERDE: el paquete se sostiene sin el nodo` (the package holds without the node), exit 0.
The file is a real position captured from a node that was then shut down: the signed epoch head,
the acknowledgement with its path, and the co-signatures. The verifier recomputes the `epochDigest`
from the fields, verifies the signature, climbs the path to the root and asks nobody anything.

```bash
./zk-ssl-verify spec/vectors/consumo/consumo.json
```
Expected: `VERDE: el consumo se publico entre las dos cabezas, sin el nodo` (the consumption was
published between the two heads). This is the consumption envelope: two signed heads of the same
ledger, the label absent under the old one and present under the new one. Within one ledger a
unit is consumed once, and the fact is published in the head.

## 2. A tampered file does not add up, and the program names the broken rule

```bash
./zk-ssl-verify spec/vectors/paquete/rechazo-n-adulterado.json; echo "exit $?"
```
Expected: exit 1 and `los siete campos NO recomponen el epochDigest empaquetado` (the seven fields
do not recompose the packaged digest). A field of the head was altered after signing; the digest no
longer matches and the program says which rule, not merely that it failed.

```bash
./zk-ssl-verify spec/vectors/consumo/rechazo-cons-ausencia-ya-estaba.json; echo "exit $?"
```
Expected: exit 1 and `el consumo YA estaba` (the consumption was already there): the same label
presented as new when the old head already carried it. That is double use within one ledger, named.

And the whole catalogue, with the harness that travels inside: every entry of every manifest states
the exit code and the text the binary must print, and the harness checks them entry by entry:

```bash
bash conformidad.sh ./zk-ssl-verify                                        # the package
bash conformidad.sh ./zk-ssl-verify spec/vectors/consumo/MANIFIESTO.txt    # the consumption
bash conformidad.sh ./zk-ssl-verify spec/vectors/conflicto/MANIFIESTO.txt  # the conflict
```
Expected, at the end of each: `conformidad: N de N entradas dicen lo que deben` (N of N entries say
what they must), with the same N on both sides. The negative vectors were derived by mutation of
real captures and are never rewritten; if one said otherwise, the harness names it.

## 3. The same label in two distinct ledgers, with both nodes off

```bash
./zk-ssl-verify spec/vectors/conflicto/conflicto.json
```
Expected: `VERDE: dos libros aceptaron el mismo consumo. Es DETECCION, no prevencion:` (two ledgers
accepted the same consumption; this is detection, not prevention) and, below it, what each ledger
signed. The envelope carries **two heads signed by two different keys** — two ledgers — and, for
each, the path climbing from the same label to the consumption root that head signs. No node takes
part: the capture was made with both nodes alive and is checked with both off. It is exactly the
fact a body needs to see when the same invoice is certified twice at two counters.

## 4. A swap of ledgers is rejected by name

```bash
./zk-ssl-verify spec/vectors/conflicto/rechazo-conf-camino-no-sube.json; echo "exit $?"
./zk-ssl-verify spec/vectors/conflicto/rechazo-conf-misma-clave.json; echo "exit $?"
```
Expected: exit 1 for both. The first is a raw capture with the two ledgers' paths **swapped**:
`libro[0]: el camino NO sube al consRoot de su cabeza` (the path does not climb to its head's
consumption root). The second presents the same ledger twice: `las cabezas llevan la MISMA clave`
(the heads carry the same key). A conflict envelope demands two real ledgers, and says which of the
two rules broke.

## 5. A receipt under the operator's signature, and what it did with it (since 0.3.0)

```bash
./zk-ssl-verify spec/vectors/completitud/resuelta-por-acuse.json; echo "exit $?"
./zk-ssl-verify spec/vectors/completitud/no-resuelta.json; echo "exit $?"
```
Expected: the first, exit 0 and `VERDE: el recibo se resolvio dentro de la ventana, y se sostiene
sin el nodo`, with `RESUELTA como transicion aplicada` in its third step; the second, exit 1 and `NO
RESUELTA EN LA VENTANA`. This is the completeness envelope (`spec/PAQUETE.md` 2.11, RFC-0010): the
receipt the node issued on RECEIVING an operation sits under a root its signed head commits to, and
the envelope says, with no node, whether it was resolved within its window — applied, or rejected
with proof — or names the operator that did not. The second is exactly that: the promise signed by
the accused, one of its heads past the window, and no resolution shown. It is not a cryptographic
proof of absence but opposable evidence: the operator refutes it by showing the resolution.

```bash
./zk-ssl-verify spec/vectors/completitud/lote-aplicado.json; echo "exit $?"
./zk-ssl-verify spec/vectors/completitud/prenda-sin-fundamento.json; echo "exit $?"
```
Expected: exit 0 and `RESUELTA como LOTE aplicado` — a batch of two sends with real proofs that an
aggregator submitted, with the acknowledgement of each proof —; and exit 1 and `RECHAZO SIN
FUNDAMENTO`: the envelope of a pledge that the verifier judges AGAIN, with the same judge and
against the head the node judged, and that verifies, against a node that said no (RFC-0014). That
vector was derived by mutation of a real capture — an honest node does not produce it — and its
manifest says so.

```bash
bash conformidad.sh ./zk-ssl-verify spec/vectors/completitud/MANIFIESTO.txt
```
Expected: `conformidad: N de N entradas dicen lo que deben`. Release 0.2.0 does not know this
envelope and rejects the four files with `tipo desconocido`: it fails closed.

## What this says, and what it does not

- **It detects; it does not prevent.** Two sovereign ledgers can accept the same label; nobody
  orders between them. What the kit shows is that a third party holding both signed heads SEES it,
  afterwards.
- **The window is not clock time.** When a node loads another ledger's signed head so as not to
  process what that ledger already holds, what applies is a **boundary in the signed `seq`** of that
  head, with two faces: it blocks nothing that ledger signed after that head, and it does not know
  whether that head has already been superseded.
- **The label is governance.** For two bodies to detect the same invoice, both must compute the
  label the same way, and that is an agreement between them, not a property of the code.
- **The oracle limit.** The proofs speak of the ledger, not of the world: an invoice is born
  outside; what is proved is that the same label was published, not that it is the same real
  invoice.
- **Within one ledger, single use is an invariant**; between ledgers, what exists is the above.
  The whole design, with its decisions and what it discards, is in RFC-0006
  (`spec/rfc/0006-consumo-publicado.md`) and in `SECURITY.md`.

## Reproducing the kit, for whoever does not trust the download

The tarball is produced on the commit `VERSION` names: with the `rustc` `VERSION` declares,
`git checkout <commit>` and `bash tools/artefacto.sh` yield the same binary (same hash, built with
`--remap-path-prefix`) and the same tarball, and `tools/canon.sh` checks that property at every
seal. The demonstrations with live nodes — bringing up two ledgers, publishing the same label in
both, capturing the envelope — are the benches `tools/banco_dos_libros.sh`, `tools/banco_consumo.sh`
and `tools/banco_apagado.sh`, and those of step 5, `tools/banco_completitud.sh`,
`tools/banco_mentiroso_sin_resolver.sh` and `tools/banco_recibo_agregado.sh`; they do not travel in
the kit because they spawn processes, and the vectors above are their captures.
