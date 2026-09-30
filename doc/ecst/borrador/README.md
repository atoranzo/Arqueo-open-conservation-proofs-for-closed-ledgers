# ECST — material de trabajo de la verificación (completo el 2026-09-30)

> ⚠️ **Esto no es el informe.** Es el material intermedio para redactarlo: la verificación, afirmación
> por afirmación, de los borradores sobre ECST (*Evidence-Carrying State Transitions*), HBS-STATE, el
> Internet-Draft `draft-toranzo-hbs-state` y el correo al CFRG, contra este árbol, contra el repositorio
> `hbs-state` y contra la literatura. **No se cita, no se deposita y no se toma como fuente.**
>
> Lo produjo un asistente de IA generativa (Claude, de Anthropic) en dos sesiones de Claude Code, sobre
> `main` en `d531c80` (2026-09-28) y en `71c5aad` (2026-09-30). Nada de aquí ha pasado todavía por la aceptación del autor que describe
> `GENAI.md`.

## Qué hay

| fichero | qué es | estado |
|---|---|---|
| `flujo-verificacion.js` | el guion del flujo de agentes: **el catálogo de afirmaciones** de los borradores, en siete bloques (H, G, L, C, M, B, P), con las rutas que cada verificador tenía que abrir | completo |
| `verificacion-hbs-state.json` | bloque H: la crate `hbs-state` y la especificación HBS-STATE v0.3 | hecho |
| `verificacion-guardian.json` | bloque G: el guardián propio de Arqueo (`zk-ssl-guardian`) y su relación con HBS-STATE | hecho |
| `verificacion-history.json` | bloque L: registro encadenado, cabeza de época, historia, paquete, verificación sin el nodo, anclaje | hecho |
| `verificacion-custody.json` | bloque C: pago en dos fases, custodia, qué revela una prueba, conservación, uso único, recuperación, poderes del operador | hecho |
| `verificacion-measures.json` | bloque M: cifras de los borradores frente a las medidas del repositorio; verificación formal; Nova | hecho |
| `verificacion-biblio.json` | bloque B: la bibliografía, con las citas corregidas (texto y BibTeX) en `extra_facts` | hecho |
| `verificacion-ietf-logic.json` | bloque P: el proceso IETF/IRTF, el Internet-Draft tal como está escrito, y la lógica formal de ECST | hecho |
| `refutacion-hbs-state.json` | el escéptico del bloque H: re-deriva cada veredicto | hecho |
| `refutacion-guardian.json` | el escéptico del bloque G | hecho |
| `refutacion-history.json`, `refutacion-custody.json`, `refutacion-measures.json`, `refutacion-biblio.json`, `refutacion-ietf-logic.json` | los escépticos de L, C, M, B y P, contra `main` 71c5aad; `refutacion-biblio.json` trae además las entradas BibTeX re-derivadas (`bibtex_verified`) | hecho el 2026-09-30 |
| `delta-guardian-hbs.json` | G y H, verificados sobre d531c80, re-comprobados contra 71c5aad (S566–S582) | hecho el 2026-09-30 |
| `final.json` | la fusión: un veredicto FINAL por afirmación (escéptico y delta por encima del verificador); de aquí se genera `../VERIFICACION.md` | hecho el 2026-09-30 |
| `simulate-2026-09-30.log` | el `simulate` re-ejecutado sobre 71c5aad | hecho |
| `REPRODUCCION.md` | lo que se re-ejecutó el 2026-09-28, con su entorno y sus salidas | hecho |
| `simulate-2026-09-28.log` | la salida íntegra de un `simulate` (una ejecución, claves deterministas del sandbox) | hecho |
| `sonda-fsync.rs.txt` | la réplica del autocontrol de `fsync` de `hbs-state` usada para la tabla de `REPRODUCCION.md` | hecho |

Cada `verificacion-*.json` tiene `findings` —`id`, `claim`, `verdict` (CONFIRMADA, PARCIAL, FALSA,
SIN_FUENTE, NO_VERIFICABLE), `severity`, `evidence`, `correct_statement`— y `extra_facts`. Cada
`refutacion-*.json` tiene una adjudicación por `id` (`agree`, `final_verdict`, `reason`, `evidence`).

## El recuento final (2026-09-30)

| bloque | afirmaciones | CONFIRMADA | PARCIAL | FALSA | SIN_FUENTE | NO_VERIFICABLE | el escéptico revocó |
|---|---|---|---|---|---|---|---|
| H | 58 | 24 | 17 | 8 | 8 | 1 | 1 |
| G | 36 | 15 | 12 | 7 | 2 | 0 | 0 |
| L | 39 | 10 | 17 | 9 | 3 | 0 | 1 |
| C | 33 | 10 | 15 | 6 | 2 | 0 | 4 |
| M | 30 | 6 | 13 | 5 | 6 | 0 | 1 |
| B | 58 | 30 | 8 | 17 | 0 | 3 | 2 |
| P | 51 | 9 | 23 | 16 | 2 | 1 | 2 |

Todos los bloques tienen ya su escéptico. El detalle, afirmación por afirmación, está en `../VERIFICACION.md`.

## Qué falta, en orden

1. ~~Los cinco escépticos que faltaban~~ — hechos el 2026-09-30.
2. ~~El registro de verificación legible~~ — `../VERIFICACION.md`, generado de `final.json`.
3. ~~El informe unificado~~ — `../ECST.md` y su gemelo `../ECST_EN.md`.
4. ~~Revisión adversarial del informe~~ — `revision-fidelidad.json` y `revision-rigor.json`; lo que se
   aplicó y por qué, en `revision-aplicada.json`.
5. **Pendiente, y es del autor:** su aceptación (`GENAI.md`) y el asiento que corresponda en `AUDITORIA.md`.
