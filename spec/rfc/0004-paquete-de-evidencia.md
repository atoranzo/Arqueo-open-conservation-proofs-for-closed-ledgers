# RFC-0004 — El paquete de evidencia portable y su especificación

- **Estado:** PROPUESTO
- **Autores:** Che, con Claude (sesión 92)
- **Fecha:** 2026-09-04
- **Versión del protocolo afectada:** `zkssl/0.3` — **no sube** (ver Compatibilidad)
- **Asiento(s) de AUDITORIA:** §243, §289, §290, §293, §322, §395, el §396, que lo sella, y el §397 (E1)

## Estado de las etapas

| etapa | qué entrega | ¿rompe el cable? | estado |
|---|---|---|---|
| E1 — la mudanza | un documento normativo propio bajo `spec/` como único productor; la cabecera del binario remite; `spec/RPC.md` y `spec/README.md` apuntan | NO | sellada — §397 |
| E2 — los vectores y la puerta | `spec/vectors/paquete/` y su bloque de conformidad en `tools/canon.sh` | NO | pendiente |

Todas las medidas de este documento se tomaron sobre `bb5322f` (§395), en dos lecturas
puras que no escribieron un byte en el árbol: `PASTE-396-M` y `PASTE-396-PRE` (ver
Referencias).

## Motivación

El **paquete de evidencia portable** existe desde §289: las respuestas del cable que el
titular ya custodia, reunidas en un fichero y verificables **sin el nodo, sin la capa y
sin el probador** (§243). Es lo que sostiene su posición cuando el operador desaparece o
miente, y `spec/RPC.md` lo declara así en su sección de apagado.

Su especificación **existe y es buena. Vive en el sitio equivocado.**

Está escrita en la cabecera de `crates/zk-ssl-verify/src/main.rs`, líneas 1..90 (sha de
región `293990fedc785833`): el formato v1, el v2 con `cofirmas` (§322), la tercera forma
—el paquete de extensión (§293)—, el orden de comprobación en tres pasos y las dos
convenciones de versión que conviven en el mismo fichero. Y `spec/RPC.md` 810..855 (sha
de región `7963cdea2e82f540`), que es normativo sobre el paquete, **delega el formato ahí
por escrito**: «formato v1, declarado en la cabecera del binario de `zk-ssl-verify`».

De ahí salen tres problemas medidos.

**1. Un tercero que recibe el binario y quiere su especificación tiene que leer el
fuente del operador.** Es el problema del §243 otra vez, por el otro lado: entonces la
única forma de *verificar* una cabeza era compilar el código del operador; hoy la única
forma de saber *qué* verifica es leer su fuente.

**2. La cabecera ya caducó una vez, y lo confesó.** Su propia sección «La tercera forma,
que esta cabecera NO declaraba (§247)» dice que la extensión **estaba publicada en
`spec/RPC.md` y ausente aquí**, y saca la conclusión: «el productor rancio era el que más
cerca queda del código». Dos productores del mismo contrato, y el que envejeció fue el de
dentro. Un solo productor normativo lo impide por construcción.

**3. Nada ata el documento al binario.** El binario lee **14 claves distintas** del JSON
y tiene **39 llamadas de rechazo con texto**; ninguna la comprueba ningún gate.
`spec/vectors/` contiene sólo los tres vectores del **cable**, y el bloque de conformidad
de `tools/canon.sh` (`:278-286`) corre `conformance --check` únicamente sobre ellos. No
hay ni un vector de paquete en el árbol, ni un fixture JSON dentro del crate.

> **Corrección (§397).** El «39» de arriba es un censo POR LÍNEA, ciego a las once llamadas
> cuyo literal empieza en la línea siguiente. Por LLAMADA, aplanando las continuaciones, son
> **50** (49 textos distintos); y las 14 claves son las del sobre: en total el binario lee 31
> nombres. La frase se conserva; la cifra que vale es la de `spec/PAQUETE.md`, re-derivada al sellar.

## Diseño

### E1 — la mudanza

Nace **un documento normativo propio bajo `spec/`**, en español como el resto de
la carpeta, y es el **único
productor normativo** del formato del paquete y del contrato del mando. No se redacta de
cero: se **muda** lo que ya está escrito en la cabecera del binario, que es el patrón de
la mudanza del guardián (§296).

