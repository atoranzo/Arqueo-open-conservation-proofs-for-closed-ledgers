# Fuentes de los preprints

El texto publicado en Zenodo, en markdown, con el estilo y el guion que lo
convierten en los PDF depositados.

Está aquí porque los tres papeles dicen que el trabajo es **reproducible
desde el artefacto**, y hasta este commit el texto publicado no estaba en
el artefacto. Una afirmación de reproducibilidad que no incluye lo
publicado es una afirmación sin comprobar.

## Los tres

| Fichero | Título | DOI de la tercera revisión |
|---|---|---|
| `ZK-SSL-preprint.md` | Comparative Implementation of a Zero-Knowledge Settlement Layer across Five Proof Systems | [10.5281/zenodo.21693706](https://doi.org/10.5281/zenodo.21693706) |
| `ZK-SSL-policy-note.md` | Provable Compliance without Full Ledger Disclosure | [10.5281/zenodo.21693709](https://doi.org/10.5281/zenodo.21693709) |
| `ZK-SSL-residual-trust.md` | From Institutional Trust to Verifiable Properties | [10.5281/zenodo.21693718](https://doi.org/10.5281/zenodo.21693718) |

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

## Cuarta revisión: escrita, sin depositar

Los ficheros de este directorio llevan ya las correcciones de la cuarta
revisión (entrada **28**): notas sobre las dos propiedades que el diseño
garantizaba y la implementación no imponía, el arreglo incompleto de la
anchura de identidades, la unidad **MiB**, y una sección nueva —§4.7 de
`ZK-SSL-residual-trust.md`— con un residual **abierto**.

⚠️ **No se ha depositado nada en Zenodo.** El texto está corregido; los DOI
de arriba siguen apuntando a las terceras revisiones, que son las que un
lector recibe hoy.
