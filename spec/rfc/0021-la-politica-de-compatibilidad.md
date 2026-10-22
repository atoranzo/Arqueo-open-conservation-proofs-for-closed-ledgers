# RFC-0021 — La política de compatibilidad: qué superficie es estable, qué la rompe, con cuánto aviso, y qué sigue verificando después

- **Estado:** PROPUESTO (§790) — el texto, la superficie enumerada, la regla de ruptura, la enmienda
  que propone a `spec/RPC.md` y sus ocho decisiones, escritos sobre lo medido en el árbol en
  `8c02941` y en el §699. El autor lo aceptó en el §799, con sus ocho decisiones TOMADAS, en cada
  una la que el texto recomienda: ver «Decisiones (del autor, §799)». La E2 y la E3a, construidas
  en el §801; la E3b y la E4 llegan con el corte, y ACEPTADO con ellas, porque el paso a ACEPTADO
  exige construir sus etapas (regla 4 del PROCESO). ⚠️ §799: hasta aquí decía «Ninguna decisión
  está tomada y ninguna etapa construida: adoptarlo es del autor». ⚠️ §801: hasta aquí decía
  «Ninguna etapa construida: ACEPTADO llega con ellas».
- **Autor:** Ángel José Toranzo Portela
- **Asistencia GenAI:** Claude, en una sesión de Claude Code en la nube (§790): midió el árbol y
  redactó el borrador, fuera del paso 4 de `GENAI.md` — ver [`GENAI.md`](../../GENAI.md)
- **Fecha:** 2026-10-14
- **Versión del protocolo afectada:** `zkssl/0.4` — **no cambia ningún byte ni ningún veredicto**:
  dice cómo cambian. Su primera aplicación será el corte `zkssl/0.5` del RFC-0018. ⚠️ §799: hasta
  aquí decía «sería».
- **Asiento(s) de AUDITORIA:** §209 (el `0.2`) y §275, §278 y §281 (sus re-emisiones), §290 (lo
  custodiado no caduca), §418 (el `tipo` desconocido, con nombre), §641, §680 y §684 (las pruebas
  que cambiaron bajo `zkssl/0.4`), §662 a §664 (la 0.4.2), §682 (la primera mudanza a `0.4/`), §692
  (las huellas), §699 (lo que `zkssl/0.4` no identifica, medido), §707 (la D-G), §716 (la emisión de los vectores, que escribe desde entonces la versión del escenario, con su marca en el hecho 5), §790 (este
  borrador), §799 (la aceptación del autor, con sus ocho decisiones), §800 (las cifras de S3 y S5 en la D-A, con la E2 del RFC-0020), §801 (la E2 y la E3a), §807 (las frases rancias, con su marca), §810 (la cita del bloque «3 ter» de `tools/canon.sh`), §823 (la condición de la máquina en «reproducible», y las citas de `doc/KIT.md` que el §815 y el §818 habían corrido, con sus marcas), §826 (desde cuándo rige la promesa, precisado, y qué verificador juzga las positivas de la E3b), §852 (la cita de `spec/vectors/paquete/MANIFIESTO.txt` del hecho 7, que sus cinco negativos corrieron, con su marca). ⚠️ §826: la lista acababa en el §799, y callaba el §800, el §801, el §807, el §810, el §823 y el §852, que tocaron este fichero: el §801 lo nombra su línea de estado; el §800, sus cifras de la D-A; y el §807, el §810, el §823 y el §852, sus marcas.
- **Backlog:** la **90** (la política de compatibilidad, como promesa), que este RFC no cierra hasta
  que se construya; con la **85** (una segunda implementación necesita contra qué comprometerse),
  la **82** (una ruptura para varias razones), la **118** (cómo se identifica una prueba) y la
  **87** (el esquema de firma, que escribe el RFC-0020, aceptado con éste en el §799). ⚠️ §799:
  hasta aquí decía «que propone el RFC-0020».

## Estado de las etapas

| etapa | qué entrega | ¿rompe el cable? | estado |
|---|---|---|---|
| E1 — la promesa, escrita | este texto: la superficie, la regla de ruptura, el aviso y lo que se garantiza después | no | **escrita (§790) y decidida (§799)** ⚠️ §799: hasta aquí decía «propuesta (§790)» |
| E2 — la sección normativa | la sección «Versionado y compatibilidad» de `spec/RPC.md`, con el texto de la D-C, y las remisiones desde `spec/PAQUETE.md` §6, `spec/NUCLEO.md` §3, `doc/KIT.md` §0 y la fila «Estado» de los dos README | no (texto) | **construida (§801)** ⚠️ §801: hasta aquí decía «sin construir» |
| E3a — la compuerta de la superficie | una herramienta del bucle del canon que deriva cada superficie de su productor y la compara con la lista de la sección (D-D) | no | **construida (§801)**: `tools/check_superficie.py`, nombre provisional que confirma el autor ⚠️ §799: hasta aquí decía «propuesta, no escrita» ⚠️ §801: hasta aquí decía «sin construir; **decidida (§799), no escrita**» |
| E3b — la compuerta de las pruebas que viajan | una prueba positiva por familia que viaja, juzgada en cada canon por el verificador del nodo y del kit (D-D) ⚠️ §826: el del kit, sólo en las cinco familias del kit; las de la capa las juzga el del nodo, porque el kit no verifica pruebas de envío, cobro, quema ni umbral (`spec/rfc/0018-el-tren-0-5.md:110`), como dice la «Compatibilidad» del RFC-0017 | no por sí misma | sin construir; con la E3 de la D-G del RFC-0018, en el corte |
| E4 — el corte, bajo la política | el corte `zkssl/0.5` con su aviso, sus mudanzas y el kit 0.5.0, como primera ruptura que la cumple | **sí**: la del RFC-0018, y con ella las del RFC-0017, el RFC-0019 y el RFC-0020; qué superficie rompe cada etapa lo dice la tabla de su RFC ⚠️ §826: decía «**sí** (la del RFC-0018)» ⚠️ §865: y la del RFC-0023 si llega construido, como propone su decisión 9, sin acordar con quien lleva el tren | sin construir |

