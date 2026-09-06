# RFC-0006 — El consumo publicado: unicidad de uso dentro de un libro, detección entre libros

- **Estado:** PROPUESTO
- **Autores:** Che, con Claude (sesión 99)
- **Fecha:** 2026-09-06
- **Versión del protocolo afectada:** `zkssl/0.3` — **sube a `zkssl/0.4` en E2** (ver Compatibilidad)
- **Asiento(s) de AUDITORIA:** §32, §36, §55, §117, §236, §275, §292, §387, §391, §392, §405, y el §412, que lo sella

## Estado de las etapas

| etapa | qué entrega | ¿rompe el cable? | estado |
|---|---|---|---|
| E1 — el árbol y la raíz en reposo | el conjunto de consumos como árbol disperso de la capa, su raíz `root:cons` guardada y comprobada al abrir (la sexta raíz en reposo), el rechazo de un consumo repetido, la instantánea que lo transporta, y los testigos negativos que lo falsan | NO | propuesta |
| E2 — la cabeza v4 | la composición `epoch_digest_v4` que mete la raíz y la cuenta de consumos bajo la firma; el conjunto aceptado pasa a {2, 3, 4}; el cable sirve los dos campos; vectores nuevos bajo su versión | **SÍ** (`zkssl/0.3` → `0.4`) | propuesta |
| E3 — la prueba portable | una forma nueva del paquete de evidencia que prueba que un consumo está bajo la raíz de una cabeza firmada y no lo estaba bajo la de una cabeza anterior; el mando la verifica sin nodo | NO | propuesta |
| E4 — el banco de dos libros | dos nodos, dos operadores, el mismo consumo publicado en los dos, y un lector que con las dos cabezas firmadas lo detecta; y el negativo: con una sola cabeza no hay nada que detectar | NO | propuesta |
| E5 — el atado en circuito | que el consumo quede restringido en el AIR a la operación que lo consume, con `nullifier_tree.rs` como pieza de partida. Fuera de este RFC: hoy no se puede escribir el testigo que lo falsaría sin E1 | NO | fuera del alcance |

Todas las medidas de este documento se tomaron sobre `2298e61` (§411), en dos lecturas puras que no
escribieron un byte en el árbol: `PASTE-NULL-M` y `PASTE-412-PRE` (ver Referencias).

## Motivación

`doc/USE_CASES.md:28` publica, en la tabla de propiedades, la fila «No double use — a unit is
consumed once (nullifiers) — measured». Lo medido es otra cosa: que la vía de producción en dos
fases no usa nulificador alguno (`SECURITY.md:376`) y que la capa no persiste ninguno (RFC-0005,
D-C). La fila es verdadera para el doble gasto de una cuenta —lo cierra el orden total del
registro— y falsa en su paréntesis. El caso de uso que la misma página describe en `:62-69` —la
doble certificación de un mismo gasto bajo dos programas— no lo cubre hoy ninguna pieza del
árbol: un tercero no puede saber, leyendo dos cabezas firmadas, si la misma unidad se consumió en
las dos.

Lo que el árbol tiene hoy, medido (`PASTE-NULL-M`):

- **Dos nulificadores vivos y uno retirado, ninguno de los cuales sirve.** El de gasto de la vía
  retirada (§32): `derive_nullifier(sk, nonce) = H(H(DOMINIO, sk), nonce)` en
  `crates/zk-core/src/spend_authority.rs:86`, portado a STARK en
  `crates/stark-experiment/src/nullifier_tree.rs` (793 líneas: no-pertenencia e inserción en
  lockstep sobre `dual_climb`, con la colisión de posición declarada como denegación de servicio
  en su cabecera); lo consumen `settlement-layer` y `circuit_settlement`, y **ningún fichero de
  `crates/zk-ssl/src`** salvo la limpieza del legado. El de umbral de custodios
  (`circuit_threshold_single_nullifier.rs:243`, `{identity_domain, custodian_set_root, nullifier,
  operation}`): uso único de una autorización, y ése sí lo usa la capa. Los dos se derivan de una
  clave privada **para que nadie pueda precomputarlos** (§117): son lo contrario de una etiqueta
  que dos organismos tienen que poder calcular por separado.
