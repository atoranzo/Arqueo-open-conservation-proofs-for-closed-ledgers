# RFC-0020 — El esquema dentro de lo que se firma: la cabeza y la cofirma dicen con qué se firmaron, y una cabeza del esquema A sigue verificando cuando el nodo firma con el B

- **Estado:** PROPUESTO (§790) — el texto, sus etapas y sus siete decisiones, escritos sobre lo
  medido en el árbol en `8c02941`. El autor lo aceptó en el §799, con sus siete decisiones TOMADAS,
  en cada una la que el texto recomienda: ver «Decisiones (del autor, §799)». Ninguna etapa
  construida: ACEPTADO llega con ellas, porque el paso a ACEPTADO exige construir sus etapas (regla
  4 del PROCESO). ⚠️ §799: hasta aquí decía «Ninguna decisión está tomada y ninguna etapa
  construida: adoptarlo, en todo o en parte, es del autor».
- **Autor:** Ángel José Toranzo Portela
- **Asistencia GenAI:** Claude, en una sesión de Claude Code en la nube (§790): midió el árbol y
  redactó el borrador, fuera del paso 4 de `GENAI.md` — ver [`GENAI.md`](../../GENAI.md)
- **Fecha:** 2026-10-14
- **Versión del protocolo afectada:** `zkssl/0.4` -> `zkssl/0.5`, con la E4: la cabeza v7 cambia
  lo que el nodo firma y sirve, y un verificador de hoy la rechaza. Viaja en el corte del RFC-0018
  (decisión 7, tomada en el §799). Las cabezas v2 a v6, las actas y los vectores publicados no
  cambian un byte. ⚠️ §799: hasta aquí decía «si se adopta» donde dice «con la E4», y «Propone
  viajar en el corte».
- **Asiento(s) de AUDITORIA:** §236 (un solo marcador de versión), §240 (el apaño del OID), §275 (el
  byte de versión elige recompositor), §297 (la cofirma), §406 (`VersionCabeza`), §643 y §644 (el
  acta, con su esquema, y su opt-in), §699 (lo que una versión no identifica), §790 (este borrador),
  §799 (la aceptación del autor, con sus siete decisiones).
- **Backlog:** la **87** (agilidad criptográfica: el esquema de firma no está versionado), que este
  RFC no cierra hasta que se construya; con la **84** (cambiar de esquema es una rotación) y la
  **108** (qué esquema sería el B: no se mide aquí).

## Estado de las etapas

| etapa | qué entrega | ¿rompe el cable? | estado |
|---|---|---|---|
| E1 — la regla, escrita | este texto: el identificador, dónde entra, la regla de la transición y su coste | no | **escrita (§790) y decidida (§799)** ⚠️ §799: hasta aquí decía «propuesta (§790)» |
| E2 — el núcleo, apagado | el registro de esquemas en `spec/NUCLEO.md` §6, el preámbulo v7 de la cabeza y el de la cofirma con sus KAT, y la segunda implementación que los reproduce; nadie los firma todavía | no (aditivo: el registro, dos funciones y tres KAT) | sin construir |
| E3 — el despacho | el verificador elige el verificador por el esquema, con un conjunto aceptado y otro emitido; un esquema que no se acepta se rechaza con su número | no (con un solo esquema registrado, nada cambia de veredicto) | sin construir |
| E4 — el corte | `VersionCabeza::V7` en el conjunto; el nodo firma y sirve v7 con `esquema`; el testigo cofirma v7; el kit lee `esquema` en cada cabeza de sus sobres; los vectores que cambian de texto, mudados a `0.4/` | **sí** | sin construir; en el sello del corte `zkssl/0.5`, como decidió el autor en el §799 (decisión 7) ⚠️ §799: hasta aquí decía «si el autor lo decide así» |
| E5 — la transición, medida | un segundo esquema registrado, una rotación A -> B en un nodo real y una cabeza de A juzgada por contención bajo B, en un banco | según el esquema B | sin construir; espera a la 108 y a la elección del autor |

## Motivación

