# `tools/segunda` — la segunda implementación: el núcleo, las cabezas, el segundo verificador, el STARK y el cable

Los spikes de [`doc/integracion-vertical-evaluacion.md`](../../doc/integracion-vertical-evaluacion.md),
secciones 5.2 y 5.3, medidos el 01-10-2026 sobre `e8ac246` (S615).

| fichero | qué es |
|---|---|
| `nucleo.py` | la **segunda implementación del NÚCLEO** (`spec/NUCLEO.md`, sección 6) en Python, sin dependencias: el campo de Goldilocks, la permutación Rescue-Prime `Rp64_256`, BLAKE3, la serialización, los dominios, los preámbulos y las composiciones, en el orden exacto de los merges; desde el §800, también los dos preámbulos v7 y el registro de esquemas del RFC-0020, con sus reglas, que el juez comprueba con un falsador |
| `juez_nucleo.py` | el **juez**: corre los 34 KAT de `spec/vectors/nucleo/` contra `nucleo.py` y dice `OK` o `ROJO` por vector, más las `NOTA` sobre lo que la sección 6 deja abierto. ⚠️ §797: hasta aquí decía «es el único llamador de `nucleo.py`», y ya no lo era: lo importan también `juez_cabezas.py:23`, `verificador.py:25` y `stark.py:30`, y desde el §797 `cable.py` y `juez_cadena.py` |
| `kat_xmss/xmss.py` | la **verificación XMSS^MT de RFC 8391** para `XMSSMT-SHA2_40/8_256`, el conjunto que la casa firma, escrita desde el texto del RFC y de ningún crate; solo verificar |
| `juez_cabezas.py` | el **juez de las cabezas firmadas**: para cada cabeza y cofirma de los vectores recompone el `epochDigest` por versión, compara el preámbulo recuperado, verifica la firma con `kat_xmss/xmss.py` y comprueba el índice embebido (`PAQUETE.md`, 2/3 y sección 8); con un falsador que voltea un byte |
| `verificador.py` | el **segundo verificador**: las cinco formas del paquete que no exigen una prueba STARK —posición v1 y v2 con acuse y cofirmas, extensión, consumo, conflicto y ancla—, y desde el §649 las `actas` que unen dos claves de un operador en la extensión y el consumo (RFC-0015: el lector, las reglas de la cadena y sus firmas XMSS), y desde el §686 el sobre del solapamiento, escritas desde `PAQUETE.md` con el contrato del mando (un argumento, `ROJO: {motivo}` del catálogo, exit 0/1/2), y juzgadas por `tools/conformidad.sh` con los mismos manifiestos que el binario; desde el §693 declara lo que no lee, en `NO_SOPORTADOS`, y lo dice con el texto reservado `NO SOPORTADO` |
| `cable.py` | el **cable en Python**, dos piezas en un fichero, y ⚠️ **ninguna es núcleo** (§797). ⚠️ §809: las dos se escribieron con el mismo nombre de fichero —la cadena en el §797, el consumidor en el §809— y se juntaron en uno sin compartir ningún nombre; `juez_cadena.py` importa la parte I y `main` es de la II. I · la **cadena del registro** como la sirve el cable, `chain_v1` y `chain_v2`, y lo que cada clase asienta como `proofDigest`, escritas desde `spec/RPC.md` §«La composición de `chain`, byte a byte»; es la clase REGISTRO de `NUCLEO.md` (RFC-0005, D-A), y por eso no vive en `nucleo.py`, del que sólo toma las primitivas. II · el **segundo consumidor del cable** (§809): juzga una respuesta de `zkssl_signedEpochHead` desde fichero y sin nodo, como `zk-ssl-cli witness --respuesta` con la memoria fresca —el cuerpo JSON-RPC, con los límites del lector de la referencia, `available`, el tipado del cable, lo que una cabeza firmada tiene que traer, la versión, la clave, la firma, el preámbulo, el `index` y la recomposición—, con el contrato del mando (un argumento, `VERDE: nueva` o una línea `ROJO: <clase> · <texto>`, exit 0/1/2), y lo juzga `tools/conformidad.sh` con el manifiesto del cable, en lugar del adaptador `tools/cable_respuesta.sh`; la cabeza se parte y se recompone con `juez_cabezas.py` y la firma se verifica con `kat_xmss/xmss.py`. Lo que `spec/RPC.md` no escribe lo fija la conducta medida del testigo, como caja negra (abajo, «El segundo consumidor del cable») |
| `stark.py` | el **verificador STARK** (§626): la maquinaria de `winter-verifier` 0.13.1 en Python, sin dependencias —el formato de `Proof`, la moneda pública sobre Blake3 en el orden exacto de Fiat-Shamir, la autenticación Merkle por lotes con la sal de `MerkleConSal`, el chequeo fuera del dominio, la composición DEEP, FRI y la envoltura `Oculta`—, leída de las fuentes de winterfell y no copiada; `INV_MDS` se calcula invirtiendo la MDS y el autotest la contrasta con la tabla de `winter-crypto` |
| `airs.py` | las **cinco AIR** del paquete —Banda, Edad, Prenda, CobroPendiente y PagoEnCurso—, **transcritas** de `crates/zk-ssl-air/src/*.rs` restricción a restricción: ninguna RFC escribe sus restricciones y el `.rs` es la única fuente, así que una AIR infra-restringida pasaría aquí igual que allí |
| `juez_stark.py` | el **juez del STARK**: compone, para cada vector con prueba, el par que el mando juzga —la prueba y el enunciado, con los campos y la cabeza que lee `zk-ssl-verify`— y lo compara con su `MANIFIESTO.txt`; tres falsadores por positivo |
| `juez_cadena.py` | el **juez de la cadena** (§797): recompone desde sus entradas cada `chain` de los vectores del cable —en la forma del vector de conformidad o en la de la lista `LogEntry[]` que sirve `zkssl_logEntries`—, dice `OK`, `DECLARADO` o `ROJO <fichero> seq <seq> (<kind>)` por entrada, corre sus falsadores, dice en sus líneas `NOTA` y `SIN EJERCITAR` lo que no ve y cierra con `cadena: N de N cadenas recompuestas` |
| `wasm_runner.py` | el **envoltorio** para correr `zk-ssl-verify` compilado a `wasm32-wasip1` dentro de wasmtime con el contrato del mando (`PAQUETE.md`, sección 6), para que `tools/conformidad.sh` lo juzgue como al binario nativo |
| `kat_xmss/` | el **corpus KAT de XMSS^MT**: claves y firmas producidas por el crate `xmss 0.1.0-pre.0` desde semillas fijas y verificadas por `kat_xmss/xmss.py`, que vive dentro para que el directorio sea un repositorio por sí solo; con su juez y su generador. Es el módulo con perfil de `hbs-state`: extraíble tal cual, sin un byte del protocolo (`kat_xmss/README.md`) |