Cubre tres objetos, que son los tres que el binario acepta:

1. **el paquete v1** — `v`, `cabeza` (el payload de `zkssl_signedEpochHead` con
   `available:true`) y `acuse` opcional (`seq`, `hashPrueba`, `s`, `camino` con
   `siblings` e `isRight`);
2. **el paquete v2** (§322) — el mismo objeto con `cofirmas`, la respuesta de
   `zkssl_cosigs` tal cual;
3. **el paquete de extensión** (§293) — `tipo: "extension"`, `vieja`, `nueva`, `camino`.

Y añade lo que hoy no está en ningún sitio como contrato: el **orden de comprobación**,
las **reglas de rechazo** y el **contrato del mando** (un argumento, y tres estados de
salida: éxito, fallo, y uso).

**El documento especifica el SOBRE, no los campos.** Los valores del paquete son
respuestas del cable **tal cual** —quien reescribe, adultera—, así que la semántica de
cada campo **remite a `spec/RPC.md` por línea**. Duplicarla crearía dos productores del
mismo contrato, que es justamente el defecto que este RFC repara.

La cabecera de `main.rs` **remite al documento y conserva su historia**: no se borra, se
cita (§247). La forma ya está practicada en este mismo crate, en
`crates/zk-ssl-verify/Cargo.toml:25-27`, que dice de su propia enumeración «la lista no
es su superficie: la superficie se lee de los `pub` de `src/lib.rs`, no de aquí».

`spec/RPC.md` 810..855 conserva su prosa entera y **cambia su puntero**: donde delega en
la cabecera del binario, pasa a delegar en ese documento.

`spec/README.md` gana su fila en la tabla «What is in this folder» y su mención en
«Where the rest lives».

**Puerta de E1**: el documento en el árbol, con el nombre que E1 fije; la cabecera del binario remitiendo y sin
enumerar; el puntero de `RPC.md` corregido; la fila del README puesta; las nueve
herramientas de `tools/` en verde y `canon.sh --sello` VERDE. Y una puerta de contenido:
el documento cubre **todas** las claves y **todas** las reglas de rechazo que el binario
implementa — el censo se **re-deriva al sellar**, no se teclea.

### E2 — los vectores y la puerta

Nace **`spec/vectors/paquete/`**, con un vector positivo por cada forma (v1, v2,
extensión) y un vector negativo por cada regla de rechazo que el documento publica.

La puerta va en el **bloque de conformidad de `tools/canon.sh`** (`:278-286`), donde ya
vive la conformidad del cable con su régimen de VERDE y RECHAZADO. Correr el binario
contra ficheros JSON es inmediato y no necesita nodo.

**La puerta no va dentro del crate.** Un `include_str!` hacia `spec/` ataría el crate al
repositorio, que es exactamente la propiedad que el §243 estableció y que el §395 acaba
de convertir en invariante comprobable.

**Qué reglas están ya implementadas y cuáles no, E2 lo MIDE antes de escribir un
vector.** Este RFC no lo afirma. Lo que sí está publicado hoy en dos sitios —`RPC.md`
835..837 y la cabecera del binario— es la regla «**un v1 que traiga `cofirmas` se
RECHAZA**, porque subir la versión es exactamente lo que las hace parte del contrato»; es
la primera candidata a vector negativo, y su estado real se mide, no se supone.

**Puerta de E2**: cada forma con su vector positivo y cada regla publicada con su
negativo; el bloque de conformidad corriéndolos; y **el gate enseñado vivo**: un vector
saboteado en un solo nibble pone el canon en rojo. Es el molde que
`tools/banco_apagado.sh` ya practica —verde con el paquete bueno, rojo con un nibble
adulterado— trasladado a un gate que no necesita levantar un nodo.

## Lo que se DESCARTÓ al medir

1. **Redactar la especificación de cero.** Descartado: ya existe y es correcta. El
   defecto no es su contenido, es su domicilio. Esto es una mudanza, no una redacción.
2. **Meter la especificación dentro de `spec/RPC.md`.** Descartado: `RPC.md` especifica
   lo que cruza el cable, y el paquete **no cruza el cable** — es un artefacto que se
   entrega a un tercero sin nodo. El propio `spec/README.md` se define como la superficie
   normativa de «everything that crosses the wire». Nombres que significan una sola cosa.