La 87 pide «un identificador de esquema **dentro de lo que se firma**, y la regla escrita de cómo
una cabeza firmada con el esquema A sigue verificando cuando el nodo ya emite con el B»
(`BACKLOG.md`, entrada 87, «Qué lo cierra»). El RFC-0015 lo dejó dicho en su tabla: «ninguna firma
dice con qué esquema se hizo: el OID va dentro de la clave publicada, que nadie firma»
(`spec/rfc/0015-el-ciclo-de-vida-del-firmante.md:61`). Lo que se firma hoy, medido en este árbol:

| lo firmado | sus bytes | ¿lleva el esquema? | dónde |
|---|---|---|---|
| **la cabeza** | `b"ZK-SSL-epoch-head" ‖ version ‖ epoch_digest`, 50 B | **no** | `spec/NUCLEO.md:329`; `crates/zk-ssl-verify/src/lib.rs:426-432`; la firma el nodo en `crates/zk-ssl-node/src/firma_cabeza.rs:221` |
| **la cofirma** | `b"ZK-SSL-witness-cosign" ‖ version ‖ epoch_digest ‖ len(u16) ‖ clave_op`, 56 + N B | **el del testigo, no**; el del operador, sólo como los cuatro bytes del OID que su clave lleva dentro | `spec/NUCLEO.md:330`; `lib.rs:463-480`; la firma el testigo en `crates/zk-ssl-cli/src/witness.rs:3440` |
| **el acta** | `b"ZK-SSL-key-act" ‖ version ‖ acta_digest`, 47 B | **sí**, el de la clave que entra: `acta_digest` compone `as_digest(esquema)` en su cuerpo | `spec/NUCLEO.md:331`; `crates/zk-ssl-hash/src/lib.rs:861-884` (el esquema, `:871`); `crates/zk-ssl-verify/src/actas.rs:35-49` |
| la huella de una clave | `Blake3(b"ZK-SSL-anchor-key-v1" ‖ len ‖ clave)` | no: cubre los bytes de la clave | `spec/NUCLEO.md:355`; `crates/zk-ssl-hash/src/lib.rs:1845-1847` |
| la nota del medio | la línea C2SP, `key_id = SHA-256(nombre ‖ "\n" ‖ 0x06 ‖ clave)[:4]` | sí, el tipo de `signed-note`, fuera del núcleo | `spec/rfc/0013-el-medio-del-ancla.md:129-132` |

Tres hechos más, del código:

1. **El verificador fija el esquema en el código, no en lo que lee.** `Conjunto =
   XmssMtSha2_40_8_256` (`crates/zk-ssl-verify/src/lib.rs:145`), y `clave_desde_bytes` sólo sabe
   leer una clave de ese conjunto (`lib.rs:621-639`). La segunda implementación lo elige por el OID
   de la clave publicada, del que deriva el largo de la firma para separarla del preámbulo que va
   adjunto (`tools/segunda/juez_cabezas.py:71-75` y `:91-92`;
   `tools/segunda/kat_xmss/xmss.py:151-163`). Con un esquema, las dos cosas dan lo mismo; con dos,
   quien verifica tiene que saber cuál antes de verificar, y eso es lo que la 87 pide atar a la
   firma.
2. **El OID de la clave no identifica el esquema ni dentro de RFC 8391.** Sus dos registros, XMSS y
   XMSS^MT, empiezan los dos en 1, y el 5 es válido en los dos (`lib.rs:535-537`, §240). El acta lo
   resuelve con la familia en los 32 bits altos: `ESQUEMA_XMSSMT_SHA2_40_8_256 = (1 << 32) |
   0x0000_0005` (`actas.rs:35-38`), y `verificar_acta` rechaza otro con su número antes de mirar nada
   más (`actas.rs:196-199`). El comentario dice «la familia en los 32 bits altos (1, el registro de
   RFC 8391)» y no dice cuál de los dos registros es la familia 1.
