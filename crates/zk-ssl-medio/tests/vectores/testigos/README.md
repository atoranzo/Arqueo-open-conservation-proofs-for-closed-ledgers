# El publicador del medio contra testigos locales (§808)

Lo que contestaron, **en `127.0.0.1`**, dos testigos de la mano del autor
de las especificaciones C2SP a nueve pasos del publicador del medio
(RFC-0013 E3a), y el programa y el guion que lo graban. El cliente es
`zk_ssl_medio::testigos` (con `firmar`); el protocolo, `tlog-witness`
v1.1.0 de C2SP (commit `b2dd989`, el mismo texto que su `main` el
2026-10-14).

La fuente de los testigos es el módulo `filippo.io/torchwood` v0.10.0:
git `2fa53be3e7049b61226d8a93ff2e1fc0af92d8d7` (la etiqueta `v0.10.0`),
`h1:zgBIImawK53X7lM0qHKcjn0pbDtFLqU04NU9qDvBnjs=` en `sum.golang.org` y
`go.mod` `h1:hZt8lhTBBCK77TmuK+q0D7mNcnt2ovQkJs+zdonDRog=`. `rehacer.sh`
lo baja con `go mod download`, que con el `GOSUMDB` por defecto lo
comprueba contra esa base de datos.

| testigo | qué es | con qué cofirma |
|---|---|---|
| `testigo.invalid/torchwood-a` y `-b` | el `add-checkpoint` de litewitness: `filippo.io/torchwood` v0.10.0 (git `2fa53be`), `internal/witness.NewWitness`, con el mismo `MaxBytesHandler` de 10 KiB y los mismos plazos que pone `cmd/litewitness` | ML-DSA-44, tipo `0x06`: lo que el publicador retiene y el kit verifica |
| `testigo.invalid/litewitness` | `cmd/litewitness` v0.10.0 **sin tocar**, con su clave en un agente SSH | Ed25519 —el binario no admite otra—: el publicador la ignora, como manda la especificación con una clave que no conoce |

Los tres guardan su estado en la base de datos de `cmd/witnessctl`, que
registra el medio (`add-log`, `add-key`) como se registra un log en un
testigo de verdad.

## Lo que hay aquí

| fichero | qué es |
|---|---|
| `torchwood.json` | la grabación: el origin, los testigos y sus `vkey`, y por cada paso lo que se le dio al cliente (`entrada`) y lo que pasó (`salida`), con cada petición y cada respuesta HTTP tal cual; y el sobre `ancla-cofirmada` de punta a punta |
| `testigo-local/main.go` | `testigo-local mldsa` (el motor de litewitness con ML-DSA-44), `agente` (el agente SSH en memoria para litewitness) y `comprobar` (verifica una cofirma con torchwood). Se compila DENTRO de una copia de torchwood, porque usa su `internal/witness` |
| `rehacer.sh` | el guion: baja torchwood v0.10.0 por el proxy de Go, lo compila con la cadena de Go 1.27 que pide su `go.mod`, levanta los tres testigos, corre los nueve pasos y la comprobación de punta a punta, y con `--guardar` reescribe `torchwood.json` |

## Los nueve pasos

`desconocido` (el origin que aún no conocen: `404`), `primera` (`old 0`:
cofirman), `sin-memoria` (el publicador vuelve a `old 0`: `409` con 3 y
el reintento cofirma), `crece` (de 3 a 7 anclas con su prueba),
`atrasado` (`old 0` con 9: `409` con 7 y la prueba de 7 a 9), `retroceso`
(el directorio restaurado a 5 anclas: `409` con 9, y el cliente PARA: la
103), `bifurcado` (un ancla reescrita, de 9 a 11: `422`), `otra-raiz` (9
anclas con otra raíz: `422`) e `impostor` (la nota de otra clave con el
mismo nombre: `403`). Las tres primeras anclas son las del catálogo
`spec/vectors/ancla-cofirmada/`, que el kit deriva de sus tres cabezas
firmadas; las demás, `SHA-256(u64be(i))`, y la reescrita, la de `i + 1000`.

Los tres testigos dicen lo mismo en los nueve pasos; lo único que cambia
es con qué firman el `200`. Y de punta a punta: la nota cofirmada de
`primera` —la del publicador del catálogo (semilla `01`) y las cofirmas
de torchwood-a y -b— entra en el sobre `cofirmada-dos-testigos.json` del
catálogo, el kit la da VERDE («la cofirman 2 testigo(s)»), el juez del
cli la ACREDITA con `k = 2` y no con `k = 3`, la línea Ed25519 de
litewitness verifica con torchwood, y torchwood lee la cofirma retenida.

## Cómo se rehace

Con Go en el `PATH` (o `GO=/ruta/al/go`), red para el proxy de Go y el
árbol en un estado cualquiera, desde la raíz:

```bash
bash crates/zk-ssl-medio/tests/vectores/testigos/rehacer.sh            # mide y no escribe
bash crates/zk-ssl-medio/tests/vectores/testigos/rehacer.sh --guardar  # reescribe torchwood.json
```

**Fuera del canon y fuera de `tools/bancos.sh`**: pide Go y red. La nota
del publicador se firma en determinista para que la petición se pueda
rehacer byte a byte; las cofirmas de los testigos llevan la marca de su
reloj y una firma con sal, así que cada grabación es otra, y
`tests/testigos_torchwood.rs` sigue valiendo con cualquiera que salga
VERDE. Lo que comprueba ese test en cada canon está en su cabecera.

## omniwitness, de otra mano (§848)

