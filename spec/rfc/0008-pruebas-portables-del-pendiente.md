# RFC-0008 — Las dos pruebas portables del pendiente: el cobro, el pago en curso y la prenda

- **Estado:** PROPUESTO
- **Autores:** Che, con Claude (sesiones 144, 145, 147, 149, 150, 151, 152, 153, 154 y 155)
- **Fecha:** 2026-09-16
- **Versión del protocolo afectada:** `zkssl/0.3` — **no sube** (ver Compatibilidad). Los dos
  métodos nuevos son aditivos; la cabeza v5 no cambia de forma; la marca de prenda es una hoja
  más del árbol de consumos, cuya raíz la cabeza ya firma desde el RFC-0006.
- **Asiento(s) de AUDITORIA:** §121, §178, §211, §275 (el acuse y su raíz), §343–§345 (el
  compromiso v2 del RFC-0003), §387 y §388 (las raíces en reposo), §412–§441 (el consumo
  publicado), §451–§453 (la cabeza v5), §458–§460 (el rechazo por caminos), §463–§467
  (la prueba de edad), §473–§479 (la banda sobre la hoja comprometida); el §483, que lo
  adopta; el §484, que decide D-F y D-G y corrige la fila E1; el §485, el instrumento de E1;
  el §486, que decide D-H; el §489, que decide D-I y corrige D-B y la Seguridad; el §494,
  que decide D-J, D-K, D-L y D-M y corrige la fila E4; el §496, que decide D-N, D-O, D-P y
  D-Q y parte la fila E4 por lados; el §497 y el §497-B, la boca del cobrador; el §498, que
  decide D-R..D-W y trae el banco; el §499, que decide D-X..D-AC y trae el catálogo; el
  §500, que escribe aquí esas doce y pone en su celda lo sellado de E1 y del cobro de E4; y el
  §501, que decide D-AD..D-AJ y abre E2; el §502 y el §503 (el instrumento y el AIR del pago),
  el §504 (el productor), el §505 (la puerta del pagador), el §506 (el brazo del mando y la
  forma 2.9), el §507 (la boca `prueba-pago`), el §508 (el banco y la credencial del pagador) y
  el §509 (el catálogo); y el §510, que escribe aquí D-AK..D-AR, sella la celda de E2, cierra la
  de E4 por los dos lados y anota la cota.
- **Hito:** H5 de la propuesta enviada a NLnet Restack (140 h), en sus palabras: *«The two
  portable proofs of a pending item. Payee side and payer side, derived from the same head;
  pledge transition; format and vectors.»*

## Estado de las etapas

| etapa | qué entrega | ¿rompe el cable? | estado |
|---|---|---|---|
| E1 — el cobro pendiente, portable | el circuito del cobrador: bajo el `pendingRoot` de una cabeza v5 firmada existe `C2 = M(C1, X)` con `C1 = H(H(receptor, sal), importe)`, a nombre de `receptor` (D-G), `importe >= inferior` (banda, molde de `InsufficientBalance`), con el camino DENTRO del circuito; su meta `(emisor, nacido)` por camino bajo `pmetaRoot`, con los mismos bits. `zkssl_pendingPath`, aditivo, sirve los dos caminos de la foto del último latido a quien presenta un aviso que recompone la hoja (D-F). Sobre `tipo: "cobro_pendiente"` en `PAQUETE.md`, verificado por el mando sin nodo | NO | **sellada** — §484 (D-F y D-G, y la fila corregida), §485 y §485-B (el instrumento), §486 (D-H), §489 (D-I), §490 y §490-B (el AIR de dos carriles y un bit), §491 y §491-B (el productor, una función libre), §492 y §492-B, §493 y §493-B (la foto del latido en la capa y en el nodo, y `zkssl_pendingPath`), §494 (D-J..D-M) y §495 y §495-B (el enlace con la cabeza, el brazo del mando y `PAQUETE.md` 2.8). **E1 queda entera** |
| E2 — el pago en curso, portable | el espejo, para el pagador: `C2` abre a `(receptor, importe)` EXACTOS, `nacido` por camino, y `nacido + delta >= T` con `delta` y `refund_id` como testigo (no se revelan). Sobre `tipo: "pago_en_curso"`, verificado sin nodo. Junto al de E1, un tercero ajeno a los dos verifica un pago disputado sin el libro de nadie | NO | **sellada** — §501 (D-AD..D-AJ), §502 y §502-B (el instrumento), §503 y §503-B (el AIR de dos carriles con el sobre compuesto dentro, y su juez), §504 y §504-B (el productor en la capa, función libre), §505 y §505-B (la puerta del pagador en `zkssl_pendingPath`, D-AE), §506 y §506-B (el séptimo brazo del mando y la forma 2.9), §507 y §507-B (la boca `prueba-pago`, D-AK..D-AO), §508 (el banco y la credencial del pagador, D-AP y D-AQ) y §509 (el catálogo, D-AR). **E2 queda entera** |
| E3 — la prenda, como transición con prueba | el receptor marca el pendiente como prendado: una MARCA con dominio propio sobre `C2` en el árbol de consumos, publicada por un método aditivo, `zkssl_pledge`, que EXIGE el sobre de PRENDA: AIR propio (D-AV) que publica la MARCA con `C2` de testigo (D-AW), sin la banda y sin la meta, en 42 columnas y 512 filas (D-AX, D-AY); una segunda MARCA es `ConsumoRepetido`, que ya tiene sobre de rechazo con prueba (RFC-0007 E3, `PAQUETE.md` 2.6). La PRENDA es el par marca + sobre (D-AS), y no toca el cobro ni el reembolso: lo que obliga es contrato, y se declara | NO | propuesta |
| E4 — el catálogo y el banco, por lados | `spec/vectors/pendiente/`: dos positivos por lado, REUNIDOS de las capturas de un nodo real (molde: `edad/`), y un negativo por regla producible; `MANIFIESTO.txt`; la familia en `FAMILIAS`; el banco que lo reproduce en vivo; la sección 9 de `PAQUETE.md`. Va POR LADOS en una sola fila (D-O): el del cobro primero —sus dos formas (D-N), la boca del cli (D-P) y sus negativos (D-Q)— y el del pago con E2; esta celda nombra lo sellado de cada lado. El giro a ACEPTADO exige la regla 4 medida letra a letra, como el §481 | NO | **sellada por el lado del cobro** — §496 (D-N..D-Q), §497 y §497-B (la boca del cobrador en el cli, `prueba-cobro`, y `simulate --v2`), §498 (D-R..D-W y el banco, `tools/banco_pendiente.sh`) y §499 (D-X..D-AC y el catálogo, `spec/vectors/pendiente/`, sexta familia del artefacto y del canon). **Y el lado del PAGO**: §508 (el banco propio, `tools/banco_pago.sh`, con su siembra de cuatro ficheros, D-AP y D-AQ) y §509 (el catálogo `spec/vectors/pago/`, séptima familia del artefacto y del canon, D-AR, que revierte D-AC y la letra de D-AJ). **Los dos lados están; la etapa queda entera**, y lo que falta para ACEPTADO es la regla 4 medida letra a letra |

Las medidas de este documento se tomaron sobre `5ef3b1b` (`TERRENO-H5-144`); las de D-F y D-G,
sobre `393032e` (`TERRENO-E1-145`); y las de D-H, con el instrumento del §485, que corrió en una
copia fuera del árbol, y leyendo `0424439`; las de D-I, leyendo `be90eb7`
(`TERRENO-AIR-E1-147`); las de D-J a D-M, leyendo `7bb3942` (`TERRENO-COBRO-149`); las de
D-N a D-Q, leyendo `9512915` (`TERRENO-E4-150`); las de D-R a D-W, leyendo `8d064c6`
(`TERRENO-B-150`); las de D-X a D-AC, leyendo `f1e0401` (`TERRENO-C-150`); y las de D-AD a
D-AJ, leyendo `477dcab` (`TERRENO-E2-151`, con su falsador medido en una copia por el
PASTE-E2-PRE). Ninguna escribió un byte en el árbol (ver Referencias).

**Correcciones del §484** (la fila E1 y D-B, como las dejó el §483). La fila decía «`receptor` es
la identidad pública de quien prueba», y el molde que nombra no restringe quién prueba (D-G).
Decía «`importe >= X`» con `X` también como sobre de reversión: un nombre para dos cosas; la cota
pasa a llamarse `inferior`, como el `lower` de `banda.rs`, y `X` queda para el sobre. Daba
`zkssl_frozenPath` como molde del método, y ese molde sirve el estado de ahora (D-F). Y este
párrafo decía que todas las medidas se tomaron sobre `5ef3b1b`.

**Correcciones del §489** (D-B y la Seguridad, como las dejaron el §483 y el §484). D-B decía
«el sobre `X` puede ser público porque es un compromiso»: `X` es un compromiso sin aleatoriedad
y no esconde nada a quien adivine `refund_id` (D-I). La Seguridad decía, sin condición, que el
sobre del cobrador no dice cuándo caduca ni quién pagó; con `X` dentro, eso sólo valía si
`refund_id` no se adivinaba. Con D-I vale sin condición, porque `X` no viaja.

**Corrección del §494** (la fila E4, como la dejó el §483). Decía que E4 entrega
`PAQUETE.md` 2.8, y la fila E1 ya ponía el sobre del cobro en ese documento: la sección del
sobre nace en E1 con su juez (D-L), y a E4 le queda la sección 9, la de los vectores.

**Correcciones del §500** (las celdas de estado de E1 y E4, como las dejaron el §483 y el §496, y
la cuenta del Diseño). La celda de E1 decía «propuesta» con la fila entera desde el §495 —y D-O
lo decía ya—; pasa a nombrar sus sellos. La de E4 decía «propuesta» cuando su propio texto pedía
que nombrara lo sellado de cada lado; nombra el del cobro y deja el del pago para E2. El estado
del documento no se mueve: PROPUESTO es del proceso (`PROCESO.md`, regla 4) y el de cada etapa
vive en su celda, como en el RFC-0005; el giro a ACEPTADO exige lo que la fila E4 dice. Y el
Diseño decía «diecisiete decisiones»: las doce de la sesión 150 —D-R..D-W del banco y D-X..D-AC
del catálogo— vivían sólo en los asientos 498 y 499, y entran aquí con su reversión.

**Corrección del §521** (D-B, D-I, la corrección del §489 y la Seguridad). Decían que `X` no viaja y
que el sobre del cobrador no la lleva: no la lleva como CAMPO, pero la prueba abre sus filas en
claro y la publica, con la sal y el importe exacto; la del pago publica la sal, el `delta` y el
`refund_id`; y la de la prenda, la clave de gasto del prendador, 42 veces. Medido en las sesiones
162 y 163: winterfell 0.13 no oculta el testigo. Las decisiones se tomaron con el dato de su fecha y
no cambian aquí; lo que cambia es lo que pueden prometer, y eso lo decide un RFC sobre qué se
promete.

### La frontera con H5b, y qué es de cada uno

Este RFC habla de un pendiente que ESTÁ en el árbol: existe, tiene importe, tiene edad, y se
puede prendar una vez. H5b habla de lo que entró por el cable y de si acabó aplicado o rechazado.
Un pago que el operador no aplicó no tiene hoja, no tiene `nacido` y no tiene prueba aquí: eso
es la completitud, y sigue siendo H5b. La prenda tampoco es completitud: lo que es único es la
PRUEBA —una sola prenda PROBADA por hoja (D-AU)—, y la MARCA es la etiqueta que la publica, con
el mismo uso único que el RFC-0006 da a cualquier etiqueta.

## Motivación

El hito H5 promete que el pendiente deje de ser la promesa del pagador y pase a ser un activo
del cobrador, con prueba y con uso único. Lo medido sobre `5ef3b1b`, y por qué cada pieza
manda la forma de este RFC:

- **El pendiente ya está comprometido de forma abrible.** `C1 = H(H(receptor, sal), importe)`
  (`crates/zk-ssl/src/pending.rs:70`), el sobre de reversión `X = M(refund_id, delta)`
  (`:88`) y la hoja `C2 = M(C1, X)` (`:108`), con la posición SALADA (B13/B14,
  `crates/zk-ssl/src/migration.rs`). Los circuitos que abren `C1` y `X` existen:
  `crates/stark-experiment/src/circuit_claim_v2.rs` y `circuit_refund_v2.rs`.
