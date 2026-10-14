# RFC-0001 — El KDF de reposo: Argon2id con sal y coste, y la versión del formato

- **Estado:** PROPUESTO (§702) — con sus tres etapas construidas en el mismo sello. El paso a
  ACEPTADO es del autor. La regla 4 del PROCESO pide la spec, el OpenRPC, los vectores y las suites:
  este RFC no toca el cable, así que ni `spec/RPC.md`, ni `spec/openrpc.json`, ni `spec/vectors/` se
  mueven; las suites de los crates que toca salieron verdes en el §702, y la decisión no está tomada.
  ⚠️ §812: y una cuarta etapa, la E4 —el recifrado—, que el autor decidió como etapa de este RFC; se
  redactó en privado, con el arreglo de un fallo de la E2 (`spec/rfc/PROCESO.md` §«Fallos de
  seguridad»), y entra construida en el §812, en el mismo sello que el arreglo. Sigue PROPUESTO.
- **Autor:** Ángel José Toranzo Portela
- **Asistencia GenAI:** Claude, en una sesión de Claude Code en la nube (§702): midió, propuso,
  aplicó y comprobó, y lo commitea la sesión que la lanzó, fuera del paso 4 de `GENAI.md` — ver [`GENAI.md`](../../GENAI.md)
  ⚠️ §812: y la E4, Claude, en una sesión de Claude Code en la nube, que la redactó en privado, la
  aplicó y la comprobó, también fuera del paso 4
- **Fecha:** 2026-10-07; la E4, 2026-10-14
- **Versión del protocolo afectada:** ninguna. `zkssl/0.4` no se mueve: nada de esto cruza el cable.
  Cambia el formato EN REPOSO: el keystore del SDK pasa de `zkssl-keystore/1` a `zkssl-keystore/2`,
  el libro cifrado gana la cabecera `meta:kdf` y la instantánea cifrada la marca `0x02`. ⚠️ §812: la
  E4 no cambia ningún formato; añade dos nombres reservados junto a todo libro que abre la capa
  —también el que el nodo abre en claro—, `<libro>.recifrando` y `<libro>.retirado`, y la regla
  con que la apertura los lee.
- **Asiento(s) de AUDITORIA:** §702, §812.

**Número.** El 0001 estaba reservado a esto desde el §206: la nota de numeración del RFC-0002 lo
dice, y los RFC-0003 a 0006 lo repiten. Este RFC lo ocupa.

## Estado de las etapas

| etapa | qué entrega | ¿rompe el cable? | estado |
|---|---|---|---|
| E1 — el KDF, con su versión | `zk_ssl::crypto::Kdf`: la versión 1 (SHA-256, sólo se lee) y la 2 (Argon2id con sal y coste); su cabecera de 29 bytes y su único lector, `Kdf::de_cabecera`; los dominios de cada uso; `argon2` clavada con `=0.5.3` en la capa y en ningún otro manifiesto | NO | sellada — §702 |
| E2 — el libro y su instantánea | `meta:kdf` en claro en cada lote; la clave se fija al abrir; un libro de la versión 1 se verifica y se migra a la 2 en un lote atómico (⚠️ §812: el lote dejaba en los ficheros lo sellado con la v1; desde el §812 la migración va por el camino de la E4); la instantánea cifrada, con la marca `0x02` y la cabecera del libro detrás; la `0x01` se sigue importando | NO | sellada — §702 |
| E3 — el keystore | `zkssl-keystore/2`, con `kdf_sal` y `kdf_coste` en claro; un `zkssl-keystore/1` abierto con su frase se reescribe en la 2, entero o nada | NO | sellada — §702 |
| E4 — el recifrado | `SovereignLayer::recifrar` cambia la frase y la sal del libro entero: lo copia, sellado con la clave nueva, a `<libro>.recifrando`, lo verifica en claro contra el original leyéndolo del disco, lo pone con un cambio de nombre en el sitio del libro y borra el viejo entero: entero o nada. La migración de la E2 va por el mismo camino, y toda apertura resuelve lo que deja una muerte a mitad: hacia atrás antes del compromiso, hacia delante después | NO | construida — §812, redactada en privado |

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
  ⚠️ §812: «todo en un lote: sled lo aplica entero o no lo aplica» era cierto de lo que se LEE, no de
  lo que queda en el DISCO. sled es un log, y el lote dejaba en los ficheros del directorio los
  valores sellados con la clave v1, sin sal: quien se lleve el disco de un libro así migrado prueba
  frases contra ellos a la velocidad de un hash, y la frase buena abre también la versión 2
  (`SECURITY.md` §3.14; abajo, en la E4, lo medido). Desde el §812 la migración es el recifrado de la
  E4 con la misma frase: un directorio nuevo con lo vivo, un cambio de nombre y el viejo borrado
  entero. Lo demás de este punto sigue en pie.
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

