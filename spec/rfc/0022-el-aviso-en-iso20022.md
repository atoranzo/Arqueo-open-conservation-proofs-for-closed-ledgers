# RFC-0022 — El aviso en ISO 20022: un elemento propio, dentro del mensaje o fuera, que escribe el lado del pagador

- **Estado:** PROPUESTO (§827) — el texto y sus tres etapas, escritos sobre lo que midió
  `doc/AVISO_EN_ISO20022.md` (§791, aceptada como análisis en el §799) y sobre las seis decisiones
  que tomó el §861, y remedidos en el árbol del sello. Las seis las tomó el §861 por delegación del
  autor («resuelvelas aplicando los principios»), con el criterio del §609 y el precedente del
  §642; REVERSIBLES; entran con su aceptación. Están escritas en `doc/AVISO_EN_ISO20022.md` §8, con
  su razón, sus descartes y la tabla de cuándo se reabren, y en la nota del §861 de la 11 del
  `BACKLOG.md`, con sus mismos nombres, de la D-1 a la D-6. Este RFC las escribe como protocolo, las
  cita y no las vuelve a razonar. Decide una sola cosa de protocolo que ellas no dicen: corrige la
  compuerta del KAT de la D-5, que nombraba la fila E, por delegación del autor, REVERSIBLE y con su
  razón (abajo, D-5). Deja dos preguntas para el autor, abiertas
  (§«Preguntas abiertas para el autor»). Ninguna etapa construida: el paso a ACEPTADO exige
  construirlas (regla 4 del PROCESO) y la aceptación del autor, que no se delega.
- **Autor:** Ángel José Toranzo Portela
- **Asistencia GenAI:** Claude, en una sesión de Claude Code en la nube (§827): redactó el texto
  sobre la nota del §791 y las decisiones del §861, fuera del paso 4 de `GENAI.md` — ver
  [`GENAI.md`](../../GENAI.md)
- **Fecha:** 2026-10-21 (redactado el 2026-10-15, sobre `d4d6d45`, el §814)
- **Versión del protocolo afectada:** `zkssl/0.4` — **no cambia el cable** (S1 del RFC-0021) ni
  nada de lo que el nodo firma. El elemento no viaja por el JSON-RPC: va dentro de un mensaje
  ISO 20022 o fuera de él. La E2 añade una familia de vectores con rutas nuevas (S5, aditivo: la
  R2 del RFC-0021).
- **Asiento(s) de AUDITORIA:** §21 (los códigos ISO contra el catálogo, que no hablan del aviso),
  §39 (el canal lateral, «sin especificar»), §106 (una sola familia de supuestos), §268 (el B11.2,
  con la 11), §609 (el criterio), §642 (el precedente), §791 (la nota, medida y sin elegir), §799
  (aceptada como análisis), §835 (`ring`, que `check_curvas` no veía), §838 (la excepción del §106
  para el transporte TLS, y `check_curvas` con R4 y R5), §860 (la D-9 de la 52, la clave por
  relación), §861 (las seis decisiones, la nota de la 11 y las frases corregidas donde viven) y §827
  (este RFC).
- **Backlog:** la **11** (canal lateral de ISO 20022), que este RFC no cierra: con él PROPUESTO,
  sigue abierta hasta la E1 y, después, solo por la E3, que es el B11.2, como dice la nota del
  §861; con la **122**, cerrada en el §838, y la **52** (su D-9, la clave del aviso cifrado).

## Estado de las etapas

