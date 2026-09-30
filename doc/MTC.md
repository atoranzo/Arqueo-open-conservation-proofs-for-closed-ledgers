# De Arqueo a `mtc-core`: la extracción

`mtc-core` es el núcleo de una autoridad de certificación de **Merkle Tree
Certificates** (`draft-ietf-plants-merkle-tree-certs`, grupo PLANTS del IETF)
que nació dentro de este repositorio, en el directorio `mtc/`, y se extrajo a
un repositorio propio: <https://github.com/atoranzo/mtc-core>.

## Por qué se extrajo

Arqueo prueba conservación en libros cerrados, con STARK y sin ceremonia. Una
CA de certificados web no prueba conservación ni usa ZK: comparte con Arqueo la
infraestructura de árbol y la disciplina, no el problema. Tenerlo aquí como
workspace aparte era un parche para no romper el canon; tenerlo en su sitio le
da a cada proyecto una imagen fiel de lo que es. Es el mismo camino que siguió
`hbs-state`.

## Qué se llevó de Arqueo

| pieza de Arqueo | en `mtc-core` |
|---|---|
| `zk-ssl-verify::mmr` (MTH, PATH, SUBPROOF de RFC 6962) | `subtree`, traducido a SHA-256 y extendido a subárboles |
| `zk-ssl-verify::inclusion` (hoja → raíz → cabeza firmada) | `proof` y `verify` |
| `zk-ssl-node::firma_cabeza` (reservar, firmar, autoverificar) | `cosign::Cosigner` |
| `zk-ssl-guardian` (ya `hbs-state`) | `guard`, como dependencia |
| la idea de `zk-ssl::sparse_tree` (nodos en caché) | `log`, sobre un árbol *append-only* |
| la regla de `zk-ssl-hash`: una definición por formato | todo el crate |

Nada de la capa STARK, ni del fork de winterfell, ni de la máquina de estados
contable viajó con él.

## Dónde está la historia

La extracción conservó los commits del directorio `mtc/` con `git subtree
split`: el repositorio nuevo empieza en la historia que aquí termina, con sus
mensajes, sus cerrojos y sus trailers de procedencia. El autor publicó esa historia
el 2026-09-30 como `main` de `mtc-core` (doce commits, el último `a4f9f39`; hoy `21836fb`, porque el 2026-09-30 el autor reescribió ese historial para quitar el trailer `Co-Authored-By` que GitHub mostraba como coautoría, y la tabla de hashes antes y después está en la entrada 16 del registro de auditoría de mtc-core), y
en el mismo momento el directorio `mtc/` se borró de aquí: queda esta nota. La
rama `mtc-core-main` de este repositorio fue la partición que se empujó; el
autor la borró después, porque la historia entera vive ya en `mtc-core`.