## Motivación

La 90 lo dice así: «un protocolo es su promesa, no su historial: qué superficie es estable, qué
puede romperse, con cuánto aviso, y qué se garantiza que seguirá verificando después de una
rotura» (`BACKLOG.md`, entrada 90). Pide una sección normativa con la superficie enumerada y la
regla de ruptura escrita, y una compuerta que la haga cierta («Qué lo cierra»). Su última línea, del
§699: «la promesa, y cómo se identifica una prueba, siguen abiertas».

**Lo que ya está escrito, y dónde.** Hay piezas de la promesa, sueltas:

| pieza | dónde | quién la hace cumplir |
|---|---|---|
| si una ruptura cambia el cable, la versión sube y los vectores viejos se conservan bajo la suya | regla 2, `spec/rfc/PROCESO.md:16-21` | `tools/check_vectores.py` contra `spec/vectors/HUELLAS.sha256` desde el §692 (`HUELLAS.sha256:1-8`); el emisor la respeta desde el §781 |
| la versión del cable sube cuando cambian «los valores que viajan», no el tamaño de la superficie | `spec/RPC.md:1290-1295` («Versionado», en «Notas operativas»), corregida en el §699 (`:1296-1298`) | nada: el test `el_aviso_de_compatibilidad_va_con_la_version` ata el aviso a la versión (`crates/zk-ssl-wire/src/openrpc.rs:400`), no la regla |
| lo que se firma crece sólo por versión; el conjunto de versiones de cabeza tiene un productor | `spec/NUCLEO.md` §«3. La regla de extensión, ejercida» | el compilador, sobre `VersionCabeza` (`crates/zk-ssl-verify/src/lib.rs:181-210`) |
| lo custodiado no caduca: el kit lee el paquete v1 y el v2 | `spec/PAQUETE.md:73` (§290) | los catálogos del kit en el canon |
| un `tipo` desconocido se rechaza con nombre | `crates/zk-ssl-verify/src/main.rs:183-221` (§418) | sus tests y su catálogo |
| ningún kit acepta dos versiones del cable | la D-E del RFC-0018, `spec/rfc/0018-el-tren-0-5.md:160-161` | nada todavía: es del corte |
| una versión nueva de AIR, de perfil o de marca es versión nueva del cable | la «Regla de extensión» de la D-G, `spec/rfc/0018-el-tren-0-5.md:334-340` | nada todavía: es del corte |
| un kit viejo sigue publicado y reconstruible desde su commit (⚠️ §823: decía «reproducible»: con la misma huella sólo está demostrado dentro de una misma máquina, y la 0.4.2 rehecha desde su commit en otra sale con 13 de sus 35 secciones distintas) | `doc/KIT.md:76-123` (⚠️ §823: decía `:19-63`; el §815 metió en medio el aviso de la 0.4.3, y el §818, que la da por vigente, el párrafo de la 0.4.2): las 0.2.0, 0.3.0, 0.4.0, 0.4.1 y 0.4.2, todas publicadas | nada lo exige; `tools/artefacto.sh --check` comprueba en cada canon que el árbol reproduce su artefacto en la máquina que lo corre y desde su misma ruta, en dos `target` (el bloque «3 ter» de `tools/canon.sh`; ⚠️ §810: hasta aquí citaba sus líneas, la `:598-603`, que el §798, el §800 y el §804 ya habían corrido a la `:690-695`, y las catorce que el §810 mete en el «3 duodecies» las llevan a la `:704-709`: se cita por su bloque), no que los publicados sigan ahí; ⚠️ §823: decía sólo «que el árbol reproduce su artefacto», sin decir en qué máquina ni desde qué ruta |

**Lo que el §699 midió, y es el punto de partida.** La práctica no fue siempre la promesa:

1. **`zkssl/0.4` no identifica qué pruebas acepta un nodo.** El AIR de las pruebas que viajan
   cambió tres veces sin subir la versión —en el §641, el §680 y el §684—, y medido entre nodos y
   clientes de seis puntos de la historia que contestan todos `zkssl/0.4`, cuando entre los dos
   queda el §641 o el §680 el envío no verifica en ningún sentido, con `VerificationFailed`
   («envio: InconsistentOodConstraintEvaluations») (`AUDITORIA.md` §699; `spec/RPC.md:1299-1316`).
   La versión siguió porque la conformidad de `0.4` no compara bytes de prueba (RFC-0009, D-AI).
