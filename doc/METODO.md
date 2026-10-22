# Cómo se trabaja aquí

Lo que un cambio tiene que llevar para entrar en Arqueo, contado fuera de los traspasos de
sesión: el sello y su asiento, cómo se corrige un documento vivo, las convenciones del
`BACKLOG.md`, los RFC y sus etapas, los niveles del canon y los bancos, y el paso 4 de `GENAI.md`.
Es la tercera pieza de la entrada 88 del `BACKLOG.md`, que pedía «dejar escrito el "cómo se trabaja
aquí" fuera de los traspasos de sesión, que hoy son el único sitio donde vive el método».

**Este documento no establece nada.** Cuenta lo que establecen otros ficheros y cita, en cada
afirmación, el que lo hace. Donde algo se hace siempre igual y ningún fichero lo manda, lo dice con
la palabra *costumbre*. Si este documento y el fichero que cita dejan de decir lo mismo, manda el
fichero, y este se corrige con su marca (sección 3).

**Lo que NO decide.** Quién mantiene el proyecto y con qué regla se acepta un RFC. La entrada 88 lo
pone primero entre lo que la cierra —«declarar el mantenimiento y el criterio de aceptación de un
RFC —el proceso ya existe en `spec/rfc/`, le falta quién decide y con qué regla—»— y es del autor.
Lo que hoy está escrito sobre quién acepta, y que no es esa regla, va en las secciones 5 y 7.

---

## 1. El método: cuatro pasos, y fuera del paso 4

`GENAI.md` §«Cómo se usa, exactamente» da el método en cuatro pasos: se **mide** primero, sobre el
árbol, y un resumen recordado no es fuente; se discute sobre esos números y **el autor decide**; el
asistente propone un **bloque**, un guion que aplica el cambio con sus propias compuertas y su camino
rojo; y **el autor lo ejecuta en su máquina**, corre el canon `--sello` y commitea sólo lo que sale
verde. De ahí salen sus dos afirmaciones: nada entra sin pasar las compuertas que declara su
asiento, y ninguna afirmación del repositorio descansa en el conocimiento del modelo.

Medir antes que afirmar es también la primera regla de `CONTRIBUTING.md` (`CONTRIBUTING.md:10-13`),
y su sección 1 añade otra, aprendida en el §59.2: toda herramienta declara su cobertura y la
declaración se comprueba (`CONTRIBUTING.md:24-27`). Un pin del canon se actualiza igual: «Se MIDE
primero y se edita la tabla despues. Nunca al reves» (`tools/canon.sh`, su sección «Como se
actualiza un pin»; ⚠️ §854: decía `:77-79`, y la cabecera creció por encima).

**Fuera del paso 4.** Una sesión de Claude Code no propone el bloque: lo ejecuta ella.
`GENAI.md` §«Fuera del paso 4: la sesión de Claude Code» dice qué cambia, y es esto:

- **quién aplica, comprueba y commitea**: la sesión, en su contenedor y su rama, con la identidad del
  autor y sin líneas que la nombren, como manda `CLAUDE.md` §«Commits: el asistente no firma»;
- **sus compuertas**: las del canon, que corre la sesión y nadie más ve correr; el asiento dice si el
  `--sello` salió VERDE sobre el árbol del cambio y, si no se corrió entero o no salió verde, qué se
  corrió y qué dio. Ninguna compuerta comprueba que lo declarado se corriera;
- **quién acepta**: el autor, y su aceptación es la entrada del cambio en `main`.

El asiento de un cambio hecho así lo dice en su apertura, casi siempre con las palabras «fuera del
paso 4 de `GENAI.md`» (la misma sección). Y lo que un asistente redactó y entró con la aceptación
pendiente lo dice el propio documento, con las palabras «integrar no es aceptar», y no cuenta como
decisión del autor hasta que un asiento suyo lo acepte (`GENAI.md` §«Alcance»).

