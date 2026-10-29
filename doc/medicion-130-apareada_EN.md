# Paired measurement §130 — legacy ↔ twins (step 5)

> **Español:** [`doc/medicion-130-apareada.md`](./medicion-130-apareada.md) is the original of this
> page, in Spanish; it prevails wherever the two differ. This translation is of commit `7766efb`
> (audit entry §917) and was produced by the project's assistant, a Claude Code session, in audit
> entry §918 (`GENAI.md`): it is a generated artefact, not text written by the author. Line
> citations of the form `file:N` refer to the Spanish files.

**Method**: `#[ignore]` instruments per circuit (pattern `metrics_33`),
construction + prove inside the clock, identical honest scenarios per
pair. Canonical run **in series**:
`cargo test --release -p stark-experiment medicion_130 -- --ignored --nocapture --test-threads=1`
**Machine**: the pilot's (WSL, the same one of the whole historical series).
**HEAD of the measurement**: `6ee02d7` · 2026-08-04.

| circuit | legacy (ms · bytes) | twin (ms · bytes) | Δ time | Δ proof |
|---|---|---|---|---|
| send | 99,2 · 39.990 | 102,8 · 42.457 | +3,6 % | +6,2 % |
| claim | 98,0 · 38.899 | 102,4 · 41.994 | +4,5 % | +8,0 % |
| burn | 44,7 · 32.526 | 91,3 · 38.232 | **×2,04** | +17,5 % |
| mint | 43,8 · 34.595 | 45,7 · 35.589 | +4,3 % | +2,9 % |
| audit | 29,2 · 30.178 | 28,7 · 29.525 | −1,7 % | **−2,2 %** |
| mint_climb | 40,8 · 33.270 | 40,9 · 35.254 | +0,2 % | +6,0 % |
| recovery | 44,2 · 35.863 | 44,8 · 36.464 | +1,4 % | +1,7 % |
| recovery_climb | 39,0 · 33.218 | 40,8 · 34.240 | +4,6 % | +3,1 % |
| freeze | 40,1 · 32.932 | 43,9 · 33.029 | +9,5 % | +0,3 % |
| frozen_climb | 15,2 · 24.352 | 14,4 · 24.001 | −5,3 % | −1,4 % |

**Readings**: (1) burn ×2,04 = its own TRACE of 1024 (§145), prediction
confirmed; (2) frozen_climb comes out free or better — the legacy already paid
32 climbs (24 + 8 of expensive padding, §60.2) and the twin charges 32 real ones
for the same price; (3) audit: a deterministically SMALLER proof (−653
bytes), fact noted; (4) the rest: 0–9,5 % (freeze = its 8 extra real
cycles). The times are from ONE serial run: indicative; the proof
sizes are deterministic and canonical.

**Post-flip (§156)**: the twins ARE the circuits — the instruments
survive as `medicion_130_<circuito>` (the `_legado` ones died with
their files). This table remains as the snapshot of the cost on the eve
of the flip: the price was paid and passed into history.
