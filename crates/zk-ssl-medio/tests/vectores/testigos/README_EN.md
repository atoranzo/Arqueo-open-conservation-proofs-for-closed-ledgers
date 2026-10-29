# The medium's publisher against local witnesses (§808)

> **Español:** [`crates/zk-ssl-medio/tests/vectores/testigos/README.md`](./README.md) is the
> original of this page, in Spanish; it prevails wherever the two differ. This translation is of
> commit `7766efb` (audit entry §917) and was produced by the project's assistant, a Claude Code
> session, in audit entry §918 (`GENAI.md`): it is a generated artefact, not text written by the
> author. Line citations of the form `file:N` refer to the Spanish files.

What two witnesses by the author of the C2SP specifications answered,
**on `127.0.0.1`**, to nine steps of the medium's publisher
(RFC-0013 E3a), and the program and the script that record it. The client is
`zk_ssl_medio::testigos` (with `firmar`); the protocol, C2SP's `tlog-witness`
v1.1.0 (commit `b2dd989`, the same text as its `main` on
2026-10-14).

The source of the witnesses is the module `filippo.io/torchwood` v0.10.0:
git `2fa53be3e7049b61226d8a93ff2e1fc0af92d8d7` (the tag `v0.10.0`),
`h1:zgBIImawK53X7lM0qHKcjn0pbDtFLqU04NU9qDvBnjs=` in `sum.golang.org` and
`go.mod` `h1:hZt8lhTBBCK77TmuK+q0D7mNcnt2ovQkJs+zdonDRog=`. `rehacer.sh`
downloads it with `go mod download`, which with the default `GOSUMDB`
checks it against that database.

| witness | what it is | what it co-signs with |
|---|---|---|
| `testigo.invalid/torchwood-a` and `-b` | litewitness's `add-checkpoint`: `filippo.io/torchwood` v0.10.0 (git `2fa53be`), `internal/witness.NewWitness`, with the same 10 KiB `MaxBytesHandler` and the same timeouts that `cmd/litewitness` sets | ML-DSA-44, type `0x06`: what the publisher retains and the kit verifies |
| `testigo.invalid/litewitness` | `cmd/litewitness` v0.10.0 **untouched**, with its key in an SSH agent | Ed25519 —the binary admits no other—: the publisher ignores it, as the specification mandates for a key it does not know |

All three keep their state in the database of `cmd/witnessctl`, which
registers the medium (`add-log`, `add-key`) as a log is registered in a
real witness.

## What is here

| file | what it is |
|---|---|
| `torchwood.json` | the recording: the origin, the witnesses and their `vkey`, and for each step what was given to the client (`entrada`) and what happened (`salida`), with every HTTP request and every response as they were; and the `ancla-cofirmada` envelope end to end |
| `testigo-local/main.go` | `testigo-local mldsa` (the litewitness engine with ML-DSA-44), `agente` (the in-memory SSH agent for litewitness) and `comprobar` (verifies a co-signature with torchwood). It is compiled INSIDE a copy of torchwood, because it uses its `internal/witness` |
| `rehacer.sh` | the script: it downloads torchwood v0.10.0 through the Go proxy, compiles it with the Go 1.27 toolchain that its `go.mod` asks for, brings up the three witnesses, runs the nine steps and the end-to-end check, and with `--guardar` rewrites `torchwood.json` |

## The nine steps

`desconocido` (the origin they do not know yet: `404`), `primera` (`old 0`:
they co-sign), `sin-memoria` (the publisher goes back to `old 0`: `409` with 3 and
the retry co-signs), `crece` (from 3 to 7 anchors with its proof),
`atrasado` (`old 0` with 9: `409` with 7 and the proof from 7 to 9), `retroceso`
(the directory restored to 5 anchors: `409` with 9, and the client STOPS:
entry 103), `bifurcado` (one anchor rewritten, from 9 to 11: `422`), `otra-raiz` (9
anchors with another root: `422`) and `impostor` (the note of another key with the
same name: `403`). The first three anchors are those of the catalogue
`spec/vectors/ancla-cofirmada/`, which the kit derives from its three signed
heads; the others, `SHA-256(u64be(i))`, and the rewritten one, that of `i + 1000`.

The three witnesses say the same in the nine steps; the only thing that changes
is what they sign the `200` with. And end to end: the co-signed note of
`primera` —the one of the catalogue's publisher (seed `01`) and the co-signatures
of torchwood-a and -b— enters the catalogue's envelope `cofirmada-dos-testigos.json`,
the kit gives it VERDE («la cofirman 2 testigo(s)»), the cli's
judge gives it ACREDITADA [accredited] with `k = 2` and not with `k = 3`, litewitness's Ed25519 line
verifies with torchwood, and torchwood reads the retained co-signature.

## How it is redone

With Go in the `PATH` (or `GO=/ruta/al/go`), network for the Go proxy and the
tree in any state, from the root:

```bash
bash crates/zk-ssl-medio/tests/vectores/testigos/rehacer.sh            # mide y no escribe
bash crates/zk-ssl-medio/tests/vectores/testigos/rehacer.sh --guardar  # reescribe torchwood.json
```

**Outside the canon and outside `tools/bancos.sh`**: it needs Go and a network. The
publisher's note is signed deterministically so that the request can be
redone byte for byte; the witnesses' co-signatures carry the mark of their
clock and a salted signature, so each recording is a different one, and
`tests/testigos_torchwood.rs` stays valid with any that comes out
VERDE. What that test checks at each canon is in its header.

