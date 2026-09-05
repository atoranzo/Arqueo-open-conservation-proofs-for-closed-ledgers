# RFC-0005 — El núcleo congelado y la regla de extensión

- **Estado:** PROPUESTO
- **Autores:** Che, con Claude (sesiones 96-97)
- **Fecha:** 2026-09-05
- **Versión del protocolo afectada:** `zkssl/0.3` — **no sube** (ver Compatibilidad)
- **Asiento(s) de AUDITORIA:** §236, §243, §290, §395, §397, §398, §404 (el primer sello de H3), y el §405, que lo sella

## Estado de las etapas

| etapa | qué entrega | ¿rompe el cable? | estado |
|---|---|---|---|
| E1 — el núcleo, escrito | un documento normativo bajo `spec/` que enumera QUÉ no cambia y POR QUÉ, con cada pieza atada por cita a la línea del código que la produce; el censo se re-deriva al sellar | NO | por hacer |
| E2 — la regla de extensión, escrita y ejercida | la regla en el mismo documento, y el atado que la ejerce: un verificador acepta un conjunto EXPLÍCITO de versiones y rechaza lo demás (§404 es el primer caso) | NO | sellada — §406 |
| E3 — los rechazos del cable, con vector | vectores negativos de lo que un consumidor del cable tiene que rechazar, bajo su propio directorio, con su puerta en `tools/canon.sh`; qué rechaza hoy el consumidor se MIDE antes de escribir un vector | NO | por hacer |
| E4 — el arnés de conformidad | una herramienta que corre el catálogo de vectores contra CUALQUIER binario que se le pase y dice si pasa y falla igual que la referencia | NO | por hacer |
| E5 — el criterio | una segunda implementación, escrita desde la spec sin leer el código de referencia, que pase y falle igual. No está en la mano del autor; E4 es lo que lo hace comprobable el día que exista | NO | fuera del árbol |

Todas las medidas de este documento se tomaron sobre `9ae055c` y `2080e5a` (§404), en
lecturas puras que no escribieron un byte en el árbol: `PASTE-H3-M`, `PASTE-H3-M2`,
`PASTE-404-M`, `PASTE-404-PRE-r2` y `PASTE-RFC5-M` (ver Referencias).

## Motivación

El hito H3 de la propuesta enviada a NLnet (2026-11-009) dice, literalmente: «*The format as a
minimal standard. Frozen core with rationale; extension rule; specification; positive and
negative conformance vectors; rejection gate. Success criterion: a second independent
implementation, written from the spec without reading the reference code, that passes and
fails the same way. Includes closing the one measured place where rejection is not yet
enforced.*» La última frase se pagó en el §404. Este RFC es el resto.

Lo que el árbol tiene hoy, medido:

- `spec/PAQUETE.md` es el único productor normativo del paquete de evidencia y del contrato del
  mando (§397), y `spec/vectors/paquete/` lleva **67 entradas** —5 positivos, 62 rechazos— con
  su puerta en `tools/canon.sh` (§398, §399). Es el molde entero de «vectores positivos y
  negativos + gate de rechazo», ya en producción.
- `spec/RPC.md` es la especificación del cable, con tres versiones conservadas bajo la suya y
  el triple gate que exige `0.3` idéntico y rechaza `0.2` y `0.1` (§354).
- `spec/rfc/PROCESO.md` fija cinco reglas; la 2 —los vectores viejos jamás se reescriben— es la
  mitad de una regla de extensión.
- Las piezas que un verificador tiene que reproducir existen y **se declaran a sí mismas
  superficie de conformidad** en sus comentarios: `preambulo` (`zk-ssl-verify/src/lib.rs:270-271`,
  «una segunda implementación tiene que producir estos bytes»), `preambulo_cofirma` (`:284-286`),
  y el REGISTRO de dominios de `zk-ssl-hash/src/lib.rs:743-757` que `tools/check_dominios.py`
  gatea en cada canon.

Y lo que falta, también medido:

1. **Nadie dice qué NO cambiará jamás, ni por qué.** Cada pieza del núcleo vive en su fichero
   con su comentario; no hay un documento que las reúna, las justifique y diga cuál es la
   única puerta por la que pueden cambiar. Un segundo implementador tiene hoy que leer el Rust
   para saber qué es contrato y qué es detalle de la implementación de referencia; el hito
   promete que «*a second implementer needs only the specification*» (v3, línea 95).
