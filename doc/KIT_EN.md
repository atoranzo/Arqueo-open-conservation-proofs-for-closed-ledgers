# The verifier kit — checking offline, without the repository, without trusting anyone

This is the script for a single download, `arqueo-verify-<version>-<host>.tar.gz`, with which a
third party — an inspector, an auditor, an applicant — checks by themselves, on their own machine
and offline, four things in this order: that a file adds up · that a tampered file does **not**
add up and the program **names the broken rule** · that the same label published in **two distinct
ledgers** is detected from the two signed heads with both nodes off · and that a swap of ledgers is
rejected by name. What travels in the tarball, and why, is in `spec/PAQUETE.md`, section 11, which
travels inside. None of the four checks needs the repository, the author, a node or a connection.

It is a CLI and not a web page on purpose: the answer to "how do I know that program does what it
says?" is "download it and run it yourself", and that only holds if the download exists and can be
reproduced.

## 0. Download, and check the download before believing it

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
and `tools/banco_apagado.sh`; they do not travel in the kit because they spawn processes, and the
vectors above are their captures.
