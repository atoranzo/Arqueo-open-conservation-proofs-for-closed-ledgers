# RFC-0001 — The at-rest KDF: Argon2id with salt and cost, and the format version

> **Español:** [`spec/rfc/0001-el-kdf-de-reposo.md`](../0001-el-kdf-de-reposo.md) is the original of
> this page, in Spanish; it prevails wherever the two differ. This translation is of commit
> `7766efb` (audit entry §917) and was produced by the project's assistant, a Claude Code session,
> in audit entry §918 (`GENAI.md`): it is a generated artefact, not text written by the author. Line
> citations of the form `file:N` refer to the Spanish files.

- **Status:** PROPOSED (§702) — with its three stages built in the same seal. The step to
  ACCEPTED is the author's. Rule 4 of the PROCESO asks for the spec, the OpenRPC, the vectors and
  the suites: this RFC does not touch the wire, so neither `spec/RPC.md`, nor `spec/openrpc.json`,
  nor `spec/vectors/` move; the suites of the crates it touches came out green in §702, and the
  decision is not taken.
  ⚠️ §812: and a fourth stage, E4 —re-encryption—, which the author decided on as a stage of this
  RFC; it was drafted in private, with the fix of a defect of E2 (`spec/rfc/PROCESO.md` §«Fallos de
  seguridad» [security defects]), and enters built in §812, in the same seal as the fix. Still
  PROPOSED.
  ⚠️ §816: and the fix of a defect of E4 —the race between an opening and a re-encryption, which
  could leave an empty ledger in the ledger's place or lose it whole—, also drafted in private and
  built in §816, with its text. Still PROPOSED.
- **Author:** Ángel José Toranzo Portela
- **GenAI assistance:** Claude, in a Claude Code session in the cloud (§702): it measured,
  proposed, applied and checked, and the session that launched it commits it, outside step 4 of
  `GENAI.md` — see [`GENAI.md`](../../../GENAI.md)
  ⚠️ §812: and E4, Claude, in a Claude Code session in the cloud, which drafted it in private,
  applied it and checked it, also outside step 4
  ⚠️ §816: and the fix of the E4 race, Claude, in a Claude Code session in the cloud, which drafted
  it in private, applied it and checked it, also outside step 4
- **Date:** 2026-10-07; E4, 2026-10-14; the fix of its race, from 2026-10-15 to 2026-10-20 (§816)
- **Protocol version affected:** none. `zkssl/0.4` does not move: none of this crosses the wire.
  The AT-REST format changes: the SDK's keystore goes from `zkssl-keystore/1` to
  `zkssl-keystore/2`, the encrypted ledger gains the `meta:kdf` header and the encrypted snapshot
  the `0x02` mark. ⚠️ §812: E4 changes no format; it adds two reserved names next to every ledger
  the layer opens —also the one the node opens in the clear—, `<libro>.recifrando` and
  `<libro>.retirado`, and the rule with which the opening reads them. ⚠️ §816: and a third,
  `<libro>.cerrojo`: an empty file the layer creates next to every ledger it opens and never
  deletes, the ledger lock.
- **AUDITORIA entry(ies):** §702, §812, §816; §819, which gives a test to the sweep's refusal and
  to its warning; §820, which closes entry 120: the snapshot export, whole or nothing; and §821,
  which closes entry 121: the layer imported with a key keeps the key.

**Number.** 0001 was reserved for this since §206: the numbering note of RFC-0002 says so, and
RFC-0003 to 0006 repeat it. This RFC takes it.

## Status of the stages

| stage | what it delivers | breaks the wire? | status |
|---|---|---|---|
| E1 — the KDF, with its version | `zk_ssl::crypto::Kdf`: version 1 (SHA-256, read only) and 2 (Argon2id with salt and cost); its 29-byte header and its single reader, `Kdf::de_cabecera`; the domains of each use; `argon2` pinned with `=0.5.3` in the layer and in no other manifest | NO | sealed — §702 |
| E2 — the ledger and its snapshot | `meta:kdf` in the clear in every batch; the key is fixed on opening; a version 1 ledger is verified and migrated to 2 in one atomic batch (⚠️ §812: the batch left in the files what was sealed with v1; since §812 the migration goes by the E4 path); the encrypted snapshot, with the `0x02` mark and the ledger's header behind it (⚠️ §821: or, if a layer imported with a key exports it, that of the snapshot it comes from, or one with a new salt if it came from v1: entry 121); `0x01` is still imported | NO | sealed — §702 |
| E3 — the keystore | `zkssl-keystore/2`, with `kdf_sal` and `kdf_coste` in the clear; a `zkssl-keystore/1` opened with its passphrase is rewritten in 2, whole or nothing | NO | sealed — §702 |
| E4 — re-encryption | `SovereignLayer::recifrar` changes the passphrase and the salt of the whole ledger: it copies it, sealed with the new key, to `<libro>.recifrando`, verifies it in the clear against the original reading it from disk, puts it in the ledger's place with a rename and deletes the old one whole: whole or nothing. The E2 migration goes by the same path, and every opening resolves what a death halfway leaves: backwards before the commit, forwards after it. ⚠️ §816: and the opening and the re-encryption take the ledger lock, `<libro>.cerrojo`: without it, an opening at the same time as a re-encryption could leave an empty ledger or lose the ledger | NO | built — §812, drafted in private; its race, fixed in §816, also in private |

## Motivation

Since §199 the SDK's keystore kept the spending key encrypted with XChaCha20-Poly1305 and a key
`SHA-256(ZK-SSL-keystore-v1 ‖ frase)`; the layer's encrypted ledger, with
`SHA-256(ZK-SSL-ledger-key-v1
‖ frase)`. Neither carried salt or cost. The module, `SECURITY.md` and `ARQUITECTURA.md` declared
it: «SHA-256 is not a password derivation function».

What that lets whoever takes away a keystore, a disk, a copy or a snapshot do:

- **Try passphrases offline at the speed of a hash.** Measured in §702, on this machine and under
  load: a version 1 derivation costs about 159 ns; a version 2 one, about 427 ms. That is some six
  orders of magnitude per attempt.
- **One table for all.** Without salt, the same passphrase gives the same key in any keystore, and
  in any ledger: a dictionary precomputed against the domain serves for every file.

That it was declared did not make it harmless: the attack is exploitable today against the SDK's
keystore, and the fix is small.

## Design

### The derivation (E1)

| version | key | written | read |
|---|---|---|---|
| 1 | `SHA-256(dominio_v1 ‖ frase)` | no | yes, to migrate |
| 2 | Argon2id, version `0x13` (RFC 9106): `P` = the passphrase in UTF-8, `S` = the salt (16 bytes from the system generator), `K` empty, `X` = `dominio_v2`, `t` = 3, `m` = 65 536 KiB, `p` = 4, `T` = 32 bytes | yes | yes |

