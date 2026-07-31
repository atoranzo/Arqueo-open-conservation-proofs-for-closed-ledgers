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

**Estado**: 24 abiertas, 27 resueltas. Ultima revision: 31 de julio de 2026.

⚠️ **La 41 no se cierra hoy, y eso es deliberado.** Se clasificaron sus 80
fallos y aparecieron **dos que no eran la clase conocida** (§78). Los 78
restantes tienen causa decidida pero **no medida uno a uno**, y marcarlos en
bloque seria el error que §77 acaba de desmentir.

⚠️ **El 31-07 aparecio la 43 leyendo codigo para otra cosa**: la capa no
verificaba las pruebas de la via de pago, y un tercero vaciaba cualquier
cuenta sin clave. Corregida el mismo dia. Queda `[x]` porque esta cerrada,
pero se deja arriba: es el fallo mas grave que ha encontrado esta auditoria
y ninguna herramienta del proyecto podia verlo.

⚠️ **La 40 dejo de ser hipotesis.** Se abrio por la mañana como lectura sin
medir y se confirmo por la tarde con testigo en release: es el **cuarto
fallo de solidez** de la auditoria (§72). Sigue abierta —confirmar no es
corregir— y es hoy la entrada de mas peso de esta lista.

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
fallos de solidez y una brecha de confianza sin declarar.

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

- [ ] **32. ⚠️ Las claves de custodio SI llegan al operador.**
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
  ~~Original:~~ No hay via
  cliente para operaciones privilegiadas: `ThresholdAuth` lleva las claves
  en crudo y la capa construye la traza (§41). Quienes conservan su clave
  solo pueden mover su dinero; quienes la entregan pueden crearlo.
  **Confianza residual no declarada** en los preprints: va con la 28.
  **Diseño de la solucion cerrado en §47** (via B); la implementacion es la
  33.

- [ ] **33. Que los custodios prueben en su maquina, y que la autorizacion
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

## D. Declaradas, acotadas, sin urgencia

- [ ] **8. `open_account` sin autorizacion.** El tope de cuentas mitiga a
  medias: un atacante aun puede agotar el cupo, y la solucion correcta
  exige un circuito nuevo (§16.1).

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

- [ ] **12. ⚠️ FONDOS MUERTOS: tres fuentes y ningun camino de vuelta.**
  ~~Sin devolucion para un pendiente no cobrado: el importe queda
  inmovilizado.~~ **Ampliada** el 31-07-2026 (§87). **Respaldo**: §30, §29, y
  el test `sending_to_a_nonexistent_recipient_loses_the_money`.
  ⚠️ **Son TRES fuentes, no una** (§87.1): el receptor no cobra; el
  destinatario no existe; y —la que lo hace **sistemico**— el destinatario
  **esta congelado**, porque enviar a una congelada funciona pero cobrar no
  (§29). **El sistema genera dinero irrecuperable operando como debe.**
  ⚠️ **Dos obstaculos concretos**: (1) la capa **no tiene nocion de tiempo**
  —`lib.rs`, y por eso la rotacion se conto en usos—, asi que una caducidad
  debe expresarse en usos, altura u operacion explicita (§87.2); (2)
  **`pending_commitment(receiver_id, salt, amount)` NO registra al emisor**,
  asi que la capa **no sabe a quien devolver**, y ese dato falta **por
  privacidad, no por descuido** (§87.3).
  **Lo que una solucion debe probar** (§87.4): que vencio el plazo, que
  sigue sin cobrar, y que quien recupera es quien pago **sin revelar el
  vinculo pagador↔pendiente**. El tercero exige un compromiso con dos
  identidades y un circuito que no existe.
  ⚠️ **Y tiene un sub-caso peligroso** (§87.5): un mecanismo que recupere
  pendientes de cuentas congeladas **puede usarse para vaciar a un
  congelado**. Mirarlo al diseñar, no despues.
  ✅ **MECANISMO PROPUESTO Y EVALUADO** (§88): reclamo por reversion que
  prueba en cero conocimiento ser el emisor, que no se cobro y que vencio el
  plazo. **(b) funciona hoy** y **(c) tiene las dos piezas** —`log.rs` lleva
  `seq` monotona y comprometida en `head()`, y la comparacion por rango es la
  del tope de emision—, pero **`seq` no es entrada publica de ningun circuito
  todavia**. ⚠️ **(a) no es un circuito que falte: es un dato que no existe**
  —el compromiso no codifica al emisor—, asi que exige **cambio de formato**,
  de la clase de la entrada 15 (§88.3). **Estado: mecanismo propuesto y
  evaluado; requiere cambio de formato del compromiso, `seq` como entrada
  publica, un circuito nuevo, y una DECISION DE POLITICA sin resolver.**
  ⚠️ La cuarta pieza **no se implementa**: un plazo tiene victimas legitimas.
  Espacio de diseño registrado en §88.5 —plazo extensible por el receptor, o
  asimetria de plazos con precedente en la regulacion de cuentas inactivas—.
  **La invariante global SE CUMPLE**: dinero perdido con la contabilidad
  cuadrada.

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