| etapa | qué entrega | ¿rompe el cable?, y qué superficie | estado |
|---|---|---|---|
| E1 — el prototipo por materiales | en `crates/zk-ssl/src/iso.rs`: `Pacs008` gana el aviso, una función que lo escribe en la serialización de referencia, otra que la lee, y un cobro que toma el aviso solo de la orden; con un test discriminante por propiedad (abajo, «E1 — El prototipo, en detalle») | no: ninguna de S1 a S6. La API de Rust queda fuera de la promesa (RFC-0021, D-A, «Fuera de la promesa») | sin construir |
| E2 — el KAT | la familia `spec/vectors/aviso/`, con su `MANIFIESTO.txt`, su línea en `HUELLAS.sha256`, su lector en `tools/segunda/` y su mutante; y el bloque `superficie` de `spec/RPC.md` §«Versionado y compatibilidad», al día, como hizo el §809 | S5, aditivo (R2): rutas nuevas | sin construir; detrás de su compuerta (D-5) |
| E3 — el aviso cifrado | el B11.2: el espacio de nombres del formato cifrado, con su KAT; y la retirada de la versión en claro dentro del mensaje, que pasa a desaconsejada | S5: lo cifrado entra aditivo; los vectores en claro se mudan (R6 b), no se reescriben | sin construir; espera a una de las dos vías de la D-4 |

## Motivación

La 11 dice que el aviso del pendiente —posición, aleatorio e importe— viaja fuera del mensaje,
«sin especificar como», y que eso «bloquea cualquier piloto» (la entrada 11 del `BACKLOG.md`). Sin
el aviso el receptor no cobra aunque el dinero esté ahí (`crates/zk-ssl/src/two_phase.rs:23-30`).
El aviso es `PendingNotice { position, salt, amount, x }` (`two_phase.rs:62-74`), y la presencia de
`x`, el sobre de reversión, es la señal de despacho entre v1 y v2 (`:68-73`).

La nota del §791 midió, contra el esquema y no contra el recuerdo, qué campos de los mensajes
podrían llevarlo, y dejó seis decisiones al autor (`doc/AVISO_EN_ISO20022.md` §8). El autor las
delegó, y las tomó el §861. Este RFC las escribe como protocolo, porque la D-5 decide que lo es.

**Lo medido, y de dónde sale cada cosa.**

1. **El puente de hoy es la vía de en medio.** `settle_pacs008_two_phase`
   (`crates/zk-ssl/src/iso.rs:425`) liquida con `layer.send(sender_key, …)` (`:449`): la clave del
   pagador entra en la capa. Ahí el aviso nace al liquidar la orden entrante, después de ella, y
   el puente lo devuelve aparte del informe. `Pacs008` son seis campos (`iso.rs:61-75`) y se
   construye en un solo sitio (`:642`); `SplmtryData` no aparece en el código: solo en siete
   líneas de comentario, las que puso el §861 en `iso.rs`, `client.rs` y el SDK (`git grep -c`).
2. **En la vía por materiales el aviso existe antes de la orden.** `send_materials` toma la
   posición con `allocate_pending` y la devuelve en `pending_position` antes de que el pagador
   pruebe (`crates/zk-ssl/src/client.rs:252-255`; `SendMaterialsDto.pending_position`,
   `crates/zk-ssl-wire/src/lib.rs:304`), y la sal y el importe son del pagador. Es la vía en la que
   ninguna clave de gasto llega a la capa (`a_whole_payment_without_giving_any_key_to_the_layer`,
   `client.rs:996`; `PRINCIPIOS.md` §4.2).
3. **El estándar tiene un campo de extensión en los tres mensajes del camino.** Leído en los
   modelos que `xsdata` genera desde los XSD, en la rueda `pyiso20022` 1.6.2, sin instalarla ni
   ejecutarla: `CdtTrfTxInf/SplmtryData` en el pacs.008.001.08 (`doc/AVISO_EN_ISO20022.md` §2, la
   fila A) y en el pain.001.001.09, y `TxDtls/SplmtryData`, una lista, en la `EntryTransaction10`
   del camt.054.001.08; las dos últimas, con sus líneas en la rueda, en `doc/AVISO_EN_ISO20022.md`
   §7, «el salto al titular», que las selló el §861. Su sobre, `Envlp`, admite un elemento de
   cualquier espacio de nombres. Es una copia derivada, no el XSD: las tres webs de ISO 20022 y de
   las prácticas de mercado dieron 403 a la sesión que decidió la 11, y el §861 lo cuenta como lo
   que no midió.
