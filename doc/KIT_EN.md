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
rebuilt from its commit; with the same hash, that is only shown within one same machine (below, «Reproducing the kit, for whoever does not trust the download»). ⚠️ §823: it said «and can be reproduced», without the machine condition.

## 0. Download, and check the download before believing it

The current release is `arqueo-verify-v0.4.3`, published at
<https://github.com/atoranzo/Arqueo-open-conservation-proofs-for-closed-ledgers/releases/tag/arqueo-verify-v0.4.3>,
produced on commit `5fd5212b9f6e04c4c619655ffe969ab04116477a`:
tarball `arqueo-verify-0.4.3-x86_64-unknown-linux-gnu.tar.gz`, sha256
`0bb21f4f5dfe7575264268f281c19648cbf3da38b4fd6809c4eea067592aa4b0` (21,737,865 bytes, 420 files);
binary `1b120e597897e00530508506b2ade934b042e3fc3f9c1138d895de25d37c8844`. Both hashes are
measured from the outside — `curl` download, the asset's `digest` in the API, the downloaded kit
running its twelve catalogues from inside the tarball — in the `AUDITORIA.md` entry that registers
this release (§818). It recomposes heads up to **v6**, the one the node signs today, and reads
twelve families: the twelve of section 11 of the `spec/PAQUETE.md` that travels in its tarball, the twelfth being rotation (§649), and it is the first published release that carries it. ⚠️ **Corrected in §807**: it said «the ten of section 11 of `spec/PAQUETE.md` and the co-signed anchor»; section 11 at commit `a4e888d` names eleven families, the co-signed anchor among them, and the tree's names twelve (`FAMILIAS` in `tools/artefacto.sh`:31). ⚠️ **§818**: until here this paragraph gave `arqueo-verify-v0.4.2`, published at <https://github.com/atoranzo/Arqueo-open-conservation-proofs-for-closed-ledgers/releases/tag/arqueo-verify-v0.4.2>, on commit `a4e888d`, as the current release, with its two hashes —abridged in its paragraph, below; in full, with its 16,677,910 bytes and its 368 files, in entry §671—, and said «reads eleven families: the eleven of section 11 of the `spec/PAQUETE.md` that travels in its tarball, the eleventh being the co-signed anchor (§633, §634); the tree's section 11 already counts twelve, with rotation (§649), which no published release carries», which was true of 0.4.2.

