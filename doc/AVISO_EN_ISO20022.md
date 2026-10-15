<!--
  AVISO_EN_ISO20022 — por dónde viajaría el aviso del pendiente dentro de un mensaje ISO 20022.
  Nota de diseño para la entrada 11 del BACKLOG, ACEPTADA como análisis en el §799 (⚠️ §799: hasta
  aquí decía «en BORRADOR»), con su otra cara: el B11.2 de CONFIANZA_RESIDUAL.md, el aviso
  cifrado. Hermana de CADUCIDAD_PENDIENTE.md y de ROTACION_DE_CUENTAS.md, a la que le añade un
  caso. No toca código, ni el cable, ni un circuito: mide los campos que podrían llevarlo y deja
  las decisiones al autor. ⚠️ §861: las seis decisiones de su §8 se tomaron por delegación del autor,
  y lo que dice el §8 desde ahí es lo decidido; el análisis no cambia.
-->

# AVISO_EN_ISO20022 — el aviso del pendiente, dentro o fuera del mensaje

**Estado**: ACEPTADA (§799), como análisis: el autor la acepta como está y deja abiertas sus
decisiones, de la D-1 a la D-6 del §8, sin elegir ninguna opción. ⚠️ §799: hasta aquí decía
«BORRADOR». Nota de diseño, sin código y sin número de RFC. Elegir el campo, su codificación y su
orden frente al cifrado del aviso es del autor (§8). No cierra la 11 ni el B11.2.
⚠️ §861: **decisiones tomadas por delegación del autor** («resuelvelas aplicando los principios»), con
el criterio del §609 y el precedente del §642; REVERSIBLES; entran con su aceptación. Son las seis
del §8, cada una debajo de su pregunta, con la tabla de cuándo se reabre; la nota del `BACKLOG.md`,
en la entrada 11. Hasta aquí este párrafo decía que las seis quedaban abiertas, sin elegir ninguna
opción, y que elegir era del autor: lo era hasta el §861. Sigue sin cerrar la 11 ni el B11.2.

**Procedencia**: la escribió Claude en una sesión de Claude Code en la
nube, sobre `8c02941` (§789), fuera del paso 4 de [`GENAI.md`](../GENAI.md); la lleva el §791 de
`AUDITORIA.md`. Integrar no es aceptar: la aceptación quedó pendiente hasta el §799, en el que el
autor la acepta como análisis, con sus decisiones abiertas. ⚠️ §799: hasta aquí decía «la
aceptación queda pendiente». ⚠️ §792: sus citas al `BACKLOG.md` por línea pasaron a citas por
entrada, la regla v5 de `tools/verificar_citas.py`.

**Convención**: ✅ medido en este árbol, con fichero:línea · 📐 derivado con una aritmética que se
reproduce · 📖 leído fuera del árbol, con su fuente y su fecha · ⚠️ matiz · ❓ no medido.

---

## 0. El problema, con su acta

La vía de dos fases deja el dinero en un pendiente, y el receptor solo lo cobra si le llega el
aviso, `PendingNotice { position, salt, amount, x }` ✅ (`crates/zk-ssl/src/two_phase.rs:62-74`): la
posición del compromiso en el árbol de pendientes, el aleatorio que eligió el pagador, el importe y,
en el compromiso v2, el sobre de reversión `X`. Sin él no se reconstruye el compromiso, y perderlo
es no poder cobrar aunque el dinero esté ahí (`two_phase.rs:23-30`).

El puente ISO lo devuelve **aparte** del informe: `settle_pacs008_two_phase` da
`(Pacs002, Option<PendingNotice>)` ✅ (`crates/zk-ssl/src/iso.rs:425-432` y `:493`), y
`claim_pacs008` lo recibe como un parámetro más (`iso.rs:531-538`). Dos comentarios del puente dicen
por qué (`iso.rs:484-491` y `:528-530`), y `doc/historia/VISION.md:679-682` lo repite:

> ISO 20022 no tiene campo donde llevarlos, y meterlos en el de información de remesa sería
> forzarlo.

`spec/RPC.md` §«Pago en dos fases» (⚠️ §861: hasta aquí citaba líneas, `:132-134`) dice lo mismo del
cable: el aviso «viaja fuera de banda del pagador al receptor (ISO 20022 no lo transporta; AUDITORIA
§21)». ⚠️ §861: la línea lleva desde el §861 su marca, que corrige las dos cosas: la cita es el §39, y
el estándar no tiene un campo hecho para el aviso, pero el de extensión lo admite (§3). La 11 lo
dejó como constatación de ausencia (la entrada 11 del `BACKLOG.md`), y el §268 la juntó, en la misma
entrada, con el B11.2. Esta nota hace lo que la 11 dejó para cuando hubiera canal que analizar:
**medir qué campos del pacs.008 y del pacs.002 pueden llevar el aviso**, contra el esquema y no
contra el recuerdo.

