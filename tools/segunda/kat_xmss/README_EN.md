# xmss-kat — known-answer vectors for XMSS^MT, with two implementations behind them

> **Español:** [`tools/segunda/kat_xmss/README.md`](./README.md) is the original of this page, in
> Spanish; it prevails wherever the two differ. This translation is of commit `7766efb` (audit entry
> §917) and was produced by the project's assistant, a Claude Code session, in audit entry §918
> (`GENAI.md`): it is a generated artefact, not text written by the author. Line citations of the
> form `file:N` refer to the Spanish files.

> **English:** a known-answer test corpus for XMSS^MT (RFC 8391): keys and signatures produced by
> RustCrypto's `xmss 0.1.0-pre.0` from fixed seeds, and a dependency-free Python verifier written
> from the RFC text that checks every one of them, with three falsifiers per signature. Two
> origins, the same bytes. Born in Arqueo; see the provenance section.

**What it is.** A corpus of XMSS^MT keys and signatures produced by `xmss 0.1.0-pre.0` —the
RustCrypto/signatures crate— from fixed seeds, and `xmss.py`, a verification of RFC 8391 written
from the text of the RFC, in Python and without any dependency, which verifies every signature of
the corpus. Two origins, the same bytes.

**Why it exists.** RustCrypto excludes the KATs from the packages it publishes, and RFC 8391 brings
no vectors for the multi-tree parameter sets. Whoever signs with XMSS^MT and pins the crate's
version has no net underneath: a change of bytes between versions is caught by nobody. This
directory is that net.

| file | what it is |
|---|---|
| `xmss.py` | the XMSS^MT verification of RFC 8391 (sections 2 to 4) for the eight SHA2 multi-tree parameter sets with n = 32; it only verifies, it does not sign |
| `vectores.json` | two parameter sets, `XMSSMT-SHA2_40/8_256` (OID 5) and `XMSSMT-SHA2_20/2_256` (OID 1): a 96-byte seed, a 68-byte public key, and four consecutive signatures, indices 0 to 3, over messages of 0, 3, 50 and 1000 bytes, in the crate's two forms, attached (`firma ‖ mensaje`) and detached |
| `juez_xmss.py` | verifies every signature with `xmss.py`, binds the embedded index to the declared one and the detached signature to the attached one, and runs three falsifiers per signature: one byte flipped in the signature, in the message and in the key's root |
| `generador/` | the Rust program that produced `vectores.json` from its fixed seeds, with the crate pinned with `=` |

```bash
python3 juez_xmss.py                     # -> kat_xmss: 8 de 8 firmas verifican ... y los tres falsadores callan
cargo run --release --manifest-path generador/Cargo.toml > /tmp/v.json && cmp vectores.json /tmp/v.json
```

## What was measured (01-10-2026)

| measure | result |
|---|---|
| signatures of the corpus verified by `xmss.py` | 8 of 8 |
| falsifiers per signature that stop verifying | 3 of 3, in all 8 |
| the detached signature is the attached one without the message | 8 of 8 |
| the embedded index is the declared one | 8 of 8 |
| signature size, 40/8 and 20/2 | 18,469 B and 4,963 B, what RFC 8391 says |
| `generador/` reproduces `vectores.json` | byte for byte: sha256 `259b8df2bb6633ff…` both times |
| one XMSS^MT 40/8 verification in Python | ~40 ms |

## What it pins

- That the published key carries the RFC 8391 OID of the **multi-tree** registry (`0x00000005` for
  40/8, `0x00000001` for 20/2) and that a library that reads it as the RFC mandates needs no
  reading workaround.
- That the crate's signature is, byte for byte, that of RFC 8391: `idx` (⌈h/8⌉) ‖ `r` ‖ d × (WOTS+ ‖
  auth), and that `sign` returns it followed by the message.
- That verification is deterministic given the state: the detached and the attached signature, at
  the same index and message from the same seed, are the same.

## What it does NOT pin

- Nothing of the private key's state nor of how to avoid reusing an index: that is
  [`hbs-state`](https://github.com/atoranzo/hbs-state), by the same author.
- Nothing of key generation beyond «this seed gives this key with this crate».
- The 512-bit parameter sets, the SHAKE ones, or single-tree XMSS: `xmss.py` only implements
  XMSS^MT with SHA2 and n = 32.
- That `xmss.py` or the crate are audited: they are not, neither of them.

## Provenance

Born in Arqueo (<https://github.com/atoranzo/Arqueo-open-conservation-proofs-for-closed-ledgers>),
in `tools/segunda/kat_xmss/`, to close entry 77 of its BACKLOG, «KAT ausente» [KAT missing], and
so that its second implementation verifies the signed heads of its vector catalogue. It was
extracted with `git subtree split`, like `mtc-core` and `hbs-state`: it shares with Arqueo the
discipline, not the problem, and carries not a byte of its protocol. It lives at
<https://github.com/atoranzo/xmss-kat>; Arqueo keeps using it from that directory.

Written in a session with generative AI assistance (Claude, by Anthropic), with the method that
Arqueo's `GENAI.md` declares: it is measured first, the author decides, and nothing enters without
passing its gates. Author: Ángel José Toranzo Portela.

## License

MIT OR Apache-2.0, at the choice of whoever uses it: `LICENSE-MIT` and `LICENSE-APACHE`. The
generator depends on RustCrypto's `xmss` crate, under the same dual license, and does not include
it.