3. **Un gate dentro del crate, con `include_str!` hacia `spec/`.** Descartado: ataría el
   crate al repositorio, contra §243 y contra el gate del §395.
4. **Una herramienta nueva en `tools/`.** Descartado: el bloque de conformidad de
   `canon.sh` ya hace exactamente esto, y una herramienta más movería el censo de las
   nueve (§384) sin ganar nada.
5. **Meter los vectores del paquete entre los `zkssl-0.N.json`.** Descartado: esos son un
   fichero por versión **del cable**, jamás reescritos; mezclar ahí vectores de otro
   objeto rompería esa semántica. Van en `spec/vectors/paquete/`.
6. **Cablear `tools/banco_apagado.sh` al canon como puerta.** Descartado *para este RFC*:
   el banco levanta un nodo real y tarda minutos, y meterlo en `--sello` cambia el coste
   de sellar. Ese coste hay que medirlo antes, y es un frente propio. Los vectores dan la
   puerta sin pagarlo.
7. **Fijar aquí el nombre del fichero que E1 creará.** Descartado POR MEDIDA, no
   por gusto: el árbol corre `tools/verificar_citas.py` dentro del canon, y esa
   compuerta exige que todo documento Markdown citado por nombre EXISTA. Al montar
   este RFC con el nombre escrito, la compuerta pasó de cero fantasmas a un fantasma
   con seis citas, y el sello se paró en rojo antes del commit. Y lo hizo con razón:
   un lector que siguiera esa referencia no encontraría nada. Un RFC PROPONE; no
   puede citar lo que todavía no existe. El nombre se fija en E1, en el mismo commit
   que crea el fichero.
8. **Subir la versión del cable.** Descartado por el criterio publicado en
   `spec/RPC.md:864`: la versión sube cuando cambian **los valores que viajan**, no
   cuando crece la superficie. Aquí no viaja ningún valor nuevo.

## Compatibilidad

| etapa | ¿rompe el cable? | vectores |
|---|---|---|
| E1 — la mudanza | **NO.** Ningún valor que viaja cambia; `zkssl/0.3` sigue vigente | los tres del cable, **intactos**; `conformance --check` debe seguir dando `0.3` IDÉNTICO y rechazando `0.2` y `0.1` como de OTRA versión |
| E2 — los vectores y la puerta | **NO.** Los vectores del paquete son un universo aparte, bajo `spec/vectors/paquete/` | ídem: los del cable no se tocan |

**Regla 2 del PROCESO: los vectores viejos jamás se reescriben.** Los tres vectores del
cable —`zkssl-0.1.json`, `zkssl-0.2.json` y `zkssl-0.3.json`— **se conservan** bajo su
versión, sin una sola modificación.

### Por qué entra por RFC si no rompe el cable

Las dos lecturas son ciertas y este RFC escribe las dos.

**Por la letra**, el ámbito del proceso (`spec/rfc/PROCESO.md:3-6`) enumera **ficheros**:
`spec/RPC.md`, `spec/openrpc.json` y los vectores de `spec/vectors/`. Este cambio toca
`spec/RPC.md` —el puntero de la sección de apagado— y añade bajo `spec/vectors/`. Cae
dentro, y entra por RFC.

**Por el espíritu**, el paquete no cruza el cable, así que no hay valor que viaje que
cambie. Eso exime de **subir la versión**, que es otra regla y otro criterio
(`RPC.md:864`). No exime del proceso.

**`spec/openrpc.json` no se toca ni se regenera**, porque este RFC no añade ningún
método. La regla 4 exige, para ACEPTADO, «la spec actualizada + OpenRPC regenerado +
vectores re-emitidos (o nuevos bajo la versión nueva) + suites verdes»: el giro a
ACEPTADO se justificará con la spec, los vectores nuevos y las suites verdes, **y con
esta declaración de que el OpenRPC queda fuera del alcance por no haber método nuevo**.

## Seguridad

**Efecto sobre el principio del API —la clave de gasto no viaja jamás: NINGUNO.** Este
RFC no añade ni cambia ningún método, ningún tipo del cable y ningún material. El paquete
son respuestas del cable ya publicadas, reunidas en un fichero.

**Efecto sobre las deudas declaradas:**

