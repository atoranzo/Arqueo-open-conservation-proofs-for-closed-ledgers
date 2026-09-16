# RFC-0008 — Las dos pruebas portables del pendiente: el cobro, el pago en curso y la prenda

- **Estado:** PROPUESTO
- **Autores:** Che, con Claude (sesión 144)
- **Fecha:** 2026-09-16
- **Versión del protocolo afectada:** `zkssl/0.3` — **no sube** (ver Compatibilidad). Los dos
  métodos nuevos son aditivos; la cabeza v5 no cambia de forma; la marca de prenda es una hoja
  más del árbol de consumos, cuya raíz la cabeza ya firma desde el RFC-0006.
- **Asiento(s) de AUDITORIA:** §121, §178, §211, §275 (el acuse y su raíz), §343–§345 (el
  compromiso v2 del RFC-0003), §387 y §388 (las raíces en reposo), §412–§441 (el consumo
  publicado), §451–§453 (la cabeza v5), §458–§460 (el rechazo por caminos), §463–§467
  (la prueba de edad), §473–§479 (la banda sobre la hoja comprometida); y el §483, que lo
  adopta.
- **Hito:** H5 de la propuesta enviada a NLnet Restack (140 h), en sus palabras: *«The two
  portable proofs of a pending item. Payee side and payer side, derived from the same head;
  pledge transition; format and vectors.»*

## Estado de las etapas

| etapa | qué entrega | ¿rompe el cable? | estado |
|---|---|---|---|
| E1 — el cobro pendiente, portable | el circuito del cobrador: bajo el `pendingRoot` de una cabeza v5 firmada existe un compromiso `C2 = M(C1, X)` con `C1 = H(H(receptor, sal), importe)`, `receptor` es la identidad pública de quien prueba, `importe >= X` (banda, molde de `InsufficientBalance`), y el camino va DENTRO del circuito (la posición no sale); su meta `(emisor, nacido)` por camino bajo `pmetaRoot`. Un método aditivo, `zkssl_pendingPath`, sirve los dos caminos a quien presenta el aviso (molde: `zkssl_frozenPath`). Sobre `tipo: "cobro_pendiente"` en `PAQUETE.md`, verificado por el mando sin nodo | NO | propuesta |
| E2 — el pago en curso, portable | el espejo, para el pagador: `C2` abre a `(receptor, importe)` EXACTOS, `nacido` por camino, y `nacido + delta >= T` con `delta` y `refund_id` como testigo (no se revelan). Sobre `tipo: "pago_en_curso"`, verificado sin nodo. Junto al de E1, un tercero ajeno a los dos verifica un pago disputado sin el libro de nadie | NO | propuesta |
| E3 — la prenda, como transición con prueba | el receptor marca el pendiente como prendado: una etiqueta con dominio propio sobre `C2` en el árbol de consumos, publicada por un método aditivo, `zkssl_pledge`, que EXIGE la prueba de apertura del cobro (la autorización de `circuit_claim_v2` sin el crédito); una segunda prenda es `ConsumoRepetido`, que ya tiene sobre de rechazo con prueba (RFC-0007 E3, `PAQUETE.md` 2.6). La prenda no toca el cobro ni el reembolso: lo que obliga es contrato, y se declara | NO | propuesta |
| E4 — el catálogo y el banco | `spec/vectors/pendiente/`: dos positivos por lado, REUNIDOS de las capturas de un nodo real (molde: `edad/`), y un negativo por regla producible; `MANIFIESTO.txt`; la familia en `FAMILIAS`; el banco que lo reproduce en vivo; `PAQUETE.md` 2.8. El giro a ACEPTADO exige la regla 4 medida letra a letra, como el §481 | NO | propuesta |

Todas las medidas de este documento se tomaron sobre `5ef3b1b`, en una lectura pura que no
escribió un byte en el árbol: `TERRENO-H5-144` (ver Referencias).

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
- **«Al menos X» es circuito, y su molde existe.** `crates/zk-ssl-air/src/banda.rs` abre una
  hoja bajo una raíz con el camino DENTRO y prueba una banda sobre un campo; el operador la
  produce sin la clave. Aquí la hoja es `C2`, la apertura son dos composiciones, y la banda es
  sobre `importe`. Revelar el importe en el sobre del cobrador contradiría la promesa.
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

Las cinco decisiones las tomó el asistente por delegación del autor (sesión 144), con la
constitución de decisión (pureza, claridad, coherencia, imagen fiel, en ese orden). Todas
llevan su condición de reversión, escrita aquí.

### D-A — La T es del pagador; el cobrador prueba «en mi nombre, al menos X, nacido en b»

El aviso no lleva `delta`, y no debe llevarlo abierto: D-1 y D-2 del RFC-0003 existen para que
el receptor no aprenda ni las elecciones de retorno ni la identidad de la clave de retorno. Tres
caminos se midieron: (a) que la mitad del cobrador diga lo que su dueño sabe —existe, importe
`>= X`, `nacido = b`— y la T la lleve la mitad del pagador, que sí lo sabe todo; (b) que el
aviso gane `delta` fuera del cable y el receptor abra `X` en circuito, lo que exige `refund_id`
como testigo y rompe D-2; (c) un compromiso v3 que separe `delta` de `refund_id`, que es rotura
de formato y nombre nuevo. Gana (a): pureza (no toca el 0003), claridad (cada mitad dice lo que
su dueño sabe y nada más), imagen fiel (la spec dirá que la caducidad la prueba el pagador). La
promesa del hito se cumple con las dos mitades juntas, que es exactamente como el texto del PDF
la formula. **Reversible** si un caso de uso medido exige la T del lado del cobrador: entonces
se abre (c), nunca (b).

### D-B — «Al menos X» es una banda en circuito, con el camino dentro

El molde es la banda de `InsufficientBalance` (RFC-0007 E5): la hoja abierta dentro del AIR,
el camino como testigo, dos cotas públicas. Aquí las cotas son `X` y el tope de importe, la
apertura es `C1` y luego `C2`, y `x` puede ser público porque es un compromiso. La posición no
sale del circuito: es lo que las posiciones saladas prometen. El coste se mide con el
instrumento de E4a antes de escribir el AIR, y si no cabe bajo un latido se declara con cifra.
**Reversible** sólo hacia una banda más estrecha; nunca hacia revelar el importe.

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
  nombre de quien prueba, por al menos `X`, nacido en `b`. Lo que NO dice: cuándo caduca, ni
  que vaya a cobrarse, ni quién lo pagó.
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
- El hito, verbatim, en la línea 46 del formulario enviado (`NLNET-form-answers-EN-v3.txt`,
  `26dcde32091e857d`/160) y en la sección 3.6 de la propuesta adjunta (el PDF
  `a6d5b620bf4e2283`, 9 páginas).
- RFC-0003 (el compromiso v2: D-1 el sobre opaco, D-2 `refund_id` comprometido), RFC-0006
  (el consumo publicado y su D-4), RFC-0007 (la cabeza v5, el rechazo por caminos, la banda).