⚠️ **Dos rancios de paso, que esta nota no corrige.** (⚠️ §861: los corrige la D-6, con marca donde
viven: `spec/RPC.md` y el doc de `settle_pacs008_two_phase`.) La cita de `spec/RPC.md` §«Pago en dos
fases» (⚠️ §861: hasta aquí citaba líneas, `:134`) no sostiene lo que cita: el §21
(`AUDITORIA.md:1844`) es el de los códigos de motivo contra el catálogo y no habla del aviso; el que
lo dice es el §39, que llama al canal «lateral» y «sin especificar» (`AUDITORIA.md:3566-3568`). Y el
doc de `settle_pacs008_two_phase` dice en «Lo que falta» que la segunda fase «no está implementada»
(`iso.rs:421-424`), cuando `claim_pacs008` la implementa (`iso.rs:531`) y
`the_full_two_phase_iso_cycle` la ejerce (`iso.rs:975`).

## 1. Lo que el árbol sabe del estándar, y lo que no

✅ **Del esquema, nada.** El puente «no es un parser XML» (`iso.rs:34-37`): `Pacs008` son seis campos
—`msg_id`, `end_to_end_id`, los dos IBAN, `amount_minor` y `currency`— (`iso.rs:61-75`), y
`Pacs002`, ocho (`iso.rs:117-129`). Ningún fichero del árbol nombra `SplmtryData`, `RmtInf` ni
`Ustrd`: `git grep` de los tres nombres sobre `8c02941` sale vacío; fuera de esta nota, solo los
nombran las notas que la enlazan. Lo que el código comprueba del estándar son los `Max35Text` de
`MsgId` y `EndToEndId` (`iso.rs:311-339`) y los códigos de estado y de motivo (`iso.rs:77-107` y
`:131-223`).

📖 **Lo que se leyó fuera, y cómo.** El sitio de ISO 20022 no era alcanzable desde la sesión: el
proxy devolvió 403. Se leyeron, el 14-10-2026, los modelos que `xsdata` genera desde los XSD
publicados, en el paquete `pyiso20022` 1.6.2 de PyPI (wheel `pyiso20022-1.6.2-py3-none-any.whl`,
sha256 `98693c96b245541141224c4f1a52e913485bc4cf3d2376101d4de60f9b1c3a50`, descargado sin instalarlo
ni ejecutarlo): `pyiso20022/pacs/pacs_008_001_08/pacs_008_001_08.py` y
`pyiso20022/pacs/pacs_002_001_10/pacs_002_001_10.py`; las líneas que se citan abajo son de esos dos
ficheros. Es **una copia derivada**, no el XSD: la cardinalidad se lee en la convención de `xsdata`
—una lista sin `max_occurs` es `maxOccurs="unbounded"`, un campo con `default=None` es opcional—, y
cotejarla con el esquema original es un pendiente declarado (§9). Se eligieron `pacs.008.001.08` y
`pacs.002.001.10` porque van juntos en una misma entrega del catálogo; el árbol no fija versión, y
qué versión hablaría un piloto queda sin medir ❓.

❓ **Lo que no se leyó**: las reglas de uso de cada infraestructura (SEPA, CBPR+, TARGET2…), que
restringen lo que el esquema permite —cuántas ocurrencias de `Ustrd`, qué juego de caracteres, si
admiten `SplmtryData`—. Nada de lo que abajo «cabe» vale todavía para ellas.

## 2. Los campos candidatos, medidos

📐 **Lo que ocupa el aviso.** El cable de la casa lo escribe `{position: Q, salt: B32, amount: Q}`
(`crates/zk-ssl-wire/src/lib.rs:266-272`), con `Q` en hexadecimal `0x…` (`lib.rs:82-88`) y `B32`
como `0x` y 64 cifras (`lib.rs:61-68` y `:105-111`). En JSON compacto, un aviso v1 ocupa **de 109 a
139 caracteres** (posición e importe de una cifra hexadecimal a dieciséis), y el v2 con `x`, como el
fichero del CLI (`crates/zk-ssl-cli/src/cobro.rs:34-44`), **de 182 a 212**. En bytes canónicos —8 de
posición, 32 de sal, 8 de importe y 32 de sobre— son 48 y 80, que en base64 dan **64 y 108
caracteres**. Se reproduce con `json.dumps(…, separators=(",", ":"))` y `base64.b64encode` sobre los
extremos.