### El recifrado (E4)

⚠️ §812. Etapa decidida por el autor; redactada en privado, con el arreglo del fallo que cuenta el
primer párrafo, y construida en el mismo sello (`spec/rfc/PROCESO.md` §«Fallos de seguridad»).

**El fallo que arregla.** La migración de la E2 sobrescribía cada valor en un `sled::Batch` sobre el
mismo `sled` y hacía `flush`. Eso cambia lo que se lee entero o nada, pero sled es un log: los
valores sobrescritos siguen en los ficheros del directorio hasta que sled reaprovecha su segmento, y
eso puede no pasar nunca. Medido en el §812 sobre sled 0.34.7, el del `Cargo.lock`, buscando los bytes
de cada valor en los ficheros del libro: con un libro de dos cuentas, sus 25 valores sellados con la
clave v1 seguían, los 25, en los ficheros tras el lote, tras 200 lotes más y tras tres reaperturas
(sobre el código de antes del §802; desde él, el mismo libro lleva 26, con la raíz de las
identidades retiradas, y el lote en sitio deja los 26);
con uno de 702 cuentas, 1425 de 1425 tras el lote, 1356 tras 200 lotes más, y ninguno tras tres
reaperturas, cuando sled rehízo sus segmentos (si los sobrescribió o los devolvió como bloques
libres al sistema de ficheros, no se midió). Con el de dos cuentas siguieron los 25 tras tres
reaperturas: no se puede contar con que reabrir limpie. Cuánto dura el resto depende de lo que el libro escriba después: en uno
pequeño o quieto, no se va. La clave v1 es `SHA-256(dominio_v1 ‖ frase)`, sin sal: quien se lleve el
disco de un libro migrado prueba frases contra ese resto a unos 159 ns por intento (§702), la
etiqueta AEAD le confirma la buena, y esa frase abre también la versión 2. El test del §702 miraba
lo que sled devuelve, no los ficheros; el endurecimiento de este RFC no protegía esos discos, y
ningún documento lo decía. Un recifrado hecho igual, con un lote en sitio, dejaría en los ficheros
todo lo sellado con la frase vieja.

**Qué hace.** `SovereignLayer::recifrar(path, …, vieja, nueva)` —los parámetros de
`open_encrypted`, porque el libro se abre con la frase vieja y `load` lo verifica entero antes de
tocar nada— recifra el libro ENTERO con una frase nueva y una sal nueva, con el coste de la versión
2, y devuelve la capa abierta con la clave nueva. No cambia la raíz ni el contenido: las mismas
claves y, abiertos sus valores, los mismos bytes; un libro recifrado lo abre también el código del
§702, con la frase nueva. La clave `nueva` tiene que llevar su frase: una ya fijada no puede tomar
otra sal y da `Malformed` antes de tocar nada. La misma frase se admite: deja sal nueva y el
directorio limpio, pero no protege las copias. Un libro de la versión 1 se migra primero, con la
vieja, y después se recifra. La migración de la E2 es esta misma pieza con la misma frase.

