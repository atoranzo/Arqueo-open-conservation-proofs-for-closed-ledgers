# Reproduction run on 2026-09-28 (WORK IN PROGRESS)

> **Español:** [`doc/ecst/borrador/REPRODUCCION.md`](./REPRODUCCION.md) is the original of this
> page, in Spanish; it prevails wherever the two differ. This translation is of commit `7766efb`
> (audit entry §917) and was produced by the project's assistant, a Claude Code session, in audit
> entry §918 (`GENAI.md`): it is a generated artefact, not text written by the author. Line
> citations of the form `file:N` refer to the Spanish files.

> ⚠️ Working draft, not a publishable result. It was run by a generative AI assistant
> (Claude, by Anthropic) in an ephemeral container, not by the author on his machine: see `GENAI.md`.
> The figures here are from ONE machine and are given with their environment; they do not replace those of `AUDITORIA.md`.

Environment: Firecracker microVM (kernel 6.18.44-fc-v37), 4 vCPU "Intel(R) Xeon(R) Processor @ 2.10GHz", 15 GiB RAM,
ext4 root on /dev/vda (virtio), /dev/shm tmpfs. rustc/cargo 1.94.1 (2026-03-24). Python 3.
Arqueo tree: d531c8016800573f1a6b3e3e4c699a3551da20cd (branch claude/awesome-pasteur-u08nq9 = main at that commit).
hbs-state tree: a960828bf0e810c30b49640c9bcb8d64f9c9557f (shallow clone of github.com/atoranzo/hbs-state).

## Arqueo
- `cargo test --release -p zk-ssl --lib t1_` -> 3 passed (t1_divergencia_localizada, t1_verify_chain_caza_sustitucion,
  t1_cabeza_ata_la_historia), 0 failed. The test uses N=12 entries and alters the K=5 one (constants in crates/zk-ssl/src/log.rs:1140-1141),
  with synthetic "proofs" (bytes b"honesta"/b"mentira"), no STARK; it recomposes with chain_digest_v2 from the raw fields.
- `cargo test --release -p zk-ssl-guardian` -> 26 passed, 0 failed (includes en_tmpfs_se_niega_a_operar, en_disco_de_verdad_si_opera,
  con_el_contador_en_uno_y_la_clave_en_cero_falla_cerrada, solo_la_clave_adelantada_no_admite_matiz).
- Independent verifier `zk-ssl-verify` (built in release in this tree): the vectors of the README kit give the expected
  outputs (posicion-v2 -> 0 VERDE; consumo -> 0 VERDE; rechazo-n-adulterado -> 1 ROJO; rechazo-cons-ausencia-ya-estaba -> 1 ROJO;
  conflicto -> 0 VERDE "DETECCION, no prevencion"; rechazo-conf-camino-no-sube -> 1 ROJO).
- `bash tools/conformidad.sh target/release/zk-ssl-verify [MANIFIESTO]` over the eight catalogues: paquete 70/70, consumo 14/14,
  conflicto 16/16, rechazo 84/84, edad 11/11, pendiente 9/9, pago 9/9, prenda 9/9 = 222/222 entries say what they should
  (binary 4305941dff1ab547).
- `cargo run --release -p zk-ssl-cli -- simulate --amount 250000` (full output in `simulate-2026-09-28.log`; ONE run, sandbox with deterministic keys, seed 0xa11ce):
  real STARK proofs; delegated issuance (mint) with two custodians: 64.3 KB [417 ms] and 65.3 KB [345 ms]; send 77.6 KB [970 ms];
  claim 76.7 KB [1198 ms]; apply 4-6 ms each; "cadena de transiciones integra (6 entradas)". It is a single run in a
  shared microVM: it is NOT a measurement with dispersion and does not replace those of AUDITORIA.md. "KB" of the CLI = bytes/1024 (crates/zk-ssl-cli/src/trace.rs:254), that is, KiB. Send+claim = 77.6+76.7 KiB ~ 158 000 bytes,
  WITHIN the published band of one payment PUBLICADA_PAGO_MIN_B..MAX_B = 145 953..167 967 bytes (crates/zk-ssl/src/metrics.rs:82-83,
  band since S538 with hiding; the figures 66 739 / 66 692 bytes of the metrics.rs comment are PRIOR to S538 -the sceptic of block M
  places them at §512, not at the date 2026-08-26 the comment gives them-: they must not be cited as current. The range
  MEASURED per payment before the 5 % margin was 155 337..159 329 bytes, according to refutacion-measures.json, M2a/M2c).

## hbs-state
- `cargo test --release` -> 27 (lib) + 8 (tests/vectors.rs) passed, 0 failed.
- `python3 spec/verify-state.py --vectors spec/state-vectors-v0.3.json --subject ./target/release/hbs-state-subject`:
  family A N1 12/12, N2 12/12, N3 12/12; family B 6/6; N0 declares 'seed_derived'; LEVEL REACHED: N3.