- **`root:nullifier` y el prefijo `null:` están ocupados por la migración.** `load`
  (`crates/zk-ssl/src/persistence.rs:456-680`) los lee, los verifica contra la raíz declarada y
  **los borra** (§36); la instantánea v3 los transporta y la v4 los verifica al importar. Una
  raíz nueva con ese nombre no sobreviviría a la primera apertura.
- **Cinco raíces en reposo, todas fail-closed.** `root:state`, `root:pending` (§387), `root:pmeta`
  (§388), `root:froz` (§391) y `root:log` (§392), leídas con `need()` en `load`
  (`persistence.rs:484-562`) y escritas en `commit` (`:774-783`). El molde de la sexta está
  escrito cinco veces.
- **El coste de un campo firmado está tarifado y sin pagar.** `epoch_digest_v3 = merge(v2,
  merge(cima, as_digest(t)))` (`crates/zk-ssl-hash/src/lib.rs:257`), sin dominio, separado sólo
  por el byte de versión del preámbulo (§236, `zk-ssl-verify/src/lib.rs:329`). El conjunto
  aceptado {2, 3} tiene un solo productor desde E2 del RFC-0005 (`VersionCabeza`,
  `lib.rs:142`, `TODAS` `:151`) y el test que lo enumera pina `[2, 3]` y el texto «v2 o v3»: **el
  falsador de E2 ya existe**. El RFC-0005 (D-B, «Familias nuevas») dice por dónde entra una
  familia bajo la firma y qué hay que medir antes: este documento es esa medida.
- **La palabra está gastada.** `nullifier` aparece 200 veces en 67 documentos, `nullificador` 39
  y `nulificador` 43; con tres grafías y tres significados (gasto, umbral, legado). El punto 25
  de la cola de la sesión 80 lo dejó abierto y esta lectura lo acota: el patrón «tiene / hay un
  árbol de nulificadores» da cero fuera de `AUDITORIA.md`, y las filas en presente que quedan de
  cara al público —`INSTITUCIONAL.md:271`, `INSTITUTIONAL.md:279`, `VISION.md:127`,
  `doc/USE_CASES.md:28`— se citan aquí y se corrigen cuando un sello toque cada fichero.

Y lo que falta, también medido: no existe en el árbol un objeto que dos operadores puedan calcular
por separado a partir de un dato acordado, ni una raíz que lo acumule, ni una cabeza que la firme,
ni un lector que compare dos cabezas. Las cuatro piezas son este RFC.

## Diseño

### D-1 — El nombre: consumo publicado

El objeto se llama **consumo**; su prefijo en reposo es `cons:` y su raíz `root:cons`. Adversario:
quien certifica la misma unidad dos veces. Prueba: el consumo está bajo la raíz de la cabeza
firmada de la época t y no lo estaba bajo la de una cabeza anterior. Un consumo es
`H(DOMINIO_CONSUMO, identificador)` sobre un identificador **acordado y público**; quien lo conoce
lo calcula. Eso es deliberado y es lo contrario del nulificador de gasto: aquí la
precomputabilidad es la propiedad, no el defecto.

Descartados, con su medida: «nulificador» (dos vivos y uno muerto, tres grafías; ley 2, un nombre
significa una sola cosa), «marca» (la marca de acreditación de la nota 99 y la marca de fin de
fichero), «uso» (`meta:cust_uses`, la cuota de custodios, §394). El juez del nombre
(`PASTE-412-PRE`): `cons:`, `root:cons` y `consumo` dan **cero** en los 158 `.rs`; en los 67 `.md`
la palabra aparece cinco veces, cuatro por «CPU de consumo» y una en `VISION.md:574` («el consumo
va al aplicar», el consumo de un pendiente). Esa vecindad se declara: el consumo de este RFC es un
objeto publicado, no la fase de un pago.

### D-2 — Dónde vive: el mínimo, y el circuito como extensión declarada