## 2. El sello y su asiento

El registro por cambio es `AUDITORIA.md`: «un asiento por cambio verificado, con su commit, sus
contadores, lo que el cambio **no** cierra y las lecciones que dejó» (`GENAI.md` §«Dónde está el
registro»). En los asientos, *sello* es el commit que lleva uno —«Un solo sello» (§780), «Un sello
corto de documentos» (§710)—, y *el canon `--sello`* es el nivel de compuertas que se corre sobre su
árbol antes de commitearlo (sección 6).

**El número.** Es el siguiente libre, y una sesión que trabaja en paralelo con otras lo busca en
todas sus ramas (§710: «ninguna de las seis ramas de `origin` que conoce el clon […] lleva un
§710»), o trabaja en un rango propio («el siguiente del rango de esta sesión (§770 a §799)», §778).
Hasta el §274 los encabezados son «## 274.»; desde el §275, «## §275 —».

**La apertura.** Costumbre de los asientos recientes —del §778 al §789, por ejemplo—: sobre qué
commit va; qué ficheros toca y cuáles no; qué es del autor y qué propuso la sesión; quién lo escribe,
lo comprueba y lo commitea, con «fuera del paso 4» si es una sesión; las zonas que no toca; y si el
canon `--sello` salió VERDE sobre ese árbol con el asiento en su sitio. Dos de esas cosas las pide
`GENAI.md` §«Fuera del paso 4: la sesión de Claude Code»: que la apertura diga que el cambio salió de
una sesión, y si el `--sello` salió VERDE o, si no, qué compuertas se corrieron y qué dieron;
`CLAUDE.md` §«Commits: el asistente no firma» pide también lo primero. Lo demás no lo manda ningún
fichero.

**Las secciones.** También costumbre, en negrita: **De dónde sale.**, **Quién lo decide.**, **Lo que
hace.**, **Medido.**, **Contadores.** y **Lo que NO hace.** (§778, §779, §780); a veces **El
defecto.** o **Por qué no lleva aviso de seguridad.** (§780). Lo que `GENAI.md` exige es menos: el
commit, los contadores, lo que no cierra y las lecciones.

**Los contadores.** Lo que el sello mueve, de dónde a dónde: los pines de tests de la tabla del
canon, cada uno con su crónica en su fila («§780: 147 -> 148, …», `tools/canon.sh`, la fila de
`zk-ssl-cli`), los totales que repiten `PRINCIPIOS.md`, `PAPER.md` y `PAPER_EN.md`, los vectores y
las líneas de manifiesto con su huella, y las casillas del `BACKLOG.md` (§780, «Contadores»). Que
las cifras de los documentos vivos sigan siendo las del canon lo vigilan `tools/check_cifras.py` y
`tools/check_figures.py`, en el bucle del canon (sección 6).

**Un asiento no se reescribe.** Lo dice el §237: «los asientos describen **un momento** y no se
reescriben»; y el §247: «los registros no se reescriben, se corrigen encima»: lo que un asiento dijo
mal lo corrige otro, posterior. Por eso las herramientas de cifras dejan fuera `AUDITORIA.md` y
`BACKLOG.md` (`tools/check_cifras.py:65-71`), y `tools/check_figures.py` comprueba la premisa que
hace honesta esa exclusión en el `BACKLOG.md` (sección 4).

**El doble hilo con los RFC.** El asiento que sella un cambio de protocolo cita el RFC por su número,
y el RFC cita el asiento (`spec/rfc/PROCESO.md` §«Reglas», regla 5).

## 3. Cómo se corrige un documento vivo

En su sitio, y con una nota que dice qué decía y por qué cambia: «en un documento vivo, el texto se
corrige en su sitio y una nota «⚠️ §778» dice qué decía y por qué cambia; en la tabla de este
documento, que tacha, se tacha y se pone al lado lo correcto» (§778, «Quién lo decide»). Ningún RFC
tacha: se corrige sin tachar y con una nota al final de la sección (§778, punto 4). Es la regla que
el `BACKLOG.md` da por suya: «este proyecto marca las correcciones en vez de borrarlas»
(la cabecera del `BACKLOG.md`).