## Cómo se corre

```bash
# 1 · la segunda implementación contra los KAT del núcleo        -> nucleo: 34 de 34
python3 tools/segunda/juez_nucleo.py

# 1 bis · las cabezas firmadas de todos los vectores, con XMSS^MT desde RFC 8391  -> ~31 s (§809)
python3 tools/segunda/juez_cabezas.py

# 1 ter · el corpus KAT de XMSS^MT: dos implementaciones, los mismos bytes       -> 8 de 8
python3 tools/segunda/kat_xmss/juez_xmss.py

# 1 quater · el segundo verificador, con el arnes y los manifiestos del binario   -> 174 de 174 (§780)
for m in paquete consumo conflicto ancla rotacion; do bash tools/conformidad.sh tools/segunda/verificador.py spec/vectors/$m/MANIFIESTO.txt | tail -1; done

# 1 quinquies · el verificador STARK: las pruebas ocultas y con sal de seis familias      -> 34 de 34 (§706)
python3 tools/segunda/stark.py          # autotest: la extension, INV_MDS, las raices, la marca
python3 tools/segunda/juez_stark.py

# 1 sexies · la cadena del registro del cable (NO es núcleo: spec/RPC.md)       -> cadena: 6 de 6
python3 tools/segunda/juez_cadena.py                          # spec/vectors/zkssl-0.4.json
python3 tools/segunda/juez_cadena.py spec/vectors/zkssl-0.*.json   # los cuatro: 24 de 24
python3 tools/segunda/juez_cadena.py respuesta.json   # o el `result` de zkssl_logEntries, desde seq 0
# 1 septies · el segundo consumidor del cable, con el arnes y el manifiesto del testigo     -> 23 de 23 (§809)
bash tools/conformidad.sh tools/segunda/cable.py spec/vectors/cable/MANIFIESTO.txt | tail -1

# 2 · el verificador a WebAssembly, y los diez manifiestos bajo wasmtime
rustup target add wasm32-wasip1
cargo build -p zk-ssl-verify --release --target wasm32-wasip1 --locked
python3 -m pip install wasmtime                                   # el motor wasmtime, con bindings
python3 tools/segunda/wasm_runner.py --precompilar target/wasm32-wasip1/release/zk-ssl-verify.wasm /tmp/zk.cwasm
printf '#!/usr/bin/env bash\nexec python3 %s/tools/segunda/wasm_runner.py /tmp/zk.cwasm "$@"\n' "$PWD" > /tmp/zk-wasm
chmod +x /tmp/zk-wasm
for m in ancla completitud conflicto consumo edad pago paquete pendiente prenda rechazo; do
  bash tools/conformidad.sh /tmp/zk-wasm spec/vectors/$m/MANIFIESTO.txt | tail -1
done
```

## Lo medido

**La segunda implementación.** 34 de 34 vectores reproducidos byte a byte (26 hasta el §643, que
añade los tres del acta, 29 hasta el §773, que añade `digest_pi` y la hoja v2 del recibo, y 31 hasta el §800, que añade el registro de esquemas y los dos preámbulos v7 del RFC-0020), escritos desde la sección 6 de `NUCLEO.md` y de ningún `.rs` de la casa.
La permutación sale de la fuente de `winter-crypto 0.13.1`, que es la primitiva y no la casa, y se
valida en el autotest contra el vector que ese crate atribuye a la implementación Sage; BLAKE3 se
escribe desde la especificación pública y se valida contra 19 salidas del crate `blake3 1.8.5` del
`Cargo.lock`.

**Las cabezas firmadas.** `juez_cabezas.py` recorre los 406 ficheros de `spec/vectors/` que llevan
cabezas, incluidos los catálogos `0.3/` y los del cable:

| medida | valor |
|---|---|
| cabezas firmadas encontradas | 537 |
| que recomponen su `epochDigest`, recuperan su preámbulo, verifican su firma y atan su índice | 485 |
| cofirmas de testigo encontradas / que verifican | 15 / 10 |
| ficheros con alguna cabeza o cofirma que NO verifica | 57, **los 57 negativos** según su `MANIFIESTO.txt` |
| positivos con alguna cabeza que no verifica | 0 |
| falsador: un byte volteado en la firma y otro en el preámbulo | las dos dejan de verificar |
| una verificación XMSS^MT en Python | ~40 ms |

Los 52 fallos de cabeza son los que los negativos fabrican: 29 cabezas cuyos campos no recomponen
el digest, 4 firmas que no verifican, 3 índices declarados que no quedan por encima del embebido
(uno por debajo y dos iguales), 2 sin `index`, 5 campos que no son canónicos (un `u64` que no es
menor que `p`, RFC-0016), 8 versiones fuera del conjunto o sin sus campos y una clave cuyo OID no es
de XMSS^MT; y los 5 de cofirma, 2 firmas que no verifican, 2 preámbulos que no son el esperado y una
`versionFormato` que no cabe en su byte (§780). La firma viaja como RFC 8391 la describe, 18.469
bytes —`idx` (5) ‖ `r` (32) ‖ 8 × (WOTS+ 67×32 ‖ auth 5×32)—, seguida del preámbulo; la clave son
`OID` (4, `0x00000005`) ‖ `root` ‖ `SEED`, 68 bytes, y una biblioteca que lea el OID como el RFC
manda no necesita el apaño del §240.

