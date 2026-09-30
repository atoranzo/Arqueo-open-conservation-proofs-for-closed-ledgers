# RFC-0013 — El medio del ancla: las cabezas como checkpoints C2SP, cofirmadas por testigos ajenos

- **Estado:** PROPUESTO — el texto y las decisiones, escritos sobre lo medido en mtc-core y en
  las especificaciones C2SP; ninguna etapa construida. Nació BORRADOR y entra PROPUESTO al
  integrarse (§603): BORRADOR no tiene cara publicada en el cerrojo de `check_publicadas`, y el
  RFC-0011 nació igual, propuesto con sus decisiones delegadas. Las decisiones D-A a D-H son DELEGADAS y
  REVERSIBLES: la aceptación es del autor.
- **Autor:** Ángel José Toranzo Portela
- **Asistencia GenAI:** Claude (la sesión de mtc-core, que escribe el borrador entero sobre el
  mapa de las piezas que midió antes; integrado a nombre del autor en el §603) — ver
  [`GENAI.md`](../../GENAI.md)
- **Fecha:** 2026-09-30
- **Versión del protocolo afectada:** `zkssl/0.4` — **no la mueve, y tampoco mueve la cabeza**:
  lo que se publica es el ancla del RFC-0012, que ya se deriva de la cabeza firmada; el medio y
  su nota van fuera del cable, como el RFC-0012 D-A exige que vaya el ancla.
- **Asiento(s) de AUDITORIA:** §174 (el diseño del anclaje externo), §248 (una raíz servida por
  el acusado no prueba nada), §291–§292 (el MMR de cabezas y su pareja firmada), §590–§593
  (RFC-0012, el ancla y su sobre), §594 (el gate del diario en todo estado), §599 (la vista
  dividida servida por un nodo real: cada testigo solo no la ve); y, en mtc-core, sus asientos
  §14 y §15 (la interoperabilidad medida contra la implementación de referencia del IETF, en las
  dos direcciones, y el checkpoint de esa herramienta reconstruido desde sus tejas).
- **Backlog:** la 86 (elegir y medir el medio), que este RFC cierra en diseño y deja abierta en
  despliegue; la pregunta abierta de la 83 —«el diseño de referencia usa Ed25519 y ZK-SSL firma
  con XMSS con estado (…) hay que medir si cada testigo firma con su propia clave XMSS o si ahí
  conviene otra primitiva. Esto se mide antes de diseñar nada»—, que aquí se contesta con medida;
  B10.6 en su mitad de despliegue; y la 103 (dos testigos en el mismo disco), que este medio
  estrecha.

## Estado de las etapas

| etapa | qué entrega | ¿rompe el cable? | estado |
|---|---|---|---|
| E1 — el medio, decidido | este texto: qué se publica (D-A), en qué árbol (D-B), con qué firma (D-C), a qué testigos y cómo (D-D), quién lo comprueba (D-E), con qué cadencia (D-F), con qué piezas (D-G) y qué no resuelve (D-H) | no | **este texto, propuesto (§603)** |
| E2 — la nota y el árbol del medio | el árbol SHA-256 de anclas, la nota `checkpoint` con su firma ML-DSA-44 tipo `0x06`, y los vectores: positivos y negativos de la nota, consistencia SHA-256 | no (aditivo, fuera del cable) | pendiente |
| E3 — el publicador | el cliente `add-checkpoint` del protocolo tlog-witness, la retención de las cofirmas de testigos y su publicación como nota cofirmada | no | pendiente |
| E4 — el sobre en el kit | `tipo: "ancla-cofirmada"`: la nota, la política de testigos con umbral, la inclusión del ancla en el árbol del medio y, debajo, el sobre del ancla del RFC-0012; su banco y su catálogo | no | pendiente |

## Motivación