3. **El único sitio firmado que nombra el esquema es opcional y no cubre al testigo.** El acta es
   opt-in: sin `--siguiente` y sin actas en el diario el nodo no firma ninguna
   (`spec/rfc/0015-el-ciclo-de-vida-del-firmante.md:154-155`, §644), y un libro sin acta génesis no
   tiene nada firmado que diga su esquema. Las claves de los testigos no tienen actas: el RFC-0015
   «es para la clave del operador» (`spec/rfc/0015-el-ciclo-de-vida-del-firmante.md:319-320`).

**Lo que este RFC reabre, y por qué.** El RFC-0015 descartó «la clave o el esquema dentro de la
cabeza: una v7 de la cabeza y todos sus vectores, para decir en cada latido lo que el acta dice una
vez por clave» (`spec/rfc/0015-el-ciclo-de-vida-del-firmante.md:381-382`). El argumento vale para un
nodo con actas. No pesó los hechos 1 y 3: que el acta es opcional, que la cofirma no tiene ninguna,
y que el verificador tiene que elegir el verificador antes de leer el acta de la clave. La decisión
1, abajo, dejaba al autor las dos ramas: la v7, o quedarse en el acta y exigirla para todo esquema
que no sea el de hoy; el autor eligió la v7 en el §799. ⚠️ §799: hasta aquí decía «deja al autor
las dos ramas», sin la elección.

**Cuánto pesa lo firmado, medido.** De los 492 ficheros `.json` de `spec/vectors/`, 405 llevan al
menos una firma XMSS^MT con su preámbulo adjunto: 678 apariciones, de las que 536 son de cabeza
(preámbulo de 50 B), 127 de acta (47 B) y 15 de cofirma (124 B, con la clave del operador de 68 B).
Distintas son 60 —49 de cabeza, 9 de acta y 2 de cofirma—: muchos vectores derivan de las mismas
capturas. Cada firma mide 18.469 B más su preámbulo (`FIRMA_RFC_BYTES`, `lib.rs:287`; el test
de `lib.rs:821`). Medido con un recorrido de Python que busca en cada vector las cadenas hex de
18.469 B más un preámbulo y lee el dominio que lo abre.

## Diseño

### D-A — El identificador: el `esquema` del acta, un `u64`, con un registro

Un solo identificador para las tres firmas: el `esquema` que el acta ya lleva, un `u64` con la
familia en los 32 bits altos y el parámetro de la familia en los bajos (`actas.rs:35-38`). Una
escritura y no dos, como pide el RFC-0016: el acta, la cabeza y la cofirma nombran un esquema con el
mismo número.

- **El registro** va a `spec/NUCLEO.md` §6, junto a los dominios, con un solo productor en el código
  —el molde de `VersionCabeza`, `lib.rs:181-210`—, y su KAT. Nace con una entrada: `0x1_0000_0005`,
  XMSS^MT-SHA2_40/8_256, y el registro dice qué registro de RFC 8391 es la familia 1 (hecho 2).
- **Cada entrada declara** la escritura de su clave pública, la de su firma —para la familia 1, la
  de hoy: la firma de RFC 8391 con el preámbulo adjunto detrás (`lib.rs:821`)—, si tiene estado (y
  con él, guardián) y el texto con que se nombra en los rechazos.
- **El cero no se asigna**, como en la marca v2 de la D-G del RFC-0018: un firmante que no declare
  su esquema no escribe un número válido.
- **Ninguna otra entrada se asigna aquí.** Qué esquema sería el B lo mide la 108; el número lo da el
  registro cuando entre.

### D-B — En la cabeza: el preámbulo v7

```text
v2 a v6, hoy:  b"ZK-SSL-epoch-head" ‖ version ‖ epoch_digest                (17 + 1 + 32 = 50)
v7:            b"ZK-SSL-epoch-head" ‖ 7 ‖ esquema (u64 BE) ‖ epoch_digest    (17 + 1 + 8 + 32 = 58)
```

- **El molde es el del §275**: el byte de versión del preámbulo es el único desambiguador (§236), y
  la versión que la firma declara elige recompositor, «en la biblioteca y en el mando»
  (`spec/RPC.md:602-606`). Aquí elige además la forma del preámbulo: con 7, lleva el esquema. El
  dominio sigue sin versión dentro (`lib.rs:146-148`).