⚠️ ~~**0.4.3 is CUT and not published (§815), and it is a security fix.** The current release remains
0.4.2 until the entry that registers 0.4.3 pins its fingerprint.~~ ⚠️ **§818: published and
current.** The author published it with its security advisory, `GHSA-6p4m-qg2w-8r6j` (<https://github.com/atoranzo/Arqueo-open-conservation-proofs-for-closed-ledgers/security/advisories/GHSA-6p4m-qg2w-8r6j>), which
names 0.3.0, 0.4.0, 0.4.1 and 0.4.2 as affected and 0.4.3 as fixed, and §818 registers it with its
fingerprint, above. **It is a security fix.** It was drafted in private
(`spec/rfc/PROCESO.md` §«Fallos de seguridad») and it changes verdicts within `0.4.x`, which rule
R6 f of RFC-0021 allows with this notice and the list of what changes. **The defect it fixes:**
0.4.2 accepts no resolution of a reception receipt that the node rejected with
`VerificationFailed` or `ProofFailed`. No command proves its rejection envelope (2.6 of
`spec/PAQUETE.md`), and the `declarada` one comes out RED, «no es de las que el RFC-0007 dejo sin
prueba portable», although D-I of RFC-0007 declares them that way and D-G of RFC-0010 sends them to
the fourth state. Measured in private before the fix: a third party **with no account** sends a
garbage proof with the public roots, the node rejects it with `VerificationFailed` and hands it its
receipt, and once the window —1,440 heads— is over the completeness envelope says «NO RESUELTA EN
LA VENTANA», a RED that names an honest operator. Every release that reads that envelope has it:
0.3.0, 0.4.0, 0.4.1 and 0.4.2 carry the same list of four causes (read at their commits,
`65cabfb`, `65d1e89`, `ff1f06f` and `a4e888d`; measured with 0.4.2), and 0.2.0 does not read the
envelope. 0.4.3 declares them: exit 3, «DECLARADA, NO PROBADA», on the direct path and in a batch;
their rejection envelope is still unproven, because that the proof did not verify is the node's
word until the kit re-verifies it (RFC-0019 E3, the 0.5 train). That rejection envelope still comes
out RED, «la causa … no la prueba este mando», and that RED says the envelope does not resolve the
receipt, not that the operator is to blame: the resolution an honest one shows is `declarada`. And
no receipt-bearing path of the node produces `ProofFailed` today —the layer only gives it when
proving; read in the code—, so a `declarada` with it is always the operator's word; it is in the
list because D-I declares it (`SECURITY.md` §3.15; entry 129 of `BACKLOG.md`). **What changes,
measured** with
the published 0.4.2 (binary `7734376463d120b0…`) and the tree's 0.4.3 over the 500 vectors of
`spec/vectors/`, exit code and last line of each:

- of the 348 vectors that travel in the 0.4.2 tarball, 346 are still at their path with the same
  bytes and the same code (of the 437 in the tree at its commit, 435); four change their text with
  the same 1 —the window is named with the index the signature attests and its start (§681,
  RFC-0019 E1)—; and the other two, GREEN under 0.4.2, give 3 and are kept, with their verdict of
  then, under `spec/vectors/0.4/completitud/` (§682, RFC-0019 E2: a `StaleState` does not resolve on
  its own);
- of the 65 vectors the tree gained since `a4e888d`, fourteen give another code under 0.4.2: those
  two, at their new path, from 0 to 3; the three of this fix, from 1 to 3
  (`completitud/declarada-verification-failed.json`, `declarada-proof-failed.json`,
  `lote-declarada-verification-failed.json`); the two §682 twins, from 0 to 3; the co-signature with
  a version that does not fit in a byte, from 0 to 1 (§780,
  `paquete/rechazo-cofirma-versionformato-0x103.json`); and six of the rotation family, from 1 to 0,
  because 0.4.3 reads `actas` and the `solapamiento` envelope, which 0.4.2 does not know (§648,
  §686: additive). And `completitud/neg-vigente-index-inflado.json` stays at 1, but under 0.4.2 it is
  «NO RESUELTA EN LA VENTANA» —a RED against the operator that anyone could forge by inflating the
  declared `index`— and under 0.4.3 «ventana ABIERTA» (§681).

The records of the measurement and the full list, vector by vector, go in entry §815.

**0.4.2 is still published and is no longer current** (`arqueo-verify-v0.4.2`, commit `a4e888d`,
tarball `33acdc9a8fe38f03…`, binary `7734376463d120b0…`). It has the defect of the notice above,
which 0.4.3 fixes (§815): it accepts no resolution of a reception receipt that the node rejected
with `VerificationFailed` or `ProofFailed` —the `declarada` one comes out RED and no command proves
the rejection envelope—, so, if anyone, with no account, sends the node a garbage proof, the honest
operator has nothing to resolve that receipt with and, once the window is over, is named, «NO
RESUELTA EN LA VENTANA». And the list of what changes, above, measures others that 0.4.3 fixes: it
gives GREEN to a `StaleState` that does not resolve (§682) and to a co-signature whose version does
not fit in a byte (§780), and gives «NO RESUELTA EN LA VENTANA», a RED against the operator, to an
envelope anyone can forge by inflating the declared `index` of the current head (§681). Use 0.4.3.

**0.4.1 is still published and is no longer current** (`arqueo-verify-v0.4.1`, commit `ff1f06f`,
tarball `505de994f243bc76…`). It has three defects that 0.4.2 fixes: it reads a `u64` of the
envelope with a `+` or leading zeros (§662); it counts the same co-signature repeated as several
witnesses in a v2 package (§663); and it reads an XMSS key with the alternate OID `0x00010005` as the
published one, so it gives GREEN to a "conflict between two ledgers" with a single ledger written
twice (§664). ~~Use 0.4.2.~~ Use 0.4.3 (§818).

**0.4.0 is still published and is no longer current** (`arqueo-verify-v0.4.0`, commit `65d1e89`,
tarball `90184e41299e737a…`). It has one defect that 0.4.1 fixes: a proof with extra nodes inside a
Merkle batch verifies anyway, with different bytes (§659). The verdict on the statement does not
change — the altered proof proves the same as the original — but its bytes no longer identify the
operation. ~~Use 0.4.2.~~ Use 0.4.3 (§818). Both read the same eleven families and give the same verdicts to every
published vector. ⚠️ **§818**: «both» are 0.4.0 and 0.4.1; 0.4.3 reads twelve and changes the
verdicts the notice, above, lists.

**Release 0.3.0 is still published and is no longer the current one** (`arqueo-verify-v0.3.0`,
commit `65cabfb`, tarball `06648502e0171fea…`). It has four defects that 0.4.0 fixes, described in
section 3.9 of `SECURITY.md`: a hex field with a multibyte character, or a hidden proof with its meta
emptied, make it exit with **101** (panic) instead of RED (§650, §651); a proof with trailing bytes
is read as if they were not there (§653); and it accepts the four negatives of the canonical reading
(RFC-0016, §640). It fails with the wrong code, never with a false GREEN for those cases; still,
use ~~0.4.2~~ 0.4.3 (§818).

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
`./zk-ssl-verify <file.json>`; exit `0` and `VERDE: …` if the file holds, exit `1` and the
**first** failure by name (`ROJO: …`) if not, exit `2` on wrong usage, and exit `3` in the fourth
state of the completeness envelope, «resolution declared, not proven», which is written by its name
and without `ROJO`, because it is not one. The texts are in Spanish; the rule names are what the
conformance manifests check. ⚠️ §801: until here it said «exit 0 … exit 1 … exit 2 on wrong usage»,
without the 3, which the command gives since §573 (`spec/PAQUETE.md` §«6. El contrato del mando»;
RFC-0021, fact 6).

What of this is promised —what does not change within a `0.N.x` line, what breaks it and what
keeps verifying after a break— is said in `spec/RPC.md` §«Versionado y compatibilidad» (§801): no
published kit is withdrawn, each one is rebuilt from its commit, and what it judged it keeps
judging the same way; a security fix may change a verdict within `0.N.x`, with its notice and the
list of what it changes, as 0.4.2 did. ⚠️ §823: it said «each one is reproducible from its commit»,
without the machine condition: with the same hash this is only shown on one same machine, and 0.4.2
rebuilt from its commit on another one comes out with another hash (below, «Reproducing the kit, for
whoever does not trust the download»). What it judged it does judge the same way: the rebuilt one
passes its eleven catalogues.

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
`--remap-path-prefix`) and the same tarball **on the same machine**; the `--check` of
`tools/artefacto.sh` that the canon runs at every seal, in `b6994c3`, builds the same tree twice, in
two `target` directories, and looks in the binary only for the `$HOME` paths. ⚠️ §823: it said «yield
the same binary (same hash, built with `--remap-path-prefix`) and the same tarball, and
`tools/canon.sh` checks that property at every seal», without the machine condition. What was
measured on 2026-10-15 (§823), except the 41 registry paths of (ii), compared on 2026-10-21:

- **On one same machine, yes.** The node and the witness of `b6994c3`, built with the flags of
  `tools/artefacto.sh` from two trees, in another `target` and with another `HOME` and another
  `CARGO_HOME`, come out byte-identical all four times (`rustc` 1.94.1): the node
  `ac27f9052c7f74c1…`, with `dev` and built together with the witness
  (`-p zk-ssl-node -p zk-ssl-cli`), and the witness `07d659c6291ad68f…`. Without the remap, two
  builds from two trees also come out equal to each other, but with another hash and with the
  `$HOME` paths inside: what the remap removes is the dependency on `HOME`, with `CARGO_HOME` and
  `RUSTUP_HOME` under it (outside it, their paths would enter whole, deduced from the rules and not
  measured: below, (i) and (ii)); the one on the tree path does not show up, because cargo hands
  `rustc` the paths of the workspace crates relative to its root.
- **From another machine, 0.4.2 does not come out.** Rebuilt from `a4e888d` with its `rustc`
  1.97.1, the binary is `14832a067f6dc4cf…` and not `7734376463d120b0…`: 13 of its 35 sections
  differ. With `rust-src` installed, a toolchain named `stable` and `RUSTUP_HOME` under `HOME`, as the
  paths inside the published one say, 3 remain (329 bytes): the `build-id`, 240 bytes of `.text`, all
  in blake3's assembly functions, and `.comment`, which names GCC 15.2.0 in the published one and
  13.3.0 here. 0.4.3, the current release since §818, is not on record as rebuilt with its `rustc`
  1.97.1 outside the machine that produced it; built here from its code with 1.94.1, as the canon of
  §817 built it, it comes out with another hash. Its published binary, `1b120e597897e005…`, carries
  what set 0.4.2 apart from its rebuild: the same eleven `rust-src` paths and a `.comment` that
  names GCC 15.2.0 (§823, read with `strings` and `readelf`, without running it).