- [ ] **15. ⚠️ El ESPACIO DE CLAVES es de 64 bits. Medido, e INVENTARIADO el
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
  **Quedan, en este orden**: **`open_account` y la capa PRIMERO** —migrar mas
  circuitos no da un bit de seguridad mas mientras la puerta de entrada sea
  estrecha—, y despues `claim` y `audit`, que **no encaja en el patron**:
  `COL_KEY` en 13, ancho 24 y **cero** sitios `state[8] = spend_key`.
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

- [ ] **16. Referencias cruzadas de los preprints.** Los tres citan
  versiones anteriores de sus companeros; primera cosa de la cuarta
  revision. **Comprobable** contra los propios ficheros de `doc/`, sin
  necesidad de seccion. ⚠️ **La cuarta revision ya no es solo esto**: se le
  han acumulado la 28 —el cobro—, la 15 —el espacio de claves de 64 bits,
  que §82.4 muestra que el paper no menciona pese a usar el criterio de los
  128 bits— y la unidad MiB/MB de la 22.

## E. Operacion

- [ ] **17. Replica y alta disponibilidad.** **Comprobable**: `grep -rn
  "replica\|cluster\|raft" crates/zk-ssl/src/` no devuelve nada.
  ⚠️ **Constatacion de arquitectura, no hallazgo**: no hay seccion porque no
  hay nada medido que registrar, y va con la entrada 23. El nodo es punto
  unico de
  fallo.

- [ ] **18. Bloqueo de directorio de `sled`.** Puede impedir un reinicio
  inmediato tras cerrar (§16.6). ⚠️ **Manifestacion MEDIDA** el 31-07-2026
  (§79): hizo fallar un test de release **1 de cada 12 pasadas** a 16 hilos,
  con `WouldBlock` al reabrir. Los tests quedan protegidos con
  `open_encrypted_retry` (entrada 45), pero **el fondo sigue abierto**: un
  nodo real que se reinicie inmediatamente tras cerrarse puede sufrir lo
  mismo, y ahi no hay reintento de test que valga. Deja de ser una
  limitacion teorica.

- [ ] **19. Sin log de escritura anticipada.** **Comprobable**: `grep -rn
  "write_ahead\|journal" crates/zk-ssl/src/` no devuelve nada. Un fallo
  entre operaciones
  detiene el arranque pidiendo intervencion manual: correcto, pero no
  automatico.

- [ ] **20. Rotacion de claves: DOS de cuatro casos, y uno no es un hueco.**
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
  - ❌ **Clave de cifrado del ledger**: **no hay recifrado**. Comprobable:
    `grep -rn "rekey\|re_encrypt" crates/zk-ssl/src/` no devuelve nada.
    Cambiar la contraseña exige reconstruir el ledger.
  - ⚠️ **Gobernanza**: `governance_set_root` **solo se fija en el
    constructor** y no tiene modificador —comprobable: `grep -n
    "governance_set_root" crates/zk-ssl/src/lib.rs` da lectura y
    constructor—. **Es inmutable por diseño, no un hueco**: si se pudiera
    rotar, quien controlara el conjunto nuevo podria cambiar los custodios
    (test `the_governance_set_survives_restart`). El coste es que **una
    clave de gobernanza comprometida lo esta para siempre**, y eso si merece
    decision propia.

- [ ] **47. `ESCALADO.md`: propuesta de sharding, sin integrar.**
  Documento externo del 31-07-2026 que dimensiona 5 × 10⁹ usuarios con el
  diseño casi intacto. ✅ **Su observacion central se sostiene**: ninguna
  operacion toca dos cuentas, luego **no hay transacciones cross-shard**, y
  el coste declarado de las dos fases —el pendiente inmovilizado, §29-§30—
  es exactamente lo que compra shardabilidad. ⚠️ **Pero presentaba como
  medido el coste de verificacion (4 ms) sin estarlo** (§89). Medido: **2,35
  ms**, asi que el documento era conservador por 1,7 y **37 shards, no 64**.
  **Requiere cuatro correcciones antes de integrarse** (§89.4), y
  **reconciliar sus B1-B9 contra este backlog**: su B8 es la entrada 12 y
  otros solapan. Anexarlo sin reconciliar duplicaria entradas.