**Los nombres**, reservados: P es el libro; R, `<P>.recifrando`; V, `<P>.retirado`; en el mismo
directorio, formados con el nombre del libro y nunca pegando texto a la ruta (con una ruta acabada en
`/`, pegar caería dentro del libro). Una ruta sin nombre —`.`, `..`— no tiene hermanos, y recifrar o
migrar rehúsan.

**Los pasos.** El libro sigue abierto, con su cerrojo, hasta el paso 7, para que nadie lo abra en
medio: tras el paso 5 sled escribe por su descriptor, en el inodo de V, y después de abrir sólo usa
la ruta para los blobs, que el libro no tiene.

| paso | qué | si falla |
|---|---|---|
| 0 | P es un directorio de verdad, no un enlace, y sólo lleva lo de sled (`conf`, `db`, `blobs/`, `snap.*`); un solo árbol de sled; la clave nueva lleva cabecera v2; V no existe | `Err`, sin escribir nada |
| 1 | crear R —falla si ya existe—, con 0700, que guarda hasta el final del paso 4: mientras lleva los ficheros que sled crea con la umask, sólo entra en él su dueño | `Err`; si R ya existía, no se toca |
| 2 | copiar a R cada valor de P, abierto con la vieja y sellado con la nueva; `meta:geometry_v7` y `meta:migrated` tal cual; la cabecera nueva; un lote, `flush`. Un valor que no abre con la vieja para y se nombra | se borra R; P intacto, con la vieja |
| 3 | verificar R leyéndolo del disco: las mismas claves que P más la cabecera, lo que va en claro igual, y cada valor sellado, abierto con la nueva, igual al de P abierto con la vieja. Las raíces (`root:*`) son valores del libro y van dentro; `pamt:` y `pmeta:`, que ninguna raíz cubre, también; y las identidades retiradas del §802, `retir:{k}` y su raíz `root:retir`, que van selladas | ídem |
| 4 | los modos de los ficheros de P pasan a los de R, y después el del directorio de P a R; sincronizar cada fichero de R, `R/blobs`, R y el padre | ídem |
| 5 | **`rename(P, V)` y sincronizar el padre: el compromiso** | si el `rename` mismo falla —EBUSY en un punto de montaje, o Windows con ficheros abiertos—, se borra R y P sigue intacto |
| 6 | `rename(R, P)` y sincronizar el padre | `Err` «pasado el compromiso», que dice con qué frase queda el libro —con la nueva al recifrar; con la misma, ya en la versión 2, al migrar—, y la apertura siguiente lo termina; hasta ella, el `.retirado`, si sigue en el disco, guarda lo sellado con la vieja |
| 7 | soltar el libro viejo, borrar V entero y sincronizar el padre | ídem |
| 8 | reabrir P y quedarse la clave nueva; nada se recarga, porque el paso 3 comprobó que la copia dice lo mismo | ídem |

**Lo que deja una muerte, y qué hace la apertura.** Toda apertura —`open`, `open_encrypted` y
`recifrar`, también la del libro que el nodo abre en claro— mira antes de abrir si están los
hermanos; sin ellos, no hace nada más.

| en disco | murió | la apertura | abre con |
|---|---|---|---|
| P | — | nada | la suya |
| P + R | en los pasos 1 a 4, o antes del 5 | borra R, que sólo lleva lo sellado con la nueva | la vieja |
| V + R, sin P | entre el 5 y el 6 | R pasa a P —está entera, verificada y sincronizada, porque 3 y 4 van antes que 5—, y V se borra | la nueva |
| P + V | tras el cambio de nombre del 6, o en el 7 | borra V, que es todo lo sellado con la vieja | la nueva |
| P + R + V, R solo, V solo, o V + R con un R sin `db` | el recifrado no los deja; los deja, por ejemplo, el código de antes del §812, que con V + R y sin P crea un libro vacío en P (medido) | rehúsa con `Malformed`, que nombra el estado; no toca nada y nunca crea un libro vacío | — |
| un hermano que no es un directorio de verdad, o que lleva algo que no es de sled | algo ajeno | rehúsa sin tocar nada | — |

