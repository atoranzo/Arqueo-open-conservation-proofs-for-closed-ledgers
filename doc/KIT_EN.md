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
  measured: below, (i) and (ii); ⚠️ §824: measured later on the kit, and they enter: below, the paragraph «Since §824, the tool»); the one on the tree path does not show up, because cargo hands
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
  their prefix, and with `RUSTUP_HOME` outside `HOME` they would enter whole (deduced; ⚠️ §824: measured later on the kit, eleven enter whole: below, the paragraph «Since §824, the tool»); 0.4.2 carries
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
  enter whole (deduced, not measured; ⚠️ §824: measured later on the kit, 41 enter whole: below, the paragraph «Since §824, the tool»). (iii) The native code the host compiles —blake3's assembly in
  the kit, the node and the witness, and ring's C and assembly in the witness— depends on its `cc`
  and its `as`, which the `VERSION` of 0.4.2 does not name. That with the same `cc` and the same `as`
  0.4.2 would come out identical is not measured: there is no machine here with theirs. The causes of
  the 13 sections are two, (i) and (iii); (ii) did not weigh.

**Since §824, the tool.** `tools/artefacto.sh` takes the remap rules from a single producer,
`reglas()`, which the kit, the node and the witness read: five, derived from `CARGO_HOME` and from
the sysroot `rustc` reports, and ordered from the shortest to the longest, because `rustc` applies
the last one that matches and so the most specific one wins. The registry enters as `/registry/…`,
and `rust-src`, if installed, as `/rustc/<commit-hash>/…`, what `rustc` writes when it is not.
`cargo`, `rustc` and `cc` run under `env -i` with only the variables needed to find the toolchain,
the registry and the network (`PASAN`, in the tool; `VERSION` names them): `CC`, `CFLAGS`,
`CC_<target>`, `RUSTC_WRAPPER`, `CARGO_PROFILE_RELEASE_*`, `CARGO_ENCODED_RUSTFLAGS` and the others
that would change the binary without `VERSION` saying so do not get through, and a cargo
configuration file carrying `[profile]`, `[env]`, `[target]`, `[host]`, `[build]`, `[source]`,
`[patch]` or `[unstable]` is ROJO. Each `target` carries the stamp of the host it was built with
—the tool, `rustc`, `cargo`, `cc`, `cc1`, `as`, the glibc and the startup objects—, and if it is not
the current one it is emptied before building, because `cargo` does not rebuild when the host's `cc`
changes nor when a variable a build.rs read without declaring it changes. `VERSION` also names the
`cc`, the `as` that `cc` calls, the host's glibc and the hash of the objects `cc` always links. The
`--check` builds the kit in two roots —the tree and a copy of what git tracks or does not ignore, in
`target/artefacto/raiz2`—, checks that each binary came from its root's sources (the dep-info cargo
leaves next to it names them), requires the same hash, and looks in both binaries for the paths of
the two roots, of `HOME`, `CARGO_HOME`, `RUSTUP_HOME` and, of the sysroot, `<sysroot>/lib/rustlib`,
whole, without requiring the slash after them —a bare `$HOME` inside the binary belongs to the
machine as much as `$HOME/something`, and it is the same in both roots—, and for `/home/x/`, which
only shows up if a general rule beats a specific one. If `strings` fails or yields nothing, or
`grep` exits with an error, it is ROJO, not «no paths». If the hashes differ, it says which ELF
sections differ. Measured with `rustc` 1.94.1, on the sources of `b6994c3` (§812), on which this
seal was written and reviewed, and with three versions of the tool —the first, on 2026-10-15, before
the four fixes of its first review; the second, on 2026-10-20 and 2026-10-21, without the clean
environment or the host stamp; and the one being sealed, on 2026-10-21—, with the same hashes in all
three; what was measured again on the tree of §817, in the next-to-last item, and on this seal's, in
the last:

- **The tree's kit comes out the same from two roots**, `326d232b81250581…`, and the same with
  `rust-src` installed in another `RUSTUP_HOME`, with `CARGO_HOME` outside `HOME`, and with both and
  another `HOME` at once (the latter, also with the one being sealed). With the tool of `b6994c3`,
  building in an empty `target`, `rust-src` took it to `a5c68056442c3d83…`, with eleven sysroot
  paths inside, and a `CARGO_HOME` outside `HOME` to `a0367207f77c57cf…`, with 41 of its paths. The
  search of its `--check` (`strings -n 8 | grep -c "$HOME"`, on its line 96), applied to those two
  binaries, gives 0: it would not have seen them; its whole `--check` was not run that way. With the
  `target` already built, `cargo` does not rebuild when only the sysroot changes, and with
  `rust-src` the usual `311df1aa…` came out.
