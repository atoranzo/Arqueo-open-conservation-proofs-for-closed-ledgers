# RFC-0014 — El recibo del lote y de la prenda: lo agregado, atado como lo directo

- **Estado:** PROPUESTO (§608), con sus decisiones TOMADAS en el §609 -delegadas por el autor con
  su criterio escrito, y REVERSIBLES, ver «Decisiones»-. Medido y diseñado; la E2 construida en el
  §610, la E3 en el §611, la E4 en el §612 (el lote) y el §613 (la prenda), y la E5 en el §614:
  todas las etapas, construidas.
- **Autor:** Ángel José Toranzo Portela
- **Asistencia GenAI:** Claude (sesión 193, §608 a §614) — ver [`GENAI.md`](../../GENAI.md)
- **Fecha:** 2026-09-30
- **Versión del protocolo afectada:** `zkssl/0.4` — **no sube**: todo es aditivo, como el recibo
  del §571 -dos respuestas y un error ganan `recepcion`, el error del lote gana `operacion`, y el
  sobre de completitud gana dos resoluciones-. La cabeza no se mueve.
- **Asiento(s) de AUDITORIA:** §608 (la propuesta); §609 (las decisiones); §610 (la E2); §611 (la
  E3); §612 (la E4a); §613 (la E4b); §614 (la E5).
- **Backlog:** la segunda parte del D-H del RFC-0010 -«una etapa por diseñar, no un olvido»-; y
  `SECURITY.md` 2.ter, el despliegue con agregadores.

## Motivación

El RFC-0010 ata al operador con un recibo por cada operación que EVALÚA por las dos vías directas
del titular. En el §576 dejó fuera, por decisión y con su razón medida, lo que entra por
`zkssl_applyMany` y por `zkssl_pledge`: un recibo suyo no tendría hoy resolución, y el sobre de
completitud acusaría a un operador honrado. Es residuo declarado -la segunda parte de su D-H- y
un testigo del nodo lo ata. Y en el despliegue con agregadores (`SECURITY.md` 2.ter) eso es TODO
lo agregado: quien paga a través de un agregador no tiene hoy objeto que oponer.

Lo que se MIDIÓ en el §608, leyendo el código del nodo y de la capa:

| vía | qué evalúa | qué devuelve al aceptar | qué devuelve al rechazar | resolución hoy |
|---|---|---|---|---|
| `zkssl_applyMany` | valida TODAS las operaciones contra la misma foto y aplica sólo si pasan todas: la primera que falla aborta el lote entero (`apply_many`, `two_phase.rs`) | por entrada, `logSeq`, `kind`, `accountsRoot` y `chain`; sin `acuse`, pero `zkssl_ackPath` sirve el de cada entrada por su `logSeq` | un error de la capa con `{causa, campos, seq}` que **no dice qué operación falló**; y dos causas son del lote como tal, `DuplicateAccountInBatch` y `DuplicatePendingInBatch` | aplicado, **sí**, operación a operación; rechazado, **no**: ni para las compañeras ni para la que falló, porque no se nombra |
| `zkssl_pledge` | verifica la prueba de prenda contra la `pendingRoot` de la ÚLTIMA cabeza firmada -su `seq` tiene que coincidir- y asienta la marca con `apply_consumo` | `accepted: true`, `yaEstaba`, `logSeq`, `s` | CINCO respuestas blandas con `accepted: false`: sin latido, cabeza sin firmar, `seq` viejo, prueba que no verifica y error de `apply_consumo` | aceptada: la marca bajo `consRoot` (`zkssl_consumoPath`) y el sobre de prenda (`PAQUETE.md` 2.10), que reverifica la prueba -existen los dos, pero nada los ata a un recibo-; rechazada: ningún sobre |

## Diseño

**D-A. El lote recibe UN recibo, no uno por operación.** El nodo lo evalúa como una unidad -una
foto, un veredicto, todo o nada-, y la regla del D-E del RFC-0010 cuenta lo que el nodo llegó a
evaluar. La hoja es la de siempre, `recibo_digest(hashPrueba, era, n)`, con
`hashPrueba := hash_del_lote`, y `hash_del_lote` es la huella, con dominio propio
(`DOMINIO_LOTE`) y longitud codificada (§116), de la COMPOSICIÓN: `k` y, por cada operación en su
orden, `(hashPrueba_i, cuenta_i, posicion_i)`. La composición la conoce quien arma el lote; el
nodo no la devuelve, y queda firmada porque su huella está bajo la `recepRoot`. Nada nuevo bajo
la firma de la cabeza.

