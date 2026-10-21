# El kit del verificador — comprobar sin red, sin repositorio y sin fiarse de nadie

Este documento es el guion de una sola descarga, `arqueo-verify-<versión>-<host>.tar.gz`, con la que
un tercero —un interventor, un auditor, un solicitante— comprueba por sí mismo, en su máquina y sin
red, cuatro cosas en este orden: que un expediente cuadra · que un expediente manipulado **no**
cuadra y el programa **nombra la regla rota** · que la misma etiqueta publicada en **dos libros
distintos** se detecta con las dos cabezas firmadas y los dos nodos apagados · y que un intercambio
de libros se rechaza con su nombre. Desde la 0.3.0, una quinta: que lo que el nodo **recibió**
bajo su firma quedó resuelto en su ventana, o el programa nombra al operador que no lo resolvió. Lo
que va dentro del tarball y por qué está en
`spec/PAQUETE.md`, sección 11, que viaja dentro. Ninguna de las cinco comprobaciones necesita el
repositorio, al autor, un nodo ni una conexión.

Es una CLI y no una página web a propósito: la respuesta a «¿cómo sé que ese programa hace lo que
dice?» es «descárguelo y córralo usted», y sólo vale si la descarga existe y se puede reconstruir desde su commit; con la misma huella, sólo está demostrado dentro de una misma máquina (abajo, «Reproducir el kit, para quien no se fíe de la descarga»). ⚠️ §823: decía «y se puede reproducir», sin la condición de la máquina.

## 0. Descargar, y comprobar la descarga antes de creerla

La release vigente es `arqueo-verify-v0.4.3`, publicada en
<https://github.com/atoranzo/Arqueo-open-conservation-proofs-for-closed-ledgers/releases/tag/arqueo-verify-v0.4.3>,
producida sobre el commit `5fd5212b9f6e04c4c619655ffe969ab04116477a`:
tarball `arqueo-verify-0.4.3-x86_64-unknown-linux-gnu.tar.gz`, sha256
`0bb21f4f5dfe7575264268f281c19648cbf3da38b4fd6809c4eea067592aa4b0` (21.737.865 B, 420 ficheros);
binario `1b120e597897e00530508506b2ade934b042e3fc3f9c1138d895de25d37c8844`. Las dos huellas están
medidas desde fuera —descarga por `curl`, el `digest` del asset en la API, el kit descargado
corriendo sus doce catálogos desde dentro del tarball— en el asiento de `AUDITORIA.md` que registra
esta release (§818). Recompone cabezas hasta la **v6**, la que el nodo firma hoy, y lee doce
familias: las doce de la sección 11 del `spec/PAQUETE.md` que viaja en su tarball, la duodécima la de la rotación (§649), y es la primera release publicada que la lleva. ⚠️ **Corregido en el §807**: decía «las diez de la sección 11 de `spec/PAQUETE.md` y la del ancla cofirmada»; la sección 11 del commit `a4e888d` nombra once familias, el ancla cofirmada entre ellas, y la del árbol doce (`FAMILIAS` en `tools/artefacto.sh`:31). ⚠️ **§818**: hasta aquí este párrafo daba como vigente la `arqueo-verify-v0.4.2`, publicada en <https://github.com/atoranzo/Arqueo-open-conservation-proofs-for-closed-ledgers/releases/tag/arqueo-verify-v0.4.2>, sobre el commit `a4e888d`, con sus dos huellas —abreviadas en su párrafo, abajo; enteras, con sus 16.677.910 B y sus 368 ficheros, en el asiento §671—, y decía «lee once familias: las once de la sección 11 del `spec/PAQUETE.md` que viaja en su tarball, la undécima la del ancla cofirmada (§633, §634); la del árbol cuenta ya doce, con la rotación (§649), que ninguna release publicada lleva», que era lo cierto de la 0.4.2.