⚠️ **El aviso del cable no lleva `x`.** `PendingNoticeDto` son tres campos y lo pone a `None` al
leer, «el 0.2 no transporta el sobre» (`zk-ssl-wire/src/lib.rs:266-272` y `:284-294`), y el puente
ISO solo produce avisos v1, con `x: None` (`two_phase.rs:931-936`). El sobre sí cruza el cable, pero
suelto y hacia el nodo: `zkssl_pendingPath` pide las cuatro piezas, `x` incluida
(`spec/RPC.md:98`). Como aviso del pagador al receptor, un v2 dentro de un mensaje ISO sería el
primer mensaje que lo lleva entero.

| | campo (ruta) | 📖 tipo y cardinalidad | v1 en claro | v2 en claro | va hacia | lo que su nombre dice |
|---|---|---|---|---|---|---|
| A | pacs.008 `CdtTrfTxInf/SplmtryData` | `SupplementaryData1`, 0..n: `PlcAndNm` 0..1 `Max350Text` y `Envlp` 1..1 con un elemento cualquiera, `##any` (`pacs_008_001_08.py:780-787`, `:1496-1513`, `:3515-3522`); también en la raíz (`:3548-3554`) | sí | sí | el acreedor: es la orden | datos suplementarios, el mecanismo de extensión del esquema |
| B | pacs.008 `CdtTrfTxInf/RmtInf/Ustrd` | `Max140Text`, 0..n, dentro de `RmtInf` 0..1 (`:3123-3141`, `:3507-3514`) | sí, en una ocurrencia | en JSON, dos ocurrencias; en base64, una | el acreedor | información de remesa, sin estructura |
| C | pacs.008 `CdtTrfTxInf/RmtInf/Strd/AddtlRmtInf` | `Max140Text`, 0..3 (`:3110-3118`) | sí | en JSON, dos de las tres; en base64, una | el acreedor | información de remesa adicional, estructurada |
| D | pacs.008 `CdtTrfTxInf/InstrForCdtrAgt/InstrInf` | `Max140Text` en `InstructionForCreditorAgent1`, 0..n (`:1305-1322`, `:3458-3464`) | sí | en base64, sí | el agente del acreedor | instrucción para el agente del acreedor |
| E | pacs.002 `TxInfAndSts/SplmtryData`, y el de la raíz | `SupplementaryData1`, 0..n (`pacs_002_001_10.py:748-755`, `:1433-1450`, `:3524-3530`, `:3563-3569`) | sí | sí | **de vuelta** a quien mandó la orden | datos suplementarios, en el informe de estado |
| F | pacs.002 `TxInfAndSts/StsRsnInf/AddtlInf` | `Max105Text`, 0..n (`:2811-2836`, `:3448-3454`) | en JSON, dos ocurrencias (109 > 105); en base64, una | en base64, dos ocurrencias (108 > 105); en JSON, hasta tres (212 > 210) | de vuelta | información adicional del motivo del estado |
| G | pacs.002 `TxInfAndSts/OrgnlTxRef/RmtInf` | la remesa de la orden original, copiada (`:3200`, `:3280-3287`) | como B | como B | de vuelta | la remesa original, devuelta |

La columna «va hacia» no es del esquema: es del árbol. El pacs.002 es la respuesta al pacs.008 que
lleva su `MsgId` y su `EndToEndId` (`iso.rs:8-19` y `:117-121`); vuelve a quien envió la orden.

## 3. Las dos frases del árbol, contra la medida

1. **«ISO 20022 no tiene campo donde llevarlos»** (`iso.rs:487` y `:528-529`;
   `doc/historia/VISION.md:680-681`; `spec/RPC.md` §«Pago en dos fases»; ⚠️ §861: hasta aquí citaba
   líneas, `:133`). 📖 Leída contra el modelo derivado, **no es exacta**: los dos mensajes tienen
   `SplmtryData`, cuyo sobre admite un elemento de cualquier espacio de nombres, en la transacción y
   en la raíz (filas A y E). Lo que sí es cierto es más estrecho: no hay un campo **hecho para** un
   aviso de pendiente, y el de extensión vale lo que la infraestructura le deje valer (❓, §1).
   Corregir la frase en el código, en `spec/RPC.md` o en la historia es del autor (D-6): esta nota
   no edita ni el código ni la spec. ⚠️ §861: la D-6 se tomó por delegación: la frase de arriba se
   corrige, con marca, en el código y en la spec, y en los documentos vivos que la repetían; la
   historia y los asientos, no.
2. **«meterlos en el de información de remesa sería forzarlo»**. 📐 **No es un problema de tamaño**:
   el v1 cabe en un solo `Ustrd`, y en base64 también el v2 (§2). Es un juicio sobre **qué es** el
   campo —su nombre dice remesa, lo que el pago liquida— y sobre **quién lo lee**. El juicio no se
   mide; se deja con su razón para que el autor lo ratifique o lo retire. ⚠️ §861: ratificado por
   delegación, en la D-1: la remesa se descarta por lo que el campo significa, no por su tamaño.