4. **Ya hay dos contenedores del aviso, y ninguno sirve aquí.** `PendingNoticeDto` no lleva `x`
   (`crates/zk-ssl-wire/src/lib.rs:268-272`); `AvisoV2`, el fichero del CLI, lo lleva obligatorio
   (`crates/zk-ssl-cli/src/cobro.rs:34-44`). Los dos son JSON. Lo que sí existe es la escritura de
   los valores: `Q` y `B32` canónicos, que `zk_ssl_hash::cantidad_canonica` y `hex_canonico` leen
   (`crates/zk-ssl-hash/src/lib.rs:1150` y `:1135`), la práctica del §640 y del §662.
5. **Las frases del árbol decían otra cosa.** «ISO 20022 no tiene campo donde llevarlos»
   (`iso.rs:484-492` y `:528-530`), y la cita al §21 de `spec/RPC.md` §«Pago en dos fases», que es
   el §39. Las corrigió el §861 con marca donde viven, por la D-6, y lo que queda por corregir lo
   dice esa decisión (abajo, D-6).

## Diseño

Cada decisión dice aquí lo que el protocolo es. Su razón, sus descartes y su condición de
reapertura están en `doc/AVISO_EN_ISO20022.md` §8, bajo la misma letra, y no se copian.

### D-1 — El campo: un formato y dos portadores

- **Dentro del mensaje**, el aviso va en `CdtTrfTxInf/SplmtryData` del pacs.008 (la fila A de
  `doc/AVISO_EN_ISO20022.md` §2). **Del banco del acreedor al titular**, el mismo elemento va en
  `Ntry/NtryDtls/TxDtls/SplmtryData` del camt.054; si no, fuera del mensaje. **Fuera del
  mensaje**, el mismo elemento (la opción E de la nota, §7, con el formato ya especificado).
- **Dónde nace.** En la vía por materiales, antes de la orden (hecho 2). Lo escribe **el lado del
  pagador**: el pagador lo pone en su pain.001 (`CdtTrfTxInf/SplmtryData`) y su banco lo lleva al
  pacs.008. **El operador de la capa no escribe nada.**
- **El puente de en medio** —`settle_pacs008_two_phase`, con la clave dentro de la capa— no es el
  modelo, y **no gana** un emisor de pacs.008. El campo de `Pacs008` modela la orden que llega con
  un aviso nacido antes.
- Descartados la remesa (`RmtInf`), `InstrForCdtrAgt`, el pacs.002 y la combinación de A con un
  `Ustrd` de reserva: `doc/AVISO_EN_ISO20022.md` §8, D-1, y §7.

### D-2 — La codificación: un elemento XML propio, que se conforma por infoset

- Un elemento propio, con un **espacio de nombres versionado**, dentro del `Envlp`. Dentro, una
  hoja por cada campo de `PendingNotice`: la posición y el importe en `Q` canónico, la sal y `x`
  en `B32` canónico, y `x` solo en un v2. La escritura de cada valor es la del cable; **sin
  base64 y sin JSON dentro del XML**. Cada limbo de la sal y de `x` es menor que `p` (RFC-0016).
- **La conformidad es del infoset** —los nombres y el texto de las hojas—, no de los bytes. Por eso
  el KAT de la E2 es un infoset más **una serialización de referencia**.
- **El lector conforme** usa un analizador XML de verdad, fuera del puente, y rechaza una hoja que
  no sea canónica. El prototipo de la E1 lee solo la serialización de referencia y lo declara:
  «No es un parser XML» (`iso.rs:34`) sigue siendo cierto, y en `zk-ssl` no entran `serde` ni un
  crate XML.
- **El contenedor es nuevo**: el tercero, junto a `PendingNoticeDto` y `AvisoV2`. Lo que no es
  nuevo es la escritura de los valores.