The cost is the **second recommended option of RFC 9106 (§4)**: the one the Python library
argon2-cffi 25.1.0 calls `RFC_9106_LOW_MEMORY` and takes by default. The RFC's text could not be
read from the §702 session —the proxy rejects rfc-editor.org and ietf.org—, and the figure comes
from that library, which attributes it to the RFC. The OWASP sheet asks for Argon2id, in its
summary, a minimum of 19 MiB, `t` = 2 and `p` = 1, and this one uses more memory and more passes.
The sheet was read from its source —the Markdown of `cheatsheets/Password_Storage_Cheat_Sheet` in
the `OWASP/CheatSheetSeries` repository, at commit `29994dd`—, because the proxy also rejects
cheatsheetseries.owasp.org.

**One cost per version.** A version 2 file with another cost is not read, even if it is higher:
raising the cost is a new version, not a free field. So no file can ask the reader for arbitrary
memory, and what version 2 promises is a single thing.

**The domain is the associated data `X`.** The new domains, `ZK-SSL-ledger-key-v2` and
`ZK-SSL-keystore-v2`, enter the registry (`zk-ssl-hash`, §286). The ledger's key and the
keystore's do not coincide even if they share passphrase, salt and cost, and a test demands it
with the keystore's salt.

**The header** (29 bytes, in the clear):

```text
0   versión   1 B   = 2
1   m_kib     4 B   u32 LE = 65536
5   t         4 B   u32 LE = 3
9   p         4 B   u32 LE = 4
13  sal      16 B
```

Version 1 has no header: its absence, with encrypted data, is version 1. `Kdf::de_cabecera` is
the single reader, and it rejects, saying why, another length, another version or another cost;
the keystore recomposes the header from its JSON and reads it with it.

**The dependency.** RustCrypto's `argon2`, in the layer and in no other manifest, with `=0.5.3`
and only the `zeroize` feature. Argon2id's memory is allocated by `crypto.rs` and wiped on
release; with the `alloc` feature the crate allocates it and returns it without wiping. 0.6.0, the
latest —of 27-08-2026—, was measured and discarded: it enters the lock with four packages and
moves the edges of `digest` 0.11.3, which is in the kit's closed list (§694), so the kit would
gain `block-buffer` 0.12.1 without compiling `argon2`. 0.5.3 enters with two, `argon2` and
`base64ct`, and moves no edge of a package that was already there. The kit does not change.

### The ledger and its snapshot (E2)

- `SovereignLayer::open_encrypted` receives the key with its passphrase
  (`LedgerKey::from_passphrase`, which no longer derives anything) and **fixes** it to the
  ledger's KDF: with `meta:kdf`, to its own; without it and with data, to version 1; without
  data, to a version 2 with a new salt. Once fixed, the key does not carry the passphrase.
- Every batch (`commit`) writes `meta:kdf` in the clear next to what it seals: the first batch of
  a new ledger writes it with its data.
- **A version 1 ledger is migrated on opening it.** First `load` verifies it whole with the old
  key; then every sealed value is opened with it and sealed again with a version 2 one, and the
  header is written, all in one batch: sled applies it whole or does not apply it. Only
  `meta:geometry_v7`, `meta:migrated` and `meta:kdf` go in the clear; a value that does not open
  with the old key stops the migration without writing anything. With another passphrase, the
  ledger is not touched.
  ⚠️ §812: «all in one batch: sled applies it whole or does not apply it» was true of what is
  READ, not of what stays on the DISK. sled is a log, and the batch left in the directory's files
  the values sealed with the v1 key, without salt: whoever takes away the disk of a ledger
  migrated that way tries passphrases against them at the speed of a hash, and the right
  passphrase also opens version 2 (`SECURITY.md` §3.14; below, in E4, what was measured). Since
  §812 the migration is the E4 re-encryption with the same passphrase: a new directory with what
  is live, a rename and the old one deleted whole. The rest of this point still stands.
- A ledger with `meta:kdf` opened without a key gives `ParameterMismatch` of «cifrado en reposo»
  [encryption at rest].
- **The encrypted snapshot** carries the `0x02` mark, the ledger's header behind it and the sealed
  body: it is opened with the passphrase, without the ledger. The `0x01` mark —the body sealed
  with the version 1 key— is still imported and is no longer written. ⚠️ §821: «the ledger's
  header», or, if a layer imported with a key exports it, that of the snapshot it comes from, or
  one with a new salt if it came from v1 (entry 121, which §821 closes).

### The keystore (E3)

```json
{
  "version": "zkssl-keystore/2",
  "kdf": "argon2id",
  "kdf_sal": "0x<16 bytes>",
  "kdf_coste": { "m_kib": 65536, "t": 3, "p": 4 },
  "aead": "xchacha20poly1305, nonce 24B aleatorio antepuesto",
  "public_id": "0x…",
  "sealed": "0x<nonce 24 B ‖ cifrado>"
}
```

`keystore::save` writes version 2, always with a new salt, to a file alongside that is renamed
over it: a cut halfway leaves the previous keystore, not a truncated one. On Unix, after the
rename the directory is synced, so that the new name survives a power cut. A process that dies
between creating the file alongside and renaming it leaves it on disk,
`<keystore>.escribiendo-<pid>`, with 0600 and the spending key encrypted as in the new keystore:
it can be deleted, and since §774 the next write that ends well removes it, on Linux, if its pid
is no longer alive. `keystore::load` reads both, and **a `zkssl-keystore/1` opened with its
passphrase is rewritten in 2** —same passphrase, same spending key— before returning the wallet;
if it cannot be rewritten, it fails and says so (in «Compatibility», what that costs). With
another passphrase it is not touched. The SDK no longer derives anything on its own:
`zk_ssl::crypto::Kdf` does it, and `sha2` leaves its manifest.

### The re-encryption (E4)

⚠️ §812. A stage decided by the author; drafted in private, with the fix of the defect the first
paragraph tells, and built in the same seal (`spec/rfc/PROCESO.md` §«Fallos de seguridad»).