**Los cerrojos.** Antes de actuar, la apertura toma, con una espera acotada —diez veces cada 50 ms—,
el cerrojo de sled sobre `db` de cada uno de P, R y V que exista: el mismo `flock` que toma sled
0.34.7 por fs2, y el de `std` es compatible con él. Si alguno sigue tomado, el libro está abierto o
recifrándose en otro proceso, y la respuesta es `Io`, sin tocar nada: el recifrador vivo tiene el de
P, que tras el paso 5 es el de V. Si no puede borrar V, la apertura falla y lo dice: no se abre con
lo viejo al lado sin que nadie lo vea.

**El contrato.** `Ok`: el libro está entero con la frase nueva, y ni él ni sus hermanos guardan nada
sellado con la vieja. `Err`: el libro está entero con UNA de las dos frases, la vieja si falló antes
del compromiso y la nueva si el mensaje dice que el recifrado lo pasó. Con un libro de la versión 1,
`recifrar` lo migra antes, y esa migración tiene su propio compromiso: si falla pasado ese, el
mensaje lo dice —«la MISMA frase con que se abrió»—, y el libro queda con la vieja, ya en la versión
2. Una muerte a mitad: la apertura siguiente lo resuelve, y el libro abre con una de las dos; quien
llama prueba las dos. Si murió antes del compromiso, la apertura vuelve atrás: el libro sigue entero
con la frase VIEJA, y lo sellado con ella sigue en el directorio hasta que el recifrado se repite y
acaba; quien cambia una frase comprometida no la tiene por cambiada porque el libro se reabra.
Después del compromiso, la apertura termina hacia delante, con la nueva. La durabilidad es la del
`fsync` del sistema de ficheros; fuera de Unix no se sincroniza el directorio.

**Lo que cuesta.** Escribir en el directorio padre, y sitio para una copia, que sale compactada: lo
vivo, sin lo sobrescrito (medido: un libro de 202 cuentas, tras 200 lotes, pasa de 1,05 MB a 0,53 MB
en el directorio, y uno de 902, de 2,10 MB a 1,06 MB). Dos Argon2id: unos 0,85 s con la cifra del §702; en la máquina
del porte, un recifrado entero de un libro de 65 claves tardó 438 ms. La migración de un libro v1
de 702 cuentas tardó 1,05 s, con su Argon2id. Con un libro de la versión 1, dos copias:
migra y recifra. Hasta el paso 7 nadie más puede abrir el libro, y si alguien lo tiene abierto,
`recifrar` da `Io` sin tocar nada; desde que suelta el viejo, en el 7, otro proceso puede abrirlo,
ya recifrado, y `recifrar` da el error de pasado el compromiso. No está probado con otro proceso
vivo.

**Lo que no cambia.** Ningún formato: los valores sellados, la cabecera de 29 bytes, `meta:kdf`, las
claves en claro, el coste de la versión 2, las marcas de la instantánea, el JSON del keystore,
`zkssl/0.4` y sus vectores. Lo único nuevo en disco son los dos nombres reservados y la regla con
que la apertura los lee.

**Lo que no hace, y se dice.**

- **El nodo** abre su libro en claro (`SovereignLayer::open`, en `crates/zk-ssl-node/src/main.rs`):
  no tiene frase que cambiar ni resto de la v1 que limpiar. Cablear su cifrado no entra en lo que
  pidió el autor, la E4 de la biblioteca (D-F). Tampoco se expone `recifrar` en el cli ni en el SDK. Pero su apertura es
  la de la capa: también resuelve los dos nombres reservados junto a su libro, con el mismo borrado y
  la misma negativa (en «Compatibilidad»).
- **Cifrar un libro en claro.** Con una frase y sin cabecera, un libro con datos se toma por uno de
  la versión 1, y `recifrar` da el error de hoy, «contraseña incorrecta o manipulación». Es la misma
  pieza, y queda para cuando se cablee el nodo.