⚠️ §793: aquí decía 331 ficheros, 418 cabezas, 376 que verifican, 11 / 7 cofirmas y 46 ficheros, y
«los 42 fallos» —26 que no recomponen, 6 firmas, 2 preámbulos, 2 índices «por debajo del embebido»,
2 sin `index` y las versiones—: las cifras del S623. Los asientos las recontaron al crecer los
vectores (el §640, 378 de 424 cabezas; el §706, 472 de 522), y esta página no las siguió. Las de
ahora son la salida de `python3 tools/segunda/juez_cabezas.py` sobre el árbol del §789 (`8c02941`); el
~40 ms es del S623 y no se ha vuelto a medir. La cofirma de `versionFormato` `0x103` que trajo el §780
cuenta entre los negativos: `juzgar_cofirma` la rechaza con el texto del kit, como `verificador.py`.

⚠️ §809: aquí decía 404 ficheros, 535 cabezas, 55 ficheros con «los 55 negativos» y «los 50 fallos», con
2 índices («uno por debajo y otro igual») y 4 campos no canónicos: la medida del §793, de antes de los
dos negativos del cable que el §809 añade, `cable/rechazo-index-atrasado.json` (el `index` igual al
embebido) y `cable/rechazo-n-no-canonico.json` (`n + p`). Las de ahora son la salida de
`python3 tools/segunda/juez_cabezas.py` sobre el árbol del §809: 485 de 537 cabezas, 10 de 15 cofirmas,
406 ficheros, 57 con alguna que no verifica y 0 positivos, en 31 a 33 s con la máquina cargada (31,9 s
de reloj y 30,1 de CPU en la última corrida). Y la línea «1 bis» de «Cómo se corre» decía «~25 s».

**El segundo verificador.** `verificador.py` pasa, con el mismo arnés y los mismos manifiestos
que el binario de referencia, las cinco familias que no exigen STARK:

| familia | entradas | segundo verificador |
|---|---|---|
| `paquete` (posición v1 y v2, extensión) | 84 | 84 de 84 |
| `consumo` (desde el §771, también entre dos claves con `actas`) | 17 | 17 de 17 |
| `conflicto` | 17 | 17 de 17 |
| `ancla` | 22 | 22 de 22 |
| `rotacion` (la extensión entre dos claves, con `actas`; §649; y el sobre del solapamiento, §686) | 34 | 34 de 34 |

Y en los positivos de las cinco, su salida es **idéntica línea a línea** a la del binario, la
huella del ancla y la posición derivada del consumo incluidas. Lo que destapó: `NUCLEO.md` §6 no
escribe `posicion_de_consumo` ni la hoja vacía ni la convención de `isRight`, que viven en
RFC-0006 §413 (los 63 bits bajos de los primeros ocho bytes; la hoja vacía es el digest cero;
`isRight[i]` es el bit `i`) y no tienen KAT; y la huella de la clave, que es Blake3, entra en
`ancla_digest` leída como Digest de cuatro elementos, lo que deja sin decir qué pasa si un limbo
queda fuera del campo (2⁻³² por limbo): aquí era ROJO con nombre, y la referencia lo REDUCÍA. Desde
el §640 (RFC-0016, `NUCLEO.md` sección 6, «Canonicidad») la regla está escrita: un productor reduce
—`limbos_reducidos`, como `resumen_con_dominio`— y un lector rechaza lo que no es menor que `p`, con
el mismo texto en las dos implementaciones.

⚠️ §793: aquí decía «las cuatro familias», y la tabla lleva cinco desde que el §649 añadió `rotacion`;
`paquete` 72, la cuenta del §640, que el §663 llevó a 73, el §693 a 83 y el §780 a 84; y `conflicto`
16, que el §664 llevó a 17; y «Cómo se corre», arriba, decía «173 de 173 (§771)» hasta el §780,
que lo llevó a 174. Las cinco filas
son la salida de `bash tools/conformidad.sh tools/segunda/verificador.py
spec/vectors/<familia>/MANIFIESTO.txt` sobre el árbol del §789 (`8c02941`): 174 de 174, con salida
0 en las cinco. Que la salida de los positivos sea la del binario línea a línea es la medida del
S623 y, para `rotacion`, la de la E5b del RFC-0015 (§649): no se ha vuelto a comparar, porque pide
el binario compilado.

**El verificador STARK** (§626). Las 58 pruebas de las seis familias que el kit verifica hoy van
**ocultas** (la marca `arqueo:oculta:1` en el meta de la traza: L = 2T, una columna más, las
exenciones del AIR interno más T) y **con sal** (cada hoja es `merge(item, sal)`); `stark.py` hace los
dos caminos. El juez compone el par de cada vector y lo compara con el manifiesto:

| medida | valor |
|---|---|
| pares comparados con su manifiesto / que dicen lo que deben | 23 / 23 |
| de ellos, positivos que verifican | 12, en las seis familias |
| negativos rechazados con la causa del juez | 11: `InconsistentOodConstraintEvaluations` donde el enunciado miente (otra cuenta, otra cota, otro importe, otro receptor, otra marca), el techo de la banda, las subraíces que no suben a la raíz, `nacido ≥ seq` |
| `completitud`, por sus tres veredictos | la prenda aceptada verifica contra su cabeza; la rechazada con prueba NO verifica contra la juzgada (`ConstraintQueryDoesNotMatchCommitment`); la del RECHAZO SIN FUNDAMENTO SÍ |
| la misma causa que imprime el binario, par a par | 23 de 23, medido corriendo `zk-ssl-verify` sobre cada vector |
| falsadores (un byte volteado en tres tercios de la prueba) que dejan de verificar | 33 de 33 |
| vectores que caen antes del juez (firma, cabeza, campos) | 35, contados y no comparados |
| una verificación en Python | 0,2 a 0,5 s (Edad, la multisegmento, la más lenta); el juez entero, 12 s |