- **The environment no longer gets in.** With the second version, `CC=clang` gave another kit,
  `7cb169729272c13a…`, and `VERSION` still named gcc (its review measured it); with a mutant of
  the one being sealed without the `env -i` of `construir()`, the same, and also with only
  `CC_x86_64-unknown-linux-gnu=clang`, which cc-rs reads and bash cannot unset. With the one being
  sealed, and ten such variables at once —`CC`, `CFLAGS`, the dashed one, `TARGET_CC`,
  `RUSTC_WRAPPER`, `RUSTC_BOOTSTRAP`, `CARGO_PROFILE_RELEASE_OPT_LEVEL`, `CARGO_INCREMENTAL`,
  `CARGO_FEATURE_PURE` and `CARGO_ENCODED_RUSTFLAGS`—, `326d232b81250581…` comes out, and the same
  tarball as without them. The clean environment alone was not enough: on a `target` that mutant had
  built with the dashed variable, the one being sealed without the host stamp gave the clang kit,
  `7cb169729272c13a…`, with `VERSION` naming gcc, because cc-rs does not declare that variable and
  `cargo` did not rebuild; with the stamp, the `target` is emptied and `326d232b81250581…` comes
  out. A `cc` that is another binary —a wrapper calling the same gcc— empties it too, and the same
  hash comes out.
- **The tree's kit hash moves with this seal.** With the sources of `b6994c3`, the binary goes from
  `311df1aa4a70f4cb…` to `326d232b81250581…`; with those of the tree of §817, from
  `a0ae4bfdddbaa207…` to `cc2077db8a608fe2…`, and with those of this seal's, from
  `3923dcc03198efb7…` to `2d94c323542bbe2d…` (below), because the registry paths go from
  `/home/x/.cargo/registry/src/…` to `/registry/…`. The tarball moves as well because of the
  `VERSION` lines the tool writes —the new ones, the `cc`, the `as`, the host's glibc, the startup
  objects, cargo's environment and how the rules are ordered, and the `rustflags` one, which goes
  from four rules to five, with other names—, besides what changes with every commit: `commit=`,
  `describe=` and, in this seal, `spec/PAQUETE.md`, which travels inside. A published kit is rebuilt
  with the `tools/artefacto.sh` of its commit, not with this tree's (with the same hash, only within
  one same machine): the flags are part of what is built. 0.4.3, published (§818), was produced on
  `5fd5212` (§817) with that commit's `tools/artefacto.sh`, which is §815's, and not with this seal's:
  its `VERSION` does not carry the new lines, it is rebuilt with that one, and its hash is the one in
  section 0.
- **The cost of the `--check`**, on four cores, with the one-minute load average in brackets: with
  the first version, 164 s cold, 41 s with no changes and 48 s with a change in
  `crates/zk-ssl-verify` (2.6 to 8.8); with the second, 66, 23 and 29 s (2.8 to 3.8); with the one
  being sealed, 62 to 65 s cold —and the first pass after changing the tool or the host is cold— and
  24 s with no changes (2.3 to 3.9). That of `b6994c3`, 156 s cold and 51 s with no
  changes (6.4 to 9.1).
- **On the tree of §817** —the one from before rebasing it onto §839 to §842, with the drafts of
  §822 and §823 on top, the tree this seal was first ported to; neither those four nor these two
  touch what the kit builds—, on 2026-10-21, on four cores with the one-minute load average from
  4.9 to 6.8 and an empty `target/artefacto`. What `-p zk-ssl-verify` builds —its crates in the tree
  and `Cargo.lock`— does not change since the commit of §815, nor against `5fd5212` (`git diff`,
  empty): it is the kit of 0.4.3's sources. With the tool from before this seal, which only changes
  comments of §815's, it comes out on this machine as
  `a0ae4bfdddbaa20739fbe83b1bf092544c465c25d98a683f94ac5a5cf60e174a` (2.587.048 B, 78 s), with 41
  `/home/x/.cargo/registry/src/…` paths; it is not the hash of the published 0.4.3,
  `1b120e597897e005…` (section 0), produced on another machine with `rustc` 1.97.1. With this
  seal's, `cc2077db8a608fe253e441c5fe6abb90a1e0e7ed7d467dac0899448e52aca98b` (2.586.408 B, 64 s),
  with those 41 as `/registry/…` and 12 of its 35 sections different. Both pass the twelve manifests
  —`paquete` 84/84, `consumo` 17/17, `conflicto` 17/17, `rechazo` 84/84, `edad` 15/15, `pendiente`
  10/10, `pago` 13/13, `prenda` 11/11, `completitud` 81/81, `ancla` 22/22, `ancla-cofirmada` 27/27
  and `rotacion` 34/34—. This seal's `--check` comes out VERDE: the same binary from the two roots,
  no machine paths, and the twelve manifests also from inside the tarball, in 177 s cold and 53 s
  with no changes, with almost twice the load of above. With the ten variables of the second item at
  once (`RUSTC_WRAPPER=/bin/false` among them), the same binary and the same tarball, in 56 s at
  most: the figure also counts the wait for another build on the machine to finish.