2. **`spec/vectors/zkssl-0.2.json` se re-emitió tres veces bajo su versión**, en el §275, el §278 y
   el §281 (`spec/RPC.md:1319-1325`). Desde el §692 la huella lo impide.

**Y cinco hechos más, medidos en este árbol, que una promesa tiene que decir:**

3. **El nodo cambió de versión de cabeza sin subir el cable**, cuatro veces: la v3, «aditivo en el
   cable, rotura en el parser: `zkssl/0.2` no sube» (`spec/RPC.md:602-603`); la v4, la v5 y la v6,
   con «`zkssl/0.3` no sube» (`spec/RPC.md:623-624`, `:656` y `:678`); y la doc de `VERSION_FORMATO`
   dice lo mismo de la v5 y la v6 (`crates/zk-ssl-verify/src/lib.rs:165-170`). Un verificador
   anterior rechaza la cabeza nueva, cerrado y con nombre; pero lo que el nodo emite cambió sin que
   la versión lo dijera, que es la clase del §699 aplicada a las cabezas. De la v6, además, los dos
   textos dan la versión equivocada: la v6 es del §570, y el cable estaba en `zkssl/0.4` desde el
   §538 (`spec/RPC.md:1290-1292`). No se corrigen aquí: lo hará el sello que el autor elija.
   ⚠️ §807: corregidos en el §807, con su marca, dentro de la lista de tareas que el autor encargó.
4. **Dentro de `0.4.x` dos arreglos cambiaron veredictos.** La 0.4.2 rechaza lo que la 0.4.1 aceptaba:
   un `u64` con `+` o con ceros a la izquierda (§662), la misma cofirma contada varias veces (§663)
   y la clave con el OID alterno, que daba VERDE a un conflicto con un solo libro (§664)
   (`doc/KIT.md:87-93`; ⚠️ §823: decía `:30-35`, que corrieron el §815, con su aviso de la 0.4.3, y el §818); y la 0.4.1 ya rechazaba la prueba con nodos de más dentro de un lote que la
   0.4.0 verificaba (§659, `doc/KIT.md:94-100`; ⚠️ §823: decía `:36-41`). Es lo correcto, y ninguna regla lo dice. La línea
   del kit tampoco ha seguido a la del cable: la 0.3.0 se cortó en el §619, con el cable en
   `zkssl/0.4` desde el §538, y la 0.4.0 (§657) rechaza lo que la 0.3.0 aceptaba
   (`doc/KIT.md:102-108`; ⚠️ §823: decía `:43-49`) sin que el cable subiera.
5. **La versión del cable se escribe en cuatro sitios del código de producción, y nada los ata**:
   la respuesta del nodo (`crates/zk-ssl-node/src/main.rs:2822`), el OpenRPC dos veces
   (`crates/zk-ssl-wire/src/openrpc.rs:239` y `:269`) y la emisión de los vectores
   (`crates/zk-ssl-cli/src/conformance.rs:106`). Los tests que la comprueban la escriben otra vez
   como literal (`crates/zk-ssl-node/src/main.rs:5129` y `:5162`; `openrpc.rs:355` y `:403`). Medido
   con `grep` del literal en `crates/` y `tools/`. ⚠️ §716: la emisión de los vectores dejó de ser una escritura del cable: desde el §716 escribe la versión del escenario de conformidad, `zkssl/0.5`, que sube antes que el cable porque el compromiso de las dos emisiones lleva el contador de emisiones (RFC-0018, «Cómo se construye»). La lista de `spec/RPC.md` la fija aparte, en `escenario`, y `tools/check_superficie.py` la compara en su R1; las escrituras del cable quedan en tres.
6. **El contrato del kit tiene cuatro códigos de salida, y uno de sus documentos cuenta tres.**
   `codigo_de_salida` da 0, 1 o 3, y el uso incorrecto 2
   (`crates/zk-ssl-verify/src/main.rs:1789-1795` y `:2937`); `spec/PAQUETE.md:1187-1190` dice los
   cuatro; `doc/KIT.md:137-139` (⚠️ §823: decía `:78-80`), «sale 0 … 1 … 2».
7. **Una cabeza v1 no la verifica el kit, la verifica la biblioteca.** El testigo y el paquete
   rechazan `formatVersion` 1 (`spec/vectors/cable/MANIFIESTO.txt:17`,
   `spec/vectors/paquete/MANIFIESTO.txt:26`; ⚠️ §852: decía `:21`, y las cinco líneas que el §852
   intercala tras la `:10` la bajan cinco), y «una cabeza v1 se verifica con la biblioteca, no con
   este mando» (`spec/PAQUETE.md:742-743`); la segunda implementación también la recompone
   (`tools/segunda/juez_cabezas.py:51-68`). Lo que la 90 llama «una cabeza v1 sigue verificando bajo
   binario v2» es cierto de la biblioteca y de la segunda, no del binario del kit.