⚠️ §793: la tabla es la del §626 y sus 23 pares no son los de hoy. Sobre el árbol del §789
(`8c02941`), `python3 tools/segunda/juez_stark.py` da 34 de 34 pares que dicen lo que deben —11 que
verifican, en las seis familias, y 23 que no, con la causa del juez—, 48 pruebas juzgadas, 35
vectores que caen antes del juez y 33 de 33 falsadores callados, lo que el párrafo del §706, abajo,
ya dice. La fila de la causa del binario, par a par, no se ha vuelto a medir: pide `zk-ssl-verify`
compilado.

Lo que la medida fijó al escribirlo: que la semilla de Fiat-Shamir es el contexto **más las
entradas públicas**, de modo que un enunciado mal compuesto no llega ni a la autenticación Merkle;
que las aserciones de «última fila» de una AIR oculta usan la longitud **interna** T; que las
columnas periódicas de Banda, Prenda, Cobro y Pago miden la traza interna entera (512) y se evalúan
en `z²`; y que `check_leading_zeros` de la moneda cuenta los ceros **finales** de la primera palabra.

**La marca, como el kit (§706).** Hasta el §706 `stark.py` leía la marca de la ocultación detrás de
las opciones, devolvía la m que trajera y juzgaba un meta vacío como prueba sin ocultar; el kit
mira la forma, después la marca, que tiene que ser la de la casa con m = 64 (`comprobar_marca`,
§651), y después las opciones. Una prueba de edad honesta con m = 32 en la marca era ROJO en el kit
y VERDE aquí, y los negativos de marca caían aquí por otra causa. Desde el §706 la segunda exige lo
mismo, en el mismo orden y con el mismo texto, `marca de la ocultacion {lectura}; el enunciado pide
m = 64`, y el autotest comprueba la de la casa y las cuatro lecturas que rechaza. Lo fijan seis
negativos de la edad y el pago (`neg-marca-m32`, `neg-meta-vacio` y `neg-marca-m32-y-opciones`):
con el `stark.py` anterior el juez da 28 de 34 pares; con este, 34 de 34, 48 pruebas juzgadas, 35
vectores que caen antes del juez y 33 de 33 falsadores callados. La comprobación se leyó del kit,
como las AIR; su texto y sus cuatro lecturas los da también `spec/PAQUETE.md` §5.

**La cadena del registro** (§797). ⚠️ Esto mide el **cable**, no el núcleo: `chain` es de la clase
REGISTRO (`spec/NUCLEO.md` §«2. Las cuatro clases»), su composición no tiene KAT (sus piezas sí:
`spec/RPC.md` §«La composición de `chain`, byte a byte») y `conformance --check`, que exige el
probador, la declara no reproducible desde el §538 (RFC-0009, D-AI). Lo que aquí se comprueba es la
composición: con las entradas tal como el vector las trae, o como las sirve `zkssl_logEntries`,
`proofDigest` incluido.

| medida | valor |
|---|---|
| cadenas de `zkssl-0.4.json` recompuestas | 6 de 6, todas de era 2 |
| sus `proof_digest` recompuestos sin la prueba | 4: los dos `OpenAccount` (constante) y los dos `Mint` (desde su `compromiso`) |
| `proof_digest` declarados | 2: `Send` y `Claim`, bytes de prueba (D-AI) |
| los cuatro vectores del cable, `zkssl-0.1` a `zkssl-0.4` | 24 de 24; las 6 de `0.1` son las únicas de era 1 |
| las seis de `zkssl-0.4.json` reescritas como las sirve el cable, `LogEntry[]` en camelCase | 6 de 6 |
| ⚠️ etiquetas que contrasta alguna entrada de los cuatro vectores | **4 de 13**: `OpenAccount` 1, `Mint` 2, `Send` 8 y `Claim` 9. Las otras nueve, sin contraste: con la de `Consumo` cambiada a 14 en una copia de `cable.py`, siguen 24 de 24 |
| ⚠️ clases cuyo `proof_digest` fija R6 sin la prueba, contrastadas | **2 de 7**: `OpenAccount` y `Mint`. Sin entrada: `Freeze`, `Recovery`, `Governance` y `MintToPending` (el sello de autorización) y `Consumo` (la prueba vacía) |
| falsadores dentro del juez (un campo alterado cada uno, o el orden del último merge) que dejan de recomponer | 8 de 8 en era 2, 7 de 7 en era 1 |
| falsadores fuera del juez, sobre copias en un directorio temporal (§797) | 13 casos. Con la primera redacción, 5 + 4 + 1 + 1 + 2: 5 tracebacks (JSON roto, raíz numérica, entrada que no es objeto, la lista del cable, `kind` que no es texto sino una lista), 4 verdes que no debían (hex en mayúscula, `seq` `0x١`, clave repetida, clave ajena), 1 rojo falso (`OpenAccount` de era 1 con la prueba vacía), 1 `ROJO` por `KeyError: 'root_old'` (una entrada del vector con la grafía mezclada, `rootOld`) y 2 verdes sin aviso (el truncado; las eras que retroceden). Ahora: 9 `ROJO` con nombre, 2 verdes con su `DECLARADO` o su `NOTA` (la lista del cable; la era 1) y los 2 que no rechaza, con su `NOTA` (el truncado; las eras que retroceden). Y dos de la segunda ronda (§797): `"compromiso": null`, `ROJO` por R7, donde `LogEntryDto` leería la era 1; y un fichero de una sola entrada (la `OpenAccount` de era 1 de arriba, o la primera de `zkssl-0.4.json` sola), al que el falsador del anterior, que lo ponía a cero cuando ya lo era, daba un `ROJO FALSADOR` falso: ahora pone `[1, 0, 0, 0]` y los dos salen verdes, y con una copia de `cable.py` que ignora el anterior el falsador lo da `ROJO` |
| ⚠️ un registro truncado (sin la última entrada) | **5 de 5**: no lo ve, y lo dice en una `NOTA` por fichero; tampoco uno reescrito de forma coherente |
| `epoch_digest` del vector | **no** se recompone desde el fichero: faltan `pendingRoot` y `frozenRoot` |
| el juez entero sobre `zkssl-0.4.json` / sobre los cuatro | ~0,5 s / ~0,9 s |