- **La v7 compone como la v6.** `epoch_digest` no cambia: el esquema es de la firma, no del libro,
  y no entra en nada que componga sobre el digest —el MMR de cabezas, el ancla, los acuses, los
  recibos—. La doc de `VERSION_FORMATO`, que dice que sube «cuando cambian los campos de
  `EpochHead`» (`lib.rs:165-166`), gana «o la forma del preámbulo».
- **En el cable**, `zkssl_signedEpochHead` sirve `esquema` (QUANTITY) junto a `formatVersion`, y lo
  mismo cada cabeza de un sobre del kit (`cabeza`, `vieja`, `nueva`, `cierre` y las demás). Quien
  verifica lo lee antes que la clave, elige con él el verificador (D-E) y compara el preámbulo
  recuperado con el que compone con ese esquema: mentir sobre él es `PreambuloDistinto`, como mentir
  sobre `formatVersion` (`lib.rs:669-676`).
- **En una cabeza v1 a v6 el campo no se lee**: lo que no se firma no existe para el núcleo
  (`spec/NUCLEO.md` §«3. La regla de extensión, ejercida», segunda mitad). Su esquema es el
  implícito, `0x1_0000_0005`, el único que firmó nunca una cabeza. La v1 no la lee el kit, sino
  la biblioteca (`spec/PAQUETE.md:742-743`), y la segunda la recompone
  (`tools/segunda/juez_cabezas.py:66`).

### D-C — En la cofirma: los dos esquemas

```text
hasta v6:  b"ZK-SSL-witness-cosign" ‖ version ‖ epoch_digest ‖ len(u16 BE) ‖ clave_op
v7:        b"ZK-SSL-witness-cosign" ‖ 7 ‖ esquema_testigo (u64 BE) ‖ esquema_op (u64 BE)
             ‖ epoch_digest ‖ len(u16 BE) ‖ clave_op                                 (72 + N)
```

El byte de versión de la cofirma es el de la cabeza que cofirma (`lib.rs:722`; §780), así que una
cofirma v7 es la de una cabeza v7, y el campo con prefijo sigue el último, «por diseño»
(`lib.rs:461-462`). El esquema del testigo es el de su firma, que hoy nada nombra (hecho 3). El del
operador hace que la cofirma diga «este operador, con esta clave **de este esquema**, publicó este
digest»: la clave del operador va dentro para que la cofirma no sea transferible (`lib.rs:440-444`),
y la misma razón vale para su esquema. El autor lo quiso en el §799 (decisión 3); sin él serían
ocho bytes menos. ⚠️ §799: hasta aquí decía «Si el autor no lo quiere, son ocho bytes menos».
`Cosig` (`spec/RPC.md:992-994`) gana `esquemaTestigo` y, con la decisión 3, `esquemaOperador`.

### D-D — En el acta: nada

El acta ya firma el esquema de la clave que entra, y la cadena de actas el de cada clave del
operador. La firma de la clave que se va, cuando la lleva, se verifica con el esquema del acta
previa. Lo que el acta no ata es el esquema de la **sucesora**: `siguiente` es la huella de su
clave, y la huella cubre bytes, no un esquema (`crates/zk-ssl-hash/src/lib.rs:1845-1847`). El
RFC-0015 lo dio por cubierto dentro de XMSS —«como la huella cubre la clave entera, OID incluido,
el conjunto XMSS queda comprometido por ella; el campo `esquema` existe para cuando la clave que
entra sea de otra familia, que es la 87»
(`spec/rfc/0015-el-ciclo-de-vida-del-firmante.md:128-130`)—, y por el hecho 2 el OID, solo, no
identifica el esquema ni dentro de RFC 8391. Si dos esquemas aceptados admitieran la misma
escritura de clave, el compromiso de la pre-rotación no diría cuál. Es la decisión 4.

### D-E — La regla de la transición: cómo una cabeza de A sigue verificando cuando el nodo firma con B

1. **El esquema declarado elige el verificador, y la firma lo ata.** En una cabeza o una cofirma
   v7, el del preámbulo; en una v1 a v6, el implícito, `0x1_0000_0005`. Una firma de otro esquema
   sobre un preámbulo v1 a v6 no existe: se rechaza con nombre.