## Diseño

### D-A — La superficie estable, enumerada

Seis superficies. Cada una con lo que la forma, su identificador y su productor, medidos hoy:

| | superficie | lo que la forma | su identificador | su productor, hoy |
|---|---|---|---|---|
| S1 | **el cable** | los 29 métodos `zkssl_*`, sus parámetros, sus resultados, sus errores con la causa en `data`, y la codificación (`spec/RPC.md`) | `zkssl/0.N`, la respuesta de `zkssl_protocolVersion` | cuatro escrituras sin atar (hecho 5; ⚠️ §716: tres, con la del escenario aparte); `spec/openrpc.json`: `info.version` `zkssl/0.4`, 32 métodos, 29 `zkssl_*` y 3 `dev_*` |
| S2 | **las pruebas que viajan** | los bytes que juzgan el nodo y el kit | ninguno bajo `zkssl/0.4` (§699); la marca v2 desde `zkssl/0.5`, si se construye la D-G | los 23 probadores con fila |
| S3 | **lo firmado** | la cabeza (v2 a v6), la cofirma y el acta: sus preámbulos y sus composiciones, con sus KAT (31 en `spec/vectors/nucleo/`; 34 desde el §800, con los tres de la E2 del RFC-0020, apagados) | `formatVersion`; la versión de cofirma; `ACTA_VERSION`; el esquema, si se construye el RFC-0020 | `VersionCabeza` (`lib.rs:181-210`), `spec/NUCLEO.md` §6 |
| S4 | **los sobres del kit** | las catorce formas de `spec/PAQUETE.md` §«2. Las catorce formas», con su catálogo (§5) y sus causas (§6.1) | `v` (1 o 2) y `tipo` (doce) | `crates/zk-ssl-verify/src/main.rs:171` y `:190-221` |
| S5 | **los vectores** | todo fichero de `spec/vectors/`: 492 `.json` y 982 líneas de 18 manifiestos, cada uno con su huella (495 `.json` desde el §800, con los tres KAT de la E2 del RFC-0020) | su ruta, su línea de manifiesto y su huella | `spec/vectors/HUELLAS.sha256`; `tools/check_vectores.py` |
| S6 | **el kit como binario** | `arqueo-verify-vX.Y.Z`: su tarball, su binario y el contrato de `spec/PAQUETE.md` §«6. El contrato del mando» —un argumento, cuatro códigos de salida, las líneas de VERDE y el ROJO con una causa de §6.1— | el tag, el commit y las dos huellas, juntos (`doc/KIT.md:125-127`; ⚠️ §823: decía `:66-68`, que corrieron el §815, con su aviso de la 0.4.3, y el §818) | `tools/artefacto.sh`; su `FAMILIAS`, `tools/artefacto.sh:31` |

**Fuera de la promesa**, y dicho para que nadie lo dé por prometido: los `dev_*`, que sólo existen
con `--dev`; la API de Rust de los crates —el SDK, y `zk-ssl-verify` como biblioteca—; las opciones
de línea de mando de `zk-ssl-cli` y del testigo; los diarios del nodo (`DIARIO_VERSION` 1,
`crates/zk-ssl-node/src/diario.rs:67`) y del testigo (5, `crates/zk-ssl-cli/src/witness.rs:610`;
⚠️ §714: decía 4, en la `:478`; la clase `bifurcacion` del §714 la sube); el
libro en disco y sus instantáneas; los límites de operación, como el del cuerpo de una petición
(`spec/RPC.md`, «Transporte y sobre»); los tiempos y los tamaños; y `doc/`. La decisión 2 la tomó
el autor en el §799: estas seis superficies, y fuera de la promesa lo que aquí se enumera, el diario
del testigo y `zk-ssl-verify` como biblioteca incluidos. ⚠️ §799: hasta aquí decía «La decisión 2
es del autor».

### D-B — La regla de ruptura

**R1 — Qué es una ruptura.** Un cambio por el que una implementación conforme con la versión
anterior produce o acepta otra cosa en una superficie de la D-A:

- cambia un valor que viaja, **también los bytes de una prueba o su AIR** —lo que el §699 midió—, o
  lo que el nodo firma, también una versión de cabeza nueva que el nodo **emite** (hecho 3);
- se quita o se renombra un método, un campo o un `tipo`, o cambia su tipo;
- cambia el veredicto o el texto pinado de un vector publicado;
- una versión sale de un conjunto aceptado: de cabeza, de paquete, de marca o de esquema;
- cambia un código de salida o una línea de VERDE del contrato del kit.

**R2 — Qué no lo es.** Lo aditivo que ningún productor publicado emite todavía: un método nuevo, un
campo opcional, un `tipo` nuevo, un KAT o un vector con una ruta nueva, y una versión nueva en un
conjunto **aceptado** que nadie emite —quien lee puede ir por delante de quien escribe y nunca al
revés (el molde de `COFIRMA_V_MAX`, `crates/zk-ssl-verify/src/lib.rs:308-314`)—. Un texto pinado
que deriva del conjunto aceptado sí cambia al ensancharlo, y eso es R1: el vector se muda (R6 b).