No se corrige en su sitio lo que no se reescribe:

- los asientos (§237, §247), que se corrigen con otro;
- los preprints depositados, que se conservan tal cual (§237: «depositados con DOI, conservados
  verbatim»);
- los vectores publicados, que jamás se reescriben: si el cable se rompe, los viejos se conservan
  bajo su versión (`spec/rfc/PROCESO.md` §«Reglas», regla 2), y desde el §692 el canon fija la huella
  de cada uno: se puede añadir un vector, o moverlo con sus mismos bytes a `spec/vectors/<versión>/`,
  y modificarlo o borrarlo es ROJO (`tools/check_vectores.py`; el bloque «2 quater» de
  `tools/canon.sh`; ⚠️ §854: decía `:343-352`, y el bloque bajó);
- y los informes del autor con la aceptación pendiente, que «no se reescriben ni se marcan sin él»
  (§710, «Lo que NO hace», sobre `doc/ecst/`).

Cuando un documento y el código dejan de coincidir, «el código gana» (el pie de `CONTRIBUTING.md`;
⚠️ §855: decía `:171-172`, sus líneas en el §793; el §814 y los que le siguieron añadieron líneas
por encima, y el §855 otras diecisiete).

## 4. El BACKLOG

Lo que dice su propia cabecera:

- es lo que falta, **numerado** y por prioridad, para que pueda citarse por su número entre sesiones;
- **los números no se reutilizan ni se renumeran**;
- lo resuelto se marca `[x]`, se anota con qué commit y **se queda en la lista**;
- lo nuevo va al final con el número siguiente, y se coloca en su grupo sin cambiar de número.

**La línea de Estado** cuenta las casillas, `- [ ]` abiertas y `- [x]` resueltas, más las
suspendidas, y dice en qué asientos se recontó: «contada, no recordada». Cuenta casillas, no
entradas: las dos que se cerraron con la `[x]` delante y el cuerpo original debajo, con su propia
casilla, la 85 y la 77, cuentan esa casilla como abierta. Costumbre: un sello que abre o cierra una entrada la recuenta y lo dice en sus
Contadores (§779).

**Contar no basta.** La «rancia de OTRA clase» (la cabecera del `BACKLOG.md`, §583): la cuenta cuadraba y la
lista mentía, porque tres entradas abiertas estaban cerradas en el árbol. Una entrada se mira contra
el árbol, no sólo se cuenta.

**Las notas de una entrada.** Costumbre: una nota que abre con el número de su sello —«⚠️ **(§639)
…**», «⚠️ **§710: …**»— y dice qué cambia (por ejemplo, las de la entrada 84); y en una entrada
abierta que avanza, «✅ a medias (§643): la E2, el núcleo del acta […]. QUEDAN el nodo (E3), […]»,
que casi siempre acaba en «sigue abierta» (la entrada 84, la del RFC-0015). Ningún fichero
define «a medias»; la entrada sigue abierta hasta que esté cerrada de verdad, y al cerrarla, «el
cierre va detrás del cuerpo» (§779, sobre la 42, «como la 69, la 102 y la 106»).

**Ninguna cifra de tests en una entrada abierta.** Las herramientas de cifras no miran el
`BACKLOG.md`, y eso sólo es honesto mientras sus cifras de tests vivan en notas cerradas:
`premisa_de_la_exclusion` lo comprueba en cada sello (`tools/check_figures.py:55`).

**Quién cierra.** Costumbre: el sello que cierra una entrada propone el cierre con la medida
delante, y lo dice en «Quién lo decide» (§779: «Cerrar la 42 y la 104 lo propone la sesión con la
medida delante»).