## 4. Quién produce el aviso y quién lo necesita

✅ **El productor depende de la vía.** En el puente, el aviso nace **dentro** de la liquidación: la
posición la asigna la capa al enviar (`two_phase.rs:875`), la sal la trae el banco del deudor como
parámetro (`iso.rs:418-419` y `:431`) y el importe sale del mensaje, así que el aviso existe después
de procesar el pacs.008 entrante, no antes (`iso.rs:449-493`). Por el cable, en cambio, el pagador
lo tiene **antes** de enviar: `zkssl_sendMaterials` le da la posición reservada (`pending_position`
en `zk-ssl-wire/src/lib.rs:299-310`; `spec/RPC.md` §«Pago en dos fases»; ⚠️ §861: hasta aquí citaba
líneas, `:121`), y la sal y el importe son suyos.

✅ **El consumidor es el lado del acreedor.** `claim_pacs008` recibe la clave y el estado del
receptor (`iso.rs:531-538`), que en el modelo del puente vienen del banco del acreedor
(`iso.rs:523-526`), igual que la clave del pagador viene de un almacén aparte del banco del deudor
(`iso.rs:42-45` y `:410-416`).

📐 **De ahí la dirección.** El aviso tiene que ir hacia el acreedor, que es la dirección del
pacs.008, y el pacs.002 va de vuelta. Por eso:

- en el puente tal como está, el aviso no puede ir en el pacs.008 **entrante**, porque todavía no
  existe, y el puente no **emite** ningún pacs.008: `Pacs008` solo se lee (`iso.rs:61-75` y
  `:425-432`). Llevarlo dentro exige que el puente reenvíe la orden al lado del acreedor con el
  aviso puesto, o que el aviso nazca antes, como en el cable;
- meterlo en el pacs.002 (filas E a G) es el cambio más pequeño a la forma del puente —el par de
  `iso.rs:432` se hace un campo—, pero lo deja **donde ya estaba**: en el lado del deudor, que lo
  conoce entero porque eligió la sal (`two_phase.rs:32-36`) y que tendría que hacerlo llegar al
  acreedor igual que hoy.

## 5. Lo que el aviso expone dentro del mensaje, y a quién

✅ **Conocer el aviso no permite cobrar.** Con el aviso completo y sin la clave del receptor, el
cobro se rechaza: `a_third_party_cannot_claim_the_pending` (`iso.rs:1029-1067`). Es el arreglo del
§39 (`AUDITORIA.md:3600`): el cobro ata la clave a la identidad del compromiso
(`two_phase.rs:1231`).

📐 **Lo que da es el vínculo.** En el pacs.008 las partes y el importe ya viajan en claro —`Dbtr`,
`Cdtr` e `IntrBkSttlmAmt` son obligatorios en la transacción (📖 `pacs_008_001_08.py:3164`, `:3388`,
`:3436`)—, así que el aviso le añade dos cosas a quien lea el mensaje:

- **la posición**, que es la clave que une un envío con su cobro: `spec/RPC.md` §«Qué ve exactamente
  un agregador» (⚠️ §861: hasta aquí citaba líneas, `:344-347`) lo mide para el agregador que ve las
  dos mitades, y con el aviso en el mensaje la tiene cualquier agente que lo lea;
- **la sal**, que con la identidad del receptor y el importe recompone el compromiso
  (`two_phase.rs:925`) y deja ver **cuándo** desaparece del árbol: el residuo que
  `two_phase.rs:32-40` declara para el pagador pasa a ser de toda la cadena de agentes.

❓ Cuánto aprende de eso un agente que ya ve la orden no está medido: es la fuga de metadatos de
`CONFIANZA_RESIDUAL.md` §3.4, con más observadores.

## 6. La relación con el B11.2, el aviso cifrado

El B11.2 pide el «cifrado del aviso a la clave del receptor (AEAD + clave efímera, ~100 B de
sobrecoste)», con el patrón de Sapling (`doc/CONFIANZA_RESIDUAL.md:315-318`); lo declara
independiente e implementable hoy (`:541-543` y `:562`), y el §268 lo dejó con esta entrada
(`AUDITORIA.md:20214-20222`). Medido contra el árbol de hoy, tiene cuatro condiciones que tocan al
canal:

1. ✅ **Ninguna cuenta tiene una clave a la que cifrar.** `AccountRecord` guarda identidad, saldo,
   nonce, `view_id` y sal de la hoja (`crates/zk-ssl/src/lib.rs:752-767`; ⚠️ §861: decía `:740-755`),
   y abrir por el cable manda `{publicId, viewId, leafSalt}` (`spec/RPC.md` §«Apertura»; ⚠️ §861:
   hasta aquí citaba líneas, `:111`): tres resúmenes y ninguna clave pública. Cifrar «a la clave del
   receptor» exige **una clave nueva por cuenta**, registrada al abrir, y con ella un caso más para
   la rotación: el sexto de [`ROTACION_DE_CUENTAS.md`](ROTACION_DE_CUENTAS.md) §2.
