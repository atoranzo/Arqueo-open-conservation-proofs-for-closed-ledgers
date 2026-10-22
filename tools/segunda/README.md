# `tools/segunda` — la segunda implementación: el núcleo, las cabezas, el segundo verificador, el STARK y el cable

Los spikes de [`doc/integracion-vertical-evaluacion.md`](../../doc/integracion-vertical-evaluacion.md),
secciones 5.2 y 5.3, medidos el 01-10-2026 sobre `e8ac246` (S615).

| fichero | qué es |
|---|---|
| `nucleo.py` | la **segunda implementación del NÚCLEO** (`spec/NUCLEO.md`, sección 6) en Python, sin dependencias: el campo de Goldilocks, la permutación Rescue-Prime `Rp64_256`, BLAKE3, la serialización, los dominios, los preámbulos y las composiciones, en el orden exacto de los merges; desde el §800, también los dos preámbulos v7 y el registro de esquemas del RFC-0020, con sus reglas, que el juez comprueba con un falsador |
| `juez_nucleo.py` | el **juez**: corre los 34 KAT de `spec/vectors/nucleo/` contra `nucleo.py` y dice `OK` o `ROJO` por vector, más las `NOTA` sobre lo que la sección 6 deja abierto. ⚠️ §797: hasta aquí decía «es el único llamador de `nucleo.py`», y ya no lo era: lo importan también `juez_cabezas.py:23`, `verificador.py:31` y `stark.py:30`, y desde el §797 `cable.py` y `juez_cadena.py`. ⚠️ §852: decía `verificador.py:25`, rancia desde el §810, que llevó el `import` a la `:30`; el `import re` del §852 lo lleva a la `:31` |
| `kat_xmss/xmss.py` | la **verificación XMSS^MT de RFC 8391** para `XMSSMT-SHA2_40/8_256`, el conjunto que la casa firma, escrita desde el texto del RFC y de ningún crate; solo verificar |
| `juez_cabezas.py` | el **juez de las cabezas firmadas**: para cada cabeza y cofirma de los vectores recompone el `epochDigest` por versión, compara el preámbulo recuperado, verifica la firma con `kat_xmss/xmss.py` y comprueba el índice embebido (`PAQUETE.md`, 2/3 y sección 8); con un falsador que voltea un byte |
| `verificador.py` | el **segundo verificador**: las cinco formas del paquete que no exigen una prueba STARK —posición v1 y v2 con acuse y cofirmas, extensión, consumo, conflicto y ancla—, y desde el §649 las `actas` que unen dos claves de un operador en la extensión y el consumo (RFC-0015: el lector, las reglas de la cadena y sus firmas XMSS), y desde el §686 el sobre del solapamiento, escritas desde `PAQUETE.md` con el contrato del mando (un argumento, `ROJO: {motivo}` del catálogo, exit 0/1/2), y juzgadas por `tools/conformidad.sh` con los mismos manifiestos que el binario; desde el §693 declara lo que no lee, en `NO_SOPORTADOS`, y lo dice con el texto reservado `NO SOPORTADO`; y desde el §810 lee el sobre del **ancla cofirmada** (RFC-0013 E4a), que no exige STARK sino el medio: la cabeza, la nota con su firma ML-DSA-44, las cofirmas y la inclusión, con `mldsa44.py` y `medio.py` |
| `mldsa44.py` | la **verificación ML-DSA-44 de FIPS 204** (§810), en Python y sin dependencias —`hashlib` trae SHAKE128 y SHAKE256—, escrita desde el texto del estándar —del crate `ml-dsa` 0.1.1 se leyó, para contrastar y no para copiar, cómo arma μ y dónde rechaza la norma y las pistas—: el algoritmo 3 con el contexto vacío y la firma pura, que es como la casa llama a `ml-dsa` 0.1.1 (`nota.rs`, `verify_with_context(mensaje, &[], firma)`), sobre el 8, con pkDecode, sigDecode, HintBitUnpack, SampleInBall, ExpandA, la NTT y UseHint; solo verificar |
| `medio.py` | el **medio del ancla** (§810): la base64 estricta, la inclusión de RFC 9162 en el árbol SHA-256 de anclas y la nota `checkpoint` con su firma `0x06` y sus cofirmas, con las reglas que declara la cabecera de `crates/zk-ssl-medio/src/nota.rs` —que es lo que `PAQUETE.md` 2.13 manda leer— y los textos de su `Display`, para que la causa de un rechazo sea la del binario |
| `juez_medio.py` | el **juez del medio** (§810): las 33 notas del §632 con el veredicto de su `MANIFIESTO.txt`, un **firmante de prueba** de FIPS 204 —KeyGen y Sign, solo aquí— que rehace byte a byte las vkeys y las líneas deterministas que firmó Go, las dos firmas de frontera de la norma, una pista donde la parte baja es cero con una clave a medida, la unicidad de la escritura de una firma, la base64, las 98 sondas de inclusión de transparency-dev y el `{:?}` de Rust con el que salen los nombres en un rechazo; y desde el §859, cinco reglas de `mldsa44.py` y `medio.py` que nada aislaba, cada una con su caso y su control |
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

# 1 quater · el segundo verificador, con el arnes y los manifiestos del binario   -> 236 de 236 (§714)
#   ⚠️ §810: decia 174 de 174 (§780), las cinco familias sin `ancla-cofirmada`, que salia NO SOPORTADO
#   ⚠️ §852: decia 201 de 201 (§810); los cinco negativos `rechazo-json-*` de `paquete` la llevan a 206
#   ⚠️ §712: decia 206 de 206 (§852); los nueve vectores del reloj, cinco de `paquete`, tres de `consumo` y uno de `ancla`, la llevan a 215
#   ⚠️ §714: decia 215 de 215 (§712); los veintiun vectores de la bifurcacion en `ancla` la llevan a 236
for m in paquete consumo conflicto ancla rotacion ancla-cofirmada; do bash tools/conformidad.sh tools/segunda/verificador.py spec/vectors/$m/MANIFIESTO.txt | tail -1; done