- Los nombres del elemento y de sus hojas, y el URI del espacio de nombres, los fija la E2 con su
  KAT: hasta entonces no hay bytes que copiar. La E1 va antes y su test 4 necesita unos: usa
  nombres y un URI provisionales, que no son protocolo, no se publican como vectores y la E2
  sustituye.
- **Lo que el infoset aún no fija**, y fija la E2 con su KAT, porque dos lectores lo resolverían
  distinto: qué hace el lector con dos avisos en la misma transacción —`SplmtryData` es 0..n— o
  con uno en el `SplmtryData` de la raíz del pacs.008, que también existe (la fila A de
  `doc/AVISO_EN_ISO20022.md` §2); con una hoja duplicada o desconocida, con blanco alrededor del
  texto de una hoja y con una hoja `x` vacía; y qué rechaza el lector ante un espacio de nombres
  desconocido (la D-3): el aviso o el mensaje, o si ignora la extensión de otra práctica de
  mercado en el mismo `Envlp`.

### D-3 — v1 y v2, con una sola señal

- Los dos. **La presencia de la hoja `x` decide** entre v1 y v2, como `PendingNotice.x`.
- **El espacio de nombres solo dice el formato**: en claro o, cuando exista, cifrado. Un elemento
  con un espacio de nombres desconocido se rechaza; qué se rechaza lo fija la E2 (D-2).

### D-4 — El orden frente al B11.2: no se espera al cifrado

- **El aviso en claro dentro del mensaje es una elección del pagador**, no lo que el estándar trae
  por defecto. **La vía fuera del mensaje, con el mismo elemento, no concede observación**
  (`SECURITY.md` §2.ter).
- **La fuga se dice**: dentro del mensaje y en claro, la posición y la sal, y en un v2 también `x`,
  llegan a toda la cadena de agentes, sin poder de cobro (Seguridad, abajo). ⚠️ La D-4 de
  `doc/AVISO_EN_ISO20022.md` §8 dice solo «posición y sal», y lleva su marca (§827): con `x`, quien
  lea el mensaje aprende qué cuenta pagó y su Δ, la fuga por la que el RFC-0008 sacó `X` de su
  sobre (su D-I). Si un v2 puede ir en claro dentro del mensaje es la pregunta 2, del autor.
- **La versión en claro entra con su retirada escrita desde el principio.** Cuando exista la
  cifrada, la en claro dentro del mensaje pasa a desaconsejada, y sus vectores se mudan por la R6 b
  del RFC-0021.
- **El cifrado necesita una de dos cosas**:
  - **(i) una enmienda del §106**, que solo puede hacer el autor, como la del §705 para ML-DSA-44:
    un KEM de retículos añade una familia de supuestos (§106.3), y un Diffie-Hellman sobre una curva
    lo prohíbe el §106.1. Es la pregunta 1, abajo;
  - **(ii) la vía simétrica**: XChaCha20-Poly1305, el AEAD de la casa
    (`crates/zk-ssl/src/crypto.rs:39-50`; `chacha20poly1305` ya está en
    `crates/zk-ssl/Cargo.toml:39` y en `crates/zk-ssl-sdk/Cargo.toml:40`), con una clave que el
    receptor entrega junto con su dirección, **una por relación**, derivada de `sk` con dominio
    propio: la D-9 de la 52, que resolvió el §860 (`doc/ROTACION_DE_CUENTAS.md` §6). No añade
    familia ni primitiva. Una sola clave por cuenta, entregada a cada pagador, dejaría a cada uno
    descifrar los avisos de los demás (DEDUCIDO).
- **Elegir entre (i) y (ii) es del B11.2**; los principios —el §106.3 y el §3 de `PRINCIPIOS.md`,
  minimalista— inclinan a la (ii). La (i) solo existe si el autor enmienda el §106.
- **Con una curva, no**, aunque `ring` ya esté en el SDK por el TLS de `ureq`: lo prohíbe el §106, y
  desde el §838 lo ve también la puerta, porque la R5 de `tools/check_curvas.py` da ROJO a un
  `ring::`, o a `rustls` o `webpki` como palabra entera, en las fuentes de la casa. La excepción del
  §106 que el §838 dio al transporte TLS del cli y del SDK es para el transporte, y no alcanza al
  aviso.