2. ✅ **Una clave efímera sobre una curva no entra en el SDK.** La casa cifra con XChaCha20-Poly1305
   y nonce de 24 bytes (`crates/zk-ssl/src/crypto.rs:39-50`), y en el árbol no hay ningún acuerdo de
   claves. El B11.2 habla de «clave efímera» sin decir sobre qué grupo, y el árbol tampoco lo dice
   de Sapling (📖 no se leyó aquí). Lo medido es esto: desde el §788, `tools/check_curvas.py` rechaza
   las bibliotecas de curvas —`x25519-dalek` y `curve25519-dalek` entre ellas— en la clausura del
   nodo, el testigo, el SDK y el kit (`tools/check_curvas.py:14-21` y `:44-50`), así que un
   Diffie-Hellman sobre una curva no pasaría esa puerta en el SDK. ⚠️ §835: decía `:10-14` y
   `:26-30`; la marca del §835 en esa cabecera movió dos líneas. Y decía que un Diffie-Hellman sobre
   una curva no pasa la puerta en el SDK. `ring`, que ya está en el SDK por el TLS de `ureq`, trae
   X25519 y ECDH P-256/P-384, y no está en el patrón (la 122 de `BACKLOG.md`): una dependencia
   directa de `ring` en el SDK sale VERDE, medido en el §835 sobre una copia. Y ni siquiera hace
   falta: `ureq` reexporta `rustls`, y el SDK puede hacer hoy un X25519 con
   `ureq::rustls::crypto::ring`, sin tocar `Cargo.toml` ni `Cargo.lock`, medido en el §835 con un
   crate aparte. Ninguna puerta que mire nombres de crate lo cierra mientras `ureq` lleve `tls`. El
   título de este punto vale para las bibliotecas del patrón, no para `ring`. Y «ningún acuerdo de
   claves» vale para el código de la casa: el TLS de `rustls` hace uno en cada conexión `https`.
   ⚠️ §838: decía `:12-16` y `:28-32`; la cabecera creció con dos marcas, R4 y R5. Y lo que la marca
   del §835 dice de `ring` ya no vale: `ring`, `rustls`, `rustls-webpki` y `aws-lc-*` están en el
   patrón; R4 admite `ring` solo bajo `rustls` y `ureq`, y solo en el cli y el SDK; y R5 da ROJO a
   `ureq::rustls`, `rustls::`, `ring::` o `webpki::` en las fuentes de la casa (`:26-31`). Una
   dependencia directa de `ring` en el SDK, o un `use ureq::rustls::crypto::ring` en el cli, salen
   ROJO, medido en el §838 sobre copias del árbol, y el título de este punto vuelve a valer para el
   código de la casa con las bibliotecas del patrón (OpenSSL, otra pila TLS, no está en él). El
   acuerdo de claves del TLS sigue: es la excepción del §106 para el transporte (§838). Uno
   poscuántico, un KEM, no está en el árbol —el grep de «ML-KEM» y de «Kyber» sale vacío— y **su
   tamaño no está medido aquí**: los ~100 B del B11.2 no se pueden dar por buenos para él.
3. 📐 **Cifrado, solo cabe entero en la extensión.** Sin contar el KEM, el AEAD de la casa añade
   40 bytes, 24 de nonce y 16 de etiqueta: 88 y 120 bytes para v1 y v2, que en base64 son 120 y 160
   caracteres. El v2 ya no cabe en un campo de 140 (filas B a D), y con el encapsulado del KEM no
   cabe ninguno de los dos; solo `SplmtryData` (filas A y E) lo lleva en un elemento. En `Ustrd`,
   que el esquema deja repetir sin tope, habría que trocearlo, y cuántas ocurrencias admite cada
   infraestructura no está medido (❓, §1).
4. ✅ **Cifrar no esconde el aviso del pagador**, que eligió la sal (`iso.rs:418-419`). El residuo de
   `two_phase.rs:32-40` se cierra, según ese mismo comentario, como lo cierra Zcash: que sea el
   receptor quien derive el aleatorio. Eso es un cambio del compromiso, no del canal.

📐 **Y el orden importa.** Si el aviso viaja primero en claro y después cifrado, el mismo campo
cambia de contenido: lo que se elija tiene que poder decir su versión —`PlcAndNm`, o una etiqueta
dentro del sobre— o el cifrado será una segunda ruptura del formato.

## 7. Las opciones, sin elegir

