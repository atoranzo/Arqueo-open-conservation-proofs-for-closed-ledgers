# RFC-0016 — Un valor, una escritura: la canonicidad del núcleo

- **Estado:** PROPUESTO (§640) — con sus cinco etapas construidas en el mismo sello y el canon
  `--sello` VERDE dentro de él. El paso a ACEPTADO es del autor: la regla 4 del PROCESO pide la
  spec (`NUCLEO.md` sección 6, «Canonicidad», y `PAQUETE.md` sección 5), el OpenRPC (no se mueve:
  ningún método ni ningún campo cambia), los vectores (cuatro negativos nuevos bajo `zkssl/0.4`) y
  las suites; todo eso está en el §640, y la decisión no.
- **Autor:** Ángel José Toranzo Portela
- **Asistencia GenAI:** Claude, en una sesión de Claude Code en la nube (§640): midió, propuso,
  aplicó, corrió el canon y commiteó, fuera del paso 4 de `GENAI.md` — ver [`GENAI.md`](../../GENAI.md)
- **Fecha:** 2026-10-01
- **Versión del protocolo afectada:** `zkssl/0.4` — **no sube**: nada de lo que el núcleo produce se
  mueve, ni un KAT ni una cabeza firmada. Cambia lo que se ACEPTA al leer: menos.
- **Asiento(s) de AUDITORIA:** §640.

## Estado de las etapas

| etapa | qué entrega | ¿rompe el cable? | estado |
|---|---|---|---|
| E1 — la regla, en el núcleo | `MODULO` y `u64_canonico` en `zk-ssl-hash`; `element_from_bytes`, y con él `digest_from_bytes`, rechaza lo que no es menor que `p`; `FormatoError::NoCanonico` | NO | sellada — §640 |
| E2 — el mando la aplica | `u64_de` del verificador independiente lee canónico; los tres lectores de digest dicen la regla por `{e}`; la partición del MMR deja de colgarse con tamaños mayores que `2^63` | NO | sellada — §640 |
| E3 — la segunda implementación, con la misma regla | `tools/segunda/nucleo.py` (un productor, `u64_canonico`, y el mismo texto), `verificador.py` y `juez_cabezas.py` | NO | sellada — §640 |
| E4 — los vectores | cuatro negativos, uno por defecto medido, en `paquete/`, `ancla/` y `completitud/`, con su entrada de manifiesto; la referencia y la segunda los rechazan con el mismo texto donde las dos leen | NO | sellada — §640 |
| E5 — una definición en el probador | las siete copias privadas del embebido en `stark-experiment` pasan a ser `zk_ssl_hash::embeber`, y `SPEND_KEY_DOMAIN` se reexporta del núcleo | NO | sellada — §640 |

## Motivación

El núcleo compone sobre el campo de Goldilocks, `F_p` con `p = 2^64 - 2^32 + 1`. Un `u64` tiene
`2^64` valores y el campo `p`: `BaseElement::new` reduce, y con él dos lecturas del núcleo.

- `as_digest(x) = [x mod p, 0, 0, 0]` es inyectiva en `[0, p)` y en ningún sitio más:
  `as_digest(x + p) = as_digest(x)` para `x < 2^32 - 1`.
- `element_from_bytes` leía ocho bytes con `BaseElement::new`: el elemento `x < 2^32 - 1` tenía dos
  escrituras, `x` y `x + p`, y con él cada digest que lo contuviera.

La firma acredita el digest, y el digest acredita cada entero **módulo `p`**. El mando, en cambio,
razona sobre el entero: la ventana de la promesa (`S - e <= n`), el tamaño del MMR, la era de un
recibo. Lo que se midió, sobre cuatro vectores reales mutados en un solo campo, con el binario de
referencia y con la segunda implementación (`tools/segunda/`):