No entra en el canon: se corre a mano.

**El verificador bajo wasmtime** (`wasmtime` 49.0.0 de PyPI; `rustc` 1.97.0):

| medida | nativo | `wasm32-wasip1` |
|---|---|---|
| tamaño del binario | 2.517.640 B | 1.587.916 B (3.502.520 B precompilado) |
| sha256, 16 primeros | `e524b7b6cb02e45c` | `0dbd46a1963dd396` |
| `conformidad.sh`, diez manifiestos | 316 de 316 | 316 de 316, mismo veredicto en cada entrada |
| los diez manifiestos, de pared | 9,28 s | 32,78 s |
| un vector de completitud, media de cinco | 18,4 ms | 93,3 ms, de los que 66,8 ms son arrancar Python y cargar wasmtime |

Los diez manifiestos: `ancla` 21, `completitud` 73, `conflicto` 16, `consumo` 14, `edad` 11,
`pago` 9, `paquete` 70, `pendiente` 9, `prenda` 9 y `rechazo` 84. El de `cable` es del testigo,
no del verificador, y no entra.

⚠️ §793: la tabla y esa cuenta son del S623, sobre `e8ac246`, y no se han vuelto a medir: hace falta
compilar el `.wasm`, y este sello no compila. Hoy esos diez manifiestos tienen 351 entradas
—`ancla` 22, `completitud` 78, `conflicto` 17, `consumo` 17, `edad` 15, `pago` 13, `paquete` 84,
`pendiente` 10, `prenda` 11 y `rechazo` 84—, y `FAMILIAS` (`tools/artefacto.sh:31`) tiene doce: con
`ancla-cofirmada` 27 y `rotacion` 34, son 412. Que el `.wasm` de hoy las pase no está medido, ni las
diez ni las doce; el bucle de arriba sigue siendo el de los diez que se midieron.

## Lo que el segundo verificador no lee, declarado (§693)

`verificador.py` lee seis formas de sobre —la de posición, sin `tipo`, y cinco tipos:
`extension`, `consumo`, `conflicto`, `ancla` y `solapamiento`, los de `TIPOS_LEIDOS`— y declara los
otros siete tipos en `NO_SOPORTADOS`: `rechazo`, `edad`, `cobro_pendiente`, `pago_en_curso`,
`prenda`, `completitud` y `ancla-cofirmada`, los que piden una prueba STARK o el medio. Uno de ellos
sale `ROJO: NO SOPORTADO: tipo {tipo}: …`, con exit 1: falla cerrado, y `tools/conformidad.sh` no lo
cuenta nunca: lo da `ROJO <fichero>: NO SOPORTADO - …`
(`spec/PAQUETE.md` §«6. El contrato del mando»). Hasta el §693 el texto era «no lee este sobre
todavia», y el arnés lo contaba como un negativo superado cuando el manifiesto pinaba sólo el
sujeto: `prenda/neg-receptor-mentido`, `prenda/neg-marca-mentida`, `edad/neg-cota-movida` y el
`neg-cota-movida` de `0.3/edad`, medido. Hoy, sobre los manifiestos de esos siete tipos, de `0.3/`
y de `0.4/`, todas las entradas salen `NO SOPORTADO` y ninguna `OK`. La causa de un rechazo se nombra con el texto del binario
(`spec/PAQUETE.md` §«6.1 La causa del rechazo»): `la firma no verifica`, `acuse: RaizDistinta`, y
la lectura de un `u64` con la escritura mínima del §662 y sus textos.

## El segundo consumidor del cable (§809)

`spec/RPC.md`, en «Los rechazos del cable», lo dejaba escrito: *«Una segunda implementacion del
consumidor pone su ejecutable en lugar del adaptador»*; y la fila de `vectors/cable/` de
`spec/README.md` dice para quién es ese manifiesto: *«you write a second consumer of
`zkssl_signedEpochHead`»*. `cable.py` es ese ejecutable. Hasta aquí el único que juzgaba la respuesta
era el testigo de la CLI (`crates/zk-ssl-cli/src/witness.rs`), el consumidor de referencia de `RPC.md`,
que verifica con `zk-ssl-verify`; `cobro`, `pago` y `prenda` también piden el método y leen su respuesta
para armar su sobre, pero no juzgan la firma. `cable.py` no comparte con el testigo una línea: lee el
cuerpo, lo tipa, y lo que queda se lo da a `juez_cabezas.py`.

**Contra el manifiesto.** `bash tools/conformidad.sh tools/segunda/cable.py
spec/vectors/cable/MANIFIESTO.txt`: 23 de 23 y exit 0, en unos 2 s; el adaptador del testigo,
`tools/cable_respuesta.sh`, da también 23 de 23.