- **El aviso al receptor es opaco a propósito.** `PendingNotice`
  (`crates/zk-ssl/src/two_phase.rs:62`) lleva `position`, `salt`, `amount` y `x: Option<Digest>`; el
  receptor reconstruye `C2` sin aprender `f` ni `delta` (D-1 del RFC-0003), y `refund_id` viaja
  comprometido y sólo se abre al reembolsar (D-2). El nodo no guarda la apertura: del pendiente
  persisten `pend:`, `pamt:` y `pmeta:` (`crates/zk-ssl/src/persistence.rs`, y la corrección §247
  del §464 en el RFC-0007).
- **La T del cobrador no se puede producir con el aviso de hoy.** `expiry = nacido + delta`; el
  receptor tiene `nacido` por camino bajo `pmetaRoot` (`meta_pendiente_hoja(emisor, nacido)`,
  `two_phase.rs:238-241`) pero `delta` vive dentro de `X`, opaco para él. Sólo el pagador lo
  sabe. Es el hallazgo que reparte las dos mitades: la T es del pagador.
- **La cabeza v5 ya firma lo que las dos pruebas necesitan**: `pendingRoot`, `pmetaRoot`,
  `next_pending` y `params_digest` con el `ttl` del reembolso (RFC-0007 E1, §451–§453). Ninguna
  familia nueva bajo la firma: el formato de la firma se queda en 5.
- **La regla de reversión ya existe y está comprometida**: `RefundTooEarly` si
  `ahora - nacido < ttl` (v1) o `< delta` (v2) (`two_phase.rs:437-452, 660-674`). «No
  reversible antes de T» no es una regla nueva: es esa, leída desde el otro lado.
- **«Al menos `inferior`» es circuito, y su molde existe.** `crates/zk-ssl-air/src/banda.rs`
  abre una hoja bajo una raíz con el camino DENTRO y prueba una banda sobre un campo; el
  operador la produce sin la clave. Aquí la hoja es `C2`, la apertura son dos composiciones, y
  la banda es sobre `importe`. Revelar el importe en el sobre del cobrador contradiría la promesa.
- **El conjunto de uso único que el PDF llamaba «la primera medida» ya existe.** No hay conjunto
  de gastados —el cobro retira la hoja—, pero el RFC-0006 dejó un árbol de etiquetas públicas
  con raíz firmada (`root:cons`), donde repetir es `ConsumoRepetido`
  (`crates/zk-ssl/src/consumo.rs:129-139`), y esa causa ya está en el catálogo de rechazos con
  prueba (`spec/PAQUETE.md`, 2.6). Publicar hoy no exige prueba (`spec/RPC.md:936`): quien
  publica primero bloquea, y eso es denegación, no doble uso (D-4 del 0006). Lo que cualquiera
  puede poner sobre el pendiente de otro es la MARCA, y la marca sola no prueba nada (D-AS): no
  es denegación, es una etiqueta más. Lo que exige prueba es la PRENDA, y por eso entra por un
  método propio que pide el sobre del cobrador con su titularidad (D-AV).
- **Los sobres portables tienen forma y juez**: `PAQUETE.md` 2.4 (consumo), 2.6 (rechazo) y
  2.7 (edad); el mando los verifica sin nodo y el artefacto los lleva dentro con su catálogo
  (`tools/artefacto.sh`, `FAMILIAS`). Dos tipos nuevos siguen ese molde sin tocar los viejos.

## Diseño

Las cincuenta y cuatro decisiones las tomó el asistente por delegación del autor (D-A..D-E en la
sesión 144; D-F, D-G y D-H en la 145; D-I en la 147; D-J..D-M en la 149; D-N..D-AC en la 150, y
de ellas D-R..D-AC las escribió aquí el §500; D-AD..D-AJ en la 151; D-AK..D-AR en la 155, y las
escribió aquí el §510; D-AS..D-AU en la 158, que las escribió el §513; D-AV..D-AY en la 159; y D-AZ..D-BB en la 161, que las escribió el §519-B),
con la constitución de decisión (pureza, claridad, coherencia, imagen fiel, en ese orden). Todas
llevan su condición de reversión, escrita aquí.

### D-A — La T es del pagador; el cobrador dice «a mi nombre, al menos `inferior`, nacido en b»

El aviso no lleva `delta`, y no debe llevarlo abierto: D-1 y D-2 del RFC-0003 existen para que
el receptor no aprenda ni las elecciones de retorno ni la identidad de la clave de retorno. Tres
caminos se midieron: (a) que la mitad del cobrador diga lo que su dueño sabe —existe, importe
`>= inferior`, `nacido = b`— y la T la lleve la mitad del pagador, que sí lo sabe todo; (b) que el
aviso gane `delta` fuera del cable y el receptor abra `X` en circuito, lo que exige `refund_id`
como testigo y rompe D-2; (c) un compromiso v3 que separe `delta` de `refund_id`, que es rotura
de formato y nombre nuevo. Gana (a): pureza (no toca el 0003), claridad (cada mitad dice lo que
su dueño sabe y nada más), imagen fiel (la spec dirá que la caducidad la prueba el pagador). La
promesa del hito se cumple con las dos mitades juntas, que es exactamente como el texto del PDF
la formula. **Reversible** si un caso de uso medido exige la T del lado del cobrador: entonces
se abre (c), nunca (b).

### D-B — «Al menos `inferior`» es una banda en circuito, con el camino dentro

El molde es la banda de `InsufficientBalance` (RFC-0007 E5): la hoja abierta dentro del AIR,
el camino como testigo, dos cotas públicas. Aquí las cotas son `inferior` y `superior` (el tope
de importe), la apertura es `C1` y luego `C2 = M(C1, X)`, y el sobre `X` es testigo (D-I): no
esconde nada a quien adivine `refund_id`. La posición no sale del circuito: es lo que las
posiciones saladas prometen.
El coste se mide antes de escribir el AIR con el molde del instrumento de la E4a del RFC-0007
(`crates/zk-ssl/src/instrumento_edad.rs`), y si no cabe bajo un latido se declara con cifra.
**Reversible** sólo hacia una banda más estrecha; nunca hacia revelar el importe.

Lo que ya está medido sobre `393032e` (`TERRENO-E1-145`). Los dos árboles tienen 32 niveles
(`TREE_DEPTH`) y la hoja de meta vive en la MISMA posición que el compromiso: los dos caminos
comparten los bits de dirección y NO los hermanos, y su primer testigo negativo es una meta de
otra posición. La cadena del compromiso son 35 ciclos de 8 filas, como la de la banda; la meta
suma 33 (una permutación con dominio y la subida). En un carril la traza pasa de 512 a 1024
filas; en dos carriles con el bit compartido se queda en 512. Las cotas medidas con las opciones
de la casa (la banda, `circuit_audit`, el cobro v1) lo dejan en milisegundos: lo que decide el
instrumento es la geometría, no si cabe. La geometría la decide D-H.

### D-C — La prenda vive en el árbol de consumos, con dominio propio y con prueba

Dos sitios se midieron: (a) una etiqueta `H(dominio_de_prenda, C2)` en el árbol de consumos,
publicada por un método nuevo que exige la prueba de apertura del cobro antes de escribirla; (b)
un campo `prendado` en la hoja de meta, que cambia `meta_pendiente_hoja`, con ella `pmetaRoot`,
y con ella la versión de FORMATO de la firma y los vectores de edad que abren esa hoja. Gana (a):
una primitiva por propiedad —uso único es consumo—, el rechazo con prueba del segundo intento
existe ya (`ConsumoRepetido`), la cabeza no cambia y ningún vector vivo caduca. El dominio se
registra en `zk-ssl-hash` como los demás y `check_dominios` lo censa. Lo que cambia respecto del
0006 no es el árbol sino quién puede escribir ESA clase de etiqueta: la publicación libre del
0006 sigue para las etiquetas de consumo; la de prenda exige prueba. **Reversible** hacia (b) si
la medida de E3 mostrara que el árbol de consumos no puede distinguir clases sin romper su regla.
⚠️ **La medida se hizo y el árbol NO las distingue**: (a) se sostiene, pero el nombre se parte en
MARCA y PRENDA. Ver D-AS, que enmienda esta decisión sin sustituirla, y D-AV, que decide cuál es
esa «prueba de apertura del cobro» que este párrafo nombra: no el sobre de E1, sino uno con el
ciclo de la clave.

### D-D — Prenda el receptor, y la prenda sólo obliga a la prenda

La autorización es conocimiento de preimagen o prueba: el pagador también tiene la apertura de
`C1`, así que «saber la apertura» no distingue. Lo que sólo el receptor tiene es lo que
`circuit_claim_v2` ya exige para cobrar: su identidad probada. La prenda es esa autorización sin
el crédito. Y la prenda NO toca el cobro ni el reembolso: un pendiente prendado se cobra igual y
se reembolsa igual tras T. Lo que la prenda garantiza es que no hay segunda prenda PROBADA
(D-AU), con prueba del rechazo; lo que obliga al cobrador con su financiero es contrato, y el
sistema produce el par condenatorio, no adjudica (§121.6). Una regla que bloqueara el reembolso
sería una regla nueva de la capa sobre los derechos del pagador: menos excepciones.
**Reversible** hacia «la prenda bloquea el reembolso hasta T» sólo con su testigo negativo
escrito antes y con la regla comprometida en la cabeza, porque una regla que no está
comprometida no sostiene una prueba.

### D-E — Dos tipos de sobre, uno por lado

`cobro_pendiente` y `pago_en_curso`, como `consumo` y `conflicto` son dos: el mando dispatcha por
`tipo`, no por un campo interior, y cada tipo lleva su lista de rechazos y sus vectores. Ambos
llevan la cabeza v5 firmada entera, como el sobre de edad. **Reversible** hacia un tipo con `lado`
sólo si el catálogo mostrara que las dos listas de rechazo son idénticas, que hoy no lo son (la
del pagador tiene la T y el `refund_id`; la del cobrador tiene la banda).

### D-F — El camino y la cabeza, del mismo estado: la foto del latido

Medido sobre `393032e`: `zkssl_signedEpochHead` sirve la cabeza del ÚLTIMO LATIDO
(`crates/zk-ssl-node/src/main.rs:1505`), y el camino que la capa sabe dar es el del estado de
AHORA (`claim_materials`, `crates/zk-ssl/src/client.rs:163`). El árbol de pendientes lo escriben
cuatro funciones de producción (`apply_deissue`, `apply_refund`, `commit_send` y `commit_claim`,
en `two_phase.rs`), así que cualquier pago entre dos latidos mueve `pendingRoot`, y un camino de
ahora deja de subir a una cabeza firmada. El molde que la fila E1 nombraba, `zkssl_frozenPath`,
vale porque su árbol sólo cambia con otra congelación (`spec/RPC.md`, «El camino de
congelados»); copiado aquí, no vale.

Cuatro caminos se midieron: (i) el latido guarda, en la misma sección crítica en la que compone
la cabeza (`crates/zk-ssl-node/src/latido.rs`, «TODO bajo el MISMO candado»), una foto de los
dos árboles, y el método sirve el camino de esa foto con el `seq` de la cabeza firmada; (ii) el
camino de ahora con su `s`, y el cliente espera una cabeza de ese `seq`; (iii) reconstruir el
árbol en el `seq` de la cabeza desde el registro; (iv) firmar una cabeza a petición. Gana (i):
coherencia (la misma noción de cabeza para el camino y para la firma) y fail-closed (sin latido,
sin `--clave` o con un pendiente nacido tras el último latido, el método lo dice, en la forma
del §241). La (ii) falla cerrada, pero en un nodo con tráfico puede no converger nunca. La (iii)
no es posible: una entrada del registro no lleva la posición ni la hoja
(`crates/zk-ssl/src/log.rs`, `LogEntry`). La (iv) quema índices XMSS a demanda. La foto es caché
en memoria: tras un reinicio no hay camino hasta el primer latido. **Precio declarado**: clonar
dos árboles dentro del candado, en tiempo y en memoria; lo mide el instrumento de E1 antes que
el método, con el molde de la medida M.1 del §252 (escrituras en serie con y sin latido, mirando
el máximo).
**Reversible** hacia (ii) si ese precio no cabe.

La foto obliga a una regla del método. Servir `(emisor, nacido)` de cualquier posición a
cualquier cuenta con credencial publicaría quién pagó y cuándo. El nodo recompone la hoja con lo
que el aviso trae y el `public_id` de la credencial, `C2 = M(H(H(public_id, salt), amount), x)`,
y la compara con la hoja de esa posición en la foto; si no casa, rehúsa sin decir qué hay. Es la
lección del §261 (un aviso que no autentica) aplicada antes de escribir el método. E1 es del
compromiso v2: un aviso sin `x` no tiene `C2`, y el método lo rehúsa.

