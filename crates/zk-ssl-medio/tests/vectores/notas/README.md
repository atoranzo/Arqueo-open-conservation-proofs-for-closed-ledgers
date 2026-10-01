# Las notas del medio, contrastadas con torchwood (§632)

33 notas `checkpoint` del medio (RFC-0013 D-A, D-C) y lo que dicen de cada una
dos verificadores escritos por manos distintas: `zk_ssl_medio::nota` y
[`filippo.io/torchwood`](https://pkg.go.dev/filippo.io/torchwood) v0.10.0,
la biblioteca Go del autor de las especificaciones C2SP (`signed-note`,
`tlog-checkpoint`, `tlog-cosignature`). torchwood firma y verifica el tipo
`0x06` con el `crypto/mldsa` de la biblioteca estándar de Go 1.27, y la nota
con `golang.org/x/mod/sumdb/note`. La parte Rust usa el crate `ml-dsa` 0.1.1
de RustCrypto: dos implementaciones de ML-DSA-44 que no comparten código.

| ficheros | quién los firmó | qué son |
|---|---|---|
| `pos-00-anclas.txt` … `pos-13-anclas.txt` | torchwood | el publicador y un testigo sobre árboles de 0, 1, 2, 5 y 13 anclas |
| `pos-sin-testigo.txt`, `pos-testigo-primero.txt` | torchwood | la de 5 anclas sin testigo, y con el testigo delante |
| `neg-*.txt` (23) | torchwood | la de 5 anclas estropeada de una manera cada una |
| `rust-*.txt` (3) | este crate | dos con sal y una determinista, sobre 3, 21 y 8 anclas |
| `publicador.vkey`, `testigo.vkey` | torchwood | las dos claves, en el formato `vkey` de `signed-note` |
| `veredictos-torchwood.txt` | torchwood | lo que dice `note.Open` con `NewLogVerifier` de cada nota |
| `MANIFIESTO.txt` | — | lo que dice cada uno de los dos, y el motivo de cada diferencia |

El `origin` de todas es `zkssl/v1/` y la huella de la clave del operador de
`spec/vectors/ancla/` (el campo `clave` de su ancla). Las hojas son
sintéticas: la huella del ancla `i` es `SHA-256(u64be(i))`, y la raíz la
calcula torchwood con el árbol de `golang.org/x/mod/sumdb/tlog`. Las
semillas son de prueba y están en `torchwood/main.go`: el publicador
`00 01 … 1f`, el testigo `20 21 … 3f` y una clave impostora `40 41 … 5f`.

## Cómo se rehacen

Con Go 1.27 o posterior y red para bajar los módulos que fija
`torchwood/go.sum`:

```bash
cd crates/zk-ssl-medio/tests/vectores/notas
go -C torchwood run . generar ..     # reescribe pos-*, neg-*, las dos vkey
go -C torchwood run . verificar ..   # reescribe veredictos-torchwood.txt
```

Las firmas son deterministas, pero la marca de tiempo es la del reloj:
regenerar da notas nuevas, con otras marcas, y el `MANIFIESTO.txt` sigue
valiendo. Las `rust-*` las escribió un programa de fuera del árbol que
llama a `Publicador::firmar`; el `verificar` las juzga junto a las demás.
El canon no corre Go: `tests/vectores_notas.rs` lee lo que Go dejó escrito.

## Lo que dice el contraste

torchwood acepta las 10 positivas, también las 3 firmadas aquí, y rechaza
19 de las 23 negativas. Las otras 4 son reglas en las que el medio es más
estricto, a propósito y declaradas en la cabecera de `src/nota.rs`: un
`origin` distinto del nombre del publicador (D-A), dos líneas del
publicador (`note.Open` verifica la primera y descarta las demás sin
mirarlas) y una firma en base64 no canónica (Go la decodifica igual). No hay
ninguna al revés: nada que torchwood rechace lo acepta el medio.

## Licencias

Nada de torchwood ni de Go se copia aquí: `torchwood/main.go` es de este
repositorio y solo los importa, y las notas son su salida. torchwood se
distribuye con licencia BSD de 3 cláusulas (Copyright The Go Authors y The
Torchwood Authors) y Go con la suya, también BSD.