## 5. Los RFC

**Qué va por RFC.** Un cambio al protocolo —lo que cruza el cable: `spec/RPC.md`, `spec/openrpc.json`
y los vectores de `spec/vectors/`— no entra por commit directo (`spec/rfc/PROCESO.md:3-6`).

**Los estados.** BORRADOR → PROPUESTO → ACEPTADO → FINAL, o RETIRADO en cualquier punto
(`spec/rfc/PROCESO.md` §«Estados»).

**Las cinco reglas** (`spec/rfc/PROCESO.md` §«Reglas»): un fichero por RFC, numerado; todo RFC
declara su compatibilidad, y si rompe el cable la versión sube y los vectores viejos se conservan
bajo la suya; todo RFC declara su efecto sobre el principio del API, que la clave de gasto no viaja
jamás, y el que lo erosione nace RETIRADO; ACEPTADO exige la spec al día, el OpenRPC regenerado, los
vectores re-emitidos o nuevos y las suites verdes; y el doble hilo con el asiento (sección 2). Un
fallo de seguridad tiene sus dos cláusulas: el arreglo propio se redacta en privado y entra de una
vez, y el borrador del reporte a un tercero no entra en el árbol público (`spec/rfc/PROCESO.md`
§«Fallos de seguridad»).

**La plantilla.** `spec/rfc/0000-plantilla.md`: la cabecera con el estado, el autor, la asistencia
GenAI, la fecha, la versión afectada y los asientos; y las secciones Motivación, Diseño,
Compatibilidad, Seguridad y Referencias.

**Las etapas y las decisiones**, que son costumbre y no regla: ni la plantilla ni el PROCESO las
piden.

- El RFC parte el trabajo en etapas, E1, E2…, y cada una se construye en su sello, que el RFC
  nombra. La mayoría las lleva en una tabla, «etapa | qué entrega | ¿rompe el cable? | estado» —el
  0001, del 0004 al 0010 y del 0012 al 0023 (⚠️ §827: decía «del 0012 al 0021»; el 0022 y el 0023 la traen desde el §827, con la tercera columna del 0018 y el 0019); por ejemplo,
  `spec/rfc/0015-el-ciclo-de-vida-del-firmante.md` §«Estado de las etapas»—; el 0002, en una tabla
  de otra forma; y el 0003 y el 0011, en una lista. En el 0018 y el 0019, desde el §826, la tercera columna es «¿rompe el cable?, y qué superficie», y la tabla va antes de «Seguridad», para no mover sus líneas citadas; la E2 del 0017 y la E4 del 0020 nombran también su superficie: lo pide la R5 del RFC-0021, un sello antes del corte. ⚠️ §826: decía «del 0012 al 0017» y «y el 0018 y el 0019, en su cabecera y su diseño»: el 0020 y el 0021 la traen desde el §790, y el 0018 y el 0019, desde el §826.
- Las decisiones abiertas van numeradas —D-A, D-B…, o 1, 2…— y son del autor. Cuando él las
  delega, las toma el asistente («Las cinco decisiones las tomó el asistente por delegación del
  autor», `spec/rfc/0009-lo-que-revela-una-prueba.md:65`), y el RFC dice con qué criterio y que son
  reversibles: «TOMADAS en el §609, por delegación del autor y con
  el criterio que dictó» (`spec/rfc/0014-el-recibo-del-lote-y-de-la-prenda.md:128`); «TOMADAS en el
  §642 por delegación del autor, con el criterio del §609, y REVERSIBLES» (la cabecera del RFC-0015).
- Un RFC suele nacer PROPUESTO con sus decisiones sin tomar y ninguna etapa construida —del
  RFC-0015, en el §639: «Cinco decisiones del autor, ninguna tomada, y ninguna etapa construida»
  (la entrada 84)—, y su cabecera dice en cada momento qué está decidido y qué construido (la
  del RFC-0019, por ejemplo).