2. **Aceptar y emitir son dos conjuntos.** El nodo emite con un esquema; el verificador acepta un
   conjunto que lo contiene. Es el molde de `COFIRMA_VERSION` y `COFIRMA_V_MAX`: «quien lee puede ir
   por delante de quien escribe y **nunca al reves**» (`lib.rs:308-314`). El conjunto aceptado tiene
   un productor y crece como el de versiones de cabeza (`spec/NUCLEO.md` §«3. La regla de
   extensión, ejercida», primera mitad); a diferencia de aquél, puede perder un esquema, y sólo por
   la regla 5.
3. **Cambiar de esquema es rotar** (la 84). La clave B entra con su acta, con `esquema` B, firmada
   por B y, si el estado de A es fiable, por A; y B es la `siguiente` que A comprometió (RFC-0015,
   D-B). Sin acta no hay cambio de esquema: una cabeza de un esquema distinto del de la clave
   fijada es un `cambio-de-clave`, que detiene, porque una clave de otro esquema es otra clave.
4. **Una cabeza de A sigue verificando con A** después de la rotación, dentro de su tramo (regla 4
   de la D-C del RFC-0015, `spec/rfc/0015-el-ciclo-de-vida-del-firmante.md:141-143`), mientras A
   esté en el conjunto aceptado. Que el nodo deje de poder firmar con A —la dependencia muere: hoy
   `xmss = "=0.1.0-pre.0"`, sin auditoría independiente (`crates/zk-ssl-verify/Cargo.toml:53-57`)—
   no lo saca del conjunto: verificar no necesita al firmante, y el verificador de A tiene dos
   implementaciones, el crate fijado y `tools/segunda/kat_xmss/xmss.py`.
5. **A sale del conjunto aceptado sólo si se rompe**, y eso es una ruptura con su versión (el
   RFC-0021). Desde entonces una cabeza de A vale sólo **contenida** en algo firmado con un esquema
   aceptado: con una prueba de extensión (§293, el MMR del §291) hasta una cabeza de B, o hasta el
   acta de rotación a B, cuya `procedencia` firma la última cabeza de A y el MMR que hereda la
   clave que entra (`spec/rfc/0015-el-ciclo-de-vida-del-firmante.md:121` y `:154-158`;
   `crates/zk-ssl-hash/src/lib.rs:875-881`). Una cabeza de A que no esté contenida —firmada después
   de la rotación, o fabricada después de la rotura— no vale. Que esto alcance a todas las cabezas
   de A es DERIVADO de leer el código; lo mide la E5.
6. **Un esquema que el verificador no acepta se rechaza con su número**, antes de mirar nada más,
   como ya hace `verificar_acta` (`actas.rs:198-199`), y con el texto del registro. Falla cerrada:
   un verificador que no conoce B no da VERDE a una cabeza de B, ni la prueba con A.

### D-F — El coste

**En bytes.** Ocho por cabeza firmada, 50 -> 58 B sobre una firma de 18.469 B (un 0,04 %); dieciséis
por cofirma con la decisión 3, 124 -> 140 B con la clave del operador de hoy, u ocho sin ella; cero
en el acta. En el cable, un campo QUANTITY por cabeza y uno o dos por `Cosig`. Ninguna permutación
de hash más: el preámbulo son bytes que el esquema resume al firmar, y nada del núcleo compone sobre
él.

**En vectores, medido.** Ninguno de los 405 ficheros con firma se reescribe: las cabezas v2 a v6
conservan su preámbulo y su veredicto. Lo que nace y lo que se muda:

- **Nacen**, en `spec/vectors/nucleo/`, tres KAT: el preámbulo v7 de la cabeza y el de la
  cofirma, al lado de `preambulo.json` (versión 3) y `preambulo_cofirma.json`, que no se tocan; y
  el del registro. En `spec/vectors/cable/`, un positivo v7 capturado de un nodo real, como pide
  la regla de «un positivo (`nueva`) por era de cabeza» (`spec/RPC.md:1262-1264`), y sus
  negativos: sin `esquema`, con un esquema que no está en el registro, con el cero, con el
  declarado distinto del firmado, y una v7 que no recompone.
