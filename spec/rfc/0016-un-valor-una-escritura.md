# RFC-0016 — Un valor, una escritura: la canonicidad del núcleo

- **Estado:** ACEPTADO (§825) — **la regla 4 del PROCESO, contada.** (1) La spec: `NUCLEO.md`
  sección 6, «Canonicidad», y `MODULO` y `u64_canonico` en el censo de su sección 4; y `PAQUETE.md`,
  el texto `{campo}: 0x… no es canonico` en el catálogo de su sección 5 y el valor que su sección 9
  echaba en falta (§640). La E6 aplica en la biblioteca las reglas de la familia CUENTAS de
  `NUCLEO.md`, que ya estaban escritas (§475), y no mueve la spec. (2) El OpenRPC no se mueve: ni un
  método ni un campo; `spec/openrpc.json` no cambió en el §640 ni en el §782. (3) Los vectores,
  nuevos bajo `zkssl/0.4`, que no sube: los cuatro negativos de la E4
  —`paquete/rechazo-n-no-canonico.json`, `paquete/rechazo-mmrSize-no-canonico.json`,
  `ancla/neg-ancla-cero-escrito-como-p.json` y `completitud/neg-n-no-canonico.json`—, con su línea
  de manifiesto y su huella en `vectors/HUELLAS.sha256`. La E6 no tiene vector: no cambia ningún
  veredicto del kit, porque el mando juzga el camino con el módulo `cuentas`, y la atan tres
  falsadores de `zk-ssl-verify`. (4) Las suites: el canon `--sello` VERDE dentro del §640 y dentro
  del §782, y entero también en el §817, en el §852, en el §858 y en el §859 (sus asientos); del
  árbol del §859 al de este giro no cambia el código del kit ni nada de lo que este RFC toca: en
  `crates/` sólo cambian la capa (`zk-ssl/src/snapshot.rs` y sus tests), `zk-core/src/merkle.rs` y
  un comentario de `settlement-layer/src/sparse_tree.rs`, y en `tools/`, `canon.sh` y `artefacto.sh`
  (`git diff`). El giro sólo cambia texto, y sobre su árbol corrieron las once herramientas del
  bucle del canon, que leen ese texto, y `check_vectores`, `check_catalogo`, `check_tests` y
  `check_modulos`. Y al girarlo, sobre los catálogos de este árbol, con los cuatro negativos entre
  ellos: la release vigente, `arqueo-verify-v0.4.3`, publicada el 21-10-2026 (§818), con su binario
  publicado (`1b120e597897e005…`), pasa `paquete/` 89 de 89, `ancla/` 22 de 22 y `completitud/` 81
  de 81; el kit compilado desde este árbol, cuyas fuentes son las de la 0.4.3 salvo comentarios de
  `zk-ssl-hash` (§849) y de `zk-ssl-medio` y un test del medio con su `[[test]]` y su grabación
  (§848), y dos guiones del medio que graban (§855), que el kit no compila, lo mismo; y la segunda
  implementación, `paquete/` 89 de 89 y `ancla/` 22 de 22, con los tres de esas dos familias.
  Propuesto en el §640, con las cinco primeras etapas construidas en el mismo sello; la E6, en el
  §782. El diseño, D-A a D-E, lo aplicó la sesión del §640 con sus decisiones REVERSIBLES (las D-1 a
  D-3 de aquel asiento), y se acepta así. La decisión de aceptarlo la delegó el autor («HACER
  todo»), con la regla 4 por criterio, y empujar este sello es su aceptación, como en el §607. **Lo
  que se acepta CON su residuo, declarado** en «Lo que NO cierra»: `as_digest` sigue reduciendo
  (D-C) y lo vigilan tests, no una compuerta; los tres textos de camino siguen sin vector; y el
  deserializador de `Q` de `zk-ssl-wire` exige la escritura mínima y deja la cota a quien lo usa. Y
  fuera del árbol, al aceptarlo: la `arqueo-verify-v0.3.0` sigue publicada y acepta los cuatro
  negativos; la vigente, la 0.4.3, los rechaza y lleva la E6 (ver «Compatibilidad»).
- **Autor:** Ángel José Toranzo Portela
- **Asistencia GenAI:** Claude, en una sesión de Claude Code en la nube (§640): midió, propuso,
  aplicó, corrió el canon y commiteó, fuera del paso 4 de `GENAI.md`; la E6 (§782) y este giro
  (§825), también en sesiones de Claude Code en la nube y fuera del paso 4 — ver
  [`GENAI.md`](../../GENAI.md)
- **Fecha:** 2026-10-01
- **Versión del protocolo afectada:** `zkssl/0.4` — **no sube**: nada de lo que el núcleo produce se
  mueve, ni un KAT ni una cabeza firmada. Cambia lo que se ACEPTA al leer: menos.
- **Asiento(s) de AUDITORIA:** §640 (la propuesta y sus cinco primeras etapas); §641 (el RFC-0017,
  que matiza una frase de «Lo que NO cierra»); §662 (la escritura mínima de `Q` en el cable, que «Lo
  que NO cierra» cita); §775 (el testigo y el SDK leen con la regla); §782 (la E6); §825 (la
  aceptación). ⚠️ §825: decía «§640.»: callaba la E6, los otros dos asientos que «Lo que NO cierra»
  cita —el §662 y el §775— y el §641.

## Estado de las etapas