**R3 — Qué sube.** Una ruptura de S1, S2 o S3 sube `zkssl/0.N`, que pasa a tener **un** productor
del que se derivan las demás escrituras (R7). Una de S4 sube el `v` del sobre o entra como `tipo`
nuevo. En S5 no se rompe: se muda (R6 b). En S6, el kit sube su versión menor con el cable, `0.N.0`,
y sólo entonces —la D-E del RFC-0018 lo dice del `0.5.0`; esto lo extiende, y cambia la práctica
de la 0.3.0 y la 0.4.0 (hecho 4)—; un `0.N.x` no rompe S1 a S4, salvo por R6 f.

**R4 — Una ruptura se paga una vez.** Las que estén listas se juntan en un corte, el corolario de
la 82 («se rompe una vez para VARIAS razones juntas», `BACKLOG.md`, entrada 82), como el tren del
RFC-0018.

**R5 — El aviso.** El RFC que rompe vive en el árbol, PROPUESTO, al menos un sello antes del corte,
con su tabla de etapas diciendo qué superficie rompe cada una; y el sello del corte escribe el aviso
donde se lee el cable —la línea bajo el título de `spec/RPC.md` (`:2`), la descripción del
OpenRPC, `doc/KIT.md` §0 y la fila «Estado» de los README—, como hizo el §699 con lo que ya había
roto. Un arreglo de seguridad sigue `spec/rfc/PROCESO.md` §«Fallos de seguridad»: se redacta en
privado y entra de una vez, con su aviso de `SECURITY.md` §5. Cuánto aviso es la decisión 4.

**R6 — Lo que sigue verificando después de una ruptura.** Es lo que se promete:

- **a. Ningún kit publicado se retira**, y cada uno sigue siendo reconstruible desde su commit
  (`doc/KIT.md` §«Reproducir el kit, para quien no se fíe de la descarga»); con la misma huella, sólo
  demostrado dentro de una misma máquina. Lo que juzgaba, lo sigue juzgando igual. ⚠️ §823: decía
  «reproducible desde su commit», sin la condición de la máquina: la 0.4.2 rehecha desde su commit en
  otra sale con otra huella, y juzga igual sus once catálogos.
- **b. Ningún vector se reescribe.** Uno cuyo veredicto o texto cambia se muda idéntico, con su
  línea de manifiesto, a `spec/vectors/<versión>/`, conserva allí el veredicto del kit publicado de
  su versión, y su ruta vieja no se reutiliza (regla 2; `spec/vectors/HUELLAS.sha256:4-7`). Es lo
  que hizo el §682 con dos vectores de la completitud
  (`spec/rfc/0019-la-completitud-que-no-se-esquiva.md:54-57`).
- **c. Una cabeza firmada que un kit aceptó la acepta todo kit posterior**: el conjunto de
  versiones de cabeza sólo crece. La v1, con la biblioteca y la segunda (hecho 7). La excepción,
  si se construye el RFC-0020: una cabeza firmada con un esquema que se rompa vale sólo contenida
  en algo firmado con un esquema aceptado (su D-E, regla 5).
- **d. Un sobre que un kit dio por VERDE** lo sigue dando el kit de su versión; uno posterior lo
  verifica o lo rechaza con nombre en la clase «fuera del contrato» de `spec/PAQUETE.md` §«6.1 La
  causa del rechazo», nunca con otra causa, salvo por R6 f.
- **e. Una prueba de `zkssl/0.N` la juzga el kit `0.N.x`**: ningún kit acepta dos versiones del
  cable, porque aceptar las dos abre una vía para bajar de versión (la D-E del RFC-0018). Un kit
  posterior la rechaza con nombre, como propone la D-G (los pasos 1b y 1c de su tabla).
- **f. La excepción: un arreglo de seguridad** puede cambiar un veredicto dentro de `0.N.x`. Sale con
  su aviso y la release dice qué veredictos cambia y por qué, como la 0.4.2 (hecho 4); los vectores
  que lo muestran entran con rutas nuevas.

**R7 — Cada superficie, un identificador y un productor.** Dos marcadores que pueden discrepar
valen menos que uno (§236). Hoy faltan tres: la versión del cable tiene cuatro escrituras (hecho 5; ⚠️ §716: tres, desde que la emisión de los vectores escribe la del escenario);
una prueba de `zkssl/0.4` no tiene identificador (la D-G del RFC-0018, la 118); y una firma no dice
su esquema (el RFC-0020, la 87). Mientras falten, la sección los declara.

### D-C — La enmienda a `spec/RPC.md`: la sección «Versionado y compatibilidad»

