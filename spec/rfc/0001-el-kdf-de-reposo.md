# RFC-0001 — El KDF de reposo: Argon2id con sal y coste, y la versión del formato

- **Estado:** PROPUESTO (§702) — con sus tres etapas construidas en el mismo sello. El paso a
  ACEPTADO es del autor. La regla 4 del PROCESO pide la spec, el OpenRPC, los vectores y las suites:
  este RFC no toca el cable, así que ni `spec/RPC.md`, ni `spec/openrpc.json`, ni `spec/vectors/` se
  mueven; las suites de los crates que toca salieron verdes en el §702, y la decisión no está tomada.
- **Autor:** Ángel José Toranzo Portela
- **Asistencia GenAI:** Claude Opus 5.5 (`claude-opus-5-5`, el modelo de la sesión), en una sesión
  de Claude Code en la nube (§702): midió, propuso, aplicó y comprobó, y lo commitea la sesión que
  la lanzó, fuera del paso 4 de `GENAI.md` — ver [`GENAI.md`](../../GENAI.md)
- **Fecha:** 2026-10-07
- **Versión del protocolo afectada:** ninguna. `zkssl/0.4` no se mueve: nada de esto cruza el cable.
  Cambia el formato EN REPOSO: el keystore del SDK pasa de `zkssl-keystore/1` a `zkssl-keystore/2`,
  el libro cifrado gana la cabecera `meta:kdf` y la instantánea cifrada la marca `0x02`.
- **Asiento(s) de AUDITORIA:** §702.

**Número.** El 0001 estaba reservado a esto desde el §206: la nota de numeración del RFC-0002 lo
dice, y los RFC-0003 a 0006 lo repiten. Este RFC lo ocupa.

## Estado de las etapas

| etapa | qué entrega | ¿rompe el cable? | estado |
|---|---|---|---|
| E1 — el KDF, con su versión | `zk_ssl::crypto::Kdf`: la versión 1 (SHA-256, sólo se lee) y la 2 (Argon2id con sal y coste); su cabecera de 29 bytes y su único lector, `Kdf::de_cabecera`; los dominios de cada uso; `argon2` clavada con `=0.5.3` en la capa y en ningún otro manifiesto | NO | sellada — §702 |
| E2 — el libro y su instantánea | `meta:kdf` en claro en cada lote; la clave se fija al abrir; un libro de la versión 1 se verifica y se migra a la 2 en un lote atómico; la instantánea cifrada, con la marca `0x02` y la cabecera del libro detrás; la `0x01` se sigue importando | NO | sellada — §702 |
| E3 — el keystore | `zkssl-keystore/2`, con `kdf_sal` y `kdf_coste` en claro; un `zkssl-keystore/1` abierto con su frase se reescribe en la 2, entero o nada | NO | sellada — §702 |

## Motivación

Desde el §199 el keystore del SDK guardaba la clave de gasto cifrada con XChaCha20-Poly1305 y una
clave `SHA-256(ZK-SSL-keystore-v1 ‖ frase)`; el libro cifrado de la capa, con `SHA-256(ZK-SSL-ledger-key-v1
‖ frase)`. Ninguna de las dos llevaba sal ni coste. El módulo, `SECURITY.md` y `ARQUITECTURA.md` lo
declaraban: «SHA-256 no es una función de derivación de contraseñas».

Lo que eso deja hacer a quien se lleve un keystore, un disco, una copia o una instantánea:

- **Probar frases fuera de línea a la velocidad de un hash.** Medido en el §702, en esta máquina y con
  carga: una derivación de la versión 1 cuesta unos 159 ns; una de la versión 2, unos 427 ms. Son
  unos seis órdenes de magnitud por intento.
- **Una sola tabla para todos.** Sin sal, la misma frase da la misma clave en cualquier keystore, y en
  cualquier libro: un diccionario precalculado contra el dominio sirve para todos los ficheros.

Que estuviera declarado no lo hacía inocuo: el ataque es explotable hoy sobre el keystore del SDK, y el
arreglo es pequeño.

## Diseño

### La derivación (E1)

| versión | clave | se escribe | se lee |
|---|---|---|---|
| 1 | `SHA-256(dominio_v1 ‖ frase)` | no | sí, para migrar |
| 2 | Argon2id, versión `0x13` (RFC 9106): `P` = la frase en UTF-8, `S` = la sal (16 bytes del generador del sistema), `K` vacío, `X` = `dominio_v2`, `t` = 3, `m` = 65 536 KiB, `p` = 4, `T` = 32 bytes | sí | sí |