| etapa | qué entrega | ¿rompe el cable? | estado |
|---|---|---|---|
| E1 — la regla, en el núcleo | `MODULO` y `u64_canonico` en `zk-ssl-hash`; `element_from_bytes`, y con él `digest_from_bytes`, rechaza lo que no es menor que `p`; `FormatoError::NoCanonico` | NO | sellada — §640 |
| E2 — el mando la aplica | `u64_de` del verificador independiente lee canónico; los tres lectores de digest dicen la regla por `{e}`; la partición del MMR deja de colgarse con tamaños mayores que `2^63` | NO | sellada — §640 |
| E3 — la segunda implementación, con la misma regla | `tools/segunda/nucleo.py` (un productor, `u64_canonico`, y el mismo texto), `verificador.py` y `juez_cabezas.py` | NO | sellada — §640 |
| E4 — los vectores | cuatro negativos, uno por defecto medido, en `paquete/`, `ancla/` y `completitud/`, con su entrada de manifiesto; la referencia y la segunda los rechazan con el mismo texto donde las dos leen | NO | sellada — §640 |
| E5 — una definición en el probador | las siete copias privadas del embebido en `stark-experiment` pasan a ser `zk_ssl_hash::embeber`, y `SPEND_KEY_DOMAIN` se reexporta del núcleo | NO | sellada — §640 |
| E6 — el camino de la cuenta, entero | las seis `verificar_inclusion*` exigen que el camino mida `ACCOUNTS_DEPTH` y que sus orientaciones sean las del `indice` del recibo, con las reglas del módulo `cuentas`; dos errores nuevos, `ProfundidadDistinta` e `IndiceCruzado` | NO | sellada — §782 |

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

El kit `arqueo-verify-v0.3.0` es anterior y lee reduciendo: acepta los cuatro negativos nuevos.
Sigue publicado, y ya no es el vigente. La `arqueo-verify-v0.4.0` (§657) lee canónico, y medida
desde dentro de su tarball descargado pasa los tres catálogos que llevan los cuatro —`paquete/` 72
de 72, `ancla/` 22 de 22 y `completitud/` 74 de 74 (§658)—; la 0.4.1, también desde dentro del suyo,
lo mismo: 72 de 72, 22 de 22 y 74 de 74 (§661); y la 0.4.2, medida desde fuera en el §671, también:
73 de 73, 22 de 22 y 74 de 74. Al aceptar este RFC la vigente es la `arqueo-verify-v0.4.3`
(`doc/KIT.md` sección 0), publicada con su aviso el 21-10-2026 y producida sobre `5fd5212`, el
commit del §817: medida desde dentro de su tarball descargado, 84 de 84, 22 de 22 y 81 de 81 (§818);
sobre los catálogos de este árbol, el punto (4) de la cabecera. La E6 (§782) es posterior a la 0.4.2
y la lleva la 0.4.3: su commit, `2638f92`, está en
`git log a4e888d..5fd5212 -- crates/zk-ssl-verify/src`, de la 0.4.2 a la 0.4.3. La E6 no cambia
ningún veredicto del kit, porque el mando no llama a esas seis funciones.
⚠️ §825: decía «El kit publicado, `arqueo-verify-v0.3.0`, es anterior y lee reduciendo: acepta los
cuatro negativos nuevos. Lo corrige una release nueva, que es decisión del autor; hasta entonces,
el árbol y el kit publicado discrepan exactamente en estos cuatro vectores, y en ninguno más». La
release nueva llegó en el §657, y desde el §658 el kit vigente y el árbol leen igual estos cuatro.

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
  (`zk_ssl_wire::cantidad_canonica`). ⚠️ §825: el RFC-0017 matiza el «ningún veredicto de un tercero
  cambia por ellos» en su sección «Lo que corrige de RFC-0016» (§641): leído en el cable sin la
  regla, un `balance` del estado que aporta el titular escrito como `b + p` descuadraba la
  conservación del suministro al reabrir; su E4 lo lee canónico en `ClientStateDto` y en
  `AccountViewDto`, antes de que la capa lo sume o reste.
- ~~**`verificar_inclusion` y sus hermanas v2 a v6** no fijan la profundidad del árbol de cuentas
  ni cruzan el camino con el `indice` que el recibo lleva, que hoy no se usa. Ningún consumidor del
  árbol las llama; la regla que sí se aplica en el mando vive en el módulo `cuentas`. Queda nombrado.~~
  ⚠️ §782: cerrado en la E6. Las seis las aplican antes de subir la hoja, con las reglas del módulo
  `cuentas`; un camino corto o el de otra cuenta, con su raíz y su cabeza recompuestas, ya no
  verifica. Siguen sin consumidor en el árbol: el mando juzga con `cuentas`, como antes.
- **`as_digest` sigue reduciendo**, por D-C: un productor nuevo que componga un `u64` sin pasar por
  `u64_canonico` vuelve a abrir el hueco. Lo vigilan los tests que pinchan la no inyectividad, no
  una compuerta.
- **Los tres textos de camino** —`sibling {i}`, `camino[{i}]`, `{mote}: siblings[{i}]`— siguen sin
  vector: hace falta un hermano con un elemento menor que `2^32 - 1` para escribirlo como `x + p`,
  y los caminos capturados no lo traen.

## Referencias

`spec/NUCLEO.md` sección 6, «Canonicidad»; `spec/PAQUETE.md` secciones 5 y 9; `zk-ssl-hash`
(`MODULO`, `u64_canonico`, `element_from_bytes`); `zk-ssl-verify` (`u64_de`, `mmr::mitad`, y desde
la E6 `inclusion::exigir_camino_de_la_cuenta`); `tools/segunda/nucleo.py`; `AUDITORIA.md` §640,
§782 y §825.