Un árbol disperso de la capa, calcado del de congelados (§391): hoja = consumo, posición = el
digest completo y no sus bits bajos (la colisión de posición de `nullifier_tree.rs` no se hereda,
como pide la nota 3045 del `BACKLOG.md`). Su raíz `root:cons` se guarda en `commit` y se
comprueba en `load` con `need()`: un libro sin `root:cons` **no abre** (fail-closed, como las
cinco anteriores; migrar es operación aparte, §392). La instantánea sube de la versión 4 a la 5
y transporta el conjunto; una v4 sigue importándose con el conjunto vacío, declarado. Publicar un
consumo es una operación de la capa que entra por el registro de transiciones como entrada
propia, y **un consumo ya presente se rechaza** con nombre. Tres testigos negativos, enseñados
rojos antes del arreglo: el repetido se rechaza; sin `root:cons` no abre; con `root:cons` falsa no
abre.

Lo que el mínimo **prueba** es exactamente eso y nada más: que un consumo está bajo la raíz y no
estaba antes. **No prueba que el consumo corresponda a un pago** ni que el pago sea el que dice: lo
que el circuito no restringe no existe, y aquí el circuito no restringe nada. El atado del consumo
a la operación en el AIR es E5, con `nullifier_tree.rs` (no-pertenencia e inserción en lockstep,
ya probada en su crate) como pieza de partida, y queda fuera de este RFC por la regla corta: sin
E1 no hay libro con consumos, luego no se puede escribir hoy el testigo que falsaría el atado.

### D-3 — La cabeza: v4 lleva una sola familia

`epoch_digest_v4 = merge(epoch_digest_v3(los nueve), merge(root_cons, as_digest(k)))`, con `k` la
cuenta de consumos bajo la raíz — el molde de §275 y §292, otra vez, sin dominio, con el byte de
versión del preámbulo pasando de 3 a 4. Génesis, declarado: la primera cabeza compone con la raíz
del árbol vacío y `k = 0`. `VersionCabeza` gana `V4` y el conjunto aceptado pasa a {2, 3, 4}: el
test que hoy exige `[2, 3]` se pone rojo y se corrige con el sello, no antes. El cable sirve los
dos campos nuevos junto a los nueve de la cabeza; la firma los cubre.

El punto 43 de la cola —la cabeza que firme `supply` y `meta_root`— **no monta en esta v4**: no
tiene RFC ni falsador escrito, y una versión que meta dos familias sin medir la segunda es la clase
de era silenciosa que la ley prohíbe. Se declara: v4 queda tomada por este RFC; el 43, cuando
escriba su testigo, es v5. Reversible en el §412.

### D-4 — El límite, escrito

- **El identificador acordado es gobernanza, no criptografía.** Ya lo dice `doc/USE_CASES.md:66-69`;
  este RFC no lo cambia. Dos organismos que no acuerdan el identificador no detectan nada.
- **La prueba no dice que la unidad sea real** (la factura, el gasto elegible): es el límite del
  oráculo de `SECURITY.md`, y el consumo lo hereda entero.
- **Dentro de un libro es invariante; entre libros es detección, no prevención.** Un solo nodo
  ordena; nadie impide que dos libros acepten el mismo consumo. Lo que este RFC da a un tercero
  es que, con las dos cabezas firmadas en la mano, lo vea. E4 falsa la detección; no promete más.
- **Quien publica primero bloquea.** Un consumo es precomputable por diseño, luego un tercero con
  acceso al nodo puede publicarlo antes que el titular legítimo. Es denegación de servicio, no
  doble uso, y se declara con el mismo peso con que `nullifier_tree.rs` declara su colisión de
  posición. Mitigarlo (atar el consumo a una autorización) es E5 o un RFC posterior.
- **Inobservabilidad: ninguna.** Quien conoce el identificador ve cuándo se consumió. Es la
  propiedad pedida por el caso de uso, y la ley prohíbe vender privacidad absoluta: aquí no se
  vende ninguna.
- **El operador sigue siendo otro adversario.** Puede omitir un consumo o retrasarlo; no puede
  falsificar la raíz que firmó ni retirar uno ya publicado sin que la cabeza deje de extender a la
  anterior (§292).

## Lo que se DESCARTÓ al medir

