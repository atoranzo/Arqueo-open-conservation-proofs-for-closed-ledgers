# RFC-0023 — El reloj de la caducidad: el suelo de trece meses en latidos de cadencia firmada, con el plazo del emisor en entradas y condición doble

- **Estado:** PROPUESTO (§827) — el texto, sus decisiones y sus etapas, redactados desde la 12 del
  `BACKLOG.md`, cuya nota del §861 pospone a este RFC el suelo `Δ_min = 13 meses`
  (`PRINCIPIOS.md` §4.3, con su ❌) y la (e) de T3b, y escritos sobre lo medido en el árbol del
  sello. Lo que esa nota ya fijó —la unidad, la condición doble, `born` en latidos y atestiguado,
  la T de la vía v1 de su nacimiento, con la D5, y el suelo para todo lo vivo, con la reversión
  por el cable cerrada hasta que exista— se escribe aquí y no se vuelve a razonar: las decisiones
  1, 2, 6 y 7. Lo que esa nota pide a este texto —la cadencia firmada, la evidencia de acelerarla y
  el caso sin latido—, desde qué latido cuenta el suelo lo nacido antes, el corte en que viaja, la
  forma de la hoja de meta y de dónde sale la cuenta del juez se toman aquí por delegación del autor
  («resuelvelas aplicando los principios»), con el criterio del §609 y el precedente del §642;
  REVERSIBLES; entran con su aceptación: las decisiones 3, 4, 5, 9, 10, 12 y 13, y la D-J para el
  suelo. La 10 lleva un coste que la nota de la 12 no justificó, y se declara (D-H). Dos quedan
  abiertas para el autor, escritas como pregunta: el Δ en latidos para los pendientes nuevos (la
  8), que enmendaría la D1 del RFC-0003, ACEPTADO; y atestiguar ante un tercero el `Δ` abierto (la
  11), la parte de la (e) de T3b que la marca del §861 en el RFC-0003 deja a este RFC. Ninguna
  etapa de código construida: la E1 es este texto. El paso a ACEPTADO exige construirlas (regla 4
  del PROCESO) y la aceptación del autor, que no se delega.
- **Autor:** Ángel José Toranzo Portela
- **Asistencia GenAI:** Claude, en una sesión de Claude Code en la nube (§827): redactó el texto
  sobre lo que midieron la decisión de la 12 y su revisión, y lo portó sobre la nota del §861,
  fuera del paso 4 de `GENAI.md` — ver [`GENAI.md`](../../GENAI.md)
- **Fecha:** 2026-10-21 (redactado el 2026-10-15, sobre `d4d6d45`, el §814)
- **Versión del protocolo afectada:** `zkssl/0.4` -> `zkssl/0.5`, con la E3 y la E4: cambia lo que
  el nodo firma —la composición de `params_digest` y la hoja de meta bajo `pmeta_root`—, que es
  una ruptura R1 del RFC-0021. Viaja en el corte del RFC-0018 si llega construido; si no, en el
  corte siguiente (decisión 9). La E1 y la E2 no cambian nada que viaje.
- **Asiento(s) de AUDITORIA:** §115 y §121 (la cadencia de un minuto), §119 (la política: el reloj
  del §119.3, el poder compuesto del §119.5, el §119.6, sin retroactividad, T3b en el §119.7 y el
  suelo del §119.8), §128 (T3b, partido), §174 (el reloj que ni el operador ni el emisor
  controlan), §178 (la caducidad), §229 (el techo de 248 op/s), §268 (las cabezas firmadas, con
  `seq`), §285 (quien firma lleva diario), §292 (`mmr_t`, desde la v3), §340 (el reloj atado a la
  cabeza), §350 (la D-4), §354 (el delta 96 de la conformidad), §388 (`born` bajo una raíz), §451
  y §452 (`params_digest` y `pmeta_root` en la cabeza v5), §501 (el «nunca» reducido), §609 (el
  criterio), §642 (el precedente), §779 (el suelo, en `PRINCIPIOS.md` con ❌), §809 (el bloque
  `superficie`, al día), §815 (en PARADA el latido no compone ni firma), §861 (la nota de la 12,
  que fija las decisiones 1, 2, 6 y 7, y la marca del RFC-0003 sobre el `Δ`, que deja la (e) a este
  RFC) y §827 (este RFC).
- **Backlog:** la **12** (fondos muertos), que este RFC no cierra: le da el suelo, y su E3 es lo que
  la 12 pide para darlo por hecho; con la **117** (el juez único, que toca las mismas llamadas).

## Estado de las etapas

| etapa | qué entrega | ¿rompe el cable?, y qué superficie | estado |
|---|---|---|---|
| E1 — el reloj, escrito | este texto: los dos relojes con su nombre, el suelo, la cadencia firmada, lo que pasa sin latido, a quién alcanza, la cuenta del juez y la (e) de T3b para el suelo | no | **escrita (§827), con sus decisiones tomadas por delegación**, salvo la 8 y la 11, del autor |
| E2 — el núcleo, apagado | dos hojas de meta nuevas, la del v2 y la del v1 —`sender`, `born` en entradas y `born` en latidos, y en la del v1 también su T—, cada una con su dominio (D-D); `params_digest` con la cadencia como octavo parámetro, con su codificación y su rango (D-C); el año y el suelo en segundos como constantes con un productor (D-B); sus KAT y la segunda implementación que los reproduce; y el bloque `superficie` de `spec/RPC.md` §«Versionado y compatibilidad», al día, como hizo el §809. Nadie los consume todavía | no: aditivo (R2), rutas nuevas en `spec/vectors/nucleo/` (S5) | sin construir |
| E3 — el juez y el nodo | la capa juzga con la condición doble en `apply_refund` y `apply_deissue`; anota la meta nueva al nacer; recibe su cuenta de latidos de una cabeza firmada, con los invariantes de la D-K; el nodo late a la cadencia del libro; con los tests de la E3, abajo | **sí**: S3, lo firmado (`params_digest` y la hoja de meta) | sin construir |
| E4 — el corte | las cabezas que firman la meta nueva y el `params_digest` nuevo; los vectores que cambian de veredicto o de texto, mudados a `spec/vectors/0.4/` (R6 b); el aviso de la R5 | **sí**: el de la E3, pagado una vez (R4) | sin construir; en el corte `zkssl/0.5` si llega, o en el siguiente |
| E5 — la aceleración, medida | un banco que compara los latidos contados desde una cabeza de partida con fecha acotada por abajo con la marca de tiempo de la cofirma `0x06` de un ancla posterior (RFC-0012, RFC-0013); no dos anclas entre sí (D-F) | no | sin construir; de otra sesión, y espera a un medio con testigos ajenos y a una cota inferior de cuándo se compone una cabeza |

