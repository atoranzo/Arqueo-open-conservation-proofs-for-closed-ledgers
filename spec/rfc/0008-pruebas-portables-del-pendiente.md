# RFC-0008 — Las dos pruebas portables del pendiente: el cobro, el pago en curso y la prenda

- **Estado:** PROPUESTO
- **Autores:** Che, con Claude (sesiones 144, 145, 147, 149 y 150)
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
  que decide D-J, D-K, D-L y D-M y corrige la fila E4; y el §496, que decide D-N, D-O, D-P y
  D-Q y parte la fila E4 por lados.
- **Hito:** H5 de la propuesta enviada a NLnet Restack (140 h), en sus palabras: *«The two
  portable proofs of a pending item. Payee side and payer side, derived from the same head;
  pledge transition; format and vectors.»*

## Estado de las etapas

| etapa | qué entrega | ¿rompe el cable? | estado |
|---|---|---|---|
| E1 — el cobro pendiente, portable | el circuito del cobrador: bajo el `pendingRoot` de una cabeza v5 firmada existe `C2 = M(C1, X)` con `C1 = H(H(receptor, sal), importe)`, a nombre de `receptor` (D-G), `importe >= inferior` (banda, molde de `InsufficientBalance`), con el camino DENTRO del circuito; su meta `(emisor, nacido)` por camino bajo `pmetaRoot`, con los mismos bits. `zkssl_pendingPath`, aditivo, sirve los dos caminos de la foto del último latido a quien presenta un aviso que recompone la hoja (D-F). Sobre `tipo: "cobro_pendiente"` en `PAQUETE.md`, verificado por el mando sin nodo | NO | propuesta |
| E2 — el pago en curso, portable | el espejo, para el pagador: `C2` abre a `(receptor, importe)` EXACTOS, `nacido` por camino, y `nacido + delta >= T` con `delta` y `refund_id` como testigo (no se revelan). Sobre `tipo: "pago_en_curso"`, verificado sin nodo. Junto al de E1, un tercero ajeno a los dos verifica un pago disputado sin el libro de nadie | NO | propuesta |
| E3 — la prenda, como transición con prueba | el receptor marca el pendiente como prendado: una etiqueta con dominio propio sobre `C2` en el árbol de consumos, publicada por un método aditivo, `zkssl_pledge`, que EXIGE la prueba de apertura del cobro (la autorización de `circuit_claim_v2` sin el crédito); una segunda prenda es `ConsumoRepetido`, que ya tiene sobre de rechazo con prueba (RFC-0007 E3, `PAQUETE.md` 2.6). La prenda no toca el cobro ni el reembolso: lo que obliga es contrato, y se declara | NO | propuesta |
| E4 — el catálogo y el banco, por lados | `spec/vectors/pendiente/`: dos positivos por lado, REUNIDOS de las capturas de un nodo real (molde: `edad/`), y un negativo por regla producible; `MANIFIESTO.txt`; la familia en `FAMILIAS`; el banco que lo reproduce en vivo; la sección 9 de `PAQUETE.md`. Va POR LADOS en una sola fila (D-O): el del cobro primero —sus dos formas (D-N), la boca del cli (D-P) y sus negativos (D-Q)— y el del pago con E2; esta celda nombra lo sellado de cada lado. El giro a ACEPTADO exige la regla 4 medida letra a letra, como el §481 | NO | propuesta |

Las medidas de este documento se tomaron sobre `5ef3b1b` (`TERRENO-H5-144`); las de D-F y D-G,
sobre `393032e` (`TERRENO-E1-145`); y las de D-H, con el instrumento del §485, que corrió en una
copia fuera del árbol, y leyendo `0424439`; las de D-I, leyendo `be90eb7`
(`TERRENO-AIR-E1-147`); las de D-J a D-M, leyendo `7bb3942` (`TERRENO-COBRO-149`); y las de
D-N a D-Q, leyendo `9512915` (`TERRENO-E4-150`). Ninguna escribió un byte en el árbol (ver
Referencias).

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

### La frontera con H5b, y qué es de cada uno