### D-G — E1 es un enunciado de ESTADO, sin titularidad

Medido sobre `393032e`: el molde que D-B nombra, `crates/zk-ssl-air/src/banda.rs`, no lleva el
ciclo de la clave; ata la identidad con cuatro aserciones contra la entrada pública que declara
quien verifica (su D-2b). Con ese molde, la fila E1 afirmaba algo que el circuito no restringe:
quien conozca la apertura —el pagador la conoce— produce la misma prueba. Dos caminos: (a) el
enunciado es de estado, «bajo esta cabeza existe un pendiente a nombre de `receptor`», y el RFC
lo dice así; (b) el ciclo de la clave de `circuit_claim_v2` (`CYC_PK`: un ciclo y cuatro
columnas más), y sólo el receptor la produce. Gana (a): pureza (una primitiva por propiedad; la
autorización es de E3, donde la prenda la exige por D-D), menos columnas, e imagen fiel (lo que
el circuito no restringe no se afirma). Y (b) no compra lo que parece: una prueba se reenvía
igual con clave o sin ella, y atarla a quien la PRESENTA pide un reto dentro del enunciado, que
ninguno de los dos lleva. **Reversible** hacia (b) sólo con ese reto y con un caso de uso medido
que lo pida.

### D-H — La geometría del cobrador: dos carriles y un solo bit

Medido con el instrumento de E1 (§485, `crates/zk-ssl/src/instrumento_cobro.rs`): la cadena del
compromiso son 35 ciclos de 8 filas y la de la meta, 33; en un carril suman 544 filas y piden una
traza de 1024, y en dos carriles caben en 512. La subida proxy de la casa (13 columnas, con
`crate::proof_options()`) probó 512 filas en 0,16 s y 43.700 B, y 1024 filas en 0,15 s y
49.631 B: en esta talla, doblar las filas sube los bytes un 13,6 % y no mueve el tiempo (una sola
corrida, sin aparear). Un segundo carril añade un estado de hash, 12 columnas; la diferencia
medida entre `circuit_audit` (31 columnas, 51.449 B) y la banda (27 columnas, 49.051 B) da unos
600 B por columna, así que el ancho cuesta del mismo orden (razonado). El coste no decide.

Dos caminos: (a) dos carriles que se colocan con UNA columna de bit, el molde del cobro v2
(`crates/stark-experiment/src/circuit_claim_v2.rs`, sus dos carriles con el mismo `bit`) sin su
igualdad de hermanos, porque aquí son dos árboles distintos; (b) un carril, con la posición
acumulada en la primera subida, recompuesta en la segunda y una aserción que las iguale. Gana
(a): pureza (que el compromiso y su meta están en la MISMA posición lo da la estructura, no una
igualdad que haya que acordarse de escribir), coherencia (el molde existe) y claridad (una
posición, una columna). La casa ya sube dos árboles por la misma posición en un carril -la
subida de congelados, tras la de cuentas- y la columna del bit de la segunda no está atada a la de
la primera: leído en los cinco circuitos que lo hacen y MEDIDO en el §487 (el camino de otra
posición verifica en los cinco; la capa lo cierra al aplicar y el arreglo B lo devolverá al AIR).
La lección del §487 refuerza (a): la posición la ata el BIT COMPARTIDO en una sola columna, que
es justo lo que a `COL_FBIT` le falta.

La forma: el carril A compone `C1` en los ciclos 0 y 1 y `C2 = M(C1, X)` en el 2, y sube del 3
al 34; el carril B deja libres los ciclos 0 y 1, compone la hoja de meta en el 2 y sube del 3 al
34 con el mismo bit. El primer testigo negativo del AIR es el del instrumento: una meta de otra
posición no verifica. **Reversible** hacia (b) si el AIR de E1, medido en su sello con las
opciones de la casa, pasa de 65.313 B, que es el cobro v1 (1024 filas por 55 columnas) y la cota
superior con la que se razona aquí.

> **Nota del §512 (S247: se cita y no se borra).** El arreglo B (5.A-272) movió esa cota:
> el cobro v1 pasa a **66.692 B** y su circuito, a **57 columnas**. La decisión se tomó con
> el dato de su fecha y **no cambia**; lo que cambia es el número contra el que se mide.

### D-I — El sobre `X` es testigo del cobrador, no entrada pública

Medido sobre `be90eb7` (`TERRENO-AIR-E1-147`): `X = M(refund_id, delta)`
(`crates/zk-ssl/src/pending.rs:88`) es un compromiso SIN aleatoriedad, y `refund_id` lo elige el
emisor (`crates/zk-ssl/src/client.rs:199`). Los tres productores de envíos v2 del árbol usan el
`public_id` de la cuenta que paga: el escenario de conformidad, con `delta = 96`
(`crates/zk-ssl-cli/src/conformance.rs:78-80`), y dos tests de la capa
(`crates/zk-ssl/src/two_phase.rs:3429` y `:3473`). Con `X` en el sobre, quien tenga una lista de
identidades candidatas prueba `M(pid, d)` para los `d` plausibles y aprende quién pagó y cuándo
caduca, las dos cosas que la Seguridad dice que el sobre calla; y dos sobres del mismo emisor con
el mismo `delta` llevan la misma `X`, enlazables sin enumerar nada. Es determinista y está
razonado; su falsador (dos envíos con la misma pareja dan la misma `x`) va con el AIR.

Dos caminos: (a) `X` es testigo: el AIR la lee de cuatro columnas en el enlace del ciclo 2, el
enunciado dice «existe un sobre `X` tal que `M(C1, X)` está bajo `pendingRoot`», y el sobre del
cobrador no la lleva; (b) `X` pública, y el RFC declara que su secreto depende de la entropía de
`refund_id`. Gana (a): la privacidad es propiedad del protocolo, no una política del emisor; lo
que no se esconde no se afirma (imagen fiel); y cuesta cuatro columnas, 44 en vez de 40
(razonado), sin mover la cota de D-H. E1 sigue siendo del compromiso v2 (D-F): una hoja v1 no es
`M(C1, X)` de ningún `C1` que se pueda abrir.

Lo que (a) deja a E2: sin `X` en el sobre, nada público enlaza la mitad del cobrador con la del
pagador; el enlace, si hace falta, se decide allí (la etiqueta de la prenda, `H(dominio, C2)`,
lleva la sal y es la candidata). Y el mismo límite vale frente al RECEPTOR, que recibe `X` en el
aviso: la opacidad de D-1 del RFC-0003 depende de que `refund_id` no se adivine. Eso es del 0003
y esta decisión no lo toca. **Reversible** hacia (b) sólo si el sobre `X` gana aleatoriedad (una
sal del emisor, rotura de formato del compromiso) y E2 mide que el enlace la necesita.

### D-J — El sobre del cobro: el molde de la edad, y lo firmado sólo en la cabeza

Medido sobre `7bb3942` (`TERRENO-COBRO-149`): el productor del §491 entrega, además de la
prueba y el enunciado, el `seq` y las dos raíces que la cabeza ya firma
(`crates/zk-ssl/src/prueba_cobro.rs`, `SobreCobro`), y fija la cota superior en el techo del
campo (`MAX_VALOR`), que el AIR declara pública. Dos caminos: (a) el molde del sobre de edad
(`spec/PAQUETE.md` 2.7), `{v: 1, tipo: "cobro_pendiente", cabeza, enunciado: {receptor, nacido,
inferior}, prueba}`, con la cabeza VERBATIM, `seq`, `pendingRoot` y `pmetaRoot` sólo de ella, y
sin `superior`, que fija el juez; (b) el sobre repite lo que el productor entrega. Gana (a):
coherencia (es el hermano que D-E nombra), claridad (un dato, una fuente: lo que el sobre no
repite no puede discrepar de lo firmado) y pureza (un enunciado, una forma; con `superior` en el
sobre, el mismo cobro tendría tantas formas como techos). Nace como `PAQUETE.md` 2.8, con el
brazo del mando, en E1 (D-L). **Reversible** hacia (b) sólo si un consumidor medido necesita el
enunciado sin abrir la cabeza; y hacia una `superior` en el sobre, sólo por la vía que D-B deja
abierta (una banda más estrecha) y con su testigo.

### D-K — El juez que enlaza vive en el crate del kit, y el nacido va antes que la cabeza

Medido sobre `7bb3942`: el juez del AIR, `zk_ssl_air::cobro_pendiente::verificar`, toma las dos
raíces como entrada pública y nada las ata a una cabeza; la prueba de edad tiene esa regla en el
mismo crate (`zk_ssl_air::verificar_contra_cabeza`), que el kit compila sin el probador; y el
productor del §491 verifica lo que produce con el juez sin enlace. Razonado del código, y sin
medir todavía: la cabeza firma `seq = log.len()` (`crates/zk-ssl/src/lib.rs`) y un pendiente
nace con `nacido = log.len()` antes de asentar su entrada (`crates/zk-ssl/src/two_phase.rs`),
así que bajo una cabeza honesta toda meta viva cumple `nacido < seq`; el AIR no lo restringe.
Dos caminos: (a) la regla vive en `cobro_pendiente` —un solo productor, en el crate que el
tercero compila—, compone el enunciado con las dos raíces de la cabeza y `superior = MAX_VALOR`,
y exige `nacido < seq` en nativo ANTES de verificar la prueba; el productor de la capa pasa a
verificar con esa misma regla (el molde del §466); (b) lo mismo, sin la regla de `nacido`. Gana
(a): fail-closed (una meta nacida después de la cabeza que la firma es una cabeza que miente, y
se para con su nombre), claridad (la Seguridad dice «nacido en `b`», y con la regla `b` es
anterior a la cabeza que lo afirma) y coherencia (el enlace de la edad). La regla se mide antes
de escribirse, en el PRE del corte, sobre un pendiente real con su cabeza. **Reversible** hacia
(b) si esa medida la desmiente —un pendiente honesto con `nacido >= seq`—: entonces la regla era
falsa y no se escribe.

### D-L — El perímetro de E1: el sobre y su juez; el catálogo, en E4

Medido sobre `7bb3942`: la fila E1 cierra con el sobre «verificado por el mando sin nodo», y la
fila E4 lleva el catálogo y el banco; el plan de trabajo, fuera del árbol, metía en E1 también
los vectores. Dos caminos: (a) E1 entrega el sobre (`PAQUETE.md` 2.8), el juez de D-K, el brazo
del mando con los negativos que caen antes de la firma y el positivo ENLAZADO, con una prueba
real, en los tests; el catálogo, la boca que escribe el sobre (D-M) y el banco son de E4; (b) E1
lleva además sus vectores. Gana (a): imagen fiel (manda el texto adoptado en el §483, y se
corrige el plan) y coherencia (el molde del RFC-0007: el §465 dio el sobre y su juez; el §466 y
el §467, la boca y el catálogo). Por lo mismo se corrige la fila E4: la sección 2.8 nace en E1,
y a E4 le queda la sección 9. **Reversible** hacia (b) si, al cerrar E1, E4 se parte por lados y
la mitad del cobro se adelanta a E2.

### D-M — La boca del cobrador es el cli, con el aviso v2 en un fichero suyo

Medido sobre `7bb3942`, aunque se ejecuta en E4. No hay forma serializada del aviso v2:
`PendingNoticeDto` no lleva `x`, y su conversión lo pone a `None`
(`crates/zk-ssl-wire/src/lib.rs`). Ninguna herramienta deja un pendiente v2 en un libro
persistido: `simulate` envía por la vía v1, y `run_send_v2` sólo lo llama la conformidad, en
memoria (`crates/zk-ssl-cli/src/`). Y el banco de edad no se copia: produce con el nodo PARADO,
y `zkssl_pendingPath` pide el nodo VIVO, con latido y `--clave`. Tres caminos: (a) el cli:
`simulate` aprende la vía v2 y escribe el aviso en un fichero propio, y un subcomando del
cobrador lee ese aviso, la respuesta de `zkssl_pendingPath` y la cabeza, y escribe el sobre;
(b) un modo del nodo, como `--prueba-edad`; (c) el SDK. Gana (a): coherencia de actores (el
testigo del cobro es del cobrador, no del operador, y su productor no lee libro) y claridad (el
aviso v2 tiene formato de cliente mientras el cable no lo transporte, y `PendingNoticeDto` no se
recicla). La (b) cambia de actor y pide parar el nodo justo tras un latido sin tráfico; la (c)
paga por la vía v1 del cable, y su pendiente no es de E1 (D-F). **Reversible** hacia un DTO v2
del cable cuando el cobro v2 viaje por él, que es la E4 del RFC-0003 y se decide allí.