## Motivación

El §119.8 escribió la línea: **«Δ_min = 13 meses.** Ningun pago puede llevar un plazo de reversion
menor», con el precedente del *recall* de SEPA y la analogía imperfecta declarada, que argumenta un
suelo «largo, nunca corto»: «Un Δ≈0 seria un pago que nunca fue promesa» (literal, sin tildes, como
el registro). El §779 la llevó a
`PRINCIPIOS.md` §4.3 marcada ❌, y sigue así: el árbol no tiene suelo. La nota del §861 en la 12
dice por qué, medido, en su (3); aquí va lo que este RFC necesita de ello, remedido en el árbol del
sello.

1. **El juez cuenta entradas, no tiempo.** `apply_refund` y `apply_deissue` toman
   `now = self.log.len()` (`crates/zk-ssl/src/two_phase.rs:676` y `:441`) y comparan
   `now - born` con el plazo: el Δ del sobre en un v2 (`:687-697` y `:452-462`), el `refund_ttl`
   global en un v1 (`:679-683` y `:444-448`). Ningún mínimo:
   `un_refund_v2_tras_delta_devuelve_al_emisor` (`two_phase.rs:3774`) reembolsa con `delta = 1`.
   Es el reloj que el §119.3 descartó: «`seq` cuenta operaciones, no tiempo. En un sistema quieto,
   un mes son cero entradas».
2. **Ningún número de entradas es un calendario.** Trece meses a 248 op/s, el techo del nodo por
   RPC (§229), son unos 8,5·10⁹ entradas (DEDUCIDO), que en un libro quieto no vencen nunca; y
   cualquier cifra menor se agota en horas con carga.
3. **El latido ya es la unidad que el §119.3 eligió, y el árbol ya la cuenta firmada.** El nodo
   emite una cabeza por latido aunque no haya operaciones (`arrancar`,
   `crates/zk-ssl-node/src/latido.rs:413-443`; cada 60 s por defecto, `:72`), y cada cabeza firma
   `mmr_t`, cuántas cabezas anteriores acumula su MMR (`crates/zk-ssl/src/log.rs:642`), desde el
   formato v3 (§292); la hoja entra al conservar la cabeza (`latido.rs:472-474`). Desde el §815, en
   PARADA el latido no compone ni firma (`latido.rs:152-154`). **Pero la cuenta vive en el nodo, no
   en el juez**: la capa compara `log.len()`.
4. **La cadencia no va firmada.** Es una opción del operador, `--latido`, y `0` apaga el latido
   (`crates/zk-ssl-node/src/main.rs:234-239`). `EpochHead` no tiene campo de cadencia ni de hora
   (`log.rs:613-680`), y `params_digest` compone los siete parámetros en reposo sin ella
   (`crates/zk-ssl-hash/src/lib.rs:972`). Sin la cadencia, un número de latidos no dice cuánto
   tiempo es: 569.790 latidos son trece meses a 60 s y unos 6,6 días a 1 s (DEDUCIDO). El
   precedente del §642 (4), «un año de latidos a la cadencia configurada», es un umbral de aviso,
   donde acelerar solo perjudica al propio operador; aquí se protege al receptor.
5. **La capa usada como biblioteca, sin nodo, no tiene latido**, y con `--latido 0` el nodo
   tampoco. Y el `mmr_t` que la cabeza firma no lo cuenta la capa: es un parámetro que el llamador
   pasa a `epoch_head` (`crates/zk-ssl/src/lib.rs:870-877`), y hay llamadores que pasan 0, el
   resumen del sandbox (`crates/zk-ssl-cli/src/sandbox.rs:329-333`) y la conformidad
   (`crates/zk-ssl-cli/src/conformance.rs:139-143`). El nodo firma la cabeza después de componerla
   y fuera de la capa (`latido.rs:227-229`); sin `--clave` la compone sin firmar y aun así mete su
   hoja en el MMR (`latido.rs:435` y `:472-474`); y sin diario su `t` vuelve a 0 al reiniciar
   (`crates/zk-ssl-node/src/main.rs:456-460`), aunque un nodo con `--clave` no arranca sin diario
   (§285).
6. **«Latido» se usa también por entrada.** `DEFAULT_REFUND_TTL` se documenta «en latidos de
   `log.seq`» (`crates/zk-ssl/src/lib.rs:229-231`), y la (i) de la 12 dice que «T cuenta latidos de
   `log.seq`»: son entradas del registro, no cabezas del latido.
7. **El `Δ` abierto no lo ve nadie.** La marca del §861 en el RFC-0003, bajo «El invariante de
   reembolso, completo», corrigió que la segunda cláusula, `now − born ≥ Δ`, la verifique
   «cualquiera»: el `Δ` no se publica, la apertura vive solo en el `RefundReceipt`, que ningún
   método del cable transporta, y un operador que compuso el envío conoce `(refund_id, delta)`. El
   autor lo clasificó como límite del modelo de confianza, no como fallo; atestiguarlo es la (e) de
   T3b, y la marca la deja a este RFC (D-J).