Este RFC habla de un pendiente que ESTÁ en el árbol: existe, tiene importe, tiene edad, y se
puede prendar una vez. H5b habla de lo que entró por el cable y de si acabó aplicado o rechazado.
Un pago que el operador no aplicó no tiene hoja, no tiene `nacido` y no tiene prueba aquí: eso
es la completitud, y sigue siendo H5b. La prenda tampoco es completitud: es uso único de una hoja
viva, la misma propiedad que el consumo publicado da a una etiqueta.

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
  publica primero bloquea, y eso es denegación, no doble uso (D-4 del 0006). Una prenda que
  cualquiera pudiera poner sobre el pendiente de otro sería esa denegación: por eso la prenda
  entra por un método que exige la prueba del cobrador.
- **Los sobres portables tienen forma y juez**: `PAQUETE.md` 2.4 (consumo), 2.6 (rechazo) y
  2.7 (edad); el mando los verifica sin nodo y el artefacto los lleva dentro con su catálogo
  (`tools/artefacto.sh`, `FAMILIAS`). Dos tipos nuevos siguen ese molde sin tocar los viejos.

## Diseño

Las diecisiete decisiones las tomó el asistente por delegación del autor (D-A..D-E en la sesión
144; D-F, D-G y D-H en la 145; D-I en la 147; D-J..D-M en la 149; D-N..D-Q en la 150), con la
constitución de decisión (pureza, claridad, coherencia, imagen fiel, en ese orden). Todas llevan
su condición de reversión, escrita aquí.

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

### D-D — Prenda el receptor, y la prenda sólo obliga a la prenda

La autorización es conocimiento de preimagen o prueba: el pagador también tiene la apertura de
`C1`, así que «saber la apertura» no distingue. Lo que sólo el receptor tiene es lo que
`circuit_claim_v2` ya exige para cobrar: su identidad probada. La prenda es esa autorización sin
el crédito. Y la prenda NO toca el cobro ni el reembolso: un pendiente prendado se cobra igual y
se reembolsa igual tras T. Lo que la prenda garantiza es que no hay segunda prenda, con prueba
del rechazo; lo que obliga al cobrador con su financiero es contrato, y el sistema produce el
par condenatorio, no adjudica (§121.6). Una regla que bloqueara el reembolso sería una regla
nueva de la capa sobre los derechos del pagador: menos excepciones. **Reversible** hacia
«la prenda bloquea el reembolso hasta T» sólo con su testigo negativo escrito antes y con la
regla comprometida en la cabeza, porque una regla que no está comprometida no sostiene una
prueba.

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

## Compatibilidad

- `zkssl/0.3` **no sube**. `zkssl_pendingPath` y `zkssl_pledge` son métodos nuevos; ningún
  método ni objeto existente cambia de forma. La versión de FORMATO de la firma se queda en 5:
  la prenda es una hoja del árbol que `consRoot` ya firma.
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
- **Lo que la prenda dice**: que esa hoja fue prendada una vez, con la autorización del cobrador,
  y que un segundo intento tiene su prueba de rechazo. Lo que NO dice: que el cobro vaya a ir al
  prendatario. Eso es contrato.
- **La confianza en la cabeza** es la de siempre: la firma custodiada y las cofirmas de los
  testigos bajo umbral la sostienen, sin garantizar que un financiero la acepte como base
  (reto 4 de la propuesta, escrito ya).
- **La clave de gasto no viaja jamás** (regla 3 del PROCESO): la prueba de prenda se produce
  en el cliente, como el cobro.

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
- El instrumento de E1 y su corrida: `crates/zk-ssl/src/instrumento_cobro.rs` (§485) y la salida
  del PASTE que lo ensayó fuera del árbol (`96cf0169b0c09589`/44, en Downloads del autor).
- El hito, verbatim, en la línea 46 del formulario enviado (`NLNET-form-answers-EN-v3.txt`,
  `26dcde32091e857d`/160) y en la sección 3.6 de la propuesta adjunta (el PDF
  `a6d5b620bf4e2283`, 9 páginas).
- RFC-0003 (el compromiso v2: D-1 el sobre opaco, D-2 `refund_id` comprometido), RFC-0006
  (el consumo publicado y su D-4), RFC-0007 (la cabeza v5, el rechazo por caminos, la banda).
