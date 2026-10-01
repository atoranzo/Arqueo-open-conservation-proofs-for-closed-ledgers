# `tools/segunda` — la segunda implementación: el núcleo, las cabezas, el segundo verificador y el STARK

Los spikes de [`doc/integracion-vertical-evaluacion.md`](../../doc/integracion-vertical-evaluacion.md),
secciones 5.2 y 5.3, medidos el 01-10-2026 sobre `e8ac246` (S615).

| fichero | qué es |
|---|---|
| `nucleo.py` | la **segunda implementación del NÚCLEO** (`spec/NUCLEO.md`, sección 6) en Python, sin dependencias: el campo de Goldilocks, la permutación Rescue-Prime `Rp64_256`, BLAKE3, la serialización, los dominios, los preámbulos y las composiciones, en el orden exacto de los merges |
| `juez_nucleo.py` | el **juez**: corre los 29 KAT de `spec/vectors/nucleo/` contra `nucleo.py` y dice `OK` o `ROJO` por vector, más las `NOTA` sobre lo que la sección 6 deja abierto; es el único llamador de `nucleo.py` |
| `kat_xmss/xmss.py` | la **verificación XMSS^MT de RFC 8391** para `XMSSMT-SHA2_40/8_256`, el conjunto que la casa firma, escrita desde el texto del RFC y de ningún crate; solo verificar |
| `juez_cabezas.py` | el **juez de las cabezas firmadas**: para cada cabeza y cofirma de los vectores recompone el `epochDigest` por versión, compara el preámbulo recuperado, verifica la firma con `kat_xmss/xmss.py` y comprueba el índice embebido (`PAQUETE.md`, 2/3 y sección 8); con un falsador que voltea un byte |
| `verificador.py` | el **segundo verificador**: las cinco formas del paquete que no exigen una prueba STARK —posición v1 y v2 con acuse y cofirmas, extensión, consumo, conflicto y ancla— escritas desde `PAQUETE.md` con el contrato del mando (un argumento, `ROJO: {motivo}` del catálogo, exit 0/1/2), y juzgadas por `tools/conformidad.sh` con los mismos manifiestos que el binario |
| `stark.py` | el **verificador STARK** (§626): la maquinaria de `winter-verifier` 0.13.1 en Python, sin dependencias —el formato de `Proof`, la moneda pública sobre Blake3 en el orden exacto de Fiat-Shamir, la autenticación Merkle por lotes con la sal de `MerkleConSal`, el chequeo fuera del dominio, la composición DEEP, FRI y la envoltura `Oculta`—, leída de las fuentes de winterfell y no copiada; `INV_MDS` se calcula invirtiendo la MDS y el autotest la contrasta con la tabla de `winter-crypto` |
| `airs.py` | las **cinco AIR** del paquete —Banda, Edad, Prenda, CobroPendiente y PagoEnCurso—, **transcritas** de `crates/zk-ssl-air/src/*.rs` restricción a restricción: ninguna RFC escribe sus restricciones y el `.rs` es la única fuente, así que una AIR infra-restringida pasaría aquí igual que allí |
| `juez_stark.py` | el **juez del STARK**: compone, para cada vector con prueba, el par que el mando juzga —la prueba y el enunciado, con los campos y la cabeza que lee `zk-ssl-verify`— y lo compara con su `MANIFIESTO.txt`; tres falsadores por positivo |
| `wasm_runner.py` | el **envoltorio** para correr `zk-ssl-verify` compilado a `wasm32-wasip1` dentro de wasmtime con el contrato del mando (`PAQUETE.md`, sección 6), para que `tools/conformidad.sh` lo juzgue como al binario nativo |
| `kat_xmss/` | el **corpus KAT de XMSS^MT**: claves y firmas producidas por el crate `xmss 0.1.0-pre.0` desde semillas fijas y verificadas por `kat_xmss/xmss.py`, que vive dentro para que el directorio sea un repositorio por sí solo; con su juez y su generador. Es el módulo con perfil de `hbs-state`: extraíble tal cual, sin un byte del protocolo (`kat_xmss/README.md`) |

## Cómo se corre