El coste es la **segunda opción recomendada del RFC 9106 (§4)**: la que la biblioteca de Python
argon2-cffi 25.1.0 llama `RFC_9106_LOW_MEMORY` y toma por defecto. El texto del RFC no se pudo leer
desde la sesión del §702 —el proxy rechaza rfc-editor.org e ietf.org—, y la cifra sale de esa
biblioteca, que la atribuye al RFC. La hoja de OWASP pide para Argon2id, en su resumen, un mínimo de
19 MiB, `t` = 2 y `p` = 1, y ésta usa más memoria y más pasadas. La hoja se leyó de su fuente —el
Markdown de `cheatsheets/Password_Storage_Cheat_Sheet` en el repositorio `OWASP/CheatSheetSeries`,
en el commit `29994dd`—, porque el proxy también rechaza cheatsheetseries.owasp.org.

**Un coste por versión.** Un fichero de la versión 2 con otro coste no se lee, aunque sea mayor: subir
el coste es una versión nueva, no un campo libre. Así ningún fichero puede pedir al lector una
memoria arbitraria, y lo que la versión 2 promete es una sola cosa.

**El dominio es el dato asociado `X`.** Los dominios nuevos, `ZK-SSL-ledger-key-v2` y
`ZK-SSL-keystore-v2`, entran en el registro (`zk-ssl-hash`, §286). La clave del libro y la del
keystore no coinciden aunque compartan frase, sal y coste, y un test lo exige con la sal del
keystore.

**La cabecera** (29 bytes, en claro):

```text
0   versión   1 B   = 2
1   m_kib     4 B   u32 LE = 65536
5   t         4 B   u32 LE = 3
9   p         4 B   u32 LE = 4
13  sal      16 B
```

La versión 1 no tiene cabecera: su ausencia, con datos cifrados, es la versión 1. `Kdf::de_cabecera`
es el único lector, y rechaza diciendo por qué otro largo, otra versión u otro coste; el keystore
recompone la cabecera de su JSON y la lee con él.

**La dependencia.** `argon2` de RustCrypto, en la capa y en ningún otro manifiesto, con
`=0.5.3` y sólo la feature `zeroize`. La memoria de Argon2id la reserva `crypto.rs` y se borra al
soltarse; con la feature `alloc` la reserva el crate y la devuelve sin borrar. La 0.6.0, la última
—del 27-08-2026—, se midió y se descartó: entra en el lock con cuatro paquetes y mueve las aristas
de `digest` 0.11.3, que está en la lista cerrada del kit (§694), así que el kit ganaría
`block-buffer` 0.12.1 sin compilar `argon2`. La 0.5.3 entra con dos, `argon2` y `base64ct`, y no
mueve ninguna arista de un paquete que ya estuviera. El kit no cambia.

### El libro y su instantánea (E2)

- `SovereignLayer::open_encrypted` recibe la clave con su frase (`LedgerKey::from_passphrase`, que ya
  no deriva nada) y la **fija** al KDF del libro: con `meta:kdf`, al suyo; sin ella y con datos, a la
  versión 1; sin datos, a una versión 2 de sal nueva. Fijada, la clave no lleva la frase.
- Cada lote (`commit`) escribe `meta:kdf` en claro junto a lo que sella: el primer lote de un libro
  nuevo la escribe con sus datos.
- **Un libro de la versión 1 se migra al abrirlo.** Primero `load` lo verifica entero con la clave
  vieja; después cada valor sellado se abre con ella y se vuelve a sellar con una de la versión 2, y
  la cabecera se escribe, todo en un lote: sled lo aplica entero o no lo aplica. Van en claro sólo
  `meta:geometry_v7`, `meta:migrated` y `meta:kdf`; un valor que no abre con la clave vieja para la
  migración sin escribir nada. Con otra frase, el libro no se toca.
- Un libro con `meta:kdf` abierto sin clave da `ParameterMismatch` de «cifrado en reposo».
- **La instantánea cifrada** lleva la marca `0x02`, la cabecera del libro detrás y el cuerpo sellado:
  se abre con la frase, sin el libro. La marca `0x01` —el cuerpo sellado con la clave de la versión 1—
  se sigue importando y ya no se escribe.

### El keystore (E3)

```json
{
  "version": "zkssl-keystore/2",
  "kdf": "argon2id",
  "kdf_sal": "0x<16 bytes>",
  "kdf_coste": { "m_kib": 65536, "t": 3, "p": 4 },
  "aead": "xchacha20poly1305, nonce 24B aleatorio antepuesto",
  "public_id": "0x…",
  "sealed": "0x<nonce 24 B ‖ cifrado>"
}
```