`spec/RPC.md` ya tiene un «Versionado»: una entrada de «Notas operativas» (`spec/RPC.md:1290-1298`),
con las dos del §699 detrás (`:1299-1325`), y la línea del aviso, bajo el título, remite a ella
(`spec/RPC.md:2`). Así que este RFC no escribe una sección en `spec/RPC.md`: propone su enmienda, y
la E2 la aplica; el autor lo adoptó en el §799 (⚠️ §799: hasta aquí decía «la E2 la aplica si el
autor lo adopta»). La sección nueva va **al final del documento**, detrás de
«Notas operativas», para no mover ninguna línea que otros documentos citen —el §699 dejó `:9`,
`:88`, `:101` y `:164-165` donde estaban—. La entrada «Versionado» no se borra: gana en su misma
línea una nota «⚠️ §M:» que remite a la sección nueva, y sigue contando lo que pasó antes. El texto
que se propone:

> ## Versionado y compatibilidad
>
> Esta sección es la promesa del protocolo a quien lo implementa y a quien custodia lo que produce.
> Rige desde el §799, el sello en que el autor aceptó el RFC-0021 con sus decisiones —el RFC sigue PROPUESTO hasta que se construyan sus etapas (regla 4 de `rfc/PROCESO.md`)—, y está escrita aquí desde el §801; lo de antes lo cuenta «Versionado», en «Notas
> operativas», que no se reescribe. ⚠️ §826: decía «Rige desde el sello que acepte el RFC-0021». El §799 aceptó sus decisiones y lo dejó PROPUESTO; leída como el sello que lo pase a ACEPTADO, que llega con su E4, la promesa regiría desde el corte que la estrena. Pero la decisión 4 del autor en el §799 fija el aviso en un sello con el RFC PROPUESTO antes del corte (R5), y eso sólo obliga si la promesa rige mientras el RFC está PROPUESTO. El §801 dejó al autor decir cuál de los dos sellos es; lo precisa el §826, por su delegación («HACER todo») y con ese criterio, y es reversible. La misma precisión va en esta sección de `spec/RPC.md` y en la D-C del RFC-0021, que la propone.
>
> **La superficie estable** son seis superficies, cada una con su identificador: (1) el cable,
> `zkssl/0.N`: los métodos `zkssl_*` de `openrpc.json`, sus parámetros, sus resultados, sus errores
> con su causa en `data`, y la codificación; (2) las pruebas que viajan, por la marca de su traza;
> (3) lo firmado —la cabeza, la cofirma y el acta—, por su versión, con sus preámbulos y sus
> composiciones (`NUCLEO.md` §6); (4) los sobres del kit, por su `v` y su `tipo` (`PAQUETE.md` §2),
> con su catálogo y sus causas; (5) los vectores de `vectors/`, por su ruta, su línea de manifiesto
> y su huella; (6) el kit, `arqueo-verify-vX.Y.Z`, por su tag, su commit y sus huellas, con el
> contrato de `PAQUETE.md` §6. Lo demás —los `dev_*`, la API de los crates, las opciones de línea de
> mando, los diarios, el libro en disco y los límites de operación— no está prometido.
>
> **Rompe** lo que hace que una implementación conforme con la versión anterior produzca o acepte
> otra cosa en una de las seis: un valor que viaja, también los bytes de una prueba o su AIR, y lo
> que el nodo firma, también una versión de cabeza nueva que emita; quitar o renombrar un método, un
> campo o un `tipo`; el veredicto o el texto de un vector publicado; sacar una versión de un conjunto
> aceptado; un código de salida del kit. **No rompe** lo aditivo que nadie emite todavía.
>
> **Sube** `zkssl/0.N` con toda ruptura de (1), (2) o (3); el `v` del sobre, o un `tipo` nuevo, con
> una de (4); el kit pasa a `0.N.0` con el cable y no antes, y un `0.N.x` no rompe nada salvo por un
> arreglo de seguridad. Las rupturas se juntan en un corte, y su RFC está en el árbol antes que él.
>
> **Después de una ruptura sigue verificando**: todo kit publicado, que no se retira y se
> reconstruye desde su commit (⚠️ §823: decía «y se reproduce desde su commit»; con la misma huella,
> sólo demostrado dentro de una misma máquina, como dice ya `spec/RPC.md`); todo vector, que no se
> reescribe y, si su veredicto cambia, se muda a `vectors/<versión>/` con el de su kit; toda cabeza
> que un kit aceptó, en todo kit posterior, porque el conjunto de versiones de cabeza sólo crece,
> salvo la de un esquema de firma roto, que vale contenida en algo firmado con uno aceptado; todo
> sobre, con el kit de su versión, y con uno posterior o verifica o se rechaza como «fuera del
> contrato»; y toda prueba, con el kit de su versión del cable, porque ningún kit acepta dos. Un
> arreglo de seguridad puede cambiar un veredicto dentro de `0.N.x`, con su aviso y la lista de lo
> que cambia.
>
> **Lo que la versión no identifica todavía** se declara aquí, con su asiento, hasta que lo
> identifique.

Las cifras de la sección —cuántos métodos, cuántas superficies— no van en el texto normativo, sino
en la lista que la compuerta de la E3a compara, para que la prosa no las tenga que perseguir.

### D-D — La compuerta, propuesta como etapa y no escrita