- **Se mudan siete líneas de manifiesto, con sus vectores**, porque pinan entera la enumeración del
  conjunto de versiones que se acepta, que derivan `VersionCabeza::texto()` y sus hermanas: con la
  v7 dentro, «v2, v3, v4, v5 o v6» pasa a decir otra cosa. Son
  `spec/vectors/cable/MANIFIESTO.txt:17`, `:18` y `:21`, `spec/vectors/paquete/MANIFIESTO.txt:21` y
  `:25`, `spec/vectors/rechazo/MANIFIESTO.txt:50` y `spec/vectors/rotacion/MANIFIESTO.txt:72`,
  halladas con `grep` sobre los dieciocho manifiestos; que cambien es DERIVADO de leer quién compone
  su texto. Las que pinan sólo el comienzo —«exige una cabeza v5», «exige cabezas v4»— siguen
  valiendo, porque el manifiesto pide que la salida *contenga* su texto (`spec/RPC.md:1272`). Dos de
  las siete son de vectores que traen `formatVersion` 7 como versión desconocida
  (`cable/rechazo-formatVersion-7.json` y `paquete/rechazo-formatVersion-7.json`), y con la v7 su
  causa cambia. Por la regla 2 del PROCESO se mueven idénticos a `spec/vectors/0.4/`, con su línea
  de manifiesto, como movió el §682 los dos de la completitud
  (`spec/rfc/0019-la-completitud-que-no-se-esquiva.md:54-57`), y su papel lo toma un vector con otro
  nombre, con `formatVersion` 8. Antes del §692 una línea así se reescribía; desde entonces su
  huella lo impide (`spec/vectors/HUELLAS.sha256:1-8`).
- **En el corte**, los positivos `zkssl/0.5` de cada familia del kit que la D-G del RFC-0018 ya
  prevé (`spec/rfc/0018-el-tren-0-5.md:398-403`) llevan cabezas v7 sin un fichero más.

### D-G — Su relación con el corte `zkssl/0.5`

Una cabeza v7 es una ruptura para quien verifica hoy: el testigo y el kit 0.4.x la rechazan, con
«formatVersion 7: el testigo recompone cabezas v2, v3, v4, v5 o v6»
(`spec/vectors/cable/MANIFIESTO.txt:21`) y su par del paquete, a partir del texto de la biblioteca
(`lib.rs:269-273`): el fallo correcto, cerrado y con nombre. La rotura se paga una vez para varias
razones juntas (el corolario de la 82, `BACKLOG.md`, entrada 82), y el corte del RFC-0018 ya rompe
el cable y emite vectores nuevos. Así que, como lo adoptó el autor en el §799 (decisión 7), **viaja
en la misma rotura** (⚠️ §799: hasta aquí decía «si el autor lo adopta»): la E4 entra en el sello
del corte, y la D-E del RFC-0018 la gobierna —el kit 0.5.0 sale cuando el nodo
hable `zkssl/0.5`, no antes y sin aceptar las dos versiones del cable
(`spec/rfc/0018-el-tren-0-5.md:160-161`)—. El kit 0.5 acepta cabezas v2 a v7: las cabezas no son
pruebas, y su conjunto sólo crece. Lo que no cambia ningún byte ni ningún veredicto —el registro,
las dos funciones de preámbulo, sus KAT, la réplica de la segunda y el despacho con un solo esquema—
puede entrar antes, apagado (E2 y E3), como dice la D-G de su lector de la marca
(`spec/rfc/0018-el-tren-0-5.md:393-396`). `VersionCabeza::V7` no: cambia los siete textos de arriba,
y entra en el corte.

La D-G del RFC-0018 decidió poner la identidad de una prueba en la prueba y no en la cabeza
(`spec/rfc/0018-el-tren-0-5.md:342-347`). Esto no la contradice: cada objeto lleva su identidad, y
lo que no es prueba lleva su propia versión (su «Regla de extensión», `:334-340`). La marca dice qué
prueba es; el preámbulo v7, con qué se firmó la cabeza.