**Quién acepta.** Lo que el registro dice al aceptar el RFC-0014 es que «la aceptación es del autor,
y la regla 4 del PROCESO pide su canon y su merge» (§618); el texto de la regla 4 pide la spec, el
OpenRPC, los vectores y las suites verdes, y no nombra el merge. El PROCESO no dice quién decide ni
con qué regla: es lo que la entrada 88 pide declarar, y este documento no lo decide.

## 6. El canon y los bancos

**Una fuente.** `tools/canon.sh` es la única fuente de las compuertas desde el §224; antes, cada
bloque las reescribía a mano (la cabecera de `tools/canon.sh`, «La fuente unica»).

**Los niveles** (la lista de órdenes de la cabecera de `tools/canon.sh`):

| orden | qué corre |
|---|---|
| `bash tools/canon.sh --sello` | cada bloque: los tests de cada crate en release y el resto de compuertas; es el de cada sello |
| `bash tools/canon.sh --largo` | lo de `--sello`, y además halo2 y plonk, y el nodo y el testigo compilados desde dos raíces (`tools/artefacto.sh --binarios`, «3 ter binarios»); ⚠️ §824: decía «lo de `--sello`, y además halo2 y plonk» |
| `bash tools/canon.sh --completo` | lo de `--largo`, y además `zk-core` |
| `bash tools/canon.sh --lista` | sólo enseña la tabla; no ejecuta |
| `bash tools/canon.sh --bancos` | los bancos, todos (`tools/bancos.sh`); ⚠️ §804: menos el de los cortes, que se corre aparte |

Son tres niveles porque meter `zk-core` en cada sello «garantizaria que alguien acabe saltandose la
compuerta, y una compuerta que se salta no protege» (§225; `tools/canon.sh`, «Los TRES niveles, y
por que tres»). El `--completo` no lo fuerza nadie: deja constancia en `.canon/ultimo-completo`, y
toda invocación dice cuándo fue y cuántos sellos han pasado (la misma sección); qué rutas lo
vencen, el bloque «lo que cubre el nivel --completo, y nadie mas» de `tools/canon.sh` (§284).
⚠️ §854: las cinco citas de esta sección hasta aquí iban por línea: `:2-6`, `:8-20`, `:49-60`,
`:62-67` y `:134-146`, las del §793. Las dos primeras seguían en su sitio; las otras tres, no: la
cabecera creció por encima, y `:134-146` caía en la tabla. Se citan por su sección o su bloque.

**Lo que lo hace robusto** (`tools/canon.sh`, «Lo que lo hace ROBUSTO, y no es la tabla»): los
miembros del workspace se leen de `Cargo.toml`, y un crate sin fila pone la compuerta roja; los
pines son exactos, no mínimos; no se para en el primer fallo; cada crate tiene su timeout; ningún
`.rs` de `src/` queda sin declarar; y todo `cargo` de los niveles corre con `--locked` (§694).
⚠️ §854: decía `:22-47`, la del §793, y la sección bajó cinco líneas.