⚠️ §861: «sin elegir» valía hasta aquí: las elige el §8, por delegación del autor (D-1).

| opción | qué cambia en el código | v1 / v2 / cifrado | va hacia | depende de |
|---|---|---|---|---|
| **A.** `SplmtryData` del pacs.008, con un espacio de nombres propio | `Pacs008` gana un campo opcional (`iso.rs:61-75`); el puente acepta el aviso en la orden entrante (vía del cable, §4) o emite la orden hacia el acreedor, que hoy no emite; el esquema del elemento, con su versión | sí / sí / sí | el acreedor | ❓ que la infraestructura admita la extensión |
| **B.** `RmtInf` (`Ustrd` o `Strd/AddtlRmtInf`), con codificación compacta | lo de A, más la codificación: base64 de los bytes canónicos y un prefijo que la distinga de una remesa de verdad | sí / en base64 / no | el acreedor | ❓ juego de caracteres y ocurrencias de las reglas de uso; el juicio del §3.2 |
| **C.** `InstrForCdtrAgt/InstrInf` | como B | sí / en base64 / no | el agente del acreedor | ❓ como B |
| **D.** `SplmtryData` del pacs.002 | `Pacs002` gana el aviso y el par de `iso.rs:432` se hace un campo | sí / sí / sí | **de vuelta al deudor**: no resuelve la entrega | — |
| **E.** Fuera del mensaje, especificado | nada en `iso.rs`; un documento del canal: quién lo manda, por dónde y con qué formato | lo que el canal elija | lo que el canal elija | ❓ el canal: `iso.rs:47` dice que no hay `camt.05x`, y un aviso al acreedor por ese lado no está mirado (⚠️ §861: ya está mirado, abajo: el camt.054 lleva `SplmtryData` en la transacción) |
| **A + B.** ⚠️ §861: la combinación, añadida y descartada: A, con un `Ustrd` compacto de reserva | lo de A y lo de B a la vez | sí / en base64 / no | el acreedor | descartada: fuerza el campo de remesa y añade una segunda escritura del mismo aviso; la reserva, si se poda `SplmtryData`, es E con el mismo elemento |

Lo que tienen en común: en las cinco (⚠️ §861: y en la combinación A + B, añadida a la tabla) **el
aviso deja de ser solo un valor de Rust**, y eso cambia lo que el comentario de `iso.rs:490-492`
quería —que la firma recordase a quien use el puente que la entrega es cosa suya—. De la A a la D
la resuelve el mensaje; en la E, un documento.

⚠️ §861: **el salto al titular.** La fila A llega al agente del acreedor, no al titular. Del banco del
acreedor a su cliente, el mismo elemento cabe en `Ntry/NtryDtls/TxDtls/SplmtryData` del camt.054:
📖 `EntryTransaction10` lleva `SplmtryData`, una lista
(`pyiso20022/camt/camt_054_001_08/camt_054_001_08.py:5997` y `:6201-6208`, de la misma rueda, con la
misma suma), y lleva también `RmtInf` y un `AddtlTxInf` de 500 caracteres; leído en el modelo
derivado, no en el XSD. Si no, viaja fuera del mensaje. Del lado del pagador, el pain.001 lleva el
mismo `CdtTrfTxInf/SplmtryData` (📖 `pyiso20022/pain/pain_001_001_09/pain_001_001_09.py:3139` y
`:3344`). El árbol no tiene camt.05x (`iso.rs:47`), y nada de esto se construye.

## 8. Decisiones (por delegación del autor, §861, REVERSIBLES)

⚠️ §861: hasta aquí este título decía «Decisiones abiertas para el autor». Las seis preguntas se quedan
como estaban, y cada una lleva debajo lo decidido, por delegación del autor («resuelvelas aplicando
los principios»), con el criterio del §609 y el precedente del §642; REVERSIBLES; entran con su
aceptación. Un formato y dos portadores. La tabla de cuándo se reabre, al final.

- **D-1. El campo**: A, B, C, D o E, o una combinación (en claro en uno y cifrado en otro).
  **Decidida (§861)**: dentro del mensaje, A, `CdtTrfTxInf/SplmtryData` del pacs.008; del banco del
  acreedor al titular, el mismo elemento en `TxDtls/SplmtryData` del camt.054 (§7); fuera del
  mensaje, el mismo elemento (E). Nace en la vía por materiales, antes de la orden
  (`zkssl_sendMaterials` da la posición; la sal y el importe son del pagador), y lo escribe el lado
  del pagador: el pagador en su pain.001 y su banco lo lleva al pacs.008. El operador no escribe
  nada, y el puente de en medio, el que mete la clave en la capa, no es el modelo y no gana un
  emisor de pacs.008. Descartadas: B, por lo que el campo significa (§3.2), y porque cifrado no
  cabe en 140 caracteres; C, porque su semántica es una instrucción al agente, y cifrado no cabe;
  D, porque va de vuelta; y la combinación de A con B, por §7.