2. **La regla de extensión existe como práctica, no como texto, y lleva una tensión.** Cuatro
   precedentes en `spec/RPC.md` (`:185`, `:441-442`, `:459-463`, `:586-587`, `:611`) y la
   normativa de `:864-867` dicen que la versión sube cuando cambian los valores que viajan, y
   que un campo aditivo no la sube. Pero 16 de los 22 tipos de `zk-ssl-wire` llevan
   `deny_unknown_fields` y `:459-461` lo llama diseño: «aditivo en el cable, rotura en el
   parser». Un mismo formato no puede decir a la vez «añadir un campo no rompe» y «un campo
   desconocido rompe». Hay que elegir, y escribirlo.
3. **La regla de extensión tenía un hueco medido y sellado esta semana.** `witness.rs:826`
   devolvía conformidad para toda versión fuera de {2, 3} (§404): la mitad «se rechaza lo que
   no se conoce» no estaba escrita en ningún sitio, y por eso pudo faltar en uno.
4. **Los vectores negativos existen sólo para el paquete.** Del cable hay tres vectores
   positivos; el `conformance --check` re-corre el escenario canónico y compara campo a campo,
   y lo único que rechaza es «otra versión» (`conformance.rs:163-168`). Cero vectores de lo que
   un consumidor del cable debe rechazar.
5. **El criterio de éxito no tiene con qué comprobarse.** El README publica como «contrato de
   la segunda implementación» (`README.md:60-62`) correr `conformance --check`: eso exige
   reproducir `proof_digest`, o sea el probador STARK completo, o sea el código de referencia.
   La nota 85 del BACKLOG lo nombra: mientras la conformidad sólo se compruebe contra sí
   misma es autoconformidad. Los 67 vectores del paquete, en cambio, los puede pasar cualquier
   binario sin probador — y hoy sólo los corre `tools/canon.sh` sobre el binario del árbol
   (`:296`).

## Diseño

### D-A — El núcleo es el lado del verificador, no el cable

Lo que se congela es **lo que un tercero necesita para verificar sin el nodo, sin la capa y sin
el probador** (§243): el paquete de evidencia y todo lo que su verificación recompone. El cable
JSON-RPC sigue siendo el contrato de interoperabilidad de `spec/RPC.md`, gobernado por este
mismo proceso, pero **fuera del núcleo congelado**: cambia por RFC, sube de versión cuando
cambian sus valores, y no exige a nadie reproducir un probador.

Razones, todas medidas: la ley del proyecto pide «un sistema que un supervisor pueda verificar
sin ver el libro»; la línea 95 de la propuesta promete un segundo implementador que sólo
necesita la especificación, y el cable no puede cumplirla (punto 5); el paquete ya tiene spec,
vectores y puerta (§397-§399) y su verificador cierra en dos crates y cuatro dependencias
directas, sin red ni reloj (§395); y los casos de uso revisados en `doc/USE_CASES.md` —el
auditor, el supervisor, el titular tras el apagado— están todos del lado del verificador.

**El censo del núcleo se deriva, no se teclea.** E1 lo produce con una lectura pura sobre los
`pub` de `zk-ssl-verify/src/lib.rs` y de `zk-ssl-hash/src/lib.rs` que ese crate usa. Lo medido
hoy, como CANDIDATO y en el orden en que un verificador lo consume:

| pieza | dónde vive hoy | por qué no puede cambiar |
|---|---|---|
| el hex canónico y la codificación del digest (`0x`, `digest_to_bytes`) | `zk-ssl-hash/src/lib.rs:444-452`, `spec/RPC.md` «Codificación» | es la frontera entre el JSON y los bytes: dos implementaciones que difieran aquí no coinciden en nada |
| la permutación de hash y el merge (`native_merge`, `as_digest`, `path_root`) | `zk-ssl-hash/src/lib.rs:100-181` | toda raíz, todo camino y toda cabeza custodiada se recompone con ellas |
| las composiciones de cabeza `epoch_digest` v1/v2/v3 | `zk-ssl-hash/src/lib.rs:200-272` | lo custodiado no caduca (§290): una composición vieja tiene que poder recomponerse siempre |
| los dominios: el REGISTRO de `u64` y de cadenas `ZK-SSL-…` | `zk-ssl-hash/src/lib.rs:743-757`, `zk-ssl-verify/src/lib.rs:113,125` | separan objetos del mismo hash; cambiar uno haría colisionar o dejaría de verificar lo firmado |
| los preámbulos que se firman (`preambulo`, `preambulo_cofirma`) | `zk-ssl-verify/src/lib.rs:276-331` | son los bytes exactos bajo la firma; una segunda implementación «tiene que producir estos bytes» (su propio doc) |
| el byte de versión del preámbulo y el conjunto aceptado {2, 3} | `zk-ssl-verify/src/lib.rs:132`, `main.rs:128`, `witness.rs:828` | es la ÚNICA puerta por la que el núcleo puede crecer (§236) |
| el esquema de firma: `XmssMtSha2_40_8_256` y el índice embebido (`ANCHO_INDICE`) | `zk-ssl-verify/src/lib.rs:109,348-370` | toda cabeza y toda cofirma custodiada está firmada con él |
| el sobre del paquete y su catálogo de rechazos | `spec/PAQUETE.md` secciones 3-5 | ya es normativo (§397) |

Lo que **no** es del núcleo aunque viva al lado, y el documento de E1 lo dirá con la misma
claridad: `aplicar_apano_del_oid` y `OFFSET_MT_UPSTREAM` (`lib.rs:399-415`) corrigen un fallo de
`xmss 0.1.0-pre.0` (§240) y son de la implementación de referencia, no del formato — una
segunda implementación con una biblioteca XMSS correcta no debe replicarlos; el campo `canon`
del vector del cable (`conformance.rs:121`) es una foto de cifras de tests, no un valor del
protocolo; y todo lo que el verificador no recompone.

### D-B — La regla de extensión, en dos mitades

**Primera mitad — lo que se firma crece sólo por versión.** Toda composición que entra bajo una
firma lleva byte de versión; una composición nueva es una versión nueva del preámbulo, con sus
vectores bajo su versión, y los vectores de las anteriores jamás se reescriben (regla 2 del
PROCESO). Un verificador del núcleo **acepta un conjunto explícito de versiones y rechaza todo
lo demás con texto**: v1, v4, 0x103 son rechazos, no conformidad. El §404 es el primer sitio
donde esta mitad se hizo cumplir; E2 la escribe y le da al conjunto aceptado **un solo
productor** —hoy son dos, el mando (`verify/main.rs:128`) y el testigo (`witness.rs:828`), atados
por copia y no por test— y lo ata con un test que lo enumere allí; los rechazos lo consumen, no lo
repiten. Un conjunto con dos productores es la clase de defecto que la Motivación censa.

**Segunda mitad — lo que no se firma no existe para el núcleo.** Un sobre puede ganar claves que
el núcleo no conoce sin subir su versión, porque el verificador **no cree nada que no
recomponga**: una clave que no entra en ninguna composición ni en ningún preámbulo no puede
alterar lo que se comprueba. El verificador del núcleo las ignora — y la referencia ya lo hace:
el mando de `zk-ssl-verify` no lleva un solo atributo `serde` y lee sus 14 claves con `.get()`
(medido en la sesión 92 sobre `main.rs`); una clave de más no cambia su veredicto. Esta mitad no
la introduce el RFC: es lo que el binario publicado en `arqueo-verify-v0.1.0` hace, declarado.
Que la implementación de referencia rechace además claves desconocidas en su consumidor del cable
(`deny_unknown_fields` en 16 tipos de `zk-ssl-wire`) es **una elección de esa implementación,
declarada como tal** en `spec/RPC.md`, no una regla del formato.

La tensión se declara, no se esconde: la ley del proyecto también dice «ante duda, se para».
Aquí las dos mitades reparten la duda: **sobre la versión, fail-closed**; **sobre las claves no
firmadas, indiferencia**, porque no hay duda que resolver — nada de lo verificado depende de
ellas. Si el autor prefiere fail-closed también en las claves, la consecuencia es que todo
campo aditivo sube versión, `spec/RPC.md:864` deja de ser cierto y el binario publicado cambia
de conducta: es una decisión reversible y se escribe como tal.