**The defect it fixes.** The E2 migration overwrote each value in a `sled::Batch` on the same `sled`
and did `flush`. That changes what is read whole or nothing, but sled is a log: the overwritten
values stay in the directory's files until sled reuses their segment, and that may never happen.
Measured in §812 on sled 0.34.7, the one in `Cargo.lock`, by searching the ledger's files for the
bytes of each value: with a ledger of two accounts, its 25 values sealed with the v1 key were still,
all 25, in the files after the batch, after 200 more batches and after three reopenings (on the code
from before §802; since it, the same ledger carries 26, with the root of the retired identities, and
the in-place batch leaves the 26);
with one of 702 accounts, 1425 of 1425 after the batch, 1356 after 200 more batches, and none after
three reopenings, when sled rebuilt its segments (whether it overwrote them or returned them as free
blocks to the file system was not measured). With the two-account one the 25 stayed after three
reopenings: one cannot count on reopening to clean. How long the residue lasts depends on what the
ledger writes afterwards: in a small or quiet one, it does not go away. The v1 key is
`SHA-256(dominio_v1 ‖ frase)`, without salt: whoever takes away the disk of a migrated ledger tries
passphrases against that residue at about 159 ns per attempt (§702), the AEAD tag confirms the right
one for them, and that passphrase also opens version 2. The test of §702 looked at what sled
returns, not at the files; the hardening of this RFC did not protect those disks, and no document
said so. A re-encryption done the same way, with an in-place batch, would leave in the files
everything sealed with the old passphrase.

**What it does.** `SovereignLayer::recifrar(path, …, vieja, nueva)` —the parameters of
`open_encrypted`, because the ledger is opened with the old passphrase and `load` verifies it whole
before touching anything— re-encrypts the WHOLE ledger with a new passphrase and a new salt, with
the cost of version 2, and returns the layer opened with the new key. It changes neither the root
nor the content: the same keys and, with their values opened, the same bytes; a re-encrypted ledger
is also opened by the code of §702, with the new passphrase. The `nueva` key has to carry its
passphrase: one already fixed cannot take another salt and gives `Malformed` before touching
anything. The same passphrase is admitted: it leaves a new salt and the directory clean, but it does
not protect the copies. A version 1 ledger is migrated first, with the old one, and then
re-encrypted. The E2 migration is this same piece with the same passphrase.

**The names**, reserved: P is the ledger; R, `<P>.recifrando`; V, `<P>.retirado`; in the same
directory, formed with the ledger's name and never by pasting text onto the path (with a path ending
in `/`, pasting would land inside the ledger). A path without a name —`.`, `..`— has no siblings,
and re-encrypting or migrating refuse. ⚠️ §816: and a third, C, `<P>.cerrojo`, the ledger lock
(below, «The race, and the ledger lock»). A path without a name has no lock either, and through it
nothing is opened or re-encrypted —`Malformed`, without touching anything—: it can name the ledger,
as `<P>/blobs/..` does, and it would be opened without a lock. ⚠️ §828: and one more, in passing,
`<P>.sonda.<pid>.<n>`: an empty file that `recifrar`, on Unix, creates and deletes before taking C
—if P is a sled directory; if not, it does not create it—, to learn from its owner who is
re-encrypting; it stays if the process dies between creating and deleting it, and nobody looks at it
or deletes it; and if it cannot be deleted —a parent with `chattr +a`—, `recifrar` refuses naming
it, without taking C, and it stays too, until it is deleted by hand (`SECURITY.md` §3.17).

⚠️ §816: the reserved names, together:

| name | what it is | created by | removed by |
|---|---|---|---|
| P, the ledger | a sled directory | the first opening | the layer does not delete it: step 5 renames it to V, and step 6 puts R in its place |
| R, `<P>.recifrando` | a sled directory: the copy sealed with the new key, until step 6 | the re-encryption, in step 1 | the re-encryption, if it fails before the commit; the next opening, if P is there, which deletes it, or if P is missing and V is there, which puts it in P's place; or step 6, which also puts it in P's place (⚠️ §828: the opening, not if R —with its `db`, its `conf` and its `blobs/`— belongs to another owner than the ledger —the owner of its directory and of its `db`, its `conf` and its `blobs/`, or those of V if P is missing—: it refuses without touching anything, `SECURITY.md` §3.17) |
| V, `<P>.retirado` | a sled directory: the ledger with the old key, from steps 5 to 7 | step 5, `rename(P, V)` | step 7, or the next opening |
| C, `<P>.cerrojo` | an empty file: the ledger lock, with an exclusive `flock` | the first opening or the first re-encryption, if it is not there, with `create_new`; on Unix, with 0600. The opening first creates the ledger's directory if it is not there, as `sled::open` does; the re-encryption creates no directories, and if there is neither ledger nor siblings it refuses without creating C (⚠️ §828: and, on Unix, if whoever re-encrypts is not the ledger's owner, or the ledger is not of a single owner, it refuses before taking it, without creating it, if it can tell; if it could not create the probe, it refuses step 1, after taking it, and after creating it if it was not there; the opening with a key by another user does not: it creates it before the migration's refusal, `SECURITY.md` §3.17) | the layer, never: it neither renames nor deletes it (it can be done by hand, and what happens then is in «What it does not do») |

**The steps.** The ledger stays open, with its lock, until step 7 (⚠️ §816: it said «so that nobody
opens it in the middle», and it did not prevent it: sled's lock goes with the inode, which after
step 5 is V's, and none is left in P; since §816 it is prevented by the ledger lock, which
`recifrar` holds from before step 0 to after step 8, and which it checks, before step 5, is still in
its place): after step 5 sled writes through its descriptor, in V's inode, and after opening it only
uses the path for the blobs, which the ledger does not have.