**D-B. La resolución del lote**, en el sobre de completitud, `resolucion.tipo = "lote"`, que
lleva la composición y el mando recompone `hash_del_lote` antes de nada:
- **aplicado**: un acuse por cada `hashPrueba` de la composición, cada uno bajo la `acusesRoot`
  de una cabeza dentro de la ventana, como el veredicto 1;
- **rechazado con prueba**: el sobre de rechazo del RFC-0007 de la operación `j` que el nodo
  nombra -el error del lote gana `operacion: j`, aditivo-, atado al recibo del lote por su
  `data.recepcion` y su `data.operacion`. Esa atadura es la PALABRA del nodo, como en el D3 del
  RFC-0010: el hash de una prueba no deja ver sus entradas públicas, y el mando no puede
  recomponerla. Las compañeras quedan resueltas por ella: el lote es la unidad;
- **rechazado por su forma**: `DuplicateAccountInBatch` y `DuplicatePendingInBatch` se prueban
  con la composición sola -dos entradas con la misma cuenta, o con la misma posición-. Y si la
  composición NO sostiene la causa que el nodo dio, el mando lo nombra, «RECHAZO SIN FUNDAMENTO»
  (§612): es la regla de la decisión 3 -donde el verificador puede repetir el juicio del operador,
  lo dice- en el otro sitio donde se puede;
- **lo demás**, las causas que el RFC-0007 dejó sin prueba portable: el cuarto estado,
  «declarada, no probada», como en el D-G del RFC-0010.

**D-C. El titular dentro de un agregador.** El recibo lo recibe quien llama: el agregador. Cada
titular del lote necesita la respuesta y la composición para armar su sobre; el agregador se las
reenvía, y el titular comprueba que su `hashPrueba` está en ella. El operador no puede negar haber
recibido ese lote: su huella está bajo su firma. El agregador que no reenvía queda fuera: lo elige
el titular, y la vía directa sigue abierta.

**D-D. La prenda recibe un recibo cuando se EVALÚA su prueba**, con `hashPrueba` el digest de la
prueba que llegó, el mismo que la casa usa para las demás (§116). No lo consumen las tres
respuestas que no llegan a evaluarla -sin latido, cabeza sin firmar, `seq` viejo-: son como el
ruido del cable del D-E del RFC-0010, y el titular vuelve a probar bajo la cabeza nueva.

**D-E. La resolución de la prenda**, `resolucion.tipo = "prenda"`, que lleva el sobre de prenda
(2.10) contra la cabeza del `seq` al que se ató, y el mando exige que su prueba tenga el
`hashPrueba` del recibo:
- **aceptada**: el sobre VERIFICA y la `marca` de su enunciado está bajo el `consRoot` de una
  cabeza dentro de la ventana -el camino de `zkssl_consumoPath`-. Es la prenda ENTERA, la que el
  2.10 llama «el PAR»;
- **rechazada con prueba**: el sobre NO verifica contra esa cabeza, y cualquiera lo comprueba sin
  el nodo. «Esa cabeza», precisada en el §613, es la que el nodo JUZGÓ: la última firmada al
  recibir, de índice `era − 1` (§567), que el sobre de completitud lleva como `juzgada` -el nodo
  compone el enunciado con la raíz de su cabeza, no con la del sobre-;
- ⚠️ **rechazada sin fundamento**: el sobre VERIFICA y el nodo dijo que no. El mando lo nombra
  -ROJO NOMBRADO, «RECHAZO SIN FUNDAMENTO»-, que es evidencia más fuerte que la no resolución:
  no hay que esperar a la ventana;
- **error de `apply_consumo`** distinto de `ConsumoRepetido`: su causa en `data`, como en el lote;
  desde el §613 ese `data` lleva su `recepcion`, y se resuelve con su sobre de rechazo (2.6).

**D-F. El testigo del §576 cae A PROPÓSITO**, en la etapa del nodo, como su propio texto anuncia:
«si una de las dos empieza a reservar, este testigo cae y la D-E se decide otra vez, con su
resolución». Se reescribe al revés: el lote y la prenda consumen recibo cuando evalúan, y las tres
respuestas previas de la prenda no.

**D-G. Lo que se añade, todo aditivo bajo `zkssl/0.4`:** `recepcion` en la respuesta y en el error
del lote, y en la respuesta de la prenda cuando evalúa; `operacion` en el error del lote; en el
REGISTRO de `NUCLEO.md`, `DOMINIO_LOTE` y `hash_del_lote`, con su KAT; y dos resoluciones en el
sobre de completitud (2.11), con sus vectores y su banco.