# 1 quinquies · el verificador STARK: las pruebas ocultas y con sal de seis familias      -> 39 de 39 (§713)
#   ⚠️ §711: decia 34 de 34 (§706); los cuatro negativos de la prueba mal formada, dos de la edad y dos del pago, la llevan a 38
#   ⚠️ §713: decia 38 de 38 (§711); el negativo capturado del resto FRI, `edad/neg-prueba-resto-ajeno`, la lleva a 39
python3 tools/segunda/stark.py          # autotest: la extension, INV_MDS, las raices, la marca, el orden de Rust, el resto atado
python3 tools/segunda/juez_stark.py

# 1 sexies · la cadena del registro del cable (NO es núcleo: spec/RPC.md)       -> cadena: 6 de 6
python3 tools/segunda/juez_cadena.py                          # spec/vectors/zkssl-0.4.json
python3 tools/segunda/juez_cadena.py spec/vectors/zkssl-0.*.json   # los cuatro: 24 de 24
python3 tools/segunda/juez_cadena.py respuesta.json   # o el `result` de zkssl_logEntries, desde seq 0
# 1 septies · el segundo consumidor del cable, con el arnes y el manifiesto del testigo     -> 27 de 27 (§850)
#   ⚠️ §850: decia 23 de 23 (§809), de antes de los tres negativos de la escritura y de `no-existe.json`
bash tools/conformidad.sh tools/segunda/cable.py spec/vectors/cable/MANIFIESTO.txt | tail -1
# 1 octies · el medio: ML-DSA-44 de FIPS 204, la nota checkpoint y la inclusion de RFC 9162   -> 92 de 92 (§859), ~2 s
#   ⚠️ §859: decia 87 de 87 (§810); los cinco casos del bloque 7 lo llevan a 92
python3 tools/segunda/juez_medio.py

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

⚠️ §815: el kit 0.4.3 trae tres vectores de completitud, cada uno con su cierre, y la salida de
`python3 tools/segunda/juez_cabezas.py` sobre el árbol del §815 es 488 de 540 cabezas, 10 de 15
cofirmas, 409 ficheros, 57 con alguna que no verifica y 0 positivos: tres cabezas más, y las tres
verifican. Las cifras de arriba son las del §809.

⚠️ §850: los tres negativos del cable que trae el §850 (`cable/rechazo-index-no-minimo.json`,
`cable/rechazo-quantity-con-ceros.json` y `cable/rechazo-hex-mayusculas.json`) el juez los da `OK`.
Lee la QUANTITY con `int(s, 16)` y el hex con `bytes.fromhex`, que admiten un cero delante y las
mayúsculas; y lo que juzga —la recomposición, el preámbulo, la firma y el índice— no cambia con la
escritura. La salida sobre el árbol del §850 es 491 de 543 cabezas, 10 de 15 cofirmas, 412 ficheros,
57 con alguna que no verifica y 0 positivos, en 29 s, con exit 0: el exit sólo mira que ningún
positivo falle, y un fichero con el código 1 en su manifiesto y todas sus cabezas `OK` no lo cuenta.
Este sello no toca `juez_cabezas.py`: la escritura del cable la juzga `cable.py`, abajo.

**El segundo verificador.** `verificador.py` pasa, con el mismo arnés y los mismos manifiestos
que el binario de referencia, las seis familias que no exigen STARK (⚠️ §810: decía «las cinco»;
la sexta, `ancla-cofirmada`, es del §810):

| familia | entradas | segundo verificador |
|---|---|---|
| `paquete` (posición v1 y v2, extensión; ⚠️ §852: decía 84 y 84 de 84, de antes de los cinco `rechazo-json-*`; ⚠️ §712: decía 89 y 89 de 89, de antes de los cinco del reloj) | 94 | 94 de 94 |
| `consumo` (desde el §771, también entre dos claves con `actas`; ⚠️ §712: decía 17 y 17 de 17, de antes de los tres del reloj) | 20 | 20 de 20 |
| `conflicto` | 17 | 17 de 17 |
| `ancla` (⚠️ §712: decía 22 y 22 de 22, de antes de `neg-ancla-bifurcada-mismo-mmrSize`; ⚠️ §714: decía 23 y 23 de 23, de antes de los veintiún de la bifurcación) | 44 | 44 de 44 |
| `rotacion` (la extensión entre dos claves, con `actas`; §649; y el sobre del solapamiento, §686) | 34 | 34 de 34 |
| `ancla-cofirmada` (la nota del medio con su firma ML-DSA-44, las cofirmas y la inclusión; §810) | 27 | 27 de 27 |

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

⚠️ §810: con `ancla-cofirmada`, seis familias y **201 de 201**, con salida 0 en las seis, sobre el árbol
del §794 (`b6491ef`) y este sello; hasta aquí sus 27 entradas salían `NO SOPORTADO` (0 de 27, medido
con el `verificador.py` del §794). En esa familia la comparación con el binario SÍ se ha hecho, con
`zk-ssl-verify` compilado en este sello (`cargo build --release --locked -p zk-ssl-verify`), sobre el
árbol del §794 y otra vez sobre el del §810, cuyos crates son los del §808: en los 5
positivos la salida es la del binario byte a byte, la huella del ancla, la marca y la huella SHA-256
de la clave de cada testigo incluidas; en los 22 negativos, la línea del `ROJO` es la misma letra a
letra y el código de salida también. Lo que el binario imprime antes del `ROJO` —las líneas `1/4` a
`3/4` a las que llegó— el segundo no lo imprime: junta las líneas y solo las da con el VERDE, en las
seis formas.