| step | what | if it fails |
|---|---|---|
| 0 | P is a real directory, not a link, and carries only sled's things (`conf`, `db`, `blobs/`, `snap.*`); a single sled tree; the new key carries a v2 header; V does not exist | `Err`, without writing anything |
| 1 | create R —it fails if it already exists—, with 0700, which it keeps until the end of step 4: while it holds the files sled creates with the umask, only its owner gets into it. ⚠️ §828: and, on Unix, if R —which is born the process user's— does not belong to the owner of the ledger's directory and of its `db`, its `conf` and its `blobs/`, or is no longer the empty directory that was born, it is deleted and the step refuses, before copying anything: the ledger of another owner, or one that is not of a single owner, is neither re-encrypted nor migrated; but after the layer opens the ledger with sled, which leaves in it what any opening by whoever opens leaves: that the guard goes here is the session's, and reversible —the option the author chose names no step—, and what follows from it is in `SECURITY.md` §3.17, «Lo que no hace» [what it does not do]. In the re-encryption, `recifrar` compares the same thing before taking the ledger lock and opening it, and refuses without creating, opening or writing anything of the ledger or of its lock, if it can tell: this guard stays as a backup, for when that comparison could not create its probe; in the migration, which does not know the ledger is v1 without opening it, it is the only one | `Err`; if R already existed, it is not touched |
| 2 | copy to R each value of P, opened with the old key and sealed with the new one; `meta:geometry_v7` and `meta:migrated` as they are; the new header; one batch, `flush`. A value that does not open with the old key stops it and is named | R is deleted; P intact, with the old one |
| 3 | verify R by reading it from disk: the same keys as P plus the header, what goes in the clear the same, and each sealed value, opened with the new key, equal to P's opened with the old one. The roots (`root:*`) are values of the ledger and go inside; `pamt:` and `pmeta:`, which no root covers, too; and the retired identities of §802, `retir:{k}` and their root `root:retir`, which go sealed | likewise |
| 4 | the modes of P's files pass to R's, and then that of P's directory to R (⚠️ §828: and only the modes: the group was that of whoever re-encrypted. Since §828, each file of R, and R, last, take the group of its namesake in P —one that has none, that of `db`—, through a descriptor that does not follow a link and only if R is still the directory that was born in step 1, and if that cannot be done, they are left without group or other bits; then, the modes; `SECURITY.md` §3.17); sync each file of R, `R/blobs`, R and the parent | likewise |
| 5 | **`rename(P, V)` and sync the parent: the commit** | if the `rename` itself fails —EBUSY on a mount point, or Windows with open files—, R is deleted and P stays intact |
| 6 | `rename(R, P)` and sync the parent | `Err` «pasado el compromiso» [past the commit], which says with which passphrase the ledger stays —with the new one when re-encrypting; with the same one, already in version 2, when migrating—, and the next opening finishes it; until then, the `.retirado`, if it is still on disk, keeps what was sealed with the old one. ⚠️ §816: until §816, not if an opening had looked at the siblings before step 5: it created an empty ledger in P, this `rename` failed because of it, and the next opening refused or took away the last copy (measured; below, «The race»). Since §816 no opening or re-encryption of the layer through the same name of the ledger opens or resolves between steps 0 and 8 (what it does not reach, in «What it does not do») |
| 7 | release the old ledger, delete V whole and sync the parent | likewise |
| 8 | reopen P and keep the new key; nothing is reloaded, because step 3 checked that the copy says the same | likewise |

**What a death leaves, and what the opening does.** Every opening —`open`, `open_encrypted` and
`recifrar`, also that of the ledger the node opens in the clear— looks, before opening, whether the
siblings are there; ~~without them, it does nothing more~~ (⚠️ §816: since §816, before looking it
takes the ledger lock —and, if it is not there, it first creates the ledger's directory, as
`sled::open` does—, holds it until it returns with the ledger open, and checks before `sled::open`
that it is still in its place).