**Contra un nodo de verdad.** Dos nodos `--dev` de release (`--latido 1`, uno con `--clave-fichero`,
diario, contador de recepción e índice de firma, y otro sin `--clave`), con su casa bajo `$HOME` y
borrada al acabar, como en los bancos. En ella van también el directorio de trabajo de los dos y el
contador de recepción del nodo sin clave, con `--contador-recepcion`: sin esa bandera, un nodo sin
clave lo abre en `recepcion.bin` de su directorio de trabajo (`zk-ssl-node --help`, §709), y arrancado
desde el árbol lo deja en su raíz, donde `git status` no lo ve porque `*.bin` está en `.gitignore`. Una
cuenta fondeada y un envío de ceros rechazado (`StaleState`), que consume un recibo. Tres respuestas
reales de `zkssl_signedEpochHead`, guardadas tal cual cruzaron el cable: la primera, antes del primer
latido (`available: false`, «aun no ha habido latido»); una cabeza v6 firmada, con `nextIndex` 1,
`totalSupply` 0x64 y `recepCount` 1; y la del nodo sin clave (`available: false` y la cabeza sin
firma). La firmada da `VERDE: nueva` en los dos consumidores, con la misma línea; las otras dos,
`ROJO: sin-firma · <el motivo que da el nodo>`, con el mismo texto desde `ROJO:` (el testigo lo
escribe detrás de `Error: `).

**Contra las manipuladas, en diferencial.** Lo que `RPC.md` no escribe lo fijó un diferencial:
respuestas manipuladas desde las tres cabezas reales del manifiesto (la v3, la v5 y la v6), juzgadas
por `zk-ssl-cli witness --respuesta` y por `cable.py` —cada clave quitada, a `null`, de otro tipo o de
más; cada QUANTITY y cada digest mal escritos o fuera del campo; bytes de la firma y de la clave, el
OID, las versiones y el `index`; parejas de defectos, para fijar en qué orden se miran; cuerpos rotos,
y los bordes de la lectura—. La primera versión de `cable.py` no miraba esos bordes, y en tres daba otra
clase que el testigo: un anidamiento hondo, un sustituto UTF-16 suelto y un número fuera del rango de un
`f64` (el punto 6). Los corpus de esos diferenciales no están en el árbol y sus cifras no se dan aquí,
porque desde él no se reproducen. Lo que fijaron son los puntos de abajo, y cada caso que nombran se
juzga a mano con los dos consumidores; juzgados así sobre el árbol del §809, con un testigo compilado de
él, dan los dos el mismo exit en todos, y la misma clase salvo en los tres del final del punto 6, en que
el testigo sale sin línea `ROJO:`. Con la misma clase, o el texto desde `ROJO:` es el mismo, o el
texto de uno es el principio del otro —`cable.py` añade detrás el nombre del campo (`QUANTITY
inválida`, y `en` y la clave) o el testigo el error de su biblioteca (`la firma no verifica: VerificationFailed`)—, o el texto
lo escribe una biblioteca: `serde_json` (`missing field`, `invalid type`, `unknown field`, sus errores
de lectura) frente al `json` de Python o al texto de `cable.py`, y el crate `xmss` (`InvalidOid(1)`,
`InvalidKeyLength {…}`), que el OID 0 y el 0x100 los escribe con su codificación interna:
`InvalidOid(131072)` y `InvalidOid(131328)`, donde `cable.py` dice el OID.

**Lo que fijó la medida, y `RPC.md` no escribe.** `RPC.md` nombra «los dieciocho campos de la cabeza,
`publicKey`, `epochDigest`, `formatVersion`, `index` y `signature`», dice que `custody` y
`custodyChecked` viajan siempre, y da el orden del camino: `result`, el tipado, el ancla de la clave,
la firma, la recomposición. Lo demás lo fijó el diferencial, mirando al testigo como caja negra:

1. **Treinta claves, y sólo cuatro de la forma.** `available`, `beatSeconds`, `custody` y
   `custodyChecked` son las que la forma del cable no deja faltar (`SignedEpochHeadDto`,
   `crates/zk-ssl-wire/src/lib.rs`:791-798), y la falta de las tres últimas es `forma del cable`; la de
   `available`, como su `null`, es `sin-firma`, porque el testigo la mira antes que la forma
   (`crates/zk-ssl-cli/src/witness.rs`:867; el punto 3). Las otras veintiséis son opcionales en la
   forma, y una cabeza firmada exige las que su versión compone y las comunes —veinticinco en una v6,
   todas menos `reason`—, con `cabeza firmada sin X: el productor la sirvio incompleta`: también
   `domain` y `emittedAtUnix`, que no se firman. Una opcional a `null` cuenta como ausente, y
   `beatSeconds`, `custody` o `custodyChecked` a `null` es `forma del cable`. Una clave repetida vale lo
   que diga la última, si lo que pisa se lee (el punto 6).
2. **Una clave de más rompe** (`forma del cable`): es el `deny_unknown_fields` de la referencia, lo que
   `RPC.md` llama «aditivo en el cable, rotura en el parser». El RFC-0005 (D-B) dice que eso es una
   elección de la implementación de referencia y no una regla del formato, y deja al autor si el
   formato la hace suya. `cable.py` elige lo mismo que la referencia, que cumple con cualquiera de las
   dos salidas de esa decisión; y por eso ningún vector lo pina: lo convertiría en regla.
3. **El orden.** Primero, que el cuerpo entero se lea (el punto 6). Después `available`, antes que la
   forma: lo que no es JSON `true` —`"true"`, `1`, `null`, o un `result` que no es un objeto— es
   `sin-firma`, con el `reason` del nodo si es una cadena y «sin motivo declarado» si no. La forma se
   mira clave a clave por orden de NOMBRE, no de llegada, y lo obligado que falta, al final. Después,
   si la versión escrita es la v4, la v5 o la v6, lo que su versión añade; luego lo común, en este
   orden: `seq`, `epochDigest`, `emittedAtUnix`, `domain`, `formatVersion`, `mmrRoot`, `mmrSize`,
   `index`, las cuatro raíces de la v1, `acusesRoot`, `n`, `signature` y `publicKey` —una cabeza v1 o
   v2 tiene que traer también la pareja del MMR—. Luego la versión, leída canónica y del conjunto; el
   `index`, canónico; la clave; la firma; el preámbulo; el `index` contra el embebido; y la
   recomposición, que lee cada valor canónico en el orden `seq`, `n`, las raíces, `acusesRoot`, y de
   ahí cada pareja en el orden de su versión.