- [ ] **21. Delegacion de prueba a clientes ligeros.** Exige verificar una
  firma dentro del circuito: **~8.000 filas** con esquema Winternitz,
  estimacion documentada en `client.rs` (§18). ⚠️ Esta cifra se retiro por
  error en §42.3 dandola por inventada, y se **restituye** (§42.5): estaba
  en el codigo, se busco donde no estaba. Mismo primitivo que la 33.

- [ ] **22. Agregacion de pruebas.** 120,4 MB por mil pagos es coste, no
  parada, pero crece linealmente. **Respaldo**: §31 explica por que la cifra
  paso de 59,1 a 120,4 —un pago son DOS pruebas desde la via en dos fases— y
  la guarda `metrics::tests::cost_per_transfer_stays_stable` la vigila con
  margenes anchos a proposito, **para detectar un cambio de orden de
  magnitud, no el byte exacto**. Ya salto una vez, en esa migracion, y tenia
  razon. Medido el 31-07-2026: **123,0 por mil**, dentro de la guarda.
  ⚠️ **Pero la UNIDAD esta mal etiquetada** (§83.3): el arnes divide entre
  `1_048_576` —MiB— y lo imprime como «MB», y esa etiqueta llega a
  `PAPER.md`, `PAPER_EN.md` y `QUESTIONS.md`. Un lector que tome MB = 10⁶
  leera **7,2 % menos** de lo real: son **129,0 MB**. Va con la 16 y la 28.

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

- [ ] **28. Corregir los tres preprints: INVENTARIADO, decision tomada.**
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

- [ ] **41. Ochenta fallos de depuracion: CLASIFICADOS, dos corregidos.**
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

- [ ] **42. `mint_to_pending` diverge de `mint` en el error del tope.**
  **Respaldo**: §73.4, donde se decidio no tocar la via antigua al corregir
  la 43 porque cambiar su error cambia lo que ven sus tests.
  La via antigua devuelve `OverRegulatoryLimit` —el error del limite
  regulatorio de una **transferencia**— para una violacion del **tope de
  emision**, donde `mint` siempre devolvio `SupplyCapExceeded`. Y suma sin
  saturar (`self.total_supply + amount`), que en depuracion desborda con
  panico. `apply_mint_pending_delegated` usa `SupplyCapExceeded` y
  `saturating_add`; la antigua **no se toco**, porque cambiar su error
  cambia lo que ven sus tests. Menor, acotado, declarado.

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

- [ ] **48. `CONFIANZA_RESIDUAL.md`: la evidencia contra el operador esta en
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

- [ ] **49. ⚠️ El contrato de `account_view` no exige autorizacion.**
  Medido el 31-07-2026 (§93.1): toma un indice, **no recibe credencial**, y
  devuelve `balance` y `nonce` de cualquier cuenta. Los indices son
  **enumerables** —`next_index += 1`—. ⚠️ **Hallazgo de CONTRATO, no de
  explotacion**: no hay capa de red en el repositorio, asi que «cualquiera
  vuelca el ledger» seria una cifra rancia hacia el drama. Lo cierto:
  **exponerlo sin control de acceso convierte una fuga hacia-el-operador —ya
  declarada— en una fuga hacia-terceros que el README no declara.**
  Arreglo barato —autorizacion en el contrato— y **no cierra la 50**.

- [ ] **50. ⚠️ PRIVACIDAD FRENTE A TERCEROS ROTA: la hoja no lleva salt.**
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
  cuenta—. ⚠️ **El obstaculo real no es el nonce** —el circuito ya usa
  nonces distintos por carril— **sino la asimetria emisor/receptor**
  (§93.5): con salt derivado de clave, el emisor no puede computar la hoja
  nueva del receptor. **Ninguna salida medida.**

- [ ] **51. Tres `native_leaf` con dominios de identidad distintos.**
  §94: misma estructura, **distinta anchura** — `Digest` (256 bits) en
  `circuit_settlement`, `BaseElement` (64) en `compliance_circuit` y
  `double_entry`. Es la entrada 15 replicada, y explica por que solo se
  corrigio la auditada. ⚠️ **Compartir nombre invita a suponer que son la
  misma funcion.** No medido si las otras dos estan en algun camino de
  produccion.

- [ ] **23. Consenso distribuido.** No anade un problema nuevo: recupera el
  del doble gasto que se cerro, y con el el limite del cumpleanos, salvo
  que se indexe por el nullificador completo (§13, §32, §36).

---

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