- **What the hash depends on, besides `rustc` and `Cargo.lock`.** (i) If `rust-src` is installed, the
  standard library's paths enter the binary with its toolchain's name, because no rule of `b6994c3`
  takes them to `/rustc/<commit-hash>`: with `RUSTUP_HOME` under `HOME`, the `$HOME` one only changes
  their prefix, and with `RUSTUP_HOME` outside `HOME` they would enter whole (deduced); 0.4.2 carries
  eleven `/home/x/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/…`. One
  more remap, from `<sysroot>/lib/rustlib/src/rust` to `/rustc/<commit-hash>`, neutralises it: with
  it, 0.4.2 built with `rust-src` comes out identical to the one built without it. (ii) With the
  flags of `b6994c3`, the registry rule never wins: `rustc` applies the last matching rule, the
  `$HOME` one goes last, and with `CARGO_HOME=$HOME/.cargo` the registry paths enter as
  `/home/x/.cargo/registry/src/…`. That is not a cause of what was measured: with
  `CARGO_HOME=$HOME/.cargo` on both machines it gives the same bytes, and the published one and the
  rebuilt one carry the same 41 `/home/x/.cargo/registry/src/…` paths. It is a dependency the
  measurement does not touch: the script of `b6994c3` derives three of its rules from `$HOME`, not
  from `CARGO_HOME`, and with a `CARGO_HOME` outside `HOME` none would match, and its paths would
  enter whole (deduced, not measured). (iii) The native code the host compiles —blake3's assembly in
  the kit, the node and the witness, and ring's C and assembly in the witness— depends on its `cc`
  and its `as`, which the `VERSION` of 0.4.2 does not name. That with the same `cc` and the same `as`
  0.4.2 would come out identical is not measured: there is no machine here with theirs. The causes of
  the 13 sections are two, (i) and (iii); (ii) did not weigh.

The demonstrations with live nodes — bringing up two ledgers, publishing the same label in both,
capturing the envelope — are the benches `tools/banco_dos_libros.sh`, `tools/banco_consumo.sh` and
`tools/banco_apagado.sh`, and those of step 5, `tools/banco_completitud.sh`,
`tools/banco_mentiroso_sin_resolver.sh` and `tools/banco_recibo_agregado.sh`; they do not travel in
the kit because they spawn processes, and the vectors above are their captures.