| on disk | it died | the opening | opens with |
|---|---|---|---|
| P | — | nothing | its own |
| P + R | in steps 1 to 4, or before 5 | deletes R, which only carries what was sealed with the new one (⚠️ §828: if R —with its `db`, its `conf` and its `blobs/`— belongs to the ledger's owner —the owner of its directory and of its `db`, its `conf` and its `blobs/`—; if not, it refuses without touching anything, `SECURITY.md` §3.17) | the old one |
| V + R, without P | between 5 and 6 | R becomes P —it is whole, verified and synced, because 3 and 4 come before 5—, and V is deleted (⚠️ §828: if R —with its `db`, its `conf` and its `blobs/`— belongs to V's owner —the owner of its directory and of its `db`, its `conf` and its `blobs/`—; if not, it refuses without touching anything: it would put another owner's ledger in the ledger's place, `SECURITY.md` §3.17) | the new one |
| P + V | after the rename of 6, or in 7 | deletes V, which is everything sealed with the old one | the new one |
| P + R + V, R alone, V alone, or V + R with an R without `db` | the re-encryption does not leave them; they are left, for example, by the code from before §812, which with V + R and without P creates an empty ledger in P (measured) | refuses with `Malformed`, which names the state; it touches nothing and never creates an empty ledger (⚠️ §816: in these states, no; but until §816 the opening created an empty ledger in P without going through this row, if a re-encryption reached its commit between its looking and its opening: that is the race, below) | — |
| a sibling that is not a real directory, or that carries something that is not sled's | something foreign | refuses without touching anything | — |

**The locks.** Before acting, the opening takes, with a bounded wait —ten times every 50 ms—, sled's
lock on `db` of each of P, R and V that exists: the same `flock` that sled 0.34.7 takes through fs2,
and `std`'s is compatible with it. If any is still taken, the ledger is open or being re-encrypted
in another process, and the answer is `Io`, without touching anything: the live re-encryptor holds
P's, which after step 5 is V's. If it cannot delete V, the opening fails and says so: it does not
open with the old one alongside without anyone seeing it. ⚠️ §816: it takes them and releases them
at once, so they tell how they were at the time of looking; they exclude no one while the opening
acts nor when it calls `sled::open`, and without siblings none is looked at. What excludes since
§816 is the ledger lock.

**The race, and the ledger lock** (⚠️ §816). Drafted in private with its fix (`spec/rfc/PROCESO.md`
§«Fallos de seguridad»). Measured on `b6994c3` by a reviewer, with two threads and injected pauses:
nothing held a lock between the opening's looking at the siblings and what it did afterwards, and
`sled::open` creates the ledger if it does not exist. (a) Without siblings: opening A looks —there
are none— and stops; a re-encryption B reaches its step 5, `rename(P, V)`; A calls `sled::open`,
which creates an EMPTY ledger in P, and opens `Ok` with no balance; B's step 6 fails because P
already exists, and its error says «la apertura siguiente lo termina» [the next opening finishes
it]; the next one refuses P + R + V, and the ledger, whole in R and in V, no longer opens without a
person. (b) With the ledger and its `.retirado` from a death after step 6: A looks at the locks
—free— and stops; B resolves the same, opens and re-encrypts up to its step 5, which puts the ledger
in a NEW `.retirado`; A deletes that one and creates an empty ledger in P; B's step 6 fails; and the
next opening with the passphrase B's error names deletes the `.recifrando` —the only copy— as a
leftover from before the commit. (c) With the ledger and its `.recifrando` from a death before the
commit —measured by a reviewer of the fix, also on `b6994c3`—: A looks and stops; B deletes that
`.recifrando`, re-encrypts up to its step 5 and leaves its own; A deletes B's and creates an empty
ledger in P; B's step 6 fails saying that the ledger stays with the NEW passphrase, and it does not;
and the next opening, with any passphrase, deletes V —the only copy— as a leftover from after the
commit, and with the old one opens `Ok` an empty ledger. In all three, A opens `Ok` an empty ledger,
without any error. In (b) and in (c) the ledger, with its balances, is lost whole, and both say so
equally badly: B's error names the new passphrase, with which the ledger no longer opens, and with
the previous one the opening opens `Ok` an empty ledger, without any error. It does not create
balance or authorise any spend. It reaches whoever uses the library with an encrypted ledger and two
openings at once, one of them a re-encryption or a migration from version 1 —opening a v1 is
migrating it—: the node and the cli open in the clear and do not re-encrypt, and the kit does not
depend on the layer. E4 brought it, in §812.

The fix is a lock on something that is not renamed: C, `<P>.cerrojo`, with an exclusive `flock`.
Every opening takes it —`open`, `open_encrypted`, the one `recifrar` does and the node's— from
before looking at the siblings until it returns with the ledger open and, if it was version 1,
migrated; and `recifrar`, from before looking at the siblings to the end —after the Argon2id of the
new passphrase, and before touching the disk—: the E4 re-encryption and the E2 migration only run
with it taken, and `resolver_restos` and the E4 piece ask for it as an argument. It is reentrant in
the thread that holds it —the opening with the old passphrase that `recifrar` does finds it taken—,
and stops being so on releasing it; another thread or another process waits for it as for sled's,
ten times every 50 ms, with the same constant, and if it does not take it gives `Io` with the lock's
name, without touching anything. Nothing is created through a link: if it is not there, it is
created with `create_new` —`O_CREAT|O_EXCL`, which does not follow a link in the last name—, and if
it is there it is opened without creating; a link that appears between looking and opening is
followed —without `O_NOFOLLOW`, which `std` does not give without `libc`—, without creating or
writing anything in its target, although it takes its `flock` for an instant, and it is refused by
its inode before touching anything: with the `flock` taken it is checked that the name is still that
file, by its device and its inode, and whoever holds it checks again before what cannot be undone:
the `sled::open` of the opening, which creates the ledger if it is not there, and the commit of the
re-encryption. If someone deleted or changed it while it was held, it refuses with `Malformed`,
without touching anything. Through a path without a name nothing is opened or re-encrypted, and
`recifrar` does not create the lock where there is neither ledger nor siblings. It does not cross
threads —it is not `Send`—: the list that makes it reentrant is that of the thread that took it.
With it taken, the opening does not create a ledger where there is a `.retirado` or a `.recifrando`
without a ledger: `resolver_restos` resolves it or refuses, and no opening or re-encryption of the
layer through the same name changes anything between its looking and its opening. If the process
dies, the system releases it: a dead process does not leave the ledger locked. The tests measure it,
the races with the same pauses: the three races, two re-encryptions at once, an opening that arrives
while another re-encrypts or migrates, the lock, which no step renames or deletes, the reentry,
which does not linger after returning, a new ledger in a directory that does not exist, a lock that
changes while it is being taken, one that is deleted or changed while it is held, a path without a
name, which is not opened, a re-encryption without a ledger, which does not leave the lock, and the
lock, which does not cross threads; each piece, with its mutant (§816).

**The contract.** `Ok`: the ledger is whole with the new passphrase, and neither it nor its siblings
keep anything sealed with the old one. `Err`: the ledger is whole with ONE of the two passphrases,
the old one if it failed before the commit and the new one if the message says the re-encryption
passed it. With a version 1 ledger, `recifrar` migrates it first, and that migration has its own
commit: if it fails past that one, the message says so —«la MISMA frase con que se abrió» [the SAME
passphrase it was opened with]—, and the ledger stays with the old one, already in version 2. A
death halfway: the next opening resolves it, and the ledger opens with one of the two; the caller
tries both. ⚠️ §816: until §816, all this only with nobody else at the same time: with an opening in
between, the `Err` of step 6 left the ledger in R and in V without opening, or lost (the race,
above); since §816 it holds with the layer's openings and re-encryptions at the same time, and what
the lock does not reach is in «What it does not do». If it died before the commit, the opening goes
back: the ledger stays whole with the OLD passphrase, and what was sealed with it stays in the
directory until the re-encryption is repeated and finishes; whoever changes a compromised passphrase
does not have it changed because the ledger reopens. After the commit, the opening finishes
forwards, with the new one. The durability is that of the file system's `fsync`; outside Unix the
directory is not synced.

**What it costs.** Writing in the parent directory, and room for a copy, which comes out compacted:
what is live, without what was overwritten (measured: a ledger of 202 accounts, after 200 batches,
goes from 1.05 MB to 0.53 MB in the directory, and one of 902, from 2.10 MB to 1.06 MB). Two
Argon2id: about 0.85 s with the figure of §702; on the port's machine, a whole re-encryption of a
ledger of 65 keys took 438 ms. The migration of a v1 ledger of 702 accounts took 1.05 s, with its
Argon2id. With a version 1 ledger, two copies: it migrates and re-encrypts. ⚠️ §816: it said «Until
step 7 nobody else can open the ledger, and if someone has it open, `recifrar` gives `Io` without
touching anything; from when it releases the old one, in 7, another process can open it, already
re-encrypted, and `recifrar` gives the error of past the commit. It is not tested with another live
process». The first part was not true: sled's lock did not prevent it (the race, above); and the
last no longer is (below). Since §816, while `recifrar` lasts no other opening or re-encryption of
the layer through the same name opens or re-encrypts the ledger: the one that arrives waits for the
ledger lock, ten times every 50 ms, and gives `Io` without touching anything; and if someone has it
open, `recifrar` gives `Io` without touching anything. Measured in the suite with threads of the
same process, which the `flock` excludes as it would another process; and outside it with two real
processes, by a reviewer on the first fix of §816 and by the session, with the same tests, on the
fix of each round, the last one too: they exclude each other in both directions, and one killed with
SIGKILL with the lock taken does not leave the ledger locked.

**What does not change.** No format: the sealed values, the 29-byte header, `meta:kdf`, the keys in
the clear, the cost of version 2, the snapshot's marks, the keystore's JSON, `zkssl/0.4` and its
vectors. The only new things on disk are the two reserved names and the rule with which the opening
reads them (⚠️ §816: and the third, the ledger lock).

**What it does not do, and says so.**

- **The node** opens its ledger in the clear (`SovereignLayer::open`, in
  `crates/zk-ssl-node/src/main.rs`): it has no passphrase to change nor v1 residue to clean. Wiring
  its encryption is not within what the author asked for, the library's E4 (D-F). Nor is `recifrar`
  exposed in the cli or in the SDK. But its opening is the layer's: it also resolves the two
  reserved names next to its ledger, with the same deletion and the same refusal (in
  «Compatibility»).
- **Encrypting a ledger in the clear.** With a passphrase and without a header, a ledger with data
  is taken for a version 1 one, and `recifrar` gives today's error, «contraseña incorrecta o
  manipulación» [wrong password or tampering]. It is the same piece, and it is left for when the
  node is wired.
- **Overwriting.** Deleting V is deleting, not overwriting, by the author's decision (D-I). What is
  kept by the blocks the file system frees, an SSD —its FTL, wear levelling, TRIM—, a copy-on-write
  or log-structured system (btrfs, ZFS, APFS, F2FS) and its snapshots, those of LVM, of a virtual
  machine or of the cloud, or the journal with `data=journal`, the library does not reach, and it is
  not promised.
- **The copies.** A copy of the directory, a snapshot exported before or a hard link to a file of
  the ledger made outside keep opening with the old passphrase wherever they are; those of a ledger
  migrated by §702 also carry the v1 residue.
- ⚠️ §816, **the earlier code does not take the lock.** A process with the code from before §816 on
  the same ledger at the same time is not excluded, and with it the race remains. Going back to that
  code leaves the `.cerrojo` alongside, without effect.
- ⚠️ §816, **the lock is that of the name through which it is opened.** Two spellings of the path of
  the same directory give the same file; but whoever opens the ledger through a link to its
  directory takes the one next to the link, not the one next to the ledger. Re-encrypting and
  migrating through a link are refused, and in the gap of a re-encryption `sled::open` does not
  create a directory through a broken link: measured by a reviewer of the fix, between steps 5 and 6
  it gives «File exists» and creates nothing, and between 7 and 8 it opens through the link the
  ledger already re-encrypted, without harm (outside the suite).
- ⚠️ §816, **deleting or replacing the lock by hand.** With the ledger open or being re-encrypted,
  deleting `<libro>.cerrojo` —or replacing it, which is the same: restoring a copy, an `rsync`— lets
  in another, who creates or takes a new one, of another inode. On the first fix of §816 a reviewer
  measured that, with an opening stopped before `sled::open`, it was race (a) again. Since the fix,
  whoever held it notices before `sled::open` and before the commit, and refuses without touching
  anything (in the suite); there remains the window between that check and the call that follows,
  and outside Unix it is only checked that the name is still a file. With a re-encryption already
  past its commit, sled's lock on its `.retirado` keeps excluding (measured by the reviewer). It is
  not deleted while anyone is there.
- **The snapshot.** ⚠️ §820: it said «`export_snapshot` truncates the previous one in place»; since
  §820 it replaces it whole, with a temporary file in its same directory,
  `{instantánea}.exportando`, and a rename; the temporary file a dead export leaves carries what the
  target would carry and is not swept. And a layer imported from an encrypted snapshot is born
  without a key and, if it is exported again, comes out in the clear. They are E2's, and they stay
  outside this stage: they are entries 120 and 121 of `BACKLOG.md`, which §812 opens; entry 120 is
  closed by §820. ⚠️ §821: and entry 121, by §821, by the author's decision: the layer
  `import_snapshot_with_key` returns keeps the snapshot's key, fixed to its header, and is
  re-exported encrypted, with the same passphrase; and by the session's decision, reversible, that
  of a v1 one, in v2 with a new salt, and with a key already fixed without the passphrase it is not
  imported (`SECURITY.md` §3.14 and §3.16).

**The remedy for a ledger already migrated.** A ledger the code of §702 migrated —from `2ac6db4`, on
07-10-2026, until §812— can carry the v1 residue in its files, and a small or quiet one does carry
it (above). The library does not tell it apart: that migration only wrote `meta:kdf`. The remedy is
`SovereignLayer::recifrar` **with a new passphrase**: the directory is left clean of what was sealed
with v1 and with the old passphrase (a test measures it). With the same passphrase the directory is
also left clean, but any copy made since the migration, and the free blocks, keep yielding the v1
passphrase at the speed of a hash, and that passphrase opens the ledger.

## Decisions

Taken in the session of §702 within what the author approved —Argon2id or scrypt, with salt and
cost, with a format version and migration of what exists—, and reversible while this RFC is not
ACCEPTED.

- **D-A. Argon2id, not scrypt.** It is the first recommendation of the OWASP sheet and the one this
  number had reserved since §206.
- **D-B. One fixed cost per version, that of the second option of RFC 9106.** Measured: about 0.43 s
  per derivation with the machine under load, and there is one per opening. The RFC's first option
  (2 GiB) is not taken: the keystore is opened on a holder's laptop.
- **D-C. The domain as associated data, with new domains for version 2.**
- **D-D. Opening is migrating**, in the ledger and in the keystore, without a separate step: a weak
  format that is only migrated if someone remembers stays weak. In the ledger, after verifying it;
  in both, without touching anything with another passphrase.
- **D-E. `argon2` 0.5.3**, because of the kit (above).

⚠️ §812: those of E4. D-F is the author's, and so is not overwriting, among what D-I says is not
done, and the sweep of D-H; the rest —the marker included—, of the session that drafted it in
private, and reversible, like the above, while this RFC is not ACCEPTED.

- **D-F. Changing the ledger's passphrase, separately and as a stage of this RFC** (the author's,
  §812). It is K1 of `doc/ROTACION_DE_CUENTAS.md`, and it resolves its D-4: it goes apart from the
  single rotation design of entry 52 of `BACKLOG.md`, as E4 of this RFC and not as an RFC of its
  own: he resolved it on asking for it. Wiring the node's encryption at rest is not within what he
  asked for.
- **D-G. A sibling directory and a rename, not an in-place batch.** An in-place batch leaves in the
  files what it overwrites (above); copying what is live to a new directory and deleting the old one
  whole does not, within what the library controls.
- **D-H. The copy is verified in the clear and read from disk before the commit**; before it, a
  failure goes back, and after it, it goes forwards. The opening refuses what the re-encryption does
  not leave, deletes nothing that is not sled's, and does not open if it cannot delete the retired
  one. And the sweep the author decided; that the import fails, without importing anything, if it
  cannot delete the `.tmp-descifrada` it has alongside —one does not import with the cleartext
  alongside without anyone seeing it—, and that it only deletes a regular file —a directory or a
  link with that name was not written by that code, and they are not touched—, is the session's,
  reversible. ⚠️ §819: it said «Neither that refusal nor the warning through standard error have a
  test; the deletion has one». Since §819 both have one: the refusal, with the deletion itself made
  to fail, and the warning, read from what the thread keeps in the tests; that it goes to standard
  error the test does not see: outside the tests it is a one-line `eprintln!`, without a test, and
  leaving it so is the session's, reversible.
- **D-I. The remedy for a ledger already migrated is by hand** —`recifrar` with a new passphrase—,
  and the same passphrase is admitted (the session's). Two things are not done. The first the author
  decided (§812), on the session's recommendation: **there is no overwriting before deleting** —it
  is deleted, and it is said that the file system's free blocks, an SSD, a copy-on-write system or
  the journal can keep copies: overwriting cannot be measured here, does not hold on those media and
  would invite promising—. The second is the session's, and the author has not decided it: **no
  marker is placed** to distinguish the ledgers already migrated, which would be a new format and
  would suggest that that is enough. The marker is reversible: it can be placed later, with its
  format and its seal.
- **D-J. The ledger lock is a sibling file, `<libro>.cerrojo`, with an exclusive `flock`** (⚠️ §816;
  the session's, by the author's delegation, and reversible while this RFC is not ACCEPTED). Three
  candidates: (i) that file, a third reserved name; (ii) a `flock` on the parent directory, without
  a new name; (iii) another. (ii) is discarded: it would lock at once all the ledgers of the
  directory while a re-encryption lasts —two Argon2id and a copy—, and the opening of another ledger
  that arrived in between would fail at the cap without having anything to do with it; on Windows
  `std` does not open a directory as a file nor does `LockFileEx` lock one (deduced, not tested
  there); and a shared directory —`/tmp`, the user's folder— can be held by anyone. Of (iii): sled's
  lock, retained, does not serve, because it goes with the inode of `P/db` and P is renamed; a file
  inside P, the same; a process `Mutex` does not exclude another process; and checking after opening
  —open, look whether siblings appeared, undo— forces deleting a newly created ledger in the middle
  of the race. A `flock` and not a file created with `O_EXCL`: if the process dies, the system
  releases it, and a dead process does not leave the ledger locked. It fails closed: a lock that is
  not taken, or that cannot be created, is `Io` with its name, and without it nothing is opened or
  re-encrypted. The price, in «Compatibility». After the two reviewers of the fix, also the
  session's and reversible: the opening creates the ledger's directory if it is not there, as
  `sled::open` does, and `recifrar` does not, because it requires a ledger that already exists; the
  lock is created with `create_new` and, with the `flock` taken and again before `sled::open` and
  the commit, it is compared by its device and its inode with what the path names, instead of
  opening it with `create`, which follows a link; and a `.cerrojo` that cannot be opened for writing
  is not opened just for reading, except on a read-only file system (why, in «Compatibility»). And,
  also the session's: through a path without a name nothing is opened —instead of forming the lock
  from the resolved path, which requires that it exists—, `recifrar` does not create the lock where
  there is neither ledger nor siblings, and the lock is not `Send`.

## Compatibility

- **The wire does not change.** Neither the version, nor a method, nor a vector.
- **Backwards, yes:** what the previous code wrote is opened. §702 measured it with files written by
  the code of `590caae`: a v1 keystore, an encrypted ledger and an encrypted snapshot.
- **Forwards, no.** A migrated keystore or ledger is not opened by the previous code: the keystore
  because of its `version`, and the ledger because the version 1 key does not open what version 2
  seals («contraseña incorrecta o manipulación»). The migration has no way back.
- **A v1 keystore is only opened where it can be rewritten.** Opening is migrating (D-D), and if one
  cannot write next to the file —a read-only medium, a directory without write permission—,
  `keystore::load` fails with any passphrase and says why. Before §702 that keystore opened. It is
  copied to a directory where one can write and opened there. A read without migrating for those
  media would be another decision, the author's.
- ⚠️ §812, **E4.** The E2 migration and the re-encryption write next to the ledger: one needs to be
  able to write in its parent directory, and room for a copy, so a version 1 ledger whose parent
  directory cannot be written, or without room for a copy, no longer opens: until §812 it migrated
  in place. On a read-only medium it did not open before either, because sled 0.34.7 opens `db` for
  writing (`src/config.rs`). The ledger's path cannot be a link —it is refused: renaming and
  deleting the link would leave the real ledger as it was—, and the directory cannot carry anything
  that is not sled's. `<libro>.recifrando` and `<libro>.retirado` are reserved names, and they are
  of no use for copies: there is no mark that distinguishes a sibling the re-encryption left from a
  directory someone named so, and **the opening DELETES** a sled directory with one of those names
  next to the ledger, if nobody has it open —a backup too—; what is not sled's it does not touch. It
  holds for every ledger the layer opens, also the one the node opens in the clear
  (`SovereignLayer::open`): on starting, the node resolves a sled `.recifrando` or `.retirado` it
  has alongside like any opening —it deletes them, or, if the ledger is missing and its `.retirado`
  is there, puts the `.recifrando` in its place—, and a state the re-encryption does not leave
  prevents it from starting. The layer declares `rust-version = "1.89"`: the lock uses
  `File::try_lock`, stable since Rust 1.89, like the guardian since §709; the sdk, the cli and the
  node already declared it. **No downgrading with leftovers on disk**: the code from before §812
  does not know the siblings, and with a `.retirado` and a `.recifrando` without a ledger it creates
  an empty ledger (measured). A re-encrypted ledger without leftovers alongside is opened by the
  code of §702 with the new passphrase; by the code from before §802, as any ledger since §802,
  without enforcing its retired identities (`SECURITY.md` §3.13). On Windows, which does not rename
  a directory with open files, the re-encryption and the migration fail without touching anything;
  it is not tested there.
- ⚠️ §816, **the ledger lock.** Every opening creates `<libro>.cerrojo` if it is not there —empty;
  on Unix, with 0600, so that only its owner can take it— next to every ledger the layer opens, also
  the one the node opens in the clear, and never deletes it. If the ledger's directory does not
  exist, the opening creates it first, as `sled::open` created it: the first fix of §816 did not,
  and a new ledger in a new directory —the node's first start, or the cli, with `--ledger` on a new
  path— gave `Io` with the lock's name without creating anything (both reviewers of the fix measured
  it; there is a test). `recifrar` creates no directories nor, if there is neither ledger nor
  siblings, the `.cerrojo`: it refuses with `Malformed` and leaves nothing (the fix, until its
  reviewers, left it created next to a path without a ledger; there is a test). A ledger whose
  parent directory cannot be written and that does not have its `.cerrojo` no longer opens —`Io`
  with the lock's name, without touching anything—, and until §816 it opened: the remedy is to
  create it once, empty and with write permission for whoever opens the ledger, who afterwards does
  not need to write in the parent. If the parent is on a read-only file system —a ledger that is a
  mount point inside a read-only one—, the `.cerrojo` that is already there is opened only for
  reading, because there nobody can write or change it; and if it is not there, the `Io` says it has
  to be created first, empty, where one can write. The fix, until its reviewers, opened it for
  writing there too, and gave `EROFS` even if it was there: measured outside the suite with the cli
  and a read-only mount, which before did not open with the `.cerrojo` or without it, and now opens
  with it and without it gives that `Io`. A `.cerrojo` that whoever opens cannot open for writing
  —another user's and with 0600: the one `root` leaves if it opens with the cli, before the node,
  the node's ledger that did not yet have its own— leaves the ledger unopened, with an `Io` that
  names the lock and says the remedy: with the ledger closed by everyone, whoever can gives it to
  them (`chown`) or deletes it, and the next opening creates it. Until §816 it opened. Outside a
  read-only system, it is not opened only for reading when write permission is missing: with 0600,
  another user does not read it either (measured outside the suite, without `CAP_DAC_OVERRIDE` or
  `CAP_DAC_READ_SEARCH` and with the file of another owner: with 0600, neither for reading nor for
  writing; with 0644, for reading yes), a `.cerrojo` that another user can read they can take and
  leave the ledger unopened, and on NFS the client emulates `flock` with `fcntl`, which for an
  exclusive lock requires the file open for writing (Linux's `flock(2)` says so; it is not tested
  here). A ledger on a read-only medium did not open before either, because sled writes in it.
  Through a path without a name —`.`, `..`, or one ending in `..`— no ledger opens any more:
  `Malformed`; until §816 whatever it named opened. A `<libro>.cerrojo` that is not a file —a
  directory, a link— makes the opening refuse with `Malformed`, and nothing is created through a
  link: the first fix of §816 opened it with `create`, which follows the link, and a reviewer
  measured that, with a link that appeared and disappeared in a loop, the opening created its
  target, a file outside the ledger's directory. Two openings of the same ledger at the same time:
  the second gives the lock's `Io` if it arrives while the first opens, and sled's if it arrives
  afterwards, as until §816. On Windows `std`'s `flock` is `LockFileEx`; it is not tested there. The
  code from before §816 neither takes it nor looks at it: a `.cerrojo` alongside changes nothing for
  it, and against a process with that code at the same time there is no exclusion.

## Security

- **API principle** (the spending key does not travel): it does not touch it. It hardens where it
  sleeps.
- **What changes:** each offline attempt costs a 64 MiB Argon2id instead of a SHA-256, and each file
  has its salt: no table serves for two. ⚠️ §812: not in the directory of a ledger migrated by the
  code of §702 —from `2ac6db4` until §812—, which kept in its files what was sealed with the v1 key,
  without salt, and there each attempt still cost a SHA-256 (E4, above, and `SECURITY.md` §3.14).
  Since §812 the migration does not leave it, and re-encrypting with a new passphrase cleans the
  directory of a ledger already migrated.
- **What does not change:** a weak passphrase is still weak; the cost multiplies the price of each
  attempt, it does not reduce the attempts. Whoever has the running process has the derived key in
  memory while the ledger or the keystore are open. On release, the passphrase, the derived key, the
  Argon2id memory and the key the cipher keeps are wiped (`chacha20poly1305` 0.10.1 wipes it in its
  `Drop`); what the system copied before, not. The salt comes from the system generator, like the
  nonces (`SECURITY.md` §3.11). The node does not open encrypted ledgers (`SovereignLayer::open`):
  the encrypted ledger is used by the layer as a library, and the keystore, by the SDK and the cli.
- ⚠️ §816, **the E4 race.** Until §816 an opening at the same time as a re-encryption or a migration
  could leave an empty ledger in the ledger's place, which the opening opened `Ok` without any
  error, or lose it whole (above, «The race, and the ledger lock»; `SECURITY.md` §3.14). It did not
  create balance or authorise any spend: what it broke is that the ledger continued. The ledger lock
  closes it.
- ⚠️ §812, **what the disk keeps.** Within what the library controls —the ledger's directory and its
  two siblings— nothing sealed with the old key remains after a migration or a re-encryption that
  finish well, nor after the opening that follows a death in the migration or a death of the
  re-encryption past its commit (the tests measure it with a death at the end of each step), or an
  error past the commit: until that opening, the `.retirado` stays on disk with what was sealed with
  the old one (⚠️ §816: until §816, only if nobody opened the ledger at the same time: after race
  (a) the next opening refused, and the `.retirado` stayed; above, «The race»). If the re-encryption
  died before its commit, the opening goes back and the ledger stays whole with the old passphrase:
  what was sealed with it stays there until the re-encryption is repeated and finishes. Outside
  that, and not promised: the blocks the file system frees on deleting —also those of the v1
  keystore that E3 rewrites with a `rename` (deduced, not measured), the segments sled truncates,
  and the previous snapshot, which `export_snapshot` replaces whole with a rename (⚠️ §820: it said
  «which `export_snapshot` truncates in place»; its blocks are freed the same way, and a dead export
  leaves `{instantánea}.exportando` alongside)—; an SSD, a copy-on-write or log-structured system,
  their snapshots and the journal; the copies of the directory and the snapshots exported before,
  which keep opening with the old passphrase; a hard link made outside; and memory —swap,
  hibernation, dumps—. And a `{instantánea}.tmp-descifrada`, IN THE CLEAR, if a process died halfway
  through an import between `5d2833e` (28-07-2026) and `c87d7d6` (§774, 08-10-2026): §774 removed
  the writer and did not sweep; since §812, by the author's decision, the import of that snapshot
  sweeps it —if next to it there is a file with that name, it deletes it and says so through
  standard error—; that, if it cannot, the import fails with an error that names it, without
  importing anything, is the session's (⚠️ §819: with a test since §819, like the warning, read from
  what the thread keeps in the tests; that it goes to standard error has no test). That of a
  snapshot that is not imported again is looked for next to it and deleted by hand. Changing the
  keystore's passphrase —`load` and `save`— does not change the spending key: an old copy opened
  with its old passphrase gives the same key.