- fsync self-check (source of the probe: `sonda-fsync.rs.txt`): literal replica of check_persistence (hbs-state src/lib.rs:523-556: 20 writes of 8 bytes with and without
  sync_all; it rejects if ratio < 10 AND con_fsync < 20 us; thresholds DECLARED in src/lib.rs:97-108), repeated 30 times per path and
  per batch, two batches:
  | path | with fsync (us) p50 [min-max] | without fsync (us) p50 | ratio p50 [min-max] | rejections |
  | ext4 /dev/vda, batch 1 | 128.9 [74.7-292.2] | 0.876 | 161.4 [92.6-292.9] | 0/30 |
  | ext4 /dev/vda, batch 2 | 173.1 [103.6-795.2] | 0.879 | 189.4 [116.9-986.5] | 0/30 |
  | tmpfs /dev/shm, batch 1 | 0.537 [0.531-1.582] | 0.421 | 1.3 [0.7-3.8] | 30/30 |
  | tmpfs /dev/shm, batch 2 | 0.536 [0.533-0.732] | 0.421 | 1.3 [0.7-1.8] | 30/30 |
  Limitation this illustrates: in a microVM the guest cannot know whether the host honours the flush down to the physical medium; the
  self-check only shows that fsync COSTS something here, which is exactly what it declares to measure.

# Re-run of 2026-09-30 on `main` 71c5aad (S582)

Same kind of environment (Firecracker microVM, kernel 6.18.44-fc-v50, 4 vCPU). Tree: 71c5aad + the commit of
this draft (71710b2). `main` advanced 18 commits since d531c80 (S566-S582: v6 head with reception
root, RFC-0010 ACCEPTED, completeness envelope).

- `cargo test --release -p zk-ssl --lib t1_` -> 3 passed, 0 failed.
- `cargo test --release -p zk-ssl-guardian` -> 26 passed, 0 failed.
- `bash tools/conformidad.sh target/release/zk-ssl-verify [MANIFIESTO]` (binary fea39a053efd7089): paquete 70/70,
  consumo 14/14, conflicto 16/16, rechazo 84/84, edad 11/11, pendiente 9/9, pago 9/9, prenda 9/9 and the NEW
  catalogue completitud 35/35 = 257/257.
- `simulate --amount 250000` (output in `simulate-2026-09-30.log`, ONE run): issuances 66.8 KiB [328 ms] and
  66.5 KiB [364 ms]; send 78.9 KiB [1333 ms]; claim 77.8 KiB [879 ms]; apply 5-6 ms. Send+claim = 156.7 KiB
  ~ 160 460 bytes, WITHIN the band 145 953..167 967. Against 2026-09-28 (77.6 + 76.7 KiB) the size
  changes from one run to another, as metrics.rs declares: with hiding each proof weighs according to q and the salt.

# The published kit against today's tree (2026-09-30)

`git worktree add` of the tag `arqueo-verify-v0.2.0` (1528943) and `cargo build --release -p zk-ssl-verify` with a
separate `CARGO_TARGET_DIR`; the binary, against vectors of `main` at 71c5aad, and the tree's beside it:

| vector | head | kit v0.2.0 | tree 71c5aad |
|---|---|---|---|
| `paquete/posicion-v2.json` | v3 | 0 VERDE | 0 |
| `consumo/consumo.json` | v4 | 0 VERDE | 0 |
| `edad/edad-todos.json` | v5 | 1 ROJO «tipo desconocido: edad» | 0 |
| `prenda/prenda.json` | v5 | 1 ROJO «tipo desconocido: prenda» | 0 |
| `pendiente/cobro-inferior-0.json` | v5 | 1 ROJO «tipo desconocido: cobro_pendiente» | 0 |
| `completitud/declarada.json` | v6 | 1 ROJO «tipo desconocido: completitud» | 3 (its output for «declarada») |
| `completitud/no-resuelta.json` | v6 | 1 ROJO «tipo desconocido: completitud» | 1 (the named red the vector asks for) |
| `ancla/ancla-derivada.json` | v6 | 1 ROJO «tipo desconocido: ancla» | 0 (§596, against `main` at ab79a50) |
| `ancla/ancla-exacta.json` | v6 | 1 ROJO «tipo desconocido: ancla» | 0 (§596, against `main` at ab79a50) |
| `ancla/ancla-extendida.json` | v6 | 1 ROJO «tipo desconocido: ancla» | 0 (§596, against `main` at ab79a50) |

And by reading: its `main.rs` imports `epoch_digest_v2`, `v3` and `v4`, and nothing else. It fails closed: no false VERDE.