El RFC-0012 dejó el ancla «correcta y sin ningún sitio donde vivir» (D-H): el medio no se
eligió, y `doc/ANCLAJE_EXTERNO.md` §2 dice qué se le exige, «append-only ajeno al operador, con
orden legible por terceros», y qué hay que medir antes de elegirlo (la 86). Y la 83 dejó una
pregunta que bloqueaba el diseño: el modelo de referencia de testigos —Sigsum— cofirma con
Ed25519, y ZK-SSL firma con XMSS con estado; ¿firma cada testigo con su propia clave XMSS, o
conviene otra primitiva? «Esto se mide antes de diseñar nada.»

Se ha medido, fuera de este árbol. mtc-core —el crate que nació del `mtc/` de aquí— implementa
el formato de cofirma que el grupo PLANTS del IETF alineó con el ecosistema de testigos de C2SP:
`tlog-cosignature` define desde 2026 un tipo de cofirma **ML-DSA-44 con marca de tiempo, tipo
`0x06`**, sin estado, cuyo mensaje firmado es exactamente el `CosignedMessage` que mtc-core
implementa y que verificó byte a byte contra la implementación de referencia en Go: mismas claves
desde la misma semilla, mismos veredictos en 26 de 26 y 9 de 9 certificados, y el checkpoint de
esa herramienta reconstruido desde sus 2.122 entradas hasta la misma raíz (mtc-core, AUDIT §14 y
§15). Por el camino apareció que la propia herramienta de referencia todavía no escribe su
checkpoint como línea de `tlog-cosignature`, y su autor principal lo confirmó y aceptó el arreglo
(`ietf-plants-wg/merkle-tree-certs` #341): el ecosistema está convergiendo en ese formato, y su
utillaje es joven.

La respuesta a la 83, con esa medida: **en el medio no firma XMSS, ni la del operador ni una por
testigo**. Las claves con estado son justo lo que ese ecosistema evita en sus testigos, que son
infraestructura pública con muchos logs cada uno; la primitiva que el ecosistema ya estandarizó
para el caso poscuántico es ML-DSA-44, sin estado, y existe implementada y contrastada. XMSS se
queda donde está, firmando la cabeza: es la firma oponible. El medio es otro canal, con otra
clave, y ninguna de las dos hereda el estado de la otra.

Y el precio de no hacerlo lo acaba de escribir el §594: el guardián y el diario viven en el mismo
disco, y «dos testigos en el mismo disco son un solo testigo frente a una restauración» (103).
Un testigo ajeno que recuerde el último tamaño que cofirmó es el segundo testigo que ese disco
no puede dar.

## Diseño

### D-A — Lo que se publica es el ancla del RFC-0012, y el medio es un log de checkpoints C2SP

No nace un objeto nuevo. Cada `M` latidos (D-F) el publicador toma la cabeza firmada, deriva su
ancla como el RFC-0012 D-A —cualquiera puede derivarla—, y la **añade como hoja** al árbol del
medio (D-B). El medio es un log de transparencia en el sentido de C2SP: una nota `checkpoint`
(`tlog-checkpoint`) con tres líneas obligatorias, `origin`, `size` y la raíz en base64, firmada
por el publicador (D-C) y cofirmada por testigos ajenos (D-D).

```text
zkssl/v1/<huella_de_clave en hex>          ← origin: la identidad es la clave XMSS del operador
<size>                                      ← cuántas anclas lleva el medio
<raíz SHA-256 del árbol de anclas, base64>
— zkssl/v1/<huella> base64(key_id || timestamp || firma ML-DSA-44)   ← el publicador (D-C)
— <nombre del testigo> base64(…)                                     ← cada testigo (D-D)
```

El `origin` lleva la huella de la clave del RFC-0012 D-B, no un nombre de host: el medio es del
libro, no de la máquina, y la identidad que un testigo recuerda es la misma que un titular
verifica en cada cabeza.

### D-B — El árbol del medio es SHA-256 según RFC 6962, y es del medio, no del libro

Los testigos genéricos comprueban la consistencia entre dos checkpoints con el MTH de RFC 6962
sobre SHA-256; ninguno recompone Rescue Prime, y no se les va a pedir. El árbol de cabezas que la
cabeza ya firma (§292, `mmr_hoja`/`mmr_nodo` sobre Goldilocks) sigue siendo **la historia
canónica** y no se toca. El medio lleva su propio árbol: la hoja `i` es la huella del ancla `i`
(32 bytes, RFC-0012 D-A), los nodos son SHA-256 con los prefijos `0x00`/`0x01` de RFC 6962, y
las pruebas de consistencia entre `size` viejo y nuevo son las que `tlog-witness` exige.

Esto responde al descarte del RFC-0012 «un árbol de anclas propio: segundo productor del árbol
de cabezas». **No es un segundo productor de la historia**: cada hoja del medio lleva dentro,
firmada por XMSS a través de la cabeza, la pareja `(mmrRoot, mmrSize)` del árbol canónico. El
árbol del medio solo afirma que la **secuencia de anclas publicadas creció y no se reescribió**,
que es exactamente la propiedad que `ANCLAJE_EXTERNO` §2 exige del medio; y si las dos
secuencias discreparan, cualquiera con las anclas lo ve recomputando ambas, y la discrepancia es
prueba contra el publicador, no ruido.

### D-C — La nota la firma el publicador con ML-DSA-44, tipo `0x06`, y esa clave no tiene estado

`signed-note` no tiene tipo para XMSS y los testigos ignoran firmas de tipos que no conocen. La
nota se firma con la construcción que `tlog-cosignature` reserva al tipo `0x06`: el mensaje
`subtree/v1` con el `origin`, el subárbol `[0, size)`, la raíz y una marca de tiempo, y en la
línea `key_id || timestamp || firma`, con `key_id = SHA-256(nombre || "\n" || 0x06 || clave)[:4]`.
Es lo que los logs de certificados Merkle del IETF firman en sus checkpoints (perfil `mtc-tlog`),
y lo que mtc-core ya produce y verifica. La clave ML-DSA-44 del publicador **no tiene índice ni
guardián**: es el primer canal firmado de esta casa que un contador borrado no puede comprometer.

**Puente, declarado**: mientras los testigos públicos de hoy solo acepten `origin` con Ed25519
(tipo `0x01`), la nota puede llevar además una línea Ed25519 del mismo publicador. Los testigos
ignoran lo que no entienden, así que las dos líneas conviven. Lo que ese puente vale y lo que
cuesta va en Seguridad.

### D-D — Los testigos son ajenos, genéricos y de umbral, y el protocolo es `tlog-witness`

El publicador envía a cada testigo `add-checkpoint`: `old <size anterior>`, hasta 63 líneas de
prueba de consistencia, y la nota firmada. El testigo verifica la firma del `origin` con la
clave que tiene registrada, verifica la consistencia con **el último tamaño que él cofirmó**, y
devuelve su cofirma. Si el tamaño viejo no es el suyo, responde `409` con el tamaño que
recuerda: **ésa es la detección de la 103**, el directorio restaurado presenta un tamaño menor
y el testigo lo dice. Qué testigos valen y cuántos lo decide **el cliente**, no el operador: el
kit ya tiene esa política para las cofirmas XMSS (`--testigos`, `--k`, S319) y la reutiliza. El
testigo propio de ZK-SSL sigue existiendo y cofirmando cabezas con XMSS; este canal no lo
sustituye, lo rodea de terceros que no son de la casa.

### D-E — Lo que un tercero comprueba, sin el nodo

El sobre `tipo: "ancla-cofirmada"` (E4) verifica, en este orden: la nota (tres líneas, firma del
publicador de tipo `0x06`); las cofirmas de los testigos contra la política de umbral; la
inclusión de la huella del ancla en la raíz de la nota (camino SHA-256); y, debajo, el sobre del
ancla del RFC-0012 tal cual: que el ancla ES la cabeza firmada que dice ser. Un VERDE dice: «esta
cabeza estaba publicada, en esta posición, cuando `k` testigos ajenos la vieron». Lo que no dice
lo dice D-H.

### D-F — La cadencia es la `M` del RFC-0012, y el nodo no espera a nadie

Una hoja por ancla, un ancla cada `M` latidos, `M` = 1.440 como propone el RFC-0012 D-F hasta que
haya operador real. La publicación es **asíncrona**: la época cierra y la cabeza se firma sin
esperar cofirmas; lo que un testigo caído cuesta es liveness del medio, no del libro, que es el
precio (b) de la 83 y se acepta como allí se acepta.

### D-G — Las piezas existen, medidas, y la decisión es dónde viven

- El árbol SHA-256 de RFC 6962 con inclusión y consistencia: `mtc-core::{subtree, log}`, que
  pasa los 65.058 casos de los vectores acumulados del borrador del IETF, sus vectores grandes
  hasta `2^64-1` hojas, y reconstruyó desde tejas el log de 2.122 entradas de la implementación
  de referencia hasta su misma raíz.
- El mensaje `subtree/v1`, el `key_id` y la firma ML-DSA-44: `mtc-core::cosign`, contrastado
  byte a byte con la implementación en Go (mismas claves desde la misma semilla).
- PEM y base64 estrictos sin dependencias: `mtc-core::pem`.
- El verificador de notas: `interop checkpoint` de mtc-core ya lee una nota, reconstruye la raíz
  y comprueba la línea del publicador en las dos formas que hoy existen.

**Decisión delegada**: depender de mtc-core por commit fijado, como se depende de `hbs-state`, o
copiar los tres módulos con su procedencia. Lo primero evita dos implementaciones del mismo
problema; lo segundo evita una dependencia sobre un crate no auditado. Cualquiera de las dos vale
para E2; ninguna cambia el cable.

### D-H — El residuo, declarado

Los del RFC-0012 D-H siguen: el medio es confianza **desplazada, no eliminada** —ahora a un
umbral de testigos ajenos, que es mejor que a un solo medio y peor que a nada—; la cola entre
anclas queda acotada por `M`, no cerrada; y el ancla prueba qué historia era canónica, no que
estuviera completa. Y los propios: (1) no hay hoy testigo público que acepte `origin` de tipo
`0x06`; hasta que lo haya, E3 se mide con el puente Ed25519 contra un testigo público real y con
`0x06` contra el verificador de mtc-core, y se dice cuál de los dos se midió; (2) `ml-dsa` no
está auditado, lo dice su propio crate; (3) la clave ML-DSA-44 del publicador es una clave más
que custodiar, sin estado pero con custodia.

## Lo que se DESCARTÓ al medir

- **Un tipo de nota para XMSS.** `signed-note` no lo define, `0x03` está reservado, y un testigo
  ajeno ignora lo que no conoce: la nota valdría solo para testigos de la casa, que es lo que la
  103 dice que no basta.
- **Una clave XMSS por testigo.** La pregunta de la 83, contestada: es el estado que el ecosistema
  de testigos evita, y ya tiene primitiva sin estado estandarizada para lo poscuántico.
- **Publicar el MMR de cabezas como árbol del medio.** Sus nodos son Rescue Prime; ningún testigo
  genérico los recompone, y pedirlo es pedir un testigo de la casa.
- **Esperar un tipo de firma de log poscuántico distinto de `0x06`.** No hay ninguno propuesto, y
  `0x06` ya lo usan los logs de certificados Merkle para firmar sus propios checkpoints.
- **Un método del cable para la nota cofirmada.** El §248 otra vez para la nota del publicador;
  las cofirmas de los testigos, en cambio, sí prueban algo sirviéndose, porque no las hizo el
  acusado. Se sirven como fichero de checkpoint en la convención de `tlog-checkpoint`, fuera
  del cable, y el kit las lee de ahí.

## Compatibilidad

No mueve `zkssl/0.4`: ni métodos, ni campos, ni bytes bajo la firma. Todo lo nuevo vive fuera del
cable —el árbol del medio, la nota, el publicador— y en el kit, como una familia nueva con su
`tipo`, sus vectores y su banco, que es la vía que `NUCLEO.md` §3 reserva y que el RFC-0012 ya
usó. Los vectores nuevos son de tres clases: notas positivas y negativas (firma mal, `origin`
distinto, raíz que no cuadra, cofirma de testigo no nombrado), consistencia SHA-256 (los del
borrador del IETF sirven tal cual, y ya están copiados en mtc-core con su licencia), y cofirmas
`0x06` (el corpus de mtc-core, contrastado con Go).

### Por qué entra por RFC

Porque elige el medio, que `ANCLAJE_EXTERNO` §2 dejó sin elegir a propósito; porque introduce
una segunda primitiva de firma en la casa, y eso es una decisión de diseño y no de código; y
porque contesta una pregunta que el BACKLOG tenía marcada como «antes de diseñar nada».

## Seguridad

- **El principio del API se conserva**: la clave de gasto no viaja. La nota lleva una raíz, dos
  números y firmas de claves públicas; ninguna clave privada de nada se acerca.
- **La firma oponible sigue siendo XMSS.** La clave ML-DSA-44 solo firma notas del medio. Una
  clave de nota robada permite hacer cofirmar a testigos una secuencia de anclas falsa; **no
  permite fabricar cabezas**, y una ancla que no corresponda a una cabeza firmada por XMSS es
  detectable por cualquiera que tenga las dos. El daño es confusión y denegación, no falsificación.
- **El puente Ed25519 no es poscuántico**, y se dice: mientras esté, la admisión en los testigos
  de hoy descansa en una clave clásica. Lo que un adversario cuántico gana con ella es lo del
  punto anterior, no más. Se retira cuando haya testigos con `0x06`.
- **La propiedad es de umbral.** Un adversario que controle más de `k` testigos la rompe; `k` y
  los nombres los pone el cliente, y la 83 lo tiene declarado.
- **Lo que este medio cierra de la 103**: un directorio restaurado presenta un `size` menor que el
  que los testigos recuerdan, y los testigos responden con el suyo. Lo que no cierra: un
  operador que además controle todos los testigos, y el diario ausente sin ancla publicada.

## Referencias

- `spec/rfc/0012-el-ancla-de-cabezas.md` (D-A, D-B, D-D, D-F, D-H y su descarte del árbol
  propio); `doc/ANCLAJE_EXTERNO.md` §2 y §7; `doc/CONFIANZA_RESIDUAL.md`, B10.6 y B10.7.
- `BACKLOG.md`: la 83 (la cadena de la confianza residual y su pregunta abierta), la 86 (elegir y
  medir el medio), la 103 (dos testigos en el mismo disco), la 87 y la 108 (el esquema versionado
  y la firma corta; ML-DSA como candidato).
- C2SP: `signed-note` (tipos `0x01`, `0x04`, `0x06`), `tlog-checkpoint`, `tlog-cosignature`
  (el mensaje `subtree/v1` y el `key_id` de ML-DSA-44), `tlog-witness` (`add-checkpoint`, `409`
  con el tamaño recordado), `mtc-tlog` (el checkpoint firmado por el cosigner de la CA como nota).
- mtc-core (`github.com/atoranzo/mtc-core`): `src/subtree.rs`, `src/log.rs`, `src/cosign.rs`,
  `src/pem.rs`, `examples/interop.rs`; su registro de auditoría, §9 (el arranque consulta el
  diario en todo estado, leído en este nodo), §14 y §15 (la interoperabilidad, medida dos veces).
- `ietf-plants-wg/merkle-tree-certs`, issue #341: el checkpoint de la herramienta de referencia
  no es todavía una línea de `tlog-cosignature`; confirmado por su autor.