Desde el §810 `juzgar` rechaza también una suplente sola (`\ud800`) en una cadena, que serde_json no
lee y `json.loads` sí: sale `ROJO: JSON ilegible: …`, con exit 1, como en el binario. ⚠️ §852: aquí
seguía «No rechaza lo demás que `json.loads` acepta y serde_json no —`NaN`, `Infinity`, `-Infinity` y
un número fuera del rango de un `f64` (`1e400`)—: en un campo que la segunda no lee siguen dando VERDE
donde el kit dice “JSON ilegible”, como ya pasaba en el §794 (medido con `"zz": NaN`, `Infinity` y
`1e400` en `ancla/ancla-derivada.json`)». Ya no es verdad para esas cuatro formas: lo dice el
párrafo siguiente. Con el `verificador.py` del §794, `ancla/ancla-derivada.json` con un campo
`"zz": "\ud800"` salía VERDE, y
el binario da `ROJO: JSON ilegible: unexpected end of hex escape …`. Medido con `zk-ssl-verify`
compilado en este sello, sobre ese sobre y dos de `ancla-cofirmada`, fuera del árbol
—`cofirmada-un-testigo.json` con un testigo más, `x\ud800`, sin línea en la nota, y con un testigo
`testigo.invalid/\ud800` que cofirma, firmado por el firmante de prueba de `juez_medio.py`—: en los
tres, el binario y la segunda dan `JSON ilegible` con exit 1. La letra después de los dos puntos es
otra (aquí, `'utf-8' codec can't encode …`), como en `paquete/rechazo-json-ilegible.json`.

⚠️ §852 · **Los números que serde_json no lee.** Desde el §852 `juzgar` lee con `parse_constant`,
`parse_float` y `parse_int`: `NaN`, `Infinity` y `-Infinity`, y un número que serde_json no lee porque
no cabe en un `f64` —un real como `1e400`, un entero de 401 cifras—, salen `ROJO: JSON ilegible: …`,
con exit 1. La regla del rango es la de `_fuera_de_rango` en `cable.py`, copiada y no importada
(`cable.py` importa `juez_cabezas.py`): compone el `f64` como serde_json, sin el redondeo correcto, y
por eso `1.7976931348623158e308`, que `float()` lee finito, no se lee. Medido con `zk-ssl-verify`
compilado en el §852, con `"zz"` al final de `paquete/posicion-v2.json`, valor a valor: en 26
—las tres constantes, reales y enteros a los dos lados del borde del `f64`, `1e-400`,
`0e999999999999`, `1e2147483648`, `18446744073709551616`—, el binario y la segunda dan el mismo
veredicto, VERDE o `JSON ilegible`; antes del §852, la segunda daba VERDE en los 17 que el binario
rechaza. Cinco entran como vectores del manifiesto `paquete`: `rechazo-json-nan.json`,
`rechazo-json-infinity.json`, `rechazo-json-menos-infinity.json`, `rechazo-json-1e400.json` y
`rechazo-json-entero-401-cifras.json`. No se ha medido otro valor, ni un número en un campo que la
segunda lee.

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

Esa tabla es la medida del §626 y no se ha rehecho. Los párrafos que siguen dicen lo que cambió
cada sello; la cifra de hoy es la del §713: 39 de 39 pares.

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

**Las causas de una prueba mal formada, como el kit (§711).** Desde el §711 el fork de winterfell
valida lo que lee de una prueba antes de construir, y la rechaza con una causa con nombre
(`crates/winter-air/src/causa.rs`): `prueba mal formada: opciones / traza / contexto / consultas /
marco OOD / FRI fuera de rango`, y `lote fuera de rango` en `zk-ssl-air/src/sal.rs`. `stark.py` lee
con las mismas condiciones y repite esas causas; mira el marco OOD en el orden de `OodFrame::parse`
y las capas FRI en el de `VerifierChannel::new`, y su autotest lo fija con cuatro pares sobre
`edad-todos.json` (`pares_fijos_del_orden`). `juez_stark.py` cuenta `prueba mal formada` entre los
mensajes del juez. Lo fijan cuatro negativos de la edad y el pago (`neg-prueba-opciones-cero` y
`neg-prueba-consultas-cero`): con el `stark.py` anterior el juez da 34 de 38 pares
(`UnacceptableProofOptions` y `ProofDeserializationError("queries")`, otra causa que el kit); con
este, 38 de 38, 52 pruebas juzgadas, 35 vectores que caen antes del juez y 33 de 33 falsadores
callados. Donde el kit da un error de deserialización sin nombre, `stark.py` da otro texto de la
misma clase: el resto de sus mensajes no lleva todavía causa con nombre.

**El reloj de la extensión, como el kit (§712).** Desde el §712 el kit juzga, después de la
consistencia, el reloj de la extensión (`el_reloj_avanza`, RFC-0012 D-C): con el mismo `mmrSize`, el
mismo digest; con un `mmrSize` mayor y la misma clave, el índice embebido de la nueva estrictamente
mayor. `verificador.py` tiene el mismo juez, con el mismo nombre y los mismos textos
(`extension: mismo mmrSize y digest distinto` y `extension: el indice embebido no crece`), y lo
llama en el mismo sitio: la extensión, el consumo y el modo 3 del ancla, donde el ancla hace de
vieja. Lo fijan los nueve vectores del §712 en `paquete/`, `consumo/` y `ancla/`, y por eso la
tabla de arriba contaba 94, 20 y 23 (en `ancla`, 44 desde el §714).