**El bucle de herramientas** (el bloque «2 ter» de `tools/canon.sh`): las de la lista de su `for`
—`check_cifras`, `check_figures`, `check_columns`, `check_constraint_layout`, `verificar_citas`,
`check_dominios`, `check_publicadas`, `check_nucleo`, `check_techo`, `check_curvas` y
`check_superficie`—, a pin cero. Y estar en el bucle no basta, «hay que mirar algo» (§695, en el
comentario del mismo bloque): `check_columns` corría ahí sin mirar ningún `.rs` y decía verde, y
desde el §695 su universo y su prueba de vida viven en la herramienta. Fuera del bucle, cada una en
su bloque, van otras puertas; entre ellas, los vectores (`tools/check_vectores.py`, «2 quater», que
sale del bucle «para que cada ROJO entre con su nombre»), el catálogo de rechazos
(`tools/check_catalogo.py`, «2 quinquies»), la conformidad del kit en sus familias («3 bis
familias»), la segunda implementación (`tools/segunda/`, «3 duodecies») y el artefacto («3 ter»; ⚠️ §824: desde aquí en dos raíces, y desde `--largo` también el nodo y el testigo, «3 ter binarios»).
El veredicto es VERDE, o ROJO con la lista de fallos (el bloque «veredicto» de `tools/canon.sh`).
⚠️ §801: las tres líneas de `tools/canon.sh` de este párrafo decían `:312-341`, `:324-330` y
`:608-621`, las del §793; el fichero creció por delante del bucle y en el §799 ya estaban donde
dicen ahora, con el mismo texto. ⚠️ §810: la tercera, la del veredicto, decía `:697-710`; el §804
ya la había corrido dos líneas, y las catorce que el §810 mete en el «3 duodecies» la corren otras
catorce: se cita por su bloque. ⚠️ §854: las otras dos también: decían `:401-430` y `:413-419`, y
el bloque ya había bajado dos líneas.

**Los bancos.** `tools/banco_*.sh` arrancan el nodo, el testigo y el mando reales, y por eso viven
fuera de los niveles (§582). `tools/bancos.sh` los corre todos: la lista se lee del directorio, cada
banco tiene su timeout, el árbol tiene que quedar limpio antes y después de cada uno, y dos modos
largos quedan fuera, declarados (`tools/bancos.sh:7-30`). Tampoco los fuerza nadie: cada pasada
verde deja constancia en `.canon/ultimo-bancos`, y toda invocación del canon dice cuándo fue y si lo
que ejercen cambió desde entonces (`tools/canon.sh`, «Los bancos, tampoco»). Desde el §786 compilan
con `--locked`, y una puerta del canon lo exige por llamada (el bloque «§786 · los bancos compilan
con --locked» de `tools/canon.sh`; ⚠️ §854: las dos citas decían `:69-75` y `:413-416`, y ninguna
caía ya en su sitio). ⚠️ §804: los corre todos menos `tools/banco_cortes.sh`, que el punto 5 de
`tools/bancos.sh` deja fuera por su nombre: repite la medida del §699 sobre seis commits fijos de la
historia, no mide el árbol del sello, y compila seis veces. Se corre aparte, y cada pasada de los
demás lo nombra.

## 7. Quién decide qué, en lo que hoy está escrito

- **El autor decide** sobre lo medido (`GENAI.md` §«Cómo se usa, exactamente», paso 2) y **acepta**:
  en el paso 4, con su commit; fuera de él, con la entrada del cambio en `main` (`GENAI.md`
  §«Fuera del paso 4: la sesión de Claude Code»).
- **Los commits** van a nombre del autor, también los que hace una sesión, y ninguno lleva
  `Co-Authored-By` ni `Claude-Session:` (`CLAUDE.md` §«Commits: el asistente no firma»).
- **Lo que propone una sesión** lo dice el asiento en «Quién lo decide» (costumbre: §778 a §789).
- **Las decisiones delegadas** las toma el asistente con el criterio que el autor dictó, y el RFC
  dice que son reversibles (sección 5).
- **Lo que un asistente redactó** y entró con la aceptación pendiente no cuenta como decisión del
  autor hasta que un asiento suyo lo acepte (`GENAI.md` §«Alcance»).
- **Lo que no está escrito**: quién mantiene el proyecto, el criterio de aceptación de un RFC y el
  segundo par de manos; la entrada 88 los pide, y no se suplen aquí.

## Procedencia

Escrito en una sesión de Claude Code (§793), fuera del paso 4 de `GENAI.md`, sobre el árbol del §789
(`8c02941`): cada cita se leyó en ese árbol. Integrar no es aceptar: la aceptación que describe
`GENAI.md` §«Alcance» queda pendiente del autor.