La 90 pide «una compuerta que la haga cierta: lo declarado estable no cambia sin que el canon lo
diga». Este RFC no la escribe: no estaba aceptado cuando se escribió (§790), y lo que vigilaría es
la sección que propone; aceptado en el §799, la compuerta llega con su etapa. La propone en dos
etapas. ⚠️ §799: hasta aquí decía «no está aceptado», sin más. ⚠️ §801: la E3a está construida:
`tools/check_superficie.py`, nombre provisional, con las siete reglas de abajo en su cabecera, y la
lista que compara al final de la sección «Versionado y compatibilidad» de `spec/RPC.md`.

**E3a — La superficie, por su productor.** Una herramienta del bucle del canon,
`tools/check_superficie.py` o el nombre que el autor elija, que lee cada superficie de su productor
y la compara con la lista fijada para la versión vigente:

1. **la versión del cable**: un productor, y cada escritura de producción igual a él (hoy cuatro,
   hecho 5); y la sección, el OpenRPC y la respuesta del nodo dicen la misma;
2. **los métodos** de `spec/openrpc.json`, con el esquema de los parámetros y del resultado de cada
   uno: uno que falta, que cambia de nombre o de tipo con el mismo `info.version` es ROJO; uno nuevo
   se cuenta y no es ROJO (R2);
3. **las versiones de cabeza**: `VersionCabeza::TODAS` sólo crece frente a la lista, y
   `VERSION_FORMATO` es miembro;
4. **los sobres**: los `tipo` del despacho del kit y los `v` que lee sólo crecen frente a
   `spec/PAQUETE.md` §2;
5. **el contrato del kit**: los cuatro códigos de `codigo_de_salida` y del uso, frente a
   `spec/PAQUETE.md` §6 y frente a `doc/KIT.md` §0 (⚠️ §801: decía «hoy discrepan, hecho 6»; el
   §801 corrigió `doc/KIT.md` y `doc/KIT_EN.md`, y ya no discrepan);
6. **los vectores**: no repite `tools/check_vectores.py`; lo exige en el mismo canon;
7. **prueba de vida en los dos sentidos**, como la del §787: una mutación de cada lista la pone roja,
   y una lista vacía también.

Es Python sobre el árbol, sin compilar, y su coste es el de las herramientas del bucle.

**E3b — Las pruebas que viajan.** Ninguna compuerta de texto ve que un AIR cambió: eso fue el
§699. Lo que lo ve es una prueba positiva por cada familia que viaja —las cinco que juzga el kit y
las catorce que juzga la capa—, guardada como vector bajo su versión, con la marca v2 que la
identifica, y juzgada en cada canon por el verificador del nodo y por el del kit: si un AIR cambia
sin que su versión suba (la «Regla de extensión» de la D-G), la prueba vieja deja de verificar y el
canon lo dice. Las cinco del kit ya lo tienen por sus catálogos; las catorce de la capa no, y la D-G
las cubre con tests de la capa (`spec/rfc/0018-el-tren-0-5.md:425-426`). Necesita compilar, y entra
con la E3 de la D-G, en el corte: una prueba de `zkssl/0.4` no dice de qué AIR es. ⚠️ §826: «y por el
del kit» vale para las cinco del kit. Las catorce de la capa las juzga el verificador del nodo: el
kit no verifica pruebas de envío, cobro, quema ni umbral (`spec/rfc/0018-el-tren-0-5.md:110`). Que
un kit las juzgue no está decidido: lo abriría la versión fuerte de `ProofFailed` y
`VerificationFailed` de la E3 del RFC-0019, en el mismo corte (la «Compatibilidad» del RFC-0017).

## Compatibilidad

Ninguna. La E1 es este texto; la E2, texto en `spec/RPC.md` y remisiones; la E3, compuertas que no
cambian nada de lo que viaja. Ningún vector se mueve por este RFC. Su primera aplicación será el
corte `zkssl/0.5` (E4): el RFC-0018 ya está en el árbol, PROPUESTO desde el §675, con sus etapas y
su estado en la cabecera, y eso cumpliría la R5 para lo que ya tiene construido. ⚠️ §799: hasta
aquí decía «sería». ⚠️ §826: no la cumplía entera. La R5 pide también la tabla de etapas con la
superficie que rompe cada una, y ni el RFC-0018 ni el RFC-0019 tenían tabla; la tienen desde el
§826; y desde entonces la del RFC-0017 nombra la superficie de su E2, que decía «NO», y la del
RFC-0020, la de su E4. ⚠️ §865: el RFC-0023, si viaja en él, la tiene desde el §827.

## Decisiones (del autor, §799)

⚠️ §799: hasta aquí esta sección se titulaba «Decisiones abiertas (del autor)» y empezaba «Ninguna
está tomada. La sesión dice lo que recomienda; decide el autor».

El autor aceptó este RFC en el §799, y con él tomó las ocho decisiones: en cada una, la rama que el
texto recomienda; en la 2, donde la sesión no recomendó sobre dos dudas, lo que la D-A ya decía.
Las preguntas y sus ramas se quedan como se escribieron en el §790; detrás de cada una, la tomada y
las descartadas. El RFC sigue PROPUESTO hasta que se construyan sus etapas.