**La bifurcación, como el kit (§714).** Desde el §714 el sobre del ancla tiene un modo 5: `rama`,
otra cabeza firmada del mismo operador con el mismo `mmrSize` y otro digest, que sale DETECCIÓN con
salida 0 (`spec/PAQUETE.md` 2.12). `verificador.py` lo juzga en `bifurcacion`, con los mismos textos
y en el mismo orden que el kit. La forma y el modo los decide la PRESENCIA de la clave, como en la
referencia: `"rama": null` es una rama, y `"ancla": null`, un ancla. Antes, `doc.get(k) is not None`
tomaba un `null` por ausente, y en cinco sobres la segunda daba VERDE donde el kit daba ROJO. En
los sobres que leen la cabeza con `leer_cabeza_de` (el ancla, el consumo, el conflicto y el
solapamiento), una cabeza que no es un objeto sale `{sujeto}: no es una cabeza (objeto)`, sin
traceback. El de extensión no pasa por ahí: con `"vieja": 7` la segunda aún revienta, con salida 1,
donde el kit da ROJO con su texto. El de la nota del medio (`ancla-cofirmada`, §810) tampoco pasa
por ahí, y no revienta: con una cabeza que no es un objeto da `falta formatVersion o no es cadena
0x`, el texto del kit. Ningún vector fija esa guarda común. Y
`verificar_cabeza` comprueba ya la firma antes que el preámbulo, como el kit: una firma válida de
otra cabeza es `la firma es VALIDA pero de otro mensaje`, el texto del kit, y no «el preambulo
recuperado no es el esperado». Sobre los 225 vectores de las seis familias que lee, ese cambio de
orden mueve la salida de uno, `ancla/neg-rama-firma-de-otra-cabeza`. Lo demás lo fijan los veintiún
vectores del §714 en `ancla/`, y por eso la tabla de arriba cuenta 44 en `ancla`.

**El resto FRI, atado como en el kit (§713).** El último compromiso FRI es el hash del resto, y
`winter-fri` 0.13.1 no lo comprueba al verificar. Desde el §713 lo comprueba el canal del fork
(`read_remainder`), antes del grado y del plegado, y `stark.py` hace lo mismo en el mismo sitio de
`verificar_fri`: `hash_e2(resto)` distinto del último compromiso es
`FriVerificationFailed(RemainderCommitmentMismatch)`, el mismo nombre que el kit. Su autotest lo
fija con dos pares (`resto_atado`): el negativo capturado de la edad, `neg-prueba-resto-ajeno`, y
`edad-todos.json` con un bit del resto. Con el `stark.py` anterior el juez da 38 de 39 pares, y el
que falla es ese negativo, VERDE; con este, 39 de 39, 53 pruebas juzgadas, 35 vectores que caen
antes del juez y 33 de 33 falsadores callados.

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

**El medio** (§810). `mldsa44.py` verifica ML-DSA-44 desde FIPS 204, y `medio.py` la nota y la
inclusión; `juez_medio.py` los juzga contra lo que escribieron otras manos, 92 de 92 (⚠️ §859: decía
87 de 87, la cuenta del §810; los cinco de la última fila la llevan a 92):

| medida | valor |
|---|---|
| las 33 notas del §632 (30 de torchwood sobre el `crypto/mldsa` de Go 1.27 y 3 del crate `ml-dsa` 0.1.1) con el veredicto de la columna 2 de su `MANIFIESTO.txt`, la variante de `ErrorDeNota` incluida | 33 de 33: 10 aceptadas, 23 rechazadas, y entre ellas las cuatro en que el medio es más estricto que torchwood (la base64 no canónica, dos líneas del publicador, dos veces, y el `origin` ajeno) |
| las vkeys de torchwood, rehechas con el KeyGen del firmante de prueba desde las semillas de `torchwood/main.go` | 2 de 2 |
| las líneas del publicador y del testigo de `pos-01`, `pos-05` y `pos-13`, firmadas aquí en determinista con su marca | 6 de 6, byte a byte con las de Go |
| las firmas de FRONTERA: z con `‖z‖∞` = γ1 − β = 130.994, válida en todo lo demás, y otra con 130.993 | la primera NO verifica y la segunda SÍ, aquí y en el binario |
| una pista donde la parte baja de w̃ es CERO, que ningún firmante de FIPS 204 pone: con una clave a medida (t1 = 0, y w̃ = A·z no depende de c), z con un solo coeficiente, −83.537, y w̃ = 42·2γ2 exacto en un coeficiente, con su única pista ahí | UseHint baja r1: con w1 = r1 − 1 verifica, y con r1 + 1 no |
| la unicidad de la escritura: las pistas desordenadas, una pista repetida, un relleno distinto de cero, un corte que pasa de ω, unos que pasan de ω + k (200, 201, 202 y 255, sin leer fuera de la firma) y el último por debajo del penúltimo, un contexto no vacío, uno de 256 bytes, un bit en cada tercio y un byte de menos | 12 de 12 no verifican, sin una excepción |
| las 98 sondas `inclusion/` de transparency-dev | 6 positivas, 26 que rechaza el tipo y 66 el árbol: las cifras de `vectores_transparency.rs` |
| todo camino de todo árbol de 1 a 33 hojas / con el índice vecino, un hermano tocado, un hash de menos o uno de más | 561 verifican / 2.225 no |
| el `{:?}` de Rust de los nombres que salen en un rechazo (`debug_rust`): once cadenas con U+200B, un acento que se combina, Zs, Zl, Zp, Cf, Co, Cn, Other_Grapheme_Extend y la comilla simple, que no se escapa | 11 de 11 con lo que imprime el `rustc` 1.94.1 que compila el kit |
| cinco reglas que existen y que nada aislaba (§859, abajo): un corte que baja sobre un polinomio sin pistas —la firma de la pista sobre r0 = 0 con los cortes [1, 0, 1, 1]—; tamaño 0 con una raíz que no es SHA-256 de nada, y una marca de 2^63, las dos en notas firmadas a mano con otra clave a medida (t1 = 0) sobre el mensaje que la regla prohíbe; una línea ajena de `pos-05` con un `+` en el nombre; y la vkey del publicador con el `key_id` de otra clave | 5 de 5 rechazadas con su causa (la firma con [1, 0, 1, 1] no verifica; `FirmaInvalida` y `FirmaInvalida`; `Malformada`; `Clave`); y sus 5 controles —la misma entrada sin lo que la regla prohíbe— se aceptan |
| una verificación ML-DSA-44 en Python (`verificar_nota` de `pos-05`, la media de 20 corridas) | de 7 a 18 ms según la carga de la máquina, en tres medidas de este sello; 8,0 ms (de 6,7 a 12,8) con la carga en 4,3 de 4 CPU. El juez entero, de 1,4 a 2 s |

