# ECST — transiciones de estado con evidencia incorporada

> **English:** `ECST_EN.md` is the report in English. The verification register and the working
> material are in Spanish.

Un informe técnico, **en borrador y no depositado**, sobre *Evidence-Carrying State Transitions*: una
disciplina para componer pruebas por transición, encadenado por hash, cabezas firmadas y reconciliación
del estado físico, con dos instanciaciones del mismo autor —HBS-STATE (repositorio `hbs-state`) y Arqueo
(este)—.

⚠️ **Procedencia.** El informe sale de varios borradores generados con IA que el autor aportó y que
mezclaban ideas razonables con cifras, hechos de código y citas inventados. Antes de escribir una línea se
verificaron sus 305 afirmaciones contra los dos árboles y la literatura, y el informe sólo recoge lo que
resistió. Todo lo de este directorio lo produjo un asistente de IA generativa (Claude, de Anthropic) y está
**pendiente de la aceptación del autor** que describe `GENAI.md`.

⚠️ **Integración (§594).** El directorio entra en `main` en el §594, con el arreglo de su §8.1, por
decisión del autor. Viene de la rama de trabajo `claude/awesome-pasteur-u08nq9`, que NO se fusionó:
sus commits llevaban como autor al asistente, y aquí se rehicieron a nombre del autor, con el mismo
contenido. Los hashes que el material cita (`a7c72b9`, `6a868ed`, `78d71a4`…) son de esa rama; su
contenido es el de aquí. Integrar no es aceptar: la aceptación de `GENAI.md` sigue pendiente.

## Qué hay

| fichero | qué es |
|---|---|
| `ECST.md` | el informe, en español |
| `ECST_EN.md` | el mismo informe, en inglés |
| `VERIFICACION.md` | el registro: cada afirmación de los borradores con su veredicto final (CONFIRMADA, PARCIAL, FALSA, SIN_FUENTE, NO_VERIFICABLE), lo que se puede afirmar y sus fuentes. Se genera entero de `borrador/final.json` |
| `referencias.bib` | la bibliografía verificada: sólo entradas re-derivadas de metadatos consultados |
| `borrador/` | el material de trabajo: el catálogo de afirmaciones, los resultados de cada verificador y de cada escéptico, la fusión, y la reproducción (`borrador/REPRODUCCION.md`) |

## Base

Arqueo `main` en 71c5aad (S582); `hbs-state` en a960828 (crate 0.2.0, especificación HBS-STATE v0.3).
Lo que cambie en los árboles después de esos commits no está recogido: las rutas `fichero:línea`
envejecen con ellos.

## Cómo se regenera el PDF

Como los preprints de `doc/preprints/`, con `pandoc`; la bibliografía va resuelta a mano en la sección
«Referencias» del propio informe, así que no hace falta `citeproc`.