### D-5 — Es protocolo, y entra por RFC

- **Sí, por RFC**: éste, con el número libre al entrar. `spec/rfc/PROCESO.md` llama protocolo a lo
  que cruza el cable, y un campo de un mensaje ISO no está en esa lista; entra igual, porque un
  formato que una segunda implementación no puede reproducir no fija estándar (§609).
- **El KAT va en una familia propia**, `spec/vectors/aviso/`, con su `MANIFIESTO.txt`, su línea en
  `HUELLAS.sha256` y su lector en `tools/segunda/`. **No va en `nucleo/`**: el núcleo es el lado del
  verificador, lo que un tercero recompone sin el nodo (RFC-0005, D-A), y `nucleo_kat.rs` da un KAT
  por cada `fn` NUCLEO (`crates/zk-ssl-cli/src/nucleo_kat.rs:3-4`). El aviso no es eso.
- **El KAT entra detrás de una compuerta**: el cotejo con el XSD original de los tres mensajes de
  la D-1 —`CdtTrfTxInf/SplmtryData` del pain.001 y del pacs.008, y `TxDtls/SplmtryData` del
  camt.054— y las reglas de uso públicas de al menos una práctica de mercado objetivo. Las dos son
  lecturas, no actos ante terceros; contactar con un banco queda fuera. Hoy las dos lecturas dan
  403 (hecho 3). ⚠️ **Es la única decisión de protocolo nueva de este RFC**, una corrección tomada
  aquí por delegación del autor, con el mismo criterio, REVERSIBLE: la D-5 de
  `doc/AVISO_EN_ISO20022.md` §8 dice «las filas A y E y el camt.054». La razón: la fila E de su §2
  es el pacs.002, que la D-1 descarta, y la opción E de su §7 va fuera del mensaje, sin XSD que
  cotejar; y el pain.001, donde el pagador escribe el aviso (D-1), no salía. La compuerta coteja
  los mensajes que llevan el aviso, y son esos tres. Lo sellado lleva su marca (§827), y se reabre
  con la D-5 o si el autor prefiere la compuerta del §861.
- **El sello de la E2 pone al día el bloque `superficie`** de `spec/RPC.md` §«Versionado y
  compatibilidad» —el manifiesto nuevo, `vectores.json` y `vectores.lineas`—, como pide la D-5 de la
  nota y como hizo el §809.

### D-6 — Las frases, corregidas donde viven

La cumplió el §861: con marca, en los comentarios de `iso.rs`, de `client.rs` y del SDK, en
`spec/RPC.md`, `ARQUITECTURA.md`, los dos `PAPER` y `doc/DIAGNOSTICO_ESCALADO.md`. Lo que queda
para otro sello —el «§21» de la plantilla de los RFC y de los RFC del 0002 al 0007, el de la 74,
cerrada, el de `doc/CADUCIDAD_PENDIENTE.md` y el del `println!` de
`crates/zk-ssl-sdk/examples/e2e.rs`, que es código— lo enumera la D-6 de
`doc/AVISO_EN_ISO20022.md` §8. Este RFC no corrige ninguna.

### Cuándo se reabre

La tabla de `doc/AVISO_EN_ISO20022.md` §8, «Cuándo se reabre», que es la de estas decisiones: la
D-1, si el XSD o las reglas de uso quitan o restringen los portadores; la D-2, si una práctica de
mercado veta espacios de nombres ajenos o acota el `Envlp`; la D-4, cuando exista la versión
cifrada o si una medida de lo que aprende un agente da más que el vínculo declarado; la D-5, si el
autor prefiere documentación del puente o cambia la D-A del RFC-0021; y todas, si el autor revoca
cualquiera. Este RFC le añade solo la condición de su corrección de la compuerta del KAT (D-5): se
reabre si el autor prefiere la del §861.

