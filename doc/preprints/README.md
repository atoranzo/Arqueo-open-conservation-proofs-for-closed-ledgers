# Fuentes de los preprints

El texto publicado en Zenodo, en markdown, con el estilo y el guion que lo
convierten en los PDF depositados.

Está aquí porque los tres papeles dicen que el trabajo es **reproducible
desde el artefacto**, y hasta este commit el texto publicado no estaba en
el artefacto. Una afirmación de reproducibilidad que no incluye lo
publicado es una afirmación sin comprobar.

## Los tres

| Fichero | Título | DOI |
|---|---|---|
| `ZK-SSL-preprint.md` | Comparative Implementation of a Zero-Knowledge Settlement Layer across Five Proof Systems | [10.5281/zenodo.21736125](https://doi.org/10.5281/zenodo.21736125) |
| `ZK-SSL-policy-note.md` | Provable Compliance without Full Ledger Disclosure | [10.5281/zenodo.21736082](https://doi.org/10.5281/zenodo.21736082) |
| `ZK-SSL-residual-trust.md` | From Institutional Trust to Verifiable Properties | [10.5281/zenodo.21905595](https://doi.org/10.5281/zenodo.21905595) |

## Cómo se regenera un PDF

```bash
./generar.sh ZK-SSL-preprint
```

Necesita `pandoc` y `wkhtmltopdf`. Los PDF depositados se hicieron con
**wkhtmltopdf 0.12.6**; otra versión puede paginar distinto sin cambiar el
contenido.

## Procedencia, que importa para leer esto bien

⚠️ **Estas fuentes son una reconstrucción, no el original.** Las dos
primeras revisiones se maquetaron desde un HTML que no se conservó, y
cuando llegó la tercera revisión hubo que **reconstruir el texto desde la
capa de texto de los PDF publicados**, aplicar las correcciones y volver a
maquetar. De ahí que los PDF de la tercera revisión pagine distinto que
los de la segunda.

Consecuencias que un lector debe conocer:

- El contenido de la tercera revisión **sí** salió de estos ficheros: son
  la fuente real de lo depositado, no una transcripción posterior.
- Frente a la segunda revisión puede haberse perdido alguna cursiva, nota
  al pie o matiz de formato que la extracción no recuperase. El texto se
  revisó, pero la comprobación fue humana, no mecánica.
- A partir de aquí el problema desaparece: la cuarta revisión será un
  parche sobre estos ficheros.

## Referencias cruzadas: corregidas

~~Las referencias cruzadas apuntan a versiones anteriores.~~ **Corregido el
01-08-2026** (entrada **16**): las siete citas —cinco cruzadas y **dos en
las que un preprint se citaba a sí mismo con su DOI de primera revisión**—
apuntan ya a las terceras.

⚠️ La entrada 16 hablaba solo de las cruzadas; las dos autocitas se
encontraron al hacerla.

## Estado de los depósitos

Los ficheros de este directorio llevan las correcciones de la cuarta
revisión (entrada **28**): notas sobre las dos propiedades que el diseño
garantizaba y la implementación no imponía, el arreglo incompleto de la
anchura de identidades, la unidad **MiB**, y —en
`ZK-SSL-residual-trust.md`— la sección §4.7.

✅ **`ZK-SSL-residual-trust.md`: cuarta revisión depositada el
2026-08-12** — [10.5281/zenodo.21905595](https://doi.org/10.5281/zenodo.21905595) —
como nueva versión del mismo registro. Añade sobre lo ya escrito: el
cierre de §4.7 (salt derivado de la clave, con la secuencia visible), la
sección nueva §4.8 (la capa de evidencia, medida, con sus cuatro
residuales), la fila de §4.1 que la entrada 1 de `ERRATA.md` reclamaba,
cifras re-medidas sobre `7ad62a9`, y la retirada del apéndice de
metadatos. ~~Este fichero del árbol ES el fuente exacto de lo depositado.~~

~~⚠️ **`ZK-SSL-preprint.md` y `ZK-SSL-policy-note.md` siguen escritas y
sin depositar**: sus DOI de arriba apuntan a las terceras revisiones,
que son las que un lector recibe hoy. Y sus citas a *residual-trust*
apuntan aún a la tercera — misma clase que la entrada 16, para su
propia revisión.~~

✅ **Corregido el 2026-08-27.** Las dos SÍ estaban depositadas desde
el 2026-08-01, comprobado abriendo los registros: `ZK-SSL-preprint.md` en
[10.5281/zenodo.21736125](https://doi.org/10.5281/zenodo.21736125) y
`ZK-SSL-policy-note.md` en
[10.5281/zenodo.21736082](https://doi.org/10.5281/zenodo.21736082).

⚠️ **El registro de `ZK-SSL-policy-note.md` tiene los ficheros
RESTRINGIDOS**: la ficha es pública y el PDF no se descarga sin cuenta.

⚠️ **Los DOI de los tres ficheros de este directorio se actualizaron en esa
misma fecha, levantando para ese corte la suspensión de la entrada 28** y
la regla de `tools/check_cifras.py` que los conservaba verbatim. **Sólo se
tocaron identificadores**: ni una cifra, ni una afirmación, ni un párrafo
de los textos. **Consecuencia declarada: desde ese commit los tres ficheros
ya no son byte a byte los PDF depositados** — difieren en las líneas de
DOI —, y por eso la frase tachada de arriba deja de valer. El siguiente
depósito los vuelve a sincronizar.

⚠️ **El 2026-09-05 (`AUDITORIA.md` §403) se mudó también la URL del
repositorio en los tres ficheros**, por la misma regla que los DOI: sólo
identificadores, ni una cifra ni un párrafo. Desde ese commit difieren de los
PDF depositados en las líneas de DOI **y en las de la URL**; el resto sigue
igual. Lo que el lector del PDF recibe queda dicho en `ERRATA.md`, entrada 4.
El siguiente depósito los vuelve a sincronizar.
