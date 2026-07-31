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

**Estado**: 19 abiertas, 24 resueltas. Ultima revision: 31 de julio de 2026.

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

- [x] **2. Cifras del repositorio.** ~~65 de 174 en depuracion, 174 tests
  de la capa, doce circuitos, y la contradiccion 56/65 dentro de
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

- [ ] **32. ⚠️ Las claves de custodio SI llegan al operador.** No hay via
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

- [ ] **7. ⚠️ Encargar la auditoria externa.** Ya no es solo por el
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
  cualquier piloto.

- [ ] **12. Sin devolucion para un pendiente no cobrado.** El importe queda
  inmovilizado y no hay camino de vuelta implementado.

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

- [ ] **15. Goldilocks es estrecho para identidades.** 64 bits son colision
  en 2³².

- [ ] **16. Referencias cruzadas de los preprints.** Los tres citan
  versiones anteriores de sus companeros; primera cosa de la cuarta
  revision.

## E. Operacion

- [ ] **17. Replica y alta disponibilidad.** El nodo es punto unico de
  fallo.

- [ ] **18. Bloqueo de directorio de `sled`.** Puede impedir un reinicio
  inmediato tras cerrar (§16.6).

- [ ] **19. Sin log de escritura anticipada.** Un fallo entre operaciones
  detiene el arranque pidiendo intervencion manual: correcto, pero no
  automatico.

- [ ] **20. Rotacion de claves operativas.** Implementada solo en parte.

- [ ] **21. Delegacion de prueba a clientes ligeros.** Exige verificar una
  firma dentro del circuito: **~8.000 filas** con esquema Winternitz,
  estimacion documentada en `client.rs` (§18). ⚠️ Esta cifra se retiro por
  error en §42.3 dandola por inventada, y se **restituye** (§42.5): estaba
  en el codigo, se busco donde no estaba. Mismo primitivo que la 33.

- [ ] **22. Agregacion de pruebas.** 120,4 MB por mil pagos es coste, no
  parada, pero crece linealmente.

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

- [ ] **28. Corregir los tres preprints tras la 27.** Describen el cobro
  como demostracion de titularidad. No tocar Zenodo hasta que 27 este
  corregida y verificada.

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

- [ ] **39. La cadena de columnas PERIODICAS no la comprueba nada.**
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

- [ ] **41. Doce fallos de depuracion sin diagnosticar.**
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