## Compatibilidad

Con la E4, el cable sube a `zkssl/0.5` en el corte del RFC-0018 (decisión 7, tomada en el §799).
Las cabezas v2 a v6 custodiadas siguen verificando con el kit de su versión y con el 0.5, con su
esquema implícito; los vectores de `zkssl/0.4` se quedan con sus bytes y su veredicto, y los siete
de arriba se mudan a `0.4/`. El kit 0.4.x no verifica una cabeza v7, y lo dice. El acta no cambia:
la decisión 4 tomó la rama (a). La E2 y la E3 no cambian nada que viaje. ⚠️ §799: hasta aquí
decía «en el corte del RFC-0018 o en uno propio (decisión 7)» y «El acta no cambia, salvo con la
decisión 4, rama (b)».

## Decisiones (del autor, §799)

⚠️ §799: hasta aquí esta sección se titulaba «Decisiones abiertas (del autor)» y empezaba «Ninguna
está tomada. La sesión dice lo que recomienda y por qué; decide el autor».

El autor aceptó este RFC en el §799, y con él tomó las siete decisiones: en cada una, la rama que el
texto recomienda. Las preguntas y sus ramas se quedan como se escribieron en el §790; detrás de
cada una, la tomada y las descartadas. El RFC sigue PROPUESTO hasta que se construyan sus etapas.

1. **¿Dónde va el identificador?** (a) En el preámbulo, v7, para la cabeza y la cofirma (D-B,
   D-C); (b) sólo en el acta, como hoy, exigiéndola para todo esquema que no sea el implícito —sin
   bytes ni vectores nuevos, pero sin nada para el testigo ni para un libro sin génesis—; (c) dentro
   de `epoch_digest`, que lo metería en el MMR, el ancla y los acuses. La sesión recomienda (a): es
   la única que cubre la cofirma y la cabeza sin acta, al precio de D-F, y (b) es la que el RFC-0015
   eligió para un nodo con actas.
   **Tomada (§799): (a)**, el preámbulo v7 de la cabeza y de la cofirma. Descartadas: (b) y (c).
2. **¿Cómo se escribe?** El `u64` del acta, ocho bytes (recomendado: una escritura por esquema), o un
   índice de una tabla en uno o dos bytes, que ahorraría seis y obligaría a traducir entre el acta y
   la cabeza.
   **Tomada (§799): el `u64` del acta**, ocho bytes. Descartado: el índice de una tabla.
3. **¿La cofirma firma también el esquema del operador?** Ocho bytes más por cofirma. La sesión lo
   recomienda, por la razón por la que ya firma su clave.
   **Tomada (§799): sí**, `esquema_testigo` y `esquema_op` en la cofirma v7 (D-C). Descartado:
   firmar sólo el del testigo.
4. **¿El compromiso de la sucesora ata su esquema?** (a) Una regla del registro: dos esquemas
   aceptados no comparten ninguna escritura de clave, y el registro lo comprueba al admitir uno; (b)
   un acta v2 que compone `esquema_siguiente` junto a `siguiente` (dos KAT nuevos, y las actas
   nuevas con otra versión); (c) una huella de clave v2 que cubra el esquema, que cambiaría el ancla
   del RFC-0012, que la compone. La sesión recomienda (a): no cambia nada firmado, y (c) toca un RFC
   ACEPTADO.
   **Tomada (§799): (a)**, la regla del registro. Descartadas: (b), el acta v2, y (c), la huella v2.
5. **¿Cuándo sale un esquema del conjunto aceptado?** Sólo si se rompe, con la regla de contención
   de la D-E (5) (recomendado); o también cuando su biblioteca muere, que dejaría sin verificar
   cabezas que nadie ha roto.
   **Tomada (§799): sólo si se rompe**, con la regla de contención. Descartado: también cuando su
   biblioteca muere.
