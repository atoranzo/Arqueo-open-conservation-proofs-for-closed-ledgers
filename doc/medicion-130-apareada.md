# Medición §130 apareada — legado ↔ gemelos (paso 5)

**Método**: instrumentos `#[ignore]` por circuito (patrón `metrics_33`),
construcción + prove dentro del reloj, escenarios honestos idénticos por
par. Corrida canónica **en serie**:
`cargo test --release -p stark-experiment medicion_130 -- --ignored --nocapture --test-threads=1`
**Máquina**: la del piloto (WSL, la misma de toda la serie histórica).
**HEAD de la medida**: `6ee02d7` · 2026-08-04.

| circuito | legado (ms · bytes) | gemelo (ms · bytes) | Δ tiempo | Δ prueba |
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

**Lecturas**: (1) burn ×2,04 = la TRACE propia 1024 (§145), predicción
confirmada; (2) frozen_climb sale gratis o mejor — el legado ya pagaba
32 subidas (24 + 8 de relleno caro, §60.2) y el gemelo cobra 32 reales
por el mismo precio; (3) audit: prueba deterministamente MENOR (−653
bytes), hecho anotado; (4) el resto: 0–9,5 % (freeze = sus 8 ciclos
reales de más). Los tiempos son de UNA corrida serie: orientativos; los
tamaños de prueba son deterministas y canónicos.

**Post-flip (§156)**: los gemelos SON los circuitos — los instrumentos
sobreviven como `medicion_130_<circuito>` (los `_legado` murieron con
sus ficheros). Esta tabla queda como la foto del coste en la víspera
del flip: el precio se pagó y pasó a la historia.