- **Sobrescribir.** Borrar V es borrar, no sobrescribir, por decisión del autor (D-I). Lo que
  guarden los bloques que el sistema de ficheros libera, un SSD —su FTL, el nivelado de desgaste,
  TRIM—, un sistema copy-on-write o de log (btrfs, ZFS, APFS, F2FS) y sus instantáneas, las de LVM,
  de una máquina virtual o de la nube, o el journal con `data=journal`, no lo alcanza la biblioteca,
  y no se promete.
- **Las copias.** Una copia del directorio, una instantánea exportada antes o un enlace duro a un
  fichero del libro hecho fuera siguen abriendo con la frase vieja dondequiera que estén; las de un
  libro migrado por el §702 llevan además el resto v1.
- **La instantánea.** `export_snapshot` trunca en sitio la anterior, y una capa importada de una
  instantánea cifrada nace sin clave y, si se vuelve a exportar, sale en claro. Son de la E2, y
  quedan fuera de esta etapa: son las entradas 120 y 121 del `BACKLOG.md`, que abre el §812.

**El remedio de un libro ya migrado.** Un libro que migró el código del §702 —desde `2ac6db4`, el
07-10-2026, hasta el §812— puede llevar en sus ficheros el resto v1, y uno pequeño o quieto lo lleva
(arriba). La biblioteca no lo distingue: aquella
migración sólo escribió `meta:kdf`. El remedio es `SovereignLayer::recifrar` **con una frase nueva**:
el directorio queda limpio de lo sellado con la v1 y con la frase vieja (un test lo mide). Con la
misma frase el directorio también queda limpio, pero cualquier copia hecha desde la migración, y los
bloques libres, siguen dando la frase v1 a la velocidad de un hash, y esa frase abre el libro.

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

⚠️ §812: las de la E4. La D-F es del autor, y también no sobrescribir, de lo que la D-I dice que no se
hace, y el barrido de la D-H; lo demás —el marcador incluido—, de la sesión que la redactó en
privado, y reversible, como lo de arriba, mientras este RFC no esté ACEPTADO.

- **D-F. Cambiar la frase del libro, aparte y como etapa de este RFC** (del autor, §812). Es la K1 de
  `doc/ROTACION_DE_CUENTAS.md`, y resuelve su D-4: va aparte del diseño único de rotación de la
  entrada 52 del `BACKLOG.md`, como E4 de este RFC y no como RFC propio: lo resolvió al pedirla.
  Cablear el cifrado en reposo del nodo no entra en lo que pidió.
- **D-G. Un directorio hermano y un cambio de nombre, no un lote en sitio.** Un lote en sitio deja
  en los ficheros lo que sobrescribe (arriba); copiar lo vivo a un directorio nuevo y borrar el viejo
  entero no, dentro de lo que la biblioteca controla.
- **D-H. La copia se verifica en claro y leída del disco antes del compromiso**; antes de él, un
  fallo vuelve atrás, y después, va hacia delante. La apertura rehúsa lo que el recifrado no deja, no
  borra nada que no sea de sled, y no abre si no puede borrar el retirado. Y el barrido lo decidió
  el autor; que la importación falle, sin importar nada, si no puede borrar el `.tmp-descifrada`
  que tenga al lado —no se importa con el claro al lado sin que nadie lo vea—, y que sólo borre un
  fichero regular —un directorio o un enlace con ese nombre no los escribió aquel código, y no se
  tocan—, es de la sesión, reversible. Ni esa negativa ni el aviso por la salida de
  errores tienen test; lo tiene que lo borra.