## Preguntas abiertas para el autor

1. **¿Se enmienda el §106 para el aviso cifrado?** Es la vía (i) de la D-4: un KEM de retículos,
   que añade una familia de supuestos (§106.3), como la enmienda del §705 añadió ML-DSA-44. Sin
   ella, la E3 solo puede ir por la (ii), la simétrica, que no la necesita. La delegación no la
   toma: enmendar el §106 es del autor, y el §861 lo dejó así («la enmienda del §106 que pediría un
   KEM para el aviso cifrado»). **Abierta.**
2. **¿Puede ir un v2 en claro dentro del mensaje?** La D-4 lo deja a elección del pagador, con la
   fuga de la posición y la sal; con `x`, quien lea el mensaje aprende también qué cuenta pagó y su
   Δ, y enlaza los avisos del mismo emisor (Seguridad). Las ramas: **sí**, con esa fuga declarada,
   como dice hoy la D-4; **no**: un v2 viaja dentro del mensaje solo cifrado, y en claro solo
   fuera de él, con el precedente de la D-I del RFC-0008 («la privacidad es propiedad del
   protocolo, no una política del emisor»); o **sí, cuando el sobre gane aleatoriedad**, que es
   rotura de formato del compromiso (la D-I la deja como su reversión). La delegación no la toma:
   la segunda revisa una decisión que el §861 tomó, y la tercera, el compromiso del RFC-0003,
   ACEPTADO. **Abierta.**

La aceptación de este RFC, también del autor.

## E1 — El prototipo, en detalle

En `crates/zk-ssl/src/iso.rs`, junto a `the_full_two_phase_iso_cycle` (`iso.rs:975`):

- **El campo.** `Pacs008` gana `aviso: Option<String>`, el contenido del `Envlp` de
  `CdtTrfTxInf/SplmtryData`. Se construye en un solo sitio (`iso.rs:642`).
- **El escritor.** `aviso_a_envlp(&PendingNotice) -> String` da la serialización de referencia.
- **El lector.** `aviso_de_envlp(&str) -> Result<PendingNotice, …>` lee solo esa serialización. Las
  hojas pasan por `cantidad_canonica` y `hex_canonico`; cada limbo de la sal y de `x` tiene que ser
  menor que `p`; un espacio de nombres desconocido se rechaza.
- **El cobro.** `claim_pacs008_materiales` toma el aviso solo de la orden y lo aplica con
  `apply_claim`, con un recibo que el receptor probó en local. Da un ACSC con los identificadores
  de esa orden, o un RJCT, con un código del catálogo que el puente ya usa, si el importe del aviso
  no es el de la orden. Ninguna clave de gasto llega a la capa.
- **Los tests**, uno discriminante por propiedad (`PRINCIPIOS.md` §9). El 1, el 2, el 3 y el 5 son
  las cuatro propiedades que nombra `doc/AVISO_EN_ISO20022.md` §9 desde el §861 —el positivo, el
  negativo, la canonicidad del lector y la atadura entre el `ACSC` y la orden—; el 4, que prueba la
  D-3, y el RJCT por importe del 5, que prueba el cobro de arriba, vienen de la decisión de la 11,
  y el §861 no los copió en el §9:
  1. **el positivo**: materiales, prueba local y `apply_send`; la orden se escribe con el aviso y
     el lado del acreedor lo lee solo de ella; cobro por materiales, ACSC con el `EndToEndId` de la
     orden, y los saldos;
  2. **el negativo**: un byte de la sal cambiado que deja el hex canónico; el lector da `Ok`, y el
     rechazo es de la capa, en el paso que se mida, no supuesto;
  3. **la canonicidad**: mayúsculas, un cero a la izquierda en `Q` y un limbo `>= p` se rechazan
     antes de llamar a la capa, y su gemelo canónico pasa;
  4. **una sola señal**: con la hoja `x` el aviso es v2, sin ella v1, y un espacio de nombres
     desconocido se rechaza;
  5. **la atadura**: con dos órdenes, el ACSC nombra la orden que trajo el aviso; un aviso con otro
     importe da RJCT. Ata el `ACSC` a la orden, no el pendiente a la orden (Seguridad, «El límite
     de `claim_pacs008`»).
