# Draft of a responsible disclosure to winterfell — the debt of §575

> **Español:** [`doc/reporte-winterfell-575.md`](./reporte-winterfell-575.md) is the original of
> this page, in Spanish; it prevails wherever the two differ. This translation is of commit
> `7766efb` (audit entry §917) and was produced by the project's assistant, a Claude Code session,
> in audit entry §918 (`GENAI.md`): it is a generated artefact, not text written by the author. Line
> citations of the form `file:N` refer to the Spanish files.

> **For the author, not for the public tree as a notice to third parties.** This is the draft of the
> report that `SECURITY.md` 3.7 and §575 left as a debt: *«winterfell upstream has the same defect,
> and notifying it is a debt»*. An assistant drafted it with the method of `GENAI.md`; sending it
> and accepting it is the author's.

## Through which channel, and which one in the end — MEASURED on 01-10-2026

The flaw is a denial of service in the deserialisation of other people's data, so it does not go as
a public issue or PR while it remains unfixed upstream. While looking for the private channel this
was measured, and not taken for granted:

- **GitHub's private vulnerability reporting is NOT enabled** on `facebook/winterfell`: there is no
  «Report a vulnerability» button. That door, the one that does not ask for Facebook, is closed, and
  only the owner of the repo can open it.
- **There is no `SECURITY.md` of its own** in winterfell's tree (`HEAD` at `2f78ee9`, 0.13.1). The
  policy that GitHub shows —report through Meta's programme at `facebook.com/whitehat`— is the
  **default template of the Facebook organisation**, inherited, not one of the project's. And that
  programme requires a Facebook account.
- **The maintainer, with a verifiable email:** whoever published 0.13.1 (commit `2f78ee9`,
  19-07-2025) is **Irakliy Khaburzaniya**, who commits with a public email, `irakliy81@gmail.com`.

Without Facebook, the honest route left for an unfixed flaw is a **private email directly to the
maintainer**. That is what was done: a report sent on **01-10-2026** to `irakliy81@gmail.com`, with
the cause and the route, the fix offered as a PR, and **without a reproducer**. The repository has
gone without a commit on `main` since 19-07-2025 and its team moved to Plonky3 (see
`doc/integracion-vertical-evaluacion.md`, §4), so there may be no answer: the email sets **90
days**, and once they pass without an answer the coordinated-disclosure escalation is a RustSec
advisory (`rustsec.org`), never a public issue in winterfell while it remains unfixed.
`AUDITORIA.md` §625.

## The flaw, in one sentence

When deserialising a proof or a Merkle batch, winterfell allocates memory for as many elements as a
length read from the bytes themselves says, **before** checking that those bytes exist. A malformed
input that declares an enormous length causes an allocation that fails, and an allocation that fails
**aborts the process** instead of returning an error: it is not a panic, so a consumer does not
catch it with `catch_unwind`. Anyone who deserialises a proof from an untrusted origin can bring
down the process that verifies it.

## What is confirmed, and on what

Measured on 01-10-2026, with rustc 1.97.0, against the crates **published on crates.io** (not a
fork): `winter-air`, `winter-utils`, `winter-crypto` and `winter-math`, all four pinned to
`=0.13.1`. Three entry routes are confirmed, each one aborting the process with a message of a
failed memory allocation, while the same call with an honest length returns `Err(UnexpectedEOF)`:

- The generic deserialisation of a `Vec` (`winter-utils`, `ByteReader::read_many`, through which
  every `Vec` that is read passes).
- `Proof::from_bytes` (`winter-air`), through the first length that a `Queries` reads.
- `BatchMerkleProof::read_from` (`winter-crypto`), through its count of node vectors.

The common cause is in `winter-utils 0.13.1`: `read_many` does `Vec::with_capacity(num_elements)`
before reading a single element, and `num_elements` comes from the bytes. The `check_eor` of the
`SliceReader` does not help because nobody calls it before that allocation.

## The fix, which already exists and is measured

Arqueo closed it in its copy in entry §575 (public in `SECURITY.md` 3.7 and in the README of
`crates/winter-air`): a reader that checks that the elements fit —at least one byte each— **before**
allocating, so that more elements than remaining bytes is `UnexpectedEOF`, the same error that
upstream already gives when it runs out of bytes. A well-formed proof is read the same, byte for
byte. It is the pattern that several deserialisation libraries apply: bounding the count by the
bytes available before allocating.

The text of the report can describe that fix; the concrete code, if they ask for it, is offered as a
pull request through the channel they indicate.

## What the report must NOT include

- A script ready to bring down a deployment: it is enough to describe the cause and the route, which
  is what a maintainer needs to fix it and reproduce it on their own.
- Nothing about specific deployments of third parties: the report is about the library, not about
  who uses it.