4. **La forma mira la escritura, no el valor.** Un `u64` o un digest fuera del campo (RFC-0016) pasa la
   forma y cae al leerlo, con `QUANTITY "<valor>": <valor> no es canonico…` o `<campo>: no es un digest
   del campo`; `beatSeconds` y `emittedAtUnix` no se leen nunca como elementos, y fuera del campo no
   rompen nada. `domain` no se compara: otro dominio pasa, porque el preámbulo lleva el suyo.
5. **La clave es de XMSSMT-SHA2_40/8_256, el OID 5, o no se lee.** Otro OID de RFC 8391 —el 1, el 4, el
   6— es `clave publica ilegible`, no una firma que no verifica.
6. **El cuerpo que no se lee** es de la clase `respuesta ilegible`, la única con un espacio en vez de
   un guion: vacío, cortado, con BOM, con `NaN`, con cola; y lo que pasa de los tres límites que el
   lector de la referencia —`serde_json` 1.0.151, el del `Cargo.lock`, sin `float_roundtrip`— pone
   sobre el cuerpo ENTERO, no sólo sobre `result`, y el `json` de Python no pone:
   - **más de 127 niveles** de `[` y `{` anidados, contados desde la llave de fuera: 127 se leen y 128
     no —`{"result":[…]}` con 126 corchetes es `sin-firma` y con 127 es ilegible, y basta con anidar
     en `id`, en `error` o en una clave de más—;
   - **un sustituto UTF-16 suelto** (`\ud800` sin su pareja, `\udc80` solo) en cualquier cadena, clave o
     valor, firmada o no: en `custody`, en `jsonrpc`, en el `reason` de una respuesta sin firma, o en
     el valor de una clave repetida que la lectura pisa;
   - **un número cuyo `f64` sale infinito** como lo compone ese lector —la mantisa en un `u64`, que
     deja caer las cifras que no caben, y una sola potencia de 10 de su tabla—, esté donde esté:
     `1e309`, un entero de 400 cifras, y también `1.7976931348623158e308`, que el redondeo correcto
     deja en el mayor `f64` finito; `1.7976931348623157e308` y `1e308` se leen.

   RFC 8259 deja al lector poner límites a la profundidad y al rango de los números (sección 9) y no
   dice qué hacer con un sustituto suelto (sección 8.2): no son reglas del formato, sino del lector de
   la referencia, como la clave de más del punto 2. `cable.py` los imita para dar la misma clase, y
   ningún vector los pina, que los convertiría en regla. El borde del `f64` se leyó de la fuente de
   `serde_json` en el registro de cargo (`f64_from_parts`, sin compilarla) y se midió contra el testigo.

   Y en tres casos el testigo no cumple el contrato del mando: sale con exit 1 y **sin línea
   `ROJO:`**, con el error de E/S de Rust —un fichero que no existe (`Error: No such file or directory
   (os error 2)`), un directorio (`Error: Is a directory (os error 21)`) y unos bytes que no son UTF-8
   (`Error: stream did not contain valid UTF-8`)—, donde `cable.py` da `ROJO: respuesta ilegible · no
   se lee el fichero: …` o el error de la decodificación. El exit es el mismo, y ninguna entrada del
   manifiesto del cable lo pide.

**Los dos negativos que pidió.** Con las 21 entradas de antes, un consumidor sin la regla del `index`
(§399) o que reduce un `u64` en vez de rechazarlo (RFC-0016) daba 21 de 21: medido, rompiendo en una
copia `cable.py` (`if False and declarado <= embebido`) y `nucleo.u64_canonico` (devolviendo `u % P` antes
de la comprobación de la cota; en lugar del `return u` final no muta nada). Entran dos
negativos, derivados por una mutación del positivo v6 (`aaffc6ceee72f2fb`) y registrados en
`HUELLAS.sha256` con `tools/check_vectores.py --registrar`: `rechazo-index-atrasado.json`, con el
`index` 5, igual al embebido, y `rechazo-n-no-canonico.json`, con `n` = 0x5a0 + p, que recompone el
mismo digest si el lector reduce. Con ellos cada mutante cae en el suyo —22 de 23, «exit 0, el
manifiesto espera 1»—, y los dos consumidores del árbol dan 23 de 23. Otros once
pasos ya los vigilaban las 21, medido también rompiendo cada uno sobre una copia: sin verificar la firma
cae `rechazo-firma`; sin comparar el preámbulo, `rechazo-v5-con-firma-de-v4`; sin comparar la
recomposición, las tres de recomposición; con la v7 dentro del conjunto, `rechazo-formatVersion-7`, y
sin mirar el conjunto, las tres de versión fuera (1, 0x103 y 7); sin lo que exige cada versión, las cinco
`sin …`; sin lo común, `rechazo-sin-index`; sin las obligadas de la forma, `rechazo-forma-del-cable`;
sin el largo del digest, `rechazo-epochDigest-corto`; sin `result`, `rechazo-sin-result`; y sin la
clase `sin-firma`, `rechazo-available-false`. Lo que ningún vector pina el manifiesto no lo vigila, y
rompiendo cada uno en una copia `cable.py` sigue dando 23 de 23: los límites del lector, a propósito
(el punto 6: la profundidad a 100.000, sin mirar el anidamiento y los sustitutos, o sin el rango del
`f64`); la clave de más (el punto 2); el OID (el punto 5); el `index` leído canónico (el punto 3); y la
escritura mínima de una QUANTITY y de un hex (el punto 4: `0x05`, o un hex en mayúsculas). Con uno de
los tres últimos rotos, `cable.py` da `VERDE` donde el testigo da `ROJO`: no hay vector de un `index`
no canónico, ni de una QUANTITY con ceros delante, ni de un hex en mayúsculas. Sólo los miden los casos
de esos puntos, juzgados a mano con los dos.

