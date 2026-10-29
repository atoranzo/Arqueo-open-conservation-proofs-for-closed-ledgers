# Vectors of the middle: provenance and licenses

> **Español:** [`crates/zk-ssl-medio/tests/vectores/README.md`](./README.md) is the original of this
> page, in Spanish; it prevails wherever the two differ. This translation is of commit `7766efb`
> (audit entry §917) and was produced by the project's assistant, a Claude Code session, in audit
> entry §918 (`GENAI.md`): it is a generated artefact, not text written by the author. Line
> citations of the form `file:N` refer to the Spanish files.

Three corpora from two outside hands, copied **byte for byte** (§631). Arqueo wrote none of
them; what Arqueo adds is in the `.rs` of `tests/` and is stated there. The fourth, the
`checkpoint` notes checked against torchwood (§632), has its own `notas/README.md`; and since
§808, what litewitness and its engine answered locally to the publisher, `testigos/README.md`.
⚠️ §848: and in the same `testigos/README.md`, what omniwitness, from
`transparency-dev/witness`, answered locally.

## 1. The accumulated vectors of the IETF draft

`../vectores_ietf.rs` reads no files: it carries inside the four final hashes of the «Test
Vectors» appendix of `draft-ietf-plants-merkle-tree-certs` (712 subtree hashes, 12,807
inclusion paths, 42,893 consistency ones and 8,646 coverages: 65,058 cases over every subtree
of every tree up to 130 leaves). The file is `tests/vectors.rs` of mtc-core
(`github.com/atoranzo/mtc-core`, commit
`d3b0ca614e51f177a0c30f1d21abd87b91208f59`), with the adaptations marked
`ADAPTADO (§631)` in its header.

## 2. The large vectors of the same draft

`large_inclusion_proofs.json` and `large_consistency_proofs.json` are the files the «Large
Subtree Test Vectors» appendix references: proofs over trees of `2^48-1`, `2^63-1` and
`2^64-1` leaves, which nobody builds and which the **verifier** has to evaluate without
overflowing. They come from the `demo/` directory of the PLANTS group's repository
(`ietf-plants-wg/merkle-tree-certs`, commit `99097c9`, 2026-09-29) through mtc-core
`d3b0ca6` (`tests/vectors/`), without changing a byte:

| file | SHA-256 (first 16) |
|---|---|
| `large_consistency_proofs.json` | `7fcc6d9d0283ecf7` |
| `large_inclusion_proofs.json` | `19ffc377d123613d` |

The source repository declares that all its material consists of contributions to the IETF
standardization process (BCP 78, BCP 79 and the *IETF Trust Legal Provisions*), and that its
code components, vectors included, are under the IETF Trust's **Simplified BSD License**.
They are kept here under that license and with this attribution, apart from the MIT OR
Apache-2.0 license of the rest of the crate:

> Copyright (c) IETF Trust and the persons identified as authors of the
> code. All rights reserved. Redistribution and use in source and binary
> forms, with or without modification, is permitted pursuant to, and
> subject to the license terms contained in, the Simplified BSD License
> set forth in Section 4.c of the IETF Trust's Legal Provisions Relating
> to IETF Documents (https://trustee.ietf.org/license-info).

## 3. The transparency-dev probes

`transparency-dev/` is the four folders of `testdata/` of
`github.com/transparency-dev/merkle`, commit
`fbbcd741c3d1c69d8498487baa8edc9e5824847c` (2026-09-21): `inclusion` (98),
`consistency` (98), `subtreeinclusion` (204) and `subtreeconsistency` (285), 685 JSON files
that its `cmd/proofgen` generates. The SHA-256 of the sorted list of their `sha256sum`
(`find . -type f | LC_ALL=C sort | xargs sha256sum | sha256sum`, from the folder) is
`d9edf5f2c30801c8acde4c783ecbad3e30ecb40eebda2cf45428c449d204da6e`, the same here as at the
source.

It is the library with which transparency-dev's witness checks the consistency of a
checkpoint before co-signing it (`github.com/transparency-dev/witness`,
`witness/witness.go`, `proof.VerifyConsistency(rfc6962.DefaultHasher, ...)`, commit
`b4c9458`).

License: **Apache License 2.0** (Copyright 2025 Google LLC, header of
`cmd/proofgen/main.go`), the same as one of Arqueo's two; its text is in `LICENSE-APACHE`,
at the root of the repository. The JSON files carry no header of their own and have not
been modified.

`../vectores_transparency.rs` counts separately what the **type** rejects (a hash that is
not 32 bytes) and what the **tree** rejects, and pins by name the four positives that cannot
be written here. Why that separation matters —a measured blind spot— is in the test
`sus_positivas_con_un_bit_cambiado_en_32_bytes`.