6. **¿Dónde vive el registro?** `spec/NUCLEO.md` §6, con un productor y su KAT (recomendado, como el
   de familias de la D-G), o una tabla en el código, sin spec.
   **Tomada (§799): `spec/NUCLEO.md` §6**, con un productor y su KAT. Descartada: la tabla en el
   código, sin spec.
7. **¿Viaja en el corte `zkssl/0.5`?** Sí, en el mismo sello que el corte, con la E2 y la E3 antes
   (recomendado); o en un corte propio después, que pagaría otra rotura, otro kit y otra mudanza de
   vectores; o no se adopta, y la 87 sigue abierta con lo que el acta da.
   **Tomada (§799): sí**, en el mismo sello que el corte, con la E2 y la E3 antes. Descartadas: un
   corte propio después, y no adoptarlo.

## Lo que se descarta

- **El byte de versión como esquema** (v7 para XMSS, v8 para otro): mete dos ejes en un marcador.
  La casa ya los separó una vez: `VERSION_FORMATO` dice qué campos entran en la firma y
  `COFIRMA_VERSION` qué formato tiene la cofirma, y «son dos ejes distintos y no se unifican»
  (`lib.rs:294-300`).
- **Probar los verificadores aceptados en orden** hasta que uno dé VERDE: el resultado lo decide
  el más débil del conjunto, y una firma no dice cuál la aceptó.
- **Leer el esquema de la clave**, como hace hoy la segunda: el OID es ambiguo dentro de RFC 8391
  (hecho 2) y no va firmado.

## Seguridad

- **La clave de gasto no viaja jamás**: este RFC no la toca. Todas sus firmas son del operador y de
  los testigos.
- **Bajar de esquema.** Con la D-E (1) y (2), quien verifica elige por lo que la firma declara y
  ata: una firma de B no se presenta como de A, porque el preámbulo recuperado no casaría, y una de
  A sobre un preámbulo v7 que diga B tampoco.
- **Un esquema roto.** Sus firmas se pueden fabricar desde la rotura. La D-E (5) deja en pie sólo las
  cabezas de A que una firma de B contiene, y una cabeza fabricada no está en el MMR que B firmó.
- **«Si no hay transición firmada por la clave anterior, cambiar de esquema es indistinguible de un
  compromiso de clave»** (la 87, que compone con la 84). Con A rota, su firma sobre el acta de B ya
  no prueba nada; lo que sostiene la rotación es el compromiso `siguiente` que A firmó cuando era
  sana, el de la pre-rotación del RFC-0015 (D-B). Sin acta génesis no hay ese compromiso, y la
  rotación a otro esquema queda donde la D-I del RFC-0015 deja a todo operador sin génesis. DERIVADO
  de leer el código y el RFC-0015; no medido.

## Lo que este RFC no decide

Qué esquema sería el B, ni con qué banco se elige (la 108 y la adenda de la 87: las tres primitivas
medidas en el hardware real, con el método del §263). La dependencia `xmss` y su fijación. El
esquema del medio, que es el de C2SP y ya lleva su tipo (RFC-0013, D-C). La escritura de la firma de
un esquema B, que la da su entrada del registro. Y la política de compatibilidad que dice cuándo un
esquema sale del conjunto: la escribe el RFC-0021, que el autor aceptó en el mismo §799. ⚠️ §799:
hasta aquí decía «la propone el RFC-0021».

## Referencias

- `BACKLOG.md`, entradas 84, 87 (con su adenda de ML-DSA y SLH-DSA) y 108.
- `AUDITORIA.md` §236, §240, §275, §291, §293, §297, §406, §643, §644, §682, §692, §699 y §780.
- RFC-0015 (`spec/rfc/0015-el-ciclo-de-vida-del-firmante.md` §«D-C — El acta», §«D-I — El residuo,
  declarado» y §«Lo que se DESCARTÓ al medir»); la D-G del RFC-0018 y su D-E; el RFC-0013, D-C; el
  RFC-0016.
- `spec/NUCLEO.md` §«3. La regla de extensión, ejercida» y §«6. Los bytes: lo que un KAT fija».
- RFC 8391 (XMSS), FIPS 204 (ML-DSA) y FIPS 205 (SLH-DSA).