- **Aviso fuera de banda (§21): ninguno.** El paquete no transporta avisos: sus catorce
  claves son `v`, `tipo`, `cabeza`, `available`, `publicKey`, `signature`,
  `epochDigest`, `acuse`, `camino`, `siblings`, `isRight`, `cofirmas`, `vieja` y
  `nueva`. Ninguna es un aviso ni su sobre.
- **Nodo único: ninguno.** Este RFC **no introduce consenso** en ninguna etapa,
  deliberadamente (`SECURITY.md` §6). Al contrario: documenta el procedimiento por el que
  una posición sigue siendo demostrable **sin** el nodo.
- **`--dev`: ninguno.** El manifiesto del verificador no declara ninguna sección
  `[features]` y el crate no tiene custodios de prueba.

**Publicar el esquema no publica nada que no estuviera ya publicado.** Cada campo del
paquete es una respuesta del cable que `spec/RPC.md` ya especifica; lo que este documento
añade es el sobre que los reúne y las reglas con que se rechaza.

**El paquete REPORTA, no juzga.** Dice cuántas cofirmas verifican contra esa cabeza y ese
operador; **qué testigos valen y cuántos hacen falta lo decide el CLIENTE** con su
política (§319), porque quien arma el paquete puede ser el operador. Este RFC **no mueve
esa frontera**, y el documento nuevo la repite en su sitio.

## Referencias

Medido sobre `bb5322f` (§395), con las huellas en `sha256 | cut -c1-16` y las líneas en
`wc -l`.

- `spec/rfc/PROCESO.md` `0eba90e3fe5d93f8` / 24 — el ámbito (`:3-6`), los estados
  (`:10`) y las cinco reglas.
- `spec/RPC.md` `3eaa7a433d53f579` / 867 — la sección «Apagado — el fin de vida,
  declarado (nota 91)», 810..855, sha de región `7963cdea2e82f540`; el criterio de
  versión en `:864`.
- `crates/zk-ssl-verify/src/main.rs` `2a99ef5e366d434c` / 582 — la cabecera 1..90, sha de
  región `293990fedc785833`.
- `crates/zk-ssl-verify/Cargo.toml` `a4f92b9047a12c97` / 51 — el precedente del puntero
  que no enumera (`:25-27`), y la ausencia de `[features]`.
- `spec/README.md` `6dacd2d90986184d` / 142 — regiones a editar 14..28
  (`8a36873fd7f99272`) y 135..142 (`f148d32d90d1de39`).
- `spec/vectors/zkssl-0.1.json` `3aa7b0623cfe1abf` / 64 · `zkssl-0.2.json`
  `d9c1b153a8d2311c` / 70 · `zkssl-0.3.json` `287f6a5a538ca7f7` / 70.
- `tools/canon.sh` `3ecad706a43c940e` / 302 — el bloque de conformidad `:278-286`.
- `spec/rfc/0002-lotes-y-transicion-de-hoja.md` `91bbdafa42d5ee51` / 389 y
  `spec/rfc/0003-compromiso-v2.md` `034c13a89173b825` / 276 — los moldes de este
  documento.
- `PASTE-396-M` `db7e5bec4a04dce0` / 657, salida `1423200c6f57acce` / 687 — el terreno del
  hito: la superficie, el esquema sin tipar, el estado de `spec/`.
- `PASTE-396-PRE` `8f1509d1a95cb6bc` / 440, salida `fc096dc5f441d600` / 535 — la sección
  de apagado, el esquema campo a campo, las 39 llamadas de rechazo y las regiones del
  README.

**Numeración.** El **RFC-0001 queda reservado** al endurecimiento del KDF del keystore
(SHA-256 → Argon2id), sin redactar, según `spec/rfc/0002-lotes-y-transicion-de-hoja.md`
`:30-34`. Este RFC toma el **0004** por correlativo, tras el 0002 y el 0003.

**Procedencia de dos citas arrastradas.** Todo lo anterior se midió en la sesión 92 salvo
dos referencias, que se declaran: `spec/RPC.md:864` —el criterio de cuándo sube la
versión— se midió en la sesión 66 y **sigue vigente porque el fichero conserva su huella**
(`3eaa7a433d53f579` / 867, re-medida hoy); y la remisión a `SECURITY.md` §6 por el nodo
único se **copia de la sección Seguridad del RFC-0003**, que es el precedente ACEPTADO.
