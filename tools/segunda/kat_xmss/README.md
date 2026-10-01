# xmss-kat — vectores de respuesta conocida para XMSS^MT, con dos implementaciones detrás

> **English:** a known-answer test corpus for XMSS^MT (RFC 8391): keys and signatures produced by
> RustCrypto's `xmss 0.1.0-pre.0` from fixed seeds, and a dependency-free Python verifier written
> from the RFC text that checks every one of them, with three falsifiers per signature. Two
> origins, the same bytes. Born in Arqueo; see the provenance section.

**Qué es.** Un corpus de claves y firmas XMSS^MT producido por `xmss 0.1.0-pre.0` —el crate de
RustCrypto/signatures— desde semillas fijas, y `xmss.py`, una verificación de RFC 8391 escrita desde
el texto del RFC, en Python y sin ninguna dependencia, que verifica cada firma del corpus. Dos
orígenes, los mismos bytes.

**Por qué existe.** RustCrypto excluye los KAT de los paquetes que publica, y RFC 8391 no trae
vectores para los conjuntos multiárbol. Quien firma con XMSS^MT y clava la versión del crate no
tiene red debajo: un cambio de bytes entre versiones no lo caza nadie. Este directorio es esa red.

| fichero | qué es |
|---|---|
| `xmss.py` | la verificación XMSS^MT de RFC 8391 (secciones 2 a 4) para los ocho conjuntos multiárbol SHA2 con n = 32; solo verifica, no firma |
| `vectores.json` | dos conjuntos, `XMSSMT-SHA2_40/8_256` (OID 5) y `XMSSMT-SHA2_20/2_256` (OID 1): semilla de 96 bytes, clave pública de 68, y cuatro firmas consecutivas, índices 0 a 3, sobre mensajes de 0, 3, 50 y 1000 bytes, en las dos formas del crate, adjunta (`firma ‖ mensaje`) y separada |
| `juez_xmss.py` | verifica cada firma con `xmss.py`, ata el índice embebido al declarado y la firma separada a la adjunta, y pasa tres falsadores por firma: un byte volteado en la firma, en el mensaje y en la raíz de la clave |
| `generador/` | el programa Rust que produjo `vectores.json` desde sus semillas fijas, con el crate clavado con `=` |

```bash
python3 juez_xmss.py                     # -> kat_xmss: 8 de 8 firmas verifican ... y los tres falsadores callan
cargo run --release --manifest-path generador/Cargo.toml > /tmp/v.json && cmp vectores.json /tmp/v.json
```

## Lo medido (01-10-2026)

| medida | resultado |
|---|---|
| firmas del corpus verificadas por `xmss.py` | 8 de 8 |
| falsadores por firma que dejan de verificar | 3 de 3, en las 8 |
| la firma separada es la adjunta sin el mensaje | 8 de 8 |
| el índice embebido es el declarado | 8 de 8 |
| tamaño de firma, 40/8 y 20/2 | 18.469 B y 4.963 B, lo que RFC 8391 dice |
| `generador/` reproduce `vectores.json` | byte a byte: sha256 `259b8df2bb6633ff…` las dos veces |
| una verificación XMSS^MT 40/8 en Python | ~40 ms |

## Lo que fija

- Que la clave publicada lleva el OID de RFC 8391 en el registro **multiárbol** (`0x00000005` para
  40/8, `0x00000001` para 20/2) y que una biblioteca que lo lea como el RFC manda no necesita
  ningún apaño de lectura.
- Que la firma del crate es, byte a byte, la de RFC 8391: `idx` (⌈h/8⌉) ‖ `r` ‖ d × (WOTS+ ‖
  auth), y que `sign` la devuelve seguida del mensaje.
- Que la verificación es determinista dado el estado: la firma separada y la adjunta, al mismo
  índice y mensaje desde la misma semilla, son la misma.

## Lo que NO fija

- Nada del estado de la clave privada ni de cómo evitar reusar un índice: eso es
  [`hbs-state`](https://github.com/atoranzo/hbs-state), del mismo autor.
- Nada de la generación de claves más allá de «esta semilla da esta clave con este crate».
- Los conjuntos de 512 bits, los de SHAKE ni XMSS de un solo árbol: `xmss.py` sólo implementa
  XMSS^MT con SHA2 y n = 32.
- Que `xmss.py` o el crate estén auditados: no lo están, ninguno de los dos.

## Procedencia

Nació en Arqueo (<https://github.com/atoranzo/Arqueo-open-conservation-proofs-for-closed-ledgers>),
en `tools/segunda/kat_xmss/`, para cerrar la entrada 77 de su BACKLOG, «KAT ausente», y para que su
segunda implementación verifique las cabezas firmadas de su catálogo de vectores. Se extrajo con
`git subtree split`, como `mtc-core` y `hbs-state`: comparte con Arqueo la disciplina, no el
problema, y no lleva ni un byte de su protocolo. Vive en <https://github.com/atoranzo/xmss-kat>;
Arqueo lo sigue usando desde ese directorio.

Escrito en una sesión con asistencia de IA generativa (Claude, de Anthropic), con el método que
declara el `GENAI.md` de Arqueo: se mide primero, el autor decide, y nada entra sin pasar sus
compuertas. Autor: Ángel José Toranzo Portela.

## Licencia

MIT OR Apache-2.0, a elección de quien lo use: `LICENSE-MIT` y `LICENSE-APACHE`. El generador
depende del crate `xmss` de RustCrypto, bajo la misma licencia dual, y no lo incluye.