- **D-I. El remedio de un libro ya migrado es a mano** —`recifrar` con una frase nueva—, y se
  admite la misma frase (de la sesión). Dos cosas no se hacen. La primera la decidió el autor (§812),
  por recomendación de la sesión: **no se sobrescribe antes de borrar** —se borra, y se dice que los
  bloques libres del sistema de ficheros, un SSD, un sistema copy-on-write o el journal pueden
  guardar copias: sobrescribir no se puede medir aquí, no vale en esos medios e invitaría a
  prometer—. La segunda es de la sesión, y el autor no la ha decidido: **no se pone un marcador** que
  distinga los libros ya migrados, que sería un formato nuevo y daría a entender que con eso basta.
  El marcador es reversible: se puede poner después, con
  su formato y su sello.

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
- ⚠️ §812, **la E4.** La migración de la E2 y el recifrado escriben al lado del libro: hace falta poder
  escribir en su directorio padre y sitio para una copia, así que un libro de la versión 1 cuyo
  directorio padre no se pueda escribir, o sin sitio para una copia, ya no se abre: hasta el §812
  migraba en sitio. En un medio de sólo lectura no se abría tampoco antes, porque sled 0.34.7 abre
  `db` para escribir (`src/config.rs`). La ruta del libro no puede ser un
  enlace —se rehúsa: renombrar y borrar el enlace dejaría el libro de verdad como estaba—, y el
  directorio no puede llevar nada que no sea de sled. `<libro>.recifrando` y `<libro>.retirado` son
  nombres reservados, y no sirven para copias: no hay marca que distinga un hermano que dejó el
  recifrado de un directorio que alguien llamó así, y **la apertura BORRA** un directorio de sled
  con uno de esos nombres junto al libro, si nadie lo tiene abierto —también una copia de seguridad—;
  lo que no es de sled no lo toca. Vale para todo libro que abre la capa, también el que el nodo
  abre en claro (`SovereignLayer::open`): al arrancar, el nodo resuelve un `.recifrando` o un
  `.retirado` de sled que tenga al lado como cualquier apertura —los borra, o, si falta el libro y
  está su `.retirado`, pone el `.recifrando` en su sitio—, y un estado que el recifrado no deja le impide arrancar. La
  capa declara `rust-version = "1.89"`: el cerrojo usa
  `File::try_lock`, estable desde Rust 1.89, como el guardián desde el §709; el sdk, el cli y el nodo
  ya la declaraban. **No se baja de versión con restos en disco**: el código de antes del §812 no
  conoce los hermanos, y con un `.retirado` y un `.recifrando` sin libro crea un libro vacío
  (medido). Un libro recifrado sin restos al lado lo abre el código del §702 con la frase nueva; el
  de antes del §802, como a cualquier libro desde el §802, sin hacer valer sus identidades retiradas
  (`SECURITY.md` §3.13). En
  Windows, que no renombra un directorio con ficheros abiertos, el recifrado y la migración fallan
  sin tocar nada; no está probado allí.

## Seguridad

- **Principio del API** (la clave de gasto no viaja): no lo toca. Endurece donde duerme.
- **Lo que cambia:** cada intento fuera de línea cuesta un Argon2id de 64 MiB en vez de un SHA-256, y
  cada fichero tiene su sal: ninguna tabla sirve para dos. ⚠️ §812: no en el directorio de un libro
  migrado por el código del §702 —desde `2ac6db4` hasta el §812—, que guardaba en sus ficheros lo
  sellado con la clave v1, sin sal, y ahí cada intento seguía costando un SHA-256 (la E4, arriba, y
  `SECURITY.md` §3.14). Desde el §812 la migración no lo deja, y recifrar con una frase nueva limpia el
  directorio de un libro ya migrado.
- **Lo que no cambia:** una frase débil sigue siendo débil; el coste multiplica el precio de cada
  intento, no reduce los intentos. Quien tenga el proceso en marcha tiene la clave derivada en
  memoria mientras el libro o el keystore están abiertos. Al soltarse se borran la frase, la clave
  derivada, la memoria de Argon2id y la clave que guarda el cifrado (`chacha20poly1305` 0.10.1 la
  borra en su `Drop`); lo que el sistema haya copiado antes, no. La sal sale del generador del
  sistema, como los nonces (`SECURITY.md` §3.11). El nodo no abre libros cifrados
  (`SovereignLayer::open`): el libro cifrado lo usa la capa como biblioteca, y el keystore, el SDK y
  el cli.
