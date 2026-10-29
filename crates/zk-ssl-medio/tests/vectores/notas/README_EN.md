# The notes of the middle, checked against torchwood (§632)

> **Español:** [`crates/zk-ssl-medio/tests/vectores/notas/README.md`](./README.md) is the original
> of this page, in Spanish; it prevails wherever the two differ. This translation is of commit
> `7766efb` (audit entry §917) and was produced by the project's assistant, a Claude Code session,
> in audit entry §918 (`GENAI.md`): it is a generated artefact, not text written by the author. Line
> citations of the form `file:N` refer to the Spanish files.

33 `checkpoint` notes of the middle (RFC-0013 D-A, D-C) and what two verifiers written by
different hands say of each one: `zk_ssl_medio::nota` and
[`filippo.io/torchwood`](https://pkg.go.dev/filippo.io/torchwood) v0.10.0, the Go library
of the author of the C2SP specifications (`signed-note`, `tlog-checkpoint`,
`tlog-cosignature`). torchwood signs and verifies type `0x06` with the `crypto/mldsa` of the
standard library of Go 1.27, and the note with `golang.org/x/mod/sumdb/note`. The Rust side
uses RustCrypto's `ml-dsa` 0.1.1 crate: two ML-DSA-44 implementations that share no code.

| files | who signed them | what they are |
|---|---|---|
| `pos-00-anclas.txt` … `pos-13-anclas.txt` | torchwood | the publisher and one witness over trees of 0, 1, 2, 5 and 13 anchors |
| `pos-sin-testigo.txt`, `pos-testigo-primero.txt` | torchwood | the 5-anchor one without a witness, and with the witness in front |
| `neg-*.txt` (23) | torchwood | the 5-anchor one broken in one way each |
| `rust-*.txt` (3) | this crate | two with salt and one deterministic, over 3, 21 and 8 anchors |
| `publicador.vkey`, `testigo.vkey` | torchwood | the two keys, in the `vkey` format of `signed-note` |
| `veredictos-torchwood.txt` | torchwood | what `note.Open` with `NewLogVerifier` says of each note |
| `MANIFIESTO.txt` | — | what each of the two says, and the reason for each difference |

The `origin` of all of them is `zkssl/v1/` and the fingerprint of the operator's key of
`spec/vectors/ancla/` (the `clave` field of its anchor). The leaves are synthetic: the
fingerprint of anchor `i` is `SHA-256(u64be(i))`, and torchwood computes the root with the
tree of `golang.org/x/mod/sumdb/tlog`. The seeds are test seeds and are in
`torchwood/main.go`: the publisher `00 01 … 1f`, the witness `20 21 … 3f` and an impostor
key `40 41 … 5f`.

## How they are regenerated

With Go 1.27 or later and network to download the modules that `torchwood/go.sum` pins:

```bash
cd crates/zk-ssl-medio/tests/vectores/notas
go -C torchwood run . generar ..     # reescribe pos-*, neg-*, las dos vkey
go -C torchwood run . verificar ..   # reescribe veredictos-torchwood.txt
```

The signatures are deterministic, but the timestamp is the clock's: regenerating gives new
notes, with other timestamps, and `MANIFIESTO.txt` still holds. The `rust-*` ones were
written by a program outside the tree that calls `Publicador::firmar`; `verificar` judges
them together with the rest. The canon does not run Go: `tests/vectores_notas.rs` reads what
Go left written.

## What the comparison says

torchwood accepts the 10 positives, the 3 signed here too, and rejects 19 of the 23
negatives. The other 4 are rules in which the middle is stricter, on purpose and declared in
the header of `src/nota.rs`: an `origin` different from the publisher's name (D-A), two
publisher lines (`note.Open` verifies the first and discards the rest without looking at
them) and a non-canonical base64 signature (Go decodes it anyway). There is none the other
way round: nothing torchwood rejects is accepted by the middle.

## Licenses

Nothing of torchwood or Go is copied here: `torchwood/main.go` belongs to this repository
and only imports them, and the notes are its output. torchwood is distributed under the
3-clause BSD license (Copyright The Go Authors and The Torchwood Authors) and Go under its
own, also BSD.