Fuera del canon, porque piden compilar, en este sello: las dos firmas de frontera y una del firmante
de prueba con sal, puestas como cofirma de un testigo más en una copia de
`cofirmada-un-testigo.json` (fuera del árbol), dan en `zk-ssl-verify` lo mismo que aquí —130.994,
`la cofirma de testigo.invalid/frontera no verifica`; 130.993 y la de FIPS, VERDE con 2 testigos—; y
el ejemplo `medio` de `zk-ssl-medio` (`cargo build --release --locked -p zk-ssl-medio --example
medio`), con semillas al azar y árboles de 1, 2, 3, 5, 8, 13, 21 y 34 anclas en su primera, su
central y su última posición, firmó 21 notas con sal y 42 cofirmas con sal: las 21 notas, las 42
cofirmas y las 21 inclusiones verifican en Python, y las 42 cofirmas con un bit cambiado no.

**Los mutantes del propio verificador** (§810), medidos uno a uno sobre una copia del árbol —cambiado
a mano, corridos el juez de 87 y el arnés de `ancla-cofirmada` sin caché de bytecode, restaurado con
el mismo sha256—: de 29, **caen 24**, cada uno con la comprobación que lo vigila, y **viven 5**, abajo.

| mutante | lo que lo mata | juez / arnés |
|---|---|---|
| en `mldsa44.py`: un coeficiente de la tabla de las ζ; el signo de c en SampleInBall; `t1·2^(d−1)`; el prefijo de dominio de μ; ExpandA con r y s cruzados; UseHint al revés; el borde q − 1 de Decompose quitado | las 10 notas aceptadas pasan a `FirmaInvalida` | de 20 a 23 `ROJO` / 16 de 27 |
| SampleInBall con `j ≥ i` en vez de `j > i` | las 3 notas del crate, la cofirma del testigo de `pos-05` y su línea rehecha | 82 de 87 / 26 de 27 |
| la norma con `>` en vez de `≥`, y la norma sin comprobar | la firma de frontera de 130.994 verifica | 86 de 87 / 27 de 27 |
| una cota de menos en la norma | la de 130.993 no verifica | 86 de 87 / 27 de 27 |
| `r0 ≥ 0` en vez de `r0 > 0` en UseHint | la pista sobre r0 = 0: UseHint sube r1, y la firma con r1 − 1 no verifica y la de r1 + 1 sí | 85 de 87 / 27 de 27 |
| las pistas con `>` en vez de `≥` | la pista repetida; las desordenadas no lo matan, porque bajan también con `>` | 86 de 87 / 27 de 27 |
| el relleno de las pistas sin mirar | el relleno distinto de cero | 86 de 87 / 27 de 27 |
| sin la cota `corte > ω` | los cortes por encima de ω + k (200, 201, 202 y 255), que leen fuera de la firma (`IndexError`) | 86 de 87 / 27 de 27 |
| en `medio.py`: los bits sobrantes de la base64 sin mirar tras un `=` | `neg-firma-no-canonica` y `neg-raiz-no-canonica` | 85 de 87 / 27 de 27 |
| el hermano de la inclusión a la derecha | las 98 sondas, sus 12 tocadas y los caminos de 1 a 33 hojas | 84 de 87 / 23 de 27 |
| el `origin` ajeno sin mirar | `neg-origen-ajeno-firmado` y `neg-origen-cambiado` | 85 de 87 / 27 de 27 |
| dos líneas del publicador | las dos `neg-firma-repetida-*` | 85 de 87 / 27 de 27 |
| los caracteres de control sin mirar | `neg-retorno-de-carro` | 86 de 87 / 27 de 27 |
| la hoja sin su `0x00` | la raíz de torchwood de 4 notas | 83 de 87 / 22 de 27 |
| el `debug_rust` que solo escapa los de control | las once cadenas | 86 de 87 / 27 de 27 |
| en `verificador.py`: el publicador de otra clave | `neg-medio-de-otra-clave` sale con exit 0: el ataque | 87 de 87 / 26 de 27 |
| la cofirma sin verificar | `neg-cofirma-tocada` sale con exit 0 | 87 de 87 / 26 de 27 |

`r0 ≥ 0` y el corte por encima de ω solo los matan las firmas que fabrica el juez: ninguna firma de
FIPS 204 lleva una pista donde la parte baja es cero —MakeHint no la pone mientras `‖c·t0‖∞ < γ2`, que
el firmante exige—, y ninguna de las notas lleva un corte por encima de ω.

⚠️ §859: la tabla y los 24 que caen son la medida del §810, con el juez de 87, y no se han vuelto a
medir con el de 92. Los cinco de abajo ya no viven: desde el §859 los mata el bloque 7 del juez, cada
uno solo con su caso, y da 91 de 92 (medido sobre copias de `tools/segunda`, con `python3 -B`); el
árbol sin mutar da 92 de 92. El arnés de `ancla-cofirmada` no cambia: sigue en 27 de 27 con cada uno,
porque ninguna de sus entradas los aísla, y el §859 no le añade ninguna. Lo que sigue es lo que se
escribió en el §810.