- ⚠️ §812, **lo que el disco guarda.** Dentro de lo que la biblioteca controla —el directorio del libro
  y sus dos hermanos— no queda nada sellado con la clave vieja tras una migración o un recifrado que
  acaban bien, ni tras la apertura que sigue a una muerte en la migración o a una muerte del
  recifrado pasado su compromiso (los tests lo miden con una muerte al acabar cada paso), o a un
  error pasado el compromiso: hasta esa apertura, el `.retirado` sigue en el disco con lo sellado
  con la vieja. Si el
  recifrado murió antes de su compromiso, la apertura vuelve atrás y el libro sigue entero con la
  frase vieja: lo sellado con ella sigue ahí hasta que el recifrado se repite y acaba. Fuera de eso,
  y no se promete: los bloques
  que el sistema de ficheros libera al borrar —también los del keystore v1 que la E3 reescribe con un
  `rename` (deducido, sin medir), los segmentos que sled trunca, y la instantánea anterior, que
  `export_snapshot` trunca en sitio—; un SSD, un sistema copy-on-write o de log, sus instantáneas y
  el journal; las copias del directorio y las instantáneas exportadas antes, que siguen abriendo con
  la frase vieja; un enlace duro hecho fuera; y la memoria —swap, hibernación, volcados—. Y un
  `{instantánea}.tmp-descifrada`, EN CLARO, si un proceso murió a mitad de una importación entre
  `5d2833e` (28-07-2026) y `c87d7d6` (§774, 08-10-2026): el §774 quitó el escritor y no barría;
  desde el §812, por decisión del autor, la importación de esa instantánea lo barre —si junto a ella
  hay un fichero con ese nombre, lo borra y lo dice por la salida de errores—; que si no puede la
  importación falle con un error que lo nombra, sin importar nada, es de la sesión. El de una
  instantánea que no se vuelva a importar se busca junto a ella y se borra a mano. Cambiar la frase del
  keystore —`load` y `save`— no cambia la clave de gasto: una copia vieja abierta con su frase vieja
  da la misma clave.

## Referencias

- `AUDITORIA.md` §199 (el keystore), §206 (la reserva del número), §286 (el registro de dominios),
  §694 (las fijaciones y la lista cerrada del kit), §702 (este RFC, sellado), §774 (los restos en
  disco del §702 que se quitaron entonces), §812 (la E4 y el arreglo del resto de la migración,
  redactados en privado).
- `crates/zk-ssl/src/crypto.rs`, `crates/zk-ssl/src/persistence.rs`, `crates/zk-ssl/src/snapshot.rs`,
  `crates/zk-ssl-sdk/src/keystore.rs`.
- sled 0.34.7, el del `Cargo.lock`: `src/config.rs` (el cerrojo `flock` sobre `db`, por fs2, y los
  nombres `conf`, `db`, `blobs/` y `snap.*`), `src/pagecache/snapshot.rs` (los `snap.*` se escriben
  al abrir) y `src/pagecache/logger.rs` (un fragmento pasa a blob por encima de un cuarto del
  segmento).
- RFC 9106, *Argon2 Memory-Hard Function for Password Hashing and Proof-of-Work Applications*: §5.3,
  el vector de Argon2id que reproduce un test de la capa, con la etiqueta del fichero `kats/argon2id`
  de la implementación de referencia (P-H-C/phc-winner-argon2); y §4, los parámetros.
- argon2-cffi 25.1.0, `argon2/profiles.py`: `RFC_9106_LOW_MEMORY`.
- OWASP, *Password Storage Cheat Sheet*, §Argon2id: leída de `OWASP/CheatSheetSeries`,
  el Markdown de `cheatsheets/Password_Storage_Cheat_Sheet`, en el commit `29994dd`.