### D-N — Las dos formas del positivo del cobro: la existencia y la banda más estrecha

Medido sobre `9512915` (`TERRENO-E4-150`): la fila E4 pide dos positivos por lado, reunidos de
las capturas de un nodo real con el molde de `edad/`, cuyos dos positivos son las dos formas que
su enunciado admite (todos, `t = 0`, y la caja vacía). El enunciado del cobro es `{receptor,
nacido, inferior}`; el productor toma `inferior` como argumento y rehúsa si supera el importe del
aviso (`crates/zk-ssl/src/prueba_cobro.rs`), y el juez fija la cota superior en `MAX_VALOR`
(D-J). Dos caminos: (a) dos formas del MISMO enunciado sobre UN pendiente: `inferior = 0` —el
pendiente existe a mi nombre, sin decir cuánto— e `inferior = importe` —la banda más estrecha que
se sostiene, «al menos lo que hay»—; (b) dos pendientes de dos corridas, con la misma forma. Gana
(a): pureza (un enunciado, dos formas, como en la edad), claridad (cada positivo afirma una cosa
distinta, y entre los dos enseñan el rango que el enunciado admite) e imagen fiel (la fila pide
dos positivos, no dos copias). **Reversible** hacia una tercera forma —una banda estricta entre 0
y el importe— sólo si un consumidor medido la necesita, y entonces entra con su vector sin
retirar estos dos.

### D-O — E4 se parte por lados, en una sola fila

Medido sobre `9512915`: D-L dejó escrita la reversión «si, al cerrar E1, E4 se parte por lados y
la mitad del cobro se adelanta a E2», y E1 está entera (§485-§495); el 5.A-272 pone el arreglo B
delante de E2, así que E2 arrastra una rotura de formato que la mitad del cobro no arrastra; y la
tabla del RFC-0007 nunca partió una etapa en dos filas: E3 y E4 nombran sus cortes (E3a, E3b,
E4a, E4b-1..3) en la celda de estado. Dos caminos: (a) la fila E4 sigue siendo una, su texto dice
que va por lados y su celda de estado nombra lo sellado de cada lado; (b) nacen dos filas, E4a
(cobro) y E4b (pago). Gana (a): coherencia (el molde del 0007), pureza (menos filas, menos
excepciones) e imagen fiel (la celda dice lo que hay: hasta que los dos lados estén, la etapa no
está). El lado del pago se hace con E2, porque su banco y su boca necesitan la prueba que E2
trae. **Reversible** hacia (b) si el lado del pago exige entregas que no quepan en una celda.

### D-P — La credencial del cobrador es suya: la boca la recibe, no la deriva

Medido sobre `9512915`: `zkssl_pendingPath` exige la credencial del receptor (`viewKey`, §261) y
la capa la cruza con el `view_id` guardado; la única boca del árbol que entrega una clave de
vista es `dev_openSeeded`, que abre una cuenta NUEVA; `derive_view_key_wide` vive en
`stark_experiment::native`, que el cli no declara aunque ya lo compila por vía de `zk-ssl`; y la
posición de una cuenta es `public_id[0] % cap` con sondeo lineal, derivable pero no pura. Tres
caminos: (a) la boca recibe `--index` y `--view-key` por argumento, como los tendría un cobrador
real, y el sandbox —que en la demostración ES la máquina del cobrador— escribe esa credencial en
un fichero PROPIO, aparte del aviso, derivada de su clave determinista, declarando
`stark-experiment` en el cli (cero nodos nuevos en el grafo); (b) la boca deriva índice y
credencial de `--key-seed` y `--to`, y sólo sirve en el sandbox; (c) un método del cable que
sirva la clave de vista. Gana (a): coherencia de actores (el aviso es del pagador y la credencial
del cobrador: dos ficheros, dos dueños, y ningún nombre reciclado), claridad (una boca que un
cobrador real puede usar) y la ley de diseño (la (c) haría que el nodo entregue lo que sólo
autoriza a leer, §261). **Reversible** hacia (b) si el fichero de la credencial resulta ser una
pieza que nadie custodia de verdad y la traza del sandbox ya lo dice todo.

### D-Q — Un negativo por regla producible con una mutación de una captura real

Medido sobre `9512915`: el brazo del mando cae antes de la firma con textos de la casa (`falta
enunciado`, `falta prueba o no es cadena 0x`, `falta cabeza`, `falta signature`, `exige una
cabeza v5`); el juez que enlaza cae con texto de la casa en `nacido >= seq` y, si la prueba no
verifica, con el prefijo `cobro:` delante de lo que pone winterfell (el molde de
`neg-cota-movida` en `edad/`). Dos caminos: (a) un vector por regla: los cinco de antes de la
firma, `nacido = seq` pinado entero, y UNO por la regla del juez —receptor movido, `inferior` por
encima del importe y prueba corrupta caen por la misma regla, así que entra uno—: siete, y
ninguna escena; (b) un vector por pieza mutada. Gana (a): coherencia (el catálogo pina la regla,
no la pieza, como en `rechazo/` y en `edad/`) y pureza (menos vectores que no dicen nada nuevo).
Se confirma con las capturas del banco y no antes: una mutación que caiga por otra regla que la
suya no entra. **Reversible** hacia más vectores sólo si una captura enseña una regla que esta
lista no nombra.

### D-R — Un solo banco para los dos lados de E4, con el nombre que la cola ya usaba

Medido sobre `8d064c6` (`TERRENO-B-150`): la fila E4 va por lados en una sola fila (D-O) y el
catálogo será uno (D-AC); el banco de edad produce con el nodo PARADO, porque la capa abre el
libro y prueba sola, y el cobro no puede copiar ese orden: la foto vive en el latido, en memoria,
y `zkssl_pendingPath` sólo la sirve con el nodo VIVO (D-F), mientras `sled` tiene el libro en
exclusiva. Dos caminos: (a) un banco, `tools/banco_pendiente.sh`, que siembra con el nodo parado
y cobra con el nodo vivo, corre hoy el lado del cobro y ganará el del pago con E2; (b) un banco
por lado. Gana (a): coherencia (una fila, un catálogo, un banco) y claridad (el nombre que la
cola ya usaba, y que el guion nombre al pendiente y no a un lado). **Reversible** hacia (b) si
el lado del pago exige un orden que no quepa en el mismo guion.

### D-S — UNA siembra: los dos positivos son dos cotas sobre el mismo pendiente

Medido sobre `8d064c6`: los dos positivos de D-N son dos formas del MISMO enunciado, y una sola
corrida evita la trampa de las posiciones lógicas del árbol disperso. Dos caminos: (a) una
siembra y dos cobros, `inferior = 0` e `inferior = importe`, sobre la misma hoja y bajo la misma
cabeza; (b) una siembra por positivo. Gana (a): pureza (D-N: un pendiente, dos formas), imagen
fiel (los dos sobres afirman cosas distintas sobre el mismo hecho) y menos escenas.
**Reversible** hacia (b) sólo si D-N se revierte hacia dos pendientes.

### D-T — La ventana del latido se paga con reintento y con latido corto

Medido sobre `8d064c6`: la boca exige `s == cabeza.seq` y no reintenta (§497); la foto vive en
el latido y el nodo late a su ritmo, así que entre pedir la cabeza y pedir la foto puede cerrarse
una época. Tres caminos: (a) el banco reintenta —cinco veces, SÓLO ante el texto de esa carrera,
y diciendo cuántas— con un latido corto; (b) un latido largo, para que la ventana no se cierre;
(c) que la boca reintente. Gana (a): la boca sigue sin reintentar, luego una carrera se distingue
de un fallo real por su texto y por su cuenta, y el banco no espera minutos. **Reversible** hacia
(b) si cinco reintentos no convergieran en un nodo con tráfico.

### D-U — Se guardan los sobres, el aviso y la credencial; la cabeza, no aparte

Medido sobre `8d064c6`: el sobre lleva la cabeza VERBATIM (D-J), y una copia aparte serían dos
fuentes del mismo dato. Dos caminos: (a) el banco guarda los dos positivos, los siete cuerpos
negativos, el aviso y la credencial, y no la cabeza; (b) también la cabeza, en su fichero. Gana
(a): claridad (un dato, una fuente: la lección de D-J) e imagen fiel (lo que el catálogo copia es
lo que el mando lee). **Reversible** hacia (b) sólo por la vía que D-J deja abierta: un
consumidor medido que necesite la cabeza sin abrir el sobre.

### D-V — El negativo de la regla del juez es la cota por encima del importe

Medido sobre `8d064c6`: `inferior` es entrada pública del AIR por una aserción de frontera
(`COL_INFERIOR`), y el productor de la capa rehúsa una cota por encima del importe por su nombre,
«la banda NO se sostiene». Dos caminos: (a) el negativo del juez es `inferior = importe + 1`: el
sobre del positivo de la banda con esa cota, que el juez rechaza con `cobro:` delante de lo que
pone winterfell, y que además la boca rechaza EN VIVO sin escribir nada; (b) una prueba corrupta
o un receptor movido, que caen por la misma regla (D-Q). Gana (a): el mismo hecho tiene dos
testigos —el productor que se para y el juez que no verifica— y una mutación que se lee en el
enunciado. **Reversible** hacia (b) si una captura enseñara que ésta cae por otra regla.

### D-W — Fuera del canon, como el de edad; lo corre el bloque que sella

Medido sobre `8d064c6`: un banco con nodo vivo, latido y una carrera que se reintenta no es una
puerta determinista, y el canon ya cuesta entre 189 y 325 s (5.A-223). Dos caminos: (a) fuera
del canon, como `banco_edad.sh`, y lo corre el bloque que sella lo que el banco produce; (b)
dentro del canon. Gana (a): coherencia (el de edad) y que lo que el canon vigila es el catálogo,
que sí es determinista (D-X). **Reversible** hacia (b) sólo si un banco pudiera correr sin nodo.

### D-X — La fuente del catálogo son las capturas del SELLO

Medido sobre `f1e0401` (`TERRENO-C-150`) y en el PASTE-499-PRE: hay dos juegos de capturas, el
de la corrida suelta y el de la corrida del §498, y los nueve sobres son los MISMOS bytes en los
dos salvo `emittedAtUnix`. Dos caminos: (a) la fuente es la corrida del sello, la que el asiento
498 nombra por su SALIDA; (b) la suelta. Gana (a): imagen fiel (una fuente que el asiento no
nombra no es fuente, y el bloque la re-mide por huella antes de copiar). **Reversible** hacia
otra corrida si el banco cambia y el sello se rehace.

### D-Y — Entran los nueve sobres; el aviso y la credencial, declarados por huella

Medido sobre `f1e0401`: `tools/conformidad.sh` exige una entrada del manifiesto por cada `.json`
del directorio, y el mando no lee ni el aviso ni la credencial: no son sobres. Dos caminos: (a)
entran los nueve sobres, y el manifiesto declara por su huella el aviso y la credencial de los
que salieron; (b) entran los once ficheros. Gana (a): el arnés manda la forma, y pureza (el
catálogo es de sobres, como los otros cinco). **Reversible** hacia (b) si un consumidor del
catálogo necesitara reproducir la boca desde él.

### D-Z — Los nombres, tal cual los guardó el banco

Medido sobre `f1e0401`: un vector se copia byte a byte de su captura y no se renombra; el nombre
es el que la SALIDA del sello imprime, y por él se re-mide. Dos caminos: (a) los nombres del
banco; (b) renombrar al estilo de otra familia. Gana (a): imagen fiel (lo que el catálogo dice
que es una captura se encuentra por su nombre en la captura). **Reversible** hacia (b) sólo si
el lado del pago trajera nombres que chocaran con éstos.

### D-AA — Lo que el manifiesto pina, y del juez sólo el prefijo

Medido sobre `f1e0401`: los positivos dan un VERDE de tres líneas cuya segunda dice el enunciado
(«por al menos 0, nacido en 4» y «por al menos 250000, nacido en 4»); los negativos caen con seis
textos de la casa y uno con prefijo, `cobro:`, seguido de lo que pone winterfell. Dos caminos:
(a) pinar la segunda línea de los positivos, los textos de la casa enteros y, del juez, SÓLO el
prefijo; (b) pinar el texto entero del juez. Gana (a): imagen fiel (el catálogo pina lo que la
casa afirma; lo que sigue al prefijo es de winterfell y cambia con él) y coherencia (el molde de
`neg-cota-movida` en `edad/`). **Reversible** hacia (b) si el manifiesto fijara winterfell por
versión.