- **On this seal's tree** —measured with `main`'s §852 and §819 to §823 on top; on §859 what the kit
  builds is the same, because, in `crates/`, `Cargo.toml` and `Cargo.lock`, §853 to §859 only change
  two scripts in `crates/zk-ssl-medio/tests/`, which the kit does not build, and it was not measured
  again—, on 2026-10-21, on four cores with the one-minute load average from 5.7 to 8.3 and an empty
  `target/artefacto`. What the kit builds does change since `5fd5212`, the commit of 0.4.3: §848 and
  §849 touch comments of `crates/zk-ssl-medio/src/lib.rs` and of `crates/zk-ssl-hash/src/lib.rs`,
  and §848 adds a `[[test]]` to the `Cargo.toml` of `zk-ssl-medio`; and `paquete` has five more
  vectors (§852). With the tool from before this seal it comes out on this machine as
  `3923dcc03198efb7a122420ae649be918ad5c00bdd1a716809488f91c514d9bd` (2.587.056 B), and not
  `a0ae4bfd…`; with the `Cargo.toml` of `zk-ssl-medio` from `5fd5212`, without the `[[test]]`, the
  same comes out (measured on 2026-10-22): what moves the hash is the comments, as «What the hash
  depends on», below, says. With this seal's,
  `2d94c323542bbe2d73db8d388403f831b3dc4d52f6819f0d56c26f88bb0426c9` (2.586.416 B), with the 41
  paths as `/registry/…` and 12 of its 35 sections different, the same as above. Both pass the
  twelve manifests, with `paquete` 89/89 and the rest as above, and neither is the hash of the
  published 0.4.3. This seal's `--check` comes out VERDE, with the same binary from the two roots,
  no machine paths and the twelve manifests also from inside the tarball. The measured times count
  the wait for `cargo`'s lock, which another build on the machine held: 193 s the kit with the tool
  from before, 237 s with this seal's, and the `--check` 182 s cold and 243 s with no changes;
  without the wait, about 71, 70, 171 and 50 s, estimated from the time each pass started.