```bash
# 1 · la segunda implementación contra los KAT del núcleo        -> nucleo: 29 de 29
python3 tools/segunda/juez_nucleo.py

# 1 bis · las cabezas firmadas de todos los vectores, con XMSS^MT desde RFC 8391  -> ~25 s
python3 tools/segunda/juez_cabezas.py

# 1 ter · el corpus KAT de XMSS^MT: dos implementaciones, los mismos bytes       -> 8 de 8
python3 tools/segunda/kat_xmss/juez_xmss.py

# 1 quater · el segundo verificador, con el arnes y los manifiestos del binario   -> 124 de 124
for m in paquete consumo conflicto ancla; do bash tools/conformidad.sh tools/segunda/verificador.py spec/vectors/$m/MANIFIESTO.txt | tail -1; done

# 1 quinquies · el verificador STARK: las pruebas ocultas y con sal de seis familias      -> 23 de 23
python3 tools/segunda/stark.py          # autotest: la extension, INV_MDS, las raices
python3 tools/segunda/juez_stark.py

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

**La segunda implementación.** 29 de 29 vectores reproducidos byte a byte (26 hasta el §643, que
añade los tres del acta), escritos desde la sección 6 de `NUCLEO.md` y de ningún `.rs` de la casa.
La permutación sale de la fuente de `winter-crypto 0.13.1`, que es la primitiva y no la casa, y se
valida en el autotest contra el vector que ese crate atribuye a la implementación Sage; BLAKE3 se
escribe desde la especificación pública y se valida contra 19 salidas del crate `blake3 1.8.5` del
`Cargo.lock`.

**Las cabezas firmadas.** `juez_cabezas.py` recorre los 331 ficheros de `spec/vectors/` que llevan
cabezas, incluidos los catálogos `0.3/` y los del cable:

| medida | valor |
|---|---|
| cabezas firmadas encontradas | 418 |
| que recomponen su `epochDigest`, recuperan su preámbulo, verifican su firma y atan su índice | 376 |
| cofirmas de testigo encontradas / que verifican | 11 / 7 |
| ficheros con alguna cabeza o cofirma que NO verifica | 46, **los 46 negativos** según su `MANIFIESTO.txt` |
| positivos con alguna cabeza que no verifica | 0 |
| falsador: un byte volteado en la firma y otro en el preámbulo | las dos dejan de verificar |
| una verificación XMSS^MT en Python | ~40 ms |

Los 42 fallos son los que los negativos fabrican: 26 cabezas cuyos campos no recomponen el digest,
6 firmas que no verifican, 2 preámbulos que no son el esperado, 2 índices declarados por debajo del
embebido, 2 sin `index`, y versiones fuera del conjunto o sin sus campos. La firma viaja como
RFC 8391 la describe, 18.469 bytes —`idx` (5) ‖ `r` (32) ‖ 8 × (WOTS+ 67×32 ‖ auth 5×32)—,
seguida del preámbulo; la clave son `OID` (4, `0x00000005`) ‖ `root` ‖ `SEED`, 68 bytes, y una
biblioteca que lea el OID como el RFC manda no necesita el apaño del §240.

**El segundo verificador.** `verificador.py` pasa, con el mismo arnés y los mismos manifiestos
que el binario de referencia, las cuatro familias que no exigen STARK:

| familia | entradas | segundo verificador |
|---|---|---|
| `paquete` (posición v1 y v2, extensión) | 72 | 72 de 72 |
| `consumo` | 14 | 14 de 14 |
| `conflicto` | 16 | 16 de 16 |
| `ancla` | 22 | 22 de 22 |

Y en los positivos de las cuatro, su salida es **idéntica línea a línea** a la del binario, la
huella del ancla y la posición derivada del consumo incluidas. Lo que destapó: `NUCLEO.md` §6 no
escribe `posicion_de_consumo` ni la hoja vacía ni la convención de `isRight`, que viven en
RFC-0006 §413 (los 63 bits bajos de los primeros ocho bytes; la hoja vacía es el digest cero;
`isRight[i]` es el bit `i`) y no tienen KAT; y la huella de la clave, que es Blake3, entra en
`ancla_digest` leída como Digest de cuatro elementos, lo que deja sin decir qué pasa si un limbo
queda fuera del campo (2⁻³² por limbo): aquí era ROJO con nombre, y la referencia lo REDUCÍA. Desde
el §640 (RFC-0016, `NUCLEO.md` sección 6, «Canonicidad») la regla está escrita: un productor reduce
—`limbos_reducidos`, como `resumen_con_dominio`— y un lector rechaza lo que no es menor que `p`, con
el mismo texto en las dos implementaciones.

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

Lo que la medida fijó al escribirlo: que la semilla de Fiat-Shamir es el contexto **más las
entradas públicas**, de modo que un enunciado mal compuesto no llega ni a la autenticación Merkle;
que las aserciones de «última fila» de una AIR oculta usan la longitud **interna** T; que las
columnas periódicas de Banda, Prenda, Cobro y Pago miden la traza interna entera (512) y se evalúan
en `z²`; y que `check_leading_zeros` de la moneda cuenta los ceros **finales** de la primera palabra.

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
crate `xmss` se vea en el sello; el segundo verificador corre contra los cuatro manifiestos, para
que un vector nuevo tenga que pasar por los dos códigos; y desde el §626 el juez del STARK corre en
el mismo bloque, para que una prueba nueva de esas familias tenga que verificar también en Python. `wasm_runner.py` no corre en el canon: necesita `wasmtime` de PyPI.
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
- Rendimiento: el `.wasm` no está optimizado para tamaño ni se ha pasado por `wasm-opt`.

## Procedencia

Escrito en una sesión con asistencia de IA generativa, en la rama `claude/nice-planck-ax35zl`,
e integrable a nombre del autor como el bloque ECST. Integrar no es aceptar: la aceptación que
describe [`GENAI.md`](../../GENAI.md) queda pendiente.