### D-AB — El catálogo con toda su prosa en un corte (C-1); el RFC, aparte (C-2)

Medido sobre `f1e0401`: `FAMILIAS` de `tools/artefacto.sh` es el único productor del artefacto y
una sexta familia mueve el tarball, así que el catálogo corre el canon entero; y la prosa que un
catálogo deja rancia —`PAQUETE.md` 9, 11 y 2.8, los dos README y la fila de `spec/README.md`— se
paga en el corte que la desmiente, censando las FRASES del fichero y no los ficheros. Dos
caminos: (a) dos cortes, el catálogo con toda esa prosa y el canon dentro, y el RFC aparte y sin
canon; (b) uno. Gana (a): un corte de canon y otro de prosa no se mezclan, y el RFC no espera al
canon. **Reversible** hacia (b) sólo si un catálogo no moviera el artefacto.

### D-AC — La familia y la estrofa se llaman `pendiente`

Medido sobre `f1e0401`: el lado del pago irá al MISMO catálogo (D-O), y cada familia del canon
lleva una estrofa derivada de la de su hermana por sustitución. Dos caminos: (a) `pendiente`, el
objeto, con dos lados dentro; (b) `cobro`, el lado. Gana (a): coherencia con D-O y claridad
(nombres que significan una sola cosa: el catálogo es del pendiente). **Reversible** hacia dos
familias si el lado del pago exigiera un manifiesto propio, la misma condición que la de D-O.

⚠️ **REVERTIDA por D-AR (§509/§510)**: el lado del pago exigió manifiesto propio, que es la
condición escrita arriba. Son dos familias, `pendiente` y `pago`.

### D-AD — El enunciado del pago: `{receptor, importe, T}` exactos, y `nacido`, como en el cobro

Medido sobre `477dcab` (`TERRENO-E2-151`): el pagador tiene la apertura entera —receptor, sal,
importe, `refund_id`, `delta` y posición—, porque `send_materials_v2` la toma de él y
`SendMaterials` la guarda (`crates/zk-ssl/src/client.rs:201`, `:301`); la capa reembolsa cuando `now
- born >= delta`, con `now = log.len()` (`crates/zk-ssl/src/two_phase.rs:674`), así que «no
reversible antes de `T`» es exactamente `T <= nacido + delta`, en épocas del registro; y el sobre
del cobro ya publica `nacido` (`crates/zk-ssl/src/prueba_cobro.rs:54`). Dos caminos: (a) el
enunciado es `{receptor, importe, T, nacido}` —los tres exactos y el `nacido` de la meta, como en
E1—, y lo que el circuito prueba es `delta >= T - nacido`: una cota inferior de `delta`, nunca
`delta`; (b) `nacido` testigo, y el circuito prueba `nacido + delta >= T` con los dos escondidos.
Gana (a): coherencia (el mismo molde que E1 y el mismo enlace `nacido < seq`), claridad (un dato,
una fuente: `nacido` sale de la meta que la cabeza firma) y pureza (una comparación en vez de dos
rangos). **Reversible** hacia (b) sólo si un consumidor medido necesita esconder `nacido`, pagando
un rango más en el circuito.

### D-AE — El camino para el pagador: el mismo método, su credencial y una puerta en la meta

Medido sobre `477dcab`: `zkssl_pendingPath` exige la credencial de `index` y deriva de ella al
receptor (`crates/zk-ssl-node/src/main.rs:1826`); la capa no discrimina: `FotoPendientes::cobro`
recompone la hoja con el aviso y devuelve el camino, los hermanos de la meta, el `emisor` y el
`nacido` (`crates/zk-ssl/src/foto_pendientes.rs:62`, `:69`); un pagador tiene el aviso y conoce al
receptor, pero no tiene la clave de vista del cobrador: hoy no puede pedir el camino del pendiente
que pagó. Tres caminos: (a) el MISMO método, con la credencial del PAGADOR, `receptor` como
parámetro en vez de derivado, y una puerta nueva en el nodo: la meta tiene que nombrar a quien pide,
`foto.emisor == index`; (b) un método nuevo, `zkssl_pendingPathPagador`; (c) que el receptor le pase
la foto. Gana (a): la misma noción de foto y de credencial (§261), cero métodos nuevos, y
fail-closed: «la meta no te nombra» recibe la misma nada que un aviso ajeno, sin decir qué hay. La
(c) haría depender al pagador de su contraparte para probar contra ella. **Reversible** hacia (b) si
la puerta doble ensuciara el método del cobrador.

### D-AF — La geometría de E1, con el hueco lleno

Medido sobre `477dcab`: `circuit_refund_v2` ya abre el compromiso entero —`(receptor, aleatorio, f,
delta)` con `C2` y el importe públicos— en cuatro merges, componiendo el sobre PRIMERO
(`crates/stark-experiment/src/circuit_refund_v2.rs:5`, `:15`); en el AIR de E1 el carril B no hashea
en los ciclos 0 y 1 (`crates/zk-ssl-air/src/cobro_pendiente.rs:20`), el enlace de `X` está en la
fila 15 (`:91`) y la banda del importe gasta tres segmentos de 64 filas (`:62`). Dos caminos: (a) la
geometría de E1: el carril B compone `X = M(refund_id, [delta, 0, 0, 0])` en sus ciclos 0-1 y el
enlace del ciclo 2 lo lee del carril B en vez de cuatro columnas de testigo; el importe deja de
tener banda —es público y el enlace del ciclo 1 ya lo ata— y sus segmentos prueban `delta - (T -
nacido)` en `[0, 2^62)`; (b) un tercer carril, o una traza de 1024. Gana (a): pureza (misma traza,
mismas columnas más `delta` y `T`) y coherencia (el molde del cobro, con `X` atado por estructura y
no por testigo). Coste RAZONADO, no medido: se mide con el instrumento de E1 (§485) antes de
escribir el AIR, como D-B manda. **Reversible** hacia (b) si el coste medido no cupiera.

### D-AG — `delta` y `T` viven en el campo; la cota del «nunca», declarada

Medido sobre `477dcab` y en el PASTE-E2-PRE (SALIDA 20260918-155647): `refund_envelope` mete `delta`
por `BaseElement::new` (`crates/zk-ssl/src/pending.rs:92`), Goldilocks reduce `u64::MAX` a `2^32 -
2` (`crates/stark-experiment/src/range_check.rs:10`), y un test en una copia del árbol lo midió con
prueba de vida: `refund_envelope(f, u64::MAX) == refund_envelope(f, 2^32 - 2)`, y `C2` ídem. Dos
caminos: (a) `T` y `nacido` en épocas del registro, `T - nacido` y `delta` acotados a 62 bits como
`MAX_VALOR`, y el RFC DECLARA que un sobre comprometido «nunca» prueba a lo sumo `T <= nacido + 2^32
- 2`; (b) que E2 cierre el sobre al campo. Gana (a): imagen fiel (el circuito prueba lo que el
compromiso ata, que es módulo `p`) y perímetro (la letra del 0003, «las elecciones del emisor atan»,
es del 0003: el punto va a su cola, y lo que decida allí —declarar o cerrar el sobre al campo, que
es rotura de formato— no es de este RFC). **Reversible** hacia (b) sólo si el 0003 cierra el sobre
al campo: entonces la cota desaparece y este párrafo pasa a historia.

### D-AH — El enlace entre las dos mitades: nada nuevo, y el fuerte para la prenda

Medido sobre `477dcab`: `SobreCobro` publica `receptor`, `nacido`, `inferior`, `seq` y las dos
raíces, y calla el importe, la sal, `X` y el emisor (`crates/zk-ssl/src/prueba_cobro.rs:52`); D-I
dejó escrito que sin `X` en el sobre nada público enlaza las dos mitades y que la etiqueta de la
prenda, `H(dominio, C2)`, es la candidata. Dos caminos: (a) nada nuevo: un tercero cruza `receptor`,
`nacido` y `seq`, que los dos sobres publican, y el enlace fuerte queda para la prenda (E3); (b)
publicar `C2` en los dos sobres. Gana (a): `C2` en el sobre haría enlazables todos los sobres de una
misma hoja y rompería lo que las posiciones saladas prometen (D-H), y una disputa se abre con la
prenda, que es la que la nombra. **Reversible** hacia (b) sólo con un caso de uso medido que la
prenda no cubra.

⚠️ **Lo que la prenda entrega, medido en la 159**: el enlace fuerte es para quien tiene el AVISO,
no para un extraño con los dos sobres. Ver D-AY, que lo mide y lo declara sin sustituir esta
decisión: el camino (a) de aquí sigue siendo el que se tomó.

### D-AI — La credencial de retorno es del pagador: tercer fichero, `--retorno`

Medido sobre `477dcab`: `simulate --v2` fija la pareja de la conformidad —`f` = la identidad del
emisor, `delta` = 96— (`crates/zk-ssl-cli/src/commands.rs:68`) y escribe el aviso y la credencial
del receptor en dos ficheros (D-P), pero no persiste nada del pagador: `(refund_id, delta)` se
derivan de la semilla y no dejan fichero. Dos caminos: (a) `simulate --v2 --retorno` escribe
`{refundId, delta}` en un fichero PROPIO del pagador, y la boca `prueba-pago` lee el aviso, el
retorno y la credencial del propio pagador: tres ficheros, dos dueños; (b) que la boca los derive de
la semilla. Gana (a): coherencia de actores (D-P: un cobrador real tiene su credencial, un pagador
real custodia su retorno), claridad (una boca que un pagador real puede usar) y nombres que
significan una sola cosa (el aviso no se recicla para llevar el retorno). **Reversible** hacia (b)
sólo si la traza del sandbox ya lo dijera todo, que es la reversión de D-P.

### D-AJ — El sobre 2.9 y sus vectores, en `pendiente/`

Medido sobre `477dcab`: la forma 2.8 es `{v, tipo: "cobro_pendiente", cabeza, enunciado, prueba}`
con la cabeza VERBATIM y las raíces sólo de ella (`spec/PAQUETE.md:239`); el mando dispatcha por
`tipo` (`crates/zk-ssl-verify/src/main.rs:167`); la primera línea del manifiesto de `pendiente/`
dice «Vectores del sobre de COBRO PENDIENTE» (`spec/vectors/pendiente/MANIFIESTO.txt:1`); y
`pago_en_curso` no existe en ningún `.rs`, `.sh`, `.json` ni `.txt`. Dos caminos: (a) la forma 2.9,
`tipo: "pago_en_curso"`, con `enunciado: {receptor, importe, t, nacido}` y su lista de rechazos
propia (D-E); los vectores `pago-*` en el mismo `pendiente/` (D-AC), reunidos de la MISMA siembra
que los del cobro (D-S): dos positivos por forma del enunciado —la `T` exacta, `T = nacido + delta`,
y una `T` menor, porque «no antes de T» admite todo `T <= nacido + delta` como `inferior` admite
todo valor bajo el importe— y un negativo por regla producible (D-Q); la primera línea del
manifiesto se paga en ese corte; (b) un tipo con `lado`. Gana (a): D-E ya lo decidió y las listas de
rechazo son distintas (la del pago lleva la `T`), y la frase del manifiesto quedaría rancia y se
paga donde se desmiente. **Reversible** hacia (b) sólo por la vía que D-E deja abierta: dos listas
de rechazo idénticas, que hoy no lo son.

⚠️ **La forma 2.9 se mantiene; el DIRECTORIO no** (D-AR, §509/§510): los vectores `pago-*`
no viven en `pendiente/` sino en `spec/vectors/pago/`. Lo que cayó fue la premisa: esta
decisión los daba <<reunidos de la MISMA siembra que los del cobro (D-S)>>, y no lo son. El
texto de arriba se CITA, no se borra (§247).

### D-AK — El productor del pago es una función libre, con la apertura entera y sin libro

Medido sobre `48699ab`: el productor del cobro (§491) es una `fn` libre que no abre libro, y el
pagador tiene lo mismo más la pareja `(refund_id, delta)` que el receptor recibe opaca. Dos
caminos: (a) fichero propio `crates/zk-ssl/src/prueba_pago.rs` al lado del molde, con
`AperturaDelPago` como entrada y reusando `FotoDelCobro` y `CabezaDePendientes` tal cual; (b) un
método del libro. Gana (a): no lee libro, luego corre en un cliente que no lo tiene, y reusar los
dos tipos evita dos productores del mismo dato. El productor **re-verifica lo que produce** con el
mismo juez que corre el tercero, y comprueba `nacido < seq` ANTES de gastar una prueba, porque un
rc que dice «no se enlaza» esconde la causa. **Reversible** si el pendiente dejara de caber en una
`fn` libre —hoy no: la foto se la sirve el nodo—.