Los cinco que viven dan 87 de 87 y 27 de 27: ninguna entrada del juez ni del arnés los aísla.
- En `mldsa44.py`, HintBitUnpack sin `corte < indice`. El corte que baja del juez es el último, que
  deja pistas en el relleno, y lo para la regla del relleno. La regla hace falta: la firma de la
  pista sobre r0 = 0 con los cortes [1, 0, 1, 1] en vez de [1, 1, 1, 1] es la misma h escrita de
  otra manera, y el árbol la rechaza y el mutante la acepta.
- En `medio.py`: el hash del subárbol vacío de `mensaje_cofirmado` sin mirar (la única nota de
  tamaño 0, `pos-00-anclas`, lleva la raíz del árbol vacío); el `+` de `nombre_valido` (ninguna
  línea de firma lleva uno en el nombre, y una vkey no puede: se parte por él); `marca > I64_MAX`
  (`neg-marca-2-63` cae igual, por la firma); y el `key_id` de `leer_vkey` sin comparar (ninguna
  vkey de los vectores lleva el de otra clave).

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
⚠️ §815: `completitud` tiene 81 desde el §815, con los tres del kit 0.4.3: 354 entradas en los diez y 415
en las doce. El segundo verificador no lee la completitud (`NO_SOPORTADOS`), y su juez STARK solo mira
la prenda que va dentro: los tres nuevos no llevan prueba, así que ninguno de los dos cambia.
⚠️ §852: `paquete` tiene 89 desde el §852, con los cinco `rechazo-json-*`: 359 entradas en los diez y
420 en las doce. Lo medido del `.wasm` no cambia: que las pase no está medido, ni antes ni ahora.
⚠️ §711: `edad` tiene 17 y `pago` 15 desde el §711, con los cuatro negativos de la prueba mal formada:
363 entradas en los diez y 424 en las doce. Lo medido del `.wasm` no cambia: que las pase no está
medido, ni antes ni ahora; los cuatro, sueltos, dan bajo `wasmtime` la misma causa que en nativo,
medido sobre el árbol en que se hizo el arreglo y no sobre éste (el asiento del §711).
⚠️ §712: `paquete` tiene 94, `consumo` 20 y `ancla` 23 desde el §712, con los nueve vectores del
reloj: 372 entradas en los diez y 433 en las doce. Lo medido del `.wasm` no cambia: que las pase no
está medido, ni antes ni ahora.
⚠️ §713: `edad` tiene 18 desde el §713, con el negativo capturado del resto FRI: 373 entradas en los
diez y 434 en las doce. Lo medido del `.wasm` no cambia: que las pase no está medido, ni antes ni
ahora; el vector, suelto, da bajo `wasmtime` la misma causa que en nativo, medido sobre el árbol en
que se hizo el arreglo y no sobre éste (el asiento del §713).
⚠️ §714: `ancla` tiene 44 desde el §714, con los veintiún de la bifurcación: 394 entradas en los diez
y 455 en las doce. Lo medido del `.wasm` no cambia: que las pase no está medido, ni antes ni ahora.

## Lo que el segundo verificador no lee, declarado (§693)