- **Lo que mueve**: el pin de `zk-ssl` en la tabla del canon y las cifras que lo repiten. Tamaño
  DEDUCIDO: de 100 a 150 líneas de código y unas 200 de tests.

## Compatibilidad

**El cable no se mueve** (S1), ni nada firmado (S3), ni los sobres del kit (S4), ni el kit (S6).
La E1 es API de Rust, fuera de la promesa. La E2 añade rutas en `spec/vectors/` (S5, R2). La E3
añade el formato cifrado con su espacio de nombres (R2) y desaconseja el en claro dentro del
mensaje: retirarlo no es una ruptura R1, porque su lector no está en S1 a S4, y en S5 sus vectores
se mudan, no se rompen (RFC-0021, D-A y la R3 de su D-B). Esto último es DEDUCIDO, de leer el
RFC-0021.

**El formato no es una superficie**, y se dice expresamente: aunque la D-5 lo llama protocolo, no
es ninguna de S1 a S6 del RFC-0021, y lo que la promesa cubre de él son sus vectores, en S5. El
lector de un banco no tiene otra promesa de compatibilidad que esos vectores y la versión de su
espacio de nombres (D-3). Hacerlo superficie pediría cambiar la D-A del RFC-0021, que es una de las
condiciones con que se reabre la D-5; este RFC no lo hace.

## Seguridad

- **La clave de gasto no viaja jamás** (regla 3 del PROCESO): **ningún efecto**. El elemento no
  lleva clave, y la vía que este RFC especifica es la de materiales. La vía de en medio, que mete
  la clave del pagador en la capa, no es el modelo y no gana emisor.
- **Conocer el aviso no permite cobrar**: con el aviso completo y sin la clave del receptor, el
  cobro se rechaza (`a_third_party_cannot_claim_the_pending`, `iso.rs:1033`).
- **Lo que el aviso en claro da: el vínculo y, en un v2, quién paga y su Δ.** La posición une un
  envío con su cobro; la sal, con la identidad del receptor y el importe, recompone el compromiso y
  deja ver cuándo desaparece del árbol (`doc/AVISO_EN_ISO20022.md` §5). Y en un v2, `x`: `X =
  M(refund_id, [delta, 0, 0, 0])` es un compromiso sin aleatoriedad
  (`crates/zk-ssl/src/pending.rs:88-98`), la capa exige que `f` sea el `public_id` de la cuenta que
  paga (`two_phase.rs:710-714`; la (7) de la 12) y las identidades se enumeran sin credencial
  (`zkssl_accountCount` y `zkssl_publicId`, `spec/RPC.md` §«Lectura»). Quien lea el pacs.008 o el
  camt.054 prueba `M(pid, d)` para los `d` plausibles y aprende qué cuenta pagó y su Δ; y dos
  avisos del mismo emisor con el mismo Δ llevan la misma `X`, enlazables sin enumerar nada. Es la
  D-I del RFC-0008, que por eso sacó `X` del sobre portable —«la privacidad es propiedad del
  protocolo, no una política del emisor»—, aplicada a toda la cadena de agentes (DEDUCIDO, de
  leerla; no medido aquí). Dentro del mensaje, todo eso pasa del pagador a toda la cadena de
  agentes. Cuánto más aprende un agente que ya ve la orden no está medido. Fuera del mensaje no hay
  observadores nuevos. Lo de `x` es más que el vínculo que la D-4 declaró: cuando se mida, es su
  condición de reapertura; si un v2 puede ir en claro dentro del mensaje es la pregunta 2.
