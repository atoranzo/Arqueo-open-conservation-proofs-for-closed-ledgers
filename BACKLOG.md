# BACKLOG

Lo que falta, numerado y por prioridad. Existe para que las cosas puedan
citarse por su numero entre sesiones, igual que las secciones de
`AUDITORIA.md`.

**Los numeros no se reutilizan ni se renumeran.** Cuando algo se resuelve
se marca `[x]` y se anota con que commit, pero **se queda en la lista**.
Una lista que borra lo resuelto no deja ver que se ha hecho ni en que
orden; y este proyecto marca las correcciones en vez de borrarlas.

Lo que entre nuevo va al final con el numero siguiente, y se coloca en su
grupo de prioridad sin cambiar de numero.

**Estado**: 43 abiertas, 90 resueltas — **3 suspendidas** (16, 22 y 28). Casillas recontadas
en el §620 (la línea seguía con las del §597, 45 y 63), el §621, el §622, el §696, que abre la
115, el §697, que abre la 116, el §705, que abre la 117, el §707, que abre la 118, el §772, que
cierra la 102, el §779, que cierra la 42 y la 104, el §783, que cierra la 18, el §802, que abre la
119, el §812, que abre la 120 y la 121, el §835, que abre la 122, el §813, que abre la 123 y la
124, el §814, que cierra la 41 y abre la 125, el §838, que cierra la 122, y el §860, que cierra la
8, la 17, la 20, la 23, la 47, la 76 y la 96 y abre la 126 y la 127, el §861, que no mueve ninguna,
el §815, que abre la 129, el §820, que cierra la 120, el §821, que cierra la 121, el §822, la 75, el §864, que cierra la 94, el §896, que abre la 130, y el §897, que abre la 131 y la 132.
Ultima revision: 30 de septiembre de 2026 — **contada, no recordada** (§583 a §587, §595, §597).

⚠️ **Rancia de OTRA clase** (30-09-2026, §583): la cuenta cuadraba —50 y 56— y aun
asi la lista mentia. Tres entradas abiertas estaban cerradas en el arbol: la 69
desde el §176, la 106 desde el §361 y la 98 desde el §448. Contar casillas no ve
eso; hay que mirar el arbol. Miradas contra el, una a una: 55, 69, 81, 94, 95, 97,
98, 101, 105 y 106. Las otras cuarenta, NO.

## La cadena de la oponibilidad, de un vistazo

⚠️ Anadida en §233 porque estaba **repartida entre las entradas 53, 56 y
62**, y reconstruirla exigia leer las tres. Sin ella se ordena por lo que
se recuerda en vez de por lo que bloquea: en §233 la cola de traspaso
puso **el acuse primero**, que es el ultimo eslabon.

```
  1. esquema de firma ......... 53  CERRADA (§127.1) · xmss MT 40/8, medido
  2. GUARDIAN DEL INDICE ...... 56  CERRADO (267) · firma_indice.rs, 9 tests
  3. cabeza firmada, emitida .. 56  CERRADO (267) · firma_cabeza.rs, 5 tests
  4. EpochHead + 2 extensiones  48  verifier_hash (§104.3) + raiz de recepcion (§121)
  5. acuse .................... 62  HECHO (§275): raiz y N firmados; camino servido
```

**Cada eslabon depende del anterior.** El acuse sin cabeza firmada no es
evidencia portable (§121.7), y la cabeza sin guardian del indice no es
oponible sino peligrosa: **reusar el indice filtra la clave** (§110.2).

⚠️ Y dos dependencias que no estan en esa columna: el acuse **hereda
§116** si usa `digest_of_proof` tal cual —§116 se cierra antes—, y
**XMSS cambia la premisa de la entrada 19**: sin WAL, perder una
operacion dejaba de ser recuperable.

⚠️ **La 41 no se cierra hoy, y eso es deliberado.** Se clasificaron sus 80
fallos y aparecieron **dos que no eran la clase conocida** (§78). Los 78
restantes tienen causa decidida pero **no medida uno a uno**, y marcarlos en
bloque seria el error que §77 acaba de desmentir.
⚠️ §814: se cierra en el §814, medida, y no marcando en bloque: el 15-10-2026, en
depuración, la capa da 443 pasan, 0 fallan y 45 ignorados, y ninguna de sus 38
marcas de depuración tapa uno de los 93 del 26-08-2026. La medida, en la entrada.

⚠️ **El 31-07 aparecio la 43 leyendo codigo para otra cosa**: la capa no
verificaba las pruebas de la via de pago, y un tercero vaciaba cualquier
cuenta sin clave. Corregida el mismo dia. Queda `[x]` porque esta cerrada,
pero se deja arriba: es el fallo mas grave que ha encontrado esta auditoria
y ninguna herramienta del proyecto podia verlo.

⚠️ **La 40 dejo de ser hipotesis.** Se abrio por la mañana como lectura sin
medir y se confirmo por la tarde con testigo en release: es el **cuarto
fallo de solidez** de la auditoria (§72). Sigue abierta —confirmar no es
corregir— y es hoy la entrada de mas peso de esta lista.

⚠️ **NOVENA rancia, cazada por el método** (05-08-2026): la cabecera
juraba «34 abiertas, 35 resueltas» y el conteo del propio fichero dio
**33 y 39** ANTES de la revisión de hoy — llevaba desfasada desde alguna
tanda anterior sin que nadie lo notara. La diferencia con las ocho
previas: esta la cazó **el script midiendo antes de escribir**, no un
lector después. La cabecera de hoy sale del conteo, y así saldrán todas.

⚠️ ~~16 abiertas, 21 resueltas.~~ **Estaba mal por los dos lados**, y se
descubrio contando los `- [ ]` y `- [x]` del propio fichero: eran **17 y
22**. Con las tres nuevas de hoy, **20 y 22**. Es la **octava** vez que esta
cabecera se queda rancia, y la nota de abajo solo llevaba la cuenta hasta la
septima. Se corrige **midiendo**, que es lo que enseñaron las entradas 2 y 4.

⚠️ **Esta cabecera se quedo rancia seis veces seguidas** durante la sesion
del 30-07: se actualizaba al cerrar una tanda y la tanda continuaba. Llego a
anunciar como abierta la entrada 37, cerrada ocho commits antes, y a
describir la 32/33 como «diseño» cuando ya era un piloto funcionando. Queda
anotado porque es el mismo fallo que la propia lista denuncia mas abajo:
**escribir desde el estado que se recuerda en vez de comprobar el arbol**.

### Solidez del circuito: cerrada

Los TRES constructores de compromisos tenian el solapamiento de §38.
`claim` (§39 titularidad, §50.7 aleatorio) y `send` (§50 identidad)
corregidos; `mint_pending` (§35) verificado sano. La **37** generalizo el
hallazgo: `tools/check_constraint_layout.py` barre los 24 circuitos y no
queda ninguna colision. ⚠️ Empezo cubriendo solo 14 y saltandose los
otros diez EN SILENCIO (§59.2); esta frase decia «14» hasta que se corrigio
—septima vez que esta cabecera se queda rancia en la misma sesion—. El frente de grados (6, 24, 25, 34) declarado como
limite conocido de winterfell (§46, §20).

⚠️ Lo que ese barrido **no** cubre: restricciones ausentes o mal formuladas.
§39 no se habria detectado con el (§53.5). Para esa clase sigue sin haber mas
instrumento que la lectura semantica y el test discriminante — que es el
argumento de la **entrada 7**.

### Custodios (32/33): de frente abierto a cola de trabajo

Mecanismo completo y verificado: dos circuitos de carril unico, variante B
decidida y medida (§52), umbral 2-de-N reconstruido fuera del circuito (§54),
operacion atada al nulificador (§55), compromiso que enlaza con cada circuito
(§56), y **piloto funcionando en `governance`** (§57). Faltan `freeze`, `recovery`,
`mint_pending` y `mint`, y ⚠️ **no son repeticion mecanica** (§58): cada uno
conserva contenido propio que el circuito debe seguir probando.

### Lo demas

Auditoria externa (7), declaradas (8, 11, 12, 15), operacion (grupo E),
consenso (23), y los preprints al final (16, 28) — que ya acumulan tres
fallos de solidez y una brecha de confianza sin declarar. ⚠️ §860: la 8 y la 23 se cerraron en el §860, decididas: la 8, sin autorización de custodios para abrir y con su residuo en `SECURITY.md` §2; la 23, sin emprenderse.

⚠️ **Reordenado el 30-07-2026.** Diez entradas nuevas entraron en una
sola sesion ancladas sobre lineas concretas, y varias quedaron en el grupo
equivocado. Se han recolocado por prioridad **sin cambiar ningun numero**,
como manda la convencion. El orden de lectura ya no es el numerico: es el
de los grupos.

⚠️ **Esta lista nacio desfasada.** Se creo en el commit `9670e76`
listando como pendientes las entradas 2 y 3, que ya estaban resueltas y
empujadas en `f673c8e` y `a6be4b2`. Se escribio desde el estado que su
autor recordaba en vez de comprobar el arbol. Queda anotado porque es
exactamente el fallo que la convencion de abajo pretende evitar, y porque
una lista de pendientes que no se verifica contra el repositorio es otra
afirmacion sin comprobar.

---

## A. Cerrada: la obra en curso

El repositorio volvio a describir lo que hay. Las cinco resueltas.

- [x] **1. Coherencia de los papers.** ~~`PAPER.md`, `PAPER_EN.md` y la
  documentacion autocontradictoria de `circuit_send` siguen describiendo
  el arbol de nullificadores como parte del estado.~~
  **Resuelto** el 30-07-2026, commit *«Align papers and circuit_send docs
  with the retired nullifier tree»*: §3.3 pasa a nombrar el arbol de
  pendientes, la tabla de no-pertenencia queda en dos filas con el doble
  gasto remitido al encadenamiento de raices, el aviso del limite del
  cumpleanos se reescribe como limite que existio, y la lista «que
  demuestra» de `circuit_send` deja de contradecir a la seccion «por que
  NO lleva nullificador» doce lineas mas abajo.

- [x] **44. ✅ Tres tests de solidez de custodios en depuracion:
  diagnosticados y corregidos.** ~~Sin diagnosticar.~~ **Medidos** el
  31-07-2026 (§77): **un sintoma, DOS causas**. (1)
  `the_index_test_rejects_for_the_right_reason` **no era un fallo**: la
  traza viola `C_ACC_FINAL` a proposito y en depuracion la caza el
  **probador** -nombrando la restriccion 23 en el paso 39- en vez del
  verificador; el test afirmaba una sola forma de rechazo para los dos
  modos. (2) Los otros dos usaban `paths_mias[0]`, y el indice 0 anula los
  bits de camino: `C_PLACE` de 126 a 63 y `C_BIT_BOOL` a 0 —entrada 6—.
  ✅ **Corregidos**: el primero espera lo que corresponde a cada modo, los
  otros usan un indice no degenerado. **Los tres vuelven a correr en
  depuracion**: se gana cobertura, no se pierde. ⚠️ **Arreglo opuesto al de
  §71.3 y con criterio explicito** (§77.3): alli no se cambio la posicion
  porque la 0 es la que produccion usa; aqui el indice del atacante es
  incidental. ⚠️ **El proyecto ya lo sabia**: `tests_support.rs` lo tenia
  escrito y los demas tests del mismo fichero usaban 1 y 2 (§77.4).

  ~~Original:~~ **44. ⚠️ Tres tests de solidez de custodios fallan en
  depuracion, sin diagnosticar.** `the_index_test_rejects_for_the_right_reason`,
  `an_attacker_cannot_bring_their_own_custodian_set` y
  `one_real_and_one_forged_custodian_do_not_meet_the_threshold`.
  **Preexistentes**: medido volviendo a `44c5b8c`, donde fallan los mismos
  tres (§76.1). Pasan en release. ⚠️ **La causa NO esta medida**: podrian ser
  la clase de la entrada 6 o no, y atribuirlo sin comprobarlo es el error
  que este proyecto lleva setenta y seis secciones registrando. Metodo que
  ha funcionado hoy: diffear el vector de grados, aislar la variable, test
  discriminante. ⚠️ Son los tres que comprueban que **un custodio falso no
  alcanza el umbral** y que **el indice no se puede mentir**: por lo que
  cubren, merecen mirarse antes que la 41.

- [x] **2. Cifras del repositorio.** ~~65 de 174 en depuracion, 174 tests
  de la capa, doce circuitos, y la contradiccion 56/65 dentro de
    ⚠️ **Se volvio a quedar rancio.** Al remedirlo el 31-07-2026 (§76) el
  README seguia publicando 201/174/«doce circuitos» —hoy 272/201/27— y, peor,
  **una afirmacion falsa**: que el crate de circuitos pasaba en los dos
  modos. No pasa, y no desde hoy. Corregido con lo medido. La entrada se
  queda `[x]`: lo que fallo no fue esta correccion sino que **nada vuelve a
  medir esas cifras sola**.
  `AUDITORIA.md`.~~ **Resuelto** el 30-07-2026, commit `f673c8e` *«Correct
  stale and contradictory test and circuit counts, measured»*: los 65
  medidos con `cargo test -p zk-ssl` sin `--release` (109 pasan, 65 fallan
  de 174).

- [x] **3. DOI de la tercera revision.** ~~La seccion de publicacion apunta
  a versiones anteriores de los tres preprints.~~ **Resuelto** el
  30-07-2026, commit `a6be4b2` *«Point publication section at the
  third-revision DOIs»*: 21693706, 21693709 y 21693718, con las versiones
  anteriores citadas y una tabla de que corrige cada salto.

- [x] **4. Medir `stark-experiment` en depuracion.** ~~El README publica
  «199 y 200» y en release hoy son 201; ultima cifra sin verificar.~~
  **Medido** el 30-07-2026: **201 en release; 199 mas 2 saltados en
  depuracion**, y 51 s frente a 9 s (cinco veces, no seis). El README
  hablaba ademas de UN test saltado y son **dos**
  (`circuit_mint_pending` y `range_check`), los dos por grado real cero.

- [x] **5. Fuentes de los preprints al repositorio.** ~~Los papeles dicen
  «reproducible desde el artefacto» y el texto publicado no esta en el
  artefacto.~~ **Resuelto** el 30-07-2026, commit *«Add preprint sources so
  the published text lives in the artifact»*: `doc/preprints/` con las tres
  fuentes, el estilo, el guion de conversion y una nota de procedencia que
  declara que son una **reconstruccion** desde la capa de texto de la
  segunda revision. La cuarta revision ya sera un parche.

## B. La siguiente de verdad: el modelo de confianza

La 32 encabeza. La 33 es su correccion de diseno. La 6, 25 y 24 son el
frente de los grados, con el diagnostico ya cerrado (§37.7) y el precio por
decidir.

- [x] **32. ✅ Las claves de custodio ya NO llegan al operador.**
  ✅ **CERRADA (§171)**: la vía-recibo de custodios —cinco funciones, cinco
  applies, cinco recibos y los dos tipos `ThresholdAuth`/`GovernanceAuth`—
  retirada del árbol (B-3b-ii, `d19b20e`, −990 líneas). Ya no existe camino
  por el que una clave de custodio llegue al operador: toda operación
  privilegiada pasa por la vía delegada (§47/§51). La suite (231 tests)
  ejercita la vía real en cada pasada; su precio se midió antes (§162:
  x3,54-3,70). La 33 es su gemela.

  ⚠️ **Estado al 31-07-2026**: las **cinco** operaciones tienen via delegada
  (§71) y las **cinco** vias antiguas estan marcadas `#[deprecated]` —§65
  habia marcado tres; `mint` y `mint_to_pending` no, porque les faltaba la
  delegada, y las tres notas viejas condicionaban la marca a eso mismo, asi
  que estaban rancias (§80.1)—. **Lo que queda es RETIRARLAS**, y el
  inventario esta medido en §80: **138 llamadas** directas y, sobre todo,
  **145 usos de `open_and_fund`**, que llama a `.mint()` por dentro — la
  mitad de la suite depende de la via antigua **sin nombrarla**. ⚠️ **El
  precio no esta medido**: la delegada genera tres pruebas donde la antigua
  genera una, y la suite esta en 31 s / 250 s. **Siguiente paso concreto
  (§80.5)**: migrar solo `open_and_fund` en una rama y cronometrar. Con ese
  numero se decide entre retirar o declarar que no se retira, como §46 hizo
  con la 6. ⚠️ Mientras existan, el fallo esta **evitable, no cerrado**
  (§65.5, §80.6).
  ✅ **§80.5 EJECUTADO** (§162-§163): instrumento apareado §89.1, cinco
  muestras — vieja 108,4-109,7 ms, delegada 388,2-403,1 ms, x3,54-3,70;
  decision por principios (§163): RETIRAR — `open_and_fund{,_wide}`
  fondean por la delegada desde B-0b-ii. Quedan: B-2 (los ~70
  llamadores directos de tests) y B-3 (jubilar las cinco marcas).
  ~~Original:~~ No hay via
  cliente para operaciones privilegiadas: `ThresholdAuth` lleva las claves
  en crudo y la capa construye la traza (§41). Quienes conservan su clave
  solo pueden mover su dinero; quienes la entregan pueden crearlo.
  **Confianza residual no declarada** en los preprints: va con la 28.
  **Diseño de la solucion cerrado en §47** (via B); la implementacion es la
  33.

- [x] **33. ✅ Los custodios prueban en su máquina, y la autorización
  cubra los parametros.** La correccion de diseno de la 32 (§41.4).
  **Diseño cerrado (§47): via B**, firmas por conocimiento de preimagen con
  mensaje atado, verificadas por separado —sin componer pruebas, que el
  proyecto no sabe hacer y no necesita aqui—. Primer paso medible en §47.5:
  separar un carril de `circuit_threshold` y medirlo. Exige toolchain; no es
  un parche. ✅ **Preparada (§51)**: inventario del circuito de dos carriles,
  el unico punto de acoplamiento localizado (el orden estricto
  `IDX_B - IDX_A - 1`), y especificacion del `circuit_threshold_single` con
  su metrica de exito. ⚠️ **Pregunta abierta que el experimento debe
  resolver**: al separar, el orden estricto —que eran custodios distintos—
  desaparece; hay que reimponerlo en la capa o el umbral acepta la misma
  firma dos veces (§51.3). ✅ **EXPERIMENTO EJECUTADO Y MEDIDO** el
  30-07-2026 (§52), en dos variantes: **la via B es viable**. Dos pruebas de
  carril unico cuestan 8,2 ms y ~30 KB frente a 5,1 ms y 20 KB de una
  conjunta —1,6× en tiempo, 1,5× en tamano, despreciable en operaciones
  raras—. **Recomendada la variante B** (nulificador desde la clave):
  preserva el anonimato dentro del conjunto que `circuit_threshold` ya
  declara, y **no cuesta mas** (14 columnas frente a 16 de la variante con
  indice publico). ⚠️ B deja **enlazabilidad** entre operaciones del mismo
  custodio (§52.4); cerrarla exige atar el nulificador al identificador de
  la operacion, que hace falta igualmente para la otra mitad de la 33.
  ✅ **VARIANTE DECIDIDA: B** (§52.7), por coherencia —arreglar la 32 no
  puede costar el anonimato que el circuito ya declaraba— y porque no cuesta
  mas. La variante A queda marcada como NO ELEGIDA, conservada como la
  comparacion medida. ✅ **(b) HECHO** (§54): `verify_threshold_pair`
  reconstruye el umbral fuera del circuito. Al escribirla aparecio un
  **segundo** agujero que la separacion abre y que ni §51 ni §52 vieron: el
  atacante puede **traer su propio conjunto de custodios** —dos pruebas
  validas, nulificadores distintos— si la raiz sale de la prueba en vez de
  ponerla la capa (§54.2). ✅ **(a) HECHO** (§55): el nulificador se
  deriva de `H(dominio, clave, operacion)`, lo que cierra **a la vez** la
  reproduccion de §54.4 y la enlazabilidad de §52.4, sin coste en filas.
  Verificado con test discriminante de la constancia de `COL_OP` (§55.2).
  ✅ **El puente hecho** (§56): `commit_operation` ata la
  autorizacion a los parametros de la operacion concreta, con un dominio por
  tipo; sin el, una autorizacion para emitir 1.000 serviria para emitir
  1.000.000. **Falta solo (c)**: sustituir `ThresholdAuth` en los cinco
  circuitos (`mint`, `mint_to_pending`, `freeze`, `recovery`, `governance`).
  ⚠️ Es **amputar** de cada uno el tramo de subida de custodios que hoy
  llevan empotrado (en `mint`: filas 272-311 y ocho columnas), no cambiar una
  firma. ✅ **PILOTO HECHO en `governance`** (§57):
  `apply_governance_delegated` cambia el conjunto de custodios sin que las
  claves lleguen al operador, con cuatro tests de rechazo. `governance` era
  el piloto correcto porque **casi todo el circuito es autorizacion**: al
  amputarlo no queda circuito, solo una suma. ⚠️ Los tests destaparon un
  fallo de diseño que seis secciones no vieron (§57.2): el circuito llevaba
  `CUSTODIAN_DOMAIN` incrustado y no podia autorizar gobernanza. Corregido:
  el dominio es entrada publica y la jerarquia queda explicita.
  ⚠️ **Faltan cuatro, y NO son repeticion mecanica** (§58): gobernanza era
  especial porque al amputarla no quedaba circuito. `freeze` (76 ranuras),
  `recovery` (114), `mint` (118) y `mint_pending` (125) **tienen contenido
  propio que debe sobrevivir**, con los carriles de hash compartidos entre su
  subida y la de custodios. Cada uno es **una sesion con toolchain**:
  circuito amputado, via delegada en la capa, tests de rechazo. `mint` y
  `mint_pending` al final. **`freeze` HECHO** (§60) y **`recovery` HECHO** (§64):
  `circuit_recovery_climb` con 6 tests -incluido el de regresion que impide
  cambiar el saldo- y `apply_recovery_delegated` con 4 de rechazo. **TRES de
  cinco operaciones ya no necesitan las claves en el operador**, y las vias
  antiguas llevan `#[deprecated]` con el motivo (§65). **`mint` HECHO** (§66, §67):
  `circuit_mint_climb` con 7 tests -conservacion y tope, este por los dos
  bordes- y `apply_mint_delegated` con 4 de rechazo, incluido que autorizar
  250.000 no permite emitir un millon. **CUATRO de cinco.** ⚠️ ~~Queda
  `mint_pending`: hasta que la tenga, el fallo de la 32 sigue abierto
  para la emision a pendientes.~~ **`mint_pending` HECHO** el 31-07-2026
  (§70, §71): `circuit_mint_pending_climb` -125 ranuras a 89, 49 columnas a
  39, 41 periodicas a 32- con 13 tests, y `apply_mint_pending_delegated` en
  **`two_phase.rs`** -no en `pending.rs`, que era errata de §68 (§71.1)-
  con el positivo y cuatro de rechazo. **CINCO DE CINCO.**
  ⚠️ **Y aun asi la 32 NO se cierra**: las vias antiguas siguen siendo
  llamables con `#[allow(deprecated)]`, y §65.5 avisa de que la garantia no
  la da la marca sino usar la via delegada. Lo que queda es **retirarlas**,
  que es lo que §65.4 fija para cuando las cinco esten -o sea, ahora-.
  **Analizada** (§42): no se mueve «al cliente» porque los custodios son
  dos y el circuito prueba conocimiento de ambas claves en una sola traza.
  La via que cierra las dos mitades es **verificar firmas en circuito**,
  mismo primitivo que la 21. Primer paso: **medir su coste**, que no esta
  medido. ⚠️ **Reencuadrado (§43): no hacen falta firmas.** El sistema
  autentica por conocimiento de preimagen de un hash, primitivo que ya
  corre en cada pago. El trabajo real es **partir la prueba conjunta de dos
  claves en dos componibles**, reestructuracion del umbral, no un primitivo
  nuevo. La 21 puede o no compartir esto, segun que signifique alli
  «tercero».
  ✅ **IMPLEMENTADA Y CERRADA (§171)**: la vía B vive en los cinco
  `apply_*_delegated`; el orden estricto §51.3 (`index_a < index_b`) se
  reimpone en la capa y todos los tests migrados lo ejercitan. La
  autorización cubre los parámetros: el compromiso ata raíces, contador y
  operación (§54.2 en vivo, §168). La evidencia es la suite por la vía
  real, no una promesa.

- [x] **6. Grado dependiente del testigo (pendientes): DECIDIDO y DECLARADO.**
  La unica comprobacion automatica del area de menor confianza (§16.3)
  esta apagada: solo corre en depuracion y ahi fallan 65 tests (§20, §35).
  **Replanteada** el 30-07-2026 (§37): el remedio no es reformular el
  circuito sino **no asignar la posicion 0**, y hay un experimento
  pre-registrado en §37.3 para decidirlo. Winterfell exige que el grado
  declarado **coincida** con el real, asi que declarar menos no es salida.
  **Precio medido (§44)**: capacidad y tests baratos, pero la correccion
  arrastra una **migracion de pendientes vivos** en ledgers existentes —del
  peso de §36—, o aceptar correccion solo-hacia-delante. ✅ **DECIDIDO**
  (§46): **se declara, no se migra**. La rama solo-hacia-delante **no
  existe** —`allocate_pending` reutiliza huecos, un ledger recae (§46.1)—, y
  migrar fondos para arreglar una comprobacion de *depuracion* es
  desproporcionado. Se unifica con 24 y 25 como limite conocido de
  winterfell. Falta redactarlo: entrada 34.
  **Experimento ejecutado** el 30-07-2026 (§37.4): tercera fila, el
  desplazamiento **se descarta** —de 21 desviadas a 13, pero los fallos
  suben de 65 a 66—. Establecido que ninguna asignacion secuencial lo
  cierra, porque las primeras posiciones tienen casi todos los bits altos a
  cero. Candidata sin verificar: permutar el contador de forma biyectiva
  (§37.5). ⚠️ **Todo lo anterior queda anulado por §37.6**: el experimento
  modifico `PendingTransfers`, un modelo que la capa **no ejecuta**, asi que
  no midio nada. ✅ **Repetido bien** el 30-07-2026 (§37.7), interviniendo
  en `two_phase::allocate_pending`: **segunda fila**, 64 fallos frente a 65
  y **ni una** `C_PEND_*` ni `C_PBIT_BOOL` desviada en ninguno de los seis
  circuitos. El diagnostico queda cerrado. ✅ **Cerrada** el 30-07-2026:
  decidido «declarar, no migrar» (§46) y **redactado** (entrada 34) en
  README y AUDITORIA §20. No queda accion: limite conocido de winterfell, no
  fallo de solidez —release genera y verifica bien—. El «falta decidir el
  precio» de mas arriba queda superado por §46.

- [x] **25. Grado dependiente del testigo (cuentas/congelados): DECLARADO.** Tras cerrar el diagnostico del pendiente (§37.7)
  quedan 64 fallos en dos bloques que aparecen en casi todos los circuitos;
  por su posicion son las subidas a **cuentas** (indices 0 y 1) y a
  **congelados** (arbol vacio). Atribucion **sin verificar**: mapear indices
  contra las constantes antes de afirmarla. ✅ **Cerrada** con la 6 (§46,
  §34): misma clase, declarada como limite conocido de winterfell.

- [x] **24. Grado dependiente del testigo (valores de dominio): DECLARADO.** `circuit_mint_pending` con el margen del tope a cero y
  `range_check` con diferencia cero degeneran por valores que el dominio
  necesita, no por como estan escritos; probablemente no tenga arreglo y
  lo correcto sea **declararlo** como limite de la herramienta (§37.2,
  caso B). Es la causa de los 2 tests que `stark-experiment` se salta en
  depuracion. ✅ **Cerrada** con la 6 (§46, §34): declarada como limite de
  winterfell; para valores de dominio no hay arreglo posible (§37.2 caso B).

*Las ocho siguientes (83, 85-91) entran en §287: la comparacion externa
de la sesion 22 (Sigsum · Validium/StarkEx · QRL, consultados el
2026-08-12; procedencia completa en el asiento §287 de AUDITORIA.md),
disuelta aqui para que no viva en un documento paralelo. **Todas son
RAZONADAS, sin terreno**: destinos con su razon escrita, no cortes. La
**84** y la **92** —el ciclo de vida del firmante— llegaron en §288,
con su linea de familia dentro de la 19 y el **93** (el nodo mentiroso,
instrumentacion) al grupo E.*

- [ ] **83. La cadena de la confianza residual: cuatro eslabones, en
  este orden.** Las cuatro piezas que quedan entre «la censura es
  evidenciable» y «la censura es imposible en solitario». **Van juntas
  en una entrada porque cada eslabón es la precondición del siguiente**:
  separadas vuelven a parecer ideas sueltas, que es lo que ya pasó con
  el acuse hasta §233.

  **Eslabón 1 — El paquete de evidencia portable.** *Casi hecho; lo que
  falta es empaquetarlo y declararlo.* La escotilla de Validium existe
  porque sin los datos el titular **no puede computar la prueba de
  Merkle** que necesita para demostrar su posición: la escotilla es el
  derecho a **construir tu prueba**, no el contrato. ZK-SSL no tiene L1,
  pero el arco §275-§282 ya construyó las piezas: cabeza firmada que el
  titular custodia, camino de acuse de época cerrada, compromiso
  recomputable, y un verificador independiente que compila **sin el
  probador** (§243). Falta juntarlo en un artefacto con nombre, formato
  y comando, y **declararlo como entregable** — lo que sostiene la
  posición del titular ante un tercero cuando el operador desaparece o
  miente.
  ⚠️ **Y es MEJOR que la escotilla de Validium en un eje concreto**: en
  validium el titular **depende** de que el comité le entregue los
  datos, y el fallo característico de ese modelo es que el comité no
  puede robar pero **sí congelar indefinidamente**. Aquí el titular ya
  custodia lo suyo: no hay a quién pedírselo. Eso es un capítulo del
  paper, no una nota de ingeniería.
  ✅ **ESLABON 1 CUMPLIDO (§289)**: el paquete existe con nombre, formato
  y comando — formato v1 declarado en la cabecera del binario de
  `zk-ssl-verify` (las respuestas del cable TAL CUAL, reunidas), y el
  mando verifica cabeza y acuse sin el nodo. La entrada sigue ABIERTA:
  quedan los eslabones 2-4. Y la 91 ya tiene su mecanismo con nombre.

  **Eslabón 2 — MMR, y por fin con su razón concreta.** Hoy el MMR es un
  titular razonado —«acumulador sin hojas»— sin motivo escrito. El
  motivo es éste: **un cliente con una cabeza vieja debe poder verificar
  que la nueva la EXTIENDE**, sin descargarse el registro. Hoy el
  encadenado existe entrada a entrada, pero no hay un objeto que pruebe
  «esta cabeza contiene aquélla». Y sin ese objeto **el eslabón 3 es
  imposible**: un testigo no puede cofirmar lo que no puede comprobar.
  Deja de ser un acumulador porque sí y pasa a ser la precondición de la
  cofirma.
  ✅ **ESLABON 2 — estructura CUMPLIDA (§291), atadura al formato
  PENDIENTE.** El objeto existe: `mmr.rs` en el verificador (cima,
  inclusion, consistencia — RFC 6962 sobre las primitivas de la casa,
  dominios `MMRHOJA1`/`MMRNODO1` en el registro), con la bifurcacion, el
  recorte y el interior-como-hoja ejercitados en rojo. **La cabeza aun
  no lo compromete**: la cima dentro del digest firmado (v3, rotura
  declarada) es el siguiente corte de la familia, y hasta entonces el
  eslabon NO esta cerrado — un objeto que nadie firma no ata a nadie.
  ✅ **ATADURA CUMPLIDA (§292) — el ESLABON 2 queda CERRADO.** La cima
  y su tamano viajan FIRMADOS: formato v3 por envoltura (el molde de
  §275), genesis declarado, el vector de conformidad pinando la
  composicion v2 para siempre, el mando de §289 aceptando v2 Y v3 (lo
  custodiado no caduca), y el nodo sembrando sus hojas del diario — el
  diario manda, la memoria es cache. Servir la prueba de consistencia
  por el cable es SERVICIO, no atadura: vive como corte propio (el
  (iii) de la familia). El eslabon 3 ya tiene su precondicion.
  ✅ **SERVICIO CUMPLIDO (§293) — la familia del eslabon 2, COMPLETA.**
  `zkssl_consistencyProof` (metodo 22) sirve el camino; el mando come
  el paquete de extension (dos cabezas v3, misma clave, el objeto de
  §291 como juez); `tools/banco_extension.sh` lo demuestra contra un
  nodo vivo — incluido el rearranque sin diario respondiendo «va por
  detras»: la promesa del reseteo visible, hecha cable. Y la reparacion
  que esto destapo: la pareja del MMR no viajaba en `signedEpochHead`
  (hueco de §292); ahora viaja, y el banco del apagado vuelve a verde
  sobre cabezas v3. Del eslabon 2 no queda nada; siguen el 3 (cofirma)
  y el 4 (v4, hash del verificador).

  **Eslabón 3 — Cofirma del testigo, con política de CLIENTE.** Hoy el
  operador firma la cabeza y el testigo la observa **después**: la vista
  dividida se detecta *a posteriori*. Si la cabeza no es válida sin
  **k cofirmas**, el operador deja de poder emitir dos cabezas para el
  mismo índice: pasa de **evidenciable** a **imposible en solitario**.
  Modelo de referencia (Sigsum): el testigo pide una **prueba de
  consistencia** contra la última cabeza que él mismo cofirmó,
  comprueba frescura y append-only, y sólo entonces firma; **qué
  testigos valen y cuántos lo decide el CLIENTE**, con su política, no
  el operador con su declaración.
  ✅ **Más barato de lo que parece**: el testigo sólo guarda **la última
  cabeza observada**, y la prueba de consistencia es O(log N) — por eso
  en ese ecosistema los testigos se operan como infraestructura pública
  sin coste. Y en ZK-SSL la cabeza **ya se firma una vez por época**,
  así que el coste de XMSS está amortizado ahí y no por operación
  (§127.1: firmar por acuse colapsaría de ~320 a ~6 op/s).
  ⚠️ **Tres precios, los tres declarados**: (a) la propiedad pasa a ser
  **de umbral**, no absoluta — un atacante que controle más del umbral
  de testigos la rompe; (b) **liveness**: si los testigos no responden,
  la época no cierra, y en el modelo de referencia eso se acepta
  explícitamente renunciando a promesas de inclusión de baja latencia;
  (c) exige **terceros de verdad** — es tanto organización como código.
  ⚠️⚠️ **PREGUNTA ABIERTA, y no es de implementación**: el diseño de
  referencia usa Ed25519 y **ZK-SSL firma con XMSS con estado**. Una
  cofirma de umbral sobre firmas hash-based con índice —presupuesto
  finito, reutilizar el índice filtra la clave (ver entrada 84)— **no
  es un detalle**: hay que medir si cada testigo firma con su propia
  clave XMSS o si ahí conviene otra primitiva. **Esto se mide antes de
  diseñar nada.**
  ✅ **Medido y contestado en el RFC-0013 (§603)**: en el medio no firma XMSS. El
  ecosistema de testigos de C2SP tiene desde 2026 una cofirma poscuántica sin
  estado, ML-DSA-44 tipo `0x06`, y mtc-core la implementó y la contrastó byte a
  byte con la implementación de referencia del IETF. XMSS sigue firmando la
  cabeza; el medio es otro canal con otra clave. Lo que sigue abierto de esta
  entrada es el despliegue, no la primitiva.
  ✅ **TRAMO (i) CUMPLIDO (§294) — el testigo CONSUME la extension.**
  Antes de anclar, el testigo pide `zkssl_consistencyProof` con el
  `mmrSize` que custodia y juzga con `mmr::verificar_consistencia`: si
  la cima nueva NO extiende a la suya, **SE DETIENE** — la clase
  hermana de la vista dividida. Canal APARTE del veredicto de la cabeza
  (son dos preguntas ortogonales), diario v2 con la pareja y el camino
  para que un tercero reaudite sin el nodo, y juicio ASINCRONO por
  diseno: la pareja firmada es el acumulador ANTES de cada cabeza, asi
  que el camino de tamano t lo firma la SIGUIENTE en emitirse. De paso
  repara un hueco declarado-vs-hecho: el testigo se CREIA el
  `epochDigest` y ahora lo RECOMPONE — lo unico que hace legitimo
  anclar una pareja que la firma cubre.
  ⚠️ **Esto NO cierra el eslabon 3**: la cofirma —clave XMSS PROPIA
  del testigo, guardian de indice, dominio nuevo— sigue abierta, y con
  ella la pregunta de arriba, que este tramo NO responde.
  ✅ **Y DEMOSTRADO EN VIVO (§295).** `tools/banco_consistencia.sh`:
  la extension NO trivial juzgada por el testigo (deT distinto de aT,
  camino no vacio), el diario v2 REVERIFICADO sin el nodo (--auditar:
  el criterio de §248, ejecutable), la adulteracion DETENIDA
  (no-extiende, exit 1, via proxy adulterador embebido) y el reseteo
  anotado SIN quemar al testigo (por-detras, exit 0 — la decision D1,
  en vivo). El limite que §294 declaro pendiente ya no lo es.
  ✅ **TRAMO (ii) CUMPLIDO (§297-§300) — el testigo COFIRMA.** El objeto
  lo definio §297 en `zk-ssl-verify`: dominio propio, preambulo con la
  clave del OPERADOR dentro —sin ella la cofirma seria TRANSFERIBLE— y
  `verificar_cofirma` para terceros, **verificable antes de que existiera
  ninguna**. §298 mudo la lectura del indice al guardian para que el testigo
  no reimplementara el invariante. §299 le dio clave XMSS propia y
  `cofirmar()`: reservar con fsync → firmar → AUTOVERIFICARSE. Y §300 el
  mando: **cofirma `Nueva` ∧ `Extiende` y NADA MAS**, a su propio fichero,
  **con la clave que ANCLO** —sin ancla la cofirma es imposible, falla
  cerrada—. Al arrancar reconcilia, y **si la clave va por delante del
  contador NO ARRANCA**.
  ⚠️ **Falta el BANCO VIVO (§301).** Y **esto NO cierra el eslabon 3**:
  queda el (iii) —recoleccion y politica k del CLIENTE— entero.
  ✅ **Y DEMOSTRADO EN VIVO (§301).** `tools/banco_cofirma.sh`: cinco
  tramos en verde contra un nodo real. **El banco que la linea de arriba
  echaba en falta YA ESTA** — queda dicho aqui en vez de borrarla (§247).
  POSITIVO: cofirmas emitidas y verificadas **solo con el fichero**, por
  `--verificar-cofirmas`, el mando del TERCERO que este mismo sello trajo
  —sin el, «la linea basta por si sola» era una propiedad que **nadie podia
  ejercer**—. CONTRATO: las marcas del diario y las lineas del fichero
  cuadran, mismos indices y ninguno repetido. Y **la REGLA, en vivo**: tras
  resetear el nodo debajo del testigo, **nueve vueltas con anomalia y
  NINGUNA cofirmada**, mientras dos legitimas si lo fueron. Mas dos rojos:
  una firma con un nibble tocado (`no-verifica`) y un indice repetido
  visto DESDE FUERA (`indice-repetido`).
  ⚠️ **Sigue sin cerrar el eslabon 3**: el (iii) esta entero.
  ✅ **DOS DE LAS TRES PIEZAS DEL (iii), HECHAS (§319).** El mando del
  tercero gana `--testigos` y `--k`: la politica que NOMBRA que claves de
  testigo valen, y la k que falla cerrada, sobre el fichero que ya existia.
  **Queda la RECOLECCION**: hoy el unico que pide `zkssl_cosigs` es el banco,
  en shell, y ningun `.rs` del cli lo nombra. Las dos lineas de arriba que dan
  el (iii) por ENTERO quedan corregidas aqui en vez de borrarlas (§247).
  ✅ **EL (iii) CERRADO, Y CON EL EL ESLABON 3 (§320) — la RECOLECCION.**
  El mando del tercero gana `--recolectar-cofirmas`: pide `zkssl_cosigs` al
  nodo, **LEE `error`** —la regla que §316 dejo para el codigo que nace hoy—,
  abre el sobre TIPADO en `CofirmaDto` y **rehace cada linea llamando a
  `linea_de_cofirma`**, el unico escritor de ese formato. Lo que sale es byte
  a byte la linea que habria escrito un testigo local, y el test que §315
  reservo —el del CONJUNTO de claves, «donde esten los dos productores»— por
  fin tiene casa. Cinco tests nuevos.
  ⚠️ **El DTO y la linea llevan los mismos ocho campos pero NO la misma
  convencion**: en el cable `v` y `vistoUnix` viajan como QUANTITY en hex y
  en el fichero del testigo van como numeros crudos. Volcar el payload del
  nodo tal cual muere en `v` —`as_u64()` sobre `"0x1"` da `None`— y el resto
  ni se mira. La frontera se cruza **por la serializacion declarada**, no
  leyendo el DTO por dentro, y se cruza A LA VISTA.
  ⚠️ **El gate de version no es adorno**: `linea_de_cofirma` estampa
  `COFIRMA_VERSION` **sin preguntar**, asi que recomponer sin mirar antes lo
  que el cable declara convertiria una cofirma de version DESCONOCIDA en una
  linea que dice ser nuestra, y el lector la aceptaria — **blanqueo de
  version**. Se mira `v` primero y se RECHAZA.
  ⚠️ **La criba va por la LINEA ENTERA, nunca por `(testigo, indice)`**:
  cribar por la clave borraria justo la evidencia que §310 existe para cazar.
  Sin ella, recolectar dos veces **fabricaba una acusacion de doble firma**.
  ⚠️ Lo recolectado **NO esta verificado** —el nodo «no verifica, solo
  filtra»—: comprueba `--verificar-cofirmas` y decide `--testigos`/`--k`. Y
  el nodo guarda **una sola epoca** (§317), asi que acumular entre epocas es
  trabajo del mando, corriendolo mas de una vez. Declarado y no hecho: pedir
  una epoca **por su nombre**, que el nodo ya sabe servir.
  **La linea de arriba que decia «Queda la RECOLECCION» queda corregida aqui
  en vez de borrarla (§247)**, y con ella las de §301 y §319 que daban el
  (iii) por abierto.

  **Eslabón 4 — El hash del verificador dentro de la cabeza
  atestiguada.** Ya está diseñado en `doc/preprints/ERRATA.md`, entrada
  1, y recogido en la tabla de §4.1 de la cuarta revisión del preprint:
  **reemplazar el verificador es hoy ilimitado y sin rastro**, porque
  ordenar, censurar y observar actúan *dentro* de las reglas, mientras
  que cambiar el verificador **redefine las reglas** bajo las que se
  juzga todo lo demás — y el sistema no tiene noción de «reglas en
  vigor». Mientras siga abierto, **toda otra propiedad lleva un
  asterisco**. Encaja con el eslabón 3: un testigo que cofirma está
  atestiguando también **qué reglas regían**.
  ⚠️ **LA RAZON DE ESTE PARRAFO ESTA CORREGIDA DESDE EL §246 (§321).**
  La condicion que da arriba —que falta «nocion de reglas en vigor»— **es
  casi circular**: `hash_verificador_vigente` **ES** el mecanismo para tener
  esa nocion, asi que la condicion se pide a si misma.
  ⚠️ **La razon real: el AIR es CODIGO, no datos.** Lo que el campo debe
  delatar es que es una transicion valida, y eso lo define el AIR. Lo unico
  hasheable en ejecucion son las `ProofOptions`, y **un operador puede cambiar
  el AIR dejandolas identicas**: el campo no seria vacio, seria **CIEGO** — y
  un campo ciego pasa desapercibido mintiendo justo sobre lo que existe para
  detectar. La correccion entera vive en la **entrada 54** y en
  `crates/zk-ssl/src/log.rs`, dentro del propio `EpochHead`.
  ⚠️ **Las dos salidas estan cerradas por razones ajenas al protocolo**:
  **hashear el fuente al compilar** no prueba que el binario se construyera de
  ese fuente —hace falta **compilacion reproducible**—, y **el AIR como
  datos** es la **entrada 55**, parada porque una especificacion escrita por
  quien escribio el circuito hereda sus puntos ciegos y debe escribirse **con
  la auditoria, no antes**.
  ⚠️ **Por tanto el eslabon 4 NO es un sello: es una decision de arquitectura
  con dos precondiciones que viven fuera del protocolo.** Las lineas de arriba
  quedan corregidas aqui en vez de borrarlas (§247).
  ⚠️ §823: **H6 da al eslabón 4 una precondición, no su cierre.** La vía de la
  compilación reproducible tiene ya su medida: el binario sale idéntico dentro
  de una misma máquina, y entre máquinas no está demostrado (la 88). Lo que no
  cambia: el campo no está en la cabeza, y una huella reproducible no prueba
  que el operador corra ese binario (la 54).

  **Cómo se cierra**: no se cierra de una vez. Cada eslabón es un sello
  o más, en el orden de arriba, y **ninguno se monta sin su terreno
  medido**. El eslabón 1 es el más barato y el que más paga en el paper;
  el 3 es el que de verdad cambia el modelo de confianza.

- [ ] **84. El ciclo de vida de la clave XMSS: sin respuesta escrita.**
  Las firmas con estado tienen **presupuesto finito** y **reutilizar el
  índice filtra la clave** (§110.2) — por eso existe el guardián del
  índice (§56). Lo que no existe es qué pasa **después**. Tres huecos,
  y ninguno es teórico para un sistema de liquidación:
  - **Agotamiento**: XMSS^MT 40/8 tiene un número finito de firmas. Qué
    ocurre cuando el árbol se acaba no está escrito en ningún sitio.
  - **Rotación**: hoy no hay una **transición firmada por la clave
    anterior**. Sin ella, rotar es **indistinguible de un robo de
    clave**, y el TOFU del testigo se detendría — con razón. El testigo
    fija la clave que ve la primera vez y se para si cambia
    (`witness.rs`, §245): sin transición firmada, una rotación legítima
    rompe a todos los testigos honestos.
  - **Pérdida del estado del índice**: qué hace el nodo si arranca sin
    saber por qué índice iba. Relacionado con la entrada 19 (sin WAL).
  ⚠️ **Un sistema de liquidación que no tiene respuesta a «hoy toca
  cambiar de clave» no está terminado**, por muchas propiedades que
  demuestre. Y toca a la entrada 83: la rotación es precisamente lo que
  una cofirma de umbral tendría que saber manejar.

  **A la 84 (ciclo de vida de la clave) — la curva de la reutilización,
  con números.** El proyecto QRL cuantificó lo que cuesta extraer la
  clave privada a partir de firmas repetidas del mismo índice: con **2**
  firmas, ~2³⁴ hashes; con **3**, ~2²³; con **4**, ~2¹⁸. Y su
  documentación operativa es tajante: un índice OTS expuesto más de una
  vez convierte la dirección en **comprometida**, y hay que mover los
  fondos.
  ⚠️ Eso cambia el estatuto del guardián del índice (§56) en la doc de
  ZK-SSL: hoy se justifica con «reutilizar el índice filtra la clave»
  (§110.2), que es cualitativo. **Con la curva, el guardián deja de ser
  una precaución y pasa a ser lo único que separa una repetición de una
  pérdida total de clave a la cuarta.** Es un dato que se puede **medir
  en el propio árbol** —no hace falta creerse el suyo— y merece un banco.

  ⚠️ **(§636) Lo que hoy pasa al cambiar de clave, medido**: una clave
  nueva con el diario de la vieja NO arranca. `diario::maximo_indice`
  agrega los índices de todas las claves aunque cada línea firmada lleve
  su `publicKey`, y el test `el_maximo_no_mira_de_que_clave_es_cada_linea`
  lo ata. La razón que dio el §594 («el diario no dice de qué clave es cada
  línea») era falsa y queda corregida en el código. Qué debe hacer el nodo
  al rotar es de esta entrada, no de un arreglo.
  ⚠️ **PRECISADO (§638): lo de arriba vale solo con un contador NUEVO.**
  El nodo no distingue una rotación de un reinicio: una clave nueva con el
  contador de la vieja da `ClaveEnCero`, se resincroniza hasta el contador
  y arranca firmando desde donde iba la vieja. Lo ata
  `una_clave_nueva_con_el_contador_de_la_vieja_arranca_y_sigue_su_cuenta`.
  Queda precisado aquí en vez de borrarlo.

  ⚠️ **(§639) La propuesta, escrita: el RFC-0015, PROPUESTO.** Pre-rotación
  como el RFC 8649: cada clave entra con un acta que compromete la huella
  de la siguiente, que vive fría, y la sucesora firma su propia acta. La
  cuenta de índices es del operador y no de la clave; el aviso de
  agotamiento es el del RFC 10033 §3.4; y el índice indeterminado tiene
  procedimiento escrito. Cinco decisiones del autor, ninguna tomada, y
  ninguna etapa construida: sigue abierta.
  ⚠️ (§642) Las cinco, TOMADAS por delegación del autor con el criterio
  del §609, las recomendadas y REVERSIBLES. Ninguna etapa construida.
  ✅ a medias (§643): la E2, el núcleo del acta —su huella, su preámbulo
  y `verificar_acta` en el kit, con tres KAT que la segunda implementación
  reproduce—. QUEDAN el nodo (E3), el testigo (E4), los sobres con su
  banco (E5) y el medio (E6): sigue abierta.
  ✅ a medias (§644): la E3a, el nodo firma su acta génesis y la de
  rotación al arrancar, opt-in con `--siguiente`, y no arranca con una
  clave que nadie comprometió; el aviso de agotamiento exige
  reconocimiento. QUEDAN la E3b (la firma de la vieja y el cable), el
  testigo, los sobres y el medio: sigue abierta.
  ✅ a medias (§645): la E3b-1, la clave que se va firma el acta de su
  sucesora con `--clave-anterior-fichero`, y en el techo el firmante no
  reserva ni firma: el latido deja de quemar un índice por latido.
  QUEDAN el cable (`zkssl_keyActs`, E3b-2), el testigo, los sobres y el
  medio: sigue abierta.
  ✅ a medias (§646): la E3b-2, `zkssl_keyActs` sirve la cadena de actas
  desde la génesis, con el JSON del acta escrito y leído por el kit, que
  es también la línea del diario. QUEDAN el testigo (E4), los sobres con
  su banco (E5) y el medio (E6): sigue abierta.
  ✅ a medias (§647): la E4, el testigo sigue una rotación que la cadena
  explica y se detiene ante la que no, con el solapamiento por su nombre,
  y `--auditar` la rejuzga sin el nodo. QUEDAN los sobres con su banco
  (E5) y el medio (E6): sigue abierta.
  ✅ a medias (§648): la E5a, los sobres del kit que exigen la continuidad
  de un firmante aceptan `actas`, y una cadena hace de dos claves UN
  operador; sin ella, ni un byte cambia. QUEDAN los vectores, el banco y la
  segunda implementación (E5b) y el medio (E6): sigue abierta.
  ✅ a medias (§649): la E5b, `spec/vectors/rotacion/` copiado de
  `tools/banco_rotacion.sh` contra un nodo que rota dos veces, con las dos
  conductas que la cadena delata sembradas con sus claves, y la segunda
  implementación que lee `actas`. QUEDA el medio (E6), tras la E3 del
  RFC-0013: sigue abierta.
  ✅ a medias (§771): lo que la E5b dejó, la rotación en los sobres de
  consumo y de completitud, con vectores del mismo nodo que rota. QUEDA el
  medio (E6): sigue abierta.
  ✅ a medias (§686): la E5c, el sobre del solapamiento —una cabeza fuera
  de los tramos de su clave es detección portable, sin el testigo—, y la
  refutación del conflicto entre dos claves de un operador, escrita. QUEDA
  el medio (E6): sigue abierta.
  ✅ a medias (§687): el testigo arma el sobre del solapamiento desde su
  diario (`--auditar --sobres`), y el banco de la rotación lo demuestra con
  un testigo de verdad. QUEDA el medio (E6): sigue abierta.
  ✅ a medias (§690): el paso 3 de la D-G, `--desde-minimo`, para rotar
  con un índice indeterminado. QUEDA el medio (E6): sigue abierta.
  ✅ a medias (§770): el testigo da el dato del paso 3 de la D-G: `--auditar`
  imprime la hoja embebida más alta de cada clave. QUEDA el medio (E6): sigue
  abierta.

- [x] **85. Una SEGUNDA implementación que pase los vectores: HECHA en cuatro hitos (§623, §626).**
  `tools/segunda/`, en Python desde la spec: 26/26 KAT del núcleo, 376/418 cabezas firmadas con
  XMSS^MT de RFC 8391 (los 42 fallos, negativos), 121/121 entradas del segundo verificador en las
  cinco formas sin STARK, con salida idéntica al binario, y el verificador STARK (`stark.py`,
  `airs.py`, `juez_stark.py`, §626): 23/23 pares de las seis familias con prueba, ocultas y con sal,
  con el veredicto y la causa del binario, y 33/33 falsadores callados. En el canon, a pin cero.
  Queda, y va dicho: las cinco AIR están transcritas del `.rs` (no hay otra fuente), los 34 vectores
  `0.3/` sin ocultar quedan fuera, y el mando de las formas con STARK no se compone aún en el
  segundo código. El cuerpo original, abajo.
  ⚠️ **(§706) La marca de la ocultación, juzgada como el kit.** El juez STARK no exigía la m de la
  casa, 64, tomaba un meta vacío por una prueba sin ocultar y miraba las opciones antes que la
  marca: una prueba de edad honesta con m = 32 en la marca era ROJO en el kit y VERDE aquí, contra
  «fallar igual» (H3). Desde el §706 exige lo mismo, en el mismo orden y con el mismo texto, y lo
  fijan seis negativos de la edad y el pago: 34 de 34 pares.
  ⚠️ **§793: las cifras de arriba son las del cierre.** Decía 26/26 KAT, 376/418 cabezas con «los 42
  fallos» y 121/121 entradas del segundo verificador; los asientos las recontaron al crecer los
  vectores (el §640, 378/424 cabezas y 124 entradas), y esta entrada no las siguió. Remedidas en
  Python sobre el árbol del §789 (`8c02941`), sin compilar nada: 31/31 KAT del núcleo (tres del §643
  y dos del §773); 485/535 cabezas y 10/15 cofirmas en 404 ficheros, con los 55 que no verifican
  negativos según su manifiesto; 174/174 entradas del segundo verificador en sus cinco familias; y
  34/34 pares STARK con 33/33 falsadores; la cofirma con `versionFormato` `0x103` del §780, entre
  los negativos, con el texto del kit en los dos lectores. La «salida
  idéntica al binario» y «la causa del binario» no se han vuelto a comparar: piden el binario
  compilado. El reparto, en `tools/segunda/README.md`.
  ⚠️ **§810: y el medio.** Desde el §810 el segundo verificador lee también el sobre `ancla-cofirmada`
  (RFC-0013 E4a), con ML-DSA-44 escrito desde FIPS 204 (`mldsa44.py`), la nota checkpoint y la
  inclusión de RFC 9162 (`medio.py`) y su juez en el canon (`juez_medio.py`, 87/87): 201/201
  entradas en seis familias. En las 27 de la sexta, la salida se comparó con el binario compilado:
  la misma, byte a byte, en los 5 positivos, y la misma línea del ROJO en los 22 negativos.
  ⚠️ §859: los «87/87» son del §810; desde el §859 el juez del medio da 92/92, con cinco casos que
  matan los cinco mutantes que vivían del §810 (`tools/segunda/README.md`).

- [ ] **85 (original). Una SEGUNDA implementación que pase los vectores.** Lo caro
  ya está hecho: vectores de conformidad versionados (`0.2` idéntico,
  `0.1` rechazado), OpenRPC **generado** desde la tabla de
  `zk-ssl-wire`, y regeneración byte-exacta como compuerta permanente.
  Lo que falta es el uso para el que existen: **que otro código los
  reproduzca**.
  ⚠️ **Una spec que sólo implementa un código no es una spec, es
  documentación**; y una conformidad que sólo se comprueba contra sí
  misma es **autoconformidad**. Mientras no exista una segunda
  implementación, la afirmación «esto es un protocolo» es una promesa.
  **Alcance mínimo suficiente**: no hace falta reimplementar la capa.
  Basta un **verificador** en otro lenguaje que reproduzca el escenario
  canónico y recompute las cabezas — el día que pase los vectores, la
  conformidad deja de ser autoconformidad. Está en la Fase 1 del
  `ROADMAP-ECOSISTEMA.md` («para que existan SEGUNDAS
  implementaciones»); esta entrada le pone el listón medible.

- [ ] **86. Anclar las cabezas de época en un log de transparencia
  externo.** Sin L1, el talón sigue siendo el **tiempo**: los límites de
  época salen del **diario del propio nodo** (autoreporte, declarado en
  §4.8 del preprint). Publicar el digest de cabeza en un log público de
  transparencia da **frescura y no-equivocación verificables por un
  tercero**, barato, **sin consenso y sin cadena**.
  Es la escotilla que ZK-SSL **sí** puede tener: no requiere contrato,
  no requiere L1, no requiere comité de disponibilidad de datos. Y
  compone con la entrada 83: un ancla externa acota la ventana del
  primer encuentro que el TOFU del testigo deja abierta —hoy declarada
  como limitación **del modelo**, no de la implementación
  (`witness.rs`, cabecera).
  ⚠️ **Por medir antes de decidir nada**: qué logs aceptan digests
  arbitrarios, con qué cadencia, y qué coste tiene por época. Y si el
  ancla externa se convierte en dependencia de liveness, va declarada
  como tal.
  ⚠️ (§590, sesión 194): el documento del §174 llegó al árbol publicado
  como marcador de posición (S548) y se reconstruyó en el §590.
  ✅ a medias (§591–§593): B10.6 y B10.7 construidos —el formato del
  ancla, su huella y el sobre `tipo: "ancla"` del RFC-0012, con la
  vista dividida dentro—. QUEDA lo que esta entrada pide medir (qué
  logs, cadencia, coste) y el despliegue: la entrada sigue abierta.

  ⚠️ **RFC-0013 (§603) elige el medio en diseño**: un log de checkpoints C2SP
  —nota `tlog-checkpoint` firmada con ML-DSA-44 tipo `0x06`, árbol SHA-256 de
  anclas, testigos ajenos por `tlog-witness`— y deja el despliegue y su medida
  como etapas E2 a E4, pendientes. Sigue abierta por eso.
  ✅ a medias (§631): la E2a, el árbol SHA-256 de anclas, construido en
  `crates/zk-ssl-medio` y contrastado con los vectores del IETF y las 685
  sondas de `transparency-dev/merkle`. QUEDAN la nota (E2b), el publicador
  (E3) y el sobre del kit (E4): sigue abierta.
  ✅ a medias (§632): la E2b, la nota `checkpoint` firmada con ML-DSA-44
  tipo `0x06`, construida y contrastada con `filippo.io/torchwood` sobre el
  ML-DSA de Go: la línea del publicador sale byte a byte. QUEDAN el
  publicador (E3), con el despliegue y su medida (qué testigos, cadencia,
  coste), y el sobre del kit (E4): sigue abierta.
  ✅ a medias (§633): la E4a, el sobre `ancla-cofirmada` en el kit, que
  reporta las cofirmas y no juzga el umbral (decisión del autor). QUEDAN
  su catálogo y su banco (E4b), el publicador (E3) y el juez del umbral
  en `zk-ssl-cli`: sigue abierta.
  ✅ a medias (§634): la E4b, el catálogo `spec/vectors/ancla-cofirmada/`
  y su banco contra un nodo real. QUEDAN el publicador (E3), con el
  despliegue y su medida (qué testigos, cadencia, coste), y el juez del
  umbral en `zk-ssl-cli`: sigue abierta.
  ✅ a medias (§635): el juez del umbral, `zk-ssl-cli witness
  --ancla-cofirmada --testigos-medio --k-medio`, con la política del
  cliente. QUEDA el publicador (E3), con el despliegue y su medida:
  sigue abierta.
  ✅ a medias (§808): la E3a, SOLO EN LOCAL: el cliente `add-checkpoint`
  de `tlog-witness`, la retención de las cofirmas `0x06` y la nota
  cofirmada (`zk_ssl_medio::testigos`), medidos contra litewitness
  v0.10.0 y su motor con ML-DSA-44 en `127.0.0.1`. QUEDA lo que pide
  testigos reales —cuáles, con TLS, la cadencia, servir la nota y medir
  el coste—, y una pregunta medida para el autor: litewitness cofirma
  con Ed25519, que la casa no verifica (RFC-0013 D-H): sigue abierta.
  ✅ decidido (§836): la D-H del RFC-0013, por el autor. Solo `0x06` en
  las dos puntas, sin puente ni cofirmas `0x04`. El alta en una red de
  testigos, cuando estén el publicador entero, testigos que acepten el
  log `0x06` y al menos dos con clave ML-DSA-44, y la dé el autor. QUEDA
  lo mismo, y el contraste local de E3 con `omniwitness` y una clave
  ML-DSA-44, leído y no corrido: sigue abierta.
  ✅ a medias (§848): el contraste local de E3a con `omniwitness`, corrido.
  `transparency-dev/witness` en `5397155` (la punta de su `main` el
  2026-10-15), sin tocar, en `127.0.0.1`, con el log `0x06` del medio
  configurado a mano (`--additional_logs`): uno con una clave ML-DSA-44 y
  otro con una ML-DSA-44 y una Ed25519 del mismo nombre. El cliente del
  §808, sin cambios, dice en los diez pasos lo que `tlog-witness` (`b2dd989`,
  el mismo texto que el `main` de C2SP el 2026-10-21) pide, y el kit da
  VERDE la nota cofirmada por los dos. Lo graba
  `crates/zk-ssl-medio/tests/vectores/testigos/rehacer_omniwitness.sh`,
  fuera del canon, y `tests/testigos_omniwitness.rs` lo relee en cada
  canon. QUEDA lo que pide testigos reales, y el alta, que la da el autor
  (§836): sigue abierta.
- [ ] **87. Agilidad criptográfica: el ESQUEMA DE FIRMA no está
  versionado.** La cabeza tiene byte de versión (§275) y el registro
  tiene dos eras (§281). **El esquema de firma no tiene ninguna de las
  dos cosas.** Y la base sobre la que descansa está declarada como
  frágil por el propio árbol: `xmss = "=0.1.0-pre.0"`, **sin auditoría
  independiente**, fijada con `=` porque es una pre-release y `master`
  ya diverge del tag (`sha3` → `shake`), más el apaño del OID
  multiárbol (§240) y su centinela.
  **El hueco**: si esa dependencia muere, si aparece un ataque, o si
  simplemente hay que pasar a otra primitiva, **no hay camino
  declarado**. Toda la cadena de custodia —las cabezas que los
  titulares guardan— quedaría atada a un esquema que ya no se puede
  emitir.
  **Qué lo cierra**: un identificador de esquema **dentro de lo que se
  firma**, y la regla escrita de cómo una cabeza firmada con el
  esquema A sigue verificando cuando el nodo ya emite con el B. Es
  exactamente la forma de §275 —el byte de versión gobierna al
  recompositor— aplicada a la firma en vez de al formato.
  ⚠️ Compone con la **84**: cambiar de algoritmo **es** una rotación,
  con un paso más. Si no hay transición firmada por la clave anterior,
  cambiar de esquema es indistinguible de un compromiso de clave.

  **A la 87 (agilidad criptográfica) — el precedente, y un destino con
  nombre.** QRL declara que su modelo de direcciones 2.0 es
  **intrínsecamente cripto-ágil**, y que SLH-DSA y otros algoritmos
  futuros pueden integrarse una vez viva la red; su repositorio ya decía
  que XMSS/W-OTS+ son nativos **con soporte extensible a más esquemas
  incorporado**.
  ⚠️ **Aquí no estamos por delante: estamos por detrás.** Ellos lo
  diseñaron desde el génesis; ZK-SSL no tiene identificador de esquema en
  lo que firma. La 87 gana con esto **un objetivo concreto** —un
  sucesor con nombre, sin estado, ya estandarizado— en vez de un
  «identificador de esquema» abstracto.

  **Adenda — ML-DSA (CRYSTALS-Dilithium) como candidato.** No abre
  entrada propia: es **el caso de prueba** que justifica esta — sin
  esquema versionado, la discusion no se puede tener; con la 87 hecha,
  se vuelve una decision medible. ⚠️ Cifras por verificar contra FIPS
  204; aqui son ordenes de magnitud:

  | | XMSS^MT 40/8 (hoy) | SLH-DSA | ML-DSA |
  |---|---|---|---|
  | Supuesto | solo el hash | solo el hash | reticulos (Module-LWE/SIS) |
  | Estado | **con estado** | sin estado | sin estado |
  | Firma | 18.469 B (medido) | ~~decenas de KB~~ **7.856 B** (128s) · 16.224 B (192s) · 29.792 B (256s), FIPS 205 | ~2,4-4,6 KB |
  | Firmar | 144,5-160,5 ms (medido) | lento | sub-ms a pocos ms |

  ⚠️ **Corrección (30-09-2026, `doc/firma-corta-evaluacion.md` §3)**: la fila «Firma»
  daba «decenas de KB» a SLH-DSA. FIPS 205, Tabla 2, leída en espejo: **7.856 B en
  SHA2-128s y 16.224 B en 192s, las dos por debajo de los 18.469 B medidos de la
  XMSS^MT 40/8 de hoy**; solo 256s (29.792 B) es mayor. El precio de «sin estado»
  que esta entrada daba por sabido hay que **re-medirlo, tiempo de firma incluido**,
  antes de decidir: es lo que la 108 deja como pendiente.

  ⚠️⚠️ **Lo decisivo no es el tamaño: sin estado, las entradas 84 y 92
  casi desaparecen** —sin indice no hay guardian, ni agotamiento, ni la
  ambiguedad rotacion/robo, ni estado que sobrevivir a un reinicio— y
  **se desbloquea la pregunta abierta del eslabon 3 de la 83**: la
  cofirma deja de ser coordinacion de indices entre testigos. **Lo que
  se perderia es el corazon del proyecto**: XMSS descansa solo en el
  hash; ML-DSA en reticulos, mas recientes y menos curtidos — moverse
  en el eje supuestos-operabilidad, no mejorar en todo, y un proyecto
  cuya tesis es «demuestra y declara» no hace ese cambio en silencio.
  **Hipotesis que merece medirse (razonada, NO medida): las DOS** —
  hash-based para la raiz de larga vida (una firma por epoca, 160 ms no
  molestan, supuestos minimos) y sin estado para lo frecuente y
  coordinado (las cofirmas de la 83). **Como se decide, y NO es en una
  conversacion**: (1) la 87 primero; (2) un banco con las tres
  primitivas sobre el hardware real, metodo de §263; (3) la decision
  escrita con su procedencia y su precio, en el paper.
  ⚠️ §790: propuesto en el RFC-0020 (`spec/rfc/0020-el-esquema-dentro-de-lo-que-se-firma.md`);
  adoptarlo es del autor. Mide dónde entra hoy un esquema en lo que se firma —sólo en el acta,
  que es opt-in y no cubre al testigo; ni la cabeza ni la cofirma lo llevan— y propone el
  `esquema` del acta, un `u64`, en un preámbulo v7 de la cabeza y de la cofirma, la regla con que
  una cabeza del esquema A sigue verificando cuando el nodo firma con el B, y su coste en bytes y
  en vectores, para el corte `zkssl/0.5`. Siete decisiones del autor, ninguna tomada, y ninguna
  etapa construida: sigue abierta.
  ⚠️ §799: el autor aceptó el RFC-0020 con sus siete decisiones, en cada una la que el RFC
  recomienda: el identificador en el preámbulo v7 de la cabeza y de la cofirma, el `u64` del acta,
  la cofirma con el esquema del operador, la regla del registro para la sucesora, la salida de un
  esquema sólo si se rompe, el registro en `spec/NUCLEO.md` §6, y el viaje en el corte
  `zkssl/0.5`, en el mismo sello que el corte, con su E2 y su E3 antes. Lo de arriba, «adoptarlo
  es del autor» y «ninguna tomada», valía hasta aquí. El RFC sigue PROPUESTO y ninguna etapa está
  construida: sigue abierta hasta que se construyan.
  ⚠️ §800: la E2 del RFC-0020 está construida —el registro de esquemas en `spec/NUCLEO.md` §6, con
  su productor `Esquema` en `zk-ssl-verify`, y los preámbulos v7 de la cabeza y de la cofirma, con
  tres KAT que `tools/segunda` reproduce—, y apagada: ni el nodo, ni el testigo, ni el kit los
  firman ni los aceptan. Lo de arriba, «ninguna etapa está construida», valía hasta aquí. Sigue
  abierta: lo que la cierra, el esquema dentro de lo que se firma, llega con la E4, en el corte.

- [ ] **88. Sucesión: qué pasa el día que el autor pare.** El proyecto
  lo lleva **una persona** con asistencia de IA, con una disciplina que
  está registrada sello a sello. Eso es una fortaleza y **el mayor
  riesgo de continuidad que tiene**. No hay mantenimiento declarado, no
  hay segundo par de manos, y el factor bus es uno.
  ⚠️ Para un artefacto cuyo objetivo es **tener segundas
  implementaciones** (entrada 85), la sucesión no es administración:
  **es una propiedad del artefacto**. Una spec que sólo su autor sabe
  interpretar no es una spec.
  **Qué lo cierra, en orden de coste**: declarar el mantenimiento y el
  criterio de aceptación de un RFC —el proceso ya existe en
  `spec/rfc/`, le falta quién decide y con qué regla—; hacer las
  **construcciones reproducibles con huellas publicadas** (anexo), para
  que un extraño pueda comprobar que el binario es el fuente; y dejar
  escrito el «cómo se trabaja aquí» fuera de los traspasos de sesión,
  que hoy son el único sitio donde vive el método.
  ⚠️ §793: **la tercera pieza, escrita.** `doc/METODO.md` cuenta el método
  fuera de los traspasos —el sello y su asiento, cómo se corrige un
  documento vivo, las convenciones de esta lista, los RFC y sus etapas, los
  niveles del canon y los bancos, y el paso 4 de `GENAI.md`—, cada cosa con
  la cita al fichero que la establece, y dice cuándo algo es costumbre y
  ningún fichero lo manda; `CONTRIBUTING.md` lo enlaza. No decide quién
  mantiene ni con qué regla se acepta un RFC, y lo dice: esa pieza, la
  primera, es del autor. La segunda, las construcciones reproducibles, este
  sello no la mira. Lo escribió una sesión y su aceptación es del autor.
  Sigue abierta.
  ⚠️ §823: **la segunda pieza, medida y no hecha (H6).** En una misma máquina,
  el nodo y el testigo de `b6994c3`, compilados con los flags de
  `tools/artefacto.sh` desde dos árboles, en otro `target` y con otro `HOME` y
  otro `CARGO_HOME`, salen idénticos byte a byte las cuatro veces (`rustc`
  1.94.1; el nodo `ac27f9052c7f74c1…`, con `dev` y compilado junto al testigo,
  y el testigo `07d659c6291ad68f…`); sin el remap salen iguales entre sí, pero
  con otra huella y con las rutas de `$HOME` dentro. Lo que no sale desde otra
  máquina es un kit publicado: la 0.4.2 rehecha desde `a4e888d` con su `rustc`
  difiere en 13 de sus 35 secciones, y en 3 imitando su `rust-src`; la 0.4.3,
  la vigente desde el §818, no consta rehecha con su `rustc` 1.97.1 fuera de la
  máquina que la produjo —con el 1.94.1 sale con otra huella—, y lleva las
  mismas once rutas de `rust-src` y la `.comment` del mismo GCC. Las dos causas
  medidas —las rutas de `rust-src`, que ninguna regla de `b6994c3` lleva a
  `/rustc/<commit-hash>`, y el código nativo que compila el host, blake3 en los
  tres binarios y ring en el testigo, con la `.comment` que nombra su GCC—, una
  dependencia que lo medido no toca —la regla del registry, que no
  gana nunca con `CARGO_HOME=$HOME/.cargo` y por eso da los mismos bytes en las
  dos máquinas (el publicado y la rehecha llevan las mismas 41 rutas
  `/home/x/.cargo/registry/src/…`), pero con un `CARGO_HOME` fuera de `HOME` no
  casaría y dejaría entrar sus rutas enteras (deducido, sin medir; ⚠️ §824: medido después, abajo), porque el
  script de `b6994c3` deriva sus reglas de `$HOME`— y lo medido están en
  `doc/KIT.md` §«Reproducir el kit, para quien no se fíe de la descarga». En
  `b6994c3` ninguna herramienta del árbol compila el nodo ni el testigo así, y
  no hay huellas publicadas de ninguno de los dos. Sigue abierta.
  ⚠️ §824: **la herramienta.** `bash tools/artefacto.sh --binarios` compila el
  nodo de producción, sin `dev`, y el testigo con las reglas del kit en dos
  raíces, y exige la misma huella byte a byte; el canon lo corre desde
  `--largo` (`doc/KIT.md` §«El nodo y el testigo, desde dos raíces»). El
  `--check` del kit compila ya en dos raíces, y las reglas cubren `rust-src` y
  un `CARGO_HOME` fuera de `HOME`; `cargo` corre con `env -i`, cada `target`
  lleva el sello del host con que se compiló, y `VERSION` nombra el `cc`, el
  `as`, la glibc y los objetos de arranque del host (`doc/KIT.md`
  §«Reproducir el kit, para quien no se fíe de la descarga»). Lo deducido de
  arriba, medido en el kit con la herramienta de `b6994c3` y un `target`
  vacío: un `CARGO_HOME` fuera de `HOME` deja dentro 41 rutas suyas, enteras,
  y `rust-src` en un `RUSTUP_HOME` fuera de él, once; la búsqueda de su
  `--check`, que sólo mira `$HOME`, no las habría visto. Lo que la cierra no
  cambia: no hay huellas publicadas del nodo ni del testigo, que publicarlas
  es del autor, y entre máquinas no está demostrado. El SDK es una
  biblioteca: no hay un binario suyo que reproducir. Sigue abierta.

- [ ] **89. El proyecto sólo se ha auditado a SÍ MISMO.** `AUDITORIA.md`
  tiene más de 21.000 líneas y registra los errores del propio método
  en vez de ocultarlos. Eso es evidencia de **disciplina**, no de
  **corrección verificada por un tercero**. Nadie de fuera ha mirado
  esto nunca.
  ⚠️ Y es el punto ciego que el propio estándar del paper implica: **un
  calendario de confianza residual auditado sólo por su autor tiene una
  entrada que no puede ver**. Es la misma asimetría de §267 —la
  verificación va donde ya está la sospecha— aplicada al proyecto
  entero: el autor no sospecha donde no mira.
  **Alcance mínimo suficiente**, para que no sea una aspiración: una
  revisión externa **dirigida** a los dos sitios donde una rotura es
  fatal —la ruta de firma con el guardián del índice, y la
  recomposición de la cabeza de época—. No hace falta auditar 21.000
  líneas para que deje de ser autoauditoría.

  **A la 89 (auditoría externa) — deja de ser una aspiración y pasa a ser
  la norma del género.** QRL está auditado por **dos** firmas externas
  (red4sec y x41 D-sec) y lo publica como parte de su descripción. En
  este ámbito, una implementación de referencia sin mirada de fuera es la
  excepción, no la regla. **Ellos lo tienen; nosotros no.**

- [ ] **90. La política de compatibilidad, escrita como PROMESA y no
  como práctica.** Hoy la práctica es buena: una cabeza v1 sigue
  verificando bajo binario v2, el registro lee sus dos eras, el vector
  anterior se rechaza **a propósito**, y la 82 dejó el corolario de que
  un formato se rompe **una vez para varias razones juntas**. Nada de
  eso está prometido a nadie: está **hecho**.
  ⚠️ La diferencia importa para quien construya encima. **Un protocolo
  es su promesa, no su historial**: qué superficie es estable, qué
  puede romperse, con cuánto aviso, y qué se garantiza que seguirá
  verificando después de una rotura. Sin eso, una segunda
  implementación (85) no tiene contra qué comprometerse, y un titular
  no sabe cuánto dura la validez de lo que custodia.
  **Qué lo cierra**: una sección normativa —al lado de la de
  `spec/RPC.md`, que ya lo es— con la superficie estable enumerada y la
  regla de ruptura escrita. Y una compuerta que la haga cierta: lo
  declarado estable no cambia sin que el canon lo diga.
  **Declarado (§699)**: la práctica tampoco fue siempre buena. `zkssl-0.2.json` se re-emitió tres
  veces bajo su versión (§275, §278 y §281), y `zkssl/0.4` no identifica qué pruebas acepta un
  nodo: el AIR de las que viajan cambió sin subirla en el §641, el §680 y el §684, y medido en el
  §699, un nodo no acepta el envío de un cliente si los dos dicen `zkssl/0.4` y entre ellos queda
  el §641 o el §680. Lo dicen `spec/RPC.md` («Versionado»), el OpenRPC, `spec/README.md`, los dos
  README y el RFC-0017; la promesa, y cómo se identifica una prueba, siguen abiertas.
  ⚠️ §790: propuesto en el RFC-0021 (`spec/rfc/0021-la-politica-de-compatibilidad.md`); adoptarlo
  es del autor. Enumera seis superficies estables, escribe la regla de ruptura, el aviso y lo que
  sigue verificando después, desde lo medido en el §699; propone la enmienda a «Versionado» de
  `spec/RPC.md`, que ya existe en «Notas operativas»; y deja la compuerta como etapa, sin
  escribir. Ocho decisiones del autor, ninguna tomada: sigue abierta.
  ⚠️ §799: el autor aceptó el RFC-0021 con sus ocho decisiones, en cada una la que el RFC
  recomienda: la promesa en `spec/RPC.md`; las seis superficies de su D-A, con el diario del
  testigo y `zk-ssl-verify` como biblioteca fuera de la promesa, como allí se enumera; emitir una
  versión de cabeza nueva rompe el cable; el aviso, un sello con el RFC PROPUESTO antes del corte;
  la línea `0.N`: dentro de `0.N.x` nada de la superficie se rompe salvo por R6 f, y entre `0.N` y
  `0.N+1` todo puede romperse con R5 y R6, mientras el protocolo esté por debajo de 1.0; un arreglo
  de seguridad puede cambiar un veredicto dentro de `0.N.x`, con su aviso; un kit publicado sigue
  publicado siempre (R6 a); y la E3b, con el corte. Lo de arriba, «adoptarlo es del autor»
  y «ninguna tomada», valía hasta aquí. El RFC sigue PROPUESTO y ninguna etapa está construida:
  sigue abierta hasta que se construyan.
  ⚠️ §801: la E2 y la E3a, construidas. La sección «Versionado y compatibilidad» cierra
  `spec/RPC.md`, con el texto de la D-C, lo que `zkssl/0.4` no identifica todavía y la lista de la
  superficie de hoy, y la remiten `spec/PAQUETE.md` §6, `spec/NUCLEO.md` §3, la sección 0 de los
  dos manuales del kit y la fila del protocolo de los dos README; `tools/check_superficie.py`,
  nombre provisional que confirma el autor, entra en el bucle del canon y compara esa lista con lo
  que da cada productor. Lo de arriba, «ninguna etapa está construida», valía hasta
  aquí. Sigue abierta: la E3b y la E4 llegan con el corte.
  ⚠️ §863: el vector del §802, construido, por delegación del autor («resuelvelas aplicando los
  principios»), con el criterio del §609 y el precedente del §642; REVERSIBLE; entra con su
  aceptación. El §802 cambió un veredicto del cable dentro de `zkssl/0.4` —`zkssl_openAccount`
  rechaza con `IdentityUnavailable` lo que antes abría—, un arreglo de seguridad, y la R6 f pide que
  entre con los vectores que lo muestran, con rutas nuevas: ninguno fijaba un error de
  `zkssl_openAccount`. Ahora `spec/vectors/nodo/abrir-identidad-con-cuenta.json`, con su línea de
  manifiesto y su huella, dice lo que responde un nodo conforme sobre un libro recién constituido:
  la identidad abre, la misma con la vista y la sal de otra clave da `-32000` con la causa
  `IdentityUnavailable`, sin campos y juzgado en el `seq` 1, y la cuenta de cuentas sigue en 1. No
  fija el censo `nextIndex`, la raíz ni el registro, que por el cable sólo se leen en
  `zkssl_epochHead` entero y en las entradas del registro: los ata la capa (`tests_una_identidad`,
  en `crates/zk-ssl/src/accounts.rs`). Es aditivo (R2): una ruta nueva en una familia nueva, la del
  NODO, cuyo manifiesto no es el contrato del mando; lo juzga en cada canon un test del nodo que
  manda cada petición por la función del transporte y compara, y el canon exige por su nombre que
  pase. La lista de la superficie lo cuenta, como el §809. De lo que pide la R6 f, el §802 tenía el
  aviso (`SECURITY.md` §3.13), no lleva release porque el kit no cambia, y le faltaba el vector,
  que ya tiene. Sigue abierta: la E3b y la E4 llegan con el corte, que decide también qué es de esta
  familia, que ningún kit juzga: si se muda a `spec/vectors/<versión>/` (R6 b), allí no hay un
  veredicto del kit que conservar ni, todavía, un consumidor.

- [x] **91. El fin de vida: cómo se apaga esto sin dejar a nadie
  dentro.** Toda capa de liquidación termina — por cierre, por
  migración o por abandono. Hoy **no hay apagado declarado**: qué debe
  publicar el operador al cerrar, qué se lleva el titular, y con qué
  sostiene su posición después.
  ✅ **Y aquí está lo bueno: el mecanismo ya existe y nadie lo ha
  llamado por su nombre.** El **paquete de evidencia portable**
  (entrada 83, eslabón 1) **es** el procedimiento de apagado: cabeza
  firmada custodiada, camino de acuse, compromiso recomputable,
  verificador que compila sin el probador. Falta declararlo como tal y
  ejercitarlo — un banco que apague el nodo y compruebe que una
  posición **sigue siendo demostrable sin él**.
  ⚠️ Es lo contrario del fallo característico de validium, donde el
  titular depende de que alguien le entregue los datos y puede quedar
  congelado indefinidamente. **Un sistema que dice cómo muere merece
  más confianza que uno que presume que no morirá**, y este puede
  decirlo con una demostración en vez de con una promesa.

  ✅ **CERRADO (§290): declarado y demostrado.** La declaracion:
  seccion «Apagado» en `spec/RPC.md`, normativa — que publica el
  operador (nada nuevo), que se lleva el titular (lo que ya custodia),
  con que sostiene la posicion (el paquete v1 y el mando de
  `zk-ssl-verify`). La demostracion: `tools/banco_apagado.sh` — nodo
  real con clave, diario y ledger; posicion fondeada; paquete capturado
  del cable; kill -9; y **la posicion en VERDE sin el nodo**, con el
  negativo de un nibble adulterado en ROJO. El banco queda para
  repetirlo cuando se quiera; fuera del canon, que no levanta procesos.

- [ ] **92. Custodia de la clave y SUPERVIVENCIA del índice: la
  mitigación que QRL tiene y aquí no está escrita.** El guardián del
  índice (§56) impide reutilizar dentro de una ejecución. Lo que no
  hay es respuesta a **dónde vive el índice y qué pasa si se pierde**.
  QRL mitiga exactamente esto con dos cosas que ZK-SSL no tiene: sus
  herramientas de cartera **siguen el estado OTS**, y hay integración
  con hardware (Ledger) para custodiar la clave — y aun así el informe
  externo lo mantiene como **riesgo operativo inherente**, distinto del
  ataque cuántico.
  **Los tres huecos**: (a) el estado del índice debe **sobrevivir a un
  reinicio** —hoy no hay WAL (entrada 19), así que un fallo entre
  operaciones puede dejarlo indeterminado, y un índice indeterminado es
  peor que uno perdido: invita a reutilizar—; (b) la clave del operador
  no tiene custodia declarada —está listado como condición de producto
  **sin sello**—; (c) no hay **procedimiento de emergencia** para
  «creo que se ha reutilizado un índice»: QRL sí lo tiene, y es mover
  los fondos.
  ⚠️ Compone con la **84** (agotamiento y rotación) y con la **19**
  (sin WAL). **Son la misma familia y convendría cortarlas juntas**:
  las tres son el ciclo de vida del firmante.
  ⚠️ (§639) El RFC-0015, la propuesta de la 84, escribe el (c) como
  procedimiento (su D-G) y deja la clave siguiente fría por diseño (D-B);
  el (a) y el (b) siguen como están. Sigue abierta.
  ✅ a medias (§690): el procedimiento del (c) tiene su pieza en el nodo:
  `--desde-minimo` hace entrar a la sucesora por encima de toda hoja que la
  clave que se va pudo firmar, quemando las de en medio, y la clave rotada
  no arranca sobre su diario. El (a) y el (b) siguen como están: sigue
  abierta.
  ✅ (§709) Un hueco de la misma familia que esta entrada no nombraba, el
  SEC-1 de `doc/blueprint-v2.md`: dos procesos sobre el mismo contador. Desde
  el §709 el guardián toma un cerrojo sobre su fichero, y al firmar los
  contadores van con ruta absoluta y sin valor por defecto. Lo que el cerrojo
  no cubre —un sistema de ficheros de red, otra máquina, la misma semilla con
  otro contador— lo dicen `SECURITY.md` §2 y `doc/CONFIANZA_RESIDUAL.md`. El
  (a) y el (b) siguen como están: sigue abierta.
  ✅ a medias (§770): la otra pieza del (c), la del testigo: `--auditar` da,
  por clave, la hoja embebida más alta entre las cabezas que reverifican, el
  número que `--desde-minimo` pide. El (a) y el (b) siguen como están: sigue
  abierta.

- [ ] **115. La conservación, comprobable por un tercero: el camino, o el título.**
  Medido en el §696: la conservación de cada transición la imponen en circuito
  las pruebas de envío, cobro y quema (`C_BALANCE`, `C_SUPPLY`), y el agregado
  (suministro = saldos + en vuelo), la capa al reabrir
  (`crates/zk-ssl/src/persistence.rs`, «CONSERVACION DEL DINERO»); la autoría,
  `C_PK_CHECK`. Las dos las comprueba el nodo, al verificar cada prueba antes de
  aplicarla. Un tercero no: el registro guarda el resumen de la prueba y no la
  prueba, ningún método del cable la sirve, y el kit no compila ninguno de esos
  circuitos, ni los de emisión y reembolso. De la conservación le llega el
  `totalSupply` que firma la cabeza desde la v5; de la autoría, sólo la prenda.
  El §696 sacó la conservación y la autoría de «lo que comprueba un tercero» en
  los siete documentos que lo decían; el título, «pruebas abiertas de
  conservación para libros cerrados», no lo tocó.
  **Lo que pide, y lo decide el autor**: o reformular el título, o
  comprometerse con un camino medido para que la conservación la compruebe un
  tercero sin el nodo. Dos candidatos, sin medir: (a) conservar o publicar las
  pruebas de transición, y que el kit verifique las de envío y cobro contra el
  suministro firmado; (b) comprometer en la cabeza el total en vuelo, con su
  coste de privacidad (el `pending` de `zkssl_supply`, §656). Cualquiera de
  los dos, con su análisis medido antes y su RFC después.

## C. Solidez y verificacion: resueltas y en revision

Los hallazgos de solidez de esta sesion. Tres cerrados, uno aplazado a
proposito, y la auditoria externa que ahora es instrumento y no deseo.

- [x] **27. ⚠️ FALLO GRAVE CONFIRMADO Y CORREGIDO: el cobro no demostraba
  titularidad.**
  `circuit_claim` no ata `COL_R_ID` a `COL_ACC_ID`; la suite existente ya lo
  demostraba (escenario con `SK=0xA11CE` cobrando un pendiente de `0xB0B`,
  y verifica). Quien conozca posicion, aleatorio e importe cobra en su
  cuenta; **el pagador los conoce todos** (§39). ✅ **Corregido** el
  30-07-2026 (§39.1): `C_PEND_IN` reconstruye el compromiso con
  `COL_ACC_ID`, atado a la clave por `C_PK_CHECK`. Escenario y los dos
  tests rehechos. 203 y 174 sin fallos.

- [x] **26. Ocho restricciones sobrescritas en `circuit_send` y
  `circuit_claim`.** Las que imponen que la identidad del receptor y el
  aleatorio no varien entre filas se calculan y se pisan; nada mas las fija
  (§38). Explotabilidad **no establecida**. Pasa por delante de la 6 y la
  25: aquellas son grados que no se comprueban, esta es una restriccion que
  se creia impuesta y no lo esta. **Premisa confirmada** el 30-07-2026
  (§38.1): comentarlas no cambia nada, 201 y 174 sin fallos. Queda abierta
  la consecuencia, que es la entrada 27.

- [x] **31. Pregunta cerrada: `circuit_mint_pending` no declara a quien
  emite** ~~(§40.3)~~ **y tampoco lo cubre la autorizacion** (§41.2). No es
  fallo de solidez: la autorizacion es posesion de claves, no aprobacion de
  una operacion.

- [x] **30. ⚠️ FALLO DE SOLIDEZ CORREGIDO Y VERIFICADO en `circuit_send`
  (§50).** ~~`send` no ata `COL_R_ID` entre la fila del compromiso y el
  resto; constancia muerta por el solapamiento de §38.~~ ✅ **Corregido** el
  30-07-2026 (§50.5): `C_TRANSPORT` recibe sus 15 ranuras (era 7), la lista
  de grados ajusta 13->21, y las 8 constancias —que ya estaban escritas— se
  imponen. El test testigo pasa de rojo (ignore) a **verde**: 204 tests, 0
  fallos. **Queda** (§50.4/§50.5): revisar `circuit_mint_pending` (tercer
  constructor de compromisos, sin verificar) y limpiar las 8 muertas de
  `circuit_claim` (sobran tras §39.1, sin urgencia). Se abren como 35 y 36.

- [x] **29. Carrera del gancho de panico global: identificada y
  eliminada.** ~~Fallo unico sin explicar; hipotesis no respaldada por
  `--test-threads=1`.~~ Ese descarte estaba mal: un hilo elimina la carrera
  que se quiere probar. **Reproducida** subiendo la contencion (1 de 40 a 16
  hilos) y **corregida** (§45): los 32 bloques `take_hook`/`set_hook`
  silenciaban un `eprintln` sin proteger el mensaje —que viaja en el `Err`
  de `catch_unwind`— y a cambio metían la carrera. Eliminados y
  **verificado con 260 pasadas a 16 hilos sin fallo** (§45.6). El registro
  se empujo una vez antes que el codigo (`bcb9f73` sin el parche, corregido
  en `97d7c7f`); queda anotado.
  ⚠️ **El ámbito del censo, anotado en el §365.** Los 32 son los
  que el censo vio: el `--stat` de `97d7c7f` son 22 ficheros, todos de
  `stark-experiment`. `crates/zk-ssl/src/tests.rs` llevaba uno desde `9a1e2f1`,
  anterior a la barrida, y se retira en el §365. La entrada **no se reabre**:
  su tesis -la carrera identificada y eliminada- se cerró; esto es residuo
  de ámbito en el cuerpo, como en la 32, la 33 y la 49-A.

- [ ] **7. ⚠️ Encargar la auditoria externa —y ESPECIFICAR contra que.**
  ⚠️ **Lo que a esta entrada le faltaba**: decia que hace falta auditar y no
  decia **contra que**. `CONFIANZA_RESIDUAL.md` lo responde (entrada 48):
  **especificacion formal de cada AIR** —que restricciones existen, que
  grados declaran, y sobre todo **que NO restringen**— como **contrato** de
  la auditoria. Sin eso, auditar circuitos es una lectura.
  ⚠️ **Y hay indicio de que habria funcionado**: §72 fue una restriccion
  escrita **sobre el carril equivocado**. Una especificacion que dijera
  «`C_PEND_IN` ata el carril A» junto a «`C_PEND_ENTRY_B` inserta el carril
  B» habria hecho visible el desajuste **leyendo la especificacion, sin
  leer el codigo**. Es plausible, no seguro: se dice como lo que es. Ya no es solo por el
  argumento lockstep (§16.4). Tras §39 hay un defecto **demostrado** de una
  clase que las herramientas del proyecto no detectan por construccion, y
  el barrido sistematico tampoco la encuentra (§40): el unico metodo
  conocido es la lectura semantica circuito a circuito, cara, manual y en
  la que el autor acaba de equivocarse tres veces en un dia.

- [x] **35. `circuit_mint_pending` verificado sano.** ~~Revisar si su
  `COL_R_ID` esta pisado como en §50.~~ **Verificado** el 30-07-2026 (§50.6):
  el test `a_mint_pending_with_inconsistent_receiver_identity_is_rejected`
  **rechaza** la traza de dos identidades; la constancia se impone
  (`C_TRANSPORT_NEW` reserva sus 12 ranuras) y rechaza por la razon correcta
  (§16.5). ⚠️ **Rectificado (§50.7)**: aqui se dijo que el fallo «no era
  sistemico»; era falso —los TRES constructores tenian el solapamiento de
  §38, `mint_pending` es el unico con la disposicion **bien contada**—. El
  test queda como regresion permanente.

- [x] **36. ⚠️ NO era limpieza: TERCER fallo de solidez, corregido.**
  ~~Borrar 8 restricciones muertas de claim, borrado seguro.~~ `claim` tenia
  el mismo solapamiento que `send`, y aunque `COL_R_ID` estaba muerto tras
  §39.1, el compromiso **aun lee `COL_SALT`**: un §50 sobre el aleatorio
  (§50.7). Confirmado por test y **corregido** (C_TRANSPORT a 15, grados
  13->21). El testigo pasa a verde. ⚠️ Degradado a «cosmetico» **tres
  veces** (§39.4, §49.1, al cerrar la 35) antes de que un test lo mirara: el
  error del dia en su forma mas pura.

- [x] **37. Barrido de disposiciones: hecho, ampliado y CORREGIDO.**
  ⚠️ **Se cerro sobre una cobertura parcial** (§59.2): la herramienta solo
  entendia `result[C_ALGO + i]` y saltaba **en silencio** los circuitos que
  indexan con numeros crudos — **10 de 24**, incluidos `compliance_circuit`,
  `solvency` y `nullifier_tree`. El resumen decia «14 circuitos limpios» y se
  registro como si fuera total. **Corregido** el 30-07-2026: el indice se
  evalua entero y los comentarios se ignoran. **Resultado: 24 circuitos,
  ninguna colision.** La conclusion era correcta pero no estaba verificada;
  ahora lo esta.
  ~~Barrer el vicio de conteo «declara N, reparte M<N» en TODOS los
  circuitos.~~ **Cerrada** el 30-07-2026 (§53): `tools/check_constraint_layout.py`
  cruza los indices absolutos que escribe cada `evaluate_transition` y detecta
  COLISION (la firma de §38), DESBORDE y ranuras MUERTAS. **Resultado: 14
  circuitos, ninguna colision** — no hay un cuarto fallo de esta clase en los
  once que nadie habia contado. La herramienta lleva `--autotest` que
  reproduce el fallo de §50 y comprueba que lo caza. ⚠️ **No cubre**
  restricciones ausentes o mal formuladas (§39 no se habria detectado): para
  esa clase sigue sin haber mas que lectura semantica y test discriminante
  (§40.4, entrada 7). Tres errores mios construyendola, en §53.4.

  ~~Original:~~ **37. Barrer el vicio de conteo «declara N, reparte M<N».** El solapamiento de §38 produjo fallos de solidez en dos de
  los tres constructores de compromisos (§50, §50.7) y estaba en los tres.
  La causa raiz —una constante `C_X = C_prev + N` con N menor que lo que su
  `evaluate_transition` escribe, de modo que el grupo siguiente la pisa—
  **no se ha buscado fuera de esos tres circuitos**. Hay una docena con la
  misma estructura (`audit`, `burn`, `freeze`, `governance`, `mint`,
  `recovery`, `settlement`, `threshold`, `compliance`, `double_entry`,
  `dual_climb`…) y ninguna comprobacion automatica de que el reparto cuadre.
  Un `tools/check_constraint_layout.py` que, por circuito, verifique que cada
  `C_X` deja sitio a lo que se le escribe cerraria la clase entera. Es la
  **generalizacion del hallazgo de §38**: mientras no se haga, no se sabe si
  quedan mas §50 sin mirar. Prioridad de solidez, por delante de lo
  declarativo.

- [ ] **116. El nivel declarado de producción y el rango de `m` de la edad: lo decide el autor.**
  Medido en el §697 con la función de `winter-air` (`proof/security.rs`) sobre las opciones de
  producción —42 consultas, blowup 16, molienda 21, extensión cuadrática— y la forma oculta de cada
  prueba: conjeturada 127 en todas; UDR 59; en las familias de longitud fija, LDR 88 con la traza
  más corta (la apertura del reembolso, T = 64), 82 con T = 512 y 80 con la más larga (envío, cobro
  y quema, T = 1024). La edad, cuya longitud depende de `m`, pierde dos bits de LDR cada vez que la
  longitud se dobla: 88 con `m = 3`, 80 con `m = 7`, 62 con `m = 16` y 48 con `m = 23`; con 80 o
  120 consultas en vez de 42 da las mismas cifras para cada `m`. Y la prueba de `m = 24`, que
  `comprobar_enunciado` admite, no se puede generar hoy. Los tests del §697 fijan lo de hoy —en las
  familias de longitud fija, conjeturada ≥ 127, LDR ≥ 80 y UDR = 59; en la edad, el piso de cada
  `m`, con `m = 24` declarada sin piso— y no deciden nada.
  **Lo que pide, y lo decide el autor**: el nivel que se declara —el piso de LDR y el umbral de
  UDR—; el rango de `m` de la edad —un tope que la deje en ese piso, u otras opciones para la
  edad, que cambian lo que el kit acepta—; y, con eso, los parámetros del corte zkssl/0.5 que
  propone el ZK-1 de `doc/blueprint-v2.md`. Lo que se decida cambia los pisos de los tests en el
  mismo sello, con su asiento.
  **Publicado (§698)**: lo medido de hoy, con la tabla de la edad por `m`, está en `SECURITY.md`
  §3.11, y lo repiten el comentario de `proof_options`, `PRINCIPIOS.md`, `FIVE_BACKENDS.md`,
  `PAPER.md`, `PAPER_EN.md`, `doc/ZENODO.md` y los dos institucionales; el nivel cuántico se
  declara no medido. Lo que se decida aquí mueve también esos textos, en el mismo sello.
  **La conjeturada, con el término DEEP (§708)**: su fórmula pone el campo entero y no descuenta el
  término DEEP de la demostrable, que crece con la longitud de la traza. Con él, no pasa de 116,8
  con 128 filas ni de 112,8 con 2.048, y en la edad, de 116,8 con `m = 3` a 96,8 con `m = 23`.
  Lo fija un test de `tests_nivel` en `zk-ssl-air` (desde el §717, sólo de `m = 3` a `m = 13`,
  con 106,8), y `SECURITY.md` §3.11 y los textos de arriba lo dan junto a los 127. Lo que se
  decida aquí mueve también esa cifra.
  **El rango de `m`, acotado (§717)**: el enunciado de la edad admite `m` de 3 a 13
  (`zk_ssl_air::M_MAX`), y `m = 24` ya no. El tope sale del coste del juez y de las cotas de la
  puerta del RFC-0007, no del nivel: con 13 la LDR es 68 y la conjeturada con el DEEP, 106,8. Los
  tests del §697 y del §708 quedan en las `m` admitidas, `SECURITY.md` §3.11 lo anota, y `PAPER.md`
  y `PAPER_EN.md` dan las cifras de `m = 13`. El piso de LDR y el umbral de UDR siguen por decidir.

- [ ] **117. El juez único de verificación, antes del corte zkssl/0.5: lo adopta el autor (§705),
  en la aceptación en bloque.** Es el ARQ-01 de `doc/blueprint-v2.md`. Hoy cada familia se
  verifica con su propia llamada y sus opciones: los 15 `verify::<…>` de la capa, que cuenta
  `toda_verificacion_viva_de_la_capa_lee_con_sal` (`crates/zk-ssl/src/lib.rs:1237`, §578), la
  pareja umbral (`verify_threshold_pair`) y las cinco `verificar` del kit en `crates/zk-ssl-air/src`.
  Son 21 sitios, y nada obliga a que todos acepten con la misma política. El autor lo adopta el
  07-10-2026, en el §705, con la aceptación en bloque y de forma reversible: **un solo juez**, por
  el que pasa toda verificación de producción, con **la política de aceptación del juez,
  constante**, y no de cada familia; y se construye **antes** del corte de cable zkssl/0.5, como
  pide la regla (3) de `doc/blueprint-v2.md` §4.6, para que el cambio de opciones de ese corte
  toque un sitio y no 21.
  **Lo que pide**: la forma —la del boceto 1 del plano, `juzgar::<J: Juez>` en
  `crates/zk-ssl-air/src/juez.rs`, u otra—, con su asiento; la puerta que la ata, ningún
  `verify::<` vivo fuera del juez, con las excepciones que declare, como la acotó el verificador del
  ARQ-01; y la versión del kit que la lleve. Medido sobre `c990868`: no hay
  `crates/zk-ssl-air/src/juez.rs`.

- [ ] **118. Cómo se identifica una prueba: la marca v2 del RFC-0018, la D-G, en estado de
  propuesta (§707).** Hoy una prueba STARK de la casa sólo lleva la marca de la ocultación,
  `arqueo:oculta:1` y m, 19 bytes en el meta de su traza; la familia la elige quien llama, y una
  prueba de otra versión de su AIR cae con un error de winterfell y no con su nombre (§699). El
  autor decide el 07-10-2026 la marca v2, de 24 bytes fijos con la familia, la versión de la AIR y
  el perfil, atada por la semilla de Fiat-Shamir, en el corte zkssl/0.5 y no antes, para todas las
  AIR con probador oculto, también las que sólo juzga el nodo; y que en la edad el juez la lea antes
  del enunciado. La sesión propuso, y rige hasta que el autor diga otra cosa (§707): la marca, la
  primera en todos los jueces y en ese orden también en la capa y en `tools/segunda`; el registro,
  en `spec/NUCLEO.md`; el perfil 255, reservado para otro motor; y `digest_pi`, el de la E3 del
  RFC-0019, atando también la familia y la versión.
  **Lo que pide**: la E3 del RFC-0018 en ese sello —el fork en sus dos orillas, los 23 probadores
  con fila, los jueces (o el juez único de la 117, si va antes), `tools/segunda`, el registro con su
  KAT, los textos en `spec/PAQUETE.md` y los vectores que lista la D-G, más un test del fork que
  mida la atadura a la semilla—; y las notas fechadas que la D-G pide en el RFC-0009 (D-H y D-S) y
  en el RFC-0005 (D-B). Medido sobre `679a035`, en las pruebas de `spec/vectors/`: fuera de `0.3/`,
  62 llevan la v1 (58 con m = 64, y las cuatro de m = 32 del §706) y 25 el meta vacío (las de los
  recibos de los lotes de rechazo y los dos negativos del §706); las 34 de `0.3/`, el meta vacío.

- [ ] **119. Abrir una cuenta prueba que se tiene la clave (§802).** Desde el §802 abrir rechaza una
  identidad que ya tiene cuenta o que una recuperación retiró (`SECURITY.md` §3.13), y eso agrava la
  ocupación anticipada: quien conozca la identidad X antes de que su dueño abra la cuenta la ocupa con
  su vista y su sal, y el dueño tiene que cambiar de clave; y quien conozca la identidad nueva de una
  recuperación antes de que se aplique la bloquea (hoy sin consecuencias: la recuperación no está en
  el cable). El autor lo dejó como residuo declarado (D-2 del §802) y lo lleva al corte `zkssl/0.5`.
  **Lo que pide**: que `zkssl_openAccount` lleve una prueba de que quien abre conoce la clave de la
  identidad —un circuito nuevo y un parámetro nuevo, con la versión que sube en el corte—, que es
  también la autorización que la 8 echa en falta. **Qué la cierra**: abrir sin esa prueba se rechaza,
  con su test y su vector, y el residuo de `SECURITY.md` §3.13 se quita con su marca.
  ⚠️ **§860**: decía que la prueba de posesión «es también la autorización que la 8 echa en falta». Lo
  es en un sentido —abrir deja de ser la única transición sin prueba
  (`crates/zk-ssl/src/accounts.rs:250-266`) y nadie ocupa una identidad cuya clave no tiene—, pero
  no en el de la 8: quien genera claves frescas las prueba todas y sigue agotando el tope de cuentas
  (`max_accounts`). La 8 se decidió en el §860 sin autorización de custodios; el «Qué la cierra» de
  esta entrada no cambia.
  ⚠️ §860: una restricción de la 52 (D-1): la prueba de apertura ata la sal como entrada pública, para
  que el operador no la cambie, y no impone su origen; si lo impusiera, cerraría la rotación
  precomprometida en la sal.

- [ ] **130. HVZK de la ocultación: el argumento, los falsadores más allá del censo y la promesa
  (A2 a A5; la cota, medida en el §896).** La ocultación está encendida desde el §538 y lo que la
  casa promete es el censo: «ningún valor de columna constante sale literal» (RFC-0009 D-A,
  `SECURITY.md` §3.bis). Cero literales no es HVZK: la suite de E2 cuenta, no simula, y lo que
  sostiene la ocultación más allá del censo es la construcción de la nota 2024/1037 con la cota de
  la D-I, que nadie había medido por familia ni argumentado para la composición exacta del fork
  (H7). El §896 mide la cota (A1): lo que el verificador recibe de cada columna y de cada trozo del
  cociente, con la dirección que DEEP y FRI dejan pasar, frente a las T filas aleatorias y al m de
  la marca, en las pruebas reales de los 23 probadores con fila, con el rango pleno de sus matrices
  y dos mutantes con nombre. **Lo que pide, en este orden y sin saltarse ninguno**: A2, el argumento
  escrito para el protocolo tal como está implementado —el simulador y los lemas: el rango de
  Cauchy por columna, Vandermonde y la cancelación en los trozos, la uniformidad de DEEP salvo una
  dirección, las hojas saladas en el modelo de oráculo aleatorio y la moneda sembrada con 32
  bytes—, cada lema atado al test del §896 que mide su hipótesis, con la lista de hipótesis (medida
  o supuesta) y de lo que no cubre, en un documento propio bajo `doc/`, citado desde el RFC-0009; A3, los
  falsadores más allá del censo: la recuperación del secreto por álgebra lineal cuando m o T bajan
  de la cota, en un juguete, y la semilla repetida en dos pruebas, en sello; el distinguidor
  estadístico sólo si cabe en `--largo` con su pin, y si no, declarado fuera; A4, la promesa nueva,
  sólo con A2 y A3 sellados: una decisión del RFC-0009 con la promesa exacta de HVZK computacional
  bajo las hipótesis de A2, `SECURITY.md` §3.bis y §3.11, la regla 3 del PROCESO y la confianza
  residual de D-A, con la frase H7 conservada hasta una revisión ajena y sin tocar «nivel
  cuántico: no medido»; A5, la D-I decidida por el autor con el argumento delante: conservar la sal
  o retirarla en un corte futuro, nunca en `zkssl/0.4`. Restricciones: ni motor nuevo, ni nivel
  poscuántico del STARK, ni `zkssl/0.4` roto. Hasta A4 nada de la casa dice «HVZK»: la promesa
  sigue siendo el censo. **Qué la cierra**: A4 sellada y la D-I decidida.
  ✅ a medias (§897): A2, escrita: `doc/ocultacion-hvzk.md`, con el simulador, los lemas L0 a L6
  atados a los tests del §896 y del §897, la tabla de hipótesis —medida, de modelo o supuesta— y lo
  que no cubre; corrige la cuenta del §896 (46 dimensiones sobre F por columna y 44 sobre E por
  trozo, sin «dirección residual», que era pública) y mide lo que una prueba TOCA, 2q + 6 = 90
  frente a T: las cinco familias con T = 64 quedan sin argumento independiente de la AIR (la 131);
  la condición R3 —que la columna aleatoria y los aleatorizadores del cociente cubran lo que FRI
  abre de DEEP— queda SIN medir, es lo primero de A3, y a la solvencia le falla por cuenta (la
  132). QUEDAN A3, A4 y A5: sigue abierta.

- [ ] **131. Cinco familias abren más puntos de los que sus filas aleatorias enmascaran: T = 64
  frente a 2q + 6 = 90 (§897).** Medido en cada canon desde el §897 (`INDEPENDIENTE_DE_LA_AIR`,
  `crates/zk-ssl/src/instrumento_cota.rs`): en NullifierThresholdAir, ThresholdAir, EdadAir con ocho
  hojas, RefundAir y RefundAirV2 la traza oculta lleva 64 filas aleatorias por columna, y lo que una
  prueba toca por columna —las 42 posiciones y sus filas siguientes, z, z·g y g²·z— son 90
  dimensiones sobre F: el lema L1 de `doc/ocultacion-hvzk.md` no las cubre, y para ellas el
  argumento de A2 necesita un lema que depende de la AIR (L6, enunciado y no demostrado) o más
  filas aleatorias, T = 128, el doble de filas y de coste para esas cinco, que cambia sus pruebas y
  por eso es de un corte futuro, nunca de `zkssl/0.4` (la D-AG del RFC-0009 acolchó a 64 con la
  cuenta de 44). Dos de las cinco son del cable, los dos umbrales; la edad con 16 hojas o más
  (T ≥ 128) cumple la cuenta, pero el instrumento sólo la mide con ocho. **Qué la cierra**: la
  decisión del autor —L6 por AIR, o T = 128 en el corte que lo lleve— y, si es T = 128, el sello que
  lo haga con la cota de los tocados en verde para las cinco. Hasta entonces, lo que A4 enuncie
  para las demás no vale para ellas.

- [ ] **132. La solvencia no tiene máscaras que cubran lo que FRI abre de DEEP: 506 dimensiones
  frente a unas 630 (§897).** `SolvencyAir` (T = 256, dos trozos) cumple la cuenta de lo tocado —la
  131 no la toca—, pero lo que FRI abre del polinomio DEEP, los cosets de plegado de la capa 0, la
  capa 1 y el resto, son unas 716 dimensiones sobre F, 630 sin determinar, y las dos máscaras que no
  dependen del testigo —la columna aleatoria, con 466 libres, y el único aleatorizador del cociente,
  con 40— suman 506: la condición R3 de `doc/ocultacion-hvzk.md` (L3 y apartado 6) le falla por
  pura cuenta, con q = 42 y con q = 40, antes de medir ningún rango. Lo que la enmascararía de más
  son las filas aleatorias de sus columnas reales, que entran en los trozos con el grado de sus
  restricciones y de forma que depende del testigo: nadie lo ha argumentado. Las salidas: una
  máscara propia de DEEP —el «polinomio aleatorio antes de FRI» del issue 9, del grado de D— en un
  corte futuro, nunca en `zkssl/0.4`; o T = 512 para la solvencia, donde la columna aleatoria sola
  llega por cuenta (978 frente a unas 758). **Qué la cierra**: la decisión del autor y, con ella, el
  sello que mida R3 en verde para la solvencia. Hasta entonces, lo que A4 enuncie no vale para ella.

## D. Declaradas, acotadas, sin urgencia

- [x] **8. `open_account` sin autorizacion: decidido (⚠️ §860), no se exige autorización de
  custodios, y el agotamiento del tope, declarado.** ~~El tope de cuentas mitiga a
  medias: un atacante aun puede agotar el cupo, y la solucion correcta
  exige un circuito nuevo (§16.1).~~
  ⚠️ **§860: decidida por delegación del autor («resuelvelas aplicando los principios»), con el
  criterio del §609 y el precedente del §642; REVERSIBLE; entra con su aceptación. Abrir no exigirá
  autorización de custodios, y la entrada se CIERRA.** La «solución correcta» del §16.1 daría a los
  custodios un segundo veto sobre el alta, sin quitar el del operador, que ya puede negarse a abrir
  (`PRINCIPIOS.md` §0); impondría el alta permisionada a todos los casos de uso, lo contrario del
  criterio del §609; y haría custodial el traspaso, la R4 de `doc/ROTACION_DE_CUENTAS.md`, que abre
  una cuenta nueva y es la única rotación sin custodios que existe hoy, sin código (la R2 y la R3
  tampoco los piden, pero no existen). La prueba de posesión de la 119 no la cierra: prueba quién
  puede ocupar una identidad, no cuántas identidades frescas se abren.
  **El residuo, declarado en `SECURITY.md` §2**: agotar el tope de cuentas (`max_accounts`) es un
  ataque de vivacidad, y la vivacidad es hoy del operador, que decide quién llega a
  `zkssl_openAccount`. Medido sobre `d4d6d45`: el nodo abre con `open_with_id` sin prueba y sin
  límite de ritmo (`crates/zk-ssl-node/src/main.rs:3783-3796`; `permiso_de_ritmo` solo lo pide
  `zkssl_verifyChain`, `:3775`), con un tope de 1.000 cuentas por defecto (`main.rs:248`). El tope
  es inmutable —al reabrir con otro, `ParameterMismatch`
  (`crates/zk-ssl/src/persistence.rs:306-310`)— y una cuenta no se cierra, así que agotarlo vale
  para toda la vida del libro, y quita también la R4: sin plaza, ningún titular rota sin custodios,
  y cada R4 legítima gasta una plaza que no vuelve (`doc/ROTACION_DE_CUENTAS.md` §4). La falta de
  autenticación y de límite de tasa del nodo, y que escuche en `127.0.0.1` por defecto, ya los
  declaraba `SECURITY.md` §3.3.
  **Descartadas**, además de la de custodios: una política de apertura fijada en la génesis (cumple
  el §609 a la letra, pero en modo libre no cierra nada y ningún caso la pide:
  `doc/USE_CASES.md` §«What none of this claims» deja fuera «that one person holds one account»);
  dar la 8 por cerrada con la 119 (arriba); un coste económico por alta (en un libro cerrado la
  unidad la emite el operador, y la reserva vuelve a pasar por una emisión custodial); una prueba de
  trabajo por alta (asimétrica, y su dificultad la fija el operador); un límite de ritmo en el nodo
  (despliegue y no estándar, y solo retrasa: a una apertura por segundo, las 1.000 plazas caen en
  unos 17 minutos, DEDUCIDO); liberar plazas (o el conjunto de retiradas crece sin límite con
  identidades frescas, la memoria que el tope existe para acotar, o se rompe la invariante del §802,
  `AUDITORIA.md:53449-53451`); que la gobernanza pueda subir el tope (enmienda la mitigación
  registrada del §16.1, un tope inmutable que va en `params_digest`, y solo aplaza: el atacante
  vuelve a llenarlo; y con la D-3 en G0, la 20, lo que se fija al constituir el libro y se compara
  al reabrirlo no lo cambia nadie dentro de él); y posponerla al corte `zkssl/0.5` (no depende de
  nada que el corte vaya a medir).
  **No responde** a lo que el §16.1 deja a un auditor —si el tope basta para el caso de uso—, que
  sigue pendiente y fuera de la delegación. **Se reabre** si
  `doc/USE_CASES.md` §«What none of this claims» deja de excluir la admisión, o un RFC pide que un
  tercero compruebe el alta de cada cuenta, y el verificador del tercero reverifica las transiciones
  (`README.md` §«Leer antes de nada»: hoy no lo hace); o si el autor reabre la 23. Entonces, la
  política de apertura fijada en la génesis, con su `OP_` en el REGISTRO y su RFC
  (`spec/NUCLEO.md` §3). Marcados en el mismo sello: el §16.1, en su línea;
  `ARQUITECTURA.md` §«`open_account` sin autorización → tope de cuentas ✅»; la R4 de
  `doc/ROTACION_DE_CUENTAS.md`; y la 119, con su línea.

- [x] **9. Congelacion sin caducidad ni motivo registrado: ya declarada,
  y reconciliada.** ~~El circuito prueba que dos custodios la autorizaron,
  no que tuvieran razon, y dura hasta que alguien la levante.~~ **Cerrada**
  el 30-07-2026: ya estaba en §16.2. Al verificar para cerrarla, §16.2 y §7
  se contradecian sobre «recibir estando congelada» —§16.2 lo llamaba
  «deliberado» sin notar que la retirada de la via de un paso (§36) dejo
  ese caso en el **limbo** que se queria evitar (§7)—. Reconciliadas. El
  fondo (sin caducidad, sin motivo) sigue abierto como decision de diseno,
  pero **declarado**; implementarlo es circuito, no redaccion.

- [x] **10. Seguridad conjeturada vs demostrable: decidido e implementado.**
  ~~Decidir sobre 127 bits conjeturados; coste 36,7 KB → 125,6 KB.~~
  **Cerrada** el 30-07-2026. Recorrido con dos errores mios corregidos: (1)
  retire «125,6 KB» por sin-fuente (§48.2) tras una busqueda incompleta;
  (2) **estaba en `iso_bridge.rs`** (§48.3). La decision **ya esta
  implementada**: la config por defecto del puente ISO usa 120 queries y
  blowup 16, alcanza **128 bits demostrables** (no conjeturados), cuesta
  ~125 KB medidos en `compliance_real_proof`, y deja la eleccion explicita.
  36,7 KB (normal) vs ~125 KB (fuerte) es el coste real, en el repo. Que
  circuito usa cual es afinado, no frente abierto.

- [ ] **11. Canal lateral de ISO 20022.** Posicion, aleatorio e importe del
  pendiente viajan fuera del mensaje, sin especificar como; bloquea
  cualquier piloto. **Comprobable**: `grep -n "PendingNotice"
  crates/zk-ssl/src/iso.rs` — el aviso que el receptor necesita para cobrar
  no forma parte del mensaje ISO que se genera. **Constatacion de ausencia,
  no analisis**: no hay seccion porque no hay nada que analizar hasta que se
  especifique el canal.


  ⚠️ **Y es el mismo objeto que el B11.2 de la entrada 48** (§268): allí se
  pedía **cifrar** el aviso; aquí, que **viaje dentro** del mensaje. Son el
  aviso pendiente mirado por dos caras, y desde §268 vive **sólo aquí**.
  Cifrarlo es trabajo de circuito y de protocolo: se decide con esta entrada
  delante, no de propina.
  ⚠️ **§791: el canal, medido y sin elegir.** `doc/AVISO_EN_ISO20022.md` (aceptada como análisis
  en el §799, con sus decisiones abiertas; ⚠️ §799: hasta aquí decía «(BORRADOR)») mide qué campos
  del pacs.008 y del pacs.002 podrían llevar el aviso, contra el esquema y no contra el recuerdo:
  leído en los modelos que `xsdata` genera desde los XSD (`pyiso20022` 1.6.2), porque el sitio de
  ISO no era alcanzable, y con el cotejo contra el XSD original pendiente. Lo que sale:
  `SplmtryData` existe en los dos mensajes y lleva el aviso entero, también cifrado, así que la
  frase «ISO 20022 no tiene campo» de `iso.rs` no es exacta; los campos de 140 caracteres —`Ustrd`,
  `AddtlRmtInf`, `InstrForCdtrAgt`— llevan el v1 en claro, y el v2, en una sola ocurrencia, solo en
  base64; y el pacs.002 va de vuelta al deudor, que ya conoce el aviso. Con el B11.2, tres
  condiciones que esta entrada no tenía: ninguna cuenta tiene una clave a la que cifrar, un acuerdo
  de claves sobre una curva no pasa `tools/check_curvas.py` (§788) y cifrado solo cabe entero en
  `SplmtryData`. ⚠️ §835: decía que un acuerdo de claves sobre una curva no pasa la puerta; `ring`,
  que ya está en el SDK por el TLS de `ureq`, trae X25519 y ECDH P-256/P-384 y no está en el patrón
  (la 122). El código de la casa sigue sin escribir ningún acuerdo de claves; el del TLS lo hace
  `rustls`, dentro de `ureq`. ⚠️ §838: decía que `ring` no está en el patrón; ya está, solo bajo
  `rustls` y `ureq`, y la puerta busca su uso en las fuentes de la casa: un acuerdo de claves con
  las bibliotecas del patrón vuelve a no pasarla (OpenSSL, otra pila TLS, no está en él). El del TLS
  es la excepción del §106. El campo, la codificación y el orden frente al cifrado son del autor; el
  prototipo, un test en `iso.rs`, necesita compilar y no está hecho. **La entrada sigue abierta.**
  ⚠️ **§861: el canal, decidido por delegación del autor («resuelvelas aplicando los principios»), con
  el criterio del §609 y el precedente del §642; REVERSIBLE; entra con su aceptación.** Un formato y
  dos portadores. **D-1**: dentro del mensaje, el aviso va en `CdtTrfTxInf/SplmtryData` del pacs.008
  (la fila A de `doc/AVISO_EN_ISO20022.md`), y del banco del acreedor al titular, en
  `TxDtls/SplmtryData` del camt.054; fuera del mensaje, el mismo elemento. Nace en la vía por
  materiales, antes de la orden, y lo escribe el lado del pagador: el operador no escribe nada, y la
  vía de en medio del puente no gana emisor. La remesa y `InstrForCdtrAgt` no: su nombre dice otra
  cosa y cifrado no caben. **D-2**: un elemento propio con espacio de nombres versionado; sus hojas,
  en `Q` y `B32` canónicos; se conforma por infoset, no por bytes; sin base64. **D-3**: v1 y v2; la
  hoja `x` decide, y el espacio de nombres solo dice si va en claro o cifrado. **D-4**: en claro
  dentro del mensaje es elección del pagador, no lo que el estándar da por defecto («conceder
  observación que no hace falta se defiende mal después», `SECURITY.md` §2.ter), con la fuga dicha
  —posición y sal a la cadena de agentes, sin poder de cobro (⚠️ §827: y en un v2 también `x`, con lo que se aprende qué cuenta pagó y su Δ, la D-I del RFC-0008; lo dice la Seguridad del RFC-0022)— y su retirada escrita desde el
  principio. Cifrarlo (el B11.2) pide una enmienda del §106, que solo da el autor, o la vía
  simétrica con el AEAD de la casa y una clave por relación, que el receptor entrega con su
  dirección y que deriva de `sk` con dominio propio (la D-9 de la 52, en
  `doc/ROTACION_DE_CUENTAS.md` §6); con una curva no, aunque `ring` ya esté en el SDK (la 122): lo
  prohíbe el §106, no la puerta (⚠️ §827: desde el §838 lo ve también la puerta, la R5 de `tools/check_curvas.py`, como dice la marca del §838 de arriba). **D-5**: por RFC, con el número libre al entrar; su KAT, en una
  familia propia, `spec/vectors/aviso/`, solo tras cotejar el XSD original y las reglas de uso
  públicas de una práctica de mercado (hoy, 403). **D-6**: «ISO 20022 no tiene campo» y la cita al
  §21, que es el §39, se corrigen con marca donde viven; `doc/historia/` no. El §861 lo hace en los
  comentarios de `iso.rs`, de `client.rs` y del SDK, en `spec/RPC.md`, `ARQUITECTURA.md`, los dos
  `PAPER` y `doc/DIAGNOSTICO_ESCALADO.md`; quedan, para otro sello, el «§21» de la plantilla de los
  RFC y de los RFC del 0002 al 0007, el de la 74, cerrada, el de `doc/CADUCIDAD_PENDIENTE.md:35` y
  el del `println!` de `crates/zk-ssl-sdk/examples/e2e.rs:25`, que es código. **Se reabre** la D-1
  si el XSD o esas reglas quitan `SplmtryData`; la D-2, si una práctica de mercado veta los espacios
  de nombres ajenos o acota el `Envlp`; la D-4, cuando exista la versión cifrada o si una medida de
  lo que aprende un agente da más que el vínculo declarado; la D-5, si el autor prefiere
  documentación del puente; y todas, si el autor las revoca (la tabla, en `doc/AVISO_EN_ISO20022.md`
  §8). **Sigue abierta** hasta el prototipo por materiales en `iso.rs` y el RFC PROPUESTO; después,
  solo por el B11.2.
  ⚠️ §827: **el RFC, PROPUESTO.** El RFC-0022 (`spec/rfc/0022-el-aviso-en-iso20022.md`) escribe como
  protocolo las seis decisiones de la nota del §861, de arriba, y cita su razón, sus descartes y su
  reapertura, en `doc/AVISO_EN_ISO20022.md` §8, sin copiarlas. Decide una sola cosa de protocolo
  que ella no dice, por delegación y REVERSIBLE: la compuerta del KAT de la D-5 coteja los tres
  mensajes de la D-1 —el pain.001, que el documento no nombraba, el pacs.008 y el camt.054—, y no
  la fila E del documento, que es el pacs.002; lo sellado lleva su marca. Tres etapas, ninguna
  construida: la E1, el prototipo por materiales en `iso.rs`; la E2, el KAT en
  `spec/vectors/aviso/`, detrás de esa compuerta; y la E3, el cifrado, que es el B11.2. Declara lo
  que la nota no decía: en un v2, el aviso en claro da también `x`, con lo que quien lea el mensaje
  aprende qué cuenta pagó y su Δ (la D-I del RFC-0008); en claro, el aviso no tiene integridad; la
  atadura del `ACSC` a la orden acota el límite de `claim_pacs008`, no lo cierra; y el formato no
  es ninguna superficie del RFC-0021, solo sus vectores. Llevan su marca la D-4 de arriba, por
  `x`; su frase «lo prohíbe el §106, no la puerta», porque desde el §838 lo ve también la puerta; y
  la del §861 en `iso.rs:530`, que la vía por materiales «lo cerrará». Deja dos preguntas abiertas
  para el autor: la enmienda del §106 que pediría la vía (i) del cifrado, y si un v2 puede ir en
  claro dentro del mensaje. Con el RFC PROPUESTO, de lo que la nota del §861 pide queda el
  prototipo: **sigue abierta** hasta la E1, y después, solo por el B11.2.
- [ ] **12. Fondos muertos — POLITICA DECIDIDA (§119), ~~piezas 1-3
  pendientes~~ piezas 1-3 HECHAS; la tienen abierta el suelo Δ_min, T3b, la
  via v1 y dos flecos de la v2 (§779).** ✅ **La reversion es un segundo
  cobro**: el **emisor, con su clave, tras un plazo**, hacia un `refund_id`
  comprometido en el envio.
  ⚠️ **Lo que mata las alternativas**: operador, gobernanza, custodios y
  **barrido automatico** serian **la primera transicion donde el dinero se
  mueve sin la clave de un titular**. «Nadie nunca» sobrevive como **Δ=∞ pago
  a pago**, eleccion del emisor.
  ⚠️ **`seq` NO es un reloj** (§119.3): cuenta operaciones, y un mes quieto
  son cero entradas. **El plazo se cuenta en cabezas de epoca firmadas**, cuya
  aceleracion deja **evidencia oponible**.
  ✅ **Sin retroactividad, estructural**: los v1 son irreversibles para
  siempre — `t3a_sin_retroactividad_por_construccion`. **T3a 3/3.**
  ⚠️ **Poder compuesto nuevo** (§119.5): el operador **ordena la carrera Y
  acelera el reloj**; cada pieza deja evidencia, **la combinacion no**.
  ⚠️ **Coste**: congelar pasa a **retener fondos de terceros**.
  **Para `PRINCIPIOS.md`**: **Δ_min = 13 meses**, precedente SEPA con la
  analogia imperfecta declarada. **Pendiente T3b.** ⚠️ **§779: llevada a
  `PRINCIPIOS.md` §4.3, marcada ❌**: el arbol no tiene suelo y T3b sigue
  pendiente (ver el recuento al final de esta entrada), y
  `doc/CONFIANZA_RESIDUAL.md:116` ya la citaba como «suelo de PRINCIPIOS».
  ✅✅ **MECANISMO EJECUTADO (§178-§181, 05-08-2026)**: la caducidad
  completa — `refund()` del EMISOR con su clave (el salt §117 fabrica la
  subida; `circuit_refund` #27 + `circuit_credit_climb` #28), **destino
  fijado por `pending_meta`, no por la prueba**; des-emisión de los
  mint-pendientes (el suministro baja al satoshi); carrera post-T declarada
  y probada en ambos órdenes; y **la no-retroactividad de §119 POR
  CONSTRUCCIÓN**: el legado sin meta es inmune para siempre. Números
  (§181): reembolso ~203 ms + 69 KB; des-emisión ~189 ms + 13,8 KB.
  ⚠️⚠️ **DOS DIVERGENCIAS con la política, DECLARADAS**: (i) **T cuenta
  latidos de `log.seq`** (⚠️ §827: son entradas del registro, no cabezas del latido; el RFC-0023, D-A) — exactamente lo que §119.3 descartó como reloj
  («un mes quieto son cero entradas») — porque las cabezas
  firmadas-publicadas (48/B10) no existían cuando esto se escribió.
  ⚠️ **YA EXISTEN: la 48 se cerró en el §268 (10-08-2026), con B10 HECHO**
  —`firma_cabeza.rs`, `zkssl_signedEpochHead`, el testigo de la CLI y el
  contador de recepción—, así que la migración estaba hecha sin saberlo.
  ✅✅ **MEDIDO en el §340: no había reloj que migrar.** `apply_refund`
  compara `self.log.len()` y `epoch_head` publica `seq: self.log.len()`
  —**el mismo número**—, y `seq` entra en `epoch_digest_v3`, que es lo
  que el nodo firma. ⚠️ **El atado NO existía y ahora sí**:
  `la_altura_entra_en_el_digest` se pone rojo si alguien saca `seq` del
  digest. Sigue siendo **detectable, no impedido**: el poder compuesto de
  §119.5 (ordenar la carrera Y acelerar el reloj) deja rastro **sólo si hay
  quien retenga una cabeza anterior y compare**.
  (ii) **Sin Δ por pago**: el «Δ=∞ elección del emisor» no está — T es
  global con knob (`set_refund_ttl`, persistida). **La entrada queda
  ABIERTA por estas dos divergencias, no por el mecanismo.**

  ⚠️⚠️ **MEDIDO (§342): la (ii) es ROTURA DE FORMATO.** (⚠️ §861: la cita no se sigue, porque
  el asiento del §342 trata la cabecera de `mint`; esto lo midió la sesión que lo selló y lo trajo
  aquí el mismo commit, `6a1c1bb`, sin asiento que lo registre, y el §343 dice que el RFC-0003 se
  midió sobre ese commit.) El compromiso de
  produccion es `pending_commitment(receiver_id, salt, amount)` —tres campos, ni
  identidad de reembolso ni caducidad— y el nativo del circuito
  (`native_refund_commitment`) declara por escrito ser identico. Para que el
  Δ sea «eleccion del emisor» tiene que entrar en el compromiso, y eso mueve sus
  cinco sitios de produccion, el nativo de `RefundAir` y los vectores que el
  canon exige identicos ⇒ **version nueva, RFC y vectores**.
  💡 El RFC no parte de cero: `t3a_reversion_como_segundo_cobro` y
  `t3b_reversion_temporal_nativa` de `pending.rs` son el prototipo del compromiso
  v2, escrito y ejercitado ~~y nunca adoptado en produccion~~ y adoptado en su
  forma: el RFC-0003 la tomo, con la semantica de Δ cambiada (D1), y paso a
  ACEPTADO en el §356.

  ⚠️ **§779: recontada contra el arbol (14-10-2026). El titulo decia «piezas 1-3
  pendientes» y las tres estan HECHAS.** Son las de §88.4
  (`AUDITORIA.md:7397-7409`; ⚠️ §861: decía `:7364-7369`):
  - **1, el compromiso ata la reversion: HECHA en la via v2.** El RFC-0003
    (`spec/rfc/0003-compromiso-v2.md:3`) compromete `C2 = M(C1, M(f, d(Δ)))`
    (RFC `:53-57`; `pending_commitment_v2`, `crates/zk-ssl/src/pending.rs:108`),
    con el Δ elegido por el emisor y `u64::MAX` como Δ=∞ (RFC `:76-80`). La capa
    juzga con el Δ del sobre (`two_phase.rs:687-696`) y solo acredita si el
    emisor es `f` (`:710-714`). **La (ii) de arriba esta cerrada para la v2**,
    salvo el Δ=∞ de abajo, y esta entrada no lo decia.
  - **2, el reloj atado a una cabeza verificable (§119.3): HECHA.** `seq` entra
    en el digest firmado (§340; lo ata `la_altura_entra_en_el_digest`,
    `crates/zk-ssl/src/log.rs:795`; ⚠️ §827: decía `:791`, que el §815 corrió) y `born` va bajo `pmeta_root`, en reposo
    desde el §388 y en la cabeza desde la v5 (§451-§452;
    `crates/zk-ssl-hash/src/lib.rs:446-479`). ⚠️ Atado no es calendario: el
    reloj sigue siendo `self.log.len()` (`two_phase.rs:676`; la cabeza publica
    ese numero, `crates/zk-ssl/src/lib.rs:891`) y `OpKind` no tiene latido
    (`log.rs:66-101`). Un mes sin operaciones sigue sin mover el reloj, que es
    lo que el §119.3 descarto. El §340 dio la (i) por «CUMPLIDA DE HECHO»: lo
    cumplido es el atado, no el calendario.
  - **3, el circuito de reversion: HECHA.** `circuit_refund` #27 y
    `circuit_credit_climb` #28 (§179-§180); `RefundAirV2` para la v2 (§346).

  **Lo que la tiene ABIERTA, y no es pieza:**
  - **El suelo Δ_min = 13 meses no existe.** `apply_refund` compara
    `now - born` con el Δ del sobre y nada mas (`two_phase.rs:694`), y
    `un_refund_v2_tras_delta_devuelve_al_emisor` reembolsa con `delta = 1`
    (`two_phase.rs:3773-3801`; ⚠️ §827: decía `:3771-3799`, que el §816 corrió; ⚠️ §832: decía `:3763-3791`, correcto hasta
    que el §832 metió ocho líneas más arriba). Con el reloj en entradas,
    «13 meses» no tiene unidad en el arbol. Por eso la linea entra en
    `PRINCIPIOS.md` con ❌.
  - **T3b.** T3b-nativo, 2/2; T3b-circuito, «especificados, condicionados y
    **NO escritos**» (§128.1, `AUDITORIA.md:10529-10544`; ⚠️ §861: decía `:10503-10509`), y ningun asiento
    posterior vuelve a nombrarlo. Los circuitos de la pieza 3 no lo saldan
    solos: sus entradas publicas son `{commitment, amount}`
    (`circuit_refund.rs:209-212`; `RefundAirV2` las reusa,
    `circuit_refund_v2.rs:172`) y el reloj no entra. El RFC-0003 deja el
    plazo a quien compare dos cabezas firmadas con el Δ que el reembolso abre
    (RFC `:128-131`); nadie ha medido si eso salda la (a) y la (e), y el §119.7
    preveia este caso: «Si (a) no puede escribirse porque el reloj no entra
    como input publico, la politica queda condicionada, no caida»
    (`AUDITORIA.md:9751-9752`; ⚠️ §861: decía `:9716-9717`). Y su (c), «receptor congelado ⇒
    rechazada» (`AUDITORIA.md:9747`; ⚠️ §861: decía `:9712`), no se cumple: el §178 eligio «sin
    frozen-check» (`doc/CADUCIDAD_PENDIENTE.md:65`) y ni `refund`, ni
    `refund_v2`, ni `apply_refund` (`two_phase.rs:539-812`) miran congelados,
    contra el §119.4 (`AUDITORIA.md:9697-9704`; ⚠️ §861: decía `:9662-9665`). Es una tercera divergencia, y
    esta entrada no la declaraba.
  - **La via v1, con el knob.** Sin apertura, `apply_refund` juzga con el
    `refund_ttl` global (`two_phase.rs:679-683`; `set_refund_ttl`, `:257`):
    ahi el plazo lo sigue moviendo el operador, y retirarlo o dejarlo de techo
    es decision de corte (RFC-0003 `:154`; ⚠️ §827: decía `:141`, y las marcas del §861 en el RFC la corrieron; §388 (d), `AUDITORIA.md:29398-29399`; ⚠️ §861: decía `:29363-29364`).
    Y todo mint-a-pendiente nace v1 (`two_phase.rs:1699`): el T-A del §388
    (b), fichado y sin hacer.
  - **Fleco 1 de la v2: Δ=∞ no es ∞.** El §501 midio que Goldilocks reduce
    `u64::MAX` a `2^32 - 2`: el emisor abre su propio «nunca» con
    `delta = 2^32 - 2` y la capa aplica ese delta (`AUDITORIA.md:36577-36594`, ⚠️ §861: decía `:36547-36559`;
    el testigo es `el_nunca_se_abre_con_el_delta_reducido_y_muere_solo_en_el_tiempo`,
    `crates/zk-ssl/src/instrumento_pago.rs:235`). El §501 lo dejo para la
    cola del RFC-0003, y el RFC no lo recoge. El «nadie nunca» de arriba dura
    `2^32 - 2` epocas del registro: milenios a mil pagos por jornada, unos
    cincuenta dias a mil operaciones por segundo (§501).
  - **Fleco 2 de la v2: el receptor no ve el Δ.** El sobre X le llega opaco
    (RFC-0003 `:109-112` y `:202-203`; ⚠️ §827: decía `:186-187`, que las marcas del §861 en el RFC corrieron), y el §119.2 queria los terminos «a la
    vista del receptor, que ve al recibir cuando puede vencer lo suyo»
    (`AUDITORIA.md:9667-9669`; ⚠️ §861: decía `:9632-9634`). El RFC lo decidio sin decir que se apartaba del
    §119.2. Lo mas cerca es la prueba del pago en curso del RFC-0008 (E2): el
    pagador puede probar `nacido + delta >= T` sin abrir Δ
    (`spec/rfc/0008-pruebas-portables-del-pendiente.md:50`), pero solo si
    quiere darla; al recibir, el receptor no ve nada.

  ⚠️ Y la cita «(§342)» de arriba no se puede seguir: el §342 del registro trata
  la cabecera de `mint` (`AUDITORIA.md:26608`; ⚠️ §861: decía `:26573`), no los metadatos del reembolso,
  y el RFC-0003 la repite (`:11`, `:22`). Se ficha; no se corrige aqui. ⚠️ §861: corregida con marca,
  arriba y en el RFC-0003, que la repite cuatro veces (`:11`, `:22`, `:218` y `:286`; ⚠️ §827: decía `:202` y `:270`, sus líneas antes de que las marcas del §861 las corrieran), no dos.

  ⚠️ **§861: decidida por delegación del autor («resuelvelas aplicando los principios»), con el
  criterio del §609 y el precedente del §642; REVERSIBLE; entra con su aceptación. Sigue ABIERTA,
  con la decisión escrita residuo por residuo, y nada se enmienda**: aplicar no es retirar el
  suelo (`PRINCIPIOS.md` §4.3, del §119.8), ni dar por buena la omisión del §119.4, ni quitar la
  reversión a los v1 que nacieron con ella (§119.6), ni abrir `X` al receptor contra la D2 del
  RFC-0003, ACEPTADO.
  (1) **El «nunca», de verdad, en el juez.** Un delta abierto congruente con `u64::MAX` módulo p
  —`2^32 − 2` o `u64::MAX`— no vence nunca, en `apply_refund` y en `apply_deissue`
  (`crates/zk-ssl/src/two_phase.rs:687-697` y `:452-461`). Es la D1 del RFC-0003 tal como está
  escrita (`spec/rfc/0003-compromiso-v2.md:76-78`) y no mueve un byte del sobre, de los vectores ni
  de la AIR. A 248 op/s (§229), el «nunca» de hoy dura unos 200 días (deducido), menos que el
  suelo. Un Δ explícito de `2^32 − 2` pasa a «nunca», y se declara: el compromiso no distingue los
  dos valores y el §119.8 manda «largo, nunca corto». A mil operaciones por segundo, esos `2^32 − 2`
  son unos 50 días (deducido, la cifra del §501), una elección finita plausible; por eso el camino
  honrado no la compone: `send_materials_v2` (`crates/zk-ssl/src/client.rs:201`) rechaza `delta =
  2^32 − 2` y todo `delta` en `[p, u64::MAX)`, y deja `u64::MAX` como el centinela. Es orientativo
  —quien compone su propio `X` se lo salta—, y el comentario de `crates/zk-ssl-cli/src/pago.rs:11-13`
  se marca. Lo construye un sello de la capa, con su test discriminante sobre la función pura del
  plazo; ninguna de las seis superficies del RFC-0021 cambia.
  (2) **Congelados: se cumple el §119.4 en la capa** —receptor congelado ⇒ `AccountFrozen`, en
  `apply_refund` y en `apply_deissue`—, igual que en el cobro
  (`crates/zk-ssl/src/client.rs:153-157`). El §178 lo dejó fuera sin citarlo
  (`doc/CADUCIDAD_PENDIENTE.md:65`). `RefundReceipt` y `DeissueReceipt` ganan `(receiver_id, salt)`,
  y la capa recompone C1 y busca la cuenta por identidad. Su coste se declara: el recibo lleva en
  claro la identidad del receptor y la sal, que la fila del RFC-0009 D-B para la apertura del
  reembolso y de la des-emisión pone en «no sale»; la fila se marca en su sello. Hoy no expone nada
  nuevo: `refund`, `refund_v2` y `deissue` ya reciben las dos dentro de la capa (`two_phase.rs:539`,
  `:593` y `:372`). Y el reembolso por el cable, cuando se abra (ARQ-02, `doc/blueprint-v2.md:390`; ⚠️ §827: decía `:388`, que el §815 corrió),
  no hereda este recibo: allí rige la letra del §119.4, `frozen_root` en circuito, que no revela al
  receptor. Un receptor cuya identidad ya está entre las retiradas —recuperado, la 52— no tiene
  cuenta que encontrar: su sello lo decide con un test, «el receptor tiene identidad retirada»;
  coherente con la D-5 de la 52 (`doc/ROTACION_DE_CUENTAS.md` §6), se trata como sin cuenta y se
  declara esa excepción al §119.4, y si se eligiera rechazar, la D-5 diría «y si el receptor estaba
  congelado, para siempre». El §119.4 nombra «el circuito de reversión»: llevarlo a la des-emisión
  es una extensión por analogía, y se dice. Retener fondos de terceros mientras dure la congelación
  es el coste que esta entrada ya declara. La parte de circuito —`frozen_root` en `RefundAir` y
  `RefundAirV2`— es T3b-circuito (c), de la sesión que lleva las AIR. Ninguna de las seis
  superficies cambia.
  (3) **El suelo no se retira** (`PRINCIPIOS.md` §4.3) **ni se escribe en entradas** (el §119.3 lo
  descartó). Su unidad ya la eligió el §119.3: cabezas al latido. El árbol cuenta el número, firmado
  —`mmr_t` (`crates/zk-ssl/src/log.rs:640-642`), una hoja por latido
  (`crates/zk-ssl-node/src/latido.rs:473`; ⚠️ §827: decía `:455`, que el §815 corrió)—, pero en el nodo, no en el juez (`two_phase.rs:676`), y
  el ritmo no va firmado: la cadencia es una bandera del operador, `--latido`
  (`crates/zk-ssl-node/src/main.rs:236-239`, 60 s por defecto, `latido.rs:72`; con `0` no hay
  latido), y ni ella ni la hora entran en la cabeza (`EpochHead`, `log.rs:613-680`; ⚠️ §827: decía `:613-676`, que el §815 alargó) ni en
  `params_digest` (`crates/zk-ssl-hash/src/lib.rs:972-994`). Con `--latido 1`, los unos 569.790
  latidos de trece meses duran unos 6,6 días (deducido). Se pospone a **un RFC del reloj de la
  caducidad**, que tiene que firmar la cadencia, decir qué evidencia deja acelerarla —hoy solo el
  reloj de un testigo, y no hay ninguno—, resolver `--latido 0` y la capa usada sin nodo, y
  desambiguar con marca «latidos de `log.seq`», que son entradas (`crates/zk-ssl/src/lib.rs:229` y
  la (i) de arriba). Si pide el corte `zkssl/0.5`, entra PROPUESTO un sello antes del corte (la R5
  del RFC-0021).
  - **Condición doble**: el Δ del emisor sigue en épocas del registro —la D1 intacta, y con ella la
    prueba del pago en curso— y el suelo va en latidos, unos 569.790 a 60 s (deducido). Se declara
    lo que eso deja: para el Δ sigue en pie el defecto del §119.3 —en un libro quieto un Δ en
    entradas puede no vencer nunca, que es dinero muerto por quietud—, y el emisor solo controla el
    calendario a través del suelo. «Funciona a cualquier ritmo» vale para el suelo, no para el Δ.
    Queda escrita para el autor la opción de un Δ en latidos para los pendientes nuevos, que sí
    enmendaría la D1.
  - **Vale para todo lo vivo**: un v2 con Δ menor que el suelo —la conformidad usa 96,
    `spec/rfc/0003-compromiso-v2.md:262-263` (⚠️ §827: decía `:246-247`, que las marcas del §861 en el RFC corrieron)— y todo v1 con meta revierten más tarde de lo que
    decía su Δ o su T, y `expiry = born + Δ` (la D1, `:67`) deja de valer por debajo del suelo. No
    es cambiar términos a posteriori: el suelo es política publicada desde el §119.8, anterior a
    todo pendiente reversible —el mecanismo es del §178, y la v2, del RFC-0003, desde el §345—,
    aunque a `PRINCIPIOS.md` §4.3 no llegó hasta el §779; y restringe la D1 sin derogarla, en el
    sentido «largo, nunca corto».
  - `born` en latidos, atestiguado (la lección del §388); la T de la vía v1, la de su nacimiento,
    con la misma condición doble.
  - **La reversión por el cable no se abre sin el suelo**: hoy el nodo solo compone v1, con T = 64
    entradas (`crates/zk-ssl/src/lib.rs:231`), unos 0,26 s a 248 op/s (deducido), y abrirla antes
    incumpliría el §119.8 («Un Δ≈0 sería un pago que nunca fue promesa»).
  **El suelo se cumple** cuando un test haga pasar un reembolso de `RefundTooEarly` a `Ok` solo
  porque pasan latidos, sin operaciones, y otro haga rebotar un Δ por debajo del suelo.
  (4) **Vía v1.** El «ni techo» ya lo decidió la D-4 del §350 (`AUDITORIA.md:26994-26997`), que esta
  entrada no citaba. El juez v1 se queda para lo que ya existe: quitarlo sería cambiar términos a
  posteriori (§119.6). Lo que cambia es que **se deja de producir v1**: hoy el `zkssl_sendMaterials`
  del nodo solo compone v1 (`crates/zk-ssl-node/src/main.rs:3900`, por `send_materials_at`, `:3921`; ⚠️ §827: decía `:3799` y `:3820`, que el §815 corrió)
  y todo mint-a-pendiente nace v1 (`two_phase.rs:1699`). El cambio va, con la T-A, al RFC que abra
  la reversión por el cable (ARQ-02) o al corte de formato, el primero que llegue; si es el corte
  `zkssl/0.5`, también PROPUESTO un sello antes. **D5, aceptada**: `set_refund_ttl` pasa a
  `cfg(test)`; lo hace un sello de la capa, aún sin asignar. Sus llamadores son doce, todos en
  módulos de test —el decimotercero que se contaba es su definición, `two_phase.rs:257`—, y así el
  operador deja de mover en ejecución la T de lo enviado por v1, sin tocar el formato; no contradice
  el «se revisa con datos» del §178, porque la T se revisa por release.
  (5) **Fleco 2: no se construye, se declara.** La D2 del RFC-0003 hizo `X` opaco por la privacidad
  de `f` y se apartó del §119.2 sin decirlo. Lo que tiene el receptor es el suelo público, cuando
  exista, y la prueba del pago en curso del RFC-0008 E2, si el pagador se la da.
  (6) **T3b, partido como en el §128.1.** (a) y (b): los cubre la capa con pruebas reales
  (`la_frontera_de_t_es_nacido_mas_delta`, `crates/zk-ssl/src/instrumento_pago.rs:199`). (c): llega
  con (2), y la des-emisión, por la analogía declarada. (d): nativo, `t3b_v1_no_es_reversible`
  (`crates/zk-ssl/src/pending.rs:601`), y en circuito, que es lo que pide el §119.7,
  `v2_proof_does_not_open_v1_commitment` (`crates/stark-experiment/src/circuit_refund_v2.rs:526`).
  (e): espera al RFC de (3). (a) y (e) no se escriben en circuito: el RFC-0003 pone `born` y `now`
  fuera, atestiguados (`spec/rfc/0003-compromiso-v2.md:46-49`), que es la rama «condicionada, no
  caída» del §119.7. ⚠️ Además, el T3b-nativo no toca ningún reloj: `ahora` es una constante y su
  `assert!` es una precondición (`pending.rs:588-589`), aunque el §128.1 diga «con el reloj en
  cabezas firmadas» (`AUDITORIA.md:10538`).
  (7) ⚠️ **Medido por lectura, nuevo.** El RFC-0003 dice que `f` «puede ser dedicada»
  (`spec/rfc/0003-compromiso-v2.md:60` y `:194-196`; ⚠️ §827: decía `:178-180`, que las marcas del §861 en el RFC corrieron), pero la capa solo acredita a la cuenta de
  `meta.sender_index` y exige que su `public_id` sea `f` (`two_phase.rs:710-714`). Con una clave de
  retorno dedicada, el reembolso v2 rebota siempre (`PendingMismatch`), y si el receptor no cobra,
  el pendiente vuelve a ser dinero muerto. Lo mismo tras una rotación en su sitio (la D-5 de la 52,
  en `doc/ROTACION_DE_CUENTAS.md` §6): la recuperación cambia la identidad de la cuenta en su
  posición (`crates/zk-ssl/src/recovery.rs:94`), y lo que esa cuenta envió por v2 antes ya no le
  vuelve; DEDUCIDO, y lo hace medido un test del sello de la 52. Falla cerrado: es una divergencia
  de disponibilidad, no de solidez. Se ficha para la cola del RFC-0003.
  (8) **Las citas.** La «(§342)» de arriba se corrige con marca, aquí y en el RFC-0003, que la
  repite cuatro veces, no dos. Las nueve citas por línea a `AUDITORIA.md` de esta entrada caían
  corridas desde que entraron, en el §779, y van corregidas con marca donde están.
  **Zonas.** (1) y (2) tocan `apply_refund` y `apply_deissue`, cuyas llamadas `verify::<…>`
  (`two_phase.rs:485`, `:506`, `:747`, `:768` y `:789`) son las que el juez único tiene que reunir:
  la 117; el orden se coordina con ella. T3b-circuito (c) y la T-A, si toca `circuit_mint_pending`,
  son de la sesión que lleva las AIR.
  **La entrada se cierra** cuando el suelo se cumpla así, con las dos piezas de la capa, (1) y (2),
  y T3b-circuito (c) ya hechos.
  ⚠️ §827: **el RFC del reloj de la caducidad, PROPUESTO**: el RFC-0023
  (`spec/rfc/0023-el-reloj-de-la-caducidad.md`). Escribe lo que la nota del §861, de arriba, fija en
  su (3) y su (4), sin volver a razonarlo, y toma lo que esa nota le pide: la cadencia, firmada como
  octavo parámetro de `params_digest`, un `u64` en segundos con `0` para el libro sin latido, y
  fija en la vida del libro, con la que trece meses son 569.790 latidos a 60 s (deducido); la
  evidencia de acelerarla, una cuenta de latidos desde una cabeza con fecha acotada por abajo frente
  a la marca de tiempo de un ancla cofirmada (RFC-0012, RFC-0013), y alguien que feche las cabezas a
  tiempo —dos anclas comparadas entre sí darían un ROJO fabricable contra el operador honrado—, y
  hoy no hay ninguna de las dos cosas; y sin latido —con cadencia `0`, sin `--clave`, en la capa sin
  nodo, sin cuenta conocida y en PARADA, en la que desde el §815 el latido no compone ni firma— el
  suelo no vence. La cuenta del juez no es el `mmr_t` que se pasa a `epoch_head`: la mueve solo una
  cabeza firmada, y no retrocede. Las hojas de meta nuevas son dos, una por dominio. Lo nacido antes,
  que no tiene `born` en latidos, cuenta su suelo desde el `mmr_t` de la primera cabeza que firma la
  cadencia, que nunca da un suelo más corto que contarlo desde el nacimiento; y declara su coste,
  que la razón de la (3) no cubre: lo retrasa también por encima del suelo contado desde el
  nacimiento, hasta un suelo entero desde esa cabeza, y eso toca el §119.6. De la (6), la (e) de
  T3b: para el suelo la toma su D-J, con el test que ata el número que juzga al de una cabeza
  firmada; atestiguar ante un tercero el `Δ` abierto, que la marca del §861 en el RFC-0003 deja a
  ese RFC, queda como pregunta abierta para el autor, y la marca lleva la suya. También para el
  autor, el Δ en latidos para los pendientes nuevos, que enmendaría la D1. Si viaja en el tren lo
  acuerda quien lo lleva, y queda por coordinar. «Latidos de `log.seq`», en la (i) de arriba y en
  el comentario de `DEFAULT_REFUND_TTL`, son entradas, y lo dicen sus marcas. Y las seis citas por
  línea al RFC-0003 de esta entrada que las marcas del §861 corrieron, en cinco sitios, y las siete
  al código y a `doc/blueprint-v2.md` que el §815 y el §816 corrieron, en seis, llevan la suya.
  Rompe lo firmado y va al corte; ninguna etapa de código construida —la E1 es el texto del RFC—:
  **sigue abierta**, y se cierra como dice la nota del §861.
- [x] **13. Senal temporal para el pagador: ya declarada, coherente.**
  ~~Puede recomputar el compromiso y ver cuando se cobra; declarado, no
  eliminado.~~ **Cerrada** el 30-07-2026: verificado que ya esta declarada
  en **tres** puntos de AUDITORIA (§175, §223, §258) y en los **tres
  preprints**, y los cuatro coinciden —el pagador sabe *cuando* se cobra,
  no *cuanto*—. A diferencia de la 9, no habia contradiccion que
  reconciliar. El residuo es del diseño en dos fases y **no se elimina**;
  esta declarado, que es lo que pedia la entrada.

- [x] **14. Techo del conjunto de custodios: ya declarado.** ~~Acoplado a
  un segmento de rango de 7 bits, sin declarar hasta que un test lo fijo.~~
  **Cerrada** el 30-07-2026: al verificar el codigo para redactarla resulta
  que **ya estaba declarada** —AUDITORIA §14 con tabla, y el test
  `the_custodian_set_size_fits_the_range_segment` que fija el acoplamiento
  `CUSTODIAN_DEPTH`/`SEGMENT_LENGTH`—. Correccion de imagen fiel: no son
  «128 custodios» sino **16 hoy** (`CUSTODIAN_DEPTH=4`); 128 es el techo
  teorico del segmento de 7 bits, que solo mordería subiendo la profundidad
  a 8 sin ampliar el segmento. Es limite de **disponibilidad**, no de
  solidez. No faltaba declararla: faltaba que este backlog no la describiera
  como pendiente.

- [x] **15. ✅ CERRADA. El espacio de claves: capacidad completa, adopcion
  opt-in.** Cerrada el 01-08-2026 (§102). ✅ **Capacidad**: los cinco
  circuitos de gasto (§92), la puerta `open_account_wide` (§97), **un pago
  completo con claves de 256 bits medido de punta a punta** (§97.3) y **la
  rotacion de una cuenta estrecha a ancha, medida** (§98.2). ⚠️ **La adopcion
  es opt-in por diseño**: quien use `open_account` sigue con 64 bits, y la
  via esta marcada `#[deprecated]`. Se intento migrar los 159 tests con una
  linea y **fallaron 59** —dos patrones mecanicos, ninguno del camino ancho—;
  **se revirtio** porque no gana un solo bit y porque **que 159 tests usen la
  via estrecha es cobertura, no deuda**: esa via sigue viva y alguien tiene
  que probarla (§102.2). ⚠️ **Migrar los tests sera necesario al retirar la
  via estrecha, que es la entrada 32.**
  ~~15. El ESPACIO DE CLAVES es de 64 bits. Medido, e INVENTARIADO el
  arreglo.** ⚠️ **Inventario y plan en §85** (31-07-2026): las ocho
  derivaciones son una sustitucion cada una y el nulificador **no** cambia de
  forma; en el estado de Rescue **hay sitio** —la clave ocupa la ranura 8 de
  12—. Lo que cuesta, contado sobre `circuit_settlement`: `TRACE_WIDTH`
  **49→52** y `NUM_CONSTRAINTS` **155→170**, o sea **+3 columnas y +15
  ranuras por circuito** —son +15 y no +6 porque la clave entra **dos veces**,
  para `pk` y para el nulificador—. Churn: **88 usos** de `derive_public_id`
  en 22 ficheros y **122** `BaseElement::new(SK_*)`, mecanicos.
  ⚠️ **«Empezar por un circuito» NO funciona** (§85.4): un formato de
  identidad no se migra por partes. ~~**Plan decidido**: medir el coste con
  columnas de relleno sobre `circuit_settlement` —que la capa no ejecuta—,
  y con ese numero hacerlo entero en un commit.~~ ✅ **Coste MEDIDO** (§86):
  **−2,7 % de tamaño y −12,5 % de tiempo** — ensanchar no encarece, abarata.
  ⚠️ **Y B no es un commit, son DOS** (§85.7): los espacios de identidad se
  separan —**gasto** en 5 circuitos, **custodios+gobernanza** acoplados por
  `build_custodian_set` en 8—, y ningun circuito mezcla los dominios. Cada
  commit deja el arbol verde. **Piloto**: `circuit_settlement`, que es de los
  cinco de gasto y cuyo Air la capa **no ejecuta**, asi que se ensancha solo
  sin meter dos formatos en produccion (§85.8). **No queda analisis
  pendiente: solo escribirlo.** ✅ **PASO 1 HECHO** el 31-07-2026:
  `derive_public_id_wide` y `native_nullifier_wide` añadidas **junto a** las
  estrechas, sin tocar ninguna firma. ⚠️ **Y rectifica una premisa que se
  venia arrastrando** (§90): ensanchar **NO invalida ninguna cuenta** —
  rellenar con ceros da la MISMA identidad, hay test—. La migracion pasa de
  **reapertura forzosa** a **rotacion gradual de claves**, y el argumento «no
  hay despliegue, asi que invalidar es gratis» deja de hacer falta —habria
  envejecido mal con un piloto—. ⚠️ **Conservar la identidad no conserva la
  seguridad**: quien no rote sigue con 64 bits. ~~**Paso 2**: ensanchar
  `circuit_settlement` (~30 sustituciones).~~ ✅ **PASO 2 HECHO** el
  31-07-2026 (§92): `circuit_settlement` opera con `sk` de **cuatro
  elementos**. `TRACE_WIDTH` **49→52**, `NUM_CONSTRAINTS` **155→170**,
  aserciones **57→51**. **18 tests en verde**, 274 y 201 sin regresion, y
  **`no_constraint_is_vacuous` pasa**: las quince ranuras nuevas disparan
  todas. ⚠️ Hubo que retirar seis aserciones que fijaban a cero `state[9..12]`
  —**eran relleno y dejaron de serlo**—; pasan de estar fijadas a CERO a
  estarlo a la CLAVE, que es mas fuerte (§92.2). ~~**Queda**: los otros cuatro
  de gasto + la derivacion compartida + `open_account` en un commit, y
  custodios+gobernanza en otro.~~ ✅ **Y `circuit_burn` tambien** (§92.6):
  `TRACE_WIDTH` **39→42**, `C_KEY_INPUT` **2→8**, `C_TRANSPORT` **7→10**, 11
  tests en verde. Sin aserciones que retirar —se comprobo antes—.
  ⚠️ **Costo CINCO rondas** por aplicar el patron en el orden equivocado
  (§92.7): **primero los TIPOS, luego compilar, y arreglar lo que el
  compilador enumere**. Esa es la secuencia para los que quedan.
  ⚠️ **Y rompio `main`**: ensanchar `circuit_burn` rompio su llamante en la
  capa —`zk-ssl/src/burn.rs`— **y se commiteo sin ver la salida** (§92.8).
  Arreglado con **una linea**, rellenando la clave en el borde, gracias a la
  propiedad de §90 (§92.9). ⚠️ **Los tres que quedan SI los ejecuta la
  capa**, asi que romperan su llamante igual: la secuencia completa esta en
  §92.10.
  ✅ **Y `circuit_send` tambien** (§92.11): `TRACE_WIDTH` **49→52**,
  `C_KEY_INPUT` **2→8**, `C_TRANSPORT` **15→18**, 17 tests en verde. **Una
  ronda para el circuito** aplicando §92.7 —los tipos primero—, frente a las
  cinco de `burn`. ⚠️ **Y `check_constraint_layout.py` cazo un fallo real**
  (§92.12): desplazamientos a mano que asumian el array de transporte viejo,
  con tres ranuras escritas dos veces y tres muertas. ⚠️ **Rellenar en el
  borde vale para la capa y NO para el cliente** (§92.13): `prove_send`
  cambia su firma a `Digest`, porque rellenar ahi dejaria el trabajo sin
  efecto.
  ⚠️⚠️ **HALLAZGO QUE REORDENA LO QUE QUEDA** (§92.14): los 256 bits estan en
  tres circuitos y **ningun cliente puede usarlos** — `open_account` solo
  crea cuentas con clave de 64, asi que la clave ancha **no es alcanzable
  desde la capa**. Lo destapo `a_whole_payment...` rechazando con
  `NotTheAccountHolder`, y el circuito tenia razon.
  ⚠️⚠️ ~~**Quedan, en este orden**: `open_account` y la capa PRIMERO.~~
  **RECTIFICADO el 01-08-2026 (§96): `open_account` va el ULTIMO.** Se conto
  cuantas operaciones del titular funcionarian con una clave ancha de verdad:
  **una de siete**. Podria **enviar** y no cobrar, ni quemar, ni auditarse
  —esas vias derivan estrecho—. Y en un sistema de dos fases, **enviar sin
  poder cobrar deja el dinero en un pendiente inmovilizado** (entrada 12).
  ⚠️ **Migrar `open_account` ahora crearia cuentas que pierden fondos**, y el
  cambio es de una linea y parece progreso (§96.5).
  ✅ **Y `circuit_claim` HECHO** (§92.15): `TRACE_WIDTH` **48→51**,
  `C_KEY_INPUT` **2→8**, `C_TRANSPORT` **15→18**. Una ronda para el circuito,
  27 circuitos limpios al primer intento, y los desplazamientos a mano
  —reincidentes por tercera vez— corregidos **de entrada**.
  ⚠️⚠️ **CUATRO circuitos migrados y CERO bits de seguridad ganados**
  (§92.16): en el test del pago completo **las dos claves van rellenadas con
  ceros**, porque `open_account` deriva estrecho. **Un lector que vea «cuatro
  de cinco» concluira que esto esta casi hecho; la parte que da seguridad no
  ha empezado.** Es lo previsto en §96 — la seguridad entra toda de golpe al
  cerrar la puerta.
  ✅ **Y `circuit_audit` HECHO** (§92.17): el analisis previo encontro que
  **es de UN SOLO CARRIL** —no absorbe la clave «de otra forma», la absorbe
  igual pero **una vez**, porque audita en vez de transitar—. `TRACE_WIDTH`
  **24→27**, `C_PK_INPUT` **1→4**, `num_assertions` **20→17**: la mitad de
  coste que los de dos carriles. ⚠️ Aqui **si** habia aserciones que retirar
  —tres, sobre `ROW_PK_START`—, comprobado **antes** de escribir.
  ✅ **LOS CINCO CIRCUITOS DE GASTO MIGRADOS**: `settlement`, `burn`, `send`,
  `claim`, `audit`. 274 y 201, cero avisos, 27 circuitos limpios.
  ✅ **LA PUERTA, ABIERTA** (§97): `open_account_wide(Digest)` añadida, y un
  **pago completo con claves de 256 bits medido de punta a punta**. 202 y
  274, cero avisos. ⚠️ **Primero se hizo mal** (§97.1): se estimo «~22
  llamadas» y son **115** —§80.2 literal, del mismo dia—; los errores pasaron
  de 15 a 85 y se tiro de un `regex` que hizo **18 sustituciones sin verlas**.
  Revertido. El diseño correcto era **el que §85.5 descarto**, y su objecion
  resulto falsa con §90 medido.
  ⚠️ **DEUDA DECLARADA** (§97.4): la migracion es **opt-in**; quien use
  `open_account` sigue con **64 bits**. **Falta**: retirar o marcar la via
  estrecha —familia de la 32— y que las cuentas **roten**.
  ⚠️ **La derivacion estrecha NO hay que migrarla en los 22 ficheros**: §90
  la hace equivalente para una clave rellenada, asi que quien solo *calcula*
  una identidad puede seguir usandola.
  ~~Goldilocks es estrecho para identidades: 64 bits son colision en 2³².~~
  ⚠️ **Eso describia un problema YA CORREGIDO** —la identidad paso a ser el
  digest de 4 elementos, 256 bits, y esta documentado en la cabecera de
  `circuit_settlement.rs`—. **Lo vivo es otra cosa** (§82): `sk` sigue
  siendo **un solo elemento** —`open_account(spend_key: BaseElement)`— asi
  que el espacio de secretos es 2^64, y `pk` es **publica** porque el pagador
  la necesita para direccionar. El ataque es **busqueda exhaustiva fuera de
  linea**, no una colision, y ensanchar la identidad **no ayuda nada** contra
  el. **MEDIDO** el 31-07-2026 con
  `el_coste_de_agotar_el_espacio_de_claves`: **122.850 derivaciones/s por
  nucleo**, o sea **2,38 millones de años-nucleo** para 2^63 — 23,8 años con
  100.000 nucleos, 87 dias con diez millones. ⚠️ **Cota superior floja**: es
  CPU sin optimizar el ataque. ⚠️ **Y va con la 28 y la 16**: `PAPER.md` §8.3
  llama al techo de 63 bits de solidez «insuficiente y no comparable con los
  ~128 bits de los otros paradigmas» —**el mismo criterio se aplica al
  espacio de claves y el paper no lo dice**, asi que quien lo lea concluira
  que la estrechez de Goldilocks tiene una sola consecuencia (§82.4)—.
  **Arreglo dimensionado en §82.5**: `sk` de 4 elementos, que invalida
  cualquier cuenta existente.

- [ ] **16. ⏸️ SUSPENDIDA. Referencias cruzadas — y NO se van a mantener.**
  ⏸️ Suspendida con la 28 el 01-08-2026, y con una decision de diseño:
  **cada paper sera independiente**, sin citar a sus companeros. Eso no
  resuelve la entrada: **la elimina**.
  ⚠️ **Y la razon por la que no se puede mantener es estructural**: los DOI
  de una version **no existen hasta despues de depositarla**, asi que toda
  revision nace con sus referencias apuntando a la anterior. No es un
  descuido que se corrija una vez: **se rompe en cada revision**.
  ⚠️ **Se descubrio ademas un fallo de deposito del 29-07-2026**: las v2 y v3
  de `ZK-SSL-policy-note` se subieron **dentro de la cadena de versiones de
  `ZK-SSL-residual-trust`**. Por eso `doc/preprints/README.md` atribuia
  `…693709` a `policy-note` cuando es **v2 de residual-trust**. Esta
  publicado y **no se puede deshacer**: solo documentarse.
  ~~16. Referencias cruzadas: CORREGIDAS.~~ Las siete el 01-08-2026
  (§101.2): cinco cruzadas **y dos autocitas** —`ZK-SSL-preprint.md` y
  `ZK-SSL-policy-note.md` se citaban a si mismos con su DOI de primera
  revision—, que la entrada no inventariaba. Cero DOI de primera revision en
  `doc/preprints/`.
  ~~16. Referencias cruzadas de los preprints.~~ Los tres citan
  versiones anteriores de sus companeros; primera cosa de la cuarta
  revision. **Comprobable** contra los propios ficheros de `doc/`, sin
  necesidad de seccion. ⚠️ **La cuarta revision ya no es solo esto**: se le
  han acumulado la 28 —el cobro—, la 15 —el espacio de claves de 64 bits,
  que §82.4 muestra que el paper no menciona pese a usar el criterio de los
  128 bits— y la unidad MiB/MB de la 22.

- [x] **75. ✅ (§795, §822) El coste de la suite: dos tercios del nivel de sello, y con
  dos causas distintas.** Medido en los bancos T.1 a T.4 y asentado en
  §263. En el canon `--sello`, `settlement-layer` cuesta **50 s** e
  `iso-bridge` **45 s** de los **150 s** del nivel.
  ⚠️ **No es la misma causa en los dos, y confundirlas mandó un banco al
  crate equivocado.** En `settlement-layer` son los **dos setups Groth16**
  de `SettlementLayer::new`, que pagan los doce tests sin excepción — esa
  capa no usa el árbol denso: sus dos árboles son `SparseMerkleTree<20>` y
  `<32>`. En `iso-bridge` sí: `zk-core/src/merkle.rs:67-84` rellena hasta
  2^20 hojas y hashea el árbol entero, **1.048.575 hashes para usar ocho**.
  **Comprobable**: `grep -n "leaves.resize(target_len"
  crates/zk-core/src/merkle.rs`. ⚠️ §822: desde el §822 ese grep no da nada, y
  las líneas 67-84 de `merkle.rs` que cita arriba eran el `build` de antes
  del §822: hoy son otras, y ya nada rellena, porque el árbol es disperso,
  abajo.
  ⚠️ **La constante no está mal**: 2^20 cuentas es lo correcto en
  producción. Lo que está mal es pagarlo en un test. Y **el arreglo ya vive
  en el repositorio**: `settlement-layer/src/sparse_tree.rs` es el mismo
  árbol, disperso, para profundidades 20 y 32, y sus cinco tests cuestan
  **0,1 s**. Las dos implementaciones llevan conviviendo sin que nadie las
  pusiera una al lado de la otra.
  **No es un cambio de una línea**: bajar `TREE_DEPTH` rompería la
  conformidad, así que el arreglo es **no construir denso**, ~~y eso toca el
  circuito~~ (⚠️ §822: no lo toca; los caminos son los mismos, abajo). Sin
  urgencia declarada: cuesta minutos de suite, no corrección.
  ⚠️ §795: **la mitad de `settlement-layer`, hecha; la de `iso-bridge`, no.**
  Los doce tests que arrancan una capa comparten ya las claves de los dos
  setups: `new_layer()`, en el módulo de tests de
  `settlement-layer/src/lib.rs`, guarda en un `OnceLock` la capa que arranca
  el `SettlementLayer::new` de producción, una sola vez, y da a cada test una
  copia campo a campo. **Ni una línea de producción tocada.** Compartir no
  cambia lo que se prueba: la semilla es 42 en los doce y `setup_generic`
  siembra con ella un `ChaCha20Rng`, así que las claves ya eran las mismas.
  Comprobado además, no sólo leído: un test temporal, retirado después,
  comparó las cuatro claves serializadas de la plantilla con las de un `new`
  fresco y salieron **iguales byte a byte** (13.091.760 B la de
  transferencia, 4.020.384 B la de emisión). Ningún test pide claves frescas.
  **Medido** con `nice -n 19 cargo test --release --locked -p
  settlement-layer`, el binario ya compilado, segunda de dos corridas: CPU
  (user+sys) **206,1 → 92,5 s**; pared **102,0 → 65,1 s**, con la máquina
  cargada (carga de 5,5 a 10,6 en 4 núcleos), así que el dato bueno es el de
  CPU. Los 17 siguen pasando y siguen siendo 17. Lo ahorrado, ~114 s de CPU,
  son once pares de setups —`setup_settlement` y `setup_mint`— de ~10 s de
  CPU cada par: `ark-groth16` compila aquí con
  `parallel` (rayon, de serie), y un setup gasta en CPU varias veces su pared
  —cuadra con los ~2,4 s de pared del §263 en cuatro núcleos, sin medirlo
  aparte—. **Sigue abierta** por la otra causa: el árbol denso de
  `iso-bridge`, que no se ha tocado.
  ✅ **CERRADA EN EL §822: la de `iso-bridge`, hecha**, y con la de
  `settlement-layer` (§795) quedan resueltas las dos causas. Decidido por
  delegación del autor, que encargó hacerlo todo (lo recogen el §814, el §816
  y el §817) y, en su mensaje a la sesión, que el registro no trae, dejó fuera
  un tercero que implemente (H3), un auditor (H7), los testigos reales y los
  avisos a NLnet: la 75 no es nada de eso. Con el criterio del §609: «que se
  aplique al mayor número de casos de uso sin modificaciones significativas».
  `SimpleMerkleTree` (`crates/zk-core/src/merkle.rs`) ya no materializa el
  relleno: el nivel `k` guarda solo los nodos que cubren alguna hoja dada, y
  cualquier otro es `vacios[k]`, el hash de un subárbol vacío de altura `k`;
  `vacios[0]` es el relleno de siempre, `F::zero()`. Con ocho hojas, 44 hashes
  en vez de 1.048.575: 20 de los subárboles vacíos y 24 de los nodos que
  cubren hojas. **Contados**, el 20-10-2026 y otra vez el 21-10-2026, en el
  arnés de una revisión que copia el `build` tal cual y cuenta las llamadas
  al hash: 20 con ninguna hoja, 40 con una, 43 con cinco y 44 con ocho; con
  el árbol lleno, 1.048.595, veinte más que el denso. **No toca el
  circuito**: un subárbol vacío tiene ese hash lo calcule quien lo calcule,
  así que la raíz y los caminos son los del árbol denso, y ni el circuito, ni
  `TREE_DEPTH`, ni ningún llamador cambian: los tests de `zk-core`, su
  ceremonia, las dos medidas de `performance.rs` y el test de `iso-bridge`
  siguen llamando a `build`, `root` y `path_for` igual. **Lo atan tres tests
  nuevos, con un KAT sacado del denso antes de cambiarlo**: medido el
  15-10-2026 sobre `0315b14` (su `zk-core` es el de `4d8017d`), un test
  temporal, que no se queda, imprimió con el `build` denso la raíz y los
  caminos —hermanos y bits— de los índices 0, 3, 4, 100 y 2^20 - 1 en cuatro
  casos —ocho hojas, cinco, ninguna y una— (213,14 s en `--release`), y el
  mismo test sobre el disperso imprimió lo mismo byte a byte (md5
  `150177c071544af39422455482a2cdb9` las dos salidas). Las cuatro raíces
  quedan como constantes en `el_arbol_disperso_da_las_raices_del_denso`, que
  exige además que el camino de cada índice lleve los bits de su índice —los
  de esa salida— y suba a su raíz: con esos bits y esa raíz, otros hermanos
  serían una colisión de Poseidon. `el_camino_fuera_del_arbol_entra_en_panico`
  ata que un índice que la profundidad no alcanza no tiene camino (el denso
  caía por el índice de su vector; el disperso respondería con subárboles
  vacíos), y `el_arbol_con_mas_hojas_de_las_que_caben_entra_en_panico`, el
  `assert` de `build`, que no cambia y que ningún test pedía. La exigencia de
  los bits la pidieron dos revisiones del sello, y el test de `build`, una:
  con la primera versión del test, un `is_right` a `false` en el relleno
  pasaba, y 11 de los 20 caminos del KAT salían con otros bits que los del
  denso.
  Veinte mutantes, uno a uno, sobre el `merkle.rs` final (21-10-2026): caen
  dieciocho, y cada test nuevo tumba alguno; sobreviven dos, declarados: el
  `assert` de `build` con `<`, que rechaza el árbol lleno —solo caería
  construyendo 2^20 hojas—, y el texto del pánico con algo detrás
  —`should_panic(expected)` busca una subcadena—.
  **Medido** el 15-10-2026 en 4 CPU compartidas, con el binario ya
  compilado (`nice -n 19 cargo test --release --locked`): `iso-bridge`, de
  53,3 y 57,6 s de pared a 3,4 y 4,3, y de 54,4 y 56,7 s de CPU (user+sys) a
  5,6 y 6,6, con carga de 7,6 a 8,3; los 3 pasan. La biblioteca de `zk-core`,
  de 3.546 s de pared (73 tests, carga 8,9) a 38 y 51 (75 tests, carga 8,0 y
  10,0). La suite entera de `zk-core`, como la corre el canon: 76 de 76 en
  486 s, de los que la ceremonia se lleva 448. Con el `merkle.rs` final, el
  21-10-2026 y con menos carga (3,9 al empezar): la suite entera, 77 de 77
  en 271 s, 15 de la biblioteca y 256 de la ceremonia; `iso-bridge`, 3 de 3
  en 1,3 s. Las dos, sobre `4d8017d`; el mismo día, sobre `5fd5212` (el §817,
  medido antes del rebase, con el mismo código) con este cierre encima y más
  carga: la suite entera, 77 de 77 en 555 s (carga 7,8), 48 de la biblioteca
  y 506 de la ceremonia; la biblioteca sola, 76 de 76 en 47 s (carga 6,3);
  `iso-bridge`, 3 de 3 en 5,2 s (carga 7,7). **Lo que NO hace**: los seis
  árboles densos de `halo2-experiment`, también a 2^20 —`NativeMerkleTree` en
  su `merkle.rs`, el `NativeTree` de `compliance_real_proof.rs` y los de los
  tests de `settlement_prover_impl.rs`, `compliance_circuit.rs`,
  `circuit_double_entry.rs` e `iso_bridge.rs`—, siguen como estaban, sin
  medir lo que cuestan; y `TREE_DEPTH` sigue en 20.

- [x] **76. El store anterior a D4: una entrada, no tres — fuera de alcance, porque este código
  no lo abre (⚠️ §860).** Funde lo que
  estaba repartido —la exposición de un ledger sin sal (§259, `spec/RPC.md`),
  el «árbol mezclado» y §258-A—, porque **son tres caras del mismo sujeto**:
  un store persistido por código anterior al flip D4. Listadas por separado
  parecían tres asuntos, y eso es la mitad de por qué sobrevivieron cinco
  sellos.
  ⚠️ **Alcanzabilidad primero**: `commit()` escribe `meta:geometry_v7` en
  **cada** persistencia (`persistence.rs:578`) y `migrated` es
  `meta:migrated` **o** `meta:geometry_v7` (`persistence.rs:334` y `:339`),
  así que **ningún ledger guardado por este código puede estar en este
  caso**. Sólo entra un store de antes.
  **Las tres caras**: (1) **expuesto** — sin sal, `sendMaterials` e
  `inclusionReceipt` dejan enumerar el saldo offline; medido en §259.
  (2) **se rompe al usarlo** — abrir una cuenta escribe hoja salada en árbol
  sin sal, y al reabrir todas se reconstruyen saladas y la raíz no cuadra:
  `IntegrityFailure`. **Razonado, no medido** (§258-A, §260 entrada 7): si
  existe un ledger así, nadie ha comprobado que la cadena ocurra, sólo que el
  código dice que ocurriría. (3) **tiene herramienta** —
  `migrate_to_salted_positions`, que §157 deja como herramienta de stores
  legacy y que **nunca tuvo llamador vivo**: `git log -S` da sólo sus tres
  commits de nacimiento (`dcc2af0`, `f1c08f2`, `2b4ce43`), ninguno que quite
  una cadena.
  **Evidencia y fecha**: medido sobre el espejo #90, sello `8745702`
  (10-08-2026). Condición cumplida para todo lo demás en §156 (`2ac37c5`) y
  §157 (`e3116c6`+`f926de6`), etiqueta `entrada-50`.
  **Sin urgencia y sin plazo**: el sujeto no es fabricable por este
  repositorio. Cerrarla es decidir si el legacy se soporta o se declara
  fuera de alcance — decisión, no trabajo.
  ⚠️ **§860: decidida por delegación del autor («resuelvelas aplicando los principios»), con el
  criterio del §609 y el precedente del §642; REVERSIBLE; entra con su aceptación. CERRADA: el store
  `sled` anterior a D4 queda fuera de alcance, y no por decreto: este código no lo abre.** Lo que
  esta entrada no vio: `load` exige con `need` `root:pending`
  (`crates/zk-ssl/src/persistence.rs:519`, fail-closed desde el §387), y también `root:pmeta`
  (`:534`), `root:froz` (`:550`), `root:cons` (`:579`) y `root:log` (`:605`). **Medido** con `git
  log -S` y `git merge-base` sobre `d4d6d45`: en el código de la capa, `root:pending` se escribe
  desde `53d339e` (S387, 03-09-2026) —en la historia entera aparece antes, en `9bf4a91` (S379), para
  decir que no existe—, y las otras cuatro nacen en `b91896b`, `8a27afe`, `f74d7f9` y `09dac0d`. Las
  cinco descienden de `2ac37c5` (D4, 04-08-2026), en cuyo `persistence.rs` `root:pending` sale 0
  veces; el de `2ac37c5^` no lee ni escribe más raíz que `root:state` y `root:nullifier`.
  **Razonado, no medido contra un store real**: un store anterior a D4 para, a más tardar, en
  `Malformed("falta la clave root:pending")`, antes de servir un camino o de abrir una cuenta —puede
  fallar antes, por el formato de sus registros—. Así, la cara (1) no la expone este código, la
  cadena (2) del §258-A no llega a empezar y la herramienta (3) no lo alcanza. La clase entera,
  «anterior al §387», que contiene a ésta, la ejercita
  `a_ledger_without_a_pending_root_does_not_open`. **Por qué basta razonado**: lo que escribe el
  código de `2ac37c5^` se lee en su fuente, y la exigencia de `need` la ejercita ese test. El sujeto
  **sí se puede fabricar** con el código viejo —`git archive` de `2ac37c5^`, el precedente del
  §702—; lo que objetaba el §260.7 era una réplica hecha a mano, y arriba lo decía «no fabricable».
  Esa medida queda opcional, fuera de este cierre.
  La frase de arriba «Sólo entra un store de antes» **dejó de ser cierta en el §387** (`53d339e`):
  desde entonces ninguno llega a abrir; el store entra en la rama sin sal de `load` y falla después.
  Aquel sello cerró de hecho esta entrada sin tocarla, ni tocar la tabla del §265 de `spec/RPC.md`.
  **Lo que sigue vivo es otra puerta**: la instantánea v3..v6 se importa como mundo viejo, sin sal,
  con los congelados a 24, en memoria y sin `sled` (`crates/zk-ssl/src/snapshot.rs:366-376`, `:446`,
  `:460` y `:485`), porque el formato promete leerse dentro de diez años
  (`ARQUITECTURA.md` §«Instantáneas: un disco perdido ya no es el ledger perdido»). Ninguna vía del
  nodo la importa, y operarla pide antes `migrate_to_salted_positions`: es **su** herramienta, de
  geometría y no de privacidad —sus cuentas con sal cero siguen siendo enumerables (§126.4,
  §132.3)—.
  **Descartadas**: soportar el store anterior a D4 (sirve a cero casos: no hay libro desplegado,
  `SECURITY.md` §3.6, y fabricaría raíces sin árbitro); el mando explícito y aparte que el §387 dejó
  previsto para migrar libros anteriores a él, por su propia condición, que no se cumple —«hoy no
  hay libros de terceros»—; retirar el legacy entero (rompe la promesa del formato y la decisión del
  §36); migrar al importar (cambia lo que significa importar, sin ningún caso que lo pida); y
  posponerla (la advertencia perpetua del §265). El criterio del §609 se aplica al formato portable,
  la instantánea, que existe para no depender de `sled` y que esta decisión conserva: el `sled` es
  interno del motor, y ningún caso de uso depende de su forma anterior a D4. **Se reabre** si
  aparece un libro de terceros anterior al §387: entonces, entrada nueva por la vía explícita del
  §387. Corregidos encima, sin borrar: la tabla de
  `spec/RPC.md` §«⚠️ CORRECCIÓN (§265): el enunciado sigue en pie, el sujeto caducó», con una
  corrección nueva encima, que es prosa y no cable, como aquélla; y los comentarios de
  `crates/zk-ssl/src/migration.rs` y de `persistence.rs`. Citas envejecidas de esta entrada:
  `persistence.rs:578` es hoy `:794`, y `:334`/`:339` son `:353`/`:358`.

- [x] **78. `proof_digest` dice que ata, y en cuatro vías no ata nada.**
  **PRESENTE y MEDIDO** sobre `bb85772`, 10-08-2026. Cuatro vías delegadas
  llaman a `append` con `&[]` — `mint.rs:131`, `freeze.rs:138`,
  `recovery.rs:166` y `governance.rs:162`— así que las cuatro comparten un
  único `proof_digest`, el de la prueba vacía:
  `74de079ffffa783f99bdd9ffa25e4112f8c395b141c83325e06ad3e10625cfa1`.
  ⚠️ **Y la doc no sólo se contradice: nombra las dos que fallan.** Sobre
  `root_old`/`root_new` decía que en gobernanza y congelación «el detalle de
  lo que sí cambiaron queda atado por `proof_digest`», y son justo las dos
  que asientan con prueba vacía. §271 corrigió **la doc**, encima y sin
  borrar; esta entrada es para **el arreglo**.
  **Gradación**: `mint` y `recovery` registran la transición —sus raíces
  difieren—; `freeze` y `governance` asientan `raiz, raiz`, así que su
  entrada dice sólo «pasó algo de esta clase en este `seq`».
  ✅ **Lo que NO es**: un agujero de autorización.
  `apply_governance_delegated` verifica contra `commit_operation(
  OP_GOVERNANCE, raíz_vieja ‖ raíz_nueva ‖ count_old ‖ count_new)`, con su
  razón escrita (§56.2). El cambio **está** ligado a la transición exacta;
  lo que no llega es al registro.
  **Arreglo**: asentar el digest de la prueba que autorizó, en vez de `&[]`.
  ⚠️ **CORRECCIÓN (§273)**: son **cinco** vías, no cuatro —faltaba
  `MintToPending` (`two_phase.rs:1285`, `raiz, raiz`, `&[]`), cuyo
  `append` multilínea el grep de línea única de §271 no podía ver— y el
  hallazgo es un **corte**, no un reparto: `Send` y `Claim` —las del
  **titular**— asientan `&receipt.proof` real (`two_phase.rs:804` y
  `:999`). **Las del titular atan; las delegadas, no.** El arreglo cubre
  las cinco.
  ✅ **HECHA (§278), y con una SEGUNDA corrección al recuento**: el censo
  estructural —paréntesis balanceados, no `grep` de línea— encontró
  **seis** asientos con prueba vacía, no cinco: faltaba `accounts.rs:243`
  (`OpenAccount`). Y por el otro lado atan **seis**, no dos: además de
  `Send` y `Claim`, `burn.rs:197`, los dos `Refund` (`two_phase.rs:401` y
  `:534`) y `migration.rs:116`. La nota se quedaba corta **en las dos
  direcciones**.
  **Qué hace §278**: las cinco delegadas asientan el **sello de
  autorización** de su `commit_operation` —vivo en el punto del asiento,
  con dominio propio y versión—, y `OpenAccount` asienta el **sello de
  ausencia declarada**. Antes las seis escribían `74de079f…`, así que
  *«no demostrable por diseño»* y *«autorizada y no registrada»* eran el
  mismo byte; ya no.
  **Lo que NO cierra, con su destino**: el sello ata el **compromiso**, no
  la prueba —las de umbral se consumen al verificarlas—, así que un
  tercero puede recomputar qué autorizaba la entrada pero **no** puede
  pedir la prueba y reverificarla. Esa es la **nota 79**, que hasta hoy no
  tenía qué recomprobar. Medido de paso: la familia delegada **no tiene
  entrada RPC de producción** (sólo `dev_fund` tras `--dev` y el sandbox
  del CLI), así que el hueco se ha cerrado **antes** de que la entrada
  exista.

- [x] **79. Sin el digest real, un log replicado no puede recomprobar que
  aquello estaba autorizado.** **FUTURO y RAZONADO**, y va aparte de la 78 a
  propósito: son dos naturalezas, y juntas la 78 se leería como consecuencia
  de ésta y quedaría archivada como cosa de mañana.
  Hoy **no muerde**, y eso está medido: no existe reconstrucción que
  reverifique — los únicos `replay` del árbol son tests de rechazo de
  repetición, no de reejecución. Muerde el día que alguien construya réplica
  o verificación desde el registro.
  ⚠️ **Menciona la 17 y la 23; NO se ata a ellas.** La 17 se leyó **entera**
  antes de escribir esto: es «constatación de arquitectura, no hallazgo… no
  hay nada medido que registrar», comprobable con un `grep` que no devuelve
  nada. **No tiene plan**, y colgar una precondición de un clavo que no
  sujeta es lo que hay que evitar.
  ✅ **HECHA (§279)**, y con la medición que la nota no tenía: el
  reverificador existe en `zk-ssl-verify` —`reverificar`, veredicto por
  entrada— y el tercero puede correrlo con `zkssl_logEntries` y sin el
  nodo. Lo que hizo posible cerrarla es que la composición del sello se
  mudó a `zk-ssl-hash` (§279), donde el verificador independiente la ve
  sin arrastrar la capa ni el probador.
  ⚠️ **Y hasta dónde llega, que es la parte honesta**: la atadura sólo es
  comprobable por un tercero en **dos de seis clases** —`OpenAccount`,
  cuyo sello es constante, y `Recovery`, cuyas raíces son las del asiento
  y cuyo contador se cuenta en el registro—. `Freeze`, `Governance` y
  `MintToPending` atan raíces de árboles que el registro no lleva
  (congelados, custodios, pendientes) y `Mint` necesita importe y
  suministro. Censo del escenario canónico, contado sobre la salida del
  instrumento: **2 verificadas · 4 parciales · 0 no derivables**.
  **Lo que queda, con su nombre y su precio**: cerrar las cuatro exige
  que el compromiso viaje **en la entrada del registro**, y eso es una
  rotura del formato del log —persistencia, store, DTO, vector, spec—.
  Va a la entrada **82**, que acumula las razones de rotura de formato
  para que el día que se rompa se rompa **una vez para todas**.

- [x] **80. El diario del nodo existe; falta el mando que lo compare con
  el del testigo.** Abierta en §272, que dejó la pieza que faltaba
  —`diario::ausentes()`, la comprobación DIRIGIDA— **ejercitada en tests
  pero sin mando de CLI**.
  ⚠️ **La propiedad es real hoy; su instrumento está incompleto.** Los
  datos están en las dos orillas y el núcleo es común —el testigo
  transcribe las claves que el nodo sirve—, pero `comparar_lineas` mapea
  por `index` y **sólo recorre los presentes en ambos**: una línea del
  testigo AUSENTE del diario del nodo le pasa en silencio, y ése es el
  caso grave —o el nodo firmó algo que no recuerda, o alguien sirvió una
  firma que el nodo no emitió—. **Alcanzable, no ejercitada.**
  ⚠️ **Y sólo una dirección vale.** El diario del nodo es COMPLETO —uno
  por latido— y el del testigo MUESTREADO —sólo lo que pidió—: contar la
  dirección contraria daría rojo en cada corrida y acabaría siendo
  paisaje.
  **Segunda mitad**: `--diario` es explícita, así que un nodo que firme
  sin ella sigue sin poder reconocer su firma. Decidir si eso se queda
  como bandera o pasa a ir con `--clave`.
  - MAPA (§283): la comprobacion dirigida ya tiene mando — `--ausentes <TESTIGO> <NODO>`
    en el testigo, con `ausentes()` mudada del nodo (pura, cero llamantes de produccion;
    ningun `Cargo.*` tocado, la foto del `--completo` vive). Ejercitado por sus partes,
    no de punta a punta: k=0, los tests de witness.rs no tocan ficheros. Los dos tests
    mudados pierden el acoplamiento con `linea()`: el formato de la linea del diario es
    un contrato compartido SIN CASA COMUN (cuarto caso del patron; ni verify ni hash
    pueden alojarlo). SEGUNDA MITAD ABIERTA: `--diario` es bandera aparte — un nodo que
    firma sin ella no puede reconocer su firma despues. Recomendacion SIN RATIFICAR:
    «quien firma, anota» (que `--diario` vaya con `--clave`).
    CUMPLIDA (§285), ratificada la recomendacion: un nodo con clave NO
    ARRANCA sin `--diario` —el molde de `--custodia fichero`—. La 80
    queda entera: mando (§283) + memoria obligada al firmar (§285).
- [x] **81. Tres warnings en un ejemplo que el canon no ve.** El canon
  no compila ejemplos, así que la fila de zk-ssl marca 0 mientras
  `crates/zk-ssl/examples/etapa_b1_lote_medido.rs` avisa tres veces en
  cada compilación: un `mut` inútil (:254) y `stale` asignado y nunca
  leído (:244 y :289). Preexistentes — §275 no los introdujo y no los
  toca. Evidencia: salida de la corrida 3 del BLOQUE-275c, 12-08-2026
  (§275). Arreglo: quitar el `mut` y las dos asignaciones muertas —o
  usarlas—; se marca aquí cuando el ejemplo compile limpio.

  Medida otra vez en el §583: los tres avisos siguen, en las mismas lineas (:244,
  :254 y :289), compilando con `--features sandbox`, que el ejemplo exige.

  ✅ **CERRADA EN EL §584**: quitado el `mut` y las dos sumas, que iban justo antes de
  un `panic!` y nadie leia; `stale` queda en un cero fijo con su razon escrita, y el
  banco hace lo mismo que antes. El ejemplo compila sin un aviso.

- [x] **82. Acumulador de razones para romper el formato del log.** Una
  rotura de formato cuesta persistencia, store, DTO, vector y spec, así
  que «un formato se rompe una vez por razón» tiene corolario: **se rompe
  una vez para VARIAS razones juntas**. Esta entrada existe para que esas
  razones estén escritas cuando llegue el día, en vez de reconstruirse de
  memoria.
  **Razón 1 (§279)**: el compromiso autorizante como **campo propio** de
  `LogEntry`. Hoy el sello lo ata pero sus parámetros no están en el
  registro, así que un tercero sólo recompone `OpenAccount` y `Recovery`
  (2 de 6). Con el compromiso en la entrada, las seis serían
  recomputables sin el nodo.
  **Cómo se cierra**: no se cierra sola. Se cierra el día que un sello
  rompa el formato por cualquier razón, y entonces **entra la lista
  entera**. Hasta entonces crece por adición.

  **Ampliación (§280, 2026-08-12) — la lista se cierra ANTES de montar la
  rotura; §281 la ejecuta.** Decisiones, con procedencia:
  - **Prospectiva por NECESIDAD, no por elección**: el compromiso
    histórico no existe en ningún sitio — no se guardó nunca, que es
    justo el hueco que §278 cerró hacia adelante. Las entradas previas
    quedan v1; el registro pasa a **dos eras**, como la cabeza.
  - **Layout v2 CERRADO: 169 bytes** = los 137 de v1 (seq 8 · kind 1 ·
    root_old 32 · root_new 32 · proof_digest 32 · chain 32) + el
    **compromiso autorizante** como campo (32). **La longitud
    discrimina la era** — precedente 49-A en el propio store — y esa
    decisión sólo es válida porque esta lista se cierra ENTERA: una
    quinta razón de formato exigiría byte de versión, no una tercera
    longitud.
  - **La tercera razón queda SUBSUMIDA, no ampliada** (elección del
    asistente, REVERSIBLE): las delegadas `raiz, raiz` no ganan campos
    propios — el compromiso, que ya CONTIENE las raíces del árbol que
    de verdad mueven (congelados, custodios, pendientes), viaja como
    campo y las ata. Leerlas en claro sería un RPC de parámetros, no
    formato; y campos extra romperían además el encadenado de cuentas
    que `verify_chain` comprueba.
  - **Los lotes (RFC-0002 etapa 2) entran AQUÍ**: no añaden bytes por
    entrada; su rotura es SEMÁNTICA —dentro de un lote el encadenado
    por entrada cambia, y la spec ya lo declara— y el `verify_chain`
    de dos eras de §281 la contempla. Dejarlos fuera sería la segunda
    rotura que esta lista existe para prohibir.
  - El MAL-hoy que acompañaba: la garantía 4 de `spec/RPC.md`
    prometía «bytes» de prueba; desde §278 ata compromiso o ausencia
    declarada según la clase. Corregida en este mismo sello.

  **CERRADA (§281, 2026-08-12) — el MAPA, pieza a pieza**: la lista se
  escribió y cerró en §280 (la ampliación de arriba); §281 la ejecutó
  ENTERA en lo de formato: compromiso como campo (`Option`, era 2
  `Some`) → `log.rs` §281; layout 169 con la tabla exacta y longitud
  discriminando → `store.rs`; encadenado v2 con el compromiso dentro →
  `chain_digest_v2` y `verify`/`verify_chain` por era; snapshot con el
  bit alto de `n_log` y prefijo u16 (un stream no discrimina por
  longitud — borde encontrado al montar) → `snapshot.rs`; DTO aditivo
  con rotura en voz alta → `wire`. Lo que esta nota NO cierra, y a
  dónde va: el reverificador a 6-de-6 prospectivo → sello siguiente
  (su fichero no viajó en el terreno y a ciegas no se parchea); la
  semántica de lotes en `verify_chain` → queda el gancho documentado
  en el propio código, se implementa cuando RFC-0002 etapa 2 exista.

  ✅ **El primer destino, CUMPLIDO en §282**: el reverificador es ya de
  dos eras — las seis clases con sello se recomputan cuando la entrada
  trae compromiso, y `COMPROMISO_AUSENTE` se mudó a `zk-ssl-hash` para
  que el verificador independiente pueda distinguirlo sin depender de la
  capa. De paso cayó una restricción que esta nota no había previsto: en
  era 2 **basta un tramo** del registro. Sigue abierto el segundo
  destino: la semántica de lotes, con su gancho documentado.

- [x] **95. El documento OpenRPC publicado no es autocontenido: cuatro
  defectos del ARTEFACTO, medidos.** ⚠️ **No es una ampliación de la 94**,
  y conviene decir por qué: la 94 es del INSTRUMENTO —clases de resultado
  que el canon no ve—; ésta es del ARTEFACTO que se publica a terceros,
  cuya cabecera promete que «una herramienta o una SEGUNDA implementacion
  la consume sin leer el codigo del nodo». Medidos los cuatro en §302:
  **(a)** los `$ref` de resultado **cuelgan**: el documento emite 23
  destinos distintos y `components/schemas` declara 4; sólo `Q`, `Digest`
  y `ProtocolVersion` resuelven. Un validador OpenRPC lo rechazaría.
  **(b)** y al revés: `DATA` está declarado y **no lo referencia nadie**.
  **(c)** `SignedEpochHead` figura como esquema de resultado **aunque el
  cable no tenga ese DTO** —el cable tipa la cabeza SIN firma
  (`EpochHeadDto`) y no la firmada—. ✅ Este punto **se cierra solo**
  cuando el tramo (iii) del eslabón 3 haga nacer `SignedEpochHeadDto`.
  **(d)** la pieza que ata `spec/openrpc.json` a `document()` compara
  **valores parseados, no bytes**, y es ciega a la FORMA: medido en §302,
  el artefacto llevaba **desde §275** con la clave `summary` fuera de
  orden en una entrada, y ninguna compuerta lo vio. El §302 lo normalizó
  al regenerar; **la ceguera sigue**. Arreglarla es un `assert_eq!`
  contra `to_string_pretty(&document())` en la pieza que ya existe, y
  **no mueve ninguna cifra**.
  **Dónde se mide la verdad**, para no escribir aquí una lista que
  envejezca: el censo de destinos emitidos contra esquemas declarados
  —`grep -oE '#/components/schemas/[A-Za-z]+' spec/openrpc.json` frente a
  las claves de `components.schemas`—, **re-medible con el mismo
  instrumento** el día que alguien lo revise.
  ⚠️ Cerrarla exige decidir **por clase**: (a) y (b) son el espacio de
  nombres de resultado y piden casar una veintena de tipos contra
  `spec/RPC.md`; (c) se resuelve solo con el (iii); (d) es barato y no
  depende de los otros.

  Medida otra vez en el §583: el documento referencia 35 esquemas distintos y declara
  5 -`Bool`, `DATA`, `Digest`, `ProtocolVersion` y `Q`-, asi que cuelgan 30 (eran 23 y
  4): la (a) ha CRECIDO con el protocolo. La (b) ya no se da: todo lo declarado se
  referencia.

  ✅ **CERRADA EN EL §585, las cuatro.** (a) Toda referencia RESUELVE: los 30 nombres
  que colgaban tienen su esquema -un puntero a `spec/RPC.md`, con el DTO del cable que
  lo tipa cuando lo hay (13, medidos contra el manejador de cada metodo) y los metodos
  que lo usan, derivados de la tabla-. (b) Un test exige que lo referenciado y lo
  declarado sean lo MISMO en los dos sentidos. (c) Se cerro sola: `SignedEpochHeadDto`
  existe. (d) El fichero se compara tambien en BYTES. Falsadores ensayados: sin una
  fila, con una de mas, con un DTO mal nombrado y con el JSON reformateado, los cuatro
  ROJOS nombrando su causa. ⚠️ **Residuo, por diseno** (cabecera de `openrpc.rs`): el
  documento RESUELVE pero no da FORMA; de el no se genera un cliente sin `RPC.md`, y
  `spec/README.md` dejo de prometerlo. REVERSIBLE: dar forma seria otra entrada.

- [x] **96. Una RELACION publicada, INVERTIDA: declarada en la Entrada 5 de la ERRATA y fundida
  en la 28 (⚠️ §860).** Los tres preprints afirman
  que la mitad cara de un pago cae en el RECEPTOR —cobro de unos 500 ms
  contra 283 ms de envio— y construyen sobre eso un argumento normativo:
  que la parte con menos capacidad de elegir si participa carga con el
  coste mayor. **Medido cuatro veces el 14-08-2026, es al reves**: envio
  260,7-286,8 ms, cobro 159,4-189,6 ms, y las bandas **no se solapan**. El
  envio no crecio —el 283 publicado sigue siendo bueno—; el que encogio
  es el cobro, unas 2,7 veces, y **nadie reviso el argumento que dependia
  de el**. **Tesis distinta de la 95**: aquella es un defecto del
  artefacto publicado; esta es una afirmacion con carga normativa que hoy
  es falsa **por inversion, no por magnitud**. **Donde se mide la
  verdad**: la banda `PUBLICADA_ENVIO_SOBRE_COBRO_MIN` de `metrics.rs`,
  que desde §304 da rojo si el sentido se invierte —con BANDA y no con
  valor, porque los tiempos dependen de la maquina y los bytes no—.
  **Cruce**: el texto vive solo en `doc/preprints/`, que la **28**
  suspende hasta el fin del proyecto, asi que aqui se DECLARA y no se
  repara. Entra REVERSIBLE: si se prefiere, su contenido cabe en el
  cuerpo de la 28 y esta se retira.
  ⚠️⚠️ **2026-09-19 (S512): LA INVERSION SE HA INVERTIDO, y lo mide el
  arbol.** El arreglo B (5.A-272, S511) cambio el AIR y con el los dos
  lados: medido APAREADO sobre `3db491f` y sobre el corte, misma maquina y
  mismo target, envio 240,8 -> 158,9 ms y cobro 160,2 -> 278,4 ms; la razon
  cobro/envio pasa de 0,67 a **1,75** y los preprints publican **1,77**. El
  sentido que publican ya NO es falso, pero **la causa que alegan sigue
  sin serlo**: la atribuyen a que el circuito del cobro recorre dos
  arboles, y hoy la produce el acumulador del arreglo B. **La nota no se
  cierra: cambia de objeto** -- de una relacion invertida a una afirmacion
  cierta por otro motivo y desde otra fecha. Y donde se mide la verdad hoy
  es `el_lado_caro_es_el_declarado` de `metrics.rs`: la banda
  `PUBLICADA_ENVIO_SOBRE_COBRO_MIN` que esta nota cita se fue en el S362.
  ⚠️ **§860: decidida por delegación del autor («resuelvelas aplicando los principios»), con el
  criterio del §609 y el precedente del §642; REVERSIBLE; entra con su aceptación. La nota cambia de
  objeto por TERCERA vez, y se CIERRA fundida en la 28**, la vía REVERSIBLE que su propio cuerpo
  ofrecía. Citaba como sitio de la verdad `el_lado_caro_es_el_declarado`, que no existe desde el
  §538: el test es `los_dos_lados_del_pago_atan_la_banda` (`crates/zk-ssl/src/metrics.rs`), y ya no
  aserta el sentido (RFC-0009 D-AJ). Con la ocultación, los dos lados midieron lo mismo dentro del
  ruido —unos 700-750 ms cada uno— y la razón envío/cobro dio 0,975 y 1,064 (SALIDA-538M). El ECST
  §7.2 lo vio invertirse entre dos días en una microVM Firecracker de 4 vCPU, con `simulate --amount
  250000` y una corrida por día —la vía de la capa por la CLI, la que el §362 descartó como objeto
  de esta relación—: envío 970 ms y cobro 1.198, y luego 1.333 y 879. Después cambiaron los
  circuitos de la vía del pago (§641, §680) y el probador (§652), y la relación **NO está medida**
  sobre el árbol de hoy.
  La causa publicada ya era errónea al depositarse: en `6384907` (30-07-2026, el commit que trajo al
  árbol el texto publicado) y en `69fa2d4` (01-08-2026), `circuit_send.rs` y `circuit_claim.rs`
  tienen la misma geometría —`TRACE_LENGTH` 1024, la raíz de cuentas en la fila 271 y la de
  pendientes en la 743—, y el envío lleva un segmento más (5 frente a 4) y una columna más; y en
  `3db491f` el envío y los dos cobros suben por los mismos ciclos de los dos árboles
  (`CYC_ACC..CYC_PK` y `CYC_PEND_CLIMB..CYC_FIN`). Lo leído es MEDIDO; que eso no distinga el coste
  de los dos lados es DEDUCIDO. Así que lo medido no sostiene lo publicado, ni el sentido ni la
  causa: «la mitad cara es del receptor», 500 ms frente a 283 —equivale a 1,77—, «porque el cobro
  recorre dos árboles», con su lectura normativa.
  **Se DECLARA** donde el árbol manda declarar toda corrección a un preprint:
  `doc/preprints/ERRATA.md`, Entrada 5, que dice además en una línea que los bytes del mismo párrafo
  se movieron. El §512 la daba por pendiente y la llamó «Entrada 3», número que ya tenía otra. El
  cuerpo de los preprints y Zenodo siguen en la 28, que gana el frente con este puntero. No vuelve
  ningún aserto de sentido: D-AJ solo se reabre si el sentido «volviera a medirse estable».
  Corregidos en su sitio, con marca y sin borrar, los comentarios que lo daban por vigente
  —`metrics.rs` y `tools/check_publicadas.py`— y la atribución de cifras de D-AJ. **Se reabre** si
  se prefiere la nota viva: vuelve con su número.

- [ ] **97. Lo que costaria un `zk-ssl-air` de verdad, ya medido.** El
  spike del §305 extrajo la mitad verificadora del AIR a un crate propio
  y la llevo hasta un guest bare-metal. Si algun dia se parte de verdad,
  el camino esta MEDIDO y no hay que redescubrirlo. **Soltar la fachada**:
  depender de `winter-air`, `winter-crypto`, `winter-math` y
  `winter-verifier` por separado, porque `winterfell` sirve el AIR A
  TRAVES del probador y con el arrastra `tracing`. **Podar**: los tres
  modulos hermanos llevan cada uno su `build_trace` y su `impl Prover`, y
  el modulo de nullifiers entra en el grafo solo porque el de nativo
  importa UNA constante suya. **Dar `alloc`**: dos lineas por modulo en
  los tres que lo piden. **En el hash**, `extern crate alloc` y **un solo
  `std::`** que resolver, el `impl std::error::Error`, mejor con
  `core::error::Error` —estable desde 1.81 y disponible: el toolchain de
  la casa es 1.97—. **Y sobra**: `AuxRandElements`,
  `ConstraintCompositionCoefficients`, `PartitionOptions`,
  `DefaultRandomCoin`, `MerkleTree`, `MerklePath`, el alias del hasher y
  la funcion de bits, que el compilador nombra uno a uno. ⚠️ Esto
  **sobrevive a la decision sobre la rama B**: vale aunque la agregacion
  se abandone, y por eso no vive dentro de la 22.

  **Cruce, sin cerrarla (§583).** El §463 hizo nacer `zk-ssl-air` con la primera pieza
  de este camino -los cuatro `winter-*` sueltos y clavados, sin `winter-prover`- para
  los AIR que el kit verifica. Lo que partio el spike del §305 no se ha partido en el
  arbol.

- [x] **98. `VISION.md` §3.8: una seccion entera construida sobre la aritmetica de la era de UN PASO.**
  «Sobre la retencion de pruebas — P7 aplicado» compara retencion central
  contra distribuida, y su aparato numerico entero viene de la via de un paso,
  **retirada desde entonces**: la acumulacion por millar (`:325`), su derivada
  diaria repetida tres veces (`:326`, `:346`, `:365`), la tabla de `:346-347`
  y el «463 veces menos» de `:349`, que sale de dividir una prueba entre una
  entrada del registro encadenado.
  ⚠️ **No es una cifra rancia: es un ANALISIS.** Repararla exige decidir que
  cuenta como «op» y, sobre todo, **que tamano de entrada aplica**: el que la
  seccion usa es el de la ERA 1, y la era 2 tiene otro —`log.rs` lo documenta
  al hablar de la longitud en disco que discrimina las dos eras—. Media
  reparacion dejaria la seccion contradiciendose a si misma, que es peor que
  dejarla coherentemente vieja con una nota que la senala.
  **Donde se mide la verdad**: el numerador es `PUBLICADA_PAGO_B` de
  `metrics.rs`, atado desde el §304. El denominador —el tamano de la entrada en
  disco— **no esta medido**, y es lo primero que hay que medir para cerrarla.
  **Cruce**: `tools/check_publicadas.py` EXCLUYE `VISION.md` por esta nota e
  imprime la razon en cada corrida; cerrar la nota es lo que retira la
  exclusion. La conclusion cualitativa de la seccion —la retencion distribuida
  gana por dos ordenes de magnitud— **probablemente sobrevive**: lo que cambia
  es la magnitud del factor, no su signo.

  ✅ **CERRADA COMO HISTORIA en el §448** (`3294986`): `VISION.md` salio de la raiz a
  `doc/historia/`, con un aviso que dice que se conserva tal cual y que sus
  afirmaciones en presente son del proyecto de entonces. Es la via que esta entrada
  preferia a media reparacion: coherentemente vieja, con una nota que la senala.
  Marcada en el recuento del §583, REVERSIBLE: si se quiere la reparacion, se reabre
  midiendo el denominador.

- [x] **99. La marca del cofirmante se dispara también cuando las dos líneas
  son IDÉNTICAS.** `contar_acreditacion` marca a un cofirmante en cuanto
  `verificar_cofirmas` levanta `indice-repetido`, y desde el §333 ese hallazgo
  mira el índice que va dentro de la firma. Pero **repetir la MISMA línea no
  revela nada**: es la misma firma sobre el mismo mensaje. Lo que compromete una
  clave de un solo uso son **dos mensajes distintos** firmados con el mismo
  índice de hoja. ⇒ hoy quien reciba un fichero puede desacreditar a un
  cofirmante honesto **duplicándole una línea**, y el `NEGATIVO-B2` de
  `tools/banco_cofirma.sh` ejercita justo ese caso —el inocuo— porque hace
  `head -1` dos veces. Repararlo exige distinguir **repetición del índice con
  digest distinto** (grave) de **línea duplicada** (inocua), y decidir qué hace
  cada una con la acreditación. Medido en el §333 y dejado fuera a propósito:
  una herencia por corte.
  ✅ **RESUELTA (§334).** El hallazgo exige ahora **dos mensajes distintos** —el
  valor del mapa de la serie lleva el preámbulo, no sólo la línea previa— y la
  marca de `contar_acreditacion` exige además que la línea **no traiga ningún
  otro hallazgo**, con lo que una copia forjada que no verifica se dice pero no
  quema. Dos pruebas nuevas lo fijan, y la VIVA enseñó el rojo antes de tocar los
  tests: los dos únicos casos que se rompieron eran justo los que ejercitaban el
  caso inocuo.
  ⚠️ El `NEGATIVO-B2` del banco **sigue sin reparar**: su parche está escrito y
  con sus compuertas verdes, pero el banco no llega hasta él desde el §331 —ver
  la **100**—, así que se aplicará con ella.
  ✅ **§338** · el parche del `NEGATIVO-B2` YA ESTA APLICADO. El banco
  distingue duplicar una linea -que no acusa a nadie- de repetir el indice sobre
  OTRO mensaje -que si-, y su VIVA corrio ENTERA por primera vez desde el §331.
  Lo que faltaba no era el parche, que llevaba escrito desde el §334, sino un
  arbol donde el TESTIGO reiniciara (§337).

- [x] **104. El fichero de cofirmas es un DETECTOR, no una prueba.** El §337 ata
  el contador del guardián del testigo a lo que las cofirmas acreditan: si el
  contador ha retrocedido por debajo del índice que va **dentro de una firma**,
  el testigo no arranca. El atado funciona mientras el fichero esté ahí.
  `--cofirmar` lo exige por `requires_all`, pero se abre en modo añadir y
  **borrarlo lo recrea vacío**: un contador restaurado hacia atrás junto con el
  fichero borrado pasa el gate sin que nadie se entere, y una rotación rutinaria
  de ficheros lo apaga sin mala fe. Es la misma forma que la nota del diario del
  nodo (§335) y que el caso «dos testigos en el mismo disco son un solo testigo»:
  **detecta un subconjunto, no demuestra nada**. Lo que falta no es código sino
  ponerlo por escrito donde se leen los límites: `doc/CONFIANZA_RESIDUAL.md`.
  Corte propio, sin urgencia.
  ✅ (§779) **CERRADA: escrito donde se leen los límites.** La cabecera de
  `doc/CONFIANZA_RESIDUAL.md` lo dice tras el límite gemelo del diario del nodo
  (§335) y el del cerrojo (§709), y antes se leyó en
  `crates/zk-ssl-cli/src/witness.rs`: un fichero ausente o vacío da `None` en
  `tope_de_cofirmas`; con `None` la clave en cero se resincroniza sin mirar nada
  —lo ata `la_clave_en_cero_sin_prueba_en_contra_se_resincroniza_hasta_el_contador`—
  y un contador borrado junto con el fichero coincide en 0 y arranca, también
  después del §594, que sólo mira el tope si lo hay. El fichero borrado está
  LEÍDO, no ejercitado: ningún test ni banco lo borra, y el `NEGATIVO` de
  `tools/banco_reutilizacion.sh` mide el caso que sí se ve, con las cofirmas en
  su sitio. Tres precisiones a lo de arriba, declaradas también: «por debajo» es
  `<=`, así que a la par tampoco arranca; `requires_all` exige una ruta y no ese
  fichero, así que otra ruta también lo apaga; y el tope falla hacia el lado
  permisivo por vías que esta entrada no nombraba —las líneas ilegibles se saltan,
  la línea de cofirma se escribe sin `fsync` y, leído y no medido, un solo byte
  que no sea UTF-8 deja el fichero entero sin tope, porque `leer` es un
  `read_to_string`: el defecto que el §666 arregló en el diario del nodo y que el
  testigo conserva—. Ningún código tocado: el «Va a CONFIANZA_RESIDUAL» de
  `politica_del_cofirmante` ya es cierto, y el arreglo del byte es corte propio.

- [x] **105. La cabecera del crate de la capa dice en presente lo que dejó de
  ser cierto.** `crates/zk-ssl/src/lib.rs` titula una sección **⚠️ Lo que esta
  capa NO es** y afirma, sin marca histórica, que **no hay persistencia**
  —«reiniciar pierde el ledger»— y que **no hay destrucción de
  circulante (burn)**. El árbol tiene `persistence.rs` con sled y clave
  sellada, y la destrucción está medida por el instrumento de la capa.
  ⚠️ **Dos viñetas más siguen SIN MEDIR** y por eso la nota nace abierta y no
  como corte: «ni política monetaria» y «la clave del emisor es única, no
  de umbral» —ésta última choca con que `mint` **exija dos custodios**,
  veintitrés líneas más arriba en el mismo comentario—.
  **Cruce**: misma figura que el §329 y el §339, ahora en los comentarios
  del código; el censo de aquellos miró `*.md`, `doc/` y `spec/` y no los
  `.rs`. Nace en el §340, que la declara en vez de colarla.

  ✅ MEDIDO Y CORREGIDO EN PARTE (§341). La cabecera son ochenta lineas y la
  seccion tiene SEIS vinetas, no cuatro. Reparadas TRES: la persistencia (existe:
  `persistence.rs`, `sled` y cifrado en reposo, con el arbol de cuentas
  guardado bajo `acct:` y su raiz en `root:state`), la destruccion de circulante
  (existe: `burn.rs`, con circuito y con medida) y la coletilla «en memoria»
  de la vineta de la red. La vineta de la RED se sostiene y no se toco.
  **SIGUE ABIERTA** por tres: la delegacion de la prueba esta respaldada por la
  cabecera de `client.rs` pero no declarada aqui; el umbral del emisor NO esta
  medido, y ademas la cabecera se contradice (§341 lo declara); y la auditoria
  por terceros sigue sin medir. Medido tambien que la misma lista vive en
  `settlement-layer`, en `zk-core` y en `ARQUITECTURA.md`, y que en los TRES es
  CIERTA porque su ambito lo fija el encabezado que la manda.

  ✅ **EL UMBRAL, CERRADO (§342).** La vineta «la clave del emisor es unica, no
  de umbral» era **FALSA**. No existe ningun `fn mint` en la capa —el unico
  `pub fn mint(` del arbol vive en `crates/settlement-layer`, la capa ANTERIOR—
  y lo que hay es `apply_mint_delegated`, que toma **dos pruebas de custodio**.
  `verify_threshold_pair` comprueba dominio, raiz puesta por la capa, operacion
  comprometida, **custodios distintos** y las dos pruebas; y ese umbral gobierna
  **cinco** operaciones: emitir, emitir a pendiente, gobernanza, congelar y
  recuperar. Corregida tambien la copia de `RESUMEN_EJECUTIVO.md`, que es
  documento publico y cuyo propio texto ya se desmentia dos veces.
  ⚠️ **SIGUE ABIERTA por DOS**: la auditoria por terceros, y la delegacion de
  la prueba, respaldada por la cabecera de `client.rs` pero no declarada aqui.

  ✅ **CERRADA EN EL §587: las dos, MEDIDAS y CIERTAS.** La delegacion: las tres
  funciones que generan prueba en la capa -`client::prove_send`, `client::prove_claim`
  y `prove_minimum` de `audit.rs`- toman la clave de gasto, y la cabecera de
  `client.rs` dice por que no hay otra via: probar sin ella exige verificar una firma
  en el circuito, que es la entrada 21, abierta. La auditoria por terceros: no hay
  ninguna; lo dicen `SECURITY.md` y `README.md`, y la 7 y la 89 siguen abiertas. Cada
  vineta de la cabecera lleva ahora su respaldo escrito al lado. Las seis vinetas de
  la seccion quedan medidas.

- [x] **106. La API de emisión RETIRADA sigue documentada en tres sitios.**
  Medido en el §342 al censar el ámbito de la cabecera de la capa.
  `ARQUITECTURA.md:6-17` abre el documento con un ejemplo que usa
  `SovereignLayer::open(..., issuer_key, ...)`, `layer.mint(issuer_key, ...)` y
  `layer.send(sk_alice, ...)` —mientras el **mismo documento** titula en `:776`
  «Emisión con umbral: se acabó la clave única» y usa `raiz` y `&auth`—. Y
  `crates/zk-ssl/src/accounts.rs:5` y `:72` siguen diciendo que emitir «exige la
  clave del emisor».
  ⚠⚠ **Hallazgo estructural de la misma medición**: las líneas `:69-72` de
  `accounts.rs` describen `open_account` y están pegadas **encima de
  `stored_view_id`**; el `open_account` real vive en `:112` y tiene doc propio.
  Es un comentario **huérfano documentando la función equivocada**, no prosa
  envejecida, y por eso no se tocó nada: decidirlo exige leer entero cada sitio.
  **Cruce**: misma familia que el §329, el §339 y el §341, ahora también en el
  documento principal y en el resumen público.

  ✅ **CERRADA EN EL §361** (`203919c`): la portada de `ARQUITECTURA.md` dejo la API
  retirada, `accounts.rs` dejo de decir que emitir exige la clave del emisor, y el doc
  traspuesto volvio a su funcion. Aquel asiento dijo «BACKLOG quieto», y la entrada
  siguio abierta hasta el recuento del §583. Fuera de lo que nombraba, y sin medir
  aqui: `layer.mint(issuer_key, …)` sigue en `ARQUITECTURA.md:1077`, en la seccion
  «Emision: la ultima puerta por la que se podia crear dinero».

## E. Operacion

- [x] **17. Replica y alta disponibilidad: no se construye ni se prescribe, y se dice por qué
  (⚠️ §860).** **Comprobable**: `grep -rn
  "replica\|cluster\|raft" crates/zk-ssl/src/` no devuelve nada.
  ⚠️ **Constatacion de arquitectura, no hallazgo**: no hay seccion porque no
  hay nada medido que registrar, y va con la entrada 23. El nodo es punto
  unico de
  fallo.
  ⚠️ **§860: decidida por delegación del autor («resuelvelas aplicando los principios»), con el
  criterio del §609 y el precedente del §642; REVERSIBLE; entra con su aceptación. La alta
  disponibilidad del operador —réplica en caliente, failover— no se construye y el estándar no la
  prescribe; la entrada se CIERRA.** El punto único de fallo que esta entrada constata es el de
  `PRINCIPIOS.md` §0: el operador que se incauta, se apaga o se corrompe
  (`ARQUITECTURA.md` §«⚠️ Lo primero: el operador del nodo ES un intermediario de confianza»). Una
  réplica bajo el mismo operador hace que caiga menos, no que haga menos falta fiarse de él. Lo que
  lo quitaría, la réplica entre partes independientes, es la 23, que se cerró sin emprenderla (§860),
  y `PRINCIPIOS.md` §8 la deja en «lo que requeriría otros recursos».
  **El §609, por sus dos caras**: el estándar —cable, pruebas, sobres— no impone topología, y por
  eso otro nodo puede añadir alta disponibilidad sin cambiarlo; el nodo de referencia no puede sin
  modificaciones significativas, y se dice: no hay WAL (la 19); una entrada del registro lleva
  raíces, compromiso y resumen de la prueba, no hojas (`crates/zk-ssl/src/log.rs:151-235`);
  `export_snapshot` se niega con dinero en vuelo (`crates/zk-ssl/src/snapshot.rs:219-230`); y
  `snapshot.rs:79-82` lo dice: replicar en caliente «exige red y coordinación, que es el problema de
  sistemas distribuidos que este proyecto no aborda». La cabecera de `doc/ESCALADO.md` llama a C7,
  réplica y failover incluidos, «ESTUDIO CONDICIONAL de una misión no declarada». Y el RFC-0015
  descartó «varias claves válidas a la vez»
  (`spec/rfc/0015-el-ciclo-de-vida-del-firmante.md` §«Lo que se DESCARTÓ al medir»), que descarta
  también un firmante por réplica, con su clave o con un tramo de hojas propio. El §5 de
  `PRINCIPIOS.md` solo vale aquí como analogía.
  **Lo que un despliegue con réplica NO DEBE romper, y cuánto de ello impone hoy el nodo**: (1) un
  escritor por libro, impuesto solo en la misma máquina —el cerrojo de `sled`, §783 y §813—: una
  réplica en otra máquina promovida con el primario vivo no lo ve; (2) un estado de firmante por
  clave, declarado y no impuesto (`SECURITY.md` §2, el índice de la clave de firma), y si se ata lo
  decide la 124, que midió la misma clave firmando dos mensajes con la misma hoja; (3) un relevo con
  el estado de la clave no fiable sigue la D-G del RFC-0015, PROPUESTO —clave quemada, rotación a la
  sucesora comprometida, `--desde-minimo` del §690—, y no clonar el estado sale de su «Lo que se
  DESCARTÓ al medir»; (4) la detección es la vista dividida, MEDIDA en la E1 del RFC-0011 (§599):
  con el contador restaurado bajo un testigo vivo, el testigo se detiene con `vista-dividida`; con
  dos nodos de la misma semilla a la vez y un testigo en cada uno, ninguno ve nada solo y solo
  `--comparar` de los diarios lo ve, que es el cerebro dividido de un failover. Y la 123 mide que
  una sonda de 2 s puede dar por caído a un nodo vivo con la máquina sobresuscrita: el disparador de
  un failover falso. La parte del firmante es la familia de la 92 y la 124, y esta entrada no la
  decide. Lo dice, con marca, `SECURITY.md` §2, en la viñeta del índice de la clave de firma.
  ⚠️ **§860: el «Comprobable» envejeció.** Medido sobre `d4d6d45`: el `grep` da 5 líneas
  (`snapshot.rs:79`, `log.rs:93`, `store.rs:45`, `migration.rs:111` y `:167`), y ninguna implementa
  nada: las tres primeras declaran que no hay réplica o comentan otra cosa, la cuarta es un
  comentario de la migración y `migration.rs:167` es el mensaje de un `assert` de un test. Y el
  «*log-shipping* … del WAL existente» de la fila B7 de `doc/ESCALADO.md` §10 no tiene base: no hay
  WAL (la 19).
  **Sigue declarado, y no se toca**: el nodo es punto único de fallo (`PRINCIPIOS.md` §0; el
  objetivo «resistente» de `PRINCIPIOS.md` §3 sigue con ⚠️) y «No hay recuperación si el nodo
  desaparece» (`SECURITY.md` §2, y `SECURITY.md` §4 lo deja fuera de alcance hoy). La vía coherente
  con el principio no es la alta disponibilidad, sino que los fondos sobrevivan al operador: el B18
  de `doc/CONFIANZA_RESIDUAL.md` §6, una propuesta sin entrada propia; adoptarla no se decide aquí.
  **Descartadas**: construir la alta disponibilidad del operador; meter en la misión la réplica
  entre operadores, que es la 23; cerrarla sin escribir por qué; posponerla a lo que decida la 23 o
  a un libro desplegado —la decisión no depende de ninguna de las dos, no porque no se puedan medir:
  `SECURITY.md` §3.6 dice que no hay libro desplegado—; y convertirla en B18. **Se reabre** si el
  autor amplía la misión (`PRINCIPIOS.md` §8) o si se reabre la 23. Marcados en el mismo sello: la
  fila B7 de `doc/ESCALADO.md` §10 y su cabecera.

- [x] **18. ✅ (§783, ⚠️ §813) Bloqueo de directorio de `sled`.** ~~Puede impedir un reinicio~~
  inmediato tras cerrar (§16.6). ⚠️ **§813: el tachado era cierto** para un rearranque que no
  espera la salida del proceso viejo —el de un supervisor que manda SIGTERM y relanza—: el
  cerrojo es del viejo hasta que sale. Lo que la cierra es que el nodo espera, abajo.
  ⚠️ **Manifestacion MEDIDA** el 31-07-2026
  (§79): hizo fallar un test de release **1 de cada 12 pasadas** a 16 hilos,
  con `WouldBlock` al reabrir. Los tests quedan protegidos con
  `open_encrypted_retry` (entrada 45), pero **el fondo sigue abierto**: un
  nodo real que se reinicie inmediatamente tras cerrarse puede sufrir lo
  mismo, y ahi no hay reintento de test que valga. Deja de ser una
  limitacion teorica.

  ⚠️ **§783: medido con un nodo de verdad, no se reproduce, y lo que queda
  se dice con su nombre.** `tools/banco_reinicio.sh` mata el nodo —SIGKILL
  esperando al viejo, y SIGTERM SIN esperar, el rearranque inmediato que esta
  entrada temia— y lo rearranca al instante sobre el MISMO libro: 15 de 15
  con cada senal en el banco, y 90 de 90 en la medida del sello. El cerrojo
  de `sled` es del proceso: muere con el. Lo que si choca es un SEGUNDO nodo
  sobre un libro que otro tiene abierto, y eso es correcto —falla cerrado,
  antes de leer ni escribir—; el §783 hace que lo diga («el libro lo tiene
  abierto OTRO proceso (el cerrojo de sled): un nodo por libro») en vez del
  `Io("could not acquire lock")` de la biblioteca, lo ata
  `dos_nodos_sobre_el_mismo_libro_el_segundo_se_niega_con_nombre` y el
  banco lo comprueba. **No se reintenta** (⚠️ §829: ya no; desde el §813
  el nodo espera al cerrojo, acotado, y el autor lo ratifico el 22-10-2026,
  como dice la nota del final): un reintento solo haria esperar a
  un nodo que no deberia estar ahi. El `WouldBlock` del §79 salio en los
  tests, que cierran y reabren el libro dentro de un mismo proceso, a 16
  hilos; ahi sigue `open_encrypted_retry` (la 45).

  ⚠️ **§813: el §783 cerraba sobre una medida que no lo era, y la 18 se cierra
  ahora porque el nodo ESPERA.** Decidido por delegacion del autor, que pidio
  «Aplicar los Principios … para resolverlos», con el criterio del §609: «que
  se aplique al mayor número de casos de uso sin modificaciones
  significativas», y un supervisor que no espera la salida del viejo vale asi
  sin tocarlo. Los «90 de 90» del sello eran 30 con SIGKILL y 30 con SIGTERM
  esperando la salida del viejo, y **30 de 30 con SIGTERM SIN esperarla**: esos
  los gano el nodo nuevo a la carrera. `kill` vuelve al poner la senal, no
  cuando el viejo ha salido, y el cerrojo —un `flock(LOCK_EX | LOCK_NB)` de
  `sled` sobre `<libro>/db`— es del viejo hasta que sale; el nodo lo pedia como
  primer acto, sin reintento. Remedido el 15-10-2026 con el nodo de `b6994c3`,
  en 4 CPU, sin esperar al viejo: **no reabrian a la primera 2 de 200
  rearranques sin carga, 8 de 200 con cuatro `yes`, y 33 de 200 y 12 de 120 con
  ocho**. El banco del §783 salia ROJO 1 pasada de 20 sin carga anadida y 1 de
  20 con cuatro `yes`, y con otro trabajo en la maquina, 8 de 8 con cuatro
  `yes` (carga de 9,2 a 12,5) y 10 de 10 con ocho (de 6,0 a 13,4). Tras la
  salida del viejo, ninguno: en los 3.700 rearranques de los registros de la
  decision (1.800 de un banco con el REINICIO esperando, 600 con SIGKILL
  esperando, 900 de la propuesta, que al nodo solo le cambiaba un comentario, y
  200 y 200 sueltos con ocho `yes`) y en 120 mas de su segunda ronda. Nunca
  hubo dos escritores: el que llegaba antes se negaba. **Lo que hace el §813**:
  el nodo espera al cerrojo, acotado —diez miradas cada 50 ms, las de
  `cerrojo_libre` en la capa (§812) y las de `open_retry` en los tests (ya en
  el §16.6; el §79.3 la lleva a `open_encrypted_retry`)—, y SOLO a ese error,
  el `flock` de `sled` que da `WouldBlock`; un `Io` de otra causa, no. Es lo
  que pedia el §16.6: «Un despliegue real debería reintentar con espera».
  Pasado el tope, la negativa del solape largo sigue cerrada y con su nombre,
  y ahora dice las dos causas —otro nodo, o el anterior que aun sale: «espera a
  que salga y vuelve a arrancarlo»— y deja de recetar otro `--ledger` a
  secas. Seguida al pie de la letra, con la misma clave y el mismo `--diario`,
  `--contador-recepcion` e `--indice-firma`: con el nodo viejo vivo, el
  guardian del indice no deja arrancar al nuevo (§709); con el viejo parado,
  esa receta mezcla dos libros en la cadena de la clave: con un libro que ya
  tenia una cuenta, el `seq` del diario va 0, 1 y vuelve a 0. Y no basta con
  cambiar esos tres: con la misma clave, un `--indice-firma` nuevo vuelve a
  firmar con hojas ya gastadas, y la misma hoja XMSS firmo dos mensajes
  distintos, con el viejo vivo y con el viejo parado, que es lo que compromete
  una clave XMSS (`SECURITY.md` §2). La negativa dice ahora que otro
  `--ledger` es OTRO nodo, con OTRA clave de firma y su propio indice, su
  contador y su diario, como ya decia la del guardian («su propio contador y
  su propia semilla»); seguida asi, con el viejo vivo, ninguna hoja firma dos
  mensajes. Lo que el nodo no detecta va en la 124. Lo atan cuatro tests del
  nodo: tomado 200 ms y suelto, abre (sin la espera, cae en la primera
  mirada); tomado mas que el tope, la negativa con nombre y con la otra clave,
  sin tocar un byte; ningun otro error se espera, ni un `flock` que falla sin
  `WouldBlock`; y `open_layer`, la espera que corre en el nodo, tarda al menos
  450 ms en negarse, con las cifras de `cerrojo_libre` escritas.
  `tools/banco_reinicio.sh` espera ahora la salida del viejo en el REINICIO y
  exige reabrir a la primera, cuenta aparte el SOLAPE, SIGTERM y relanzar sin
  esperar, y en DOS NODOS exige que el segundo tarde al menos 450 ms en
  negarse. Medido el 15-10-2026 en 4 CPU con el nodo del §813, 10 pasadas sin
  carga anadida, 10 con cuatro `yes` y 10 con ocho (con otro trabajo en la
  maquina: carga de 4,3 a 5,8, de 5,9 a 9,7 y de 9,4 a 16,5), las 30 VERDES:
  los 900 rearranques del REINICIO abren a la primera; de los 450 del SOLAPE,
  abren a la primera 405 y tras esperar dentro del nodo 45, y ninguno se
  niega; y en DOS NODOS el segundo se niega en 618 a 1.575 ms. Con un nodo que
  mira sin dormir, el banco sale ROJO: se niega en 110 ms. DOS NODOS sigue
  atando la exclusion. El «No se reintenta» de arriba deja de valer, y el de
  `error_al_abrir` queda corregido con marca.

  ⚠️ **§829: el autor ratifico el §813 el 22-10-2026.** Lo de arriba se decidio
  por delegacion; el autor lo ratifico ese dia en el chat de otra sesion que
  trabaja en el arbol, con «sí a todo lo recomendado», y lo confirmo en el de la
  sesion que lo sella, con «si». La espera acotada al cerrojo del libro —diez
  miradas cada 50 ms— revisa el «No se reintenta» del §783, y es ya decision del
  autor. La 18 sigue cerrada y no se reabre. El control lo midio la otra sesion,
  no la que lo sella, sobre `7ed7e17`, en 4 CPU, y lo detalla su respuesta de
  ese dia: en 26 pasadas de `tools/banco_reinicio.sh`, 11 con
  `BANCO_REINICIO_N=20` y 15 con 50, de los 970 rearranques del SOLAPE, 940
  abren a la primera y 30 tras esperar al cerrojo, y los 1.940 del REINICIO, a
  la primera, segun la logica del banco y el total de un agente de su triaje, no
  su registro de serie; otras seis pasadas, de otro agente y con otra tarea en
  la maquina, dan en el SOLAPE 180 mas, 176 y 4. Ninguno se niega, y ningun
  `WouldBlock` llego a error. Lo que ese banco aun dio en ROJO, con el nodo a
  menos prioridad que la carga, es la 123, no esta. `AUDITORIA.md` §829.

- [ ] **19. Sin log de escritura anticipada — y la consecuencia que se
  describia NO existe.** **Comprobable**: `grep -rn "write_ahead|journal"
  crates/zk-ssl/src/` no devuelve nada. ✅ Eso es cierto.
  ⚠️⚠️ **CORREGIDO el 01-08-2026 (§111.2)**: esta entrada decia que un fallo
  entre operaciones «detiene el arranque pidiendo intervencion manual».
  **Medido: cero coincidencias de `intervencion`, `manual` o
  `StoreError::Malformed`. Nada detiene el arranque.** Y `persistence.rs`
  documenta lo contrario: **el ledger queda coherente en los tres casos de
  fallo**; lo que se pierde es **durabilidad, no integridad**.
  ⚠️ **Quinta de la familia §95.2 y de la peor clase**: las otras cuatro
  prometian de mas sobre algo real; **esta describia un mecanismo
  inventado**.
  ⚠️ **La 53 NO depende de esta entrada** (§111.1): XMSS necesita **su propio
  contador con `fsync`**, no un WAL del ledger. El argumento de
  `persistence.rs` —un WAL propio añade superficie sobre lo que `sled` ya
  hace— **sigue siendo valido**.
  ⚠️ Compone con la **84** y la **92** (§288): el estado que aqui
  importa no es el del ledger sino el del FIRMANTE —el indice XMSS que
  debe sobrevivir a un reinicio—. Las tres son el ciclo de vida del
  firmante y convendria cortarlas juntas.

- [x] **20. Rotacion de claves: ~~DOS~~ TRES de cuatro casos (⚠️ §812), y ~~uno no es un hueco~~ el
  cuarto, decidido (⚠️ §860): la gobernanza sigue inmutable.**
  ~~Implementada solo en parte.~~ ⚠️ **«En parte» no se podia ni confirmar ni
  refutar**, y esa era su unica pega: una entrada irrefutable no envejece, se
  queda para siempre pareciendo pendiente (§83). Descompuesta y comprobada el
  31-07-2026:
  - ✅ **Custodios**: rotacion **por uso**, no por tiempo —esta capa no tiene
    nocion de tiempo—. `custodian_uses` sube con cada emision, congelacion y
    recuperacion; al llegar a `max_custodian_uses` los custodios dejan de
    poder actuar hasta que la gobernanza rote el conjunto (`lib.rs`, campo
    `max_custodian_uses`).
  - ✅ **Clave de cuenta**: `recover` deja fuera la clave comprometida, con
    autorizacion de custodios. Test:
    `recovery_locks_out_the_compromised_key`.
  - ~~❌~~ ✅ (§812) **Clave de cifrado del ledger**: ~~**no hay recifrado**~~. Comprobable:
    `grep -rn "rekey\|re_encrypt" crates/zk-ssl/src/` no devuelve nada.
    ~~Cambiar la contraseña exige reconstruir el ledger.~~
    ⚠️ §812: ya lo hay, en la biblioteca: `SovereignLayer::recifrar` cambia la frase y la sal del
    libro entero, entero o nada (⚠️ §816: hasta el §816, sólo con nadie más a la vez: una apertura en medio podía dejar un libro vacío o perderlo; `SECURITY.md` §3.14), por un directorio hermano (RFC-0001, E4, decidida por el autor como
    etapa de ese RFC). El grep de arriba sigue vacío: la función se llama `recifrar`. El nodo sigue abriendo su
    libro en claro: para él no hay frase que cambiar, y cablear su cifrado queda fuera.
  - ⚠️ **Gobernanza**: `governance_set_root` **solo se fija en el
    constructor** y no tiene modificador —comprobable: `grep -n
    "governance_set_root" crates/zk-ssl/src/lib.rs` da lectura y
    constructor—. **Es inmutable por diseño, no un hueco**: si se pudiera
    rotar, quien controlara el conjunto nuevo podria cambiar los custodios
    (test `the_governance_set_survives_restart`). El coste es que **una
    clave de gobernanza comprometida lo esta para siempre**, y eso si merece
    decision propia.
  ⚠️ **§791: el ❌ del recifrado, recontado, y un nombre viejo.** El grep de arriba sigue vacío, pero
  desde el §702 `migrar_a_kdf_v2` vuelve a sellar cada valor del libro con una clave nueva
  (`crates/zk-ssl/src/persistence.rs:1031-1065`; ⚠️ §812: aquí decía `:1003-1047`, las líneas de antes
  del §802 y del §812; ⚠️ §816: y `:1019-1052`, las de antes del §816) y el keystore se migra al abrirse: es con la MISMA
  frase, y ~~cambiarla sigue sin existir~~. ⚠️ §812: aquel re-sellado iba en un lote sobre el mismo
  `sled` y dejaba en los ficheros lo sellado con la clave v1 (`SECURITY.md` §3.14); desde el §812 va
  por un directorio hermano, y cambiar la frase ya existe (arriba). Y el nodo no cifra en reposo
  (`crates/zk-ssl-node/src/main.rs:2258-2261`, declarado en `SECURITY.md`): para él no hay frase que
  cambiar. El ✅ de la clave de cuenta nombra `recover`, que el §65 retiró
  (`crates/zk-ssl/src/recovery.rs:229-230`); la vía es `apply_recovery_delegated`, y el test que
  cita la recorre (`crates/zk-ssl/src/tests.rs:2079`). ~~Los dos casos abiertos~~ El caso abierto, la
  gobernanza (⚠️ §812: eran dos, con el recifrado), con la 52, va a
  `doc/ROTACION_DE_CUENTAS.md` (aceptada como análisis en el §799, con sus decisiones abiertas;
  ⚠️ §799: hasta aquí decía «(BORRADOR)»), que cuesta los casos y no elige. **Sigue abierta.**
  ⚠️ **§812: el recifrado, hecho; la entrada sigue abierta por la gobernanza.** El autor resolvió la
  D-4 de esa nota —K1, aparte del diseño único, como E4 del RFC-0001— y la E4 entra construida en
  la biblioteca (arriba). Queda el caso de la gobernanza inmutable, que «eso si merece decision
  propia», y es la D-3 de la nota, sin decidir. **Sigue abierta.**
  ⚠️ **§860: la gobernanza, decidida por delegación del autor («resuelvelas aplicando los
  principios»), con el criterio del §609 y el precedente del §642; REVERSIBLE; entra con su
  aceptación: G0, y la entrada se CIERRA.** Es la D-3 de `doc/ROTACION_DE_CUENTAS.md`, y la misma
  para la 52. La raíz de gobernanza se fija al constituir el libro y al reabrirlo se compara. La
  fija `changing_the_governance_set_on_an_existing_ledger_fails`, no
  `the_governance_set_survives_restart`, que fija otra cosa (⚠️ en el caso 4 del §2 de
  `doc/ROTACION_DE_CUENTAS.md`). **La contraria, dicha**: rotarla con su propio umbral (G1) curaría
  lo que G0 no cura, una clave filtrada por debajo del umbral y la renovación de miembros. Pero
  dejaría atrincherarse a un umbral comprometido, y pide un código de operación nuevo y que la
  apertura deje de compararla. **Se elige G0** porque no mueve nada, porque no hay libro desplegado
  que pague hoy su coste y porque es el caso de cero rotaciones de cualquier variante futura:
  añadirla después no invalida ningún libro, y quitarla sí. **Se reabre** antes de constituir el
  primer libro desplegado, o si un caso de uso pide renovar gobernadores (la autorrotación con
  quórum mayor, que pide un parámetro de constitución), o al construirse R3 en la 52 (la gobernanza
  precomprometida va con ella). El coste, compromiso y pérdida, pasa a `SECURITY.md` §2. El nodo que
  abre su libro en claro no es un caso de rotación: pasa a su propia entrada, la 127. **CERRADA**:
  tres casos hechos y uno decidido.

- [x] **47. `doc/ESCALADO.md`: propuesta de sharding, COMMITTEADA (§122), sin adoptar: decidida y
  repartida, no descartada (⚠️ §860).**
  Documento externo del 31-07-2026 que dimensiona 5 × 10⁹ usuarios con el
  diseño casi intacto. ✅ **Su observacion central se sostiene**: ninguna
  operacion toca dos cuentas, luego **no hay transacciones cross-shard**, y
  el coste declarado de las dos fases —el pendiente inmovilizado, §29-§30—
  es exactamente lo que compra shardabilidad. ⚠️ **Pero presentaba como
  medido el coste de verificacion (4 ms) sin estarlo** (§89). Medido: **2,35
  ms**, asi que el documento era conservador por 1,7 y **37 shards, no 64**.
  **Requiere cuatro correcciones antes de integrarse** (§89.4), y
  ~~**reconciliar sus B1-B9 contra este backlog**: su B8 es la entrada 12 y
  otros solapan. Anexarlo sin reconciliar duplicaria entradas.~~
  ✅ **B1-B9 RECONCILIADOS (§779), sin adoptar nada.** La tabla, pieza a pieza
  y con fuentes, vive en `doc/ESCALADO.md` bajo su §10. Tienen entrada
  propia dos: **B8 es la 12**, y la replica de B7 es la **17**. El limite de
  **B1** lo evito otra via —la retirada de la via de un paso, §32 y §36;
  evitado, no resuelto— y su residuo es la **23**. **B4** y **B6** estan
  hechos en otra forma y sin B3: B4 es `apply_many` (RFC-0002, §215), con
  una entrada del registro por operacion y sin epocas; B6, `apply` (§487)
  mas el AIR (§680, en el arbol y sin version de cable hasta `zkssl/0.5`),
  contra la raiz vigente y sin ventana. **B2** no tiene entrada: el RFC-0002
  descarto el anti-replay por `nonce`, y solo tiene objeto dentro de B3.
  **B9** tampoco tiene entrada propia, y la **77** lo manda aqui.
  ⚠️ **Lo que queda en esta entrada**, y nada mas: **B3**, que no tiene otro
  sitio —la 48 mando aqui su B11 al cerrarse (§268): B11.1 de
  `CONFIANZA_RESIDUAL.md` depende de el, y B11.3 de B11.1—; **B5**, que
  sigue colgando de B3; el particionado y el enrutador de **B7**, que son
  estudio condicional y no entran sin decision de mision (`VISION.md` §5,
  §122.2); **B9**, que es una medida y no espera a ninguna decision; y, de las
  cuatro correcciones del §89.4, la sensibilidad «× 4 peor» de su §6. Las
  otras tres las recoge la cabecera-mapa del documento (§122), aunque el
  §128.3, escrito despues que ella, aun contaba las cuatro como pendientes;
  esa cabecera pide rehacer la sensibilidad sobre 13.600 TPS/shard, y no se
  ha rehecho.
  ⚠️ **Y una que no es de esta entrada**: la divergencia (ii) de la 12, el Δ
  por pago, la tomo el RFC-0003, ACEPTADO en el §356, y el cuerpo de la 12 no
  lo dice.
  ⚠️ **§860: decidida por delegación del autor («resuelvelas aplicando los principios»), con el
  criterio del §609 y el precedente del §642; REVERSIBLE; entra con su aceptación. CERRADA: decidida
  y repartida, no descartada.** **El dimensionamiento 5 × 10⁹ no se adopta**: C7, §1 y §5-§7 de
  `doc/ESCALADO.md`, y con ellos el dimensionado de B5 y el particionado y el enrutador de B7.
  Quedan fuera del rechazo la fila de la capa de liquidación de `doc/ESCALADO.md` §6 —«3,3 tx/s: el
  código actual la ejecuta hoy, sin cambios»— y el párrafo de la federación que la sigue, que
  `doc/CONFIANZA_RESIDUAL.md` cita y que es la segunda vía del residuo 2 de `SECURITY.md` §2.bis: su
  diseño no es dimensionamiento.
  **Por qué**: amplía el alcance a una misión que el proyecto no declara —la vigente está en el
  `README.md` y en `doc/USE_CASES.md`, que mide la escala en «Thousands of operations a year»
  (`doc/USE_CASES.md` §«Institutional clearing: four tests») y llama al dinero minorista de banco
  central «an argument, not a use case»
  (`doc/USE_CASES.md` §«Central-bank money: an argument, not a use case»)—; cada shard conserva el
  operador único y añade un enrutador nuevo (`doc/ESCALADO.md` §9), así que no quita ningún
  intermediario (`PRINCIPIOS.md` §0); la hoja de ruta de `PRINCIPIOS.md` §8 se corrigió a lo
  alcanzable; el RFC-0002, ACEPTADO, abre la partición «solo si el pico objetivo lo exige»; y
  `doc/DIAGNOSTICO_ESCALADO.md` §6.7 pone la elegibilidad antes que el rendimiento.
  `PRINCIPIOS.md` §2 cuenta la «escalabilidad limitada» entre lo que motiva el proyecto, y apunta al
  otro lado; no gana, porque la respuesta del documento multiplica el problema de lo centralizado de
  esa misma lista y `PRINCIPIOS.md` §3 no tiene ningún objetivo de escala. La P4 de
  `doc/historia/VISION.md` queda como apoyo histórico: la decisión se sostiene sin ella.
  **El invariante que sí se sostiene, escrito**: ninguna operación toca dos cuentas —el envío toca
  la hoja del pagador, el cobro la del receptor—, así que particionar por cuenta no tiene
  transacciones entre particiones (`doc/ESCALADO.md` §3.2; el RFC-0002: «el diseño de dos fases ya
  es el protocolo entre particiones»). Es lo que mantiene posible la reapertura sin modificar el
  estándar, y un RFC que lo rompa tiene que decirlo. La D-2 de la 52 se apoya en él.
  **Se reabre** si un caso de `doc/USE_CASES.md` declara un pico, en operaciones de capa por segundo
  —un pago son dos—, por encima del techo del `apply` del nodo único, remedido en ese momento con el
  protocolo del §89.1: la última medida con protocolo es la del §217, 265-320 op/s, anterior al
  §538, y el §637 midió el `apply` en memoria en 4,8-6,2 ms, con carga y sin ese protocolo. Al
  reabrirse, entra en el diseño la restricción del orden total de la 23: el cobro y el reembolso de
  un mismo pendiente en particiones distintas no comparten orden (DEDUCIDO). El documento se queda
  en `doc/` como estudio condicional, con el cuerpo VERBATIM.
  **Repartida.** **B3 (C3) sí sirve a la misión** (la cabecera de `doc/ESCALADO.md`): es uno de los
  dos caminos que `SECURITY.md` §2.bis da para el residuo «el operador ve el estado» —el operador
  ciego (B11) o la federación—. Pasa, con B2, B11.1 y B11.3, a la **126**, que lo decidirá por RFC
  con su análisis medido antes. **B5** cae con el dimensionamiento: su único motivo escrito es la
  latencia de `doc/ESCALADO.md` §7; si el RFC de la 126 necesita anclas más viejas que la raíz
  vigente o la instantánea del lote, lo plantea allí. **B9** pasa a la **21**, porque mide lo que
  decide si hace falta: la generación del titular en un dispositivo de gama baja, sin medir (aquí
  `uname -m` da `x86_64`). La firma XMSS en ARM, que es del nodo y del testigo, se queda en la
  **77** (original). La sensibilidad «× 4 peor» **no se rehace**: el estudio no se adopta, y su base
  medida (§89.1, 2,35 ms) es anterior al §538; el §637 midió 3,98-4,93 ms por prueba, con carga y
  sin protocolo. La cabecera de `doc/ESCALADO.md` lo dice y marca como anteriores al §538 los 13.600
  TPS por shard. Su condición (6), registrar el hallazgo §2.2, está cumplida: `AUDITORIA.md` §123,
  la (iv) de `doc/USE_CASES.md` §«Institutional clearing: four tests» y
  `doc/INSTITUCIONAL.md` §«Las limitaciones de escala, cuantificadas», n.º 1, que el `README.md`
  enlaza. Marcados en el mismo sello: la cabecera y la tabla del §10 de `doc/ESCALADO.md`,
  `doc/CONFIANZA_RESIDUAL.md`, la 21 y la 77 (original).
  ⚠️ **§860**: lo de la 12 que esta entrada decía que su cuerpo no recogía, lo recoge desde el §779.

- [ ] **21. Delegacion de prueba a clientes ligeros.** Exige verificar una
  firma dentro del circuito: **~8.000 filas** con esquema Winternitz,
  estimacion documentada en `client.rs` (§18). ⚠️ Esta cifra se retiro por
  error en §42.3 dandola por inventada, y se **restituye** (§42.5): estaba
  en el codigo, se busco donde no estaba. Mismo primitivo que la 33.
  ⚠️ **§860: gana B9**, de la 47 al cerrarse: el banco de generación en dispositivos ARM de gama baja
  de `doc/ESCALADO.md` §10. Mide lo que decide si hace falta delegar: la generación del titular en
  un dispositivo de gama baja, sin medir; aquí `uname -m` da `x86_64`, y un tiempo emulado no mide
  un dispositivo. Necesita ese hardware, y conseguirlo queda fuera del repositorio. Alimenta además
  a B11.1 (la 126; `doc/CONFIANZA_RESIDUAL.md` §10, su punto 4) y a B11.2 (la 11). La firma XMSS en
  ARM no es B9: es del nodo y del testigo, y se queda en la 77 (original); si B9 se corre en ARM, la
  misma tanda mide la firma.

- [ ] **22. ⏸️ SUSPENDIDA. Agregacion de pruebas.** 127,2 MiB por mil pagos es coste, no
  parada, pero crece linealmente: un pago son **DOS pruebas** y **133.431
  bytes**, medidos el 14-08-2026 en release con la configuracion real de
  la capa. **Donde se mide la verdad**: la constante `PUBLICADA_PAGO_B` en
  `crates/zk-ssl/src/metrics.rs`, atada al instrumento por el gemelo
  `la_cifra_publicada_sigue_siendo_la_medida` y a los documentos por
  `tools/check_publicadas.py`. Cada eslabon dice una cosa distinta al
  fallar: uno que se movio el sistema, otro que se quedo atras un
  documento.
  **Respaldo**: §31 explica por que la cifra paso de 59,1 a 120,4 —un pago
  son DOS pruebas desde la via en dos fases—; §304 explica por que paso de
  120,4 a 126,2 y, sobre todo, por que nadie lo vio en dos semanas.
  ⚠️ Correccion (§304): este cuerpo citaba
  `cost_per_transfer_stays_stable` como la guarda de la acumulacion. **No
  lo es**: esa mide la estabilidad del coste entre envios sucesivos. La
  guarda ancha de la acumulacion vive en `metrics_of_the_layer`, y es
  ancha a proposito —detecta un cambio de orden de magnitud, no el byte
  exacto—. Salto una vez, en la migracion, y tenia razon.
  ⚠️ **La UNIDAD ya no esta mal etiquetada** (§83.3, cerrado en §304): el
  arnes imprime MiB, y `PAPER.md`, `PAPER_EN.md` y `QUESTIONS.md` dan el
  equivalente SI cuando lo dan. Lo que queda de aquel punto son los tres
  preprints, bajo la **28**.
  ⚠⚠ **El desenlace por LOTE esta descartado, y por ESTRUCTURA.**
  `apply_many` consume recibos **ya probados** —`BatchOp` los lleva
  dentro— y devuelve `Result<(), LayerError>`: **apila pruebas, no las
  agrega**. Nacio contra la contencion del §204, que es un problema de
  tiempo, no de bytes. Por eso la puerta G0 del plan **no se declara
  verde ni roja: se declara INAPLICABLE**, porque medía bytes para
  evaluar una herramienta hecha para otra cosa. Lo MEDIDO es la
  estructura; lo RAZONADO es que la curva sea plana: **nunca se corrio un
  lote de 100**, y eso se declara en vez de absorberse. Queda viva la
  envoltura recursiva, con su puerta G1 sin jugar.
  ⚠️⚠️ **G1 JUGADA (§305): ROJA por COSTE, no por imposibilidad.** El
  verificador de la capa, extraido a un crate de solo-verificador sin la
  fachada `winterfell`, **corre dentro de una zkVM y dice VERIFICA** con
  una prueba real: 47.513.440 ciclos, 49 segmentos, 955 ms de ejecucion,
  `Halted(0)`. Probarlo es lo que no sale: medido en un portatil de ocho
  nucleos sin GPU, **2.109 ciclos/s** con segmentos de 2^18 y cuatro
  hilos y **2.621 ciclos/s** con 2^20 y ocho —doblar los hilos solo da un
  24 %, asi que el probador no escala aqui—, lo que proyecta **5 h 07 por
  UNA prueba** contra un umbral de sesenta minutos. Un pago son dos
  pruebas: unas 10,1 horas; mil pagos, 420 dias de CPU. ⚠️ **El rojo es
  de ESTA maquina**: ninguna con GPU esta medida, y ahi la cifra podria
  caer en minutos. **La nota NO se cierra**: M1b sigue estimada y no
  medida, y el eje de la decision se ha movido de «se puede» a **«quien
  paga los ciclos»**. Lo que un crate de verdad costaria vive en la 97.
  **Desenlace de G1 (§306): VERDE en GPU.** En una RTX 5090 la prueba
  composite del verificador dentro del zkVM tarda **40,9 s**, y comprimir a
  sucinto **23,8 s** mas; en el portatil sin GPU la proyeccion era 5 h 07.
  El coste deja de ser la barrera: un pago son **81,8 s** y mil pagos
  **~23 h de GPU**, contra los 420 dias de CPU del §305.
  ⚠️ **Y aparece el dato que cambia la forma de esta nota**: el receipt
  **sucinto mide 223.234 B**, o sea **3,3 veces mas que una prueba suelta de
  66.739 B**. Envolver UNA prueba no ahorra bytes: los pierde. La rama B
  ahorra por **AGREGAR**, no por envolver — el sucinto es de tamano
  constante, asi que el equilibrio esta en unas **cuatro pruebas** y a mil
  pagos el mismo receipt de 218 KiB frente a 127,8 MiB de STARKs da un
  factor de **600**. **No se cierra**: falta el coste del guest agregador
  —verificar N pruebas cuesta N x 47,5 M ciclos y no cabe en una sesion sin
  arbol de recursion— y M1b sigue estimada.
  ✅ **Constancia MEDIDA (§307).** Lo que la linea de arriba afirmaba como
  propiedad de la recursion **ya esta medido**: el receipt sucinto mide
  **223.234 B exactos** tanto a 49 segmentos (`po2 20`) como a **222**
  (`po2 18`), mientras el composite crece de 13.784.514 a **56.762.300 B** y
  su verificacion de 1.103,87 a **4.380,23 ms**. El control ve la diferencia
  por tres vias y el sucinto no se mueve por ninguna; y verificarlo cuesta
  18,2 ms en los dos casos. Sale ademas el **modelo de coste**: los joins son
  **N-1** y cada uno cuesta **~0,49 s** en una RTX 5090, asi que agregar N
  pruebas son **N x 24 s de compresion sobre N x 40,9 s de prueba**, con la
  salida quieta. ⚠️ **Sigue SIN medir el DIARIO**: el guest publica un `u32`
  y un agregador publicaria N digests — el unico termino que crece con el
  numero de pruebas distintas.
  ⏸️ **SUSPENDIDA (§318), Y LA ESCALA VA NOMBRADA.** No es cuello **por
  debajo de 100.000 pagos** — unos **12,4 GiB** de pruebas acumuladas, que es
  `AVISO_ACUMULACION_PAGOS` por `PUBLICADA_PAGO_B` y lo ata una comprobacion
  del nodo. **Donde se mide la verdad**: el umbral en
  `crates/zk-ssl-node/src/latido.rs`, el byte por pago en
  `crates/zk-ssl/src/metrics.rs`, y la aritmetica entre los dos en
  `el_umbral_de_acumulacion_cuadra_con_la_cifra_publicada`.
  ⚠️ **Y lo que significa cruzar esa escala, escrito al derecho: 12,4 GiB es
  la ultima en la que NO DUELE.** Sonar no dice que ya duela: dice que **se
  acabo el margen** y que la siguiente si. Escrito al reves — la ultima
  escala rutinaria — el aviso quedaria desacreditado el dia que sonara,
  porque 12,4 GiB se descargan de una sentada.
  ⚠️⚠️ **NO HAY CRUCE DE CURVAS, y por eso la escala no sale de comparar
  costes.** Agregar cuesta unos 65 s de GPU por prueba y ahorra unos 67 KB
  por prueba **a cualquier escala** (§307: 49 joins a ~0,49 s mas 40,9 s de
  prueba, contra 66.998 B): las dos magnitudes son lineales en N y la razon
  es constante, asi que **no existe un punto en que agregar se vuelva
  rentable por tamano**. Lo que NO es lineal es la CAPACIDAD de quien
  descarga — 12 GiB se bajan sin pensarlo y 12 TiB no —, y de ahi que el
  disparador vaya sobre stock acumulado y no sobre una comparacion.
  ✅ **Y no es una tolerancia declarada una vez: esta CABLEADA.** El latido
  del nodo cuenta los pagos del registro de transiciones — `Send` de la via
  de dos fases mas `Transfer` de la de un paso, que infravalorar los viejos
  seria contar solo los primeros — y avisa **una sola vez** con `tracing`
  y target propio. Un disparador que no esta cableado es un comentario; el
  precedente que lo enseno fue el aviso de `protoc` del entorno del pod, que
  salto solo. **Silenciarlo sin desactivarlo**: `--log
  zk_ssl_node::acumulacion=off`.
  ⚠️ **Lo que esta suspension NO afirma.** No dice que la rama B sea
  inviable: G1 esta VERDE en GPU (§306) y la constancia del receipt, medida
  (§307). Lo que se suspende es **construirla**, porque el trueque es malo
  hoy y lo seguira siendo a cualquier escala mientras el precio no cambie.
  **Lo que la reabriria**: que el byte por pago suba, que el ciclo de GPU
  baje, o que suene el disparador. Y el cambio de razon respecto al §305:
  **M1b sigue estimada y el coste del guest agregador sin medir**, pero
  ninguna de las dos importa mientras no se construya.

- [x] **93. El nodo MENTIROSO: la variante que hace ejercitables las
  defensas.** Instrumentacion, no modelo de confianza: una variante que
  sirva dos cabezas para el mismo indice, censure una entrada, omita un
  acuse o firme algo que su diario no recoge. Sin el, media docena de
  propiedades se quedan en «alcanzable, no ejercitada» — y `--ausentes`
  (§283) nunca dara rojo en un banco.
  **Como se garantiza que no llega a produccion**, de mas fuerte a mas
  debil: (1) **crate aparte del que el nodo no depende** —precedente
  medido: `zk-core` depende de `ceremony` solo como dev-dependency, a
  proposito, para el grafo aciclico—; (2) `#[cfg(feature)]` no-default,
  pero vive en el mismo crate y `--all-features` lo compila; (3) bandera
  en tiempo de ejecucion, la mas debil.
  ⚠️ Las mentiras que interesan —vista dividida, o una firma que el
  diario no recoge— **exigen la clave**: un proxy delante del nodo solo
  puede mentir por omision. La forma seria **una costura, no una rama**:
  el nodo expone el punto de extension, el binario de produccion
  construye solo la implementacion honesta, y la mentirosa vive en el
  crate aparte (patron «envoltorio en vez de cambio de firma», §281). Y
  lo que lo hace COMPROBABLE en vez de prometido: una compuerta del
  canon que verifique que el arbol de dependencias del nodo no contiene
  el crate mentiroso salvo como dev-dependency.
  ⚠️ Ningun auto-informe del nodo protege de nada: «yo soy honesto» es
  el operador hablando de si mismo.

  **RFC-0011, PROPUESTO (§588):** `spec/rfc/0011-el-nodo-mentiroso.md` fija este
  diseño -crate aparte, costura, compuerta del canon- con el censo de defensas medido, el
  coste de la costura (el nodo es hoy SOLO binario) y cinco etapas, la compuerta antes que
  ninguna mentira. Sigue abierta hasta que el autor lo acepte y las etapas se sellen.
  **Decisiones, §598** (delegadas por el autor, REVERSIBLES): ni crate ni biblioteca del nodo;
  las mentiras con clave las produce el binario honesto con la semilla -el banco del ancla ya lo
  hace-, y viven en bancos de `tools/`. Etapas: vista dividida contra el testigo, firma sin anotar,
  censura antes del recibo, recibo sin resolver.
  **E1, §599**: `tools/banco_mentiroso_vista.sh`. La vista dividida, servida por un nodo REAL: dos
  testigos que, cada uno solo, no ven nada, y `--comparar` ROJO con cada índice; y el contador
  restaurado bajo un testigo vivo, que se detiene, con `--auditar` ROJO en el índice. La primera fila
  de la tabla del RFC deja de ser «alcanzable, no ejercitada». Siguen E2 a E4.
  **E2, §600**: `tools/banco_mentiroso_ausentes.sh`. `--ausentes` da ROJO en un banco por primera
  vez: el operador reinicia con otro `--diario` y enseña sólo el nuevo, y una línea firmada se borra
  a mano; las dos veces nombra exactamente lo que falta, y contra la verdad entera sale VERDE. Siguen
  E3 y E4.
  **E3, §601**: `tools/banco_mentiroso_censura.sh`. La censura antes del recibo, con un proxy del
  operador delante de un nodo honesto: el titular sabe que no tiene recibo, el registro firmado no
  se mueve, y todas las defensas callan -asertado-. El residuo D-H del RFC-0010, medido. Sigue E4.
  **E4, §602**: `tools/banco_mentiroso_sin_resolver.sh`. El recibo que no se resuelve, por un nodo
  de verdad y sin código: el operador aplica, responde con el recibo y, antes de la cabeza siguiente,
  restaura el libro a una copia. El recibo queda firmado y el acuse no llega nunca; el testigo calla,
  y el sobre de completitud es la única defensa que lo ve, al expirar la ventana. **Las cuatro etapas,
  hechas, y D-B no se reabre**: queda el giro del RFC a ACEPTADO. Deja la 109 y la 110.
  ✅ **CERRADA en el §604: el RFC-0011, ACEPTADO.** Las defensas contra el operador que miente ya
  no están en «alcanzable, no ejercitada»: cada una se ejercita en un banco contra un nodo real que
  miente con la clave, con un fichero o con un proxy, y la única que no ve nada -la censura antes
  del recibo- tiene su silencio asertado. Lo que sigue vive en la 109 y la 110.

- [x] **94. Clases de resultado que el canon NO VE: las cegueras del
  instrumento, medidas — decididas por clase y declaradas; la (a) de lo que se
  compila sin el probador, vista en el canon (⚠️ §864).** El canon corre en release y cuenta los
  warnings de la compilación de TESTS; fuera de su vista quedan clases
  enteras de resultado, y ya hay CUATRO medidas más la que la 81 tenía:
  **(a)** los `debug_assert` sólo viven en DEBUG — hay tests de
  la capa que caen hoy en depuración («transition constraint degrees
  didn't match», winter-prover; verificado contra el árbol virgen el
  13-08-2026) y el canon es estructuralmente ciego a ellos. ⚠️ **La
  cifra no vive aquí**: es de la **41**, que posee los fallos, y del
  asiento §292, que la mide — esta nota es del INSTRUMENTO que no los
  ve, no de los grados. ⚠️ **§814: hoy no cae ninguno, medido a mano.** El
  15-10-2026, `cargo test --locked -p zk-ssl` en depuración no da ningún
  fallo, y la 41 se cierra en el §814 con su cifra. La ceguera de la (a)
  sigue entera: el canon corre en release y no vería el día que uno vuelva
  a caer; los que llevan `cfg_attr(debug_assertions, ignore = …)` no
  corren en depuración en ninguna orden; y el canon tampoco corre los
  `#[ignore]` sin condición: uno de ellos, el instrumento del vecino, no
  corre en ningún perfil y nada lo decía (la 125).
  **(b)** los warnings de `cargo build` no
  son los de `cargo test`: un accesor usado sólo por tests da
  cero warnings en el canon y warning en el build normal — cazado por
  un `--help` (§295, y corregido allí borrando el accesor).
  **(c)** la **81**: los warnings de un ejemplo, que el canon tampoco
  compila. Cerrarla exige decidir POR CLASE: o el canon gana pasadas
  nuevas con pines propios —un build normal; un `cargo test` en
  debug ACOTADO a los crates sin prover—, o la ceguera se declara
  PERMANENTE con su razón escrita. Lo que no vale es el estado de hoy:
  ceguera sin declarar, que es exactamente la clase de silencio que
  `check_cifras` vino a impedir con los números.
  **(d) La PROSA envejece sin que ningún gate se ponga rojo.** §297 lo
  midió en la cabecera de `zk-ssl-verify`: dos afirmaciones caducadas
  —«solo de xmss», que §292 dejó falsa, y «su superficie es el dominio, la
  versión y verificar_cabeza», que eran quince elementos—. **Y §275 y §279
  escribieron AL LADO que la superficie crecía, citando esa cabecera, sin
  subir la corrección al párrafo que contradecían**: no es que nadie lo
  viera, es que ningún instrumento contrasta un doc-comment contra lo que
  describe. **(e) `check_dominios` sólo ve literales de BYTES**: su
  `LIT_ZK` casa `b"ZK-SSL-..."`, y los dominios que viajan como
  cadena JSON —`witness.rs`, `node/main.rs`— son invisibles al censo,
  así que **R5 no los protege**. Puede ser por diseño —el registro cubre
  entradas de hash— pero no está declarado, y **el §298 va a agravarlo** en
  cuanto el diario del testigo escriba el dominio de la cofirma. Se nombra
  ahora porque el corte que agrava una ceguera es el que mejor la conoce.
  ⚠️ **§789: la (e), cerrada.** `check_dominios` gana R6: una cadena
  `"ZK-SSL-..."` en codigo de produccion tiene que ser el VALOR de un dominio
  de bytes declarado, o es ROJO con su sitio; las de las zonas `#[cfg(test)]`
  —un test escribe a proposito un dominio falso— no se juzgan, pero se
  cuentan, y si no hay ninguna el censo de cadenas esta ciego y es ROJO. Hoy:
  0 en produccion —ningun codigo de produccion escribe un dominio como
  literal de cadena— y 7 en tests. La entrada sigue abierta por las otras
  cegueras.
  ⚠⚠ **Y esta entrada nacio en ROJO, por su propio tema.** La primera
  version llevaba la cifra de los fallos de debug, y `check_figures`
  la tumbo: vigila LA PREMISA de la exclusion de este fichero —cero
  cifras de tests en entradas ABIERTAS—, y `check_cifras` lo
  aviso a la vez. La nota que habla de lo que el instrumento NO VE fue
  cazada por el instrumento en lo que SI ve. Queda escrito porque es el
  argumento de la propia nota, del derecho.

  **(f) Los BANCOS** tampoco los ve: viven fuera del canon, y el de la reutilizacion
  estuvo ROJO del §337 al §579 sin que nada lo dijera. ✅ **VIGILADA desde el §582, no
  cerrada**: `canon.sh --bancos` los corre todos, y TODA invocacion dice cuando pasaron
  VERDES y si lo que ejercen cambio desde entonces; nada obliga a correrlos.

  ⚠️ **§864: decidida POR CLASE, por delegación del autor («resuelvelas aplicando los principios»),
  con el criterio del §609 y el precedente del §642; REVERSIBLE; entra con su aceptación. Y se
  CIERRA en el mismo sello, con lo que pedía: en el canon, el bloque «Ceguera declarada», la
  clasificación comprobada contra `members` y la pasada `aserciones`, en `--sello`, con sus
  falsadores; la razón medida, en el asiento.** **(b)** y **(c)**, CERRADAS por el §834: cada
  `bin` y cada `example`, en release con `--locked`, con sus features y pin exacto de avisos por
  paquete; las bibliotecas ya las compilaba `cargo test` en su perfil normal —la fila de
  `ceremony` cuenta sus avisos «(lib)»—; fuera, declarado allí, el generador de `kat_xmss`; y el
  nodo sin `dev`, el del §784, cuyo contador pasa en este sello de cota sobre una lista de clases
  a pin exacto. Fuera del build normal queda, declarado al pie de `tools/canon.sh` con sus
  instancias medidas, el código que solo compila con una feature que no enciende ninguna fila ni
  ninguna puerta: `parallel` de `ceremony`, `test-setup` de `nova-experiment`, y en los forks,
  `concurrent`, `async` y el `no_std`. **(a)**: lo que un tercero compila sin el probador —el kit,
  `zk-ssl-verify`; su clausura, `zk-ssl-hash`, `zk-ssl-air` y `zk-ssl-medio`; y el guardián del
  testigo, `zk-ssl-guardian`— gana una pasada, el «1 bis» de `tools/canon.sh`, con un perfil
  `aserciones` del `Cargo.toml` raíz, que hereda de release y enciende `debug-assertions` en su
  propio `target/aserciones/`, con los pines de su fila y ROJO si difieren, y ROJO también si el
  JSON de cargo da un artefacto de la pasada sin las aserciones. Entra en `--sello` porque, en la
  medida del asiento, no pasó del doble de esas cinco filas en la misma corrida, ni en caliente ni
  tras tocar `zk-ssl-hash`; si una corrida posterior pasa del doble en uno de esos dos escenarios,
  baja a `--completo`, y la que compila un solo perfil —en frío, o con release al día y la pasada
  por compilar— se informa y no decide. Es lo que `CONTRIBUTING.md` §4 manda correr en depuración y
  nadie comprobaba fuera de la capa. La premisa de la (a), «hay tests de la capa que caen hoy en
  depuración», la retiró la 41 en su §814. En los crates con probador y en los de arkworks,
  PERMANENTE y declarada: no son la clausura del estándar, costaría como poco otra vez la fila más
  cara del sello (§225), y las marcas `cfg_attr(debug_assertions, ignore)` del probador son de la
  migración de Winterfell, que la reabre cuando las decida; los forks del juez los ejerce la pasada
  como dependencias, sin sus tests de upstream. Los `#[ignore]` sin condición, que no corre ningún
  nivel, quedan declarados; el del vecino es de la 125. **(d)** PERMANENTE: la prosa no admite un
  test discriminante (PRINCIPIOS §9); lo que tiene productor ya tiene herramienta, y lo demás se
  corrige con marca, como «y hoy lo hace» de `CONTRIBUTING.md` §2 en este sello. **(e)**, cerrada
  en el §789. **(f)**, como la dejó el §582. Todo, en «Ceguera declarada», al pie de
  `tools/canon.sh` —su cabecera lo nombra—, y cada miembro del workspace en una clase, comprobado
  en su bloque «0 bis»: ROJO si falta o sobra uno (`CONTRIBUTING.md` §1).

- [x] **100. Un nodo que ha firmado no vuelve a arrancar.** ⚠️ **RESUELTA
  en el §335.** La clave se resincroniza al contador al arrancar, y el
  contador se ata al **maximo indice anotado en el diario**: si ha retrocedido,
  se falla cerrada. Demostrado en vivo por el `NEGATIVO-A` del propio banco, que
  vuelve a dar su veredicto. El parche del `NEGATIVO-B2` **sigue pendiente** y va
  detras, como emision propia. Lo que sigue abajo es el texto original. El SK no se
  persiste: al reiniciar, la clave se rederiva de la semilla y su índice vuelve a
  **cero**, mientras el contador del guardián sobrevive en el fichero de
  `--indice-firma`. Esa pareja es exactamente `ClaveEnCero`, que desde el §331
  **falla cerrada y no arranca** —y hace bien, porque firmar ahí reutilizaría
  índices que pueden estar quemados—. Pero el efecto es que **cualquier reinicio
  posterior a la primera cabeza firmada deja el nodo muerto**. Medido en vivo:
  `tools/banco_cofirma.sh` muere en `NEGATIVO-A` con «LA CLAVE ESTA EN CERO Y EL
  CONTADOR EN 13», y ese literal vive **sólo** en `crates/zk-ssl-node/src/main.rs`,
  lo que desempata quién se negó a arrancar.
  ⚠️⚠️ **El banco lleva rojo desde el §331 y nadie lo vio en tres sellos**, porque
  el canon no corre los bancos: sus siete herramientas no los incluyen. Un banco
  que no se corre no es un adorno: es una afirmación caducada.
  ⚠️ El montaje del `NEGATIVO-A` lo agrava: pasa `--indice-firma` **sin el
  sufijo** que sí llevan el ledger, el diario y el contador de recepción, así que
  el contador del nodo sobrevive a un reseteo que se pretendía completo.
  Repararlo de verdad es territorio de la **84** y la **92**: que la clave sepa
  volver a su índice. Con esta entrada va también el parche del `NEGATIVO-B2`
  que el §334 dejó escrito y sin aplicar.
  ✅ **§338** · el parche del `NEGATIVO-B2` YA ESTA APLICADO. El banco
  distingue duplicar una linea -que no acusa a nadie- de repetir el indice sobre
  OTRO mensaje -que si-, y su VIVA corrio ENTERA por primera vez desde el §331.
  Lo que faltaba no era el parche, que llevaba escrito desde el §334, sino un
  arbol donde el TESTIGO reiniciara (§337).

- [x] **101. El ancho del indice esta declarado en DOS sitios y ninguno lo lee
  de `xmss`.** `zk-ssl-guardian::ancho_indice()` devuelve 5 y
  `zk-ssl-verify::ANCHO_INDICE` vale lo mismo; el §332 dejo el test que
  **los ata entre si**, y el doc de `verify` dice con todas las letras que es
  «la segunda copia, no una tercera». El residuo real es otro:
  **ninguna de las dos esta atada a lo que `xmss` sabe**, porque `full_height`
  e `index_bytes` son `pub(crate)`. Lo unico publico es el NOMBRE DEL TIPO,
  `XmssMtSha2_40_8_256`. Mientras siga asi, un cambio de conjunto de parametros
  no rompe ningun test nuestro: **valida mal, en silencio**. Tercera vez que la
  grieta asoma (§296, §298, §335) y primera que tiene entrada.
  ⚠️ El §335 la estrecha sin cerrarla: `poner_indice_en_sk` **no anade una
  tercera copia** —usa `ancho_indice()`— y su techo se DERIVA del ancho
  del campo, `2^(8*ancho)`, que aqui coincide con `2^h` porque 40 = 8x5.

  ✅ **CERRADA EN EL §586.** `zk-ssl-verify` gana
  `el_ancho_del_indice_sale_del_conjunto_de_xmss`: DERIVA el ancho del `NAME` del
  conjunto (`XMSSMT-SHA2_40/8_256`: h, d y n) con la regla de la propia `xmss`, lo
  confirma contra su `SK_LEN` y su `SIG_LEN`, y exige `8*ancho = h`, que es lo que hace
  que el techo del guardian sea `2^h`. La cadena queda `xmss` -> `verify` -> guardian,
  con el test del §332 como segundo eslabon. Falsadores: `Conjunto` en 60/6 tumba SOLO el
  nuevo -el del §332 sigue VERDE: era la grieta-, y `ANCHO_INDICE = 4` tumba los dos; en
  40/4, mismo ancho, pasa, como debe. ⚠️ **CORRECCION**: esta entrada decia «Lo unico
  publico es el NOMBRE DEL TIPO». Falso: el trait `XmssParameter` es publico, y con el
  sus constantes `NAME`, `SK_LEN`, `VK_LEN`, `SIG_LEN` y `SEED_LEN`.

- [x] **102. El borrado del buffer del SK es BEST-EFFORT, y upstream deja una
  copia sin borrar.** `resincronizar_a` saca los bytes del SK a un `Vec` para
  parchear el indice y lo borra con `zeroize` al terminar; el SK **viejo** si se
  zeroiza solo, porque `SigningKey` tiene `Drop`. Pero un `Vec` pudo REUBICARSE
  mientras se construia, asi que borrar el ultimo puntero **no promete nada
  sobre las copias intermedias**, y el doc lo dice.
  ⚠️⚠️ Y hay una fuga que no es nuestra: `KeyPair::from_seed` construye el
  SK en un `Vec`, lo copia a su `Array` y **suelta el `Vec` sin borrar**, en cada
  arranque. Leido en `xmss.rs:620-645`; `init_keypair_buffers` NO esta leida, asi
  que la lectura es parcial y se declara. No se arregla aqui —es de un crate
  ajeno, una herencia por corte—: se DECLARA.
  ✅ **CERRADA EN EL §772.** Lo nuestro, arreglado: el buffer del SK es
  `zeroize::Zeroizing` desde que nace en los dos `resincronizar_a`, el del nodo y
  el del testigo, y en `firmar_con_la_anterior` del nodo, que tenía el mismo
  defecto; se borra al soltarse por CUALQUIER salida, también por los `?` que
  hasta el §772 lo soltaban sin borrar. Y no se reubica: dos tests atan que el
  puntero, la capacidad y la longitud salen del parcheo como entraron, y otros
  dos que una resincronización que no cabe falla cerrada sin mover la clave ni
  el contador. Lo de upstream, leído entero y DECLARADO: `init_keypair_buffers`
  (`params.rs:1168-1198`) solo reserva el `Vec` en ceros con el OID; el secreto
  lo escribe `from_seed` y esa copia se suelta sin borrar en cada arranque. Es
  de `xmss`, y sigue en `doc/CONFIANZA_RESIDUAL.md`.

- [ ] **103. El gate del contador contra el diario se apaga quitando un
  fichero.** Con `--diario` ausente o vacio no hay segundo testigo y el arranque
  resincroniza igual, que es lo correcto —el caso del `NEGATIVO-A` es
  exactamente ese— pero significa que **una rotacion de logs rutinaria apaga
  la comprobacion sin mala fe**. Se midio que el nodo NO rota el diario, asi que
  hoy es accion de operador y no camino de codigo. Y ante una restauracion del
  DIRECTORIO ENTERO no hay nada que hacer: contador y diario vuelven juntos.
  **Dos testigos en el mismo disco son un solo testigo frente a una
  restauracion.** Declarado tambien en `doc/CONFIANZA_RESIDUAL.md`.
  ⚠️ §779: **recontada, y SIGUE ABIERTA.** Lo de arriba sigue cierto en el codigo: el
  tope sale de `diario::maximo_indice`, que da `None` con el fichero ausente o sin
  ninguna firma, y con `None` el arranque resincroniza o arranca sin mirar nada
  (`politica_de_reconciliacion`, en `crates/zk-ssl-node/src/main.rs`); el nodo sigue
  sin rotar el diario. Desde que nacio se ha estrechado por cuatro sitios, y ninguno
  la cierra: el §594 consulta el diario en TODO estado que lea un contador, asi que
  un contador borrado con el diario en su sitio ya no pasa; el testigo de fuera del
  disco ve el directorio restaurado (`vista-dividida`, §599) y el diario quitado
  (`--ausentes`, §600), medido contra un nodo real; desde el §666 un byte que no es
  UTF-8 se lleva su linea y no el diario entero, que antes dejaba el arranque sin
  tope como un fichero quitado; y cuando la perdida SE SABE, la D-G del RFC-0015
  tiene procedimiento, `--desde-minimo` (§690) con la hoja que da `--auditar` (§770).
  **Lo que le queda es la perdida que NO se sabe**: el nodo solo no ve ni el
  directorio restaurado ni el diario quitado, y lo de fuera lo ve DESPUES de firmar
  la hoja repetida, cuando la clave ya se puede extraer (RFC-0015, la fila del
  indice perdido). La cerraria un segundo testigo FUERA del disco consultado ANTES
  de firmar, y lo construido y lo propuesto ven despues: el medio del RFC-0013 la
  estrecha —sus testigos recuerdan el `size` que cofirmaron (D-D)—, pero su arbol
  es de anclas de cabezas ya firmadas, su E3 esta pendiente y no cubre «el diario
  ausente sin ancla publicada». Su gemela en el testigo, el fichero de cofirmas, es
  la 104, cerrada en el mismo §779 como declaracion y no como arreglo: el testigo lee
  sus cofirmas con `read_to_string` y no tiene el arreglo del §666.

- [x] **109. El sdk del titular TIRA el recibo: quien paga con él no puede armar
  su sobre de completitud.** Medido en el §602: `zk-ssl-sdk` no lee `recepcion`
  ni `acuse` de la respuesta de `zkssl_applySend` ni de `zkssl_applyClaim` —ni
  una mención en `src/lib.rs`— y `Account::pay` devuelve sólo el aviso. El
  RFC-0010 ata al operador con un recibo que el TITULAR custodia; con el sdk de
  la casa, el titular no lo custodia. El banco del §602 lo suple con un proxy de
  registro de su lado, y eso es instrumentación, no una salida para un titular
  de verdad. **Lo que pide**: que `pay` y `claim` devuelvan el recibo y el acuse
  tal como llegaron -el sobre los compara con lo que el cierre firma-. Es código
  del sdk: no toca el nodo, el cable ni los vectores.
  ✅ **CERRADA en el §606**: `Account::pay` devuelve un `Pago` -el aviso y su
  `Constancia`- y `Account::claim`, su `Constancia`: el recibo, el acuse y el
  `logSeq` de la respuesta tal como llegaron, y `None` a la vista si el nodo no
  dio recibo. Dos tests sin nodo; el ejemplo `e2e` imprime las constancias, y el
  banco del §602 deja su proxy y arma los sobres con ellas.

- [x] **110. El veredicto 1 del sobre de completitud ya se produce en vivo: falta
  su vector.** El RFC-0010 se aceptó con ese residuo declarado (§577): el
  veredicto 1 pedía una prueba STARK real de un envío aplicado. El banco del §602
  la pone -el ejemplo `e2e` del sdk contra un nodo `--dev`- y el sobre con su
  acuse sale VERDE. Capturarlo como vector cierra el residuo: un `--guardar` en
  el banco, la entrada en `spec/vectors/completitud/` y su manifiesto, y los
  pines del canon y del artefacto, que pasan de 35 a 36. Corte propio: mueve el
  catálogo, el canon y el tarball.
  ✅ **CERRADA en el §605**: `resuelta-por-acuse`, el cuarto positivo de
  `spec/vectors/completitud/`, copiado de `banco_mentiroso_sin_resolver.sh
  --guardar`; el arnés da 36 de 36, desde el árbol y desde el tarball.

- [x] **111. Tres ejemplos del sdk llevan rotos desde el §261.** Medido en el §614:
  `h1_techo_apply`, `i1_concurrencia` y `j1_lote_mixto` llaman a
  `zkssl_sendMaterials` -y el último también a `zkssl_claimMaterials`- sin la clave
  de VISTA que esos brazos exigen desde el §261, y el nodo los rechaza con
  `missing field viewKey`. Compilan, así que el canon no los ve: son instrumentos de
  medida que nadie ha vuelto a correr. `d2_lote_rpc` tenía el mismo fallo y se
  arregló en el §614 porque el banco del recibo agregado lo usa. **Lo que pide**:
  mandar `viewKey` desde la cartera, como hace `Account`, y correrlos una vez.
  Toca sólo ejemplos: ni el nodo, ni el cable, ni los vectores.
  ✅ **CERRADA en el §615**: los tres mandan la clave de vista y corren de punta a
  punta contra un nodo `--dev`. Y al correr `h1_techo_apply` salió lo que ningún
  cerrojo miraba: su último lote, de 15, ya NO cabe bajo el muro del cuerpo -una
  operación pesa hoy ~159 KB y no los 132,7 KB del §218-, así que caben 13; el
  ejemplo, `spec/RPC.md`, `spec/README.md` y el comentario del nodo lo dicen ya.

- [x] **112. Las citas por línea de `spec/README.md` a `spec/RPC.md` están
  desfasadas.** Medido en el §615: la guía «Reading `RPC.md` in order» cita
  tramos como `RPC.md:133-142` para «Errors», y en 221170f -antes de los sellos
  del RFC-0014- ese tramo ya empezaba en la sección `dev_*`; de las diecisiete,
  sólo la del transporte empieza en su encabezado. `verificar_citas` censa nombres y secciones, no
  números de línea, así que nada lo ata. **Lo que pide**: recomputar cada tramo
  por su encabezado y, mejor, que un cerrojo lo compruebe o que la guía cite por
  sección y no por línea.
  ✅ **CERRADA en el §616**: la guía cita por ENCABEZADO -`RPC.md` §«Errores»,
  la forma que `ARQUITECTURA.md` ya usaba- y `verificar_citas` (v3) comprueba que
  cada encabezado citado abre uno vivo del documento: treinta, ninguno muerto. Al
  estrenarse nombró el precedente, que estaba roto: citaba en `README.md` una
  sección «Estado y límites» que no existe.

- [x] **113. La cifra de vectores de `README.md` no la ata nada.** Medido en el
  §616: la tabla «Estado» decía que `spec/vectors/` tiene 380 ficheros, y en
  221170f tenía 381; el §610 le sumó un KAT y el §614 treinta y siete vectores sin
  moverla, porque `check_figures` sólo vigila cifras de tests. Corregida a 419 en
  los dos README. **Lo que pide**: que un cerrojo la compare con `git ls-files
  spec/vectors`, o que la tabla no dé un número que nadie recuenta.
  ✅ **CERRADA en el §617**: el ATADO E de `check_publicadas` cuenta los ficheros
  de `spec/vectors/` y exige que `README.md` y `README_EN.md` digan ese número, y
  que ninguno de los dos deje de decirlo sin salir de su lista con su razón.

- [x] **114. Un test del canon da ROJO por azar: el de la correlación entre
  importe y tamaño de prueba.** Medido en el §620: el primer `--sello` sobre ese
  commit, que no toca código, cayó en `proof_size_does_not_correlate_with_amount`
  con r lineal −0,791 (el umbral es 0,7) sobre 16 pruebas. Diez corridas más de
  ese test, todas en verde: r entre +0,03 y +0,43. Agrupadas las once (176
  pruebas, centradas por corrida), log2(importe) frente a tamaño da r 0,053, con
  p 0,49 por permutación: ni rastro de dependencia del importe. **La causa**:
  desde el §538 el tamaño de la prueba es aleatorio de una corrida a otra (la
  ocultación lleva sal de la entropía); su distribución tiene cola izquierda
  (asimetría −0,79); y la r LINEAL contra importes en potencias de dos la deciden
  los dos últimos puntos. Bajo la hipótesis nula, con esas 176 medidas, la
  compuerta lineal salta en el **0,5 %** de las corridas, y la logarítmica en
  torno al 0,1 %: un canon que un tercero corre puede dar ROJO sin que nada esté
  mal. **Abierto además**: en esas once corridas el índice 13 (importe 2^16)
  sale +457 B sobre la media de su corrida en diez de once (t +4,5; con
  dieciséis índices, p ≈ 0,02). El test no puede separar el importe del ORDEN en
  que se genera cada prueba, ni de la sal de su compromiso, que va por índice.
  **Lo que pide**: medir con los importes barajados por corrida -importe, orden
  o nada-, y después una compuerta calibrada (permutación, con falsa alarma
  declarada y muy por debajo del 1 %) que conserve la potencia contra una fuga
  grosera. Ni se salta ni se ignora el test mientras tanto.
  ✅ **CERRADA en el §621**: con los importes BARAJADOS por corrida (12 corridas,
  192 pruebas) ni el importe ni la posición explican el tamaño (p 0,64 y 0,65), y
  el índice 13 baja a +113 B: era azar. La compuerta es ahora un p de permutación
  con semilla fija, 32 pruebas (los 16 importes dos veces, la segunda al revés) y
  umbral 1e-4 por escala: falsa alarma ≤ 2e-4, y más potencia que la vieja
  (simulado con el ruido medido: frente a una correlación verdadera de 0,7, el 83 %
  contra el 52 %). Cuesta 31 s en vez de 15.

- [x] **120. La exportación de la instantánea no es atómica: trunca en sitio la anterior (§812).**
  `export_snapshot` abre el destino con `File::create` (`crates/zk-ssl/src/snapshot.rs:304`), que lo
  trunca, y escribe encima. Un proceso que muere a mitad deja en el sitio de la instantánea buena una
  a medias, que no importa —la importación verifica las raíces—, y la anterior ya no está. Y la
  anterior, truncada, deja sus bloques al sistema de ficheros: si iba en claro, o sellada con una
  frase que se acaba de cambiar con `recifrar`, sigue en los bloques libres. Visto al leer el código
  para la E4 del RFC-0001 (§812), sin medir una muerte a mitad; lo declaran el RFC-0001 y
  `SECURITY.md` §3.14. **Lo que pide**: que la exportación escriba entera a un temporal junto al
  destino, lo sincronice, lo cambie de nombre por el destino y sincronice el directorio, como el
  keystore (`escribir_entero`, en `crates/zk-ssl-sdk/src/keystore.rs`), y que barra el temporal que
  deje un proceso muerto. **Qué la cierra**: un test que mata la exportación a mitad y encuentra en
  el destino la instantánea anterior, entera, que importa; con su mutante, la escritura en sitio.
  ✅ **CERRADA EN EL §820.** `export_snapshot` escribe la instantánea entera en
  `{instantánea}.exportando`, en su mismo directorio, la sincroniza, la cambia de nombre por el destino y sincroniza el
  directorio (`escribir_entera`, al final de `snapshot.rs`). **Qué la cierra**, cumplido: con una
  muerte inyectada a mitad (`una_exportacion_muerta_a_mitad_deja_la_anterior_y_su_temporal_no_se_confunde`)
  y con un error inyectado a mitad (`una_exportacion_que_falla_a_mitad_deja_la_anterior_entera`), el
  destino sigue siendo la anterior, byte a byte, e importa; y una que acaba deja la nueva. Su
  mutante, la escritura en sitio otra vez con los mismos puntos de inyección, cae en los dos porque
  la anterior se tocó. El temporal lleva lo que llevaría la nueva —cifrado si la capa tiene clave, en
  claro si no: una capa sin clave que exporta sobre una cifrada, como la importada de una (la 121; ⚠️ §821: la importada con clave, ya no: desde el §821 conserva la clave, y la 121 está cerrada),
  deja un claro junto a ella, el mismo que dejaría si acabara—, nace 0600 en Unix, y la importación
  no lo lee. De lo que pide, una cosa no se hace así: el
  temporal que deja una exportación muerta **no se barre**, porque la biblioteca no sabe si es de
  una exportación viva, y borrarlo rompería el entera o nada de esa; la exportación siguiente a ese
  destino rehúsa nombrándolo, y se borra a mano. No barrerlo es de la sesión, y reversible: falla
  cerrado. Y dos que no pedía, porque el cambio de nombre las abriría: la nueva toma de la anterior,
  con privilegios, su dueño —sin ellos, es del usuario del proceso—, y su grupo y sus bits de modo
  —si no puede tomar su grupo, se queda sin los bits de grupo y de otros, que con otro grupo
  abrirían la instantánea a quien antes no la leía; una nueva nace 0600, y hasta el §820 nacía con
  0666 menos la umask—, y un enlace como destino se rehúsa
  (`la_exportacion_guarda_los_permisos_de_la_anterior_y_no_sustituye_lo_que_no_es_un_fichero`); un
  FIFO, un socket o un dispositivo, también, sin test. Un directorio también, por la misma guarda y
  con test; ya lo rehusaba `File::create`, y lo rehusaría igual el cambio de nombre (`EISDIR`). Lo
  que la escritura en sitio conservaba y la nueva no toma: las ACL POSIX de la anterior y sus demás
  atributos extendidos. Con una ACL extendida, los bits de grupo del modo son su máscara, y la nueva,
  sin ACL, se los da al grupo dueño: medido en la sesión con la capa, con un test que no queda en el
  árbol, una anterior 0640 con `user:4242:r--`, `group::---` y `mask::r--` deja una nueva 0640 sin
  ACL, que el grupo dueño lee —en claro, si la capa no tiene clave— y antes no leía. Copiarlas pide
  `fgetxattr` y `fsetxattr`, una dependencia que la capa no tiene; se declara, sin test. Cambia lo
  que hace la API en dos cosas más: exportar pide poder
  escribir en el directorio del destino, para crear el temporal, y `/dev/stdout` —un enlace— o un
  FIFO ya no son destinos (una copia por una tubería). Los bloques de la anterior se liberan al
  borrarla como se liberaban al truncarla: lo que guarden no lo alcanza la biblioteca (`SECURITY.md`
  §3.14). Sin medir: un corte de corriente —quitar el `sync_all` del temporal y la sincronización
  del directorio no tumba ningún test de la capa, medido—, y que un destino que no se puede abrir
  para escribir se rehúse (en la máquina de la sesión los tests corren como root).

- [x] **121. Una capa importada de una instantánea cifrada nace sin clave, y se reexporta en claro
  (§812).** `import_snapshot_with_key` descifra la instantánea con su frase, pero la capa que devuelve
  lleva `key: None` (`crates/zk-ssl/src/snapshot.rs:460-463`: «quien la persista decidirá con qué
  clave»), y `export_snapshot` exporta en claro, con la marca `0x00`, la capa que no tiene clave.
  Medido por una de las revisiones de la E4 del RFC-0001 (§812): una instantánea cifrada, importada con
  su frase y vuelta a exportar, sale con la marca `0x00`. Quien restaura una copia cifrada y la
  vuelve a guardar la deja en claro, y nada lo dice. Lo declaran el RFC-0001 y `SECURITY.md` §3.14.
  **Lo que pide**, y lo decide el autor: que la capa importada con clave conserve la clave, fijada a
  la cabecera de la instantánea, y exporte cifrada; o que exportar sin clave una capa que vino
  cifrada se rehúse. Las dos cambian lo que hace la API. **Qué la cierra**: un test que importa una
  instantánea cifrada y la vuelve a exportar, y la encuentra cifrada —o la exportación rehusada—, con
  su mutante; y la marca en el RFC-0001 y en `SECURITY.md` §3.14.
  ✅ **CERRADA EN EL §821**, con la primera de las dos: la decidió el autor el 21-10-2026, a la
  pregunta de la sesión, «Conservar la clave»; la otra, rehusar exportar sin clave, no. La capa que
  devuelve `import_snapshot_with_key` sale con la clave fijada a la cabecera de la instantánea, la
  que `abrir_sellada` fija para abrirla (`conservar_la_clave`, al final de `snapshot.rs`), y
  `export_snapshot` la reexporta cifrada, sin cambiar ningún formato: de una v2, con la marca `0x02`
  y la misma cabecera, byte a byte, y sin otro Argon2id (se deduce del código, y sólo lo ve el
  tiempo: medido en el asiento). De una v1 —la marca `0x01`, SHA-256 sin sal, que desde el §702 sólo
  se lee— la sesión elige, reversible, lo que falla cerrado: la clave se fija a una versión 2 de sal
  nueva, como `open_encrypted` migra un libro v1, y la reexportada lleva la marca `0x02`, con la
  misma frase, a costa de un Argon2id al importar; con una clave ya fijada, que no lleva la frase,
  no se puede, y la importación falla con un error que lo dice. Reexportarla en la v1 escribiría lo
  que nada escribe desde el §702. **Qué la cierra**, cumplido:
  `una_capa_importada_de_una_cifrada_se_reexporta_cifrada_con_su_cabecera` y
  `una_capa_importada_de_una_v1_se_reexporta_en_la_version_2` importan una cifrada, de la v2 y de la
  v1, la reexportan y la encuentran cifrada, en la v2, que abre con la misma frase y no sin clave ni
  con otra frase, con los saldos, el suministro y las cinco raíces de la de partida; la de la v2,
  también después de abrir una cuenta en la capa importada, y la de la v1, con otra sal en cada
  importación. Su mutante, la capa importada otra vez sin clave, cae en los dos, con la marca
  `0x00`; los demás, en el asiento. La marca, en el RFC-0001 («La instantánea.», y dos en la E2: su
  fila de la tabla de etapas y su sección), en `SECURITY.md` §3.14 y dos en la §3.16. Cambia lo que
  hace la API en dos cosas, que se declaran: una capa importada con clave y reexportada sale
  cifrada, y no en claro; y una v1 ya no se importa con una clave fijada sin la frase. Lo que no
  cambia: la capa de `import_snapshot`, sin clave, se reexporta en claro, y una capa sin clave que
  exporta sobre una cifrada deja un claro, como dice la 120.

- [x] **122. El TLS del cli y del SDK enlaza curvas, y `check_curvas` no lo ve (§835).** `ureq`
  2.12.1 entra en `zk-ssl-cli` y en `zk-ssl-sdk` con su `tls` por defecto, y trae `rustls` sobre
  `ring`: X25519, P-256 y P-384 para el intercambio, y verificación ECDSA P-256/P-384 y Ed25519 (y
  RSA, que no es curva) de certificados y del handshake. `ring` no está en el patrón de
  `tools/check_curvas.py`. Y la vía no pide ninguna dependencia nueva: `ureq` reexporta `rustls`
  (`pub use rustls;`), y el cli y el SDK pueden hoy hacer un X25519 o verificar Ed25519 o ECDSA con
  `ureq::rustls::crypto::ring`, sin tocar `Cargo.toml` ni `Cargo.lock`, medido en el §835 con un
  crate aparte. Ninguna puerta que mire nombres de crate la cierra mientras `ureq` lleve `tls`.
  `zk-ssl-verify`, `zk-ssl-node` y `zk-ssl-medio` no enlazan `ring`. Al nodo viaja la clave de VISTA
  (`crates/zk-ssl-sdk/src/lib.rs:192`), y el nodo no tiene TLS: `SECURITY.md` §3.3 pide un proxy
  delante. **Opciones**, y decide el autor: (a) enmendar el §106 para el transporte, declarado como
  el §705; su coste: la confidencialidad de la clave de vista en tránsito descansa en un ECDHE
  clásico, expuesto a guardar ahora y descifrar después. (b) quitar `tls` a `ureq`, y hoy eso es todo
  el cli y todo el SDK: sus agentes toman la URL del usuario y ningún fuente escribe una `https://`;
  su coste: la clave de vista viaja en claro hasta el proxy, contra `SECURITY.md` §3.3, o hace falta
  un túnel fuera del árbol, que saca las curvas del `Cargo.lock` sin quitarlas del camino. (c) no es
  alternativa a la (a), sino su complemento: con la excepción del transporte declarada, `ring`,
  `aws-lc-rs`, `rustls` y `rustls-webpki` en el patrón con una regla por aristas (`ring` y
  `rustls-webpki` solo bajo `rustls`, y `rustls` solo bajo `ureq`), y un grep de `ureq::rustls`,
  `ring::` y `webpki::` en los fuentes de las raíces, con su mutante. Una excepción por nombre o por
  raíz no basta, y sin el grep la (c) declara la ceguera en vez de cerrarla. El publicador del medio
  no elige transporte hasta que esto se decida. **Qué la cierra**: la opción elegida, con su marca en
  el §106 o en la cabecera de `check_curvas`; si es (c), con sus mutantes.
  ✅ **CERRADA en el §838**, con la (a) y la (c), que decidió el autor el 2026-10-15. El §106 se
  enmienda para el transporte del cli y del SDK, como excepción declarada al modo del §705, con su
  coste escrito en el asiento y en `SECURITY.md` §3.3. `tools/check_curvas.py` pone `ring`,
  `aws-lc-*`, `rustls`, `rustls-webpki` y `webpki` en el patrón; su R4 admite el transporte solo en
  el cli y el SDK y solo por sus aristas (`ureq` bajo una de esas dos raíces, `rustls` bajo `ureq`,
  `rustls-webpki` bajo `rustls`, `ring` bajo `rustls` o `rustls-webpki`; `aws-lc-*`, nunca); y su R5
  da ROJO a `ring::`, o a `rustls` o `webpki` como palabra, fuera de comentarios en las fuentes de los
  crates de la casa de cada clausura. Los mutantes, en el asiento. Queda abierto, y no en esta
  entrada: el transporte del publicador del medio, que no enlaza TLS y, si lo necesita, amplía R4
  en un sello propio.

- [ ] **123. Un nodo vivo que no contesta en 2 s con la máquina sobresuscrita (§813).** Medido el
  15-10-2026 sobre `b6994c3`, en 4 CPU con ocho `yes` y carga de 6,6 a 8,7, para el DOS NODOS de
  `tools/banco_reinicio.sh`: de 100 sondas `zkssl_supply` con 2 s de tope a un nodo vivo en cada
  caso (200 en total), **10 quedan sin respuesta SIN segundo nodo y 5 CON él**. Las que contestan
  lo hacen con mediana de 1 ms, p90 de 9 a 12 ms (⚠️ §829: no se reproduce; con ocho `yes` y el nodo
  a `nice` 0, en dos series de la otra sesión pasa de 0,5 s: 1,82 s sobre `7ed7e17`, y sin cifra
  impresa sobre `b6994c3`; el de aquí es de su fecha: abajo) y máximo de 1,9 a 2,0 s, y las que no,
  reintentadas con 30 s de tope, contestan todas, en 0,6 s como mucho. El segundo nodo no lo
  explica, y no se ha medido nada más: ni dónde se va el tiempo —el planificador, el latido de
  1 s, el runtime, la escucha—, ni sin carga, ni con otra sonda (⚠️ §829: sin carga lo midió
  después la otra sesión, y las sondas lentas solapan un latido, con carga y sin ella; dónde se va
  el tiempo dentro del nodo sigue sin medirse: abajo). Lo que hace el banco desde el §813:
  el DOS NODOS sondea al primero con hasta cinco sondas de 2 s seguidas en vez de una -la misma
  sonda que su arranque, que hace hasta cien con 0,1 s entre una y otra-, porque lo que comprueba
  es que sigue vivo y contesta. **Lo que pide**: medir dónde se va el tiempo de una sonda sin
  respuesta, con el reloj del nodo al recibir y al contestar, con y sin carga, y decir si un
  cliente del cable con un plazo de 2 s puede dar por caído a un nodo vivo. **Qué la cierra**: esa
  medida; si es el nodo, el arreglo con su test, y si es la máquina, la cifra dicha donde el cable
  dice sus plazos.

  ⚠️ **§829: la medida de la otra sesión, con su control, y lo que recomienda el autor.** La midió
  otra sesión que trabaja en el árbol, no la que lo sella, sobre `7ed7e17`, con datos de `b6994c3`
  para comparar, en 4 CPU, y la mandó el 22-10-2026 en un mensaje y en una respuesta que lo corrige
  y que dice contrastada por dos verificadores con las transcripciones. Sus registros y sus guiones
  se borraron: queda lo que imprimieron los agentes de su triaje. Esta sesión no la ha medido ni
  repetido; lo leído lo leyó en el árbol, y lo deducido no se midió.
  - **El control: el cerrojo ya no falla.** En 26 pasadas de `tools/banco_reinicio.sh` sobre
    `7ed7e17`, 11 con `BANCO_REINICIO_N=20` y 15 con 50, el SOLAPE hizo 970 rearranques: 940 a la
    primera, 30 tras esperar al cerrojo y ninguno se niega (sin carga añadida, 418 y 2 en 15; con
    carga, 522 y 28 en 11). Los 1.940 del REINICIO, a la primera, no los apunta su registro de
    serie: salen de la lógica del banco y del total de un agente de su triaje. Otras seis pasadas,
    de otro agente y con otra tarea a `nice` 10 en la máquina, dan en el SOLAPE 180 más, 176 y 4,
    sin negativas. Ningún `WouldBlock` llegó a error.
  - **Lo que queda es esta entrada.** El DOS NODOS sale ROJO, «el primer nodo dejo de contestar»,
    con el nodo vivo, en 4 de las 5 pasadas con ocho `yes` a `nice` 0 y el banco a `nice` 10 —tres
    de `N` 50 y una de 20, de esas seis; la otra salió VERDE, el DOS NODOS en 3.021 ms—, y en
    ninguna de las otras 27; en esas cuatro, el REINICIO y el SOLAPE pasan enteros. Con nodo y carga
    a la misma prioridad, VERDE siempre, de 620 a 926 ms. Los ROJO, dice el mensaje, se lanzaron a
    mano, con los `yes` a `nice` 0, y `serie.sh` usaba `nice` 10: esa receta no se repite tal cual;
    la respuesta, que las 26 las lanzaba un guion de serie, y no cómo se lanzaron los `yes`.
  - **El RPC no contesta mientras dura `latir`, también sin carga.** En 100 sondas sin carga sobre
    `7ed7e17`, la mediana del `ms=` de los 20 latidos de su ventana es de 804 ms (802,5 ms en
    `b6994c3`), y el latido ocupa el 44 % del tiempo. Las 20 sondas lentas solapan un latido; de las
    80 rápidas, ninguna empieza dentro de uno, cuando por azar lo harían unas 35. Las lentas acaban
    de 7 a 11 ms antes de la línea «cabeza de epoca FIRMADA», y con ocho `yes`, de 13 a 44 ms antes.
    Con ocho `yes` y el nodo a `nice` 0, quedan sin respuesta 4 de 100 sondas en `7ed7e17` y 10 de
    100 en `b6994c3`, y 0 de 100 en otra serie sobre `7ed7e17`: no es una regresión. Con el nodo a
    `nice` 10, el latido dura de 14 a 16 s, y quedan sin respuesta 66 de 100 sondas, y 30 de 50 en
    otra, que corrió con la otra tarea, como unos 35 s del final de la de `b6994c3`.
  - **El p90.** El 1,85 s del mensaje es de la serie de `b6994c3`, con las 10 sin respuesta dentro,
    y no se compara con el p90 «de las que contestan» de arriba; el comparable es el de la serie
    sobre `7ed7e17` en que contestaron las 100: 1,82 s. En la de `b6994c3`, 11 de las 90 que
    contestan pasan de 0,5 s, y su p90 también, sin cifra impresa. Los 9 a 12 ms no se reproducen.
  - **La sonda** no es la del banco: son guiones de la otra sesión, que arrancan un nodo con los
    argumentos del banco, `--latido 1` incluido, salvo tres: `--log info`, el `nice` del nodo y el
    puerto; lo sondean con `zkssl_supply` y `curl --max-time 2`, con 0,2 s entre una y otra, y un
    cruce con la línea «cabeza de epoca FIRMADA … ms=» de cada latido da los 804 ms y el 44 %.
  - **Los matices.** El máximo de 0,80 s que el mensaje da «con la máquina libre» es de una serie de
    60 sondas lanzada al acabar una con carga, con 4,87 a 1 minuto al empezar. El porcentaje con un
    latido cada 60 s, el de omisión (`crates/zk-ssl-node/src/latido.rs:72`), no está medido: el
    periodo es `cada` más lo que dura `latir`, y da 804 / 60.804, el 1,32 %, y de 1,24 a 1,55 % con
    latidos de 752 a 947 ms, deducido. El `ms=` mide `latir` entera, no solo la firma, y no se puede
    comparar con los 144,5 ms del comentario: leído.
  - **La pista del mecanismo, sin medir.** `latir` es síncrona y corre dentro de un `tokio::spawn`
    (`crates/zk-ssl-node/src/latido.rs:413-421`), y `#[tokio::main]` no fija `worker_threads`
    (`crates/zk-ssl-node/src/main.rs:1755`): leído. Lo que retiene al RPC —el ejecutor, un candado o
    la escucha— NO está medido.
  - **Lo que comprobó la sesión que lo sella.** El banco arranca sus dos nodos —el de `arranca`, la
    `:60` de `tools/banco_reinicio.sh`, y el segundo del DOS NODOS, la `:116`— con `--latido 1` y
    `--log warn` (`:62` y `:118`), con el que no sale la línea del latido, un `tracing::info!`
    (`crates/zk-ssl-node/src/latido.rs:429-431`); el latido duerme `cada` y luego llama a `latir`
    (`:417-422`); y los 10 de 100 en `b6994c3` coinciden con los de arriba sin segundo nodo.
  - **Lo que recomienda el autor**, como titula el mensaje lo que el autor decidió el 22-10-2026 en
    el chat de la otra («sí a todo lo recomendado») y confirmó en el de la que lo sella («si»):
    sellar ahora solo esta nota, con la medida y su control; arreglar `latir` más adelante, cuando
    se mida el mecanismo; y no alargar las sondas del DOS NODOS, descartado porque taparía esta
    entrada. Sigue abierta; lo que pide y lo que la cierra no cambian. `AUDITORIA.md` §829.

- [ ] **124. Nada ata una clave de firma a un solo libro ni a un solo contador (§813).** Medido el
  15-10-2026, en la revisión del §813: un nodo B arranca con la MISMA `--clave-fichero` que un nodo
  A, sobre otro `--ledger`, con otro `--diario`, otro `--contador-recepcion` y otro
  `--indice-firma` -lo que recetaba la negativa del libro tomado en un borrador del §813, antes de
  su revisión-, y firma. B vuelve a empezar por la hoja XMSS 1, y **la misma hoja de la misma
  clave firma dos mensajes distintos**: con A parado y una cuenta abierta solo en B (en `--dev`),
  la hoja 2 (la cabeza de A con `seq` 0, la de B con `seq` 1); con A vivo, las hojas 2 y 3. Dos
  mensajes con la misma hoja comprometen la clave (`SECURITY.md` §2), y el nodo no lo ve: el
  cerrojo del guardián es por fichero, y un contador nuevo con un diario nuevo no tiene con qué
  compararse. Con el MISMO `--diario`, `--contador-recepcion` e `--indice-firma` -la receta del
  §783-, con A vivo B no arranca (el guardián del índice, §709), y con A parado B no reusa hojas
  (firma en la 4), pero el diario mezcla los dos libros: su `seq` va 0, 1 y vuelve a 0. Es «la
  misma semilla con otro contador», que `SECURITY.md` §2 y `doc/CONFIANZA_RESIDUAL.md` ya
  declaraban como lo que el cerrojo no cubre, que la negativa del guardián ya dice («su propio
  contador y su propia semilla») y que la 92 nombra en su nota del §709; lo nuevo es la medida, y
  que una negativa del nodo llegara a recetarlo. Lo que hace el §813: la negativa del libro tomado
  dice que otro `--ledger` es OTRO nodo, con OTRA clave de firma y su propio índice, su contador y
  su diario; seguida así, con A vivo, ninguna hoja firma dos mensajes. Fuera del disco, solo
  podría verlo un testigo que haya visto las dos cadenas de la clave, y eso no se ha medido. **Lo
  que pide**: decidir si el nodo ata la clave a un solo contador y a un solo libro -por ejemplo,
  el libro guarda la huella de la clave con la que firma, y el contador la de su clave- o si queda
  declarado, con esta medida, como hasta ahora. **Qué la cierra**: esa decisión, con su test y su
  banco si se construye, o la frase de `SECURITY.md` §2 con la medida.

- [ ] **125. El instrumento del vecino no corre: falla igual en depuración y en release (§814).**
  Medido el 15-10-2026: `client::tests_privacidad::a_neighbour_leaf_does_not_reveal_its_balance`,
  ignorado en todo perfil y corrido solo, a mano, con `--ignored`, panica antes de barrer nada, en
  depuración (3,96 s, `vecino_debug.log`, que no dice el commit) y en release (0,49 s, sobre
  `4d8017d`, `vecino_release.log`), con el mismo mensaje: `client.rs:676`, «deben ser vecinos de
  arbol», `left: 3899196792`, `right: 625182909`. `client.rs` y `accounts.rs` son los mismos en
  `b6994c3`, `0315b14` y `4d8017d`. Es el instrumento de la 50, cerrada: el que midió en el §93 el
  diccionario de 10,84 s sobre la hoja del vecino SIN sal, y que desde el flip (§156) su docstring
  da por contrato -si el barrido acierta, el cegado se rompió-. Su premisa es que la víctima y el
  atacante, abiertos uno tras otro, caen en los índices 0 y 1 (`victima ^ 1 == atacante`): la de
  `next_index += 1`. Desde F3 (§157) el índice sale de la identidad (`public_id[0] mod capacidad`,
  con sondeo lineal, en `accounts.rs`), y dos altas seguidas ya no caen ahí. Eso es lectura del
  código: desde cuándo falla no está medido, no se ha bisecado. ⚠️ **Y contradice lo publicado**:
  el §157 dice que este instrumento «ya afirmaba `hallado.is_none()` — era un contrato durmiente
  que el mundo salted puso en verde SOLO». Hoy no llega a ese aserto: panica antes, en el de los
  índices, que está desde `d907efe` (01-08-2026). F3 cambia la colocación en `e3116c6`
  (04-08-2026), que solo toca `accounts.rs`: es el candidato natural para bisecar, y no se ha
  bisecado. Ese «verde» del §157 queda sin respaldo medido. El canon no lo ve: corre la capa sin
  `--ignored`, y los ignorados de su fila no corren; es una ceguera de la clase de la 94, que
  ninguno de sus apartados nombra (⚠️ §864: desde el §864 la nombran la nota de la 94 y el pie «Ceguera declarada» de `tools/canon.sh`, que declaran los `#[ignore]` sin condición y remiten este a la 125, sin decidir nada de él). **Lo que pide**: decidir si se reescribe para un atacante que sí
  sea vecino de alguien en el árbol de hoy, o si se retira con marca; y decir qué deja eso a la 50,
  que se cerró en el §266 con sus testigos y cita el ataque que este instrumento midió, el de los
  10,84 s, como el que con sal no acierta. Para esa decisión pesa un dato: el testigo nativo que la
  50 cita para eso, `T2b-nativo` (`t2b_diccionario_sin_salt_no_acierta`, en `circuit_settlement.rs`
  de `stark-experiment`), se borró con el museo en el §176 (`1f41a1d`). Queda el del circuito,
  `t2b_circuito_diccionario_sin_salt_no_verifica` (`crates/stark-experiment/src/circuit_send.rs`,
  cuyo docstring aún dice «Hermano de `t2b_diccionario_sin_salt_no_acierta`»), y en
  `crates/zk-ssl/src` no hay otro test que barra por diccionario un hermano de camino: retirar este
  deja a la capa sin el suyo. **Qué la cierra**: el instrumento corriendo, con su cifra y su fecha,
  o retirado con marca, y en los dos casos una nota en la 50 que lo diga.

- [ ] **126. El operador ve los saldos: el operador ciego (B11.1 y B11.3) sobre la transición local
  de hoja (C3/B3, con B2) (§860).** Viene de la 47 al cerrarse (§860), que lo tenía desde que la 48 se
  cerró (§268). Es uno de los dos caminos que `SECURITY.md` §2.bis da para el segundo residuo
  —«arquitectura de operador ciego (B11) o federación»—, y sirve a la misión a cualquier escala (la
  cabecera de `doc/ESCALADO.md`; §122.2). El diseño está en `doc/ESCALADO.md` C3 y en
  `doc/CONFIANZA_RESIDUAL.md` §3. Le falta todo lo que se mide antes de un RFC. (a) **La
  conservación con orden parcial**, sin medir (`doc/DIAGNOSTICO_ESCALADO.md` §5.4). (b) **Quién
  demuestra la transición de raíz**: hoy el circuito, y con C3 la compondría el servidor
  (`doc/DIAGNOSTICO_ESCALADO.md` §5.3). (c) DEDUCIDO, sin medir: el agregado —suministro = saldos +
  en vuelo— que la capa comprueba al reabrir (`README.md` §«Leer antes de nada»), sobre saldos en
  claro, no sobrevive tal cual a hojas *hiding*, y eso toca la 115. (d) Cada fila de la tabla de
  garantías que cambie necesita su test discriminante (`doc/ESCALADO.md` §8, y la condición (4) de
  su cabecera). (e) **B2 entra solo como pieza de B3**, y la primera pregunta del RFC es por qué el
  RFC-0002 descartó el anti-replay por `nonce`
  (`spec/rfc/0002-lotes-y-transicion-de-hoja.md` §«Lo que se DESCARTÓ al medir»: quitar la atadura
  de raíz reintroduce el marcador de doble gasto del §32 y el §36) y lo que dice
  `doc/DIAGNOSTICO_ESCALADO.md` §5.2. (f) Los puntos 3 y 4 de `doc/CONFIANZA_RESIDUAL.md` §10: B11
  no mide lo que se aprende de los metadatos, y el coste del segmento de rango de B11.1 lo absorbía
  el presupuesto de latencia de `doc/ESCALADO.md` §7, que la 47 no adoptó: ya no hay presupuesto que
  lo absorba. Cambia el cable y las entradas públicas del envío y del cobro, así que va por RFC
  (`spec/rfc/PROCESO.md`); su corte de versión se coordina con el tren del RFC-0018, y los circuitos
  son de la zona de la migración. B11.2, cifrar el aviso, sigue en la 11. **Qué la cierra**: un RFC
  del operador ciego que conteste de la (a) a la (f) con lo medido, y la decisión del autor sobre
  él.

- [ ] **127. El nodo abre su libro en claro (`SECURITY.md` §3.3) (§860).** Lo nombraba la 20, aunque
  no era un caso de rotación, y `SECURITY.md` §3.3 remitía a la 49, cerrada. El nodo abre con
  `SovereignLayer::open`, que llama a `open_encrypted` sin clave. El autor dejó cablearlo fuera de
  la E4 del RFC-0001 (D-4, §812). **Qué la cierra**: que el nodo abra cifrado, con una frase de
  origen declarado y un test que no lea el libro sin ella; o una decisión del autor que lo deje como
  residuo de despliegue.

- [ ] **129. El recibo de un rechazo depende de lo que nadie conserva, y la declarada no la ata nada
  (§815).** Dos restos del sobre de completitud que el §815 encontró al arreglar el recibo de recepción
  (`SECURITY.md` §3.15) y que no cierra. **(a) El rechazo que nadie conserva** (deducido, sin
  medir). El nodo anota con `fsync` la hoja de un rechazo y no guarda su `data`: la causa, sus
  campos y el `seq` del juicio viajan solo en la respuesta. Si el nodo muere entre el `fsync` de la
  hoja y la respuesta, si la respuesta se pierde o si el titular la calla, el operador honrado no
  tiene el sobre de rechazo que exhibir, y una causa con prueba —`StaleState`, la común cuando dos
  titulares compiten— no se declara: pasada la ventana, ROJO contra él. Es anterior al §815. El orden
  del §815 lo hace barato de cerrar —el veredicto ya se conoce al anotar, y la entrada local del
  registro, que no viaja, podría llevarlo sin otro `fsync`—, pero exhibirlo choca con `exige_misma`
  (pide una cabeza firmada con el `seq` exacto del juicio, que con tráfico puede no existir) y con
  el disco: el registro no se poda (§580). **(b) La declarada no la ata nada** (medido en privado
  sobre `4d8017d`, con el kit de entonces). El kit da 3 a una declarada con cualquier causa de su
  lista, venga o no a cuento —`NotTheAccountHolder` sobre un envío—, y hasta sin cabeza vigente: lo
  único que la ata al recibo es el `hashPrueba`, que el operador conoce, así que cualquier ROJO baja
  a 3 con la palabra del operador. Es lo que la D-G del RFC-0010 llama «declarada, no probada», pero
  nadie lo había dicho con todas sus letras; y por eso la 0.4.3, que añade `ProofFailed` y
  `VerificationFailed` a la lista, no abre una puerta nueva. Lo cierra en parte el tren `zkssl/0.5`:
  con el `digest_pi` bajo la firma (RFC-0019 E3), las dos de la prueba se podrán probar o refutar, y
  saldrán de la lista; para las otras cuatro no hay nada planeado. Dos ataduras que ya existen y no
  esperan al 0.5: `ProofFailed` no la produce hoy ningún brazo con recibo —la capa sólo la da al
  probar; leído en el código, sin medir—, así que acotar por vía la sacaría de todas; y en el lote
  el kit no cruza una causa declarada que no es de forma con `juzgar_forma`: `lote-declarada`
  (publicado desde el §614) declara `NotTheIssuer` sobre una composición que repite la cuenta, que
  la refuta sola, y un kit que ate la forma tendrá que mudar ese vector con su aviso (RFC-0021, R6
  b). El vector del lote del §815 se derivó de una composición sin defecto de forma, para no repetirlo
  (medido en una revisión del §815: con esa composición y la causa `DuplicateAccountInBatch`, la 0.4.3
  lo da por resuelto por su forma). **Lo que pide**: (a) decidir si el nodo conserva el material de
  sus rechazos, cuánto y cuánto tiempo; (b) decidir si la declarada se acota —por ejemplo, a las
  causas que caben en cada vía— o si queda como la palabra del operador, dicho así en la D-G del
  RFC-0010 y en el veredicto 4 de `spec/PAQUETE.md` 2.11. **Qué la cierra**: cada decisión, con su
  test si se construye, o su frase en `SECURITY.md` §3.15.

## F. Publicacion, cuando el circuito este cerrado

La 28 se hace **al final**, por decision explicita: no se tocan los
preprints ni Zenodo hasta que los frentes de solidez del circuito esten
cerrados, para no publicar dos veces. Acumula ya: titularidad del cobro
(§39), claves de custodio (§41) y las cifras.

- [x] **34. Redactar el limite de grados en depuracion.** ~~La decision de
  la 6/24/25 esta tomada (§46).~~ **Hecho** el 30-07-2026: declarado en
  AUDITORIA §20 y en el README como limite conocido, con corregida de paso
  la cifra 172->174 y el comentario contradictorio de `allocate_pending`.
  Con esto el **frente de grados (6, 24, 25, 34) queda cerrado**: decidido y
  documentado, sin migrar nada.

- [ ] **28. ⏸️ SUSPENDIDA hasta el fin del proyecto. Preprints.**
  ⏸️ **Decision del 01-08-2026**: las revisiones de preprints consumen tiempo
  desproporcionado y su enredo de versiones en Zenodo no aporta al codigo.
  **Se retoman al final**, con los tres papers rehechos de una vez y **sin
  referencias cruzadas** —cada uno independiente—.
  ⚠️ **Lo publicado sigue publicado**: los tres tienen DOI y un lector los
  recibe hoy tal como estan. Suspender es aplazar el trabajo, **no retirar el
  problema**, y por eso la entrada no se cierra.
  ✅ **El trabajo hecho no se pierde**: `doc/preprints/` ya lleva los cuatro
  frentes corregidos (§100, §101), listo para cuando toque.
  ⚠️⚠️ **§84 inventario los ficheros EQUIVOCADOS** (§101.1): `PAPER.md` y
  `QUESTIONS.md` de la raiz **no son los preprints publicados** —978 lineas
  contra 448, numeracion y contenido distintos—. Las notas del 01-08 estan en
  **material interno**; la 28 no estaba hecha. **§99.5 por segunda vez.**
  ✅ **Hecho sobre `doc/preprints/`**: los cuatro frentes en
  `ZK-SSL-preprint.md`; **dos filas de §4.1** de `ZK-SSL-residual-trust.md`
  cuya garantia no se cumplia —§73 y §74—; la unidad **MiB**, tercera
  correccion de esa cifra; **§4.7 nueva**, con la privacidad frente a
  terceros y **sin checkmark**, porque §99 descarto las soluciones; y el
  frente A en `ZK-SSL-policy-note.md`.
  ⚠️⚠️ **NO DEPOSITADO.** Los DOI apuntan a las terceras revisiones, **que
  son las que un lector recibe hoy**. Depositar es el paso que cierra esta
  entrada, y es una decision, no trabajo.
  ✅ Escritos el 01-08-2026 (§100) en `PAPER.md`, `PAPER_EN.md` y
  `QUESTIONS.md`: **notas de correccion** para la clave de gasto (§73) y el
  cobro (§27) —«la capa no verificaba las pruebas… la clave no hacia falta
  para mover fondos ajenos»; «cualquiera con el aviso podia reclamarlo»—, la
  **correccion incompleta de §8.2** y la **unidad MiB** en seis sitios.
  ⚠️ **El frente C era peor de lo inventariado** (§100.2): no faltaba un
  analisis, **habia una afirmacion publicada que se queda corta** —§8.2 dice
  que la correccion «consiste en emplear digests de cuatro elementos», y eso
  arregla la identidad, **no la clave**—.
  ⚠️ **Y `QUESTIONS.md` se quedo fuera del primer parche** (§100.3): el
  inventario listaba seis sitios y se cubrieron cuatro.
  ⚠️⚠️ **NO SUBIR A ZENODO todavia**: la **entrada 16** sigue abierta —los
  tres se citan entre si por versiones antiguas— y publicar con referencias
  cruzadas rotas **crearia el mismo problema otra vez**. Los cuatro frentes
  estan escritos; **la revision no esta publicada**.
  ~~28. Corregir los tres preprints: INVENTARIADO, decision tomada.~~
  ⚠️ **Inventario completo en §84**: cuatro frentes, **once pasajes** en tres
  documentos, con fichero y linea. ⚠️ **DECIDIDO el 31-07-2026 (§84.3): la
  cuarta revision ANOTA** que las dos propiedades —la clave de gasto y el
  cobro— **no estaban impuestas** en v1, v2 y v3, y desde cuando lo estan.
  Precedente: `PRINCIPIOS.md` §8 ya corrigio la hoja de ruta porque callarla
  «seria faltar al principio de transparencia». ⚠️ **Y el frente C no es una
  errata sino una conclusion** (§84.4): si el criterio de los ~128 bits vale
  para la solidez, vale para el espacio de claves, y entonces el paper debe
  decir que el sistema **no alcanza ese liston** mientras `sk` sea un
  elemento (entrada 15). **No tocar Zenodo hasta las cuatro**: un documento
  publicado no se rectifica con `git revert`.
  ~~Original:~~ **28. Corregir los tres preprints: ya no es solo la 27.** Describen el
  cobro como demostracion de titularidad —§27, corregida y verificada—. ⚠️ **Y
  desde el 31-07-2026 se le han sumado tres cosas mas**, todas medidas:
  (a) §73.2 — los tres citan como argumento institucional central que la
  clave de gasto no sale de la maquina del cliente, y **la capa no lo imponia
  hasta ese dia** porque no verificaba las pruebas; (b) §82.4 — el paper
  llama al techo de 63 bits de solidez «insuficiente y no comparable con los
  ~128 bits», y **no aplica ese criterio al espacio de claves**, que es de 64
  bits (entrada 15); (c) §83.3 — la cifra de acumulacion esta en MiB
  etiquetado «MB», un 7,2 % por debajo si se lee en SI (entrada 22).
  ⚠️ **No tocar Zenodo hasta tener las cuatro resueltas**, no solo la 27.
  ⚠️ **§371 levanto esta suspension SOLO para los DOI** (27-08-2026): los
  tres ficheros de `doc/preprints/` llevan ya los identificadores vigentes. Se
  tocaron **solo identificadores** -ni una cifra, ni un parrafo- y desde ese
  commit **ya no son byte a byte los PDF depositados**. Rehacer los papers
  sigue suspendido: esto no lo adelanta.
  ⚠️ **§860: gana el frente de la 96**, que se cierra fundida aquí, por la vía REVERSIBLE que su
  cuerpo ofrecía: «la mitad cara es del receptor, porque el cobro recorre dos árboles» no la
  sostiene lo medido, ni el sentido ni la causa, y se declara en `doc/preprints/ERRATA.md`, Entrada
  5. En la próxima revisión la frase o se retira o se vuelve a medir sobre el árbol de esa revisión,
  y no se copia de la Entrada 5. Y los bytes del mismo párrafo —«about 62 KB» por prueba y «120.4
  MiB» por mil pagos— también se movieron (la banda D-AC del RFC-0009, en
  `crates/zk-ssl/src/metrics.rs`): la Entrada 5 lo dice en una línea, y no son su objeto.

- [x] **107. El kit publicado no comprueba lo que el nodo firma hoy.**
  `arqueo-verify-v0.2.0` (§442, §443; commit `1528943`) recompone cabezas hasta
  la v4 y el nodo firma la v5 desde el §452 y la v6 desde el §570. **Medido el
  2026-09-30** con el binario compilado desde su tag contra los vectores de
  71c5aad: sus paquetes (cabezas v3 y v4) salen VERDE; `edad`, `prenda`,
  `cobro_pendiente` y `completitud` (cabezas v5 y v6) salen ROJO con salida 1,
  «tipo desconocido»; y el `ancla` del §593 (cabeza v6), medida en el §596 contra
  `main` en ab79a50: sus tres vectores, salida 1, «tipo desconocido: ancla».
  **Falla cerrada, nunca un VERDE falso.** Declarado ya en
  `doc/KIT.md` y `doc/KIT_EN.md` (seccion 0) y en los dos README. ⚠️ §725: sí lo da en el punto 2 de la 0.4.4, como la 0.1.0: con el mismo `mmrSize` y distinto `epochDigest`, la extensión da VERDE desde la 0.1.0 y el consumo desde la 0.2.0, y la razón (c) de abajo, «no hay aceptacion falsa que cerrar», tampoco se sostiene (`SECURITY.md` §3.18; `doc/KIT.md`, sección 0, y los dos README, con su marca).
  **Decision (2026-09-30, sesion de ECST): A LA COLA, no inmediata**, por tres
  razones del proyecto: (a) publicar es acto manual del autor, con huellas
  medidas desde fuera y dos sellos (D-K5 del §442), no algo que un asistente
  corte; (b) el nucleo no esta congelado (RFC-0005 sigue propuesto) y la v6 y el
  sobre de completitud son de esta misma semana: cortar hoy es arriesgarse a
  otra release rancia en dias; (c) no hay aceptacion falsa que cerrar: lo que se
  pierde es cobertura, no solidez, y eso se DECLARA, que ya esta hecho.
  **Disparador**: cortar la 0.3.0 —cabezas v5 y v6, los sobres posteriores a la
  0.2.0 con sus catalogos, `tools/artefacto.sh` y el asiento del registro medido
  desde fuera— cuando el autor de por estable la v6, y **en todo caso ANTES** de
  cualquier deposito, anuncio o correo que invite a un tercero a verificar un
  paquete de un nodo real.
  **Disparador CUMPLIDO (2026-10-01)**: el autor dio la v6 por estable, con el
  RFC-0014 ya aceptado (§618). La 0.3.0 se CORTA en el §619 -el crate, los dos
  manuales con su paso 5 y los sitios que publican la version- y se publica
  sobre ese commit en el orden de la seccion 11 de `PAQUETE.md`; la entrada se
  cierra con el asiento que REGISTRE la release medida desde fuera.
  **El commit del corte pasa al §620**: preparando el registro, cuatro
  manifiestos que viajan en el tarball decian que los vectores del cable 0.3 los
  verifica `arqueo-verify-v0.2.0`, y medido es falso (0 de 38; los verifica el
  arbol en `0eda58c`, 38 de 38). Se corrige antes de publicar, y el tag va sobre
  el commit del §620.
  ✅ **CERRADA en el §622**: `arqueo-verify-v0.3.0` publicada por el autor sobre
  `65cabfb` (tag ligero, tarball producido sobre `target/artefacto/` vacio,
  `gh release create --verify-tag`) y medida desde fuera en DOS maquinas, que
  dicen lo mismo byte a byte: tarball `06648502e0171fea` (15.485.894 B, 334
  ficheros), binario `38fe5b9a6d04144b`, y los cinco pasos del manual y los diez
  catalogos desde el kit DESCARGADO. La 0.2.0 sigue publicada, ya no vigente,
  con su limite declarado en la seccion 0 de los dos manuales.

## G. Otro proyecto, no una incidencia

- [x] **38. Prueba de vacuidad en TODOS los circuitos: hecha.**
  ~~Nueve circuitos sin ella.~~ **Cerrada** el 30-07-2026 (§63): añadida a
  los ocho que quedaban y **23 de 23 en verde** — ningun circuito del
  proyecto tiene una restriccion que no imponga nada. Solo queda fuera el
  `WorkAir` de `lib.rs`, que es el circuito de demostracion de winterfell y
  no protege nada. De paso aparecieron dos nombres inconsistentes:
  `range_check` llama `TRACE_ROWS` a lo que los demas llaman `TRACE_LENGTH`,
  y `nullifier`/`rescue_hash` no tienen `TRACE_WIDTH` porque su traza ES el
  estado del hash.

  ~~Original:~~ **38. Prueba de vacuidad en los nueve circuitos que no la tienen.**
  `crate::mutation::buscar_vacias` detecta restricciones que no imponen nada,
  y la usan **15 de los 24** circuitos con `impl Air` (§62). Sin ella en
  `compliance_circuit`, `double_entry`, `dual_climb`, `lib`, `merkle`,
  `nullifier`, `range_check`, `rescue_hash` y `solvency`. ⚠️ Mismo patron que
  §59.2: herramienta util aplicada a parte del codigo sin que conste a que
  parte. Es un test por circuito, no un rediseño.

- [x] **39. ✅ La cadena de columnas PERIODICAS: ya la comprueba el barrido.**
  ~~No la comprueba nada.~~ **Cerrada** el 31-07-2026 (§81):
  `check_constraint_layout.py` cruza ahora tambien los indices de
  `periodic[...]`, con `DESBORDE PERIODICA` y `MUERTA PERIODICA`, y con
  autotest propio sobre el caso real de §66.2. Reproduce sola el recuento
  que hubo que hacer a mano al amputar `mint_pending` —32 y 41— y donde §68
  conto ocho periodicas cuando eran nueve. ⚠️ **Costo TRES errores**, todos
  registrados en §81.3: la regex casaba `periodic[` y no `periodic_values[`
  -159 columnas muertas falsas, **el mismo agujero de §59.2 cometido al
  cerrarlo**-; el contador devolvia 0 en vez de «no comprobado» -seis
  desbordes falsos en `solvency`-; y un `\b` se colapso en un caracter de
  retroceso -823 desbordes falsos-. Los tres compilaban y los tres producian
  un informe con aspecto de informe.

  ~~Original:~~ **39. La cadena de columnas PERIODICAS no la comprueba
  nada.**
  `check_constraint_layout.py` verifica los indices de `result[...]`, pero
  **no los de `periodic[...]`**: son dos arrays distintos y la herramienta
  solo mira uno. Al extraer `circuit_mint_climb` se dejaron tres constantes
  `P_*` muertas en la cadena, `P_SEG_LINK` quedo desplazado y el indice se
  salio del array (§66.2). ⚠️ **Ahi se noto porque desbordo**; si el
  desplazamiento fuera hacia ABAJO, la restriccion leeria la columna
  periodica equivocada **en silencio**. Es el mismo patron que §59.2 y
  §62.2, por tercera vez: herramienta util que cubre parte del problema sin
  que conste que parte.

- [x] **40. ✅ FALLO DE SOLIDEZ CORREGIDO Y VERIFICADO: el compromiso del
  pendiente no estaba atado al importe declarado.** **Corregida** el
  31-07-2026 (§75): `C_PEND_IN` y `C_PEND_VAL` pasan al **carril B** -el que
  se inserta- y `C_PEND_VAL` de **5 a 12** ranuras, fijando tambien la
  capacidad y el **relleno** de su fila. 89→96 y 125→132, en los dos
  circuitos. Los tres testigos de rojo a verde con el positivo en pie, **272
  y 201 tests sin ninguno ignorado**. ⚠️ El relleno era la mitad del fallo:
  mover de carril a secas lo habria dejado vivo con otra forma (§75.1).
  ⚠️ `build_trace` **no cambio** —la traza honesta ya cumplia lo que ahora se
  exige—, que es justo por lo que ningun test lo veia (§75.3). Queda
  declarado como computo muerto el compromiso que el carril A sigue
  calculando sin que nadie lo lea (§75.4).

  ~~Original:~~ **40. ⚠️ FALLO DE SOLIDEZ CONFIRMADO: el compromiso del
  pendiente no esta atado al importe declarado.** **Medido en release el 31-07-2026
  (§72), en los DOS circuitos**: `circuit_mint_pending_climb` y
  `circuit_mint_pending` -el de produccion- aceptan una traza que declara
  emitir 250.000, sube el suministro en 250.000 y deposita un pendiente de
  1.000.000. ⚠️ **Diagnostico afinado**: no falta una restriccion, **esta en
  el carril equivocado** -`C_PEND_IN`/`C_PEND_VAL` construyen el compromiso
  en el carril A, cuyo digest `C_PEND_ENTRY_A` descarta forzando la hoja a
  cero; lo que se inserta es el del carril B (§72.2)-. **Alcance**: la via
  delegada esta protegida porque la capa recomputa `pending_commitment`; la
  antigua `apply_mint_to_pending` **NO lo estaria, por LECTURA y sin medir**
  (§72.3). Exige dos claves de custodio: no hay escalada de privilegio, pero
  crea dinero **fuera del suministro y por tanto fuera del tope**, que es lo
  que §66 y §67 demostraron. Dos testigos ROJOS con `#[ignore]`, patron de
  §50. ~~**Siguiente paso: test de capa** que decida si la via antigua lo
  acepta de punta a punta.~~ **HECHO el 31-07-2026: la aceptaba** -suministro
  +250.000, pendiente de 1.000.000 depositado y cobrable-. Al escribir ese
  test aparecio la **43**, que era la causa de fondo. ⚠️ **Y sigue abierta
  tras corregir la 43**: se añadio a `apply_mint_to_pending` la exigencia de
  que el aviso declare el importe de la prueba, pero eso es **mitigacion
  parcial** (§73.4). Camino residual **sin medir**: depositar un compromiso
  de un millon declarando doscientos cincuenta mil -aviso y prueba coherentes
  entre si- y cobrarlo con el aviso verdadero, porque `apply_claim` no
  contrasta el importe contra `pending_amounts`. ~~**Medir eso es lo
  siguiente**, y decide si la 40 es un hueco de auditabilidad externa o
  conservacion rota en produccion.~~ **MEDIDO el 31-07-2026 (§74): es
  CONSERVACION ROTA.** Bob acaba con 1.000.000 y el suministro emitido es
  250.000. ⚠️ **El deposito devolvio `Ok(())` con la prueba ya verificada**,
  y el cobro **no falsifica nada** —el compromiso del millon esta de verdad
  en el arbol—: no hay ningun paso que un verificador pueda detectar.
  ⚠️ **El arreglo minimo NO basta** (§74.2): mover `C_PEND_IN`/`C_PEND_VAL`
  al carril B deja libres siete elementos -capacidad y relleno- que
  `C_PEND_VAL` no fija. Dimensionado en §74.3: `C_PEND_VAL` de 5 a 12
  ranuras, 89→96 en el `_climb` y 125→132 en produccion, en **los dos
  circuitos**. Criterio de exito: los tres testigos rojos en verde.
  **Es hoy la entrada de mas peso de esta lista**, por delante del README y
  de retirar las vias antiguas.

  ~~Original:~~ **40. ⚠️ El carril B del compromiso pendiente no esta atado.**
  `C_PEND_IN` y `C_PEND_VAL` restringen **solo el carril A** -no hay ningun
  `LANE_B` en ellas-, asi que lo que `C_PEND_ENTRY_B` mete en el arbol es lo
  que el carril B haya calculado, sin que el circuito lo ate a `COL_R_ID`,
  `COL_SALT` ni `COL_AMOUNT`. Hoy lo sujeta la capa al recomputar
  `pending_commitment` y comparar raices; **un auditor externo que solo vea
  la prueba no puede** (§70.4). Afecta a `circuit_mint_pending` y a su
  `_climb`. ⚠️ **Hipotesis de lectura, NO medida**: es de la clase de §39 y
  §27 -restriccion ausente, no colisionada- que
  `check_constraint_layout.py` no detecta por construccion. Lo decide un
  test discriminante que mute la entrada del carril B en la fila 40. **No
  degradar a «cosmetico» antes de que un test lo mire**: eso se hizo tres
  veces con la 36.

- [x] **41. ✅ (§814) ~~Noventa y tres fallos de depuracion: TRES clases, dos vacias.~~
  Ninguno: el 15-10-2026, en depuración, la capa da 443 pasan, 0 fallan y 45 ignorados.**
  **Remedido el 26-08-2026**, tras los §364 y §365: son **noventa y tres**,
  y **todos** de la clase de grados; las otras dos clases quedan **vacias**.
  ⚠️ **La tabla del §78.1 clasificaba por FICHERO**, y bajo
  `trace/mod.rs` caben DOS mensajes que no son lo mismo: uno de asercion de
  frontera y otro de restriccion de TRANSICION que la traza no cumple. Los tres
  de esa tercera clase eran `the_delegated_cap_is_enforced_at_apply`,
  `burning_frees_up_minting_capacity` y `the_total_supply_survives_restart`:
  los tres panicaban GENERANDO la subida de un enunciado falso a proposito, en
  su mitad de rechazo, y el §364 los devolvio a depuracion con el remedio del
  §77.1 -esperar lo que corresponde a cada modo, que gana cobertura- en vez
  de marcarlos. ⚠️ **El precio declarado abajo vencio**: de ochenta a
  noventa y seis se escondieron dieciseis fallos nuevos, y tres de ellos eran
  de una clase que este mismo cuerpo daba por cerrada.

  ~~Ochenta fallos: CLASIFICADOS, dos corregidos.~~
  ~~Doce fallos sin diagnosticar.~~ ⚠️ **Eran ochenta, no doce** -los doce
  eran los de `tests_delegada`; el conteo original miro un subconjunto-.
  **Clasificados** el 31-07-2026 por clase de panico (§78): **78** son
  `degrees didn't match` -grado declarado que no se realiza, entradas 6, 24
  y 25, decidido en §46- y **2** son `trace does not satisfy assertion
  main_trace(16, 39)`, que **no es la misma clase**: una asercion que la
  traza no cumple. ✅ **Esos 2 corregidos**: son escenarios de rechazo
  -impostor con claves de custodio por caminos de gobernanza- y en
  depuracion winterfell los caza al generar; marcados con el motivo medido.
  ⚠️ Uno de ellos, `the_governance_set_survives_restart`, **estaba escondido
  detras de su nombre**: suena a camino legitimo y la propiedad esta en el
  negativo que lleva dentro (§78.2). ⚠️ **Los 78 NO se marcan en bloque**
  (§78.4): su causa varia y la 44 enseño que a veces es mejor cambiar el
  testigo que marcarlo, lo que hay que decidir test a test. **Precio
  declarado**: mientras sigan rojos, un fallo nuevo en depuracion se esconde
  entre ellos.

  ~~Original:~~ **41. Doce fallos de depuracion sin diagnosticar.**
  `mint`, `freeze` y `recovery` acumulan doce `tests_delegada` que fallan en
  depuracion con *"transition constraint degrees didn't match"*.
  **Preexistentes**, comprobado con `git stash`. Divergen en los indices
  **44 y 73-88**; los cinco de `mint_pending`, en **50-70** por la posicion
  0 (§71.3). **Otra disposicion, causa distinta y no medida**, asi que no se
  les puso la marca de aquellos: seria atribuirles una causa sin comprobar
  (§71.4). El metodo que funciono: diffear el vector, aislar la variable,
  test discriminante. Probable emparentada con la 25 -subidas a cuentas
  (indices 0 y 1) y a congelados (arbol vacio)-, **sin verificar**.

  ⚠️ **§814: CERRADA EN EL §814, y no por arreglo: el censo de la sesión la encontró hecha y sin
  registrar.** Medido el 15-10-2026 en 4 CPU, sobre `0315b14` (el §813; de ahí a `4d8017d`
  `crates/zk-ssl` solo cambia la limpieza de un test de `two_phase.rs`, el §832, sin añadir ni
  quitar tests): `cargo test --locked -p zk-ssl`, en depuración, da **443 pasan, 0 fallan y 45
  ignorados**, en 3.253,94 s, y los doc-tests, ninguno (`todo.log`); con otro trabajo en la
  máquina, la carga fue de 9,2 a 18,6 durante la suite (`carga.txt`). El censo de la misma
  sesión, sobre `b6994c3` y con `--lib`, ya había dado lo mismo, en 2.613,48 s (`debug41.log`).
  De los 45, **7 son instrumentos de medida**, ignorados en todo perfil, y **38 llevan
  `cfg_attr(debug_assertions, ignore = …)`**: 32 con «winterfell valida grados en depuracion:
  juez release» (`instrumento_edad.rs` 4, `instrumento_pago.rs` 2, `instrumento_revela.rs` 21,
  `nivel_de_produccion.rs` 1 y `prueba_edad.rs` 4), los 5 de `two_phase.rs` con «grado
  dependiente del testigo» y `the_governance_set_survives_restart`, el escenario de rechazo del
  §78. **Corridos en depuración con `--ignored`, pasan los 38** (`marcados.log`). En release, la
  fila de `zk-ssl` de `tools/canon.sh` fija 481 pasan y 7 ignorados: son los 443 y los 38.
  **Ninguna de las 38 marcas tapa uno de los 93**: las 32 entraron con sus tests, nuevos, del
  11-09 al 07-10-2026 (§463 a §697), y las 6 restantes ya estaban el 31-07 (§71.3 y §78), antes
  de contarse los 93 (`git log -S` con el motivo de cada marca): lo que queda de los 93 pasa en
  depuración sin marca. Los registros son de la sesión del §814, y su asiento los nombra.
  **Por qué dejaron de fallar los de grados NO está medido**, ni desde cuándo: no se ha bisecado.
  La hipótesis del censo, y es solo eso: la ocultación del fork de winterfell (§538, RFC-0009;
  `crates/winter-prover/README.md`) aleatoriza la traza, y el grado declarado se realiza.
  **Lo que NO se hace**: las 38 marcas se quedan, aunque escondan en depuración tests que hoy
  pasan ahí. Que sigan pasando puede depender del fork de winterfell -es la hipótesis de
  arriba-, que migra otra sesión, y quitarlas es de esa sesión, después de la migración. El
  precio declarado arriba -un fallo nuevo en depuración se esconde entre los rojos- deja de
  cobrarse, porque la orden de la capa ya no da ninguno, salvo en los 38 marcados, que en
  depuración no corren y los mira release. De los 7 instrumentos, corridos en depuración con
  `--ignored`, 5 dan `ok`; `instrumento_edad::instrumento_de_la_prueba_de_edad` no terminó: lo
  paró, tras unos 63 min, el límite de dos horas del guion de la medida (`marcados.log`); y
  `a_neighbour_leaf_does_not_reveal_its_balance` falla, y no por la depuración: falla igual en
  release (la 125). La (a) de la 94, que remitía aquí la cifra, gana su marca del §814.

- [x] **42. ✅ (§779) ~~`mint_to_pending` diverge de `mint` en el error del tope.~~
  Ya no diverge: la via antigua no existe.**
  **Respaldo**: §73.4, donde se decidio no tocar la via antigua al corregir
  la 43 porque cambiar su error cambia lo que ven sus tests.
  La via antigua devuelve `OverRegulatoryLimit` —el error del limite
  regulatorio de una **transferencia**— para una violacion del **tope de
  emision**, donde `mint` siempre devolvio `SupplyCapExceeded`. Y suma sin
  saturar (`self.total_supply + amount`), que en depuracion desborda con
  panico. `apply_mint_pending_delegated` usa `SupplyCapExceeded` y
  `saturating_add`; la antigua **no se toco**, porque cambiar su error
  cambia lo que ven sus tests. Menor, acotado, declarado.

  ✅ **CERRADA EN EL §779, y no por arreglo: se fue su objeto.** `mint_to_pending`
  y `apply_mint_to_pending` se retiraron con el resto de la via-recibo de
  custodios en B-3b-ii (`d19b20e`, 05-08-2026), que asienta el §171. Antes de
  ese commit la primera devolvia `OverRegulatoryLimit` sumando sin saturar
  (`git show d19b20e^:crates/zk-ssl/src/two_phase.rs`, `:682` y `:692-693`).
  Hoy no hay en `crates/` ninguna `fn mint_to_pending(` ni
  `fn apply_mint_to_pending(`: el nombre sobrevive en el ayudante de tests
  `mint_to_pending_delegated` (`tests_support.rs:577`), en nombres de test y
  en comentarios. La unica emision a pendiente, `apply_mint_pending_delegated`,
  devuelve `SupplyCapExceeded` y suma con `saturating_add` y `checked_add`
  (`two_phase.rs:1690-1704`, lineas tras este §779), igual que
  `apply_mint_delegated` (`mint.rs:43-63`): ya no hay dos lados que diverjan.
  ⚠️ Siguio abierta dos meses despues de quedarse sin objeto: es la rancia de
  OTRA clase del §583, la que contar casillas no ve, y la 42 era de «las otras
  cuarenta» que aquel recuento no miro contra el arbol. Los dos comentarios
  que la daban por viva como «la via antigua de arriba» se corrigen en el
  mismo §779 (quedan en `two_phase.rs:1658-1669` y `:1790-1794`): es lo que el
  §375 dejo «fichado y NO tocado», y su referente era este, no `send`.

- [x] **43. ⚠️ FALLO GRAVE CORREGIDO: la capa no verificaba las pruebas de
  la via de pago.** ~~`apply_send`, `apply_claim` y `apply_mint_to_pending`
  aplicaban sin llamar a `verify`.~~ Medido con **cinco testigos** en
  release (§73): un pago con prueba de 32 ceros se aplicaba; un estado de
  titular mentido escribia la mentira; **gastar no requeria la clave del
  titular** —robo consumado: la victima paso de 1.000.000 a 750.000 y el
  dinero quedo en un pendiente dirigido a la atacante—; y **cobrar tampoco
  requeria la del receptor**. ✅ **Corregido** el 31-07-2026: las tres
  verifican antes de tocar el estado, con el patron que ya existia siete
  veces en el repositorio, y `apply_claim` comprueba ademas `frozen_root`.
  Cinco testigos de rojo a verde, **200 tests sin regresion**. ⚠️ **No
  cierra la 40**, cuyo residuo queda declarado en §73.4. ⚠️ **Y va con la
  28**: los tres preprints citan como argumento central una propiedad que
  el circuito demostraba y la capa no imponia (§73.2).

- [x] **45. ✅ Fallo inestable en release: identificado y corregido.**
  ~~Un fallo unico sin explicar en `zk-ssl` release.~~ **Reproducido**
  subiendo la contencion —12 pasadas a 16 hilos, **1 de 12**— y no
  descartandolo con un hilo, que es el error que §29 documenta.
  `an_encrypted_ledger_needs_the_right_passphrase` fallaba al **reabrir con
  la contraseña correcta** por el bloqueo de directorio de `sled`
  (entrada 18). ⚠️ **No era un fallo de seguridad**: la contraseña
  incorrecta nunca abrio nada, y distinguirlo exigio leer el mensaje y no el
  nombre del test (§79.2). ✅ **Corregido**: `open_encrypted_retry` en las
  **nueve** llamadas que abrian cifrado sin red —`open_retry` ya protegia
  otras 39: la proteccion existia y no estaba aplicada a todo, §59.2 por
  quinta vez—. 12 de 12 en verde tras el arreglo. ⚠️ **Doce en verde no
  demuestran que este arreglado** (§79.4), y **la 18 no se cierra**: esto
  protege los tests, no un nodo real.

- [ ] **46. Anomalia: mas columnas y mas restricciones dan una prueba MENOR.**
  Medido el 31-07-2026 (§86) con relleno sobre `circuit_settlement`: pasar de
  **49 columnas y 155 restricciones** a **52 y 170** deja la prueba en
  **39.538 B en vez de 40.645** —2,7 % menos— y la generacion en **97,8 ms en
  vez de 111,8** —12,5 % menos—. Tres ejecuciones de cada lado: el tamaño es
  **determinista**, no ruido. ✅ **Descartado que pruebe menos**: la suite del
  circuito pasa 16 de 16 con el relleno puesto, incluidos los ocho negativos
  y la prueba por mutacion (§86.4). ⚠️ **La causa NO esta identificada** y
  apunta a algo estructural de winterfell —particionado del compromiso,
  columnas de composicion—. **No bloquea nada**: la decision de ensanchar la
  clave (entrada 15) no depende de entenderlo. Pero si el efecto es real y
  general, **es una optimizacion que el proyecto desconoce**, y eso merece
  mirarse.

- [ ] **57. Especificar `circuit_mint` — parcial, con una defensa ya
  comprobada.** Al escribirla (§107) aparecio la pregunta que §80 enuncia y
  nadie habia probado: **¿que impide que un custodio firme dos veces?**
  ✅ **MEDIDO**: el circuito lo rechaza —`one_custodian_cannot_sign_twice`, con
  `InconsistentOodConstraintEvaluations`—. La defensa es la **descomposicion
  binaria**: `IDX_B − IDX_A − 1` vale `p−1` si los indices son iguales y no
  cabe en el segmento. **Esta en una restriccion, no en el constructor.**
  ⚠️ **Tercera «garantia por consecuencia»** del proyecto: la dan `C_ACC`,
  `C_ACC_FINAL` y el segmento **juntas**; ninguna sola.
  ⚠️ **Dos mediciones malas por el camino** (§107.2, §107.3): un test **vacuo
  por construccion** —`prove()` no falla en release— que habria anunciado un
  fallo de solidez inexistente, y una sospecha **acertada por el motivo
  equivocado** —el tiempo, sin referencia—.
  ✅ **La especificacion de `mint` esta COMPLETA**: `doc/air/circuit_mint.md`.
  Añadio **§4.5** —`C_TRANSPORT` tiene 11 ranuras y a primera vista solo se
  contaban 10 columnas; la undecima era `COL_MAX_SUPPLY`, y **sin ella el
  tope seria falsificable**— y **§4.6**: el circuito prueba que **dos indices
  distintos** autorizaron, **no que haya dos personas**.
  ⚠️ **Van DOS de veintisiete**, con **tres garantias por consecuencia**
  documentadas. **Las 25 restantes siguen sujetas a §105.3.**

- [x] **56. Firmar las cabezas con XMSS — decision tomada, sin implementar.**
  ⚠️ **DECISION del 01-08-2026 (§106)**: **todo el camino de produccion es
  post-cuantico**; los backends comparados se conservan **como evidencia de
  por que se eligio STARK**, no como alternativa. ✅ **Verificado**:
  `zk-ssl` y `stark-experiment` **no tienen ninguna dependencia de curva
  eliptica** — una sola familia de supuestos, la resistencia del hash.
  **XMSS para las firmas** (entrada 53), y **no por ser «el mas
  post-cuantico»**: es el unico que **no añade una familia de supuestos
  nueva** —SPHINCS+ tambien es de hash pero firma 8-17 KB; ML-DSA mete
  reticulos, y **un sistema con dos familias es tan fuerte como la mas
  debil**—. ⚠️ **Su estado es aqui una ventaja** (§106.4): reusar indice
  filtra la clave, y con `seq` como indice **reusarlo ES la vista dividida**.
  ⚠️ **Sin resolver**: persistir el indice a traves de un reinicio —un nodo
  que lo pierda puede reusarlo— y elegir el tamaño del arbol, que fija
  **cuantas epocas se pueden firmar antes de agotarse**.


  ✅ **CERRADA (§267, 10-08-2026)**: **está implementada**, y las dos
  incógnitas que esta nota daba por «sin resolver» están resueltas —medido
  contra el árbol, no contra el texto de la nota—.
  · **Persistir el índice a través de un reinicio**:
    `crates/zk-ssl-node/src/firma_indice.rs`, «el guardián del índice de
    firma» — un contador propio con `fsync`, **aislado del ledger** (§111.1),
    que es exactamente lo que esta nota pedía. **9 tests**, pinados en el
    canon.
  · **El tamaño del árbol**: `XmssMtSha2_40_8_256` —MT **40/8**— fijado como
    `pub type Conjunto` en `crates/zk-ssl-verify/src/lib.rs`. Firma de **137
    bytes** = OID(4) + índice(5) + 4×32.
  Y el eslabón que perseguía existe entero: `crates/zk-ssl-node/src/firma_cabeza.rs`
  firma `EpochHead` con XMSS **con el guardián delante** (§234), **5 tests**;
  `xmss` es dependencia con versión exacta en dos crates y hay código en
  nueve ficheros `.rs`.
  ⚠️ **Lo que de XMSS sí queda no desaparece**: KAT, el issue upstream y ARM
  pasan a la **nota 77**, cada uno con su medida.
  ⚠️ **(§705) El §106, enmendado, no revocado.** Desde el §632 la nota del
  medio del ancla se firma con ML-DSA-44, y desde el §633 el kit la
  verifica: el §705 la declara como la excepción a «una sola familia de
  supuestos», con su coste, y deja sin excepción «todo el camino de
  produccion es post-cuantico», así que el puente Ed25519 del RFC-0013 no
  se construye (lo adopta la aceptación en bloque, reversible). Las
  cabezas y su firma XMSS no cambian.
  ⚠️ **(§838) Y otra vez, para el transporte.** La mitad que el §705 dejaba
  sin excepción la tiene ahora el TLS del cli y del SDK, `rustls` sobre
  `ring` por `ureq`, con su coste: la clave de vista en tránsito descansa
  en un ECDHE clásico. No cubre nada de lo que la casa firma o juzga: ni
  cabezas, ni notas, ni cofirmas (la 122).
- [ ] **55. ⚠️ B12.1: el formato de especificacion del AIR, probado en un
  circuito.** `doc/air/circuit_burn.md`, escrito el 01-08-2026 (§105).
  ✅ **El formato funciona**: la seccion «que NO se restringe» es la unica que
  no se puede extraer del codigo, y produjo **tres razonamientos que no
  existian** — la asimetria de los lanes, y **dos garantias por consecuencia
  y no por restriccion**, que son la forma exacta de §72.
  ❌ **No encontro ningun fallo** en `circuit_burn`.
  ⚠️ **Las otras 26 NO se escriben, y no por esfuerzo** (§105.3): una
  especificacion escrita por quien escribio el circuito **hereda sus puntos
  ciegos**, y una completa y firmada por el autor **parece un contrato** que
  un auditor podria auditar **en vez de** auditar el codigo.
  **Forma correcta**: escribirlas **con** la auditoria, no antes — entrada 7.
  ⚠️ **Y la via automatica no sirve**: dos barridos aprobaron el circuito sin
  ver nada, porque §72 fue una restriccion **bien formada sobre el objeto
  equivocado**.

- [ ] **54. ⚠️ Cambiar el verificador es invisible — REGISTRADO, y con
  vehiculo nuevo (§135).** ✅ En `SECURITY.md` §2 y en
  **`doc/preprints/ERRATA.md` entrada 1**.
  ⚠️⚠️ **El parrafo se habia insertado EN el cuerpo del preprint** —un
  artefacto publicado— **dentro de un commit sobre medicion de tiempos**, y
  el asiento que lo narraba **lo daba por revertido cuando no lo estaba**
  (§135.1). **Revertido: 783 bytes, 12 lineas.**
  ✅ **`ERRATA.md` creado**: el vehiculo **aditivo** que faltaba — corregir lo
  publicado **sin reescribirlo**. Con su entrada 2 admitiendo que **las
  cuatro notas de §100 usaron el vehiculo equivocado**.
  ⚠️ **El cierre sigue pendiente**: `hash_verificador_vigente` **requiere que
  el sistema tenga nocion de «reglas vigentes»**.
  `SECURITY.md` (§128.2).** ✅ Aplicado el parrafo en `SECURITY.md` §2: **el
  poder mayor no estaba en ninguna de las dos enumeraciones** del proyecto.
  ⏸️ **El parrafo del preprint queda redactado y SIN aplicar**: los preprints
  estan **suspendidos** (entrada 28) y ya depositados — no llegaria a un
  lector hasta una quinta revision. Texto listo para cuando se levante.
  ⚠️ **El cierre sigue pendiente, y la razon se CORRIGIO en §246**: decia
  que hacia falta «nocion de reglas vigentes», lo cual es **casi circular**
  —el campo ES ese mecanismo—.
  ⚠️ **La razon real: el AIR es CODIGO, no datos.** Lo unico hasheable en
  ejecucion son las `ProofOptions`, y **un operador puede cambiar el AIR
  dejandolas identicas**: el campo seria **CIEGO, no vacio** — y un campo
  ciego pasa desapercibido mintiendo sobre lo que existe para detectar.
  **Depende de la entrada 55** (el AIR como datos) **o de compilacion
  reproducible**. Hashear el fuente al compilar NO vale: no prueba que el
  binario se construyera de ese fuente.
  ⚠️ §823: **la compilación reproducible da una precondición, no el cierre
  (H6).** El nodo y el testigo salen idénticos byte a byte dentro de una misma
  máquina, y entre máquinas no está demostrado (`doc/KIT.md` §«Reproducir el
  kit, para quien no se fíe de la descarga»). Y aunque lo estuviera, una
  huella reproducible dice que un binario publicado sale de su fuente, no que
  el operador corra ese binario: `hash_verificador_vigente` seguiría siendo
  una declaración suya. Sigue abierta.

- [x] **63. ✅ La contencion del anclaje: REGISTRADA (§123).**
  ✅ **Mecanismo comprobado (T5a)**: dos titulares que no comparten nada
  quedan serializados por el anclaje global — `StaleState`. **Cualitativo y
  firme.**
  ⚠️ **El numero va en RANGO**: **1,53 y 1,87 TPS**, 22 % de diferencia entre
  corridas. Publicar una cifra unica repetiria lo de `ESCALADO.md`.
  ⚠️ **El 1,6 anterior acerto por compensacion de errores** (§123.3): las dos
  constantes estaban mal.
  ✅ **§22 arbitrado**: `apply` son **62-90 ms**, no 177.

- [x] **65. ✅ Re-medido (§130) — y el instrumento caracterizado (§131).**
  ✅ **Canon nuevo con protocolo proceso-por-muestra**: send gen **320-355
  ms**, claim **217-240**, apply **32-38**, prueba **64,7/65,2 KB**. **El 620
  canonico pasa a historico**: estaba un 75 % por encima.
  ✅ **Hallazgo**: «la prueba» son **dos numeros** — claim genera un **33 %
  mas barato** que send. El canon anterior trataba los circuitos como uno.
  ⚠️⚠️ **Y el instrumento tiene DOS dispersiones** (§131): **σ 0,5 %
  intra-tanda** y **~9 % entre tandas** — 322,2 ± 1,8 frente a 353,2 ± 2,3,
  **trece sigmas**. §130 publico la primera como si fuera el canon; **es
  §123.2 con un instrumento mejor**.
  ⚠️ **Alcanza a todo lo medido hoy**: los 10,84 s del diccionario, los 28 ms
  del digest y los 160 ms de XMSS **arrastran la misma deriva** entre
  sesiones.

- [ ] **68. ⚠️ Que causa la deriva del 9 % entre tandas.** Termico, turbo,
  cache o carga — **no investigado y no conjeturado** (§131.5). Hasta
  saberlo, **ninguna medida de tiempo es comparable entre sesiones** a menos
  del 10 %. Medido **314 y
  375 ms** frente al **620 canonico** de `README.md`, `PRINCIPIOS.md` y
  `doc/ESCALADO.md` (§123.5). ⚠️ **Si se confirma, cae en cascada**: «mil
  transferencias ~620 s», el 1,24 s/dia, los percentiles de §3.1/§7 y la
  tabla de `PRINCIPIOS.md` §5. El README **ya lleva la marca de revision**.
  **Media hora.** `doc/ESCALADO.md` §2.2 la identifica como **su limite n.º 1**,
  y **medido el 02-08-2026: cero coincidencias** en `AUDITORIA.md` y
  `README.md` (§122.4).
  ⚠️ **Debe registrarse aunque el escalado no se adopte jamas**: es un limite
  medido **del sistema actual**, y no depende de si se adopta la propuesta
  que lo encontro. Criterio 6 de `VISION.md` §5 aplicado.

- [x] **62. Acuse de recepcion (B10.3) — POLITICA DECIDIDA (§121).**
  ⚠️ **AVANCE (§274)**: emitidos y acumulados. El acuse viaja en la
  respuesta de `applySend`/`applyClaim` —{`epoca` = seq+1, `n` = 1440,
  `hashPrueba`}— y `vista_acuses::raiz_de_epoca` construye el arbol desde
  el registro con las reglas de `zk_ssl_verify::acuses`, que §275 usara
  SIN reescribir. **Queda**: raiz + `N` firmados en la cabeza y el RPC del
  camino —§275, un solo rompimiento de formato—. **Limite declarado** en
  el asiento §274 y en `spec/RPC.md`: la respuesta no va firmada
  (160,5 ms/firma medidos → colapso ~x50 sobre §217, y §121.2 ya lo
  habia matado por indices); la ventana es <=1 latido con operador
  honesto.
  ✅ **N va comprometido en cada acuse**, inmutable y a la vista. Bajo
  congestion el operador honesto **emite N mayor y lo declara**: la
  congestion pasa de fabricar falsa evidencia a ser **degradacion visible y
  firmada**, y **un N obeso delata al censor**. Simetrico a §119: alli el
  emisor sobre un **suelo**, aqui el operador bajo un **techo**.
  ⚠️⚠️ **Correccion del borrador** (§121.2): decia que el operador **firma**
  cada acuse — **a miles de op/s eso agota los 2^40 indices XMSS en
  semanas**. Los acuses son **hojas bajo una raiz en la cabeza**: heredan la
  firma, **cero indices extra**. ⚠️ Es la **segunda extension pendiente de
  `EpochHead`** antes de su primer testigo.
  ✅ **Detector que no necesita N** (§121.3): contador de recepcion monotono
  — *«una operacion posterior entro y la mia no»* es **reordenacion inmune a
  la congestion**, porque **la congestion retrasa a todos y solo la censura
  adelanta**. Un `u64`.
  ⚠️ **Hereda §116** si usa `digest_of_proof` tal cual: **§116 se cierra
  antes**. Y **reservar «acuse»**: ya hay dos `receipt` que son otro animal.
  ⚠️ **Limite sin maquillar** (§121.7): **solo encarece censurar lo
  acusado**; negarse a emitir acuses sigue sin dejar evidencia portable.

  **CERRADA (§275, 2026-08-12) — el MAPA de la nota, pieza a pieza**: las
  reglas (pertenencia, posición, época, hoja) → §274 (`verify::acuses`);
  la emisión en la respuesta y el árbol por época → §274
  (`vista_acuses`); la **raíz y `n` FIRMADOS en la cabeza (v2)** y el
  **camino servido** (`zkssl_ackPath`) → §275; la firma que todo esto
  hereda → §236, extendida en §275 (`formatVersion` 2). Lo que esta nota
  **NO** cierra, y a dónde va: pruebas reales para las delegadas →
  nota 78; el mando CLI que compare diarios → nota 80.
  **Para `PRINCIPIOS.md`**: **N_max = 1.440 cabezas** —24 h, precedente MMD
  de CT—; a 70 ms/apply, 24 h absorben **>1,2 M operaciones**: a ese
  horizonte «era congestion» muere como defensa. **Caveat**: CT adjudicaba;
  **aqui el sistema produce el par condenatorio, no adjudica**.
  **Pendiente T4.**

- [x] **61. ⚠️⚠️ El codigo cita DOS DOCUMENTOS QUE NO EXISTEN.**
  Medido el 01-08-2026 (§120): `CONFIANZA_RESIDUAL.md` y `ESCALADO.md` **no
  estan en el arbol ni en el historial**, y se citan **24 veces** — **6 en
  codigo publicado** (`lib.rs`, `log.rs` ×3, `metrics.rs` ×2), 12 en
  `AUDITORIA.md`, 6 aqui.
  ⚠️ **El propio preprint tiene la frase que lo condena**: *«un numero que no
  describe nada ejecutable es una dependencia residual de la confianza del
  lector en el autor»*. **Una cita a un fichero inexistente es exactamente
  eso.**
  ⚠️ **Cuatro de las seis citas del codigo son de esa misma sesion**
  (§120.3): §101 cometido otra vez seis horas despues.
  ⚠️ **Sexta de la familia §95.2 y de clase nueva**: no una garantia sin
  verificar ni una salvaguarda inventada, sino **una comprobacion ofrecida y
  hecha imposible**.
  **DECISION ABIERTA** (§120.5): **A** commitear los dos —cerrando las 24
  citas, al precio de meter texto no revisado con cifras desfasadas— o **B**
  repuntar cada cita —mas fiel, pero **algunas no tienen destino**—.
  Recomendacion: **A con cabecera de estado**.


  ✅ **CERRADA (§266, 10-08-2026)**: **los dos documentos existen** —
  `doc/CONFIANZA_RESIDUAL.md` y `doc/ESCALADO.md`—, añadidos **los dos en el
  mismo commit `c46071c`**: «Commit the two ghost documents with
  correction-map headers, and add the citation guard». Es la **decisión A**
  que esta nota recomendaba, y las cabeceras de los propios ficheros lo
  declaran: «añadida al committear (AUDITORIA §120, decisión A)». Vino con
  compuerta: `tools/verificar_citas.py`.
  ⚠️ **Estado real de esa compuerta, medido hoy**: existe, pero
  `canon.sh` **no la invoca** y sale **en rojo sobre el árbol limpio** por
  una **plantilla de nombre de fichero** que `spec/rfc/PROCESO.md:14` da como
  ejemplo de cómo bautizar una RFC, y que la compuerta toma por cita. No invalida el
  cierre de esta nota: los documentos existen. Pero el tapón no lo mira
  nadie, y eso va con la **66**.
  ⚠️ **Lo que NO cierra**: el ámbito de esa compuerta es sólo `.rs`, y eso
  es la **entrada 66**, que sigue abierta. La 61 era el agujero; la 66 es
  el alcance del tapón.
- [x] **59. ✅ `native_merge`: una definicion (§125.1).** El miedo de §117
  —«dos Rescue que deben coincidir»— **se desmintio por lectura**: eran **el
  mismo wrapper caracter a caracter**. Cierre por **delegacion**, cero
  call-sites tocados.

- [x] **60. ✅ Dominios: una definicion canonica (§125.2).** `pub use` en los
  demas modulos: las rutas viejas resuelven, **el valor ya no puede
  divergir**. ⚠️ **Retirada honesta**: los dos `assert_eq!` de T2a se
  volvieron **tautologicos** y se retiran — **un test que no discrimina es
  una garantia falsa**.

- [x] **64. ✅ Citas a secciones: una viva, clase cerrada (§125.3).**
  `log.rs` §8.1 → **§10.1**, y `verificar_citas.py` **v2** valida que la
  seccion exista. **0 fantasmas · 0 secciones muertas.**

- [ ] **66. ⚠️ El guardian v2 excluye por FICHERO, no por bloque.** Su ambito
  es **solo `.rs`** (§125.4), asi que deja sin cubrir **las cientos de citas
  de `AUDITORIA.md` y `BACKLOG.md`** —el tejido del registro— para proteger
  ~30 de las cabeceras-mapa, que narran numeraciones viejas **a proposito**.
  ⚠️ **Si una cita de `AUDITORIA.md` apunta a una seccion renumerada, el
  registro se rompe por dentro y nadie lo sabe.** Arreglo: excluir por bloque
  marcado, no por extension.
  `rescue_hash.rs:121` y `merkle.rs:91`: **cuerpos identicos linea a linea**
  —mismo estado, mismas posiciones, misma permutacion— con firmas distintas
  solo por alias (`[BaseElement; 4]` vs `Digest`). **No es un re-export: son
  dos funciones.**
  ⚠️ **Hoy coinciden. Nada lo comprueba.** Es §94 aplicado a la funcion que
  compone **todos** los hashes del sistema.
  ⚠️ **Y toca lo decidido en §117**: el asiento dice que `derive_leaf_salt`
  usa «el mismo `native_merge`» **por construccion** — con dos definiciones,
  «el mismo» depende de **cual importe cada modulo**. Si una divergiera,
  **el salt y la identidad se computarian con hashes distintos**.
  **Arreglo**: re-export, o test que las compare sobre muestras.
  ⚠️ §792: **las dos mitades, medidas; SIGUE ABIERTA por lo que el guardian no mira.**
  La primera: `tools/verificar_citas.py` v4 mira las citas §n de `AUDITORIA.md` y de este
  fichero —5.844 y 962 en `8c02941`— contra los encabezados `## n.` y `## §n` del registro,
  y las «AUDITORIA §n» de fuera (394), y excluye por BLOQUE marcado
  (`<!-- numeracion-vieja -->` y `<!-- /numeracion-vieja -->`), no por extension. Medido
  antes de escribir la regla: ninguna cita muerta de verdad. Las que no nombran un
  encabezado son 31 a numeros que el registro RESERVA para otras sesiones (630, de 711 a
  739 y de 741 a 769) o al resto del rango de la que lo escribe (hasta 799), y dos al cero:
  el de otro documento, sin nombrarlo, en el §235, y el numero provisional que narra el
  §594, un cero con «-ECST» detras; todas en asientos, que no se reescriben: se toleran por
  su asiento, declaradas en la herramienta, y en este fichero no hay tolerancia. Ninguna
  cabecera-mapa necesita hoy la marca: existe, con su prueba de vida, y no va en ningun
  sitio. Lo que NO mira: el `.m` de `§n.m` —de 620 en `AUDITORIA.md` sin otro documento
  delante, 102 no abren ningun encabezado `n.m` del registro, muchas de otro documento con
  su nombre detras o sin el—, los `§n` sueltos de los demas documentos, y la v2 sigue siendo
  de `.rs`: sobre los `.md` daria, en `8c02941`, seis rojos que no son rot,
  `PRINCIPIOS.md` §7.2 a §7.4, que nombran los puntos en negrita de su §7 y no encabezados.
  ✅ **La segunda mitad esta resuelta desde el §254.** Hay una sola definicion,
  `crates/zk-ssl-hash/src/lib.rs:100`, y `crates/stark-experiment/src/rescue_hash.rs:129` y
  `crates/stark-experiment/src/merkle.rs:103` la reexportan con
  `pub use zk_ssl_hash::native_merge;`: el §125 habia dejado un delegado, y el §254 lo
  cambio por el `pub use` (`rescue_hash.rs:122-128` lo cuenta). `grep -rn 'fn native_merge'
  crates` da una.

- [x] **58. ✅ `digest_of_proof` INYECTIVO (§124).** Codificacion por limbs
  de **32 bits sin reduccion al campo** + bloque final de longitud.
  ⚠️⚠️ **El arreglo que §116 habia registrado NO habria bastado** (§124.1):
  al implementarlo aparecio una **segunda familia** —`p` colisionando con
  ceros **de la misma longitud**— que la longitud sola no cubre. §95.2
  **cazado antes de committearse**, primera vez del dia.
  ✅ **Y abarata**: 3.876 merges frente a 7.750 para 62 KB —**la mitad**—;
  **28,06 ms** medidos (n=20), ~1/3 del `apply`.
  ✅ **Corte limpio**: `persistence.rs` no contiene «chain», `LogEntry` no
  deriva serde ⇒ **nada almacenado codifica el digest viejo**. **Sin
  compatibilidad hacia atras a proposito**: un modo compatible conservaria la
  debilidad.
  ✅ **T6 4/4**, y **213 verdes siguen verdes** —T1 y T3a incluidos: la
  propiedad retroactiva de §115 se sostiene sobre el digest nuevo—.
  **Desbloquea §121.5**: el acuse nace sin heredar deuda.
  Hallazgo colateral de §115 (§116): **dos pruebas que difieran solo en ceros
  finales colisionan**. Su doc lo llama «atar, no hash de proposito general»,
  y no habilita fabricar pruebas validas alternativas — **pero eso depende de
  que el verificador rechace la prueba alterada**: es la **CUARTA garantia
  por consecuencia** del proyecto (§105.2, §107.1).
  ⚠️ **Agravante**: el `chain_digest` que T1 valida —y que sostiene la
  decision de cadencia de §115— **incluye `digest_of_proof`**. La cadena que
  ata toda la historia es tan fuerte como ese resumen.
  **Arreglo**: bloque final con la longitud del mensaje.

- [x] **53. ✅ CERRADA (§127.1). Firma XMSS: elegida, evaluada, cadencia
  medida.** Lo unico vivo era **un juicio declarado**, ratificado **como
  juicio**: que 60 s no compran nada a un operador deshonesto es **premisa
  sobre el USO** —liquidacion institucional supervisada—, **revisable si la
  mision cambia**. ⚠️ **Reversion viva**: sub-minuto **sin peticion** cuesta
  **0,58 TB/año**.

- [x] **49-A. ✅ CERRADA 5/5. Clave de vista, ALCANCE MEDIDO (§129), desplegada en cinco
  pasos.** ⚠️⚠️ **NO es «desplegar una credencial»: es migracion de formato de
  cuenta en disco** —dos rutas de serializacion, `[u8; 48]` → 80, sin byte de
  version—. ⚠️ **`snapshot.rs:333` lleva `take(48)` codificado**: un parche que
  solo tocara `store.rs` **escribiria 80 y leeria 48 — corrupcion silenciosa,
  sin error de compilacion**.
  ⚠️ **Indicador corregido**: pone verde **UN** rojo, no tres —
  `account_indices_are_not_predictable` es **49-B** y
  `a_neighbour_leaf...` es la **50**, y su cuerpo lo declara—.
  ⚠️ **Y no se exige credencial en las cuatro puertas**: **97 usos de
  `balance_of` en once ficheros**, 13 en `iso.rs` y 7 en `two_phase.rs` —
  **camino de produccion**. Se **añade** `account_view_authenticated`.
  ✅ **Discriminador por longitud** —48 viejo, 80 nuevo— con precedente en la
  migracion de `legacy_null`. **NO-RETROACTIVA** (§129.3): centinela para las
  cuentas viejas, **misma familia que §117 y §119**.
  **Sub-decision abierta**: ¿los snapshots viejos se migran o se regeneran?
  ⚠️⚠️ **Su «depende de la 53» era verdad AL REVES** (§127.2): XMSS firma
  **cabezas del operador**, una identidad; la 49 necesita que **cada titular**
  pruebe **su** cuenta. **La 53 tenia que resolverse para ver que no la
  resuelve.**
  ✅ **Decidido: clave de vista** `derive_view_key(sk)`, verificada
  **nativamente** —un merge, y **no viaja en cada operacion**, a diferencia
  del salt que §109 descarto por eso—. Se guarda **`view_id` = hash**, no la
  clave: el operador puede **comparar**, no **leer**.
  ✅ **T7 3/3**, incluido `t7_la_vista_no_es_credencial_de_GASTO` —sin el,
  autenticar la lectura seria entregar el gasto— y `t7_dominio`: **si
  coincidiera con `LEAF_SALT`, presentar la vista revelaria el salt de §117**
  y reabriria la 50.
  ⚠️ **Limitacion declarada** (§127.5): acoplada a la clave de gasto, asi que
  **rotarla exige rotar la de gasto** — si se compromete, **no hay privacidad
  de lectura sin cambiar de cuenta**.
  ✅ **PASOS 1-2 CERRADOS con test** (sesion 2026-08-03, guarda `store.rs`
  `58a2a353…`, `accounts.rs` `ed0947c3…`):
  · Paso 1 — serializacion dual por longitud (48 viejo→centinela, 80 nuevo)
    + `VIEW_ID_LEGACY`; `record_v2_roundtrip_y_dual` 5/5.
  · Paso 2 — `AccountRecord` gana `view_id`; poblado en 13 constructores de
    9 ficheros; `derive_view_key_wide`/`view_id_of_wide` (heredan §90);
    accessor `stored_view_id` (lo usa el paso 4); `t_paso2_view_id` 4/4.
  ⚠️⚠️ **HALLAZGO DE SEGURIDAD (lo destapo el compilador, NO estaba en §129)**:
  `two_phase` ×2 y `burn` reconstruyen el record desde el **ClientState
  ENTRANTE** (input del cliente), no desde el guardado. Poblar
  `view_id: input.view_id` dejaria a **un cliente reescribir su propia
  credencial de lectura en cada operacion**, anulando 49-A por dentro.
  Corregido: leen `self.records.get(&idx).view_id`. `mint`/`transfer`
  verificados (parten del record guardado). Regresion
  `operar_preserva_el_view_id` lo blinda.
  ⚠️ **COSTURA 49-A↔52 declarada** (`recovery.rs` ×2): recovery rota el
  `public_id`, el view_id viejo ya no deriva de la clave nueva y la capa no
  puede recalcularlo (§93.4); se copia el viejo con TODO. El cierre —traer
  el view_id nuevo en el receipt— es diseno de la 52. Test de recovery
  pendiente (necesita receipt valido).
  ⚠️ **FALTAN pasos 3-5**: (3) migrar 5 call-sites `#[deprecated]` de
  `record_to_bytes`/`_from_bytes` (WRITE de `snapshot.rs:177` + 4 tests) +
  sub-decision snapshots viejos migran/regeneran; (4) puerta
  `account_view_authenticated` (usa `stored_view_id`) → pone verde
  `reading_a_balance_requires_authority`; (5) eliminar shims deprecated.
  ✅✅ **CERRADA (5/5)**: P3 snapshot v5 (MAGIC_V5, 80 B) + persistence
  WRITE _v2, `snapshot_v5_preserva_view_id` 1/1. P4 `account_view`→
  `pub(crate)` + `account_view_authenticated`, `reading_a_balance_
  requires_authority` VERDE. P5 imports muertos + BUG corregido (el
  `#[deprecated]` de `open_account` estaba desplazado sobre
  `stored_view_id` desde el P2). Suite 228 verdes; los 2 rojos son 49-B
  (§133). Hallazgos de seguridad cazados: ClientState reescribible (P2),
  atributo desplazado (P5) — ambos invisibles a un despliegue de una tacada.

- [x] **67. ⚠️ Indices predecibles — INTENTADA Y REVERTIDA (asiento del
  2026-08-03): imposible EN SOLITARIO, condicionada a la migracion unica.**
  ⚠️⚠️ **El intento rompio 27 tests y la causa es UNA constante:
  `FROZEN_DEPTH = 24`.** El indice de cuenta es coordenada COMPARTIDA
  entre el arbol de cuentas (2^32) y el de congelados (2^24): la posicion
  derivada produce indices que el frozen no puede alojar, y send/claim/
  burn/transfer llevan `frozen.path_for(indice)` dentro. La invariante
  «indice < 2^24» la cumplia la secuencia de gratis; nadie la habia
  declarado. Reversion inmediata; arbol en 232/2.
  **Salida elegida**: crecer frozen a 32 → es SU propia migracion → la
  migracion unica de B13/B14 pasa de dos frentes a TRES (reposicionar +
  salt-cero + frozen a 32, con sus AIR de congelacion). Encoger el espacio
  a 2^24 destriparia la mitigacion (molienda de horas a minutos). El «una
  pasada» de §133 no era eficiencia: era necesidad estructural.
  **Sobrevive del intento**: derived_position con sondeo (correcto sobre
  frozen-32), cota ~2^31, snapshot ya disperso-compatible, 2 tests listos.
  **Regla de metodo nueva**: antes de cambiar el dominio de un
  identificador, censar TODAS las estructuras que lo usan como coordenada
  — el grep es `path_for(.*index)` en todos los arboles, no `0..next`.
  ⚠️⚠️ **«La mitigacion mas barata» era lo contrario**: el indice **ES la
  coordenada del arbol**, y el cliente **lo recibe, no lo recupera** — hoy es
  secuencial y por eso **un cliente que lo perdio puede barrer 0..n**.
  **Aleatorizarlo cambia una fuga de privacidad por un MODO DE PERDIDA DE
  FONDOS** y rompe la recuperacion de §127. **Cuarto §95.2 de la sesion.**
  ✅ **Solucion: `posicion = H(public_id)`** — el atacante **no elige
  vecino** (objetivo cumplido) **y la recuperacion MEJORA**: con solo la
  clave se encuentra la cuenta **sin barrer**. Y **el arbol ya es disperso**
  —`TREE_DEPTH` 32, 2^32 slots—: cabe nativo.
  ⚠️ **MEDIA, no barata**: colisiones **con argumento de solidez** —que un
  atacante no pueda forzar coincidencia—, snapshot disperso, y migracion.
  ⚠️ **Dependencia dura con B13/B14** (§133.4): **las dos recomputan el
  arbol**. Coordinarlas ahorra un evento **y es mas arriesgado** —dos cambios
  con una sola prueba de que salio bien—. **Sin resolver.** Mitad B de la 49 (§127.6): las altas dan **indices
  consecutivos**, asi que **quien controla el momento de su alta ELIGE a su
  vecino de arbol y con dos altas lo rodea** — convierte la fuga de la 50 de
  **oportunista en DIRIGIDA**. Medido desde §93.3 y **sin entrada propia
  hasta hoy**. Arreglo: aleatorizar la posicion de alta o derivarla del
  `public_id`. **Ortogonal a la autenticacion.**
  ✅ **Cadencia (§115)**: **1/min + a demanda**, con cache por `seq`
  idempotente. El peor caso —un testigo pidiendo cada segundo— **degenera al
  escenario 1/s ya cuantificado**: techo conocido, no regimen.
  ⚠️ **CORREGIDO (§636): el «a demanda» se decidió y NO se construyó.** Sólo
  firma el latido; `zkssl_signedEpochHead` sirve la última cabeza conservada,
  y el test `pedir_la_cabeza_firmada_no_firma_otra` lo ata. Un testigo pidiendo
  cada segundo no cuesta ninguna firma. Queda corregido aquí en vez de borrarlo.
  ✅ **La premisa esta MEDIDA**: `t1_chain_retroactivo`, **3/3** — una cabeza
  firmada en `n` **ata las epocas anteriores**, asi que los 60 s son
  **latencia de oponibilidad, no impunidad**.
  ✅ **Cierra**: la retencion muere sin decidirse (9,7 GB/año), la CPU baja al
  **0,27 %**, y BDS pasa de necesidad a mejora.
  ⚠️ **Reversion viva**: prometer oponibilidad **sub-minuto sin peticion**
  cuesta el escenario 1/s —0,58 TB/año—. Cambio de mision.
  ⚠️ **Juicio declarado**: que 60 s no compran nada a un operador deshonesto
  **es una premisa sobre el uso**, no un hecho del codigo.
  ✅ **Evaluado el 01-08-2026** (§112-114, `doc/xmss-evaluacion.md`):
  **`xmss` 0.1.0-pre.0 de RustCrypto** —misma organizacion que `sha2`, ya
  aceptada— es **la unica viable**. Solo hash ✅, agotamiento ✅, alturas ✅;
  **indice sin API** ⚠️ y **sin auditoria** ⚠️.
  ⚠️⚠️ **El criterio 3 era insatisfacible** (§113): un blob restaurado es
  indistinguible de uno legitimo, y **SP 800-208 asume lo mismo** —por eso
  exige hardware—. **Decimotercera correccion del dia**, de un criterio
  redactado veinte minutos antes.
  ⚠️ **El guardian no es contingencia: es el mecanismo.** Contador propio,
  `fsync`, firmar-despues-de-persistir, test de layout y reconciliacion que
  **nunca retrocede**. **Se declara: la seguridad pasa a depender de codigo
  propio NO auditado.**
  ⚠️ **La firma cuesta O(d·2^(h/d))** —reconstruye el arbol cada vez—:
  **160,5 ms** en MT 40/8. Propiedad de la **implementacion**, no del esquema.
  ⚠️⚠️ **INCOGNITA ABIERTA** (§114.1): toda la evaluacion asume **1 firma/s**.
  A **1/min** el almacenamiento cae de **0,58 TB/año a 9,7 GB** y `40/4`
  vuelve a ser viable. **No es un parametro: es cuanto tarda una vista
  dividida en ser oponible.** Decision con victimas, sin tomar.
  **Pendientes**: KAT, fijar el tag, issue upstream
  (`doc/issue-rustcrypto.md`), el guardian, y medir en ARM.
  ⚠️⚠️ **CORRECCION (§110.1)**: se dijo que la 53 era «lo unico que puede
  empezarse sin decidir nada mas». **Es falso**: XMSS necesita persistir su
  indice **antes de firmar**, y eso es un requisito sobre la persistencia que
  el proyecto **no cumple**.
  ⚠️ **XMSS convierte una perdida de durabilidad en perdida de SECRETO**
  (§110.2): `persistence.rs` justifica no tener WAL porque «perder una
  operacion es recuperable»; con XMSS **no lo es**, porque reusar el indice
  **filtra la clave**. La entrada 19 no estaba mal — **XMSS cambia su
  premisa**.
  ⚠️ **Y el reuso de indice es AMBIGUO** (§110.3): §103.3 lo celebraba como
  «el modo de fallo ES el fraude», y un reinicio honesto produce el mismo
  evento. **«Te pille mintiendo» y «acabas de perder tu clave» no se
  distinguen desde fuera** — con la cabeza ya publicada.
  **Sigue en pie**: XMSS es la eleccion correcta (§106.3). **Cambia el
  orden**: primero garantizar que el indice no retroceda. Dos vias sin medir
  en §110.5.


  ✅ **CERRADA (§266, 10-08-2026)**: la salida que esta nota eligió
  —`posición = H(public_id)`— es la que se desplegó. **F3 (§157,
  `e3116c6`+`f926de6`)**: `open_account` coloca por `public_id mod
  capacidad` con sondeo lineal, `next_index` queda como censo, y **los dos
  contratos que esta nota perseguía vigilan en verde** —
  `account_indices_are_not_enumerable` y `account_indices_are_not_predictable`,
  los dos en `crates/zk-ssl/src/client.rs`—. La «dependencia dura con
  B13/B14» que quedaba sin resolver se resolvió coordinándolas: el mismo
  evento recomputó el árbol una vez.
- [x] **48. `CONFIANZA_RESIDUAL.md`: la evidencia contra el operador esta en
  manos del operador.** Documento externo del 31-07-2026, sin integrar.
  ✅ **Su tesis central se sostiene y esta bien anclada**: el README afirma
  que el operador «no puede reescribir el historial en secreto», y **la
  garantia es condicional**: un log encadenado solo impide reescrituras
  detectables por quien ya vio una cabeza anterior, y **hoy nadie fuera del
  operador ve cabezas**. De ahi salen la vista dividida, que el supervisor
  verifica contra raices que le da el propio operador, y que **censura y
  no-recepcion son indistinguibles**.
  **Tres piezas**: **B10** cabezas firmadas y publicadas a testigos —patron
  de Certificate Transparency, RFC 6962—, con prueba de fraude portable y
  recibos de recepcion e inclusion; **B11** operador ciego —limite por
  operacion al circuito con la maquinaria de Horner que ya existe, y cifrado
  del aviso—; **B12** especificacion formal del AIR y auditoria.
  ⚠️ **RECONCILIACION, no anexo**:
  • **B12.2 ES la entrada 7.** No se duplica: la 7 queda actualizada con lo
    que esta propuesta le aporta —el contrato—.
  • **B11 depende del C3 de `ESCALADO.md`** (entrada 47). Sin ese rediseño
    de entradas publicas, el operador necesita ver estado para componer.
    **B11.2 —cifrar el aviso— es la excepcion: independiente y hoy.**
  • **B10.1 no depende de nada**: el log ya tiene `seq` y `chain_digest`;
    publicar es aditivo. Implementable sobre el nodo unico actual.
  ⚠️ **Su propia §8 acierta el punto debil**: la independencia de los
  testigos es un **supuesto social, no criptografico**, y «quien atestigua a
  los que atestiguan» no tiene respuesta tecnica. El documento no finge
  tenerla, y eso hay que reconocerselo.
  ⚠️ **Y repite una decision de politica ya conocida**: «no inclusion en N
  epocas = censura» exige elegir N, y un N corto convierte congestion en
  falsa evidencia. **Misma clase que el timeout de §88.5**, mismo
  tratamiento: es politica, no parametro.
  ⚠️ **Cita ~620 ms** para la transferencia; es la maquina lenta de §22. En
  la de hoy son 437 ms. No invalida su argumento —usa el valor conservador—
  pero al integrarlo hay que fechar la maquina, como obligo §89 con
  `ESCALADO.md`.


  ✅ **CERRADA (§268, 10-08-2026) — REPARTIDA, no descartada.** Medido contra
  el árbol: sus tres bloques ya tienen dueño, y lo único suyo que no lo tenía
  —la tesis— se corrige en el `README`.
  · **B10 — cabezas firmadas a testigos, con recibos: HECHO.**
    `crates/zk-ssl-node/src/firma_cabeza.rs` firma la cabeza con XMSS;
    `zkssl_signedEpochHead` la sirve; `crates/zk-ssl-cli/src/witness.rs` es un
    **testigo de referencia con mando propio en la CLI** que la verifica con
    `zk-ssl-verify` y **fija la clave que ve la primera vez** —TOFU—. El
    recibo de inclusión es §256-§259, y el **contador de recepción** de §121.3
    vive en `crates/zk-ssl-node/src/recepcion.rs`, monótono y persistido.
  · **B11 — operador ciego: va con la entrada 47**, como esta misma nota ya
    decía (depende del C3 de `ESCALADO.md`). ⚠️ **Su excepción B11.2 —cifrar
    el aviso— es el mismo objeto que la entrada 11**, que lo mira por la otra
    cara: el aviso viaja fuera del mensaje ISO. **Anotado allí**, para que no
    siga viviendo en dos sitios.
  · **B12.2 — especificación y auditoría: es la entrada 7**, como esta nota ya
    decía. No se duplica.
  ⚠️ **Y su tesis cambió de signo.** Se apoyaba en que «hoy nadie fuera del
  operador ve cabezas», y eso dejó de ser cierto: hay método que las sirve
  firmadas y testigo que las consume sin compilar el código del operador. La
  garantía sigue siendo **condicional** —vale para quien ya vio una cabeza
  anterior— pero **ahora la condición se puede cumplir**. El `README` la
  afirmaba sin condición; §268 se la pone, encima y sin tocar el párrafo.
- [x] **69. Tres ayudantes de test sin usar en `circuit_settlement.rs`.**
  `SK` y `d()` en `t2b_recuperacion_nativa`, `claves()` en `t2a_salt_hoja`
  (§136.3). **Peso muerto, no garantia falsa** —a diferencia de los
  `assert_eq!` de §125.2—. Limpieza, no hallazgo.

  ✅ **CERRADA POR DERRIBO en el §176** (`1f41a1d`): el `circuit_settlement.rs` de
  `stark-experiment` se borro entero, y con el los tres ayudantes; ni
  `t2b_recuperacion_nativa` ni `t2a_salt_hoja` existen en el arbol. ⚠️ Siguio
  abierta hasta el recuento del §583: el derribo no miro esta lista.

- [x] **50. ⚠️ PRIVACIDAD FRENTE A TERCEROS — propiedad DEMOSTRADA,
  despliegue pendiente (§126).**
  ✅ **La clausula de caida de §117 se resuelve A FAVOR**: `T2b-nativo` 3/3
  demuestra que **un titular que pierde todo salvo la clave reconstruye su
  hoja**, y que el ataque medido de 10,84 s **no acierta** con salt —**con
  control de que si acierta con el salt correcto**, para que el cegado no
  oculte tambien al legitimo—.
  ⚠️ **T2b se partio en dos** (§126.2): la clausula decia «cae si la
  propiedad es **irrealizable**», no «si el test no puede escribirse hoy» —la
  hoja se computa **dentro de la traza**, asi que el salt es cambio de AIR en
  los cinco circuitos—. **T2b-circuito queda especificado y NO escrito**: un
  test que no compila disfrazado de verde seria peor que ninguno.
  ⚠️⚠️ **Y se cierra HACIA DELANTE** (§126.4): **las cuentas abiertas antes
  del despliegue siguen barribles en 10,84 s**, y **ningun salt futuro lo
  arregla**. A diferencia de §98.2, aqui **hay que rotar para ganar la
  privacidad**: una cuenta vieja que nunca rote **queda expuesta para
  siempre**.
  ⚠️⚠️ **B13/B14 DISEÑADA (§132), ningun AIR abierto.** **NO es «`native_
  leaf_salted` en cinco AIR»**: son **OCHO** circuitos con layout propio, la
  restriccion de hoja es una **maquina de estados de dos carriles**, y el
  **coste NO es negativo** —§86 midio *ensanchar*; el salt **añade un
  merge**—. ⚠️ **Esa cita era §95.2 y se repitio tres veces.**
  ✅ **Arquitectura elegida**: **envoltura uniforme salt-cero** —una sola
  formula en los ocho, **sin selector ni solidez condicional**, y la
  no-retroactividad de §126.4 **cae por construccion**—.
  ⚠️ **El paso 1 NO es tocar un AIR**: es la **migracion de raiz** —`OpKind`
  propio, prueba raiz_nueva↔raiz_vieja, snapshots—, que **es su propia
  entrada**.
  ⚠️ **Cada restriccion nueva exige test de MUTACION**: sabotear el salt y
  ver que el AIR rechaza — **sin el, la restriccion es fe**.
  ⚠️ **Mitigacion provisional**: vale documentar el riesgo en la apertura;
  **se descarta** meter entropia en balance/nonce —**el balance es dinero** y
  el nonce lo ata `C_NONCE`— (§132.6).
  (§108).** ⚠️ **§99.4 descarto mal** la familia «salt en el estado»: pidio
  «ocultante frente a todos» cuando **la 50 es frente a TERCEROS** —el
  operador ya ve los saldos y esta declarado— y **no distinguio derivar de
  conservar**: la capa no necesita computar el salt, **lo transporta** como
  transporta `public_id`. **Verificado**: `AccountView` —lo que lee
  cualquiera— **no tiene por que llevarlo**; el camino Merkle son hashes;
  `state_of` **no existe como API**, asi que el canal del titular **se puede
  diseñar con autorizacion desde el principio**. ⚠️ **Sin resolver**: de donde
  sale el salt al abrir y si se puede recuperar (§93.4), y el coste en los
  cinco circuitos —**clase entrada 15**, coste medido negativo en §86—.
  ⚠️ **No es una solucion: es una familia viva.** Y es la **novena
  autocorreccion del dia** (§108.5).
  ~~50. ⚠️ PRIVACIDAD FRENTE A TERCEROS ROTA: la hoja no lleva salt.**
  **Medido** el 31-07-2026 (§93.2): el saldo del vecino de arbol se recupera
  por diccionario en **10,84 s** desde `sender_path.siblings[0]`, que el
  propio protocolo entrega al cliente. `native_leaf(pk, saldo, nonce)` **no
  lleva salt** y `native_merge` no tiene cegado. ⚠️ **El coste es una CURVA
  sobre el rango de saldo asumido**: **2,4 min** para 0-10.000 EUR, 4,1 h
  para 0-1 M, 8,3×10^7 años-nucleo si el saldo fuera uniforme en 64 bits —y
  **nunca lo es en un sistema de dinero**: por eso el salt es la unica fuente
  de entropia posible—. **Alcance: 1 cuenta**, solo `siblings[0]`; los 31
  hermanos restantes no son diccionariables. **Regimen 1D confirmado**: el
  nonce nace en cero. ⚠️ **Y el vecino es ELEGIBLE** (§93.3): las altas dan
  indices consecutivos, asi que quien controla su alta elige victima.
  ⚠️ **NO es clase entrada 15** (§93.4): un salt exige que el cliente
  **custodie estado**, y hoy `ClientState` lo pide todo a la capa. Es cambio
  de modelo de cliente, con victimas —quien pierde el salt, pierde la
  cuenta—. ⚠️⚠️ **RECTIFICADO el 01-08-2026 (§99): el obstaculo que §93.5 dio por
  bloqueante era FALSO.** El emisor **nunca computa la hoja del receptor**
  —`circuit_send` usa `COL_R_ID` solo para el compromiso del pendiente—; el
  diseño de dos fases existe para eso. **Cada uno recompone su propia hoja
  con su propia clave.** Y §93.4 tambien quedo desfasada: desde §97,
  `prove_send` y `prove_claim` reciben la clave de 256 bits, asi que un salt
  derivado seria recuperable **sin almacen nuevo**.
  ⚠️ **El obstaculo REAL es otro** (§99.3): **la capa escribe hojas sin
  conocer el secreto**. `open_account`, `mint`, `freeze` y `recover`
  recomponen `native_leaf` del titular **sin su clave** —`circuit_mint` usa
  las de los custodios—. Un salt derivado de `sk` haria esas hojas
  incomputables. **Descarta la unica familia de soluciones que parecia
  viable, y no hay otra medida.**
  ~~El obstaculo real no es el nonce~~ —el circuito ya usa
  nonces distintos por carril— **sino la asimetria emisor/receptor**
  (§93.5): con salt derivado de clave, el emisor no puede computar la hoja
  nueva del receptor. **Ninguna salida medida.**

  ✅ **HITOS B13/B14 — paso 1 COMPLETO (2026-08-04)**: (1a) `leaf_salt`
  en el record, formato v3 (112 B) + snapshot v6, 10 constructores, el
  salt cruza el disco verificado (`2327877`). (1b) el evento de
  migración, CUATRO sub-frentes probados — reposicionar con sondeo
  determinista, envolver con el salt DEL RECORD (enmienda E1 a la spec,
  fijada en test), frozen-32 sobrevive reinicios, sled con marcador
  `meta:migrated` y carga geometría-consciente; el ledger migrado
  REINICIA y la re-migración se rechaza (`dcc2af0`, `f1c08f2`). NO se
  ejecuta en vivo hasta el flip: los AIR de hoy verifican hoja sin salt
  y frozen-24. (1.5) spec de la máquina de hoja ANTES de tocar AIR
  (`doc/spec-maquina-de-hoja.md`): carriles = estado viejo/nuevo, delta
  salt = +4 columnas y 6 restricciones `C_SALT_*` (§92.2 aplicada al
  salt), frozen-32 con presupuesto POR circuito, mutación obligatoria;
  cuestión previa señalada (limbos 9..11 del rate del nonce). Ambas
  specs en `doc/`.
  ✅ **Paso 2 (previo) — cuestión previa de la máquina de hoja RESUELTA**
  (`1ba848d`): `C_NONCE` ata solo `next[8]`; los limbos 9..11 del rate no
  tienen restricción propia, PERO no es explotable — `C_HASH` ata la
  permutación Rescue completa (rate incluido) y un limbo corrupto rompe
  aguas abajo. Probado con test + canario (no supuesto), leído el porqué.
  Asiento §138. Para `C_SALT_IN` se atarán los 4 limbos por claridad.
  SIGUE: paso 2 propiamente — añadir `C_SALT_*` a `circuit_send` (el
  tercer merge del salt + 2 mutaciones obligatorias).
  ⚠️ **Paso 2 — OBSTÁCULO encontrado, TRES reversiones (asiento §139)**:
  insertar el merge del salt en `circuit_send` por corrimiento manual es
  inviable — el trace codifica la posición de cada camino en TRES sitios
  desincronizables (constantes `ROW_*` en filas, offsets `(N+level)*
  CYCLE_LENGTH` en columnas, y rangos+aritmética en el `match r` 477-489,
  con ciclos-frontera no uniformes). Tres intentos, tres `git checkout`,
  árbol 286/0 intacto. Tercera aparición de §137 (censar TODAS las
  representaciones). **ESPERA DECISIÓN DE ESTRATEGIA**: (a) HACK — salt en
  la holgura del final (744→751, cero corrimiento, trace no-temporal); (b)
  REFACTOR — unificar las 3 representaciones antes de insertar. Recomendado
  PRIMERO: producir el mapa completo de la geometría de `circuit_send` como
  documento; con él la elección es obvia. El plan SB1→SB5 sigue válido
  salvo la mecánica de dónde meter las 8 filas.
  ✅ **Paso 2 — decisión tomada y SB0 EJECUTADO (2026-08-04, §140-§141)**:
  el mapa completo existe (`doc/mapa-geometria-circuit_send.md` +
  `tools/verifica_geometria.py`, verificación mecánica que reproduce los
  intentos de §139); la elección fue REFACTOR — frozen-32 obliga al
  corrimiento que el hack pretendía evitar. SB0 completo en cuatro pasos
  compilables (`0451462`, `564f45e`, `0639a52`, `96264ba`): geometría en
  UNA representación (`CYC_*` → `ROW_*`, bucles y `match` derivados),
  `debug_assert` en los tres `place_*`, y la guarda de presupuesto
  `ROW_PENDING_ROOT < TRACE_LENGTH` en compilación. Suite release
  canónica en cada eslabón. SIGUE: SB1 — el salt de verdad (columnas,
  `link_salt`, las seis `C_SALT_*`, 2 mutaciones); la estadificación de
  `FROZEN_DEPTH` (§140.3) ABIERTA, del autor.
  → **Estrategia del piloto RATIFICADA: C, gemelo andamio (§142,
  `bab7097`)** — `circuit_send_salted` nacido con cláusula de retirada
  en el flip; la capa sigue 240/2 toda la campaña; la estadificación
  §140.3 queda resuelta de facto (profundidad local del gemelo, el
  flip unifica). SIGUE: SB1.b-e sobre el gemelo.
  ✅ **Paso 2 COMPLETO — el PILOTO send, entero en el gemelo (§143,
  `b9f8a74`→`7f84ff8`)**: salt (tercer merge, seis `C_SALT_*`/24
  ranuras) + frozen-32 local + las 2 mutaciones de la spec §4
  rechazando + nativo↔circuito limbo a limbo. Presupuesto MEDIDO:
  `ROW_PENDING_ROOT` 815, holgura 208 — primera fila de la tabla de la
  spec §3. SIGUE: paso 3 (los nueve gemelos, lista de rangos de §141;
  dos avisos para el flip en §143: `frozen_climb` clava 24, y los
  números de comentario se revisan EN el paso que mueve geometría).
  Receta destilada: `doc/playbook-replica-gemelos.md` — gemelo-primero
  y SB0 DENTRO del gemelo (el legacy no se toca: muere en el flip).
  ✅ **claim COMPLETO (§144, `e0986fb`→`a9e178e`)**: seis eslabones
  R1-R6, las dos mutaciones rechazan, fila 2 de la tabla §3 (815/208,
  CABE). Dos gemelos de diez. SIGUE: burn.
  ✅ **burn COMPLETO (§145, `8a0da2b`→`48b0ef9`)**: PRIMER DESBORDE
  cazado por la tabla (512 no alcanza: 543) → TRACE propia 1024
  (holgura 480; coste ~2× crudo: 15,0-15,6 s vs 13,9 s, para el paso
  5). Tres de diez. SIGUE: mint (un tramo, sin frozen; spec §5: sin
  carril de resta).
  ✅ **mint COMPLETO (§146, `892874f`→`198e5a9`)**: doble árbol
  (custodios 2-de-N intactos), frontera sin sombra documentada, el
  titular asciende al mundo ancho (su salt, §117), R6 n/a. Fila 4 de
  la tabla §3 (319/192, CABE en 512). Cuatro de diez. SIGUE: audit
  (spec §5: no muta estado).
  ✅ **audit COMPLETO (§147, `972dc25`→`20aa611`)**: UN carril — tres
  familias/12 ranuras, espejo de una hoja, salt en el TESTIGO (cero
  firmas). Costura del atacante (salt observable; el secreto es la
  clave). Fila 5 (287/224, CABE). **CINCO DE DIEZ: mitad de
  campaña.** SIGUE: mint_climb.
  ✅ **mint_climb COMPLETO (§148, `f006c32`→`04fd772`)**: la fase de
  cuentas de mint aislada; dos derivas del legado (doc de custodios,
  cadena P amputada) reparadas/respetadas; dos abortos de guarda con
  CERO bytes. Fila 6 (279/232, CABE). Seis de diez. SIGUE: recovery
  (el salt se COPIA — §93.4, costura 52).
  ✅ **recovery COMPLETO (§149, `9d22008`→`7de56e3`)**: LA COPIA
  certificada por el espejo (mismo salt, ids y nonces distintos);
  doble ascenso ancho; C_NONCE+1 ata el salto del nonce. Fila 7
  (319/192, CABE). Siete de diez. SIGUE: recovery_climb.
  ✅ **recovery_climb COMPLETO (§150, `92180eb`→`e866d25`)**: forma
  mint_climb + semántica recovery (salto de nonce, LA COPIA); raíz
  pub; un aborto de guarda (el trío compacto). Fila 8 (279/232,
  CABE). OCHO de diez. SIGUE: freeze y frozen_climb — SIN salt,
  rito R1+R2+R6 con mutaciones de profundidad.
  ✅ **freeze COMPLETO (§151, `4090d23`→`0fb407f`)**: la CASA de
  FROZEN_DEPTH gira 24→32 (§128) — R2 derivó, R6 fue una línea; dos
  mutaciones de profundidad estrenan categoría; regla nueva de anclas
  (clausuras homónimas → nombre completo). Fila 9 (295/216, CABE).
  NUEVE de diez. SIGUE: frozen_climb (24 clavado en :166; relleno
  CARO §60.2).
  ✅ **frozen_climb COMPLETO (§152, `5f799b0`→`1dc4249`)**: giro a 32
  local con ajuste EXACTO (ROW_ROOT = TRACE−1; §60.2 disuelto por
  geometría); los dos avisos del paso 2 eran prosa/índices, no
  código. Fila 10 (255/0, CABE exacto).
  ✅✅ **DIEZ DE DIEZ — CAMPAÑA DEL PASO 3 COMPLETA** (§143-§152, 47
  commits de campaña). Sigue: asiento de cierre + informe de
  traspaso.
  ✅ **CIERRE DE CAMPAÑA (§153)**: resumen en una página — hallazgos
  (burn 1024, audit un carril, freeze la CASA + ajuste exacto),
  variantes (titular ancho, LA COPIA), reglas del playbook, y lo que
  queda armado (paso 4, paso 5 §130, flip D4). El paso 3 se archiva.
  ✅ **Paso 4 COMPLETO — T2b-circuito (§154)**: la recuperación de
  §117 extremo a extremo — la clave sola produce prueba que VERIFICA
  (gemelo del piloto); el diccionario sin salt NO verifica; el nativo
  apunta a sus hermanos; la mitad de `apply` queda anotada para D4.
  SIGUE: paso 5 — medición apareada (§130).
  ✅ **Paso 5 COMPLETO — medición §130 apareada (§155)**: 20
  instrumentos ignored; corrida canónica en serie; burn ×2,04
  (predicción §145 clavada), frozen_climb gratis (§60.2 cerrado en
  cash), audit encoge; resto 0–9,5 %. Tabla en
  doc/medicion-130-apareada.md. **Los cinco pasos pre-flip de
  B13/B14, COMPLETOS.** SIGUE: el flip D4.
  ✅✅ **FLIP D4 HECHO (§156, `2ac37c5`)**: release única — gemelos
  sustituyen, legacy muere (−15.078 líneas), capa salted extremo a
  extremo, v7 + pines, marcador de geometría, museo en 24 local.
  Guardián 27; suites 316/0 y 240/2. SIGUE: **F3** (migración en vivo,
  los 2 rojos de índices, etiqueta 50) y **F4** (flecos censados).
  ✅✅✅ **F3 COMPLETO (§157, `e3116c6`+`f926de6`) — ETIQUETA
  `entrada-50` PUESTA**: colocación pid-mod, contratos de privacidad
  en verde, cinco fósiles enterrados, **CERO ROJOS** (316/0 · 242/0).
  La entrada 50 se CIERRA. SIGUE: **F4** (flecos) — y nada más.
  ✅✅✅✅ **F4 COMPLETO (§158, `a2f414d`→`af75c0a`)**: prosa al
  presente, jaulas cfg(test) a t2b/t2a (compilaban en la lib sin
  puerta), allows con puntero a las deudas de 49-A y la 32,
  **CERO·CERO** (warnings 0·0; suites 316/0 · 242/0). Retirada del
  museo REGISTRADA como futura (⚠️ los helpers nativos se mudan
  antes). **COLA: VACÍA.**

  ✅ **CERRADA (§266, 10-08-2026)**: **la nota ya declaraba su propio cierre
  y nadie marcó la casilla** — «La entrada 50 se CIERRA» tras F3 (§157,
  `e3116c6`+`f926de6`) y «**COLA: VACÍA**» tras F4 (§158, `a2f414d` →
  `af75c0a`). No se añade trabajo: el cierre estaba hecho **y escrito aquí
  dentro** desde entonces. Lo que faltaba era el corchete.
- [ ] **51. Tres `native_leaf` con dominios de identidad distintos.**
  §94: misma estructura, **distinta anchura** — `Digest` (256 bits) en
  `circuit_settlement`, `BaseElement` (64) en `compliance_circuit` y
  `double_entry`. Es la entrada 15 replicada, y explica por que solo se
  corrigio la auditada. ⚠️ **Compartir nombre invita a suponer que son la
  misma funcion.** No medido si las otras dos estan en algun camino de
  produccion.
  ⚠️ **§840: medido, y con su guarda; sigue abierta.** El censo de hoy: diez
  `fn native_leaf…` en `crates/`. La ancha, `native_leaf(public_id: Digest, …)`, y su
  `native_leaf_salted`, en `zk-ssl-hash/src/lib.rs`, que `stark-experiment` reexporta
  (`native.rs`): es la de produccion. Dos estrechas, con identidad `BaseElement`, en
  `stark-experiment`: `compliance_circuit.rs` y `double_entry.rs`. Y seis en los comparativos:
  cinco de halo2 (`Fp`) y una de plonk (`BlsScalar`). ⚠️ §840: decia «`Digest` (256 bits) en
  `circuit_settlement`»; hoy `circuit_settlement` (`zk-core`) no tiene ninguna, y la ancha es la
  de `zk-ssl-hash`. Ninguna estrecha esta en un camino de produccion: la de `compliance_circuit`
  la usan `compliance_real_proof`, `iso_bridge` y `settlement_prover_impl`, y el
  registro de nulificadores nombra ese modulo en sus tests; ningun fuente de `src/` de las cuatro
  raices de `tools/check_curvas.py` ni de los crates de la casa de su clausura nombra esos seis
  modulos, fuera de los ficheros solo de tests -el instrumento de D-B prueba `double_entry` a
  proposito-, y ninguno de los 28 modulos de `stark-experiment` que la produccion enlaza los
  nombra. Leido en los fuentes, no medido: la estrecha compone lo mismo que la ancha sobre
  `embeber(id)`; la diferencia es el dominio de la identidad, no la funcion. **La guarda**, dos
  tests de `crates/zk-ssl/src/censo_de_hojas.rs`: el censo, contra una lista escrita con la
  anchura de cada una, y que ninguna estrecha ni el modulo de quien la usa llegue a produccion.
  Cerrarla con la guarda, o dejarla abierta hasta retirar las estrechas, lo decide el autor.

- [ ] **52. ⚠️ CLAVE DE BOVEDA: un solo diseño de rotacion, o se hara tres
  veces (§134.1).** **Tres decisiones difirieron su coste aqui**: §117 —rotar
  implica hoja nueva—, §127.5 —la clave de vista **es irrotable sin rotar la
  de gasto**— y B18.2, cuyo reclamo de migracion **ES una rotacion**.
  ⚠️ **Y medio espacio de diseño ya existe**: `recover` **rota la clave CON
  custodios** (entrada 20) — el acoplamiento que hay que leer **antes** de
  proponer rotacion autoautorizada.
  ⚠️ **Restriccion estructural a leer primero**: la rotacion de custodios es
  **por uso, no por tiempo** —*«esta capa no tiene nocion de tiempo»*—, la
  misma razon por la que §119.3 anclo el reloj a **cabezas firmadas**. **La
  ausencia de tiempo ya condiciono dos diseños.**
  **Alcance real**: unificar los dos casos abiertos de la 20 + el reclamo de
  B18.2 + los tres costes diferidos. **Sesion propia.**
  ⚠️ **§791: el análisis, hecho y sin elegir.** `doc/ROTACION_DE_CUENTAS.md` (aceptada como
  análisis en el §799, con sus decisiones abiertas; ⚠️ §799: hasta aquí decía «(BORRADOR)») junta los
  casos en un núcleo común —la hoja vieja pasa a una con identidad, sal y vista nuevas, sin mover
  saldo— y cinco opciones con su coste medido en ficheros, circuitos y vectores: la recuperación
  completada, la rotación propia, la precomprometida (el patrón del RFC-0015), el traspaso sin
  código y el reclamo como rotación; más la gobernanza (G0 a G2) y el recifrado (K0 y K1), y nueve
  decisiones para el autor. Medido de paso: la subida de la recuperación impone la copia de la sal
  con una sola columna; el test que `recovery.rs:101` cita no existe; y el único reloj de los plazos
  es `log.len()`. Deducido de la lectura, sin ejecutar: una cuenta recuperada no pasaría la
  credencial de vista del nodo ni re-deriva su sal, que es lo que el B18.2 necesita; y un pendiente
  en vuelo hacia la identidad vieja no lo cobra la clave nueva. **Sigue abierta.**
  ⚠️ **§802: el pendiente en vuelo, medido; el robo, cerrado.** Lo que el §791 dedujo, medido: tras
  recuperar X→Y, la clave nueva genera la prueba de cobro de un pendiente hacia X y aplicarla da
  `StaleState`. Y lo que no vio: la clave VIEJA sí lo cobraba, abriendo otra cuenta con la identidad
  retirada X (medido en privado: cobró 7.000). Desde el §802 abrir rechaza una identidad que ya tiene
  cuenta o que una recuperación retiró (`IdentityUnavailable`; `SECURITY.md` §3.13), y el pendiente
  solo vuelve al emisor por reembolso al caducar, salvo un duplicado que la clave vieja abriera antes
  del §802 (el residuo de `SECURITY.md` §3.13). Para el diseño que se elija: **toda rotación que
  retire una identidad tiene que añadirla al mismo conjunto de retiradas, en el mismo lote**, o la
  identidad vieja vuelve a abrir; y la D-5 de `doc/ROTACION_DE_CUENTAS.md` se estrecha: la identidad
  vieja ya no puede seguir cobrando en una cuenta nueva, solo en la rotada si un diseño lo admite. La
  cita `recovery.rs:101` del §791 es hoy `:105`. **Sigue abierta.**
  ⚠️ §812: la D-4, resuelta por el autor —el recifrado, aparte del diseño único, como E4 del
  RFC-0001, construida—; las otras ocho decisiones siguen abiertas.
  ⚠️ **§860: las ocho decisiones, tomadas por delegación del autor («resuelvelas aplicando los
  principios»), con el criterio del §609 y el precedente del §642; REVERSIBLES; entran con su
  aceptación: UNA transición y varios autorizadores.** La transición es N1–N4 de
  `doc/ROTACION_DE_CUENTAS.md` §3: la hoja, en su posición, pasa a `(id', saldo, nonce + 1, sal')`,
  con el `view_id'` atado en el compromiso de la autorización y la identidad vieja a las retiradas
  en el mismo lote (§802). **D-1**: el robo, la pérdida y cualquier sospecha sobre `sk` van por los
  custodios (R1, completada: retira). La vista sola y la higiene sin sospecha van por el traspaso
  (R4, sin código), y se declara lo que no hace: no retira ni la clave ni la identidad viejas, que
  siguen cobrando y abriendo la vista; y agotado el tope de cuentas (la 8), no hay R4. **R3 se
  adopta en la sal, no en la hoja**: la cuenta que quiera abre con `sal = merge(sal de sk, H(DOM, sk
  siguiente))`, que las doce AIR solo leen como testigo. No toca la preimagen ni migra hojas, y
  responde al robo sin custodios. Se construye tras tres puertas: N1; que la 119 ate la sal sin
  imponer su origen —sin eso, el operador plantaría su propia siguiente al abrir—; y medidos el uso
  con sal compuesta y T2b con las dos claves. Si cae una, la higiene que retira pasa a R2. R2 no se
  construye; R3 solo en el registro se descarta. **D-2**: en su posición —a otra del mismo libro no:
  rompería «ninguna operación toca dos cuentas», que la 47, cerrada en el §860, deja escrito—; a otro
  libro, solo por el reclamo del B18.2. **D-3**: G0, la que cierra la 20. **D-5**: varados hasta el
  reembolso del emisor, y se declara. Y, DEDUCIDO y sin test, lo que una cuenta envió por v2 antes
  de rotar ya no le vuelve: su reembolso da `PendingMismatch`
  (`crates/zk-ssl/src/two_phase.rs:710-714`), y solo sale si el receptor lo cobra; falla cerrado.
  **D-6**: `recovery_count`, solo custodios. **D-7**: ninguna rotación saca a una cuenta de su
  posición, así que la congelada sigue congelada. **D-8**: N1, en el tren `zkssl/0.5` antes del
  corte, o en el siguiente. **D-9**: derivada de `sk`, con dominio propio y, en la vía simétrica del
  B11.2 (la 11), una por relación. **Sigue abierta** hasta N4; al cerrarla, R3 y el B18.2 pasan a
  una entrada propia si no se han construido.

- [x] **23. Consenso distribuido: NO se emprende en Arqueo — decidido y declarado (⚠️ §860).** No
  anade un problema nuevo: recupera el
  del doble gasto que se cerro, y con el el limite del cumpleanos, salvo
  que se indexe por el nullificador completo (§13, §32, §36).
  ⚠️ **§860: decidida por delegación del autor («resuelvelas aplicando los principios»), con el
  criterio del §609 y el precedente del §642; REVERSIBLE; entra con su aceptación. No se emprende
  —es una decisión, no una deuda—, y la entrada se CIERRA.** Arqueo no construye consenso
  distribuido, y `main` ya lo decía: `PRINCIPIOS.md` §8 lo pone en «Lo que requeriría otros
  recursos», fuera del alcance actual, y deja escrito que sin él la neutralidad y la soberanía no se
  cumplen; `PREGUNTAS.md` y `RESUMEN_EJECUTIVO.md` lo llaman «otra disciplina y no es el camino de
  este proyecto»; y `README.md` §«Qué garantiza y qué no» nombra la alternativa que sí se persigue:
  la responsabilidad demostrable, con cabezas firmadas, testigos, acuse y el ancla externa de la 86.
  La fórmula es la de la E4c del RFC-0006: «es una decisión, no una deuda».
  **En este punto el principio fundador sigue sin cumplirse**: `PRINCIPIOS.md` §0 y §8 quedan
  intactos, el operador sigue siendo el intermediario que nombra `PRINCIPIOS.md` §0, y nada pasa a
  «resuelto». El camino que se persigue **detecta la censura, no la previene** —antes del recibo no
  deja rastro (`SECURITY.md` §2)—, y `PRINCIPIOS.md` §2, que cuenta la censura entre lo que hace
  vulnerable a lo centralizado, apunta al otro lado: se dice. **Las razones**: un consenso reparte
  al que ordena, no lo quita (`SECURITY.md` §6); impondría validadores a cada libro cerrado de un
  operador, que es el caso medido (`doc/USE_CASES.md` §«Institutional clearing: four tests»: «one
  node, one writer»), lo contrario del criterio del §609; y su seguridad no tiene aquí test
  discriminante (`PRINCIPIOS.md` §9; `PREGUNTAS.md`: «un consenso mal implementado es más peligroso
  que ninguno»). **Por qué `[x]` y no ⏸️**: la 23 nació en `9670e76` como la única entrada del grupo
  «E. Otro proyecto, no una incidencia», y la 9 y la 13 se cerraron con su residuo vivo y declarado;
  la 16, suspendida, apunta al otro lado, y elegir entre las dos es convención, no principio: el
  contenido de la decisión es el mismo.
  **Para quien lo distribuya, en otro proyecto**: el encadenamiento de raíces exige orden total, y
  sin él vuelven el doble gasto y el límite del cumpleaños (`SECURITY.md` §3.4; §13, §32, §36): hará
  falta un identificador de gasto único indexado por su valor completo (la opción C del §13) u otro
  mecanismo de gasto único. B7 no necesita consenso para particionar, pero esta restricción le
  alcanza si la 47 se reabre (`doc/ESCALADO.md` §2.3). El failover automático elige quién escribe, y
  cae aquí. **Se reabre**, y solo por el autor, si dedica los otros recursos de `PRINCIPIOS.md` §8,
  o si un caso de `doc/USE_CASES.md` pide impedir —no solo detectar— el doble uso entre operadores
  que no se fían. Marcados en el mismo sello: «Lo demas», en la cabecera, y la fila B1 de
  `doc/ESCALADO.md` §10.

---

- [x] **77. XMSS: el KAT ausente, CERRADO (§623).** `tools/segunda/kat_xmss/` trae un corpus
  producido por `xmss 0.1.0-pre.0` desde semillas fijas y verificado por una implementación de RFC
  8391 escrita desde el RFC: 8/8, tres falsadores callados, partido como `xmss-kat-main` y publicado
  en <https://github.com/atoranzo/xmss-kat>. Queda vivo el issue upstream (lo pide el §575, con su
  borrador en `doc/reporte-winterfell-575.md`) y ARM. El cuerpo original, abajo.
  ⚠️ §861: el issue de esta entrada es `doc/issue-rustcrypto.md`; el que pide el §575, con su borrador
  en `doc/reporte-winterfell-575.md`, es el reporte a winterfell, otro, que salió en el §625.

- [ ] **77 (original). XMSS: lo que queda después de cerrar la 56 — KAT, el issue
  upstream y ARM.** Nace en §267 al cerrar la 56. La firma de cabezas está
  implementada y sus dos incógnitas resueltas, pero la lista de pendientes de
  la evaluación **estaba enterrada en el cuerpo de la 67** —87 líneas y tres
  asuntos— y §266 cerró esa nota. Aquí queda lo vivo, con su medida.
  ⚠️ **KAT ausente.** Buscando por estructura en los dos crates que usan
  `xmss` no aparece ningún vector de respuesta conocida. Lo que sale son los
  **vectores de conformidad de `zkssl/0.2`**, que son del protocolo y no del
  esquema de firma: **no sirven de KAT**. Sin uno, un cambio de versión del
  crate no tiene quien lo cace, y la versión está fijada exacta justamente
  porque no hay red debajo.
  ⚠️ **El issue upstream es un BORRADOR.** `doc/issue-rustcrypto.md` abre por
  «Issue draft — RustCrypto/signatures» y pide dos cosas concretas: que
  `SigningKey` exponga su índice, y los planes de estado de travesía BDS. **No
  consta que se haya presentado**, y comprobarlo es de fuera del árbol.
  ⚠️ **§710: su parte del OID ya está aguas arriba.** La reporta
  RustCrypto/signatures#1442, que no abrió el autor: según lo leído el
  28-09-2026 (`doc/ecst/VERIFICACION.md`, G10c), la abrió otra cuenta el
  10-09-2026. El borrador lo dice en una nota interna, que se quita antes de
  enviarlo, y corrige su cifra: no chocan cinco de los ocho conjuntos SHA2-256
  sino los ocho, y 21 de los 56 XMSS^MT, que `xmss` rechaza con `InvalidOid`.
  Si se envía, esa parte va como comentario en #1442, no como issue nuevo; del
  resto sigue sin constar envío. El §4.9 de los dos ECST sigue diciendo que
  esos 21 «cargan mal»: marcarlo es del autor.
  ⚠️ **§793: el borrador gana su parte 5, `getrandom` y WebAssembly.** `xmss`
  pide `rand 0.10` con sus features por defecto, y eso mete `getrandom 0.4.3`
  en la clausura del verificador, que no genera claves; en
  `wasm32-unknown-unknown` su fuente lo para con un `compile_error!`. Medido
  con `cargo tree -i getrandom@0.4.3 --locked --offline -p zk-ssl-verify -e
  normal`; no se ha compilado para ese objetivo. Una nota interna del
  borrador dice que el verificador lleva otro, `getrandom 0.2.17`, por la sal
  de `zk-ssl-air` (§535), que es de la casa y el issue no toca. Sigue sin
  enviar: enviarlo es del autor.
  ⚠️ **ARM sin medir.** `doc/xmss-evaluacion.md` avisa de que los 0,62 ms por
  hoja son **de esa máquina** y deja las medidas en ARM pendientes junto a B9;
  `doc/ESCALADO.md` tiene B9 como medición pendiente. **Va con la 47.**
  ⚠️ §860: ya no va con la 47 (cerrada, §860): B9 está en la 21; la firma en ARM se queda aquí.
  ✅ **Lo que ya NO está pendiente**, y por eso no entra aquí: el guardián del
  índice (`firma_indice.rs`, §111.1 y §234) y fijar la versión del crate,
  que es exacta en los dos que la usan.
  **Evidencia**: medido sobre `af9e786`, 10-08-2026.
  **Dónde vive**: con la 56 y la 53, donde acaben. La sección G está declarada
  **cajón** en §266 y ese reparto no lo toca este sello.
  ⚠️ **§861: decidida por delegación del autor («resuelvelas aplicando los principios»), con el
  criterio del §609 y el precedente del §642; REVERSIBLE; entra con su aceptación. Sigue abierta
  solo por un acto del autor: enviar el issue.** Decidir cabe en la delegación, porque esta entrada
  nombraba las dos decisiones como suyas —«enviarlo es del autor», «marcarlo es del autor»—;
  actuar fuera del árbol, no.
  (1) **El KAT ya no falta**: lo cerró el §623 (`tools/segunda/kat_xmss/`, juzgado en el canon
  por la implementación de RFC 8391), y el §786 le fijó la huella (el bloque del corpus KAT de
  `xmss` en el «3 duodecies» de `tools/canon.sh`; ⚠️ §855: decía `:723-741`, y el §855 añade
  líneas por encima).
  «KAT ausente», arriba, valía hasta entonces. Y la versión de `xmss` está fijada exacta en tres
  crates —`zk-ssl-cli`, `zk-ssl-node` y `zk-ssl-verify`— y en el generador del KAT, no «en los dos
  que la usan». La cabecera de la casilla cerrada, arriba, lleva su marca: el issue de esta entrada
  no es el que pedía el §575.
  (2) **El issue se envía, sin su parte 2**: las partes 1 (`index()` y `remaining()`), 3 (BDS), 4
  (`Clone`) y 5 (`rand`, `getrandom` y wasm) van en un issue NUEVO, con una línea que remite a
  #1442 y a su PR para el OID; conservan su número. No como otro comentario en #1442 —trata solo
  del OID, y el PR que dice cerrarla la cerrará al fusionarse, con lo que el resto quedaría
  enterrado—, ni un issue por parte: el borrador ya fija el orden —se pregunta y se ofrece el PR—,
  y el cierre por «sus cuatro partes» vale igual con uno que con cuatro. Según lo leído el
  15-10-2026 con WebFetch —un resumen, no la página en bruto: la API y el HTML de GitHub dan 403 a
  la sesión—, #1442 **sigue abierta, con un PR que dice cerrarla**, el #1468 («xmss: parse raw
  OIDs by the expected key kind», abierto el 15-10-2026, «Fixes #1442»), y lleva un comentario de
  la cuenta `atoranzo`, la dueña de `origin`, del 26-09-2026, con lo de `SigningKey` y los 21 de
  56: «si se envía, esa parte va como comentario en #1442» ya está hecho. G10c de
  `doc/ecst/VERIFICACION.md`, que la leyó sin comentarios el 28-09, no lo recoge, y sus propias
  fuentes de `hbs-state` ya daban el reporte por hecho («Reported in RustCrypto/signatures#1442»):
  lo que apoya que aquella lectura fue la fallida (DEDUCIDO). Confirmar que el comentario es suyo
  es del autor. El `master` de RustCrypto/signatures, leído en bruto el mismo día, está en la
  0.1.0-pre.1, sin publicar, y sigue sin `index()` ni `remaining()`, con `#[derive(Clone)]` en
  `SigningKey`, con `rand = "0.10"` sin `default-features = false` y con el `or_else` de
  `xmss.rs:21`; la parte 3 no se comprobó. crates.io no tiene más que la 0.1.0-pre.0. Si se publica
  una versión con el arreglo de #1468, el centinela `el_apano_del_oid_sigue_haciendo_falta` se
  pondrá en rojo, como está previsto. El borrador, `doc/issue-rustcrypto.md`, queda al día en el
  mismo sello que esta nota; enviarlo, después de mirar otra vez `master` ese día, es del autor.
  (3) **El §4.9 de `doc/ecst/ECST.md` y el de su traducción se marcan**: «cargan mal» pasa a «no se
  releen de sus propios bytes: `xmss` rechaza la lectura con `InvalidOid` y no devuelve la clave de
  otro conjunto», con **[MEDIDO por una sesión de Claude Code, no en la máquina del autor:
  `AUDITORIA.md` §710]**, como el propio informe marca lo que midió una sesión (`doc/ecst/ECST.md`
  §8.1): su leyenda da el [MEDIDO] a secas a lo que midió el autor. La base `71c5aad` lleva la misma
  `xmss 0.1.0-pre.0`. Marcarlo no acepta el ECST, y G10c no se toca.
  (4) **La firma XMSS en ARM se queda aquí**, y no va con B9: B9 es el banco de generación de
  pruebas del cliente en ARM de gama baja, y la firma es del nodo y del testigo, que también firma
  (el `Cofirmante`, `crates/zk-ssl-cli/src/witness.rs:3377-3380`). Se mide cuando el nodo o el
  testigo tengan un objetivo ARM declarado, o cuando B9 se corra en ARM (la 21), y entonces la misma
  tanda mide la firma. Una cifra en ARM es de otra máquina, y no se compara sin decirlo con los
  144,5 ms del §236. A la cadencia de 60 s (`crates/zk-ssl-node/src/latido.rs:72`), esos 144,5 ms
  dejan un margen de unas 415 veces: **deducido, no medido**. El arnés `tests/medidas.rs` vive en
  `~/eval-xmss`, fuera del árbol (la cabecera de `doc/xmss-evaluacion.md`).
  **Se cierra** cuando conste aquí el enlace del issue enviado, o cuando una versión publicada de
  `xmss` deje sin objeto sus cuatro partes; si se cierra así con ARM sin medir, ARM pasa a una
  entrada propia.

- [ ] **108. Una firma poscuántica de menos de 100 bytes, sin estado: evaluada,
  y NO sale de las piezas de la casa.** Nace el 30-09-2026 de una pregunta hecha
  con lo que la casa maneja —la firma de cabezas con estado, su guardián, los
  árboles de Merkle del acuse y del consumo, las pruebas STARK—: ¿se puede
  diseñar una firma resistente a cuánticos de **menos de 100 bytes**, sin perder
  seguridad y sin árboles de estado? `doc/firma-corta-evaluacion.md` la contesta
  **con medida, no con opinión**, y la respuesta es no:
  - **La aritmética del hash lo cierra** (§2 del doc): con elementos de 32 B caben
    tres, y la fórmula de WOTS+ del RFC 8391 —la misma que ata
    `el_ancho_del_indice_sale_del_conjunto_de_xmss` en el verificador— exige
    entonces `log2 w ≥ 86`, es decir **más de 2⁸⁵ hashes por cadena** para
    verificar. Bajar el resumen para esquivarlo es bajar a 64 bits poscuánticos.
  - **Lo único vivo por debajo de 100 B es `uov-Is`** (96 B, clave pública de
    66 KB, multivariante): la familia de Rainbow y GeMSS, y adoptarla revoca §106.
    SQIsign se queda en 200 B desde el 01-09-2026; Falcon en 666. **El listón no
    son los 2.420 B de ML-DSA-44**, que además es categoría 2, no 1.
  - **Los árboles de Merkle amortizan, no comprimen**: `32 · h` B por elemento,
    **1.024 B medidos** con `h = 32` en 74 vectores. Bajo 100 B solo con `h ≤ 3` y
    un verificador que ya tenga la cabeza: eso es un recibo bajo la firma de otro,
    no una firma, y es el árbol que la pregunta quería evitar.
  ⚠️ **Hallazgo colateral, y es de la 87**: su tabla daba «decenas de KB» a
  SLH-DSA. FIPS 205 dice 7.856 B (128s) y 16.224 B (192s), **menos que los
  18.469 B de la XMSS^MT que la casa firma hoy**, sin estado y con el mismo
  supuesto. Corregido en la 87 con tachado; el tiempo de firma **no está medido**.
  **Lo que deja, y por eso queda abierta**: (1) cotejar con el original cada
  cifra leída en espejo —la red de la sesión bloqueaba nist.gov, ietf.org y
  eprint—; (2) re-medir la 87 con SLH-DSA-SHA2-128s y 192s sobre el hardware
  real, tiempo de firma incluido, método de §263; (3) si alguien quiere el recibo
  de lote de `h = 3`, es un banco, no un diseño. **No antes que la 87.**
  **Dónde vive**: G, porque diseñar una firma nueva es otro proyecto —y este
  asiento dice que ni siquiera ése—.

## Lo que NO esta en esta lista

Porque esta construido y medido: conservacion del valor, autoridad de
gasto sin entregar claves, emision con umbral, congelacion impuesta en
circuito, revelacion selectiva en tres modos, registro encadenado de
transiciones, persistencia e instantaneas con verificacion de integridad,
y cifrado en reposo.

Y una advertencia que pertenece aqui tanto como los pendientes: **esta
lista no se termina.** Cada cosa que se cierre destapara otras — ha pasado
tres veces esta semana. El valor del proyecto no esta en llegar a cero
pendientes, sino en saber con precision que es y que no es.

- [x] **70. Triaje de las 14 medidas del articulo «el ultimo intermediario»
  (dinero cuantico).** Documentos externos (1.txt/2.txt) anclados a un
  articulo de prensa, NO al registro — describen la direccion del proyecto
  pero reflejan un estado de hace semanas. Colacionadas (sesion 2026-08-03);
  analisis completo en `doc/triaje-14-medidas.md`. Veredicto por grupos:
  ⚠️⚠️ **DOS revierten un asiento — NO implementar literalmente**:
  · Medida 3a («que `balance_of`/`nonce_of`/`public_id_of` exijan credencial
    o se eliminen») **revierte §129**: son accessors del OPERADOR (que ya ve
    saldos), 97 usos legitimos; la puerta correcta se **añade**
    (`account_view_authenticated`, 49-A paso 4), no se lisian las internas.
  · Medida 4b/14 («actualizar los preprints para que no mencionen la via
    retirada») **revierte §135**: los articulos publicados NO se editan, se
    corrigen por `ERRATA.md`. En docs VIVOS si; en preprints, por errata.
  ✅ **YA HECHAS o EN CURSO con mas rigor que la lista**: 3b (salt en hoja =
  50/B13-B14, disenada §132); 1/2/11 (custodios via B = 32/33, diseno cerrado
  §47/§51) ⚠️ con el matiz que la lista NO ve: migrar sin resolver antes el
  **orden estricto `IDX_B-IDX_A-1`** abre doble-uso de custodio (§51); 4a
  (`transfer()` ni esta `#[deprecated]` — su retirada ES la 32 entera).
  🟢 **NUEVAS y sin choque — candidatas a ejecutar**:
  · **Medida 9** (la mejor): tests de «que aprende cada participante»
    (operador/contraparte/tercero), extendiendo §16 — mecanismo, no
    narrativa; fortalece la tesis central.
  · Medidas 7/10/12/13 (documentales): seccion «relacion con el dinero
    cuantico» en docs vivos; interfaz de anclaje externo (**documento CREADO**: `doc/ANCLAJE_EXTERNO.md`, §174 — B10.6/B10.7 en el backlog de CONFIANZA)
    —conecta con el intermediario residual = orden+completitud que el acuse
    de §119/§121 ya ataca—; actualizar SECURITY.md/modelo de amenaza;
    documento de posicionamiento publico.
  **Accion**: ejecutar 9 y las documentales que no toquen articulos
  publicados; el resto ya esta hecho, en curso, o pendiente de su diseno.
  ✅ **Campaña A COMPLETA (§159, sello 63ca178)** — el puntero se
  cierra con el párrafo final del sello, calcado:
  **Del triaje quedan vivas**: la **9** («qué aprende cada
  participante», la mejor — Campaña C), ~~la **6**~~ (notas/UTXO — EXAMINADA y resuelta, §177: vecino cerrado por colocación hash, inmovilización es coste declarado con caducidad pendiente, consenso ortogonal; no exige rediseño; sesión
  propia), la **10** (interfaz de anclaje, conecta con el acuse), y el
  bloque **1/2/4a/11 = entradas 32/33 = Campaña B**, hoy reencuadrada
  por censo: la vía delegada EXISTE completa
  (`apply_*_delegated` ×4) y el orden estricto de §51 ya vive en los
  appliers — el trabajo es migrar ~229 llamadas de la vía antigua,
  amputar `transfer` y los `deprecated`, y medir después (11).
  ✅✅ **CERRADO ENTERO (§182, 05-08-2026)**: Campaña B ejecutada (32/33
  arriba), la **9** hecha (Campaña C, §173), la **10** hecha (§174,
  `doc/ANCLAJE_EXTERNO.md`), la **6** examinada contra tres ataques
  (§177), y las documentales **7/12/13** en el árbol (README «Dinero
  cuántico», SECURITY §1-§2, POSICIONAMIENTO refrescado). De 14:
  ejecutadas con más rigor, rechazadas con acta, o promovidas con examen.
  El triaje nació para ordenar y muere ordenado.

- [x] **71. FV-1: el CENSO DE CELDAS del guardián (grupo C, solidez).**
  Nace de la crítica externa de sub-restringimiento (§183): la suite prueba
  PUNTOS, no universales, y el guardián audita RANURAS, no CELDAS. **Qué
  verifica**: para cada circuito, cada columna × clase-de-fila (filas-hash,
  cada enlace, fila 0, segmentos — clases derivadas de las periódicas que
  el guardián ya parsea) tiene DUEÑO: transición activa, aserción, o
  **celda libre DECLARADA** (convención `// CELDAS_LIBRES:` que el guardián
  parseará; libre sin declarar = fallo; declarada con dueño = aviso rancio).
  **Caso-mutación obligatorio antes del verde**: una copia de
  `circuit_refund` con un `C_CAP` borrado debe hacer gritar «celda sin
  dueño». **Honestidad en su propia salida**: referenciada ≠ determinada —
  caza la clase empíricamente dominante (la celda que nadie mira), no la
  determinación semántica; complementa a los discriminantes. **Primer paso
  OBLIGADO**: leer entero `tools/check_constraint_layout.py` (600+ líneas)
  — se extiende lo leído, no lo adivinado. Diseño completo:
  `doc/VERIFICACION_FORMAL.md` §1. **Sesión propia.**
  ✅ **PROTOTIPO EJECUTADO (§184)**: `doc/fv/censo_celdas_prototipo.py`
  corre sobre `circuit_refund` — el cruce funciona, el caso-mutación NO se
  distingue en clases agregadas. **Requisito REAL medido**: el núcleo de
  FV-1 es el **intérprete de selectores periódicos** (derivar «clase de
  fila = filas donde el selector vale 1» leyendo
  `get_periodic_column_values` fila a fila), NO el injerto — ése es la parte
  fácil, ya prototipada. El prototipo vive en `doc/fv/`, no en `tools/`: no
  caza su mutante, luego no vigila (§42.5).
  ✅✅ **FRONTERA CRUZADA (§185)**: `doc/fv/interprete_selectores.py`
  deriva clases de fila del selector real y **CAZA el mutante** (C_CAP
  borrado → 4 capacidades sin dueño en clase «enlace»). Hallazgo que lo
  desbloqueó: una aserción vive en UNA fila → UNA clase; tratarla como
  cobertura universal borraba la distinción (clase de §72). **Núcleo
  RESUELTO sobre el circuito simple.** Queda: generalizar a los 28
  (selectores multi-ciclo, carriles duales, aserciones en filas no-cero) e
  injertar en el guardián. Concepto hecho; el resto es cobertura.
  ✅✅ **DOS CARRILES (§186)**: `doc/fv/interprete_dos_carriles.py` cubre
  `circuit_frozen_climb` (sano 0 huérfanas, mutante cazado) con bucle de
  carril + índices crudos + selector booleano + **seguimiento de aliases**.
  Hallazgo capital: un falso positivo (`COL_BIT` leído vía `let bit=next[]`)
  casi se disfraza de sub-restringimiento — un verificador que grita sobre
  código sano se autodestruye (§42.5/§137). Resuelto. Queda: selectores
  multi-ciclo (circuitos de cuentas) + injerto en el guardián.
  ✅✅ **CAPAS 1-2 DEL MAPA EJECUTADAS (§188-§189, 05-08-2026)**:
  `doc/fv/interprete_multiciclo.py` censa `circuit_mint_climb` (14 clases ·
  588 celdas-clase · 140 libres declaradas · 0 sin dueño · mutantes 2/2) y
  desde §189 es COMPUERTA del guardián (subprocess + política: MUERTA y
  MUERTA PERIODICA suben a graves; `CASO_188` sembrado en el autotest).
  Complementariedad MEDIDA en ambos sentidos: `C_SEG_LINK` borrado = censo
  ciego, redes graves; selector cambiado = redes ciegas, censo 5 sin dueño.
  La entrada SIGUE ABIERTA: queda extender censo+compuerta circuito a
  circuito (CUENTAS primero: `credit_climb`, `recovery_climb`, `mint`,
  `send`, `claim`), sin falso verde — la línea limpia nombra su alcance.
  ✅ **credit_climb CENSADO Y COMPUERTA (§191)**: 12 clases · 468
  celdas-clase · 83 libres · mutantes 2/2 · candidato→guardián (TRIPLE
  grave, exit 1).
  ✅ **recovery_climb CENSADO Y COMPUERTA (§192)**: 9 clases · 387
  celdas-clase · 68 libres · mutantes 2/2 · candidato→guardián (TRIPLE
  grave). Quedan: `mint`, `send`, `claim`.
  ✅ **mint (pagos) CENSADO Y COMPUERTA (§194)**: primera pieza nacida
  ✅ claim (§196): ESPEC `doc/fv/interprete_claim.py` (82) — la gemela invertida; COMPUERTA-CLAIM 21 clases · 1155 celdas-clase · 235 libres · 0 sin dueño · mutantes 2/2, limpia a la primera; núcleo v5 (gemelas al núcleo, paridad ×5, send 129→88); guardián v7 (SEXTO censo, 975); cazas ×3 (TRIPLE 4+4+CENSO · 1 vs 7 · selector 5). §39 anclado en el censo. LA 71 SE CIERRA: seis piezas, checkbox abajo.
  ESPEC — 21 clases · 1029 · 343 libres (0 rancias a la primera) ·
  mutantes 2/2 · candidato→guardián (CUARTA, ya fuera de *_climb).
  Núcleo v2 (alias-suma) y v3 (sin A ni B), paridad ×3 en cada paso.
  Quedan: `send`, `claim`.

- [x] **72. FV-2: spike SMT sobre `circuit_refund` (tras la 71).**
  Exportador en `tools/` del sistema (20 restricciones + 12 aserciones,
  selectores evaluados por fila) a SMT2 cuerpo-finito (cvc5 `--ff`, primo
  Goldilocks). Preguntas: determinación fijados los publics; consistencia
  de la cadena sin seguir Rescue. **Expectativa declarada de antemano**:
  grado 7 en un primo de 64 bits puede ser intratable — «TIMEOUT con estos
  parámetros» es un entregable VÁLIDO y se registra; el spike compra
  conocimiento del coste, no promete un verde. Si trata, la pregunta de
  escalar a `circuit_send` (51 columnas) se abre con datos; si no, se
  cierra con acta. Diseño: `doc/VERIFICACION_FORMAL.md` §2.
  ✅✅ **CERRADA CON ACTA (§190, 05-08-2026)**: q2 (cadena, abstracción-R)
  → unsat en 18-46 ms; q1/q1m (sistema séptico entero) → intratables por
  MEMORIA (6 GB en ~21 s, SIGABRT, modos gb y split idénticos); escalar a
  `circuit_send` cerrado con datos. Exportador en `doc/fv/` (mutante SMT
  sin consumar, §42.5); constantes por `examples/volcado_rescue.rs`.
- [x] **73. Refactor paramétrico de la estirpe FV-1 (núcleo + ESPECs).**
  Nació con datos: tres hermanos derivados por cortes y un cuarto sujeto
  (pagos) cuyo drift es de lógica (familias nuevas), no de forma.
  ✅✅ **EJECUTADA (§193, 06-08-2026)**: `censo_nucleo.py` (397) + tres
  ✅ send (§195): ESPEC `doc/fv/interprete_send.py` (129) sobre el núcleo v4; COMPUERTA-SEND 23 clases · 1288 celdas-clase · 258 libres · 0 sin dueño · mutantes 2/2; 13 declaraciones CELDAS_LIBRES en el .rs; guardián v6 (QUINTO censo, 961); cazas ×3 (TRIPLE 5+5+CENSO · 1 vs 10 · selector 8). Primera pieza fuera de la traza 512 (dos carriles, 1024×56). Queda: claim.
  intérpretes finos (62/49/49) con PARIDAD BYTE A BYTE contra los stdouts
  patrón-oro (mutantes incluidos); guardián intacto (933), autotest 3/3.

- [x] **74. Ecosistema RPC: de implementación a protocolo.**
  Nació en §197 con la Fase 0 cumplida: wire + spec (`zkssl/0.1`) + nodo
  de referencia + SDK + CLI, y el criterio probado — pago en dos fases
  SDK↔nodo sin que la clave de gasto viaje (verificado en código y en
  ejecución). Fuente: ROADMAP-ECOSISTEMA.md. Qué falta (Fase 1, para que
  existan SEGUNDAS implementaciones): OpenRPC generado desde
  `zk-ssl-wire` · vectores de conformidad POR VERSIÓN de circuito (el
  determinismo por operación ya está medido: raíces idénticas CLI↔RPC
  con la misma semilla) · proceso RFC del protocolo · keystore cifrado.
  Deudas heredadas DECLARADAS: el aviso viaja fuera de banda (§21) ·
  nodo único (el operador ordena y puede censurar) · `--dev` usa
  custodios de PRUEBA (producción exige raíces reales por flags, marcado
  en `zk-ssl-node/src/main.rs`).
  ✅ Fase 0 (§197): 4 crates + `spec/RPC.md`; e2e con claves aleatorias
  (750000/250000, `verifyChain` ok:true); tamaño real de prueba medido y
  la spec corregida (54–66 KB).
  ✅ Fase 1a (§198): `spec/openrpc.json` GENERADO desde la tabla de `zk-ssl-wire` (17 métodos, tests 2/2; regenerar reproduce BYTE A BYTE — compuerta del BLOQUE) · vectores `spec/vectors/zkssl-0.1.json` (escenario canónico, 6 entradas; `conformance --check` los reproduce — determinismo probado ×2 y ahora COMPUERTA permanente) · proceso RFC en `spec/rfc/`.
  ✅✅ **CERRADA (§199, 06-08-2026)** con el keystore: `zk-ssl-sdk/src/keystore.rs` (217) — la MISMA construcción de reposo que el ledger con dominio propio `ZK-SSL-keystore-v1`; test que EXIGE que la clave del ledger NO abra el keystore; binding public_id declarado==derivado; cifrado autenticado; 0600 en Unix; 5 tests + ejemplo vivo, verdes a la primera. De implementación a protocolo: spec + OpenRPC + vectores-compuerta + RFC + wallet en reposo. Deudas declaradas (§21 fuera de banda, nodo único, --dev) siguen escritas con su cauce.