`keystore::save` escribe la versión 2, siempre con sal nueva, en un fichero al lado que se renombra
encima: un corte a mitad deja el keystore anterior, no uno truncado. En Unix, después del rename se
sincroniza el directorio, para que el nombre nuevo sobreviva a un corte de corriente. Un proceso que
muere entre crear el fichero de al lado y renombrarlo lo deja en disco,
`<keystore>.escribiendo-<pid>`, con 0600 y la clave de gasto cifrada como en el keystore nuevo: se
puede borrar, y desde el §774 lo quita la siguiente escritura que acaba bien, en Linux, si su pid ya
no vive. `keystore::load` lee las dos, y **un `zkssl-keystore/1` abierto con su frase se
reescribe en la 2** —misma frase, misma clave de gasto— antes de devolver el wallet; si no se puede
reescribir, falla y lo dice (en «Compatibilidad», lo que eso cuesta). Con otra frase no se toca.
El SDK ya no deriva nada por su cuenta: lo hace `zk_ssl::crypto::Kdf`, y `sha2` sale de su
manifiesto.

## Decisiones

Tomadas en la sesión del §702 dentro de lo que el autor aprobó —Argon2id o scrypt, con sal y coste,
con versión de formato y migración de lo que existe—, y reversibles mientras este RFC no esté
ACEPTADO.

- **D-A. Argon2id, no scrypt.** Es la primera recomendación de la hoja de OWASP y la que este número
  llevaba reservada desde el §206.
- **D-B. Un coste fijo por versión, el de la segunda opción del RFC 9106.** Medido: unos 0,43 s por
  derivación con la máquina cargada, y hay una por apertura. La primera opción del RFC (2 GiB) no se toma: el keystore se abre
  en el portátil de un titular.
- **D-C. El dominio como dato asociado, con dominios nuevos para la versión 2.**
- **D-D. Abrir es migrar**, en el libro y en el keystore, sin un paso aparte: un formato débil que
  sólo se migra si alguien se acuerda se queda débil. En el libro, después de verificarlo; en los dos,
  sin tocar nada con otra frase.
- **D-E. `argon2` 0.5.3**, por el kit (arriba).

## Compatibilidad

- **El cable no cambia.** Ni la versión, ni un método, ni un vector.
- **Hacia atrás, sí:** lo que escribió el código anterior se abre. El §702 lo midió con ficheros
  escritos por el código de `590caae`: un keystore v1, un libro cifrado y una instantánea cifrada.
- **Hacia delante, no.** Un keystore o un libro migrados no los abre el código anterior: el keystore
  por su `version`, y el libro porque la clave de la versión 1 no abre lo que sella la 2 («contraseña
  incorrecta o manipulación»). La migración no tiene vuelta.
- **Un keystore v1 sólo se abre donde se puede reescribir.** Abrir es migrar (D-D), y si no se puede
  escribir al lado del fichero —un medio de sólo lectura, un directorio sin permiso de escritura—,
  `keystore::load` falla con cualquier frase y dice por qué. Antes del §702 ese keystore se abría.
  Se copia a un directorio donde se pueda escribir y se abre allí. Una lectura sin migrar para esos
  medios sería otra decisión, del autor.

## Seguridad

- **Principio del API** (la clave de gasto no viaja): no lo toca. Endurece donde duerme.
- **Lo que cambia:** cada intento fuera de línea cuesta un Argon2id de 64 MiB en vez de un SHA-256, y
  cada fichero tiene su sal: ninguna tabla sirve para dos.
- **Lo que no cambia:** una frase débil sigue siendo débil; el coste multiplica el precio de cada
  intento, no reduce los intentos. Quien tenga el proceso en marcha tiene la clave derivada en
  memoria mientras el libro o el keystore están abiertos. Al soltarse se borran la frase, la clave
  derivada, la memoria de Argon2id y la clave que guarda el cifrado (`chacha20poly1305` 0.10.1 la
  borra en su `Drop`); lo que el sistema haya copiado antes, no. La sal sale del generador del
  sistema, como los nonces (`SECURITY.md` §3.11). El nodo no abre libros cifrados
  (`SovereignLayer::open`): el libro cifrado lo usa la capa como biblioteca, y el keystore, el SDK y
  el cli.

## Referencias

- `AUDITORIA.md` §199 (el keystore), §206 (la reserva del número), §286 (el registro de dominios),
  §694 (las fijaciones y la lista cerrada del kit), §702 (este RFC, sellado).
- `crates/zk-ssl/src/crypto.rs`, `crates/zk-ssl/src/persistence.rs`, `crates/zk-ssl/src/snapshot.rs`,
  `crates/zk-ssl-sdk/src/keystore.rs`.
- RFC 9106, *Argon2 Memory-Hard Function for Password Hashing and Proof-of-Work Applications*: §5.3,
  el vector de Argon2id que reproduce un test de la capa, con la etiqueta del fichero `kats/argon2id`
  de la implementación de referencia (P-H-C/phc-winner-argon2); y §4, los parámetros.
- argon2-cffi 25.1.0, `argon2/profiles.py`: `RFC_9106_LOW_MEMORY`.
- OWASP, *Password Storage Cheat Sheet*, §Argon2id: leída de `OWASP/CheatSheetSeries`,
  el Markdown de `cheatsheets/Password_Storage_Cheat_Sheet`, en el commit `29994dd`.