- **D-2. La codificación**: el JSON del cable (`zk-ssl-wire/src/lib.rs:266-272`), los bytes
  canónicos en base64, o un elemento XML propio con su espacio de nombres.
  **Decidida (§861)**: un elemento XML propio, con espacio de nombres versionado, en el `Envlp`. Sus
  hojas usan la escritura del cable: `Q` canónico para la posición y el importe, `B32` canónico
  para la sal y `x`; sin base64 y sin JSON dentro del XML. La conformidad es del infoset —nombres y
  texto de las hojas—, no de los bytes, porque un agente puede reserializar el XML: el KAT es un
  infoset más una serialización de referencia, y el lector conforme usa un analizador XML de
  verdad, fuera del puente, y rechaza una hoja que no sea canónica. El contenedor es nuevo —el
  tercero, junto a `PendingNoticeDto` y `AvisoV2`—, y el RFC lo dice; la escritura de los valores,
  no.
- **D-3. v1 o v2**: si viaja el sobre `x`. El puente produce v1 (`two_phase.rs:931-936`), el aviso
  del cable no lo lleva (`zk-ssl-wire/src/lib.rs:266-272` y `:291`), `zkssl_pendingPath` sí lo pide
  (`spec/RPC.md:98`) y el fichero del CLI ya lleva el v2 (`cobro.rs:34-44`).
  **Decidida (§861)**: los dos. La presencia de la hoja `x` decide entre v1 y v2, como
  `PendingNotice.x`; el espacio de nombres solo dice el formato —en claro o, cuando exista,
  cifrado—, y uno desconocido se rechaza. Una sola señal para cada hecho.
- **D-4. El orden frente al B11.2**: en claro ya, con la versión en el formato, o esperar al cifrado
  y a la clave por cuenta que exige (§6.1), que es diseño de la rotación.
  **Decidida (§861)**: no se espera al cifrado. En claro **dentro** del mensaje es una elección del
  pagador, no lo que el estándar da por defecto: la vía fuera del mensaje, con el mismo elemento,
  no concede observación («Conceder observación que no hace falta se defiende mal después»,
  `SECURITY.md` §2.ter). La fuga se dice: posición y sal para toda la cadena de agentes, sin poder
  de cobro (§5). La versión en claro entra con su retirada escrita desde el principio: cuando
  exista la cifrada, la en claro dentro del mensaje pasa a desaconsejada, y sus vectores se mudan
  (R3 y R6 b del RFC-0021). El cifrado necesita una de dos cosas: (i) una enmienda del §106, que
  solo puede dar el autor —un KEM de retículos añade familia de supuestos, y un Diffie-Hellman
  sobre una curva lo prohíbe el §106—, o (ii) la vía simétrica, con el XChaCha20-Poly1305 de la
  casa y una clave que el receptor entrega con su dirección, una por relación, derivada de `sk`
  con dominio propio (la D-9 de la 52 del `BACKLOG.md`): no añade familia ni primitiva, y no
  necesita la clave pública por cuenta de §6.1. Elegir es del B11.2; los principios inclinan a la
  (ii). Con una curva no, aunque `ring` ya esté en el SDK (la 122 del `BACKLOG.md`): lo prohíbe el
  §106, no la puerta.
- **D-5. Si es protocolo.** `spec/rfc/PROCESO.md:3-5` llama protocolo a lo que cruza el cable
  —`spec/RPC.md`, `spec/openrpc.json` y los vectores de `spec/vectors/`—, y un campo de un mensaje
  ISO no está en esa lista. Si un formato de interoperabilidad bancaria entra por RFC, con vectores,
  o como documentación del puente, el proceso no lo dice.
  **Decidida (§861)**: sí, por RFC, con el número libre al entrar, PROPUESTO. Declara su
  compatibilidad —aditivo, R2; el cable S1 no se mueve— y su efecto sobre «la clave de gasto no
  viaja jamás» (`spec/rfc/PROCESO.md`, regla 3): ninguno, porque el elemento no lleva clave y la vía
  que especifica es la de materiales. Su KAT va en una familia propia, `spec/vectors/aviso/`, con
  su `MANIFIESTO.txt`, su línea en `HUELLAS.sha256` y su lector en `tools/segunda/`, no en
  `nucleo/`, y entra solo después de cotejar las filas A y E y el camt.054 con el XSD original y
  con las reglas de uso públicas de al menos una práctica de mercado. Las dos son lecturas, no
  actos ante terceros; hoy las dos dan 403. El sello que lo entre pone al día también el bloque
  `superficie` de `spec/RPC.md` §«Versionado y compatibilidad» —el `manifiesto` nuevo,
  `vectores.json` y `vectores.lineas`—, como hizo el §809.