### D-AL — El nodo anterior al §505 ignora `receiverId`: se DECLARA, no se arregla

Medido en el terreno de E2e: la `P` de `zkssl_pendingPath` nunca llevó `deny_unknown_fields`, así
que un nodo viejo toma el campo de más, lo tira y responde como si no viniera: sirve la NADA en
vez de un error, y el pagador no sabe por qué. Dos caminos: (a) declararlo donde lo lee quien lo
sufre —`spec/RPC.md` y `PAQUETE.md` 2.9—, diciendo que lo que dice si un nodo sabe de qué habla es
su `spec/openrpc.json` y no la versión del protocolo, que no sube; (b) subir la versión. Gana (a):
subirla por un campo aditivo rompe el cable para todos por un caso que sólo sufre quien habla con
un nodo viejo. **Reversible** si apareciera un tercero que no pueda leer el OpenRPC.

### D-AM — La boca del pagador es un fichero propio, y tres privadas suben al crate

Medido sobre `6ecb063`: la boca del cobro (§497) tiene lo público reutilizable —`leer_cabeza`,
`leer_foto`, `escribir`, `notice_de`, `credencial_de` y sus dos DTO— y CUATRO funciones privadas,
de las que la del pago usa tres (`q_de`, `b32_de`, `respuesta`). Dos caminos: (a) `pago.rs` propio
y esas tres a `pub(crate)` en su sitio; (b) el pago dentro de `cobro.rs`. Gana (a): la cabecera de
`cobro.rs` dice «la BOCA del cobrador», y meter el pago dentro la haría mentir —vara 2: un nombre,
una cosa—. `cuerpo` se queda privada, que sólo la usa `respuesta`: abrir lo que no se usa es
cosmético. **Reversible** hacia un módulo común si naciera una tercera boca.

### D-AN — El retorno viaja en su fichero, y una puerta barata lo falsa contra el aviso

Medido en el PASTE-E2g-M: `simulate --v2` no persistía nada del pagador —la pareja se derivaba de
la semilla— y `refund_envelope` es el ÚNICO productor de `X`. Dos caminos: (a) un tercer fichero
`{refundId, delta}` escrito por `simulate --v2 --retorno`, con el precedente de D-P —un fichero
por dueño—, y la boca comprobando `refund_envelope(retorno) == aviso.x` ANTES de pedirle nada al
nodo; (b) banderas sueltas. Gana (a): la puerta cuesta cero, falla cerrada y ahorra una prueba, y
el fichero se pasa como se pasa el aviso. Lo que la puerta NO prueba va escrito en la doc de la
función: `delta` entra en Goldilocks y dos deltas congruentes dan el mismo sobre (D-AG); descarta
lo evidente, no fija el delta. **Reversible** hacia (b) en un cliente sin ficheros.

### D-AO — `--t` es ABSOLUTO

Medido: el productor toma `T` absoluta y el enunciado la publica (`PAQUETE.md` 2.9). Dos caminos:
(a) absoluta; (b) relativa a `nacido`. Gana (a): relativa obligaría a leer la foto antes de
componer la orden, y la MISMA orden daría sobres distintos según cuándo se corre. **Reversible**
si una boca de alto nivel quisiera ofrecer las dos, con la absoluta como la que viaja.

### D-AP — La credencial del PAGADOR también va en fichero, y su testigo es el banco

Medido al escribir el banco: `simulate --v2 --credencial` escribe la del RECEPTOR, y
`zkssl_pendingPath` exige la del PAGADOR cuando quien pide es el pagador (D-AE). Ese fichero no
existía, y la boca lo salva por bandera sólo porque un humano puede teclearla; un banco no. Dos
caminos: (a) `--credencial-pagador`, la MISMA terna y la MISMA `credencial_de` con `a.from`; (b)
meterla dentro del retorno. Gana (a): son dos objetos distintos con dos tipos distintos, y juntar
la terna con la pareja daría un fichero que no es ni una cosa ni la otra. Su testigo NO es un
unitario: si escribiera la del receptor, el nodo la aceptaría —es válida— y la puerta
`f.emisor == p.index.0` serviría la nada, así que el falsador es el banco. **Reversible** hacia un
solo fichero del pagador si un día el retorno dejara de ser opaco.

### D-AQ — El banco del pago es propio, con su siembra de cuatro ficheros

Medido en el §508: el hermano del cobro (§498) siembra dos ficheros de dos dueños; el pago
necesita cuatro, y su rechazo barato no llega ni a pedir. Dos caminos: (a) `tools/banco_pago.sh`
propio, con el molde del hermano y el mismo orden que la foto del latido impone; (b) un parámetro
`--lado` en el del cobro. Gana (a): las dos siembras ya no son la misma —la del pago escribe el
retorno y la credencial del pagador— y un banco con dos modos es un banco que un día prueba el que
no era. **Reversible** hacia (b) si las dos siembras volvieran a coincidir.

### D-AR — El catálogo del pago va en `spec/vectors/pago/`, séptima familia

Medido en el §509, y esto REVIERTE D-AC y la letra de D-AJ por la puerta que D-AC dejó escrita
—«si el lado del pago exigiera un manifiesto propio»—, con tres hechos: la siembra del pago es
OTRA (D-AQ), la lista de rechazos es distinta (D-E ya lo decía) y `tools/conformidad.sh` exige que
cada `.json` del directorio tenga entrada, así que un solo directorio sería un manifiesto de
dieciocho entradas mezclando dos sobres y dos bancos, con UNA cuenta en el canon donde hay dos.
Dos caminos: (a) `pago/` con su manifiesto, su estrofa y su familia; (b) `pendiente/` con los
dieciocho. Gana (a), y de propina el manifiesto del pendiente no queda rancio. ⚠️ **Se tomó de
hecho en el §509 ANTES de escribirse aquí**, siguiendo la línea del TRASPASO que citaba D-AJ y
decía «la séptima familia»: un resumen que se había apartado de la decisión que cita. La lección
va al asiento. **Reversible** hacia (b) si las dos listas de rechazo se igualaran.

### D-AS — La MARCA y la PRENDA son dos objetos; D-C se sostiene en (a), enmendada

Medido en el terreno de E3 sobre `0bfe066`: la hoja del árbol de consumos ES el digest entero
(`consumo.rs:143`) y la posición son sus 63 bits bajos (`zk-ssl-hash:243`), así que la capa
recibe un opaco y no puede distinguir clases sin la preimagen; y `apply_consumo` tiene diecisiete
líneas de cuerpo con cero dominio, cero prueba y cero autorización, que es lo que su propio
doc-comment ya declaraba por D-4 del 0006. La condición que D-C se dejó escrita **se cumple**: el
árbol no distingue clases. Pero lo que cae no es (a): es el NOMBRE, que significaba dos cosas. Se
parte en dos. La MARCA es `H(DOMINIO_PRENDA, C2)`, una hoja más del árbol de consumos, pública y
precomputable por quien tenga el aviso —medido: el aviso con el sobre opaco recompone `C2` exacto,
`two_phase.rs:3364`—, y **no afirma nada por sí sola**. La PRENDA es el PAR: la marca bajo la raíz
firmada más el sobre con la prueba de apertura del cobro; un tercero exige el par. Así la garantía
no la sostiene quién escribió la etiqueta —puede escribirla el pagador, que también tiene el
aviso— sino quién pudo producir el sobre, que es sólo el receptor: la titularidad que
`circuit_claim_v2` ya restringe en `COL_KEY` 25..29 con `SPEND_KEY_DOMAIN`. Gana a (b) por la vara
entera: una primitiva por propiedad, ni una columna nueva, ninguna era de formato y ningún vector
vivo caduca. **Reversible** hacia (b) sólo con un caso de uso medido que exija exclusividad de la
POSICIÓN y no sólo de la prueba; entonces se paga la era 5 → 6 con los vectores de edad detrás.

⚠️ **El sobre del par NO es el del cobro**: el de E1 no restringe titularidad (D-G), medido otra
vez en la 159 —cero clave de gasto en `cobro_pendiente.rs` y en `circuit_cobro_pendiente.rs`—.
El par lleva el sobre de PRENDA, que sí la lleva. Ver D-AV, que decide su prueba sin sustituir
esta decisión.

### D-AT — El límite de la marca, y va escrito donde lo lee quien lo sufre

Cualquiera con el aviso puede publicar la marca antes que el receptor, por la boca libre
`zkssl_publishConsumo`, que no pide prueba ni autorización; y quien sólo mire el árbol no sabe si
hubo prenda. No es doble uso ni denegación: es que la marca sola no prueba nada (D-AS), y es la
misma clase que la D-4 del 0006 ya publica —quien publica primero bloquea—. Adelantarse tampoco
daña al receptor: la marca que el pagador escribiera es **la misma hoja** que el receptor
necesitaba, y el sobre sigue siendo suyo. La denegación real sería una `ConsumoColision` DIRIGIDA
—otro digest en los mismos 63 bits bajos—, que la capa ya rechaza con su nombre
(`consumo.rs:222`) y que cuesta del orden de 2^63 intentos: **razonado, no medido**. Esto se dice
en Seguridad de este RFC y, cuando E3 se monte, en `spec/PAQUETE.md`, que es donde lo lee quien va
a prendar. **Reversible** sólo si una medida rebajara esa cota, y entonces la prenda pide árbol
propio.

### D-AU — D-D se estrecha en una palabra: segunda prenda PROBADA

Donde D-D dice «no hay segunda prenda» pasa a decir «no hay segunda prenda PROBADA», y donde la
fila E3 dice «una segunda prenda es `ConsumoRepetido`» pasa a decir «una segunda MARCA». El
rechazo con prueba es el mismo y sigue donde estaba (RFC-0007 E3, `PAQUETE.md` 2.6): lo que cambia
es de qué objeto habla. El resto de D-D se ratifica sin tocar una coma: la prenda no toca el cobro
ni el reembolso, un pendiente prendado se cobra y se reembolsa igual tras `T`, y lo que obliga al
cobrador con su financiero es contrato —el sistema produce el par condenatorio, no adjudica—.
**Reversible** hacia «la prenda bloquea el reembolso hasta `T`» sólo con su testigo negativo
escrito antes y con la regla comprometida en la cabeza, porque una regla que no está comprometida
no sostiene una prueba.

### D-AV — La prueba de la prenda es AIR propio: el molde del cobro con el ciclo de la clave

Medido sobre `4aee725`: `crates/zk-ssl-air/src/cobro_pendiente.rs` y
`crates/stark-experiment/src/circuit_cobro_pendiente.rs` no nombran la clave de gasto ni una
vez, y las entradas públicas del juez del cobro son `{pending_root, pmeta_root, receptor,
nacido, inferior, superior}`: el enunciado de ESTADO que D-G eligió a propósito. El del pago
tampoco la nombra. Sólo `circuit_claim_v2` la lleva —ocho veces, con `SPEND_KEY_DOMAIN` en
`COL_KEY` 25..29—, y va soldada al crédito: sus entradas públicas son la transición entera
(`ClaimPublicInputs`, con las dos raíces de cuentas, el importe y el suministro) y sus
restricciones acreditan el saldo (`C_BALANCE` y `C_SUPPLY`, `circuit_claim_v2.rs:855` y `:857`).
Tres caminos. (a) El par lleva el sobre del cobro, que ya existe: cae, porque ese sobre no
restringe titularidad y el pagador lo produce igual (D-G), así que el par no condenaría a nadie.
(b) `circuit_claim_v2` «sin el crédito», como si fuera un modo suyo: cae, porque no es un modo
sino otro circuito —quitar el crédito cambia las entradas públicas y las restricciones—, y
dejarlo publicaría el importe y las raíces de cuentas, que la prenda no necesita. (c) AIR
propio: el molde de `cobro_pendiente` más el ciclo de la clave que D-G dejó nombrado para aquí
(`CYC_PK`: un ciclo y cuatro columnas más). Gana (c) por la vara 1 y la 4: lo que el circuito no
restringe no existe, y una primitiva por propiedad —el enunciado de la prenda es «bajo esta
cabeza firmada existe este `C2` a mi nombre, y soy quien podría cobrarlo», sin transición y sin
crédito—. La objeción que D-G puso a su (b) no muerde aquí: allí se quería atar la prueba a
quien la PRESENTA, y eso pide un reto dentro del enunciado; la prenda sólo necesita que nadie
más HAYA PODIDO producirla, que es justo lo que el ciclo de la clave da. El coste va declarado:
pin en `zk-ssl-air` y en `stark-experiment`, y los ~40 s por sello que cuesta un testigo que
produce y verifica un STARK real (puntos 303 y 310). Esto decide la PRUEBA, no el montaje: el
método, el dominio sexto y el octavo brazo del mando siguen sin escribirse, y E3 sigue
propuesta. **Reversible** hacia (a) sólo si un caso de uso medido mostrara que basta la marca
con un sobre de estado; entonces la prenda deja de ser autorización y es etiqueta, y D-D cae con
ella.