- **En claro, el aviso no tiene integridad.** Un agente puede quitar la hoja `x` —su presencia
  decide (D-3)— y dejar un v2 como v1, o cambiar una hoja: el cobro sale RJCT, el dinero no se
  mueve y, si nadie reenvía el aviso por fuera, el pendiente revierte al emisor cuando venza
  (DEDUCIDO). La vía fuera del mensaje no pasa por los agentes; dentro, solo la daría el aviso
  cifrado con el AEAD y una clave que el agente no tiene, la vía (ii) de la D-4 (DEDUCIDO).
- **El residuo del pagador sigue** (`two_phase.rs:32-40`): eligió la sal, y cifrar el aviso no se
  lo esconde. Cerrarlo es cambiar el compromiso, no el canal.
- **El límite de `claim_pacs008`.** Copia los identificadores de `previous` sin comprobar que el
  aviso sea de ese pago (`iso.rs:539-542`): un ACSC puede nombrar el `EndToEndId` de otro pago. No
  crea saldo ni gasta sin clave; es un límite del puente, que su documentación declara desde el
  §861 (`iso.rs:530`). La vía nueva lo acota, no lo cierra: el test 5 de la E1 ata el `ACSC` a la
  orden que trae el aviso y a su importe, pero nada ata el pendiente a la orden —el compromiso es
  `M(M(receiver_id, salt), amount)`, sin el `EndToEndId` (`pending.rs:70`; `two_phase.rs:925`)— y
  el aviso en claro no va autenticado. Si el pagador o un agente pone el aviso del pendiente A en
  la orden B, con el mismo importe y el mismo acreedor, el `ACSC` nombra B por el dinero de A
  (DEDUCIDO). ⚠️ El §861 escribió en `iso.rs:530` que la vía por materiales «lo cerrará», y lleva
  su marca (§827).
- **El §106**: nada de este RFC añade una familia de supuestos. El cifrado queda condicionado a la
  enmienda que solo da el autor, o a la vía que no la necesita.

## Lo que este RFC no hace

No toca código, ni el cable, ni un circuito: el prototipo es la E1, y su sello solo marca, en su
misma línea, el comentario de `iso.rs:530`. No elige entre las dos vías del cifrado, que es el
B11.2, ni enmienda el §106, ni decide si un v2 va en claro dentro del mensaje (la pregunta 2). No
decide la clave del aviso cifrado: la decidió la D-9 de la 52 (`doc/ROTACION_DE_CUENTAS.md` §6), en
el §860. No coteja el XSD original, no lee las reglas de uso de ninguna infraestructura y no mide
el tamaño de ningún KEM. No construye el
camt.054: el árbol no tiene camt.05x (`iso.rs:47`). No toca `nucleo/` ni `doc/historia/VISION.md`.
No cierra la 11, y no mueve a ✅ «Transición ordenada desde infraestructuras tradicionales»
(`PRINCIPIOS.md` §3), que sigue en ❌ hasta que el prototipo exista: esta decisión le da una vía, no
la cierra.

## Referencias

- `BACKLOG.md`, entradas 11, 52 y 122.
- `AUDITORIA.md` §21, §39, §106, §268, §609, §640, §642, §662, §705, §791, §799, §809, §835, §838,
  §860 y §861.
- `doc/AVISO_EN_ISO20022.md` §2, §5, §6, §7, §8 y §9; `doc/ROTACION_DE_CUENTAS.md` §6;
  `doc/CONFIANZA_RESIDUAL.md` (el B11.2).
- `spec/rfc/PROCESO.md` §«Reglas»; RFC-0003, D2; RFC-0005, D-A; RFC-0008, D-I; RFC-0016; RFC-0021,
  D-A y D-B.
- `PRINCIPIOS.md` §3, §4.2 y §9; `SECURITY.md` §2.ter; `tools/check_curvas.py` (R4 y R5, §838);
  `spec/RPC.md` §«Lectura».
- ISO 20022: pain.001.001.09, pacs.008.001.08 y camt.054.001.08, leídos en los modelos derivados
  de `pyiso20022` 1.6.2, no en el XSD.