1. **Llamarlo nulificador.** Descartado: tres significados vivos o muertos en el mismo árbol (D-1).
2. **Reutilizar `root:nullifier` y `null:`.** Descartado: `load` los borra (§36, `persistence.rs:656-668`).
3. **Empezar por el circuito.** Descartado (reversible): el testigo negativo del mínimo —repetido,
   sin raíz, raíz falsa— se escribe hoy; el del atado en AIR exige E1. Testigo antes que feature.
4. **Meter `supply` y `meta_root` (punto 43) en la misma v4.** Descartado (reversible): sin RFC ni
   falsador propio. El 43 sería v5.
5. **La etiqueta con material de clave compartido (OPRF), como apuntaba D-C del RFC-0005.**
   Descartado por ahora (reversible): exige una suposición de confianza nueva y un material que nadie
   tenga entero; el mínimo publica la etiqueta en claro y declara lo que eso cuesta (D-4). Si el
   caso de uso exige que un tercero no vea cuándo se consumió, es familia nueva con su RFC.
6. **Llamar «prevención» a la detección entre libros.** Descartado: un solo nodo ordena; sin un
   lector de dos cabezas no hay nada, y con él hay detección (D-4).
7. **Fijar aquí el nombre del método del cable, del fichero de vectores o de la forma del paquete.**
   Descartado por la misma medida que en los RFC 0004 y 0005 (descarte 7): `tools/verificar_citas.py`
   exige que todo documento citado exista, y este RFC no puede citar lo que aún no existe. Se fijan
   en E1, E2 y E3, en el commit que los crea.
8. **Derivar la posición de los bits bajos del consumo.** Descartado: es la colisión declarada de
   `nullifier_tree.rs`; el árbol de la capa indexa por el digest completo, como `froz:`.

## Compatibilidad

| etapa | ¿rompe el cable? | vectores |
|---|---|---|
| E1 — el árbol y la raíz | **NO.** Ningún método ni tipo del cable cambia; la entrada del registro es aditiva | los tres del cable, los del paquete, los del cable negativo y los KAT, intactos |
| E2 — la cabeza v4 | **SÍ.** `formatVersion` pasa a 4 y viajan dos campos firmados nuevos: `zkssl/0.3` → `zkssl/0.4` | `zkssl-0.1.json`, `0.2` y `0.3` intactos; nace `zkssl-0.4.json`; los rechazos del cable de `0.3` quedan bajo su versión (un consumidor `0.3` sigue rechazando `formatVersion: 4`, y eso es correcto); nace el KAT de `epoch_digest_v4` y del preámbulo v4 |
| E3 — la prueba portable | **NO.** Forma nueva del paquete, con su `tipo` y sus vectores bajo su directorio | los 67 del paquete, intactos; nace un catálogo aparte |
| E4 — el banco | **NO.** Herramienta, no formato | intactos |

**Regla 2 del PROCESO: los vectores viejos jamás se reescriben.** `zkssl-0.1.json`,
`zkssl-0.2.json`, `zkssl-0.3.json`, los 66 ficheros de `spec/vectors/paquete/`, los 11 de
`spec/vectors/cable/` y los 18 de `spec/vectors/nucleo/` se conservan sin una modificación.

### Por qué entra por RFC

Por la letra: E2 cambia valores que viajan (`spec/RPC.md`, `spec/openrpc.json`, los vectores) y
sube la versión del cable; E3 añade bajo `spec/vectors/`. Por el espíritu: es la primera familia
que entra **bajo la firma de la cabeza** desde que el RFC-0005 escribió por dónde entra una; si
no entra por RFC, la regla de extensión se estrena incumplida.

## Seguridad

**Efecto sobre el principio del API —la clave de gasto no viaja jamás: NINGUNO.** El consumo se
deriva de un identificador público; no interviene ninguna clave, ni de gasto, ni de vista, ni de
custodio. Publicarlo no exige demostrar nada al nodo, y eso es una limitación declarada (D-4,
«quien publica primero bloquea»), no una erosión del principio.

**Efecto sobre las deudas declaradas:**

- **Aviso fuera de banda (§21): ninguno.** El consumo no transporta avisos.
- **Nodo único: ninguno.** E4 corre dos nodos porque son dos libros de dos operadores, no dos
  réplicas de uno; este RFC no introduce consenso en ninguna etapa (`SECURITY.md`, «un solo
  nodo»), y la detección entre libros funciona precisamente porque no lo necesita.