## Diseño

### D-A — Dos relojes, cada uno con su nombre

- **Entrada**, o época del registro: `seq`, `self.log.len()`. La firma la cabeza desde el §268, y
  el §340 lo comprobó y lo ató con un test (`la_altura_entra_en_el_digest`, `log.rs:795`). En
  entradas se cuentan el Δ del emisor (RFC-0003 §«El compromiso»: «`Δ` = plazo en alturas de
  época», con el sentido de su D1) y la T de la vía v1.
- **Latido**: una cabeza del latido. Lo cuenta `mmr_t`, que la cabeza firma desde la v3. En
  latidos se cuenta el suelo.
- Donde el árbol dice «latido» por entrada se corrige con marca, sin borrar, en las dos que nombra
  la nota del §861: la (i) de la 12 y el comentario de `DEFAULT_REFUND_TTL`
  (`crates/zk-ssl/src/lib.rs:229`), las dos en el sello de este RFC (§827) y en su misma línea, como
  el §861 marcó comentarios de `.rs` en un sello de documentos.

### D-B — El suelo, en segundos del calendario y contado en latidos de cadencia firmada

- **El suelo son trece meses de un año de 365,25 días**: 13/12 de 31.557.600 s, que son
  34.187.400 s; los 31.557.600 son el año del aviso de agotamiento del nodo (`SEGUNDOS_ANUALES`,
  `main.rs:915`, la D-F del RFC-0015). El año y el suelo, constantes con un productor: la E2 los
  pone en el núcleo con su KAT, y la E3 hace que el nodo derive de ellos su `SEGUNDOS_ANUALES`, que
  hoy es una constante local de `aviso_de_agotamiento`; si no, el núcleo sería un segundo productor
  del mismo año.
- **En latidos**, el suelo es `⌈34.187.400 / c⌉`, con `c` la cadencia firmada del libro, en
  segundos: a 60 s, 569.790 latidos (DEDUCIDO).
- **La condición doble**, que fija la nota de la 12 en su (3): un reembolso o una des-emisión se
  aceptan solo si se cumplen las dos, `now - born >= plazo`, en entradas —el Δ del sobre en un v2,
  la T en un v1—, **y** `latidos_now - latidos_born >= suelo`, en latidos. La primera es la de hoy,
  intacta; la segunda solo restringe: por debajo del suelo, `expiry = born + Δ` deja de ser el
  momento en que se revierte, y se revierte más tarde.
- **La des-emisión**, por analogía, que la nota de la 12 ya declara en su (2) para los congelados y
  aquí se declara igual: un pendiente de emisión que caduca deja de existir para el receptor igual
  que uno de pago vuelve al emisor. Es extensión, no letra del §119.8.
- **El «nunca» de verdad** —que un Δ de la clase del centinela no venza— es la pieza (1) de la nota
  de la 12, que va en la capa sin formato; el suelo no lo toca.

### D-C — La cadencia, firmada: el octavo parámetro de `params_digest`, fijo en la vida del libro

- La cadencia del latido pasa a ser **un parámetro del libro**, el octavo que compone
  `params_digest` (`crates/zk-ssl-hash/src/lib.rs:972`), que la cabeza firma desde la v5 (§451,
  §452). Es el sitio de las reglas: «Dos cabezas del mismo operador con `params_digest` distintos
  dicen que las reglas se movieron» (su documentación, sobre la D-B del RFC-0007).
- **Su escritura**: `c`, un `u64` en segundos. `c >= 1` es una cadencia. `c = 0` es el libro sin
  latido, y se firma igual: así un libro sin latido también tiene su parámetro en `params_digest`,
  y su suelo no vence (D-E). `⌈34.187.400 / c⌉` solo se calcula con `c >= 1`.
- **Fija en la vida del libro**: el nodo late a la cadencia del libro y no arranca con un `--latido`
  distinto, tampoco con `--latido 0` en un libro de `c = 60` ni con `--latido 60` en uno de `c =
  0`. Cambiarla reinterpretaría los latidos ya contados: los que se contaron a 60 s valdrían otra
  cosa a 30 s (DEDUCIDO).
- **Su rango**: una `c` enorme deja el suelo en un solo latido —con `c >= 34.187.400`, `⌈34.187.400
  / c⌉ = 1`— y con él la evidencia de acelerar en fechar una sola cabeza (DEDUCIDO). La cota
  superior de `c` la fija la E2 con su KAT; este texto no fija el número.

### D-D — `born` en latidos, atestiguado

- **La hoja de meta del pendiente gana `born` en latidos** y, en un v1, su T (la D-G). Hoy es
  `commit_operation(DOMINIO_META_PENDIENTE, [sender, born])`, con dominio `PMETA_V1`
  (`crates/zk-ssl-hash/src/lib.rs:906-912`), y la de hoy se queda con el suyo. Como
  `commit_operation` supone longitud FIJA por dominio, sin relleno (en el mismo fichero, `:1753`, y
  lo repite la marca de `:947`), son **dos hojas nuevas, cada una con su dominio**: la del v2,
  `[sender, born, born_latidos]`, y la del v1, `[sender, born, born_latidos, T]` (la decisión 12).
  Sus nombres y valores los fija la E2 con su KAT. Van bajo `pmeta_root`, que se guarda y se
  comprueba al abrir desde el §388 y que la cabeza firma desde la v5: la lección del §388, que cita
  la nota de la 12.
- **El juez lee un número de latidos firmado**: con la cuenta de la D-K anota `born` en latidos al
  nacer el pendiente y juzga después. El test 4 de la E3 lo ata como `la_altura_entra_en_el_digest`
  ata `seq` (§340): el número que juzga es el de una cabeza cuya firma verifica.

### D-E — Sin latido, el suelo no vence