| vector | mutación | referencia, antes | segunda, antes |
|---|---|---|---|
| `paquete/posicion-v2.json` | `cabeza.n` pasa a `n + p` | VERDE, salida 0 | VERDE, salida 0 |
| `completitud/no-resuelta.json` | `n` del cierre y del recibo pasan a `n + p` | «ventana ABIERTA… el sobre es prematuro», salida 1 | no lee la familia |
| `paquete/extension.json` | `nueva.mmrSize` pasa a `t + p` | **se cuelga**: salida 124 a los 20 s | ROJO «NO extiende», salida 1 |
| `ancla/ancla-exacta.json` | el cero del `chainDigest` escrito como `p` | VERDE, salida 0 | ROJO «elemento fuera del campo», salida 1 |

Cuatro defectos distintos de una sola causa:

1. **Una firma, dos cabezas.** La misma firma XMSS y las mismas cofirmas acreditan una cabeza con
   `n` y otra con `n + p`.
2. **El operador sale de su propio rojo.** El sobre de completitud que lo nombra —«NO RESUELTA EN
   LA VENTANA»— pasa a «ventana ABIERTA», con la misma firma, si el `n` que el operador sirvió es
   `n + p`. El acusado elige el veredicto.
3. **El mando se cuelga.** La partición del MMR doblaba `k` mientras `k * 2 < n`; para `n > 2^63`
   el doble de `2^63` desborda, en release vuelve a cero y el bucle no acaba. Con `t + p` la firma
   lo acreditaba; un operador que firme un tamaño mayor que `2^63` lo provoca sin mutar nada.
4. **Las dos implementaciones divergían.** La segunda, escrita desde la spec, ya rechazaba el
   elemento no canónico; la referencia lo reducía. `NUCLEO.md` sección 6 no decía cuál de las dos
   era la regla, y la sección 9 de `PAQUETE.md` declaraba que cuatro textos de rechazo no tenían
   vector porque «no se conoce un valor» que `digest_from_bytes` rechace: no lo había.

Y un indicio de que la regla ya estaba en la intención: el cable (`zk-ssl-wire`) declara
`WireError::NotCanonical`, «bytes no canónicos para el cuerpo», sobre `digest_from_bytes` y
`element_from_bytes`, y no podía producirlo nunca.

## Diseño

**D-A — un elemento se lee canónico.** `element_from_bytes` pasa el `u64` por `u64_canonico` antes
de hacer el elemento, y rechaza con `FormatoError::NoCanonico(x)`, que se escribe
`0x… no es canonico: no es menor que p = 2^64 - 2^32 + 1` con el valor tal como llegó.
`digest_from_bytes` lo hereda por cada uno de sus cuatro elementos. Con esto `element_to_bytes` y
`element_from_bytes` son inversas exactas: una biyección de `F_p` sobre `[0, p)`, y un digest tiene
una escritura de 32 bytes.

**D-B — un `u64` que compone se lee canónico.** `u64_canonico` es el único productor de la regla.
El mando la aplica en `u64_de`, a todos los `u64` del sobre: todos entran en una composición o
razonan junto a una, y ningún productor de la casa escribe uno que no quepa. El rechazo nombra el
campo, `n: 0xffffffff000005a1 no es canonico: …`, y llega ANTES de recomponer y antes de la firma.

**D-C — `as_digest` no cambia.** Es núcleo congelado: sus KAT y todas las cabezas custodiadas
dependen de ella. La alternativa de hacerla inyectiva en todo `u64` —partir el entero en dos limbos
de 32 bits— movería los bytes de todo valor mayor que `2^32` y pediría una versión nueva del
preámbulo para un rango que ningún productor honesto usa. La frontera va en la lectura.

**D-D — la partición del MMR es total.** `mitad(n)` pasa a ser el bit más alto de `n - 1`,
`1 << (63 - lz(n - 1))`: la misma partición para todo `n` en que el bucle acababa, atado en los
tests contra el bucle viejo, y `2^63` donde aquel no acababa. Un tamaño enorme ya no cuelga: el
camino no es el de esa historia y la verificación sale `false`.