1. **¿Dónde vive la promesa?** En `spec/RPC.md`, como la sección de la D-C (recomendado: es el
   documento normativo que lee primero quien implementa el cable, y la 90 la pone «al lado» de la
   suya), o en un documento propio de `spec/`, con remisiones desde `spec/RPC.md`,
   `spec/PAQUETE.md` y `spec/NUCLEO.md`.
   **Tomada (§799): en `spec/RPC.md`**, la sección de la D-C. Descartado: un documento propio.
2. **¿Qué entra en la superficie?** Las seis de la D-A y lo que dice fuera. Dos dudas que la sesión
   no resuelve: el diario del testigo, que `--auditar` relee y que un auditor custodia; y
   `zk-ssl-verify` como biblioteca, que con la segunda es lo único que verifica hoy una cabeza v1.
   **Tomada (§799): las seis de la D-A**, y «Fuera de la promesa» lo que allí se enumera, el diario
   del testigo y `zk-ssl-verify` como biblioteca incluidos. Descartado: meter en la superficie el
   diario del testigo o la biblioteca.
3. **¿Emitir una versión de cabeza nueva rompe el cable?** La sesión recomienda que sí (R1): cierra
   la práctica de la v3 a la v6 (hecho 3), y es lo que el RFC-0020 propone para su v7.
   **Tomada (§799): sí**, es R1. Descartado: que no rompa.
4. **¿Cuánto aviso?** Un sello con el RFC PROPUESTO antes del corte (R5, recomendado, porque es lo
   que la casa puede comprobar); un plazo en días; o una release del kit que acepte lo nuevo antes de
   que el nodo lo emita, que la R2 permite para las versiones de cabeza y la D-E del RFC-0018 prohíbe
   para el cable.
   **Tomada (§799): un sello con el RFC PROPUESTO antes del corte** (R5). Descartados: el plazo en
   días y la release del kit por delante del nodo.
5. **¿Qué promete una línea `0.N`?** Que dentro de `0.N.x` nada de la superficie se rompe, salvo por
   R6 f, y que entre `0.N` y `0.N+1` todo puede romperse con R5 y R6 (recomendado, mientras el
   protocolo esté por debajo de 1.0); o una promesa más fuerte desde ahora.
   **Tomada (§799): la primera**, mientras el protocolo esté por debajo de 1.0. Descartada: una
   promesa más fuerte desde ahora.
6. **¿Un arreglo de seguridad puede cambiar veredictos dentro de `0.N.x`?** La sesión recomienda que
   sí, con su aviso (R6 f), como la 0.4.2; la otra rama obliga a una línea nueva por cada arreglo.
   **Tomada (§799): sí**, con su aviso (R6 f). Descartada: una línea nueva por cada arreglo.
7. **¿Cuánto tiempo sigue publicado un kit?** Siempre, que es la práctica (recomendado, R6 a); o
   hasta una fecha que la release declare.
   **Tomada (§799): siempre** (R6 a). Descartado: hasta una fecha que la release declare.
8. **¿La E3b entra con el corte o antes?** Con el corte (recomendado): antes, una prueba de
   `zkssl/0.4` no dice de qué AIR es, y la compuerta sólo podría compararla con el commit.
   **Tomada (§799): con el corte.** Descartado: antes.

## Seguridad

- **La clave de gasto no viaja jamás**: este RFC no la toca.
- **Bajar de versión.** La R6 e y la D-E del RFC-0018: ningún kit acepta dos versiones del cable,
  así que una prueba vieja no entra por un kit nuevo.
- **Prometer estabilidad no obliga a aceptar lo roto.** La R6 f deja arreglar un veredicto con aviso,
  y la R6 c, con el RFC-0020, deja de aceptar las firmas de un esquema roto sin dejar huérfanas las
  cabezas que otra firma contiene.
- **Lo que se promete y no se comprueba.** Hasta la E3, la promesa es texto; la R7 y la sección
  dicen lo que la versión no identifica, como hace hoy el §699 con las pruebas.

## Lo que este RFC no decide

Cómo se identifica una prueba (la D-G del RFC-0018, la 118) ni una firma (el RFC-0020): sólo exige
que cada superficie tenga su identificador. Quién acepta un RFC el día que el autor no esté (la 88).
Los parámetros del corte (la D-F del RFC-0018, la 116). Y el juez único (la 117).

## Referencias

- `BACKLOG.md`, entradas 82, 85, 87, 88, 90, 116, 117 y 118.
- `AUDITORIA.md` §209, §236, §275, §278, §281, §290, §418, §641, §662, §663, §664, §680, §682, §684,
  §692, §699, §707, §781 y §787.
- `spec/rfc/PROCESO.md` §«Reglas» y §«Fallos de seguridad»; `spec/RPC.md` §«Notas operativas»;
  `spec/PAQUETE.md` §«6. El contrato del mando»; `spec/NUCLEO.md` §«3. La regla de extensión,
  ejercida»; `doc/KIT.md` §«0. Descargar, y comprobar la descarga antes de creerla»; `SECURITY.md`
  §«5. Cómo reportar una vulnerabilidad».
- La D-E y la D-G del RFC-0018; la E2 del RFC-0019; el RFC-0020.