- **`--dev`: ninguno.** No hay custodios de prueba en el camino del consumo.

**Lo que este RFC publica de nuevo:** una raíz más bajo la firma y un conjunto de etiquetas
públicas y precomputables. Lo que **no** afirma: que un consumo corresponda a un pago (E5), que el
identificador sea correcto (gobernanza), que la unidad exista (oráculo), que dos libros no puedan
aceptar el mismo consumo (detección, no prevención), ni que nadie vea cuándo se consumió.

## Referencias

Medido sobre `2298e61`; huellas en `sha256 | cut -c1-16`, líneas en `wc -l`.

- `spec/rfc/PROCESO.md` `0eba90e3fe5d93f8` / 24 · `spec/rfc/0000-plantilla.md` `488e7055cbfcad09` / 29 ·
  `spec/rfc/0005-nucleo-congelado.md` `3489769d5af99460` / 315 — el molde; D-B «Familias nuevas»
  (`:146-154`), D-C (`:156-167`), descarte 6 (`:219-221`).
- `crates/zk-ssl/src/persistence.rs` `3264d97198d68236` / 878 — el legado `null:`/`root:nullifier`
  (`:456-680`), las cinco `need("root:…")` de `load` (`:484-562`), las cinco escrituras de `commit`
  (`:774-783`).
- `crates/zk-ssl-hash/src/lib.rs` `2f05cfd9fcf7e690` / 896 — `epoch_digest` `:200` / v2 `:224` / v3 `:257`.
- `crates/zk-ssl-verify/src/lib.rs` `b4a2cefe564aae03` / 1154 — `VERSION_FORMATO` `:132`,
  `VersionCabeza` `:142`, `TODAS` `:151`, `texto` `:157`, `preambulo` `:329`.
- `crates/zk-ssl-wire/src/lib.rs` `9669d5d02050ad70` / 1354 — `SignedEpochHeadDto` `:695`.
- `crates/zk-ssl-cli/src/witness.rs` `819df9e2d790dde8` / 4738 — `recomponer` `:843`, el `match` de
  versiones `:871`. `crates/zk-ssl-verify/src/main.rs` `6dc17e15b8b2c97e` / 570 — `:129`, `:148`.
- `crates/zk-core/src/spend_authority.rs` `2afc3874aea442aa` / 239 — `derive_nullifier` `:86`.
  `crates/stark-experiment/src/nullifier_tree.rs` `7dd2d2b4d0b2578c` / 793 — cabecera `:1-43`,
  `NullifierPublicInputs` `:221`. `crates/stark-experiment/src/circuit_threshold_single_nullifier.rs`
  `02bd2cdc63108d89` / 1119 — `:243`.
- `spec/RPC.md` `272ff83140600046` / 889 — `zkssl_epochHead` `:69`, `zkssl_signedEpochHead`
  `:433-478`, la confianza residual `:873`.
- `doc/USE_CASES.md` `22fcbb837ae7df72` / 137 — la fila `:28`, el caso `:62-69`.
  `SECURITY.md` `e98ebd8462b0a3c5` / 651 — `:376`. `BACKLOG.md` `c2db6c5e38a442a2` / 3235 — `:3045`.
- Lecturas puras de la sesión 99: `PASTE-NULL-M` `de16e15a4efa9bed` (salida `af3599b28a30b0d5` / 2228) ·
  `PASTE-412-PRE` `fb58926ddaf09346` (`9fe4664b793ad743` / 835).

**Numeración.** El **RFC-0001 sigue reservado** al endurecimiento del KDF del keystore. Este RFC toma
el **0006** por correlativo, tras el 0005 (derivado en `PASTE-412-PRE`: ocupados 0, 2, 3, 4, 5).

**Decisiones delegadas (sesión 99).** El autor delegó en el asistente las cuatro decisiones D-1 a
D-4 con la instrucción de aplicar la constitución de decisión del proyecto. Todas son REVERSIBLES y
el §412, que sella este RFC, las recoge una a una.