**Familias nuevas.** Un objeto nuevo (otra prueba portable, otro sobre) entra con su propio
`tipo` o su propio dominio, sus propios vectores bajo su propio directorio y su propio RFC —
como hizo el paquete de extensión (`tipo: "extension"`) y como hicieron los vectores del paquete
respecto de los del cable (descarte 5 del RFC-0004). El núcleo no se toca para acoger una
familia — salvo que la familia necesite entrar **bajo la firma de la cabeza**: entonces no entra
por aquí sino por la primera mitad, como versión nueva del preámbulo con sus vectores bajo su
versión, y su coste (una composición `epoch_digest` nueva, el cable aditivo, el verificador lector
hasta esa versión, los vectores) se mide antes de escribirla. Es el camino previsto para la
unicidad entre libros de D-C.

### D-C — El núcleo no identifica nada fuera del libro

Ninguna pieza congelada nombra, deriva ni transporta un identificador de fuera del libro: ni
una factura, ni un contador, ni una etiqueta compartida entre operadores. Está medido que la
capa no persiste nulificador alguno, que los 27 tipos de entradas públicas de los circuitos no
toman identificadores externos y que el nulificador que existe se deriva de la clave privada
de gasto precisamente para que nadie precompute los ajenos (§395, `PASTE-H2-M`). La unicidad
**entre** libros —el caso de la doble certificación entre organismos que `doc/USE_CASES.md:65-69`
ya declara como gobernanza y no criptografía— exigiría material de clave compartido que nadie
tenga entero, es decir una suposición de confianza nueva: es una **familia nueva** en el sentido
de D-B, con su RFC, y el núcleo se congela de forma que pueda existir sin romperlo. Este RFC no
la construye: hoy no se puede escribir el testigo que la falsaría.

### E3 — Los rechazos del cable, con vector

El consumidor del cable de la referencia es el testigo (`zk-ssl-cli`), y publica sus clases de
rechazo como texto estable (`witness.rs`, `clase()`: `no-verifica`, `version-desconocida`,
`indice-discordante`, `campo-torcido`…, medidas en el `PASTE-404-PRE-r2`). E3 mide primero qué
respuestas del cable rechaza hoy y con qué clase, y sólo entonces escribe un vector por rechazo,
bajo un directorio propio de `spec/vectors/` —jamás entre los `zkssl-0.N.json`— y su bloque en
`tools/canon.sh` calcado del «3 bis» del paquete. La primera candidata es la que el §404 acaba
de cerrar: una cabeza servida con `formatVersion` fuera de {2, 3}.

### E4 — El arnés de conformidad para cualquier binario

`tools/canon.sh:291-296` ya corre el binario del árbol contra cada entrada del MANIFIESTO del
paquete. E4 saca ese bucle a una herramienta que recibe **la ruta de un binario cualquiera** y
el MANIFIESTO, y dice por entrada si el rc y el texto coinciden con lo esperado. Es lo que
convierte E5 en comprobable: el día que exista una segunda implementación, el resultado
publicable es la salida de ese arnés sobre su binario, no una afirmación. Hasta entonces, el
arnés sobre el binario de referencia es un resultado publicable por sí mismo: el catálogo
completo, corrido fuera del canon, por cualquiera.

## Lo que se DESCARTÓ al medir

1. **Congelar el cable.** Descartado: reproducirlo exige el probador (`proof_digest`), luego
   exige leer el código de referencia; contradice la línea 95 de la propuesta y la nota 85.
2. **Rechazar claves desconocidas como regla del formato.** Descartado (reversible): haría de
   todo campo aditivo una rotura y dejaría falsa la normativa de `spec/RPC.md:864` y sus cuatro
   precedentes. Queda como elección declarada de la implementación de referencia.
3. **Meter el núcleo en `spec/RPC.md`.** Descartado: `spec/RPC.md` especifica lo que cruza el cable,
   y el núcleo es lo que se verifica sin cable (descarte 2 del RFC-0004, misma razón).
4. **Incluir en el núcleo el apaño del OID de `xmss`.** Descartado: es un parche sobre un fallo
   de una biblioteca en pre-release (§240); congelarlo obligaría a toda segunda implementación
   a reproducir un defecto ajeno.