### D-AW — El sobre de la prenda publica la MARCA; `C2` se queda de testigo

Un tercero tiene que poder cruzar lo que el sobre dice con lo que el árbol de consumos lleva. Dos
caminos: (a) el sobre publica `C2` y quien juzga calcula la marca fuera del circuito; (b) el
sobre publica la MARCA y el circuito prueba dentro que `marca = H(DOMINIO_PRENDA, C2)`, con `C2`
de testigo. Cae (a): publicar `C2` hace enlazables todos los sobres de una misma hoja y rompe lo
que las posiciones saladas prometen, que es exactamente lo que D-AH descartó por su lado. Gana
(b): la marca es lo que el árbol lleva, así que quien juzga compara lo que ve con lo que el sobre
dice y no calcula nada; y el molde ya está escrito en la casa —el carril B de E1 arranca su ciclo
poniéndose su dominio en la capacidad, `b[0] = DOMINIO_META_PENDIENTE`
(`circuit_cobro_pendiente.rs:157`)—, así que el dominio sexto entra en el circuito como
CAPACIDAD y no sólo como fila del REGISTRO de `zk-ssl-hash`. Cuesta un ciclo de hash, y cabe sin
una fila nueva (D-AX). **Reversible** hacia (a) sólo si el árbol de consumos dejara de ser
público, que hoy lo es por el RFC-0006.

### D-AX — La geometría: ni una columna nueva ni una fila más

Medido lado a lado sobre `3806f1e` (`PASTE-E3GEO-M`, lectura pura, rc 0): los dos jueces que
existen comparten `ANCHO` 44, `TRAZA` 512 y `ROW_RAIZ` 279, y se diferencian en piezas
contables —E2 lleva diez restricciones más y dos periódicas menos que E1—. La banda que la
prenda no necesita cuesta, contada de su propio fichero, CUATRO columnas (`COL_INFERIOR`,
`COL_SUPERIOR`, `COL_SBIT`, `COL_SACC`), OCHO restricciones (de `C_SBIT_BOOL` a
`NUM_RESTRICCIONES`), CINCO periódicas y 192 filas. El ciclo de la clave, tasado donde vive hoy,
es UN ciclo y CUATRO columnas con doce restricciones (`C_KEY_INPUT` ocho y `C_PK_CHECK` cuatro).
Y bajo la raíz quedan 232 filas libres —veintinueve ciclos— cuando hacen falta dos. Luego la
clave ocupa las columnas que la banda deja y la marca se hashea en un ciclo libre: `TRAZA` se
queda en 512 y `ANCHO` no pasa de 44 en ninguna de las dos formas —44 exactos con la meta, 42 sin
ella (D-AY)—. La cuenta de restricciones queda del orden de las 120 de E2, y el positivo, del
orden de los 213-218 ms que E2 tarda en el mismo arnés, contra los 127-137 de E1. Eso último es
PREDICCIÓN y no medida: el instrumento no puede correr un circuito que no existe, y su puerta es
el AIR escrito. **Reversible** hacia un ancho mayor sólo si al escribirlo una restricción no
cupiera; entonces se declara la columna y su causa.

### D-AY — La prenda no lleva la meta, y el enlace con el cobro lo hace el AVISO

El enunciado de la prenda es «bajo esta cabeza firmada existe este `C2`, cuya marca es la que se
publica, y la clave que lo cobraría es mía». No necesita `nacido`, que es lo que la hoja de meta
aporta. Dos caminos: (a) llevar el carril de la meta, como E1 y E2, y publicar `nacido`; (b) no
llevarlo. Gana (b) por pureza —lo que el enunciado no necesita no entra— y porque sale más barato
de verdad: medido, el carril B no hashea en los ciclos 0 y 1 y sube la meta desde el 2 hasta la
raíz (`circuit_cobro_pendiente.rs:131` y `:140`), así que sin meta queda libre entero para la
clave y la marca, sin tocar una fila; y se van además `COL_EMISOR` y `COL_NACIDO`, con lo que el
ancho baja de 44 a 42.

⚠️ **Y hay que decir lo que esto le hace a D-AH**, que dejó escrito que «el enlace fuerte queda
para la prenda». Medido: no lo queda para un extraño. El sobre del cobro no publica `C2`, así que
quien sólo tenga los dos sobres sigue cruzando `receptor` y `seq`, que es el enlace DÉBIL que la
propia D-AH describe. Fuerte lo tiene quien tiene el AVISO: con él recompone `C2`, calcula la
marca y comprueba que las dos mitades hablan de la misma hoja. Eso le basta al prendatario, que
recibe el aviso al prendar, y es imagen fiel: el sistema produce el par condenatorio para quien es
parte, no una prueba frente a todos. **Reversible** hacia (a), y hacia meter la banda DENTRO de la
prenda para que diga ella sola el importe; esto último costaría 46 columnas, dos más que E1, y
pide un caso de uso medido que lo exija.

### D-AZ — `zkssl_pledge` verifica el sobre y sólo entonces escribe, y no pide credencial

El método aditivo que la fila E3 anuncia recibe `{prueba, receptor, marca, seq}`, compone el
enunciado con el `pendingRoot` de la cabeza firmada que el NODO custodia —nunca con uno que venga
de fuera—, llama a `zk_ssl_air::prenda::verificar_contra_cabeza` y sólo con verde llama a
`apply_consumo`. Cuatro caminos: (a) exigir el sobre sin mirarlo, que cae por imagen fiel —un
método que dice exigir algo que no comprueba guarda bytes que nadie verificó—; (b) verificar y
escribir; (c) verificar y no escribir, dejando la marca a la boca libre; (d) que el método no
exista. Gana (b) por coherencia y por lo que el propio juez declara: él no comprueba que la marca
esté bajo `consRoot` porque «es la puerta de quien escribe en él», y quien escribe es el nodo. El
patrón de la casa —`prove` en el cliente, `apply` VERIFICA sin ver la clave— se cumple al pie de
la letra, porque el sobre no lleva clave (D-AW).

⚠️ **No es una puerta del árbol de consumos, y esto cierra el 5.A-353.** La boca libre
`zkssl_publishConsumo` sigue abierta y sin pedir nada (D-AT): esto es una boca CON prueba al lado
de una boca libre. Y por eso mismo **no exige credencial**: la autorización es la prueba, no la
posesión de una clave de vista en el servidor.

⚠️ **Una prenda vale dentro de su época**, y es una restricción MEDIDA que no estaba escrita: el
nodo custodia UNA cabeza firmada y la pierde al reiniciar, así que el `seq` que declara quien
llama tiene que ser el de la última. Si no lo es, se rechaza NOMBRANDO el que hay —al revés que
la negativa muda de D-AE, porque la marca es pública y precomputable por quien tenga el aviso—.
El orden es `zkssl_pendingPath`, producir, `zkssl_pledge`, bajo el mismo latido. **Reversible**
hacia (c) si alguna vez se decide que el nodo no compile un verificador STARK; hoy `zk-ssl-air`
ya estaba en su grafo por vía de `zk-ssl-verify` y declararlo sumó cero nodos, medido.

### D-BA — Lo que devuelve: el molde del hermano aditivo, más un campo

En verde, `{accepted: true, yaEstaba, logSeq, s}`; en rojo, `{accepted: false, reason, data}` con
la forma del §454. `logSeq` es el molde exacto de `zkssl_publishConsumo`; `s` es el `seq` de la
cabeza contra la que se juzgó, que es lo que un tercero necesita para pedir después el camino; y
`yaEstaba` entra porque es lo único de la respuesta que quien llama no puede computar por su
cuenta (D-BB). No devuelve la marca, que la mandó él, ni el camino, que es `zkssl_consumoPath`.

⚠️ **Y no acepta `pendingRoot` como parámetro.** Recibirlo sería dejar que el que pide fabrique
la vara con la que se le mide, que es la misma razón por la que la cabeza no viaja en §248.
**Reversible**: añadir campos a una respuesta es aditivo y no sube `zkssl/0.3`; quitarlos, no.
Por eso se entra con los mínimos.

### D-BB — Un repetido cuyo sobre verifica NO es un fallo de la prenda

`zkssl_pledge` verifica SIEMPRE primero. Si la prueba no verifica, rechazo, y da igual lo que haya
en el árbol. Si verifica y la hoja ya está y es LA MISMA, la respuesta es `accepted: true` con
`yaEstaba: true` y la capa no se toca. Una `ConsumoColision` sí es rechazo, con su causa como
dato. Sale de cruzar dos decisiones ya tomadas: el par es la marca bajo la raíz MÁS el sobre
(D-AS), y la marca que escribiera el pagador es la MISMA hoja (D-AT). Si las dos son ciertas, el
par existe en cuanto el sobre verifica, lo escribiera quien lo escribiera, y devolver
`ConsumoRepetido` sería llamar fallo a un éxito.

⚠️ **La deducción que lo sostiene va declarada**: dos prendas distintas del mismo pendiente no
pueden existir, porque la marca es función de la hoja (`marca_prenda`, un solo productor) y la
hoja lleva dentro al receptor; con otra clave la hoja no sube a la raíz y el productor falla
cerrado. Su falsador —dos sobres del mismo aviso con dos claves distintas, y el segundo no
verifica— no es del nodo: vive donde vive el productor, y queda pendiente. **Reversible** si se
decide que la constancia debe distinguir QUIÉN escribió la hoja; entonces la prenda pide árbol
propio, que es la salida que la propia D-AT deja escrita.

## Lo que se DESCARTÓ al medir

1. Abrir `X` del lado del cobrador para probar la T: rompe D-2 del RFC-0003 (el receptor
   aprendería `refund_id`).
2. Revelar el importe en el sobre del cobrador en vez de la banda: contradice el texto del hito
   y el molde ya existe.
3. La prenda como campo de la meta: rotura de formato (5 → 6) y caducidad de los vectores de
   edad, para una propiedad que el árbol de consumos ya da.
4. Un «conjunto de gastados o prendados» propio, como el PDF lo nombraba: la primera medida
   dijo que el conjunto de uso único con raíz firmada ya existe, y que lo que falta es la
   autorización, no el árbol.
5. Que prende el pagador, o cualquiera con acceso al nodo: es la denegación de servicio del
   0006 aplicada al activo del cobrador.
6. Que la prenda bloquee el reembolso: regla nueva sobre los derechos del pagador sin testigo
   escrito y sin compromiso en la cabeza.
7. Servir el camino del pendiente con el molde de `zkssl_frozenPath`, del estado de ahora: en
   cuanto un pago cae entre latidos, no sube a ninguna cabeza firmada (D-F).
8. Reconstruir el árbol en el `seq` de la cabeza desde el registro: sus entradas no llevan la
   posición ni la hoja (D-F).
9. Firmar una cabeza a petición de quien pide el camino: quema índices XMSS (D-F).
10. La titularidad en E1 sin un reto en el enunciado: no ata la prueba a quien la presenta (D-G).
11. El cobrador en un carril, con la posición acumulada y una igualdad que la ate: un atado que
    la casa no ha escrito nunca y que depende de acordarse de escribirlo (D-H).
12. El sobre `X` como entrada pública del cobrador: con `refund_id` adivinable, dice quién pagó y
    cuándo caduca, y enlaza los sobres del mismo emisor (D-I).
13. Que el sobre del cobro repita el `seq` y las dos raíces: dos fuentes del mismo dato
    firmado (D-J).
14. Los vectores del cobro dentro de E1: contradice la fila E4 adoptada (D-L).
15. La boca del cobrador en un modo del nodo, que cambia de actor, o en el SDK, que paga por
    una vía que E1 rehúsa (D-M).
16. Dos pendientes de dos corridas como los dos positivos del cobro: la misma forma dos veces
    no enseña el rango que el enunciado admite (D-N).
17. E4 en dos filas, E4a y E4b: la tabla del RFC-0007 nombra sus cortes en la celda de estado
    y nunca partió una etapa (D-O).