- En un libro de cadencia firmada `0` (D-C), que el nodo corre con `--latido 0`; en un nodo sin
  `--clave`, que compone sus cabezas sin firmar (hecho 5); en la capa usada como biblioteca, sin
  nodo; en PARADA, en la que desde el §815 el latido no compone ni firma (hecho 3); y mientras la
  capa no conoce una cuenta firmada (D-K), no hay calendario firmado: **el suelo no vence**, y el
  rechazo lo dice con su causa. Es el «largo, nunca corto» del §119.8 aplicado a quien no da
  reloj. Vale también para lo nacido antes (D-H). Su coste se declara: ahí, lo que el suelo
  alcanza no revierte mientras falte el reloj.
- Un reloj que ponga quien usa la biblioteca sería el que el §119.3 descartó: uno que no deja
  evidencia de su aceleración. Por eso la cuenta del juez no es el `mmr_t` que se pasa a
  `epoch_head` (D-K).

### D-F — Lo que deja acelerar el latido, y quién lo ve

- **Acelerar es latir más deprisa que la cadencia firmada.** Con la cadencia y `mmr_t` firmados, lo
  que falta para verlo es un reloj ajeno: dos lecturas fechadas por alguien que no sea el operador
  —una que acote por abajo cuándo se compuso una cabeza y otra que acote por arriba cuándo existía
  otra posterior—, y entre ellas más latidos de los que caben a esa cadencia. La nota de la 12 lo
  dice así: hoy solo el reloj de un testigo, y no hay ninguno.
- **El ancla ya lleva el número, y su fecha es una cota superior**: el ancla del RFC-0012
  compromete `mmrSize`, el mismo `mmr_t` (su D-A), y el medio del RFC-0013 la publica con cofirmas
  `0x06` con marca de tiempo de testigos ajenos (su D-C y su D-D). Esa marca dice que la cabeza ya
  existía cuando la vieron —«esta cabeza estaba publicada, en esta posición, cuando `k` testigos
  ajenos la vieron» (su D-E)—, no cuándo se compuso; y la publicación es asíncrona, un ancla cada
  `M` = 1.440 latidos, y «el nodo no espera a nadie» (su D-F).
- **Dos anclas comparadas entre sí no bastan.** La regla «dos anclas cofirmadas dan `(Δτ,
  Δmmr_t)`, y más latidos que `Δτ / c` es aceleración» no es sólida: (a) si un testigo cae o el
  publicador se retrasa, el ancla anterior de un nodo honrado se cofirma tarde, `Δτ` sale menor
  que el tiempo real y la regla da «aceleración», un ROJO fabricable contra un operador honrado, la
  clase que cierra el RFC-0019; (b) un operador que acelera y retiene las anclas de las cabezas
  aceleradas, para publicarlas después a su cadencia, no deja nada que se vea (DEDUCIDO las dos).
- **Lo que sí sería evidencia** pide dos cosas que hoy no hay. Una cota inferior de cuándo se
  compuso una cabeza de partida: con ella, una cuenta acumulada desde esa cabeza es sólida —si
  entre ella, compuesta después de `τ₀`, y una cabeza cofirmada en `τ` hay `N` latidos y `N · c >
  τ − τ₀`, el nodo latió deprisa, y un medio que se retrasa solo agranda `τ`, así que no fabrica
  ROJO (DEDUCIDO)—. Y alguien distinto del operador que feche las cabezas a tiempo, sin el que el
  caso (b) no se ve. Qué da esa cota inferior no lo fija este RFC. El §174 quería un reloj «que ni
  el operador ni el emisor controlan»: el ancla da la cota superior, fechada por otros, y la
  inferior está por hacer.
- **Hoy no lo ve nadie, y no hay con qué**: no hay un solo testigo (`PRINCIPIOS.md` §0), ni un
  medio con testigos ajenos desplegado, ni una cabeza que lleve una cota inferior de su
  composición. Acelerar el latido no deja hoy evidencia, y se declara. Construir la comparación es
  la E5, de otra sesión, sobre la cuenta acumulada y no sobre dos anclas entre sí.

### D-G — La T de la vía v1, la de su nacimiento

- Lo fija la nota de la 12 en su (3) y su (4): un v1 que nazca con este RFC lleva en su meta la T
  con que nació, y el juez usa ésa; el `refund_ttl` vigente decide solo la T de lo que nace. Hoy un
  cambio de `refund_ttl` mueve también la T de lo ya enviado (`set_refund_ttl`, `two_phase.rs:257`,
  que hoy solo llaman tests); la D5, aceptada en esa nota y sin construir, lo lleva a `cfg(test)`, y
  la T se revisa por release, como el §178 la quería, «se declara, se publica y se revisa con
  datos» (`crates/zk-ssl/src/lib.rs:230`). Lo nacido antes, sin T en su meta, se juzga con la
  vigente, como hoy.
- Si un corte deja de producir v1 —la nota de la 12 lo lleva al RFC que abra la reversión por el
  cable o al corte de formato, el primero que llegue—, esta decisión se queda sin a quién aplicarse
  hacia delante, y escrita.

### D-H — Para todo lo vivo, y lo nacido antes, desde la cabeza que estrena la cadencia

- **El suelo alcanza a todo lo vivo**, como fija la nota de la 12 en su (3), con su razón, que aquí
  no se repite: un v2 con Δ menor que el suelo —la conformidad usa 96 (RFC-0003, la nota del
  §354)— y todo v1 con meta revierten más tarde de lo que decían su Δ o su T. Lo nacido sin meta no
  cambia: no revierte nunca («el legado sin meta es inmune para siempre», la 12).