**D-H. El residuo, declarado.** El agregador que no reenvía; la prenda con `seq` viejo, que no
deja recibo y el titular reintenta -y un «seq viejo» FALSO es conducta visible, no evidencia
portable-; las causas sin prueba portable, en el cuarto estado; y lo de
siempre, el operador que no contesta (D-H del RFC-0010), que tampoco aquí deja rastro.

## Decisiones

Las tres que este texto deja al autor, con la que recomienda primero:

1. **Un recibo por lote (D-A)**, o uno por operación con la huella del lote dentro de cada hoja.
   Por operación daría a cada titular su propio `rx`, pero cambia la hoja del recibo -un tercer
   campo bajo la firma, un `recibo_digest` nuevo y sus vectores- para ganar lo que la composición
   ya da.
2. **Qué respuestas de la prenda consumen recibo (D-D)**: las que evalúan la prueba, o todas.
   Contar las previas abriría huecos en el registro a quien mande prendas contra una cabeza vieja.
3. **El rechazo de prenda cuyo sobre verifica (D-E)**: un veredicto con nombre, «RECHAZO SIN
   FUNDAMENTO», o dejarlo caer en «no resuelta en la ventana». El nombre no cuesta nada que la
   verificación no haga ya, y convierte veinticuatro horas de espera en evidencia inmediata.

**TOMADAS en el §609**, por delegación del autor y con el criterio que dictó: los principios del
proyecto y su portada -una capa base «neutral, minimalista y resistente» (`PRINCIPIOS.md` §3), la
evidencia que «un verificador independiente comprueba con el nodo apagado» y la «responsabilidad
demostrable, al modo de Certificate Transparency» (`README.md`)-, aplicables **al mayor número de
casos de uso sin modificaciones significativas**, porque el objetivo es fijar el estándar. Todas
REVERSIBLES.

- **1, el recibo va por LOTE (D-A).** Es la regla que ya hay -un recibo por cada cosa que el nodo
  EVALÚA (RFC-0010, D-E)- aplicada a lo que el nodo evalúa como una unidad: una foto, un
  veredicto, todo o nada. No toca nada existente: ni la hoja `recibo_digest(hash_prueba, era, n)`,
  ni `DOMINIO_RECEP`, ni la cabeza v6, ni un vector; sólo nace la huella de la composición, con su
  dominio y su KAT. Vale para cualquier agregador, con cualquier tamaño y mezcla de lote, y también
  para el lote de uno; el titular no cambia su prueba, y lo único que necesita -la respuesta y la
  composición- el agregador ya lo tiene. Uno por operación habría puesto un tercer campo bajo la
  firma -otro `recibo_digest`, otro KAT, otros vectores, el 2.11 cambiado para todas las vías- y
  prometido resolución a operaciones que el nodo NO evaluó, porque el lote aborta en la primera que
  falla. Una regla, una hoja, todas las vías: eso es lo que un estándar puede pedir a otra
  implementación.
- **2, consumen recibo sólo las respuestas que EVALÚAN la prueba (D-D).** Es la misma frontera del
  D-E -«ni el ruido»- y el mismo punto del §569: se reserva tras comprobar el `seq` y antes de
  verificar, como en la vía directa. Contar las tres previas abriría huecos en el registro ajeno a
  quien mande prendas contra una cabeza vieja, sin coste. El titular las ve al instante -la
  respuesta dice por qué- y vuelve a probar bajo la cabeza nueva. Residuo, declarado en D-H: un
  «seq viejo» falso es conducta visible, no evidencia portable, la misma clase que el operador que
  no contesta.
- **3, el rechazo de una prenda cuyo sobre verifica tiene nombre: «RECHAZO SIN FUNDAMENTO»
  (D-E).** No cuesta nada nuevo: el juez de la prenda ya está en el kit, sin el probador, y es EL
  MISMO con el que el nodo juzga (`verificar_contra_cabeza`, `PAQUETE.md` 2.10); el sobre de
  completitud ya delega en otros jueces, y aquí delega en uno más. Un operador honrado no puede
  producirlo: el recibo sólo existe si el `seq` casó, luego el nodo juzgó contra esa cabeza con ese
  juez. Y es la única vía donde hoy se puede: en la directa, la prueba del rechazo depende del
  estado (la grieta D-G del RFC-0010). Convierte una espera de `n` cabezas en evidencia inmediata,
  y fija la regla del estándar: donde el verificador puede repetir el juicio del operador, lo dice.

## Etapas