18. Que la boca del cobrador derive su credencial de la semilla del sandbox, o que un método
    del cable la sirva: la primera sólo vale en el sandbox y la segunda haría que el nodo
    entregue lo que sólo autoriza a leer (D-P).
19. Un vector por pieza mutada donde la regla es la misma: el catálogo pina la regla (D-Q).
20. Un banco por lado de E4: una fila, un catálogo y dos bancos (D-R).
21. Una siembra por positivo: dos pendientes donde D-N pide dos formas del mismo (D-S).
22. Que la boca reintente, o un latido largo que tape la ventana: la boca sigue sin reintentar y
    el banco dice cuántas veces lo hizo (D-T).
23. Guardar la cabeza aparte del sobre que ya la lleva: dos fuentes del mismo dato (D-U).
24. La prueba corrupta o el receptor movido como negativo del juez: caen por la misma regla que
    la cota, y la cota además la rechaza la boca en vivo (D-V).
25. El banco dentro del canon: un nodo vivo con latido no es una puerta determinista (D-W).
26. Las capturas de la corrida suelta como fuente del catálogo: el asiento no las nombra (D-X).
27. El aviso y la credencial dentro del catálogo: no son sobres, y el arnés les exigiría una
    entrada (D-Y).
28. Renombrar los vectores: un vector se copia, no se renombra (D-Z).
29. Pinar el texto entero del juez: lo que sigue a `cobro:` lo pone winterfell (D-AA).
30. El catálogo y el RFC en un solo corte: un corte de canon y uno de prosa (D-AB).
31. Llamar `cobro` a la familia: el lado del pago va al mismo catálogo (D-AC).
32. `nacido` testigo en el sobre del pago: dos rangos donde basta una comparación, y otro molde
    que el de E1 (D-AD).
33. Un método nuevo para el camino del pagador, o que el receptor le pase la foto: dos nociones de
    foto, o depender de la contraparte (D-AE).
34. Un tercer carril o una traza de 1024 para el pago: el carril B tiene los ciclos 0 y 1 libres
    (D-AF).
35. Que E2 cierre el sobre al campo: es la letra del 0003 y una rotura de formato; aquí se declara
    la cota (D-AG).
36. Publicar `C2` en los dos sobres para enlazarlos: enlaza todos los sobres de una hoja (D-AH).
37. Que la boca del pagador derive su retorno de la semilla: sólo vale en el sandbox (D-AI).
38. Un tipo de sobre con `lado`: las listas de rechazo del cobro y del pago no son idénticas
    (D-AJ).
39. El sobre del cobro (E1) como sobre de la prenda: no restringe titularidad —cero clave de
    gasto en su juez y en su probador, medido en la 159—, y el pagador lo produce igual (D-AV).
40. `circuit_claim_v2` «sin el crédito» como modo del mismo circuito: quitar el crédito mueve
    sus entradas públicas y sus restricciones, luego es otro circuito y no un modo (D-AV).
41. Publicar `C2` en el sobre de la prenda para que el enlace sea público: enlaza todos los
    sobres de esa hoja, que es lo que D-AH descartó por el otro lado (D-AW).
42. La prenda con la banda dentro, para que diga ella sola el importe: 46 columnas, dos más que
    E1, y el prendatario ya recibe el sobre del cobro (D-AY).
43. La prenda con el carril de la meta: publica un `nacido` que su enunciado no usa y cuesta dos
    columnas y el ascenso entero del carril B (D-AY).

## Compatibilidad

- `zkssl/0.3` **no sube**. `zkssl_pendingPath` y `zkssl_pledge` son métodos nuevos; ningún
  método ni objeto existente cambia de forma. La versión de FORMATO de la firma se queda en 5:
  la MARCA es una hoja del árbol que `consRoot` ya firma, y la prenda es el par (D-AS), que no
  vive en la cabeza.
- Los vectores de `cable/`, `nucleo/`, `paquete/`, `consumo/`, `conflicto/`, `rechazo/` y
  `edad/` no se tocan. Nace `pendiente/` bajo su propio manifiesto.
- El aviso `PendingNotice` no cambia: la mitad del cobrador se produce con lo que ya recibe.

### Por qué entra por RFC

Toca `spec/RPC.md` y `spec/openrpc.json` (dos métodos) y nacen vectores: las tres cosas que el
`PROCESO.md` reserva a un RFC. Y decide una función nueva del libro, la prenda, que merece su
expediente aunque no rompa nada.

## Seguridad

- **El operador ve.** Ninguna de las dos pruebas oculta nada al nodo: la privacidad es frente al
  verificador externo, como en todo el sistema.
- **Lo que el sobre del cobrador dice**: que bajo esa cabeza firmada existe un pendiente a
  nombre de `receptor`, por al menos `inferior`, nacido en `b`. Lo que NO dice: cuándo caduca,
  ni que vaya a cobrarse, ni quién lo pagó, ni quién produjo la prueba (D-G). El sobre `X` no
  viaja (D-I): con él, quien adivinara `refund_id` sabría quién pagó y cuándo caduca.
- **Lo que el método del camino revela**: a quien presenta un aviso que recompone la hoja, los
  dos caminos y la meta de ESA posición, en el estado del último latido; a quien no, nada (D-F).
- **Lo que el sobre del pagador dice**: que bajo esa cabeza el pago está comprometido, por
  `importe` exacto, a `receptor`, y que no puede revertirse antes de `T`. Lo que NO dice: que
  esté hecho; se consuma al cobrar.
- **Lo que la prenda dice**: que el cobrador autorizó la prenda de esa hoja, y lo dice el PAR —la
  marca bajo la raíz firmada más el sobre de PRENDA, que prueba la apertura Y la titularidad
  (D-AV)—, nunca la marca sola, y nunca el sobre del cobro, que cualquiera con la apertura
  produce. El sobre publica la marca y calla `C2` (D-AW), y no publica `nacido` (D-AY). Un
  segundo intento PROBADO tiene su prueba de rechazo (D-AU). Lo que NO dice: que el cobro vaya a
  ir al prendatario, ni —para quien no tenga el aviso— que esta hoja sea la del sobre del cobro.
  Las dos cosas son contrato.
- **Lo que la marca NO dice**: nada. Es una hoja pública y precomputable por cualquiera que tenga
  el aviso —el pagador lo tiene, porque lo construyó—, y publicarla por la boca libre del 0006 no
  impide la prenda: es la misma hoja que el receptor necesitaba. La denegación dirigida exigiría
  una colisión de posición del orden de 2^63 (D-AT).
- **La confianza en la cabeza** es la de siempre: la firma custodiada y las cofirmas de los
  testigos bajo umbral la sostienen, sin garantizar que un financiero la acepte como base
  (reto 4 de la propuesta, escrito ya).
- **Lo que la cabeza NO firma, medido**: `emittedAtUnix`. Entre la corrida suelta y la del sello
  del banco, los nueve sobres del catálogo son los mismos bytes salvo ese campo de la cabeza: la
  prueba STARK y la firma XMSS son deterministas, y la hora no entra ni en el digest ni en la
  firma (PASTE-499-PRE, SALIDA 20260918-112533). Es imagen fiel de la cabeza v5 del RFC-0007,
  no una regla de este RFC; si es deuda —declararlo en `RPC.md`, o cubrirlo—, se decide aparte
  (5.A-296).
- **Lo que un sobre «nunca» puede probar, medido**: el sobre de reversión mete `delta` en el
  campo, y Goldilocks reduce `u64::MAX` a `2^32 - 2`: dos `delta` que difieren en `p` dan el
  mismo `X` y el mismo `C2` (PASTE-E2-PRE, SALIDA 20260918-155647, medido en una copia del árbol
  con prueba de vida). Luego, en el circuito del pago, un pendiente comprometido «nunca» prueba a
  lo sumo `T <= nacido + 2^32 - 2`, en épocas del registro (D-AG). Es imagen fiel del compromiso
  v2 del RFC-0003; lo que el 0003 haga con su letra («las elecciones del emisor atan», hoy módulo
  `p`) se decide allí, y va a su cola.
- **Hasta dónde alcanza la cota temporal, medido**: el segmento de 64 filas del carril del pago
  NO prueba <<cabe en 64 bits>> —en este campo eso no diría nada, porque `p < 2^64`—: prueba
  **`v < 2^63`**, porque su primera fila exige bit y acumulador a CERO y el acumulador dobla en las
  63 siguientes. Luego el AIR sostiene el enunciado mientras **`delta - (T - nacido) < 2^63`**, y
  `comprobar_enunciado` acota `importe`, `T` y `nacido` a `MAX_VALOR = 2^62 - 1` y exige
  `nacido <= T`. El <<nunca>> del punto de arriba, que el campo reduce a `2^32 - 2`, cae holgado
  dentro. Quien lea un sobre por encima de esas cotas no lee un sobre: no hay ninguno.
- **La regla 3 del PROCESO —la clave de gasto no viaja jamás— hoy no se cumple aquí.** La prueba de
  prenda se produce en el cliente, como el cobro, pero publica la clave: sus filas abiertas la
  llevan en claro, 42 veces (§521).

## Referencias

- La lectura pura de la sesión 144 sobre `5ef3b1b`, `TERRENO-H5-144` (texto,
  `19c76ad04ad1d84b`/124; vive fuera del árbol, en Downloads del autor, como los PASTE de los
  RFC anteriores).
- La lectura pura de la sesión 145 sobre `393032e`, `TERRENO-E1-145` (texto,
  `953f0aeca3a1175c`/122; en Downloads del autor, como la anterior).
- La lectura pura de la sesión 147 sobre `be90eb7`, `TERRENO-AIR-E1-147` (texto,
  `70c808caa9918072`/126; en Downloads del autor, como las anteriores).
- La lectura pura de la sesión 149 sobre `7bb3942`, `TERRENO-COBRO-149` (texto,
  `e14211dd032dd4c0`/213, con 54 citas que `verifica_terreno149.py` re-aserta; en Downloads
  del autor, como las anteriores).
- La lectura pura de la sesión 150 sobre `9512915`, `TERRENO-E4-150` (texto,
  `705a74e9a9a26487`/294, con 55 citas que `verifica_terreno150.py` re-aserta, 55 de 55 en el
  árbol del autor con porcelain 0 antes y después; en Downloads del autor, como las
  anteriores).
- La lectura pura de la sesión 150 sobre `8d064c6`, `TERRENO-B-150` (texto,
  `2ed966d6bb84daf2`/225, con 54 citas que `verifica_terrenoB.py` re-aserta, 60 de 60 en el
  árbol del autor con porcelain 0 antes y después; en Downloads del autor, como las anteriores).
- La lectura pura de la sesión 150 sobre `f1e0401`, `TERRENO-C-150` (texto,
  `9d510f4b488375d0`/216, con 44 citas y las once capturas del sello por huella, que
  `verifica_terrenoC.py` re-aserta, 60 de 60; en Downloads del autor, como las anteriores).
- La lectura pura del corte C-1, PASTE-499-PRE (`ffefe25c314f6e53`/206; SALIDA
  20260918-112533), que midió que entre dos corridas del banco sólo se mueve `emittedAtUnix`.
- La lectura pura de la sesión 151 sobre `477dcab`, `TERRENO-E2-151` (texto,
  `274acf0873d38344`/287, con 54 citas y tres ausencias que `verifica_terrenoE2.py`
  (`e95466ce8a5b1f70`/173) re-aserta, 79 de 79 en el árbol del autor; en Downloads del autor,
  como las anteriores).
- El falsador de la sección 4 de ese terreno, medido en una copia del árbol por el PASTE-E2-PRE
  (`386cbcc36c4da6f3`/206; SALIDA 20260918-155647: `cargo test` verde, con prueba de vida).
- El instrumento de E1 y su corrida: `crates/zk-ssl/src/instrumento_cobro.rs` (§485) y la salida
  del PASTE que lo ensayó fuera del árbol (`96cf0169b0c09589`/44, en Downloads del autor).
- El hito, verbatim, en la línea 46 del formulario enviado (`NLNET-form-answers-EN-v3.txt`,
  `26dcde32091e857d`/160) y en la sección 3.6 de la propuesta adjunta (el PDF
  `a6d5b620bf4e2283`, 9 páginas).
- RFC-0003 (el compromiso v2: D-1 el sobre opaco, D-2 `refund_id` comprometido), RFC-0006
  (el consumo publicado y su D-4), RFC-0007 (la cabeza v5, el rechazo por caminos, la banda).