- **Lo nacido antes no tiene `born` en latidos, y no se le inventa.** Su suelo cuenta desde el
  `mmr_t` de la primera cabeza que firma el `params_digest` con la cadencia (D-C): la que estrena
  las reglas, firmada. Cualquiera con las cabezas la encuentra por la versión de cabeza o de
  composición que fije el corte (Compatibilidad), no por ser la primera con un `params_digest`
  distinto: ese digest cambia también con `refund_ttl`, uno de sus siete parámetros, que la D-G
  revisa por release, y con la gobernanza (`crates/zk-ssl-hash/src/lib.rs:953-968`). Nunca da un
  suelo más corto que contarlo desde el nacimiento: ningún pendiente vivo revierte antes de que
  pase un suelo entero desde que el suelo existe (DEDUCIDO). El juez guarda ese número una vez, al
  recibir esa cabeza firmada (D-K); cómo se persiste lo fija la E3. Sin latido no hay esa cabeza,
  y el suelo no vence (D-E).
- **Lo que eso cuesta, y la nota de la 12 no justificó.** La nota da el suelo para todo lo vivo
  porque el de trece meses era política publicada desde el §119.8: «No es cambiar términos a
  posteriori». Contar desde esa cabeza retrasa también lo que esa razón no cubre: lo que vencía
  más de trece meses después de nacer y menos de trece meses después de esa cabeza, y lo que ya
  era reversible al llegar ella. Un v2 nacido diez meses antes de esa cabeza, con un Δ en entradas
  que vence a los veinte meses de nacer, revierte a los veinte contado desde su nacimiento y a los
  veintitrés con esta decisión; uno ya vencido y no reembolsado al llegar la cabeza espera un suelo
  entero desde ella (DEDUCIDO). Ese retraso, por encima del suelo contado desde el nacimiento, no es
  política publicada, y toca el §119.6, sin retroactividad, que la nota invoca. Se declara como
  coste de la decisión 10, y su reapertura lo nombra. Contar desde la cabeza que cubrió cada
  nacimiento lo evitaría, pero contaría latidos de antes de que la cadencia fuera firmada, que un
  operador pudo dar más deprisa sin que nada lo dijera (hecho 4): daría un suelo más corto, que el
  §119.8 no admite.
- **La reversión por el cable no se abre sin el suelo**, como fija la nota de la 12: el reembolso
  por RPC va después, con su RFC (ARQ-02, `doc/blueprint-v2.md` §«3.4 Arquitectura y API»; no hay
  método de reembolso en `spec/RPC.md` §«Pago en dos fases»), y ese RFC no se acepta antes de que
  la E3 de éste esté construida.
- La otra forma de contar lo nacido antes, desde la cabeza que cubrió su nacimiento, se descarta
  (la decisión 10).

### D-I — El Δ en entradas, con su defecto declarado

- Con la condición doble, el Δ del emisor sigue en entradas, y **el defecto del §119.3 sigue en pie
  para el Δ**, como lo declara la nota de la 12: en un libro lento, un Δ en entradas puede no vencer
  nunca, y el emisor controla el calendario solo a través del suelo. «Funciona a cualquier ritmo»
  vale para el suelo, no para el Δ.
- Pasar el Δ a latidos para los pendientes nuevos enmendaría la D1 del RFC-0003, ACEPTADO: es la
  pregunta 8, del autor.

### D-J — La (e) de T3b: el input temporal, atado a una cabeza firmada

- **Qué pide.** La (e) del §119.7: «el input temporal atado a una cabeza firmada». La nota de la
  12 la deja a este RFC (su (6): «(e): espera al RFC de (3)»), y la marca del §861 en el RFC-0003
  dice que atestiguar el `Δ` abierto es la (e) y que la resuelve este RFC (hecho 7).
- **Para el suelo, tomada.** Los dos números del suelo son latidos de cabezas firmadas:
  `latidos_now` es la cuenta del juez, que solo mueve una cabeza firmada (D-K), y `latidos_born`,
  el que anota la hoja de meta bajo `pmeta_root`, que la cabeza firma desde la v5; para lo nacido
  antes, el `mmr_t` de la cabeza que estrena la cadencia (D-H). El test 4 de la E3 ata el número
  que juzga al de una cabeza cuya firma verifica, en el nodo, porque la firma ocurre fuera de la
  capa (hecho 5). Es la (e) en el sentido en que el §340 la cumplió para `seq`: el número del juez
  es el firmado.
- **Lo que eso no da**: que un tercero pruebe que un reembolso concreto se aplicó antes del suelo o
  del `Δ`. De un `Refund`, el registro guarda el digest de su prueba (la tabla «Lo que asienta cada
  clase» de `spec/RPC.md`), y la apertura y el `born` se quedan en la capa. La prueba del pago en
  curso del RFC-0008 (E2) cuenta en entradas, `nacido + delta >= T`, y no dice nada de latidos: del
  suelo no muestra nada. Atestiguarlos ante un tercero es la pregunta 11, del autor. Este RFC no lo
  construye ni lo descarta.

### D-K — La cuenta del juez: de dónde sale, y lo que no puede hacer

- **No es el parámetro de `epoch_head`.** El `mmr_t` que firma la cabeza lo pasa quien la compone,
  y hay llamadores que pasan 0 (hecho 5). Si el juez contara con ese parámetro, el reloj lo pondría
  cualquiera que use la biblioteca, que es lo que la D-E y la decisión 5 descartan. La cuenta del
  juez, `latidos_now`, entra por otra puerta: el nodo se la da a la capa **después de firmar** la
  cabeza, con la cabeza firmada. Qué función es y qué comprueba la capa de esa firma lo fija la
  E3.
- **Solo la mueve una cabeza firmada, bajo una cadencia firmada `c >= 1`.** Las cabezas sin firmar
  de un nodo sin `--clave` no la mueven, aunque su hoja entre en el MMR; un libro de `c = 0`, ni
  la tiene.