El mismo cliente contra **`omniwitness`**, el testigo de
`github.com/transparency-dev/witness`, **sin tocar**, en `127.0.0.1`. Es
el commit `539715523ab9d0ee7d036926f9d1b00266e1c567` (`5397155`, la punta
de su `main` el 2026-10-15; el módulo
`v0.0.0-20261014123032-539715523ab9`,
`h1:stSGpwkttvZrI5MHMcq7PluZdqpsulroZU9NvKWB8Bw=` en `sum.golang.org` y
`go.mod` `h1:DoW27RnDDy0bIJt3k8QkRJF+yCTRWAJdaPcztwEqucs=`). Sus claves
de cofirma son las de `github.com/transparency-dev/formats` en `340a9e5`
(`v0.1.2-0.20261013130728-340a9e55b2b7`), que fija su `go.mod`. El
protocolo es el mismo `tlog-witness` (`b2dd989`): su texto, y el de
`tlog-cosignature`, no cambiaron hasta el `main` de C2SP del 2026-10-21
(`e5746a0`).

| testigo | qué es | con qué cofirma |
|---|---|---|
| `testigo.invalid/omniwitness-a` | `cmd/omniwitness` con una clave (`--private_key_path`) | ML-DSA-44, tipo `0x06`: lo que el publicador retiene y el kit verifica |
| `testigo.invalid/omniwitness-b` | `cmd/omniwitness` con dos claves del mismo nombre, como en la rotación de su README: primero una ML-DSA-44 y detrás una Ed25519 | las dos en cada `200`: el cliente retiene la `0x06` e ignora la Ed25519, de una clave que no conoce |

El log del medio no está en su lista: se le configura a mano, una
entrada `Origin` y `PublicKey` (la `vkey` `0x06` del publicador) en el
YAML de `--additional_logs`. Sin red pública: ni métricas, ni
distribuidor, ni lista de una red de testigos.

| fichero | qué es |
|---|---|
| `omniwitness.json` | la grabación, con la misma forma que `torchwood.json`: el origin, los testigos con sus claves (la semilla y la `vkey` de cada una), cada paso con su `entrada` y su `salida` HTTP tal cual, y el sobre `ancla-cofirmada` |
| `omniwitness-local/main.go` | `omniwitness-local clave` (la clave privada que lee omniwitness, de una semilla, y su `vkey`) y `comprobar` (verifica una cofirma con el verificador de `formats`). Se compila DENTRO de una copia de `witness`, con las versiones que fija su `go.mod` |
| `rehacer_omniwitness.sh` | el guion: baja `witness` en `5397155` por el proxy de Go, compila `omniwitness` y `omniwitness-local` con la cadena de Go que pide (1.26), corre los diez pasos y la comprobación de punta a punta, y con `--guardar` reescribe `omniwitness.json` |

Los diez pasos, en términos de `tlog-witness`: `desconocido` (el origin
que aún no está configurado: `404`); se para cada testigo y se vuelve a
levantar con el log en `--additional_logs` y la misma base de datos;
`primera` (`old 0`: cofirman), `sin-memoria` (`old 0` con 3: `409` con 3
y el reintento cofirma), `crece` (de 3 a 7 con su prueba), `atrasado`
(`old 0` con 9: `409` con 7 y la prueba de 7 a 9), `nota-menor` (una nota
de 5 con `old 0`: `409` con 9, un tamaño mayor que el de la nota, y el
cliente PARA), `prueba-invalida` (de 9 a 11 con una prueba de consistencia
que no verifica contra la raíz de 9 que el testigo tiene: `422`),
`otra-raiz` (9 con otra raíz: `422`), `impostor` (la nota de otra clave
con el mismo origin: `403`) e `igual` (9 con `old 9` y la prueba vacía:
cofirman). Las hojas son las de `torchwood.json`.

Los dos testigos dicen lo mismo en los diez pasos; omniwitness-b firma
el `200` con dos líneas. Y de punta a punta: la nota cofirmada de
`primera` entra en el sobre `cofirmada-dos-testigos.json` del catálogo, el
kit la da VERDE («la cofirman 2 testigo(s)»), el juez del cli la ACREDITA
con `k = 2` y no con `k = 3`, `formats` lee las dos cofirmas retenidas,
y la línea Ed25519 de omniwitness-b verifica con `formats`.

```bash
bash crates/zk-ssl-medio/tests/vectores/testigos/rehacer_omniwitness.sh            # mide y no escribe
bash crates/zk-ssl-medio/tests/vectores/testigos/rehacer_omniwitness.sh --guardar  # reescribe omniwitness.json
```

Fuera del canon, como `rehacer.sh`. `tests/testigos_omniwitness.rs`
relee la grabación en cada canon, con lo que dice su cabecera, y además
comprueba que la `vkey` ML-DSA-44 de cada testigo, que `omniwitness-local`
sacó de su semilla con el ML-DSA de Go (`filippo.io/mldsa` v1.0.0, el de
`formats`), es la que saca de la misma semilla el `ml-dsa` de este crate.

## Licencias

Nada de torchwood ni de Go se copia aquí: `testigo-local/main.go` es de
este repositorio y solo los importa, y `torchwood.json` es lo que
contestaron. torchwood se distribuye con licencia BSD de 3 cláusulas
(Copyright The Go Authors y The Torchwood Authors) y Go con la suya,
también BSD. Las semillas son de prueba.

⚠️ §848: tampoco se copia nada de `transparency-dev/witness` ni de
`transparency-dev/formats`, que se distribuyen con la Apache License 2.0
(la cabecera de `cmd/omniwitness/monolith.go` dice Copyright Google LLC),
ni de `filippo.io/mldsa` (BSD de 3 cláusulas):
`omniwitness-local/main.go` es de este repositorio y solo los importa, y
`omniwitness.json` es lo que contestó omniwitness.