| etapa | qué entrega | ¿rompe el cable? | estado |
|---|---|---|---|
| E1 — la promesa, escrita | este texto | no | **propuesto (§608), decidido (§609)** |
| E2 — el núcleo | `DOMINIO_LOTE` y `hash_del_lote` en `zk-ssl-hash`, su KAT en `spec/vectors/nucleo/` y su fila del REGISTRO | no (aditivo) | **construida (§610)**: `DOMINIO_LOTE` en la familia bytes del REGISTRO de dominios, `hash_del_lote` con su fila en el censo de `NUCLEO.md` (RECIBOS), tres testigos, y el KAT `hash_del_lote.json`, el vigesimosexto, con los veinticinco anteriores byte a byte iguales; `k` va en la longitud codificada |
| E3 — el nodo | `zkssl_applyMany` y `zkssl_pledge` reservan y anotan su recibo en el mismo punto que la vía directa (§569); `operacion` en el error del lote; el testigo del §576, reescrito | no (aditivo) | **construida (§611)**: la capa gana `apply_many_con_operacion` -el mismo juicio, y el error con el índice- y `apply_many` es ella sin el índice; el lote reserva tras el parseo y antes de la capa, sobre `hash_del_lote` de su composición, re-exportada por el kit (`zk_ssl_verify::recibos`); la prenda, tras comprobar el `seq` y antes del juez; el testigo del §576 cae y se reescribe al revés, con las tres previas asertadas sin recibo. ⚠️ Hasta la E4 el recibo existe y su resolución no: el mando no sabe aún resolver el de un lote aplicado ni el de una prenda, y no se le lleva ninguno |
| E4a — el mando, el lote | `resolucion.tipo` «lote» en el sobre de completitud: la composición recompuesta, y aplicado, rechazado con prueba, por su forma -o sin fundamento- y declarado | no | **construida (§612)**: la huella recompuesta con la función del kit antes de nada; `acuses` uno por operación resueltos como el veredicto 1, `sobre` como el 2 con su `operacion` dentro del lote, y `data` con la forma juzgada otra vez o el cuarto estado; los dos veredictos de la vía directa salen a funciones que el lote reusa, con sus textos intactos y los 36 vectores iguales |
| E4b — el mando, la prenda | `resolucion.tipo` «prenda», con el «RECHAZO SIN FUNDAMENTO» | no (aditivo: el `data` del rechazo de la capa gana `recepcion`) | **construida (§613)**: el sobre de la 2.10 atado al recibo por el digest de su prueba; `consumo` -el PAR, la marca bajo el `consRoot` en la ventana-, `respuesta` con la `juzgada` -el juicio repetido contra la cabeza de índice `era − 1`: rechazada con prueba, o RECHAZO SIN FUNDAMENTO- o `rechazo` -la 2.6 de la causa de la capa, con el consumo rechazado igual a la marca-; humo vivo con prendas de verdad, 17 de 17 |
| E5 — catálogo y banco | sus vectores en `completitud/`, y un banco que siembre contra un nodo real las dos resoluciones del lote, las tres de la prenda y el rechazo sin fundamento | no | **construida (§614)**: `tools/banco_recibo_agregado.sh` contra tres nodos reales -el lote APLICADO con pruebas STARK reales del ejemplo `d2_lote_rpc`, que ahora reenvía la composición y la respuesta como un agregador-, 37 de 37 sobres; sus 37 vectores en `completitud/`, COPIADOS de esa corrida, y el arnés da 73 de 73 |

## Compatibilidad

Ninguna rotura. Quien no lea `recepcion` en el lote o en la prenda sigue igual; los vectores del
cable no se reescriben; la cabeza no cambia. El testigo del §576 cambia de sentido en la E3, y eso
está anunciado en él.

## Seguridad

La clave de gasto no viaja: nada de esto toca las pruebas, sólo lo que el nodo promete sobre
ellas. La respuesta del lote sólo gana su `recepcion`: la composición la tiene quien lo arma. Lo
que no cubre está en el D-H.

## Referencias

RFC-0010 (D-E y su decisión del §576, D-F, D-G, D-H); RFC-0007 (el sobre de rechazo); `PAQUETE.md`
2.10 y 2.11; `SECURITY.md` 2.ter; §116, §569, §571, §576; en `crates/zk-ssl-node/src/main.rs`,
`zkssl_applyMany`, `zkssl_pledge` y el testigo
`el_lote_y_la_prenda_evaluan_sin_consumir_recibo_y_la_via_directa_si`; y `apply_many` en
`crates/zk-ssl/src/two_phase.rs`.