- **Los invariantes, que son del protocolo**:
  1. la cuenta nunca retrocede: una cabeza con un `mmr_t` menor que la última cuenta conocida no
     la mueve;
  2. `born` en latidos nunca queda por debajo de la última cuenta firmada que la capa conoce al
     nacer el pendiente;
  3. sin cuenta conocida —tras arrancar y antes de la primera cabeza firmada, si no se sembró; en
     la capa sin nodo; con `c = 0`— el suelo no vence (D-E);
  4. restar satura: con `latidos_now` menor que `latidos_born`, lo transcurrido es 0.
- **Lo que nace sin cuenta conocida no anota 0**: lo haría vencer enseguida, que es el daño que el
  suelo evita. Cómo lo anota —con la última cuenta que la capa conserve junto al lote, o con una
  marca de «desconocido» que el juez cuenta desde la primera cuenta firmada que llegue, como lo
  nacido antes (D-H)— lo fija la E3, con un test, dentro de los invariantes. Si la cuenta se
  persiste con el lote o se siembra del diario al arrancar, también.

## Los tests de la E3

Uno discriminante por propiedad (`PRINCIPIOS.md` §9), en `apply_refund` y en `apply_deissue`:

1. **El latido, solo**: con el plazo en entradas ya cumplido y sin operaciones, un reembolso pasa
   de `RefundTooEarly` a `Ok` cuando pasan los latidos del suelo, y no antes.
2. **El suelo manda sobre un Δ corto**: con las entradas de sobra y los latidos por debajo del
   suelo, `RefundTooEarly`; al llegar al suelo, `Ok`.
3. **Sin latido**, en un libro de `c = 0`, en PARADA, en un nodo sin `--clave` y en la capa sin
   nodo, el suelo no vence; tras arrancar sin cuenta conocida, tampoco, y lo que nace en esa
   ventana no anota 0 (D-K).
4. **El número que juzga es el que se firma**, en el nodo: la cuenta con que la capa juzga es el
   `mmr_t` de una cabeza cuya firma verifica; una cabeza sin firmar no la mueve, ni una con un
   `mmr_t` menor; y una cabeza a la que se cambia solo `mmr_t` cambia su digest y su firma deja de
   verificar.
5. **La cadencia, firmada**: cambiarla cambia `params_digest`, y un nodo con un `--latido` distinto
   de la `c` del libro no arranca, también `--latido 0` con `c = 60` y `--latido 60` con `c = 0`.
6. **La T de la v1, la de su nacimiento**: un v1 juzgado tras cambiar el `refund_ttl` vigente sigue
   con la T con que nació.
7. **Para todo lo vivo**: un pendiente nacido con la meta de hoy no revierte hasta que pasan los
   latidos del suelo contados desde la cabeza que estrena la cadencia, aunque su Δ o su T ya se
   hayan cumplido; el mismo, nacido con la meta nueva, cuenta desde su `born` en latidos.

Los dos primeros son la condición con que la nota de la 12 da el suelo por cumplido.

## Compatibilidad

Con la E3 y la E4, el cable sube a `zkssl/0.5`: cambia lo que el nodo firma, `params_digest` y la
hoja de meta bajo `pmeta_root` (R1 del RFC-0021). Qué versión de cabeza lo anuncia la fija el
corte, junto a la v7 del RFC-0020: si van en el mismo, una sola versión nueva para las dos cosas
(DEDUCIDO). Las cabezas v2 a v6 y sus vectores no cambian un byte; lo que cambie de veredicto o de
texto se muda a `spec/vectors/0.4/` (R6 b). La E1 y la E2 no cambian nada que viaje. Este RFC
está PROPUESTO al menos un sello antes del corte, como pide la R5 y como pide la nota de la 12.

## Decisiones (por delegación del autor, §827), y dos preguntas para él

Las 1, 2, 6 y 7 las fijó la nota del §861 en la 12, y aquí se escriben con su sitio en el diseño,
sin volver a razonarlas; las 3, 4, 5, 9, 10, 12 y 13 se toman aquí, con su razón y lo descartado
—las 3, 4 y 5 las pide esa nota a este texto, la 10 la pide la 6, y la 12 y la 13 las pide el
diseño de la D-D y de la D-K—; la 8 y la 11 son preguntas para el autor, abiertas.

1. **¿En qué unidad cuenta el suelo?** **Fijada por la nota de la 12 (§861, su (3)): en latidos**
   (D-B), ni retirado ni en entradas.
2. **¿Un reloj o dos?** **Fijada por la nota de la 12: la condición doble** (D-B), con el defecto
   del §119.3 declarado para el Δ (D-I). Todo a latidos es la pregunta 8.
3. **¿Dónde se firma la cadencia?** En `params_digest`, como octavo parámetro y fija en la vida del
   libro (recomendada: es el sitio de las reglas, y ya va firmado); en un campo propio de la
   cabeza; o sin firmar. **Tomada: en `params_digest`, fija**, un `u64` en segundos, `0` para el
   libro sin latido y con la cota superior que fije la E2 (D-C). Descartadas: un campo propio,
   que pagaría otra versión de cabeza por lo que los parámetros ya agrupan; sin firmar, porque el
   suelo en latidos no diría cuánto tiempo es (hecho 4); y una cadencia que cambia, que pediría un
   reloj de segundos declarados —la suma de la cadencia de cada latido— y otro campo firmado.
4. **¿Qué evidencia deja acelerarla?** La del ancla cofirmada con marca de tiempo, con una cuenta
   acumulada desde una cabeza de partida con fecha acotada por abajo (recomendada: el ancla ya lleva
   el número, la fecha la ponen otros, y la cuenta acumulada no da ROJO cuando el medio se
   retrasa); dos anclas comparadas entre sí; o un campo de hora en la cabeza, que fecha el operador.
   **Tomada: la del ancla, con la cuenta acumulada** (D-F), declarada hoy sin evidencia: faltan la
   cota inferior y quien feche las cabezas a tiempo. Descartados: dos anclas entre sí, que dan un
   ROJO fabricable contra el operador honrado cuando un testigo cae o el publicador se retrasa; y
   la hora del operador, que no es un reloj ajeno.