5. **Tomar el campo `canon` del vector del cable como parte del protocolo.** Descartado: es una
   foto de cifras de tests (§284); una segunda implementación sólo podría copiarla. Queda
   fichado para el frente del cable.
6. **Construir la identidad entre libros dentro de este RFC.** Descartado por la regla corta de
   la ley: no se puede escribir hoy el testigo que la falsaría (no hay dos libros ni etiqueta
   compartida). Es familia nueva, con RFC propio.
7. **Fijar aquí el nombre del documento de E1.** Descartado por la misma medida que en el
   RFC-0004 (descarte 7): `tools/verificar_citas.py` exige que todo `.md` citado exista, y un RFC
   no puede citar lo que aún no existe. El nombre se fija en E1, en el commit que lo crea.
8. **Llamar «estándar» a nada de esto.** Descartado hasta que E5 se cumpla: el proyecto decidió
   en la sesión 83 que esa palabra no se escribe hasta que exista la segunda implementación que
   pase y falle igual. Este RFC dice «núcleo» y «contrato».

## Compatibilidad

| etapa | ¿rompe el cable? | vectores |
|---|---|---|
| E1 — el núcleo, escrito | **NO.** Documenta lo que ya es; ningún valor que viaja cambia | los tres del cable, intactos; los 67 del paquete, intactos |
| E2 — la regla, escrita y ejercida | **NO.** El conjunto aceptado {2, 3} ya es el que el árbol aplica desde §404 | intactos |
| E3 — los rechazos del cable | **NO.** Vectores nuevos bajo un directorio propio | los del cable y los del paquete, intactos; nace un catálogo aparte |
| E4 — el arnés | **NO.** Herramienta, no formato | intactos |

**Regla 2 del PROCESO: los vectores viejos jamás se reescriben.** `zkssl-0.1.json`,
`zkssl-0.2.json`, `zkssl-0.3.json` y los 66 ficheros de `spec/vectors/paquete/` se conservan
sin una modificación.

### Por qué entra por RFC si no rompe el cable

Por la letra, el ámbito del proceso (`spec/rfc/PROCESO.md:3-6`) enumera `spec/RPC.md`,
`spec/openrpc.json` y los vectores de `spec/vectors/`: E1 toca `spec/` (un documento nuevo y la
declaración en `spec/RPC.md` de la elección de la referencia sobre las claves desconocidas) y E3
añade bajo `spec/vectors/`. Por el espíritu, es la primera vez que el proyecto escribe qué no
cambiará: eso, si no entra por RFC, no entra por nada. `spec/openrpc.json` no se toca ni se
regenera: no hay método nuevo; el giro a ACEPTADO lo declarará como hizo el RFC-0004.

## Seguridad

**Efecto sobre el principio del API —la clave de gasto no viaja jamás: NINGUNO.** Este RFC no
añade ni cambia ningún método, ningún tipo del cable y ningún material que viaje.

**Efecto sobre las deudas declaradas:**

- **Aviso fuera de banda (§21): ninguno.** Nada del núcleo transporta avisos.
- **Nodo único: ninguno.** Este RFC no introduce consenso en ninguna etapa, deliberadamente
  (`SECURITY.md` §6). Congelar lo que un tercero verifica sin el nodo es lo contrario de
  depender de él.
- **`--dev`: ninguno.** El verificador no tiene features ni custodios de prueba.

**Congelar no publica nada nuevo.** Cada pieza del núcleo ya es pública, ya está en el árbol y
ya la ejercita un gate. Lo que este RFC añade es la promesa escrita de que no cambiará, y la
única puerta por la que puede crecer. **El operador sigue siendo otro adversario**: el núcleo
le impide falsificar lo custodiado; no le impide omitir ni ordenar (`doc/USE_CASES.md`, «What
none of this claims»).

## Referencias

Medido sobre `9ae055c` y, tras el §404, sobre `2080e5a`; huellas en `sha256 | cut -c1-16`,
líneas en `wc -l`.

- `NLNET-form-answers-EN-v3.txt` `26dcde32091e857d` / 160 — el texto de H3 (línea 42) y la
  promesa del segundo implementador (línea 95). Fuera del árbol, en el expediente.