**D-E — una definición del embebido.** Las siete copias privadas de `as_digest(x: BaseElement)` en
`stark-experiment` eran `embeber` letra a letra, y la cabecera de `embeber` las censaba desde el
§258. Se importan del núcleo con su nombre de siempre; `SPEND_KEY_DOMAIN` del probador pasa a ser
la del núcleo, que la regla R2 de `check_dominios` mantenía igual a mano. No se mueve un byte: los
403 tests del probador, sus KAT de entradas fijas y la conformidad `zkssl/0.4` lo atan.

## Compatibilidad

No rompe el cable ni el formato. Lo medido:

- **Los 26 KAT del núcleo**, iguales, por la referencia y por la segunda implementación.
- **Los diez manifiestos**, entrada a entrada, dicen lo mismo que antes en las 316 entradas que ya
  tenían; las cuatro nuevas son negativos.
- **Las cabezas firmadas de todos los vectores**, que el juez de la segunda implementación
  recompone y verifica: ninguna de las que verificaban deja de hacerlo.
- **El cable**: `digest_from_wire` y `elem_from_wire` heredan D-A, y por fin producen el
  `NotCanonical` que declaraban. El nodo y el SDK escriben elementos con `as_int`, canónico: nada de
  lo persistido ni de lo publicado cambia de lectura.

El kit publicado, `arqueo-verify-v0.3.0`, es anterior y lee reduciendo: acepta los cuatro negativos
nuevos. Lo corrige una release nueva, que es decisión del autor; hasta entonces, el árbol y el kit
publicado discrepan exactamente en estos cuatro vectores, y en ninguno más.

## Seguridad

- **El principio del API** —la clave de gasto no viaja— no se toca.
- **Lo que cierra**: una cabeza firmada fija cada entero, no su clase módulo `p`; el operador ya no
  elige el veredicto del sobre de completitud escribiendo `n + p`; ningún sobre firmado cuelga el
  mando por su tamaño de MMR; y la referencia y la segunda implementación leen igual el mismo byte.
- **Tras el cambio, el truco no le sirve al operador.** Un titular con un `n + p` en la mano ve un
  rojo con nombre, y la reducción módulo `p` le devuelve el `n` que la firma acredita: con él, el
  sobre vuelve a nombrar al operador.

## Lo que NO cierra

- **El cable y el testigo leen `u64` sin la regla.** El lector de `QUANTITY` de `zk-ssl-wire` y el
  recompositor del testigo aceptan `n + p` y recomponen el mismo digest. Lo que firman y comparan
  es el digest, no el entero, así que ningún veredicto de un tercero cambia por ellos; queda
  nombrado, sin etapa. ⚠️ §775: el testigo ya lee con la regla —la escritura mínima y la cota
  `< p`—, y el SDK también, en el `logSeq` de la constancia. Queda el deserializador de `Q` de
  `zk-ssl-wire`, que desde el §662 exige la escritura mínima y deja la cota a quien lo usa
  (`zk_ssl_wire::cantidad_canonica`).
- **`verificar_inclusion` y sus hermanas v2 a v6** no fijan la profundidad del árbol de cuentas
  ni cruzan el camino con el `indice` que el recibo lleva, que hoy no se usa. Ningún consumidor del
  árbol las llama; la regla que sí se aplica en el mando vive en el módulo `cuentas`. Queda nombrado.
- **`as_digest` sigue reduciendo**, por D-C: un productor nuevo que componga un `u64` sin pasar por
  `u64_canonico` vuelve a abrir el hueco. Lo vigilan los tests que pinchan la no inyectividad, no
  una compuerta.
- **Los tres textos de camino** —`sibling {i}`, `camino[{i}]`, `{mote}: siblings[{i}]`— siguen sin
  vector: hace falta un hermano con un elemento menor que `2^32 - 1` para escribirlo como `x + p`,
  y los caminos capturados no lo traen.

## Referencias

`spec/NUCLEO.md` sección 6, «Canonicidad»; `spec/PAQUETE.md` secciones 5 y 9; `zk-ssl-hash`
(`MODULO`, `u64_canonico`, `element_from_bytes`); `zk-ssl-verify` (`u64_de`, `mmr::mitad`);
`tools/segunda/nucleo.py`; `AUDITORIA.md` §640.