## References

- `AUDITORIA.md` §199 (the keystore), §206 (the reservation of the number), §286 (the registry of
  domains), §694 (the pins and the kit's closed list), §702 (this RFC, sealed), §774 (the on-disk
  leftovers of §702 that were removed then), §812 (E4 and the fix of the migration's residue,
  drafted in private), §816 (the E4 race and the ledger lock, drafted in private).
- `crates/zk-ssl/src/crypto.rs`, `crates/zk-ssl/src/persistence.rs`,
  `crates/zk-ssl/src/snapshot.rs`, `crates/zk-ssl-sdk/src/keystore.rs`.
- sled 0.34.7, the one in `Cargo.lock`: `src/config.rs` (the `flock` lock on `db`, through fs2, and
  the names `conf`, `db`, `blobs/` and `snap.*`), `src/pagecache/snapshot.rs` (the `snap.*` are
  written on opening) and `src/pagecache/logger.rs` (a fragment becomes a blob above a quarter of
  the segment).
- RFC 9106, *Argon2 Memory-Hard Function for Password Hashing and Proof-of-Work Applications*: §5.3,
  the Argon2id vector that a test of the layer reproduces, with the label of the file
  `kats/argon2id` of the reference implementation (P-H-C/phc-winner-argon2); and §4, the parameters.
- argon2-cffi 25.1.0, `argon2/profiles.py`: `RFC_9106_LOW_MEMORY`.
- OWASP, *Password Storage Cheat Sheet*, §Argon2id: read from `OWASP/CheatSheetSeries`, the Markdown
  of `cheatsheets/Password_Storage_Cheat_Sheet`, at commit `29994dd`.