- `spec/rfc/PROCESO.md` `0eba90e3fe5d93f8` / 24 · `spec/rfc/0000-plantilla.md` `488e7055cbfcad09` / 29 ·
  `spec/rfc/0004-paquete-de-evidencia.md` `53ea2afb28ba92d8` / 270 — el molde.
- `spec/PAQUETE.md` `1278699dc460aa44` / 283 — secciones 3-5 (`ea39d85164e9b6c9`,
  `8b603a4ef60d139e`, `6dbda8c65a1b2bf7`) y la 8 («lo que este documento NO afirma»).
- `spec/vectors/paquete/MANIFIESTO.txt` `9dafd1f061b52dc8` / 71 — 5 positivos, 62 rechazos.
- `spec/RPC.md` `cb40263b3b91237c` / 867 — la regla del parser 455..466 (`88a03af26f9e5016`) y
  la normativa de la versión 856..867 (`cfd8fc6b0e642463`).
- `crates/zk-ssl-verify/src/lib.rs` `4327e89dd517dc58` / 1068 — `preambulo` 270..282,
  `preambulo_cofirma` 284..331, `Conjunto` :109, `DOMINIO` :113, `DOMINIO_COFIRMA` :125,
  `VERSION_FORMATO` :132, `ANCHO_INDICE` :370, el apaño del OID 399..415, `verificar_cabeza` 494..530.
- `crates/zk-ssl-hash/src/lib.rs` `2f05cfd9fcf7e690` / 896 — `native_merge` :100, `as_digest`
  :130, `path_root` :181, `epoch_digest` :200 / v2 :224 / v3 :257, `acuse_digest` :333,
  `mmr_hoja` :374, `mmr_nodo` :380, `digest_to_bytes` :444, el REGISTRO 743..757.
- `crates/zk-ssl-wire/src/lib.rs` `9669d5d02050ad70` / 1354 — 22 tipos, 16 con `deny_unknown_fields`.
- `crates/zk-ssl-cli/src/witness.rs` `7d1da2575eb0dc06` / 4647 — `recomponer` con la guarda del
  §404; las clases de `clase()`.
- `crates/zk-ssl-cli/src/conformance.rs` `c308b7576c074781` / 209 — qué compara `--check` (:163-205).
- `doc/USE_CASES.md` `22fcbb837ae7df72` / 137 · `ROADMAP-ECOSISTEMA.md` `6008684d123173c3` / 56 ·
  `README.md` `a6d412397da863a6` / 569 (:60-62, :74-89) · `BACKLOG.md` `c2db6c5e38a442a2` / 3235
  (nota 85, :642-658).
- Lecturas puras de la sesión 96: `PASTE-H3-M` `0fd433f8c96ff378` (salida `5d0caab078ab9dd2` / 383) ·
  `PASTE-H3-M2` `b510877df2c85375` (`f5f6c9aca96bd222` / 797) · `PASTE-404-M` `c8e5c40a20598301`
  (`da0aa8ae2b2d24d0` / 199) · `PASTE-404-PRE-r2` `6c4385ff7069f4d3` (`db513d279f11f8e5` / 208) ·
  `PASTE-RFC5-M` `7967b37e9e29b8f2` (`f4abd1572c2c8567` / 1192).

**Numeración.** El **RFC-0001 queda reservado** al endurecimiento del KDF del keystore, sin
redactar (`spec/rfc/0002-lotes-y-transicion-de-hoja.md:39`). Este RFC toma el **0005** por
correlativo, tras el 0004.

**Procedencia de una cita arrastrada.** La medida del uso único (ningún nulificador persistido,
27 tipos de entradas públicas sin identificador externo, derivación desde la clave privada) se
tomó en la sesión 91 sobre `bb5322f` (`PASTE-H2-M`); se cita porque el §395 la selló y ninguno
de los sellos posteriores tocó esos productores.

**Decisiones delegadas (r2, sesión 97).** El autor delegó en el asistente las decisiones de
alcance D-A, D-B y D-C y las tres precisiones de la r2 (el conjunto aceptado con un solo
productor; la segunda mitad de D-B como hecho medido del binario; la entrada de una familia
bajo la firma por la primera mitad), con la instrucción de aplicar los principios y el
manifiesto del proyecto. Todas son REVERSIBLES y el §405, que sella este RFC, las recoge.