5. **¿Y sin latido?** El suelo no vence (recomendada: largo, nunca corto); o un reloj de quien usa
   la biblioteca. **Tomada: no vence** (D-E), también en PARADA, sin `--clave` y sin cuenta
   conocida. Descartado: el reloj sin firma.
6. **¿El suelo es retroactivo?** **Fijada por la nota de la 12: para todo lo vivo** (D-H), con la
   reversión por el cable cerrada hasta que el suelo exista. Desde qué latido cuenta lo nacido
   antes es la decisión 10.
7. **¿La T de la vía v1?** **Fijada por la nota de la 12: la de su nacimiento** (D-G), con su D5.
8. **¿El Δ en latidos para los pendientes nuevos?** Cerraría el defecto del §119.3 para el Δ
   (D-I), y enmendaría la D1 del RFC-0003, ACEPTADO, que cuenta el Δ «en alturas de época». La
   nota de la 12 la deja escrita para el autor. **Abierta, del autor**: aplicar no es enmendar, y
   la delegación no la toma.
9. **¿Viaja en el corte `zkssl/0.5`?** Sí, si la E2 y la E3 llegan construidas (recomendada: una
   ruptura se paga una vez, R4); si no, en el corte siguiente. **Tomada: así**, en lo que toca a
   este RFC. Si viaja en el tren lo acuerda quien lo lleva (RFC-0018), y queda por coordinar: la
   fila del corte del RFC-0018 no lo nombra todavía, ni la Compatibilidad del RFC-0021, que cuenta
   qué RFC del corte cumplen ya la R5. Descartado: un corte propio. ⚠️ §865: lo nombran sin acordar.
10. **¿Desde qué latido cuenta el suelo lo nacido antes?** Desde el `mmr_t` de la primera cabeza
    que firma la cadencia (recomendada: está firmado, es uno solo para todo el libro y nunca da un
    suelo más corto que contarlo desde el nacimiento); o desde la cabeza que cubrió el nacimiento
    de cada uno. **Tomada: desde la primera cabeza que firma la cadencia** (D-H), con su coste
    declarado: retrasa también lo que vencía por encima del suelo contado desde el nacimiento, y lo
    ya reversible al llegar esa cabeza, hasta un suelo entero desde ella, que la razón de la nota de
    la 12 no cubre y que toca el §119.6. Descartado: desde la cabeza de cada nacimiento, que pide al
    juez una cabeza vieja por pendiente, cuenta latidos de antes de que la cadencia fuera firmada,
    que pudieron ir más deprisa sin que nada lo dijera, y lo nacido antes de las cabezas v3 no
    tiene ninguna que firme `mmr_t` (§292).
11. **¿Se atestigua ante un tercero la apertura de un reembolso, su `Δ` y su `born`?** Es la parte
    de la (e) de T3b que la D-J no toma (hecho 7). Las ramas, sin recomendar ninguna:
    - **no**: queda el límite del modelo de confianza que el autor clasificó en el §861, declarado;
      el suelo restringe al juez honrado, y lo que deja oponible es la cadencia firmada y los
      latidos, con los que se vería acelerar el reloj cuando haya la cuenta acumulada y quien
      feche las cabezas (D-F), no un juez que se salta el plazo;
    - **la apertura en la entrada del registro del reembolso**, en claro o en su compromiso: con dos
      cabezas, cualquiera comprueba las dos condiciones. Cambia lo que el registro compromete (S3,
      al corte) y lo que la fila de la apertura del reembolso del RFC-0009 D-B deja en el recibo,
      fuera de la prueba; y en claro revelaría `f`, contra la D2 del RFC-0003, que hizo `X` opaco
      por su privacidad;
    - **`born` y `Δ` en las entradas públicas de `RefundAirV2`**: lo comprobaría el verificador de
      la prueba, y aparta la decisión del RFC-0003 de dejar `born` y `now` fuera del circuito,
      atestiguados; es de la zona de las AIR. Y sola no da a un tercero nada que comprobar: el
      registro guarda el resumen de la prueba y no la prueba, ningún método del cable la sirve y el
      kit no compila el circuito del reembolso (la 115 del `BACKLOG.md`), así que pide también una
      de las dos salidas de la 115.

    **Abierta, del autor**: la segunda y la tercera mueven lo que deciden dos RFC ACEPTADOS, el
    0009 y el 0003, y la primera da por definitivo un límite que el §861 dejó a este RFC; elegir
    entre ellas no es aplicar los principios, es fijarlos.
12. **¿Cómo lleva la hoja de meta la T de la v1?** Dos hojas, cada una con su dominio (recomendada:
    `commit_operation` supone longitud fija por dominio, sin relleno); o una sola, con un centinela
    de T en el v2. **Tomada: dos hojas, dos dominios** (D-D). Descartado: el centinela, un valor con
    dos sentidos, que es lo que la pieza (1) de la nota de la 12 arregla en el Δ.
13. **¿De dónde sale la cuenta del juez?** De una cabeza firmada, que el nodo le da a la capa
    después de firmarla, con los invariantes de la D-K (recomendada: el número del juez es el
    firmado, y sin cuenta el suelo no vence); o del `mmr_t` que se pasa a `epoch_head`. **Tomada: de
    la cabeza firmada** (D-K). Descartado: el parámetro de `epoch_head`, que pone quien compone, y
    que el sandbox y la conformidad ya pasan a 0.