**What the hash depends on, with this seal's tool**: on `rustc` (its version and commit, in
`VERSION`), on `Cargo.lock`, on what is built and with which features (the kit, `-p zk-ssl-verify`
alone), and on the host: on its `cc` and `as`, because the host assembles blake3's assembly (in
0.4.2, 240 bytes of `.text` differ), and on the objects `cc` links into every executable, GCC's,
`crtbeginS.o` and `crtendS.o`, and those of the host's glibc, `Scrt1.o`, `crti.o` and `crtn.o`, from
libc6-dev. Measured with `nm` on the kit, the node and the witness: `_start` and `__abi_tag` come
from `Scrt1.o`, `_init` and `_fini` from `crti.o`, and the rest from `crtbeginS.o` and `crtendS.o`;
from `libc_nonshared.a`, `libgcc.a` and `libgcc_eh.a` they carry no symbol, except `__floattidf` in
the node, which `compiler_builtins` also defines (unwinding is the dynamic `libgcc_s.so.1`'s). In
0.4.2 the startup objects did not weigh: the 240 bytes of `.text` all fall in six blake3 functions.
`.comment` names the host's GCC because `crtbeginS.o` and `crtendS.o` carry the «GCC: …» string
(measured with `readelf`; blake3's objects, `libgcc.a`, `libgcc_eh.a` and `libc_nonshared.a`, none).
The `cc` is the one cc-rs uses by default when the host is the target, `cc`, and the same one
`rustc` links with; `VERSION` names it, and the `as` that `cc` calls, the host's glibc (the first
line of `ldd --version`) and the hash of the five startup objects. The linker is not the host's: the
published one's `.comment` says «Linker: LLD 22.1.6», and that of the kit here, «LLD 21.1.8», each
toolchain's `rust-lld`. No longer on the tree path, the user, `CARGO_HOME`, `RUSTUP_HOME`, whether
`rust-src` is installed or the environment variables not in `PASAN`: measured above. And it does
depend on the bytes of the source, not only on what compiles: one comment line at the end of
`crates/zk-ssl-verify/src/lib.rs` takes the kit from `326d232b81250581…` to `7f344beb37edc753…`, with
the same size and two different sections, the symbol names (`.strtab`) and the `build-id`; the code,
the same. That two machines with the same `rustc`, `cc`, `as` and startup objects give the same hash
remains unshown: there is a single machine here.

The demonstrations with live nodes — bringing up two ledgers, publishing the same label in both,
capturing the envelope — are the benches `tools/banco_dos_libros.sh`, `tools/banco_consumo.sh` and
`tools/banco_apagado.sh`, and those of step 5, `tools/banco_completitud.sh`,
`tools/banco_mentiroso_sin_resolver.sh` and `tools/banco_recibo_agregado.sh`; they do not travel in
the kit because they spawn processes, and the vectors above are their captures.

## The node and the witness, from two roots

`bash tools/artefacto.sh --binarios` (§824) builds the production node
—`-p zk-ssl-node --no-default-features`, without `dev`, the issuance with the test custodians— and
the witness —`-p zk-ssl-cli`— with the same rules, the same environment and the same host stamp as
the kit, in the same two roots, and requires each to come out byte-identical and with no machine
paths; otherwise it says which ELF sections differ. Each goes in its own `cargo` invocation: built
together, `cargo` unifies features and the node's `zk-ssl` would come in with the `sandbox` the
witness asks for. That is why the node of the measurement above, with `dev` and built together with
the witness, is another binary. Measured with `rustc` 1.94.1 and the three versions of the tool
above, with the same hashes: the node `bc10653591e71134…` (10.623.960 B) and the witness
`1d074aaece334c65…` (14.581.944 B), equal from both roots. Those are the sources of `b6994c3`: since
then the sources of the node have changed (§813, §815, §816, §817, §819, §820, §821, §832, §833,
§835, §839, §840, §841, §842, §848, §849, §860 and §861), and those of the witness (the same except
§813, plus §850), and neither on the tree of §817 nor on this seal's have they been built this way.
On four cores, with the one-minute load average in brackets: with the first version, 474 s cold
—after the cold `--check`—, 5 s with no changes and 188 s with a change in `crates/zk-ssl` (2.2 to
4.4); with the second, 341, 4 and 164 s (3.9 to 7.3); with the one being sealed, 366 s cold —after
the `--check`— and 4 s with no changes (2.4 to 3.7). The canon runs it from `--largo`, in the «3 ter
binarios» block of `tools/canon.sh`, and not at every seal, by the criterion of the levels: a gate
that makes every seal dearer ends up being skipped (`doc/METODO.md` §«6. El canon y los bancos»,
§225), and a seal touching the layer would pay 164 to 188 s, and a cold one 341 to 474, on top of
the 470 to 926 of a `--sello` (`tools/canon.sh`, at the foot). As with the kit, one comment line at
the end of `crates/zk-ssl/src/lib.rs` takes them to `4963bb3a7cc4b69c…` and `4d2acfa127d232b4…`: the
`build-id`, `.symtab` and `.strtab` differ, the symbol names; the code, the same.

**What it leaves on disk.** After `--binarios`, `target/artefacto/tA` and `target/artefacto/raiz2`,
with its `target`, take 624 and 687 MB, about 1.3 GB, and the tool does not delete them: every tree
that passes a `--largo` keeps them.
`rm -rf target/artefacto/tA target/artefacto/raiz2 target/artefacto/raiz2.lista` removes them, and
the next pass is cold.

**What its hash depends on**: the same as the kit's —`rustc`, `Cargo.lock`, what is built and with
which features, and the host's `cc`, `as` and startup objects—, and the witness carries more native
code: ring's C and assembly, besides blake3's assembly, which both carry. Not on the tree path, the
user, `CARGO_HOME`, `RUSTUP_HOME` or whether `rust-src` is installed: measured on 2026-10-20 with the
second version, with another `HOME`, a `CARGO_HOME` outside it and `rust-src` installed in another
`RUSTUP_HOME`, the three at once, the same two hashes come out (563 s cold). Across machines,
unshown.

**What it does NOT prove.** That the operator runs that binary. A reproducible hash says that a
published binary comes from its source; the operator starts the node on its own machine, and nothing
it signs says which binary signed it: `hash_verificador_vigente` is not in the head (`BACKLOG.md`,
entries 54 and 83). It is a precondition of that road, not its closure, and `SECURITY.md` §«2. Qué
NO protege» —«El operador puede cambiar el verificador, y hoy eso es invisible»— still says what it
says. Nor are there published hashes of the node or the witness: no release carries them, and
publishing them is the author's decision.