## omniwitness, by another hand (§848)

The same client against **`omniwitness`**, the witness of
`github.com/transparency-dev/witness`, **untouched**, on `127.0.0.1`. It is
commit `539715523ab9d0ee7d036926f9d1b00266e1c567` (`5397155`, the tip
of its `main` on 2026-10-15; the module
`v0.0.0-20261014123032-539715523ab9`,
`h1:stSGpwkttvZrI5MHMcq7PluZdqpsulroZU9NvKWB8Bw=` in `sum.golang.org` and
`go.mod` `h1:DoW27RnDDy0bIJt3k8QkRJF+yCTRWAJdaPcztwEqucs=`). Its co-signing
keys are those of `github.com/transparency-dev/formats` at `340a9e5`
(`v0.1.2-0.20261013130728-340a9e55b2b7`), which its `go.mod` pins. The
protocol is the same `tlog-witness` (`b2dd989`): its text, and that of
`tlog-cosignature`, did not change up to C2SP's `main` of 2026-10-21
(`e5746a0`).

| witness | what it is | what it co-signs with |
|---|---|---|
| `testigo.invalid/omniwitness-a` | `cmd/omniwitness` with one key (`--private_key_path`) | ML-DSA-44, type `0x06`: what the publisher retains and the kit verifies |
| `testigo.invalid/omniwitness-b` | `cmd/omniwitness` with two keys of the same name, as in the rotation of its README: first an ML-DSA-44 one and behind it an Ed25519 one | both in every `200`: the client retains the `0x06` one and ignores the Ed25519 one, from a key it does not know |

The medium's log is not on its list: it is configured by hand, an
`Origin` and `PublicKey` entry (the publisher's `0x06` `vkey`) in the
YAML of `--additional_logs`. No public network: no metrics, no
distributor, no list of a witness network.

| file | what it is |
|---|---|
| `omniwitness.json` | the recording, with the same shape as `torchwood.json`: the origin, the witnesses with their keys (the seed and the `vkey` of each one), each step with its HTTP `entrada` and `salida` as they were, and the `ancla-cofirmada` envelope |
| `omniwitness-local/main.go` | `omniwitness-local clave` (the private key that omniwitness reads, from a seed, and its `vkey`) and `comprobar` (verifies a co-signature with the verifier of `formats`). It is compiled INSIDE a copy of `witness`, with the versions that its `go.mod` pins |
| `rehacer_omniwitness.sh` | the script: it downloads `witness` at `5397155` through the Go proxy, compiles `omniwitness` and `omniwitness-local` with the Go toolchain it asks for (1.26), runs the ten steps and the end-to-end check, and with `--guardar` rewrites `omniwitness.json` |

The ten steps, in terms of `tlog-witness`: `desconocido` (the origin
that is not configured yet: `404`); each witness is stopped and brought up
again with the log in `--additional_logs` and the same database;
`primera` (`old 0`: they co-sign), `sin-memoria` (`old 0` with 3: `409` with 3
and the retry co-signs), `crece` (from 3 to 7 with its proof), `atrasado`
(`old 0` with 9: `409` with 7 and the proof from 7 to 9), `nota-menor` (a note
of 5 with `old 0`: `409` with 9, a size larger than the note's, and the
client STOPS), `prueba-invalida` (from 9 to 11 with a consistency proof
that does not verify against the root of 9 that the witness holds: `422`),
`otra-raiz` (9 with another root: `422`), `impostor` (the note of another key
with the same origin: `403`) and `igual` (9 with `old 9` and the empty proof:
they co-sign). The leaves are those of `torchwood.json`.

The two witnesses say the same in the ten steps; omniwitness-b signs
the `200` with two lines. And end to end: the co-signed note of
`primera` enters the catalogue's envelope `cofirmada-dos-testigos.json`, the
kit gives it VERDE («la cofirman 2 testigo(s)»), the cli's judge gives it ACREDITADA
with `k = 2` and not with `k = 3`, `formats` reads the two retained co-signatures,
and omniwitness-b's Ed25519 line verifies with `formats`.

```bash
bash crates/zk-ssl-medio/tests/vectores/testigos/rehacer_omniwitness.sh            # mide y no escribe
bash crates/zk-ssl-medio/tests/vectores/testigos/rehacer_omniwitness.sh --guardar  # reescribe omniwitness.json
```

Outside the canon, like `rehacer.sh`. `tests/testigos_omniwitness.rs`
re-reads the recording at each canon, with what its header says, and also
checks that the ML-DSA-44 `vkey` of each witness, which `omniwitness-local`
derived from its seed with Go's ML-DSA (`filippo.io/mldsa` v1.0.0, the one of
`formats`), is the one that this crate's `ml-dsa` derives from the same seed.

## Licenses

Nothing of torchwood or of Go is copied here: `testigo-local/main.go` belongs to
this repository and only imports them, and `torchwood.json` is what they
answered. torchwood is distributed under the 3-clause BSD license
(Copyright The Go Authors and The Torchwood Authors) and Go under its own,
also BSD. The seeds are test seeds.

⚠️ §848: nor is anything copied from `transparency-dev/witness` or from
`transparency-dev/formats`, which are distributed under the Apache License 2.0
(the header of `cmd/omniwitness/monolith.go` says Copyright Google LLC),
or from `filippo.io/mldsa` (3-clause BSD):
`omniwitness-local/main.go` belongs to this repository and only imports them, and
`omniwitness.json` is what omniwitness answered.