- **D-6. Las frases del §3**: si se corrigen con su nota en `iso.rs:484-491` y `:528-530`, en
  `spec/RPC.md` §«Pago en dos fases» (con la cita al §21 del §0) y en la historia, o se quedan
  como están con esta nota al lado. ⚠️ §861: hasta aquí citaba líneas, `:132-134`.
  **Decidida (§861)**: se corrigen, con marca, donde viven. La frase nueva: no hay un campo hecho
  para el aviso; el de extensión, `SplmtryData`, lo admite (leído en un modelo derivado, sin
  cotejar con el XSD); usar el de remesa sería forzarlo. La cita al §21, que trata los códigos de
  motivo, pasa al §39. El sello de esta decisión lo hace en los comentarios de `iso.rs`, de
  `client.rs` y del SDK —también en el ejemplo de su cabecera, que solo cita el §21—, en
  `spec/RPC.md`, `ARQUITECTURA.md`, los dos `PAPER` y `doc/DIAGNOSTICO_ESCALADO.md`. Quedan, para
  otro sello, el «§21» con que nombran la deuda «aviso fuera de banda» la plantilla de los RFC y
  los RFC del 0002 al 0007, y la entrada 74 del `BACKLOG.md`, cerrada; el de
  `doc/CADUCIDAD_PENDIENTE.md:35`; y el `println!` de `crates/zk-ssl-sdk/examples/e2e.rs:25`, que
  es código. `doc/historia/`, los preprints depositados y los asientos de `AUDITORIA.md` no se
  tocan.

**Cuándo se reabre** (⚠️ §861):

| decisión | se reabre si |
|---|---|
| D-1 | el XSD original, o las reglas de uso públicas de una práctica de mercado, quitan o restringen `CdtTrfTxInf/SplmtryData` del pacs.008 o `TxDtls/SplmtryData` del camt.054; entonces se elige entre E sola y la combinación, con las reglas medidas |
| D-2 | una práctica de mercado prohíbe espacios de nombres ajenos en el `Envlp`, o lo acota en tamaño o en juego de caracteres |
| D-4 | existe la versión cifrada —y la en claro dentro del mensaje se desaconseja, por su retirada escrita—, o una medida de lo que aprende un agente da más que el vínculo declarado: entonces la en claro dentro del mensaje se retira antes del cifrado |
| D-5 | el autor prefiere documentación del puente, fuera de la promesa, o cambia la D-A del RFC-0021 |
| todas | el autor revoca cualquiera. Nada de lo que se construye es irreversible: un RFC PROPUESTO, un campo opcional, dos funciones y una del puente, con sus tests |

## 9. El siguiente paso, que necesita compilar

**El prototipo no se hace aquí**: esta sesión no compila. Lo que se escribiría, en `iso.rs` y junto
a `the_full_two_phase_iso_cycle` (`iso.rs:975-1027`):

1. un test que liquida un pacs.008, **escribe el aviso en el campo elegido** —una cadena con la
   forma de `SplmtryData` o de `Ustrd`, según D-1 y D-2—, lo **lee de vuelta solo desde esa
   cadena**, cobra con lo leído y llega a `ACSC`;
2. su negativo: el mismo campo con un byte de la sal cambiado no cobra, con el rechazo que dé la
   capa, medido y no supuesto;
3. antes de fijar el formato, el cotejo de los campos de §2 contra el XSD publicado, que esta nota
   leyó en una copia derivada.

⚠️ §861: con las decisiones del §8, es la E1 del RFC de la D-5, y va por la vía de materiales: el aviso
se escribe en la orden y el lado del acreedor lo lee solo de ella, sin que ninguna clave de gasto
llegue a la capa. Su negativo cambia un byte de la sal dejando el hex canónico, para que caiga la
apertura del compromiso y no el lector; la canonicidad del lector y la atadura entre el `ACSC` y la
orden se prueban aparte. El cotejo del punto 3 es la compuerta del KAT (D-5), no la del prototipo,
que lee solo la serialización de referencia: el puente sigue sin ser un parser XML (§1).

## 10. Lo que esta nota no hace

No toca código, ni el cable, ni un circuito; no gasta número de RFC; no elige campo; no cierra la 11
ni el B11.2; no mide las reglas de uso de ninguna infraestructura, no valida contra el XSD original
y no mide el tamaño de ningún KEM. ⚠️ §861: «no elige campo» era cierto hasta aquí: el campo lo
eligió la delegación (§8, D-1). Lo demás sigue en pie: la nota no toca código, ni el cable, ni un
circuito, ni gasta número de RFC, y el RFC del §8, D-5, lo gastará al entrar.