Las decisiones tomadas, todas REVERSIBLES. **Cuándo se reabren**: la 3, si un operador necesita
cambiar la cadencia de un libro vivo; la 4, si el medio no llega a tener testigos ajenos, si sus
cofirmas pierden la marca de tiempo o si no aparece una cota inferior de cuándo se compone una
cabeza; la 5, si la capa sin nodo o un libro de cadencia 0 llegan a tener una cuenta firmada que no
ponga quien los usa; la 9, si el corte se cierra sin la E3, o si quien lleva el tren no lo admite;
la 10, si el juez llega a leer la cabeza de cada nacimiento y el autor prefiere contar desde ella,
o si el autor juzga el retraso que declara la D-H un cambio de términos a posteriori (§119.6); la
12, si `commit_operation` gana una regla de relleno (`crates/zk-ssl-hash/src/lib.rs:1753`); la 13,
si la capa llega a firmar sus propias cabezas; la D-J, si la respuesta a la 11 pone la apertura en
el registro o en el circuito; las 1, 2, 6 y 7, con la nota de la 12; y todas, si el autor revoca
cualquiera.

## Lo que se DESCARTÓ al medir

Lo descartado por la nota de la 12 —retirar el suelo, el suelo en entradas y el suelo solo para lo
que nazca después— y por la D-4 del §350 —`refund_ttl` como techo del Δ v2, que la nota cita en su
(4)— no se repite. Aquí:

- **El suelo como política del SDK**: un pagador honrado no compondría un Δ corto, pero quien
  compone su propio sobre se lo salta, y el RFC-0003 pone el juez en la capa.
- **Contar lo nacido antes desde su nacimiento** (la decisión 10).
- **Comparar dos anclas entre sí** (la decisión 4): un medio que se retrasa da ROJO a un operador
  honrado (D-F).
- **Juzgar con el `mmr_t` que se pasa a `epoch_head`** (la decisión 13): el reloj lo pondría quien
  compone (D-K).

## Seguridad

- **La clave de gasto no viaja jamás**: este RFC no la toca. Cambia cuándo vence un plazo, no quién
  firma.
- **El suelo solo restringe**: ningún reembolso que hoy se rechace pasa a aceptarse. El destino no
  cambia: sigue siendo el que el sobre compromete en un v2 y el emisor de la meta en un v1.
- **La cadencia, firmada y fija**, quita al operador el poder de acortar el suelo cambiando
  `--latido` sin que una cabeza lo diga. Acelerar el latido por debajo de la cadencia firmada no
  deja hoy evidencia: la daría una cuenta acumulada desde una cabeza con fecha acotada por abajo
  frente a la marca de una cofirma, y contra quien retiene sus anclas, solo alguien que feche las
  cabezas a tiempo (D-F); no hay ninguna de las dos. Dos anclas comparadas entre sí no la dan, y
  fabricarían un ROJO contra el operador honrado.
- **La cuenta del juez** la mueven solo cabezas firmadas, no retrocede, y sin ella el suelo no
  vence (D-K): quien usa la biblioteca, o un nodo sin `--clave`, no adelanta el suelo pasando otro
  `mmr_t`.
- **Lo nacido antes** queda bajo el suelo, contado desde la primera cabeza que firma la cadencia
  (D-H): ningún plazo vivo vence antes de un suelo entero desde que el suelo existe. Y algunos
  vencen más tarde de lo que pediría el suelo contado desde su nacimiento, hasta un suelo entero
  desde esa cabeza, que es el coste declarado de la decisión 10. La reversión por el cable no se
  abre sin el suelo.
- **Lo que el suelo no da**: que un tercero pruebe que un juez se lo saltó. El operador que corre
  el juez puede adelantar un reembolso sin evidencia oponible, como hoy con el `Δ` (la marca del
  §861 en el RFC-0003): el dinero vuelve al emisor, que es el «acelerar devoluciones, nunca
  desviarlas» de `doc/CADUCIDAD_PENDIENTE.md`. Es la pregunta 11.

## Lo que este RFC no hace

- No toca código: el núcleo es la E2, el juez y el nodo son la E3. Su sello solo marca, en su
  misma línea, el comentario de `DEFAULT_REFUND_TTL` (D-A).
- No mete el reloj en un circuito: el RFC-0003 deja `born` y `now` fuera, atestiguados, que es la
  rama «condicionada, no caída» del §119.7. La cláusula (c) de T3b en circuito es de la sesión de
  las AIR.
- No construye el juez único: la E3 toca las llamadas a `verify::<…>` de `apply_refund` y
  `apply_deissue` (la 117), y se ordena con él.
- No decide el «nunca» de verdad, ni el receptor congelado, ni dejar de producir v1: son las
  piezas (1), (2) y (4) de la nota de la 12.
- No decide la 8 ni la 11.
- No construye el medio ni sus testigos (RFC-0013), ni la comparación de la E5, ni la cota inferior
  de cuándo se compone una cabeza.
- No acuerda con quien lleva el tren que viaje en el corte `zkssl/0.5` (la decisión 9).

## Referencias

- `BACKLOG.md`, entradas 12, 115 y 117.
- `AUDITORIA.md` §115, §119, §121, §128, §174, §178, §229, §268, §285, §292, §340, §350, §354,
  §388, §451, §452, §501, §609, §642, §779, §809, §815 y §861.
- `PRINCIPIOS.md` §0, §4.3 y §9; `doc/CADUCIDAD_PENDIENTE.md` §2 y §6; `doc/blueprint-v2.md`
  §«3.4 Arquitectura y API» (ARQ-01, ARQ-02).
- RFC-0003 (§«El compromiso», §«El invariante de reembolso, completo», con la marca del §861, D1,
  D2), RFC-0007 (D-B), RFC-0008 (la prueba del pago en curso), RFC-0009 (D-B), RFC-0012 (D-A),
  RFC-0013 (D-C, D-D, D-E, D-F), RFC-0015 (D-F), RFC-0018 (el corte), RFC-0019, RFC-0020 (la v7) y
  RFC-0021 (D-B y su Compatibilidad).