## Lo que la segunda implementación destapó de la spec

Es para lo que existe la entrada 85 del BACKLOG: *«una spec que sólo implementa un código no es
una spec»*. Cuatro cosas que quien escriba desde la sección 6 no puede saber:

1. **`hoja_de_acuse(hashPrueba, seq, n)`** no estaba en la sección 6. `RPC.md`, en «`zkssl_ackPath`», da
   la hoja como `acuse_digest(hashPrueba, epoca, n)`, y en «El acuse en la respuesta (§274)» dice
   `epoca = logSeq + 1`, pero ninguna
   página dice que el `seq` de la hoja es ese `logSeq`. Con `epoca = seq` el KAT no reproduce;
   con `seq + 1`, sí. Medido.
2. **`hash_del_lote`**: la longitud codificada es la de los **bytes** de la composición, `48·k`,
   como en el molde del §116; no es `k`. La frase «`k` va en la longitud» de la sección 6 y del
   RFC-0014 E2 hay que leerla como «`k` queda implícito en la longitud». Medido.
3. **`cima`**: la sección 6 no dice si cada hoja pasa por `mmr_hoja` dentro de la cima. Sí pasa,
   que es la lectura de RFC 6962; el KAT lo sostiene.
4. **El formato de los KAT** codifica el mismo tipo de dos maneras: `embeber.x` y `balance` y
   `nonce` de `native_leaf*` van como u64 en **ocho bytes little-endian**; `seq`, `n`, `t`, los
   contadores y `as_digest.x` como el u64 en **hex**. Un juez tiene que saberlo por fuera.

## En el canon

Los tres jueces y el segundo verificador corren en cada sello (`tools/canon.sh`, bloque «3 duodecies»), a pin cero: un KAT
nuevo o una versión nueva de cabeza que la segunda implementación no reproduzca pone el sello en
rojo con su nombre; el juez de `kat_xmss/` corre en el mismo bloque, para que un cambio de bytes del
crate `xmss` se vea en el sello; el segundo verificador corre contra los cinco manifiestos —⚠️ §793:
aquí decía «los cuatro», y el bucle del canon (`tools/canon.sh:578`) lleva cinco desde el §649—, para
que un vector nuevo tenga que pasar por los dos códigos; y desde el §626 el juez del STARK corre en
el mismo bloque, para que una prueba nueva de esas familias tenga que verificar también en Python. `wasm_runner.py` no corre en el canon: necesita `wasmtime` de PyPI.
`cable.py` tampoco (§809): ni `RPC.md` ni `spec/README.md`, que lo piden, dicen que el canon lo corra, y
el bloque «3 bis cable» sigue corriendo el adaptador del testigo. Ponerlo junto al segundo verificador
es una fila más del «3 duodecies», a pin cero, y queda por hacer.
El `--sello` entero se corrió con el bloque dentro el 01-10-2026: VERDE, 1.054 s de tests.

## Lo que NO cubre

- Las AIR como segunda opinión: `airs.py` las transcribe del `.rs`, la única fuente que existe.
  Lo que el juez del STARK mide es la maquinaria del verificador, no si una AIR restringe lo que
  dice restringir.
- El mando entero de las formas con STARK en el segundo código: `juez_stark.py` juzga el par
  (prueba y enunciado) que el binario compone, pero no recorre la cabeza, la firma y los campos de
  esos sobres como `verificador.py` recorre los de las cinco formas sin STARK.
- Los 34 vectores conservados de `spec/vectors/0.3/`: sin ocultar y sin sal, con las AIR de
  `0eda58c`; el kit de hoy tampoco los verifica (rechaza su forma, D-AD).
- Que el `.wasm` corra en un navegador: `wasm32-unknown-unknown` lo para `getrandom`, que `xmss`
  arrastra por `rand 0.10` para generar claves que el verificador no usa (medido con
  `cargo tree -i getrandom@0.4.3`). Es una línea más para el issue a RustCrypto de la entrada 77.
  ⚠️ §793: la línea está desde el §793 en el borrador, `doc/issue-rustcrypto.md`, parte 5, sin enviar.
  Y no es el único `getrandom` del verificador: `cargo tree -i getrandom --locked --offline -p
  zk-ssl-verify -e normal` se niega por ambiguo, porque hay dos, y el otro, `getrandom 0.2.17`, llega
  por `rand_core 0.6.4` de `zk-ssl-air`, la sal de `MerkleConSal` (§535, D-X): es de la casa, y el
  issue no lo arregla. Que los dos paren la compilación es lo que dice su fuente, leída en el
  registro de cargo y no compilada: un `compile_error!` para `wasm*-unknown-unknown` sin la feature
  `wasm_js` (`getrandom` 0.4.3, `src/backends.rs:170-182`) o sin `js` (0.2.17, `src/lib.rs:344-349`).
- El testigo entero en el segundo código: `cable.py` juzga UNA respuesta con la memoria fresca, como
  `witness --respuesta`. La clave fijada de una vuelta a la siguiente, el segundo canal (la
  consistencia, con `zkssl_consistencyProof`), las cofirmas y el diario no tienen segunda
  implementación.
- Rendimiento: el `.wasm` no está optimizado para tamaño ni se ha pasado por `wasm-opt`.

## Procedencia

Escrito en una sesión con asistencia de IA generativa, en la rama `claude/nice-planck-ax35zl`,
e integrable a nombre del autor como el bloque ECST. Integrar no es aceptar: la aceptación que
describe [`GENAI.md`](../../GENAI.md) queda pendiente.