`verificador.py` lee siete formas de sobre —la de posición, sin `tipo`, y seis tipos:
`extension`, `consumo`, `conflicto`, `ancla`, `ancla-cofirmada` y `solapamiento`, los de
`TIPOS_LEIDOS`— y declara los otros seis en `NO_SOPORTADOS`: `rechazo`, `edad`, `cobro_pendiente`,
`pago_en_curso`, `prenda` y `completitud`, los que piden una prueba STARK. ⚠️ §810: decía seis formas
y siete tipos declarados, con `ancla-cofirmada` entre ellos, «los que piden una prueba STARK o el
medio»; el medio lo lee desde el §810. Uno de ellos
sale `ROJO: NO SOPORTADO: tipo {tipo}: …`, con exit 1: falla cerrado, y `tools/conformidad.sh` no lo
cuenta nunca: lo da `ROJO <fichero>: NO SOPORTADO - …`
(`spec/PAQUETE.md` §«6. El contrato del mando»). Hasta el §693 el texto era «no lee este sobre
todavia», y el arnés lo contaba como un negativo superado cuando el manifiesto pinaba sólo el
sujeto: `prenda/neg-receptor-mentido`, `prenda/neg-marca-mentida`, `edad/neg-cota-movida` y el
`neg-cota-movida` de `0.3/edad`, medido. Hoy, sobre los manifiestos de esos siete tipos, de `0.3/`
y de `0.4/`, todas las entradas salen `NO SOPORTADO` y ninguna `OK`. La causa de un rechazo se nombra con el texto del binario
(`spec/PAQUETE.md` §«6.1 La causa del rechazo»): `la firma no verifica`, `acuse: RaizDistinta`, y
la lectura de un `u64` con la escritura mínima del §662 y sus textos.
⚠️ §810: medido otra vez sobre los manifiestos de los seis tipos que quedan, de `0.3/` y de `0.4/`:
254 entradas, todas `NO SOPORTADO` y ninguna `OK`; con las 201 que lee, las 455 de los diecisiete
manifiestos que no son del cable.

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
spec/vectors/cable/MANIFIESTO.txt`: 27 de 27 y exit 0, en unos 2 s; el adaptador del testigo,
`tools/cable_respuesta.sh`, da también 27 de 27 (⚠️ §850: decía 23 de 23 en los dos, de antes de los
tres negativos de la escritura y de `no-existe.json`; abajo, «Los dos negativos que pidió»).

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
el testigo sale sin línea `ROJO:` (⚠️ §850: hasta el §850; abajo, al final del punto 6). Con la misma clase, o el texto desde `ROJO:` es el mismo, o el
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

   ⚠️ §850: eso era hasta el §850. Desde él el testigo da en los tres una línea `ROJO: respuesta
   ilegible · …` y exit 1: `no se lee el fichero: No such file or directory (os error 2)`, `no se lee
   el fichero: Is a directory (os error 21)` e `invalid utf-8 sequence of 1 bytes from index 0`, medido
   con unos bytes que empiezan por `0xff`. La entrada `no-existe.json` del manifiesto lo pide, con el
   texto `ROJO: respuesta ilegible · no se lee el fichero`, que los dos consumidores dan.

**Los dos negativos que pidió.** Con las 21 entradas de antes, un consumidor sin la regla del `index`
(§399) o que reduce un `u64` en vez de rechazarlo (RFC-0016) daba 21 de 21: medido, rompiendo en una
copia `cable.py` (`if False and declarado <= embebido`) y `nucleo.u64_canonico` (devolviendo `u % P` antes
de la comprobación de la cota; en lugar del `return u` final no muta nada). Entran dos
negativos, derivados por una mutación del positivo v6 (`aaffc6ceee72f2fb`) y registrados en
`HUELLAS.sha256` con `tools/check_vectores.py --registrar`: `rechazo-index-atrasado.json`, con el
`index` 5, igual al embebido, y `rechazo-n-no-canonico.json`, con `n` = 0x5a0 + p, que recompone el
mismo digest si el lector reduce. Con ellos cada mutante cae en el suyo —22 de 23, «exit 0, el
manifiesto espera 1»—, y los dos consumidores del árbol del §809 daban 23 de 23 (⚠️ §850: decía «del
árbol dan 23 de 23»; desde el §850 dan 27 de 27). Otros once pasos ya los vigilaban las 21, medido
también rompiendo cada uno sobre una copia: sin verificar la firma cae `rechazo-firma`; sin comparar
el preámbulo, `rechazo-v5-con-firma-de-v4`; sin comparar la recomposición, las tres de recomposición;
con la v7 dentro del conjunto, `rechazo-formatVersion-7`, y sin mirar el conjunto, las tres de versión
fuera (1, 0x103 y 7); sin lo que exige cada versión, las cinco `sin …`; sin lo común,
`rechazo-sin-index`; sin las obligadas de la forma, `rechazo-forma-del-cable`; sin el largo del
digest, `rechazo-epochDigest-corto`; sin `result`, `rechazo-sin-result`; y sin la clase `sin-firma`,
`rechazo-available-false`. Lo que ningún vector pina el manifiesto no lo vigila, y rompiendo cada uno
en una copia `cable.py` sigue dando 27 de 27 (⚠️ §850: decía 23 de 23): los límites del lector, a
propósito (el punto 6: la profundidad a 100.000, sin mirar el anidamiento y los sustitutos, o sin el
rango del `f64`); la clave de más (el punto 2); el OID (el punto 5); y el `index` leído canónico (el
punto 3). Con el último roto, `cable.py` da `VERDE` donde el testigo da `ROJO`, y sólo lo miden los
casos de ese punto, juzgados a mano con los dos.

⚠️ §850: aquí decía también «y la escritura mínima de una QUANTITY y de un hex (el punto 4: `0x05`, o
un hex en mayúsculas)», «con uno de los tres últimos rotos» y «no hay vector de un `index` no
canónico, ni de una QUANTITY con ceros delante, ni de un hex en mayúsculas». Desde el §850 los hay de
un `index` con un cero delante, de una QUANTITY con ceros delante y de un hex en mayúsculas; del
`index` no canónico (la cota del punto 3) sigue sin haberlo. Los tres derivan por una mutación de la
escritura del mismo positivo v6, registrados con `--registrar`: `rechazo-index-no-minimo.json`
(`index` `0x06`), `rechazo-quantity-con-ceros.json` (`mmrSize` `0x05`) y `rechazo-hex-mayusculas.json`
(el `epochDigest` en mayúsculas). Los dos consumidores los rechazan al tipar, con `forma del cable`.
Medido en una copia, cada mutante de `cable.py` cae en el suyo: sin el cero delante en `tipar`, 25 de
27 (caen el del `index` y el de la QUANTITY); sin él sólo para el `index`, 26 de 27; sólo para
`mmrSize`, 26 de 27; y con `_hex` que admite mayúsculas, 26 de 27 (cae el del hex). El `index` leído
sin la cota de `u64_canonico` (`int(cab["index"], 16)` en lugar de `q(cab, "index")`) sigue en 27 de
27: ninguno de los tres lo mira. Los otros cuatro de la lista de arriba, rotos igual en una copia,
también dan 27 de 27: la profundidad a 100.000 sin mirar el anidamiento ni los sustitutos, sin el
rango del `f64`, la clave de más y el OID. Con las 23 entradas de antes, los tres primeros daban 23 de
23, medido.

## Lo que la segunda implementación destapó de la spec

Es para lo que existe la entrada 85 del BACKLOG: *«una spec que sólo implementa un código no es
una spec»*. Cinco cosas que quien escriba desde la spec no puede saber (⚠️ §810: decía «cuatro
cosas» y «desde la sección 6»; la quinta es del medio, del §810):

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
5. **Los textos de la nota del medio** (§810). `PAQUETE.md` §5 da `la nota: {e}` con `{e}` «la regla
   de `zk-ssl-medio::nota` que cae», y `el ancla de esta cabeza no esta en la posicion {p} de las {n}
   de la nota: {e}` sin decir `{e}`; el `MANIFIESTO.txt` de `ancla-cofirmada` pina tres de esos textos
   (`checkpoint mal formado`, `ninguna firma del publicador`, `la firma del publicador no verifica`),
   que solo están en el `Display` de `ErrorDeNota`, y los de la inclusión, en inglés, en el de
   `SubtreeError`. `medio.py` los copia del `.rs`, como `airs.py` las AIR. Y uno de ellos nombra mal
   a quien firma: una cofirma de TESTIGO que no verifica sale `la cofirma de {testigo} no verifica:
   la firma del publicador no verifica`, en el binario y aquí.

## En el canon

Los tres jueces y el segundo verificador corren en cada sello (`tools/canon.sh`, bloque «3 duodecies»), a pin cero: un KAT
nuevo o una versión nueva de cabeza que la segunda implementación no reproduzca pone el sello en
rojo con su nombre; el juez de `kat_xmss/` corre en el mismo bloque, para que un cambio de bytes del
crate `xmss` se vea en el sello; el segundo verificador corre contra los seis manifiestos (⚠️ §810:
decía «los cinco»; la sexta, `ancla-cofirmada`, es del §810) —⚠️ §793:
aquí decía «los cuatro», y el bucle del canon (el `for F in …` del bloque «3 duodecies» de
`tools/canon.sh`; ⚠️ §810: hasta aquí lo citaba por su línea, la `:578`, que el §798, el §800 y el
§804 ya habían corrido a la 670, y las catorce que el §810 mete antes en el bloque la llevan a la 684:
se cita por su bloque, que no se mueve) lleva cinco desde el §649—, para
que un vector nuevo tenga que pasar por los dos códigos; y desde el §626 el juez del STARK corre en
el mismo bloque, para que una prueba nueva de esas familias tenga que verificar también en Python.
Desde el §810 corren también `juez_medio.py` y, en el bucle del segundo verificador, la sexta familia,
`ancla-cofirmada`: unos 6 s más. `wasm_runner.py` no corre en el canon: necesita `wasmtime` de PyPI.
Desde el §851 corren también, en el mismo bloque y a pin cero, las dos piezas de `cable.py`: la
cadena del registro, con `juez_cadena.py` sobre los vectores `spec/vectors/zkssl-0.*.json` (un glob:
un catálogo nuevo entra solo), y el segundo consumidor, con `tools/conformidad.sh` y el manifiesto del
cable, el mismo que el bloque «3 bis cable» corre con el adaptador del testigo: unos 3 s más. Las líneas
`SIN EJERCITAR` del juez de la cadena no ponen el sello en rojo; quedan en su salida. ⚠️ §851: aquí
decía «`cable.py` tampoco (§809): ni `RPC.md` ni `spec/README.md`, que lo piden, dicen que el canon
lo corra, y el bloque «3 bis cable» sigue corriendo el adaptador del testigo. Ponerlo junto al segundo
verificador es una fila más del «3 duodecies», a pin cero, y queda por hacer». Y en «Lo medido», bajo
la tabla de la cadena del registro, «No entra en el canon: se corre a mano» vale hasta el §851.
El `--sello` entero se corrió con el bloque dentro el 01-10-2026: VERDE, 1.054 s de tests.

## Lo que NO cubre

- El medio entero: `medio.py` no comprueba la consistencia entre dos tamaños del árbol (la comprueba
  un testigo antes de cofirmar; el sobre no la pide), ni juzga a los testigos, ni la frescura de las
  marcas de tiempo: lo mismo que el kit, que reporta y no juzga (RFC-0013 D-D). Y el firmante de
  prueba de `juez_medio.py` es del juez y no del verificador: firma en determinista para rehacer
  las líneas de Go, y fuera de FIPS 204 para las dos de frontera; la clave a medida de la pista
  sobre r0 = 0 (t1 = 0) la firma cualquiera, y no sirve para otra cosa. Desde el §859, el bloque 7
  firma a mano, con otra clave a medida (t1 = 0, z = 0 y sin pistas), dos mensajes que
  `mensaje_cofirmado` se niega a formar —el de tamaño 0 con una raíz que no es la del árbol vacío y el
  de una marca de 2^63—: los escribe byte a byte `_mensaje_sin_reglas`, del juez, sin ninguna regla.
  Ninguno es un vector: el kit no los ha juzgado en este sello, ni los juzga el arnés.
- El `{:?}` de Rust de un nombre con un carácter asignado después de Unicode 14.0, o que ha cambiado
  de categoría desde entonces. `debug_rust` toma las categorías de `unicodedata`, el Unicode del
  Python que corre (14.0 en 3.11), y la tabla Other_Grapheme_Extend de la 16.0 (la de `regex-syntax`
  0.8.11); el kit, las del `rustc` 1.94.1 que lo compila, la 17.0 (`char::UNICODE_VERSION`). Medido en
  este sello escalar a escalar, los 1.112.064, contra lo que imprime ese `rustc`: 1.097.084 salen
  iguales y 14.980 no, todos escapados aquí y en claro en el kit; 14.979 son Cn en el 14.0 —asignados
  después— y uno, U+1171E, es Mn en el 14.0 y Mc en el 16.0. Ninguno sale en claro aquí y escapado en
  el kit, y ninguno lo cambia la tabla de extensiones de la 16.0 frente a la 17.0. Así, un nombre
  con U+1FAE9, que está entre los 14.979, sale escapado aquí y en claro en el kit. Cambia la letra
  de la causa, no el veredicto: `debug_rust` solo escribe el texto de un rechazo ya decidido
  (`tools/segunda/medio.py:451-452` y `tools/segunda/verificador.py:956-957`). Un `{:?}` exacto
  pediría las tablas de Unicode del kit dentro de la segunda. ⚠️ §852: decía `verificador.py:955-956`;
  el `import re` del §852 baja una línea esas dos.
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
