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
dice?» es «descárguelo y córralo usted», y sólo vale si la descarga existe y se puede reproducir.

## 0. Descargar, y comprobar la descarga antes de creerla

La release vigente es `arqueo-verify-v0.4.1`, publicada en
<https://github.com/atoranzo/Arqueo-open-conservation-proofs-for-closed-ledgers/releases/tag/arqueo-verify-v0.4.1>,
producida sobre el commit `ff1f06f8677522b5a01fce66a64e4d38c896c9fc`:
tarball `arqueo-verify-0.4.1-x86_64-unknown-linux-gnu.tar.gz`, sha256
`505de994f243bc7669045e7f546496423e8f4bfe03c7777d6829656f73827e14` (16.544.680 B, 366 ficheros);
binario `d81923cc942455d787392e044005f687fdc4cabd96b9aa58e2a88809ab4d1b7b`. Las dos huellas están
medidas desde fuera —descarga por `curl`, el `digest` del asset en la API, el kit descargado
corriendo sus once catálogos desde dentro del tarball— en el asiento de `AUDITORIA.md` que registra
esta release (§661). Recompone cabezas hasta la **v6**, la que el nodo firma hoy, y lee once
familias: las diez de la sección 11 de `spec/PAQUETE.md` y la del ancla cofirmada (§633, §634).

**La 0.4.0 sigue publicada y ya no es la vigente** (`arqueo-verify-v0.4.0`, commit `65d1e89`,
tarball `90184e41299e737a…`). Tiene un defecto que la 0.4.1 corrige: una prueba con nodos de más
dentro de un lote de Merkle verifica igual, con otros bytes (§659). El veredicto sobre el enunciado
no cambia —la prueba retocada prueba lo mismo que la original—, pero sus bytes dejan de identificar
la operación. Use la 0.4.1. Las dos leen las mismas once familias y dan los mismos veredictos a
todos los vectores publicados.

**La 0.3.0 sigue publicada y ya no es la vigente** (`arqueo-verify-v0.3.0`, commit `65cabfb`,
tarball `06648502e0171fea…`). Tiene cuatro defectos que la 0.4.0 corrige, descritos en la sección 3.9
de `SECURITY.md`: un campo hex con un carácter multibyte, o una prueba oculta con el meta vaciado, la
hacen salir con **101** (pánico) en vez de ROJO (§650, §651); una prueba con bytes de cola se lee
igual que sin ellos (§653); y acepta los cuatro negativos de la lectura canónica (RFC-0016, §640).
Falla con un código que no es el suyo, nunca con un VERDE falso para esos casos; aun así, use la
0.4.1.

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
<fichero.json>`; sale 0 y escribe `VERDE: …` si el fichero se sostiene, 1 y el **primer** fallo con
nombre (`ROJO: …`) si no, 2 si el uso es incorrecto.

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
compilado con `--remap-path-prefix`) y el mismo tarball, y `tools/canon.sh` comprueba esa propiedad
en cada sello. Las demostraciones con nodos vivos —levantar dos libros, publicar la misma etiqueta
en los dos, capturar el sobre— son los bancos `tools/banco_dos_libros.sh`, `tools/banco_consumo.sh`
y `tools/banco_apagado.sh`, y los del paso 5, `tools/banco_completitud.sh`,
`tools/banco_mentiroso_sin_resolver.sh` y `tools/banco_recibo_agregado.sh`; no viajan en el kit
porque levantan procesos, y los vectores de arriba son sus capturas.