⚠️ ~~**La 0.4.3 está CORTADA y no publicada (§815), y es un arreglo de seguridad.** La vigente sigue
siendo la 0.4.2 hasta que el asiento que registre la 0.4.3 fije su huella.~~ ⚠️ **§818: publicada y
vigente.** El autor la publicó con su aviso de seguridad, `GHSA-6p4m-qg2w-8r6j` (<https://github.com/atoranzo/Arqueo-open-conservation-proofs-for-closed-ledgers/security/advisories/GHSA-6p4m-qg2w-8r6j>), que nombra
afectadas la 0.3.0, la 0.4.0, la 0.4.1 y la 0.4.2 y arreglada la 0.4.3, y el §818 la registra con
su huella, arriba. **Es un arreglo de seguridad.** Se redactó en privado
(`spec/rfc/PROCESO.md` §«Fallos de seguridad») y cambia veredictos dentro de `0.4.x`, que es lo que
la R6 f del RFC-0021 permite con este aviso y la lista de lo que cambia. **El defecto que corrige:**
la 0.4.2 no acepta ninguna resolución de un recibo de recepción que el nodo rechazó por
`VerificationFailed` o `ProofFailed`. Su sobre de rechazo no lo prueba ningún mando (2.6 de
`spec/PAQUETE.md`), y la `declarada` sale ROJO, «no es de las que el RFC-0007 dejo sin prueba
portable», aunque la D-I del RFC-0007 las declara así y la D-G del RFC-0010 las manda al cuarto
estado. Medido en privado antes del arreglo: un tercero **sin cuenta** manda una prueba basura con
las raíces públicas, el nodo la rechaza con `VerificationFailed` y le da su recibo, y pasada la
ventana —1.440 cabezas— el sobre de completitud sale «NO RESUELTA EN LA VENTANA», un ROJO que
nombra a un operador honrado. Lo tienen todas las releases que leen ese sobre: la 0.3.0, la 0.4.0, la
0.4.1 y la 0.4.2 llevan la misma lista de cuatro causas (leído en sus commits, `65cabfb`,
`65d1e89`, `ff1f06f` y `a4e888d`; medido con la 0.4.2), y la 0.2.0 no lee el sobre. La 0.4.3 las
declara: salida 3, «DECLARADA, NO PROBADA», en la vía directa y en el lote; su sobre de rechazo
sigue sin probarse, porque que la prueba no verificara es la palabra del nodo hasta que el kit la
re-verifique (RFC-0019 E3, el tren 0.5). Ese sobre de rechazo sigue saliendo ROJO, «la causa … no
la prueba este mando», y ese ROJO dice que el sobre no resuelve el recibo, no acusa al operador:
la resolución que el honrado exhibe es la `declarada`. Y `ProofFailed` no la produce hoy ningún
brazo del nodo con recibo —la capa sólo la da al probar; leído en el código—, así que una
declarada con ella es siempre la palabra del operador; entra porque la D-I la declara
(`SECURITY.md` §3.15; la 129 del `BACKLOG.md`). **Lo que cambia, medido** con la 0.4.2 publicada
(binario `7734376463d120b0…`) y la 0.4.3 del árbol sobre los 500 vectores de `spec/vectors/`,
código de salida y última línea de cada uno:

- de los 348 vectores que viajan en el tarball de la 0.4.2, 346 siguen en su ruta con los mismos
  bytes y el mismo código (de los 437 del árbol en su commit, 435); cuatro cambian el texto con el
  mismo 1 —la ventana se nombra con el índice que la firma acredita y su inicio (§681, RFC-0019
  E1)—; y los otros dos, VERDE con la 0.4.2, dan 3 y se conservan, con su veredicto de entonces,
  en `spec/vectors/0.4/completitud/` (§682, RFC-0019 E2: un `StaleState` no resuelve solo);
- de los 65 vectores que el árbol ganó desde `a4e888d`, catorce dan otro código con la 0.4.2: esos
  dos, en su ruta nueva, de 0 a 3; los tres de este arreglo, de 1 a 3
  (`completitud/declarada-verification-failed.json`, `declarada-proof-failed.json`,
  `lote-declarada-verification-failed.json`); los dos gemelos del
  §682, de 0 a 3; la cofirma con una versión que no cabe en un byte, de 0 a 1 (§780,
  `paquete/rechazo-cofirma-versionformato-0x103.json`); y seis de la rotación, de 1 a 0, porque la
  0.4.3 lee `actas` y el sobre `solapamiento`, que la 0.4.2 no conoce (§648, §686: aditivo). Y
  `completitud/neg-vigente-index-inflado.json` sigue en 1, pero con la 0.4.2 es «NO RESUELTA EN LA
  VENTANA» —un ROJO contra el operador que cualquiera fabricaba inflando el `index` declarado— y
  con la 0.4.3 «ventana ABIERTA» (§681).

Los registros de la medida y la lista entera, vector a vector, van en el asiento §815.

**La 0.4.2 sigue publicada y ya no es la vigente** (`arqueo-verify-v0.4.2`, commit `a4e888d`,
tarball `33acdc9a8fe38f03…`, binario `7734376463d120b0…`). Tiene el defecto del aviso de arriba,
que la 0.4.3 corrige (§815): no acepta ninguna resolución de un recibo de recepción que el nodo
rechazó por `VerificationFailed` o `ProofFailed` —la `declarada` sale ROJO y el sobre de rechazo no
lo prueba ningún mando—, así que, si cualquiera, sin cuenta, manda una prueba basura al nodo, el
operador honrado no tiene con qué resolver ese recibo y, pasada la ventana, queda nombrado, «NO
RESUELTA EN LA VENTANA». Y la lista de lo que cambia, arriba, mide otros que la 0.4.3 corrige: da
VERDE a un `StaleState` que no resuelve (§682) y a una cofirma con una versión que no cabe en un
byte (§780), y da «NO RESUELTA EN LA VENTANA», un ROJO contra el operador, a un sobre que cualquiera
fabrica inflando el `index` declarado de la cabeza vigente (§681). Use la 0.4.3.

**La 0.4.1 sigue publicada y ya no es la vigente** (`arqueo-verify-v0.4.1`, commit `ff1f06f`,
tarball `505de994f243bc76…`). Tiene tres defectos que la 0.4.2 corrige: lee un `u64` del sobre con
`+` o con ceros a la izquierda (§662); cuenta la misma cofirma repetida como varios testigos en el
paquete v2 (§663); y lee una clave XMSS con el OID alterno `0x00010005` como la publicada, con lo que
da VERDE a un «conflicto entre dos libros» con un solo libro escrito dos veces (§664). ~~Use la 0.4.2.~~
Use la 0.4.3 (§818).

**La 0.4.0 sigue publicada y ya no es la vigente** (`arqueo-verify-v0.4.0`, commit `65d1e89`,
tarball `90184e41299e737a…`). Tiene un defecto que la 0.4.1 corrige: una prueba con nodos de más
dentro de un lote de Merkle verifica igual, con otros bytes (§659). El veredicto sobre el enunciado
no cambia —la prueba retocada prueba lo mismo que la original—, pero sus bytes dejan de identificar
la operación. ~~Use la 0.4.2.~~ Use la 0.4.3 (§818). Las dos leen las mismas once familias y dan los mismos veredictos a
todos los vectores publicados. ⚠️ **§818**: «las dos» son la 0.4.0 y la 0.4.1; la 0.4.3 lee doce y
cambia los veredictos que dice el aviso, arriba.

**La 0.3.0 sigue publicada y ya no es la vigente** (`arqueo-verify-v0.3.0`, commit `65cabfb`,
tarball `06648502e0171fea…`). Tiene cuatro defectos que la 0.4.0 corrige, descritos en la sección 3.9
de `SECURITY.md`: un campo hex con un carácter multibyte, o una prueba oculta con el meta vaciado, la
hacen salir con **101** (pánico) en vez de ROJO (§650, §651); una prueba con bytes de cola se lee
igual que sin ellos (§653); y acepta los cuatro negativos de la lectura canónica (RFC-0016, §640).
Falla con un código que no es el suyo, nunca con un VERDE falso para esos casos; aun así, use la
~~0.4.2~~ 0.4.3 (§818).

⚠️ **En la 0.3.0, una frase vieja viaja en su `NOTICE`, declarada; la 0.4.0 lleva el corregido.** El `NOTICE` de aquel tarball dice que la
ocultación del testigo del fork de winterfell la enciende sólo `Prover::ocultacion`, «que ningún
probador de ARQUEO devuelve», y que el fork toca «ocho ficheros y dos nuevos». Era la descripción
del §534; desde el §538 todos los probadores de ARQUEO la encienden, y desde el §575 el fork toca
once ficheros y tres nuevos. No cambia lo que el verificador comprueba -verifica las pruebas con la
ocultación encendida, y sus catálogos lo miden-, pero es una descripción falsa del fork. Un tarball
publicado no se corrige: el `NOTICE` del árbol se corrigió en el §623, y la próxima release lo
lleva.

**La 0.2.0 sigue publicada y ya no es la vigente** (`arqueo-verify-v0.2.0`, commit `1528943`,
tarball `2fe9030a310a1e0b…`). Recompone cabezas hasta la v4: con los vectores de su propio tarball
funciona, y los sobres que nacieron después —`edad`, `prenda`, `cobro_pendiente`, `completitud` y el
`ancla`, con cabezas v5 y v6— los rechaza con salida 1 («tipo desconocido»). Falla cerrada: nunca da
un VERDE falso (medido el 2026-09-30; `BACKLOG.md`, entrada 107).

Cada release lleva un tag y se produce sobre el commit que su fichero `VERSION` nombra; la huella
del tarball se publica **con su commit al lado**, en la página de la release y en el asiento de
`AUDITORIA.md` que la selló, nunca como número suelto. Con el tarball en la mano:

```bash
sha256sum arqueo-verify-*.tar.gz          # tiene que ser la huella publicada junto al commit
tar xzf arqueo-verify-*.tar.gz && cd arqueo-verify-*/
sha256sum -c SHA256SUMS                   # cada fichero de dentro, contra su huella
cat VERSION                               # commit, describe, toolchain, glibc_max
```

Requisitos: Linux x86_64 y una glibc igual o mayor que la que `VERSION` declara en `glibc_max`. El
binario no es estático, y se dice. El contrato del mando es de una línea: `./zk-ssl-verify
<fichero.json>`; sale `0` y escribe `VERDE: …` si el fichero se sostiene, `1` y el **primer** fallo
con nombre (`ROJO: …`) si no, `2` si el uso es incorrecto, y `3` en el cuarto estado del sobre de
completitud, «resolución declarada, no probada», que se escribe con su nombre y sin `ROJO`, porque
no lo es. ⚠️ §801: hasta aquí decía «sale 0 … 1 … 2 si el uso es incorrecto», y no contaba el 3, que
el mando da desde el §573 (`spec/PAQUETE.md` §«6. El contrato del mando»; RFC-0021, hecho 6).

Qué de esto se promete —qué no cambia dentro de una línea `0.N.x`, qué lo rompe y qué sigue
verificando después de una ruptura— lo dice `spec/RPC.md` §«Versionado y compatibilidad» (§801):
ningún kit publicado se retira, cada uno se reconstruye desde su commit, y lo que juzgaba lo sigue
juzgando igual; un arreglo de seguridad puede cambiar un veredicto dentro de `0.N.x`, con su aviso
y la lista de lo que cambia, como la 0.4.2. ⚠️ §823: decía «cada uno se reproduce desde su commit»,
sin la condición de la máquina: con la misma huella sólo está demostrado en una misma máquina, y la
0.4.2 rehecha desde su commit en otra sale con otra (abajo, «Reproducir el kit, para quien no se fíe
de la descarga»). Lo que juzgaba sí lo juzga igual: la rehecha pasa sus once catálogos.

## 1. Un expediente que cuadra

```bash
./zk-ssl-verify spec/vectors/paquete/posicion-v2.json
```
Esperado: `VERDE: el paquete se sostiene sin el nodo`, código de salida 0. El fichero es una
posición real capturada de un nodo que después se apagó: la cabeza de época firmada, el acuse con
su camino, y las cofirmas. El verificador recompone el `epochDigest` desde los campos, verifica la
firma, sube el camino hasta la raíz y no pregunta nada a nadie.

```bash
./zk-ssl-verify spec/vectors/consumo/consumo.json
```
Esperado: `VERDE: el consumo se publico entre las dos cabezas, sin el nodo`. Es el sobre de
consumo: dos cabezas firmadas del mismo libro, la ausencia de la etiqueta bajo la vieja y su
presencia bajo la nueva. Dentro de un libro, una unidad se consume una vez y el hecho queda
publicado en la cabeza.

## 2. Un expediente manipulado no cuadra, y el programa nombra la regla rota

```bash
./zk-ssl-verify spec/vectors/paquete/rechazo-n-adulterado.json; echo "salida $?"
```
Esperado: salida 1 y `los siete campos NO recomponen el epochDigest empaquetado`. Un campo de la
cabeza fue alterado tras firmarla; el digest ya no cuadra y el programa dice cuál es la regla, no
sólo que falla.

```bash
./zk-ssl-verify spec/vectors/consumo/rechazo-cons-ausencia-ya-estaba.json; echo "salida $?"
```
Esperado: salida 1 y `el consumo YA estaba`: la misma etiqueta presentada como nueva cuando la
cabeza vieja ya la tenía. Es el doble uso dentro de un libro, nombrado.

Y el catálogo entero, con el arnés que viaja dentro: cada entrada de cada manifiesto dice el código
de salida y el texto que el binario tiene que emitir, y el arnés lo comprueba entrada a entrada:

```bash
bash conformidad.sh ./zk-ssl-verify                                        # el paquete
bash conformidad.sh ./zk-ssl-verify spec/vectors/consumo/MANIFIESTO.txt    # el consumo
bash conformidad.sh ./zk-ssl-verify spec/vectors/conflicto/MANIFIESTO.txt  # el conflicto
```
Esperado, al final de cada uno: `conformidad: N de N entradas dicen lo que deben`, con el mismo N a
los dos lados. Los vectores negativos se derivaron por mutación de capturas reales y no se
reescriben nunca; si alguno dijera otra cosa, el arnés lo nombra.

## 3. La misma etiqueta en dos libros distintos, con los dos nodos apagados

```bash
./zk-ssl-verify spec/vectors/conflicto/conflicto.json
```
Esperado: `VERDE: dos libros aceptaron el mismo consumo. Es DETECCION, no prevencion:` y, debajo,
lo que cada libro firmó. El sobre lleva **dos cabezas firmadas por dos claves distintas** —dos
libros— y, para cada una, el camino que sube desde la misma etiqueta hasta la raíz de consumos que
esa cabeza firma. Ningún nodo participa: la captura se hizo con los dos nodos vivos y se comprueba
con los dos apagados. Es exactamente el hecho que un organismo necesita ver cuando la misma factura
se certifica dos veces ante dos ventanillas.

## 4. Un intercambio de libros se rechaza con su nombre

```bash
./zk-ssl-verify spec/vectors/conflicto/rechazo-conf-camino-no-sube.json; echo "salida $?"
./zk-ssl-verify spec/vectors/conflicto/rechazo-conf-misma-clave.json; echo "salida $?"
```
Esperado: salida 1 en los dos. El primero es una captura tal cual con los caminos de los dos libros
**intercambiados**: `libro[0]: el camino NO sube al consRoot de su cabeza`. El segundo presenta el
mismo libro dos veces: `las cabezas llevan la MISMA clave`. Un sobre de conflicto exige dos libros
de verdad, y dice cuál de las dos reglas se rompió.

## 5. Un recibo bajo la firma del operador, y lo que hizo con él (desde la 0.3.0)

```bash
./zk-ssl-verify spec/vectors/completitud/resuelta-por-acuse.json; echo "salida $?"
./zk-ssl-verify spec/vectors/completitud/no-resuelta.json; echo "salida $?"
```
Esperado: el primero, salida 0 y `VERDE: el recibo se resolvio dentro de la ventana, y se sostiene
sin el nodo`, con `RESUELTA como transicion aplicada` en su tercer paso; el segundo, salida 1 y `NO
RESUELTA EN LA VENTANA`. Es el sobre de completitud (`spec/PAQUETE.md` 2.11, RFC-0010): el recibo
que el nodo emitió al RECIBIR una operación queda bajo una raíz que su cabeza firmada compromete, y
el sobre dice, sin el nodo, si la resolvió dentro de su ventana -aplicada, o rechazada con prueba- o
nombra al operador que no lo hizo. El segundo es eso: la promesa firmada por el acusado, una cabeza
suya fuera de la ventana y ninguna resolución exhibida. No es una prueba criptográfica de ausencia,
sino evidencia oponible: el operador la desmiente exhibiendo la resolución.

```bash
./zk-ssl-verify spec/vectors/completitud/lote-aplicado.json; echo "salida $?"
./zk-ssl-verify spec/vectors/completitud/prenda-sin-fundamento.json; echo "salida $?"
```
Esperado: salida 0 y `RESUELTA como LOTE aplicado` -un lote de dos envíos con pruebas reales que
mandó un agregador, con el acuse de cada prueba-; y salida 1 y `RECHAZO SIN FUNDAMENTO`: el sobre de
una prenda que el verificador juzga OTRA VEZ, con el mismo juez y contra la cabeza que el nodo
juzgó, y que verifica, frente a un nodo que dijo que no (RFC-0014). Ese vector se derivó por
mutación de una captura real -un nodo honrado no lo produce- y su manifiesto lo declara.

```bash
bash conformidad.sh ./zk-ssl-verify spec/vectors/completitud/MANIFIESTO.txt
```
Esperado: `conformidad: N de N entradas dicen lo que deben`. La 0.2.0 no conoce este sobre y
rechaza los cuatro ficheros con `tipo desconocido`: falla cerrada.

## Lo que esto dice, y lo que no

- **Detecta, no impide.** Dos libros soberanos pueden aceptar la misma etiqueta; nadie ordena entre
  ellos. Lo que el kit demuestra es que un tercero con las dos cabezas firmadas lo VE, después.
- **La ventana no es tiempo de reloj.** Cuando un nodo carga la cabeza firmada de otro libro para
  no tramitar lo que ese libro ya tiene, lo que aplica es una **frontera en el `seq` firmado** de
  esa cabeza, con dos caras: no bloquea nada que ese libro haya firmado después de esa cabeza, y no
  sabe si esa cabeza ya fue superada.
- **La etiqueta es gobernanza.** Para que dos organismos detecten la misma factura, los dos tienen
  que calcular la etiqueta igual, y eso es un acuerdo entre ellos, no una propiedad del código.
- **El límite oráculo.** Las pruebas hablan del libro, no del mundo: una factura nace fuera; lo que
  se prueba es que la misma etiqueta se publicó, no que sea la misma factura real.
- **Dentro de un libro, el uso único es un invariante**; entre libros, lo que hay es lo de arriba.
  El diseño entero, con sus decisiones y lo que descarta, está en el RFC-0006
  (`spec/rfc/0006-consumo-publicado.md`) y en `SECURITY.md`.

## Reproducir el kit, para quien no se fíe de la descarga

El tarball se produce sobre el commit que `VERSION` nombra: con el `rustc` que `VERSION` declara,
`git checkout <commit>` y `bash tools/artefacto.sh` vuelven a dar el mismo binario (misma huella,
compilado con `--remap-path-prefix`) y el mismo tarball **en la misma máquina**; el `--check` de
`tools/artefacto.sh` que corre el canon en cada sello, en `b6994c3`, compila el mismo árbol dos
veces, en dos `target`, y busca en el binario sólo las rutas de `$HOME`. ⚠️ §823: decía «vuelven a dar
el mismo binario (misma huella, compilado con `--remap-path-prefix`) y el mismo tarball, y
`tools/canon.sh` comprueba esa propiedad en cada sello», sin la condición de la máquina. Lo medido el
15-10-2026 (§823), salvo las 41 rutas del registry de (ii), contadas y comparadas el 21-10-2026:

- **En una misma máquina, sí.** El nodo y el testigo de `b6994c3`, compilados con los flags de
  `tools/artefacto.sh` desde dos árboles, en otro `target` y con otro `HOME` y otro `CARGO_HOME`,
  salen idénticos byte a byte las cuatro veces (`rustc` 1.94.1): el nodo `ac27f9052c7f74c1…`, con
  `dev` y compilado junto al testigo (`-p zk-ssl-node -p zk-ssl-cli`), y el testigo
  `07d659c6291ad68f…`. Sin el remap, dos compilaciones desde dos árboles también salen iguales entre
  sí, pero con otra huella y con las rutas de `$HOME` dentro: lo que el remap quita es la
  dependencia de `HOME`, con `CARGO_HOME` y `RUSTUP_HOME` bajo él (fuera de él, sus rutas entrarían
  enteras, deducido de las reglas y sin medir: abajo, (i) y (ii); ⚠️ §824: medido después en el kit, y entran: abajo, el párrafo «Desde el §824, la herramienta»); la de la ruta del árbol no
  aparece, porque cargo le pasa a `rustc` las rutas de los crates del workspace relativas a su raíz.
- **Desde otra máquina, la 0.4.2 no sale.** Rehecha desde `a4e888d` con su `rustc` 1.97.1, el
  binario es `14832a067f6dc4cf…` y no `7734376463d120b0…`: 13 de sus 35 secciones distintas. Con
  `rust-src` instalado, un toolchain llamado `stable` y `RUSTUP_HOME` bajo `HOME`, como dicen las
  rutas que lleva el publicado, quedan 3 (329 bytes): el `build-id`, 240 bytes de `.text`, todos en
  las funciones en ensamblador de blake3, y la `.comment`, que en el publicado nombra GCC 15.2.0 y
  aquí 13.3.0. La 0.4.3, la vigente desde el §818, no consta rehecha con su `rustc` 1.97.1 fuera de
  la máquina que la produjo; compilada aquí desde su código con el 1.94.1, como la compiló el canon
  del §817, sale con otra huella. Su binario publicado, `1b120e597897e005…`, lleva lo mismo que
  separó a la 0.4.2 de su rehecha: las mismas once rutas de `rust-src` y una `.comment` que nombra
  GCC 15.2.0 (§823, leído con `strings` y `readelf`, sin ejecutarlo).
- **De qué depende la huella, además del `rustc` y del `Cargo.lock`.** (i) Si `rust-src` está
  instalado, las rutas de la biblioteca estándar entran en el binario con el nombre de su toolchain,
  porque ninguna regla de `b6994c3` las lleva a `/rustc/<commit-hash>`: con `RUSTUP_HOME` bajo
  `HOME`, la de `$HOME` sólo les cambia el prefijo, y con `RUSTUP_HOME` fuera de `HOME` entrarían
  enteras (deducido, sin medir; ⚠️ §824: medido después en el kit, entran once, enteras: abajo, el párrafo «Desde el §824, la herramienta»); la 0.4.2 lleva once
  `/home/x/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/…`. Un remap más,
  de `<sysroot>/lib/rustlib/src/rust` a `/rustc/<commit-hash>`, lo neutraliza: con él, la 0.4.2
  compilada con `rust-src` sale idéntica a la compilada sin él. (ii) Con los flags de `b6994c3`, la
  regla del registry no gana nunca: `rustc` aplica la última regla que casa, la de `$HOME` va la
  última, y con `CARGO_HOME=$HOME/.cargo` las rutas del registry entran como
  `/home/x/.cargo/registry/src/…`. Eso no es una causa de lo medido: con `CARGO_HOME=$HOME/.cargo` en
  las dos máquinas da los mismos bytes, y el publicado y la rehecha llevan las mismas 41 rutas
  `/home/x/.cargo/registry/src/…`. Es una dependencia que lo medido no toca: el script de `b6994c3`
  deriva tres de sus reglas de `$HOME`, no de `CARGO_HOME`, y con un `CARGO_HOME` fuera de `HOME`
  ninguna casaría con sus rutas, que entrarían enteras (deducido, sin medir; ⚠️ §824: medido después en el kit, entran 41, enteras: abajo, el párrafo «Desde el §824, la herramienta»). (iii) El código nativo
  que compila el host —el ensamblador de blake3 en el kit, en el nodo y en el testigo, y el C y el
  ensamblador de ring en el testigo— depende de su `cc` y de su `as`, que el `VERSION` de la 0.4.2 no
  nombra. Que con el mismo `cc` y el mismo `as` la 0.4.2 saliera idéntica no está medido: aquí no hay
  una máquina con los suyos. Las causas de las 13 secciones son dos, (i) y (iii); la (ii) no pesó.

**Desde el §824, la herramienta.** `tools/artefacto.sh` saca las reglas del remap de un solo
productor, `reglas()`, que leen el kit, el nodo y el testigo: cinco, derivadas de `CARGO_HOME` y del
sysroot que dice `rustc`, y ordenadas de la más corta a la más larga, porque `rustc` aplica la
última que casa y así gana la más específica. El registry entra como `/registry/…`, y `rust-src`, si
está instalado, como `/rustc/<commit-hash>/…`, lo que `rustc` escribe cuando no lo está. `cargo`,
`rustc` y el `cc` corren con `env -i` y sólo las variables que hacen falta para encontrar el
toolchain, el registry y la red (`PASAN`, en la herramienta; `VERSION` las nombra): `CC`, `CFLAGS`,
`CC_<target>`, `RUSTC_WRAPPER`, `CARGO_PROFILE_RELEASE_*`, `CARGO_ENCODED_RUSTFLAGS` y las demás que
cambiarían el binario sin que `VERSION` lo dijera no llegan, y un fichero de configuración de cargo
que traiga `[profile]`, `[env]`, `[target]`, `[host]`, `[build]`, `[source]`, `[patch]` o
`[unstable]` es ROJO. Cada `target` lleva el sello del host con que se compiló —la herramienta, el
`rustc`, el `cargo`, el `cc`, el `cc1`, el `as`, la glibc y los objetos de arranque—, y si no es el
de ahora se vacía antes de compilar, porque `cargo` no recompila cuando cambia el `cc` del host ni
cuando cambia una variable que un build.rs leyó sin declararla. `VERSION` nombra además el `cc`, el
`as` que ese `cc` llama, la glibc del host y la huella de los objetos que el `cc` enlaza siempre. El
`--check` compila el kit en dos raíces —el árbol y una copia de lo que git sigue o no ignora, en
`target/artefacto/raiz2`—, comprueba que cada binario salió de las fuentes de su raíz (el dep-info
que cargo deja al lado las nombra), exige la misma huella, y busca en los dos binarios las rutas de
las dos raíces, de `HOME`, de `CARGO_HOME`, de `RUSTUP_HOME` y, del sysroot, `<sysroot>/lib/rustlib`,
enteras, sin exigirles la barra de detrás —un `$HOME` desnudo dentro del binario es tan de la
máquina como `$HOME/algo`, y es igual en las dos raíces—, y la de `/home/x/`, que sólo aparece si
una regla general le gana a una específica. Si `strings` falla o no saca nada, o `grep` sale con un
error, es ROJO, no «ninguna ruta». Si las huellas difieren, dice qué secciones del ELF difieren.
Medido con `rustc` 1.94.1, sobre las fuentes de `b6994c3` (el §812), en las que se escribió y se
revisó este sello, y con tres versiones de la herramienta —la primera, el 15-10-2026, antes de los
cuatro arreglos de su primera revisión; la segunda, el 20 y el 21-10-2026, sin el entorno limpio ni
el sello del host; y la que se sella, el 21-10-2026—, con las mismas huellas en las tres; lo que se
volvió a medir sobre el árbol del §817, en el penúltimo punto, y sobre el de este sello, en el último:

- **El kit del árbol sale igual desde dos raíces**, `326d232b81250581…`, y el mismo con `rust-src`
  instalado en otro `RUSTUP_HOME`, con `CARGO_HOME` fuera de `HOME`, y con las dos cosas y otro
  `HOME` a la vez (esto último, también con la que se sella). Con la herramienta de `b6994c3`,
  compilando en un `target` vacío, `rust-src` lo llevaba a `a5c68056442c3d83…`, con once rutas del
  sysroot dentro, y un `CARGO_HOME` fuera de `HOME`, a `a0367207f77c57cf…`, con 41 rutas suyas. La
  búsqueda de su `--check` (`strings -n 8 | grep -c "$HOME"`, en su línea 96), aplicada a esos dos
  binarios, da 0: no las habría visto; su `--check` entero no se corrió así. Con el `target` ya
  compilado, `cargo` no recompila cuando sólo cambia el sysroot, y con `rust-src` salía el
  `311df1aa…` de siempre.
- **El entorno ya no entra.** Con la segunda versión, `CC=clang` daba otro kit,
  `7cb169729272c13a…`, y `VERSION` seguía nombrando el gcc (lo midió su revisión); con un mutante
  de la que se sella sin el `env -i` de `construir()`, lo mismo, y también con sólo
  `CC_x86_64-unknown-linux-gnu=clang`, que cc-rs lee y bash no sabe quitar. Con la que se sella, y
  diez variables así a la vez —`CC`, `CFLAGS`, la del guion, `TARGET_CC`, `RUSTC_WRAPPER`,
  `RUSTC_BOOTSTRAP`, `CARGO_PROFILE_RELEASE_OPT_LEVEL`, `CARGO_INCREMENTAL`, `CARGO_FEATURE_PURE` y
  `CARGO_ENCODED_RUSTFLAGS`—, sale `326d232b81250581…` y el mismo tarball que sin ellas. El entorno
  limpio solo no bastaba: sobre un `target` que ese mutante había compilado con la variable del
  guion, la que se sella sin el sello del host daba el kit de clang, `7cb169729272c13a…`, con
  `VERSION` nombrando el gcc, porque cc-rs no declara esa variable y `cargo` no recompiló; con el
  sello, el `target` se vacía y sale `326d232b81250581…`. Un `cc` que es otro binario —un envoltorio
  que llama al mismo gcc— también lo vacía, y sale la misma huella.
- **La huella del kit del árbol se mueve con este sello.** Con las fuentes de `b6994c3`, el binario
  pasa de `311df1aa4a70f4cb…` a `326d232b81250581…`; con las del árbol del §817, de
  `a0ae4bfdddbaa207…` a `cc2077db8a608fe2…`, y con las de este sello, de `3923dcc03198efb7…` a
  `2d94c323542bbe2d…` (abajo), porque las rutas del registry pasan de
  `/home/x/.cargo/registry/src/…` a `/registry/…`. El tarball se mueve además por las líneas de
  `VERSION` que pone la herramienta —las nuevas, el `cc`, el `as`, la glibc del host, los objetos de
  arranque, el entorno de cargo y cómo se ordenan las reglas, y la de `rustflags`, que pasa de
  cuatro reglas a cinco, con otros nombres—, aparte de lo que cambia con cada commit: `commit=`,
  `describe=` y, en este sello, `spec/PAQUETE.md`, que viaja dentro. Un kit publicado se
  reconstruye con el `tools/artefacto.sh` de su commit, no con el de este árbol (con la misma
  huella, sólo dentro de una misma máquina): los flags son parte de lo que se compila. La 0.4.3,
  publicada (§818), se produjo sobre `5fd5212` (el §817) con el `tools/artefacto.sh` de ese commit,
  que es el del §815, y no con el de este sello: su `VERSION` no lleva las líneas nuevas, se
  reconstruye con aquél, y su huella es la de la sección 0.
- **El coste del `--check`**, en cuatro núcleos, con la carga media del minuto entre paréntesis: con
  la primera versión, 164 s en frío, 41 s sin cambios y 48 s con un cambio en
  `crates/zk-ssl-verify` (de 2,6 a 8,8); con la segunda, 66, 23 y 29 s (de 2,8 a 3,8); con la que se
  sella, de 62 a 65 s en frío —y lo es la primera pasada tras cambiar la herramienta o el host— y
  24 s sin cambios (de 2,3 a 3,9). El de `b6994c3`, 156 s en frío y 51 s sin cambios (de 6,4
  a 9,1).
- **Sobre el árbol del §817** —el de antes de rebasarlo sobre el §839 al §842, con los borradores
  del §822 y del §823 encima, el árbol al que se portó primero este sello; ni aquellos cuatro ni
  estos dos tocan lo que compila el kit—, el 21-10-2026, en cuatro núcleos con la carga media del
  minuto de 4,9 a 6,8 y un `target/artefacto` vacío. Lo que compila `-p zk-ssl-verify` —sus crates
  del árbol y el `Cargo.lock`— no cambia desde el commit del §815, ni frente a `5fd5212`
  (`git diff`, vacío): es el kit de las fuentes de la 0.4.3. Con la herramienta de antes de este
  sello, que sólo cambia comentarios de la del §815, sale en esta máquina
  `a0ae4bfdddbaa20739fbe83b1bf092544c465c25d98a683f94ac5a5cf60e174a` (2.587.048 B, 78 s), con 41
  rutas `/home/x/.cargo/registry/src/…`; no es la huella de la 0.4.3 publicada, `1b120e597897e005…`
  (la sección 0), producida en otra máquina con `rustc` 1.97.1. Con la de este sello,
  `cc2077db8a608fe253e441c5fe6abb90a1e0e7ed7d467dac0899448e52aca98b` (2.586.408 B, 64 s), con esas
  41 como `/registry/…` y 12 de sus 35 secciones distintas. Los dos pasan los doce manifiestos
  —paquete 84/84, consumo 17/17, conflicto 17/17, rechazo 84/84, edad 15/15, pendiente 10/10, pago
  13/13, prenda 11/11, completitud 81/81, ancla 22/22, ancla cofirmada 27/27 y rotación 34/34—. El
  `--check` de este sello sale VERDE: el mismo binario desde las dos raíces, sin rutas de la
  máquina, y los doce manifiestos también desde dentro del tarball, en 177 s en frío y 53 s sin
  cambios, con casi el doble de carga que arriba. Con las diez variables del segundo punto a la vez
  (`RUSTC_WRAPPER=/bin/false` entre ellas), el mismo binario y el mismo tarball, en 56 s como mucho:
  la cifra cuenta también la espera a que acabara otra compilación de la máquina.
- **Sobre el árbol de este sello** —medido con el §852 de `main` y el §819 al §823 encima; sobre el
  §859 lo que compila el kit es lo mismo, porque del §853 al §859, en `crates/`, en el `Cargo.toml` y
  en el `Cargo.lock` sólo cambian dos guiones de `crates/zk-ssl-medio/tests/`, que el kit no compila,
  y no se volvió a medir—, el 21-10-2026, en cuatro núcleos con la carga media del minuto de 5,7 a
  8,3 y un `target/artefacto` vacío. Lo que compila el kit sí cambia desde `5fd5212`, el commit de
  la 0.4.3: el §848 y el §849 tocan comentarios de `crates/zk-ssl-medio/src/lib.rs` y de
  `crates/zk-ssl-hash/src/lib.rs`, y el §848 añade un `[[test]]` al `Cargo.toml` del medio; y el
  paquete tiene cinco vectores más (§852). Con la herramienta de antes de este sello sale en esta máquina
  `3923dcc03198efb7a122420ae649be918ad5c00bdd1a716809488f91c514d9bd` (2.587.056 B), y no
  `a0ae4bfd…`; con el `Cargo.toml` del medio de `5fd5212`, sin el `[[test]]`, sale la misma
  (medido el 22-10-2026): lo que mueve la huella son los comentarios, como dice abajo «De qué
  depende la huella». Con la de este sello,
  `2d94c323542bbe2d73db8d388403f831b3dc4d52f6819f0d56c26f88bb0426c9` (2.586.416 B), con las 41
  rutas como `/registry/…` y 12 de sus 35 secciones distintas, las mismas que arriba. Los dos pasan
  los doce manifiestos, con paquete 89/89 y los demás como arriba, y ninguno es la huella de la
  0.4.3 publicada. El `--check` de este sello sale VERDE, con el mismo binario desde las dos raíces,
  sin rutas de la máquina y con los doce manifiestos también desde dentro del tarball. Los tiempos
  medidos cuentan la espera del cerrojo de `cargo`, que tenía otra compilación de la máquina: 193 s
  el kit con la herramienta de antes, 237 s con la de este sello, y el `--check` 182 s en frío y
  243 s sin cambios; sin la espera, unos 71, 70, 171 y 50 s, estimados por la hora a la que arrancó
  cada pasada.

**De qué depende la huella, con la herramienta de este sello**: del `rustc` (su versión y su commit,
en `VERSION`), del `Cargo.lock`, de lo que se compila y con qué features (el kit, `-p zk-ssl-verify`
solo), y del host: de su `cc` y de su `as`, porque el ensamblador de blake3 lo ensambla el host (en
la 0.4.2, 240 bytes de `.text` distintos), y de los objetos que el `cc` enlaza en todo ejecutable,
los de GCC, `crtbeginS.o` y `crtendS.o`, y los de la glibc del host, `Scrt1.o`, `crti.o` y `crtn.o`,
de libc6-dev. Medido con `nm` en el kit, el nodo y el testigo: `_start` y `__abi_tag` vienen de
`Scrt1.o`, `_init` y `_fini` de `crti.o`, y lo demás de `crtbeginS.o` y `crtendS.o`; de
`libc_nonshared.a`, `libgcc.a` y `libgcc_eh.a` no llevan ningún símbolo, salvo `__floattidf` en el
nodo, que `compiler_builtins` también define (el desenrollado es el de `libgcc_s.so.1`, dinámico).
En la 0.4.2 los objetos de arranque no pesaron: los 240 bytes de `.text` caen todos en seis
funciones de blake3. La `.comment` nombra el GCC del host porque `crtbeginS.o` y `crtendS.o` llevan
la cadena «GCC: …» (medido con `readelf`; los objetos de blake3, `libgcc.a`, `libgcc_eh.a` y
`libc_nonshared.a`, ninguna). El `cc` es el que cc-rs usa por defecto cuando el host es el target,
`cc`, y el mismo con que `rustc` enlaza; `VERSION` lo nombra, y el `as` que ese `cc` llama, la glibc
del host (la primera línea de `ldd --version`) y la huella de los cinco objetos de arranque. El
enlazador no es el del host: la `.comment` del publicado dice «Linker: LLD 22.1.6», y la del kit de
aquí, «LLD 21.1.8», el `rust-lld` de cada toolchain. Ya no de la ruta del árbol, del usuario, de
`CARGO_HOME`, de `RUSTUP_HOME`, de si `rust-src` está instalado ni de las variables del entorno que
no están en `PASAN`: medido arriba. Y sí de los bytes de la fuente, no sólo de lo que compila: una
línea de comentario al final de `crates/zk-ssl-verify/src/lib.rs` lleva el kit de
`326d232b81250581…` a `7f344beb37edc753…`, con el mismo tamaño y dos secciones distintas, los
nombres de los símbolos (`.strtab`) y el `build-id`; el código, igual. Que dos máquinas con el mismo
`rustc`, el mismo `cc`, el mismo `as` y los mismos objetos de arranque den la misma huella sigue sin
demostrar: aquí hay una sola.

Las demostraciones con nodos vivos —levantar dos libros, publicar la misma etiqueta en los dos,
capturar el sobre— son los bancos `tools/banco_dos_libros.sh`, `tools/banco_consumo.sh` y
`tools/banco_apagado.sh`, y los del paso 5, `tools/banco_completitud.sh`,
`tools/banco_mentiroso_sin_resolver.sh` y `tools/banco_recibo_agregado.sh`; no viajan en el kit
porque levantan procesos, y los vectores de arriba son sus capturas.

## El nodo y el testigo, desde dos raíces

`bash tools/artefacto.sh --binarios` (§824) compila el nodo de producción
—`-p zk-ssl-node --no-default-features`, sin `dev`, la emisión con los custodios de prueba— y el
testigo —`-p zk-ssl-cli`— con las mismas reglas, el mismo entorno y el mismo sello del host que el
kit, en las mismas dos raíces, y exige que cada uno salga idéntico byte a byte y sin rutas de la
máquina; si no, dice qué secciones del ELF difieren. Cada uno va en su invocación de `cargo`:
compilados juntos, `cargo` unifica las features y el `zk-ssl` del nodo entraría con la `sandbox` que
pide el testigo. Por eso el nodo de la medida de arriba, con `dev` y compilado junto al testigo, es
otro binario. Medido con `rustc` 1.94.1 y las tres versiones de la herramienta de arriba, con las
mismas huellas: el nodo `bc10653591e71134…` (10.623.960 B) y el testigo `1d074aaece334c65…`
(14.581.944 B), iguales desde las dos raíces. Son las fuentes de `b6994c3`: desde entonces cambian
las del nodo (el §813, el §815, el §816, el §817, el §819, el §820, el §821, el §832, el §833, el
§835, el §839, el §840, el §841, el §842, el §848, el §849, el §860 y el §861) y las del testigo
(las mismas salvo el §813, más el §850), y ni sobre el árbol del §817 ni sobre el de este sello se
han compilado así. En cuatro núcleos, con la carga media del minuto entre paréntesis: con la primera
versión, 474 s en frío —tras el `--check` en frío—, 5 s sin cambios y 188 s con un cambio en
`crates/zk-ssl` (de 2,2 a 4,4); con la segunda, 341, 4 y 164 s (de 3,9 a 7,3); con la que se sella,
366 s en frío —tras el `--check`— y 4 s sin cambios (de 2,4 a 3,7). El canon lo corre desde
`--largo`, en el bloque «3 ter binarios» de `tools/canon.sh`, y no en cada sello, con el criterio de
los niveles: una compuerta que encarece cada sello acaba saltándose (`doc/METODO.md` §«6. El canon y
los bancos», el §225), y un sello que toque la capa pagaría de 164 a 188 s, y uno en frío de 341 a
474, sobre los de 470 a 926 de un `--sello` (`tools/canon.sh`, al pie). Como en el kit, una línea de
comentario al final de `crates/zk-ssl/src/lib.rs` los lleva a `4963bb3a7cc4b69c…` y
`4d2acfa127d232b4…`: el `build-id`, `.symtab` y `.strtab` distintos, los nombres de los símbolos; el
código, igual.

**Lo que deja en disco.** Tras `--binarios`, `target/artefacto/tA` y `target/artefacto/raiz2`, con
su `target`, ocupan 624 y 687 MB, unos 1,3 GB, y la herramienta no los borra: cada árbol que pase un
`--largo` los guarda.
`rm -rf target/artefacto/tA target/artefacto/raiz2 target/artefacto/raiz2.lista` los quita, y la
pasada siguiente es en frío.

**De qué depende su huella**: de lo mismo que la del kit —el `rustc`, el `Cargo.lock`, lo que se
compila y con qué features, y el `cc`, el `as` y los objetos de arranque del host—, y el testigo
lleva más código nativo: el C y el ensamblador de ring, además del ensamblador de blake3, que llevan
los dos. No de la ruta del árbol, del usuario, de `CARGO_HOME`, de `RUSTUP_HOME` ni de si
`rust-src` está instalado: medido el 20-10-2026 con la segunda versión, con otro `HOME`, un
`CARGO_HOME` fuera de él y `rust-src` instalado en otro `RUSTUP_HOME`, las tres cosas a la vez,
salen las mismas dos huellas (563 s en frío). Entre máquinas, sin demostrar.

**Lo que NO prueba.** Que el operador corra ese binario. Una huella reproducible dice que un binario
publicado sale de su fuente; el nodo lo arranca el operador en su máquina, y nada de lo que firma
dice qué binario lo firmó: `hash_verificador_vigente` no está en la cabeza (`BACKLOG.md`, entradas
54 y 83). Es una precondición de esa vía, no su cierre, y `SECURITY.md` §«2. Qué NO protege» —«El
operador puede cambiar el verificador, y hoy eso es invisible»— sigue diciendo lo que dice. Tampoco
hay huellas publicadas del nodo ni del testigo: ninguna release los lleva, y publicarlas es del
autor.
