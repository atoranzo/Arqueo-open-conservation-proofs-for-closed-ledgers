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

## Licencias

Nada de torchwood ni de Go se copia aquí: `testigo-local/main.go` es de
este repositorio y solo los importa, y `torchwood.json` es lo que
contestaron. torchwood se distribuye con licencia BSD de 3 cláusulas
(Copyright The Go Authors y The Torchwood Authors) y Go con la suya,
también BSD. Las semillas son de prueba.
